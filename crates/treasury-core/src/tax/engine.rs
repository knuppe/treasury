//! Cálculo da apuração mensal e anual. Reproduz as operações de renda variável brasileira
//! de cada titular com preço médio (Receita Federal), separa day trade (compra e venda do
//! mesmo ativo no mesmo dia) das operações comuns e aplica isenção, compensação de
//! prejuízos por categoria, IRRF e o mínimo do DARF.

#![allow(clippy::type_complexity)]

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use super::rules::TaxRules;
use super::*;
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::finance::average_cost::{Position, PositionEvent};
use crate::finance::calendar::is_business_day;
use crate::i18n::{args, t, t_args};
use crate::portfolio::TradeSide;
use crate::portfolio::engine::{Book, load_book};
use crate::time::{add_months, fmt_date, month_bounds, parse_month};

fn cents(v: Decimal) -> i64 {
    (v * Decimal::ONE_HUNDRED)
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap_or(0)
}

fn category_of(class: AssetClass) -> Option<TaxCategory> {
    match class {
        AssetClass::StockBr => Some(TaxCategory::Stocks),
        AssetClass::EtfBr => Some(TaxCategory::Etf),
        AssetClass::Bdr => Some(TaxCategory::Bdr),
        AssetClass::Fii => Some(TaxCategory::Fii),
        _ => None,
    }
}

/// Contas do titular (pela tabela de titulares). Sem titular informado: todas as contas.
pub fn member_accounts(conn: &Connection, member_id: Option<&str>) -> Result<Option<Vec<String>>> {
    let Some(m) = member_id else { return Ok(None) };
    let mut stmt = conn.prepare("SELECT account_id FROM account_member WHERE member_id = ?1")?;
    let ids: Vec<String> = stmt
        .query_map([m], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Some(ids))
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MonthAgg {
    pub(crate) sales: BTreeMap<TaxCategory, Decimal>,
    pub(crate) result: BTreeMap<TaxCategory, Decimal>,
    pub(crate) irrf: Decimal,
    /// A parte do IRRF que é de day trade (1% do ganho), que só compensa até dezembro.
    pub(crate) irrf_day_trade: Decimal,
    pub(crate) irrf_estimated: bool,
    pub(crate) realized: Vec<RealizedItem>,
    /// Vendas maiores que a posição registrada: (ativo, data, quantidade a mais).
    pub(crate) oversold: Vec<(String, NaiveDate, Decimal)>,
}

/// Resultados realizados por mês (`YYYY-MM`) de todos os ativos brasileiros de renda variável.
pub(crate) fn realized_by_month(
    conn: &Connection,
    accounts: Option<&[String]>,
    rules: &dyn Fn(i32) -> TaxRules,
) -> Result<BTreeMap<String, MonthAgg>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT a.id FROM asset a WHERE a.class IN ('stock_br', 'etf_br', 'bdr', 'fii')
           AND (EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id)
                OR EXISTS (SELECT 1 FROM corporate_action c WHERE c.new_asset_id = a.id))",
    )?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut months: BTreeMap<String, MonthAgg> = BTreeMap::new();
    for id in ids {
        let book = load_book(conn, &id, accounts)?;
        let Some(cat) = category_of(book.asset.class) else {
            continue;
        };
        replay_for_tax(&book, cat, &mut months, rules)?;
    }
    Ok(months)
}

