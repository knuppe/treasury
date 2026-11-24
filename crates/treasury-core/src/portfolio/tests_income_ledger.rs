//! Income ledger: entitlement by the position on the record date, official notices first,
//! duplicates, amount and tax fixes, the rebuild and the automatic recording. Invented fund,
//! company and amounts.

use rusqlite::params;

use super::income_ledger::{self, EntryState, Fix, LedgerArgs, RebuildArgs};
use super::*;
use crate::assets::{self, AssetClass, AssetInput};
use crate::changes::{Author, record};
use crate::storage::Db;
use crate::time::parse_date;
use crate::vault::KDF_TEST;

fn db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
        crate::household::seed(tx, "Família", "BRL", &["Ana".into()])?;
        crate::categories::seed_defaults(tx)?;
        tx.execute(
            "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
             VALUES ('xp', 'Corretora Exemplo', 'brokerage', 'BRL', '2024-01-01', 'x', 'x')",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

fn asset(db: &mut Db, ticker: &str, class: AssetClass) -> String {
    let input = AssetInput {
        ticker: Some(ticker.into()),
        name: format!("{ticker} Exemplo"),
        official_name: None,
        account_id: None,
        class,
        market: None,
        currency: None,
        country: None,
        isin: None,
        cnpj: None,
        treasury_code: None,
        sector: None,
        segment: None,
        price_provider: None,
        provider_symbol: None,
        notes: None,
        fixed_income: None,
        pension: None,
    };
    record(db.conn_mut(), &Author::user(), "ativo", |tx| {
        assets::create(tx, &input)
    })
    .unwrap()
    .value
}

fn trade(db: &mut Db, asset_id: &str, date: &str, side: TradeSide, qty: &str, price: &str) {
    let t = TradeInput {
        asset_id: asset_id.into(),
        account_id: "xp".into(),
        date: date.into(),
        side,
        quantity: qty.into(),
        price: price.into(),
        fees: Some(0),
        taxes: None,
        fx_rate: None,
        day_trade: None,
        note_number: None,
        notes: None,
        cash_movement: Some(false),
        external_id: None,
    };
    record(db.conn_mut(), &Author::user(), "op", |tx| {
        records::create_trade(tx, &t)
    })
    .unwrap();
}

#[allow(clippy::too_many_arguments)]
fn notice(db: &Db, doc: i64, kind: &str, ex: &str, value: &str, pay: &str, exempt: Option<i64>) {
    db.conn()
        .execute(
            "INSERT INTO research_fund_notice (document_id, version, cnpj, ticker, kind, ex_date, value, pay_date, exempt,
                 delivered_at, source, fetched_at)
             VALUES (?1, 1, '11222333000144', 'FUND11', ?2, ?3, ?4, ?5, ?6, '01/01/2025 10:00', 'fundos_net', 'x')",
            params![doc, kind, ex, value, pay, exempt],
        )
        .unwrap();
}

fn income(
    db: &mut Db,
    asset_id: &str,
    kind: IncomeKind,
    ex: Option<&str>,
    pay: &str,
    gross: i64,
    tax: i64,
) -> String {
    let i = IncomeInput {
        asset_id: asset_id.into(),
        account_id: "xp".into(),
        kind,
        ex_date: ex.map(str::to_string),
        pay_date: pay.into(),
        quantity: None,
        gross,
        tax_withheld: Some(tax),
        fx_rate: None,
        status: None,
        cash_movement: Some(false),
    };
    record(db.conn_mut(), &Author::user(), "provento", |tx| {
        records::create_income(tx, &i)
    })
    .unwrap()
    .value
}

fn day(s: &str) -> chrono::NaiveDate {
    parse_date(s).unwrap()
}

fn ledger(db: &Db, today: &str) -> income_ledger::Ledger {
    income_ledger::build_on(db.conn(), &LedgerArgs::default(), day(today)).unwrap()
}

fn entry<'a>(
    l: &'a income_ledger::Ledger,
    ex: &str,
    kind: IncomeKind,
    state: EntryState,
) -> &'a income_ledger::LedgerEntry {
    l.entries
        .iter()
        .find(|e| e.ex_date.as_deref() == Some(ex) && e.kind == kind && e.state == state)
        .unwrap_or_else(|| panic!("{ex} {kind:?} {state:?}: {:#?}", l.entries))
}

