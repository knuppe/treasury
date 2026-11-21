//! Entradas das ferramentas de escrita (docs/ai-agent.md §Ação). Toda escrita vira
//! **proposta** pendente (fila de aprovação) que o usuário confirma na tela.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::inputs::{
    DATE_PATTERN, MONTH_PATTERN, ToolInput, TxnKind, TxnStatus, check_date, check_month,
    check_non_empty, to_object, tx, tx_args, tx_campo,
};

fn check_len(field: &str, v: &str, max: usize) -> Result<(), String> {
    if v.chars().count() > max {
        return Err(tx_args(
            "mcp-valida-tamanho",
            &[("campo", field.to_string().into()), ("max", max.into())],
        ));
    }
    Ok(())
}

fn check_decimal(field: &str, v: &str) -> Result<(), String> {
    v.trim()
        .parse::<rust_decimal::Decimal>()
        .map(|_| ())
        .map_err(|_| tx_campo("mcp-valida-decimal", field))
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateTransactionInput {
    /// Account (see `list_accounts`).
    pub account_id: String,
    /// Date `YYYY-MM-DD` (accrual basis; on a credit card, the purchase date).
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Signed amount in cents relative to the account: negative goes out (expense), positive
    /// comes in.
    pub amount: i64,
    /// Short description.
    #[schemars(length(min = 1, max = 200))]
    pub description: String,
    /// Type (default: expense if negative, income if positive).
    pub kind: Option<TxnKind>,
    /// Status (default `cleared`; use `planned` for a planned transaction).
    pub status: Option<TxnStatus>,
    /// Payee.
    pub payee: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Category (see `list_categories`).
    pub category_id: Option<String>,
    /// Member (see `list_members`).
    pub member_id: Option<String>,
    /// Tags by name.
    pub tags: Option<Vec<String>>,
    /// Project (renovation, home improvement, trip…; see `list_projects`).
    pub project_id: Option<String>,
}

impl ToolInput for CreateTransactionInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        check_date("date", &Some(self.date.clone()))?;
        check_non_empty("description", &self.description)?;
        check_len("description", &self.description, 200)?;
        if self.amount == 0 {
            return Err(tx("mcp-valida-valor-zero"));
        }
        to_object(self)
    }
}

/// Field that can be cleared in `update_transaction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ClearableTxnField {
    Payee,
    Notes,
    CategoryId,
    MemberId,
    ProjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateTransactionInput {
    /// Transaction id (see `query_transactions`).
    pub id: String,
    /// New date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: Option<String>,
    /// New signed amount in cents.
    pub amount: Option<i64>,
    /// New description.
    pub description: Option<String>,
    /// New type.
    pub kind: Option<TxnKind>,
    /// New status.
    pub status: Option<TxnStatus>,
    /// New payee.
    pub payee: Option<String>,
    /// New notes.
    pub notes: Option<String>,
    /// New category (see `list_categories`).
    pub category_id: Option<String>,
    /// New member (see `list_members`).
    pub member_id: Option<String>,
    /// Replaces the tags.
    pub tags: Option<Vec<String>>,
    /// The transaction's project (see `list_projects`).
    pub project_id: Option<String>,
    /// Transfer to (or from) another family account (see `list_accounts`): links to the leg
    /// that already exists there (same amount, nearby date) or creates that leg. For third
    /// parties, use `kind: "transfer"` with `payee` and, if applicable, `category_id`.
    pub transfer_account_id: Option<String>,
    /// Amount (cents, positive) in the other account when it uses another currency.
    pub transfer_amount: Option<i64>,
    /// Fields to clear (e.g. `["category_id"]` removes the category).
    pub clear: Option<Vec<ClearableTxnField>>,
}

impl ToolInput for UpdateTransactionInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("date", &self.date)?;
        if let Some(d) = &self.description {
            check_non_empty("description", d)?;
            check_len("description", d, 200)?;
        }
        let mut input = self.clone();
        let clear = input.clear.take().unwrap_or_default();
        let mut v = to_object(&input)?;
        for f in clear {
            let key = match f {
                ClearableTxnField::Payee => "payee",
                ClearableTxnField::Notes => "notes",
                ClearableTxnField::CategoryId => "category_id",
                ClearableTxnField::MemberId => "member_id",
                ClearableTxnField::ProjectId => "project_id",
            };
            v[key] = Value::Null;
        }
        if v.as_object().is_some_and(|m| m.len() <= 1) {
            return Err(tx("mcp-valida-um-campo"));
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CategorizeTransactionsInput {
    /// Transactions to categorize (up to 500), all with the same category.
    #[schemars(length(min = 1, max = 500))]
    pub transaction_ids: Vec<String>,
    /// Category (see `list_categories`).
    pub category_id: String,
}

impl ToolInput for CategorizeTransactionsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.transaction_ids.is_empty() || self.transaction_ids.len() > 500 {
            return Err(tx("mcp-valida-lancamentos-1-500"));
        }
        check_non_empty("category_id", &self.category_id)?;
        Ok(json!({"ids": self.transaction_ids, "category_id": self.category_id}))
    }
}

