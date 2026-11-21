//! Cenários "e se" (docs/scenarios.md): eventos de vida (comprar um imóvel, ter um filho,
//! pausar a carreira, mudar de renda) aplicados sobre o cenário base e comparados com ele,
//! mês a mês, em valores reais de hoje.
//!
//! - **Base**: renda e gasto médios dos últimos 12 meses completos (competência); o que sobra
//!   vai para o patrimônio líquido (contas do dia a dia + carteira), que rende o retorno real
//!   do plano de aposentadoria (padrão 4% a.a.) quando positivo; bens e dívidas ficam no
//!   valor de hoje.
//! - **Cenário**: o mesmo, com os eventos: entrada ou saída única, valor mensal a mais ou a
//!   menos, renda em % do normal por um período, bem comprado (com valorização real) e
//!   financiamento (Price: o saldo vira dívida e a parcela sai todo mês).
//! - **Previsão de saldo**: os próximos 12 meses da previsão de saldo (`forecast`) com o
//!   efeito em caixa dos eventos somado, e o primeiro dia negativo de cada um.
//! - **Aposentadoria**: o plano salvo simulado duas vezes (`goals::retirement::simulate`), a
//!   segunda com a diferença anual de fluxo do cenário como eventos de vida.

pub mod api;
#[cfg(test)]
mod tests;

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use fluent_bundle::FluentValue;

use crate::ai::alerts::pct;
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::time::{add_months, fmt_date, month_of, parse_date};

pub const SAVED_SETTING: &str = "scenarios.saved";
const MAX_MONTHS: u32 = 480;

/// Evento de vida. Datas `YYYY-MM-DD`; valores em centavos de hoje.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, rename = "ScenarioEvent")]
pub enum Event {
    /// Entrada (+) ou saída (−) única: entrada do imóvel, herança, festa.
    OneOff {
        label: String,
        date: String,
        #[ts(type = "number")]
        amount: i64,
    },
    /// Valor por mês a mais (+) ou a menos (−), por `months` meses (sem = para sempre):
    /// custo de um filho, aluguel que deixa de pagar, aumento.
    Monthly {
        label: String,
        start: String,
        #[ts(optional)]
        #[serde(default)]
        months: Option<u32>,
        #[ts(type = "number")]
        amount: i64,
    },
    /// Renda em `pct`% do normal (0 = pausa na carreira, 50 = meio período, 120 = aumento).
    IncomePct {
        label: String,
        start: String,
        #[ts(optional)]
        #[serde(default)]
        months: Option<u32>,
        pct: f64,
    },
    /// Bem que entra no patrimônio (o imóvel ou o carro), com valorização real a.a. em %.
    Asset {
        label: String,
        date: String,
        #[ts(type = "number")]
        value: i64,
        #[ts(optional)]
        #[serde(default)]
        real_growth_pct: Option<f64>,
    },
    /// Financiamento pela tabela Price: o saldo vira dívida e a parcela sai todo mês.
    Loan {
        label: String,
        date: String,
        #[ts(type = "number")]
        principal: i64,
        /// Juros reais ao ano em % (acima da inflação).
        annual_rate_pct: f64,
        months: u32,
    },
}

impl Event {
    pub fn label(&self) -> &str {
        match self {
            Event::OneOff { label, .. }
            | Event::Monthly { label, .. }
            | Event::IncomePct { label, .. }
            | Event::Asset { label, .. }
            | Event::Loan { label, .. } => label,
        }
    }

