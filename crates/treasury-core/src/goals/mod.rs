//! Metas (reserva de emergência, viagem, quitação de dívida…) com progresso e projeção,
//! e a calculadora de reserva de emergência com os gastos reais da família.

pub mod api;
pub mod education;
#[cfg(test)]
mod education_tests;
pub mod holdings;
pub mod inss;
pub mod living;
pub mod pension_pt;
pub mod pension_us;
pub mod retirement;
pub mod sources;
#[cfg(test)]
mod sources_tests;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::categories::CategoryKind;
use crate::error::{Error, Result};
use crate::household::base_currency;
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::reports::engine::{
    self, CategoryTree, FlowQuery, Fx, Key, balances_at, complete_months_before, div_ceil,
    div_round, first_flow_month, month_dates, resolve_basis,
};
use crate::time::{fmt_date, now_utc, parse_date};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalKind {
    Emergency,
    Travel,
    DebtPayoff,
    Purchase,
    Retirement,
    Education,
    Other,
}

impl GoalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            GoalKind::Emergency => "emergency",
            GoalKind::Travel => "travel",
            GoalKind::DebtPayoff => "debt_payoff",
            GoalKind::Purchase => "purchase",
            GoalKind::Retirement => "retirement",
            GoalKind::Education => "education",
            GoalKind::Other => "other",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "emergency" => GoalKind::Emergency,
            "travel" => GoalKind::Travel,
            "debt_payoff" => GoalKind::DebtPayoff,
            "purchase" => GoalKind::Purchase,
            "retirement" => GoalKind::Retirement,
            "education" => GoalKind::Education,
            _ => GoalKind::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Goal {
    pub id: Id,
    pub name: String,
    pub kind: GoalKind,
    #[ts(type = "number")]
    pub target_amount: i64,
    pub currency: String,
    pub target_date: Option<String>,
    /// Valor já guardado fora das contas vinculadas.
    #[ts(type = "number")]
    pub initial_amount: i64,
    #[ts(type = "number | null")]
    pub monthly_contribution: Option<i64>,
    pub account_ids: Vec<Id>,
    pub archived: bool,
    pub notes: Option<String>,
    /// Retorno real esperado ao ano (acima da inflação; 0.03 = 3%). Com ele, o alvo fica em
    /// dinheiro de hoje e o aporte e a projeção contam o rendimento; sem ele, a conta linear.
    #[ts(type = "number | null")]
    pub real_return: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalView {
    #[serde(flatten)]
    pub goal: Goal,
    /// initial_amount + saldo das contas vinculadas (na moeda da meta).
    #[ts(type = "number")]
    pub current_amount: i64,
    /// 0–1 (pode passar de 1).
    pub progress: f64,
    #[ts(type = "number")]
    pub remaining: i64,
    /// Aporte mensal necessário para chegar na data-alvo.
    #[ts(type = "number | null")]
    pub monthly_needed: Option<i64>,
    /// Média de aportes líquidos nas contas vinculadas nos últimos 6 meses.
    #[ts(type = "number")]
    pub avg_monthly_contribution: i64,
    /// Data projetada pelo ritmo atual (ou pelo aporte planejado, sem ritmo positivo).
    pub projected_date: Option<String>,
    /// Chega até a data-alvo no ritmo projetado (null sem data-alvo).
    pub on_track: Option<bool>,
    /// Moedas de contas vinculadas sem câmbio (saldos fora do valor atual).
    pub missing_fx: Vec<String>,
    pub estimated: bool,
    /// Meta ligada ao plano de aposentadoria: o alvo vem do plano e, sem contas vinculadas,
    /// o valor atual é a carteira.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub linked_to_plan: Option<bool>,
    /// Soma dos lançamentos vinculados à meta (já dentro do valor atual), quando há algum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub linked_deposits: Option<i64>,
    /// Quanto as posições ligadas valem para a meta hoje (já dentro do valor atual).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub holdings_value: Option<i64>,
    /// Meta de faculdade: o plano resumido (o alvo, a data e o aporte vêm dele).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub education: Option<education::EducationBrief>,
    /// Meta com rendimento (`real_return`): a conta com juros e a inflação.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub growth: Option<GoalGrowth>,
    /// As fontes ligadas (contas, posições e etiquetas).
    pub sources: Vec<sources::GoalSourceRef>,
    /// Quanto cada fonte vale para a meta hoje e qual valor entrou (o valor atual é a soma).
    pub breakdown: Vec<sources::GoalSourceValue>,
    /// Fontes que podem estar desatualizadas (conta sem extrato recente, ativo sem cotação).
    pub stale_sources: Vec<sources::GoalSourceValue>,
}

/// A conta da meta com rendimento, em dinheiro de hoje (e o alvo na moeda da época).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalGrowth {
    /// Retorno real ao ano usado (premissa da pessoa).
    pub real_return: f64,
    /// Inflação ao ano usada para o alvo na moeda da época (média dos últimos anos da série
    /// do país; `None` sem dados).
    pub inflation: Option<f64>,
    /// Meses até a data-alvo (0 sem data ou com a data passada).
    pub months: u32,
    /// Quanto entra de aportes até a data com o aporte necessário (o guardado hoje incluso).
    #[ts(type = "number | null")]
    pub from_contributions: Option<i64>,
    /// Quanto vem do rendimento até a data (alvo − aportes).
    #[ts(type = "number | null")]
    pub from_returns: Option<i64>,
    /// O alvo corrigido pela inflação até a data (dinheiro da época).
    #[ts(type = "number | null")]
    pub target_nominal: Option<i64>,
    /// Aporte mensal da conta linear (restante ÷ meses), para comparar.
    #[ts(type = "number | null")]
    pub linear_monthly_needed: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalListArgs {
    #[ts(optional)]
    pub include_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalInput {
    pub name: String,
    pub kind: GoalKind,
    #[ts(type = "number")]
    pub target_amount: i64,
    #[ts(optional)]
    pub currency: Option<String>,
    #[ts(optional)]
    pub target_date: Option<String>,
    #[ts(optional, type = "number")]
    pub initial_amount: Option<i64>,
    #[ts(optional, type = "number")]
    pub monthly_contribution: Option<i64>,
    #[ts(optional)]
    pub account_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub notes: Option<String>,
    /// Retorno real esperado ao ano (0.03 = 3%); vazio: conta linear.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub real_return: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub kind: Option<GoalKind>,
    #[ts(optional, type = "number")]
    pub target_amount: Option<i64>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub target_date: Option<Option<String>>,
    #[ts(optional, type = "number")]
    pub initial_amount: Option<i64>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub monthly_contribution: Option<Option<i64>>,
    #[ts(optional)]
    pub account_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub archived: Option<bool>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
    /// Retorno real esperado ao ano; `null` volta para a conta linear.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub real_return: Option<Option<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmergencyReserveArgs {
    /// Meses de cobertura desejados (padrão 6).
    #[ts(optional)]
    pub target_months: Option<u32>,
    /// Meses analisados (padrão 6, só meses completos).
    #[ts(optional)]
    pub months_back: Option<u32>,
    /// Contar os gastos anuais (IPVA, IPTU, material escolar) pela provisão mensal, no lugar
    /// dos picos (padrão: sim, quando há gastos anuais detectados).
    #[serde(default)]
    #[ts(optional)]
    pub include_annual: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmergencyReserve {
    /// Média mensal das despesas em categorias essenciais.
    #[ts(type = "number")]
    pub avg_essential_monthly: i64,
    /// Média mensal dos essenciais sem as categorias de gasto anual.
    #[ts(type = "number")]
    pub avg_essential_regular_monthly: i64,
    /// Custo mensal usado na conta: os essenciais do mês mais a provisão dos gastos anuais
    /// essenciais (com `include_annual`), ou a média dos essenciais.
    #[ts(type = "number")]
    pub monthly_cost: i64,
    /// Os gastos anuais pelos últimos 12 meses (provisão mensal).
    pub annual: living::AnnualExpenses,
    /// A provisão dos gastos anuais entrou no custo mensal.
    pub include_annual: bool,
    #[ts(type = "number")]
    pub avg_total_expense_monthly: i64,
    pub months_analyzed: u32,
    pub target_months: u32,
    #[ts(type = "number")]
    pub target_amount: i64,
    /// Valor atual da meta de reserva (ou das contas correntes/poupança, se não houver meta).
    #[ts(type = "number")]
    pub current_amount: i64,
    /// current_amount ÷ monthly_cost.
    pub coverage_months: Option<f64>,
    pub goal_id: Option<Id>,
    pub currency: String,
    /// Meses analisados (`YYYY-MM`).
    pub months: Vec<String>,
    pub basis: crate::budget::Basis,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

/// Meses usados na média de aportes das contas vinculadas.
pub const CONTRIBUTION_MONTHS: u32 = 6;

fn row_to_goal(r: &rusqlite::Row<'_>) -> rusqlite::Result<Goal> {
    Ok(Goal {
        id: r.get(0)?,
        name: r.get(1)?,
        kind: GoalKind::parse(&r.get::<_, String>(2)?),
        target_amount: r.get(3)?,
        currency: r.get(4)?,
        target_date: r.get(5)?,
        initial_amount: r.get(6)?,
        monthly_contribution: r.get(7)?,
        account_ids: vec![],
        archived: r.get::<_, i64>(8)? != 0,
        notes: r.get(9)?,
        real_return: r.get(10)?,
    })
}

const GOAL_COLUMNS: &str = "id, name, kind, target_amount, currency, target_date, initial_amount,
    monthly_contribution, archived, notes, real_return";

fn account_ids(conn: &Connection, goal_id: &str) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare_cached(
        "SELECT account_id FROM goal_account WHERE goal_id = ?1 ORDER BY account_id",
    )?;
    let rows = stmt
        .query_map([goal_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn load(conn: &Connection, id: &str) -> Result<Goal> {
    let mut g = conn
        .query_row(
            &format!("SELECT {GOAL_COLUMNS} FROM goal WHERE id = ?1"),
            [id],
            row_to_goal,
        )
        .optional()?
        .ok_or_else(|| Error::not_found(goal_ref(id)))?;
    g.account_ids = account_ids(conn, id)?;
    Ok(g)
}

pub fn list(conn: &Connection, include_archived: bool) -> Result<Vec<GoalView>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {GOAL_COLUMNS} FROM goal WHERE ?1 OR archived = 0 ORDER BY archived, created_at, id"
    ))?;
    let goals = stmt
        .query_map([include_archived], row_to_goal)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let today = engine::today();
    goals
        .into_iter()
        .map(|mut g| {
            g.account_ids = account_ids(conn, &g.id)?;
            view(conn, g, today)
        })
        .collect()
}

pub fn get(conn: &Connection, id: &str) -> Result<GoalView> {
    view(conn, load(conn, id)?, engine::today())
}

/// "meta <id>", para o erro de não encontrado.
fn goal_ref(id: &str) -> String {
    t_args(
        "nucleo-orcamento-meta",
        &args(&[("id", id.to_string().into())]),
    )
}

fn validate_name(name: &str) -> Result<String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-sem-nome")));
    }
    Ok(n.to_string())
}

fn validate_amounts(target: i64, initial: i64, monthly: Option<i64>) -> Result<()> {
    if target <= 0 {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-alvo")));
    }
    if initial < 0 {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-guardado")));
    }
    if monthly.is_some_and(|m| m < 0) {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-aporte")));
    }
    Ok(())
}

/// Retorno real aceito: de −5% a 15% ao ano.
fn validate_return(r: Option<f64>) -> Result<()> {
    if r.is_some_and(|r| !r.is_finite() || !(-0.05..=0.15).contains(&r)) {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-retorno")));
    }
    Ok(())
}

fn clean_accounts(conn: &Connection, ids: &[Id]) -> Result<Vec<Id>> {
    let set: BTreeSet<&Id> = ids.iter().collect();
    for id in &set {
        let exists: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM account WHERE id = ?1)",
            [id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(Error::not_found(t_args(
                "nucleo-orcamento-conta",
                &args(&[("id", id.to_string().into())]),
            )));
        }
    }
    Ok(set.into_iter().cloned().collect())
}

