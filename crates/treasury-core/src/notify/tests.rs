use super::*;
use crate::reports::fixture::{self, expense, new_db, write};
use crate::transactions::{TxnKind, TxnStatus};

#[test]
fn reminders_create_alerts_once_and_notices_follow_prefs() {
    let today = parse_date("2026-10-15").unwrap();
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 30_000, "2026-01-01");
        // Electricity bill expected tomorrow, bigger than the balance: due date and negative
        // balance.
        let mut p = fixture::new_txn(
            tx,
            &checking,
            "2026-10-16",
            -30_000,
            TxnKind::Expense,
            None,
            None,
        );
        p.status = TxnStatus::Planned;
        p.description = "Conta de luz".into();
        crate::transactions::insert(tx, &p)?;
        // Subscription whose price went up.
        for (date, amount) in [
            ("2026-07-10", 3_990),
            ("2026-08-10", 3_990),
            ("2026-09-10", 3_990),
            ("2026-10-10", 4_490),
        ] {
            expense(tx, &checking, date, -amount, None, None, "Netflix");
        }
        Ok(())
    });
    let conn = t.db.conn();
    let r = run(conn, today).unwrap();
    let all = alerts::list(conn, &Default::default()).unwrap();
    let kinds: Vec<&str> = all.iter().map(|a| a.kind.as_str()).collect();
    for k in ["due_soon", "negative_balance", "subscription_increase"] {
        assert!(kinds.contains(&k), "{k} in {kinds:?}");
    }
    assert_eq!(r.created.len(), all.len());
    let due = all.iter().find(|a| a.kind == "due_soon").unwrap();
    assert_eq!(due.title, "Conta de luz vence amanhã");
    let neg = all.iter().find(|a| a.kind == "negative_balance").unwrap();
    assert_eq!(neg.title, "Conta: saldo previsto fica negativo");
    assert!(neg.body.contains("a partir de 16/10"), "{}", neg.body);
    // The same check does not repeat alerts.
    assert!(run(conn, today).unwrap().created.is_empty());
    assert!(run_due(conn, today).unwrap().is_some());
    assert!(
        run_due(conn, today).unwrap().is_none(),
        "at most once an hour"
    );

    // Notifications: no amounts by default; turned-off types are left out.
    let n = notices(conn, &all).unwrap();
    assert_eq!(n.len(), all.len());
    assert!(
        n.iter()
            .all(|x| x.body == "Abra o Treasury para ver os detalhes.")
    );
    save_prefs(
        conn,
        &NotifyPrefs {
            show_amounts: true,
            subscriptions: false,
            bills_days: 99,
            ..NotifyPrefs::default()
        },
    )
    .unwrap();
    assert_eq!(prefs(conn).unwrap().bills_days, 15);
    let n = notices(conn, &all).unwrap();
    assert_eq!(n.len(), all.len() - 1);
    assert!(n.iter().any(|x| x.body.contains("R$")));
    save_prefs(
        conn,
        &NotifyPrefs {
            system: false,
            ..NotifyPrefs::default()
        },
    )
    .unwrap();
    assert!(notices(conn, &all).unwrap().is_empty());
}

