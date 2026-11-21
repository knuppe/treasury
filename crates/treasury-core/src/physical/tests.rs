use chrono::NaiveDate;
use rust_decimal_macros::dec;
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

fn months_ago(n: i32) -> String {
    fmt_date(add_months(today_local(), -n))
}

fn i(v: &Value, ptr: &str) -> i64 {
    v.pointer(ptr)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("{ptr} in {v}"))
}

/// Index of 1% per month from 36 months ago to next month.
fn seed_index(api: &Api, series: &str) {
    let first = add_months(today_local(), -36);
    let vals: Vec<(NaiveDate, rust_decimal::Decimal, bool)> = (0..38)
        .map(|k| {
            let d = add_months(first, k);
            (
                NaiveDate::from_ymd_opt(chrono::Datelike::year(&d), chrono::Datelike::month(&d), 1)
                    .unwrap(),
                dec!(1),
                false,
            )
        })
        .collect();
    api.with_session(|s, _| crate::indices::upsert_values(s.db.conn(), series, &vals, "teste"))
        .unwrap();
}

#[test]
fn apartment_with_costs_improvements_appraisal_loan_and_net_worth() {
    let (_d, api) = api();
    let loan = api
        .call(
            "debt.create",
            json!({"name": "Financiamento do apê", "kind": "mortgage", "principal": 40_000_000, "annual_rate": "10",
                   "system": "sac", "contract_date": months_ago(24), "first_due_date": months_ago(23), "term_months": 360}),
        )
        .unwrap();
    let loan_id = loan["id"].as_str().unwrap().to_string();
    let a = api
        .call(
            "physical.create",
            json!({"name": "Apartamento Pinheiros", "kind": "real_estate", "purchase_date": months_ago(24),
                   "purchase_price": 60_000_000, "valuation_method": "none",
                   "acquisition_costs": [{"kind": "itbi", "amount": 1_800_000}, {"kind": "registry", "amount": 400_000},
                                         {"kind": "deed", "amount": 0}],
                   "address": "Rua dos Pinheiros, 100", "loan_ids": [loan_id]}),
        )
        .unwrap();
    let id = a["id"].as_str().unwrap().to_string();
    assert_eq!(
        a["acquisition_costs"].as_array().unwrap().len(),
        2,
        "zero cost does not count"
    );
    assert_eq!(i(&a, "/figures/initial_cost"), 62_200_000);
    assert_eq!(a["figures"]["value_basis"], "cost");
    assert_eq!(a["figures"]["estimated"], true);
    assert_eq!(a["loans"][0]["loan_id"], loan_id.as_str());
    let balance = i(&a, "/loans/0/balance");
    assert!(balance > 0 && balance < 40_000_000);
    assert_eq!(i(&a, "/figures/equity"), 62_200_000 - balance);

    // A renovation enters the cost; maintenance does not.
    api.call(
        "physical.add_improvement",
        json!({"asset_id": id, "date": months_ago(12), "description": "Reforma da cozinha", "category": "renovation", "amount": 5_000_000}),
    )
    .unwrap();
    let v = api
        .call(
            "physical.add_improvement",
            json!({"asset_id": id, "date": months_ago(6), "description": "Pintura", "category": "maintenance", "amount": 300_000}),
        )
        .unwrap();
    assert_eq!(i(&v, "/figures/cost_basis"), 67_200_000);
    assert_eq!(i(&v, "/figures/maintenance"), 300_000);
    assert_eq!(v["improvements"][1]["capital"], false);

    // Appraisal becomes the value (not estimated).
    let v = api
        .call(
            "physical.add_valuation",
            json!({"asset_id": id, "date": months_ago(1), "value": 80_000_000, "source": "report", "notes": "Laudo da Caixa"}),
        )
        .unwrap();
    let f = &v["figures"];
    assert_eq!(i(f, "/value"), 80_000_000);
    assert_eq!(f["value_basis"], "appraisal");
    assert_eq!(f["estimated"], false);
    assert_eq!(i(f, "/nominal_gain"), 80_000_000 - 67_200_000);
    assert!(f["annualized"].as_f64().unwrap() > 0.05);
    let ltv = f["ltv"].as_f64().unwrap();
    assert!((ltv - balance as f64 / 80_000_000.0).abs() < 0.02, "{ltv}");

    // Net worth: the asset enters at gross value and the financing only once, under debts.
    let nw = api.call("reports.net_worth", json!({})).unwrap();
    assert_eq!(i(&nw, "/physical"), 80_000_000);
    assert_eq!(i(&nw, "/assets"), 80_000_000);
    assert_eq!(i(&nw, "/debts"), i(&nw, "/liabilities"));
    assert_eq!(i(&nw, "/net"), 80_000_000 - i(&nw, "/debts"));
    let ex = api
        .call(
            "reports.explain",
            json!({"metric": "net_worth", "params": {}}),
        )
        .unwrap();
    let sources = ex["sources"].as_array().unwrap();
    let phys: Vec<&Value> = sources.iter().filter(|s| s["kind"] == "physical").collect();
    assert_eq!(phys.len(), 1);
    assert_eq!(phys[0]["id"], id.as_str());
    assert!(
        phys[0]["label"].as_str().unwrap().contains("laudo"),
        "{}",
        phys[0]["label"]
    );
    let sum: i64 = sources.iter().map(|s| s["amount"].as_i64().unwrap()).sum();
    assert_eq!(sum, i(&ex, "/value"));

    // Outside net worth: gone from the total, but the financing stays under debts.
    api.call(
        "physical.update",
        json!({"id": id, "include_in_net_worth": false}),
    )
    .unwrap();
    let nw2 = api.call("reports.net_worth", json!({})).unwrap();
    assert_eq!(i(&nw2, "/physical"), 0);
    assert_eq!(i(&nw2, "/debts"), i(&nw, "/debts"));

    // Undo puts the asset back into net worth.
    let ch = api.call("changes.list", json!({"limit": 1})).unwrap();
    api.call("changes.undo", json!({"id": ch[0]["id"]}))
        .unwrap();
    assert_eq!(
        i(
            &api.call("reports.net_worth", json!({})).unwrap(),
            "/physical"
        ),
        80_000_000
    );

    // Month-by-month history: cost rises with the renovation; the appraisal shows in its month.
    let h = api.call("physical.history", json!({"id": id})).unwrap();
    let h = h.as_array().unwrap();
    assert_eq!(h.len(), 25);
    assert_eq!(i(&h[0], "/cost_basis"), 62_200_000);
    assert_eq!(i(h.last().unwrap(), "/cost_basis"), 67_200_000);
    assert!(
        h.iter()
            .any(|p| p["appraisal"].as_i64() == Some(80_000_000))
    );
    assert!(i(&h[0], "/loan_balance") > i(h.last().unwrap(), "/loan_balance"));

    // Summary adds up value, cost and outstanding balance.
    let s = api.call("physical.summary", json!({})).unwrap();
    assert_eq!(i(&s, "/total_value"), 80_000_000);
    assert_eq!(
        i(&s, "/total_equity"),
        80_000_000 - i(&s, "/total_loan_balance")
    );

    // Deleting the contract undoes the link (no foreign key error).
    api.call("debt.delete", json!({"id": loan_id})).unwrap();
    let v = api.call("physical.get", json!({"id": id})).unwrap();
    assert!(v["loans"].as_array().unwrap().is_empty());
    assert_eq!(i(&v, "/figures/equity"), 80_000_000);
}

