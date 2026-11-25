//! Sales without a recorded purchase: sales larger than the position on their date.
//!
//! An official source (the B3 investor area, a brokerage note) reports a sale whose purchase
//! is older than the imported history. The sale is recorded as reported and nothing is made
//! up for the missing purchase: the portfolio takes from the position only what it had, and
//! the rest stays out of the average cost, of the realized result and of the tax
//! computation. This module lists what is left to explain, straight from the replay of each
//! asset (all accounts together, like the sales book and the tax), so the list always
//! follows the book:
//!
//! - recording the missing purchase (or importing the older history) takes the sale off the
//!   list by itself;
//! - a sale whose cost the person does not have can be **accepted**: it stays out of the
//!   results and stops asking. The accepted trade ids live in the [`ACCEPTED_SETTING`]
//!   setting, inside the vault.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine;
use super::records::to_minor;
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::ids::Id;
use crate::time::fmt_date;

/// Setting with the ids of the sales accepted without a cost.
pub const ACCEPTED_SETTING: &str = "portfolio.uncovered_sales_accepted";

/// Quantities that differ by less than this are the same.
const ROUNDING: Decimal = Decimal::from_parts(1, 0, 0, false, 6);

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "UncoveredSalesArgs")]
pub struct UncoveredSalesArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
    /// Only the sales of these trades (an import asks for the ones it wrote).
    #[serde(default)]
    #[ts(optional)]
    pub trade_ids: Option<Vec<Id>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "UncoveredSale")]
pub struct UncoveredSale {
    pub trade_id: Id,
    pub date: String,
    pub asset_id: Id,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub account_id: Id,
    pub account_name: String,
    /// Quantity the trade records.
    pub quantity: String,
    /// Part of it the position did not have on the date.
    pub missing_quantity: String,
    pub price: String,
    pub currency: String,
    /// Sale value of the missing part (quantity × price), in cents of `currency`.
    #[ts(type = "number")]
    pub missing_value: i64,
    /// Accepted without a cost: stays out of the results and no longer asks.
    pub accepted: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "UncoveredSales")]
pub struct UncoveredSales {
    /// Most recent first.
    pub rows: Vec<UncoveredSale>,
    /// Sales still to explain.
    #[ts(type = "number")]
    pub open: u32,
    #[ts(type = "number")]
    pub accepted: u32,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, rename = "UncoveredSaleAcceptArgs")]
pub struct UncoveredSaleAcceptArgs {
    pub trade_id: Id,
    /// `false` puts the sale back on the list.
    pub accepted: bool,
}

fn accepted_ids(conn: &Connection) -> Result<HashSet<String>> {
    Ok(
        crate::settings::get_or::<Vec<String>>(conn, ACCEPTED_SETTING, vec![])?
            .into_iter()
            .collect(),
    )
}

/// The sales larger than the position on their date, most recent first.
pub fn list(conn: &Connection, args: &UncoveredSalesArgs) -> Result<UncoveredSales> {
    let assets: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT asset_id FROM trade
              WHERE side = 'sell' AND (?1 IS NULL OR asset_id = ?1)",
        )?;
        stmt.query_map([&args.asset_id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let wanted: Option<HashSet<&str>> = args
        .trade_ids
        .as_ref()
        .map(|ids| ids.iter().map(String::as_str).collect());
    let accepted = accepted_ids(conn)?;
    let mut accounts: Option<HashMap<String, String>> = None;
    let mut rows = vec![];
    for id in assets {
        let book = engine::load_book(conn, &id, None)?;
        let state = engine::replay(conn, &book, NaiveDate::MAX)?;
        for (trade_id, missing) in &state.uncovered {
            let Some(t) = book.trades.iter().find(|t| &t.id == trade_id) else {
                continue;
            };
            // A position adjustment is a correction, not a sale to explain; a difference
            // below a millionth of a unit is spreadsheet rounding (Tesouro fractions).
            if t.adjustment
                || *missing <= ROUNDING
                || wanted.as_ref().is_some_and(|w| !w.contains(t.id.as_str()))
            {
                continue;
            }
            if accounts.is_none() {
                let mut stmt = conn.prepare("SELECT id, name FROM account")?;
                accounts = Some(
                    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                        .collect::<rusqlite::Result<_>>()?,
                );
            }
            let account_name = accounts
                .as_ref()
                .and_then(|a| a.get(&t.account_id))
                .cloned()
                .unwrap_or_default();
            rows.push((
                t.date,
                UncoveredSale {
                    trade_id: t.id.clone(),
                    date: fmt_date(t.date),
                    asset_id: book.asset.id.clone(),
                    ticker: book.asset.ticker.clone(),
                    name: book.asset.name.clone(),
                    class: book.asset.class,
                    account_id: t.account_id.clone(),
                    account_name,
                    quantity: t.qty.normalize().to_string(),
                    missing_quantity: missing.normalize().to_string(),
                    price: t.price.normalize().to_string(),
                    currency: book.asset.currency.clone(),
                    missing_value: to_minor(*missing * t.price, &book.asset.currency),
                    accepted: accepted.contains(&t.id),
                },
            ));
        }
    }
    rows.sort_by(|a, b| (b.0, &a.1.name, &a.1.trade_id).cmp(&(a.0, &b.1.name, &b.1.trade_id)));
    let rows: Vec<UncoveredSale> = rows.into_iter().map(|(_, r)| r).collect();
    let accepted = rows.iter().filter(|r| r.accepted).count() as u32;
    Ok(UncoveredSales {
        open: rows.len() as u32 - accepted,
        accepted,
        rows,
    })
}

/// Number of sales still to explain (not accepted).
pub fn open_count(conn: &Connection) -> Result<u32> {
    // Without a sale in the vault there is nothing to replay.
    let any: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM trade WHERE side = 'sell')",
        [],
        |r| r.get(0),
    )?;
    if !any {
        return Ok(0);
    }
    Ok(list(conn, &UncoveredSalesArgs::default())?.open)
}

/// Accepts a sale without a cost (or puts it back on the list). Ids of trades that no longer
/// exist leave the setting on the way.
pub fn accept(conn: &Connection, a: &UncoveredSaleAcceptArgs) -> Result<()> {
    let is_sale: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM trade WHERE id = ?1 AND side = 'sell')",
        [&a.trade_id],
        |r| r.get(0),
    )?;
    if !is_sale {
        return Err(Error::not_found(crate::i18n::t(
            "nucleo-carteira-venda-sem-compra-nao-encontrada",
        )));
    }
    let mut ids = accepted_ids(conn)?;
    if a.accepted {
        ids.insert(a.trade_id.clone());
    } else {
        ids.remove(&a.trade_id);
    }
    let mut kept = vec![];
    {
        let mut stmt = conn.prepare("SELECT EXISTS (SELECT 1 FROM trade WHERE id = ?1)")?;
        for id in ids {
            if stmt.query_row([&id], |r| r.get::<_, bool>(0))? {
                kept.push(id);
            }
        }
    }
    kept.sort();
    crate::settings::set(conn, ACCEPTED_SETTING, &kept)
}
