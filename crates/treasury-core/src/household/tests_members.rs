//! Finances per member: effective holder, shares of joint accounts, dashboard, health, overview
//! and each person's profile.

use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

fn setup() -> (tempfile::TempDir, Api, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana", "Bruno"]}),
    )
    .unwrap();
    let members = api.call("household.list_members", json!({})).unwrap();
    let id = |n: &str| {
        members
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == n)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    (dir, api, id("Ana"), id("Bruno"))
}

fn account(api: &Api, name: &str, owners: &[&str], opening: i64) -> String {
    api.call(
        "accounts.create",
        json!({"name": name, "kind": "checking", "member_ids": owners,
               "opening_balance": opening, "opening_date": "2026-01-01"}),
    )
    .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn txn(api: &Api, account: &str, amount: i64, desc: &str, member: Option<&str>) {
    let today = crate::time::fmt_date(crate::time::today_local());
    let mut a =
        json!({"account_id": account, "date": today, "amount": amount, "description": desc});
    if let Some(m) = member {
        a["member_id"] = json!(m);
    }
    api.call("transactions.create", a).unwrap();
}

fn i(v: &Value, ptr: &str) -> i64 {
    v.pointer(ptr)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("{ptr} in {v}"))
}

#[test]
fn each_member_sees_what_is_theirs() {
    let (_d, api, ana, bruno) = setup();
    let a_acc = account(&api, "Conta da Ana", &[&ana], 100_000);
    let joint = account(&api, "Conjunta", &[&ana, &bruno], 50_000);
    // Salary in Ana's account, no holder: it is Ana's (sole owner of the account).
    txn(&api, &a_acc, 500_000, "Salário", None);
    // Groceries in the joint account with no holder: the family's; with a holder, that person's.
    txn(&api, &joint, -30_000, "Mercado", None);
    txn(&api, &joint, -12_000, "Farmácia", Some(&bruno));

    let fam = api.call("reports.dashboard", json!({})).unwrap();
    assert_eq!(
        (i(&fam, "/flow/income"), i(&fam, "/flow/expense")),
        (500_000, 42_000)
    );
    let d_ana = api
        .call("reports.dashboard", json!({"member_id": ana}))
        .unwrap();
    assert_eq!(
        (i(&d_ana, "/flow/income"), i(&d_ana, "/flow/expense")),
        (500_000, 0)
    );
    let d_bruno = api
        .call("reports.dashboard", json!({"member_id": bruno}))
        .unwrap();
    assert_eq!(
        (i(&d_bruno, "/flow/income"), i(&d_bruno, "/flow/expense")),
        (0, 12_000)
    );
    assert!(d_bruno["budget_planned"].is_null());
    // Net worth: own account in full and half of the joint one.
    let joint_balance = 50_000 - 42_000;
    assert_eq!(i(&d_ana, "/net_worth/net"), 600_000 + joint_balance / 2);
    assert_eq!(i(&d_bruno, "/net_worth/net"), joint_balance / 2);

    let o = api.call("household.members_overview", json!({})).unwrap();
    assert_eq!(o["members"].as_array().unwrap().len(), 2);
    let ov = |n: &str| {
        o["members"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["member"]["name"] == n)
            .unwrap()
            .clone()
    };
    assert_eq!(i(&ov("Ana"), "/month/income"), 500_000);
    assert_eq!(i(&ov("Bruno"), "/month/expense"), 12_000);
    assert_eq!(i(&ov("Bruno"), "/accounts"), 1);
    assert_eq!(i(&o, "/shared/expense"), 30_000);
    assert_eq!(ov("Ana")["has_own_profile"], false);

    // Financial health per member (same model, with each person's data).
    let h = api
        .call("health.score", json!({"member_id": bruno}))
        .unwrap();
    assert!(h["summary"].is_string(), "{h}");
    let flow = api
        .call("reports.month_flow", json!({"from_month": "2026-01", "to_month": crate::time::month_of(crate::time::today_local()), "member_id": ana}))
        .unwrap();
    assert_eq!(flow.as_array().unwrap().last().unwrap()["income"], 500_000);
    assert!(
        api.call("reports.dashboard", json!({"member_id": "nao-existe"}))
            .is_err()
    );
}

#[test]
fn each_member_has_own_profile_and_agent_tone() {
    let (_d, api, _ana, bruno) = setup();
    api.call(
        "accounts.create",
        json!({"name": "Conta", "kind": "checking"}),
    )
    .unwrap();
    let p = api
        .call("profile.get", json!({"member_id": bruno}))
        .unwrap();
    assert_eq!(
        (p["level"].clone(), p["source"].clone()),
        (json!("avancado"), json!("familia"))
    );
    let p = api
        .call(
            "profile.answer",
            json!({"member_id": bruno, "answers": {"confianca": ["nada"], "controle": ["nunca"], "termos": ["perdido"], "tom": ["simples"]}}),
        )
        .unwrap();
    assert_eq!(p["level"], "iniciante");
    // The family's does not change; Bruno's is kept.
    assert_eq!(
        api.call("profile.get", json!({})).unwrap()["level"],
        "avancado"
    );
    assert_eq!(
        api.call("profile.get", json!({"member_id": bruno}))
            .unwrap()["source"],
        "questionario"
    );
    let line = api
        .with_session(|s, _| crate::profile::prompt_line_for(s.db.conn(), Some(&bruno)))
        .unwrap();
    assert!(
        line.contains("Bruno") && line.contains("beginner"),
        "{line}"
    );
    let o = api.call("household.members_overview", json!({})).unwrap();
    let b = o["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["member"]["name"] == "Bruno")
        .unwrap();
    assert_eq!(
        (b["level"].clone(), b["has_own_profile"].clone()),
        (json!("iniciante"), json!(true))
    );
}