/// A fund held from January to June: what each notice was worth for the position on its record
/// date, against what the family recorded (twice, with a wrong amount, with a typo in the year).
fn fund_scenario() -> (tempfile::TempDir, Db, String) {
    let (dir, mut db) = db();
    let f = asset(&mut db, "FUND11", AssetClass::Fii);
    trade(&mut db, &f, "2025-01-10", TradeSide::Buy, "100", "9");
    // Bought after February's record date: those 50 only count from March on.
    trade(&mut db, &f, "2025-03-05", TradeSide::Buy, "50", "9");
    trade(&mut db, &f, "2025-06-20", TradeSide::Sell, "150", "9.5");
    notice(
        &db,
        1,
        "fii_income",
        "2025-01-31",
        "0.10",
        "2025-02-14",
        Some(1),
    );
    notice(
        &db,
        8,
        "amortization",
        "2025-01-15",
        "0.05",
        "2025-01-20",
        None,
    );
    notice(
        &db,
        2,
        "fii_income",
        "2025-02-28",
        "0.055",
        "2025-03-14",
        Some(1),
    );
    notice(
        &db,
        3,
        "fii_income",
        "2025-03-31",
        "0.0682",
        "2025-04-15",
        Some(0),
    );
    notice(
        &db,
        4,
        "fii_income",
        "2025-04-30",
        "0.11",
        "2025-05-15",
        Some(1),
    );
    notice(
        &db,
        5,
        "amortization",
        "2025-04-30",
        "0.330560945",
        "2025-05-15",
        None,
    );
    // After the sale: nobody in the family is entitled.
    notice(
        &db,
        6,
        "fii_income",
        "2025-06-30",
        "0.11",
        "2025-07-15",
        Some(1),
    );
    // B3 has March too (checks the notice).
    db.conn()
        .execute(
            "INSERT INTO research_dividend (ticker, kind, ex_date, value, per_shares, pay_date, seq, source, fetched_at)
             VALUES ('FUND11', 'fii_income', '2025-03-31', '0.0682', 1, '2025-04-15', 0, 'b3_listados', 'x')",
            [],
        )
        .unwrap();
    // The calendar rounds February and reports the ex date: after Carnival, five days later.
    db.conn()
        .execute(
            "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
             VALUES ('y-feb', ?1, 'fii_income', '2025-03-05', '2025-03-14', '0.06', 'BRL', 0, 'yahoo', 'x')",
            [&f],
        )
        .unwrap();
    income(
        &mut db,
        &f,
        IncomeKind::FiiIncome,
        Some("2025-01-31"),
        "2025-02-14",
        1000,
        0,
    );
    income(
        &mut db,
        &f,
        IncomeKind::FiiIncome,
        Some("2025-02-28"),
        "2025-03-14",
        550,
        0,
    );
    // The same February, typed again from the calendar's suggestion.
    income(
        &mut db,
        &f,
        IncomeKind::FiiIncome,
        Some("2025-03-05"),
        "2025-03-14",
        600,
        0,
    );
    income(
        &mut db,
        &f,
        IncomeKind::FiiIncome,
        Some("2025-03-31"),
        "2025-04-15",
        1100,
        0,
    );
    // January again, with the year typed wrong.
    income(
        &mut db,
        &f,
        IncomeKind::Dividend,
        None,
        "2125-02-14",
        1000,
        0,
    );
    // Something no source describes.
    income(
        &mut db,
        &f,
        IncomeKind::FiiIncome,
        None,
        "2025-04-02",
        777,
        0,
    );
    (dir, db, f)
}

#[test]
fn the_ledger_pairs_each_record_with_the_official_notice() {
    let (_d, db, _) = fund_scenario();
    let l = ledger(&db, "2025-05-10");
    let jan = entry(&l, "2025-01-31", IncomeKind::FiiIncome, EntryState::Matches);
    assert_eq!(jan.quantity.as_deref(), Some("100"));
    assert_eq!(jan.expected_net, Some(1000));
    assert_eq!(jan.source.as_deref(), Some("fundos_net"));
    // The partial amortization nobody recorded.
    let amort = entry(
        &l,
        "2025-01-15",
        IncomeKind::Amortization,
        EntryState::Missing,
    );
    assert_eq!(amort.expected_gross, Some(500));
    assert_eq!(amort.fix, Some(Fix::Create));
    // February: the notice's 0.055 for the 100 units of the record date (not 0.06, not 150).
    let feb = entry(&l, "2025-02-28", IncomeKind::FiiIncome, EntryState::Matches);
    assert_eq!(feb.expected_gross, Some(550));
    assert_eq!(feb.amount_per_share.as_deref(), Some("0.055"));
    let check = feb.check.as_ref().unwrap();
    assert_eq!(check.reference, "fundos_net");
    assert!(
        check.sources.iter().any(|s| s.source == "yahoo"),
        "the calendar's ex date pairs across Carnival"
    );
    // The second February record repeats the first.
    let dup: Vec<_> = l
        .entries
        .iter()
        .filter(|e| e.state == EntryState::Duplicate)
        .collect();
    assert_eq!(dup.len(), 2, "{dup:#?}");
    assert!(dup.iter().all(|e| e.fix == Some(Fix::Delete)));
    assert!(
        dup.iter()
            .any(|e| e.recorded.as_ref().unwrap().pay_date == "2125-02-14")
    );
    // March: 150 units at 0.0682, taxed at 20% (the notice says not exempt), checked by B3.
    let mar = entry(
        &l,
        "2025-03-31",
        IncomeKind::FiiIncome,
        EntryState::ValueDiff,
    );
    assert_eq!(mar.expected_gross, Some(1023));
    assert_eq!(mar.expected_tax, Some(204));
    assert_eq!(mar.expected_net, Some(819));
    assert_eq!(mar.fix, Some(Fix::Update));
    assert_eq!(
        mar.check.as_ref().unwrap().status,
        pending::CheckStatus::Confirmed
    );
    // April is paid after "today"; June is after the sale.
    assert_eq!(
        entry(
            &l,
            "2025-04-30",
            IncomeKind::FiiIncome,
            EntryState::Upcoming
        )
        .expected_net,
        Some(1650)
    );
    assert!(
        !l.entries
            .iter()
            .any(|e| e.ex_date.as_deref() == Some("2025-06-30"))
    );
    let odd = l
        .entries
        .iter()
        .find(|e| e.state == EntryState::Unexpected)
        .unwrap();
    assert_eq!(odd.recorded.as_ref().unwrap().net, 777);
    assert_eq!(odd.fix, None);
    // Totals: what the notices say was paid (the unexpected record counts as it is).
    assert_eq!(l.received_base, 500 + 1000 + 550 + 819 + 777);
    assert_eq!(l.upcoming_base, 1650 + 4958);
    assert_eq!(l.counts.fixes, 4);
    assert!(!l.auto);
}

