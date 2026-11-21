//! Eventos qualitativos (§8.2): recuperação judicial, fraude, litígio, desastre ambiental,
//! governança, regulação. Cada evento tem data, evidência (link), severidade e a decisão do
//! usuário: só os aceitos e não resolvidos entram no score, com penalidade que decai pela
//! meia-vida e teto que relaxa na mesma proporção. Os fatos relevantes e comunicados da CVM
//! (IPE) viram sugestões por palavras-chave; nada é aplicado sem o aceite do usuário.

use chrono::NaiveDate;
use rusqlite::{Connection, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::config::{Methodology, Severity};
use super::{AssetScore, ScoreFlag, ScoreStatus};
use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::time::{now_utc, parse_date};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "AssetScoreEventStatus")]
pub enum EventStatus {
    /// Registrado, aguardando a decisão do usuário (não afeta o score).
    Pending,
    Accepted,
    Rejected,
    /// O problema foi resolvido (deixa de afetar o score).
    Resolved,
}

impl EventStatus {
    fn as_str(self) -> &'static str {
        match self {
            EventStatus::Pending => "pending",
            EventStatus::Accepted => "accepted",
            EventStatus::Rejected => "rejected",
            EventStatus::Resolved => "resolved",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "accepted" => EventStatus::Accepted,
            "rejected" => EventStatus::Rejected,
            "resolved" => EventStatus::Resolved,
            _ => EventStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreEvent")]
pub struct ScoreEvent {
    pub id: Id,
    pub asset_id: Option<String>,
    pub ticker: Option<String>,
    pub kind: String,
    pub kind_label: String,
    pub title: String,
    pub event_date: String,
    pub severity: Severity,
    #[ts(type = "number | null")]
    pub cap: Option<f64>,
    #[ts(type = "number | null")]
    pub penalty: Option<f64>,
    pub evidence_url: Option<String>,
    pub source: String,
    pub status: EventStatus,
    pub resolved_at: Option<String>,
    pub note: Option<String>,
    /// Efeito hoje (teto e penalidade depois do decaimento), se aceito e não resolvido.
    #[ts(type = "number | null")]
    pub effective_cap: Option<f64>,
    #[ts(type = "number | null")]
    pub effective_penalty: Option<f64>,
}

/// Fato relevante da CVM que parece um evento (só sugestão).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreEventSuggestion")]
pub struct EventSuggestion {
    pub kind: String,
    pub kind_label: String,
    pub title: String,
    pub event_date: String,
    pub evidence_url: String,
    pub source: String,
    /// Palavra que casou no assunto.
    pub matched: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export, rename = "AssetScoreEventInput")]
pub struct EventInput {
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[ts(optional)]
    pub ticker: Option<String>,
    pub kind: String,
    pub title: String,
    /// `AAAA-MM-DD`.
    pub event_date: String,
    /// Severidade (padrão: a do tipo).
    #[ts(optional)]
    pub severity: Option<Severity>,
    /// Teto e penalidade (padrão: os do tipo na metodologia).
    #[ts(optional, type = "number")]
    pub cap: Option<f64>,
    #[ts(optional, type = "number")]
    pub penalty: Option<f64>,
    #[ts(optional)]
    pub evidence_url: Option<String>,
    /// `user` (padrão) ou `cvm_ipe` (aceite de uma sugestão).
    #[ts(optional)]
    pub source: Option<String>,
    /// Já entra aceito (padrão: sim, quando o próprio usuário registra).
    #[ts(optional)]
    pub accepted: Option<bool>,
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export, rename = "AssetScoreEventStatusArgs")]
pub struct EventStatusArgs {
    pub id: Id,
    pub status: EventStatus,
    #[ts(optional)]
    pub note: Option<String>,
}

fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Info | Severity::Low => "low",
        Severity::Medium => "medium",
        Severity::High => "high",
        Severity::Critical => "critical",
    }
}

fn severity_parse(s: &str) -> Severity {
    match s {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        _ => Severity::Low,
    }
}

/// Fator de decaimento: 0,5 elevado aos trimestres passados ÷ meia-vida.
pub fn decay(m: &Methodology, event_date: NaiveDate, today: NaiveDate) -> f64 {
    let quarters = ((today - event_date).num_days().max(0) as f64) / 91.3;
    0.5f64.powf(quarters / m.events.half_life_quarters.max(0.1))
}

fn effect(m: &Methodology, e: &ScoreEvent, today: NaiveDate) -> (Option<f64>, Option<f64>) {
    if e.status != EventStatus::Accepted {
        return (None, None);
    }
    let Ok(d) = parse_date(&e.event_date) else {
        return (None, None);
    };
    let k = decay(m, d, today);
    (
        e.cap
            .map(|c| ((c + (100.0 - c) * (1.0 - k)) * 10.0).round() / 10.0),
        e.penalty.map(|p| (p * k * 10.0).round() / 10.0),
    )
}

