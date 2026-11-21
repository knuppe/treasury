//! `income.*`: rendas dos membros, holerites e a previsão dos próximos meses.

use serde::Deserialize;
use serde_json::Value;

use super::payslip::{self, PayslipArgs};
use super::{IncomeSourceInput, IncomeSourcePatch, IncomeSummaryArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::time::today_local;

/// Motivo da alteração com o nome da renda.
fn reason(key: &str, name: &str) -> String {
    t_args(key, &args(&[("renda", name.trim().to_string().into())]))
}

pub const READ_ONLY: &[&str] = &[
    "list_sources",
    "summary",
    "payslip_preview",
    "list_payslips",
    "simulate_termination",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize, Default)]
struct ListArgs {
    #[serde(default)]
    member_id: Option<String>,
    #[serde(default)]
    source_id: Option<String>,
    #[serde(default)]
    include_archived: bool,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "simulate_termination" => to_value(super::termination::simulate(
            ctx.conn(),
            &parse_args::<super::termination::TerminationInput>(args)?,
        )?),
        "list_sources" => {
            let a: ListArgs = parse_args(args)?;
            to_value(super::list_sources(
                ctx.conn(),
                a.member_id.as_deref(),
                a.include_archived,
            )?)
        }
        "summary" => to_value(super::summary(
            ctx.conn(),
            &parse_args::<IncomeSummaryArgs>(args)?,
            today_local(),
        )?),
        "create_source" => {
            let i: IncomeSourceInput = parse_args(args)?;
            let reason = reason("nucleo-renda-motivo-criada", &i.name);
            let id = ctx.write(&reason, |tx| super::create_source(tx, &i))?.value;
            to_value(super::get_source(ctx.conn(), &id)?)
        }
        "update_source" => {
            let p: IncomeSourcePatch = parse_args(args)?;
            let reason = reason("nucleo-renda-motivo-alterada", &p.source.name);
            ctx.write(&reason, |tx| super::update_source(tx, &p))?;
            to_value(super::get_source(ctx.conn(), &p.id)?)
        }
        "delete_source" => {
            let id = parse_args::<IdArg>(args)?.id;
            let name = super::get_source(ctx.conn(), &id)?.name;
            ctx.write(&reason("nucleo-renda-motivo-apagada", &name), |tx| {
                super::delete_source(tx, &id)
            })?;
            Ok(serde_json::json!({"deleted": id}))
        }
        "payslip_preview" => to_value(payslip::preview(
            ctx.conn(),
            &parse_args::<PayslipArgs>(args)?,
        )?),
        "payslip_import" => {
            let a: PayslipArgs = parse_args(args)?;
            let r = ctx.write(&t("nucleo-renda-motivo-holerite-importado"), |tx| {
                payslip::import(tx, &a)
            })?;
            to_value(r.value)
        }
        "list_payslips" => {
            let a: ListArgs = parse_args(args)?;
            to_value(payslip::list(
                ctx.conn(),
                a.source_id.as_deref(),
                a.member_id.as_deref(),
            )?)
        }
        "delete_payslip" => {
            let id = parse_args::<IdArg>(args)?.id;
            ctx.write(&t("nucleo-renda-motivo-holerite-apagado"), |tx| {
                payslip::delete(tx, &id)
            })?;
            Ok(serde_json::json!({"deleted": id}))
        }
        _ => Err(Error::UnknownCommand(format!("income.{action}"))),
    }
}
