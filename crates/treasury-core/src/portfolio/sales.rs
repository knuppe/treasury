//! Livro de vendas (docs/sales-real-return.md): cada venda com o resultado nominal e o
//! real, descontada a inflação (IPCA, IGP-M, inflação do país, US CPI) ou a expansão
//! monetária (M2) entre as compras e a venda.
//!
//! No preço médio, cada venda leva a mesma fração de todas as compras que formaram a posição.
//! O livro guarda essas camadas (data e custo na moeda base) e, a cada venda, tira a fração
//! vendida de cada uma: o custo corrigido é a soma de cada pedaço vezes o fator do índice da
//! data da compra até a da venda, e o prazo médio é ponderado pelo custo. Renda fixa e
//! Tesouro usam os lotes do PEPS que a carteira já usa. O custo e o resultado nominal são os
//! da carteira (e do IR).
//!
//! Proventos (retorno total): os recebidos pelas cotas vendidas entre a compra e a venda
//! (dividendos, JCP, rendimentos de FII, juros, cupons; líquidos do IR retido). Como o custo,
//! eles formam um saldo da posição que cada venda leva na mesma fração; entram pela data-com
//! (sem ela, pelo pagamento) quando ela é anterior à venda. Amortização não entra: ela devolve
//! capital e já reduz o custo. O resultado "com proventos" fica ao lado do de preço, que é o
//! do IR; o real traz cada provento à data da venda pelo mesmo índice.
//!
//! Método de custo (docs/cost-method.md): fora do Brasil, o custo da
//! renda variável segue a regra do país da família (PEPS ou o pool do Reino Unido, ver
//! [`super::cost_method`]); o custo e o resultado pelo preço médio ficam ao lado
//! (`average_cost`, `average_result`), que é a base do IR brasileiro. Cada venda traz também o
//! retorno anualizado (`irr`): a taxa que iguala o custo de cada compra que saiu, os proventos e
//! o valor da venda.

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::cost_method::{self, CostMethod, Move};
use super::engine::{self, Step};
use super::records::to_minor;
use crate::assets::AssetClass;
use crate::error::Result;
use crate::ids::Id;
use crate::indices::{self, Deflator};
use crate::time::{fmt_date, parse_date, today_local};

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "SalesArgs")]
pub struct SalesArgs {
    /// Início `YYYY-MM-DD` (inclusive); sem ele, desde a primeira venda.
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    /// Fim `YYYY-MM-DD` (inclusive); padrão hoje.
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub deflator: Option<Deflator>,
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub account_ids: Option<Vec<Id>>,
    #[serde(default)]
    #[ts(optional)]
    pub classes: Option<Vec<AssetClass>>,
    /// Método de custo da renda variável (padrão: o de Ajustes, ou o do país da família).
    #[serde(default)]
    #[ts(optional)]
    pub cost_method: Option<CostMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "SaleRow")]
pub struct SaleRow {
    pub trade_id: Id,
    pub date: String,
    pub asset_id: Id,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub account_id: Id,
    pub account_name: String,
    pub quantity: String,
    /// Valor líquido da venda (preço × quantidade − custos), na moeda base, em centavos.
    #[ts(type = "number")]
    pub proceeds: i64,
    /// Custo pelo método do relatório (preço médio, PEPS ou o pool do Reino Unido; renda fixa:
    /// os lotes PEPS), em centavos.
    #[ts(type = "number")]
    pub cost: i64,
    #[ts(type = "number")]
    pub result: i64,
    pub result_pct: Option<f64>,
    /// Custo trazido para a data da venda pelo deflator (sem dado do índice: `null`).
    #[ts(type = "number | null")]
    pub real_cost: Option<i64>,
    #[ts(type = "number | null")]
    pub real_result: Option<i64>,
    pub real_pct: Option<f64>,
    /// Inflação (ou M2) do período, ponderada pelo custo: fator − 1.
    pub index_change: Option<f64>,
    /// Dias entre as compras e a venda, ponderados pelo custo.
    #[ts(type = "number | null")]
    pub holding_days: Option<i64>,
    /// Algum mês do índice é estimado (≈).
    pub estimated: bool,
    /// Proventos recebidos pelas cotas vendidas entre a compra e a venda (líquidos), em
    /// centavos na moeda base.
    #[ts(type = "number")]
    pub income: i64,
    /// Resultado com proventos: `result + income` (o IR continua sobre `result`).
    #[ts(type = "number")]
    pub total_result: i64,
    pub total_pct: Option<f64>,
    /// Proventos trazidos à data da venda pelo deflator.
    #[ts(type = "number | null")]
    pub real_income: Option<i64>,
    /// Resultado real com proventos: `real_result + real_income`.
    #[ts(type = "number | null")]
    pub real_total_result: Option<i64>,
    pub real_total_pct: Option<f64>,
    /// Custo e resultado pelo preço médio (a base do IR brasileiro), quando o método do
    /// relatório é outro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub average_cost: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub average_result: Option<i64>,
    /// Retorno anualizado da venda com os proventos (fração ao ano; sem prazo, `null`).
    pub irr: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "SalesTotals")]