#[test]
fn index_correction_and_real_gain() {
    let (_d, api) = api();
    seed_index(&api, "igpm");
    seed_index(&api, "ipca");
    let a = api
        .call(
            "physical.create",
            json!({"name": "Terreno em Atibaia", "kind": "land", "purchase_date": months_ago(12),
                   "purchase_price": 20_000_000, "index_series": "igpm"}),
        )
        .unwrap();
    assert_eq!(a["valuation_method"], "index", "land: index by default");
    let f = &a["figures"];
    let v = i(f, "/value");
    // 12 months at 1%.
    assert!((v - 22_536_500).abs() < 1_000, "{v}");
    assert_eq!(f["value_basis"], "automatic");
    assert_eq!(f["estimated"], true);
    assert!(f["formula"].as_str().unwrap().contains("IGP-M"));
    // Same correction as the deflator (IPCA also 1% per month): real gain ≈ 0.
    assert!(i(f, "/real_gain").abs() < 50_000, "{f}");
    assert!(f["real_annualized"].as_f64().unwrap().abs() < 0.01);
}

#[test]
fn vehicle_depreciation_sale_and_validation() {
    let (_d, api) = api();
    let car = api
        .call(
            "physical.create",
            json!({"name": "Onix 2022", "kind": "vehicle", "purchase_date": months_ago(24), "purchase_price": 8_000_000}),
        )
        .unwrap();
    let id = car["id"].as_str().unwrap();
    assert_eq!(car["valuation_method"], "declining");
    assert_eq!(car["depreciation_rate"], "12");
    let v = i(&car, "/figures/value");
    assert!(
        (v - 6_195_000).abs() < 40_000,
        "0.88² × 80 thousand ≈ 61,950: {v}"
    );
    assert!(i(&car, "/figures/nominal_gain") < 0);

    // Sale: leaves net worth from the date and the gain becomes realized.
    let sold = api
        .call(
            "physical.sell",
            json!({"id": id, "sale_date": months_ago(1), "sale_price": 6_500_000, "sale_costs": 100_000}),
        )
        .unwrap();
    assert_eq!(sold["figures"]["sold"], true);
    assert_eq!(sold["archived"], true);
    assert_eq!(i(&sold, "/figures/sale_net"), 6_400_000);
    assert_eq!(i(&sold, "/figures/nominal_gain"), 6_400_000 - 8_000_000);
    assert_eq!(
        i(
            &api.call("reports.net_worth", json!({})).unwrap(),
            "/physical"
        ),
        0
    );
    let before = api
        .call("reports.net_worth", json!({"as_of": months_ago(2)}))
        .unwrap();
    assert!(
        i(&before, "/physical") > 0,
        "before the sale it still counted"
    );
    assert!(
        api.call("physical.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        api.call("physical.list", json!({"include_archived": true}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // Validations.
    for bad in [
        json!({"name": " ", "kind": "other", "purchase_date": "2026-01-01", "purchase_price": 1}),
        json!({"name": "X", "kind": "other", "purchase_date": "2026-13-01", "purchase_price": 1}),
        json!({"name": "X", "kind": "vehicle", "purchase_date": "2026-01-01", "purchase_price": 1, "valuation_method": "fipe"}),
        json!({"name": "X", "kind": "equipment", "purchase_date": "2026-01-01", "purchase_price": 1, "useful_life_years": 0}),
        json!({"name": "X", "kind": "vehicle", "purchase_date": "2026-01-01", "purchase_price": 1, "depreciation_rate": "abc"}),
    ] {
        assert!(api.call("physical.create", bad.clone()).is_err(), "{bad}");
    }
    assert!(
        api.call(
            "physical.sell",
            json!({"id": id, "sale_date": months_ago(30), "sale_price": 1})
        )
        .is_err(),
        "sale before purchase"
    );
    assert!(
        api.call(
            "physical.add_improvement",
            json!({"asset_id": id, "date": months_ago(1), "description": "X", "category": "renovation", "amount": 0})
        )
        .is_err()
    );
}

#[test]
fn loan_links_one_asset_and_improvement_edits() {
    let (_d, api) = api();
    let loan = api
        .call(
            "debt.create",
            json!({"name": "CDC", "kind": "vehicle", "principal": 5_000_000, "annual_rate": "20", "system": "price",
                   "contract_date": months_ago(3), "first_due_date": months_ago(2), "term_months": 48}),
        )
        .unwrap();
    let loan_id = loan["id"].as_str().unwrap();
    let mk = |name: &str| {
        api.call(
            "physical.create",
            json!({"name": name, "kind": "equipment", "purchase_date": months_ago(3), "purchase_price": 1_000_000}),
        )
        .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (a, b) = (mk("A"), mk("B"));
    api.call(
        "physical.link_loan",
        json!({"asset_id": a, "loan_id": loan_id}),
    )
    .unwrap();
    // Linking again to the same asset does not duplicate; to another asset it is a conflict.
    api.call(
        "physical.link_loan",
        json!({"asset_id": a, "loan_id": loan_id}),
    )
    .unwrap();
    let err = api
        .call(
            "physical.link_loan",
            json!({"asset_id": b, "loan_id": loan_id}),
        )
        .unwrap_err();
    assert!(err.to_string().contains("outro bem"), "{err}");
    let v = api
        .call(
            "physical.unlink_loan",
            json!({"asset_id": a, "loan_id": loan_id}),
        )
        .unwrap();
    assert!(v["loans"].as_array().unwrap().is_empty());

    let v = api
        .call(
            "physical.add_improvement",
            json!({"asset_id": a, "date": months_ago(1), "description": "Upgrade", "category": "improvement", "amount": 200_000}),
        )
        .unwrap();
    let imp = v["improvements"][0]["id"].as_str().unwrap().to_string();
    let v = api
        .call(
            "physical.update_improvement",
            json!({"id": imp, "category": "maintenance", "notes": "na verdade, conserto"}),
        )
        .unwrap();
    assert_eq!(i(&v, "/figures/cost_basis"), 1_000_000);
    assert_eq!(v["improvements"][0]["notes"], "na verdade, conserto");
    let v = api
        .call("physical.delete_improvement", json!({"id": imp}))
        .unwrap();
    assert!(v["improvements"].as_array().unwrap().is_empty());
    // Straight-line: equipment uses 5 years by default.
    assert_eq!(v["valuation_method"], "straight_line");
    assert_eq!(v["useful_life_years"], 5);
    api.call("physical.delete", json!({"id": b})).unwrap();
    assert!(api.call("physical.get", json!({"id": b})).is_err());
}

#[test]
fn fipe_quote_is_recorded_as_valuation() {
    let (_d, api) = api();
    let car = api
        .call(
            "physical.create",
            json!({"name": "Amarok", "kind": "vehicle", "purchase_date": months_ago(36), "purchase_price": 15_000_000,
                   "fipe_vehicle_type": "carros", "fipe_brand_code": "59", "fipe_model_code": "5940", "fipe_year_code": "2014-3"}),
        )
        .unwrap();
    assert_eq!(car["valuation_method"], "fipe");
    let id = car["id"].as_str().unwrap().to_string();
    let q = super::fipe::parse_quote(
        r#"{"Valor":"R$ 84.753,00","Marca":"VW - VolksWagen","Modelo":"AMAROK","AnoModelo":2014,"Combustivel":"Diesel","CodigoFipe":"005340-6","MesReferencia":"outubro de 2026"}"#,
    )
    .unwrap();
    let record = |q: &super::fipe::FipeQuote| {
        api.with_session(|s, _| {
            crate::changes::record(
                s.db.conn_mut(),
                &crate::changes::Author::user(),
                "Cotação FIPE",
                |tx| super::fipe::apply_quote(tx, &id, q),
            )
            .map(|_| ())
        })
    };
    record(&q).unwrap();
    // Again on the same day: replaces it.
    record(&q).unwrap();
    let v = api.call("physical.get", json!({"id": id})).unwrap();
    assert_eq!(v["valuations"].as_array().unwrap().len(), 1);
    assert_eq!(v["fipe_code"], "005340-6");
    assert_eq!(i(&v, "/figures/value"), 8_475_300);
    assert_eq!(v["figures"]["value_basis"], "appraisal");
    assert!(v["figures"]["source"].as_str().unwrap().contains("FIPE"));
    // A code different from the registered one is rejected.
    let mut other = q.clone();
    other.fipe_code = "999999-9".into();
    assert!(record(&other).is_err());
}
