//! Recompensas do cartão (docs/design/cards.md, fase 4): cashback e pontos por fatura.
//!
//! Previsto: as compras da fatura (menos estornos; sem anuidade, juros e encargos) vezes a taxa
//! da categoria (ou a básica), com o teto do ciclo. Cashback recebido: crédito com a categoria
//! "Cashback e recompensas" (ou "cashback" na descrição), no cartão ou na conta de destino; cada
//! crédito conta para a última fatura que fechou antes dele. O cashback é receita: o gasto de
//! cada categoria continua sendo o preço pago.

use std::collections::HashMap;
use std::str::FromStr;

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::{Id, new_id};
use crate::time::{add_months, fmt_date, now_utc, parse_date};
use crate::transactions::util::clean_opt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RewardKind {
    Cashback,
    Points,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RewardDestination {
    Statement,
    Account,
    Investment,
}

impl RewardKind {
    fn as_str(self) -> &'static str {
        match self {
            RewardKind::Cashback => "cashback",
            RewardKind::Points => "points",
        }
    }
}

impl RewardDestination {
    fn as_str(self) -> &'static str {
        match self {
            RewardDestination::Statement => "statement",
            RewardDestination::Account => "account",
            RewardDestination::Investment => "investment",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "account" => RewardDestination::Account,
            "investment" => RewardDestination::Investment,
            _ => RewardDestination::Statement,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RewardMultiplier {
    pub category_id: Id,
    pub category_path: Option<String>,
    /// Taxa da categoria (decimal em texto).
    pub rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RewardMultiplierInput {
    pub category_id: Id,
    pub rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RewardProgram {
    pub id: Id,
    pub account_id: Id,
    pub kind: RewardKind,
    /// Cashback: fração (`"0.01"` = 1%). Pontos: pontos por unidade de `rate_currency`.
    pub base_rate: String,
    pub rate_currency: Option<String>,
    /// Teto por fatura: centavos (cashback) ou pontos.
    #[ts(type = "number | null")]
    pub cycle_cap: Option<i64>,
    pub destination: RewardDestination,
    pub destination_account_id: Option<Id>,
    pub payout_days: u32,
    pub starts_on: String,
    pub ends_on: Option<String>,
    pub notes: Option<String>,
    pub multipliers: Vec<RewardMultiplier>,
    /// Pontos: o programa de pontos onde eles caem (fase 5).
    pub loyalty_program_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RewardProgramInput {
    pub account_id: Id,
    pub kind: RewardKind,
    pub base_rate: String,
    #[ts(optional)]
    pub rate_currency: Option<String>,
    #[ts(optional, type = "number")]
    pub cycle_cap: Option<i64>,
    #[ts(optional)]
    pub destination: Option<RewardDestination>,
    #[ts(optional)]
    pub destination_account_id: Option<Id>,
    #[ts(optional)]
    pub payout_days: Option<u32>,
    /// Padrão: hoje.
    #[ts(optional)]
    pub starts_on: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
    #[ts(optional)]
    pub multipliers: Option<Vec<RewardMultiplierInput>>,
    /// Pontos: programa de pontos onde eles caem.
    #[ts(optional)]
    pub loyalty_program_id: Option<Id>,
}

/// Recompensa de uma fatura.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementReward {
    pub statement_id: Id,
    pub reference_month: String,
    pub closing_date: String,
    /// Compras que dão recompensa, menos estornos (centavos).
    #[ts(type = "number")]
    pub eligible_spend: i64,
    /// Previsto: centavos (cashback) ou pontos.
    #[ts(type = "number | null")]
    pub expected: Option<i64>,
    /// Cashback recebido que conta para esta fatura (centavos); pontos: `null`.
    #[ts(type = "number | null")]
    pub received: Option<i64>,
    /// Data até quando o crédito costuma cair.
    pub due_by: String,
    /// Passou do prazo e veio menos da metade do previsto.
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RewardsView {
    pub account_id: Id,
    pub currency: String,
    pub program: Option<RewardProgram>,
    /// Faturas dos últimos 12 meses (a em aberto incluída), da mais recente para trás.
    pub statements: Vec<StatementReward>,
    #[ts(type = "number")]
    pub expected_12m: i64,
    #[ts(type = "number")]
    pub received_12m: i64,
}

fn rate(s: &str) -> Result<Decimal> {
    let d = Decimal::from_str(s.trim().replace(',', ".").as_str())
        .map_err(|_| Error::invalid(t("nucleo-contas-recompensa-erro-taxa")))?;
    if d < Decimal::ZERO || d > Decimal::from(1000) {
        return Err(Error::invalid(t("nucleo-contas-recompensa-erro-taxa")));
    }
    Ok(d)
}

/// Taxa em texto (decimal; aceita vírgula).
pub(crate) fn parse_rate(s: &str) -> Result<Decimal> {
    rate(s)
}

const COLS: &str = "id, account_id, kind, base_rate, rate_currency, cycle_cap, destination,
                    destination_account_id, payout_days, starts_on, ends_on, notes, loyalty_program_id";

fn map(r: &rusqlite::Row<'_>) -> rusqlite::Result<RewardProgram> {
    let kind: String = r.get(2)?;
    Ok(RewardProgram {
        id: r.get(0)?,
        account_id: r.get(1)?,
        kind: if kind == "points" {
            RewardKind::Points
        } else {
            RewardKind::Cashback
        },
        base_rate: r.get(3)?,
        rate_currency: r.get(4)?,
        cycle_cap: r.get(5)?,
        destination: RewardDestination::parse(&r.get::<_, String>(6)?),
        destination_account_id: r.get(7)?,
        payout_days: r.get(8)?,
        starts_on: r.get(9)?,
        ends_on: r.get(10)?,
        notes: r.get(11)?,
        multipliers: vec![],
        loyalty_program_id: r.get(12)?,
    })
}

fn load_multipliers(conn: &Connection, p: &mut RewardProgram) -> Result<()> {
    let paths = crate::categories::path_map(conn)?;
    let mut stmt = conn.prepare_cached(
        "SELECT category_id, rate FROM card_reward_multiplier WHERE program_id = ?1 ORDER BY rate DESC",
    )?;
    p.multipliers = stmt
        .query_map([&p.id], |r| {
            let category_id: String = r.get(0)?;
            Ok(RewardMultiplier {
                category_path: paths.get(&category_id).cloned(),
                category_id,
                rate: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(())
}

/// Programa em vigor numa data.
pub fn active(conn: &Connection, account_id: &str, date: &str) -> Result<Option<RewardProgram>> {
    let p = conn
        .prepare_cached(&format!(
            "SELECT {COLS} FROM card_reward_program WHERE account_id = ?1 AND starts_on <= ?2
               AND (ends_on IS NULL OR ends_on >= ?2) ORDER BY starts_on DESC LIMIT 1"
        ))?
        .query_row(params![account_id, date], map)
        .optional()?;
    match p {
        Some(mut p) => {
            load_multipliers(conn, &mut p)?;
            Ok(Some(p))
        }
        None => Ok(None),
    }
}

/// Grava um programa novo; o anterior em vigor termina no dia anterior.
pub fn set(tx: &Transaction<'_>, input: &RewardProgramInput, today: NaiveDate) -> Result<Id> {
    super::require_card(tx, &input.account_id)?;
    let base = rate(&input.base_rate)?;
    if input.kind == RewardKind::Cashback && base > Decimal::ONE {
        return Err(Error::invalid(t("nucleo-contas-recompensa-erro-taxa")));
    }
    let rate_currency = match clean_opt(input.rate_currency.clone()) {
        Some(c) => Some(crate::money::validate_currency(&c.to_ascii_uppercase())?),
        None => None,
    };
    if input.cycle_cap.is_some_and(|c| c < 0) {
        return Err(Error::invalid(t("nucleo-contas-recompensa-erro-teto")));
    }
    let payout = input.payout_days.unwrap_or(40);
    if payout > 180 {
        return Err(Error::invalid(t("nucleo-contas-recompensa-erro-prazo")));
    }
    let destination = input.destination.unwrap_or(RewardDestination::Statement);
    let dest_account = clean_opt(input.destination_account_id.clone());
    if let Some(a) = &dest_account {
        crate::accounts::currency_and_kind(tx, a)?;
    }
    let loyalty = match input.kind {
        RewardKind::Points => clean_opt(input.loyalty_program_id.clone()),
        RewardKind::Cashback => None,
    };
    if let Some(l) = &loyalty {
        super::loyalty::require_program(tx, l)?;
    }
    let starts = input.starts_on.clone().unwrap_or_else(|| fmt_date(today));
    let start_date = parse_date(&starts)?;
    let now = now_utc();
    tx.execute(
        "DELETE FROM card_reward_program WHERE account_id = ?1 AND starts_on = ?2",
        params![input.account_id, starts],
    )?;
    tx.execute(
        "UPDATE card_reward_program SET ends_on = ?3, updated_at = ?4
          WHERE account_id = ?1 AND starts_on < ?2 AND (ends_on IS NULL OR ends_on >= ?2)",
        params![
            input.account_id,
            starts,
            fmt_date(start_date.pred_opt().expect("data válida")),
            now
        ],
    )?;
    let id = new_id();
    tx.execute(
        &format!(
            "INSERT INTO card_reward_program ({COLS}, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, ?11, ?13, ?12, ?12)"
        ),
        params![
            id,
            input.account_id,
            input.kind.as_str(),
            base.normalize().to_string(),
            rate_currency,
            input.cycle_cap,
            destination.as_str(),
            dest_account,
            payout,
            starts,
            clean_opt(input.notes.clone()),
            now,
            loyalty
        ],
    )?;
    for m in input.multipliers.iter().flatten() {
        let r = rate(&m.rate)?;
        crate::transactions::check_category(
            tx,
            Some(&m.category_id),
            crate::transactions::TxnKind::Expense,
        )?;
        tx.execute(
            "INSERT INTO card_reward_multiplier (id, program_id, category_id, rate, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT (program_id, category_id) DO UPDATE SET rate = excluded.rate",
            params![new_id(), id, m.category_id, r.normalize().to_string(), now],
        )?;
    }
    Ok(id)
}

/// Encerra o programa do cartão a partir de hoje.
pub fn end(tx: &Transaction<'_>, account_id: &str, today: NaiveDate) -> Result<usize> {
    let today_s = fmt_date(today);
    tx.execute(
        "DELETE FROM card_reward_program WHERE account_id = ?1 AND starts_on > ?2",
        params![account_id, today_s],
    )?;
    Ok(tx.execute(
        "UPDATE card_reward_program SET ends_on = ?2, updated_at = ?3
          WHERE account_id = ?1 AND (ends_on IS NULL OR ends_on > ?2)",
        params![account_id, today_s, now_utc()],
    )?)
}

/// Id da categoria "Cashback e recompensas", se existir no cofre.
fn cashback_category(conn: &Connection) -> Result<Option<Id>> {
    let key = crate::categories::seed_key(Some("Reembolsos"), "Cashback e recompensas");
    Ok(conn
        .query_row("SELECT id FROM category WHERE seed_key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?)
}

/// Taxa de uma compra: a da categoria (ou do pai dela) ou a básica.
fn rate_for(
    rates: &HashMap<String, Decimal>,
    base: Decimal,
    category: Option<&str>,
    parent: Option<&str>,
) -> Decimal {
    category
        .and_then(|c| rates.get(c))
        .or_else(|| parent.and_then(|c| rates.get(c)))
        .copied()
        .unwrap_or(base)
}

/// Previsto de uma fatura: compras elegíveis e a recompensa (centavos ou pontos).
fn expected_for(
    conn: &Connection,
    p: &RewardProgram,
    statement_id: &str,
    closing_date: &str,
    card_currency: &str,
) -> Result<(i64, Option<i64>)> {
    let base = rate(&p.base_rate)?;
    let rates: HashMap<String, Decimal> = p
        .multipliers
        .iter()
        .filter_map(|m| rate(&m.rate).ok().map(|r| (m.category_id.clone(), r)))
        .collect();
    let mut stmt = conn.prepare_cached(
        "SELECT t.amount, t.description, t.category_id, c.parent_id FROM txn t
           LEFT JOIN category c ON c.id = t.category_id
          WHERE t.statement_id = ?1 AND t.kind = 'expense'",
    )?;
    let mut q = stmt.query([statement_id])?;
    let mut spend = 0i64;
    let mut reward = Decimal::ZERO;
    while let Some(r) = q.next()? {
        let (amount, desc, cat, parent): (i64, String, Option<String>, Option<String>) =
            (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?);
        let d = crate::transactions::normalize_description(&desc);
        if d.contains("anuidade") || super::revolving::is_charge(&desc) || d.contains("cashback") {
            continue;
        }
        spend -= amount;
        let rt = rate_for(&rates, base, cat.as_deref(), parent.as_deref());
        reward += Decimal::from(-amount) * rt;
    }
    let value = match p.kind {
        RewardKind::Cashback => reward.round().to_i64(),
        RewardKind::Points => {
            // Pontos por unidade (não por centavo) da moeda da regra.
            let units = match p.rate_currency.as_deref() {
                Some(c) if c != card_currency => {
                    match crate::fx::convert(conn, 10_000, card_currency, c, closing_date)? {
                        Some(conv) => reward * Decimal::from(conv.amount) / Decimal::from(10_000),
                        None => return Ok((spend, None)),
                    }
                }
                _ => reward,
            };
            (units / Decimal::from(100)).floor().to_i64()
        }
    };
    let value = value.map(|v| match p.cycle_cap {
        Some(cap) => v.min(cap),
        None => v,
    });
    Ok((spend, value.map(|v| v.max(0))))
}

/// Cashback recebido por fatura: cada crédito conta para a última fatura que fechou antes dele.
fn received_by_statement(
    conn: &Connection,
    p: &RewardProgram,
    closings: &[(Id, String)],
    from: &str,
) -> Result<HashMap<Id, i64>> {
    let cat = cashback_category(conn)?;
    let mut accounts = vec![p.account_id.clone()];
    accounts.extend(p.destination_account_id.clone());
    let mut out: HashMap<Id, i64> = HashMap::new();
    for acc in accounts {
        let mut stmt = conn.prepare_cached(
            "SELECT date, amount, description, category_id FROM txn
              WHERE account_id = ?1 AND amount > 0 AND date >= ?2 AND kind IN ('income', 'expense', 'adjustment')",
        )?;
        let mut q = stmt.query(params![acc, from])?;
        while let Some(r) = q.next()? {
            let (date, amount, desc, c): (String, i64, String, Option<String>) =
                (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?);
            let is_cb = (cat.is_some() && c == cat)
                || crate::transactions::normalize_description(&desc).contains("cashback");
            if !is_cb {
                continue;
            }
            if let Some((id, _)) = closings
                .iter()
                .rev()
                .find(|(_, c)| c.as_str() < date.as_str())
            {
                *out.entry(id.clone()).or_default() += amount;
            }
        }
    }
    Ok(out)
}

pub fn view(conn: &Connection, account_id: &str, today: NaiveDate) -> Result<RewardsView> {
    super::require_card(conn, account_id)?;
    let (currency, _) = crate::accounts::currency_and_kind(conn, account_id)?;
    let today_s = fmt_date(today);
    let program = active(conn, account_id, &today_s)?;
    let from = fmt_date(add_months(today, -12));
    let mut statements = vec![];
    let (mut expected_12m, mut received_12m) = (0, 0);
    if let Some(p) = &program {
        // Faturas que já começaram (as futuras só têm parcelas previstas).
        let all: Vec<(Id, String, String)> = conn
            .prepare_cached(
                "SELECT id, reference_month, closing_date FROM card_statement
                  WHERE account_id = ?1 AND period_start <= ?2 ORDER BY closing_date",
            )?
            .query_map(params![account_id, today_s], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        let closings: Vec<(Id, String)> = all
            .iter()
            .filter(|(_, _, c)| c.as_str() < today_s.as_str())
            .map(|(id, _, c)| (id.clone(), c.clone()))
            .collect();
        let received = match (p.kind, &p.loyalty_program_id) {
            (RewardKind::Cashback, _) => received_by_statement(conn, p, &closings, &from)?,
            (RewardKind::Points, Some(l)) => super::loyalty::credited_by_statement(conn, l)?,
            (RewardKind::Points, None) => HashMap::new(),
        };
        let tracks = p.kind == RewardKind::Cashback || p.loyalty_program_id.is_some();
        for (id, month, closing) in all.iter().rev() {
            if closing.as_str() < from.as_str() || closing.as_str() < p.starts_on.as_str() {
                continue;
            }
            let (spend, expected) = expected_for(conn, p, id, closing, &currency)?;
            let due_by = parse_date(closing)? + Duration::days(i64::from(p.payout_days));
            let got = tracks.then(|| received.get(id).copied().unwrap_or(0));
            let missing = tracks
                && due_by < today
                && expected.is_some_and(|e| e > 0 && got.unwrap_or(0) * 2 < e);
            expected_12m += expected.unwrap_or(0);
            received_12m += got.unwrap_or(0);
            statements.push(StatementReward {
                statement_id: id.clone(),
                reference_month: month.clone(),
                closing_date: closing.clone(),
                eligible_spend: spend,
                expected,
                received: got,
                due_by: fmt_date(due_by),
                missing,
            });
        }
    }
    Ok(RewardsView {
        account_id: account_id.to_string(),
        currency,
        program,
        statements,
        expected_12m,
        received_12m,
    })
}
