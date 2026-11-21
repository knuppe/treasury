//! Patrimônio físico: imóveis, veículos, terrenos, equipamentos e coleções. Guarda a compra
//! (data, preço e custos de aquisição itemizados), as benfeitorias, as avaliações e o
//! financiamento ligado (contratos de `debt`), e calcula custo, valor atual (avaliação ou
//! estimativa automática, ver [`valuation`]), ganho nominal e real, valorização anualizada,
//! patrimônio líquido do bem (valor − saldo devedor) e LTV.

pub mod api;
pub mod fipe;
pub mod iptu;
pub mod valuation;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::debt::{kind_str, parse_kind};
use crate::error::{Error, Result};
use crate::finance::returns::{Flow, real_return, xirr};
use crate::i18n::{self, t, t_args};
use crate::ids::{Id, new_id};
use crate::indices::Deflator;
use crate::time::{fmt_date, now_utc, parse_date, today_local};
use valuation::{Appraisal, IndexTable, Model, Rule, ValueBasis};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PhysicalKind {
    RealEstate,
    Vehicle,
    Land,
    Equipment,
    Collectible,
    Other,
}

/// Estimativa automática do valor entre avaliações.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PhysicalValuationMethod {
    /// Sem estimativa: vale a última avaliação (ou o custo).
    None,
    /// Correção do custo (ou da última avaliação) por um índice.
    Index,
    /// Depreciação composta a uma taxa anual.
    Declining,
    /// Depreciação linear pela vida útil até um valor residual.
    StraightLine,
    /// Tabela FIPE (consulta grava uma avaliação); entre consultas, depreciação composta.
    Fipe,
}

/// Índices de correção de imóveis e terrenos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PhysicalIndex {
    Ipca,
    Igpm,
    Incc,
}

