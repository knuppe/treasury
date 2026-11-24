//! Entradas das ferramentas MCP da fase 2 (carteira, ativos, índices e câmbio).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::inputs::{
    DATE_PATTERN, MONTH_PATTERN, ToolInput, check_date, check_month, check_non_empty, to_object, tx,
};

/// Deflator for the real return: `ipca` (default, Brazil's official inflation), `igpm`
/// (IGP-M, FGV), `us_cpi`
/// (US inflation, for USD assets), `m2_br`/`m2_us` (money supply growth), `country`
/// (inflation of the family's reference country, from the country catalog; IPCA in Brazil)
/// or `none`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Deflator {
    Ipca,
    Igpm,
    UsCpi,
    M2Br,
    M2Us,
    None,
    Country,
}

/// Cost method for the sales ledger: `average` (average price, Brazil's income tax),
/// `fifo` (oldest lots first) or `uk_pool` (same day, next 30 days, then the section 104 pool).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum CostMethod {
    Average,
    Fifo,
    UkPool,
}

/// Asset class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AssetClass {
    StockBr,
    Fii,
    EtfBr,
    Bdr,
    FixedIncome,
    TreasuryBr,
    Fund,
    StockUs,
    EtfUs,
    Reit,
    Crypto,
    Pension,
    Other,
    StockIntl,
    EtfIntl,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PortfolioInput {
    /// Valuation date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub as_of: Option<String>,
    /// Deflator for the real XIRR (default: `ipca`).
    pub deflator: Option<Deflator>,
    /// Only these brokerage accounts.
    pub account_ids: Option<Vec<String>>,
    /// One row per account instead of summing the asset across accounts.
    pub by_account: Option<bool>,
    /// Include positions already closed (with realized result).
    pub include_closed: Option<bool>,
}

impl ToolInput for PortfolioInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("as_of", &self.as_of)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PositionInput {
    /// Asset id (see `list_assets` or `get_portfolio`).
    pub asset_id: String,
    /// Only this account (default: all).
    pub account_id: Option<String>,
    /// Valuation date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub as_of: Option<String>,
    /// Deflator for the real return (default: `ipca`).
    pub deflator: Option<Deflator>,
}

impl ToolInput for PositionInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_date("as_of", &self.as_of)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReturnsInput {
    /// First month `YYYY-MM` (default: month of the first trade).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "from", alias = "start_month")]
    pub from_month: Option<String>,
    /// Last month `YYYY-MM` (default: current month).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "to", alias = "end_month")]
    pub to_month: Option<String>,
    /// Deflator for the real return (default: `ipca`).
    pub deflator: Option<Deflator>,
    /// Only one asset.
    pub asset_id: Option<String>,
    /// Only one class.
    pub class: Option<AssetClass>,
    /// How returns are chained: `monthly` (Modified Dietz, the default) or `daily` (each
    /// day's return chained; slower). Default: the user's advanced setting.
    pub method: Option<TwrMethod>,
}

/// Time-weighted return method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TwrMethod {
    Monthly,
    Daily,
}

impl ToolInput for ReturnsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("from_month", &self.from_month)?;
        check_month("to_month", &self.to_month)?;
        to_object(self)
    }
}

/// Performance bridge: from the start value to the end value, in money.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BridgeInput {
    /// First month `YYYY-MM` (default: month of the first trade).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "from", alias = "start_month")]
    pub from_month: Option<String>,
    /// Last month `YYYY-MM` (default: current month).
    #[schemars(pattern(MONTH_PATTERN))]
    #[serde(alias = "to", alias = "end_month")]
    pub to_month: Option<String>,
    /// Deflator for the inflation loss (default: `ipca`).
    pub deflator: Option<Deflator>,
    /// Only one asset.
    pub asset_id: Option<String>,
    /// Only one class.
    pub class: Option<AssetClass>,
}

impl ToolInput for BridgeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("from_month", &self.from_month)?;
        check_month("to_month", &self.to_month)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListAssetsInput {
    /// Searches ticker and name.
    pub text: Option<String>,
    /// Only these classes.
    pub classes: Option<Vec<AssetClass>>,
    /// Only assets with trades (default: no).
    pub held_only: Option<bool>,
    /// Include archived assets (default: no).
    pub include_archived: Option<bool>,
    /// Position today: "open" (in the portfolio; pension plans without contributions too),
    /// "closed" (closed out) or "traded" (with trades). Without it, all registered assets
    /// (including the watchlist).
    pub position: Option<String>,
}

