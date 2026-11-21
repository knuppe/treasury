//! Plano de educação (meta de faculdade dos filhos, docs/design/college-goal.md).
//!
//! A família diz qual faculdade quer pagar (tipo, duração, mensalidade de hoje) e o plano faz a
//! conta em dinheiro de hoje: cada mensalidade sobe, em degrau a cada ano letivo, pela diferença
//! entre a inflação da educação e a inflação geral (média geométrica dos últimos 10 anos
//! completos das séries que o app já baixa); o alvo é o valor presente dos pagamentos no início
//! do curso, descontados pelo retorno real que a família supõe; o aporte por mês sai da fórmula
//! da anuidade (corrigido pela inflação uma vez por ano, ou fixo em reais, se a família
//! preferir). A meta `education` ligada ao plano recebe o alvo, a data e o aporte, como a meta da
//! aposentadoria. O app não sugere produto, título nem prazo.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{self, args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{add_months, fmt_date, now_utc, parse_date};

/// Retorno real suposto padrão (a.a.): abaixo dos 4% da aposentadoria, porque o prazo é menor e
/// o dinheiro precisa estar disponível numa data certa.
pub const DEFAULT_REAL_RETURN: f64 = 0.03;
/// Inflação geral suposta quando não há série com anos completos (só para os valores em
/// dinheiro da época e para o aporte fixo).
pub const DEFAULT_HEADLINE: f64 = 0.04;
/// Anos completos das séries na média da inflação da educação.
pub const SPREAD_YEARS: u32 = 10;
/// Mudança do aporte (fração) que vira aviso no recálculo anual.
pub const REVIEW_ALERT_CHANGE: f64 = 0.10;
/// Mês do recálculo anual (abril: depois do IPCA de fevereiro, que traz o reajuste das
/// mensalidades).
pub const REVIEW_MONTH: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InstitutionType {
    Private,
    Public,
    Abroad,
}

impl InstitutionType {
    pub fn as_str(self) -> &'static str {
        match self {
            InstitutionType::Private => "private",
            InstitutionType::Public => "public",
            InstitutionType::Abroad => "abroad",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "public" => InstitutionType::Public,
            "abroad" => InstitutionType::Abroad,
            _ => InstitutionType::Private,
        }
    }
}

/// Como o plano chega à inflação da educação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum InflationMode {
    /// Média dos últimos 10 anos completos da série de educação contra a inflação geral.
    #[serde(rename = "series_10y")]
    Series10y,
    /// Só a inflação geral (diferença zero).
    #[serde(rename = "headline")]
    Headline,
    /// Diferença fixa digitada pela família.
    #[serde(rename = "fixed_spread")]
    FixedSpread,
}

impl InflationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            InflationMode::Series10y => "series_10y",
            InflationMode::Headline => "headline",
            InflationMode::FixedSpread => "fixed_spread",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "headline" => InflationMode::Headline,
            "fixed_spread" => InflationMode::FixedSpread,
            _ => InflationMode::Series10y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ContributionMode {
    /// Em dinheiro de hoje: na prática, sobe com a inflação uma vez por ano.
    Indexed,
    /// O mesmo valor em reais por todo o prazo (começa mais alto).
    Flat,
}

impl ContributionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ContributionMode::Indexed => "indexed",
            ContributionMode::Flat => "flat",
        }
    }
    fn parse(s: &str) -> Self {
        if s == "flat" {
            ContributionMode::Flat
        } else {
            ContributionMode::Indexed
        }
    }
}

/// Como um pote comum (conta ou ativo ligado a várias metas sem fração) se divide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PoolRule {
    /// Quem começa primeiro recebe primeiro (o prazo dele é o mais curto).
    EarliestFirst,
    /// Proporcional ao valor necessário de cada meta.
    Proportional,
}

