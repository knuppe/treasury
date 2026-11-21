//! Comandos `score.*`: score de um ativo, da carteira, histórico, metodologia; segmento
//! manual e "discordo" só pela interface.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::assess::{self, AssetScoreSummary};
use super::config::methodology;
use super::{store, taxonomy};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::research::data::SubjectRef;
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &[
    "get",
    "portfolio",
    "assets",
    "history",
    "methodology",
    "feedback.list",
    "events.list",
];

/// Ativo cadastrado (`asset_id`) ou papel do universo (`ticker`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export, rename = "AssetScoreArgs")]
pub struct ScoreArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
}

impl ScoreArgs {
    fn subject(&self) -> SubjectRef {
        SubjectRef {
            asset_id: self.asset_id.clone().filter(|s| !s.trim().is_empty()),
            ticker: self.ticker.clone().filter(|s| !s.trim().is_empty()),
        }
    }
}

/// Ativos cadastrados para o resumo do score (lista de Ativos).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export, rename = "AssetScoreAssetsArgs")]
pub struct ScoreAssetsArgs {
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export, rename = "AssetScoreSegmentArgs")]
pub struct SegmentArgs {
    pub ticker: String,
    /// `equity_br` ou `reit`.
    pub class: String,
    /// Segmento da metodologia; `null` volta ao automático.
    pub segment: Option<String>,
}

