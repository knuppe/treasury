//! Manual link between an installment and a transaction (`debt.link_payment`,
//! `debt.payment_candidates`) and how it shows up in Lançamentos.

use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

struct Fx {
    _dir: tempfile::TempDir,
    api: Api,
    acc: String,
    other: String,
    cat: String,
    loan: String,
    sched: Vec<Value>,
}

impl Fx {
    fn call(&self, cmd: &str, args: Value) -> Value {
        self.api
            .call(cmd, args)
            .unwrap_or_else(|e| panic!("{cmd}: {e}"))
    }
    fn err(&self, cmd: &str, args: Value) -> String {
        self.api
            .call(cmd, args)
            .expect_err(&format!("{cmd} deveria falhar"))
            .to_string()
    }
    fn txn(&self, account: &str, date: &str, amount: i64, description: &str) -> String {
        self.call(
            "transactions.create",
            json!({"account_id": account, "date": date, "amount": amount, "description": description}),
        )["id"]
            .as_str()
            .unwrap()
            .to_string()
    }
    fn inst(&self, n: usize) -> &Value {
        &self.sched[n - 1]
    }
    fn total(&self, n: usize) -> i64 {
        self.inst(n)["total"].as_i64().unwrap()
    }
    fn id(&self, n: usize) -> String {
        self.inst(n)["id"].as_str().unwrap().to_string()
    }
    fn schedule(&self) -> Vec<Value> {
        self.call("debt.schedule", json!({"id": self.loan}))
            .as_array()
            .unwrap()
            .clone()
    }
}

/// 12-installment payroll loan (due on the 10th from Feb 2026), debit account `acc` and
/// payments category "Juros e multas".
fn fx() -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let account = |name: &str| {
        api.call(
            "accounts.create",
            json!({"name": name, "kind": "checking", "opening_date": "2025-01-01"}),
        )
        .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let acc = account("Conta");
    let other = account("Outra");
    let cats = api.call("categories.list", json!({})).unwrap();
    let cat = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Juros e multas")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let loan = api
        .call(
            "debt.create",
            json!({"name": "Consignado", "kind": "payroll", "principal": 1_000_000, "annual_rate": "20",
                   "system": "price", "contract_date": "2026-01-10", "first_due_date": "2026-02-10",
                   "term_months": 12, "debit_account_id": acc, "category_id": cat}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut f = Fx {
        _dir: dir,
        api,
        acc,
        other,
        cat,
        loan,
        sched: vec![],
    };
    f.sched = f.schedule();
    f
}

#[test]
fn link_payment_marks_paid_fills_category_and_shows_in_transactions() {
    let f = fx();
    let t1 = f.txn(&f.acc, "2026-02-12", -(f.total(1) + 500), "DEB PARCELA");
    let r = f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(1), "txn_id": t1}),
    );
    assert_eq!(r["paid"], true, "{r}");
    assert_eq!(r["paid_date"], "2026-02-12");
    assert_eq!(r["paid_amount"], f.total(1) + 500);
    assert_eq!(r["txn_id"], t1.as_str());
    assert_eq!(r["txn_description"], "DEB PARCELA");
    assert_eq!(r["txn_account_name"], "Conta");
    assert_eq!(r["version_number"], 1);
    // No category: takes the contract's payments category.
    let v = f.call("transactions.get", json!({"id": t1}));
    assert_eq!(v["category_id"], f.cat.as_str());
    assert_eq!(v["loan_id"], f.loan.as_str());
    assert_eq!(v["loan_name"], "Consignado");
    assert_eq!(v["loan_installment_id"], f.id(1).as_str());
    assert_eq!(v["loan_installment_number"], 1);
    // The Lançamentos query returns the link, without repeating rows.
    let page = f.call("transactions.query", json!({"account_ids": [f.acc]}));
    let items = page["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["loan_installment_number"], 1);
    // The history records the link with the installment and the contract.
    let hist = f.call("changes.list", json!({"limit": 1}));
    assert!(
        hist.to_string()
            .contains("Parcela 1 de Consignado ligada ao lançamento"),
        "{hist}"
    );

    // The same outflow does not pay two installments.
    let e = f.err(
        "debt.link_payment",
        json!({"installment_id": f.id(2), "txn_id": t1}),
    );
    assert!(
        e.contains("já está ligado à parcela 1 de Consignado"),
        "{e}"
    );
    // Linking again to the same installment changes nothing.
    f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(1), "txn_id": t1}),
    );

    // Swapping the installment's transaction: the previous one becomes free.
    let t2 = f.txn(&f.other, "2026-02-10", -f.total(1), "PIX BANCO");
    let r = f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(1), "txn_id": t2}),
    );
    assert_eq!(r["txn_id"], t2.as_str());
    assert_eq!(r["paid_amount"], f.total(1));
    assert_eq!(r["paid_date"], "2026-02-10");
    let v1 = f.call("transactions.get", json!({"id": t1}));
    assert_eq!(v1["loan_id"], Value::Null);
    assert_eq!(v1["loan_installment_number"], Value::Null);
    // Now the first one can pay installment 2.
    f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(2), "txn_id": t1}),
    );

    // Unlinking (undoing the payment) clears the link on both sides.
    f.call("debt.unmark_paid", json!({"id": f.id(1)}));
    let s = f.schedule();
    assert_eq!(s[0]["paid"], false);
    assert_eq!(s[0]["txn_id"], Value::Null);
    assert_eq!(s[0]["txn_description"], Value::Null);
    let v2 = f.call("transactions.get", json!({"id": t2}));
    assert_eq!(v2["loan_installment_id"], Value::Null);
}

