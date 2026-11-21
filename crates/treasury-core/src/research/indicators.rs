//! Indicadores fundamentalistas e preço justo (Graham e Bazin), calculados a partir dos
//! fatos do emissor, da cotação e dos proventos. Funções puras sobre [`Data`] (carregado em
//! [`super::data`]): cada indicador sai com fórmula, período e fonte, ou com o motivo de
//! estar indisponível.
//!
//! Convenções:
//! - Fluxos (lucro, receita, EBIT…) são dos últimos 12 meses (LTM); saldos são do último
//!   balanço. Lucro e patrimônio são os atribuídos aos controladores.
//! - Valor de mercado soma cada classe de ação pela cotação da própria classe quando há
//!   cotação de todas; senão usa o total de ações × cotação do papel consultado.
//! - Units: indicadores por ação são multiplicados pelas ações da unit.
//! - Bancos e seguradoras: sem EBITDA, valor da firma, dívida líquida, margens nem liquidez
//!   corrente (marcados como indisponíveis). Índice de Basileia não está nos arquivos da CVM.
//! - Razões em fração (0,083 = 8,3%); múltiplos em vezes; dinheiro na moeda do ativo.

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, NaiveDate};
use rust_decimal::{Decimal, MathematicalOps};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Fact, Layout, PeriodType, dstr, items, periods, to_f64};
use crate::assets::AssetClass;
use crate::i18n::{args, t, t_args};
use crate::market::PriceOn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum IndicatorGroup {
    Market,
    Valuation,
    Profitability,
    Debt,
    Growth,
    PerShare,
    Dividends,
    FairPrice,
    Fii,
    Bank,
    /// Score de saúde em lote (`market_score`), só no universo (screener e rankings).
    Score,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum IndicatorUnit {
    /// Valor na moeda do ativo (`currency`).
    Money,
    /// Múltiplo (vezes).
    Multiple,
    /// Fração (0,083 = 8,3%).
    Percent,
    /// Contagem.
    Count,
}

/// A quem o indicador se aplica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Stock,
    Fii,
    Both,
}

/// Um indicador do catálogo. O nome e a fórmula ficam no catálogo de idiomas
/// (`nucleo-pesquisa-ind-<chave>` e `nucleo-pesquisa-ind-<chave>-formula`).
pub struct IndicatorDef {
    pub key: &'static str,
    pub group: IndicatorGroup,
    pub unit: IndicatorUnit,
    /// Ordem padrão do ranking: `Some(true)` maior é melhor, `Some(false)` menor é melhor.
    pub higher_is_better: Option<bool>,
    /// No ranking, ignora valores ≤ 0 (múltiplos com lucro negativo não são "baratos").
    pub positive_only: bool,
    pub scope: Scope,
}

impl IndicatorDef {
    /// Nome no idioma da instalação.
    pub fn label(&self) -> String {
        t(&format!("nucleo-pesquisa-ind-{}", self.key))
    }

    /// Fórmula no idioma da instalação.
    pub fn formula(&self) -> String {
        t(&format!("nucleo-pesquisa-ind-{}-formula", self.key))
    }
}

use IndicatorGroup as G;
use IndicatorUnit as U;

macro_rules! def {
    ($key:expr, $group:expr, $unit:expr, $hib:expr, $pos:expr, $scope:expr) => {
        IndicatorDef {
            key: $key,
            group: $group,
            unit: $unit,
            higher_is_better: $hib,
            positive_only: $pos,
            scope: $scope,
        }
    };
}

/// Catálogo de indicadores (ordem de exibição).
pub static CATALOG: &[IndicatorDef] = &[
    def!("price", G::Market, U::Money, None, false, Scope::Both),
    def!(
        "market_cap",
        G::Market,
        U::Money,
        Some(true),
        true,
        Scope::Both
    ),
    def!("ev", G::Market, U::Money, None, false, Scope::Stock),
    def!(
        "liquidity",
        G::Market,
        U::Money,
        Some(true),
        true,
        Scope::Both
    ),
    def!(
        "pl",
        G::Valuation,
        U::Multiple,
        Some(false),
        true,
        Scope::Stock
    ),
    def!(
        "pvp",
        G::Valuation,
        U::Multiple,
        Some(false),
        true,
        Scope::Both
    ),
    def!(
        "ev_ebitda",
        G::Valuation,
        U::Multiple,
        Some(false),
        true,
        Scope::Stock
    ),
    def!(
        "ev_ebit",
        G::Valuation,
        U::Multiple,
        Some(false),
        true,
        Scope::Stock
    ),
    def!(
        "psr",
        G::Valuation,
        U::Multiple,
        Some(false),
        true,
        Scope::Stock
    ),
    def!(
        "p_ativo",
        G::Valuation,
        U::Multiple,
        Some(false),
        true,
        Scope::Stock
    ),
    def!(
        "dy",
        G::Dividends,
        U::Percent,
        Some(true),
        false,
        Scope::Both
    ),
    def!(
        "payout",
        G::Dividends,
        U::Percent,
        None,
        false,
        Scope::Stock
    ),
    def!(
        "last_dividend",
        G::Dividends,
        U::Money,
        Some(true),
        false,
        Scope::Fii
    ),
    def!(
        "roe",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "roic",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "roa",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "gross_margin",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "ebitda_margin",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "ebit_margin",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "net_margin",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "net_debt_equity",
        G::Debt,
        U::Multiple,
        Some(false),
        false,
        Scope::Stock
    ),
    def!(
        "net_debt_ebitda",
        G::Debt,
        U::Multiple,
        Some(false),
        false,
        Scope::Stock
    ),
    def!(
        "current_ratio",
        G::Debt,
        U::Multiple,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "revenue_cagr_5y",
        G::Growth,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "earnings_cagr_5y",
        G::Growth,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "loss_years_10y",
        G::Growth,
        U::Count,
        Some(false),
        false,
        Scope::Stock
    ),
    // Score em lote do universo (docs/design/market-explorer.md, 4.2): valores vindos de
    // `market_score`, não calculados em `compute`.
    def!("score", G::Score, U::Count, Some(true), false, Scope::Both),
    def!("health", G::Score, U::Count, Some(true), false, Scope::Both),
    def!(
        "attractiveness",
        G::Score,
        U::Count,
        Some(true),
        false,
        Scope::Both
    ),
    def!(
        "confidence",
        G::Score,
        U::Percent,
        Some(true),
        false,
        Scope::Both
    ),
    def!(
        "piotroski",
        G::Score,
        U::Count,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "altman_z",
        G::Score,
        U::Multiple,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "beneish",
        G::Score,
        U::Multiple,
        Some(false),
        false,
        Scope::Stock
    ),
    def!(
        "momentum_12_1",
        G::Score,
        U::Percent,
        Some(true),
        false,
        Scope::Both
    ),
    def!(
        "volatility_1y",
        G::Score,
        U::Percent,
        Some(false),
        false,
        Scope::Both
    ),
    // Indicadores dos métodos do explorador (docs/design/market-explorer.md, 4.3).
    def!(
        "earnings_yield",
        G::Valuation,
        U::Percent,
        Some(true),
        true,
        Scope::Stock
    ),
    def!(
        "magic_roc",
        G::Profitability,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "fcf_yield",
        G::Valuation,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "peg",
        G::Valuation,
        U::Multiple,
        Some(false),
        true,
        Scope::Stock
    ),
    def!(
        "eps_growth_10y",
        G::Growth,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "dividend_years",
        G::Dividends,
        U::Count,
        Some(true),
        false,
        Scope::Both
    ),
    def!(
        "dividend_cagr_5y",
        G::Dividends,
        U::Percent,
        Some(true),
        false,
        Scope::Both
    ),
    def!(
        "lpa",
        G::PerShare,
        U::Money,
        Some(true),
        false,
        Scope::Stock
    ),
    def!(
        "vpa",
        G::PerShare,
        U::Money,
        Some(true),
        false,
        Scope::Stock
    ),
    def!("nav_per_share", G::Fii, U::Money, None, false, Scope::Fii),
    def!("holders", G::Fii, U::Count, Some(true), false, Scope::Fii),
    def!(
        "vacancy",
        G::Fii,
        U::Percent,
        Some(false),
        false,
        Scope::Fii
    ),
    def!(
        "basel",
        G::Bank,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!("graham", G::FairPrice, U::Money, None, false, Scope::Stock),
    def!(
        "graham_upside",
        G::FairPrice,
        U::Percent,
        Some(true),
        false,
        Scope::Stock
    ),
    def!("bazin", G::FairPrice, U::Money, None, false, Scope::Both),
    def!(
        "bazin_upside",
        G::FairPrice,
        U::Percent,
        Some(true),
        false,
        Scope::Both
    ),
];

pub fn def(key: &str) -> Option<&'static IndicatorDef> {
    CATALOG.iter().find(|d| d.key == key)
}