impl ToolInput for ListAssetsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(p) = &self.position
            && !matches!(p.as_str(), "open" | "closed" | "traded")
        {
            return Err(tx("mcp-valida-position"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListTradesInput {
    /// Asset id (see `list_assets`).
    pub asset_id: Option<String>,
    /// Brokerage account id (see `list_accounts`).
    pub account_id: Option<String>,
    /// Start `YYYY-MM-DD` (inclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "from", alias = "start_date", alias = "from_date")]
    pub date_from: Option<String>,
    /// End `YYYY-MM-DD` (inclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "to", alias = "end_date", alias = "to_date")]
    pub date_to: Option<String>,
    /// Maximum number of trades (default 100, max 500).
    pub limit: Option<i64>,
}

impl ToolInput for ListTradesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("date_from", &self.date_from)?;
        check_date("date_to", &self.date_to)?;
        let mut v = to_object(self)?;
        v["limit"] = json!(self.limit.unwrap_or(100).clamp(1, 500));
        Ok(v)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TradeSummaryInput {
    /// Asset id (see `list_assets`).
    pub asset_id: String,
    /// Only this brokerage account (default: all).
    pub account_id: Option<String>,
}

impl ToolInput for TradeSummaryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListIncomeInput {
    /// Asset id (see `list_assets`).
    pub asset_id: Option<String>,
    /// Payment date from `YYYY-MM-DD` on.
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// End `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
}

impl ToolInput for ListIncomeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DividendCalendarInput {
    /// Start `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// End `YYYY-MM-DD` (default: 60 days from now).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
    /// Only portfolio assets (default: yes).
    pub only_mine: Option<bool>,
}

/// Life event in the retirement simulation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetirementEventInput {
    /// Age at which it happens.
    #[schemars(range(min = 16, max = 110))]
    pub age: u32,
    /// In today's cents: positive is an inflow (inheritance, property sale), negative an outflow
    /// (college).
    pub amount: i64,
    /// What it is (e.g. "inheritance", "Ana's college").
    pub label: String,
}

/// Retirement simulation. Omitted fields use the plan saved by the user.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetirementInput {
    /// Current age (default: the saved plan's, else the plan member's from their birth date).
    #[schemars(range(min = 16, max = 100))]
    pub current_age: Option<u32>,
    /// Age at which contributions stop and spending starts (default 65).
    #[schemars(range(min = 16, max = 100))]
    pub retirement_age: Option<u32>,
    /// Life expectancy (default 95).
    #[schemars(range(min = 17, max = 110))]
    pub life_expectancy: Option<u32>,
    /// Invested wealth today in cents (default: market value of the portfolio).
    #[schemars(range(min = 0))]
    pub current_wealth: Option<i64>,
    /// Monthly contribution until retirement, in today's cents.
    #[schemars(range(min = 0))]
    pub monthly_contribution: Option<i64>,
    /// Desired monthly spending in retirement, in today's cents.
    #[schemars(range(min = 0))]
    pub monthly_spending: Option<i64>,
    /// Estimated monthly INSS (public pension), in today's cents, and the age at which it starts.
    #[schemars(range(min = 0))]
    pub inss_monthly: Option<i64>,
    /// Age at which INSS starts (default: the retirement age).
    pub inss_age: Option<u32>,
    /// Expected real return and volatility, annual fractions (default 0.04 and 0.10).
    #[schemars(range(min = -0.2, max = 0.3))]
    pub real_return: Option<f64>,
    /// Annual volatility of the return, as a fraction (default 0.10).
    #[schemars(range(min = 0.0, max = 0.6))]
    pub volatility: Option<f64>,
    /// Life events (large inflows and outflows at a given age).
    pub events: Option<Vec<RetirementEventInput>>,
    /// Compute INSS by the reform rules (applies when `inss_monthly` is not given).
    pub inss_calc: Option<InssCalcInput>,
    /// Compute US Social Security (family in the US; applies when `inss_monthly` is not given).
    pub us_ssa_calc: Option<UsSsaCalcInput>,
    /// Compute the Portuguese old-age pension (family in Portugal; applies when
    /// `inss_monthly` is not given).
    pub pt_pension_calc: Option<PtPensionCalcInput>,
}

/// Data to compute US Social Security: the PIA from the SSA statement, or the AIME.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsSsaCalcInput {
    /// Birth date (YYYY-MM-DD); without it, the current age is used.
    pub birth_date: Option<String>,
    /// Monthly benefit at full retirement age (PIA), in today's cents.
    #[schemars(range(min = 0))]
    pub pia: Option<i64>,
    /// Average indexed monthly earnings (AIME), in cents, when the PIA is not known.
    #[schemars(range(min = 0))]
    pub aime: Option<i64>,
    /// Credits (quarters of coverage) earned so far (default 40).
    #[schemars(range(min = 0, max = 200))]
    pub credits: Option<u32>,
    /// Age until which the person keeps working (default: the retirement age).
    #[schemars(range(min = 16, max = 100))]
    pub work_until_age: Option<u32>,
}

/// Data to compute the Portuguese old-age pension (general scheme).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PtPensionCalcInput {
    /// Birth date (YYYY-MM-DD); without it, the current age is used.
    pub birth_date: Option<String>,
    /// Calendar years with registered earnings up to today (fractions accepted).
    #[schemars(range(min = 0.0, max = 60.0))]
    pub contribution_years: f64,
    /// Reference earnings: career earnings ÷ (14 × years), in today's cents.
    #[schemars(range(min = 0))]
    pub reference_salary: i64,
    /// Age until which contributions continue (default: the retirement age).
    #[schemars(range(min = 16, max = 100))]
    pub contribute_until_age: Option<u32>,
    /// Started contributing before 17 (very long career with 46 years).
    pub started_before_17: Option<bool>,
}

