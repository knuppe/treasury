use serde::Deserialize;
use serde_json::Value;

use super::*;
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const READ_ONLY: &[&str] = &[
    "list",
    "get",
    "summary",
    "history",
    "iptu_list",
    "iptu_matches",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct GetArgs {
    id: String,
    #[serde(flatten)]
    read: PhysicalReadArgs,
}

#[derive(Deserialize)]
struct HistoryArgs {
    id: String,
    #[serde(default)]
    months: Option<u32>,
}

#[derive(Deserialize)]
struct LoanLinkArgs {
    asset_id: String,
    loan_id: String,
}

#[derive(Deserialize)]
struct AssetIdArg {
    asset_id: String,
}

#[derive(Deserialize)]
struct InstallmentArg {
    installment_id: String,
}

/// O carnê da parcela, depois de marcar ou desfazer o pagamento.
fn iptu_of_installment(ctx: &Ctx<'_>, installment_id: &str) -> Result<Value> {
    let bill: String = ctx.conn().query_row(
        "SELECT bill_id FROM property_tax_installment WHERE id = ?1",
        [installment_id],
        |r| r.get(0),
    )?;
    to_value(super::iptu::get(ctx.conn(), &bill, today_local())?)
}

fn get(ctx: &Ctx<'_>, id: &str) -> Result<Value> {
    to_value(view(ctx.conn(), id, &PhysicalReadArgs::default())?)
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => to_value(list(ctx.conn(), &parse_args::<PhysicalReadArgs>(args)?)?),
        "get" => {
            let a: GetArgs = parse_args(args)?;
            to_value(view(ctx.conn(), &a.id, &a.read)?)
        }
        "summary" => to_value(summary(ctx.conn(), &parse_args::<PhysicalReadArgs>(args)?)?),
        "history" => {
            let a: HistoryArgs = parse_args(args)?;
            to_value(history(ctx.conn(), &a.id, a.months)?)
        }
        "create" => {
            let i: PhysicalAssetInput = parse_args(args)?;
            let reason = t_args(
                "nucleo-carteira-bem-motivo-cadastrado",
                &i18n::args(&[("nome", i.name.trim().to_string().into())]),
            );
            let r = ctx.write(&reason, |tx| create(tx, &i))?;
            get(ctx, &r.value)
        }
        "update" => {
            let p: PhysicalAssetPatch = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-bem-motivo-alterado"), |tx| {
                update(tx, &p)
            })?;
            get(ctx, &p.id)
        }
        "delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-bem-motivo-removido"), |tx| {
                delete(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "add_improvement" => {
            let i: PhysicalImprovementInput = parse_args(args)?;
            let reason = t_args(
                "nucleo-carteira-bem-motivo-benfeitoria",
                &i18n::args(&[("descricao", i.description.trim().to_string().into())]),
            );
            ctx.write(&reason, |tx| add_improvement(tx, &i))?;
            get(ctx, &i.asset_id)
        }
        "update_improvement" => {
            let p: PhysicalImprovementPatch = parse_args(args)?;
            let r = ctx.write(
                &t("nucleo-carteira-bem-motivo-benfeitoria-alterada"),
                |tx| update_improvement(tx, &p),
            )?;
            get(ctx, &r.value)
        }
        "delete_improvement" => {
            let a: IdArg = parse_args(args)?;
            let r = ctx.write(
                &t("nucleo-carteira-bem-motivo-benfeitoria-removida"),
                |tx| delete_improvement(tx, &a.id),
            )?;
            get(ctx, &r.value)
        }
        "add_valuation" => {
            let v: PhysicalValuationInput = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-bem-motivo-avaliacao"), |tx| {
                add_valuation(tx, &v)
            })?;
            get(ctx, &v.asset_id)
        }
        "delete_valuation" => {
            let a: IdArg = parse_args(args)?;
            let r = ctx.write(&t("nucleo-carteira-bem-motivo-avaliacao-removida"), |tx| {
                delete_valuation(tx, &a.id)
            })?;
            get(ctx, &r.value)
        }
        "link_loan" => {
            let a: LoanLinkArgs = parse_args(args)?;
            ctx.write(
                &t("nucleo-carteira-bem-motivo-financiamento-ligado"),
                |tx| link_loan(tx, &a.asset_id, &a.loan_id),
            )?;
            get(ctx, &a.asset_id)
        }
        "unlink_loan" => {
            let a: LoanLinkArgs = parse_args(args)?;
            ctx.write(
                &t("nucleo-carteira-bem-motivo-financiamento-desligado"),
                |tx| unlink_loan(tx, &a.asset_id, &a.loan_id),
            )?;
            get(ctx, &a.asset_id)
        }
        "sell" => {
            let a: PhysicalSaleArgs = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-bem-motivo-venda"), |tx| sell(tx, &a))?;
            get(ctx, &a.id)
        }
        "iptu_list" => {
            let a: AssetIdArg = parse_args(args)?;
            to_value(super::iptu::list(ctx.conn(), &a.asset_id, today_local())?)
        }
        "iptu_matches" => {
            let a: InstallmentArg = parse_args(args)?;
            let (due, amount): (String, i64) = ctx
                .conn()
                .query_row(
                    "SELECT due_date, amount FROM property_tax_installment WHERE id = ?1",
                    [&a.installment_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?
                .ok_or_else(|| {
                    Error::not_found(format!(
                        "{} {}",
                        crate::i18n::t("iptu-parcela"),
                        a.installment_id
                    ))
                })?;
            to_value(super::iptu::matches(ctx.conn(), &due, amount)?)
        }
        "iptu_save" => {
            let i: super::iptu::IptuInput = parse_args(args)?;
            let reason = crate::i18n::t_args(
                "iptu-historico-salvo",
                &crate::i18n::args(&[("ano", i.year.to_string().into())]),
            );
            let r = ctx.write(&reason, |tx| super::iptu::save(tx, &i))?;
            to_value(super::iptu::get(ctx.conn(), &r.value, today_local())?)
        }
        "iptu_delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&crate::i18n::t("iptu-historico-apagado"), |tx| {
                super::iptu::delete(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "iptu_pay" => {
            let a: super::iptu::PayArgs = parse_args(args)?;
            ctx.write(&crate::i18n::t("iptu-historico-pago"), |tx| {
                super::iptu::pay(tx, &a)
            })?;
            iptu_of_installment(ctx, &a.installment_id)
        }
        "iptu_unpay" => {
            let a: InstallmentArg = parse_args(args)?;
            ctx.write(&crate::i18n::t("iptu-historico-desfeito"), |tx| {
                super::iptu::unpay(tx, &a.installment_id)
            })?;
            iptu_of_installment(ctx, &a.installment_id)
        }
        _ => Err(Error::UnknownCommand(format!("physical.{action}"))),
    }
}