impl PoolRule {
    pub fn as_str(self) -> &'static str {
        match self {
            PoolRule::EarliestFirst => "earliest_first",
            PoolRule::Proportional => "proportional",
        }
    }
    pub fn parse(s: &str) -> Self {
        if s == "proportional" {
            PoolRule::Proportional
        } else {
            PoolRule::EarliestFirst
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ReviewReason {
    Annual,
    Edit,
    FeeUpdate,
    Created,
}

impl ReviewReason {
    pub fn as_str(self) -> &'static str {
        match self {
            ReviewReason::Annual => "annual",
            ReviewReason::Edit => "edit",
            ReviewReason::FeeUpdate => "fee_update",
            ReviewReason::Created => "created",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "annual" => ReviewReason::Annual,
            "fee_update" => ReviewReason::FeeUpdate,
            "created" => ReviewReason::Created,
            _ => ReviewReason::Edit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EducationPhase {
    /// Antes do curso: acumulando.
    Saving,
    /// Durante o curso: pagando com o que foi guardado.
    Paying,
    /// O curso terminou.
    Finished,
}

/// O plano como a família o define. Valores em centavos na moeda da meta, a preços de hoje.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct EducationPlan {
    /// O filho (membro da família); sem ele, `expected_birth`.
    pub member_id: Option<Id>,
    /// Filho que ainda vai nascer: data prevista (`YYYY-MM-DD`).
    pub expected_birth: Option<String>,
    /// Idade de início do curso.
    pub start_age: u32,
    /// Mês de início do ano letivo (1–12; fevereiro no Brasil).
    pub start_month: u32,
    /// Início do curso (`YYYY-MM-DD`), por cima da idade e do mês.
    pub start_date: Option<String>,
    pub institution_type: InstitutionType,
    /// Curso (texto livre, só para lembrar).
    pub course: Option<String>,
    pub duration_months: u32,
    /// Parcelas por ano (1, 2, 4 ou 12).
    pub payments_per_year: u32,
    /// Mensalidade (ou anuidade, semestre) de hoje, por parcela.
    #[ts(type = "number")]
    pub fee: i64,
    /// Data da cotação da mensalidade (`YYYY-MM-DD`).
    pub fee_date: String,
    /// Gastos extras por ano (material, matrícula).
    #[ts(type = "number")]
    pub yearly_extras: i64,
    /// Custo de vida por mês durante o curso (moradia, transporte, alimentação em outra cidade).
    #[ts(type = "number")]
    pub monthly_living: i64,
    /// Bolsa ou desconto sobre a mensalidade (0–100).
    pub discount_pct: f64,
    pub inflation_mode: InflationMode,
    /// Série de inflação da educação (sem ela, a do país).
    pub inflation_series: Option<String>,
    /// Diferença fixa sobre a inflação geral (fração a.a.), só com `fixed_spread`.
    pub fixed_spread: Option<f64>,
    /// Retorno acima da inflação suposto para o dinheiro guardado (fração a.a.).
    pub real_return: f64,
    pub contribution_mode: ContributionMode,
    pub pool_rule: PoolRule,
}

impl Default for EducationPlan {
    fn default() -> Self {
        Self {
            member_id: None,
            expected_birth: None,
            start_age: 18,
            start_month: 2,
            start_date: None,
            institution_type: InstitutionType::Private,
            course: None,
            duration_months: 48,
            payments_per_year: 12,
            fee: 0,
            fee_date: fmt_date(crate::time::today_local()),
            yearly_extras: 0,
            monthly_living: 0,
            discount_pct: 0.0,
            inflation_mode: InflationMode::Series10y,
            inflation_series: None,
            fixed_spread: None,
            real_return: DEFAULT_REAL_RETURN,
            contribution_mode: ContributionMode::Indexed,
            pool_rule: PoolRule::EarliestFirst,
        }
    }
}

/// Inflação da educação usada na conta: a diferença sobre a inflação geral e de onde ela veio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationInflation {
    pub country: String,
    pub mode: InflationMode,
    /// Série de educação (ex.: `ipca_g8`) e a da inflação geral (ex.: `ipca`).
    pub series_id: Option<String>,
    pub series_name: Option<String>,
    pub headline_id: Option<String>,
    pub headline_name: Option<String>,
    /// Diferença ao ano: (1 + educação) ÷ (1 + geral) − 1.
    pub spread: f64,
    /// Médias geométricas ao ano no período (`null` sem anos completos).
    pub education_annual: Option<f64>,
    pub headline_annual: Option<f64>,
    /// Período usado (`YYYY-MM`).
    pub from: Option<String>,
    pub to: Option<String>,
    pub years: u32,
    /// Último mês com dado das duas séries.
    pub last_data: Option<String>,
    /// Sem série de educação ou sem anos completos: a diferença fica 0.
    pub missing: bool,
}

/// Uma linha da tabela ano a ano.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationYear {
    pub year: i32,
    /// Ano com mensalidade a pagar.
    pub in_course: bool,
    /// Mensalidade (por parcela, com o desconto) em dinheiro de hoje.
    #[ts(type = "number")]
    pub fee_real: i64,
    /// A mesma em dinheiro da época (com a inflação geral suposta).
    #[ts(type = "number")]
    pub fee_nominal: i64,
    /// Soma dos pagamentos do ano, em dinheiro de hoje.
    #[ts(type = "number")]
    pub payments_real: i64,
    /// Saldo no começo do ano seguindo o plano, em dinheiro de hoje.
    #[ts(type = "number")]
    pub planned_balance: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationResult {
    pub birth_date: Option<String>,
    pub child_age: Option<u32>,
    pub start_date: String,
    /// Último mês do curso (`YYYY-MM-DD`, dia 1).
    pub end_date: String,
    pub months_to_start: i32,
    pub phase: EducationPhase,
    pub inflation: EducationInflation,
    /// Inflação geral suposta (a.a.) para os valores em dinheiro da época e o aporte fixo.
    pub headline_assumed: f64,
    /// A inflação geral suposta é o padrão (sem série com anos completos).
    pub headline_default: bool,
    /// Mensalidade no início do curso (com o desconto), em dinheiro de hoje e da época.
    #[ts(type = "number")]
    pub fee_at_start: i64,
    #[ts(type = "number")]
    pub fee_at_start_nominal: i64,
    /// Soma de todos os pagamentos, em dinheiro de hoje.
    #[ts(type = "number")]
    pub total_payments: i64,
    /// Valor necessário no início do curso (valor presente dos pagamentos), dinheiro de hoje.
    #[ts(type = "number")]
    pub target_at_start: i64,
    /// Já guardado (o valor atual da meta).
    #[ts(type = "number")]
    pub saved: i64,
    /// Aporte por mês pelo modo escolhido.
    #[ts(type = "number")]
    pub monthly_needed: i64,
    /// Em dinheiro de hoje (sobe com a inflação uma vez por ano).
    #[ts(type = "number")]
    pub monthly_indexed: i64,
    /// Fixo em reais por todo o prazo.
    #[ts(type = "number")]
    pub monthly_flat: i64,
    /// O aporte se a educação subisse só com a inflação geral (para comparar).
    #[ts(type = "number")]
    pub monthly_headline_only: i64,
    /// O aporte corrigido, em dinheiro do ano que vem.
    #[ts(type = "number")]
    pub monthly_next_year: i64,
    /// Durante o curso: o ano (1, 2…) do primeiro pagamento que o guardado não cobre;
    /// `null` se cobre tudo (ou antes do curso).
    pub uncovered_year: Option<u32>,
    /// O que falta (no início do curso, ou hoje durante ele), em dinheiro de hoje.
    #[ts(type = "number")]
    pub shortfall: i64,
    pub years: Vec<EducationYear>,
    pub notes: Vec<String>,
}

/// Um recálculo gravado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationReview {
    pub id: Id,
    pub reviewed_at: String,
    pub reason: ReviewReason,
    pub spread: f64,
    pub spread_from: Option<String>,
    pub spread_to: Option<String>,
    #[ts(type = "number")]
    pub fee: i64,
    #[ts(type = "number")]
    pub target_at_start: i64,
    #[ts(type = "number")]
    pub saved: i64,
    #[ts(type = "number")]
    pub monthly_needed: i64,
}

/// O plano com a conta de hoje, as posições ligadas e o histórico.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationPlanView {
    pub goal: super::GoalView,
    pub plan: EducationPlan,
    pub child_name: Option<String>,
    pub result: EducationResult,
    pub holdings: Vec<super::holdings::HoldingView>,
    pub reviews: Vec<EducationReview>,
}

/// Resumo de um plano para a lista de metas e o agente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationBrief {
    pub member_id: Option<Id>,
    pub child_name: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub months_to_start: i32,
    pub phase: EducationPhase,
    pub institution_type: InstitutionType,
    pub course: Option<String>,
    pub spread: f64,
    #[ts(type = "number")]
    pub target_at_start: i64,
    #[ts(type = "number")]
    pub monthly_needed: i64,
}

