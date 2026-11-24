//! Empréstimos e financiamentos (fase 5): contrato, cronograma (SAC/Price, carência,
//! indexador, seguros e taxas), conciliação das parcelas com os lançamentos, amortização
//! extraordinária (reduzir prazo × parcela), portabilidade e o custo real da dívida.

pub mod api;
pub mod cet;
pub mod link;
pub mod market_rates;
pub mod payoff;
pub mod postfixed;
pub mod schedule;
pub mod statement;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_link;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::indices::Deflator;
use crate::time::{fmt_date, now_utc, parse_date, today_local};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LoanKind {
    Mortgage,
    Vehicle,
    Personal,
    Payroll,
    Student,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AmortSystem {
    Sac,
    Price,
    /// Só juros a cada vencimento e o principal no último (CCB, capital de giro).
    Bullet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LoanIndexer {
    None,
    Tr,
    Ipca,
    Igpm,
    /// Pós-fixado: juros do período pelo CDI acumulado nos dias úteis (× percentual, + spread).
    Cdi,
    /// Pós-fixado pela Selic diária (over), como o CDI.
    Selic,
    /// Correção mensal pelo INPC (IBGE).
    Inpc,
    /// Correção mensal pelo INCC (FGV), comum em imóvel na planta.
    Incc,
    /// Correção pela rentabilidade da poupança na data de aniversário.
    Poupanca,
}

impl LoanIndexer {
    /// Série diária no cofre dos pós-fixados de taxa diária.
    pub fn daily_series(self) -> Option<&'static str> {
        match self {
            LoanIndexer::Cdi => Some("cdi"),
            LoanIndexer::Selic => Some("selic"),
            _ => None,
        }
    }

    /// Nome curto do índice, para mensagens.
    pub fn label(self) -> &'static str {
        match self {
            LoanIndexer::None => "",
            LoanIndexer::Tr => "TR",
            LoanIndexer::Ipca => "IPCA",
            LoanIndexer::Igpm => "IGP-M",
            LoanIndexer::Cdi => "CDI",
            LoanIndexer::Selic => "Selic",
            LoanIndexer::Inpc => "INPC",
            LoanIndexer::Incc => "INCC",
            LoanIndexer::Poupanca => "Poupança",
        }
    }
}

/// Pró-rata do primeiro período quando ele não tem um mês exato (contrato → 1º vencimento).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Prorata {
    /// Sem pró-rata: o primeiro período vale um mês (como antes).
    None,
    /// Por dias corridos: fator^(dias do período ÷ dias do mês cheio).
    Calendar,
    /// Por dias úteis: fator^(du do período ÷ du do mês cheio).
    Business,
}

/// Como a taxa pré (ou os juros reais) entra em cada período.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InterestBasis {
    /// Taxa mensal equivalente à anual, um mês por parcela (como antes).
    Monthly,
    /// `(1 + r)^(du/252)` com os dias úteis de cada período.
    Bd252,
    /// `(1 + r)^(dc/365)` com os dias corridos de cada período.
    Dc365,
}

/// Qual valor do índice corrige cada período.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CorrectionReference {
    /// Variação do mês M − defasagem (IPCA, IGP-M, INPC, INCC).
    Month,
    /// Valor do período que começa na data de aniversário (TR do SFH, poupança).
    Anniversary,
}

