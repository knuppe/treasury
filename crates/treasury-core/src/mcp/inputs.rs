//! Entradas das ferramentas MCP: esquemas JSON (schemars) espelhando os DTOs do núcleo
//! (`TxnFilter`, `SpendingArgs`, `ExplainArgs`, `ChangeListArgs`…), com descrições em
//! inglês, validação (mensagens no idioma da instalação, `mcp.ftl`) e conversão para os
//! argumentos do comando da API.
//!
//! Os DTOs de domínio não derivam `JsonSchema` (são de outros módulos); os testes de
//! contrato conferem que cada entrada convertida desserializa no DTO correspondente.

use fluent_bundle::FluentValue;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::i18n::{args, t, t_args};
use crate::time;

/// Tamanho de página padrão de `query_transactions`.
pub const TXN_DEFAULT_LIMIT: i64 = 50;
/// Tamanho máximo de página de `query_transactions`.
pub const TXN_MAX_LIMIT: i64 = 200;
/// Padrão e máximo de `get_change_history`.
pub const CHANGES_DEFAULT_LIMIT: i64 = 20;
pub const CHANGES_MAX_LIMIT: i64 = 100;
/// Máximo de meses em `get_monthly_flow`.
pub const FLOW_MAX_MONTHS: i64 = 120;

pub(crate) const DATE_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}$";
pub(crate) const MONTH_PATTERN: &str = r"^\d{4}-\d{2}$";

/// Mensagem (de validação, nota ou resumo) no idioma da instalação: `locales/<idioma>/mcp.ftl`.
pub(crate) fn tx(key: &str) -> String {
    t(key)
}

/// Mensagem sobre um campo da ferramenta (`$campo`, que fica como está).
pub(crate) fn tx_campo(key: &str, campo: &str) -> String {
    tx_args(key, &[("campo", campo.to_string().into())])
}

/// Mensagem com argumentos.
pub(crate) fn tx_args(key: &str, a: &[(&'static str, FluentValue<'static>)]) -> String {
    t_args(key, &args(a))
}

/// Entrada de uma ferramenta: validação e montagem dos argumentos do comando.
pub(crate) trait ToolInput: DeserializeOwned + JsonSchema + Send + 'static {
    /// Valida e devolve os argumentos JSON do comando da API (mensagem no idioma da instalação).
    fn to_api_args(&self) -> Result<Value, String>;
    /// Ajustes locais no resultado (filtros que o comando não oferece).
    fn post_process(&self, value: Value) -> Value {
        value
    }
}

// ---------------------------------------------------------------------------
// Enums espelhados

/// `accrual` = accrual basis (purchase date; default). `cash` = cash basis (payment date;
/// credit card purchases count on the card statement due date).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Basis {
    Accrual,
    Cash,
}

/// Transaction type: `income` income, `expense` expense, `transfer` transfer between
/// accounts, `card_payment` card statement payment, `investment` contribution/withdrawal,
/// `adjustment` adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TxnKind {
    Income,
    Expense,
    Transfer,
    CardPayment,
    Investment,
    Adjustment,
}

/// `planned` planned (not counted in the balance), `cleared` cleared, `reconciled` reconciled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TxnStatus {
    Planned,
    Cleared,
    Reconciled,
}

/// Sort order: `date_desc` (default), `date_asc`, `amount_desc`, `amount_asc`, and by text
/// `description_*`, `account_*`, `category_*`, `member_*` and `status_*` (`_asc`/`_desc`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TxnSort {
    DateDesc,
    DateAsc,
    AmountDesc,
    AmountAsc,
    DescriptionAsc,
    DescriptionDesc,
    AccountAsc,
    AccountDesc,
    CategoryAsc,
    CategoryDesc,
    MemberAsc,
    MemberDesc,
    StatusAsc,
    StatusDesc,
}

/// `expense` expenses (default) or `income` income.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum CategoryKind {
    Income,
    Expense,
}

/// Spending group of an expense category (where the income goes): `essential` (housing,
/// groceries, utilities, transportation, health, school), `debt` (loan and financing
/// installments, interest, card fees), `future` (emergency fund, investments, retirement, goals)
/// or `non_essential` (leisure, restaurants, shopping, subscriptions, travel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum SpendGroup {
    Essential,
    Debt,
    Future,
    NonEssential,
}

