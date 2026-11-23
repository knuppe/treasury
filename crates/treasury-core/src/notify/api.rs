//! `notify.*`: preferências de notificação (troca só pela interface) e conferir os
//! lembretes agora.

use serde_json::{Value, json};

use super::NotifyPrefs;
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["get_prefs"];

#[derive(serde::Deserialize)]
struct ReminderArgs {
    recurrence_id: String,
    remind: bool,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "get_prefs" => to_value(super::prefs(ctx.conn())?),
        "set_prefs" | "run_now" | "set_recurrence_reminder" if *ctx.actor != Actor::User => Err(
            Error::Forbidden(crate::i18n::t("nucleo-base-notificacoes-so-interface")),
        ),
        "set_prefs" => to_value(super::save_prefs(
            ctx.conn(),
            &parse_args::<NotifyPrefs>(args)?,
        )?),
        "run_now" => {
            let r = super::run(ctx.conn(), today_local())?;
            Ok(json!({"created": r.created.len()}))
        }
        // Aviso de vencimento de uma recorrência (diarista, mesada): ligar ou desligar.
        "set_recurrence_reminder" => {
            let a: ReminderArgs = parse_args(args)?;
            let resolved = super::quiet::set(ctx.conn(), &a.recurrence_id, a.remind)?;
            Ok(json!({"recurrence_id": a.recurrence_id, "remind": a.remind, "resolved": resolved}))
        }
        _ => Err(Error::UnknownCommand(format!("notify.{action}"))),
    }
}
