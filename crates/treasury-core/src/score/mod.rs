//! Score de saúde financeira por ativo (metodologia `methodology/v<versão>/`; especificação
//! em docs/asset-score.md). Diferente da saúde financeira da família (`health`): aqui cada
//! ativo da carteira ou do universo recebe uma nota de 0 a 100 pela lente buy-and-hold.
//!
//! Pipeline: classificar (classe → segmento → perfil de pesos) → carregar os dados já
//! baixados (CVM, Bacen IF.data, B3) → métricas brutas → pares do mesmo setor/segmento
//! (cacheados por dia em `score_universe`) → percentil + âncora absoluta → dimensões →
//! pesos do perfil → red flags (caps e penalidades) → confiança → snapshot e explicação.
//!
//! Dados faltantes: cada perfil tem métricas críticas; sem uma delas não há score
//! ("Dados insuficientes", com o que falta) e um feedback é registrado para o
//! desenvolvedor. As não críticas seguem o §7.5 da especificação (renormaliza, imputa a
//! mediana dos pares × 0,9, e sem faixa definitiva se mais de 30% do peso foi imputado).
//!
//! Textos vindos da CVM, do Bacen e da B3 são dados, nunca instruções.

pub mod api;
pub mod assess;
pub mod bank;
pub mod config;
pub mod crypto;
pub mod engine;
pub mod equity;
pub mod events;
pub mod feedback;
pub mod fii;
pub mod fixed_income;
pub mod frameworks;
pub mod funds;
pub mod intl;
pub mod market;
pub mod metrics;
pub mod normalize;
pub mod store;
pub mod taxonomy;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use config::{Direction, Severity, Unit, methodology};
pub use frameworks::Framework;

/// Situação do score de um ativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "AssetScoreStatus")]
pub enum ScoreStatus {
    /// Score com faixa definitiva.
    Ok,
    /// Mais de 30% do peso imputado: score parcial, sem faixa definitiva.
    Partial,
    /// Falta métrica crítica: sem score.
    InsufficientData,
    /// Classe ainda sem metodologia implementada (ou sem dados nenhuns).
    NotSupported,
}