    fn start(&self) -> &str {
        match self {
            Event::OneOff { date, .. } | Event::Asset { date, .. } | Event::Loan { date, .. } => {
                date
            }
            Event::Monthly { start, .. } | Event::IncomePct { start, .. } => start,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScenarioArgs {
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    pub events: Vec<Event>,
    /// Anos à frente (1 a 40; padrão 10).
    #[serde(default)]
    #[ts(optional)]
    pub years: Option<u32>,
    /// Retorno real a.a. em fração (padrão: o do plano de aposentadoria ou 4%).
    #[serde(default)]
    #[ts(optional)]
    pub real_return: Option<f64>,
    /// Renda e gasto mensais de partida (padrão: a média dos últimos 12 meses).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub monthly_income: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub monthly_expense: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScenarioPoint {
    pub month: String,
    /// Contas + carteira.
    #[ts(type = "number")]
    pub liquid: i64,
    #[ts(type = "number")]
    pub net_worth: i64,
    #[ts(type = "number")]
    pub income: i64,
    #[ts(type = "number")]
    pub expense: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScenarioPath {
    pub points: Vec<ScenarioPoint>,
    /// Primeiro mês em que o patrimônio líquido fica negativo.
    pub first_short: Option<String>,
    #[ts(type = "number")]
    pub end_liquid: i64,
    #[ts(type = "number")]
    pub end_net_worth: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScenarioForecastPoint {
    pub date: String,
    #[ts(type = "number")]
    pub base: i64,
    #[ts(type = "number")]
    pub scenario: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScenarioForecast {
    pub points: Vec<ScenarioForecastPoint>,
    pub base_first_negative: Option<String>,
    pub scenario_first_negative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScenarioRetirement {
    pub base_success: f64,
    pub scenario_success: f64,
    #[ts(type = "number")]
    pub base_wealth_at_retirement: i64,
    #[ts(type = "number")]
    pub scenario_wealth_at_retirement: i64,
    #[ts(type = "number")]
    pub base_sustainable_spending: i64,
    #[ts(type = "number")]
    pub scenario_sustainable_spending: i64,
    pub retirement_age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScenarioResult {
    pub name: String,
    pub start: String,
    pub years: u32,
    pub real_return: f64,
    #[ts(type = "number")]
    pub monthly_income: i64,
    #[ts(type = "number")]
    pub monthly_expense: i64,
    pub base: ScenarioPath,
    pub scenario: ScenarioPath,
    /// Cenário − base no fim do horizonte.
    #[ts(type = "number")]
    pub net_worth_delta: i64,
    #[ts(type = "number")]
    pub liquid_delta: i64,
    pub forecast: ScenarioForecast,
    /// `None` sem plano de aposentadoria com idade (Orçamento › Aposentadoria).
    pub retirement: Option<ScenarioRetirement>,
    pub explanation: String,
    pub notes: Vec<String>,
}

/// Cenário salvo (ajuste `scenarios.saved`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SavedScenario {
    pub name: String,
    pub events: Vec<Event>,
}

pub fn saved(conn: &Connection) -> Result<Vec<SavedScenario>> {
    crate::settings::get_or(conn, SAVED_SETTING, Vec::new())
}

/// Ponto de partida medido no banco.
struct Start {
    liquid: i64,
    others: i64,
    income: i64,
    expense: i64,
}

fn start(conn: &Connection, today: NaiveDate) -> Result<Start> {
    use crate::reports::engine::{FlowQuery, Fx, Key};
    let nw = crate::reports::net_worth(conn, &fmt_date(today))?;
    let liquid = nw.net - nw.physical + nw.debts;
    let others = nw.physical - nw.debts;
    // Últimos 12 meses completos.
    let first = add_months(today.with_day(1).expect("dia 1"), -12);
    let last = today.with_day(1).expect("dia 1") - chrono::Duration::days(1);
    let mut fx = Fx::new(conn, &nw.base_currency);
    let c = FlowQuery::new(
        &fmt_date(first),
        &fmt_date(last),
        crate::budget::Basis::Accrual,
    )
    .aggregate(conn, &mut fx, Key::None, Key::None)?
    .total();
    // Meses com movimento (cofre novo: divide só pelo que existe).
    let months: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT substr(date, 1, 7)) FROM txn
          WHERE status <> 'planned' AND date BETWEEN ?1 AND ?2",
        [fmt_date(first), fmt_date(last)],
        |r| r.get(0),
    )?;
    let m = months.max(1);
    Ok(Start {
        liquid,
        others,
        income: c.income / m,
        expense: c.expense / m,
    })
}

fn months_from(today: NaiveDate, date: &str) -> Result<i64> {
    let d = parse_date(date)?;
    Ok((d.year() - today.year()) as i64 * 12 + d.month() as i64 - today.month() as i64)
}

fn active(start: i64, months: Option<u32>, m: i64) -> bool {
    m >= start && months.is_none_or(|n| m < start + n as i64)
}

fn price_payment(principal: f64, i: f64, n: u32) -> f64 {
    if i.abs() < 1e-12 {
        return principal / n.max(1) as f64;
    }
    principal * i / (1.0 - (1.0 + i).powi(-(n.max(1) as i32)))
}

/// Efeito de um evento no mês `m` (meses depois do atual): caixa e bens/dívidas.
struct Step {
    income_factor: f64,
    cash: f64,
    asset_change: f64,
}

/// Projeção mês a mês; devolve o caminho e o fluxo de caixa de cada mês (para a
/// aposentadoria e a previsão).
fn project(
    s: &Start,
    events: &[Event],
    today: NaiveDate,
    months: u32,
    rate: f64,
) -> Result<(ScenarioPath, Vec<f64>)> {
    let monthly_rate = (1.0 + rate).powf(1.0 / 12.0) - 1.0;
    let starts: Vec<i64> = events
        .iter()
        .map(|e| months_from(today, e.start()))
        .collect::<Result<_>>()?;
    let mut liquid = s.liquid as f64;
    let mut others = s.others as f64;
    // Bens do cenário (valor, crescimento mensal) e dívidas (saldo, taxa, parcela).
    let mut assets: Vec<(f64, f64)> = vec![];
    // Dívidas do cenário: saldo, taxa mensal, parcela e o mês em que entraram.
    let mut loans: Vec<(f64, f64, f64, i64)> = vec![];
    let mut points = vec![];
    let mut flows = vec![];
    // Já negativo hoje: falta dinheiro desde o mês atual (não "acaba" no mês que vem).
    let mut first_short = (liquid < 0.0).then(|| month_of(today));
    for m in 1..=months as i64 {
        let mut step = Step {
            income_factor: 1.0,
            cash: 0.0,
            asset_change: 0.0,
        };
        for (e, &st) in events.iter().zip(&starts) {
            match e {
                Event::OneOff { amount, .. } if m == st.max(1) => step.cash += *amount as f64,
                Event::Monthly { months, amount, .. } if active(st.max(1), *months, m) => {
                    step.cash += *amount as f64
                }
                Event::IncomePct { months, pct, .. } if active(st.max(1), *months, m) => {
                    step.income_factor *= pct / 100.0
                }
                Event::Asset {
                    value,
                    real_growth_pct,
                    ..
                } if m == st.max(1) => {
                    let g = (1.0 + real_growth_pct.unwrap_or(0.0) / 100.0).powf(1.0 / 12.0) - 1.0;
                    assets.push((*value as f64, g));
                    step.asset_change += *value as f64;
                }
                Event::Loan {
                    principal,
                    annual_rate_pct,
                    months: n,
                    ..
                } if m == st.max(1) => {
                    let i = (1.0 + annual_rate_pct / 100.0).powf(1.0 / 12.0) - 1.0;
                    // O dinheiro do financiamento paga o bem: entra no caixa e vira dívida.
                    step.cash += *principal as f64;
                    loans.push((
                        *principal as f64,
                        i,
                        price_payment(*principal as f64, i, *n),
                        m,
                    ));
                    step.asset_change -= *principal as f64;
                }
                _ => {}
            }
        }
        // Parcelas das dívidas do cenário: a primeira vence no mês seguinte ao da contratação.
        let mut paid = 0.0;
        let mut amortized = 0.0;
        for l in loans.iter_mut() {
            if l.0 <= 0.5 || l.3 >= m {
                continue;
            }
            let interest = l.0 * l.1;
            let pay = l.2.min(l.0 + interest);
            l.0 = l.0 + interest - pay;
            paid += pay;
            amortized += pay - interest;
        }
        // Valorização dos bens do cenário.
        let mut grown = 0.0;
        for a in assets.iter_mut() {
            let g = a.0 * a.1;
            a.0 += g;
            grown += g;
        }
        let income = s.income as f64 * step.income_factor;
        let expense = s.expense as f64;
        let flow = income - expense + step.cash - paid;
        if liquid > 0.0 {
            liquid *= 1.0 + monthly_rate;
        }
        liquid += flow;
        others += step.asset_change + amortized + grown;
        flows.push(flow);
        if first_short.is_none() && liquid < 0.0 {
            first_short = Some(month_of(add_months(today, m as i32)));
        }
        points.push(ScenarioPoint {
            month: month_of(add_months(today, m as i32)),
            liquid: liquid.round() as i64,
            net_worth: (liquid + others).round() as i64,
            income: income.round() as i64,
            expense: (expense - step.cash.min(0.0) + paid).round() as i64,
        });
    }
    let end = points
        .last()
        .map_or((s.liquid, s.liquid + s.others), |p| (p.liquid, p.net_worth));
    Ok((
        ScenarioPath {
            points,
            first_short,
            end_liquid: end.0,
            end_net_worth: end.1,
        },
        flows,
    ))
}

/// Efeito em caixa dos eventos por data (próximos 12 meses), para somar à previsão.
fn cash_effects(
    events: &[Event],
    today: NaiveDate,
    to: NaiveDate,
    income: i64,
) -> Result<Vec<(NaiveDate, i64)>> {
    let mut out = vec![];
    for e in events {
        let d0 = parse_date(e.start())?.max(today);
        match e {
            Event::OneOff { amount, .. } if d0 <= to => out.push((d0, *amount)),
            Event::Monthly { months, amount, .. } => {
                for k in 0..months.unwrap_or(u32::MAX).min(13) {
                    let d = add_months(d0, k as i32);
                    if d > to {
                        break;
                    }
                    out.push((d, *amount));
                }
            }
            Event::IncomePct { months, pct, .. } => {
                let delta = (income as f64 * (pct / 100.0 - 1.0)).round() as i64;
                for k in 0..months.unwrap_or(u32::MAX).min(13) {
                    let d = add_months(d0, k as i32);
                    if d > to {
                        break;
                    }
                    out.push((d, delta));
                }
            }
            Event::Loan {
                principal,
                annual_rate_pct,
                months: n,
                ..
            } => {
                let i = (1.0 + annual_rate_pct / 100.0).powf(1.0 / 12.0) - 1.0;
                let pay = price_payment(*principal as f64, i, *n).round() as i64;
                // O dinheiro do financiamento entra no dia da compra (paga o bem).
                if d0 <= to {
                    out.push((d0, *principal));
                }
                for k in 1..=(*n).min(13) {
                    let d = add_months(d0, k as i32);
                    if d > to {
                        break;
                    }
                    out.push((d, -pay));
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

fn forecast(
    conn: &Connection,
    events: &[Event],
    today: NaiveDate,
    income: i64,
) -> Result<ScenarioForecast> {
    let f = crate::forecast::forecast(
        conn,
        &crate::forecast::ForecastArgs {
            months: Some(12),
            account_ids: None,
        },
        today,
    )?;
    let to = parse_date(&f.to)?;
    let mut effects = cash_effects(events, today, to, income)?;
    effects.sort();
    let mut dates: Vec<String> = f.total_points.iter().map(|p| p.date.clone()).collect();
    dates.extend(effects.iter().map(|(d, _)| fmt_date(*d)));
    dates.sort();
    dates.dedup();
    let mut points = vec![];
    let (mut base_neg, mut scen_neg) = (None, None);
    for d in dates {
        let base = f
            .total_points
            .iter()
            .take_while(|p| p.date <= d)
            .last()
            .map_or(f.total_start, |p| p.balance);
        let delta: i64 = effects
            .iter()
            .filter(|(e, _)| fmt_date(*e) <= d)
            .map(|(_, v)| v)
            .sum();
        let scenario = base + delta;
        if base_neg.is_none() && base < 0 {
            base_neg = Some(d.clone());
        }
        if scen_neg.is_none() && scenario < 0 {
            scen_neg = Some(d.clone());
        }
        points.push(ScenarioForecastPoint {
            date: d,
            base,
            scenario,
        });
    }
    Ok(ScenarioForecast {
        points,
        base_first_negative: base_neg,
        scenario_first_negative: scen_neg,
    })
}

fn retirement(
    conn: &Connection,
    base_flows: &[f64],
    scen_flows: &[f64],
) -> Result<Option<ScenarioRetirement>> {
    use crate::goals::retirement as r;
    let plan = r::saved(conn)?;
    let Some(age) = plan.current_age else {
        return Ok(None);
    };
    let wealth = r::portfolio_value(conn)?;
    let base = r::simulate(&plan, wealth)?;
    // Diferença de fluxo de cada ano do cenário como evento de vida naquela idade.
    let mut events = plan.events.clone().unwrap_or_default();
    for (y, chunk) in scen_flows.chunks(12).zip(base_flows.chunks(12)).enumerate() {
        let delta: f64 = chunk.0.iter().sum::<f64>() - chunk.1.iter().sum::<f64>();
        if delta.abs() >= 100.0 {
            events.push(r::LifeEvent {
                age: age + y as u32,
                amount: delta.round() as i64,
                label: "cenário".into(),
            });
        }
    }
    let scen = r::simulate(
        &r::Plan {
            events: Some(events),
            ..plan.clone()
        },
        wealth,
    )?;
    Ok(Some(ScenarioRetirement {
        base_success: base.success,
        scenario_success: scen.success,
        base_wealth_at_retirement: base.wealth_at_retirement,
        scenario_wealth_at_retirement: scen.wealth_at_retirement,
        base_sustainable_spending: base.sustainable_spending,
        scenario_sustainable_spending: scen.sustainable_spending,
        retirement_age: base.plan.retirement_age.unwrap_or(65),
    }))
}

/// `YYYY-MM-DD` → data no idioma da instalação (`14/10/2026`, `Oct 14, 2026`).
fn date_label(d: &str) -> String {
    parse_date(d).map_or_else(|_| d.to_string(), |d| i18n::date(i18n::current(), d))
}

/// `YYYY-MM` → `MM/YYYY` (numérico, igual em todos os idiomas).
fn numeric_month(m: &str) -> String {
    match m.split_once('-') {
        Some((y, mm)) => format!("{mm}/{y}"),
        None => m.to_string(),
    }
}

/// Erro de validação de um evento, com o nome dele.
fn event_error(key: &str, label: &str) -> String {
    t_args(key, &i18n::args(&[("evento", label.to_string().into())]))
}

fn validate(events: &[Event]) -> Result<()> {
    for e in events {
        if e.label().trim().is_empty() {
            return Err(Error::Invalid(t(
                "nucleo-planejamento-cenario-erro-evento-sem-nome",
            )));
        }
        parse_date(e.start())?;
        match e {
            Event::IncomePct { pct, .. } if !(0.0..=1000.0).contains(pct) => {
                return Err(Error::Invalid(event_error(
                    "nucleo-planejamento-cenario-erro-renda",
                    e.label(),
                )));
            }
            Event::Loan {
                principal,
                months,
                annual_rate_pct,
                ..
            } if *principal <= 0 || *months == 0 || *months > 600 || *annual_rate_pct < 0.0 => {
                return Err(Error::Invalid(event_error(
                    "nucleo-planejamento-cenario-erro-financiamento",
                    e.label(),
                )));
            }
            Event::Asset { value, .. } if *value <= 0 => {
                return Err(Error::Invalid(event_error(
                    "nucleo-planejamento-cenario-erro-bem",
                    e.label(),
                )));
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn simulate(conn: &Connection, a: &ScenarioArgs, today: NaiveDate) -> Result<ScenarioResult> {
    // Sem eventos e com o nome de um cenário salvo: os eventos dele.
    let mut owned;
    let a = if a.events.is_empty()
        && let Some(n) = &a.name
        && let Some(s) = saved(conn)?.into_iter().find(|s| &s.name == n)
    {
        owned = a.clone();
        owned.events = s.events;
        &owned
    } else {
        a
    };
    validate(&a.events)?;
    let years = a.years.unwrap_or(10).clamp(1, 40);
    let months = (years * 12).min(MAX_MONTHS);
    let plan = crate::goals::retirement::saved(conn)?;
    let rate = a.real_return.or(plan.real_return).unwrap_or(0.04);
    let mut s = start(conn, today)?;
    if let Some(i) = a.monthly_income {
        s.income = i;
    }
    if let Some(e) = a.monthly_expense {
        s.expense = e;
    }
    let (base, base_flows) = project(&s, &[], today, months, rate)?;
    let (scenario, scen_flows) = project(&s, &a.events, today, months, rate)?;
    let forecast = forecast(conn, &a.events, today, s.income)?;
    let retirement = retirement(conn, &base_flows, &scen_flows)?;
    let l = i18n::current();
    let cur = crate::household::base_currency(conn)?;
    let money = |c: i64| -> FluentValue<'static> { i18n::money(l, c, &cur).into() };
    let nw_delta = scenario.end_net_worth - base.end_net_worth;
    let liq_delta = scenario.end_liquid - base.end_liquid;
    let mut explanation = t_args(
        "nucleo-planejamento-cenario-explicacao",
        &i18n::args(&[
            ("anos", years.into()),
            ("cenario", money(scenario.end_net_worth)),
            ("base", money(base.end_net_worth)),
            ("diferenca", money(nw_delta)),
            ("liquido", money(liq_delta)),
        ]),
    );
    let mut sentence = |key: &str, a: &[(&'static str, FluentValue<'static>)]| {
        explanation.push(' ');
        explanation.push_str(&t_args(key, &i18n::args(a)));
    };
    match &scenario.first_short {
        Some(m) if *m == month_of(today) => {
            sentence("nucleo-planejamento-cenario-negativo-hoje", &[]);
        }
        Some(m) => sentence(
            "nucleo-planejamento-cenario-dinheiro-acaba",
            &[("mes", numeric_month(m).into())],
        ),
        None => {}
    }
    if let Some(d) = &forecast.scenario_first_negative
        && forecast.base_first_negative.as_deref() != Some(d.as_str())
    {
        sentence(
            "nucleo-planejamento-cenario-saldo-negativo",
            &[("data", date_label(d).into())],
        );
    }
    if let Some(r) = &retirement {
        sentence(
            "nucleo-planejamento-cenario-aposentadoria",
            &[
                ("base", pct(l, r.base_success, 0).into()),
                ("cenario", pct(l, r.scenario_success, 0).into()),
            ],
        );
    }
    let mut notes = vec![
        t_args(
            "nucleo-planejamento-cenario-nota-premissas",
            &i18n::args(&[
                ("renda", money(s.income)),
                ("gasto", money(s.expense)),
                ("retorno", pct(l, rate, 1).into()),
            ]),
        ),
        t("nucleo-planejamento-cenario-nota-bens"),
    ];
    if retirement.is_none() {
        notes.push(t("nucleo-planejamento-cenario-nota-sem-idade"));
    }
    Ok(ScenarioResult {
        name: a
            .name
            .clone()
            .unwrap_or_else(|| t("nucleo-planejamento-cenario-nome")),
        start: month_of(today),
        years,
        real_return: rate,
        monthly_income: s.income,
        monthly_expense: s.expense,
        base,
        scenario,
        net_worth_delta: nw_delta,
        liquid_delta: liq_delta,
        forecast,
        retirement,
        explanation,
        notes,
    })
}