/// Eventos de um ativo (cadastrado ou ticker), mais recentes primeiro.
pub fn list(
    conn: &Connection,
    m: &Methodology,
    asset_id: Option<&str>,
    ticker: Option<&str>,
    today: NaiveDate,
) -> Result<Vec<ScoreEvent>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM score_event WHERE (?1 IS NOT NULL AND asset_id = ?1) OR (?2 IS NOT NULL AND upper(ticker) = upper(?2))
          ORDER BY event_date DESC, created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![asset_id, ticker], |r| {
            let kind: String = r.get("kind")?;
            Ok(ScoreEvent {
                id: r.get("id")?,
                asset_id: r.get("asset_id")?,
                ticker: r.get("ticker")?,
                kind_label: m.event_label(&kind),
                kind,
                title: r.get("title")?,
                event_date: r.get("event_date")?,
                severity: severity_parse(&r.get::<_, String>("severity")?),
                cap: r.get("cap")?,
                penalty: r.get("penalty")?,
                evidence_url: r.get("evidence_url")?,
                source: r.get("source")?,
                status: EventStatus::parse(&r.get::<_, String>("status")?),
                resolved_at: r.get("resolved_at")?,
                note: r.get("note")?,
                effective_cap: None,
                effective_penalty: None,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .map(|mut e| {
            let (c, p) = effect(m, &e, today);
            e.effective_cap = c;
            e.effective_penalty = p;
            e
        })
        .collect())
}

fn clean_url(u: Option<&str>) -> Result<Option<String>> {
    let Some(u) = u.map(str::trim).filter(|u| !u.is_empty()) else {
        return Ok(None);
    };
    if !(u.starts_with("https://") || u.starts_with("http://")) || u.len() > 2000 {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-score-erro-evento-link",
        )));
    }
    Ok(Some(u.to_string()))
}

pub fn add(tx: &Transaction<'_>, m: &Methodology, i: &EventInput) -> Result<Id> {
    let kind = m.events.kinds.get(&i.kind).ok_or_else(|| {
        Error::invalid(crate::i18n::t_args(
            "nucleo-score-erro-evento-tipo",
            &crate::i18n::args(&[("tipo", i.kind.clone().into())]),
        ))
    })?;
    let title = i.title.trim();
    if title.is_empty() || title.chars().count() > 300 {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-score-erro-evento-titulo",
        )));
    }
    parse_date(&i.event_date)?;
    if i.asset_id.is_none() && i.ticker.as_deref().is_none_or(|t| t.trim().is_empty()) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-score-erro-evento-ativo",
        )));
    }
    for v in [i.cap, i.penalty].into_iter().flatten() {
        if !(0.0..=100.0).contains(&v) {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-score-erro-evento-limites",
            )));
        }
    }
    let id = new_id();
    let now = now_utc();
    let status = if i.accepted.unwrap_or(true) {
        EventStatus::Accepted
    } else {
        EventStatus::Pending
    };
    let (cap, penalty) = if i.cap.is_some() || i.penalty.is_some() {
        (i.cap, i.penalty)
    } else {
        (kind.cap, kind.penalty)
    };
    tx.execute(
        "INSERT INTO score_event (id, asset_id, ticker, kind, title, event_date, severity, cap, penalty, half_life_quarters,
                                  evidence_url, source, status, resolved_at, note, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, NULL, ?14, ?15, ?15)",
        params![
            id,
            i.asset_id,
            i.ticker.as_deref().map(|t| t.trim().to_uppercase()),
            i.kind,
            title,
            i.event_date,
            severity_str(i.severity.unwrap_or(kind.severity)),
            cap,
            penalty,
            m.events.half_life_quarters,
            clean_url(i.evidence_url.as_deref())?,
            i.source.clone().unwrap_or_else(|| "user".into()),
            status.as_str(),
            i.note.as_deref().map(str::trim).filter(|n| !n.is_empty()),
            now
        ],
    )?;
    Ok(id)
}

pub fn set_status(tx: &Transaction<'_>, a: &EventStatusArgs) -> Result<()> {
    let now = now_utc();
    let n = tx.execute(
        "UPDATE score_event SET status = ?2, updated_at = ?3,
                resolved_at = CASE WHEN ?2 = 'resolved' THEN ?3 ELSE NULL END,
                note = coalesce(?4, note)
          WHERE id = ?1",
        params![a.id, a.status.as_str(), now, a.note],
    )?;
    if n == 0 {
        return Err(Error::not_found(crate::i18n::t("nucleo-score-ref-evento")));
    }
    Ok(())
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM score_event WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(crate::i18n::t("nucleo-score-ref-evento")));
    }
    Ok(())
}