fn replace_accounts(tx: &Transaction<'_>, goal_id: &str, ids: &[Id]) -> Result<()> {
    let current = account_ids(tx, goal_id)?;
    for old in current.iter().filter(|c| !ids.contains(c)) {
        tx.execute(
            "DELETE FROM goal_account WHERE goal_id = ?1 AND account_id = ?2",
            params![goal_id, old],
        )?;
    }
    for new in ids.iter().filter(|i| !current.contains(i)) {
        tx.execute(
            "INSERT INTO goal_account (goal_id, account_id) VALUES (?1, ?2)",
            params![goal_id, new],
        )?;
    }
    Ok(())
}

pub fn create(tx: &Transaction<'_>, input: &GoalInput) -> Result<Id> {
    let name = validate_name(&input.name)?;
    let currency = match &input.currency {
        Some(c) => crate::money::validate_currency(c)?,
        None => base_currency(tx)?,
    };
    if let Some(d) = &input.target_date {
        parse_date(d)?;
    }
    let initial = input.initial_amount.unwrap_or(0);
    validate_amounts(input.target_amount, initial, input.monthly_contribution)?;
    validate_return(input.real_return)?;
    let accounts = clean_accounts(tx, input.account_ids.as_deref().unwrap_or(&[]))?;
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO goal (id, name, kind, target_amount, currency, target_date, initial_amount,
                           monthly_contribution, archived, notes, real_return, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?11, ?10, ?10)",
        params![
            id,
            name,
            input.kind.as_str(),
            input.target_amount,
            currency,
            input.target_date,
            initial,
            input.monthly_contribution,
            input
                .notes
                .as_deref()
                .map(str::trim)
                .filter(|n| !n.is_empty()),
            now,
            input.real_return
        ],
    )?;
    replace_accounts(tx, &id, &accounts)?;
    Ok(id)
}

