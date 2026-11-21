//! `attention.*`: a lista "Para resolver", o próximo passo e o roteiro de fechar o mês
//! (leitura, também para clientes MCP); dispensar um item e marcar os passos do mês só pela
//! interface.

use serde_json::{Value, json};

use super::AttentionDismissArgs;
use super::closing::{MonthCloseArgs, MonthCloseSetArgs};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["list", "coverage", "next_step", "month_close"];

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => {
            let mut v = to_value(super::list(ctx.conn(), today_local())?)?;
            // Avisos nos termos do país da família (docs/countries.md).
            crate::country::localize_value(ctx.conn(), &mut v);
            Ok(v)
        }
        "coverage" => to_value(super::coverage(ctx.conn(), today_local())?),
        "next_step" => {
            let mut v = to_value(super::ladder::next_step(ctx.conn(), today_local())?)?;
            crate::country::localize_value(ctx.conn(), &mut v);
            Ok(v)
        }
        "month_close" => {
            let a: MonthCloseArgs = parse_args(args)?;
            let today = today_local();
            let attention = super::list(ctx.conn(), today)?;
            to_value(super::closing::month_close(
                ctx.conn(),
                &a,
                &attention,
                today,
            )?)
        }
        "dismiss" | "month_close_set" if *ctx.actor != Actor::User => Err(Error::Forbidden(
            crate::i18n::t("atencao-erro-so-interface"),
        )),
        "month_close_set" => {
            let a: MonthCloseSetArgs = parse_args(args)?;
            let today = today_local();
            // Fechar só com os cinco passos feitos (reabrir e marcar passos, sempre).
            if a.step == "closed" && a.done {
                let attention = super::list(ctx.conn(), today)?;
                let month = MonthCloseArgs {
                    month: Some(a.month.clone()),
                };
                let c = super::closing::month_close(ctx.conn(), &month, &attention, today)?;
                if c.done < c.steps.len() as i64 {
                    return Err(Error::invalid(crate::i18n::t("fechamento-erro-incompleto")));
                }
            }
            super::closing::set(ctx.conn(), &a, today)?;
            Ok(json!({"month": a.month, "step": a.step, "done": a.done}))
        }
        "dismiss" => {
            let a: AttentionDismissArgs = parse_args(args)?;
            super::dismiss(ctx.conn(), &a.id, today_local())?;
            Ok(json!({"dismissed": a.id}))
        }
        _ => Err(Error::UnknownCommand(format!("attention.{action}"))),
    }
}
