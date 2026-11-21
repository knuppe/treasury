//! Retenção do histórico do agente: quanto as conversas e os anexos ocupam no cofre, a
//! faxina (conversas não fixadas sem uso há N dias e anexos guardados vencidos) e o
//! `VACUUM` depois de uma faxina grande, para o arquivo do cofre encolher de fato.
//!
//! A faxina roda junto das rotinas do app (`ai::run_due`), no máximo uma vez por dia com o
//! cofre aberto, e pelo botão "Limpar agora" (`ai.history.cleanup`).
//!
//! Sobre o `VACUUM`: o SQLite não devolve ao sistema o espaço das linhas apagadas, só o
//! marca como livre (`freelist`) para reaproveitar. O cofre usa `auto_vacuum = NONE`
//! (padrão), então `PRAGMA incremental_vacuum` não faz nada sem antes trocar o modo, o que
//! exige um `VACUUM` completo de qualquer jeito. Por isso, depois da faxina, se o espaço
//! livre passar de [`VACUUM_MIN_FREE`] ou de 25 % do arquivo, roda um `VACUUM` completo:
//! reescreve o banco cifrado (SQLCipher recifra cada página com a mesma chave) com
//! `temp_store = MEMORY`, para que a cópia temporária nunca vá para um arquivo.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::settings;
use crate::error::Result;
use crate::time::now_utc;

/// Espaço livre a partir do qual a faxina compacta o arquivo.
pub const VACUUM_MIN_FREE: i64 = 8 * 1024 * 1024;
/// Faxina automática no máximo uma vez a cada 24 h.
pub const LAST_CLEANUP: &str = "ai.history.last_cleanup";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HistoryStats {
    #[ts(type = "number")]
    pub conversations: i64,
    #[ts(type = "number")]
    pub pinned: i64,
    #[ts(type = "number")]
    pub messages: i64,
    /// Anexos com metadados e quantos têm os bytes guardados.
    #[ts(type = "number")]
    pub attachments: i64,
    #[ts(type = "number")]
    pub stored_attachments: i64,
    /// Bytes dos anexos guardados (dados + texto extraído).
    #[ts(type = "number")]
    pub attachment_bytes: i64,
    /// Tamanho aproximado do histórico (texto das mensagens + anexos guardados).
    #[ts(type = "number")]
    pub history_bytes: i64,
    /// Arquivo do cofre e o espaço livre dentro dele (recuperável com `VACUUM`).
    #[ts(type = "number")]
    pub db_bytes: i64,
    #[ts(type = "number")]
    pub free_bytes: i64,
    pub last_cleanup: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CleanupReport {
    #[ts(type = "number")]
    pub conversations_deleted: i64,
    #[ts(type = "number")]
    pub messages_deleted: i64,
    /// Anexos cujos bytes venceram (retenção "7 dias") e foram apagados.
    #[ts(type = "number")]
    pub attachments_expired: i64,
    pub vacuumed: bool,
    #[ts(type = "number")]
    pub db_bytes_before: i64,
    #[ts(type = "number")]
    pub db_bytes_after: i64,
}

/// Valor inteiro de um PRAGMA (o SQLCipher responde `page_size` em texto).
fn pragma_i64(conn: &Connection, name: &str) -> Result<i64> {
    let v: rusqlite::types::Value = conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))?;
    Ok(match v {
        rusqlite::types::Value::Integer(i) => i,
        rusqlite::types::Value::Text(t) => t.trim().parse().unwrap_or(0),
        _ => 0,
    })
}

/// Tamanho do arquivo e espaço livre, pelas páginas do banco.
pub fn db_size(conn: &Connection) -> Result<(i64, i64)> {
    let page = pragma_i64(conn, "page_size")?;
    Ok((
        pragma_i64(conn, "page_count")? * page,
        pragma_i64(conn, "freelist_count")? * page,
    ))
}

