//! Entradas das ferramentas do feedback ao desenvolvedor: listar (lista ou quadro), ver,
//! adicionar, reforçar, resolver, editar, mover, priorizar, vincular, apagar e exportar em
//! Markdown. Ver `devfeedback`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::inputs::{ToolInput, check_non_empty, to_object, tx_args};

/// Part of Treasury: `agente` (conversation, prompt, memory), `mcp` (tools and their
/// descriptions), `importacao` (statements, card statements, brokerage notes, B3),
/// `lancamentos` (transactions, categories, rules, budget), `carteira` (investments, assets,
/// IR), `ui` (screens), `dados` (market data, indices, exchange rates, sources), `outro`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum DevFeedbackArea {
    Agente,
    Mcp,
    Importacao,
    Lancamentos,
    Carteira,
    Ui,
    Dados,
    Outro,
}

/// `bug` error or wrong behavior, `melhoria` something that would save steps,
/// `ferramenta_faltando` a tool that was missing to fulfill the request, `dados` confusing,
/// inconsistent or missing data in a result, `documentacao` a tool description or guide that
/// was missing or misleading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum DevFeedbackKind {
    Bug,
    Melhoria,
    FerramentaFaltando,
    Dados,
    Documentacao,
}

/// `aberto`, `em_andamento`, `resolvido` or `descartado`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum DevFeedbackStatus {
    Aberto,
    EmAndamento,
    Resolvido,
    Descartado,
}

/// `baixo`, `medio` or `alto`: how much the problem weighs for the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum DevFeedbackImpact {
    Baixo,
    Medio,
    Alto,
}

/// `duplicado`: this item duplicates the other (merged into it: its reinforcements move to
/// the other and this one is discarded); `depende_de`: this item depends on the other;
/// `relacionado`: related.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum DevFeedbackLinkKind {
    Duplicado,
    DependeDe,
    Relacionado,
}

/// A tool call that led to the problem (no amounts, names or personal data).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DevFeedbackToolCall {
    /// Tool name (e.g. `get_monthly_flow`).
    #[schemars(length(min = 1, max = 120))]
    pub tool: String,
    /// Parameters, summarized and without values.
    #[schemars(length(max = 1000))]
    pub params: Option<String>,
    /// Result or error, summarized.
    #[schemars(length(max = 1000))]
    pub outcome: Option<String>,
}

