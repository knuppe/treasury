//! Plásticos do cartão: titular, adicionais, virtuais e temporários (docs/design/cards.md,
//! fase 1). A fatura e o limite são da conta; o plástico diz **quem gastou**. Só os 4 últimos
//! dígitos são guardados, nunca o número inteiro.
//!
//! Membro do lançamento do cartão: o escolhido à mão; sem ele, o do plástico; sem ele, o titular
//! único da conta (`transactions::inherited_member`).

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{self, args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc};
use crate::transactions::util::{clean_opt, require_member};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PlasticKind {
    Holder,
    Additional,
    Virtual,
    Temporary,
}

impl PlasticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PlasticKind::Holder => "holder",
            PlasticKind::Additional => "additional",
            PlasticKind::Virtual => "virtual",
            PlasticKind::Temporary => "temporary",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "holder" => PlasticKind::Holder,
            "additional" => PlasticKind::Additional,
            "virtual" => PlasticKind::Virtual,
            "temporary" => PlasticKind::Temporary,
            _ => return Err(Error::invalid(t("nucleo-contas-plastico-erro-tipo"))),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardPlastic {
    pub id: Id,
    pub account_id: Id,
    pub kind: PlasticKind,
    pub last4: Option<String>,
    /// Membro do plástico; `null` = vale o titular da conta.
    pub member_id: Option<Id>,
    pub member_name: Option<String>,
    pub label: Option<String>,
    /// Sublimite informativo por ciclo, em centavos (avisa, não bloqueia).
    #[ts(type = "number | null")]
    pub spend_limit: Option<i64>,
    pub active: bool,
    /// Gasto deste plástico na fatura em aberto hoje (compras − estornos).
    #[ts(type = "number")]
    pub cycle_spent: i64,
    #[ts(type = "number")]
    pub txn_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardPlasticInput {
    pub account_id: Id,
    pub kind: PlasticKind,
    /// Só os 4 últimos dígitos.
    #[ts(optional)]
    pub last4: Option<String>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[ts(optional)]
    pub label: Option<String>,
    #[ts(optional, type = "number")]
    pub spend_limit: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardPlasticPatch {
    pub id: Id,
    #[ts(optional)]
    pub kind: Option<PlasticKind>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub last4: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub member_id: Option<Option<Id>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub label: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub spend_limit: Option<Option<i64>>,
    #[ts(optional)]
    pub active: Option<bool>,
}

/// Subtotal da fatura por plástico (como o banco imprime: um bloco por final).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlasticSubtotal {
    /// `null` = lançamentos sem plástico.
    pub card_plastic_id: Option<Id>,
    pub kind: Option<PlasticKind>,
    pub last4: Option<String>,
    pub label: Option<String>,
    /// Quem gastou: o membro do plástico ou, sem ele, o titular único da conta.
    pub member_id: Option<Id>,
    pub member_name: Option<String>,
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub txn_count: i64,
}

fn plastic_ref(id: &str) -> String {
    t_args(
        "nucleo-contas-ref-plastico",
        &args(&[("id", id.to_string().into())]),
    )
}

/// Normaliza o final: aceita `1234`, `final 1234` ou `**** 1234`; recusa mais de 4 dígitos
/// (nunca guardar o número inteiro do cartão).
pub fn normalize_last4(raw: &str) -> Result<String> {
    let digits: String = raw.chars().filter(char::is_ascii_digit).collect();
    if digits.len() > 4 {
        return Err(Error::invalid(t(
            "nucleo-contas-plastico-erro-numero-inteiro",
        )));
    }
    if digits.len() != 4 {
        return Err(Error::invalid(t("nucleo-contas-cartao-erro-final")));
    }
    Ok(digits)
}

fn clean_last4(raw: Option<String>) -> Result<Option<String>> {
    clean_opt(raw).map(|s| normalize_last4(&s)).transpose()
}

fn check_limit(limit: Option<i64>) -> Result<()> {
    if limit.is_some_and(|l| l < 0) {
        return Err(Error::invalid(t("nucleo-contas-plastico-erro-sublimite")));
    }
    Ok(())
}

fn check_unique(
    conn: &Connection,
    account_id: &str,
    last4: &str,
    self_id: Option<&str>,
) -> Result<()> {
    let other: Option<String> = conn
        .query_row(
            "SELECT id FROM card_plastic WHERE account_id = ?1 AND last4 = ?2 AND id IS NOT ?3",
            params![account_id, last4, self_id],
            |r| r.get(0),
        )
        .optional()?;
    if other.is_some() {
        return Err(Error::conflict(t_args(
            "nucleo-contas-plastico-erro-final-repetido",
            &args(&[("final", last4.to_string().into())]),
        )));
    }
    Ok(())
}

const LIST_SQL: &str = "SELECT p.id, p.account_id, p.kind, p.last4, p.member_id, m.name, p.label, p.spend_limit, p.active,
       (SELECT COALESCE(SUM(-t.amount), 0) FROM txn t JOIN card_statement s ON s.id = t.statement_id
         WHERE t.card_plastic_id = p.id AND t.kind <> 'card_payment'
           AND s.period_start <= ?2 AND s.closing_date >= ?2),
       (SELECT COUNT(*) FROM txn t WHERE t.card_plastic_id = p.id)
  FROM card_plastic p LEFT JOIN member m ON m.id = p.member_id";

fn map(r: &rusqlite::Row<'_>) -> rusqlite::Result<(CardPlastic, String)> {
    Ok((
        CardPlastic {
            id: r.get(0)?,
            account_id: r.get(1)?,
            kind: PlasticKind::Holder,
            last4: r.get(3)?,
            member_id: r.get(4)?,
            member_name: r.get(5)?,
            label: r.get(6)?,
            spend_limit: r.get(7)?,
            active: r.get::<_, i64>(8)? == 1,
            cycle_spent: r.get(9)?,
            txn_count: r.get(10)?,
        },
        r.get(2)?,
    ))
}

fn finish((mut p, kind): (CardPlastic, String)) -> Result<CardPlastic> {
    p.kind = PlasticKind::parse(&kind)?;
    Ok(p)
}

/// Plásticos de um cartão (ou de todos), titular primeiro.
pub fn list(
    conn: &Connection,
    account_id: Option<&str>,
    today: NaiveDate,
) -> Result<Vec<CardPlastic>> {
    let mut stmt = conn.prepare_cached(&format!(
        "{LIST_SQL} WHERE (?1 IS NULL OR p.account_id = ?1)
          ORDER BY p.account_id, CASE p.kind WHEN 'holder' THEN 0 WHEN 'additional' THEN 1
                   WHEN 'virtual' THEN 2 ELSE 3 END, p.active DESC, p.last4, p.created_at"
    ))?;
    let raw = stmt
        .query_map(params![account_id, fmt_date(today)], map)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    raw.into_iter().map(finish).collect()
}

pub fn get(conn: &Connection, id: &str, today: NaiveDate) -> Result<CardPlastic> {
    let raw = conn
        .query_row(
            &format!("{LIST_SQL} WHERE p.id = ?1"),
            params![id, fmt_date(today)],
            map,
        )
        .optional()?
        .ok_or_else(|| Error::not_found(plastic_ref(id)))?;
    finish(raw)
}

/// Plástico ativo (ou não) de um cartão pelo final, se cadastrado.
pub fn find_by_last4(conn: &Connection, account_id: &str, last4: &str) -> Result<Option<Id>> {
    Ok(conn
        .prepare_cached("SELECT id FROM card_plastic WHERE account_id = ?1 AND last4 = ?2")?
        .query_row(params![account_id, last4], |r| r.get(0))
        .optional()?)
}

/// Membro do plástico (`None` se o plástico não tem membro próprio).
pub fn member_of(conn: &Connection, plastic_id: &str) -> Result<Option<Id>> {
    Ok(conn
        .prepare_cached("SELECT member_id FROM card_plastic WHERE id = ?1")?
        .query_row([plastic_id], |r| r.get::<_, Option<Id>>(0))
        .optional()?
        .flatten())
}

/// Confere que o plástico existe e é do cartão do lançamento.
pub fn require_on_account(conn: &Connection, plastic_id: &str, account_id: &str) -> Result<()> {
    let acc: Option<String> = conn
        .query_row(
            "SELECT account_id FROM card_plastic WHERE id = ?1",
            [plastic_id],
            |r| r.get(0),
        )
        .optional()?;
    match acc {
        None => Err(Error::not_found(plastic_ref(plastic_id))),
        Some(a) if a != account_id => Err(Error::invalid(t(
            "nucleo-contas-plastico-erro-outro-cartao",
        ))),
        Some(_) => Ok(()),
    }
}

pub fn create(tx: &Transaction<'_>, input: &CardPlasticInput) -> Result<Id> {
    super::require_card(tx, &input.account_id)?;
    let last4 = clean_last4(input.last4.clone())?;
    if let Some(l) = &last4 {
        check_unique(tx, &input.account_id, l, None)?;
    }
    let member_id = clean_opt(input.member_id.clone());
    if let Some(m) = &member_id {
        require_member(tx, m)?;
    }
    check_limit(input.spend_limit)?;
    let id = new_id();
    tx.execute(
        "INSERT INTO card_plastic (id, account_id, kind, last4, member_id, label, spend_limit, active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)",
        params![id, input.account_id, input.kind.as_str(), last4, member_id,
                clean_opt(input.label.clone()), input.spend_limit, now_utc()],
    )?;
    Ok(id)
}

/// Altera o plástico. Mudar o membro refaz o membro herdado dos lançamentos do cartão.
pub fn update(tx: &Transaction<'_>, patch: &CardPlasticPatch, today: NaiveDate) -> Result<()> {
    let cur = get(tx, &patch.id, today)?;
    let kind = patch.kind.unwrap_or(cur.kind);
    let last4 = match &patch.last4 {
        Some(l) => clean_last4(l.clone())?,
        None => cur.last4.clone(),
    };
    if let Some(l) = &last4 {
        check_unique(tx, &cur.account_id, l, Some(&cur.id))?;
    }
    let member_id = match &patch.member_id {
        Some(m) => clean_opt(m.clone()),
        None => cur.member_id.clone(),
    };
    if let Some(m) = &member_id {
        require_member(tx, m)?;
    }
    let label = match &patch.label {
        Some(l) => clean_opt(l.clone()),
        None => cur.label.clone(),
    };
    let spend_limit = patch.spend_limit.unwrap_or(cur.spend_limit);
    check_limit(spend_limit)?;
    let active = patch.active.unwrap_or(cur.active);
    tx.execute(
        "UPDATE card_plastic SET kind = ?2, last4 = ?3, member_id = ?4, label = ?5, spend_limit = ?6,
                active = ?7, updated_at = ?8 WHERE id = ?1",
        params![cur.id, kind.as_str(), last4, member_id, label, spend_limit, i64::from(active), now_utc()],
    )?;
    if member_id != cur.member_id {
        crate::transactions::reinherit_members(tx, &cur.account_id)?;
    }
    Ok(())
}

/// Apaga o plástico: os lançamentos dele ficam sem plástico (e voltam ao titular da conta, se o
/// membro era herdado). Devolve quantos lançamentos estavam nele.
pub fn delete(tx: &Transaction<'_>, id: &str, today: NaiveDate) -> Result<usize> {
    let cur = get(tx, id, today)?;
    let n = tx.execute(
        "UPDATE txn SET card_plastic_id = NULL, updated_at = ?2 WHERE card_plastic_id = ?1",
        params![id, now_utc()],
    )?;
    tx.execute("DELETE FROM card_plastic WHERE id = ?1", [id])?;
    crate::transactions::reinherit_members(tx, &cur.account_id)?;
    Ok(n)
}

/// O cartão mudou o final no cadastro da conta: o plástico do titular acompanha (ou nasce).
pub fn sync_holder_last4(
    tx: &Transaction<'_>,
    account_id: &str,
    last4: Option<&str>,
) -> Result<()> {
    let Some(last4) = last4 else {
        return Ok(());
    };
    if find_by_last4(tx, account_id, last4)?.is_some() {
        return Ok(());
    }
    let holder: Option<String> = tx
        .query_row(
            "SELECT id FROM card_plastic WHERE account_id = ?1 AND kind = 'holder' ORDER BY created_at LIMIT 1",
            [account_id],
            |r| r.get(0),
        )
        .optional()?;
    let now = now_utc();
    match holder {
        Some(h) => tx.execute(
            "UPDATE card_plastic SET last4 = ?2, updated_at = ?3 WHERE id = ?1",
            params![h, last4, now],
        )?,
        None => tx.execute(
            "INSERT INTO card_plastic (id, account_id, kind, last4, active, created_at, updated_at)
             VALUES (?1, ?2, 'holder', ?3, 1, ?4, ?4)",
            params![new_id(), account_id, last4, now],
        )?,
    };
    Ok(())
}

/// Subtotais de uma fatura por plástico, na ordem dos plásticos (sem plástico por último).
pub fn statement_subtotals(conn: &Connection, statement_id: &str) -> Result<Vec<PlasticSubtotal>> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.card_plastic_id, p.kind, p.last4, p.label, t.member_id, m.name,
                COALESCE(SUM(-t.amount), 0), COUNT(*)
           FROM txn t
           LEFT JOIN card_plastic p ON p.id = t.card_plastic_id
           LEFT JOIN member m ON m.id = t.member_id
          WHERE t.statement_id = ?1 AND t.kind <> 'card_payment'
          GROUP BY t.card_plastic_id, t.member_id
          ORDER BY t.card_plastic_id IS NULL,
                   CASE p.kind WHEN 'holder' THEN 0 WHEN 'additional' THEN 1 WHEN 'virtual' THEN 2 ELSE 3 END,
                   p.last4, m.name",
    )?;
    let rows = stmt
        .query_map([statement_id], |r| {
            Ok((
                PlasticSubtotal {
                    card_plastic_id: r.get(0)?,
                    kind: None,
                    last4: r.get(2)?,
                    label: r.get(3)?,
                    member_id: r.get(4)?,
                    member_name: r.get(5)?,
                    total: r.get(6)?,
                    txn_count: r.get(7)?,
                },
                r.get::<_, Option<String>>(1)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut out: Vec<PlasticSubtotal> = Vec::with_capacity(rows.len());
    for (mut s, kind) in rows {
        s.kind = kind.as_deref().map(PlasticKind::parse).transpose()?;
        out.push(s);
    }
    // Sem nenhum plástico na fatura, o agrupamento não ajuda.
    if out.iter().all(|s| s.card_plastic_id.is_none()) {
        out.clear();
    }
    Ok(out)
}

/// Rótulo curto de um plástico para mensagens: "final 1234" ou o tipo.
pub fn short_label(p: &CardPlastic) -> String {
    match &p.last4 {
        Some(l) => t_args(
            "nucleo-contas-plastico-final",
            &args(&[("final", l.clone().into())]),
        ),
        None => i18n::t(match p.kind {
            PlasticKind::Holder => "nucleo-contas-plastico-tipo-titular",
            PlasticKind::Additional => "nucleo-contas-plastico-tipo-adicional",
            PlasticKind::Virtual => "nucleo-contas-plastico-tipo-virtual",
            PlasticKind::Temporary => "nucleo-contas-plastico-tipo-temporario",
        }),
    }
}
