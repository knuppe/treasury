//! brapi.dev (opcional, com token do usuário): segunda fonte de indicadores e proventos de
//! ações e FIIs da B3, usada só para conferir os dados oficiais (CVM, B3) e, quando a fonte
//! oficial não tem um dado, como fonte secundária marcada. Só os ativos cadastrados, no máximo
//! uma vez por dia por papel, com o ritmo por site de [`super::net`]. O token vai no cabeçalho
//! `Authorization`, nunca na URL (que aparece no relatório da coleta).
//!
//! Plano e cota (docs/brazil/brapi.md): o plano e o consumo vêm da própria API (`/api/v2/user/usage`,
//! lidos no máximo a cada [`USAGE_REFRESH_HOURS`]) e o app conta os pedidos que faz entre uma
//! leitura e outra. Nada é pedido depois de [`DEFAULT_LIMIT_PCT`]% da cota (ajustável). No
//! plano gratuito só há cotação: a conferência dos indicadores e proventos (módulos pagos) fica
//! desligada, sem gastar cota, e a cotação serve de reserva do Yahoo para os papéis da B3
//! (no máximo uma por papel por dia). A resolução de tickers renomeados não usa token.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Ajuste com o token (no cofre cifrado, como a chave da FRED).
pub const TOKEN_SETTING: &str = "research.brapi_token";
/// Fonte no controle de coleta (`research_fetch_log`).
pub const LOG_SOURCE: &str = "brapi";
/// Horas até buscar de novo o mesmo papel.
pub const FRESH_HOURS: i64 = 20;
/// Estado do plano e do consumo (ajuste técnico, no cofre).
pub const STATE_SETTING: &str = "research.brapi_state";
/// Porcentagem da cota mensal que o app se permite usar.
pub const LIMIT_SETTING: &str = "research.brapi_limit_pct";
pub const DEFAULT_LIMIT_PCT: i64 = 80;
/// Cota do plano gratuito (pedidos por mês), enquanto o plano não foi lido da API.
pub const FREE_QUOTA: i64 = 15_000;
/// O consumo é lido de novo da API depois disto.
pub const USAGE_REFRESH_HOURS: i64 = 6;
/// Controle das cotações de reserva (`research_fetch_log`), uma por papel por dia.
pub const QUOTE_LOG_SOURCE: &str = "brapi_quote";
/// Controle da resolução de tickers renomeados (por papel), a cada 30 dias.
pub const RESOLVE_LOG_SOURCE: &str = "brapi_resolve";
pub const RESOLVE_REFRESH_DAYS: i64 = 30;
pub const BASE: &str = "https://brapi.dev";

/// Plano e consumo da conta, como a API informou, mais os pedidos feitos depois disso.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BrapiState {
    /// `free`, `startup` ou `pro`.
    pub plan: Option<String>,
    pub plan_limit: Option<i64>,
    /// Consumo informado pela API na última leitura.
    pub used: i64,
    /// Fim da janela da cota (renovação).
    pub window_end: Option<String>,
    pub checked_at: Option<String>,
    /// Pedidos feitos pelo app depois da última leitura.
    pub since_check: i64,
}

impl BrapiState {
    pub fn quota(&self) -> i64 {
        self.plan_limit.filter(|l| *l > 0).unwrap_or(FREE_QUOTA)
    }
    pub fn used_now(&self) -> i64 {
        self.used + self.since_check
    }
    /// Plano com proventos e indicadores (Startup ou Pro).
    pub fn paid(&self) -> bool {
        matches!(self.plan.as_deref(), Some("startup") | Some("pro"))
    }
    /// Precisa ler o consumo de novo (nunca lido, velho ou janela vencida).
    pub fn stale(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        let old = self
            .checked_at
            .as_deref()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .is_none_or(|t| {
                (now - t.with_timezone(&chrono::Utc)).num_hours() >= USAGE_REFRESH_HOURS
            });
        let renewed = self
            .window_end
            .as_deref()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .is_some_and(|t| now >= t.with_timezone(&chrono::Utc));
        old || renewed
    }
}

pub fn state(conn: &Connection) -> Result<BrapiState> {
    crate::settings::get_or(conn, STATE_SETTING, BrapiState::default())
}

pub fn limit_pct(conn: &Connection) -> Result<i64> {
    Ok(crate::settings::get_or(conn, LIMIT_SETTING, DEFAULT_LIMIT_PCT)?.clamp(10, 100))
}

