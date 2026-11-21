//! `notify.*`: preferências de notificação (troca só pela interface) e conferir os
//! lembretes agora.

use serde_json::{Value, json};

use super::NotifyPrefs;
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["get_prefs"];

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "get_prefs" => to_value(super::prefs(ctx.conn())?),
        "set_prefs" | "run_now" if *ctx.actor != Actor::User => Err(Error::Forbidden(
            crate::i18n::t("nucleo-base-notificacoes-so-interface"),
        )),
        "set_prefs" => to_value(super::save_prefs(
            ctx.conn(),
            &parse_args::<NotifyPrefs>(args)?,
        )?),
        "run_now" => {
            let r = super::run(ctx.conn(), today_local())?;
            Ok(json!({"created": r.created.len()}))
        }
        _ => Err(Error::UnknownCommand(format!("notify.{action}"))),
    }
}