pub fn stats(conn: &Connection) -> Result<HistoryStats> {
    let (conversations, pinned): (i64, i64) = conn.query_row(
        "SELECT count(*), COALESCE(sum(pinned), 0) FROM ai_conversation",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let (messages, text_bytes): (i64, i64) = conn.query_row(
        "SELECT count(*), COALESCE(sum(length(CAST(content AS BLOB)) + COALESCE(length(CAST(reasoning AS BLOB)), 0)
            + length(CAST(tool_calls AS BLOB))), 0) FROM ai_message",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let (attachments, stored, att_bytes): (i64, i64, i64) = conn.query_row(
        "SELECT count(*),
                COALESCE(sum(data IS NOT NULL OR text IS NOT NULL), 0),
                COALESCE(sum(COALESCE(length(data), 0) + COALESCE(length(CAST(text AS BLOB)), 0)), 0)
         FROM ai_attachment",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let (db_bytes, free_bytes) = db_size(conn)?;
    Ok(HistoryStats {
        conversations,
        pinned,
        messages,
        attachments,
        stored_attachments: stored,
        attachment_bytes: att_bytes,
        history_bytes: text_bytes + att_bytes,
        db_bytes,
        free_bytes,
        last_cleanup: crate::settings::get(conn, LAST_CLEANUP)?,
    })
}

/// Vale compactar: espaço livre grande em absoluto ou em proporção do arquivo.
pub fn should_vacuum(db_bytes: i64, free_bytes: i64) -> bool {
    free_bytes >= VACUUM_MIN_FREE || (free_bytes >= 1024 * 1024 && free_bytes * 4 >= db_bytes)
}

/// `VACUUM` completo com a cópia temporária na memória (nunca num arquivo).
pub fn vacuum(conn: &Connection) -> Result<()> {
    let previous = pragma_i64(conn, "temp_store")?;
    conn.execute_batch("PRAGMA temp_store = MEMORY; VACUUM;")?;
    conn.execute_batch(&format!("PRAGMA temp_store = {previous}"))?;
    Ok(())
}

/// Instante (RFC 3339) de `days` dias atrás.
fn days_ago(now: chrono::DateTime<chrono::Utc>, days: i64) -> String {
    (now - chrono::Duration::days(days)).to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Faxina: conversas não fixadas sem uso há mais do que a retenção, bytes de anexos
/// vencidos e, se sobrou muito espaço livre, `VACUUM`.
pub fn cleanup(conn: &Connection, now: chrono::DateTime<chrono::Utc>) -> Result<CleanupReport> {
    let s = settings::load(conn)?;
    let (db_bytes_before, _) = db_size(conn)?;
    let mut report = CleanupReport {
        db_bytes_before,
        ..Default::default()
    };
    if s.history_retention_days > 0 {
        let cutoff = days_ago(now, s.history_retention_days);
        report.messages_deleted = conn.query_row(
            "SELECT count(*) FROM ai_message m JOIN ai_conversation c ON c.id = m.conversation_id
             WHERE c.pinned = 0 AND c.updated_at < ?1",
            [&cutoff],
            |r| r.get(0),
        )?;
        report.conversations_deleted = conn.execute(
            "DELETE FROM ai_conversation WHERE pinned = 0 AND updated_at < ?1",
            [&cutoff],
        )? as i64;
    }
    let now_s = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    report.attachments_expired = super::attachments::expire(conn, &now_s)?;
    let (db, free) = db_size(conn)?;
    if should_vacuum(db, free) {
        vacuum(conn)?;
        report.vacuumed = true;
    }
    report.db_bytes_after = db_size(conn)?.0;
    crate::settings::set(conn, LAST_CLEANUP, &now_utc())?;
    Ok(report)
}

/// A faxina automática está para rodar (nunca rodou ou faz mais de 24 h).
pub fn cleanup_due(conn: &Connection, now: chrono::DateTime<chrono::Utc>) -> Result<bool> {
    let last: Option<String> = crate::settings::get(conn, LAST_CLEANUP)?;
    Ok(
        match last.and_then(|l| chrono::DateTime::parse_from_rfc3339(&l).ok()) {
            Some(t) => now.signed_duration_since(t) >= chrono::Duration::hours(24),
            None => true,
        },
    )
}

/// Faxina automática, se devida (chamada por `ai::run_due` com o cofre aberto).
pub fn run_due(conn: &Connection) -> Result<Option<CleanupReport>> {
    let now = chrono::Utc::now();
    if !cleanup_due(conn, now)? {
        return Ok(None);
    }
    cleanup(conn, now).map(Some)
}

/// Marca uma conversa como usada em `at` (testes da faxina).
#[cfg(test)]
pub(crate) fn touch(conn: &Connection, id: &str, at: &str) -> Result<()> {
    conn.execute(
        "UPDATE ai_conversation SET updated_at = ?2 WHERE id = ?1",
        rusqlite::params![id, at],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
