//! Cards, phase 2: limit use and history, best purchase day, real statement dates, what the
//! printed statement shows and the annual fee (rule, waiver, checks, warnings).

use super::annual_fee::{AnnualFeeInput, FeeIssue, WaiverKind};
use super::limits::LimitChangeInput;
use super::*;
use crate::transactions::testkit::{Fx, d};

fn overview(fx: &Fx, card: &str, today: &str) -> limits::CardOverview {
    limits::overview(fx.conn(), card, d(today)).unwrap()
}

#[test]
fn limit_use_best_day_and_history() {
    let mut fx = Fx::new();
    // Closes on the 3rd, due on the 10th, limit R$ 1,000.
    let card = fx.card("Cartão", 3, 10, Some(100_000));
    fx.txn(&card, "2026-10-05", -40_000, "Mercado");
    let plan = InstallmentInput {
        account_id: card.clone(),
        description: "Geladeira".into(),
        total_amount: 40_000,
        installments: 4,
        purchase_date: "2026-10-06".into(),
        category_id: None,
        member_id: None,
        tags: None,
    };
    fx.write(|tx| create_installments(tx, &plan, d("2026-10-06")))
        .unwrap();
    let o = overview(&fx, &card, "2026-10-06");
    assert_eq!(o.used, 80_000, "purchase + the whole installment plan");
    assert_eq!(
        o.committed_installments, 30_000,
        "three future installments"
    );
    assert_eq!(o.available, Some(20_000));
    assert_eq!(o.utilization_bp, Some(8_000));
    // Open statement closes on Nov 3: best day Nov 4, which is due on Dec 10.
    assert_eq!(o.best_purchase_date, "2026-11-04");
    assert_eq!(o.best_purchase_due_date, "2026-12-10");
    assert_eq!(o.best_purchase_days, 36);
    assert_eq!(
        o.open_statement
            .as_ref()
            .map(|s| s.reference_month.as_str()),
        Some("2026-11")
    );

    // The history starts with the registered limit and follows the change.
    assert_eq!(o.limit_history.len(), 1);
    fx.write(|tx| {
        limits::add_change(
            tx,
            &LimitChangeInput {
                account_id: card.clone(),
                date: "2026-10-20".into(),
                credit_limit: 200_000,
            },
        )
    })
    .unwrap();
    let o = overview(&fx, &card, "2026-10-21");
    assert_eq!(o.credit_limit, Some(200_000), "the most recent record wins");
    assert_eq!(o.utilization_bp, Some(4_000));
    assert_eq!(
        limits::limit_at(fx.conn(), &card, "2026-10-10").unwrap(),
        Some(100_000)
    );
    // The limit printed on a statement comes in as imported.
    let st = statement_for(fx.conn(), &card, "2026-10-05")
        .unwrap()
        .unwrap();
    fx.write(|tx| {
        apply_printed(
            tx,
            &st,
            &Printed {
                due_date: Some("2026-11-11".into()),
                minimum: Some(12_000),
                credit_limit: Some(150_000),
                previous_balance: Some(0),
            },
        )
    })
    .unwrap();
    let v = statement_view(fx.conn(), &st, d("2026-10-21")).unwrap();
    assert_eq!(v.due_date, "2026-11-11");
    assert_eq!(v.dates_source, DatesSource::Import);
    assert_eq!(
        (v.informed_minimum, v.informed_limit),
        (Some(12_000), Some(150_000))
    );
    let o = overview(&fx, &card, "2026-10-21");
    assert_eq!(o.limit_history.len(), 3);
    assert_eq!(
        o.credit_limit,
        Some(150_000),
        "the 11/11 statement is the most recent"
    );
}

#[test]
fn edited_dates_survive_a_cycle_change() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    fx.txn(&card, "2026-10-05", -1_000, "Padaria");
    let st = statement_for(fx.conn(), &card, "2026-10-05")
        .unwrap()
        .unwrap();
    // The bank postponed the closing (holiday) and the due date.
    fx.write(|tx| {
        update_statement(
            tx,
            &StatementPatch {
                id: st.clone(),
                closing_date: Some("2026-11-04".into()),
                due_date: Some("2026-11-11".into()),
                ..Default::default()
            },
            d("2026-10-06"),
        )
    })
    .unwrap();
    // Due date before closing is rejected.
    let e = fx
        .write(|tx| {
            update_statement(
                tx,
                &StatementPatch {
                    id: st.clone(),
                    due_date: Some("2026-11-01".into()),
                    ..Default::default()
                },
                d("2026-10-06"),
            )
        })
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    // Changing the card's cycle does not redo edited dates.
    fx.write(|tx| {
        tx.execute(
            "UPDATE credit_card SET closing_day = 5, due_day = 12 WHERE account_id = ?1",
            [&card],
        )?;
        recompute_statements(tx, &card, d("2026-10-06"))
    })
    .unwrap();
    let v = statement_view(fx.conn(), &st, d("2026-10-06")).unwrap();
    assert_eq!(
        (v.closing_date.as_str(), v.due_date.as_str(), v.dates_source),
        ("2026-11-04", "2026-11-11", DatesSource::Manual)
    );
    // Back to the computed ones.
    fx.write(|tx| {
        update_statement(
            tx,
            &StatementPatch {
                id: st.clone(),
                reset_dates: Some(true),
                ..Default::default()
            },
            d("2026-10-06"),
        )
    })
    .unwrap();
    let v = statement_view(fx.conn(), &st, d("2026-10-06")).unwrap();
    assert_eq!(
        (v.closing_date.as_str(), v.due_date.as_str(), v.dates_source),
        ("2026-11-05", "2026-11-12", DatesSource::Calc)
    );
}

