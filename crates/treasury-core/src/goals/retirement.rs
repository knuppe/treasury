//! Planejador de aposentadoria e independência financeira (`goals.retirement`): Monte Carlo
//! do patrimônio da idade de hoje até a expectativa de vida, em valores reais (acima do IPCA,
//! como o resto do app).
//!
//! Cada ano: patrimônio × (1 + retorno sorteado) + aportes (até a aposentadoria) − gasto
//! desejado + INSS (a partir da idade do benefício) ± eventos de vida (herança, venda de
//! imóvel, faculdade dos filhos…). Retorno anual lognormal com a média e a volatilidade
//! reais informadas. Sucesso = o dinheiro não acaba antes da expectativa de vida.
//!
//! O plano fica salvo (`goals.retirement_save`) e os campos não informados usam o salvo; o
//! patrimônio padrão é o valor de mercado da carteira (inclui previdência aberta e fechada).
//! O INSS pode vir calculado pelas regras ([`super::inss`], campo `inss_calc`).
//!
//! Além do plano principal, cenários com nome (`retirement_scenarios`, até
//! [`MAX_SCENARIOS`]) para comparar ("aposentar aos 55", "gastar menos"). Uma meta do tipo
//! aposentadoria pode ficar ligada ao plano principal (`retirement_link_goal`): o alvo é o
//! patrimônio necessário na aposentadoria, a data é a da aposentadoria e o aporte é o do
//! plano; salvar o plano atualiza a meta.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, current, t, t_args};

pub const PLAN_SETTING: &str = "goals.retirement_plan";
/// Cenários salvos além do plano principal (nome + plano).
pub const SCENARIOS_SETTING: &str = "goals.retirement_scenarios";
/// Meta de aposentadoria ligada ao plano principal.
pub const GOAL_SETTING: &str = "goals.retirement_goal";
pub const MAX_SCENARIOS: usize = 20;
pub const DEFAULT_SIMULATIONS: u32 = 2000;

/// Evento de vida: entrada (+) ou saída (−) em centavos de hoje, numa idade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementEvent")]
pub struct LifeEvent {
    pub age: u32,
    #[ts(type = "number")]
    pub amount: i64,
    pub label: String,
}

/// Plano (salvo) e parâmetros da simulação. Valores em centavos de hoje (reais).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementPlan")]
pub struct Plan {
    #[ts(optional)]
    #[serde(default)]
    pub current_age: Option<u32>,
    #[ts(optional)]
    #[serde(default)]
    pub retirement_age: Option<u32>,
    /// Expectativa de vida (padrão 95).
    #[ts(optional)]
    #[serde(default)]
    pub life_expectancy: Option<u32>,
    /// Patrimônio investido hoje (padrão: valor de mercado da carteira).
    #[ts(optional, type = "number")]
    #[serde(default)]
    pub current_wealth: Option<i64>,
    #[ts(optional, type = "number")]
    #[serde(default)]
    pub monthly_contribution: Option<i64>,
    /// Gasto mensal desejado na aposentadoria.
    #[ts(optional, type = "number")]
    #[serde(default)]
    pub monthly_spending: Option<i64>,
    /// INSS mensal estimado e a idade em que começa (padrão: a da aposentadoria).
    #[ts(optional, type = "number")]
    #[serde(default)]
    pub inss_monthly: Option<i64>,
    #[ts(optional)]
    #[serde(default)]
    pub inss_age: Option<u32>,
    /// Retorno real esperado e volatilidade, frações a.a. (padrão 4% e 10%).
    #[ts(optional)]
    #[serde(default)]
    pub real_return: Option<f64>,
    #[ts(optional)]
    #[serde(default)]
    pub volatility: Option<f64>,
    #[ts(optional)]
    #[serde(default)]
    pub events: Option<Vec<LifeEvent>>,
    #[ts(optional)]
    #[serde(default)]
    pub simulations: Option<u32>,
    /// Calcular o INSS pelas regras (vale quando `inss_monthly` não é informado).
    #[ts(optional)]
    #[serde(default)]
    pub inss_calc: Option<super::inss::Input>,
    /// Calcular a Social Security dos EUA (família nos EUA, sem `inss_monthly`).
    #[ts(optional)]
    #[serde(default)]
    pub us_ssa_calc: Option<super::pension_us::Input>,
    /// Calcular a pensão de velhice de Portugal (família em Portugal, sem `inss_monthly`).
    #[ts(optional)]
    #[serde(default)]
    pub pt_pension_calc: Option<super::pension_pt::Input>,
    /// Somar o saldo FGTS projetado como saque na aposentadoria (padrão: sim, no Brasil e com
    /// conta FGTS; docs/brazil/fgts.md, seção 6.7).
    #[ts(optional)]
    #[serde(default)]
    pub include_fgts: Option<bool>,
}

