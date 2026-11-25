//! Closing an account (docs/account-closing.md): a checking, brokerage or card account the
//! family closed at the bank. Closing records the date (`account.closed_on`) and archives the
//! account, so it leaves the lists and every alert that already skips archived accounts. The
//! history stays: reports and net worth count the account on the dates it had a balance.
//!
//! An account closes only when it is empty on the date: zero balance, no position at the
//! broker, no card installment still to come and nothing recorded after the date. Closing
//! pauses the recurrences that move money in or out of it and removes its planned
//! transactions (they will not happen any more). After the date, no transaction can be
//! recorded in it until it is reopened.

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{account_ref, currency_and_kind};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, now_utc, parse_date};
use crate::transactions::util::date_text;

/// Why an account cannot be closed on the date (all of them are listed in the preview).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, rename = "AccountCloseBlocker")]
pub enum CloseBlocker {
    /// The date is after today.
    FutureDate,
    /// The date is before the account's opening date.
    BeforeOpening { opening_date: String },
    /// The account is already closed.
    AlreadyClosed { closed_on: String },
    /// Balance on the date in the account's currency (cleared and reconciled transactions).
    NonZeroBalance {
        #[ts(type = "number")]
        balance: i64,
        currency: String,
    },
    /// Transactions or trades recorded after the date.
    ActivityAfter {
        #[ts(type = "number")]
        count: i64,
        last_date: String,
    },
    /// Assets still held in the account (ticker or name).
    OpenPositions { assets: Vec<String> },
    /// Card installments still to come.
    PendingInstallments {
        #[ts(type = "number")]
        count: i64,
    },
}

/// A planned transaction the closing removes.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AccountClosePlanned")]
pub struct PlannedItem {
    pub id: Id,
    pub date: String,
    pub description: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
}

/// Something linked to the account by id and name (recurrence, loan, card).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AccountCloseLink")]
pub struct Linked {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountCloseArgs {
    pub id: Id,
    /// Closing date `YYYY-MM-DD` (default: today).
    #[ts(optional)]
    pub date: Option<String>,
}

/// What closing the account on the date does, and what stops it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountClosePreview {
    pub account_id: Id,
    pub date: String,
    /// Balance on the date, in the account's currency.
    #[ts(type = "number")]
    pub balance: i64,
    pub currency: String,
    /// Empty: the account can close on the date.
    pub blockers: Vec<CloseBlocker>,
    /// Planned transactions that will be removed (both legs of a planned transfer).
    pub planned: Vec<PlannedItem>,
    /// Active recurrences that will be paused (from or to this account).
    pub recurrences: Vec<Linked>,
    /// Loans whose installments are debited from this account (change the account there).
    pub loans: Vec<Linked>,
    /// Cards whose statement is paid from this account (change the paying account there).
    pub cards_paid: Vec<Linked>,
}

/// The account's closing date, if it is closed.
pub fn closed_on(conn: &Connection, account_id: &str) -> Result<Option<String>> {
    Ok(conn
        .prepare_cached("SELECT closed_on FROM account WHERE id = ?1")?
        .query_row([account_id], |r| r.get::<_, Option<String>>(0))
        .optional()?
        .flatten())
}