/// Negative balance for several days: a single alert, updated in place; warns again only if
/// it gets much worse; closes when the forecast goes back to positive and opens another if it
/// goes negative again. The old alerts (one per day) become one.
#[test]
fn negative_balance_is_one_alert_per_episode() {
    let day = |d: &str| parse_date(d).unwrap();
    let mut t = new_db();
    let acc = write(&mut t.db, |tx| {
        let acc = fixture::account(tx, "Conta", "checking", "BRL", 10_000, "2026-01-01");
        let mut p = fixture::new_txn(
            tx,
            &acc,
            "2026-10-20",
            -30_000,
            TxnKind::Expense,
            None,
            None,
        );
        p.status = TxnStatus::Planned;
        p.description = "Aluguel".into();
        crate::transactions::insert(tx, &p)?;
        // Two alerts the old way (one per day) for the same account.
        for (key, at) in [
            ("negative:x:2026-10-14", "2026-10-14T09:00:00.000Z"),
            ("negative:x:2026-10-15", "2026-10-15T09:00:00.000Z"),
        ] {
            tx.execute(
                "INSERT INTO alert (id, kind, severity, title, body, status, source, dedupe_key, ref_table, ref_id, data, created_at, updated_at)
                 VALUES (?1, 'negative_balance', 'warning', 'Conta: saldo fica negativo em …', '', 'new', 'lembretes', ?1, 'account', ?2, '{\"min\": -20000}', ?3, ?3)",
                rusqlite::params![key, acc, at],
            )?;
        }
        Ok(acc)
    });
    let conn = t.db.conn();
    let neg = |c: &rusqlite::Connection| -> Vec<alerts::Alert> {
        alerts::list(
            c,
            &alerts::AlertListArgs {
                status: None,
                kind: Some("negative_balance".into()),
                limit: None,
            },
        )
        .unwrap()
        .into_iter()
        .chain(
            alerts::list(
                c,
                &alerts::AlertListArgs {
                    status: Some(alerts::AlertStatus::Resolved),
                    kind: Some("negative_balance".into()),
                    limit: None,
                },
            )
            .unwrap(),
        )
        .collect()
    };
    let r = run(conn, day("2026-10-16")).unwrap();
    assert!(
        alerts::list(conn, &Default::default())
            .unwrap()
            .iter()
            .filter(|a| a.kind == "negative_balance")
            .all(|a| !r.created.contains(&a.id)),
        "the old ones become the episode"
    );
    let a = neg(conn);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].title, "Conta: saldo previsto fica negativo");
    // Following days, still negative: nothing new, the same alert.
    // Following days, still negative: nothing new, the same alert (the rent due date is
    // another reminder).
    for d in ["2026-10-17", "2026-10-18", "2026-10-19"] {
        run(conn, day(d)).unwrap();
        assert_eq!(neg(conn).len(), 1, "{d}");
    }
    assert_eq!(neg(conn).len(), 1);
    // Seen by the user: stays seen as long as it does not get much worse.
    alerts::set_status(
        conn,
        &alerts::AlertStatusArgs {
            ids: vec![a[0].id.clone()],
            status: alerts::AlertStatus::Seen,
        },
    )
    .unwrap();
    run(conn, day("2026-10-19")).unwrap();
    assert_eq!(neg(conn)[0].status, alerts::AlertStatus::Seen);
    // Gets much worse (−R$ 200 to −R$ 900): it is new again and warns again.
    let before = now_utc();
    std::thread::sleep(std::time::Duration::from_millis(5));
    {
        let tx = conn.unchecked_transaction().unwrap();
        let mut p = fixture::new_txn(
            &tx,
            &acc,
            "2026-10-25",
            -70_000,
            TxnKind::Expense,
            None,
            None,
        );
        p.status = TxnStatus::Planned;
        crate::transactions::insert(&tx, &p).unwrap();
        tx.commit().unwrap();
    }
    run(conn, day("2026-10-19")).unwrap();
    let a = neg(conn);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].status, alerts::AlertStatus::New);
    assert!(a[0].body.contains("R$ 900,00"), "{}", a[0].body);
    let again = alerts::created_since(conn, &before).unwrap();
    assert!(
        again.iter().any(|x| x.id == a[0].id),
        "reopened enters the notifications"
    );
    // Back to positive: the episode closes (resolved, with the reason why).
    {
        let tx = conn.unchecked_transaction().unwrap();
        tx.execute("DELETE FROM txn WHERE status = 'planned'", [])
            .unwrap();
        tx.commit().unwrap();
    }
    run(conn, day("2026-10-20")).unwrap();
    let a = neg(conn);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].status, alerts::AlertStatus::Resolved);
    assert!(a[0].body.contains("não fica mais negativo"));
    // Goes negative again: new episode.
    {
        let tx = conn.unchecked_transaction().unwrap();
        let mut p = fixture::new_txn(
            &tx,
            &acc,
            "2026-11-01",
            -50_000,
            TxnKind::Expense,
            None,
            None,
        );
        p.status = TxnStatus::Planned;
        crate::transactions::insert(&tx, &p).unwrap();
        tx.commit().unwrap();
    }
    run(conn, day("2026-10-21")).unwrap();
    let a = neg(conn);
    assert_eq!(a.len(), 2);
    assert_eq!(
        a.iter()
            .filter(|x| x.status == alerts::AlertStatus::New)
            .count(),
        1
    );
}

