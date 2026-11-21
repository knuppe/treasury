//! `subscriptions.*`: assinaturas detectadas (lê) e dispensar ou trazer de volta uma
//! detecção (só pela interface: é preferência do usuário, fora do histórico).

use serde::Deserialize;
use serde_json::Value;

use super::{DISMISSED_SETTING, detect, dismissed};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["list"];

#[derive(Deserialize)]
struct KeyArg {
    key: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => to_value(detect(ctx.conn(), today_local())?),
        "dismiss" | "restore" => {
            if *ctx.actor != Actor::User {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-orcamento-erro-assinaturas-so-interface",
                )));
            }
            let key = parse_args::<KeyArg>(args)?.key;
            let mut keys = dismissed(ctx.conn())?;
            keys.retain(|k| k != &key);
            if action == "dismiss" {
                keys.push(key);
            }
            crate::settings::set(ctx.conn(), DISMISSED_SETTING, &keys)?;
            to_value(detect(ctx.conn(), today_local())?)
        }
        _ => Err(Error::UnknownCommand(format!("subscriptions.{action}"))),
    }
}
