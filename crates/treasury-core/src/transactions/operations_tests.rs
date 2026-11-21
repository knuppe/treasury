//! Linking transactions to portfolio operations, capital shares and goals, through the API.

use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

struct T {
    _dir: tempfile::TempDir,
    api: Api,
    bank: String,
    broker: String,
}

fn id(v: Value) -> String {
    v["id"].as_str().unwrap().to_string()
}

fn setup() -> T {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let bank = id(api
        .call(
            "accounts.create",
            json!({"name": "Sicoob", "kind": "checking", "opening_date": "2026-01-01"}),
        )
        .unwrap());
    let broker = id(api
        .call(
            "accounts.create",
            json!({"name": "Banco (investimentos)", "kind": "brokerage", "opening_date": "2026-01-01"}),
        )
        .unwrap());
    T {
        _dir: dir,
        api,
        bank,
        broker,
    }
}

impl T {
    fn txn(&self, date: &str, amount: i64, desc: &str) -> String {
        let kind = if amount < 0 { "expense" } else { "income" };
        id(self
            .api
            .call(
                "transactions.create",
                json!({"account_id": self.bank, "date": date, "amount": amount,
                       "kind": kind, "description": desc}),
            )
            .unwrap())
    }
    fn get(&self, txn: &str) -> Value {
        self.api
            .call("transactions.get", json!({"id": txn}))
            .unwrap()
    }
    fn cdb_purchase(&self, date: &str) -> String {
        let asset = id(self
            .api
            .call(
                "assets.create",
                json!({"name": "CDB Sicoob 110% CDI", "class": "fixed_income",
                       "fixed_income": {"kind": "cdb", "indexer": "cdi", "rate": "110",
                                        "maturity_date": "2028-01-01", "liquidity": "daily",
                                        "ir_exempt": false}}),
            )
            .unwrap());
        id(self
            .api
            .call(
                "portfolio.create_trade",
                json!({"asset_id": asset, "account_id": self.broker, "date": date, "side": "buy",
                       "quantity": "1", "price": "1000", "cash_movement": false}),
            )
            .unwrap())
    }
}

#[test]
fn a_bank_debit_becomes_the_contribution_of_a_cdb_and_goes_back_on_unlink() {
    let t = setup();
    let food = id(t
        .api
        .call(
            "categories.create",
            json!({"name": "Diversos", "kind": "expense"}),
        )
        .unwrap());
    let trade = t.cdb_purchase("2026-10-05");
    let txn = t.txn("2026-10-06", -100_000, "APLICACAO CDB");
    t.api
        .call(
            "transactions.set_category",
            json!({"ids": [txn], "category_id": food}),
        )
        .unwrap();

    // The CDB shows up as the most likely candidate (same amount, 1 day apart).
    let c = t
        .api
        .call("transactions.operation_candidates", json!({"txn_id": txn}))
        .unwrap();
    assert_eq!(c[0]["kind"], "trade", "{c}");
    assert_eq!(c[0]["target_id"], trade.as_str());
    assert!(
        c[0]["reason"].as_str().unwrap().contains("mesmo valor"),
        "{c}"
    );

    let r = t
        .api
        .call(
            "transactions.link_operation",
            json!({"txn_id": txn, "kind": "trade", "target_id": trade}),
        )
        .unwrap();
    assert!(r["change_set_id"].is_string());
    assert!(
        r["operation"]["label"]
            .as_str()
            .unwrap()
            .starts_with("Aporte em CDB Sicoob"),
        "{r}"
    );
    let v = t.get(&txn);
    assert_eq!(v["kind"], "investment");
    assert!(v["category_id"].is_null());
    assert_eq!(v["operation"]["kind"], "trade");

    // The operation already has an owner: it does not show up again nor accept another transaction.
    let other = t.txn("2026-10-06", -100_000, "APLICACAO CDB");
    let c = t
        .api
        .call(
            "transactions.operation_candidates",
            json!({"txn_id": other}),
        )
        .unwrap();
    assert!(
        c.as_array().unwrap().iter().all(|x| x["kind"] != "trade"),
        "{c}"
    );
    assert!(
        t.api
            .call(
                "transactions.link_operation",
                json!({"txn_id": other, "kind": "trade", "target_id": trade}),
            )
            .is_err()
    );
    // Linked, it does not become a debt installment nor a transfer.
    assert!(
        t.api
            .call(
                "transactions.update",
                json!({"id": txn, "transfer_account_id": t.broker}),
            )
            .is_err()
    );

    // Unlinking returns the type and category from before.
    t.api
        .call("transactions.unlink_operation", json!({"txn_id": txn}))
        .unwrap();
    let v = t.get(&txn);
    assert_eq!(v["kind"], "expense");
    assert_eq!(v["category_id"], food.as_str());
    assert!(v["operation"].is_null());
    // And the history redoes the link.
    let r = t
        .api
        .call(
            "transactions.link_operation",
            json!({"txn_id": txn, "kind": "trade", "target_id": trade}),
        )
        .unwrap();
    t.api
        .call("changes.undo", json!({"id": r["change_set_id"]}))
        .unwrap();
    assert_eq!(t.get(&txn)["kind"], "expense");
}

