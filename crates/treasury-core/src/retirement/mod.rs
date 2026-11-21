//! Contas de aposentadoria (docs/international.md, V137): o tipo de conta `retirement` com o
//! plano do país, e quanto já entrou no ano contra o limite daquele ano. O limite é por
//! pessoa: contas do mesmo grupo (401(k) e Roth 401(k); IRA e Roth IRA) com os mesmos
//! titulares somam juntas. Conta como aporte toda entrada na conta no ano (transferência,
//! aporte ou depósito); o rendimento que o app registra à parte não entra.
//!
//! Os limites ficam aqui, com a fonte de cada ano. Planos sem limite anual (o 529 americano)
//! só mostram o que entrou.

pub mod api;
#[cfg(test)]
mod tests;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::Id;
use crate::time::now_utc;

/// O que o limite mede.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "RetirementLimitKind")]
pub enum LimitKind {
    /// Teto de aportes do ano (401(k), IRA, HSA, plan de pensiones).
    Contribution,
    /// Aporte que já dá todo o benefício fiscal (PPR: 20% até 400/350/300 €).
    TaxBenefit,
}

/// Limite de um ano: o valor-base, faixas por idade (até a idade, inclusive), o adicional a
/// partir de uma idade e o da cobertura familiar (HSA). Em centavos da moeda do plano.
pub struct YearLimit {
    pub year: i32,
    pub base: i64,
    pub by_age: &'static [(u32, i64)],
    pub catch_up: Option<(u32, i64)>,
    pub family: Option<i64>,
    pub source: &'static str,
}

pub struct Plan {
    pub id: &'static str,
    pub country: &'static str,
    /// Chave do nome no catálogo (`nucleo-aposentadoria-plano-*`).
    pub name: &'static str,
    /// Planos do mesmo grupo dividem o limite.
    pub group: &'static str,
    pub currency: &'static str,
    pub kind: LimitKind,
    /// Tem cobertura individual e familiar (HSA).
    pub coverage: bool,
    /// Início do ano do limite (mês, dia): 1º de janeiro, ou 6 de abril no Reino Unido.
    pub year_start: (u32, u32),
    /// Sem limites: o plano não tem teto anual.
    pub limits: &'static [YearLimit],
}

const IRS_2026: &str = "https://www.irs.gov/newsroom/401k-limit-increases-to-24500-for-2026-ira-limit-increases-to-7500";
const IRS_2025: &str =
    "https://www.irs.gov/newsroom/401k-limit-increases-to-23500-for-2025-ira-limit-remains-7000";
const IRS_HSA_2026: &str = "https://www.irs.gov/pub/irs-drop/rp-25-19.pdf";
const IRS_HSA_2025: &str = "https://www.irs.gov/pub/irs-drop/rp-24-25.pdf";
const PT_EBF_21: &str = "https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/bf_rep/Pages/ebf-artigo-21-ordm-.aspx";
const GB_ISA: &str = "https://www.gov.uk/individual-savings-accounts";
const GB_PENSION: &str = "https://www.gov.uk/tax-on-your-private-pension/annual-allowance";
const CA_LIMITS: &str =
    "https://www.advisor.ca/industry-news/industry/essential-tax-numbers-updated-for-2026/";
const DE_RIESTER: &str = "https://www.deutsche-rentenversicherung.de/DE/Rente/Allgemeine-Informationen/Riester-Rente/riester-rente_node.html";
const CL_APV: &str =
    "https://www.spensiones.cl/portal/institucional/594/w3-propertyvalue-9915.html";
const ES_LIRPF_52: &str = "https://www.boe.es/buscar/act.php?id=BOE-A-2006-20764#a52";

const K401: &[YearLimit] = &[
    YearLimit {
        year: 2026,
        base: 2_450_000,
        by_age: &[],
        catch_up: Some((50, 800_000)),
        family: None,
        source: IRS_2026,
    },
    YearLimit {
        year: 2025,
        base: 2_350_000,
        by_age: &[],
        catch_up: Some((50, 750_000)),
        family: None,
        source: IRS_2025,
    },
];

