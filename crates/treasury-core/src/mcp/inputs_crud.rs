//! Entradas do CRUD completo dos agentes: apagar qualquer registro, alterar operações e
//! registrar ou listar eventos corporativos (desdobramento, grupamento, bonificação…).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::inputs::{DATE_PATTERN, ToolInput, check_date, check_non_empty, to_object, tx, tx_args};
use super::inputs_write::TradeSide;

/// What to delete (the target command decides what goes with it; the proposal shows
/// everything).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum DeleteKind {
    Transaction,
    Account,
    Asset,
    Category,
    Rule,
    Recurrence,
    Goal,
    Trade,
    Income,
    CorporateAction,
    Institution,
    Loan,
    Tag,
    PhysicalAsset,
    PhysicalImprovement,
    PhysicalValuation,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteItemInput {
    /// Record type.
    pub kind: DeleteKind,
    /// Record id (from the list tools).
    pub id: String,
    /// Category only: id of the category that receives the deleted one's transactions, budget
    /// and rules.
    pub reassign_to: Option<String>,
    /// Recurring item only: also delete the planned transactions it generated (default: yes).
    pub delete_planned: Option<bool>,
}

impl ToolInput for DeleteItemInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListActionsInput {
    /// Only events for this asset.
    pub asset_id: Option<String>,
}

impl ToolInput for ListActionsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum CorporateActionKind {
    /// Split: `factor` > 1 (2 = each share becomes two).
    Split,
    /// Reverse split: `factor` < 1 (0.1 = ten become one).
    ReverseSplit,
    /// Bonus shares: `factor` = new shares per share (0.1 = 10%), with `unit_cost`.
    Bonus,
    Subscription,
    Merger,
    TickerChange,
    SpinOff,
    /// Capital return: total `amount` in cents reduces the cost.
    CapitalReturn,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordActionInput {
    /// Asset (see `list_assets`).
    pub asset_id: String,
    /// Data-com (cum date) `YYYY-MM-DD`, from which the event applies.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Event type.
    pub kind: CorporateActionKind,
    /// Factor, decimal as text (split, reverse split, bonus shares, merger).
    pub factor: Option<String>,
    /// Assigned unit cost (bonus shares), decimal as text.
    pub unit_cost: Option<String>,
    /// Total amount in cents (capital return).
    pub amount: Option<i64>,
    /// Asset that receives the position (ticker change, merger).
    pub new_asset_id: Option<String>,
    /// Notes (e.g. source of the event).
    pub notes: Option<String>,
}

impl ToolInput for RecordActionInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_date("date", &Some(self.date.clone()))?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateTradeInput {
    /// Trade id (see `list_trades`).
    pub id: String,
    /// New trading date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: Option<String>,
    /// Buy or sell.
    pub side: Option<TradeSide>,
    /// Quantity, decimal as text.
    pub quantity: Option<String>,
    /// Unit price, decimal as text.
    pub price: Option<String>,
    /// Fees in cents.
    pub fees: Option<i64>,
    /// Taxes in cents.
    pub taxes: Option<i64>,
    /// Brokerage note number.
    pub note_number: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// `true` creates the linked transaction in the account's cash (when the trade has
    /// none), `false` removes it. Omit to keep it as is. Before `true`, check that the
    /// brokerage statement did not already bring that debit or credit (it would count twice).
    pub cash_movement: Option<bool>,
    /// `true` marks the trade as a position adjustment (no money moved: out of the sales
    /// ledger, no cash effect), `false` makes it a real trade again. A trade with a cash entry
    /// needs `cash_movement: false` in the same call.
    pub adjustment: Option<bool>,
}

impl ToolInput for UpdateTradeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("date", &self.date)?;
        to_object(self)
    }
}

/// File sent in base64 by an external MCP client, with no conversation. With the file
/// attached in the conversation, use `attachments` (e.g. `["1"]`) instead of `files`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoteFileInput {
    /// File name (e.g. "nota-2026-10-15.pdf").
    pub filename: String,
    /// File content in base64 (the PDF bytes, not the text).
    pub content_base64: String,
}