/// Pedidos que ainda cabem no limite do app.
pub fn budget(conn: &Connection) -> Result<i64> {
    let s = state(conn)?;
    Ok((s.quota() * limit_pct(conn)? / 100 - s.used_now()).max(0))
}

/// Conta `n` pedidos feitos com o token.
pub fn count(conn: &Connection, n: i64) -> Result<()> {
    if n <= 0 {
        return Ok(());
    }
    let mut s = state(conn)?;
    s.since_check += n;
    crate::settings::set(conn, STATE_SETTING, &s)
}

pub fn usage_url() -> String {
    format!("{BASE}/api/v2/user/usage")
}

/// Plano e consumo informados pela API.
#[derive(Debug, Clone, PartialEq)]
pub struct Usage {
    pub plan: Option<String>,
    pub limit: Option<i64>,
    pub current: i64,
    pub window_end: Option<String>,
}

pub fn parse_usage(body: &[u8]) -> std::result::Result<Usage, String> {
    let v: serde_json::Value = serde_json::from_slice(body).map_err(|e| format!("brapi: {e}"))?;
    if let Some(msg) = v.get("message").and_then(|m| m.as_str())
        && v.get("usage").is_none()
    {
        return Err(format!("brapi: {msg}"));
    }
    let u = v
        .get("usage")
        .ok_or_else(|| crate::i18n::t("nucleo-pesquisa-erro-brapi-sem-resultados"))?;
    Ok(Usage {
        plan: u
            .get("planName")
            .and_then(|x| x.as_str())
            .map(|s| s.trim().to_lowercase()),
        limit: u.get("planLimit").and_then(|x| x.as_i64()),
        current: u.get("currentUsage").and_then(|x| x.as_i64()).unwrap_or(0),
        window_end: u
            .pointer("/usageWindow/end")
            .and_then(|x| x.as_str())
            .map(String::from),
    })
}

/// Lê o plano e o consumo da conta (rede; sem banco).
pub fn fetch_usage(f: &dyn super::net::Fetcher, token: &str) -> std::result::Result<Usage, String> {
    let auth = format!("Bearer {}", token.trim());
    match f.get_with(&usage_url(), &[("Authorization", auth.as_str())], "")? {
        super::net::Fetched::Body { bytes, .. } => parse_usage(&bytes),
        _ => Err(crate::i18n::t("nucleo-pesquisa-erro-brapi-sem-resultados")),
    }
}

/// Grava o que a API informou (zera a contagem local).
pub fn store_usage(conn: &Connection, u: &Usage, now: &str) -> Result<BrapiState> {
    let s = BrapiState {
        plan: u.plan.clone(),
        plan_limit: u.limit,
        used: u.current,
        window_end: u.window_end.clone(),
        checked_at: Some(now.to_string()),
        since_check: 0,
    };
    crate::settings::set(conn, STATE_SETTING, &s)?;
    Ok(s)
}

/// Cotação atual de um papel (endpoint v2, um papel por pedido no plano gratuito).
pub fn quote_v2_url(ticker: &str) -> String {
    format!(
        "{BASE}/api/v2/stocks/quote?symbols={}",
        ticker.trim().to_uppercase()
    )
}

/// Cotação: data do pregão (fuso da B3), preço e moeda.
#[derive(Debug, Clone, PartialEq)]
pub struct BrapiQuote {
    pub date: NaiveDate,
    pub price: f64,
    pub currency: Option<String>,
}

pub fn parse_quote_v2(body: &[u8]) -> std::result::Result<Option<BrapiQuote>, String> {
    let v: serde_json::Value = serde_json::from_slice(body).map_err(|e| format!("brapi: {e}"))?;
    if let Some(msg) = v.get("message").and_then(|m| m.as_str())
        && v.get("results").is_none()
    {
        return Err(format!("brapi: {msg}"));
    }
    let Some(d) = v.pointer("/results/0/data") else {
        return Ok(None);
    };
    let Some(price) = d
        .get("regularMarketPrice")
        .and_then(|x| x.as_f64())
        .filter(|x| x.is_finite() && *x > 0.0)
    else {
        return Ok(None);
    };
    let b3 = chrono::FixedOffset::west_opt(3 * 3600).expect("fuso válido");
    let date = d
        .get("regularMarketTime")
        .and_then(|x| x.as_str())
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .map(|t| t.with_timezone(&b3).date_naive());
    let Some(date) = date else {
        return Ok(None);
    };
    Ok(Some(BrapiQuote {
        date,
        price,
        currency: d.get("currency").and_then(|x| x.as_str()).map(String::from),
    }))
}