/// Descrição de um indicador para a interface e para o agente.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IndicatorInfo {
    pub key: String,
    pub label: String,
    pub group: IndicatorGroup,
    pub unit: IndicatorUnit,
    pub formula: String,
    pub higher_is_better: Option<bool>,
    /// `stock`, `fii` ou ambos.
    pub applies_to: Vec<String>,
}

pub fn catalog() -> Vec<IndicatorInfo> {
    CATALOG
        .iter()
        .map(|d| IndicatorInfo {
            key: d.key.into(),
            label: d.label(),
            group: d.group,
            unit: d.unit,
            formula: d.formula(),
            higher_is_better: d.higher_is_better,
            applies_to: match d.scope {
                Scope::Stock => vec!["stock".into()],
                Scope::Fii => vec!["fii".into()],
                Scope::Both => vec!["stock".into(), "fii".into()],
            },
        })
        .collect()
}

/// Um indicador calculado (ou indisponível, com o motivo).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Indicator {
    pub key: String,
    pub label: String,
    pub group: IndicatorGroup,
    pub unit: IndicatorUnit,
    /// Para ordenar e filtrar (fração, múltiplo, contagem ou valor na moeda).
    pub value: Option<f64>,
    /// Valor exato em decimal texto (dinheiro e por ação).
    pub exact: Option<String>,
    pub currency: Option<String>,
    pub formula: String,
    /// Período em português (ex.: "12 meses até 30/06/2026", "saldo em 30/06/2026").
    pub period: Option<String>,
    pub period_end: Option<String>,
    /// Documentos e cotação usados.
    pub source: Option<String>,
    /// Estimado (ex.: rendimento de FII calculado a partir do informe mensal).
    pub estimated: bool,
    pub unavailable_reason: Option<String>,
}

/// Ativo analisado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Subject {
    pub asset_id: Option<String>,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub currency: String,
    pub issuer_id: Option<String>,
    pub issuer_name: Option<String>,
    /// CNPJ formatado (companhias e FIIs) ou CIK (`cik:…`).
    pub issuer_code: Option<String>,
    pub sector: Option<String>,
    pub segment: Option<String>,
    pub mandate: Option<String>,
    pub layout: Option<Layout>,
    /// ON, PN, UNIT, CTF.
    pub share_kind: Option<String>,
}

impl Subject {
    pub fn is_fii(&self) -> bool {
        self.class == AssetClass::Fii
    }
}

/// Cotação de outra classe do mesmo emissor (para o valor de mercado).
#[derive(Debug, Clone, PartialEq)]
pub struct Sibling {
    pub ticker: String,
    pub share_kind: String,
    pub close: Decimal,
}

/// Provento por cota/ação do papel consultado.
#[derive(Debug, Clone, PartialEq)]
pub struct DividendPoint {
    pub date: NaiveDate,
    pub amount: Decimal,
}

/// Tudo o que os indicadores precisam (sem acesso ao banco).
#[derive(Debug, Clone)]
pub struct Data {
    pub subject: Subject,
    pub facts: Vec<Fact>,
    pub price: Option<(Decimal, PriceOn)>,
    /// Ações por unit (1 para ON/PN).
    pub per_unit: Option<Decimal>,
    pub siblings: Vec<Sibling>,
    pub dividends: Vec<DividendPoint>,
    /// De onde vieram os proventos (`dividend_event` do Yahoo, lançamentos…).
    pub dividends_source: Option<String>,
    pub liquidity: Option<(Decimal, String)>,
    pub today: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IndicatorsView {
    pub subject: Subject,
    pub price: Option<PriceOn>,
    pub as_of: String,
    /// Fim do período dos últimos 12 meses usado nos fluxos.
    pub reference_date: Option<String>,
    pub indicators: Vec<Indicator>,
    pub notes: Vec<String>,
}

/// Premissa de um preço justo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Assumption {
    pub label: String,
    pub value: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FairPriceMethod {
    /// `graham` ou `bazin`.
    pub method: String,
    pub label: String,
    /// Preço justo por ação/cota/unit (decimal texto).
    pub value: Option<String>,
    pub currency: String,
    /// Preço justo ÷ cotação − 1.
    pub upside: Option<f64>,
    pub formula: String,
    pub assumptions: Vec<Assumption>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FairPriceView {
    pub subject: Subject,
    pub price: Option<PriceOn>,
    pub graham: FairPriceMethod,
    pub bazin: FairPriceMethod,
    pub disclaimer: String,
}

/// Aviso do preço justo, no idioma da instalação.
pub fn disclaimer() -> String {
    t("nucleo-pesquisa-aviso-preco-justo")
}

/// Parâmetros do preço justo de Bazin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BazinParams {
    /// Taxa mínima (fração: 0,06 = 6%).
    pub rate: Decimal,
    pub years: u32,
}

impl Default for BazinParams {
    fn default() -> Self {
        Self {
            rate: Decimal::new(6, 2),
            years: 5,
        }
    }
}

// ---------------------------------------------------------------------------

/// Data no formato do idioma da instalação (14/10/2026, Oct 14, 2026).
pub(crate) fn br(d: NaiveDate) -> String {
    crate::i18n::date(crate::i18n::current(), d)
}

/// "12 meses até <data>".
pub(crate) fn ltm_until(d: NaiveDate) -> String {
    t_args(
        "nucleo-pesquisa-periodo-12m",
        &args(&[("data", br(d).into())]),
    )
}

/// "saldo em <data>".
pub(crate) fn balance_at(d: NaiveDate) -> String {
    t_args(
        "nucleo-pesquisa-periodo-saldo",
        &args(&[("data", br(d).into())]),
    )
}

/// "informe de <mm/aaaa>".
fn report_of(d: NaiveDate) -> String {
    t_args(
        "nucleo-pesquisa-ind-periodo-informe",
        &args(&[("mes", d.format("%m/%Y").to_string().into())]),
    )
}

fn ltm_period(f: &Fact) -> String {
    ltm_until(f.end)
}

/// Valores-base do emissor: fluxos LTM e saldos mais recentes.
struct Base<'a> {
    d: &'a Data,
    ltm: HashMap<String, Fact>,
    reference: Option<NaiveDate>,
    layout: Layout,
    per_unit: Option<Decimal>,
    /// Cotação por ação (cotação da unit ÷ ações por unit).
    price_share: Option<Decimal>,
    /// Fator da quantidade de ações (1; 1.000 quando o documento informou em milhares).
    shares_factor: Decimal,
    shares_note: Option<String>,
}

/// Quantidade de ações na mesma escala de uma referência (documentos que informam em
/// milhares sem dizer): fator de ~1.000 entre as duas vira 1.000 exato.
pub(crate) fn consistent_shares(raw: Decimal, reference: Decimal) -> Decimal {
    if raw <= Decimal::ZERO || reference <= Decimal::ZERO {
        return raw;
    }
    let r = reference / raw;
    let (lo, hi) = (Decimal::from(300), Decimal::from(3000));
    if r > lo && r < hi {
        raw * Decimal::from(1000)
    } else if r < Decimal::ONE / lo && r > Decimal::ONE / hi {
        raw / Decimal::from(1000)
    } else {
        raw
    }
}

/// Ações em circulação do último documento, já na escala certa (ver [`Base`]).
pub(crate) fn reference_shares(d: &Data) -> Option<Decimal> {
    let b = Base::new(d);
    b.shares().map(|f| f.value)
}