/// Benefício público pelas regras (INSS, Social Security ou pensão de velhice), quando o
/// plano pede o cálculo e não traz o valor informado. `retire` é a idade em que para de
/// trabalhar.
pub fn public_estimate(
    p: &Plan,
    retire: u32,
    today: chrono::NaiveDate,
) -> Result<Option<super::inss::Estimate>> {
    if p.inss_monthly.is_some() {
        return Ok(None);
    }
    Ok(if let Some(i) = &p.inss_calc {
        Some(super::inss::estimate(i, p.current_age, retire, today)?)
    } else if let Some(i) = &p.us_ssa_calc {
        Some(super::pension_us::estimate(
            i,
            p.current_age,
            retire,
            today,
        )?)
    } else if let Some(i) = &p.pt_pension_calc {
        Some(super::pension_pt::estimate(
            i,
            p.current_age,
            retire,
            today,
        )?)
    } else {
        None
    })
}

/// Nome do benefício de cada sistema de regras.
fn system_name(system: &str) -> &'static str {
    match system {
        "us_ssa" => "Social Security",
        "pt_ss" => "Segurança Social",
        _ => "INSS",
    }
}

/// Moeda do benefício de cada sistema de regras.
fn system_currency(system: &str) -> &'static str {
    match system {
        "us_ssa" => "USD",
        "pt_ss" => "EUR",
        _ => "BRL",
    }
}