/// Asset chosen for a security description on the note (not recognized or recognized
/// wrongly): a registered asset (`asset_id`) or a code to register (`ticker`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoteAssetChoiceInput {
    /// Security description as it came in the preview (the trade's `spec`).
    pub spec: String,
    /// Registered asset (see `list_assets`).
    pub asset_id: Option<String>,
    /// Trading code to use when the asset is not registered yet (e.g. "GGRC11").
    pub ticker: Option<String>,
    /// Class of the asset to register (stock_br, fii, etf_br, bdr…); if omitted, it is inferred.
    pub class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BrokerageNotesInput {
    /// Brokerage account of the notes (see `list_accounts`).
    pub account_id: String,
    /// PDFs of the notes (external MCP clients).
    pub files: Option<Vec<NoteFileInput>>,
    /// File the user attached in the conversation (app agent and CLIs): its order (`"1"`), name
    /// or the id from `read_attachment`. Treasury reads the content; do not repeat the bytes or
    /// the text. E.g. `["1"]`. With nothing in the call, a preview uses the conversation's only
    /// attachment; an import needs it named.
    pub attachments: Option<Vec<String>>,
    /// File in Treasury's inbox (see `list_inbox`), instead of the bytes: an external agent
    /// saves the file in the inbox folder (default `~/Treasury/Inbox`) and passes its id here.
    /// E.g. `["01JB…"]`.
    // Lido por `inbox_files::resolve` antes da validação; aqui, só para o esquema.
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub inbox_id: Option<Vec<String>>,
    /// Local path of the file, instead of the bytes; only inside the allowed folders (Downloads
    /// and the inbox folder). Prefer the inbox (`inbox_id`).
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub path: Option<String>,
    /// Password of a protected PDF (many brokerages use the first 3 digits of the CPF); not
    /// stored. Ask the user, never guess.
    pub password: Option<String>,
    /// Assets chosen for the unrecognized or corrected descriptions (in the preview, to check;
    /// on import, to save).
    pub assets: Option<Vec<NoteAssetChoiceInput>>,
    /// Merge executions of the same order into one trade at the average price (default: yes).
    /// Use the same value in the preview and the import, because `skip` uses the preview's indices.
    pub group: Option<bool>,
    /// Preview indices that must not be saved (import only).
    pub skip: Option<Vec<u32>>,
    /// Post the movement to the brokerage's cash balance (default: no).
    pub cash_movement: Option<bool>,
}

