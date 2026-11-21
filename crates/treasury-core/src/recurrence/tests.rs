use super::*;
use crate::changes::{AuthorKind, ChangeListArgs};
use crate::transactions::testkit::{Fx, d};

fn monthly(day: Option<u32>) -> RuleSpec {
    RuleSpec {
        freq: Freq::Monthly,
        interval: None,
        by_month_day: day,
        by_day: None,
        by_month: None,
        count: None,
    }
}

fn input(
    account: &str,
    amount: i64,
    kind: TxnKind,
    start: &str,
    spec: RuleSpec,
) -> RecurrenceInput {
    RecurrenceInput {
        description: "Aluguel".into(),
        account_id: account.into(),
        amount,
        kind,
        category_id: None,
        member_id: None,
        to_account_id: None,
        spec,
        start_date: start.into(),
        end_date: None,
    }
}

fn empty_patch(id: &str) -> RecurrencePatch {
    RecurrencePatch {
        id: id.into(),
        description: None,
        amount: None,
        category_id: None,
        member_id: None,
        spec: None,
        end_date: None,
        active: None,
        update_planned: None,
    }
}

/// (date, amount, status) of the recurrence's transactions, by date.
fn generated(fx: &Fx, rec: &str) -> Vec<(String, i64, String)> {
    let mut stmt = fx
        .conn()
        .prepare(
            "SELECT date, amount, status FROM txn WHERE recurrence_id = ?1 ORDER BY date, amount",
        )
        .unwrap();
    stmt.query_map([rec], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn dates(fx: &Fx, rec: &str) -> Vec<String> {
    generated(fx, rec).into_iter().map(|(d, _, _)| d).collect()
}

fn materialize(fx: &mut Fx, today: &str) -> usize {
    fx.write(|tx| materialize_all(tx, d(today), DEFAULT_HORIZON_DAYS))
        .unwrap()
}

#[test]
fn create_materializes_from_today_with_normalized_rule() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let mut i = input(&acc, 150000, TxnKind::Expense, "2026-09-05", monthly(None));
    i.category_id = Some(fx.cat("Aluguel"));
    let id = fx.write(|tx| create(tx, &i, d("2026-10-14"))).unwrap();
    let r = get(fx.conn(), &id, d("2026-10-14")).unwrap();
    assert_eq!(r.rrule, "FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=5");
    assert_eq!(r.amount, -150000, "expense stays negative");
    assert_eq!(r.human, "todo mês no dia 5");
    assert_eq!(r.next_dates, vec!["2026-11-05", "2026-12-05", "2027-01-05"]);
    assert_eq!(r.monthly_equivalent, -150000);
    // Nothing in the past; up to today + 62 days (Dec 15).
    assert_eq!(
        generated(&fx, &id),
        vec![
            ("2026-11-05".into(), -150000, "planned".into()),
            ("2026-12-05".into(), -150000, "planned".into())
        ]
    );
    let t = crate::transactions::views(
        fx.conn(),
        &crate::transactions::query_ids(
            fx.conn(),
            &crate::transactions::TxnFilter {
                account_ids: Some(vec![acc.clone()]),
                ..Default::default()
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        t.iter()
            .all(|v| v.category_path.as_deref() == Some("Moradia › Aluguel"))
    );
    let until: String = fx
        .conn()
        .query_row(
            "SELECT generated_until FROM recurrence WHERE id = ?1",
            [&id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(until, "2026-12-15");

    // Validations.
    assert!(
        fx.write(|tx| create(
            tx,
            &input(&acc, 0, TxnKind::Expense, "2026-10-01", monthly(None)),
            d("2026-10-14")
        ))
        .is_err()
    );
    let mut bad = input(&acc, -10, TxnKind::Expense, "2026-10-01", monthly(None));
    bad.category_id = Some(fx.cat("Salário"));
    assert!(fx.write(|tx| create(tx, &bad, d("2026-10-14"))).is_err());
    let mut bad = input(&acc, -10, TxnKind::Expense, "2026-10-01", monthly(None));
    bad.end_date = Some("2026-09-01".into());
    assert!(fx.write(|tx| create(tx, &bad, d("2026-10-14"))).is_err());
    let bad = input(&acc, -10, TxnKind::Transfer, "2026-10-01", monthly(None));
    assert!(
        fx.write(|tx| create(tx, &bad, d("2026-10-14"))).is_err(),
        "no destination"
    );
    let bad = input(&acc, -10, TxnKind::Adjustment, "2026-10-01", monthly(None));
    assert!(fx.write(|tx| create(tx, &bad, d("2026-10-14"))).is_err());
    let bad = input(&acc, -10, TxnKind::Expense, "2026-10-01", monthly(Some(40)));
    assert!(fx.write(|tx| create(tx, &bad, d("2026-10-14"))).is_err());
}

#[test]
fn materialization_is_idempotent_and_respects_deleted() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let id = fx
        .write(|tx| {
            create(
                tx,
                &input(
                    &acc,
                    -1000,
                    TxnKind::Expense,
                    "2026-10-05",
                    monthly(Some(5)),
                ),
                d("2026-10-01"),
            )
        })
        .unwrap();
    assert_eq!(dates(&fx, &id), vec!["2026-10-05", "2026-11-05"]);
    assert_eq!(materialize(&mut fx, "2026-10-01"), 0);
    assert_eq!(materialize(&mut fx, "2026-10-02"), 0);
    // The user deletes November's planned one: it does not come back.
    let nov: String = fx
        .conn()
        .query_row(
            "SELECT id FROM txn WHERE recurrence_id = ?1 AND date = '2026-11-05'",
            [&id],
            |r| r.get(0),
        )
        .unwrap();
    fx.write(|tx| crate::transactions::delete(tx, &[nov], d("2026-10-02")))
        .unwrap();
    assert_eq!(materialize(&mut fx, "2026-10-03"), 0);
    assert_eq!(dates(&fx, &id), vec!["2026-10-05"]);
    // With time, the new dates come in (and only they).
    assert_eq!(materialize(&mut fx, "2026-11-05"), 2);
    assert_eq!(
        dates(&fx, &id),
        vec!["2026-10-05", "2026-12-05", "2027-01-05"]
    );
    // Even without generated_until, already generated dates do not duplicate.
    fx.conn()
        .execute("UPDATE recurrence SET generated_until = NULL", [])
        .unwrap();
    assert_eq!(
        materialize(&mut fx, "2026-11-05"),
        1,
        "only Nov 5, which was missing"
    );
    assert_eq!(
        dates(&fx, &id),
        vec!["2026-10-05", "2026-11-05", "2026-12-05", "2027-01-05"]
    );
}

#[test]
fn day_31_and_end_date() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let mut i = input(
        &acc,
        -1000,
        TxnKind::Expense,
        "2027-01-31",
        monthly(Some(31)),
    );
    i.end_date = Some("2027-04-30".into());
    let id = fx.write(|tx| create(tx, &i, d("2027-01-15"))).unwrap();
    assert_eq!(dates(&fx, &id), vec!["2027-01-31", "2027-02-28"]);
    materialize(&mut fx, "2027-04-01");
    assert_eq!(
        dates(&fx, &id),
        vec!["2027-01-31", "2027-02-28", "2027-03-31", "2027-04-30"]
    );
    let r = get(fx.conn(), &id, d("2027-04-01")).unwrap();
    assert_eq!(r.human, "todo mês no último dia, até 30/04/2027");
    assert_eq!(r.next_dates, vec!["2027-04-30"]);
}

#[test]
fn recurring_transfer_generates_both_legs() {
    let mut fx = Fx::new();
    let a = fx.checking("Corrente", 0);
    let b = fx.checking("Poupança", 0);
    let card = fx.card("Cartão", 3, 10, None);
    let usd = fx.account("EUA", crate::accounts::AccountKind::Foreign, "USD", 0);
    let mut i = input(
        &a,
        50000,
        TxnKind::Transfer,
        "2026-10-20",
        monthly(Some(20)),
    );
    i.description = "Reserva mensal".into();
    i.to_account_id = Some(b.clone());
    let id = fx.write(|tx| create(tx, &i, d("2026-10-14"))).unwrap();
    assert_eq!(
        generated(&fx, &id),
        vec![
            ("2026-10-20".into(), -50000, "planned".into()),
            ("2026-10-20".into(), 50000, "planned".into()),
            ("2026-11-20".into(), -50000, "planned".into()),
            ("2026-11-20".into(), 50000, "planned".into()),
        ]
    );
    assert_eq!(
        fx.count("SELECT count(DISTINCT transfer_id) FROM txn WHERE transfer_id IS NOT NULL"),
        2
    );
    // Deleting one leg deletes both.
    let leg: String = fx
        .conn()
        .query_row(
            "SELECT id FROM txn WHERE account_id = ?1 AND date = '2026-10-20'",
            [&b],
            |r| r.get(0),
        )
        .unwrap();
    fx.write(|tx| crate::transactions::delete(tx, &[leg], d("2026-10-14")))
        .unwrap();
    assert_eq!(dates(&fx, &id), vec!["2026-11-20", "2026-11-20"]);

    // For a card it becomes a statement payment, with the card leg on a statement.
    let mut i = input(
        &a,
        -30000,
        TxnKind::Transfer,
        "2026-11-10",
        monthly(Some(10)),
    );
    i.to_account_id = Some(card.clone());
    let pid = fx.write(|tx| create(tx, &i, d("2026-10-14"))).unwrap();
    let r = get(fx.conn(), &pid, d("2026-10-14")).unwrap();
    assert_eq!(r.kind, TxnKind::CardPayment);
    let legs = crate::transactions::views(
        fx.conn(),
        &crate::transactions::query_ids(
            fx.conn(),
            &crate::transactions::TxnFilter {
                account_ids: Some(vec![card.clone()]),
                ..Default::default()
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(legs.len(), 2);
    assert!(
        legs.iter()
            .all(|l| l.kind == TxnKind::CardPayment && l.amount == 30000)
    );
    assert!(legs.iter().all(|l| l.statement_id.is_some()));

    let mut bad = input(&a, -100, TxnKind::Transfer, "2026-11-10", monthly(Some(10)));
    bad.to_account_id = Some(usd);
    assert!(
        fx.write(|tx| create(tx, &bad, d("2026-10-14"))).is_err(),
        "different currencies"
    );
    bad.to_account_id = Some(a.clone());
    assert!(
        fx.write(|tx| create(tx, &bad, d("2026-10-14"))).is_err(),
        "same account"
    );
}

#[test]
fn update_rewrites_only_future_planned() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let id = fx
        .write(|tx| {
            create(
                tx,
                &input(
                    &acc,
                    -1000,
                    TxnKind::Expense,
                    "2026-10-05",
                    monthly(Some(5)),
                ),
                d("2026-10-01"),
            )
        })
        .unwrap();
    // Today is Oct 14: the planned one of Oct 5 was left behind (overdue), the one of Nov 5 is
    // future.
    let mut p = empty_patch(&id);
    p.amount = Some(-1200);
    p.description = Some("Aluguel reajustado".into());
    fx.write(|tx| update(tx, &p, d("2026-10-14"))).unwrap();
    let g = generated(&fx, &id);
    assert_eq!(g[0], ("2026-10-05".into(), -1000, "planned".into()));
    assert!(g[1..].iter().all(|(_, a, _)| *a == -1200));
    assert_eq!(
        dates(&fx, &id),
        vec!["2026-10-05", "2026-11-05", "2026-12-05"]
    );
    let desc: String = fx
        .conn()
        .query_row(
            "SELECT description FROM txn WHERE recurrence_id = ?1 AND date = '2026-11-05'",
            [&id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(desc, "Aluguel reajustado");
    // Without update_planned, the planned ones stay as they are.
    let mut p = empty_patch(&id);
    p.amount = Some(-1500);
    p.update_planned = Some(false);
    fx.write(|tx| update(tx, &p, d("2026-10-14"))).unwrap();
    assert!(generated(&fx, &id)[1..].iter().all(|(_, a, _)| *a == -1200));
    assert_eq!(get(fx.conn(), &id, d("2026-10-14")).unwrap().amount, -1500);

    // New rule: redoes the future planned ones on the new dates.
    let mut p = empty_patch(&id);
    p.spec = Some(monthly(Some(20)));
    fx.write(|tx| update(tx, &p, d("2026-10-14"))).unwrap();
    assert_eq!(
        generated(&fx, &id),
        vec![
            ("2026-10-05".into(), -1000, "planned".into()),
            ("2026-10-20".into(), -1500, "planned".into()),
            ("2026-11-20".into(), -1500, "planned".into()),
        ]
    );

    // Pausing removes the future ones; reactivating does not generate the paused period.
    let mut p = empty_patch(&id);
    p.active = Some(false);
    fx.write(|tx| update(tx, &p, d("2026-10-14"))).unwrap();
    assert_eq!(dates(&fx, &id), vec!["2026-10-05"]);
    assert_eq!(materialize(&mut fx, "2026-12-01"), 0, "paused");
    assert!(
        get(fx.conn(), &id, d("2026-12-01"))
            .unwrap()
            .next_dates
            .is_empty()
    );
    let mut p = empty_patch(&id);
    p.active = Some(true);
    fx.write(|tx| update(tx, &p, d("2027-01-10"))).unwrap();
    assert_eq!(
        dates(&fx, &id),
        vec!["2026-10-05", "2027-01-20", "2027-02-20"]
    );
}

#[test]
fn delete_with_and_without_planned() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let mk = |fx: &mut Fx| {
        fx.write(|tx| {
            create(
                tx,
                &input(
                    &acc,
                    -1000,
                    TxnKind::Expense,
                    "2026-10-05",
                    monthly(Some(5)),
                ),
                d("2026-10-01"),
            )
        })
        .unwrap()
    };
    let id = mk(&mut fx);
    // October's happened.
    fx.conn()
        .execute(
            "UPDATE txn SET status = 'cleared' WHERE recurrence_id = ?1 AND date = '2026-10-05'",
            [&id],
        )
        .unwrap();
    let n = fx
        .write(|tx| delete(tx, &id, true, d("2026-10-14")))
        .unwrap();
    assert_eq!(n, 1);
    assert_eq!(fx.count("SELECT count(*) FROM txn"), 1);
    assert_eq!(
        fx.count("SELECT count(*) FROM txn WHERE recurrence_id IS NOT NULL"),
        0
    );
    assert!(get(fx.conn(), &id, d("2026-10-14")).is_err());

    let id = mk(&mut fx);
    let n = fx
        .write(|tx| delete(tx, &id, false, d("2026-10-14")))
        .unwrap();
    assert_eq!(n, 0);
    assert_eq!(
        fx.count("SELECT count(*) FROM txn WHERE status = 'planned'"),
        2
    );
}

#[test]
fn undo_of_creation_removes_rule_and_planned() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let (id, cs) = fx
        .write_cs(|tx| {
            create(
                tx,
                &input(
                    &acc,
                    -1000,
                    TxnKind::Expense,
                    "2026-10-05",
                    monthly(Some(5)),
                ),
                d("2026-10-01"),
            )
        })
        .unwrap();
    assert_eq!(dates(&fx, &id).len(), 2);
    fx.undo(&cs.unwrap()).unwrap();
    assert_eq!(fx.count("SELECT count(*) FROM recurrence"), 0);
    assert_eq!(fx.count("SELECT count(*) FROM txn"), 0);
}

#[test]
fn unlock_routines_are_system_changes_and_quiet_when_idle() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let card = fx.card("Cartão", 3, 10, None);
    fx.write(|tx| {
        create(
            tx,
            &input(
                &acc,
                -1000,
                TxnKind::Expense,
                "2026-10-05",
                monthly(Some(5)),
            ),
            d("2026-10-01"),
        )
    })
    .unwrap();
    let plan = crate::cards::InstallmentInput {
        account_id: card,
        description: "Sofá".into(),
        total_amount: 3000,
        installments: 3,
        purchase_date: "2026-10-01".into(),
        category_id: None,
        member_id: None,
        tags: None,
    };
    fx.write(|tx| crate::cards::create_installments(tx, &plan, d("2026-10-01")))
        .unwrap();
    let before = crate::changes::list(fx.conn(), &ChangeListArgs::default())
        .unwrap()
        .len();
    let (created, cleared) = run_unlock_routines(fx.db.conn_mut(), d("2026-11-02")).unwrap();
    assert_eq!(
        (created, cleared),
        (1, 1),
        "planned on Dec 5 and installment on Nov 1"
    );
    let list = crate::changes::list(fx.conn(), &ChangeListArgs::default()).unwrap();
    assert_eq!(list.len(), before + 2);
    let reasons: Vec<&str> = list.iter().take(2).map(|c| c.reason.as_str()).collect();
    assert!(reasons.contains(&"Lançamentos previstos de recorrências"));
    assert!(reasons.contains(&"Parcelas do cartão efetivadas"));
    assert!(
        list.iter()
            .take(2)
            .all(|c| c.author.kind == AuthorKind::System)
    );
    // Running again on the same day writes nothing.
    assert_eq!(
        run_unlock_routines(fx.db.conn_mut(), d("2026-11-02")).unwrap(),
        (0, 0)
    );
    assert_eq!(
        crate::changes::list(fx.conn(), &ChangeListArgs::default())
            .unwrap()
            .len(),
        before + 2
    );
}

#[test]
fn preview_and_listing() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let p = preview(&PreviewArgs {
        spec: RuleSpec {
            freq: Freq::Weekly,
            interval: Some(2),
            by_month_day: None,
            by_day: Some(vec!["mo".into()]),
            by_month: None,
            count: Some(3),
        },
        start_date: "2026-10-14".into(),
        count: None,
        end_date: None,
    })
    .unwrap();
    assert_eq!(p.rrule, "FREQ=WEEKLY;INTERVAL=2;BYDAY=MO;COUNT=3");
    assert_eq!(p.human, "a cada 2 semanas às segundas, 3 vezes");
    // Weeks counted from the week of the start date (the one of Oct 12).
    assert_eq!(p.dates, vec!["2026-10-26", "2026-11-09", "2026-11-23"]);
    let mut i = input(
        &acc,
        500000,
        TxnKind::Income,
        "2026-10-05",
        monthly(Some(5)),
    );
    i.description = "Salário".into();
    fx.write(|tx| create(tx, &i, d("2026-10-14"))).unwrap();
    let mut i2 = input(
        &acc,
        3990,
        TxnKind::Expense,
        "2026-10-14",
        RuleSpec {
            freq: Freq::Yearly,
            ..monthly(None)
        },
    );
    i2.description = "Anuidade".into();
    let yearly = fx.write(|tx| create(tx, &i2, d("2026-10-14"))).unwrap();
    let mut p = empty_patch(&yearly);
    p.active = Some(false);
    fx.write(|tx| update(tx, &p, d("2026-10-14"))).unwrap();
    let all = list(fx.conn(), &RecurrenceListArgs::default(), d("2026-10-14")).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].description, "Salário", "active first");
    assert_eq!(all[0].amount, 500000);
    assert_eq!(all[1].human, "todo ano em 14 de outubro");
    assert_eq!(all[1].monthly_equivalent, -333);
    let active = list(
        fx.conn(),
        &RecurrenceListArgs {
            account_id: Some(acc.clone()),
            active: Some(true),
        },
        d("2026-10-14"),
    )
    .unwrap();
    assert_eq!(active.len(), 1);
}
