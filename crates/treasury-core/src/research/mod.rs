//! Análise de ativos (fase 3, docs/investments.md): fundamentos de companhias abertas e
//! FIIs (CVM Dados Abertos), de empresas americanas (SEC EDGAR), indicadores prudenciais dos
//! bancos (Banco Central, IF.data) e cotações do universo (B3 COTAHIST), com indicadores,
//! preço justo (Graham e Bazin), rankings, screener, telas salvas, comparador, documentos IPE
//! e radar de dividendos.
//!
//! Organização:
//! - [`cvm`], [`sec`], [`b3`]: adaptadores isolados (só parse de bytes, testados com fixtures).
//! - [`net`]: downloads condicionais (ETag) atrás de um trait, para testar a coleta sem rede.
//! - [`sync`]: planejar → baixar sem travar o banco → gravar, arquivo a arquivo, com relatório
//!   de erros por item.
//! - [`store`]: leitura e gravação de emissores, códigos de negociação e fatos.
//! - [`periods`]: exercício, acumulado no ano, trimestres e últimos 12 meses (LTM).
//! - [`indicators`]: indicadores e preço justo, sempre com fórmula, período e fonte.
//! - [`screen`]: universo diário, rankings, screener e telas salvas.
//! - [`radar`]: projeção de proventos a partir do histórico.
//! - [`ifdata`]: indicadores prudenciais dos bancos (Banco Central, IF.data).
//!
//! Dados baixados ficam fora do histórico de alterações; só as telas salvas passam por
//! `ctx.write`. Textos vindos da CVM e da SEC (nomes, assuntos de documentos) são dados,
//! nunca instruções.

pub mod api;
pub mod b3;
pub mod b3_listados;
pub mod brapi;
pub mod config;
pub mod cvm;
pub mod data;
pub mod dividends;
pub mod doc_read;
pub mod fundamentus;
pub mod ifdata;
pub mod indicators;
pub mod market;
pub mod methods;
pub mod net;
pub mod news;
pub mod periods;
pub mod radar;
pub mod screen;
pub mod screen_alerts;
pub mod sec;
pub mod store;
pub mod sync;
pub mod tags;
pub mod views;

#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};

/// Tipo de período de um fato.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PeriodType {
    /// Saldo numa data (balanço, número de ações).
    Instant,
    /// Exercício social completo.
    Annual,
    /// Acumulado do início do exercício até o fim do trimestre (ITR, 10-Q).
    Ytd,
    /// Três meses (derivado das diferenças entre acumulados).
    Quarter,
    /// Mês de referência (informe mensal de FII).
    Month,
    /// Últimos 12 meses, calculado.
    Ltm,
}

impl PeriodType {
    pub fn as_str(self) -> &'static str {
        match self {
            PeriodType::Instant => "instant",
            PeriodType::Annual => "annual",
            PeriodType::Ytd => "ytd",
            PeriodType::Quarter => "quarter",
            PeriodType::Month => "month",
            PeriodType::Ltm => "ltm",
        }
    }
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "instant" => PeriodType::Instant,
            "annual" => PeriodType::Annual,
            "ytd" => PeriodType::Ytd,
            "quarter" => PeriodType::Quarter,
            "month" => PeriodType::Month,
            "ltm" => PeriodType::Ltm,
            _ => {
                return Err(Error::invalid(crate::i18n::t_args(
                    "nucleo-pesquisa-erro-tipo-periodo",
                    &crate::i18n::args(&[("tipo", s.to_string().into())]),
                )));
            }
        })
    }
}

/// Valor de um item das demonstrações num período (unidades da moeda, escala já aplicada).
#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub item: String,
    pub period_type: PeriodType,
    pub start: Option<NaiveDate>,
    pub end: NaiveDate,
    pub value: Decimal,
    /// BRL, USD; `shares` para quantidades de ações ou cotas; `ratio` para razões.
    pub currency: String,
    pub source: String,
    pub document: String,
    /// Dado original do documento (2) vence a reapresentação no documento seguinte (1).
    pub priority: i64,
}

/// Tipo de emissor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum IssuerKind {
    CompanyBr,
    Fii,
    CompanyUs,
}

impl IssuerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            IssuerKind::CompanyBr => "company_br",
            IssuerKind::Fii => "fii",
            IssuerKind::CompanyUs => "company_us",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "fii" => IssuerKind::Fii,
            "company_us" => IssuerKind::CompanyUs,
            _ => IssuerKind::CompanyBr,
        }
    }
}

/// Leiaute das demonstrações: decide quais indicadores se aplicam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Layout {
    /// Empresas não financeiras (receita, EBIT, dívida, circulante).
    Industrial,
    /// Instituições financeiras: sem EBITDA, dívida líquida nem liquidez corrente.
    Bank,
    /// Seguradoras: idem.
    Insurance,
    /// Fundos (FIIs).
    Fund,
}