pub fn update(tx: &Transaction<'_>, p: &GoalPatch) -> Result<()> {
    let old = load(tx, &p.id)?;
    let mut g = old.clone();
    if let Some(n) = &p.name {
        g.name = validate_name(n)?;
    }
    if let Some(k) = p.kind {
        g.kind = k;
    }
    if let Some(v) = p.target_amount {
        g.target_amount = v;
    }
    if let Some(d) = &p.target_date {
        if let Some(d) = d {
            parse_date(d)?;
        }
        g.target_date = d.clone();
    }
    if let Some(v) = p.initial_amount {
        g.initial_amount = v;
    }
    if let Some(v) = p.monthly_contribution {
        g.monthly_contribution = v;
    }
    if let Some(a) = p.archived {
        g.archived = a;
    }
    if let Some(n) = &p.notes {
        g.notes = n
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(str::to_string);
    }
    if let Some(ids) = &p.account_ids {
        g.account_ids = clean_accounts(tx, ids)?;
    }
    if let Some(r) = p.real_return {
        g.real_return = r;
    }
    validate_amounts(g.target_amount, g.initial_amount, g.monthly_contribution)?;
    validate_return(g.real_return)?;
    if g.account_ids != old.account_ids {
        replace_accounts(tx, &g.id, &g.account_ids)?;
    }
    let mut fields = g.clone();
    fields.account_ids = old.account_ids.clone();
    if fields != old {
        tx.execute(
            "UPDATE goal SET name = ?2, kind = ?3, target_amount = ?4, target_date = ?5, initial_amount = ?6,
                 monthly_contribution = ?7, archived = ?8, notes = ?9, updated_at = ?10, real_return = ?11
             WHERE id = ?1",
            params![
                g.id,
                g.name,
                g.kind.as_str(),
                g.target_amount,
                g.target_date,
                g.initial_amount,
                g.monthly_contribution,
                g.archived as i64,
                g.notes,
                now_utc(),
                g.real_return
            ],
        )?;
    }
    Ok(())
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    tx.execute("DELETE FROM goal_account WHERE goal_id = ?1", [id])?;
    if tx.execute("DELETE FROM goal WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(goal_ref(id)));
    }
    Ok(())
}