/// Grouping: `category` leaf category (default, with `parent_key`), `parent_category`
/// top level only, `member` member, `account` account, `month` month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum GroupBy {
    Category,
    ParentCategory,
    Member,
    Account,
    Month,
}

/// Author of the change set: `user`, `import`, `ai` or `system`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AuthorKind {
    User,
    Import,
    Ai,
    System,
}

/// Metric to explain: `account_balance` (requires `account_id`; `as_of` optional),
/// `net_worth` (`as_of` optional), `spending` (requires `from` and `to`; `basis`,
/// `category_id` and `member_id` optional), `income` (requires `from` and `to`; `basis`
/// optional), `savings_rate` (requires `month`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Metric {
    AccountBalance,
    NetWorth,
    Spending,
    Income,
    SavingsRate,
}

// ---------------------------------------------------------------------------
// Validação

pub(crate) fn check_date(field: &str, v: &Option<String>) -> Result<(), String> {
    if let Some(d) = v {
        time::parse_date(d).map_err(|_| tx_campo("mcp-valida-data", field))?;
    }
    Ok(())
}

pub(crate) fn check_month(field: &str, v: &Option<String>) -> Result<(), String> {
    if let Some(m) = v {
        time::parse_month(m).map_err(|_| tx_campo("mcp-valida-mes", field))?;
    }
    Ok(())
}

fn check_range(from_field: &str, from: &str, to_field: &str, to: &str) -> Result<(), String> {
    if from > to {
        return Err(tx_args(
            "mcp-valida-ordem",
            &[
                ("de", from_field.to_string().into()),
                ("ate", to_field.to_string().into()),
            ],
        ));
    }
    Ok(())
}

pub(crate) fn check_non_empty(field: &str, v: &str) -> Result<(), String> {
    if v.trim().is_empty() {
        return Err(tx_campo("mcp-valida-informe", field));
    }
    Ok(())
}

/// Serializa a entrada removendo campos ausentes.
pub(crate) fn to_object<T: Serialize>(v: &T) -> Result<Value, String> {
    let mut value = serde_json::to_value(v).map_err(|e| e.to_string())?;
    if let Value::Object(map) = &mut value {
        map.retain(|_, v| !v.is_null());
    }
    Ok(value)
}

fn current_month() -> String {
    time::month_of(time::today_local())
}

fn months_between(from: &str, to: &str) -> Result<i64, String> {
    let (fy, fm) = time::parse_month(from).map_err(|e| e.to_string())?;
    let (ty, tm) = time::parse_month(to).map_err(|e| e.to_string())?;
    Ok((ty as i64 * 12 + tm as i64) - (fy as i64 * 12 + fm as i64) + 1)
}

fn shift_month(month: &str, delta: i32) -> String {
    let (y, m) = time::parse_month(month).unwrap_or((2000, 1));
    let total = y * 12 + (m as i32 - 1) + delta;
    time::fmt_month(total.div_euclid(12), (total.rem_euclid(12) + 1) as u32)
}

fn filter_array(value: Value, keep: impl Fn(&Map<String, Value>) -> bool) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .filter(|v| v.as_object().is_none_or(&keep))
                .collect(),
        ),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Entradas

/// No parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EmptyInput {}

impl ToolInput for EmptyInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(Value::Null)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NetWorthInput {
    /// Reference date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub as_of: Option<String>,
}

impl ToolInput for NetWorthInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("as_of", &self.as_of)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListAccountsInput {
    /// Include archived accounts (default: no).
    pub include_archived: Option<bool>,
}

impl ToolInput for ListAccountsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountBalanceInput {
    /// Account id (see `list_accounts`).
    pub account_id: String,
}

impl ToolInput for AccountBalanceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        Ok(json!({ "id": self.account_id.trim() }))
    }
}

