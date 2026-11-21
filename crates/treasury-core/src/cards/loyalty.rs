//! Pontos e milhas (docs/design/cards.md, fase 5): programas por membro, movimentos, saldo,
//! vencimentos (lotes consumidos na ordem de vencimento) e transferências com bônus.
//!
//! Pontos ficam fora do patrimônio. O valor estimado (premissa da família, por milheiro) é só
//! uma linha à parte.

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc, parse_date};
use crate::transactions::util::{clean_opt, require_member};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LoyaltyKind {
    Bank,
    Airline,
    Retail,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EntryKind {
    Earn,
    Bonus,
    Purchase,
    TransferIn,
    TransferOut,
    Redeem,
    Expire,
    Adjust,
}

impl LoyaltyKind {
    fn as_str(self) -> &'static str {
        match self {
            LoyaltyKind::Bank => "bank",
            LoyaltyKind::Airline => "airline",
            LoyaltyKind::Retail => "retail",
            LoyaltyKind::Other => "other",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "airline" => LoyaltyKind::Airline,
            "retail" => LoyaltyKind::Retail,
            "other" => LoyaltyKind::Other,
            _ => LoyaltyKind::Bank,
        }
    }
}

impl EntryKind {
    fn as_str(self) -> &'static str {
        match self {
            EntryKind::Earn => "earn",
            EntryKind::Bonus => "bonus",
            EntryKind::Purchase => "purchase",
            EntryKind::TransferIn => "transfer_in",
            EntryKind::TransferOut => "transfer_out",
            EntryKind::Redeem => "redeem",
            EntryKind::Expire => "expire",
            EntryKind::Adjust => "adjust",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "bonus" => EntryKind::Bonus,
            "purchase" => EntryKind::Purchase,
            "transfer_in" => EntryKind::TransferIn,
            "transfer_out" => EntryKind::TransferOut,
            "redeem" => EntryKind::Redeem,
            "expire" => EntryKind::Expire,
            "adjust" => EntryKind::Adjust,
            _ => EntryKind::Earn,
        }
    }
    /// Entradas somam; saídas subtraem; o ajuste vale com o sinal informado.
    fn sign(self) -> i64 {
        match self {
            EntryKind::TransferOut | EntryKind::Redeem | EntryKind::Expire => -1,
            EntryKind::Adjust => 0,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LinkedCard {
    pub account_id: Id,
    pub account_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoyaltyProgram {
    pub id: Id,
    pub name: String,
    pub kind: LoyaltyKind,
    pub member_id: Option<Id>,
    pub member_name: Option<String>,
    pub unit_label: Option<String>,
    /// Premissa da família: valor de 1.000 pontos, em centavos.
    #[ts(type = "number | null")]
    pub value_per_1000: Option<i64>,
    pub value_currency: Option<String>,
    pub active: bool,
    pub notes: Option<String>,
    #[ts(type = "number")]
    pub balance: i64,
    /// Pontos que vencem nos próximos 90 dias e a data do primeiro vencimento.
    #[ts(type = "number")]
    pub expiring_90d: i64,
    pub next_expiry: Option<String>,
    #[ts(type = "number")]
    pub next_expiry_points: i64,
    /// Saldo × valor do milheiro (fora do patrimônio).
    #[ts(type = "number | null")]
    pub estimated_value: Option<i64>,
    /// Custo médio do milheiro comprado (centavos), quando houve compra de pontos.
    #[ts(type = "number | null")]
    pub purchase_cost_per_1000: Option<i64>,
    pub cards: Vec<LinkedCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoyaltyProgramInput {
    pub name: String,
    #[ts(optional)]
    pub kind: Option<LoyaltyKind>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[ts(optional)]
    pub unit_label: Option<String>,
    #[ts(optional, type = "number")]
    pub value_per_1000: Option<i64>,
    #[ts(optional)]
    pub value_currency: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoyaltyProgramPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub kind: Option<LoyaltyKind>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub member_id: Option<Option<Id>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub unit_label: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub value_per_1000: Option<Option<i64>>,
    #[ts(optional)]
    pub active: Option<bool>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoyaltyEntry {
    pub id: Id,
    pub program_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub points: i64,
    pub kind: EntryKind,
    pub expires_on: Option<String>,
    pub statement_id: Option<Id>,
    pub transfer_id: Option<Id>,
    #[ts(type = "number | null")]
    pub cost: Option<i64>,
    pub cost_currency: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoyaltyEntryInput {
    pub program_id: Id,
    pub date: String,
    /// Positivo; o tipo dá o sinal (no ajuste, vale o sinal informado).
    #[ts(type = "number")]
    pub points: i64,
    pub kind: EntryKind,
    #[ts(optional)]
    pub expires_on: Option<String>,
    /// Fatura do cartão que gerou os pontos (acúmulo).
    #[ts(optional)]
    pub statement_id: Option<Id>,
    /// Compra de pontos: quanto custou (centavos) e a moeda.
    #[ts(optional, type = "number")]
    pub cost: Option<i64>,
    #[ts(optional)]
    pub cost_currency: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoyaltyTransferInput {
    pub from_program_id: Id,
    pub to_program_id: Id,
    pub date: String,
    /// Pontos que saem da origem.
    #[ts(type = "number")]
    pub points_out: i64,
    /// Pontos que entram no destino (com o bônus de transferência).
    #[ts(type = "number")]
    pub points_in: i64,
    #[ts(optional)]
    pub expires_on: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
}

fn program_ref(id: &str) -> String {
    t_args(
        "nucleo-pontos-ref-programa",
        &args(&[("id", id.to_string().into())]),
    )
}

fn check_date(d: &Option<String>) -> Result<()> {
    if let Some(d) = d {
        parse_date(d)?;
    }
    Ok(())
}

pub fn require_program(conn: &Connection, id: &str) -> Result<()> {
    if !crate::transactions::util::exists(conn, "loyalty_program", id)? {
        return Err(Error::not_found(program_ref(id)));
    }
    Ok(())
}

pub fn entries(conn: &Connection, program_id: &str) -> Result<Vec<LoyaltyEntry>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, program_id, date, points, kind, expires_on, statement_id, transfer_id, cost,
                cost_currency, notes
           FROM loyalty_entry WHERE program_id = ?1 ORDER BY date DESC, created_at DESC",
    )?;
    let rows = stmt
        .query_map([program_id], |r| {
            Ok(LoyaltyEntry {
                id: r.get(0)?,
                program_id: r.get(1)?,
                date: r.get(2)?,
                points: r.get(3)?,
                kind: EntryKind::parse(&r.get::<_, String>(4)?),
                expires_on: r.get(5)?,
                statement_id: r.get(6)?,
                transfer_id: r.get(7)?,
                cost: r.get(8)?,
                cost_currency: r.get(9)?,
                notes: r.get(10)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Lotes que ainda não venceram nem foram usados, em ordem de data: cada saída consome os
/// lotes vivos na data dela, os que vencem antes primeiro (os sem vencimento por último); um
/// lote que já venceu na data da saída não é usado por ela. O registro de expiração consome
/// primeiro os lotes já vencidos. Devolve (vencimento, pontos) dos lotes com saldo e os pontos
/// que venceram sem uso nem registro de expiração (saem do saldo).
pub fn open_lots(list: &[LoyaltyEntry], today: NaiveDate) -> (Vec<(Option<String>, i64)>, i64) {
    // Entradas antes das saídas no mesmo dia.
    let mut events: Vec<&LoyaltyEntry> = list.iter().filter(|e| e.points != 0).collect();
    events.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then((a.points < 0).cmp(&(b.points < 0)))
    });
    // (vencimento, pontos que sobram)
    let mut lots: Vec<(Option<String>, i64)> = vec![];
    let by_expiry = |a: &(Option<String>, i64), b: &(Option<String>, i64)| match (&a.0, &b.0) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    };
    for e in events {
        if e.points > 0 {
            lots.push((e.expires_on.clone(), e.points));
            lots.sort_by(by_expiry);
            continue;
        }
        let mut out = -e.points;
        let alive =
            |exp: &Option<String>| exp.as_ref().is_none_or(|x| x.as_str() >= e.date.as_str());
        // Expiração registrada: primeiro os lotes já vencidos na data; saídas comuns só usam
        // os vivos.
        let passes: &[bool] = if e.kind == EntryKind::Expire {
            &[false, true]
        } else {
            &[true]
        };
        for &want_alive in passes {
            for lot in lots.iter_mut() {
                if out == 0 {
                    break;
                }
                if lot.1 <= 0 || alive(&lot.0) != want_alive {
                    continue;
                }
                let used = out.min(lot.1);
                lot.1 -= used;
                out -= used;
            }
        }
    }
    let today_s = fmt_date(today);
    let mut open = vec![];
    let mut expired = 0;
    for (exp, left) in lots {
        if left <= 0 {
            continue;
        }
        if exp.as_ref().is_none_or(|e| e.as_str() >= today_s.as_str()) {
            open.push((exp, left));
        } else {
            expired += left;
        }
    }
    (open, expired)
}

fn view_of(conn: &Connection, row: ProgramRow, today: NaiveDate) -> Result<LoyaltyProgram> {
    let (id, name, kind, member_id, member_name, unit_label, value, value_currency, active, notes) =
        row;
    let list = entries(conn, &id)?;
    let (lots, expired) = open_lots(&list, today);
    // Venceu sem uso: sai do saldo mesmo sem o registro de expiração.
    let balance: i64 = list.iter().map(|e| e.points).sum::<i64>() - expired;
    let limit = fmt_date(today + Duration::days(90));
    let expiring_90d: i64 = lots
        .iter()
        .filter(|(e, _)| e.as_ref().is_some_and(|e| e.as_str() <= limit.as_str()))
        .map(|(_, p)| p)
        .sum();
    let next = lots.iter().find(|(e, _)| e.is_some()).cloned();
    let (next_expiry, next_expiry_points) = match next {
        Some((Some(d), _)) => {
            let pts: i64 = lots
                .iter()
                .filter(|(e, _)| e.as_deref() == Some(d.as_str()))
                .map(|(_, p)| p)
                .sum();
            (Some(d), pts)
        }
        _ => (None, 0),
    };
    let bought: Vec<&LoyaltyEntry> = list
        .iter()
        .filter(|e| e.kind == EntryKind::Purchase && e.cost.is_some())
        .collect();
    let bought_points: i64 = bought.iter().map(|e| e.points).sum();
    let bought_cost: i64 = bought.iter().filter_map(|e| e.cost).sum();
    let cards: Vec<LinkedCard> = conn
        .prepare_cached(
            "SELECT DISTINCT a.id, a.name FROM card_reward_program r JOIN account a ON a.id = r.account_id
              WHERE r.loyalty_program_id = ?1 AND (r.ends_on IS NULL OR r.ends_on >= ?2)
              ORDER BY a.sort_order, a.name",
        )?
        .query_map(params![id, fmt_date(today)], |r| {
            Ok(LinkedCard {
                account_id: r.get(0)?,
                account_name: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(LoyaltyProgram {
        estimated_value: value.map(|v| (balance.max(0) as i128 * v as i128 / 1000) as i64),
        purchase_cost_per_1000: (bought_points > 0)
            .then(|| (bought_cost as i128 * 1000 / bought_points as i128) as i64),
        id,
        name,
        kind: LoyaltyKind::parse(&kind),
        member_id,
        member_name,
        unit_label,
        value_per_1000: value,
        value_currency,
        active,
        notes,
        balance,
        expiring_90d,
        next_expiry,
        next_expiry_points,
        cards,
    })
}

const PROGRAM_SQL: &str =
    "SELECT p.id, p.name, p.kind, p.member_id, m.name, p.unit_label, p.value_per_1000,
       p.value_currency, p.active, p.notes
  FROM loyalty_program p LEFT JOIN member m ON m.id = p.member_id";

type ProgramRow = (
    Id,
    String,
    String,
    Option<Id>,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<String>,
    bool,
    Option<String>,
);

fn map_program(r: &rusqlite::Row<'_>) -> rusqlite::Result<ProgramRow> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
        r.get::<_, i64>(8)? == 1,
        r.get(9)?,
    ))
}

pub fn list(conn: &Connection, today: NaiveDate) -> Result<Vec<LoyaltyProgram>> {
    let rows: Vec<ProgramRow> = conn
        .prepare_cached(&format!(
            "{PROGRAM_SQL} ORDER BY p.active DESC, m.name, p.name"
        ))?
        .query_map([], map_program)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|r| view_of(conn, r, today)).collect()
}

pub fn get(conn: &Connection, id: &str, today: NaiveDate) -> Result<LoyaltyProgram> {
    let row = conn
        .query_row(&format!("{PROGRAM_SQL} WHERE p.id = ?1"), [id], map_program)
        .optional()?
        .ok_or_else(|| Error::not_found(program_ref(id)))?;
    view_of(conn, row, today)
}

fn clean_name(name: &str) -> Result<String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(Error::invalid(t("nucleo-pontos-erro-nome")));
    }
    Ok(n.chars().take(80).collect())
}

pub fn create_program(tx: &Transaction<'_>, input: &LoyaltyProgramInput) -> Result<Id> {
    let name = clean_name(&input.name)?;
    let member = clean_opt(input.member_id.clone());
    if let Some(m) = &member {
        require_member(tx, m)?;
    }
    if input.value_per_1000.is_some_and(|v| v < 0) {
        return Err(Error::invalid(t("nucleo-pontos-erro-valor")));
    }
    let currency = match clean_opt(input.value_currency.clone()) {
        Some(c) => Some(crate::money::validate_currency(&c.to_ascii_uppercase())?),
        None if input.value_per_1000.is_some() => Some(crate::household::base_currency(tx)?),
        None => None,
    };
    let id = new_id();
    tx.execute(
        "INSERT INTO loyalty_program (id, name, kind, member_id, unit_label, value_per_1000, value_currency,
                                      active, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?9)",
        params![
            id,
            name,
            input.kind.unwrap_or(LoyaltyKind::Bank).as_str(),
            member,
            clean_opt(input.unit_label.clone()),
            input.value_per_1000,
            currency,
            clean_opt(input.notes.clone()),
            now_utc()
        ],
    )?;
    Ok(id)
}

pub fn update_program(tx: &Transaction<'_>, p: &LoyaltyProgramPatch) -> Result<()> {
    require_program(tx, &p.id)?;
    let now = now_utc();
    if let Some(n) = &p.name {
        tx.execute(
            "UPDATE loyalty_program SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, clean_name(n)?, now],
        )?;
    }
    if let Some(k) = p.kind {
        tx.execute(
            "UPDATE loyalty_program SET kind = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, k.as_str(), now],
        )?;
    }
    if let Some(m) = &p.member_id {
        let m = clean_opt(m.clone());
        if let Some(m) = &m {
            require_member(tx, m)?;
        }
        tx.execute(
            "UPDATE loyalty_program SET member_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, m, now],
        )?;
    }
    if let Some(u) = &p.unit_label {
        tx.execute(
            "UPDATE loyalty_program SET unit_label = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, clean_opt(u.clone()), now],
        )?;
    }
    if let Some(v) = p.value_per_1000 {
        if v.is_some_and(|v| v < 0) {
            return Err(Error::invalid(t("nucleo-pontos-erro-valor")));
        }
        let base = crate::household::base_currency(tx)?;
        tx.execute(
            "UPDATE loyalty_program SET value_per_1000 = ?2,
                    value_currency = CASE WHEN ?2 IS NULL THEN NULL ELSE COALESCE(value_currency, ?4) END,
                    updated_at = ?3 WHERE id = ?1",
            params![p.id, v, now, base],
        )?;
    }
    if let Some(a) = p.active {
        tx.execute(
            "UPDATE loyalty_program SET active = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, i64::from(a), now],
        )?;
    }
    if let Some(n) = &p.notes {
        tx.execute(
            "UPDATE loyalty_program SET notes = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, clean_opt(n.clone()), now],
        )?;
    }
    Ok(())
}

pub fn delete_program(tx: &Transaction<'_>, id: &str) -> Result<()> {
    require_program(tx, id)?;
    tx.execute(
        "UPDATE card_reward_program SET loyalty_program_id = NULL WHERE loyalty_program_id = ?1",
        [id],
    )?;
    tx.execute("DELETE FROM loyalty_entry WHERE program_id = ?1", [id])?;
    tx.execute("DELETE FROM loyalty_program WHERE id = ?1", [id])?;
    Ok(())
}

fn insert_entry(
    tx: &Transaction<'_>,
    input: &LoyaltyEntryInput,
    transfer_id: Option<&str>,
) -> Result<Id> {
    require_program(tx, &input.program_id)?;
    parse_date(&input.date)?;
    check_date(&input.expires_on)?;
    if input.points == 0 {
        return Err(Error::invalid(t("nucleo-pontos-erro-pontos")));
    }
    let points = match input.kind.sign() {
        0 => input.points,
        s => input.points.abs() * s,
    };
    if input.cost.is_some_and(|c| c < 0) {
        return Err(Error::invalid(t("nucleo-pontos-erro-valor")));
    }
    let statement = clean_opt(input.statement_id.clone());
    if let Some(s) = &statement
        && !crate::transactions::util::exists(tx, "card_statement", s)?
    {
        return Err(Error::not_found(t("nucleo-pontos-ref-fatura")));
    }
    let cost_currency = match (input.cost, clean_opt(input.cost_currency.clone())) {
        (Some(_), Some(c)) => Some(crate::money::validate_currency(&c.to_ascii_uppercase())?),
        (Some(_), None) => Some(crate::household::base_currency(tx)?),
        _ => None,
    };
    let id = new_id();
    tx.execute(
        "INSERT INTO loyalty_entry (id, program_id, date, points, kind, expires_on, statement_id, transfer_id,
                                    cost, cost_currency, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
        params![
            id,
            input.program_id,
            input.date,
            points,
            input.kind.as_str(),
            if points > 0 { input.expires_on.clone() } else { None },
            statement,
            transfer_id,
            input.cost,
            cost_currency,
            clean_opt(input.notes.clone()),
            now_utc()
        ],
    )?;
    Ok(id)
}

pub fn add_entry(tx: &Transaction<'_>, input: &LoyaltyEntryInput) -> Result<Id> {
    if matches!(input.kind, EntryKind::TransferIn | EntryKind::TransferOut) {
        return Err(Error::invalid(t("nucleo-pontos-erro-transferencia-par")));
    }
    insert_entry(tx, input, None)
}

/// Transferência entre programas (ex.: banco → companhia aérea com bônus): duas entradas
/// ligadas pelo mesmo `transfer_id`.
pub fn transfer(tx: &Transaction<'_>, input: &LoyaltyTransferInput) -> Result<Id> {
    if input.from_program_id == input.to_program_id {
        return Err(Error::invalid(t("nucleo-pontos-erro-mesmo-programa")));
    }
    if input.points_out <= 0 || input.points_in <= 0 {
        return Err(Error::invalid(t("nucleo-pontos-erro-pontos")));
    }
    let tid = new_id();
    insert_entry(
        tx,
        &LoyaltyEntryInput {
            program_id: input.from_program_id.clone(),
            date: input.date.clone(),
            points: input.points_out,
            kind: EntryKind::TransferOut,
            expires_on: None,
            statement_id: None,
            cost: None,
            cost_currency: None,
            notes: input.notes.clone(),
        },
        Some(&tid),
    )?;
    insert_entry(
        tx,
        &LoyaltyEntryInput {
            program_id: input.to_program_id.clone(),
            date: input.date.clone(),
            points: input.points_in,
            kind: EntryKind::TransferIn,
            expires_on: input.expires_on.clone(),
            statement_id: None,
            cost: None,
            cost_currency: None,
            notes: input.notes.clone(),
        },
        Some(&tid),
    )?;
    Ok(tid)
}

/// Apaga um movimento (e o par, se for transferência). Devolve quantos saíram.
pub fn delete_entry(tx: &Transaction<'_>, id: &str) -> Result<usize> {
    let transfer: Option<Option<String>> = tx
        .query_row(
            "SELECT transfer_id FROM loyalty_entry WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    match transfer {
        None => Err(Error::not_found(t("nucleo-pontos-ref-movimento"))),
        Some(Some(tid)) => {
            Ok(tx.execute("DELETE FROM loyalty_entry WHERE transfer_id = ?1", [tid])?)
        }
        Some(None) => Ok(tx.execute("DELETE FROM loyalty_entry WHERE id = ?1", [id])?),
    }
}

/// Pontos creditados por fatura (acúmulos ligados a ela), para comparar com o previsto.
pub fn credited_by_statement(
    conn: &Connection,
    program_id: &str,
) -> Result<std::collections::HashMap<Id, i64>> {
    let mut stmt = conn.prepare_cached(
        "SELECT statement_id, SUM(points) FROM loyalty_entry
          WHERE program_id = ?1 AND statement_id IS NOT NULL AND kind IN ('earn', 'bonus')
          GROUP BY statement_id",
    )?;
    let rows = stmt
        .query_map([program_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}