impl<'a> Base<'a> {
    fn new(d: &'a Data) -> Self {
        let stored: Vec<Fact> = d
            .facts
            .iter()
            .filter(|f| f.period_type == PeriodType::Ltm)
            .cloned()
            .collect();
        let ltm_facts = if stored.is_empty() {
            periods::compute_ltm(&d.facts)
        } else {
            stored
        };
        let reference = ltm_facts.iter().map(|f| f.end).max();
        let per_unit = d.per_unit;
        let price_share = match (&d.price, per_unit) {
            (Some((p, _)), Some(u)) if !u.is_zero() => Some(*p / u),
            _ => None,
        };
        // Escala das ações conferida pela cotação: P/VP abaixo de 0,02 (ou acima de 1.000)
        // com o fator 1.000 trazendo para a faixa normal é quantidade informada em milhares.
        let mut shares_factor = Decimal::ONE;
        let mut shares_note = None;
        let raw = periods::instant_at(&d.facts, items::SHARES, d.today);
        let eq = periods::instant_at(&d.facts, items::EQUITY, d.today);
        if let (Some(s), Some(e), Some(ps)) = (raw, eq, price_share)
            && e.value > Decimal::ZERO
            && s.value > Decimal::ZERO
            && !d.subject.is_fii()
        {
            let pvp = s.value * ps / e.value;
            let k = Decimal::from(1000);
            if pvp < Decimal::new(2, 2) && pvp * k <= Decimal::from(100) {
                shares_factor = k;
            } else if pvp > k && pvp / k >= Decimal::new(2, 2) {
                shares_factor = Decimal::ONE / k;
            }
            if shares_factor != Decimal::ONE {
                shares_note = Some(t_args(
                    "nucleo-pesquisa-ind-nota-escala-acoes",
                    &args(&[
                        ("documento", s.document.clone().into()),
                        (
                            "escala",
                            (if shares_factor > Decimal::ONE {
                                "milhares"
                            } else {
                                "unidades"
                            })
                            .into(),
                        ),
                        ("pvp", dstr(pvp.round_dp(4)).into()),
                        ("fator", dstr(shares_factor).into()),
                    ]),
                ));
            }
        }
        Self {
            d,
            ltm: ltm_facts.into_iter().map(|f| (f.item.clone(), f)).collect(),
            reference,
            layout: d.subject.layout.unwrap_or(Layout::Industrial),
            per_unit,
            price_share,
            shares_factor,
            shares_note,
        }
    }

    /// Fato de quantidade de ações mais recente, na escala corrigida.
    fn shares_item(&self, item: &str) -> Option<Fact> {
        self.inst(item).map(|f| Fact {
            value: f.value * self.shares_factor,
            ..f.clone()
        })
    }

    fn shares(&self) -> Option<Fact> {
        self.shares_item(items::SHARES)
    }

    fn flow(&self, item: &str) -> Option<&Fact> {
        self.ltm.get(item)
    }

    fn inst(&self, item: &str) -> Option<&'a Fact> {
        periods::instant_at(&self.d.facts, item, self.d.today)
    }

    fn month(&self, item: &str) -> Option<&'a Fact> {
        self.d
            .facts
            .iter()
            .filter(|f| f.item == item && f.period_type == PeriodType::Month)
            .max_by_key(|f| f.end)
    }

    fn price_src(&self) -> Option<String> {
        self.d.price.as_ref().map(|(_, p)| {
            t_args(
                "nucleo-pesquisa-fonte-cotacao",
                &args(&[
                    ("fonte", p.source.clone().into()),
                    ("data", p.date.clone().into()),
                ]),
            )
        })
    }

    /// Valor de mercado e a explicação do cálculo.
    fn market_cap(&self) -> Result<(Decimal, String), String> {
        let Some((price, _)) = &self.d.price else {
            return Err(t("nucleo-pesquisa-ind-sem-cotacao"));
        };
        if self.d.subject.is_fii() {
            let shares = self
                .month(items::FII_SHARES)
                .ok_or_else(|| t("nucleo-pesquisa-ind-sem-cotas"))?;
            return Ok((
                shares.value * *price,
                t_args(
                    "nucleo-pesquisa-ind-cotas-x-cotacao",
                    &args(&[("cotas", dstr(shares.value).into())]),
                ),
            ));
        }
        let shares = self
            .shares()
            .ok_or_else(|| t("nucleo-pesquisa-ind-sem-acoes"))?;
        let on = self
            .shares_item(items::SHARES_ON)
            .map(|f| f.value)
            .unwrap_or_default();
        let pn = self
            .shares_item(items::SHARES_PN)
            .map(|f| f.value)
            .unwrap_or_default();
        let price_of = |kind: &str| {
            self.d
                .siblings
                .iter()
                .find(|s| s.share_kind == kind)
                .map(|s| s.close)
        };
        if on + pn == shares.value && !shares.value.is_zero() {
            let p_on = (on > Decimal::ZERO).then(|| price_of("ON")).flatten();
            let p_pn = (pn > Decimal::ZERO).then(|| price_of("PN")).flatten();
            if (on.is_zero() || p_on.is_some()) && (pn.is_zero() || p_pn.is_some()) {
                let v = on * p_on.unwrap_or_default() + pn * p_pn.unwrap_or_default();
                if on > Decimal::ZERO && pn > Decimal::ZERO {
                    return Ok((v, t("nucleo-pesquisa-ind-on-pn-x-cotacao")));
                }
            }
        }
        let ps = self
            .price_share
            .ok_or_else(|| t("nucleo-pesquisa-ind-unit-desconhecida"))?;
        Ok((
            shares.value * ps,
            t_args(
                "nucleo-pesquisa-ind-acoes-x-cotacao",
                &args(&[("acoes", dstr(shares.value).into())]),
            ),
        ))
    }
}

fn ratio(a: Decimal, b: Decimal) -> Option<f64> {
    (!b.is_zero()).then(|| to_f64(a / b))
}

struct Out<'a> {
    list: Vec<Indicator>,
    currency: &'a str,
}

impl Out<'_> {
    fn push(&mut self, key: &str, r: Result<Calc, String>) {
        let d = def(key).expect("indicador do catálogo");
        let mut ind = Indicator {
            key: d.key.into(),
            label: d.label(),
            group: d.group,
            unit: d.unit,
            value: None,
            exact: None,
            currency: (d.unit == U::Money).then(|| self.currency.to_string()),
            formula: d.formula(),
            period: None,
            period_end: None,
            source: None,
            estimated: false,
            unavailable_reason: None,
        };
        match r {
            Ok(c) => {
                ind.value = c.value.filter(|v| v.is_finite());
                ind.exact = c.exact.map(dstr);
                ind.period = c.period;
                ind.period_end = c.period_end.map(|d| d.to_string());
                ind.source = c.source;
                ind.estimated = c.estimated;
                if let Some(f) = c.formula {
                    ind.formula = f;
                }
                if ind.value.is_none() {
                    ind.unavailable_reason = Some(t("nucleo-pesquisa-ind-divisao-zero"));
                }
            }
            Err(e) => ind.unavailable_reason = Some(e),
        }
        self.list.push(ind);
    }
}

#[derive(Default)]
struct Calc {
    value: Option<f64>,
    exact: Option<Decimal>,
    period: Option<String>,
    period_end: Option<NaiveDate>,
    source: Option<String>,
    estimated: bool,
    formula: Option<String>,
}

fn join(parts: &[Option<String>]) -> Option<String> {
    let v: Vec<String> = parts.iter().flatten().cloned().collect::<Vec<_>>();
    let mut uniq: Vec<String> = vec![];
    for p in v {
        if !uniq.contains(&p) {
            uniq.push(p);
        }
    }
    (!uniq.is_empty()).then(|| uniq.join(" · "))
}

fn financial_na() -> String {
    t("nucleo-pesquisa-ind-nao-se-aplica-financeiras")
}

/// Alíquota nominal de IR sobre o lucro (Brasil: IRPJ 25% + CSLL 9%; EUA: 21% federal).
fn tax_rate(currency: &str) -> Decimal {
    if currency == "USD" {
        Decimal::new(21, 2)
    } else {
        Decimal::new(34, 2)
    }
}

/// Proventos por ação/cota nos últimos 12 meses.
struct Div12 {
    per_share: Decimal,
    source: String,
    /// Estimado (não é a soma dos proventos pagos).
    estimated: bool,
    /// Fórmula, quando estimado.
    formula: Option<String>,
}

/// Rendimento do mês acima disso (fração do VP) não é rendimento: erro de unidade no
/// informe ou devolução de capital.
const FII_MAX_MONTHLY_YIELD: f64 = 0.05;

