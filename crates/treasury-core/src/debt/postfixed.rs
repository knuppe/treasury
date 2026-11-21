//! Pós-fixados de taxa diária (CDI, Selic): fator de juros de cada período do contrato,
//! pela metodologia da B3 para o DI (docs/brazil/floating-rate-loans.md):
//!
//! - período da data-base (assinatura ou vencimento anterior), inclusive, ao vencimento,
//!   exclusive, contando só dias úteis (calendário nacional da ANBIMA);
//! - fator do dia `1 + TDIₖ × p` com a taxa diária do BCB (SGS 12 para o CDI, 11 para a
//!   Selic, já em % a.d. com 8 casas) e o percentual `p` do índice, com 16 casas truncadas,
//!   como o produtório;
//! - fator do índice no período com 8 casas; spread exponencial `(1 + s)^(du/252)` ou linear
//!   `1 + s × dc/360`, com 9 casas; fator de juros = índice × spread, com 9 casas;
//! - vencimento em dia não útil vai para o próximo dia útil (os juros correm até lá);
//! - defasagem opcional: o DI de `n` dias úteis antes de cada dia.
//!
//! Dias sem taxa publicada (o futuro) usam a última conhecida e marcam o período como
//! estimado; sem nenhuma taxa no cofre, usa a Selic meta − 0,10 p.p. como CDI projetado.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use super::schedule::Period;
use crate::error::Result;
use crate::finance::calendar::{is_business_day, next_business_day};
use crate::finance::fixed_income::{cdi_day_factor, daily_from_annual_252};
use crate::finance::pow_dec;
use crate::time::{add_months, today_local};

/// Forma do spread sobre o índice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spread {
    /// `(1 + s)^(du/252)`: o padrão do mercado (CCB, capital de giro, debêntures).
    Exp252,
    /// `1 + s × dc/360`: juros simples no spread, em dias corridos.
    Lin360,
}

#[derive(Debug, Clone)]
pub struct Terms {
    /// Série diária no cofre: `cdi` ou `selic`.
    pub series: &'static str,
    /// Percentual do índice (100 = 100%).
    pub pct: Decimal,
    /// Spread % a.a.
    pub spread_pct: Decimal,
    pub spread: Spread,
    /// Defasagem em dias úteis.
    pub lag: u32,
}

/// `n` dias úteis antes de `d`.
fn back_business_days(mut d: NaiveDate, n: u32) -> NaiveDate {
    let mut left = n;
    while left > 0 {
        d = d.pred_opt().expect("data válida");
        if is_business_day(d) {
            left -= 1;
        }
    }
    d
}

/// Fator do índice (8 casas), dias úteis e se algum dia foi projetado, de `start` (incl.)
/// a `end` (excl.).
fn index_factor(
    rates: &BTreeMap<NaiveDate, Decimal>,
    fallback: Decimal,
    t: &Terms,
    start: NaiveDate,
    end: NaiveDate,
) -> (Decimal, u32, bool) {
    let mut acc = Decimal::ONE;
    let mut du = 0u32;
    let mut estimated = false;
    let mut d = start;
    while d < end {
        if is_business_day(d) {
            du += 1;
            let r = back_business_days(d, t.lag);
            let rate = match rates.get(&r) {
                Some(v) => *v,
                None => {
                    estimated = true;
                    rates
                        .range(..r)
                        .next_back()
                        .map(|(_, v)| *v)
                        .unwrap_or(fallback)
                }
            };
            acc = (acc * cdi_day_factor(rate, t.pct))
                .round_dp_with_strategy(16, RoundingStrategy::ToZero);
        }
        d = d.succ_opt().expect("data válida");
    }
    (acc.round_dp(8), du, estimated)
}

fn spread_factor(t: &Terms, du: u32, start: NaiveDate, end: NaiveDate) -> Decimal {
    let s = t.spread_pct / Decimal::ONE_HUNDRED;
    match t.spread {
        Spread::Exp252 => {
            pow_dec(Decimal::ONE + s, Decimal::from(du) / Decimal::from(252)).round_dp(9)
        }
        Spread::Lin360 => {
            let dc = (end - start).num_days().max(0);
            (Decimal::ONE + s * Decimal::from(dc) / Decimal::from(360)).round_dp(9)
        }
    }
}

/// Vencimentos ajustados ao dia útil: `first_due`, +1 mês, … (`count` parcelas).
pub fn due_dates(first_due: NaiveDate, count: u32) -> Vec<NaiveDate> {
    (0..count)
        .map(|k| next_business_day(add_months(first_due, k as i32)))
        .collect()
}

/// Um período por parcela, do contrato ao vencimento de cada uma.
pub fn periods(
    conn: &Connection,
    t: &Terms,
    contract_date: NaiveDate,
    first_due: NaiveDate,
    count: u32,
) -> Result<Vec<Period>> {
    let dues = due_dates(first_due, count);
    let Some(&last_due) = dues.last() else {
        return Ok(vec![]);
    };
    let from = back_business_days(contract_date, t.lag + 10);
    let rates: BTreeMap<NaiveDate, Decimal> =
        crate::indices::daily_rates(conn, t.series, from, last_due)?
            .into_iter()
            .collect();
    let fallback = projected_daily(conn)?;
    let mut out = Vec::with_capacity(dues.len());
    let mut start = contract_date;
    for due in dues {
        let (fdi, du, estimated) = index_factor(&rates, fallback, t, start, due);
        let factor = (fdi * spread_factor(t, du, start, due)).round_dp(9);
        out.push(Period {
            due,
            factor,
            business_days: du,
            estimated,
        });
        start = due;
    }
    Ok(out)
}

