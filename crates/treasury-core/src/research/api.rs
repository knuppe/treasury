//! Comandos `research.*`. A coleta (`research.sync`) fica em `api::system`, como as outras
//! coletas, para liberar o banco durante a rede.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::data::{self, Cache, SubjectRef};
use super::indicators::{self, BazinParams, FairPriceView};
use super::radar::RadarArgs;
use super::screen::{RankArgs, ScreenArgs, ScreenInput, ScreenPatch};
use super::views::{CompareAssetsArgs, DocumentsArgs, HistoryArgs};
use super::{config, radar, screen, views};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &[
    "catalog",
    "status",
    "get_settings",
    "fundamentals",
    "indicators",
    "history",
    "fair_price",
    "rank",
    "screen",
    "screens.list",
    "screens.get",
    "compare",
    "documents",
    "documents_status",
    "news",
    "document_read",
    "market_overview",
    "methods",
    "method_counts",
    "method_run",
    "method_explain",
    "method_badges",
    "tags",
    "market_map",
    "observed",
    "fundamentus",
    "dividends_check",
    "fii_result",
];

/// Ativo cadastrado (`asset_id`) ou papel do universo (`ticker`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SubjectArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
}

impl SubjectArgs {
    fn subject(&self) -> SubjectRef {
        SubjectRef {
            asset_id: self.asset_id.clone(),
            ticker: self.ticker.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FairPriceArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
    /// Taxa mínima do Bazin (0.06 ou 6 = 6%; padrão dos ajustes).
    #[serde(default)]
    #[ts(optional)]
    pub bazin_rate: Option<f64>,
    /// Anos da média de dividendos (padrão dos ajustes, 1 a 10).
    #[serde(default)]
    #[ts(optional)]
    pub years: Option<u32>,
}

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

fn user_only(ctx: &Ctx<'_>) -> Result<()> {
    if *ctx.actor != Actor::User {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-pesquisa-erro-ajustes-so-interface",
        )));
    }
    Ok(())
}

