//! Federal Reserve Bank of New York, Markets Data API (JSON, sem chave):
//! `{"refRates":[{"effectiveDate":"2026-10-16","type":"EFFR","percentRate":3.88}]}`.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::{parse_period, sorted};
use crate::error::{Error, Result};

pub fn parse_rates(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| super::bad_response("NY Fed", e))?;
    let rows = v
        .get("refRates")
        .and_then(|o| o.as_array())
        .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-ativos-erro-nyfed-sem-taxas")))?;
    let mut out = vec![];
    for r in rows {
        let (Some(d), Some(x)) = (
            r.get("effectiveDate").and_then(|d| d.as_str()),
            r.get("percentRate").and_then(|x| x.as_f64()),
        ) else {
            continue;
        };
        let Some(x) = Decimal::from_f64_retain(x) else {
            continue;
        };
        out.push((parse_period(d)?, x.round_dp(4)));
    }
    Ok(sorted(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::parse_date;
    use rust_decimal_macros::dec;

    #[test]
    fn reads_effr() {
        let body = r#"{ "refRates": [ { "effectiveDate": "2026-10-16", "type": "EFFR" ,"percentRate": 3.88 ,"targetRateFrom": 3.75 }, { "effectiveDate": "2026-10-15", "type": "EFFR" ,"percentRate": 3.87 } ] }"#;
        let v = parse_rates(body).unwrap();
        assert_eq!(
            v,
            vec![
                (parse_date("2026-10-15").unwrap(), dec!(3.87)),
                (parse_date("2026-10-16").unwrap(), dec!(3.88)),
            ]
        );
        assert!(parse_rates("{}").is_err());
    }
}
