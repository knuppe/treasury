//! Entradas das ferramentas de planejamento: previsão de saldo, quanto posso gastar hoje,
//! simulador de quitação de dívidas e assinaturas detectadas. Todas somente leitura.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::inputs::{ToolInput, to_object, tx, tx_args, tx_campo};

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BalanceForecastInput {
    /// Months ahead (1 to 24; default 3).
    pub months: Option<u32>,
    /// Only these accounts (ids from `list_accounts`). Default: the everyday accounts (no
    /// brokerages).
    pub account_ids: Option<Vec<String>>,
}

impl ToolInput for BalanceForecastInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.months.is_some_and(|m| !(1..=24).contains(&m)) {
            return Err(tx_args(
                "mcp-valida-faixa",
                &[
                    ("campo", "months".into()),
                    ("min", 1.into()),
                    ("max", 24.into()),
                ],
            ));
        }
        to_object(self)
    }
}

/// Month to close.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MonthCloseInput {
    /// Month `YYYY-MM` that has already ended (default: the last one).
    pub month: Option<String>,
}

impl ToolInput for MonthCloseInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// No arguments.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DailyAllowanceInput {}

impl ToolInput for DailyAllowanceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(json!({}))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PayoffCustomDebt {
    /// Name (e.g. "Overdraft").
    pub name: String,
    /// Outstanding balance today, in cents.
    pub balance: i64,
    /// Annual interest in %, decimal as text (`"151.8"`); with an index, `"4,75 + TR"` (the
    /// latest index value is added and held constant).
    pub annual_rate: String,
    /// Minimum monthly payment, in cents.
    pub min_payment: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DebtPayoffInput {
    /// How much is left each month to pay beyond the installments, in cents (default 0).
    pub extra_monthly: Option<i64>,
    /// Only these contracts (ids from `list_loans`). Default: all active ones with a balance.
    pub loan_ids: Option<Vec<String>>,
    /// Debts outside the app (overdraft, revolving credit, installment plans).
    pub custom: Option<Vec<PayoffCustomDebt>>,
}

impl ToolInput for DebtPayoffInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.extra_monthly.is_some_and(|e| e < 0) {
            return Err(tx_campo("mcp-valida-nao-negativo", "extra_monthly"));
        }
        to_object(self)
    }
}

/// No arguments.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionsInput {}

impl ToolInput for SubscriptionsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(json!({}))
    }
}

/// Life event in the scenario (same shapes as `scenarios::Event`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScenarioEventInput {
    /// One-time inflow (+) or outflow (−), in cents.
    OneOff {
        label: String,
        /// `YYYY-MM-DD`.
        date: String,
        amount: i64,
    },
    /// Amount more (+) or less (−) per month for `months` months (omitted = forever).
    Monthly {
        label: String,
        start: String,
        months: Option<u32>,
        amount: i64,
    },
    /// Income at `pct`% of normal (0 = career break, 50 = part time).
    IncomePct {
        label: String,
        start: String,
        months: Option<u32>,
        pct: f64,
    },
    /// Physical asset added to net worth, with real appreciation per year in %.
    Asset {
        label: String,
        date: String,
        value: i64,
        real_growth_pct: Option<f64>,
    },
    /// Financing with fixed installments (Price table): the amount goes into cash and becomes
    /// debt; an installment every month.
    Loan {
        label: String,
        date: String,
        principal: i64,
        /// Real interest per year in %.
        annual_rate_pct: f64,
        months: u32,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScenarioInput {
    /// Scenario name (or the name of a saved one from `list_scenarios`, if `events` is empty).
    pub name: Option<String>,
    /// Life events. Buying a home = `one_off` with the price (negative) + `loan` with the
    /// financed amount + `asset` with the home's value.
    #[serde(default)]
    pub events: Vec<ScenarioEventInput>,
    /// Years ahead (1 to 40; default 10).
    pub years: Option<u32>,
    /// Real annual return as a fraction (default: the retirement plan's, or 0.04).
    pub real_return: Option<f64>,
    /// Starting monthly income in cents (default: average of the last 12 months).
    pub monthly_income: Option<i64>,
    /// Starting monthly spending in cents (default: average of the last 12 months).
    pub monthly_expense: Option<i64>,
}

impl ToolInput for ScenarioInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.years.is_some_and(|y| !(1..=40).contains(&y)) {
            return Err(tx_args(
                "mcp-valida-faixa",
                &[
                    ("campo", "years".into()),
                    ("min", 1.into()),
                    ("max", 40.into()),
                ],
            ));
        }
        to_object(self)
    }
}

