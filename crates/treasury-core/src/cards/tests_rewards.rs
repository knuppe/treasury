//! Cards, phase 4: expected cashback per statement (rate per category, cap, no fees and
//! charges), received (credit after closing) and the warning when it does not arrive.

use super::rewards::{RewardKind, RewardMultiplierInput, RewardProgramInput};
use super::*;
use crate::transactions::TxnKind;
use crate::transactions::testkit::{Fx, d};

/// Month, eligible purchases, expected, received, did not arrive.
type Row<'a> = (&'a str, i64, Option<i64>, Option<i64>, bool);

fn program(fx: &mut Fx, card: &str, kind: RewardKind, base: &str, cap: Option<i64>) {
    let restaurantes = fx.cat("Restaurantes");
    let input = RewardProgramInput {
        account_id: card.into(),
        kind,
        base_rate: base.into(),
        rate_currency: None,
        cycle_cap: cap,
        destination: None,
        destination_account_id: None,
        payout_days: Some(20),
        starts_on: Some("2026-01-01".into()),
        notes: None,
        multipliers: Some(vec![RewardMultiplierInput {
            category_id: restaurantes,
            rate: if kind == RewardKind::Cashback {
                "0.03".into()
            } else {
                "5".into()
            },
        }]),
        loyalty_program_id: None,
    };
    fx.write(|tx| rewards::set(tx, &input, d("2026-01-01")))
        .unwrap();
}

fn buy(fx: &mut Fx, card: &str, date: &str, amount: i64, desc: &str, cat: Option<&str>) {
    let mut i = Fx::input(card, date, amount, desc);
    i.kind = Some(TxnKind::Expense);
    i.category_id = cat.map(|c| fx.cat(c));
    fx.txn_with(i).unwrap();
}

#[test]
fn cashback_expected_received_and_missing() {
    let mut fx = Fx::new();
    // Closes on the 3rd, due on the 10th.
    let card = fx.card("Cartão", 3, 10, None);
    program(&mut fx, &card, RewardKind::Cashback, "0.01", None);
    // October statement (Sep 4 to Oct 3): R$ 1,000 ordinary + R$ 200 at a restaurant − refund of
    // R$ 100; annual fee and interest do not count.
    buy(&mut fx, &card, "2026-09-10", -100_000, "Loja", None);
    buy(
        &mut fx,
        &card,
        "2026-09-12",
        -20_000,
        "Restaurante",
        Some("Restaurantes"),
    );
    buy(&mut fx, &card, "2026-09-14", 10_000, "Estorno Loja", None);
    buy(&mut fx, &card, "2026-09-15", -5_000, "ANUIDADE 1/12", None);
    buy(&mut fx, &card, "2026-09-16", -3_000, "JUROS", None);
    // November statement: R$ 500.
    buy(&mut fx, &card, "2026-10-10", -50_000, "Mercado", None);
    // October cashback credited to the card after the Oct 3 closing.
    let mut cb = Fx::input(&card, "2026-10-08", 1_500, "CASHBACK");
    cb.kind = Some(TxnKind::Income);
    fx.txn_with(cb).unwrap();

    let v = rewards::view(fx.conn(), &card, d("2026-12-01")).unwrap();
    let got: Vec<Row<'_>> = v
        .statements
        .iter()
        .map(|s| {
            (
                s.reference_month.as_str(),
                s.eligible_spend,
                s.expected,
                s.received,
                s.missing,
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            // November: 1% of 500 = 5; nothing received and the deadline (Nov 23) has passed.
            ("2026-11", 50_000, Some(500), Some(0), true),
            // October: 1% of (1,000 − 100) + 3% of 200 = 9 + 6 = 15.
            ("2026-10", 110_000, Some(1_500), Some(1_500), false),
        ]
    );
    assert_eq!((v.expected_12m, v.received_12m), (2_000, 1_500));

    // The warning for cashback that did not arrive (one per statement).
    crate::notify::run(fx.conn(), d("2026-12-01")).unwrap();
    crate::notify::run(fx.conn(), d("2026-12-02")).unwrap();
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'card_cashback_missing'"),
        1
    );
}

#[test]
fn points_and_cap() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    // 2 points per real, 5 at a restaurant, cap of 1,000 points per statement.
    program(&mut fx, &card, RewardKind::Points, "2", Some(1_000));
    buy(&mut fx, &card, "2026-09-10", -30_000, "Loja", None);
    buy(
        &mut fx,
        &card,
        "2026-09-12",
        -2_000,
        "Restaurante",
        Some("Restaurantes"),
    );
    let v = rewards::view(fx.conn(), &card, d("2026-10-05")).unwrap();
    let oct = v
        .statements
        .iter()
        .find(|s| s.reference_month == "2026-10")
        .unwrap();
    // 300 × 2 + 20 × 5 = 700 points; no cashback received.
    assert_eq!((oct.expected, oct.received), (Some(700), None));
    buy(&mut fx, &card, "2026-09-13", -50_000, "Viagem", None);
    let v = rewards::view(fx.conn(), &card, d("2026-10-05")).unwrap();
    assert_eq!(
        v.statements
            .iter()
            .find(|s| s.reference_month == "2026-10")
            .unwrap()
            .expected,
        Some(1_000),
        "cap"
    );
    // Invalid rate and end of the program.
    let bad = RewardProgramInput {
        account_id: card.clone(),
        kind: RewardKind::Cashback,
        base_rate: "1.5".into(),
        rate_currency: None,
        cycle_cap: None,
        destination: None,
        destination_account_id: None,
        payout_days: None,
        starts_on: None,
        notes: None,
        multipliers: None,
        loyalty_program_id: None,
    };
    assert_eq!(
        fx.write(|tx| rewards::set(tx, &bad, d("2026-10-05")))
            .unwrap_err()
            .code(),
        "invalid"
    );
    fx.write(|tx| rewards::end(tx, &card, d("2026-10-05")))
        .unwrap();
    assert!(
        rewards::view(fx.conn(), &card, d("2026-10-06"))
            .unwrap()
            .program
            .is_none()
    );
}
