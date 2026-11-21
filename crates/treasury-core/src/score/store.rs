//! Gravação do score: universo de pares do dia (`score_universe`), snapshots imutáveis
//! (`score_snapshot`), eventos qualitativos (`score_event`) e discordâncias
//! (`score_feedback`).

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::Peer;
use super::{AssetScore, ScoreStatus};
use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc};

/// Universo gravado de uma classe, se for do dia, da versão atual e posterior às coletas.
pub fn cached_universe(
    conn: &Connection,
    class: &str,
    today: NaiveDate,
    version: &str,
) -> Result<Option<Vec<Peer>>> {
    let meta: Option<(String, String, String)> = conn
        .query_row(
            "SELECT as_of, min(computed_at), min(methodology_version) FROM score_universe WHERE class = ?1",
            [class],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .and_then(|(a, c, v): (Option<String>, Option<String>, Option<String>)| Some((a?, c?, v?)));
    let Some((as_of, computed, ver)) = meta else {
        return Ok(None);
    };
    let last_sync = [
        crate::research::config::LAST_SYNC_SETTING,
        crate::indices::sync::LAST_SYNC_SETTING,
    ]
    .iter()
    .filter_map(|k| crate::settings::get::<String>(conn, k).ok().flatten())
    .max();
    if as_of != fmt_date(today) || ver != version || last_sync.is_some_and(|l| computed < l) {
        return Ok(None);
    }
    let mut stmt = conn.prepare(
        "SELECT id, ticker, name, sector, segment, metrics FROM score_universe WHERE class = ?1 ORDER BY id",
    )?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        Option<String>,
        String,
        Option<String>,
        String,
        String,
    )> = stmt
        .query_map([class], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (id, ticker, name, sector, segment, metrics) in rows {
        out.push(Peer {
            id,
            ticker,
            name,
            sector,
            segment,
            metrics: serde_json::from_str(&metrics)?,
        });
    }
    Ok(Some(out))
}

/// Troca o universo de uma classe (ignora falha de escrita: banco somente leitura).
pub fn store_universe(
    conn: &Connection,
    class: &str,
    peers: &[Peer],
    today: NaiveDate,
    version: &str,
) {
    let Ok(tx) = conn.unchecked_transaction() else {
        return;
    };
    let res: Result<()> = (|| {
        tx.execute("DELETE FROM score_universe WHERE class = ?1", [class])?;
        let now = now_utc();
        let mut stmt = tx.prepare(
            "INSERT INTO score_universe (class, id, ticker, name, sector, segment, metrics, as_of, computed_at, methodology_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )?;
        for p in peers {
            stmt.execute(params![
                class,
                p.id,
                p.ticker,
                p.name,
                p.sector,
                p.segment,
                serde_json::to_string(&p.metrics)?,
                fmt_date(today),
                now,
                version
            ])?;
        }
        Ok(())
    })();
    if res.is_ok() {
        let _ = tx.commit();
    }
}

pub fn subject_key(asset_id: Option<&str>, ticker: Option<&str>) -> String {
    match (asset_id, ticker) {
        (Some(a), _) => a.to_string(),
        (None, Some(t)) => format!("ticker:{}", t.to_uppercase()),
        _ => "desconhecido".into(),
    }
}

fn status_str(s: ScoreStatus) -> &'static str {
    match s {
        ScoreStatus::Ok => "ok",
        ScoreStatus::Partial => "partial",
        ScoreStatus::InsufficientData => "insufficient_data",
        ScoreStatus::NotSupported => "not_supported",
    }
}

/// Snapshot anterior (de outro dia) do mesmo ativo e versão.
pub fn previous(
    conn: &Connection,
    subject: &str,
    before: &str,
    version: &str,
) -> Result<Option<AssetScore>> {
    let p: Option<String> = conn
        .query_row(
            "SELECT payload FROM score_snapshot WHERE subject = ?1 AND as_of < ?2 AND methodology_version = ?3
             ORDER BY as_of DESC LIMIT 1",
            params![subject, before, version],
            |r| r.get(0),
        )
        .optional()?;
    Ok(p.and_then(|p| serde_json::from_str(&p).ok()))
}

/// Snapshot de um dia (o cálculo já feito hoje, para listas longas).
pub fn snapshot_on(
    conn: &Connection,
    subject: &str,
    as_of: &str,
    version: &str,
) -> Result<Option<AssetScore>> {
    let p: Option<String> = conn
        .query_row(
            "SELECT payload FROM score_snapshot WHERE subject = ?1 AND as_of = ?2 AND methodology_version = ?3",
            params![subject, as_of, version],
            |r| r.get(0),
        )
        .optional()?;
    Ok(p.and_then(|p| serde_json::from_str(&p).ok()))
}

/// Grava o snapshot do dia (os de dias anteriores ficam como estão). Ignora falha de
/// escrita (banco somente leitura).
pub fn save_snapshot(conn: &Connection, s: &AssetScore) {
    let subject = subject_key(s.asset_id.as_deref(), s.ticker.as_deref());
    let Ok(payload) = serde_json::to_string(s) else {
        return;
    };
    let _ = conn.execute(
        "INSERT INTO score_snapshot (subject, asset_id, ticker, as_of, methodology_version, status, score, band, confidence, payload, computed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT (subject, as_of, methodology_version) DO UPDATE SET
             status = excluded.status, score = excluded.score, band = excluded.band,
             confidence = excluded.confidence, payload = excluded.payload, computed_at = excluded.computed_at",
        params![
            subject,
            s.asset_id,
            s.ticker,
            s.as_of,
            s.methodology_version,
            status_str(s.status),
            s.score,
            s.band,
            s.confidence,
            payload,
            now_utc()
        ],
    );
}

/// Ponto da série histórica do score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScorePoint")]
pub struct ScorePoint {
    pub as_of: String,
    pub methodology_version: String,
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    pub band: String,
    #[ts(type = "number")]
    pub confidence: f64,
}

pub fn history(conn: &Connection, subject: &str) -> Result<Vec<ScorePoint>> {
    let mut stmt = conn.prepare(
        "SELECT as_of, methodology_version, score, band, confidence FROM score_snapshot
          WHERE subject = ?1 ORDER BY as_of, methodology_version",
    )?;
    let rows = stmt
        .query_map([subject], |r| {
            Ok(ScorePoint {
                as_of: r.get(0)?,
                methodology_version: r.get(1)?,
                score: r.get(2)?,
                band: r.get(3)?,
                confidence: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Discordâncias (§12.6)

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export, rename = "AssetScoreFeedbackInput")]
pub struct FeedbackInput {
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[ts(optional)]
    pub ticker: Option<String>,
    /// `discordo` ou `concordo`.
    pub verdict: String,
    /// Faixa que o usuário esperaria (opcional).
    #[ts(optional)]
    pub expected_band: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreFeedback")]
pub struct Feedback {
    pub id: Id,
    pub subject: String,
    pub asset_id: Option<String>,
    pub ticker: Option<String>,
    pub as_of: String,
    pub methodology_version: String,
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    pub band: String,
    pub verdict: String,
    pub expected_band: Option<String>,
    pub reason: String,
    pub created_at: String,
}

pub fn add_feedback(tx: &Transaction<'_>, i: &FeedbackInput, s: &AssetScore) -> Result<Id> {
    if !matches!(i.verdict.as_str(), "discordo" | "concordo") {
        return Err(Error::invalid(crate::i18n::t("nucleo-score-erro-veredito")));
    }
    let reason = i.reason.trim();
    if reason.is_empty() || reason.chars().count() > 2000 {
        return Err(Error::invalid(crate::i18n::t("nucleo-score-erro-motivo")));
    }
    let id = new_id();
    tx.execute(
        "INSERT INTO score_feedback (id, subject, asset_id, ticker, as_of, methodology_version, score, band, verdict, expected_band, reason, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            id,
            subject_key(s.asset_id.as_deref(), s.ticker.as_deref()),
            s.asset_id,
            s.ticker,
            s.as_of,
            s.methodology_version,
            s.score,
            s.band,
            i.verdict,
            i.expected_band,
            reason,
            now_utc()
        ],
    )?;
    Ok(id)
}

pub fn list_feedback(conn: &Connection, subject: Option<&str>) -> Result<Vec<Feedback>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM score_feedback WHERE (?1 IS NULL OR subject = ?1) ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![subject], |r| {
            Ok(Feedback {
                id: r.get("id")?,
                subject: r.get("subject")?,
                asset_id: r.get("asset_id")?,
                ticker: r.get("ticker")?,
                as_of: r.get("as_of")?,
                methodology_version: r.get("methodology_version")?,
                score: r.get("score")?,
                band: r.get("band")?,
                verdict: r.get("verdict")?,
                expected_band: r.get("expected_band")?,
                reason: r.get("reason")?,
                created_at: r.get("created_at")?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}