/// Rendimentos por cota declarados nos 4 últimos informes trimestrais (seguidos e recentes) ÷
/// cotas emitidas do último informe mensal.
fn fii_declared_12m(d: &Data) -> Option<(Decimal, NaiveDate)> {
    let mut q: Vec<&Fact> = d
        .facts
        .iter()
        .filter(|f| f.item == items::FII_DECLARED_INCOME && f.period_type == PeriodType::Quarter)
        .collect();
    q.sort_by_key(|f| std::cmp::Reverse(f.end));
    let last = q.first()?.end;
    if last <= crate::time::add_months(d.today, -9) || q.len() < 4 {
        return None;
    }
    let four = &q[..4];
    let consecutive = four
        .windows(2)
        .all(|w| crate::time::add_months(w[1].end, 3).with_day(1) == w[0].end.with_day(1));
    let shares = d
        .facts
        .iter()
        .filter(|f| f.item == items::FII_SHARES && f.period_type == PeriodType::Month)
        .max_by_key(|f| f.end)
        .map(|f| f.value)
        .filter(|v| *v > Decimal::ZERO)?;
    consecutive.then(|| (four.iter().map(|f| f.value).sum::<Decimal>() / shares, last))
}

/// Rendimento por cota de um informe mensal dentro do plausível (até
/// [`FII_MAX_MONTHLY_YIELD`] do VP por cota do mesmo informe).
fn fii_income_plausible(d: &Data, f: &Fact) -> bool {
    d.facts
        .iter()
        .find(|n| {
            n.item == items::NAV_PER_SHARE && n.period_type == PeriodType::Month && n.end == f.end
        })
        .is_none_or(|n| {
            n.value <= Decimal::ZERO || to_f64(f.value / n.value) <= FII_MAX_MONTHLY_YIELD
        })
}

/// Rendimentos por cota dos informes mensais dos últimos 12 meses, sem os fora do plausível.
fn fii_income_months(d: &Data) -> Vec<&Fact> {
    let mut v: Vec<&Fact> = d
        .facts
        .iter()
        .filter(|f| {
            f.item == items::INCOME_PER_SHARE
                && f.period_type == PeriodType::Month
                && f.end > crate::time::add_months(d.today, -13)
                && fii_income_plausible(d, f)
        })
        .collect();
    v.sort_by_key(|f| f.end);
    v
}

