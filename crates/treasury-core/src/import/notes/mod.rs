//! Importação de **notas de corretagem** em PDF (padrão SINACOR).
//!
//! `notes.preview` extrai o texto (com a senha, quando o PDF é protegido; muitas corretoras
//! usam os 3 primeiros dígitos do CPF), lê as notas, identifica os ativos, rateia taxas e
//! IRRF entre as operações e compara com o que já está gravado: operação nova, operação já
//! importada (ex.: pelo relatório da B3) sem custos, que só ganha os custos e o número da
//! nota, ou duplicada. Nada é gravado.
//!
//! `notes.commit` refaz a mesma leitura (sem estado na sessão) e grava tudo num único
//! conjunto de alterações com autor "importação"; desfazer = `changes.undo`.
//!
//! A senha do PDF não é guardada. O texto das notas é **dado**, nunca instrução.

pub mod api;
mod engine;
pub mod parse;

#[cfg(test)]
pub(crate) mod tests;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::assets::AssetClass;
use crate::ids::Id;
use crate::portfolio::TradeSide;

pub use engine::{TRADING_NAMES_SETTING, commit, preview, store_trading_names};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteFile {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NotePreviewArgs {
    pub files: Vec<NoteFile>,
    /// Conta da corretora das notas.
    pub account_id: Id,
    /// Senha dos PDFs protegidos (não é guardada).
    #[serde(default)]
    #[ts(optional)]
    pub password: Option<String>,
    /// Correções de ativo já feitas na tela (a prévia mostra o resultado delas).
    #[serde(default)]
    #[ts(optional)]
    pub assets: Option<Vec<NoteAssetChoice>>,
    /// Somar as execuções da mesma ordem (mesmo título, lado e mercado na nota) numa
    /// operação só, com preço médio (padrão: sim).
    #[serde(default)]
    #[ts(optional)]
    pub group: Option<bool>,
}

/// Ativo escolhido pelo usuário para uma especificação (não reconhecida ou corrigida):
/// um ativo cadastrado (`asset_id`) ou um código a cadastrar (`ticker`, com a classe).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteAssetChoice {
    pub spec: String,
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub class: Option<AssetClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteCommitArgs {
    pub files: Vec<NoteFile>,
    pub account_id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub password: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub assets: Option<Vec<NoteAssetChoice>>,
    /// Índices da prévia que não devem ser gravados.
    #[serde(default)]
    #[ts(optional)]
    pub skip: Option<Vec<u32>>,
    /// Lançar a movimentação no caixa da corretora (padrão: não, porque o extrato da conta
    /// costuma ser importado à parte).
    #[serde(default)]
    #[ts(optional)]
    pub cash_movement: Option<bool>,
    /// Igual ao da prévia: os índices de `skip` valem para a mesma escolha.
    #[serde(default)]
    #[ts(optional)]
    pub group: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum NoteTradeStatus {
    /// Operação nova: será cadastrada com os custos rateados.
    New,
    /// Já existe a mesma operação sem custos (ex.: vinda do relatório da B3): ganha os
    /// custos, o IRRF e o número da nota.
    FillCosts,
    /// Já registrada com esta nota ou com custos: ignorada.
    Duplicate,
    /// Ativo não identificado: escolha o ativo para gravar.
    Unresolved,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteTrade {
    #[ts(type = "number")]
    pub index: u32,
    pub filename: String,
    pub note_number: Option<String>,
    pub date: String,
    pub side: TradeSide,
    pub market: String,
    /// Especificação do título como veio na nota.
    pub spec: String,
    pub ticker: Option<String>,
    pub asset_id: Option<Id>,
    /// Ativo a cadastrar no commit (ticker reconhecido, ainda não cadastrado).
    pub create_asset: Option<NoteNewAsset>,
    pub quantity: String,
    pub price: String,
    /// Valor da operação, em centavos.
    #[ts(type = "number")]
    pub gross: i64,
    /// Parte das taxas da nota (liquidação, registro, emolumentos, corretagem, ISS…).
    #[ts(type = "number")]
    pub fees: i64,
    /// Parte do IRRF (só vendas).
    #[ts(type = "number")]
    pub taxes: i64,
    pub day_trade: bool,
    /// Execuções da nota somadas nesta operação (1 = sem agrupamento).
    #[ts(type = "number")]
    pub executions: u32,
    /// O ativo veio de uma escolha do usuário (não do reconhecimento automático).
    pub chosen: bool,
    pub status: NoteTradeStatus,
    /// Inconsistência que não impede gravar (ex.: venda maior que a posição na data).
    pub warning: Option<String>,
    pub existing_trade_id: Option<Id>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteNewAsset {
    pub ticker: String,
    pub name: String,
    pub class: AssetClass,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteCosts {
    #[ts(type = "number")]
    pub settlement: i64,
    #[ts(type = "number")]
    pub registration: i64,
    #[ts(type = "number")]
    pub emoluments: i64,
    #[ts(type = "number")]
    pub brokerage: i64,
    #[ts(type = "number")]
    pub iss: i64,
    #[ts(type = "number")]
    pub other: i64,
    #[ts(type = "number")]
    pub irrf: i64,
    /// Soma das taxas (sem o IRRF).
    #[ts(type = "number")]
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteSummary {
    pub filename: String,
    pub number: Option<String>,
    pub date: Option<String>,
    #[ts(type = "number")]
    pub trades: u32,
    pub costs: NoteCosts,
    /// Líquido da nota (positivo = crédito), em centavos.
    #[ts(type = "number | null")]
    pub net: Option<i64>,
    /// Líquido recalculado pelas operações e custos lidos; difere do `net` quando alguma
    /// linha não foi lida.
    #[ts(type = "number")]
    pub computed_net: i64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteFileError {
    pub filename: String,
    pub message: String,
    /// O PDF é protegido e a senha não foi informada ou está errada.
    pub needs_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NotePreview {
    pub notes: Vec<NoteSummary>,
    pub trades: Vec<NoteTrade>,
    /// Especificações sem ativo: escolha em `assets` no commit.
    pub unresolved_specs: Vec<String>,
    pub errors: Vec<NoteFileError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteCommitResult {
    #[ts(type = "number")]
    pub created: u32,
    #[ts(type = "number")]
    pub updated: u32,
    #[ts(type = "number")]
    pub assets_created: u32,
    #[ts(type = "number")]
    pub skipped: u32,
    pub change_set_id: Option<Id>,
}
