//! Goal sources (docs/goal-sources.md): the acceptance criteria of the Nov 3, 2026 request, through
//! the API and through MCP. Dates are relative to today (the portfolio and the reserve use the
//! system date).

use chrono::Duration;
use serde_json::{Value, json};

use crate::api::{Actor, Api};
use crate::mcp::inproc;
use crate::paths::TreasuryPaths;
use crate::time::{fmt_date, today_local};

fn setup() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    (dir, api)
}

fn id(v: &Value) -> String {
    v["id"].as_str().unwrap().to_string()
}

fn ago(days: i64) -> String {
    fmt_date(today_local() - Duration::days(days))
}

fn account(api: &Api, name: &str, kind: &str, opening: i64) -> String {
    id(&api
        .call(
            "accounts.create",
            json!({"name": name, "kind": kind, "opening_balance": opening, "opening_date": ago(400)}),
        )
        .unwrap())
}

/// 12% a year fixed-rate CDB, bought 400 days ago (17.5% IR on the gain at redemption).
fn cdb(api: &Api, acc: &str, name: &str, reais: u32) -> String {
    let asset = id(&api
        .call(
            "assets.create",
            json!({"name": name, "class": "fixed_income", "account_id": acc,
                   "fixed_income": {"kind": "cdb", "indexer": "prefixado", "rate": "12",
                                    "issue_date": ago(400), "liquidity": "daily",
                                    "ir_exempt": false}}),
        )
        .unwrap());
    api.call(
        "portfolio.create_trade",
        json!({"asset_id": asset, "account_id": acc, "date": ago(400), "side": "buy",
               "quantity": reais.to_string(), "price": "1", "cash_movement": false}),
    )
    .unwrap();
    asset
}

/// Stock with today's quote: the market value is exact.
fn stock(api: &Api, acc: &str, ticker: &str, qty: u32, price: &str) -> String {
    let asset = id(&api
        .call(
            "assets.create",
            json!({"ticker": ticker, "name": format!("{ticker} S.A."), "class": "stock_br"}),
        )
        .unwrap());
    api.call(
        "portfolio.create_trade",
        json!({"asset_id": asset, "account_id": acc, "date": ago(30), "side": "buy",
               "quantity": qty.to_string(), "price": price, "cash_movement": false}),
    )
    .unwrap();
    api.call(
        "market.set_price",
        json!({"asset_id": asset, "date": fmt_date(today_local()), "close": price}),
    )
    .unwrap();
    asset
}

fn goal(api: &Api, name: &str, kind: &str) -> String {
    id(&api
        .call(
            "goals.create",
            json!({"name": name, "kind": kind, "target_amount": 3_000_000}),
        )
        .unwrap())
}

fn link(api: &Api, goal: &str, kind: &str, r: &str) -> Value {
    api.call(
        "goals.link_source",
        json!({"goal_id": goal, "kind": kind, "ref_id": r}),
    )
    .unwrap()
}

fn get(api: &Api, goal: &str) -> Value {
    api.call("goals.get", json!({"id": goal})).unwrap()
}

