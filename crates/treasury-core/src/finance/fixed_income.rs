//! Renda fixa marcada na curva (dias úteis, base 252) e estimativa de resgate líquido
//! (IR regressivo e IOF). Isto é estimativa de resgate, não a apuração de IR (fase 3).

use chrono::NaiveDate;
use rust_decimal::prelude::*;

use super::calendar::{business_days_between, is_business_day};

/// Fator de um dia de CDI: `1 + taxa_dia% × percentual%` (a série 12 do SGS já traz a
/// taxa diária em % a.d.). Ex.: CDI 0,055131% a.d. a 110% → 1,000606441.
pub fn cdi_day_factor(cdi_daily_pct: Decimal, pct_of_cdi: Decimal) -> Decimal {
    let f = Decimal::ONE + cdi_daily_pct / Decimal::ONE_HUNDRED * pct_of_cdi / Decimal::ONE_HUNDRED;
    f.round_dp_with_strategy(16, RoundingStrategy::ToZero)
}

/// Taxa diária equivalente (em % a.d., 8 casas) a uma taxa anual base 252.
pub fn daily_from_annual_252(annual_pct: Decimal) -> Decimal {
    let base = Decimal::ONE + annual_pct / Decimal::ONE_HUNDRED;
    let d = super::pow_dec(base, Decimal::ONE / Decimal::from(252)) - Decimal::ONE;
    (d * Decimal::ONE_HUNDRED).round_dp(8)
}

/// Acumula o CDI em dias úteis do intervalo (início, fim]. `rates` é a série diária
/// (data, % a.d.); dias úteis sem taxa usam a última conhecida e marcam `estimated`.
pub fn accrue_cdi(
    principal: Decimal,
    rates: &[(NaiveDate, Decimal)],
    pct_of_cdi: Decimal,
    start: NaiveDate,
    end: NaiveDate,
) -> (Decimal, bool) {
    let mut sorted: Vec<(NaiveDate, Decimal)> = rates.to_vec();
    sorted.sort_by_key(|(d, _)| *d);
    let mut factor = Decimal::ONE;
    let mut estimated = false;
    let mut last: Option<Decimal> = sorted
        .iter()
        .rev()
        .find(|(d, _)| *d <= start)
        .map(|(_, r)| *r);
    let mut d = start;
    let mut idx = sorted.partition_point(|(rd, _)| *rd <= start);
    while d < end {
        d = d.succ_opt().expect("data válida");
        if !is_business_day(d) {
            continue;
        }
        while idx < sorted.len() && sorted[idx].0 < d {
            last = Some(sorted[idx].1);
            idx += 1;
        }
        let rate = if idx < sorted.len() && sorted[idx].0 == d {
            let r = sorted[idx].1;
            last = Some(r);
            idx += 1;
            r
        } else {
            estimated = true;
            match last {
                Some(r) => r,
                None => continue,
            }
        };
        factor = (factor * cdi_day_factor(rate, pct_of_cdi))
            .round_dp_with_strategy(16, RoundingStrategy::ToZero);
    }
    (principal * factor, estimated)
}

/// Prefixado: P × (1 + taxa)^(du/252).
pub fn prefixed_value(
    principal: Decimal,
    annual_pct: Decimal,
    start: NaiveDate,
    end: NaiveDate,
) -> Decimal {
    let du = business_days_between(start, end).max(0);
    let base = Decimal::ONE + annual_pct / Decimal::ONE_HUNDRED;
    principal * super::pow_dec(base, Decimal::from(du) / Decimal::from(252))
}

/// IPCA + taxa real: P × fator do IPCA no período × (1 + taxa)^(du/252).
pub fn ipca_plus_value(
    principal: Decimal,
    real_pct: Decimal,
    ipca_factor: Decimal,
    start: NaiveDate,
    end: NaiveDate,
) -> Decimal {
    prefixed_value(principal * ipca_factor, real_pct, start, end)
}

/// Alíquota de IR regressivo por dias corridos de aplicação.
pub fn ir_rate(days_held: i64) -> Decimal {
    match days_held {
        ..=180 => Decimal::new(225, 3),
        181..=360 => Decimal::new(200, 3),
        361..=720 => Decimal::new(175, 3),
        _ => Decimal::new(150, 3),
    }
}

