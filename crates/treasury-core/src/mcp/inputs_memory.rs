//! Entradas das ferramentas da memória de procedimentos do agente (`recall_procedures`,
//! `remember_procedure`, `record_procedure_outcome`). Ver `ai::procedures`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::inputs::{ToolInput, check_non_empty, to_object, tx_args};

/// Topic: `statement` account statement, `card_bill` card statement, `investments`
/// portfolio (positions, trades, brokerage notes, B3 reports), `income` income
/// distributions, `prices` asset quotes and indicators, `tax` income tax (income reports,
/// tax calculation), `registration` data entry, `other` other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ProcedureTopic {
    Statement,
    CardBill,
    Investments,
    Income,
    Prices,
    Tax,
    Registration,
    Other,
}

/// `worked` it worked, `failed` it failed (path changed, different file…).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ProcedureOutcome {
    Worked,
    Failed,
}

fn check_max(field: &str, v: &Option<String>, max: usize) -> Result<(), String> {
    if v.as_ref().is_some_and(|s| s.chars().count() > max) {
        return Err(tx_args(
            "mcp-valida-tamanho",
            &[("campo", field.to_string().into()), ("max", max.into())],
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecallProceduresInput {
    /// Words to search in the title, steps, site and linked names (e.g.
    /// "itau statement").
    #[schemars(length(max = 200))]
    pub query: Option<String>,
    /// Only this topic.
    pub topic: Option<ProcedureTopic>,
    /// Only those linked to this account (see `list_accounts`).
    pub account_id: Option<String>,
    /// Only those linked to this institution (see `list_institutions`).
    pub institution_id: Option<String>,
    /// Only those linked to this asset (see `list_assets`).
    pub asset_id: Option<String>,
    /// Only this site (domain; also matches subdomains).
    pub site: Option<String>,
    /// Maximum (default 5).
    #[schemars(range(min = 1, max = 20))]
    pub limit: Option<i64>,
}

impl ToolInput for RecallProceduresInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_max("query", &self.query, 200)?;
        let mut v = to_object(self)?;
        v["limit"] = Value::from(self.limit.unwrap_or(5).clamp(1, 20));
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RememberProcedureInput {
    /// Id of an existing note to update (from `recall_procedures`); without an id, creates one
    /// (or updates the one with the same topic, title and links).
    pub id: Option<String>,
    pub topic: ProcedureTopic,
    /// Short title (e.g. "OFX statement for the Itaú checking account").
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    /// Short steps: where to sign in, menus, filters, export button, what the file contains
    /// and caveats. No passwords, codes, account numbers or documents.
    #[schemars(length(min = 1, max = 2000))]
    pub steps: String,
    /// Site domain (e.g. `www.itau.com.br`), without path or parameters.
    pub site: Option<String>,
    /// Format of the downloaded file (OFX, CSV, PDF, XLSX…).
    #[schemars(length(max = 20))]
    pub file_format: Option<String>,
    /// Linked institution (see `list_institutions`); taken from the account if omitted.
    pub institution_id: Option<String>,
    /// Linked account (see `list_accounts`).
    pub account_id: Option<String>,
    /// Linked asset (see `list_assets`).
    pub asset_id: Option<String>,
}

impl ToolInput for RememberProcedureInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("title", &self.title)?;
        check_non_empty("steps", &self.steps)?;
        check_max("title", &Some(self.title.clone()), 120)?;
        check_max("steps", &Some(self.steps.clone()), 2000)?;
        check_max("file_format", &self.file_format, 20)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProcedureOutcomeInput {
    /// Note id (from `recall_procedures` or the message's notes block).
    pub id: String,
    pub outcome: ProcedureOutcome,
    /// What changed or went wrong, briefly (e.g. "menu is now Extratos › Exportar").
    #[schemars(length(max = 300))]
    pub note: Option<String>,
}

impl ToolInput for ProcedureOutcomeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_max("note", &self.note, 300)?;
        to_object(self)
    }
}

/// Re-read this conversation's history.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadChatHistoryInput {
    /// Text to search for in the messages (case- and accent-insensitive).
    #[schemars(length(max = 200))]
    pub query: Option<String>,
    /// First message by number (`#`).
    #[schemars(range(min = 1))]
    pub from_seq: Option<i64>,
    /// Last message by number (`#`; same as `from_seq` to read a single one in full).
    #[schemars(range(min = 1))]
    pub to_seq: Option<i64>,
    /// Maximum number of messages (default 20, up to 50).
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<i64>,
}

impl ToolInput for ReadChatHistoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_max("query", &self.query, 200)?;
        to_object(self)
    }
}

/// Read an attachment of this conversation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadAttachmentInput {
    /// Attachment id (from the references in the history) or the file name; empty: list.
    #[schemars(length(max = 200))]
    pub id: Option<String>,
    /// Starting character (default 0; use `next_offset` from the previous read).
    #[schemars(range(min = 0))]
    pub offset: Option<usize>,
    /// How many characters (default 20,000, up to 40,000).
    #[schemars(range(min = 500, max = 40000))]
    pub limit: Option<usize>,
}

impl ToolInput for ReadAttachmentInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_max("id", &self.id, 200)?;
        to_object(self)
    }
}

/// Search the manager memory through the link graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecallMemoryInput {
    /// Topic: names of accounts, assets or people, or words (e.g. "nubank card statement").
    #[schemars(length(max = 300))]
    pub query: Option<String>,
    /// Ids of an account, asset, member, category, goal or institution.
    pub entity_ids: Option<Vec<String>>,
    /// Maximum number of notes (default 8, up to 30).
    #[schemars(range(min = 1, max = 30))]
    pub limit: Option<usize>,
}

impl ToolInput for RecallMemoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_max("query", &self.query, 300)?;
        to_object(self)
    }
}