/// Transaction filter (same fields as the core `TxnFilter`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryTransactionsInput {
    /// Start date `YYYY-MM-DD` (inclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "from", alias = "start_date", alias = "from_date")]
    pub date_from: Option<String>,
    /// End date `YYYY-MM-DD` (inclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "to", alias = "end_date", alias = "to_date")]
    pub date_to: Option<String>,
    /// Date basis for the filter (default `accrual`).
    pub basis: Option<Basis>,
    /// Only these accounts (ids from `list_accounts`).
    pub account_ids: Option<Vec<String>>,
    /// Only these categories (ids from `list_categories`).
    pub category_ids: Option<Vec<String>>,
    /// Include subcategories of the filtered categories (default: yes).
    pub include_subcategories: Option<bool>,
    /// Only these members (ids from `list_members`).
    pub member_ids: Option<Vec<String>>,
    /// Only these transaction types.
    pub kinds: Option<Vec<TxnKind>>,
    /// Only these statuses (default: all, including planned).
    pub statuses: Option<Vec<TxnStatus>>,
    /// Searches description, payee and notes (case- and accent-insensitive).
    #[schemars(length(max = 200))]
    pub text: Option<String>,
    /// Minimum absolute amount, in cents (e.g. 10000 = 100.00).
    #[schemars(range(min = 0))]
    pub amount_min: Option<i64>,
    /// Maximum absolute amount, in cents.
    #[schemars(range(min = 0))]
    pub amount_max: Option<i64>,
    /// Only transactions with these tags.
    pub tag_ids: Option<Vec<String>>,
    /// Only transactions from these projects (see `list_projects`).
    pub project_ids: Option<Vec<String>>,
    /// Only transactions from this import batch.
    pub import_id: Option<String>,
    /// Only transactions on this card statement (see `list_card_statements`).
    pub statement_id: Option<String>,
    /// Only uncategorized transactions.
    pub uncategorized: Option<bool>,
    /// Sort order (default `date_desc`).
    pub sort: Option<TxnSort>,
    /// Page size (default 50, max 200).
    #[schemars(range(min = 1, max = 200))]
    pub limit: Option<i64>,
    /// How many results to skip (pagination; default 0).
    #[schemars(range(min = 0))]
    pub offset: Option<i64>,
}

impl ToolInput for QueryTransactionsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("date_from", &self.date_from)?;
        check_date("date_to", &self.date_to)?;
        if let (Some(f), Some(t)) = (&self.date_from, &self.date_to) {
            check_range("date_from", f, "date_to", t)?;
        }
        if self.amount_min.is_some_and(|v| v < 0) || self.amount_max.is_some_and(|v| v < 0) {
            return Err(tx("mcp-valida-valores-absolutos"));
        }
        if let (Some(a), Some(b)) = (self.amount_min, self.amount_max)
            && a > b
        {
            return Err(tx("mcp-valida-minimo-maior"));
        }
        if self.offset.is_some_and(|v| v < 0) {
            return Err(tx_campo("mcp-valida-nao-negativo", "offset"));
        }
        let mut input = self.clone();
        input.limit = Some(
            self.limit
                .unwrap_or(TXN_DEFAULT_LIMIT)
                .clamp(1, TXN_MAX_LIMIT),
        );
        input.offset = Some(self.offset.unwrap_or(0));
        to_object(&input)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpendingSummaryInput {
    /// Start date `YYYY-MM-DD` (inclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: String,
    /// End date `YYYY-MM-DD` (inclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: String,
    /// Date basis (default `accrual`).
    pub basis: Option<Basis>,
    /// Expenses (default) or income.
    pub kind: Option<CategoryKind>,
    /// Grouping (default `category`).
    pub group_by: Option<GroupBy>,
    /// Only these members.
    pub member_ids: Option<Vec<String>>,
    /// Only these accounts.
    pub account_ids: Option<Vec<String>>,
    /// Only these categories (and their subcategories).
    pub category_ids: Option<Vec<String>>,
}

impl ToolInput for SpendingSummaryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &Some(self.from.clone()))?;
        check_date("to", &Some(self.to.clone()))?;
        check_range("from", &self.from, "to", &self.to)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BudgetStatusInput {
    /// Month `YYYY-MM` (default: current month).
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: Option<String>,
    /// Date basis (default: the app setting, usually `accrual`).
    pub basis: Option<Basis>,
}

impl ToolInput for BudgetStatusInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("month", &self.month)?;
        let mut input = self.clone();
        input.month = Some(self.month.clone().unwrap_or_else(current_month));
        to_object(&input)
    }
}

/// `average` or `median` of the months (the median ignores one month out of the curve).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum SuggestMethodInput {
    Average,
    Median,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SuggestBudgetInput {
    /// Month to budget `YYYY-MM` (default: current month).
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: Option<String>,
    /// Complete months before it to look at (1 to 24; default 6).
    #[schemars(range(min = 1, max = 24))]
    pub months_back: Option<u32>,
    /// Default `median`.
    pub method: Option<SuggestMethodInput>,
    /// Date basis (default: the app setting, usually `accrual`).
    pub basis: Option<Basis>,
}