/// Métrica de um perfil (para a explicação "como é calculado").
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreMethodologyMetric")]
pub struct MethodologyMetric {
    pub key: String,
    pub label: String,
    #[ts(type = "number")]
    pub weight: f64,
    pub formula: String,
    pub critical: bool,
    pub unavailable: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreMethodologyDimension")]
pub struct MethodologyDimension {
    pub key: String,
    pub label: String,
    #[ts(type = "number")]
    pub weight: f64,
    pub attractiveness: bool,
    pub metrics: Vec<MethodologyMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreMethodologyProfile")]
pub struct MethodologyProfile {
    pub key: String,
    pub label: String,
    pub dimensions: Vec<MethodologyDimension>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreSegment")]
pub struct SegmentOption {
    pub class: String,
    pub key: String,
    pub label: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreMethodology")]
pub struct MethodologyView {
    pub version: String,
    pub profiles: Vec<MethodologyProfile>,
    pub segments: Vec<SegmentOption>,
    /// (rótulo, mínimo, significado)
    pub bands: Vec<(String, f64, String)>,
    pub overrides: std::collections::BTreeMap<String, String>,
    /// Tipos de evento qualitativo: (chave, rótulo).
    pub event_kinds: Vec<(String, String)>,
}

fn methodology_view(conn: &rusqlite::Connection) -> Result<MethodologyView> {
    let m = methodology();
    let profiles = m
        .profiles
        .iter()
        .map(|(k, p)| MethodologyProfile {
            key: k.clone(),
            label: m.profile_label(k),
            dimensions: p
                .dimensions
                .0
                .iter()
                .map(|(dk, d)| MethodologyDimension {
                    key: dk.clone(),
                    label: m.dimension_label(p, dk),
                    weight: d.weight,
                    attractiveness: p.attractiveness.contains(dk),
                    metrics: d
                        .metrics
                        .0
                        .iter()
                        .map(|(mk, w)| MethodologyMetric {
                            key: mk.clone(),
                            label: m.metric_label(mk),
                            weight: *w,
                            formula: m.metric_formula(mk),
                            critical: p.critical.contains(mk),
                            unavailable: m.metric_unavailable(mk),
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect();
    let mut segments = vec![];
    let classes: Vec<String> = ["equity_br", "reit"]
        .iter()
        .map(|s| s.to_string())
        .chain(m.class_segments.keys().cloned())
        .collect();
    for class in &classes {
        for (k, s) in m.segments_of(class) {
            segments.push(SegmentOption {
                class: class.clone(),
                label: m.segment_label(class, &k, &s.label),
                key: k,
                profile: s.profile.clone(),
            });
        }
    }
    Ok(MethodologyView {
        version: m.version.clone(),
        profiles,
        segments,
        bands: m
            .bands
            .iter()
            .map(|b| (b.label_text(), b.min, b.meaning_text()))
            .collect(),
        overrides: taxonomy::overrides(conn)?,
        event_kinds: m
            .events
            .kinds
            .keys()
            .map(|k| (k.clone(), m.event_label(k)))
            .collect(),
    })
}

/// `key`: a mensagem de "só pela interface do app" de cada ação.
fn user_only(ctx: &Ctx<'_>, key: &str) -> Result<()> {
    if *ctx.actor != Actor::User {
        return Err(Error::Forbidden(crate::i18n::t(key)));
    }
    Ok(())
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let today = today_local();
    // Leitura de cliente MCP não grava (snapshot do dia, alertas, feedback de dado ausente):
    // isso fica para a interface e as rotinas do app.
    let persist = !matches!(ctx.actor, Actor::Mcp { .. });
    match action {
        "get" => {
            let a: ScoreArgs = parse_args(args)?;
            to_value(assess::assess_with(
                ctx.conn(),
                &a.subject(),
                today,
                persist,
            )?)
        }
        "portfolio" => {
            let out: Vec<AssetScoreSummary> = assess::portfolio(ctx.conn(), today, persist)?;
            to_value(out)
        }
        "assets" => {
            let a: ScoreAssetsArgs = parse_args(args)?;
            to_value(assess::summaries(ctx.conn(), &a.asset_ids, today, persist)?)
        }
        "history" => {
            let a: ScoreArgs = parse_args(args)?;
            let subject = match (&a.asset_id, &a.ticker) {
                (Some(id), _) => id.clone(),
                (None, Some(t)) => match crate::assets::find_by_ticker(ctx.conn(), t)? {
                    Some(asset) => asset.id,
                    None => store::subject_key(None, Some(t)),
                },
                _ => {
                    return Err(Error::invalid(crate::i18n::t(
                        "nucleo-pesquisa-erro-asset-ou-ticker",
                    )));
                }
            };
            to_value(store::history(ctx.conn(), &subject)?)
        }
        "methodology" => to_value(methodology_view(ctx.conn())?),
        "set_segment" => {
            user_only(ctx, "nucleo-score-erro-segmento-so-interface")?;
            let a: SegmentArgs = parse_args(args)?;
            taxonomy::set_override(
                ctx.conn(),
                methodology(),
                &a.class,
                &a.ticker,
                a.segment.as_deref(),
            )?;
            // O universo do dia muda com o segmento: recalcula na próxima consulta.
            ctx.conn()
                .execute("DELETE FROM score_universe WHERE class = ?1", [&a.class])?;
            to_value(methodology_view(ctx.conn())?)
        }
        "feedback.add" => {
            user_only(ctx, "nucleo-score-erro-avaliacao-so-interface")?;
            let i: store::FeedbackInput = parse_args(args)?;
            let s = assess::assess(
                ctx.conn(),
                &SubjectRef {
                    asset_id: i.asset_id.clone(),
                    ticker: i.ticker.clone(),
                },
                today,
            )?;
            let rec = ctx.write(&crate::i18n::t("nucleo-score-motivo-avaliacao"), |tx| {
                store::add_feedback(tx, &i, &s)
            })?;
            to_value(rec.value)
        }
        "events.list" => {
            let a: ScoreArgs = parse_args(args)?;
            to_value(super::events::list(
                ctx.conn(),
                methodology(),
                a.asset_id.as_deref(),
                a.ticker.as_deref(),
                today,
            )?)
        }
        "events.add" => {
            user_only(ctx, "nucleo-score-erro-eventos-so-interface")?;
            let i: super::events::EventInput = parse_args(args)?;
            let rec = ctx.write(&crate::i18n::t("nucleo-score-motivo-evento"), |tx| {
                super::events::add(tx, methodology(), &i)
            })?;
            to_value(rec.value)
        }
        "events.set_status" => {
            user_only(ctx, "nucleo-score-erro-eventos-so-interface")?;
            let a: super::events::EventStatusArgs = parse_args(args)?;
            ctx.write(
                &crate::i18n::t("nucleo-score-motivo-decisao-evento"),
                |tx| super::events::set_status(tx, &a),
            )?;
            Ok(Value::Null)
        }
        "events.delete" => {
            user_only(ctx, "nucleo-score-erro-eventos-so-interface")?;
            #[derive(Deserialize)]
            struct IdArg {
                id: String,
            }
            let a: IdArg = parse_args(args)?;
            ctx.write(&crate::i18n::t("nucleo-score-motivo-apagar-evento"), |tx| {
                super::events::delete(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "feedback.list" => {
            let a: ScoreArgs = parse_args(args)?;
            let subject = (a.asset_id.is_some() || a.ticker.is_some())
                .then(|| store::subject_key(a.asset_id.as_deref(), a.ticker.as_deref()));
            to_value(store::list_feedback(ctx.conn(), subject.as_deref())?)
        }
        _ => Err(Error::UnknownCommand(format!("score.{action}"))),
    }
}