pub struct SalesTotals {
    pub count: u32,
    /// Vendas com ganho e com perda (nominal).
    pub gains: u32,
    pub losses: u32,
    /// Vendas com ganho e com perda (real), entre as que têm índice.
    pub real_gains: u32,
    pub real_losses: u32,
    #[ts(type = "number")]
    pub proceeds: i64,
    #[ts(type = "number")]
    pub cost: i64,
    #[ts(type = "number")]
    pub result: i64,
    /// Soma das vendas com índice (as sem índice ficam de fora e contam em `missing`).
    #[ts(type = "number | null")]
    pub real_cost: Option<i64>,
    #[ts(type = "number | null")]
    pub real_result: Option<i64>,
    /// Quanto a inflação (ou o M2) comeu do resultado: custo corrigido − custo.
    #[ts(type = "number | null")]
    pub inflation_effect: Option<i64>,
    /// Vendas sem dado do índice no período.
    pub missing: u32,
    /// Proventos das cotas vendidas e o resultado com eles (nominal e real).
    #[serde(default)]
    #[ts(type = "number")]
    pub income: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub total_result: i64,
    #[serde(default)]
    #[ts(type = "number | null")]
    pub real_total_result: Option<i64>,
    /// Resultado pelo preço médio (IR brasileiro), quando o método do relatório é outro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub average_result: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "SalesGroup")]
pub struct SalesGroup {
    /// Mês `YYYY-MM`, ano `YYYY`, classe ou id do ativo, conforme a lista.
    pub key: String,
    pub totals: SalesTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "SalesReport")]
pub struct SalesReport {
    pub deflator: Deflator,
    /// Método de custo da renda variável e se ele segue o país da família.
    pub cost_method: CostMethod,
    pub cost_method_automatic: bool,
    pub currency: String,
    pub from: Option<String>,
    pub to: String,
    /// Mais recentes primeiro.
    pub rows: Vec<SaleRow>,
    pub totals: SalesTotals,
    pub by_month: Vec<SalesGroup>,
    pub by_year: Vec<SalesGroup>,
    pub by_class: Vec<SalesGroup>,
    pub by_asset: Vec<SalesGroup>,
}

/// Custo na moeda base ainda em carteira, por data de compra.
type Layers = Vec<(NaiveDate, Decimal)>;

/// Uma venda achada na reprodução, com o custo que saiu de cada camada.
struct Found {
    date: NaiveDate,
    qty: Decimal,
    cost: Decimal,
    /// Custo pelo preço médio (o da carteira), quando o método é outro.
    average: Option<Decimal>,
    /// Posição da venda entre as do ativo (para o casamento por lote).
    disp: usize,
    proceeds: Decimal,
    out: Vec<(NaiveDate, Decimal)>,
    /// Proventos que saíram com as cotas vendidas: (data do pagamento, valor na moeda base).
    income: Vec<(NaiveDate, Decimal)>,
    trade_id: Id,
    account_id: Id,
}