#[test]
fn link_payment_validation() {
    let f = fx();
    let inst = f.id(1);
    let link = |txn: &str| {
        f.err(
            "debt.link_payment",
            json!({"installment_id": inst, "txn_id": txn}),
        )
    };
    // An inflow does not pay an installment.
    let income = f.txn(&f.acc, "2026-02-10", 50_000, "Estorno");
    assert!(link(&income).contains("precisa ser uma saída"));
    // Leg of a transfer between accounts.
    let tr = f.call(
        "transactions.create_transfer",
        json!({"from_account_id": f.acc, "to_account_id": f.other, "date": "2026-02-10", "amount": f.total(1)}),
    );
    let leg = tr["from"]["id"].as_str().unwrap();
    assert!(link(leg).contains("transferência"), "{}", link(leg));
    // Planned transaction.
    let planned = f.call(
        "transactions.create",
        json!({"account_id": f.acc, "date": "2026-02-10", "amount": -f.total(1),
               "description": "Agendado", "status": "planned"}),
    )["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(link(&planned).contains("previsto"));
    // Nonexistent ones.
    assert!(link("nao-existe").contains("lançamento"));
    let t = f.txn(&f.acc, "2026-02-10", -f.total(1), "Parcela");
    let e = f.err(
        "debt.link_payment",
        json!({"installment_id": "nao-existe", "txn_id": t}),
    );
    assert!(e.contains("parcela"), "{e}");
    // An extraordinary amortization posted to the account already belongs to the contract.
    let l = f.call(
        "debt.add_extra_payment",
        json!({"loan_id": f.loan, "date": "2026-03-01", "amount": 100_000, "mode": "reduce_term"}),
    );
    assert!(l["id"].is_string());
    let extra = f.call(
        "transactions.query",
        json!({"account_ids": [f.acc], "text": "Amortização extraordinária"}),
    )["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    // The amortization recalculates the open installments (new ids).
    let first = f.schedule()[0]["id"].as_str().unwrap().to_string();
    let e = f.err(
        "debt.link_payment",
        json!({"installment_id": first, "txn_id": extra}),
    );
    assert!(e.contains("amortização extraordinária"), "{e}");
    // Nothing was written to the installment.
    assert_eq!(f.schedule()[0]["paid"], false);
}

#[test]
fn link_keeps_existing_category_and_skips_other_kinds() {
    let f = fx();
    let cats = f.call("categories.list", json!({}));
    let mercado = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Mercado")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let t = f.call(
        "transactions.create",
        json!({"account_id": f.acc, "date": "2026-02-10", "amount": -f.total(1),
               "description": "Parcela", "category_id": mercado}),
    )["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(1), "txn_id": t}),
    );
    let v = f.call("transactions.get", json!({"id": t}));
    assert_eq!(v["category_id"], mercado.as_str());
    // An adjustment (uncategorized by nature) is linked but gets no category.
    let adj = f.call(
        "transactions.create",
        json!({"account_id": f.acc, "date": "2026-03-10", "amount": -f.total(2),
               "description": "Débito", "kind": "adjustment"}),
    )["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(2), "txn_id": adj}),
    );
    let v = f.call("transactions.get", json!({"id": adj}));
    assert_eq!(v["category_id"], Value::Null);
    assert_eq!(v["loan_installment_number"], 2);
}

#[test]
fn payment_candidates_by_amount_then_date() {
    let f = fx();
    let due = f.inst(1)["due_date"].as_str().unwrap().to_string();
    assert_eq!(due, "2026-02-10");
    let total = f.total(1);
    let far = f.txn(&f.other, "2026-02-20", -total, "Mesmo valor, 10 dias");
    let near = f.txn(&f.acc, "2026-02-11", -total, "Mesmo valor, 1 dia");
    let off = f.txn(&f.acc, "2026-02-10", -(total + 5_000), "Valor diferente");
    f.txn(&f.acc, "2026-03-05", -total, "Fora da janela");
    // A card purchase is not in the list (the installment is paid from the account).
    let card = f.call(
        "accounts.create",
        json!({"name": "Cartão", "kind": "credit_card", "opening_date": "2026-01-01",
               "card": {"closing_day": 1, "due_day": 10}}),
    )["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.txn(&card, "2026-02-10", -total, "Compra no cartão");
    f.txn(&f.acc, "2026-02-10", total, "Entrada");
    f.call(
        "transactions.create",
        json!({"account_id": f.acc, "date": "2026-02-10", "amount": -total,
               "description": "Previsto", "status": "planned"}),
    );
    f.call(
        "transactions.create_transfer",
        json!({"from_account_id": f.acc, "to_account_id": f.other, "date": "2026-02-10", "amount": total}),
    );
    let c = f.call(
        "debt.payment_candidates",
        json!({"installment_id": f.id(1)}),
    );
    let ids: Vec<&str> = c
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["txn_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, [near.as_str(), far.as_str(), off.as_str()], "{c}");
    assert_eq!(c[0]["in_debit_account"], true);
    assert_eq!(c[0]["account_name"], "Conta");
    assert_eq!(c[0]["amount"], -total);
    assert_eq!(c[0]["currency"], "BRL");
    assert_eq!(c[1]["in_debit_account"], false);
    // Already linked to an installment: leaves the candidates (of this one and the others).
    f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(1), "txn_id": near}),
    );
    let c = f.call(
        "debt.payment_candidates",
        json!({"installment_id": f.id(1)}),
    );
    assert!(!c.to_string().contains(&near), "{c}");
    // It is a read: allowed for read-only MCP clients.
    assert!(super::api::READ_ONLY.contains(&"payment_candidates"));
}