/// Data to compute INSS by the rules (EC 103/2019).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InssCalcInput {
    /// "male" or "female".
    #[schemars(extend("enum" = ["male", "female"]))]
    pub sex: String,
    /// Birth date (YYYY-MM-DD); without it, the current age is used.
    pub birth_date: Option<String>,
    /// Contribution time up to today, in years (fractions accepted).
    #[schemars(range(min = 0.0, max = 60.0))]
    pub contribution_years: f64,
    /// Contribution time on 13/11/2019 (default: estimated assuming uninterrupted
    /// contributions).
    #[schemars(range(min = 0.0, max = 60.0))]
    pub contribution_at_reform: Option<f64>,
    /// Monthly average of the contribution salaries since 07/1994, in cents.
    #[schemars(range(min = 0))]
    pub average_salary: i64,
    /// Age until which contributions continue (default: the retirement age).
    #[schemars(range(min = 16, max = 100))]
    pub contribute_until_age: Option<u32>,
}

impl InssCalcInput {
    fn check(&self) -> Result<(), String> {
        if !matches!(self.sex.as_str(), "male" | "female") {
            return Err(tx("mcp-valida-sexo"));
        }
        Ok(())
    }
}

/// Standalone INSS estimate by the rules (without the simulation).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InssEstimateInput {
    #[serde(flatten)]
    pub inss: InssCalcInput,
    /// Current age (default: the saved plan's, else the plan member's from their birth date;
    /// not needed with `birth_date`).
    #[schemars(range(min = 16, max = 100))]
    pub current_age: Option<u32>,
    /// Age at which work stops (default: the saved plan's, or 65).
    #[schemars(range(min = 16, max = 100))]
    pub retirement_age: Option<u32>,
}

impl ToolInput for InssEstimateInput {
    fn to_api_args(&self) -> Result<Value, String> {
        self.inss.check()?;
        Ok(serde_json::json!({
            "input": serde_json::to_value(&self.inss).map_err(|e| e.to_string())?,
            "current_age": self.current_age,
            "retirement_age": self.retirement_age,
        }))
    }
}

/// Portfolio risk.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PortfolioRiskInput {
    /// Full months analyzed (default 36).
    #[schemars(range(min = 12, max = 120))]
    pub months: Option<u32>,
}

impl ToolInput for PortfolioRiskInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

impl ToolInput for RetirementInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if let Some(i) = &self.inss_calc {
            i.check()?;
        }
        to_object(self)
    }
}

/// Cost of fund and pension plan fees.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FeeCostInput {
    /// Assumed gross real return, annual fraction (default 0.04 = 4% above IPCA).
    #[schemars(range(min = -0.5, max = 0.5))]
    pub real_return: Option<f64>,
    /// Management fee in the alternative scenario, annual fraction (default 0.005).
    #[schemars(range(min = 0.0, max = 0.2))]
    pub alternative_fee: Option<f64>,
    /// Expected excess return over the benchmark, for the performance fee (default 0).
    #[schemars(range(min = 0.0, max = 0.5))]
    pub excess_return: Option<f64>,
    /// Total monthly contribution in cents (split across the funds and plans).
    #[schemars(range(min = 0))]
    pub monthly_contribution: Option<i64>,
    /// Horizons in years (default 10, 20 and 30).
    pub horizons: Option<Vec<u32>>,
}

impl ToolInput for FeeCostInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// How a fixed-income offer earns: `cdi_pct` (rate = % of CDI, e.g. 110), `cdi_plus` (CDI +
/// rate % p.a.), `selic_plus` (Tesouro Selic: Selic + rate % p.a., usually near 0), `prefixed`
/// (rate % p.a.), `ipca_plus` (IPCA + rate % p.a.) or `savings` (poupança rule; rate ignored).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum OfferIndex {
    CdiPct,
    CdiPlus,
    SelicPlus,
    Prefixed,
    IpcaPlus,
    Savings,
}

/// One fixed-income offer the user received.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FixedIncomeOfferInput {
    /// Name to show ("Bank A CDB"); default: type and rate.
    pub label: Option<String>,
    pub kind: super::inputs_register::FixedIncomeKind,
    pub index: OfferIndex,
    /// % of CDI for `cdi_pct`, otherwise % p.a. (0 for savings).
    #[serde(default)]
    pub rate: f64,
    /// Redemption date (`YYYY-MM-DD`); or `months`.
    #[schemars(regex(pattern = DATE_PATTERN))]
    pub end: Option<String>,
    /// Term in months from the start date, when there is no `end`.
    #[schemars(range(min = 1, max = 600))]
    pub months: Option<u32>,
    /// Exempt from income tax and IOF; default by type (LCI, LCA, CRI, CRA, incentivized
    /// debenture and poupança).
    pub ir_exempt: Option<bool>,
    /// Custody or management fee, annual fraction; default: B3's 0.002 on Tesouro, 0 otherwise.
    #[schemars(range(min = 0.0, max = 0.05))]
    pub custody: Option<f64>,
}

