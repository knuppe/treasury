//! Income ledger: what each account was entitled to, distribution by distribution, against
//! what is recorded.
//!
//! - **Expected**: for each distribution of the asset ([`super::pending::all_candidates`]: the
//!   fund's Fundos.NET notice, B3's official events, the agenda), the position of the account at
//!   the end of the record date ("data-com"; the day before the ex date for the sources that only
//!   report the ex date) times the amount per share. Trades count on the trade date, and splits,
//!   reverse splits, bonus shares and ticker changes come from the portfolio engine, so the
//!   quantity is always in the unit of that date and no amount is adjusted.
//! - **Amounts**: gross = quantity × amount per share rounded to the cent; tax withheld by
//!   [`super::records::withholding`] (JCP at the rate of the credit date, fund income only when
//!   the notice says it is not exempt), dropping the fraction of a cent as the custodians do.
//! - **Matching**: each recorded distribution pairs with at most one expected one of the same
//!   family (JCP, amortization, other income), with the record date at most two business days
//!   apart (or, without a record date, the payment within the window of the expected one). What
//!   repeats an already paired payment is a duplicate; what pairs with nothing is unexpected.
//! - **States and fixes**: `matches`; `tax_diff` (same net, gross and tax recorded with another
//!   rate: fix the amounts); `value_diff` (fix the amounts, except when the record came from the
//!   B3 statement, whose credited amount wins); `missing` (record it); `duplicate` (delete it);
//!   `upcoming` (paid after today); `unexpected` (nothing to propose).
//!
//! - **Cash**: a record made as received without a cash entry in the account did not move the
//!   brokerage balance. Paid after the history cut (the start of the automatic recording, or the
//!   last [`CASH_PENDING_DAYS`] days while it is off), it is `cash_pending`: the amounts may match
//!   the source and the credit still has to be checked in the statement. Older ones are history,
//!   recorded without cash on purpose, and only carry a note.
//!
//! Totals count, per payment month and in the family's base currency, the expected amount of the
//! paid distributions (the recorded one when it came from the B3 statement or nothing official
//! describes it), so they show what the family received even before the fixes are applied.

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{Book, brief_of, entitled_on, fx_to_base, load_book, replay_observed};
use super::pending::{self, Candidate, kind_group, near};
use super::records::{self, to_minor};
use super::{AssetBrief, IncomeInput, IncomeKind, IncomeStatus};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, parse_date};

/// Automatic recording of the official distributions: `{ "since": "AAAA-MM-DD" }` once the
/// family rebuilt the history (absent = off).
pub const AUTO_SETTING: &str = "portfolio.income_auto";
/// Days before the start of the automatic recording whose payments are history: recorded without
/// a brokerage cash entry, like the rebuild.
const HISTORY_GRACE_DAYS: i64 = 7;
/// A payment date this far in the future is a typo.
const IMPLAUSIBLE_DAYS: i64 = 400;
/// Without a record date, a recorded payment pairs with an expected one paid up to this many
/// days after the record date.
const PAY_WINDOW_DAYS: i64 = 90;
/// Amount difference treated as the custodian's rounding, in the smallest unit.
const ROUNDING: i64 = 1;
/// With the automatic recording off, a record without a cash entry paid up to this many days ago
/// still waits for its credit; older ones are history.
const CASH_PENDING_DAYS: i64 = 90;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoState {
    pub since: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "IncomeLedgerState")]
pub enum EntryState {
    /// Recorded as the source says.
    Matches,
    /// Paid and not recorded.
    Missing,
    /// Payment after today ("a receber").
    Upcoming,
    /// Recorded with another net amount.
    ValueDiff,
    /// Same net, gross and tax recorded with another rate.
    TaxDiff,
    /// Repeats a payment that is already recorded.
    Duplicate,
    /// Recorded without any source describing it.
    Unexpected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "IncomeLedgerFix")]
pub enum Fix {
    /// Record the distribution.
    Create,
    /// Fix the gross, the tax and the quantity of the record.
    Update,
    /// Delete the repeated record.
    Delete,
}

/// A recorded distribution as the ledger sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeLedgerRecorded")]
pub struct Recorded {
    pub id: Id,
    pub kind: IncomeKind,
    pub ex_date: Option<String>,
    pub pay_date: String,
    pub quantity: Option<String>,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub tax_withheld: i64,
    #[ts(type = "number")]
    pub net: i64,
    pub status: IncomeStatus,
    /// `manual`, `b3` (statement import), `official`, `agenda`…
    pub source: String,
    /// It moved the brokerage cash.
    pub cash: bool,
}

