//! Entradas das ferramentas MCP da análise de ativos (fase 3): fundamentos, screener,
//! rankings, comparador, preço justo, documentos IPE e os métodos do Mercado. Leitura, salvo
//! salvar tela e observar um papel (propostas).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::inputs::{
    DATE_PATTERN, ToolInput, check_date, check_non_empty, to_object, tx, tx_args, tx_campo,
};
use super::inputs_invest::AssetClass;

fn check_subject(asset_id: &Option<String>, ticker: &Option<String>) -> Result<(), String> {
    let has = |v: &Option<String>| v.as_deref().is_some_and(|s| !s.trim().is_empty());
    if !has(asset_id) && !has(ticker) {
        return Err(tx("mcp-valida-ativo-ou-ticker"));
    }
    Ok(())
}

fn check_limit(limit: Option<i64>, max: i64) -> Result<Option<i64>, String> {
    match limit {
        Some(l) if l < 1 => Err(tx("mcp-valida-limit")),
        Some(l) => Ok(Some(l.min(max))),
        None => Ok(None),
    }
}

/// Health score of an asset.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetScoreInput {
    /// Id of a registered asset (see `list_assets`).
    pub asset_id: Option<String>,
    /// Or the trading code of any stock/FII in the downloaded universe (e.g. `ITUB4`,
    /// `HGLG11`).
    pub ticker: Option<String>,
}

impl ToolInput for AssetScoreInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_subject(&self.asset_id, &self.ticker)?;
        to_object(self)
    }
}

/// Portfolio scores (no arguments).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PortfolioScoresInput {}

impl ToolInput for PortfolioScoresInput {
    fn to_api_args(&self) -> Result<Value, String> {
        Ok(json!({}))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetFundamentalsInput {
    /// Id of a registered asset (see `list_assets`).
    pub asset_id: Option<String>,
    /// Or the trading code of any stock/FII in the downloaded universe (e.g. `PETR4`,
    /// `HGLG11`).
    pub ticker: Option<String>,
}

impl ToolInput for GetFundamentalsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_subject(&self.asset_id, &self.ticker)?;
        to_object(self)
    }
}

/// Order: `asc` (smallest first) or `desc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// `and` (all conditions, default) or `or` (any of them).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Logic {
    And,
    Or,
}

/// Operator: `gt`, `gte`, `lt`, `lte`, `eq`, `ne` (number or text), `between` (`[min,
/// max]`), `in`/`not_in` (list), `contains` (text), `exists`/`missing` (indicator
/// available or not; no `value`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum CmpOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Ne,
    Between,
    In,
    NotIn,
    Contains,
    Exists,
    Missing,
}

/// Condition `field op value`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionInput {
    /// Indicator (`pl`, `pvp`, `dy`, `roe`, `roic`, `net_margin`, `net_debt_ebitda`,
    /// `liquidity`, `market_cap`, `graham_upside`…; list in `rank_assets`) or text field
    /// (`ticker`, `name`, `class`, `sector`, `segment`, `currency`, `score_band`, `red_flags`)
    /// or `held`/`watched`.
    pub field: String,
    pub op: CmpOp,
    /// Number (percentages as fractions: 0.06 = 6%), text, list or `[min, max]`.
    pub value: Option<Value>,
}

/// Group of conditions combined by `logic`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionGroupInput {
    /// How the conditions in the group combine (default `and`).
    pub logic: Option<Logic>,
    pub conditions: Vec<ConditionInput>,
}

/// Slice of the universe.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UniverseFilterInput {
    /// Only these classes (`stock_br`, `fii`, `stock_us`…).
    pub classes: Option<Vec<AssetClass>>,
    /// Parts of the sector name (e.g. `energia`, `bancos`), accent-insensitive.
    pub sectors: Option<Vec<String>>,
    /// Parts of the FII segment name (e.g. `logística`, `shoppings`).
    pub segments: Option<Vec<String>>,
    /// Minimum average daily liquidity, in currency units (e.g. 1000000 = 1 million per day).
    pub min_liquidity: Option<f64>,
    /// Only assets in the portfolio and the watchlist.
    pub only_mine: Option<bool>,
}

