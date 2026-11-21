//! Entradas das ferramentas de **cadastro** do agente companheiro (docs/ai-agent.md §Ação):
//! instituições, contas e cartões, categorias, ativos (com classe, bolsa e renda fixa),
//! recorrências, metas, divisões e transferências, e patrimônio físico. Como as demais
//! escritas, cada chamada vira **proposta** pendente, com o nível da política
//! (`api::policy::AI_WRITES`): reversível em lote, sensível item a item.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::inputs::{
    CategoryKind, DATE_PATTERN, SpendGroup, ToolInput, TxnStatus, check_date, check_non_empty,
    to_object, tx, tx_args, tx_campo,
};
use super::inputs_invest::AssetClass;
use super::inputs_write::{Freq, GoalKind};

fn check_len(field: &str, v: &str, max: usize) -> Result<(), String> {
    if v.chars().count() > max {
        return Err(tx_args(
            "mcp-valida-tamanho",
            &[("campo", field.to_string().into()), ("max", max.into())],
        ));
    }
    Ok(())
}

fn check_decimal(field: &str, v: &Option<String>) -> Result<(), String> {
    if let Some(v) = v {
        v.trim()
            .replace(',', ".")
            .parse::<rust_decimal::Decimal>()
            .map_err(|_| tx_campo("mcp-valida-decimal", field))?;
    }
    Ok(())
}

fn check_currency(v: &Option<String>) -> Result<(), String> {
    if let Some(c) = v
        && (c.len() != 3 || !c.chars().all(|c| c.is_ascii_alphabetic()))
    {
        return Err(tx("mcp-valida-moeda"));
    }
    Ok(())
}

/// Aplica a lista `clear` (campos a limpar com `null`) e confere que sobrou o que mudar.
fn with_clears(mut v: Value, clears: &[&'static str]) -> Result<Value, String> {
    for key in clears {
        v[*key] = Value::Null;
    }
    if v.as_object().is_some_and(|m| m.len() <= 1) {
        return Err(tx("mcp-valida-um-campo"));
    }
    Ok(v)
}

// ---------------------------------------------------------------------------
// Instituições

/// `bank` bank, `broker` brokerage, `issuer` card issuer, `other` other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum InstitutionKind {
    Bank,
    Broker,
    Issuer,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateInstitutionInput {
    /// Name (e.g. "Itaú", "XP Investimentos", "Nubank").
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    /// Type (default `bank`).
    pub kind: Option<InstitutionKind>,
    /// Bank code (COMPE, 3 digits), if known.
    pub code: Option<String>,
    /// Official website (https), if the user gave it.
    pub website: Option<String>,
}

impl ToolInput for CreateInstitutionInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        check_len("name", &self.name, 120)?;
        to_object(self)
    }
}

/// Field that can be cleared in `update_institution`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ClearableInstitutionField {
    Code,
    Website,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateInstitutionInput {
    /// Institution id (see `list_institutions`).
    pub id: String,
    /// New name.
    pub name: Option<String>,
    /// New type.
    pub kind: Option<InstitutionKind>,
    /// New bank code.
    pub code: Option<String>,
    /// New website (https).
    pub website: Option<String>,
    /// Fields to clear.
    pub clear: Option<Vec<ClearableInstitutionField>>,
}

impl ToolInput for UpdateInstitutionInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        let mut input = self.clone();
        let clear = input.clear.take().unwrap_or_default();
        let keys: Vec<&'static str> = clear
            .iter()
            .map(|c| match c {
                ClearableInstitutionField::Code => "code",
                ClearableInstitutionField::Website => "website",
            })
            .collect();
        with_clears(to_object(&input)?, &keys)
    }
}

// ---------------------------------------------------------------------------
// Contas e cartões

/// `checking` checking, `savings` savings, `cash` cash, `credit_card` credit card,
/// `brokerage` brokerage, `foreign` foreign account, `fgts` FGTS linked account (Brazil only,
/// one per employer), `benefit` meal/food benefit card (VR, VA, Flash), `retirement` the
/// country's retirement account (401(k), IRA, PPR…; plan in `retirement_plan`), `other` other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AccountKind {
    Checking,
    Savings,
    Cash,
    CreditCard,
    Brokerage,
    Foreign,
    Fgts,
    Benefit,
    Retirement,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CardDetailsInput {
    /// Limit in cents.
    pub credit_limit: Option<i64>,
    /// Card statement closing day (1–31).
    #[schemars(range(min = 1, max = 31))]
    pub closing_day: u32,
    /// Due day (1–31).
    #[schemars(range(min = 1, max = 31))]
    pub due_day: u32,
    /// Account that pays the card statement (see `list_accounts`).
    pub payment_account_id: Option<String>,
    /// Card network (Visa, Mastercard, Elo…).
    pub network: Option<String>,
    /// Only the last 4 digits, if the user gave them; never the full number.
    pub last4: Option<String>,
}

impl CardDetailsInput {
    fn check(&self) -> Result<(), String> {
        if !(1..=31).contains(&self.closing_day) || !(1..=31).contains(&self.due_day) {
            return Err(tx("mcp-valida-dias-cartao"));
        }
        if let Some(l) = &self.last4
            && (l.len() != 4 || !l.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(tx("mcp-valida-last4"));
        }
        if self.credit_limit.is_some_and(|l| l < 0) {
            return Err(tx_campo("mcp-valida-nao-negativo", "credit_limit"));
        }
        Ok(())
    }

