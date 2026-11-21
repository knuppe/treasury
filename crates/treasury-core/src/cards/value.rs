//! "Este cartão vale a pena?" (docs/design/cards.md, fase 6): por cartão, nos últimos 12
//! meses, o que ele custou (anuidade, juros e encargos, IOF) e o que devolveu (cashback recebido
//! e pontos creditados × valor do milheiro, se a família definiu). E uma simulação: quanto cada
//! cartão com programa devolveria se recebesse as compras de todos os cartões da família.
//!
//! É uma análise dos dados da família, não uma recomendação de produto financeiro.

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::ids::Id;
use crate::time::{add_months, fmt_date};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardValue {
    pub account_id: Id,
    pub account_name: String,
    pub currency: String,
    /// Compras (menos estornos), sem tarifas, encargos e créditos.
    #[ts(type = "number")]
    pub spend: i64,
    #[ts(type = "number")]
    pub annual_fee: i64,
    /// Juros, encargos, multa e mora (rotativo, atraso, parcelamento da fatura).
    #[ts(type = "number")]
    pub charges: i64,
    /// IOF lançado no cartão (compras no exterior e financiamento).
    #[ts(type = "number")]
    pub iof: i64,
    #[ts(type = "number")]
    pub cashback: i64,
    #[ts(type = "number")]
    pub points: i64,
    /// Pontos × valor do milheiro do programa (`null` sem valor definido).
    #[ts(type = "number | null")]
    pub points_value: Option<i64>,
    /// Retorno menos custos (positivo = o cartão devolveu mais do que custou).
    #[ts(type = "number")]
    pub net: i64,
}

/// Se todas as compras da família fossem para este cartão.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardScenario {
    pub account_id: Id,
    pub account_name: String,
    /// Cashback (centavos) ou pontos × valor do milheiro; `null` se for pontos sem valor.
    #[ts(type = "number | null")]
    pub reward: Option<i64>,
    #[ts(type = "number | null")]
    pub points: Option<i64>,
    /// Anuidade do ano com essa concentração (isenção por gasto considerada).
    #[ts(type = "number")]
    pub annual_fee: i64,
    #[ts(type = "number | null")]
    pub net: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardValueReport {
    pub from: String,
    pub to: String,
    pub currency: String,
    pub cards: Vec<CardValue>,
    /// Compras de todos os cartões na moeda base (a base da simulação).
    #[ts(type = "number")]
    pub total_spend: i64,
    pub scenarios: Vec<CardScenario>,
}

/// Tipo de linha do cartão para a conta.
enum Line {
    Fee,
    Iof,
    Charge,
    Cashback,
    Purchase,
}

fn line_kind(desc: &str, amount: i64, cashback_cat: bool) -> Line {
    let d = crate::transactions::normalize_description(desc);
    if d.contains("anuidade") {
        Line::Fee
    } else if d.split_whitespace().any(|w| w == "iof") {
        Line::Iof
    } else if super::revolving::is_charge(desc) {
        Line::Charge
    } else if amount > 0 && (cashback_cat || d.contains("cashback")) {
        Line::Cashback
    } else {
        Line::Purchase
    }
}