/// "Faculdade dos filhos": os planos somados.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationOverview {
    pub plans: Vec<EducationOverviewItem>,
    /// Soma dos aportes por mês (só metas na moeda base).
    #[ts(type = "number")]
    pub monthly_total: i64,
    #[ts(type = "number")]
    pub saved_total: i64,
    #[ts(type = "number")]
    pub target_total: i64,
    pub currency: String,
    /// Anos em que dois ou mais cursos pagam ao mesmo tempo.
    pub overlap_years: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationOverviewItem {
    pub goal_id: Id,
    pub name: String,
    pub currency: String,
    pub brief: EducationBrief,
    #[ts(type = "number")]
    pub saved: i64,
}

/// Criar ou alterar a meta de faculdade (a meta `education` e o plano).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EducationSaveArgs {
    /// Sem ele, cria a meta.
    #[ts(optional)]
    pub goal_id: Option<Id>,
    /// Nome da meta (padrão: "Faculdade de <filho>").
    #[ts(optional)]
    pub name: Option<String>,
    pub plan: EducationPlan,
    /// Já guardado fora das contas e posições ligadas.
    #[ts(optional, type = "number")]
    pub initial_amount: Option<i64>,
    #[ts(optional)]
    pub currency: Option<String>,
}

/// Simulação sem gravar: o plano (os campos informados por cima do plano da meta, ou do
/// padrão) e o que já está guardado.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct EducationSimulateArgs {
    /// Os campos do plano a usar (a interface manda o plano inteiro; o MCP, só o que muda).
    #[ts(optional, as = "Option<EducationPlan>")]
    pub plan: Option<serde_json::Value>,
    /// Já guardado (dinheiro de hoje); com `goal_id`, o valor atual da meta.
    #[ts(optional, type = "number")]
    pub saved: Option<i64>,
    #[ts(optional)]
    pub goal_id: Option<Id>,
    /// Data de nascimento por cima da do membro (para o "e se").
    #[ts(optional)]
    pub birth_date: Option<String>,
}

// ---------------------------------------------------------------------------
// Inflação da educação

/// Séries de educação e de inflação geral do país de referência: (educação, geral).
fn country_series(conn: &Connection) -> Result<(String, Option<String>, Option<String>)> {
    let (country, _) = crate::indices::catalog::reference_country(conn)?;
    let education: Option<String> = conn
        .query_row(
            "SELECT cs.series_id FROM country_series cs JOIN index_series s ON s.id = cs.series_id
              WHERE cs.country = ?1 AND cs.role = 'education_inflation' AND s.kind = 'monthly_rate'
              ORDER BY cs.priority LIMIT 1",
            [&country],
            |r| r.get(0),
        )
        .optional()?;
    let headline = crate::indices::catalog::inflation_series(conn)?
        .into_iter()
        .find(|(_, kind)| kind == "monthly_rate")
        .map(|(id, _)| id);
    Ok((country, education, headline))
}

fn series_name(conn: &Connection, id: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT name FROM index_series WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?)
}

/// Fator de cada ano completo e o último mês com dado (ano, mês).
type YearFactors = (BTreeMap<i32, f64>, Option<(i32, u32)>);

/// Variação de cada ano completo (12 meses com dado) da série mensal, de `first` a `last`.
fn annual_factors(conn: &Connection, series: &str, first: i32, last: i32) -> Result<YearFactors> {
    use rust_decimal::prelude::ToPrimitive;
    let from = NaiveDate::from_ymd_opt(first, 1, 1).expect("data válida");
    let to = NaiveDate::from_ymd_opt(last, 12, 31).expect("data válida");
    let mut months: BTreeMap<i32, (u32, f64)> = BTreeMap::new();
    let mut latest = None;
    for r in crate::indices::monthly_rates(conn, series, from, to)? {
        let e = months.entry(r.year).or_insert((0, 1.0));
        e.0 += 1;
        e.1 *= 1.0 + r.pct.to_f64().unwrap_or(0.0) / 100.0;
        if latest.is_none_or(|l| (r.year, r.month) > l) {
            latest = Some((r.year, r.month));
        }
    }
    Ok((
        months
            .into_iter()
            .filter(|(_, (n, _))| *n >= 12)
            .map(|(y, (_, f))| (y, f))
            .collect(),
        latest,
    ))
}

