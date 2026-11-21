//! Bank of Canada, API Valet (JSON): `{"observations":[{"d":"2026-10-14","FXUSDCAD":{"v":"1.4263"}}]}`.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::{parse_period, parse_value, sorted};
use crate::error::{Error, Result};

pub fn parse_valet(body: &str, series: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| super::bad_response("Bank of Canada", e))?;
    if let Some(m) = v.get("message").and_then(|m| m.as_str()) {
        return Err(Error::invalid(format!("Bank of Canada: {m}")));
    }
    let obs = v
        .get("observations")
        .and_then(|o| o.as_array())
        .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-ativos-erro-boc-sem-observacoes")))?;
    let mut out = vec![];
    for o in obs {
        let (Some(d), Some(val)) = (
            o.get("d").and_then(|d| d.as_str()),
            o.get(series)
                .and_then(|s| s.get("v"))
                .and_then(|v| v.as_str()),
        ) else {
            continue;
        };
        if let Some(x) = parse_value(val)? {
            out.push((parse_period(d)?, x));
        }
    }
    Ok(sorted(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::parse_date;
    use rust_decimal_macros::dec;

    #[test]
    fn valet_fx() {
        let v = parse_valet(include_str!("fixtures/boc_fx.json"), "FXUSDCAD").unwrap();
        assert!(v.len() >= 3);
        assert!(v.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(v.last().unwrap().0, parse_date("2026-10-14").unwrap());
        assert_eq!(v.last().unwrap().1, dec!(1.4263));
        // Outra série no mesmo corpo: nada.
        assert!(
            parse_valet(include_str!("fixtures/boc_fx.json"), "FXEURCAD")
                .unwrap()
                .is_empty()
        );
        assert!(parse_valet(r#"{"message":"Series not found"}"#, "X").is_err());
        assert!(parse_valet("<html>", "X").is_err());
    }
}