/// Brazilian fixed-income offers to compare side by side.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompareFixedIncomeInput {
    /// Amount invested in cents (BRL).
    #[schemars(range(min = 1))]
    pub amount: i64,
    /// Investment date (`YYYY-MM-DD`, default today).
    #[schemars(regex(pattern = DATE_PATTERN))]
    pub start: Option<String>,
    /// 1 to 12 offers, compared in the given order.
    pub offers: Vec<FixedIncomeOfferInput>,
    /// CDI assumption, annual fraction (default: the vault's latest CDI).
    #[schemars(range(min = 0.0, max = 1.0))]
    pub cdi: Option<f64>,
    /// Selic target, annual fraction, for the poupança rule (default: the vault's).
    #[schemars(range(min = 0.0, max = 1.0))]
    pub selic: Option<f64>,
    /// Expected IPCA, annual fraction (default: Focus projection or the last 12 months).
    #[schemars(range(min = -0.2, max = 1.0))]
    pub ipca: Option<f64>,
    /// TR, monthly fraction (default: the vault's latest).
    #[schemars(range(min = 0.0, max = 0.05))]
    pub tr: Option<f64>,
}

impl ToolInput for CompareFixedIncomeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("start", &self.start)?;
        if self.offers.is_empty() || self.offers.len() > 12 {
            return Err(tx("mcp-valida-ofertas-renda-fixa"));
        }
        for o in &self.offers {
            check_date("end", &o.end)?;
            if o.end.is_none() && o.months.is_none() {
                return Err(tx("mcp-valida-prazo-renda-fixa"));
            }
        }
        to_object(self)
    }
}

/// Zero-coupon title paying at maturity: `prefixed` or `ipca_plus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum SaleIndex {
    Prefixed,
    IpcaPlus,
}

/// Selling a fixed-rate or IPCA+ title before maturity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FixedIncomeSaleInput {
    /// Amount invested in cents.
    #[schemars(range(min = 1))]
    pub amount: i64,
    /// Investment date (`YYYY-MM-DD`).
    #[schemars(regex(pattern = DATE_PATTERN))]
    pub start: String,
    /// Maturity date (`YYYY-MM-DD`).
    #[schemars(regex(pattern = DATE_PATTERN))]
    pub maturity: String,
    /// Sale date (`YYYY-MM-DD`, default today).
    #[schemars(regex(pattern = DATE_PATTERN))]
    pub sale_date: Option<String>,
    pub index: SaleIndex,
    /// Contracted rate, % p.a. (fixed rate, or the real rate of IPCA+).
    pub contracted_rate: f64,
    /// Today's market rate for the same maturity, % p.a.
    pub market_rate: f64,
    /// Exempt from income tax (default false).
    pub ir_exempt: Option<bool>,
    /// Expected IPCA, annual fraction (IPCA+ only; default: the vault's).
    #[schemars(range(min = -0.2, max = 1.0))]
    pub ipca: Option<f64>,
}

impl ToolInput for FixedIncomeSaleInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("start", &Some(self.start.clone()))?;
        check_date("maturity", &Some(self.maturity.clone()))?;
        check_date("sale_date", &self.sale_date)?;
        to_object(self)
    }
}

/// Income distributions to confirm.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PendingIncomeInput {
    /// Earliest data-com `YYYY-MM-DD` (default: one year ago).
    #[schemars(pattern(DATE_PATTERN))]
    pub since: Option<String>,
    /// Only one account (see `list_accounts`).
    pub account_id: Option<String>,
}

/// Income ledger.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IncomeLedgerInput {
    /// Only this asset (see `list_assets`).
    pub asset_id: Option<String>,
    /// Only this account (see `list_accounts`).
    pub account_id: Option<String>,
    /// Payment date from `YYYY-MM-DD` on.
    #[schemars(pattern(DATE_PATTERN))]
    pub from: Option<String>,
    /// Payment date until `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub to: Option<String>,
    /// Leave out the distributions that match (totals, fixes and upcoming only).
    pub only_problems: Option<bool>,
}

impl ToolInput for IncomeLedgerInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        to_object(self)
    }
}

impl ToolInput for PendingIncomeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("since", &self.since)?;
        to_object(self)
    }
}

impl ToolInput for DividendCalendarInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        let today = crate::time::today_local();
        let from = self
            .from
            .clone()
            .unwrap_or_else(|| crate::time::fmt_date(today));
        let to = self
            .to
            .clone()
            .unwrap_or_else(|| crate::time::fmt_date(today + chrono::Duration::days(60)));
        Ok(json!({ "from": from, "to": to, "only_mine": self.only_mine.unwrap_or(true) }))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RebalanceInput {
    /// New contribution to allocate, in cents of the family's base currency (default: 0 =
    /// deviation only).
    pub contribution: Option<i64>,
}

