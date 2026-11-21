//! CoinGecko, API pública (sem chave; ~10 pedidos por minuto): ids pelo símbolo, dados de
//! mercado, oferta, desenvolvimento e o histórico de 12 meses (para a inflação da oferta).
//! Respostas são dados, nunca instruções.

use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};

const BASE: &str = "https://api.coingecko.com/api/v3";

/// Moedas por símbolo (as de maior valor de mercado primeiro).
pub fn markets_url(symbols: &[String]) -> String {
    let s: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
    format!(
        "{BASE}/coins/markets?vs_currency=usd&symbols={}&include_tokens=top&per_page=250",
        s.join(",")
    )
}

pub fn coin_url(id: &str) -> String {
    format!(
        "{BASE}/coins/{id}?localization=false&tickers=false&market_data=true&community_data=true&developer_data=true&sparkline=false"
    )
}

pub fn chart_url(id: &str) -> String {
    format!("{BASE}/coins/{id}/market_chart?vs_currency=usd&days=365&interval=daily")
}

/// Símbolo → id (o de maior valor de mercado entre os de mesmo símbolo).
pub fn parse_markets(bytes: &[u8]) -> Result<Vec<(String, String)>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-coingecko",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let rows = v
        .as_array()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-coingecko-sem-moedas"))?;
    let mut best: std::collections::BTreeMap<String, (f64, String)> = Default::default();
    for r in rows {
        let (Some(id), Some(sym)) = (r["id"].as_str(), r["symbol"].as_str()) else {
            continue;
        };
        let mc = r["market_cap"].as_f64().unwrap_or(0.0);
        let sym = sym.to_uppercase();
        if best.get(&sym).is_none_or(|b| mc > b.0) {
            best.insert(sym, (mc, id.to_string()));
        }
    }
    Ok(best.into_iter().map(|(s, (_, id))| (s, id)).collect())
}

/// Dados de uma moeda (`/coins/{id}`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Coin {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub categories: Vec<String>,
    pub genesis_date: Option<NaiveDate>,
    pub rank: Option<f64>,
    pub watchlist_users: Option<f64>,
    pub price_usd: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub fdv_usd: Option<f64>,
    pub volume_usd: Option<f64>,
    pub ath_usd: Option<f64>,
    /// Distância da máxima histórica (fração positiva).
    pub ath_drawdown: Option<f64>,
    pub circulating: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub commits_4w: Option<f64>,
    pub stars: Option<f64>,
}

fn pos(v: &serde_json::Value) -> Option<f64> {
    v.as_f64().filter(|x| x.is_finite() && *x > 0.0)
}

pub fn parse_coin(bytes: &[u8]) -> Result<Coin, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-coingecko",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let id = v["id"]
        .as_str()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-coingecko-sem-id"))?;
    let md = &v["market_data"];
    let usd = |k: &str| pos(&md[k]["usd"]);
    Ok(Coin {
        id: id.to_string(),
        name: v["name"].as_str().unwrap_or_default().to_string(),
        symbol: v["symbol"].as_str().unwrap_or_default().to_uppercase(),
        categories: v["categories"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| c.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        genesis_date: v["genesis_date"]
            .as_str()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        rank: pos(&v["market_cap_rank"]).or_else(|| pos(&md["market_cap_rank"])),
        watchlist_users: pos(&v["watchlist_portfolio_users"]),
        price_usd: usd("current_price"),
        market_cap_usd: usd("market_cap"),
        fdv_usd: usd("fully_diluted_valuation"),
        volume_usd: usd("total_volume"),
        ath_usd: usd("ath"),
        ath_drawdown: md["ath_change_percentage"]["usd"]
            .as_f64()
            .map(|x| (-x / 100.0).max(0.0)),
        circulating: pos(&md["circulating_supply"]),
        total_supply: pos(&md["total_supply"]),
        max_supply: pos(&md["max_supply"]),
        commits_4w: v["developer_data"]["commit_count_4_weeks"].as_f64(),
        stars: v["developer_data"]["stars"].as_f64(),
    })
}

pub fn is_stablecoin(c: &Coin) -> bool {
    c.categories
        .iter()
        .any(|x| x.to_lowercase().contains("stablecoin"))
}

