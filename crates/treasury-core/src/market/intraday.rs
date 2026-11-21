//! Cotações intradiárias (Yahoo Finance, ~15 min de atraso): a última cotação de cada
//! ativo acompanhado (`market_quote`) e os pontos de 5 minutos dos últimos dias
//! (`intraday_price`, apagados depois de [`KEEP_DAYS`]). As duas tabelas são técnicas: ficam
//! fora do histórico de alterações e dos dados do usuário.
//!
//! A rotina do app ([`refresh_if_due`]) roda a cada [`POLL_MINUTES`] minutos, só com o cofre
//! aberto, só para ativos com operações ou na watchlist, e só enquanto o pregão da bolsa do
//! ativo está aberto (catálogo `exchange`, no fuso da bolsa) ou fechou há pouco. Erros
//! recuam exponencialmente, por símbolo e no geral (rede fora).
//!
//! Os fechamentos diários continuam sendo a fonte do histórico; a carteira só usa a cotação
//! intradiária para "hoje" ([`super::price_on`]), marcada como atrasada e com o horário.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::exchange::{self, Exchange, SessionState};
use super::sync::{Provider, default_source};
use super::yahoo::{self, YahooIntraday, YahooMarketState};
use crate::assets::{Asset, AssetClass};
use crate::error::{Error, Result};
use crate::time::now_utc;

/// Atraso típico das cotações do Yahoo.
pub const DELAY_MINUTES: i64 = 15;
/// Intervalo da rotina do app.
pub const POLL_MINUTES: i64 = 5;
/// Depois do fechamento, a coleta continua por este tempo (o último preço chega atrasado).
pub const GRACE_MINUTES: i64 = DELAY_MINUTES + 5;
/// Pontos intradiários mais velhos que isto são apagados.
pub const KEEP_DAYS: i64 = 7;
/// Recuo máximo depois de erros seguidos.
pub const MAX_BACKOFF_MINUTES: i64 = 60;

/// Última cotação intradiária de um ativo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MarketQuote {
    pub asset_id: String,
    pub symbol: String,
    /// Decimais em texto, na moeda do ativo.
    pub price: String,
    pub previous_close: Option<String>,
    /// Variação sobre o fechamento anterior (preço) e em razão (0,0123 = +1,23%).
    pub change: Option<String>,
    pub change_pct: Option<f64>,
    pub day_high: Option<String>,
    pub day_low: Option<String>,
    pub volume: Option<String>,
    pub currency: String,
    /// Instante do último negócio (UTC RFC 3339).
    pub market_time: String,
    /// Data do pregão no fuso da bolsa.
    pub market_date: String,
    pub market_state: SessionState,
    pub timezone: Option<String>,
    #[ts(type = "number")]
    pub delay_minutes: i64,
    pub source: String,
    pub fetched_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IntradayPointView {
    /// Início da barra (segundos Unix, UTC).
    #[ts(type = "number")]
    pub ts: i64,
    pub price: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IntradaySeries {
    pub asset_id: String,
    /// "1d" (último pregão) ou "5d".
    pub range: String,
    pub quote: Option<MarketQuote>,
    pub points: Vec<IntradayPointView>,
    /// Fuso da bolsa, para os rótulos do gráfico.
    pub timezone: Option<String>,
    /// Bolsa do catálogo e situação do pregão agora (pelo horário regular).
    pub exchange: Option<String>,
    pub session_now: Option<SessionState>,
}

/// Selo de cotação intradiária num preço da carteira ou da watchlist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IntradayMark {
    pub quoted_at: String,
    pub previous_close: Option<String>,
    pub change_pct: Option<f64>,
    pub market_state: SessionState,
    #[ts(type = "number")]
    pub delay_minutes: i64,
}

fn d(s: &str) -> Option<Decimal> {
    s.parse().ok()
}

