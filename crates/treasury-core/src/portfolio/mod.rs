//! Carteira: operações, eventos corporativos, proventos, posições (preço médio), renda fixa
//! na curva, resultado realizado e não realizado, XIRR nominal e real, TWR mensal,
//! alocação-alvo, watchlist, agenda de dividendos e renda passiva.

pub mod api;
pub mod benchmark;
pub mod bridge;
pub mod bulk;
pub mod come_cotas;
pub mod compare;
pub mod concentration;
pub mod contrib;
pub mod cost_method;
pub mod engine;
pub mod fee_cost;
pub mod pending;
pub mod plans;
pub mod quick_income;
pub mod records;
pub mod risk;
pub mod sales;
pub mod statement;
pub mod trade_summary;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::assets::AssetClass;
use crate::ids::Id;
use crate::indices::Deflator;
use crate::market::PriceOn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Trade {
    pub id: Id,
    pub asset_id: Id,
    pub ticker: Option<String>,
    pub asset_name: String,
    /// Renda fixa (CDB, RDB, LCI…): a pessoa só vê valores; a quantidade é interna (valor × PU 1
    /// na compra) e a interface não a mostra.
    pub by_value: bool,
    pub account_id: Id,
    pub account_name: String,
    pub date: String,
    pub side: TradeSide,
    /// Decimal em texto.
    pub quantity: String,
    /// Preço unitário na moeda do ativo, decimal em texto.
    pub price: String,
    #[ts(type = "number")]
    pub fees: i64,
    #[ts(type = "number")]
    pub taxes: i64,
    pub currency: String,
    pub fx_rate: Option<String>,
    pub day_trade: bool,
    /// quantidade × preço, na menor unidade.
    #[ts(type = "number")]
    pub gross: i64,
    /// Efeito no caixa da corretora: compra = −(bruto + custos), venda = bruto − custos.
    #[ts(type = "number")]
    pub cash_amount: i64,
    pub txn_id: Option<Id>,
    pub note_number: Option<String>,
    pub notes: Option<String>,
    pub import_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TradeInput {
    pub asset_id: Id,
    /// Conta da corretora (saldo em caixa).
    pub account_id: Id,
    pub date: String,
    pub side: TradeSide,
    pub quantity: String,
    pub price: String,
    #[ts(optional, type = "number")]
    pub fees: Option<i64>,
    #[ts(optional, type = "number")]
    pub taxes: Option<i64>,
    /// Câmbio do dia para ativos em outra moeda (padrão: PTAX do banco de dados).
    #[ts(optional)]
    pub fx_rate: Option<String>,
    #[ts(optional)]
    pub day_trade: Option<bool>,
    #[ts(optional)]
    pub note_number: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
    /// Lançar a movimentação no caixa da corretora (padrão: sim).
    #[ts(optional)]
    pub cash_movement: Option<bool>,
    #[ts(optional)]
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TradePatch {
    pub id: Id,
    #[ts(optional)]
    pub date: Option<String>,
    #[ts(optional)]
    pub side: Option<TradeSide>,
    #[ts(optional)]
    pub quantity: Option<String>,
    #[ts(optional)]
    pub price: Option<String>,
    #[ts(optional, type = "number")]
    pub fees: Option<i64>,
    #[ts(optional, type = "number")]
    pub taxes: Option<i64>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub fx_rate: Option<Option<String>>,
    #[ts(optional)]
    pub day_trade: Option<bool>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub note_number: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
    /// Lançar no caixa da conta: `true` cria o lançamento ligado (se não houver), `false` o
    /// remove. Ausente mantém como está.
    #[ts(optional)]
    pub cash_movement: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TradeFilter {
    #[ts(optional)]
    pub asset_id: Option<Id>,
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[ts(optional)]
    pub date_from: Option<String>,
    #[ts(optional)]
    pub date_to: Option<String>,
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
    #[ts(optional, type = "number")]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CorporateActionKind {
    /// Desdobramento: `factor` > 1 (ex.: 2 = cada ação vira duas).
    Split,
    /// Grupamento: `factor` < 1 (ex.: 0,1 = dez viram uma).
    ReverseSplit,
    /// Bonificação: `factor` = ações novas por ação (ex.: 0,1 = 10%), com `unit_cost` atribuído.
    Bonus,
    Subscription,
    Merger,
    TickerChange,
    SpinOff,
    /// Devolução de capital: `amount` total reduz o custo.
    CapitalReturn,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CorporateAction {
    pub id: Id,
    pub asset_id: Id,
    pub date: String,
    pub kind: CorporateActionKind,
    pub factor: Option<String>,
    pub unit_cost: Option<String>,
    #[ts(type = "number | null")]
    pub amount: Option<i64>,
    pub new_asset_id: Option<Id>,
    pub notes: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CorporateActionInput {
    pub asset_id: Id,
    pub date: String,
    pub kind: CorporateActionKind,
    #[ts(optional)]
    pub factor: Option<String>,
    #[ts(optional)]
    pub unit_cost: Option<String>,
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[ts(optional)]
    pub new_asset_id: Option<Id>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum IncomeKind {
    Dividend,
    Jcp,
    FiiIncome,
    Coupon,
    Interest,
    Rent,
    Amortization,
    Other,
}

impl IncomeKind {
    /// Nome do tipo no idioma da instalação.
    pub fn label(self) -> String {
        crate::i18n::t(match self {
            IncomeKind::Dividend => "nucleo-carteira-provento-dividendos",
            IncomeKind::Jcp => "nucleo-carteira-provento-jcp",
            IncomeKind::FiiIncome => "nucleo-carteira-provento-fii",
            IncomeKind::Coupon => "nucleo-carteira-provento-cupom",
            IncomeKind::Interest => "nucleo-carteira-provento-juros",
            IncomeKind::Rent => "nucleo-carteira-provento-aluguel",
            IncomeKind::Amortization => "nucleo-carteira-provento-amortizacao",
            IncomeKind::Other => "nucleo-carteira-provento-outros",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum IncomeStatus {
    Announced,
    Received,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeEvent {
    pub id: Id,
    pub asset_id: Id,
    pub ticker: Option<String>,
    pub asset_name: String,
    pub account_id: Id,
    pub kind: IncomeKind,
    pub ex_date: Option<String>,
    pub pay_date: String,
    pub quantity: Option<String>,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub tax_withheld: i64,
    #[ts(type = "number")]
    pub net: i64,
    pub currency: String,
    pub fx_rate: Option<String>,
    pub status: IncomeStatus,
    pub txn_id: Option<Id>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeInput {
    pub asset_id: Id,
    pub account_id: Id,
    pub kind: IncomeKind,
    #[ts(optional)]
    pub ex_date: Option<String>,
    pub pay_date: String,
    #[ts(optional)]
    pub quantity: Option<String>,
    #[ts(type = "number")]
    pub gross: i64,
    /// Padrão: 15% para JCP, 30% para dividendos de ações americanas, 0 nos demais.
    #[ts(optional, type = "number")]
    pub tax_withheld: Option<i64>,
    #[ts(optional)]
    pub fx_rate: Option<String>,
    #[ts(optional)]
    pub status: Option<IncomeStatus>,
    /// Lançar o crédito no caixa da corretora (padrão: sim, quando recebido).
    #[ts(optional)]
    pub cash_movement: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Valuation {
    /// Cotação de mercado.
    Market,
    /// Marcação na curva (renda fixa).
    Curve,
    /// Sem cotação disponível: valor = custo.
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetBrief {
    pub id: Id,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub class_label: String,
    pub currency: String,
}

/// Posição num ativo (numa conta ou somada entre contas). Valores `*_base` na moeda base da
/// família.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PositionView {
    pub asset: AssetBrief,
    /// `null` = soma de todas as contas.
    pub account_id: Option<Id>,
    pub quantity: String,
    pub avg_price: Option<String>,
    /// Custo total na moeda do ativo.
    #[ts(type = "number")]
    pub cost: i64,
    #[ts(type = "number")]
    pub cost_base: i64,
    pub price: Option<PriceOn>,
    #[ts(type = "number | null")]
    pub market_value: Option<i64>,
    #[ts(type = "number | null")]
    pub market_value_base: Option<i64>,
    #[ts(type = "number | null")]
    pub unrealized_base: Option<i64>,
    #[ts(type = "number")]
    pub realized_base: i64,
    /// Proventos líquidos recebidos.
    #[ts(type = "number")]
    pub income_base: i64,
    /// não realizado + realizado + proventos.
    #[ts(type = "number | null")]
    pub total_result_base: Option<i64>,
    /// Resultado real, em dinheiro da data da posição: cada compra, venda e provento corrigido
    /// pelo deflator até ela (docs/real-return.md). Igual ao nominal com `none`; sem dado do
    /// índice, `null`.
    #[ts(type = "number | null")]
    pub real_result_base: Option<i64>,
    /// Nominal − real: o que a inflação (ou o M2) levou do resultado; em geral positivo (o mesmo
    /// sinal do `inflation_effect` das vendas).
    #[ts(type = "number | null")]
    pub inflation_effect_base: Option<i64>,
    /// Retorno anual efetivo (XIRR) na moeda base.
    pub xirr: Option<f64>,
    /// XIRR descontado pelo deflator escolhido.
    pub xirr_real: Option<f64>,
    /// Dias desde a abertura da posição atual (a primeira compra, ou a recompra depois de
    /// zerar).
    #[ts(type = "number | null")]
    pub holding_days: Option<i64>,
    /// Retorno simples do período (resultado total ÷ custo), fração.
    pub period_return: Option<f64>,
    /// O mesmo, real (resultado real ÷ custo).
    pub period_return_real: Option<f64>,
    /// Qual número vale: `period` (menos de [`MIN_DAYS_FOR_ANNUALIZED`] dias: `xirr` nulo),
    /// `period+annualized` (até um ano: o do período em destaque, o anualizado ao lado) ou
    /// `annualized`.
    pub return_basis: ReturnBasis,
    /// Participação no valor total da carteira (0–1).
    pub weight: Option<f64>,
    pub valuation: Valuation,
    /// Renda fixa: valor líquido estimado de IR e IOF num resgate hoje.
    #[ts(type = "number | null")]
    pub net_redemption_base: Option<i64>,
    pub first_date: Option<String>,
    /// Algum valor veio de câmbio/índice estimado ou cotação defasada.
    pub estimated: bool,
    pub warnings: Vec<String>,
}

/// Dias de carteira a partir dos quais o retorno anualizado (XIRR) aparece. Antes disso a taxa
/// explode (5 dias de −6% viram −99% a.a.) e só o retorno do período vale.
pub const MIN_DAYS_FOR_ANNUALIZED: i64 = 30;
/// A partir de um ano, o anualizado é o número principal.
pub const FULL_YEAR_DAYS: i64 = 365;

/// Qual retorno de uma posição vale, pela idade dela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ReturnBasis {
    #[serde(rename = "period")]
    Period,
    #[serde(rename = "period+annualized")]
    PeriodAndAnnualized,
    #[serde(rename = "annualized")]
    Annualized,
}

impl ReturnBasis {
    pub fn for_days(days: Option<i64>) -> ReturnBasis {
        match days {
            Some(d) if d < MIN_DAYS_FOR_ANNUALIZED => ReturnBasis::Period,
            Some(d) if d < FULL_YEAR_DAYS => ReturnBasis::PeriodAndAnnualized,
            Some(_) => ReturnBasis::Annualized,
            None => ReturnBasis::Period,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ClassSummary {
    /// XIRR médio da classe, ponderado pelo valor, só com as posições com taxa anualizada
    /// (as recentes ficam de fora).
    pub xirr: Option<f64>,
    pub class: AssetClass,
    pub label: String,
    #[ts(type = "number")]
    pub market_value_base: i64,
    #[ts(type = "number")]
    pub cost_base: i64,
    pub weight: f64,
    pub target_pct: Option<f64>,
    /// peso − alvo, em pontos percentuais.
    pub deviation_pp: Option<f64>,
    pub out_of_band: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioArgs {
    #[ts(optional)]
    pub as_of: Option<String>,
    #[ts(optional)]
    pub account_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub deflator: Option<Deflator>,
    /// Uma linha por conta em vez de somar o ativo entre contas.
    #[ts(optional)]
    pub by_account: Option<bool>,
    #[ts(optional)]
    pub include_closed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PortfolioSummary {
    pub as_of: String,
    pub base_currency: String,
    pub deflator: Deflator,
    #[ts(type = "number")]
    pub market_value: i64,
    #[ts(type = "number")]
    pub cost: i64,
    #[ts(type = "number")]
    pub unrealized: i64,
    #[ts(type = "number")]
    pub realized: i64,
    #[ts(type = "number")]
    pub income: i64,
    #[ts(type = "number")]
    pub total_result: i64,
    /// Resultado real pelo deflator (soma das posições); sem dado do índice, `null`.
    #[ts(type = "number | null")]
    pub real_result: Option<i64>,
    /// Nominal − real (soma das posições).
    #[ts(type = "number | null")]
    pub inflation_effect: Option<i64>,
    pub xirr: Option<f64>,
    pub xirr_real: Option<f64>,
    pub by_class: Vec<ClassSummary>,
    pub positions: Vec<PositionView>,
    /// Ativos sem cotação, avaliados pelo custo (a previdência fica de fora: ver
    /// `balance_due`).
    pub missing_prices: Vec<String>,
    /// Previdência sem saldo informado (valendo o custo) ou com o último saldo ou cota de
    /// mais de [`BALANCE_DUE_DAYS`] dias: um lembrete para informar o saldo.
    #[serde(default)]
    pub balance_due: Vec<BalanceDue>,
    pub estimated: bool,
    /// Ativos, grupos (etiquetas marcadas) e classes acima do limite de concentração.
    #[serde(default)]
    pub concentration_alerts: Vec<concentration::ConcentrationAlert>,
}

/// Dias sem saldo ou cota novos para a previdência pedir o saldo atual.
pub const BALANCE_DUE_DAYS: i64 = 90;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BalanceDue {
    pub asset_id: Id,
    pub name: String,
    /// Data do último saldo ou cota; `null` sem nenhum (avaliado pelo custo).
    pub as_of: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerformanceArgs {
    #[ts(optional)]
    pub from_month: Option<String>,
    #[ts(optional)]
    pub to_month: Option<String>,
    #[ts(optional)]
    pub deflator: Option<Deflator>,
    #[ts(optional)]
    pub asset_id: Option<Id>,
    #[ts(optional)]
    pub class: Option<AssetClass>,
    /// Como encadear o retorno: mensal por Modified Dietz (padrão) ou diário. Sem ele, vale o
    /// ajuste avançado `twr_daily`.
    #[serde(default)]
    #[ts(optional)]
    pub method: Option<TwrMethod>,
}

/// Retorno ponderado pelo tempo: mensal (Modified Dietz, o padrão) ou diário (cada dia,
/// (valor final + saídas) ÷ (valor inicial + entradas) − 1, encadeado; entradas no início do
/// dia, saídas e proventos no fim). O diário pesa mais (um valor de carteira por dia útil).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TwrMethod {
    #[default]
    Monthly,
    Daily,
}

/// Risco da série de retornos da rentabilidade (mensal ou diária, conforme o método).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReturnRisk {
    /// Períodos com retorno (meses ou dias úteis).
    pub periods: u32,
    /// Semidesvio anualizado: o desvio só dos retornos abaixo da média (o tamanho típico das
    /// quedas), fração.
    pub semideviation: f64,
    /// Volatilidade anualizada, fração.
    pub volatility: f64,
    /// Pior queda do acumulado (fração negativa) e as datas do topo, do fundo e da volta ao
    /// topo (`null` = ainda não voltou).
    pub max_drawdown: f64,
    pub drawdown_peak: Option<String>,
    pub drawdown_trough: Option<String>,
    pub drawdown_recovered: Option<String>,
    /// Maior tempo abaixo de um topo, em dias corridos (do topo até voltar a ele, ou até a
    /// última data se ainda não voltou).
    #[ts(type = "number")]
    pub longest_drawdown_days: i64,
    pub longest_drawdown_ongoing: bool,
}

/// Rentabilidade mensal (TWR por Modified Dietz) e acumulada, com benchmarks.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Performance {
    pub months: Vec<String>,
    /// Retorno do mês (razão). `null` em meses sem posição.
    pub monthly: Vec<Option<f64>>,
    /// Acumulado desde o início do período.
    pub cumulative: Vec<f64>,
    /// Acumulado real (descontado pelo deflator).
    pub cumulative_real: Vec<Option<f64>>,
    /// Juro de referência do país acumulado (no Brasil, o CDI; docs/countries.md).
    pub cdi_cumulative: Vec<Option<f64>>,
    /// Nome do juro de referência (CDI, Fed Funds, €STR…).
    pub benchmark_label: String,
    /// Ibovespa (pontos) acumulado no período.
    pub ibov_cumulative: Vec<Option<f64>>,
    /// S&P 500 convertido para reais pela PTAX, acumulado no período.
    pub sp500_brl_cumulative: Vec<Option<f64>>,
    /// Variação do dólar (PTAX) no período.
    pub usd_cumulative: Vec<Option<f64>>,
    pub inflation_cumulative: Vec<Option<f64>>,
    pub deflator: Deflator,
    #[ts(type = "number[]")]
    pub value_base: Vec<i64>,
    #[ts(type = "number[]")]
    pub net_contributions: Vec<i64>,
    pub estimated: bool,
    pub method: TwrMethod,
    /// Semidesvio, volatilidade e pior queda da série (`null` com menos de 2 períodos).
    pub risk: Option<ReturnRisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TargetInput {
    pub scope: TargetScope,
    /// Classe (`stock_br`…) ou id do ativo.
    pub key: String,
    /// % da carteira, decimal em texto.
    pub target_pct: String,
    #[ts(optional)]
    pub tolerance_pct: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TargetScope {
    Class,
    Asset,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Target {
    pub id: Id,
    pub scope: TargetScope,
    pub key: String,
    pub label: String,
    pub target_pct: String,
    pub tolerance_pct: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RebalanceArgs {
    /// Novo aporte a distribuir (centavos da moeda base). 0 = só mostrar o desvio.
    #[ts(optional, type = "number")]
    pub contribution: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RebalanceLine {
    pub class: AssetClass,
    pub label: String,
    #[ts(type = "number")]
    pub current: i64,
    pub current_pct: f64,
    pub target_pct: f64,
    /// Quanto falta (positivo) ou sobra (negativo) para o alvo.
    #[ts(type = "number")]
    pub gap: i64,
    /// Parte sugerida do novo aporte.
    #[ts(type = "number")]
    pub suggested: i64,
    pub out_of_band: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RebalanceConcentrationLine {
    pub asset: AssetBrief,
    #[ts(type = "number")]
    pub current: i64,
    /// Peso depois do aporte e limite, frações (`over_limit` olha o peso de hoje).
    pub weight: f64,
    pub limit: f64,
    /// Quanto ainda cabe até o limite (do ativo e dos grupos dele), na moeda base.
    #[ts(type = "number")]
    pub room: i64,
    /// Parte sugerida do aporte (0 para quem já está acima do limite).
    #[ts(type = "number")]
    pub suggested: i64,
    pub over_limit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RebalanceAssetLine {
    pub asset: AssetBrief,
    #[ts(type = "number")]
    pub current: i64,
    /// Peso atual dentro da classe (%).
    pub current_pct: f64,
    /// Alvo dentro da classe (%).
    pub target_pct: f64,
    #[ts(type = "number")]
    pub gap: i64,
    /// Parte sugerida do aporte destinado à classe.
    #[ts(type = "number")]
    pub suggested: i64,
    pub out_of_band: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Rebalance {
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub contribution: i64,
    pub lines: Vec<RebalanceLine>,
    /// Ativos com alvo dentro da classe (alvos de escopo `asset`, em % da classe).
    pub assets: Vec<RebalanceAssetLine>,
    /// Soma dos alvos ≠ 100%.
    pub targets_incomplete: bool,
    /// Limites de concentração: sem alvo de classe, para onde vai o aporte sem estourar o
    /// limite de cada ativo e grupo (proporcional ao espaço que sobra); com alvo, o aporte de um
    /// ativo fica limitado ao espaço dele. Nunca sugere vender.
    pub concentration: Vec<RebalanceConcentrationLine>,
    /// Parte do aporte que não cabe em nenhum ativo sem passar do limite de concentração.
    #[serde(default)]
    #[ts(type = "number")]
    pub unallocated: i64,
    /// Análise baseada nos seus dados e alvos, não é recomendação de investimento.
    pub disclaimer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WatchlistItem {
    pub id: Id,
    pub asset: AssetBrief,
    pub target_price: Option<String>,
    pub alert_above: Option<String>,
    pub alert_below: Option<String>,
    pub notes: Option<String>,
    pub price: Option<PriceOn>,
    /// preço ÷ preço-alvo − 1
    pub upside_to_target: Option<f64>,
    pub triggered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WatchlistInput {
    pub asset_id: Id,
    #[ts(optional)]
    pub target_price: Option<String>,
    #[ts(optional)]
    pub alert_above: Option<String>,
    #[ts(optional)]
    pub alert_below: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DividendCalendarArgs {
    pub from: String,
    pub to: String,
    /// Só ativos da carteira (padrão: sim).
    #[ts(optional)]
    pub only_mine: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DividendCalendarItem {
    pub asset: AssetBrief,
    pub kind: IncomeKind,
    pub ex_date: Option<String>,
    pub pay_date: Option<String>,
    pub amount_per_share: String,
    pub currency: String,
    /// Quantidade da família na data-com (ou hoje, se futura).
    pub quantity: Option<String>,
    #[ts(type = "number | null")]
    pub expected_net: Option<i64>,
    /// Anunciado (false) ou projeção do radar (true).
    pub estimated: bool,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PendingIncomeArgs {
    /// Data-com mínima (padrão: um ano atrás).
    #[ts(optional)]
    pub since: Option<String>,
    #[ts(optional)]
    pub account_id: Option<Id>,
}

/// Provento anunciado (fonte de mercado) que a família tinha direito a receber e ainda não
/// foi lançado. `input` vai pronto para `portfolio.create_income`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PendingIncome {
    pub dividend_event_id: Id,
    pub asset: AssetBrief,
    pub account_id: Id,
    pub kind: IncomeKind,
    pub ex_date: String,
    pub pay_date: String,
    /// A fonte não informou o pagamento; usamos a data-ex (confira no extrato).
    pub pay_date_guessed: bool,
    pub quantity: String,
    pub amount_per_share: String,
    pub currency: String,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub tax_withheld: i64,
    #[ts(type = "number")]
    pub net: i64,
    pub source: String,
    /// Conferência entre as fontes (agenda do ativo e B3 oficial).
    pub check: pending::Check,
    /// Ativo sem cotação nova há meses e sem negócios: pode ter sido deslistado ou
    /// incorporado (confira antes de lançar).
    pub asset_warning: Option<String>,
    pub input: IncomeInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PassiveIncomeArgs {
    /// Meta de renda mensal (centavos da moeda base).
    #[ts(optional, type = "number")]
    pub target_monthly: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PassiveIncomeAsset {
    pub asset: AssetBrief,
    #[ts(type = "number")]
    pub last_12m: i64,
    /// Proventos dos últimos 12 meses ÷ valor de mercado atual.
    pub yield_12m: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PassiveIncome {
    #[ts(type = "number")]
    pub last_12m: i64,
    #[ts(type = "number")]
    pub monthly_avg: i64,
    /// Por mês, últimos 12 meses (mais antigo primeiro).
    pub by_month: Vec<crate::cards::MonthAmount>,
    pub by_asset: Vec<PassiveIncomeAsset>,
    #[ts(type = "number | null")]
    pub target_monthly: Option<i64>,
    /// Patrimônio necessário para a meta no yield atual da carteira.
    #[ts(type = "number | null")]
    pub capital_needed: Option<i64>,
    pub portfolio_yield_12m: Option<f64>,
}
