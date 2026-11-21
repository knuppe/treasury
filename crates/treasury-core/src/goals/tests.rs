//! Tests for goals and the emergency reserve. Projections depend on today, so the
//! dates in the data are relative to the current month.

use chrono::{Datelike, NaiveDate};

use super::*;
use crate::reports::fixture::{self, cat, expense, new_db, transfer, write};
use crate::time::add_months;
use crate::transactions::{TxnKind, TxnStatus};

fn month_start(today: NaiveDate, delta: i32) -> NaiveDate {
    add_months(today.with_day(1).unwrap(), delta)
}

fn day(today: NaiveDate, delta: i32, d: u32) -> String {
    fmt_date(month_start(today, delta).with_day(d).unwrap())
}

fn input(name: &str, target: i64) -> GoalInput {
    GoalInput {
        name: name.into(),
        kind: GoalKind::Travel,
        target_amount: target,
        currency: None,
        target_date: None,
        initial_amount: None,
        monthly_contribution: None,
        account_ids: None,
        notes: None,
        real_return: None,
    }
}

/// Reserve opened 8 months ago with R$ 2,000; contributions of R$ 500 in the last 6 full
/// months, a withdrawal of R$ 200 2 months ago and a contribution of R$ 100 in the current
/// month (outside the average).
fn savings_scenario(t: &mut fixture::TestDb, today: NaiveDate) -> (Id, Id) {
    write(&mut t.db, |tx| {
        let open = fmt_date(month_start(today, -8));
        let checking = fixture::account(tx, "Corrente", "checking", "BRL", 0, &open);
        let savings = fixture::account(tx, "Reserva", "savings", "BRL", 200_000, &open);
        for m in -6..=-1 {
            transfer(
                tx,
                &checking,
                &savings,
                &day(today, m, 10),
                50_000,
                TxnKind::Transfer,
                TxnStatus::Cleared,
            );
        }
        transfer(
            tx,
            &savings,
            &checking,
            &day(today, -2, 15),
            20_000,
            TxnKind::Transfer,
            TxnStatus::Cleared,
        );
        transfer(
            tx,
            &checking,
            &savings,
            &day(today, 0, 1),
            10_000,
            TxnKind::Transfer,
            TxnStatus::Cleared,
        );
        // Planned does not count in the balance.
        transfer(
            tx,
            &checking,
            &savings,
            &day(today, 2, 1),
            99_000,
            TxnKind::Transfer,
            TxnStatus::Planned,
        );
        fixture::fx_rate(tx, "USD", "BRL", &open, "5");
        Ok((checking, savings))
    })
}

#[test]
fn goal_progress_projection_and_pace() {
    let today = engine::today();
    let mut t = new_db();
    let (_, savings) = savings_scenario(&mut t, today);

    let mut g = input("Viagem", 1_000_000);
    g.initial_amount = Some(100_000);
    g.target_date = Some(fmt_date(add_months(today, 10)));
    g.account_ids = Some(vec![savings.clone(), savings.clone()]);
    let id = write(&mut t.db, |tx| create(tx, &g));
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!(v.goal.account_ids, vec![savings.clone()]);
    assert_eq!(v.goal.currency, "BRL");
    // 1,000 + 2,000 + 6 × 500 − 200 + 100.
    assert_eq!(v.current_amount, 590_000);
    assert!((v.progress - 0.59).abs() < 1e-12);
    assert_eq!(v.remaining, 410_000);
    assert_eq!(v.monthly_needed, Some(41_000));
    // (6 × 500 − 200) ÷ 6 = 466.666… → 466.67.
    assert_eq!(v.avg_monthly_contribution, 46_667);
    // 4,100 ÷ 466.67 = 8.8 → 9 months.
    assert_eq!(v.projected_date, Some(fmt_date(add_months(today, 9))));
    assert_eq!(v.on_track, Some(true));
    assert!(v.missing_fx.is_empty());

    // Goal in USD over the BRL account (inverse and old exchange rate: estimated).
    let mut g = input("Viagem em dólar", 200_000);
    g.currency = Some("usd".into());
    g.monthly_contribution = Some(10_000);
    g.account_ids = Some(vec![savings.clone()]);
    let id = write(&mut t.db, |tx| create(tx, &g));
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!(v.goal.currency, "USD");
    assert_eq!(v.current_amount, 98_000);
    assert!(v.estimated);
    // (6 × 100 − 40) ÷ 6 = 93.33 USD.
    assert_eq!(v.avg_monthly_contribution, 9_333);
    assert_eq!(v.monthly_needed, None);
    assert_eq!(v.on_track, None);
    assert_eq!(v.projected_date, Some(fmt_date(add_months(today, 11))));

    // No accounts: the projection uses the planned contribution.
    let mut g = input("Notebook", 120_000);
    g.kind = GoalKind::Purchase;
    g.monthly_contribution = Some(10_000);
    g.target_date = Some(fmt_date(add_months(today, 6)));
    let id = write(&mut t.db, |tx| create(tx, &g));
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!((v.current_amount, v.avg_monthly_contribution), (0, 0));
    assert_eq!(v.monthly_needed, Some(20_000));
    assert_eq!(v.projected_date, Some(fmt_date(add_months(today, 12))));
    assert_eq!(v.on_track, Some(false));

    // Already reached.
    let mut g = input("Presente", 50_000);
    g.initial_amount = Some(60_000);
    g.target_date = Some(fmt_date(add_months(today, -1)));
    let id = write(&mut t.db, |tx| create(tx, &g));
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!((v.remaining, v.monthly_needed), (0, Some(0)));
    assert!((v.progress - 1.2).abs() < 1e-12);
    assert_eq!(v.projected_date, Some(fmt_date(today)));
    assert_eq!(v.on_track, Some(true));

    // Past target date with no pace: needs everything now and does not get there.
    let mut g = input("Atrasada", 100_000);
    g.target_date = Some(fmt_date(add_months(today, -1)));
    let id = write(&mut t.db, |tx| create(tx, &g));
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!(v.monthly_needed, Some(100_000));
    assert_eq!((v.projected_date, v.on_track), (None, Some(false)));
}

