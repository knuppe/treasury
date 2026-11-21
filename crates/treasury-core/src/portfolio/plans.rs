//! Aportes programados (docs/scheduled-contributions.md): "comprar R$ 500 do ativo X todo dia 10".
//!
//! O app nunca grava a compra sozinho. Na data, o aporte aparece em "a confirmar" com o preço
//! do fechamento do dia (ou o último anterior) e a quantidade que o valor compra; a pessoa
//! confirma (vira uma operação comum, com a saída no caixa da corretora) ou pula. Os aportes já
//! decididos ficam em `investment_plan_run` e não voltam.

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::records::{self, from_minor};
use super::{TradeInput, TradeSide};
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{add_months, fmt_date, now_utc, parse_date};

/// Até quantos meses para trás os aportes não decididos aparecem.
pub const LOOKBACK_MONTHS: i32 = 12;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InvestmentPlan {
    pub id: Id,
    pub name: String,
    pub asset_id: Id,
    pub account_id: Id,
    /// Valor de cada aporte, na moeda do ativo (menor unidade).
    #[ts(type = "number")]
    pub amount: i64,
    /// Dia do mês (1–28).
    pub day_of_month: u32,
    pub start_date: String,
    pub end_date: Option<String>,
    /// Meta para a qual o aporte vai (informativo).
    pub goal_id: Option<Id>,
    pub active: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InvestmentPlanView {
    #[serde(flatten)]
    pub plan: InvestmentPlan,
    pub asset_name: String,
    pub ticker: Option<String>,
    pub account_name: String,
    pub currency: String,
    pub goal_name: Option<String>,
    /// Próxima data do aporte depois de hoje (`None`: terminou ou está pausado).
    pub next_due: Option<String>,
    /// Aportes vencidos ainda sem decisão.
    pub pending: u32,
    /// Aportes confirmados (operações gravadas).
    pub confirmed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InvestmentPlanInput {
    /// Sem `id`, cria; com ele, altera.
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<Id>,
    pub name: String,
    pub asset_id: Id,
    pub account_id: Id,
    #[ts(type = "number")]
    pub amount: i64,
    pub day_of_month: u32,
    pub start_date: String,
    #[serde(default)]
    #[ts(optional)]
    pub end_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub goal_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub active: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub notes: Option<String>,
}

/// Um aporte vencido esperando a decisão, com a compra sugerida.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlanDue {
    pub plan_id: Id,
    pub plan_name: String,
    pub asset_id: Id,
    pub asset_name: String,
    pub ticker: Option<String>,
    pub account_id: Id,
    pub account_name: String,
    pub due_date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    /// Fechamento do dia (ou o último anterior); `None` sem cotação.
    pub price: Option<String>,
    pub price_date: Option<String>,
    /// Quantidade que o valor compra ao preço (inteira em ações, FIIs, ETFs e BDRs da B3).
    pub quantity: Option<String>,
    pub goal_id: Option<Id>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct PlanConfirmArgs {
    pub plan_id: Id,
    pub due_date: String,
    pub quantity: String,
    pub price: String,
    /// Data da operação (padrão: a do aporte).
    #[serde(default)]
    #[ts(optional)]
    pub date: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub fees: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct PlanSkipArgs {
    pub plan_id: Id,
    pub due_date: String,
}

const COLUMNS: &str = "id, name, asset_id, account_id, amount, day_of_month, start_date, end_date,
    goal_id, active, notes";

fn row_to_plan(r: &rusqlite::Row<'_>) -> rusqlite::Result<InvestmentPlan> {
    Ok(InvestmentPlan {
        id: r.get(0)?,
        name: r.get(1)?,
        asset_id: r.get(2)?,
        account_id: r.get(3)?,
        amount: r.get(4)?,
        day_of_month: r.get(5)?,
        start_date: r.get(6)?,
        end_date: r.get(7)?,
        goal_id: r.get(8)?,
        active: r.get::<_, i64>(9)? != 0,
        notes: r.get(10)?,
    })
}

fn not_found(id: &str) -> Error {
    Error::not_found(t_args(
        "nucleo-carteira-plano",
        &args(&[("id", id.to_string().into())]),
    ))
}

pub fn load(conn: &Connection, id: &str) -> Result<InvestmentPlan> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM investment_plan WHERE id = ?1"),
        [id],
        row_to_plan,
    )
    .optional()?
    .ok_or_else(|| not_found(id))
}

/// As datas do plano de `from` a `to` (inclusive), no dia escolhido de cada mês.
fn due_dates(p: &InvestmentPlan, from: NaiveDate, to: NaiveDate) -> Result<Vec<NaiveDate>> {
    let start = parse_date(&p.start_date)?;
    let end = p.end_date.as_deref().map(parse_date).transpose()?;
    let mut out = vec![];
    let mut month = NaiveDate::from_ymd_opt(start.year(), start.month(), 1).expect("dia 1");
    while month <= to {
        let d = month.with_day(p.day_of_month).expect("dia 1–28");
        if d >= start && d >= from && d <= to && end.is_none_or(|e| d <= e) {
            out.push(d);
        }
        if end.is_some_and(|e| d > e) {
            break;
        }
        month = add_months(month, 1);
    }
    Ok(out)
}

fn decided(conn: &Connection, plan_id: &str) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT due_date, status FROM investment_plan_run WHERE plan_id = ?1 ORDER BY due_date",
    )?;
    let rows = stmt
        .query_map([plan_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

fn names(conn: &Connection, p: &InvestmentPlan) -> Result<(crate::assets::Asset, String)> {
    let asset = crate::assets::get(conn, &p.asset_id)?;
    let account: String = conn.query_row(
        "SELECT name FROM account WHERE id = ?1",
        [&p.account_id],
        |r| r.get(0),
    )?;
    Ok((asset, account))
}

fn pending_dates(
    conn: &Connection,
    p: &InvestmentPlan,
    today: NaiveDate,
) -> Result<Vec<NaiveDate>> {
    if !p.active {
        return Ok(vec![]);
    }
    let done = decided(conn, &p.id)?;
    Ok(due_dates(p, add_months(today, -LOOKBACK_MONTHS), today)?
        .into_iter()
        .filter(|d| !done.iter().any(|(x, _)| *x == fmt_date(*d)))
        .collect())
}

pub fn list(conn: &Connection, today: NaiveDate) -> Result<Vec<InvestmentPlanView>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM investment_plan ORDER BY active DESC, created_at, id"
    ))?;
    let plans = stmt
        .query_map([], row_to_plan)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    plans.into_iter().map(|p| view(conn, p, today)).collect()
}

