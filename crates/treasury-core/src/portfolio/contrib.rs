//! Maiores contribuições do período (docs/portfolio-compare.md): quanto cada ativo somou ou
//! tirou do resultado da carteira entre o fim do mês anterior ao primeiro mês e o fim do último
//! (ou hoje), em dinheiro e em pontos percentuais do retorno.
//!
//! Resultado do ativo = valor no fim − valor no início − compras + vendas + proventos do período.
//! Os pontos percentuais dividem pelo mesmo denominador do Modified Dietz da carteira no período
//! inteiro: o valor no início mais as compras e vendas pesadas pelo tempo que ficaram aplicadas.
//! Desempenho passado, sem ranking de compra (CVM Res. 20/2021).

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{Book, brief_of, fx_to_base, load_book, replay, traded_assets, value_state};
use super::records::to_minor;
use super::{AssetBrief, TradeSide};
use crate::error::Result;
use crate::time::{fmt_date, month_bounds, parse_month};

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct ContributionArgs {
    /// Primeiro mês (`YYYY-MM`); sem ele, desde a primeira compra.
    #[ts(optional)]
    pub from_month: Option<String>,
    /// Último mês (`YYYY-MM`); sem ele, até hoje.
    #[ts(optional)]
    pub to_month: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Contribution {
    pub asset: AssetBrief,
    /// Resultado do ativo no período, na moeda base (menor unidade).
    #[ts(type = "number")]
    pub result: i64,
    /// Proventos do período (já dentro do resultado).
    #[ts(type = "number")]
    pub income: i64,
    /// Pontos percentuais do retorno da carteira no período (0,012 = 1,2 p.p.).
    pub points: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ContributionReport {
    /// Início (o fim do mês anterior ao primeiro mês) e fim, `YYYY-MM-DD`.
    pub from: String,
    pub to: String,
    pub base_currency: String,
    /// Do maior resultado para o menor (os que tiraram ficam no fim).
    pub rows: Vec<Contribution>,
    #[ts(type = "number")]
    pub total: i64,
    /// Retorno da carteira no período pelo mesmo denominador (soma dos pontos).
    pub total_points: Option<f64>,
}

fn fx_or_zero(
    conn: &Connection,
    ccy: &str,
    fixed: Option<Decimal>,
    d: NaiveDate,
) -> Result<Decimal> {
    Ok(match fixed {
        Some(f) => f,
        None => fx_to_base(conn, ccy, d)?
            .map(|(r, _)| r)
            .unwrap_or(Decimal::ZERO),
    })
}

fn value_on(conn: &Connection, b: &Book, d: NaiveDate) -> Result<Decimal> {
    let st = replay(conn, b, d)?;
    Ok(value_state(conn, b, &st, d)?
        .value_base
        .unwrap_or(Decimal::ZERO))
}

pub fn contributions(conn: &Connection, args: &ContributionArgs) -> Result<ContributionReport> {
    let today = crate::time::today_local();
    let base_ccy = crate::household::base_currency(conn)?;
    let mut books: Vec<Book> = vec![];
    for (asset_id, _) in traded_assets(conn, None)? {
        if !books.iter().any(|b| b.asset.id == asset_id) {
            books.push(load_book(conn, &asset_id, None)?);
        }
    }
    let first = books
        .iter()
        .filter_map(|b| b.trades.first().map(|t| t.date))
        .min();
    let (y0, m0) = match (&args.from_month, first) {
        (Some(m), _) => parse_month(m)?,
        (None, Some(d)) => (d.year(), d.month()),
        (None, None) => (today.year(), today.month()),
    };
    let start = month_bounds(y0, m0).0 - chrono::Duration::days(1);
    let end = match &args.to_month {
        Some(m) => {
            let (y, mo) = parse_month(m)?;
            month_bounds(y, mo).1.min(today)
        }
        None => today,
    };
    let span = Decimal::from((end - start).num_days().max(1));
    let mut denom = Decimal::ZERO;
    let mut raw: Vec<(AssetBrief, Decimal, Decimal)> = vec![];
    for b in &books {
        let v0 = value_on(conn, b, start)?;
        let v1 = value_on(conn, b, end)?;
        let mut flows = Decimal::ZERO;
        let mut weighted = Decimal::ZERO;
        let mut traded = false;
        for t in b.trades.iter().filter(|t| t.date > start && t.date <= end) {
            let fx = fx_or_zero(conn, &b.asset.currency, t.fx, t.date)?;
            let gross = t.qty * t.price;
            // Dinheiro que entrou no ativo: compra positiva, venda negativa.
            let f = match t.side {
                TradeSide::Buy => gross + t.costs,
                TradeSide::Sell => -(gross - t.costs),
            } * fx;
            flows += f;
            weighted += f * Decimal::from((end - t.date).num_days()) / span;
            traded = true;
        }
        let mut income = Decimal::ZERO;
        for i in b
            .incomes
            .iter()
            .filter(|i| i.pay_date > start && i.pay_date <= end)
        {
            income += i.net * fx_or_zero(conn, &b.asset.currency, i.fx, i.pay_date)?;
        }
        if v0.is_zero() && v1.is_zero() && !traded && income.is_zero() {
            continue;
        }
        denom += v0 + weighted;
        raw.push((brief_of(&b.asset), v1 - v0 - flows + income, income));
    }
    let points = |r: Decimal| -> Option<f64> {
        (denom > Decimal::ZERO)
            .then(|| (r / denom).to_f64())
            .flatten()
    };
    let mut rows: Vec<Contribution> = raw
        .into_iter()
        .map(|(asset, r, inc)| Contribution {
            asset,
            result: to_minor(r, &base_ccy),
            income: to_minor(inc, &base_ccy),
            points: points(r),
        })
        .collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.result));
    let total: i64 = rows.iter().map(|r| r.result).sum();
    let total_points =
        (denom > Decimal::ZERO).then(|| rows.iter().filter_map(|r| r.points).sum::<f64>());
    Ok(ContributionReport {
        from: fmt_date(start),
        to: fmt_date(end),
        base_currency: base_ccy,
        rows,
        total,
        total_points,
    })
}
