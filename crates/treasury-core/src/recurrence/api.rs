use serde::Deserialize;
use serde_json::Value;

use super::{
    DEFAULT_HORIZON_DAYS, MaterializeArgs, MaterializeResult, PreviewArgs, RecurrenceDeleteArgs,
    RecurrenceInput, RecurrenceListArgs, RecurrencePatch,
};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t};
use crate::time::today_local;
use crate::transactions::api::with_desc;

pub const READ_ONLY: &[&str] = &["list", "get", "preview"];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let today = today_local();
    match action {
        "list" => to_value(super::list(
            ctx.conn(),
            &parse_args::<RecurrenceListArgs>(args)?,
            today,
        )?),
        "get" => to_value(super::get(
            ctx.conn(),
            &parse_args::<IdArg>(args)?.id,
            today,
        )?),
        "preview" => to_value(super::preview(&parse_args::<PreviewArgs>(args)?)?),
        "create" => {
            let input: RecurrenceInput = parse_args(args)?;
            let reason = with_desc(
                "nucleo-lancamentos-historico-recorrencia-criada",
                &input.description,
            );
            let id = ctx
                .write(&reason, |tx| super::create(tx, &input, today))?
                .value;
            to_value(super::get(ctx.conn(), &id, today)?)
        }
        "update" => {
            let p: RecurrencePatch = parse_args(args)?;
            let cur = super::get(ctx.conn(), &p.id, today)?;
            let reason = with_desc(
                "nucleo-lancamentos-historico-recorrencia-alterada",
                &cur.description,
            );
            ctx.write(&reason, |tx| super::update(tx, &p, today))?;
            to_value(super::get(ctx.conn(), &p.id, today)?)
        }
        "delete" => {
            let a: RecurrenceDeleteArgs = parse_args(args)?;
            let cur = super::get(ctx.conn(), &a.id, today)?;
            let reason = with_desc(
                "nucleo-lancamentos-historico-recorrencia-apagada",
                &cur.description,
            );
            let out = ctx.write(&reason, |tx| {
                super::delete(tx, &a.id, a.delete_planned.unwrap_or(true), today)
            })?;
            to_value(super::RecurrenceDeleteResult {
                deleted_planned: out.value as i64,
                change_set_id: out.change_set_id,
            })
        }
        "materialize" => {
            let a: MaterializeArgs = parse_args(args)?;
            let horizon = a.horizon_days.unwrap_or(DEFAULT_HORIZON_DAYS);
            if !(0..=super::MAX_HORIZON_DAYS).contains(&horizon) {
                return Err(Error::invalid(i18n::t_args(
                    "nucleo-lancamentos-recorrencia-erro-horizonte",
                    &i18n::args(&[("max", super::MAX_HORIZON_DAYS.into())]),
                )));
            }
            let out = ctx.write(&t("nucleo-lancamentos-historico-previstos"), |tx| {
                super::materialize_all(tx, today, horizon)
            })?;
            to_value(MaterializeResult {
                created: out.value as i64,
                change_set_id: out.change_set_id,
            })
        }
        _ => Err(Error::UnknownCommand(format!("recurrences.{action}"))),
    }
}