impl ToolInput for SuggestBudgetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("month", &self.month)?;
        let mut v = to_object(self)?;
        v["month"] = Value::from(self.month.clone().unwrap_or_else(current_month));
        v["months_back"] = Value::from(self.months_back.unwrap_or(6).clamp(1, 24));
        v["method"] = serde_json::to_value(self.method.unwrap_or(SuggestMethodInput::Median))
            .map_err(|e| e.to_string())?;
        if self.basis.is_none() {
            v.as_object_mut().map(|o| o.remove("basis"));
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardStatementsInput {
    /// Credit card account id (see `list_accounts`, type `credit_card`).
    pub account_id: String,
    /// First due-date month `YYYY-MM` (optional).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "from", alias = "start_month")]
    pub from_month: Option<String>,
    /// Last due-date month `YYYY-MM` (optional).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "to", alias = "end_month")]
    pub to_month: Option<String>,
}

/// Cards of a credit card account (holder, additional, virtual).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardPlasticsInput {
    /// Credit card account id (see `list_accounts`); all cards when omitted.
    pub account_id: Option<String>,
}

impl ToolInput for CardPlasticsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// One credit card account.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardOverviewInput {
    /// Credit card account id (see `list_accounts`, type `credit_card`).
    pub account_id: String,
}

impl ToolInput for CardOverviewInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        to_object(self)
    }
}

/// How the annual fee is waived.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FeeWaiverInput {
    /// No waiver.
    None,
    /// Always waived.
    Full,
    /// The installment charged on a bill is waived when that bill's purchases reach `waiver_threshold`.
    Spend,
    /// Waived while the balance at the card's bank reaches `waiver_threshold`.
    Investment,
    /// Fixed discount (`waiver_discount`, "0.5" = 50%).
    Discount,
}

/// The card's annual fee rule.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetCardAnnualFeeInput {
    /// Credit card account id.
    pub account_id: String,
    /// Full yearly amount in cents.
    pub amount: i64,
    /// Installments per year (1 to 12, default 12).
    pub installments: Option<u32>,
    /// Month the fee renews (1 to 12).
    pub anniversary_month: Option<u32>,
    /// How the fee is waived (default none).
    pub waiver_kind: Option<FeeWaiverInput>,
    /// Spending per bill or amount invested that waives the fee, in cents.
    pub waiver_threshold: Option<i64>,
    /// Discount as a decimal in text, "0.5" = 50%.
    pub waiver_discount: Option<String>,
    /// Date the rule starts `YYYY-MM-DD` (default today).
    #[schemars(pattern(DATE_PATTERN))]
    pub starts_on: Option<String>,
    /// Free notes (e.g. what the bank agreed on the phone).
    pub notes: Option<String>,
}

impl ToolInput for SetCardAnnualFeeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        check_date("starts_on", &self.starts_on)?;
        to_object(self)
    }
}

/// Reward program kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RewardKindInput {
    /// Money back.
    Cashback,
    /// Points or miles.
    Points,
}

/// Where the cashback goes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RewardDestinationInput {
    /// Credit on the card bill.
    Statement,
    /// Deposit in an account (`destination_account_id`).
    Account,
    /// Into an investment (`destination_account_id`).
    Investment,
}

/// A category with its own rate.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RewardMultiplierToolInput {
    /// Expense category id (see `list_categories`).
    pub category_id: String,
    /// Rate for that category, decimal in text.
    pub rate: String,
}

/// The card's reward program.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetCardRewardProgramInput {
    /// Credit card account id.
    pub account_id: String,
    pub kind: RewardKindInput,
    /// Cashback: fraction in text ("0.01" = 1%). Points: points per unit of `rate_currency` ("2.2").
    pub base_rate: String,
    /// Points: currency the rate counts in (many programs count in USD).
    pub rate_currency: Option<String>,
    /// Cap per bill: cents (cashback) or points.
    pub cycle_cap: Option<i64>,
    /// Where the cashback goes (default: the bill).
    pub destination: Option<RewardDestinationInput>,
    /// Account that receives the cashback, when it isn't the bill.
    pub destination_account_id: Option<String>,
    /// Days after closing until the credit usually arrives (default 40).
    pub payout_days: Option<u32>,
    /// Date the rule starts `YYYY-MM-DD` (default today).
    #[schemars(pattern(DATE_PATTERN))]
    pub starts_on: Option<String>,
    /// Free notes.
    pub notes: Option<String>,
    /// Categories with their own rate.
    pub multipliers: Option<Vec<RewardMultiplierToolInput>>,
    /// Points: the points program they go to (see `list_loyalty_programs`).
    pub loyalty_program_id: Option<String>,
}