impl ToolInput for RebalanceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.contribution.is_some_and(|c| c < 0) {
            return Err(tx("mcp-valida-aporte-negativo"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PassiveIncomeInput {
    /// Monthly income goal in cents of the family's base currency (to compute the required
    /// wealth).
    pub target_monthly: Option<i64>,
}

impl ToolInput for PassiveIncomeInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// Index series: `ipca`, `igpm`, `cdi`, `selic`, `selic_meta`, `tr`, `m2_br`, `us_cpi`,
/// `us_m2`, `fed_funds`, `ibov`, `sp500`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum SeriesId {
    Ipca,
    Igpm,
    Cdi,
    Selic,
    SelicMeta,
    Tr,
    M2Br,
    UsCpi,
    UsM2,
    FedFunds,
    Ibov,
    Sp500,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexSeriesInput {
    pub id: SeriesId,
    /// Start `YYYY-MM-DD` (default: 24 months before the end).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// End `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
}

impl ToolInput for IndexSeriesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InflationInput {
    /// Deflator for the real return (default: `ipca`).
    pub deflator: Deflator,
    /// Start `YYYY-MM-DD` (inclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: String,
    /// End `YYYY-MM-DD` (exclusive).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: String,
}

impl ToolInput for InflationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &Some(self.from.clone()))?;
        check_date("to", &Some(self.to.clone()))?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConvertInput {
    /// Amount in cents (smallest unit) of the source currency.
    pub amount: i64,
    /// Source currency (ISO 4217, e.g. `USD`).
    pub from: String,
    /// Target currency (e.g. `BRL`).
    pub to: String,
    /// Exchange rate date `YYYY-MM-DD` (default: today; uses the latest PTAX up to that date).
    #[schemars(pattern(DATE_PATTERN))]
    pub date: Option<String>,
}

impl ToolInput for ConvertInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("date", &self.date)?;
        check_non_empty("from", &self.from)?;
        check_non_empty("to", &self.to)?;
        let date = self
            .date
            .clone()
            .unwrap_or_else(|| crate::time::fmt_date(crate::time::today_local()));
        Ok(
            json!({ "amount": self.amount, "from": self.from.trim().to_uppercase(), "to": self.to.trim().to_uppercase(), "date": date }),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaxMonthInput {
    /// Tax calculation month `YYYY-MM`.
    #[schemars(pattern(MONTH_PATTERN))]
    pub month: String,
    /// Member (CPF) for the tax calculation (see `list_members`); default: all accounts.
    pub member_id: Option<String>,
}

impl ToolInput for TaxMonthInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_month("month", &Some(self.month.clone()))?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaxYearInput {
    /// Calendar year (e.g. 2026).
    pub year: i32,
    /// Member (CPF) for the tax calculation; default: all accounts.
    pub member_id: Option<String>,
}

impl ToolInput for TaxYearInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if !(2000..=2100).contains(&self.year) {
            return Err(tx("mcp-valida-ano"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListLoansInput {
    /// Include archived (paid-off) contracts.
    pub include_archived: Option<bool>,
}

impl ToolInput for ListLoansInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(json!({ "include_archived": self.include_archived.unwrap_or(false) }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoanIdInput {
    /// Contract id (see `list_loans`).
    pub loan_id: String,
}

impl ToolInput for LoanIdInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("loan_id", &self.loan_id)?;
        Ok(json!({ "id": self.loan_id.trim() }))
    }
}

/// Payment of one installment, as in a bank screenshot (amounts in cents).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoanPaymentInput {
    /// Contract id (see `list_loans`).
    pub loan_id: String,
    /// Payment date `YYYY-MM-DD` (the installment is that month's).
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Amount paid, in cents (if omitted, the installment amount).
    pub amount: Option<i64>,
    /// Outstanding balance after the payment, in cents: syncs the contract balance.
    pub balance_after: Option<i64>,
    /// Installments left after this one (when the bank shows it; changes after a prepayment
    /// that shortens the term).
    pub remaining_term: Option<u32>,
    /// Amortization, in cents.
    pub amortization: Option<i64>,
    /// Interest, in cents.
    pub interest: Option<i64>,
    /// Insurance, in cents.
    pub insurance: Option<i64>,
    /// Fees, in cents.
    pub fees: Option<i64>,
    /// Monetary adjustment for the month (TR…), in cents.
    pub adjustment: Option<i64>,
}

impl ToolInput for LoanPaymentInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("loan_id", &self.loan_id)?;
        check_date("date", &Some(self.date.clone()))?;
        let mut v = to_object(self)?;
        let m = v.as_object_mut().expect("objeto");
        m.remove("loan_id");
        m.insert("id".into(), json!(self.loan_id.trim()));
        Ok(v)
    }
}

/// Transaction that paid an installment of the contract.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoanLinkInput {
    /// Installment id (see `get_amortization_schedule`).
    pub installment_id: String,
    /// Id of the outgoing transaction that paid the installment.
    pub txn_id: String,
}

impl ToolInput for LoanLinkInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("installment_id", &self.installment_id)?;
        check_non_empty("txn_id", &self.txn_id)?;
        Ok(json!({
            "installment_id": self.installment_id.trim(),
            "txn_id": self.txn_id.trim(),
        }))
    }
}

/// Row of the contract statement (amounts in cents).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoanStatementRowInput {
    /// Due/payment date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// The month's installment (total charge), in cents.
    pub total: i64,
    /// Amortization, in cents.
    pub amortization: i64,
    /// Interest, in cents.
    pub interest: i64,
    /// Insurance (MIP + DFI), in cents.
    pub insurance: Option<i64>,
    /// Fees (administration), in cents.
    pub fees: Option<i64>,
    /// Monetary adjustment of the balance in the month (TR…), in cents.
    pub adjustment: Option<i64>,
    /// Outstanding balance after the installment, in cents.
    pub balance_after: i64,
    /// Amount paid, in cents (if omitted, the installment).
    pub paid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoanStatementInput {
    /// Contract id (see `list_loans`).
    pub loan_id: String,
    /// Statement rows, in date order (transcribed from the bank's image or PDF).
    pub rows: Option<Vec<LoanStatementRowInput>>,
    /// Or the statement text with a header row (columns separated by tab or `;`).
    pub text: Option<String>,
    /// Installments left after the last row (bank statements usually show it; if omitted,
    /// computed from the last installment).
    pub remaining_term: Option<u32>,
}