impl UniverseFilterInput {
    fn to_value(&self) -> Result<Value, String> {
        if self.min_liquidity.is_some_and(|l| l < 0.0) {
            return Err(tx("mcp-valida-liquidez"));
        }
        to_object(self)
    }
}

/// Screener filter: conditions and groups combined by `logic`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScreenFilterInput {
    /// How to combine conditions and groups (default `and`).
    pub logic: Option<Logic>,
    pub conditions: Option<Vec<ConditionInput>>,
    /// Groups, for `(A and B) or (C and D)`.
    pub groups: Option<Vec<ConditionGroupInput>>,
    pub universe: Option<UniverseFilterInput>,
    /// Characteristics the security must all have (keys from `list_asset_tags`, e.g.
    /// `["nunca_prejuizo", "proventos_10_anos"]`).
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunScreenInput {
    /// Filters (e.g. `{"conditions": [{"field": "pl", "op": "lt", "value": 10}, {"field":
    /// "dy", "op": "gte", "value": 0.06}]}`).
    pub filters: Option<ScreenFilterInput>,
    /// Or the id of a screen saved by the user.
    pub screen_id: Option<String>,
    /// Indicator to sort by (default `market_cap`).
    pub sort_by: Option<String>,
    /// Default: the natural order of the sort indicator.
    pub order: Option<SortOrder>,
    /// Maximum number of securities (default 50, max 200).
    pub limit: Option<i64>,
    /// Extra indicators in each row.
    pub fields: Option<Vec<String>>,
}