/// Condition field: `description`, `payee`, `notes`, `amount` (signed cents),
/// `account_id`, `kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum RuleField {
    Description,
    Payee,
    Notes,
    Amount,
    AccountId,
    Kind,
}

/// Operator: text (`contains`, `equals`, `starts_with`, `ends_with`, `regex`) or number
/// (`between` with `value2`, `gte`, `lte`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum RuleOp {
    Contains,
    Equals,
    StartsWith,
    EndsWith,
    Regex,
    Between,
    Gte,
    Lte,
}

/// Text or number, depending on the field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RuleValue {
    Number(i64),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuleConditionInput {
    /// Field compared.
    pub field: RuleField,
    /// Operator.
    pub op: RuleOp,
    /// Value compared (text, or signed cents for `amount`).
    pub value: RuleValue,
    /// Upper bound for `between` (cents).
    pub value2: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuleActionsInput {
    /// Category to apply.
    pub category_id: Option<String>,
    /// Member to apply.
    pub member_id: Option<String>,
    /// Tags to add.
    pub tags: Option<Vec<String>>,
    /// New description.
    pub rename_to: Option<String>,
    /// New type.
    pub kind: Option<TxnKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRuleInput {
    /// Rule name (e.g. "Description contains UBER").
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    /// Conditions (at least one).
    #[schemars(length(min = 1))]
    pub conditions: Vec<RuleConditionInput>,
    /// What the rule does to the matching transactions.
    pub actions: RuleActionsInput,
    /// All conditions (default) or any of them.
    pub match_all: Option<bool>,
    /// Priority (lower runs first; default 100).
    pub priority: Option<i64>,
}

impl ToolInput for CreateRuleInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        if self.conditions.is_empty() {
            return Err(tx("mcp-valida-condicao"));
        }
        let a = &self.actions;
        if a.category_id.is_none()
            && a.member_id.is_none()
            && a.tags.as_ref().is_none_or(Vec::is_empty)
            && a.rename_to.is_none()
            && a.kind.is_none()
        {
            return Err(tx("mcp-valida-acao"));
        }
        let mut v = to_object(self)?;
        v["actions"] = to_object(&self.actions)?;
        v["conditions"] = Value::Array(
            self.conditions
                .iter()
                .map(to_object)
                .collect::<Result<Vec<_>, _>>()?,
        );
        v["active"] = json!(true);
        Ok(v)
    }
}

/// Frequency: `daily`, `weekly`, `monthly`, `yearly`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRecurrenceInput {
    /// Description (e.g. "Rent").
    #[schemars(length(min = 1, max = 200))]
    pub description: String,
    /// Account (see `list_accounts`).
    pub account_id: String,
    /// Signed amount in cents (negative = expense).
    pub amount: i64,
    /// Type of the generated transactions.
    pub kind: TxnKind,
    /// Category (see `list_categories`).
    pub category_id: Option<String>,
    /// Member (see `list_members`).
    pub member_id: Option<String>,
    /// Destination account (recurring transfers).
    pub to_account_id: Option<String>,
    pub freq: Freq,
    /// Every N periods (default 1).
    #[schemars(range(min = 1, max = 365))]
    pub interval: Option<u32>,
    /// Day of the month (monthly/yearly); 31 = last day.
    #[schemars(range(min = 1, max = 31))]
    pub by_month_day: Option<u32>,
    /// Weekdays (weekly): `MO`, `TU`, `WE`, `TH`, `FR`, `SA`, `SU`.
    pub by_day: Option<Vec<String>>,
    /// Month (yearly), 1–12.
    #[schemars(range(min = 1, max = 12))]
    pub by_month: Option<u32>,
    /// Number of occurrences.
    pub count: Option<u32>,
    /// First date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub start_date: String,
    /// Last date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub end_date: Option<String>,
}

