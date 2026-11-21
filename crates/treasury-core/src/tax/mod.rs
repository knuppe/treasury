//! Apuração de IR (fase 3): ganho de capital mensal em renda variável (ações, ETFs, BDRs,
//! FIIs; operações comuns e day trade) com isenção de R$ 20 mil/mês em ações, compensação
//! de prejuízos por categoria, IRRF "dedo-duro", DARF 6015 (mínimo de R$ 10), e o relatório
//! anual para a declaração (bens e direitos, rendimentos isentos e exclusivos, exterior).
//!
//! É uma apuração auxiliar calculada a partir dos dados do usuário; não substitui a
//! conferência no programa da Receita Federal.

pub mod annual;
pub mod annual_draft;
pub mod api;
pub mod engine;
pub mod exclusive;
pub mod gcap;
pub mod late;
pub mod rules;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_dirpf_golden;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaxCategory {
    /// Ações no mercado à vista (operações comuns) — com isenção de R$ 20 mil/mês.
    Stocks,
    /// ETFs de renda variável (operações comuns), sem isenção.
    Etf,
    /// BDRs (operações comuns), sem isenção.
    Bdr,
    /// FIIs e Fiagros (venda de cotas).
    Fii,
    /// Day trade (compra e venda do mesmo ativo no mesmo dia).
    DayTrade,
}

impl TaxCategory {
    /// Nome da categoria no idioma da instalação.
    pub fn label(self) -> String {
        crate::i18n::t(match self {
            TaxCategory::Stocks => "nucleo-impostos-ir-categoria-acoes",
            TaxCategory::Etf => "nucleo-impostos-ir-categoria-etf",
            TaxCategory::Bdr => "nucleo-impostos-ir-categoria-bdr",
            TaxCategory::Fii => "nucleo-impostos-ir-categoria-fii",
            TaxCategory::DayTrade => "nucleo-impostos-ir-categoria-day-trade",
        })
    }
}

