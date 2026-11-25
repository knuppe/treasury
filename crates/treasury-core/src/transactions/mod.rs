//! Lançamentos: receitas, despesas, transferências, pagamento de fatura, aportes,
//! com divisão em categorias/titulares, tags e anexos.
//!
//! Convenção de sinal: `amount` é relativo à conta (+ entra, − sai). Uma compra no cartão
//! é negativa na conta do cartão. Transferências têm duas pernas com o mesmo `transfer_id`.
//! Transferências, pagamento de fatura e aportes **não** contam como receita nem despesa.

pub mod api;
pub mod b64;
mod bulk;
pub mod duplicates;
mod extras;
pub mod operations;
pub(crate) mod query;
pub mod suggest;
pub mod transfers;
pub(crate) mod util;
mod write;

#[cfg(test)]
pub(crate) mod testkit;
#[cfg(test)]
mod tests_domain;
#[cfg(test)]
mod tests_member;

pub use bulk::{EditTagsArgs, SameDescription, SameDescriptionArgs, edit_tags, same_description};
pub use extras::{
    AttachmentContent, AttachmentInput, MAX_ATTACHMENT_BYTES, TagInput, TagPatch, TagSummary,
    add_attachment, attachment_meta, create_tag, delete_attachment, delete_tag, get_attachment,
    get_tag, list_attachments, list_tags, update_tag,
};
pub use query::{get, query, query_ids, search_fold, views};
pub use write::{
    TransferIds, create, create_transfer, create_transfer_with, delete, kind_label, set_category,
    set_member, set_status, status_label, update,
};
pub(crate) use write::{TxnRow, check_category, delete_rows, load_row};

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TxnKind {
    Income,
    Expense,
    Transfer,
    CardPayment,
    Investment,
    Adjustment,
}