impl ToolInput for CreateRecurrenceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("description", &self.description)?;
        check_non_empty("account_id", &self.account_id)?;
        check_date("start_date", &Some(self.start_date.clone()))?;
        check_date("end_date", &self.end_date)?;
        let mut spec = Map::new();
        spec.insert(
            "freq".into(),
            json!(match self.freq {
                Freq::Daily => "DAILY",
                Freq::Weekly => "WEEKLY",
                Freq::Monthly => "MONTHLY",
                Freq::Yearly => "YEARLY",
            }),
        );
        for (k, v) in [
            ("interval", self.interval.map(|v| json!(v))),
            ("by_month_day", self.by_month_day.map(|v| json!(v))),
            ("by_day", self.by_day.clone().map(|v| json!(v))),
            ("by_month", self.by_month.map(|v| json!(v))),
            ("count", self.count.map(|v| json!(v))),
        ] {
            if let Some(v) = v {
                spec.insert(k.into(), v);
            }
        }
        let mut out = json!({
            "description": self.description,
            "account_id": self.account_id,
            "amount": self.amount,
            "kind": self.kind,
            "spec": spec,
            "start_date": self.start_date,
        });
        for (k, v) in [
            ("category_id", &self.category_id),
            ("member_id", &self.member_id),
            ("to_account_id", &self.to_account_id),
            ("end_date", &self.end_date),
        ] {
            if let Some(v) = v {
                out[k] = json!(v);
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BudgetLineInput {
    /// Category (see `list_categories`).
    pub category_id: String,
    /// Budgeted amount in cents, positive; 0 removes the line.
    #[schemars(range(min = 0))]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetBudgetInput {
    /// Month `YYYY-MM`.
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: String,
    /// Lines to set (the others stay as they are).
    #[schemars(length(min = 1, max = 200))]
    pub lines: Vec<BudgetLineInput>,
}

impl ToolInput for SetBudgetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("month", &Some(self.month.clone()))?;
        if self.lines.is_empty() {
            return Err(tx("mcp-valida-uma-linha"));
        }
        if self.lines.iter().any(|l| l.amount < 0) {
            return Err(tx("mcp-valida-orcamento-positivo"));
        }
        to_object(self)
    }
}

/// Goal type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum GoalKind {
    Emergency,
    Travel,
    DebtPayoff,
    Purchase,
    Retirement,
    Education,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateGoalInput {
    /// Goal name.
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    /// Goal type.
    pub kind: GoalKind,
    /// Target amount in cents.
    #[schemars(range(min = 1))]
    pub target_amount: i64,
    /// Currency (default: the base currency).
    pub currency: Option<String>,
    /// Target date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub target_date: Option<String>,
    /// Amount already saved, in cents.
    pub initial_amount: Option<i64>,
    /// Planned monthly contribution, in cents.
    pub monthly_contribution: Option<i64>,
    /// Accounts that make up the goal.
    pub account_ids: Option<Vec<String>>,
    /// Notes.
    pub notes: Option<String>,
    /// Expected real return a year, above inflation (0.03 = 3%; from -0.05 to 0.15). With it,
    /// the target is in today's money and the monthly contribution and the projected date count
    /// the growth; without it, the plain division (remaining ÷ months). The user's assumption,
    /// not a promise: ask before setting it.
    #[schemars(range(min = -0.05, max = 0.15))]
    pub real_return: Option<f64>,
}

impl ToolInput for CreateGoalInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        check_date("target_date", &self.target_date)?;
        if self.target_amount <= 0 {
            return Err(tx_campo("mcp-valida-positivo", "target_amount"));
        }
        to_object(self)
    }
}

/// `buy` buy, `sell` sell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordTradeInput {
    /// Asset (see `list_assets`).
    pub asset_id: String,
    /// Brokerage account.
    pub account_id: String,
    /// Trading date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Buy or sell.
    pub side: TradeSide,
    /// Quantity, decimal as text.
    pub quantity: String,
    /// Unit price in the asset's currency, decimal as text.
    pub price: String,
    /// Fees (brokerage, exchange fees), in cents.
    pub fees: Option<i64>,
    /// Withheld taxes, in cents.
    pub taxes: Option<i64>,
    /// Exchange rate on the day (assets in another currency), decimal as text.
    pub fx_rate: Option<String>,
    /// Day trade (buy and sell on the same day).
    pub day_trade: Option<bool>,
    /// Brokerage note number.
    pub note_number: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Post the movement to the brokerage's cash balance (default: yes).
    pub cash_movement: Option<bool>,
}

