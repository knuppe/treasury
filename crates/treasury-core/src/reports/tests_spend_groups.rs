//! Where the income goes (`reports.spend_groups`) over the reference scenario (see
//! `fixture`), plus a contribution, a loan installment and savings linked to a goal.

use rusqlite::params;

use super::fixture::{Reference, account, cat, expense, new_db, seed_reference, transfer, write};
use super::spend_groups::{SpendGroupMonth, SpendGroupsArgs, spend_groups};
use crate::budget::{Basis, GroupTargets, group_targets, set_group_targets};
use crate::categories::SpendGroup;
use crate::ids::new_id;
use crate::time::now_utc;
use crate::transactions::{TxnKind, TxnStatus};

fn args(from: &str, to: &str, basis: Basis) -> SpendGroupsArgs {
    SpendGroupsArgs {
        from: from.into(),
        to: to.into(),
        basis: Some(basis),
        member_id: None,
    }
}

fn amount(m: &SpendGroupMonth, g: SpendGroup) -> i64 {
    m.groups.iter().find(|x| x.group == g).unwrap().amount
}

fn top(m: &SpendGroupMonth, g: SpendGroup) -> Vec<(String, i64)> {
    m.groups
        .iter()
        .find(|x| x.group == g)
        .unwrap()
        .top
        .iter()
        .map(|c| (c.label.clone(), c.amount))
        .collect()
}

/// Reference scenario + contribution of 500 (Oct 7) and loan installment of 300 (Oct 9).
fn scenario() -> (super::fixture::TestDb, Reference) {
    let mut t = new_db();
    let r = seed_reference(&mut t.db);
    write(&mut t.db, |tx| {
        let broker = account(tx, "Corretora", "brokerage", "BRL", 0, "2026-01-01");
        transfer(
            tx,
            &r.checking,
            &broker,
            "2026-10-07",
            50_000,
            TxnKind::Investment,
            TxnStatus::Cleared,
        );
        // The redemption coming back does not reduce what was saved.
        transfer(
            tx,
            &broker,
            &r.checking,
            "2026-10-26",
            20_000,
            TxnKind::Investment,
            TxnStatus::Cleared,
        );
        let emprestimo = cat(tx, "Pagamentos › Empréstimos");
        expense(
            tx,
            &r.checking,
            "2026-10-09",
            -30_000,
            Some(&emprestimo),
            Some(&r.carlos),
            "Parcela do empréstimo",
        );
        Ok(())
    });
    (t, r)
}

#[test]
fn the_month_by_group_with_contributions_and_without_ordinary_transfers() {
    let (t, _) = scenario();
    let rep = spend_groups(t.db.conn(), &args("2026-10", "2026-10", Basis::Accrual)).unwrap();
    assert_eq!(rep.currency, "BRL");
    assert_eq!(rep.months.len(), 1);
    let oct = &rep.months[0];
    assert_eq!(oct.month, "2026-10");
    assert_eq!(oct.income, 1_020_000);
    let order: Vec<SpendGroup> = oct.groups.iter().map(|g| g.group).collect();
    assert_eq!(order, SpendGroup::ALL);
    // Rent 3,000 + Energy 250 + Groceries (split share) 450.
    assert_eq!(amount(oct, SpendGroup::Essential), 370_000);
    assert_eq!(amount(oct, SpendGroup::Debt), 30_000);
    // Only the contribution: the transfer to savings (no goal) stays out, and the redemption does
    // not reduce it.
    assert_eq!(amount(oct, SpendGroup::Future), 50_000);
    assert_eq!(
        top(oct, SpendGroup::Future),
        [("Aportes".to_string(), 50_000)]
    );
    // Leisure 750 (USD converted; JPY with no exchange rate stays out), Restaurants 170 (with the
    // refund), Pet food 150 and 40 uncategorized.
    assert_eq!(amount(oct, SpendGroup::NonEssential), 111_000);
    assert_eq!(oct.spending, 370_000 + 30_000 + 50_000 + 111_000);
    assert_eq!(
        top(oct, SpendGroup::Essential),
        [
            ("Moradia › Aluguel".to_string(), 300_000),
            ("Alimentação › Mercado".to_string(), 45_000),
            ("Moradia › Energia".to_string(), 25_000),
        ]
    );
    assert_eq!(
        top(oct, SpendGroup::NonEssential),
        [
            ("Lazer › Viagens".to_string(), 50_000),
            ("Lazer › Passeios".to_string(), 25_000),
            ("Alimentação › Restaurantes".to_string(), 17_000),
        ]
    );
    let essential = oct
        .groups
        .iter()
        .find(|g| g.group == SpendGroup::Essential)
        .unwrap();
    let share = essential.share_of_income.unwrap();
    assert!((share - 370_000.0 / 1_020_000.0).abs() < 1e-9, "{share}");
    assert_eq!(rep.targets, GroupTargets::default());
    // One-month period: the total is the month itself.
    assert_eq!(rep.total.month, "total");
    assert_eq!(rep.total.spending, oct.spending);
}

