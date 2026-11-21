//! `demo.*`: gera o histórico de exemplo no cofre aberto (sem lançamentos). O assistente usa
//! `vaults.new_demo`, que cria o cofre separado e chama este comando.

use serde_json::Value;

use super::DemoProfile;
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &[];

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "generate" => {
            if *ctx.actor != Actor::User {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-geral-erro-demo-so-interface",
                )));
            }
            let p: DemoProfile = parse_args(args)?;
            let today = today_local();
            let r = ctx.write(&crate::i18n::t("nucleo-geral-motivo-demo"), |tx| {
                super::generate(tx, &p, today)
            })?;
            to_value(r.value)
        }
        _ => Err(Error::UnknownCommand(format!("demo.{action}"))),
    }
}
