//! Retenção (docs/email-ingest.md §Retenção): o que já pode sair do provedor (ou da pasta
//! local). Nunca apaga fora das pastas permitidas, numa pasta bloqueada, sem regra casada
//! (o padrão da conta não apaga) nem no modo de teste; com "guardar original", só depois de
//! conferir que a cópia no cofre está inteira. Toda exclusão vai para a auditoria e não tem
//! desfazer; o modo de teste registra o que teria apagado.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, params};

use super::config::{self, Retention, RetentionMode, RetentionState};
use super::filter;
use super::ingest::{Prepared, sha256_hex};
use super::store::{self, MailStatus};
use crate::error::Result;
use crate::i18n::{args, t, t_args};
use crate::ids::Id;

/// Uma mensagem que a retenção tiraria.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: Id,
    pub folder: String,
    pub uid: Option<u32>,
    pub uidvalidity: Option<u32>,
    pub file_name: Option<String>,
    /// Message-ID (para achar na lixeira do Gmail, se pedido).
    pub message_header_id: Option<String>,
    pub subject: String,
    pub reason: String,
    /// Retenção que vale (a da regra ou a da conta).
    pub retention: Retention,
}

pub enum Plan {
    /// Nada a fazer (exclusão desligada ou nada vencido).
    Nothing,
    /// Modo de teste (ou esperando confirmação): só registrar.
    Test(Vec<Candidate>),
    Delete(Vec<Candidate>),
}

fn parse_at(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// O que já venceu, pelas regras de retenção.
pub fn candidates(conn: &Connection, p: &Prepared, now: DateTime<Utc>) -> Result<Vec<Candidate>> {
    let acc = &p.account;
    let mut stmt = conn.prepare(
        "SELECT id, folder, uid, uidvalidity, file_name, message_id, subject, rule_id, status, received_at,
                decided_at, raw, raw_hash
           FROM mail_message
          WHERE account_id = ?1 AND deleted_at IS NULL AND rule_id IS NOT NULL",
    )?;
    type Row = (
        String,
        String,
        Option<i64>,
        Option<i64>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        String,
        Option<String>,
        Option<Vec<u8>>,
        Option<String>,
    );
    let rows: Vec<Row> = stmt
        .query_map([&acc.id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
                r.get(10)?,
                r.get(11)?,
                r.get(12)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (
        id,
        folder,
        uid,
        uidv,
        file,
        msgid,
        subject,
        rule_id,
        status,
        received,
        decided,
        raw,
        hash,
    ) in rows
    {
        // Sem a regra (apagada ou desligada), nada sai.
        let Some(rule) = p.rules.iter().find(|r| r.id == rule_id) else {
            continue;
        };
        let ret = rule
            .retention
            .clone()
            .unwrap_or_else(|| acc.retention.clone());
        if !ret.deletes() {
            continue;
        }
        // A pasta continua permitida e não bloqueada (pelo nome; o IMAP confere a marca).
        if !acc
            .allowed_folders
            .iter()
            .any(|f| filter::folder_matches(f, &folder, &[]))
            || acc
                .blocked_folders
                .iter()
                .any(|f| filter::folder_matches(f, &folder, &[]))
        {
            continue;
        }
        let status = MailStatus::parse(&status);
        let success = matches!(status, MailStatus::Imported | MailStatus::Duplicate)
            && decided
                .as_deref()
                .and_then(parse_at)
                .is_some_and(|d| d + Duration::days(ret.grace_days as i64) <= now);
        let aged = parse_at(&received).is_some_and(|d| d + Duration::days(ret.days as i64) <= now);
        let after_success = || {
            t_args(
                "nucleo-email-retencao-motivo-sucesso",
                &args(&[
                    ("status", label(status).into()),
                    ("dias", ret.grace_days.into()),
                ]),
            )
        };
        let received = || {
            t_args(
                "nucleo-email-retencao-motivo-recebida",
                &args(&[("dias", ret.days.into())]),
            )
        };
        let reason = match ret.mode {
            RetentionMode::Never => continue,
            RetentionMode::AfterSuccess if success => after_success(),
            RetentionMode::AfterDays if aged => received(),
            RetentionMode::Both if success => after_success(),
            RetentionMode::Both if aged => received(),
            _ => continue,
        };
        // Guardar original: a cópia no cofre precisa estar inteira antes de apagar.
        if acc.keep_original && !matches!((&raw, &hash), (Some(r), Some(h)) if &sha256_hex(r) == h)
        {
            let logged: bool = conn.query_row(
                "SELECT EXISTS (SELECT 1 FROM mail_audit WHERE message_id = ?1 AND action = 'skipped_delete')",
                [&id],
                |r| r.get(0),
            )?;
            if !logged {
                store::audit(
                    conn,
                    Some(&acc.id),
                    "skipped_delete",
                    Some(&id),
                    &t("nucleo-email-retencao-copia-diferente"),
                )?;
            }
            continue;
        }
        out.push(Candidate {
            id,
            folder,
            uid: uid.and_then(|u| u32::try_from(u).ok()),
            uidvalidity: uidv.and_then(|u| u32::try_from(u).ok()),
            file_name: file,
            message_header_id: msgid,
            subject: subject.unwrap_or_default(),
            reason,
            retention: ret,
        });
    }
    Ok(out)
}

/// Para o seletor do catálogo (`[repetida]`, `[importada]`).
fn label(s: MailStatus) -> &'static str {
    if s == MailStatus::Duplicate {
        "repetida"
    } else {
        "importada"
    }
}

/// Decide o que fazer agora: nada, registrar (teste) ou apagar. No teste, cada mensagem é
/// registrada uma vez como "teria apagado".
pub fn plan(conn: &Connection, p: &Prepared, now: DateTime<Utc>) -> Result<Plan> {
    let state = config::retention_state(
        conn,
        &p.account,
        &now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    )?;
    if state == RetentionState::Off {
        return Ok(Plan::Nothing);
    }
    let list = candidates(conn, p, now)?;
    if list.is_empty() {
        return Ok(Plan::Nothing);
    }
    if state == RetentionState::Active {
        return Ok(Plan::Delete(list));
    }
    for c in &list {
        let logged: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM mail_audit WHERE message_id = ?1 AND action = 'would_delete')",
            [&c.id],
            |r| r.get(0),
        )?;
        if !logged {
            store::audit(
                conn,
                Some(&p.account.id),
                "would_delete",
                Some(&c.id),
                &t_args(
                    "nucleo-email-retencao-teste",
                    &args(&[
                        ("assunto", c.subject.clone().into()),
                        ("motivo", c.reason.clone().into()),
                    ]),
                ),
            )?;
        }
    }
    Ok(Plan::Test(list))
}