/// Resolução de tickers renomeados (até 20 por pedido; não exige token).
pub fn resolve_url(tickers: &[String]) -> String {
    format!(
        "{BASE}/api/v2/tickers/resolve?symbols={}",
        tickers
            .iter()
            .map(|t| t.trim().to_uppercase())
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// Ticker antigo → ticker atual e a data da mudança (só os renomeados).
#[derive(Debug, Clone, PartialEq)]
pub struct Renamed {
    pub from: String,
    pub to: String,
    pub effective: Option<String>,
}

pub fn parse_resolve(body: &[u8]) -> std::result::Result<Vec<Renamed>, String> {
    let v: serde_json::Value = serde_json::from_slice(body).map_err(|e| format!("brapi: {e}"))?;
    if let Some(msg) = v.get("message").and_then(|m| m.as_str())
        && v.get("results").is_none()
    {
        return Err(format!("brapi: {msg}"));
    }
    Ok(v.get("results")
        .and_then(|r| r.as_array())
        .into_iter()
        .flatten()
        .filter(|r| r.get("status").and_then(|s| s.as_str()) == Some("renamed"))
        .filter_map(|r| {
            let from = r.get("requestedSymbol")?.as_str()?.trim().to_uppercase();
            let to = r.get("symbol")?.as_str()?.trim().to_uppercase();
            (from != to && !to.is_empty()).then(|| Renamed {
                from,
                to,
                effective: r
                    .get("effectiveDate")
                    .and_then(|d| d.as_str())
                    .map(String::from),
            })
        })
        .collect())
}

pub fn quote_url(ticker: &str) -> String {
    format!(
        "https://brapi.dev/api/quote/{}?modules=financialData,defaultKeyStatistics&dividends=true",
        ticker.trim().to_uppercase()
    )
}

/// Provento informado pela brapi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrapiDividend {
    pub ex_date: NaiveDate,
    pub value: f64,
    pub label: String,
}

/// Indicadores na unidade do score (frações e múltiplos) e proventos de um papel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrapiSnapshot {
    pub ticker: String,
    pub price: Option<f64>,
    /// Chave da métrica do score → valor (roe, net_margin, pb…).
    pub metrics: BTreeMap<String, f64>,
    pub dividends: Vec<BrapiDividend>,
    pub fetched_at: String,
}

fn num(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for p in path {
        cur = cur.get(*p)?;
    }
    cur.as_f64().filter(|x| x.is_finite())
}

