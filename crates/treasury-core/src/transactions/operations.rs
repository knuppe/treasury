//! Vínculo de um lançamento com a operação que ele pagou (ou de onde o dinheiro veio):
//!
//! - **aporte na carteira** (`trade`): a operação de compra (ou venda) de um ativo, inclusive
//!   previdência e contratos de investimento cadastrados como ativo (CDB do banco, título,
//!   fundo): o lançamento da conta corrente que pagou a aplicação;
//! - **cotas capital** (`capital_share`): a conta da cooperativa onde as cotas estão ativas;
//!   o lançamento entra nos aportes das cotas mesmo sem bater com o texto configurado;
//! - **meta / poupança programada** (`goal`): o depósito (ou resgate) entra no valor da meta.
//!
//! As parcelas de dívida seguem na ligação própria (`debt.link_payment`), e as
//! transferências entre contas, em `transactions.link_transfers`.
//!
//! Vinculado, o lançamento vira **aporte** (`investment`): sai de receitas e despesas e perde
//! a categoria; o tipo e a categoria de antes ficam guardados e voltam ao desvincular. Um
//! lançamento tem no máximo um vínculo (com uma operação, uma parcela ou uma transferência).

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{TxnKind, TxnStatus};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::import::text::fold;
use crate::time::now_utc;

/// Distância máxima (dias) entre o lançamento e a operação nos candidatos.
pub const CANDIDATE_DAYS: i64 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "TxnOperationKind")]
pub enum OperationKind {
    /// Operação da carteira (aporte ou resgate num ativo).
    Trade,
    /// Cotas capital de cooperativa (o id é a conta da cooperativa).
    CapitalShare,
    /// Meta ou poupança programada.
    Goal,
}

impl OperationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            OperationKind::Trade => "trade",
            OperationKind::CapitalShare => "capital_share",
            OperationKind::Goal => "goal",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "trade" => Some(OperationKind::Trade),
            "capital_share" => Some(OperationKind::CapitalShare),
            "goal" => Some(OperationKind::Goal),
            _ => None,
        }
    }
}

/// O vínculo de um lançamento, como as telas mostram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "TxnOperation")]
pub struct Operation {
    pub kind: OperationKind,
    pub target_id: Id,
    /// "Aporte em PETR4 (compra de 10 em 05/10/2026)", "Cotas capital · Sicoob", "Meta Viagem".
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, rename = "LinkOperationArgs")]
pub struct LinkArgs {
    pub txn_id: Id,
    pub kind: OperationKind,
    pub target_id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub note: Option<String>,
}

/// Uma operação que pode ser a do lançamento.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "OperationCandidate")]
pub struct Candidate {
    pub kind: OperationKind,
    pub target_id: Id,
    pub label: String,
    /// Por que aparece (valor e data perto, texto do extrato…).
    pub reason: String,
    /// 0 a 100.
    #[ts(type = "number")]
    pub score: i64,
}

use super::util::date_text;

/// "Cotas capital · {conta}".
fn capital_label(account: String) -> String {
    t_args(
        "nucleo-lancamentos-operacao-cotas-capital",
        &args(&[("conta", account.into())]),
    )
}

/// "Meta {nome}".
fn goal_label(name: String) -> String {
    t_args(
        "nucleo-lancamentos-operacao-meta",
        &args(&[("nome", name.into())]),
    )
}

/// Detalhe do "não encontrado" de cada tipo de operação.
fn missing(kind: OperationKind) -> String {
    t(match kind {
        OperationKind::Trade => "nucleo-lancamentos-ref-operacao-carteira",
        OperationKind::CapitalShare => "nucleo-lancamentos-ref-cotas-capital",
        OperationKind::Goal => "nucleo-lancamentos-ref-meta",
    })
}