fn replay_for_tax(
    book: &Book,
    cat: TaxCategory,
    months: &mut BTreeMap<String, MonthAgg>,
    rules: &dyn Fn(i32) -> TaxRules,
) -> Result<()> {
    let mut pos = Position::default();
    let mut dates: Vec<NaiveDate> = book
        .trades
        .iter()
        .map(|t| t.date)
        .chain(book.actions.iter().map(|a| a.date))
        .chain(book.inherited.iter().map(|i| i.date))
        .collect();
    dates.sort();
    dates.dedup();
    for day in dates {
        for i in book.inherited.iter().filter(|i| i.date == day) {
            pos.qty += i.qty;
            pos.cost += i.cost;
        }
        for a in book.actions.iter().filter(|a| a.date == day) {
            use crate::portfolio::CorporateActionKind as K;
            if a.moved_out {
                pos = Position::default();
                continue;
            }
            match (a.kind, a.factor) {
                (K::Split | K::ReverseSplit, Some(f)) => {
                    pos.apply(&PositionEvent::Split { factor: f })?;
                }
                (K::Bonus, f) if a.quantity.is_some() || f.is_some() => {
                    let qty = a
                        .quantity
                        .unwrap_or_else(|| pos.qty * f.unwrap_or_default());
                    pos.apply(&PositionEvent::Bonus {
                        qty,
                        unit_cost: a.unit_cost.unwrap_or(Decimal::ZERO),
                    })?;
                }
                (K::CapitalReturn, _) => {
                    if let Some(v) = a.amount {
                        pos.apply(&PositionEvent::CapitalReturn { amount: v })?;
                    }
                }
                _ => {}
            }
        }
        let trades: Vec<_> = book.trades.iter().filter(|t| t.date == day).collect();
        if trades.is_empty() {
            continue;
        }
        let month = crate::time::month_of(day);
        let r = rules(day.year());
        let agg = months.entry(month).or_default();
        // Day trade: compra e venda do mesmo ativo, no mesmo dia, na mesma corretora
        // (IN RFB 1.585/2015, art. 65), com os preços médios do próprio dia.
        let mut accounts: Vec<&str> = trades.iter().map(|t| t.account_id.as_str()).collect();
        accounts.sort();
        accounts.dedup();
        let (mut rem_buy_qty, mut rem_buy_cost) = (Decimal::ZERO, Decimal::ZERO);
        let mut rem_sells: Vec<(Decimal, Decimal, Decimal, Decimal, Vec<String>)> = vec![]; // (qty, preço líquido médio, bruto/qty, irrf, ids)
        for acc in accounts {
            let group: Vec<_> = trades.iter().filter(|t| t.account_id == acc).collect();
            let buys: Vec<_> = group.iter().filter(|t| t.side == TradeSide::Buy).collect();
            let sells: Vec<_> = group.iter().filter(|t| t.side == TradeSide::Sell).collect();
            let bq: Decimal = buys.iter().map(|t| t.qty).sum();
            let sq: Decimal = sells.iter().map(|t| t.qty).sum();
            let buy_cost: Decimal = buys.iter().map(|t| t.qty * t.price + t.fees).sum();
            let sell_gross: Decimal = sells.iter().map(|t| t.qty * t.price).sum();
            let sell_net: Decimal = sells.iter().map(|t| t.qty * t.price - t.fees).sum();
            let irrf_recorded: Decimal = sells.iter().map(|t| t.irrf).sum();
            let sell_ids: Vec<String> = sells.iter().map(|t| t.id.clone()).collect();
            let dt_qty = bq.min(sq);
            if dt_qty > Decimal::ZERO {
                let avg_buy = buy_cost / bq;
                let avg_sell_net = sell_net / sq;
                let result = dt_qty * (avg_sell_net - avg_buy);
                let gross = sell_gross * dt_qty / sq;
                *agg.sales.entry(TaxCategory::DayTrade).or_default() += gross;
                *agg.result.entry(TaxCategory::DayTrade).or_default() += result;
                if irrf_recorded > Decimal::ZERO {
                    agg.irrf += irrf_recorded * dt_qty / sq;
                    agg.irrf_day_trade += irrf_recorded * dt_qty / sq;
                } else if result > Decimal::ZERO {
                    // Retenção até R$ 1,00 é dispensada.
                    let est = result * r.irrf_day_trade;
                    if est > Decimal::ONE {
                        agg.irrf += est;
                        agg.irrf_day_trade += est;
                        agg.irrf_estimated = true;
                    }
                }
                agg.realized.push(RealizedItem {
                    date: fmt_date(day),
                    asset_id: book.asset.id.clone(),
                    ticker: book.asset.ticker.clone(),
                    category: TaxCategory::DayTrade,
                    quantity: dt_qty.normalize().to_string(),
                    sale_value: cents(gross),
                    proceeds: cents(avg_sell_net * dt_qty),
                    cost: cents(avg_buy * dt_qty),
                    result: cents(result),
                    trade_ids: sell_ids.clone(),
                });
            }
            if bq > dt_qty {
                rem_buy_qty += bq - dt_qty;
                rem_buy_cost += (bq - dt_qty) * buy_cost / bq;
            }
            if sq > dt_qty {
                let q = sq - dt_qty;
                let irrf = if irrf_recorded > Decimal::ZERO {
                    irrf_recorded * q / sq
                } else {
                    Decimal::NEGATIVE_ONE
                };
                rem_sells.push((q, sell_net / sq, sell_gross / sq, irrf, sell_ids));
            }
        }
        // Compras que sobram entram na posição; vendas que sobram são operações comuns
        // contra o preço médio (único por titular, somando todas as corretoras).
        if rem_buy_qty > Decimal::ZERO {
            pos.apply(&PositionEvent::Buy {
                qty: rem_buy_qty,
                price: rem_buy_cost / rem_buy_qty,
                fees: Decimal::ZERO,
            })?;
        }
        for (q, net_unit, gross_unit, irrf, ids) in rem_sells {
            if q > pos.qty {
                // Falta histórico de compras (ou é venda a descoberto): o excesso não tem
                // custo e fica fora da apuração; o mês avisa.
                let label = book.asset.ticker.clone().unwrap_or(book.asset.name.clone());
                let extra = q - pos.qty.max(Decimal::ZERO);
                agg.oversold.push((label, day, extra));
                // The sale happened all the same: its value counts in what was sold in the
                // month (the base of the stock exemption). Only the result is unknown.
                *agg.sales.entry(cat).or_default() += extra * gross_unit;
            }
            let qty = q.min(pos.qty);
            if qty <= Decimal::ZERO {
                continue;
            }
            let avg_cost = pos.cost / pos.qty;
            let proceeds = qty * net_unit;
            let cost = qty * avg_cost;
            pos.apply(&PositionEvent::Sell {
                qty,
                price: net_unit,
                fees: Decimal::ZERO,
            })?;
            let gross = qty * gross_unit;
            *agg.sales.entry(cat).or_default() += gross;
            *agg.result.entry(cat).or_default() += proceeds - cost;
            if irrf >= Decimal::ZERO {
                agg.irrf += irrf * qty / q;
            } else {
                // Retenção até R$ 1,00 é dispensada.
                let est = gross * r.irrf_swing;
                if est > Decimal::ONE {
                    agg.irrf += est;
                    agg.irrf_estimated = true;
                }
            }
            agg.realized.push(RealizedItem {
                date: fmt_date(day),
                asset_id: book.asset.id.clone(),
                ticker: book.asset.ticker.clone(),
                category: cat,
                quantity: qty.normalize().to_string(),
                sale_value: cents(gross),
                proceeds: cents(proceeds),
                cost: cents(cost),
                result: cents(proceeds - cost),
                trade_ids: ids,
            });
        }
    }
    Ok(())
}

/// Ajuste com o lugar da custódia de cripto escolhido por conta (`true` = exterior).
pub const CRYPTO_ABROAD_SETTING: &str = "tax.crypto_abroad_accounts";

/// Onde a cripto de cada conta está custodiada: a escolha do usuário, senão pela moeda da
/// conta (em reais = Brasil, ganho de capital mensal; outra moeda = exterior, Lei 14.754/2023).
pub fn crypto_custody(conn: &Connection) -> Result<Vec<CryptoCustody>> {
    let chosen: BTreeMap<String, bool> =
        crate::settings::get_or(conn, CRYPTO_ABROAD_SETTING, BTreeMap::new())?;
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.currency,
                EXISTS (SELECT 1 FROM trade t JOIN asset s ON s.id = t.asset_id
                         WHERE t.account_id = a.id AND s.class = 'crypto')
           FROM account a ORDER BY a.name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, bool>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .map(|(id, name, currency, has_crypto)| {
            let user = chosen.get(&id).copied();
            CryptoCustody {
                abroad: user.unwrap_or(currency != "BRL"),
                chosen: user.is_some(),
                account_id: id,
                name,
                currency,
                has_crypto,
            }
        })
        .collect())
}

