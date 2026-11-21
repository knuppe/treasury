//! Testes do orçamento com o cenário de referência (`reports::fixture`).

use std::collections::HashMap;

use super::*;
use crate::reports::fixture::{cat, new_db, reference_budget_lines, seed_reference, write};

fn set(db: &mut crate::storage::Db, month: &str, lines: Vec<BudgetLineInput>) {
    write(db, |tx| {
        set_lines(
            tx,
            &SetBudgetArgs {
                month: month.into(),
                lines,
            },
        )
    });
}

fn line(conn: &Connection, s: &BudgetStatus, path: &str) -> Option<BudgetLineStatus> {
    let id = cat(conn, path);
    s.lines.iter().find(|l| l.category_id == id).cloned()
}

#[test]
fn october_planned_vs_actual_with_subcategories() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let lines = reference_budget_lines(t.db.conn());
    set(&mut t.db, "2026-10", lines);
    let c = t.db.conn();
    let s = get(c, "2026-10", Some(Basis::Accrual)).unwrap();
    let l = |p: &str| line(c, &s, p).unwrap_or_else(|| panic!("linha {p}"));
    let v = |p: &str| {
        let x = l(p);
        (
            x.planned,
            x.actual,
            x.remaining,
            x.committed,
            x.has_own_line,
        )
    };
    assert_eq!(v("Moradia"), (350_000, 325_000, 25_000, 10_000, true));
    assert_eq!(v("Moradia › Aluguel"), (0, 300_000, -300_000, 0, false));
    assert_eq!(v("Moradia › Energia"), (0, 25_000, -25_000, 0, false));
    // Só o previsto, sem gasto realizado: aparece pelo comprometido.
    assert_eq!(v("Moradia › Internet"), (0, 0, 0, 10_000, false));
    // Pai sem linha própria soma o previsto dos filhos.
    assert_eq!(v("Alimentação"), (65_000, 62_000, 3_000, 0, false));
    assert_eq!(v("Alimentação › Mercado"), (50_000, 45_000, 5_000, 0, true));
    assert_eq!(
        v("Alimentação › Restaurantes"),
        (15_000, 17_000, -2_000, 0, true)
    );
    assert!((l("Alimentação › Restaurantes").ratio.unwrap() - 17_000.0 / 15_000.0).abs() < 1e-12);
    assert_eq!(v("Transporte"), (30_000, 0, 30_000, 0, true));
    assert_eq!(l("Transporte").ratio, Some(0.0));
    assert_eq!(v("Lazer"), (60_000, 75_000, -15_000, 0, true));
    assert_eq!(v("Lazer › Passeios"), (0, 25_000, -25_000, 0, false));
    // Gasto sem orçamento também aparece.
    assert_eq!(v("Pets"), (0, 15_000, -15_000, 0, false));
    assert_eq!(l("Pets").ratio, None);
    assert_eq!(v("Pets › Ração"), (0, 15_000, -15_000, 0, false));
    assert_eq!(l("Salário").actual, 1_000_000);
    assert_eq!(l("Rendimentos").actual, 20_000);
    assert_eq!(l("Rendimentos › Dividendos").kind, CategoryKind::Income);
    // Sem valores, sem linha.
    assert!(line(c, &s, "Saúde").is_none());
    assert!(
        line(c, &s, "Transporte › Combustível").is_none(),
        "compra de setembro"
    );

    // Pais antes dos filhos; despesas antes das receitas.
    let pos: HashMap<&str, usize> = s
        .lines
        .iter()
        .enumerate()
        .map(|(i, l)| (l.category_path.as_str(), i))
        .collect();
    assert!(pos["Moradia"] < pos["Moradia › Aluguel"]);
    assert!(pos["Alimentação"] < pos["Alimentação › Mercado"]);
    assert!(pos["Pets › Ração"] < pos["Salário"]);

    assert_eq!(
        (
            s.totals.planned_expense,
            s.totals.actual_expense,
            s.totals.planned_income,
            s.totals.actual_income
        ),
        (505_000, 481_000, 0, 1_020_000)
    );
    assert_eq!(s.totals.committed_expense, 10_000);
    assert_eq!(s.totals.uncategorized_expense, 4_000);
    assert_eq!(s.missing_fx, vec!["JPY".to_string()]);
    assert_eq!(s.currency, "BRL");
    assert!((0.0..=1.0).contains(&s.month_elapsed));
    assert_eq!(get(c, "2026-08", None).unwrap().month_elapsed, 1.0);
    assert_eq!(get(c, "2099-01", None).unwrap().month_elapsed, 0.0);
}