/// Meses inteiros entre o mês de `a` e o de `b` (negativo se `b` vier antes).
fn months_until(a: NaiveDate, b: NaiveDate) -> i32 {
    (b.year() * 12 + b.month0() as i32) - (a.year() * 12 + a.month0() as i32)
}

/// Progresso e projeção da meta em `today`.
///
/// - Valor atual = valor inicial + saldo de hoje das contas vinculadas, convertido para a
///   moeda da meta (sem câmbio, o saldo fica de fora e a moeda vai para `missing_fx`).
/// - Aporte necessário = restante ÷ meses até o mês da data-alvo (arredondado para cima;
///   todo o restante se a data já chegou).
/// - Ritmo atual = média dos aportes líquidos (entradas − saídas efetivadas) nas contas
///   vinculadas nos últimos 6 meses completos, contando só meses desde a abertura da conta
///   mais antiga. Sem ritmo positivo, a projeção usa o aporte planejado.
pub fn view(conn: &Connection, goal: Goal, today: NaiveDate) -> Result<GoalView> {
    let today_s = fmt_date(today);
    let mut fx = Fx::new(conn, &goal.currency);
    // O valor de hoje: a soma das fontes (docs/goal-sources.md).
    let value = sources::current_value(conn, &goal, today, &mut fx)?;
    let current = value.total;
    let plan_linked = goal.kind == GoalKind::Retirement
        && retirement::linked_goal(conn)?.as_deref() == Some(goal.id.as_str());
    let linked: Vec<_> = balances_at(conn, &today_s)?
        .into_iter()
        .filter(|a| goal.account_ids.contains(&a.id))
        .collect();
    let deposits = linked_deposits(conn, &goal, &today_s)?;
    let deposited = value.of(sources::GoalSourceKind::Deposits).unwrap_or(0);
    let holdings_value = value.holdings_value();
    let deposits_avg = match deposits.first() {
        None => 0,
        Some((first, ..)) => {
            let months = complete_months_before(
                &engine::current_month_of(today),
                CONTRIBUTION_MONTHS,
                Some(&first[..7]),
            )?;
            match (months.first(), months.last()) {
                (Some(a), Some(b)) => {
                    let (from, _) = month_dates(a)?;
                    let (_, to) = month_dates(b)?;
                    let mut net = 0;
                    for (date, amount, currency) in &deposits {
                        if date.as_str() >= from.as_str()
                            && date.as_str() <= to.as_str()
                            && let Some(c) = fx.convert(-amount, currency, date)?
                        {
                            net += c.amount;
                        }
                    }
                    div_round(net, months.len() as i64)
                }
                _ => 0,
            }
        }
    };
    let avg = deposits_avg
        + if linked.is_empty() {
            0
        } else {
            let first_open = linked
                .iter()
                .map(|a| &a.opening_date[..7])
                .min()
                .map(str::to_string);
            let months = complete_months_before(
                &engine::current_month_of(today),
                CONTRIBUTION_MONTHS,
                first_open.as_deref(),
            )?;
            match (months.first(), months.last()) {
                (Some(a), Some(b)) => {
                    let (from, _) = month_dates(a)?;
                    let (_, to) = month_dates(b)?;
                    let ids: Vec<&str> = linked.iter().map(|a| a.id.as_str()).collect();
                    let net = net_inflow(conn, &mut fx, &ids, &from, &to)?;
                    div_round(net, months.len() as i64)
                }
                _ => 0,
            }
        };
    // Meta de faculdade: o alvo, a data e o aporte vêm da conta do plano com o guardado de hoje.
    let mut goal = goal;
    let edu = match goal.kind {
        GoalKind::Education => match education::load(conn, &goal.id)? {
            Some(plan) => match education::result_of(conn, &plan, current, today) {
                Ok(r) => {
                    goal.target_amount = r.target_at_start.max(1);
                    goal.target_date = Some(r.start_date.clone());
                    Some(education::brief(conn, &plan, &r)?)
                }
                Err(e) => {
                    tracing::warn!("plano de educação {}: {e}", goal.id);
                    None
                }
            },
            None => None,
        },
        _ => None,
    };
    let remaining = (goal.target_amount - current).max(0);
    let target_date = goal.target_date.as_deref().map(parse_date).transpose()?;
    let monthly_needed = target_date.map(|t| {
        let left = months_until(today, t);
        if remaining == 0 {
            0
        } else if left <= 0 {
            remaining
        } else {
            div_ceil(remaining, left as i64)
        }
    });
    let pace = if avg > 0 {
        avg
    } else {
        goal.monthly_contribution.unwrap_or(0)
    };
    let linear_projected = if remaining == 0 {
        Some(today)
    } else if pace > 0 {
        let months = div_ceil(remaining, pace).min(12 * 200) as i32;
        Some(crate::time::add_months(today, months))
    } else {
        None
    };
    // Meta com rendimento (a de faculdade já tem a conta própria).
    let (monthly_needed, projected, growth) = match goal.real_return.filter(|_| edu.is_none()) {
        None => (monthly_needed, linear_projected, None),
        Some(r) => {
            let i = monthly_rate(r);
            let n = target_date.map_or(0, |t| months_until(today, t).max(0));
            let needed = target_date.map(|_| {
                if remaining == 0 {
                    0
                } else if n == 0 {
                    remaining
                } else {
                    annuity_payment(goal.target_amount, current, i, n)
                }
            });
            let projected = if remaining == 0 {
                Some(today)
            } else {
                months_to_reach(goal.target_amount, current, pace, i)
                    .map(|m| crate::time::add_months(today, m))
            };
            let inflation =
                education::inflation(conn, education::InflationMode::Headline, None, None, today)
                    .ok()
                    .and_then(|x| x.headline_annual);
            let contributions = needed.filter(|_| n > 0).map(|m| current + m * n as i64);
            let growth = GoalGrowth {
                real_return: r,
                inflation,
                months: n as u32,
                from_contributions: contributions,
                from_returns: contributions.map(|c| (goal.target_amount - c).max(0)),
                target_nominal: inflation.filter(|_| n > 0).map(|pi| {
                    (goal.target_amount as f64 * (1.0 + pi).powf(n as f64 / 12.0)).round() as i64
                }),
                linear_monthly_needed: monthly_needed,
            };
            (needed, projected, Some(growth))
        }
    };
    let on_track = target_date.map(|t| match (&edu, projected) {
        // O aporte do plano já conta com o retorno: no caminho se o ritmo cobre o aporte.
        (Some(b), _) => b.monthly_needed == 0 || pace >= b.monthly_needed,
        _ if remaining == 0 => true,
        (_, Some(p)) => p <= t,
        (_, None) => false,
    });
    let monthly_needed = match &edu {
        Some(b) => Some(b.monthly_needed),
        None => monthly_needed,
    };
    Ok(GoalView {
        current_amount: current,
        progress: current as f64 / goal.target_amount as f64,
        remaining,
        monthly_needed,
        avg_monthly_contribution: avg,
        projected_date: projected.map(fmt_date),
        on_track,
        missing_fx: fx.missing_vec(),
        estimated: fx.estimated,
        linked_to_plan: plan_linked.then_some(true),
        linked_deposits: (!deposits.is_empty()).then_some(deposited),
        holdings_value,
        education: edu,
        growth,
        sources: sources::refs(conn, &goal)?,
        stale_sources: value.stale_sources(),
        breakdown: value.breakdown,
        goal,
    })
}

