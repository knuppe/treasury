use serde::Deserialize;
use serde_json::Value;

use super::{
    AddDefaultsArgs, CategoryDeleteArgs, CategoryInput, CategoryListArgs, CategoryMergeArgs,
    CategoryPatch, SetGroupArgs, SetGroupResult,
};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const READ_ONLY: &[&str] = &[
    "list",
    "tree",
    "get",
    "usage",
    "missing_defaults",
    "untranslated_defaults",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => to_value(super::list(
            ctx.conn(),
            &parse_args::<CategoryListArgs>(args)?,
        )?),
        "tree" => to_value(super::tree(
            ctx.conn(),
            &parse_args::<CategoryListArgs>(args)?,
        )?),
        "get" => to_value(super::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "usage" => to_value(super::usage(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "missing_defaults" => to_value(super::missing_defaults(ctx.conn())?),
        // Troca de idioma (docs/i18n-design.md › Dados semeados): quantas categorias padrão
        // ainda têm o nome padrão de outro idioma, e traduzi-las (as renomeadas ficam).
        "untranslated_defaults" => to_value(super::untranslated_defaults(ctx.conn())?),
        "translate_defaults" => to_value(
            ctx.write(
                &t("nucleo-lancamentos-historico-categorias-traduzidas"),
                super::translate_defaults,
            )?
            .value,
        ),
        "add_defaults" => {
            let a: AddDefaultsArgs = parse_args(args)?;
            to_value(
                ctx.write(
                    &t("nucleo-lancamentos-historico-categorias-sugeridas"),
                    |tx| super::add_missing_defaults(tx, a.paths.as_deref()),
                )?
                .value,
            )
        }
        "create" => {
            let input: CategoryInput = parse_args(args)?;
            let reason = t_args(
                "nucleo-lancamentos-historico-categoria-criada",
                &i18n::args(&[("nome", input.name.trim().to_string().into())]),
            );
            to_value(ctx.write(&reason, |tx| super::create(tx, &input))?.value)
        }
        "update" => {
            let p: CategoryPatch = parse_args(args)?;
            to_value(
                ctx.write(
                    &t("nucleo-lancamentos-historico-categoria-alterada"),
                    |tx| super::update(tx, &p),
                )?
                .value,
            )
        }
        // Edição em lote do grupo de gasto (Ajustes › Categorias › "Mudar grupo").
        "set_group" => {
            let a: SetGroupArgs = parse_args(args)?;
            let n = a.ids.iter().collect::<std::collections::HashSet<_>>().len();
            let reason = t_args(
                "nucleo-lancamentos-historico-categorias-grupo",
                &i18n::args(&[("n", n.into()), ("grupo", a.group.label().into())]),
            );
            let updated = ctx
                .write(&reason, |tx| super::set_group(tx, &a.ids, a.group))?
                .value;
            to_value(SetGroupResult { updated })
        }
        "delete" => {
            let a: CategoryDeleteArgs = parse_args(args)?;
            let name = super::get(ctx.conn(), &a.id)?.path;
            let reason = t_args(
                match &a.reassign_to {
                    Some(_) => "nucleo-lancamentos-historico-categoria-apagada-reatribuida",
                    None => "nucleo-lancamentos-historico-categoria-apagada",
                },
                &i18n::args(&[("nome", name.into())]),
            );
            to_value(
                ctx.write(&reason, |tx| {
                    super::delete(tx, &a.id, a.reassign_to.as_deref())
                })?
                .value,
            )
        }
        "merge" => {
            let a: CategoryMergeArgs = parse_args(args)?;
            let from = super::get(ctx.conn(), &a.from)?.path;
            let into = super::get(ctx.conn(), &a.into)?.path;
            let reason = t_args(
                "nucleo-lancamentos-historico-categorias-juntadas",
                &i18n::args(&[("origem", from.into()), ("destino", into.into())]),
            );
            to_value(
                ctx.write(&reason, |tx| super::merge(tx, &a.from, &a.into))?
                    .value,
            )
        }
        _ => Err(Error::UnknownCommand(format!("categories.{action}"))),
    }
}
