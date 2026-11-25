//! Motor da carteira: reproduz operações e eventos de cada ativo para obter posição,
//! preço médio (na moeda do ativo e na moeda base), lotes de renda fixa, valor de mercado ou na
//! curva, resultado, XIRR nominal/real e TWR mensal (Modified Dietz).

#![allow(clippy::type_complexity)] // linhas lidas como tuplas do SQLite

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use super::records::{dec, from_minor, to_minor};
use super::*;
use crate::assets::{Asset, AssetClass, FixedIncomeTerms, Indexer};
use crate::error::{Error, Result};
use crate::finance::average_cost::{Position, PositionEvent};
use crate::finance::calendar::is_business_day;
use crate::finance::fixed_income;
use crate::finance::returns::{Flow, xirr};
use crate::i18n::{self, t, t_args};
use crate::indices::{self, Deflator};
use crate::time::{add_months, fmt_date, month_bounds, parse_date, parse_month};

#[derive(Debug, Clone)]
pub(crate) struct TradeRow {
    pub(crate) id: String,
    pub(crate) account_id: String,
    pub(crate) date: NaiveDate,
    pub(crate) side: TradeSide,
    pub(crate) qty: Decimal,
    pub(crate) price: Decimal,
    /// Custos e impostos na moeda do ativo (unidade maior).
    pub(crate) costs: Decimal,
    /// Só corretagem e emolumentos (sem IRRF), unidade maior.
    pub(crate) fees: Decimal,
    /// IRRF retido ("dedo-duro"), unidade maior.
    pub(crate) irrf: Decimal,
    pub(crate) fx: Option<Decimal>,
    /// Position adjustment ([`super::records::is_position_adjustment`]): it moves the quantity
    /// and the cost like any trade, and stays out of the sales book.
    pub(crate) adjustment: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ActionRow {
    pub(crate) date: NaiveDate,
    pub(crate) kind: CorporateActionKind,
    pub(crate) factor: Option<Decimal>,
    pub(crate) unit_cost: Option<Decimal>,
    pub(crate) amount: Option<Decimal>,
    /// Evento sintético: a posição saiu para outro ativo (mudança de ticker/incorporação).
    pub(crate) moved_out: bool,
    /// Evento sintético: come-cotas, cotas resgatadas para o IR (o custo total fica).
    pub(crate) come_cotas: Option<Decimal>,
    /// Bonus credited by quantity (in an account the book covers) instead of a factor.
    pub(crate) quantity: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub(crate) struct IncomeRow {
    pub(crate) pay_date: NaiveDate,
    pub(crate) net: Decimal,
    /// IR retido na fonte (unidade maior).
    pub(crate) tax: Decimal,
    pub(crate) fx: Option<Decimal>,
}

/// Tudo o que se sabe de um ativo (numa conta ou em todas).
#[derive(Debug, Clone)]
pub struct Book {
    pub asset: Asset,
    pub(crate) trades: Vec<TradeRow>,
    pub(crate) actions: Vec<ActionRow>,
    pub(crate) incomes: Vec<IncomeRow>,
    /// Posições herdadas de ativos que viraram este (mudança de ticker, incorporação).
    pub(crate) inherited: Vec<Inherited>,
}

/// Posição que chega de outro ativo numa data, com o custo de lá (sem realizar resultado).
#[derive(Debug, Clone)]
pub(crate) struct Inherited {
    pub(crate) date: NaiveDate,
    pub(crate) qty: Decimal,
    pub(crate) cost: Decimal,
    pub(crate) cost_base: Decimal,
    /// Fluxos do ativo de origem: entram no XIRR deste ativo, mas não no agregado da
    /// carteira (lá já são contados no ativo de origem).
    pub(crate) flows: Vec<Flow>,
}

#[derive(Debug, Clone)]
struct Lot {
    qty: Decimal,
    principal: Decimal,
    principal_base: Decimal,
    date: NaiveDate,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    native: Position,
    base: Position,
    lots: Vec<Lot>,
    pub estimated: bool,
    pub warnings: Vec<String>,
    /// Sales larger than the position on their date (trade id, quantity the position did not
    /// have): the purchases are missing from the book. The position takes only what it had
    /// and the rest stays out of the cost and of the realized result.
    pub(crate) uncovered: Vec<(String, Decimal)>,
}

impl State {
    pub fn quantity(&self) -> Decimal {
        self.native.qty
    }
    /// Custo total na moeda do ativo.
    pub fn cost(&self) -> Decimal {
        self.native.cost
    }
    /// Custo total na moeda base (câmbio das compras).
    pub fn cost_base(&self) -> Decimal {
        self.base.cost
    }
    /// Resultado realizado acumulado na moeda base.
    pub fn realized_base(&self) -> Decimal {
        self.base.realized
    }
}

/// Câmbio 1 `ccy` = x na moeda base da família na data (ou o último anterior).
pub(crate) fn fx_to_base(
    conn: &Connection,
    ccy: &str,
    date: NaiveDate,
) -> Result<Option<(Decimal, bool)>> {
    let base = crate::household::base_currency(conn)?;
    if ccy == base {
        return Ok(Some((Decimal::ONE, false)));
    }
    Ok(crate::fx::rate(conn, ccy, &base, &fmt_date(date))?.map(|(r, _, _, est)| (r, est)))
}

fn parse_opt(s: Option<String>, what: &str) -> Result<Option<Decimal>> {
    s.map(|v| dec(&v, what)).transpose()
}

fn load_trades(
    conn: &Connection,
    asset_id: &str,
    accounts: Option<&[String]>,
    before: Option<NaiveDate>,
) -> Result<Vec<TradeRow>> {
    let mut stmt = conn.prepare(
        "SELECT account_id, date, side, quantity, price, fees, taxes, currency, fx_rate, id, external_id
         FROM trade WHERE asset_id = ?1 ORDER BY date, created_at",
    )?;
    let mut rows = stmt.query([asset_id])?;
    let mut out = vec![];
    while let Some(r) = rows.next()? {
        let acc: String = r.get(0)?;
        if accounts.is_some_and(|a| !a.contains(&acc)) {
            continue;
        }
        let date = parse_date(&r.get::<_, String>(1)?)?;
        if before.is_some_and(|b| date >= b) {
            continue;
        }
        let side: String = r.get(2)?;
        let (fees, taxes, cur): (i64, i64, String) = (r.get(5)?, r.get(6)?, r.get(7)?);
        out.push(TradeRow {
            id: r.get(9)?,
            account_id: acc,
            date,
            side: if side == "sell" {
                TradeSide::Sell
            } else {
                TradeSide::Buy
            },
            qty: dec(&r.get::<_, String>(3)?, "nucleo-carteira-campo-quantidade")?,
            price: dec(&r.get::<_, String>(4)?, "nucleo-carteira-campo-preco")?,
            costs: from_minor(fees + taxes, &cur),
            fees: from_minor(fees, &cur),
            irrf: from_minor(taxes, &cur),
            fx: parse_opt(r.get(8)?, "nucleo-carteira-campo-cambio")?,
            adjustment: super::records::is_position_adjustment(
                r.get::<_, Option<String>>(10)?.as_deref(),
            ),
        });
    }
    Ok(out)
}

fn load_actions(
    conn: &Connection,
    asset_id: &str,
    accounts: Option<&[String]>,
    before: Option<NaiveDate>,
) -> Result<Vec<ActionRow>> {
    let mut out = vec![];
    let base = crate::household::base_currency(conn)?;
    for a in super::records::list_actions(conn, Some(asset_id))? {
        let date = parse_date(&a.date)?;
        if before.is_some_and(|b| date >= b) {
            continue;
        }
        // An event of one account (bonus by quantity) counts only in books that cover it.
        if let (Some(acc), Some(list)) = (&a.account_id, accounts)
            && !list.iter().any(|x| x == acc)
        {
            continue;
        }
        let moved_out = matches!(
            a.kind,
            CorporateActionKind::TickerChange | CorporateActionKind::Merger
        ) && a.new_asset_id.is_some();
        out.push(ActionRow {
            date,
            kind: a.kind,
            factor: parse_opt(a.factor, "nucleo-carteira-campo-fator")?,
            unit_cost: parse_opt(a.unit_cost, "nucleo-carteira-campo-custo")?,
            amount: a.amount.map(|v| from_minor(v, &base)),
            moved_out,
            come_cotas: None,
            quantity: parse_opt(a.quantity, "nucleo-carteira-campo-quantidade")?,
        });
    }
    Ok(out)
}

fn load_incomes(
    conn: &Connection,
    asset_id: &str,
    accounts: Option<&[String]>,
) -> Result<Vec<IncomeRow>> {
    let mut stmt = conn.prepare(
        "SELECT account_id, pay_date, net, currency, fx_rate, tax_withheld FROM income_event WHERE asset_id = ?1 AND status = 'received' ORDER BY pay_date",
    )?;
    type Row = (String, String, i64, String, Option<String>, i64);
    let rows: Vec<Row> = stmt
        .query_map([asset_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (acc, d, net, cur, fx, tax) in rows {
        if accounts.is_some_and(|a| !a.contains(&acc)) {
            continue;
        }
        out.push(IncomeRow {
            pay_date: parse_date(&d)?,
            net: from_minor(net, &cur),
            tax: from_minor(tax, &cur),
            fx: parse_opt(fx, "nucleo-carteira-campo-cambio")?,
        });
    }
    Ok(out)
}

/// Monta o livro do ativo, herdando a posição de ativos que viraram este (mudança de
/// ticker ou incorporação).
/// Operações de um ativo (para o come-cotas).
pub(crate) fn load_trades_pub(
    conn: &Connection,
    asset_id: &str,
    accounts: Option<&[String]>,
) -> Result<Vec<TradeRow>> {
    load_trades(conn, asset_id, accounts, None)
}

/// Cota do fundo num dia para o come-cotas: o fechamento do dia ou de até 10 dias antes.
pub(crate) fn fund_quota_on(
    conn: &Connection,
    asset_id: &str,
    d: NaiveDate,
) -> Result<Option<Decimal>> {
    Ok(
        crate::market::close_on(conn, asset_id, d)?.and_then(|(p, on)| {
            let age = parse_date(&on.date)
                .map(|x| (d - x).num_days())
                .unwrap_or(i64::MAX);
            (age <= 10).then_some(p)
        }),
    )
}

/// Come-cotas dos fundos sujeitos como eventos sintéticos do livro (somados por data).
fn come_cotas_actions(
    conn: &Connection,
    asset: &Asset,
    trades: &[TradeRow],
) -> Result<Vec<ActionRow>> {
    if !super::come_cotas::applies_to(asset) || trades.is_empty() {
        return Ok(vec![]);
    }
    let rule = super::come_cotas::rule(conn, asset)?;
    let mut warnings = vec![];
    let evs = super::come_cotas::events(
        trades,
        rule.regime,
        crate::time::today_local(),
        |d| fund_quota_on(conn, &asset.id, d),
        &mut warnings,
    )?;
    let mut by_date: BTreeMap<NaiveDate, Decimal> = BTreeMap::new();
    for e in evs {
        *by_date.entry(parse_date(&e.date)?).or_default() += super::come_cotas::parse(&e.shares)?;
    }
    Ok(by_date
        .into_iter()
        .map(|(date, shares)| ActionRow {
            date,
            kind: CorporateActionKind::CapitalReturn,
            factor: None,
            unit_cost: None,
            amount: None,
            moved_out: false,
            come_cotas: Some(shares),
            quantity: None,
        })
        .collect())
}

pub fn load_book(conn: &Connection, asset_id: &str, accounts: Option<&[String]>) -> Result<Book> {
    load_book_depth(conn, asset_id, accounts, 0)
}

fn load_book_depth(
    conn: &Connection,
    asset_id: &str,
    accounts: Option<&[String]>,
    depth: u32,
) -> Result<Book> {
    if depth > 8 {
        return Err(Error::invalid(t("nucleo-carteira-erro-cadeia-ticker")));
    }
    let asset = crate::assets::get(conn, asset_id)?;
    let trades = load_trades(conn, asset_id, accounts, None)?;
    let mut actions = load_actions(conn, asset_id, accounts, None)?;
    actions.extend(come_cotas_actions(conn, &asset, &trades)?);
    actions.sort_by_key(|a| a.date);
    let incomes = load_incomes(conn, asset_id, accounts)?;
    let mut stmt = conn.prepare(
        "SELECT asset_id, date, factor FROM corporate_action WHERE new_asset_id = ?1 AND kind IN ('ticker_change', 'merger')",
    )?;
    let preds: Vec<(String, String, Option<String>)> = stmt
        .query_map([asset_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut inherited = vec![];
    for (pred, d, factor) in preds {
        let date = parse_date(&d)?;
        let before = date.pred_opt().expect("data válida");
        let pb = load_book_depth(conn, &pred, accounts, depth + 1)?;
        let st = replay(conn, &pb, before)?;
        if st.native.qty.is_zero() {
            continue;
        }
        let factor = parse_opt(factor, "nucleo-carteira-campo-fator")?.unwrap_or(Decimal::ONE);
        let mut flows = flows_base(conn, &pb, before)?;
        for i in &pb.inherited {
            flows.extend(i.flows.iter().copied());
        }
        inherited.push(Inherited {
            date,
            qty: st.native.qty * factor,
            cost: st.native.cost,
            cost_base: st.base.cost,
            flows,
        });
    }
    inherited.sort_by_key(|i| i.date);
    Ok(Book {
        asset,
        trades,
        actions,
        incomes,
        inherited,
    })
}

impl Book {
    /// O livro só com as primeiras `trades` operações e `actions` eventos (extrato linha a
    /// linha: a posição depois de cada lançamento).
    pub(crate) fn prefix(&self, trades: usize, actions: usize, until: NaiveDate) -> Book {
        Book {
            asset: self.asset.clone(),
            trades: self.trades[..trades].to_vec(),
            actions: self.actions[..actions].to_vec(),
            incomes: vec![],
            inherited: self
                .inherited
                .iter()
                .filter(|i| i.date <= until)
                .cloned()
                .collect(),
        }
    }
}

pub(crate) fn is_curve(asset: &Asset) -> bool {
    asset.class.is_curve_valued()
        || (asset.class == AssetClass::TreasuryBr && asset.fixed_income.is_some())
}

/// Reproduz o livro até `until` (inclusive). Eventos corporativos de uma data valem antes
/// das operações da mesma data (data "ex").
/// Um passo da reprodução visto de fora (livro de vendas): o custo na moeda base antes e
/// depois, e, numa venda, a operação, o valor líquido e os lotes de renda fixa que saíram.
pub(crate) struct Step<'a> {
    pub(crate) date: NaiveDate,
    pub(crate) sell: Option<&'a TradeRow>,
    pub(crate) qty_sold: Decimal,
    pub(crate) cost_before: Decimal,
    pub(crate) cost_after: Decimal,
    /// Venda: valor líquido na moeda base.
    pub(crate) proceeds_base: Decimal,
    /// Venda de renda fixa (PEPS): data e custo na moeda base de cada lote que saiu.
    pub(crate) lots_out: Vec<(NaiveDate, Decimal)>,
    /// A posição saiu inteira para outro ativo.
    pub(crate) reset: bool,
    /// Quantidade (na moeda do ativo não muda: cotas) antes e depois do passo, para os
    /// métodos de custo por lote do livro de vendas (PEPS, pool do Reino Unido).
    pub(crate) qty_before: Decimal,
    pub(crate) qty_after: Decimal,
}

pub fn replay(conn: &Connection, book: &Book, until: NaiveDate) -> Result<State> {
    replay_observed(conn, book, until, &mut |_| Ok(()))
}

/// Como [`replay`], chamando `obs` depois de cada compra, venda, evento ou herança.
pub(crate) fn replay_observed(
    conn: &Connection,
    book: &Book,
    until: NaiveDate,
    obs: &mut dyn FnMut(&Step<'_>) -> Result<()>,
) -> Result<State> {
    let mut st = State::default();
    let step = |date: NaiveDate, before: Decimal, after: Decimal| Step {
        date,
        sell: None,
        qty_sold: Decimal::ZERO,
        cost_before: before,
        cost_after: after,
        proceeds_base: Decimal::ZERO,
        lots_out: vec![],
        reset: false,
        qty_before: Decimal::ZERO,
        qty_after: Decimal::ZERO,
    };
    let curve = is_curve(&book.asset);
    let ccy = book.asset.currency.clone();
    let (mut ti, mut ai) = (0, 0);
    // Posições herdadas entram no início do dia da mudança.
    let mut inh = book.inherited.iter().filter(|i| i.date <= until).peekable();
    loop {
        let next_day = [
            book.trades
                .get(ti)
                .filter(|t| t.date <= until)
                .map(|t| t.date),
            book.actions
                .get(ai)
                .filter(|a| a.date <= until)
                .map(|a| a.date),
        ]
        .into_iter()
        .flatten()
        .min();
        if let Some(i) = inh.peek()
            && next_day.is_none_or(|d| i.date <= d)
        {
            let i = inh.next().expect("peek");
            let before = st.base.cost;
            let qty_before = st.native.qty;
            st.native.qty += i.qty;
            st.native.cost += i.cost;
            st.base.qty += i.qty;
            st.base.cost += i.cost_base;
            obs(&Step {
                qty_before,
                qty_after: st.native.qty,
                ..step(i.date, before, st.base.cost)
            })?;
            continue;
        }
        let next_trade = book.trades.get(ti).filter(|t| t.date <= until);
        let next_action = book.actions.get(ai).filter(|a| a.date <= until);
        let take_action = match (next_trade, next_action) {
            (None, None) => break,
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (Some(t), Some(a)) => a.date <= t.date,
        };
        if take_action {
            let a = &book.actions[ai];
            ai += 1;
            let before = st.base.cost;
            let qty_before = st.native.qty;
            apply_action(&mut st, a, curve)?;
            obs(&Step {
                reset: a.moved_out,
                qty_before,
                qty_after: st.native.qty,
                ..step(a.date, before, st.base.cost)
            })?;
            continue;
        }
        let t = &book.trades[ti];
        ti += 1;
        let fx = match t.fx {
            Some(f) => f,
            None => match fx_to_base(conn, &ccy, t.date)? {
                Some((f, est)) => {
                    st.estimated |= est;
                    f
                }
                None => {
                    st.estimated = true;
                    st.warnings.push(t_args(
                        "nucleo-carteira-aviso-sem-cambio",
                        &i18n::args(&[
                            ("moeda", ccy.to_string().into()),
                            ("base", crate::household::base_currency(conn)?.into()),
                            ("data", fmt_date(t.date).into()),
                        ]),
                    ));
                    Decimal::ZERO
                }
            },
        };
        let before = st.base.cost;
        let qty_before = st.native.qty;
        match t.side {
            TradeSide::Buy => {
                st.native.apply(&PositionEvent::Buy {
                    qty: t.qty,
                    price: t.price,
                    fees: t.costs,
                })?;
                st.base.apply(&PositionEvent::Buy {
                    qty: t.qty,
                    price: t.price * fx,
                    fees: t.costs * fx,
                })?;
                if curve {
                    let principal = t.qty * t.price + t.costs;
                    st.lots.push(Lot {
                        qty: t.qty,
                        principal,
                        principal_base: principal * fx,
                        date: t.date,
                    });
                }
                obs(&Step {
                    qty_before,
                    qty_after: st.native.qty,
                    ..step(t.date, before, st.base.cost)
                })?;
            }
            TradeSide::Sell => {
                let qty = t.qty.min(st.native.qty);
                if qty < t.qty {
                    st.warnings.push(t_args(
                        "nucleo-carteira-aviso-venda-maior",
                        &i18n::args(&[("data", fmt_date(t.date).into())]),
                    ));
                    st.uncovered.push((t.id.clone(), t.qty - qty));
                }
                if qty.is_zero() {
                    continue;
                }
                if curve {
                    // Renda fixa: PEPS (primeiro que entra, primeiro que sai).
                    let mut left = qty;
                    let mut cost_out = Decimal::ZERO;
                    let mut cost_out_base = Decimal::ZERO;
                    let mut lots_out = vec![];
                    while left > Decimal::ZERO && !st.lots.is_empty() {
                        let lot = &mut st.lots[0];
                        let take = left.min(lot.qty);
                        let share = take / lot.qty;
                        cost_out += lot.principal * share;
                        cost_out_base += lot.principal_base * share;
                        lots_out.push((lot.date, lot.principal_base * share));
                        lot.principal -= lot.principal * share;
                        lot.principal_base -= lot.principal_base * share;
                        lot.qty -= take;
                        left -= take;
                        if lot.qty.is_zero() {
                            st.lots.remove(0);
                        }
                    }
                    let proceeds = qty * t.price - t.costs;
                    st.native.qty -= qty;
                    st.native.cost = (st.native.cost - cost_out).max(Decimal::ZERO);
                    st.native.realized += proceeds - cost_out;
                    st.base.qty -= qty;
                    st.base.cost = (st.base.cost - cost_out_base).max(Decimal::ZERO);
                    st.base.realized += proceeds * fx - cost_out_base;
                    obs(&Step {
                        sell: Some(t),
                        qty_sold: qty,
                        proceeds_base: proceeds * fx,
                        lots_out,
                        qty_before,
                        qty_after: st.native.qty,
                        ..step(t.date, before, st.base.cost)
                    })?;
                } else {
                    st.native.apply(&PositionEvent::Sell {
                        qty,
                        price: t.price,
                        fees: t.costs,
                    })?;
                    let realized = st.base.apply(&PositionEvent::Sell {
                        qty,
                        price: t.price * fx,
                        fees: t.costs * fx,
                    })?;
                    obs(&Step {
                        sell: Some(t),
                        qty_sold: qty,
                        proceeds_base: realized + (before - st.base.cost),
                        qty_before,
                        qty_after: st.native.qty,
                        ..step(t.date, before, st.base.cost)
                    })?;
                }
            }
        }
    }
    Ok(st)
}

fn apply_action(st: &mut State, a: &ActionRow, curve: bool) -> Result<()> {
    if let Some(shares) = a.come_cotas {
        // IR pago com cotas: a quantidade cai, o custo total fica (o preço médio sobe).
        let take = shares.min(st.native.qty);
        st.native.qty -= take;
        st.base.qty -= take.min(st.base.qty);
        return Ok(());
    }
    if a.moved_out {
        st.native = Position {
            realized: st.native.realized,
            ..Default::default()
        };
        st.base = Position {
            realized: st.base.realized,
            ..Default::default()
        };
        st.lots.clear();
        return Ok(());
    }
    match a.kind {
        CorporateActionKind::Split | CorporateActionKind::ReverseSplit => {
            if let Some(f) = a.factor {
                st.native.apply(&PositionEvent::Split { factor: f })?;
                st.base.apply(&PositionEvent::Split { factor: f })?;
                if curve {
                    for l in &mut st.lots {
                        l.qty *= f;
                    }
                }
            }
        }
        CorporateActionKind::Bonus => {
            if let Some(qty) = a.quantity.or_else(|| a.factor.map(|f| st.native.qty * f)) {
                let unit = a.unit_cost.unwrap_or(Decimal::ZERO);
                st.native.apply(&PositionEvent::Bonus {
                    qty,
                    unit_cost: unit,
                })?;
                st.base.apply(&PositionEvent::Bonus {
                    qty,
                    unit_cost: unit,
                })?;
            }
        }
        CorporateActionKind::CapitalReturn => {
            if let Some(v) = a.amount {
                st.native
                    .apply(&PositionEvent::CapitalReturn { amount: v })?;
                st.base.apply(&PositionEvent::CapitalReturn { amount: v })?;
            }
        }
        CorporateActionKind::SpinOff | CorporateActionKind::Subscription => {
            st.warnings.push(t("nucleo-carteira-aviso-cisao"));
        }
        CorporateActionKind::TickerChange | CorporateActionKind::Merger => {}
    }
    Ok(())
}

/// Quantidade em carteira até a data (para validar vendas).
pub fn quantity_held(
    conn: &Connection,
    asset_id: &str,
    account_id: Option<&str>,
    date: &str,
) -> Result<Decimal> {
    let accounts = account_id.map(|a| vec![a.to_string()]);
    let book = load_book(conn, asset_id, accounts.as_deref())?;
    Ok(replay(conn, &book, parse_date(date)?)?.quantity())
}

/// Valor na curva de um lote de renda fixa na data.
fn curve_value(
    conn: &Connection,
    terms: &FixedIncomeTerms,
    principal: Decimal,
    start: NaiveDate,
    at: NaiveDate,
) -> Result<(Decimal, bool)> {
    let end = match terms.maturity_date.as_deref().map(parse_date).transpose()? {
        Some(m) if m < at => m,
        _ => at,
    };
    if end <= start {
        return Ok((principal, false));
    }
    let rate = terms.rate_dec()?;
    Ok(match terms.indexer {
        Indexer::Cdi | Indexer::Selic => {
            let sid = if terms.indexer == Indexer::Cdi {
                "cdi"
            } else {
                "selic"
            };
            let rates = indices::daily_rates(conn, sid, start, end)?;
            if rates.is_empty() {
                (principal, true)
            } else {
                fixed_income::accrue_cdi(principal, &rates, rate, start, end)
            }
        }
        Indexer::Prefixado => (
            fixed_income::prefixed_value(principal, rate, start, end),
            false,
        ),
        Indexer::Ipca | Indexer::Igpm => {
            let f = if terms.indexer == Indexer::Ipca {
                indices::deflator_factor(conn, Deflator::Ipca, start, end)?
            } else {
                let rates = indices::monthly_rates(conn, "igpm", start, end)?;
                crate::finance::inflation::accumulated_factor(&rates, start, end)
            };
            match f {
                Some(f) => (
                    fixed_income::ipca_plus_value(principal, rate, f.factor, start, end),
                    f.estimated,
                ),
                None => (
                    fixed_income::prefixed_value(principal, rate, start, end),
                    true,
                ),
            }
        }
    })
}

#[derive(Debug, Clone)]
pub struct Valued {
    pub value: Option<Decimal>,
    pub value_base: Option<Decimal>,
    pub price: Option<PriceOn>,
    pub valuation: Valuation,
    pub net_redemption_base: Option<Decimal>,
    pub estimated: bool,
}

/// Avalia o estado na data: mercado, curva ou custo (sem cotação).
pub fn value_state(conn: &Connection, book: &Book, st: &State, at: NaiveDate) -> Result<Valued> {
    let asset = &book.asset;
    if st.native.qty.is_zero() {
        return Ok(Valued {
            value: Some(Decimal::ZERO),
            value_base: Some(Decimal::ZERO),
            price: None,
            valuation: Valuation::Market,
            net_redemption_base: None,
            estimated: false,
        });
    }
    let fx = fx_to_base(conn, &asset.currency, at)?;
    let mut estimated = fx.is_some_and(|(_, e)| e);
    let market = if asset.class.is_curve_valued() {
        None
    } else {
        crate::market::price_on(conn, &asset.id, at)?
    };
    if let Some((close, p)) = market {
        estimated |= p.stale && (at - parse_date(&p.date)?).num_days() > 7;
        // Cotação intradiária com o pregão em andamento: preço ainda não é o de fechamento.
        estimated |= p.intraday.as_ref().is_some_and(|m| {
            matches!(
                m.market_state,
                crate::market::exchange::SessionState::Open
                    | crate::market::exchange::SessionState::Lunch
            )
        });
        let v = st.native.qty * close;
        return Ok(Valued {
            value: Some(v),
            value_base: fx.map(|(r, _)| v * r),
            price: Some(p),
            valuation: Valuation::Market,
            net_redemption_base: None,
            estimated,
        });
    }
    if let (true, Some(terms)) = (is_curve(asset), &asset.fixed_income) {
        let mut gross = Decimal::ZERO;
        let mut net = Decimal::ZERO;
        for lot in &st.lots {
            let (v, est) = curve_value(conn, terms, lot.principal, lot.date, at)?;
            estimated |= est;
            gross += v;
            let days = (at - lot.date).num_days();
            net += fixed_income::net_redemption(lot.principal, v, days, terms.ir_exempt).net;
        }
        let r = fx.map(|(r, _)| r);
        return Ok(Valued {
            value: Some(gross),
            value_base: r.map(|r| gross * r),
            price: None,
            valuation: Valuation::Curve,
            net_redemption_base: r.map(|r| net * r),
            estimated,
        });
    }
    // Sem cotação: avalia pelo custo e avisa.
    Ok(Valued {
        value: Some(st.native.cost),
        value_base: Some(st.base.cost),
        price: None,
        valuation: Valuation::Missing,
        net_redemption_base: None,
        estimated: true,
    })
}

/// Fluxos do investidor na moeda base para o XIRR (aportes negativos, resgates e proventos
/// positivos).
fn flows_base(conn: &Connection, book: &Book, until: NaiveDate) -> Result<Vec<Flow>> {
    let ccy = &book.asset.currency;
    let mut out = vec![];
    for t in book.trades.iter().filter(|t| t.date <= until) {
        let fx = match t.fx {
            Some(f) => f,
            None => fx_to_base(conn, ccy, t.date)?
                .map(|(r, _)| r)
                .unwrap_or(Decimal::ZERO),
        };
        let gross = t.qty * t.price;
        let amt = match t.side {
            TradeSide::Buy => -(gross + t.costs),
            TradeSide::Sell => gross - t.costs,
        } * fx;
        out.push(Flow {
            date: t.date,
            amount: amt.to_f64().unwrap_or(0.0),
        });
    }
    for i in book.incomes.iter().filter(|i| i.pay_date <= until) {
        let fx = match i.fx {
            Some(f) => f,
            None => fx_to_base(conn, ccy, i.pay_date)?
                .map(|(r, _)| r)
                .unwrap_or(Decimal::ZERO),
        };
        out.push(Flow {
            date: i.pay_date,
            amount: (i.net * fx).to_f64().unwrap_or(0.0),
        });
    }
    Ok(out)
}

/// Traz cada fluxo a valores da data final pelo deflator (para o XIRR real).
fn deflate_flows(
    conn: &Connection,
    flows: &[Flow],
    to: NaiveDate,
    deflator: Deflator,
) -> Result<Option<Vec<Flow>>> {
    let mut out = Vec::with_capacity(flows.len());
    for f in flows {
        match indices::deflator_factor(conn, deflator, f.date, to)? {
            Some(fac) => out.push(Flow {
                date: f.date,
                amount: f.amount * fac.factor.to_f64().unwrap_or(1.0),
            }),
            None => return Ok(None),
        }
    }
    Ok(Some(out))
}

fn income_base(conn: &Connection, book: &Book, until: NaiveDate) -> Result<Decimal> {
    let mut s = Decimal::ZERO;
    for i in book.incomes.iter().filter(|i| i.pay_date <= until) {
        let fx = match i.fx {
            Some(f) => f,
            None => fx_to_base(conn, &book.asset.currency, i.pay_date)?
                .map(|(r, _)| r)
                .unwrap_or(Decimal::ZERO),
        };
        s += i.net * fx;
    }
    Ok(s)
}

pub(crate) fn brief_of(a: &Asset) -> AssetBrief {
    AssetBrief {
        id: a.id.clone(),
        ticker: a.ticker.clone(),
        name: a.name.clone(),
        class: a.class,
        class_label: a.class_label.clone(),
        currency: a.currency.clone(),
    }
}

/// Posição completa de um livro na data.
pub fn position_view(
    conn: &Connection,
    book: &Book,
    at: NaiveDate,
    deflator: Deflator,
    account_id: Option<String>,
) -> Result<(PositionView, Vec<Flow>)> {
    // Idade da posição: desde a última vez que a quantidade saiu do zero (recompra depois de
    // zerar recomeça); a herdada de outro ticker conta desde a primeira compra de lá.
    let mut opened: Option<NaiveDate> = None;
    let st = replay_observed(conn, book, at, &mut |s| {
        if s.qty_before.is_zero() && !s.qty_after.is_zero() {
            opened = Some(s.date);
        }
        Ok(())
    })?;
    if let Some(d) = opened
        && let Some(i) = book.inherited.iter().find(|i| i.date == d)
        && let Some(first) = i.flows.iter().map(|f| f.date).min()
    {
        opened = Some(first.min(d));
    }
    let v = value_state(conn, book, &st, at)?;
    let ccy = &book.asset.currency;
    let base = crate::household::base_currency(conn)?;
    let income = income_base(conn, book, at)?;
    let mut own = flows_base(conn, book, at)?;
    // Efeito da inflação no resultado: o nominal menos cada fluxo próprio (compras, vendas,
    // proventos) trazido para a data pelo deflator. O valor de mercado já está na data.
    let inflation_effect = if deflator == Deflator::None {
        Some(0.0)
    } else {
        deflate_flows(conn, &own, at, deflator)?.map(|real| {
            real.iter()
                .zip(&own)
                .map(|(r, n)| n.amount - r.amount)
                .sum::<f64>()
        })
    };
    let open = !st.native.qty.is_zero();
    if open && let Some(vb) = v.value_base {
        own.push(Flow {
            date: at,
            amount: vb.to_f64().unwrap_or(0.0),
        });
    }
    // XIRR do ativo inclui os fluxos herdados; o agregado da carteira usa só os próprios.
    let mut flows: Vec<Flow> = book
        .inherited
        .iter()
        .filter(|i| i.date <= at)
        .flat_map(|i| i.flows.iter().copied())
        .collect();
    flows.extend(own.iter().copied());
    let irr = xirr(&flows);
    let irr_real = match &irr {
        Some(_) if deflator != Deflator::None => {
            let mut nominal_only: Vec<Flow> = flows.clone();
            if open {
                nominal_only.pop();
            }
            match deflate_flows(conn, &nominal_only, at, deflator)? {
                Some(mut real) => {
                    if open && let Some(vb) = v.value_base {
                        real.push(Flow {
                            date: at,
                            amount: vb.to_f64().unwrap_or(0.0),
                        });
                    }
                    xirr(&real)
                }
                None => None,
            }
        }
        Some(r) => Some(*r),
        None => None,
    };
    let cost_base = to_minor(st.base.cost, &base);
    let mv_base = v.value_base.map(|x| to_minor(x, &base));
    let unrealized = mv_base.map(|m| m - cost_base);
    let realized_base = to_minor(st.base.realized, &base);
    let income_minor = to_minor(income, &base);
    let total_result_base = unrealized.map(|u| u + realized_base + income_minor);
    let inflation_effect_base =
        inflation_effect.map(|e| to_minor(Decimal::from_f64(e).unwrap_or_default(), &base));
    let real_result_base = match (total_result_base, inflation_effect_base) {
        (Some(t), Some(e)) => Some(t - e),
        _ => None,
    };
    let mut warnings = st.warnings.clone();
    if v.valuation == Valuation::Missing {
        warnings.push(t("nucleo-carteira-aviso-sem-cotacao"));
    }
    let first_date = book.trades.first().map(|t| fmt_date(t.date));
    let holding_days = opened
        .or_else(|| book.trades.first().map(|t| t.date))
        .map(|d| (at - d).num_days().max(0));
    let basis = ReturnBasis::for_days(holding_days);
    let period = |r: Option<i64>| match r {
        Some(r) if cost_base > 0 => Some(r as f64 / cost_base as f64),
        _ => None,
    };
    // Abaixo do mínimo, a taxa anualizada explode (−99% a.a. com 5 dias de queda de 6%):
    // só o retorno do período.
    let (irr, irr_real) = if basis == ReturnBasis::Period {
        (None, None)
    } else {
        (irr, irr_real)
    };
    let pv = PositionView {
        asset: brief_of(&book.asset),
        account_id,
        quantity: st.native.qty.normalize().to_string(),
        avg_price: st
            .native
            .avg_price()
            .map(|p| p.round_dp(8).normalize().to_string()),
        cost: to_minor(st.native.cost, ccy),
        cost_base,
        price: v.price.clone(),
        market_value: v.value.map(|x| to_minor(x, ccy)),
        market_value_base: mv_base,
        unrealized_base: unrealized,
        realized_base,
        income_base: income_minor,
        total_result_base,
        real_result_base,
        inflation_effect_base,
        xirr: irr,
        xirr_real: irr_real,
        holding_days,
        period_return: period(total_result_base),
        period_return_real: period(real_result_base),
        return_basis: basis,
        weight: None,
        valuation: v.valuation,
        net_redemption_base: v.net_redemption_base.map(|x| to_minor(x, &base)),
        first_date,
        estimated: st.estimated || v.estimated,
        warnings,
    };
    Ok((pv, own))
}

/// Ativos com operações (ou herdados por mudança de ticker) nas contas filtradas.
pub(crate) fn traded_assets(
    conn: &Connection,
    accounts: Option<&[String]>,
) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT asset_id, account_id FROM trade
         UNION SELECT DISTINCT asset_id, account_id FROM corporate_action WHERE account_id IS NOT NULL
         UNION SELECT DISTINCT ca.new_asset_id, t.account_id FROM corporate_action ca JOIN trade t ON t.asset_id = ca.asset_id
           WHERE ca.new_asset_id IS NOT NULL AND ca.kind IN ('ticker_change', 'merger')",
    )?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter(|(_, acc)| accounts.is_none_or(|a| a.contains(acc)))
        .collect())
}

pub(crate) fn targets_by_class(conn: &Connection) -> Result<HashMap<String, (f64, f64)>> {
    let mut m = HashMap::new();
    for t in super::records::list_targets(conn)? {
        if t.scope == TargetScope::Class {
            let pct = t.target_pct.parse::<f64>().unwrap_or(0.0);
            let tol = t.tolerance_pct.parse::<f64>().unwrap_or(5.0);
            m.insert(t.key, (pct, tol));
        }
    }
    Ok(m)
}

/// Soma na posição os aportes de previdência ainda sem cota, pelo valor aportado (sem
/// rendimento), com o aviso.
pub fn add_pending(p: &mut PositionView, amount: i64) {
    p.cost += amount;
    p.cost_base += amount;
    p.market_value = Some(p.market_value.unwrap_or(0) + amount);
    p.market_value_base = Some(p.market_value_base.unwrap_or(0) + amount);
    p.estimated = true;
    p.warnings.push(crate::i18n::t_args(
        "nucleo-carteira-aviso-aportes-sem-cota",
        &crate::i18n::args(&[(
            "valor",
            crate::i18n::money(crate::i18n::current(), amount, "BRL").into(),
        )]),
    ));
}

pub fn summary(conn: &Connection, args: &PortfolioArgs) -> Result<PortfolioSummary> {
    let at = match &args.as_of {
        Some(d) => parse_date(d)?,
        None => crate::time::today_local(),
    };
    let deflator = indices::effective_deflator(conn, args.deflator.unwrap_or_default());
    let accounts = args.account_ids.clone();
    let pairs = traded_assets(conn, accounts.as_deref())?;
    let mut units: Vec<(String, Option<String>)> = vec![];
    if args.by_account.unwrap_or(false) {
        for (a, acc) in &pairs {
            if !units
                .iter()
                .any(|(x, y)| x == a && y.as_deref() == Some(acc))
            {
                units.push((a.clone(), Some(acc.clone())));
            }
        }
    } else {
        for (a, _) in &pairs {
            if !units.iter().any(|(x, _)| x == a) {
                units.push((a.clone(), None));
            }
        }
    }
    let mut positions = vec![];
    let mut all_flows: Vec<Flow> = vec![];
    let mut missing = vec![];
    let mut balance_due: Vec<BalanceDue> = vec![];
    let mut estimated = false;
    let (mut mv, mut cost, mut realized, mut income) = (0i64, 0i64, 0i64, 0i64);
    // Efeito da inflação somado das posições (abertas e encerradas, como o realizado).
    let mut effect: Option<i64> = Some(0);
    for (asset_id, acc) in units {
        let scope: Option<Vec<String>> = match &acc {
            Some(a) => Some(vec![a.clone()]),
            None => accounts.clone(),
        };
        let book = load_book(conn, &asset_id, scope.as_deref())?;
        let (pv, flows) = position_view(conn, &book, at, deflator, acc)?;
        realized += pv.realized_base;
        income += pv.income_base;
        effect = effect.zip(pv.inflation_effect_base).map(|(a, b)| a + b);
        all_flows.extend(flows);
        let open = pv.quantity != "0";
        if open {
            mv += pv.market_value_base.unwrap_or(pv.cost_base);
            cost += pv.cost_base;
            // Previdência não tem cota pública (sobretudo a fechada): vale o último saldo
            // informado, e sem ele (ou com ele antigo) pede o saldo em vez de entrar na lista
            // dos avaliados pelo custo.
            let pension = pv.asset.class == crate::assets::AssetClass::Pension;
            let price_date = pv.price.as_ref().and_then(|p| parse_date(&p.date).ok());
            if pension
                && (pv.valuation == Valuation::Missing
                    || price_date.is_some_and(|d| (at - d).num_days() > BALANCE_DUE_DAYS))
            {
                if !balance_due.iter().any(|b| b.asset_id == pv.asset.id) {
                    balance_due.push(BalanceDue {
                        asset_id: pv.asset.id.clone(),
                        name: pv.asset.name.clone(),
                        as_of: price_date.map(fmt_date),
                    });
                }
            } else if pv.valuation == Valuation::Missing {
                missing.push(pv.asset.ticker.clone().unwrap_or(pv.asset.name.clone()));
            }
            estimated |= pv.estimated;
        }
        if open || args.include_closed.unwrap_or(false) {
            positions.push(pv);
        }
    }
    // Aportes de previdência que ainda esperam a cota da CVM: entram pelo valor aportado, na
    // posição do plano (ou numa linha dele, se ainda não tem cotas).
    let by_acc = args.by_account.unwrap_or(false);
    for (asset_id, acc, amount, first) in
        crate::pension::payments::pending_by_asset(conn, &fmt_date(at), accounts.as_deref())?
    {
        mv += amount;
        cost += amount;
        estimated = true;
        let same = |p: &PositionView| {
            p.asset.id == asset_id && (!by_acc || p.account_id.as_deref() == Some(acc.as_str()))
        };
        match positions.iter_mut().find(|p| same(p)) {
            Some(p) => add_pending(p, amount),
            None => {
                let asset = crate::assets::get(conn, &asset_id)?;
                let mut p = PositionView {
                    asset: brief_of(&asset),
                    account_id: by_acc.then(|| acc.clone()),
                    quantity: "0".into(),
                    avg_price: None,
                    cost: 0,
                    cost_base: 0,
                    price: None,
                    market_value: Some(0),
                    market_value_base: Some(0),
                    unrealized_base: Some(0),
                    realized_base: 0,
                    income_base: 0,
                    total_result_base: Some(0),
                    real_result_base: Some(0),
                    inflation_effect_base: Some(0),
                    xirr: None,
                    xirr_real: None,
                    holding_days: None,
                    period_return: None,
                    period_return_real: None,
                    return_basis: ReturnBasis::Period,
                    weight: None,
                    valuation: Valuation::Missing,
                    net_redemption_base: None,
                    first_date: Some(first),
                    estimated: true,
                    warnings: vec![],
                };
                add_pending(&mut p, amount);
                positions.push(p);
            }
        }
    }
    for p in &mut positions {
        if mv > 0 {
            p.weight = p.market_value_base.map(|v| v as f64 / mv as f64);
        }
    }
    positions.sort_by(|a, b| {
        b.market_value_base
            .unwrap_or(0)
            .cmp(&a.market_value_base.unwrap_or(0))
    });
    // XIRR da carteira: fluxos de todos os ativos (já incluem o valor final de cada posição aberta).
    let irr = xirr(&all_flows);
    let irr_real = if deflator == Deflator::None {
        irr
    } else {
        let finals: Vec<Flow> = all_flows
            .iter()
            .filter(|f| f.date == at && f.amount > 0.0)
            .copied()
            .collect();
        let rest: Vec<Flow> = all_flows
            .iter()
            .filter(|f| !(f.date == at && f.amount > 0.0))
            .copied()
            .collect();
        match deflate_flows(conn, &rest, at, deflator)? {
            Some(mut r) => {
                r.extend(finals);
                xirr(&r)
            }
            None => None,
        }
    };
    let targets = targets_by_class(conn)?;
    let mut by: BTreeMap<String, (AssetClass, i64, i64)> = BTreeMap::new();
    // Média do XIRR por classe, ponderada pelo valor, só com as posições que já têm taxa
    // anualizada (as recentes ficam de fora: [`MIN_DAYS_FOR_ANNUALIZED`]).
    let mut irr_by: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for p in &positions {
        // Encerrada (sem cotas e sem valor); a linha dos aportes sem cota tem valor.
        if p.quantity == "0" && p.market_value_base.unwrap_or(0) == 0 {
            continue;
        }
        let e = by
            .entry(p.asset.class.as_str().to_string())
            .or_insert((p.asset.class, 0, 0));
        e.1 += p.market_value_base.unwrap_or(p.cost_base);
        e.2 += p.cost_base;
        if let Some(x) = p.xirr.filter(|_| p.return_basis != ReturnBasis::Period) {
            let w = p.market_value_base.unwrap_or(p.cost_base).max(0) as f64;
            let a = irr_by
                .entry(p.asset.class.as_str().to_string())
                .or_default();
            a.0 += x * w;
            a.1 += w;
        }
    }
    for k in targets.keys() {
        if let Ok(c) = AssetClass::parse(k) {
            by.entry(k.clone()).or_insert((c, 0, 0));
        }
    }
    let by_class = by
        .into_iter()
        .map(|(k, (class, v, c))| {
            let weight = if mv > 0 { v as f64 / mv as f64 } else { 0.0 };
            let t = targets.get(&k);
            let dev = t.map(|(pct, _)| weight * 100.0 - pct);
            ClassSummary {
                xirr: irr_by.get(&k).filter(|(_, w)| *w > 0.0).map(|(s, w)| s / w),
                class,
                label: class.label(),
                market_value_base: v,
                cost_base: c,
                weight,
                target_pct: t.map(|(p, _)| *p),
                deviation_pp: dev,
                out_of_band: match (dev, t) {
                    (Some(d), Some((_, tol))) => d.abs() > *tol,
                    _ => false,
                },
            }
        })
        .collect();
    let mut out = PortfolioSummary {
        as_of: fmt_date(at),
        base_currency: crate::household::base_currency(conn)?,
        deflator,
        market_value: mv,
        cost,
        unrealized: mv - cost,
        realized,
        income,
        total_result: mv - cost + realized + income,
        real_result: effect.map(|e| mv - cost + realized + income - e),
        inflation_effect: effect,
        xirr: irr,
        xirr_real: irr_real,
        by_class,
        positions,
        missing_prices: missing,
        balance_due,
        estimated,
        concentration_alerts: vec![],
    };
    out.concentration_alerts = super::concentration::alerts(conn, &out)?;
    Ok(out)
}

/// Rentabilidade mensal (Modified Dietz) e acumulada, com CDI e inflação no período.
/// Livros da rentabilidade e da ponte do resultado: todos os ativos com operações, ou só um
/// ativo ou uma classe.
pub(crate) fn performance_books(
    conn: &Connection,
    asset_id: Option<&str>,
    class: Option<AssetClass>,
) -> Result<Vec<Book>> {
    let mut books = vec![];
    for (id, _) in traded_assets(conn, None)? {
        if books.iter().any(|b: &Book| b.asset.id == id) {
            continue;
        }
        if asset_id.is_some_and(|a| a != id) {
            continue;
        }
        let b = load_book(conn, &id, None)?;
        if class.is_some_and(|c| c != b.asset.class) {
            continue;
        }
        books.push(b);
    }
    Ok(books)
}

pub fn performance(conn: &Connection, args: &PerformanceArgs) -> Result<Performance> {
    let today = crate::time::today_local();
    let deflator = indices::effective_deflator(conn, args.deflator.unwrap_or_default());
    let base = crate::household::base_currency(conn)?;
    let books = performance_books(conn, args.asset_id.as_deref(), args.class)?;
    let first = books
        .iter()
        .filter_map(|b| b.trades.first().map(|t| t.date))
        .min();
    let start_month = match (&args.from_month, first) {
        (Some(m), _) => parse_month(m)?,
        (None, Some(d)) => (d.year(), d.month()),
        (None, None) => (today.year(), today.month()),
    };
    let end_month = match &args.to_month {
        Some(m) => parse_month(m)?,
        None => (today.year(), today.month()),
    };
    let mut months = vec![];
    let (mut y, mut m) = start_month;
    while (y, m) <= end_month {
        months.push((y, m));
        let n = add_months(NaiveDate::from_ymd_opt(y, m, 1).expect("data"), 1);
        (y, m) = (n.year(), n.month());
        if months.len() > 600 {
            return Err(Error::invalid(t("nucleo-carteira-erro-periodo-longo")));
        }
    }
    let period_start = NaiveDate::from_ymd_opt(start_month.0, start_month.1, 1).expect("data")
        - chrono::Duration::days(1);
    let value_at = |d: NaiveDate| -> Result<(Decimal, bool)> {
        let mut total = Decimal::ZERO;
        let mut est = false;
        for b in &books {
            let st = replay(conn, b, d)?;
            let v = value_state(conn, b, &st, d)?;
            total += v.value_base.unwrap_or(Decimal::ZERO);
            est |= v.estimated;
        }
        Ok((total, est))
    };
    let (mut v_prev, mut estimated) = value_at(period_start)?;
    let mut out = Performance {
        months: vec![],
        monthly: vec![],
        cumulative: vec![],
        cumulative_real: vec![],
        cdi_cumulative: vec![],
        benchmark_label: String::new(),
        ibov_cumulative: vec![],
        sp500_brl_cumulative: vec![],
        usd_cumulative: vec![],
        inflation_cumulative: vec![],
        deflator,
        value_base: vec![],
        net_contributions: vec![],
        estimated: false,
        method: TwrMethod::Monthly,
        risk: None,
    };
    let daily = match args.method {
        Some(m) => m == TwrMethod::Daily,
        None => crate::settings::get_or(conn, DAILY_TWR_SETTING, false)?,
    };
    if daily {
        out.method = TwrMethod::Daily;
    }
    // Série para o risco: (data, retorno do período), mensal ou diária.
    let mut series: Vec<(NaiveDate, f64)> = vec![];
    let mut v_day = v_prev;
    let mut acc = 1.0f64;
    let mut contrib_total = Decimal::ZERO;
    let cdi_rates = indices::daily_rates(conn, "cdi", period_start, today)?;
    // Fora do Brasil: o juro de referência do país (taxa anual), no lugar do CDI.
    let country_rate = if crate::country::is_brazil(conn) {
        None
    } else {
        indices::catalog::reference_rate(conn)?
    };
    let bench_label = if crate::country::is_brazil(conn) {
        "CDI".to_string()
    } else {
        crate::country::info(conn, &crate::country::family_country(conn))
            .map(|i| i.rate_label)
            .unwrap_or_else(|_| t("nucleo-carteira-juro-referencia"))
    };
    out.benchmark_label = bench_label;
    let country_points = match &country_rate {
        Some((_, id)) => {
            indices::daily_rates(conn, id, period_start - chrono::Duration::days(40), today)?
        }
        None => vec![],
    };
    let usd_on = |d: NaiveDate| -> Result<Option<Decimal>> {
        Ok(crate::fx::rate(conn, "USD", &base, &fmt_date(d))?.map(|(r, ..)| r))
    };
    // O Ibovespa é a referência de quem investe no Brasil; fora dele, a linha some.
    let ibov_0 = if crate::country::is_brazil(conn) {
        indices::level_on(conn, "ibov", period_start)?
    } else {
        None
    };
    let usd_0 = usd_on(period_start)?;
    let sp_0 = match (
        indices::level_on(conn, "sp500", period_start)?,
        usd_on(period_start)?,
    ) {
        (Some(l), Some(fx)) => Some(l * fx),
        _ => None,
    };
    let growth = |base: Option<Decimal>, now: Option<Decimal>| -> Option<f64> {
        match (base, now) {
            (Some(b), Some(n)) if !b.is_zero() => (n / b - Decimal::ONE).to_f64(),
            _ => None,
        }
    };
    for (y, m) in months {
        let (first_day, last_day) = month_bounds(y, m);
        let end = last_day.min(today);
        let (v_end, est) = value_at(end)?;
        estimated |= est;
        let dim = (end - first_day).num_days() + 1;
        let mut flows = Decimal::ZERO;
        let mut weighted = Decimal::ZERO;
        let mut income = Decimal::ZERO;
        for b in &books {
            for t in b
                .trades
                .iter()
                .filter(|t| t.date >= first_day && t.date <= end)
            {
                let fx =
                    t.fx.or_else(|| {
                        fx_to_base(conn, &b.asset.currency, t.date)
                            .ok()
                            .flatten()
                            .map(|(r, _)| r)
                    })
                    .unwrap_or(Decimal::ZERO);
                let gross = t.qty * t.price;
                let f = match t.side {
                    TradeSide::Buy => gross + t.costs,
                    TradeSide::Sell => -(gross - t.costs),
                } * fx;
                let w = Decimal::from((end - t.date).num_days() + 1) / Decimal::from(dim);
                flows += f;
                weighted += f * w;
            }
            for i in b
                .incomes
                .iter()
                .filter(|i| i.pay_date >= first_day && i.pay_date <= end)
            {
                let fx =
                    i.fx.or_else(|| {
                        fx_to_base(conn, &b.asset.currency, i.pay_date)
                            .ok()
                            .flatten()
                            .map(|(r, _)| r)
                    })
                    .unwrap_or(Decimal::ZERO);
                income += i.net * fx;
            }
        }
        contrib_total += flows;
        let r = if daily {
            let mut acc_m = 1.0f64;
            let mut any = false;
            let mut d = first_day;
            while d <= end {
                let (inflow, outflow) = day_flows(conn, &books, d);
                if d == end || is_business_day(d) || !inflow.is_zero() || !outflow.is_zero() {
                    let (v, est) = value_at(d)?;
                    estimated |= est;
                    let denom = v_day + inflow;
                    if denom > Decimal::ZERO
                        && let Some(x) = ((v + outflow) / denom - Decimal::ONE).to_f64()
                    {
                        acc_m *= 1.0 + x;
                        any = true;
                        series.push((d, x));
                    }
                    v_day = v;
                }
                d = d.succ_opt().expect("data válida");
            }
            any.then_some(acc_m - 1.0)
        } else {
            let denom = v_prev + weighted;
            if denom > Decimal::ZERO {
                ((v_end - v_prev - flows + income) / denom).to_f64()
            } else {
                None
            }
        };
        if let Some(r) = r {
            acc *= 1.0 + r;
            if !daily {
                series.push((end, r));
            }
        }
        let cum = acc - 1.0;
        let infl = indices::deflator_factor(
            conn,
            deflator,
            period_start + chrono::Duration::days(1),
            end + chrono::Duration::days(1),
        )?;
        let cdi_slice: Vec<(NaiveDate, Decimal)> = cdi_rates
            .iter()
            .filter(|(d, _)| *d <= end)
            .cloned()
            .collect();
        let cdi = if !crate::country::is_brazil(conn) {
            indices::annual_rate_growth(&country_points, period_start, end)
        } else if cdi_slice.is_empty() {
            None
        } else {
            let (v, _) = fixed_income::accrue_cdi(
                Decimal::ONE,
                &cdi_slice,
                Decimal::ONE_HUNDRED,
                period_start,
                end,
            );
            (v - Decimal::ONE).to_f64()
        };
        out.months.push(crate::time::fmt_month(y, m));
        out.monthly.push(r);
        out.cumulative.push(cum);
        out.inflation_cumulative.push(
            infl.as_ref()
                .and_then(|f| (f.factor - Decimal::ONE).to_f64()),
        );
        out.cumulative_real.push(
            infl.as_ref()
                .and_then(|f| f.factor.to_f64())
                .map(|f| (1.0 + cum) / f - 1.0),
        );
        if infl.as_ref().is_some_and(|f| f.estimated) {
            estimated = true;
        }
        out.cdi_cumulative.push(cdi);
        out.ibov_cumulative
            .push(growth(ibov_0, indices::level_on(conn, "ibov", end)?));
        let sp_now = match (indices::level_on(conn, "sp500", end)?, usd_on(end)?) {
            (Some(l), Some(fx)) => Some(l * fx),
            _ => None,
        };
        out.sp500_brl_cumulative.push(growth(sp_0, sp_now));
        out.usd_cumulative.push(growth(usd_0, usd_on(end)?));
        out.value_base.push(to_minor(v_end, &base));
        out.net_contributions.push(to_minor(contrib_total, &base));
        v_prev = v_end;
    }
    out.estimated = estimated;
    out.risk = return_risk(&series, period_start, if daily { 252.0 } else { 12.0 });
    Ok(out)
}

/// Ajuste avançado: rentabilidade pelo retorno diário em vez do mensal (Modified Dietz).
pub const DAILY_TWR_SETTING: &str = "twr_daily";

/// Entradas (compras com custos) e saídas (vendas líquidas e proventos líquidos) do dia, na
/// moeda base.
fn day_flows(conn: &Connection, books: &[Book], d: NaiveDate) -> (Decimal, Decimal) {
    let (mut inflow, mut outflow) = (Decimal::ZERO, Decimal::ZERO);
    for b in books {
        let fx_of = |fx: Option<Decimal>| {
            fx.or_else(|| {
                fx_to_base(conn, &b.asset.currency, d)
                    .ok()
                    .flatten()
                    .map(|(r, _)| r)
            })
            .unwrap_or(Decimal::ZERO)
        };
        for t in b.trades.iter().filter(|t| t.date == d) {
            let gross = t.qty * t.price;
            match t.side {
                TradeSide::Buy => inflow += (gross + t.costs) * fx_of(t.fx),
                TradeSide::Sell => outflow += (gross - t.costs) * fx_of(t.fx),
            }
        }
        for i in b.incomes.iter().filter(|i| i.pay_date == d) {
            outflow += i.net * fx_of(i.fx);
        }
    }
    (inflow, outflow)
}

/// Semidesvio, volatilidade e pior queda de uma série de retornos por período, com o nível 1
/// em `start`. `per_year`: 12 (mensal) ou 252 (diário). `None` com menos de 2 períodos.
pub(crate) fn return_risk(
    series: &[(NaiveDate, f64)],
    start: NaiveDate,
    per_year: f64,
) -> Option<ReturnRisk> {
    if series.len() < 2 {
        return None;
    }
    let n = series.len() as f64;
    let mean = series.iter().map(|(_, r)| r).sum::<f64>() / n;
    let var = series.iter().map(|(_, r)| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let down = series
        .iter()
        .filter(|(_, r)| *r < mean)
        .map(|(_, r)| (r - mean).powi(2))
        .sum::<f64>()
        / n;
    let (mut level, mut peak, mut peak_at) = (1.0f64, 1.0f64, start);
    let (mut worst, mut worst_peak, mut worst_trough) = (0.0f64, None, None);
    let (mut longest, mut ongoing) = (0i64, false);
    let mut underwater = false;
    for (d, r) in series {
        level *= 1.0 + r;
        if level >= peak {
            if underwater {
                longest = longest.max((*d - peak_at).num_days());
                underwater = false;
            }
            // O topo é o primeiro dia nesse nível (dias parados no topo não o movem).
            if level > peak {
                peak = level;
                peak_at = *d;
            }
            continue;
        }
        underwater = true;
        let dd = level / peak - 1.0;
        if dd < worst {
            worst = dd;
            worst_peak = Some(peak_at);
            worst_trough = Some(*d);
        }
    }
    let last = series.last().map(|(d, _)| *d).expect("série não vazia");
    if underwater {
        let open = (last - peak_at).num_days();
        if open >= longest {
            longest = open;
            ongoing = true;
        }
    }
    // Volta ao topo da pior queda: o primeiro dia depois do fundo com o nível de novo no topo.
    let recovered = worst_trough.and_then(|trough| {
        let mut lv = 1.0f64;
        let mut top = (worst_peak == Some(start)).then_some(1.0f64);
        for (d, r) in series {
            lv *= 1.0 + r;
            if Some(*d) == worst_peak {
                top = Some(lv);
            }
            if *d > trough && top.is_some_and(|p| lv >= p) {
                return Some(*d);
            }
        }
        None
    });
    let round6 = |v: f64| (v * 1e6).round() / 1e6;
    Some(ReturnRisk {
        periods: series.len() as u32,
        semideviation: round6(down.sqrt() * per_year.sqrt()),
        volatility: round6(var.sqrt() * per_year.sqrt()),
        max_drawdown: round6(worst),
        drawdown_peak: worst_peak.map(fmt_date),
        drawdown_trough: worst_trough.map(fmt_date),
        drawdown_recovered: recovered.map(fmt_date),
        longest_drawdown_days: longest,
        longest_drawdown_ongoing: ongoing,
    })
}

/// Reparte `amount` entre as linhas com falta (gap > 0), proporcional ao que falta.
fn distribute(amount: i64, gaps: &[i64]) -> Vec<i64> {
    let mut out = vec![0; gaps.len()];
    let positive: i64 = gaps.iter().map(|g| (*g).max(0)).sum();
    if amount <= 0 || positive <= 0 {
        return out;
    }
    let mut given = 0;
    for (i, g) in gaps.iter().enumerate() {
        if *g > 0 {
            out[i] = (amount as i128 * *g as i128 / positive as i128) as i64;
            given += out[i];
        }
    }
    // Sobra do arredondamento vai para quem mais precisa.
    if given < amount
        && let Some((i, _)) = gaps
            .iter()
            .enumerate()
            .filter(|(_, g)| **g > 0)
            .max_by_key(|(_, g)| **g)
    {
        out[i] += amount - given;
    }
    out
}

/// Como [`distribute`], mas ninguém passa do seu teto: o que um teto corta vai para quem
/// ainda tem espaço (na proporção do peso e, sem peso, do espaço). O que não cabe em ninguém
/// fica de fora (a soma pode ser menor que `amount`).
fn distribute_capped(amount: i64, weights: &[i64], caps: &[i64]) -> Vec<i64> {
    let mut out = vec![0i64; weights.len()];
    let mut left = amount.max(0);
    for pass in 0..2 {
        // 1ª passada: pelo peso; 2ª: pelo espaço que sobrou (peso zero também recebe).
        for _ in 0..weights.len().max(1) {
            if left <= 0 {
                return out;
            }
            let w: Vec<i64> = (0..weights.len())
                .map(|i| {
                    let room = caps[i] - out[i];
                    if room <= 0 {
                        0
                    } else if pass == 0 {
                        weights[i].max(0)
                    } else {
                        room
                    }
                })
                .collect();
            if w.iter().all(|x| *x <= 0) {
                break;
            }
            let mut moved = 0;
            for (i, v) in distribute(left, &w).into_iter().enumerate() {
                let take = v.min(caps[i] - out[i]).max(0);
                out[i] += take;
                moved += take;
            }
            left -= moved;
            if moved == 0 {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
pub(crate) fn distribute_capped_for_tests(amount: i64, weights: &[i64], caps: &[i64]) -> Vec<i64> {
    distribute_capped(amount, weights, caps)
}

pub fn rebalance(conn: &Connection, args: &RebalanceArgs) -> Result<Rebalance> {
    let s = summary(
        conn,
        &PortfolioArgs {
            as_of: None,
            account_ids: None,
            deflator: Some(Deflator::None),
            by_account: None,
            include_closed: None,
        },
    )?;
    let contribution = args.contribution.unwrap_or(0).max(0);
    let total_after = s.market_value + contribution;
    let targets = targets_by_class(conn)?;
    let sum_targets: f64 = targets.values().map(|(p, _)| p).sum();
    let line = |class: AssetClass, label: String, current: i64, weight: f64, tp: f64, out: bool| {
        let ideal = (total_after as f64 * tp / 100.0).round() as i64;
        RebalanceLine {
            class,
            label,
            current,
            current_pct: weight * 100.0,
            target_pct: tp,
            gap: ideal - current,
            suggested: 0,
            out_of_band: out,
        }
    };
    let mut lines: Vec<RebalanceLine> = s
        .by_class
        .iter()
        .filter(|c| c.target_pct.is_some())
        .map(|c| {
            line(
                c.class,
                c.label.clone(),
                c.market_value_base,
                c.weight,
                c.target_pct.unwrap_or(0.0),
                c.out_of_band,
            )
        })
        .collect();
    // Classes com alvo e ainda sem posição também recebem aporte.
    let mut missing: Vec<(&String, &(f64, f64))> = targets
        .iter()
        .filter(|(k, _)| !s.by_class.iter().any(|c| c.class.as_str() == k.as_str()))
        .collect();
    missing.sort_by(|a, b| a.0.cmp(b.0));
    for (k, (tp, tol)) in missing {
        let class = AssetClass::parse(k)?;
        lines.push(line(class, class.label(), 0, 0.0, *tp, *tp > *tol));
    }
    let gaps: Vec<i64> = lines.iter().map(|l| l.gap).collect();
    for (l, v) in lines.iter_mut().zip(distribute(contribution, &gaps)) {
        l.suggested = v;
    }

    // Dentro de cada classe, alvos por ativo dividem o valor ideal da classe e o aporte dela.
    let mut asset_targets: BTreeMap<AssetClass, Vec<(String, f64, f64)>> = BTreeMap::new();
    for t in super::records::list_targets(conn)? {
        if t.scope != TargetScope::Asset {
            continue;
        }
        let a = crate::assets::get(conn, &t.key)?;
        asset_targets.entry(a.class).or_default().push((
            t.key,
            t.target_pct.parse().unwrap_or(0.0),
            t.tolerance_pct.parse().unwrap_or(5.0),
        ));
    }
    // Espaço de cada ativo até o limite de concentração (só os que já têm posição).
    let rooms = super::concentration::rooms(conn, &s, total_after)?;
    let room_of = |id: &str| {
        rooms
            .iter()
            .find(|(a, ..)| a.id == id)
            .map(|r| r.3)
            .unwrap_or(i64::MAX / 4)
    };
    let mut unallocated = 0;
    let mut assets = vec![];
    for (class, ts) in asset_targets {
        let class_now: i64 = s
            .positions
            .iter()
            .filter(|p| p.asset.class == class)
            .map(|p| p.market_value_base.unwrap_or(p.cost_base))
            .sum();
        let (class_ideal, class_money) = match lines.iter().find(|l| l.class == class) {
            Some(l) => (l.current + l.gap, l.suggested),
            None => (class_now, 0),
        };
        let mut group: Vec<RebalanceAssetLine> = ts
            .iter()
            .map(|(id, tp, tol)| {
                let pos = s.positions.iter().find(|p| &p.asset.id == id);
                let current = pos
                    .map(|p| p.market_value_base.unwrap_or(p.cost_base))
                    .unwrap_or(0);
                let cur_pct = if class_now > 0 {
                    current as f64 * 100.0 / class_now as f64
                } else {
                    0.0
                };
                let brief = match pos {
                    Some(p) => Ok(p.asset.clone()),
                    None => crate::assets::get(conn, id).map(|a| brief_of(&a)),
                };
                brief.map(|asset| RebalanceAssetLine {
                    asset,
                    current,
                    current_pct: cur_pct,
                    target_pct: *tp,
                    gap: (class_ideal as f64 * tp / 100.0).round() as i64 - current,
                    suggested: 0,
                    out_of_band: (cur_pct - tp).abs() > *tol,
                })
            })
            .collect::<Result<_>>()?;
        // O aporte da classe vai pelos alvos sem passar do limite de concentração de cada
        // ativo; o que o limite corta vai para os outros ativos da classe com espaço.
        let gaps: Vec<i64> = group.iter().map(|l| l.gap).collect();
        let caps: Vec<i64> = group.iter().map(|l| room_of(&l.asset.id)).collect();
        let given = distribute_capped(class_money, &gaps, &caps);
        unallocated += class_money - given.iter().sum::<i64>();
        for (l, v) in group.iter_mut().zip(given) {
            l.suggested = v;
        }
        assets.extend(group);
    }
    // Sem alvo de classe, o aporte vai para quem tem espaço até o limite, na proporção do
    // espaço; o que não cabe em ninguém fica sem destino (e a tela avisa).
    let free: Vec<i64> = rooms.iter().map(|r| r.3).collect();
    let spread = if lines.is_empty() {
        let v = distribute_capped(contribution, &free, &free);
        unallocated += contribution - v.iter().sum::<i64>();
        v
    } else {
        vec![0; free.len()]
    };
    let concentration = rooms
        .into_iter()
        .zip(spread)
        .map(
            |((asset, current, limit, room, exempt), suggested)| RebalanceConcentrationLine {
                weight: if total_after > 0 {
                    (current + suggested) as f64 / total_after as f64
                } else {
                    0.0
                },
                over_limit: !exempt
                    && s.market_value > 0
                    && current as f64 / s.market_value as f64 > limit,
                asset,
                current,
                limit,
                room,
                suggested,
            },
        )
        .collect();
    Ok(Rebalance {
        total: s.market_value,
        contribution,
        lines,
        assets,
        concentration,
        unallocated,
        targets_incomplete: (sum_targets - 100.0).abs() > 0.01,
        disclaimer: t("nucleo-carteira-rebalanceamento-aviso"),
    })
}

pub fn dividend_calendar(
    conn: &Connection,
    args: &DividendCalendarArgs,
) -> Result<Vec<DividendCalendarItem>> {
    parse_date(&args.from)?;
    parse_date(&args.to)?;
    let mut stmt = conn.prepare(
        "SELECT asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source FROM dividend_event
         WHERE coalesce(pay_date, ex_date) BETWEEN ?1 AND ?2 ORDER BY coalesce(pay_date, ex_date)",
    )?;
    let rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        i64,
        String,
    )> = stmt
        .query_map(params![args.from, args.to], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let today = crate::time::today_local();
    let mut out = vec![];
    for (asset_id, kind, ex, pay, aps, cur, est, source) in rows {
        let qdate = ex
            .as_deref()
            .map(parse_date)
            .transpose()?
            .map(|d| entitled_on(d, &source).min(today))
            .unwrap_or(today);
        let book = load_book(conn, &asset_id, None)?;
        let qty = replay(conn, &book, qdate)?.quantity();
        if args.only_mine.unwrap_or(true) && qty.is_zero() {
            continue;
        }
        let kind: IncomeKind =
            serde_json::from_value(serde_json::Value::String(kind)).unwrap_or(IncomeKind::Other);
        let per = dec(&aps, "nucleo-carteira-campo-valor-cota")?;
        let us = book.asset.class.is_us();
        let withholding = match kind {
            IncomeKind::Jcp => {
                let ex = ex.as_deref().and_then(|d| parse_date(d).ok());
                let pay = pay.as_deref().and_then(|d| parse_date(d).ok());
                let on = super::records::credit_date(ex, pay.or(ex).unwrap_or(today));
                Decimal::ONE - super::records::jcp_rate(on)
            }
            IncomeKind::Dividend if us => Decimal::new(70, 2),
            _ => Decimal::ONE,
        };
        out.push(DividendCalendarItem {
            asset: brief_of(&book.asset),
            kind,
            ex_date: ex,
            pay_date: pay,
            amount_per_share: per.normalize().to_string(),
            currency: cur.clone(),
            quantity: (!qty.is_zero()).then(|| qty.normalize().to_string()),
            expected_net: (!qty.is_zero()).then(|| to_minor(qty * per * withholding, &cur)),
            estimated: est == 1,
            source,
        });
    }
    Ok(out)
}

/// Dia cuja posição dá direito ao provento. Para a B3 guardamos a data-com; o Yahoo, a Tiingo
/// e a EODHD informam a data-ex (primeiro pregão sem direito), então vale a posição do dia anterior.
pub(crate) fn entitled_on(ex: NaiveDate, source: &str) -> NaiveDate {
    if matches!(source, "yahoo" | "tiingo" | "eodhd") {
        ex.pred_opt().expect("data válida")
    } else {
        ex
    }
}

/// Sem cotação há mais de [`STALE_QUOTE_DAYS`] dias e a última sem volume: o ativo pode ter
/// sido deslistado ou incorporado.
const STALE_QUOTE_DAYS: i64 = 60;

pub(crate) fn stale_quote_warning(
    conn: &Connection,
    asset_id: &str,
    today: NaiveDate,
) -> Result<Option<String>> {
    let last: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT date, volume FROM price WHERE asset_id = ?1 ORDER BY date DESC LIMIT 1",
            [asset_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((date, volume)) = last else {
        return Ok(None);
    };
    let d = parse_date(&date)?;
    let no_volume = volume
        .as_deref()
        .is_none_or(|v| v.trim().parse::<f64>().is_ok_and(|x| x == 0.0));
    Ok(
        ((today - d).num_days() > STALE_QUOTE_DAYS && no_volume).then(|| {
            t_args(
                "nucleo-carteira-aviso-cotacao-parada",
                &i18n::args(&[("data", i18n::date(i18n::current(), d).into())]),
            )
        }),
    )
}

/// Proventos anunciados já pagos (ou com data-ex passada) que ninguém lançou, por conta: the
/// ledger's missing entries ([`super::income_ledger`]). With the automatic recording on, the
/// official ones are recorded by the collection and only the others wait here.
pub fn pending_income(conn: &Connection, args: &PendingIncomeArgs) -> Result<Vec<PendingIncome>> {
    let today = crate::time::today_local();
    let since = match &args.since {
        Some(s) => parse_date(s)?,
        None => add_months(today, -12),
    };
    let ledger = super::income_ledger::build_on(
        conn,
        &super::income_ledger::LedgerArgs {
            account_id: args.account_id.clone(),
            from: Some(fmt_date(since)),
            ..Default::default()
        },
        today,
    )?;
    let mut stale: std::collections::HashMap<String, Option<String>> = Default::default();
    let mut out = vec![];
    for e in ledger.entries.into_iter().rev() {
        if e.state != super::income_ledger::EntryState::Missing || (ledger.auto && e.official) {
            continue;
        }
        let (Some(ex), Some(gross), Some(tax), Some(qty), Some(per), Some(check)) = (
            e.ex_date.clone(),
            e.expected_gross,
            e.expected_tax,
            e.quantity.clone(),
            e.amount_per_share.clone(),
            e.check.clone(),
        ) else {
            continue;
        };
        let asset_warning = match stale.get(&e.asset.id) {
            Some(w) => w.clone(),
            None => {
                let w = stale_quote_warning(conn, &e.asset.id, today)?;
                stale.insert(e.asset.id.clone(), w.clone());
                w
            }
        };
        let pay_date = e.pay_date.clone().unwrap_or_else(|| ex.clone());
        let event_id = e.key.split('|').next().unwrap_or(&e.key).to_string();
        out.push(PendingIncome {
            dividend_event_id: event_id,
            account_id: e.account_id.clone(),
            kind: e.kind,
            ex_date: ex.clone(),
            pay_date: pay_date.clone(),
            pay_date_guessed: e.pay_date_estimated,
            quantity: qty.clone(),
            amount_per_share: per,
            currency: e.currency.clone(),
            gross,
            tax_withheld: tax,
            net: gross - tax,
            source: e.source.clone().unwrap_or_default(),
            check,
            asset_warning,
            input: IncomeInput {
                asset_id: e.asset.id.clone(),
                account_id: e.account_id.clone(),
                kind: e.kind,
                ex_date: Some(ex),
                pay_date,
                quantity: Some(qty),
                gross,
                tax_withheld: Some(tax),
                fx_rate: None,
                status: None,
                cash_movement: None,
            },
            asset: e.asset,
        });
    }
    Ok(out)
}

pub fn passive_income(conn: &Connection, args: &PassiveIncomeArgs) -> Result<PassiveIncome> {
    let today = crate::time::today_local();
    let base = crate::household::base_currency(conn)?;
    let start = add_months(today.with_day(1).expect("dia 1"), -11);
    let mut by_month: BTreeMap<String, i64> = BTreeMap::new();
    for i in 0..12 {
        by_month.insert(crate::time::month_of(add_months(start, i)), 0);
    }
    let mut by_asset: HashMap<String, i64> = HashMap::new();
    for ev in
        super::records::list_income(conn, None, Some(&fmt_date(start)), Some(&fmt_date(today)))?
    {
        if ev.status != IncomeStatus::Received {
            continue;
        }
        let d = parse_date(&ev.pay_date)?;
        let fx = match ev.fx_rate.as_deref() {
            Some(f) => dec(f, "nucleo-carteira-campo-cambio")?,
            None => fx_to_base(conn, &ev.currency, d)?
                .map(|(r, _)| r)
                .unwrap_or(Decimal::ZERO),
        };
        let v = to_minor(from_minor(ev.net, &ev.currency) * fx, &base);
        *by_month.entry(crate::time::month_of(d)).or_default() += v;
        *by_asset.entry(ev.asset_id.clone()).or_default() += v;
    }
    let s = summary(
        conn,
        &PortfolioArgs {
            as_of: None,
            account_ids: None,
            deflator: Some(Deflator::None),
            by_account: None,
            include_closed: None,
        },
    )?;
    let last_12m: i64 = by_month.values().sum();
    let yield_12m = (s.market_value > 0).then(|| last_12m as f64 / s.market_value as f64);
    let mut assets: Vec<PassiveIncomeAsset> = by_asset
        .into_iter()
        .map(|(id, v)| {
            let mv = s
                .positions
                .iter()
                .find(|p| p.asset.id == id)
                .and_then(|p| p.market_value_base);
            Ok(PassiveIncomeAsset {
                asset: super::records::asset_brief(conn, &id)?,
                last_12m: v,
                yield_12m: mv.filter(|m| *m > 0).map(|m| v as f64 / m as f64),
            })
        })
        .collect::<Result<_>>()?;
    assets.sort_by_key(|a| std::cmp::Reverse(a.last_12m));
    Ok(PassiveIncome {
        last_12m,
        monthly_avg: last_12m / 12,
        by_month: by_month
            .into_iter()
            .map(|(month, amount)| crate::cards::MonthAmount { month, amount })
            .collect(),
        by_asset: assets,
        target_monthly: args.target_monthly,
        capital_needed: match (args.target_monthly, yield_12m) {
            (Some(t), Some(y)) if y > 0.0 => Some((t as f64 * 12.0 / y).round() as i64),
            _ => None,
        },
        portfolio_yield_12m: yield_12m,
    })
}