/// Position values in the portfolio (base currency): (market, net).
fn position(api: &Api, asset: &str) -> (i64, Option<i64>) {
    let s = api
        .call("portfolio.summary", json!({"by_account": true}))
        .unwrap();
    let p = s["positions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["asset"]["id"] == asset)
        .unwrap()
        .clone();
    (
        p["market_value_base"].as_i64().unwrap(),
        p["net_redemption_base"].as_i64(),
    )
}

#[test]
fn asset_sources_sum_the_net_redemption_value() {
    let (_d, api) = setup();
    let bank = account(&api, "Banco", "brokerage", 0);
    let a = cdb(&api, &bank, "CDB A", 3000);
    let b = cdb(&api, &bank, "CDB B", 2000);
    let g = goal(&api, "Reserva", "emergency");
    link(&api, &g, "asset", &a);
    let v = link(&api, &g, "asset", &b);
    let (ma, na) = position(&api, &a);
    let (mb, nb) = position(&api, &b);
    let (na, nb) = (na.unwrap(), nb.unwrap());
    // The net value already discounts the IR on the gain.
    assert!(na < ma && nb < mb, "{na} {ma} {nb} {mb}");
    assert_eq!(v["current_amount"], na + nb, "{}", v["breakdown"]);
    let breakdown = v["breakdown"].as_array().unwrap();
    assert_eq!(breakdown.len(), 2);
    assert!(
        breakdown
            .iter()
            .all(|s| s["kind"] == "asset" && s["value_used"] == "net_redemption")
    );
    assert_eq!(v["sources"].as_array().unwrap().len(), 2);
    // Linking to market again changes that source's value.
    api.call(
        "goals.link_source",
        json!({"goal_id": g, "kind": "asset", "ref_id": a, "value_mode": "market"}),
    )
    .unwrap();
    assert_eq!(get(&api, &g)["current_amount"], ma + nb);
    // Turning off removes the source.
    api.call(
        "goals.unlink_source",
        json!({"goal_id": g, "kind": "asset", "ref_id": a}),
    )
    .unwrap();
    assert_eq!(get(&api, &g)["current_amount"], nb);
    assert!(
        api.call(
            "goals.unlink_source",
            json!({"goal_id": g, "kind": "asset", "ref_id": a}),
        )
        .is_err()
    );
}

#[test]
fn a_negative_account_does_not_reduce_the_goal() {
    let (_d, api) = setup();
    let overdrawn = account(&api, "Corrente no vermelho", "checking", -100_000);
    let broker = account(&api, "Corretora", "brokerage", 0);
    let s = stock(&api, &broker, "ABCD3", 100, "50");
    let g = goal(&api, "Reserva", "emergency");
    link(&api, &g, "account", &overdrawn);
    let v = link(&api, &g, "asset", &s);
    // R$ 5,000 of the investment; the −R$ 1,000 account stays out (it is debt, not reserve).
    assert_eq!(v["current_amount"], 500_000, "{}", v["breakdown"]);
    let acc = v["breakdown"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["kind"] == "account")
        .unwrap()
        .clone();
    assert_eq!(
        (acc["value"].as_i64(), acc["ignored_negative"].as_i64()),
        (Some(0), Some(-100_000))
    );
    assert_eq!(acc["value_used"], "balance");
    // Old goal with only one positive account: the same value as before.
    let saved = account(&api, "Poupança", "savings", 80_000);
    let old = id(&api
        .call(
            "goals.create",
            json!({"name": "Viagem", "kind": "travel", "target_amount": 1_000_000,
                   "account_ids": [saved]}),
        )
        .unwrap());
    assert_eq!(get(&api, &old)["current_amount"], 80_000);
    // Without a reserve goal, the reserve does not discount the negative account either.
    api.call("goals.delete", json!({"id": g})).unwrap();
    let er = api.call("goals.emergency_reserve", json!({})).unwrap();
    assert_eq!(er["current_amount"], 80_000, "{er}");
}

#[test]
fn a_label_source_takes_new_positions_without_editing_the_goal() {
    let (_d, api) = setup();
    let broker = account(&api, "Corretora", "brokerage", 0);
    let a = stock(&api, &broker, "AAAA3", 10, "100");
    let b = stock(&api, &broker, "BBBB3", 20, "50");
    api.call(
        "assets.labels_set",
        json!({"asset_id": a, "labels": ["reserva"]}),
    )
    .unwrap();
    let g = goal(&api, "Reserva", "emergency");
    let v = link(&api, &g, "asset_tag", "Reserva");
    assert_eq!(v["current_amount"], 100_000, "{}", v["breakdown"]);
    let tag = &v["breakdown"][0];
    assert_eq!(
        (tag["kind"].as_str(), tag["name"].as_str()),
        (Some("asset_tag"), Some("reserva"))
    );
    // The other asset gets the label: it comes in on its own.
    api.call(
        "assets.labels_set",
        json!({"asset_id": b, "labels": ["reserva"]}),
    )
    .unwrap();
    assert_eq!(get(&api, &g)["current_amount"], 200_000);
    // A label that does not exist is refused.
    assert!(
        api.call(
            "goals.link_source",
            json!({"goal_id": g, "kind": "asset_tag", "ref_id": "não existe"}),
        )
        .is_err()
    );
}

#[test]
fn a_stale_account_source_is_listed_in_stale_sources() {
    let (_d, api) = setup();
    let acc = account(&api, "Corrente parada", "checking", 200_000);
    let g = goal(&api, "Reserva", "emergency");
    let v = link(&api, &g, "account", &acc);
    // Open for 400 days and with no transactions: possibly incomplete.
    let stale = v["stale_sources"].as_array().unwrap();
    assert_eq!(stale.len(), 1, "{v}");
    assert_eq!(stale[0]["ref_id"], acc.as_str());
}

#[test]
fn the_reserve_and_financial_health_follow_an_investment_source() {
    let (_d, api) = setup();
    let broker = account(&api, "Corretora", "brokerage", 0);
    let s = stock(&api, &broker, "ABCD3", 100, "50");
    let g = goal(&api, "Reserva", "emergency");
    let before = api.call("goals.emergency_reserve", json!({})).unwrap();
    assert_eq!(before["current_amount"], 0);
    link(&api, &g, "asset", &s);
    let after = api.call("goals.emergency_reserve", json!({})).unwrap();
    assert_eq!(after["current_amount"], 500_000);
    assert_eq!(after["goal_id"], g.as_str());
}

fn agent() -> Actor {
    Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    }
}

fn mcp(api: &Api, name: &str, args: Value) -> Value {
    let o = inproc::call_tool(api, &agent(), name, args, inproc::AGENT_ACCESS);
    assert!(!o.is_error, "{name}: {}", o.content);
    let (_, data) = o.content.split_once('\n').unwrap();
    serde_json::from_str(data).unwrap()
}

#[test]
fn linking_through_mcp_is_a_proposal_and_list_goals_shows_the_breakdown() {
    let (_d, api) = setup();
    let broker = account(&api, "Corretora", "brokerage", 0);
    let s = stock(&api, &broker, "ABCD3", 100, "50");
    let g = goal(&api, "Reserva", "emergency");
    let p = mcp(
        &api,
        "link_goal_source",
        json!({"goal_id": g, "kind": "asset", "ref_id": s}),
    );
    assert_eq!(p["status"], "pending_approval", "{p}");
    // Nothing changes before approval.
    assert_eq!(get(&api, &g)["current_amount"], 0);
    api.call("ai.approve", json!({"ids": [p["proposal_id"]]}))
        .unwrap();
    assert_eq!(get(&api, &g)["current_amount"], 500_000);
    let list = mcp(&api, "list_goals", json!({}));
    let mine = list["goals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == g.as_str())
        .unwrap()
        .clone();
    assert_eq!(mine["breakdown"][0]["kind"], "asset");
    assert_eq!(mine["breakdown"][0]["value_used"], "market");
    // Turning off also becomes a proposal.
    let p = mcp(
        &api,
        "unlink_goal_source",
        json!({"goal_id": g, "kind": "asset", "ref_id": s}),
    );
    assert_eq!(p["status"], "pending_approval", "{p}");
    assert_eq!(get(&api, &g)["current_amount"], 500_000);
}
