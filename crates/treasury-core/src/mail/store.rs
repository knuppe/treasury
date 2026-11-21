//! Fila do e-mail de ingestão: as mensagens que passaram nos filtros, as peças de cada uma
//! (anexos e o corpo) com o que a leitura achou, e a auditoria (buscas, recusadas, exclusões).
//! Tabelas fora do histórico de alterações: o que conta para desfazer é o que a importação
//! gravou nos dados da família.

use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};

/// "peça {id}" (detalhe do erro "não encontrado").
fn part_ref(id: &str) -> String {
    t_args(
        "nucleo-email-ref-peca",
        &args(&[("id", id.to_string().into())]),
    )
}
use crate::time::now_utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MailStatus {
    AwaitingReview,
    Imported,
    Duplicate,
    Failed,
    Unrecognized,
    Ignored,
    /// Peça esperando a leitura pela IA.
    AiPending,
}

impl MailStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwaitingReview => "awaiting_review",
            Self::Imported => "imported",
            Self::Duplicate => "duplicate",
            Self::Failed => "failed",
            Self::Unrecognized => "unrecognized",
            Self::Ignored => "ignored",
            Self::AiPending => "ai_pending",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "imported" => Self::Imported,
            "duplicate" => Self::Duplicate,
            "failed" => Self::Failed,
            "unrecognized" => Self::Unrecognized,
            "ignored" => Self::Ignored,
            "ai_pending" => Self::AiPending,
            _ => Self::AwaitingReview,
        }
    }
    /// Estado final (a mensagem sai da fila).
    pub fn is_final(self) -> bool {
        !matches!(self, Self::AwaitingReview | Self::AiPending)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailPart {
    pub id: Id,
    #[ts(type = "number")]
    pub idx: i64,
    pub filename: String,
    pub mime: Option<String>,
    #[ts(type = "number")]
    pub size: i64,
    /// `statement`, `card_statement`, `brokerage_note`, `b3_report`, `pension_statement`,
    /// `payslip`, `receipt`, `image`, `purchase`, `pix`, `template`, `unknown`.
    pub kind: String,
    pub label: String,
    pub layout_id: Option<String>,
    pub status: MailStatus,
    /// O que a leitura achou (o lançamento do aviso, o link do cupom, o resumo da prévia).
    #[ts(type = "Record<string, unknown> | null")]
    pub detail: Option<Value>,
    pub result_ref: Option<String>,
    pub error: Option<String>,
    /// O conteúdo ainda está no cofre (para abrir no importador).
    pub has_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailMessage {
    pub id: Id,
    pub account_id: Id,
    pub account_name: String,
    pub folder: String,
    pub file_name: Option<String>,
    pub sender: Option<String>,
    pub original_sender: Option<String>,
    pub subject: Option<String>,
    pub date: Option<String>,
    #[ts(type = "number")]
    pub size: i64,
    pub authenticated: bool,
    /// Pode ser aplicada sozinha (remetente autenticado, ou encaminhada por membro com o
    /// remetente original nos permitidos da regra).
    pub trusted: bool,
    pub auth_detail: Option<String>,
    pub rule_id: Option<Id>,
    pub rule_name: Option<String>,
    pub status: MailStatus,
    pub error: Option<String>,
    pub received_at: String,
    pub decided_at: Option<String>,
    pub deleted_at: Option<String>,
    pub has_raw: bool,
    pub parts: Vec<MailPart>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailListArgs {
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Padrão: todas.
    #[serde(default)]
    #[ts(optional)]
    pub status: Option<Vec<MailStatus>>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailAuditEntry {
    pub id: Id,
    pub account_id: Option<Id>,
    pub at: String,
    /// `sync`, `rejected`, `imported`, `duplicate`, `deleted`, `would_delete`, `skipped_delete`,
    /// `ai`, `reviewed`, `error`.
    pub action: String,
    pub message_id: Option<Id>,
    pub detail: String,
}

const MSG_COLS: &str = "m.id, m.account_id, a.name, m.folder, m.file_name, m.sender, m.original_sender, m.subject,
    m.date, m.size, m.authenticated, m.trusted, m.auth_detail, m.rule_id, m.rule_name, m.status, m.error,
    m.received_at, m.decided_at, m.deleted_at, m.raw IS NOT NULL";

fn msg_row(r: &Row<'_>) -> rusqlite::Result<MailMessage> {
    let status: String = r.get(15)?;
    Ok(MailMessage {
        id: r.get(0)?,
        account_id: r.get(1)?,
        account_name: r.get(2)?,
        folder: r.get(3)?,
        file_name: r.get(4)?,
        sender: r.get(5)?,
        original_sender: r.get(6)?,
        subject: r.get(7)?,
        date: r.get(8)?,
        size: r.get(9)?,
        authenticated: r.get::<_, i64>(10)? != 0,
        trusted: r.get::<_, i64>(11)? != 0,
        auth_detail: r.get(12)?,
        rule_id: r.get(13)?,
        rule_name: r.get(14)?,
        status: MailStatus::parse(&status),
        error: r.get(16)?,
        received_at: r.get(17)?,
        decided_at: r.get(18)?,
        deleted_at: r.get(19)?,
        has_raw: r.get(20)?,
        parts: vec![],
    })
}

const PART_COLS: &str = "id, idx, filename, mime, size, kind, label, layout_id, status, detail, result_ref, error, content IS NOT NULL";

fn part_row(r: &Row<'_>) -> rusqlite::Result<MailPart> {
    let status: String = r.get(8)?;
    let detail: Option<String> = r.get(9)?;
    Ok(MailPart {
        id: r.get(0)?,
        idx: r.get(1)?,
        filename: r.get(2)?,
        mime: r.get(3)?,
        size: r.get(4)?,
        kind: r.get(5)?,
        label: r.get(6)?,
        layout_id: r.get(7)?,
        status: MailStatus::parse(&status),
        detail: detail.and_then(|d| serde_json::from_str(&d).ok()),
        result_ref: r.get(10)?,
        error: r.get(11)?,
        has_content: r.get(12)?,
    })
}

pub fn parts_of(conn: &Connection, message_id: &str) -> Result<Vec<MailPart>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PART_COLS} FROM mail_part WHERE message_id = ?1 ORDER BY idx"
    ))?;
    Ok(stmt
        .query_map([message_id], part_row)?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn list(conn: &Connection, a: &MailListArgs) -> Result<Vec<MailMessage>> {
    let statuses: Vec<&str> = a
        .status
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect();
    let statuses_json = serde_json::to_string(&statuses)?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {MSG_COLS} FROM mail_message m JOIN mail_account a ON a.id = m.account_id
          WHERE (?1 IS NULL OR m.account_id = ?1)
            AND (?2 = '[]' OR m.status IN (SELECT value FROM json_each(?2)))
          ORDER BY m.received_at DESC LIMIT ?3"
    ))?;
    let mut out: Vec<MailMessage> = stmt
        .query_map(
            params![
                a.account_id,
                statuses_json,
                a.limit.unwrap_or(200).clamp(1, 1000)
            ],
            msg_row,
        )?
        .collect::<rusqlite::Result<_>>()?;
    for m in &mut out {
        m.parts = parts_of(conn, &m.id)?;
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> Result<MailMessage> {
    let mut m = conn
        .query_row(
            &format!(
                "SELECT {MSG_COLS} FROM mail_message m JOIN mail_account a ON a.id = m.account_id WHERE m.id = ?1"
            ),
            [id],
            msg_row,
        )
        .optional()?
        .ok_or_else(|| {
            Error::not_found(t_args(
                "nucleo-email-ref-mensagem",
                &args(&[("id", id.to_string().into())]),
            ))
        })?;
    m.parts = parts_of(conn, id)?;
    Ok(m)
}

/// Mensagem dona da peça.
pub fn message_of_part(conn: &Connection, part_id: &str) -> Result<Id> {
    conn.query_row(
        "SELECT message_id FROM mail_part WHERE id = ?1",
        [part_id],
        |r| r.get(0),
    )
    .optional()?
    .ok_or_else(|| Error::not_found(part_ref(part_id)))
}

/// Conteúdo de uma peça (para abrir no importador ou mandar à IA).
pub struct PartContent {
    pub filename: String,
    pub mime: Option<String>,
    pub kind: String,
    pub bytes: Vec<u8>,
}

type ContentRow = (String, Option<String>, String, Option<Vec<u8>>);

pub fn part_content(conn: &Connection, part_id: &str) -> Result<PartContent> {
    let row: Option<ContentRow> = conn
        .query_row(
            "SELECT filename, mime, kind, content FROM mail_part WHERE id = ?1",
            [part_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let (filename, mime, kind, content) = row.ok_or_else(|| Error::not_found(part_ref(part_id)))?;
    let bytes = content.ok_or_else(|| Error::conflict(t("nucleo-email-erro-peca-sem-conteudo")))?;
    Ok(PartContent {
        filename,
        mime,
        kind,
        bytes,
    })
}

/// Estado da mensagem pelo das peças.
pub fn rollup(parts: &[MailStatus]) -> MailStatus {
    use MailStatus::*;
    if parts.is_empty() {
        return Unrecognized;
    }
    if parts
        .iter()
        .any(|s| matches!(s, AwaitingReview | AiPending))
    {
        return AwaitingReview;
    }
    if parts.contains(&Imported) {
        return Imported;
    }
    if parts.contains(&Failed) {
        return Failed;
    }
    if parts.contains(&Unrecognized) {
        return Unrecognized;
    }
    if parts.contains(&Duplicate) {
        return Duplicate;
    }
    Ignored
}

/// Recalcula o estado da mensagem; ao chegar num estado final, marca a decisão (e, sem
/// "guardar original", tira do cofre o conteúdo das peças já tratadas).
pub fn refresh_message(conn: &Connection, message_id: &str) -> Result<MailStatus> {
    let statuses: Vec<MailStatus> = parts_of(conn, message_id)?
        .iter()
        .map(|p| p.status)
        .collect();
    let s = rollup(&statuses);
    let now = now_utc();
    conn.execute(
        "UPDATE mail_message SET status = ?2, updated_at = ?3,
            decided_at = CASE WHEN ?4 THEN COALESCE(decided_at, ?3) ELSE NULL END
          WHERE id = ?1",
        params![message_id, s.as_str(), now, s.is_final()],
    )?;
    if s.is_final() {
        conn.execute(
            "UPDATE mail_part SET content = NULL
              WHERE message_id = ?1 AND status IN ('imported', 'duplicate', 'ignored')
                AND NOT (SELECT keep_original FROM mail_account a JOIN mail_message m ON m.account_id = a.id
                          WHERE m.id = ?1)",
            [message_id],
        )?;
    }
    Ok(s)
}

/// Muda o estado de uma peça (revisão do usuário ou resultado da importação).
pub fn set_part(
    conn: &Connection,
    part_id: &str,
    status: MailStatus,
    result_ref: Option<&str>,
    error: Option<&str>,
) -> Result<MailStatus> {
    let msg = message_of_part(conn, part_id)?;
    conn.execute(
        "UPDATE mail_part SET status = ?2, result_ref = COALESCE(?3, result_ref), error = ?4, updated_at = ?5
          WHERE id = ?1",
        params![part_id, status.as_str(), result_ref, error, now_utc()],
    )?;
    refresh_message(conn, &msg)
}

pub fn set_part_detail(conn: &Connection, part_id: &str, detail: &Value) -> Result<()> {
    conn.execute(
        "UPDATE mail_part SET detail = ?2, updated_at = ?3 WHERE id = ?1",
        params![part_id, serde_json::to_string(detail)?, now_utc()],
    )?;
    Ok(())
}

/// Ignora as peças ainda abertas da mensagem.
pub fn ignore_message(conn: &Connection, id: &str) -> Result<MailStatus> {
    get(conn, id)?;
    conn.execute(
        "UPDATE mail_part SET status = 'ignored', updated_at = ?2
          WHERE message_id = ?1 AND status IN ('awaiting_review', 'ai_pending', 'unrecognized', 'failed')",
        params![id, now_utc()],
    )?;
    refresh_message(conn, id)
}

pub fn audit(
    conn: &Connection,
    account_id: Option<&str>,
    action: &str,
    message_id: Option<&str>,
    detail: &str,
) -> Result<()> {
    let detail: String = detail.chars().take(500).collect();
    conn.execute(
        "INSERT INTO mail_audit (id, account_id, at, action, message_id, detail) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![new_id(), account_id, now_utc(), action, message_id, detail],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailAuditArgs {
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

pub fn audit_list(conn: &Connection, a: &MailAuditArgs) -> Result<Vec<MailAuditEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, at, action, message_id, detail FROM mail_audit
          WHERE ?1 IS NULL OR account_id = ?1 ORDER BY at DESC LIMIT ?2",
    )?;
    Ok(stmt
        .query_map(
            params![a.account_id, a.limit.unwrap_or(200).clamp(1, 2000)],
            |r| {
                Ok(MailAuditEntry {
                    id: r.get(0)?,
                    account_id: r.get(1)?,
                    at: r.get(2)?,
                    action: r.get(3)?,
                    message_id: r.get(4)?,
                    detail: r.get(5)?,
                })
            },
        )?
        .collect::<rusqlite::Result<_>>()?)
}

/// Quantas vezes a ação aconteceu hoje (UTC) na conta: os limites diários.
pub fn count_today(conn: &Connection, account_id: &str, action: &str) -> Result<i64> {
    let today = &now_utc()[..10];
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM mail_audit WHERE account_id = ?1 AND action = ?2 AND substr(at, 1, 10) = ?3",
        params![account_id, action, today],
        |r| r.get(0),
    )?)
}

/// Mensagens recebidas hoje (UTC) na conta.
pub fn messages_today(conn: &Connection, account_id: &str) -> Result<i64> {
    let today = &now_utc()[..10];
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM mail_message WHERE account_id = ?1 AND substr(received_at, 1, 10) = ?2",
        params![account_id, today],
        |r| r.get(0),
    )?)
}