#[test]
fn goal_crud_validation_and_links() {
    let today = engine::today();
    let mut t = new_db();
    let (checking, savings) = savings_scenario(&mut t, today);
    let err = |t: &mut fixture::TestDb, g: GoalInput| {
        crate::changes::record(
            t.db.conn_mut(),
            &crate::changes::Author::user(),
            "teste",
            |tx| create(tx, &g),
        )
        .unwrap_err()
        .code()
    };
    assert_eq!(err(&mut t, input("  ", 100)), "invalid");
    assert_eq!(err(&mut t, input("x", 0)), "invalid");
    let mut g = input("x", 100);
    g.target_date = Some("2026-13-01".into());
    assert_eq!(err(&mut t, g), "invalid");
    let mut g = input("x", 100);
    g.account_ids = Some(vec!["nao-existe".into()]);
    assert_eq!(err(&mut t, g), "not_found");
    let mut g = input("x", 100);
    g.monthly_contribution = Some(-1);
    assert_eq!(err(&mut t, g), "invalid");

    let mut g = input("Reserva", 1_000_000);
    g.kind = GoalKind::Emergency;
    g.account_ids = Some(vec![savings.clone()]);
    g.target_date = Some("2030-01-01".into());
    g.notes = Some("  ".into());
    let id = write(&mut t.db, |tx| create(tx, &g));
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!(v.goal.notes, None);
    assert_eq!(v.goal.kind, GoalKind::Emergency);

    let patch = |p: serde_json::Value| -> GoalPatch {
        let mut base = serde_json::json!({ "id": id });
        base.as_object_mut()
            .unwrap()
            .extend(p.as_object().unwrap().clone());
        serde_json::from_value(base).unwrap()
    };
    // An empty patch writes nothing.
    let rec = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::user(),
        "teste",
        |tx| update(tx, &patch(serde_json::json!({}))),
    )
    .unwrap();
    assert!(rec.change_set_id.is_none());

    write(&mut t.db, |tx| {
        update(
            tx,
            &patch(serde_json::json!({
                "name": "Reserva da família",
                "account_ids": [checking, savings],
                "target_date": null,
                "monthly_contribution": 30_000,
                "notes": "seis meses"
            })),
        )
    });
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!(v.goal.name, "Reserva da família");
    let mut expected = vec![checking.clone(), savings.clone()];
    expected.sort();
    assert_eq!(v.goal.account_ids, expected);
    assert_eq!(v.goal.target_date, None);
    assert_eq!(v.goal.monthly_contribution, Some(30_000));
    assert_eq!(v.goal.notes.as_deref(), Some("seis meses"));

    write(&mut t.db, |tx| {
        update(
            tx,
            &patch(serde_json::json!({"account_ids": [], "archived": true})),
        )
    });
    let v = get(t.db.conn(), &id).unwrap();
    assert!(v.goal.account_ids.is_empty() && v.goal.archived);
    assert!(list(t.db.conn(), false).unwrap().is_empty());
    assert_eq!(list(t.db.conn(), true).unwrap().len(), 1);

    let e = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::user(),
        "teste",
        |tx| update(tx, &patch(serde_json::json!({"target_amount": -5}))),
    )
    .unwrap_err();
    assert_eq!(e.code(), "invalid");

    write(&mut t.db, |tx| delete(tx, &id));
    assert_eq!(get(t.db.conn(), &id).unwrap_err().code(), "not_found");
    let n: i64 =
        t.db.conn()
            .query_row("SELECT count(*) FROM goal_account", [], |r| r.get(0))
            .unwrap();
    assert_eq!(n, 0);
}