fn row_to_quote(r: &rusqlite::Row<'_>) -> rusqlite::Result<MarketQuote> {
    let price: String = r.get("price")?;
    let prev: Option<String> = r.get("previous_close")?;
    let (change, change_pct) = match (d(&price), prev.as_deref().and_then(d)) {
        (Some(p), Some(c)) if !c.is_zero() => (
            Some((p - c).normalize().to_string()),
            ((p - c) / c).to_f64(),
        ),
        _ => (None, None),
    };
    Ok(MarketQuote {
        asset_id: r.get("asset_id")?,
        symbol: r.get("symbol")?,
        price,
        previous_close: prev,
        change,
        change_pct,
        day_high: r.get("day_high")?,
        day_low: r.get("day_low")?,
        volume: r.get("volume")?,
        currency: r.get("currency")?,
        market_time: r.get("market_time")?,
        market_date: r.get("market_date")?,
        market_state: SessionState::parse(&r.get::<_, String>("market_state")?),
        timezone: r.get("timezone")?,
        delay_minutes: r.get("delay_minutes")?,
        source: r.get("source")?,
        fetched_at: r.get("fetched_at")?,
    })
}

pub fn quote(conn: &Connection, asset_id: &str) -> Result<Option<MarketQuote>> {
    Ok(conn
        .query_row(
            "SELECT * FROM market_quote WHERE asset_id = ?1",
            [asset_id],
            row_to_quote,
        )
        .optional()?)
}

/// Cotações de vários ativos (todas, se `ids` vier vazio).
pub fn quotes(conn: &Connection, ids: &[String]) -> Result<Vec<MarketQuote>> {
    let mut stmt = conn.prepare("SELECT * FROM market_quote ORDER BY asset_id")?;
    let all = stmt
        .query_map([], row_to_quote)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(if ids.is_empty() {
        all
    } else {
        all.into_iter()
            .filter(|q| ids.contains(&q.asset_id))
            .collect()
    })
}

/// Bolsa usada para o horário de um ativo: a do catálogo; sem ela, B3 para as classes
/// brasileiras e NYSE para as americanas. Cripto não tem bolsa (negocia o tempo todo).
fn exchange_of(conn: &Connection, a: &Asset) -> Result<Option<Exchange>> {
    if let Some(e) = &a.exchange {
        return exchange::get(conn, &e.code);
    }
    match a.class {
        AssetClass::StockBr | AssetClass::Fii | AssetClass::EtfBr | AssetClass::Bdr => {
            exchange::get(conn, "B3")
        }
        AssetClass::StockUs | AssetClass::EtfUs | AssetClass::Reit => exchange::get(conn, "NYSE"),
        _ => Ok(None),
    }
}