/// Forma do spread nos pós de taxa diária (ver [`postfixed`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SpreadMode {
    /// `(1 + s)^(du/252)`.
    Exp252,
    /// `1 + s × dc/360`.
    Lin360,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PrepayMode {
    ReduceTerm,
    ReduceInstallment,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoanInput {
    pub name: String,
    /// Número do contrato como aparece no contrato ou no internet banking.
    #[ts(optional)]
    pub contract_number: Option<String>,
    /// Contrato já em andamento: marca como pagas (no vencimento, pelo valor do cronograma)
    /// as parcelas que venceram antes de hoje.
    #[ts(optional)]
    pub mark_past_paid: Option<bool>,
    pub kind: LoanKind,
    #[ts(optional)]
    pub institution_id: Option<Id>,
    #[ts(type = "number")]
    pub principal: i64,
    /// Taxa efetiva % a.a. (decimal texto, ex.: "11.5"). Vazia: calculada pela parcela.
    pub annual_rate: String,
    /// Valor da parcela (com seguros e taxas mensais) quando a taxa não é informada: com o
    /// prazo e o valor financiado, o núcleo calcula a taxa (ver [`solve_rate`]).
    #[ts(optional, type = "number")]
    pub installment_amount: Option<i64>,
    pub system: AmortSystem,
    #[ts(optional)]
    pub indexer: Option<LoanIndexer>,
    /// Pós de taxa diária: percentual do índice (decimal texto, padrão "100").
    #[ts(optional)]
    pub index_pct: Option<String>,
    #[ts(optional)]
    pub spread_mode: Option<SpreadMode>,
    /// Defasagem do índice em dias úteis (0 a 10).
    #[ts(optional)]
    pub index_lag_days: Option<u32>,
    /// Correção mensal: mês do índice = M − n (padrão 1).
    #[ts(optional)]
    pub correction_lag_months: Option<u32>,
    #[ts(optional)]
    pub correction_prorata: Option<Prorata>,
    /// Meses de deflação não reduzem o saldo.
    #[ts(optional)]
    pub deflation_floor: Option<bool>,
    #[ts(optional)]
    pub interest_basis: Option<InterestBasis>,
    /// Padrão: aniversário na TR e na poupança; mês nos demais.
    #[ts(optional)]
    pub correction_reference: Option<CorrectionReference>,
    pub contract_date: String,
    pub first_due_date: String,
    pub term_months: u32,
    #[ts(optional)]
    pub grace_months: Option<u32>,
    #[ts(optional, type = "number")]
    pub insurance_monthly: Option<i64>,
    #[ts(optional, type = "number")]
    pub fees_monthly: Option<i64>,
    /// Financiamento imobiliário: valor do imóvel (base do DFI), em centavos.
    #[ts(optional, type = "number")]
    pub property_value: Option<i64>,
    /// Seguro MIP em % ao mês sobre o saldo devedor (ex.: "0,0247").
    #[ts(optional)]
    pub mip_rate: Option<String>,
    /// Seguro DFI em % ao mês sobre o valor do imóvel (ex.: "0,0075").
    #[ts(optional)]
    pub dfi_rate: Option<String>,
    /// MIP em valor fixo por mês (centavos), quando o contrato traz valor e não percentual.
    #[ts(optional, type = "number")]
    pub mip_amount: Option<i64>,
    /// DFI em valor fixo por mês (centavos).
    #[ts(optional, type = "number")]
    pub dfi_amount: Option<i64>,
    #[ts(optional)]
    pub cet_annual: Option<String>,
    /// Costs paid on day zero (IOF, one-time fees, single-premium insurance), in cents. They
    /// come out of the amount released in the CET, whether financed or paid apart.
    #[ts(optional, type = "number")]
    pub upfront_iof: Option<i64>,
    #[ts(optional, type = "number")]
    pub upfront_fees: Option<i64>,
    #[ts(optional, type = "number")]
    pub upfront_insurance: Option<i64>,
    #[ts(optional)]
    pub debit_account_id: Option<Id>,
    /// Categoria dos pagamentos: um lançamento nela marca a parcela mais próxima como paga.
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Installment {
    pub id: Id,
    pub number: u32,
    /// Versão do contrato (1 = original; cada amortização extraordinária abre outra, como
    /// os bancos fazem) e o número da parcela dentro dela (recomeça em 1).
    pub version: u32,
    pub version_number: u32,
    pub due_date: String,
    #[ts(type = "number")]
    pub amortization: i64,
    #[ts(type = "number")]
    pub interest: i64,
    #[ts(type = "number")]
    pub adjustment: i64,
    /// Seguros da parcela (fixo + MIP + DFI).
    #[ts(type = "number")]
    pub insurance: i64,
    /// Parte do seguro que é MIP (sobre o saldo devedor).
    #[ts(type = "number")]
    pub insurance_mip: i64,
    /// Parte do seguro que é DFI (sobre o valor do imóvel).
    #[ts(type = "number")]
    pub insurance_dfi: i64,
    #[ts(type = "number")]
    pub fees: i64,
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub balance_after: i64,
    #[ts(type = "number")]
    pub extra_payment: i64,
    pub estimated: bool,
    pub paid: bool,
    pub paid_date: Option<String>,
    #[ts(type = "number | null")]
    pub paid_amount: Option<i64>,
    pub txn_id: Option<Id>,
    /// Lançamento ligado (pagamento): data, descrição e conta, para mostrar na tabela.
    pub txn_date: Option<String>,
    pub txn_description: Option<String>,
    pub txn_account_id: Option<Id>,
    pub txn_account_name: Option<String>,
    pub overdue: bool,
    /// Pós-fixados: fator de juros do período (índice × spread) e dias úteis.
    pub index_factor: Option<String>,
    pub business_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoanView {
    pub id: Id,
    pub name: String,
    pub contract_number: Option<String>,
    pub kind: LoanKind,
    pub institution_id: Option<Id>,
    #[ts(type = "number")]
    pub principal: i64,
    pub currency: String,
    pub annual_rate: String,
    pub monthly_rate: String,
    pub system: AmortSystem,
    pub indexer: LoanIndexer,
    pub index_pct: String,
    pub spread_mode: SpreadMode,
    pub index_lag_days: u32,
    pub correction_lag_months: u32,
    pub correction_prorata: Prorata,
    pub deflation_floor: bool,
    pub interest_basis: InterestBasis,
    pub correction_reference: CorrectionReference,
    pub contract_date: String,
    pub first_due_date: String,
    pub term_months: u32,
    pub grace_months: u32,
    #[ts(type = "number")]
    pub insurance_monthly: i64,
    #[ts(type = "number")]
    pub fees_monthly: i64,
    #[ts(type = "number | null")]
    pub property_value: Option<i64>,
    pub mip_rate: Option<String>,
    pub dfi_rate: Option<String>,
    /// Seguros em valor fixo por mês (centavos), no lugar do percentual.
    #[ts(type = "number | null")]
    pub mip_amount: Option<i64>,
    #[ts(type = "number | null")]
    pub dfi_amount: Option<i64>,
    /// Extrato do contrato importado: parcela até onde os valores são os do banco (a
    /// projeção parte do saldo dela).
    #[ts(type = "number | null")]
    pub statement_until: Option<u32>,
    pub cet_annual: Option<String>,
    /// Costs paid on day zero (IOF, one-time fees, single-premium insurance).
    pub upfront: cet::UpfrontCosts,
    /// CET of the contract as signed, computed by Res. CMN 4.881/2020 (see `debt::cet`).
    pub cet: Option<cet::CetReport>,
    pub debit_account_id: Option<Id>,
    /// Categoria dos pagamentos (liga os lançamentos às parcelas).
    pub category_id: Option<Id>,
    pub member_id: Option<Id>,
    pub archived: bool,
    pub notes: Option<String>,
    /// Saldo devedor hoje (pelo cronograma: depois da última parcela vencida).
    #[ts(type = "number")]
    pub balance: i64,
    pub paid_count: u32,
    pub remaining_count: u32,
    pub overdue_count: u32,
    pub next_installment: Option<Installment>,
    #[ts(type = "number")]
    pub interest_paid: i64,
    #[ts(type = "number")]
    pub interest_remaining: i64,
    /// Seguros + taxas pagos.
    #[ts(type = "number")]
    pub charges_paid: i64,
    /// Custo efetivo realizado (XIRR dos fluxos pagos) ao ano.
    pub realized_cet: Option<f64>,
    /// Balance drops with no payment recorded that the realized CET counted as paid.
    pub realized_drops: Vec<cet::UnrecordedDrop>,
    /// Custo real: realizado descontado pelo IPCA no período.
    pub real_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PrepayArgs {
    pub loan_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub mode: PrepayMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PrepaySimulation {
    pub mode: PrepayMode,
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(type = "number")]
    pub interest_before: i64,
    #[ts(type = "number")]
    pub interest_after: i64,
    #[ts(type = "number")]
    pub interest_saved: i64,
    pub months_before: u32,
    pub months_after: u32,
    #[ts(type = "number")]
    pub next_payment_before: i64,
    #[ts(type = "number")]
    pub next_payment_after: i64,
    pub schedule: Vec<Installment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortabilityArgs {
    pub loan_id: Id,
    /// Nova taxa efetiva % a.a.
    pub annual_rate: String,
    #[ts(optional)]
    pub term_months: Option<u32>,
    #[ts(optional)]
    pub system: Option<AmortSystem>,
    /// Custos da portabilidade (tarifas, avaliação), em centavos.
    #[ts(optional, type = "number")]
    pub costs: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortabilitySimulation {
    #[ts(type = "number")]
    pub balance: i64,
    #[ts(type = "number")]
    pub current_remaining_total: i64,
    #[ts(type = "number")]
    pub new_total: i64,
    #[ts(type = "number")]
    pub costs: i64,
    /// Economia (positivo) ou prejuízo (negativo) total.
    #[ts(type = "number")]
    pub savings: i64,
    #[ts(type = "number")]
    pub current_next_payment: i64,
    #[ts(type = "number")]
    pub new_first_payment: i64,
    pub new_term_months: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DebtSummary {
    #[ts(type = "number")]
    pub total_balance: i64,
    /// Soma das próximas parcelas (compromisso mensal).
    #[ts(type = "number")]
    pub monthly_commitment: i64,
    #[ts(type = "number")]
    pub interest_remaining: i64,
    pub loans: Vec<LoanView>,
}

pub(crate) fn kind_str<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

pub(crate) fn parse_kind<T: serde::de::DeserializeOwned>(s: &str) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Decimal de um campo; `what` é a chave do nome do campo (`nucleo-planejamento-divida-campo-…`).
fn dec(s: &str, what: &str) -> Result<Decimal> {
    s.trim().replace(',', ".").parse().map_err(|_| {
        Error::invalid(t_args(
            "nucleo-planejamento-divida-erro-invalido",
            &args(&[("campo", t(what).into()), ("valor", s.to_string().into())]),
        ))
    })
}

/// "empréstimo <id>", para o erro de não encontrado.
fn loan_ref(id: &str) -> String {
    t_args(
        "nucleo-planejamento-divida-emprestimo",
        &args(&[("id", id.to_string().into())]),
    )
}

struct LoanRow {
    principal: i64,
    annual_rate: Decimal,
    system: AmortSystem,
    indexer: LoanIndexer,
    index_pct: Decimal,
    spread_mode: SpreadMode,
    index_lag: u32,
    correction_lag: u32,
    prorata: Prorata,
    floor: bool,
    basis: InterestBasis,
    reference: CorrectionReference,
    contract_date: NaiveDate,
    first_due: NaiveDate,
    term: u32,
    grace: u32,
    insurance: i64,
    fees: i64,
    /// MIP em fração ao mês sobre o saldo (ou fixo, em centavos); DFI do mês em centavos.
    mip_rate: Decimal,
    mip_fixed: i64,
    dfi: i64,
}

/// Seguros do financiamento imobiliário: valor do imóvel, MIP e DFI em % a.m. e em valor
/// fixo por mês.
struct Housing {
    property: Option<i64>,
    mip_rate: Option<String>,
    dfi_rate: Option<String>,
    mip_amount: Option<i64>,
    dfi_amount: Option<i64>,
}

fn housing_insurance(conn: &Connection, id: &str) -> Result<Housing> {
    Ok(conn.query_row(
        "SELECT property_value, mip_rate, dfi_rate, mip_amount, dfi_amount FROM loan WHERE id = ?1",
        [id],
        |r| {
            Ok(Housing {
                property: r.get(0)?,
                mip_rate: r.get(1)?,
                dfi_rate: r.get(2)?,
                mip_amount: r.get(3)?,
                dfi_amount: r.get(4)?,
            })
        },
    )?)
}

/// Percentual ao mês ("0,0247") em fração; vazio = 0.
fn pct_fraction(v: Option<&str>, what: &str) -> Result<Decimal> {
    match v.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(Decimal::ZERO),
        Some(v) => Ok(dec(v, what)? / Decimal::ONE_HUNDRED),
    }
}

/// Valida as taxas de seguro (% ao mês, de 0 a 5) e devolve o texto normalizado.
fn clean_insurance_rate(v: Option<&str>, what: &str) -> Result<Option<String>> {
    let Some(v) = v.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let d = dec(v, what)?;
    if d < Decimal::ZERO || d > Decimal::from(5) {
        return Err(Error::invalid(t_args(
            "nucleo-planejamento-divida-erro-taxa-seguro",
            &args(&[("campo", t(what).into())]),
        )));
    }
    Ok(Some(d.normalize().to_string()))
}

fn load_row(conn: &Connection, id: &str) -> Result<LoanRow> {
    let r = conn
        .query_row(
            "SELECT principal, annual_rate, system, indexer, first_due_date, term_months, grace_months, insurance_monthly, fees_monthly,
                    index_pct, spread_mode, index_lag_days, contract_date, correction_lag_months,
                    correction_prorata, deflation_floor, interest_basis, correction_reference
             FROM loan WHERE id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    parse_kind::<AmortSystem>(&r.get::<_, String>(2)?)?,
                    parse_kind::<LoanIndexer>(&r.get::<_, String>(3)?)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, u32>(5)?,
                    r.get::<_, u32>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, i64>(8)?,
                    r.get::<_, String>(9)?,
                    parse_kind::<SpreadMode>(&r.get::<_, String>(10)?)?,
                    r.get::<_, u32>(11)?,
                    r.get::<_, String>(12)?,
                    (
                        r.get::<_, u32>(13)?,
                        parse_kind::<Prorata>(&r.get::<_, String>(14)?)?,
                        r.get::<_, i64>(15)? == 1,
                        parse_kind::<InterestBasis>(&r.get::<_, String>(16)?)?,
                        parse_kind::<CorrectionReference>(&r.get::<_, String>(17)?)?,
                    ),
                ))
            },
        )
        .optional()?
        .ok_or_else(|| Error::not_found(loan_ref(id)))?;
    let h = housing_insurance(conn, id)?;
    let dfi = match h.dfi_amount {
        Some(v) => v,
        None => (Decimal::from(h.property.unwrap_or(0))
            * pct_fraction(
                h.dfi_rate.as_deref(),
                "nucleo-planejamento-divida-campo-taxa-dfi",
            )?)
        .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap_or(0),
    };
    Ok(LoanRow {
        mip_rate: if h.mip_amount.is_some() {
            Decimal::ZERO
        } else {
            pct_fraction(
                h.mip_rate.as_deref(),
                "nucleo-planejamento-divida-campo-taxa-mip",
            )?
        },
        mip_fixed: h.mip_amount.unwrap_or(0),
        dfi,
        principal: r.0,
        annual_rate: dec(&r.1, "nucleo-planejamento-divida-campo-taxa")?,
        system: r.2,
        indexer: r.3,
        index_pct: dec(&r.9, "nucleo-planejamento-divida-campo-percentual-indice")?,
        spread_mode: r.10,
        index_lag: r.11,
        contract_date: parse_date(&r.12)?,
        correction_lag: r.13.0,
        prorata: r.13.1,
        floor: r.13.2,
        basis: r.13.3,
        reference: r.13.4,
        first_due: parse_date(&r.4)?,
        term: r.5,
        grace: r.6,
        insurance: r.7,
        fees: r.8,
    })
}

/// Fração do primeiro período em relação a um mês cheio (pró-rata), ou `None` se ele é um
/// mês exato ou o contrato não usa pró-rata.
fn first_period_fraction(l: &LoanRow) -> Option<Decimal> {
    use crate::finance::calendar::business_days_between;
    let full_start = crate::time::add_months(l.first_due, -1);
    if l.prorata == Prorata::None || l.contract_date == full_start {
        return None;
    }
    let (part, full) = match l.prorata {
        Prorata::None => return None,
        Prorata::Calendar => (
            (l.first_due - l.contract_date).num_days(),
            (l.first_due - full_start).num_days(),
        ),
        Prorata::Business => (
            business_days_between(l.contract_date, l.first_due),
            business_days_between(full_start, l.first_due),
        ),
    };
    (full > 0 && part >= 0).then(|| Decimal::from(part) / Decimal::from(full))
}

/// Fatores de correção do saldo por parcela (docs/brazil/floating-rate-loans.md, fases 2 e 3):
/// - por mês: variação do índice no mês M − defasagem (padrão: o mês anterior ao vencimento);
/// - por aniversário (TR do SFH, poupança): valor do período que começa no vencimento
///   anterior (ou na assinatura);
/// - piso zero opcional para deflação e pró-rata do primeiro período.
///
/// Meses sem dado usam o último conhecido e ficam estimados.
fn index_factors(conn: &Connection, l: &LoanRow, count: u32) -> Result<Vec<(Decimal, bool)>> {
    let series = match l.indexer {
        // Sem correção do saldo: os pós de taxa diária entram nos juros de cada período.
        LoanIndexer::None | LoanIndexer::Cdi | LoanIndexer::Selic => return Ok(vec![]),
        LoanIndexer::Ipca => "ipca",
        LoanIndexer::Igpm => "igpm",
        LoanIndexer::Inpc => "inpc",
        LoanIndexer::Incc => "incc",
        LoanIndexer::Tr => "tr",
        LoanIndexer::Poupanca => "poupanca",
    };
    let anniversary =
        l.reference == CorrectionReference::Anniversary || l.indexer == LoanIndexer::Poupanca;
    let lag = l.correction_lag as i32;
    let from = crate::time::add_months(l.contract_date.min(l.first_due), -(lag + 2));
    let to = crate::time::add_months(l.first_due, count as i32);
    let pts = crate::indices::points(conn, series, from, to)?;
    let fraction = first_period_fraction(l);
    let mut out = vec![];
    let mut last = Decimal::ZERO;
    for k in 0..count {
        let found = if anniversary {
            let start = if k == 0 {
                l.contract_date
            } else {
                crate::time::add_months(l.first_due, k as i32 - 1)
            };
            // O valor do dia de aniversário; sem ele (fim de semana na série), o último antes.
            pts.iter()
                .rev()
                .find(|p| p.date == start)
                .map(|p| (p.value, p.estimated))
                .or_else(|| {
                    pts.iter()
                        .rev()
                        .find(|p| p.date <= start && (start - p.date).num_days() <= 3)
                        .map(|p| (p.value, true))
                })
        } else {
            let due = crate::time::add_months(l.first_due, k as i32);
            let ref_month = crate::time::add_months(due, -lag);
            let key = (
                chrono::Datelike::year(&ref_month),
                chrono::Datelike::month(&ref_month),
            );
            pts.iter()
                .rev()
                .find(|p| {
                    (
                        chrono::Datelike::year(&p.date),
                        chrono::Datelike::month(&p.date),
                    ) == key
                })
                .map(|p| (p.value, p.estimated))
        };
        let (value, estimated) = match found {
            Some((v, e)) => {
                last = v;
                (v, e)
            }
            None => (last, true),
        };
        let value = if l.floor {
            value.max(Decimal::ZERO)
        } else {
            value
        };
        let mut factor = Decimal::ONE + value / Decimal::ONE_HUNDRED;
        if k == 0
            && let Some(f) = fraction
        {
            factor = crate::finance::pow_dec(factor, f).round_dp(12);
        }
        out.push((factor, estimated));
    }
    Ok(out)
}

/// Períodos de juros com datas reais nos contratos de taxa pré ou juros reais quando a base
/// não é a mensal fixa: `(1+r)^(du/252)`, `(1+r)^(dc/365)` ou a mensal com pró-rata no 1º
/// período. Vazio quando o cronograma mensal de antes já é o certo.
fn interest_periods(l: &LoanRow, count: u32) -> Vec<schedule::Period> {
    use crate::finance::calendar::business_days_between;
    let fraction = first_period_fraction(l);
    if l.basis == InterestBasis::Monthly && fraction.is_none() {
        return vec![];
    }
    let r = l.annual_rate / Decimal::ONE_HUNDRED;
    let monthly = schedule::monthly_rate(l.annual_rate);
    let mut start = l.contract_date;
    (0..count)
        .map(|k| {
            let due = crate::time::add_months(l.first_due, k as i32);
            let du = business_days_between(start, due).max(0);
            let dc = (due - start).num_days().max(0);
            let factor = match l.basis {
                InterestBasis::Bd252 => crate::finance::pow_dec(
                    Decimal::ONE + r,
                    Decimal::from(du) / Decimal::from(252),
                ),
                InterestBasis::Dc365 => crate::finance::pow_dec(
                    Decimal::ONE + r,
                    Decimal::from(dc) / Decimal::from(365),
                ),
                InterestBasis::Monthly => match fraction {
                    Some(f) if k == 0 => crate::finance::pow_dec(Decimal::ONE + monthly, f),
                    _ => Decimal::ONE + monthly,
                },
            }
            .round_dp(9);
            start = due;
            schedule::Period {
                due,
                factor,
                business_days: du as u32,
                estimated: false,
            }
        })
        .collect()
}

fn extras(
    conn: &Connection,
    id: &str,
    first_due: NaiveDate,
    after: Option<NaiveDate>,
) -> Result<Vec<schedule::Extra>> {
    let mut stmt = conn.prepare(
        "SELECT date, amount, mode FROM loan_extra_payment WHERE loan_id = ?1 AND date > ?2 ORDER BY date",
    )?;
    let since = after.map(fmt_date).unwrap_or_default();
    let rows: Vec<(String, i64, String)> = stmt
        .query_map(params![id, since], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(|(d, amount, mode)| {
            let date = parse_date(&d)?;
            // Entra depois da última parcela vencida até a data.
            let mut after = 0u32;
            while crate::time::add_months(first_due, after as i32) <= date {
                after += 1;
            }
            Ok(schedule::Extra {
                after_number: after,
                amount,
                mode: if mode == "reduce_installment" {
                    schedule::ExtraMode::ReduceInstallment
                } else {
                    schedule::ExtraMode::ReduceTerm
                },
            })
        })
        .collect()
}

pub(crate) fn params_for(
    conn: &Connection,
    id: &str,
    extra: Option<schedule::Extra>,
) -> Result<schedule::Params> {
    let mut l = load_row(conn, id)?;
    // Extrato importado: a projeção começa no último saldo real, com o prazo que falta.
    let rebase = rebase_of(conn, id)?;
    if let Some(rb) = &rebase {
        l.contract_date = crate::time::add_months(l.first_due, rb.number as i32 - 1);
        l.first_due = crate::time::add_months(l.first_due, rb.number as i32);
        l.principal = rb.balance;
        l.term = rb.term;
        l.grace = 0;
        l.prorata = Prorata::None;
    }
    let mut ex = extras(
        conn,
        id,
        l.first_due,
        rebase.as_ref().map(|_| l.contract_date),
    )?;
    if let Some(e) = extra {
        ex.push(e);
    }
    Ok(schedule::Params {
        principal: l.principal,
        annual_rate_pct: l.annual_rate,
        system: sched_system(l.system),
        term: l.term,
        grace: l.grace,
        first_due: l.first_due,
        insurance: l.insurance,
        fees: l.fees,
        mip_rate: l.mip_rate,
        mip_fixed: l.mip_fixed,
        dfi: l.dfi,
        index_factors: index_factors(conn, &l, l.term + 120)?,
        extras: ex,
        periods: match l.indexer.daily_series() {
            Some(series) => postfixed::periods(
                conn,
                &postfixed::Terms {
                    series,
                    pct: l.index_pct,
                    spread_pct: l.annual_rate,
                    spread: match l.spread_mode {
                        SpreadMode::Exp252 => postfixed::Spread::Exp252,
                        SpreadMode::Lin360 => postfixed::Spread::Lin360,
                    },
                    lag: l.index_lag,
                },
                l.contract_date,
                l.first_due,
                l.term + 120,
            )?,
            None => interest_periods(&l, l.term + 120),
        },
    })
}

pub(crate) fn sched_system(s: AmortSystem) -> schedule::System {
    match s {
        AmortSystem::Sac => schedule::System::Sac,
        AmortSystem::Price => schedule::System::Price,
        AmortSystem::Bullet => schedule::System::Bullet,
    }
}

/// Ponto de projeção vindo do extrato do contrato.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Rebase {
    /// Número da última parcela do extrato (as anteriores ficam como estão).
    pub number: u32,
    pub balance: i64,
    pub term: u32,
}

pub(crate) fn rebase_of(conn: &Connection, id: &str) -> Result<Option<Rebase>> {
    let r: (Option<u32>, Option<i64>, Option<u32>) = conn.query_row(
        "SELECT rebase_number, rebase_balance, rebase_term FROM loan WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok(match r {
        (Some(number), Some(balance), Some(term)) => Some(Rebase {
            number,
            balance,
            term,
        }),
        _ => None,
    })
}

/// Cronograma a projetar, com os números das parcelas, e até que número as parcelas
/// existentes ficam intocadas (as do extrato importado).
fn projected(conn: &Connection, id: &str) -> Result<(Vec<schedule::Row>, u32)> {
    let mut rows = schedule::build(&params_for(conn, id, None)?);
    let keep = rebase_of(conn, id)?.map_or(0, |rb| rb.number);
    for r in &mut rows {
        r.number += keep;
    }
    Ok((rows, keep))
}

/// Regrava as parcelas não pagas a partir do cronograma recalculado (as pagas ficam).
pub fn regenerate(tx: &Transaction<'_>, id: &str) -> Result<()> {
    let (rows, keep) = projected(tx, id)?;
    let paid: Vec<u32> = {
        let mut stmt = tx.prepare(
            "SELECT number FROM loan_installment WHERE loan_id = ?1 AND status = 'paid'",
        )?;
        stmt.query_map([id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    tx.execute(
        "DELETE FROM loan_installment WHERE loan_id = ?1 AND status <> 'paid' AND number > ?2",
        params![id, keep],
    )?;
    let now = now_utc();
    let mut stmt = tx.prepare(
        "INSERT INTO loan_installment (id, loan_id, number, due_date, amortization, interest, adjustment, insurance, fees, total,
                                       balance_after, estimated, extra_payment, status, created_at, updated_at,
                                       index_factor, business_days, insurance_mip, insurance_dfi)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'planned', ?14, ?14, ?15, ?16, ?17, ?18)",
    )?;
    for r in rows
        .iter()
        .filter(|r| !paid.contains(&r.number) && r.number > keep)
    {
        stmt.execute(params![
            new_id(),
            id,
            r.number,
            fmt_date(r.due_date),
            r.amortization,
            r.interest,
            r.adjustment,
            r.insurance,
            r.fees,
            r.total,
            r.balance_after,
            r.estimated as i64,
            r.extra_payment,
            now,
            r.index_factor.map(|f| f.normalize().to_string()),
            r.business_days,
            r.mip,
            r.dfi,
        ])?;
    }
    Ok(())
}

/// Recalcula as parcelas em aberto dos contratos indexados (TR, IPCA, IGP-M, CDI, Selic)
/// depois de uma coleta de índices: as estimadas passam a usar os valores publicados. Só
/// grava (e só entra no histórico, como alteração do sistema) quando algo mudou.
pub fn refresh_indexed(conn: &mut Connection) -> Result<u32> {
    let ids: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT id FROM loan WHERE indexer <> 'none' AND archived = 0 ORDER BY created_at",
        )?;
        stmt.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let mut changed = 0;
    for id in ids {
        let (rows, keep) = projected(conn, &id)?;
        let current: Vec<(u32, String, i64, i64, bool)> = {
            let mut stmt = conn.prepare(
                "SELECT number, due_date, total, balance_after, estimated FROM loan_installment
                 WHERE loan_id = ?1 AND status <> 'paid' AND number > ?2 ORDER BY number",
            )?;
            stmt.query_map(params![&id, keep], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get::<_, i64>(4)? == 1,
                ))
            })?
            .collect::<rusqlite::Result<_>>()?
        };
        let paid: Vec<u32> = {
            let mut stmt = conn.prepare(
                "SELECT number FROM loan_installment WHERE loan_id = ?1 AND status = 'paid'",
            )?;
            stmt.query_map([&id], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?
        };
        let fresh: Vec<(u32, String, i64, i64, bool)> = rows
            .iter()
            .filter(|r| !paid.contains(&r.number) && r.number > keep)
            .map(|r| {
                (
                    r.number,
                    fmt_date(r.due_date),
                    r.total,
                    r.balance_after,
                    r.estimated,
                )
            })
            .collect();
        if fresh != current {
            crate::changes::record(
                conn,
                &crate::changes::Author::system(t("nucleo-planejamento-divida-autor-indices")),
                &t("nucleo-planejamento-divida-motivo-indices"),
                |tx| regenerate(tx, &id),
            )?;
            changed += 1;
        }
    }
    Ok(changed)
}

