use super::closing::{self, AccountCloseArgs, CloseBlocker};
use super::*;
use crate::recurrence::{Freq, RecurrenceInput, RuleSpec};
use crate::transactions::TxnKind;
use crate::transactions::testkit::{Fx, d};

const TODAY: &str = "2026-10-14";

fn args(id: &str, date: &str) -> AccountCloseArgs {
    AccountCloseArgs {
        id: id.into(),
        date: Some(date.into()),
    }
}

fn kinds(p: &closing::AccountClosePreview) -> Vec<&'static str> {
    p.blockers
        .iter()
        .map(|b| match b {
            CloseBlocker::FutureDate => "future",
            CloseBlocker::BeforeOpening { .. } => "before_opening",
            CloseBlocker::AlreadyClosed { .. } => "already_closed",
            CloseBlocker::NonZeroBalance { .. } => "balance",
            CloseBlocker::ActivityAfter { .. } => "after",
            CloseBlocker::OpenPositions { .. } => "positions",
            CloseBlocker::PendingInstallments { .. } => "installments",
        })
        .collect()
}

fn monthly_rent(account: &str, to: Option<&str>) -> RecurrenceInput {
    RecurrenceInput {
        description: "Aluguel da sala".into(),
        account_id: account.into(),
        amount: -150000,
        kind: if to.is_some() {
            TxnKind::Transfer
        } else {
            TxnKind::Expense
        },
        category_id: None,
        member_id: None,
        to_account_id: to.map(Into::into),
        spec: RuleSpec {
            freq: Freq::Monthly,
            interval: None,
            by_month_day: Some(20),
            by_day: None,
            by_month: None,
            count: None,
        },
        start_date: "2026-06-20".into(),
        end_date: None,
    }
}

#[test]
fn closing_needs_an_empty_account_on_the_date() {
    let mut fx = Fx::new();
    let acc = fx.checking("Banco Antigo", 80000);
    fx.txn(&acc, "2026-09-02", -30000, "Mercado do bairro");

    let p = closing::preview(fx.conn(), &args(&acc, "2026-09-30"), d(TODAY)).unwrap();
    assert_eq!(kinds(&p), vec!["balance"]);
    assert_eq!(p.balance, 50000);
    let err = fx
        .write(|tx| closing::close(tx, &args(&acc, "2026-09-30"), d(TODAY)))
        .unwrap_err();
    assert!(err.to_string().contains("500,00"), "{err}");

    let p = closing::preview(fx.conn(), &args(&acc, "2026-11-01"), d(TODAY)).unwrap();
    assert!(kinds(&p).contains(&"future"));
    let p = closing::preview(fx.conn(), &args(&acc, "2025-12-31"), d(TODAY)).unwrap();
    assert!(kinds(&p).contains(&"before_opening"));

    // Emptied on 30/09, but something was recorded after the date.
    fx.txn(&acc, "2026-09-30", -50000, "Saque do saldo final");
    fx.txn(&acc, "2026-10-05", -1000, "Tarifa");
    fx.txn(&acc, "2026-10-06", 1000, "Estorno da tarifa");
    let p = closing::preview(fx.conn(), &args(&acc, "2026-09-30"), d(TODAY)).unwrap();
    assert_eq!(
        p.blockers,
        vec![CloseBlocker::ActivityAfter {
            count: 2,
            last_date: "2026-10-06".into()
        }]
    );
    let p = closing::preview(fx.conn(), &args(&acc, "2026-10-06"), d(TODAY)).unwrap();
    assert!(p.blockers.is_empty(), "{:?}", p.blockers);
}