impl Layout {
    pub fn as_str(self) -> &'static str {
        match self {
            Layout::Industrial => "industrial",
            Layout::Bank => "bank",
            Layout::Insurance => "insurance",
            Layout::Fund => "fund",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "bank" => Layout::Bank,
            "insurance" => Layout::Insurance,
            "fund" => Layout::Fund,
            _ => Layout::Industrial,
        }
    }
    pub fn is_financial(self) -> bool {
        matches!(self, Layout::Bank | Layout::Insurance)
    }
}

/// Emissor (companhia aberta, FII ou empresa americana).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Issuer {
    /// CNPJ só com dígitos ou `cik:0000320193`.
    pub id: String,
    pub kind: IssuerKind,
    pub name: String,
    pub cvm_code: Option<String>,
    pub sector: Option<String>,
    pub segment: Option<String>,
    pub mandate: Option<String>,
    pub layout: Layout,
    pub currency: String,
    pub source: String,
}

/// Código de negociação ligado a um emissor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Security {
    pub ticker: String,
    pub issuer_id: String,
    /// `stock_br`, `fii`, `bdr`, `stock_us`.
    pub class: String,
    /// ON, PN, UNIT, CTF (cota de FII), COMMON.
    pub share_kind: Option<String>,
    /// Composição da unit (ex.: "1 ON / 2 PN").
    pub unit_composition: Option<String>,
    pub isin: Option<String>,
    pub listing_segment: Option<String>,
    pub active: bool,
    pub source: String,
}

impl Security {
    /// Ações por unit (soma dos números da composição: "1 ON / 2 PN" = 3).
    pub fn shares_per_unit(&self) -> Option<Decimal> {
        if self.share_kind.as_deref() != Some("UNIT") {
            return Some(Decimal::ONE);
        }
        let comp = self.unit_composition.as_deref()?;
        let total = unit_count(comp);
        (total > 0).then(|| Decimal::from(total))
    }
}

/// Ações numa composição de unit: soma os números que são quantidade ("1 ON / 2 PN",
/// "1 ON e 2PNs", "1 KLBN3 + 4 KLBN4" = 5). Número colado depois de uma letra é parte de um
/// código de negociação (o 3 de KLBN3) e não conta.
pub fn unit_count(comp: &str) -> u32 {
    let chars: Vec<char> = comp.chars().collect();
    let mut total = 0u32;
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let glued = start > 0 && chars[start - 1].is_alphabetic();
            if !glued {
                let n: String = chars[start..i].iter().collect();
                total += n.parse::<u32>().unwrap_or(0);
            }
        } else {
            i += 1;
        }
    }
    total
}

/// Chaves dos itens gravados em `fundamental`.
pub mod items {
    pub const REVENUE: &str = "revenue";
    pub const GROSS_PROFIT: &str = "gross_profit";
    pub const EBIT: &str = "ebit";
    pub const DA: &str = "da";
    pub const NET_INCOME: &str = "net_income";
    pub const CFO: &str = "cfo";
    pub const CAPEX: &str = "capex";
    pub const DIVIDENDS_PAID: &str = "dividends_paid";
    pub const DIVIDENDS_DECLARED: &str = "dividends_declared";
    /// Dividendos declarados por ação (SEC).
    pub const DPS_DECLARED: &str = "dps_declared";
    // Convenção de sinal dos itens abaixo: custos, despesas e saídas de caixa são gravados
    // como magnitudes positivas, salvo indicação em contrário.
    /// Custo dos bens e/ou serviços vendidos (3.02, magnitude). Só no leiaute industrial.
    pub const COGS: &str = "cogs";
    /// Despesas com vendas + gerais e administrativas (filhas de 3.04, magnitude). Industrial.
    pub const SGA: &str = "sga";
    /// Receitas financeiras (filha do resultado financeiro, positiva). Industrial.
    pub const FIN_INCOME: &str = "fin_income";
    /// Despesas financeiras (filha do resultado financeiro, magnitude). Industrial.
    pub const FIN_EXPENSE: &str = "fin_expense";
    /// Resultado antes dos tributos sobre o lucro (como informado).
    pub const EBT: &str = "ebt";
    /// IR e CSLL sobre o lucro, como informado (negativo = despesa).
    pub const INCOME_TAX: &str = "income_tax";
    /// Aquisição de ações próprias (recompra, tesouraria) na DFC 6.03 (magnitude).
    pub const BUYBACKS: &str = "buybacks";
    /// Aumento de capital, emissão e integralização de ações na DFC 6.03 (positivo).
    pub const SHARE_ISSUANCE: &str = "share_issuance";

