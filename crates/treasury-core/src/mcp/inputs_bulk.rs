//! Entradas das ferramentas de cadastro em lote (lançamentos e investimentos lidos de um
//! extrato, anexo ou página) e dos guias de cadastro.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::inputs::{
    DATE_PATTERN, ToolInput, check_date, check_non_empty, to_object, tx, tx_args, tx_campo,
};
use super::inputs_register::{CreateAccountInput, CreateAssetInput};
use super::inputs_write::{IncomeKind, IncomeStatus, TradeSide};

fn check_decimal(field: &str, v: &str) -> Result<(), String> {
    v.trim()
        .replace(',', ".")
        .parse::<rust_decimal::Decimal>()
        .map(|_| ())
        .map_err(|_| tx_campo("mcp-valida-decimal", field))
}

/// Transaction read from the page.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TxnRowInput {
    /// Date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Signed amount in cents: negative = outflow (purchase, payment), positive = inflow.
    pub amount: i64,
    /// Description as it appears on the statement.
    #[schemars(length(min = 1, max = 300))]
    pub description: String,
    /// Payee or merchant, if the page shows it separately.
    pub payee: Option<String>,
    /// The bank's id for the transaction, if the page shows it.
    pub external_id: Option<String>,
    /// Notes (e.g. "installment 2/10").
    pub notes: Option<String>,
    /// Card only: due date `YYYY-MM-DD` of the statement the issuer put this purchase in (as
    /// the card page shows it). Overrides the batch one and the one computed from the closing
    /// day.
    #[schemars(pattern(DATE_PATTERN))]
    pub statement_due_date: Option<String>,
    /// Card only: last 4 digits of the card used (additional or virtual card, as the statement
    /// block shows: "card ending 1234"). Overrides the batch one. Never the full number.
    #[schemars(pattern(r"^\d{4}$"))]
    pub card_last4: Option<String>,
    /// Purchase abroad: positive amount in the original currency, in cents (the statement
    /// line "AMAZON USD 80,00" → 8000).
    pub orig_amount: Option<i64>,
    /// Purchase abroad: original currency, ISO code (`USD`).
    #[schemars(length(min = 3, max = 3))]
    pub orig_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportTransactionsInput {
    /// Account or card (see `list_accounts`).
    pub account_id: String,
    /// Where they came from, e.g. "Itaú statement Sep/2026, internet banking".
    #[schemars(length(min = 1, max = 120))]
    pub source: String,
    /// Transactions (up to 500 per call).
    #[schemars(length(min = 1, max = 500))]
    pub rows: Vec<TxnRowInput>,
    /// Also save the possible duplicates (default: no; only if the user confirms).
    pub include_possible_duplicates: Option<bool>,
    /// Card only: due date `YYYY-MM-DD` of the statement for the whole batch (the statement
    /// shown on the page). Without it, each purchase goes to the statement of its date by the
    /// card's closing day.
    #[schemars(pattern(DATE_PATTERN))]
    pub statement_due_date: Option<String>,
    /// Card only: last 4 digits of the card used for the whole batch.
    #[schemars(pattern(r"^\d{4}$"))]
    pub card_last4: Option<String>,
}

impl ToolInput for ImportTransactionsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        check_non_empty("source", &self.source)?;
        if self.rows.is_empty() || self.rows.len() > 500 {
            return Err(tx("mcp-valida-rows-1-500"));
        }
        check_date("statement_due_date", &self.statement_due_date)?;
        for (i, r) in self.rows.iter().enumerate() {
            check_date(&format!("rows[{i}].date"), &Some(r.date.clone()))?;
            check_date(
                &format!("rows[{i}].statement_due_date"),
                &r.statement_due_date,
            )?;
        }
        to_object(self)
    }
}

/// Trade (buy or sell) read from the page.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BulkTradeInput {
    /// Ticker (PETR4, HGLG11, AAPL) or asset id; can be an asset from `assets`.
    pub asset: String,
    /// Trading date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    pub side: TradeSide,
    /// Quantity, decimal as text.
    pub quantity: String,
    /// Unit price in the asset's currency, decimal as text.
    pub price: String,
    /// Fees in cents.
    pub fees: Option<i64>,
    /// Withheld taxes in cents.
    pub taxes: Option<i64>,
    /// Brokerage note (nota de corretagem) number.
    pub note_number: Option<String>,
    /// The brokerage's id for the trade, if any.
    pub external_id: Option<String>,
    /// Post to the account's cash balance (default: yes).
    pub cash_movement: Option<bool>,
}