/// Fatos relevantes e comunicados do emissor (últimos 24 meses) cujo assunto casa com as
/// palavras-chave de um tipo, sem os já registrados como evento.
pub fn suggestions(
    conn: &Connection,
    m: &Methodology,
    issuer_id: &str,
    events: &[ScoreEvent],
    today: NaiveDate,
) -> Result<Vec<EventSuggestion>> {
    let since = crate::time::add_months(today, -24);
    let mut stmt = conn.prepare(
        "SELECT category, subject, delivered_at, link FROM corporate_document
          WHERE issuer_id = ?1 AND delivered_at >= ?2 ORDER BY delivered_at DESC LIMIT 400",
    )?;
    let rows: Vec<(String, Option<String>, String, String)> = stmt
        .query_map(params![issuer_id, crate::time::fmt_date(since)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (category, subject, delivered, link) in rows {
        if events
            .iter()
            .any(|e| e.evidence_url.as_deref() == Some(link.as_str()))
        {
            continue;
        }
        let text = crate::research::norm(&format!(
            "{category} {}",
            subject.clone().unwrap_or_default()
        ));
        'kinds: for (key, k) in &m.events.kinds {
            for w in &k.keywords {
                if !w.is_empty() && text.contains(w.as_str()) {
                    out.push(EventSuggestion {
                        kind: key.clone(),
                        kind_label: m.event_label(key),
                        title: subject
                            .clone()
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or(category.clone()),
                        event_date: delivered.get(..10).unwrap_or(&delivered).to_string(),
                        evidence_url: link.clone(),
                        source: format!("CVM IPE ({category})"),
                        matched: w.clone(),
                    });
                    break 'kinds;
                }
            }
        }
    }
    Ok(out)
}

/// Eventos aceitos viram red flags (com o efeito de hoje) e o score é refeito sobre o score
/// bruto: teto = o menor de todos; penalidades somadas.
pub fn apply(m: &Methodology, s: &mut AssetScore, events: &[ScoreEvent]) {
    for e in events.iter().filter(|e| e.status == EventStatus::Accepted) {
        if e.effective_cap.is_none() && e.effective_penalty.is_none() {
            continue;
        }
        s.flags.push(ScoreFlag {
            key: format!("event:{}", e.id),
            label: crate::i18n::t_args(
                "nucleo-score-flag-evento",
                &crate::i18n::args(&[("tipo", e.kind_label.clone().into())]),
            ),
            severity: e.severity,
            cap: e.effective_cap,
            penalty: e.effective_penalty.filter(|p| *p > 0.0),
            detail: format!(
                "{} ({}){}",
                e.title,
                e.event_date,
                e.evidence_url
                    .as_deref()
                    .map(|u| format!(" — {u}"))
                    .unwrap_or_default()
            ),
        });
    }
    if !s.flags.iter().any(|f| f.key.starts_with("event:")) {
        return;
    }
    let cap = s
        .flags
        .iter()
        .filter_map(|f| f.cap)
        .fold(f64::INFINITY, f64::min);
    let penalty: f64 = s.flags.iter().filter_map(|f| f.penalty).sum();
    let r1 = |x: f64| (x * 10.0).round() / 10.0;
    let apply = |x: f64| r1((x - penalty).min(cap).clamp(0.0, 100.0));
    if let Some(raw) = s.raw_score
        && s.status != ScoreStatus::InsufficientData
    {
        s.score = Some(apply(raw));
        if let Some(h) = s.health_raw {
            s.health = Some(apply(h));
        }
        if s.status == ScoreStatus::Ok {
            let b = m.band(s.score.unwrap_or(0.0));
            s.band = b.key.clone();
            s.band_label = b.label_text();
            s.band_meaning = b.meaning_text();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::config::methodology;

    #[test]
    fn decay_halves_every_half_life() {
        let m = methodology();
        let d = |s: &str| parse_date(s).unwrap();
        assert!((decay(m, d("2026-01-01"), d("2026-01-01")) - 1.0).abs() < 1e-9);
        let k = decay(m, d("2025-01-01"), d("2026-01-01"));
        assert!((k - 0.5).abs() < 0.01, "{k}");
        let e = ScoreEvent {
            id: "x".into(),
            asset_id: None,
            ticker: Some("X".into()),
            kind: "fraude".into(),
            kind_label: "Fraude".into(),
            title: "t".into(),
            event_date: "2025-01-01".into(),
            severity: Severity::Critical,
            cap: Some(25.0),
            penalty: Some(10.0),
            evidence_url: None,
            source: "user".into(),
            status: EventStatus::Accepted,
            resolved_at: None,
            note: None,
            effective_cap: None,
            effective_penalty: None,
        };
        let (c, p) = effect(m, &e, d("2026-01-01"));
        // A year later (4 quarters = one half-life): cap 25 → ~62.5; penalty 10 → ~5.
        assert!((c.unwrap() - 62.5).abs() < 1.0, "{c:?}");
        assert!((p.unwrap() - 5.0).abs() < 0.2, "{p:?}");
        let pending = ScoreEvent {
            status: EventStatus::Pending,
            ..e
        };
        assert_eq!(effect(m, &pending, d("2026-01-01")), (None, None));
    }
}