/// Credit card statement PDF to read without saving.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardPdfInput {
    /// The card statement PDF (external MCP clients; one per call).
    pub files: Option<Vec<super::inputs_crud::NoteFileInput>>,
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
    /// Password of the protected PDF, if the user gave it in the conversation.
    pub password: Option<String>,
}

impl ToolInput for CardPdfInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.attachments.is_some() {
            return Err(tx("mcp-valida-anexo-nao-encontrado"));
        }
        let [f] = self.files.as_deref().unwrap_or_default() else {
            return Err(tx("mcp-valida-um-pdf"));
        };
        let mut v = json!({"filename": f.filename, "content_base64": f.content_base64});
        if let Some(p) = &self.password {
            v["password"] = p.clone().into();
        }
        Ok(v)
    }
}

/// Market interest rates (BCB) and how the loans compare.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MarketRatesInput {
    /// Download again even if the data is recent (default: only if older than 7 days).
    pub refresh: Option<bool>,
}

impl ToolInput for MarketRatesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IncomeSummaryInput {
    /// Forecast months (1 to 24; default 12).
    pub months: Option<u32>,
    /// Only one member (id from `list_members`).
    pub member_id: Option<String>,
}

impl ToolInput for IncomeSummaryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.months.is_some_and(|m| !(1..=24).contains(&m)) {
            return Err(tx_args(
                "mcp-valida-faixa",
                &[
                    ("campo", "months".into()),
                    ("min", 1.into()),
                    ("max", 24.into()),
                ],
            ));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncomeKindInput {
    /// Formal employment (carteira assinada, CLT).
    Clt,
    /// Public servant.
    PublicServant,
    /// Retirement (INSS or a public servants' scheme).
    Retirement,
    /// Pension (survivor's or alimony).
    Pension,
    /// Company contractor (PJ).
    Pj,
    /// Self-employed.
    SelfEmployed,
    /// Informal (odd jobs, day work, unregistered).
    Informal,
    /// Rent received.
    Rental,
    /// Social benefit (Bolsa Família, BPC, unemployment insurance).
    Benefit,
    /// Other.
    Other,
}

/// A member's income entered by hand (no payslip: informal, PJ, rent, retirement; or CLT when
/// the user only gives the amounts). Amounts in cents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateIncomeSourceInput {
    /// Member who receives it (ids from `list_members` or `get_income_summary`).
    pub member_id: String,
    /// Income name (e.g. "Salário Ailos", "Diárias de faxina", "Aluguel do apto").
    pub name: String,
    /// Income kind.
    pub kind: IncomeKindInput,
    /// Employer or payer.
    pub employer: Option<String>,
    /// Employer's CNPJ (digits only or formatted).
    pub employer_cnpj: Option<String>,
    /// Reference gross monthly amount, in cents (e.g. 446086 = 4,460.86).
    #[schemars(range(min = 0))]
    pub gross_monthly: Option<i64>,
    /// Reference net monthly amount, in cents.
    #[schemars(range(min = 0))]
    pub net_monthly: Option<i64>,
    /// The amount changes every month (commission, odd jobs).
    pub variable: Option<bool>,
    /// Payday (1 to 31).
    #[schemars(range(min = 1, max = 31))]
    pub pay_day: Option<u32>,
    /// Receives a 13th salary (default: yes for incomes with payroll deductions).
    pub has_thirteenth: Option<bool>,
    /// Vacation month (1 to 12), if known.
    #[schemars(range(min = 1, max = 12))]
    pub vacation_month: Option<u32>,
    /// Income tax dependents.
    pub dependents: Option<u32>,
    /// Account the money goes into (ids from `list_accounts`).
    pub account_id: Option<String>,
    /// Start `YYYY-MM-DD`.
    #[schemars(pattern(super::inputs::DATE_PATTERN))]
    pub start_date: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl ToolInput for CreateIncomeSourceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.member_id.trim().is_empty() || self.name.trim().is_empty() {
            return Err(tx("mcp-valida-membro-nome"));
        }
        to_object(self)
    }
}