fn fee(fx: &mut Fx, card: &str, kind: WaiverKind, threshold: Option<i64>) {
    let input = AnnualFeeInput {
        account_id: card.into(),
        amount: 60_000,
        installments: Some(12),
        anniversary_month: Some(12),
        waiver_kind: Some(kind),
        waiver_threshold: threshold,
        waiver_discount: None,
        starts_on: Some("2026-01-01".into()),
        notes: None,
    };
    fx.write(|tx| annual_fee::set(tx, &input, d("2026-01-01")))
        .unwrap();
}

#[test]
fn annual_fee_waiver_and_charge_checks() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, Some(500_000));
    fee(&mut fx, &card, WaiverKind::Spend, Some(100_000));
    // October statement (purchases Sep 4 to Oct 3): R$ 1,200 in purchases, but charged the fee.
    fx.txn(&card, "2026-09-10", -120_000, "Viagem");
    let isenta = fx.txn(&card, "2026-09-15", -5_000, "ANUIDADE DIFERENCIADA 09/12");
    // November statement: R$ 300 in purchases, installment of R$ 50 (correct) and one of R$ 80.
    fx.txn(&card, "2026-10-10", -30_000, "Mercado");
    fx.txn(&card, "2026-10-15", -5_000, "Anuidade 10/12");
    let errada = fx.txn(&card, "2026-10-16", -8_000, "ANUIDADE 11/12");

    let o = overview(&fx, &card, "2026-10-20");
    let s = o.annual_fee.unwrap();
    assert_eq!(s.installment_amount, 5_000);
    assert_eq!(s.paid_12m, 18_000);
    assert_eq!(s.next_renewal.as_deref(), Some("2026-12-01"));
    let issues: Vec<(String, Option<FeeIssue>)> = s
        .charges
        .iter()
        .map(|c| (c.txn_id.clone(), c.issue))
        .collect();
    assert_eq!(
        issues,
        vec![
            (isenta, Some(FeeIssue::ChargedWhileWaived)),
            (issues[1].0.clone(), None),
            (errada, Some(FeeIssue::AmountDiffers)),
        ]
    );
    assert_eq!(s.issues, 2);
    // Waiver on the open statement (November): R$ 300 of R$ 1,000.
    assert_eq!(
        (s.waiver.current, s.waiver.remaining, s.waiver.met),
        (30_000, 70_000, false)
    );

    // The warnings: a charge that does not match (one per charge) and the renewal in 30 days.
    let run = crate::notify::run(fx.conn(), d("2026-11-05")).unwrap();
    let kinds: Vec<String> = fx
        .conn()
        .prepare("SELECT kind FROM alert ORDER BY kind")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(!run.created.is_empty());
    assert_eq!(
        kinds
            .iter()
            .filter(|k| k.as_str() == "card_fee_issue")
            .count(),
        2
    );
    assert!(kinds.iter().any(|k| k == "card_fee_renewal"), "{kinds:?}");
    // Running again does not repeat it.
    let again = crate::notify::run(fx.conn(), d("2026-11-05")).unwrap();
    assert!(again.created.is_empty(), "{again:?}");

    // Card with no annual fee from today.
    fx.write(|tx| annual_fee::end(tx, &card, d("2026-11-05")))
        .unwrap();
    assert!(overview(&fx, &card, "2026-11-06").annual_fee.is_none());
}

#[test]
fn annual_fee_rules_replace_and_validate() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    fee(&mut fx, &card, WaiverKind::None, None);
    let mut input = AnnualFeeInput {
        account_id: card.clone(),
        amount: 30_000,
        installments: Some(1),
        anniversary_month: None,
        waiver_kind: Some(WaiverKind::Discount),
        waiver_threshold: None,
        waiver_discount: Some("0.5".into()),
        starts_on: Some("2026-06-01".into()),
        notes: None,
    };
    fx.write(|tx| annual_fee::set(tx, &input, d("2026-06-01")))
        .unwrap();
    let rules = annual_fee::rules(fx.conn(), &card).unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[1].ends_on.as_deref(), Some("2026-05-31"));
    // 50% discount on the single installment: expects R$ 150.
    fx.txn(&card, "2026-07-01", -15_000, "ANUIDADE");
    let s = overview(&fx, &card, "2026-07-02").annual_fee.unwrap();
    assert_eq!(s.charges[0].expected, Some(15_000));
    assert_eq!(s.charges[0].issue, None);
    // Waiver by spend without a value and a discount outside 0..1 are rejected.
    input.waiver_kind = Some(WaiverKind::Spend);
    assert_eq!(
        fx.write(|tx| annual_fee::set(tx, &input, d("2026-06-01")))
            .unwrap_err()
            .code(),
        "invalid"
    );
    input.waiver_kind = Some(WaiverKind::Discount);
    input.waiver_discount = Some("1.5".into());
    assert_eq!(
        fx.write(|tx| annual_fee::set(tx, &input, d("2026-06-01")))
            .unwrap_err()
            .code(),
        "invalid"
    );
}

#[test]
fn limit_warning_once_per_band_and_statement() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, Some(100_000));
    fx.txn(&card, "2026-10-05", -85_000, "TV");
    crate::notify::run(fx.conn(), d("2026-10-06")).unwrap();
    crate::notify::run(fx.conn(), d("2026-10-07")).unwrap();
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'card_limit'"),
        1
    );
    fx.txn(&card, "2026-10-08", -12_000, "Sofá");
    crate::notify::run(fx.conn(), d("2026-10-08")).unwrap();
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'card_limit'"),
        2,
        "went over 95%"
    );
}