/// Faixa do patrimônio numa idade (percentis 10, 50 e 90 das simulações).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementBand")]
pub struct Band {
    pub age: u32,
    #[ts(type = "number")]
    pub p10: i64,
    #[ts(type = "number")]
    pub p50: i64,
    #[ts(type = "number")]
    pub p90: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementSimulation")]
pub struct Simulation {
    /// Plano usado (salvo + o que veio na chamada + padrões).
    pub plan: Plan,
    /// Fração das simulações em que o dinheiro dura até a expectativa de vida.
    pub success: f64,
    pub simulations: u32,
    /// Patrimônio de partida usado (o informado ou a carteira).
    #[ts(type = "number")]
    pub start_wealth: i64,
    /// Patrimônio na idade da aposentadoria (mediana das simulações).
    #[ts(type = "number")]
    pub wealth_at_retirement: i64,
    /// Maior gasto mensal com 90% de sucesso (mesmos aportes, INSS e eventos).
    #[ts(type = "number")]
    pub sustainable_spending: i64,
    /// Idade em que o dinheiro acaba no cenário ruim (percentil 10), se acabar.
    pub depletion_age_p10: Option<u32>,
    /// Patrimônio necessário na aposentadoria para 90% de sucesso dali em diante (mesmo
    /// gasto, INSS e eventos).
    #[ts(type = "number")]
    pub required_wealth: i64,
    /// INSS calculado pelas regras (com `inss_calc` e sem `inss_monthly`).
    pub inss: Option<super::inss::Estimate>,
    /// Saque do FGTS na aposentadoria somado como evento de vida (reais de hoje).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub fgts_at_retirement: Option<i64>,
    pub bands: Vec<Band>,
    pub notes: Vec<String>,
    /// Risco de longevidade (linha da vida ligada e data de nascimento do titular): a idade do
    /// plano, a chance de passar dela e a idade "1 em 10" sugerida.
    #[serde(default)]
    #[ts(inline)]
    pub longevity: Option<crate::longevity::RetirementLongevity>,
}

/// Gerador determinístico (xorshift64*): a mesma entrada dá o mesmo resultado.
struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        let v = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        ((v >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }

    /// Normal padrão (Box-Muller).
    fn normal(&mut self) -> f64 {
        let u1 = self.next_f64();
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Parâmetros completos (já com os padrões).
#[derive(Debug, Clone)]
struct Params {
    age: u32,
    retire: u32,
    end: u32,
    wealth: f64,
    contribution: f64,
    spending: f64,
    inss: f64,
    inss_age: u32,
    mu: f64,
    sigma: f64,
    events: Vec<LifeEvent>,
    n: u32,
}

fn params(p: &Plan, default_wealth: i64) -> Result<Params> {
    let age = p
        .current_age
        .ok_or_else(|| Error::invalid(t("nucleo-orcamento-erro-idade-hoje")))?;
    let retire = p.retirement_age.unwrap_or(65.max(age));
    let end = p.life_expectancy.unwrap_or(95);
    if !(16..=100).contains(&age) || retire < age || end <= retire || end > 110 {
        return Err(Error::invalid(t("nucleo-orcamento-erro-idades")));
    }
    let ret = p.real_return.unwrap_or(0.04);
    let vol = p.volatility.unwrap_or(0.10);
    if !(-0.2..=0.3).contains(&ret) || !(0.0..=0.6).contains(&vol) {
        return Err(Error::invalid(t(
            "nucleo-orcamento-erro-retorno-volatilidade",
        )));
    }
    // Lognormal com a média aritmética pedida.
    let sigma = ((1.0 + vol * vol / ((1.0 + ret) * (1.0 + ret))).ln()).sqrt();
    let mu = (1.0 + ret).ln() - sigma * sigma / 2.0;
    Ok(Params {
        age,
        retire,
        end,
        wealth: p.current_wealth.unwrap_or(default_wealth).max(0) as f64,
        contribution: p.monthly_contribution.unwrap_or(0).max(0) as f64 * 12.0,
        spending: p.monthly_spending.unwrap_or(0).max(0) as f64 * 12.0,
        inss: p.inss_monthly.unwrap_or(0).max(0) as f64 * 12.0,
        inss_age: p.inss_age.unwrap_or(retire),
        mu,
        sigma,
        events: p.events.clone().unwrap_or_default(),
        n: p.simulations
            .unwrap_or(DEFAULT_SIMULATIONS)
            .clamp(100, 20_000),
    })
}

/// Trajetórias: patrimônio no início de cada idade (`age..=end`), por simulação.
fn run(p: &Params, spending: f64) -> Vec<Vec<f64>> {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let years = (p.end - p.age) as usize;
    (0..p.n)
        .map(|_| {
            let mut w = p.wealth;
            let mut path = Vec::with_capacity(years + 1);
            path.push(w);
            for y in 0..years {
                let age = p.age + y as u32;
                let r = (p.mu + p.sigma * rng.normal()).exp() - 1.0;
                let mut flow = 0.0;
                if age < p.retire {
                    flow += p.contribution;
                } else {
                    flow -= spending;
                }
                if age >= p.inss_age {
                    flow += p.inss;
                }
                flow += p
                    .events
                    .iter()
                    .filter(|e| e.age == age)
                    .map(|e| e.amount as f64)
                    .sum::<f64>();
                // Fluxos no meio do ano: metade rende.
                w = if w > 0.0 {
                    w * (1.0 + r) + flow * (1.0 + r / 2.0)
                } else {
                    w + flow
                };
                path.push(w);
            }
            path
        })
        .collect()
}

fn success_rate(paths: &[Vec<f64>]) -> f64 {
    let ok = paths.iter().filter(|p| p.iter().all(|w| *w >= 0.0)).count();
    ok as f64 / paths.len().max(1) as f64
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i]
}

pub fn simulate(p: &Plan, default_wealth: i64) -> Result<Simulation> {
    simulate_on(p, default_wealth, crate::time::today_local())
}

/// [`simulate`] com o nome da aposentadoria pública do país nas notas (valor informado).
pub fn simulate_named(p: &Plan, default_wealth: i64, pension: &str) -> Result<Simulation> {
    simulate_in(p, default_wealth, crate::time::today_local(), pension)
}

/// Patrimônio na aposentadoria com 90% de sucesso dali até a expectativa de vida.
fn required_at_retirement(pr: &Params) -> f64 {
    let after = Params {
        age: pr.retire,
        wealth: 0.0,
        events: pr
            .events
            .iter()
            .filter(|e| e.age >= pr.retire)
            .cloned()
            .collect(),
        n: pr.n.min(800),
        ..pr.clone()
    };
    let (mut lo, mut hi) = (0.0f64, (pr.spending * 60.0).max(1.0));
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        let t = Params {
            wealth: mid,
            ..after.clone()
        };
        if success_rate(&run(&t, t.spending)) >= 0.9 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

pub fn simulate_on(p: &Plan, default_wealth: i64, today: chrono::NaiveDate) -> Result<Simulation> {
    simulate_in(p, default_wealth, today, "INSS")
}

fn simulate_in(
    p: &Plan,
    default_wealth: i64,
    today: chrono::NaiveDate,
    pension: &str,
) -> Result<Simulation> {
    // Benefício público pelas regras quando não informado.
    let retire = p
        .retirement_age
        .unwrap_or(65.max(p.current_age.unwrap_or(65)));
    let inss = public_estimate(p, retire, today)?;
    let mut p = p.clone();
    let inss_age_given = p.inss_age.is_some();
    if let Some(e) = &inss {
        // Renda mensal equivalente: o benefício × pagamentos no ano ÷ 12 (13º do INSS,
        // subsídios de Natal e férias em Portugal).
        let monthly = e.monthly.unwrap_or(0);
        p.inss_monthly = Some(monthly * e.payments_per_year as i64 / 12);
        if p.inss_age.is_none() {
            p.inss_age = e.start_age;
        }
    }
    let p = &p;
    let pr = params(p, default_wealth)?;
    let paths = run(&pr, pr.spending);
    let success = success_rate(&paths);
    let years = (pr.end - pr.age) as usize;
    let mut bands = vec![];
    let mut depletion = None;
    for y in 0..=years {
        let mut col: Vec<f64> = paths.iter().map(|p| p[y]).collect();
        col.sort_by(f64::total_cmp);
        let p10 = percentile(&col, 0.10);
        if depletion.is_none() && p10 < 0.0 {
            depletion = Some(pr.age + y as u32);
        }
        bands.push(Band {
            age: pr.age + y as u32,
            p10: p10.max(0.0).round() as i64,
            p50: percentile(&col, 0.50).max(0.0).round() as i64,
            p90: percentile(&col, 0.90).max(0.0).round() as i64,
        });
    }
    let at_retirement = bands
        .get((pr.retire - pr.age) as usize)
        .map_or(0, |b| b.p50);
    // Maior gasto com 90% de sucesso (busca binária, menos simulações).
    let quick = Params {
        n: pr.n.min(800),
        ..pr.clone()
    };
    let (mut lo, mut hi) = (
        0.0f64,
        (pr.wealth + pr.contribution * 60.0 + pr.inss * 60.0).max(1.0),
    );
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        if success_rate(&run(&quick, mid)) >= 0.9 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let l = current();
    let mut notes = vec![t_args(
        "nucleo-orcamento-aposentadoria-nota-premissas",
        &args(&[
            (
                "retorno",
                crate::ai::alerts::pct(l, p.real_return.unwrap_or(0.04), 1).into(),
            ),
            (
                "volatilidade",
                crate::ai::alerts::pct(l, p.volatility.unwrap_or(0.10), 0).into(),
            ),
            ("simulacoes", pr.n.into()),
        ]),
    )];
    if p.current_wealth.is_none() {
        notes.push(t("nucleo-orcamento-aposentadoria-nota-patrimonio"));
    }
    if let Some(e) = &inss {
        match (e.monthly, e.start_age) {
            (Some(m), Some(a)) => notes.push(t_args(
                "nucleo-orcamento-aposentadoria-nota-inss",
                &args(&[
                    ("sistema", system_name(&e.system).into()),
                    (
                        "valor",
                        crate::i18n::money(l, m, system_currency(&e.system)).into(),
                    ),
                    ("idade", a.into()),
                    (
                        "regra",
                        e.rules
                            .iter()
                            .find(|r| Some(&r.id) == e.chosen.as_ref())
                            .map_or_else(
                                || t("nucleo-orcamento-aposentadoria-regra-escolhida"),
                                |r| r.name.clone(),
                            )
                            .into(),
                    ),
                ]),
            )),
            _ => notes.push(t_args(
                "nucleo-orcamento-aposentadoria-nota-inss-nenhuma",
                &args(&[("sistema", system_name(&e.system).into())]),
            )),
        }
    } else if p.inss_monthly.unwrap_or(0) == 0 {
        notes.push(t_args(
            "nucleo-orcamento-aposentadoria-nota-sem-inss",
            &args(&[("sistema", pension.to_string().into())]),
        ));
    }
    let required = required_at_retirement(&pr);
    let mut plan = p.clone();
    // O INSS calculado não vira valor informado no plano devolvido.
    if inss.is_some() {
        plan.inss_monthly = None;
    }
    plan.retirement_age = Some(pr.retire);
    plan.life_expectancy = Some(pr.end);
    // A idade do INSS calculado acompanha o cálculo (não vira valor informado).
    plan.inss_age = (inss.is_none() || inss_age_given).then_some(pr.inss_age);
    Ok(Simulation {
        plan,
        success,
        simulations: pr.n,
        start_wealth: pr.wealth.round() as i64,
        wealth_at_retirement: at_retirement,
        sustainable_spending: (lo / 12.0).floor() as i64,
        depletion_age_p10: depletion,
        required_wealth: required.round() as i64,
        inss,
        bands,
        notes,
        longevity: None,
        fgts_at_retirement: None,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementScenario")]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub plan: Plan,
    pub updated_at: String,
}

/// Cenário com o resumo da simulação (ou o erro, se o plano não fecha).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementScenarioView")]
pub struct ScenarioView {
    pub id: String,
    pub name: String,
    pub plan: Plan,
    pub updated_at: String,
    pub success: Option<f64>,
    #[ts(type = "number | null")]
    pub sustainable_spending: Option<i64>,
    #[ts(type = "number | null")]
    pub required_wealth: Option<i64>,
    #[ts(type = "number | null")]
    pub wealth_at_retirement: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, rename = "RetirementScenarioInput")]
pub struct ScenarioInput {
    /// Sem id: cenário novo.
    #[ts(optional)]
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub plan: Plan,
}

pub fn scenarios(conn: &rusqlite::Connection) -> Result<Vec<Scenario>> {
    crate::settings::get_or(conn, SCENARIOS_SETTING, Vec::new())
}

/// Cenários com o resumo (simulação mais curta).
pub fn scenario_views(conn: &rusqlite::Connection) -> Result<Vec<ScenarioView>> {
    let wealth = portfolio_value(conn)?;
    Ok(scenarios(conn)?
        .into_iter()
        .map(|s| {
            let sim = simulate(
                &Plan {
                    simulations: Some(s.plan.simulations.unwrap_or(800).min(800)),
                    ..s.plan.clone()
                },
                wealth,
            );
            let (ok, err) = match sim {
                Ok(v) => (Some(v), None),
                Err(e) => (None, Some(e.to_string())),
            };
            ScenarioView {
                id: s.id,
                name: s.name,
                plan: s.plan,
                updated_at: s.updated_at,
                success: ok.as_ref().map(|v| v.success),
                sustainable_spending: ok.as_ref().map(|v| v.sustainable_spending),
                required_wealth: ok.as_ref().map(|v| v.required_wealth),
                wealth_at_retirement: ok.as_ref().map(|v| v.wealth_at_retirement),
                error: err,
            }
        })
        .collect())
}

/// Cria ou substitui um cenário (o plano é validado antes). Devolve o cenário salvo.
pub fn save_scenario(
    conn: &rusqlite::Connection,
    input: &ScenarioInput,
    default_wealth: i64,
) -> Result<Scenario> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(Error::invalid(t("nucleo-orcamento-erro-cenario-sem-nome")));
    }
    simulate(
        &Plan {
            simulations: Some(100),
            ..input.plan.clone()
        },
        default_wealth,
    )?;
    let mut list = scenarios(conn)?;
    if list
        .iter()
        .any(|s| s.name.eq_ignore_ascii_case(name) && Some(&s.id) != input.id.as_ref())
    {
        return Err(Error::conflict(t_args(
            "nucleo-orcamento-erro-cenario-repetido",
            &args(&[("nome", name.to_string().into())]),
        )));
    }
    let now = crate::time::now_utc();
    let saved = match &input.id {
        Some(id) => {
            let s = list
                .iter_mut()
                .find(|s| &s.id == id)
                .ok_or_else(|| Error::not_found(scenario_ref(id)))?;
            s.name = name.to_string();
            s.plan = input.plan.clone();
            s.updated_at = now;
            s.clone()
        }
        None => {
            if list.len() >= MAX_SCENARIOS {
                return Err(Error::invalid(t_args(
                    "nucleo-orcamento-erro-cenarios-maximo",
                    &args(&[("maximo", MAX_SCENARIOS.into())]),
                )));
            }
            let s = Scenario {
                id: crate::ids::new_id(),
                name: name.to_string(),
                plan: input.plan.clone(),
                updated_at: now,
            };
            list.push(s.clone());
            s
        }
    };
    crate::settings::set(conn, SCENARIOS_SETTING, &list)?;
    Ok(saved)
}

/// "cenário <id>", para o erro de não encontrado.
pub(super) fn scenario_ref(id: &str) -> String {
    t_args(
        "nucleo-orcamento-cenario",
        &args(&[("id", id.to_string().into())]),
    )
}

pub fn delete_scenario(conn: &rusqlite::Connection, id: &str) -> Result<String> {
    let mut list = scenarios(conn)?;
    let Some(i) = list.iter().position(|s| s.id == id) else {
        return Err(Error::not_found(scenario_ref(id)));
    };
    let s = list.remove(i);
    crate::settings::set(conn, SCENARIOS_SETTING, &list)?;
    Ok(s.name)
}

/// Meta ligada ao plano (se ainda existe e não está arquivada).
pub fn linked_goal(conn: &rusqlite::Connection) -> Result<Option<String>> {
    let id: Option<String> = crate::settings::get_or(conn, GOAL_SETTING, None)?;
    let Some(id) = id else { return Ok(None) };
    let alive: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM goal WHERE id = ?1 AND archived = 0)",
        [&id],
        |r| r.get(0),
    )?;
    Ok(alive.then_some(id))
}