#[test]
fn closing_pauses_recurrences_removes_planned_and_blocks_later_entries() {
    let mut fx = Fx::new();
    let acc = fx.checking("Banco Antigo", 0);
    let other = fx.checking("Banco Novo", 0);
    let rent = fx
        .write(|tx| crate::recurrence::create(tx, &monthly_rent(&acc, None), d(TODAY)))
        .unwrap();
    let transfer = fx
        .write(|tx| crate::recurrence::create(tx, &monthly_rent(&other, Some(&acc)), d(TODAY)))
        .unwrap();
    let mut planned = Fx::input(&acc, "2026-10-01", -2000, "Previsto esquecido");
    planned.status = Some(crate::transactions::TxnStatus::Planned);
    fx.txn_with(planned).unwrap();
    let planned_in = |fx: &Fx, a: &str| {
        fx.conn()
            .query_row(
                "SELECT count(*) FROM txn WHERE account_id = ?1 AND status = 'planned'",
                [a],
                |r| r.get::<_, i64>(0),
            )
            .unwrap()
    };
    assert!(planned_in(&fx, &acc) > 2);
    let other_before = planned_in(&fx, &other);
    assert!(other_before > 0);

    let p = closing::preview(fx.conn(), &args(&acc, TODAY), d(TODAY)).unwrap();
    assert!(p.blockers.is_empty(), "{:?}", p.blockers);
    assert_eq!(p.recurrences.len(), 2);
    // Both legs of each planned transfer leave.
    assert_eq!(p.planned.len() as i64, planned_in(&fx, &acc) + other_before);

    let res = fx
        .write(|tx| closing::close(tx, &args(&acc, TODAY), d(TODAY)))
        .unwrap();
    assert_eq!(res.recurrences_paused, 2);
    assert_eq!(res.planned_removed, p.planned.len() as i64);
    assert_eq!(planned_in(&fx, &acc), 0);
    assert_eq!(planned_in(&fx, &other), 0);
    for r in [&rent, &transfer] {
        let rec = crate::recurrence::get(fx.conn(), r, d(TODAY)).unwrap();
        assert!(!rec.active);
        assert_eq!(rec.end_date.as_deref(), Some(TODAY));
    }
    let v = get(fx.conn(), &acc, d(TODAY)).unwrap();
    assert!(v.account.archived);
    assert_eq!(v.account.closed_on.as_deref(), Some(TODAY));
    assert!(v.balance_status.is_none());
    assert!(
        !list(fx.conn(), false, d(TODAY))
            .unwrap()
            .iter()
            .any(|a| a.account.id == acc)
    );

    // Nothing after the date, in any way: a new entry, a transfer, moving an old one.
    let late = Fx::input(&acc, "2026-10-15", -100, "Depois do encerramento");
    let err = fx.txn_with(late).unwrap_err();
    assert!(err.to_string().contains("14/10/2026"), "{err}");
    let old = fx.txn(&acc, "2026-10-14", -100, "Última tarifa");
    let moved = crate::transactions::TxnPatch {
        id: old.clone(),
        account_id: None,
        date: Some("2026-10-20".into()),
        amount: None,
        kind: None,
        status: None,
        description: None,
        payee: None,
        notes: None,
        category_id: None,
        member_id: None,
        splits: None,
        tags: None,
        transfer_account_id: None,
        transfer_amount: None,
        project_id: None,
        card_plastic_id: None,
        orig_amount: None,
        orig_currency: None,
    };
    assert!(
        fx.write(|tx| crate::transactions::update(tx, &moved, d(TODAY)))
            .is_err()
    );
    // Reactivated by hand, the recurrence generates nothing after the closing date.
    fx.write(|tx| {
        crate::recurrence::update(
            tx,
            &crate::recurrence::RecurrencePatch {
                id: rent.clone(),
                description: None,
                amount: None,
                category_id: None,
                member_id: None,
                spec: None,
                end_date: Some(None),
                active: Some(true),
                update_planned: None,
            },
            d(TODAY),
        )
    })
    .unwrap();
    assert_eq!(planned_in(&fx, &acc), 0);
}

#[test]
fn undoing_the_closing_brings_the_account_and_its_plans_back() {
    let mut fx = Fx::new();
    let acc = fx.checking("Banco Antigo", 0);
    let rent = fx
        .write(|tx| crate::recurrence::create(tx, &monthly_rent(&acc, None), d(TODAY)))
        .unwrap();
    let planned = |fx: &Fx| fx.count("SELECT count(*) FROM txn WHERE status = 'planned'");
    let before = planned(&fx);
    assert!(before > 0);
    let (_, cs) = fx
        .write_cs(|tx| closing::close(tx, &args(&acc, TODAY), d(TODAY)))
        .unwrap();
    assert_eq!(planned(&fx), 0);
    fx.undo(&cs.unwrap()).unwrap();
    let v = get(fx.conn(), &acc, d(TODAY)).unwrap();
    assert!(!v.account.archived);
    assert_eq!(v.account.closed_on, None);
    assert_eq!(planned(&fx), before);
    assert!(
        crate::recurrence::get(fx.conn(), &rent, d(TODAY))
            .unwrap()
            .active
    );
}

