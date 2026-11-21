//! Figura 07 · Inflação e relatório do mês em PDF, sobre o cenário de referência.

use rusqlite::params;

use super::fixture::{new_db, seed_reference};
use super::inflation::{InflationArgs, inflation};
use super::pdf_report::render;
use crate::budget::Basis;

/// IPCA de 0,5% ao mês de nov/2025 a ago/2026 (setembro e outubro ainda não saíram).
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
    // O último mês já está em dinheiro de hoje.
    assert_eq!(r.real[last], Some(r.nominal[last]));
    // Agosto vale mais hoje: setembro e outubro projetados pela média (0,5%), estimados.
    let aug = r.months.iter().position(|m| m == "2026-08").unwrap();
    assert!(r.nominal[aug] > 0);
    let real = r.real[aug].unwrap() as f64;
    let expected = r.nominal[aug] as f64 * 1.005f64.powf(61.0 / 30.4375);
    assert!((real - expected).abs() <= 2.0, "{real} × {expected}");
    assert!(r.estimated[aug]);
    // Inflação acumulada de nov/2025 a out/2026, com os dois últimos meses estimados.
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
    let doc = lopdf::Document::load_mem(&bytes).expect("PDF válido");
    assert!(doc.get_pages().len() >= 2);
    let p1 = doc.extract_text(&[1]).unwrap();
    for s in [
        "Relat",
        "outubro de 2026",
        "Receitas",
        "Para onde foi o dinheiro",
        "Moradia",
        // Grupos de gasto: os quatro, com a parte da renda e a meta.
        "Para onde foi a renda",
        "ESSENCIAIS",
        "da renda",
        "meta 50%",
        "sem meta",
    ] {
        assert!(p1.contains(s), "faltou {s:?} em {p1}");
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