impl ToolInput for BrokerageNotesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        if self.attachments.is_some() {
            return Err(tx("mcp-valida-anexo-nao-encontrado"));
        }
        let files = self.files.as_deref().unwrap_or_default();
        if files.is_empty() {
            return Err(tx("mcp-valida-notas-files"));
        }
        if files.len() > 50 {
            return Err(tx("mcp-valida-notas-max"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StatementFileInput {
    /// Account or card of the statement (see `list_accounts`). Can be omitted for OFX: the
    /// account is matched by the registered bank, branch and number (or by the card's last digits).
    pub account_id: Option<String>,
    /// The OFX or CSV file (external MCP clients; one per call).
    pub files: Option<Vec<NoteFileInput>>,
    /// File the user attached in the conversation (app agent and CLIs): its order (`"1"`), name
    /// or the id from `read_attachment`. Treasury reads the content; do not repeat the bytes or
    /// the text. E.g. `["1"]`. With nothing in the call, a preview uses the conversation's only
    /// attachment; an import needs it named.
    pub attachments: Option<Vec<String>>,
    /// File in Treasury's inbox (see `list_inbox`), instead of the bytes: an external agent
    /// saves the file in the inbox folder (default `~/Treasury/Inbox`) and passes its id here.
    /// E.g. `["01JB…"]`.
    // Lido por `inbox_files::resolve` antes da validação; aqui, só para o esquema.
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub inbox_id: Option<Vec<String>>,
    /// Local path of the file, instead of the bytes; only inside the allowed folders (Downloads
    /// and the inbox folder). Prefer the inbox (`inbox_id`).
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub path: Option<String>,
    /// CSV: bank preset (default: detect from the header).
    pub preset_id: Option<String>,
    /// OFX with several statements: `ACCTID` of the statement to import.
    pub ofx_account: Option<String>,
    /// Also save the possible duplicates (default: no; only if the user confirms).
    pub include_possible_duplicates: Option<bool>,
    /// Password-protected PDF card statement: the password the user gave in the conversation.
    pub password: Option<String>,
}

impl ToolInput for StatementFileInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(a) = &self.account_id {
            check_non_empty("account_id", a)?;
        }
        if self.attachments.is_some() {
            return Err(tx("mcp-valida-anexo-nao-encontrado"));
        }
        let [f] = self.files.as_deref().unwrap_or_default() else {
            return Err(tx("mcp-valida-um-arquivo"));
        };
        let mut v = serde_json::json!({
            "filename": f.filename,
            "content_base64": f.content_base64,
        });
        if let Some(a) = &self.account_id {
            v["account_id"] = a.clone().into();
        }
        if let Some(p) = &self.preset_id {
            v["preset_id"] = p.clone().into();
        }
        if let Some(a) = &self.ofx_account {
            v["ofx_account"] = a.clone().into();
        }
        if let Some(i) = self.include_possible_duplicates {
            v["include_possible_duplicates"] = i.into();
        }
        if let Some(p) = &self.password {
            v["password"] = p.clone().into();
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetStatementInput {
    /// Asset (see `list_assets`).
    pub asset_id: String,
    /// Only this brokerage account.
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapitalShareGetInput {
    /// Checking account at the credit union (cooperativa) (see `list_accounts`).
    pub account_id: String,
}

impl ToolInput for CapitalShareGetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapitalBalanceToolInput {
    /// Credit union checking account with capital shares enabled.
    pub account_id: String,
    /// Balance date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Balance of the capital shares (cotas capital) on the date, in cents (from the credit
    /// union's statement or app).
    #[schemars(range(min = 0))]
    pub balance: i64,
    /// Where it came from (e.g. "2025 annual report").
    pub note: Option<String>,
}

impl ToolInput for CapitalBalanceToolInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        check_date("date", &Some(self.date.clone()))?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FxReferenceInput {
    /// Foreign currency (ISO 4217, e.g. USD, EUR).
    #[schemars(length(min = 3, max = 3))]
    pub currency: String,
    /// Date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub date: Option<String>,
}

impl ToolInput for FxReferenceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("date", &self.date)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExchangeInput {
    /// Account the money leaves (see `list_accounts`).
    pub from_account_id: String,
    /// Receiving account, in another currency.
    pub to_account_id: String,
    /// Exchange date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Amount sent, before charges, in cents of the source currency. Give this or `amount_to`.
    pub amount_from: Option<i64>,
    /// Amount received, in cents of the destination currency.
    pub amount_to: Option<i64>,
    /// Applied rate ("1 foreign unit = X in the family's base currency"; between two foreign
    /// currencies, "1 source unit = X destination"), as on the exchange note.
    pub rate: Option<String>,
    /// Reference rate (see `get_fx_reference_rate`).
    pub reference_rate: Option<String>,
    /// Source of the reference (e.g. bcb_ptax).
    pub reference_source: Option<String>,
    /// Spread over the reference, in %, when there is no applied rate.
    pub spread_pct: Option<String>,
    /// IOF (Brazil) in % of the amount in the base currency, as on the note.
    pub iof_pct: Option<String>,
    /// Flat fee in cents of the source currency.
    pub fee: Option<i64>,
    /// Notes (e.g. exchange contract number).
    pub notes: Option<String>,
}

impl ToolInput for ExchangeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("from_account_id", &self.from_account_id)?;
        check_non_empty("to_account_id", &self.to_account_id)?;
        check_date("date", &Some(self.date.clone()))?;
        if self.amount_from.is_none() && self.amount_to.is_none() {
            return Err(tx_args(
                "mcp-valida-informe-um",
                &[("a", "amount_from".into()), ("b", "amount_to".into())],
            ));
        }
        if self.rate.is_none() && self.reference_rate.is_none() {
            return Err(tx_args(
                "mcp-valida-informe-um",
                &[("a", "rate".into()), ("b", "reference_rate".into())],
            ));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BalanceSyncListInput {
    /// Only this account (see `list_accounts`).
    pub account_id: Option<String>,
    /// Only differences that still have no reason.
    pub open_only: Option<bool>,
}

impl ToolInput for BalanceSyncListInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BalanceSyncIdInput {
    /// Balance sync (see `list_balance_syncs`).
    pub id: String,
}

impl ToolInput for BalanceSyncIdInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SyncBalanceInput {
    /// Account (see `list_accounts`).
    pub account_id: String,
    /// Date of the balance the bank shows, `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Bank balance on that date, in cents (negative if overdrawn).
    pub balance: i64,
    /// Where the balance came from (e.g. "September PDF statement").
    pub note: Option<String>,
}

impl ToolInput for SyncBalanceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("account_id", &self.account_id)?;
        check_date("date", &Some(self.date.clone()))?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResolveSyncStatus {
    /// Reason found (`explanation` required).
    Explained,
    /// The user accepted the difference.
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolveSyncInput {
    /// Balance sync (see `list_balance_syncs`).
    pub id: String,
    /// `explained` (with the reason) or `ignored`.
    pub status: ResolveSyncStatus,
    /// The reason for the difference, in one or two sentences (cite the transactions).
    pub explanation: Option<String>,
}

impl ToolInput for ResolveSyncInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        if self.status == ResolveSyncStatus::Explained
            && self
                .explanation
                .as_deref()
                .is_none_or(|e| e.trim().is_empty())
        {
            return Err(tx("mcp-valida-explicacao"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CrossAccountDuplicatesInput {
    /// Only the copies recorded in this account (see `list_accounts`).
    pub account_id: Option<String>,
    /// Maximum number of pairs (default: all).
    #[schemars(range(min = 1, max = 5000))]
    pub limit: Option<i64>,
}

impl ToolInput for CrossAccountDuplicatesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransferCandidatesInput {
    /// Only transactions from the last N days (default 180).
    #[schemars(range(min = 1, max = 3650))]
    pub days: Option<u32>,
    /// Maximum number of pairs (default 200).
    #[schemars(range(min = 1, max = 1000))]
    pub limit: Option<u32>,
}

impl ToolInput for TransferCandidatesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransferPairInput {
    /// Outgoing transaction (negative amount), from `list_transfer_candidates`.
    pub from_id: String,
    /// Incoming transaction (positive amount), in another account.
    pub to_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkTransfersInput {
    /// Pairs to link as transfers between the family's own accounts.
    #[schemars(length(min = 1, max = 200))]
    pub pairs: Vec<TransferPairInput>,
}

impl ToolInput for LinkTransfersInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.pairs.is_empty() {
            return Err(tx("mcp-valida-par"));
        }
        for p in &self.pairs {
            check_non_empty("from_id", &p.from_id)?;
            check_non_empty("to_id", &p.to_id)?;
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnlinkTransferInput {
    /// One leg of the linked transfer (the other leg is unlinked too).
    pub txn_id: String,
}

impl ToolInput for UnlinkTransferInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("txn_id", &self.txn_id)?;
        to_object(self)
    }
}

/// Transaction whose linked operation to look up (or unlink).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationTxnInput {
    /// Transaction id (from `query_transactions`).
    pub txn_id: String,
}

impl ToolInput for OperationTxnInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("txn_id", &self.txn_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OperationKindInput {
    /// Portfolio trade (buy or sell of an asset: stock, fund, CDB, pension plan…).
    Trade,
    /// Credit union capital shares (the id is the credit union account).
    CapitalShare,
    /// Goal or scheduled savings plan.
    Goal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkOperationInput {
    /// Id of the transaction that paid for (or received) the operation.
    pub txn_id: String,
    /// Operation type.
    pub kind: OperationKindInput,
    /// Operation id (`target_id` from `list_operation_candidates`).
    pub target_id: String,
    /// Short note (optional).
    pub note: Option<String>,
}

impl ToolInput for LinkOperationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("txn_id", &self.txn_id)?;
        check_non_empty("target_id", &self.target_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FundamentusInput {
    /// B3 ticker (e.g. "ITSA4", "HGLG11").
    #[schemars(length(min = 4, max = 11))]
    pub ticker: String,
    /// Fetch from the site again even if the cache is fresh (only if the user asks).
    pub refresh: Option<bool>,
}

impl ToolInput for FundamentusInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("ticker", &self.ticker)?;
        to_object(self)
    }
}

impl ToolInput for AssetStatementInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}
