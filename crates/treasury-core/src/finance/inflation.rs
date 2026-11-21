//! Deflação por índices mensais (IPCA, US CPI) ou pela expansão monetária (M2).
//! Dentro do mês, a variação é distribuída de forma geométrica pró-rata pelos dias corridos.

use chrono::{Datelike, NaiveDate};
use rust_decimal::prelude::*;

/// Variação mensal de um índice (ex.: IPCA de out/2026 = 0,56%).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonthlyRate {
    pub year: i32,
    pub month: u32,
    /// Variação no mês, em % (0,56 = 0,56%).
    pub pct: Decimal,
    /// Projeção (Focus) ou repetição do último dado: ainda não oficial.
    pub estimated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Factor {
    pub factor: Decimal,
    pub estimated: bool,
}

fn days_in_month(y: i32, m: u32) -> u32 {
    crate::time::days_in_month(y, m)
}

/// Fator acumulado do índice entre `from` (inclusive) e `to` (exclusive).
/// `None` se faltar algum mês do intervalo na série.
pub fn accumulated_factor(
    series: &[MonthlyRate],
    from: NaiveDate,
    to: NaiveDate,
) -> Option<Factor> {
    if to <= from {
        return Some(Factor {
            factor: Decimal::ONE,
            estimated: false,
        });
    }
    let mut factor = Decimal::ONE;
    let mut estimated = false;
    let (mut y, mut m) = (from.year(), from.month());
    loop {
        let first = NaiveDate::from_ymd_opt(y, m, 1)?;
        if first >= to {
            break;
        }
        let dim = days_in_month(y, m);
        let next = crate::time::add_months(first, 1);
        let start = from.max(first);
        let end = to.min(next);
        let days = (end - start).num_days();
        if days > 0 {
            let rate = series.iter().find(|r| r.year == y && r.month == m)?;
            estimated |= rate.estimated;
            let base = Decimal::ONE + rate.pct / Decimal::ONE_HUNDRED;
            let f = if days as u32 == dim {
                base
            } else {
                super::pow_dec(base, Decimal::from(days) / Decimal::from(dim))
            };
            factor *= f;
        }
        (y, m) = (next.year(), next.month());
    }
    Some(Factor { factor, estimated })
}

/// Fator a partir de um nível (estoque) como o M2: nível(t₁) ÷ nível(t₀), com
/// interpolação geométrica entre fechamentos mensais.
pub fn level_factor(
    levels: &[(NaiveDate, Decimal)],
    from: NaiveDate,
    to: NaiveDate,
) -> Option<Decimal> {
    let at = |d: NaiveDate| -> Option<Decimal> {
        let mut v: Vec<&(NaiveDate, Decimal)> = levels.iter().collect();
        v.sort_by_key(|(dd, _)| *dd);
        let after = v.iter().position(|(dd, _)| *dd >= d)?;
        let (d1, l1) = *v[after];
        if d1 == d || after == 0 {
            return (d1 == d).then_some(l1);
        }
        let (d0, l0) = *v[after - 1];
        let t = Decimal::from((d - d0).num_days()) / Decimal::from((d1 - d0).num_days());
        Some(l0 * super::pow_dec(l1 / l0, t))
    };
    Some(at(to)? / at(from)?)
}

/// Converte um valor de `from` para moeda de `to` (centavos, arredondado).
pub fn adjust(amount_cents: i64, factor: Decimal) -> i64 {
    (Decimal::from(amount_cents) * factor)
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn series() -> Vec<MonthlyRate> {
        vec![
            MonthlyRate {
                year: 2026,
                month: 1,
                pct: dec!(0.50),
                estimated: false,
            },
            MonthlyRate {
                year: 2026,
                month: 2,
                pct: dec!(1.00),
                estimated: false,
            },
            MonthlyRate {
                year: 2026,
                month: 3,
                pct: dec!(0.20),
                estimated: true,
            },
        ]
    }

    #[test]
    fn whole_months_compound() {
        let f = accumulated_factor(&series(), d("2026-01-01"), d("2026-03-01")).unwrap();
        assert_eq!(f.factor, dec!(1.005) * dec!(1.01));
        assert!(!f.estimated);
    }

    #[test]
    fn partial_months_pro_rata_and_estimated_flag() {
        let f = accumulated_factor(&series(), d("2026-01-16"), d("2026-03-16")).unwrap();
        let expected = super::super::pow_dec(dec!(1.005), dec!(16) / dec!(31))
            * dec!(1.01)
            * super::super::pow_dec(dec!(1.002), dec!(15) / dec!(31));
        assert!((f.factor - expected).abs() < dec!(0.0000000001));
        assert!(f.estimated);
        // Month missing from the series.
        assert!(accumulated_factor(&series(), d("2026-01-01"), d("2026-05-01")).is_none());
        assert_eq!(
            accumulated_factor(&series(), d("2026-02-01"), d("2026-02-01"))
                .unwrap()
                .factor,
            Decimal::ONE
        );
    }

    #[test]
    fn half_month_split_is_consistent() {
        // Splitting the interval does not change the total factor.
        let a = accumulated_factor(&series(), d("2026-01-10"), d("2026-02-20"))
            .unwrap()
            .factor;
        let b = accumulated_factor(&series(), d("2026-01-10"), d("2026-01-25"))
            .unwrap()
            .factor
            * accumulated_factor(&series(), d("2026-01-25"), d("2026-02-20"))
                .unwrap()
                .factor;
        assert!((a - b).abs() < dec!(0.0000000001));
    }

    #[test]
    fn m2_levels_and_adjust() {
        let levels = vec![(d("2026-01-31"), dec!(1000)), (d("2026-02-28"), dec!(1010))];
        assert_eq!(
            level_factor(&levels, d("2026-01-31"), d("2026-02-28")).unwrap(),
            dec!(1.01)
        );
        let mid = level_factor(&levels, d("2026-01-31"), d("2026-02-14")).unwrap();
        assert!(mid > Decimal::ONE && mid < dec!(1.01));
        assert!(level_factor(&levels, d("2025-12-01"), d("2026-02-28")).is_none());
        assert_eq!(adjust(10000, dec!(1.0456)), 10456);
    }
}