pub fn series(
    conn: &Connection,
    asset_id: &str,
    range: &str,
    now: DateTime<Utc>,
) -> Result<IntradaySeries> {
    if !yahoo::RANGES.contains(&range) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-intradiario-periodo",
        )));
    }
    let asset = crate::assets::get(conn, asset_id)?;
    let ex = exchange_of(conn, &asset)?;
    let q = quote(conn, asset_id)?;
    let tz = q
        .as_ref()
        .and_then(|q| q.timezone.clone())
        .or(ex.as_ref().map(|e| e.timezone.clone()));
    let mut stmt =
        conn.prepare("SELECT ts, price FROM intraday_price WHERE asset_id = ?1 ORDER BY ts")?;
    let mut points = stmt
        .query_map([asset_id], |r| {
            Ok(IntradayPointView {
                ts: r.get(0)?,
                price: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let date_of = |ts: i64| -> NaiveDate {
        let at = DateTime::from_timestamp(ts, 0).unwrap_or(now);
        exchange::local_date(tz.as_deref().unwrap_or("UTC"), at)
    };
    let mut days: Vec<NaiveDate> = points.iter().map(|p| date_of(p.ts)).collect();
    days.dedup();
    let keep = if range == "1d" { 1 } else { 5 };
    if days.len() > keep {
        let first = days[days.len() - keep];
        points.retain(|p| date_of(p.ts) >= first);
    }
    Ok(IntradaySeries {
        asset_id: asset_id.into(),
        range: range.into(),
        quote: q,
        points,
        timezone: tz,
        session_now: ex
            .as_ref()
            .map(|e| exchange::session_at(e, now))
            .transpose()?,
        exchange: ex.map(|e| e.code),
    })
}

// ── Coleta ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QuoteTask {
    pub asset: Asset,
    pub symbol: String,
    pub exchange: Option<Exchange>,
    /// "1d" se já há pontos recentes; "5d" para preencher o gráfico de 5 dias.
    pub range: &'static str,
}

/// Ativos acompanhados (com operações ou na watchlist) com cotação do Yahoo cuja bolsa está
/// em pregão. `only` limita a alguns ativos e ignora o horário (atualização pedida na tela).
pub fn plan(
    conn: &Connection,
    now: DateTime<Utc>,
    only: Option<&[String]>,
) -> Result<Vec<QuoteTask>> {
    if !crate::sources::enabled(conn, crate::sources::SourceId::Yahoo) {
        return Ok(vec![]);
    }
    let ids: Vec<String> = match only {
        Some(ids) => ids.to_vec(),
        None => super::sync::tracked_assets(conn)?,
    };
    let mut out = vec![];
    for id in ids {
        let asset = crate::assets::get(conn, &id)?;
        let Some((Provider::Yahoo, symbol)) = default_source(&asset) else {
            continue;
        };
        let ex = exchange_of(conn, &asset)?;
        let last_fetch: Option<String> = conn
            .query_row(
                "SELECT fetched_at FROM market_quote WHERE asset_id = ?1",
                [&id],
                |r| r.get(0),
            )
            .optional()?;
        let age = last_fetch
            .as_deref()
            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            .map(|t| now - t.with_timezone(&Utc));
        if only.is_none() {
            let open = match (&ex, asset.class) {
                (Some(e), _) => exchange::should_poll(e, now, GRACE_MINUTES)?,
                (None, AssetClass::Crypto) => true,
                (None, _) => false,
            };
            if !open
                || age.is_some_and(|a| {
                    a < chrono::Duration::minutes(POLL_MINUTES) - chrono::Duration::seconds(30)
                })
            {
                continue;
            }
        } else if age.is_some_and(|a| a < chrono::Duration::seconds(60)) {
            continue;
        }
        let recent: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM intraday_price WHERE asset_id = ?1 AND ts >= ?2)",
            params![id, now.timestamp() - 3 * 86_400],
            |r| r.get(0),
        )?;
        out.push(QuoteTask {
            asset,
            symbol,
            exchange: ex,
            range: if recent { "1d" } else { "5d" },
        });
    }
    Ok(out)
}