    fn to_value(&self) -> Value {
        // O núcleo espera todos os campos (os opcionais como `null`).
        json!({
            "credit_limit": self.credit_limit,
            "closing_day": self.closing_day,
            "due_day": self.due_day,
            "payment_account_id": self.payment_account_id,
            "network": self.network,
            "last4": self.last4,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateAccountInput {
    /// Account name (e.g. "Itaú checking", "Nubank Ultravioleta").
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    /// Account type. A credit card requires `card`.
    pub kind: AccountKind,
    /// Currency (default: the family's base currency).
    pub currency: Option<String>,
    /// Institution already registered (see `list_institutions`).
    pub institution_id: Option<String>,
    /// Institution name: uses the existing one with that name or creates it in the same
    /// proposal. Use when the institution is not registered yet.
    pub institution_name: Option<String>,
    /// Account holders (members; see `list_members`).
    pub member_ids: Option<Vec<String>>,
    /// Opening balance in cents (on the opening date), if the user gave it.
    pub opening_balance: Option<i64>,
    /// Opening balance date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub opening_date: Option<String>,
    /// Counts toward net worth (default: yes).
    pub include_in_net_worth: Option<bool>,
    /// Notes.
    pub notes: Option<String>,
    /// Card details (only for `credit_card`).
    pub card: Option<CardDetailsInput>,
    /// Overdraft (only for a `checking` account).
    pub overdraft: Option<OverdraftInput>,
    /// Branch (digits and hyphen only), as shown on the statement; matches the account in OFX.
    pub branch: Option<String>,
    /// Account number with check digit (e.g. "12345-6"); matches the account in OFX.
    pub account_number: Option<String>,
    /// Only for `retirement`: the country's plan (US `us_401k`, `us_roth_401k`, `us_ira`,
    /// `us_roth_ira`, `us_hsa`, `us_529`; PT `pt_ppr`; ES `es_plan_pensiones`,
    /// `es_plan_empleo`; GB `gb_isa`, `gb_lisa`, `gb_pension`; CA `ca_rrsp`, `ca_tfsa`,
    /// `ca_fhsa`; FR `fr_per`, `fr_pea`; DE `de_riester`, `de_ruerup`; CL `cl_apv`; default:
    /// the country's first). The app tracks the year's contributions against the
    /// plan's annual limit.
    pub retirement_plan: Option<String>,
    /// Only for an HSA: `self` or `family` coverage (the limit differs).
    pub retirement_coverage: Option<String>,
}

impl ToolInput for CreateAccountInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        check_len("name", &self.name, 120)?;
        check_currency(&self.currency)?;
        check_date("opening_date", &self.opening_date)?;
        // Plano e cobertura conferidos antes de virar proposta (sem conta-teste na fila).
        if self.retirement_plan.is_some() || self.retirement_coverage.is_some() {
            if self.kind != AccountKind::Retirement {
                return Err(tx("mcp-valida-plano-so-aposentadoria"));
            }
            if let Some(p) = &self.retirement_plan
                && crate::retirement::plan(p.trim()).is_none()
            {
                let all: Vec<&str> = crate::retirement::PLANS.iter().map(|p| p.id).collect();
                return Err(tx_args(
                    "mcp-valida-plano-aposentadoria",
                    &[
                        ("plano", p.clone().into()),
                        ("planos", all.join(", ").into()),
                    ],
                ));
            }
            if let Some(c) = &self.retirement_coverage
                && !matches!(c.trim(), "self" | "family")
            {
                return Err(tx_args(
                    "mcp-valida-cobertura",
                    &[("cobertura", c.clone().into())],
                ));
            }
        }
        match (self.kind, &self.card) {
            (AccountKind::CreditCard, None) => {
                return Err(tx("mcp-valida-cartao-sem-card"));
            }
            (AccountKind::CreditCard, Some(c)) => c.check()?,
            (_, Some(_)) => return Err(tx("mcp-valida-card-so-cartao")),
            (_, None) => {}
        }
        let mut v = to_object(self)?;
        if let Some(c) = &self.card {
            v["card"] = c.to_value();
        }
        Ok(v)
    }
}

/// Checking account overdraft (cheque especial, a credit line), as the bank states in the
/// contract or the app. Rates in % as decimal text ("8,5").
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OverdraftInput {
    /// Limit in cents.
    #[schemars(range(min = 0))]
    pub limit: i64,
    /// Monthly interest rate (%).
    pub monthly_rate: Option<String>,
    /// Discount percentage on the rate (%).
    pub discount_pct: Option<String>,
    /// Annual interest rate charged (%). Floating rates go with the index in the same text:
    /// "4,75 + TR" (4.75% a year over TR; also CDI, Selic, IPCA, IGP-M, INPC, poupança).
    pub annual_rate: Option<String>,
    /// IOF base rate, per day (%), usually "0,0082".
    pub iof_daily_rate: Option<String>,
    /// IOF additional rate (%), usually "0,38".
    pub iof_additional_rate: Option<String>,
    /// CET (total effective cost) per month (%).
    pub cet_month: Option<String>,
    /// CET per year (%).
    pub cet_year: Option<String>,
}

/// Field that can be cleared in `update_account`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ClearableAccountField {
    InstitutionId,
    Notes,
    /// Removes the overdraft.
    Overdraft,
    Branch,
    AccountNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateAccountInput {
    /// Account id (see `list_accounts`).
    pub id: String,
    /// New name.
    pub name: Option<String>,
    /// New institution (see `list_institutions`).
    pub institution_id: Option<String>,
    /// Replaces the account holders.
    pub member_ids: Option<Vec<String>>,
    /// New opening balance in cents (changes the balance history).
    pub opening_balance: Option<i64>,
    /// New opening balance date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub opening_date: Option<String>,
    /// Counts toward net worth.
    pub include_in_net_worth: Option<bool>,
    /// Archive (hidden from lists, history kept).
    pub archived: Option<bool>,
    /// New notes.
    pub notes: Option<String>,
    /// New card details (replaces all of them).
    pub card: Option<CardDetailsInput>,
    /// Checking account overdraft (replaces all its data).
    pub overdraft: Option<OverdraftInput>,
    /// Branch (digits and hyphen only).
    pub branch: Option<String>,
    /// Account number with check digit.
    pub account_number: Option<String>,
    /// Fields to clear.
    pub clear: Option<Vec<ClearableAccountField>>,
}

impl ToolInput for UpdateAccountInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("opening_date", &self.opening_date)?;
        if let Some(n) = &self.name {
            check_non_empty("name", n)?;
        }
        if let Some(c) = &self.card {
            c.check()?;
        }
        let mut input = self.clone();
        let clear = input.clear.take().unwrap_or_default();
        let mut v = to_object(&input)?;
        if let Some(c) = &self.card {
            v["card"] = c.to_value();
        }
        let keys: Vec<&'static str> = clear
            .iter()
            .map(|c| match c {
                ClearableAccountField::InstitutionId => "institution_id",
                ClearableAccountField::Notes => "notes",
                ClearableAccountField::Overdraft => "overdraft",
                ClearableAccountField::Branch => "branch",
                ClearableAccountField::AccountNumber => "account_number",
            })
            .collect();
        with_clears(v, &keys)
    }
}

// ---------------------------------------------------------------------------
// Categorias

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateCategoryInput {
    /// Name (e.g. "Pet", "Streaming").
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    /// Income or expense (same as the parent's, if any).
    pub kind: CategoryKind,
    /// Parent category (see `list_categories`), for a subcategory.
    pub parent_id: Option<String>,
    /// Essential expense (counts toward the emergency fund base): true for the `essential`
    /// and `debt` groups.
    pub is_essential: Option<bool>,
    /// Spending group of an expense category (default: the parent's; without a parent,
    /// `non_essential`). Income categories have none.
    pub spend_group: Option<SpendGroup>,
}

impl ToolInput for CreateCategoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        check_len("name", &self.name, 80)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateCategoryInput {
    /// Category id (see `list_categories`).
    pub id: String,
    /// New name.
    pub name: Option<String>,
    /// New parent (see `list_categories`).
    pub parent_id: Option<String>,
    /// Move to the top level (no parent).
    pub move_to_root: Option<bool>,
    /// Essential expense: `true` moves a `non_essential` or `future` category to `essential`
    /// (`debt` stays `debt`); `false` moves `essential` or `debt` to `non_essential`.
    pub is_essential: Option<bool>,
    /// Spending group of an expense category (wins over `is_essential`).
    pub spend_group: Option<SpendGroup>,
    /// Archive (hidden from lists, history kept).
    pub archived: Option<bool>,
}

impl ToolInput for UpdateCategoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        if let Some(n) = &self.name {
            check_non_empty("name", n)?;
        }
        if self.parent_id.is_some() && self.move_to_root == Some(true) {
            return Err(tx_args(
                "mcp-valida-um-ou-outro",
                &[("a", "parent_id".into()), ("b", "move_to_root".into())],
            ));
        }
        let mut input = self.clone();
        let keys: &[&'static str] = if input.move_to_root.take() == Some(true) {
            &["parent_id"]
        } else {
            &[]
        };
        with_clears(to_object(&input)?, keys)
    }
}