struct Factors<'a> {
    conn: &'a Connection,
    deflator: Deflator,
    cache: HashMap<(NaiveDate, NaiveDate), Option<(Decimal, bool)>>,
}

impl Factors<'_> {
    fn get(&mut self, from: NaiveDate, to: NaiveDate) -> Result<Option<(Decimal, bool)>> {
        if let Some(v) = self.cache.get(&(from, to)) {
            return Ok(*v);
        }
        let v = indices::deflator_factor(self.conn, self.deflator, from, to)?
            .map(|f| (f.factor, f.estimated));
        self.cache.insert((from, to), v);
        Ok(v)
    }
}

/// O que saiu de cada camada numa venda: a fração do custo (preço médio) ou os lotes (PEPS).
fn take(layers: &mut Layers, s: &Step<'_>) -> Vec<(NaiveDate, Decimal)> {
    if !s.lots_out.is_empty() {
        // PEPS: os lotes que saíram; as camadas acompanham na mesma proporção.
        if s.cost_before > Decimal::ZERO {
            let keep = s.cost_after / s.cost_before;
            for l in layers.iter_mut() {
                l.1 *= keep;
            }
        }
        return s.lots_out.clone();
    }
    if s.cost_before <= Decimal::ZERO {
        return vec![];
    }
    let share = ((s.cost_before - s.cost_after) / s.cost_before).min(Decimal::ONE);
    let out: Vec<(NaiveDate, Decimal)> = layers.iter().map(|(d, c)| (*d, *c * share)).collect();
    for l in layers.iter_mut() {
        l.1 -= l.1 * share;
    }
    layers.retain(|(_, c)| !c.is_zero());
    out
}

/// Compras, bonificações e heranças viram camadas; amortização e come-cotas mexem no custo
/// de todas na mesma proporção.
fn track(layers: &mut Layers, s: &Step<'_>) {
    if s.reset {
        layers.clear();
        return;
    }
    let delta = s.cost_after - s.cost_before;
    if delta > Decimal::ZERO {
        layers.push((s.date, delta));
    } else if delta < Decimal::ZERO && s.cost_before > Decimal::ZERO {
        let keep = s.cost_after / s.cost_before;
        for l in layers.iter_mut() {
            l.1 *= keep;
        }
    }
}

/// Provento: data que dá direito, data do pagamento e valor líquido na moeda base.
type IncomeAt = (NaiveDate, NaiveDate, Decimal);

