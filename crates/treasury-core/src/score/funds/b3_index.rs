//! Carteira teórica do dia dos índices da B3 (Ibovespa, IBrX, Small Cap, IDIV…), a
//! composição dos ETFs de ações que replicam esses índices: dá a herança do score dos
//! componentes (§7.2) e a concentração. Guardada no ajuste técnico `score.index_portfolios`.

use std::collections::BTreeMap;

use base64::Engine as _;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::Result;

pub const SETTING: &str = "score.index_portfolios";
/// A carteira teórica muda a cada quadrimestre (e com eventos): uma vez por dia basta.
pub const REFRESH_HOURS: i64 = 20;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holding {
    pub ticker: String,
    /// Participação como fração (0,0236 = 2,36%).
    pub weight: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Portfolio {
    /// Data da carteira informada pela B3 (dd/mm/aa).
    pub date: String,
    pub holdings: Vec<Holding>,
    pub fetched_at: String,
}

pub fn url(index: &str) -> String {
    let body = serde_json::json!({
        "language": "pt-br",
        "pageNumber": 1,
        "pageSize": 500,
        "index": index,
        "segment": "1"
    });
    let p = base64::engine::general_purpose::STANDARD.encode(body.to_string());
    format!("https://sistemaswebb3-listados.b3.com.br/indexProxy/indexCall/GetPortfolioDay/{p}")
}

/// Número no formato brasileiro ("2,359", "93.803.750.400").
fn br_num(s: &str) -> Option<f64> {
    s.trim().replace('.', "").replace(',', ".").parse().ok()
}

pub fn parse(bytes: &[u8]) -> std::result::Result<(String, Vec<Holding>), String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-b3-carteira-invalida",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let date = v["header"]["date"].as_str().unwrap_or_default().to_string();
    let rows = v["results"]
        .as_array()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-b3-sem-carteira"))?;
    let mut out: Vec<Holding> = rows
        .iter()
        .filter_map(|r| {
            let t = r["cod"].as_str()?.trim().to_uppercase();
            let w = br_num(r["part"].as_str()?)? / 100.0;
            (!t.is_empty() && w > 0.0).then_some(Holding {
                ticker: t,
                weight: w,
            })
        })
        .collect();
    if out.is_empty() {
        return Err(crate::i18n::t("nucleo-score-erro-carteira-vazia"));
    }
    out.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    Ok((date, out))
}

pub fn load(conn: &Connection) -> Result<BTreeMap<String, Portfolio>> {
    Ok(crate::settings::get(conn, SETTING)?.unwrap_or_default())
}

pub fn apply(
    conn: &Connection,
    index: &str,
    (date, holdings): (String, Vec<Holding>),
    now: &str,
) -> Result<u64> {
    let mut all = load(conn)?;
    let n = holdings.len() as u64;
    all.insert(
        index.to_string(),
        Portfolio {
            date,
            holdings,
            fetched_at: now.to_string(),
        },
    );
    crate::settings::set(conn, SETTING, &all)?;
    Ok(n)
}

/// Peso das `n` maiores posições.
pub fn top_weight(p: &Portfolio, n: usize) -> f64 {
    let total: f64 = p.holdings.iter().map(|h| h.weight).sum();
    if total <= 0.0 {
        return 0.0;
    }
    p.holdings.iter().take(n).map(|h| h.weight).sum::<f64>() / total
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const FIXTURE: &str = r#"{"page":{"pageNumber":1,"pageSize":120,"totalRecords":3,"totalPages":1},"header":{"date":"16/10/26","text":"Quantidade Teórica Total","part":"100,000"},"results":[{"segment":null,"cod":"ALOS3","asset":"ALLOS","type":"ON      NM","part":"0,580","theoricalQty":"478.975.645"},{"segment":null,"cod":"ABEV3","asset":"AMBEV S/A","type":"ON","part":"2,359","theoricalQty":"4.043.349.180"},{"segment":null,"cod":"VALE3","asset":"VALE","type":"ON      NM","part":"11,061","theoricalQty":"1"}]}"#;

    #[test]
    fn parses_b3_portfolio() {
        let (date, h) = parse(FIXTURE.as_bytes()).unwrap();
        assert_eq!(date, "16/10/26");
        assert_eq!(h[0].ticker, "VALE3");
        assert!((h[0].weight - 0.11061).abs() < 1e-12);
        assert_eq!(h.len(), 3);
        let p = Portfolio {
            date,
            holdings: h,
            fetched_at: String::new(),
        };
        let total = 0.11061 + 0.02359 + 0.0058;
        assert!((top_weight(&p, 2) - (0.11061 + 0.02359) / total).abs() < 1e-12);
        assert!(parse(b"{\"results\":[]}").is_err());
        assert!(url("IBOV").contains("GetPortfolioDay/eyJ"));
    }
}