/// Rótulo da operação vinculada.
pub fn label(conn: &Connection, kind: OperationKind, id: &str) -> Result<Option<String>> {
    Ok(match kind {
        OperationKind::Trade => conn
            .query_row(
                "SELECT COALESCE(a.ticker, a.name), t.side, t.quantity, t.date FROM trade t JOIN asset a ON a.id = t.asset_id WHERE t.id = ?1",
                [id],
                |r| {
                    let side: String = r.get(1)?;
                    Ok(t_args(
                        if side == "buy" {
                            "nucleo-lancamentos-operacao-aporte"
                        } else {
                            "nucleo-lancamentos-operacao-resgate"
                        },
                        &args(&[
                            ("ativo", r.get::<_, String>(0)?.into()),
                            ("quantidade", r.get::<_, String>(2)?.into()),
                            ("data", date_text(&r.get::<_, String>(3)?).into()),
                        ]),
                    ))
                },
            )
            .optional()?,
        OperationKind::CapitalShare => conn
            .query_row(
                "SELECT a.name FROM capital_share c JOIN account a ON a.id = c.account_id WHERE c.account_id = ?1",
                [id],
                |r| Ok(capital_label(r.get::<_, String>(0)?)),
            )
            .optional()?,
        OperationKind::Goal => conn
            .query_row("SELECT name FROM goal WHERE id = ?1", [id], |r| {
                Ok(goal_label(r.get::<_, String>(0)?))
            })
            .optional()?,
    })
}

/// Vínculo atual de um lançamento.
pub fn get(conn: &Connection, txn_id: &str) -> Result<Option<Operation>> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT target_kind, target_id FROM txn_link WHERE txn_id = ?1",
            [txn_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((k, id)) = row else { return Ok(None) };
    let Some(kind) = OperationKind::parse(&k) else {
        return Ok(None);
    };
    let label = label(conn, kind, &id)?.unwrap_or_else(|| t("nucleo-lancamentos-operacao-apagada"));
    Ok(Some(Operation {
        kind,
        target_id: id,
        label,
    }))
}

/// Recusa ligar a outra coisa (parcela, transferência) um lançamento já vinculado a uma
/// operação.
pub fn ensure_free(conn: &Connection, txn_id: &str) -> Result<()> {
    match get(conn, txn_id)? {
        Some(op) => Err(Error::conflict(t_args(
            "nucleo-lancamentos-erro-vinculado",
            &args(&[("operacao", op.label.into())]),
        ))),
        None => Ok(()),
    }
}