/// Taxa diária projetada quando não há nenhuma no cofre: Selic meta − 0,10 p.p. (o CDI
/// costuma ficar 0,10 abaixo da meta), em % a.d. base 252; sem a meta, 10% a.a.
fn projected_daily(conn: &Connection) -> Result<Decimal> {
    let meta = crate::indices::level_on(conn, "selic_meta", today_local())?;
    let annual = meta.map_or(Decimal::from(10), |m| m - Decimal::new(10, 2));
    Ok(daily_from_annual_252(annual.max(Decimal::ZERO)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    fn terms(pct: Decimal, spread_pct: Decimal, spread: Spread) -> Terms {
        Terms {
            series: "cdi",
            pct,
            spread_pct,
            spread,
            lag: 0,
        }
    }

    /// Constant CDI of 14.90% a.a. (0.05501% a.d.) on every day.
    fn flat(from: &str, to: &str, daily: Decimal) -> BTreeMap<NaiveDate, Decimal> {
        let mut m = BTreeMap::new();
        let mut x = d(from);
        while x <= d(to) {
            if is_business_day(x) {
                m.insert(x, daily);
            }
            x = x.succ_opt().unwrap();
        }
        m
    }

    #[test]
    fn index_factor_counts_business_days_start_inclusive_end_exclusive() {
        // 2026-03-02 (Mon) to 2026-04-01 (Wed): 22 business days, no national holiday.
        let rates = flat("2026-02-20", "2026-04-30", dec!(0.05501));
        let t = terms(dec!(100), dec!(0), Spread::Exp252);
        let (f, du, est) = index_factor(&rates, dec!(0), &t, d("2026-03-02"), d("2026-04-01"));
        assert_eq!(du, 22);
        assert!(!est);
        // (1.0005501)^22 = 1.01216... with daily truncation.
        let expected = (0..22).fold(Decimal::ONE, |a, _| {
            (a * dec!(1.0005501)).round_dp_with_strategy(16, RoundingStrategy::ToZero)
        });
        assert_eq!(f, expected.round_dp(8));
    }

    #[test]
    fn percent_of_cdi_and_spreads() {
        let rates = flat("2026-02-20", "2026-04-30", dec!(0.05501));
        let (s, e) = (d("2026-03-02"), d("2026-04-01"));
        // 120% of CDI: daily factor 1 + 0.05501% × 1.2.
        let t = terms(dec!(120), dec!(0), Spread::Exp252);
        let (f120, _, _) = index_factor(&rates, dec!(0), &t, s, e);
        let (f100, _, _) = index_factor(
            &rates,
            dec!(0),
            &terms(dec!(100), dec!(0), Spread::Exp252),
            s,
            e,
        );
        assert!(f120 > f100);
        // Exponential spread of 2% a.a. over 22 business days: 1.02^(22/252).
        let t2 = terms(dec!(100), dec!(2), Spread::Exp252);
        let sp = spread_factor(&t2, 22, s, e);
        assert!((sp.to_f64().unwrap() - 1.02f64.powf(22.0 / 252.0)).abs() < 1e-8);
        // Linear 360 over 30 calendar days: 1 + 0.02 × 30/360.
        let t3 = terms(dec!(100), dec!(2), Spread::Lin360);
        assert_eq!(spread_factor(&t3, 22, s, e), dec!(1.001666667));
    }

    #[test]
    fn missing_days_use_last_rate_and_mark_estimated() {
        let rates = flat("2026-02-20", "2026-03-10", dec!(0.05501));
        let t = terms(dec!(100), dec!(0), Spread::Exp252);
        let (f, du, est) = index_factor(&rates, dec!(0), &t, d("2026-03-02"), d("2026-04-01"));
        let (full, _, _) = index_factor(
            &flat("2026-02-20", "2026-04-30", dec!(0.05501)),
            dec!(0),
            &t,
            d("2026-03-02"),
            d("2026-04-01"),
        );
        assert_eq!(du, 22);
        assert!(est);
        assert_eq!(f, full);
    }

    #[test]
    fn lag_uses_earlier_business_day() {
        let mut rates = flat("2026-02-20", "2026-04-30", dec!(0.05));
        rates.insert(d("2026-02-27"), dec!(0.10)); // Friday before the period
        let mut t = terms(dec!(100), dec!(0), Spread::Exp252);
        let (no_lag, _, _) = index_factor(&rates, dec!(0), &t, d("2026-03-02"), d("2026-03-03"));
        t.lag = 1;
        let (lag1, _, _) = index_factor(&rates, dec!(0), &t, d("2026-03-02"), d("2026-03-03"));
        assert_eq!(no_lag, dec!(1.0005));
        assert_eq!(lag1, dec!(1.001));
    }

    #[test]
    fn due_dates_roll_to_next_business_day() {
        // 2026-11-15 is a Sunday (a holiday the day before does not count): moves to 2026-11-16.
        let v = due_dates(d("2026-10-15"), 2);
        assert_eq!(v, vec![d("2026-10-15"), d("2026-11-16")]);
    }
}
