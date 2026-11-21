use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::PriceRow;
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::{fmt_date, parse_date};

pub const READ_ONLY: &[&str] = &[
    "prices",
    "data_sources",
    "exchanges",
    "quote",
    "quotes",
    "intraday",
];

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PricePoint {
    pub date: String,
    pub close: String,
    pub source: String,
}

#[derive(Deserialize)]
struct PricesArgs {
    asset_id: String,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ManualPriceArgs {
    pub asset_id: String,
    pub date: String,
    /// Decimal em texto.
    pub close: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataSources {
    /// Chave da FRED cadastrada (o valor nunca sai do núcleo).
    pub fred_key_set: bool,
    /// Token da brapi.dev cadastrado (opcional; o valor nunca sai do núcleo).
    pub brapi_token_set: bool,
    /// Plano e consumo da brapi (lidos da própria API), quando há token.
    pub brapi: Option<BrapiUsageView>,
    /// Chave da Tiingo cadastrada (opcional; o valor nunca sai do núcleo).
    pub tiingo_token_set: bool,
    /// Chave da EODHD cadastrada (opcional; o valor nunca sai do núcleo).
    pub eodhd_token_set: bool,
    pub last_sync_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BrapiUsageView {
    /// `free`, `startup`, `pro` ou `None` (ainda não lido).
    pub plan: Option<String>,
    #[ts(type = "number")]
    pub quota: i64,
    #[ts(type = "number")]
    pub used: i64,
    /// Porcentagem da cota que o app se permite usar.
    #[ts(type = "number")]
    pub limit_pct: i64,
    /// Indicadores e proventos (conferência do score) só nos planos pagos.
    pub paid: bool,
    pub checked_at: Option<String>,
    pub window_end: Option<String>,
}

pub fn brapi_view(conn: &rusqlite::Connection) -> crate::error::Result<BrapiUsageView> {
    let s = crate::research::brapi::state(conn)?;
    Ok(BrapiUsageView {
        plan: s.plan.clone(),
        quota: s.quota(),
        used: s.used_now(),
        limit_pct: crate::research::brapi::limit_pct(conn)?,
        paid: s.paid(),
        checked_at: s.checked_at.clone(),
        window_end: s.window_end.clone(),
    })
}

#[derive(Deserialize)]
struct PctArg {
    pct: i64,
}

#[derive(Deserialize)]
struct KeyArg {
    key: String,
}

#[derive(Deserialize)]
struct AssetArg {
    asset_id: String,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct QuotesArgs {
    /// Só estes ativos (padrão: todos com cotação intradiária).
    #[ts(optional)]
    pub asset_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct IntradayArgs {
    pub asset_id: String,
    /// "1d" (padrão) ou "5d".
    #[ts(optional)]
    pub range: Option<String>,
}

#[derive(Deserialize)]
struct CodeArg {
    code: String,
}

/// Grava (ou apaga, com texto vazio) a chave de uma fonte de cotação. Só a pessoa, pela
/// interface; o valor fica cifrado no cofre e nunca volta.
fn set_source_key(ctx: &mut Ctx<'_>, args: Value, setting: &str) -> Result<Value> {
    if *ctx.actor != crate::api::Actor::User {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-ativos-erro-chaves-so-interface",
        )));
    }
    let a: KeyArg = parse_args(args)?;
    let k = a.key.trim();
    if k.is_empty() {
        crate::settings::delete(ctx.conn(), setting)?;
    } else if k.len() < 10
        || k.len() > 200
        || !k
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
    {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-chave-fonte",
        )));
    } else {
        crate::settings::set(ctx.conn(), setting, &k)?;
    }
    Ok(Value::Null)
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "data_sources" => {
            let key: Option<String> =
                crate::settings::get(ctx.conn(), crate::indices::sync::FRED_KEY_SETTING)?;
            to_value(DataSources {
                fred_key_set: key.is_some_and(|k| !k.is_empty()),
                brapi_token_set: crate::research::brapi::token(ctx.conn())?.is_some(),
                brapi: if crate::research::brapi::token(ctx.conn())?.is_some() {
                    Some(brapi_view(ctx.conn())?)
                } else {
                    None
                },
                tiingo_token_set: super::tiingo::token(ctx.conn())?.is_some(),
                eodhd_token_set: super::eodhd::token(ctx.conn())?.is_some(),
                last_sync_at: crate::settings::get(
                    ctx.conn(),
                    crate::indices::sync::LAST_SYNC_SETTING,
                )?,
            })
        }
        "set_tiingo_token" => set_source_key(ctx, args, super::tiingo::TOKEN_SETTING),
        "set_eodhd_token" => set_source_key(ctx, args, super::eodhd::TOKEN_SETTING),
        "set_fred_key" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-ativos-erro-chaves-so-interface",
                )));
            }
            let a: KeyArg = parse_args(args)?;
            let k = a.key.trim();
            if k.is_empty() {
                crate::settings::delete(ctx.conn(), crate::indices::sync::FRED_KEY_SETTING)?;
            } else if k.len() != 32 || !k.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-ativos-erro-chave-fred",
                )));
            } else {
                crate::settings::set(ctx.conn(), crate::indices::sync::FRED_KEY_SETTING, &k)?;
            }
            Ok(Value::Null)
        }
        "set_brapi_token" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-ativos-erro-chaves-so-interface",
                )));
            }
            let a: KeyArg = parse_args(args)?;
            let k = a.key.trim();
            if k.is_empty() {
                crate::settings::delete(ctx.conn(), crate::research::brapi::TOKEN_SETTING)?;
            } else if k.len() < 10
                || k.len() > 200
                || !k
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
            {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-ativos-erro-token-brapi",
                )));
            } else {
                crate::settings::set(ctx.conn(), crate::research::brapi::TOKEN_SETTING, &k)?;
            }
            // Outro token (ou nenhum): o plano e o consumo serão lidos de novo.
            crate::settings::delete(ctx.conn(), crate::research::brapi::STATE_SETTING)?;
            Ok(Value::Null)
        }
        "set_brapi_limit" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-ativos-erro-chaves-so-interface",
                )));
            }
            let a: PctArg = parse_args(args)?;
            if !(10..=100).contains(&a.pct) {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-ativos-erro-limite-brapi",
                )));
            }
            crate::settings::set(ctx.conn(), crate::research::brapi::LIMIT_SETTING, &a.pct)?;
            to_value(brapi_view(ctx.conn())?)
        }
        "prices" => {
            let a: PricesArgs = parse_args(args)?;
            let to =
                a.to.as_deref()
                    .map(parse_date)
                    .transpose()?
                    .unwrap_or_else(crate::time::today_local);
            let from = a
                .from
                .as_deref()
                .map(parse_date)
                .transpose()?
                .unwrap_or_else(|| crate::time::add_months(to, -12));
            let mut stmt = ctx
                .conn()
                .prepare("SELECT date, close, source FROM price WHERE asset_id = ?1 AND date BETWEEN ?2 AND ?3 ORDER BY date")?;
            let rows = stmt
                .query_map(
                    rusqlite::params![a.asset_id, fmt_date(from), fmt_date(to)],
                    |r| {
                        Ok(PricePoint {
                            date: r.get(0)?,
                            close: r.get(1)?,
                            source: r.get(2)?,
                        })
                    },
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            to_value(rows)
        }
        "set_price" => {
            let a: ManualPriceArgs = parse_args(args)?;
            let close: rust_decimal::Decimal =
                a.close.trim().replace(',', ".").parse().map_err(|_| {
                    Error::invalid(crate::i18n::t_args(
                        "nucleo-ativos-erro-preco-invalido",
                        &crate::i18n::args(&[("preco", a.close.clone().into())]),
                    ))
                })?;
            if close <= rust_decimal::Decimal::ZERO {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-ativos-erro-preco-positivo",
                )));
            }
            crate::assets::get(ctx.conn(), &a.asset_id)?;
            let row = PriceRow {
                date: parse_date(&a.date)?,
                open: None,
                high: None,
                low: None,
                close,
                adj_close: None,
                volume: None,
            };
            super::upsert_prices(ctx.conn(), &a.asset_id, &[row], "manual")?;
            Ok(Value::Null)
        }
        "exchanges" => to_value(super::exchange::list(ctx.conn())?),
        "exchange_save" => {
            let input: super::exchange::ExchangeInput = parse_args(args)?;
            let label = input.code.trim().to_uppercase();
            let reason = crate::i18n::t_args(
                "nucleo-ativos-motivo-bolsa-salva",
                &crate::i18n::args(&[("bolsa", label.into())]),
            );
            let r = ctx.write(&reason, |tx| super::exchange::save(tx, &input))?;
            to_value(super::exchange::get(ctx.conn(), &r.value)?)
        }
        "exchange_delete" => {
            let a: CodeArg = parse_args(args)?;
            let reason = crate::i18n::t_args(
                "nucleo-ativos-motivo-bolsa-removida",
                &crate::i18n::args(&[("bolsa", a.code.to_uppercase().into())]),
            );
            ctx.write(&reason, |tx| super::exchange::delete(tx, &a.code))?;
            Ok(Value::Null)
        }
        "quote" => {
            let a: AssetArg = parse_args(args)?;
            to_value(super::intraday::quote(ctx.conn(), &a.asset_id)?)
        }
        "quotes" => {
            let a: QuotesArgs = parse_args(args)?;
            to_value(super::intraday::quotes(
                ctx.conn(),
                &a.asset_ids.unwrap_or_default(),
            )?)
        }
        "intraday" => {
            let a: IntradayArgs = parse_args(args)?;
            to_value(super::intraday::series(
                ctx.conn(),
                &a.asset_id,
                a.range.as_deref().unwrap_or("1d"),
                chrono::Utc::now(),
            )?)
        }
        _ => Err(Error::UnknownCommand(format!("market.{action}"))),
    }
}