impl ToolInput for RunScreenInput {
    fn to_api_args(&self) -> Result<Value, String> {
        let mut v = json!({});
        if let Some(f) = &self.filters {
            let mut fv = json!({
                "logic": f.logic.unwrap_or(Logic::And),
                "conditions": f.conditions.clone().unwrap_or_default(),
                "groups": f.groups.iter().flatten().map(|g| json!({
                    "logic": g.logic.unwrap_or(Logic::And),
                    "conditions": g.conditions,
                })).collect::<Vec<_>>(),
            });
            if let Some(u) = &f.universe {
                fv["universe"] = u.to_value()?;
            }
            if let Some(t) = f.tags.as_ref().filter(|t| !t.is_empty()) {
                fv["tags"] = json!(t);
            }
            v["filters"] = fv;
        }
        if let Some(s) = &self.screen_id {
            v["screen_id"] = json!(s);
        }
        if let Some(s) = &self.sort_by {
            v["sort_by"] = json!(s);
        }
        if let Some(o) = self.order {
            v["order"] = json!(o);
        }
        if let Some(l) = check_limit(self.limit, 200)? {
            v["limit"] = json!(l);
        }
        if let Some(f) = &self.fields {
            v["fields"] = json!(f);
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RankAssetsInput {
    /// Indicator: `pl`, `pvp`, `ev_ebitda`, `ev_ebit`, `psr`, `p_ativo`, `dy`, `payout`,
    /// `roe`, `roic`, `roa`, `gross_margin`, `ebitda_margin`, `ebit_margin`, `net_margin`,
    /// `net_debt_equity`, `gross_debt_equity`, `net_debt_ebitda`, `current_ratio`, `p_ebit`,
    /// `p_working_capital`, `p_ncav`, `ebit_assets`, `asset_turnover`, `revenue_cagr_5y`,
    /// `earnings_cagr_5y`, `loss_years_10y`, `lpa`, `vpa`, `market_cap`, `ev`, `liquidity`,
    /// `price`, `graham`, `graham_upside`, `bazin`, `bazin_upside`, `nav_per_share`,
    /// `holders`, `last_dividend`, the method indicators `earnings_yield`, `magic_roc`,
    /// `fcf_yield`, `peg`, `eps_growth_10y`, `dividend_years`, `dividend_cagr_5y`, and the
    /// health score of the whole universe: `score`, `health`, `attractiveness`, `confidence`,
    /// `piotroski`, `altman_z`, `beneish`, `momentum_12_1`, `volatility_1y`.
    pub indicator: String,
    /// Default: the natural order (lowest P/L (P/E) first, highest DY first).
    pub order: Option<SortOrder>,
    /// Maximum number of securities (default 20, max 200).
    pub limit: Option<i64>,
    /// Only these classes (`stock_br`, `fii`, `stock_us`…).
    pub classes: Option<Vec<AssetClass>>,
    /// Parts of the sector name (e.g. `energia`).
    pub sectors: Option<Vec<String>>,
    /// Parts of the FII segment name (e.g. `logística`).
    pub segments: Option<Vec<String>>,
    /// Minimum average daily liquidity, in currency units.
    pub min_liquidity: Option<f64>,
    /// Only assets in the portfolio and the watchlist.
    pub only_mine: Option<bool>,
}

impl ToolInput for RankAssetsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.indicator.trim().is_empty() {
            return Err(tx_campo("mcp-valida-informe", "indicator"));
        }
        let filters = UniverseFilterInput {
            classes: self.classes.clone(),
            sectors: self.sectors.clone(),
            segments: self.segments.clone(),
            min_liquidity: self.min_liquidity,
            only_mine: self.only_mine,
        }
        .to_value()?;
        let mut v = json!({ "indicator": self.indicator.trim(), "filters": filters });
        if let Some(o) = self.order {
            v["order"] = json!(o);
        }
        if let Some(l) = check_limit(self.limit, 200)? {
            v["limit"] = json!(l);
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompareAssetsInput {
    /// Ids of registered assets.
    pub asset_ids: Option<Vec<String>>,
    /// Trading codes from the universe (e.g. `ITUB4`). 2 to 6 assets in total.
    pub tickers: Option<Vec<String>>,
}

impl ToolInput for CompareAssetsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        let n =
            self.asset_ids.as_ref().map_or(0, Vec::len) + self.tickers.as_ref().map_or(0, Vec::len);
        if !(2..=6).contains(&n) {
            return Err(tx("mcp-valida-comparar"));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FairPriceInput {
    /// Id of a registered asset.
    pub asset_id: Option<String>,
    /// Or the trading code (e.g. `TAEE11`).
    pub ticker: Option<String>,
    /// Bazin minimum rate as a fraction (0.06 = 6%; default from the user's settings).
    pub bazin_rate: Option<f64>,
    /// Years in the Bazin dividend average (1 to 10, default 5).
    pub years: Option<u32>,
}

impl ToolInput for FairPriceInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_subject(&self.asset_id, &self.ticker)?;
        if self.bazin_rate.is_some_and(|r| !(r > 0.0 && r <= 1.0)) {
            return Err(tx("mcp-valida-bazin"));
        }
        if self.years.is_some_and(|y| !(1..=10).contains(&y)) {
            return Err(tx_args(
                "mcp-valida-faixa",
                &[
                    ("campo", "years".into()),
                    ("min", 1.into()),
                    ("max", 10.into()),
                ],
            ));
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CorporateDocumentsInput {
    /// Id of a registered asset (default: all in the portfolio and the watchlist).
    pub asset_id: Option<String>,
    /// Or the trading code.
    pub ticker: Option<String>,
    /// Filed from `YYYY-MM-DD` on (default: 90 days ago).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// Up to `YYYY-MM-DD` (default: today).
    #[schemars(pattern(DATE_PATTERN))]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
    /// Categories: `Fato Relevante`, `Comunicado ao Mercado`, `Aviso aos Acionistas`,
    /// `Calendário de Eventos Corporativos`, `Dados Econômico-Financeiros`, `Assembleia`,
    /// `Reunião da Administração`, `Comunicação sobre Transação entre Partes Relacionadas`,
    /// `Informações de Companhias em Recuperação Judicial ou Extrajudicial`.
    pub categories: Option<Vec<String>>,
    /// Maximum number of documents (default 50, max 200).
    pub limit: Option<i64>,
}

impl ToolInput for CorporateDocumentsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        let mut v = to_object(self)?;
        if let Some(l) = check_limit(self.limit, 200)? {
            v["limit"] = json!(l);
        }
        Ok(v)
    }
}

/// Why an asset has documents or not.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DocumentsStatusInput {
    /// Id of a registered asset…
    pub asset_id: Option<String>,
    /// …or the trading code (also outside the registry).
    pub ticker: Option<String>,
}

impl ToolInput for DocumentsStatusInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.asset_id.as_deref().is_none_or(|s| s.trim().is_empty())
            && self.ticker.as_deref().is_none_or(|s| s.trim().is_empty())
        {
            return Err(tx("mcp-valida-ativo-ou-ticker"));
        }
        to_object(self)
    }
}

/// Asset data field enriched from the sources (see `get_asset_data_gaps`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AssetDataField {
    Sector,
    Subsector,
    Segment,
    Isin,
    Cnpj,
    IssuerCnpj,
    IssuerName,
    ShareKind,
    ListingSegment,
    CvmCode,
    Cik,
    FiiMandate,
    FiiManagement,
    FundClassCvm,
    FundClassAnbima,
    Benchmark,
    Manager,
    Administrator,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetDataGapsInput {
    /// Only these assets (see `list_assets`); without it, every non-archived asset.
    pub asset_ids: Option<Vec<String>>,
    /// Only assets with trades.
    pub held_only: Option<bool>,
    /// Also fields with nothing to do (`ok`, `missing`); default: only fields with a
    /// suggestion, conflict or divergence.
    pub include_ok: Option<bool>,
}

impl ToolInput for AssetDataGapsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetFieldSourcesInput {
    /// Id of a registered asset (see `list_assets`).
    pub asset_id: String,
}

impl ToolInput for AssetFieldSourcesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        super::inputs::check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetEnrichmentItem {
    /// Asset id.
    pub asset_id: String,
    pub field: AssetDataField,
    /// One of the values the sources gave for this field (`candidate.value` or one of the
    /// `options` in `get_asset_data_gaps`). Values not observed in a source are refused.
    pub value: String,
    /// Replace a value that is already filled in (the user's, or another source's). Only when
    /// the user asked for it.
    pub replace: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProposeAssetEnrichmentInput {
    /// Fields to fill in, one item per asset and field.
    pub items: Vec<AssetEnrichmentItem>,
}

impl ToolInput for ProposeAssetEnrichmentInput {
    fn to_api_args(&self) -> Result<Value, String> {
        if self.items.is_empty() {
            return Err(tx_campo("mcp-valida-informe", "items"));
        }
        for i in &self.items {
            super::inputs::check_non_empty("asset_id", &i.asset_id)?;
            super::inputs::check_non_empty("value", &i.value)?;
        }
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetLookupInput {
    /// Id of a registered asset (see `list_assets`).
    pub asset_id: String,
    /// Ask the sources again even if they were checked in the last 30 days.
    pub refresh: Option<bool>,
}

impl ToolInput for AssetLookupInput {
    fn to_api_args(&self) -> Result<Value, String> {
        super::inputs::check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetStudiesInput {
    /// Id of a registered asset (see `list_assets`).
    pub asset_id: String,
}

impl ToolInput for AssetStudiesInput {
    fn to_api_args(&self) -> Result<Value, String> {
        super::inputs::check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetStudyInput {
    /// Study id (see `list_asset_studies`).
    pub id: String,
}

impl ToolInput for AssetStudyInput {
    fn to_api_args(&self) -> Result<Value, String> {
        super::inputs::check_non_empty("id", &self.id)?;
        to_object(self)
    }
}

/// Screener method (`list_screen_methods`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MethodRunInput {
    /// Method key: `graham_defensivo`, `bazin`, `magic_formula`, `qualidade_preco_justo`,
    /// `fii_desconto`.
    pub key: String,
    /// Maximum number of securities (default 30, max 200).
    pub limit: Option<i64>,
}

impl ToolInput for MethodRunInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("key", &self.key)?;
        Ok(json!({ "key": self.key }))
    }

    fn post_process(&self, mut value: Value) -> Value {
        let limit = self.limit.unwrap_or(30).clamp(1, 200) as usize;
        if let Some(rows) = value.get_mut("rows").and_then(Value::as_array_mut) {
            rows.truncate(limit);
        }
        value
    }
}

/// Why a security meets a method or not.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MethodExplainInput {
    /// Method key (see `list_screen_methods`).
    pub key: String,
    /// Trading code (e.g. PETR4).
    pub ticker: String,
}

impl ToolInput for MethodExplainInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("key", &self.key)?;
        check_non_empty("ticker", &self.ticker)?;
        to_object(self)
    }
}

/// Watch a security of the market universe.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WatchAssetInput {
    /// Trading code of a B3 stock or FII in the universe (e.g. PETR4, HGLG11).
    pub ticker: String,
}

impl ToolInput for WatchAssetInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("ticker", &self.ticker)?;
        to_object(self)
    }
}

