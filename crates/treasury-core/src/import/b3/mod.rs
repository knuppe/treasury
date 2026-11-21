//! Importação da **Área do Investidor B3**: relatórios de negociação, movimentação e
//! posição, em Excel (.xlsx, todas as abas) ou CSV exportado da planilha.
//!
//! Fluxo, igual ao da importação de extratos:
//! `b3.detect` (tipo de relatório de cada aba) → `b3.preview` (lê os arquivos, propõe
//! ativos, operações, proventos e eventos, marca duplicados, simula tudo no motor da
//! carteira e confere com a posição da B3; guarda a prévia na sessão) → `b3.commit` (grava
//! num único conjunto de alterações com autor "importação"; desfazer = `changes.undo`).
//!
//! A prévia é uma **simulação de verdade**: as mesmas funções do commit rodam numa
//! transação que é desfeita no fim. Assim, vendas maiores que a posição, fatores de
//! desdobramento calculados pelas quantidades e a conferência com a posição mostram
//! exatamente o que o commit vai fazer.
//!
//! Regras de leitura em [`rules`]; leitura das planilhas em [`sheet`]; montagem do plano
//! em [`plan`]; simulação e gravação em [`apply`].
//!
//! Textos vindos dos arquivos (nomes de produtos, instituições) são **dados**: gravados e
//! comparados, nunca interpretados como instruções.

pub mod api;
mod apply;
mod engine;
mod parse;
mod plan;
pub mod rules;
pub mod sheet;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_api;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::assets::{AssetClass, FixedIncomeTerms};
use crate::ids::Id;
use crate::portfolio::{CorporateActionKind, IncomeKind, TradeSide};

pub use engine::{commit, detect, preview};

/// Formato do arquivo enviado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum B3FileFormat {
    Xlsx,
    Csv,
}