/// Confere se o lançamento pode ser vinculado à operação (sem gravar nada).
fn check(tx: &Connection, a: &LinkArgs) -> Result<super::TxnRow> {
    let row = super::load_row(tx, &a.txn_id)?;
    if row.amount == 0 {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-sem-valor")));
    }
    if row.status == TxnStatus::Planned {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-vincular-previsto",
        )));
    }
    if row.transfer_id.is_some() {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-vincular-transferencia",
        )));
    }
    if !matches!(
        row.kind,
        TxnKind::Expense | TxnKind::Income | TxnKind::Transfer | TxnKind::Investment
    ) {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-vincular-tipo")));
    }
    let split: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM txn_split WHERE txn_id = ?1)",
        [&a.txn_id],
        |r| r.get(0),
    )?;
    if split {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-vincular-dividido",
        )));
    }
    if let Some(cur) = get(tx, &a.txn_id)? {
        return Err(Error::conflict(t_args(
            "nucleo-lancamentos-erro-ja-vinculado",
            &args(&[("operacao", cur.label.into())]),
        )));
    }
    let loan: Option<String> = tx
        .query_row(
            "SELECT l.name FROM loan_installment li JOIN loan l ON l.id = li.loan_id WHERE li.txn_id = ?1 LIMIT 1",
            [&a.txn_id],
            |r| r.get(0),
        )
        .optional()?;
    let extra: Option<String> = tx
        .query_row(
            "SELECT l.name FROM loan_extra_payment e JOIN loan l ON l.id = e.loan_id WHERE e.txn_id = ?1 LIMIT 1",
            [&a.txn_id],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(name) = loan.or(extra) {
        return Err(Error::conflict(t_args(
            "nucleo-lancamentos-erro-vincular-parcela",
            &args(&[("divida", name.into())]),
        )));
    }
    // A operação existe e combina com o lançamento.
    match a.kind {
        OperationKind::Trade => {
            let (side, cash): (String, Option<String>) = tx
                .query_row(
                    "SELECT side, txn_id FROM trade WHERE id = ?1",
                    [&a.target_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?
                .ok_or_else(|| Error::not_found(missing(OperationKind::Trade)))?;
            if cash.as_deref() == Some(a.txn_id.as_str()) {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-vincular-caixa-corretora",
                )));
            }
            let taken: Option<String> = tx
                .query_row(
                    "SELECT txn_id FROM txn_link WHERE target_kind = 'trade' AND target_id = ?1",
                    [&a.target_id],
                    |r| r.get(0),
                )
                .optional()?;
            if taken.is_some() {
                return Err(Error::conflict(t(
                    "nucleo-lancamentos-erro-operacao-ocupada",
                )));
            }
            if (side == "buy") != (row.amount < 0) {
                return Err(Error::invalid(t(if side == "buy" {
                    "nucleo-lancamentos-erro-aporte-saida"
                } else {
                    "nucleo-lancamentos-erro-resgate-entrada"
                })));
            }
        }
        OperationKind::CapitalShare => {
            let ok: bool = tx.query_row(
                "SELECT EXISTS (SELECT 1 FROM capital_share WHERE account_id = ?1)",
                [&a.target_id],
                |r| r.get(0),
            )?;
            if !ok {
                return Err(Error::not_found(missing(OperationKind::CapitalShare)));
            }
        }
        OperationKind::Goal => {
            let archived: bool = tx
                .query_row(
                    "SELECT archived FROM goal WHERE id = ?1",
                    [&a.target_id],
                    |r| r.get::<_, i64>(0).map(|v| v != 0),
                )
                .optional()?
                .ok_or_else(|| Error::not_found(missing(OperationKind::Goal)))?;
            if archived {
                return Err(Error::invalid(t("nucleo-lancamentos-erro-meta-arquivada")));
            }
        }
    }
    Ok(row)
}

/// Vincula o lançamento à operação.
pub fn link(tx: &Transaction<'_>, a: &LinkArgs) -> Result<Operation> {
    let row = check(tx, a)?;
    let now = now_utc();
    tx.execute(
        "INSERT INTO txn_link (txn_id, target_kind, target_id, previous_kind, previous_category_id, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            a.txn_id,
            a.kind.as_str(),
            a.target_id,
            row.kind.as_str(),
            row.category_id,
            a.note.as_deref().map(str::trim).filter(|n| !n.is_empty()),
            now
        ],
    )?;
    tx.execute(
        "UPDATE txn SET kind = 'investment', category_id = NULL, updated_at = ?2 WHERE id = ?1",
        params![a.txn_id, now],
    )?;
    get(tx, &a.txn_id)?
        .ok_or_else(|| Error::Other(t("nucleo-lancamentos-erro-vinculo-nao-gravado")))
}

/// Desfaz o vínculo: o lançamento volta ao tipo e à categoria de antes.
pub fn unlink(tx: &Transaction<'_>, txn_id: &str) -> Result<()> {
    let prev: Option<(String, Option<String>)> = tx
        .query_row(
            "SELECT previous_kind, previous_category_id FROM txn_link WHERE txn_id = ?1",
            [txn_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((kind, cat)) = prev else {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-nao-vinculado")));
    };
    tx.execute("DELETE FROM txn_link WHERE txn_id = ?1", [txn_id])?;
    // A categoria antiga pode ter sido apagada.
    let cat = cat.filter(|c| {
        tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM category WHERE id = ?1)",
            [c],
            |r| r.get(0),
        )
        .unwrap_or(false)
    });
    tx.execute(
        "UPDATE txn SET kind = ?2, category_id = ?3, updated_at = ?4 WHERE id = ?1",
        params![txn_id, kind, cat, now_utc()],
    )?;
    Ok(())
}

