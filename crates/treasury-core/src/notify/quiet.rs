//! Recorrências sem aviso de vencimento (docs/notifications.md › "Sem aviso"): despesas que a
//! família paga sem pensar (diarista, mesada) e que não precisam de lembrete. O previsto continua
//! na previsão de saldo, no orçamento e nos próximos vencimentos do Painel; só os avisos somem
//! (lembretes, rotina de alertas e "Para resolver").
//!
//! Fica no ajuste `notify.quiet_recurrences` do cofre (lista de ids), sem migração.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};

use crate::error::{Error, Result};
use crate::i18n::{args, t_args};
use crate::time::{fmt_date, now_utc};

pub const SETTING: &str = "notify.quiet_recurrences";

/// As recorrências que não avisam o vencimento.
pub fn list(conn: &Connection) -> Result<BTreeSet<String>> {
    crate::settings::get_or(conn, SETTING, BTreeSet::new())
}

/// O previsto desta recorrência deve ficar fora dos avisos?
pub fn is_quiet(quiet: &BTreeSet<String>, recurrence_id: Option<&str>) -> bool {
    recurrence_id.is_some_and(|r| quiet.contains(r))
}

/// Liga (`remind = true`) ou desliga o aviso de vencimento de uma recorrência. Ao desligar, os
/// alertas de vencimento ainda abertos dos previstos dela são resolvidos. Devolve quantos.
pub fn set(conn: &Connection, recurrence_id: &str, remind: bool) -> Result<usize> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM recurrence WHERE id = ?1",
            [recurrence_id],
            |r| r.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Err(Error::not_found(t_args(
            "nucleo-lancamentos-ref-recorrencia",
            &args(&[("id", recurrence_id.to_string().into())]),
        )));
    }
    let mut quiet = list(conn)?;
    // Faxina: ids de recorrências apagadas saem da lista.
    let mut alive = conn.prepare("SELECT 1 FROM recurrence WHERE id = ?1")?;
    quiet.retain(|id| alive.exists([id]).unwrap_or(true));
    if remind {
        quiet.remove(recurrence_id);
    } else {
        quiet.insert(recurrence_id.to_string());
    }
    crate::settings::set(conn, SETTING, &quiet)?;
    if remind {
        return Ok(0);
    }
    let now = now_utc();
    Ok(conn.execute(
        "UPDATE alert SET status = 'resolved', resolved_at = ?2, updated_at = ?2
          WHERE kind = 'due_soon' AND status <> 'resolved' AND ref_table = 'txn'
            AND ref_id IN (SELECT id FROM txn WHERE recurrence_id = ?1)",
        params![recurrence_id, now],
    )?)
}

/// Alertas de vencimento que perderam o sentido: a data já passou ("vence em 5 dias" de uma
/// semana atrás), o previsto foi efetivado ou apagado, ou a fatura foi paga. O vencido sem
/// pagamento continua em "Para resolver", calculado na hora. Devolve quantos foram resolvidos.
pub fn close_stale_due(conn: &Connection, today: NaiveDate) -> Result<usize> {
    let now = now_utc();
    Ok(conn.execute(
        "UPDATE alert SET status = 'resolved', resolved_at = ?2, updated_at = ?2
          WHERE kind = 'due_soon' AND status <> 'resolved'
            AND (json_extract(data, '$.date') < ?1
                 OR (ref_table = 'txn' AND NOT EXISTS
                       (SELECT 1 FROM txn t WHERE t.id = alert.ref_id AND t.status = 'planned'))
                 OR (ref_table = 'card_statement' AND EXISTS
                       (SELECT 1 FROM card_statement s WHERE s.id = alert.ref_id
                          AND s.status = 'paid')))",
        params![fmt_date(today), now],
    )?)
}
