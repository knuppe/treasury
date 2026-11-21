//! Dados on-chain gratuitos e sem chave: endereços ativos pela Coin Metrics (API Community,
//! CC BY-NC 4.0, com atribuição), pools de mineração do bitcoin pelo mempool.space (para o
//! coeficiente de Nakamoto) e incidentes contra redes pela lista de ataques da DefiLlama.
//! Respostas são dados, nunca instruções.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use serde::Deserialize;

pub const CM_CATALOG_URL: &str = "https://community-api.coinmetrics.io/v4/catalog-v2/asset-metrics?metrics=AdrActCnt&page_size=10000";
pub const MEMPOOL_POOLS_URL: &str = "https://mempool.space/api/v1/mining/pools/1w";
pub const HACKS_URL: &str = "https://api.llama.fi/hacks";
/// Dias da média de endereços ativos.
pub const ACTIVE_DAYS: i64 = 30;

/// Endereços ativos por dia dos últimos 30 dias (ids da Coin Metrics, minúsculos).
pub fn cm_addresses_url(assets: &[String], today: NaiveDate) -> String {
    let start = today - chrono::Duration::days(ACTIVE_DAYS + 1);
    format!(
        "https://community-api.coinmetrics.io/v4/timeseries/asset-metrics?assets={}&metrics=AdrActCnt&frequency=1d&start_time={}&page_size=10000",
        assets.join(","),
        crate::time::fmt_date(start)
    )
}

/// Ativos com endereços ativos diários na API Community.
pub fn parse_cm_catalog(bytes: &[u8]) -> Result<BTreeSet<String>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-coin-metrics-catalogo",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let rows = v["data"]
        .as_array()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-coin-metrics-sem-ativos"))?;
    Ok(rows
        .iter()
        .filter(|a| {
            a["metrics"].as_array().is_some_and(|ms| {
                ms.iter().any(|m| {
                    m["metric"] == "AdrActCnt"
                        && m["frequencies"].as_array().is_some_and(|fs| {
                            fs.iter()
                                .any(|f| f["frequency"] == "1d" && f["community"] != false)
                        })
                })
            })
        })
        .filter_map(|a| a["asset"].as_str().map(str::to_lowercase))
        .collect())
}

/// Média diária de endereços ativos por ativo e o último dia.
pub fn parse_cm_addresses(bytes: &[u8]) -> Result<BTreeMap<String, (f64, NaiveDate)>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-coin-metrics",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let rows = v["data"]
        .as_array()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-coin-metrics-sem-serie"))?;
    let mut by: BTreeMap<String, Vec<(NaiveDate, f64)>> = BTreeMap::new();
    for r in rows {
        let (Some(a), Some(t), Some(x)) = (
            r["asset"].as_str(),
            r["time"].as_str().and_then(|t| t.get(..10)),
            r["AdrActCnt"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| r["AdrActCnt"].as_f64()),
        ) else {
            continue;
        };
        let Ok(d) = NaiveDate::parse_from_str(t, "%Y-%m-%d") else {
            continue;
        };
        if x.is_finite() && x >= 0.0 {
            by.entry(a.to_lowercase()).or_default().push((d, x));
        }
    }
    Ok(by
        .into_iter()
        .filter_map(|(a, pts)| {
            let last = pts.iter().map(|p| p.0).max()?;
            let recent: Vec<f64> = pts
                .iter()
                .filter(|p| (last - p.0).num_days() < ACTIVE_DAYS)
                .map(|p| p.1)
                .collect();
            let mean = recent.iter().sum::<f64>() / recent.len() as f64;
            Some((a, (mean, last)))
        })
        .collect())
}

#[derive(Deserialize)]
struct Pools {
    pools: Vec<Pool>,
}

#[derive(Deserialize)]
struct Pool {
    #[serde(rename = "blockCount")]
    block_count: u64,
}

/// Coeficiente de Nakamoto do bitcoin: menor número de pools com mais da metade dos blocos
/// da semana. Devolve (coeficiente, blocos na semana).
pub fn parse_mempool_pools(bytes: &[u8]) -> Result<(u32, u64), String> {
    let p: Pools = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-mempool",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let mut counts: Vec<u64> = p.pools.iter().map(|x| x.block_count).collect();
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return Err(crate::i18n::t("nucleo-score-erro-mempool-sem-blocos"));
    }
    counts.sort_unstable_by(|a, b| b.cmp(a));
    let mut acc = 0;
    for (i, c) in counts.iter().enumerate() {
        acc += c;
        if acc * 2 > total {
            return Ok((i as u32 + 1, total));
        }
    }
    Ok((counts.len() as u32, total))
}

