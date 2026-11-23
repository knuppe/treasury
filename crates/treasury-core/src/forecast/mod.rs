//! Previsão de saldo (docs/forecast.md): o saldo de cada conta, e o total, dia a dia até 3, 6
//! ou 12 meses, a partir do saldo de hoje e do que já se sabe que vai acontecer:
//!
//! - lançamentos previstos (gerados pelas recorrências e os cadastrados à mão);
//! - recorrências além do horizonte já gerado (até o fim da previsão);
//! - faturas de cartão em aberto, pagas pela conta de pagamento do cartão no vencimento (a
//!   menos que já haja um pagamento previsto para ela);
//! - parcelas de empréstimos e financiamentos com conta de débito (a menos que já haja um
//!   lançamento previsto na categoria do contrato perto do vencimento);
//! - parcelas detectadas pela descrição ("Parcela 3/10", docs/design/detected-installments.md):
//!   as restantes, como estimativa; no cartão, a compra no fechamento e o pagamento no
//!   vencimento pela conta de pagamento (a menos que já haja um previsto parecido).
//! - gastos anuais essenciais (IPVA, IPTU, seguro, material escolar): o que foi pago numa
//!   categoria essencial de poucos meses por ano (`goals::living`) entra de novo doze meses
//!   depois, como estimativa pelo valor do ano anterior (a menos que a categoria já tenha
//!   recorrência, um previsto perto da data ou o pagamento deste ano já lançado).
//!
//! Também diz quando cada conta fica negativa ("fica negativo em DD/MM").