/// Relatório de todos os cartões (não arquivados) nos 12 meses até `today`.
pub fn report(conn: &Connection, today: NaiveDate) -> Result<CardValueReport> {
    let base = crate::household::base_currency(conn)?;
    let from = fmt_date(add_months(today, -12));
    let to = fmt_date(today);
    let cards: Vec<(Id, String, String)> = conn
        .prepare(
            "SELECT a.id, a.name, a.currency FROM account a JOIN credit_card c ON c.account_id = a.id
              WHERE a.archived = 0 ORDER BY a.sort_order, a.name",
        )?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let cashback_cat: Option<String> = conn
        .query_row(
            "SELECT id FROM category WHERE seed_key = ?1",
            [crate::categories::seed_key(
                Some("Reembolsos"),
                "Cashback e recompensas",
            )],
            |r| r.get(0),
        )
        .ok();
    let mut out = vec![];
    // Compras por categoria de todos os cartões (moeda base), para a simulação.
    let mut by_category: HashMap<Option<String>, i64> = HashMap::new();
    let mut parents: HashMap<String, Option<String>> = HashMap::new();
    for (id, name, currency) in &cards {
        let mut v = CardValue {
            account_id: id.clone(),
            account_name: name.clone(),
            currency: currency.clone(),
            spend: 0,
            annual_fee: 0,
            charges: 0,
            iof: 0,
            cashback: 0,
            points: 0,
            points_value: None,
            net: 0,
        };
        let mut stmt = conn.prepare_cached(
            "SELECT t.description, t.amount, t.kind, t.category_id, c.parent_id FROM txn t
               LEFT JOIN category c ON c.id = t.category_id
              WHERE t.account_id = ?1 AND t.date > ?2 AND t.date <= ?3 AND t.kind <> 'card_payment'
                AND t.status <> 'planned'",
        )?;
        let mut q = stmt.query(rusqlite::params![id, from, to])?;
        while let Some(r) = q.next()? {
            let (desc, amount, kind, cat, parent): (
                String,
                i64,
                String,
                Option<String>,
                Option<String>,
            ) = (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?);
            let is_cb_cat = cashback_cat.is_some() && cat == cashback_cat;
            match line_kind(&desc, amount, is_cb_cat) {
                Line::Fee => v.annual_fee -= amount,
                Line::Iof => v.iof -= amount,
                Line::Charge => v.charges -= amount,
                Line::Cashback => v.cashback += amount,
                Line::Purchase if kind == "expense" => {
                    v.spend -= amount;
                    let in_base = if *currency == base {
                        Some(-amount)
                    } else {
                        crate::fx::convert(conn, -amount, currency, &base, &to)?.map(|c| c.amount)
                    };
                    if let Some(a) = in_base {
                        *by_category.entry(cat.clone()).or_default() += a;
                    }
                    if let Some(c) = &cat {
                        parents.insert(c.clone(), parent);
                    }
                }
                Line::Purchase => {}
            }
        }
        // Cashback que caiu em outra conta (destino do programa).
        if let Some(p) = super::rewards::active(conn, id, &to)?
            && let Some(dest) = &p.destination_account_id
        {
            let mut stmt = conn.prepare_cached(
                "SELECT description, amount, category_id FROM txn
                  WHERE account_id = ?1 AND date > ?2 AND date <= ?3 AND amount > 0",
            )?;
            let mut q = stmt.query(rusqlite::params![dest, from, to])?;
            while let Some(r) = q.next()? {
                let (desc, amount, cat): (String, i64, Option<String>) =
                    (r.get(0)?, r.get(1)?, r.get(2)?);
                let is_cb = (cashback_cat.is_some() && cat == cashback_cat)
                    || crate::transactions::normalize_description(&desc).contains("cashback");
                if is_cb {
                    v.cashback += amount;
                }
            }
        }
        // Pontos creditados das faturas deste cartão.
        if let Some(p) = super::rewards::active(conn, id, &to)?
            && let Some(program) = &p.loyalty_program_id
        {
            let pts: i64 = conn.query_row(
                "SELECT COALESCE(SUM(e.points), 0) FROM loyalty_entry e JOIN card_statement s ON s.id = e.statement_id
                  WHERE e.program_id = ?1 AND s.account_id = ?2 AND e.date > ?3 AND e.date <= ?4
                    AND e.kind IN ('earn', 'bonus')",
                rusqlite::params![program, id, from, to],
                |r| r.get(0),
            )?;
            v.points = pts;
            let value: Option<i64> = conn.query_row(
                "SELECT value_per_1000 FROM loyalty_program WHERE id = ?1",
                [program],
                |r| r.get(0),
            )?;
            v.points_value = value.map(|per| pts * per / 1000);
        }
        v.net = v.cashback + v.points_value.unwrap_or(0) - v.annual_fee - v.charges - v.iof;
        out.push(v);
    }
    let total_spend: i64 = by_category.values().sum();
    let mut scenarios = vec![];
    for (id, name, _) in &cards {
        let Some(p) = super::rewards::active(conn, id, &to)? else {
            continue;
        };
        let base_rate = super::rewards::parse_rate(&p.base_rate).unwrap_or(Decimal::ZERO);
        let rates: HashMap<String, Decimal> = p
            .multipliers
            .iter()
            .filter_map(|m| {
                super::rewards::parse_rate(&m.rate)
                    .ok()
                    .map(|r| (m.category_id.clone(), r))
            })
            .collect();
        let mut raw = Decimal::ZERO;
        for (cat, amount) in &by_category {
            let parent = cat.as_ref().and_then(|c| parents.get(c).cloned().flatten());
            let r = cat
                .as_ref()
                .and_then(|c| rates.get(c))
                .or_else(|| parent.as_ref().and_then(|c| rates.get(c)))
                .copied()
                .unwrap_or(base_rate);
            raw += Decimal::from(*amount) * r;
        }
        // Teto por fatura vira teto anual aproximado (12 faturas).
        let (reward, points) = match p.kind {
            super::rewards::RewardKind::Cashback => {
                let mut c = raw.round().to_i64().unwrap_or(0);
                if let Some(cap) = p.cycle_cap {
                    c = c.min(cap * 12);
                }
                (Some(c), None)
            }
            super::rewards::RewardKind::Points => {
                let mut pts = (raw / Decimal::from(100)).floor().to_i64().unwrap_or(0);
                if let Some(cap) = p.cycle_cap {
                    pts = pts.min(cap * 12);
                }
                let value: Option<i64> = match &p.loyalty_program_id {
                    Some(l) => conn.query_row(
                        "SELECT value_per_1000 FROM loyalty_program WHERE id = ?1",
                        [l],
                        |r| r.get(0),
                    )?,
                    None => None,
                };
                (value.map(|per| pts * per / 1000), Some(pts))
            }
        };
        // Anuidade com toda a família no cartão: isenta por gasto se a média mensal chega lá.
        let fee = match super::annual_fee::active(conn, id, &to)? {
            Some(f) => {
                use super::annual_fee::WaiverKind as W;
                match f.waiver_kind {
                    W::Full => 0,
                    W::Spend if total_spend / 12 >= f.waiver_threshold.unwrap_or(i64::MAX) => 0,
                    W::Discount => {
                        let d =
                            super::rewards::parse_rate(f.waiver_discount.as_deref().unwrap_or("0"))
                                .unwrap_or(Decimal::ZERO);
                        (Decimal::from(f.amount) * (Decimal::ONE - d))
                            .round()
                            .to_i64()
                            .unwrap_or(f.amount)
                    }
                    _ => f.amount,
                }
            }
            None => 0,
        };
        scenarios.push(CardScenario {
            account_id: id.clone(),
            account_name: name.clone(),
            reward,
            points,
            annual_fee: fee,
            net: reward.map(|r| r - fee),
        });
    }
    scenarios.sort_by(|a, b| b.net.unwrap_or(i64::MIN).cmp(&a.net.unwrap_or(i64::MIN)));
    Ok(CardValueReport {
        from,
        to,
        currency: base,
        cards: out,
        total_spend,
        scenarios,
    })
}