fn days_between(a: &str, b: &str) -> i64 {
    match (
        chrono::NaiveDate::parse_from_str(a, "%Y-%m-%d"),
        chrono::NaiveDate::parse_from_str(b, "%Y-%m-%d"),
    ) {
        (Ok(x), Ok(y)) => (x - y).num_days().abs(),
        _ => i64::MAX,
    }
}

/// (id, data, quantidade, preço, corretagem, impostos, moeda, ticker, nome).
type TradeRow = (
    String,
    String,
    String,
    String,
    i64,
    i64,
    String,
    String,
    String,
);

/// Operações que podem ser a do lançamento, da mais provável para a menos.
pub fn candidates(conn: &Connection, txn_id: &str) -> Result<Vec<Candidate>> {
    let row = super::load_row(conn, txn_id)?;
    let desc = fold(&row.description);
    let abs = row.amount.abs();
    let mut out = vec![];
    // Operações da carteira perto da data, de valor parecido (bruto + custos).
    let side = if row.amount < 0 { "buy" } else { "sell" };
    let mut stmt = conn.prepare(
        "SELECT t.id, t.date, t.quantity, t.price, t.fees, t.taxes, t.currency, COALESCE(a.ticker, a.name), a.name
           FROM trade t JOIN asset a ON a.id = t.asset_id
          WHERE t.side = ?1 AND abs(julianday(t.date) - julianday(?2)) <= ?3
            AND t.id NOT IN (SELECT target_id FROM txn_link WHERE target_kind = 'trade')
            AND (t.txn_id IS NULL OR t.txn_id <> ?4)
          ORDER BY abs(julianday(t.date) - julianday(?2))",
    )?;
    let trades: Vec<TradeRow> = stmt
        .query_map(params![side, row.date, CANDIDATE_DAYS, txn_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    for (id, date, qty, price, fees, taxes, cur, ticker, name) in trades {
        let (Ok(q), Ok(p)) = (
            qty.parse::<rust_decimal::Decimal>(),
            price.parse::<rust_decimal::Decimal>(),
        ) else {
            continue;
        };
        let gross = crate::portfolio::records::to_minor(q * p, &cur);
        let total = if side == "buy" {
            gross + fees + taxes
        } else {
            gross - fees - taxes
        };
        if total <= 0 || cur != row.currency {
            continue;
        }
        let diff = (total - abs).abs() as f64 / total as f64;
        if diff > 0.1 {
            continue;
        }
        let gap = days_between(&date, &row.date);
        let named = desc.contains(&fold(&ticker)) || desc.contains(&fold(&name));
        let score =
            (90 - (diff * 300.0) as i64 - gap * 5 + if named { 10 } else { 0 }).clamp(5, 100);
        out.push(Candidate {
            kind: OperationKind::Trade,
            target_id: id.clone(),
            label: label(conn, OperationKind::Trade, &id)?.unwrap_or_default(),
            reason: t_args(
                "nucleo-lancamentos-motivo-operacao",
                &args(&[
                    (
                        "valor",
                        if diff < 0.005 { "igual" } else { "parecido" }.into(),
                    ),
                    ("dias", gap.into()),
                    (
                        "diferenca",
                        super::util::money_text((total - abs).abs(), &cur).into(),
                    ),
                ]),
            ),
            score,
        });
    }
    // Cotas capital: a da própria conta ou a que o texto do extrato cita.
    let mut stmt = conn.prepare(
        "SELECT c.account_id, a.name, COALESCE(c.contribution_pattern, ?1) FROM capital_share c JOIN account a ON a.id = c.account_id",
    )?;
    let shares: Vec<(String, String, String)> = stmt
        .query_map([crate::accounts::capital::DEFAULT_PATTERN], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    for (acc, name, pattern) in shares {
        let by_text = pattern
            .split('|')
            .map(|t| fold(t.trim()))
            .any(|t| !t.is_empty() && desc.contains(&t));
        let same = acc == row.account_id;
        if !by_text && !same {
            continue;
        }
        out.push(Candidate {
            kind: OperationKind::CapitalShare,
            target_id: acc.clone(),
            label: capital_label(name),
            reason: t(if by_text {
                "nucleo-lancamentos-motivo-texto-cota-capital"
            } else {
                "nucleo-lancamentos-motivo-conta-cotas-capital"
            }),
            score: if by_text { 85 } else { 40 },
        });
    }
    // Metas ativas: as que o texto cita primeiro.
    let mut stmt =
        conn.prepare("SELECT id, name FROM goal WHERE archived = 0 ORDER BY created_at")?;
    let goals: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let programmed = desc.contains("poupanca programada")
        || desc.contains("capitalizacao")
        || desc.contains("aplicacao programada");
    for (id, name) in goals {
        let named = fold(&name)
            .split_whitespace()
            .any(|w| w.len() >= 4 && desc.contains(w));
        out.push(Candidate {
            kind: OperationKind::Goal,
            target_id: id,
            label: goal_label(name),
            reason: t(if named {
                "nucleo-lancamentos-motivo-texto-meta"
            } else if programmed {
                "nucleo-lancamentos-motivo-poupanca-programada"
            } else {
                "nucleo-lancamentos-motivo-meta-ativa"
            }),
            score: if named {
                70
            } else if programmed {
                45
            } else {
                10
            },
        });
    }
    out.sort_by_key(|c| std::cmp::Reverse(c.score));
    Ok(out)
}

/// Vários lançamentos para a mesma operação (ex.: os depósitos mensais da poupança programada).
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, rename = "LinkOperationsArgs")]
pub struct BulkLinkArgs {
    pub txn_ids: Vec<Id>,
    pub kind: OperationKind,
    pub target_id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub note: Option<String>,
}

/// Um lançamento da prévia: entra no vínculo ou fica de fora, com o motivo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "OperationLinkItem")]
pub struct BulkItem {
    pub txn_id: Id,
    pub date: String,
    pub description: String,
    pub account_name: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    pub ok: bool,
    /// Por que fica de fora (já vinculado, previsto, parcela de dívida…).
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "OperationLinkTotal")]
pub struct BulkTotal {
    pub currency: String,
    #[ts(type = "number")]
    pub amount: i64,
}