/// Um resultado realizado (venda) que entrou na apuração.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RealizedItem {
    pub date: String,
    pub asset_id: Id,
    pub ticker: Option<String>,
    pub category: TaxCategory,
    pub quantity: String,
    /// Valor de venda bruto (centavos).
    #[ts(type = "number")]
    pub sale_value: i64,
    /// Valor líquido de custos (centavos).
    #[ts(type = "number")]
    pub proceeds: i64,
    /// Custo pelo preço médio (centavos).
    #[ts(type = "number")]
    pub cost: i64,
    #[ts(type = "number")]
    pub result: i64,
    pub trade_ids: Vec<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryMonth {
    pub category: TaxCategory,
    pub label: String,
    /// Vendas brutas no mês (centavos).
    #[ts(type = "number")]
    pub sales: i64,
    /// Resultado líquido do mês (ganhos − perdas).
    #[ts(type = "number")]
    pub result: i64,
    /// Ganho isento (ações com vendas até R$ 20 mil).
    #[ts(type = "number")]
    pub exempt_gain: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MonthStatus {
    /// Sem vendas no mês.
    NoActivity,
    /// Houve vendas, mas sem imposto (isenção, prejuízo ou compensação).
    NoTax,
    /// Imposto abaixo de R$ 10: acumula para o mês seguinte.
    BelowMinimum,
    ToPay,
    Paid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LossPool {
    /// Operações comuns: ações, ETFs, BDRs.
    Common,
    DayTrade,
    Fii,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PoolMonth {
    pub pool: LossPool,
    /// Resultado tributável do mês na categoria (após isenção).
    #[ts(type = "number")]
    pub result: i64,
    /// Prejuízo acumulado antes do mês.
    #[ts(type = "number")]
    pub loss_before: i64,
    /// Prejuízo compensado no mês.
    #[ts(type = "number")]
    pub loss_used: i64,
    /// Base de cálculo.
    #[ts(type = "number")]
    pub base: i64,
    pub rate: String,
    #[ts(type = "number")]
    pub tax: i64,
    /// Prejuízo a compensar depois do mês.
    #[ts(type = "number")]
    pub loss_after: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxMonth {
    pub month: String,
    pub member_id: Option<Id>,
    pub categories: Vec<CategoryMonth>,
    pub pools: Vec<PoolMonth>,
    /// Vendas de ações (operações comuns) no mês, base da isenção.
    #[ts(type = "number")]
    pub stock_sales: i64,
    pub stock_exempt: bool,
    /// Imposto bruto do mês (soma das categorias).
    #[ts(type = "number")]
    pub tax: i64,
    /// IRRF do mês (retido nas notas ou estimado).
    #[ts(type = "number")]
    pub irrf: i64,
    pub irrf_estimated: bool,
    /// IRRF de meses anteriores (no mesmo ano) usado agora.
    #[ts(type = "number")]
    pub irrf_carried_in: i64,
    #[ts(type = "number")]
    pub irrf_carried_out: i64,
    /// A parte de day trade do IRRF que passa adiante (vence em dezembro).
    #[ts(type = "number")]
    pub irrf_day_trade_carried_out: i64,
    /// Imposto devido no mês depois do IRRF.
    #[ts(type = "number")]
    pub due: i64,
    /// Valores abaixo do mínimo trazidos de meses anteriores.
    #[ts(type = "number")]
    pub carried_in: i64,
    /// Valor do DARF (0 se abaixo do mínimo).
    #[ts(type = "number")]
    pub darf_amount: i64,
    #[ts(type = "number")]
    pub carried_out: i64,
    /// Vencimento: último dia útil do mês seguinte.
    pub due_date: String,
    pub revenue_code: String,
    pub status: MonthStatus,
    pub darf_id: Option<Id>,
    pub realized: Vec<RealizedItem>,
    pub notes: Vec<String>,
    /// Criptoativos em corretora no Brasil (ganho de capital, DARF próprio). `None` sem vendas
    /// de cripto no mês.
    pub crypto: Option<CryptoMonth>,
}

/// Criptoativos custodiados no Brasil: ganho de capital apurado no mês, isento quando o total
/// vendido em cripto (no Brasil e no exterior) não passa de R$ 35 mil; acima disso, 15% a
/// 22,5% sobre o ganho, em DARF 4600 até o último dia útil do mês seguinte. Fontes: Lei
/// 9.250/1995 art. 22; Lei 8.981/1995 art. 21; Perguntas e Respostas IRPF 2026, pergunta 653.
/// Cripto em corretora no exterior fica no resumo anual (Lei 14.754/2023 art. 3º §1º).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CryptoMonth {
    /// Total vendido em criptoativos no mês, no Brasil e no exterior (BRL): decide a isenção.
    #[ts(type = "number")]
    pub sales: i64,
    /// A parte vendida em corretoras no Brasil.
    #[ts(type = "number")]
    pub sales_brazil: i64,
    /// Soma dos ganhos das vendas no Brasil.
    #[ts(type = "number")]
    pub gain: i64,
    /// Soma das perdas das vendas no Brasil (não abatem os ganhos).
    #[ts(type = "number")]
    pub loss: i64,
    pub exempt: bool,
    #[ts(type = "number")]
    pub tax: i64,
    /// Valores abaixo do mínimo de R$ 10 trazidos de meses anteriores.
    #[ts(type = "number")]
    pub carried_in: i64,
    #[ts(type = "number")]
    pub darf_amount: i64,
    #[ts(type = "number")]
    pub carried_out: i64,
    pub revenue_code: String,
    pub due_date: String,
    /// DARF 4600 gerado para o mês (pendente ou pago).
    pub darf_id: Option<Id>,
    pub paid: bool,
}

/// Onde a cripto de uma conta está custodiada, para o imposto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CryptoCustody {
    pub account_id: Id,
    pub name: String,
    pub currency: String,
    /// `true`: exterior (resumo anual, Lei 14.754/2023); `false`: Brasil (ganho de capital mensal).
    pub abroad: bool,
    /// Escolhido pelo usuário (senão, pela moeda da conta).
    pub chosen: bool,
    /// A conta tem operações com criptoativos.
    pub has_crypto: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetCryptoCustodyArgs {
    pub account_id: Id,
    /// `null` volta ao automático (pela moeda da conta).
    #[serde(default)]
    pub abroad: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DarfChargesArgs {
    pub id: Id,
    /// Data do pagamento (padrão: hoje).
    #[serde(default)]
    #[ts(optional)]
    pub paid_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxMonthArgs {
    pub month: String,
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Só para `generate_darf`: "4600" gera o DARF de cripto (ganho de capital); sem ele, o
    /// de renda variável (6015).
    #[serde(default)]
    #[ts(optional)]
    pub revenue_code: Option<String>,
}

/// Dividendos de uma empresa num mês acima do limite da Lei 15.270/2025.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DividendWithholding {
    pub month: String,
    /// Tickers da empresa (mesmo CNPJ, ou a mesma raiz do ticker sem CNPJ).
    pub tickers: Vec<String>,
    /// Dividendos brutos do mês (BRL).
    #[ts(type = "number")]
    pub amount: i64,
    /// 10% do total (a retenção é sobre tudo, não só sobre o que passa de R$ 50 mil).
    #[ts(type = "number")]
    pub expected_withholding: i64,
    /// Retido informado nos proventos.
    #[ts(type = "number")]
    pub withheld: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxYearArgs {
    #[ts(type = "number")]
    pub year: i32,
    #[ts(optional)]
    pub member_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeLine {
    pub asset_id: Id,
    pub ticker: Option<String>,
    pub name: String,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetHolding {
    pub asset_id: Id,
    pub ticker: Option<String>,
    pub name: String,
    pub class_label: String,
    /// Grupo e código sugeridos em Bens e Direitos (confira no programa da Receita).
    pub group_code: String,
    pub description: String,
    pub quantity_prev: String,
    #[ts(type = "number")]
    pub cost_prev: i64,
    pub quantity: String,
    /// Situação em 31/12: custo de aquisição em reais (preço médio).
    #[ts(type = "number")]
    pub cost: i64,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForeignSummary {
    /// Ganho líquido realizado no ano em aplicações no exterior (BRL).
    #[ts(type = "number")]
    pub realized: i64,
    /// Dividendos e juros recebidos do exterior (BRL, brutos: antes do imposto retido lá fora).
    #[ts(type = "number")]
    pub income: i64,
    /// Imposto retido no exterior sobre esses rendimentos (BRL).
    #[ts(type = "number")]
    pub tax_paid_abroad: i64,
    /// Crédito do imposto pago lá fora (Lei 14.754/2023 art. 4º): até o IR brasileiro de cada
    /// rendimento (15%), sem passar do imposto do ano.
    #[ts(type = "number")]
    pub tax_credit: i64,
    /// Imposto pago em países sem acordo nem reciprocidade reconhecida: não vira crédito.
    #[ts(type = "number")]
    pub tax_without_credit: i64,
    /// Prejuízo de anos anteriores (desde 2024) usado contra o ganho do ano (art. 9º).
    #[ts(type = "number")]
    pub loss_carried_in: i64,
    /// Prejuízo que sobra para os anos seguintes.
    #[ts(type = "number")]
    pub loss_carried_out: i64,
    /// Imposto estimado: 15% sobre o ganho (depois do prejuízo) e os rendimentos brutos,
    /// menos o crédito.
    #[ts(type = "number")]
    pub estimated_tax: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxYear {
    #[ts(type = "number")]
    pub year: i32,
    pub member_id: Option<Id>,
    pub months: Vec<TaxMonth>,
    /// Prejuízos a compensar em 31/12.
    #[ts(type = "number")]
    pub loss_common: i64,
    #[ts(type = "number")]
    pub loss_day_trade: i64,
    #[ts(type = "number")]
    pub loss_fii: i64,
    /// IRRF "dedo-duro" (Lei 11.033/2004) que sobrou no ano: entra na declaração de ajuste
    /// como imposto retido (art. 2º §7º, III).
    #[ts(type = "number")]
    pub irrf_lei_11033_left: i64,
    /// IRRF de day trade que sobrou no ano: não passa para a declaração; só restituição
    /// (Perguntas e Respostas IRPF 2026, perguntas 714 e 715).
    #[ts(type = "number")]
    pub irrf_day_trade_left: i64,
    #[ts(type = "number")]
    pub total_tax: i64,
    #[ts(type = "number")]
    pub total_darf: i64,
    /// Ganhos isentos de ações (vendas até R$ 20 mil no mês) — "rendimentos isentos".
    #[ts(type = "number")]
    pub exempt_stock_gains: i64,
    /// Dividendos recebidos (isentos).
    pub dividends: Vec<IncomeLine>,
    /// Meses em que uma mesma empresa pagou mais de R$ 50 mil em dividendos: retenção de 10%
    /// sobre o total (Lei 9.250/1995 art. 6º-A, desde 2026).
    pub dividends_over_limit: Vec<DividendWithholding>,
    /// Rendimentos de FII recebidos (isentos).
    pub fii_income: Vec<IncomeLine>,
    /// JCP recebidos (tributação exclusiva, líquido).
    pub jcp: Vec<IncomeLine>,
    /// Bens e direitos em 31/12.
    pub holdings: Vec<AssetHolding>,
    pub foreign: ForeignSummary,
    pub disclaimer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DarfStatus {
    Pending,
    Paid,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Darf {
    pub id: Id,
    pub member_id: Option<Id>,
    pub period: String,
    pub revenue_code: String,
    #[ts(type = "number")]
    pub principal: i64,
    #[ts(type = "number")]
    pub fine: i64,
    #[ts(type = "number")]
    pub interest: i64,
    #[ts(type = "number")]
    pub total: i64,
    pub due_date: String,
    pub status: DarfStatus,
    pub paid_at: Option<String>,
    pub txn_id: Option<Id>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MarkDarfPaidArgs {
    pub id: Id,
    pub paid_at: String,
    /// Lançamento do pagamento (despesa em Impostos e taxas), se já existir.
    #[ts(optional)]
    pub txn_id: Option<Id>,
    /// Criar o lançamento de pagamento nesta conta.
    #[ts(optional)]
    pub from_account_id: Option<Id>,
}