/// Save a screener screen.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveScreenInput {
    /// Screen name.
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    /// What the screen looks for, in a sentence.
    pub description: Option<String>,
    /// The same filters as `run_screen`.
    pub filters: ScreenFilterInput,
}

impl ToolInput for SaveScreenInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_non_empty("name", &self.name)?;
        let run = RunScreenInput {
            filters: Some(self.filters.clone()),
            screen_id: None,
            sort_by: None,
            order: None,
            limit: None,
            fields: None,
        };
        let v = run.to_api_args()?;
        Ok(json!({
            "name": self.name.trim(),
            "description": self.description,
            "filters": v["filters"],
        }))
    }
}
/// Interval of the price history points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PriceInterval {
    /// Every close recorded.
    Daily,
    /// The last close of each week.
    Weekly,
    /// The last close of each month.
    Monthly,
}

/// Closing price history of a registered asset.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PriceHistoryInput {
    /// Id of a registered asset (see `list_assets`).
    pub asset_id: String,
    /// Start date `YYYY-MM-DD` (default: 12 months before `to`).
    #[schemars(regex(pattern = DATE_PATTERN))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "date_from", alias = "start_date", alias = "from_date")]
    pub from: Option<String>,
    /// End date `YYYY-MM-DD` (default: today).
    #[schemars(regex(pattern = DATE_PATTERN))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "date_to", alias = "end_date", alias = "to_date")]
    pub to: Option<String>,
    /// Points: `daily`, `weekly` or `monthly` (default: daily up to 4 months, weekly up to
    /// 13 months, monthly beyond).
    #[serde(default, skip_serializing)]
    pub interval: Option<PriceInterval>,
}

