//! Score em lote do universo (docs/design/market-explorer.md, seção 4.2): depois do
//! universo do dia, a coleta calcula o score de cada ação e FII do universo e grava um resumo
//! por papel em `market_score`, para o screener filtrar por score sem calcular sob demanda.
//!
//! O cálculo é o mesmo do score de um ativo ([`super::assess`]), sem snapshot, alerta nem
//! feedback: o histórico e os alertas continuam só para a carteira e a watchlist. A coleta
//! grava em lotes pequenos, soltando o banco entre eles, como os outros passos.

use std::collections::HashMap;
use std::time::Instant;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AssetScore, ScoreStatus, methodology};
use crate::error::Result;
use crate::research::data::SubjectRef;
use crate::time::{fmt_date, now_utc};

/// Papéis por lote (o banco fica livre entre um lote e outro).
const CHUNK: usize = 20;

/// Resumo do score de um papel do universo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MarketScore {
    pub ticker: String,
    pub class: String,
    pub status: ScoreStatus,
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    pub band: Option<String>,
    #[ts(type = "number | null")]
    pub confidence: Option<f64>,
    #[ts(type = "number | null")]
    pub health: Option<f64>,
    #[ts(type = "number | null")]
    pub attractiveness: Option<f64>,
    pub red_flags: Vec<String>,
    pub segment: Option<String>,
    #[ts(type = "number | null")]
    pub piotroski: Option<f64>,
    #[ts(type = "number | null")]
    pub altman_z: Option<f64>,
    #[ts(type = "number | null")]
    pub beneish: Option<f64>,
    #[ts(type = "number | null")]
    pub momentum_12_1: Option<f64>,
    #[ts(type = "number | null")]
    pub volatility_1y: Option<f64>,
    pub as_of: String,
    pub methodology_version: String,
}

/// O que a coleta fez.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MarketScoreReport")]
pub struct BatchReport {
    pub scored: u32,
    /// Papéis sem score (dados insuficientes ou erro de leitura).
    pub without: u32,
    #[ts(type = "number")]
    pub elapsed_ms: u64,
}

impl MarketScore {
    pub fn of(s: &AssetScore, class: &str, ticker: &str) -> Self {
        let framework = |key: &str| {
            s.frameworks
                .iter()
                .find(|f| f.key == key)
                .and_then(|f| f.value)
        };
        let metric = |key: &str| {
            s.dimensions
                .iter()
                .flat_map(|d| d.metrics.iter())
                .find(|m| m.key == key)
                .and_then(|m| m.value)
        };
        let scored = matches!(s.status, ScoreStatus::Ok | ScoreStatus::Partial);
        MarketScore {
            ticker: ticker.to_string(),
            class: class.to_string(),
            status: s.status,
            score: s.score,
            band: scored.then(|| s.band.clone()),
            confidence: scored.then_some(s.confidence),
            health: s.health,
            attractiveness: s.attractiveness,
            red_flags: s.flags.iter().map(|f| f.key.clone()).collect(),
            segment: Some(s.segment.clone()).filter(|x| !x.is_empty()),
            piotroski: framework("piotroski"),
            altman_z: framework("altman_z2"),
            beneish: framework("beneish"),
            momentum_12_1: metric("momentum_12_1"),
            volatility_1y: metric("volatility_1y"),
            as_of: s.as_of.clone(),
            methodology_version: s.methodology_version.clone(),
        }
    }
}

/// Ações e FIIs do universo do dia (o score em lote cobre a B3; EUA ficam para depois).
pub fn universe_tickers(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT ticker, class FROM research_universe WHERE class IN ('stock_br', 'fii') ORDER BY ticker",
    )?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Score de um papel do universo, sem gravar snapshot, alerta nem feedback.
pub fn score_one(
    conn: &Connection,
    ticker: &str,
    class: &str,
    today: NaiveDate,
) -> Option<MarketScore> {
    super::assess::evaluate(conn, &SubjectRef::ticker(ticker), today)
        .ok()
        .map(|s| MarketScore::of(&s, class, ticker))
}