/// O que a confirmação mostra antes de vincular vários lançamentos.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "OperationLinkPreview")]
pub struct BulkPreview {
    /// Rótulo da operação de destino.
    pub label: String,
    pub items: Vec<BulkItem>,
    #[ts(type = "number")]
    pub linkable: u32,
    /// Soma dos que entram, por moeda.
    pub totals: Vec<BulkTotal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "LinkOperationsResult")]
pub struct BulkLinkResult {
    pub label: String,
    #[ts(type = "number")]
    pub linked: u32,
    #[ts(type = "number")]
    pub skipped: u32,
    pub change_set_id: Option<Id>,
}

/// Máximo de lançamentos por vínculo em lote.
pub const MAX_BULK: usize = 500;

fn too_many() -> Error {
    Error::invalid(t_args(
        "nucleo-lancamentos-erro-maximo-lote",
        &args(&[("max", MAX_BULK.into())]),
    ))
}

/// Prévia do vínculo em lote: quais lançamentos entram e por que os outros ficam de fora.
pub fn preview_many(conn: &Connection, a: &BulkLinkArgs) -> Result<BulkPreview> {
    let ids = super::util::dedup_ids(&a.txn_ids);
    if ids.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-nenhum-selecionado",
        )));
    }
    if ids.len() > MAX_BULK {
        return Err(too_many());
    }
    let label =
        label(conn, a.kind, &a.target_id)?.ok_or_else(|| Error::not_found(missing(a.kind)))?;
    let mut items = Vec::with_capacity(ids.len());
    let mut totals: Vec<BulkTotal> = vec![];
    let mut trade_taken = false;
    for id in ids {
        let row = super::load_row(conn, &id)?;
        let account_name: String = conn.query_row(
            "SELECT name FROM account WHERE id = ?1",
            [&row.account_id],
            |r| r.get(0),
        )?;
        let single = LinkArgs {
            txn_id: id.clone(),
            kind: a.kind,
            target_id: a.target_id.clone(),
            note: None,
        };
        let mut reason = check(conn, &single).err().map(|e| e.message());
        // Uma operação da carteira aceita um lançamento só: o primeiro que serve.
        if reason.is_none() && a.kind == OperationKind::Trade {
            if trade_taken {
                reason = Some(t("nucleo-lancamentos-erro-operacao-um-lancamento"));
            }
            trade_taken = true;
        }
        let ok = reason.is_none();
        if ok {
            match totals.iter_mut().find(|t| t.currency == row.currency) {
                Some(t) => t.amount += row.amount,
                None => totals.push(BulkTotal {
                    currency: row.currency.clone(),
                    amount: row.amount,
                }),
            }
        }
        items.push(BulkItem {
            txn_id: id,
            date: row.date,
            description: row.description,
            account_name,
            amount: row.amount,
            currency: row.currency,
            ok,
            reason,
        });
    }
    let linkable = items.iter().filter(|i| i.ok).count() as u32;
    Ok(BulkPreview {
        label,
        items,
        linkable,
        totals,
    })
}