/// Alvo, data e aporte da meta a partir da simulação do plano principal.
pub fn goal_fields(
    sim: &Simulation,
    today: chrono::NaiveDate,
) -> (i64, Option<String>, Option<i64>) {
    let age = sim.plan.current_age.unwrap_or(0);
    let retire = sim.plan.retirement_age.unwrap_or(age);
    let date = crate::time::add_months(today, ((retire.saturating_sub(age)) * 12) as i32);
    (
        sim.required_wealth.max(1),
        Some(crate::time::fmt_date(date)),
        sim.plan.monthly_contribution,
    )
}

/// Cria (sem `goal_id`) ou liga uma meta ao plano e atualiza alvo, data e aporte.
pub fn link_goal(
    tx: &rusqlite::Transaction<'_>,
    goal_id: Option<&str>,
    sim: &Simulation,
    today: chrono::NaiveDate,
) -> Result<String> {
    let (target, date, monthly) = goal_fields(sim, today);
    let id = match goal_id {
        Some(id) => {
            let g = super::load(tx, id)?;
            if g.archived {
                return Err(Error::invalid(t("nucleo-orcamento-erro-meta-arquivada")));
            }
            id.to_string()
        }
        None => super::create(
            tx,
            &super::GoalInput {
                name: t("nucleo-orcamento-aposentadoria-meta"),
                kind: super::GoalKind::Retirement,
                target_amount: target,
                // Na moeda base da família, a mesma da simulação.
                currency: None,
                target_date: date.clone(),
                initial_amount: None,
                monthly_contribution: monthly,
                account_ids: None,
                notes: Some(t("nucleo-orcamento-aposentadoria-meta-nota")),
                real_return: None,
            },
        )?,
    };
    update_goal(tx, &id, target, date, monthly)?;
    crate::settings::set(tx, GOAL_SETTING, &Some(&id))?;
    Ok(id)
}