const IRA: &[YearLimit] = &[
    YearLimit {
        year: 2026,
        base: 750_000,
        by_age: &[],
        catch_up: Some((50, 110_000)),
        family: None,
        source: IRS_2026,
    },
    YearLimit {
        year: 2025,
        base: 700_000,
        by_age: &[],
        catch_up: Some((50, 100_000)),
        family: None,
        source: IRS_2025,
    },
];

const HSA: &[YearLimit] = &[
    YearLimit {
        year: 2026,
        base: 440_000,
        by_age: &[],
        catch_up: Some((55, 100_000)),
        family: Some(875_000),
        source: IRS_HSA_2026,
    },
    YearLimit {
        year: 2025,
        base: 430_000,
        by_age: &[],
        catch_up: Some((55, 100_000)),
        family: Some(855_000),
        source: IRS_HSA_2025,
    },
];

/// PPR: 20% do aportado abate ao IRS até 400 € (menos de 35 anos), 350 € (35 a 50) e 300 €
/// (mais de 50); o aporte que dá o abatimento máximo é 5 vezes isso.
const PPR: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 150_000,
    by_age: &[(34, 200_000), (50, 175_000)],
    catch_up: None,
    family: None,
    source: PT_EBF_21,
}];

const ES_INDIVIDUAL: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 150_000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: ES_LIRPF_52,
}];

const ES_EMPLEO: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 850_000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: ES_LIRPF_52,
}];

const GB_ISA_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 2000000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: GB_ISA,
}];
const GB_LISA_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 400000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: GB_ISA,
}];
const GB_PENSION_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 6000000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: GB_PENSION,
}];
const CA_RRSP_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 3381000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: CA_LIMITS,
}];
const CA_TFSA_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 700000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: CA_LIMITS,
}];
const CA_FHSA_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 800000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: CA_LIMITS,
}];
const DE_RIESTER_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 210000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: DE_RIESTER,
}];
/// APV regime B: 600 UF por ano, em pesos com a UF de outubro de 2026 (~41.150).
const CL_APV_L: &[YearLimit] = &[YearLimit {
    year: 2026,
    base: 24690000,
    by_age: &[],
    catch_up: None,
    family: None,
    source: CL_APV,
}];

