//! Result of a real estate fund (FII) line by line, from the CVM quarterly report already
//! collected (`inf_trimestral_fii_resultado_contabil_financeiro`, cash-basis columns): rent,
//! interest on securities, liquidity, sales of assets, expenses and fees, the net result and the
//! distributions declared, for each quarter, the last 12 months and the last quarter. These are
//! the same lines that feed the income coverage of the health score, so the two always agree.
//! Amounts in cents (BRL); per-unit values as decimal strings.

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::data::{self, Cache, SubjectRef};
use super::{Fact, PeriodType, cvm, items};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::time::{add_months, fmt_date};

/// Quarters returned (two years).
pub const MAX_QUARTERS: usize = 8;
/// Share of the 12-month result coming from sales of assets that gets a note.
const MATERIAL_SALES: f64 = 0.10;

/// Lines of one period. Amounts in cents; `None` when the report left the line empty.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FiiResultLines {
    /// Rent from properties held for income.
    #[ts(type = "number | null")]
    pub rent_revenue: Option<i64>,
    /// Interest and inflation adjustment on securities (CRI, LCI and others).
    #[ts(type = "number | null")]
    pub securities_income: Option<i64>,
    /// Net result of the cash investments.
    #[ts(type = "number | null")]
    pub liquidity_income: Option<i64>,
    /// Result of selling properties, inventory, securities and cash investments.
    #[ts(type = "number | null")]
    pub sales_result: Option<i64>,
    /// Everything else in the result (maintenance, other property and securities lines,
    /// derivatives): the result minus the lines above and the expenses.
    #[ts(type = "number | null")]
    pub other: Option<i64>,
    /// Fund expenses (negative), fees included.
    #[ts(type = "number | null")]
    pub expenses: Option<i64>,
    /// Administration and performance fees (negative), part of `expenses`.
    #[ts(type = "number | null")]
    pub fees_paid: Option<i64>,
    /// Net financial result (cash basis), sales included.
    #[ts(type = "number | null")]
    pub result: Option<i64>,
    /// Net financial result without the sales of assets.
    #[ts(type = "number | null")]
    pub result_ex_sales: Option<i64>,
    /// Distributions declared for the period.
    #[ts(type = "number | null")]
    pub declared_income: Option<i64>,
    /// Per unit (units of the latest monthly report), decimal strings.
    pub result_per_unit: Option<String>,
    pub result_ex_sales_per_unit: Option<String>,
    pub declared_per_unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FiiResultQuarter {
    /// Quarter end (reference date of the report), `YYYY-MM-DD`.
    pub end: String,
    pub start: String,
    pub lines: FiiResultLines,
    /// "Informe trimestral FII <date> v<version>".
    pub document: String,
    /// Filing date at the CVM.
    pub filed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FiiResultPeriod {
    pub from: String,
    pub to: String,
    #[ts(type = "number")]
    pub quarters: u32,
    pub lines: FiiResultLines,
}

/// Result ÷ distributions declared, as the score computes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FiiResultCoverage {
    /// Same value as the score's `income_coverage`.
    pub value: f64,
    /// Without the sales of assets in the same quarters.
    pub ex_sales: Option<f64>,
    pub period_end: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FiiMonthAmount {
    #[ts(type = "number")]
    pub amount: i64,
    /// Month of the report, `YYYY-MM-DD` (last day).
    pub month: String,
    pub document: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FiiResultView {
    pub ticker: Option<String>,
    pub name: String,
    /// Fund CNPJ (digits).
    pub issuer_id: String,
    pub currency: String,
    /// Where the lines come from, in words.
    pub source: String,
    /// Open data file of the latest quarter at the CVM.
    pub source_url: Option<String>,
    /// Reference date of the latest quarter.
    pub data_date: Option<String>,
    /// Newest first.
    pub quarters: Vec<FiiResultQuarter>,
    pub last_quarter: Option<FiiResultQuarter>,
    /// The last four consecutive quarters.
    pub last_12_months: Option<FiiResultPeriod>,
    pub coverage: Option<FiiResultCoverage>,
    /// Units issued (latest monthly report), decimal string.
    pub units: Option<String>,
    /// Obligations for property acquisitions (latest monthly report).
    pub acquisition_obligations: Option<FiiMonthAmount>,
    pub notes: Vec<String>,
}

fn cents(v: Decimal) -> i64 {
    (v * Decimal::from(100)).round().to_i64().unwrap_or(0)
}

fn per_unit(v: Option<i64>, units: Option<Decimal>) -> Option<String> {
    let u = units.filter(|u| *u > Decimal::ZERO)?;
    Some(
        (Decimal::from(v?) / Decimal::from(100) / u)
            .round_dp(6)
            .normalize()
            .to_string(),
    )
}

const LINE_ITEMS: [&str; 8] = [
    items::FII_RENT_REVENUE,
    items::FII_SECURITIES_INCOME,
    items::FII_LIQUIDITY_INCOME,
    items::FII_SALES_RESULT,
    items::FII_EXPENSES,
    items::FII_FEES_PAID,
    items::FII_RECURRING_RESULT,
    items::FII_DECLARED_INCOME,
];

/// Lines of one quarter from the facts (`None` per line when absent).
fn quarter_lines(facts: &[&Fact], end: NaiveDate) -> [Option<i64>; 8] {
    LINE_ITEMS.map(|item| {
        facts
            .iter()
            .find(|f| f.item == item && f.end == end)
            .map(|f| cents(f.value))
    })
}

/// Fills the derived lines (other, result without sales, per unit).
fn lines_of(raw: [Option<i64>; 8], units: Option<Decimal>) -> FiiResultLines {
    let [rent, sec, liq, sales, exp, fees, result, declared] = raw;
    let other = result.map(|r| {
        r - rent.unwrap_or(0)
            - sec.unwrap_or(0)
            - liq.unwrap_or(0)
            - sales.unwrap_or(0)
            - exp.unwrap_or(0)
    });
    let ex_sales = result.map(|r| r - sales.unwrap_or(0));
    FiiResultLines {
        rent_revenue: rent,
        securities_income: sec,
        liquidity_income: liq,
        sales_result: sales,
        other,
        expenses: exp,
        fees_paid: fees,
        result,
        result_ex_sales: ex_sales,
        declared_income: declared,
        result_per_unit: per_unit(result, units),
        result_ex_sales_per_unit: per_unit(ex_sales, units),
        declared_per_unit: per_unit(declared, units),
    }
}

fn sum(rows: &[[Option<i64>; 8]]) -> [Option<i64>; 8] {
    std::array::from_fn(|i| {
        rows.iter()
            .filter_map(|r| r[i])
            .reduce(|a, b| a + b)
            .filter(|_| {
                // A line counts in the sum only when every quarter has it, except the sales
                // and fees, which are often left empty when there were none.
                i == 3 || i == 5 || rows.iter().all(|r| r[i].is_some())
            })
    })
}

fn quarter_start(end: NaiveDate) -> NaiveDate {
    use chrono::Datelike;
    add_months(end.with_day(1).expect("day 1"), -2)
}

/// Builds the view from the fund's facts (tested without a database).
pub fn build(
    facts: &[Fact],
    filed: &dyn Fn(NaiveDate) -> Option<String>,
    today: NaiveDate,
) -> FiiResultView {
    let q: Vec<&Fact> = facts
        .iter()
        .filter(|f| f.period_type == PeriodType::Quarter && LINE_ITEMS.contains(&f.item.as_str()))
        .collect();
    let mut ends: Vec<NaiveDate> = q.iter().map(|f| f.end).collect();
    ends.sort();
    ends.dedup();
    ends.reverse();
    let latest_month = |item: &str| {
        facts
            .iter()
            .filter(|f| f.item == item && f.period_type == PeriodType::Month)
            .max_by_key(|f| f.end)
    };
    let units = latest_month(items::FII_SHARES).map(|f| f.value);
    let quarter = |end: NaiveDate| {
        let doc = q
            .iter()
            .find(|f| f.end == end)
            .map(|f| f.document.clone())
            .unwrap_or_default();
        FiiResultQuarter {
            end: fmt_date(end),
            start: fmt_date(quarter_start(end)),
            lines: lines_of(quarter_lines(&q, end), units),
            document: doc,
            filed_at: filed(end),
        }
    };
    let quarters: Vec<FiiResultQuarter> = ends
        .iter()
        .take(MAX_QUARTERS)
        .map(|e| quarter(*e))
        .collect();
    // Last four quarters in a row.
    let ttm = (ends.len() >= 4
        && ends[..4]
            .windows(2)
            .all(|w| add_months(quarter_start(w[0]), -3) == quarter_start(w[1])))
    .then(|| {
        let raw: Vec<[Option<i64>; 8]> = ends[..4].iter().map(|e| quarter_lines(&q, *e)).collect();
        FiiResultPeriod {
            from: fmt_date(quarter_start(ends[3])),
            to: fmt_date(ends[0]),
            quarters: 4,
            lines: lines_of(sum(&raw), units),
        }
    });
    // The score's coverage, computed by the score itself.
    let none = |_: NaiveDate, _: NaiveDate| None;
    let metrics = crate::score::fii::fii_metrics(&crate::score::fii::FiiInput {
        facts,
        today,
        pvp: None,
        dy: None,
        liquidity: None,
        cdi_12m: None,
        inflation: &none,
    })
    .metrics;
    let coverage = metrics.get("income_coverage").and_then(|m| {
        let value = m.value?;
        Some(FiiResultCoverage {
            value,
            ex_sales: sales_free_coverage(facts),
            period_end: fmt_date(m.period_end?),
        })
    });
    let acquisition_obligations =
        latest_month(items::FII_ACQ_OBLIGATIONS).map(|f| FiiMonthAmount {
            amount: cents(f.value),
            month: fmt_date(f.end),
            document: f.document.clone(),
        });
    let mut notes = vec![t("nucleo-pesquisa-fii-resultado-nota-caixa")];
    if let Some(p) = &ttm
        && let (Some(r), Some(s)) = (p.lines.result, p.lines.sales_result)
        && r > 0
        && s as f64 / r as f64 > MATERIAL_SALES
    {
        notes.push(t_args(
            "nucleo-pesquisa-fii-resultado-nota-vendas",
            &args(&[("pct", format!("{:.0}%", s as f64 / r as f64 * 100.0).into())]),
        ));
    }
    if ttm.is_none() {
        notes.push(t("nucleo-pesquisa-fii-resultado-nota-poucos"));
    }
    notes.push(t("nucleo-pesquisa-fii-resultado-nota-fundamentus"));
    use chrono::Datelike;
    FiiResultView {
        ticker: None,
        name: String::new(),
        issuer_id: String::new(),
        currency: "BRL".into(),
        source: t("nucleo-pesquisa-fii-resultado-fonte"),
        source_url: ends.first().map(|e| cvm::fii_quarterly_url(e.year())),
        data_date: ends.first().map(|e| fmt_date(*e)),
        last_quarter: quarters.first().cloned(),
        quarters,
        last_12_months: ttm,
        coverage,
        units: units.map(|u| u.normalize().to_string()),
        acquisition_obligations,
        notes,
    }
}

/// Coverage without the sales of assets, over the same quarters the score uses (the latest
/// four of the result and of the declared distributions).
pub fn sales_free_coverage(facts: &[Fact]) -> Option<f64> {
    let latest4 = |item: &str| -> Vec<&Fact> {
        let mut v: Vec<&Fact> = facts
            .iter()
            .filter(|f| f.item == item && f.period_type == PeriodType::Quarter)
            .collect();
        v.sort_by_key(|f| std::cmp::Reverse(f.end));
        v.truncate(4);
        v
    };
    let (rec, dec) = (
        latest4(items::FII_RECURRING_RESULT),
        latest4(items::FII_DECLARED_INCOME),
    );
    if rec.len() < 4 || dec.len() < 4 {
        return None;
    }
    let sales: Decimal = rec
        .iter()
        .filter_map(|r| {
            facts
                .iter()
                .find(|f| f.item == items::FII_SALES_RESULT && f.end == r.end)
                .map(|f| f.value)
        })
        .sum();
    let r: Decimal = rec.iter().map(|f| f.value).sum::<Decimal>() - sales;
    let d: Decimal = dec.iter().map(|f| f.value).sum();
    (d > Decimal::ZERO).then(|| super::to_f64(r / d))
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct FiiResultArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
}

/// `research.fii_result`.
pub fn view(conn: &Connection, a: &FiiResultArgs, today: NaiveDate) -> Result<FiiResultView> {
    let d = data::load(
        conn,
        &SubjectRef {
            asset_id: a.asset_id.clone(),
            ticker: a.ticker.clone(),
        },
        today,
        &mut Cache::default(),
    )?;
    if !d.subject.is_fii() {
        return Err(Error::invalid(t("nucleo-pesquisa-fii-resultado-so-fii")));
    }
    let Some(issuer) = d.subject.issuer_id.clone() else {
        return Err(Error::not_found(t(
            "nucleo-pesquisa-fii-resultado-sem-emissor",
        )));
    };
    let mut stmt = conn.prepare(
        "SELECT reference, filed_at FROM research_filing WHERE issuer_id = ?1 AND kind = 'fii_trimestral'",
    )?;
    let filings: Vec<(String, String)> = stmt
        .query_map([&issuer], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let filed = |end: NaiveDate| {
        let e = fmt_date(end);
        filings
            .iter()
            .find(|(r, _)| *r == e)
            .map(|(_, f)| f.clone())
    };
    let mut v = build(&d.facts, &filed, today);
    v.ticker = d.subject.ticker.clone();
    v.name = d
        .subject
        .issuer_name
        .clone()
        .unwrap_or(d.subject.name.clone());
    v.issuer_id = issuer;
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    fn fact(item: &str, pt: PeriodType, end: &str, v: Decimal) -> Fact {
        Fact {
            item: item.into(),
            period_type: pt,
            start: None,
            end: d(end),
            value: v,
            currency: "BRL".into(),
            source: "cvm_fii_trimestral".into(),
            document: format!("Informe trimestral FII {end} v1"),
            priority: 2,
        }
    }

    /// Four quarters of an invented fund: rent 1000, interest 100, expenses −200, result 900,
    /// declared 850; the last quarter also sold a property with a gain of 400.
    fn facts() -> Vec<Fact> {
        let mut v = vec![];
        for end in ["2025-12-31", "2026-03-31", "2026-06-30", "2026-09-30"] {
            let sale = end == "2026-09-30";
            let q = |item: &str, x: Decimal| fact(item, PeriodType::Quarter, end, x);
            v.push(q(items::FII_RENT_REVENUE, dec!(1000)));
            v.push(q(items::FII_SECURITIES_INCOME, dec!(100)));
            v.push(q(items::FII_EXPENSES, dec!(-200)));
            v.push(q(items::FII_FEES_PAID, dec!(-120.5)));
            v.push(q(
                items::FII_RECURRING_RESULT,
                if sale { dec!(1300) } else { dec!(900) },
            ));
            v.push(q(items::FII_DECLARED_INCOME, dec!(850)));
            if sale {
                v.push(q(items::FII_SALES_RESULT, dec!(400)));
            }
        }
        let mut m = fact(
            items::FII_SHARES,
            PeriodType::Month,
            "2026-10-31",
            dec!(1000),
        );
        m.document = "Informe mensal FII 2026-10 v1".into();
        v.push(m);
        v.push(fact(
            items::FII_ACQ_OBLIGATIONS,
            PeriodType::Month,
            "2026-10-31",
            dec!(12345.678),
        ));
        v
    }

    #[test]
    fn lines_by_quarter_and_for_twelve_months() {
        let v = build(
            &facts(),
            &|e| Some(fmt_date(e + chrono::Duration::days(45))),
            d("2026-11-24"),
        );
        assert_eq!(v.quarters.len(), 4);
        let last = v.last_quarter.as_ref().unwrap();
        assert_eq!(
            (last.start.as_str(), last.end.as_str()),
            ("2026-07-01", "2026-09-30")
        );
        assert_eq!(last.filed_at.as_deref(), Some("2026-11-14"));
        assert_eq!(last.document, "Informe trimestral FII 2026-09-30 v1");
        let l = &last.lines;
        assert_eq!(l.result, Some(130_000));
        assert_eq!(l.sales_result, Some(40_000));
        assert_eq!(l.result_ex_sales, Some(90_000));
        assert_eq!(l.other, Some(0), "1300 − 1000 − 100 − 400 + 200");
        assert_eq!(l.fees_paid, Some(-12_050));
        assert_eq!(l.liquidity_income, None);
        assert_eq!(l.result_per_unit.as_deref(), Some("1.3"));
        assert_eq!(l.declared_per_unit.as_deref(), Some("0.85"));
        let y = v.last_12_months.as_ref().unwrap();
        assert_eq!(
            (y.from.as_str(), y.to.as_str()),
            ("2025-10-01", "2026-09-30")
        );
        assert_eq!(y.lines.result, Some(400_000));
        assert_eq!(y.lines.sales_result, Some(40_000), "only one quarter sold");
        assert_eq!(y.lines.result_ex_sales, Some(360_000));
        assert_eq!(y.lines.declared_income, Some(340_000));
        assert_eq!(y.lines.rent_revenue, Some(400_000));
        assert_eq!(y.lines.liquidity_income, None, "never reported");
        // Same coverage as the score: 4000 ÷ 3400; without the sale 3600 ÷ 3400.
        let c = v.coverage.as_ref().unwrap();
        assert!((c.value - 4000.0 / 3400.0).abs() < 1e-9);
        assert!((c.ex_sales.unwrap() - 3600.0 / 3400.0).abs() < 1e-9);
        assert_eq!(c.period_end, "2026-09-30");
        assert_eq!(
            v.acquisition_obligations
                .as_ref()
                .map(|a| (a.amount, a.month.as_str())),
            Some((1_234_568, "2026-10-31"))
        );
        assert_eq!(v.units.as_deref(), Some("1000"));
        assert_eq!(v.data_date.as_deref(), Some("2026-09-30"));
        assert!(
            v.source_url
                .as_deref()
                .unwrap()
                .ends_with("inf_trimestral_fii_2026.zip")
        );
        // 400 of 4000 = 10%: not above the threshold, no note about sales.
        assert_eq!(v.notes.len(), 2);
    }

    #[test]
    fn a_gap_in_the_quarters_means_no_twelve_months() {
        let f: Vec<Fact> = facts()
            .into_iter()
            .filter(|f| f.end != d("2026-03-31"))
            .collect();
        let v = build(&f, &|_| None, d("2026-11-24"));
        assert_eq!(v.quarters.len(), 3);
        assert!(v.last_12_months.is_none());
        assert!(v.coverage.is_none());
        assert_eq!(v.notes.len(), 3);
    }

    #[test]
    fn material_sales_get_a_note() {
        let mut f = facts();
        for x in f.iter_mut().filter(|x| x.item == items::FII_SALES_RESULT) {
            x.value = dec!(800);
        }
        for x in f
            .iter_mut()
            .filter(|x| x.item == items::FII_RECURRING_RESULT && x.end == d("2026-09-30"))
        {
            x.value = dec!(1700);
        }
        let v = build(&f, &|_| None, d("2026-11-24"));
        assert_eq!(v.notes.len(), 3, "{:?}", v.notes);
        assert!(v.notes[1].contains("18%"), "{}", v.notes[1]);
    }
}
