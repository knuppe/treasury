//! Entradas das leituras que a interface já mostrava e o MCP ainda não lia (D18; auditoria da
//! visão de 17/11/2026): IR anual e DARFs, informes de rendimentos, contas de aposentadoria,
//! simulações de dívida, Consumo e algumas leituras menores.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::inputs::{
    DATE_PATTERN, MONTH_PATTERN, ToolInput, check_date, check_month, check_non_empty, to_object,
    tx, tx_args, tx_campo,
};

fn check_year(year: i32) -> Result<(), String> {
    if (2000..=2100).contains(&year) {
        Ok(())
    } else {
        Err(tx("mcp-valida-ano"))
    }
}

fn check_not_negative(field: &str, v: i64) -> Result<(), String> {
    if v < 0 {
        return Err(tx_campo("mcp-valida-nao-negativo", field));
    }
    Ok(())
}

fn check_limit(limit: Option<i64>) -> Result<(), String> {
    if let Some(l) = limit
        && !(1..=200).contains(&l)
    {
        return Err(tx_args(
            "mcp-valida-faixa",
            &[
                ("campo", "limit".to_string().into()),
                ("min", 1.into()),
                ("max", 200.into()),
            ],
        ));
    }
    Ok(())
}

/// Corta a lista em `limit` (padrão `default`) e diz quantas havia.
fn cap(value: Value, limit: Option<i64>, default: i64) -> Value {
    let Value::Array(mut items) = value else {
        return value;
    };
    let total = items.len();
    items.truncate(limit.unwrap_or(default).clamp(1, 200) as usize);
    json!({ "items": items, "total": total, "truncated": total > limit.unwrap_or(default) as usize })
}

// ---------------------------------------------------------------------------
// IR (Brasil)

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnnualTaxDraftInput {
    /// Calendar year of the income (the return is filed the following year), e.g. 2025.
    pub year: i32,
    /// Member (CPF) whose return to draft (see `list_members`); default: the whole family.
    pub member_id: Option<String>,
}

impl ToolInput for AnnualTaxDraftInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_year(self.year)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListDarfsInput {
    /// Only DARFs of this calendar year (by the tax period).
    pub year: Option<i32>,
    /// Only this member's DARFs; default: all.
    pub member_id: Option<String>,
}

impl ToolInput for ListDarfsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(y) = self.year {
            check_year(y)?;
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DarfChargesInput {
    /// DARF id (see `list_darfs`).
    pub id: String,
    /// Payment date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub paid_at: Option<String>,
}

impl ToolInput for DarfChargesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("paid_at", &self.paid_at)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CarneLeaoInput {
    /// Month `YYYY-MM` the income was received.
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: String,
    /// Received in the month from individuals and from abroad, in cents (BRL).
    pub income: i64,
    /// Official social security (INSS) paid in the month, in cents.
    pub social_security: Option<i64>,
    /// Number of dependents.
    pub dependents: Option<u32>,
    /// Court-ordered alimony paid in the month, in cents.
    pub alimony: Option<i64>,
    /// Professional expenses (livro-caixa) of the month, in cents.
    pub livro_caixa: Option<i64>,
    /// Tax paid abroad on that income in the month, in cents (offsets the tax due).
    pub foreign_tax: Option<i64>,
}

impl ToolInput for CarneLeaoInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("month", &Some(self.month.clone()))?;
        check_not_negative("income", self.income)?;
        for (f, v) in [
            ("social_security", self.social_security),
            ("alimony", self.alimony),
            ("livro_caixa", self.livro_caixa),
            ("foreign_tax", self.foreign_tax),
        ] {
            check_not_negative(f, v.unwrap_or(0))?;
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalGainInput {
    /// Physical asset id (see `list_physical_assets`).
    pub asset_id: String,
    /// Sale date `YYYY-MM-DD` (default: the sale recorded on the asset).
    #[schemars(pattern(DATE_PATTERN))]
    pub sale_date: Option<String>,
    /// Sale price in cents (default: the one recorded on the asset).
    pub sale_price: Option<i64>,
    /// Sale costs (brokerage, fees) in cents.
    pub sale_costs: Option<i64>,
    /// The family's only property (exemption up to the legal limit).
    pub only_property: Option<bool>,
    /// Amount reinvested in another home within 180 days, in cents (exemption).
    pub reinvested: Option<i64>,
}

impl ToolInput for PhysicalGainInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_date("sale_date", &self.sale_date)?;
        for (f, v) in [
            ("sale_price", self.sale_price),
            ("sale_costs", self.sale_costs),
            ("reinvested", self.reinvested),
        ] {
            check_not_negative(f, v.unwrap_or(0))?;
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IncomeReportsInput {
    /// Calendar year (default: the previous year).
    pub year: Option<i32>,
}

impl ToolInput for IncomeReportsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(y) = self.year {
            check_year(y)?;
        }
        to_object(self)
    }
}

// ---------------------------------------------------------------------------
// Dívidas

/// `last`: from the last installments backwards (shortens the contract); `next`: the next
/// installments onwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AnticipateOrder {
    Last,
    Next,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnticipationInput {
    /// Contract id (see `list_loans`).
    pub loan_id: String,
    /// Payment date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// How many installments to pay in advance.
    pub count: u32,
    /// Which installments: `last` or `next`.
    pub order: AnticipateOrder,
}

impl ToolInput for AnticipationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("loan_id", &self.loan_id)?;
        check_date("date", &Some(self.date.clone()))?;
        if self.count == 0 {
            return Err(tx_campo("mcp-valida-positivo", "count"));
        }
        to_object(self)
    }
}