/// Termos do contrato já conferidos (taxa resolvida pela parcela, padrões do índice).
struct Terms {
    rate: Decimal,
    indexer: LoanIndexer,
    index_pct: Decimal,
    correction_lag: u32,
    reference: CorrectionReference,
    lag: u32,
}

/// Taxa escrita com o indexador num campo só, como os bancos informam: "4,75 + TR",
/// "TR + 4,75% a.a.", "2,5 + IPCA", "1,2 + CDI". Devolve o spread (o número, como veio) e o
/// indexador; `None` quando não há "+" com um indexador conhecido.
pub fn split_rate_index(s: &str) -> Option<(String, LoanIndexer)> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    let [a, b] = parts.as_slice() else {
        return None;
    };
    let starts_digit = |x: &str| x.chars().next().is_some_and(|c| c.is_ascii_digit());
    let (num, name) = if starts_digit(a) { (*a, *b) } else { (*b, *a) };
    let name = crate::import::text::fold(name).replace(['-', '.', ' '], "");
    let indexer = match name.as_str() {
        "tr" => LoanIndexer::Tr,
        "ipca" => LoanIndexer::Ipca,
        "igpm" => LoanIndexer::Igpm,
        "inpc" => LoanIndexer::Inpc,
        "incc" => LoanIndexer::Incc,
        "cdi" | "di" => LoanIndexer::Cdi,
        "selic" => LoanIndexer::Selic,
        "poupanca" => LoanIndexer::Poupanca,
        _ => return None,
    };
    // "4,75% a.a." / "4,75 aa" / "4,75 ao ano": só o número.
    let num: String = num
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
        .collect();
    if num.is_empty() {
        return None;
    }
    Some((num, indexer))
}