#[test]
fn wrong_direction_and_planned_entries_are_refused() {
    let t = setup();
    let trade = t.cdb_purchase("2026-10-05");
    let credit = t.txn("2026-10-06", 100_000, "RESGATE CDB");
    let e = t
        .api
        .call(
            "transactions.link_operation",
            json!({"txn_id": credit, "kind": "trade", "target_id": trade}),
        )
        .unwrap_err();
    assert!(e.to_string().contains("saída"), "{e}");
    let planned = id(t
        .api
        .call(
            "transactions.create",
            json!({"account_id": t.bank, "date": "2026-10-06", "amount": -100_000,
                   "kind": "expense", "status": "planned", "description": "APLICACAO"}),
        )
        .unwrap());
    assert!(
        t.api
            .call(
                "transactions.link_operation",
                json!({"txn_id": planned, "kind": "trade", "target_id": trade}),
            )
            .is_err()
    );
}

#[test]
fn programmed_savings_count_in_the_goal() {
    let t = setup();
    let goal = id(t
        .api
        .call(
            "goals.create",
            json!({"name": "Viagem", "kind": "travel", "target_amount": 1_000_000,
                   "initial_amount": 50_000}),
        )
        .unwrap());
    let a = t.txn("2026-08-10", -20_000, "POUPANCA PROGRAMADA VIAGEM");
    let b = t.txn("2026-09-10", -20_000, "POUPANCA PROGRAMADA VIAGEM");
    let c = t
        .api
        .call("transactions.operation_candidates", json!({"txn_id": a}))
        .unwrap();
    assert_eq!(c[0]["kind"], "goal", "{c}");
    assert_eq!(c[0]["target_id"], goal.as_str());
    for x in [&a, &b] {
        t.api
            .call(
                "transactions.link_operation",
                json!({"txn_id": x, "kind": "goal", "target_id": goal}),
            )
            .unwrap();
    }
    let g = t.api.call("goals.get", json!({"id": goal})).unwrap();
    assert_eq!(g["current_amount"], 90_000, "{g}");
    assert_eq!(g["linked_deposits"], 40_000);
    assert!(g["avg_monthly_contribution"].as_i64().unwrap() > 0, "{g}");
    // Changing the type by hand undoes the link.
    t.api
        .call("transactions.update", json!({"id": b, "kind": "expense"}))
        .unwrap();
    assert!(t.get(&b)["operation"].is_null());
    let g = t.api.call("goals.get", json!({"id": goal})).unwrap();
    assert_eq!(g["current_amount"], 70_000);
}

#[test]
fn capital_share_contributions_follow_explicit_links() {
    let t = setup();
    t.api
        .call("accounts.capital_share_set", json!({"account_id": t.bank}))
        .unwrap();
    // Text that does not match the shares' pattern: only enters linked.
    let odd = t.txn("2026-09-05", -10_000, "DEB INTEGR SOCIO");
    // Matches by text, but it is something else: linked to a goal, leaves the shares.
    let wrong = t.txn("2026-09-06", -5_000, "COTA CAPITAL CONSORCIO");
    let goal = id(t
        .api
        .call(
            "goals.create",
            json!({"name": "Carro", "kind": "purchase", "target_amount": 1_000_000}),
        )
        .unwrap());
    let before = t
        .api
        .call("accounts.capital_share", json!({"account_id": t.bank}))
        .unwrap();
    assert_eq!(before["total_contributions"], 5_000);
    t.api
        .call(
            "transactions.link_operation",
            json!({"txn_id": odd, "kind": "capital_share", "target_id": t.bank}),
        )
        .unwrap();
    t.api
        .call(
            "transactions.link_operation",
            json!({"txn_id": wrong, "kind": "goal", "target_id": goal}),
        )
        .unwrap();
    let after = t
        .api
        .call("accounts.capital_share", json!({"account_id": t.bank}))
        .unwrap();
    assert_eq!(after["total_contributions"], 10_000, "{after}");
    assert_eq!(t.get(&odd)["operation"]["label"], "Cotas capital · Sicoob");
}