/// Reminders store the key (V128): the text comes out in the reader's language, and what is
/// saved in `title`/`body` stays in pt-BR.
#[test]
fn reminders_follow_the_reader_locale() {
    use crate::i18n::{Locale, set_current};
    let today = parse_date("2026-10-15").unwrap();
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 30_000, "2026-01-01");
        let mut p = fixture::new_txn(
            tx,
            &checking,
            "2026-10-16",
            -40_000,
            TxnKind::Expense,
            None,
            None,
        );
        p.status = TxnStatus::Planned;
        p.description = "Conta de luz".into();
        crate::transactions::insert(tx, &p)?;
        Ok(())
    });
    let conn = t.db.conn();
    set_current(Locale::En);
    run(conn, today).unwrap();
    let all = alerts::list(conn, &Default::default()).unwrap();
    let due = all.iter().find(|a| a.kind == "due_soon").unwrap();
    assert_eq!(due.title, "Conta de luz is due tomorrow");
    assert_eq!(due.body, "R$400.00 from Conta, due Oct 16.");
    assert_eq!(due.source, "reminders");
    let neg = all.iter().find(|a| a.kind == "negative_balance").unwrap();
    assert_eq!(neg.title, "Conta: forecast balance goes negative");
    assert!(
        neg.body
            .starts_with("According to the balance forecast, Conta goes negative from Oct 16"),
        "{}",
        neg.body
    );
    assert!(
        notices(conn, &all)
            .unwrap()
            .iter()
            .all(|x| x.body == "Open Treasury to see the details.")
    );
    let stored: String = conn
        .query_row("SELECT title FROM alert WHERE id = ?1", [&due.id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(stored, "Conta de luz vence amanhã");
    set_current(Locale::Es);
    assert_eq!(
        alerts::get(conn, &due.id).unwrap().title,
        "Conta de luz vence mañana"
    );
    set_current(Locale::PtBr);
    let due = alerts::get(conn, &due.id).unwrap();
    assert_eq!(due.title, "Conta de luz vence amanhã");
    assert_eq!(due.body, "R$ 400,00 em Conta, vencimento 16/10.");
    assert_eq!(due.source, "lembretes");
}

/// The stored text: typed arguments (money, date, percentage, another message, list)
/// come back from the JSON and come out formatted in the reader's language; ready text passes
/// as is.
#[test]
fn stored_alert_text_round_trips() {
    use crate::ai::alerts::{Arg, Msg, NewAlert, Part};
    use crate::i18n::{Locale, set_current};
    let t = new_db();
    let conn = t.db.conn();
    let id = alerts::create(
        conn,
        &NewAlert {
            kind: "monthly_close",
            severity: AlertSeverity::Info,
            title: Msg::new("alertas-fechamento-titulo").arg("mes", Arg::month("2026-09")),
            body: vec![
                Msg::new("alertas-fechamento-maiores")
                    .arg(
                        "lista",
                        Arg::join(
                            vec![
                                Msg::new("alertas-item-valor")
                                    .arg("nome", "Mercado")
                                    .arg("valor", Arg::money(123_456, "BRL"))
                                    .into(),
                                Arg::pct(0.125, 1),
                            ],
                            "; ",
                        ),
                    )
                    .into(),
                Part::from("\n"),
                Part::from("texto do motor"),
            ],
            source: "rotina: fechamento mensal (Motor A)",
            dedupe_key: None,
            ref_table: None,
            ref_id: None,
            data: serde_json::json!({}),
        },
    )
    .unwrap()
    .unwrap();
    let a = alerts::get(conn, &id).unwrap();
    assert_eq!(a.title, "Fechamento de setembro de 2026");
    assert_eq!(
        a.body,
        "- Maiores gastos: Mercado R$ 1.234,56; 12,5%\ntexto do motor"
    );
    set_current(Locale::En);
    let a = alerts::get(conn, &id).unwrap();
    assert_eq!(a.title, "September 2026 close");
    assert_eq!(
        a.body,
        "- Largest expenses: Mercado R$1,234.56; 12.5%\ntexto do motor"
    );
    assert_eq!(a.source, "routine: monthly close (Motor A)");
    set_current(Locale::PtBr);
}

/// MCP tool names leave the text for the family and go to the alert's sources,
/// also in alerts saved before (ready text, no key).
#[test]
fn alert_text_hides_tool_names_and_keeps_them_as_sources() {
    use crate::ai::alerts::{Msg, NewAlert, Part, strip_tool_names};
    let (t, f) = strip_tool_names(
        "**Resumo de setembro de 2026** (fonte: get_monthly_flow, spending_summary, get_budget_status, get_net_worth)\n\
         1. Educação R$ 2.400,00 (spending_summary): vale conferir.\n\
         2. Já foram R$ 859,91 de R$ 900,00 (95,5%) com 64,5% do mês decorrido (get_budget_status).\n\
         3. Gastos subiram (12% a mais, get_monthly_flow). Despesas de R$ 8.420,10 — get_monthly_flow.\n\
         4. According to get_budget_status, compras estão no limite. O mercado (ou feira) segue igual.",
    );
    assert_eq!(
        t,
        "**Resumo de setembro de 2026**\n\
         1. Educação R$ 2.400,00: vale conferir.\n\
         2. Já foram R$ 859,91 de R$ 900,00 (95,5%) com 64,5% do mês decorrido.\n\
         3. Gastos subiram (12% a mais). Despesas de R$ 8.420,10.\n\
         4. compras estão no limite. O mercado (ou feira) segue igual."
    );
    assert_eq!(
        f,
        vec![
            "get_monthly_flow",
            "spending_summary",
            "get_budget_status",
            "get_net_worth"
        ]
    );
    let db = new_db();
    let conn = db.db.conn();
    let id = alerts::create(
        conn,
        &NewAlert {
            kind: "budget_pace",
            severity: AlertSeverity::Info,
            title: Msg::new("alertas-fechamento-titulo").arg("mes", "2026-09"),
            body: vec![Part::from(
                "Já foram R$ 859,91 de R$ 900,00 com 64,5% do mês decorrido (get_budget_status).",
            )],
            source: "rotina: alertas",
            dedupe_key: None,
            ref_table: None,
            ref_id: None,
            data: serde_json::json!({"month": "2026-09"}),
        },
    )
    .unwrap()
    .unwrap();
    let a = alerts::get(conn, &id).unwrap();
    assert_eq!(
        a.body,
        "Já foram R$ 859,91 de R$ 900,00 com 64,5% do mês decorrido."
    );
    assert_eq!(a.data["sources"], serde_json::json!(["get_budget_status"]));
    assert_eq!(a.data["month"], "2026-09");
}

/// Each alert says where to open its origin (asset, account, monthly report…) and the type's
/// name in the reader's language; a generic alert has no destination.
#[test]
fn alert_carries_its_target_and_kind_label() {
    use crate::ai::alerts::{AlertScreen, Msg, NewAlert, Part};
    use crate::i18n::{Locale, set_current};
    let db = new_db();
    let conn = db.db.conn();
    let mk =
        |kind: &str, table: Option<&'static str>, id: Option<&str>, data: serde_json::Value| {
            let id = alerts::create(
                conn,
                &NewAlert {
                    kind,
                    severity: AlertSeverity::Info,
                    title: Msg::new("alertas-fechamento-titulo").arg("mes", "2026-09"),
                    body: vec![Part::from("x")],
                    source: "lembretes",
                    dedupe_key: None,
                    ref_table: table,
                    ref_id: id.map(str::to_string),
                    data,
                },
            )
            .unwrap()
            .unwrap();
            alerts::get(conn, &id).unwrap()
        };
    set_current(Locale::PtBr);
    let a = mk(
        "negative_balance",
        Some("account"),
        Some("conta-1"),
        serde_json::json!({}),
    );
    let t = a.target.unwrap();
    assert_eq!(
        (t.screen, t.id.as_deref()),
        (AlertScreen::Account, Some("conta-1"))
    );
    assert_eq!(a.kind_label, "Saldo negativo previsto");
    let a = mk(
        "asset_score",
        Some("asset"),
        Some("ativo-1"),
        serde_json::json!({}),
    );
    assert_eq!(a.target.unwrap().screen, AlertScreen::Asset);
    let a = mk(
        "monthly_close",
        None,
        None,
        serde_json::json!({"numbers": {"month": "2026-09"}}),
    );
    let t = a.target.unwrap();
    assert_eq!(
        (t.screen, t.month.as_deref()),
        (AlertScreen::Reports, Some("2026-09"))
    );
    let a = mk(
        "budget_pace",
        Some("category"),
        Some("cat-1"),
        serde_json::json!({"month": "2026-10"}),
    );
    let t = a.target.unwrap();
    assert_eq!(
        (t.screen, t.month.as_deref()),
        (AlertScreen::Budget, Some("2026-10"))
    );
    // A transaction that no longer exists and a generic type: no destination.
    assert!(
        mk(
            "due_soon",
            Some("txn"),
            Some("sumiu"),
            serde_json::json!({})
        )
        .target
        .is_none()
    );
    let a = mk("outro_tipo", None, None, serde_json::json!({}));
    assert!(a.target.is_none());
    assert_eq!(a.kind_label, "outro_tipo");
    set_current(Locale::En);
    assert_eq!(alerts::get(conn, &a.id).unwrap().kind_label, "outro_tipo");
    let l = alerts::list(conn, &Default::default()).unwrap();
    assert!(
        l.iter()
            .any(|x| x.kind_label == "Forecast negative balance")
    );
    set_current(Locale::PtBr);
}

