//! Rendas da família (docs/income.md): cada membro tem uma ou mais rendas (CLT, servidor,
//! aposentadoria, pensão, PJ, autônomo, informal, aluguel, benefício), com bruto e líquido de
//! referência, e os holerites importados (bruto, INSS, IRRF, outros descontos e líquido de
//! cada mês).
//!
//! A previsão dos próximos meses usa, por renda:
//! - o último holerite (até 3 meses), senão o bruto e o líquido informados;
//! - nas rendas com desconto em folha (CLT, servidor, aposentadoria e pensão do INSS), o INSS
//!   e o IRRF das regras do país (`tax_burden`), quando falta um dos dois valores;
//! - o 13º (metade em novembro, sem descontos; o resto em dezembro, com INSS e IRRF sobre o
//!   13º inteiro) e o terço de férias no mês escolhido;
//! - nas rendas variáveis (informal, comissão), a média dos últimos 6 meses dos lançamentos da
//!   categoria e conta da renda, quando houver;
//! - o início e o fim da renda.

pub mod api;
pub mod payslip;
pub mod payslip_specs;
pub mod termination;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_agent;
#[cfg(test)]
mod tests_country;
#[cfg(test)]
mod tests_layout;
#[cfg(test)]
mod tests_termination;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::tax_burden::rules::{self, PayrollRules};
use crate::time::{add_months, fmt_date, month_of, now_utc, parse_date};
use crate::transactions::util::{clean_name, clean_opt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum IncomeSourceKind {
    /// Emprego com carteira assinada.
    Clt,
    PublicServant,
    /// Aposentadoria (INSS ou RPPS).
    Retirement,
    /// Pensão (por morte ou alimentícia).
    Pension,
    Pj,
    SelfEmployed,
    Informal,
    Rental,
    /// Benefício social (Bolsa Família, BPC, seguro-desemprego).
    Benefit,
    Other,
}

impl IncomeSourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Clt => "clt",
            Self::PublicServant => "public_servant",
            Self::Retirement => "retirement",
            Self::Pension => "pension",
            Self::Pj => "pj",
            Self::SelfEmployed => "self_employed",
            Self::Informal => "informal",
            Self::Rental => "rental",
            Self::Benefit => "benefit",
            Self::Other => "other",
        }
    }

    fn parse(s: &str) -> Self {
        serde_json::from_value(serde_json::Value::String(s.into())).unwrap_or(Self::Other)
    }

    /// Renda formal (com vínculo e recolhimentos).
    pub fn formal(self) -> bool {
        matches!(
            self,
            Self::Clt | Self::PublicServant | Self::Retirement | Self::Pension | Self::Pj
        )
    }

    /// Desconto em folha de INSS (ou RPPS) e IRRF.
    fn withholding(self) -> bool {
        matches!(
            self,
            Self::Clt | Self::PublicServant | Self::Retirement | Self::Pension
        )
    }

    /// Contribui para a previdência na folha (aposentado e pensionista não).
    fn social_security(self) -> bool {
        matches!(self, Self::Clt | Self::PublicServant)
    }

    /// Nome do tipo (os do Brasil) no idioma da instalação; os de outros países saem de
    /// `country::info` (`payroll.kinds`).
    pub fn label(self) -> String {
        t(match self {
            Self::Clt => "nucleo-pais-br-tipo-clt",
            Self::PublicServant => "nucleo-pais-tipo-servidor",
            Self::Retirement => "nucleo-pais-tipo-aposentadoria",
            Self::Pension => "nucleo-pais-tipo-pensao",
            Self::Pj => "nucleo-pais-br-tipo-pj",
            Self::SelfEmployed => "nucleo-pais-tipo-autonomo",
            Self::Informal => "nucleo-pais-tipo-informal",
            Self::Rental => "nucleo-pais-tipo-aluguel",
            Self::Benefit => "nucleo-pais-tipo-beneficio",
            Self::Other => "nucleo-pais-tipo-outra",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeSource {
    pub id: Id,
    pub member_id: Id,
    pub member_name: String,
    pub name: String,
    pub kind: IncomeSourceKind,
    pub formal: bool,
    pub employer: Option<String>,
    pub employer_cnpj: Option<String>,
    #[ts(type = "number | null")]
    pub gross_monthly: Option<i64>,
    #[ts(type = "number | null")]
    pub net_monthly: Option<i64>,
    pub variable: bool,
    pub pay_day: Option<u32>,
    pub has_thirteenth: bool,
    pub vacation_month: Option<u32>,
    pub dependents: u32,
    pub account_id: Option<Id>,
    pub category_id: Option<Id>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub archived: bool,
    pub notes: Option<String>,
    /// Holerites importados.
    pub payslips: u32,
    /// Competência do último holerite (`YYYY-MM`).
    pub last_payslip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeSourceInput {
    pub member_id: Id,
    pub name: String,
    pub kind: IncomeSourceKind,
    #[serde(default)]
    #[ts(optional)]
    pub employer: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub employer_cnpj: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub gross_monthly: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub net_monthly: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub variable: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub pay_day: Option<u32>,
    /// Padrão: sim nas rendas com desconto em folha.
    #[serde(default)]
    #[ts(optional)]
    pub has_thirteenth: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub vacation_month: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub dependents: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub start_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub end_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeSourcePatch {
    pub id: Id,
    /// Os campos da renda (substituem todos).
    pub source: IncomeSourceInput,
    #[serde(default)]
    #[ts(optional)]
    pub archived: Option<bool>,
}

const COLS: &str = "s.id, s.member_id, m.name, s.name, s.kind, s.employer, s.employer_cnpj, s.gross_monthly,
    s.net_monthly, s.variable, s.pay_day, s.has_thirteenth, s.vacation_month, s.dependents, s.account_id,
    s.category_id, s.start_date, s.end_date, s.archived, s.notes,
    (SELECT COUNT(*) FROM payslip p WHERE p.source_id = s.id),
    (SELECT MAX(reference_month) FROM payslip p WHERE p.source_id = s.id)";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<IncomeSource> {
    let kind = IncomeSourceKind::parse(&r.get::<_, String>(4)?);
    Ok(IncomeSource {
        id: r.get(0)?,
        member_id: r.get(1)?,
        member_name: r.get(2)?,
        name: r.get(3)?,
        kind,
        formal: kind.formal(),
        employer: r.get(5)?,
        employer_cnpj: r.get(6)?,
        gross_monthly: r.get(7)?,
        net_monthly: r.get(8)?,
        variable: r.get(9)?,
        pay_day: r.get(10)?,
        has_thirteenth: r.get(11)?,
        vacation_month: r.get(12)?,
        dependents: r.get(13)?,
        account_id: r.get(14)?,
        category_id: r.get(15)?,
        start_date: r.get(16)?,
        end_date: r.get(17)?,
        archived: r.get(18)?,
        notes: r.get(19)?,
        payslips: r.get(20)?,
        last_payslip: r.get(21)?,
    })
}

pub fn list_sources(
    conn: &Connection,
    member_id: Option<&str>,
    include_archived: bool,
) -> Result<Vec<IncomeSource>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM income_source s JOIN member m ON m.id = s.member_id
          WHERE (?1 IS NULL OR s.member_id = ?1) AND (?2 OR s.archived = 0)
          ORDER BY s.archived, m.sort_order, m.name, s.name"
    ))?;
    Ok(stmt
        .query_map(params![member_id, include_archived], row)?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn get_source(conn: &Connection, id: &str) -> Result<IncomeSource> {
    conn.query_row(
        &format!(
            "SELECT {COLS} FROM income_source s JOIN member m ON m.id = s.member_id WHERE s.id = ?1"
        ),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-renda-erro-renda",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

fn validate(conn: &Connection, i: &IncomeSourceInput) -> Result<()> {
    crate::household::get_member(conn, &i.member_id)?;
    for (v, key) in [
        (i.gross_monthly, "nucleo-renda-erro-bruto-negativo"),
        (i.net_monthly, "nucleo-renda-erro-liquido-negativo"),
    ] {
        if v.is_some_and(|v| v < 0) {
            return Err(Error::invalid(t(key)));
        }
    }
    if let (Some(g), Some(n)) = (i.gross_monthly, i.net_monthly)
        && n > g
    {
        return Err(Error::invalid(t("nucleo-renda-erro-liquido-maior")));
    }
    if i.pay_day.is_some_and(|d| !(1..=31).contains(&d)) {
        return Err(Error::invalid(t("nucleo-renda-erro-dia-pagamento")));
    }
    if i.vacation_month.is_some_and(|m| !(1..=12).contains(&m)) {
        return Err(Error::invalid(t("nucleo-renda-erro-mes-ferias")));
    }
    for d in [&i.start_date, &i.end_date].into_iter().flatten() {
        parse_date(d)?;
    }
    if let (Some(s), Some(e)) = (&i.start_date, &i.end_date)
        && e < s
    {
        return Err(Error::invalid(t("nucleo-renda-erro-termina-antes")));
    }
    if let Some(a) = &i.account_id {
        crate::accounts::currency_and_kind(conn, a)?;
    }
    Ok(())
}

fn cnpj(s: &Option<String>) -> Option<String> {
    s.as_deref()
        .map(crate::research::cnpj_digits)
        .filter(|c| c.len() == 14)
}

pub fn create_source(tx: &Transaction<'_>, i: &IncomeSourceInput) -> Result<Id> {
    validate(tx, i)?;
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO income_source (id, member_id, name, kind, employer, employer_cnpj, gross_monthly, net_monthly,
                                    variable, pay_day, has_thirteenth, vacation_month, dependents, account_id,
                                    category_id, start_date, end_date, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?19)",
        params![
            id,
            i.member_id,
            clean_name(&i.name, "da renda")?,
            i.kind.as_str(),
            clean_opt(i.employer.clone()),
            cnpj(&i.employer_cnpj),
            i.gross_monthly,
            i.net_monthly,
            i.variable.unwrap_or(i.kind == IncomeSourceKind::Informal),
            i.pay_day,
            i.has_thirteenth.unwrap_or(i.kind.withholding()),
            i.vacation_month,
            i.dependents.unwrap_or(0),
            i.account_id,
            i.category_id,
            i.start_date,
            i.end_date,
            clean_opt(i.notes.clone()),
            now
        ],
    )?;
    Ok(id)
}

pub fn update_source(tx: &Transaction<'_>, p: &IncomeSourcePatch) -> Result<()> {
    let cur = get_source(tx, &p.id)?;
    let i = &p.source;
    validate(tx, i)?;
    tx.execute(
        "UPDATE income_source SET member_id = ?2, name = ?3, kind = ?4, employer = ?5, employer_cnpj = ?6,
                gross_monthly = ?7, net_monthly = ?8, variable = ?9, pay_day = ?10, has_thirteenth = ?11,
                vacation_month = ?12, dependents = ?13, account_id = ?14, category_id = ?15, start_date = ?16,
                end_date = ?17, notes = ?18, archived = ?19, updated_at = ?20
          WHERE id = ?1",
        params![
            cur.id,
            i.member_id,
            clean_name(&i.name, "da renda")?,
            i.kind.as_str(),
            clean_opt(i.employer.clone()),
            cnpj(&i.employer_cnpj),
            i.gross_monthly,
            i.net_monthly,
            i.variable.unwrap_or(cur.variable),
            i.pay_day,
            i.has_thirteenth.unwrap_or(cur.has_thirteenth),
            i.vacation_month,
            i.dependents.unwrap_or(cur.dependents),
            i.account_id,
            i.category_id,
            i.start_date,
            i.end_date,
            clean_opt(i.notes.clone()),
            p.archived.unwrap_or(cur.archived),
            now_utc()
        ],
    )?;
    Ok(())
}

pub fn delete_source(tx: &Transaction<'_>, id: &str) -> Result<()> {
    get_source(tx, id)?;
    tx.execute("DELETE FROM income_source WHERE id = ?1", [id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Previsão e resumo

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeMonth {
    pub month: String,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub social_security: i64,
    #[ts(type = "number")]
    pub income_tax: i64,
    #[ts(type = "number")]
    pub other_deductions: i64,
    #[ts(type = "number")]
    pub net: i64,
    /// O que entra neste mês além do salário (13º, férias).
    pub extras: Vec<String>,
}

impl IncomeMonth {
    fn empty(month: String) -> Self {
        Self {
            month,
            gross: 0,
            social_security: 0,
            income_tax: 0,
            other_deductions: 0,
            net: 0,
            extras: vec![],
        }
    }

    fn add(&mut self, o: &IncomeMonth) {
        self.gross += o.gross;
        self.social_security += o.social_security;
        self.income_tax += o.income_tax;
        self.other_deductions += o.other_deductions;
        self.net += o.net;
        for e in &o.extras {
            if !self.extras.contains(e) {
                self.extras.push(e.clone());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SourceProjection {
    pub source_id: Id,
    pub name: String,
    pub kind: IncomeSourceKind,
    /// De onde vem a base: `payslip`, `declared`, `average` (média dos lançamentos) ou `none`.
    pub basis: String,
    pub months: Vec<IncomeMonth>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberIncome {
    pub member_id: Id,
    pub member_name: String,
    pub sources: Vec<IncomeSource>,
    pub projection: Vec<SourceProjection>,
    /// Soma da previsão dos próximos meses.
    pub next: IncomeMonth,
    /// Holerites dos últimos 12 meses somados.
    pub last_12m: IncomeMonth,
    /// (INSS + IRRF) ÷ bruto, nos holerites dos últimos 12 meses.
    pub effective_tax_rate: Option<f64>,
}

/// Um mês passado com holerite: o que veio de fato e o que a previsão de hoje daria para ele.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeActual {
    /// Soma dos holerites do mês (todas as rendas e tipos: mensal, 13º, férias, PLR…).
    pub real: IncomeMonth,
    /// A previsão pela base atual (o último holerite) para o mesmo mês, só das rendas que
    /// têm holerite nele.
    pub expected: IncomeMonth,
}

/// Um holerite dos últimos 12 meses no resumo: mês, tipo, bruto e líquido.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomePayslipBrief {
    pub month: String,
    pub kind: payslip::PayslipKind,
    /// Tipo em texto ("mensal", "PLR", "2ª parcela do 13º"…).
    pub label: String,
    pub member_name: String,
    pub source_name: String,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub net: i64,
    /// Pagamento que não se repete todo mês e que a previsão não calcula (PLR, outro): fica
    /// fora da série "real × previsto".
    pub extra: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeSummary {
    pub from: String,
    pub months: u32,
    pub currency: String,
    pub members: Vec<MemberIncome>,
    /// Família, mês a mês.
    pub monthly: Vec<IncomeMonth>,
    pub next: IncomeMonth,
    pub last_12m: IncomeMonth,
    /// Meses dos últimos 12 (até o atual) com holerite, do mais antigo ao mais recente. PLR e
    /// outros pagamentos que a previsão não calcula ficam de fora (estão em `payslips`).
    pub actual: Vec<IncomeActual>,
    /// Holerites dos últimos 12 meses (até o atual), do mais recente ao mais antigo, com o tipo.
    pub payslips: Vec<IncomePayslipBrief>,
    /// Competência do holerite mais recente usado como base da previsão (`YYYY-MM`); null
    /// quando nenhuma renda é prevista por holerite.
    pub forecast_base_month: Option<String>,
    /// Previsão do mês corrente quando ele ainda não tem holerite (o gráfico emenda a previsão
    /// já neste mês, sem buraco antes de `monthly`); `null` quando o holerite do mês já entrou.
    /// Fora de `next`, que segue somando os próximos meses.
    pub current_month: Option<IncomeMonth>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeSummaryArgs {
    /// Meses de previsão (1 a 24; padrão 12).
    #[serde(default)]
    #[ts(optional)]
    pub months: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
}

/// Base mensal de uma renda: bruto, INSS, IRRF, outros descontos e líquido.
#[derive(Debug, Clone, Copy, Default)]
struct Base {
    gross: i64,
    inss: i64,
    irrf: i64,
    other: i64,
    net: i64,
}

/// Regras da folha do país da família (docs/countries.md, docs/international.md): Brasil,
/// Estados Unidos, os da União Europeia adaptados (Portugal, Espanha, França, Alemanha) e os
/// de fora dela (Reino Unido, Canadá, Argentina, Chile); nos outros o app não calcula os
/// descontos. Onde o país tem tabela própria para o estado civil (chefe de família, declaração
/// conjunta), ele sai do cadastro da família (`tax_burden::for_person`).
fn rules_at(conn: &Connection, month: &str, s: &IncomeSource) -> Option<PayrollRules> {
    country_rules(conn, month)
        .map(|r| crate::tax_burden::for_person(conn, &r, Some(&s.member_id), s.dependents))
}

fn country_rules(conn: &Connection, month: &str) -> Option<PayrollRules> {
    let country = crate::country::family_country(conn);
    let (ds, region) = match country.as_str() {
        "BR" => ("BR", None),
        "US" => ("US", None),
        "PT" | "ES" | "FR" | "DE" => ("EU", Some(country.as_str())),
        "GB" | "CA" | "AR" | "CL" => {
            let (_, rules) = crate::tax_burden::dataset::payroll_only().ok()?;
            return rules
                .into_iter()
                .filter(|r| r.region.as_deref() == Some(country.as_str()))
                .filter(|r| r.from.as_str() <= month)
                .max_by(|a, b| a.from.cmp(&b.from));
        }
        _ => return None,
    };
    let ds = crate::tax_burden::dataset::get(ds).ok()?;
    crate::tax_burden::rules_for(&ds, month, region).cloned()
}

/// Pagamentos extras do ano: o 13º e o terço de férias do Brasil, os subsídios de Natal e de
/// férias de Portugal, as pagas extra de junho e dezembro da Espanha (um salário cada); nos
/// Estados Unidos e nos países que o app não conhece, nenhum.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExtraPay {
    Brazil,
    Portugal,
    Spain,
    /// Argentina: o aguinaldo (SAC), meio salário em junho e outro em dezembro.
    Argentina,
    None,
}

fn extra_pay(conn: &Connection) -> ExtraPay {
    match crate::country::family_country(conn).as_str() {
        "BR" => ExtraPay::Brazil,
        "PT" => ExtraPay::Portugal,
        "ES" => ExtraPay::Spain,
        "AR" => ExtraPay::Argentina,
        _ => ExtraPay::None,
    }
}

/// Descontos de um bruto pelas regras do mês.
fn withhold(conn: &Connection, gross: i64, s: &IncomeSource, month: &str) -> (i64, i64) {
    match rules_at(conn, month, s) {
        Some(r) => {
            let inss = if s.kind.social_security() {
                rules::contribution(gross, &r.inss_bands)
            } else {
                0
            };
            (inss, rules::monthly_irrf(gross, inss, s.dependents, &r))
        }
        None => (0, 0),
    }
}

/// Último holerite mensal (até 3 meses antes de `today`).
fn last_payslip(conn: &Connection, source: &str, today: NaiveDate) -> Result<Option<Base>> {
    let since = month_of(add_months(today, -3));
    Ok(conn
        .query_row(
            "SELECT gross, social_security, income_tax, other_deductions, net FROM payslip
              WHERE source_id = ?1 AND kind = 'monthly' AND reference_month >= ?2
              ORDER BY reference_month DESC LIMIT 1",
            params![source, since],
            |r| {
                Ok(Base {
                    gross: r.get(0)?,
                    inss: r.get(1)?,
                    irrf: r.get(2)?,
                    other: r.get(3)?,
                    net: r.get(4)?,
                })
            },
        )
        .optional()?)
}

/// Média mensal dos últimos 6 meses completos dos lançamentos de receita da renda (categoria
/// e, se informada, conta).
fn recent_average(conn: &Connection, s: &IncomeSource, today: NaiveDate) -> Result<Option<i64>> {
    let Some(cat) = &s.category_id else {
        return Ok(None);
    };
    let first = add_months(today.with_day(1).expect("dia 1"), -6);
    let last = today.with_day(1).expect("dia 1") - chrono::Duration::days(1);
    let total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM txn
          WHERE kind = 'income' AND status <> 'planned' AND category_id = ?1
            AND (?2 IS NULL OR account_id = ?2) AND date BETWEEN ?3 AND ?4",
        params![cat, s.account_id, fmt_date(first), fmt_date(last)],
        |r| r.get(0),
    )?;
    Ok((total > 0).then_some(total / 6))
}

fn base_of(conn: &Connection, s: &IncomeSource, today: NaiveDate) -> Result<(Base, &'static str)> {
    if let Some(b) = last_payslip(conn, &s.id, today)? {
        return Ok((b, "payslip"));
    }
    if s.variable
        && let Some(avg) = recent_average(conn, s, today)?
    {
        return Ok((
            Base {
                gross: avg,
                net: avg,
                ..Default::default()
            },
            "average",
        ));
    }
    let month = month_of(today);
    Ok(match (s.gross_monthly, s.net_monthly) {
        (Some(g), Some(n)) => {
            let (inss, mut irrf) = if s.kind.withholding() {
                withhold(conn, g, s, &month)
            } else {
                (0, 0)
            };
            // O líquido informado manda: imposto calculado a mais sai do IRRF; a menos vira
            // "outros descontos" (plano de saúde, pensão…).
            let gap = g - inss - irrf - n;
            if gap < 0 {
                irrf = (irrf + gap).max(0);
            }
            (
                Base {
                    gross: g,
                    inss,
                    irrf,
                    other: (g - inss - irrf - n).max(0),
                    net: n,
                },
                "declared",
            )
        }
        (Some(g), None) => {
            let (inss, irrf) = if s.kind.withholding() {
                withhold(conn, g, s, &month)
            } else {
                (0, 0)
            };
            (
                Base {
                    gross: g,
                    inss,
                    irrf,
                    other: 0,
                    net: g - inss - irrf,
                },
                "declared",
            )
        }
        (None, Some(n)) => {
            let g = match (s.kind.withholding(), rules_at(conn, &month, s)) {
                (true, Some(r)) => rules::gross_from_net(n, s.dependents, &r),
                _ => n,
            };
            let (inss, irrf) = if s.kind.withholding() {
                withhold(conn, g, s, &month)
            } else {
                (0, 0)
            };
            (
                Base {
                    gross: g,
                    inss,
                    irrf,
                    other: (g - inss - irrf - n).max(0),
                    net: n,
                },
                "declared",
            )
        }
        (None, None) => (Base::default(), "none"),
    })
}

fn project_source(
    conn: &Connection,
    s: &IncomeSource,
    today: NaiveDate,
    months: u32,
) -> Result<SourceProjection> {
    let (b, basis) = base_of(conn, s, today)?;
    let scheme = extra_pay(conn);
    let start = s.start_date.as_deref().map(parse_date).transpose()?;
    let end = s.end_date.as_deref().map(parse_date).transpose()?;
    let mut out = vec![];
    for k in 1..=months {
        let first = add_months(today.with_day(1).expect("dia 1"), k as i32);
        out.push(project_month(conn, s, &b, scheme, (start, end), first));
    }
    Ok(SourceProjection {
        source_id: s.id.clone(),
        name: s.name.clone(),
        kind: s.kind,
        basis: basis.into(),
        months: out,
    })
}

/// Um mês da previsão de uma renda a partir da base `b` (com o 13º, as férias ou os
/// subsídios do país quando caem nesse mês).
fn project_month(
    conn: &Connection,
    s: &IncomeSource,
    b: &Base,
    scheme: ExtraPay,
    (start, end): (Option<NaiveDate>, Option<NaiveDate>),
    first: NaiveDate,
) -> IncomeMonth {
    let month = month_of(first);
    let mut m = IncomeMonth::empty(month.clone());
    let last_day = add_months(first, 1) - chrono::Duration::days(1);
    let active = start.is_none_or(|d| d <= last_day) && end.is_none_or(|d| d >= first);
    if active && b.gross > 0 {
        m.gross = b.gross;
        m.social_security = b.inss;
        m.income_tax = b.irrf;
        m.other_deductions = b.other;
        m.net = b.net;
        match scheme {
            ExtraPay::Brazil => {
                // 13º: metade em novembro (sem descontos), o resto em dezembro (INSS e
                // IRRF do 13º inteiro, tributação exclusiva).
                if s.has_thirteenth && first.month() == 11 {
                    m.gross += b.gross / 2;
                    m.net += b.gross / 2;
                    m.extras.push(t("nucleo-renda-extra-13-1"));
                }
                if s.has_thirteenth && first.month() == 12 {
                    let (inss, irrf) = if s.kind.withholding() {
                        withhold(conn, b.gross, s, &month)
                    } else {
                        (0, 0)
                    };
                    let second = b.gross - b.gross / 2;
                    m.gross += second;
                    m.social_security += inss;
                    m.income_tax += irrf;
                    m.net += second - inss - irrf;
                    m.extras.push(t("nucleo-renda-extra-13-2"));
                }
                if s.vacation_month == Some(first.month()) {
                    let third = b.gross / 3;
                    let (inss_all, irrf_all) = if s.kind.withholding() {
                        withhold(conn, b.gross + third, s, &month)
                    } else {
                        (b.inss, b.irrf)
                    };
                    let (di, dr) = ((inss_all - b.inss).max(0), (irrf_all - b.irrf).max(0));
                    m.gross += third;
                    m.social_security += di;
                    m.income_tax += dr;
                    m.net += third - di - dr;
                    m.extras.push(t("nucleo-renda-extra-terco-ferias"));
                }
            }
            ExtraPay::Portugal => {
                // Subsídios de Natal (dezembro) e de férias (no mês das férias): um salário
                // cada, com retenção própria pelas mesmas regras (14 pagamentos no ano).
                let mut subsidy = |key: &str| {
                    let (ss, irs) = if s.kind.withholding() {
                        withhold(conn, b.gross, s, &month)
                    } else {
                        (0, 0)
                    };
                    m.gross += b.gross;
                    m.social_security += ss;
                    m.income_tax += irs;
                    m.net += b.gross - ss - irs;
                    m.extras.push(t(key));
                };
                if s.has_thirteenth && first.month() == 12 {
                    subsidy("nucleo-renda-extra-subsidio-natal");
                }
                if s.vacation_month == Some(first.month()) {
                    subsidy("nucleo-renda-extra-subsidio-ferias");
                }
            }
            ExtraPay::Spain => {
                // Pagas extra de junho e de dezembro: um salário cada, com a retenção pelas
                // mesmas regras (14 pagas no ano).
                if s.has_thirteenth && matches!(first.month(), 6 | 12) {
                    let (ss, irpf) = if s.kind.withholding() {
                        withhold(conn, b.gross, s, &month)
                    } else {
                        (0, 0)
                    };
                    m.gross += b.gross;
                    m.social_security += ss;
                    m.income_tax += irpf;
                    m.net += b.gross - ss - irpf;
                    m.extras.push(t(if first.month() == 6 {
                        "nucleo-renda-extra-paga-junho"
                    } else {
                        "nucleo-renda-extra-paga-dezembro"
                    }));
                }
            }
            ExtraPay::Argentina => {
                // Aguinaldo (SAC): meio salário em junho e em dezembro, com os mesmos aportes e
                // a retenção de Ganancias.
                if s.has_thirteenth && matches!(first.month(), 6 | 12) {
                    let half = b.gross / 2;
                    let (ap, gan) = if s.kind.withholding() {
                        withhold(conn, half, s, &month)
                    } else {
                        (0, 0)
                    };
                    m.gross += half;
                    m.social_security += ap;
                    m.income_tax += gan;
                    m.net += half - ap - gan;
                    m.extras.push(t("nucleo-renda-extra-aguinaldo"));
                }
            }
            ExtraPay::None => {}
        }
    }
    m
}

/// Bruto, INSS, IRRF, outros descontos e líquido de uma renda pelo que foi informado (ou pelo
/// último holerite), como a previsão calcula. O cofre de demonstração usa para os holerites.
pub(crate) fn declared_payslip(
    conn: &Connection,
    s: &IncomeSource,
    today: NaiveDate,
) -> Result<(i64, i64, i64, i64, i64)> {
    let (b, _) = base_of(conn, s, today)?;
    Ok((b.gross, b.inss, b.irrf, b.other, b.net))
}

/// Holerites dos últimos 12 meses de um membro, somados.
fn history(conn: &Connection, member: &str, today: NaiveDate) -> Result<IncomeMonth> {
    let since = month_of(add_months(today, -12));
    conn.query_row(
        "SELECT COALESCE(SUM(p.gross), 0), COALESCE(SUM(p.social_security), 0), COALESCE(SUM(p.income_tax), 0),
                COALESCE(SUM(p.other_deductions), 0), COALESCE(SUM(p.net), 0)
           FROM payslip p JOIN income_source s ON s.id = p.source_id
          WHERE s.member_id = ?1 AND p.reference_month > ?2",
        params![member, since],
        |r| {
            Ok(IncomeMonth {
                month: "12m".into(),
                gross: r.get(0)?,
                social_security: r.get(1)?,
                income_tax: r.get(2)?,
                other_deductions: r.get(3)?,
                net: r.get(4)?,
                extras: vec![],
            })
        },
    )
    .map_err(Into::into)
    .and_then(|mut m: IncomeMonth| {
        // Os pagamentos além do salário que entraram na soma (PLR, 13º, férias…).
        let mut stmt = conn.prepare(
            "SELECT DISTINCT p.kind FROM payslip p JOIN income_source s ON s.id = p.source_id
              WHERE s.member_id = ?1 AND p.reference_month > ?2 AND p.kind <> 'monthly'
              ORDER BY p.kind",
        )?;
        let kinds: Vec<String> = stmt
            .query_map(params![member, since], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        m.extras = kinds
            .iter()
            .filter_map(|k| payslip_extra(k))
            .collect();
        Ok(m)
    })
}

/// Tipos de holerite que a previsão não calcula (não se repetem todo ano numa data certa).
fn unforecast(kind: &str) -> bool {
    matches!(kind, "plr" | "other")
}

/// Holerites dos últimos 12 meses (até `until`) com o tipo, do mais recente ao mais antigo.
fn payslip_briefs(
    conn: &Connection,
    member: Option<&str>,
    since: &str,
    until: &str,
) -> Result<Vec<IncomePayslipBrief>> {
    let mut stmt = conn.prepare(
        "SELECT p.reference_month, p.kind, m.name, s.name, p.gross, p.net
           FROM payslip p JOIN income_source s ON s.id = p.source_id JOIN member m ON m.id = s.member_id
          WHERE p.reference_month BETWEEN ?1 AND ?2 AND (?3 IS NULL OR s.member_id = ?3)
          ORDER BY p.reference_month DESC, p.kind",
    )?;
    let rows = stmt.query_map(params![since, until, member], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, i64>(5)?,
        ))
    })?;
    let mut out = vec![];
    for r in rows {
        let (month, kind, member_name, source_name, gross, net) = r?;
        out.push(IncomePayslipBrief {
            label: payslip_extra(&kind).unwrap_or_else(|| t("nucleo-renda-mensal")),
            extra: unforecast(&kind),
            kind: payslip::PayslipKind::parse(&kind),
            month,
            member_name,
            source_name,
            gross,
            net,
        });
    }
    Ok(out)
}

/// Rótulo do tipo de holerite que não é o mensal (o mesmo texto das extras da previsão), no
/// idioma da instalação.
fn payslip_extra(kind: &str) -> Option<String> {
    let key = match kind {
        "thirteenth_1" => "nucleo-renda-extra-13-1",
        "thirteenth_2" => "nucleo-renda-extra-13-2",
        "vacation" => "nucleo-renda-extra-ferias",
        "plr" => "nucleo-renda-extra-plr",
        "advance" => "nucleo-renda-extra-adiantamento",
        "other" => "nucleo-renda-extra-outro",
        _ => return None,
    };
    Some(t(key))
}

/// Holerites de uma renda por competência, de `since` a `until` (inclusive).
fn payslips_by_month(
    conn: &Connection,
    source: &str,
    since: &str,
    until: &str,
) -> Result<Vec<IncomeMonth>> {
    let mut stmt = conn.prepare(
        "SELECT reference_month, kind, gross, social_security, income_tax, other_deductions, net
           FROM payslip WHERE source_id = ?1 AND reference_month BETWEEN ?2 AND ?3
          ORDER BY reference_month",
    )?;
    let rows = stmt.query_map(params![source, since, until], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            IncomeMonth {
                month: r.get(0)?,
                gross: r.get(2)?,
                social_security: r.get(3)?,
                income_tax: r.get(4)?,
                other_deductions: r.get(5)?,
                net: r.get(6)?,
                extras: vec![],
            },
        ))
    })?;
    let mut out: Vec<IncomeMonth> = vec![];
    for row in rows {
        let (month, kind, mut m) = row?;
        // PLR e outros pagamentos avulsos: a previsão não os calcula, então ficam fora do
        // "real × previsto" (aparecem na lista de holerites do resumo).
        if unforecast(&kind) {
            continue;
        }
        if let Some(e) = payslip_extra(&kind) {
            m.extras.push(e);
        }
        match out.last_mut() {
            Some(last) if last.month == month => last.add(&m),
            _ => out.push(m),
        }
    }
    Ok(out)
}

/// Competência do holerite mensal mais recente que serve de base para a renda.
fn base_payslip_month(conn: &Connection, source: &str, today: NaiveDate) -> Result<Option<String>> {
    let since = month_of(add_months(today, -3));
    Ok(conn
        .query_row(
            "SELECT reference_month FROM payslip
              WHERE source_id = ?1 AND kind = 'monthly' AND reference_month >= ?2
              ORDER BY reference_month DESC LIMIT 1",
            params![source, since],
            |r| r.get(0),
        )
        .optional()?)
}

pub fn summary(
    conn: &Connection,
    a: &IncomeSummaryArgs,
    today: NaiveDate,
) -> Result<IncomeSummary> {
    let months = a.months.unwrap_or(12).clamp(1, 24);
    let sources = list_sources(conn, a.member_id.as_deref(), false)?;
    let members = crate::household::list_members(conn, false)?;
    let mut out = vec![];
    let mut monthly: Vec<IncomeMonth> = (1..=months)
        .map(|k| {
            IncomeMonth::empty(month_of(add_months(
                today.with_day(1).expect("dia 1"),
                k as i32,
            )))
        })
        .collect();
    let mut family_next = IncomeMonth::empty("next".into());
    let mut family_hist = IncomeMonth::empty("12m".into());
    let mut notes = vec![];
    // Meses passados com holerite: o real e a previsão de hoje para o mesmo mês.
    let this_month = today.with_day(1).expect("dia 1");
    let since = month_of(add_months(this_month, -11));
    let until = month_of(this_month);
    let scheme = extra_pay(conn);
    let mut actual: std::collections::BTreeMap<String, IncomeActual> = Default::default();
    let mut forecast_base_month: Option<String> = None;
    let mut current = IncomeMonth::empty(until.clone());
    for m in members
        .iter()
        .filter(|m| a.member_id.as_deref().is_none_or(|x| x == m.id))
    {
        let mine: Vec<IncomeSource> = sources
            .iter()
            .filter(|s| s.member_id == m.id)
            .cloned()
            .collect();
        let mut projection = vec![];
        let mut next = IncomeMonth::empty("next".into());
        for s in &mine {
            let p = project_source(conn, s, today, months)?;
            if p.basis == "payslip"
                && let Some(bm) = base_payslip_month(conn, &s.id, today)?
                && forecast_base_month.as_ref().is_none_or(|x| *x < bm)
            {
                forecast_base_month = Some(bm);
            }
            {
                let (b, _) = base_of(conn, s, today)?;
                let range = (
                    s.start_date.as_deref().map(parse_date).transpose()?,
                    s.end_date.as_deref().map(parse_date).transpose()?,
                );
                current.add(&project_month(conn, s, &b, scheme, range, this_month));
            }
            let slips = payslips_by_month(conn, &s.id, &since, &until)?;
            if !slips.is_empty() {
                let (b, _) = base_of(conn, s, today)?;
                let range = (
                    s.start_date.as_deref().map(parse_date).transpose()?,
                    s.end_date.as_deref().map(parse_date).transpose()?,
                );
                for real in slips {
                    let first = parse_date(&format!("{}-01", real.month))?;
                    let e = actual
                        .entry(real.month.clone())
                        .or_insert_with(|| IncomeActual {
                            real: IncomeMonth::empty(real.month.clone()),
                            expected: IncomeMonth::empty(real.month.clone()),
                        });
                    e.expected
                        .add(&project_month(conn, s, &b, scheme, range, first));
                    e.real.add(&real);
                }
            }
            for (i, mm) in p.months.iter().enumerate() {
                next.add(mm);
                monthly[i].add(mm);
            }
            if p.basis == "none" {
                notes.push(t_args(
                    "nucleo-renda-nota-sem-valor",
                    &args(&[("renda", s.name.clone().into())]),
                ));
            }
            projection.push(p);
        }
        let hist = history(conn, &m.id, today)?;
        family_next.add(&next);
        family_hist.add(&hist);
        let eff = (hist.gross > 0)
            .then(|| (hist.social_security + hist.income_tax) as f64 / hist.gross as f64);
        out.push(MemberIncome {
            member_id: m.id.clone(),
            member_name: m.name.clone(),
            sources: mine,
            projection,
            next,
            last_12m: hist,
            effective_tax_rate: eff,
        });
    }
    if sources.iter().any(|s| s.kind.withholding()) {
        notes.push(t(match crate::country::family_country(conn).as_str() {
            "BR" => "nucleo-renda-nota-br",
            "US" => "nucleo-renda-nota-us",
            "PT" => "nucleo-renda-nota-pt",
            "ES" => "nucleo-renda-nota-es",
            "GB" | "CA" | "FR" | "DE" | "AR" | "CL" => "nucleo-renda-nota-adaptado",
            _ => "nucleo-renda-nota-outro",
        }));
    }
    Ok(IncomeSummary {
        from: month_of(add_months(today.with_day(1).expect("dia 1"), 1)),
        months,
        currency: crate::household::base_currency(conn)?,
        members: out,
        monthly,
        next: family_next,
        last_12m: family_hist,
        actual: actual.values().cloned().collect(),
        payslips: payslip_briefs(conn, a.member_id.as_deref(), &since, &until)?,
        forecast_base_month,
        current_month: (!actual.contains_key(&until) && current.gross > 0).then_some(current),
        notes,
    })
}
