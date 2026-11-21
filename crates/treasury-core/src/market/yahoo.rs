//! Yahoo Finance (API de gráfico v8, não oficial e sem chave): fechamentos diários,
//! proventos e desdobramentos. Fica isolado aqui para trocar sem impacto se parar de
//! funcionar (docs/data-sources.md).
//!
//! Também lê as cotações intradiárias (`interval=1m|5m|15m`, `range=1d|5d`, ~15 min de
//! atraso) com o resumo da cotação que vem no `meta` do gráfico (preço, fechamento anterior,
//! horário e fuso do pregão), e a busca de tickers (`v1/finance/search`). Nenhum deles exige
//! chave. Bolsas que cotam em subunidades (LSE em pence `GBp`, JSE em `ZAc`, Tel Aviv em
//! `ILA`) são convertidas para a moeda principal.
//!
//! O `close` do Yahoo vem ajustado por desdobramentos. O Treasury guarda o preço
//! efetivamente negociado no dia (desfaz o ajuste com os fatores de desdobramento
//! posteriores), porque o motor da carteira aplica o desdobramento na quantidade na data
//! do evento; assim quantidade e preço estão sempre na mesma base.

use std::time::Duration;

use chrono::{DateTime, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Deserialize;

use super::PriceRow;

/// O app se identifica como é, sem se passar por navegador.
const UA: &str = crate::research::net::DEFAULT_USER_AGENT;

#[derive(Debug, Clone, PartialEq)]
pub struct YahooDividend {
    pub date: NaiveDate,
    pub amount: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YahooSplit {
    pub date: NaiveDate,
    /// Nova quantidade = antiga × fator (2 = desdobramento 1→2; 0,1 = grupamento 10→1).
    pub factor: Decimal,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct YahooSeries {
    pub currency: Option<String>,
    pub bars: Vec<PriceRow>,
    pub dividends: Vec<YahooDividend>,
    pub splits: Vec<YahooSplit>,
}

#[derive(Deserialize)]
struct Resp {
    chart: Chart,
}
#[derive(Deserialize)]
struct Chart {
    result: Option<Vec<ResultItem>>,
    error: Option<serde_json::Value>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Meta {
    currency: Option<String>,
    gmtoffset: Option<i64>,
    price_hint: Option<u32>,
    symbol: Option<String>,
    exchange_name: Option<String>,
    full_exchange_name: Option<String>,
    instrument_type: Option<String>,
    exchange_timezone_name: Option<String>,
    regular_market_time: Option<i64>,
    regular_market_price: Option<f64>,
    previous_close: Option<f64>,
    chart_previous_close: Option<f64>,
    regular_market_day_high: Option<f64>,
    regular_market_day_low: Option<f64>,
    regular_market_volume: Option<f64>,
    long_name: Option<String>,
    short_name: Option<String>,
    range: Option<String>,
    current_trading_period: Option<TradingPeriods>,
}
#[derive(Deserialize, Default)]
struct TradingPeriods {
    pre: Option<Period>,
    regular: Option<Period>,
    post: Option<Period>,
}
#[derive(Deserialize, Default, Clone, Copy)]
struct Period {
    start: i64,
    end: i64,
}
#[derive(Deserialize)]
struct ResultItem {
    meta: Meta,
    timestamp: Option<Vec<i64>>,
    indicators: Indicators,
    events: Option<Events>,
}
#[derive(Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
    adjclose: Option<Vec<AdjClose>>,
}
#[derive(Deserialize)]
struct Quote {
    open: Option<Vec<Option<f64>>>,
    high: Option<Vec<Option<f64>>>,
    low: Option<Vec<Option<f64>>>,
    close: Option<Vec<Option<f64>>>,
    volume: Option<Vec<Option<f64>>>,
}
#[derive(Deserialize)]
struct AdjClose {
    adjclose: Option<Vec<Option<f64>>>,
}
#[derive(Deserialize)]
struct Events {
    dividends: Option<std::collections::BTreeMap<String, DivEv>>,
    splits: Option<std::collections::BTreeMap<String, SplitEv>>,
}
#[derive(Deserialize)]
struct DivEv {
    amount: f64,
    date: i64,
}
#[derive(Deserialize)]
struct SplitEv {
    date: i64,
    numerator: f64,
    denominator: f64,
}

/// Subunidades de moeda usadas pelo Yahoo → (moeda ISO, fator para a unidade principal).
pub fn normalize_currency(c: &str) -> (String, Decimal) {
    match c {
        "GBp" | "GBX" => ("GBP".into(), Decimal::new(1, 2)),
        "ZAc" | "ZAC" => ("ZAR".into(), Decimal::new(1, 2)),
        "ILA" => ("ILS".into(), Decimal::new(1, 2)),
        other => (other.to_uppercase(), Decimal::ONE),
    }
}

fn local_date(ts: i64, offset: i64) -> Option<NaiveDate> {
    DateTime::from_timestamp(ts + offset, 0).map(|d| d.date_naive())
}

/// f64 do JSON → decimal pela representação mais curta, arredondado às casas do ativo.
fn dec(v: f64, dp: u32) -> Option<Decimal> {
    if !v.is_finite() {
        return None;
    }
    Decimal::from_str(&format!("{v}"))
        .ok()
        .map(|d| d.round_dp(dp).normalize())
}

/// "resposta do Yahoo inválida: …", no idioma da instalação.
fn bad_response(e: &serde_json::Error) -> String {
    crate::indices::providers::bad_response("Yahoo", e).to_string()
}

pub fn parse(body: &str) -> Result<YahooSeries, String> {
    let r: Resp = serde_json::from_str(body).map_err(|e| bad_response(&e))?;
    if let Some(err) = r.chart.error.filter(|e| !e.is_null()) {
        return Err(format!("Yahoo: {err}"));
    }
    let Some(item) = r.chart.result.and_then(|mut v| v.pop()) else {
        return Err(crate::i18n::t("nucleo-ativos-erro-yahoo-sem-resultado"));
    };
    let offset = item.meta.gmtoffset.unwrap_or(0);
    let (currency, unit) = match item.meta.currency.as_deref() {
        Some(c) => {
            let (c, f) = normalize_currency(c);
            (Some(c), f)
        }
        None => (None, Decimal::ONE),
    };
    // Em subunidades (pence), duas casas a mais para não perder precisão na conversão.
    let dp =
        item.meta.price_hint.unwrap_or(2).clamp(2, 8) + if unit == Decimal::ONE { 0 } else { 2 };
    let px = |v: f64, dp: u32| dec(v, dp + 2).map(|d| (d * unit).round_dp(dp).normalize());
    let mut out = YahooSeries {
        currency,
        ..Default::default()
    };
    if let Some(ev) = item.events {
        for (_, s) in ev.splits.unwrap_or_default() {
            if s.denominator > 0.0
                && s.numerator > 0.0
                && let (Some(d), Some(n), Some(den)) = (
                    local_date(s.date, offset),
                    dec(s.numerator, 8),
                    dec(s.denominator, 8),
                )
            {
                out.splits.push(YahooSplit {
                    date: d,
                    factor: (n / den).normalize(),
                });
            }
        }
        for (_, dv) in ev.dividends.unwrap_or_default() {
            if let (Some(d), Some(a)) = (local_date(dv.date, offset), px(dv.amount, 8)) {
                out.dividends.push(YahooDividend { date: d, amount: a });
            }
        }
    }
    out.splits.sort_by_key(|s| s.date);
    out.dividends.sort_by_key(|d| d.date);
    let ts = item.timestamp.unwrap_or_default();
    let q = item.indicators.quote.into_iter().next();
    let adj = item
        .indicators
        .adjclose
        .and_then(|mut v| v.pop())
        .and_then(|a| a.adjclose);
    let Some(q) = q else { return Ok(out) };
    let get = |v: &Option<Vec<Option<f64>>>, i: usize| {
        v.as_ref().and_then(|x| x.get(i).copied().flatten())
    };
    for (i, t) in ts.iter().enumerate() {
        let Some(date) = local_date(*t, offset) else {
            continue;
        };
        let Some(close) = get(&q.close, i).and_then(|c| px(c, dp)) else {
            continue;
        };
        // Desfaz o ajuste por desdobramentos posteriores à barra.
        let factor: Decimal = out
            .splits
            .iter()
            .filter(|s| s.date > date)
            .map(|s| s.factor)
            .product();
        let un = |v: Option<Decimal>| v.map(|x| (x * factor).round_dp(dp).normalize());
        out.bars.push(PriceRow {
            date,
            open: un(get(&q.open, i).and_then(|v| px(v, dp))),
            high: un(get(&q.high, i).and_then(|v| px(v, dp))),
            low: un(get(&q.low, i).and_then(|v| px(v, dp))),
            close: (close * factor).round_dp(dp).normalize(),
            adj_close: adj
                .as_ref()
                .and_then(|a| a.get(i).copied().flatten())
                .and_then(|v| px(v, dp)),
            volume: get(&q.volume, i).and_then(|v| dec(v, 0)),
        });
    }
    // Proventos do Yahoo também vêm ajustados: volta à base da data.
    for d in &mut out.dividends {
        let factor: Decimal = out
            .splits
            .iter()
            .filter(|s| s.date > d.date)
            .map(|s| s.factor)
            .product();
        d.amount = (d.amount * factor).round_dp(8).normalize();
    }
    // Uma barra por dia (a última do dia vence).
    out.bars.dedup_by(|b, a| {
        if a.date == b.date {
            *a = b.clone();
            true
        } else {
            false
        }
    });
    Ok(out)
}

pub fn fetch(symbol: &str, from: NaiveDate) -> Result<YahooSeries, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    let p1 = from
        .and_hms_opt(0, 0, 0)
        .expect("hora")
        .and_utc()
        .timestamp()
        - 86_400;
    let p2 = chrono::Utc::now().timestamp() + 86_400;
    let sym = symbol.replace('^', "%5E");
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{sym}?period1={p1}&period2={p2}&interval=1d&events=div%2Csplits&includeAdjustedClose=true"
    );
    let body = agent
        .get(&url)
        .header("User-Agent", UA)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .with_config()
        .limit(32 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    parse(&body)
}

// ── Intradiário e resumo da cotação ───────────────────────────────

/// Situação do pregão segundo o Yahoo (`currentTradingPeriod`), no instante da coleta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YahooMarketState {
    Pre,
    Regular,
    Post,
    Closed,
}

/// Resumo da cotação (o `meta` do gráfico; não exige chave nem cookie).
#[derive(Debug, Clone, PartialEq)]
pub struct YahooQuote {
    pub symbol: String,
    /// Moeda ISO já normalizada (GBp → GBP).
    pub currency: Option<String>,
    pub price: Decimal,
    pub previous_close: Option<Decimal>,
    pub day_high: Option<Decimal>,
    pub day_low: Option<Decimal>,
    pub volume: Option<Decimal>,
    /// Instante do último negócio (segundos Unix).
    pub market_time: i64,
    /// Fuso IANA da bolsa (America/Sao_Paulo).
    pub timezone: Option<String>,
    /// Código de bolsa do Yahoo (SAO, NMS, LSE…) e nome por extenso.
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub instrument_type: Option<String>,
    pub name: Option<String>,
    pub state: YahooMarketState,
    /// Início e fim do pregão regular corrente (ou do próximo), segundos Unix.
    pub regular_session: Option<(i64, i64)>,
}

impl YahooQuote {
    /// Variação sobre o fechamento anterior (razão: 0,0123 = +1,23%).
    pub fn change_ratio(&self) -> Option<Decimal> {
        self.previous_close
            .filter(|p| !p.is_zero())
            .map(|p| (self.price - p) / p)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntradayPoint {
    /// Início da barra (segundos Unix, UTC).
    pub ts: i64,
    pub price: Decimal,
    pub volume: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YahooIntraday {
    pub quote: YahooQuote,
    pub points: Vec<IntradayPoint>,
}

/// Intervalos aceitos pelo app (o Yahoo limita 1m a ~7 dias).
pub const INTERVALS: &[&str] = &["1m", "5m", "15m"];
pub const RANGES: &[&str] = &["1d", "5d"];

fn state_at(p: &TradingPeriods, now: i64) -> YahooMarketState {
    let within = |x: &Option<Period>| x.is_some_and(|p| now >= p.start && now < p.end);
    if within(&p.regular) {
        YahooMarketState::Regular
    } else if within(&p.pre) {
        YahooMarketState::Pre
    } else if within(&p.post) {
        YahooMarketState::Post
    } else {
        YahooMarketState::Closed
    }
}

/// Lê o gráfico intradiário: o resumo da cotação e os pontos (fechamento de cada barra).
/// `now` (segundos Unix) decide a situação do pregão.
pub fn parse_intraday(body: &str, now: i64) -> Result<YahooIntraday, String> {
    let r: Resp = serde_json::from_str(body).map_err(|e| bad_response(&e))?;
    if let Some(err) = r.chart.error.filter(|e| !e.is_null()) {
        return Err(format!("Yahoo: {err}"));
    }
    let Some(item) = r.chart.result.and_then(|mut v| v.pop()) else {
        return Err(crate::i18n::t("nucleo-ativos-erro-yahoo-sem-resultado"));
    };
    let m = &item.meta;
    let (currency, unit) = match m.currency.as_deref() {
        Some(c) => {
            let (c, f) = normalize_currency(c);
            (Some(c), f)
        }
        None => (None, Decimal::ONE),
    };
    let dp = m.price_hint.unwrap_or(2).clamp(2, 8) + if unit == Decimal::ONE { 0 } else { 2 };
    let px = |v: f64| dec(v, dp + 2).map(|d| (d * unit).round_dp(dp).normalize());
    let pos = |v: Option<f64>| v.filter(|x| *x > 0.0).and_then(px);
    let q = item.indicators.quote.first();
    let ts = item.timestamp.clone().unwrap_or_default();
    let mut points = vec![];
    if let Some(q) = q {
        for (i, t) in ts.iter().enumerate() {
            let close = q
                .close
                .as_ref()
                .and_then(|c| c.get(i).copied().flatten())
                .and_then(px);
            let Some(price) = close.filter(|p| *p > Decimal::ZERO) else {
                continue;
            };
            let volume = q
                .volume
                .as_ref()
                .and_then(|v| v.get(i).copied().flatten())
                .and_then(|v| dec(v, 0));
            points.push(IntradayPoint {
                ts: *t,
                price,
                volume,
            });
        }
    }
    let last_point = points.last().cloned();
    let price = pos(m.regular_market_price)
        .or(last_point.as_ref().map(|p| p.price))
        .ok_or_else(|| crate::i18n::t("nucleo-ativos-erro-yahoo-sem-preco"))?;
    let market_time = m
        .regular_market_time
        .or(last_point.as_ref().map(|p| p.ts))
        .ok_or_else(|| crate::i18n::t("nucleo-ativos-erro-yahoo-sem-horario"))?;
    // `chartPreviousClose` só é o fechamento anterior quando o gráfico cobre um dia.
    let previous_close = pos(m.previous_close).or(if m.range.as_deref() == Some("1d") {
        pos(m.chart_previous_close)
    } else {
        None
    });
    let periods = m.current_trading_period.as_ref();
    Ok(YahooIntraday {
        quote: YahooQuote {
            symbol: m.symbol.clone().unwrap_or_default(),
            currency,
            price,
            previous_close,
            day_high: pos(m.regular_market_day_high),
            day_low: pos(m.regular_market_day_low),
            volume: m
                .regular_market_volume
                .filter(|v| *v > 0.0)
                .and_then(|v| dec(v, 0)),
            market_time,
            timezone: m.exchange_timezone_name.clone(),
            exchange: m.exchange_name.clone(),
            exchange_name: m.full_exchange_name.clone(),
            instrument_type: m.instrument_type.clone(),
            name: m
                .long_name
                .clone()
                .or(m.short_name.clone())
                .map(|n| n.split_whitespace().collect::<Vec<_>>().join(" ")),
            state: periods
                .map(|p| state_at(p, now))
                .unwrap_or(YahooMarketState::Closed),
            regular_session: periods.and_then(|p| p.regular).map(|r| (r.start, r.end)),
        },
        points,
    })
}

fn agent(secs: u64) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(secs)))
        .build()
        .into()
}

fn get_json(url: &str, secs: u64) -> Result<String, String> {
    agent(secs)
        .get(url)
        .header("User-Agent", UA)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .with_config()
        .limit(8 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())
}

/// Gráfico intradiário com ~15 min de atraso (`interval` 1m/5m/15m, `range` 1d/5d).
pub fn fetch_intraday(symbol: &str, interval: &str, range: &str) -> Result<YahooIntraday, String> {
    if !INTERVALS.contains(&interval) || !RANGES.contains(&range) {
        return Err(crate::i18n::t_args(
            "nucleo-ativos-erro-yahoo-intervalo",
            &crate::i18n::args(&[
                ("intervalo", interval.to_string().into()),
                ("periodo", range.to_string().into()),
            ]),
        ));
    }
    let sym = symbol.replace('^', "%5E");
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{sym}?interval={interval}&range={range}&includePrePost=false"
    );
    parse_intraday(&get_json(&url, 20)?, chrono::Utc::now().timestamp())
}