#[test]
fn cash_basis_moves_card_purchases_to_due_month() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let c = t.db.conn();
    let s = get(c, "2026-10", Some(Basis::Cash)).unwrap();
    assert_eq!(line(c, &s, "Lazer").unwrap().actual, 25_000);
    assert!(line(c, &s, "Lazer › Viagens").is_none());
    assert_eq!(
        line(c, &s, "Assinaturas › Streaming").unwrap().actual,
        8_000
    );
    assert_eq!(s.totals.actual_expense, 439_000);
    let nov = get(c, "2026-11", Some(Basis::Cash)).unwrap();
    assert_eq!(line(c, &nov, "Lazer › Viagens").unwrap().actual, 50_000);
    assert_eq!(
        line(c, &nov, "Transporte › Combustível").unwrap().actual,
        12_000
    );
    // Sem `basis`, vale o ajuste.
    crate::settings::set(c, "budget_basis", &Basis::Cash).unwrap();
    assert_eq!(get(c, "2026-10", None).unwrap().basis, Basis::Cash);
}

#[test]
fn set_lines_zero_removes_and_validates() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let lines = reference_budget_lines(t.db.conn());
    set(&mut t.db, "2026-10", lines);
    let moradia = cat(t.db.conn(), "Moradia");
    set(
        &mut t.db,
        "2026-10",
        vec![BudgetLineInput {
            category_id: moradia.clone(),
            amount: 0,
        }],
    );
    let c = t.db.conn();
    let s = get(c, "2026-10", Some(Basis::Accrual)).unwrap();
    let m = line(c, &s, "Moradia").unwrap();
    assert_eq!((m.planned, m.has_own_line, m.actual), (0, false, 325_000));
    assert_eq!(s.totals.planned_expense, 155_000);

    let bad = |db: &mut crate::storage::Db, l: BudgetLineInput| {
        crate::changes::record(
            db.conn_mut(),
            &crate::changes::Author::user(),
            "teste",
            |tx| {
                set_lines(
                    tx,
                    &SetBudgetArgs {
                        month: "2026-10".into(),
                        lines: vec![l],
                    },
                )
            },
        )
        .unwrap_err()
    };
    let e = bad(
        &mut t.db,
        BudgetLineInput {
            category_id: moradia.clone(),
            amount: -1,
        },
    );
    assert_eq!(e.code(), "invalid");
    let e = bad(
        &mut t.db,
        BudgetLineInput {
            category_id: "nao-existe".into(),
            amount: 100,
        },
    );
    assert_eq!(e.code(), "not_found");
    // Só zeros num mês sem orçamento: nada é criado.
    set(
        &mut t.db,
        "2027-01",
        vec![BudgetLineInput {
            category_id: moradia,
            amount: 0,
        }],
    );
    assert_eq!(
        list_months(t.db.conn()).unwrap(),
        vec!["2026-10".to_string()]
    );
}

