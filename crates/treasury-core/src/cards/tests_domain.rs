use super::*;
use crate::transactions::testkit::{Fx, d};
use crate::transactions::{TransferInput, TxnKind, TxnStatus};

fn due_of(fx: &Fx, id: &str) -> String {
    crate::transactions::get(fx.conn(), id)
        .unwrap()
        .statement_due_date
        .unwrap()
}

fn statements(fx: &Fx, card: &str, today: &str) -> Vec<StatementView> {
    list_statements(
        fx.conn(),
        &ListStatementsArgs {
            account_id: Some(card.into()),
            from_month: None,
            to_month: None,
        },
        d(today),
    )
    .unwrap()
}

fn refund(fx: &mut Fx, card: &str, date: &str, amount: i64) -> Id {
    let mut i = Fx::input(card, date, amount, "Estorno");
    i.kind = Some(TxnKind::Expense);
    fx.txn_with(i).unwrap()
}

#[test]
fn statement_total_status_and_payments() {
    let mut fx = Fx::new();
    let bank = fx.checking("Corrente", 100000);
    let card = fx.card("Cartão", 3, 10, Some(100000));
    fx.txn(&card, "2026-09-15", -10000, "Mercado");
    refund(&mut fx, &card, "2026-09-20", 2000);
    let st = &statements(&fx, &card, "2026-10-01")[0];
    assert_eq!(st.reference_month, "2026-10");
    assert_eq!(
        (st.closing_date.as_str(), st.due_date.as_str()),
        ("2026-10-03", "2026-10-10")
    );
    assert_eq!(st.total, 8000, "purchases − refunds");
    assert_eq!(st.txn_count, 2);
    assert_eq!(st.status, StatementStatus::Open);
    let st = statements(&fx, &card, "2026-10-05").remove(0);
    assert_eq!(st.status, StatementStatus::Closed);

    let pay = |fx: &mut Fx, amount: Option<i64>, date: &str, today: &str| {
        let args = PayStatementArgs {
            statement_id: st.id.clone(),
            from_account_id: bank.clone(),
            amount,
            date: Some(date.into()),
        };
        fx.write(|tx| pay_statement(tx, &args, d(today)))
    };
    let t = pay(&mut fx, Some(3000), "2026-10-05", "2026-10-05").unwrap();
    let to = crate::transactions::get(fx.conn(), &t.to_id).unwrap();
    assert_eq!(to.kind, TxnKind::CardPayment);
    assert_eq!(to.statement_id.as_deref(), Some(st.id.as_str()));
    assert_eq!(to.amount, 3000);
    let from = crate::transactions::get(fx.conn(), &t.from_id).unwrap();
    assert_eq!((from.amount, from.statement_id), (-3000, None));
    let v = statement_view(fx.conn(), &st.id, d("2026-10-05")).unwrap();
    assert_eq!((v.total, v.paid, v.remaining), (8000, 3000, 5000));
    assert_eq!(v.status, StatementStatus::Partial);
    assert_eq!(
        v.txn_count, 2,
        "payment counts neither in the count nor in the total"
    );
    // No amount: pays the remainder.
    pay(&mut fx, None, "2026-10-06", "2026-10-06").unwrap();
    let v = statement_view(fx.conn(), &st.id, d("2026-10-06")).unwrap();
    assert_eq!(
        (v.paid, v.remaining, v.status),
        (8000, 0, StatementStatus::Paid)
    );
    let stored: String = fx
        .conn()
        .query_row(
            "SELECT status FROM card_statement WHERE id = ?1",
            [&st.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stored, "paid", "status stored in the table");
    assert!(
        pay(&mut fx, None, "2026-10-06", "2026-10-06").is_err(),
        "nothing to pay"
    );
    assert_eq!(
        crate::accounts::balance(fx.conn(), &bank, "2026-10-31", false).unwrap(),
        92000
    );
    // Scheduled payment (future date) is forecast and does not count as paid.
    fx.txn(&card, "2026-10-04", -500, "Café");
    let nov = statements(&fx, &card, "2026-10-06")
        .into_iter()
        .find(|s| s.reference_month == "2026-11")
        .unwrap();
    let args = PayStatementArgs {
        statement_id: nov.id.clone(),
        from_account_id: bank.clone(),
        amount: None,
        date: None,
    };
    let t = fx
        .write(|tx| pay_statement(tx, &args, d("2026-10-06")))
        .unwrap();
    let to = crate::transactions::get(fx.conn(), &t.to_id).unwrap();
    assert_eq!(to.status, TxnStatus::Planned);
    assert_eq!(to.date, "2026-11-10");
    assert_eq!(
        statement_view(fx.conn(), &nov.id, d("2026-10-06"))
            .unwrap()
            .paid,
        0
    );
    // Account in another currency does not pay the statement.
    let usd = fx.account("EUA", crate::accounts::AccountKind::Foreign, "USD", 0);
    let args = PayStatementArgs {
        statement_id: nov.id.clone(),
        from_account_id: usd,
        amount: Some(100),
        date: None,
    };
    assert!(
        fx.write(|tx| pay_statement(tx, &args, d("2026-10-06")))
            .is_err()
    );
}

#[test]
fn closing_in_short_months() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 30, 5, None);
    let feb = fx.txn(&card, "2026-02-28", -100, "último dia de fevereiro");
    let mar = fx.txn(&card, "2026-03-01", -100, "primeiro de março");
    assert_eq!(due_of(&fx, &feb), "2026-03-05");
    assert_eq!(due_of(&fx, &mar), "2026-04-05");
    let card31 = fx.card("Cartão 31", 31, 10, None);
    let leap = fx.txn(&card31, "2028-02-29", -100, "bissexto");
    assert_eq!(due_of(&fx, &leap), "2028-03-10");
    let st = statements(&fx, &card31, "2028-01-01").remove(0);
    assert_eq!(
        (st.period_start.as_str(), st.closing_date.as_str()),
        ("2028-02-01", "2028-02-29")
    );
    // Dates coming from the reference month match those of the purchase.
    let dd = dates_for_reference_month("2028-03", 31, 10).unwrap();
    assert_eq!(dd, statement_dates(d("2028-02-15"), 31, 10));
    let dd = dates_for_reference_month("2026-11", 25, 5).unwrap();
    assert_eq!(dd, statement_dates(d("2026-10-20"), 25, 5));
}