fn view(conn: &Connection, p: InvestmentPlan, today: NaiveDate) -> Result<InvestmentPlanView> {
    let (asset, account_name) = names(conn, &p)?;
    let goal_name: Option<String> = match &p.goal_id {
        Some(g) => conn
            .query_row("SELECT name FROM goal WHERE id = ?1", [g], |r| r.get(0))
            .optional()?,
        None => None,
    };
    let next_due = if p.active {
        due_dates(&p, today + chrono::Duration::days(1), add_months(today, 2))?
            .first()
            .map(|d| fmt_date(*d))
    } else {
        None
    };
    let pending = pending_dates(conn, &p, today)?.len() as u32;
    let confirmed = decided(conn, &p.id)?
        .iter()
        .filter(|(_, s)| s == "confirmed")
        .count() as u32;
    Ok(InvestmentPlanView {
        asset_name: asset.name,
        ticker: asset.ticker,
        account_name,
        currency: asset.currency,
        goal_name,
        next_due,
        pending,
        confirmed,
        plan: p,
    })
}

/// Ações, FIIs, ETFs e BDRs da B3 só compram cotas inteiras.
fn whole_units(class: AssetClass) -> bool {
    matches!(
        class,
        AssetClass::StockBr | AssetClass::Fii | AssetClass::EtfBr | AssetClass::Bdr
    )
}