/// Confere os termos do contrato (cadastro e edição usam a mesma regra).
fn validate_terms(i: &LoanInput) -> Result<Terms> {
    if i.name.trim().is_empty() {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-sem-nome",
        )));
    }
    if i.principal <= 0 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-valor-positivo",
        )));
    }
    // "4,75 + TR" no campo da taxa: o spread vai para a taxa e o TR para o indexador.
    let (annual_rate, indexer) = match split_rate_index(&i.annual_rate) {
        Some((rate, idx)) => {
            if i.indexer
                .is_some_and(|x| x != LoanIndexer::None && x != idx)
            {
                return Err(Error::invalid(t(
                    "nucleo-planejamento-divida-erro-indexador-diferente",
                )));
            }
            (rate, idx)
        }
        None => (
            i.annual_rate.clone(),
            i.indexer.unwrap_or(LoanIndexer::None),
        ),
    };
    let daily = indexer.daily_series().is_some();
    if i.system == AmortSystem::Bullet && i.grace_months.unwrap_or(0) > 0 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-bullet-carencia",
        )));
    }
    let index_pct = match &i.index_pct {
        Some(p) if daily => dec(p, "nucleo-planejamento-divida-campo-percentual-indice")?,
        _ => Decimal::ONE_HUNDRED,
    };
    if index_pct <= Decimal::ZERO || index_pct > Decimal::from(1000) {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-percentual-indice",
        )));
    }
    let correction_lag = i.correction_lag_months.unwrap_or(1);
    if correction_lag > 3 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-defasagem-mensal",
        )));
    }
    // TR do SFH e poupança corrigem pelo período que começa no aniversário do contrato.
    let reference = i.correction_reference.unwrap_or(match indexer {
        LoanIndexer::Tr | LoanIndexer::Poupanca => CorrectionReference::Anniversary,
        _ => CorrectionReference::Month,
    });
    let lag = i.index_lag_days.unwrap_or(0);
    if lag > 10 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-defasagem-diaria",
        )));
    }
    if daily && annual_rate.trim().is_empty() && i.installment_amount.is_some() {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-posfixado-spread",
        )));
    }
    let rate = if daily && annual_rate.trim().is_empty() {
        Decimal::ZERO
    } else if annual_rate.trim().is_empty() {
        let Some(payment) = i.installment_amount else {
            return Err(Error::invalid(t(
                "nucleo-planejamento-divida-erro-taxa-ou-parcela",
            )));
        };
        solve_rate(&RateArgs {
            principal: i.principal,
            installment_amount: payment,
            term_months: i.term_months,
            grace_months: i.grace_months,
            system: i.system,
            insurance_monthly: i.insurance_monthly,
            fees_monthly: i.fees_monthly,
        })?
        .annual_rate
        .parse::<Decimal>()
        .map_err(|_| Error::invalid(t("nucleo-planejamento-divida-erro-taxa-calculada")))?
    } else {
        dec(&annual_rate, "nucleo-planejamento-divida-campo-taxa")?
    };
    if (!daily && rate < Decimal::ZERO) || rate < Decimal::from(-100) || rate > Decimal::from(1000)
    {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-taxa-intervalo",
        )));
    }
    let (cd, fd) = (
        parse_date(&i.contract_date)?,
        parse_date(&i.first_due_date)?,
    );
    if fd < cd {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-primeira-parcela",
        )));
    }
    if i.term_months == 0 || i.grace_months.unwrap_or(0) >= i.term_months {
        return Err(Error::invalid(t("nucleo-planejamento-divida-erro-prazo")));
    }
    if let Some(c) = &i.cet_annual {
        dec(c, "nucleo-planejamento-divida-campo-cet")?;
    }
    Ok(Terms {
        rate,
        indexer,
        index_pct,
        correction_lag,
        reference,
        lag,
    })
}

pub fn create(tx: &Transaction<'_>, i: &LoanInput) -> Result<Id> {
    let Terms {
        rate,
        indexer,
        index_pct,
        correction_lag,
        reference,
        lag,
    } = validate_terms(i)?;
    // Moeda do contrato: a da conta de débito; sem ela, a moeda base da família.
    let currency = match &i.debit_account_id {
        Some(acc) => crate::accounts::currency_and_kind(tx, acc)?.0,
        None => crate::household::base_currency(tx)?,
    };
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO loan (id, name, kind, institution_id, principal, annual_rate, system, indexer, contract_date, first_due_date,
                           term_months, grace_months, insurance_monthly, fees_monthly, cet_annual, debit_account_id, member_id,
                           notes, created_at, updated_at, contract_number, index_pct, spread_mode, index_lag_days,
                           correction_lag_months, correction_prorata, deflation_floor, interest_basis, correction_reference,
                           currency)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?19, ?20, ?21, ?22, ?23,
                 ?24, ?25, ?26, ?27, ?28, ?29)",
        params![
            id,
            i.name.trim(),
            kind_str(&i.kind),
            i.institution_id,
            i.principal,
            rate.normalize().to_string(),
            kind_str(&i.system),
            kind_str(&indexer),
            i.contract_date,
            i.first_due_date,
            i.term_months,
            i.grace_months.unwrap_or(0),
            i.insurance_monthly.unwrap_or(0),
            i.fees_monthly.unwrap_or(0),
            i.cet_annual,
            i.debit_account_id,
            i.member_id,
            i.notes,
            now,
            clean_contract_number(i.contract_number.as_deref()),
            index_pct.normalize().to_string(),
            kind_str(&i.spread_mode.unwrap_or(SpreadMode::Exp252)),
            lag,
            correction_lag,
            kind_str(&i.correction_prorata.unwrap_or(Prorata::None)),
            i.deflation_floor.unwrap_or(false) as i64,
            kind_str(&i.interest_basis.unwrap_or(InterestBasis::Monthly)),
            kind_str(&reference),
            currency,
        ],
    )?;
    if let Some(c) = &i.category_id {
        set_category(tx, &id, Some(c))?;
    }
    set_housing_insurance(
        tx,
        &id,
        &HousingInput {
            property_value: i.property_value,
            mip_rate: i.mip_rate.as_deref(),
            dfi_rate: i.dfi_rate.as_deref(),
            mip_amount: i.mip_amount,
            dfi_amount: i.dfi_amount,
        },
    )?;
    set_upfront_costs(tx, &id, i)?;
    regenerate(tx, &id)?;
    if i.mark_past_paid.unwrap_or(false) {
        mark_paid_until(tx, &id, &fmt_date(today_local()))?;
    }
    Ok(id)
}

