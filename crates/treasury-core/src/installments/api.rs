//! `installments.*`: parcelas detectadas pela descrição (lê) e "não é parcela" (só pela
//! interface ou proposta do agente; desfazível pelo histórico).

use serde::Deserialize;
use serde_json::Value;

use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["detected", "ignored"];

#[derive(Deserialize)]
struct KeyArg {
    key: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args_v: Value) -> Result<Value> {
    let today = today_local();
    match action {
        "detected" => to_value(super::detect(ctx.conn(), today)?),
        "ignored" => to_value(super::list_ignored(ctx.conn())?),
        "ignore" => {
            let key = parse_args::<KeyArg>(args_v)?.key;
            let all = super::detect(ctx.conn(), today)?;
            let s = all
                .series
                .into_iter()
                .find(|s| s.key == key)
                .ok_or_else(|| Error::not_found(t("nucleo-parcelas-ref-serie")))?;
            let reason = t_args(
                "nucleo-parcelas-historico-ignorar",
                &args(&[("descricao", s.description.clone().into())]),
            );
            ctx.write(&reason, |tx| super::ignore(tx, &s))?;
            to_value(super::detect(ctx.conn(), today)?)
        }
        "unignore" => {
            let key = parse_args::<KeyArg>(args_v)?.key;
            ctx.write(&t("nucleo-parcelas-historico-voltar"), |tx| {
                super::unignore(tx, &key)
            })?;
            to_value(super::detect(ctx.conn(), today)?)
        }
        _ => Err(Error::UnknownCommand(format!("installments.{action}"))),
    }
}