// ---------------------------------------------------------------------------
// Ativos

/// Security type: `cdb`, `rdb` (bank deposit receipt, like a CDB), `lc`, `lci`, `lca`, `cri`, `cra`, `debenture`,
/// `debenture_incentivada`, `tesouro_selic`, `tesouro_prefixado`, `tesouro_ipca`,
/// `tesouro_renda`, `poupanca`, `other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum FixedIncomeKind {
    Cdb,
    Rdb,
    Lc,
    Lci,
    Lca,
    Cri,
    Cra,
    Debenture,
    DebentureIncentivada,
    TesouroSelic,
    TesouroPrefixado,
    TesouroIpca,
    TesouroRenda,
    Poupanca,
    Other,
}

impl FixedIncomeKind {
    fn exempt(self) -> bool {
        matches!(
            self,
            FixedIncomeKind::Lci
                | FixedIncomeKind::Lca
                | FixedIncomeKind::Cri
                | FixedIncomeKind::Cra
                | FixedIncomeKind::DebentureIncentivada
                | FixedIncomeKind::Poupanca
        )
    }
}

/// Rate index: `cdi`, `selic`, `ipca`, `igpm`, `prefixado` (fixed rate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Indexer {
    Cdi,
    Selic,
    Ipca,
    Igpm,
    Prefixado,
}

/// Liquidity: `daily` daily, `at_maturity` at maturity, `after_grace` after the grace period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Liquidity {
    Daily,
    AtMaturity,
    AfterGrace,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FixedIncomeInput {
    /// Security type.
    pub kind: FixedIncomeKind,
    /// Rate index.
    pub indexer: Indexer,
    /// % of CDI/Selic (e.g. "110") or annual rate (prefixado "12.5"; IPCA + "6.2"), decimal
    /// as text.
    pub rate: String,
    /// Issue date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub issue_date: Option<String>,
    /// Maturity date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub maturity_date: Option<String>,
    /// End of the grace period `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub grace_until: Option<String>,
    /// Liquidity (default: at maturity).
    pub liquidity: Option<Liquidity>,
    /// Exempt from IR (default by type: LCI, LCA, CRI, CRA, incentivized debentures and
    /// poupança).
    pub ir_exempt: Option<bool>,
    /// Issuer (bank or company).
    pub issuer: Option<String>,
}

impl FixedIncomeInput {
    fn to_value(&self) -> Result<Value, String> {
        check_decimal("rate", &Some(self.rate.clone()))?;
        check_date("issue_date", &self.issue_date)?;
        check_date("maturity_date", &self.maturity_date)?;
        check_date("grace_until", &self.grace_until)?;
        Ok(json!({
            "kind": self.kind,
            "indexer": self.indexer,
            "rate": self.rate.trim(),
            "issue_date": self.issue_date,
            "maturity_date": self.maturity_date,
            "grace_until": self.grace_until,
            "liquidity": self.liquidity.unwrap_or(Liquidity::AtMaturity),
            "ir_exempt": self.ir_exempt.unwrap_or(self.kind.exempt()),
            "issuer": self.issuer,
        }))
    }
}

/// Price source: `b3_cotahist`, `brapi`, `yahoo`, `coingecko`, `tesouro`, `cvm` or
/// `manual`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum PriceProvider {
    B3Cotahist,
    Brapi,
    Yahoo,
    Coingecko,
    Tesouro,
    Cvm,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateAssetInput {
    /// Trading code (PETR4, HGLG11, IVVB11, AAPL, BTC); empty for fixed income.
    pub ticker: Option<String>,
    /// Name shown in the app (e.g. "Petrobras PN", "CDB Banco X 110% CDI", "Caixa VGBL").
    #[schemars(length(min = 1, max = 160))]
    pub name: String,
    /// Official name in the registry (CVM, B3, insurer), when `name` is a friendlier one.
    #[schemars(length(max = 200))]
    pub official_name: Option<String>,
    /// Asset class.
    pub class: AssetClass,
    /// Exchange or market: `B3`, `NASDAQ`, `NYSE`, `CRYPTO`, `TESOURO`, `CVM`.
    pub market: Option<String>,
    /// Currency (default: the exchange's; without one, BRL for Brazilian classes, USD for US
    /// classes and crypto, the family's base currency for fixed income, pension and other).
    pub currency: Option<String>,
    /// Country (ISO, e.g. BR, US).
    pub country: Option<String>,
    /// ISIN.
    pub isin: Option<String>,
    /// CNPJ (funds).
    pub cnpj: Option<String>,
    /// Security name in Tesouro Direto (e.g. "Tesouro IPCA+ 2035").
    pub treasury_code: Option<String>,
    /// Sector.
    pub sector: Option<String>,
    /// Segment (FIIs: logística, lajes…).
    pub segment: Option<String>,
    /// Preferred price source.
    pub price_provider: Option<PriceProvider>,
    /// Symbol at the source (PETR4.SA, bitcoin…).
    pub provider_symbol: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Fixed income terms (required for `fixed_income`).
    pub fixed_income: Option<FixedIncomeInput>,
    /// Account where the asset is held (see `list_accounts`): the brokerage, the bank of a
    /// CDB/LCI/LCA or bank fund, or a foreign account. Trades use it by default.
    pub account_id: Option<String>,
}

impl ToolInput for CreateAssetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        check_len("name", &self.name, 160)?;
        if let Some(o) = &self.official_name {
            check_len("official_name", o, 200)?;
        }
        check_currency(&self.currency)?;
        if self.class == AssetClass::FixedIncome && self.fixed_income.is_none() {
            return Err(tx("mcp-valida-renda-fixa"));
        }
        let mut v = to_object(self)?;
        if let Some(f) = &self.fixed_income {
            v["fixed_income"] = f.to_value()?;
        }
        Ok(v)
    }
}

/// Field that can be cleared in `update_asset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ClearableAssetField {
    Ticker,
    OfficialName,
    AccountId,
    Market,
    Sector,
    Segment,
    PriceProvider,
    ProviderSymbol,
    Notes,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateAssetInput {
    /// Asset id (see `list_assets`).
    pub id: String,
    /// New name shown in the app.
    pub name: Option<String>,
    /// New official name in the registry (CVM, B3, insurer).
    #[schemars(length(max = 200))]
    pub official_name: Option<String>,
    /// New trading code.
    pub ticker: Option<String>,
    /// New class.
    pub class: Option<AssetClass>,
    /// New exchange or market.
    pub market: Option<String>,
    /// New currency (only if there are no trades or income distributions).
    pub currency: Option<String>,
    /// New sector.
    pub sector: Option<String>,
    /// New segment.
    pub segment: Option<String>,
    /// New price source.
    pub price_provider: Option<PriceProvider>,
    /// New symbol at the source.
    pub provider_symbol: Option<String>,
    /// New notes.
    pub notes: Option<String>,
    /// Account where the asset is held (brokerage, bank or foreign account).
    pub account_id: Option<String>,
    /// CNPJ: of the fund, or of the issuing bank in fixed income (CDB, RDB, LCI…; the asset
    /// score looks up the issuer at the Central Bank by it). Digits or formatted.
    pub cnpj: Option<String>,
    /// Archive.
    pub archived: Option<bool>,
    /// New fixed income terms (replaces all of them).
    pub fixed_income: Option<FixedIncomeInput>,
    /// Pension plan (class `pension` only): plan type and tax regime (progressive or
    /// regressive), insurer and fees. Only the fields given change.
    pub pension: Option<PensionPlanUpdate>,
    /// Fields to clear.
    pub clear: Option<Vec<ClearableAssetField>>,
}