/// Proventos recebidos de um ativo (sem amortização), na ordem em que entram na posição.
fn incomes(conn: &Connection, asset_id: &str) -> Result<Vec<IncomeAt>> {
    let mut stmt = conn.prepare(
        "SELECT ex_date, pay_date, net, currency, fx_rate FROM income_event
          WHERE asset_id = ?1 AND status = 'received' AND kind <> 'amortization'",
    )?;
    type Row = (Option<String>, String, i64, String, Option<String>);
    let rows: Vec<Row> = stmt
        .query_map([asset_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (ex, pay, net, cur, fx) in rows {
        let pay = parse_date(&pay)?;
        let on = match ex.as_deref().filter(|e| !e.is_empty()) {
            Some(e) => parse_date(e)?,
            None => pay,
        };
        let rate = match fx.and_then(|f| f.parse::<Decimal>().ok()) {
            Some(f) => Some(f),
            None => engine::fx_to_base(conn, &cur, pay)?.map(|(r, _)| r),
        };
        if let Some(rate) = rate {
            out.push((on, pay, super::records::from_minor(net, &cur) * rate));
        }
    }
    out.sort_by_key(|(on, pay, _)| (*on, *pay));
    Ok(out)
}

fn pct(num: i64, den: i64) -> Option<f64> {
    (den > 0).then(|| num as f64 / den as f64)
}

fn add(t: &mut SalesTotals, r: &SaleRow) {
    t.count += 1;
    t.proceeds += r.proceeds;
    t.cost += r.cost;
    t.result += r.result;
    t.income += r.income;
    t.total_result += r.total_result;
    t.average_result = Some(t.average_result.unwrap_or(0) + r.average_result.unwrap_or(r.result));
    if let Some(v) = r.real_total_result {
        t.real_total_result = Some(t.real_total_result.unwrap_or(0) + v);
    }
    if r.result > 0 {
        t.gains += 1;
    } else if r.result < 0 {
        t.losses += 1;
    }
    match (r.real_cost, r.real_result) {
        (Some(c), Some(res)) => {
            t.real_cost = Some(t.real_cost.unwrap_or(0) + c);
            t.real_result = Some(t.real_result.unwrap_or(0) + res);
            t.inflation_effect = Some(t.inflation_effect.unwrap_or(0) + c - r.cost);
            if res > 0 {
                t.real_gains += 1;
            } else if res < 0 {
                t.real_losses += 1;
            }
        }
        _ => t.missing += 1,
    }
}

fn groups(rows: &[SaleRow], key: impl Fn(&SaleRow) -> String) -> Vec<SalesGroup> {
    let mut m: BTreeMap<String, SalesTotals> = BTreeMap::new();
    for r in rows {
        add(m.entry(key(r)).or_default(), r);
    }
    m.into_iter()
        .map(|(key, totals)| SalesGroup { key, totals })
        .collect()
}

pub fn sales(conn: &Connection, args: &SalesArgs) -> Result<SalesReport> {
    let deflator = indices::effective_deflator(conn, args.deflator.unwrap_or_default());
    let state = cost_method::state(conn)?;
    let method = args.cost_method.unwrap_or(state.method);
    let base = crate::household::base_currency(conn)?;
    let to = match &args.to {
        Some(d) => parse_date(d)?,
        None => today_local(),
    };
    let from = args.from.as_deref().map(parse_date).transpose()?;
    let accounts: HashMap<String, String> = {
        let mut stmt = conn.prepare("SELECT id, name FROM account")?;
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?
    };
    // Ativos com venda no período (o preço médio é o de todas as contas, como no IR).
    let assets: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT asset_id FROM trade
              WHERE side = 'sell' AND date <= ?1 AND date >= coalesce(?2, '0000-00-00')
                AND (?3 IS NULL OR asset_id = ?3)",
        )?;
        stmt.query_map(
            rusqlite::params![fmt_date(to), from.map(fmt_date), args.asset_id],
            |r| r.get(0),
        )?
        .collect::<rusqlite::Result<_>>()?
    };
    let mut factors = Factors {
        conn,
        deflator,
        cache: HashMap::new(),
    };
    let mut rows = vec![];
    for id in assets {
        let book = engine::load_book(conn, &id, None)?;
        let asset = book.asset.clone();
        if let Some(c) = &args.classes
            && !c.is_empty()
            && !c.contains(&asset.class)
        {
            continue;
        }
        let mut layers: Layers = vec![];
        let mut found: Vec<Found> = vec![];
        // Movimentos da posição para o PEPS e o pool (renda fixa já sai por lotes).
        let by_lot = method != CostMethod::Average && !engine::is_curve(&asset);
        let mut moves: Vec<Move> = vec![];
        let mut disp = 0usize;
        // Proventos da posição ainda não levados por uma venda (data do pagamento, valor).
        let events = incomes(conn, &id)?;
        let mut next = 0;
        let mut pool: Vec<(NaiveDate, Decimal)> = vec![];
        engine::replay_observed(conn, &book, to, &mut |s| {
            // Com direito antes deste passo (a venda na data-com já não leva o provento).
            while let Some((_, pay, v)) = events.get(next).filter(|e| e.0 < s.date) {
                pool.push((*pay, *v));
                next += 1;
            }
            if s.reset {
                pool.clear();
            }
            if by_lot {
                moves.extend(lot_move(s, disp));
            }
            let Some(t) = s.sell else {
                track(&mut layers, s);
                return Ok(());
            };
            // Proventos: a fração das cotas vendidas (no preço médio, a mesma do custo).
            let share = if by_lot {
                if s.qty_before > Decimal::ZERO {
                    (s.qty_sold / s.qty_before).min(Decimal::ONE)
                } else {
                    Decimal::ZERO
                }
            } else if s.cost_before > Decimal::ZERO {
                ((s.cost_before - s.cost_after) / s.cost_before).min(Decimal::ONE)
            } else {
                Decimal::ZERO
            };
            let income: Vec<(NaiveDate, Decimal)> =
                pool.iter().map(|(d, v)| (*d, *v * share)).collect();
            for p in pool.iter_mut() {
                p.1 -= p.1 * share;
            }
            pool.retain(|p| !p.1.is_zero());
            let out = take(&mut layers, s);
            let wanted = from.is_none_or(|f| s.date >= f)
                && args
                    .account_ids
                    .as_ref()
                    .is_none_or(|a| a.is_empty() || a.contains(&t.account_id));
            if wanted {
                found.push(Found {
                    date: s.date,
                    qty: s.qty_sold,
                    cost: s.cost_before - s.cost_after,
                    average: None,
                    disp,
                    proceeds: s.proceeds_base,
                    out,
                    income,
                    trade_id: t.id.clone(),
                    account_id: t.account_id.clone(),
                });
            }
            disp += 1;
            Ok(())
        })?;
        if by_lot {
            let matched = cost_method::match_lots(method, &moves);
            for f in found.iter_mut() {
                if let Some(m) = matched.get(&f.disp) {
                    f.average = Some(f.cost);
                    f.cost = m.cost;
                    f.out = m.out.clone();
                }
            }
        }
        for Found {
            date,
            qty,
            cost,
            average,
            disp: _,
            proceeds,
            out,
            income,
            trade_id,
            account_id,
        } in found
        {
            let cost_m = to_minor(cost, &base);
            let proceeds_m = to_minor(proceeds, &base);
            let (mut real, mut estimated, mut weight, mut days) =
                (Some(Decimal::ZERO), false, Decimal::ZERO, Decimal::ZERO);
            for (d, c) in &out {
                weight += *c;
                days += *c * Decimal::from((date - *d).num_days().max(0));
                match factors.get(*d, date)? {
                    Some((f, est)) => {
                        real = real.map(|r| r + *c * f);
                        estimated |= est;
                    }
                    None => real = None,
                }
            }
            // Sem camadas (posição herdada sem custo, por exemplo): o custo vale como está.
            if out.is_empty() {
                real = Some(cost);
            }
            let real_cost = real.map(|r| to_minor(r, &base));
            let real_result = real_cost.map(|c| proceeds_m - c);
            // Proventos: nominal e trazidos à data da venda.
            let income_sum: Decimal = income.iter().map(|(_, v)| *v).sum();
            let mut real_inc = Some(Decimal::ZERO);
            for (d, v) in &income {
                match factors.get(*d, date)? {
                    Some((f, est)) => {
                        real_inc = real_inc.map(|r| r + *v * f);
                        estimated |= est;
                    }
                    None => real_inc = None,
                }
            }
            let income_m = to_minor(income_sum, &base);
            let real_income = real_inc.map(|r| to_minor(r, &base));
            let average_cost = average.map(|a| to_minor(a, &base));
            let irr = sale_irr(&out, &income, date, proceeds);
            let real_total_result = real_result.zip(real_income).map(|(a, b)| a + b);
            rows.push(SaleRow {
                trade_id,
                date: fmt_date(date),
                asset_id: asset.id.clone(),
                ticker: asset.ticker.clone(),
                name: asset.name.clone(),
                class: asset.class,
                account_name: accounts.get(&account_id).cloned().unwrap_or_default(),
                account_id,
                quantity: qty.normalize().to_string(),
                proceeds: proceeds_m,
                cost: cost_m,
                result: proceeds_m - cost_m,
                result_pct: pct(proceeds_m - cost_m, cost_m),
                real_cost,
                real_result,
                real_pct: real_cost.and_then(|c| pct(proceeds_m - c, c)),
                index_change: (cost > Decimal::ZERO)
                    .then(|| real.map(|r| r / cost - Decimal::ONE))
                    .flatten()
                    .and_then(|v| v.to_f64()),
                holding_days: (weight > Decimal::ZERO)
                    .then(|| (days / weight).round().to_i64())
                    .flatten(),
                estimated,
                income: income_m,
                total_result: proceeds_m - cost_m + income_m,
                total_pct: pct(proceeds_m - cost_m + income_m, cost_m),
                real_income,
                real_total_result,
                real_total_pct: real_cost
                    .zip(real_total_result)
                    .and_then(|(c, r)| pct(r, c)),
                average_cost,
                average_result: average_cost.map(|c| proceeds_m - c),
                irr,
            });
        }
    }
    rows.sort_by(|a, b| b.date.cmp(&a.date).then(a.name.cmp(&b.name)));
    let mut totals = SalesTotals::default();
    for r in &rows {
        add(&mut totals, r);
    }
    let mut report = SalesReport {
        deflator,
        cost_method: method,
        cost_method_automatic: args.cost_method.is_none() && state.automatic,
        currency: base,
        from: from.map(fmt_date),
        to: fmt_date(to),
        by_month: groups(&rows, |r| r.date[..7].to_string()),
        by_year: groups(&rows, |r| r.date[..4].to_string()),
        by_class: groups(&rows, |r| r.class.as_str().to_string()),
        by_asset: groups(&rows, |r| r.asset_id.clone()),
        totals,
        rows,
    };
    // No preço médio, o resultado do IR é o próprio resultado.
    if method == CostMethod::Average {
        report.totals.average_result = None;
        for g in report
            .by_month
            .iter_mut()
            .chain(report.by_year.iter_mut())
            .chain(report.by_class.iter_mut())
            .chain(report.by_asset.iter_mut())
        {
            g.totals.average_result = None;
        }
    }
    Ok(report)
}