/// Grava (ou, com `None`, apaga) a escolha de uma conta.
pub fn set_crypto_custody(conn: &Connection, account_id: &str, abroad: Option<bool>) -> Result<()> {
    let mut map: BTreeMap<String, bool> =
        crate::settings::get_or(conn, CRYPTO_ABROAD_SETTING, BTreeMap::new())?;
    match abroad {
        Some(v) => {
            map.insert(account_id.to_string(), v);
        }
        None => {
            map.remove(account_id);
        }
    }
    crate::settings::set(conn, CRYPTO_ABROAD_SETTING, &map)
}

/// Contas cuja cripto conta como custodiada no Brasil.
fn brl_accounts(conn: &Connection) -> Result<Vec<String>> {
    Ok(crypto_custody(conn)?
        .into_iter()
        .filter(|c| !c.abroad)
        .map(|c| c.account_id)
        .collect())
}

#[derive(Debug, Clone, Default)]
struct CryptoAgg {
    /// Vendido no mês, no Brasil e no exterior (BRL, unidade maior).
    sales: Decimal,
    sales_brazil: Decimal,
    gain: Decimal,
    loss: Decimal,
    oversold: Vec<(String, NaiveDate, Decimal)>,
}

/// Vendas de criptoativos por mês: o total (Brasil e exterior, que decide a isenção) e o
/// ganho ou a perda de cada venda em conta em reais, pelo custo médio em reais.
fn crypto_by_month(
    conn: &Connection,
    accounts: Option<&[String]>,
) -> Result<BTreeMap<String, CryptoAgg>> {
    let brl = brl_accounts(conn)?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT a.id FROM asset a WHERE a.class = 'crypto'
           AND EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id)",
    )?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out: BTreeMap<String, CryptoAgg> = BTreeMap::new();
    for id in ids {
        let book = load_book(conn, &id, accounts)?;
        let cur = book.asset.currency.clone();
        let mut trades = book.trades.clone();
        // No mesmo dia, compras antes das vendas.
        trades.sort_by_key(|t| (t.date, t.side == TradeSide::Sell));
        let (mut qty, mut cost) = (Decimal::ZERO, Decimal::ZERO);
        for t in &trades {
            let fx = if cur == "BRL" {
                Decimal::ONE
            } else if let Some(f) = t.fx {
                f
            } else {
                crate::fx::rate(conn, &cur, "BRL", &fmt_date(t.date))?
                    .map(|(r, ..)| r)
                    .unwrap_or(Decimal::ZERO)
            };
            let in_brazil = brl.contains(&t.account_id);
            match t.side {
                TradeSide::Buy if in_brazil => {
                    qty += t.qty;
                    cost += (t.qty * t.price + t.costs) * fx;
                }
                TradeSide::Buy => {}
                TradeSide::Sell => {
                    let gross = t.qty * t.price * fx;
                    let agg = out.entry(crate::time::month_of(t.date)).or_default();
                    agg.sales += gross;
                    if !in_brazil {
                        continue;
                    }
                    agg.sales_brazil += gross;
                    if t.qty > qty {
                        let label = book.asset.ticker.clone().unwrap_or(book.asset.name.clone());
                        agg.oversold.push((label, t.date, t.qty - qty));
                    }
                    let sold = t.qty.min(qty);
                    let avg = if qty.is_zero() {
                        Decimal::ZERO
                    } else {
                        cost / qty
                    };
                    let result = gross - t.costs * fx - avg * sold;
                    if result >= Decimal::ZERO {
                        agg.gain += result;
                    } else {
                        agg.loss -= result;
                    }
                    cost -= avg * sold;
                    qty -= sold;
                }
            }
        }
    }
    Ok(out)
}