/// Lê a resposta de `/api/quote`. As razões já vêm em fração.
pub fn parse(ticker: &str, body: &[u8], now: &str) -> std::result::Result<BrapiSnapshot, String> {
    let v: serde_json::Value = serde_json::from_slice(body).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-brapi-invalida",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    if let Some(msg) = v.get("message").and_then(|m| m.as_str())
        && v.get("results").is_none()
    {
        return Err(format!("brapi: {msg}"));
    }
    let r = v
        .get("results")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| crate::i18n::t("nucleo-pesquisa-erro-brapi-sem-resultados"))?;
    let mut m = BTreeMap::new();
    let mut put = |k: &str, x: Option<f64>| {
        if let Some(x) = x {
            m.insert(k.to_string(), x);
        }
    };
    let fd = |k: &str| num(r, &["financialData", k]);
    let ks = |k: &str| num(r, &["defaultKeyStatistics", k]);
    put("roe", fd("returnOnEquity"));
    put("roa", fd("returnOnAssets"));
    put(
        "net_margin",
        fd("profitMargins").or_else(|| ks("profitMargins")),
    );
    put("ebit_margin", fd("operatingMargins"));
    put("current_ratio", fd("currentRatio"));
    put("pb", ks("priceToBook").filter(|x| *x > 0.0));
    put("pvp_fii", ks("priceToBook").filter(|x| *x > 0.0));
    put("ev_ebitda", ks("enterpriseToEbitda").filter(|x| *x > 0.0));
    put(
        "earnings_to_price",
        num(r, &["priceEarnings"])
            .filter(|x| *x != 0.0)
            .map(|x| 1.0 / x),
    );
    if let (Some(debt), Some(cash), Some(e)) = (fd("totalDebt"), fd("totalCash"), fd("ebitda"))
        && e > 0.0
    {
        put("net_debt_ebitda", Some((debt - cash) / e));
    }
    if let (Some(f), Some(rev)) = (fd("freeCashflow"), fd("totalRevenue"))
        && rev > 0.0
    {
        put("fcf_margin", Some(f / rev));
    }
    let mut dividends = vec![];
    for d in r
        .get("dividendsData")
        .and_then(|x| x.get("cashDividends"))
        .and_then(|x| x.as_array())
        .into_iter()
        .flatten()
    {
        let date = d
            .get("exDate")
            .or_else(|| d.get("lastDatePrior"))
            .and_then(|x| x.as_str())
            .and_then(|s| s.get(..10))
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let (Some(ex_date), Some(value)) = (date, d.get("rate").and_then(|x| x.as_f64())) else {
            continue;
        };
        dividends.push(BrapiDividend {
            ex_date,
            value,
            label: d
                .get("label")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    let price = num(r, &["regularMarketPrice"]);
    // DY de 12 meses pelos proventos informados (conferência do DY oficial).
    if let (Some(p), Some(last)) = (price, dividends.iter().map(|d| d.ex_date).max())
        && p > 0.0
    {
        let since = last - chrono::Duration::days(365);
        let sum: f64 = dividends
            .iter()
            .filter(|d| d.ex_date > since)
            .map(|d| d.value)
            .sum();
        put("dy", Some(sum / p));
        put("dy_12m", Some(sum / p));
    }
    Ok(BrapiSnapshot {
        ticker: ticker.trim().to_uppercase(),
        price,
        metrics: m,
        dividends,
        fetched_at: now.to_string(),
    })
}

/// Token configurado (se houver).
pub fn token(conn: &Connection) -> Result<Option<String>> {
    Ok(crate::settings::get::<String>(conn, TOKEN_SETTING)?.filter(|t| !t.trim().is_empty()))
}

pub fn store(conn: &Connection, s: &BrapiSnapshot) -> Result<()> {
    conn.execute(
        "INSERT INTO research_fetch_log (source, key, fetched_at, status, detail) VALUES (?1, ?2, ?3, 'ok', ?4)
         ON CONFLICT (source, key) DO UPDATE SET fetched_at = excluded.fetched_at, status = 'ok', detail = excluded.detail",
        params![LOG_SOURCE, s.ticker, s.fetched_at, serde_json::to_string(s)?],
    )?;
    Ok(())
}

pub fn store_failure(
    conn: &Connection,
    ticker: &str,
    now: &str,
    status: &str,
    detail: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO research_fetch_log (source, key, fetched_at, status, detail) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (source, key) DO UPDATE SET fetched_at = excluded.fetched_at, status = excluded.status,
             detail = CASE WHEN research_fetch_log.status = 'ok' THEN research_fetch_log.detail ELSE excluded.detail END",
        params![LOG_SOURCE, ticker.to_uppercase(), now, status, detail],
    )?;
    Ok(())
}

/// Último retrato bom de um papel (mesmo antigo: serve para conferência, com a data).
pub fn cached(conn: &Connection, ticker: &str) -> Result<Option<BrapiSnapshot>> {
    let d: Option<String> = conn
        .query_row(
            "SELECT detail FROM research_fetch_log WHERE source = ?1 AND key = upper(?2) AND detail IS NOT NULL AND detail LIKE '{%'",
            params![LOG_SOURCE, ticker.trim()],
            |r| r.get(0),
        )
        .optional()?;
    Ok(d.and_then(|d| serde_json::from_str(&d).ok()))
}

/// Papéis cadastrados (ações e FIIs da B3, com posição ou na watchlist) sem consulta recente.
pub fn due_tickers(conn: &Connection, now: chrono::DateTime<chrono::Utc>) -> Result<Vec<String>> {
    let cut = (now - chrono::Duration::hours(FRESH_HOURS)).to_rfc3339();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT upper(a.ticker) FROM asset a
          WHERE a.archived = 0 AND a.ticker IS NOT NULL AND a.class IN ('stock_br', 'fii')
            AND (EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id)
                 OR EXISTS (SELECT 1 FROM watchlist_item w WHERE w.asset_id = a.id))
            AND NOT EXISTS (SELECT 1 FROM research_fetch_log l
                             WHERE l.source = ?1 AND l.key = upper(a.ticker) AND l.fetched_at >= ?2)
          ORDER BY 1",
    )?;
    let rows = stmt
        .query_map(params![LOG_SOURCE, cut], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"results":[{"symbol":"PETR4","regularMarketPrice":50.0,"priceEarnings":5.0,
      "financialData":{"totalCash":100,"ebitda":200,"totalDebt":500,"currentRatio":0.85,"returnOnAssets":0.1,
        "returnOnEquity":0.277,"freeCashflow":105,"totalRevenue":548,"operatingMargins":0.34,"profitMargins":0.24},
      "defaultKeyStatistics":{"priceToBook":1.53,"enterpriseToEbitda":4.98},
      "dividendsData":{"cashDividends":[
        {"rate":2.0,"label":"DIVIDENDO","exDate":"2026-08-24T03:00:00.000Z"},
        {"rate":1.5,"label":"JCP","exDate":"2026-03-01T03:00:00.000Z"},
        {"rate":9.0,"label":"DIVIDENDO","exDate":"2025-01-02T03:00:00.000Z"}]}}]}"#;

    #[test]
    fn parses_quote_with_modules_and_dividends() {
        let s = parse("petr4", SAMPLE.as_bytes(), "2026-10-16T00:00:00Z").unwrap();
        assert_eq!(s.ticker, "PETR4");
        let m = |k: &str| s.metrics[k];
        assert!((m("roe") - 0.277).abs() < 1e-12);
        assert!((m("net_debt_ebitda") - 2.0).abs() < 1e-12);
        assert!((m("earnings_to_price") - 0.2).abs() < 1e-12);
        assert!((m("fcf_margin") - 105.0 / 548.0).abs() < 1e-12);
        // DY: dividends of the 12 months up to the last one (2.0 + 1.5) ÷ 50.
        assert!((m("dy") - 0.07).abs() < 1e-12);
        assert_eq!(s.dividends.len(), 3);
        assert!(parse("X", br#"{"message":"token invalido"}"#, "t").is_err());
    }

    #[test]
    fn usage_quote_and_renames() {
        let u = parse_usage(
            br#"{"usage":{"planName":"Free","planLimit":15000,"currentUsage":1248,"remainingUsage":13752,
                "usageWindow":{"type":"billing-cycle","start":"2026-10-01T00:00:00.000Z","end":"2026-11-01T00:00:00.000Z"}},
                "requestedAt":"x","took":1}"#,
        )
        .unwrap();
        assert_eq!(u.plan.as_deref(), Some("free"));
        assert_eq!((u.limit, u.current), (Some(15000), 1248));
        assert!(parse_usage(br#"{"error":true,"message":"Token invalido"}"#).is_err());
        // Quote: the date is the trading day in B3's time zone (04:42 UTC = 01:42 in Brasília).
        let q = parse_quote_v2(
            br#"{"results":[{"symbol":"PETR4","data":{"currency":"BRL","regularMarketPrice":57.75,
                "regularMarketTime":"2026-10-22T04:42:09.000Z"}}]}"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(q.date, NaiveDate::from_ymd_opt(2026, 10, 22).unwrap());
        assert_eq!(q.price, 57.75);
        let q = parse_quote_v2(
            br#"{"results":[{"symbol":"X","data":{"regularMarketPrice":10.0,
            "regularMarketTime":"2026-10-22T02:00:00.000Z"}}]}"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(q.date, NaiveDate::from_ymd_opt(2026, 10, 21).unwrap());
        assert!(parse_quote_v2(br#"{"results":[]}"#).unwrap().is_none());
        let r = parse_resolve(
            br#"{"results":[{"requestedSymbol":"ELET3","symbol":"AXIA3","changed":true,"status":"renamed","effectiveDate":"2025-11-10"},
              {"requestedSymbol":"TAEE11","symbol":"TAEE11","changed":false,"status":"active","effectiveDate":null}]}"#,
        )
        .unwrap();
        assert_eq!(
            r,
            vec![Renamed {
                from: "ELET3".into(),
                to: "AXIA3".into(),
                effective: Some("2025-11-10".into())
            }]
        );
        assert!(resolve_url(&["elet3".into(), "TAEE11".into()]).ends_with("symbols=ELET3,TAEE11"));
    }

    #[test]
    fn plan_and_budget() {
        let mut s = BrapiState::default();
        assert_eq!(s.quota(), FREE_QUOTA);
        assert!(!s.paid());
        assert!(s.stale(chrono::Utc::now()));
        s.plan = Some("startup".into());
        s.plan_limit = Some(150_000);
        s.used = 100;
        s.since_check = 5;
        s.checked_at = Some(chrono::Utc::now().to_rfc3339());
        assert!(s.paid());
        assert_eq!(s.used_now(), 105);
        assert!(!s.stale(chrono::Utc::now()));
        // Window of the renewed quota: reads again.
        s.window_end = Some((chrono::Utc::now() - chrono::Duration::minutes(1)).to_rfc3339());
        assert!(s.stale(chrono::Utc::now()));
    }
}
