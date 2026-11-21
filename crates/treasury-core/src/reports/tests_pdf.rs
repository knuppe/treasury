//! Figure 07 · Inflation and the month report in PDF, over the reference scenario.

use rusqlite::params;

use super::fixture::{new_db, seed_reference};
use super::inflation::{InflationArgs, inflation};
use super::pdf_report::render;
use crate::budget::Basis;

/// IPCA of 0.5% per month from Nov 2025 to Aug 2026 (September and October are not out yet).
fn ipca(conn: &rusqlite::Connection) {
    for m in super::engine::months_between("2025-11", "2026-08").unwrap() {
        conn.execute(
            "INSERT INTO index_value (series_id, date, value, estimated, source, fetched_at) VALUES ('ipca', ?1, '0.5', 0, 'teste', 'x')",
            params![format!("{m}-01")],
        )
        .unwrap();
    }
}

fn args() -> InflationArgs {
    InflationArgs {
        months: Some(12),
        end_month: Some("2026-10".into()),
        basis: Some(Basis::Accrual),
    }
}

#[test]
fn inflation_corrects_each_month_to_the_last_one_and_marks_estimates() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    ipca(t.db.conn());
    let r = inflation(t.db.conn(), &args(), Basis::Accrual).unwrap();
    assert_eq!(r.months.len(), 12);
    assert_eq!(r.months.last().unwrap(), "2026-10");
    let last = r.months.len() - 1;
    // The last month is already in today's money.
    assert_eq!(r.real[last], Some(r.nominal[last]));
    // August is worth more today: September and October projected by the average (0.5%), estimated.
    let aug = r.months.iter().position(|m| m == "2026-08").unwrap();
    assert!(r.nominal[aug] > 0);
    let real = r.real[aug].unwrap() as f64;
    let expected = r.nominal[aug] as f64 * 1.005f64.powf(61.0 / 30.4375);
    assert!((real - expected).abs() <= 2.0, "{real} × {expected}");
    assert!(r.estimated[aug]);
    // Cumulative inflation from Nov 2025 to Oct 2026, with the last two months estimated.
    let total = r.inflation.unwrap();
    assert!(total > 0.05 && total < 0.07, "{total}");
    assert!(r.inflation_estimated);
    let ix = r.index.unwrap();
    assert_eq!(ix.series_id, "ipca");
    assert_eq!(ix.last_date.as_deref(), Some("2026-08-01"));
}

#[test]
fn without_index_data_the_correction_is_missing_not_invented() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let r = inflation(t.db.conn(), &args(), Basis::Accrual).unwrap();
    let last = r.months.len() - 1;
    assert_eq!(r.real[last], Some(r.nominal[last]));
    assert!(r.real[..last].iter().all(Option::is_none));
    assert_eq!(r.inflation, None);
}

#[test]
fn the_month_pdf_has_the_summary_categories_and_inflation_in_the_app_language() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    ipca(t.db.conn());
    let bytes = render(t.db.conn(), Some("2026-10"), Some(Basis::Accrual)).unwrap();
    let doc = lopdf::Document::load_mem(&bytes).expect("valid PDF");
    assert!(doc.get_pages().len() >= 2);
    let p1 = doc.extract_text(&[1]).unwrap();
    for s in [
        "Relat",
        "outubro de 2026",
        "Receitas",
        "Para onde foi o dinheiro",
        "Moradia",
        // Spending groups: the four, with the share of income and the target.
        "Para onde foi a renda",
        "ESSENCIAIS",
        "da renda",
        "meta 50%",
        "sem meta",
    ] {
        assert!(p1.contains(s), "missing {s:?} in {p1}");
    }
    let p2 = doc.extract_text(&[2]).unwrap();
    assert!(p2.contains("Infla"), "{p2}");

    crate::i18n::set_current(crate::i18n::Locale::En);
    let bytes = render(t.db.conn(), Some("2026-10"), Some(Basis::Accrual)).unwrap();
    crate::i18n::set_current(crate::i18n::Locale::PtBr);
    let doc = lopdf::Document::load_mem(&bytes).unwrap();
    let p1 = doc.extract_text(&[1]).unwrap();
    assert!(p1.contains("October 2026 report"), "{p1}");
    assert!(p1.contains("Where the money went"), "{p1}");
    assert!(p1.contains("Where the income went"), "{p1}");
    assert!(p1.contains("of income"), "{p1}");
}
