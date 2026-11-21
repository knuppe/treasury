//! Importação de extratos e faturas: OFX (SGML 1.x e XML 2.x) e CSV com mapeamento de
//! colunas, deduplicação por `FITID` e por impressão digital, regras de categorização e
//! prévia antes de gravar.
//!
//! Fluxo: `detect` (amostra e preset sugerido) → `preview` (lê, deduplica, aplica regras e
//! guarda a prévia na sessão) → `commit` (grava tudo num único conjunto de alterações com
//! autor "importação"; desfazer a importação = `changes.undo` desse conjunto).
//!
//! Textos vindos dos arquivos (descrições, nomes, memos) são **dados**: são gravados e
//! comparados, nunca interpretados como instruções.

pub mod api;
pub mod b3;
pub mod caixa;
pub mod card_pdf;
pub mod csv_file;
mod engine;
pub mod image;
pub mod inbox;
pub mod norma43;
pub mod notes;
pub mod ofx;
pub mod sniff;
pub mod text;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) mod testutil;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::Id;
use crate::transactions::TxnKind;

/// SHA-256 em hexadecimal (o mesmo da importação de extratos).
pub(crate) fn engine_sha256(bytes: &[u8]) -> String {
    engine::sha256_hex(bytes)
}

pub(crate) use engine::is_bill_payment;
#[cfg(test)]
pub(crate) use engine::statement_text;
pub use engine::{
    BACKUP_THRESHOLD, card_pdf_preview, commit, detect, list_batches, preview, preview_rows,
};

/// Máximo de lançamentos por chamada de `import.rows` (o agente divide extratos longos).
pub const MAX_AGENT_ROWS: usize = 500;

/// Lançamento lido de uma página pelo agente.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RowInput {
    /// `YYYY-MM-DD`.
    pub date: String,
    /// Centavos, com sinal (negativo = saída).
    #[ts(type = "number")]
    pub amount: i64,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub payee: Option<String>,
    /// Identificador do lançamento no banco, se a página mostrar (evita duplicar).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub notes: Option<String>,
    /// Cartão: vencimento (`YYYY-MM-DD`) da fatura em que o emissor pôs a compra; vale mais que
    /// o do lote e que o calculado pelo dia de fechamento.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub statement_due_date: Option<String>,
    /// Cartão: final (4 últimos dígitos) do cartão adicional ou virtual da compra; o membro do
    /// lançamento vem do cartão com esse final.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub card_last4: Option<String>,
    /// Compra no exterior: valor positivo na moeda de origem e a moeda (`USD`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub orig_amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub orig_currency: Option<String>,
}

/// `import.rows`: importa lançamentos estruturados (prévia e gravação numa chamada).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RowsImportArgs {
    pub account_id: Id,
    /// De onde vieram (ex.: "extrato Itaú setembro/2026, internet banking").
    pub source: String,
    pub rows: Vec<RowInput>,
    /// Gravar também os possíveis duplicados (padrão: não).
    #[serde(default)]
    #[ts(optional)]
    pub include_possible_duplicates: Option<bool>,
    /// Cartão: vencimento (`YYYY-MM-DD`) da fatura de todo o lote (a tela da fatura).
    #[serde(default)]
    #[ts(optional)]
    pub statement_due_date: Option<String>,
    /// Cartão: final do cartão de todo o lote (vale quando a linha não traz o dela).
    #[serde(default)]
    #[ts(optional)]
    pub card_last4: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ImportFormat {
    Ofx,
    Csv,
    /// Fatura de cartão em PDF (docs/card-statement-pdf.md).
    Pdf,
}

