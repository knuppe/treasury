//! Saúde financeira: uma nota de 0 a 100 calculada com os dados do próprio cofre, por um de
//! dois modelos (ou nenhum, se o usuário desligar a análise em Ajustes):
//!
//! - **I-SFB** (padrão): inspirado no Índice de Saúde Financeira do Brasileiro (Febraban e
//!   Banco Central, 2024: escala de 0 a 100, média nacional 56,7). Quatro dimensões
//!   (comportamento, habilidade, liberdade e segurança), cada uma de 0 a 100, e a média
//!   ponderada dá a nota e a faixa. O índice oficial vem de um questionário; aqui é uma
//!   **estimativa** a partir dos lançamentos, das parcelas, das faturas e dos saldos.
//! - **Indicadores clássicos**: as razões da educação financeira (taxa de poupança, reserva
//!   de emergência, comprometimento da renda, custo da moradia, crédito rotativo e evolução
//!   do patrimônio), cada uma com meta e pontos.
//!
//! "Medido, nunca inventado": cada indicador diz o período, a fonte e a conta feita, e fica
//! **sem dados** (sem pontos, fora da média) quando o histórico não basta, em vez de
//! penalizar. Fórmulas em `docs/health.md`.

pub mod api;
pub(crate) mod facts;
mod models;
#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::reports::Period;
use crate::reports::engine::normalize_month;
use crate::time::month_of;

pub use facts::MIN_MONTHS;

/// Ajuste guardado no cofre (`setting`).
pub const SETTINGS_KEY: &str = "health.settings";

/// Matéria de referência do I-SFB 2024.
pub const SOURCE_URL: &str = "https://meubolsoemdia.com.br/Materias/indice-saude-financeira-2024";