/// Edita todos os termos de um contrato e recalcula o cronograma. O que o usuário
/// registrou à mão fica: parcelas pagas (data, valor pago e lançamento), amortizações
/// extraordinárias e antecipações; só as parcelas em aberto são refeitas.
pub fn update_terms(tx: &Transaction<'_>, id: &str, i: &LoanInput) -> Result<()> {
    let Terms {
        rate,
        indexer,
        index_pct,
        correction_lag,
        reference,
        lag,
    } = validate_terms(i)?;
    let n = tx.execute(
        "UPDATE loan SET name = ?2, kind = ?3, institution_id = ?4, principal = ?5, annual_rate = ?6, system = ?7,
                         indexer = ?8, contract_date = ?9, first_due_date = ?10, term_months = ?11, grace_months = ?12,
                         insurance_monthly = ?13, fees_monthly = ?14, cet_annual = ?15, debit_account_id = ?16,
                         member_id = ?17, notes = ?18, updated_at = ?19, contract_number = ?20, index_pct = ?21,
                         spread_mode = ?22, index_lag_days = ?23, correction_lag_months = ?24,
                         correction_prorata = ?25, deflation_floor = ?26, interest_basis = ?27,
                         correction_reference = ?28
          WHERE id = ?1",
        params![
            id,
            i.name.trim(),
            kind_str(&i.kind),
            i.institution_id,
            i.principal,
            rate.normalize().to_string(),
            kind_str(&i.system),
            kind_str(&indexer),
            i.contract_date,
            i.first_due_date,
            i.term_months,
            i.grace_months.unwrap_or(0),
            i.insurance_monthly.unwrap_or(0),
            i.fees_monthly.unwrap_or(0),
            i.cet_annual,
            i.debit_account_id,
            i.member_id,
            i.notes,
            now_utc(),
            clean_contract_number(i.contract_number.as_deref()),
            index_pct.normalize().to_string(),
            kind_str(&i.spread_mode.unwrap_or(SpreadMode::Exp252)),
            lag,
            correction_lag,
            kind_str(&i.correction_prorata.unwrap_or(Prorata::None)),
            i.deflation_floor.unwrap_or(false) as i64,
            kind_str(&i.interest_basis.unwrap_or(InterestBasis::Monthly)),
            kind_str(&reference),
        ],
    )?;
    if n == 0 {
        return Err(Error::not_found(t("nucleo-planejamento-divida-contrato")));
    }
    set_category(tx, id, i.category_id.as_deref())?;
    set_housing_insurance(
        tx,
        id,
        &HousingInput {
            property_value: i.property_value,
            mip_rate: i.mip_rate.as_deref(),
            dfi_rate: i.dfi_rate.as_deref(),
            mip_amount: i.mip_amount,
            dfi_amount: i.dfi_amount,
        },
    )?;
    set_upfront_costs(tx, id, i)?;
    regenerate(tx, id)?;
    if i.mark_past_paid.unwrap_or(false) {
        mark_paid_until(tx, id, &fmt_date(today_local()))?;
    }
    Ok(())
}

/// Marca como pagas, no próprio vencimento e pelo valor do cronograma, as parcelas em
/// aberto que venceram antes de `before`. Devolve quantas.
pub fn mark_paid_until(tx: &Transaction<'_>, id: &str, before: &str) -> Result<u32> {
    let n = tx.execute(
        "UPDATE loan_installment SET status = 'paid', paid_date = due_date, paid_amount = total, updated_at = ?3
         WHERE loan_id = ?1 AND status <> 'paid' AND due_date < ?2",
        params![id, before, now_utc()],
    )?;
    Ok(n as u32)
}

/// Ordem da antecipação: das últimas parcelas para as primeiras (encurta o contrato) ou das
/// próximas em diante (adianta as que vêm a seguir).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AnticipateOrder {
    Last,
    Next,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnticipateArgs {
    pub loan_id: Id,
    /// Data do pagamento antecipado.
    pub date: String,
    /// Quantas parcelas antecipar.
    pub count: u32,
    pub order: AnticipateOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnticipatedInstallment {
    pub number: u32,
    pub due_date: String,
    /// Valor da parcela no cronograma, sem seguros e taxas (centavos).
    #[ts(type = "number")]
    pub nominal: i64,
    /// Valor presente na data do pagamento (centavos).
    #[ts(type = "number")]
    pub present_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnticipationSimulation {
    pub installments: Vec<AnticipatedInstallment>,
    /// Soma das parcelas no cronograma, sem seguros e taxas.
    #[ts(type = "number")]
    pub nominal_total: i64,
    /// Quanto pagar hoje.
    #[ts(type = "number")]
    pub to_pay: i64,
    /// Desconto dos juros (nominal − valor presente).
    #[ts(type = "number")]
    pub discount: i64,
    /// Seguros e taxas das parcelas antecipadas que deixam de ser cobrados.
    #[ts(type = "number")]
    pub charges_dropped: i64,
    /// Parcelas que continuam em aberto depois.
    pub remaining_after: u32,
}

/// Antecipação de parcelas com desconto dos juros (Resolução CMN 3.516): cada parcela é
/// trazida a valor presente pela taxa do contrato, por dias corridos ÷ 30
/// (`P / (1 + i)^(dc/30)`). Seguros e taxas mensais das parcelas antecipadas não são
/// cobrados. Nos pós-fixados a taxa é a do período de cada parcela (estimada no futuro).
pub fn simulate_anticipation(
    conn: &Connection,
    a: &AnticipateArgs,
) -> Result<AnticipationSimulation> {
    let date = parse_date(&a.date)?;
    if a.count == 0 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-quantas-antecipar",
        )));
    }
    let l = load_row(conn, &a.loan_id)?;
    let monthly = schedule::monthly_rate(l.annual_rate);
    let inst = installments(conn, &a.loan_id)?;
    let mut open: Vec<&Installment> = inst
        .iter()
        .filter(|i| !i.paid && i.due_date.as_str() > a.date.as_str())
        .collect();
    if open.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-sem-futuras",
        )));
    }
    if a.count as usize > open.len() {
        return Err(Error::invalid(t_args(
            "nucleo-planejamento-divida-erro-poucas-futuras",
            &args(&[("n", open.len().into())]),
        )));
    }
    let picked: Vec<&Installment> = match a.order {
        AnticipateOrder::Next => open.drain(..a.count as usize).collect(),
        AnticipateOrder::Last => {
            let start = open.len() - a.count as usize;
            open.drain(start..).collect()
        }
    };
    let mut out = vec![];
    let (mut nominal_total, mut pv_total, mut charges) = (0i64, 0i64, 0i64);
    for i in &picked {
        let due = parse_date(&i.due_date)?;
        let dc = (due - date).num_days().max(0);
        let rate = i
            .index_factor
            .as_deref()
            .and_then(|f| f.parse::<Decimal>().ok())
            .map(|f| f - Decimal::ONE)
            .unwrap_or(monthly);
        let nominal = i.total - i.insurance - i.fees;
        let factor =
            crate::finance::pow_dec(Decimal::ONE + rate, Decimal::from(dc) / Decimal::from(30));
        let pv = (Decimal::from(nominal) / factor)
            .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
            .to_i64()
            .unwrap_or(nominal);
        nominal_total += nominal;
        pv_total += pv;
        charges += i.insurance + i.fees;
        out.push(AnticipatedInstallment {
            number: i.number,
            due_date: i.due_date.clone(),
            nominal,
            present_value: pv,
        });
    }
    out.sort_by_key(|x| x.number);
    Ok(AnticipationSimulation {
        installments: out,
        nominal_total,
        to_pay: pv_total,
        discount: nominal_total - pv_total,
        charges_dropped: charges,
        remaining_after: open.len() as u32,
    })
}

/// Registra a antecipação: as parcelas escolhidas ficam pagas na data, pelo valor presente.
pub fn apply_anticipation(
    tx: &Transaction<'_>,
    a: &AnticipateArgs,
    txn_id: Option<&str>,
) -> Result<AnticipationSimulation> {
    let sim = simulate_anticipation(tx, a)?;
    let now = now_utc();
    for i in &sim.installments {
        tx.execute(
            "UPDATE loan_installment SET status = 'paid', paid_date = ?3, paid_amount = ?4, txn_id = ?5, updated_at = ?6
             WHERE loan_id = ?1 AND number = ?2",
            params![a.loan_id, i.number, a.date, i.present_value, txn_id, now],
        )?;
    }
    Ok(sim)
}

/// Número do contrato sem espaços nas pontas; vazio vira ausente.
pub(crate) fn clean_contract_number(n: Option<&str>) -> Option<String> {
    n.map(str::trim)
        .filter(|n| !n.is_empty())
        .map(|n| n.chars().take(80).collect())
}

/// Dados para calcular a taxa de um contrato pela parcela.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RateArgs {
    #[ts(type = "number")]
    pub principal: i64,
    /// Parcela paga por mês (com seguros e taxas mensais, que são descontados). No SAC, a
    /// primeira parcela depois da carência.
    #[ts(type = "number")]
    pub installment_amount: i64,
    pub term_months: u32,
    #[ts(optional)]
    pub grace_months: Option<u32>,
    pub system: AmortSystem,
    #[ts(optional, type = "number")]
    pub insurance_monthly: Option<i64>,
    #[ts(optional, type = "number")]
    pub fees_monthly: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RateSolution {
    /// % a.a. efetiva (decimal texto).
    pub annual_rate: String,
    /// % a.m. (decimal texto).
    pub monthly_rate: String,
    /// Total pago no prazo, sem seguros e taxas (centavos), pela tabela escolhida.
    #[ts(type = "number")]
    pub total_paid: i64,
    #[ts(type = "number")]
    pub total_interest: i64,
}