impl ToolInput for SetCardRewardProgramInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        check_non_empty("base_rate", &self.base_rate)?;
        check_date("starts_on", &self.starts_on)?;
        to_object(self)
    }
}

/// No arguments.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoyaltyProgramsInput {}

impl ToolInput for CardValueInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// No arguments.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardValueInput {}

impl ToolInput for LoyaltyProgramsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// Points entry kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoyaltyEntryKindInput {
    /// Points earned (e.g. from a card bill).
    Earn,
    /// Promotional bonus.
    Bonus,
    /// Points bought (`cost` in cents).
    Purchase,
    /// Points used (tickets, products).
    Redeem,
    /// Points that expired.
    Expire,
    /// Correction, with sign.
    Adjust,
}

/// A points or miles entry read from the program's statement.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordLoyaltyEntryInput {
    /// Points program id (see `list_loyalty_programs`).
    pub program_id: String,
    /// Date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Number of points (positive; the kind gives the sign, except `adjust`).
    pub points: i64,
    pub kind: LoyaltyEntryKindInput,
    /// When these points expire `YYYY-MM-DD` (earned, bonus, bought).
    #[schemars(pattern(DATE_PATTERN))]
    pub expires_on: Option<String>,
    /// Card bill that generated the points, to compare with the expected.
    pub statement_id: Option<String>,
    /// Points bought: what they cost, in cents.
    pub cost: Option<i64>,
    /// Free notes.
    pub notes: Option<String>,
}

impl ToolInput for RecordLoyaltyEntryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("program_id", &self.program_id)?;
        check_date("date", &Some(self.date.clone()))?;
        check_date("expires_on", &self.expires_on)?;
        to_object(self)
    }
}

impl ToolInput for CardStatementsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        check_month("from_month", &self.from_month)?;
        check_month("to_month", &self.to_month)?;
        if let (Some(f), Some(t)) = (&self.from_month, &self.to_month) {
            check_range("from_month", f, "to_month", t)?;
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListGoalsInput {
    /// Include archived goals (default: no).
    pub include_archived: Option<bool>,
}

impl ToolInput for ListGoalsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangeHistoryInput {
    /// How many change sets to return (default 20, max 100).
    #[schemars(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    /// How many to skip (pagination).
    #[schemars(range(min = 0))]
    pub offset: Option<i64>,
    /// Only changes by this author.
    pub author_kind: Option<AuthorKind>,
    /// Only changes that touched this table (e.g. `txn`, `account`, `budget_line`).
    pub table_name: Option<String>,
    /// With `table_name`: only changes to this row (e.g. a transaction id).
    pub row_id: Option<String>,
}

impl ToolInput for ChangeHistoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.row_id.is_some() && self.table_name.is_none() {
            return Err(tx("mcp-valida-row-id"));
        }
        if self.offset.is_some_and(|v| v < 0) {
            return Err(tx_campo("mcp-valida-nao-negativo", "offset"));
        }
        let mut input = self.clone();
        input.limit = Some(
            self.limit
                .unwrap_or(CHANGES_DEFAULT_LIMIT)
                .clamp(1, CHANGES_MAX_LIMIT),
        );
        to_object(&input)
    }
}

/// Where a number comes from. Each metric's parameters are top-level fields.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExplainValueInput {
    /// Metric to explain.
    pub metric: Metric,
    /// Account (`account_balance`).
    pub account_id: Option<String>,
    /// Reference date `YYYY-MM-DD` (`account_balance`, `net_worth`; default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub as_of: Option<String>,
    /// Start date `YYYY-MM-DD` (`spending`, `income`).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// End date `YYYY-MM-DD` (`spending`, `income`).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
    /// Date basis (`spending`, `income`).
    pub basis: Option<Basis>,
    /// Category (`spending`, optional).
    pub category_id: Option<String>,
    /// Member (`spending`, optional).
    pub member_id: Option<String>,
    /// Month `YYYY-MM` (`savings_rate`).
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: Option<String>,
    /// Maximum number of sources listed (default 200).
    #[schemars(range(min = 1, max = 500))]
    pub limit: Option<u32>,
}