#[test]
fn emergency_reserve_from_essential_spending() {
    let today = engine::today();
    let mut t = new_db();
    let (checking, savings) = write(&mut t.db, |tx| {
        let open = fmt_date(month_start(today, -4));
        let checking = fixture::account(tx, "Corrente", "checking", "BRL", 2_000_000, &open);
        let savings = fixture::account(tx, "Poupança", "savings", "BRL", 1_000_000, &open);
        let old_cash = fixture::account(tx, "Cofre antigo", "cash", "BRL", 50_000, &open);
        tx.execute("UPDATE account SET archived = 1 WHERE id = ?1", [&old_cash])?;
        let card = fixture::card(tx, "Cartão", 25, 5);
        let (aluguel, mercado) = (
            cat(tx, "Moradia › Aluguel"),
            cat(tx, "Alimentação › Mercado"),
        );
        let (rest, viagens) = (
            cat(tx, "Alimentação › Restaurantes"),
            cat(tx, "Lazer › Viagens"),
        );
        for m in -3..=-1 {
            expense(
                tx,
                &checking,
                &day(today, m, 5),
                -200_000,
                Some(&aluguel),
                None,
                "aluguel",
            );
            expense(
                tx,
                &checking,
                &day(today, m, 6),
                -100_000,
                Some(&mercado),
                None,
                "mercado",
            );
            expense(
                tx,
                &checking,
                &day(today, m, 7),
                -30_000,
                Some(&rest),
                None,
                "restaurante",
            );
            expense(
                tx,
                &checking,
                &day(today, m, 8),
                -70_000,
                Some(&viagens),
                None,
                "viagem",
            );
        }
        // The current (incomplete) month does not enter the average, but affects the balance.
        expense(
            tx,
            &checking,
            &day(today, 0, 1),
            -200_000,
            Some(&aluguel),
            None,
            "aluguel",
        );
        // Card debt does not count as reserve.
        expense(
            tx,
            &card,
            &day(today, 0, 1),
            -40_000,
            Some(&viagens),
            None,
            "cartão",
        );
        Ok((checking, savings))
    });
    let _ = checking;
    let c = t.db.conn();
    let r = emergency_reserve(
        c,
        &EmergencyReserveArgs {
            target_months: None,
            months_back: None,
            include_annual: None,
        },
    )
    .unwrap();
    // First transaction 3 months ago: 3 months analyzed, not 6.
    assert_eq!(r.months_analyzed, 3);
    assert_eq!(r.months.len(), 3);
    assert_eq!(
        r.months[0],
        crate::reports::engine::current_month_of(month_start(today, -3))
    );
    assert_eq!(r.avg_essential_monthly, 300_000);
    assert_eq!(r.avg_total_expense_monthly, 400_000);
    assert_eq!((r.target_months, r.target_amount), (6, 1_800_000));
    // Checking 20,000 − 3 × 4,000 − 2,000 + savings 10,000; the archived vault stays out.
    assert_eq!(r.current_amount, 1_600_000);
    assert!((r.coverage_months.unwrap() - 16.0 / 3.0).abs() < 1e-12);
    assert_eq!(r.goal_id, None);

    let r = emergency_reserve(
        c,
        &EmergencyReserveArgs {
            target_months: Some(12),
            months_back: Some(2),
            include_annual: None,
        },
    )
    .unwrap();
    assert_eq!(
        (r.months_analyzed, r.avg_essential_monthly, r.target_amount),
        (2, 300_000, 3_600_000)
    );

    // With a reserve goal, the current value is the goal's.
    let mut g = input("Reserva", 1_800_000);
    g.kind = GoalKind::Emergency;
    g.initial_amount = Some(100_000);
    g.account_ids = Some(vec![savings]);
    let id = write(&mut t.db, |tx| create(tx, &g));
    let r = emergency_reserve(
        t.db.conn(),
        &EmergencyReserveArgs {
            target_months: None,
            months_back: None,
            include_annual: None,
        },
    )
    .unwrap();
    assert_eq!(r.goal_id.as_deref(), Some(id.as_str()));
    assert_eq!(r.current_amount, 1_100_000);
    assert!((r.coverage_months.unwrap() - 11.0 / 3.0).abs() < 1e-12);

    // No transactions: nothing analyzed.
    let empty = new_db();
    let r = emergency_reserve(
        empty.db.conn(),
        &EmergencyReserveArgs {
            target_months: None,
            months_back: None,
            include_annual: None,
        },
    )
    .unwrap();
    assert_eq!(
        (
            r.months_analyzed,
            r.avg_essential_monthly,
            r.coverage_months
        ),
        (0, 0, None)
    );
}