/// Crescimento da oferta em circulação em 12 meses (valor de mercado ÷ preço no primeiro e
/// no último ponto do histórico).
pub fn parse_supply_growth(bytes: &[u8]) -> Result<Option<(NaiveDate, f64)>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-coingecko",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let pts = |k: &str| -> Vec<(i64, f64)> {
        v[k].as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|p| Some((p[0].as_f64()? as i64, pos(&p[1])?)))
                    .collect()
            })
            .unwrap_or_default()
    };
    let (prices, caps) = (pts("prices"), pts("market_caps"));
    let supply = |ts: i64| -> Option<f64> {
        let p = prices.iter().find(|x| x.0 == ts)?.1;
        let c = caps.iter().find(|x| x.0 == ts)?.1;
        Some(c / p)
    };
    let (Some(first), Some(last)) = (caps.first(), caps.last()) else {
        return Ok(None);
    };
    let span = (last.0 - first.0) / 86_400_000;
    if span < 300 {
        return Ok(None);
    }
    let (Some(a), Some(b)) = (supply(first.0), supply(last.0)) else {
        return Ok(None);
    };
    let date = DateTime::from_timestamp_millis(last.0).map(|d| d.date_naive());
    Ok(date.map(|d| (d, b / a - 1.0)))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const COIN: &str = r#"{"id":"bitcoin","symbol":"btc","name":"Bitcoin","categories":["Smart Contract Platform","Layer 1 (L1)","FTX Holdings"],"genesis_date":"2009-01-03","market_cap_rank":1,"watchlist_portfolio_users":2000000,"market_data":{"current_price":{"usd":80000.0},"ath":{"usd":126000.0},"ath_change_percentage":{"usd":-36.5},"market_cap":{"usd":1600000000000.0},"market_cap_rank":1,"fully_diluted_valuation":{"usd":1680000000000.0},"total_volume":{"usd":40000000000.0},"total_supply":19900000.0,"max_supply":21000000.0,"circulating_supply":19900000.0},"developer_data":{"forks":36000,"stars":73000,"commit_count_4_weeks":120}}"#;
    pub(crate) const STABLE: &str = r#"{"id":"tether","symbol":"usdt","name":"Tether","categories":["Stablecoins","USD Stablecoin"],"genesis_date":null,"market_cap_rank":3,"market_data":{"current_price":{"usd":0.9994},"ath":{"usd":1.32},"ath_change_percentage":{"usd":-24.4},"market_cap":{"usd":183000000000.0},"fully_diluted_valuation":{"usd":183000000000.0},"total_volume":{"usd":90000000000.0},"total_supply":183000000000.0,"max_supply":null,"circulating_supply":183000000000.0},"developer_data":{"stars":0,"commit_count_4_weeks":0}}"#;

    #[test]
    fn parses_coin_markets_and_chart() {
        let c = parse_coin(COIN.as_bytes()).unwrap();
        assert_eq!(c.id, "bitcoin");
        assert_eq!(c.symbol, "BTC");
        assert_eq!(c.rank, Some(1.0));
        assert_eq!(c.genesis_date, NaiveDate::from_ymd_opt(2009, 1, 3));
        assert!((c.ath_drawdown.unwrap() - 0.365).abs() < 1e-12);
        assert_eq!(c.max_supply, Some(21_000_000.0));
        assert_eq!(c.commits_4w, Some(120.0));
        assert!(!is_stablecoin(&c));
        let t = parse_coin(STABLE.as_bytes()).unwrap();
        assert!(is_stablecoin(&t));
        assert_eq!(t.max_supply, None);
        assert_eq!(t.genesis_date, None);
        // Dois "ETH": vale o de maior valor de mercado.
        let m = br#"[{"id":"ethereum","symbol":"eth","market_cap":3e11},{"id":"eth-fake","symbol":"eth","market_cap":5},{"id":"bitcoin","symbol":"btc","market_cap":1.6e12}]"#;
        let ids = parse_markets(m).unwrap();
        assert_eq!(
            ids,
            vec![
                ("BTC".to_string(), "bitcoin".to_string()),
                ("ETH".into(), "ethereum".into())
            ]
        );
        // Oferta: 100 → 102 unidades (valor de mercado ÷ preço) em 365 dias.
        let day = 86_400_000i64;
        let chart = format!(
            r#"{{"prices":[[0,10.0],[{a},20.0]],"market_caps":[[0,1000.0],[{a},2040.0]],"total_volumes":[]}}"#,
            a = 365 * day
        );
        let (d, g) = parse_supply_growth(chart.as_bytes()).unwrap().unwrap();
        assert!((g - 0.02).abs() < 1e-12);
        assert_eq!(d, NaiveDate::from_ymd_opt(1971, 1, 1).unwrap());
        assert!(parse_coin(b"{}").is_err());
    }

    #[test]
    #[ignore = "rede: CoinGecko"]
    fn live_coingecko() {
        let body = ureq::get(&coin_url("bitcoin"))
            .header("User-Agent", crate::research::net::DEFAULT_USER_AGENT)
            .call()
            .unwrap()
            .body_mut()
            .read_to_vec()
            .unwrap();
        let c = parse_coin(&body).unwrap();
        assert!(c.market_cap_usd.unwrap() > 1e11, "{c:?}");
    }
}