#[test]
fn card_payment_transfer_picks_oldest_unpaid_closed_statement() {
    let mut fx = Fx::new();
    let bank = fx.checking("Corrente", 0);
    let card = fx.card("Cartão", 3, 10, None);
    fx.txn(&card, "2026-08-20", -1000, "setembro");
    fx.txn(&card, "2026-09-20", -2000, "outubro");
    let input = TransferInput {
        from_account_id: bank.clone(),
        to_account_id: card.clone(),
        date: "2026-10-08".into(),
        amount: 1000,
        to_amount: None,
        description: None,
        kind: Some(TxnKind::CardPayment),
        status: None,
        member_id: None,
    };
    let ids = fx
        .write(|tx| crate::transactions::create_transfer(tx, &input, d("2026-10-08")))
        .unwrap();
    assert_eq!(due_of(&fx, &ids.to_id), "2026-09-10", "the oldest open one");
    let ids = fx
        .write(|tx| crate::transactions::create_transfer(tx, &input, d("2026-10-08")))
        .unwrap();
    assert_eq!(due_of(&fx, &ids.to_id), "2026-10-10");
    // card_payment only with a card as the destination.
    let bad = TransferInput {
        to_account_id: bank.clone(),
        from_account_id: card.clone(),
        ..input
    };
    assert!(
        fx.write(|tx| crate::transactions::create_transfer(tx, &bad, d("2026-10-08")))
            .is_err()
    );
}

#[test]
fn informed_total_and_move_between_statements() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    let other = fx.card("Outro", 3, 10, None);
    let t = fx.txn(&card, "2026-10-01", -1500, "Compra");
    let alien = fx.txn(&other, "2026-10-01", -1500, "Compra no outro");
    let st = statements(&fx, &card, "2026-10-01").remove(0);
    fx.write(|tx| set_informed_total(tx, &st.id, Some(1490)))
        .unwrap();
    assert_eq!(
        statement_view(fx.conn(), &st.id, d("2026-10-01"))
            .unwrap()
            .informed_total,
        Some(1490)
    );
    assert!(fx.write(|tx| set_informed_total(tx, "nada", None)).is_err());
    let args = MoveToStatementArgs {
        txn_ids: vec![t.clone()],
        statement_id: None,
        reference_month: Some("2026-11".into()),
        account_id: Some(card.clone()),
    };
    let n = fx
        .write(|tx| move_to_statement(tx, &args, d("2026-10-01")))
        .unwrap();
    assert_eq!(n, 1);
    assert_eq!(due_of(&fx, &t), "2026-11-10");
    assert_eq!(
        statement_view(fx.conn(), &st.id, d("2026-10-01"))
            .unwrap()
            .total,
        0
    );
    let args = MoveToStatementArgs {
        txn_ids: vec![alien],
        statement_id: Some(st.id.clone()),
        reference_month: None,
        account_id: None,
    };
    assert!(
        fx.write(|tx| move_to_statement(tx, &args, d("2026-10-01")))
            .is_err()
    );
}

