//! IPTU through the API: the carnê with a single payment and installments, the derived
//! numbers, payment, reminders and IPTU in taxes paid. Dates relative to today.

use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;
use crate::time::{add_months, fmt_date, today_local};

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
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

fn day(n: i64) -> String {
    fmt_date(today_local() + Duration::days(n))
}

fn house(api: &Api) -> String {
    id(&api
        .call(
            "physical.create",
            json!({"name": "Casa", "kind": "real_estate", "purchase_date": "2020-01-10",
                   "purchase_price": 50_000_000, "valuation_method": "none"}),
        )
        .unwrap())
}

/// The year's carnê: assessed value R$ 300k, taxes R$ 3,000, street lighting R$ 100, good-payer
/// discount R$ 150; single payment of R$ 2,800 in `single_in` days or 10 installments of
/// R$ 295 starting on the same date.
fn bill(api: &Api, asset: &str, year: i32, single_in: i64) -> Value {
    let first = today_local() + Duration::days(single_in);
    let mut inst = vec![json!({"number": 0, "due_date": fmt_date(first), "amount": 280_000})];
    for k in 0..10 {
        inst.push(
            json!({"number": k + 1, "due_date": fmt_date(add_months(first, k)), "amount": 29_500}),
        );
    }
    api.call(
        "physical.iptu_save",
        json!({"asset_id": asset, "year": year, "registration": "123.456.789-0", "city": "Curitiba",
               "land_area": "300", "built_area": "120,5",
               "land_value": 12_000_000, "building_value": 18_000_000,
               "land_tax": 100_000, "building_tax": 200_000, "lighting_fee": 10_000,
               "good_payer_discount": 15_000, "total_due": 295_000, "installments": inst}),
    )
    .unwrap()
}

fn txn(api: &Api, date: &str, amount: i64, desc: &str) -> String {
    let acc = api
        .call(
            "accounts.create",
            json!({"name": format!("Conta {desc}"), "kind": "checking", "opening_date": "2020-01-01"}),
        )
        .unwrap();
    id(&api
        .call(
            "transactions.create",
            json!({"account_id": id(&acc), "date": date, "amount": amount, "kind": "expense",
                   "description": desc}),
        )
        .unwrap())
}

#[test]
fn bill_with_single_payment_and_installments() {
    let (_d, api) = api();
    let h = house(&api);
    let year = today_local().year();
    let v = bill(&api, &h, year, 20);
    assert_eq!(v["venal_total"], 30_000_000);
    assert_eq!(v["taxes_total"], 300_000);
    assert_eq!(v["built_area"], "120.5");
    assert!((v["effective_rate"].as_f64().unwrap() - 0.01).abs() < 1e-9);
    assert!(v["plan"].is_null());
    assert_eq!(v["installments"].as_array().unwrap().len(), 11);
    // No payment plan chosen: the lower of the two is missing (the single payment).
    assert_eq!(v["open"], 280_000);
    let s = &v["single_vs_installments"];
    assert_eq!(s["saving"], 15_000);
    assert_eq!(s["count"], 10);
    assert!(s["monthly_rate"].as_f64().unwrap() > 0.0, "{s}");
    assert_eq!(s["best"], "single", "sem CDI, a cota única mais barata");
    let kinds: Vec<&str> = v["insights"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["pay_upfront"], "{v}");
    // Assessed value R$ 300k × cost R$ 500k.
    assert!(
        (v["market"]["ratio"].as_f64().unwrap() - 0.6).abs() < 1e-9,
        "{v}"
    );

    // Saving the same year again replaces it (no duplicate).
    let again = bill(&api, &h, year, 20);
    assert_eq!(again["id"], v["id"]);
    let list = api
        .call("physical.iptu_list", json!({"asset_id": h}))
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Total that does not match the carnê: warning.
    let w = api
        .call(
            "physical.iptu_save",
            json!({"asset_id": h, "year": year, "taxes_total": 300_000, "total_due": 400_000}),
        )
        .unwrap();
    assert!(
        w["insights"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == "total_mismatch"),
        "{w}"
    );
    // A vehicle has no IPTU.
    let car = id(&api
        .call(
            "physical.create",
            json!({"name": "Carro", "kind": "vehicle", "purchase_date": "2022-01-01",
                   "purchase_price": 8_000_000, "valuation_method": "none"}),
        )
        .unwrap());
    assert!(
        api.call(
            "physical.iptu_save",
            json!({"asset_id": car, "year": year, "total_due": 1}),
        )
        .is_err()
    );
}