pub fn fetch(tasks: &[QuoteTask]) -> Vec<(QuoteTask, std::result::Result<YahooIntraday, String>)> {
    tasks
        .iter()
        .map(|t| (t.clone(), yahoo::fetch_intraday(&t.symbol, "5m", t.range)))
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuoteRefreshReport {
    pub refreshed: Vec<String>,
    pub errors: Vec<QuoteError>,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuoteError {
    pub asset_id: String,
    pub ticker: Option<String>,
    pub symbol: String,
    pub error: String,
}

fn state_of(y: YahooMarketState, ex: Option<&Exchange>, now: DateTime<Utc>) -> SessionState {
    match y {
        YahooMarketState::Pre => SessionState::Pre,
        YahooMarketState::Post => SessionState::Post,
        YahooMarketState::Closed => SessionState::Closed,
        YahooMarketState::Regular => match ex.map(|e| exchange::session_at(e, now)) {
            Some(Ok(SessionState::Lunch)) => SessionState::Lunch,
            _ => SessionState::Open,
        },
    }
}

pub fn apply(
    conn: &Connection,
    fetched: &[(QuoteTask, std::result::Result<YahooIntraday, String>)],
    now: DateTime<Utc>,
) -> Result<QuoteRefreshReport> {
    let mut rep = QuoteRefreshReport::default();
    let fetched_at = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    for (t, res) in fetched {
        let err = |e: String| QuoteError {
            asset_id: t.asset.id.clone(),
            ticker: t.asset.ticker.clone(),
            symbol: t.symbol.clone(),
            error: e,
        };
        let data = match res {
            Ok(d) => d,
            Err(e) => {
                rep.errors.push(err(e.clone()));
                continue;
            }
        };
        let q = &data.quote;
        if let Some(c) = &q.currency
            && *c != t.asset.currency
        {
            rep.errors
                .push(err(super::sync::currency_mismatch(c, &t.asset.currency)));
            continue;
        }
        let at = DateTime::from_timestamp(q.market_time, 0).unwrap_or(now);
        let tz = q
            .timezone
            .clone()
            .or(t.exchange.as_ref().map(|e| e.timezone.clone()));
        let s = |v: &Option<Decimal>| v.map(|x| x.normalize().to_string());
        conn.execute(
            "INSERT INTO market_quote (asset_id, symbol, price, previous_close, day_high, day_low, volume, currency,
                                       market_time, market_date, market_state, timezone, delay_minutes, source, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'yahoo_intraday', ?14)
             ON CONFLICT (asset_id) DO UPDATE SET symbol = excluded.symbol, price = excluded.price,
                 previous_close = excluded.previous_close, day_high = excluded.day_high, day_low = excluded.day_low,
                 volume = excluded.volume, currency = excluded.currency, market_time = excluded.market_time,
                 market_date = excluded.market_date, market_state = excluded.market_state, timezone = excluded.timezone,
                 delay_minutes = excluded.delay_minutes, source = excluded.source, fetched_at = excluded.fetched_at",
            params![
                t.asset.id,
                t.symbol,
                q.price.normalize().to_string(),
                s(&q.previous_close),
                s(&q.day_high),
                s(&q.day_low),
                s(&q.volume),
                t.asset.currency,
                at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                exchange::local_date(tz.as_deref().unwrap_or("UTC"), at).to_string(),
                state_of(q.state, t.exchange.as_ref(), now).as_str(),
                tz,
                if t.asset.class == AssetClass::Crypto { 0 } else { DELAY_MINUTES },
                fetched_at
            ],
        )?;
        let mut stmt = conn.prepare_cached(
            "INSERT INTO intraday_price (asset_id, ts, price, volume) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (asset_id, ts) DO UPDATE SET price = excluded.price, volume = excluded.volume",
        )?;
        for p in &data.points {
            stmt.execute(params![
                t.asset.id,
                p.ts,
                p.price.normalize().to_string(),
                s(&p.volume)
            ])?;
        }
        rep.refreshed.push(t.asset.id.clone());
    }
    conn.execute(
        "DELETE FROM intraday_price WHERE ts < ?1",
        [now.timestamp() - KEEP_DAYS * 86_400],
    )?;
    rep.finished_at = now_utc();
    Ok(rep)
}

// ── Recuo em erros ───────────────────────────────────────────────

/// Recuo exponencial por símbolo (ativo inexistente, resposta inválida) e geral (rede fora:
/// todos os símbolos falharam): 5, 10, 20, 40 e no máximo 60 minutos.
#[derive(Debug, Default)]
pub struct Backoff {
    per_symbol: HashMap<String, (u32, DateTime<Utc>)>,
    global: Option<(u32, DateTime<Utc>)>,
}

fn delay(failures: u32) -> chrono::Duration {
    let m = POLL_MINUTES.saturating_mul(1 << failures.saturating_sub(1).min(5));
    chrono::Duration::minutes(m.min(MAX_BACKOFF_MINUTES))
}

impl Backoff {
    pub fn global_allows(&self, now: DateTime<Utc>) -> bool {
        self.global.is_none_or(|(_, until)| now >= until)
    }
    pub fn allows(&self, symbol: &str, now: DateTime<Utc>) -> bool {
        self.per_symbol
            .get(symbol)
            .is_none_or(|(_, until)| now >= *until)
    }
    pub fn record(&mut self, report: &QuoteRefreshReport, tasks: &[QuoteTask], now: DateTime<Utc>) {
        let failed: Vec<&str> = report.errors.iter().map(|e| e.symbol.as_str()).collect();
        for t in tasks {
            if failed.contains(&t.symbol.as_str()) {
                let n = self.per_symbol.get(&t.symbol).map_or(0, |(n, _)| *n) + 1;
                self.per_symbol
                    .insert(t.symbol.clone(), (n, now + delay(n)));
            } else {
                self.per_symbol.remove(&t.symbol);
            }
        }
        if !tasks.is_empty() && report.refreshed.is_empty() && !report.errors.is_empty() {
            let n = self.global.map_or(0, |(n, _)| n) + 1;
            self.global = Some((n, now + delay(n)));
        } else if !report.refreshed.is_empty() {
            self.global = None;
        }
    }
}

struct LoopState {
    last_run: Option<Instant>,
    backoff: Backoff,
}

static LOOP: LazyLock<Mutex<LoopState>> = LazyLock::new(|| {
    Mutex::new(LoopState {
        last_run: None,
        backoff: Backoff::default(),
    })
});

/// Atualização pedida pela interface (página do ativo, botão "Atualizar"): ignora o
/// horário do pregão, mas não repete um ativo coletado há menos de um minuto.
pub fn refresh(api: &crate::Api, asset_ids: &[String]) -> Result<QuoteRefreshReport> {
    let now = Utc::now();
    let tasks = api.with_session(|s, _| {
        let ids = if asset_ids.is_empty() {
            super::sync::tracked_assets(s.db.conn())?
        } else {
            asset_ids.to_vec()
        };
        plan(s.db.conn(), now, Some(&ids))
    })?;
    let fetched = fetch(&tasks);
    api.with_session(|s, _| apply(s.db.conn(), &fetched, Utc::now()))
}

/// Rotina do app: a cada [`POLL_MINUTES`] min, com o cofre aberto, para os ativos cuja bolsa
/// está em pregão; recua em erros. `None` quando nada precisou rodar.
pub fn refresh_if_due(api: &crate::Api) -> Result<Option<QuoteRefreshReport>> {
    if !api.is_unlocked() {
        return Ok(None);
    }
    let now = Utc::now();
    {
        let mut st = LOOP.lock().unwrap_or_else(|p| p.into_inner());
        if st
            .last_run
            .is_some_and(|t| t.elapsed() < Duration::from_secs((POLL_MINUTES * 60 - 30) as u64))
            || !st.backoff.global_allows(now)
        {
            return Ok(None);
        }
        st.last_run = Some(Instant::now());
    }
    let tasks = api.with_session(|s, _| plan(s.db.conn(), now, None))?;
    let tasks: Vec<QuoteTask> = {
        let st = LOOP.lock().unwrap_or_else(|p| p.into_inner());
        tasks
            .into_iter()
            .filter(|t| st.backoff.allows(&t.symbol, now))
            .collect()
    };
    if tasks.is_empty() {
        return Ok(None);
    }
    let fetched = fetch(&tasks);
    let report = api.with_session(|s, _| apply(s.db.conn(), &fetched, Utc::now()))?;
    LOOP.lock()
        .unwrap_or_else(|p| p.into_inner())
        .backoff
        .record(&report, &tasks, now);
    if !report.refreshed.is_empty() {
        crate::events::emit_system("market.refresh_quotes");
    }
    Ok(Some(report))
}

#[cfg(test)]
mod tests;