/// O movimento da posição num passo da carteira, para o PEPS e o pool.
fn lot_move(s: &Step<'_>, disp: usize) -> Option<Move> {
    if s.reset {
        return Some(Move::Reset);
    }
    if s.sell.is_some() {
        return Some(Move::Disp {
            date: s.date,
            qty: s.qty_sold,
            id: disp,
        });
    }
    let (qb, qa) = (s.qty_before, s.qty_after);
    let (cb, ca) = (s.cost_before, s.cost_after);
    let ratio = |a: Decimal, b: Decimal| {
        if b > Decimal::ZERO {
            a / b
        } else {
            Decimal::ONE
        }
    };
    if qa > qb && (ca > cb || qb.is_zero()) {
        // Compra, herança ou bonificação com custo: um lote novo.
        return Some(Move::Acq {
            date: s.date,
            qty: qa - qb,
            cost: (ca - cb).max(Decimal::ZERO),
        });
    }
    if qa == qb && ca == cb {
        return None;
    }
    // Desdobramento, grupamento, bonificação sem custo, come-cotas ou amortização.
    Some(Move::Scale {
        qty: ratio(qa, qb),
        cost: ratio(ca, cb),
    })
}

/// Retorno anualizado de uma venda: o custo de cada compra que saiu, os proventos recebidos e o
/// valor da venda.
fn sale_irr(
    out: &[(NaiveDate, Decimal)],
    income: &[(NaiveDate, Decimal)],
    date: NaiveDate,
    proceeds: Decimal,
) -> Option<f64> {
    use crate::finance::returns::{Flow, xirr};
    let mut flows: Vec<Flow> = out
        .iter()
        .filter(|(_, c)| *c > Decimal::ZERO)
        .map(|(d, c)| Flow {
            date: *d,
            amount: -c.to_f64().unwrap_or(0.0),
        })
        .collect();
    // Sem prazo (compra e venda no mesmo dia) a taxa ao ano não faz sentido.
    if flows.iter().all(|f| f.date >= date) {
        return None;
    }
    flows.extend(income.iter().map(|(d, v)| Flow {
        date: *d,
        amount: v.to_f64().unwrap_or(0.0),
    }));
    flows.push(Flow {
        date,
        amount: proceeds.to_f64().unwrap_or(0.0),
    });
    xirr(&flows)
}