/// Média nacional do I-SFB em 2024.
pub const NATIONAL_AVERAGE: f64 = 56.7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum HealthModel {
    /// Estimativa no estilo do I-SFB (Febraban + Banco Central).
    #[default]
    Isf,
    /// Indicadores clássicos de finanças pessoais.
    Ratios,
    /// Análise desligada.
    Off,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthSettings {
    pub model: HealthModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthModelInfo {
    pub id: HealthModel,
    pub label: String,
    pub description: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthSettingsView {
    pub model: HealthModel,
    /// Modelos disponíveis, na ordem de exibição.
    pub models: Vec<HealthModelInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthScoreArgs {
    /// Mês avaliado `YYYY-MM` (padrão: o atual). No mês atual, a janela termina no mês
    /// anterior (o último completo); num mês passado, termina nele.
    #[serde(default)]
    #[ts(optional)]
    pub as_of: Option<String>,
    /// Membro da família (docs/members.md): a saúde financeira dele. Padrão: a família.
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<crate::ids::Id>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum HealthStatus {
    Good,
    Ok,
    Bad,
    /// Dados insuficientes: fica fora da nota.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum HealthUnit {
    /// Fração (0,153 = 15,3%).
    Percent,
    /// Meses (de gastos essenciais).
    Months,
    /// Fração de ocorrências (0–1), ex.: meses no azul ÷ meses analisados.
    Ratio,
    /// Contagem (meses, parcelas).
    Count,
}

/// Valor em dinheiro que entrou na conta de um indicador (a interface mostra com o modo
/// privacidade; os textos nunca trazem valores).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthAmount {
    pub label: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthMetric {
    pub key: String,
    pub label: String,
    /// Dimensão do I-SFB (`comportamento`, `habilidade`, `liberdade`, `seguranca`); `null`
    /// nos indicadores clássicos.
    pub dimension: Option<String>,
    /// Valor medido na unidade de `unit`; `null` sem dados.
    pub value: Option<f64>,
    pub unit: HealthUnit,
    /// Valor formatado no idioma da instalação (`15,3%`, `4,2 meses`, `11 de 12 meses`).
    pub value_text: Option<String>,
    /// Meta em texto (`≥ 20%`).
    pub target: String,
    pub target_value: Option<f64>,
    pub status: HealthStatus,
    /// Pontos de 0 a 100 (`null` sem dados).
    pub points: Option<f64>,
    /// Peso dentro da dimensão (I-SFB) ou da nota (clássicos), em fração.
    pub weight: f64,
    /// O que o indicador mede, numa linha.
    pub explanation: String,
    /// Como foi calculado: período, dados usados e a conta.
    pub computation: String,
    /// De 1 a 3 sugestões concretas quando o indicador não está bom.
    pub suggestions: Vec<String>,
    pub amounts: Vec<HealthAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthDimension {
    pub key: String,
    pub label: String,
    pub description: String,
    pub weight: f64,
    /// 0–100; `null` quando nenhum indicador da dimensão tem dados.
    pub score: Option<f64>,
    pub status: HealthStatus,
    /// Chaves dos indicadores da dimensão.
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthBand {
    pub key: String,
    pub label: String,
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthReference {
    pub label: String,
    /// O rótulo sem a fonte e o ano ("Média nacional"), para a régua da nota.
    pub short_label: String,
    pub score: f64,
    pub source_url: String,
}

/// Conta que entra na nota e não está em dia (`possibly_incomplete`): a nota sai estimada, e o
/// saldo negativo dela fica para conferir em vez de contar como cheque especial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StaleInput {
    pub account_id: crate::ids::Id,
    pub name: String,
    pub kind: crate::accounts::AccountKind,
    pub last_txn_date: Option<String>,
    #[ts(type = "number | null")]
    pub staleness_days: Option<i64>,
    /// O mês passado sem lançamento numa conta de todo mês.
    pub missed_month: Option<String>,
    /// Saldo negativo agora: pode ser lançamento faltando, não necessariamente cheque especial.
    pub negative: bool,
    #[ts(type = "number")]
    pub balance: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthReport {
    pub model: HealthModel,
    /// `false` com o modelo `off` (nota, dimensões e indicadores vazios).
    pub enabled: bool,
    /// Mês avaliado (`YYYY-MM`).
    pub as_of: String,
    /// Janela de meses completos analisada (`null` sem lançamentos).
    pub period: Option<Period>,
    pub months_analyzed: u32,
    /// Nota de 0 a 100 (uma casa); `null` com dados insuficientes.
    pub score: Option<f64>,
    pub band: Option<HealthBand>,
    /// Todas as faixas do modelo, da pior para a melhor.
    pub bands: Vec<HealthBand>,
    /// Média nacional (I-SFB).
    pub reference: Option<HealthReference>,
    /// Dimensões (I-SFB); vazio nos indicadores clássicos.
    pub dimensions: Vec<HealthDimension>,
    pub metrics: Vec<HealthMetric>,
    /// Uma frase com a nota e o ponto mais fraco.
    pub summary: String,
    /// Avisos (estimativa, dados insuficientes, câmbio).
    pub notes: Vec<String>,
    pub currency: String,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
    /// Contas desatualizadas que entram na nota (só na avaliação do mês atual).
    pub stale_inputs: Vec<StaleInput>,
}

// ---------------------------------------------------------------------------
// Ajustes

pub fn load_settings(conn: &Connection) -> Result<HealthSettings> {
    Ok(crate::settings::get(conn, SETTINGS_KEY)?.unwrap_or_default())
}

pub fn save_settings(conn: &Connection, s: &HealthSettings) -> Result<()> {
    crate::settings::set(conn, SETTINGS_KEY, s)
}

/// Os modelos, com nome e descrição no idioma da instalação.
pub fn models() -> Vec<HealthModelInfo> {
    vec![
        HealthModelInfo {
            id: HealthModel::Isf,
            label: t("nucleo-saude-modelo-isf"),
            description: t("nucleo-saude-modelo-isf-descricao"),
            source_url: Some(SOURCE_URL.into()),
        },
        HealthModelInfo {
            id: HealthModel::Ratios,
            label: t("nucleo-saude-modelo-classicos"),
            description: t("nucleo-saude-modelo-classicos-descricao"),
            source_url: None,
        },
        HealthModelInfo {
            id: HealthModel::Off,
            label: t("nucleo-saude-modelo-desativado"),
            description: t("nucleo-saude-modelo-desativado-descricao"),
            source_url: None,
        },
    ]
}

pub fn settings_view(conn: &Connection) -> Result<HealthSettingsView> {
    Ok(HealthSettingsView {
        model: load_settings(conn)?.model,
        models: models(),
    })
}

// ---------------------------------------------------------------------------
// Nota

/// Avaliação com o modelo dos ajustes, no mês pedido (padrão: o atual).
pub fn score(conn: &Connection, args: &HealthScoreArgs, today: NaiveDate) -> Result<HealthReport> {
    let model = load_settings(conn)?.model;
    score_with(conn, model, args, today)
}

pub fn score_with(
    conn: &Connection,
    model: HealthModel,
    args: &HealthScoreArgs,
    today: NaiveDate,
) -> Result<HealthReport> {
    let current = month_of(today);
    let as_of = match args
        .as_of
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        // Aceita `YYYY-MM-DD` (usa o mês).
        Some(m) => normalize_month(m.get(..7).unwrap_or(m))?,
        None => current.clone(),
    };
    if as_of > current {
        return Err(Error::invalid(t("nucleo-saude-erro-mes-futuro")));
    }
    let currency = crate::household::base_currency(conn)?;
    if model == HealthModel::Off {
        return Ok(HealthReport {
            model,
            enabled: false,
            as_of,
            period: None,
            months_analyzed: 0,
            score: None,
            band: None,
            bands: vec![],
            reference: None,
            dimensions: vec![],
            metrics: vec![],
            summary: t("nucleo-saude-desativada"),
            notes: vec![],
            currency,
            missing_fx: vec![],
            estimated: false,
            stale_inputs: vec![],
        });
    }
    let scope = args
        .member_id
        .as_deref()
        .filter(|m| !m.is_empty())
        .map(|m| crate::reports::member::MemberScope::load(conn, m))
        .transpose()?;
    let f = facts::Facts::load(conn, &as_of, today, scope.as_ref())?;
    Ok(match model {
        HealthModel::Isf => models::isf(&f),
        HealthModel::Ratios => models::ratios(&f),
        HealthModel::Off => unreachable!("tratado acima"),
    })
}
