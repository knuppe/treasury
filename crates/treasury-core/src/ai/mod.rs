//! IA (fase 4): fila de aprovação das propostas de escrita, memória do gestor, motores
//! (APIs, CLIs locais, modelos locais) com segredos cifrados, roteamento por tarefa com
//! fallback e limites de gasto, uso e custo, o agente (loop com as ferramentas do MCP do
//! app), jobs assíncronos, conversas, alertas e as rotinas do gestor.

pub mod activity;
pub mod agent;
pub mod alerts;
pub mod api;
pub mod ask;
pub mod attachments;
pub mod cli;
pub mod conversation;
pub mod effort;
pub mod engine;
pub mod export;
pub mod history;
pub mod jobs;
pub mod mcp_extra;
pub mod memory_graph;
pub mod playbooks;
pub mod procedures;
pub mod prompt;
pub mod proposals;
pub mod providers;
pub mod redact;
pub mod routines;
pub mod routing;
pub mod secret;
pub mod settings;
pub mod suggestions;
pub mod usage;

pub use routines::run_due;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    /// A aprovação falhou (dados mudaram depois da proposta).
    Failed,
    /// Retirada sozinha: os mesmos dados mudaram antes da aprovação (à mão, numa
    /// importação ou por outra proposta aprovada), e aprovar sobrescreveria essa mudança.
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProposalLevel {
    /// Pode ser aprovada em lote.
    Reversible,
    /// Aprovação item a item.
    Sensitive,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Proposal {
    pub id: Id,
    pub change_set_id: Id,
    pub client: String,
    pub command: String,
    pub level: ProposalLevel,
    pub status: ProposalStatus,
    pub reason: String,
    pub created_at: String,
    pub reviewed_at: Option<String>,
    pub error: Option<String>,
    pub note: Option<String>,
    #[ts(type = "number")]
    pub item_count: i64,
    #[ts(type = "Record<string, Record<string, number>>")]
    pub summary: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProposalDetail {
    #[serde(flatten)]
    pub proposal: Proposal,
    /// Antes/depois de cada linha que será alterada.
    pub items: Vec<crate::changes::ChangeItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProposalListArgs {
    #[ts(optional)]
    pub status: Option<ProposalStatus>,
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewArgs {
    pub ids: Vec<Id>,
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewResult {
    pub id: Id,
    pub status: ProposalStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MemoryKind {
    /// Perfil de investidor, horizonte, objetivos.
    Profile,
    /// Regras da família ("o aluguel vence dia 5", "não sugerir cripto").
    FamilyRule,
    /// Nota gravada pelo agente com `remember`.
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemoryItem {
    pub id: Id,
    pub kind: MemoryKind,
    pub content: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RememberArgs {
    pub kind: MemoryKind,
    pub content: String,
}