pub const PLANS: &[Plan] = &[
    Plan {
        id: "us_401k",
        country: "US",
        name: "nucleo-aposentadoria-plano-us-401k",
        group: "us_401k",
        currency: "USD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: K401,
    },
    Plan {
        id: "us_roth_401k",
        country: "US",
        name: "nucleo-aposentadoria-plano-us-roth-401k",
        group: "us_401k",
        currency: "USD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: K401,
    },
    Plan {
        id: "us_ira",
        country: "US",
        name: "nucleo-aposentadoria-plano-us-ira",
        group: "us_ira",
        currency: "USD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: IRA,
    },
    Plan {
        id: "us_roth_ira",
        country: "US",
        name: "nucleo-aposentadoria-plano-us-roth-ira",
        group: "us_ira",
        currency: "USD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: IRA,
    },
    Plan {
        id: "us_hsa",
        country: "US",
        name: "nucleo-aposentadoria-plano-us-hsa",
        group: "us_hsa",
        currency: "USD",
        kind: LimitKind::Contribution,
        coverage: true,
        year_start: (1, 1),
        limits: HSA,
    },
    Plan {
        id: "us_529",
        country: "US",
        name: "nucleo-aposentadoria-plano-us-529",
        group: "us_529",
        currency: "USD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: &[],
    },
    Plan {
        id: "pt_ppr",
        country: "PT",
        name: "nucleo-aposentadoria-plano-pt-ppr",
        group: "pt_ppr",
        currency: "EUR",
        kind: LimitKind::TaxBenefit,
        coverage: false,
        year_start: (1, 1),
        limits: PPR,
    },
    Plan {
        id: "es_plan_pensiones",
        country: "ES",
        name: "nucleo-aposentadoria-plano-es-individual",
        group: "es_individual",
        currency: "EUR",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: ES_INDIVIDUAL,
    },
    Plan {
        id: "es_plan_empleo",
        country: "ES",
        name: "nucleo-aposentadoria-plano-es-empleo",
        group: "es_empleo",
        currency: "EUR",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: ES_EMPLEO,
    },
    Plan {
        id: "gb_isa",
        country: "GB",
        name: "nucleo-aposentadoria-plano-gb-isa",
        group: "gb_isa",
        currency: "GBP",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (4, 6),
        limits: GB_ISA_L,
    },
    Plan {
        id: "gb_lisa",
        country: "GB",
        name: "nucleo-aposentadoria-plano-gb-lisa",
        group: "gb_lisa",
        currency: "GBP",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (4, 6),
        limits: GB_LISA_L,
    },
    Plan {
        id: "gb_pension",
        country: "GB",
        name: "nucleo-aposentadoria-plano-gb-pension",
        group: "gb_pension",
        currency: "GBP",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (4, 6),
        limits: GB_PENSION_L,
    },
    Plan {
        id: "ca_rrsp",
        country: "CA",
        name: "nucleo-aposentadoria-plano-ca-rrsp",
        group: "ca_rrsp",
        currency: "CAD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: CA_RRSP_L,
    },
    Plan {
        id: "ca_tfsa",
        country: "CA",
        name: "nucleo-aposentadoria-plano-ca-tfsa",
        group: "ca_tfsa",
        currency: "CAD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: CA_TFSA_L,
    },
    Plan {
        id: "ca_fhsa",
        country: "CA",
        name: "nucleo-aposentadoria-plano-ca-fhsa",
        group: "ca_fhsa",
        currency: "CAD",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: CA_FHSA_L,
    },
    Plan {
        id: "fr_per",
        country: "FR",
        name: "nucleo-aposentadoria-plano-fr-per",
        group: "fr_per",
        currency: "EUR",
        kind: LimitKind::TaxBenefit,
        coverage: false,
        year_start: (1, 1),
        limits: &[],
    },
    Plan {
        id: "fr_pea",
        country: "FR",
        name: "nucleo-aposentadoria-plano-fr-pea",
        group: "fr_pea",
        currency: "EUR",
        kind: LimitKind::Contribution,
        coverage: false,
        year_start: (1, 1),
        limits: &[],
    },
    Plan {
        id: "de_riester",
        country: "DE",
        name: "nucleo-aposentadoria-plano-de-riester",
        group: "de_riester",
        currency: "EUR",
        kind: LimitKind::TaxBenefit,
        coverage: false,
        year_start: (1, 1),
        limits: DE_RIESTER_L,
    },
    Plan {
        id: "de_ruerup",
        country: "DE",
        name: "nucleo-aposentadoria-plano-de-ruerup",
        group: "de_ruerup",
        currency: "EUR",
        kind: LimitKind::TaxBenefit,
        coverage: false,
        year_start: (1, 1),
        limits: &[],
    },
    Plan {
        id: "cl_apv",
        country: "CL",
        name: "nucleo-aposentadoria-plano-cl-apv",
        group: "cl_apv",
        currency: "CLP",
        kind: LimitKind::TaxBenefit,
        coverage: false,
        year_start: (1, 1),
        limits: CL_APV_L,
    },
];

pub fn plan(id: &str) -> Option<&'static Plan> {
    PLANS.iter().find(|p| p.id == id)
}

