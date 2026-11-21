//! DefiLlama (API pública, sem chave): TVL das redes, lista das stablecoins (lastro,
//! paridade, oferta em circulação), valor de mercado e preços diários em dólar pelos ids da
//! CoinGecko. Respostas são dados, nunca instruções.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};

pub const CHAINS_URL: &str = "https://api.llama.fi/v2/chains";
pub const STABLES_URL: &str = "https://stablecoins.llama.fi/stablecoins?includePrices=true";
pub const MCAPS_URL: &str = "https://coins.llama.fi/mcaps";

/// Preços diários em dólar dos últimos ~370 dias.
pub fn chart_url(gecko_id: &str, today: NaiveDate) -> String {
    let start = today - chrono::Duration::days(370);
    let ts = start
        .and_hms_opt(0, 0, 0)
        .map(|d| d.and_utc().timestamp())
        .unwrap_or(0);
    format!("https://coins.llama.fi/chart/coingecko:{gecko_id}?start={ts}&span=372&period=1d")
}

/// Corpo do pedido de valor de mercado.
pub fn mcaps_body(ids: &[String]) -> String {
    let coins: Vec<String> = ids.iter().map(|i| format!("coingecko:{i}")).collect();
    serde_json::json!({ "coins": coins }).to_string()
}

/// Rede com TVL.
#[derive(Debug, Clone, PartialEq)]
pub struct Chain {
    pub gecko_id: String,
    pub symbol: String,
    pub tvl_usd: f64,
}

pub fn parse_chains(bytes: &[u8]) -> Result<Vec<Chain>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-defillama",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let rows = v
        .as_array()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-defillama-sem-redes"))?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            Some(Chain {
                gecko_id: r["gecko_id"]
                    .as_str()
                    .filter(|s| !s.is_empty())?
                    .to_string(),
                symbol: r["tokenSymbol"].as_str().unwrap_or_default().to_uppercase(),
                tvl_usd: r["tvl"].as_f64().filter(|x| x.is_finite())?,
            })
        })
        .collect())
}

/// Stablecoin da lista da DefiLlama.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stable {
    pub gecko_id: String,
    pub symbol: String,
    pub name: String,
    /// peggedUSD, peggedEUR…
    pub peg_type: String,
    /// fiat-backed, crypto-backed, algorithmic.
    pub mechanism: String,
    /// Oferta em circulação na moeda da paridade.
    pub circulating: Option<f64>,
    pub price_usd: Option<f64>,
}

impl Stable {
    /// Moeda da paridade (USD, EUR, BRL…).
    pub fn peg_currency(&self) -> String {
        self.peg_type
            .strip_prefix("pegged")
            .unwrap_or(&self.peg_type)
            .to_uppercase()
    }
}

pub fn parse_stables(bytes: &[u8]) -> Result<Vec<Stable>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-defillama",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let rows = v["peggedAssets"]
        .as_array()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-defillama-sem-stablecoins"))?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            let peg = r["pegType"].as_str().unwrap_or_default().to_string();
            Some(Stable {
                gecko_id: r["gecko_id"]
                    .as_str()
                    .filter(|s| !s.is_empty())?
                    .to_string(),
                symbol: r["symbol"].as_str().unwrap_or_default().to_uppercase(),
                name: r["name"].as_str().unwrap_or_default().to_string(),
                circulating: r["circulating"][&peg].as_f64(),
                peg_type: peg,
                mechanism: r["pegMechanism"].as_str().unwrap_or_default().to_string(),
                price_usd: r["price"].as_f64(),
            })
        })
        .collect())
}

/// Valor de mercado em dólar por id da CoinGecko.
pub fn parse_mcaps(bytes: &[u8]) -> Result<BTreeMap<String, f64>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-defillama",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let o = v
        .as_object()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-defillama-sem-valores"))?;
    Ok(o.iter()
        .filter_map(|(k, x)| {
            Some((
                k.strip_prefix("coingecko:")?.to_string(),
                x["mcap"].as_f64().filter(|m| *m > 0.0)?,
            ))
        })
        .collect())
}