#[test]
fn quiet_recurrence_stops_due_reminders_and_closes_the_open_ones() {
    // A cleaner paid every week: the family does not want a reminder for it, but the forecast
    // still counts it.
    let today = parse_date("2026-11-22").unwrap();
    let mut t = new_db();
    let rec = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 500_000, "2026-01-01");
        let input: crate::recurrence::RecurrenceInput = serde_json::from_value(json!({
            "description": "Diarista", "account_id": checking, "amount": -17_000,
            "kind": "expense", "spec": {"freq": "WEEKLY"}, "start_date": "2026-11-24",
        }))
        .unwrap();
        crate::recurrence::create(tx, &input, today)
    });
    let conn = t.db.conn();
    run(conn, today).unwrap();
    let open = |conn: &Connection| -> Vec<Alert> {
        alerts::list(conn, &Default::default())
            .unwrap()
            .into_iter()
            .filter(|a| a.kind == "due_soon" && a.status != alerts::AlertStatus::Resolved)
            .collect()
    };
    assert_eq!(open(conn).len(), 1, "the reminder exists before");
    let up = crate::reports::upcoming(conn, today, 7).unwrap();
    assert_eq!(up[0].recurrence_id.as_deref(), Some(rec.as_str()));

    // Turning the reminder off closes the open alert and no new one appears.
    assert_eq!(quiet::set(conn, &rec, false).unwrap(), 1);
    assert!(open(conn).is_empty());
    run(conn, today).unwrap();
    assert!(open(conn).is_empty());
    let attention = crate::attention::list(conn, today).unwrap();
    assert!(
        attention
            .items
            .iter()
            .all(|i| i.source != crate::attention::AttentionSource::Due),
        "{:?}",
        attention.items
    );
    let view = crate::recurrence::get(conn, &rec, today).unwrap();
    assert!(!view.remind);
    // The forecast still has it.
    assert!(!crate::reports::upcoming(conn, today, 7).unwrap().is_empty());

    // Back on: the reminder returns with the next occurrence (the one already closed stays closed).
    quiet::set(conn, &rec, true).unwrap();
    assert!(crate::recurrence::get(conn, &rec, today).unwrap().remind);
    run(conn, parse_date("2026-11-29").unwrap()).unwrap();
    assert_eq!(open(conn).len(), 1);
    assert!(quiet::set(conn, "nope", false).is_err());
}

#[test]
fn due_alert_closes_once_its_date_passes() {
    let today = parse_date("2026-11-15").unwrap();
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 500_000, "2026-01-01");
        let mut p = fixture::new_txn(
            tx,
            &checking,
            "2026-11-17",
            -17_000,
            TxnKind::Expense,
            None,
            None,
        );
        p.status = TxnStatus::Planned;
        p.description = "Diarista".into();
        crate::transactions::insert(tx, &p)?;
        Ok(())
    });
    let conn = t.db.conn();
    run(conn, today).unwrap();
    let still_open = |conn: &Connection| {
        alerts::list(conn, &Default::default())
            .unwrap()
            .into_iter()
            .filter(|a| a.kind == "due_soon" && a.status != alerts::AlertStatus::Resolved)
            .count()
    };
    assert_eq!(still_open(conn), 1);
    // A week later "due in 5 days" no longer makes sense.
    run(conn, parse_date("2026-11-23").unwrap()).unwrap();
    assert_eq!(still_open(conn), 0);
}