#[test]
fn installments_split_cents_and_become_cleared() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, Some(1000000));
    let input = InstallmentInput {
        account_id: card.clone(),
        description: "Compra".into(),
        total_amount: 100000,
        installments: 3,
        purchase_date: "2026-01-31".into(),
        category_id: Some(fx.cat("Eletrônicos")),
        member_id: Some(fx.member("Ana")),
        tags: Some(vec!["casa".into()]),
    };
    let plan = fx
        .write(|tx| create_installments(tx, &input, d("2026-03-01")))
        .unwrap();
    let mut stmt = fx
        .conn()
        .prepare(
            "SELECT date, amount, description, status, installment_number FROM txn
              WHERE installment_plan_id = ?1 ORDER BY installment_number",
        )
        .unwrap();
    let rows: Vec<(String, i64, String, String, i64)> = stmt
        .query_map([&plan], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    drop(stmt);
    assert_eq!(
        rows,
        vec![
            (
                "2026-01-31".into(),
                -33334,
                "Compra 1/3".into(),
                "cleared".into(),
                1
            ),
            (
                "2026-02-28".into(),
                -33333,
                "Compra 2/3".into(),
                "cleared".into(),
                2
            ),
            (
                "2026-03-31".into(),
                -33333,
                "Compra 3/3".into(),
                "planned".into(),
                3
            ),
        ]
    );
    let views = crate::transactions::views(
        fx.conn(),
        &crate::transactions::query_ids(
            fx.conn(),
            &crate::transactions::TxnFilter {
                account_ids: Some(vec![card.clone()]),
                sort: Some(crate::transactions::TxnSort::DateAsc),
                ..Default::default()
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(views[0].installment_total, Some(3));
    assert_eq!(views[0].tags[0].name, "casa");
    assert_eq!(
        views
            .iter()
            .map(|v| v.statement_due_date.clone().unwrap())
            .collect::<Vec<_>>(),
        vec!["2026-02-10", "2026-03-10", "2026-04-10"]
    );
    let p = get_installment_plan(fx.conn(), &plan, d("2026-03-01")).unwrap();
    assert_eq!((p.paid_installments, p.remaining_amount), (2, 33333));

    // When the date arrives, the planned installment becomes cleared.
    assert_eq!(
        clear_due_installments(fx.conn(), d("2026-03-30")).unwrap(),
        0
    );
    assert_eq!(
        clear_due_installments(fx.conn(), d("2026-03-31")).unwrap(),
        1
    );
    assert_eq!(
        fx.count("SELECT count(*) FROM txn WHERE status = 'planned'"),
        0
    );

    // Invalid values.
    let mut bad = input.clone();
    bad.installments = 0;
    assert!(
        fx.write(|tx| create_installments(tx, &bad, d("2026-02-15")))
            .is_err()
    );
    let mut bad = input.clone();
    bad.category_id = Some(fx.cat("Salário"));
    assert!(
        fx.write(|tx| create_installments(tx, &bad, d("2026-02-15")))
            .is_err()
    );
    let bank = fx.checking("Corrente", 0);
    let mut bad = input.clone();
    bad.account_id = bank;
    assert!(
        fx.write(|tx| create_installments(tx, &bad, d("2026-02-15")))
            .is_err()
    );

    // Deleting the installment plan takes the installments with it (unless one is reconciled).
    fx.conn()
        .execute(
            "UPDATE txn SET status = 'reconciled' WHERE installment_number = 1",
            [],
        )
        .unwrap();
    assert!(
        fx.write(|tx| delete_installment_plan(tx, &plan, d("2026-02-15")))
            .is_err()
    );
    fx.conn()
        .execute(
            "UPDATE txn SET status = 'cleared' WHERE installment_number = 1",
            [],
        )
        .unwrap();
    let n = fx
        .write(|tx| delete_installment_plan(tx, &plan, d("2026-02-15")))
        .unwrap();
    assert_eq!(n, 3);
    assert_eq!(fx.count("SELECT count(*) FROM txn"), 0);
    assert_eq!(fx.count("SELECT count(*) FROM installment_plan"), 0);
    assert_eq!(fx.count("SELECT count(*) FROM txn_tag"), 0);
}

#[test]
fn future_commitments_by_due_month() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    fx.txn(&card, "2026-10-01", -5000, "outubro, ainda não paga");
    let input = InstallmentInput {
        account_id: card.clone(),
        description: "TV".into(),
        total_amount: 100000,
        installments: 3,
        purchase_date: "2026-10-14".into(),
        category_id: None,
        member_id: None,
        tags: None,
    };
    fx.write(|tx| create_installments(tx, &input, d("2026-10-14")))
        .unwrap();
    let got: Vec<(String, i64)> = future_commitments(fx.conn(), Some(&card), 5, d("2026-10-14"))
        .unwrap()
        .into_iter()
        .map(|m| (m.month, m.amount))
        .collect();
    assert_eq!(
        got,
        vec![
            ("2026-10".into(), 5000),
            ("2026-11".into(), 33334),
            ("2026-12".into(), 33333),
            ("2027-01".into(), 33333),
            ("2027-02".into(), 0),
        ]
    );
    let all = future_commitments(fx.conn(), None, 2, d("2026-10-14")).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[1].amount, 33334);
}
