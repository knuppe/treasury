//! Budget, goals and reports through the dispatcher (`Api::call`), with the reference scenario
//! of `reports::fixture` (values worked out by hand in that module's comment).

use std::time::Instant;

use serde_json::{Value, json};

use super::*;
use crate::reports::fixture::{self, Reference};

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Carlos", "Ana"]}),
    )
    .unwrap();
    (dir, api)
}

fn with_reference() -> (tempfile::TempDir, Api, Reference) {
    let (d, api) = api();
    let r = api
        .with_session(|s, _| Ok(fixture::seed_reference(&mut s.db)))
        .unwrap();
    (d, api, r)
}

fn cat(api: &Api, path: &str) -> String {
    api.with_session(|s, _| Ok(fixture::cat(s.db.conn(), path)))
        .unwrap()
}

fn lines_json(api: &Api) -> Value {
    let lines = api
        .with_session(|s, _| Ok(fixture::reference_budget_lines(s.db.conn())))
        .unwrap();
    serde_json::to_value(lines).unwrap()
}

fn find<'a>(arr: &'a Value, key: &str, val: &str) -> &'a Value {
    arr.as_array()
        .unwrap()
        .iter()
        .find(|x| x[key] == val)
        .unwrap_or_else(|| panic!("{key} = {val}"))
}

#[test]
fn budget_commands_and_undo() {
    let (_d, api, _) = with_reference();
    let st = api
        .call(
            "budget.set_lines",
            json!({"month": "2026-10", "lines": lines_json(&api)}),
        )
        .unwrap();
    assert_eq!(st["month"], "2026-10");
    assert_eq!(st["totals"]["planned_expense"], 505_000);
    assert_eq!(st["totals"]["actual_expense"], 481_000);
    assert_eq!(st["totals"]["actual_income"], 1_020_000);
    let moradia = find(&st["lines"], "category_path", "Moradia");
    assert_eq!(
        (
            &moradia["planned"],
            &moradia["actual"],
            &moradia["committed"]
        ),
        (&json!(350_000), &json!(325_000), &json!(10_000))
    );
    let alim = find(&st["lines"], "category_path", "Alimentação");
    assert_eq!(
        (&alim["planned"], &alim["has_own_line"]),
        (&json!(65_000), &json!(false))
    );

    let cash = api
        .call("budget.get", json!({"month": "2026-10", "basis": "cash"}))
        .unwrap();
    assert_eq!(cash["basis"], "cash");
    assert_eq!(cash["totals"]["actual_expense"], 439_000);

    let copied = api
        .call(
            "budget.copy",
            json!({"from_month": "2026-10", "to_month": "2026-11"}),
        )
        .unwrap();
    assert_eq!(copied["month"], "2026-11");
    assert_eq!(copied["totals"]["planned_expense"], 505_000);
    assert_eq!(
        api.call("budget.list_months", Value::Null).unwrap(),
        json!(["2026-11", "2026-10"])
    );
    let sug = api
        .call(
            "budget.suggest",
            json!({"month": "2026-10", "months_back": 3, "basis": "accrual"}),
        )
        .unwrap();
    assert_eq!(
        sug[0],
        json!({"category_id": cat(&api, "Moradia"), "amount": 295_000})
    );
    assert_eq!(sug.as_array().unwrap().len(), 4);

    // Each write is a set of changes that can be undone.
    let changes: Vec<changes::ChangeSetSummary> =
        serde_json::from_value(api.call("changes.list", json!({})).unwrap()).unwrap();
    let copy = changes
        .iter()
        .find(|c| c.reason == "Orçamento de out/2026 copiado para nov/2026")
        .unwrap();
    assert!(
        changes
            .iter()
            .any(|c| c.reason == "Orçamento de out/2026 atualizado")
    );
    api.call("changes.undo", json!({"id": copy.id})).unwrap();
    assert_eq!(
        api.call("budget.list_months", Value::Null).unwrap(),
        json!(["2026-10"])
    );

    assert_eq!(
        api.call("budget.get", json!({"month": "2026-13"}))
            .unwrap_err()
            .code(),
        "invalid"
    );
    assert_eq!(
        api.call(
            "budget.set_lines",
            json!({"month": "2026-10", "lines": [{"category_id": cat(&api, "Moradia"), "amount": -1}]})
        )
        .unwrap_err()
        .code(),
        "invalid"
    );
}