impl ToolInput for ExplainValueInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("as_of", &self.as_of)?;
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        check_month("month", &self.month)?;
        let require = |field: &str, v: &Option<String>| -> Result<String, String> {
            match v {
                Some(s) if !s.trim().is_empty() => Ok(s.trim().to_string()),
                _ => Err(tx_campo("mcp-valida-obrigatorio-medida", field)),
            }
        };
        let mut params = Map::new();
        let mut put = |k: &str, v: Option<Value>| {
            if let Some(v) = v {
                params.insert(k.to_string(), v);
            }
        };
        let basis = self
            .basis
            .map(|b| serde_json::to_value(b).unwrap_or(Value::Null));
        match self.metric {
            Metric::AccountBalance => {
                put(
                    "account_id",
                    Some(require("account_id", &self.account_id)?.into()),
                );
                put("as_of", self.as_of.clone().map(Value::from));
            }
            Metric::NetWorth => put("as_of", self.as_of.clone().map(Value::from)),
            Metric::Spending | Metric::Income => {
                let from = require("from", &self.from)?;
                let to = require("to", &self.to)?;
                check_range("from", &from, "to", &to)?;
                put("from", Some(from.into()));
                put("to", Some(to.into()));
                put("basis", basis);
                if self.metric == Metric::Spending {
                    put("category_id", self.category_id.clone().map(Value::from));
                    put("member_id", self.member_id.clone().map(Value::from));
                }
            }
            Metric::SavingsRate => put("month", Some(require("month", &self.month)?.into())),
        }
        let mut out = json!({ "metric": self.metric, "params": params });
        if let Some(limit) = self.limit {
            out["limit"] = json!(limit.clamp(1, 500));
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MonthlyFlowInput {
    /// First month `YYYY-MM` (default: 11 months before `to_month`).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "from", alias = "start_month")]
    pub from_month: Option<String>,
    /// Last month `YYYY-MM` (default: current month).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "to", alias = "end_month")]
    pub to_month: Option<String>,
    /// Date basis (default `accrual`).
    pub basis: Option<Basis>,
}

impl ToolInput for MonthlyFlowInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("from_month", &self.from_month)?;
        check_month("to_month", &self.to_month)?;
        let to = self.to_month.clone().unwrap_or_else(current_month);
        let from = self
            .from_month
            .clone()
            .unwrap_or_else(|| shift_month(&to, -11));
        check_range("from_month", &from, "to_month", &to)?;
        if months_between(&from, &to)? > FLOW_MAX_MONTHS {
            return Err(tx_args(
                "mcp-valida-periodo-maximo",
                &[("meses", FLOW_MAX_MONTHS.into())],
            ));
        }
        let mut out = json!({ "from_month": from, "to_month": to });
        if let Some(b) = self.basis {
            out["basis"] = json!(b);
        }
        Ok(out)
    }
}

/// Máximo de meses em `get_spend_groups`.
pub const GROUPS_MAX_MONTHS: i64 = 60;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpendGroupsInput {
    /// First month `YYYY-MM` (default: the same as `to`).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "from_month", alias = "start_month")]
    pub from: Option<String>,
    /// Last month `YYYY-MM` (default: current month).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "to_month", alias = "end_month")]
    pub to: Option<String>,
    /// Date basis (default: the budget setting).
    pub basis: Option<Basis>,
    /// Only what belongs to one family member (id from `list_members`). Default: the family.
    pub member_id: Option<String>,
}