impl ToolInput for LoanStatementInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("loan_id", &self.loan_id)?;
        if self.rows.as_ref().is_none_or(Vec::is_empty)
            && self.text.as_deref().is_none_or(|t| t.trim().is_empty())
        {
            return Err(tx("mcp-valida-extrato-linhas"));
        }
        let mut v = to_object(self)?;
        let m = v.as_object_mut().expect("objeto");
        m.remove("loan_id");
        m.insert("id".into(), json!(self.loan_id.trim()));
        Ok(v)
    }
}

/// Pension plan statement (pension fund, "Extrato Financeiro do Participante" layout).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PensionStatementInput {
    /// Pension asset that receives the statement (see `list_assets`); without it, the plan with
    /// the same CNPJ or a new asset.
    pub asset_id: Option<String>,
    /// Account for the units (required to import; see `list_accounts`).
    pub account_id: Option<String>,
    /// Post the risk insurance (disability and death) as an expense in this account.
    pub risk_account_id: Option<String>,
    /// The statement PDF (external MCP clients).
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
    /// Or the statement text (copied from the PDF).
    pub text: Option<String>,
    /// Preview indices that must not be saved (import only).
    pub skip: Option<Vec<u32>>,
    /// Preview with all rows (default: no; you get the totals, the check, the count and the
    /// sums by type and status, and only the rows that are not new, such as those already
    /// imported and insurance without an account). Ask for the rows only if you need to choose
    /// `skip`.
    pub rows: Option<bool>,
}

/// Prévia do extrato de previdência sem as linhas (o resultado inteiro passa fácil de 50
/// KB): contagem e somas por tipo e por situação e só as linhas que não são novas.
fn compact_pension_preview(mut v: Value) -> Value {
    let Some(rows) = v.get("rows").and_then(Value::as_array).cloned() else {
        return v;
    };
    let mut by: std::collections::BTreeMap<String, (u32, i64, i64)> = Default::default();
    for r in &rows {
        let key = format!(
            "{}/{}",
            r["kind"].as_str().unwrap_or("?"),
            r["status"].as_str().unwrap_or("?")
        );
        let e = by.entry(key).or_default();
        e.0 += 1;
        e.1 += r["paid"].as_i64().unwrap_or(0);
        e.2 += r["retirement"].as_i64().unwrap_or(0);
    }
    let competences: Vec<&str> = rows
        .iter()
        .filter_map(|r| r["competence"].as_str())
        .collect();
    let shown: Vec<Value> = rows
        .iter()
        .filter(|r| r["status"] != "new")
        .take(40)
        .map(|r| {
            json!({
                "index": r["index"], "competence": r["competence"], "kind": r["kind"],
                "status": r["status"], "paid": r["paid"], "description": r["description"],
            })
        })
        .collect();
    let m = v.as_object_mut().expect("objeto");
    m.insert(
        "row_summary".into(),
        json!({
            "count": rows.len(),
            "first_competence": competences.iter().min(),
            "last_competence": competences.iter().max(),
            "by_kind_and_status": by.into_iter().map(|(k, (n, paid, ret))| json!({
                "group": k, "count": n, "paid": paid, "retirement": ret,
            })).collect::<Vec<_>>(),
        }),
    );
    m.insert("rows".into(), Value::Array(shown));
    m.insert(
        "rows_note".into(),
        Value::from(tx("mcp-nota-linhas-previdencia")),
    );
    v
}

impl ToolInput for PensionStatementInput {
    fn post_process(&self, value: Value) -> Value {
        if self.rows.unwrap_or(false) {
            value
        } else {
            compact_pension_preview(value)
        }
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
            m.remove("rows");
        }
        Ok(v)
    }
}

/// `reduce_term` (keeps the installment and shortens the term; saves more interest) or
/// `reduce_installment` (keeps the term and lowers the installment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum PrepayMode {
    ReduceTerm,
    ReduceInstallment,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrepaymentInput {
    /// Contract id.
    pub loan_id: String,
    /// Prepayment date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Amount in cents of the contract's currency.
    pub amount: i64,
    pub mode: PrepayMode,
}

impl ToolInput for PrepaymentInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("loan_id", &self.loan_id)?;
        check_date("date", &Some(self.date.clone()))?;
        if self.amount <= 0 {
            return Err(tx("mcp-valida-valor-positivo"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListPhysicalAssetsInput {
    /// Valuation date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub as_of: Option<String>,
    /// Deflator for the real gain (default: `ipca`).
    pub deflator: Option<Deflator>,
    /// Include archived physical assets (sold or written off).
    pub include_archived: Option<bool>,
}

impl ToolInput for ListPhysicalAssetsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("as_of", &self.as_of)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalAssetIdInput {
    /// Physical asset id (see `list_physical_assets`).
    pub asset_id: String,
    /// Valuation date `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    pub as_of: Option<String>,
    /// Deflator for the real gain (default: `ipca`).
    pub deflator: Option<Deflator>,
}

impl ToolInput for PhysicalAssetIdInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_date("as_of", &self.as_of)?;
        let mut v = to_object(self)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("asset_id");
            o.insert("id".into(), json!(self.asset_id.trim()));
        }
        Ok(v)
    }
}

