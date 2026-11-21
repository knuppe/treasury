//! Comandos `longevity.*`. Ler é livre para a interface; os ajustes só mudam pela interface.
//! Clientes MCP (o agente) só leem, e só com a função ligada e sem os membros que esconderam
//! a própria barra (as ferramentas nem aparecem com ela desligada; ver `mcp::tools`).

use serde::Deserialize;
use serde_json::Value;

use super::LongevitySettings;
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const READ_ONLY: &[&str] = &["settings", "stages", "family", "outlook"];

#[derive(Deserialize)]
struct MemberArg {
    member_id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    if !READ_ONLY.contains(&action) && *ctx.actor != Actor::User {
        return Err(Error::Forbidden(t("nucleo-longevidade-erro-so-interface")));
    }
    let agent = matches!(ctx.actor, Actor::Mcp { .. });
    if agent && matches!(action, "family" | "outlook") && !super::enabled(ctx.conn()) {
        return Err(Error::Forbidden(t("nucleo-longevidade-erro-desligada")));
    }
    match action {
        "settings" => to_value(super::settings(ctx.conn())?),
        "set_settings" => {
            let s: LongevitySettings = parse_args(args)?;
            to_value(super::save_settings(ctx.conn(), &s)?)
        }
        "stages" => to_value(super::stages()),
        "family" => to_value(super::family(ctx.conn())?),
        "outlook" => {
            let a: MemberArg = parse_args(args)?;
            if agent
                && super::settings(ctx.conn())?
                    .hidden_members
                    .contains(&a.member_id)
            {
                let name = crate::household::get_member(ctx.conn(), &a.member_id)?.name;
                return Err(Error::Forbidden(t_args(
                    "nucleo-longevidade-erro-escondido",
                    &i18n::args(&[("nome", name.into())]),
                )));
            }
            to_value(super::outlook(ctx.conn(), &a.member_id)?)
        }
        _ => Err(Error::UnknownCommand(format!("longevity.{action}"))),
    }
}