/// Plan type of a pension asset.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PensionPlanTypeInput {
    Vgbl,
    Pgbl,
    /// Pension fund (closed plan, "fundo de pensão").
    Efpc,
}

/// Income tax regime of a pension plan.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PensionTaxRegimeInput {
    /// Progressive table (15% withheld at redemption).
    Progressive,
    /// Regressive table (35% down to 10% by how long each contribution stays).
    Regressive,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PensionPlanUpdate {
    pub plan_type: Option<PensionPlanTypeInput>,
    pub tax_regime: Option<PensionTaxRegimeInput>,
    /// Insurer or pension fund entity.
    pub insurer: Option<String>,
    /// Administration fee, % per year (decimal text, e.g. "1.5").
    pub admin_fee_pct: Option<String>,
    /// Loading fee, % of each contribution (decimal text).
    pub loading_fee_pct: Option<String>,
}

impl ToolInput for UpdateAssetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_currency(&self.currency)?;
        if let Some(c) = &self.cnpj
            && crate::research::cnpj_digits(c).len() != 14
        {
            return Err(tx_args("mcp-valida-cnpj", &[("campo", "cnpj".into())]));
        }
        let mut input = self.clone();
        let clear = input.clear.take().unwrap_or_default();
        let mut v = to_object(&input)?;
        if let Some(f) = &self.fixed_income {
            v["fixed_income"] = f.to_value()?;
        }
        let keys: Vec<&'static str> = clear
            .iter()
            .map(|c| match c {
                ClearableAssetField::Ticker => "ticker",
                ClearableAssetField::OfficialName => "official_name",
                ClearableAssetField::AccountId => "account_id",
                ClearableAssetField::Market => "market",
                ClearableAssetField::Sector => "sector",
                ClearableAssetField::Segment => "segment",
                ClearableAssetField::PriceProvider => "price_provider",
                ClearableAssetField::ProviderSymbol => "provider_symbol",
                ClearableAssetField::Notes => "notes",
            })
            .collect();
        with_clears(v, &keys)
    }
}

// ---------------------------------------------------------------------------
// Recorrências, metas, divisões e transferências

/// Field that can be cleared in `update_recurrence`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ClearableRecurrenceField {
    CategoryId,
    MemberId,
    EndDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateRecurrenceInput {
    /// Recurring item id (see `list_recurring`).
    pub id: String,
    /// New description.
    pub description: Option<String>,
    /// New signed amount in cents (negative = expense).
    pub amount: Option<i64>,
    /// New category (see `list_categories`).
    pub category_id: Option<String>,
    /// New member (see `list_members`).
    pub member_id: Option<String>,
    /// New frequency (together with the repeat fields below, replaces the rule).
    pub freq: Option<Freq>,
    /// Every N periods.
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
    /// New last date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub end_date: Option<String>,
    /// Active (`false` pauses it).
    pub active: Option<bool>,
    /// Reapply to the future planned transactions already generated (default: yes).
    pub update_planned: Option<bool>,
    /// Fields to clear.
    pub clear: Option<Vec<ClearableRecurrenceField>>,
}

impl ToolInput for UpdateRecurrenceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("end_date", &self.end_date)?;
        let schedule = self.interval.is_some()
            || self.by_month_day.is_some()
            || self.by_day.is_some()
            || self.by_month.is_some();
        if schedule && self.freq.is_none() {
            return Err(tx("mcp-valida-freq"));
        }
        let mut out = Map::new();
        out.insert("id".into(), json!(self.id));
        for (k, v) in [
            ("description", self.description.clone().map(|v| json!(v))),
            ("amount", self.amount.map(|v| json!(v))),
            ("category_id", self.category_id.clone().map(|v| json!(v))),
            ("member_id", self.member_id.clone().map(|v| json!(v))),
            ("end_date", self.end_date.clone().map(|v| json!(v))),
            ("active", self.active.map(|v| json!(v))),
            ("update_planned", self.update_planned.map(|v| json!(v))),
        ] {
            if let Some(v) = v {
                out.insert(k.into(), v);
            }
        }
        if let Some(freq) = self.freq {
            let mut spec = Map::new();
            spec.insert(
                "freq".into(),
                json!(match freq {
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
            ] {
                if let Some(v) = v {
                    spec.insert(k.into(), v);
                }
            }
            out.insert("spec".into(), Value::Object(spec));
        }
        let keys: Vec<&'static str> = self
            .clear
            .iter()
            .flatten()
            .map(|c| match c {
                ClearableRecurrenceField::CategoryId => "category_id",
                ClearableRecurrenceField::MemberId => "member_id",
                ClearableRecurrenceField::EndDate => "end_date",
            })
            .collect();
        with_clears(Value::Object(out), &keys)
    }
}

/// Field that can be cleared in `update_goal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ClearableGoalField {
    TargetDate,
    MonthlyContribution,
    Notes,
    /// Back to the plain division (no growth).
    RealReturn,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateGoalInput {
    /// Goal id (see `list_goals`).
    pub id: String,
    /// New name.
    pub name: Option<String>,
    /// New type.
    pub kind: Option<GoalKind>,
    /// New target amount in cents.
    #[schemars(range(min = 1))]
    pub target_amount: Option<i64>,
    /// New target date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub target_date: Option<String>,
    /// Amount already saved outside the accounts, in cents.
    pub initial_amount: Option<i64>,
    /// Planned monthly contribution, in cents.
    pub monthly_contribution: Option<i64>,
    /// Replaces the goal's accounts.
    pub account_ids: Option<Vec<String>>,
    /// Archive.
    pub archived: Option<bool>,
    /// New notes.
    pub notes: Option<String>,
    /// Expected real return a year, above inflation (0.03 = 3%; from -0.05 to 0.15). With it,
    /// the target is in today's money and the monthly contribution and the projected date count
    /// the growth; without it, the plain division (remaining ÷ months). The user's assumption,
    /// not a promise: ask before setting it.
    #[schemars(range(min = -0.05, max = 0.15))]
    pub real_return: Option<f64>,
    /// Fields to clear.
    pub clear: Option<Vec<ClearableGoalField>>,
}

impl ToolInput for UpdateGoalInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("target_date", &self.target_date)?;
        if self.target_amount.is_some_and(|t| t <= 0) {
            return Err(tx_campo("mcp-valida-positivo", "target_amount"));
        }
        let mut input = self.clone();
        let clear = input.clear.take().unwrap_or_default();
        let keys: Vec<&'static str> = clear
            .iter()
            .map(|c| match c {
                ClearableGoalField::TargetDate => "target_date",
                ClearableGoalField::MonthlyContribution => "monthly_contribution",
                ClearableGoalField::Notes => "notes",
                ClearableGoalField::RealReturn => "real_return",
            })
            .collect();
        with_clears(to_object(&input)?, &keys)
    }
}