/// Errors, retirement notes and INSS rules come out in the installation's language.
#[test]
fn texts_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let mut t = new_db();
    set_current(Locale::En);
    let e = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::user(),
        "teste",
        |tx| create(tx, &input("x", 0)),
    )
    .unwrap_err()
    .message();
    let sim = retirement::simulate(
        &retirement::Plan {
            current_age: Some(40),
            retirement_age: Some(60),
            current_wealth: Some(100_000_000),
            simulations: Some(100),
            ..Default::default()
        },
        0,
    )
    .unwrap();
    let inss = inss::estimate(
        &inss::Input {
            sex: inss::Sex::Male,
            birth_date: Some("1971-10-01".into()),
            contribution_years: 30.0,
            contribution_at_reform: None,
            average_salary: 500_000,
            contribute_until_age: None,
        },
        None,
        65,
        NaiveDate::from_ymd_opt(2026, 10, 18).unwrap(),
    )
    .unwrap();
    set_current(Locale::PtBr);
    assert_eq!(e, "the goal's target amount must be positive");
    assert!(
        sim.notes[0]
            .starts_with("Today's real values; real return of 4.0% a year with 10% volatility"),
        "{:?}",
        sim.notes
    );
    let prog = inss
        .rules
        .iter()
        .find(|r| r.id == "progressive_age")
        .unwrap();
    assert_eq!(prog.name, "Transition by progressive minimum age");
    assert!(
        prog.requirement
            .contains("an age of 64 years and 6 months in 2026"),
        "{}",
        prog.requirement
    );
}

#[test]
fn goal_with_real_return_uses_the_annuity_and_the_growth() {
    let mut t = new_db();
    let today = engine::today();
    // R$ 1 million in 20 years, from zero, with 4% real per year.
    let mut g = input("Primeiro milhão", 100_000_000);
    g.target_date = Some(fmt_date(add_months(today, 240)));
    g.real_return = Some(0.04);
    let id = write(&mut t.db, |tx| create(tx, &g));
    let v = get(t.db.conn(), &id).unwrap();
    let i = (1.04f64).powf(1.0 / 12.0) - 1.0;
    let expected = (100_000_000.0 * i / ((1.0 + i).powi(240) - 1.0)).ceil() as i64;
    assert_eq!(v.monthly_needed, Some(expected));
    let growth = v.growth.as_ref().unwrap();
    assert_eq!(growth.linear_monthly_needed, Some(416_667));
    assert!(expected < 300_000, "{expected}");
    assert_eq!(growth.months, 240);
    let (c, r) = (
        growth.from_contributions.unwrap(),
        growth.from_returns.unwrap(),
    );
    assert_eq!(c, expected * 240);
    assert_eq!(c + r, 100_000_000);
    // No pace or planned contribution: zero does not grow, no projected date.
    assert_eq!(v.projected_date, None);
    assert_eq!(v.on_track, Some(false));

    // Contributing what is needed, it arrives on the date (in its month).
    let patch = |p: serde_json::Value| -> GoalPatch { serde_json::from_value(p).unwrap() };
    write(&mut t.db, |tx| {
        update(
            tx,
            &patch(serde_json::json!({"id": id, "monthly_contribution": expected})),
        )
    });
    let v = get(t.db.conn(), &id).unwrap();
    assert_eq!(v.projected_date, g.target_date);
    assert_eq!(v.on_track, Some(true));

    // A return outside the range is rejected; `null` goes back to the linear calculation.
    let bad = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::user(),
        "teste",
        |tx| {
            update(
                tx,
                &patch(serde_json::json!({"id": id, "real_return": 0.5})),
            )
        },
    )
    .unwrap_err();
    assert_eq!(bad.code(), "invalid");
    write(&mut t.db, |tx| {
        update(
            tx,
            &patch(serde_json::json!({"id": id, "real_return": null})),
        )
    });
    let v = get(t.db.conn(), &id).unwrap();
    assert!(v.growth.is_none());
    assert_eq!(v.monthly_needed, Some(416_667));
}