impl PhysicalIndex {
    pub fn series_id(self) -> &'static str {
        match self {
            PhysicalIndex::Ipca => "ipca",
            PhysicalIndex::Igpm => "igpm",
            PhysicalIndex::Incc => "incc",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            PhysicalIndex::Ipca => "IPCA",
            PhysicalIndex::Igpm => "IGP-M",
            PhysicalIndex::Incc => "INCC",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AcquisitionCostKind {
    Itbi,
    /// Escritura.
    Deed,
    /// Registro em cartório.
    Registry,
    /// Corretagem.
    Brokerage,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ImprovementCategory {
    /// Reforma.
    Renovation,
    /// Ampliação.
    Expansion,
    /// Benfeitoria.
    Improvement,
    /// Manutenção: despesa, não entra no custo.
    Maintenance,
}

impl ImprovementCategory {
    pub fn is_capital(self) -> bool {
        self != ImprovementCategory::Maintenance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PhysicalValuationSource {
    /// Estimativa do próprio usuário.
    Manual,
    /// Avaliação de corretor ou imobiliária.
    Appraisal,
    /// Laudo técnico.
    Report,
    /// Tabela FIPE.
    Fipe,
    /// Valor corrigido por índice registrado manualmente.
    Index,
}

impl PhysicalValuationSource {
    /// Nome da fonte no idioma da instalação.
    pub fn label(self) -> String {
        t(match self {
            PhysicalValuationSource::Manual => "nucleo-carteira-bem-fonte-manual",
            PhysicalValuationSource::Appraisal => "nucleo-carteira-bem-fonte-avaliacao",
            PhysicalValuationSource::Report => "nucleo-carteira-bem-fonte-laudo",
            PhysicalValuationSource::Fipe => "nucleo-carteira-bem-fonte-fipe",
            PhysicalValuationSource::Index => "nucleo-carteira-bem-fonte-indice",
        })
    }
}

/// Tipo de veículo na tabela FIPE (como no caminho da API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FipeVehicleType {
    Carros,
    Motos,
    Caminhoes,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AcquisitionCostInput {
    pub kind: AcquisitionCostKind,
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(optional)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalAcquisitionCost {
    pub id: Id,
    pub kind: AcquisitionCostKind,
    pub description: Option<String>,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalImprovement {
    pub id: Id,
    pub asset_id: Id,
    pub date: String,
    pub description: String,
    pub category: ImprovementCategory,
    #[ts(type = "number")]
    pub amount: i64,
    pub txn_id: Option<Id>,
    pub notes: Option<String>,
    /// Entra no custo (tudo menos manutenção).
    pub capital: bool,
    /// Projeto ligado: com lançamentos no projeto, a benfeitoria vale o que ele gastou
    /// (`amount`), cada lançamento na sua data.
    pub project_id: Option<Id>,
    pub project_name: Option<String>,
    /// Valor informado no cadastro (é o que vale enquanto o projeto não tem lançamentos).
    #[ts(type = "number")]
    pub entered_amount: i64,
    #[ts(type = "number")]
    pub linked_count: i64,
    /// Saídas do projeto (data, valor); vazio = `amount` na `date`.
    #[serde(skip)]
    #[ts(skip)]
    pub flows: Vec<(String, i64)>,
}

impl PhysicalImprovement {
    /// Gastos da benfeitoria com as datas: os lançamentos do projeto ou o valor informado.
    fn spends(&self) -> Vec<(String, i64)> {
        if self.flows.is_empty() {
            vec![(self.date.clone(), self.amount)]
        } else {
            self.flows.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalValuation {
    pub id: Id,
    pub asset_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub value: i64,
    pub source: PhysicalValuationSource,
    pub notes: Option<String>,
}

/// Financiamento ligado ao bem, com o saldo devedor na data.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalLinkedLoan {
    pub loan_id: Id,
    pub name: String,
    pub kind: crate::debt::LoanKind,
    #[ts(type = "number")]
    pub principal: i64,
    #[ts(type = "number")]
    pub balance: i64,
    pub currency: String,
    pub archived: bool,
}

/// Números do bem numa data. Valores na moeda do bem.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalFigures {
    pub as_of: String,
    /// Compra + custos de aquisição.
    #[ts(type = "number")]
    pub initial_cost: i64,
    /// Benfeitorias de capital (reforma, ampliação, benfeitoria) até a data.
    #[ts(type = "number")]
    pub capital_improvements: i64,
    /// Manutenção até a data (despesa, fora do custo).
    #[ts(type = "number")]
    pub maintenance: i64,
    /// Custo = compra + custos + benfeitorias de capital.
    #[ts(type = "number")]
    pub cost_basis: i64,
    /// Valor atual (0 depois da venda).
    #[ts(type = "number")]
    pub value: i64,
    pub value_basis: ValueBasis,
    pub estimated: bool,
    /// Como o valor foi obtido, em português.
    pub formula: String,
    /// Fonte curta do valor.
    pub source: String,
    /// Âncora da estimativa: última avaliação até a data, ou a compra.
    pub anchor_date: String,
    #[ts(type = "number")]
    pub anchor_value: i64,
    /// Ganho nominal = valor − custo (na venda: líquido da venda − custo).
    #[ts(type = "number")]
    pub nominal_gain: i64,
    pub nominal_gain_pct: Option<f64>,
    pub deflator: Deflator,
    /// Custo trazido a valores da data pelo deflator (cada parcela na sua data).
    #[ts(type = "number | null")]
    pub real_cost_basis: Option<i64>,
    /// Ganho real = valor − custo corrigido.
    #[ts(type = "number | null")]
    pub real_gain: Option<i64>,
    pub real_gain_pct: Option<f64>,
    /// Valorização anualizada (TIR dos aportes: compra, custos e benfeitorias × valor).
    pub annualized: Option<f64>,
    /// Valorização anualizada real (descontado o deflator).
    pub real_annualized: Option<f64>,
    /// Saldo devedor dos financiamentos ligados.
    #[ts(type = "number")]
    pub loan_balance: i64,
    /// Patrimônio líquido do bem = valor − saldo devedor.
    #[ts(type = "number")]
    pub equity: i64,
    /// Saldo devedor ÷ valor.
    pub ltv: Option<f64>,
    pub sold: bool,
    /// Valor líquido da venda (preço − custos).
    #[ts(type = "number | null")]
    pub sale_net: Option<i64>,
    #[ts(type = "number")]
    pub holding_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalAssetView {
    pub id: Id,
    pub kind: PhysicalKind,
    pub name: String,
    pub member_ids: Vec<Id>,
    pub purchase_date: String,
    #[ts(type = "number")]
    pub purchase_price: i64,
    pub currency: String,
    pub acquisition_costs: Vec<PhysicalAcquisitionCost>,
    pub valuation_method: PhysicalValuationMethod,
    pub index_series: Option<PhysicalIndex>,
    /// % a.a. (decimal texto).
    pub depreciation_rate: Option<String>,
    pub useful_life_years: Option<u32>,
    /// % do custo ao fim da vida útil (decimal texto).
    pub residual_pct: Option<String>,
    pub fipe_vehicle_type: Option<FipeVehicleType>,
    pub fipe_brand_code: Option<String>,
    pub fipe_model_code: Option<String>,
    pub fipe_year_code: Option<String>,
    pub fipe_code: Option<String>,
    pub address: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub include_in_net_worth: bool,
    pub archived: bool,
    pub sale_date: Option<String>,
    #[ts(type = "number | null")]
    pub sale_price: Option<i64>,
    #[ts(type = "number")]
    pub sale_costs: i64,
    pub improvements: Vec<PhysicalImprovement>,
    pub valuations: Vec<PhysicalValuation>,
    pub loans: Vec<PhysicalLinkedLoan>,
    pub figures: PhysicalFigures,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalAssetInput {
    pub name: String,
    pub kind: PhysicalKind,
    pub purchase_date: String,
    #[ts(type = "number")]
    pub purchase_price: i64,
    #[ts(optional)]
    pub currency: Option<String>,
    #[ts(optional)]
    pub member_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub acquisition_costs: Option<Vec<AcquisitionCostInput>>,
    /// Padrão por tipo: imóvel/terreno → índice (IPCA), veículo → depreciação 12% a.a.
    /// (FIPE com os códigos), equipamento → linear em 5 anos, demais → sem estimativa.
    #[ts(optional)]
    pub valuation_method: Option<PhysicalValuationMethod>,
    #[ts(optional)]
    pub index_series: Option<PhysicalIndex>,
    #[ts(optional)]
    pub depreciation_rate: Option<String>,
    #[ts(optional)]
    pub useful_life_years: Option<u32>,
    #[ts(optional)]
    pub residual_pct: Option<String>,
    #[ts(optional)]
    pub fipe_vehicle_type: Option<FipeVehicleType>,
    #[ts(optional)]
    pub fipe_brand_code: Option<String>,
    #[ts(optional)]
    pub fipe_model_code: Option<String>,
    #[ts(optional)]
    pub fipe_year_code: Option<String>,
    #[ts(optional)]
    pub fipe_code: Option<String>,
    #[ts(optional)]
    pub address: Option<String>,
    #[ts(optional)]
    pub description: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
    #[ts(optional)]
    pub include_in_net_worth: Option<bool>,
    /// Financiamentos (`debt`) a ligar ao bem.
    #[ts(optional)]
    pub loan_ids: Option<Vec<Id>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalAssetPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub kind: Option<PhysicalKind>,
    #[ts(optional)]
    pub purchase_date: Option<String>,
    #[ts(optional, type = "number")]
    pub purchase_price: Option<i64>,
    #[ts(optional)]
    pub member_ids: Option<Vec<Id>>,
    /// Substitui todos os custos de aquisição.
    #[ts(optional)]
    pub acquisition_costs: Option<Vec<AcquisitionCostInput>>,
    #[ts(optional)]
    pub valuation_method: Option<PhysicalValuationMethod>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub index_series: Option<Option<PhysicalIndex>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub depreciation_rate: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub useful_life_years: Option<Option<u32>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub residual_pct: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub fipe_vehicle_type: Option<Option<FipeVehicleType>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub fipe_brand_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub fipe_model_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub fipe_year_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub fipe_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub address: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
    #[ts(optional)]
    pub include_in_net_worth: Option<bool>,
    #[ts(optional)]
    pub archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalImprovementInput {
    pub asset_id: Id,
    pub date: String,
    pub description: String,
    pub category: ImprovementCategory,
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(optional)]
    pub txn_id: Option<Id>,
    #[ts(optional)]
    pub notes: Option<String>,
    /// Projeto dos gastos da obra (veja `projects.list`).
    #[serde(default)]
    #[ts(optional)]
    pub project_id: Option<Id>,
    /// Cria um projeto com o nome da benfeitoria, do mesmo bem, e liga a ela.
    #[serde(default)]
    #[ts(optional)]
    pub create_project: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalImprovementPatch {
    pub id: Id,
    #[ts(optional)]
    pub date: Option<String>,
    #[ts(optional)]
    pub description: Option<String>,
    #[ts(optional)]
    pub category: Option<ImprovementCategory>,
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub txn_id: Option<Option<Id>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub project_id: Option<Option<Id>>,
    /// Cria um projeto com o nome da benfeitoria, do mesmo bem, e liga a ela.
    #[serde(default)]
    #[ts(optional)]
    pub create_project: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalValuationInput {
    pub asset_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub value: i64,
    pub source: PhysicalValuationSource,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalSaleArgs {
    pub id: Id,
    pub sale_date: String,
    #[ts(type = "number")]
    pub sale_price: i64,
    /// Corretagem, impostos e taxas da venda.
    #[ts(optional, type = "number")]
    pub sale_costs: Option<i64>,
    /// Arquivar o bem (padrão: sim).
    #[ts(optional)]
    pub archive: Option<bool>,
}

/// Argumentos de leitura: data e deflator do ganho real.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalReadArgs {
    #[ts(optional)]
    pub as_of: Option<String>,
    #[ts(optional)]
    pub deflator: Option<Deflator>,
    #[ts(optional)]
    pub include_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalSummary {
    pub as_of: String,
    /// Moeda dos totais (a moeda base da família).
    pub currency: String,
    pub count: u32,
    #[ts(type = "number")]
    pub total_value: i64,
    #[ts(type = "number")]
    pub total_cost_basis: i64,
    #[ts(type = "number")]
    pub total_gain: i64,
    #[ts(type = "number")]
    pub total_loan_balance: i64,
    #[ts(type = "number")]
    pub total_equity: i64,
    /// Parte do valor que entra no patrimônio líquido (`include_in_net_worth`).
    #[ts(type = "number")]
    pub net_worth_value: i64,
    pub estimated: bool,
    /// Moedas sem câmbio (bens fora dos totais).
    pub missing_fx: Vec<String>,
    pub assets: Vec<PhysicalAssetView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalHistoryPoint {
    pub month: String,
    pub as_of: String,
    #[ts(type = "number")]
    pub cost_basis: i64,
    #[ts(type = "number")]
    pub value: i64,
    pub estimated: bool,
    /// Avaliação registrada no mês (a última), se houver.
    #[ts(type = "number | null")]
    pub appraisal: Option<i64>,
    #[ts(type = "number")]
    pub loan_balance: i64,
    #[ts(type = "number")]
    pub equity: i64,
}

// ---------------------------------------------------------------------------
// Linhas

/// Linha de `physical_asset` (também a forma editada no `update`).
#[derive(Debug, Clone)]
struct AssetRow {
    id: Id,
    kind: PhysicalKind,
    name: String,
    purchase_date: String,
    purchase_price: i64,
    currency: String,
    valuation_method: PhysicalValuationMethod,
    index_series: Option<PhysicalIndex>,
    depreciation_rate: Option<String>,
    useful_life_years: Option<u32>,
    residual_pct: Option<String>,
    fipe_vehicle_type: Option<FipeVehicleType>,
    fipe_brand_code: Option<String>,
    fipe_model_code: Option<String>,
    fipe_year_code: Option<String>,
    fipe_code: Option<String>,
    address: Option<String>,
    description: Option<String>,
    notes: Option<String>,
    include_in_net_worth: bool,
    archived: bool,
    sale_date: Option<String>,
    sale_price: Option<i64>,
    sale_costs: i64,
}

/// Taxa padrão de depreciação de veículos: carros perdem ~15–20% no 1º ano e ~8–10% nos
/// seguintes; 12% a.a. composto é a média dos primeiros cinco anos.
pub const DEFAULT_VEHICLE_RATE: &str = "12";
/// Vida útil padrão de equipamentos (eletrônicos e máquinas leves; a Receita usa 5 anos
/// para computadores e 10 para máquinas).
pub const DEFAULT_USEFUL_LIFE: u32 = 5;

fn default_method(kind: PhysicalKind, has_fipe: bool) -> PhysicalValuationMethod {
    match kind {
        PhysicalKind::RealEstate | PhysicalKind::Land => PhysicalValuationMethod::Index,
        PhysicalKind::Vehicle if has_fipe => PhysicalValuationMethod::Fipe,
        PhysicalKind::Vehicle => PhysicalValuationMethod::Declining,
        PhysicalKind::Equipment => PhysicalValuationMethod::StraightLine,
        PhysicalKind::Collectible | PhysicalKind::Other => PhysicalValuationMethod::None,
    }
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Percentual de um campo; `what` é a chave do nome do campo (`nucleo-carteira-bem-campo-…`).
fn pct(s: &str, what: &str) -> Result<Decimal> {
    s.trim()
        .replace(',', ".")
        .parse::<Decimal>()
        .map_err(|_| crate::portfolio::records::invalid_field(what, s))
}

fn ref_of(key: &str, id: &str) -> String {
    t_args(key, &i18n::args(&[("id", id.to_string().into())]))
}

fn norm_pct(s: Option<String>, what: &str, max: Decimal) -> Result<Option<String>> {
    match clean(s) {
        None => Ok(None),
        Some(v) => {
            let d = pct(&v, what)?;
            if d < Decimal::ZERO || d > max {
                return Err(Error::invalid(t_args(
                    "nucleo-carteira-bem-erro-fora-intervalo",
                    &i18n::args(&[("campo", t(what).into())]),
                )));
            }
            Ok(Some(d.normalize().to_string()))
        }
    }
}

impl AssetRow {
    fn validate(&mut self) -> Result<()> {
        self.name = self.name.trim().to_string();
        if self.name.is_empty() {
            return Err(Error::invalid(t("nucleo-carteira-bem-erro-nome")));
        }
        let pd = parse_date(&self.purchase_date)?;
        if self.purchase_price < 0 {
            return Err(Error::invalid(t("nucleo-carteira-bem-erro-preco")));
        }
        self.currency = crate::money::validate_currency(&self.currency)?;
        self.depreciation_rate = norm_pct(
            self.depreciation_rate.take(),
            "nucleo-carteira-bem-campo-taxa-depreciacao",
            Decimal::ONE_HUNDRED,
        )?;
        self.residual_pct = norm_pct(
            self.residual_pct.take(),
            "nucleo-carteira-bem-campo-valor-residual",
            Decimal::ONE_HUNDRED,
        )?;
        if let Some(y) = self.useful_life_years
            && !(1..=100).contains(&y)
        {
            return Err(Error::invalid(t("nucleo-carteira-bem-erro-vida-util")));
        }
        for f in [
            &mut self.fipe_brand_code,
            &mut self.fipe_model_code,
            &mut self.fipe_year_code,
            &mut self.fipe_code,
            &mut self.address,
            &mut self.description,
            &mut self.notes,
        ] {
            *f = clean(f.take());
        }
        match self.valuation_method {
            PhysicalValuationMethod::Index if self.index_series.is_none() => {
                self.index_series = Some(PhysicalIndex::Ipca);
            }
            PhysicalValuationMethod::Declining if self.depreciation_rate.is_none() => {
                self.depreciation_rate = Some(DEFAULT_VEHICLE_RATE.into());
            }
            PhysicalValuationMethod::StraightLine if self.useful_life_years.is_none() => {
                self.useful_life_years = Some(DEFAULT_USEFUL_LIFE);
            }
            PhysicalValuationMethod::Fipe => {
                if self.fipe_vehicle_type.is_none()
                    || self.fipe_brand_code.is_none()
                    || self.fipe_model_code.is_none()
                    || self.fipe_year_code.is_none()
                {
                    return Err(Error::invalid(t("nucleo-carteira-bem-erro-fipe-dados")));
                }
                if self.depreciation_rate.is_none() {
                    self.depreciation_rate = Some(DEFAULT_VEHICLE_RATE.into());
                }
            }
            _ => {}
        }
        if let (Some(sd), Some(sp)) = (&self.sale_date, self.sale_price) {
            if parse_date(sd)? < pd {
                return Err(Error::invalid(t("nucleo-carteira-bem-erro-venda-antes")));
            }
            if sp < 0 || self.sale_costs < 0 {
                return Err(Error::invalid(t("nucleo-carteira-bem-erro-venda-negativa")));
            }
        }
        Ok(())
    }

    fn save(&self, tx: &Transaction<'_>, insert: bool) -> Result<()> {
        let now = now_utc();
        let p = params![
            self.id,
            kind_str(&self.kind),
            self.name,
            self.purchase_date,
            self.purchase_price,
            self.currency,
            kind_str(&self.valuation_method),
            self.index_series.map(|i| i.series_id()),
            self.depreciation_rate,
            self.useful_life_years,
            self.residual_pct,
            self.fipe_vehicle_type.map(|t| kind_str(&t)),
            self.fipe_brand_code,
            self.fipe_model_code,
            self.fipe_year_code,
            self.fipe_code,
            self.address,
            self.description,
            self.notes,
            self.include_in_net_worth as i64,
            self.archived as i64,
            self.sale_date,
            self.sale_price,
            self.sale_costs,
            now
        ];
        if insert {
            tx.execute(
                "INSERT INTO physical_asset (id, kind, name, purchase_date, purchase_price, currency, valuation_method,
                    index_series, depreciation_rate, useful_life_years, residual_pct, fipe_vehicle_type, fipe_brand_code,
                    fipe_model_code, fipe_year_code, fipe_code, address, description, notes, include_in_net_worth,
                    archived, sale_date, sale_price, sale_costs, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
                         ?21, ?22, ?23, ?24, ?25, ?25)",
                p,
            )?;
        } else {
            tx.execute(
                "UPDATE physical_asset SET kind = ?2, name = ?3, purchase_date = ?4, purchase_price = ?5, currency = ?6,
                    valuation_method = ?7, index_series = ?8, depreciation_rate = ?9, useful_life_years = ?10,
                    residual_pct = ?11, fipe_vehicle_type = ?12, fipe_brand_code = ?13, fipe_model_code = ?14,
                    fipe_year_code = ?15, fipe_code = ?16, address = ?17, description = ?18, notes = ?19,
                    include_in_net_worth = ?20, archived = ?21, sale_date = ?22, sale_price = ?23, sale_costs = ?24,
                    updated_at = ?25
                 WHERE id = ?1",
                p,
            )?;
        }
        Ok(())
    }
}

fn index_from_str(s: &str) -> Option<PhysicalIndex> {
    match s {
        "ipca" => Some(PhysicalIndex::Ipca),
        "igpm" => Some(PhysicalIndex::Igpm),
        "incc" => Some(PhysicalIndex::Incc),
        _ => None,
    }
}

fn load_row(conn: &Connection, id: &str) -> Result<AssetRow> {
    conn.query_row("SELECT * FROM physical_asset WHERE id = ?1", [id], |r| {
        Ok(AssetRow {
            id: r.get("id")?,
            kind: parse_kind(&r.get::<_, String>("kind")?)?,
            name: r.get("name")?,
            purchase_date: r.get("purchase_date")?,
            purchase_price: r.get("purchase_price")?,
            currency: r.get("currency")?,
            valuation_method: parse_kind(&r.get::<_, String>("valuation_method")?)?,
            index_series: r
                .get::<_, Option<String>>("index_series")?
                .as_deref()
                .and_then(index_from_str),
            depreciation_rate: r.get("depreciation_rate")?,
            useful_life_years: r.get("useful_life_years")?,
            residual_pct: r.get("residual_pct")?,
            fipe_vehicle_type: r
                .get::<_, Option<String>>("fipe_vehicle_type")?
                .map(|s| parse_kind(&s))
                .transpose()?,
            fipe_brand_code: r.get("fipe_brand_code")?,
            fipe_model_code: r.get("fipe_model_code")?,
            fipe_year_code: r.get("fipe_year_code")?,
            fipe_code: r.get("fipe_code")?,
            address: r.get("address")?,
            description: r.get("description")?,
            notes: r.get("notes")?,
            include_in_net_worth: r.get::<_, i64>("include_in_net_worth")? == 1,
            archived: r.get::<_, i64>("archived")? == 1,
            sale_date: r.get("sale_date")?,
            sale_price: r.get("sale_price")?,
            sale_costs: r.get("sale_costs")?,
        })
    })
    .optional()?
    .ok_or_else(|| Error::not_found(ref_of("nucleo-carteira-bem", id)))
}

fn require_asset(conn: &Connection, id: &str) -> Result<()> {
    let ok: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM physical_asset WHERE id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    if ok {
        Ok(())
    } else {
        Err(Error::not_found(ref_of("nucleo-carteira-bem", id)))
    }
}

fn members(conn: &Connection, id: &str) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare(
        "SELECT pm.member_id FROM physical_asset_member pm JOIN member m ON m.id = pm.member_id
         WHERE pm.asset_id = ?1 ORDER BY m.sort_order, m.name",
    )?;
    Ok(stmt
        .query_map([id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

fn replace_members(tx: &Transaction<'_>, id: &str, ids: &[Id]) -> Result<()> {
    let mut wanted: Vec<&Id> = vec![];
    for m in ids {
        if !wanted.contains(&m) {
            crate::household::get_member(tx, m)?;
            wanted.push(m);
        }
    }
    let current = members(tx, id)?;
    for m in current.iter().filter(|m| !wanted.contains(m)) {
        tx.execute(
            "DELETE FROM physical_asset_member WHERE asset_id = ?1 AND member_id = ?2",
            params![id, m],
        )?;
    }
    for m in wanted.iter().filter(|m| !current.contains(m)) {
        tx.execute(
            "INSERT INTO physical_asset_member (asset_id, member_id) VALUES (?1, ?2)",
            params![id, m],
        )?;
    }
    Ok(())
}

fn replace_costs(tx: &Transaction<'_>, id: &str, costs: &[AcquisitionCostInput]) -> Result<()> {
    if costs.iter().any(|c| c.amount < 0) {
        return Err(Error::invalid(t("nucleo-carteira-bem-erro-custos")));
    }
    tx.execute(
        "DELETE FROM physical_acquisition_cost WHERE asset_id = ?1",
        [id],
    )?;
    let now = now_utc();
    for c in costs.iter().filter(|c| c.amount > 0) {
        tx.execute(
            "INSERT INTO physical_acquisition_cost (id, asset_id, kind, description, amount, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![new_id(), id, kind_str(&c.kind), clean(c.description.clone()), c.amount, now],
        )?;
    }
    Ok(())
}

fn costs(conn: &Connection, id: &str) -> Result<Vec<PhysicalAcquisitionCost>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, description, amount FROM physical_acquisition_cost WHERE asset_id = ?1 ORDER BY created_at, id",
    )?;
    Ok(stmt
        .query_map([id], |r| {
            Ok(PhysicalAcquisitionCost {
                id: r.get(0)?,
                kind: parse_kind(&r.get::<_, String>(1)?)?,
                description: r.get(2)?,
                amount: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

fn improvements(conn: &Connection, id: &str) -> Result<Vec<PhysicalImprovement>> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.asset_id, i.date, i.description, i.category, i.amount, i.txn_id, i.notes,
                i.project_id, p.name
           FROM physical_improvement i LEFT JOIN project p ON p.id = i.project_id
         WHERE i.asset_id = ?1 ORDER BY i.date, i.created_at, i.id",
    )?;
    let mut list = stmt
        .query_map([id], |r| {
            let category: ImprovementCategory = parse_kind(&r.get::<_, String>(4)?)?;
            let amount: i64 = r.get(5)?;
            Ok(PhysicalImprovement {
                id: r.get(0)?,
                asset_id: r.get(1)?,
                date: r.get(2)?,
                description: r.get(3)?,
                category,
                amount,
                txn_id: r.get(6)?,
                notes: r.get(7)?,
                capital: category.is_capital(),
                project_id: r.get(8)?,
                project_name: r.get(9)?,
                entered_amount: amount,
                linked_count: 0,
                flows: vec![],
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for i in &mut list {
        if let Some(p) = &i.project_id {
            i.flows = crate::projects::outflows(conn, p)?;
            i.linked_count = i.flows.len() as i64;
            if !i.flows.is_empty() {
                i.amount = i.flows.iter().map(|(_, v)| v).sum();
            }
        }
    }
    Ok(list)
}

fn valuations(conn: &Connection, id: &str) -> Result<Vec<PhysicalValuation>> {
    let mut stmt = conn.prepare(
        "SELECT id, asset_id, date, value, source, notes FROM physical_valuation
         WHERE asset_id = ?1 ORDER BY date, created_at, id",
    )?;
    Ok(stmt
        .query_map([id], |r| {
            Ok(PhysicalValuation {
                id: r.get(0)?,
                asset_id: r.get(1)?,
                date: r.get(2)?,
                value: r.get(3)?,
                source: parse_kind(&r.get::<_, String>(4)?)?,
                notes: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

fn loan_ids(conn: &Connection, id: &str) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare(
        "SELECT pl.loan_id FROM physical_asset_loan pl JOIN loan l ON l.id = pl.loan_id
         WHERE pl.asset_id = ?1 ORDER BY l.contract_date, l.id",
    )?;
    Ok(stmt
        .query_map([id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

// ---------------------------------------------------------------------------
// Escritas

pub fn create(tx: &Transaction<'_>, i: &PhysicalAssetInput) -> Result<Id> {
    let has_fipe = i.fipe_vehicle_type.is_some()
        && i.fipe_brand_code.is_some()
        && i.fipe_model_code.is_some()
        && i.fipe_year_code.is_some();
    let base = crate::household::base_currency(tx)?;
    let mut row = AssetRow {
        id: new_id(),
        kind: i.kind,
        name: i.name.clone(),
        purchase_date: i.purchase_date.clone(),
        purchase_price: i.purchase_price,
        currency: i.currency.clone().unwrap_or(base),
        valuation_method: i
            .valuation_method
            .unwrap_or_else(|| default_method(i.kind, has_fipe)),
        index_series: i.index_series,
        depreciation_rate: i.depreciation_rate.clone(),
        useful_life_years: i.useful_life_years,
        residual_pct: i.residual_pct.clone(),
        fipe_vehicle_type: i.fipe_vehicle_type,
        fipe_brand_code: i.fipe_brand_code.clone(),
        fipe_model_code: i.fipe_model_code.clone(),
        fipe_year_code: i.fipe_year_code.clone(),
        fipe_code: i.fipe_code.clone(),
        address: i.address.clone(),
        description: i.description.clone(),
        notes: i.notes.clone(),
        include_in_net_worth: i.include_in_net_worth.unwrap_or(true),
        archived: false,
        sale_date: None,
        sale_price: None,
        sale_costs: 0,
    };
    row.validate()?;
    row.save(tx, true)?;
    replace_members(tx, &row.id, i.member_ids.as_deref().unwrap_or_default())?;
    replace_costs(
        tx,
        &row.id,
        i.acquisition_costs.as_deref().unwrap_or_default(),
    )?;
    for l in i.loan_ids.iter().flatten() {
        link_loan(tx, &row.id, l)?;
    }
    Ok(row.id)
}

pub fn update(tx: &Transaction<'_>, p: &PhysicalAssetPatch) -> Result<()> {
    let mut row = load_row(tx, &p.id)?;
    if let Some(v) = &p.name {
        row.name = v.clone();
    }
    if let Some(v) = p.kind {
        row.kind = v;
    }
    if let Some(v) = &p.purchase_date {
        row.purchase_date = v.clone();
    }
    if let Some(v) = p.purchase_price {
        row.purchase_price = v;
    }
    if let Some(v) = p.valuation_method {
        row.valuation_method = v;
    }
    if let Some(v) = p.index_series {
        row.index_series = v;
    }
    if let Some(v) = &p.depreciation_rate {
        row.depreciation_rate = v.clone();
    }
    if let Some(v) = p.useful_life_years {
        row.useful_life_years = v;
    }
    if let Some(v) = &p.residual_pct {
        row.residual_pct = v.clone();
    }
    if let Some(v) = p.fipe_vehicle_type {
        row.fipe_vehicle_type = v;
    }
    for (src, dst) in [
        (&p.fipe_brand_code, &mut row.fipe_brand_code),
        (&p.fipe_model_code, &mut row.fipe_model_code),
        (&p.fipe_year_code, &mut row.fipe_year_code),
        (&p.fipe_code, &mut row.fipe_code),
        (&p.address, &mut row.address),
        (&p.description, &mut row.description),
        (&p.notes, &mut row.notes),
    ] {
        if let Some(v) = src {
            *dst = v.clone();
        }
    }
    if let Some(v) = p.include_in_net_worth {
        row.include_in_net_worth = v;
    }
    if let Some(v) = p.archived {
        row.archived = v;
    }
    row.validate()?;
    row.save(tx, false)?;
    if let Some(m) = &p.member_ids {
        replace_members(tx, &row.id, m)?;
    }
    if let Some(c) = &p.acquisition_costs {
        replace_costs(tx, &row.id, c)?;
    }
    Ok(())
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM physical_asset WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(ref_of("nucleo-carteira-bem", id)));
    }
    Ok(())
}

fn check_txn(conn: &Connection, txn_id: &Option<Id>) -> Result<()> {
    if let Some(t) = txn_id {
        let ok: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM txn WHERE id = ?1)",
            [t],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(Error::not_found(ref_of(
                "nucleo-lancamentos-ref-lancamento",
                t,
            )));
        }
    }
    Ok(())
}

fn check_improvement(description: &str, amount: i64, date: &str) -> Result<()> {
    if description.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-carteira-bem-erro-benfeitoria")));
    }
    if amount <= 0 {
        return Err(Error::invalid(t("nucleo-carteira-bem-erro-valor-obra")));
    }
    parse_date(date)?;
    Ok(())
}

/// Projeto da benfeitoria: o escolhido, um novo com o nome dela (`create_project`) ou
/// nenhum.
fn improvement_project(
    tx: &Transaction<'_>,
    asset_id: &str,
    description: &str,
    date: &str,
    project_id: Option<Id>,
    create: bool,
) -> Result<Option<Id>> {
    if create {
        let id = crate::projects::create(
            tx,
            &crate::projects::ProjectInput {
                name: description.trim().to_string(),
                physical_asset_id: Some(asset_id.to_string()),
                status: None,
                budget: None,
                start_date: Some(date.to_string()),
                end_date: None,
                notes: None,
            },
        )?;
        return Ok(Some(id));
    }
    let project_id = clean(project_id);
    crate::projects::check(tx, project_id.as_deref())?;
    Ok(project_id)
}

/// Valor guardado da benfeitoria: o informado ou, ligada a um projeto e sem valor, o gasto
/// (ou o orçamento) do projeto.
fn improvement_amount(conn: &Connection, amount: i64, project_id: Option<&str>) -> Result<i64> {
    let Some(p) = project_id.filter(|_| amount <= 0) else {
        return Ok(amount);
    };
    let spent = crate::projects::spent(conn, p)?;
    let budget = crate::projects::get(conn, p)?.budget.unwrap_or(0);
    let v = if spent > 0 { spent } else { budget };
    if v <= 0 {
        return Err(Error::invalid(t("nucleo-carteira-bem-erro-valor-projeto")));
    }
    Ok(v)
}

pub fn add_improvement(tx: &Transaction<'_>, i: &PhysicalImprovementInput) -> Result<Id> {
    require_asset(tx, &i.asset_id)?;
    parse_date(&i.date)?;
    let project = improvement_project(
        tx,
        &i.asset_id,
        &i.description,
        &i.date,
        i.project_id.clone(),
        i.create_project.unwrap_or(false),
    )?;
    let amount = improvement_amount(tx, i.amount, project.as_deref())?;
    check_improvement(&i.description, amount, &i.date)?;
    check_txn(tx, &i.txn_id)?;
    let id = new_id();
    tx.execute(
        "INSERT INTO physical_improvement (id, asset_id, date, description, category, amount, txn_id, notes, project_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![
            id,
            i.asset_id,
            i.date,
            i.description.trim(),
            kind_str(&i.category),
            amount,
            i.txn_id,
            clean(i.notes.clone()),
            project,
            now_utc()
        ],
    )?;
    Ok(id)
}

/// Altera uma benfeitoria; devolve o bem dela.
pub fn update_improvement(tx: &Transaction<'_>, p: &PhysicalImprovementPatch) -> Result<Id> {
    let cur = tx
        .query_row(
            "SELECT asset_id, date, description, category, amount, txn_id, notes, project_id FROM physical_improvement WHERE id = ?1",
            [&p.id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    parse_kind::<ImprovementCategory>(&r.get::<_, String>(3)?)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| Error::not_found(ref_of("nucleo-carteira-bem-benfeitoria", &p.id)))?;
    let date = p.date.clone().unwrap_or(cur.1);
    let description = p.description.clone().unwrap_or(cur.2);
    let category = p.category.unwrap_or(cur.3);
    let txn_id = p.txn_id.clone().unwrap_or(cur.5);
    let notes = p.notes.clone().map(clean).unwrap_or(cur.6);
    parse_date(&date)?;
    let project = improvement_project(
        tx,
        &cur.0,
        &description,
        &date,
        p.project_id.clone().unwrap_or(cur.7),
        p.create_project.unwrap_or(false),
    )?;
    let amount = improvement_amount(tx, p.amount.unwrap_or(cur.4), project.as_deref())?;
    check_improvement(&description, amount, &date)?;
    check_txn(tx, &txn_id)?;
    tx.execute(
        "UPDATE physical_improvement SET date = ?2, description = ?3, category = ?4, amount = ?5, txn_id = ?6, notes = ?7,
            project_id = ?8, updated_at = ?9 WHERE id = ?1",
        params![
            p.id,
            date,
            description.trim(),
            kind_str(&category),
            amount,
            txn_id,
            notes,
            project,
            now_utc()
        ],
    )?;
    Ok(cur.0)
}

/// Apaga uma benfeitoria; devolve o bem dela.
pub fn delete_improvement(tx: &Transaction<'_>, id: &str) -> Result<Id> {
    let asset: Option<String> = tx
        .query_row(
            "SELECT asset_id FROM physical_improvement WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let asset =
        asset.ok_or_else(|| Error::not_found(ref_of("nucleo-carteira-bem-benfeitoria", id)))?;
    tx.execute("DELETE FROM physical_improvement WHERE id = ?1", [id])?;
    Ok(asset)
}

pub fn add_valuation(tx: &Transaction<'_>, v: &PhysicalValuationInput) -> Result<Id> {
    require_asset(tx, &v.asset_id)?;
    parse_date(&v.date)?;
    if v.value < 0 {
        return Err(Error::invalid(t("nucleo-carteira-bem-erro-avaliacao")));
    }
    let id = new_id();
    tx.execute(
        "INSERT INTO physical_valuation (id, asset_id, date, value, source, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            v.asset_id,
            v.date,
            v.value,
            kind_str(&v.source),
            clean(v.notes.clone()),
            now_utc()
        ],
    )?;
    Ok(id)
}

/// Apaga uma avaliação; devolve o bem dela.
pub fn delete_valuation(tx: &Transaction<'_>, id: &str) -> Result<Id> {
    let asset: Option<String> = tx
        .query_row(
            "SELECT asset_id FROM physical_valuation WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let asset =
        asset.ok_or_else(|| Error::not_found(ref_of("nucleo-carteira-bem-avaliacao", id)))?;
    tx.execute("DELETE FROM physical_valuation WHERE id = ?1", [id])?;
    Ok(asset)
}

pub fn link_loan(tx: &Transaction<'_>, asset_id: &str, loan_id: &str) -> Result<()> {
    require_asset(tx, asset_id)?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM loan WHERE id = ?1)",
        [loan_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(Error::not_found(ref_of(
            "nucleo-planejamento-quitacao-contrato",
            loan_id,
        )));
    }
    let other: Option<(String, String)> = tx
        .query_row(
            "SELECT pa.id, pa.name FROM physical_asset_loan pl JOIN physical_asset pa ON pa.id = pl.asset_id
             WHERE pl.loan_id = ?1",
            [loan_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match other {
        Some((a, _)) if a == asset_id => Ok(()),
        Some((_, name)) => Err(Error::conflict(t_args(
            "nucleo-carteira-bem-erro-contrato-outro",
            &i18n::args(&[("bem", name.into())]),
        ))),
        None => {
            tx.execute(
                "INSERT INTO physical_asset_loan (asset_id, loan_id) VALUES (?1, ?2)",
                params![asset_id, loan_id],
            )?;
            Ok(())
        }
    }
}

pub fn unlink_loan(tx: &Transaction<'_>, asset_id: &str, loan_id: &str) -> Result<()> {
    if tx.execute(
        "DELETE FROM physical_asset_loan WHERE asset_id = ?1 AND loan_id = ?2",
        params![asset_id, loan_id],
    )? == 0
    {
        return Err(Error::not_found(t("nucleo-carteira-bem-ligacao")));
    }
    Ok(())
}

pub fn sell(tx: &Transaction<'_>, a: &PhysicalSaleArgs) -> Result<()> {
    let mut row = load_row(tx, &a.id)?;
    if a.sale_price < 0 || a.sale_costs.unwrap_or(0) < 0 {
        return Err(Error::invalid(t("nucleo-carteira-bem-erro-venda-negativa")));
    }
    row.sale_date = Some(a.sale_date.clone());
    row.sale_price = Some(a.sale_price);
    row.sale_costs = a.sale_costs.unwrap_or(0);
    row.archived = a.archive.unwrap_or(true);
    row.validate()?;
    row.save(tx, false)
}

// ---------------------------------------------------------------------------
// Leitura e cálculo

/// Tudo o que o motor precisa de um bem (carregado uma vez; vale para várias datas).
pub(crate) struct Loaded {
    row: AssetRow,
    members: Vec<Id>,
    costs: Vec<PhysicalAcquisitionCost>,
    improvements: Vec<PhysicalImprovement>,
    valuations: Vec<PhysicalValuation>,
    loans: Vec<Id>,
    model: Model,
}

impl Loaded {
    pub fn id(&self) -> &str {
        &self.row.id
    }
    pub fn name(&self) -> &str {
        &self.row.name
    }
    pub fn currency(&self) -> &str {
        &self.row.currency
    }
    /// Entra no patrimônio líquido na data (incluído, comprado, não vendido e não
    /// arquivado sem venda).
    pub fn counts_at(&self, at: NaiveDate) -> bool {
        if !self.row.include_in_net_worth || self.model.purchase_date > at {
            return false;
        }
        if self.row.archived && self.row.sale_date.is_none() {
            return false;
        }
        !matches!(self.model.sale, Some((d, _)) if d <= at)
    }
    pub fn estimate(&self, at: NaiveDate) -> valuation::Estimate {
        self.model.value_at(at)
    }
}

/// Tabelas de índice compartilhadas entre bens.
type Tables = HashMap<&'static str, IndexTable>;

fn table_for(conn: &Connection, tables: &mut Tables, idx: PhysicalIndex) -> Result<IndexTable> {
    if let Some(t) = tables.get(idx.series_id()) {
        return Ok(t.clone());
    }
    let from = parse_date("1990-01-01")?;
    let to = crate::time::add_months(today_local(), 24);
    let t = IndexTable {
        rates: crate::indices::monthly_rates(conn, idx.series_id(), from, to)?,
    };
    tables.insert(idx.series_id(), t.clone());
    Ok(t)
}

fn load_with(conn: &Connection, id: &str, tables: &mut Tables) -> Result<Loaded> {
    let row = load_row(conn, id)?;
    let costs = costs(conn, id)?;
    let improvements = improvements(conn, id)?;
    let valuations = valuations(conn, id)?;
    let purchase_date = parse_date(&row.purchase_date)?;
    let rule = match row.valuation_method {
        PhysicalValuationMethod::None => Rule::Flat,
        PhysicalValuationMethod::Index => {
            let idx = row.index_series.unwrap_or(PhysicalIndex::Ipca);
            Rule::Index {
                name: idx.label().into(),
                table: table_for(conn, tables, idx)?,
            }
        }
        PhysicalValuationMethod::Declining | PhysicalValuationMethod::Fipe => Rule::Declining {
            rate_pct: pct(
                row.depreciation_rate
                    .as_deref()
                    .unwrap_or(DEFAULT_VEHICLE_RATE),
                "nucleo-carteira-bem-campo-taxa",
            )?,
            fipe: row.valuation_method == PhysicalValuationMethod::Fipe,
        },
        PhysicalValuationMethod::StraightLine => Rule::StraightLine {
            life_years: row.useful_life_years.unwrap_or(DEFAULT_USEFUL_LIFE),
            residual_pct: pct(
                row.residual_pct.as_deref().unwrap_or("0"),
                "nucleo-carteira-bem-campo-residual",
            )?,
        },
    };
    let capital = improvements
        .iter()
        .filter(|i| i.capital)
        .flat_map(PhysicalImprovement::spends)
        .map(|(d, v)| Ok((parse_date(&d)?, v)))
        .collect::<Result<Vec<_>>>()?;
    let appraisals = valuations
        .iter()
        .map(|v| {
            Ok(Appraisal {
                date: parse_date(&v.date)?,
                value: v.value,
                source: v.source.label(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let sale = match (&row.sale_date, row.sale_price) {
        (Some(d), Some(p)) => Some((parse_date(d)?, p - row.sale_costs)),
        _ => None,
    };
    let model = Model {
        currency: row.currency.clone(),
        purchase_date,
        initial_cost: row.purchase_price + costs.iter().map(|c| c.amount).sum::<i64>(),
        capital,
        appraisals,
        rule,
        sale,
    };
    Ok(Loaded {
        members: members(conn, id)?,
        loans: loan_ids(conn, id)?,
        row,
        costs,
        improvements,
        valuations,
        model,
    })
}

fn all_ids(conn: &Connection, include_archived: bool) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM physical_asset WHERE (?1 OR archived = 0) ORDER BY purchase_date, name, id",
    )?;
    Ok(stmt
        .query_map([include_archived], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

/// Todos os bens carregados (para o patrimônio mês a mês).
pub(crate) fn load_all(conn: &Connection) -> Result<Vec<Loaded>> {
    let mut tables = Tables::new();
    all_ids(conn, true)?
        .iter()
        .map(|id| load_with(conn, id, &mut tables))
        .collect()
}

/// Saldo devedor de um contrato na moeda do bem.
fn loan_balance_in(conn: &Connection, loan_id: &str, at: &str, currency: &str) -> Result<i64> {
    let b = crate::debt::balance_at(conn, loan_id, at)?;
    let ccy: String =
        conn.query_row("SELECT currency FROM loan WHERE id = ?1", [loan_id], |r| {
            r.get(0)
        })?;
    if ccy == currency {
        return Ok(b);
    }
    Ok(crate::fx::convert(conn, b, &ccy, currency, at)?
        .map(|c| c.amount)
        .unwrap_or(b))
}

fn linked_loans(conn: &Connection, l: &Loaded, at: &str) -> Result<Vec<PhysicalLinkedLoan>> {
    l.loans
        .iter()
        .map(|id| {
            let (name, kind, principal, currency, archived): (String, String, i64, String, i64) =
                conn.query_row(
                    "SELECT name, kind, principal, currency, archived FROM loan WHERE id = ?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                )?;
            Ok(PhysicalLinkedLoan {
                loan_id: id.clone(),
                name,
                kind: parse_kind(&kind)?,
                principal,
                balance: crate::debt::balance_at(conn, id, at)?,
                currency,
                archived: archived == 1,
            })
        })
        .collect()
}

fn ratio(a: i64, b: i64) -> Option<f64> {
    (b != 0).then(|| a as f64 / b as f64)
}

fn figures(
    conn: &Connection,
    l: &Loaded,
    at: NaiveDate,
    deflator: Deflator,
) -> Result<PhysicalFigures> {
    let as_of = fmt_date(at);
    let m = &l.model;
    let e = m.value_at(at);
    let sold = e.basis == ValueBasis::Sold;
    // Na venda, as contas param na data da venda.
    let end = match m.sale {
        Some((d, _)) if sold => d,
        _ => at,
    };
    let cost_basis = m.cost_basis(end);
    let capital_improvements = cost_basis - m.initial_cost;
    let maintenance = l
        .improvements
        .iter()
        .filter(|i| !i.capital)
        .flat_map(PhysicalImprovement::spends)
        .filter(|(d, _)| d.as_str() <= as_of.as_str())
        .map(|(_, v)| v)
        .sum();
    let sale_net = m.sale.filter(|_| sold).map(|(_, v)| v);
    let final_value = sale_net.unwrap_or(e.value);
    let nominal_gain = final_value - cost_basis;
    // Aportes: custo inicial na compra e benfeitorias nas suas datas.
    let mut contributions = vec![(m.purchase_date, m.initial_cost)];
    contributions.extend(m.capital.iter().filter(|(d, _)| *d <= end).copied());
    let mut real_cost = Some(0i64);
    for (d, v) in &contributions {
        let f = crate::indices::deflator_factor(conn, deflator, (*d).max(m.purchase_date), end)?;
        real_cost = match (real_cost, f) {
            (Some(acc), Some(f)) => Some(acc + crate::finance::inflation::adjust(*v, f.factor)),
            _ => None,
        };
    }
    let real_gain = real_cost.map(|c| final_value - c);
    let holding_days = (end - m.purchase_date).num_days().max(0);
    let annualized = if holding_days >= 30 && final_value > 0 {
        let mut flows: Vec<Flow> = contributions
            .iter()
            .filter(|(_, v)| *v > 0)
            .map(|(d, v)| Flow {
                date: (*d).max(m.purchase_date),
                amount: -(*v as f64),
            })
            .collect();
        flows.push(Flow {
            date: end,
            amount: final_value as f64,
        });
        xirr(&flows)
    } else {
        None
    };
    let real_annualized = match annualized {
        Some(a) if holding_days > 0 => {
            crate::indices::deflator_factor(conn, deflator, m.purchase_date, end)?.map(|f| {
                let infl = f
                    .factor
                    .to_f64()
                    .unwrap_or(1.0)
                    .powf(365.0 / holding_days as f64)
                    - 1.0;
                real_return(a, infl)
            })
        }
        _ => None,
    };
    let mut loan_balance = 0;
    for id in &l.loans {
        loan_balance += loan_balance_in(conn, id, &as_of, &m.currency)?;
    }
    if sold {
        loan_balance = 0;
    }
    Ok(PhysicalFigures {
        as_of: as_of.clone(),
        initial_cost: m.initial_cost,
        capital_improvements,
        maintenance,
        cost_basis,
        value: e.value,
        value_basis: e.basis,
        estimated: e.estimated,
        formula: e.formula,
        source: e.source,
        anchor_date: fmt_date(e.anchor_date),
        anchor_value: e.anchor_value,
        nominal_gain,
        nominal_gain_pct: ratio(nominal_gain, cost_basis),
        deflator,
        real_cost_basis: real_cost,
        real_gain,
        real_gain_pct: real_cost.and_then(|c| real_gain.and_then(|g| ratio(g, c))),
        annualized,
        real_annualized,
        loan_balance,
        equity: e.value - loan_balance,
        ltv: if e.value > 0 {
            ratio(loan_balance, e.value)
        } else {
            None
        },
        sold,
        sale_net,
        holding_days,
    })
}

fn to_view(
    conn: &Connection,
    l: Loaded,
    at: NaiveDate,
    deflator: Deflator,
) -> Result<PhysicalAssetView> {
    let figures = figures(conn, &l, at, deflator)?;
    let loans = linked_loans(conn, &l, &figures.as_of)?;
    let r = l.row;
    Ok(PhysicalAssetView {
        id: r.id,
        kind: r.kind,
        name: r.name,
        member_ids: l.members,
        purchase_date: r.purchase_date,
        purchase_price: r.purchase_price,
        currency: r.currency,
        acquisition_costs: l.costs,
        valuation_method: r.valuation_method,
        index_series: r.index_series,
        depreciation_rate: r.depreciation_rate,
        useful_life_years: r.useful_life_years,
        residual_pct: r.residual_pct,
        fipe_vehicle_type: r.fipe_vehicle_type,
        fipe_brand_code: r.fipe_brand_code,
        fipe_model_code: r.fipe_model_code,
        fipe_year_code: r.fipe_year_code,
        fipe_code: r.fipe_code,
        address: r.address,
        description: r.description,
        notes: r.notes,
        include_in_net_worth: r.include_in_net_worth,
        archived: r.archived,
        sale_date: r.sale_date,
        sale_price: r.sale_price,
        sale_costs: r.sale_costs,
        improvements: l.improvements,
        valuations: l.valuations,
        loans,
        figures,
    })
}

fn read_date(a: &PhysicalReadArgs) -> Result<NaiveDate> {
    match &a.as_of {
        Some(d) => parse_date(d),
        None => Ok(today_local()),
    }
}

pub fn view(conn: &Connection, id: &str, a: &PhysicalReadArgs) -> Result<PhysicalAssetView> {
    let at = read_date(a)?;
    let l = load_with(conn, id, &mut Tables::new())?;
    to_view(
        conn,
        l,
        at,
        crate::indices::effective_deflator(conn, a.deflator.unwrap_or_default()),
    )
}

pub fn list(conn: &Connection, a: &PhysicalReadArgs) -> Result<Vec<PhysicalAssetView>> {
    let at = read_date(a)?;
    let mut tables = Tables::new();
    all_ids(conn, a.include_archived.unwrap_or(false))?
        .iter()
        .map(|id| {
            let l = load_with(conn, id, &mut tables)?;
            to_view(
                conn,
                l,
                at,
                crate::indices::effective_deflator(conn, a.deflator.unwrap_or_default()),
            )
        })
        .collect()
}

pub fn summary(conn: &Connection, a: &PhysicalReadArgs) -> Result<PhysicalSummary> {
    let assets = list(conn, a)?;
    let base = crate::household::base_currency(conn)?;
    let as_of = fmt_date(read_date(a)?);
    let mut s = PhysicalSummary {
        as_of: as_of.clone(),
        currency: base.clone(),
        count: assets.len() as u32,
        total_value: 0,
        total_cost_basis: 0,
        total_gain: 0,
        total_loan_balance: 0,
        total_equity: 0,
        net_worth_value: 0,
        estimated: false,
        missing_fx: vec![],
        assets: vec![],
    };
    for v in &assets {
        let conv = |x: i64| -> Result<Option<i64>> {
            if v.currency == base {
                return Ok(Some(x));
            }
            Ok(crate::fx::convert(conn, x, &v.currency, &base, &as_of)?.map(|c| c.amount))
        };
        let f = &v.figures;
        let (Some(val), Some(cost), Some(loan)) =
            (conv(f.value)?, conv(f.cost_basis)?, conv(f.loan_balance)?)
        else {
            if !s.missing_fx.contains(&v.currency) {
                s.missing_fx.push(v.currency.clone());
            }
            continue;
        };
        if f.sold {
            continue;
        }
        s.total_value += val;
        s.total_cost_basis += cost;
        s.total_loan_balance += loan;
        s.estimated |= f.estimated;
        if v.include_in_net_worth && !(v.archived && v.sale_date.is_none()) {
            s.net_worth_value += val;
        }
    }
    s.total_gain = s.total_value - s.total_cost_basis;
    s.total_equity = s.total_value - s.total_loan_balance;
    s.assets = assets;
    Ok(s)
}

/// Valor mês a mês desde a compra (até 240 meses), com custo, avaliações e o saldo
/// devedor dos financiamentos ligados.
pub fn history(
    conn: &Connection,
    id: &str,
    months: Option<u32>,
) -> Result<Vec<PhysicalHistoryPoint>> {
    let l = load_with(conn, id, &mut Tables::new())?;
    let today = today_local();
    let end = match l.model.sale {
        Some((d, _)) if d <= today => d,
        _ => today,
    };
    let max = months.unwrap_or(240).clamp(1, 240) as i32;
    let pm = l.model.purchase_date;
    let mut start = crate::time::add_months(end, -(max - 1));
    if start < pm {
        start = pm;
    }
    let mut out = vec![];
    let (mut y, mut mo) = (
        chrono::Datelike::year(&start),
        chrono::Datelike::month(&start),
    );
    loop {
        let (_, last) = crate::time::month_bounds(y, mo);
        let at = last.min(end);
        let month = crate::time::fmt_month(y, mo);
        let e = l.model.value_at(at);
        let as_of = fmt_date(at);
        let mut loan_balance = 0;
        for lid in &l.loans {
            loan_balance += loan_balance_in(conn, lid, &as_of, &l.model.currency)?;
        }
        let appraisal = l
            .valuations
            .iter()
            .rfind(|v| v.date.starts_with(&month) && v.date.as_str() <= as_of.as_str())
            .map(|v| v.value);
        out.push(PhysicalHistoryPoint {
            month,
            as_of,
            cost_basis: l.model.cost_basis(at),
            value: e.value,
            estimated: e.estimated,
            appraisal,
            loan_balance,
            equity: e.value - loan_balance,
        });
        if at >= end {
            break;
        }
        (y, mo) = if mo == 12 { (y + 1, 1) } else { (y, mo + 1) };
    }
    Ok(out)
}

/// Bem no patrimônio numa data: valor na moeda do bem.
pub(crate) struct NetWorthItem {
    pub id: Id,
    pub name: String,
    pub currency: String,
    pub value: i64,
    pub estimated: bool,
    pub formula: String,
}

pub(crate) fn net_worth_items(loaded: &[Loaded], at: NaiveDate) -> Vec<NetWorthItem> {
    loaded
        .iter()
        .filter(|l| l.counts_at(at))
        .map(|l| {
            let e = l.estimate(at);
            NetWorthItem {
                id: l.id().to_string(),
                name: l.name().to_string(),
                currency: l.currency().to_string(),
                value: e.value,
                estimated: e.estimated,
                formula: e.formula,
            }
        })
        .filter(|i| i.value != 0)
        .collect()
}

/// Bens com consulta FIPE configurada: (tipo, marca, modelo, ano, código FIPE esperado).
pub(crate) fn fipe_ref(
    conn: &Connection,
    id: &str,
) -> Result<(FipeVehicleType, String, String, String, Option<String>)> {
    let r = load_row(conn, id)?;
    match (
        r.fipe_vehicle_type,
        r.fipe_brand_code,
        r.fipe_model_code,
        r.fipe_year_code,
    ) {
        (Some(t), Some(b), Some(m), Some(y)) => Ok((t, b, m, y, r.fipe_code)),
        _ => Err(Error::invalid(t("nucleo-carteira-bem-erro-fipe-ref"))),
    }
}