/// A goal source: an account (its balance, never below zero), one asset position or every
/// position with an asset label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum GoalSourceKind {
    Account,
    Asset,
    AssetTag,
}

/// Position value: `net_redemption` (default: fixed income net of income tax and IOF) or
/// `market`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum GoalValueMode {
    Market,
    NetRedemption,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGoalSourceInput {
    /// Goal id (see `list_goals`).
    pub goal_id: String,
    pub kind: GoalSourceKind,
    /// Account id, asset id or label (id or name, see `list_asset_labels`).
    pub ref_id: String,
    /// Account of the asset, when it is in more than one.
    pub account_id: Option<String>,
    /// Part of the asset for this goal (0.01–1); without it, the whole position.
    #[schemars(range(min = 0.01, max = 1.0))]
    pub share: Option<f64>,
    pub value_mode: Option<GoalValueMode>,
}

impl ToolInput for LinkGoalSourceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("goal_id", &self.goal_id)?;
        check_non_empty("ref_id", &self.ref_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UnlinkGoalSourceInput {
    /// Goal id (see `list_goals`).
    pub goal_id: String,
    pub kind: GoalSourceKind,
    /// As in the goal's `sources`.
    pub ref_id: String,
    /// Account of the asset (without it, the asset in every account).
    pub account_id: Option<String>,
}

impl ToolInput for UnlinkGoalSourceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("goal_id", &self.goal_id)?;
        check_non_empty("ref_id", &self.ref_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SplitPartInput {
    /// Amount of the part in cents, with the same sign as the transaction.
    pub amount: i64,
    /// Category of the part (see `list_categories`).
    pub category_id: Option<String>,
    /// Member of the part (see `list_members`).
    pub member_id: Option<String>,
    /// Note for the part.
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SplitTransactionInput {
    /// Transaction id (see `query_transactions`).
    pub id: String,
    /// Parts (2 to 20) that add up to the transaction amount; an empty list removes the split.
    #[schemars(length(max = 20))]
    pub splits: Vec<SplitPartInput>,
}

impl ToolInput for SplitTransactionInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        if self.splits.len() == 1 || self.splits.len() > 20 {
            return Err(tx("mcp-valida-partes"));
        }
        let parts = self
            .splits
            .iter()
            .map(to_object)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(json!({"id": self.id, "splits": parts}))
    }
}

/// `transfer` transfer (default), `card_payment` card statement payment, `investment`
/// contribution/withdrawal at a brokerage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TransferKind {
    Transfer,
    CardPayment,
    Investment,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateTransferInput {
    /// Source account (see `list_accounts`).
    pub from_account_id: String,
    /// Destination account (to pay a card statement: the card).
    pub to_account_id: String,
    /// Date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Positive amount leaving the source account, in cents of its currency.
    #[schemars(range(min = 1))]
    pub amount: i64,
    /// Amount entering the destination account when the currencies differ, in cents.
    pub to_amount: Option<i64>,
    /// Description.
    pub description: Option<String>,
    /// Type.
    pub kind: Option<TransferKind>,
    /// Status (default `cleared`).
    pub status: Option<TxnStatus>,
    /// Member (see `list_members`).
    pub member_id: Option<String>,
}

impl ToolInput for CreateTransferInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("from_account_id", &self.from_account_id)?;
        check_non_empty("to_account_id", &self.to_account_id)?;
        check_date("date", &Some(self.date.clone()))?;
        if self.amount <= 0 || self.to_amount.is_some_and(|a| a <= 0) {
            return Err(tx("mcp-valida-transferencia-positiva"));
        }
        if self.from_account_id == self.to_account_id {
            return Err(tx("mcp-valida-transferencia-contas"));
        }
        to_object(self)
    }
}

// ---------------------------------------------------------------------------
// Patrimônio físico

/// `real_estate` property, `vehicle` vehicle, `land` land, `equipment` equipment,
/// `collectible` collectible, `other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum PhysicalKind {
    RealEstate,
    Vehicle,
    Land,
    Equipment,
    Collectible,
    Other,
}

/// Estimate between appraisals: `none`, `index` (adjustment by an index), `declining`
/// (compound depreciation), `straight_line` (linear over the useful life), `fipe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum PhysicalValuationMethod {
    None,
    Index,
    Declining,
    StraightLine,
    Fipe,
}

/// Adjustment index: `ipca`, `igpm`, `incc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum PhysicalIndex {
    Ipca,
    Igpm,
    Incc,
}

/// Acquisition cost: `itbi`, `deed` deed, `registry` registration, `brokerage`
/// broker's commission, `other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AcquisitionCostKind {
    Itbi,
    Deed,
    Registry,
    Brokerage,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AcquisitionCostItem {
    /// Cost type.
    pub kind: AcquisitionCostKind,
    /// Amount in cents.
    #[schemars(range(min = 0))]
    pub amount: i64,
    /// Description.
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreatePhysicalAssetInput {
    /// Name (e.g. "Pinheiros apartment", "Corolla 2022").
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    /// Physical asset type.
    pub kind: PhysicalKind,
    /// Purchase date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub purchase_date: String,
    /// Purchase price in cents.
    #[schemars(range(min = 0))]
    pub purchase_price: i64,
    /// Currency (default: the base currency).
    pub currency: Option<String>,
    /// Members who own it (see `list_members`).
    pub member_ids: Option<Vec<String>>,
    /// Acquisition costs (ITBI, deed, registration, broker's commission).
    pub acquisition_costs: Option<Vec<AcquisitionCostItem>>,
    /// Value estimate method (default by type).
    pub valuation_method: Option<PhysicalValuationMethod>,
    /// Adjustment index (with `index`).
    pub index_series: Option<PhysicalIndex>,
    /// Depreciation % per year, decimal as text (with `declining`).
    pub depreciation_rate: Option<String>,
    /// Useful life in years (with `straight_line`).
    #[schemars(range(min = 1, max = 100))]
    pub useful_life_years: Option<u32>,
    /// Residual value as % of cost, decimal as text (with `straight_line`).
    pub residual_pct: Option<String>,
    /// Address (real estate).
    pub address: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Counts toward net worth (default: yes).
    pub include_in_net_worth: Option<bool>,
    /// Financing contracts to link to the physical asset (see `list_loans`).
    pub loan_ids: Option<Vec<String>>,
}

impl ToolInput for CreatePhysicalAssetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        check_len("name", &self.name, 120)?;
        check_date("purchase_date", &Some(self.purchase_date.clone()))?;
        check_currency(&self.currency)?;
        check_decimal("depreciation_rate", &self.depreciation_rate)?;
        check_decimal("residual_pct", &self.residual_pct)?;
        if self.purchase_price < 0 {
            return Err(tx_campo("mcp-valida-nao-negativo", "purchase_price"));
        }
        let mut v = to_object(self)?;
        if let Some(c) = &self.acquisition_costs {
            v["acquisition_costs"] =
                Value::Array(c.iter().map(to_object).collect::<Result<Vec<_>, _>>()?);
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdatePhysicalAssetInput {
    /// Physical asset id (see `list_physical_assets`).
    pub id: String,
    /// New name.
    pub name: Option<String>,
    /// New purchase date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub purchase_date: Option<String>,
    /// New purchase price in cents.
    pub purchase_price: Option<i64>,
    /// New value estimate method.
    pub valuation_method: Option<PhysicalValuationMethod>,
    /// New adjustment index.
    pub index_series: Option<PhysicalIndex>,
    /// New depreciation % per year, decimal as text.
    pub depreciation_rate: Option<String>,
    /// New useful life in years.
    #[schemars(range(min = 1, max = 100))]
    pub useful_life_years: Option<u32>,
    /// New residual value as % of cost.
    pub residual_pct: Option<String>,
    /// New address.
    pub address: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New notes.
    pub notes: Option<String>,
    /// Counts toward net worth.
    pub include_in_net_worth: Option<bool>,
    /// Archive.
    pub archived: Option<bool>,
}

impl ToolInput for UpdatePhysicalAssetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("id", &self.id)?;
        check_date("purchase_date", &self.purchase_date)?;
        check_decimal("depreciation_rate", &self.depreciation_rate)?;
        check_decimal("residual_pct", &self.residual_pct)?;
        with_clears(to_object(self)?, &[])
    }
}