/// Tipo de relatório da Área do Investidor, reconhecido pelo cabeçalho.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum B3ReportKind {
    /// Negociação: compras e vendas na data do negócio.
    Trades,
    /// Movimentação: proventos, eventos, liquidações, transferências.
    Movements,
    /// Posição: quantidade por ativo e instituição numa data.
    Position,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3File {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3SheetInfo {
    /// Nome da aba (no CSV, o nome do arquivo).
    pub name: String,
    /// `null` = a aba não tem um relatório reconhecido.
    pub kind: Option<B3ReportKind>,
    /// Abas de posição: classe dos ativos da aba (Ações, BDR, ETF, FII, Renda Fixa, Tesouro).
    pub position_class: Option<AssetClass>,
    pub headers: Vec<String>,
    /// Linha do cabeçalho (1-based).
    #[ts(type = "number | null")]
    pub header_line: Option<u32>,
    #[ts(type = "number")]
    pub row_count: u32,
    /// Até 5 linhas, como texto.
    pub sample_rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3DetectResult {
    pub filename: String,
    pub format: B3FileFormat,
    /// CSV: codificação detectada.
    pub encoding: Option<String>,
    pub sheets: Vec<B3SheetInfo>,
    /// Relatórios encontrados no arquivo.
    pub kinds: Vec<B3ReportKind>,
    pub institutions: Vec<String>,
    pub period_from: Option<String>,
    pub period_to: Option<String>,
    pub warnings: Vec<String>,
}

/// Escolha da conta de corretora para uma instituição do arquivo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3AccountChoice {
    /// Nome da instituição como veio no arquivo (comparado sem maiúsculas e acentos).
    pub institution: String,
    /// `null` = não importar as linhas desta instituição.
    pub account_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3PreviewArgs {
    /// Um ou mais relatórios (negociação, movimentação, posição) do mesmo período.
    pub files: Vec<B3File>,
    /// Instituições sem escolha usam a conta sugerida pelo nome, quando há uma só.
    #[ts(optional)]
    pub accounts: Option<Vec<B3AccountChoice>>,
    /// Data da posição para a conferência (padrão: data no nome do arquivo de posição, ou hoje).
    #[ts(optional)]
    pub position_date: Option<String>,
}

/// De onde veio a linha (para a interface apontar o arquivo, a aba e a linha).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3Source {
    pub filename: String,
    pub sheet: String,
    #[ts(type = "number")]
    pub line: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum B3RowStatus {
    /// Será gravada.
    New,
    /// Já gravada antes (mesmo id externo ou mesma operação vinda do outro relatório): ignorada.
    Duplicate,
    /// Parecida com um lançamento feito à mão (mesma data, ativo, quantidade e preço/valor):
    /// não é gravada, a menos que o commit peça `include_possible_duplicates`.
    PossibleDuplicate,
    /// A instituição não tem conta escolhida: não é gravada.
    NoAccount,
    /// Precisa de aprovação (ex.: evento sem fator calculável): só é gravada se aprovada.
    Review,
    /// Não dá para gravar (ex.: venda maior que a posição).
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3Institution {
    pub name: String,
    pub account_id: Option<Id>,
    pub account_name: Option<String>,
    /// Conta sugerida pelo nome da instituição (`null` se nenhuma ou mais de uma).
    pub suggested_account_id: Option<Id>,
    /// A conta usada veio da sugestão (o usuário não escolheu).
    pub from_suggestion: bool,
    /// Linhas do arquivo desta instituição.
    #[ts(type = "number")]
    pub rows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3AssetProposal {
    /// Chave do ativo na prévia: o ticker, ou o nome do produto (Tesouro e renda fixa).
    pub key: String,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    /// Por que essa classe (aba da posição, sufixo do código, nome com "FII"…).
    pub class_reason: String,
    pub existing_asset_id: Option<Id>,
    /// Será cadastrado no commit.
    pub create: bool,
    /// Classe presumida ou dados incompletos (ex.: renda fixa sem taxa): confira.
    pub needs_review: bool,
    pub note: Option<String>,
    /// Renda fixa: termos lidos da posição (a taxa a B3 não informa).
    pub fixed_income: Option<FixedIncomeTerms>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum B3TradeOrigin {
    /// Relatório de negociação (data do negócio).
    Trades,
    /// Liquidação da movimentação ("Transferência - Liquidação"), usada sem o relatório de
    /// negociação; a data é estimada (2 dias úteis antes da liquidação).
    Settlement,
    /// Compra, venda, resgate ou vencimento de Tesouro/renda fixa na movimentação.
    Movement,
    /// Venda da fração no leilão de frações.
    FractionAuction,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3TradeProposal {
    /// Identifica a linha no commit (`skip`).
    #[ts(type = "number")]
    pub index: u32,
    pub source: B3Source,
    pub origin: B3TradeOrigin,
    pub date: String,
    pub asset_key: String,
    pub ticker: Option<String>,
    pub institution: String,
    pub account_id: Option<Id>,
    pub side: TradeSide,
    pub quantity: String,
    pub price: String,
    /// quantidade × preço, em centavos.
    #[ts(type = "number")]
    pub gross: i64,
    pub market: Option<String>,
    pub external_id: Option<String>,
    pub status: B3RowStatus,
    pub duplicate_of: Option<Id>,
    pub note: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3IncomeProposal {
    #[ts(type = "number")]
    pub index: u32,
    pub source: B3Source,
    pub pay_date: String,
    pub asset_key: String,
    pub ticker: Option<String>,
    pub institution: String,
    pub account_id: Option<Id>,
    pub kind: IncomeKind,
    /// Movimentação como veio no arquivo (ex.: "Juros Sobre Capital Próprio").
    pub movement: String,
    pub quantity: Option<String>,
    /// Bruto, imposto e líquido em centavos. JCP vem líquido da B3: bruto = líquido ÷ (1 − alíquota
    /// do JCP na data: 15% até 2025, 17,5% desde 2026).
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub tax_withheld: i64,
    #[ts(type = "number")]
    pub net: i64,
    pub external_id: Option<String>,
    pub status: B3RowStatus,
    pub duplicate_of: Option<Id>,
    pub note: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3ActionProposal {
    #[ts(type = "number")]
    pub index: u32,
    /// Linhas do arquivo (uma por instituição).
    pub sources: Vec<B3Source>,
    pub date: String,
    pub asset_key: String,
    pub ticker: Option<String>,
    pub kind: CorporateActionKind,
    pub movement: String,
    /// Quantidade da movimentação somada entre as instituições com conta (recebida no
    /// desdobro/bonificação; baixada ou resultante no grupamento).
    pub file_quantity: Option<String>,
    /// Posição antes do evento, pelo motor da carteira (contando esta importação).
    pub held_quantity: Option<String>,
    /// Nova quantidade = antiga × fator (bonificação: ações novas por ação).
    pub factor: Option<String>,
    /// Bonificação: custo atribuído por ação (preço unitário da movimentação).
    pub unit_cost: Option<String>,
    pub status: B3RowStatus,
    pub duplicate_of: Option<Id>,
    pub note: Option<String>,
    pub error: Option<String>,
}

/// Por que uma linha não vira lançamento. Cada regra está explicada em [`rules`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum B3IgnoreRule {
    Option,
    Forward,
    Future,
    UnsupportedMarket,
    Lending,
    CustodyTransfer,
    SettlementDuplicate,
    SubscriptionRight,
    BalanceUpdate,
    ManualEvent,
    ForwardedIncome,
    IncomeReversal,
    NoValue,
    Fraction,
    UnknownMovement,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3IgnoredRow {
    pub source: B3Source,
    pub report: B3ReportKind,
    pub rule: B3IgnoreRule,
    /// Resumo da linha (ex.: "Opção de Compra · PETRA300 · 100").
    pub description: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum B3PositionStatus {
    /// Mesma quantidade.
    Match,
    Divergent,
    /// A instituição não tem conta escolhida.
    NoAccount,
    /// O app tem posição que a B3 não lista nesta instituição.
    MissingInB3,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3PositionLine {
    pub asset_key: String,
    pub ticker: Option<String>,
    pub name: String,
    pub institution: Option<String>,
    pub account_id: Option<Id>,
    pub account_name: Option<String>,
    pub asset_id: Option<Id>,
    pub b3_quantity: Option<String>,
    /// Quantidade pelo motor da carteira na data, com esta importação.
    pub computed_quantity: Option<String>,
    /// calculada − B3.
    pub difference: Option<String>,
    pub close_price: Option<String>,
    #[ts(type = "number | null")]
    pub b3_value: Option<i64>,
    pub status: B3PositionStatus,
    pub source: Option<B3Source>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3PositionCheck {
    pub date: String,
    pub lines: Vec<B3PositionLine>,
    #[ts(type = "number")]
    pub matched: u32,
    /// Divergentes + ausentes na B3 (sem contar as instituições sem conta).
    #[ts(type = "number")]
    pub divergent: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3Counts {
    #[ts(type = "number")]
    pub new_assets: u32,
    /// Operações, proventos e eventos que serão gravados.
    #[ts(type = "number")]
    pub trades: u32,
    #[ts(type = "number")]
    pub incomes: u32,
    #[ts(type = "number")]
    pub actions: u32,
    #[ts(type = "number")]
    pub duplicates: u32,
    #[ts(type = "number")]
    pub possible_duplicates: u32,
    #[ts(type = "number")]
    pub no_account: u32,
    #[ts(type = "number")]
    pub review: u32,
    #[ts(type = "number")]
    pub errors: u32,
    #[ts(type = "number")]
    pub ignored: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3FileSummary {
    pub filename: String,
    pub sha256: String,
    pub format: B3FileFormat,
    pub kinds: Vec<B3ReportKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3Preview {
    /// Identifica a prévia em `b3.commit` (válida enquanto o cofre estiver aberto).
    pub token: String,
    pub files: Vec<B3FileSummary>,
    pub period_from: Option<String>,
    pub period_to: Option<String>,
    /// Sem relatório de negociação: as compras e vendas vieram das liquidações.
    pub uses_settlements: bool,
    pub institutions: Vec<B3Institution>,
    pub assets: Vec<B3AssetProposal>,
    pub trades: Vec<B3TradeProposal>,
    pub incomes: Vec<B3IncomeProposal>,
    pub actions: Vec<B3ActionProposal>,
    pub ignored: Vec<B3IgnoredRow>,
    /// Presente quando um relatório de posição veio junto.
    pub position_check: Option<B3PositionCheck>,
    pub counts: B3Counts,
    pub warnings: Vec<String>,
}

/// Aprova um evento corporativo em revisão (ou corrige o fator/custo de qualquer evento).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3ActionApproval {
    #[ts(type = "number")]
    pub index: u32,
    /// Obrigatório quando a prévia não conseguiu calcular o fator.
    #[ts(optional)]
    pub factor: Option<String>,
    #[ts(optional)]
    pub unit_cost: Option<String>,
}

/// Ajuste de um ativo proposto (pela `key` da prévia).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3AssetOverride {
    pub key: String,
    /// Usar um ativo já cadastrado em vez de criar (ex.: o ticker mudou).
    #[ts(optional)]
    pub asset_id: Option<Id>,
    #[ts(optional)]
    pub class: Option<AssetClass>,
    #[ts(optional)]
    pub name: Option<String>,
    /// Renda fixa: termos completos (com a taxa). Sem eles, o ativo é criado como "Outros".
    #[ts(optional)]
    pub fixed_income: Option<FixedIncomeTerms>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3CommitArgs {
    pub token: String,
    /// Índices (de operações, proventos ou eventos) a não gravar.
    #[ts(optional, type = "number[]")]
    pub skip: Option<Vec<u32>>,
    /// Gravar também os possíveis duplicados (padrão: não).
    #[ts(optional)]
    pub include_possible_duplicates: Option<bool>,
    #[ts(optional)]
    pub approve_actions: Option<Vec<B3ActionApproval>>,
    #[ts(optional)]
    pub assets: Option<Vec<B3AssetOverride>>,
    /// Lançar compras, vendas e proventos no caixa da corretora. Padrão: não, porque a B3
    /// não informa o caixa (nem as taxas) da corretora.
    #[ts(optional)]
    pub cash_movement: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3CommitError {
    #[ts(type = "number")]
    pub index: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct B3CommitResult {
    /// Desfazer a importação = `changes.undo` deste conjunto.
    pub change_set_id: Option<Id>,
    #[ts(type = "number")]
    pub assets_created: u32,
    #[ts(type = "number")]
    pub trades: u32,
    #[ts(type = "number")]
    pub incomes: u32,
    #[ts(type = "number")]
    pub actions: u32,
    /// Linhas propostas que não foram gravadas (duplicadas, sem conta, puladas, com erro).
    #[ts(type = "number")]
    pub skipped: u32,
    pub errors: Vec<B3CommitError>,
    /// Backup feito antes de uma importação grande.
    pub backup_id: Option<Id>,
}
