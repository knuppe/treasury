//! IPTU dos imóveis e terrenos (Brasil; docs/brazil/iptu.md, V126): o carnê de cada ano com o
//! cadastro na prefeitura, o valor venal (terreno e construção), os impostos territorial e
//! predial, as taxas (iluminação pública, lixo), os descontos (cota única, bom pagador), o total
//! devido e as parcelas (0 = cota única), com o pagamento ligado ao lançamento.
//!
//! O que se tira disso, sem rede nem IA:
//! - alíquota efetiva (impostos ÷ valor venal);
//! - variação do valor venal e do imposto contra o ano anterior e contra o IPCA do ano;
//! - valor venal × valor de mercado do bem (avaliação ou estimativa do app);
//! - cota única × parcelas: o desconto à vista como uma taxa ao mês, contra o CDI;
//! - o lançamento provável de cada parcela em aberto (texto "IPTU", valor e data perto);
//! - o lembrete do vencimento (`notify`) e o IPTU nos impostos pagos (`tax_burden`).

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n;
use crate::ids::{Id, new_id};
use crate::import::text::fold;
use crate::time::{fmt_date, now_utc, parse_date};

/// Como o IPTU do ano vai ser pago.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "IptuPlan")]
pub enum Plan {
    /// Cota única (parcela 0), em geral com desconto.
    Single,
    /// Parcelado (parcelas 1, 2, …).
    Installments,
}