/// Calcula a taxa do contrato a partir do valor financiado, da parcela e do prazo, no mesmo
/// modelo do cronograma (períodos mensais a partir do primeiro vencimento). Assim, o
/// cronograma gerado reproduz a parcela informada, a menos de centavos de arredondamento.
pub fn solve_rate(a: &RateArgs) -> Result<RateSolution> {
    let grace = a.grace_months.unwrap_or(0);
    if a.term_months == 0 || grace >= a.term_months {
        return Err(Error::invalid(t("nucleo-planejamento-divida-erro-prazo")));
    }
    if a.principal <= 0 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-valor-positivo",
        )));
    }
    let net = a.installment_amount - a.insurance_monthly.unwrap_or(0) - a.fees_monthly.unwrap_or(0);
    if net <= 0 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-parcela-seguros",
        )));
    }
    let system = sched_system(a.system);
    let n = a.term_months - grace;
    let i = schedule::solve_monthly_rate(a.principal, net, n, system)
        .ok_or_else(|| Error::invalid(t("nucleo-planejamento-divida-erro-taxa-nao-fecha")))?;
    let annual = schedule::annual_pct_from_monthly(i);
    let rows = schedule::build(&schedule::Params {
        principal: a.principal,
        annual_rate_pct: annual,
        system,
        term: a.term_months,
        grace,
        first_due: NaiveDate::from_ymd_opt(2000, 1, 1).expect("data fixa"),
        insurance: 0,
        fees: 0,
        mip_rate: Decimal::ZERO,
        mip_fixed: 0,
        dfi: 0,
        index_factors: vec![],
        periods: vec![],
        extras: vec![],
    });
    Ok(RateSolution {
        annual_rate: annual.normalize().to_string(),
        monthly_rate: (i * Decimal::ONE_HUNDRED)
            .round_dp(6)
            .normalize()
            .to_string(),
        total_paid: rows.iter().map(|r| r.total).sum(),
        total_interest: rows.iter().map(|r| r.interest).sum(),
    })
}

fn row_installment(r: &rusqlite::Row<'_>, today: &str) -> rusqlite::Result<Installment> {
    let paid = r.get::<_, String>("status")? == "paid";
    let due: String = r.get("due_date")?;
    Ok(Installment {
        id: r.get("id")?,
        number: r.get("number")?,
        version: 1,
        version_number: 0,
        overdue: !paid && due.as_str() < today,
        due_date: due,
        amortization: r.get("amortization")?,
        interest: r.get("interest")?,
        adjustment: r.get("adjustment")?,
        insurance: r.get("insurance")?,
        insurance_mip: r.get("insurance_mip")?,
        insurance_dfi: r.get("insurance_dfi")?,
        fees: r.get("fees")?,
        total: r.get("total")?,
        balance_after: r.get("balance_after")?,
        extra_payment: r.get("extra_payment")?,
        estimated: r.get::<_, i64>("estimated")? == 1,
        paid,
        paid_date: r.get("paid_date")?,
        paid_amount: r.get("paid_amount")?,
        txn_id: r.get("txn_id")?,
        txn_date: r.get("txn_date")?,
        txn_description: r.get("txn_description")?,
        txn_account_id: r.get("txn_account_id")?,
        txn_account_name: r.get("txn_account_name")?,
        index_factor: r.get("index_factor")?,
        business_days: r.get("business_days")?,
    })
}

/// Parcelas com o lançamento ligado (data, descrição e conta) para [`row_installment`].
const INSTALLMENT_SELECT: &str = "SELECT li.*, coalesce(li.source, '') AS src, t.date AS txn_date,
        t.description AS txn_description, t.account_id AS txn_account_id, a.name AS txn_account_name
   FROM loan_installment li
   LEFT JOIN txn t ON t.id = li.txn_id
   LEFT JOIN account a ON a.id = t.account_id";