impl TxnKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TxnKind::Income => "income",
            TxnKind::Expense => "expense",
            TxnKind::Transfer => "transfer",
            TxnKind::CardPayment => "card_payment",
            TxnKind::Investment => "investment",
            TxnKind::Adjustment => "adjustment",
        }
    }
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "income" => TxnKind::Income,
            "expense" => TxnKind::Expense,
            "transfer" => TxnKind::Transfer,
            "card_payment" => TxnKind::CardPayment,
            "investment" => TxnKind::Investment,
            "adjustment" => TxnKind::Adjustment,
            _ => {
                return Err(Error::invalid(t_args(
                    "nucleo-lancamentos-erro-tipo-invalido",
                    &args(&[("valor", s.to_string().into())]),
                )));
            }
        })
    }
    /// Conta no fluxo de caixa (receita/despesa)?
    pub fn is_cash_flow(self) -> bool {
        matches!(self, TxnKind::Income | TxnKind::Expense)
    }
    /// Movimentação entre contas (duas pernas, ou uma só quando o outro lado é de fora
    /// da família ou não foi lançado).
    pub fn is_transfer_like(self) -> bool {
        matches!(
            self,
            TxnKind::Transfer | TxnKind::CardPayment | TxnKind::Investment
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TxnStatus {
    /// Previsto (recorrência, parcela futura, agendado). Não entra no saldo nem no realizado.
    Planned,
    /// Efetivado.
    Cleared,
    /// Conferido com o extrato.
    Reconciled,
}

impl TxnStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            TxnStatus::Planned => "planned",
            TxnStatus::Cleared => "cleared",
            TxnStatus::Reconciled => "reconciled",
        }
    }
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "planned" => TxnStatus::Planned,
            "cleared" => TxnStatus::Cleared,
            "reconciled" => TxnStatus::Reconciled,
            _ => {
                return Err(Error::invalid(t_args(
                    "nucleo-lancamentos-erro-status-invalido",
                    &args(&[("valor", s.to_string().into())]),
                )));
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Split {
    pub id: Id,
    pub category_id: Option<Id>,
    pub member_id: Option<Id>,
    #[ts(type = "number")]
    pub amount: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SplitInput {
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Com o mesmo sinal do lançamento; a soma das partes deve ser igual ao total.
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Tag {
    pub id: Id,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttachmentMeta {
    pub id: Id,
    pub txn_id: Option<Id>,
    pub filename: String,
    pub mime: String,
    #[ts(type = "number")]
    pub size_bytes: i64,
    pub sha256: String,
    pub created_at: String,
}

/// Lançamento como devolvido pela API, já com nomes para exibição.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnView {
    pub id: Id,
    pub account_id: Id,
    pub account_name: String,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    pub kind: TxnKind,
    pub status: TxnStatus,
    pub description: String,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<Id>,
    /// `Alimentação › Mercado`
    pub category_path: Option<String>,
    pub member_id: Option<Id>,
    pub member_name: Option<String>,
    pub transfer_id: Option<Id>,
    /// Conta da outra perna (transferências).
    pub counterpart_account_id: Option<Id>,
    pub counterpart_account_name: Option<String>,
    pub statement_id: Option<Id>,
    /// Data de vencimento da fatura (cartão), base da visão por caixa.
    pub statement_due_date: Option<String>,
    pub installment_plan_id: Option<Id>,
    #[ts(type = "number | null")]
    pub installment_number: Option<i64>,
    #[ts(type = "number | null")]
    pub installment_total: Option<i64>,
    pub recurrence_id: Option<Id>,
    pub import_id: Option<Id>,
    pub external_id: Option<String>,
    pub splits: Vec<Split>,
    pub tags: Vec<Tag>,
    #[ts(type = "number")]
    pub attachment_count: i64,
    pub created_at: String,
    pub updated_at: String,
    /// Projeto do lançamento (reforma, benfeitoria, viagem…).
    pub project_id: Option<Id>,
    pub project_name: Option<String>,
    /// Plástico do cartão (titular, adicional, virtual) em que a compra foi feita.
    pub card_plastic_id: Option<Id>,
    /// Final desse plástico.
    pub card_last4: Option<String>,
    /// Compra no exterior: valor (com o sinal do lançamento) e moeda de origem.
    #[ts(type = "number | null")]
    pub orig_amount: Option<i64>,
    pub orig_currency: Option<String>,
    /// Parcela de empréstimo/financiamento paga por este lançamento (Dívidas).
    pub loan_id: Option<Id>,
    pub loan_name: Option<String>,
    pub loan_installment_id: Option<Id>,
    #[ts(type = "number | null")]
    pub loan_installment_number: Option<i64>,
    /// Operação vinculada (aporte na carteira, cotas capital, meta).
    pub operation: Option<operations::Operation>,
    /// A compra de Consumo que detalha os itens deste lançamento (a primeira, se houver mais).
    #[serde(default)]
    #[ts(optional)]
    pub receipt_id: Option<Id>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TxnSort {
    #[default]
    DateDesc,
    DateAsc,
    AmountDesc,
    AmountAsc,
    DescriptionAsc,
    DescriptionDesc,
    AccountAsc,
    AccountDesc,
    CategoryAsc,
    CategoryDesc,
    MemberAsc,
    MemberDesc,
    StatusAsc,
    StatusDesc,
}

/// Filtro de consulta de lançamentos (tela 03, drill-down dos gráficos, MCP).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnFilter {
    #[ts(optional)]
    pub date_from: Option<String>,
    #[ts(optional)]
    pub date_to: Option<String>,
    /// `accrual` (padrão) filtra pela data do lançamento; `cash` usa o vencimento da
    /// fatura para lançamentos de cartão.
    #[ts(optional)]
    pub basis: Option<crate::budget::Basis>,
    #[ts(optional)]
    pub account_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub category_ids: Option<Vec<Id>>,
    /// Incluir subcategorias das categorias filtradas (padrão: sim).
    #[ts(optional)]
    pub include_subcategories: Option<bool>,
    #[ts(optional)]
    pub member_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub kinds: Option<Vec<TxnKind>>,
    #[ts(optional)]
    pub statuses: Option<Vec<TxnStatus>>,
    /// Busca em descrição, favorecido e notas (sem diferenciar maiúsculas/acentos).
    #[ts(optional)]
    pub text: Option<String>,
    /// Faixa de valor absoluto, em centavos.
    #[ts(optional, type = "number")]
    pub amount_min: Option<i64>,
    #[ts(optional, type = "number")]
    pub amount_max: Option<i64>,
    #[ts(optional)]
    pub tag_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub project_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub import_id: Option<Id>,
    #[ts(optional)]
    pub statement_id: Option<Id>,
    #[ts(optional)]
    pub uncategorized: Option<bool>,
    #[ts(optional)]
    pub sort: Option<TxnSort>,
    /// Tamanho da página (padrão 100, máximo 2000).
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
    #[ts(optional, type = "number")]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CurrencyTotal {
    pub currency: String,
    /// Soma dos valores positivos (entradas).
    #[ts(type = "number")]
    pub inflow: i64,
    /// Soma dos valores negativos, em módulo (saídas, positivo).
    #[ts(type = "number")]
    pub outflow: i64,
    /// Entradas − saídas.
    #[ts(type = "number")]
    pub net: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnPage {
    pub items: Vec<TxnView>,
    #[ts(type = "number")]
    pub total: i64,
    /// Totais de todos os resultados do filtro (não só da página), por moeda.
    pub totals: Vec<CurrencyTotal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnInput {
    pub account_id: Id,
    pub date: String,
    /// Com sinal (+ entra, − sai). Se `kind` for omitido: negativo = despesa, positivo = receita.
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(optional)]
    pub kind: Option<TxnKind>,
    #[ts(optional)]
    pub status: Option<TxnStatus>,
    pub description: String,
    #[ts(optional)]
    pub payee: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[ts(optional)]
    pub splits: Option<Vec<SplitInput>>,
    /// Tags por nome (criadas se não existirem).
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub project_id: Option<Id>,
    /// Plástico do cartão (adicional, virtual…) da compra; o membro vem dele quando não é
    /// informado.
    #[serde(default)]
    #[ts(optional)]
    pub card_plastic_id: Option<Id>,
    /// Compra no exterior: valor na moeda de origem (positivo; o sinal segue o do lançamento)
    /// e a moeda (`USD`).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub orig_amount: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub orig_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnPatch {
    pub id: Id,
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[ts(optional)]
    pub date: Option<String>,
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[ts(optional)]
    pub kind: Option<TxnKind>,
    #[ts(optional)]
    pub status: Option<TxnStatus>,
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub payee: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub category_id: Option<Option<Id>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub member_id: Option<Option<Id>>,
    /// Substitui a divisão inteira (`[]` remove a divisão).
    #[ts(optional)]
    pub splits: Option<Vec<SplitInput>>,
    /// Substitui as tags.
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
    /// Projeto do lançamento (`null` tira do projeto).
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub project_id: Option<Option<Id>>,
    /// Plástico do cartão (`null` tira o plástico).
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub card_plastic_id: Option<Option<Id>>,
    /// Valor e moeda de origem (compra no exterior); `null` em `orig_amount` tira os dois.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub orig_amount: Option<Option<i64>>,
    #[serde(default)]
    #[ts(optional)]
    pub orig_currency: Option<String>,
    /// Transferência para (ou de) outra conta da família: liga este lançamento à perna
    /// correspondente nessa conta (a que já existe, com o mesmo valor e data próxima) ou
    /// cria essa perna.
    #[serde(default)]
    #[ts(optional)]
    pub transfer_account_id: Option<Id>,
    /// Valor que entrou (ou saiu) na outra conta, quando ela usa outra moeda.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub transfer_amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TransferInput {
    pub from_account_id: Id,
    pub to_account_id: Id,
    pub date: String,
    /// Valor positivo que sai da conta de origem (na moeda dela).
    #[ts(type = "number")]
    pub amount: i64,
    /// Valor que entra no destino, quando as moedas diferem (padrão: igual a `amount`).
    #[ts(optional, type = "number")]
    pub to_amount: Option<i64>,
    #[ts(optional)]
    pub description: Option<String>,
    /// `transfer` (padrão), `card_payment` ou `investment`.
    #[ts(optional)]
    pub kind: Option<TxnKind>,
    #[ts(optional)]
    pub status: Option<TxnStatus>,
    #[ts(optional)]
    pub member_id: Option<Id>,
}

/// Resultado de ações em lote (categoria, status, mover para fatura).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BulkResult {
    /// Lançamentos efetivamente alterados.
    #[ts(type = "number")]
    pub updated: i64,
    /// Conjunto de alterações (para desfazer); `null` se nada mudou.
    pub change_set_id: Option<Id>,
    /// Marcados que ficaram de fora porque o tipo não aceita a mudança (categoria num aporte,
    /// pagamento de fatura ou ajuste).
    #[serde(default, skip_serializing_if = "is_zero")]
    #[ts(optional, type = "number")]
    pub skipped: i64,
}

fn is_zero(n: &i64) -> bool {
    *n == 0
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetCategoryArgs {
    pub ids: Vec<Id>,
    /// `null` remove a categoria.
    pub category_id: Option<Id>,
}

/// Titular de vários lançamentos (`null` tira o titular).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetMemberArgs {
    pub ids: Vec<Id>,
    pub member_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetStatusArgs {
    pub ids: Vec<Id>,
    pub status: TxnStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TransferResult {
    pub transfer_id: Id,
    pub from: TxnView,
    pub to: TxnView,
}

/// Dados para inserir um lançamento (uso interno: API, importação, recorrências, parcelas).
#[derive(Debug, Clone)]
pub struct NewTxn {
    pub account_id: Id,
    pub date: String,
    pub amount: i64,
    pub currency: String,
    pub kind: TxnKind,
    pub status: TxnStatus,
    pub description: String,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<Id>,
    pub member_id: Option<Id>,
    pub transfer_id: Option<Id>,
    pub statement_id: Option<Id>,
    pub installment_plan_id: Option<Id>,
    pub installment_number: Option<i64>,
    pub recurrence_id: Option<Id>,
    pub import_id: Option<Id>,
    pub external_id: Option<String>,
    pub card_plastic_id: Option<Id>,
    /// Compra no exterior (com o sinal de `amount`).
    pub orig_amount: Option<i64>,
    pub orig_currency: Option<String>,
}

impl NewTxn {
    pub fn new(
        account_id: &str,
        date: &str,
        amount: i64,
        currency: &str,
        kind: TxnKind,
        description: &str,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            date: date.to_string(),
            amount,
            currency: currency.to_string(),
            kind,
            status: TxnStatus::Cleared,
            description: description.to_string(),
            payee: None,
            notes: None,
            category_id: None,
            member_id: None,
            transfer_id: None,
            statement_id: None,
            installment_plan_id: None,
            installment_number: None,
            recurrence_id: None,
            import_id: None,
            external_id: None,
            card_plastic_id: None,
            orig_amount: None,
            orig_currency: None,
        }
    }
}

/// Valor e moeda de origem de uma compra no exterior: os dois juntos, moeda de 3 letras
/// diferente da do lançamento, valor positivo (gravado com o sinal do lançamento).
pub fn check_orig(
    amount: i64,
    currency: &str,
    orig_amount: Option<i64>,
    orig_currency: Option<&str>,
) -> Result<(Option<i64>, Option<String>)> {
    let cur = orig_currency
        .map(|c| c.trim().to_ascii_uppercase())
        .filter(|c| !c.is_empty());
    match (orig_amount, cur) {
        (None, None) => Ok((None, None)),
        (Some(v), Some(c)) => {
            let c = crate::money::validate_currency(&c)?;
            if c == currency {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-origem-mesma-moeda",
                )));
            }
            if v == 0 {
                return Err(Error::invalid(t("nucleo-lancamentos-erro-origem-valor")));
            }
            Ok((Some(v.abs() * amount.signum()), Some(c)))
        }
        _ => Err(Error::invalid(t("nucleo-lancamentos-erro-origem-par"))),
    }
}

/// Normaliza descrições para comparação: minúsculas, sem acentos, sem pontuação,
/// espaços colapsados e sem sequências de dígitos longas (números de documento).
pub fn normalize_description(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = true;
    for ch in s.chars() {
        let c = fold_accent(ch).to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.split(' ')
        .filter(|w| !(w.len() >= 6 && w.chars().all(|c| c.is_ascii_digit())))
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

pub fn fold_accent(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'ç' => 'c',
        'Ç' => 'C',
        'ñ' => 'n',
        'Ñ' => 'N',
        other => other,
    }
}

/// Impressão digital para deduplicação: data + valor + descrição normalizada.
pub fn fingerprint(date: &str, amount: i64, description: &str) -> String {
    let mut h = Sha256::new();
    h.update(date.as_bytes());
    h.update(b"|");
    h.update(amount.to_string().as_bytes());
    h.update(b"|");
    h.update(normalize_description(description).as_bytes());
    hex::encode(&h.finalize()[..12])
}

/// Valida a divisão: soma igual ao total e mesmo sinal (`currency`: a do lançamento, para a
/// mensagem).
pub fn validate_splits(total: i64, splits: &[SplitInput], currency: &str) -> Result<()> {
    if splits.is_empty() {
        return Ok(());
    }
    if splits.len() < 2 {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-divisao-partes")));
    }
    let sum: i64 = splits.iter().map(|s| s.amount).sum();
    if sum != total {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-divisao-soma",
            &args(&[
                ("soma", util::money_text(sum, currency).into()),
                ("total", util::money_text(total, currency).into()),
            ]),
        )));
    }
    if splits
        .iter()
        .any(|s| s.amount == 0 || (s.amount > 0) != (total > 0))
    {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-divisao-sinal")));
    }
    Ok(())
}