#[test]
fn paying_an_installment_chooses_the_plan() {
    let (_d, api) = api();
    let h = house(&api);
    let v = bill(&api, &h, today_local().year(), -3);
    let first = v["installments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["number"] == 1)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // The payment transaction shows up as a suggestion (mentions IPTU and has the amount).
    let paid = txn(&api, &day(-4), -29_500, "PAGTO IPTU PARC 1");
    let v = api
        .call("physical.iptu_list", json!({"asset_id": h}))
        .unwrap()[0]
        .clone();
    let p1 = v["installments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["number"] == 1)
        .unwrap();
    assert_eq!(p1["status"], "late");
    assert_eq!(p1["suggestion"]["txn_id"], paid.as_str(), "{p1}");
    assert_eq!(p1["suggestion"]["named"], true);

    let v = api
        .call(
            "physical.iptu_pay",
            json!({"installment_id": first, "txn_id": paid}),
        )
        .unwrap();
    assert_eq!(v["plan"], "installments");
    let by = |n: i64| {
        v["installments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["number"] == n)
            .unwrap()
            .clone()
    };
    assert_eq!(by(1)["status"], "paid");
    assert_eq!(by(1)["paid_on"], day(-4));
    assert_eq!(by(0)["status"], "not_chosen");
    assert_eq!(v["paid"], 29_500);
    assert_eq!(v["open"], 9 * 29_500);
    // The same transaction does not pay two installments.
    let second = by(2)["id"].as_str().unwrap().to_string();
    assert!(
        api.call(
            "physical.iptu_pay",
            json!({"installment_id": second, "txn_id": paid}),
        )
        .is_err()
    );
    // Paid without a transaction, by date; and undo.
    api.call(
        "physical.iptu_pay",
        json!({"installment_id": second, "paid_on": day(0)}),
    )
    .unwrap();
    let v = api
        .call("physical.iptu_unpay", json!({"installment_id": second}))
        .unwrap();
    assert_eq!(v["paid"], 29_500);
}

fn alerts(api: &Api) -> Vec<Value> {
    api.with_session(|s, _| {
        crate::notify::run(s.db.conn(), today_local())?;
        Ok(())
    })
    .unwrap();
    api.call("ai.alerts.list", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["kind"].as_str().unwrap_or("").starts_with("iptu"))
        .cloned()
        .collect()
}

#[test]
fn reminders_once_per_installment_and_for_the_missing_bill() {
    let (_d, api) = api();
    let h = house(&api);
    let today = today_local();
    bill(&api, &h, today.year(), 2);
    let a = alerts(&api);
    // No plan chosen: only the first date, reminding to decide.
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(
        a[0]["title"].as_str().unwrap().contains("cota única"),
        "{}",
        a[0]
    );
    assert!(
        a[0]["body"]
            .as_str()
            .unwrap()
            .contains("à vista ou parcelado")
    );
    // Again: nothing new.
    assert_eq!(alerts(&api).len(), 1);

    // Last year's carnê and none for this one: the reminder to register appears once.
    let other = house(&api);
    let last_year_first_due =
        NaiveDate::from_ymd_opt(today.year() - 1, today.month(), today.day().min(28)).unwrap();
    api.call(
        "physical.iptu_save",
        json!({"asset_id": other, "year": today.year() - 1, "total_due": 100_000,
               "installments": [{"number": 0, "due_date": fmt_date(last_year_first_due), "amount": 100_000}]}),
    )
    .unwrap();
    let a = alerts(&api);
    assert!(a.iter().any(|x| x["kind"] == "iptu_missing"), "{a:?}");
    assert_eq!(
        alerts(&api)
            .iter()
            .filter(|x| x["kind"] == "iptu_missing")
            .count(),
        1
    );
}

#[test]
fn a_named_payment_silences_the_reminder() {
    let (_d, api) = api();
    let h = house(&api);
    bill(&api, &h, today_local().year(), 1);
    txn(&api, &day(0), -280_000, "IPTU COTA UNICA");
    assert!(alerts(&api).is_empty());
}

#[test]
fn iptu_from_bills_counts_in_taxes_paid() {
    let (_d, api) = api();
    let h = house(&api);
    let last = today_local().year() - 1;
    let first = NaiveDate::from_ymd_opt(last, 2, 10).unwrap();
    let mut inst = vec![json!({"number": 0, "due_date": fmt_date(first), "amount": 280_000})];
    for k in 0..10 {
        inst.push(
            json!({"number": k + 1, "due_date": fmt_date(add_months(first, k)), "amount": 29_500}),
        );
    }
    let v = api
        .call(
            "physical.iptu_save",
            json!({"asset_id": h, "year": last, "land_value": 30_000_000, "taxes_total": 300_000,
                   "total_due": 295_000, "plan": "installments", "installments": inst}),
        )
        .unwrap();
    assert_eq!(v["plan"], "installments");
    let e = api
        .call("tax_burden.estimate", json!({"year": last}))
        .unwrap();
    let lines: Vec<&Value> = e["property"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["tax"] == "iptu")
        .collect();
    assert_eq!(lines.len(), 1, "{e}");
    assert_eq!(lines[0]["amount"], 10 * 29_500);
    assert_eq!(lines[0]["value"], 30_000_000);
}

#[test]
fn tax_above_inflation_against_last_year() {
    let (_d, api) = api();
    let h = house(&api);
    let year = today_local().year();
    let vals: Vec<(NaiveDate, Decimal, bool)> = (1..=12)
        .map(|m| {
            (
                NaiveDate::from_ymd_opt(year - 1, m, 1).unwrap(),
                Decimal::new(5, 1),
                false,
            )
        })
        .collect();
    api.with_session(|s, _| crate::indices::upsert_values(s.db.conn(), "ipca", &vals, "teste"))
        .unwrap();
    api.call(
        "physical.iptu_save",
        json!({"asset_id": h, "year": year - 1, "land_value": 25_000_000, "taxes_total": 250_000,
               "total_due": 250_000}),
    )
    .unwrap();
    let v = api
        .call(
            "physical.iptu_save",
            json!({"asset_id": h, "year": year, "land_value": 30_000_000, "taxes_total": 300_000,
                   "total_due": 300_000}),
        )
        .unwrap();
    let p = &v["previous"];
    assert_eq!(p["year"], year - 1);
    assert!((p["tax_change"].as_f64().unwrap() - 0.2).abs() < 1e-9);
    assert!((p["ipca"].as_f64().unwrap() - (1.005f64.powi(12) - 1.0)).abs() < 1e-9);
    assert!(
        v["insights"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == "above_inflation"),
        "{v}"
    );
}

/// Monthly installments of the same amount: the windows overlap, and a payment can only be
/// suggested for one of them (the one with the nearest due date); the other stays reminded.
#[test]
fn one_payment_is_suggested_for_one_installment_only() {
    let (_d, api) = api();
    let h = house(&api);
    let first = today_local() - Duration::days(20);
    let v = api
        .call(
            "physical.iptu_save",
            json!({"asset_id": h, "year": today_local().year(), "total_due": 59_000, "plan": "installments",
                   "installments": [
                       {"number": 1, "due_date": fmt_date(first), "amount": 29_500},
                       {"number": 2, "due_date": fmt_date(first + Duration::days(30)), "amount": 29_500}
                   ]}),
        )
        .unwrap();
    assert_eq!(v["installments"].as_array().unwrap().len(), 2);
    // Paid 5 days after the 1st due date: inside both installments' windows.
    let paid = txn(
        &api,
        &fmt_date(first + Duration::days(5)),
        -29_500,
        "IPTU PARCELA",
    );
    let v = api
        .call("physical.iptu_list", json!({"asset_id": h}))
        .unwrap()[0]
        .clone();
    let by = |n: i64| {
        v["installments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["number"] == n)
            .unwrap()
            .clone()
    };
    assert_eq!(by(1)["suggestion"]["txn_id"], paid.as_str(), "{v}");
    assert!(by(2)["suggestion"].is_null(), "{v}");
}
