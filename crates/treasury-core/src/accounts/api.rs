use serde::Deserialize;
use serde_json::Value;

use super::{AccountInput, AccountListArgs, AccountPatch, BalanceHistoryArgs, ReconcileArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::time::today_local;

/// Motivo do histórico com o nome da conta.
fn with_name(key: &str, name: &str) -> String {
    t_args(
        key,
        &i18n::args(&[("nome", name.trim().to_string().into())]),
    )
}

pub const READ_ONLY: &[&str] = &[
    "list",
    "get",
    "balance_history",
    "match_ofx",
    "list_syncs",
    "sync_hints",
    "capital_share",
    "staleness_limits",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let today = today_local();
    match action {
        "list" => {
            let a: AccountListArgs = parse_args(args)?;
            to_value(super::list(
                ctx.conn(),
                a.include_archived.unwrap_or(false),
                today,
            )?)
        }
        "get" => to_value(super::get(
            ctx.conn(),
            &parse_args::<IdArg>(args)?.id,
            today,
        )?),
        "create" => {
            let input: AccountInput = parse_args(args)?;
            let reason = with_name("nucleo-contas-historico-criada", &input.name);
            let id = ctx
                .write(&reason, |tx| super::create(tx, &input, today))?
                .value;
            to_value(super::get(ctx.conn(), &id, today)?)
        }
        "update" => {
            let p: AccountPatch = parse_args(args)?;
            let reason = with_name(
                "nucleo-contas-historico-alterada",
                &super::account_name(ctx.conn(), &p.id)?,
            );
            ctx.write(&reason, |tx| super::update(tx, &p, today))?;
            to_value(super::get(ctx.conn(), &p.id, today)?)
        }
        "delete" => {
            let a: IdArg = parse_args(args)?;
            let reason = with_name(
                "nucleo-contas-historico-apagada",
                &super::account_name(ctx.conn(), &a.id)?,
            );
            ctx.write(&reason, |tx| super::delete(tx, &a.id))?;
            Ok(Value::Null)
        }
        "match_ofx" => to_value(super::match_ofx(ctx.conn(), &parse_args(args)?, today)?),
        "balance_history" => to_value(super::balance_history(
            ctx.conn(),
            &parse_args::<BalanceHistoryArgs>(args)?,
        )?),
        "reconcile" => {
            let a: ReconcileArgs = parse_args(args)?;
            let mut res = super::reconcile_check(ctx.conn(), &a)?;
            if a.apply.unwrap_or(false) && res.difference == 0 {
                let reason = super::reconcile_reason(ctx.conn(), &a)?;
                let out = ctx.write(&reason, |tx| {
                    super::mark_reconciled(tx, &a.account_id, &a.statement_date)
                })?;
                res.marked_reconciled = out.value as i64;
                res.change_set_id = out.change_set_id;
            }
            to_value(res)
        }
        "list_syncs" => to_value(super::sync::list(ctx.conn(), &parse_args(args)?)?),
        "staleness_limits" => to_value(super::freshness::settings(ctx.conn())?),
        "set_staleness_limits" => {
            let a: super::freshness::SetStalenessArgs = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-contas-historico-limites-atualizacao"), |tx| {
                    super::freshness::set(tx, &a)
                })?
                .value,
            )
        }
        "capital_share" => {
            #[derive(Deserialize)]
            struct A {
                account_id: String,
            }
            let a: A = parse_args(args)?;
            to_value(super::capital::view(ctx.conn(), &a.account_id)?)
        }
        "capital_share_set" => {
            let a: super::capital::CapitalShareInput = parse_args(args)?;
            let reason = with_name(
                "nucleo-contas-historico-cotas-capital",
                &super::account_name(ctx.conn(), &a.account_id)?,
            );
            ctx.write(&reason, |tx| super::capital::set(tx, &a))?;
            to_value(super::capital::view(ctx.conn(), &a.account_id)?)
        }
        "capital_share_remove" => {
            #[derive(Deserialize)]
            struct A {
                account_id: String,
            }
            let a: A = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-cotas-removidas"), |tx| {
                super::capital::remove(tx, &a.account_id)
            })?;
            Ok(Value::Null)
        }
        "capital_share_add_balance" => {
            let a: super::capital::CapitalBalanceInput = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-cotas-saldo"), |tx| {
                super::capital::add_balance(tx, &a)
            })?;
            to_value(super::capital::view(ctx.conn(), &a.account_id)?)
        }
        "capital_share_mark_contributions" => {
            #[derive(Deserialize)]
            struct A {
                account_id: String,
            }
            let a: A = parse_args(args)?;
            let n = ctx
                .write(&t("nucleo-contas-historico-cotas-aportes"), |tx| {
                    super::capital::mark_contributions(tx, &a.account_id)
                })?
                .value;
            to_value(serde_json::json!({ "updated": n }))
        }
        "capital_share_delete_balance" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-cotas-saldo-removido"), |tx| {
                super::capital::delete_balance(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "sync_hints" => {
            let a: IdArg = parse_args(args)?;
            to_value(super::sync::hints(ctx.conn(), &a.id)?)
        }
        "sync_balance" => {
            let a: super::sync::SyncBalanceArgs = parse_args(args)?;
            let reason = t_args(
                "nucleo-contas-historico-sincronia",
                &i18n::args(&[
                    (
                        "nome",
                        super::account_name(ctx.conn(), &a.account_id)?.into(),
                    ),
                    ("data", crate::transactions::util::date_text(&a.date).into()),
                ]),
            );
            let id = ctx
                .write(&reason, |tx| super::sync::sync_balance(tx, &a, today))?
                .value;
            to_value(super::sync::get(ctx.conn(), &id)?)
        }
        "resolve_sync" => {
            let a: super::sync::ResolveSyncArgs = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-sincronia-motivo"), |tx| {
                super::sync::resolve(tx, &a)
            })?;
            to_value(super::sync::get(ctx.conn(), &a.id)?)
        }
        _ => Err(Error::UnknownCommand(format!("accounts.{action}"))),
    }
}
