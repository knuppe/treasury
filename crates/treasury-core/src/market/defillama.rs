//! Preço diário de criptoativos pela DefiLlama (API aberta e gratuita, sem chave), em dólar,
//! pelo id da CoinGecko do ativo (docs/data-sources.md). É a fonte padrão da cotação de cripto:
//! o Yahoo só continua para quem o escolher no ativo. Respostas são dados, nunca instruções.

use chrono::{DateTime, NaiveDate};
use rust_decimal::Decimal;

use super::PriceRow;

/// Dias por pedido (a API recusa janelas muito maiores que um ano).
const SPAN_DAYS: i64 = 370;
/// Teto de pedidos por ativo numa coleta (~10 anos de histórico).
const MAX_WINDOWS: usize = 10;

fn chart_url(gecko_id: &str, start: NaiveDate, span: i64) -> String {
    let ts = start
        .and_hms_opt(0, 0, 0)
        .map(|d| d.and_utc().timestamp())
        .unwrap_or(0);
    format!("https://coins.llama.fi/chart/coingecko:{gecko_id}?start={ts}&span={span}&period=1d")
}

/// Pontos (instante Unix, preço em dólar) viram fechamentos diários. A API devolve um ponto
/// perto da meia-noite UTC de cada dia: é o fechamento do dia anterior (o dia de cripto fecha à
/// meia-noite UTC, como nas outras fontes). O ponto mais novo, no meio do dia, é a cotação de
/// hoje até agora.
pub fn bars(points: &[(i64, f64)]) -> Vec<PriceRow> {
    let mut by = std::collections::BTreeMap::new();
    for (ts, px) in points {
        let Some(at) = DateTime::from_timestamp(*ts, 0) else {
            continue;
        };
        let Some(close) = Decimal::from_f64_retain(*px).filter(|p| *p > Decimal::ZERO) else {
            continue;
        };
        let day = at.date_naive();
        let date = if ts.rem_euclid(86_400) < 3_600 {
            day.pred_opt().unwrap_or(day)
        } else {
            day
        };
        by.insert(date, close.round_dp(8));
    }
    by.into_iter()
        .map(|(date, close)| PriceRow {
            date,
            open: None,
            high: None,
            low: None,
            close,
            adj_close: None,
            volume: None,
        })
        .collect()
}

/// Lê `{"coins": {"coingecko:<id>": {"prices": [{"timestamp", "price"}]}}}`.
pub fn parse(body: &str) -> Result<Vec<(i64, f64)>, String> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-score-erro-defillama",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let coins = v["coins"]
        .as_object()
        .ok_or_else(|| crate::i18n::t("nucleo-score-erro-defillama-sem-precos"))?;
    let mut out = vec![];
    for c in coins.values() {
        for p in c["prices"].as_array().into_iter().flatten() {
            if let (Some(ts), Some(px)) = (p["timestamp"].as_i64(), p["price"].as_f64()) {
                out.push((ts, px));
            }
        }
    }
    Ok(out)
}

/// Fechamentos em dólar de `from` até hoje, em janelas de um ano, uma de cada vez.
pub fn fetch(gecko_id: &str, from: NaiveDate, today: NaiveDate) -> Result<Vec<PriceRow>, String> {
    let agent = crate::indices::sync::agent();
    let mut points = vec![];
    let mut start = from;
    for _ in 0..MAX_WINDOWS {
        if start > today {
            break;
        }
        let span = ((today - start).num_days() + 2).clamp(2, SPAN_DAYS);
        let url = chart_url(gecko_id, start, span);
        crate::research::net::pace(&url);
        points.extend(parse(&crate::indices::sync::get_text(&agent, &url)?)?);
        start += chrono::Duration::days(span);
    }
    Ok(bars(&points)
        .into_iter()
        .filter(|b| b.date >= from && b.date <= today)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    #[test]
    fn midnight_points_close_the_day_before() {
        // 2026-11-17 00:01 UTC, 2026-11-18 00:01 UTC e 2026-11-18 15:00 UTC.
        let body = r#"{"coins":{"coingecko:exemplo":{"symbol":"EXM","confidence":0.99,"prices":[
            {"timestamp":1794873660,"price":100.5},{"timestamp":1794960060,"price":101.25},
            {"timestamp":1795014000,"price":99.0},{"timestamp":1795014001,"price":0}]}}}"#;
        let rows = bars(&parse(body).unwrap());
        let got: Vec<(NaiveDate, Decimal)> = rows.iter().map(|r| (r.date, r.close)).collect();
        assert_eq!(
            got,
            vec![
                (d("2026-11-16"), dec!(100.5)),
                (d("2026-11-17"), dec!(101.25)),
                (d("2026-11-18"), dec!(99)),
            ]
        );
        assert!(parse("{}").is_err());
        assert!(chart_url("bitcoin", d("2026-01-01"), 370).ends_with("&span=370&period=1d"));
    }

    /// `TREASURY_LIVE=1 cargo test -p treasury-core --lib live_defillama_prices -- --ignored`
    #[test]
    #[ignore]
    fn live_defillama_prices() {
        if std::env::var("TREASURY_LIVE").is_err() {
            return;
        }
        let today = chrono::Utc::now().date_naive();
        let rows = fetch("bitcoin", today - chrono::Duration::days(500), today).unwrap();
        eprintln!("{} dias, último {:?}", rows.len(), rows.last());
        assert!(rows.len() > 480);
    }
}
