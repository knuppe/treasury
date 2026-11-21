//! SDMX em CSV (BCE `format=csvdata` e OCDE `format=csvfile`): uma linha por observação,
//! com as colunas `TIME_PERIOD` e `OBS_VALUE`. A consulta pede uma série só.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::{parse_period, parse_value, sorted};
use crate::error::{Error, Result};

fn csv_error(e: csv::Error) -> Error {
    Error::invalid(crate::i18n::t_args(
        "nucleo-ativos-erro-sdmx-csv",
        &crate::i18n::args(&[("erro", e.to_string().into())]),
    ))
}

pub fn parse_csv(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    let body = body.trim_start_matches('\u{feff}');
    if body.trim().is_empty() {
        return Ok(vec![]);
    }
    if body.trim_start().starts_with('<') || body.trim() == "NoRecordsFound" {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-sdmx-nao-csv",
        )));
    }
    let mut rd = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(body.as_bytes());
    let headers = rd.headers().map_err(csv_error)?.clone();
    let col = |name: &str| {
        headers.iter().position(|h| h == name).ok_or_else(|| {
            Error::invalid(crate::i18n::t_args(
                "nucleo-ativos-erro-sdmx-coluna",
                &crate::i18n::args(&[("coluna", name.to_string().into())]),
            ))
        })
    };
    let (t, v) = (col("TIME_PERIOD")?, col("OBS_VALUE")?);
    let mut out = vec![];
    for rec in rd.records() {
        let rec = rec.map_err(csv_error)?;
        let (Some(period), Some(value)) = (rec.get(t), rec.get(v)) else {
            continue;
        };
        if let Some(x) = parse_value(value)? {
            out.push((parse_period(period)?, x));
        }
    }
    Ok(sorted(out))
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
    fn ecb_hicp_monthly_index() {
        let v = parse_csv(include_str!("fixtures/ecb_hicp.csv")).unwrap();
        assert_eq!(v.len(), 4);
        assert_eq!(v[0], (d("2026-06-01"), dec!(104.12)));
        assert_eq!(v[3], (d("2026-09-01"), dec!(105.08)));
    }

    #[test]
    fn ecb_full_csv_with_metadata_columns() {
        // Câmbio de referência (formato completo, com títulos entre aspas e vírgulas).
        let v = parse_csv(include_str!("fixtures/ecb_est.csv")).unwrap();
        assert!(!v.is_empty());
        assert!(v.iter().all(|(_, x)| *x > dec!(0) && *x < dec!(10)));
        assert_eq!(v.first().unwrap().0, d("2026-10-09"));
    }

    #[test]
    fn ecb_exchange_rates() {
        // Duas moedas na mesma resposta: a coleta pede uma série por vez; aqui só confere
        // que linhas válidas são lidas e ordenadas.
        let v = parse_csv(include_str!("fixtures/ecb_exr.csv")).unwrap();
        assert_eq!(v.len(), 2, "datas repetidas viram um ponto só");
    }

    #[test]
    fn oecd_cpi_unsorted_rows() {
        let v = parse_csv(include_str!("fixtures/oecd_cpi.csv")).unwrap();
        assert_eq!(
            v.iter().map(|(d, _)| *d).collect::<Vec<_>>(),
            vec![
                d("2026-05-01"),
                d("2026-06-01"),
                d("2026-07-01"),
                d("2026-08-01")
            ]
        );
        assert_eq!(v[3].1, dec!(116.4783));
    }

    #[test]
    fn empty_and_invalid() {
        assert!(parse_csv("").unwrap().is_empty());
        assert!(
            parse_csv("KEY,TIME_PERIOD,OBS_VALUE\nX,1987-01,\n")
                .unwrap()
                .is_empty()
        );
        assert!(parse_csv("<html>blocked</html>").is_err());
        assert!(parse_csv("NoRecordsFound").is_err());
        assert!(parse_csv("A,B\n1,2\n").is_err());
    }
}