impl ToolInput for RecordTradeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_non_empty("account_id", &self.account_id)?;
        check_date("date", &Some(self.date.clone()))?;
        check_decimal("quantity", &self.quantity)?;
        check_decimal("price", &self.price)?;
        if let Some(f) = &self.fx_rate {
            check_decimal("fx_rate", f)?;
        }
        to_object(self)
    }
}

/// Income distribution type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum IncomeKind {
    Dividend,
    Jcp,
    FiiIncome,
    Coupon,
    Interest,
    Rent,
    Amortization,
    Other,
}

/// `announced` announced, `received` received.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum IncomeStatus {
    Announced,
    Received,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordIncomeInput {
    /// Asset (see `list_assets`).
    pub asset_id: String,
    /// Brokerage account that receives it.
    pub account_id: String,
    /// Income distribution type.
    pub kind: IncomeKind,
    /// Payment date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub pay_date: String,
    /// Data-com (cum date, last day to hold for the right) `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub ex_date: Option<String>,
    /// Entitled quantity, decimal as text.
    pub quantity: Option<String>,
    /// Gross amount in cents (asset's currency).
    #[schemars(range(min = 1))]
    pub gross: i64,
    /// Tax withheld in cents (default: from the type's rule).
    pub tax_withheld: Option<i64>,
    /// Exchange rate on the day (assets in another currency), decimal as text.
    pub fx_rate: Option<String>,
    /// Received (default) or only announced.
    pub status: Option<IncomeStatus>,
    /// Post the credit to the brokerage's cash balance (default: yes, when received).
    pub cash_movement: Option<bool>,
}

impl ToolInput for RecordIncomeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_non_empty("account_id", &self.account_id)?;
        check_date("pay_date", &Some(self.pay_date.clone()))?;
        check_date("ex_date", &self.ex_date)?;
        if let Some(q) = &self.quantity {
            check_decimal("quantity", q)?;
        }
        to_object(self)
    }
}

/// `profile` profile and horizon, `family_rule` family rule, `note` agent note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum MemoryKind {
    Profile,
    FamilyRule,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RememberInput {
    /// Memory type.
    pub kind: MemoryKind,
    /// Short, objective content (up to 2,000 characters).
    #[schemars(length(min = 1, max = 2000))]
    pub content: String,
}

impl ToolInput for RememberInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("content", &self.content)?;
        check_len("content", &self.content, 2000)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListAlertsInput {
    /// `new`, `seen` or `resolved` (default: new and seen).
    pub status: Option<String>,
    /// Maximum (default 50).
    #[schemars(range(min = 1, max = 200))]
    pub limit: Option<i64>,
}

