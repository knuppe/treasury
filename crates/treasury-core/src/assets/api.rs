use serde::Deserialize;
use serde_json::Value;

use super::{AssetFilter, AssetInput, AssetPatch};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};

pub const READ_ONLY: &[&str] = &[
    "list",
    "get",
    "find",
    "data_gaps",
    "field_sources",
    "enrich_lookup",
    "study_list",
    "study_get",
    "study_state",
    "study_package",
    "labels_list",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct AssetIdArg {
    asset_id: String,
}

#[derive(Deserialize)]
struct TickerArg {
    ticker: String,
}

/// Escritas que só a interface faz (o aviso do estudo, o contexto e apagar estudos).
fn user_only(ctx: &Ctx<'_>) -> Result<()> {
    if *ctx.actor != crate::api::Actor::User {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-ativos-estudo-erro-so-interface",
        )));
    }
    Ok(())
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => to_value(super::list(ctx.conn(), &parse_args::<AssetFilter>(args)?)?),
        "get" => to_value(super::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "find" => to_value(super::find_by_ticker(
            ctx.conn(),
            &parse_args::<TickerArg>(args)?.ticker,
        )?),
        "create" => {
            let input: AssetInput = parse_args(args)?;
            let label = input.ticker.clone().unwrap_or_else(|| input.name.clone());
            let reason = crate::i18n::t_args(
                "nucleo-ativos-motivo-cadastrado",
                &crate::i18n::args(&[("ativo", label.into())]),
            );
            let r = ctx.write(&reason, |tx| super::create(tx, &input))?;
            to_value(super::get(ctx.conn(), &r.value)?)
        }
        // Etiquetas da família (grupos, fontes de meta e de concentração).
        "labels_list" => to_value(super::labels::list(ctx.conn())?),
        "labels_set" => {
            let a: super::labels::SetAssetLabelsArgs = parse_args(args)?;
            let r = ctx.write(&crate::i18n::t("nucleo-ativos-motivo-etiquetas"), |tx| {
                super::labels::set_for_asset(tx, &a)
            })?;
            to_value(r.value)
        }
        "label_update" => {
            let a: super::labels::UpdateAssetLabelArgs = parse_args(args)?;
            let r = ctx.write(&crate::i18n::t("nucleo-ativos-motivo-etiquetas"), |tx| {
                super::labels::update(tx, &a)
            })?;
            to_value(r.value)
        }
        "update" => {
            let p: AssetPatch = parse_args(args)?;
            let id = p.id.clone();
            ctx.write(&crate::i18n::t("nucleo-ativos-motivo-atualizado"), |tx| {
                super::update(tx, &p)
            })?;
            to_value(super::get(ctx.conn(), &id)?)
        }
        "data_gaps" => to_value(super::enrich::data_gaps(
            ctx.conn(),
            &parse_args::<super::enrich::DataGapsArgs>(args)?,
        )?),
        "field_sources" => to_value(super::enrich::field_sources(
            ctx.conn(),
            &parse_args::<AssetIdArg>(args)?.asset_id,
        )?),
        "enrich_apply" => {
            let a: super::enrich::EnrichApplyArgs = parse_args(args)?;
            let r = ctx.write(&crate::i18n::t("nucleo-ativos-motivo-dados"), |tx| {
                super::enrich::apply(tx, &a.items)
            })?;
            to_value(r.value)
        }
        "field_lock" => {
            let a: super::enrich::FieldLockArgs = parse_args(args)?;
            let key = if a.locked {
                "nucleo-ativos-motivo-campo-travado"
            } else {
                "nucleo-ativos-motivo-campo-destravado"
            };
            ctx.write(&crate::i18n::t(key), |tx| super::enrich::set_lock(tx, &a))?;
            Ok(Value::Null)
        }
        "study_list" => to_value(super::study::list(
            ctx.conn(),
            &parse_args::<AssetIdArg>(args)?.asset_id,
        )?),
        "study_get" => to_value(super::study::get(
            ctx.conn(),
            &parse_args::<IdArg>(args)?.id,
        )?),
        // Pedido de estudo pelo agente: vira proposta; aprovado, o app dispara o estudo.
        "study_request" => {
            let a: super::study::StudyRequestArgs = parse_args(args)?;
            let ticker = super::get(ctx.conn(), &a.asset_id)
                .map(|x| x.ticker.unwrap_or(x.name))
                .unwrap_or_default();
            let reason = crate::i18n::t_args(
                "nucleo-ativos-motivo-estudo-pedido",
                &crate::i18n::args(&[("ativo", ticker.into())]),
            );
            to_value(
                ctx.write(&reason, |tx| super::study::request(tx, &a))?
                    .value,
            )
        }
        "study_context_set" => {
            user_only(ctx)?;
            let a: super::study::StudyContextArgs = parse_args(args)?;
            ctx.write(
                &crate::i18n::t("nucleo-ativos-motivo-contexto-estudo"),
                |tx| super::study::set_context(tx, &a),
            )?;
            Ok(Value::Null)
        }
        "study_accept" => {
            user_only(ctx)?;
            super::study::accept(ctx.conn())?;
            Ok(Value::Null)
        }
        "study_delete" => {
            user_only(ctx)?;
            let a: IdArg = parse_args(args)?;
            ctx.write(
                &crate::i18n::t("nucleo-ativos-motivo-estudo-apagado"),
                |tx| super::study::delete(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        "delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&crate::i18n::t("nucleo-ativos-motivo-removido"), |tx| {
                super::delete(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("assets.{action}"))),
    }
}