/// Amortization system of the new contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AmortSystem {
    Sac,
    Price,
    Bullet,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PortabilityInput {
    /// Contract id (see `list_loans`).
    pub loan_id: String,
    /// New effective annual rate in percent, as decimal text (e.g. "11.5").
    pub annual_rate: String,
    /// New term in months (default: the remaining term).
    pub term_months: Option<u32>,
    /// Amortization system of the new contract (default: the current one).
    pub system: Option<AmortSystem>,
    /// Portability costs (fees, appraisal), in cents.
    pub costs: Option<i64>,
}

impl ToolInput for PortabilityInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("loan_id", &self.loan_id)?;
        if self
            .annual_rate
            .trim()
            .parse::<rust_decimal::Decimal>()
            .is_err()
        {
            return Err(tx_campo("mcp-valida-decimal", "annual_rate"));
        }
        check_not_negative("costs", self.costs.unwrap_or(0))?;
        to_object(self)
    }
}

// ---------------------------------------------------------------------------
// Consumo

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConsumptionSummaryInput {
    /// Month `YYYY-MM` (default: the current month).
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: Option<String>,
    /// Only this member's share (see `list_members`).
    pub member_id: Option<String>,
    /// Detail one segment (root product category id, from a previous call): its classes,
    /// categories and products.
    pub segment_id: Option<String>,
}

impl ToolInput for ConsumptionSummaryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("month", &self.month)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListProductsInput {
    /// `true`: food only; `false`: home and hygiene only; absent: all.
    pub food: Option<bool>,
    /// Words of the product name, or a barcode.
    pub query: Option<String>,
    /// Product category id.
    pub category_id: Option<String>,
    /// How many to return (1–200, default 50).
    pub limit: Option<i64>,
}

impl ToolInput for ListProductsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_limit(self.limit)?;
        Ok(json!({
            "food": self.food,
            "query": self.query,
            "category_id": self.category_id,
        }))
        .map(|mut v| {
            if let Value::Object(m) = &mut v {
                m.retain(|_, x| !x.is_null());
            }
            v
        })
    }

    fn post_process(&self, value: Value) -> Value {
        cap(value, self.limit, 50)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductHistoryInput {
    /// Product id (see `list_products`).
    pub id: String,
}

impl ToolInput for ProductHistoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListPurchasesInput {
    /// Start date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub from: Option<String>,
    /// End date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub to: Option<String>,
    /// Only purchases of this member (see `list_members`).
    pub member_id: Option<String>,
    /// Store or item words.
    pub query: Option<String>,
    /// Only purchases with items of this segment (root product category id).
    pub segment_id: Option<String>,
    /// How many to return, newest first (1–200, default 30).
    pub limit: Option<i64>,
}

impl ToolInput for ListPurchasesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        check_limit(self.limit)?;
        let mut v = json!({
            "from": self.from,
            "to": self.to,
            "member_id": self.member_id,
            "query": self.query,
            "segment_id": self.segment_id,
        });
        if let Value::Object(m) = &mut v {
            m.retain(|_, x| !x.is_null());
        }
        Ok(v)
    }

    fn post_process(&self, value: Value) -> Value {
        cap(value, self.limit, 30)
    }
}

// ---------------------------------------------------------------------------
// Leituras menores

/// `day` (one point per day) or `month` (one point at the end of each month).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum BalanceInterval {
    Day,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BalanceHistoryInput {
    /// Account id (see `list_accounts`).
    pub id: String,
    /// Start date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub from: String,
    /// End date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub to: String,
    /// Default `day`; use `month` for periods over a few months.
    pub interval: Option<BalanceInterval>,
    /// Include planned (future recurring) transactions. Default: no.
    pub include_planned: Option<bool>,
}

impl ToolInput for BalanceHistoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("from", &Some(self.from.clone()))?;
        check_date("to", &Some(self.to.clone()))?;
        if self.from > self.to {
            return Err(tx_args(
                "mcp-valida-ordem",
                &[
                    ("de", "from".to_string().into()),
                    ("ate", "to".to_string().into()),
                ],
            ));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListPayslipsInput {
    /// Only this member's payslips (see `list_members`).
    pub member_id: Option<String>,
    /// Income source id (see `get_income_summary`).
    pub source_id: Option<String>,
}

impl ToolInput for ListPayslipsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FxOperationsInput {
    /// Only operations touching this account.
    pub account_id: Option<String>,
}

impl ToolInput for FxOperationsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardCommitmentsInput {
    /// Card account id; default: all cards.
    pub account_id: Option<String>,
    /// Months from the current one (default 12, max 120).
    pub months: Option<u32>,
}

impl ToolInput for CardCommitmentsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(m) = self.months
            && !(1..=120).contains(&m)
        {
            return Err(tx_args(
                "mcp-valida-faixa",
                &[
                    ("campo", "months".to_string().into()),
                    ("min", 1.into()),
                    ("max", 120.into()),
                ],
            ));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalHistoryInput {
    /// Physical asset id (see `list_physical_assets`).
    pub id: String,
    /// How many months back (default: since the purchase).
    pub months: Option<u32>,
}

impl ToolInput for PhysicalHistoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        to_object(self)
    }
}