impl ToolInput for ListAlertsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(s) = &self.status
            && !matches!(s.as_str(), "new" | "seen" | "resolved")
        {
            return Err(tx("mcp-valida-status"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PendingProposalsInput {
    /// Maximum (default 50).
    #[schemars(range(min = 1, max = 200))]
    pub limit: Option<i64>,
    /// Which proposals (default `pending`). `all` also shows the ones the user already
    /// approved or rejected, with `reviewed_at`: use it to check what became of a proposal.
    pub status: Option<ProposalStatusInput>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatusInput {
    /// Still waiting for the user.
    Pending,
    /// Approved by the user and applied.
    Approved,
    /// Rejected by the user.
    Rejected,
    /// Approval failed because the data changed after the proposal.
    Failed,
    /// Withdrawn on its own: the same data changed before the approval.
    Withdrawn,
    /// Every status, most recent first.
    All,
}

impl ToolInput for PendingProposalsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        let limit = self.limit.unwrap_or(50).clamp(1, 200);
        Ok(match self.status.unwrap_or(ProposalStatusInput::Pending) {
            ProposalStatusInput::All => json!({"limit": limit}),
            s => json!({"status": serde_json::to_value(s).unwrap_or_default(), "limit": limit}),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_inputs_match_core_dtos() {
        let t = CreateTransactionInput {
            account_id: "a".into(),
            date: "2026-10-01".into(),
            amount: -500,
            description: "Padaria".into(),
            kind: Some(TxnKind::Expense),
            status: Some(TxnStatus::Planned),
            payee: Some("P".into()),
            notes: None,
            category_id: Some("c".into()),
            member_id: None,
            tags: Some(vec!["x".into()]),
            project_id: Some("p".into()),
        };
        let _: crate::transactions::TxnInput =
            serde_json::from_value(t.to_api_args().unwrap()).unwrap();
        let u = UpdateTransactionInput {
            id: "t".into(),
            date: None,
            amount: Some(-10),
            description: None,
            kind: None,
            status: None,
            payee: None,
            notes: None,
            category_id: None,
            member_id: None,
            tags: None,
            project_id: None,
            transfer_account_id: None,
            transfer_amount: None,
            clear: Some(vec![ClearableTxnField::CategoryId]),
        };
        let args = u.to_api_args().unwrap();
        assert!(args["category_id"].is_null() && args.get("category_id").is_some());
        let p: crate::transactions::TxnPatch = serde_json::from_value(args).unwrap();
        assert_eq!(p.category_id, Some(None), "null limpa a categoria");
        let r = CreateRuleInput {
            name: "Uber".into(),
            conditions: vec![RuleConditionInput {
                field: RuleField::Description,
                op: RuleOp::Contains,
                value: RuleValue::Text("uber".into()),
                value2: None,
            }],
            actions: RuleActionsInput {
                category_id: Some("c".into()),
                ..Default::default()
            },
            match_all: None,
            priority: None,
        };
        let _: crate::rules::RuleInput = serde_json::from_value(r.to_api_args().unwrap()).unwrap();
        let rec = CreateRecurrenceInput {
            description: "Aluguel".into(),
            account_id: "a".into(),
            amount: -200000,
            kind: TxnKind::Expense,
            category_id: None,
            member_id: None,
            to_account_id: None,
            freq: Freq::Monthly,
            interval: None,
            by_month_day: Some(5),
            by_day: None,
            by_month: None,
            count: None,
            start_date: "2026-11-05".into(),
            end_date: None,
        };
        let dto: crate::recurrence::RecurrenceInput =
            serde_json::from_value(rec.to_api_args().unwrap()).unwrap();
        assert_eq!(dto.spec.by_month_day, Some(5));
        let b = SetBudgetInput {
            month: "2026-11".into(),
            lines: vec![BudgetLineInput {
                category_id: "c".into(),
                amount: 100000,
            }],
        };
        let _: crate::budget::SetBudgetArgs =
            serde_json::from_value(b.to_api_args().unwrap()).unwrap();
        let g = CreateGoalInput {
            name: "Reserva".into(),
            kind: GoalKind::Emergency,
            target_amount: 3_000_000,
            currency: None,
            target_date: Some("2027-12-31".into()),
            initial_amount: None,
            monthly_contribution: Some(100000),
            account_ids: None,
            notes: None,
            real_return: Some(0.04),
        };
        let _: crate::goals::GoalInput = serde_json::from_value(g.to_api_args().unwrap()).unwrap();
        let tr = RecordTradeInput {
            asset_id: "x".into(),
            account_id: "b".into(),
            date: "2026-10-01".into(),
            side: TradeSide::Buy,
            quantity: "10".into(),
            price: "30.5".into(),
            fees: Some(100),
            taxes: None,
            fx_rate: None,
            day_trade: None,
            note_number: None,
            notes: None,
            cash_movement: None,
        };
        let _: crate::portfolio::TradeInput =
            serde_json::from_value(tr.to_api_args().unwrap()).unwrap();
        let inc = RecordIncomeInput {
            asset_id: "x".into(),
            account_id: "b".into(),
            kind: IncomeKind::Jcp,
            pay_date: "2026-10-01".into(),
            ex_date: None,
            quantity: Some("10".into()),
            gross: 500,
            tax_withheld: None,
            fx_rate: None,
            status: Some(IncomeStatus::Received),
            cash_movement: None,
        };
        let _: crate::portfolio::IncomeInput =
            serde_json::from_value(inc.to_api_args().unwrap()).unwrap();
        let m = RememberInput {
            kind: MemoryKind::FamilyRule,
            content: "Aluguel vence dia 5".into(),
        };
        let _: crate::ai::RememberArgs = serde_json::from_value(m.to_api_args().unwrap()).unwrap();
        assert!(
            TradeSide::Sell != TradeSide::Buy
                && RecordTradeInput {
                    quantity: "dez".into(),
                    ..tr
                }
                .to_api_args()
                .is_err()
        );
    }
}

/// Withdraws a pending proposal this client made itself.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WithdrawProposalInput {
    /// Proposal id (see `list_pending_proposals`).
    pub id: String,
    /// Why (shown in the queue history), e.g. "test of an optional field".
    #[schemars(length(max = 200))]
    pub reason: Option<String>,
}

impl ToolInput for WithdrawProposalInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        to_object(self)
    }
}
