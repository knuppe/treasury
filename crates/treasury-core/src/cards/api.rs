use serde::Deserialize;
use serde_json::Value;

use super::plastics::{CardPlasticInput, CardPlasticPatch};
use super::{
    FutureCommitmentsArgs, InstallmentInput, ListStatementsArgs, MoveToStatementArgs,
    PayStatementArgs, PayStatementResult,
};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::time::today_local;
use crate::transactions::api::{short, with_desc, with_n};
use crate::transactions::{BulkResult, TransferResult};

pub const READ_ONLY: &[&str] = &[
    "list_statements",
    "get_statement",
    "list_installment_plans",
    "future_commitments",
    "list_plastics",
    "overview",
    "rewards",
    "loyalty_list",
    "loyalty_entries",
    "value",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct InformedTotalArgs {
    id: String,
    informed_total: Option<i64>,
}

#[derive(Deserialize)]
struct ProgramArg {
    program_id: String,
}

#[derive(Deserialize)]
struct AccountArg {
    account_id: String,
}

#[derive(Deserialize, Default)]
struct PlansArgs {
    account_id: Option<String>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let today = today_local();
    match action {
        "list_statements" => to_value(super::list_statements(
            ctx.conn(),
            &parse_args::<ListStatementsArgs>(args)?,
            today,
        )?),
        "get_statement" => to_value(super::get_statement(
            ctx.conn(),
            &parse_args::<IdArg>(args)?.id,
            today,
        )?),
        "pay_statement" => {
            let a: PayStatementArgs = parse_args(args)?;
            let st = super::statement_view(ctx.conn(), &a.statement_id, today)?;
            let reason = t_args(
                "nucleo-contas-historico-pagamento-fatura",
                &i18n::args(&[
                    ("cartao", st.account_name.clone().into()),
                    (
                        "vencimento",
                        crate::transactions::util::date_text(&st.due_date).into(),
                    ),
                ]),
            );
            let ids = ctx
                .write(&reason, |tx| super::pay_statement(tx, &a, today))?
                .value;
            to_value(PayStatementResult {
                statement: super::statement_view(ctx.conn(), &a.statement_id, today)?,
                transfer: TransferResult {
                    transfer_id: ids.transfer_id,
                    from: crate::transactions::get(ctx.conn(), &ids.from_id)?,
                    to: crate::transactions::get(ctx.conn(), &ids.to_id)?,
                },
            })
        }
        "set_informed_total" => {
            let a: InformedTotalArgs = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-total-informado"), |tx| {
                super::set_informed_total(tx, &a.id, a.informed_total)
            })?;
            to_value(super::statement_view(ctx.conn(), &a.id, today)?)
        }
        "move_to_statement" => {
            let a: MoveToStatementArgs = parse_args(args)?;
            let reason = with_n("nucleo-contas-historico-movidos-fatura", a.txn_ids.len());
            let out = ctx.write(&reason, |tx| super::move_to_statement(tx, &a, today))?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "create_installments" => {
            let input: InstallmentInput = parse_args(args)?;
            let reason = t_args(
                "nucleo-contas-historico-compra-parcelada",
                &i18n::args(&[
                    ("n", input.installments.into()),
                    ("descricao", short(&input.description).into()),
                ]),
            );
            let id = ctx
                .write(&reason, |tx| super::create_installments(tx, &input, today))?
                .value;
            to_value(super::get_installment_plan(ctx.conn(), &id, today)?)
        }
        "list_installment_plans" => {
            let a: PlansArgs = parse_args(args)?;
            to_value(super::list_installment_plans(
                ctx.conn(),
                a.account_id.as_deref(),
                today,
            )?)
        }
        "delete_installment_plan" => {
            let a: IdArg = parse_args(args)?;
            let plan = super::get_installment_plan(ctx.conn(), &a.id, today)?;
            let reason = with_desc(
                "nucleo-contas-historico-parcelamento-apagado",
                &plan.description,
            );
            let out = ctx.write(&reason, |tx| {
                super::delete_installment_plan(tx, &a.id, today)
            })?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "future_commitments" => {
            let a: FutureCommitmentsArgs = parse_args(args)?;
            to_value(super::future_commitments(
                ctx.conn(),
                a.account_id.as_deref(),
                a.months.unwrap_or(12),
                today,
            )?)
        }
        "list_plastics" => {
            let a: PlansArgs = parse_args(args)?;
            to_value(super::plastics::list(
                ctx.conn(),
                a.account_id.as_deref(),
                today,
            )?)
        }
        "create_plastic" => {
            let input: CardPlasticInput = parse_args(args)?;
            let name = crate::accounts::account_name(ctx.conn(), &input.account_id)?;
            let reason = t_args(
                "nucleo-contas-historico-plastico-criado",
                &i18n::args(&[("cartao", name.into())]),
            );
            let id = ctx
                .write(&reason, |tx| super::plastics::create(tx, &input))?
                .value;
            to_value(super::plastics::get(ctx.conn(), &id, today)?)
        }
        "update_plastic" => {
            let patch: CardPlasticPatch = parse_args(args)?;
            let cur = super::plastics::get(ctx.conn(), &patch.id, today)?;
            let reason = t_args(
                "nucleo-contas-historico-plastico-alterado",
                &i18n::args(&[("plastico", super::plastics::short_label(&cur).into())]),
            );
            ctx.write(&reason, |tx| super::plastics::update(tx, &patch, today))?;
            to_value(super::plastics::get(ctx.conn(), &patch.id, today)?)
        }
        "delete_plastic" => {
            let a: IdArg = parse_args(args)?;
            let cur = super::plastics::get(ctx.conn(), &a.id, today)?;
            let reason = t_args(
                "nucleo-contas-historico-plastico-apagado",
                &i18n::args(&[("plastico", super::plastics::short_label(&cur).into())]),
            );
            let out = ctx.write(&reason, |tx| super::plastics::delete(tx, &a.id, today))?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "overview" => {
            let a: AccountArg = parse_args(args)?;
            to_value(super::limits::overview(ctx.conn(), &a.account_id, today)?)
        }
        "rewards" => {
            let a: AccountArg = parse_args(args)?;
            to_value(super::rewards::view(ctx.conn(), &a.account_id, today)?)
        }
        "set_reward_program" => {
            let input: super::rewards::RewardProgramInput = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-recompensa"), |tx| {
                super::rewards::set(tx, &input, today)
            })?;
            to_value(super::rewards::view(ctx.conn(), &input.account_id, today)?)
        }
        "end_reward_program" => {
            let a: AccountArg = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-recompensa-fim"), |tx| {
                super::rewards::end(tx, &a.account_id, today)
            })?;
            to_value(super::rewards::view(ctx.conn(), &a.account_id, today)?)
        }
        "loyalty_list" => to_value(super::loyalty::list(ctx.conn(), today)?),
        "value" => to_value(super::value::report(ctx.conn(), today)?),
        "loyalty_entries" => {
            let a: ProgramArg = parse_args(args)?;
            super::loyalty::require_program(ctx.conn(), &a.program_id)?;
            to_value(super::loyalty::entries(ctx.conn(), &a.program_id)?)
        }
        "loyalty_create" => {
            let input: super::loyalty::LoyaltyProgramInput = parse_args(args)?;
            let reason = with_desc("nucleo-pontos-historico-programa", &input.name);
            let id = ctx
                .write(&reason, |tx| super::loyalty::create_program(tx, &input))?
                .value;
            to_value(super::loyalty::get(ctx.conn(), &id, today)?)
        }
        "loyalty_update" => {
            let p: super::loyalty::LoyaltyProgramPatch = parse_args(args)?;
            let cur = super::loyalty::get(ctx.conn(), &p.id, today)?;
            let reason = with_desc("nucleo-pontos-historico-programa-alterado", &cur.name);
            ctx.write(&reason, |tx| super::loyalty::update_program(tx, &p))?;
            to_value(super::loyalty::get(ctx.conn(), &p.id, today)?)
        }
        "loyalty_delete" => {
            let a: IdArg = parse_args(args)?;
            let cur = super::loyalty::get(ctx.conn(), &a.id, today)?;
            let reason = with_desc("nucleo-pontos-historico-programa-apagado", &cur.name);
            ctx.write(&reason, |tx| super::loyalty::delete_program(tx, &a.id))?;
            Ok(Value::Null)
        }
        "loyalty_add_entry" => {
            let input: super::loyalty::LoyaltyEntryInput = parse_args(args)?;
            let cur = super::loyalty::get(ctx.conn(), &input.program_id, today)?;
            let reason = with_desc("nucleo-pontos-historico-movimento", &cur.name);
            ctx.write(&reason, |tx| super::loyalty::add_entry(tx, &input))?;
            to_value(super::loyalty::get(ctx.conn(), &input.program_id, today)?)
        }
        "loyalty_transfer" => {
            let input: super::loyalty::LoyaltyTransferInput = parse_args(args)?;
            let from = super::loyalty::get(ctx.conn(), &input.from_program_id, today)?;
            let reason = with_desc("nucleo-pontos-historico-transferencia", &from.name);
            ctx.write(&reason, |tx| super::loyalty::transfer(tx, &input))?;
            to_value(super::loyalty::list(ctx.conn(), today)?)
        }
        "loyalty_delete_entry" => {
            let a: IdArg = parse_args(args)?;
            let out = ctx.write(&t("nucleo-pontos-historico-movimento-apagado"), |tx| {
                super::loyalty::delete_entry(tx, &a.id)
            })?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "update_statement" => {
            let p: super::StatementPatch = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-fatura-alterada"), |tx| {
                super::update_statement(tx, &p, today)
            })?;
            to_value(super::statement_view(ctx.conn(), &p.id, today)?)
        }
        "add_limit_change" => {
            let input: super::limits::LimitChangeInput = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-limite"), |tx| {
                super::limits::add_change(tx, &input)
            })?;
            to_value(super::limits::overview(
                ctx.conn(),
                &input.account_id,
                today,
            )?)
        }
        "delete_limit_change" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-limite"), |tx| {
                super::limits::delete_change(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "set_annual_fee" => {
            let input: super::annual_fee::AnnualFeeInput = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-anuidade"), |tx| {
                super::annual_fee::set(tx, &input, today)
            })?;
            to_value(super::limits::overview(
                ctx.conn(),
                &input.account_id,
                today,
            )?)
        }
        "end_annual_fee" => {
            let a: AccountArg = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-anuidade-fim"), |tx| {
                super::annual_fee::end(tx, &a.account_id, today)
            })?;
            to_value(super::limits::overview(ctx.conn(), &a.account_id, today)?)
        }
        _ => Err(Error::UnknownCommand(format!("cards.{action}"))),
    }
}