// ── Busca de tickers ─────────────────────────────────────────────

/// Um resultado da busca do Yahoo.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub symbol: String,
    pub name: String,
    /// Código de bolsa do Yahoo (SAO, NMS, LSE, JPX…).
    pub exchange: Option<String>,
    pub exchange_display: Option<String>,
    /// EQUITY, ETF, MUTUALFUND, CRYPTOCURRENCY, INDEX…
    pub quote_type: String,
    pub sector: Option<String>,
    pub industry: Option<String>,
}

#[derive(Deserialize)]
struct SearchResp {
    #[serde(default)]
    quotes: Vec<SearchQuote>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuote {
    symbol: Option<String>,
    shortname: Option<String>,
    longname: Option<String>,
    exchange: Option<String>,
    exch_disp: Option<String>,
    quote_type: Option<String>,
    sector: Option<String>,
    industry: Option<String>,
}

/// Tipos que viram ativo no Treasury (índices, moedas e futuros ficam de fora).
const SEARCH_TYPES: &[&str] = &["EQUITY", "ETF", "MUTUALFUND", "CRYPTOCURRENCY"];

/// Código à vista da B3 (PETR4, HGLG11, AAPL34): exclui fracionário (PETR4F) e afins.
fn b3_spot(ticker: &str) -> bool {
    let b = ticker.as_bytes();
    (5..=6).contains(&b.len())
        && b[..4].iter().all(u8::is_ascii_alphabetic)
        && b[4..].iter().all(u8::is_ascii_digit)
}

pub fn parse_search(body: &str) -> Result<Vec<SearchHit>, String> {
    let r: SearchResp = serde_json::from_str(body).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-ativos-erro-yahoo-busca",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    Ok(r.quotes
        .into_iter()
        .filter_map(|q| {
            let symbol = q.symbol?.trim().to_uppercase();
            let quote_type = q.quote_type?.to_uppercase();
            if symbol.is_empty() || !SEARCH_TYPES.contains(&quote_type.as_str()) {
                return None;
            }
            if let Some(t) = symbol.strip_suffix(".SA")
                && !b3_spot(t)
            {
                return None;
            }
            let name = q.longname.or(q.shortname).unwrap_or_else(|| symbol.clone());
            Some(SearchHit {
                symbol,
                name: name.split_whitespace().collect::<Vec<_>>().join(" "),
                exchange: q.exchange,
                exchange_display: q.exch_disp,
                quote_type,
                sector: q.sector,
                industry: q.industry,
            })
        })
        .collect())
}