pub fn installments(conn: &Connection, id: &str) -> Result<Vec<Installment>> {
    let today = fmt_date(today_local());
    let mut stmt = conn.prepare(&format!(
        "{INSTALLMENT_SELECT} WHERE li.loan_id = ?1 ORDER BY li.number"
    ))?;
    let rows: Vec<(Installment, bool)> = stmt
        .query_map([id], |r| {
            Ok((
                row_installment(r, &today)?,
                r.get::<_, String>("src")? == "statement",
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    // Versões do contrato: a amortização do extrato do banco fica na primeira parcela da
    // versão nova; a do cronograma, na parcela depois da qual ela entra.
    let mut out = Vec::with_capacity(rows.len());
    let (mut version, mut k) = (1u32, 0u32);
    let mut prev_schedule_extra = false;
    for (mut inst, statement) in rows {
        let starts = (statement && inst.extra_payment > 0) || prev_schedule_extra;
        if starts && k > 0 {
            version += 1;
            k = 0;
        }
        k += 1;
        inst.version = version;
        inst.version_number = k;
        prev_schedule_extra = !statement && inst.extra_payment > 0;
        out.push(inst);
    }
    Ok(out)
}

/// Pagamento de uma parcela como o banco mostra (ex.: num print): valor pago e, se houver,
/// o saldo devedor depois dele e a decomposição. Com o saldo, a projeção passa a partir
/// dele (sincroniza o saldo); com a amortização também, uma amortização extraordinária é
/// achada pelo salto do saldo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PaymentArgs {
    /// Contrato.
    pub id: Id,
    /// Parcela (sem ela, a do mês da data).
    #[serde(default)]
    #[ts(optional)]
    pub installment_id: Option<Id>,
    pub date: String,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    /// Saldo devedor depois do pagamento.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub balance_after: Option<i64>,
    /// Parcelas que faltam depois desta (muda depois de amortização reduzindo o prazo).
    #[serde(default)]
    #[ts(optional)]
    pub remaining_term: Option<u32>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub amortization: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub interest: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub insurance: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub fees: Option<i64>,
    /// Correção monetária do mês (TR…).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub adjustment: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub txn_id: Option<Id>,
}

pub fn record_payment(tx: &Transaction<'_>, a: &PaymentArgs) -> Result<Installment> {
    let date = parse_date(&a.date)?;
    let first_due = parse_date(
        &tx.query_row(
            "SELECT first_due_date FROM loan WHERE id = ?1",
            [&a.id],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(format!("empréstimo {}", a.id)))?,
    )?;
    let (iid, number): (String, u32) = match &a.installment_id {
        Some(i) => tx
            .query_row(
                "SELECT id, number FROM loan_installment WHERE id = ?1 AND loan_id = ?2",
                params![i, a.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| {
                Error::not_found(crate::i18n::t_args(
                    "nucleo-planejamento-divida-parcela-n",
                    &crate::i18n::args(&[("n", i.to_string().into())]),
                ))
            })?,
        None => {
            let n = statement::number_for(first_due, date).ok_or_else(|| {
                Error::invalid(t("nucleo-planejamento-divida-erro-pagamento-antes"))
            })?;
            tx.query_row(
                "SELECT id, number FROM loan_installment WHERE loan_id = ?1 AND number = ?2",
                params![a.id, n],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| Error::not_found(t("nucleo-planejamento-divida-parcela-do-mes")))?
        }
    };
    if a.balance_after.is_some_and(|b| b < 0) || a.amount.is_some_and(|v| v < 0) {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-pagamento-negativo",
        )));
    }
    let now = now_utc();
    let set = |col: &str, v: Option<i64>| -> Result<()> {
        if let Some(v) = v {
            tx.execute(
                &format!("UPDATE loan_installment SET {col} = ?2 WHERE id = ?1"),
                params![iid, v],
            )?;
        }
        Ok(())
    };
    set("amortization", a.amortization)?;
    set("interest", a.interest)?;
    set("insurance", a.insurance)?;
    set("fees", a.fees)?;
    set("adjustment", a.adjustment)?;
    if a.amortization.is_some() || a.interest.is_some() {
        tx.execute(
            "UPDATE loan_installment SET total = amortization + interest + insurance + fees WHERE id = ?1",
            [&iid],
        )?;
    }
    tx.execute(
        "UPDATE loan_installment SET status = 'paid', paid_date = ?2, paid_amount = coalesce(?3, total),
                txn_id = coalesce(?4, txn_id), updated_at = ?5
          WHERE id = ?1",
        params![iid, fmt_date(date), a.amount, a.txn_id, now],
    )?;
    if let Some(bal) = a.balance_after {
        // Amortização extraordinária: o saldo caiu além da amortização da parcela.
        let prev: Option<i64> = tx
            .query_row(
                "SELECT balance_after FROM loan_installment WHERE loan_id = ?1 AND number = ?2",
                params![a.id, number.saturating_sub(1)],
                |r| r.get(0),
            )
            .optional()?;
        let (amort, adj): (i64, i64) = tx.query_row(
            "SELECT amortization, adjustment FROM loan_installment WHERE id = ?1",
            [&iid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let extra = match (prev, a.amortization) {
            (Some(p), Some(_)) => (p + adj - amort - bal).max(0),
            _ => 0,
        };
        tx.execute(
            "UPDATE loan_installment SET balance_after = ?2, estimated = 0, source = 'statement',
                    extra_payment = CASE WHEN ?3 > 5000 THEN ?3 ELSE extra_payment END
              WHERE id = ?1",
            params![iid, bal, extra],
        )?;
        // Sincroniza: a projeção parte deste saldo (só para a frente).
        let current = rebase_of(tx, &a.id)?;
        if current.is_none_or(|rb| rb.number <= number) {
            let term = match a.remaining_term {
                Some(t) => t,
                None => tx.query_row(
                    "SELECT count(*) FROM loan_installment WHERE loan_id = ?1 AND number > ?2",
                    params![a.id, number],
                    |r| r.get(0),
                )?,
            };
            if term == 0 && bal > 0 {
                return Err(Error::invalid(t(
                    "nucleo-planejamento-divida-erro-saldo-restante",
                )));
            }
            tx.execute(
                "UPDATE loan SET rebase_number = ?2, rebase_balance = ?3, rebase_term = ?4, updated_at = ?5 WHERE id = ?1",
                params![a.id, number, bal, term, now],
            )?;
            regenerate(tx, &a.id)?;
        }
    }
    let today = fmt_date(today_local());
    Ok(tx.query_row(
        &format!("{INSTALLMENT_SELECT} WHERE li.id = ?1"),
        [&iid],
        |r| row_installment(r, &today),
    )?)
}

/// Saldo devedor numa data pelo cronograma (depois da última parcela vencida até lá).
pub fn balance_at(conn: &Connection, id: &str, at: &str) -> Result<i64> {
    let (principal, contract): (i64, String) = conn.query_row(
        "SELECT principal, contract_date FROM loan WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if contract.as_str() > at {
        return Ok(0);
    }
    let last: Option<i64> = conn
        .query_row(
            "SELECT balance_after FROM loan_installment WHERE loan_id = ?1 AND due_date <= ?2 ORDER BY number DESC LIMIT 1",
            params![id, at],
            |r| r.get(0),
        )
        .optional()?;
    // Parcelas antecipadas (pagas antes do vencimento): o principal delas já saiu do saldo.
    let anticipated: i64 = conn.query_row(
        "SELECT coalesce(sum(amortization), 0) FROM loan_installment
         WHERE loan_id = ?1 AND status = 'paid' AND due_date > ?2 AND paid_date <= ?2",
        params![id, at],
        |r| r.get(0),
    )?;
    Ok((last.unwrap_or(principal) - anticipated).max(0))
}

/// Soma dos saldos devedores de todos os contratos numa data (passivo do patrimônio).
/// Saldo devedor numa data dos contratos de um membro (titular do contrato).
pub fn member_balance_at(conn: &Connection, member_id: &str, at: &str) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT id FROM loan WHERE archived = 0 AND member_id = ?1")?;
    let ids: Vec<String> = stmt
        .query_map([member_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    ids.iter().map(|id| balance_at(conn, id, at)).sum()
}

pub fn total_balance_at(conn: &Connection, at: &str) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT id FROM loan WHERE archived = 0")?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    ids.iter().map(|id| balance_at(conn, id, at)).sum()
}

pub fn view(conn: &Connection, id: &str) -> Result<LoanView> {
    let inst = installments(conn, id)?;
    let today = fmt_date(today_local());
    let base = conn
        .query_row("SELECT * FROM loan WHERE id = ?1", [id], |r| {
            Ok(LoanView {
                id: r.get("id")?,
                name: r.get("name")?,
                contract_number: r.get("contract_number")?,
                kind: parse_kind(&r.get::<_, String>("kind")?)?,
                institution_id: r.get("institution_id")?,
                principal: r.get("principal")?,
                currency: r.get("currency")?,
                annual_rate: r.get("annual_rate")?,
                monthly_rate: String::new(),
                system: parse_kind(&r.get::<_, String>("system")?)?,
                indexer: parse_kind(&r.get::<_, String>("indexer")?)?,
                index_pct: r.get("index_pct")?,
                spread_mode: parse_kind(&r.get::<_, String>("spread_mode")?)?,
                index_lag_days: r.get("index_lag_days")?,
                correction_lag_months: r.get("correction_lag_months")?,
                correction_prorata: parse_kind(&r.get::<_, String>("correction_prorata")?)?,
                deflation_floor: r.get::<_, i64>("deflation_floor")? == 1,
                interest_basis: parse_kind(&r.get::<_, String>("interest_basis")?)?,
                correction_reference: parse_kind(&r.get::<_, String>("correction_reference")?)?,
                contract_date: r.get("contract_date")?,
                first_due_date: r.get("first_due_date")?,
                term_months: r.get("term_months")?,
                grace_months: r.get("grace_months")?,
                insurance_monthly: r.get("insurance_monthly")?,
                fees_monthly: r.get("fees_monthly")?,
                property_value: r.get("property_value")?,
                mip_rate: r.get("mip_rate")?,
                dfi_rate: r.get("dfi_rate")?,
                mip_amount: r.get("mip_amount")?,
                dfi_amount: r.get("dfi_amount")?,
                statement_until: r.get("rebase_number")?,
                cet_annual: r.get("cet_annual")?,
                upfront: cet::UpfrontCosts {
                    iof: r.get::<_, Option<i64>>("upfront_iof")?.unwrap_or(0),
                    fees: r.get::<_, Option<i64>>("upfront_fees")?.unwrap_or(0),
                    insurance: r.get::<_, Option<i64>>("upfront_insurance")?.unwrap_or(0),
                },
                cet: None,
                debit_account_id: r.get("debit_account_id")?,
                category_id: r.get("category_id")?,
                member_id: r.get("member_id")?,
                archived: r.get::<_, i64>("archived")? == 1,
                notes: r.get("notes")?,
                balance: 0,
                paid_count: 0,
                remaining_count: 0,
                overdue_count: 0,
                next_installment: None,
                interest_paid: 0,
                interest_remaining: 0,
                charges_paid: 0,
                realized_cet: None,
                realized_drops: vec![],
                real_cost: None,
            })
        })
        .optional()?
        .ok_or_else(|| Error::not_found(loan_ref(id)))?;
    let mut v = base;
    let rate = dec(&v.annual_rate, "nucleo-planejamento-divida-campo-taxa")?;
    v.monthly_rate = (schedule::monthly_rate(rate) * Decimal::ONE_HUNDRED)
        .round_dp(4)
        .normalize()
        .to_string();
    v.balance = balance_at(conn, id, &today)?;
    v.paid_count = inst.iter().filter(|i| i.paid).count() as u32;
    v.remaining_count = inst.iter().filter(|i| !i.paid).count() as u32;
    v.overdue_count = inst.iter().filter(|i| i.overdue).count() as u32;
    v.next_installment = inst.iter().find(|i| !i.paid).cloned();
    v.interest_paid = inst.iter().filter(|i| i.paid).map(|i| i.interest).sum();
    v.interest_remaining = inst.iter().filter(|i| !i.paid).map(|i| i.interest).sum();
    v.charges_paid = inst
        .iter()
        .filter(|i| i.paid)
        .map(|i| i.insurance + i.fees)
        .sum();
    let contract_date = parse_date(&v.contract_date)?;
    let released = v.principal - v.upfront.total();
    (v.realized_cet, v.realized_drops) =
        cet::realized(released, v.principal, contract_date, &inst)?;
    v.cet = cet::contract(conn, id, v.indexer, v.upfront, v.cet_annual.as_deref())?;
    if let Some(cet) = v.realized_cet {
        let from = parse_date(&v.contract_date)?;
        let days = (today_local() - from).num_days().max(1);
        if let Some(f) = crate::indices::deflator_factor(conn, Deflator::Ipca, from, today_local())?
        {
            let infl_annual = f.factor.to_f64().unwrap_or(1.0).powf(365.0 / days as f64) - 1.0;
            v.real_cost = Some(crate::finance::returns::real_return(cet, infl_annual));
        }
    }
    Ok(v)
}

pub fn list(conn: &Connection, include_archived: bool) -> Result<Vec<LoanView>> {
    let mut stmt =
        conn.prepare("SELECT id FROM loan WHERE (?1 OR archived = 0) ORDER BY contract_date")?;
    let ids: Vec<String> = stmt
        .query_map([include_archived], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    ids.iter().map(|id| view(conn, id)).collect()
}

pub fn simulate_prepayment(conn: &Connection, a: &PrepayArgs) -> Result<PrepaySimulation> {
    if a.amount <= 0 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-amortizacao-positiva",
        )));
    }
    let date = parse_date(&a.date)?;
    let base = params_for(conn, &a.loan_id, None)?;
    let before = schedule::build(&base);
    let mut after_n = 0u32;
    while crate::time::add_months(base.first_due, after_n as i32) <= date {
        after_n += 1;
    }
    let mode = match a.mode {
        PrepayMode::ReduceTerm => schedule::ExtraMode::ReduceTerm,
        PrepayMode::ReduceInstallment => schedule::ExtraMode::ReduceInstallment,
    };
    let after = schedule::build(&params_for(
        conn,
        &a.loan_id,
        Some(schedule::Extra {
            after_number: after_n,
            amount: a.amount,
            mode,
        }),
    )?);
    let interest = |rows: &[schedule::Row]| {
        rows.iter()
            .filter(|r| r.number > after_n)
            .map(|r| r.interest)
            .sum::<i64>()
    };
    let next = |rows: &[schedule::Row]| {
        rows.iter()
            .find(|r| r.number == after_n + 1)
            .map(|r| r.total)
            .unwrap_or(0)
    };
    let (ib, ia) = (interest(&before), interest(&after));
    Ok(PrepaySimulation {
        mode: a.mode,
        amount: a.amount,
        interest_before: ib,
        interest_after: ia,
        interest_saved: ib - ia,
        months_before: before.len() as u32,
        months_after: after.len() as u32,
        next_payment_before: next(&before),
        next_payment_after: next(&after),
        schedule: after
            .iter()
            .map(|r| Installment {
                id: String::new(),
                number: r.number,
                version: 1,
                version_number: r.number,
                due_date: fmt_date(r.due_date),
                amortization: r.amortization,
                interest: r.interest,
                adjustment: r.adjustment,
                insurance: r.insurance,
                insurance_mip: r.mip,
                insurance_dfi: r.dfi,
                fees: r.fees,
                total: r.total,
                balance_after: r.balance_after,
                extra_payment: r.extra_payment,
                estimated: r.estimated,
                paid: false,
                paid_date: None,
                paid_amount: None,
                txn_id: None,
                txn_date: None,
                txn_description: None,
                txn_account_id: None,
                txn_account_name: None,
                overdue: false,
                index_factor: r.index_factor.map(|f| f.normalize().to_string()),
                business_days: r.business_days,
            })
            .collect(),
    })
}