    pub const TOTAL_ASSETS: &str = "total_assets";
    pub const CURRENT_ASSETS: &str = "current_assets";
    pub const CASH: &str = "cash";
    pub const CURRENT_LIABILITIES: &str = "current_liabilities";
    pub const TOTAL_LIABILITIES: &str = "total_liabilities";
    pub const GROSS_DEBT: &str = "gross_debt";
    pub const EQUITY: &str = "equity";
    pub const SHARES: &str = "shares_outstanding";
    pub const SHARES_ON: &str = "shares_on";
    pub const SHARES_PN: &str = "shares_pn";
    /// Contas a receber de curto prazo (1.01.03).
    pub const RECEIVABLES: &str = "receivables";
    /// Estoques de curto prazo (1.01.04).
    pub const INVENTORIES: &str = "inventories";
    /// Imobilizado (1.02.03).
    pub const PPE: &str = "ppe";
    /// Intangível (1.02.04).
    pub const INTANGIBLE: &str = "intangible";
    /// Fornecedores de curto prazo (2.01.02). Industrial.
    pub const SUPPLIERS: &str = "suppliers";
    /// Empréstimos e financiamentos de curto prazo (2.01.04). Industrial.
    pub const DEBT_ST: &str = "debt_st";
    /// Empréstimos e financiamentos de longo prazo (2.02.01). Industrial.
    pub const DEBT_LT: &str = "debt_lt";
    /// Passivos de arrendamento fora de 2.01.04/2.02.01 (linhas "arrendamento" sob 2.01.05 e
    /// 2.02.02). Industrial.
    pub const LEASES: &str = "leases";
    /// Lucros retidos no Altman BR = reservas de lucro + lucros/prejuízos acumulados, como
    /// informado (pode ser negativo).
    pub const RETAINED: &str = "retained";
    /// Participação dos não controladores no patrimônio líquido.
    pub const MINORITY: &str = "minority";

    // Conferência dos dados de entrada (docs/indicators.md, "Input validation").
    /// Lucro consolidado do período, com a parte dos não controladores (margem líquida e ROA).
    pub const NET_INCOME_TOTAL: &str = "net_income_total";
    /// Lucro dos controladores no consolidado, como a companhia informou (pode vir zerado).
    pub const NET_INCOME_CONSOLIDATED: &str = "net_income_consolidated";
    /// Lucro da demonstração individual (pela equivalência, o lucro dos controladores).
    pub const NET_INCOME_INDIVIDUAL: &str = "net_income_individual";
    /// Resultado de equivalência patrimonial (filha de 3.04). Industrial.
    pub const EQUITY_INCOME: &str = "equity_income";
    /// Patrimônio dos controladores no consolidado (total menos não controladores).
    pub const EQUITY_CONSOLIDATED: &str = "equity_consolidated";
    /// Patrimônio da demonstração individual.
    pub const EQUITY_INDIVIDUAL: &str = "equity_individual";
    /// Ações do capital integralizado (com as em tesouraria), da composição do capital.
    pub const SHARES_TOTAL: &str = "shares_total";
    /// Ações em tesouraria, da composição do capital.
    pub const SHARES_TREASURY: &str = "shares_treasury";
    /// Total de ações do capital social na B3 (API de empresas listadas), com a data da
    /// consulta.
    pub const SHARES_B3: &str = "shares_b3";
    pub const SHARES_B3_ON: &str = "shares_b3_on";
    pub const SHARES_B3_PN: &str = "shares_b3_pn";
    /// Comparativo reapresentado no documento seguinte (o original fica no item sem sufixo).
    pub const REVENUE_RESTATED: &str = "revenue_restated";
    pub const EBIT_RESTATED: &str = "ebit_restated";
    pub const NET_INCOME_RESTATED: &str = "net_income_restated";

    // FIIs (informe mensal).
    pub const NAV: &str = "nav";
    pub const NAV_PER_SHARE: &str = "nav_per_share";
    pub const FII_SHARES: &str = "fii_shares";
    pub const HOLDERS: &str = "holders";
    pub const DY_MONTH: &str = "dy_month";
    pub const INCOME_PER_SHARE: &str = "income_per_share";
    pub const FII_TOTAL_ASSETS: &str = "fii_total_assets";
    pub const FII_LIABILITIES: &str = "fii_liabilities";
    pub const FII_ACQ_OBLIGATIONS: &str = "fii_acq_obligations";
    pub const FII_SECURITIZATION: &str = "fii_securitization";
    pub const FII_REAL_ESTATE: &str = "fii_real_estate";
    pub const FII_CRI: &str = "fii_cri";
    pub const FII_FII_SHARES: &str = "fii_fii_shares";
    pub const FII_LIQUIDITY: &str = "fii_liquidity";
    pub const FII_RENT_RECEIVABLE: &str = "fii_rent_receivable";
    /// Despesa com taxa de administração no mês ÷ PL (fração).
    pub const FII_ADMIN_FEE_PCT: &str = "fii_admin_fee_pct";
    /// Rentabilidade patrimonial do mês (fração).
    pub const FII_EQUITY_RETURN_MONTH: &str = "fii_equity_return_month";

