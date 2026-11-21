//! Entradas das ferramentas do FGTS (docs/brazil/fgts.md, seção 7). Só no Brasil: fora dele o
//! núcleo recusa com uma mensagem clara.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::inputs::{ToolInput, to_object, tx, tx_args, tx_campo};
use super::inputs_crud::NoteFileInput;

fn required(v: &str, field: &str) -> Result<(), String> {
    if v.trim().is_empty() {
        Err(tx_campo("mcp-valida-informe", field))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsListInput {
    /// Also list archived accounts (default: no; they are not in the total either).
    pub include_archived: Option<bool>,
}

impl ToolInput for FgtsListInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(json!({ "active_only": !self.include_archived.unwrap_or(false) }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsAccountInput {
    /// FGTS account id (see `list_fgts_accounts`).
    pub account_id: String,
}

impl ToolInput for FgtsAccountInput {
    fn to_api_args(&self) -> Result<Value, String> {
        required(&self.account_id, "account_id")?;
        Ok(json!({ "account_id": self.account_id.trim() }))
    }
}

/// Caixa FGTS statement ("Histórico" PDF, one per employer account).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsStatementInput {
    /// The statement PDF (external MCP clients).
    pub files: Option<Vec<NoteFileInput>>,
    /// File the user attached in the conversation (app agent and CLIs): its order (`"1"`), name
    /// or the id from `read_attachment`. Treasury reads the content; do not repeat the bytes or
    /// the text.
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
    /// Or the statement text (copied from the PDF). Together with the PDF, it only completes the
    /// header (what the PDF did not show); the lines always come from the PDF.
    pub text: Option<String>,
    /// FGTS account that receives it (see `list_fgts_accounts`); default: the one with the same
    /// account number or employer CNPJ, or a new account.
    pub account_id: Option<String>,
    /// Holder of a new account (member id, see `list_members`).
    pub member_id: Option<String>,
    /// Name of a new account (default "FGTS · <employer>").
    pub name: Option<String>,
    /// Reason of each withdrawal group (key: the group's `key` in the preview): `rescisao`,
    /// `aposentadoria`, `moradia`, `aniversario`, `doenca`, `emergencial`, `outros`. Code 99
    /// defaults to `moradia`.
    pub reasons: Option<BTreeMap<String, String>>,
    /// Account that received each withdrawal (key: the group's `key`): becomes a transfer.
    pub destinations: Option<BTreeMap<String, String>>,
    /// Employment data the statement header did not show or that was read wrong: they replace
    /// the values read (check the `header` of the preview). Employer name.
    pub employer_name: Option<String>,
    /// Employer CNPJ.
    pub employer_cnpj: Option<String>,
    /// Admission date (`YYYY-MM-DD`).
    pub admission_date: Option<String>,
    /// FGTS option date (`YYYY-MM-DD`).
    pub option_date: Option<String>,
    /// Leave date (`YYYY-MM-DD`), when the job ended.
    pub leave_date: Option<String>,
    /// Leave code (e.g. `I1`).
    pub leave_code: Option<String>,
    /// Worker category on the statement (e.g. `1`).
    pub category: Option<String>,
    /// CLT income of this job (see `get_income_summary`), to check deposits against payslips.
    pub income_source_id: Option<String>,
    /// Preview with every line (default: no; you get the totals, the checks, the withdrawal
    /// groups and the lines that differ from the rule).
    pub lines: Option<bool>,
}

impl ToolInput for FgtsStatementInput {
    fn post_process(&self, mut value: Value) -> Value {
        if self.lines.unwrap_or(false) {
            return value;
        }
        let nested = value.get("preview").is_some_and(Value::is_object);
        let v = if nested {
            value.get_mut("preview").expect("prévia")
        } else {
            &mut value
        };
        if let Some(lines) = v.get("lines").and_then(Value::as_array).cloned() {
            let kept: Vec<Value> = lines
                .into_iter()
                .filter(|l| {
                    l["jam_expected"]
                        .as_i64()
                        .is_some_and(|e| Some(e) != l["amount"].as_i64())
                })
                .take(40)
                .collect();
            if let Some(m) = v.as_object_mut() {
                m.insert("lines".into(), Value::Array(kept));
                m.insert("lines_note".into(), Value::from(tx("mcp-nota-linhas-fgts")));
            }
        }
        value
    }

    fn to_api_args(&self) -> Result<Value, String> {
        if self.attachments.is_some() {
            return Err(tx("mcp-valida-anexo-nao-encontrado"));
        }
        if self.files.as_ref().is_none_or(Vec::is_empty)
            && self.text.as_deref().is_none_or(|t| t.trim().is_empty())
        {
            return Err(tx("mcp-valida-extrato-pdf"));
        }
        let mut v = to_object(self)?;
        if let Some(m) = v.as_object_mut() {
            m.remove("lines");
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsCompareInput {
    /// One FGTS account; default: all of them together.
    pub account_id: Option<String>,
    /// Period start `YYYY-MM-DD` (default: the first entry).
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// Period end `YYYY-MM-DD` (default: today).
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
}

impl ToolInput for FgtsCompareInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsProjectInput {
    /// One FGTS account; default: all of the member's (or all).
    pub account_id: Option<String>,
    /// Member id (see `list_members`).
    pub member_id: Option<String>,
    /// Target date `YYYY-MM-DD`…
    pub date: Option<String>,
    /// …or the holder's age.
    #[schemars(range(min = 16, max = 100))]
    pub age: Option<u32>,
}

impl ToolInput for FgtsProjectInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.date.is_none() && self.age.is_none() {
            return Err(tx_args(
                "mcp-valida-informe-um",
                &[("a", "date".into()), ("b", "age".into())],
            ));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsAnniversaryInput {
    /// Member id (see `list_members`); default: the holder of the FGTS accounts.
    pub member_id: Option<String>,
}

impl ToolInput for FgtsAnniversaryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsWithdrawalInput {
    /// FGTS account id (see `list_fgts_accounts`).
    pub account_id: String,
    /// Date `YYYY-MM-DD`.
    pub date: String,
    /// Amount withdrawn, in cents (positive).
    #[schemars(range(min = 1))]
    pub amount: i64,
    /// Withdrawal code on the statement (e.g. "01", "60", "99").
    pub code: Option<String>,
    /// `rescisao`, `aposentadoria`, `moradia`, `aniversario`, `doenca`, `emergencial`, `outros`.
    pub reason: Option<String>,
    /// Account that received the money (becomes a transfer; see `list_accounts`).
    pub destination_account_id: Option<String>,
}

impl ToolInput for FgtsWithdrawalInput {
    fn to_api_args(&self) -> Result<Value, String> {
        required(&self.account_id, "account_id")?;
        to_object(self)
    }
}

/// `reduce_term` (default) or `reduce_installment`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum FgtsPrepayMode {
    ReduceTerm,
    ReduceInstallment,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsHousingInput {
    /// FGTS account id (see `list_fgts_accounts`).
    pub account_id: String,
    /// Home financing contract id (see `list_loans`; kind `mortgage`).
    pub loan_id: String,
    /// Date `YYYY-MM-DD`.
    pub date: String,
    /// Amount in cents.
    #[schemars(range(min = 1))]
    pub amount: i64,
    /// What the extra payment reduces (default `reduce_term`).
    pub mode: Option<FgtsPrepayMode>,
}

impl ToolInput for FgtsHousingInput {
    fn to_api_args(&self) -> Result<Value, String> {
        required(&self.account_id, "account_id")?;
        required(&self.loan_id, "loan_id")?;
        to_object(self)
    }
}

/// `rescisao` (dismissal withdrawal, the default) or `aniversario` (anniversary withdrawal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum FgtsModality {
    Rescisao,
    Aniversario,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsModalityInput {
    /// Member id (see `list_members`).
    pub member_id: String,
    /// The withdrawal option recorded.
    pub modality: FgtsModality,
    /// Since when `YYYY-MM-DD` (default today).
    pub since: Option<String>,
    /// When the return to the dismissal withdrawal was requested `YYYY-MM-DD`.
    pub return_requested_at: Option<String>,
    /// Loan id of the anniversary-withdrawal advance, if any (see `list_loans`).
    pub anticipation_debt_id: Option<String>,
}

impl ToolInput for FgtsModalityInput {
    fn to_api_args(&self) -> Result<Value, String> {
        required(&self.member_id, "member_id")?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FgtsUpdateAccountInput {
    /// FGTS account id (see `list_fgts_accounts`).
    pub account_id: String,
    /// Employer name.
    pub employer_name: Option<String>,
    /// Employer CNPJ (00.000.000/0000-00).
    pub employer_cnpj: Option<String>,
    /// Admission date `YYYY-MM-DD`.
    pub admission_date: Option<String>,
    /// FGTS option date `YYYY-MM-DD`.
    pub option_date: Option<String>,
    /// Leave (end of employment) date `YYYY-MM-DD`.
    pub leave_date: Option<String>,
    /// Leave code on the statement (e.g. "I1").
    pub leave_code: Option<String>,
    /// Linked CLT income (see `list_income_sources`), to check deposits against payslips.
    pub income_source_id: Option<String>,
}

impl ToolInput for FgtsUpdateAccountInput {
    fn to_api_args(&self) -> Result<Value, String> {
        required(&self.account_id, "account_id")?;
        let mut v = to_object(self)?;
        // Campos ausentes não mexem; `null` explícito não chega por aqui.
        if let Some(m) = v.as_object_mut() {
            m.retain(|_, x| !x.is_null());
        }
        Ok(v)
    }
}