/// Alíquota de IOF regressivo sobre o rendimento (dias 1–29; zero a partir do 30º).
pub fn iof_rate(days_held: i64) -> Decimal {
    const TABLE: [u32; 30] = [
        96, 93, 90, 86, 83, 80, 76, 73, 70, 66, 63, 60, 56, 53, 50, 46, 43, 40, 36, 33, 30, 26, 23,
        20, 16, 13, 10, 6, 3, 0,
    ];
    if days_held <= 0 {
        return Decimal::ONE;
    }
    match usize::try_from(days_held - 1)
        .ok()
        .and_then(|i| TABLE.get(i))
    {
        Some(p) => Decimal::new(*p as i64, 2),
        None => Decimal::ZERO,
    }
}

/// Arredonda para centavos, metade para longe do zero.
pub fn round_money(v: Decimal) -> Decimal {
    v.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redemption {
    pub gross: Decimal,
    pub income: Decimal,
    pub iof: Decimal,
    pub ir: Decimal,
    pub net: Decimal,
}

/// Resgate líquido estimado. `ir_exempt` para LCI, LCA, CRI, CRA, debêntures incentivadas
/// (que também são isentas de IOF para pessoa física).
pub fn net_redemption(
    invested: Decimal,
    gross: Decimal,
    days_held: i64,
    ir_exempt: bool,
) -> Redemption {
    let income = (gross - invested).max(Decimal::ZERO);
    let iof = if ir_exempt {
        Decimal::ZERO
    } else {
        round_money(income * iof_rate(days_held))
    };
    let ir = if ir_exempt {
        Decimal::ZERO
    } else {
        round_money((income - iof) * ir_rate(days_held))
    };
    Redemption {
        gross,
        income,
        iof,
        ir,
        net: gross - iof - ir,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn cdi_day_factor_reference() {
        assert_eq!(cdi_day_factor(dec!(0.055131), dec!(100)), dec!(1.00055131));
        assert_eq!(cdi_day_factor(dec!(0.055131), dec!(110)), dec!(1.000606441));
    }

    #[test]
    fn daily_from_annual() {
        // 14,90% a.a. ≈ 0,05513% a.d. (base 252)
        let r = daily_from_annual_252(dec!(14.90));
        assert!((r - dec!(0.05513)).abs() < dec!(0.00001), "{r}");
    }

    #[test]
    fn accrues_over_business_days_with_gaps() {
        // Sexta 09/10/2026 → terça 13/10/2026: só 13/10 é dia útil (12/10 é feriado).
        let rates = vec![(d("2026-10-09"), dec!(0.05)), (d("2026-10-13"), dec!(0.06))];
        let (v, est) = accrue_cdi(
            dec!(1000),
            &rates,
            dec!(100),
            d("2026-10-09"),
            d("2026-10-13"),
        );
        assert_eq!(v, dec!(1000.6000));
        assert!(!est);
        // Sem taxa para 14/10: usa a última e marca estimado.
        let (v2, est2) = accrue_cdi(
            dec!(1000),
            &rates,
            dec!(100),
            d("2026-10-09"),
            d("2026-10-14"),
        );
        assert!(est2);
        assert!(v2 > v);
    }

    #[test]
    fn prefixed_one_year_of_business_days() {
        // 252 dias úteis a 10% a.a. = exatamente 10%.
        let start = d("2025-12-31");
        let mut end = start;
        while business_days_between(start, end) < 252 {
            end = end.succ_opt().unwrap();
        }
        let v = prefixed_value(dec!(1000), dec!(10), start, end);
        assert!((v - dec!(1100)).abs() < dec!(0.0000001), "{v}");
    }

    #[test]
    fn regressive_tables() {
        assert_eq!(ir_rate(1), dec!(0.225));
        assert_eq!(ir_rate(181), dec!(0.200));
        assert_eq!(ir_rate(720), dec!(0.175));
        assert_eq!(ir_rate(721), dec!(0.150));
        assert_eq!(iof_rate(1), dec!(0.96));
        assert_eq!(iof_rate(29), dec!(0.03));
        assert_eq!(iof_rate(30), dec!(0));
        let r = net_redemption(dec!(1000), dec!(1010), 10, false);
        assert_eq!(r.iof, dec!(6.60)); // 10 × 66%
        assert_eq!(r.ir, dec!(0.77)); // (10 − 6,60) × 22,5% = 0,765 → 0,77
        assert_eq!(r.net, dec!(1002.63));
        let lci = net_redemption(dec!(1000), dec!(1010), 10, true);
        assert_eq!(lci.net, dec!(1010));
    }
}
