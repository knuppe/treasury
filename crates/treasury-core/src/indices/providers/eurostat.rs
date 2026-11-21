//! Eurostat (API de disseminação, JSON-stat 2.0). A consulta fixa todas as dimensões menos
//! o tempo (`geo`, `unit`, `coicop18`), então cada posição de `value` é um período.

use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::{parse_period, sorted};
use crate::error::{Error, Result};

#[derive(Deserialize)]
struct Category {
    index: serde_json::Value,
}

#[derive(Deserialize)]
struct Dimension {
    category: Category,
}

#[derive(Deserialize)]
struct Dataset {
    id: Vec<String>,
    size: Vec<usize>,
    value: serde_json::Value,
    dimension: HashMap<String, Dimension>,
}

pub fn parse_json(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    let ds: Dataset = serde_json::from_str(body).map_err(|e| super::bad_response("Eurostat", e))?;
    let time_pos =
        ds.id.iter().position(|d| d == "time").ok_or_else(|| {
            Error::invalid(crate::i18n::t("nucleo-ativos-erro-eurostat-sem-tempo"))
        })?;
    let others: usize = ds
        .size
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != time_pos)
        .map(|(_, s)| *s)
        .product();
    if others != 1 || time_pos + 1 != ds.size.len() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-eurostat-varias-series",
        )));
    }
    // Posição → período.
    let periods: HashMap<usize, String> = match &ds.dimension["time"].category.index {
        serde_json::Value::Object(m) => m
            .iter()
            .filter_map(|(k, v)| v.as_u64().map(|i| (i as usize, k.clone())))
            .collect(),
        serde_json::Value::Array(a) => a
            .iter()
            .enumerate()
            .filter_map(|(i, k)| k.as_str().map(|k| (i, k.to_string())))
            .collect(),
        _ => {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-ativos-erro-eurostat-indice-tempo",
            )));
        }
    };
    let num = |v: &serde_json::Value| -> Result<Option<Decimal>> {
        match v {
            serde_json::Value::Number(n) => super::parse_value(&n.to_string()),
            serde_json::Value::Null => Ok(None),
            other => Err(Error::invalid(crate::i18n::t_args(
                "nucleo-ativos-erro-eurostat-valor",
                &crate::i18n::args(&[("valor", other.to_string().into())]),
            ))),
        }
    };
    let mut out = vec![];
    let mut push = |pos: usize, v: &serde_json::Value| -> Result<()> {
        if let (Some(p), Some(x)) = (periods.get(&pos), num(v)?) {
            out.push((parse_period(p)?, x));
        }
        Ok(())
    };
    match &ds.value {
        // Formato esparso: {"posição": valor}; posições sem valor não aparecem.
        serde_json::Value::Object(m) => {
            for (k, v) in m {
                let pos: usize = k.parse().map_err(|_| {
                    Error::invalid(crate::i18n::t("nucleo-ativos-erro-eurostat-posicao"))
                })?;
                push(pos, v)?;
            }
        }
        serde_json::Value::Array(a) => {
            for (pos, v) in a.iter().enumerate() {
                push(pos, v)?;
            }
        }
        _ => {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-ativos-erro-eurostat-valores",
            )));
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
    fn hicp_index_sparse_values() {
        let v = parse_json(include_str!("fixtures/eurostat_hicp.json")).unwrap();
        // Four periods, the last (September) still with no published value.
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], (parse_date("2026-06-01").unwrap(), dec!(129.98)));
        assert_eq!(v[2].0, parse_date("2026-08-01").unwrap());
    }

    #[test]
    fn rejects_multi_series_and_garbage() {
        let multi = r#"{"id":["geo","time"],"size":[2,1],"value":{"0":1,"1":2},
            "dimension":{"geo":{"category":{"index":{"PT":0,"ES":1}}},"time":{"category":{"index":{"2026-01":0}}}}}"#;
        assert!(parse_json(multi).is_err());
        assert!(parse_json("{}").is_err());
        let arr = r#"{"id":["geo","time"],"size":[1,2],"value":[1.5,null],
            "dimension":{"geo":{"category":{"index":{"PT":0}}},"time":{"category":{"index":["2026-01","2026-02"]}}}}"#;
        assert_eq!(
            parse_json(arr).unwrap(),
            vec![(parse_date("2026-01-01").unwrap(), dec!(1.5))]
        );
    }
}