fn update_goal(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
    target: i64,
    date: Option<String>,
    monthly: Option<i64>,
) -> Result<()> {
    super::update(
        tx,
        &super::GoalPatch {
            id: id.to_string(),
            name: None,
            kind: Some(super::GoalKind::Retirement),
            target_amount: Some(target),
            target_date: Some(date),
            initial_amount: None,
            monthly_contribution: Some(monthly),
            account_ids: None,
            archived: None,
            notes: None,
            real_return: None,
        },
    )
}

/// Depois de salvar o plano: a meta ligada acompanha.
pub fn sync_linked_goal(
    tx: &rusqlite::Transaction<'_>,
    sim: &Simulation,
    today: chrono::NaiveDate,
) -> Result<Option<String>> {
    let Some(id) = linked_goal(tx)? else {
        return Ok(None);
    };
    let (target, date, monthly) = goal_fields(sim, today);
    update_goal(tx, &id, target, date, monthly)?;
    Ok(Some(id))
}

/// O plano salvo com o que veio na chamada por cima.
pub fn merged(saved: &Plan, over: &Plan) -> Plan {
    Plan {
        current_age: over.current_age.or(saved.current_age),
        retirement_age: over.retirement_age.or(saved.retirement_age),
        life_expectancy: over.life_expectancy.or(saved.life_expectancy),
        current_wealth: over.current_wealth.or(saved.current_wealth),
        monthly_contribution: over.monthly_contribution.or(saved.monthly_contribution),
        monthly_spending: over.monthly_spending.or(saved.monthly_spending),
        inss_monthly: over.inss_monthly.or(saved.inss_monthly),
        inss_age: over.inss_age.or(saved.inss_age),
        real_return: over.real_return.or(saved.real_return),
        volatility: over.volatility.or(saved.volatility),
        events: over.events.clone().or_else(|| saved.events.clone()),
        simulations: over.simulations.or(saved.simulations),
        inss_calc: over.inss_calc.clone().or_else(|| saved.inss_calc.clone()),
        us_ssa_calc: over
            .us_ssa_calc
            .clone()
            .or_else(|| saved.us_ssa_calc.clone()),
        pt_pension_calc: over
            .pt_pension_calc
            .clone()
            .or_else(|| saved.pt_pension_calc.clone()),
        include_fgts: over.include_fgts.or(saved.include_fgts),
    }
}

