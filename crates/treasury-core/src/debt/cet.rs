//! Total effective cost (CET) of a loan, by Resolução CMN nº 4.881/2020 (docs/design/loan-cet.md):
//!
//! ```text
//! Σ (j = 1..N) FCj / (1 + CET)^((dj − d0) / 365) − FC0 = 0
//! ```
//!
//! - `FC0`: the credit granted minus the costs paid upfront (IOF, fees, registry, single-premium
//!   insurance), on the contract date. Financed or paid apart, the borrower ends day zero with
//!   the same money, so the same subtraction serves both cases.
//! - `FCj`: each installment of the contract as signed (amortization, interest, insurance and
//!   fees) on its due date. Extra payments are not part of the contract.
//! - Floating references (TR, IPCA, CDI…) stay out (art. 5º): the schedule is rebuilt with the
//!   index at zero, so only the fixed rate (or the spread) and the charges count.
//! - Effective annual rate, shown with two decimals (ABNT NBR 5891: a trailing 5 goes to the even
//!   digit).
//!
//! The same flows give the US APR of Regulation Z (12 CFR 1026, Appendix J): a monthly
//! unit-period rate `i` solved with whole months counted back from each payment and the odd days
//! over 30, times 12 (a nominal rate).
//!
//! The realized CET uses what was actually paid: installments, extra payments and balance drops
//! with no payment recorded, plus the balance still owed right after the last installment paid.

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Installment, LoanIndexer, SpreadMode, schedule};
use crate::error::Result;
use crate::finance::returns::{Flow, xirr};
use crate::time::{add_months, fmt_date, parse_date};

/// Costs the borrower pays on day zero, in cents.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpfrontCosts {
    #[ts(type = "number")]
    pub iof: i64,
    /// Registration, appraisal, registry and other fees charged once.
    #[ts(type = "number")]
    pub fees: i64,
    /// Insurance paid once (single-premium credit insurance).
    #[ts(type = "number")]
    pub insurance: i64,
}

impl UpfrontCosts {
    pub fn total(&self) -> i64 {
        self.iof + self.fees + self.insurance
    }
}

/// The CET of the contract as signed, with the parts of the calculation, for the screen and
/// the assistants to show next to the number the bank informed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CetReport {
    /// Effective annual rate as a fraction (0.4649 = 46.49% a year), not rounded.
    pub annual: f64,
    /// Annual rate in %, two decimals (NBR 5891), as the bank must disclose it.
    pub annual_pct: String,
    /// Equivalent monthly rate in %, two decimals.
    pub monthly_pct: String,
    /// US APR (Regulation Z): monthly unit-period rate × 12, as a fraction.
    pub us_apr: Option<f64>,
    /// Credit granted (the contract's principal).
    #[ts(type = "number")]
    pub principal: i64,
    pub upfront: UpfrontCosts,
    /// `FC0`: what reached the borrower (principal − upfront costs).
    #[ts(type = "number")]
    pub released: i64,
    /// Sums of the installments of the contract as signed.
    #[ts(type = "number")]
    pub amortization: i64,
    #[ts(type = "number")]
    pub interest: i64,
    #[ts(type = "number")]
    pub insurance: i64,
    #[ts(type = "number")]
    pub fees: i64,
    /// Everything the borrower pays: installments plus upfront costs.
    #[ts(type = "number")]
    pub total_paid: i64,
    pub installments: u32,
    /// `d0` and the last `dj`, and `N` in calendar days.
    pub release_date: String,
    pub last_date: String,
    #[ts(type = "number")]
    pub days: i64,
    /// Floating reference left out by art. 5º (TR, IPCA, CDI…), if any.
    pub index_excluded: Option<String>,
    /// The contracted CET the user typed minus this one, in percentage points.
    pub contracted_gap_pp: Option<f64>,
}

/// Balance drop with no payment recorded, counted as money the borrower paid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnrecordedDrop {
    pub number: u32,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
}

/// Drops below this (R$ 50,00) are rounding in statements, not payments.
const DROP_TOLERANCE: i64 = 5_000;

/// Rounds a percentage to two decimals with the rule of NBR 5891.
pub fn round_pct(rate: f64) -> String {
    Decimal::from_f64(rate * 100.0)
        .map(|d| {
            d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
                .to_string()
        })
        .unwrap_or_default()
}