#[test]
fn transfers_to_goal_accounts_count_as_future_and_the_period_adds_up() {
    let (mut t, r) = scenario();
    write(&mut t.db, |tx| {
        let goal = new_id();
        tx.execute(
            "INSERT INTO goal (id, name, kind, target_amount, currency, created_at, updated_at)
             VALUES (?1, 'Reserva', 'emergency', 1000000, 'BRL', ?2, ?2)",
            params![goal, now_utc()],
        )?;
        tx.execute(
            "INSERT INTO goal_account (goal_id, account_id) VALUES (?1, ?2)",
            params![goal, r.savings],
        )?;
        Ok(())
    });
    let rep = spend_groups(t.db.conn(), &args("2026-09", "2026-10", Basis::Accrual)).unwrap();
    let (sep, oct) = (&rep.months[0], &rep.months[1]);
    // The settled transfer of 1,000 to the goal's savings counts; the forecast 500 does not.
    assert_eq!(amount(oct, SpendGroup::Future), 150_000);
    assert_eq!(
        top(oct, SpendGroup::Future),
        [
            ("Transferências para metas".to_string(), 100_000),
            ("Aportes".to_string(), 50_000),
        ]
    );
    // September: Rent 3,000 + Groceries 812.34 + Fuel 120 (card purchase, on the purchase
    // date); Streaming 80 non-essential.
    assert_eq!(sep.income, 1_000_000);
    assert_eq!(amount(sep, SpendGroup::Essential), 393_234);
    assert_eq!(amount(sep, SpendGroup::NonEssential), 8_000);
    assert_eq!(rep.total.income, 2_020_000);
    for g in SpendGroup::ALL {
        assert_eq!(
            amount(&rep.total, g),
            amount(sep, g) + amount(oct, g),
            "{g:?}"
        );
    }
    assert_eq!(rep.total.spending, sep.spending + oct.spending);
    // Over the period, the biggest categories sum both months.
    assert_eq!(
        top(&rep.total, SpendGroup::Essential)[0],
        ("Moradia › Aluguel".to_string(), 600_000)
    );
}

#[test]
fn cash_basis_and_member_filter_follow_the_other_reports() {
    let (t, r) = scenario();
    // Cash: the trip on the card (due Nov 5) leaves October; September's streaming (due
    // Oct 5) comes in.
    let cash = spend_groups(t.db.conn(), &args("2026-10", "2026-10", Basis::Cash)).unwrap();
    assert_eq!(
        amount(&cash.months[0], SpendGroup::NonEssential),
        111_000 - 50_000 + 8_000
    );
    // Ana: dividends, restaurant (with the refund) and the split pet food; the contribution with no
    // holder and Carlos's installment stay out.
    let ana = spend_groups(
        t.db.conn(),
        &SpendGroupsArgs {
            member_id: Some(r.ana.clone()),
            ..args("2026-10", "2026-10", Basis::Accrual)
        },
    )
    .unwrap();
    let m = &ana.months[0];
    assert_eq!(m.income, 20_000);
    assert_eq!(amount(m, SpendGroup::NonEssential), 17_000 + 15_000);
    assert_eq!(amount(m, SpendGroup::Essential), 0);
    assert_eq!(amount(m, SpendGroup::Debt), 0);
    assert_eq!(amount(m, SpendGroup::Future), 0);
    // With no income in the month, the income share is null.
    let nov = spend_groups(t.db.conn(), &args("2026-11", "2026-11", Basis::Cash)).unwrap();
    assert!(
        nov.months[0]
            .groups
            .iter()
            .all(|g| g.share_of_income.is_none())
    );
    assert!(amount(&nov.months[0], SpendGroup::NonEssential) > 0);
}

#[test]
fn group_targets_default_to_50_30_20_and_are_saved_in_the_vault() {
    let t = new_db();
    let conn = t.db.conn();
    let d = group_targets(conn).unwrap();
    assert_eq!(
        (d.essential, d.debt, d.future, d.non_essential),
        (Some(0.5), None, Some(0.2), Some(0.3))
    );
    let mine = GroupTargets {
        essential: Some(0.55),
        debt: Some(0.1),
        future: None,
        non_essential: Some(0.25),
    };
    assert_eq!(set_group_targets(conn, &mine).unwrap(), mine);
    assert_eq!(group_targets(conn).unwrap(), mine);
    let rep = spend_groups(conn, &args("2026-10", "2026-10", Basis::Accrual)).unwrap();
    assert_eq!(rep.targets, mine);
    let bad = GroupTargets {
        essential: Some(1.5),
        ..mine
    };
    assert!(matches!(
        set_group_targets(conn, &bad),
        Err(crate::Error::Invalid(_))
    ));
    assert_eq!(group_targets(conn).unwrap(), mine);
}

#[test]
fn the_api_serves_the_report_and_the_targets() {
    let dir = tempfile::tempdir().unwrap();
    let api = crate::api::Api::new(
        crate::paths::TreasuryPaths::with_root(dir.path().join(".t")).unwrap(),
    )
    .unwrap();
    api.call(
        "vault.create",
        serde_json::json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
    )
    .unwrap();
    let rep = api
        .call(
            "reports.spend_groups",
            serde_json::json!({"from": "2026-10", "to": "2026-10"}),
        )
        .unwrap();
    assert_eq!(rep["total"]["month"], "total");
    assert_eq!(rep["months"][0]["groups"][3]["group"], "non_essential");
    assert_eq!(rep["targets"]["essential"], 0.5);
    assert!(rep["targets"]["debt"].is_null());
    let saved = api
        .call(
            "budget.set_group_targets",
            serde_json::json!({"essential": 0.6, "debt": null, "future": 0.15, "non_essential": 0.25}),
        )
        .unwrap();
    assert_eq!(saved["essential"], 0.6);
    assert_eq!(
        api.call("budget.group_targets", serde_json::json!({}))
            .unwrap()["future"],
        0.15
    );
}
