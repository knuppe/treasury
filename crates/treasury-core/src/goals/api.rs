use serde::Deserialize;
use serde_json::Value;

use super::{EmergencyReserveArgs, GoalInput, GoalListArgs, GoalPatch};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const READ_ONLY: &[&str] = &[
    "list",
    "get",
    "emergency_reserve",
    "retirement",
    "retirement_scenarios",
    "retirement_defaults",
    "inss_estimate",
    "education_get",
    "education_simulate",
    "education_inflation",
    "education_overview",
    "holdings_get",
    "objectives",
    "liquidity_candidates",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

/// Motivo do histórico com o nome da meta ou do cenário.
fn named(key: &str, name: &str) -> String {
    t_args(key, &i18n::args(&[("nome", name.to_string().into())]))
}

/// Risco de longevidade do plano (só com a linha da vida ligada); um erro nas tábuas não
/// derruba a simulação.
fn longevity_block(
    conn: &rusqlite::Connection,
    sim: &super::retirement::Simulation,
) -> Option<crate::longevity::RetirementLongevity> {
    let plan_age = sim
        .plan
        .life_expectancy
        .unwrap_or(crate::longevity::DEFAULT_PLAN_AGE);
    crate::longevity::retirement_block(conn, plan_age, crate::time::today_local())
        .ok()
        .flatten()
}

/// Simulação com o saque do FGTS na aposentadoria como evento de vida (Brasil, com conta FGTS
/// e sem `include_fgts: false`); o plano devolvido fica sem o evento, para não ser gravado.
fn simulate(
    conn: &rusqlite::Connection,
    plan: &super::retirement::Plan,
    wealth: i64,
) -> Result<super::retirement::Simulation> {
    let pension = crate::country::pension_name(conn);
    let fgts = if plan.include_fgts == Some(false) {
        None
    } else {
        let retire = plan
            .retirement_age
            .unwrap_or(65.max(plan.current_age.unwrap_or(65)));
        crate::fgts::retirement_withdrawal(
            conn,
            plan.current_age,
            retire,
            crate::time::today_local(),
        )?
        .map(|v| (retire, v))
    };
    let Some((retire, value)) = fgts else {
        return super::retirement::simulate_named(plan, wealth, &pension);
    };
    let mut with = plan.clone();
    let mut events = with.events.clone().unwrap_or_default();
    events.push(super::retirement::LifeEvent {
        age: retire,
        amount: value,
        label: t("nucleo-fgts-evento-aposentadoria"),
    });
    with.events = Some(events);
    let mut sim = super::retirement::simulate_named(&with, wealth, &pension)?;
    sim.plan.events = plan.events.clone();
    sim.fgts_at_retirement = Some(value);
    sim.notes.push(t_args(
        "nucleo-fgts-nota-aposentadoria",
        &i18n::args(&[(
            "valor",
            crate::i18n::money(crate::i18n::current(), value, "BRL").into(),
        )]),
    ));
    Ok(sim)
}

/// Idade de hoje do membro do plano de aposentadoria (pela data de nascimento cadastrada).
fn member_age(conn: &rusqlite::Connection) -> Result<Option<u32>> {
    Ok(crate::longevity::retirement_defaults(conn, None)?.current_age)
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => {
            let a: GoalListArgs = parse_args(args)?;
            to_value(super::list(
                ctx.conn(),
                a.include_archived.unwrap_or(false),
            )?)
        }
        "get" => to_value(super::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "retirement" => {
            // O plano salvo com o que veio na chamada por cima (sem gravar).
            let over: super::retirement::Plan = parse_args(args)?;
            let mut plan = super::retirement::merged(&super::retirement::saved(ctx.conn())?, &over);
            // Sem idade no pedido nem no plano salvo: a do membro do plano (data de nascimento),
            // como na tela.
            if plan.current_age.is_none() {
                plan.current_age = member_age(ctx.conn())?;
            }
            let wealth = super::retirement::portfolio_value(ctx.conn())?;
            let mut sim = simulate(ctx.conn(), &plan, wealth)?;
            sim.longevity = longevity_block(ctx.conn(), &sim);
            to_value(sim)
        }
        // Idade de hoje e horizonte sugerido pela linha da vida (docs/life-expectancy.md §7).
        "retirement_defaults" => {
            #[derive(Deserialize, Default)]
            struct A {
                #[serde(default)]
                member_id: Option<String>,
            }
            let a: A = parse_args(args)?;
            to_value(crate::longevity::retirement_defaults(
                ctx.conn(),
                a.member_id.as_deref(),
            )?)
        }
        "retirement_save" => {
            let plan: super::retirement::Plan = parse_args(args)?;
            let wealth = super::retirement::portfolio_value(ctx.conn())?;
            // Valida antes de gravar.
            let mut sim = simulate(ctx.conn(), &plan, wealth)?;
            sim.longevity = longevity_block(ctx.conn(), &sim);
            let today = crate::time::today_local();
            ctx.write(&t("nucleo-orcamento-motivo-plano-aposentadoria"), |tx| {
                crate::settings::set(tx, super::retirement::PLAN_SETTING, &plan)?;
                super::retirement::sync_linked_goal(tx, &sim, today)
            })?;
            to_value(sim)
        }
        "inss_estimate" => {
            #[derive(Deserialize)]
            struct A {
                input: super::inss::Input,
                #[serde(default)]
                current_age: Option<u32>,
                #[serde(default)]
                retirement_age: Option<u32>,
            }
            let a: A = parse_args(args)?;
            let saved = super::retirement::saved(ctx.conn())?;
            let age = match a.current_age.or(saved.current_age) {
                Some(x) => Some(x),
                None if a.input.birth_date.is_none() => member_age(ctx.conn())?,
                None => None,
            };
            let retire = a
                .retirement_age
                .or(saved.retirement_age)
                .unwrap_or(65.max(age.unwrap_or(65)));
            to_value(super::inss::estimate(
                &a.input,
                age,
                retire,
                crate::time::today_local(),
            )?)
        }
        "retirement_scenarios" => to_value(super::retirement::scenario_views(ctx.conn())?),
        "retirement_scenario_save" => {
            let input: super::retirement::ScenarioInput = parse_args(args)?;
            let wealth = super::retirement::portfolio_value(ctx.conn())?;
            let reason = named("nucleo-orcamento-motivo-cenario-salvo", input.name.trim());
            let out = ctx.write(&reason, |tx| {
                super::retirement::save_scenario(tx, &input, wealth)
            })?;
            to_value(out.value)
        }
        "retirement_scenario_delete" => {
            let a: IdArg = parse_args(args)?;
            let name = super::retirement::scenarios(ctx.conn())?
                .into_iter()
                .find(|s| s.id == a.id)
                .map(|s| s.name)
                .ok_or_else(|| Error::not_found(super::retirement::scenario_ref(&a.id)))?;
            ctx.write(
                &named("nucleo-orcamento-motivo-cenario-apagado", &name),
                |tx| super::retirement::delete_scenario(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        "retirement_link_goal" => {
            #[derive(Deserialize)]
            struct A {
                #[serde(default)]
                goal_id: Option<String>,
            }
            let a: A = parse_args(args)?;
            let plan = super::retirement::saved(ctx.conn())?;
            if plan.current_age.is_none() {
                return Err(Error::invalid(t(
                    "nucleo-orcamento-erro-plano-antes-da-meta",
                )));
            }
            let wealth = super::retirement::portfolio_value(ctx.conn())?;
            let sim = simulate(ctx.conn(), &plan, wealth)?;
            let today = crate::time::today_local();
            let id = ctx
                .write(&t("nucleo-orcamento-motivo-meta-ligada"), |tx| {
                    super::retirement::link_goal(tx, a.goal_id.as_deref(), &sim, today)
                })?
                .value;
            to_value(super::get(ctx.conn(), &id)?)
        }
        "retirement_unlink_goal" => {
            ctx.write(&t("nucleo-orcamento-motivo-meta-desligada"), |tx| {
                crate::settings::set(tx, super::retirement::GOAL_SETTING, &None::<String>)
            })?;
            Ok(Value::Null)
        }
        "create" => {
            let input: GoalInput = parse_args(args)?;
            let reason = named("nucleo-orcamento-motivo-meta-criada", input.name.trim());
            let id = ctx.write(&reason, |tx| super::create(tx, &input))?.value;
            to_value(super::get(ctx.conn(), &id)?)
        }
        "update" => {
            let patch: GoalPatch = parse_args(args)?;
            let name = super::load(ctx.conn(), &patch.id)?.name;
            ctx.write(
                &named("nucleo-orcamento-motivo-meta-alterada", &name),
                |tx| super::update(tx, &patch),
            )?;
            to_value(super::get(ctx.conn(), &patch.id)?)
        }
        "delete" => {
            let a: IdArg = parse_args(args)?;
            let name = super::load(ctx.conn(), &a.id)?.name;
            ctx.write(
                &named("nucleo-orcamento-motivo-meta-removida", &name),
                |tx| super::delete(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        // Meta de faculdade dos filhos (docs/design/college-goal.md).
        "education_get" => {
            #[derive(Deserialize)]
            struct A {
                goal_id: String,
            }
            let a: A = parse_args(args)?;
            to_value(super::education::plan_view(
                ctx.conn(),
                &a.goal_id,
                crate::time::today_local(),
            )?)
        }
        "education_simulate" => {
            let a: super::education::EducationSimulateArgs = parse_args(args)?;
            to_value(super::education::simulate(
                ctx.conn(),
                &a,
                crate::time::today_local(),
            )?)
        }
        "education_inflation" => {
            #[derive(Deserialize, Default)]
            #[serde(default)]
            struct A {
                series_id: Option<String>,
            }
            let a: A = parse_args(args)?;
            to_value(super::education::inflation(
                ctx.conn(),
                super::education::InflationMode::Series10y,
                a.series_id.as_deref(),
                None,
                crate::time::today_local(),
            )?)
        }
        "education_overview" => to_value(super::education::overview(
            ctx.conn(),
            crate::time::today_local(),
        )?),
        "education_save" => {
            let a: super::education::EducationSaveArgs = parse_args(args)?;
            let today = crate::time::today_local();
            let reason = match &a.goal_id {
                Some(id) => named(
                    "nucleo-orcamento-motivo-meta-alterada",
                    &super::load(ctx.conn(), id)?.name,
                ),
                None => t("nucleo-educacao-motivo-meta-criada"),
            };
            let id = ctx
                .write(&reason, |tx| super::education::save(tx, &a, today))?
                .value;
            to_value(super::education::plan_view(ctx.conn(), &id, today)?)
        }
        // Classificação "Objetivo": a carteira dividida pelas metas, pelos pesos das posições
        // ligadas.
        "objectives" => to_value(super::holdings::objectives(
            ctx.conn(),
            crate::time::today_local(),
        )?),
        "liquidity_candidates" => {
            #[derive(Deserialize)]
            struct A {
                goal_id: String,
            }
            let a: A = parse_args(args)?;
            to_value(super::sources::liquidity_candidates(
                ctx.conn(),
                &a.goal_id,
                crate::time::today_local(),
            )?)
        }
        "link_source" => {
            let a: super::sources::GoalLinkArgs = parse_args(args)?;
            let name = super::load(ctx.conn(), &a.goal_id)?.name;
            ctx.write(
                &named("nucleo-orcamento-motivo-meta-fonte-ligada", &name),
                |tx| super::sources::link(tx, &a),
            )?;
            to_value(super::get(ctx.conn(), &a.goal_id)?)
        }
        "unlink_source" => {
            let a: super::sources::GoalUnlinkArgs = parse_args(args)?;
            let name = super::load(ctx.conn(), &a.goal_id)?.name;
            ctx.write(
                &named("nucleo-orcamento-motivo-meta-fonte-desligada", &name),
                |tx| super::sources::unlink(tx, &a),
            )?;
            to_value(super::get(ctx.conn(), &a.goal_id)?)
        }
        "holdings_get" => {
            #[derive(Deserialize)]
            struct A {
                goal_id: String,
            }
            let a: A = parse_args(args)?;
            let goal = super::load(ctx.conn(), &a.goal_id)?;
            to_value(super::holdings::views(
                ctx.conn(),
                &goal,
                crate::time::today_local(),
            )?)
        }
        "holdings_set" => {
            let a: super::holdings::HoldingsSetArgs = parse_args(args)?;
            let goal = super::load(ctx.conn(), &a.goal_id)?;
            ctx.write(
                &named("nucleo-educacao-motivo-posicoes", &goal.name),
                |tx| super::holdings::set(tx, &a),
            )?;
            to_value(super::holdings::views(
                ctx.conn(),
                &super::load(ctx.conn(), &a.goal_id)?,
                crate::time::today_local(),
            )?)
        }
        "emergency_reserve" => to_value(super::emergency_reserve(
            ctx.conn(),
            &parse_args::<EmergencyReserveArgs>(args)?,
        )?),
        _ => Err(Error::UnknownCommand(format!("goals.{action}"))),
    }
}