/// Quantidade que `amount` compra a `price` (para baixo: nunca passa do valor).
pub(crate) fn quantity_for(amount: Decimal, price: Decimal, class: AssetClass) -> Option<Decimal> {
    if price <= Decimal::ZERO {
        return None;
    }
    let q = amount / price;
    let q = if whole_units(class) {
        q.floor()
    } else if class == AssetClass::TreasuryBr {
        q.round_dp_with_strategy(2, RoundingStrategy::ToZero)
    } else {
        q.round_dp_with_strategy(6, RoundingStrategy::ToZero)
    };
    (q > Decimal::ZERO).then(|| q.normalize())
}

/// Aportes vencidos até `today` (nos últimos 12 meses) que ainda esperam a decisão.
pub fn pending(conn: &Connection, today: NaiveDate) -> Result<Vec<PlanDue>> {
    let mut out = vec![];
    for v in list(conn, today)? {
        let p = &v.plan;
        let asset = crate::assets::get(conn, &p.asset_id)?;
        for d in pending_dates(conn, p, today)? {
            let price = crate::market::close_on(conn, &p.asset_id, d)?;
            let amount = from_minor(p.amount, &asset.currency);
            out.push(PlanDue {
                plan_id: p.id.clone(),
                plan_name: p.name.clone(),
                asset_id: p.asset_id.clone(),
                asset_name: v.asset_name.clone(),
                ticker: v.ticker.clone(),
                account_id: p.account_id.clone(),
                account_name: v.account_name.clone(),
                due_date: fmt_date(d),
                amount: p.amount,
                currency: asset.currency.clone(),
                quantity: price
                    .as_ref()
                    .and_then(|(c, _)| quantity_for(amount, *c, asset.class))
                    .map(|q| q.to_string()),
                price: price.as_ref().map(|(_, on)| on.close.clone()),
                price_date: price.map(|(_, on)| on.date),
                goal_id: p.goal_id.clone(),
            });
        }
    }
    out.sort_by(|a, b| {
        a.due_date
            .cmp(&b.due_date)
            .then(a.plan_name.cmp(&b.plan_name))
    });
    Ok(out)
}

fn validate(conn: &Connection, i: &InvestmentPlanInput) -> Result<()> {
    if i.name.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-carteira-plano-erro-nome")));
    }
    if i.amount <= 0 {
        return Err(Error::invalid(t("nucleo-carteira-plano-erro-valor")));
    }
    if !(1..=28).contains(&i.day_of_month) {
        return Err(Error::invalid(t("nucleo-carteira-plano-erro-dia")));
    }
    let start = parse_date(&i.start_date)?;
    if let Some(e) = &i.end_date
        && parse_date(e)? < start
    {
        return Err(Error::invalid(t("nucleo-carteira-plano-erro-fim")));
    }
    crate::assets::get(conn, &i.asset_id)?;
    crate::accounts::currency_and_kind(conn, &i.account_id)?;
    if let Some(g) = &i.goal_id {
        crate::goals::load(conn, g)?;
    }
    Ok(())
}

fn clean(s: &Option<String>) -> Option<String> {
    s.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Cria ou altera um plano (com `id`).
pub fn save(tx: &Transaction<'_>, i: &InvestmentPlanInput) -> Result<Id> {
    validate(tx, i)?;
    let now = now_utc();
    let active = i.active.unwrap_or(true) as i64;
    match &i.id {
        None => {
            let id = new_id();
            tx.execute(
                "INSERT INTO investment_plan (id, name, asset_id, account_id, amount, day_of_month,
                     start_date, end_date, goal_id, active, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
                params![
                    id,
                    i.name.trim(),
                    i.asset_id,
                    i.account_id,
                    i.amount,
                    i.day_of_month,
                    i.start_date,
                    i.end_date,
                    i.goal_id,
                    active,
                    clean(&i.notes),
                    now
                ],
            )?;
            Ok(id)
        }
        Some(id) => {
            load(tx, id)?;
            tx.execute(
                "UPDATE investment_plan SET name = ?2, asset_id = ?3, account_id = ?4, amount = ?5,
                     day_of_month = ?6, start_date = ?7, end_date = ?8, goal_id = ?9, active = ?10,
                     notes = ?11, updated_at = ?12 WHERE id = ?1",
                params![
                    id,
                    i.name.trim(),
                    i.asset_id,
                    i.account_id,
                    i.amount,
                    i.day_of_month,
                    i.start_date,
                    i.end_date,
                    i.goal_id,
                    active,
                    clean(&i.notes),
                    now
                ],
            )?;
            Ok(id.clone())
        }
    }
}

/// Apaga o plano; as operações já confirmadas ficam.
pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    tx.execute("DELETE FROM investment_plan_run WHERE plan_id = ?1", [id])?;
    if tx.execute("DELETE FROM investment_plan WHERE id = ?1", [id])? == 0 {
        return Err(not_found(id));
    }
    Ok(())
}

