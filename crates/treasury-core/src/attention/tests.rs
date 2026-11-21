use serde_json::{Value, json};

use super::*;
use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;
use crate::reports::fixture::{self, expense, new_db, write};
use crate::transactions::{TxnKind, TxnStatus};

fn day(s: &str) -> NaiveDate {
    parse_date(s).unwrap()
}

fn planned(tx: &rusqlite::Transaction<'_>, account: &str, date: &str, amount: i64, what: &str) {
    let mut p = fixture::new_txn(tx, account, date, amount, TxnKind::Expense, None, None);
    p.status = TxnStatus::Planned;
    p.description = what.into();
    crate::transactions::insert(tx, &p).unwrap();
}

fn kinds(a: &Attention) -> Vec<&str> {
    a.items.iter().map(|i| i.kind.as_str()).collect()
}

fn item<'a>(a: &'a Attention, kind: &str) -> &'a AttentionItem {
    a.items
        .iter()
        .find(|i| i.kind == kind)
        .unwrap_or_else(|| panic!("{kind} in {:?}", kinds(a)))
}

#[test]
fn one_list_with_what_is_true_now_and_no_fact_twice() {
    let today = day("2026-10-15");
    let mut t = new_db();
    let (checking, light) = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 900_000, "2026-01-01");
        let savings = fixture::account(tx, "Poupança", "savings", "BRL", 100_000, "2026-01-01");
        // Account up to date, with one uncategorized expense.
        expense(tx, &checking, "2026-10-10", -5_000, None, None, "Padaria");
        // Savings idle since August.
        expense(tx, &savings, "2026-08-01", -1_000, None, None, "Tarifa");
        planned(tx, &checking, "2026-10-16", -30_000, "Conta de luz");
        planned(tx, &checking, "2026-10-05", -12_000, "Internet");
        let light: String = tx.query_row(
            "SELECT id FROM txn WHERE description = 'Conta de luz'",
            [],
            |r| r.get(0),
        )?;
        Ok((checking, light))
    });
    let conn = t.db.conn();
    // Reminders store the due-date alerts and the alerts for accounts without a statement.
    crate::notify::run(conn, today).unwrap();
    let stored = alerts::list(conn, &Default::default()).unwrap();
    assert!(stored.iter().any(|a| a.kind == "due_soon"), "{stored:?}");
    assert!(stored.iter().any(|a| a.kind == "stale_accounts"));

    let a = list(conn, today).unwrap();
    // Each fact only once: stored alerts of those kinds stay out.
    for k in ["due_soon", "overdue", "stale_accounts", "uncategorized"] {
        assert_eq!(
            a.items.iter().filter(|i| i.kind == k).count(),
            1,
            "{k} em {:?}",
            kinds(&a)
        );
    }
    assert!(a.items.iter().all(
        |i| i.source != AttentionSource::Alert || !LIVE_ALERT_KINDS.contains(&i.kind.as_str())
    ));
    let due = item(&a, "due_soon");
    assert_eq!(due.title, "Conta de luz vence amanhã");
    assert_eq!(due.severity, AttentionSeverity::Warning);
    assert_eq!(
        due.target.as_ref().unwrap().screen,
        AlertScreen::Transactions
    );
    assert_eq!(
        due.target.as_ref().unwrap().account_id.as_deref(),
        Some(checking.as_str())
    );
    let late = item(&a, "overdue");
    assert_eq!(late.title, "Internet venceu em 05/10");
    assert!(late.body.contains("R$ 120,00"), "{}", late.body);
    let stale = item(&a, "stale_accounts");
    assert_eq!(stale.source, AttentionSource::Data);
    assert!(stale.body.contains("Poupança"), "{}", stale.body);
    assert_eq!(stale.target.as_ref().unwrap().screen, AlertScreen::Import);
    // From most severe to least severe.
    let sev: Vec<_> = a.items.iter().map(|i| i.severity).collect();
    assert!(sev.windows(2).all(|w| w[0] <= w[1]), "{sev:?}");
    assert_eq!(a.counts.total, a.items.len() as i64);
    assert!(a.counts.urgent >= 3 && a.counts.urgent <= a.counts.total);

    // The bill was paid: it leaves the list without anyone resolving the alert.
    conn.execute("UPDATE txn SET status = 'cleared' WHERE id = ?1", [&light])
        .unwrap();
    let a = list(conn, today).unwrap();
    assert!(!kinds(&a).contains(&"due_soon"), "{:?}", kinds(&a));
    assert!(
        alerts::list(conn, &Default::default())
            .unwrap()
            .iter()
            .any(|x| x.kind == "due_soon")
    );
}