/// Valuation source: `manual` estimate, `appraisal` broker/real estate agency, `report`
/// appraisal report, `fipe`, `index`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ValuationSource {
    Manual,
    Appraisal,
    Report,
    Fipe,
    Index,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddPhysicalValuationInput {
    /// Physical asset id (see `list_physical_assets`).
    pub asset_id: String,
    /// Valuation date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Value in cents.
    #[schemars(range(min = 0))]
    pub value: i64,
    /// Source of the value.
    pub source: ValuationSource,
    /// Notes (who appraised it, reference).
    pub notes: Option<String>,
}

impl ToolInput for AddPhysicalValuationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_date("date", &Some(self.date.clone()))?;
        if self.value < 0 {
            return Err(tx_campo("mcp-valida-nao-negativo", "value"));
        }
        to_object(self)
    }
}

/// `renovation` renovation, `expansion` expansion, `improvement` improvement, `maintenance`
/// maintenance (expense; not part of the cost).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ImprovementCategory {
    Renovation,
    Expansion,
    Improvement,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddPhysicalImprovementInput {
    /// Physical asset id (see `list_physical_assets`).
    pub asset_id: String,
    /// Date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Description (e.g. "Kitchen renovation").
    #[schemars(length(min = 1, max = 200))]
    pub description: String,
    /// Category.
    pub category: ImprovementCategory,
    /// Amount in cents (can be 0 with `project_id` or `create_project`: the spending in the
    /// project's transactions counts).
    #[schemars(range(min = 0))]
    pub amount: i64,
    /// Transaction that paid for it (see `query_transactions`).
    pub txn_id: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Project holding the work's spending (see `list_projects`): the improvement's value
    /// becomes the sum of its transactions.
    pub project_id: Option<String>,
    /// Creates a project named after the improvement, for the same physical asset, and links it
    /// (then use `link_transactions_to_project`).
    pub create_project: Option<bool>,
}

impl ToolInput for AddPhysicalImprovementInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_non_empty("description", &self.description)?;
        check_date("date", &Some(self.date.clone()))?;
        let project = self.project_id.is_some() || self.create_project == Some(true);
        if self.amount < 0 || (self.amount == 0 && !project) {
            return Err(tx("mcp-valida-valor-projeto"));
        }
        to_object(self)
    }
}

// ---------------------------------------------------------------------------
// Projetos

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstimateTaxesInput {
    /// Year (default: the current one, up to today).
    #[schemars(range(min = 2000, max = 2100))]
    pub year: Option<i32>,
}

impl ToolInput for EstimateTaxesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListProjectsInput {
    /// Include completed ones (default: yes).
    pub include_done: Option<bool>,
    /// Only projects for this physical asset (see `list_physical_assets`).
    pub physical_asset_id: Option<String>,
}

