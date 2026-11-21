//! Fila de aprovação: propostas da IA ligadas a conjuntos de alterações pendentes. Uma
//! proposta sai da fila sozinha (`withdrawn`) quando outra alteração aplicada mexe nos mesmos
//! dados antes da aprovação: aprovar depois sobrescreveria a mudança feita à mão.

use std::collections::BTreeSet;

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use super::*;
use crate::changes;
use crate::error::{Error, Result};
use crate::time::now_utc;

fn parse_status(s: &str) -> ProposalStatus {
    match s {
        "approved" => ProposalStatus::Approved,
        "rejected" => ProposalStatus::Rejected,
        "failed" => ProposalStatus::Failed,
        "withdrawn" => ProposalStatus::Withdrawn,
        _ => ProposalStatus::Pending,
    }
}

fn status_str(s: ProposalStatus) -> &'static str {
    match s {
        ProposalStatus::Pending => "pending",
        ProposalStatus::Approved => "approved",
        ProposalStatus::Rejected => "rejected",
        ProposalStatus::Failed => "failed",
        ProposalStatus::Withdrawn => "withdrawn",
    }
}

const SELECT: &str = "SELECT p.*, cs.summary AS cs_summary,
    (SELECT count(*) FROM change_item ci WHERE ci.change_set_id = p.change_set_id) AS item_count
    FROM ai_proposal p JOIN change_set cs ON cs.id = p.change_set_id";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Proposal> {
    let summary: String = r.get("cs_summary")?;
    Ok(Proposal {
        id: r.get("id")?,
        change_set_id: r.get("change_set_id")?,
        client: r.get("client")?,
        command: r.get("command")?,
        level: if r.get::<_, String>("level")? == "sensitive" {
            ProposalLevel::Sensitive
        } else {
            ProposalLevel::Reversible
        },
        status: parse_status(&r.get::<_, String>("status")?),
        reason: r.get("reason")?,
        created_at: r.get("created_at")?,
        reviewed_at: r.get("reviewed_at")?,
        error: r.get("error")?,
        note: r.get("note")?,
        item_count: r.get("item_count")?,
        summary: serde_json::from_str(&summary).unwrap_or_default(),
    })
}

pub fn list(conn: &Connection, a: &ProposalListArgs) -> Result<Vec<Proposal>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE (?1 IS NULL OR p.status = ?1) ORDER BY p.created_at DESC LIMIT ?2"
    ))?;
    Ok(stmt
        .query_map(
            params![
                a.status.map(status_str),
                a.limit.unwrap_or(100).clamp(1, 1000)
            ],
            row,
        )?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<ProposalDetail> {
    let p = conn
        .query_row(&format!("{SELECT} WHERE p.id = ?1"), [id], row)
        .optional()?
        .ok_or_else(|| {
            Error::not_found(crate::i18n::t_args(
                "nucleo-ia-proposta-id",
                &crate::i18n::args(&[("id", id.to_string().into())]),
            ))
        })?;
    let items = changes::get(conn, &p.change_set_id)?.items;
    Ok(ProposalDetail { proposal: p, items })
}

pub fn pending_count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT count(*) FROM ai_proposal WHERE status = 'pending'",
        [],
        |r| r.get(0),
    )?)
}