/// A contribution from an insurer's payment history (PGBL/VGBL): date and amount paid.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PensionPaymentInput {
    /// Payment date `YYYY-MM-DD`.
    #[schemars(pattern(DATE_PATTERN))]
    pub date: String,
    /// Amount paid, in cents.
    #[schemars(range(min = 1))]
    pub amount: i64,
}

/// Contributions saved from a pension plan's payment history.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PensionPaymentsListInput {
    /// Pension asset (see `list_assets` with `classes: ["pension"]`).
    pub asset_id: String,
    /// From this date `YYYY-MM-DD` (inclusive).
    #[serde(alias = "from", alias = "start_date", alias = "from_date")]
    pub date_from: Option<String>,
    /// Up to this date `YYYY-MM-DD` (inclusive).
    #[serde(alias = "to", alias = "end_date", alias = "to_date")]
    pub date_to: Option<String>,
    /// Page size (default 100, max 500), newest first.
    pub limit: Option<i64>,
    /// How many to skip (pagination).
    pub offset: Option<i64>,
}

impl ToolInput for PensionPaymentsListInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        check_date("date_from", &self.date_from)?;
        check_date("date_to", &self.date_to)?;
        to_object(self)
    }
}

/// Payment history of a PGBL/VGBL plan (e.g. the Caixa Vida e Previdência app: amount, "Pago"
/// and date), turned into unit purchases with the CVM unit price of the plan's fund.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PensionPaymentsInput {
    /// Pension asset already registered (see `list_assets` with `classes: ["pension"]`).
    pub asset_id: String,
    /// Account that holds the plan's units. Without it: the account that already has the
    /// plan's units or, with none, a new one created in the same proposal. Cash is not touched.
    pub account_id: Option<String>,
    /// Each payment: date and amount in cents. Not needed with `attachments` or `texts`.
    #[schemars(length(max = 600))]
    #[serde(default)]
    pub payments: Vec<PensionPaymentInput>,
    /// Screenshots of the history the user attached in the conversation (app agent and CLIs),
    /// in scroll order: `["1", "2", "3"]`. Treasury reads them (amount, "Pago" and date); do
    /// not transcribe them. Overlapping screenshots do not repeat payments.
    pub attachments: Option<Vec<String>>,
    /// Or the text of the screenshots (copied or read by OCR), one item per screenshot in
    /// scroll order.
    pub texts: Option<Vec<String>>,
    /// Name of each screenshot, in the order of `texts` (filled by Treasury from
    /// `attachments`; optional otherwise).
    pub sources: Option<Vec<String>>,
}

impl ToolInput for PensionPaymentsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        if self.attachments.is_some() {
            return Err(tx("mcp-valida-anexo-nao-encontrado"));
        }
        let has_texts = self
            .texts
            .as_ref()
            .is_some_and(|t| t.iter().any(|x| !x.trim().is_empty()));
        if self.payments.is_empty() && !has_texts {
            return Err(tx("mcp-valida-pagamentos-vazio"));
        }
        for p in &self.payments {
            check_date("date", &Some(p.date.clone()))?;
            if p.amount <= 0 {
                return Err(tx("mcp-valida-pagamento-valor"));
            }
        }
        to_object(self)
    }
}

/// Sales ledger: each sale with the nominal and the real result.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SalesInput {
    /// Start `YYYY-MM-DD` (inclusive); default: since the first sale.
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// End `YYYY-MM-DD` (inclusive); default: today.
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
    /// Deflator for the real result (default: `ipca`).
    pub deflator: Option<Deflator>,
    /// Only this asset (see `list_assets`).
    pub asset_id: Option<String>,
    /// Only sales in these accounts (see `list_accounts`).
    pub account_ids: Option<Vec<String>>,
    /// Only these classes.
    pub classes: Option<Vec<AssetClass>>,
    /// Cost method (default: the family's choice, or the rule of the family's country).
    pub cost_method: Option<CostMethod>,
    /// Include each sale (default: no; you get the totals by year, class and asset and the
    /// 20 most recent sales).
    pub rows: Option<bool>,
}

impl ToolInput for SalesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        let mut v = to_object(self)?;
        if let Some(m) = v.as_object_mut() {
            m.remove("rows");
        }
        Ok(v)
    }

    fn post_process(&self, mut value: Value) -> Value {
        if !self.rows.unwrap_or(false)
            && let Some(rows) = value.get_mut("rows").and_then(Value::as_array_mut)
        {
            rows.truncate(20);
        }
        if let Some(m) = value.as_object_mut() {
            m.remove("by_month");
        }
        value
    }
}

/// A college goal (see `list_goals`: goals with `education`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EducationPlanInput {
    /// Goal id (see `list_goals`).
    pub goal_id: String,
}

impl ToolInput for EducationPlanInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