fn ensure_2dp(s: String) -> String {
    match s.split_once('.') {
        Some((i, f)) => format!("{i}.{f:0<2}"),
        None if s.is_empty() => s,
        None => format!("{s}.00"),
    }
}

/// Art. 4º: the effective annual rate that makes the installments worth `released` on
/// `release`. `None` when it does not converge (no installments, or costs above the credit).
pub fn rate(released: i64, release: NaiveDate, flows: &[(NaiveDate, i64)]) -> Option<f64> {
    let mut f = Vec::with_capacity(flows.len() + 1);
    f.push(Flow {
        date: release,
        amount: -(released as f64),
    });
    f.extend(flows.iter().map(|(d, a)| Flow {
        date: *d,
        amount: *a as f64,
    }));
    xirr(&f)
}

/// Whole months counted back from `to` while they stay on or after `from`, and the days left
/// at the start (Appendix J, (b)(5)(ii)).
fn months_and_days(from: NaiveDate, to: NaiveDate) -> (u32, i64) {
    use chrono::Datelike;
    // Start from the calendar difference and adjust by one at most, instead of stepping back
    // one month at a time (quadratic over a 30-year schedule).
    let ym = |d: NaiveDate| d.year() * 12 + d.month() as i32;
    let mut t = (ym(to) - ym(from)).max(0) as u32;
    while t > 0 && add_months(to, -(t as i32)) < from {
        t -= 1;
    }
    while add_months(to, -(t as i32 + 1)) >= from {
        t += 1;
    }
    (t, (add_months(to, -(t as i32)) - from).num_days())
}