impl ToolInput for SpendGroupsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("from", &self.from)?;
        check_month("to", &self.to)?;
        let to = self.to.clone().unwrap_or_else(current_month);
        let from = self.from.clone().unwrap_or_else(|| to.clone());
        check_range("from", &from, "to", &to)?;
        if months_between(&from, &to)? > GROUPS_MAX_MONTHS {
            return Err(tx_args(
                "mcp-valida-periodo-maximo",
                &[("meses", GROUPS_MAX_MONTHS.into())],
            ));
        }
        let mut out = json!({ "from": from, "to": to });
        if let Some(b) = self.basis {
            out["basis"] = json!(b);
        }
        if let Some(m) = self.member_id.as_deref().filter(|m| !m.trim().is_empty()) {
            out["member_id"] = json!(m);
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListCategoriesInput {
    /// Only income or only expense categories (default: all).
    pub kind: Option<CategoryKind>,
    /// Include archived categories (default: no).
    pub include_archived: Option<bool>,
}

impl ToolInput for ListCategoriesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        // Filtros aplicados aqui, sobre a lista completa.
        Ok(Value::Null)
    }

    fn post_process(&self, value: Value) -> Value {
        let kind = self.kind.map(|k| match k {
            CategoryKind::Income => "income",
            CategoryKind::Expense => "expense",
        });
        let archived = self.include_archived.unwrap_or(false);
        filter_array(value, |c| {
            kind.is_none_or(|k| c.get("kind").and_then(Value::as_str) == Some(k))
                && (archived || c.get("archived").and_then(Value::as_bool) != Some(true))
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListMembersInput {
    /// Include archived members (default: no).
    pub include_archived: Option<bool>,
}

impl ToolInput for ListMembersInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(Value::Null)
    }

    fn post_process(&self, value: Value) -> Value {
        let archived = self.include_archived.unwrap_or(false);
        filter_array(value, |m| {
            archived || m.get("archived").and_then(Value::as_bool) != Some(true)
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DashboardInput {
    /// Month `YYYY-MM` (default: current month).
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: Option<String>,
    /// Only what belongs to one family member (id from `list_members`): transactions where
    /// they are the member, their share of accounts and physical assets, and their debts.
    /// Default: the family.
    pub member_id: Option<String>,
}

impl ToolInput for DashboardInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("month", &self.month)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FinancialHealthInput {
    /// Month assessed `YYYY-MM` (default: the current one, with the window ending the previous
    /// month).
    #[schemars(pattern(MONTH_PATTERN))]
    pub as_of: Option<String>,
    /// Financial health of one family member (id from `list_members`). Default: the family.
    pub member_id: Option<String>,
}

impl ToolInput for FinancialHealthInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("as_of", &self.as_of)?;
        to_object(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_month_crosses_years() {
        assert_eq!(shift_month("2026-03", -11), "2025-04");
        assert_eq!(shift_month("2026-01", -1), "2025-12");
        assert_eq!(shift_month("2025-12", 1), "2026-01");
        assert_eq!(months_between("2025-04", "2026-03").unwrap(), 12);
    }

    #[test]
    fn txn_limit_defaults_and_clamps() {
        let a = QueryTransactionsInput::default().to_api_args().unwrap();
        assert_eq!(a["limit"], 50);
        assert_eq!(a["offset"], 0);
        let big = QueryTransactionsInput {
            limit: Some(10_000),
            ..Default::default()
        };
        assert_eq!(big.to_api_args().unwrap()["limit"], 200);
        let bad = QueryTransactionsInput {
            date_from: Some("2026-13-01".into()),
            ..Default::default()
        };
        assert!(bad.to_api_args().is_err());
        let inverted = QueryTransactionsInput {
            date_from: Some("2026-02-01".into()),
            date_to: Some("2026-01-01".into()),
            ..Default::default()
        };
        assert!(inverted.to_api_args().is_err());
    }

    #[test]
    fn explain_requires_metric_params() {
        let mut e = ExplainValueInput {
            metric: Metric::AccountBalance,
            account_id: None,
            as_of: None,
            from: None,
            to: None,
            basis: None,
            category_id: None,
            member_id: None,
            month: None,
            limit: None,
        };
        assert!(e.to_api_args().is_err());
        e.account_id = Some("abc".into());
        let a = e.to_api_args().unwrap();
        assert_eq!(a["metric"], "account_balance");
        assert_eq!(a["params"]["account_id"], "abc");
        e.metric = Metric::SavingsRate;
        assert!(e.to_api_args().is_err());
        e.month = Some("2026-09".into());
        assert_eq!(
            e.to_api_args().unwrap()["params"],
            json!({"month": "2026-09"})
        );
    }

    #[test]
    fn category_filters_are_local() {
        let input = ListCategoriesInput {
            kind: Some(CategoryKind::Income),
            include_archived: None,
        };
        let out = input.post_process(json!([
            {"id": "1", "kind": "income", "archived": false},
            {"id": "2", "kind": "expense", "archived": false},
            {"id": "3", "kind": "income", "archived": true},
        ]));
        assert_eq!(
            out,
            json!([{"id": "1", "kind": "income", "archived": false}])
        );
    }
}