/// Quantity and average price of an asset on a date (consolidated portfolio).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PositionItemInput {
    /// Ticker, asset id (from `list_assets`) or the `name` of an asset in `assets` of this
    /// call.
    pub asset: String,
    /// Total quantity on the date, decimal as text. Higher than recorded: the difference
    /// becomes a purchase; lower: in fixed income (bank RDB/CDB, PU 1) the difference becomes a
    /// redemption (sale without cash movement), in other classes it is skipped and you record
    /// the sale with `record_trade`.
    pub quantity: String,
    /// Average price in the asset's currency, decimal as text.
    pub average_price: String,
}

/// Positions shown on the page: an object `{date, items: [{asset, quantity, average_price}]}`,
/// not a list.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PositionsInput {
    /// Position date `YYYY-MM-DD` (the page's date, usually today).
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    pub items: Vec<PositionItemInput>,
}

/// Income distribution read from the page.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BulkIncomeInput {
    /// Ticker or asset id; can be an asset from `assets`.
    pub asset: String,
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
    /// Received (default) or only announced.
    pub status: Option<IncomeStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportInvestmentsInput {
    /// Where they came from, e.g. "XP, Minha carteira, 15/10/2026".
    #[schemars(length(min = 1, max = 120))]
    pub source: String,
    /// Brokerage account already registered (see `list_accounts`)…
    pub account_id: Option<String>,
    /// …or the account to use by name (reuses an existing one) or to create, with the
    /// institution by its exact name on the page (`institution_name`). Brokerage: `brokerage`;
    /// foreign brokerage: `brokerage` with `currency: "USD"`.
    pub account: Option<CreateAccountInput>,
    /// Assets to use or register (no duplicates: reuses by ticker; without a ticker, by name,
    /// official name or a CNPJ only one asset of the class has).
    pub assets: Option<Vec<CreateAssetInput>>,
    /// Trades (buys and sells) with date and price.
    pub trades: Option<Vec<BulkTradeInput>>,
    /// Position on a date, when the page only shows quantity and average price: records only
    /// the difference from what is already recorded, without touching cash.
    pub positions: Option<PositionsInput>,
    /// Income distributions (dividends, JCP, FII distributions, coupons).
    pub income: Option<Vec<BulkIncomeInput>>,
}

impl ToolInput for ImportInvestmentsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("source", &self.source)?;
        if self.account_id.is_some() && self.account.is_some() {
            return Err(tx_args(
                "mcp-valida-um-ou-outro",
                &[("a", "account_id".into()), ("b", "account".into())],
            ));
        }
        let mut v = json!({ "source": self.source.trim() });
        if let Some(id) = &self.account_id {
            check_non_empty("account_id", id)?;
            v["account_id"] = json!(id);
        }
        if let Some(a) = &self.account {
            v["account"] = a.to_api_args()?;
        }
        if let Some(list) = &self.assets {
            let mut out = vec![];
            for a in list {
                out.push(a.to_api_args()?);
            }
            v["assets"] = Value::Array(out);
        }
        if let Some(list) = &self.trades {
            for (i, t) in list.iter().enumerate() {
                check_non_empty(&format!("trades[{i}].asset"), &t.asset)?;
                check_date(&format!("trades[{i}].date"), &Some(t.date.clone()))?;
                check_decimal(&format!("trades[{i}].quantity"), &t.quantity)?;
                check_decimal(&format!("trades[{i}].price"), &t.price)?;
            }
            v["trades"] = to_object(list)?;
        }
        if let Some(p) = &self.positions {
            check_date("positions.date", &Some(p.date.clone()))?;
            for (i, it) in p.items.iter().enumerate() {
                check_non_empty(&format!("positions.items[{i}].asset"), &it.asset)?;
                check_decimal(&format!("positions.items[{i}].quantity"), &it.quantity)?;
                check_decimal(
                    &format!("positions.items[{i}].average_price"),
                    &it.average_price,
                )?;
            }
            v["positions"] = to_object(p)?;
        }
        if let Some(list) = &self.income {
            for (i, it) in list.iter().enumerate() {
                check_date(&format!("income[{i}].pay_date"), &Some(it.pay_date.clone()))?;
                check_date(&format!("income[{i}].ex_date"), &it.ex_date)?;
            }
            v["income"] = to_object(list)?;
        }
        Ok(v)
    }
}