/// Até quantos pontos a ferramenta devolve (os mais recentes).
const MAX_PRICE_POINTS: usize = 400;

/// Fechamentos (`date`, `close` em texto) reduzidos ao último de cada semana ou mês, com o
/// resumo do período: primeiro e último, variação (fração), mínimo e máximo.
pub(crate) fn summarize_prices(
    asset_id: &str,
    points: &[Value],
    interval: Option<PriceInterval>,
) -> Value {
    use chrono::Datelike;
    let date = |p: &Value| {
        p["date"]
            .as_str()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
    };
    let close = |p: &Value| p["close"].as_str().and_then(|c| c.parse::<f64>().ok());
    let span = match (points.first().and_then(date), points.last().and_then(date)) {
        (Some(a), Some(b)) => (b - a).num_days(),
        _ => 0,
    };
    let interval = interval.unwrap_or(if span <= 122 {
        PriceInterval::Daily
    } else if span <= 400 {
        PriceInterval::Weekly
    } else {
        PriceInterval::Monthly
    });
    let key = |p: &Value| -> Option<(i32, u32)> {
        let d = date(p)?;
        Some(match interval {
            PriceInterval::Daily => (d.year(), d.ordinal()),
            PriceInterval::Weekly => (d.iso_week().year(), d.iso_week().week()),
            PriceInterval::Monthly => (d.year(), d.month()),
        })
    };
    let mut kept: Vec<Value> = vec![];
    for (i, p) in points.iter().enumerate() {
        let next_same = points.get(i + 1).is_some_and(|n| key(n) == key(p));
        if !next_same {
            kept.push(json!({ "date": p["date"], "close": p["close"], "source": p["source"] }));
        }
    }
    let truncated = kept.len() > MAX_PRICE_POINTS;
    if truncated {
        kept.drain(..kept.len() - MAX_PRICE_POINTS);
    }
    let closes: Vec<f64> = points.iter().filter_map(close).collect();
    let first = points.first().and_then(close);
    let last = points.last().and_then(close);
    let change = match (first, last) {
        (Some(a), Some(b)) if a != 0.0 => Some(((b / a - 1.0) * 10_000.0).round() / 10_000.0),
        _ => None,
    };
    json!({
        "asset_id": asset_id,
        "interval": interval,
        "from": points.first().map(|p| p["date"].clone()),
        "to": points.last().map(|p| p["date"].clone()),
        "first_close": points.first().map(|p| p["close"].clone()),
        "last_close": points.last().map(|p| p["close"].clone()),
        "change": change,
        "min_close": closes.iter().cloned().fold(None, |m: Option<f64>, c| Some(m.map_or(c, |m| m.min(c)))),
        "max_close": closes.iter().cloned().fold(None, |m: Option<f64>, c| Some(m.map_or(c, |m| m.max(c)))),
        "recorded": points.len(),
        "truncated": truncated,
        "count": kept.len(),
        "points": kept,
    })
}