#[test]
fn deleting_a_linked_transaction_reopens_the_installment() {
    let f = fx();
    let t = f.txn(&f.acc, "2026-02-10", -f.total(1), "Parcela");
    f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(1), "txn_id": t}),
    );
    let del = f.call("transactions.delete", json!({"ids": [t]}));
    let s = f.schedule();
    assert_eq!(s[0]["paid"], false);
    assert_eq!(s[0]["txn_id"], Value::Null);
    assert_eq!(s[0]["paid_date"], Value::Null);
    // Undoing the deletion brings back the transaction and the link.
    f.call("changes.undo", json!({"id": del["change_set_id"]}));
    let s = f.schedule();
    assert_eq!(s[0]["paid"], true);
    assert_eq!(s[0]["txn_id"], t.as_str());
    // The extraordinary amortization loses only the transaction (it stays registered).
    f.call(
        "debt.add_extra_payment",
        json!({"loan_id": f.loan, "date": "2026-03-01", "amount": 100_000, "mode": "reduce_term"}),
    );
    let extra = f.call(
        "transactions.query",
        json!({"account_ids": [f.acc], "text": "Amortização extraordinária"}),
    )["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let before = f.schedule();
    assert!(before.len() < 12, "shortens the term: {}", before.len());
    f.call("transactions.delete", json!({"ids": [extra]}));
    let after = f.schedule();
    assert_eq!(after.len(), before.len());
    assert_eq!(
        after.last().unwrap()["total"],
        before.last().unwrap()["total"]
    );
}

/// Layered month close (feedback 01a19866): the paid installment is debt service even when
/// posted under another category; the flow bridge separates consumption and installment (with
/// interest and amortization from the schedule), and the Dívidas group adds up the installment.
#[test]
fn paid_installment_counts_as_debt_service_in_flow_and_groups() {
    let f = fx();
    let cats = f.call("categories.list", json!({}));
    let moradia = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Moradia")
        .unwrap()["id"]
        .clone();
    let t1 = f.txn(&f.acc, "2026-02-12", -f.total(1), "DEB PARCELA");
    // Posted under Moradia before linking: the category stays (it already had one).
    f.call(
        "transactions.update",
        json!({"id": t1, "category_id": moradia}),
    );
    f.call(
        "debt.link_payment",
        json!({"installment_id": f.id(1), "txn_id": t1}),
    );
    f.txn(&f.acc, "2026-02-15", -30_000, "MERCADO");
    let flow = f.call(
        "reports.month_flow",
        json!({"from_month": "2026-02", "to_month": "2026-02"}),
    );
    let m = &flow[0];
    let b = &m["bridge"];
    assert_eq!(b["debt_service"], f.total(1), "{m}");
    assert_eq!(
        b["consumption"].as_i64().unwrap(),
        m["expense"].as_i64().unwrap() - f.total(1)
    );
    assert_eq!(b["debt_interest"], f.inst(1)["interest"]);
    assert_eq!(b["debt_amortization"], f.inst(1)["amortization"]);
    assert_eq!(b["adjustments"], 0);
    let groups = f.call(
        "reports.spend_groups",
        json!({"from": "2026-02", "to": "2026-02"}),
    );
    let debt = groups["total"]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["group"] == "debt")
        .unwrap()
        .clone();
    assert_eq!(debt["amount"], f.total(1), "{groups}");
}