#[test]
fn guarded_money_grows_without_contributions() {
    // R$ 50 thousand saved and 6% real per year: the R$ 100 thousand goal arrives in 12 years
    // with no contribution.
    assert_eq!(annuity_payment(10_000_000, 5_000_000, 0.0, 12), 416_667);
    let i = (1.06f64).powf(1.0 / 12.0) - 1.0;
    assert_eq!(annuity_payment(10_000_000, 5_000_000, i, 12 * 12), 0);
    let m = months_to_reach(10_000_000, 5_000_000, 0, i).unwrap();
    assert!((142..=144).contains(&m), "{m}");
    assert_eq!(months_to_reach(10_000_000, 0, 0, i), None);
}

#[test]
fn annual_expenses_become_a_monthly_provision_in_the_reserve() {
    let mut t = new_db();
    let today = engine::today();
    write(&mut t.db, |tx| {
        let open = fmt_date(month_start(today, -14));
        let checking = fixture::account(tx, "Corrente", "checking", "BRL", 5_000_000, &open);
        let (aluguel, ipva, iptu, viagens) = (
            cat(tx, "Moradia › Aluguel"),
            cat(tx, "Transporte › IPVA e licenciamento"),
            cat(tx, "Moradia › IPTU"),
            cat(tx, "Lazer › Viagens"),
        );
        for m in -13..=-1 {
            expense(
                tx,
                &checking,
                &day(today, m, 5),
                -200_000,
                Some(&aluguel),
                None,
                "aluguel",
            );
        }
        // IPVA of R$ 2,400 in one month, IPTU of R$ 1,200 in two months, a trip of R$ 6,000.
        expense(
            tx,
            &checking,
            &day(today, -2, 10),
            -240_000,
            Some(&ipva),
            None,
            "ipva",
        );
        expense(
            tx,
            &checking,
            &day(today, -3, 10),
            -60_000,
            Some(&iptu),
            None,
            "iptu 1",
        );
        expense(
            tx,
            &checking,
            &day(today, -9, 10),
            -60_000,
            Some(&iptu),
            None,
            "iptu 2",
        );
        expense(
            tx,
            &checking,
            &day(today, -4, 10),
            -600_000,
            Some(&viagens),
            None,
            "viagem",
        );
        Ok(())
    });
    let args = |include: Option<bool>| EmergencyReserveArgs {
        target_months: Some(6),
        months_back: Some(6),
        include_annual: include,
    };
    let r = emergency_reserve(t.db.conn(), &args(None)).unwrap();
    assert!(r.annual.complete_year);
    let names: Vec<&str> = r.annual.items.iter().map(|i| i.category.as_str()).collect();
    assert_eq!(
        names,
        [
            "Lazer › Viagens",
            "Transporte › IPVA e licenciamento",
            "Moradia › IPTU"
        ]
    );
    assert_eq!(r.annual.items[2].months.len(), 2);
    // Essential provision: (2,400 + 1,200) ÷ 12 = R$ 300; with the trip, R$ 800.
    assert_eq!(
        (r.annual.essential_monthly, r.annual.monthly),
        (30_000, 80_000)
    );
    // In the 6 months: rent 2,000/month + IPVA 2,400 + IPTU 600 → essentials average R$ 2,500.
    assert_eq!(r.avg_essential_monthly, 250_000);
    assert_eq!(r.avg_essential_regular_monthly, 200_000);
    assert!(r.include_annual);
    assert_eq!(r.monthly_cost, 230_000);
    assert_eq!(r.target_amount, 1_380_000);

    let off = emergency_reserve(t.db.conn(), &args(Some(false))).unwrap();
    assert!(!off.include_annual);
    assert_eq!((off.monthly_cost, off.target_amount), (250_000, 1_500_000));

    // Less than a year of history: nothing detected, the previous calculation.
    let mut short = new_db();
    write(&mut short.db, |tx| {
        let open = fmt_date(month_start(today, -4));
        let checking = fixture::account(tx, "Corrente", "checking", "BRL", 0, &open);
        let ipva = cat(tx, "Transporte › IPVA e licenciamento");
        expense(
            tx,
            &checking,
            &day(today, -2, 10),
            -240_000,
            Some(&ipva),
            None,
            "ipva",
        );
        Ok(())
    });
    let r = emergency_reserve(short.db.conn(), &args(None)).unwrap();
    assert!(!r.annual.complete_year && r.annual.items.is_empty() && !r.include_annual);
    assert_eq!(r.monthly_cost, r.avg_essential_monthly);
}