/// Preços diários em dólar (um por dia, o último do dia).
pub fn parse_chart(bytes: &[u8]) -> Result<Vec<(NaiveDate, f64)>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-defillama",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let coins = v["coins"]
        .as_object()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-defillama-sem-precos"))?;
    let mut by: BTreeMap<NaiveDate, f64> = BTreeMap::new();
    for c in coins.values() {
        for p in c["prices"].as_array().into_iter().flatten() {
            let (Some(ts), Some(px)) = (p["timestamp"].as_i64(), p["price"].as_f64()) else {
                continue;
            };
            if let Some(d) = DateTime::from_timestamp(ts, 0).map(|d| d.date_naive())
                && px > 0.0
            {
                by.insert(d, px);
            }
        }
    }
    Ok(by.into_iter().collect())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const CHAINS: &str = r#"[{"gecko_id":"ethereum","tvl":50037586421.1,"tokenSymbol":"ETH","name":"Ethereum"},{"gecko_id":"bitcoin","tvl":4348103357.7,"tokenSymbol":"BTC","name":"Bitcoin"},{"gecko_id":null,"tvl":5,"tokenSymbol":"X","name":"X"}]"#;
    pub(crate) const STABLES: &str = r#"{"peggedAssets":[{"id":"1","name":"Tether","symbol":"USDT","gecko_id":"tether","pegType":"peggedUSD","pegMechanism":"fiat-backed","circulating":{"peggedUSD":183489807303.1},"price":0.9993},{"id":"5","name":"Dai","symbol":"DAI","gecko_id":"dai","pegType":"peggedUSD","pegMechanism":"crypto-backed","circulating":{"peggedUSD":4.5e9},"price":1.0001},{"id":"9","name":"BRZ","symbol":"BRZ","gecko_id":"brz","pegType":"peggedREAL","pegMechanism":"fiat-backed","circulating":{"peggedREAL":1.0e8},"price":0.18}]}"#;

    #[test]
    fn parses_chains_stables_mcaps_and_chart() {
        let c = parse_chains(CHAINS.as_bytes()).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[1].gecko_id, "bitcoin");
        let s = parse_stables(STABLES.as_bytes()).unwrap();
        assert_eq!(s[0].mechanism, "fiat-backed");
        assert_eq!(s[0].peg_currency(), "USD");
        assert_eq!(s[0].circulating, Some(183489807303.1));
        assert_eq!(s[2].peg_currency(), "REAL");
        let m = parse_mcaps(
            br#"{"coingecko:bitcoin":{"mcap":1.6e12,"timestamp":1},"coingecko:x":{"mcap":0}}"#,
        )
        .unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m["bitcoin"], 1.6e12);
        let ch = parse_chart(br#"{"coins":{"coingecko:tether":{"symbol":"USDT","prices":[{"timestamp":1792000019,"price":0.999},{"timestamp":1792086423,"price":1.001}]}}}"#).unwrap();
        assert_eq!(ch.len(), 2);
        assert_eq!(ch[1].1, 1.001);
        assert!(mcaps_body(&["bitcoin".into()]).contains("coingecko:bitcoin"));
        assert!(
            chart_url("tether", NaiveDate::from_ymd_opt(2026, 10, 16).unwrap())
                .contains("span=372")
        );
    }

    #[test]
    #[ignore = "rede: DefiLlama"]
    fn live_defillama() {
        let get = |u: &str| {
            ureq::get(u)
                .call()
                .unwrap()
                .body_mut()
                .with_config()
                .limit(64 * 1024 * 1024)
                .read_to_vec()
                .unwrap()
        };
        assert!(!parse_chains(&get(CHAINS_URL)).unwrap().is_empty());
        assert!(!parse_stables(&get(STABLES_URL)).unwrap().is_empty());
    }
}