/// Aprova na ordem de criação. Cada proposta é aplicada independente: uma falha (dados
/// mudaram) não impede as demais.
pub fn approve(
    conn: &mut Connection,
    ids: &[String],
    note: Option<&str>,
) -> Result<Vec<ReviewResult>> {
    let mut ordered: Vec<(String, String, String)> = vec![];
    for id in ids {
        let p = get(conn, id)?;
        // Retirada (os dados mudaram antes): sai no resultado, sem impedir as outras.
        if p.proposal.status != ProposalStatus::Pending
            && p.proposal.status != ProposalStatus::Withdrawn
        {
            return Err(Error::conflict(crate::i18n::t_args(
                "nucleo-ia-proposta-nao-pendente",
                &crate::i18n::args(&[("id", id.to_string().into())]),
            )));
        }
        ordered.push((
            p.proposal.created_at.clone(),
            id.clone(),
            p.proposal.change_set_id.clone(),
        ));
    }
    ordered.sort();
    let mut out = vec![];
    for (_, id, cs) in ordered {
        let now = now_utc();
        // Uma proposta aprovada antes, neste lote, pode ter retirado esta (mesmos dados).
        let p = get(conn, &id)?.proposal;
        if p.status == ProposalStatus::Withdrawn {
            out.push(ReviewResult {
                id,
                status: ProposalStatus::Withdrawn,
                error: p.note,
            });
            continue;
        }
        match changes::approve(conn, &cs) {
            Ok(_) => {
                conn.execute(
                    "UPDATE ai_proposal SET status = 'approved', reviewed_at = ?2, note = ?3 WHERE id = ?1",
                    params![id, now, note],
                )?;
                out.push(ReviewResult {
                    id,
                    status: ProposalStatus::Approved,
                    error: None,
                });
            }
            Err(e) => {
                conn.execute(
                    "UPDATE ai_proposal SET status = 'failed', reviewed_at = ?2, error = ?3 WHERE id = ?1",
                    params![id, now, e.to_string()],
                )?;
                let _ = changes::reject(conn, &cs);
                out.push(ReviewResult {
                    id,
                    status: ProposalStatus::Failed,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    Ok(out)
}

pub fn reject(
    conn: &mut Connection,
    ids: &[String],
    note: Option<&str>,
) -> Result<Vec<ReviewResult>> {
    let mut out = vec![];
    for id in ids {
        let p = get(conn, id)?;
        if p.proposal.status == ProposalStatus::Withdrawn {
            out.push(ReviewResult {
                id: id.clone(),
                status: ProposalStatus::Withdrawn,
                error: p.proposal.note,
            });
            continue;
        }
        if p.proposal.status != ProposalStatus::Pending {
            return Err(Error::conflict(crate::i18n::t_args(
                "nucleo-ia-proposta-nao-pendente",
                &crate::i18n::args(&[("id", id.to_string().into())]),
            )));
        }
        changes::reject(conn, &p.proposal.change_set_id)?;
        conn.execute(
            "UPDATE ai_proposal SET status = 'rejected', reviewed_at = ?2, note = ?3 WHERE id = ?1",
            params![id, now_utc(), note],
        )?;
        out.push(ReviewResult {
            id: id.clone(),
            status: ProposalStatus::Rejected,
            error: None,
        });
    }
    Ok(out)
}

/// O agente retira a própria proposta pendente: a fila deixa de mostrá-la e o conjunto de
/// mudanças é descartado. Proposta de outro cliente ou já decidida não muda.
pub fn withdraw_own(
    conn: &Connection,
    id: &str,
    client: &str,
    reason: Option<&str>,
) -> Result<ReviewResult> {
    let p = get(conn, id)?;
    if p.proposal.client != client {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-agente-retirar-de-outro",
        )));
    }
    if p.proposal.status != ProposalStatus::Pending {
        return Err(Error::conflict(crate::i18n::t(
            "nucleo-agente-retirar-decidida",
        )));
    }
    changes::reject(conn, &p.proposal.change_set_id)?;
    let note = crate::i18n::t_args(
        "nucleo-agente-retirada-pelo-agente",
        &crate::i18n::args(&[(
            "motivo",
            reason.map(str::trim).unwrap_or_default().to_string().into(),
        )]),
    );
    conn.execute(
        "UPDATE ai_proposal SET status = 'withdrawn', reviewed_at = ?2, note = ?3
          WHERE id = ?1 AND status = 'pending'",
        params![id, now_utc(), note],
    )?;
    Ok(ReviewResult {
        id: id.to_string(),
        status: ProposalStatus::Withdrawn,
        error: None,
    })
}

/// O que um conjunto mexe numa linha: as colunas alteradas, ou a linha toda (`None`) quando
/// ela foi criada ou apagada.
type Touch = (String, String, Option<BTreeSet<String>>);

fn touches(conn: &Connection, change_set_id: &str) -> Result<Vec<Touch>> {
    let mut stmt = conn.prepare(
        "SELECT table_name, row_id, op, before, after FROM change_item WHERE change_set_id = ?1",
    )?;
    let rows = stmt
        .query_map([change_set_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let parse = |s: &Option<String>| -> serde_json::Map<String, Value> {
        s.as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    };
    Ok(rows
        .into_iter()
        .filter_map(|(table, row, op, before, after)| {
            if op != "update" {
                return Some((table, row, None));
            }
            // Na atualização, `before` e `after` trazem as colunas alteradas (e a chave, igual
            // nos dois): ficam as que mudaram de fato.
            let (b, a) = (parse(&before), parse(&after));
            let cols: BTreeSet<String> = b
                .keys()
                .chain(a.keys())
                .filter(|k| !changes::STAMP_COLUMNS.contains(&k.as_str()) && b.get(*k) != a.get(*k))
                .cloned()
                .collect();
            (!cols.is_empty()).then_some((table, row, Some(cols)))
        })
        .collect())
}

fn overlaps(a: &[Touch], b: &[Touch]) -> bool {
    a.iter().any(|(ta, ra, ca)| {
        b.iter().any(|(tb, rb, cb)| {
            ta == tb
                && ra == rb
                && match (ca, cb) {
                    (Some(x), Some(y)) => !x.is_disjoint(y),
                    _ => true,
                }
        })
    })
}

/// Retira as propostas pendentes que mexem nos mesmos dados (mesma linha e coluna, ou a
/// linha criada ou apagada) que o conjunto `applied` acabou de mudar. Devolve quantas.
pub fn withdraw_overlapping(conn: &Connection, applied: &str) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT p.id, p.change_set_id FROM ai_proposal p
           JOIN change_item pi ON pi.change_set_id = p.change_set_id
           JOIN change_item ai ON ai.change_set_id = ?1
                              AND ai.table_name = pi.table_name AND ai.row_id = pi.row_id
          WHERE p.status = 'pending' AND p.change_set_id <> ?1",
    )?;
    let candidates = stmt
        .query_map([applied], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if candidates.is_empty() {
        return Ok(0);
    }
    let mine = touches(conn, applied)?;
    let (kind, reason): (String, String) = conn.query_row(
        "SELECT author_kind, reason FROM change_set WHERE id = ?1",
        [applied],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let who = crate::i18n::t(match kind.as_str() {
        "user" => "nucleo-ia-proposta-retirada-usuario",
        "import" => "nucleo-ia-proposta-retirada-importacao",
        "ai" => "nucleo-ia-proposta-retirada-agente",
        _ => "nucleo-ia-proposta-retirada-outra",
    });
    let note = crate::i18n::t_args(
        "nucleo-ia-proposta-retirada",
        &crate::i18n::args(&[("motivo", who.into()), ("alteracao", reason.into())]),
    );
    let now = now_utc();
    let mut n = 0;
    for (id, cs) in candidates {
        if !overlaps(&mine, &touches(conn, &cs)?) {
            continue;
        }
        conn.execute(
            "UPDATE change_set SET status = 'rejected' WHERE id = ?1 AND status = 'pending'",
            [&cs],
        )?;
        n += conn.execute(
            "UPDATE ai_proposal SET status = 'withdrawn', reviewed_at = ?2, note = ?3
              WHERE id = ?1 AND status = 'pending'",
            params![id, now, note],
        )?;
    }
    Ok(n)
}