fn check_max(field: &str, v: &str, max: usize) -> Result<(), String> {
    if v.chars().count() > max {
        return Err(tx_args(
            "mcp-valida-tamanho",
            &[("campo", field.to_string().into()), ("max", max.into())],
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListDevFeedbackInput {
    /// Words that must appear in the title or body (e.g. "card statement").
    #[schemars(length(max = 200))]
    pub query: Option<String>,
    /// Only open and in-progress items (default: true).
    pub open_only: Option<bool>,
    /// Only this status (turns off `open_only`).
    pub status: Option<DevFeedbackStatus>,
    /// Only this area.
    pub area: Option<DevFeedbackArea>,
    /// Only this kind.
    pub kind: Option<DevFeedbackKind>,
    /// Only items with at least this priority (importance).
    #[schemars(range(min = 1, max = 1000))]
    pub min_priority: Option<i64>,
    /// Only items with at most this priority (importance).
    #[schemars(range(min = 1, max = 1000))]
    pub max_priority: Option<i64>,
    /// Only items assigned to this (e.g. `agente-cli`, `usuario`).
    #[schemars(length(max = 40))]
    pub assignee: Option<String>,
    /// Only items with this label.
    #[schemars(length(max = 40))]
    pub label: Option<String>,
    /// Only items resolved or discarded since this instant (RFC 3339): what was closed since
    /// your last check.
    #[schemars(length(max = 40))]
    pub closed_since: Option<String>,
    /// `status`: returns the board (one column per status, in queue order) instead of the
    /// list; all statuses unless `status`/`open_only` say otherwise.
    pub group_by: Option<DevFeedbackGroupBy>,
    /// Maximum (default 20; 100 with `group_by`).
    #[schemars(range(min = 1, max = 200))]
    pub limit: Option<i64>,
}

/// `status`: the board, grouped by status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum DevFeedbackGroupBy {
    Status,
}

impl ToolInput for ListDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(q) = &self.query {
            check_max("query", q, 200)?;
        }
        let mut v = to_object(self)?;
        let board = self.group_by.is_some();
        let closed = self.closed_since.is_some();
        v["open_only"] =
            Value::from(self.status.is_none() && !closed && self.open_only.unwrap_or(!board));
        let limit = self.limit.unwrap_or(if board { 100 } else { 20 });
        v["limit"] = Value::from(limit.clamp(1, 200));
        if let Some(obj) = v.as_object_mut() {
            if let Some(p) = obj.remove("min_priority") {
                obj.insert("min_importance".into(), p);
            }
            if let Some(p) = obj.remove("max_priority") {
                obj.insert("max_importance".into(), p);
            }
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetDevFeedbackInput {
    /// Feedback id (from `list_dev_feedback`).
    pub id: String,
}

impl ToolInput for GetDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        to_object(self)
    }
}

/// Edits a feedback item; absent fields stay as they are, an empty text clears the field.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateDevFeedbackInput {
    /// Feedback id (from `list_dev_feedback`).
    pub id: String,
    /// New title.
    #[schemars(length(min = 1, max = 160))]
    pub title: Option<String>,
    /// The problem (free text).
    #[schemars(length(min = 1, max = 4000))]
    pub body: Option<String>,
    /// New area.
    pub area: Option<DevFeedbackArea>,
    /// New kind.
    pub kind: Option<DevFeedbackKind>,
    /// Priority (importance), 1 to 1000.
    #[schemars(range(min = 1, max = 1000))]
    pub priority: Option<i64>,
    /// What to do.
    #[schemars(length(max = 2000))]
    pub proposal: Option<String>,
    /// How to know it is fixed (suggested test).
    #[schemars(length(max = 2000))]
    pub acceptance_criteria: Option<String>,
    /// Files/lines to look at (`crates/treasury-core/src/x.rs:120`); replaces the list.
    pub code_pointers: Option<Vec<String>>,
    /// Free labels; replaces the list.
    pub labels: Option<Vec<String>>,
    /// Who takes care of it (e.g. `agente-cli`, `usuario`).
    #[schemars(length(max = 40))]
    pub assignee: Option<String>,
    /// What was expected.
    #[schemars(length(max = 2000))]
    pub expected: Option<String>,
    /// What happened instead.
    #[schemars(length(max = 2000))]
    pub actual: Option<String>,
    /// How to reproduce, step by step.
    #[schemars(length(max = 2000))]
    pub repro_steps: Option<String>,
    /// How much it weighs for the user.
    pub user_impact: Option<DevFeedbackImpact>,
    /// The user confirmed the fact.
    pub confirmed_by_user: Option<bool>,
}

impl ToolInput for UpdateDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        let mut v = to_object(self)?;
        if let Some(obj) = v.as_object_mut()
            && let Some(p) = obj.remove("priority")
        {
            obj.insert("importance".into(), p);
        }
        Ok(v)
    }
}

/// Moves a feedback item to another column of the board.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetDevFeedbackStatusInput {
    /// Feedback id (from `list_dev_feedback`).
    pub id: String,
    /// The new column.
    pub status: DevFeedbackStatus,
    /// Why (goes to the item's timeline).
    #[schemars(length(max = 1000))]
    pub reason: Option<String>,
    /// Place in the new column (1 = top); without it, by priority.
    #[schemars(range(min = 1))]
    pub position: Option<i64>,
}

impl ToolInput for SetDevFeedbackStatusInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        if let Some(r) = &self.reason {
            check_max("reason", r, 1000)?;
        }
        to_object(self)
    }
}

/// Priority (importance) and/or manual place in the column.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetDevFeedbackPriorityInput {
    /// Feedback id (from `list_dev_feedback`).
    pub id: String,
    /// New priority (importance), 1 to 1000.
    #[schemars(range(min = 1, max = 1000))]
    pub priority: Option<i64>,
    /// Place in the item's column (1 = top), independent of reinforcements.
    #[schemars(range(min = 1))]
    pub position: Option<i64>,
}

impl ToolInput for SetDevFeedbackPriorityInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        if self.priority.is_none() && self.position.is_none() {
            return Err(tx_args(
                "mcp-valida-informe-um",
                &[("a", "priority".into()), ("b", "position".into())],
            ));
        }
        Ok(serde_json::json!({
            "id": self.id,
            "importance": self.priority,
            "position": self.position,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkDevFeedbackInput {
    /// Feedback id (from `list_dev_feedback`).
    pub id: String,
    /// The other feedback item (with `duplicado`, the main one that stays).
    pub related_id: String,
    /// What the link means, read from `id`.
    pub kind: DevFeedbackLinkKind,
}

impl ToolInput for LinkDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_non_empty("related_id", &self.related_id)?;
        to_object(self)
    }
}

/// Markdown for a coding agent: one item (`id`) or the ones matching the filter.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportDevFeedbackInput {
    /// One feedback id; without it, the filter below (open items by default).
    pub id: Option<String>,
    /// Only this status (default: open and in progress).
    pub status: Option<DevFeedbackStatus>,
    /// Only this area.
    pub area: Option<DevFeedbackArea>,
    /// Only this kind.
    pub kind: Option<DevFeedbackKind>,
    /// Only items with at least this priority.
    #[schemars(range(min = 1, max = 1000))]
    pub min_priority: Option<i64>,
    /// Only items assigned to this.
    #[schemars(length(max = 40))]
    pub assignee: Option<String>,
    /// Maximum items (default 10).
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<i64>,
}

impl ToolInput for ExportDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        let mut v = to_object(self)?;
        if let Some(obj) = v.as_object_mut()
            && let Some(p) = obj.remove("min_priority")
        {
            obj.insert("min_importance".into(), p);
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddDevFeedbackInput {
    /// Short, specific title (e.g. "Missing tool to move a transaction to another card statement").
    #[schemars(length(min = 1, max = 160))]
    pub title: String,
    /// What you tried, what happened (tool, parameters, error) and what would help.
    /// No amounts, names or personal data of the family.
    #[schemars(length(min = 1, max = 4000))]
    pub body: String,
    pub area: DevFeedbackArea,
    pub kind: DevFeedbackKind,
    /// What to do (optional).
    #[schemars(length(max = 2000))]
    pub proposal: Option<String>,
    /// How to know it is fixed: a suggested test.
    #[schemars(length(max = 2000))]
    pub acceptance_criteria: Option<String>,
    /// Suspect files/lines (`crates/treasury-core/src/x.rs:120`).
    pub code_pointers: Option<Vec<String>>,
    /// Free labels.
    pub labels: Option<Vec<String>>,
    /// Who should take care of it (e.g. `agente-cli`, `usuario`).
    #[schemars(length(max = 40))]
    pub assignee: Option<String>,
    /// What you expected, briefly.
    #[schemars(length(max = 2000))]
    pub expected: Option<String>,
    /// What you got instead, briefly.
    #[schemars(length(max = 2000))]
    pub actual: Option<String>,
    /// How to reproduce, step by step (tools, not data).
    #[schemars(length(max = 2000))]
    pub repro_steps: Option<String>,
    /// How much it weighs for the user.
    pub user_impact: Option<DevFeedbackImpact>,
    /// The user confirmed the fact.
    pub confirmed_by_user: Option<bool>,
    /// Your client and version (e.g. "my-cli 2.1").
    #[schemars(length(max = 120))]
    pub client_version: Option<String>,
    /// The calls that led to the problem (no values).
    pub tool_calls: Option<Vec<DevFeedbackToolCall>>,
    /// Related feedback ids.
    pub related_ids: Option<Vec<String>>,
}

impl ToolInput for AddDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("title", &self.title)?;
        check_non_empty("body", &self.body)?;
        check_max("title", &self.title, 160)?;
        check_max("body", &self.body, 4000)?;
        let mut v = to_object(self)?;
        // Campos opcionais ausentes não vão (o núcleo trata `null` e ausência igual, mas a
        // entrada fica limpa no registro da chamada).
        if let Some(obj) = v.as_object_mut() {
            obj.retain(|_, x| !x.is_null());
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BumpDevFeedbackInput {
    /// Feedback id (from `list_dev_feedback`).
    pub id: String,
    /// The new case or what changed, briefly.
    #[schemars(length(max = 1000))]
    pub note: Option<String>,
}

impl ToolInput for BumpDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        if let Some(n) = &self.note {
            check_max("note", n, 1000)?;
        }
        to_object(self)
    }
}

/// Marks a feedback item as resolved, with the reason.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolveDevFeedbackInput {
    /// Feedback id (from `list_dev_feedback`).
    pub id: String,
    /// What fixed it and where (version, commit or change), briefly. Required.
    #[schemars(length(min = 1, max = 1000))]
    pub reason: String,
}

impl ToolInput for ResolveDevFeedbackInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_non_empty("reason", &self.reason)?;
        check_max("reason", &self.reason, 1000)?;
        to_object(self)
    }
}