pub fn save(conn: &Connection, rows: &[MarketScore]) -> Result<()> {
    let now = now_utc();
    let mut stmt = conn.prepare_cached(
        "INSERT INTO market_score (ticker, class, status, score, band, confidence, health, attractiveness, red_flags,
                                   segment, piotroski, altman_z, beneish, momentum_12_1, volatility_1y, as_of,
                                   methodology_version, computed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
         ON CONFLICT (ticker) DO UPDATE SET class = excluded.class, status = excluded.status, score = excluded.score,
             band = excluded.band, confidence = excluded.confidence, health = excluded.health,
             attractiveness = excluded.attractiveness, red_flags = excluded.red_flags, segment = excluded.segment,
             piotroski = excluded.piotroski, altman_z = excluded.altman_z, beneish = excluded.beneish,
             momentum_12_1 = excluded.momentum_12_1, volatility_1y = excluded.volatility_1y, as_of = excluded.as_of,
             methodology_version = excluded.methodology_version, computed_at = excluded.computed_at",
    )?;
    for r in rows {
        stmt.execute(params![
            r.ticker,
            r.class,
            serde_json::to_value(r.status)?
                .as_str()
                .unwrap_or("not_supported"),
            r.score,
            r.band,
            r.confidence,
            r.health,
            r.attractiveness,
            serde_json::to_string(&r.red_flags)?,
            r.segment,
            r.piotroski,
            r.altman_z,
            r.beneish,
            r.momentum_12_1,
            r.volatility_1y,
            r.as_of,
            r.methodology_version,
            now
        ])?;
    }
    Ok(())
}

/// Resumos gravados, por ticker (só os da metodologia atual).
pub fn load(conn: &Connection) -> Result<HashMap<String, MarketScore>> {
    let version = methodology().version.clone();
    let mut stmt = conn.prepare(
        "SELECT ticker, class, status, score, band, confidence, health, attractiveness, red_flags, segment,
                piotroski, altman_z, beneish, momentum_12_1, volatility_1y, as_of, methodology_version
           FROM market_score WHERE methodology_version = ?1",
    )?;
    let rows = stmt
        .query_map([&version], |r| {
            let status: String = r.get(2)?;
            let flags: String = r.get(8)?;
            Ok(MarketScore {
                ticker: r.get(0)?,
                class: r.get(1)?,
                status: serde_json::from_value(serde_json::Value::String(status))
                    .unwrap_or(ScoreStatus::NotSupported),
                score: r.get(3)?,
                band: r.get(4)?,
                confidence: r.get(5)?,
                health: r.get(6)?,
                attractiveness: r.get(7)?,
                red_flags: serde_json::from_str(&flags).unwrap_or_default(),
                segment: r.get(9)?,
                piotroski: r.get(10)?,
                altman_z: r.get(11)?,
                beneish: r.get(12)?,
                momentum_12_1: r.get(13)?,
                volatility_1y: r.get(14)?,
                as_of: r.get(15)?,
                methodology_version: r.get(16)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows.into_iter().map(|m| (m.ticker.clone(), m)).collect())
}

/// Data do último lote (para saber se o de hoje já foi feito).
pub fn last_as_of(conn: &Connection) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT max(as_of) FROM market_score", [], |r| r.get(0))
        .optional()?
        .flatten())
}

/// Calcula e grava o score de todo o universo, em lotes de [`CHUNK`] papéis: `with` abre o
/// banco para cada lote (a coleta passa o seu, que solta o lock entre os lotes).
pub fn run(
    mut with: impl FnMut(&mut dyn FnMut(&mut Connection) -> Result<()>) -> Result<()>,
    today: NaiveDate,
) -> Result<BatchReport> {
    let started = Instant::now();
    let mut tickers = vec![];
    with(&mut |c| {
        tickers = universe_tickers(c)?;
        Ok(())
    })?;
    let mut rep = BatchReport::default();
    // Tarefa em segundo plano (docs/background-tasks.md): progresso por papel.
    crate::tasks::stage(crate::tasks::TaskStage::Scores);
    crate::tasks::set_total(Some(tickers.len() as u64));
    for chunk in tickers.chunks(CHUNK) {
        // Cancelada: os papéis já calculados ficam; o resto, na próxima coleta.
        if crate::tasks::cancelled() {
            rep.elapsed_ms = started.elapsed().as_millis() as u64;
            return Ok(rep);
        }
        crate::tasks::advance_by(chunk.len() as u64);
        with(&mut |c| {
            let rows: Vec<MarketScore> = chunk
                .iter()
                .filter_map(|(t, class)| score_one(c, t, class, today))
                .collect();
            rep.scored += rows.iter().filter(|r| r.score.is_some()).count() as u32;
            rep.without += (chunk.len() - rows.iter().filter(|r| r.score.is_some()).count()) as u32;
            save(c, &rows)
        })?;
    }
    // Papéis que saíram do universo saem também do lote.
    with(&mut |c| {
        c.execute(
            "DELETE FROM market_score WHERE ticker NOT IN (SELECT ticker FROM research_universe)
                OR as_of < ?1",
            [fmt_date(today - chrono::Duration::days(30))],
        )?;
        Ok(())
    })?;
    rep.elapsed_ms = started.elapsed().as_millis() as u64;
    Ok(rep)
}