/// Registra o resultado das exclusões (a auditoria é a única memória delas).
pub fn record(
    conn: &Connection,
    account_id: &str,
    results: &[(Candidate, std::result::Result<String, String>)],
) -> Result<u32> {
    let mut n = 0;
    for (c, r) in results {
        match r {
            Ok(how) => {
                n += 1;
                conn.execute(
                    "UPDATE mail_message SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
                    params![c.id, crate::time::now_utc()],
                )?;
                store::audit(
                    conn,
                    Some(account_id),
                    "deleted",
                    Some(&c.id),
                    &t_args(
                        "nucleo-email-retencao-apagada",
                        &args(&[
                            ("assunto", c.subject.clone().into()),
                            ("motivo", c.reason.clone().into()),
                            ("como", how.clone().into()),
                        ]),
                    ),
                )?;
            }
            Err(e) => store::audit(
                conn,
                Some(account_id),
                "error",
                Some(&c.id),
                &t_args(
                    "nucleo-email-retencao-erro",
                    &args(&[
                        ("assunto", c.subject.clone().into()),
                        ("erro", e.clone().into()),
                    ]),
                ),
            )?,
        }
    }
    Ok(n)
}

/// O que o modo de teste já registrou (para a tela de confirmação).
pub fn would_delete(conn: &Connection, account_id: &str) -> Result<Vec<store::MailAuditEntry>> {
    Ok(store::audit_list(
        conn,
        &store::MailAuditArgs {
            account_id: Some(account_id.into()),
            limit: Some(2000),
        },
    )?
    .into_iter()
    .filter(|e| e.action == "would_delete")
    .collect())
}