#[test]
fn reopening_clears_the_date_and_the_archive() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corretora Antiga", 0);
    fx.write(|tx| closing::close(tx, &args(&acc, "2026-08-31"), d(TODAY)))
        .unwrap();
    let p = closing::preview(fx.conn(), &args(&acc, TODAY), d(TODAY)).unwrap();
    assert_eq!(
        p.blockers,
        vec![CloseBlocker::AlreadyClosed {
            closed_on: "2026-08-31".into()
        }]
    );

    // Unarchiving a closed account reopens it.
    let mut patch = super::tests::empty_patch(&acc);
    patch.archived = Some(false);
    fx.write(|tx| update(tx, &patch, d(TODAY))).unwrap();
    let a = get(fx.conn(), &acc, d(TODAY)).unwrap().account;
    assert!(!a.archived && a.closed_on.is_none());

    // The patch closes (same checks) and reopens with `null`.
    let mut patch = super::tests::empty_patch(&acc);
    patch.closed_on = Some(Some("2026-09-15".into()));
    fx.write(|tx| update(tx, &patch, d(TODAY))).unwrap();
    let a = get(fx.conn(), &acc, d(TODAY)).unwrap().account;
    assert!(a.archived);
    assert_eq!(a.closed_on.as_deref(), Some("2026-09-15"));
    let mut patch = super::tests::empty_patch(&acc);
    patch.closed_on = Some(None);
    fx.write(|tx| update(tx, &patch, d(TODAY))).unwrap();
    let a = get(fx.conn(), &acc, d(TODAY)).unwrap().account;
    assert!(!a.archived && a.closed_on.is_none());
    fx.txn(&acc, TODAY, 100, "Depois de reabrir");

    // An account with money cannot close through the patch either.
    let mut patch = super::tests::empty_patch(&acc);
    patch.closed_on = Some(Some(TODAY.into()));
    assert!(fx.write(|tx| update(tx, &patch, d(TODAY))).is_err());
}

#[test]
fn a_card_with_installments_or_a_broker_with_assets_stays_open() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão Antigo", 3, 10, Some(500000));
    fx.write(|tx| {
        crate::cards::create_installments(
            tx,
            &crate::cards::InstallmentInput {
                account_id: card.clone(),
                description: "Geladeira".into(),
                total_amount: 300000,
                installments: 10,
                purchase_date: "2026-09-05".into(),
                category_id: None,
                member_id: None,
                tags: None,
            },
            d(TODAY),
        )
    })
    .unwrap();
    let p = closing::preview(fx.conn(), &args(&card, TODAY), d(TODAY)).unwrap();
    assert!(kinds(&p).contains(&"installments"), "{:?}", p.blockers);

    let broker = fx.account("Corretora Antiga", AccountKind::Brokerage, "BRL", 0);
    fx.conn()
        .execute_batch(&format!(
            "INSERT INTO asset (id, ticker, name, class, currency, created_at, updated_at)
             VALUES ('ativo-x', 'XPTO3', 'Xpto SA', 'stock_br', 'BRL', 'x', 'x');
             INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
             VALUES ('t1', 'ativo-x', '{broker}', '2026-03-10', 'buy', '100', '10', 'BRL', 'x', 'x');"
        ))
        .unwrap();
    let p = closing::preview(fx.conn(), &args(&broker, TODAY), d(TODAY)).unwrap();
    assert_eq!(
        p.blockers,
        vec![CloseBlocker::OpenPositions {
            assets: vec!["XPTO3".into()]
        }]
    );
    // Sold (or moved to another broker) on 01/10: the account closes from then on.
    fx.conn()
        .execute(
            &format!(
                "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
                 VALUES ('t2', 'ativo-x', '{broker}', '2026-10-01', 'sell', '100', '12', 'BRL', 'x', 'x')"
            ),
            [],
        )
        .unwrap();
    let p = closing::preview(fx.conn(), &args(&broker, "2026-09-30"), d(TODAY)).unwrap();
    assert!(kinds(&p).contains(&"after"), "{:?}", p.blockers);
    assert!(kinds(&p).contains(&"positions"), "{:?}", p.blockers);
    let p = closing::preview(fx.conn(), &args(&broker, "2026-10-01"), d(TODAY)).unwrap();
    assert!(p.blockers.is_empty(), "{:?}", p.blockers);
}

#[test]
fn archived_accounts_bring_no_upcoming_bill() {
    let mut fx = Fx::new();
    let acc = fx.checking("Banco Antigo", 0);
    let mut bill = Fx::input(&acc, "2026-10-18", -9000, "Conta de luz");
    bill.status = Some(crate::transactions::TxnStatus::Planned);
    fx.txn_with(bill).unwrap();
    let up = |fx: &Fx| {
        crate::reports::upcoming(fx.conn(), d(TODAY), 10)
            .unwrap()
            .len()
    };
    assert_eq!(up(&fx), 1);
    let mut patch = super::tests::empty_patch(&acc);
    patch.archived = Some(true);
    fx.write(|tx| update(tx, &patch, d(TODAY))).unwrap();
    assert_eq!(up(&fx), 0);
}