/// Taxa mensal equivalente à anual (juros compostos, não ÷ 12).
fn monthly_rate(annual: f64) -> f64 {
    (1.0 + annual).powf(1.0 / 12.0) - 1.0
}

/// Aporte mensal (no fim de cada mês, por `n` meses) para `current` chegar a `target` com a
/// taxa mensal `i`; 0 quando o rendimento do que já está guardado basta.
pub(crate) fn annuity_payment(target: i64, current: i64, i: f64, n: i32) -> i64 {
    let growth = (1.0 + i).powi(n);
    let need = target as f64 - current as f64 * growth;
    if need <= 0.0 {
        return 0;
    }
    let pmt = if i.abs() < 1e-12 {
        need / n as f64
    } else {
        need * i / (growth - 1.0)
    };
    pmt.ceil() as i64
}

/// Meses até `current` (com `pace` por mês e a taxa `i`) chegar a `target`; `None` se não
/// chega em 200 anos.
pub(crate) fn months_to_reach(target: i64, current: i64, pace: i64, i: f64) -> Option<i32> {
    let mut balance = current as f64;
    for m in 1..=12 * 200 {
        balance = balance * (1.0 + i) + pace as f64;
        if balance >= target as f64 {
            return Some(m);
        }
        if pace <= 0 && balance <= 0.0 {
            return None;
        }
    }
    None
}