/// Imposto sobre o ganho de capital pelas faixas (cada alíquota sobre a parcela do ganho).
fn capital_gain_tax(gain: Decimal, r: &TaxRules) -> Decimal {
    let mut prev = Decimal::ZERO;
    let mut tax = Decimal::ZERO;
    for (up_to, rate) in r.capital_gain_bands {
        let top = up_to.map(|u| Decimal::from(u) / Decimal::ONE_HUNDRED);
        let slice = match top {
            Some(top) => gain.min(top) - prev,
            None => gain - prev,
        };
        if slice <= Decimal::ZERO {
            break;
        }
        tax += slice * rate;
        prev = top.unwrap_or(gain);
    }
    tax.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Último dia útil do mês seguinte ao de apuração.
pub fn darf_due_date(month: &str) -> Result<NaiveDate> {
    let (y, m) = parse_month(month)?;
    let next = add_months(NaiveDate::from_ymd_opt(y, m, 1).expect("data"), 1);
    let (_, mut last) = month_bounds(next.year(), next.month());
    while !is_business_day(last) {
        last = last.pred_opt().expect("data");
    }
    Ok(last)
}

struct Pools {
    common: Decimal,
    day_trade: Decimal,
    fii: Decimal,
    /// IRRF "dedo-duro" (Lei 11.033/2004) ainda não compensado no ano.
    irrf_carry: Decimal,
    /// IRRF de day trade ainda não compensado no ano.
    irrf_dt_carry: Decimal,
    darf_carry: Decimal,
    irrf_year: i32,
}

fn pool_month(pool: LossPool, result: Decimal, loss: &mut Decimal, rate: Decimal) -> PoolMonth {
    let before = *loss;
    let (base, used) = if result <= Decimal::ZERO {
        *loss += -result;
        (Decimal::ZERO, Decimal::ZERO)
    } else {
        let used = result.min(*loss);
        *loss -= used;
        (result - used, used)
    };
    let tax = (base * rate).round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    PoolMonth {
        pool,
        result: cents(result),
        loss_before: cents(before),
        loss_used: cents(used),
        base: cents(base),
        rate: rate.normalize().to_string(),
        tax: cents(tax),
        loss_after: cents(*loss),
    }
}

/// Apuração de todos os meses desde o primeiro com vendas até `last_month` (inclusive).
pub fn compute_months(
    conn: &Connection,
    member_id: Option<&str>,
    last_month: &str,
) -> Result<Vec<TaxMonth>> {
    let accounts = member_accounts(conn, member_id)?;
    let agg = realized_by_month(conn, accounts.as_deref(), &TaxRules::for_year)?;
    let crypto = crypto_by_month(conn, accounts.as_deref())?;
    let Some(first) = agg.keys().chain(crypto.keys()).min().cloned() else {
        return Ok(vec![]);
    };
    let mut crypto_carry = 0i64;
    let end = last_month.to_string();
    let mut out = vec![];
    let (fy, fm) = parse_month(&first)?;
    let mut cur = NaiveDate::from_ymd_opt(fy, fm, 1).expect("data");
    let (ey, em) = parse_month(&end)?;
    let end_date = NaiveDate::from_ymd_opt(ey, em, 1).expect("data");
    let mut pools = Pools {
        common: Decimal::ZERO,
        day_trade: Decimal::ZERO,
        fii: Decimal::ZERO,
        irrf_carry: Decimal::ZERO,
        irrf_dt_carry: Decimal::ZERO,
        darf_carry: Decimal::ZERO,
        irrf_year: fy,
    };
    while cur <= end_date {
        let month = crate::time::month_of(cur);
        let r = TaxRules::for_year(cur.year());
        if cur.year() != pools.irrf_year {
            // IRRF só compensa dentro do mesmo ano-calendário.
            // O saldo do "dedo-duro" vai para a declaração de ajuste (Lei 11.033/2004 art. 2º
            // §7º, III); o de day trade só pode ser pedido em restituição (Perguntas e
            // Respostas IRPF 2026, perguntas 714 e 715). Os dois aparecem no relatório anual.
            pools.irrf_carry = Decimal::ZERO;
            pools.irrf_dt_carry = Decimal::ZERO;
            pools.irrf_year = cur.year();
        }
        let a = agg.get(&month).cloned().unwrap_or_default();
        let get =
            |m: &BTreeMap<TaxCategory, Decimal>, c| m.get(&c).copied().unwrap_or(Decimal::ZERO);
        let stock_sales = get(&a.sales, TaxCategory::Stocks);
        let stock_result = get(&a.result, TaxCategory::Stocks);
        let exempt = cents(stock_sales) <= r.stock_exemption_limit;
        let exempt_gain = if exempt && stock_result > Decimal::ZERO {
            stock_result
        } else {
            Decimal::ZERO
        };
        let common_result = (stock_result - exempt_gain)
            + get(&a.result, TaxCategory::Etf)
            + get(&a.result, TaxCategory::Bdr);
        let pc = pool_month(
            LossPool::Common,
            common_result,
            &mut pools.common,
            r.swing_rate,
        );
        let pd = pool_month(
            LossPool::DayTrade,
            get(&a.result, TaxCategory::DayTrade),
            &mut pools.day_trade,
            r.day_trade_rate,
        );
        let pf = pool_month(
            LossPool::Fii,
            get(&a.result, TaxCategory::Fii),
            &mut pools.fii,
            r.fii_rate,
        );
        let tax = pc.tax + pd.tax + pf.tax;
        let irrf = cents(a.irrf);
        let irrf_dt = cents(a.irrf_day_trade).min(irrf);
        // O de day trade (que vence em dezembro) é usado primeiro.
        let dt_available = irrf_dt + cents(pools.irrf_dt_carry);
        let swing_available = (irrf - irrf_dt) + cents(pools.irrf_carry);
        let dt_used = dt_available.min(tax);
        let swing_used = swing_available.min(tax - dt_used);
        let irrf_used = dt_used + swing_used;
        let irrf_carried_in = (cents(pools.irrf_carry) + cents(pools.irrf_dt_carry)).min(tax);
        pools.irrf_dt_carry = Decimal::from(dt_available - dt_used) / Decimal::ONE_HUNDRED;
        pools.irrf_carry = Decimal::from(swing_available - swing_used) / Decimal::ONE_HUNDRED;
        let due = tax - irrf_used;
        let carried_in = cents(pools.darf_carry);
        let total = due + carried_in;
        let (darf_amount, carried_out) = if total >= r.darf_minimum {
            (total, 0)
        } else {
            (0, total)
        };
        pools.darf_carry = Decimal::from(carried_out) / Decimal::ONE_HUNDRED;
        let has_sales = !a.sales.is_empty();
        let mut categories = vec![];
        for c in [
            TaxCategory::Stocks,
            TaxCategory::Etf,
            TaxCategory::Bdr,
            TaxCategory::Fii,
            TaxCategory::DayTrade,
        ] {
            if a.sales.contains_key(&c) || a.result.contains_key(&c) {
                categories.push(CategoryMonth {
                    category: c,
                    label: c.label(),
                    sales: cents(get(&a.sales, c)),
                    result: cents(get(&a.result, c)),
                    exempt_gain: if c == TaxCategory::Stocks {
                        cents(exempt_gain)
                    } else {
                        0
                    },
                });
            }
        }
        let darf: Option<(String, String)> = conn
            .query_row(
                "SELECT id, status FROM darf WHERE coalesce(member_id, '') = coalesce(?1, '') AND period = ?2
                   AND revenue_code = ?3 AND status <> 'cancelled'",
                params![member_id, month, r.darf_code],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let status = match (&darf, darf_amount, has_sales, carried_out) {
            (Some((_, s)), _, _, _) if s == "paid" => MonthStatus::Paid,
            (_, d, _, _) if d > 0 => MonthStatus::ToPay,
            (_, _, _, c) if c > 0 => MonthStatus::BelowMinimum,
            (_, _, true, _) => MonthStatus::NoTax,
            _ => MonthStatus::NoActivity,
        };
        let crypto_month = match crypto.get(&month) {
            None => None,
            Some(c) => {
                let exempt = cents(c.sales) <= r.crypto_exemption_limit;
                let tax = if exempt {
                    0
                } else {
                    cents(capital_gain_tax(c.gain, &r))
                };
                let total = tax + crypto_carry;
                let (darf_amount, carried_out) = if total >= r.darf_minimum {
                    (total, 0)
                } else {
                    (0, total)
                };
                let carried_in = crypto_carry;
                crypto_carry = carried_out;
                let darf: Option<(String, String)> = conn
                    .query_row(
                        "SELECT id, status FROM darf WHERE coalesce(member_id, '') = coalesce(?1, '') AND period = ?2
                           AND revenue_code = ?3 AND status <> 'cancelled'",
                        params![member_id, month, r.capital_gain_darf_code],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()?;
                Some(CryptoMonth {
                    paid: darf.as_ref().is_some_and(|(_, s)| s == "paid"),
                    darf_id: darf.map(|(id, _)| id),
                    sales: cents(c.sales),
                    sales_brazil: cents(c.sales_brazil),
                    gain: cents(c.gain),
                    loss: cents(c.loss),
                    exempt,
                    tax,
                    carried_in,
                    darf_amount,
                    carried_out,
                    revenue_code: r.capital_gain_darf_code.into(),
                    due_date: fmt_date(darf_due_date(&month)?),
                })
            }
        };
        let mut notes = vec![];
        if has_sales && exempt && stock_sales > Decimal::ZERO {
            notes.push(t_args(
                "nucleo-impostos-ir-nota-isencao",
                &args(&[(
                    "valor",
                    crate::i18n::money(crate::i18n::current(), cents(stock_sales), "BRL").into(),
                )]),
            ));
        }
        if a.irrf_estimated {
            notes.push(t("nucleo-impostos-ir-nota-irrf-estimado"));
        }
        let crypto_oversold = crypto
            .get(&month)
            .map(|c| c.oversold.as_slice())
            .unwrap_or(&[]);
        let incomplete = !a.oversold.is_empty() || !crypto_oversold.is_empty();
        for (asset, day, extra) in a.oversold.iter().chain(crypto_oversold) {
            notes.push(t_args(
                "nucleo-impostos-ir-nota-venda-maior",
                &args(&[
                    ("ativo", asset.clone().into()),
                    (
                        "data",
                        crate::i18n::date(crate::i18n::current(), *day).into(),
                    ),
                    ("quantidade", extra.normalize().to_string().into()),
                ]),
            ));
        }
        out.push(TaxMonth {
            month: month.clone(),
            member_id: member_id.map(String::from),
            categories,
            pools: vec![pc, pd, pf],
            stock_sales: cents(stock_sales),
            stock_exempt: exempt,
            tax,
            irrf,
            irrf_estimated: a.irrf_estimated,
            irrf_carried_in,
            irrf_carried_out: cents(pools.irrf_carry) + cents(pools.irrf_dt_carry),
            irrf_day_trade_carried_out: cents(pools.irrf_dt_carry),
            due,
            carried_in,
            darf_amount,
            carried_out,
            due_date: fmt_date(darf_due_date(&month)?),
            revenue_code: r.darf_code.into(),
            status,
            darf_id: darf.map(|(id, _)| id),
            realized: a.realized,
            notes,
            crypto: crypto_month,
            incomplete,
        });
        cur = add_months(cur, 1);
    }
    Ok(out)
}

pub fn month(conn: &Connection, member_id: Option<&str>, month: &str) -> Result<TaxMonth> {
    parse_month(month)?;
    let months = compute_months(conn, member_id, month)?;
    Ok(months
        .into_iter()
        .find(|m| m.month == month)
        .unwrap_or_else(|| TaxMonth {
            month: month.into(),
            member_id: member_id.map(String::from),
            categories: vec![],
            pools: vec![],
            stock_sales: 0,
            stock_exempt: true,
            tax: 0,
            irrf: 0,
            irrf_estimated: false,
            irrf_carried_in: 0,
            irrf_carried_out: 0,
            irrf_day_trade_carried_out: 0,
            due: 0,
            carried_in: 0,
            darf_amount: 0,
            carried_out: 0,
            due_date: darf_due_date(month).map(fmt_date).unwrap_or_default(),
            revenue_code: TaxRules::for_year(2026).darf_code.into(),
            status: MonthStatus::NoActivity,
            darf_id: None,
            realized: vec![],
            notes: vec![],
            crypto: None,
            incomplete: false,
        }))
}

/// Nome do país (Bens e Direitos) pelo código ISO das bolsas do catálogo.
fn country_name(iso: &str) -> &'static str {
    match iso.to_uppercase().as_str() {
        "BR" => "Brasil",
        "US" => "Estados Unidos",
        "CA" => "Canadá",
        "MX" => "México",
        "AR" => "Argentina",
        "CL" => "Chile",
        "GB" => "Reino Unido",
        "DE" => "Alemanha",
        "FR" => "França",
        "NL" => "Países Baixos",
        "BE" => "Bélgica",
        "PT" => "Portugal",
        "IE" => "Irlanda",
        "IT" => "Itália",
        "ES" => "Espanha",
        "CH" => "Suíça",
        "AT" => "Áustria",
        "SE" => "Suécia",
        "DK" => "Dinamarca",
        "FI" => "Finlândia",
        "NO" => "Noruega",
        "PL" => "Polônia",
        "TR" => "Turquia",
        "ZA" => "África do Sul",
        "IL" => "Israel",
        "SA" => "Arábia Saudita",
        "IN" => "Índia",
        "CN" => "China",
        "HK" => "Hong Kong",
        "JP" => "Japão",
        "KR" => "Coreia do Sul",
        "TW" => "Taiwan",
        "SG" => "Singapura",
        "ID" => "Indonésia",
        "TH" => "Tailândia",
        "AU" => "Austrália",
        "NZ" => "Nova Zelândia",
        _ => "Exterior (confira o país)",
    }
}

/// Stablecoins da tabela do programa IRPF 2026 (código 08.03); o resto, fora o Bitcoin, é
/// altcoin (08.02).
const STABLECOINS: [&str; 13] = [
    "USDT", "USDC", "BRZ", "BUSD", "DAI", "TUSD", "GUSD", "USDP", "PAX", "PAXG", "FDUSD", "PYUSD",
    "USDE",
];

/// Grupo/código sugerido em Bens e Direitos (confira no programa da Receita).
///
/// Códigos da tabela de Bens e Direitos do programa IRPF 2026 (grupos 03, 04, 07, 08 e 99) e
/// do guia da Receita citado em docs/brazil/income-tax-validation.md (E6): poupança 04.01; títulos tributados
/// (Tesouro, CDB, RDB) 04.02; isentos (LCI, LCA, CRI, CRA, debênture incentivada) 04.03; BDR
/// 04.04; fundos com come-cotas 07.01; fundos de ações 07.04; FII 07.03; ETF (entidade de
/// investimento) 07.06, de renda fixa 07.08; Bitcoin 08.01, altcoins 08.02, stablecoins 08.03.
fn bens_code(conn: &Connection, a: &crate::assets::Asset) -> Result<(&'static str, &'static str)> {
    use crate::assets::FixedIncomeKind as K;
    Ok(match a.class {
        AssetClass::StockBr => ("03.01", "Ações"),
        AssetClass::Fii => ("07.03", "Fundos de Investimento Imobiliário (FII)"),
        AssetClass::EtfBr => (
            "07.06",
            "ETF (entidade de investimento; ETF de renda fixa vai em 07.08)",
        ),
        AssetClass::Bdr => ("04.04", "BDR"),
        AssetClass::FixedIncome => match a.fixed_income.as_ref().map(|f| f.kind) {
            Some(K::Poupanca) => ("04.01", "Depósito em conta poupança"),
            Some(K::Lci | K::Lca | K::Cri | K::Cra | K::DebentureIncentivada) => (
                "04.03",
                "Títulos isentos de tributação (LCI, LCA, CRI, CRA, debêntures incentivadas)",
            ),
            _ => ("04.02", "Títulos públicos e privados sujeitos à tributação"),
        },
        AssetClass::TreasuryBr => ("04.02", "Títulos públicos (Tesouro Direto)"),
        AssetClass::Fund => {
            let rule = crate::portfolio::come_cotas::rule(conn, a)?;
            if rule.regime == crate::portfolio::come_cotas::Regime::Exempt {
                ("07.04", "Fundos de Investimento em Ações")
            } else {
                (
                    "07.01",
                    "Fundos de investimento sujeitos ao come-cotas (Lei 14.754/2023)",
                )
            }
        }
        AssetClass::StockUs
        | AssetClass::EtfUs
        | AssetClass::Reit
        | AssetClass::StockIntl
        | AssetClass::EtfIntl => ("03.01", "Ações/ETFs no exterior"),
        AssetClass::Crypto => {
            let ticker = a.ticker.as_deref().unwrap_or_default().to_uppercase();
            let base = ticker.split(['-', '/']).next().unwrap_or_default();
            if base == "BTC" {
                ("08.01", "Criptoativo Bitcoin (BTC)")
            } else if STABLECOINS.contains(&base) {
                ("08.03", "Criptoativos stablecoins")
            } else {
                ("08.02", "Outras criptomoedas (altcoins)")
            }
        }
        // VGBL: grupo 99, código 06, pelo valor aplicado (o PGBL não vai em Bens e Direitos).
        AssetClass::Pension => ("99.06", "VGBL – Vida Gerador de Benefício Livre"),
        AssetClass::Other => ("99.99", "Outros bens e direitos"),
    })
}

/// Países com acordo para evitar a dupla tributação em vigor (Receita, página "Acordos para
/// evitar a dupla tributação", atualizada em 28/08/2026; o da Alemanha está sem efeito desde
/// 2006) e os de reciprocidade reconhecida (Alemanha, Estados Unidos e Reino Unido: Perguntas e
/// Respostas IRPF 2026, perguntas 136 e 142). Só o imposto pago nesses países vira crédito
/// (Lei 14.754/2023 art. 4º). O da Polônia (Decreto 12.865/2026) fica de fora até valer.
const CREDIT_COUNTRIES: [&str; 40] = [
    "ZA", "AR", "AT", "BE", "CA", "CL", "CN", "KR", "DK", "AE", "EC", "SK", "ES", "PH", "FI", "FR",
    "HU", "IN", "IL", "IT", "JP", "LU", "MX", "NO", "NL", "PE", "PT", "CZ", "RU", "SG", "SE", "CH",
    "TT", "TR", "UA", "UY", "VE", "DE", "US", "GB",
];

/// O imposto retido lá fora sobre os proventos deste ativo pode virar crédito: país da bolsa
/// (ou do ativo) com acordo ou reciprocidade.
fn credit_allowed(a: &crate::assets::Asset) -> bool {
    let country = if a.class.is_us() {
        Some("US".to_string())
    } else {
        a.exchange
            .as_ref()
            .map(|e| e.country.clone())
            .or_else(|| a.country.clone())
    };
    country.is_some_and(|c| CREDIT_COUNTRIES.contains(&c.to_uppercase().as_str()))
}

/// Retenção de 10% sobre dividendos de uma mesma empresa acima de R$ 50 mil no mês, a partir
/// de 2026 (Lei 9.250/1995 art. 6º-A, incluído pela Lei 15.270/2025: "10% ... sobre o total do
/// valor pago"; Receita, Perguntas e Respostas "Tributação de Altas Rendas", 19/12/2025). Não
/// vale para lucros apurados até 2025 com distribuição aprovada até 31/12/2025 (§3º): o
/// Treasury não sabe a origem do lucro, então lista o mês para conferir.
pub const DIVIDEND_WITHHOLDING_FROM: i32 = 2026;
const DIVIDEND_MONTHLY_LIMIT: i64 = 5_000_000;

fn dividends_over_limit(
    conn: &Connection,
    accounts: Option<&[String]>,
    year: i32,
) -> Result<Vec<DividendWithholding>> {
    if year < DIVIDEND_WITHHOLDING_FROM {
        return Ok(vec![]);
    }
    let mut stmt = conn.prepare(
        "SELECT i.account_id, a.ticker, a.cnpj, substr(i.pay_date, 1, 7), i.gross, i.tax_withheld
           FROM income_event i JOIN asset a ON a.id = i.asset_id
          WHERE i.kind = 'dividend' AND i.status = 'received' AND i.currency = 'BRL'
            AND a.class IN ('stock_br', 'bdr') AND substr(i.pay_date, 1, 4) = ?1",
    )?;
    let rows = stmt
        .query_map([year.to_string()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut by: BTreeMap<(String, String), (Vec<String>, i64, i64)> = BTreeMap::new();
    for (acc, ticker, cnpj, month, gross, withheld) in rows {
        if accounts.is_some_and(|a| !a.iter().any(|x| x == &acc)) {
            continue;
        }
        let ticker = ticker.unwrap_or_default();
        let company = cnpj
            .map(|c| crate::research::cnpj_digits(&c))
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| ticker.chars().take(4).collect());
        let e = by.entry((month, company)).or_default();
        if !e.0.contains(&ticker) {
            e.0.push(ticker);
        }
        e.1 += gross;
        e.2 += withheld;
    }
    Ok(by
        .into_iter()
        .filter(|(_, (_, amount, _))| *amount > DIVIDEND_MONTHLY_LIMIT)
        .map(
            |((month, _), (tickers, amount, withheld))| DividendWithholding {
                month,
                tickers,
                amount,
                expected_withholding: amount / 10,
                withheld,
            },
        )
        .collect())
}

/// Primeiro ano da Lei 14.754/2023 (aplicações financeiras no exterior).
const FOREIGN_LAW_FROM: i32 = 2024;

/// Ganho (ou perda, negativo) realizado no ano em aplicações no exterior: ativos listados fora
/// do Brasil e criptoativos em contas que não são em reais.
fn foreign_realized_in(conn: &Connection, accounts: Option<&[String]>, year: i32) -> Result<i64> {
    let end = NaiveDate::from_ymd_opt(year, 12, 31).expect("data");
    let prev = NaiveDate::from_ymd_opt(year - 1, 12, 31).expect("data");
    let brl = brl_accounts(conn)?;
    let abroad: Vec<String> = {
        let mut stmt = conn.prepare("SELECT id FROM account")?;
        let all: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        all.into_iter()
            .filter(|a| !brl.contains(a) && accounts.is_none_or(|m| m.contains(a)))
            .collect()
    };
    let mut total = 0;
    for id in traded_assets(conn)? {
        let book = load_book(conn, &id, accounts)?;
        if !is_foreign_investment(&book.asset) {
            continue;
        }
        let book = if book.asset.class == AssetClass::Crypto {
            load_book(conn, &id, Some(&abroad))?
        } else {
            book
        };
        let st = crate::portfolio::engine::replay(conn, &book, end)?;
        let sp = crate::portfolio::engine::replay(conn, &book, prev)?;
        total += cents(st.realized_base() - sp.realized_base());
    }
    Ok(total)
}

/// Prejuízo no exterior ainda não compensado ao começar `year` (Lei 14.754/2023 art. 9º §2º),
/// somando os anos desde 2024.
fn foreign_loss_before(conn: &Connection, accounts: Option<&[String]>, year: i32) -> Result<i64> {
    let mut loss = 0;
    for y in FOREIGN_LAW_FROM..year {
        let realized = foreign_realized_in(conn, accounts, y)?;
        loss = (loss - realized).max(0);
    }
    Ok(loss)
}

/// Ativos que podem entrar no resumo do exterior (Lei 14.754/2023): os listados fora do Brasil
/// e os criptoativos (só os de contas fora do real; ver [`foreign_realized_in`]).
fn is_foreign_investment(a: &crate::assets::Asset) -> bool {
    a.class.is_foreign_listed() || a.class == AssetClass::Crypto
}

/// Ativos com operações (e os que nasceram de eventos societários).
fn traded_assets(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT asset_id FROM trade")?;
    let mut ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT new_asset_id FROM corporate_action WHERE new_asset_id IS NOT NULL",
    )?;
    for id in stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?
    {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Ok(ids)
}

pub fn year(conn: &Connection, member_id: Option<&str>, year: i32) -> Result<TaxYear> {
    if !(2000..=2100).contains(&year) {
        return Err(Error::invalid(t("nucleo-impostos-ir-erro-ano")));
    }
    let all = compute_months(conn, member_id, &crate::time::fmt_month(year, 12))?;
    let months: Vec<TaxMonth> = all
        .iter()
        .filter(|m| m.month.starts_with(&format!("{year}-")))
        .cloned()
        .collect();
    let last = all.last();
    let pool = |p: LossPool| {
        last.and_then(|m| m.pools.iter().find(|x| x.pool == p))
            .map(|x| x.loss_after)
            .unwrap_or(0)
    };
    let accounts = member_accounts(conn, member_id)?;
    let (from, to) = (format!("{year}-01-01"), format!("{year}-12-31"));
    // Proventos recebidos no ano por tipo.
    let mut stmt = conn.prepare(
        "SELECT i.asset_id, a.ticker, a.name, a.class, i.kind, i.net, i.currency, i.fx_rate, i.pay_date, i.account_id,
                i.gross, i.tax_withheld
         FROM income_event i JOIN asset a ON a.id = i.asset_id
         WHERE i.status = 'received' AND i.pay_date BETWEEN ?1 AND ?2",
    )?;
    let rows: Vec<(
        String,
        Option<String>,
        String,
        String,
        String,
        i64,
        String,
        Option<String>,
        String,
        String,
        i64,
        i64,
    )> = stmt
        .query_map(params![from, to], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
                r.get(10)?,
                r.get(11)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let r = TaxRules::for_year(year);
    let mut div: BTreeMap<String, IncomeLine> = BTreeMap::new();
    let mut fii: BTreeMap<String, IncomeLine> = BTreeMap::new();
    let mut jcp: BTreeMap<String, IncomeLine> = BTreeMap::new();
    let (mut foreign_income, mut foreign_tax_paid, mut foreign_credit) = (0i64, 0i64, 0i64);
    let mut foreign_no_credit = 0i64;
    for (aid, ticker, name, class, kind, net, cur, fx, pay, acc, gross, withheld) in rows {
        if accounts.as_ref().is_some_and(|a| !a.contains(&acc)) {
            continue;
        }
        // Cotação da moeda em reais na data do pagamento (a da operação, se informada).
        let rate = if cur == "BRL" {
            Some(Decimal::ONE)
        } else {
            match fx.and_then(|f| f.parse::<Decimal>().ok()) {
                Some(r) => Some(r),
                None => crate::fx::rate(conn, &cur, "BRL", &pay)?.map(|(r, ..)| r),
            }
        };
        let to_brl = |v: i64| {
            rate.map(|r| cents(Decimal::from(v) / Decimal::ONE_HUNDRED * r))
                .unwrap_or(0)
        };
        let net_brl = to_brl(net);
        let class = AssetClass::parse(&class)?;
        if class.is_foreign_listed() {
            // Lei 14.754/2023 art. 2º §1º: 15% sobre o rendimento, sem dedução (o bruto); o
            // imposto pago lá fora vira crédito (art. 4º), limitado ao IR brasileiro daquele
            // rendimento (§1º), convertido na data do pagamento (§2º).
            let (gross_brl, tax_brl) = (to_brl(gross), to_brl(withheld));
            foreign_income += gross_brl;
            foreign_tax_paid += tax_brl;
            let asset = crate::assets::get(conn, &aid)?;
            if credit_allowed(&asset) {
                foreign_credit += tax_brl.min(cents(
                    Decimal::from(gross_brl) / Decimal::ONE_HUNDRED * r.foreign_rate,
                ));
            } else {
                foreign_no_credit += tax_brl;
            }
            continue;
        }
        let target = match (kind.as_str(), class) {
            ("jcp", _) => &mut jcp,
            ("fii_income", _) | (_, AssetClass::Fii) => &mut fii,
            _ => &mut div,
        };
        let e = target.entry(aid.clone()).or_insert(IncomeLine {
            asset_id: aid,
            ticker,
            name,
            amount: 0,
        });
        e.amount += net_brl;
    }
    // Bens e direitos: posições em 31/12 do ano e do ano anterior, pelo custo.
    let mut holdings = vec![];
    let ids_all = traded_assets(conn)?;
    let end = NaiveDate::from_ymd_opt(year, 12, 31).expect("data");
    let prev = NaiveDate::from_ymd_opt(year - 1, 12, 31).expect("data");
    for id in ids_all {
        let book = load_book(conn, &id, accounts.as_deref())?;
        let st = crate::portfolio::engine::replay(conn, &book, end)?;
        let sp = crate::portfolio::engine::replay(conn, &book, prev)?;
        let a = &book.asset;
        if st.quantity().is_zero() && sp.quantity().is_zero() {
            continue;
        }
        let (mut code, mut label) = bens_code(conn, a)?;
        let plan = if a.class == AssetClass::Pension {
            crate::pension::get_plan(conn, &a.id)?.map(|p| p.plan_type)
        } else {
            None
        };
        if plan == Some(crate::pension::PlanType::Pgbl) {
            code = "—";
            label = "PGBL (não vai em Bens e Direitos: os aportes do ano entram em Pagamentos Efetuados, código 36, até 12% da renda tributável)";
        }
        if plan == Some(crate::pension::PlanType::Efpc) {
            code = "—";
            label = "Fundo de pensão (EFPC: não vai em Bens e Direitos; as contribuições do ano entram em Pagamentos Efetuados, como previdência complementar, até 12% da renda tributável)";
        }
        let country = if a.class.is_us() {
            "Estados Unidos"
        } else if a.class.is_foreign_listed() {
            // Outras bolsas: o país da bolsa (ou o informado no ativo).
            country_name(
                a.exchange
                    .as_ref()
                    .map(|e| e.country.as_str())
                    .or(a.country.as_deref())
                    .unwrap_or(""),
            )
        } else {
            "Brasil"
        };
        holdings.push(AssetHolding {
            asset_id: a.id.clone(),
            ticker: a.ticker.clone(),
            name: a.name.clone(),
            class_label: a.class_label.clone(),
            group_code: code.into(),
            description: format!(
                "{label}: {} {} — {}. Custo médio de aquisição.",
                st.quantity().normalize(),
                a.ticker.clone().unwrap_or_default(),
                a.name
            ),
            quantity_prev: sp.quantity().normalize().to_string(),
            cost_prev: cents(sp.cost_base()),
            quantity: st.quantity().normalize().to_string(),
            cost: cents(st.cost_base()),
            country: country.into(),
        });
    }
    holdings.sort_by(|a, b| {
        a.group_code
            .cmp(&b.group_code)
            .then(a.ticker.cmp(&b.ticker))
    });
    // Lei 14.754/2023 art. 9º: o prejuízo compensa ganhos do mesmo ano e o saldo passa para
    // os anos seguintes (não abate dividendos e juros comuns).
    let foreign_realized = foreign_realized_in(conn, accounts.as_deref(), year)?;
    let loss_in = foreign_loss_before(conn, accounts.as_deref(), year)?;
    let gain_after_loss = (foreign_realized - loss_in).max(0);
    let loss_out = (loss_in - foreign_realized).max(0);
    let tax_before_credit = cents(
        Decimal::from(gain_after_loss + foreign_income) / Decimal::ONE_HUNDRED * r.foreign_rate,
    );
    let credit = foreign_credit.min(tax_before_credit);
    Ok(TaxYear {
        year,
        member_id: member_id.map(String::from),
        loss_common: pool(LossPool::Common),
        loss_day_trade: pool(LossPool::DayTrade),
        loss_fii: pool(LossPool::Fii),
        irrf_lei_11033_left: months
            .last()
            .map_or(0, |m| m.irrf_carried_out - m.irrf_day_trade_carried_out),
        irrf_day_trade_left: months.last().map_or(0, |m| m.irrf_day_trade_carried_out),
        total_tax: months.iter().map(|m| m.tax).sum(),
        total_darf: months.iter().map(|m| m.darf_amount).sum(),
        exempt_stock_gains: months
            .iter()
            .flat_map(|m| &m.categories)
            .filter(|c| c.category == TaxCategory::Stocks)
            .map(|c| c.exempt_gain)
            .sum(),
        dividends: div.into_values().collect(),
        dividends_over_limit: dividends_over_limit(conn, accounts.as_deref(), year)?,
        fii_income: fii.into_values().collect(),
        jcp: jcp.into_values().collect(),
        holdings,
        foreign: ForeignSummary {
            realized: foreign_realized,
            income: foreign_income,
            tax_paid_abroad: foreign_tax_paid,
            tax_credit: credit,
            tax_without_credit: foreign_no_credit,
            loss_carried_in: loss_in,
            loss_carried_out: loss_out,
            estimated_tax: tax_before_credit - credit,
        },
        months,
        disclaimer: t("nucleo-impostos-ir-aviso"),
    })
}
