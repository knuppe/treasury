use serde::Deserialize;
use serde_json::Value;

use super::{HouseholdPatch, InstitutionInput, InstitutionPatch, MemberInput, MemberPatch};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

/// Motivo da alteração com um nome (`{ $nome }`).
fn named(key: &str, name: &str) -> String {
    t_args(key, &args(&[("nome", name.trim().to_string().into())]))
}

pub const READ_ONLY: &[&str] = &[
    "get",
    "list_members",
    "list_institutions",
    "banks",
    "members_overview",
];

#[derive(Deserialize, Default)]
struct ListMembersArgs {
    #[serde(default)]
    include_archived: bool,
}

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct ArchiveArgs {
    id: String,
    #[serde(default)]
    archived: Option<bool>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "get" => to_value(super::get(ctx.conn())?),
        // Finanças por membro (docs/members.md).
        "members_overview" => to_value(super::overview::overview(
            ctx.conn(),
            &parse_args::<super::overview::MembersOverviewArgs>(args)?,
        )?),
        "update" => {
            let p: HouseholdPatch = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-geral-motivo-familia"), |tx| {
                    super::update(tx, &p)
                })?
                .value,
            )
        }
        "list_members" => {
            let a: ListMembersArgs = parse_args(args)?;
            to_value(super::list_members(ctx.conn(), a.include_archived)?)
        }
        "create_member" => {
            let input: MemberInput = parse_args(args)?;
            let reason = named("nucleo-geral-motivo-membro-criado", &input.name);
            to_value(
                ctx.write(&reason, |tx| super::create_member(tx, &input))?
                    .value,
            )
        }
        "update_member" => {
            let p: MemberPatch = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-geral-motivo-membro-alterado"), |tx| {
                    super::update_member(tx, &p)
                })?
                .value,
            )
        }
        "archive_member" => {
            let a: ArchiveArgs = parse_args(args)?;
            let archived = a.archived.unwrap_or(true);
            let reason = t(if archived {
                "nucleo-geral-motivo-membro-arquivado"
            } else {
                "nucleo-geral-motivo-membro-reativado"
            });
            let patch = MemberPatch {
                id: a.id,
                name: None,
                color: None,
                is_primary: None,
                archived: Some(archived),
                sort_order: None,
                profile: None,
            };
            to_value(
                ctx.write(&reason, |tx| super::update_member(tx, &patch))?
                    .value,
            )
        }
        "list_institutions" => to_value(super::list_institutions(ctx.conn())?),
        "banks" => to_value(super::banks::search(ctx.conn(), &parse_args(args)?)?),
        "create_institution" => {
            let input: InstitutionInput = parse_args(args)?;
            let reason = named("nucleo-geral-motivo-instituicao-criada", &input.name);
            to_value(
                ctx.write(&reason, |tx| super::create_institution(tx, &input))?
                    .value,
            )
        }
        "update_institution" => {
            let p: InstitutionPatch = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-geral-motivo-instituicao-alterada"), |tx| {
                    super::update_institution(tx, &p)
                })?
                .value,
            )
        }
        "delete_institution" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-geral-motivo-instituicao-apagada"), |tx| {
                super::delete_institution(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("household.{action}"))),
    }
}