/// (data, valor, moeda) dos lançamentos efetivados vinculados à meta até `until`, fora das
/// contas da própria meta (essas já entram pelo saldo).
fn linked_deposits(
    conn: &Connection,
    goal: &Goal,
    until: &str,
) -> Result<Vec<(String, i64, String)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.date, t.amount, t.currency, t.account_id FROM txn_link l JOIN txn t ON t.id = l.txn_id
          WHERE l.target_kind = 'goal' AND l.target_id = ?1 AND t.date <= ?2
            AND t.status IN ('cleared', 'reconciled')
          ORDER BY t.date, t.id",
    )?;
    let rows = stmt.query_map(params![goal.id, until], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    })?;
    let mut out = vec![];
    for r in rows {
        let (d, a, c, acc) = r?;
        if !goal.account_ids.contains(&acc) {
            out.push((d, a, c));
        }
    }
    Ok(out)
}

/// Entradas − saídas efetivadas nas contas, no período, convertidas para `fx.target` na
/// data de cada lançamento.
fn net_inflow(
    conn: &Connection,
    fx: &mut Fx<'_>,
    ids: &[&str],
    from: &str,
    to: &str,
) -> Result<i64> {
    let mut p = engine::Params::default();
    let target = p.bind(fx.target.clone());
    let (a, b) = (p.bind(from.to_string()), p.bind(to.to_string()));
    let list = ids
        .iter()
        .map(|id| p.bind(id.to_string()))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT t.currency, CASE WHEN t.currency = {target} THEN NULL ELSE t.date END AS d,
                MIN(t.date), SUM(t.amount)
         FROM txn t WHERE t.account_id IN ({list}) AND t.date BETWEEN {a} AND {b}
           AND t.status IN ('cleared', 'reconciled')
         GROUP BY t.currency, d"
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(rusqlite::params_from_iter(p.0.iter()))?;
    let mut total = 0;
    while let Some(r) = rows.next()? {
        let currency: String = r.get(0)?;
        let date: String = r.get(2)?;
        let amount: i64 = r.get(3)?;
        if let Some(c) = fx.convert(amount, &currency, &date)? {
            total += c.amount;
        }
    }
    Ok(total)
}