/// Busca de tickers e nomes no Yahoo (sem chave).
pub fn search(query: &str, count: u32) -> Result<Vec<SearchHit>, String> {
    let q: String = query
        .trim()
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect();
    let url = format!(
        "https://query2.finance.yahoo.com/v1/finance/search?q={q}&quotesCount={count}&newsCount=0&listsCount=0&enableFuzzyQuery=false"
    );
    parse_search(&get_json(&url, 10)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const SAMPLE: &str = r#"{"chart":{"result":[{"meta":{"currency":"BRL","gmtoffset":-10800,"priceHint":2},
      "timestamp":[1791205200,1791291600,1791378000],
      "indicators":{"quote":[{"open":[10.0,10.2,null],"high":[10.5,10.4,null],"low":[9.9,10.0,null],
                              "close":[10.10000038,10.29999924,null],"volume":[1000,2000,null]}],
                    "adjclose":[{"adjclose":[9.8,10.0,null]}]},
      "events":{"splits":{"1791291600":{"date":1791291600,"numerator":2,"denominator":1}},
                "dividends":{"1791205200":{"amount":0.25,"date":1791205200}}}}],"error":null}}"#;

    #[test]
    fn parses_and_undoes_split_adjustment() {
        let s = parse(SAMPLE).unwrap();
        assert_eq!(s.currency.as_deref(), Some("BRL"));
        assert_eq!(s.bars.len(), 2, "a bar with no close is discarded");
        // 1st bar is before the 2:1 split → real price = 10.10 × 2.
        assert_eq!(s.bars[0].close, dec!(20.2));
        assert_eq!(s.bars[1].close, dec!(10.3));
        assert_eq!(s.splits[0].factor, dec!(2));
        assert_eq!(
            s.dividends[0].amount,
            dec!(0.5),
            "dividend goes back to the date's base"
        );
        assert_eq!(s.bars[0].date.to_string(), "2026-10-05");
    }

    #[test]
    fn reports_errors() {
        assert!(parse(r#"{"chart":{"result":null,"error":{"code":"Not Found"}}}"#).is_err());
        assert!(parse("<html>").is_err());
    }
}