    // FIIs (informe trimestral).
    pub const FII_VACANCY_FINANCIAL: &str = "fii_vacancy_financial";
    pub const FII_VACANCY_PHYSICAL: &str = "fii_vacancy_physical";
    pub const FII_DEFAULT_RATE: &str = "fii_default_rate";
    /// Inadimplência ponderada pela área (sem a participação de cada imóvel na receita).
    pub const FII_DEFAULT_RATE_AREA: &str = "fii_default_rate_area";
    pub const FII_TOP_PROPERTY_SHARE: &str = "fii_top_property_share";
    /// Maior imóvel pela área (sem a participação de cada imóvel na receita).
    pub const FII_TOP_PROPERTY_SHARE_AREA: &str = "fii_top_property_share_area";
    pub const FII_PROPERTIES: &str = "fii_properties";
    pub const FII_TOP_TENANT_SHARE: &str = "fii_top_tenant_share";
    pub const FII_WALT_YEARS: &str = "fii_walt_years";
    pub const FII_UNDETERMINED_SHARE: &str = "fii_undetermined_share";
    pub const FII_RECURRING_RESULT: &str = "fii_recurring_result";
    pub const FII_DECLARED_INCOME: &str = "fii_declared_income";
    pub const FII_CRI_TOP_ISSUER_SHARE: &str = "fii_cri_top_issuer_share";

    /// Fluxos (somam no tempo e têm LTM).
    pub const FLOWS: &[&str] = &[
        REVENUE,
        GROSS_PROFIT,
        EBIT,
        DA,
        NET_INCOME,
        CFO,
        CAPEX,
        DIVIDENDS_PAID,
        DIVIDENDS_DECLARED,
        DPS_DECLARED,
        COGS,
        SGA,
        FIN_INCOME,
        FIN_EXPENSE,
        EBT,
        INCOME_TAX,
        BUYBACKS,
        SHARE_ISSUANCE,
        NET_INCOME_TOTAL,
        NET_INCOME_CONSOLIDATED,
        NET_INCOME_INDIVIDUAL,
        EQUITY_INCOME,
    ];

    /// Nome do item no idioma da instalação (`nucleo-pesquisa-item-<item>`; desconhecido:
    /// "Item").
    pub fn label(item: &str) -> String {
        let key = format!("nucleo-pesquisa-item-{item}");
        if crate::i18n::has_key(crate::i18n::Locale::PtBr, &key) {
            crate::i18n::t(&key)
        } else {
            crate::i18n::t("nucleo-pesquisa-item-desconhecido")
        }
    }

    pub fn is_flow(item: &str) -> bool {
        FLOWS.contains(&item)
    }
}

/// CNPJ só com dígitos.
pub fn cnpj_digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}

/// CNPJ formatado (`33.000.167/0001-01`).
pub fn format_cnpj(digits: &str) -> String {
    if digits.len() != 14 {
        return digits.to_string();
    }
    format!(
        "{}.{}.{}/{}-{}",
        &digits[0..2],
        &digits[2..5],
        &digits[5..8],
        &digits[8..12],
        &digits[12..14]
    )
}

/// Texto normalizado para comparar descrições de contas (minúsculas, sem acentos nem
/// pontuação).
pub(crate) fn norm(s: &str) -> String {
    crate::transactions::normalize_description(s)
}

/// Decimal de texto da CVM/SEC: ponto decimal, aceita notação científica.
pub(crate) fn parse_decimal(s: &str) -> Option<Decimal> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(d) = t.parse::<Decimal>() {
        return Some(d);
    }
    if t.contains(['e', 'E']) {
        return Decimal::from_scientific(t).ok();
    }
    None
}

/// Decimal → f64 para razões de exibição.
pub(crate) fn to_f64(d: Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(f64::NAN)
}

pub(crate) fn dstr(d: Decimal) -> String {
    d.normalize().to_string()
}

#[cfg(test)]
mod unit_tests {
    use super::unit_count;

    #[test]
    fn unit_count_ignores_ticker_digits() {
        assert_eq!(unit_count("1 ON / 2 PN"), 3);
        assert_eq!(unit_count("1 ON e 2PNs"), 3);
        assert_eq!(unit_count("1 KLBN3 + 4 KLBN4"), 5);
        assert_eq!(unit_count("2 ações preferenciais e 1 ação ordinária"), 3);
        assert_eq!(unit_count("1 ON E 2 PNA"), 3);
        assert_eq!(unit_count(""), 0);
    }
}
