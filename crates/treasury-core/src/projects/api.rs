use serde::Deserialize;
use serde_json::Value;

use super::{
    ProjectInput, ProjectItemInput, ProjectItemPatch, ProjectLinkArgs, ProjectListArgs,
    ProjectPatch, ProjectRemoveArgs,
};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t_args};
use crate::transactions::BulkResult;

pub const READ_ONLY: &[&str] = &["list", "get", "entries"];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => to_value(super::list(
            ctx.conn(),
            &parse_args::<ProjectListArgs>(args)?,
        )?),
        "get" => to_value(super::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "entries" => to_value(super::entries(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "create" => {
            let input: ProjectInput = parse_args(args)?;
            let reason = t_args(
                "nucleo-orcamento-motivo-projeto-criado",
                &i18n::args(&[("nome", input.name.trim().to_string().into())]),
            );
            let id = ctx.write(&reason, |tx| super::create(tx, &input))?.value;
            to_value(super::get(ctx.conn(), &id)?)
        }
        "update" => {
            let patch: ProjectPatch = parse_args(args)?;
            let name = super::name_of(ctx.conn(), &patch.id)?;
            let reason = t_args(
                "nucleo-orcamento-motivo-projeto-alterado",
                &i18n::args(&[("nome", name.into())]),
            );
            ctx.write(&reason, |tx| super::update(tx, &patch))?;
            to_value(super::get(ctx.conn(), &patch.id)?)
        }
        "delete" => {
            let a: IdArg = parse_args(args)?;
            let name = super::name_of(ctx.conn(), &a.id)?;
            let reason = t_args(
                "nucleo-orcamento-motivo-projeto-removido",
                &i18n::args(&[("nome", name.into())]),
            );
            ctx.write(&reason, |tx| super::delete(tx, &a.id))?;
            Ok(Value::Null)
        }
        "link" => {
            let a: ProjectLinkArgs = parse_args(args)?;
            let n = a.txn_ids.len() as i64;
            let reason = match &a.project_id {
                Some(p) => t_args(
                    "nucleo-orcamento-motivo-lancamentos-no-projeto",
                    &i18n::args(&[
                        ("n", n.into()),
                        ("nome", super::name_of(ctx.conn(), p)?.into()),
                    ]),
                ),
                None => t_args(
                    "nucleo-orcamento-motivo-lancamentos-tirados",
                    &i18n::args(&[("n", n.into())]),
                ),
            };
            let out = ctx.write(&reason, |tx| {
                super::link(tx, &a.txn_ids, a.project_id.as_deref())
            })?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "add_item" => {
            let input: ProjectItemInput = parse_args(args)?;
            let reason = t_args(
                "nucleo-orcamento-motivo-item-adicionado",
                &i18n::args(&[
                    (
                        "nome",
                        super::name_of(ctx.conn(), &input.project_id)?.into(),
                    ),
                    ("item", input.description.trim().to_string().into()),
                ]),
            );
            let id = ctx.write(&reason, |tx| super::add_item(tx, &input))?.value;
            to_value(super::entries::item_entry(ctx.conn(), &id)?)
        }
        "update_item" => {
            let patch: ProjectItemPatch = parse_args(args)?;
            let reason = t_args(
                "nucleo-orcamento-motivo-item-alterado",
                &i18n::args(&[(
                    "item",
                    super::entries::item_description(ctx.conn(), &patch.id)?.into(),
                )]),
            );
            ctx.write(&reason, |tx| super::update_item(tx, &patch))?;
            to_value(super::entries::item_entry(ctx.conn(), &patch.id)?)
        }
        "remove_entries" => {
            let a: ProjectRemoveArgs = parse_args(args)?;
            let n = (a.txn_ids.len() + a.item_ids.len()) as i64;
            let reason = t_args(
                "nucleo-orcamento-motivo-entradas-tiradas",
                &i18n::args(&[("n", n.into())]),
            );
            let out = ctx.write(&reason, |tx| super::remove_entries(tx, &a))?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        _ => Err(Error::UnknownCommand(format!("projects.{action}"))),
    }
}
