//! Reino Unido: ONS (CPI e CPIH, JSON da página da série) e Bank of England (SONIA, Bank
//! Rate e câmbio, CSV do IADB).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::{parse_value, sorted};
use crate::error::{Error, Result};

/// `{"months":[{"date":"2026 AUG","value":"143.6"}]}` (a página da série na ONS).
pub fn parse_ons(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    #[derive(Deserialize)]
    struct Obs {
        date: String,
        value: String,
    }
    #[derive(Deserialize)]
    struct Resp {
        months: Vec<Obs>,
    }
    let r: Resp = serde_json::from_str(body).map_err(|e| super::bad_response("ONS", e))?;
    let mut out = vec![];
    for o in r.months {
        let Some(v) = parse_value(&o.value)? else {
            continue;
        };
        let d = NaiveDate::parse_from_str(&format!("{} 01", o.date.trim()), "%Y %b %d").map_err(
            |_| {
                Error::invalid(crate::i18n::t_args(
                    "nucleo-ativos-erro-ons-mes",
                    &crate::i18n::args(&[("mes", o.date.clone().into())]),
                ))
            },
        )?;
        out.push((d, v));
    }
    Ok(sorted(out))
}

/// `DATE,IUDSOIA` / `08 Oct 2026,3.7332`.
pub fn parse_boe_csv(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    let mut lines = body.lines().filter(|l| !l.trim().is_empty());
    match lines.next() {
        Some(h) if h.trim_start_matches('\u{feff}').starts_with("DATE,") => {}
        _ => {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-ativos-erro-boe-nao-csv",
            )));
        }
    }
    let mut out = vec![];
    for l in lines {
        let Some((d, v)) = l.split_once(',') else {
            continue;
        };
        let Some(v) = parse_value(v)? else {
            continue;
        };
        let d = NaiveDate::parse_from_str(d.trim(), "%d %b %Y").map_err(|_| {
            Error::invalid(crate::i18n::t_args(
                "nucleo-ativos-erro-boe-data",
                &crate::i18n::args(&[("data", d.to_string().into())]),
            ))
        })?;
        out.push((d, v));
    }
    Ok(sorted(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::parse_date;
    use rust_decimal_macros::dec;

    #[test]
    fn ons_monthly_index() {
        let v = parse_ons(include_str!("fixtures/ons_cpi.json")).unwrap();
        assert_eq!(v.len(), 3);
        assert_eq!(v[2], (parse_date("2026-08-01").unwrap(), dec!(143.6)));
        assert!(parse_ons(r#"{"months":[{"date":"2026 XYZ","value":"1"}]}"#).is_err());
    }

    #[test]
    fn boe_daily_rate() {
        let v = parse_boe_csv(include_str!("fixtures/boe_sonia.csv")).unwrap();
        assert_eq!(v[0], (parse_date("2026-10-08").unwrap(), dec!(3.7332)));
        assert_eq!(v.len(), 3);
        assert!(parse_boe_csv("<html>Access denied</html>").is_err());
    }
}
