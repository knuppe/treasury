//! Cards, phase 3: revolving credit (overdue statement with a balance), charges and the cap,
//! purchase abroad.
use super::*;
use crate::transactions::testkit::{Fx, d};
use crate::transactions::{TxnPatch, TxnStatus};

fn pay(fx: &mut Fx, bank: &str, statement: &str, amount: i64, date: &str) {
    let args = PayStatementArgs {
        statement_id: statement.into(),
        from_account_id: bank.into(),
        amount: Some(amount),
        date: Some(date.into()),
    };
    fx.write(|tx| pay_statement(tx, &args, d(date))).unwrap();
}

#[test]
fn partial_payment_enters_revolving_and_charges_are_compared_with_the_cap() {
    let mut fx = Fx::new();
    let bank = fx.checking("Corrente", 1_000_000);
    // Closes on the 3rd, due on the 10th.
    let card = fx.card("Cartão", 3, 10, Some(500_000));
    fx.txn(&card, "2026-09-15", -100_000, "Loja");
    let oct = statement_for(fx.conn(), &card, "2026-09-15")
        .unwrap()
        .unwrap();
    // With no payment recorded at all: not revolving (it may just be missing data).
    assert!(
        revolving::status(fx.conn(), &card, d("2026-10-11"))
            .unwrap()
            .is_none()
    );
    pay(&mut fx, &bank, &oct, 30_000, "2026-10-10");
    let r = revolving::status(fx.conn(), &card, d("2026-10-11"))
        .unwrap()
        .unwrap();
    assert_eq!((r.statement_id.as_str(), r.unpaid), (oct.as_str(), 70_000));
    assert!(!r.beyond_next_statement);
    // Charges on the next statement; an ordinary purchase and the IOF on an international
    // purchase do not count.
    fx.txn(&card, "2026-10-12", -20_000, "JUROS DO ROTATIVO");
    fx.txn(&card, "2026-10-12", -1_000, "IOF FINANCIAMENTO");
    fx.txn(&card, "2026-10-13", -500, "IOF COMPRA INTERNACIONAL");
    fx.txn(&card, "2026-10-14", -9_000, "Mercado");
    let r = revolving::status(fx.conn(), &card, d("2026-10-20"))
        .unwrap()
        .unwrap();
    assert_eq!(r.charges, 21_000);
    assert_eq!(r.charges_bp, 3_000);
    assert!(!r.over_cap);
    fx.txn(&card, "2026-10-15", -60_000, "ENCARGOS DE ATRASO");
    let r = revolving::status(fx.conn(), &card, d("2026-10-20"))
        .unwrap()
        .unwrap();
    assert!(r.over_cap, "81 thousand in charges over 70 thousand");

    // The revolving notice and the cap notice (the test vault is Brazilian).
    crate::notify::run(fx.conn(), d("2026-10-20")).unwrap();
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'card_revolving'"),
        1
    );
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'card_revolving_cap'"),
        1
    );

    // The next statement fell due unpaid: it stays in revolving, beyond the next statement.
    let r = revolving::status(fx.conn(), &card, d("2026-11-11"))
        .unwrap()
        .unwrap();
    assert!(r.beyond_next_statement);
    // Paid in full: leaves revolving.
    let nov = statement_for(fx.conn(), &card, "2026-10-12")
        .unwrap()
        .unwrap();
    pay(&mut fx, &bank, &oct, 70_000, "2026-11-12");
    let rest = statement_view(fx.conn(), &nov, d("2026-11-12"))
        .unwrap()
        .remaining;
    pay(&mut fx, &bank, &nov, rest, "2026-11-12");
    assert!(
        revolving::status(fx.conn(), &card, d("2026-11-13"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn purchase_abroad_keeps_original_amount_and_currency() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    let mut i = Fx::input(&card, "2026-10-05", -41_200, "AMAZON");
    i.orig_amount = Some(8_000);
    i.orig_currency = Some("usd".into());
    let id = fx.txn_with(i).unwrap();
    let v = crate::transactions::get(fx.conn(), &id).unwrap();
    assert_eq!(
        (v.orig_amount, v.orig_currency.as_deref()),
        (Some(-8_000), Some("USD"))
    );
    // Only one of the two, or the same currency as the account, is refused.
    let mut i = Fx::input(&card, "2026-10-05", -100, "X");
    i.orig_amount = Some(20);
    assert_eq!(fx.txn_with(i).unwrap_err().code(), "invalid");
    let mut i = Fx::input(&card, "2026-10-05", -100, "X");
    i.orig_amount = Some(20);
    i.orig_currency = Some("BRL".into());
    assert_eq!(fx.txn_with(i).unwrap_err().code(), "invalid");
    // Remove the original amount.
    fx.write(|tx| {
        crate::transactions::update(
            tx,
            &TxnPatch {
                id: id.clone(),
                account_id: None,
                date: None,
                amount: None,
                kind: None,
                status: Some(TxnStatus::Cleared),
                description: None,
                payee: None,
                notes: None,
                category_id: None,
                member_id: None,
                splits: None,
                tags: None,
                project_id: None,
                transfer_account_id: None,
                transfer_amount: None,
                card_plastic_id: None,
                orig_amount: Some(None),
                orig_currency: None,
            },
            d("2026-10-06"),
        )
    })
    .unwrap();
    let v = crate::transactions::get(fx.conn(), &id).unwrap();
    assert_eq!((v.orig_amount, v.orig_currency), (None, None));
}

#[test]
fn builtin_rule_puts_card_charges_in_interest() {
    let fx = Fx::new();
    let cat: Option<String> = fx
        .conn()
        .query_row(
            "SELECT json_extract(actions, '$.category_id') FROM rule WHERE name LIKE 'Juros e encargos%'",
            [],
            |r| r.get(0),
        )
        .ok();
    if let Some(cat) = cat {
        let name: String = fx
            .conn()
            .query_row("SELECT name FROM category WHERE id = ?1", [&cat], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(name, "Juros e multas");
    }
    crate::rules::builtin::check_list("BR");
}
