//! `food.*` (docs/consumption.md, docs/consumption.md): resumo da tela 06 · Consumo, compras (e a
//! compra cadastrada à mão), produtos com o preço por kg/L, categorias de produto e os
//! pagamentos do cupom.

use serde::Deserialize;
use serde_json::Value;

use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &[
    "summary",
    "products",
    "product_history",
    "categories",
    "purchases",
    "purchase_candidates",
    "stores",
    "basket_inflation",
    "health_summary",
    "merge_suggestions",
    "diet_list",
    "guess",
    "profile",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct GtinArgs {
    product_id: String,
    code: String,
}

#[derive(Deserialize)]
struct MergeArgs {
    from_id: String,
    into_id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "summary" => to_value(super::summary(
            ctx.conn(),
            &parse_args::<super::SummaryArgs>(args)?,
            today_local(),
        )?),
        "products" => to_value(super::products(
            ctx.conn(),
            &parse_args::<super::ProductsArgs>(args)?,
        )?),
        "product_history" => to_value(super::product_history(
            ctx.conn(),
            &parse_args::<IdArg>(args)?.id,
        )?),
        "categories" => to_value(super::categories(ctx.conn())?),
        "profile" => to_value(super::profile::family(ctx.conn())),
        "guess" => to_value(super::classify::guess(
            ctx.conn(),
            &parse_args::<super::classify::GuessArgs>(args)?,
        )?),
        "purchases" => to_value(super::purchase::list(
            ctx.conn(),
            &parse_args::<super::purchase::PurchasesArgs>(args)?,
        )?),
        "purchase_candidates" => to_value(super::purchase::candidates(
            ctx.conn(),
            &parse_args::<super::purchase::MatchArgs>(args)?,
        )?),
        "stores" => to_value(super::purchase::stores(ctx.conn())?),
        // Código de barras digitado ou lido da foto (docs/gtin-inflation.md §3).
        "product_gtin_add" => {
            let a: GtinArgs = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-alimentacao-motivo-gtin"), |tx| {
                    super::add_gtin(tx, &a.product_id, &a.code)
                })?
                .value,
            )
        }
        "product_gtin_remove" => {
            let a: GtinArgs = parse_args(args)?;
            ctx.write(&t("nucleo-alimentacao-motivo-gtin-removido"), |tx| {
                super::remove_gtin(tx, &a.product_id, &a.code)
            })?;
            Ok(Value::Null)
        }
        // Saúde da cesta, sugestões de juntar e alergias (docs/gtin-inflation.md §4 e §7).
        "health_summary" => to_value(super::health::summary(ctx.conn(), today_local())?),
        "merge_suggestions" => to_value(super::line::merge_suggestions(ctx.conn(), 20)?),
        "diet_list" => to_value(super::health::diet_list(ctx.conn())?),
        "diet_set" => {
            let a: super::health::DietInput = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-alimentacao-motivo-restricoes"), |tx| {
                    super::health::diet_set(tx, &a)
                })?
                .value,
            )
        }
        // Índice da sua cesta (docs/gtin-inflation.md §5).
        "basket_inflation" => to_value(super::basket::basket(
            ctx.conn(),
            &parse_args::<super::basket::BasketArgs>(args)?,
            today_local(),
        )?),
        "purchase_save" => {
            let p: super::purchase::PurchaseInput = parse_args(args)?;
            let key = if p.id.is_some() {
                "nucleo-alimentacao-motivo-compra-editada"
            } else {
                "nucleo-alimentacao-motivo-compra"
            };
            let r = ctx.write(&t(key), |tx| super::purchase::save(tx, &p, today_local()))?;
            to_value(crate::receipts::get(ctx.conn(), &r.value)?)
        }
        "purchase_delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-alimentacao-motivo-compra-apagada"), |tx| {
                super::purchase::delete(tx, &a.id, today_local())
            })?;
            Ok(Value::Null)
        }
        "relink" => {
            let r = ctx.write(&t("nucleo-alimentacao-motivo-ligar"), |tx| {
                super::link_items(tx, None)
            })?;
            to_value(serde_json::json!({ "linked": r.value }))
        }
        "product_update" => {
            let p: super::ProductPatch = parse_args(args)?;
            ctx.write(&t("nucleo-alimentacao-motivo-produto"), |tx| {
                super::update_product(tx, &p)
            })?;
            to_value(super::product_history(ctx.conn(), &p.id)?.product)
        }
        "product_merge" => {
            let a: MergeArgs = parse_args(args)?;
            ctx.write(&t("nucleo-alimentacao-motivo-juntar"), |tx| {
                super::merge_products(tx, &a.from_id, &a.into_id)
            })?;
            to_value(super::product_history(ctx.conn(), &a.into_id)?.product)
        }
        "category_create" => {
            let i: super::CategoryInput = parse_args(args)?;
            let r = ctx.write(&t("nucleo-alimentacao-motivo-categoria"), |tx| {
                super::create_category(tx, &i)
            })?;
            to_value(
                super::categories(ctx.conn())?
                    .into_iter()
                    .find(|c| c.id == r.value),
            )
        }
        "category_update" => {
            let p: super::CategoryPatch = parse_args(args)?;
            ctx.write(&t("nucleo-alimentacao-motivo-categoria"), |tx| {
                super::update_category(tx, &p)
            })?;
            to_value(super::categories(ctx.conn())?)
        }
        "category_delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-alimentacao-motivo-categoria-apagada"), |tx| {
                super::delete_category(tx, &a.id)
            })?;
            to_value(super::categories(ctx.conn())?)
        }
        "set_payments" => {
            let a: super::PaymentsArgs = parse_args(args)?;
            ctx.write(&t("nucleo-alimentacao-motivo-pagamentos"), |tx| {
                super::set_payments(tx, &a)
            })?;
            to_value(crate::receipts::get(ctx.conn(), &a.receipt_id)?)
        }
        _ => Err(Error::UnknownCommand(format!("food.{action}"))),
    }
}