/// US APR (Regulation Z, Appendix J) for monthly payments: solves the monthly unit-period
/// rate `i` in `released = Σ Pj / ((1 + fj·i)(1 + i)^tj)` and returns `12 × i`.
pub fn us_apr(released: i64, release: NaiveDate, flows: &[(NaiveDate, i64)]) -> Option<f64> {
    if released <= 0 || flows.is_empty() {
        return None;
    }
    let periods: Vec<(f64, f64, f64)> = flows
        .iter()
        .map(|(d, a)| {
            let (t, days) = months_and_days(release, *d);
            (t as f64, days as f64 / 30.0, *a as f64)
        })
        .collect();
    let pv = |i: f64| -> f64 {
        periods
            .iter()
            .map(|(t, f, a)| a / ((1.0 + f * i) * (1.0 + i).powf(*t)))
            .sum()
    };
    let target = released as f64;
    let (mut lo, mut hi) = (-0.99_f64, 10.0_f64);
    if (pv(lo) - target).signum() == (pv(hi) - target).signum() {
        return None;
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        // The present value falls as the rate rises.
        if pv(mid) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(12.0 * (lo + hi) / 2.0)
}

/// Schedule of the contract as signed, with the floating reference at zero (art. 5º).
fn contract_rows(conn: &Connection, id: &str) -> Result<(Vec<schedule::Row>, NaiveDate, i64)> {
    let l = super::load_row(conn, id)?;
    let daily = l.indexer.daily_series().is_some();
    let periods = if daily {
        // Only the spread over the index: `(1 + s)^(du/252)` or `1 + s × dc/360`.
        let s = l.annual_rate / Decimal::ONE_HUNDRED;
        let mut start = l.contract_date;
        super::postfixed::due_dates(l.first_due, l.term)
            .into_iter()
            .map(|due| {
                let du = crate::finance::calendar::business_days_between(start, due).max(0);
                let dc = (due - start).num_days().max(0);
                let factor = match l.spread_mode {
                    SpreadMode::Exp252 => crate::finance::pow_dec(
                        Decimal::ONE + s,
                        Decimal::from(du) / Decimal::from(252),
                    ),
                    SpreadMode::Lin360 => Decimal::ONE + s * Decimal::from(dc) / Decimal::from(360),
                }
                .round_dp(9);
                start = due;
                schedule::Period {
                    due,
                    factor,
                    business_days: du as u32,
                    estimated: false,
                }
            })
            .collect()
    } else {
        super::interest_periods(&l, l.term)
    };
    let p = schedule::Params {
        principal: l.principal,
        annual_rate_pct: l.annual_rate,
        system: super::sched_system(l.system),
        term: l.term,
        grace: l.grace,
        first_due: l.first_due,
        insurance: l.insurance,
        fees: l.fees,
        mip_rate: l.mip_rate,
        mip_fixed: l.mip_fixed,
        dfi: l.dfi,
        index_factors: vec![],
        extras: vec![],
        periods,
    };
    Ok((schedule::build(&p), l.contract_date, l.principal))
}

/// The CET of the contract as signed. `contracted` is the CET the user typed (% a year).
pub fn contract(
    conn: &Connection,
    id: &str,
    indexer: LoanIndexer,
    upfront: UpfrontCosts,
    contracted: Option<&str>,
) -> Result<Option<CetReport>> {
    let (rows, release, principal) = contract_rows(conn, id)?;
    let released = principal - upfront.total();
    let flows: Vec<(NaiveDate, i64)> = rows.iter().map(|r| (r.due_date, r.total)).collect();
    let Some(last) = rows.last() else {
        return Ok(None);
    };
    let Some(annual) = rate(released, release, &flows) else {
        return Ok(None);
    };
    let sum = |f: fn(&schedule::Row) -> i64| rows.iter().map(f).sum::<i64>();
    let total: i64 = sum(|r| r.total);
    let monthly = (1.0 + annual).powf(1.0 / 12.0) - 1.0;
    let annual_pct = ensure_2dp(round_pct(annual));
    let contracted_gap_pp = contracted
        .and_then(|c| c.trim().replace(',', ".").parse::<f64>().ok())
        .and_then(|c| annual_pct.parse::<f64>().ok().map(|a| c - a));
    Ok(Some(CetReport {
        annual,
        annual_pct,
        monthly_pct: ensure_2dp(round_pct(monthly)),
        us_apr: us_apr(released, release, &flows),
        principal,
        upfront,
        released,
        amortization: sum(|r| r.amortization),
        interest: sum(|r| r.interest),
        insurance: sum(|r| r.insurance),
        fees: sum(|r| r.fees),
        total_paid: total + upfront.total(),
        installments: rows.len() as u32,
        release_date: fmt_date(release),
        last_date: fmt_date(last.due_date),
        days: (last.due_date - release).num_days(),
        index_excluded: (indexer != LoanIndexer::None).then(|| indexer.label().to_string()),
        contracted_gap_pp,
    }))
}

/// Realized CET: `released` on the contract date; each paid installment (amount paid, or the
/// installment's total) plus its extra payment and any unexplained balance drop on the date it
/// was paid; and, while something is owed, the balance after the last installment paid on its
/// due date. Returns the rate and the drops it counted.
pub fn realized(
    released: i64,
    principal: i64,
    contract_date: NaiveDate,
    inst: &[Installment],
) -> Result<(Option<f64>, Vec<UnrecordedDrop>)> {
    let mut flows = vec![];
    let mut drops = vec![];
    // Number and balance of the installment before; drops are only checked between
    // consecutive installments (a gap means installments not marked paid, not a payment).
    let mut prev = (0u32, principal);
    let mut last: Option<(NaiveDate, i64)> = None;
    for i in inst.iter().filter(|i| i.paid) {
        let due = parse_date(&i.due_date)?;
        let date = i
            .paid_date
            .as_deref()
            .map(parse_date)
            .transpose()?
            .unwrap_or(due);
        let drop = if prev.0 + 1 == i.number {
            prev.1 + i.adjustment - i.amortization - i.extra_payment - i.balance_after
        } else {
            0
        };
        let drop = if drop > DROP_TOLERANCE {
            drops.push(UnrecordedDrop {
                number: i.number,
                date: fmt_date(date),
                amount: drop,
            });
            drop
        } else {
            0
        };
        flows.push((
            date,
            i.paid_amount.unwrap_or(i.total) + i.extra_payment + drop,
        ));
        prev = (i.number, i.balance_after);
        last = Some((due, i.balance_after));
    }
    let Some((due, balance)) = last else {
        return Ok((None, drops));
    };
    if balance > 0 {
        flows.push((due.max(contract_date), balance));
    }
    Ok((rate(released, contract_date, &flows), drops))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    /// 12 monthly installments of R$ 1.019,68 from 10/04/2026 (made-up loan of the design doc).
    fn flows() -> Vec<(NaiveDate, i64)> {
        (0..12)
            .map(|k| (add_months(d("2026-04-10"), k), 101_968))
            .collect()
    }

    fn paid(number: u32, due: &str, amort: i64, interest: i64, balance_after: i64) -> Installment {
        Installment {
            id: format!("i{number}"),
            number,
            version: 1,
            version_number: number,
            due_date: due.into(),
            amortization: amort,
            interest,
            adjustment: 0,
            insurance: 0,
            insurance_mip: 0,
            insurance_dfi: 0,
            fees: 0,
            total: amort + interest,
            balance_after,
            extra_payment: 0,
            estimated: false,
            paid: true,
            paid_date: Some(due.into()),
            paid_amount: None,
            txn_id: None,
            txn_date: None,
            txn_description: None,
            txn_account_id: None,
            txn_account_name: None,
            overdue: false,
            index_factor: None,
            business_days: None,
        }
    }

    #[test]
    fn a_balance_drop_with_no_payment_counts_as_paid() {
        // 1% a month on R$ 10.000,00; in the 2nd month the balance falls R$ 3.000,00 more than
        // the amortization (an extra payment that was never recorded).
        let inst = vec![
            paid(1, "2026-02-10", 100_000, 10_000, 900_000),
            paid(2, "2026-03-10", 100_000, 9_000, 500_000),
            paid(3, "2026-04-10", 100_000, 5_000, 400_000),
        ];
        let (r, drops) = realized(1_000_000, 1_000_000, d("2026-01-10"), &inst).unwrap();
        assert_eq!(
            drops,
            vec![UnrecordedDrop {
                number: 2,
                date: "2026-03-10".into(),
                amount: 300_000
            }]
        );
        let r = r.unwrap();
        assert!((r - 0.1268).abs() < 0.003, "{r}");
    }

    #[test]
    fn the_open_balance_sits_on_the_last_installment_paid() {
        // Paid on the due date with nothing accrued after it: the rate is the contract's,
        // whatever today is.
        let inst = vec![
            paid(1, "2026-04-10", 100_000, 10_000, 900_000),
            paid(2, "2026-05-10", 100_000, 9_000, 800_000),
        ];
        let (r, drops) = realized(1_000_000, 1_000_000, d("2026-03-10"), &inst).unwrap();
        assert!(drops.is_empty());
        let r = r.unwrap();
        assert!((r - 0.1268).abs() < 0.003, "{r}");
    }

    #[test]
    fn a_gap_in_the_paid_installments_is_not_a_drop() {
        let inst = vec![paid(3, "2026-04-10", 100_000, 8_000, 700_000)];
        let (_, drops) = realized(1_000_000, 1_000_000, d("2026-01-10"), &inst).unwrap();
        assert!(drops.is_empty());
    }

    #[test]
    fn upfront_costs_come_out_of_the_released_amount() {
        let financed = rate(1_085_000, d("2026-03-10"), &flows()).unwrap();
        let released = rate(1_000_000, d("2026-03-10"), &flows()).unwrap();
        assert_eq!(round_pct(financed), "25.19");
        assert_eq!(round_pct(released), "46.49");
    }

    #[test]
    fn a_short_first_period_raises_the_cet() {
        let r = rate(1_000_000, d("2026-03-25"), &flows()).unwrap();
        assert_eq!(round_pct(r), "51.45");
    }

    #[test]
    fn us_apr_is_the_monthly_rate_times_twelve() {
        let apr = us_apr(1_000_000, d("2026-03-10"), &flows()).unwrap();
        assert!((apr - 0.3900).abs() < 0.0001, "{apr}");
        // A loan with no costs: the APR is the contract's nominal rate (1.9% a month × 12).
        let apr = us_apr(1_085_000, d("2026-03-10"), &flows()).unwrap();
        assert!((apr - 0.228).abs() < 0.0002, "{apr}");
    }

    #[test]
    fn odd_days_count_over_thirty() {
        assert_eq!(months_and_days(d("1978-07-01"), d("1978-09-05")), (2, 4));
        assert_eq!(months_and_days(d("2026-03-10"), d("2026-04-10")), (1, 0));
    }

    #[test]
    fn rounding_follows_nbr_5891() {
        assert_eq!(round_pct(0.123_449), "12.34");
        assert_eq!(round_pct(0.123_451), "12.35");
        assert_eq!(ensure_2dp(round_pct(0.1)), "10.00");
        assert_eq!(ensure_2dp("7.5".into()), "7.50");
    }
}