pub fn saved(conn: &rusqlite::Connection) -> Result<Plan> {
    crate::settings::get_or(conn, PLAN_SETTING, Plan::default())
}

/// Valor de mercado da carteira hoje, na moeda base (patrimônio investido padrão).
pub fn portfolio_value(conn: &rusqlite::Connection) -> Result<i64> {
    Ok(crate::portfolio::engine::summary(
        conn,
        &crate::portfolio::PortfolioArgs {
            as_of: None,
            account_ids: None,
            deflator: None,
            by_account: None,
            include_closed: None,
        },
    )?
    .market_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> Plan {
        Plan {
            current_age: Some(40),
            retirement_age: Some(60),
            life_expectancy: Some(90),
            current_wealth: Some(50_000_000),
            monthly_contribution: Some(300_000),
            monthly_spending: Some(1_000_000),
            inss_monthly: Some(300_000),
            ..Default::default()
        }
    }

    #[test]
    fn deterministic_and_sensible() {
        let a = simulate(&plan(), 0).unwrap();
        let b = simulate(&plan(), 0).unwrap();
        assert_eq!(a, b, "same input, same result");
        assert_eq!(a.bands.first().unwrap().age, 40);
        assert_eq!(a.bands.last().unwrap().age, 90);
        assert!(a.success > 0.0 && a.success <= 1.0);
        // Spending more reduces success; the sustainable spending stays near the 90% limit.
        let rich = simulate(
            &Plan {
                monthly_spending: Some(3_000_000),
                ..plan()
            },
            0,
        )
        .unwrap();
        assert!(rich.success < a.success);
        let at = simulate(
            &Plan {
                monthly_spending: Some(a.sustainable_spending),
                ..plan()
            },
            0,
        )
        .unwrap();
        assert!(at.success >= 0.85, "{}", at.success);
        // An inheritance of R$ 1 million at 70 improves the result.
        let heir = simulate(
            &Plan {
                monthly_spending: Some(3_000_000),
                events: Some(vec![LifeEvent {
                    age: 70,
                    amount: 100_000_000,
                    label: "herança".into(),
                }]),
                ..plan()
            },
            0,
        )
        .unwrap();
        assert!(heir.success > rich.success);
    }

    #[test]
    fn public_pension_of_each_country_counts_every_payment_of_the_year() {
        let today = crate::time::parse_date("2026-10-21").unwrap();
        let base = Plan {
            inss_monthly: None,
            retirement_age: Some(67),
            ..plan()
        };
        let us = Plan {
            us_ssa_calc: Some(crate::goals::pension_us::Input {
                birth_date: None,
                pia: Some(250_000),
                aime: None,
                credits: None,
                work_until_age: None,
            }),
            ..base.clone()
        };
        let pt = Plan {
            pt_pension_calc: Some(crate::goals::pension_pt::Input {
                birth_date: None,
                contribution_years: 18.0,
                reference_salary: 150_000,
                contribute_until_age: None,
                started_before_17: None,
            }),
            ..base.clone()
        };
        for (p, system, per_year) in [(us, "us_ssa", 12), (pt, "pt_ss", 14)] {
            let s = simulate_on(&p, 0, today).unwrap();
            let e = s.inss.clone().unwrap();
            assert_eq!(e.system, system);
            assert_eq!(e.payments_per_year, per_year);
            let monthly = e.monthly.unwrap();
            // The same as entering the benefit × payments ÷ 12 at the calculated age.
            let manual = simulate_on(
                &Plan {
                    inss_monthly: Some(monthly * per_year as i64 / 12),
                    inss_age: e.start_age,
                    ..base.clone()
                },
                0,
                today,
            )
            .unwrap();
            assert_eq!(s.success, manual.success, "{system}");
            assert_eq!(s.sustainable_spending, manual.sustainable_spending);
            assert!(s.notes.iter().any(|n| n.contains(if system == "us_ssa" {
                "Social Security"
            } else {
                "Segurança Social"
            })));
        }
    }

    #[test]
    fn without_volatility_matches_the_closed_form() {
        // No volatility and no flows: 1,000,000 × 1.04^20.
        let s = simulate(
            &Plan {
                current_age: Some(40),
                retirement_age: Some(60),
                life_expectancy: Some(61),
                current_wealth: Some(100_000_000),
                volatility: Some(0.0),
                simulations: Some(100),
                ..Default::default()
            },
            0,
        )
        .unwrap();
        let expected = 100_000_000.0 * 1.04f64.powi(20);
        assert!((s.wealth_at_retirement as f64 - expected).abs() / expected < 1e-6);
        assert_eq!(s.success, 1.0);
    }
}

#[cfg(test)]
mod api_tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    fn api() -> (tempfile::TempDir, Api) {
        let dir = tempfile::tempdir().unwrap();
        let api =
            Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
                .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        (dir, api)
    }

    fn plan(spending: i64) -> serde_json::Value {
        json!({
            "current_age": 45, "retirement_age": 62, "life_expectancy": 90,
            "current_wealth": 80_000_000, "monthly_contribution": 400_000,
            "monthly_spending": spending, "simulations": 300,
            "inss_calc": {"sex": "female", "contribution_years": 22.0,
                          "average_salary": 700_000}
        })
    }

    #[test]
    fn inss_by_the_rules_feeds_the_simulation() {
        let (_d, api) = api();
        let s = api.call("goals.retirement", plan(1_200_000)).unwrap();
        let inss = &s["inss"];
        assert!(inss["monthly"].as_i64().unwrap() > 0, "{inss}");
        assert!(inss["start_age"].as_u64().unwrap() >= 62);
        assert!(
            s["plan"]["inss_monthly"].is_null(),
            "the calculated value does not become the given one"
        );
        assert!(s["required_wealth"].as_i64().unwrap() > 0);
        assert!(
            s["notes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n.as_str().unwrap().starts_with("INSS calculado"))
        );
        // The given value wins over the calculated one.
        let mut p = plan(1_200_000);
        p["inss_monthly"] = json!(100_000);
        let s = api.call("goals.retirement", p).unwrap();
        assert!(s["inss"].is_null());
        // The standalone tool also calculates.
        let e = api
            .call(
                "goals.inss_estimate",
                json!({"input": {"sex": "male", "contribution_years": 30.0,
                                 "average_salary": 500_000},
                       "current_age": 55, "retirement_age": 60}),
            )
            .unwrap();
        assert!(e["rules"].as_array().unwrap().len() >= 4, "{e}");
    }

    #[test]
    fn scenarios_are_saved_listed_and_deleted() {
        let (_d, api) = api();
        let a = api
            .call(
                "goals.retirement_scenario_save",
                json!({"name": "Aos 62", "plan": plan(1_000_000)}),
            )
            .unwrap();
        api.call(
            "goals.retirement_scenario_save",
            json!({"name": "Gastar mais", "plan": plan(2_000_000)}),
        )
        .unwrap();
        // A repeated name is rejected; editing by id is not.
        assert!(
            api.call(
                "goals.retirement_scenario_save",
                json!({"name": "aos 62", "plan": plan(1)}),
            )
            .is_err()
        );
        api.call(
            "goals.retirement_scenario_save",
            json!({"id": a["id"], "name": "Aos 62", "plan": plan(900_000)}),
        )
        .unwrap();
        let list = api.call("goals.retirement_scenarios", json!({})).unwrap();
        let list = list.as_array().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0]["plan"]["monthly_spending"], 900_000);
        assert!(
            list[0]["success"].as_f64().unwrap() >= list[1]["success"].as_f64().unwrap(),
            "spending less gives more chance"
        );
        api.call("goals.retirement_scenario_delete", json!({"id": a["id"]}))
            .unwrap();
        assert_eq!(
            api.call("goals.retirement_scenarios", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn retirement_goal_follows_the_saved_plan() {
        let (_d, api) = api();
        // No saved plan: a clear error.
        assert!(api.call("goals.retirement_link_goal", json!({})).is_err());
        let s = api.call("goals.retirement_save", plan(1_000_000)).unwrap();
        let g = api.call("goals.retirement_link_goal", json!({})).unwrap();
        assert_eq!(g["kind"], "retirement");
        assert_eq!(g["linked_to_plan"], true);
        assert_eq!(g["target_amount"], s["required_wealth"]);
        assert_eq!(g["monthly_contribution"], 400_000);
        assert!(g["target_date"].is_string());
        // Saving the plan with another spending updates the goal's target.
        let s2 = api.call("goals.retirement_save", plan(2_000_000)).unwrap();
        let g2 = api.call("goals.get", json!({"id": g["id"]})).unwrap();
        assert_eq!(g2["target_amount"], s2["required_wealth"]);
        assert!(g2["target_amount"].as_i64() > g["target_amount"].as_i64());
        // Turning it off: the goal stays, without following the plan.
        api.call("goals.retirement_unlink_goal", json!({})).unwrap();
        let g3 = api.call("goals.get", json!({"id": g["id"]})).unwrap();
        assert!(g3.get("linked_to_plan").is_none());
        api.call("goals.retirement_save", plan(1_000_000)).unwrap();
        assert_eq!(
            api.call("goals.get", json!({"id": g["id"]})).unwrap()["target_amount"],
            g2["target_amount"]
        );
    }
}
