//! EODHD (API documentada, com a chave da própria pessoa): fechamento diário, dividendos e
//! desdobramentos de bolsas do mundo todo (docs/data-sources.md). Os termos permitem ao
//! usuário não profissional guardar e analisar os dados para uso privado; nada é
//! redistribuído. A API só aceita a chave na URL (`api_token`): por isso a URL nunca vai para
//! log, relatório de coleta ou mensagem de erro. Escrito pela documentação: ainda não
//! conferido com uma chave real. Respostas são dados, nunca instruções.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::PriceRow;
use super::yahoo::{YahooDividend, YahooSeries, YahooSplit, normalize_currency};

/// Ajuste com a chave (cifrada no cofre; oculta em toda listagem).
pub const TOKEN_SETTING: &str = "market.eodhd_token";

pub fn token(conn: &rusqlite::Connection) -> crate::error::Result<Option<String>> {
    Ok(crate::settings::get::<String>(conn, TOKEN_SETTING)?.filter(|t| !t.trim().is_empty()))
}

/// Código da bolsa na EODHD a partir do código do catálogo de bolsas do app. Bolsa fora da
/// lista: a pessoa informa o símbolo completo (`TICKER.BOLSA`) no símbolo do provedor.
pub fn exchange_code(code: &str) -> Option<&'static str> {
    Some(match code {
        "NYSE" | "NASDAQ" | "AMEX" | "ARCA" | "BATS" => "US",
        "B3" => "SA",
        "TSX" => "TO",
        "TSXV" => "V",
        "BMV" => "MX",
        "BYMA" => "BA",
        "BCS" => "SN",
        "LSE" => "LSE",
        "XETRA" => "XETRA",
        "FWB" => "F",
        "EPA" => "PA",
        "AMS" => "AS",
        "EBR" => "BR",
        "ELI" => "LS",
        "ISE" => "IR",
        "MIL" => "MI",
        "BME" => "MC",
        "SIX" => "SW",
        "VIE" => "VI",
        "STO" => "ST",
        "CPH" => "CO",
        "HEL" => "HE",
        "OSL" => "OL",
        "WSE" => "WAR",
        "BIST" => "IS",
        "JSE" => "JSE",
        "TASE" => "TA",
        "TADAWUL" => "SR",
        "NSE" => "NSE",
        "BSE" => "BSE",
        "SSE" => "SHG",
        "SZSE" => "SHE",
        "HKEX" => "HK",
        "TSE" => "TSE",
        "KRX" => "KO",
        "KOSDAQ" => "KQ",
        "TWSE" => "TW",
        "TPEX" => "TWO",
        "IDX" => "JK",
        "SET" => "BK",
        "ASX" => "AU",
        "NZX" => "NZ",
        _ => return None,
    })
}

/// `TICKER.BOLSA` (BRK.B → BRK-B.US).
pub fn symbol(ticker: &str, exchange: &str) -> String {
    format!(
        "{}.{exchange}",
        ticker.trim().to_uppercase().replace(['.', '/'], "-")
    )
}

/// Bolsas que cotam em subunidade (pence, centavos de rand, agorot): fator para a moeda.
fn price_factor(symbol: &str) -> Decimal {
    match symbol.rsplit('.').next() {
        Some("LSE" | "JSE" | "TA") => Decimal::new(1, 2),
        _ => Decimal::ONE,
    }
}

#[derive(Deserialize)]
struct Bar {
    date: String,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: f64,
    adjusted_close: Option<f64>,
    volume: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Div {
    date: String,
    value: Option<f64>,
    unadjusted_value: Option<f64>,
    currency: Option<String>,
}

#[derive(Deserialize)]
struct Split {
    date: String,
    split: String,
}

fn dec(v: f64, dp: u32) -> Option<Decimal> {
    Decimal::from_f64_retain(v).map(|d| d.round_dp(dp).normalize())
}

fn day(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.get(..10)?, "%Y-%m-%d").ok()
}

fn json<T: serde::de::DeserializeOwned>(body: &str) -> Result<Vec<T>, String> {
    serde_json::from_str(body).map_err(|e| {
        // Erros vêm como texto simples ("Unauthenticated", "Ticker Not Found.").
        let t = body.trim();
        if t.is_empty() || t.len() > 120 || t.starts_with(['[', '{']) {
            e.to_string()
        } else {
            t.to_string()
        }
    })
}