impl ToolInput for PriceHistoryInput {
    fn to_api_args(&self) -> Result<Value, String> {
        super::inputs::check_non_empty("asset_id", &self.asset_id)?;
        check_date("from", &self.from)?;
        check_date("to", &self.to)?;
        to_object(self)
    }

    fn post_process(&self, value: Value) -> Value {
        let points = value.as_array().cloned().unwrap_or_default();
        summarize_prices(&self.asset_id, &points, self.interval)
    }
}

/// Asks for a new AI study of a registered asset.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StartAssetStudyInput {
    /// Id of a registered asset (see `list_assets`).
    pub asset_id: String,
    /// What the study should answer, in the user's words (optional, up to 500 characters).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// Send the asset's weight in the portfolio and the gain (%) to the study engine (never
    /// quantities or amounts). Only when the user asked for it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_position: Option<bool>,
}

impl ToolInput for StartAssetStudyInput {
    fn to_api_args(&self) -> Result<Value, String> {
        super::inputs::check_non_empty("asset_id", &self.asset_id)?;
        to_object(self)
    }
}

/// News about one asset.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetNewsInput {
    /// Id of a registered asset (see `list_assets`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// Or the trading code (also outside the registry).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
}

impl ToolInput for AssetNewsInput {
    fn to_api_args(&self) -> Result<Value, String> {
        check_subject(&self.asset_id, &self.ticker)?;
        to_object(self)
    }
}

/// Text of one CVM IPE document.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadDocumentInput {
    /// Document id from `list_corporate_documents`.
    pub id: String,
    /// Character to start from (default 0; use `next_offset` from the previous part).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    /// How many characters (default 20,000, at most 40,000).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

impl ToolInput for ReadDocumentInput {
    fn to_api_args(&self) -> Result<Value, String> {
        super::inputs::check_non_empty("id", &self.id)?;
        to_object(self)
    }
}
