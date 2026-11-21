//! Entradas das ferramentas da linha da vida (docs/life-expectancy.md §7): só leitura, e só
//! existem para o cliente com a função ligada no cofre (grupo `Longevity` de `tools`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::inputs::{ToolInput, tx_campo};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MemberOutlookInput {
    /// Family member id (from `list_members`).
    pub member_id: String,
}

impl ToolInput for MemberOutlookInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.member_id.trim().is_empty() {
            return Err(tx_campo("mcp-valida-informe", "member_id"));
        }
        Ok(json!({ "member_id": self.member_id.trim() }))
    }
}

/// No arguments.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FamilyOutlookInput {}

impl ToolInput for FamilyOutlookInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(json!({}))
    }
}