/// Payslip (holerite) to read or import.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PayslipInput {
    /// The payslip PDF (external MCP clients; one per call).
    pub files: Option<Vec<super::inputs_crud::NoteFileInput>>,
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
    /// Or the whole payslip text, as it is in the document (no summarizing: the reader looks for
    /// lines like "Total de vencimentos 4.460,86" and "Líquido a receber 2.043,36").
    pub text: Option<String>,
    /// PDF password, if the user gave it in the conversation.
    pub password: Option<String>,
    /// Income source that receives the payslip (ids from `get_income_summary`); default: the
    /// one with the same CNPJ.
    pub source_id: Option<String>,
    /// No income source registered: the member of the new income source (CLT, formal
    /// employment, with the payslip's employer).
    pub member_id: Option<String>,
    /// Reference month (competência) `YYYY-MM` when the payslip doesn't state it.
    pub reference_month: Option<String>,
    /// Replaces the payslip with the same income source, reference month and type (import only).
    pub replace: Option<bool>,
}

impl ToolInput for PayslipInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.attachments.is_some() {
            return Err(tx("mcp-valida-anexo-nao-encontrado"));
        }
        if self.files.as_ref().is_none_or(Vec::is_empty)
            && self.text.as_deref().is_none_or(|t| t.trim().is_empty())
        {
            return Err(tx("mcp-valida-holerite"));
        }
        to_object(self)
    }
}

/// Receipt item read from a photo.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReceiptItemInput {
    /// Description as on the receipt ("ARROZ TIO JOAO 5KG").
    pub description: String,
    /// Quantity as on the receipt ("1", "0,535").
    pub quantity: Option<String>,
    /// Unit ("UN", "KG").
    pub unit: Option<String>,
    /// Unit price in cents.
    pub unit_price: Option<i64>,
    /// Item amount in cents (positive).
    pub total: i64,
}

/// Receipt read from a photo (no QR code, or an unreadable one).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ManualReceiptInput {
    /// Purchase date `YYYY-MM-DD`.
    pub date: String,
    /// Store name.
    pub store_name: Option<String>,
    /// Items as on the receipt.
    pub items: Vec<ReceiptItemInput>,
    /// Total discount in cents.
    pub discount: Option<i64>,
    /// Amount paid in cents (default: items − discount).
    pub paid: Option<i64>,
    /// "Tributos aproximados" (approximate taxes) printed on the receipt, in cents, if any.
    pub tax_total: Option<i64>,
}

/// Fiscal receipt (cupom fiscal) to read or import (Brazil only).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReceiptInput {
    /// Link from the receipt's QR code (SEFAZ public lookup, `https://….gov.br/…?p=…`).
    pub url: Option<String>,
    /// The DANFE NFC-e PDF or the invoice XML (external MCP clients).
    pub files: Option<Vec<super::inputs_crud::NoteFileInput>>,
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
    /// Or the text of the receipt / of the lookup page.
    pub text: Option<String>,
    /// Or the items read from a photo of the receipt.
    pub manual: Option<ManualReceiptInput>,
    /// Date, when the receipt doesn't show it.
    pub date: Option<String>,
    /// Purchase transaction (default: the one matching the amount paid, near the date).
    pub txn_id: Option<String>,
    /// Member who made the purchase (default: the transaction's member).
    pub member_id: Option<String>,
    /// Replaces the receipt with the same access key (import only).
    pub replace: Option<bool>,
}