pub mod api;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{add_months, fmt_date, parse_date};

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForecastArgs {
    /// Meses à frente (1 a 24; padrão 3).
    #[serde(default)]
    #[ts(optional)]
    pub months: Option<u32>,
    /// Só estas contas (padrão: as de movimento do dia a dia).
    #[serde(default)]
    #[ts(optional)]
    pub account_ids: Option<Vec<Id>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ForecastSource {
    Planned,
    Recurrence,
    CardStatement,
    Loan,
    /// Parcela detectada pela descrição (estimativa).
    Detected,
    /// Gasto anual essencial repetido do ano anterior (estimativa).
    Seasonal,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForecastEvent {
    pub date: String,
    pub description: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub source: ForecastSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForecastPoint {
    pub date: String,
    #[ts(type = "number")]
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountForecast {
    pub account_id: Id,
    pub name: String,
    pub kind: crate::accounts::AccountKind,
    pub currency: String,
    #[ts(type = "number")]
    pub start: i64,
    #[ts(type = "number")]
    pub end: i64,
    #[ts(type = "number")]
    pub min: i64,
    pub min_date: String,
    /// Primeiro dia com saldo negativo (cartão: nunca; o saldo dele é dívida).
    pub first_negative: Option<String>,
    /// Saldo depois de cada dia com movimento (degraus), do início ao fim.
    pub points: Vec<ForecastPoint>,
    /// O que mexe no saldo (até 300, em ordem de data).
    pub events: Vec<ForecastEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BalanceForecast {
    pub from: String,
    pub to: String,
    pub months: u32,
    pub currency: String,
    pub accounts: Vec<AccountForecast>,
    /// Total das contas na moeda base (o cartão entra como dívida).
    #[ts(type = "number")]
    pub total_start: i64,
    #[ts(type = "number")]
    pub total_end: i64,
    #[ts(type = "number")]
    pub total_min: i64,
    pub total_min_date: String,
    pub total_first_negative: Option<String>,
    pub total_points: Vec<ForecastPoint>,
}

/// Contas que entram por padrão: as do dia a dia (não corretora nem arquivada).
const DAILY: &[&str] = &[
    "checking",
    "savings",
    "cash",
    "credit_card",
    "benefit",
    "other",
    "foreign",
];

struct Acc {
    id: Id,
    name: String,
    kind: crate::accounts::AccountKind,
    currency: String,
    balance: i64,
}

/// Dias em volta da data esperada em que um previsto, ou o pagamento deste ano já lançado, conta
/// como "já está na conta".
const SEASONAL_NEAR_DAYS: i64 = 45;

/// Gastos anuais essenciais do ano anterior que caem de novo dentro da previsão: a conta, a
/// data (doze meses depois), o valor e a descrição. Só categorias essenciais com gasto em
/// poucos meses do ano (`goals::living::annual_expenses`), pagas por uma das contas da previsão
/// que não seja cartão. Ficam de fora a categoria com recorrência ativa, a que já tem um
/// previsto perto da data e a que já foi paga neste ciclo.
fn seasonal(
    conn: &Connection,
    accounts: &[Acc],
    base: &str,
    today: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<(Id, NaiveDate, i64, String)>> {
    let mut fx = crate::reports::engine::Fx::new(conn, base);
    let basis = crate::reports::engine::resolve_basis(conn, None)?;
    let annual = crate::goals::living::annual_expenses(conn, &mut fx, basis, today)?;
    let (Some(first), Some(last)) = (annual.months.first(), annual.months.last()) else {
        return Ok(vec![]);
    };
    let (from, _) = crate::reports::engine::month_dates(first)?;
    let (_, to) = crate::reports::engine::month_dates(last)?;
    let mut out = vec![];
    for item in annual.items.iter().filter(|i| i.essential) {
        let cat = &item.category_id;
        let recurring: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM recurrence WHERE active = 1 AND category_id = ?1)",
            [cat],
            |r| r.get(0),
        )?;
        if recurring {
            continue;
        }
        let mut stmt = conn.prepare_cached(
            "SELECT t.account_id, t.date, t.amount, t.description FROM txn t
              WHERE t.category_id = ?1 AND t.kind = 'expense' AND t.status != 'planned'
                AND t.date BETWEEN ?2 AND ?3
              ORDER BY t.date, t.id",
        )?;
        let rows: Vec<(Id, String, i64, String)> = stmt
            .query_map(params![cat, from, to], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        for (account, date, amount, description) in rows {
            let paid = parse_date(&date)?;
            let expected = add_months(paid, 12);
            if expected <= today || expected > end {
                continue;
            }
            // Cartão: a compra entraria na fatura, que a previsão ainda não estima.
            if !accounts
                .iter()
                .any(|a| a.id == account && !a.kind.is_liability())
            {
                continue;
            }
            let near = Duration::days(SEASONAL_NEAR_DAYS);
            let covered: bool = conn.query_row(
                "SELECT EXISTS (SELECT 1 FROM txn WHERE category_id = ?1 AND kind = 'expense'
                                   AND date BETWEEN ?2 AND ?3 AND (status = 'planned' OR date <= ?4))",
                params![
                    cat,
                    fmt_date(expected - near),
                    fmt_date(expected + near),
                    fmt_date(today)
                ],
                |r| r.get(0),
            )?;
            if covered {
                continue;
            }
            let label = t_args(
                "nucleo-planejamento-sazonal",
                &args(&[
                    (
                        "descricao",
                        if description.trim().is_empty() {
                            item.category.clone()
                        } else {
                            description
                        }
                        .into(),
                    ),
                    ("ano", paid.format("%Y").to_string().into()),
                ]),
            );
            out.push((account, expected, amount, label));
        }
    }
    Ok(out)
}

pub fn forecast(conn: &Connection, a: &ForecastArgs, today: NaiveDate) -> Result<BalanceForecast> {
    let months = a.months.unwrap_or(3).clamp(1, 24);
    let end = add_months(today, months as i32);
    let base = crate::household::base_currency(conn)?;
    let wanted = a.account_ids.as_ref().filter(|v| !v.is_empty());
    let accounts: Vec<Acc> = crate::reports::engine::balances_at(conn, &fmt_date(today))?
        .into_iter()
        .filter(|r| !r.archived)
        .filter(|r| match wanted {
            Some(ids) => ids.contains(&r.id),
            None => DAILY.contains(&r.kind.as_str()),
        })
        .map(|r| Acc {
            id: r.id,
            name: r.name,
            kind: r.kind,
            currency: r.currency,
            balance: r.balance,
        })
        .collect();
    if let Some(ids) = wanted
        && ids.iter().any(|id| !accounts.iter().any(|x| &x.id == id))
    {
        return Err(Error::not_found(t(
            "nucleo-planejamento-conta-ou-arquivada",
        )));
    }
    let mut events: HashMap<Id, Vec<ForecastEvent>> = HashMap::new();
    let mut push = |acc: &str, date: NaiveDate, amount: i64, desc: String, src: ForecastSource| {
        if amount != 0 {
            events
                .entry(acc.to_string())
                .or_default()
                .push(ForecastEvent {
                    date: fmt_date(date),
                    description: desc,
                    amount,
                    source: src,
                });
        }
    };
    let (from_s, to_s) = (fmt_date(today + Duration::days(1)), fmt_date(end));

    // 1) Lançamentos previstos.
    {
        let mut stmt = conn.prepare(
            "SELECT t.account_id, t.date, t.amount, t.description FROM txn t
              WHERE t.status = 'planned' AND t.date BETWEEN ?1 AND ?2",
        )?;
        let mut rows = stmt.query([&from_s, &to_s])?;
        while let Some(r) = rows.next()? {
            let d = parse_date(&r.get::<_, String>(1)?)?;
            push(
                &r.get::<_, String>(0)?,
                d,
                r.get(2)?,
                r.get(3)?,
                ForecastSource::Planned,
            );
        }
    }

    // 2) Recorrências além do que já foi gerado.
    {
        let mut stmt = conn.prepare(
            "SELECT r.account_id, r.to_account_id, r.amount, r.description, r.rrule, r.start_date,
                    r.end_date, r.generated_until
               FROM recurrence r WHERE r.active = 1",
        )?;
        #[allow(clippy::type_complexity)] // linha lida como tupla do SQLite
        let rows: Vec<(
            Id,
            Option<Id>,
            i64,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
        )> = stmt
            .query_map([], |r| {
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
        let mut cards: HashMap<Id, Option<(u32, u32)>> = HashMap::new();
        let mut payers: HashMap<Id, Id> = HashMap::new();
        // Cartão já pago por uma recorrência (pagamento de fatura programado): não paga de novo.
        let paid_by_rule: HashSet<Id> = rows.iter().filter_map(|r| r.1.clone()).collect();
        for (acc, to, amount, desc, rule, start, stop, generated) in rows {
            let start = parse_date(&start)?;
            let stop = stop.as_deref().map(parse_date).transpose()?;
            let after = match generated.as_deref() {
                Some(g) => parse_date(g)?.max(today),
                None => today,
            } + Duration::days(1);
            let spec = crate::recurrence::rrule::parse(&rule)?;
            // No cartão, a compra entra na fatura e sai da conta de pagamento no vencimento.
            let card = match cards.get(&acc) {
                Some(c) => *c,
                None => {
                    let c = crate::accounts::card_details(conn, &acc)?
                        .and_then(|c| c.payment_account_id.map(|p| (c.closing_day, c.due_day, p)));
                    let v = c.as_ref().map(|(cl, du, _)| (*cl, *du));
                    if let Some((_, _, p)) = c {
                        payers.insert(acc.clone(), p);
                    }
                    cards.insert(acc.clone(), v);
                    v
                }
            };
            for d in crate::recurrence::rrule::occurrences_between(&spec, start, stop, after, end)?
            {
                push(&acc, d, amount, desc.clone(), ForecastSource::Recurrence);
                if let Some(t) = &to {
                    push(t, d, amount.abs(), desc.clone(), ForecastSource::Recurrence);
                }
                if to.is_none()
                    && !paid_by_rule.contains(&acc)
                    && let Some((closing, due_day)) = card
                    && let Some(payer) = payers.get(&acc)
                {
                    let due = crate::cards::statement_dates(d, closing, due_day).due_date;
                    if due <= end {
                        push(&acc, due, -amount, desc.clone(), ForecastSource::Recurrence);
                        push(payer, due, amount, desc.clone(), ForecastSource::Recurrence);
                    }
                }
            }
        }
    }

    // 3) Faturas em aberto, pagas no vencimento pela conta de pagamento do cartão.
    {
        let mut stmt = conn.prepare(
            "SELECT cs.id, cs.account_id, cc.payment_account_id, cs.due_date, a.name,
                    COALESCE((SELECT -SUM(t.amount) FROM txn t
                               WHERE t.statement_id = cs.id AND t.kind <> 'card_payment'
                                 AND (t.status <> 'planned' OR t.date >= ?1)), 0),
                    COALESCE((SELECT SUM(t.amount) FROM txn t
                               WHERE t.statement_id = cs.id AND t.kind = 'card_payment'
                                 AND t.status IN ('cleared', 'reconciled')), 0),
                    EXISTS (SELECT 1 FROM txn p WHERE p.status = 'planned'
                              AND ((p.statement_id = cs.id AND p.kind = 'card_payment')
                                OR (p.account_id = cs.account_id AND p.amount > 0
                                    AND p.kind IN ('card_payment', 'transfer')
                                    AND abs(julianday(p.date) - julianday(cs.due_date)) <= 5)))
               FROM card_statement cs
               JOIN account a ON a.id = cs.account_id
               LEFT JOIN credit_card cc ON cc.account_id = cs.account_id
              WHERE cs.status <> 'paid' AND cs.due_date BETWEEN ?1 AND ?2",
        )?;
        let mut rows = stmt.query([&from_s, &to_s])?;
        while let Some(r) = rows.next()? {
            let (card, pay): (Id, Option<Id>) = (r.get(1)?, r.get(2)?);
            let due = parse_date(&r.get::<_, String>(3)?)?;
            let name: String = r.get(4)?;
            let remaining = r.get::<_, i64>(5)? - r.get::<_, i64>(6)?;
            if remaining <= 0 || r.get::<_, bool>(7)? {
                continue;
            }
            let desc = t_args(
                "nucleo-planejamento-fatura",
                &args(&[("cartao", name.into())]),
            );
            push(
                &card,
                due,
                remaining,
                desc.clone(),
                ForecastSource::CardStatement,
            );
            if let Some(p) = pay {
                push(&p, due, -remaining, desc, ForecastSource::CardStatement);
            }
        }
    }

    // 4) Parcelas de contratos com conta de débito.
    {
        let mut stmt = conn.prepare(
            "SELECT l.debit_account_id, i.due_date, i.total, l.name
               FROM loan_installment i JOIN loan l ON l.id = i.loan_id
              WHERE l.archived = 0 AND l.debit_account_id IS NOT NULL AND i.status = 'planned'
                AND i.due_date BETWEEN ?1 AND ?2
                AND NOT EXISTS (SELECT 1 FROM txn p WHERE p.status = 'planned'
                                  AND p.account_id = l.debit_account_id
                                  AND l.category_id IS NOT NULL AND p.category_id = l.category_id
                                  AND abs(julianday(p.date) - julianday(i.due_date)) <= 7)",
        )?;
        let mut rows = stmt.query([&from_s, &to_s])?;
        while let Some(r) = rows.next()? {
            let d = parse_date(&r.get::<_, String>(1)?)?;
            push(
                &r.get::<_, String>(0)?,
                d,
                -r.get::<_, i64>(2)?,
                t_args(
                    "nucleo-planejamento-parcela",
                    &args(&[("contrato", r.get::<_, String>(3)?.into())]),
                ),
                ForecastSource::Loan,
            );
        }
    }

    // 5) Parcelas detectadas pela descrição (estimativa).
    {
        let detected = crate::installments::detect(conn, today)?;
        let mut payers: HashMap<Id, Option<Id>> = HashMap::new();
        // Já há um previsto parecido na conta perto da data: não soma de novo.
        let mut planned = conn.prepare(
            "SELECT EXISTS (SELECT 1 FROM txn WHERE account_id = ?1 AND status = 'planned'
                              AND abs(amount + ?2) <= max(100, abs(?2) / 100)
                              AND abs(julianday(date) - julianday(?3)) <= 7)",
        )?;
        for s in detected.series.iter().filter(|s| !s.missing) {
            let desc = t_args(
                "nucleo-parcelas-previsao",
                &args(&[("descricao", s.description.clone().into())]),
            );
            for p in &s.projected {
                let d = parse_date(&p.date)?;
                // No cartão, a parcela da fatura já fechada (e ainda não importada) não entra no
                // cartão, mas sai da conta de pagamento no vencimento.
                let closed = d <= today && p.due_date.is_some();
                if (d <= today && !closed) || d > end {
                    continue;
                }
                if planned.query_row(params![s.account_id, p.amount, p.date], |r| r.get(0))? {
                    continue;
                }
                let label = format!("{desc} ({}/{})", p.number, s.total);
                if !closed {
                    push(
                        &s.account_id,
                        d,
                        -p.amount,
                        label.clone(),
                        ForecastSource::Detected,
                    );
                }
                if let Some(due) = p.due_date.as_deref() {
                    let due = parse_date(due)?;
                    if due <= today || due > end {
                        continue;
                    }
                    let payer = match payers.get(&s.account_id) {
                        Some(p) => p.clone(),
                        None => {
                            let p = crate::accounts::card_details(conn, &s.account_id)?
                                .and_then(|c| c.payment_account_id);
                            payers.insert(s.account_id.clone(), p.clone());
                            p
                        }
                    };
                    if let Some(pay) = payer {
                        if !closed {
                            push(
                                &s.account_id,
                                due,
                                p.amount,
                                label.clone(),
                                ForecastSource::Detected,
                            );
                        }
                        push(&pay, due, -p.amount, label, ForecastSource::Detected);
                    }
                }
            }
        }
    }

    // 6) Gastos anuais essenciais, pelo ano anterior (estimativa).
    for (acc, date, amount, desc) in seasonal(conn, &accounts, &base, today, end)? {
        push(&acc, date, amount, desc, ForecastSource::Seasonal);
    }

    let mut out = vec![];
    let mut total_daily: BTreeMap<NaiveDate, i64> = BTreeMap::new();
    let mut total_start = 0;
    for acc in accounts {
        let mut evs = events.remove(&acc.id).unwrap_or_default();
        evs.sort_by(|x, y| x.date.cmp(&y.date).then(x.amount.cmp(&y.amount)));
        let in_base = acc.currency == base;
        if in_base {
            total_start += acc.balance;
        }
        let mut balance = acc.balance;
        let (mut min, mut min_date) = (balance, fmt_date(today));
        let liability = acc.kind.is_liability();
        let mut first_negative = (!liability && balance < 0).then(|| fmt_date(today));
        let mut points = vec![ForecastPoint {
            date: fmt_date(today),
            balance,
        }];
        let mut by_day: BTreeMap<&str, i64> = BTreeMap::new();
        for e in &evs {
            *by_day.entry(e.date.as_str()).or_default() += e.amount;
        }
        for (d, delta) in by_day {
            balance += delta;
            if in_base && let Ok(day) = parse_date(d) {
                *total_daily.entry(day).or_default() += delta;
            }
            points.push(ForecastPoint {
                date: d.to_string(),
                balance,
            });
            if balance < min {
                min = balance;
                min_date = d.to_string();
            }
            if !liability && balance < 0 && first_negative.is_none() {
                first_negative = Some(d.to_string());
            }
        }
        evs.truncate(300);
        out.push(AccountForecast {
            account_id: acc.id,
            name: acc.name,
            kind: acc.kind,
            currency: acc.currency,
            start: acc.balance,
            end: balance,
            min,
            min_date,
            first_negative,
            points,
            events: evs,
        });
    }
    let mut total = total_start;
    let (mut total_min, mut total_min_date) = (total, fmt_date(today));
    let mut total_first_negative = (total < 0).then(|| fmt_date(today));
    let mut total_points = vec![ForecastPoint {
        date: fmt_date(today),
        balance: total,
    }];
    for (d, delta) in total_daily {
        total += delta;
        let ds = fmt_date(d);
        if total < total_min {
            total_min = total;
            total_min_date = ds.clone();
        }
        if total < 0 && total_first_negative.is_none() {
            total_first_negative = Some(ds.clone());
        }
        total_points.push(ForecastPoint {
            date: ds,
            balance: total,
        });
    }
    let _ = params![];
    Ok(BalanceForecast {
        from: fmt_date(today),
        to: fmt_date(end),
        months,
        currency: base,
        accounts: out,
        total_start,
        total_end: total,
        total_min,
        total_min_date,
        total_first_negative,
        total_points,
    })
}

// ---------------------------------------------------------------------------
// Quanto posso gastar hoje

/// "Quanto posso gastar hoje": o que sobra do mês para os gastos do dia a dia, por dia.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DailyAllowance {
    pub month: String,
    /// Dias que faltam no mês, contando hoje.
    pub days_left: u32,
    /// Receitas do mês: o que já entrou e o que está previsto até o fim do mês.
    #[ts(type = "number")]
    pub income: i64,
    /// Gastos do mês até hoje, incluindo os de hoje (competência).
    #[ts(type = "number")]
    pub spent: i64,
    /// Gastos de hoje.
    #[ts(type = "number")]
    pub spent_today: i64,
    /// Contas fixas que ainda vencem no mês (previstos e parcelas).
    #[ts(type = "number")]
    pub fixed_remaining: i64,
    /// O que ainda falta guardar no mês para as metas (contribuição mensal menos o já aportado).
    #[ts(type = "number")]
    pub savings_remaining: i64,
    /// Disponível para o resto do mês, depois dos gastos de hoje (pode ser negativo).
    #[ts(type = "number")]
    pub available: i64,
    /// Por dia, a partir de hoje: o que sobrava no começo do dia (`available` mais os gastos
    /// de hoje) dividido pelos dias que faltam, contando hoje (zero quando não sobra).
    #[ts(type = "number")]
    pub per_day: i64,
    /// Quanto ainda dá para gastar hoje (por dia menos o que já foi gasto hoje).
    #[ts(type = "number")]
    pub today_left: i64,
    /// Por dia a partir de amanhã: `available` dividido pelos dias depois de hoje (zero no
    /// último dia do mês ou quando não sobra).
    #[ts(type = "number")]
    pub per_day_from_tomorrow: i64,
    pub currency: String,
    /// `ok`, `tight` (menos de 30% do que entra por dia), `over` (já passou) ou `no_income`
    /// (nenhuma receita recebida nem prevista no mês: sem ela não há o que dividir, e a falta
    /// de dado não vira "mês estourado").
    pub status: String,
    pub explanation: String,
}

pub fn daily_allowance(conn: &Connection, today: NaiveDate) -> Result<DailyAllowance> {
    use crate::reports::engine::{FlowQuery, Fx, Key, Status, month_dates};
    use chrono::Datelike;
    let month = format!("{:04}-{:02}", today.year(), today.month());
    let (first, last) = month_dates(&month)?;
    let last_day = parse_date(&last)?;
    let days_left = ((last_day - today).num_days() + 1).max(1) as u32;
    let base = crate::household::base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let today_s = fmt_date(today);
    let realized = FlowQuery::new(&first, &today_s, crate::budget::Basis::Accrual)
        .aggregate(conn, &mut fx, Key::None, Key::None)?
        .total();
    let spent_today = FlowQuery::new(&today_s, &today_s, crate::budget::Basis::Accrual)
        .aggregate(conn, &mut fx, Key::None, Key::None)?
        .total()
        .expense;
    let tomorrow = fmt_date(today + Duration::days(1));
    let mut planned_q = FlowQuery::new(&tomorrow, &last, crate::budget::Basis::Accrual);
    planned_q.status = Status::Planned;
    let planned = if tomorrow <= last {
        planned_q
            .aggregate(conn, &mut fx, Key::None, Key::None)?
            .total()
    } else {
        crate::reports::engine::Cell::default()
    };
    // Parcelas do mês sem lançamento previsto (como na previsão de saldo).
    let loans: i64 = conn.query_row(
        "SELECT COALESCE(SUM(i.total), 0) FROM loan_installment i JOIN loan l ON l.id = i.loan_id
          WHERE l.archived = 0 AND i.status = 'planned' AND i.due_date BETWEEN ?1 AND ?2
            AND NOT EXISTS (SELECT 1 FROM txn p WHERE p.status = 'planned'
                              AND l.category_id IS NOT NULL AND p.category_id = l.category_id
                              AND abs(julianday(p.date) - julianday(i.due_date)) <= 7)",
        params![today_s, last],
        |r| r.get(0),
    )?;
    let goals: i64 = conn.query_row(
        "SELECT COALESCE(SUM(monthly_contribution), 0) FROM goal WHERE archived = 0",
        [],
        |r| r.get(0),
    )?;
    let invested: i64 = conn.query_row(
        "SELECT COALESCE(-SUM(t.amount), 0) FROM txn t JOIN account a ON a.id = t.account_id
          WHERE t.kind = 'investment' AND t.amount < 0 AND a.kind NOT IN ('brokerage', 'retirement')
            AND t.status IN ('cleared', 'reconciled') AND t.date BETWEEN ?1 AND ?2",
        params![first, today_s],
        |r| r.get(0),
    )?;
    let income = realized.income + planned.income;
    let fixed_remaining = planned.expense + loans;
    let savings_remaining = (goals - invested).max(0);
    // O dia de hoje divide o que sobrava no começo dele: os gastos de hoje saem do valor de
    // hoje (e não também do que sobra para o mês, o que os contaria duas vezes).
    let spent_before = realized.expense - spent_today;
    let available_start = income - spent_before - fixed_remaining - savings_remaining;
    let available = available_start - spent_today;
    let per_day = (available_start / i64::from(days_left)).max(0);
    let today_left = (per_day - spent_today).max(0);
    let per_day_from_tomorrow = if days_left > 1 {
        (available / i64::from(days_left - 1)).max(0)
    } else {
        0
    };
    let daily_income = income / i64::from(days_left.max(1) + today.day() - 1);
    let status = if income <= 0 {
        "no_income"
    } else if available <= 0 {
        "over"
    } else if per_day * 10 < daily_income * 3 {
        "tight"
    } else {
        "ok"
    };
    let m = |v: i64| -> fluent_bundle::FluentValue<'static> {
        crate::i18n::money(crate::i18n::current(), v, &base).into()
    };
    let mut explanation = if income <= 0 {
        t("nucleo-planejamento-gastar-sem-renda")
    } else {
        t_args(
            "nucleo-planejamento-gastar-hoje",
            &args(&[
                ("receitas", m(income)),
                ("gastos", m(spent_before)),
                ("contas", m(fixed_remaining)),
                ("guardar", m(savings_remaining)),
                ("disponivel", m(available_start)),
                ("dias", days_left.into()),
                ("por_dia", m(per_day)),
            ]),
        )
    };
    if income > 0 && spent_today > 0 && days_left > 1 {
        explanation.push(' ');
        explanation.push_str(&t_args(
            "nucleo-planejamento-gastar-amanha",
            &args(&[
                ("hoje", m(spent_today)),
                ("amanha", m(per_day_from_tomorrow)),
            ]),
        ));
    }
    Ok(DailyAllowance {
        month,
        days_left,
        income,
        spent: realized.expense,
        spent_today,
        fixed_remaining,
        savings_remaining,
        available,
        per_day,
        today_left,
        per_day_from_tomorrow,
        currency: base,
        status: status.into(),
        explanation,
    })
}