/// Único membro da conta (o que os lançamentos sem membro herdam), se ela tiver um só.
pub fn sole_owner(conn: &Connection, account_id: &str) -> Result<Option<Id>> {
    let owners: Vec<Id> = conn
        .prepare_cached("SELECT member_id FROM account_member WHERE account_id = ?1 LIMIT 2")?
        .query_map([account_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(if owners.len() == 1 {
        owners.into_iter().next()
    } else {
        None
    })
}

/// Membro que um lançamento sem membro escolhido herda: o do plástico do cartão, se ele tiver
/// um; senão o titular único da conta (docs/members.md).
pub fn inherited_member(
    conn: &Connection,
    account_id: &str,
    card_plastic_id: Option<&str>,
) -> Result<Option<Id>> {
    if let Some(p) = card_plastic_id
        && let Some(m) = crate::cards::plastics::member_of(conn, p)?
    {
        return Ok(Some(m));
    }
    sole_owner(conn, account_id)
}

/// Expressão SQL do membro herdado de um lançamento `txn` (o do plástico ou `?2`, o titular
/// único da conta).
const INHERITED_SQL: &str =
    "COALESCE((SELECT p.member_id FROM card_plastic p WHERE p.id = txn.card_plastic_id), ?2)";

/// A conta (ou um plástico do cartão) mudou de membros: os lançamentos com membro herdado (ou
/// sem membro e sem escolha) passam a ter o membro novo, ou ficam sem membro se não houver um.
/// Os escolhidos à mão não mudam. Devolve quantos mudaram.
pub fn reinherit_members(tx: &Transaction<'_>, account_id: &str) -> Result<usize> {
    let owner = sole_owner(tx, account_id)?;
    Ok(tx.execute(
        &format!(
            "UPDATE txn SET member_id = {INHERITED_SQL},
                            member_origin = CASE WHEN {INHERITED_SQL} IS NULL THEN NULL ELSE 'auto' END,
                            updated_at = ?3
              WHERE account_id = ?1
                AND (member_origin = 'auto' OR (member_origin IS NULL AND member_id IS NULL))
                AND member_id IS NOT {INHERITED_SQL}"
        ),
        params![account_id, owner, now_utc()],
    )?)
}

/// Refaz o membro herdado de um lançamento (depois de mudar a conta ou o plástico). Não mexe
/// no escolhido à mão.
pub fn reinherit_one(tx: &Transaction<'_>, txn_id: &str) -> Result<()> {
    let (account_id, plastic): (String, Option<String>) = tx.query_row(
        "SELECT account_id, card_plastic_id FROM txn WHERE id = ?1",
        [txn_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let member = inherited_member(tx, &account_id, plastic.as_deref())?;
    tx.execute(
        "UPDATE txn SET member_id = ?2, member_origin = CASE WHEN ?2 IS NULL THEN NULL ELSE 'auto' END
          WHERE id = ?1 AND (member_origin = 'auto' OR (member_origin IS NULL AND member_id IS NULL))
            AND member_id IS NOT ?2",
        params![txn_id, member],
    )?;
    Ok(())
}

/// Insere um lançamento (sem passar por regras). Calcula `fingerprint` e, para
/// contas de cartão, associa a fatura pela data da compra quando `statement_id` vier vazio.
pub fn insert(tx: &Transaction<'_>, t: &NewTxn) -> Result<Id> {
    crate::time::parse_record_date(&t.date)?;
    let id = new_id();
    let now = now_utc();
    let statement_id = match &t.statement_id {
        Some(s) => Some(s.clone()),
        None => crate::cards::statement_for(tx, &t.account_id, &t.date)?,
    };
    // Sem membro informado: o do plástico do cartão ou o da conta, quando ela tem um membro só
    // (docs/members.md).
    let (member_id, member_origin) = match &t.member_id {
        Some(m) => (Some(m.clone()), Some("manual")),
        None => match inherited_member(tx, &t.account_id, t.card_plastic_id.as_deref())? {
            Some(m) => (Some(m), Some("auto")),
            None => (None, None),
        },
    };
    tx.execute(
        "INSERT INTO txn (id, account_id, date, amount, currency, kind, status, description, payee, notes,
                          category_id, member_id, transfer_id, statement_id, installment_plan_id, installment_number,
                          recurrence_id, import_id, external_id, fingerprint, created_at, updated_at, member_origin,
                          card_plastic_id, orig_amount, orig_currency)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?21, ?22, ?23,
                 ?24, ?25)",
        params![
            id,
            t.account_id,
            t.date,
            t.amount,
            t.currency,
            t.kind.as_str(),
            t.status.as_str(),
            t.description.trim(),
            t.payee,
            t.notes,
            t.category_id,
            member_id,
            t.transfer_id,
            statement_id,
            t.installment_plan_id,
            t.installment_number,
            t.recurrence_id,
            t.import_id,
            t.external_id,
            fingerprint(&t.date, t.amount, &t.description),
            now,
            member_origin,
            t.card_plastic_id,
            t.orig_amount,
            t.orig_currency,
        ],
    )?;
    Ok(id)
}

/// Grava a divisão de um lançamento (substitui a existente).
pub fn replace_splits(
    tx: &Transaction<'_>,
    txn_id: &str,
    total: i64,
    splits: &[SplitInput],
) -> Result<()> {
    let currency: String = match tx
        .query_row("SELECT currency FROM txn WHERE id = ?1", [txn_id], |r| {
            r.get(0)
        })
        .optional()?
    {
        Some(c) => c,
        None => crate::household::base_currency(tx)?,
    };
    validate_splits(total, splits, &currency)?;
    tx.execute("DELETE FROM txn_split WHERE txn_id = ?1", [txn_id])?;
    let now = now_utc();
    for s in splits {
        tx.execute(
            "INSERT INTO txn_split (id, txn_id, category_id, member_id, amount, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![new_id(), txn_id, s.category_id, s.member_id, s.amount, s.notes, now],
        )?;
    }
    Ok(())
}

/// Garante as tags por nome e associa ao lançamento (substitui as existentes).
pub fn replace_tags(tx: &Transaction<'_>, txn_id: &str, names: &[String]) -> Result<()> {
    tx.execute("DELETE FROM txn_tag WHERE txn_id = ?1", [txn_id])?;
    for name in names.iter().map(|n| n.trim()).filter(|n| !n.is_empty()) {
        let tag_id = ensure_tag(tx, name)?;
        tx.execute(
            "INSERT OR IGNORE INTO txn_tag (txn_id, tag_id) VALUES (?1, ?2)",
            params![txn_id, tag_id],
        )?;
    }
    Ok(())
}

pub fn ensure_tag(conn: &Connection, name: &str) -> Result<Id> {
    if let Ok(id) = conn.query_row(
        "SELECT id FROM tag WHERE name = ?1 COLLATE NOCASE",
        [name],
        |r| r.get::<_, String>(0),
    ) {
        return Ok(id);
    }
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO tag (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, name, now],
    )?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_descriptions() {
        assert_eq!(
            normalize_description("PAG*Padaria São João  123456789"),
            "pag padaria sao joao"
        );
        assert_eq!(normalize_description("Uber   *Trip"), "uber trip");
        assert_eq!(
            fingerprint("2026-10-01", -1000, "UBER *TRIP"),
            fingerprint("2026-10-01", -1000, "uber trip")
        );
        assert_ne!(
            fingerprint("2026-10-01", -1000, "uber"),
            fingerprint("2026-10-02", -1000, "uber")
        );
    }

    #[test]
    fn validates_splits() {
        let s = |a| SplitInput {
            category_id: None,
            member_id: None,
            amount: a,
            notes: None,
        };
        assert!(validate_splits(-1000, &[s(-600), s(-400)], "BRL").is_ok());
        assert!(validate_splits(-1000, &[s(-600), s(-300)], "BRL").is_err());
        assert!(validate_splits(-1000, &[s(-1100), s(100)], "BRL").is_err());
        assert!(validate_splits(-1000, &[s(-1000)], "BRL").is_err());
        assert!(validate_splits(-1000, &[], "BRL").is_ok());
    }
}