/// O limite do ano (o último ano conhecido, se o pedido ainda não tem números).
fn year_limit(p: &Plan, year: i32) -> Option<&'static YearLimit> {
    p.limits
        .iter()
        .filter(|l| l.year <= year)
        .max_by_key(|l| l.year)
        .or_else(|| p.limits.iter().min_by_key(|l| l.year))
}

/// O valor do limite para a idade (no fim do ano) e a cobertura.
fn amount(l: &YearLimit, age: Option<u32>, family: bool) -> i64 {
    let mut v = if family {
        l.family.unwrap_or(l.base)
    } else {
        l.base
    };
    if let Some(a) = age {
        if let Some((_, by)) = l.by_age.iter().find(|(up_to, _)| a <= *up_to) {
            v = *by;
        }
        if let Some((from, extra)) = l.catch_up
            && a >= from
        {
            v += extra;
        }
    }
    v
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementAccountPlan")]
pub struct PlanInfo {
    pub id: String,
    pub name: String,
    pub country: String,
    pub currency: String,
    pub kind: LimitKind,
    pub coverage: bool,
    /// Limite do ano corrente (sem idade nem cobertura familiar); `null`: sem teto anual.
    #[ts(type = "number | null")]
    pub limit: Option<i64>,
}

/// Planos do país (ou todos).
pub fn plans(country: Option<&str>, year: i32) -> Vec<PlanInfo> {
    PLANS
        .iter()
        .filter(|p| country.is_none_or(|c| c == p.country))
        .map(|p| PlanInfo {
            id: p.id.into(),
            name: t(p.name),
            country: p.country.into(),
            currency: p.currency.into(),
            kind: p.kind,
            coverage: p.coverage,
            limit: year_limit(p, year).map(|l| l.base),
        })
        .collect()
}

/// O país tem planos (o tipo de conta aparece).
pub fn has_plans(country: &str) -> bool {
    PLANS.iter().any(|p| p.country == country)
}

/// Grava o plano da conta (criação ou troca).
pub fn set_plan(
    tx: &Transaction<'_>,
    account_id: &str,
    plan_id: &str,
    coverage: Option<&str>,
) -> Result<()> {
    let p = plan(plan_id).ok_or_else(|| Error::invalid(t("nucleo-aposentadoria-erro-plano")))?;
    let coverage = match (p.coverage, coverage.map(str::trim)) {
        (true, None | Some("self")) => Some("self"),
        (true, Some("family")) => Some("family"),
        (_, Some(_)) if p.coverage => {
            return Err(Error::invalid(t("nucleo-aposentadoria-erro-cobertura")));
        }
        _ => None,
    };
    let now = now_utc();
    tx.execute(
        "INSERT INTO retirement_account (account_id, plan, coverage, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT (account_id) DO UPDATE SET plan = excluded.plan, coverage = excluded.coverage,
                updated_at = excluded.updated_at
          WHERE plan IS NOT excluded.plan OR coverage IS NOT excluded.coverage",
        params![account_id, p.id, coverage, now],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "RetirementAccountStatus")]
pub struct Status {
    pub account_id: Id,
    pub plan: PlanInfo,
    pub coverage: Option<String>,
    #[ts(type = "number")]
    pub year: i32,
    /// O que entrou no ano nesta conta e nas outras do mesmo grupo e titulares.
    #[ts(type = "number")]
    pub contributed: i64,
    /// Limite do ano para a idade e a cobertura; `null`: sem teto anual.
    #[ts(type = "number | null")]
    pub limit: Option<i64>,
    #[ts(type = "number | null")]
    pub remaining: Option<i64>,
    /// Outras contas que dividem o limite.
    pub shared_with: Vec<String>,
    /// Idade do titular no fim do ano (com a data de nascimento cadastrada).
    #[ts(type = "number | null")]
    pub age: Option<u32>,
    pub source: Option<String>,
    /// O ano ainda não tem limite publicado: vale o do último ano conhecido.
    pub limit_year: Option<i32>,
}

/// Idade no fim do ano do único titular da conta.
fn owner_age(conn: &Connection, account_id: &str, year: i32) -> Result<Option<u32>> {
    let birth: Option<String> = conn
        .query_row(
            "SELECT m.birth_date FROM account_member am JOIN member m ON m.id = am.member_id
              WHERE am.account_id = ?1 AND (SELECT count(*) FROM account_member x WHERE x.account_id = ?1) = 1",
            [account_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(birth
        .and_then(|b| crate::time::parse_date(&b).ok())
        .map(|b| (year - b.year()).max(0) as u32))
}

fn members_of(conn: &Connection, account_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT member_id FROM account_member WHERE account_id = ?1 ORDER BY member_id")?;
    let rows = stmt
        .query_map([account_id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(rows)
}

/// Situação de todas as contas de aposentadoria abertas (para o agente, que não sabe de antemão
/// quais contas são de aposentadoria).
pub fn statuses(conn: &Connection, today: NaiveDate) -> Result<Vec<Status>> {
    let mut stmt = conn.prepare(
        "SELECT r.account_id FROM retirement_account r JOIN account a ON a.id = r.account_id
          WHERE a.archived = 0 ORDER BY a.name",
    )?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    ids.iter().map(|id| status(conn, id, today)).collect()
}

pub fn status(conn: &Connection, account_id: &str, today: NaiveDate) -> Result<Status> {
    let (plan_id, coverage): (String, Option<String>) = conn
        .query_row(
            "SELECT plan, coverage FROM retirement_account WHERE account_id = ?1",
            [account_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(t("nucleo-aposentadoria-erro-conta")))?;
    let p = plan(&plan_id).ok_or_else(|| Error::invalid(t("nucleo-aposentadoria-erro-plano")))?;
    // O ano do limite: de 1º de janeiro, ou do início do ano fiscal (6 de abril no Reino Unido).
    let (sm, sd) = p.year_start;
    let start_this = NaiveDate::from_ymd_opt(today.year(), sm, sd).expect("data");
    let year = if today >= start_this {
        today.year()
    } else {
        today.year() - 1
    };
    let from = NaiveDate::from_ymd_opt(year, sm, sd).expect("data");
    let to = from
        .checked_add_months(chrono::Months::new(12))
        .and_then(|d| d.pred_opt())
        .expect("data");
    // As contas do mesmo grupo com os mesmos titulares.
    let members = members_of(conn, account_id)?;
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, r.plan FROM retirement_account r JOIN account a ON a.id = r.account_id
          WHERE a.archived = 0",
    )?;
    let all: Vec<(String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut group = vec![account_id.to_string()];
    let mut shared_with = vec![];
    for (id, name, other) in all {
        if id == account_id || plan(&other).is_none_or(|o| o.group != p.group) {
            continue;
        }
        if members_of(conn, &id)? == members {
            group.push(id);
            shared_with.push(name);
        }
    }
    let ids = serde_json::to_string(&group).unwrap_or_default();
    let contributed: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM txn
          WHERE account_id IN (SELECT value FROM json_each(?1)) AND amount > 0
            AND status <> 'planned' AND date BETWEEN ?2 AND ?3",
        params![ids, crate::time::fmt_date(from), crate::time::fmt_date(to)],
        |r| r.get(0),
    )?;
    let age = owner_age(conn, account_id, year)?;
    let yl = year_limit(p, year);
    let limit = yl.map(|l| amount(l, age, coverage.as_deref() == Some("family")));
    Ok(Status {
        account_id: account_id.into(),
        plan: plans(None, year)
            .into_iter()
            .find(|x| x.id == p.id)
            .expect("plano do catálogo"),
        coverage,
        year,
        contributed,
        remaining: limit.map(|l| (l - contributed).max(0)),
        limit,
        shared_with,
        age,
        source: yl.map(|l| l.source.to_string()),
        limit_year: yl.map(|l| l.year).filter(|y| *y != year),
    })
}
