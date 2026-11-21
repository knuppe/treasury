use serde_json::Value;

use super::{BudgetGetArgs, CopyBudgetArgs, GroupTargets, SetBudgetArgs, SuggestBudgetArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t_args};
use crate::reports::engine::{month_label, normalize_month};

pub const READ_ONLY: &[&str] = &["get", "suggest", "list_months", "group_targets"];

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "get" => {
            let a: BudgetGetArgs = parse_args(args)?;
            to_value(super::get(ctx.conn(), &a.month, a.basis)?)
        }
        "set_lines" => {
            let a: SetBudgetArgs = parse_args(args)?;
            let month = normalize_month(&a.month)?;
            ctx.write(
                &t_args(
                    "nucleo-orcamento-motivo-atualizado",
                    &i18n::args(&[("mes", month_label(&month).into())]),
                ),
                |tx| super::set_lines(tx, &a),
            )?;
            to_value(super::get(ctx.conn(), &month, None)?)
        }
        "copy" => {
            let a: CopyBudgetArgs = parse_args(args)?;
            let (from, to) = (
                normalize_month(&a.from_month)?,
                normalize_month(&a.to_month)?,
            );
            ctx.write(
                &t_args(
                    "nucleo-orcamento-motivo-copiado",
                    &i18n::args(&[
                        ("de", month_label(&from).into()),
                        ("para", month_label(&to).into()),
                    ]),
                ),
                |tx| super::copy(tx, &a),
            )?;
            to_value(super::get(ctx.conn(), &to, None)?)
        }
        "suggest" => to_value(super::suggest(
            ctx.conn(),
            &parse_args::<SuggestBudgetArgs>(args)?,
        )?),
        "list_months" => to_value(super::list_months(ctx.conn())?),
        // Metas por grupo de gasto (% da renda): ajuste do cofre, como os outros ajustes.
        "group_targets" => to_value(super::group_targets(ctx.conn())?),
        "set_group_targets" => {
            let a: GroupTargets = parse_args(args)?;
            to_value(super::set_group_targets(ctx.conn(), &a)?)
        }
        _ => Err(Error::UnknownCommand(format!("budget.{action}"))),
    }
}