/// Refuses a transaction dated after the account's closing date. Called on every insert and
/// edit of a transaction (imports, recurrences, the agent's proposals included).
pub fn check_open_on(conn: &Connection, account_id: &str, date: &str) -> Result<()> {
    let row: Option<(Option<String>, String)> = conn
        .prepare_cached("SELECT closed_on, name FROM account WHERE id = ?1")?
        .query_row([account_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;
    match row {
        Some((Some(closed), name)) if date > closed.as_str() => Err(Error::conflict(t_args(
            "nucleo-contas-erro-encerrada",
            &args(&[("nome", name.into()), ("data", date_text(&closed).into())]),
        ))),
        _ => Ok(()),
    }
}

/// Ids of the planned transactions the closing removes: every planned one in the account
/// (except card installments, which block the closing) and the other leg of planned
/// transfers.
fn planned_ids(conn: &Connection, account_id: &str) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM txn
          WHERE status = 'planned' AND installment_plan_id IS NULL
            AND (account_id = ?1
                 OR transfer_id IN (SELECT transfer_id FROM txn
                                     WHERE account_id = ?1 AND status = 'planned'
                                       AND installment_plan_id IS NULL AND transfer_id IS NOT NULL))
          ORDER BY date, id",
    )?;
    Ok(stmt
        .query_map([account_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

fn linked(conn: &Connection, sql: &str, account_id: &str) -> Result<Vec<Linked>> {
    let mut stmt = conn.prepare(sql)?;
    Ok(stmt
        .query_map([account_id], |r| {
            Ok(Linked {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

const RECURRENCES_SQL: &str = "SELECT id, description FROM recurrence
  WHERE active = 1 AND (account_id = ?1 OR to_account_id = ?1) ORDER BY description, id";

/// Assets with a position in the account on the date or today (a trade after the date counts).
fn open_positions(conn: &Connection, account_id: &str, dates: &[NaiveDate]) -> Result<Vec<String>> {
    let only = [account_id.to_string()];
    let mut out = vec![];
    let mut seen = std::collections::HashSet::new();
    for (asset_id, _) in crate::portfolio::engine::traded_assets(conn, Some(&only))? {
        if !seen.insert(asset_id.clone()) {
            continue;
        }
        let book = crate::portfolio::engine::load_book(conn, &asset_id, Some(&only))?;
        let mut held = false;
        for d in dates {
            if crate::portfolio::engine::replay(conn, &book, *d)?.quantity() > Decimal::ZERO {
                held = true;
                break;
            }
        }
        if held {
            let label: String = conn.query_row(
                "SELECT COALESCE(ticker, name) FROM asset WHERE id = ?1",
                [&asset_id],
                |r| r.get(0),
            )?;
            out.push(label);
        }
    }
    out.sort();
    Ok(out)
}

/// What closing the account on `date` (default today) does and what stops it.
pub fn preview(
    conn: &Connection,
    a: &AccountCloseArgs,
    today: NaiveDate,
) -> Result<AccountClosePreview> {
    let date = match &a.date {
        Some(d) => parse_date(d)?,
        None => today,
    };
    let date_s = fmt_date(date);
    let (currency, _kind) = currency_and_kind(conn, &a.id)?;
    let (opening_date, closed): (String, Option<String>) = conn.query_row(
        "SELECT opening_date, closed_on FROM account WHERE id = ?1",
        [&a.id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let mut blockers = vec![];
    if let Some(c) = closed {
        blockers.push(CloseBlocker::AlreadyClosed { closed_on: c });
    }
    if date > today {
        blockers.push(CloseBlocker::FutureDate);
    }
    if date_s < opening_date {
        blockers.push(CloseBlocker::BeforeOpening { opening_date });
    }
    let balance = super::balance(conn, &a.id, &date_s, false)?;
    if balance != 0 {
        blockers.push(CloseBlocker::NonZeroBalance {
            balance,
            currency: currency.clone(),
        });
    }
    let (count, last): (i64, Option<String>) = conn.query_row(
        "SELECT count(*), max(d) FROM (
            SELECT date AS d FROM txn WHERE account_id = ?1 AND status <> 'planned' AND date > ?2
            UNION ALL SELECT date FROM trade WHERE account_id = ?1 AND date > ?2)",
        params![a.id, date_s],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if let (true, Some(last_date)) = (count > 0, last) {
        blockers.push(CloseBlocker::ActivityAfter { count, last_date });
    }
    let positions = open_positions(conn, &a.id, &[date, today.max(date)])?;
    if !positions.is_empty() {
        blockers.push(CloseBlocker::OpenPositions { assets: positions });
    }
    let installments: i64 = conn.query_row(
        "SELECT count(*) FROM txn WHERE account_id = ?1 AND status = 'planned'
            AND installment_plan_id IS NOT NULL",
        [&a.id],
        |r| r.get(0),
    )?;
    if installments > 0 {
        blockers.push(CloseBlocker::PendingInstallments {
            count: installments,
        });
    }
    let planned = planned_ids(conn, &a.id)?
        .iter()
        .map(|id| {
            let r = crate::transactions::load_row(conn, id)?;
            Ok(PlannedItem {
                id: r.id,
                date: r.date,
                description: r.description,
                amount: r.amount,
                currency: r.currency,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(AccountClosePreview {
        account_id: a.id.clone(),
        date: date_s,
        balance,
        currency,
        blockers,
        planned,
        recurrences: linked(conn, RECURRENCES_SQL, &a.id)?,
        loans: linked(
            conn,
            "SELECT id, name FROM loan WHERE debit_account_id = ?1 AND archived = 0 ORDER BY name, id",
            &a.id,
        )?,
        cards_paid: linked(
            conn,
            "SELECT a.id, a.name FROM credit_card c JOIN account a ON a.id = c.account_id
              WHERE c.payment_account_id = ?1 AND a.archived = 0 ORDER BY a.name, a.id",
            &a.id,
        )?,
    })
}

/// The message of a blocker, for the error when closing anyway.
pub fn blocker_text(b: &CloseBlocker) -> String {
    match b {
        CloseBlocker::FutureDate => t("nucleo-contas-encerrar-erro-futuro"),
        CloseBlocker::BeforeOpening { opening_date } => t_args(
            "nucleo-contas-encerrar-erro-antes-abertura",
            &args(&[("data", date_text(opening_date).into())]),
        ),
        CloseBlocker::AlreadyClosed { closed_on } => t_args(
            "nucleo-contas-encerrar-erro-ja-encerrada",
            &args(&[("data", date_text(closed_on).into())]),
        ),
        CloseBlocker::NonZeroBalance { balance, currency } => t_args(
            "nucleo-contas-encerrar-erro-saldo",
            &args(&[(
                "valor",
                crate::i18n::money(crate::i18n::current(), *balance, currency).into(),
            )]),
        ),
        CloseBlocker::ActivityAfter { count, last_date } => t_args(
            "nucleo-contas-encerrar-erro-depois",
            &args(&[
                ("n", (*count).into()),
                ("data", date_text(last_date).into()),
            ]),
        ),
        CloseBlocker::OpenPositions { assets } => t_args(
            "nucleo-contas-encerrar-erro-posicoes",
            &args(&[("ativos", assets.join(", ").into())]),
        ),
        CloseBlocker::PendingInstallments { count } => t_args(
            "nucleo-contas-encerrar-erro-parcelas",
            &args(&[("n", (*count).into())]),
        ),
    }
}

/// What the closing did.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountCloseResult {
    #[ts(type = "number")]
    pub planned_removed: i64,
    #[ts(type = "number")]
    pub recurrences_paused: i64,
}

/// Closes the account on the date: refuses when the preview has a blocker, pauses the
/// recurrences, removes the planned transactions and archives the account.
pub fn close(
    tx: &Transaction<'_>,
    a: &AccountCloseArgs,
    today: NaiveDate,
) -> Result<AccountCloseResult> {
    let p = preview(tx, a, today)?;
    if let Some(b) = p.blockers.first() {
        return Err(Error::conflict(blocker_text(b)));
    }
    let mut recurrences_paused = 0;
    for r in &p.recurrences {
        let (start, end): (String, Option<String>) = tx.query_row(
            "SELECT start_date, end_date FROM recurrence WHERE id = ?1",
            [&r.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        // The last occurrence is the closing date (an earlier end date stays).
        let end_date = (start <= p.date && end.as_deref().is_none_or(|e| e > p.date.as_str()))
            .then(|| Some(p.date.clone()));
        crate::recurrence::update(
            tx,
            &crate::recurrence::RecurrencePatch {
                id: r.id.clone(),
                description: None,
                amount: None,
                category_id: None,
                member_id: None,
                spec: None,
                end_date,
                active: Some(false),
                update_planned: Some(true),
            },
            today,
        )?;
        recurrences_paused += 1;
    }
    // Pausing removed the recurrences' planned items from today on; the rest goes now.
    let rows = planned_ids(tx, &a.id)?
        .iter()
        .map(|id| crate::transactions::load_row(tx, id))
        .collect::<Result<Vec<_>>>()?;
    crate::transactions::delete_rows(tx, &rows, today)?;
    tx.execute(
        "UPDATE account SET closed_on = ?2, archived = 1, updated_at = ?3 WHERE id = ?1",
        params![a.id, p.date, now_utc()],
    )?;
    Ok(AccountCloseResult {
        planned_removed: p.planned.len() as i64,
        recurrences_paused,
    })
}

/// Reopens a closed account: clears the date and the archive flag (`archived` says whether it
/// stays archived). The paused recurrences stay paused.
pub fn reopen(tx: &Transaction<'_>, id: &str, archived: bool) -> Result<()> {
    let n = tx.execute(
        "UPDATE account SET closed_on = NULL, archived = ?2, updated_at = ?3
          WHERE id = ?1 AND (closed_on IS NOT NULL OR archived IS NOT ?2)",
        params![id, archived, now_utc()],
    )?;
    if n == 0 && !crate::transactions::util::exists(tx, "account", id)? {
        return Err(Error::not_found(account_ref(id)));
    }
    Ok(())
}
