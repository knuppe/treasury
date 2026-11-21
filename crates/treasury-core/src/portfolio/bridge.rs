//! Ponte do resultado (`portfolio.bridge`): do valor da carteira no início do período ao valor
//! no fim, em dinheiro, linha a linha. Responde "de onde veio a diferença" sem percentuais:
//!
//! ```text
//! valor inicial
//! + ganho de preço   (cotações e curva da renda fixa; em outra moeda, preço e câmbio)
//! + proventos        (brutos, com amortizações e restituições de capital)
//! − taxas            (corretagem e emolumentos)
//! − impostos         (IR retido nas vendas e nos proventos, come-cotas)
//! + aportes          (compras com custos)
//! − retiradas        (vendas líquidas e proventos recebidos)
//! = valor final
//! ```
//!
//! O "resultado" (`delta`) é a variação do valor menos aportes e mais retiradas: quanto os
//! investimentos renderam em dinheiro. A perda para a inflação corrige o valor inicial e cada
//! aporte e retirada pelo deflator até o fim do período.
//!
//! Nos ativos em outra moeda, o ganho de preço na moeda do ativo vai pelo câmbio do fim do
//! período; o resto do ganho em reais (o valor inicial e cada compra e venda pela variação do
//! câmbio desde a data deles) é o câmbio. Herança por mudança de ticker ou incorporação entre
//! dois ativos da mesma ponte se anula na soma; com o filtro de um ativo só, vira preço.

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{
    Book, brief_of, fund_quota_on, fx_to_base, performance_books, replay, value_state,
};
use super::records::to_minor;
use super::{AssetBrief, CorporateActionKind, PerformanceArgs, TradeSide};
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::indices::{self, Deflator};
use crate::time::{fmt_date, month_bounds, parse_month};