impl ToolInput for ListProjectsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// `planned` planned, `active` in progress, `done` done.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ProjectStatus {
    Planned,
    Active,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateProjectInput {
    /// Name (e.g. "Bathroom renovation", "Car bodywork").
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    /// The project's physical asset (property, vehicle; see `list_physical_assets`).
    pub physical_asset_id: Option<String>,
    /// Status (default `active`).
    pub status: Option<ProjectStatus>,
    /// Budget in cents.
    #[schemars(range(min = 0))]
    pub budget: Option<i64>,
    /// Start `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub start_date: Option<String>,
    /// End `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub end_date: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl ToolInput for CreateProjectInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        check_date("start_date", &self.start_date)?;
        check_date("end_date", &self.end_date)?;
        if self.budget.is_some_and(|b| b < 0) {
            return Err(tx_campo("mcp-valida-nao-negativo", "budget"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkProjectInput {
    /// Transactions (up to 500; see `query_transactions`).
    #[schemars(length(min = 1, max = 500))]
    pub transaction_ids: Vec<String>,
    /// Project (see `list_projects`); `null` removes the transactions from the project.
    pub project_id: Option<String>,
}

impl ToolInput for LinkProjectInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.transaction_ids.is_empty() || self.transaction_ids.len() > 500 {
            return Err(tx("mcp-valida-lancamentos-1-500"));
        }
        Ok(json!({"txn_ids": self.transaction_ids, "project_id": self.project_id}))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectEntriesInput {
    /// Project (see `list_projects`).
    pub project_id: String,
}

impl ToolInput for ProjectEntriesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("project_id", &self.project_id)?;
        Ok(json!({ "id": self.project_id.trim() }))
    }
}

/// `income` income, `expense` expense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ProjectFlow {
    Income,
    Expense,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddProjectItemInput {
    /// Project (see `list_projects`).
    pub project_id: String,
    /// Date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Income or expense.
    pub flow: ProjectFlow,
    /// Positive amount in cents (the sign comes from `flow`).
    #[schemars(range(min = 1))]
    pub amount: i64,
    /// Description (e.g. "Bricklayer, paid in cash").
    #[schemars(length(min = 1, max = 200))]
    pub description: String,
    /// Category (see `list_categories`).
    pub category_id: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

impl ToolInput for AddProjectItemInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("project_id", &self.project_id)?;
        check_non_empty("description", &self.description)?;
        check_date("date", &Some(self.date.clone()))?;
        if self.amount <= 0 {
            return Err(tx("mcp-valida-valor-fluxo"));
        }
        to_object(self)
    }
}

// ---------------------------------------------------------------------------
// Sugestões (leitura)

/// App screen: `painel` (Dashboard), `contas` (Accounts), `lancamentos` (Transactions),
/// `orcamento` (Budget), `dividas` (Debts), `carteira` (Portfolio), `ativos` (Assets),
/// `relatorios` (Reports), `agente` (Agent), `ajustes` (Settings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ScreenId {
    Painel,
    Contas,
    Lancamentos,
    Orcamento,
    Dividas,
    Carteira,
    Ativos,
    Relatorios,
    Agente,
    Ajustes,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SuggestionsInput {
    /// Screen currently open, to prioritize its suggestions.
    pub screen: Option<ScreenId>,
    /// Maximum number of suggestions (default 6).
    #[schemars(range(min = 1, max = 20))]
    pub limit: Option<u32>,
}

impl ToolInput for SuggestionsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto<T: serde::de::DeserializeOwned>(input: &impl ToolInput) -> T {
        serde_json::from_value(input.to_api_args().unwrap()).unwrap()
    }

    #[test]
    fn register_inputs_match_core_dtos() {
        let _: crate::household::InstitutionInput = dto(&CreateInstitutionInput {
            name: "XP".into(),
            kind: Some(InstitutionKind::Broker),
            code: None,
            website: Some("https://www.xpi.com.br".into()),
        });
        let p: crate::household::InstitutionPatch = dto(&UpdateInstitutionInput {
            id: "i".into(),
            name: None,
            kind: None,
            code: None,
            website: None,
            clear: Some(vec![ClearableInstitutionField::Website]),
        });
        assert_eq!(p.website, Some(None));

        let card = CardDetailsInput {
            credit_limit: Some(1_000_000),
            closing_day: 3,
            due_day: 10,
            payment_account_id: None,
            network: Some("Mastercard".into()),
            last4: Some("1234".into()),
        };
        let a: crate::accounts::AccountInput = dto(&CreateAccountInput {
            name: "Nubank".into(),
            kind: AccountKind::CreditCard,
            currency: None,
            institution_id: None,
            institution_name: Some("Nubank".into()),
            member_ids: None,
            opening_balance: None,
            opening_date: None,
            include_in_net_worth: None,
            notes: None,
            card: Some(card.clone()),
            overdraft: None,
            branch: None,
            account_number: None,
            retirement_plan: None,
            retirement_coverage: None,
        });
        assert_eq!(a.card.unwrap().due_day, 10);
        assert_eq!(a.institution_name.as_deref(), Some("Nubank"));
        let no_card = CreateAccountInput {
            name: "Nu".into(),
            kind: AccountKind::CreditCard,
            currency: None,
            institution_id: None,
            institution_name: None,
            member_ids: None,
            opening_balance: None,
            opening_date: None,
            include_in_net_worth: None,
            notes: None,
            card: None,
            overdraft: None,
            branch: None,
            account_number: None,
            retirement_plan: None,
            retirement_coverage: None,
        };
        assert!(no_card.to_api_args().is_err(), "card without data");
        let bad_last4 = CreateAccountInput {
            card: Some(CardDetailsInput {
                last4: Some("5555444433332222".into()),
                ..card.clone()
            }),
            ..no_card.clone()
        };
        assert!(bad_last4.to_api_args().is_err(), "never the full number");
        let up: crate::accounts::AccountPatch = dto(&UpdateAccountInput {
            id: "a".into(),
            name: None,
            institution_id: None,
            member_ids: None,
            opening_balance: Some(10),
            opening_date: None,
            include_in_net_worth: None,
            archived: None,
            notes: None,
            card: Some(card),
            overdraft: None,
            branch: None,
            account_number: None,
            clear: Some(vec![ClearableAccountField::InstitutionId]),
        });
        assert_eq!(up.institution_id, Some(None));
        assert_eq!(up.card.unwrap().closing_day, 3);

        let _: crate::categories::CategoryInput = dto(&CreateCategoryInput {
            name: "Pet".into(),
            kind: CategoryKind::Expense,
            parent_id: None,
            is_essential: Some(false),
            spend_group: Some(SpendGroup::Future),
        });
        let c: crate::categories::CategoryPatch = dto(&UpdateCategoryInput {
            id: "c".into(),
            name: Some("Pets".into()),
            parent_id: None,
            move_to_root: Some(true),
            is_essential: None,
            spend_group: Some(SpendGroup::NonEssential),
            archived: None,
        });
        assert_eq!(c.parent_id, Some(None));
        assert_eq!(
            c.spend_group,
            Some(crate::categories::SpendGroup::NonEssential)
        );

        let fi = FixedIncomeInput {
            kind: FixedIncomeKind::Lci,
            indexer: Indexer::Cdi,
            rate: "95".into(),
            issue_date: Some("2026-01-02".into()),
            maturity_date: Some("2028-01-02".into()),
            grace_until: None,
            liquidity: None,
            ir_exempt: None,
            issuer: Some("Banco X".into()),
        };
        let asset: crate::assets::AssetInput = dto(&CreateAssetInput {
            ticker: None,
            name: "LCI Banco X".into(),
            official_name: None,
            account_id: None,
            class: AssetClass::FixedIncome,
            market: None,
            currency: None,
            country: None,
            isin: None,
            cnpj: None,
            treasury_code: None,
            sector: None,
            segment: None,
            price_provider: None,
            provider_symbol: None,
            notes: None,
            fixed_income: Some(fi.clone()),
        });
        assert!(asset.fixed_income.unwrap().ir_exempt, "LCI is exempt");
        let stock: crate::assets::AssetInput = dto(&CreateAssetInput {
            ticker: Some("PETR4".into()),
            name: "Petrobras PN".into(),
            official_name: None,
            account_id: None,
            class: AssetClass::StockBr,
            market: Some("B3".into()),
            currency: None,
            country: None,
            isin: None,
            cnpj: None,
            treasury_code: None,
            sector: None,
            segment: None,
            price_provider: Some(PriceProvider::B3Cotahist),
            provider_symbol: None,
            notes: None,
            fixed_income: None,
        });
        assert_eq!(stock.price_provider.as_deref(), Some("b3_cotahist"));
        let ap: crate::assets::AssetPatch = dto(&UpdateAssetInput {
            id: "x".into(),
            name: None,
            official_name: None,
            account_id: None,
            cnpj: None,
            ticker: None,
            class: None,
            market: None,
            currency: None,
            sector: None,
            segment: None,
            price_provider: Some(PriceProvider::Yahoo),
            provider_symbol: Some("PETR4.SA".into()),
            notes: None,
            archived: None,
            fixed_income: Some(fi),
            pension: Some(PensionPlanUpdate {
                plan_type: None,
                tax_regime: Some(PensionTaxRegimeInput::Progressive),
                insurer: None,
                admin_fee_pct: None,
                loading_fee_pct: None,
            }),
            clear: Some(vec![ClearableAssetField::Sector]),
        });
        assert_eq!(ap.sector, Some(None));
        // The plan's regime reaches the core (the rest of the plan stays as it was).
        let pp = ap.pension.expect("plan");
        assert_eq!(pp.tax_regime, Some(crate::pension::TaxRegime::Progressive));
        assert!(pp.plan_type.is_none());

        let r: crate::recurrence::RecurrencePatch = dto(&UpdateRecurrenceInput {
            id: "r".into(),
            description: None,
            amount: Some(-210000),
            category_id: None,
            member_id: None,
            freq: Some(Freq::Monthly),
            interval: None,
            by_month_day: Some(10),
            by_day: None,
            by_month: None,
            end_date: None,
            active: None,
            update_planned: None,
            clear: Some(vec![ClearableRecurrenceField::EndDate]),
        });
        assert_eq!(r.spec.unwrap().by_month_day, Some(10));
        assert_eq!(r.end_date, Some(None));

        let g: crate::goals::GoalPatch = dto(&UpdateGoalInput {
            id: "g".into(),
            name: None,
            kind: None,
            target_amount: Some(500000),
            target_date: None,
            initial_amount: None,
            monthly_contribution: None,
            account_ids: None,
            archived: None,
            notes: None,
            real_return: None,
            clear: Some(vec![
                ClearableGoalField::MonthlyContribution,
                ClearableGoalField::RealReturn,
            ]),
        });
        assert_eq!(g.monthly_contribution, Some(None));

        let s: crate::transactions::TxnPatch = dto(&SplitTransactionInput {
            id: "t".into(),
            splits: vec![
                SplitPartInput {
                    amount: -600,
                    category_id: Some("a".into()),
                    member_id: None,
                    notes: None,
                },
                SplitPartInput {
                    amount: -400,
                    category_id: Some("b".into()),
                    member_id: None,
                    notes: None,
                },
            ],
        });
        assert_eq!(s.splits.unwrap().len(), 2);
        let _: crate::transactions::TransferInput = dto(&CreateTransferInput {
            from_account_id: "a".into(),
            to_account_id: "b".into(),
            date: "2026-10-05".into(),
            amount: 150000,
            to_amount: None,
            description: Some("Fatura".into()),
            kind: Some(TransferKind::CardPayment),
            status: None,
            member_id: None,
        });

        let _: crate::physical::PhysicalAssetInput = dto(&CreatePhysicalAssetInput {
            name: "Apartamento".into(),
            kind: PhysicalKind::RealEstate,
            purchase_date: "2020-05-10".into(),
            purchase_price: 50_000_000,
            currency: None,
            member_ids: None,
            acquisition_costs: Some(vec![AcquisitionCostItem {
                kind: AcquisitionCostKind::Itbi,
                amount: 1_500_000,
                description: None,
            }]),
            valuation_method: Some(PhysicalValuationMethod::Index),
            index_series: Some(PhysicalIndex::Ipca),
            depreciation_rate: None,
            useful_life_years: None,
            residual_pct: None,
            address: None,
            description: None,
            notes: None,
            include_in_net_worth: None,
            loan_ids: None,
        });
        let _: crate::physical::PhysicalAssetPatch = dto(&UpdatePhysicalAssetInput {
            id: "p".into(),
            name: None,
            purchase_date: None,
            purchase_price: None,
            valuation_method: Some(PhysicalValuationMethod::Declining),
            index_series: None,
            depreciation_rate: Some("10".into()),
            useful_life_years: None,
            residual_pct: None,
            address: None,
            description: None,
            notes: None,
            include_in_net_worth: None,
            archived: None,
        });
        let _: crate::physical::PhysicalValuationInput = dto(&AddPhysicalValuationInput {
            asset_id: "p".into(),
            date: "2026-10-01".into(),
            value: 60_000_000,
            source: ValuationSource::Appraisal,
            notes: None,
        });
        let _: crate::physical::PhysicalImprovementInput = dto(&AddPhysicalImprovementInput {
            asset_id: "p".into(),
            date: "2026-10-01".into(),
            description: "Reforma".into(),
            category: ImprovementCategory::Renovation,
            amount: 2_000_000,
            txn_id: None,
            notes: None,
            project_id: Some("p1".into()),
            create_project: None,
        });
        let _: crate::projects::ProjectInput = dto(&CreateProjectInput {
            name: "Reforma da cozinha".into(),
            physical_asset_id: Some("p".into()),
            status: Some(ProjectStatus::Planned),
            budget: Some(1_000),
            start_date: Some("2026-10-01".into()),
            end_date: None,
            notes: None,
        });
        let _: crate::projects::ProjectItemInput = dto(&AddProjectItemInput {
            project_id: "p1".into(),
            date: "2026-10-01".into(),
            flow: ProjectFlow::Expense,
            amount: 1_000,
            description: "Pedreiro".into(),
            category_id: Some("c".into()),
            notes: None,
        });
        let _: crate::projects::ProjectLinkArgs = serde_json::from_value(
            LinkProjectInput {
                transaction_ids: vec!["t".into()],
                project_id: None,
            }
            .to_api_args()
            .unwrap(),
        )
        .unwrap();
    }
    /// Issuer CNPJ in fixed income through update_asset (the score finds the bank at the Banco
    /// Central).
    #[test]
    fn update_asset_takes_the_issuer_cnpj() {
        let base = UpdateAssetInput {
            id: "x".into(),
            cnpj: Some("30.680.829/0001-43".into()),
            ..serde_json::from_value(json!({"id": "x"})).unwrap()
        };
        let v = base.to_api_args().unwrap();
        assert_eq!(v["cnpj"], "30.680.829/0001-43");
        let bad = UpdateAssetInput {
            cnpj: Some("123".into()),
            ..base
        };
        assert!(bad.to_api_args().is_err());
    }
}