#[test]
fn copy_keeps_or_overwrites_and_skips_archived() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let lines = reference_budget_lines(t.db.conn());
    set(&mut t.db, "2026-10", lines);
    let lazer = cat(t.db.conn(), "Lazer");
    set(
        &mut t.db,
        "2026-11",
        vec![BudgetLineInput {
            category_id: lazer.clone(),
            amount: 99_900,
        }],
    );
    let transporte = cat(t.db.conn(), "Transporte");
    t.db.conn()
        .execute(
            "UPDATE category SET archived = 1 WHERE id = ?1",
            [&transporte],
        )
        .unwrap();
    let args = |overwrite| CopyBudgetArgs {
        from_month: "2026-10".into(),
        to_month: "2026-11".into(),
        overwrite,
    };
    let n = write(&mut t.db, |tx| copy(tx, &args(None)));
    assert_eq!(
        n, 3,
        "Moradia, Mercado e Restaurantes (Lazer já existe, Transporte arquivada)"
    );
    let own = lines_of(t.db.conn(), "2026-11").unwrap();
    assert_eq!(own[&lazer], 99_900);
    assert!(!own.contains_key(&transporte));
    assert_eq!(own[&cat(t.db.conn(), "Moradia")], 350_000);
    write(&mut t.db, |tx| copy(tx, &args(Some(true))));
    assert_eq!(lines_of(t.db.conn(), "2026-11").unwrap()[&lazer], 60_000);
    assert_eq!(
        list_months(t.db.conn()).unwrap(),
        vec!["2026-11".to_string(), "2026-10".to_string()]
    );

    let err = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::user(),
        "teste",
        |tx| {
            copy(
                tx,
                &CopyBudgetArgs {
                    from_month: "2025-01".into(),
                    to_month: "2026-12".into(),
                    overwrite: None,
                },
            )
        },
    )
    .unwrap_err();
    assert_eq!(err.code(), "not_found");
}

#[test]
fn suggest_by_median_ignores_an_outlier_month() {
    assert_eq!(super::median(vec![100, 9_000, 120]), 120);
    assert_eq!(super::median(vec![100, 200]), 150);
    assert_eq!(super::median(vec![]), 0);
    let mut t = new_db();
    seed_reference(&mut t.db);
    let c = t.db.conn();
    // Agosto e setembro: a mediana de dois meses é a média deles (Moradia: 2.900 e 3.000).
    let s = suggest(
        c,
        &SuggestBudgetArgs {
            month: "2026-10".into(),
            months_back: Some(6),
            basis: Some(Basis::Accrual),
            method: Some(SuggestMethod::Median),
        },
    )
    .unwrap();
    let moradia = s
        .iter()
        .find(|l| l.category_id == cat(c, "Moradia"))
        .unwrap();
    assert_eq!(moradia.amount, 295_000);
}

#[test]
fn suggest_rounds_average_up_to_ten_reais() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let c = t.db.conn();
    // Meses completos antes de outubro desde o primeiro lançamento (agosto): agosto e setembro.
    let s = suggest(
        c,
        &SuggestBudgetArgs {
            month: "2026-10".into(),
            months_back: Some(3),
            basis: Some(Basis::Accrual),
            method: None,
        },
    )
    .unwrap();
    let got: Vec<(String, i64)> = s.into_iter().map(|l| (l.category_id, l.amount)).collect();
    assert_eq!(
        got,
        vec![
            // (2.900 + 3.000) ÷ 2
            (cat(c, "Moradia"), 295_000),
            // 812,34 ÷ 2 = 406,17 → 410,00
            (cat(c, "Alimentação"), 41_000),
            (cat(c, "Transporte"), 6_000),
            (cat(c, "Assinaturas"), 4_000),
        ]
    );
    // Sem dados, sem sugestão.
    let empty = new_db();
    assert!(
        suggest(
            empty.db.conn(),
            &SuggestBudgetArgs {
                month: "2026-10".into(),
                months_back: None,
                basis: None,
                method: None,
            }
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn archived_category_with_values_still_shows() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let c = t.db.conn();
    let racao = cat(c, "Pets › Ração");
    c.execute("UPDATE category SET archived = 1 WHERE id = ?1", [&racao])
        .unwrap();
    let s = get(c, "2026-10", Some(Basis::Accrual)).unwrap();
    let l = line(c, &s, "Pets › Ração").unwrap();
    assert!(l.archived);
    assert_eq!(l.actual, 15_000);
}

/// Os erros saem no idioma da instalação, com o mês no formato do idioma.
#[test]
fn errors_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let mut t = new_db();
    set_current(Locale::En);
    let err = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::user(),
        "teste",
        |tx| {
            copy(
                tx,
                &CopyBudgetArgs {
                    from_month: "2025-01".into(),
                    to_month: "2026-12".into(),
                    overwrite: None,
                },
            )
        },
    )
    .unwrap_err()
    .message();
    set_current(Locale::PtBr);
    assert_eq!(err, "not found: budget for Jan 2025");
}