/// A inflação da educação do plano: pela série (média dos últimos anos completos), só a geral
/// ou a diferença fixa.
pub fn inflation(
    conn: &Connection,
    mode: InflationMode,
    series_override: Option<&str>,
    fixed: Option<f64>,
    today: NaiveDate,
) -> Result<EducationInflation> {
    let (country, education, headline) = country_series(conn)?;
    let education = series_override.map(str::to_string).or(education);
    let mut out = EducationInflation {
        country,
        mode,
        series_name: match &education {
            Some(id) => series_name(conn, id)?,
            None => None,
        },
        headline_name: match &headline {
            Some(id) => series_name(conn, id)?,
            None => None,
        },
        series_id: education.clone(),
        headline_id: headline.clone(),
        spread: 0.0,
        education_annual: None,
        headline_annual: None,
        from: None,
        to: None,
        years: 0,
        last_data: None,
        missing: false,
    };
    // A inflação geral média também serve aos valores em dinheiro da época.
    let last = today.year() - 1;
    let first = last - SPREAD_YEARS as i32 - 2;
    let (head, head_latest) = match &headline {
        Some(h) => annual_factors(conn, h, first, last)?,
        None => (BTreeMap::new(), None),
    };
    let (edu, edu_latest) = match &education {
        Some(e) => annual_factors(conn, e, first, last)?,
        None => (BTreeMap::new(), None),
    };
    out.last_data = match (edu_latest, head_latest) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
    .map(|(y, m)| format!("{y:04}-{m:02}"));
    // Os últimos anos completos e seguidos das duas séries.
    let mut years: Vec<i32> = vec![];
    let mut y = last;
    while years.len() < SPREAD_YEARS as usize && head.contains_key(&y) && edu.contains_key(&y) {
        years.push(y);
        y -= 1;
    }
    let head_years: Vec<i32> = {
        let mut v = vec![];
        let mut y = last;
        while v.len() < SPREAD_YEARS as usize && head.contains_key(&y) {
            v.push(y);
            y -= 1;
        }
        v
    };
    if !head_years.is_empty() {
        let f: f64 = head_years.iter().map(|y| head[y]).product();
        out.headline_annual = Some(f.powf(1.0 / head_years.len() as f64) - 1.0);
    }
    match mode {
        InflationMode::Headline => {}
        InflationMode::FixedSpread => out.spread = fixed.unwrap_or(0.0),
        InflationMode::Series10y => {
            if years.is_empty() {
                out.missing = true;
            } else {
                let n = years.len() as f64;
                let e: f64 = years.iter().map(|y| edu[y]).product();
                let h: f64 = years.iter().map(|y| head[y]).product();
                out.education_annual = Some(e.powf(1.0 / n) - 1.0);
                out.headline_annual = Some(h.powf(1.0 / n) - 1.0);
                out.spread = (e / h).powf(1.0 / n) - 1.0;
                out.years = years.len() as u32;
                out.from = years.last().map(|y| format!("{y:04}-01"));
                out.to = years.first().map(|y| format!("{y:04}-12"));
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// A conta

/// Meses inteiros do mês de `a` ao mês de `b`.
fn months_between(a: NaiveDate, b: NaiveDate) -> i32 {
    (b.year() * 12 + b.month0() as i32) - (a.year() * 12 + a.month0() as i32)
}

fn first_of_month(d: NaiveDate) -> NaiveDate {
    d.with_day(1).expect("dia 1")
}

/// Reajustes (degraus no mês de início do ano letivo) entre a cotação e a data.
fn steps(fee_date: NaiveDate, date: NaiveDate, month: u32) -> i32 {
    let boundary = |y: i32| NaiveDate::from_ymd_opt(y, month, 1).expect("data válida");
    let first = if boundary(fee_date.year()) > fee_date {
        fee_date.year()
    } else {
        fee_date.year() + 1
    };
    let last = if boundary(date.year()) <= date {
        date.year()
    } else {
        date.year() - 1
    };
    (last - first + 1).max(0)
}

/// Início do curso: a data informada ou o mês de início no ano em que o filho faz a idade.
pub fn start_of(plan: &EducationPlan, birth: Option<NaiveDate>) -> Result<NaiveDate> {
    if let Some(s) = &plan.start_date {
        return Ok(first_of_month(parse_date(s)?));
    }
    let b = birth.ok_or_else(|| Error::invalid(t("nucleo-educacao-erro-sem-nascimento")))?;
    NaiveDate::from_ymd_opt(b.year() + plan.start_age as i32, plan.start_month, 1)
        .ok_or_else(|| Error::invalid(t("nucleo-educacao-erro-inicio")))
}

/// Pagamentos do curso mês a mês (em dinheiro de hoje), do primeiro ao último mês.
fn payments(plan: &EducationPlan, start: NaiveDate, spread: f64) -> Result<Vec<i64>> {
    let fee_date = parse_date(&plan.fee_date)?;
    let every = 12 / plan.payments_per_year.max(1);
    let fee = plan.fee as f64 * (1.0 - plan.discount_pct / 100.0);
    let month = start.month();
    Ok((0..plan.duration_months)
        .map(|k| {
            let date = add_months(start, k as i32);
            let in_year = k % 12;
            let mut edu = 0.0;
            if in_year % every == 0 {
                edu += fee;
            }
            if in_year == 0 {
                edu += plan.yearly_extras as f64;
            }
            let grow = (1.0 + spread).powi(steps(fee_date, date, month));
            (edu * grow).round() as i64 + plan.monthly_living
        })
        .collect())
}

fn monthly_rate(annual: f64) -> f64 {
    (1.0 + annual).powf(1.0 / 12.0) - 1.0
}

/// Valor presente no início do curso dos pagamentos, com o retorno real.
fn present_value(pays: &[i64], r_m: f64) -> f64 {
    pays.iter()
        .enumerate()
        .map(|(k, p)| *p as f64 / (1.0 + r_m).powi(k as i32))
        .sum()
}

/// Aporte de fim de mês, em dinheiro de hoje, para juntar `need` em `n` meses.
fn indexed_payment(need: f64, r_m: f64, n: i32) -> f64 {
    if need <= 0.0 || n <= 0 {
        return need.max(0.0);
    }
    if r_m.abs() < 1e-12 {
        return need / n as f64;
    }
    need * r_m / ((1.0 + r_m).powi(n) - 1.0)
}

/// Aporte fixo em reais (perde valor com a inflação) para juntar `need` (dinheiro de hoje).
fn flat_payment(need: f64, r_m: f64, pi_m: f64, n: i32) -> f64 {
    if need <= 0.0 || n <= 0 {
        return need.max(0.0);
    }
    let factor: f64 = (1..=n)
        .map(|j| (1.0 + r_m).powi(n - j) / (1.0 + pi_m).powi(j))
        .sum();
    need / factor
}

/// A conta do plano em `today`, com `saved` já guardado (dinheiro de hoje).
pub fn compute(
    plan: &EducationPlan,
    birth: Option<NaiveDate>,
    saved: i64,
    infl: EducationInflation,
    today: NaiveDate,
) -> Result<EducationResult> {
    let start = start_of(plan, birth)?;
    let end = add_months(start, plan.duration_months as i32 - 1);
    let now = first_of_month(today);
    let n = months_between(now, start);
    let spread = match plan.inflation_mode {
        InflationMode::Headline => 0.0,
        InflationMode::FixedSpread => plan.fixed_spread.unwrap_or(0.0),
        InflationMode::Series10y => infl.spread,
    };
    let headline_default = infl.headline_annual.is_none();
    let pi = infl.headline_annual.unwrap_or(DEFAULT_HEADLINE);
    let r_m = monthly_rate(plan.real_return);
    let pi_m = monthly_rate(pi);
    let pays = payments(plan, start, spread)?;
    let target = present_value(&pays, r_m);
    let phase = if now > end {
        EducationPhase::Finished
    } else if n <= 0 {
        EducationPhase::Paying
    } else {
        EducationPhase::Saving
    };
    let mut result = EducationResult {
        birth_date: birth.map(fmt_date),
        child_age: birth.map(|b| today.years_since(b).unwrap_or(0)),
        start_date: fmt_date(start),
        end_date: fmt_date(end),
        months_to_start: n,
        phase,
        headline_assumed: pi,
        headline_default,
        fee_at_start: 0,
        fee_at_start_nominal: 0,
        total_payments: pays.iter().sum(),
        target_at_start: target.round() as i64,
        saved,
        monthly_needed: 0,
        monthly_indexed: 0,
        monthly_flat: 0,
        monthly_headline_only: 0,
        monthly_next_year: 0,
        uncovered_year: None,
        shortfall: 0,
        years: vec![],
        notes: vec![],
        inflation: infl,
    };
    let fee_date = parse_date(&plan.fee_date)?;
    let net_fee = plan.fee as f64 * (1.0 - plan.discount_pct / 100.0);
    let fee_real_on = |d: NaiveDate| {
        (net_fee * (1.0 + spread).powi(steps(fee_date, d, start.month()))).round() as i64
    };
    let years_from_now = |d: NaiveDate| months_between(now, d).max(0) as f64 / 12.0;
    result.fee_at_start = fee_real_on(start);
    result.fee_at_start_nominal =
        (result.fee_at_start as f64 * (1.0 + pi).powf(years_from_now(start))).round() as i64;
    match phase {
        EducationPhase::Saving => {
            let need = target - saved as f64 * (1.0 + r_m).powi(n);
            let indexed = indexed_payment(need, r_m, n);
            let flat = flat_payment(need, r_m, pi_m, n);
            let head_target = present_value(&payments(plan, start, 0.0)?, r_m);
            let head_need = head_target - saved as f64 * (1.0 + r_m).powi(n);
            result.monthly_indexed = indexed.round() as i64;
            result.monthly_flat = flat.round() as i64;
            result.monthly_headline_only = indexed_payment(head_need, r_m, n).round() as i64;
            result.monthly_next_year = (indexed * (1.0 + pi)).round() as i64;
            result.shortfall = need.max(0.0).round() as i64;
            result.monthly_needed = match plan.contribution_mode {
                ContributionMode::Indexed => result.monthly_indexed,
                ContributionMode::Flat => result.monthly_flat,
            };
        }
        EducationPhase::Paying => {
            let done = (-n) as usize;
            let left = &pays[done.min(pays.len())..];
            let mut balance = saved as f64;
            for (i, p) in left.iter().enumerate() {
                balance -= *p as f64;
                if balance < -0.5 {
                    result.uncovered_year = Some(((done + i) / 12) as u32 + 1);
                    break;
                }
                balance *= 1.0 + r_m;
            }
            let pv_left = present_value(left, r_m);
            let short = (pv_left - saved as f64).max(0.0);
            result.shortfall = short.round() as i64;
            let m = left.len() as i32;
            let pmt = if short <= 0.0 || m <= 0 {
                0.0
            } else if r_m.abs() < 1e-12 {
                short / m as f64
            } else {
                short * r_m / (1.0 - (1.0 + r_m).powi(-m))
            };
            result.monthly_indexed = pmt.round() as i64;
            result.monthly_flat = result.monthly_indexed;
            result.monthly_headline_only = result.monthly_indexed;
            result.monthly_next_year = (pmt * (1.0 + pi)).round() as i64;
            result.monthly_needed = result.monthly_indexed;
        }
        EducationPhase::Finished => {}
    }
    // Ano a ano, seguindo o plano: aporte até o início, pagamentos durante o curso.
    if phase != EducationPhase::Finished {
        let pmt = if phase == EducationPhase::Saving {
            result.monthly_indexed as f64
        } else {
            result.monthly_needed as f64
        };
        let mut balance = saved as f64;
        let mut by_year: BTreeMap<i32, EducationYear> = BTreeMap::new();
        let mut month = now;
        while month <= end {
            let year = month.year();
            let row = by_year.entry(year).or_insert_with(|| EducationYear {
                year,
                in_course: false,
                fee_real: 0,
                fee_nominal: 0,
                payments_real: 0,
                planned_balance: balance.round() as i64,
            });
            let k = months_between(start, month);
            if k >= 0 {
                let p = pays.get(k as usize).copied().unwrap_or(0);
                row.in_course = true;
                row.payments_real += p;
                balance -= p as f64;
                if phase == EducationPhase::Paying {
                    balance += pmt;
                }
            } else {
                balance += pmt;
            }
            balance *= 1.0 + r_m;
            month = add_months(month, 1);
        }
        for row in by_year.values_mut() {
            let on = NaiveDate::from_ymd_opt(row.year, start.month(), 1)
                .expect("data válida")
                .max(now);
            row.fee_real = fee_real_on(on);
            row.fee_nominal =
                (row.fee_real as f64 * (1.0 + pi).powf(years_from_now(on))).round() as i64;
        }
        result.years = by_year.into_values().collect();
    }
    let notes = &mut result.notes;
    if plan.inflation_mode == InflationMode::Series10y && result.inflation.missing {
        notes.push(t("nucleo-educacao-nota-sem-serie"));
    }
    if headline_default {
        notes.push(t_args(
            "nucleo-educacao-nota-inflacao-suposta",
            &args(&[(
                "taxa",
                crate::ai::alerts::pct(i18n::current(), DEFAULT_HEADLINE, 1).into(),
            )]),
        ));
    }
    if plan.institution_type == InstitutionType::Public && plan.monthly_living == 0 {
        notes.push(t("nucleo-educacao-nota-publica"));
    }
    if plan.institution_type == InstitutionType::Abroad {
        notes.push(t("nucleo-educacao-nota-exterior"));
    }
    if plan.institution_type != InstitutionType::Public && plan.fee == 0 {
        notes.push(t("nucleo-educacao-nota-sem-mensalidade"));
    }
    if phase == EducationPhase::Finished {
        notes.push(t("nucleo-educacao-nota-terminou"));
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Cadastro

const PLAN_COLUMNS: &str = "member_id, expected_birth, start_age, start_month, start_date,
    institution_type, course, duration_months, payments_per_year, fee, fee_date, yearly_extras,
    monthly_living, discount_pct, inflation_mode, inflation_series, fixed_spread, real_return,
    contribution_mode, pool_rule";

fn row_to_plan(r: &rusqlite::Row<'_>) -> rusqlite::Result<EducationPlan> {
    Ok(EducationPlan {
        member_id: r.get(0)?,
        expected_birth: r.get(1)?,
        start_age: r.get(2)?,
        start_month: r.get(3)?,
        start_date: r.get(4)?,
        institution_type: InstitutionType::parse(&r.get::<_, String>(5)?),
        course: r.get(6)?,
        duration_months: r.get(7)?,
        payments_per_year: r.get(8)?,
        fee: r.get(9)?,
        fee_date: r.get(10)?,
        yearly_extras: r.get(11)?,
        monthly_living: r.get(12)?,
        discount_pct: r.get(13)?,
        inflation_mode: InflationMode::parse(&r.get::<_, String>(14)?),
        inflation_series: r.get(15)?,
        fixed_spread: r.get(16)?,
        real_return: r.get(17)?,
        contribution_mode: ContributionMode::parse(&r.get::<_, String>(18)?),
        pool_rule: PoolRule::parse(&r.get::<_, String>(19)?),
    })
}

/// O plano da meta (`None`: meta sem plano).
pub fn load(conn: &Connection, goal_id: &str) -> Result<Option<EducationPlan>> {
    Ok(conn
        .query_row(
            &format!("SELECT {PLAN_COLUMNS} FROM education_plan WHERE goal_id = ?1"),
            [goal_id],
            row_to_plan,
        )
        .optional()?)
}

/// O filho do plano: (nome, nascimento).
pub fn child(
    conn: &Connection,
    plan: &EducationPlan,
) -> Result<(Option<String>, Option<NaiveDate>)> {
    match &plan.member_id {
        Some(id) => {
            let m = crate::household::get_member(conn, id)?;
            let birth = m
                .profile
                .birth_date
                .as_deref()
                .map(parse_date)
                .transpose()?;
            Ok((Some(m.name), birth))
        }
        None => Ok((
            None,
            plan.expected_birth.as_deref().map(parse_date).transpose()?,
        )),
    }
}

fn validate(conn: &Connection, plan: &EducationPlan) -> Result<()> {
    let bad = |key: &str| Err(Error::invalid(t(key)));
    if plan.member_id.is_none() && plan.expected_birth.is_none() {
        return bad("nucleo-educacao-erro-sem-filho");
    }
    if let Some(d) = &plan.expected_birth {
        parse_date(d)?;
    }
    if let Some(d) = &plan.start_date {
        parse_date(d)?;
    }
    parse_date(&plan.fee_date)?;
    if plan.start_age > 60 || !(1..=12).contains(&plan.start_month) {
        return bad("nucleo-educacao-erro-inicio");
    }
    if !(1..=180).contains(&plan.duration_months) {
        return bad("nucleo-educacao-erro-duracao");
    }
    if ![1, 2, 4, 12].contains(&plan.payments_per_year) {
        return bad("nucleo-educacao-erro-parcelas");
    }
    if plan.fee < 0 || plan.yearly_extras < 0 || plan.monthly_living < 0 {
        return bad("nucleo-educacao-erro-valor-negativo");
    }
    if !(0.0..=100.0).contains(&plan.discount_pct) {
        return bad("nucleo-educacao-erro-desconto");
    }
    if !(-0.05..=0.15).contains(&plan.real_return) {
        return bad("nucleo-educacao-erro-retorno");
    }
    if plan.inflation_mode == InflationMode::FixedSpread
        && !plan.fixed_spread.is_some_and(|s| (-0.1..=0.2).contains(&s))
    {
        return bad("nucleo-educacao-erro-diferenca");
    }
    if let Some(s) = &plan.inflation_series {
        let ok: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM index_series WHERE id = ?1 AND kind = 'monthly_rate')",
            [s],
            |r| r.get(0),
        )?;
        if !ok {
            return bad("nucleo-educacao-erro-serie");
        }
    }
    // O filho precisa da data de nascimento (ou o plano, da data de início).
    let (_, birth) = child(conn, plan)?;
    start_of(plan, birth)?;
    Ok(())
}

/// A conta de um plano já validado com o que a meta tem guardado.
pub fn result_of(
    conn: &Connection,
    plan: &EducationPlan,
    saved: i64,
    today: NaiveDate,
) -> Result<EducationResult> {
    let (_, birth) = child(conn, plan)?;
    let infl = inflation(
        conn,
        plan.inflation_mode,
        plan.inflation_series.as_deref(),
        plan.fixed_spread,
        today,
    )?;
    compute(plan, birth, saved, infl, today)
}

pub fn brief(
    conn: &Connection,
    plan: &EducationPlan,
    result: &EducationResult,
) -> Result<EducationBrief> {
    let (name, _) = child(conn, plan)?;
    Ok(EducationBrief {
        member_id: plan.member_id.clone(),
        child_name: name,
        start_date: result.start_date.clone(),
        end_date: result.end_date.clone(),
        months_to_start: result.months_to_start,
        phase: result.phase,
        institution_type: plan.institution_type,
        course: plan.course.clone(),
        spread: match plan.inflation_mode {
            InflationMode::Headline => 0.0,
            InflationMode::FixedSpread => plan.fixed_spread.unwrap_or(0.0),
            InflationMode::Series10y => result.inflation.spread,
        },
        target_at_start: result.target_at_start,
        monthly_needed: result.monthly_needed,
    })
}

fn write_plan(tx: &Transaction<'_>, goal_id: &str, p: &EducationPlan) -> Result<()> {
    let now = now_utc();
    tx.execute(
        &format!(
            "INSERT INTO education_plan (goal_id, {PLAN_COLUMNS}, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                     ?18, ?19, ?20, ?21, ?22, ?22)
             ON CONFLICT (goal_id) DO UPDATE SET member_id = excluded.member_id,
                 expected_birth = excluded.expected_birth, start_age = excluded.start_age,
                 start_month = excluded.start_month, start_date = excluded.start_date,
                 institution_type = excluded.institution_type, course = excluded.course,
                 duration_months = excluded.duration_months,
                 payments_per_year = excluded.payments_per_year, fee = excluded.fee,
                 fee_date = excluded.fee_date, yearly_extras = excluded.yearly_extras,
                 monthly_living = excluded.monthly_living, discount_pct = excluded.discount_pct,
                 inflation_mode = excluded.inflation_mode,
                 inflation_series = excluded.inflation_series,
                 fixed_spread = excluded.fixed_spread, real_return = excluded.real_return,
                 contribution_mode = excluded.contribution_mode, pool_rule = excluded.pool_rule,
                 updated_at = excluded.updated_at"
        ),
        params![
            goal_id,
            p.member_id,
            p.expected_birth,
            p.start_age,
            p.start_month,
            p.start_date,
            p.institution_type.as_str(),
            p.course.as_deref().map(str::trim).filter(|c| !c.is_empty()),
            p.duration_months,
            p.payments_per_year,
            p.fee,
            p.fee_date,
            p.yearly_extras,
            p.monthly_living,
            p.discount_pct,
            p.inflation_mode.as_str(),
            p.inflation_series,
            p.fixed_spread,
            p.real_return,
            p.contribution_mode.as_str(),
            p.pool_rule.as_str(),
            now
        ],
    )?;
    Ok(())
}

/// Grava um recálculo (`at`: o instante, RFC 3339).
pub fn record_review(
    conn: &Connection,
    goal_id: &str,
    at: &str,
    reason: ReviewReason,
    plan: &EducationPlan,
    r: &EducationResult,
) -> Result<()> {
    let assumptions = serde_json::json!({
        "plan": plan,
        "inflation": r.inflation,
        "headline_assumed": r.headline_assumed,
        "start_date": r.start_date,
        "end_date": r.end_date,
    });
    conn.execute(
        "INSERT INTO education_plan_review (id, goal_id, reviewed_at, reason, spread, spread_from,
             spread_to, fee, target_at_start, saved, monthly_needed, assumptions)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            new_id(),
            goal_id,
            at,
            reason.as_str(),
            brief_spread(plan, r),
            r.inflation.from,
            r.inflation.to,
            plan.fee,
            r.target_at_start,
            r.saved,
            r.monthly_needed,
            assumptions.to_string()
        ],
    )?;
    Ok(())
}

fn brief_spread(plan: &EducationPlan, r: &EducationResult) -> f64 {
    match plan.inflation_mode {
        InflationMode::Headline => 0.0,
        InflationMode::FixedSpread => plan.fixed_spread.unwrap_or(0.0),
        InflationMode::Series10y => r.inflation.spread,
    }
}

pub fn reviews(conn: &Connection, goal_id: &str) -> Result<Vec<EducationReview>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, reviewed_at, reason, spread, spread_from, spread_to, fee, target_at_start,
                saved, monthly_needed
           FROM education_plan_review WHERE goal_id = ?1 ORDER BY reviewed_at DESC, id",
    )?;
    let rows = stmt
        .query_map([goal_id], |r| {
            Ok(EducationReview {
                id: r.get(0)?,
                reviewed_at: r.get(1)?,
                reason: ReviewReason::parse(&r.get::<_, String>(2)?),
                spread: r.get(3)?,
                spread_from: r.get(4)?,
                spread_to: r.get(5)?,
                fee: r.get(6)?,
                target_at_start: r.get(7)?,
                saved: r.get(8)?,
                monthly_needed: r.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Nome padrão da meta: "Faculdade de Ana" (ou com a data prevista do nascimento).
fn default_name(conn: &Connection, plan: &EducationPlan) -> Result<String> {
    let (name, birth) = child(conn, plan)?;
    Ok(match (name, birth) {
        (Some(n), _) => t_args("nucleo-educacao-meta-nome", &args(&[("nome", n.into())])),
        (None, Some(b)) => t_args(
            "nucleo-educacao-meta-nome-previsto",
            &args(&[("data", i18n::date(i18n::current(), b).into())]),
        ),
        (None, None) => t("nucleo-educacao-meta-nome-sem-filho"),
    })
}

/// Cria ou altera a meta de faculdade e o plano; a meta recebe o alvo (valor necessário no
/// início do curso), a data de início e o aporte, e o recálculo fica no histórico.
pub fn save(tx: &Transaction<'_>, a: &EducationSaveArgs, today: NaiveDate) -> Result<Id> {
    let plan = &a.plan;
    validate(tx, plan)?;
    let old = match &a.goal_id {
        Some(id) => {
            let g = super::load(tx, id)?;
            if g.archived {
                return Err(Error::invalid(t("nucleo-orcamento-erro-meta-arquivada")));
            }
            load(tx, id)?
        }
        None => None,
    };
    let id = match &a.goal_id {
        Some(id) => {
            super::update(
                tx,
                &super::GoalPatch {
                    id: id.clone(),
                    name: a.name.clone(),
                    kind: Some(super::GoalKind::Education),
                    target_amount: None,
                    target_date: None,
                    initial_amount: a.initial_amount,
                    monthly_contribution: None,
                    account_ids: None,
                    archived: None,
                    notes: None,
                    real_return: None,
                },
            )?;
            id.clone()
        }
        None => super::create(
            tx,
            &super::GoalInput {
                name: match &a.name {
                    Some(n) if !n.trim().is_empty() => n.clone(),
                    _ => default_name(tx, plan)?,
                },
                kind: super::GoalKind::Education,
                target_amount: 1,
                currency: a.currency.clone(),
                target_date: None,
                initial_amount: a.initial_amount,
                monthly_contribution: None,
                account_ids: None,
                notes: None,
                real_return: None,
            },
        )?,
    };
    write_plan(tx, &id, plan)?;
    let view = super::view(tx, super::load(tx, &id)?, today)?;
    let result = result_of(tx, plan, view.current_amount, today)?;
    super::update(
        tx,
        &super::GoalPatch {
            id: id.clone(),
            name: None,
            kind: None,
            target_amount: Some(result.target_at_start.max(1)),
            target_date: Some(Some(result.start_date.clone())),
            initial_amount: None,
            monthly_contribution: Some(Some(result.monthly_needed)),
            account_ids: None,
            archived: None,
            notes: None,
            real_return: None,
        },
    )?;
    let reason = match &old {
        None => ReviewReason::Created,
        Some(o) => {
            let mut same = o.clone();
            same.fee = plan.fee;
            same.fee_date = plan.fee_date.clone();
            if &same == plan && (o.fee != plan.fee || o.fee_date != plan.fee_date) {
                ReviewReason::FeeUpdate
            } else {
                ReviewReason::Edit
            }
        }
    };
    if old.as_ref() != Some(plan) || old.is_none() {
        record_review(tx, &id, &now_utc(), reason, plan, &result)?;
    }
    Ok(id)
}

/// O plano completo de uma meta.
pub fn plan_view(conn: &Connection, goal_id: &str, today: NaiveDate) -> Result<EducationPlanView> {
    let plan = load(conn, goal_id)?.ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-educacao-plano",
            &args(&[("id", goal_id.to_string().into())]),
        ))
    })?;
    let goal = super::view(conn, super::load(conn, goal_id)?, today)?;
    let result = result_of(conn, &plan, goal.current_amount, today)?;
    let (child_name, _) = child(conn, &plan)?;
    Ok(EducationPlanView {
        holdings: super::holdings::views(conn, &goal.goal, today)?,
        reviews: reviews(conn, goal_id)?,
        child_name,
        result,
        plan,
        goal,
    })
}

/// Simulação sem gravar.
pub fn simulate(
    conn: &Connection,
    a: &EducationSimulateArgs,
    today: NaiveDate,
) -> Result<EducationResult> {
    let base = match &a.goal_id {
        Some(id) => load(conn, id)?.unwrap_or_default(),
        None => EducationPlan::default(),
    };
    let mut merged = serde_json::to_value(&base)?;
    if let (Some(serde_json::Value::Object(over)), serde_json::Value::Object(m)) =
        (&a.plan, &mut merged)
    {
        for (k, v) in over {
            m.insert(k.clone(), v.clone());
        }
    }
    let plan = &serde_json::from_value::<EducationPlan>(merged)?;
    let saved = match (&a.saved, &a.goal_id) {
        (Some(s), _) => *s,
        (None, Some(id)) => super::view(conn, super::load(conn, id)?, today)?.current_amount,
        (None, None) => 0,
    };
    let birth = match &a.birth_date {
        Some(b) => Some(parse_date(b)?),
        None => {
            if plan.member_id.is_some() || plan.expected_birth.is_some() {
                child(conn, plan)?.1
            } else {
                None
            }
        }
    };
    if a.birth_date.is_none() {
        validate(conn, plan)?;
    }
    let infl = inflation(
        conn,
        plan.inflation_mode,
        plan.inflation_series.as_deref(),
        plan.fixed_spread,
        today,
    )?;
    compute(plan, birth, saved, infl, today)
}

/// Todos os planos (metas não arquivadas), somados.
pub fn overview(conn: &Connection, today: NaiveDate) -> Result<EducationOverview> {
    let base = crate::household::base_currency(conn)?;
    let ids: Vec<Id> = conn
        .prepare(
            "SELECT p.goal_id FROM education_plan p JOIN goal g ON g.id = p.goal_id
              WHERE g.archived = 0 ORDER BY g.target_date, g.created_at",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = EducationOverview {
        plans: vec![],
        monthly_total: 0,
        saved_total: 0,
        target_total: 0,
        currency: base.clone(),
        overlap_years: vec![],
    };
    let mut courses: BTreeMap<i32, u32> = BTreeMap::new();
    for id in ids {
        let Some(plan) = load(conn, &id)? else {
            continue;
        };
        let goal = super::view(conn, super::load(conn, &id)?, today)?;
        let result = match result_of(conn, &plan, goal.current_amount, today) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("plano de educação {id}: {e}");
                continue;
            }
        };
        let b = brief(conn, &plan, &result)?;
        if goal.goal.currency == base {
            out.monthly_total += b.monthly_needed;
            out.saved_total += goal.current_amount;
            out.target_total += b.target_at_start;
        }
        let (s, e) = (parse_date(&b.start_date)?, parse_date(&b.end_date)?);
        for y in s.year()..=e.year() {
            *courses.entry(y).or_default() += 1;
        }
        out.plans.push(EducationOverviewItem {
            goal_id: id,
            name: goal.goal.name.clone(),
            currency: goal.goal.currency.clone(),
            saved: goal.current_amount,
            brief: b,
        });
    }
    out.overlap_years = courses
        .into_iter()
        .filter(|(_, n)| *n >= 2)
        .map(|(y, _)| y)
        .collect();
    Ok(out)
}