/// A data é um aporte do plano ainda sem decisão.
fn check_due(tx: &Transaction<'_>, p: &InvestmentPlan, due: &str) -> Result<NaiveDate> {
    let d = parse_date(due)?;
    let is_due = due_dates(p, d, d)?.contains(&d);
    let done: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM investment_plan_run WHERE plan_id = ?1 AND due_date = ?2)",
        params![p.id, due],
        |r| r.get(0),
    )?;
    if !is_due || done {
        return Err(Error::invalid(t("nucleo-carteira-plano-erro-data")));
    }
    Ok(d)
}

/// Confirma o aporte: grava a compra (com a saída no caixa da corretora) e marca a data.
pub fn confirm(tx: &Transaction<'_>, a: &PlanConfirmArgs) -> Result<Id> {
    let p = load(tx, &a.plan_id)?;
    check_due(tx, &p, &a.due_date)?;
    let trade = records::create_trade(
        tx,
        &TradeInput {
            asset_id: p.asset_id.clone(),
            account_id: p.account_id.clone(),
            date: a.date.clone().unwrap_or_else(|| a.due_date.clone()),
            side: TradeSide::Buy,
            quantity: a.quantity.clone(),
            price: a.price.clone(),
            fees: a.fees,
            taxes: None,
            fx_rate: None,
            day_trade: None,
            note_number: None,
            notes: Some(t_args(
                "nucleo-carteira-plano-nota",
                &args(&[("plano", p.name.clone().into())]),
            )),
            cash_movement: None,
            external_id: None,
        },
    )?;
    tx.execute(
        "INSERT INTO investment_plan_run (plan_id, due_date, status, trade_id, created_at)
         VALUES (?1, ?2, 'confirmed', ?3, ?4)",
        params![p.id, a.due_date, trade, now_utc()],
    )?;
    Ok(trade)
}

/// Pula o aporte da data (não volta a aparecer).
pub fn skip(tx: &Transaction<'_>, a: &PlanSkipArgs) -> Result<()> {
    let p = load(tx, &a.plan_id)?;
    check_due(tx, &p, &a.due_date)?;
    tx.execute(
        "INSERT INTO investment_plan_run (plan_id, due_date, status, created_at)
         VALUES (?1, ?2, 'skipped', ?3)",
        params![p.id, a.due_date, now_utc()],
    )?;
    Ok(())
}

/// Plano pelo id, com os nomes.
pub fn get(conn: &Connection, id: &str, today: NaiveDate) -> Result<InvestmentPlanView> {
    view(conn, load(conn, id)?, today)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_shares_on_b3_and_fractions_elsewhere() {
        let d = |s: &str| Decimal::from_str(s).unwrap();
        assert_eq!(
            quantity_for(d("500"), d("32.10"), AssetClass::StockBr),
            Some(d("15"))
        );
        assert_eq!(
            quantity_for(d("500"), d("15432.10"), AssetClass::TreasuryBr),
            Some(d("0.03"))
        );
        assert_eq!(
            quantity_for(d("100"), d("3.0"), AssetClass::Fund),
            Some(d("33.333333"))
        );
        assert_eq!(quantity_for(d("10"), d("32.10"), AssetClass::StockBr), None);
        assert_eq!(quantity_for(d("10"), d("0"), AssetClass::Fund), None);
    }
}
