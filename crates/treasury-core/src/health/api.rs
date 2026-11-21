//! Comandos `health.*`: a nota (só lê) e o modelo escolhido (troca só pela interface).

use serde_json::Value;

use super::{HealthScoreArgs, HealthSettings};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["score", "get_settings"];

fn user_only(ctx: &Ctx<'_>) -> Result<()> {
    if *ctx.actor != Actor::User {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-saude-erro-so-interface",
        )));
    }
    Ok(())
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "score" => {
            let mut v = to_value(super::score(
                ctx.conn(),
                &parse_args::<HealthScoreArgs>(args)?,
                today_local(),
            )?)?;
            // Dicas nos termos do país da família (docs/countries.md), no idioma da instalação.
            crate::country::localize_value(ctx.conn(), &mut v);
            Ok(v)
        }
        "get_settings" => to_value(super::settings_view(ctx.conn())?),
        "set_settings" => {
            user_only(ctx)?;
            let s: HealthSettings = parse_args(args)?;
            super::save_settings(ctx.conn(), &s)?;
            to_value(super::settings_view(ctx.conn())?)
        }
        _ => Err(Error::UnknownCommand(format!("health.{action}"))),
    }
}
