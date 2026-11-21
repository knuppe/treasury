//! Acréscimos de DARF pago em atraso (Lei 9.430/1996 art. 61):
//! - multa de mora de 0,33% por dia de atraso, contada do dia seguinte ao vencimento até o dia
//!   do pagamento, limitada a 20% (caput e §§ 1º e 2º);
//! - juros pela Selic acumulada mensalmente, do primeiro dia do mês seguinte ao vencimento até
//!   o mês anterior ao do pagamento, mais 1% no mês do pagamento (§3º; Perguntas e Respostas
//!   IRPF 2026, perguntas 063 e 064).
//!
//! A Selic de cada mês vem da série diária do cofre (`selic`, BCB SGS 11), composta no mês;
//! mês sem dado usa o último mês conhecido e o resultado fica marcado como estimado.

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::time::add_months;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LateCharges {
    /// Dias de atraso (corridos, do dia seguinte ao vencimento até o pagamento).
    #[ts(type = "number")]
    pub days: i64,
    /// Multa em % (texto decimal, ex.: "3.3"), já limitada a 20%.
    pub fine_pct: String,
    #[ts(type = "number")]
    pub fine: i64,
    /// Juros em % (Selic dos meses entre o vencimento e o pagamento + 1%).
    pub interest_pct: String,
    #[ts(type = "number")]
    pub interest: i64,
    #[ts(type = "number")]
    pub total: i64,
    /// Algum mês sem Selic no cofre (usado o último conhecido).
    pub estimated: bool,
}

fn cents(v: Decimal) -> i64 {
    v.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap_or(0)
}

/// Selic acumulada em cada mês (% no mês), composta a partir da série diária.
fn monthly_selic(
    conn: &Connection,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<(i32, u32, Decimal)>> {
    let mut out: Vec<(i32, u32, Decimal)> = vec![];
    for (d, rate) in crate::indices::daily_rates(conn, "selic", from, to)? {
        let factor = Decimal::ONE + rate / Decimal::ONE_HUNDRED;
        match out.last_mut() {
            Some((y, m, acc)) if *y == d.year() && *m == d.month() => *acc *= factor,
            _ => out.push((d.year(), d.month(), factor)),
        }
    }
    Ok(out
        .into_iter()
        .map(|(y, m, f)| (y, m, (f - Decimal::ONE) * Decimal::ONE_HUNDRED))
        .collect())
}

/// Juros em %: soma da Selic mensal dos meses estritamente entre o do vencimento e o do
/// pagamento, mais 1% no mês do pagamento. `monthly` dá a Selic de cada mês (se conhecida).
pub fn interest_pct(
    due: NaiveDate,
    paid: NaiveDate,
    monthly: &dyn Fn(i32, u32) -> Option<Decimal>,
) -> (Decimal, bool) {
    if paid <= due {
        return (Decimal::ZERO, false);
    }
    let mut pct = Decimal::ZERO;
    let mut estimated = false;
    let mut last_known = None;
    let first = NaiveDate::from_ymd_opt(due.year(), due.month(), 1).expect("data");
    let mut m = add_months(first, 1);
    let pay_month = NaiveDate::from_ymd_opt(paid.year(), paid.month(), 1).expect("data");
    while m < pay_month {
        match monthly(m.year(), m.month()) {
            Some(v) => {
                pct += v;
                last_known = Some(v);
            }
            None => {
                estimated = true;
                pct += last_known.unwrap_or(Decimal::ONE);
            }
        }
        m = add_months(m, 1);
    }
    (pct + Decimal::ONE, estimated)
}

/// Multa e juros de um DARF de `principal` (centavos) vencido em `due` e pago em `paid`.
pub fn compute(
    principal: i64,
    due: NaiveDate,
    paid: NaiveDate,
    monthly: &dyn Fn(i32, u32) -> Option<Decimal>,
) -> LateCharges {
    let days = (paid - due).num_days().max(0);
    let fine_pct = (Decimal::new(33, 2) * Decimal::from(days)).min(Decimal::from(20));
    let fine = cents(Decimal::from(principal) * fine_pct / Decimal::ONE_HUNDRED);
    let (interest_pct, estimated) = interest_pct(due, paid, monthly);
    let interest = cents(Decimal::from(principal) * interest_pct / Decimal::ONE_HUNDRED);
    LateCharges {
        days,
        fine_pct: fine_pct.normalize().to_string(),
        fine,
        interest_pct: interest_pct.round_dp(2).normalize().to_string(),
        interest,
        total: principal + fine + interest,
        estimated,
    }
}

/// Acréscimos com a Selic do cofre. A Selic do mês em curso (incompleta) não é usada: ela só
/// entra em meses já fechados, e o mês do pagamento leva o 1% fixo.
pub fn for_darf(
    conn: &Connection,
    principal: i64,
    due: NaiveDate,
    paid: NaiveDate,
) -> Result<LateCharges> {
    let rates = monthly_selic(conn, due, paid)?;
    let today = crate::time::today_local();
    let lookup = |y: i32, m: u32| {
        let closed = NaiveDate::from_ymd_opt(y, m, 1)
            .map(|d| add_months(d, 1) <= today)
            .unwrap_or(false);
        rates
            .iter()
            .find(|(ry, rm, _)| *ry == y && *rm == m)
            .filter(|_| closed)
            .map(|(_, _, v)| *v)
    };
    Ok(compute(principal, due, paid, &lookup))
}