/// Recálculo anual (a partir de abril, uma vez por ano): grava o recálculo de cada plano que
/// ainda não teve o deste ano e devolve os que mudaram mais de 10% no aporte, com o aporte
/// antigo e o novo, e os cursos que terminaram.
pub struct AnnualChange {
    pub goal_id: Id,
    pub name: String,
    pub currency: String,
    pub old_monthly: i64,
    pub new_monthly: i64,
    pub finished: bool,
}

pub fn annual_review(conn: &Connection, today: NaiveDate) -> Result<Vec<AnnualChange>> {
    if today.month() < REVIEW_MONTH {
        return Ok(vec![]);
    }
    let since = format!("{:04}-{:02}-01", today.year(), REVIEW_MONTH);
    let due: Vec<Id> = conn
        .prepare(
            "SELECT p.goal_id FROM education_plan p JOIN goal g ON g.id = p.goal_id
              WHERE g.archived = 0
                AND NOT EXISTS (SELECT 1 FROM education_plan_review r
                                 WHERE r.goal_id = p.goal_id AND r.reason = 'annual'
                                   AND r.reviewed_at >= ?1)
              ORDER BY p.goal_id",
        )?
        .query_map([&since], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for id in due {
        let Some(plan) = load(conn, &id)? else {
            continue;
        };
        let goal = super::view(conn, super::load(conn, &id)?, today)?;
        let result = match result_of(conn, &plan, goal.current_amount, today) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("recálculo do plano de educação {id}: {e}");
                continue;
            }
        };
        let old: Option<i64> = conn
            .query_row(
                "SELECT monthly_needed FROM education_plan_review WHERE goal_id = ?1
                  ORDER BY reviewed_at DESC LIMIT 1",
                [&id],
                |r| r.get(0),
            )
            .optional()?;
        // Pelo dia do recálculo (o mesmo que decide se o deste ano já foi feito).
        let at = format!("{}T12:00:00.000Z", fmt_date(today));
        record_review(conn, &id, &at, ReviewReason::Annual, &plan, &result)?;
        let finished = result.phase == EducationPhase::Finished;
        let changed = old.is_some_and(|o| {
            let base = (o.abs()).max(1) as f64;
            ((result.monthly_needed - o).abs() as f64) / base > REVIEW_ALERT_CHANGE
        });
        if changed || finished {
            out.push(AnnualChange {
                goal_id: id,
                name: goal.goal.name.clone(),
                currency: goal.goal.currency.clone(),
                old_monthly: old.unwrap_or(0),
                new_monthly: result.monthly_needed,
                finished,
            });
        }
    }
    Ok(out)
}