/// Proventos por ação/cota nos últimos 12 meses: da agenda do papel (ou a B3 dizendo que não
/// houve rendimento), ou dos fundamentos. FIIs sem proventos em dinheiro conhecidos: os
/// rendimentos declarados nos informes trimestrais e, sem eles, os do informe mensal (DY do
/// mês × VP por cota), com os meses na fórmula para conferir.
fn dividends_12m(b: &Base<'_>) -> Result<Div12, String> {
    let d = b.d;
    let since = d.today - chrono::Duration::days(365);
    if !d.dividends.is_empty() || (d.subject.is_fii() && d.dividends_source.is_some()) {
        let sum: Decimal = d
            .dividends
            .iter()
            .filter(|x| x.date > since && x.date <= d.today)
            .map(|x| x.amount)
            .sum();
        return Ok(Div12 {
            per_share: sum,
            source: d
                .dividends_source
                .clone()
                .unwrap_or_else(|| t("nucleo-pesquisa-ind-fonte-agenda")),
            estimated: false,
            formula: None,
        });
    }
    if d.subject.is_fii() {
        if let Some((sum, last)) = fii_declared_12m(d) {
            return Ok(Div12 {
                per_share: sum,
                source: t_args(
                    "nucleo-pesquisa-ind-fonte-declarados-fii",
                    &args(&[("data", br(last).into())]),
                ),
                estimated: true,
                formula: Some(t("nucleo-pesquisa-ind-formula-dy-fii-declarados")),
            });
        }
        let months = fii_income_months(d);
        let latest = months
            .iter()
            .map(|f| f.end)
            .max()
            .ok_or_else(|| t("nucleo-pesquisa-ind-sem-rendimentos-informe"))?;
        let from = crate::time::add_months(latest, -11);
        let used: Vec<&&Fact> = months.iter().filter(|f| f.end >= from).collect();
        let sum: Decimal = used.iter().map(|f| f.value).sum();
        let list = used
            .iter()
            .map(|f| {
                format!(
                    "{} {}",
                    f.end.format("%m/%Y"),
                    crate::ai::alerts::decimal(
                        crate::i18n::current(),
                        &f.value.round_dp(4).normalize().to_string(),
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Ok(Div12 {
            per_share: sum,
            source: t("nucleo-pesquisa-ind-fonte-informe-fii"),
            estimated: true,
            formula: Some(t_args(
                "nucleo-pesquisa-ind-formula-dy-fii-meses",
                &args(&[("meses", list.into())]),
            )),
        });
    }
    let paid = b
        .flow(items::DIVIDENDS_PAID)
        .ok_or_else(|| t("nucleo-pesquisa-ind-sem-proventos"))?;
    let shares = b
        .shares()
        .ok_or_else(|| t("nucleo-pesquisa-ind-sem-acoes"))?;
    if shares.value.is_zero() {
        return Err(t("nucleo-pesquisa-ind-sem-acoes"));
    }
    let unit = b
        .per_unit
        .ok_or_else(|| t("nucleo-pesquisa-ind-unit-desconhecida"))?;
    Ok(Div12 {
        per_share: paid.value / shares.value * unit,
        source: t_args(
            "nucleo-pesquisa-ind-fonte-dividendos-dfc",
            &args(&[("documento", paid.document.clone().into())]),
        ),
        estimated: true,
        formula: Some(t("nucleo-pesquisa-ind-formula-dy-dfc")),
    })
}

/// Dividendos por ação/cota por ano civil (ou exercício) dos últimos `years` anos completos.
/// Devolve (valores por ano, fonte, observação de cobertura).
pub(crate) fn dividends_by_year(
    d: &Data,
    years: u32,
    reference_shares: Option<Decimal>,
) -> (BTreeMap<i32, Decimal>, Option<String>, Option<String>) {
    let last = d.today.year() - 1;
    let first = last - years as i32 + 1;
    let window = |y: i32| y >= first && y <= last;
    // Agenda do papel.
    let mut ev: BTreeMap<i32, Decimal> = BTreeMap::new();
    let ev_first = d.dividends.iter().map(|x| x.date.year()).min();
    for x in &d.dividends {
        if window(x.date.year()) {
            *ev.entry(x.date.year()).or_default() += x.amount;
        }
    }
    let ev_full = ev_first.is_some_and(|y| y <= first);
    if ev_full {
        for y in first..=last {
            ev.entry(y).or_default();
        }
        return (ev, d.dividends_source.clone(), None);
    }
    // Fundamentos.
    let mut fu: BTreeMap<i32, Decimal> = BTreeMap::new();
    let fu_src;
    if d.subject.is_fii() {
        for f in d.facts.iter().filter(|f| {
            f.item == items::INCOME_PER_SHARE
                && f.period_type == PeriodType::Month
                && fii_income_plausible(d, f)
        }) {
            if window(f.end.year()) {
                *fu.entry(f.end.year()).or_default() += f.value;
            }
        }
        // Só anos com os 12 informes.
        let counts: BTreeMap<i32, usize> = d
            .facts
            .iter()
            .filter(|f| f.item == items::INCOME_PER_SHARE && f.period_type == PeriodType::Month)
            .fold(BTreeMap::new(), |mut m, f| {
                *m.entry(f.end.year()).or_default() += 1;
                m
            });
        fu.retain(|y, _| counts.get(y).copied().unwrap_or(0) >= 12);
        fu_src = Some(t("nucleo-pesquisa-ind-fonte-informe-fii"));
    } else {
        let unit = d.per_unit.unwrap_or(Decimal::ONE);
        let dps = periods::annual(&d.facts, items::DPS_DECLARED);
        if !dps.is_empty() {
            for (end, f) in &dps {
                if window(end.year()) {
                    fu.insert(end.year(), f.value * unit);
                }
            }
            fu_src = Some(t("nucleo-pesquisa-ind-fonte-dividendos-sec"));
        } else {
            for (end, f) in periods::annual(&d.facts, items::DIVIDENDS_PAID) {
                let Some(sh) =
                    periods::instant_at(&d.facts, items::SHARES, end + chrono::Duration::days(100))
                        .filter(|s| !s.value.is_zero())
                else {
                    continue;
                };
                if window(end.year()) {
                    let sh = reference_shares
                        .map(|r| consistent_shares(sh.value, r))
                        .unwrap_or(sh.value);
                    fu.insert(end.year(), f.value / sh * unit);
                }
            }
            fu_src = Some(t("nucleo-pesquisa-ind-fonte-dividendos-dfc-acoes"));
        }
    }
    let n = years as usize;
    if fu.len() >= n || (fu.len() >= ev.len().max(1) && !fu.is_empty()) {
        let note = (fu.len() < n).then(|| {
            t_args(
                "nucleo-pesquisa-ind-media-anos",
                &args(&[
                    ("anos", (fu.len() as i64).into()),
                    ("pedidos", i64::from(years).into()),
                ]),
            )
        });
        return (fu, fu_src, note);
    }
    if !ev.is_empty() {
        let note = Some(t_args(
            "nucleo-pesquisa-ind-media-anos-agenda",
            &args(&[
                ("ano", ev_first.unwrap_or(first).to_string().into()),
                ("anos", (ev.len() as i64).into()),
                ("pedidos", i64::from(years).into()),
            ]),
        ));
        return (ev, d.dividends_source.clone(), note);
    }
    (BTreeMap::new(), None, None)
}

/// Preços justos de Graham e Bazin, com as premissas.
pub fn fair_price(d: &Data, bazin: BazinParams) -> (FairPriceMethod, FairPriceMethod) {
    let b = Base::new(d);
    let cur = d.subject.currency.clone();
    let price = d.price.as_ref().map(|(p, _)| *p);
    let upside = |v: Decimal| price.filter(|p| !p.is_zero()).map(|p| to_f64(v / p) - 1.0);

    let mut graham = FairPriceMethod {
        method: "graham".into(),
        label: "Graham".into(),
        value: None,
        currency: cur.clone(),
        upside: None,
        formula: t("nucleo-pesquisa-ind-formula-graham"),
        assumptions: vec![],
        unavailable_reason: None,
    };
    if d.subject.is_fii() {
        graham.unavailable_reason = Some(t("nucleo-pesquisa-ind-nao-se-aplica-fii"));
    } else {
        match per_share(&b) {
            Ok((lpa, vpa, ni, eq, sh)) => {
                graham.assumptions = vec![
                    Assumption {
                        label: t("nucleo-pesquisa-ind-premissa-lpa"),
                        value: dstr(lpa.round_dp(4)),
                        source: Some(ni.document.clone()),
                    },
                    Assumption {
                        label: t("nucleo-pesquisa-ind-premissa-vpa"),
                        value: dstr(vpa.round_dp(4)),
                        source: Some(eq.document.clone()),
                    },
                    Assumption {
                        label: t("nucleo-pesquisa-ind-premissa-acoes"),
                        value: dstr(sh.value),
                        source: Some(sh.document.clone()),
                    },
                ];
                if lpa <= Decimal::ZERO {
                    graham.unavailable_reason = Some(t("nucleo-pesquisa-ind-lpa-negativo"));
                } else if vpa <= Decimal::ZERO {
                    graham.unavailable_reason = Some(t("nucleo-pesquisa-ind-vpa-negativo"));
                } else if let Some(v) = (Decimal::new(225, 1) * lpa * vpa).sqrt() {
                    let v = v.round_dp(2);
                    graham.value = Some(dstr(v));
                    graham.upside = upside(v);
                }
            }
            Err(e) => graham.unavailable_reason = Some(e),
        }
    }

    let mut bz = FairPriceMethod {
        method: "bazin".into(),
        label: "Bazin".into(),
        value: None,
        currency: cur,
        upside: None,
        formula: t("nucleo-pesquisa-ind-formula-bazin"),
        assumptions: vec![
            Assumption {
                label: t("nucleo-pesquisa-ind-premissa-taxa"),
                value: format!("{}%", dstr(bazin.rate * Decimal::from(100))),
                source: None,
            },
            Assumption {
                label: t("nucleo-pesquisa-ind-premissa-anos"),
                value: bazin.years.to_string(),
                source: None,
            },
        ],
        unavailable_reason: None,
    };
    if bazin.rate <= Decimal::ZERO {
        bz.unavailable_reason = Some(t("nucleo-pesquisa-ind-taxa-positiva"));
        return (graham, bz);
    }
    let (by_year, src, note) = dividends_by_year(d, bazin.years, b.shares().map(|f| f.value));
    if by_year.is_empty() {
        bz.unavailable_reason = Some(t("nucleo-pesquisa-ind-sem-historico-dividendos"));
        return (graham, bz);
    }
    let avg = by_year.values().copied().sum::<Decimal>() / Decimal::from(by_year.len());
    for (y, v) in &by_year {
        bz.assumptions.push(Assumption {
            label: t_args(
                "nucleo-pesquisa-ind-premissa-dividendos-ano",
                &args(&[("ano", y.to_string().into())]),
            ),
            value: dstr(v.round_dp(4)),
            source: src.clone(),
        });
    }
    bz.assumptions.push(Assumption {
        label: t("nucleo-pesquisa-ind-premissa-media"),
        value: dstr(avg.round_dp(4)),
        source: note,
    });
    if avg <= Decimal::ZERO {
        bz.unavailable_reason = Some(t("nucleo-pesquisa-ind-sem-dividendos-periodo"));
        return (graham, bz);
    }
    let v = (avg / bazin.rate).round_dp(2);
    bz.value = Some(dstr(v));
    bz.upside = upside(v);
    (graham, bz)
}

/// LPA, VPA e os fatos usados (lucro, patrimônio, ações).
type PerShare = (Decimal, Decimal, Fact, Fact, Fact);

/// LPA e VPA por papel (× ações por unit).
fn per_share(b: &Base<'_>) -> Result<PerShare, String> {
    let ni = b
        .flow(items::NET_INCOME)
        .ok_or_else(|| t("nucleo-pesquisa-ind-sem-lucro-12m"))?;
    let eq = b
        .inst(items::EQUITY)
        .ok_or_else(|| t("nucleo-pesquisa-ind-sem-patrimonio"))?;
    let sh = b
        .shares()
        .ok_or_else(|| t("nucleo-pesquisa-ind-sem-acoes"))?;
    if sh.value.is_zero() {
        return Err(t("nucleo-pesquisa-ind-sem-acoes"));
    }
    let unit = b
        .per_unit
        .ok_or_else(|| t("nucleo-pesquisa-ind-unit-desconhecida"))?;
    Ok((
        ni.value / sh.value * unit,
        eq.value / sh.value * unit,
        ni.clone(),
        eq.clone(),
        sh.clone(),
    ))
}

/// Todos os indicadores do ativo.
pub fn compute(d: &Data, bazin: BazinParams) -> IndicatorsView {
    let b = Base::new(d);
    let cur = d.subject.currency.as_str();
    let mut out = Out {
        list: vec![],
        currency: cur,
    };
    let mut notes = vec![];
    let fii = d.subject.is_fii();
    let financial = b.layout.is_financial();
    let psrc = b.price_src();
    let price = d.price.as_ref().map(|(p, _)| *p);

    out.push(
        "price",
        price
            .map(|p| Calc {
                value: Some(to_f64(p)),
                exact: Some(p),
                period_end: d
                    .price
                    .as_ref()
                    .and_then(|(_, x)| crate::time::parse_date(&x.date).ok()),
                source: psrc.clone(),
                ..Default::default()
            })
            .ok_or_else(|| t("nucleo-pesquisa-ind-sem-cotacao")),
    );
    let mc = b.market_cap();
    out.push(
        "market_cap",
        mc.clone().map(|(v, how)| Calc {
            value: Some(to_f64(v)),
            exact: Some(v.round_dp(2)),
            source: join(&[psrc.clone(), b.shares().map(|f| f.document.clone())]),
            formula: Some(t_args(
                "nucleo-pesquisa-ind-formula-valor-mercado",
                &args(&[("como", how.into())]),
            )),
            ..Default::default()
        }),
    );
    out.push(
        "liquidity",
        d.liquidity
            .clone()
            .map(|(v, src)| Calc {
                value: Some(to_f64(v)),
                exact: Some(v.round_dp(2)),
                source: Some(src),
                ..Default::default()
            })
            .ok_or_else(|| t("nucleo-pesquisa-ind-sem-volume")),
    );

    if fii {
        compute_fii(&b, &mut out, &psrc);
        push_dividend_history(&b, &mut out);
    } else {
        compute_stock(
            &b,
            &mut out,
            mc.as_ref().ok().map(|x| x.0),
            &psrc,
            financial,
            &mut notes,
        );
    }
    let (g, bz) = fair_price(d, bazin);
    for (m, key, ukey) in [
        (&g, "graham", "graham_upside"),
        (&bz, "bazin", "bazin_upside"),
    ] {
        if fii && key == "graham" {
            continue;
        }
        let src = join(
            &m.assumptions
                .iter()
                .map(|a| a.source.clone())
                .collect::<Vec<_>>(),
        );
        let v = m.value.as_deref().and_then(|v| v.parse::<Decimal>().ok());
        let r = || {
            m.unavailable_reason
                .clone()
                .unwrap_or_else(|| t("nucleo-pesquisa-ind-indisponivel"))
        };
        out.push(
            key,
            v.map(|v| Calc {
                value: Some(to_f64(v)),
                exact: Some(v),
                source: src.clone(),
                formula: Some(format!(
                    "{} ({})",
                    m.formula,
                    m.assumptions
                        .iter()
                        .map(|a| format!("{}: {}", a.label, a.value))
                        .collect::<Vec<_>>()
                        .join("; ")
                )),
                ..Default::default()
            })
            .ok_or_else(r),
        );
        out.push(
            ukey,
            match (v, m.upside) {
                (Some(_), Some(u)) => Ok(Calc {
                    value: Some(u),
                    source: join(&[src.clone(), psrc.clone()]),
                    ..Default::default()
                }),
                (Some(_), None) => Err(t("nucleo-pesquisa-ind-sem-cotacao")),
                _ => Err(r()),
            },
        );
    }
    if let Some(n) = &b.shares_note {
        notes.push(n.clone());
    }
    if financial {
        notes.push(t("nucleo-pesquisa-ind-nota-financeira"));
    }
    if b.per_unit.is_some_and(|u| u > Decimal::ONE) {
        notes.push(t_args(
            "nucleo-pesquisa-ind-nota-unit",
            &args(&[("acoes", dstr(b.per_unit.unwrap_or(Decimal::ONE)).into())]),
        ));
    }
    if d.facts.is_empty()
        && !matches!(
            d.subject.class,
            AssetClass::Fii
                | AssetClass::StockBr
                | AssetClass::StockUs
                | AssetClass::Reit
                | AssetClass::Bdr
        )
    {
        notes.push(t("nucleo-pesquisa-ind-nota-classe"));
    } else if d.facts.is_empty() {
        notes.push(t("nucleo-pesquisa-ind-nota-sem-demonstracoes"));
    }
    IndicatorsView {
        subject: d.subject.clone(),
        price: d.price.as_ref().map(|(_, p)| p.clone()),
        as_of: d.today.to_string(),
        reference_date: b.reference.map(|r| r.to_string()),
        indicators: out.list,
        notes,
    }
}

fn compute_stock(
    b: &Base<'_>,
    out: &mut Out<'_>,
    mc: Option<Decimal>,
    psrc: &Option<String>,
    financial: bool,
    notes: &mut Vec<String>,
) {
    let d = b.d;
    let need = |f: Option<&Fact>, key: &str| f.cloned().ok_or_else(|| t(key));
    let mc_r = || mc.ok_or_else(|| t("nucleo-pesquisa-ind-sem-valor-mercado"));
    let ni = need(
        b.flow(items::NET_INCOME),
        "nucleo-pesquisa-ind-sem-lucro-12m",
    );
    let rev = need(
        b.flow(items::REVENUE),
        "nucleo-pesquisa-ind-sem-receita-12m",
    );
    let ebit = need(b.flow(items::EBIT), "nucleo-pesquisa-ind-sem-ebit-12m");
    let da = need(b.flow(items::DA), "nucleo-pesquisa-ind-sem-da-12m");
    let gross = need(
        b.flow(items::GROSS_PROFIT),
        "nucleo-pesquisa-ind-sem-lucro-bruto-12m",
    );
    let eq = need(b.inst(items::EQUITY), "nucleo-pesquisa-ind-sem-patrimonio");
    let assets = need(
        b.inst(items::TOTAL_ASSETS),
        "nucleo-pesquisa-ind-sem-ativo-total",
    );
    let debt = need(
        b.inst(items::GROSS_DEBT),
        "nucleo-pesquisa-ind-sem-divida-bruta",
    );
    let cash = b.inst(items::CASH).cloned();
    let ca = need(
        b.inst(items::CURRENT_ASSETS),
        "nucleo-pesquisa-ind-sem-ativo-circulante",
    );
    let cl = need(
        b.inst(items::CURRENT_LIABILITIES),
        "nucleo-pesquisa-ind-sem-passivo-circulante",
    );
    let fin = |r: Result<Calc, String>| {
        if financial { Err(financial_na()) } else { r }
    };

    let ebitda = || -> Result<(Decimal, String, NaiveDate), String> {
        let e = ebit.clone()?;
        let a = da.clone()?;
        Ok((
            e.value + a.value,
            format!("{} (EBIT + D&A)", e.document),
            e.end,
        ))
    };
    let net_debt = || -> Result<(Decimal, String), String> {
        let dv = debt.clone()?;
        let c = cash.as_ref().map(|c| c.value).unwrap_or_default();
        Ok((dv.value - c, dv.document))
    };
    let ev = || -> Result<Decimal, String> { Ok(mc_r()? + net_debt()?.0) };
    let flow_calc = |v: Option<f64>, f: &Fact, extra: &[Option<String>]| Calc {
        value: v,
        period: Some(ltm_period(f)),
        period_end: Some(f.end),
        source: join(&[extra.to_vec(), vec![Some(f.document.clone())]].concat()),
        ..Default::default()
    };
    let inst_calc = |v: Option<f64>, f: &Fact, extra: &[Option<String>]| Calc {
        value: v,
        period: Some(balance_at(f.end)),
        period_end: Some(f.end),
        source: join(&[extra.to_vec(), vec![Some(f.document.clone())]].concat()),
        ..Default::default()
    };

    out.push(
        "ev",
        fin((|| {
            let v = ev()?;
            Ok(Calc {
                value: Some(to_f64(v)),
                exact: Some(v.round_dp(2)),
                source: join(&[psrc.clone(), Some(net_debt()?.1)]),
                ..Default::default()
            })
        })()),
    );
    out.push(
        "pl",
        (|| {
            let n = ni.clone()?;
            if n.value.is_zero() {
                return Err(t("nucleo-pesquisa-ind-lucro-zero"));
            }
            Ok(flow_calc(
                ratio(mc_r()?, n.value),
                &n,
                std::slice::from_ref(psrc),
            ))
        })(),
    );
    out.push(
        "pvp",
        (|| {
            let e = eq.clone()?;
            if e.value <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-patrimonio-negativo"));
            }
            Ok(inst_calc(
                ratio(mc_r()?, e.value),
                &e,
                std::slice::from_ref(psrc),
            ))
        })(),
    );
    out.push(
        "ev_ebitda",
        fin((|| {
            let (e, doc, end) = ebitda()?;
            if e <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-ebitda-negativo"));
            }
            Ok(Calc {
                value: ratio(ev()?, e),
                period: Some(ltm_until(end)),
                period_end: Some(end),
                source: join(&[psrc.clone(), Some(doc)]),
                ..Default::default()
            })
        })()),
    );
    out.push(
        "ev_ebit",
        fin((|| {
            let e = ebit.clone()?;
            if e.value <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-ebit-negativo"));
            }
            Ok(flow_calc(
                ratio(ev()?, e.value),
                &e,
                std::slice::from_ref(psrc),
            ))
        })()),
    );
    out.push(
        "psr",
        fin((|| {
            let r = rev.clone()?;
            if r.value <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-receita-zero"));
            }
            Ok(flow_calc(
                ratio(mc_r()?, r.value),
                &r,
                std::slice::from_ref(psrc),
            ))
        })()),
    );
    out.push(
        "p_ativo",
        (|| {
            let a = assets.clone()?;
            Ok(inst_calc(
                ratio(mc_r()?, a.value),
                &a,
                std::slice::from_ref(psrc),
            ))
        })(),
    );
    out.push(
        "dy",
        (|| {
            let div = dividends_12m(b)?;
            let p = d
                .price
                .as_ref()
                .ok_or_else(|| t("nucleo-pesquisa-ind-sem-cotacao"))?
                .0;
            Ok(Calc {
                value: ratio(div.per_share, p),
                exact: None,
                period: Some(ltm_until(d.today)),
                period_end: Some(d.today),
                source: join(&[Some(div.source), psrc.clone()]),
                estimated: div.estimated,
                formula: div.formula,
            })
        })(),
    );
    out.push(
        "payout",
        (|| {
            let n = ni.clone()?;
            if n.value <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-lucro-negativo"));
            }
            let (dv, formula) = match b.flow(items::DIVIDENDS_DECLARED) {
                Some(x) => (x.clone(), None),
                None => (
                    need(
                        b.flow(items::DIVIDENDS_PAID),
                        "nucleo-pesquisa-ind-sem-dividendos",
                    )?,
                    Some(t("nucleo-pesquisa-ind-formula-payout-pagos")),
                ),
            };
            let mut c = flow_calc(ratio(dv.value, n.value), &n, &[Some(dv.document.clone())]);
            c.formula = formula;
            Ok(c)
        })(),
    );
    let ret =
        |key: &str, num: Result<Fact, String>, den: Result<Fact, String>, positive_den: bool| {
            (|| {
                let n = num?;
                let dd = den?;
                if positive_den && dd.value <= Decimal::ZERO {
                    return Err(t("nucleo-pesquisa-ind-denominador-negativo"));
                }
                Ok(flow_calc(
                    ratio(n.value, dd.value),
                    &n,
                    &[Some(dd.document.clone())],
                ))
            })()
            .map(|c| (key.to_string(), c))
        };
    for (key, r) in [
        ("roe", ret("roe", ni.clone(), eq.clone(), true)),
        ("roa", ret("roa", ni.clone(), assets.clone(), true)),
    ] {
        out.push(key, r.map(|x| x.1));
    }
    out.push(
        "roic",
        fin((|| {
            let e = ebit.clone()?;
            let q = eq.clone()?;
            let (nd, _) = net_debt()?;
            let capital = q.value + nd;
            if capital <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-capital-negativo"));
            }
            let t = tax_rate(&d.subject.currency);
            let mut c = flow_calc(
                ratio(e.value * (Decimal::ONE - t), capital),
                &e,
                &[Some(q.document.clone())],
            );
            c.formula = Some(t_args(
                "nucleo-pesquisa-ind-formula-roic",
                &args(&[("aliquota", dstr(t * Decimal::from(100)).into())]),
            ));
            Ok(c)
        })()),
    );
    let margin = |num: Result<Fact, String>| {
        fin((|| {
            let n = num?;
            let r = rev.clone()?;
            if r.value <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-receita-zero"));
            }
            Ok(flow_calc(
                ratio(n.value, r.value),
                &n,
                &[Some(r.document.clone())],
            ))
        })())
    };
    out.push("gross_margin", margin(gross.clone()));
    out.push(
        "ebitda_margin",
        fin((|| {
            let (e, doc, end) = ebitda()?;
            let r = rev.clone()?;
            Ok(Calc {
                value: ratio(e, r.value),
                period: Some(ltm_until(end)),
                period_end: Some(end),
                source: Some(doc),
                ..Default::default()
            })
        })()),
    );
    out.push("ebit_margin", margin(ebit.clone()));
    out.push("net_margin", margin(ni.clone()));
    out.push(
        "net_debt_equity",
        fin((|| {
            let (nd, doc) = net_debt()?;
            let e = eq.clone()?;
            if e.value <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-patrimonio-negativo"));
            }
            Ok(inst_calc(ratio(nd, e.value), &e, &[Some(doc)]))
        })()),
    );
    out.push(
        "net_debt_ebitda",
        fin((|| {
            let (nd, doc) = net_debt()?;
            let (e, edoc, end) = ebitda()?;
            if e <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-ebitda-negativo"));
            }
            Ok(Calc {
                value: ratio(nd, e),
                period: Some(t_args(
                    "nucleo-pesquisa-ind-periodo-divida-ebitda",
                    &args(&[
                        (
                            "divida",
                            br(debt.as_ref().map(|f| f.end).unwrap_or(end)).into(),
                        ),
                        ("ebitda", br(end).into()),
                    ]),
                )),
                period_end: Some(end),
                source: join(&[Some(doc), Some(edoc)]),
                ..Default::default()
            })
        })()),
    );
    out.push(
        "current_ratio",
        fin((|| {
            let a = ca.clone()?;
            let l = cl.clone()?;
            Ok(inst_calc(ratio(a.value, l.value), &a, &[]))
        })()),
    );
    for (key, item) in [
        ("revenue_cagr_5y", items::REVENUE),
        ("earnings_cagr_5y", items::NET_INCOME),
    ] {
        out.push(
            key,
            (|| {
                let series = periods::annual(&d.facts, item);
                let (last_end, last) = series
                    .iter()
                    .next_back()
                    .ok_or_else(|| t("nucleo-pesquisa-ind-sem-exercicios"))?;
                let target = crate::time::add_months(*last_end, -60);
                let first = series
                    .iter()
                    .find(|(e, _)| periods::near(**e, target))
                    .map(|(_, f)| f)
                    .ok_or_else(|| t("nucleo-pesquisa-ind-poucos-exercicios"))?;
                let g = periods::cagr(first.value, last.value, 5)
                    .ok_or_else(|| t("nucleo-pesquisa-ind-ponta-negativa"))?;
                Ok(Calc {
                    value: Some(g),
                    period: Some(t_args(
                        "nucleo-pesquisa-ind-periodo-exercicios",
                        &args(&[
                            ("de", first.end.year().to_string().into()),
                            ("ate", last_end.year().to_string().into()),
                        ]),
                    )),
                    period_end: Some(*last_end),
                    source: join(&[Some(first.document.clone()), Some(last.document.clone())]),
                    ..Default::default()
                })
            })(),
        );
    }
    out.push(
        "loss_years_10y",
        (|| {
            let series = periods::annual(&d.facts, items::NET_INCOME);
            if series.is_empty() {
                return Err(t("nucleo-pesquisa-ind-sem-exercicios"));
            }
            let last10: Vec<&Fact> = series.values().rev().take(10).collect();
            let n = last10.iter().filter(|f| f.value < Decimal::ZERO).count();
            Ok(Calc {
                value: Some(n as f64),
                period: Some(t_args(
                    "nucleo-pesquisa-ind-periodo-n-exercicios",
                    &args(&[
                        ("n", (last10.len() as i64).into()),
                        (
                            "de",
                            last10
                                .last()
                                .map(|f| f.end.year())
                                .unwrap_or_default()
                                .to_string()
                                .into(),
                        ),
                        (
                            "ate",
                            last10
                                .first()
                                .map(|f| f.end.year())
                                .unwrap_or_default()
                                .to_string()
                                .into(),
                        ),
                    ]),
                )),
                period_end: last10.first().map(|f| f.end),
                source: Some(t("nucleo-pesquisa-ind-fonte-dfp-lucro")),
                ..Default::default()
            })
        })(),
    );
    // Métodos do explorador: Magic Formula (EBIT/EV e ROC), fluxo de caixa livre, PEG e
    // o crescimento do lucro em 10 anos (Graham defensivo).
    out.push(
        "earnings_yield",
        fin((|| {
            let e = ebit.clone()?;
            let v = ev()?;
            Ok(flow_calc(ratio(e.value, v), &e, std::slice::from_ref(psrc)))
        })()),
    );
    out.push(
        "magic_roc",
        fin((|| {
            let e = ebit.clone()?;
            let a = ca.clone()?;
            let l = cl.clone()?;
            let ppe = need(b.inst(items::PPE), "nucleo-pesquisa-ind-sem-imobilizado")?;
            let capital = (a.value - l.value).max(Decimal::ZERO) + ppe.value;
            if capital <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-divisao-zero"));
            }
            Ok(flow_calc(
                ratio(e.value, capital),
                &e,
                &[Some(ppe.document.clone())],
            ))
        })()),
    );
    out.push(
        "fcf_yield",
        fin((|| {
            let cfo = need(b.flow(items::CFO), "nucleo-pesquisa-ind-sem-fco-12m")?;
            let capex = b.flow(items::CAPEX).map(|f| f.value).unwrap_or_default();
            let m = mc_r()?;
            Ok(flow_calc(
                ratio(cfo.value - capex, m),
                &cfo,
                std::slice::from_ref(psrc),
            ))
        })()),
    );
    let value_of =
        |out: &Out<'_>, key: &str| out.list.iter().find(|i| i.key == key).and_then(|i| i.value);
    let pl = value_of(out, "pl");
    let growth = value_of(out, "earnings_cagr_5y");
    out.push(
        "peg",
        match (pl, growth) {
            (Some(p), Some(g)) if p > 0.0 && g > 0.0 => Ok(Calc {
                value: Some(p / (g * 100.0)),
                source: Some(t("nucleo-pesquisa-ind-fonte-peg")),
                ..Default::default()
            }),
            (Some(_), Some(_)) => Err(t("nucleo-pesquisa-ind-peg-negativo")),
            _ => Err(t("nucleo-pesquisa-ind-peg-sem-dados")),
        },
    );
    out.push(
        "eps_growth_10y",
        (|| {
            let annual = periods::annual(&d.facts, items::NET_INCOME);
            let series: Vec<&Fact> = annual.values().rev().take(10).collect();
            if series.len() < 10 {
                return Err(t("nucleo-pesquisa-ind-poucos-exercicios"));
            }
            let avg = |f: &[&Fact]| {
                f.iter().map(|x| x.value).sum::<Decimal>() / Decimal::from(f.len() as i64)
            };
            let recent = avg(&series[..3]);
            let old = avg(&series[7..10]);
            if old <= Decimal::ZERO {
                return Err(t("nucleo-pesquisa-ind-ponta-negativa"));
            }
            Ok(Calc {
                value: Some(to_f64(recent / old - Decimal::ONE)),
                period: Some(t_args(
                    "nucleo-pesquisa-ind-periodo-exercicios",
                    &args(&[
                        ("de", series[9].end.year().to_string().into()),
                        ("ate", series[0].end.year().to_string().into()),
                    ]),
                )),
                period_end: Some(series[0].end),
                source: Some(t("nucleo-pesquisa-ind-fonte-dfp-lucro")),
                ..Default::default()
            })
        })(),
    );
    push_dividend_history(b, out);
    match per_share(b) {
        Ok((lpa, vpa, n, e, sh)) => {
            out.push(
                "lpa",
                Ok(Calc {
                    value: Some(to_f64(lpa)),
                    exact: Some(lpa.round_dp(4)),
                    period: Some(ltm_period(&n)),
                    period_end: Some(n.end),
                    source: join(&[Some(n.document.clone()), Some(sh.document.clone())]),
                    ..Default::default()
                }),
            );
            out.push(
                "vpa",
                Ok(Calc {
                    value: Some(to_f64(vpa)),
                    exact: Some(vpa.round_dp(4)),
                    period: Some(balance_at(e.end)),
                    period_end: Some(e.end),
                    source: join(&[Some(e.document.clone()), Some(sh.document.clone())]),
                    ..Default::default()
                }),
            );
        }
        Err(e) => {
            out.push("lpa", Err(e.clone()));
            out.push("vpa", Err(e));
        }
    }
    if b.layout == Layout::Bank {
        out.push("basel", Err(t("nucleo-pesquisa-ind-basileia-fora-cvm")));
    }
    if d.subject.currency == "USD" && b.inst(items::SHARES).is_some() {
        notes.push(t("nucleo-pesquisa-ind-nota-acoes-sec"));
    }
}

