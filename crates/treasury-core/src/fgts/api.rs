//! `fgts.*` (docs/brazil/fgts.md, seção 7): contas vinculadas, extrato da Caixa, conferência, saques,
//! moradia, saque-aniversário, projeção e comparação do rendimento.

use serde::Deserialize;
use serde_json::Value;

use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &[
    "list",
    "entries",
    "statement_preview",
    "check",
    "yield_compare",
    "project",
    "anniversary_simulate",
];

#[derive(Deserialize)]
struct AccountArg {
    account_id: String,
}

#[derive(Deserialize, Default)]
struct ListArg {
    /// Sem as contas arquivadas.
    #[serde(default)]
    active_only: bool,
}

#[derive(Deserialize, Default)]
struct MemberArg {
    #[serde(default)]
    member_id: Option<String>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    super::require_brazil(ctx.conn())?;
    let today = today_local();
    match action {
        "list" => {
            let a: ListArg = if args.is_null() {
                ListArg::default()
            } else {
                parse_args(args)?
            };
            let mut list = super::list(ctx.conn(), today)?;
            if a.active_only {
                list.retain(|x| !x.archived);
            }
            to_value(list)
        }
        "entries" => {
            let a: AccountArg = parse_args(args)?;
            to_value(super::entries(ctx.conn(), &a.account_id)?)
        }
        "statement_preview" => {
            let a: super::statement::StatementArgs = parse_args(args)?;
            to_value(super::statement::preview(ctx.conn(), &a)?)
        }
        "statement_import" => {
            let a: super::statement::StatementArgs = parse_args(args)?;
            let r = ctx.write(&t("nucleo-fgts-motivo-extrato"), |tx| {
                super::statement::import(tx, &a, today)
            })?;
            to_value(r.value)
        }
        "check" => {
            let a: AccountArg = parse_args(args)?;
            to_value(super::check(ctx.conn(), &a.account_id, today)?)
        }
        "yield_compare" => {
            let a: super::compare::CompareArgs = parse_args(args)?;
            to_value(super::compare::compare(ctx.conn(), &a, today)?)
        }
        "project" => {
            let a: super::ProjectArgs = parse_args(args)?;
            to_value(super::project(ctx.conn(), &a, today)?)
        }
        "anniversary_simulate" => {
            let a: MemberArg = parse_args(args)?;
            to_value(super::anniversary(
                ctx.conn(),
                a.member_id.as_deref(),
                today,
            )?)
        }
        "record_withdrawal" => {
            let w: super::WithdrawalInput = parse_args(args)?;
            let r = ctx.write(&t("nucleo-fgts-motivo-saque"), |tx| {
                super::record_withdrawal(tx, &w, today)
            })?;
            to_value(serde_json::json!({ "txn_id": r.value }))
        }
        "use_for_housing" => {
            let h: super::HousingInput = parse_args(args)?;
            let r = ctx.write(&t("nucleo-fgts-motivo-moradia"), |tx| {
                super::use_for_housing(tx, &h, today)
            })?;
            to_value(r.value)
        }
        "set_modality" => {
            let m: super::ModalityInput = parse_args(args)?;
            ctx.write(&t("nucleo-fgts-motivo-modalidade"), |tx| {
                super::set_modality(tx, &m, today)
            })?;
            to_value(super::anniversary(ctx.conn(), Some(&m.member_id), today)?)
        }
        "update_account" => {
            let p: super::AccountPatch = parse_args(args)?;
            ctx.write(&t("nucleo-fgts-motivo-conta"), |tx| {
                super::update_account(tx, &p)
            })?;
            to_value(
                super::list(ctx.conn(), today)?
                    .into_iter()
                    .find(|a| a.account_id == p.account_id),
            )
        }
        _ => Err(Error::UnknownCommand(format!("fgts.{action}"))),
    }
}
