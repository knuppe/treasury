use serde::Deserialize;
use serde_json::Value;

use super::{RuleApplyArgs, RuleInput, RuleTestArgs, RuleUpdate};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t_args};

/// Motivo do histórico com o nome da regra.
fn reason(key: &str, name: &str) -> String {
    t_args(
        key,
        &i18n::args(&[("nome", name.trim().to_string().into())]),
    )
}

pub const READ_ONLY: &[&str] = &["list", "get", "test", "suggest_from_txn", "learned"];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct TxnIdArg {
    txn_id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => to_value(super::list(ctx.conn())?),
        "get" => to_value(super::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "create" => {
            let input: RuleInput = parse_args(args)?;
            let reason = reason("nucleo-lancamentos-historico-regra-criada", &input.name);
            let id = ctx.write(&reason, |tx| super::create(tx, &input))?.value;
            to_value(super::get(ctx.conn(), &id)?)
        }
        "update" => {
            let u: RuleUpdate = parse_args(args)?;
            let reason = reason("nucleo-lancamentos-historico-regra-alterada", &u.input.name);
            ctx.write(&reason, |tx| super::update(tx, &u.id, &u.input))?;
            to_value(super::get(ctx.conn(), &u.id)?)
        }
        "delete" => {
            let id = parse_args::<IdArg>(args)?.id;
            let name = super::get(ctx.conn(), &id)?.name;
            ctx.write(
                &reason("nucleo-lancamentos-historico-regra-excluida", &name),
                |tx| super::delete(tx, &id),
            )?;
            Ok(Value::Null)
        }
        "test" => to_value(super::test(ctx.conn(), &parse_args::<RuleTestArgs>(args)?)?),
        "apply" => {
            let a: RuleApplyArgs = parse_args(args)?;
            to_value(super::apply(ctx, &a)?)
        }
        "suggest_from_txn" => to_value(super::suggest_from_txn(
            ctx.conn(),
            &parse_args::<TxnIdArg>(args)?.txn_id,
        )?),
        // Regras que o app pode oferecer: a mesma categoria três vezes no mesmo estabelecimento.
        "learned" => to_value(super::learned::learned(
            ctx.conn(),
            crate::time::today_local(),
        )?),
        "learned_ignore" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(i18n::t("atencao-erro-so-interface")));
            }
            let a: super::learned::LearnedIgnoreArgs = parse_args(args)?;
            super::learned::ignore(ctx.conn(), &a.key)?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("rules.{action}"))),
    }
}