/// Anos seguidos com provento (até 20, terminando no último ano completo) e o crescimento
/// dos proventos por ação/cota em 5 anos (Graham defensivo, Bazin, crescimento de
/// dividendos).
fn push_dividend_history(b: &Base<'_>, out: &mut Out<'_>) {
    let d = b.d;
    let (by_year, src, _) = dividends_by_year(d, 20, b.shares().map(|f| f.value));
    let last = d.today.year() - 1;
    let streak = (0..20)
        .map(|k| last - k)
        .take_while(|y| by_year.get(y).is_some_and(|v| *v > Decimal::ZERO))
        .count();
    out.push(
        "dividend_years",
        if by_year.is_empty() {
            Err(t("nucleo-pesquisa-ind-sem-proventos"))
        } else {
            Ok(Calc {
                value: Some(streak as f64),
                source: src.clone(),
                ..Default::default()
            })
        },
    );
    let first = by_year.get(&(last - 5)).copied();
    let end = by_year.get(&last).copied();
    out.push(
        "dividend_cagr_5y",
        match (first, end) {
            (Some(f), Some(l)) => periods::cagr(f, l, 5)
                .map(|g| Calc {
                    value: Some(g),
                    period: Some(t_args(
                        "nucleo-pesquisa-ind-periodo-exercicios",
                        &args(&[
                            ("de", (last - 5).to_string().into()),
                            ("ate", last.to_string().into()),
                        ]),
                    )),
                    source: src,
                    ..Default::default()
                })
                .ok_or_else(|| t("nucleo-pesquisa-ind-ponta-negativa")),
            _ => Err(t("nucleo-pesquisa-ind-poucos-exercicios")),
        },
    );
}