/// Property whose IPTU (Brazilian municipal property tax) to show.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetIptuInput {
    /// Property or land id (from `list_physical_assets`).
    pub asset_id: String,
}

impl ToolInput for GetIptuInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IptuInstallmentItem {
    /// 0 = single payment (cota única); 1, 2, … = installments.
    #[schemars(range(min = 0, max = 24))]
    pub number: u32,
    /// Due date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub due_date: String,
    /// Amount in cents.
    #[schemars(range(min = 1))]
    pub amount: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IptuPlanInput {
    /// Single payment (cota única).
    Single,
    /// In installments.
    Installments,
}

/// One year's IPTU bill (replaces the one for the same property and year). Amounts in cents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordIptuInput {
    /// Property or land id (from `list_physical_assets`).
    pub asset_id: String,
    /// Year (fiscal year) of the bill.
    #[schemars(range(min = 1990, max = 2100))]
    pub year: i32,
    /// City hall registration number (inscrição imobiliária, indicação fiscal, SQL).
    pub registration: Option<String>,
    /// Municipality.
    pub city: Option<String>,
    /// Land area in m², decimal as text.
    pub land_area: Option<String>,
    /// Built area in m², decimal as text.
    pub built_area: Option<String>,
    /// Assessed value of the land (valor venal).
    pub land_value: Option<i64>,
    /// Assessed value of the building.
    pub building_value: Option<i64>,
    /// Land tax.
    pub land_tax: Option<i64>,
    /// Building tax.
    pub building_tax: Option<i64>,
    /// Total taxes (without it: land + building).
    pub taxes_total: Option<i64>,
    /// Public lighting fee.
    pub lighting_fee: Option<i64>,
    /// Waste collection fee.
    pub waste_fee: Option<i64>,
    /// Other fees.
    pub other_fees: Option<i64>,
    /// Discount.
    pub discount: Option<i64>,
    /// Good payer discount.
    pub good_payer_discount: Option<i64>,
    /// Total due.
    #[schemars(range(min = 0))]
    pub total_due: i64,
    /// Chosen payment plan, if the user said.
    pub plan: Option<IptuPlanInput>,
    /// Single payment (number 0) and installments, with due date and amount.
    pub installments: Option<Vec<IptuInstallmentItem>>,
    /// Notes.
    pub notes: Option<String>,
}

impl ToolInput for RecordIptuInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        for i in self.installments.iter().flatten() {
            check_date("installments.due_date", &Some(i.due_date.clone()))?;
        }
        let mut v = to_object(self)?;
        v["installments"] = Value::Array(
            self.installments
                .iter()
                .flatten()
                .map(to_object)
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(v)
    }
}