/// Income distribution with only date, ticker and amount received.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuickIncomeRowInput {
    /// Payment date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Ticker of a registered asset (e.g. ITSA4, HGLG11).
    #[schemars(length(min = 1, max = 20))]
    pub ticker: String,
    /// Amount received in cents, in the asset's currency (the net amount credited; the gross
    /// amount is also recognized when it matches the announced distribution).
    #[schemars(range(min = 1))]
    pub amount: i64,
    /// Only if the page states the type and it differs from the announced one.
    pub kind: Option<IncomeKind>,
    /// Only if the user names the account (otherwise: the brokerage holding the position on the
    /// data-com).
    pub account_id: Option<String>,
}

/// Income distributions typed by the user or read from a simple statement.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordIncomeByTickerInput {
    /// One row per payment, up to 500.
    #[schemars(length(min = 1, max = 500))]
    pub rows: Vec<QuickIncomeRowInput>,
}

impl ToolInput for RecordIncomeByTickerInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.rows.is_empty() {
            return Err(tx("mcp-valida-rows-vazio"));
        }
        for (i, r) in self.rows.iter().enumerate() {
            check_date(&format!("rows[{i}].date"), &Some(r.date.clone()))?;
            check_non_empty(&format!("rows[{i}].ticker"), &r.ticker)?;
            if r.amount <= 0 {
                return Err(tx_campo(
                    "mcp-valida-linha-positiva",
                    &format!("rows[{i}].amount"),
                ));
            }
        }
        to_object(self)
    }
}

/// Data entry guide.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlaybookInput {
    /// Topic: `discovery` (how to collect from files, attachments and pages), `investments`
    /// (portfolio, brokerage, positions, trades), `foreign_broker` (foreign brokerage),
    /// `fixed_income` (fixed income and Tesouro), `income` (income distributions),
    /// `bank_statement` (bank statement), `card_bill` (card statement), `assets` (how to
    /// register each class), `corporate_actions` (splits, reverse splits, bonus shares,
    /// mergers), `payslip` (payslips and earned income), `pension` (PGBL, VGBL and pension
    /// funds). No topic: the list.
    pub topic: Option<String>,
}

impl ToolInput for PlaybookInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// A document as Markdown.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DocumentMarkdownInput {
    /// The file (one) in base64 (external MCP clients); prefer `inbox_id` or `path`.
    pub files: Option<Vec<super::inputs_crud::NoteFileInput>>,
    /// File the user attached in the conversation (app agent and CLIs): its order (`"1"`), name
    /// or id.
    pub attachments: Option<Vec<String>>,
    // Lido por `inbox_files::resolve` antes da validação; aqui, só para o esquema.
    /// File in Treasury's inbox (see `list_inbox`): its id.
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub inbox_id: Option<Vec<String>>,
    /// Local path, only inside the allowed folders (Downloads and the inbox folder).
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub path: Option<String>,
    /// Password of a protected PDF; not stored. Ask the user, never guess.
    pub password: Option<String>,
    /// Character to start from (default 0).
    pub offset: Option<u32>,
    /// How many characters (default 20 thousand, up to 40 thousand).
    pub limit: Option<u32>,
}

impl ToolInput for DocumentMarkdownInput {
    fn to_api_args(&self) -> Result<Value, String> {
        let files = self.files.as_deref().unwrap_or_default();
        let [f] = files else {
            return Err(tx("mcp-valida-um-arquivo"));
        };
        let mut v = json!({"filename": f.filename, "content_base64": f.content_base64});
        if let Some(p) = &self.password {
            v["password"] = json!(p);
        }
        if let Some(o) = self.offset {
            v["offset"] = json!(o);
        }
        if let Some(l) = self.limit {
            v["limit"] = json!(l);
        }
        Ok(v)
    }
}

/// Files in the inbox.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InboxListInput {
    /// Also the files already imported or dismissed (default: only pending and unrecognized).
    pub include_done: Option<bool>,
}

impl ToolInput for InboxListInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// Tools for an intent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FindToolsInput {
    /// What you want to do, in your own words and in any language ("pay off a loan",
    /// "quitar empréstimo", "cheque especial", "rebalancear a carteira").
    pub query: String,
    /// How many tools (default 5, max 15).
    #[schemars(range(min = 1, max = 15))]
    pub limit: Option<u32>,
}

impl ToolInput for FindToolsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.query.trim().is_empty() {
            return Err("`query` is empty".into());
        }
        to_object(self)
    }
}