/// Incidente contra uma rede (não contra um protocolo dela): nome normalizado da rede.
pub fn parse_chain_hacks(bytes: &[u8]) -> Result<Vec<(String, NaiveDate)>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-ataques",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let rows = v
        .as_array()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-ataques-sem-lista"))?;
    Ok(rows
        .iter()
        .filter(|r| r["targetType"] == "Chain")
        .filter_map(|r| {
            let name = r["name"].as_str()?;
            let ts = r["date"].as_i64()?;
            let d = chrono::DateTime::from_timestamp(ts, 0)?.date_naive();
            Some((chain_key(name), d))
        })
        .collect())
}

/// Nome de rede para comparar: sem o que está entre parênteses ("Terra (LUNA)").
pub fn chain_key(name: &str) -> String {
    let base = name.split('(').next().unwrap_or(name);
    crate::research::norm(base).trim().to_string()
}

/// Incidentes contra a rede de uma moeda (pelo nome da CoinGecko).
pub fn incidents_for(hacks: &[(String, NaiveDate)], coin_name: &str) -> u32 {
    let k = chain_key(coin_name);
    if k.is_empty() {
        return 0;
    }
    hacks.iter().filter(|(n, _)| *n == k).count() as u32
}

/// Moeda de prova de trabalho (categorias da CoinGecko): sem alocações com desbloqueio.
pub fn is_proof_of_work(categories: &[String]) -> bool {
    categories
        .iter()
        .any(|c| c.to_lowercase().contains("proof of work"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn coin_metrics_catalog_and_mean_of_active_addresses() {
        let cat = br#"{"data":[
            {"asset":"btc","metrics":[{"metric":"AdrActCnt","frequencies":[{"frequency":"1d","community":true}]}]},
            {"asset":"eth","metrics":[{"metric":"AdrActCnt","frequencies":[{"frequency":"1b","community":true}]}]}
        ]}"#;
        assert_eq!(
            parse_cm_catalog(cat).unwrap(),
            ["btc".to_string()].into_iter().collect()
        );
        let ts = br#"{"data":[
            {"asset":"btc","time":"2026-08-01T00:00:00.000000000Z","AdrActCnt":"10"},
            {"asset":"btc","time":"2026-10-18T00:00:00.000000000Z","AdrActCnt":"600000"},
            {"asset":"btc","time":"2026-10-19T00:00:00.000000000Z","AdrActCnt":"700000"}
        ]}"#;
        let m = parse_cm_addresses(ts).unwrap();
        // The August point is outside the 30-day window.
        assert_eq!(m["btc"], (650_000.0, d("2026-10-19")));
        assert!(
            cm_addresses_url(&["btc".into(), "eth".into()], d("2026-10-20"))
                .contains("assets=btc,eth&metrics=AdrActCnt&frequency=1d&start_time=2026-09-19")
        );
    }

    #[test]
    fn nakamoto_from_pool_blocks() {
        let b = br#"{"pools":[{"blockCount":267},{"blockCount":233},{"blockCount":150},{"blockCount":100},{"blockCount":258}]}"#;
        // 1008 blocks: 267 + 258 = 525 > 504.
        assert_eq!(parse_mempool_pools(b).unwrap(), (2, 1008));
        assert!(parse_mempool_pools(br#"{"pools":[]}"#).is_err());
    }

    #[test]
    fn chain_incidents_by_network_name() {
        let b = br#"[
            {"date":1546300800,"name":"Ethereum Classic","targetType":"Chain"},
            {"date":1596240000,"name":"Ethereum Classic","targetType":"Chain"},
            {"date":1652000000,"name":"Terra (LUNA)","targetType":"Chain"},
            {"date":1652000000,"name":"Uniswap","targetType":"DeFi Protocol"}
        ]"#;
        let h = parse_chain_hacks(b).unwrap();
        assert_eq!(h.len(), 3);
        assert_eq!(incidents_for(&h, "Ethereum Classic"), 2);
        assert_eq!(incidents_for(&h, "Terra"), 1);
        assert_eq!(incidents_for(&h, "Bitcoin"), 0);
        assert!(is_proof_of_work(&["Proof of Work (PoW)".into()]));
        assert!(!is_proof_of_work(&["Smart Contract Platform".into()]));
    }
}