impl Recorded {
    /// Credited amount reported by B3 (statement import): it wins over the calculation.
    fn credited(&self) -> bool {
        self.source == "b3"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeLedgerEntry")]
pub struct LedgerEntry {
    /// Stable key: the distribution's reference and the account (`<ref>|<account>`), or
    /// `rec:<income id>` for a record that pairs with nothing expected.
    pub key: String,
    pub asset: AssetBrief,
    pub account_id: Id,
    pub kind: IncomeKind,
    /// Record date (data-com) or, for the sources that only have it, the ex date.
    pub ex_date: Option<String>,
    pub pay_date: Option<String>,
    /// No source gave the payment date: the record date is used.
    pub pay_date_estimated: bool,
    /// Position on the record date.
    pub quantity: Option<String>,
    pub amount_per_share: Option<String>,
    pub currency: String,
    #[ts(type = "number | null")]
    pub expected_gross: Option<i64>,
    #[ts(type = "number | null")]
    pub expected_tax: Option<i64>,
    #[ts(type = "number | null")]
    pub expected_net: Option<i64>,
    pub recorded: Option<Recorded>,
    pub state: EntryState,
    pub fix: Option<Fix>,
    /// The reference is official (Fundos.NET or B3).
    pub official: bool,
    /// Reference source.
    pub source: Option<String>,
    /// Reference month, as the fund's notice writes it.
    pub period: Option<String>,
    /// Fund income tax exempt, as the notice says.
    pub exempt: Option<bool>,
    pub check: Option<pending::Check>,
    pub note: Option<String>,
    /// Amount counted in the totals (base currency): 0 for duplicates and upcoming.
    #[ts(type = "number")]
    pub net_base: i64,
    /// Upcoming amount (base currency).
    #[ts(type = "number")]
    pub upcoming_base: i64,
    /// Recorded as received without a cash entry in the account, paid after the history cut: the
    /// amounts may match the source, and the credit is still to be checked in the statement
    /// (`portfolio.income_cash` records it).
    pub cash_pending: bool,
}

impl LedgerEntry {
    /// Month and amount that count in the totals.
    fn month(&self) -> Option<String> {
        let d = self
            .pay_date
            .as_deref()
            .or(self.ex_date.as_deref())
            .or(self.recorded.as_ref().map(|r| r.pay_date.as_str()))?;
        Some(d.get(..7)?.to_string())
    }

    /// A record that no source describes, dated outside what a payment can be (a typo in the
    /// year): too far ahead of `today`, or before any record the app accepts.
    fn implausible(&self, today: NaiveDate) -> bool {
        self.expected_net.is_none()
            && self
                .recorded
                .as_ref()
                .and_then(|r| d(&r.pay_date))
                .is_some_and(|p| {
                    (p - today).num_days() > IMPLAUSIBLE_DAYS
                        || p.year() < crate::time::MIN_RECORD_YEAR
                })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeLedgerPeriod")]
pub struct PeriodTotal {
    /// `AAAA` (year) or `AAAA-MM` (month).
    pub period: String,
    #[ts(type = "number")]
    pub received_base: i64,
    #[ts(type = "number")]
    pub upcoming_base: i64,
    /// What is recorded today in the period (base currency).
    #[ts(type = "number")]
    pub recorded_base: i64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeLedgerAsset")]
pub struct AssetTotal {
    pub asset: AssetBrief,
    /// First trade of the asset.
    pub first_date: Option<String>,
    #[ts(type = "number")]
    pub received_base: i64,
    #[ts(type = "number")]
    pub upcoming_base: i64,
    #[ts(type = "number")]
    pub recorded_base: i64,
    pub years: Vec<PeriodTotal>,
    /// Entries to check (with a fix, official or not).
    pub fixes: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeLedgerCounts")]
pub struct Counts {
    pub matches: u32,
    pub missing: u32,
    pub upcoming: u32,
    pub value_diff: u32,
    pub tax_diff: u32,
    pub duplicate: u32,
    pub unexpected: u32,
    /// Entries with a fix to apply (what the rebuild proposes).
    pub fixes: u32,
    /// Missing entries without an official source (they wait for the family's confirmation).
    pub unofficial_missing: u32,
    /// Recorded as received without a cash entry in the account (after the history cut).
    pub cash_pending: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeLedger")]
pub struct Ledger {
    pub base_currency: String,
    /// Newest first.
    pub entries: Vec<LedgerEntry>,
    pub assets: Vec<AssetTotal>,
    pub years: Vec<PeriodTotal>,
    pub months: Vec<PeriodTotal>,
    #[ts(type = "number")]
    pub received_base: i64,
    #[ts(type = "number")]
    pub upcoming_base: i64,
    #[ts(type = "number")]
    pub recorded_base: i64,
    pub counts: Counts,
    /// Automatic recording of the official distributions is on (after the first rebuild).
    pub auto: bool,
    pub auto_since: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeLedgerArgs")]
pub struct LedgerArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Only distributions paid (or with the record date) from this date on.
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
    /// Leave out the entries that match (only totals, fixes and upcoming).
    #[serde(default)]
    #[ts(optional)]
    pub only_problems: Option<bool>,
}

pub fn auto_state(conn: &Connection) -> Result<Option<AutoState>> {
    crate::settings::get(conn, AUTO_SETTING)
}

// ---------------------------------------------------------------------------------------
// Expected distributions
// ---------------------------------------------------------------------------------------

/// Position after each step of the book, to read the quantity on any date.
struct Timeline(Vec<(NaiveDate, Decimal)>);

impl Timeline {
    fn of(conn: &Connection, book: &Book) -> Result<Self> {
        let far = NaiveDate::from_ymd_opt(9999, 12, 31).expect("data");
        let mut steps = vec![];
        replay_observed(conn, book, far, &mut |s| {
            steps.push((s.date, s.qty_after));
            Ok(())
        })?;
        Ok(Self(steps))
    }

    /// Quantity at the end of `day`.
    fn at(&self, day: NaiveDate) -> Decimal {
        let i = self.0.partition_point(|(d, _)| *d <= day);
        if i == 0 {
            Decimal::ZERO
        } else {
            self.0[i - 1].1
        }
    }

    fn first(&self) -> Option<NaiveDate> {
        self.0.first().map(|(d, _)| *d)
    }
}

#[derive(Debug, Clone)]
struct Expected {
    cand: Candidate,
    qty: Decimal,
    gross: i64,
    tax: i64,
}

impl Expected {
    fn net(&self) -> i64 {
        self.gross - self.tax
    }
}

fn expected_of(c: &Candidate, qty: Decimal, class: crate::assets::AssetClass) -> Expected {
    let gross = to_minor(qty * c.per_share, &c.currency);
    let pay = c.pay_date.unwrap_or(c.ex_date);
    let on = records::credit_date(Some(c.ex_date), pay);
    let tax = records::withholding(c.kind, class, gross, on, c.exempt);
    Expected {
        cand: c.clone(),
        qty,
        gross,
        tax,
    }
}

fn recorded_of(conn: &Connection, asset_id: &str, account_id: &str) -> Result<Vec<Recorded>> {
    let rows = conn
        .prepare_cached(
            "SELECT id, kind, ex_date, pay_date, quantity, gross, tax_withheld, net, status, source, txn_id
               FROM income_event WHERE asset_id = ?1 AND account_id = ?2 ORDER BY pay_date, id",
        )?
        .query_map(params![asset_id, account_id], |r| {
            let kind: String = r.get(1)?;
            let status: String = r.get(8)?;
            let txn: Option<String> = r.get(10)?;
            Ok(Recorded {
                id: r.get(0)?,
                kind: serde_json::from_value(serde_json::Value::String(kind)).unwrap_or(IncomeKind::Other),
                ex_date: r.get(2)?,
                pay_date: r.get(3)?,
                quantity: r.get(4)?,
                gross: r.get(5)?,
                tax_withheld: r.get(6)?,
                net: r.get(7)?,
                status: if status == "announced" {
                    IncomeStatus::Announced
                } else {
                    IncomeStatus::Received
                },
                source: r.get(9)?,
                cash: txn.is_some(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Accounts that held the asset: with trades in it, or that received it through a ticker change.
fn accounts_of(conn: &Connection, asset_id: &str) -> Result<Vec<String>> {
    let mut accounts: Vec<String> = conn
        .prepare_cached(
            "SELECT DISTINCT account_id FROM trade WHERE asset_id = ?1 ORDER BY account_id",
        )?
        .query_map([asset_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let inherited: Vec<String> = conn
        .prepare_cached(
            "SELECT DISTINCT t.account_id FROM corporate_action ca JOIN trade t ON t.asset_id = ca.asset_id
              WHERE ca.new_asset_id = ?1 AND ca.kind IN ('ticker_change', 'merger')",
        )?
        .query_map([asset_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for a in inherited {
        if !accounts.contains(&a) {
            accounts.push(a);
        }
    }
    Ok(accounts)
}

fn d(s: &str) -> Option<NaiveDate> {
    parse_date(s).ok()
}

/// The recorded distribution describes the expected one (dates only; same kind family).
fn same_payment(e: &Expected, r: &Recorded) -> bool {
    if kind_group(e.cand.kind) != kind_group(r.kind) {
        return false;
    }
    let pay = r.pay_date.as_str();
    match r.ex_date.as_deref().and_then(d) {
        Some(x) => {
            near(x, e.cand.ex_date)
                // Typed record date, but the same payment date.
                || (e.cand.pay_date.is_some_and(|p| fmt_date(p) == pay)
                    && (x - e.cand.ex_date).num_days().abs() <= 10)
        }
        None => d(pay).is_some_and(|p| {
            let after = (p - e.cand.ex_date).num_days();
            (0..=PAY_WINDOW_DAYS).contains(&after)
                && e.cand
                    .pay_date
                    .is_none_or(|ep| (p - ep).num_days().abs() <= 7)
        }),
    }
}

/// How well the record describes the expected distribution (lower is better).
fn score(e: &Expected, r: &Recorded) -> (i64, i64, i64) {
    let ex = r
        .ex_date
        .as_deref()
        .and_then(d)
        .map_or(9, |x| pending::business_days_apart(x, e.cand.ex_date));
    let pay = match (d(&r.pay_date), e.cand.pay_date) {
        (Some(a), Some(b)) => (a - b).num_days().abs(),
        _ => 30,
    };
    (ex, pay, (r.net - e.net()).abs())
}

/// The record repeats `other` (a typo in the year of the payment, or a second record of the
/// same payment).
fn repeats(r: &Recorded, other: &Recorded) -> bool {
    if r.net != other.net {
        return false;
    }
    let (Some(a), Some(b)) = (d(&r.pay_date), d(&other.pay_date)) else {
        return false;
    };
    a.month() == b.month() && a.day() == b.day()
}

fn base_of(conn: &Connection, currency: &str, on: NaiveDate, v: i64, base: &str) -> Result<i64> {
    if v == 0 {
        return Ok(0);
    }
    let rate = fx_to_base(conn, currency, on)?.map_or(Decimal::ZERO, |(r, _)| r);
    Ok(to_minor(records::from_minor(v, currency) * rate, base))
}

fn note(key: &str, args: &[(&'static str, String)]) -> String {
    records::text(key, args)
}

fn br(d: &str) -> String {
    parse_date(d).map_or_else(|_| d.to_string(), |x| i18n::date(i18n::current(), x))
}

/// First payment date whose record without a cash entry still waits for the credit: the start
/// of the automatic recording (what came before is history, recorded without cash on purpose)
/// or, with it off, the last [`CASH_PENDING_DAYS`] days.
fn cash_cut(conn: &Connection, today: NaiveDate) -> Result<NaiveDate> {
    Ok(match auto_state(conn)? {
        Some(a) => parse_date(&a.since)? - chrono::Duration::days(HISTORY_GRACE_DAYS),
        None => today - chrono::Duration::days(CASH_PENDING_DAYS),
    })
}

/// Cash of a recorded distribution: `(cash_pending, note)`. A typo in the payment date and a
/// repeated record have their own note and wait for nothing.
fn cash_of(r: &Recorded, state: EntryState, cut: NaiveDate) -> (bool, Option<String>) {
    if r.cash || r.status != IncomeStatus::Received || state == EntryState::Duplicate {
        return (false, None);
    }
    match d(&r.pay_date) {
        Some(p) if p >= cut => (
            true,
            Some(note("nucleo-carteira-razao-nota-sem-caixa", &[])),
        ),
        _ => (
            false,
            Some(note("nucleo-carteira-razao-nota-historico-sem-caixa", &[])),
        ),
    }
}

/// Builds the ledger of one asset in one account.
#[allow(clippy::too_many_arguments)]
fn ledger_of(
    conn: &Connection,
    book: &Book,
    account: &str,
    cands: &[&Candidate],
    today: NaiveDate,
    cut: NaiveDate,
    base: &str,
    out: &mut Vec<LedgerEntry>,
) -> Result<Option<NaiveDate>> {
    let timeline = Timeline::of(conn, book)?;
    let brief = brief_of(&book.asset);
    let class = book.asset.class;
    let mut expected: Vec<Expected> = vec![];
    for c in cands {
        let qty = timeline.at(entitled_on(c.ex_date, &c.source));
        if qty <= Decimal::ZERO {
            continue;
        }
        let e = expected_of(c, qty, class);
        if e.gross > 0 {
            expected.push(e);
        }
    }
    let recorded = recorded_of(conn, &book.asset.id, account)?;
    // Best pairs first: each record pairs once, each expected distribution once.
    let mut pairs: Vec<((i64, i64, i64), usize, usize)> = vec![];
    for (ei, e) in expected.iter().enumerate() {
        for (ri, r) in recorded.iter().enumerate() {
            if same_payment(e, r) {
                pairs.push((score(e, r), ei, ri));
            }
        }
    }
    pairs.sort();
    let mut by_exp: Vec<Option<usize>> = vec![None; expected.len()];
    let mut by_rec: Vec<Option<usize>> = vec![None; recorded.len()];
    for (_, ei, ri) in pairs {
        if by_exp[ei].is_none() && by_rec[ri].is_none() {
            by_exp[ei] = Some(ri);
            by_rec[ri] = Some(ei);
        }
    }
    let ccy = book.asset.currency.clone();
    for (ei, e) in expected.iter().enumerate() {
        let c = &e.cand;
        let rec = by_exp[ei].map(|ri| recorded[ri].clone());
        let pay = c.pay_date.map(fmt_date);
        let paid_on = c.pay_date.unwrap_or(c.ex_date);
        let paid = paid_on <= today;
        let (state, fix, note_text, counted) = match &rec {
            None if !paid => (EntryState::Upcoming, None, None, 0),
            None => (EntryState::Missing, Some(Fix::Create), None, e.net()),
            Some(r) => {
                let net_diff = (r.net - e.net()).abs();
                let gross_diff = (r.gross - e.gross).abs();
                if net_diff <= ROUNDING && gross_diff <= ROUNDING {
                    (EntryState::Matches, None, None, r.net)
                } else if net_diff <= ROUNDING {
                    (
                        EntryState::TaxDiff,
                        (!r.credited()).then_some(Fix::Update),
                        None,
                        r.net,
                    )
                } else if r.credited() {
                    (
                        EntryState::ValueDiff,
                        None,
                        Some(note("nucleo-carteira-razao-nota-creditado", &[])),
                        r.net,
                    )
                } else {
                    (EntryState::ValueDiff, Some(Fix::Update), None, e.net())
                }
            }
        };
        let note_text = note_text.or_else(|| {
            (c.pay_date.is_none() && state != EntryState::Upcoming)
                .then(|| note("nucleo-carteira-razao-nota-pagamento-estimado", &[]))
        });
        let (cash_pending, cash_note) = rec
            .as_ref()
            .map_or((false, None), |r| cash_of(r, state, cut));
        let note_text = note_text.or(cash_note);
        let upcoming = if state == EntryState::Upcoming {
            e.net()
        } else {
            0
        };
        out.push(LedgerEntry {
            key: format!("{}|{account}", c.id),
            asset: brief.clone(),
            account_id: account.to_string(),
            kind: c.kind,
            ex_date: Some(fmt_date(c.ex_date)),
            pay_date: pay,
            pay_date_estimated: c.pay_date.is_none(),
            quantity: Some(e.qty.normalize().to_string()),
            amount_per_share: Some(c.per_share.normalize().to_string()),
            currency: ccy.clone(),
            expected_gross: Some(e.gross),
            expected_tax: Some(e.tax),
            expected_net: Some(e.net()),
            recorded: rec,
            state,
            fix,
            official: c.official,
            source: Some(c.source.clone()),
            period: c.period.clone(),
            exempt: c.exempt,
            check: Some(c.check.clone()),
            note: note_text,
            net_base: base_of(conn, &ccy, paid_on, counted, base)?,
            upcoming_base: base_of(conn, &ccy, paid_on, upcoming, base)?,
            cash_pending,
        });
    }
    // Records that pair with nothing expected: a repetition of a paired payment, or unexpected.
    for (ri, r) in recorded.iter().enumerate() {
        if by_rec[ri].is_some() {
            continue;
        }
        let twin = by_exp.iter().enumerate().find_map(|(ei, m)| {
            let other = &recorded[(*m)?];
            (same_payment(&expected[ei], r) || repeats(r, other)).then_some(other)
        });
        let typo = d(&r.pay_date).is_some_and(|p| (p - today).num_days() > IMPLAUSIBLE_DAYS);
        let (state, fix, note_text, counted) = match twin {
            Some(o) => (
                EntryState::Duplicate,
                Some(Fix::Delete),
                Some(note(
                    "nucleo-carteira-razao-nota-repete",
                    &[("data", br(&o.pay_date))],
                )),
                0,
            ),
            None if typo => (
                EntryState::Unexpected,
                None,
                Some(note("nucleo-carteira-razao-nota-data-improvavel", &[])),
                0,
            ),
            None => (
                EntryState::Unexpected,
                None,
                Some(note("nucleo-carteira-razao-nota-sem-fonte", &[])),
                if r.status == IncomeStatus::Received {
                    r.net
                } else {
                    0
                },
            ),
        };
        let on = d(&r.pay_date).unwrap_or(today);
        // The typo and the repetition keep their own note; nothing waits for their credit.
        let cash_pending = !typo && cash_of(r, state, cut).0;
        out.push(LedgerEntry {
            key: format!("rec:{}", r.id),
            asset: brief.clone(),
            account_id: account.to_string(),
            kind: r.kind,
            ex_date: r.ex_date.clone(),
            pay_date: Some(r.pay_date.clone()),
            pay_date_estimated: false,
            quantity: r.quantity.clone(),
            amount_per_share: None,
            currency: ccy.clone(),
            expected_gross: None,
            expected_tax: None,
            expected_net: None,
            recorded: Some(r.clone()),
            state,
            fix,
            official: false,
            source: None,
            period: None,
            exempt: None,
            check: None,
            note: note_text,
            net_base: base_of(conn, &ccy, on.min(today), counted, base)?,
            upcoming_base: 0,
            cash_pending,
        });
    }
    Ok(timeline.first())
}

/// Everything the ledger needs from the vault, in one pass.
pub fn build(conn: &Connection, args: &LedgerArgs) -> Result<Ledger> {
    build_on(conn, args, crate::time::today_local())
}

pub fn build_on(conn: &Connection, args: &LedgerArgs, today: NaiveDate) -> Result<Ledger> {
    let base = crate::household::base_currency(conn)?;
    let from = args.from.as_deref().map(parse_date).transpose()?;
    let to = args.to.as_deref().map(parse_date).transpose()?;
    let assets: Vec<String> = match &args.asset_id {
        Some(a) => vec![a.clone()],
        None => conn
            .prepare(
                "SELECT DISTINCT a.id FROM asset a
                  WHERE a.id IN (SELECT asset_id FROM trade)
                     OR a.id IN (SELECT asset_id FROM income_event)
                     OR a.id IN (SELECT ca.new_asset_id FROM corporate_action ca
                                  WHERE ca.new_asset_id IS NOT NULL AND ca.kind IN ('ticker_change', 'merger'))
                  ORDER BY a.id",
            )?
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?,
    };
    let since = NaiveDate::from_ymd_opt(1990, 1, 1).expect("data");
    let all = pending::all_candidates(conn, since)?;
    let mut by_asset: HashMap<&str, Vec<&Candidate>> = HashMap::new();
    for c in &all {
        by_asset.entry(c.asset_id.as_str()).or_default().push(c);
    }
    let cut = cash_cut(conn, today)?;
    let mut entries: Vec<LedgerEntry> = vec![];
    let mut first_dates: HashMap<String, NaiveDate> = HashMap::new();
    for asset_id in &assets {
        let cands = by_asset.get(asset_id.as_str()).cloned().unwrap_or_default();
        let mut accounts = accounts_of(conn, asset_id)?;
        // Records in accounts without trades (typed by hand) also show up.
        let extra: Vec<String> = conn
            .prepare_cached("SELECT DISTINCT account_id FROM income_event WHERE asset_id = ?1")?
            .query_map([asset_id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for a in extra {
            if !accounts.contains(&a) {
                accounts.push(a);
            }
        }
        if let Some(a) = &args.account_id {
            accounts.retain(|x| x == a);
        }
        for acc in accounts {
            let book = load_book(conn, asset_id, Some(std::slice::from_ref(&acc)))?;
            if let Some(f) = ledger_of(conn, &book, &acc, &cands, today, cut, &base, &mut entries)?
            {
                first_dates
                    .entry(asset_id.clone())
                    .and_modify(|x| *x = (*x).min(f))
                    .or_insert(f);
            }
        }
    }
    // Period filter (payment date, or the record date when there is none).
    entries.retain(|e| {
        let Some(day) = e
            .pay_date
            .as_deref()
            .or(e.ex_date.as_deref())
            .and_then(|x| parse_date(x).ok())
        else {
            return true;
        };
        from.is_none_or(|f| day >= f) && to.is_none_or(|t| day <= t)
    });
    let mut counts = Counts::default();
    let mut years: BTreeMap<String, PeriodTotal> = BTreeMap::new();
    let mut months: BTreeMap<String, PeriodTotal> = BTreeMap::new();
    let mut per_asset: BTreeMap<String, (AssetBrief, BTreeMap<String, PeriodTotal>, u32)> =
        BTreeMap::new();
    let (mut received, mut upcoming, mut recorded_total) = (0i64, 0i64, 0i64);
    for e in &entries {
        match e.state {
            EntryState::Matches => counts.matches += 1,
            EntryState::Missing => {
                counts.missing += 1;
                if !e.official {
                    counts.unofficial_missing += 1;
                }
            }
            EntryState::Upcoming => counts.upcoming += 1,
            EntryState::ValueDiff => counts.value_diff += 1,
            EntryState::TaxDiff => counts.tax_diff += 1,
            EntryState::Duplicate => counts.duplicate += 1,
            EntryState::Unexpected => counts.unexpected += 1,
        }
        if e.fix.is_some() && (e.official || e.state != EntryState::Missing) {
            counts.fixes += 1;
        }
        if e.cash_pending {
            counts.cash_pending += 1;
        }
        let rec_base = match &e.recorded {
            Some(r) if r.status == IncomeStatus::Received => {
                let on = parse_date(&r.pay_date).unwrap_or(today).min(today);
                base_of(conn, &e.currency, on, r.net, &base)?
            }
            _ => 0,
        };
        received += e.net_base;
        upcoming += e.upcoming_base;
        recorded_total += rec_base;
        let Some(month) = e.month() else { continue };
        let year = month[..4].to_string();
        // A record with an implausible payment date (a typo in the year) counts nothing and
        // must not open a year or a month of its own.
        let dated = !e.implausible(today);
        if dated {
            for (map, key) in [(&mut years, year.clone()), (&mut months, month.clone())] {
                let p = map.entry(key.clone()).or_insert_with(|| PeriodTotal {
                    period: key,
                    ..Default::default()
                });
                p.received_base += e.net_base;
                p.upcoming_base += e.upcoming_base;
                p.recorded_base += rec_base;
                if e.net_base != 0 || e.upcoming_base != 0 {
                    p.count += 1;
                }
            }
        }
        let a = per_asset
            .entry(e.asset.id.clone())
            .or_insert_with(|| (e.asset.clone(), BTreeMap::new(), 0));
        if dated {
            let p = a.1.entry(year.clone()).or_insert_with(|| PeriodTotal {
                period: year,
                ..Default::default()
            });
            p.received_base += e.net_base;
            p.upcoming_base += e.upcoming_base;
            p.recorded_base += rec_base;
            if e.net_base != 0 || e.upcoming_base != 0 {
                p.count += 1;
            }
        }
        // Everything to check, including distributions without an official notice.
        if e.fix.is_some() {
            a.2 += 1;
        }
    }
    let mut assets_out: Vec<AssetTotal> = per_asset
        .into_values()
        .map(|(asset, ys, fixes)| {
            let years: Vec<PeriodTotal> = ys.into_values().collect();
            AssetTotal {
                first_date: first_dates.get(&asset.id).map(|d| fmt_date(*d)),
                received_base: years.iter().map(|y| y.received_base).sum(),
                upcoming_base: years.iter().map(|y| y.upcoming_base).sum(),
                recorded_base: years.iter().map(|y| y.recorded_base).sum(),
                asset,
                years,
                fixes,
            }
        })
        .collect();
    assets_out.sort_by(|a, b| {
        b.received_base
            .cmp(&a.received_base)
            .then(a.asset.name.cmp(&b.asset.name))
    });
    entries.sort_by(|a, b| {
        let key = |e: &LedgerEntry| e.pay_date.clone().or(e.ex_date.clone()).unwrap_or_default();
        key(b)
            .cmp(&key(a))
            .then(a.asset.name.cmp(&b.asset.name))
            .then(a.key.cmp(&b.key))
    });
    if args.only_problems.unwrap_or(false) {
        entries.retain(|e| e.state != EntryState::Matches || e.cash_pending);
    }
    let auto = auto_state(conn)?;
    Ok(Ledger {
        base_currency: base,
        entries,
        assets: assets_out,
        years: years.into_values().collect(),
        months: months.into_values().collect(),
        received_base: received,
        upcoming_base: upcoming,
        recorded_base: recorded_total,
        counts,
        auto: auto.is_some(),
        auto_since: auto.map(|a| a.since),
    })
}

// ---------------------------------------------------------------------------------------
// Rebuild and automatic recording
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeRebuildArgs")]
pub struct RebuildArgs {
    /// Keys of the entries to fix (from the ledger), including distributions without an
    /// official notice that the family checked. Empty or absent = every official fix.
    #[serde(default)]
    #[ts(optional)]
    pub keys: Option<Vec<String>>,
    /// Only this asset.
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "IncomeRebuildReport")]
pub struct RebuildReport {
    pub created: u32,
    pub updated: u32,
    pub deleted: u32,
    pub change_set_id: Option<Id>,
    /// Automatic recording is on from now on.
    pub auto: bool,
}

/// Entries the rebuild may fix: everything with a fix, except unofficial distributions to
/// create (they wait for the family's confirmation in "To confirm").
pub fn proposable(e: &LedgerEntry) -> bool {
    e.fix.is_some() && (e.official || e.fix != Some(Fix::Create))
}

/// Applies the fixes of the chosen entries. `cash`: new records move the brokerage cash.
fn apply_fixes(
    tx: &Transaction<'_>,
    entries: &[&LedgerEntry],
    cash: impl Fn(&LedgerEntry) -> bool,
    source: &str,
) -> Result<RebuildReport> {
    let mut rep = RebuildReport::default();
    for e in entries {
        match e.fix {
            Some(Fix::Create) => {
                let (Some(gross), Some(tax), Some(ex)) =
                    (e.expected_gross, e.expected_tax, &e.ex_date)
                else {
                    continue;
                };
                let input = IncomeInput {
                    asset_id: e.asset.id.clone(),
                    account_id: e.account_id.clone(),
                    kind: e.kind,
                    ex_date: Some(ex.clone()),
                    pay_date: e.pay_date.clone().unwrap_or_else(|| ex.clone()),
                    quantity: e.quantity.clone(),
                    gross,
                    tax_withheld: Some(tax),
                    fx_rate: None,
                    status: Some(IncomeStatus::Received),
                    cash_movement: Some(cash(e)),
                };
                let origin = if e.official { source } else { "agenda" };
                records::create_income_with(tx, &input, origin, Some(&e.key))?;
                rep.created += 1;
            }
            Some(Fix::Update) => {
                let (Some(r), Some(gross), Some(tax)) =
                    (&e.recorded, e.expected_gross, e.expected_tax)
                else {
                    continue;
                };
                records::update_income_values(tx, &r.id, gross, tax, e.quantity.as_deref())?;
                rep.updated += 1;
            }
            Some(Fix::Delete) => {
                let Some(r) = &e.recorded else { continue };
                records::delete_income(tx, &r.id)?;
                rep.deleted += 1;
            }
            None => {}
        }
    }
    Ok(rep)
}

/// What the rebuild would fix, and the reason of its change set.
fn rebuild_plan(
    conn: &Connection,
    args: &RebuildArgs,
    today: NaiveDate,
) -> Result<(Vec<LedgerEntry>, String)> {
    let ledger = build_on(
        conn,
        &LedgerArgs {
            asset_id: args.asset_id.clone(),
            ..Default::default()
        },
        today,
    )?;
    let chosen: Vec<LedgerEntry> = ledger
        .entries
        .into_iter()
        .filter(|e| match &args.keys {
            // Chosen by the family: also the distributions without an official notice.
            Some(k) if !k.is_empty() => e.fix.is_some() && k.contains(&e.key),
            _ => proposable(e),
        })
        .collect();
    let reason = t_args(
        "nucleo-carteira-razao-motivo-reconstruir",
        &i18n::args(&[("n", (chosen.len() as i64).into())]),
    );
    Ok((chosen, reason))
}

fn rebuild_tx(
    tx: &Transaction<'_>,
    chosen: &[LedgerEntry],
    args: &RebuildArgs,
    today: NaiveDate,
) -> Result<RebuildReport> {
    let refs: Vec<&LedgerEntry> = chosen.iter().collect();
    let mut rep = apply_fixes(tx, &refs, |_| false, "official")?;
    if args.asset_id.is_none() && auto_state(tx)?.is_none() {
        crate::settings::set(
            tx,
            AUTO_SETTING,
            &AutoState {
                since: fmt_date(today),
            },
        )?;
    }
    rep.auto = auto_state(tx)?.is_some();
    Ok(rep)
}

/// Rebuilds the recorded distributions from the ledger: records the missing official ones
/// (without a brokerage cash entry: it is history), fixes amounts and deletes repetitions, in a
/// single change set, and turns the automatic recording on (rebuilding the whole portfolio).
pub fn rebuild(ctx: &mut crate::api::Ctx<'_>, args: &RebuildArgs) -> Result<RebuildReport> {
    let today = crate::time::today_local();
    let (chosen, reason) = rebuild_plan(ctx.conn(), args, today)?;
    let r = ctx.write(&reason, |tx| rebuild_tx(tx, &chosen, args, today))?;
    let mut rep = r.value;
    rep.change_set_id = r.change_set_id;
    Ok(rep)
}

/// [`rebuild`] straight on a connection, as `author` (tests and tools).
pub fn rebuild_as(
    conn: &mut Connection,
    author: &crate::changes::Author,
    args: &RebuildArgs,
    today: NaiveDate,
) -> Result<RebuildReport> {
    let (chosen, reason) = rebuild_plan(conn, args, today)?;
    let r = crate::changes::record(conn, author, &reason, |tx| {
        rebuild_tx(tx, &chosen, args, today)
    })?;
    let mut rep = r.value;
    rep.change_set_id = r.change_set_id;
    Ok(rep)
}

/// Turns the automatic recording off (the records already made stay).
pub fn set_auto(conn: &Connection, on: bool) -> Result<()> {
    if on {
        if auto_state(conn)?.is_none() {
            crate::settings::set(
                conn,
                AUTO_SETTING,
                &AutoState {
                    since: fmt_date(crate::time::today_local()),
                },
            )?;
        }
        Ok(())
    } else {
        crate::settings::delete(conn, AUTO_SETTING)
    }
}

/// The family deleted or undid an automatic record of this entry: it does not come back.
fn dismissed(conn: &Connection, key: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM change_item ci JOIN change_set cs ON cs.id = ci.change_set_id
           WHERE ci.table_name = 'income_event'
             AND ((ci.op = 'insert' AND cs.status = 'undone' AND json_extract(ci.after, '$.external_id') = ?1)
               OR (ci.op = 'delete' AND cs.status = 'applied' AND json_extract(ci.before, '$.external_id') = ?1)))",
        [key],
        |r| r.get(0),
    )?)
}

/// After a collection, with the automatic recording on: records the official distributions
/// already paid that are missing (a payment from before the start of the automatic recording is
/// history and goes without a cash entry), as the system, in one change set.
pub fn auto_record(conn: &mut Connection, today: NaiveDate) -> Result<u32> {
    let Some(state) = auto_state(conn)? else {
        return Ok(0);
    };
    let start = parse_date(&state.since)? - chrono::Duration::days(HISTORY_GRACE_DAYS);
    let ledger = build_on(conn, &LedgerArgs::default(), today)?;
    let mut todo: Vec<&LedgerEntry> = vec![];
    for e in &ledger.entries {
        if e.state == EntryState::Missing
            && e.official
            && !e.pay_date_estimated
            && !dismissed(conn, &e.key)?
        {
            todo.push(e);
        }
    }
    if todo.is_empty() {
        return Ok(0);
    }
    let n = todo.len() as u32;
    let reason = t_args(
        "nucleo-carteira-razao-motivo-automatico",
        &i18n::args(&[("n", i64::from(n).into())]),
    );
    crate::changes::record(
        conn,
        &crate::changes::Author::system("official"),
        &reason,
        |tx| {
            apply_fixes(
                tx,
                &todo,
                |e| {
                    e.pay_date
                        .as_deref()
                        .and_then(|p| parse_date(p).ok())
                        .is_some_and(|p| p >= start)
                },
                "official",
            )
        },
    )?;
    Ok(n)
}

/// Guard for writes the agent cannot do.
pub fn ensure_user(ctx: &crate::api::Ctx<'_>) -> Result<()> {
    if *ctx.actor != crate::api::Actor::User {
        return Err(Error::Forbidden(t("nucleo-carteira-razao-erro-so-usuario")));
    }
    Ok(())
}