/// Junta as três respostas (`/api/eod`, `/api/div`, `/api/splits`). Preços sem ajuste;
/// dividendo pelo valor pago na data (`unadjustedValue`), na moeda principal; fator = novas
/// ações por antiga (`"4.000000/1.000000"` → 4).
pub fn parse(symbol: &str, eod: &str, div: &str, splits: &str) -> Result<YahooSeries, String> {
    let f = price_factor(symbol);
    let px = |v: f64| dec(v, 6).map(|d| (d * f).normalize());
    let mut out = YahooSeries::default();
    for b in json::<Bar>(eod)? {
        let (Some(date), Some(close)) = (day(&b.date), px(b.close)) else {
            continue;
        };
        if close <= Decimal::ZERO {
            continue;
        }
        out.bars.push(PriceRow {
            date,
            open: b.open.and_then(px),
            high: b.high.and_then(px),
            low: b.low.and_then(px),
            close,
            adj_close: b.adjusted_close.and_then(px),
            volume: b.volume.and_then(|v| dec(v, 0)),
        });
    }
    out.bars.sort_by_key(|b| b.date);
    for d in json::<Div>(div)? {
        let Some(date) = day(&d.date) else { continue };
        let Some(v) = d.unadjusted_value.or(d.value).filter(|v| *v > 0.0) else {
            continue;
        };
        // A moeda do dividendo pode ser a subunidade (GBX) mesmo quando a resposta diz GBP.
        let k = d
            .currency
            .as_deref()
            .map(|c| normalize_currency(c).1)
            .unwrap_or(Decimal::ONE);
        if let Some(amount) = dec(v, 8).map(|a| (a * k).normalize()) {
            out.dividends.push(YahooDividend { date, amount });
        }
    }
    for s in json::<Split>(splits)? {
        let Some(date) = day(&s.date) else { continue };
        let mut parts = s.split.split('/').map(|p| p.trim().parse::<f64>());
        let (Some(Ok(new)), Some(Ok(old))) = (parts.next(), parts.next()) else {
            continue;
        };
        if new > 0.0
            && old > 0.0
            && (new - old).abs() > 1e-9
            && let Some(factor) = dec(new / old, 8)
        {
            out.splits.push(YahooSplit { date, factor });
        }
    }
    Ok(out)
}

fn get(path: &str, symbol: &str, from: NaiveDate, token: &str) -> Result<String, String> {
    let url = format!(
        "https://eodhd.com/api/{path}/{symbol}?from={}&fmt=json&api_token={}",
        crate::time::fmt_date(from),
        token.trim()
    );
    crate::research::net::pace("https://eodhd.com/api");
    crate::indices::sync::agent()
        .get(&url)
        .header("User-Agent", crate::research::net::DEFAULT_USER_AGENT)
        .header("Accept", "application/json")
        .call()
        // O erro do cliente HTTP pode citar a URL, que leva a chave: fica só o que não a contém.
        .map_err(|e| match e {
            ureq::Error::StatusCode(c) => format!("HTTP {c}"),
            _ => crate::i18n::t("nucleo-ativos-erro-eodhd-rede"),
        })?
        .body_mut()
        .with_config()
        .limit(32 * 1024 * 1024)
        .read_to_string()
        .map_err(|_| crate::i18n::t("nucleo-ativos-erro-eodhd-rede"))
}

pub fn fetch(symbol: &str, from: NaiveDate, token: &str) -> Result<YahooSeries, String> {
    let eod = get("eod", symbol, from, token)?;
    let div = get("div", symbol, from, token)?;
    let splits = get("splits", symbol, from, token)?;
    parse(symbol, &eod, &div, &splits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const EOD: &str = r#"[{"date":"2026-08-28","open":295.05,"high":297.28,"low":295.05,"close":297.04,"adjusted_close":277.9465,"volume":4458400},
        {"date":"2026-08-31","open":74.5,"high":75.0,"low":74.0,"close":74.25,"adjusted_close":74.25,"volume":9000000}]"#;
    const DIV: &str = r#"[{"date":"2026-05-11","declarationDate":"2026-04-30","recordDate":"2026-05-11","paymentDate":"2026-05-14","period":"Quarterly","value":0.0675,"unadjustedValue":0.27,"currency":"USD"}]"#;
    const SPLITS: &str = r#"[{"date":"2026-08-31","split":"4.000000/1.000000"},{"date":"2020-01-02","split":"1.000000/10.000000"}]"#;

    #[test]
    fn joins_prices_dividends_and_splits() {
        let s = parse("EXEM.US", EOD, DIV, SPLITS).unwrap();
        assert_eq!(s.bars.len(), 2);
        assert_eq!(s.bars[0].close, dec!(297.04));
        // The dividend is the one paid on the date, not the one adjusted for the later split.
        assert_eq!(s.dividends[0].amount, dec!(0.27));
        assert_eq!(s.splits[0].factor, dec!(4));
        assert_eq!(s.splits[1].factor, dec!(0.1));
        // London quotes in pence and pays dividends in pence (GBX): everything becomes pounds.
        let l = parse(
            "EXEM.LSE",
            EOD,
            &DIV.replace(r#""currency":"USD""#, r#""currency":"GBX""#),
            "[]",
        )
        .unwrap();
        assert_eq!(l.bars[0].close, dec!(2.9704));
        assert_eq!(l.dividends[0].amount, dec!(0.0027));
        assert_eq!(
            parse("X.US", "Unauthenticated", "[]", "[]").unwrap_err(),
            "Unauthenticated"
        );
        assert_eq!(symbol("brk.b", "US"), "BRK-B.US");
        assert_eq!(exchange_code("TSE"), Some("TSE"));
        assert_eq!(exchange_code("SGX"), None);
    }
}