#[test]
fn reports_commands() {
    let (_d, api, r) = with_reference();
    api.call("settings.set", json!({"budget_basis": "accrual"}))
        .unwrap();

    let d = api
        .call("reports.dashboard", json!({"month": "2026-10"}))
        .unwrap();
    assert_eq!(d["flow"]["income"], 1_020_000);
    assert_eq!(d["flow"]["expense"], 481_000);
    assert_eq!(d["flow"]["missing_fx"], json!(["JPY"]));
    assert_eq!(d["flow_history"].as_array().unwrap().len(), 12);
    assert_eq!(d["top_categories"][0]["label"], "Moradia");
    assert_eq!(d["uncategorized_count"], 1);
    assert_eq!(d["budget_planned"], Value::Null);
    assert_eq!(d["currency"], "BRL");

    let nw = api
        .call("reports.net_worth", json!({"as_of": "2026-10-31"}))
        .unwrap();
    assert_eq!(
        (&nw["assets"], &nw["liabilities"], &nw["net"]),
        (&json!(1_109_766), &json!(62_000), &json!(1_047_766))
    );
    assert_eq!(nw["estimated"], true);
    let hist = api
        .call(
            "reports.net_worth_history",
            json!({"months": 3, "end_month": "2026-10"}),
        )
        .unwrap();
    let nets: Vec<i64> = hist
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["net"].as_i64().unwrap())
        .collect();
    assert_eq!(nets, vec![-190_000, 408_766, 1_047_766]);

    let flow = api
        .call(
            "reports.month_flow",
            json!({"from_month": "2026-10", "to_month": "2026-11", "basis": "cash"}),
        )
        .unwrap();
    assert_eq!(flow[0]["expense"], 439_000);
    assert_eq!(flow[1]["expense"], 62_000);
    assert_eq!(flow[1]["savings_rate"], Value::Null);

    let sp = api
        .call(
            "reports.spending",
            json!({"from": "2026-10-01", "to": "2026-10-31", "group_by": "member", "basis": "accrual"}),
        )
        .unwrap();
    assert_eq!(sp["total"], 481_000);
    assert_eq!(find(&sp["groups"], "label", "Carlos")["amount"], 420_000);

    let sk = api
        .call(
            "reports.sankey",
            json!({"from": "2026-10-01", "to": "2026-10-31", "basis": "accrual"}),
        )
        .unwrap();
    let savings = sk["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["target"] == "savings")
        .unwrap();
    assert_eq!(savings["value"], 539_000);
    let sk = api
        .call(
            "reports.sankey",
            json!({"from": "2026-11-01", "to": "2026-11-30", "basis": "cash"}),
        )
        .unwrap();
    assert_eq!(
        sk["links"][0],
        json!({"source": "deficit", "target": "hub", "value": 62_000})
    );
    // Without a basis in the request, the flow diagram uses its own setting (cash basis), not the
    // budget's.
    let sk = api
        .call(
            "reports.sankey",
            json!({"from": "2026-11-01", "to": "2026-11-30"}),
        )
        .unwrap();
    assert_eq!(sk["basis"], "cash");
    assert_eq!(sk["links"][0]["value"], 62_000);
    assert!(sk["card_bills_paid"].as_i64().unwrap() >= 0);
    api.call("settings.set", json!({"flow_basis": "accrual"}))
        .unwrap();
    let sk = api
        .call(
            "reports.sankey",
            json!({"from": "2026-10-01", "to": "2026-10-31"}),
        )
        .unwrap();
    assert_eq!(sk["basis"], "accrual");
    api.call("settings.set", json!({"flow_basis": "cash"}))
        .unwrap();

    let cal = api
        .call(
            "reports.calendar",
            json!({"from": "2026-10-01", "to": "2026-10-31"}),
        )
        .unwrap();
    assert_eq!(cal["days"].as_array().unwrap().len(), 31);
    assert_eq!(cal["days"][9]["expense"], 300_000);
    assert_eq!(cal["days"][0]["count"], 0);

    let ev = api
        .call(
            "reports.monthly_evolution",
            json!({"months": 3, "end_month": "2026-10", "top_n": 2}),
        )
        .unwrap();
    assert_eq!(ev["moving_avg_3"], json!([290_000, 345_617, 390_745]));
    assert_eq!(ev["series"][2]["label"], "Outras");

    let cmp = api
        .call(
            "reports.compare",
            json!({"a": {"from": "2026-09-01", "to": "2026-09-30"}, "b": {"from": "2026-10-01", "to": "2026-10-31"}}),
        )
        .unwrap();
    assert_eq!(
        (&cmp["total_a"], &cmp["total_b"]),
        (&json!(401_234), &json!(481_000))
    );
    assert_eq!(cmp["rows"][0]["label"], "Lazer");

    let bm = api
        .call(
            "reports.by_member",
            json!({"from": "2026-10-01", "to": "2026-10-31"}),
        )
        .unwrap();
    assert_eq!(bm["members"][2]["label"], "Sem membro");
    assert_eq!(bm["categories"][0]["values"], json!([300_000, 0, 25_000]));

    let up = api
        .call(
            "reports.upcoming",
            json!({"from": "2026-10-14", "days": 30}),
        )
        .unwrap();
    assert_eq!(up.as_array().unwrap().len(), 3);
    assert_eq!(up[2]["kind"], "card_due");
    assert_eq!(up[2]["ref_id"], json!(r.nov_statement));

    let ex = api
        .call(
            "reports.explain",
            json!({"metric": "spending", "params": {"from": "2026-10-01", "to": "2026-10-31"}}),
        )
        .unwrap();
    assert_eq!(ex["value"], 481_000);
    let sum: i64 = ex["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["amount"].as_i64().unwrap())
        .sum();
    assert_eq!(sum, 481_000);
    let ex = api
        .call(
            "reports.explain",
            json!({"metric": "account_balance", "params": {"account_id": r.checking, "as_of": "2026-10-14"}}),
        )
        .unwrap();
    assert!(
        ex["formula"]
            .as_str()
            .unwrap()
            .starts_with("saldo inicial de R$ 1.000,00 + 10 lançamentos efetivados até 14/10/2026")
    );

    // Without `basis`, the reports follow the setting.
    api.call("settings.set", json!({"budget_basis": "cash"}))
        .unwrap();
    let flow = api
        .call(
            "reports.month_flow",
            json!({"from_month": "2026-10", "to_month": "2026-10"}),
        )
        .unwrap();
    assert_eq!(flow[0]["expense"], 439_000);

    // Errors in Portuguese, with a stable code.
    let e = api
        .call(
            "reports.sankey",
            json!({"from": "2026-10-31", "to": "2026-10-01"}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    assert!(e.to_string().contains("posterior"));
    assert_eq!(
        api.call("reports.explain", json!({"metric": "nada", "params": {}}))
            .unwrap_err()
            .code(),
        "invalid"
    );
    assert_eq!(
        api.call("reports.nada", Value::Null).unwrap_err().code(),
        "unknown_command"
    );
}

#[test]
fn goals_commands() {
    let (_d, api, r) = with_reference();
    let g = api
        .call(
            "goals.create",
            json!({"name": "Reserva", "kind": "emergency", "target_amount": 3_000_000,
                   "initial_amount": 50_000, "account_ids": [r.savings]}),
        )
        .unwrap();
    let id = g["id"].as_str().unwrap().to_string();
    // R$ 500 + R$ 1,000 transfer to savings (the planned one does not count).
    assert_eq!(g["current_amount"], 150_000);
    assert_eq!(g["account_ids"], json!([r.savings]));
    assert_eq!(
        api.call("goals.list", Value::Null)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let g = api
        .call(
            "goals.update",
            json!({"id": id, "name": "Reserva de emergência", "target_date": "2030-12-31"}),
        )
        .unwrap();
    assert_eq!(g["name"], "Reserva de emergência");
    assert_eq!(g["target_date"], "2030-12-31");
    assert!(g["monthly_needed"].as_i64().unwrap() > 0);
    let er = api.call("goals.emergency_reserve", json!({})).unwrap();
    assert_eq!(er["goal_id"], json!(id));
    assert_eq!(er["current_amount"], 150_000);
    assert_eq!(er["target_months"], 6);
    api.call("goals.update", json!({"id": id, "archived": true}))
        .unwrap();
    assert!(
        api.call("goals.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        api.call("goals.list", json!({"include_archived": true}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    api.call("goals.delete", json!({"id": id})).unwrap();
    assert_eq!(
        api.call("goals.get", json!({"id": id})).unwrap_err().code(),
        "not_found"
    );
    assert_eq!(
        api.call(
            "goals.create",
            json!({"name": "x", "kind": "travel", "target_amount": 0})
        )
        .unwrap_err()
        .code(),
        "invalid"
    );
}

#[test]
fn read_only_mcp_can_read_but_not_write() {
    let (_d, api, _) = with_reference();
    let mcp = Actor::Mcp {
        client: "claude-desktop".into(),
        read_only: true,
    };
    for (cmd, args) in [
        ("reports.dashboard", json!({"month": "2026-10"})),
        (
            "reports.explain",
            json!({"metric": "net_worth", "params": {}}),
        ),
        ("budget.get", json!({"month": "2026-10"})),
        ("budget.suggest", json!({"month": "2026-10"})),
        ("goals.list", json!({})),
        ("goals.emergency_reserve", json!({})),
    ] {
        assert!(api.call_as(&mcp, cmd, args).is_ok(), "{cmd}");
    }
    for (cmd, args) in [
        ("budget.set_lines", json!({"month": "2026-10", "lines": []})),
        (
            "budget.copy",
            json!({"from_month": "2026-10", "to_month": "2026-11"}),
        ),
        (
            "goals.create",
            json!({"name": "x", "kind": "travel", "target_amount": 1}),
        ),
    ] {
        assert!(
            matches!(api.call_as(&mcp, cmd, args), Err(Error::Forbidden(_))),
            "{cmd}"
        );
    }
}

/// Performance smoke test: ~20 thousand transactions over 3 years, including card, splits,
/// transfers and foreign currency. The limit is generous (debug build on a loaded machine);
/// the measured time shows with `--nocapture`.
#[test]
fn dashboard_is_fast_with_20k_transactions() {
    let (_d, api) = api();
    let n = api
        .with_session(|s, _| {
            let (checking, card, savings, usd, cats, income) = fixture::write(&mut s.db, |tx| {
                let checking = fixture::account(tx, "Corrente", "checking", "BRL", 1_000_000, "2023-11-01");
                let card = fixture::card(tx, "Cartão", 25, 5);
                let savings = fixture::account(tx, "Poupança", "savings", "BRL", 0, "2023-11-01");
                let usd = fixture::account(tx, "Dólar", "foreign", "USD", 100_000, "2023-11-01");
                fixture::fx_rate(tx, "USD", "BRL", "2023-11-01", "5.1");
                fixture::fx_rate(tx, "USD", "BRL", "2026-10-01", "5.4");
                let mut stmt = tx.prepare(
                    "SELECT id FROM category WHERE kind = 'expense' AND parent_id IS NOT NULL ORDER BY id",
                )?;
                let cats: Vec<String> = stmt
                    .query_map([], |r| r.get(0))?
                    .collect::<rusqlite::Result<_>>()?;
                let income = fixture::cat(tx, "Salário");
                Ok((checking, card, savings, usd, cats, income))
            });
            let tx = s.db.conn_mut().transaction()?;
            let start = crate::time::parse_date("2023-11-01")?;
            let days = 1096i64;
            let now = crate::time::now_utc();
            let mut seed: u64 = 42;
            let mut next = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (seed >> 33) as i64
            };
            let total = 20_000i64;
            {
                let mut ins = tx.prepare(
                    "INSERT INTO txn (id, account_id, date, amount, currency, kind, status, description, category_id,
                                      transfer_id, statement_id, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
                )?;
                let mut split = tx.prepare(
                    "INSERT INTO txn_split (id, txn_id, category_id, amount, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                )?;
                for i in 0..total {
                    let date = crate::time::fmt_date(start + chrono::Duration::days(i * days / total));
                    let id = crate::ids::new_id();
                    let cat = &cats[(next() as usize) % cats.len()];
                    let amount = -(next() % 50_000 + 500);
                    let status = if date.as_str() > "2026-10-14" { "planned" } else { "cleared" };
                    match i % 40 {
                        0 => {
                            ins.execute(rusqlite::params![id, checking, date, 900_000, "BRL", "income", status, "salário", income, None::<String>, None::<String>, now])?;
                        }
                        1 => {
                            let tid = crate::ids::new_id();
                            ins.execute(rusqlite::params![id, checking, date, -20_000, "BRL", "transfer", status, "transf.", None::<String>, tid, None::<String>, now])?;
                            ins.execute(rusqlite::params![crate::ids::new_id(), savings, date, 20_000, "BRL", "transfer", status, "transf.", None::<String>, tid, None::<String>, now])?;
                        }
                        2 => {
                            ins.execute(rusqlite::params![id, usd, date, amount / 10, "USD", "expense", status, "exterior", cat, None::<String>, None::<String>, now])?;
                        }
                        k if k % 4 == 0 => {
                            let st = crate::cards::statement_for(&tx, &card, &date)?;
                            ins.execute(rusqlite::params![id, card, date, amount, "BRL", "expense", status, "cartão", cat, None::<String>, st, now])?;
                        }
                        k if k % 13 == 0 => {
                            ins.execute(rusqlite::params![id, checking, date, amount * 2, "BRL", "expense", status, "dividida", None::<String>, None::<String>, None::<String>, now])?;
                            let other = &cats[(next() as usize) % cats.len()];
                            split.execute(rusqlite::params![crate::ids::new_id(), id, cat, amount, now])?;
                            split.execute(rusqlite::params![crate::ids::new_id(), id, other, amount, now])?;
                        }
                        _ => {
                            ins.execute(rusqlite::params![id, checking, date, amount, "BRL", "expense", status, "compra", cat, None::<String>, None::<String>, now])?;
                        }
                    }
                }
            }
            tx.commit()?;
            let n: i64 = s.db.conn().query_row("SELECT count(*) FROM txn", [], |r| r.get(0))?;
            Ok(n)
        })
        .unwrap();
    assert!(n >= 20_000, "{n} transactions");

    let time = |cmd: &str, args: Value| {
        let t = Instant::now();
        let v = api.call(cmd, args).unwrap();
        (t.elapsed(), v)
    };
    // First call (prepares the queries) and a second one, already cached.
    let (first, d) = time("reports.dashboard", json!({"month": "2026-10"}));
    let (second, _) = time("reports.dashboard", json!({"month": "2026-10"}));
    assert!(d["flow"]["expense"].as_i64().unwrap() > 0);
    assert_eq!(d["flow_history"].as_array().unwrap().len(), 12);
    let (sankey, _) = time(
        "reports.sankey",
        json!({"from": "2026-01-01", "to": "2026-10-31"}),
    );
    let (evolution, _) = time(
        "reports.monthly_evolution",
        json!({"months": 24, "end_month": "2026-10"}),
    );
    let (history, _) = time(
        "reports.net_worth_history",
        json!({"months": 36, "end_month": "2026-10"}),
    );
    let (explain, ex) = time(
        "reports.explain",
        json!({"metric": "spending", "params": {"from": "2023-11-01", "to": "2026-10-31"}}),
    );
    assert_eq!(ex["truncated"], true);
    eprintln!(
        "{n} transactions: dashboard {first:?} (2nd call {second:?}), Sankey 10 months {sankey:?}, \
         evolution 24 months {evolution:?}, net worth 36 months {history:?}, explain 3 years {explain:?}"
    );
    assert!(
        first.as_secs_f64() < 5.0 && second.as_secs_f64() < 5.0,
        "slow dashboard: {first:?} / {second:?}"
    );
}