/// Vincula à operação os lançamentos que servem e pula os outros (os da prévia).
pub fn link_many(tx: &Transaction<'_>, a: &BulkLinkArgs) -> Result<(String, u32, u32)> {
    let p = preview_many(tx, a)?;
    if p.linkable == 0 {
        let why = p
            .items
            .iter()
            .find_map(|i| i.reason.clone())
            .unwrap_or_default();
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-nenhum-vinculavel",
            &args(&[("operacao", p.label.into()), ("motivo", why.into())]),
        )));
    }
    for i in p.items.iter().filter(|i| i.ok) {
        link(
            tx,
            &LinkArgs {
                txn_id: i.txn_id.clone(),
                kind: a.kind,
                target_id: a.target_id.clone(),
                note: a.note.clone(),
            },
        )?;
    }
    Ok((p.label, p.linkable, p.items.len() as u32 - p.linkable))
}

/// Candidatos para vários lançamentos: as operações que servem a algum deles, primeiro as
/// que combinam com mais lançamentos. Compras e vendas da carteira só aparecem quando há um
/// lançamento só (cada uma aceita um lançamento).
pub fn candidates_many(conn: &Connection, txn_ids: &[Id]) -> Result<Vec<Candidate>> {
    let ids = super::util::dedup_ids(txn_ids);
    if ids.len() > MAX_BULK {
        return Err(too_many());
    }
    if ids.len() == 1 {
        return candidates(conn, &ids[0]);
    }
    let mut merged: Vec<(Candidate, u32)> = vec![];
    for id in &ids {
        for c in candidates(conn, id)? {
            if c.kind == OperationKind::Trade {
                continue;
            }
            match merged
                .iter_mut()
                .find(|(m, _)| m.kind == c.kind && m.target_id == c.target_id)
            {
                Some((m, n)) => {
                    *n += 1;
                    if c.score > m.score {
                        m.score = c.score;
                        m.reason = c.reason;
                    }
                }
                None => merged.push((c, 1)),
            }
        }
    }
    let total = ids.len();
    merged.sort_by_key(|(c, n)| (std::cmp::Reverse(*n), std::cmp::Reverse(c.score)));
    Ok(merged
        .into_iter()
        .map(|(mut c, n)| {
            if (n as usize) < total {
                c.reason = t_args(
                    "nucleo-lancamentos-motivo-combina-com",
                    &args(&[
                        ("motivo", c.reason.into()),
                        ("n", n.into()),
                        ("total", total.into()),
                    ]),
                );
            }
            c
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "LinkOperationResult")]
pub struct LinkResult {
    pub operation: Operation,
    pub change_set_id: Option<Id>,
}

#[cfg(test)]
#[path = "operations_tests.rs"]
mod tests;