impl ToolInput for ReceiptInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.attachments.is_some() {
            return Err(tx("mcp-valida-anexo-nao-encontrado"));
        }
        let has = self.url.as_deref().is_some_and(|u| !u.trim().is_empty())
            || self.files.as_ref().is_some_and(|f| !f.is_empty())
            || self.text.as_deref().is_some_and(|t| !t.trim().is_empty())
            || self.manual.is_some();
        if !has {
            return Err(tx("mcp-valida-cupom"));
        }
        to_object(self)
    }
}

/// Period and member of the purchases.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReceiptPeriodInput {
    /// `YYYY-MM-DD` (default: January 1 of the year).
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// `YYYY-MM-DD` (default: today).
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
    /// Only one member's purchases (ids from `list_members`).
    pub member_id: Option<String>,
}

impl ToolInput for ReceiptPeriodInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// Window of the family's own basket index.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BasketInflationInput {
    /// Months shown, from 3 to 36 (default 12).
    pub months: Option<u32>,
}

impl ToolInput for BasketInflationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// Why the job ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TerminationReasonInput {
    /// Dismissal without just cause.
    SemJustaCausa,
    /// The employee resigns.
    PedidoDemissao,
    /// Dismissal for just cause.
    JustaCausa,
    /// Mutual agreement (CLT art. 484-A).
    Acordo,
    /// End of the trial contract on the agreed date.
    FimExperiencia,
    /// Trial contract ended early by the employer.
    QuebraExperienciaEmpregador,
    /// Trial contract ended early by the employee.
    QuebraExperienciaEmpregado,
}

/// `indenizado` (paid, not worked), `trabalhado` (served) or `nao_cumprido` (resignation
/// without serving: the employer may deduct 30 days).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum NoticeModeInput {
    Indenizado,
    Trabalhado,
    NaoCumprido,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SimulateTerminationInput {
    /// The CLT income (id from `get_income_summary`): salary, hiring date, dependents and the
    /// linked FGTS account come from it.
    pub source_id: Option<String>,
    /// Gross monthly salary in cents (default: the income's).
    #[schemars(range(min = 1))]
    pub gross_monthly: Option<i64>,
    /// Hiring date `YYYY-MM-DD` (default: the income's start or the FGTS admission).
    pub admission_date: Option<String>,
    /// Last day of work `YYYY-MM-DD`.
    pub termination_date: String,
    pub reason: TerminationReasonInput,
    /// Default: `indenizado` when the employer dismisses, `trabalhado` on resignation.
    pub notice: Option<NoticeModeInput>,
    /// IRRF dependents (default: the income's).
    #[schemars(range(min = 0, max = 20))]
    pub dependents: Option<u32>,
    /// Overdue vacation days not taken (0 to 60).
    #[schemars(range(min = 0, max = 60))]
    pub overdue_vacation_days: Option<u32>,
    /// FGTS balance for termination purposes of this job, in cents (default: the linked FGTS
    /// account's).
    #[schemars(range(min = 0))]
    pub fgts_balance: Option<i64>,
    /// Agreed end of the trial contract `YYYY-MM-DD` (required when it ends early).
    pub contract_end: Option<String>,
    /// How many times unemployment insurance was already received (0 = first claim).
    #[schemars(range(min = 0, max = 10))]
    pub previous_claims: Option<u32>,
    /// Months worked with a formal contract in the last 36 (all jobs; default: this job's).
    #[schemars(range(min = 0, max = 36))]
    pub months_worked_36: Option<u32>,
    /// Last gross salaries before the dismissal (up to 3, cents), for the insurance average.
    pub last_salaries: Option<Vec<i64>>,
}

impl ToolInput for SimulateTerminationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        let mut v = to_object(self)?;
        if let Some(obj) = v.as_object_mut() {
            obj.retain(|_, x| !x.is_null());
        }
        Ok(v)
    }
}
