//! Banco Mundial (World Development Indicators): inflação anual ao consumidor
//! (`FP.CPI.TOTL.ZG`, % de variação da média anual) de quase todos os países.
//!
//! Para servir de deflator, a série anual vira um índice mensal: a média de cada ano fica
//! no 1º de julho (meio do ano) e os meses entre dois anos são interpolados de forma
//! geométrica. Todos os pontos são **estimados**; meses depois do último 1º de julho
//! publicado são projetados pelo deflator repetindo a última variação mensal.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::error::{Error, Result};
use crate::finance::pow_dec;
use crate::time::add_months;

/// `[{"page":1,…},[{"date":"2025","value":2.33}, …]]` → (ano, % no ano), em ordem.
pub fn parse_json(body: &str) -> Result<Vec<(i32, Decimal)>> {
    let v: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| super::bad_response(&crate::sources::SourceId::WorldBank.name(), e))?;
    let arr = v.as_array().ok_or_else(|| {
        Error::invalid(crate::i18n::t_args(
            "nucleo-ativos-erro-resposta-invalida-sem-detalhe",
            &crate::i18n::args(&[("fonte", crate::sources::SourceId::WorldBank.name().into())]),
        ))
    })?;
    if let Some(msg) = arr
        .first()
        .and_then(|m| m.get("message"))
        .and_then(|m| m.as_array())
        .and_then(|m| m.first())
        .and_then(|m| m.get("value"))
        .and_then(|m| m.as_str())
    {
        return Err(Error::invalid(format!(
            "{}: {msg}",
            crate::sources::SourceId::WorldBank.name()
        )));
    }
    let rows = match arr.get(1) {
        Some(serde_json::Value::Array(r)) => r,
        // País sem dados: `[{…"total":0}, null]`.
        _ => return Ok(vec![]),
    };
    let mut out = vec![];
    for r in rows {
        let (Some(y), Some(n)) = (
            r.get("date")
                .and_then(|d| d.as_str())
                .and_then(|d| d.parse::<i32>().ok()),
            r.get("value").and_then(|v| v.as_number()),
        ) else {
            continue;
        };
        if let Some(x) = super::parse_value(&n.to_string())? {
            out.push((y, x));
        }
    }
    out.sort_by_key(|(y, _)| *y);
    out.dedup_by_key(|(y, _)| *y);
    Ok(out)
}

/// Índice mensal (base 100 no primeiro ano) a partir da inflação anual. Usa só o último
/// trecho de anos consecutivos (um ano sem dado quebra o encadeamento).
pub fn annual_to_monthly(annual: &[(i32, Decimal)]) -> Vec<(NaiveDate, Decimal)> {
    let mut run: Vec<(i32, Decimal)> = vec![];
    for &(y, r) in annual {
        if run.last().is_some_and(|(py, _)| *py + 1 != y) {
            run.clear();
        }
        run.push((y, r));
    }
    if run.is_empty() {
        return vec![];
    }
    // Nível médio de cada ano: o primeiro é 100; os seguintes acumulam a variação.
    let mut levels: Vec<(NaiveDate, Decimal)> = vec![];
    let mut level = Decimal::ONE_HUNDRED;
    for (i, (y, r)) in run.iter().enumerate() {
        if i > 0 {
            level *= Decimal::ONE + r / Decimal::ONE_HUNDRED;
        }
        if let Some(mid) = NaiveDate::from_ymd_opt(*y, 7, 1) {
            levels.push((mid, level));
        }
    }
    let mut out = vec![];
    for w in levels.windows(2) {
        let ((d0, l0), (_, l1)) = (w[0], w[1]);
        if l0 <= Decimal::ZERO || l1 <= Decimal::ZERO {
            continue;
        }
        let ratio = l1 / l0;
        for m in 0..12 {
            let f = pow_dec(ratio, Decimal::from(m) / Decimal::from(12));
            out.push((add_months(d0, m), (l0 * f).round_dp(6)));
        }
    }
    if let Some((d, l)) = levels.last() {
        out.push((*d, l.round_dp(6)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::parse_date;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn parses_annual_inflation_with_nulls() {
        let v = parse_json(include_str!("fixtures/worldbank_ar.json")).unwrap();
        // 2025 comes as null for Argentina.
        assert!(v.iter().all(|(y, _)| *y >= 2020 && *y <= 2024));
        assert!(v.iter().any(|(y, r)| *y == 2024 && *r > dec!(200)));
        assert!(
            parse_json(r#"[{"message":[{"id":"120","key":"Invalid value","value":"The provided parameter value is not valid"}]}]"#)
                .is_err()
        );
        assert!(
            parse_json(r#"[{"page":0,"pages":0,"total":0},null]"#)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn interpolates_mid_year_levels() {
        let m = annual_to_monthly(&[(2023, dec!(4)), (2024, dec!(10)), (2025, dec!(5))]);
        // Jul/2023 … Jul/2025: 25 monthly points.
        assert_eq!(m.len(), 25);
        assert_eq!(m[0], (d("2023-07-01"), dec!(100)));
        assert_eq!(m[12], (d("2024-07-01"), dec!(110)));
        assert_eq!(m[24], (d("2025-07-01"), dec!(115.5)));
        // Geometric: six months later ≈ square root of 1.10.
        let half = m[6].1;
        assert!((half - dec!(104.880885)).abs() < dec!(0.00001), "{half}");
        assert!(m.windows(2).all(|w| w[0].1 < w[1].1));
        // Gap: only the last continuous stretch counts.
        let g = annual_to_monthly(&[(2019, dec!(3)), (2021, dec!(2)), (2022, dec!(2))]);
        assert_eq!(g.first().unwrap().0, d("2021-07-01"));
        assert!(annual_to_monthly(&[]).is_empty());
        assert_eq!(annual_to_monthly(&[(2025, dec!(3))]).len(), 1);
    }
}