fn compute_fii(b: &Base<'_>, out: &mut Out<'_>, psrc: &Option<String>) {
    let d = b.d;
    let nav = b.month(items::NAV_PER_SHARE);
    out.push(
        "nav_per_share",
        nav.map(|f| Calc {
            value: Some(to_f64(f.value)),
            exact: Some(f.value.round_dp(4)),
            period: Some(report_of(f.end)),
            period_end: Some(f.end),
            source: Some(f.document.clone()),
            ..Default::default()
        })
        .ok_or_else(|| t("nucleo-pesquisa-ind-sem-informe")),
    );
    out.push(
        "pvp",
        (|| {
            let n = nav.ok_or_else(|| t("nucleo-pesquisa-ind-sem-vp-cota"))?;
            let p = d
                .price
                .as_ref()
                .ok_or_else(|| t("nucleo-pesquisa-ind-sem-cotacao"))?
                .0;
            Ok(Calc {
                value: ratio(p, n.value),
                period: Some(t_args(
                    "nucleo-pesquisa-ind-periodo-vp-informe",
                    &args(&[("mes", n.end.format("%m/%Y").to_string().into())]),
                )),
                period_end: Some(n.end),
                source: join(&[psrc.clone(), Some(n.document.clone())]),
                formula: Some(t("nucleo-pesquisa-ind-formula-pvp-fii")),
                ..Default::default()
            })
        })(),
    );
    out.push(
        "dy",
        (|| {
            let div = dividends_12m(b)?;
            let p = d
                .price
                .as_ref()
                .ok_or_else(|| t("nucleo-pesquisa-ind-sem-cotacao"))?
                .0;
            Ok(Calc {
                value: ratio(div.per_share, p),
                period: Some(ltm_until(d.today)),
                period_end: Some(d.today),
                source: join(&[Some(div.source), psrc.clone()]),
                estimated: div.estimated,
                formula: div.formula,
                exact: None,
            })
        })(),
    );
    out.push(
        "last_dividend",
        (|| {
            if let Some(last) = d.dividends.iter().max_by_key(|x| x.date) {
                return Ok(Calc {
                    value: Some(to_f64(last.amount)),
                    exact: Some(last.amount),
                    period: Some(t_args(
                        "nucleo-pesquisa-ind-periodo-data-com",
                        &args(&[("data", br(last.date).into())]),
                    )),
                    period_end: Some(last.date),
                    source: d.dividends_source.clone(),
                    ..Default::default()
                });
            }
            if d.dividends_source.is_some() {
                return Err(t("nucleo-pesquisa-ind-sem-rendimentos-b3"));
            }
            let f = b
                .month(items::INCOME_PER_SHARE)
                .ok_or_else(|| t("nucleo-pesquisa-ind-sem-rendimentos"))?;
            Ok(Calc {
                value: Some(to_f64(f.value)),
                exact: Some(f.value.round_dp(4)),
                period: Some(report_of(f.end)),
                period_end: Some(f.end),
                source: Some(f.document.clone()),
                estimated: true,
                ..Default::default()
            })
        })(),
    );
    out.push(
        "holders",
        b.month(items::HOLDERS)
            .map(|f| Calc {
                value: Some(to_f64(f.value)),
                period: Some(report_of(f.end)),
                period_end: Some(f.end),
                source: Some(f.document.clone()),
                ..Default::default()
            })
            .ok_or_else(|| t("nucleo-pesquisa-ind-sem-informe")),
    );
    // Vacância física (área vaga ÷ área) do informe trimestral mais recente.
    out.push(
        "vacancy",
        d.facts
            .iter()
            .filter(|f| {
                f.item == items::FII_VACANCY_PHYSICAL
                    && f.period_type == PeriodType::Instant
                    && f.end > crate::time::add_months(d.today, -12)
            })
            .max_by_key(|f| f.end)
            .map(|f| Calc {
                value: Some(to_f64(f.value)),
                period: Some(t_args(
                    "nucleo-pesquisa-ind-periodo-trimestral",
                    &args(&[("data", br(f.end).into())]),
                )),
                period_end: Some(f.end),
                source: Some(f.document.clone()),
                ..Default::default()
            })
            .ok_or_else(|| t("nucleo-pesquisa-ind-vacancia-fora")),
    );
}

/// Valor de um indicador numa lista.
pub fn value_of(list: &[Indicator], key: &str) -> Option<f64> {
    list.iter().find(|i| i.key == key).and_then(|i| i.value)
}

#[cfg(test)]
mod tests;