pub fn simulate_portability(
    conn: &Connection,
    a: &PortabilityArgs,
) -> Result<PortabilitySimulation> {
    let today = fmt_date(today_local());
    let inst = installments(conn, &a.loan_id)?;
    let remaining: Vec<&Installment> = inst
        .iter()
        .filter(|i| !i.paid && i.due_date >= today)
        .collect();
    let balance = balance_at(conn, &a.loan_id, &today)?;
    let current_total: i64 = remaining.iter().map(|i| i.total).sum();
    let l = load_row(conn, &a.loan_id)?;
    let term = a.term_months.unwrap_or(remaining.len() as u32).max(1);
    let system = sched_system(a.system.unwrap_or(l.system));
    let first_due = remaining
        .first()
        .map(|i| parse_date(&i.due_date))
        .transpose()?
        .unwrap_or(today_local());
    let rows = schedule::build(&schedule::Params {
        principal: balance,
        annual_rate_pct: dec(&a.annual_rate, "nucleo-planejamento-divida-campo-taxa")?,
        system,
        term,
        grace: 0,
        first_due,
        insurance: l.insurance,
        fees: l.fees,
        mip_rate: l.mip_rate,
        mip_fixed: l.mip_fixed,
        dfi: l.dfi,
        index_factors: vec![],
        periods: vec![],
        extras: vec![],
    });
    let new_total: i64 = rows.iter().map(|r| r.total).sum();
    let costs = a.costs.unwrap_or(0);
    Ok(PortabilitySimulation {
        balance,
        current_remaining_total: current_total,
        new_total,
        costs,
        savings: current_total - new_total - costs,
        current_next_payment: remaining.first().map(|i| i.total).unwrap_or(0),
        new_first_payment: rows.first().map(|r| r.total).unwrap_or(0),
        new_term_months: rows.len() as u32,
    })
}

/// Grava valor do imóvel e taxas do MIP e do DFI (o chamador regenera o cronograma).
/// Seguros do financiamento imobiliário. Cada seguro vai em percentual ou em valor fixo
/// por mês (o valor fixo, quando informado, vale no lugar do percentual).
pub struct HousingInput<'a> {
    pub property_value: Option<i64>,
    pub mip_rate: Option<&'a str>,
    pub dfi_rate: Option<&'a str>,
    pub mip_amount: Option<i64>,
    pub dfi_amount: Option<i64>,
}

pub fn set_housing_insurance(tx: &Transaction<'_>, id: &str, h: &HousingInput<'_>) -> Result<()> {
    if h.property_value.is_some_and(|v| v < 0) {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-imovel-negativo",
        )));
    }
    if h.mip_amount.is_some_and(|v| v < 0) || h.dfi_amount.is_some_and(|v| v < 0) {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-seguro-negativo",
        )));
    }
    let mip_amount = h.mip_amount.filter(|v| *v > 0);
    let dfi_amount = h.dfi_amount.filter(|v| *v > 0);
    let mip = if mip_amount.is_some() {
        None
    } else {
        clean_insurance_rate(h.mip_rate, "nucleo-planejamento-divida-campo-taxa-mip")?
    };
    let dfi = if dfi_amount.is_some() {
        None
    } else {
        clean_insurance_rate(h.dfi_rate, "nucleo-planejamento-divida-campo-taxa-dfi")?
    };
    if dfi.is_some() && h.property_value.unwrap_or(0) == 0 {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-dfi-sem-imovel",
        )));
    }
    tx.execute(
        "UPDATE loan SET property_value = ?2, mip_rate = ?3, dfi_rate = ?4, mip_amount = ?5, dfi_amount = ?6, updated_at = ?7
          WHERE id = ?1 AND (property_value IS NOT ?2 OR mip_rate IS NOT ?3 OR dfi_rate IS NOT ?4
                             OR mip_amount IS NOT ?5 OR dfi_amount IS NOT ?6)",
        params![id, h.property_value, mip, dfi, mip_amount, dfi_amount, now_utc()],
    )?;
    Ok(())
}

/// Stores the costs paid on day zero (zero or empty clears each one). Together they must stay
/// below the principal, since they come out of the amount released.
fn set_upfront_costs(tx: &Transaction<'_>, id: &str, i: &LoanInput) -> Result<()> {
    let costs = [i.upfront_iof, i.upfront_fees, i.upfront_insurance];
    if costs.iter().flatten().any(|v| *v < 0) {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-custo-inicial-negativo",
        )));
    }
    if costs.iter().flatten().sum::<i64>() >= i.principal {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-custo-inicial-maior",
        )));
    }
    let [iof, fees, insurance] = costs.map(|v| v.filter(|v| *v > 0));
    tx.execute(
        "UPDATE loan SET upfront_iof = ?2, upfront_fees = ?3, upfront_insurance = ?4, updated_at = ?5
          WHERE id = ?1 AND (upfront_iof IS NOT ?2 OR upfront_fees IS NOT ?3 OR upfront_insurance IS NOT ?4)",
        params![id, iof, fees, insurance, now_utc()],
    )?;
    Ok(())
}

/// Liga o contrato a uma categoria de despesa (uma categoria serve a um contrato só).
pub fn set_category(tx: &Transaction<'_>, id: &str, category: Option<&str>) -> Result<()> {
    if let Some(c) = category {
        let kind: Option<String> = tx
            .query_row("SELECT kind FROM category WHERE id = ?1", [c], |r| r.get(0))
            .optional()?;
        match kind.as_deref() {
            None => return Err(Error::not_found(t("nucleo-planejamento-divida-categoria"))),
            Some("expense") => {}
            Some(_) => {
                return Err(Error::invalid(t(
                    "nucleo-planejamento-divida-erro-categoria-despesa",
                )));
            }
        }
        let other: Option<String> = tx
            .query_row(
                "SELECT name FROM loan WHERE category_id = ?1 AND id <> ?2 AND archived = 0",
                params![c, id],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(n) = other {
            return Err(Error::conflict(t_args(
                "nucleo-planejamento-divida-erro-categoria-ligada",
                &args(&[("contrato", n.into())]),
            )));
        }
    }
    tx.execute(
        "UPDATE loan SET category_id = ?2, updated_at = ?3 WHERE id = ?1 AND category_id IS NOT ?2",
        params![id, category, now_utc()],
    )?;
    Ok(())
}

/// Lançamentos de saída numa categoria ligada a um contrato marcam como paga a parcela em
/// aberto de vencimento mais próximo (até 20 dias de distância). Chamado ao importar,
/// criar ou categorizar lançamentos. Devolve quantas parcelas foram ligadas.
pub fn link_category_payments(tx: &Transaction<'_>, txn_ids: &[Id]) -> Result<u32> {
    let mut n = 0;
    for tid in txn_ids {
        let row: Option<(String, String, i64)> = tx
            .query_row(
                "SELECT l.id, t.date, t.amount FROM txn t
                   JOIN loan l ON l.category_id = t.category_id AND l.archived = 0
                  WHERE t.id = ?1 AND t.amount < 0 AND t.status IN ('cleared', 'reconciled')
                    AND NOT EXISTS (SELECT 1 FROM loan_installment li WHERE li.txn_id = t.id)",
                [tid],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((loan, date, amount)) = row else {
            continue;
        };
        let inst: Option<String> = tx
            .query_row(
                "SELECT id FROM loan_installment
                  WHERE loan_id = ?1 AND status = 'planned'
                    AND abs(julianday(due_date) - julianday(?2)) <= 20
                  ORDER BY abs(julianday(due_date) - julianday(?2)), number LIMIT 1",
                params![loan, date],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(iid) = inst {
            tx.execute(
                "UPDATE loan_installment SET status = 'paid', paid_date = ?2, paid_amount = ?3, txn_id = ?4, updated_at = ?5 WHERE id = ?1",
                params![iid, date, -amount, tid, now_utc()],
            )?;
            n += 1;
        }
    }
    Ok(n)
}

/// Liga parcelas vencidas aos lançamentos de pagamento na conta de débito (valor igual com
/// tolerância de 1% e data até 7 dias de distância).
pub fn reconcile(tx: &Transaction<'_>, id: &str) -> Result<u32> {
    let acc: Option<String> = tx.query_row(
        "SELECT debit_account_id FROM loan WHERE id = ?1",
        [id],
        |r| r.get(0),
    )?;
    // Primeiro, os lançamentos já na categoria do contrato (qualquer conta e valor).
    let in_category: Vec<Id> = {
        let mut stmt = tx.prepare(
            "SELECT t.id FROM txn t JOIN loan l ON l.category_id = t.category_id
              WHERE l.id = ?1 AND t.amount < 0 ORDER BY t.date",
        )?;
        stmt.query_map([id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let by_category = link_category_payments(tx, &in_category)?;
    let Some(acc) = acc else {
        if by_category > 0 {
            return Ok(by_category);
        }
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-erro-conciliar-sem-conta",
        )));
    };
    let horizon = fmt_date(today_local() + chrono::Duration::days(7));
    let mut stmt = tx.prepare(
        "SELECT id, due_date, total FROM loan_installment WHERE loan_id = ?1 AND status = 'planned' AND due_date <= ?2 ORDER BY number",
    )?;
    let pending: Vec<(String, String, i64)> = stmt
        .query_map(params![id, horizon], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let category: Option<String> =
        tx.query_row("SELECT category_id FROM loan WHERE id = ?1", [id], |r| {
            r.get(0)
        })?;
    let mut n = by_category;
    for (iid, due, total) in pending {
        let tol = (total / 100).max(100);
        let found: Option<(String, String, i64)> = tx
            .query_row(
                "SELECT t.id, t.date, t.amount FROM txn t
                 WHERE t.account_id = ?1 AND t.status IN ('cleared', 'reconciled') AND t.amount < 0
                   AND abs(-t.amount - ?2) <= ?3 AND abs(julianday(t.date) - julianday(?4)) <= 7
                   AND NOT EXISTS (SELECT 1 FROM loan_installment li WHERE li.txn_id = t.id)
                 ORDER BY abs(julianday(t.date) - julianday(?4)) LIMIT 1",
                params![acc, total, tol, due],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        if let Some((tid, date, amount)) = found {
            tx.execute(
                "UPDATE loan_installment SET status = 'paid', paid_date = ?2, paid_amount = ?3, txn_id = ?4, updated_at = ?5 WHERE id = ?1",
                params![iid, date, -amount, tid, now_utc()],
            )?;
            // Pagamento achado pelo valor e sem categoria: recebe a do contrato.
            if let Some(c) = &category {
                tx.execute(
                    "UPDATE txn SET category_id = ?2, updated_at = ?3 WHERE id = ?1 AND category_id IS NULL",
                    params![tid, c, now_utc()],
                )?;
            }
            n += 1;
        }
    }
    Ok(n)
}