#[test]
fn coverage_says_until_when_the_numbers_are_complete() {
    let today = day("2026-10-15");
    let mut t = new_db();
    assert_eq!(
        coverage(t.db.conn(), today).unwrap().status,
        CoverageStatus::Empty
    );
    let savings = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 100_000, "2026-01-01");
        let savings = fixture::account(tx, "Poupança", "savings", "BRL", 100_000, "2026-01-01");
        // A brokerage is not an everyday account: it does not count toward coverage.
        fixture::account(tx, "Corretora", "brokerage", "BRL", 0, "2026-01-01");
        expense(tx, &checking, "2026-10-10", -5_000, None, None, "Padaria");
        expense(tx, &savings, "2026-08-01", -1_000, None, None, "Tarifa");
        Ok(savings)
    });
    let c = coverage(t.db.conn(), today).unwrap();
    assert_eq!(c.status, CoverageStatus::Partial);
    assert_eq!(c.accounts, 2);
    assert_eq!(c.stale.len(), 1);
    assert_eq!(c.stale[0].name, "Poupança");
    assert_eq!(c.stale[0].last_txn_date.as_deref(), Some("2026-08-01"));
    assert_eq!(c.complete_until.as_deref(), Some("2026-08-01"));

    write(&mut t.db, |tx| {
        expense(tx, &savings, "2026-10-12", -1_000, None, None, "Tarifa");
        Ok(())
    });
    let c = coverage(t.db.conn(), today).unwrap();
    assert_eq!(c.status, CoverageStatus::Complete);
    assert!(c.stale.is_empty());
    // The oldest news among the accounts: until then all of them have data.
    assert_eq!(c.complete_until.as_deref(), Some("2026-10-10"));
}

#[test]
fn dismissing_snoozes_in_the_vault_and_resolves_alerts() {
    let today = day("2026-10-15");
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 900_000, "2026-01-01");
        expense(tx, &checking, "2026-10-10", -5_000, None, None, "Padaria");
        planned(tx, &checking, "2026-10-16", -30_000, "Conta de luz");
        Ok(())
    });
    let conn = t.db.conn();
    let id = alerts::create(
        conn,
        &alerts::NewAlert {
            kind: "card_limit",
            severity: AlertSeverity::Critical,
            title: Msg::new("alertas-vence-conta")
                .arg("descricao", "Cartão")
                .arg("quando", Msg::new("alertas-quando").arg("dias", 0)),
            body: vec![],
            source: "lembretes",
            dedupe_key: Some("teste:1".into()),
            ref_table: None,
            ref_id: None,
            data: json!({}),
        },
    )
    .unwrap()
    .unwrap();

    let a = list(conn, today).unwrap();
    // The critical alert comes first.
    assert_eq!(a.items[0].alert_id.as_deref(), Some(id.as_str()));
    assert_eq!(a.items[0].severity, AttentionSeverity::Critical);
    assert!(kinds(&a).contains(&"uncategorized"));

    // Dismissed task: gone now and back in a week, if it still applies.
    dismiss(conn, "todo:uncategorized", today).unwrap();
    assert!(!kinds(&list(conn, today).unwrap()).contains(&"uncategorized"));
    assert!(
        !kinds(&list(conn, today + Duration::days(7)).unwrap()).contains(&"uncategorized"),
        "still snoozed on the seventh day"
    );
    assert!(kinds(&list(conn, today + Duration::days(8)).unwrap()).contains(&"uncategorized"));

    // Dismissed due date: does not come back (the key carries the date).
    let due = item(&a, "due_soon").id.clone();
    dismiss(conn, &due, today).unwrap();
    assert!(!kinds(&list(conn, today).unwrap()).contains(&"due_soon"));

    // Dismissed alert: resolved in the alert inbox.
    dismiss(conn, &format!("alert:{id}"), today).unwrap();
    assert_eq!(
        alerts::get(conn, &id).unwrap().status,
        AlertStatus::Resolved
    );
    assert!(
        list(conn, today)
            .unwrap()
            .items
            .iter()
            .all(|i| i.alert_id.as_deref() != Some(id.as_str()))
    );

    // The cleanup removes from the setting whatever has already expired.
    dismiss(conn, "data:inbox", today + Duration::days(200)).unwrap();
    let kept: BTreeMap<String, String> = crate::settings::get(conn, DISMISSED_SETTING)
        .unwrap()
        .unwrap();
    assert_eq!(kept.keys().collect::<Vec<_>>(), ["data:inbox"]);

    assert!(dismiss(conn, "qualquer-coisa", today).is_err());
}

