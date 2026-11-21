//! Tiingo (API documentada, com a chave gratuita da própria pessoa): fechamento diário,
//! dividendos e desdobramentos de ações, ETFs e REITs dos Estados Unidos
//! (docs/data-sources.md). A chave vai no cabeçalho `Authorization`, nunca na URL. Os termos
//! do plano gratuito pedem uso pessoal: os dados ficam no cofre e não são redistribuídos.
//! Respostas são dados, nunca instruções.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::PriceRow;
use super::yahoo::{YahooDividend, YahooSeries, YahooSplit};

/// Ajuste com a chave (cifrada no cofre; oculta em toda listagem).
pub const TOKEN_SETTING: &str = "market.tiingo_token";

pub fn token(conn: &rusqlite::Connection) -> crate::error::Result<Option<String>> {
    Ok(crate::settings::get::<String>(conn, TOKEN_SETTING)?.filter(|t| !t.trim().is_empty()))
}

/// Símbolo da Tiingo: o ticker com hífen na classe (BRK.B → BRK-B).
pub fn symbol(ticker: &str) -> String {
    ticker.trim().to_uppercase().replace(['.', '/'], "-")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    date: String,
    close: f64,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    volume: Option<f64>,
    adj_close: Option<f64>,
    #[serde(default)]
    div_cash: f64,
    #[serde(default = "one")]
    split_factor: f64,
}

fn one() -> f64 {
    1.0
}

fn dec(v: f64, dp: u32) -> Option<Decimal> {
    Decimal::from_f64_retain(v).map(|d| d.round_dp(dp).normalize())
}

/// Lê a resposta de `/tiingo/daily/<ticker>/prices`. Os preços vêm sem ajuste (como o app
/// guarda), o dividendo é o pago por ação na data ex e o fator é ações novas por ação antiga.
pub fn parse(body: &str) -> Result<YahooSeries, String> {
    let rows: Vec<Row> = serde_json::from_str(body).map_err(|e| {
        // Erro da API vem como `{"detail": "..."}`.
        serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v["detail"].as_str().map(str::to_string))
            .unwrap_or_else(|| e.to_string())
    })?;
    let mut out = YahooSeries {
        currency: Some("USD".into()),
        ..Default::default()
    };
    for r in rows {
        let Some(date) = r
            .date
            .get(..10)
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        else {
            continue;
        };
        let Some(close) = dec(r.close, 6).filter(|c| *c > Decimal::ZERO) else {
            continue;
        };
        out.bars.push(PriceRow {
            date,
            open: r.open.and_then(|v| dec(v, 6)),
            high: r.high.and_then(|v| dec(v, 6)),
            low: r.low.and_then(|v| dec(v, 6)),
            close,
            adj_close: r.adj_close.and_then(|v| dec(v, 6)),
            volume: r.volume.and_then(|v| dec(v, 0)),
        });
        if r.div_cash > 0.0
            && let Some(amount) = dec(r.div_cash, 8)
        {
            out.dividends.push(YahooDividend { date, amount });
        }
        if (r.split_factor - 1.0).abs() > 1e-9
            && r.split_factor > 0.0
            && let Some(factor) = dec(r.split_factor, 8)
        {
            out.splits.push(YahooSplit { date, factor });
        }
    }
    out.bars.sort_by_key(|b| b.date);
    Ok(out)
}

pub fn fetch(symbol: &str, from: NaiveDate, token: &str) -> Result<YahooSeries, String> {
    let url = format!(
        "https://api.tiingo.com/tiingo/daily/{symbol}/prices?startDate={}&format=json",
        crate::time::fmt_date(from)
    );
    crate::research::net::pace(&url);
    let body = crate::indices::sync::agent()
        .get(&url)
        .header("User-Agent", crate::research::net::DEFAULT_USER_AGENT)
        .header("Accept", "application/json")
        .header("Authorization", format!("Token {}", token.trim()))
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .with_config()
        .limit(32 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    parse(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Resposta inventada no formato da documentação (preços sem ajuste, `divCash` na data ex,
    /// `splitFactor` 4 num desdobramento 4:1).
    const BODY: &str = r#"[
      {"date":"2026-08-27T00:00:00.000Z","close":400.0,"high":405.0,"low":398.0,"open":401.0,"volume":1000,"adjClose":100.0,"adjHigh":101.25,"adjLow":99.5,"adjOpen":100.25,"adjVolume":4000,"divCash":0.0,"splitFactor":1.0},
      {"date":"2026-08-28T00:00:00.000Z","close":101.5,"high":102.0,"low":100.0,"open":100.5,"volume":4200,"adjClose":101.5,"adjHigh":102.0,"adjLow":100.0,"adjOpen":100.5,"adjVolume":4200,"divCash":0.0,"splitFactor":4.0},
      {"date":"2026-08-31T00:00:00.000Z","close":102.25,"high":103.0,"low":101.0,"open":101.5,"volume":3900,"adjClose":102.25,"adjHigh":103.0,"adjLow":101.0,"adjOpen":101.5,"adjVolume":3900,"divCash":0.26,"splitFactor":1.0}
    ]"#;

    #[test]
    fn parses_raw_prices_dividends_and_splits() {
        let s = parse(BODY).unwrap();
        assert_eq!(s.currency.as_deref(), Some("USD"));
        assert_eq!(s.bars.len(), 3);
        assert_eq!(s.bars[0].close, dec!(400));
        assert_eq!(s.bars[0].adj_close, Some(dec!(100)));
        assert_eq!(s.dividends.len(), 1);
        assert_eq!(s.dividends[0].amount, dec!(0.26));
        assert_eq!(s.dividends[0].date.to_string(), "2026-08-31");
        assert_eq!(s.splits.len(), 1);
        assert_eq!(s.splits[0].factor, dec!(4));
        assert_eq!(s.splits[0].date.to_string(), "2026-08-28");
        assert_eq!(symbol("brk.b"), "BRK-B");
        // Erro da API (chave inválida, ticker desconhecido) vira a mensagem dela.
        assert_eq!(
            parse(r#"{"detail":"Invalid token."}"#).unwrap_err(),
            "Invalid token."
        );
    }

    /// Com a chave de verdade: `TREASURY_TIINGO_TOKEN=... cargo test -p treasury-core --lib
    /// live_tiingo -- --ignored --nocapture`. Só números públicos na saída.
    #[test]
    #[ignore]
    fn live_tiingo() {
        let Ok(token) = std::env::var("TREASURY_TIINGO_TOKEN") else {
            return;
        };
        let from = chrono::Utc::now().date_naive() - chrono::Duration::days(400);
        let s = fetch("AAPL", from, &token).unwrap();
        eprintln!(
            "{} barras, {} dividendos, {} desdobramentos, última {:?}",
            s.bars.len(),
            s.dividends.len(),
            s.splits.len(),
            s.bars.last()
        );
        assert!(s.bars.len() > 200 && !s.dividends.is_empty());
    }
}