/// Mapeamento de colunas do CSV (índice 0-based ou nome do cabeçalho).
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct CsvMapping {
    /// `,`, `;` ou `\t`. Padrão: detectar.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub delimiter: Option<String>,
    pub has_header: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub skip_rows: Option<u32>,
    pub date_column: String,
    /// Formato strftime, ex.: `%d/%m/%Y`.
    pub date_format: String,
    /// Coluna única de valor com sinal…
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub amount_column: Option<String>,
    /// …ou colunas separadas de débito e crédito.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub debit_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub credit_column: Option<String>,
    pub description_column: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub payee_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub external_id_column: Option<String>,
    /// Vírgula decimal (`1.234,56`).
    pub decimal_comma: bool,
    /// Inverter o sinal (faturas que mostram compras como positivas).
    pub invert_sign: bool,
    /// `utf-8`, `windows-1252`, `iso-8859-1`. Padrão: detectar.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub encoding: Option<String>,
    /// Descrição é o descritor da maquininha (`IFD*LOJA CIDADE BRA`): o favorecido sugerido
    /// sai sem o prefixo do adquirente nem o país no fim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub card_descriptor: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CsvPreset {
    pub id: String,
    pub name: String,
    pub mapping: CsvMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportPreviewArgs {
    pub account_id: Id,
    pub filename: String,
    pub content_base64: String,
    /// Padrão: detectar pelo conteúdo.
    #[ts(optional)]
    pub format: Option<ImportFormat>,
    /// Mapeamento explícito (vence o preset).
    #[ts(optional)]
    pub csv_mapping: Option<CsvMapping>,
    /// Preset de CSV; sem preset nem mapeamento, usa o sugerido pelo cabeçalho.
    #[ts(optional)]
    pub preset_id: Option<String>,
    /// Codificação forçada (`utf-8`, `windows-1252`, `iso-8859-1`). Padrão: detectar.
    #[ts(optional)]
    pub encoding: Option<String>,
    /// OFX com vários extratos: `ACCTID` do extrato a importar (padrão: o primeiro do
    /// mesmo tipo da conta).
    #[ts(optional)]
    pub ofx_account: Option<String>,
    /// Senha do PDF protegido (fatura de cartão); não é guardada.
    #[serde(default)]
    #[ts(optional)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DetectArgs {
    pub filename: String,
    pub content_base64: String,
    #[ts(optional)]
    pub encoding: Option<String>,
    /// País da família (o núcleo preenche): decide datas e vírgula decimal quando o CSV não
    /// deixa claro.
    #[serde(default)]
    #[ts(skip)]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ListBatchesArgs {
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RowStatus {
    New,
    /// Mesmo `FITID`/id externo já importado: ignorado.
    Duplicate,
    /// Mesma data, valor e descrição de um lançamento existente: revisar.
    PossibleDuplicate,
    Error,
    /// Não é lançamento (linha de saldo, valor zero): não é gravada.
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PreviewRow {
    #[ts(type = "number")]
    pub index: u32,
    /// Linha no arquivo (CSV), para mensagens.
    #[ts(type = "number | null")]
    pub line: Option<u32>,
    pub date: Option<String>,
    /// Com sinal relativo à conta (compras no cartão negativas).
    #[ts(type = "number | null")]
    pub amount: Option<i64>,
    /// Descrição como veio do arquivo.
    pub description: String,
    pub payee: Option<String>,
    pub external_id: Option<String>,
    pub status: RowStatus,
    pub duplicate_of: Option<Id>,
    /// Tipo inferido pelo sinal (ou pagamento de fatura), depois das regras.
    pub kind: Option<TxnKind>,
    pub suggested_category_id: Option<Id>,
    pub suggested_member_id: Option<Id>,
    pub suggested_tags: Vec<String>,
    /// Nova descrição pedida por uma regra.
    pub suggested_description: Option<String>,
    pub rule_id: Option<Id>,
    pub error: Option<String>,
    /// Observação (motivo de duplicado/ignorado, ajuste de sinal).
    pub note: Option<String>,
    /// Cartão: final do cartão da linha e o plástico cadastrado com ele (`null` se não houver).
    #[serde(default)]
    pub card_last4: Option<String>,
    #[serde(default)]
    pub card_plastic_id: Option<Id>,
    /// Compra no exterior lida da linha (valor positivo e moeda).
    #[serde(default)]
    #[ts(type = "number | null")]
    pub orig_amount: Option<i64>,
    #[serde(default)]
    pub orig_currency: Option<String>,
    /// Lançamento previsto (recorrência, conta a pagar) que esta linha efetiva ao importar.
    #[serde(default)]
    pub planned_id: Option<Id>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PreviewCounts {
    #[ts(type = "number")]
    pub new: u32,
    #[ts(type = "number")]
    pub duplicate: u32,
    #[ts(type = "number")]
    pub possible_duplicate: u32,
    #[ts(type = "number")]
    pub error: u32,
    #[ts(type = "number")]
    pub ignored: u32,
}

/// Dados da conta lidos do OFX, para conferir com a conta escolhida.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OfxAccountInfo {
    pub bank_id: Option<String>,
    pub branch_id: Option<String>,
    pub account_id: Option<String>,
    pub account_type: Option<String>,
    /// Extrato de cartão de crédito (`CCSTMTRS`).
    pub credit_card: bool,
    pub currency: Option<String>,
    #[ts(type = "number | null")]
    pub ledger_balance: Option<i64>,
    pub balance_date: Option<String>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    /// Quantos extratos o arquivo tem (alguns bancos juntam conta e poupança).
    #[ts(type = "number")]
    pub statements: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportPreview {
    /// Identifica a prévia em `import.commit` (válida enquanto o cofre estiver aberto).
    pub token: String,
    pub format: ImportFormat,
    pub encoding: String,
    pub account_id: Id,
    pub filename: String,
    pub file_sha256: String,
    /// Este mesmo arquivo já foi importado antes.
    pub already_imported: bool,
    pub already_imported_batch_id: Option<Id>,
    /// Moeda da conta (valores na menor unidade dela).
    pub currency: String,
    pub rows: Vec<PreviewRow>,
    pub counts: PreviewCounts,
    pub ofx: Option<OfxAccountInfo>,
    /// CSV: mapeamento efetivamente usado (preset ou explícito, com delimitador detectado).
    pub csv_mapping: Option<CsvMapping>,
    pub preset_id: Option<String>,
    /// Avisos para o usuário (moeda diferente, sinais invertidos, FITID repetido…).
    pub warnings: Vec<String>,
    /// Fatura de cartão em PDF: layout, vencimento e conferência do total.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub card_pdf: Option<CardPdfInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardPdfArgs {
    pub filename: String,
    pub content_base64: String,
    #[serde(default)]
    #[ts(optional)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardPdfRowView {
    pub date: String,
    pub description: String,
    /// Compra negativa; estorno ou crédito positivo.
    #[ts(type = "number")]
    pub amount: i64,
}

/// Fatura em PDF lida, sem gravar (`import.card_pdf_preview`).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardPdfPreview {
    pub info: CardPdfInfo,
    pub rows: Vec<CardPdfRowView>,
    pub warnings: Vec<String>,
}

/// O que a fatura em PDF diz de si mesma.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardPdfInfo {
    pub layout: crate::layouts::LayoutInfo,
    /// Banco reconhecido pelo nome no PDF.
    pub issuer: Option<String>,
    pub due_date: Option<String>,
    /// Mês de vencimento: os lançamentos vão para esta fatura do cartão.
    pub reference_month: Option<String>,
    /// Total impresso (positivo = a pagar).
    #[ts(type = "number | null")]
    pub total: Option<i64>,
    /// Compras menos créditos lidos.
    #[ts(type = "number")]
    pub rows_total: i64,
    /// O total impresso bate com as linhas lidas.
    pub total_matches: Option<bool>,
    /// Pagamentos da fatura anterior que ficaram de fora.
    pub payments_skipped: u32,
    /// Pagamento mínimo, limite total e saldo da fatura anterior impressos.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub minimum: Option<i64>,
    #[serde(default)]
    #[ts(type = "number | null")]
    pub credit_limit: Option<i64>,
    #[serde(default)]
    #[ts(type = "number | null")]
    pub previous_balance: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RowOverride {
    #[ts(type = "number")]
    pub index: u32,
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[ts(optional)]
    pub description: Option<String>,
    #[ts(optional)]
    pub kind: Option<TxnKind>,
    /// Substitui as tags sugeridas.
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportCommitArgs {
    pub token: String,
    /// Linhas a ignorar.
    #[ts(optional, type = "number[]")]
    pub skip_indexes: Option<Vec<u32>>,
    /// Importar também os possíveis duplicados (padrão: não).
    #[ts(optional)]
    pub include_possible_duplicates: Option<bool>,
    #[ts(optional)]
    pub overrides: Option<Vec<RowOverride>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportResult {
    pub batch_id: Id,
    #[ts(type = "number")]
    pub inserted: u32,
    #[ts(type = "number")]
    pub skipped: u32,
    /// Parcelas de contratos marcadas como pagas por lançamentos na categoria ligada.
    #[ts(type = "number")]
    pub installments_paid: u32,
    /// Pares ligados sozinhos como transferência (contas marcadas como "sempre ligar").
    #[ts(type = "number")]
    pub transfers_linked: u32,
    /// Previstos (recorrências, contas a pagar) efetivados pelos lançamentos importados.
    #[serde(default)]
    #[ts(type = "number")]
    pub planned_settled: u32,
    /// Desfazer a importação = `changes.undo` deste conjunto.
    pub change_set_id: Option<Id>,
    /// Backup feito antes de uma importação grande.
    pub backup_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportBatch {
    pub id: Id,
    pub filename: String,
    pub format: ImportFormat,
    pub file_sha256: String,
    pub account_id: Id,
    pub account_name: String,
    #[ts(type = "number")]
    pub total_rows: i64,
    #[ts(type = "number")]
    pub inserted: i64,
    #[ts(type = "number")]
    pub duplicates: i64,
    pub errors: Vec<ImportRowError>,
    pub created_at: String,
    /// Conjunto de alterações da importação (desfazer = `changes.undo`).
    pub change_set_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportRowError {
    #[ts(type = "number")]
    pub index: u32,
    #[ts(type = "number | null")]
    pub line: Option<u32>,
    pub message: String,
}

/// Amostra do arquivo para montar o mapeamento de colunas na interface.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DetectResult {
    pub format: ImportFormat,
    pub encoding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub delimiter: Option<String>,
    /// CSV: linhas antes da tabela (preâmbulo do banco).
    #[ts(type = "number")]
    pub skip_rows: u32,
    pub has_header: bool,
    pub headers: Vec<String>,
    /// Até 10 linhas. OFX: data, valor, descrição, FITID.
    pub sample_rows: Vec<Vec<String>>,
    #[ts(type = "number")]
    pub row_count: u32,
    /// Preset reconhecido pelo cabeçalho (ex.: Nubank).
    pub suggested_preset_id: Option<String>,
    /// Mapeamento do preset sugerido, já com delimitador e linhas a pular detectados.
    pub suggested_mapping: Option<CsvMapping>,
    pub ofx: Option<OfxAccountInfo>,
    /// PDF: o extrato de conta reconhecido (o extrato por período da Caixa), para a interface
    /// não tratar o arquivo como fatura de cartão.
    #[serde(default)]
    #[ts(optional)]
    pub layout: Option<crate::layouts::LayoutInfo>,
}
