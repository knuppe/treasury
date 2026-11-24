//! Totals of an asset's trades (docs/asset-page.md): buys and sells with quantity, gross
//! value and average price, fees and withheld tax, the share of costs in the traded volume
//! and the trading period. Quantities are in today's units: a split or reverse split after a
//! trade multiplies its quantity by the factor (the gross stays), so the average prices
//! compare with the current one. The current position stays in `portfolio.position`;
//! `events` counts what else changes the quantity (bonuses, come-cotas, ticker changes).

use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{TradeRow, load_book};
use super::records::to_minor;
use super::{CorporateActionKind, TradeSide};
use crate::error::Result;
use crate::ids::Id;

/// Events that only change the unit: the quantity is adjusted, not explained.
const SPLITS: [CorporateActionKind; 2] = [
    CorporateActionKind::Split,
    CorporateActionKind::ReverseSplit,
];

#[derive(Debug, Clone, Deserialize)]
pub struct TradeSummaryArgs {
    pub asset_id: Id,
    /// Only this account (default: all).
    #[serde(default)]
    pub account_id: Option<Id>,
}

/// Totals of one side (buys or sells). Amounts in the smallest unit of the asset's currency.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TradeSideTotals {
    #[ts(type = "number")]
    pub count: i64,
    /// Decimal string.
    pub quantity: String,
    /// Sum of quantity × price of each trade (rounded per trade, as in the trade list).
    #[ts(type = "number")]
    pub gross: i64,
    /// Brokerage and exchange fees.
    #[ts(type = "number")]
    pub fees: i64,
    /// Tax withheld on the trade (IRRF).
    #[ts(type = "number")]
    pub taxes: i64,
    /// Gross ÷ quantity, without costs; `null` with no trades.
    pub avg_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TradeSummary {
    pub asset_id: Id,
    pub currency: String,
    /// Fixed income: only values mean something (quantity is internal).
    pub by_value: bool,
    pub buys: TradeSideTotals,
    pub sells: TradeSideTotals,
    /// Fees + taxes of both sides.
    #[ts(type = "number")]
    pub costs: i64,
    /// Gross of buys + gross of sells.
    #[ts(type = "number")]
    pub volume: i64,
    /// Costs ÷ volume (fraction); `null` with no volume.
    pub cost_share: Option<f64>,
    /// Bought − sold in today's units (decimal string).
    pub net_quantity: String,
    /// Some quantity was adjusted for a later split or reverse split.
    pub split_adjusted: bool,
    /// Other corporate events, come-cotas and positions inherited from another ticker: they
    /// change the quantity, so the current position may differ from `net_quantity`.
    #[ts(type = "number")]
    pub events: i64,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
}

#[derive(Default)]
struct Acc {
    count: i64,
    qty: Decimal,
    value: Decimal,
    gross: i64,
    fees: Decimal,
    taxes: Decimal,
}

impl Acc {
    fn add(&mut self, t: &TradeRow, factor: Decimal, currency: &str) {
        self.count += 1;
        self.qty += t.qty * factor;
        self.value += t.qty * t.price;
        self.gross += to_minor(t.qty * t.price, currency);
        self.fees += t.fees;
        self.taxes += t.irrf;
    }

    fn totals(&self, currency: &str) -> TradeSideTotals {
        TradeSideTotals {
            count: self.count,
            quantity: self.qty.normalize().to_string(),
            gross: self.gross,
            fees: to_minor(self.fees, currency),
            taxes: to_minor(self.taxes, currency),
            avg_price: (!self.qty.is_zero())
                .then(|| (self.value / self.qty).round_dp(8).normalize().to_string()),
        }
    }
}

pub fn summary(conn: &Connection, args: &TradeSummaryArgs) -> Result<TradeSummary> {
    let accounts = args.account_id.clone().map(|a| vec![a]);
    let book = load_book(conn, &args.asset_id, accounts.as_deref())?;
    let currency = book.asset.currency.clone();
    let splits: Vec<_> = book
        .actions
        .iter()
        .filter(|a| SPLITS.contains(&a.kind))
        .filter_map(|a| a.factor.filter(|f| *f > Decimal::ZERO).map(|f| (a.date, f)))
        .collect();
    let (mut buys, mut sells) = (Acc::default(), Acc::default());
    let mut split_adjusted = false;
    for t in &book.trades {
        // Events of a date apply before the trades of that date (engine::replay).
        let factor: Decimal = splits
            .iter()
            .filter(|(d, _)| *d > t.date)
            .map(|(_, f)| *f)
            .product();
        split_adjusted |= factor != Decimal::ONE;
        match t.side {
            TradeSide::Buy => buys.add(t, factor, &currency),
            TradeSide::Sell => sells.add(t, factor, &currency),
        }
    }
    let (buys, sells, net) = (
        buys.totals(&currency),
        sells.totals(&currency),
        buys.qty - sells.qty,
    );
    let costs = buys.fees + buys.taxes + sells.fees + sells.taxes;
    let volume = buys.gross + sells.gross;
    let dates = book.trades.iter().map(|t| t.date);
    Ok(TradeSummary {
        asset_id: book.asset.id.clone(),
        by_value: book.asset.class == crate::assets::AssetClass::FixedIncome,
        currency,
        cost_share: (volume > 0).then(|| costs as f64 / volume as f64),
        costs,
        volume,
        buys,
        sells,
        net_quantity: net.normalize().to_string(),
        split_adjusted,
        events: (book
            .actions
            .iter()
            .filter(|a| !SPLITS.contains(&a.kind))
            .count()
            + book.inherited.len()) as i64,
        first_date: dates.clone().min().map(crate::time::fmt_date),
        last_date: dates.max().map(crate::time::fmt_date),
    })
}