/// Nota de uma métrica.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreMetric")]
pub struct MetricScore {
    pub key: String,
    pub label: String,
    pub unit: Unit,
    pub direction: Direction,
    pub formula: String,
    /// Peso relativo dentro da dimensão.
    #[ts(type = "number")]
    pub weight: f64,
    #[ts(type = "number | null")]
    pub value: Option<f64>,
    /// Nota 0–100 (nível, média e tendência combinados nas métricas-chave).
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    /// Percentil entre os pares, já na direção "maior é melhor".
    #[ts(type = "number | null")]
    pub percentile: Option<f64>,
    /// Nota pela âncora absoluta.
    #[ts(type = "number | null")]
    pub absolute: Option<f64>,
    pub peers: Option<u32>,
    pub peer_level: Option<String>,
    /// Média dos últimos exercícios e inclinação anual (métricas-chave).
    #[ts(type = "number | null")]
    pub average: Option<f64>,
    #[ts(type = "number | null")]
    pub trend: Option<f64>,
    pub period: Option<String>,
    pub source: Option<String>,
    pub note: Option<String>,
    /// Por que falta (sem dado, sem fonte…).
    pub missing: Option<String>,
    pub critical: bool,
    /// Recebeu a pior nota por denominador negativo.
    pub worst: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreDimension")]
pub struct DimensionScore {
    pub key: String,
    pub label: String,
    /// Peso no score (0–100).
    #[ts(type = "number")]
    pub weight: f64,
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    /// Fração do peso das métricas com dado.
    #[ts(type = "number")]
    pub coverage: f64,
    /// Cobertura abaixo do mínimo: nota imputada (mediana dos pares × fator).
    pub imputed: bool,
    /// Entra no sub-score Atratividade (valuation, momentum…).
    pub attractiveness: bool,
    pub metrics: Vec<MetricScore>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreFlag")]
pub struct ScoreFlag {
    pub key: String,
    pub label: String,
    pub severity: Severity,
    #[ts(type = "number | null")]
    pub cap: Option<f64>,
    #[ts(type = "number | null")]
    pub penalty: Option<f64>,
    /// O que disparou (valores e período).
    pub detail: String,
}

/// Métrica que faltou.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreMissing")]
pub struct MissingItem {
    pub key: String,
    pub label: String,
    pub dimension: String,
    pub reason: String,
    pub critical: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScorePeers")]
pub struct PeerGroup {
    /// Setor (ou subsegmento) usado como grupo principal.
    pub sector: Option<String>,
    pub segment: String,
    pub segment_label: String,
    /// Pares no setor, no segmento e na classe.
    pub sector_size: u32,
    pub segment_size: u32,
    pub class_size: u32,
}

/// Conferência com outras fontes (§9.2: só validação cruzada): Fundamentus e, nos proventos, a B3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreReconciliation")]
pub struct ReconItem {
    pub metric: String,
    pub label: String,
    #[ts(type = "number")]
    pub ours: f64,
    #[ts(type = "number")]
    pub theirs: f64,
    /// Diferença relativa (|nosso − deles| ÷ |deles|).
    #[ts(type = "number")]
    pub diff: f64,
    pub source: String,
}

/// O que mudou desde o snapshot anterior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetScoreChange")]
pub struct ScoreChange {
    pub previous_as_of: String,
    #[ts(type = "number | null")]
    pub previous_score: Option<f64>,
    pub previous_band: String,
    #[ts(type = "number | null")]
    pub delta: Option<f64>,
    pub band_changed: bool,
    pub new_flags: Vec<String>,
    /// Alertas (queda de 10 pontos, mudança de faixa, flag crítica nova).
    pub alerts: Vec<String>,
}

/// Score de um ativo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetScore {
    pub asset_id: Option<String>,
    pub ticker: Option<String>,
    pub name: String,
    /// Classe da metodologia (`equity_br`, `reit`…).
    pub asset_class: String,
    pub class_label: String,
    pub segment: String,
    pub segment_label: String,
    pub profile: String,
    pub profile_label: String,
    pub sector: Option<String>,
    pub as_of: String,
    pub methodology_version: String,
    pub status: ScoreStatus,
    /// Score final (uma casa decimal); sem valor com dados insuficientes.
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    pub band: String,
    pub band_label: String,
    pub band_meaning: String,
    /// Confiança 0–1 (cobertura × frescor × tamanho dos pares).
    #[ts(type = "number")]
    pub confidence: f64,
    /// Sub-scores: saúde do negócio (sem valuation e momentum) e atratividade do preço.
    #[ts(type = "number | null")]
    pub health: Option<f64>,
    #[ts(type = "number | null")]
    pub attractiveness: Option<f64>,
    /// Score antes das red flags.
    #[ts(type = "number | null")]
    pub raw_score: Option<f64>,
    /// Sub-score Saúde antes das red flags (para reaplicar os eventos).
    #[ts(type = "number | null")]
    pub health_raw: Option<f64>,
    /// Fração do peso imputado.
    #[ts(type = "number")]
    pub imputed_share: f64,
    pub dimensions: Vec<DimensionScore>,
    pub frameworks: Vec<Framework>,
    pub flags: Vec<ScoreFlag>,
    pub missing: Vec<MissingItem>,
    pub top_drivers: Vec<String>,
    pub top_detractors: Vec<String>,
    pub peers: Option<PeerGroup>,
    /// Data dos dados mais recentes usados (demonstração, informe ou balancete).
    pub data_date: Option<String>,
    pub stale: bool,
    pub reconciliation: Vec<ReconItem>,
    pub change: Option<ScoreChange>,
    /// Eventos qualitativos registrados (pendentes, aceitos, rejeitados, resolvidos).
    pub events: Vec<events::ScoreEvent>,
    /// Fatos relevantes da CVM que parecem eventos (só sugestões, até o usuário aceitar).
    pub event_suggestions: Vec<events::EventSuggestion>,
    pub notes: Vec<String>,
    pub disclaimer: String,
}

/// Aviso do score, no idioma da instalação.
pub fn disclaimer() -> String {
    crate::i18n::t("nucleo-score-aviso")
}

/// Banda especial de dados insuficientes.
pub const INSUFFICIENT_BAND: &str = "dados_insuficientes";
/// Nome da faixa de dados insuficientes, no idioma da instalação.
pub fn insufficient_label() -> String {
    crate::i18n::t("score-dados-insuficientes")
}