/// As linhas da ponte, na moeda base (centavos).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BridgeLines {
    #[ts(type = "number")]
    pub start_value: i64,
    /// Ganho de preço total (inclui o câmbio).
    #[ts(type = "number")]
    pub price_gain: i64,
    /// Parte do ganho de preço que veio do câmbio (ativos em outra moeda).
    #[ts(type = "number")]
    pub fx_gain: i64,
    /// Proventos brutos (antes do IR retido).
    #[ts(type = "number")]
    pub income: i64,
    #[ts(type = "number")]
    pub fees: i64,
    #[ts(type = "number")]
    pub taxes: i64,
    #[ts(type = "number")]
    pub contributions: i64,
    #[ts(type = "number")]
    pub withdrawals: i64,
    #[ts(type = "number")]
    pub end_value: i64,
    /// Resultado em dinheiro: fim − início − aportes + retiradas.
    #[ts(type = "number")]
    pub delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BridgeAsset {
    pub asset: AssetBrief,
    pub lines: BridgeLines,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerformanceBridge {
    /// Primeiro e último dia do período.
    pub from: String,
    pub to: String,
    pub currency: String,
    pub lines: BridgeLines,
    pub deflator: Deflator,
    /// O que a inflação levou no período (valor inicial e cada aporte e retirada corrigidos
    /// até o fim); `null` sem o índice.
    #[ts(type = "number | null")]
    pub inflation_loss: Option<i64>,
    /// Resultado depois da inflação.
    #[ts(type = "number | null")]
    pub delta_real: Option<i64>,
    /// Por ativo, do maior valor final ao menor (desempenho passado, não recomendação).
    pub assets: Vec<BridgeAsset>,
    /// Algum valor veio de cotação defasada, câmbio estimado ou índice projetado.
    pub estimated: bool,
}

#[derive(Default)]
struct Acc {
    start: Decimal,
    end: Decimal,
    /// Ganho de preço na moeda do ativo (para separar o câmbio).
    gain_local: Decimal,
    buy_gross: Decimal,
    sell_gross: Decimal,
    income: Decimal,
    fees: Decimal,
    taxes: Decimal,
    contributions: Decimal,
    withdrawals: Decimal,
    /// Saídas de valor que não passam por compra e venda (restituição de capital e
    /// come-cotas), devolvidas ao ganho de preço.
    other_out: Decimal,
    /// Fluxos datados (positivo = aporte, negativo = retirada) para a inflação.
    flows: Vec<(NaiveDate, Decimal)>,
}

fn lines(a: &Acc, fx_end: Option<Decimal>, base: &str) -> BridgeLines {
    let price = a.end - a.start - a.buy_gross + a.sell_gross + a.other_out;
    let fx = fx_end.map_or(Decimal::ZERO, |f| price - a.gain_local * f);
    let m = |v: Decimal| to_minor(v, base);
    let mut l = BridgeLines {
        start_value: m(a.start),
        price_gain: 0,
        fx_gain: m(fx),
        income: m(a.income),
        fees: m(a.fees),
        taxes: m(a.taxes),
        contributions: m(a.contributions),
        withdrawals: m(a.withdrawals),
        end_value: m(a.end),
        delta: 0,
    };
    // O ganho de preço fecha a ponte em centavos (o arredondamento de cada linha fica nele).
    l.price_gain =
        l.end_value - l.start_value - l.income + l.fees + l.taxes - l.contributions + l.withdrawals;
    l.delta = l.end_value - l.start_value - l.contributions + l.withdrawals;
    l
}

fn add(t: &mut BridgeLines, l: &BridgeLines) {
    t.start_value += l.start_value;
    t.price_gain += l.price_gain;
    t.fx_gain += l.fx_gain;
    t.income += l.income;
    t.fees += l.fees;
    t.taxes += l.taxes;
    t.contributions += l.contributions;
    t.withdrawals += l.withdrawals;
    t.end_value += l.end_value;
    t.delta += l.delta;
}

/// Ponte de um livro entre `start` (valor no fim do dia) e `end`.
fn book_bridge(
    conn: &Connection,
    b: &Book,
    start: NaiveDate,
    end: NaiveDate,
    estimated: &mut bool,
) -> Result<Acc> {
    let mut a = Acc::default();
    let fx_on = |fx: Option<Decimal>, d: NaiveDate, est: &mut bool| -> Result<Decimal> {
        if let Some(f) = fx {
            return Ok(f);
        }
        Ok(match fx_to_base(conn, &b.asset.currency, d)? {
            Some((r, e)) => {
                *est |= e;
                r
            }
            None => {
                *est = true;
                Decimal::ZERO
            }
        })
    };
    for (d, slot_base, slot_local) in [(start, true, true), (end, false, false)] {
        let st = replay(conn, b, d)?;
        let v = value_state(conn, b, &st, d)?;
        *estimated |= v.estimated || st.estimated;
        let base = v.value_base.unwrap_or(Decimal::ZERO);
        let local = v.value.unwrap_or(Decimal::ZERO);
        if slot_base {
            a.start = base;
        } else {
            a.end = base;
        }
        if slot_local {
            a.gain_local -= local;
        } else {
            a.gain_local += local;
        }
    }
    let inside = |d: NaiveDate| d > start && d <= end;
    for tr in b.trades.iter().filter(|t| inside(t.date)) {
        let fx = fx_on(tr.fx, tr.date, estimated)?;
        let gross = tr.qty * tr.price;
        let taxes = tr.costs - tr.fees;
        a.fees += tr.fees * fx;
        a.taxes += taxes * fx;
        match tr.side {
            TradeSide::Buy => {
                a.buy_gross += gross * fx;
                a.gain_local -= gross;
                let amount = (gross + tr.costs) * fx;
                a.contributions += amount;
                a.flows.push((tr.date, amount));
            }
            TradeSide::Sell => {
                a.sell_gross += gross * fx;
                a.gain_local += gross;
                let amount = (gross - tr.costs) * fx;
                a.withdrawals += amount;
                a.flows.push((tr.date, -amount));
            }
        }
    }
    for i in b.incomes.iter().filter(|i| inside(i.pay_date)) {
        let fx = fx_on(i.fx, i.pay_date, estimated)?;
        a.income += (i.net + i.tax) * fx;
        a.taxes += i.tax * fx;
        a.withdrawals += i.net * fx;
        a.flows.push((i.pay_date, -(i.net * fx)));
    }
    for ac in b.actions.iter().filter(|x| inside(x.date)) {
        if ac.kind != CorporateActionKind::CapitalReturn {
            continue;
        }
        let fx = fx_on(None, ac.date, estimated)?;
        if let Some(shares) = ac.come_cotas {
            // Come-cotas: cotas resgatadas para o IR; o valor delas é imposto.
            match fund_quota_on(conn, &b.asset.id, ac.date)? {
                Some(q) => {
                    let v = shares * q;
                    a.taxes += v * fx;
                    a.other_out += v * fx;
                    a.gain_local += v;
                }
                None => *estimated = true,
            }
        } else if let Some(v) = ac.amount {
            // Restituição de capital: dinheiro que sai do ativo como provento.
            a.income += v * fx;
            a.withdrawals += v * fx;
            a.other_out += v * fx;
            a.gain_local += v;
            a.flows.push((ac.date, -(v * fx)));
        }
    }
    Ok(a)
}

/// Período da ponte: o mesmo filtro de meses da rentabilidade (sem `from_month`, desde a
/// primeira operação; sem `to_month`, até hoje).
fn period(books: &[Book], args: &PerformanceArgs) -> Result<(NaiveDate, NaiveDate)> {
    let today = crate::time::today_local();
    let first = books
        .iter()
        .filter_map(|b| b.trades.first().map(|t| t.date))
        .min();
    let (fy, fm) = match (&args.from_month, first) {
        (Some(m), _) => parse_month(m)?,
        (None, Some(d)) => (d.year(), d.month()),
        (None, None) => (today.year(), today.month()),
    };
    let (ty, tm) = match &args.to_month {
        Some(m) => parse_month(m)?,
        None => (today.year(), today.month()),
    };
    let from = month_bounds(fy, fm).0;
    let to = month_bounds(ty, tm).1.min(today);
    if to < from {
        return Err(Error::invalid(t("nucleo-carteira-erro-ponte-periodo")));
    }
    Ok((from, to))
}

pub fn bridge(conn: &Connection, args: &PerformanceArgs) -> Result<PerformanceBridge> {
    let base = crate::household::base_currency(conn)?;
    let deflator = indices::effective_deflator(conn, args.deflator.unwrap_or_default());
    let books = performance_books(conn, args.asset_id.as_deref(), args.class)?;
    let (from, to) = period(&books, args)?;
    let start = from - chrono::Duration::days(1);
    let mut estimated = false;
    let mut total = BridgeLines::default();
    let mut assets = vec![];
    let mut flows: Vec<(NaiveDate, Decimal)> = vec![];
    for b in &books {
        let acc = book_bridge(conn, b, start, to, &mut estimated)?;
        let fx_end = if b.asset.currency == base {
            None
        } else {
            fx_to_base(conn, &b.asset.currency, to)?.map(|(r, e)| {
                estimated |= e;
                r
            })
        };
        let l = lines(&acc, fx_end, &base);
        if l == BridgeLines::default() {
            continue;
        }
        add(&mut total, &l);
        flows.push((from, acc.start));
        flows.extend(acc.flows.iter().copied());
        assets.push(BridgeAsset {
            asset: brief_of(&b.asset),
            lines: l,
        });
    }
    assets.sort_by(|x, y| {
        y.lines
            .end_value
            .cmp(&x.lines.end_value)
            .then_with(|| x.asset.name.cmp(&y.asset.name))
    });
    // Perda para a inflação: cada valor que entrou (ou saiu) corrigido até o fim.
    let mut factors: HashMap<NaiveDate, Option<Decimal>> = HashMap::new();
    let mut loss = Some(Decimal::ZERO);
    if deflator == Deflator::None {
        loss = None;
    }
    let end_next = to + chrono::Duration::days(1);
    for (d, v) in &flows {
        let Some(acc) = loss.as_mut() else { break };
        if v.is_zero() {
            continue;
        }
        let f = match factors.get(d) {
            Some(f) => *f,
            None => {
                let f = indices::deflator_factor(conn, deflator, *d, end_next)?.map(|x| {
                    estimated |= x.estimated;
                    x.factor
                });
                factors.insert(*d, f);
                f
            }
        };
        match f {
            Some(f) => *acc += *v * (f - Decimal::ONE),
            None => loss = None,
        }
    }
    let inflation_loss = loss.map(|l| to_minor(l, &base));
    Ok(PerformanceBridge {
        from: fmt_date(from),
        to: fmt_date(to),
        currency: base,
        delta_real: inflation_loss.map(|l| total.delta - l),
        lines: total,
        deflator,
        inflation_loss,
        assets,
        estimated,
    })
}