#[test]
fn several_deposits_go_to_the_same_goal_with_a_preview() {
    let t = setup();
    let goal = id(t
        .api
        .call(
            "goals.create",
            json!({"name": "Reforma", "kind": "purchase", "target_amount": 1_000_000}),
        )
        .unwrap());
    let a = t.txn("2026-07-10", -20_000, "POUPANCA PROGRAMADA REFORMA");
    let b = t.txn("2026-08-10", -20_000, "POUPANCA PROGRAMADA REFORMA");
    let planned = id(t
        .api
        .call(
            "transactions.create",
            json!({"account_id": t.bank, "date": "2026-11-10", "amount": -20_000,
                   "kind": "expense", "status": "planned", "description": "POUPANCA PROGRAMADA REFORMA"}),
        )
        .unwrap());
    let other = id(t
        .api
        .call(
            "goals.create",
            json!({"name": "Carro", "kind": "purchase", "target_amount": 100}),
        )
        .unwrap());
    let done = t.txn("2026-09-10", -20_000, "POUPANCA PROGRAMADA REFORMA");
    t.api
        .call(
            "transactions.link_operation",
            json!({"txn_id": done, "kind": "goal", "target_id": other}),
        )
        .unwrap();
    let ids = json!([a, b, planned, done, a]);

    // Batch candidates: the goal cited in the texts comes first.
    let c = t
        .api
        .call("transactions.operation_candidates", json!({"txn_ids": ids}))
        .unwrap();
    assert_eq!(c[0]["target_id"], goal.as_str(), "{c}");

    let p = t
        .api
        .call(
            "transactions.operation_link_preview",
            json!({"txn_ids": ids, "kind": "goal", "target_id": goal}),
        )
        .unwrap();
    assert_eq!(p["label"], "Meta Reforma");
    assert_eq!(
        p["items"].as_array().unwrap().len(),
        4,
        "repeats count once"
    );
    assert_eq!(p["linkable"], 2);
    assert_eq!(p["totals"], json!([{"currency": "BRL", "amount": -40_000}]));
    let skipped: Vec<&str> = p["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["ok"] == false)
        .map(|i| i["reason"].as_str().unwrap())
        .collect();
    assert!(
        skipped.iter().any(|r| r.contains("previsto")),
        "{skipped:?}"
    );
    assert!(
        skipped.iter().any(|r| r.contains("Meta Carro")),
        "{skipped:?}"
    );

    let r = t
        .api
        .call(
            "transactions.link_operations",
            json!({"txn_ids": ids, "kind": "goal", "target_id": goal}),
        )
        .unwrap();
    assert_eq!(r["linked"], 2);
    assert_eq!(r["skipped"], 2);
    let g = t.api.call("goals.get", json!({"id": goal})).unwrap();
    assert_eq!(g["linked_deposits"], 40_000);
    assert_eq!(t.get(&done)["operation"]["target_id"], other.as_str());
    // A single set in the history: undoing brings both back.
    t.api
        .call("changes.undo", json!({"id": r["change_set_id"]}))
        .unwrap();
    assert!(t.get(&a)["operation"].is_null());
    assert!(t.get(&b)["operation"].is_null());
    // No one serving: clear error.
    let e = t
        .api
        .call(
            "transactions.link_operations",
            json!({"txn_ids": [planned], "kind": "goal", "target_id": goal}),
        )
        .unwrap_err();
    assert!(e.to_string().contains("nenhum"), "{e}");
}

#[test]
fn a_trade_takes_one_entry_even_in_a_batch() {
    let t = setup();
    let trade = t.cdb_purchase("2026-10-05");
    let a = t.txn("2026-10-05", -50_000, "APLICACAO CDB 1");
    let b = t.txn("2026-10-05", -50_000, "APLICACAO CDB 2");
    let p = t
        .api
        .call(
            "transactions.operation_link_preview",
            json!({"txn_ids": [a, b], "kind": "trade", "target_id": trade}),
        )
        .unwrap();
    assert_eq!(p["linkable"], 1);
    assert!(
        p["items"][1]["reason"]
            .as_str()
            .unwrap()
            .contains("um lançamento só")
    );
    // The portfolio does not show up among a batch's candidates.
    let c = t
        .api
        .call(
            "transactions.operation_candidates",
            json!({"txn_ids": [a, b]}),
        )
        .unwrap();
    assert!(
        c.as_array().unwrap().iter().all(|x| x["kind"] != "trade"),
        "{c}"
    );
}
