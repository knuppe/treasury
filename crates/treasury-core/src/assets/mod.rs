//! Cadastro de ativos (BR e internacionais) e termos de renda fixa.

pub mod api;
pub mod data_alert;
pub mod enrich;
pub mod labels;
pub mod lookup;
pub mod study;

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
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
    /// Ações listadas fora do Brasil e dos EUA (LSE, Xetra, Tóquio…).
    StockIntl,
    /// ETFs listados fora do Brasil e dos EUA.
    EtfIntl,
}

impl AssetClass {
    pub const ALL: [AssetClass; 15] = [
        AssetClass::StockBr,
        AssetClass::Fii,
        AssetClass::EtfBr,
        AssetClass::Bdr,
        AssetClass::FixedIncome,
        AssetClass::TreasuryBr,
        AssetClass::Fund,
        AssetClass::StockUs,
        AssetClass::EtfUs,
        AssetClass::Reit,
        AssetClass::Crypto,
        AssetClass::Pension,
        AssetClass::Other,
        AssetClass::StockIntl,
        AssetClass::EtfIntl,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            AssetClass::StockBr => "stock_br",
            AssetClass::Fii => "fii",
            AssetClass::EtfBr => "etf_br",
            AssetClass::Bdr => "bdr",
            AssetClass::FixedIncome => "fixed_income",
            AssetClass::TreasuryBr => "treasury_br",
            AssetClass::Fund => "fund",
            AssetClass::StockUs => "stock_us",
            AssetClass::EtfUs => "etf_us",
            AssetClass::Reit => "reit",
            AssetClass::Crypto => "crypto",
            AssetClass::Pension => "pension",
            AssetClass::Other => "other",
            AssetClass::StockIntl => "stock_intl",
            AssetClass::EtfIntl => "etf_intl",
        }
    }
    pub fn parse(s: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|c| c.as_str() == s)
            .ok_or_else(|| {
                Error::invalid(t_args(
                    "nucleo-ativos-erro-classe-invalida",
                    &args(&[("classe", s.to_string().into())]),
                ))
            })
    }
    /// Nome da classe no idioma da instalação (o mesmo rótulo da interface,
    /// `carteira-classe-<classe>`).
    pub fn label(self) -> String {
        t(&format!(
            "carteira-classe-{}",
            self.as_str().replace('_', "-")
        ))
    }
    /// Listado nos EUA (fundamentos da SEC, retenção de 30% nos dividendos).
    pub fn is_us(self) -> bool {
        matches!(
            self,
            AssetClass::StockUs | AssetClass::EtfUs | AssetClass::Reit
        )
    }
    /// Aplicação no exterior negociada em bolsa (EUA e demais bolsas fora do Brasil): entra
    /// no regime anual da Lei 14.754/2023, não na apuração mensal de renda variável.
    pub fn is_foreign_listed(self) -> bool {
        self.is_us() || matches!(self, AssetClass::StockIntl | AssetClass::EtfIntl)
    }
    /// Negociado em bolsa com ticker (o catálogo de bolsas dá moeda, país e símbolo).
    pub fn is_exchange_listed(self) -> bool {
        matches!(
            self,
            AssetClass::StockBr | AssetClass::Fii | AssetClass::EtfBr | AssetClass::Bdr
        ) || self.is_foreign_listed()
    }
    /// Avaliado pela curva (sem cotação de mercado diária).
    pub fn is_curve_valued(self) -> bool {
        matches!(self, AssetClass::FixedIncome)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FixedIncomeKind {
    Cdb,
    /// Recibo de Depósito Bancário (Nubank, Inter…): como o CDB, com FGC e IR regressivo.
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
    /// Isento de IR (e IOF) para pessoa física.
    pub fn default_ir_exempt(self) -> bool {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Indexer {
    Cdi,
    Selic,
    Ipca,
    Igpm,
    Prefixado,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Liquidity {
    Daily,
    AtMaturity,
    AfterGrace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FixedIncomeTerms {
    pub kind: FixedIncomeKind,
    pub indexer: Indexer,
    /// % do CDI/Selic (ex.: "110") ou taxa a.a. (prefixado "12.5"; IPCA + "6.2"). Decimal em texto.
    pub rate: String,
    pub issue_date: Option<String>,
    pub maturity_date: Option<String>,
    pub grace_until: Option<String>,
    pub liquidity: Liquidity,
    pub ir_exempt: bool,
    pub issuer: Option<String>,
}

impl FixedIncomeTerms {
    pub fn rate_dec(&self) -> Result<Decimal> {
        self.rate.trim().replace(',', ".").parse().map_err(|_| {
            Error::invalid(t_args(
                "nucleo-ativos-erro-taxa-invalida",
                &args(&[("taxa", self.rate.clone().into())]),
            ))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Asset {
    pub id: Id,
    pub ticker: Option<String>,
    /// Nome mostrado na interface (o que a pessoa escolheu).
    pub name: String,
    /// Nome oficial do registro (CVM, B3, seguradora), quando difere do mostrado.
    pub official_name: Option<String>,
    pub class: AssetClass,
    pub class_label: String,
    pub market: Option<String>,
    pub currency: String,
    pub country: Option<String>,
    pub isin: Option<String>,
    pub cnpj: Option<String>,
    pub treasury_code: Option<String>,
    pub sector: Option<String>,
    pub segment: Option<String>,
    pub price_provider: Option<String>,
    pub provider_symbol: Option<String>,
    pub archived: bool,
    pub notes: Option<String>,
    pub fixed_income: Option<FixedIncomeTerms>,
    /// Conta onde o ativo fica (corretora, banco ou conta no exterior); a operação usa essa
    /// conta por padrão.
    pub account_id: Option<Id>,
    /// Bolsa do catálogo cujo código é o `market` do ativo.
    pub exchange: Option<ExchangeBrief>,
    /// Símbolo usado na coleta de cotações do Yahoo (ticker + sufixo da bolsa), se houver.
    pub quote_symbol: Option<String>,
}

/// Resumo da bolsa de um ativo (o catálogo completo está em `market.exchanges`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExchangeBrief {
    pub code: String,
    pub name: String,
    pub country: String,
    pub timezone: String,
    pub yahoo_suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetInput {
    #[ts(optional)]
    pub ticker: Option<String>,
    pub name: String,
    /// Nome oficial do registro, quando o `name` é um apelido.
    #[serde(default)]
    #[ts(optional)]
    pub official_name: Option<String>,
    pub class: AssetClass,
    /// Código da bolsa (B3, NYSE, LSE, TSE…; ver `market.exchanges`) ou CRYPTO/TESOURO/CVM.
    /// Obrigatório para `stock_intl` e `etf_intl`.
    #[ts(optional)]
    pub market: Option<String>,
    /// Padrão: a moeda da bolsa; sem bolsa, BRL para classes brasileiras, USD para EUA e cripto
    /// e a moeda base da família para renda fixa, previdência e outros.
    #[ts(optional)]
    pub currency: Option<String>,
    #[ts(optional)]
    pub country: Option<String>,
    #[ts(optional)]
    pub isin: Option<String>,
    #[ts(optional)]
    pub cnpj: Option<String>,
    #[ts(optional)]
    pub treasury_code: Option<String>,
    #[ts(optional)]
    pub sector: Option<String>,
    #[ts(optional)]
    pub segment: Option<String>,
    #[ts(optional)]
    pub price_provider: Option<String>,
    #[ts(optional)]
    pub provider_symbol: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
    /// Obrigatório para `fixed_income`; opcional para `treasury_br` (avaliado pela curva quando não há PU).
    #[ts(optional)]
    pub fixed_income: Option<FixedIncomeTerms>,
    /// Plano de previdência (VGBL/PGBL), só para `pension`: grava junto com o ativo.
    #[serde(default)]
    #[ts(optional)]
    pub pension: Option<crate::pension::PensionPlanTerms>,
    /// Conta onde o ativo fica (custódia): corretora, banco ou conta no exterior.
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub official_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub ticker: Option<Option<String>>,
    #[ts(optional)]
    pub class: Option<AssetClass>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub market: Option<Option<String>>,
    /// Só enquanto o ativo não tem operações nem proventos.
    #[ts(optional)]
    pub currency: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub country: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub isin: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub cnpj: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub treasury_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub sector: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub segment: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub price_provider: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub provider_symbol: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
    #[ts(optional)]
    pub archived: Option<bool>,
    #[ts(optional)]
    pub fixed_income: Option<FixedIncomeTerms>,
    /// Plano de previdência (só `pension`): os campos informados mudam, os outros ficam.
    #[serde(default)]
    #[ts(optional)]
    pub pension: Option<crate::pension::PensionPlanPatch>,
    /// Conta de custódia (`null` desliga).
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub account_id: Option<Option<Id>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetFilter {
    /// Busca em ticker, nome e nome oficial.
    #[ts(optional)]
    pub text: Option<String>,
    #[ts(optional)]
    pub classes: Option<Vec<AssetClass>>,
    #[ts(optional)]
    pub include_archived: Option<bool>,
    /// Só ativos com posição ou operações.
    #[ts(optional)]
    pub held_only: Option<bool>,
    /// Posição hoje: em carteira (quantidade > 0), encerrada (teve operações e zerou) ou
    /// com operações (as duas). Sem ele, todos os cadastrados.
    #[ts(optional)]
    pub position: Option<PositionFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PositionFilter {
    Open,
    Closed,
    Traded,
}

/// Moeda sem bolsa: real nas classes brasileiras (fundo vai para a CVM), dólar nas dos EUA e
/// em cripto; renda fixa, previdência e outros, a moeda base da família (`base`).
fn default_currency(class: AssetClass, base: &str) -> String {
    match class {
        AssetClass::StockUs | AssetClass::EtfUs | AssetClass::Reit | AssetClass::Crypto => {
            "USD".into()
        }
        // Sem bolsa não há moeda padrão; o cadastro exige a bolsa para estas classes.
        AssetClass::StockIntl | AssetClass::EtfIntl => "USD".into(),
        AssetClass::StockBr
        | AssetClass::Fii
        | AssetClass::EtfBr
        | AssetClass::Bdr
        | AssetClass::TreasuryBr
        | AssetClass::Fund => "BRL".into(),
        AssetClass::FixedIncome | AssetClass::Pension | AssetClass::Other => base.to_string(),
    }
}

/// Mercados fora do catálogo de bolsas que o app usa sozinho (os padrões das classes).
const OTHER_MARKETS: &[&str] = &["TESOURO", "CVM", "CRYPTO"];

/// Mercado que não está no catálogo de bolsas: só os de `OTHER_MARKETS` ou o que o ativo já
/// tem; o resto é erro com a lista (antes de qualquer gravação ou proposta).
fn known_market(conn: &Connection, id: &str, m: &str) -> Result<String> {
    let up = m.trim().to_uppercase();
    let current: Option<String> = conn
        .query_row("SELECT market FROM asset WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .flatten();
    if OTHER_MARKETS.contains(&up.as_str()) || current.as_deref() == Some(up.as_str()) {
        return Ok(up);
    }
    let mut codes: Vec<String> = crate::market::exchange::list(conn)?
        .into_iter()
        .map(|e| e.code)
        .collect();
    codes.extend(OTHER_MARKETS.iter().map(|s| s.to_string()));
    Err(Error::invalid(t_args(
        "nucleo-ativos-erro-mercado",
        &args(&[
            ("mercado", m.trim().to_string().into()),
            ("lista", codes.join(", ").into()),
        ]),
    )))
}

fn default_market(class: AssetClass) -> Option<&'static str> {
    match class {
        AssetClass::StockBr | AssetClass::Fii | AssetClass::EtfBr | AssetClass::Bdr => Some("B3"),
        AssetClass::TreasuryBr => Some("TESOURO"),
        AssetClass::Fund => Some("CVM"),
        AssetClass::Crypto => Some("CRYPTO"),
        AssetClass::FixedIncome
        | AssetClass::StockUs
        | AssetClass::EtfUs
        | AssetClass::Reit
        | AssetClass::Pension
        | AssetClass::Other
        | AssetClass::StockIntl
        | AssetClass::EtfIntl => None,
    }
}

fn liquidity_str(l: Liquidity) -> &'static str {
    match l {
        Liquidity::Daily => "daily",
        Liquidity::AtMaturity => "at_maturity",
        Liquidity::AfterGrace => "after_grace",
    }
}

fn enum_str<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

fn from_str<T: serde::de::DeserializeOwned>(s: &str) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn row_to_asset(r: &Row<'_>) -> rusqlite::Result<Asset> {
    let class = AssetClass::parse(&r.get::<_, String>("class")?).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let fi_kind: Option<String> = r.get("fi_kind")?;
    let fixed_income = match fi_kind {
        Some(k) => Some(FixedIncomeTerms {
            kind: from_str(&k)?,
            indexer: from_str(&r.get::<_, String>("fi_indexer")?)?,
            rate: r.get("fi_rate")?,
            issue_date: r.get("fi_issue_date")?,
            maturity_date: r.get("fi_maturity_date")?,
            grace_until: r.get("fi_grace_until")?,
            liquidity: from_str(&r.get::<_, String>("fi_liquidity")?)?,
            ir_exempt: r.get::<_, i64>("fi_ir_exempt")? == 1,
            issuer: r.get("fi_issuer")?,
        }),
        None => None,
    };
    let exchange = match r.get::<_, Option<String>>("ex_code")? {
        Some(code) => Some(ExchangeBrief {
            code,
            name: r.get("ex_name")?,
            country: r.get("ex_country")?,
            timezone: r.get("ex_timezone")?,
            yahoo_suffix: r.get("ex_suffix")?,
        }),
        None => None,
    };
    let mut asset = Asset {
        id: r.get("id")?,
        ticker: r.get("ticker")?,
        name: r.get("name")?,
        official_name: r.get("official_name")?,
        account_id: r.get("account_id")?,
        class_label: class.label(),
        class,
        market: r.get("market")?,
        currency: r.get("currency")?,
        country: r.get("country")?,
        isin: r.get("isin")?,
        cnpj: r.get("cnpj")?,
        treasury_code: r.get("treasury_code")?,
        sector: r.get("sector")?,
        segment: r.get("segment")?,
        price_provider: r.get("price_provider")?,
        provider_symbol: r.get("provider_symbol")?,
        archived: r.get::<_, i64>("archived")? == 1,
        notes: r.get("notes")?,
        fixed_income,
        exchange,
        quote_symbol: None,
    };
    asset.quote_symbol = match crate::market::sync::default_source(&asset) {
        Some((crate::market::sync::Provider::Yahoo, s)) => Some(s),
        _ => None,
    };
    Ok(asset)
}

/// Conta de custódia válida (existe e não é cartão de crédito); vazio vira `None`.
fn custody(tx: &Connection, id: Option<&str>) -> Result<Option<String>> {
    let Some(id) = id.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let kind: Option<String> = tx
        .query_row("SELECT kind FROM account WHERE id = ?1", [id], |r| r.get(0))
        .optional()?;
    match kind.as_deref() {
        None => Err(Error::not_found(crate::i18n::t_args(
            "nucleo-ativos-erro-conta-custodia",
            &crate::i18n::args(&[("conta", id.to_string().into())]),
        ))),
        Some("credit_card") => Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-custodia-cartao",
        ))),
        Some(_) => Ok(Some(id.to_string())),
    }
}

const SELECT: &str = "SELECT a.*, f.kind AS fi_kind, f.indexer AS fi_indexer, f.rate AS fi_rate, f.issue_date AS fi_issue_date,
        f.maturity_date AS fi_maturity_date, f.grace_until AS fi_grace_until, f.liquidity AS fi_liquidity,
        f.ir_exempt AS fi_ir_exempt, f.issuer AS fi_issuer,
        e.code AS ex_code, e.name AS ex_name, e.country AS ex_country, e.timezone AS ex_timezone,
        e.yahoo_suffix AS ex_suffix
    FROM asset a LEFT JOIN fixed_income_terms f ON f.asset_id = a.id
        LEFT JOIN exchange e ON e.code = a.market";

pub fn get(conn: &Connection, id: &str) -> Result<Asset> {
    conn.query_row(&format!("{SELECT} WHERE a.id = ?1"), [id], row_to_asset)
        .optional()?
        .ok_or_else(|| Error::not_found(asset_ref(id)))
}

pub fn find_by_ticker(conn: &Connection, ticker: &str) -> Result<Option<Asset>> {
    Ok(conn
        .query_row(
            &format!("{SELECT} WHERE upper(a.ticker) = upper(?1) ORDER BY a.archived LIMIT 1"),
            [ticker.trim()],
            row_to_asset,
        )
        .optional()?)
}

pub fn list(conn: &Connection, f: &AssetFilter) -> Result<Vec<Asset>> {
    let mut sql = format!("{SELECT} WHERE 1 = 1");
    let mut args: Vec<String> = vec![];
    if !f.include_archived.unwrap_or(false) {
        sql.push_str(" AND a.archived = 0");
    }
    if let Some(t) = f.text.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        args.push(format!("%{}%", t.to_lowercase()));
        sql.push_str(&format!(
            " AND (lower(a.ticker) LIKE ?{0} OR lower(a.name) LIKE ?{0}
                   OR lower(a.official_name) LIKE ?{0})",
            args.len()
        ));
    }
    if let Some(classes) = f.classes.as_ref().filter(|c| !c.is_empty()) {
        let list = classes
            .iter()
            .map(|c| format!("'{}'", c.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        sql.push_str(&format!(" AND a.class IN ({list})"));
    }
    if f.held_only.unwrap_or(false)
        || matches!(
            f.position,
            Some(PositionFilter::Closed | PositionFilter::Traded)
        )
    {
        sql.push_str(" AND EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id)");
    } else if f.position == Some(PositionFilter::Open) {
        // Plano de previdência cadastrado sem aporte lançado ainda é da carteira (ninguém
        // acompanha um VGBL de longe): aparece em "Em carteira" para receber os aportes.
        sql.push_str(
            " AND (EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id) OR a.class = 'pension')",
        );
    }
    sql.push_str(" ORDER BY a.class, coalesce(a.ticker, a.name)");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(args.iter()), row_to_asset)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let want_open = match f.position {
        Some(PositionFilter::Open) => true,
        Some(PositionFilter::Closed) => false,
        _ => return Ok(rows),
    };
    // A quantidade de hoje vem do histórico (desdobramentos, bonificações, vendas).
    let today = crate::time::fmt_date(crate::time::today_local());
    let mut out = Vec::with_capacity(rows.len());
    for a in rows {
        let q = crate::portfolio::engine::quantity_held(conn, &a.id, None, &today)?;
        let plan_without_trades = a.class == AssetClass::Pension
            && !conn.query_row(
                "SELECT EXISTS (SELECT 1 FROM trade WHERE asset_id = ?1)",
                [&a.id],
                |r| r.get::<_, bool>(0),
            )?;
        if (q > rust_decimal::Decimal::ZERO || plan_without_trades) == want_open {
            out.push(a);
        }
    }
    Ok(out)
}

/// "ativo <id>" para os erros de não encontrado.
fn asset_ref(id: &str) -> String {
    t_args(
        "nucleo-ativos-ref-ativo",
        &args(&[("id", id.to_string().into())]),
    )
}

/// Nome oficial só quando difere do mostrado (sem distinguir maiúsculas).
fn official(s: &Option<String>, name: &str) -> Option<String> {
    clean(s).filter(|o| o.to_lowercase() != name.trim().to_lowercase())
}

fn clean(s: &Option<String>) -> Option<String> {
    s.as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn write_terms(tx: &Transaction<'_>, asset_id: &str, t: &FixedIncomeTerms) -> Result<()> {
    t.rate_dec()?;
    for d in [&t.issue_date, &t.maturity_date, &t.grace_until]
        .into_iter()
        .flatten()
    {
        crate::time::parse_date(d)?;
    }
    let now = now_utc();
    tx.execute(
        "INSERT INTO fixed_income_terms (asset_id, kind, indexer, rate, issue_date, maturity_date, grace_until, liquidity, ir_exempt, issuer, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
         ON CONFLICT (asset_id) DO UPDATE SET kind = excluded.kind, indexer = excluded.indexer, rate = excluded.rate,
             issue_date = excluded.issue_date, maturity_date = excluded.maturity_date, grace_until = excluded.grace_until,
             liquidity = excluded.liquidity, ir_exempt = excluded.ir_exempt, issuer = excluded.issuer, updated_at = excluded.updated_at",
        params![
            asset_id,
            enum_str(&t.kind),
            enum_str(&t.indexer),
            t.rate_dec()?.normalize().to_string(),
            t.issue_date,
            t.maturity_date,
            t.grace_until,
            liquidity_str(t.liquidity),
            t.ir_exempt as i64,
            t.issuer,
            now
        ],
    )?;
    Ok(())
}

pub fn create(tx: &Transaction<'_>, a: &AssetInput) -> Result<Id> {
    let name = a.name.trim();
    if name.is_empty() {
        return Err(Error::invalid(t("nucleo-ativos-erro-nome")));
    }
    if a.class == AssetClass::FixedIncome && a.fixed_income.is_none() {
        return Err(Error::invalid(t("nucleo-ativos-erro-renda-fixa-termos")));
    }
    if a.pension.is_some() && a.class != AssetClass::Pension {
        return Err(Error::invalid(t("nucleo-ativos-erro-previdencia-classe")));
    }
    let ticker = clean(&a.ticker).map(|t| t.to_uppercase());
    // A bolsa do catálogo (pelo código, nome ou apelido) normaliza o mercado e dá a moeda e o
    // país padrão das classes negociadas em bolsa.
    let exchange = match clean(&a.market) {
        Some(m) => crate::market::exchange::resolve(tx, &m)?,
        None => None,
    };
    if matches!(a.class, AssetClass::StockIntl | AssetClass::EtfIntl) && exchange.is_none() {
        return Err(Error::invalid(match clean(&a.market) {
            Some(m) => t_args(
                "nucleo-ativos-erro-bolsa-fora",
                &args(&[("bolsa", m.into())]),
            ),
            None => t("nucleo-ativos-erro-bolsa-falta"),
        }));
    }
    let market = match &exchange {
        Some(e) => Some(e.code.clone()),
        None => clean(&a.market)
            .map(|m| m.to_uppercase())
            .or_else(|| default_market(a.class).map(String::from)),
    };
    if let Some(t) = &ticker {
        let dup: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM asset WHERE upper(ticker) = ?1 AND coalesce(market, '') = coalesce(?2, ''))",
            params![t, market],
            |r| r.get(0),
        )?;
        if dup {
            return Err(Error::conflict(t_args(
                "nucleo-ativos-erro-duplicado",
                &args(&[("ticker", t.clone().into())]),
            )));
        }
    }
    let listed_ex = exchange.as_ref().filter(|_| a.class.is_exchange_listed());
    let fallback = default_currency(a.class, &crate::household::base_currency(tx)?);
    let currency = crate::money::validate_currency(
        clean(&a.currency)
            .as_deref()
            .or(listed_ex.map(|e| e.currency.as_str()))
            .unwrap_or(&fallback),
    )?;
    let country = clean(&a.country)
        .map(|c| c.to_uppercase())
        .or_else(|| listed_ex.map(|e| e.country.clone()));
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO asset (id, ticker, name, class, market, currency, country, isin, cnpj, treasury_code, sector, segment,
                            price_provider, provider_symbol, notes, created_at, updated_at, official_name, account_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16, ?17, ?18)",
        params![
            id,
            ticker,
            name,
            a.class.as_str(),
            market,
            currency,
            country,
            clean(&a.isin),
            clean(&a.cnpj),
            clean(&a.treasury_code),
            clean(&a.sector),
            clean(&a.segment),
            clean(&a.price_provider),
            clean(&a.provider_symbol),
            clean(&a.notes),
            now,
            official(&a.official_name, name),
            custody(tx, a.account_id.as_deref())?,
        ],
    )?;
    if let Some(t) = &a.fixed_income {
        write_terms(tx, &id, t)?;
    }
    if let Some(p) = &a.pension {
        crate::pension::set_plan(tx, &p.for_asset(&id))?;
    }
    Ok(id)
}

pub fn update(tx: &Transaction<'_>, p: &AssetPatch) -> Result<()> {
    let before = get(tx, &p.id)?;
    let now = now_utc();
    if let Some(n) = &p.name {
        if n.trim().is_empty() {
            return Err(Error::invalid(t("nucleo-ativos-erro-nome")));
        }
        tx.execute(
            "UPDATE asset SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, n.trim(), now],
        )?;
    }
    let set_opt = |col: &str, v: &Option<Option<String>>| -> Result<()> {
        if let Some(v) = v {
            tx.execute(
                &format!("UPDATE asset SET {col} = ?2, updated_at = ?3 WHERE id = ?1"),
                params![p.id, clean(v), now],
            )?;
        }
        Ok(())
    };
    set_opt(
        "ticker",
        &p.ticker
            .as_ref()
            .map(|t| t.as_ref().map(|s| s.to_uppercase())),
    )?;
    let market = match &p.market {
        Some(Some(m)) if !m.trim().is_empty() => {
            Some(Some(match crate::market::exchange::resolve(tx, m)? {
                Some(e) => e.code,
                None => known_market(tx, &p.id, m)?,
            }))
        }
        other => other.clone(),
    };
    set_opt("market", &market)?;
    set_opt(
        "country",
        &p.country
            .as_ref()
            .map(|t| t.as_ref().map(|s| s.to_uppercase())),
    )?;
    set_opt(
        "isin",
        &p.isin
            .as_ref()
            .map(|t| t.as_ref().map(|s| s.to_uppercase())),
    )?;
    set_opt("cnpj", &p.cnpj)?;
    set_opt("treasury_code", &p.treasury_code)?;
    if let Some(c) = &p.currency {
        let c = c.trim().to_uppercase();
        if c.len() != 3 || !c.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Err(Error::invalid(t_args(
                "nucleo-ativos-erro-moeda-invalida",
                &args(&[("moeda", c.into())]),
            )));
        }
        let current: String =
            tx.query_row("SELECT currency FROM asset WHERE id = ?1", [&p.id], |r| {
                r.get(0)
            })?;
        if current != c {
            let used: bool = tx.query_row(
                "SELECT EXISTS (SELECT 1 FROM trade WHERE asset_id = ?1) OR EXISTS (SELECT 1 FROM income_event WHERE asset_id = ?1)",
                [&p.id],
                |r| r.get(0),
            )?;
            if used {
                return Err(Error::conflict(t("nucleo-ativos-erro-moeda-em-uso")));
            }
            tx.execute(
                "UPDATE asset SET currency = ?2, updated_at = ?3 WHERE id = ?1",
                params![p.id, c, now],
            )?;
        }
    }
    set_opt("sector", &p.sector)?;
    set_opt("segment", &p.segment)?;
    set_opt("price_provider", &p.price_provider)?;
    set_opt("provider_symbol", &p.provider_symbol)?;
    set_opt("notes", &p.notes)?;
    if let Some(o) = &p.official_name {
        let name: String = tx.query_row("SELECT name FROM asset WHERE id = ?1", [&p.id], |r| {
            r.get(0)
        })?;
        tx.execute(
            "UPDATE asset SET official_name = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, official(o, &name), now],
        )?;
    }
    if let Some(acc) = &p.account_id {
        tx.execute(
            "UPDATE asset SET account_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, custody(tx, acc.as_deref())?, now],
        )?;
    }
    if let Some(c) = p.class {
        tx.execute(
            "UPDATE asset SET class = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, c.as_str(), now],
        )?;
    }
    if let Some(a) = p.archived {
        tx.execute(
            "UPDATE asset SET archived = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, a as i64, now],
        )?;
    }
    if let Some(t) = &p.fixed_income {
        write_terms(tx, &p.id, t)?;
    }
    let after = get(tx, &p.id)?;
    // Campo mudado à mão passa a ter origem `user` (o enriquecimento só mostra divergência).
    for (field, old, new) in [
        ("sector", &before.sector, &after.sector),
        ("segment", &before.segment, &after.segment),
        ("isin", &before.isin, &after.isin),
        ("cnpj", &before.cnpj, &after.cnpj),
    ] {
        let n = |v: &Option<String>| v.as_deref().map(|x| enrich::normalize(field, x));
        if n(old) != n(new) {
            enrich::mark_user(tx, &p.id, field, new.as_deref())?;
        }
    }
    if matches!(after.class, AssetClass::StockIntl | AssetClass::EtfIntl)
        && after.exchange.is_none()
    {
        return Err(Error::invalid(t("nucleo-ativos-erro-bolsa-catalogo")));
    }
    if let Some(pp) = &p.pension {
        crate::pension::patch_plan(tx, &p.id, pp)?;
    }
    Ok(())
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    let used: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM trade WHERE asset_id = ?1) OR EXISTS (SELECT 1 FROM income_event WHERE asset_id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    if used {
        return Err(Error::conflict(t("nucleo-ativos-erro-apagar-em-uso")));
    }
    tx.execute("DELETE FROM watchlist_item WHERE asset_id = ?1", [id])?;
    tx.execute("DELETE FROM asset_field_meta WHERE asset_id = ?1", [id])?;
    tx.execute("DELETE FROM asset_attribute WHERE asset_id = ?1", [id])?;
    tx.execute("DELETE FROM asset_observation WHERE asset_id = ?1", [id])?;
    tx.execute("DELETE FROM asset_study WHERE asset_id = ?1", [id])?;
    tx.execute("DELETE FROM asset_study_context WHERE asset_id = ?1", [id])?;
    tx.execute("DELETE FROM corporate_action WHERE asset_id = ?1", [id])?;
    tx.execute(
        "DELETE FROM target_allocation WHERE scope = 'asset' AND key = ?1",
        [id],
    )?;
    let n = tx.execute("DELETE FROM asset WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(Error::not_found(asset_ref(id)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Locale, has_key, set_current};

    #[test]
    fn class_labels_and_errors_follow_the_locale() {
        for c in AssetClass::ALL {
            let key = format!("carteira-classe-{}", c.as_str().replace('_', "-"));
            for l in [Locale::PtBr, Locale::En, Locale::Es] {
                assert!(has_key(l, &key), "{l:?}: {key}");
            }
        }
        assert_eq!(AssetClass::StockUs.label(), "Ações nos EUA");
        set_current(Locale::En);
        assert_eq!(AssetClass::StockBr.label(), "Stocks");
        let e = AssetClass::parse("xyz").unwrap_err();
        assert_eq!(e.message(), "invalid asset class: xyz");
        set_current(Locale::PtBr);
    }
}