/// What-if for a college plan: the given fields over the goal's plan (with `goal_id`) or over
/// the defaults. Amounts in cents at today's prices; fractions per year (0.03 = 3%).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SimulateEducationInput {
    /// Start from this goal's plan and saved amount (see `list_goals`).
    pub goal_id: Option<String>,
    /// The child (member id from `list_members`; needs a birth date).
    pub member_id: Option<String>,
    /// Birth date `YYYY-MM-DD` instead of the member's (child not yet born, or "what if").
    #[schemars(pattern(DATE_PATTERN))]
    pub birth_date: Option<String>,
    /// Age when the course starts (default 18).
    #[schemars(range(min = 0, max = 60))]
    pub start_age: Option<u32>,
    /// Month the academic year starts, 1–12 (default 2, February).
    #[schemars(range(min = 1, max = 12))]
    pub start_month: Option<u32>,
    /// Course start `YYYY-MM-DD`, instead of age and month.
    #[schemars(pattern(DATE_PATTERN))]
    pub start_date: Option<String>,
    /// `private`, `public` or `abroad`.
    pub institution_type: Option<String>,
    /// Course length in months (48 = 4 years).
    #[schemars(range(min = 1, max = 180))]
    pub duration_months: Option<u32>,
    /// Payments per year: 1, 2, 4 or 12.
    pub payments_per_year: Option<u32>,
    /// Today's tuition per payment, in cents.
    #[schemars(range(min = 0))]
    pub fee: Option<i64>,
    /// Extras per year (materials, enrollment), in cents.
    #[schemars(range(min = 0))]
    pub yearly_extras: Option<i64>,
    /// Living costs per month during the course, in cents.
    #[schemars(range(min = 0))]
    pub monthly_living: Option<i64>,
    /// Scholarship or discount on tuition, 0–100.
    #[schemars(range(min = 0.0, max = 100.0))]
    pub discount_pct: Option<f64>,
    /// `series_10y` (education inflation from the series), `headline` (general inflation only)
    /// or `fixed_spread`.
    pub inflation_mode: Option<String>,
    /// Education inflation above general inflation, a year (with `fixed_spread`).
    #[schemars(range(min = -0.1, max = 0.2))]
    pub fixed_spread: Option<f64>,
    /// Assumed real return of the saved money, a year (default 0.03). An assumption, not a
    /// promise.
    #[schemars(range(min = -0.05, max = 0.15))]
    pub real_return: Option<f64>,
    /// `indexed` (rises with inflation once a year) or `flat` (same amount every month).
    pub contribution_mode: Option<String>,
    /// Already saved, in cents (default: the goal's current amount, or 0).
    #[schemars(range(min = 0))]
    pub saved: Option<i64>,
}

impl ToolInput for SimulateEducationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        let all = to_object(self)?;
        let mut plan = Map::new();
        let mut out = Map::new();
        if let Value::Object(m) = all {
            for (k, v) in m {
                match k.as_str() {
                    "goal_id" | "saved" | "birth_date" => {
                        out.insert(k, v);
                    }
                    _ => {
                        plan.insert(k, v);
                    }
                }
            }
        }
        if !plan.is_empty() {
            out.insert("plan".into(), Value::Object(plan));
        }
        Ok(Value::Object(out))
    }
}

/// Concentration of the portfolio.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConcentrationInput {
    /// Date (`YYYY-MM-DD`, default today).
    #[schemars(regex(pattern = DATE_PATTERN))]
    pub as_of: Option<String>,
}

impl ToolInput for ConcentrationInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("as_of", &self.as_of)?;
        to_object(self)
    }
}

/// Replace the labels of an asset.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetAssetLabelsInput {
    /// Asset id (from `list_assets` or `get_portfolio`).
    pub asset_id: String,
    /// The asset's labels (replaces the list; new ones are created), e.g. ["reserva"].
    pub labels: Vec<String>,
}

impl ToolInput for SetAssetLabelsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}

/// Mark a label as a concentration group.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConcentrationGroupInput {
    /// Label name (created if missing), e.g. "musk".
    pub name: String,
    /// true: the label's assets are checked together against the group limit.
    #[serde(default = "yes")]
    pub is_concentration_group: bool,
    /// Group limit, % of the portfolio (default: the group default, 30).
    #[schemars(range(min = 0.1, max = 100.0))]
    pub concentration_limit_pct: Option<f64>,
}

fn yes() -> bool {
    true
}

impl ToolInput for ConcentrationGroupInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        to_object(self)
    }
}

/// What a concentration limit applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum ConcentrationScope {
    Asset,
    AssetDefault,
    GroupDefault,
}

/// Set a concentration limit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConcentrationLimitInput {
    /// `asset` (needs `asset_id`), `asset_default` (20 unless changed) or `group_default` (30).
    pub scope: ConcentrationScope,
    /// Asset id, with `scope: asset`.
    pub asset_id: Option<String>,
    /// % of the portfolio; null goes back to the default.
    #[schemars(range(min = 0.1, max = 100.0))]
    pub pct: Option<f64>,
}

impl ToolInput for ConcentrationLimitInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.scope == ConcentrationScope::Asset && self.asset_id.is_none() {
            return Err(tx("mcp-valida-limite-ativo"));
        }
        to_object(self)
    }
}