impl Plan {
    fn as_str(self) -> &'static str {
        match self {
            Plan::Single => "single",
            Plan::Installments => "installments",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "single" => Some(Plan::Single),
            "installments" => Some(Plan::Installments),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuInstallmentInput")]
pub struct InstallmentInput {
    /// 0 = cota única; 1, 2, … = parcelas.
    pub number: u32,
    pub due_date: String,
    #[ts(type = "number")]
    pub amount: i64,
}

/// O carnê de um ano (grava por cima do mesmo imóvel e ano).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuInput")]
pub struct IptuInput {
    pub asset_id: Id,
    pub year: i32,
    #[serde(default)]
    #[ts(optional)]
    pub registration: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub city: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub land_area: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub built_area: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub land_value: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub building_value: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub land_tax: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub building_tax: Option<i64>,
    /// Total de impostos; sem ele, a soma do territorial e do predial.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub taxes_total: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub lighting_fee: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub waste_fee: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub other_fees: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub discount: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub good_payer_discount: Option<i64>,
    #[ts(type = "number")]
    pub total_due: i64,
    #[serde(default)]
    #[ts(optional)]
    pub plan: Option<Plan>,
    #[serde(default)]
    pub installments: Vec<InstallmentInput>,
    #[serde(default)]
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "IptuInstallmentStatus")]
pub enum InstallmentStatus {
    Paid,
    Open,
    /// Venceu sem pagamento.
    Late,
    /// Da outra forma de pagamento (cota única com o parcelado escolhido, ou o contrário).
    NotChosen,
}

/// Lançamento que parece ser o pagamento de uma parcela.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuTxnMatch")]
pub struct TxnMatch {
    pub txn_id: Id,
    pub date: String,
    pub description: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// O texto ou a categoria falam em IPTU.
    pub named: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuInstallment")]
pub struct Installment {
    pub id: Id,
    pub number: u32,
    pub due_date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub paid_on: Option<String>,
    pub txn_id: Option<Id>,
    pub status: InstallmentStatus,
    /// Em aberto: o lançamento provável do pagamento.
    pub suggestion: Option<TxnMatch>,
}

/// Comparação com o ano anterior.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuChange")]
pub struct Change {
    pub year: i32,
    /// Variação do valor venal total (fração: 0,1 = 10%).
    pub venal_change: Option<f64>,
    /// Variação do total de impostos.
    pub tax_change: Option<f64>,
    /// IPCA dos 12 meses do ano anterior ao carnê (a correção usual da planta de valores).
    pub ipca: Option<f64>,
}

/// Valor venal × valor de mercado do bem no app.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuMarket")]
pub struct Market {
    #[ts(type = "number")]
    pub value: i64,
    pub as_of: String,
    /// O valor é estimado (índice, depreciação) e não uma avaliação.
    pub estimated: bool,
    /// Valor venal ÷ valor de mercado.
    pub ratio: f64,
}

/// Cota única × parcelado.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuSingleVsInstallments")]
pub struct SingleVsInstallments {
    #[ts(type = "number")]
    pub single: i64,
    #[ts(type = "number")]
    pub installments: i64,
    #[ts(type = "number")]
    pub count: u32,
    /// Quanto a cota única economiza (negativo: o parcelado sai mais barato).
    #[ts(type = "number")]
    pub saving: i64,
    /// O desconto à vista como taxa ao mês (o que o dinheiro "rende" pagando à vista).
    pub monthly_rate: Option<f64>,
    /// CDI ao mês pela última taxa conhecida.
    pub cdi_monthly: Option<f64>,
    pub best: Plan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "IptuInsightKind")]
pub enum InsightKind {
    PayUpfront,
    PayInInstallments,
    AboveInflation,
    VenalAboveMarket,
    VenalFarBelowMarket,
    TotalMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuInsight")]
pub struct Insight {
    pub kind: InsightKind,
    /// Atenção (ex.: valor venal acima do mercado) ou informação.
    pub warning: bool,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuView")]
pub struct IptuView {
    pub id: Id,
    pub asset_id: Id,
    pub year: i32,
    pub registration: Option<String>,
    pub city: Option<String>,
    pub land_area: Option<String>,
    pub built_area: Option<String>,
    #[ts(type = "number | null")]
    pub land_value: Option<i64>,
    #[ts(type = "number | null")]
    pub building_value: Option<i64>,
    /// Valor venal total (terreno + predial).
    #[ts(type = "number | null")]
    pub venal_total: Option<i64>,
    #[ts(type = "number | null")]
    pub land_tax: Option<i64>,
    #[ts(type = "number | null")]
    pub building_tax: Option<i64>,
    #[ts(type = "number | null")]
    pub taxes_total: Option<i64>,
    #[ts(type = "number")]
    pub lighting_fee: i64,
    #[ts(type = "number")]
    pub waste_fee: i64,
    #[ts(type = "number")]
    pub other_fees: i64,
    #[ts(type = "number")]
    pub discount: i64,
    #[ts(type = "number")]
    pub good_payer_discount: i64,
    #[ts(type = "number")]
    pub total_due: i64,
    pub currency: String,
    /// A forma escolhida (ou deduzida do que já foi pago).
    pub plan: Option<Plan>,
    pub plan_chosen: bool,
    pub notes: Option<String>,
    pub installments: Vec<Installment>,
    /// Impostos ÷ valor venal.
    pub effective_rate: Option<f64>,
    #[ts(type = "number")]
    pub paid: i64,
    /// O que falta pagar na forma escolhida (sem forma: o menor dos dois).
    #[ts(type = "number")]
    pub open: i64,
    pub previous: Option<Change>,
    pub market: Option<Market>,
    pub single_vs_installments: Option<SingleVsInstallments>,
    pub insights: Vec<Insight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IptuPayArgs")]
pub struct PayArgs {
    pub installment_id: Id,
    /// O lançamento do pagamento (a data dele vira a do pagamento).
    #[serde(default)]
    #[ts(optional)]
    pub txn_id: Option<Id>,
    /// Pago sem lançamento no app: a data.
    #[serde(default)]
    #[ts(optional)]
    pub paid_on: Option<String>,
}

/// Imóveis e terrenos têm IPTU.
pub fn has_iptu(kind: &str) -> bool {
    matches!(kind, "real_estate" | "land")
}

fn clean(s: &Option<String>) -> Option<String> {
    s.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn check_area(s: &Option<String>) -> Result<Option<String>> {
    let Some(v) = clean(s) else { return Ok(None) };
    // Com vírgula, é o formato brasileiro ("1.250,5"); sem vírgula, o ponto é o decimal
    // ("72.5", como o agente e o inglês escrevem), salvo se ele só agrupa milhares ("1.250").
    let normalized = if v.contains(',') {
        v.replace('.', "").replace(',', ".")
    } else if v.matches('.').count() > 1 || v.rsplit_once('.').is_some_and(|(_, d)| d.len() == 3) {
        v.replace('.', "")
    } else {
        v.clone()
    };
    let n: rust_decimal::Decimal = normalized
        .parse()
        .map_err(|_| Error::invalid(i18n::t("iptu-erro-area")))?;
    if n.is_sign_negative() {
        return Err(Error::invalid(i18n::t("iptu-erro-area")));
    }
    Ok(Some(n.normalize().to_string()))
}

fn non_negative(v: Option<i64>) -> Result<Option<i64>> {
    match v {
        Some(n) if n < 0 => Err(Error::invalid(i18n::t("iptu-erro-negativo"))),
        _ => Ok(v),
    }
}

/// Grava o carnê do ano (cria ou substitui o do mesmo imóvel e ano). As parcelas pagas que
/// continuam (mesmo número) guardam o pagamento.
pub fn save(tx: &Transaction<'_>, a: &IptuInput) -> Result<Id> {
    let kind: String = tx
        .query_row(
            "SELECT kind FROM physical_asset WHERE id = ?1",
            [&a.asset_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(format!("{} {}", i18n::t("iptu-imovel"), a.asset_id)))?;
    if !has_iptu(&kind) {
        return Err(Error::invalid(i18n::t("iptu-erro-tipo")));
    }
    if !(1990..=2100).contains(&a.year) {
        return Err(Error::invalid(i18n::t("iptu-erro-ano")));
    }
    if a.total_due < 0 {
        return Err(Error::invalid(i18n::t("iptu-erro-negativo")));
    }
    let vals = [
        a.land_value,
        a.building_value,
        a.land_tax,
        a.building_tax,
        a.taxes_total,
        a.lighting_fee,
        a.waste_fee,
        a.other_fees,
        a.discount,
        a.good_payer_discount,
    ];
    for v in vals {
        non_negative(v)?;
    }
    let taxes_total = a.taxes_total.or(match (a.land_tax, a.building_tax) {
        (None, None) => None,
        (l, b) => Some(l.unwrap_or(0) + b.unwrap_or(0)),
    });
    let mut numbers = std::collections::BTreeSet::new();
    for i in &a.installments {
        if i.number > 24 {
            return Err(Error::invalid(i18n::t("iptu-erro-parcela")));
        }
        if i.amount <= 0 {
            return Err(Error::invalid(i18n::t("iptu-erro-parcela-valor")));
        }
        parse_date(&i.due_date)?;
        if !numbers.insert(i.number) {
            return Err(Error::invalid(i18n::t("iptu-erro-parcela-repetida")));
        }
    }
    let currency: String = tx.query_row(
        "SELECT currency FROM physical_asset WHERE id = ?1",
        [&a.asset_id],
        |r| r.get(0),
    )?;
    let now = now_utc();
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM property_tax_bill WHERE asset_id = ?1 AND year = ?2",
            params![a.asset_id, a.year],
            |r| r.get(0),
        )
        .optional()?;
    let id = existing.clone().unwrap_or_else(new_id);
    let land_area = check_area(&a.land_area)?;
    let built_area = check_area(&a.built_area)?;
    let plan = a.plan.map(Plan::as_str);
    if existing.is_some() {
        tx.execute(
            "UPDATE property_tax_bill SET registration = ?2, city = ?3, land_area = ?4, built_area = ?5,
                land_value = ?6, building_value = ?7, land_tax = ?8, building_tax = ?9, taxes_total = ?10,
                lighting_fee = ?11, waste_fee = ?12, other_fees = ?13, discount = ?14,
                good_payer_discount = ?15, total_due = ?16, plan = ?17, notes = ?18, updated_at = ?19
              WHERE id = ?1",
            params![
                id,
                clean(&a.registration),
                clean(&a.city),
                land_area,
                built_area,
                a.land_value,
                a.building_value,
                a.land_tax,
                a.building_tax,
                taxes_total,
                a.lighting_fee.unwrap_or(0),
                a.waste_fee.unwrap_or(0),
                a.other_fees.unwrap_or(0),
                a.discount.unwrap_or(0),
                a.good_payer_discount.unwrap_or(0),
                a.total_due,
                plan,
                clean(&a.notes),
                now
            ],
        )?;
    } else {
        tx.execute(
            "INSERT INTO property_tax_bill (id, asset_id, year, registration, city, land_area, built_area,
                land_value, building_value, land_tax, building_tax, taxes_total, lighting_fee, waste_fee,
                other_fees, discount, good_payer_discount, total_due, currency, plan, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?22)",
            params![
                id,
                a.asset_id,
                a.year,
                clean(&a.registration),
                clean(&a.city),
                land_area,
                built_area,
                a.land_value,
                a.building_value,
                a.land_tax,
                a.building_tax,
                taxes_total,
                a.lighting_fee.unwrap_or(0),
                a.waste_fee.unwrap_or(0),
                a.other_fees.unwrap_or(0),
                a.discount.unwrap_or(0),
                a.good_payer_discount.unwrap_or(0),
                a.total_due,
                currency,
                plan,
                clean(&a.notes),
                now
            ],
        )?;
    }
    // Parcelas: as que somem saem; as que ficam guardam o pagamento.
    let keep: Vec<i64> = a.installments.iter().map(|i| i.number as i64).collect();
    let mut stmt =
        tx.prepare("SELECT id, number FROM property_tax_installment WHERE bill_id = ?1")?;
    let old: Vec<(String, i64)> = stmt
        .query_map([&id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    for (iid, n) in &old {
        if !keep.contains(n) {
            tx.execute("DELETE FROM property_tax_installment WHERE id = ?1", [iid])?;
        }
    }
    for i in &a.installments {
        match old.iter().find(|(_, n)| *n == i.number as i64) {
            Some((iid, _)) => {
                tx.execute(
                    "UPDATE property_tax_installment SET due_date = ?2, amount = ?3 WHERE id = ?1",
                    params![iid, i.due_date, i.amount],
                )?;
            }
            None => {
                tx.execute(
                    "INSERT INTO property_tax_installment (id, bill_id, number, due_date, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![new_id(), id, i.number, i.due_date, i.amount],
                )?;
            }
        }
    }
    Ok(id)
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM property_tax_bill WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(format!("{} {id}", i18n::t("iptu-carne"))));
    }
    Ok(())
}

/// Marca a parcela como paga (pelo lançamento ou por uma data). Pagar a cota única escolhe a
/// cota única; pagar uma parcela escolhe o parcelado.
pub fn pay(tx: &Transaction<'_>, a: &PayArgs) -> Result<()> {
    let (bill_id, number): (String, i64) = tx
        .query_row(
            "SELECT bill_id, number FROM property_tax_installment WHERE id = ?1",
            [&a.installment_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| {
            Error::not_found(format!("{} {}", i18n::t("iptu-parcela"), a.installment_id))
        })?;
    let paid_on = match (&a.txn_id, &a.paid_on) {
        (Some(t), _) => {
            let used: Option<i64> = tx
                .query_row(
                    "SELECT number FROM property_tax_installment WHERE txn_id = ?1 AND id <> ?2",
                    params![t, a.installment_id],
                    |r| r.get(0),
                )
                .optional()?;
            if used.is_some() {
                return Err(Error::conflict(i18n::t("iptu-erro-lancamento-usado")));
            }
            let row = crate::transactions::load_row(tx, t)?;
            if row.amount >= 0 {
                return Err(Error::invalid(i18n::t("iptu-erro-lancamento-saida")));
            }
            row.date
        }
        (None, Some(d)) => fmt_date(parse_date(d)?),
        (None, None) => return Err(Error::invalid(i18n::t("iptu-erro-pagamento"))),
    };
    tx.execute(
        "UPDATE property_tax_installment SET paid_on = ?2, txn_id = ?3 WHERE id = ?1",
        params![a.installment_id, paid_on, a.txn_id],
    )?;
    let plan = if number == 0 {
        Plan::Single
    } else {
        Plan::Installments
    };
    tx.execute(
        "UPDATE property_tax_bill SET plan = ?2, updated_at = ?3 WHERE id = ?1",
        params![bill_id, plan.as_str(), now_utc()],
    )?;
    Ok(())
}

pub fn unpay(tx: &Transaction<'_>, installment_id: &str) -> Result<()> {
    if tx.execute(
        "UPDATE property_tax_installment SET paid_on = NULL, txn_id = NULL WHERE id = ?1",
        [installment_id],
    )? == 0
    {
        return Err(Error::not_found(format!(
            "{} {installment_id}",
            i18n::t("iptu-parcela")
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Leitura

struct BillRow {
    id: Id,
    asset_id: Id,
    year: i32,
    registration: Option<String>,
    city: Option<String>,
    land_area: Option<String>,
    built_area: Option<String>,
    land_value: Option<i64>,
    building_value: Option<i64>,
    land_tax: Option<i64>,
    building_tax: Option<i64>,
    taxes_total: Option<i64>,
    lighting_fee: i64,
    waste_fee: i64,
    other_fees: i64,
    discount: i64,
    good_payer_discount: i64,
    total_due: i64,
    currency: String,
    plan: Option<Plan>,
    notes: Option<String>,
}

const BILL_COLS: &str = "id, asset_id, year, registration, city, land_area, built_area, land_value,
    building_value, land_tax, building_tax, taxes_total, lighting_fee, waste_fee, other_fees, discount,
    good_payer_discount, total_due, currency, plan, notes";

fn bill_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<BillRow> {
    Ok(BillRow {
        id: r.get(0)?,
        asset_id: r.get(1)?,
        year: r.get(2)?,
        registration: r.get(3)?,
        city: r.get(4)?,
        land_area: r.get(5)?,
        built_area: r.get(6)?,
        land_value: r.get(7)?,
        building_value: r.get(8)?,
        land_tax: r.get(9)?,
        building_tax: r.get(10)?,
        taxes_total: r.get(11)?,
        lighting_fee: r.get(12)?,
        waste_fee: r.get(13)?,
        other_fees: r.get(14)?,
        discount: r.get(15)?,
        good_payer_discount: r.get(16)?,
        total_due: r.get(17)?,
        currency: r.get(18)?,
        plan: r
            .get::<_, Option<String>>(19)?
            .as_deref()
            .and_then(Plan::parse),
        notes: r.get(20)?,
    })
}

struct InstRow {
    id: Id,
    number: u32,
    due_date: String,
    amount: i64,
    paid_on: Option<String>,
    txn_id: Option<Id>,
}

fn installments_of(conn: &Connection, bill_id: &str) -> Result<Vec<InstRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, number, due_date, amount, paid_on, txn_id FROM property_tax_installment
          WHERE bill_id = ?1 ORDER BY number",
    )?;
    Ok(stmt
        .query_map([bill_id], |r| {
            Ok(InstRow {
                id: r.get(0)?,
                number: r.get(1)?,
                due_date: r.get(2)?,
                amount: r.get(3)?,
                paid_on: r.get(4)?,
                txn_id: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

/// A forma de pagamento: a escolhida ou a que já começou a ser paga.
fn effective_plan(plan: Option<Plan>, inst: &[InstRow]) -> Option<Plan> {
    plan.or_else(|| {
        if inst.iter().any(|i| i.number > 0 && i.paid_on.is_some()) {
            Some(Plan::Installments)
        } else if inst.iter().any(|i| i.number == 0 && i.paid_on.is_some()) {
            Some(Plan::Single)
        } else {
            None
        }
    })
}

fn belongs(plan: Option<Plan>, number: u32) -> bool {
    match plan {
        Some(Plan::Single) => number == 0,
        Some(Plan::Installments) => number > 0,
        None => true,
    }
}

/// Lançamentos que parecem pagar a parcela: saída de valor parecido (±2%) entre 25 dias antes
/// e 15 depois do vencimento, ainda sem parcela de IPTU; primeiro os que falam em IPTU.
pub fn matches(conn: &Connection, due_date: &str, amount: i64) -> Result<Vec<TxnMatch>> {
    let due = parse_date(due_date)?;
    let from = fmt_date(due - chrono::Duration::days(25));
    let to = fmt_date(due + chrono::Duration::days(15));
    let tol = (amount / 50).max(100);
    let mut stmt = conn.prepare_cached(
        "SELECT t.id, t.date, t.description, t.amount, COALESCE(c.name, '')
           FROM txn t LEFT JOIN category c ON c.id = t.category_id
          WHERE t.date BETWEEN ?1 AND ?2 AND t.amount BETWEEN ?3 AND ?4
            AND t.status IN ('cleared', 'reconciled')
            AND t.id NOT IN (SELECT txn_id FROM property_tax_installment WHERE txn_id IS NOT NULL)
          ORDER BY abs(t.amount + ?5), abs(julianday(t.date) - julianday(?6))",
    )?;
    let rows = stmt.query_map(
        params![from, to, -amount - tol, -amount + tol, amount, due_date],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
            ))
        },
    )?;
    let mut out = vec![];
    for r in rows {
        let (txn_id, date, description, amount, cat) = r?;
        let named = fold(&description).contains("iptu") || fold(&cat).contains("iptu");
        out.push(TxnMatch {
            txn_id,
            date,
            description,
            amount,
            named,
        });
    }
    out.sort_by_key(|m| !m.named);
    Ok(out)
}

/// IPCA acumulado nos 12 meses de um ano (só com os meses publicados).
fn ipca_of_year(conn: &Connection, year: i32) -> Result<Option<f64>> {
    let (Some(from), Some(to)) = (
        NaiveDate::from_ymd_opt(year, 1, 1),
        NaiveDate::from_ymd_opt(year, 12, 31),
    ) else {
        return Ok(None);
    };
    let rates = crate::indices::monthly_rates(conn, "ipca", from, to)?;
    if rates.iter().filter(|r| !r.estimated).count() < 12 {
        return Ok(None);
    }
    Ok(rates
        .iter()
        .try_fold(1.0f64, |acc, r| Some(acc * (1.0 + r.pct.to_f64()? / 100.0)))
        .map(|f| f - 1.0))
}

/// CDI ao mês pela última taxa diária conhecida até `date` (série `cdi`, % a.d.).
fn cdi_monthly(conn: &Connection, date: &str) -> Result<Option<f64>> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM index_value WHERE series_id = 'cdi' AND date <= ?1
              ORDER BY date DESC LIMIT 1",
            [date],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v.and_then(|s| s.parse::<f64>().ok())
        .map(|d| (1.0 + d / 100.0).powf(252.0 / 12.0) - 1.0))
}

/// Taxa ao mês que iguala a cota única ao valor presente das parcelas (sem solução: `None`).
fn implied_monthly_rate(
    single: i64,
    single_due: NaiveDate,
    parts: &[(NaiveDate, i64)],
) -> Option<f64> {
    let pv = |r: f64| -> f64 {
        parts
            .iter()
            .map(|(d, a)| {
                let months = ((*d - single_due).num_days().max(0) as f64) / 30.4375;
                *a as f64 / (1.0 + r).powf(months)
            })
            .sum()
    };
    let target = single as f64;
    if pv(0.0) <= target {
        return None;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    if pv(hi) > target {
        return None;
    }
    for _ in 0..80 {
        let mid = (lo + hi) / 2.0;
        if pv(mid) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

fn pct(f: f64) -> String {
    let l = i18n::current();
    let v = (f * 1000.0).round() / 10.0;
    let s = format!("{v:.1}");
    match l {
        i18n::Locale::En => s,
        _ => s.replace('.', ","),
    }
}

fn venal_total(b: &BillRow) -> Option<i64> {
    match (b.land_value, b.building_value) {
        (None, None) => None,
        (l, v) => Some(l.unwrap_or(0) + v.unwrap_or(0)),
    }
}

/// O lançamento provável de cada parcela em aberto, sem repetir o mesmo lançamento em duas
/// parcelas (as janelas de parcelas mensais se sobrepõem): primeiro os que falam em IPTU,
/// depois o vencimento mais perto da data do lançamento.
fn assign_suggestions(
    conn: &Connection,
    open: &[&InstRow],
) -> Result<std::collections::HashMap<Id, TxnMatch>> {
    let mut pairs: Vec<(bool, i64, usize, TxnMatch)> = vec![];
    for (k, i) in open.iter().enumerate() {
        let due = parse_date(&i.due_date)?;
        for m in matches(conn, &i.due_date, i.amount)? {
            let gap = parse_date(&m.date).map_or(i64::MAX, |d| (d - due).num_days().abs());
            pairs.push((!m.named, gap, k, m));
        }
    }
    pairs.sort_by_key(|p| (p.0, p.1, p.2));
    let mut out = std::collections::HashMap::new();
    let mut used = std::collections::HashSet::new();
    for (_, _, k, m) in pairs {
        let id = &open[k].id;
        if out.contains_key(id) || used.contains(&m.txn_id) {
            continue;
        }
        used.insert(m.txn_id.clone());
        out.insert(id.clone(), m);
    }
    Ok(out)
}

fn view_of(conn: &Connection, b: BillRow, today: NaiveDate) -> Result<IptuView> {
    let rows = installments_of(conn, &b.id)?;
    let plan = effective_plan(b.plan, &rows);
    let open: Vec<&InstRow> = rows
        .iter()
        .filter(|i| i.paid_on.is_none() && belongs(plan, i.number))
        .collect();
    let mut suggestions = assign_suggestions(conn, &open)?;
    let today_s = fmt_date(today);
    let mut installments = vec![];
    for i in &rows {
        let status = if i.paid_on.is_some() {
            InstallmentStatus::Paid
        } else if !belongs(plan, i.number) {
            InstallmentStatus::NotChosen
        } else if i.due_date.as_str() < today_s.as_str() {
            InstallmentStatus::Late
        } else {
            InstallmentStatus::Open
        };
        let suggestion = if matches!(status, InstallmentStatus::Open | InstallmentStatus::Late) {
            suggestions.remove(&i.id)
        } else {
            None
        };
        installments.push(Installment {
            id: i.id.clone(),
            number: i.number,
            due_date: i.due_date.clone(),
            amount: i.amount,
            paid_on: i.paid_on.clone(),
            txn_id: i.txn_id.clone(),
            status,
            suggestion,
        });
    }
    let paid: i64 = rows
        .iter()
        .filter(|i| i.paid_on.is_some())
        .map(|i| i.amount)
        .sum();
    let open_of = |p: Plan| -> i64 {
        rows.iter()
            .filter(|i| i.paid_on.is_none() && belongs(Some(p), i.number))
            .map(|i| i.amount)
            .sum()
    };
    let open = match plan {
        Some(p) => open_of(p),
        None if rows.is_empty() => b.total_due,
        None => {
            let s = open_of(Plan::Single);
            let n = open_of(Plan::Installments);
            match (s, n) {
                (0, n) => n,
                (s, 0) => s,
                (s, n) => s.min(n),
            }
        }
    };
    let venal = venal_total(&b);
    let effective_rate = match (b.taxes_total, venal) {
        (Some(t), Some(v)) if v > 0 => Some(t as f64 / v as f64),
        _ => None,
    };

    // Ano anterior e IPCA.
    let prev: Option<BillRow> = conn
        .query_row(
            &format!("SELECT {BILL_COLS} FROM property_tax_bill WHERE asset_id = ?1 AND year = ?2"),
            params![b.asset_id, b.year - 1],
            bill_row,
        )
        .optional()?;
    let ratio = |new: Option<i64>, old: Option<i64>| match (new, old) {
        (Some(n), Some(o)) if o > 0 => Some(n as f64 / o as f64 - 1.0),
        _ => None,
    };
    let previous = prev.as_ref().map(|p| -> Result<Change> {
        Ok(Change {
            year: p.year,
            venal_change: ratio(venal, venal_total(p)),
            tax_change: ratio(b.taxes_total, p.taxes_total),
            ipca: ipca_of_year(conn, b.year - 1)?,
        })
    });
    let previous = previous.transpose()?;

    // Valor de mercado do bem (no começo do ano do carnê, ou hoje se for antes).
    let jan1 = NaiveDate::from_ymd_opt(b.year, 1, 1).unwrap_or(today);
    let as_of = fmt_date(jan1.min(today));
    let market = match venal {
        Some(v) if v > 0 => {
            let pv = super::view(
                conn,
                &b.asset_id,
                &super::PhysicalReadArgs {
                    as_of: Some(as_of.clone()),
                    deflator: None,
                    include_archived: Some(true),
                },
            )?;
            let f = pv.figures;
            (f.value > 0).then(|| Market {
                value: f.value,
                as_of: f.as_of.clone(),
                estimated: f.estimated,
                ratio: v as f64 / f.value as f64,
            })
        }
        _ => None,
    };

    // Cota única × parcelas.
    let single = rows.iter().find(|i| i.number == 0);
    let parts: Vec<&InstRow> = rows.iter().filter(|i| i.number > 0).collect();
    let svi = match single {
        Some(s) if !parts.is_empty() => {
            let total: i64 = parts.iter().map(|i| i.amount).sum();
            let s_due = parse_date(&s.due_date)?;
            let flows: Vec<(NaiveDate, i64)> = parts
                .iter()
                .map(|i| Ok((parse_date(&i.due_date)?, i.amount)))
                .collect::<Result<_>>()?;
            let rate = implied_monthly_rate(s.amount, s_due, &flows);
            let cdi = cdi_monthly(conn, &s.due_date.clone().min(today_s.clone()))?;
            let best = match (rate, cdi) {
                (Some(r), Some(c)) if r < c => Plan::Installments,
                (Some(_), _) => Plan::Single,
                (None, _) if s.amount < total => Plan::Single,
                (None, _) => Plan::Installments,
            };
            Some(SingleVsInstallments {
                single: s.amount,
                installments: total,
                count: parts.len() as u32,
                saving: total - s.amount,
                monthly_rate: rate,
                cdi_monthly: cdi,
                best,
            })
        }
        _ => None,
    };

    // Percepções.
    let mut insights = vec![];
    if let Some(s) = &svi
        && s.saving > 0
        && rows.iter().all(|i| i.paid_on.is_none())
        && single.is_some_and(|x| x.due_date.as_str() >= today_s.as_str())
    {
        let money = |c: i64| i18n::money(i18n::current(), c, &b.currency);
        let rate = s.monthly_rate.map(pct).unwrap_or_default();
        let (kind, key) = match (s.best, s.cdi_monthly) {
            (Plan::Single, Some(_)) => (InsightKind::PayUpfront, "iptu-percepcao-a-vista-cdi"),
            (Plan::Single, None) => (InsightKind::PayUpfront, "iptu-percepcao-a-vista"),
            (Plan::Installments, _) => (InsightKind::PayInInstallments, "iptu-percepcao-parcelado"),
        };
        insights.push(Insight {
            kind,
            warning: false,
            text: i18n::t_args(
                key,
                &i18n::args(&[
                    ("economia", money(s.saving).into()),
                    ("taxa", rate.into()),
                    ("cdi", s.cdi_monthly.map(pct).unwrap_or_default().into()),
                ]),
            ),
        });
    }
    if let Some(c) = &previous
        && let (Some(t), Some(i)) = (c.tax_change, c.ipca)
        && t > i + 0.005
    {
        insights.push(Insight {
            kind: InsightKind::AboveInflation,
            warning: true,
            text: i18n::t_args(
                "iptu-percepcao-acima-ipca",
                &i18n::args(&[
                    ("variacao", pct(t).into()),
                    ("ipca", pct(i).into()),
                    ("ano", (b.year - 1).to_string().into()),
                ]),
            ),
        });
    }
    if let Some(m) = &market {
        if m.ratio > 1.0 {
            insights.push(Insight {
                kind: InsightKind::VenalAboveMarket,
                warning: true,
                text: i18n::t_args(
                    "iptu-percepcao-venal-acima",
                    &i18n::args(&[("diferenca", pct(m.ratio - 1.0).into())]),
                ),
            });
        } else if m.ratio < 0.4 && !m.estimated {
            insights.push(Insight {
                kind: InsightKind::VenalFarBelowMarket,
                warning: false,
                text: i18n::t_args(
                    "iptu-percepcao-venal-abaixo",
                    &i18n::args(&[("proporcao", pct(m.ratio).into())]),
                ),
            });
        }
    }
    if let Some(t) = b.taxes_total {
        let expected =
            t + b.lighting_fee + b.waste_fee + b.other_fees - b.discount - b.good_payer_discount;
        if (expected - b.total_due).abs() > 100 {
            let money = |c: i64| i18n::money(i18n::current(), c, &b.currency);
            insights.push(Insight {
                kind: InsightKind::TotalMismatch,
                warning: true,
                text: i18n::t_args(
                    "iptu-percepcao-total-diferente",
                    &i18n::args(&[
                        ("calculado", money(expected).into()),
                        ("total", money(b.total_due).into()),
                    ]),
                ),
            });
        }
    }

    Ok(IptuView {
        id: b.id,
        asset_id: b.asset_id,
        year: b.year,
        registration: b.registration,
        city: b.city,
        land_area: b.land_area,
        built_area: b.built_area,
        land_value: b.land_value,
        building_value: b.building_value,
        venal_total: venal,
        land_tax: b.land_tax,
        building_tax: b.building_tax,
        taxes_total: b.taxes_total,
        lighting_fee: b.lighting_fee,
        waste_fee: b.waste_fee,
        other_fees: b.other_fees,
        discount: b.discount,
        good_payer_discount: b.good_payer_discount,
        total_due: b.total_due,
        currency: b.currency,
        plan,
        plan_chosen: b.plan.is_some(),
        notes: b.notes,
        installments,
        effective_rate,
        paid,
        open,
        previous,
        market,
        single_vs_installments: svi,
        insights,
    })
}

/// Os carnês de um imóvel, do ano mais novo para o mais antigo.
pub fn list(conn: &Connection, asset_id: &str, today: NaiveDate) -> Result<Vec<IptuView>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {BILL_COLS} FROM property_tax_bill WHERE asset_id = ?1 ORDER BY year DESC"
    ))?;
    let rows: Vec<BillRow> = stmt
        .query_map([asset_id], bill_row)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|b| view_of(conn, b, today)).collect()
}

pub fn get(conn: &Connection, id: &str, today: NaiveDate) -> Result<IptuView> {
    let b = conn
        .query_row(
            &format!("SELECT {BILL_COLS} FROM property_tax_bill WHERE id = ?1"),
            [id],
            bill_row,
        )
        .optional()?
        .ok_or_else(|| Error::not_found(i18n::t("iptu-carne")))?;
    view_of(conn, b, today)
}

// ---------------------------------------------------------------------------
// Para os lembretes e os impostos pagos

/// Parcela a lembrar: em aberto, da forma escolhida (sem forma: todas), sem lançamento
/// provável que fale em IPTU.
pub struct DueInstallment {
    pub id: Id,
    pub asset_name: String,
    pub year: i32,
    pub number: u32,
    pub due_date: String,
    pub amount: i64,
    pub currency: String,
    /// Sem forma escolhida e com cota única: a mensagem lembra de decidir.
    pub undecided: bool,
}

/// Parcelas em aberto com vencimento até `until` (as vencidas há até 30 dias também).
pub fn due_installments(
    conn: &Connection,
    today: NaiveDate,
    until: NaiveDate,
) -> Result<Vec<DueInstallment>> {
    let mut stmt = conn.prepare(
        "SELECT b.id, a.name, b.year, b.currency, b.plan FROM property_tax_bill b
           JOIN physical_asset a ON a.id = b.asset_id
          WHERE a.archived = 0 AND a.sale_date IS NULL",
    )?;
    let bills: Vec<(String, String, i32, String, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let from = fmt_date(today - chrono::Duration::days(30));
    let until = fmt_date(until);
    let mut out = vec![];
    for (bill, name, year, currency, plan) in bills {
        let rows = installments_of(conn, &bill)?;
        let plan = effective_plan(plan.as_deref().and_then(Plan::parse), &rows);
        let has_single = rows.iter().any(|i| i.number == 0);
        let open: Vec<&InstRow> = rows
            .iter()
            .filter(|i| i.paid_on.is_none() && belongs(plan, i.number))
            .collect();
        let paid_by = assign_suggestions(conn, &open)?;
        for i in rows.iter().filter(|i| {
            i.paid_on.is_none()
                && belongs(plan, i.number)
                && i.due_date.as_str() >= from.as_str()
                && i.due_date.as_str() <= until.as_str()
        }) {
            // Sem forma escolhida: lembra só a primeira data (cota única ou parcela 1).
            if plan.is_none() && i.number > 1 {
                continue;
            }
            if plan.is_none() && i.number == 1 && has_single {
                continue;
            }
            if paid_by.get(&i.id).is_some_and(|m| m.named) {
                continue;
            }
            out.push(DueInstallment {
                id: i.id.clone(),
                asset_name: name.clone(),
                year,
                number: i.number,
                due_date: i.due_date.clone(),
                amount: i.amount,
                currency: currency.clone(),
                undecided: plan.is_none() && has_single,
            });
        }
    }
    Ok(out)
}

/// Imóvel com o carnê do ano anterior e sem o deste, quando o deste já deve ter chegado
/// (20 dias antes do primeiro vencimento do ano anterior, no mesmo dia e mês).
pub struct MissingBill {
    pub asset_id: Id,
    pub asset_name: String,
    pub year: i32,
}

pub fn missing_bills(conn: &Connection, today: NaiveDate) -> Result<Vec<MissingBill>> {
    let year = today.year();
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, MIN(i.due_date) FROM property_tax_bill b
           JOIN physical_asset a ON a.id = b.asset_id
           LEFT JOIN property_tax_installment i ON i.bill_id = b.id
          WHERE b.year = ?1 AND a.archived = 0 AND a.sale_date IS NULL
            AND NOT EXISTS (SELECT 1 FROM property_tax_bill n WHERE n.asset_id = a.id AND n.year = ?2)
          GROUP BY a.id, a.name",
    )?;
    let rows: Vec<(String, String, Option<String>)> = stmt
        .query_map(params![year - 1, year], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (asset_id, asset_name, first_due) in rows {
        let (m, d) = first_due
            .as_deref()
            .and_then(|s| parse_date(s).ok())
            .map(|d| (d.month(), d.day()))
            .unwrap_or((2, 10));
        let same = NaiveDate::from_ymd_opt(year, m, d.min(28))
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, 2, 10).expect("data válida"));
        if today >= same - chrono::Duration::days(20) {
            out.push(MissingBill {
                asset_id,
                asset_name,
                year,
            });
        }
    }
    Ok(out)
}

/// Uma linha de IPTU para os impostos pagos: o que o carnê manda pagar no período.
pub struct PeriodTax {
    pub asset_id: Id,
    pub asset_name: String,
    #[allow(dead_code)]
    pub year: i32,
    pub venal: i64,
    pub amount: i64,
    pub paid: bool,
}

/// O IPTU de cada carnê com vencimentos no período (a forma escolhida; sem forma, a cota
/// única quando houver).
pub fn period_taxes(conn: &Connection, from: &str, to: &str) -> Result<Vec<PeriodTax>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {BILL_COLS} FROM property_tax_bill WHERE id IN (
            SELECT bill_id FROM property_tax_installment WHERE due_date BETWEEN ?1 AND ?2)"
    ))?;
    let bills: Vec<BillRow> = stmt
        .query_map(params![from, to], bill_row)?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for b in bills {
        let rows = installments_of(conn, &b.id)?;
        let plan = effective_plan(b.plan, &rows).or(if rows.iter().any(|i| i.number == 0) {
            Some(Plan::Single)
        } else {
            Some(Plan::Installments)
        });
        let chosen: Vec<&InstRow> = rows
            .iter()
            .filter(|i| {
                belongs(plan, i.number) && i.due_date.as_str() >= from && i.due_date.as_str() <= to
            })
            .collect();
        if chosen.is_empty() {
            continue;
        }
        let name: String = conn.query_row(
            "SELECT name FROM physical_asset WHERE id = ?1",
            [&b.asset_id],
            |r| r.get(0),
        )?;
        out.push(PeriodTax {
            venal: venal_total(&b).unwrap_or(0),
            amount: chosen.iter().map(|i| i.amount).sum(),
            paid: chosen.iter().all(|i| i.paid_on.is_some()),
            asset_id: b.asset_id,
            asset_name: name,
            year: b.year,
        });
    }
    Ok(out)
}

#[cfg(test)]
#[path = "iptu_tests.rs"]
mod tests;

#[cfg(test)]
mod unit {
    use super::check_area;

    #[test]
    fn areas_in_both_decimal_styles() {
        let a = |s: &str| check_area(&Some(s.into())).unwrap().unwrap();
        assert_eq!(a("72.5"), "72.5");
        assert_eq!(a("72,5"), "72.5");
        assert_eq!(a("1.250,5"), "1250.5");
        assert_eq!(a("1.250"), "1250");
        assert_eq!(a("1250"), "1250");
        assert_eq!(a("1.250.000"), "1250000");
        assert!(check_area(&Some("-3".into())).is_err());
        assert!(check_area(&Some("abc".into())).is_err());
    }
}