#[test]
fn old_alerts_stay_only_in_the_alerts_box() {
    let t = new_db();
    let conn = t.db.conn();
    let id = alerts::create(
        conn,
        &alerts::NewAlert {
            kind: "card_limit",
            severity: AlertSeverity::Warning,
            title: Msg::new("alertas-vence-conta")
                .arg("descricao", "Cartão")
                .arg("quando", Msg::new("alertas-quando").arg("dias", 0)),
            body: vec![],
            source: "lembretes",
            dedupe_key: Some("teste:velho".into()),
            ref_table: None,
            ref_id: None,
            data: json!({}),
        },
    )
    .unwrap()
    .unwrap();
    let has = |d: NaiveDate| {
        list(conn, d)
            .unwrap()
            .items
            .iter()
            .any(|i| i.alert_id.as_deref() == Some(id.as_str()))
    };
    // The alert was created now (real clock): it shows up today and leaves 31 days later.
    let now = crate::time::today_local();
    assert!(has(now));
    assert!(!has(now + Duration::days(ALERT_DAYS + 2)));
}

fn api_month(api: &Api) -> Value {
    api.call("attention.month_close", Value::Null).unwrap()
}

#[test]
fn api_lists_for_everyone_and_only_the_interface_dismisses() {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let v = api.call("attention.list", Value::Null).unwrap();
    assert_eq!(v["coverage"]["status"], "empty");
    // Vault without accounts: the first step is the setup.
    let items = v["items"].as_array().unwrap();
    assert!(items.iter().any(|i| i["kind"] == "onboarding"), "{v}");
    assert_eq!(v["counts"]["total"], items.len());

    let mcp = Actor::Mcp {
        client: "teste".into(),
        read_only: true,
    };
    assert!(api.call_as(&mcp, "attention.list", Value::Null).is_ok());
    assert!(api.call_as(&mcp, "attention.coverage", Value::Null).is_ok());
    assert!(
        api.call_as(&mcp, "attention.dismiss", json!({"id": "todo:onboarding"}))
            .is_err()
    );
    // Next step and month close: read access for everyone; marking and closing, only the interface.
    let step = api.call("attention.next_step", Value::Null).unwrap();
    assert_eq!(step["rung"], 1);
    assert_eq!(step["rungs"].as_array().unwrap().len(), 6);
    let close = api_month(&api);
    assert_eq!(close["applicable"], false);
    assert!(
        api.call_as(&mcp, "attention.next_step", Value::Null)
            .is_ok()
    );
    assert!(
        api.call_as(&mcp, "attention.month_close", Value::Null)
            .is_ok()
    );
    let month = close["month"].as_str().unwrap().to_string();
    let set = |step: &str| json!({"month": month, "step": step, "done": true});
    assert!(
        api.call_as(&mcp, "attention.month_close_set", set("balances"))
            .is_err()
    );
    api.call("attention.month_close_set", set("balances"))
        .unwrap();
    assert_eq!(api_month(&api)["steps"][2]["done"], true);
    // Close only with the five steps done.
    assert!(
        api.call("attention.month_close_set", set("closed"))
            .is_err()
    );
    api.call("attention.month_close_set", set("summary"))
        .unwrap();
    // No accounts and no pending items besides the setup, which is dismissed below.
    api.call("attention.dismiss", json!({"id": "todo:onboarding"}))
        .unwrap();
    api.call("attention.month_close_set", set("closed"))
        .unwrap();
    assert!(api_month(&api)["closed_at"].is_string());
    let v = api.call("attention.list", Value::Null).unwrap();
    assert!(
        v["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["kind"] != "onboarding")
    );
}

#[test]
fn outdated_embedded_tables_become_a_quiet_notice() {
    let t = new_db();
    let conn = t.db.conn();
    let year = crate::goals::inss::TABLE.year;
    let on = |d: NaiveDate| {
        list(conn, d)
            .unwrap()
            .items
            .into_iter()
            .find(|i| i.kind == "outdated_tables")
    };
    // Within the table's year and in January of the following year, nothing.
    assert!(on(NaiveDate::from_ymd_opt(year, 12, 20).unwrap()).is_none());
    assert!(on(NaiveDate::from_ymd_opt(year + 1, 1, 20).unwrap()).is_none());
    let late = NaiveDate::from_ymd_opt(year + 1, 3, 1).unwrap();
    let item = on(late).expect("warning for an outdated table");
    assert_eq!(item.severity, AttentionSeverity::Info);
    assert_eq!(item.title, "1 tabela do app pode estar desatualizada");
    assert!(
        item.body.contains(&format!("INSS (piso e teto) de {year}")),
        "{}",
        item.body
    );
    assert_eq!(item.route.as_deref(), Some("/ajustes?secao=sobre"));
    // Dismissed, it does not come back the following week.
    dismiss(conn, &item.id, late).unwrap();
    assert!(on(late + Duration::days(30)).is_none());
}