/// Reserva de emergência com os gastos reais: média mensal das despesas em categorias
/// essenciais (`is_essential` da própria categoria) nos últimos N meses completos (padrão 6,
/// sem contar meses anteriores ao primeiro lançamento). Meta = média × meses-alvo (padrão 6).
/// Valor atual: a meta de tipo `emergency` (a mais antiga não arquivada) ou, sem ela, o saldo
/// das contas corrente, poupança e dinheiro não arquivadas que entram no patrimônio.
pub fn emergency_reserve(
    conn: &Connection,
    args: &EmergencyReserveArgs,
) -> Result<EmergencyReserve> {
    let target_months = args.target_months.unwrap_or(6).clamp(1, 120);
    let months_back = args.months_back.unwrap_or(6).clamp(1, 120);
    let basis = resolve_basis(conn, None)?;
    let base = base_currency(conn)?;
    let today = engine::today();
    let today_s = fmt_date(today);
    let mut fx = Fx::new(conn, &base);
    let months = match first_flow_month(conn)? {
        Some(first) => {
            complete_months_before(&engine::current_month_of(today), months_back, Some(&first))?
        }
        None => vec![],
    };
    let annual = living::annual_expenses(conn, &mut fx, basis, today)?;
    let annual_ids = annual.essential_ids();
    let (mut essential, mut total, mut essential_annual) = (0i64, 0i64, 0i64);
    if let (Some(a), Some(b)) = (months.first(), months.last()) {
        let (from, _) = month_dates(a)?;
        let (_, to) = month_dates(b)?;
        let mut q = FlowQuery::new(&from, &to, basis);
        q.kind = Some(CategoryKind::Expense);
        let agg = q.aggregate(conn, &mut fx, Key::Category, Key::None)?;
        let tree = CategoryTree::load(conn)?;
        for ((cat, _), cell) in &agg.cells {
            total += cell.expense;
            if cat
                .as_deref()
                .and_then(|c| tree.get(c))
                .is_some_and(|c| c.is_essential)
            {
                essential += cell.expense;
                if cat.as_deref().is_some_and(|c| annual_ids.contains(c)) {
                    essential_annual += cell.expense;
                }
            }
        }
    }
    let n = months.len() as i64;
    let avg_essential = if n > 0 { div_round(essential, n) } else { 0 };
    let avg_total = if n > 0 { div_round(total, n) } else { 0 };
    let avg_regular = if n > 0 {
        div_round(essential - essential_annual, n)
    } else {
        0
    };
    let include_annual =
        args.include_annual.unwrap_or(annual.essential_monthly > 0) && annual.complete_year;
    let monthly_cost = if include_annual {
        avg_regular + annual.essential_monthly
    } else {
        avg_essential
    };

    let goal_id: Option<Id> = conn
        .query_row(
            "SELECT id FROM goal WHERE kind = 'emergency' AND archived = 0 ORDER BY created_at, id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let current = match &goal_id {
        Some(id) => {
            let v = view(conn, load(conn, id)?, today)?;
            fx.missing.extend(v.missing_fx.iter().cloned());
            fx.estimated |= v.estimated;
            let cur = v.goal.currency.clone();
            fx.convert(v.current_amount, &cur, &today_s)?
                .map(|c| c.amount)
                .unwrap_or(0)
        }
        None => {
            let mut sum = 0;
            for a in balances_at(conn, &today_s)? {
                use crate::accounts::AccountKind::*;
                if a.archived
                    || !a.include_in_net_worth
                    || !matches!(a.kind, Checking | Savings | Cash)
                {
                    continue;
                }
                // Conta no vermelho é dívida (cheque especial), não reduz a reserva.
                if let Some(c) = fx.convert(a.balance, &a.currency, &today_s)? {
                    sum += c.amount.max(0);
                }
            }
            sum
        }
    };
    Ok(EmergencyReserve {
        avg_essential_monthly: avg_essential,
        avg_essential_regular_monthly: avg_regular,
        monthly_cost,
        annual,
        include_annual,
        avg_total_expense_monthly: avg_total,
        months_analyzed: months.len() as u32,
        target_months,
        target_amount: monthly_cost * target_months as i64,
        current_amount: current,
        coverage_months: (monthly_cost > 0).then(|| current as f64 / monthly_cost as f64),
        goal_id,
        currency: base,
        months,
        basis,
        missing_fx: fx.missing_vec(),
        estimated: fx.estimated,
    })
}
