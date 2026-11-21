//! `pension.*`: plano de previdência (VGBL/PGBL), aportes por idade e simulação de resgate.

use serde::Deserialize;
use serde_json::Value;

use super::{PensionPlanInput, WithdrawalArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &[
    "get",
    "simulate",
    "statement_preview",
    "payments_preview",
    "payments_list",
];

#[derive(Deserialize)]
struct AssetArg {
    asset_id: String,
}

#[derive(Deserialize)]
struct PaymentsListArg {
    asset_id: String,
    #[serde(default)]
    date_from: Option<String>,
    #[serde(default)]
    date_to: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "get" => {
            let a: AssetArg = parse_args(args)?;
            to_value(super::view(ctx.conn(), &a.asset_id, today_local())?)
        }
        "simulate" => to_value(super::simulate_withdrawal(
            ctx.conn(),
            &parse_args::<WithdrawalArgs>(args)?,
            today_local(),
        )?),
        // Saldo informado (planos sem cota pública): vira a cota do dia.
        "set_balance" => {
            #[derive(Deserialize)]
            struct A {
                asset_id: String,
                date: String,
                balance: i64,
            }
            let a: A = parse_args(args)?;
            let price = super::set_balance(ctx.conn(), &a.asset_id, &a.date, a.balance)?;
            Ok(serde_json::json!({ "price": price }))
        }
        "set_plan" => {
            let i: PensionPlanInput = parse_args(args)?;
            let name = crate::assets::get(ctx.conn(), &i.asset_id)?.name;
            let reason = t_args(
                "nucleo-planejamento-previdencia-motivo-plano",
                &i18n::args(&[("nome", name.into())]),
            );
            ctx.write(&reason, |tx| super::set_plan(tx, &i))?;
            to_value(super::view(ctx.conn(), &i.asset_id, today_local())?)
        }
        // Extrato do plano (docs/brazil/pension-statement.md): prévia sem gravar e importação.
        "statement_preview" => {
            let a: super::statement::PensionStatementArgs = parse_args(args)?;
            to_value(super::statement::preview(ctx.conn(), &a)?)
        }
        "statement_import" => {
            let a: super::statement::PensionStatementArgs = parse_args(args)?;
            let r = ctx.write(&t("nucleo-planejamento-previdencia-motivo-extrato"), |tx| {
                super::statement::import(tx, &a, today_local())
            })?;
            to_value(r.value)
        }
        // Histórico de pagamentos (data e valor) de PGBL/VGBL: cotas pela cota da CVM.
        "payments_preview" => {
            let a: super::payments::PaymentsArgs = parse_args(args)?;
            to_value(super::payments::preview(ctx.conn(), &a)?)
        }
        // Aportes gravados (com e sem cota) e as pendências do histórico de pagamentos.
        "payments_list" => {
            let a: PaymentsListArg = parse_args(args)?;
            let all = super::payments::records(ctx.conn(), &a.asset_id)?;
            let mut rows: Vec<_> = all
                .into_iter()
                .filter(|p| a.date_from.as_deref().is_none_or(|d| p.date.as_str() >= d))
                .filter(|p| a.date_to.as_deref().is_none_or(|d| p.date.as_str() <= d))
                .collect();
            let total = rows.len();
            let pending = rows.iter().filter(|p| p.status == "pending_quote").count();
            let amount: i64 = rows.iter().map(|p| p.amount).sum();
            let first = rows.iter().map(|p| p.date.clone()).min();
            let last = rows.iter().map(|p| p.date.clone()).max();
            let offset = a.offset.unwrap_or(0).min(total);
            let limit = a.limit.unwrap_or(100).clamp(1, 500);
            let page: Vec<_> = rows.drain(offset..).take(limit).collect();
            to_value(serde_json::json!({
                "total": total,
                "pending_quote": pending,
                "amount": amount,
                "first_date": first,
                "last_date": last,
                "offset": offset,
                "payments": page,
                "issues": super::payments::saved_issues(ctx.conn(), &a.asset_id)?,
            }))
        }
        "payments_import" => {
            let a: super::payments::PaymentsArgs = parse_args(args)?;
            // O que vai na proposta: quantos aportes, com e sem cota, e as pendências.
            let p = super::payments::preview(ctx.conn(), &a)?;
            ctx.set_report(super::payments::report(&p));
            let r = ctx.write(&super::payments::reason(&p), |tx| {
                super::payments::import(tx, &a, today_local())
            })?;
            to_value(r.value)
        }
        _ => Err(Error::UnknownCommand(format!("pension.{action}"))),
    }
}