fn bazin_of(ctx: &Ctx<'_>) -> Result<BazinParams> {
    Ok(config::load(ctx.conn())?.bazin())
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let today = today_local();
    match action {
        "catalog" => to_value(indicators::catalog()),
        "status" => to_value(views::status(ctx.conn())?),
        "get_settings" => to_value(config::view(ctx.conn())?),
        "set_settings" => {
            user_only(ctx)?;
            let patch: serde_json::Map<String, Value> = parse_args(args)?;
            let mut cur = serde_json::to_value(config::load(ctx.conn())?)?;
            for (k, v) in patch {
                if cur.get(&k).is_none() {
                    return Err(Error::invalid(crate::i18n::t_args(
                        "nucleo-pesquisa-erro-ajuste-desconhecido",
                        &crate::i18n::args(&[("ajuste", k.into())]),
                    )));
                }
                cur[&k] = v;
            }
            let new: config::ResearchSettings =
                serde_json::from_value(cur).map_err(|e| Error::invalid(e.to_string()))?;
            config::save(ctx.conn(), &new)?;
            to_value(config::view(ctx.conn())?)
        }
        "fundamentals" => {
            let a: SubjectArgs = parse_args(args)?;
            to_value(views::fundamentals(ctx.conn(), &a.subject(), today)?)
        }
        "indicators" => {
            let a: SubjectArgs = parse_args(args)?;
            let d = data::load(ctx.conn(), &a.subject(), today, &mut Cache::default())?;
            to_value(indicators::compute(&d, bazin_of(ctx)?))
        }
        "dividends_check" => {
            let a: SubjectArgs = parse_args(args)?;
            to_value(super::dividends::check_subject(
                ctx.conn(),
                &a.subject(),
                today,
            )?)
        }
        "history" => to_value(views::history(
            ctx.conn(),
            &parse_args::<HistoryArgs>(args)?,
            today,
        )?),
        "fair_price" => {
            let a: FairPriceArgs = parse_args(args)?;
            let mut p = bazin_of(ctx)?;
            if let Some(r) = a.bazin_rate {
                let r = config::normalize_rate(r)?;
                p.rate = rust_decimal::Decimal::from_f64_retain(r)
                    .map(|d| d.round_dp(6))
                    .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-pesquisa-erro-taxa")))?;
            }
            if let Some(y) = a.years {
                p.years = config::validate_years(y)?;
            }
            let s = SubjectRef {
                asset_id: a.asset_id,
                ticker: a.ticker,
            };
            let d = data::load(ctx.conn(), &s, today, &mut Cache::default())?;
            let (graham, bazin) = indicators::fair_price(&d, p);
            to_value(FairPriceView {
                subject: d.subject.clone(),
                price: d.price.as_ref().map(|(_, p)| p.clone()),
                graham,
                bazin,
                disclaimer: indicators::disclaimer(),
            })
        }
        "rank" => {
            let a: RankArgs = parse_args(args)?;
            let (rows, as_of) = screen::universe(ctx.conn(), today, bazin_of(ctx)?)?;
            to_value(screen::rank(ctx.conn(), &rows, &as_of, &a)?)
        }
        "screen" => {
            let a: ScreenArgs = parse_args(args)?;
            let (rows, as_of) = screen::universe(ctx.conn(), today, bazin_of(ctx)?)?;
            to_value(screen::run_screen(ctx.conn(), &rows, &as_of, &a)?)
        }
        "refresh_universe" => {
            let b = bazin_of(ctx)?;
            let tx = ctx.db.conn_mut().transaction()?;
            let n = screen::recompute_universe(&tx, today, b)?;
            tx.commit()?;
            to_value(serde_json::json!({ "universe": n, "as_of": crate::time::fmt_date(today) }))
        }
        // Score em lote do universo (a coleta já faz; aqui, para recalcular na hora).
        "score_batch" => {
            let conn = ctx.db.conn_mut();
            to_value(crate::score::market::run(|f| f(conn), today)?)
        }
        // Tela Mercado: panorama por setor, mapa Saúde × Atratividade e Observar.
        "market_overview" => to_value(super::market::overview(ctx.conn(), today, bazin_of(ctx)?)?),
        "market_map" => to_value(super::market::map(ctx.conn(), today, bazin_of(ctx)?)?),
        // Métodos do explorador (receitas versionadas em methodology/screens).
        "methods" => to_value(super::methods::list()),
        "method_counts" => to_value(super::methods::counts(ctx.conn(), today, bazin_of(ctx)?)?),
        "method_badges" => to_value(super::methods::badges(ctx.conn(), today, bazin_of(ctx)?)?),
        // Características como tags (catálogo em methodology/tags), com a contagem de hoje.
        "tags" => {
            let (rows, as_of) = super::screen::universe(ctx.conn(), today, bazin_of(ctx)?)?;
            to_value(super::tags::catalog(&rows, &as_of))
        }
        "method_run" => {
            #[derive(Deserialize)]
            struct A {
                key: String,
            }
            let a: A = parse_args(args)?;
            to_value(super::methods::run(
                ctx.conn(),
                &a.key,
                today,
                bazin_of(ctx)?,
            )?)
        }
        "method_explain" => {
            #[derive(Deserialize)]
            struct A {
                key: String,
                ticker: String,
            }
            let a: A = parse_args(args)?;
            to_value(super::methods::explain(
                ctx.conn(),
                &a.key,
                &a.ticker,
                today,
                bazin_of(ctx)?,
            )?)
        }
        "observed" => {
            let a: IdArg = parse_args(args)?;
            to_value(super::market::is_observed(ctx.conn(), &a.id)?)
        }
        "observe" => {
            #[derive(Deserialize)]
            struct A {
                ticker: String,
            }
            let a: A = parse_args(args)?;
            let reason = crate::i18n::t_args(
                "nucleo-pesquisa-motivo-observar",
                &crate::i18n::args(&[("ticker", a.ticker.trim().to_uppercase().into())]),
            );
            let r = ctx.write(&reason, |tx| super::market::observe(tx, &a.ticker))?;
            to_value(serde_json::json!({ "asset_id": r.value }))
        }
        "unobserve" => {
            #[derive(Deserialize)]
            struct A {
                asset_id: String,
            }
            let a: A = parse_args(args)?;
            let r = ctx.write(
                &crate::i18n::t("nucleo-pesquisa-motivo-deixar-observar"),
                |tx| super::market::unobserve(tx, &a.asset_id),
            )?;
            to_value(serde_json::json!({ "removed": r.value }))
        }
        "screens.list" => to_value(screen::list(ctx.conn())?),
        "screens.get" => to_value(screen::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "screens.create" => {
            let i: ScreenInput = parse_args(args)?;
            let reason = crate::i18n::t_args(
                "nucleo-pesquisa-motivo-tela-criada",
                &crate::i18n::args(&[("tela", i.name.trim().to_string().into())]),
            );
            let r = ctx.write(&reason, |tx| screen::create(tx, &i))?;
            to_value(screen::get(ctx.conn(), &r.value)?)
        }
        "screens.update" => {
            let p: ScreenPatch = parse_args(args)?;
            let id = p.id.clone();
            ctx.write(
                &crate::i18n::t("nucleo-pesquisa-motivo-tela-alterada"),
                |tx| screen::update(tx, &p),
            )?;
            to_value(screen::get(ctx.conn(), &id)?)
        }
        "screens.delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(
                &crate::i18n::t("nucleo-pesquisa-motivo-tela-removida"),
                |tx| screen::delete(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        "compare" => to_value(views::compare(
            ctx.conn(),
            &parse_args::<CompareAssetsArgs>(args)?,
            today,
            bazin_of(ctx)?,
        )?),
        "documents" => to_value(views::documents(
            ctx.conn(),
            &parse_args::<DocumentsArgs>(args)?,
            today,
        )?),
        "fii_result" => to_value(super::fii_result::view(
            ctx.conn(),
            &parse_args::<super::fii_result::FiiResultArgs>(args)?,
            today,
        )?),
        "documents_status" => to_value(views::documents_status(
            ctx.conn(),
            &parse_args::<DocumentsArgs>(args)?,
            today,
        )?),
        "dividend_radar" => {
            let a: RadarArgs = parse_args(args)?;
            let ids = match a.asset_ids.filter(|v| !v.is_empty()) {
                Some(v) => {
                    for id in &v {
                        crate::assets::get(ctx.conn(), id)?;
                    }
                    v
                }
                None => data::held_or_watched(ctx.conn())?,
            };
            to_value(radar::run(ctx.conn(), &ids, today)?)
        }
        _ => Err(Error::UnknownCommand(format!("research.{action}"))),
    }
}