#[test]
fn the_rebuild_fixes_everything_in_one_change_and_turns_on_the_automatic_recording() {
    let (_d, mut db, f) = fund_scenario();
    let rep = income_ledger::rebuild_as(
        db.conn_mut(),
        &Author::user(),
        &RebuildArgs::default(),
        day("2025-05-10"),
    )
    .unwrap();
    assert_eq!((rep.created, rep.updated, rep.deleted), (1, 1, 2));
    assert!(rep.auto);
    assert!(rep.change_set_id.is_some());
    let l = ledger(&db, "2025-05-10");
    assert_eq!(l.counts.fixes, 0, "{:#?}", l.entries);
    assert_eq!(l.counts.matches, 4);
    assert!(l.auto);
    // History without a brokerage cash entry.
    let created: (Option<String>, String, Option<String>) = db
        .conn()
        .query_row(
            "SELECT txn_id, source, external_id FROM income_event WHERE asset_id = ?1 AND kind = 'amortization'",
            [&f],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(created.0, None);
    assert_eq!(created.1, "official");
    assert!(created.2.unwrap().starts_with("fundos_net:8:"));
    // After the payment date, the collection records April by itself (with cash), once.
    assert_eq!(
        income_ledger::auto_record(db.conn_mut(), day("2025-05-20")).unwrap(),
        2
    );
    let cash: i64 = db
        .conn()
        .query_row(
            "SELECT count(*) FROM income_event WHERE asset_id = ?1 AND ex_date = '2025-04-30' AND txn_id IS NOT NULL",
            [&f],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(cash, 2);
    assert_eq!(
        income_ledger::auto_record(db.conn_mut(), day("2025-05-21")).unwrap(),
        0
    );
    // Deleted by the family: it does not come back.
    let id: String = db
        .conn()
        .query_row(
            "SELECT id FROM income_event WHERE asset_id = ?1 AND ex_date = '2025-04-30' AND kind = 'amortization'",
            [&f],
            |r| r.get(0),
        )
        .unwrap();
    record(db.conn_mut(), &Author::user(), "apagar", |tx| {
        records::delete_income(tx, &id)
    })
    .unwrap();
    assert_eq!(
        income_ledger::auto_record(db.conn_mut(), day("2025-05-22")).unwrap(),
        0
    );
    // With the automatic recording on, the official ones no longer wait in "To confirm".
    let pend = engine::pending_income(
        db.conn(),
        &PendingIncomeArgs {
            since: Some("2025-01-01".into()),
            account_id: None,
        },
    )
    .unwrap();
    assert!(pend.iter().all(|p| p.source != "fundos_net"), "{pend:#?}");
}

#[test]
fn a_rebuild_of_chosen_keys_leaves_the_rest() {
    let (_d, mut db, _) = fund_scenario();
    let l = ledger(&db, "2025-05-10");
    let key = entry(
        &l,
        "2025-03-31",
        IncomeKind::FiiIncome,
        EntryState::ValueDiff,
    )
    .key
    .clone();
    let rep = income_ledger::rebuild_as(
        db.conn_mut(),
        &Author::user(),
        &RebuildArgs {
            keys: Some(vec![key]),
            asset_id: None,
        },
        day("2025-05-10"),
    )
    .unwrap();
    assert_eq!((rep.created, rep.updated, rep.deleted), (0, 1, 0));
    let after = ledger(&db, "2025-05-10");
    assert_eq!(after.counts.fixes, 3);
    let mar = entry(
        &after,
        "2025-03-31",
        IncomeKind::FiiIncome,
        EntryState::Matches,
    );
    assert_eq!(mar.recorded.as_ref().unwrap().tax_withheld, 204);
}

#[test]
fn a_jcp_recorded_at_the_old_rate_is_fixed_keeping_the_net() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ACME3", AssetClass::StockBr);
    trade(&mut db, &a, "2025-06-02", TradeSide::Buy, "1224", "10");
    db.conn()
        .execute(
            "INSERT INTO research_dividend (ticker, kind, ex_date, value, per_shares, pay_date, seq, source, fetched_at)
             VALUES ('ACME3', 'jcp', '2026-03-19', '0.116', 1, '2026-08-28', 0, 'b3_listados', 'x')",
            [],
        )
        .unwrap();
    // Net as credited, gross back-calculated at 15%.
    income(
        &mut db,
        &a,
        IncomeKind::Jcp,
        Some("2026-03-19"),
        "2026-08-28",
        13781,
        2067,
    );
    let l = ledger(&db, "2026-09-01");
    let e = entry(&l, "2026-03-19", IncomeKind::Jcp, EntryState::TaxDiff);
    // 1,224 × 0.116 = 141.98; 17.5% withheld without the fraction of a cent: 24.84.
    assert_eq!(e.expected_gross, Some(14198));
    assert_eq!(e.expected_tax, Some(2484));
    assert_eq!(e.expected_net, Some(11714));
    assert_eq!(e.fix, Some(Fix::Update));
    income_ledger::rebuild_as(
        db.conn_mut(),
        &Author::user(),
        &RebuildArgs::default(),
        day("2026-09-01"),
    )
    .unwrap();
    let (g, t, n): (i64, i64, i64) = db
        .conn()
        .query_row(
            "SELECT gross, tax_withheld, net FROM income_event WHERE asset_id = ?1",
            [&a],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((g, t, n), (14198, 2484, 11714));
}

#[test]
fn the_credited_amount_of_the_b3_statement_prevails() {
    let (_d, mut db) = db();
    let f = asset(&mut db, "FUND11", AssetClass::Fii);
    trade(&mut db, &f, "2025-01-10", TradeSide::Buy, "83", "9");
    notice(
        &db,
        1,
        "fii_income",
        "2025-01-31",
        "0.617",
        "2025-02-14",
        Some(1),
    );
    let id = income(
        &mut db,
        &f,
        IncomeKind::FiiIncome,
        None,
        "2025-02-14",
        5100,
        0,
    );
    db.conn()
        .execute("UPDATE income_event SET source = 'b3' WHERE id = ?1", [&id])
        .unwrap();
    let l = ledger(&db, "2025-03-01");
    let e = entry(
        &l,
        "2025-01-31",
        IncomeKind::FiiIncome,
        EntryState::ValueDiff,
    );
    assert_eq!(e.expected_gross, Some(5121));
    assert_eq!(e.fix, None);
    assert_eq!(l.received_base, 5100);
}

#[test]
fn a_distribution_without_an_official_notice_is_recorded_only_when_chosen() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ACME3", AssetClass::StockBr);
    trade(&mut db, &a, "2025-01-10", TradeSide::Buy, "100", "10");
    db.conn()
        .execute(
            "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
             VALUES ('f1', ?1, 'dividend', '2025-03-10', '2025-03-20', '0.25', 'BRL', 0, 'fundamentus', 'x')",
            [&a],
        )
        .unwrap();
    let l = ledger(&db, "2025-04-01");
    let e = entry(&l, "2025-03-10", IncomeKind::Dividend, EntryState::Missing);
    assert!(!e.official);
    assert_eq!(l.counts.fixes, 0);
    let key = e.key.clone();
    // The rebuild of everything leaves it to the family.
    let rep = income_ledger::rebuild_as(
        db.conn_mut(),
        &Author::user(),
        &RebuildArgs::default(),
        day("2025-04-01"),
    )
    .unwrap();
    assert_eq!(rep.created, 0);
    // Checked by the family: recorded, with the agenda as the origin.
    let rep = income_ledger::rebuild_as(
        db.conn_mut(),
        &Author::user(),
        &RebuildArgs {
            keys: Some(vec![key]),
            asset_id: None,
        },
        day("2025-04-01"),
    )
    .unwrap();
    assert_eq!(rep.created, 1);
    let src: String = db
        .conn()
        .query_row(
            "SELECT source FROM income_event WHERE asset_id = ?1",
            [&a],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(src, "agenda");
}
