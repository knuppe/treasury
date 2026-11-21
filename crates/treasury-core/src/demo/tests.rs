use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

const PASSWORD: &str = "senha-mestra-forte";

fn setup() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
    api.call(
        "vault.create",
        json!({"password": PASSWORD, "household_name": "Casa de verdade"}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta real", "kind": "checking"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2026-10-01", "amount": -1000, "description": "real"}),
    )
    .unwrap();
    (dir, api)
}

fn profile() -> Value {
    json!({
        "months": 12,
        "adults": [
            {"name": "Ana", "income": 900_000},
            {"name": "Bruno", "income": 600_000, "variable": true}
        ],
        "children": ["Clara"],
        "bank": "Itaú",
        "card_limit": 1_500_000,
        "other_income": 150_000,
        "housing": "rent",
        "housing_amount": 350_000,
        "condo": 80_000,
        "utilities": 60_000,
        "groceries": 180_000,
        "transport": "car",
        "transport_amount": 90_000,
        "school_per_child": 150_000,
        "health": 120_000,
        "leisure": 100_000,
        "subscriptions": 9_000,
        "shopping": 60_000,
        "reserve": 3_000_000,
        "reserve_monthly": 50_000,
        "stocks": 2_000_000,
        "fiis": 1_500_000,
        "fixed_income": 1_000_000,
        "invest_monthly": 100_000,
        "pension": 800_000,
        "pension_monthly": 50_000,
        "car_installment": 120_000,
        "car_installments_left": 24,
        "level": "intermediario",
        "seed": 7
    })
}

fn new_demo(api: &Api, name: &str, p: Value) -> Value {
    api.call(
        "vaults.new_demo",
        json!({"name": name, "household_name": "Família Demo", "password": PASSWORD, "profile": p}),
    )
    .unwrap()
}

#[test]
fn new_demo_is_a_separate_vault_with_a_full_history() {
    let (_d, api) = setup();
    let r = new_demo(&api, "Demonstração", profile());
    let st = &r["status"];
    assert_eq!(st["state"], "unlocked");
    assert_eq!(st["demo"], true);
    assert_eq!(st["vault_name"], "Demonstração");
    assert_eq!(st["vault_count"], 2);
    assert_eq!(st["household_name"], "Família Demo");
    let s = &r["summary"];
    assert!(s["transactions"].as_u64().unwrap() > 300, "{s}");
    assert_eq!(
        (s["accounts"].as_u64(), s["loans"].as_u64()),
        // Account, card, reserve, brokerage, pension and Ana's FGTS (CLT).
        (Some(6), Some(1))
    );
    assert_eq!(s["assets"], 9);

    // Members, accounts and main screens work with the generated data.
    let members = api.call("household.list_members", json!({})).unwrap();
    assert_eq!(members.as_array().unwrap().len(), 3);
    let accounts = api.call("accounts.list", json!({})).unwrap();
    assert_eq!(accounts.as_array().unwrap().len(), 6);
    for (cmd, args) in [
        ("reports.dashboard", json!({})),
        ("reports.net_worth", json!({})),
        ("portfolio.summary", json!({})),
        ("debt.list", json!({})),
        (
            "budget.get",
            json!({"month": &crate::time::fmt_date(crate::time::today_local())[..7]}),
        ),
        ("goals.list", json!({})),
        ("health.score", json!({})),
        ("recurrences.list", json!({})),
    ] {
        api.call(cmd, args).unwrap_or_else(|e| panic!("{cmd}: {e}"));
    }
    let nw = api.call("reports.net_worth", json!({})).unwrap();
    assert!(nw["investments"].as_i64().unwrap() > 0, "{nw}");
    let flow = api
        .call(
            "reports.month_flow",
            json!({"from_month": "2026-01", "to_month": "2026-09"}),
        )
        .unwrap();
    for m in flow.as_array().unwrap() {
        assert!(m["income"].as_i64().unwrap() > 0 && m["expense"].as_i64().unwrap() > 0);
    }
    // Overdue statements paid; the debt with the installments in the history paid.
    let unpaid: i64 = {
        let loans = api.call("debt.list", json!({})).unwrap();
        let l = &loans.as_array().unwrap()[0];
        assert_eq!(l["name"], "Financiamento do carro");
        assert_eq!(l["remaining_count"], 24);
        l["overdue_count"].as_i64().unwrap_or(0)
    };
    assert_eq!(unpaid, 0);
    // Receipts (cupons fiscais) for recent grocery purchases, with products and price per kg (06 ·
    // Consumption), summing the value of each transaction.
    let receipts = api
        .call("receipts.list", json!({"from": "2000-01-01"}))
        .unwrap();
    let rs = receipts.as_array().unwrap();
    assert!(!rs.is_empty(), "{receipts}");
    assert!(rs.iter().all(|r| r["txn_id"].is_string()));
    let products = api.call("food.products", json!({"food": true})).unwrap();
    assert!(products.as_array().unwrap().len() >= 5);
    let profile = api.call("profile.get", json!({})).unwrap();
    assert_eq!(profile["level"], "intermediario");
    assert_eq!(profile["onboarding_pending"], false);

    // The real vault stays as it was.
    let list = api.call("vaults.list", json!({})).unwrap();
    let principal = list["vaults"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Principal")
        .unwrap()["id"]
        .clone();
    api.call("vaults.select", json!({"id": principal})).unwrap();
    api.call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
    let st = api.call("vault.status", json!({})).unwrap();
    assert_eq!(
        (st["demo"].as_bool(), st["household_name"].as_str()),
        (Some(false), Some("Casa de verdade"))
    );
    let txns = api.call("transactions.query", json!({})).unwrap();
    assert_eq!(txns["total"], 1, "{txns}");
}

/// With the app in English, the demo vault is born with the categories in English and the assistant
/// finds each one by its pt-BR path (through the origin key); the ready-made rules too.
#[test]
fn demo_works_with_the_categories_in_another_language() {
    let (_d, api) = setup();
    crate::i18n::set_current(crate::i18n::Locale::En);
    let r = new_demo(&api, "Demo", profile());
    assert!(r["summary"]["transactions"].as_u64().unwrap() > 300, "{r}");
    let cats = api.call("categories.list", json!({})).unwrap();
    let names: Vec<&str> = cats
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Groceries") && !names.contains(&"Mercado"));
    let rules = api.call("rules.list", json!({})).unwrap();
    assert!(rules.as_array().unwrap().len() > 10, "{rules}");
    crate::i18n::set_current(crate::i18n::Locale::PtBr);
}

#[test]
fn same_input_same_history_and_mortgage_option() {
    let (_d, api) = setup();
    let a = new_demo(&api, "Demo A", profile());
    let b = new_demo(&api, "Demo B", profile());
    assert_eq!(a["summary"]["transactions"], b["summary"]["transactions"]);

    let mut p = profile();
    p["housing"] = json!("mortgage");
    p["housing_amount"] = json!(280_000);
    p["card_limit"] = Value::Null;
    p["car_installment"] = json!(0);
    p["adults"] = json!([{"name": "Ana", "income": 1_200_000}]);
    p["children"] = json!([]);
    p["months"] = json!(6);
    let c = new_demo(&api, "Demo C", p);
    assert_eq!(c["summary"]["loans"], 1);
    // Account, reserve, brokerage, pension and Ana's FGTS.
    assert_eq!(c["summary"]["accounts"], 5);
    let loans = api.call("debt.list", json!({})).unwrap();
    assert_eq!(loans[0]["name"], "Financiamento do apartamento");
}

#[test]
fn invalid_input_changes_nothing_and_generate_needs_an_empty_vault() {
    let (_d, api) = setup();
    for bad in [
        json!({"months": 2}),
        json!({"adults": []}),
        json!({"adults": [{"name": "Ana", "income": 0}]}),
        json!({"children": ["Ana"]}),
        json!({"housing": "mortgage", "housing_amount": 0}),
    ] {
        let mut p = profile();
        for (k, v) in bad.as_object().unwrap() {
            p[k] = v.clone();
        }
        let err = api
            .call(
                "vaults.new_demo",
                json!({"name": "X", "household_name": "F", "password": PASSWORD, "profile": p}),
            )
            .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Invalid(_)),
            "{bad}: {err}"
        );
    }
    // Nothing changed: the real vault is still open and alone in the list.
    let st = api.call("vault.status", json!({})).unwrap();
    assert_eq!(
        (st["vault_count"].as_u64(), st["state"].as_str()),
        (Some(1), Some("unlocked"))
    );
    // Generating in a vault with transactions is refused.
    assert!(api.call("demo.generate", profile()).is_err());
}

/// Without a password, the demo vault opens with the app's fixed key; vaults with a password
/// still ask for the password.
#[test]
fn demo_without_password_unlocks_without_asking() {
    let (_d, api) = setup();
    let r = api
        .call(
            "vaults.new_demo",
            json!({"name": "Sem senha", "household_name": "Família Demo", "profile": profile()}),
        )
        .unwrap();
    assert_eq!(r["status"]["demo"], true);
    assert_eq!(r["status"]["demo_no_password"], true);
    let list = api.call("vaults.list", json!({})).unwrap();
    let demo = list["vaults"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "Sem senha")
        .unwrap();
    assert_eq!(demo["demo"], true);
    api.call("vault.lock", Value::Null).unwrap();
    let st = api.call("vault.status", Value::Null).unwrap();
    assert_eq!(st["state"], "locked");
    assert_eq!(st["demo_no_password"], true);
    let st = api.call("vault.unlock_demo", Value::Null).unwrap();
    assert_eq!(st["state"], "unlocked");
    assert!(
        api.call("accounts.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len()
            > 1
    );

    // The copy also opens without a password.
    let id = demo["id"].as_str().unwrap().to_string();
    let copies = api.call("vaults.copy", json!({"id": id})).unwrap();
    assert!(
        copies["vaults"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["name"] == "Sem senha (cópia)" && v["demo"] == true),
        "{copies}"
    );

    // In a vault with a password, the fixed key does not open it.
    new_demo(&api, "Com senha", profile());
    api.call("vault.lock", Value::Null).unwrap();
    let st = api.call("vault.status", Value::Null).unwrap();
    assert_eq!(st["demo_no_password"], false);
    assert!(api.call("vault.unlock_demo", Value::Null).is_err());
}

fn descriptions(api: &Api) -> Vec<String> {
    let page = api
        .call("transactions.query", json!({"limit": 5000}))
        .unwrap();
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["description"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// Family in hardship: overdraft with interest, late bills with a fine, personal
/// loan, revolving card credit and Bolsa Família; payslips in fixed income.
#[test]
fn struggling_family_has_debt_overdraft_and_late_bills() {
    let (_d, api) = setup();
    let p = json!({
        "months": 12,
        "kind": "struggling",
        "adults": [{"name": "Rosa", "income": 210_000}],
        "children": ["Davi", "Elisa"],
        "bank": "Caixa",
        "card_limit": 150_000,
        "housing": "rent",
        "housing_amount": 90_000,
        "utilities": 30_000,
        "groceries": 120_000,
        "transport": "public",
        "transport_amount": 22_000,
        "leisure": 8_000,
        "shopping": 12_000,
        "personal_loan": 38_000,
        "personal_loan_left": 18,
        "benefit": 60_000,
        "overdraft_limit": 100_000,
        "late_bills": true,
        "card_revolving": true,
        "seed": 3
    });
    let r = api
        .call(
            "vaults.new_demo",
            json!({"name": "Dificuldade", "household_name": "Família Rosa", "profile": p}),
        )
        .unwrap();
    assert_eq!(r["summary"]["loans"], 1);
    let d = descriptions(&api);
    assert!(
        d.iter().any(|x| x == "Juros do cheque especial"),
        "no overdraft interest"
    );
    assert!(d.iter().any(|x| x.starts_with("Multa e juros por atraso")));
    assert!(d.iter().any(|x| x == "Juros do rotativo do cartão"));
    assert!(d.iter().any(|x| x == "Bolsa Família"));
    assert!(d.iter().any(|x| x == "Parcela empréstimo pessoal"));
    let list = api.call("vaults.list", json!({})).unwrap();
    assert!(
        list.to_string().contains("família em dificuldade"),
        "{list}"
    );
    let slips = api.call("income.list_payslips", json!({})).unwrap();
    assert_eq!(slips.as_array().unwrap().len(), 6, "{slips}");
    let s = api.call("income.summary", json!({"months": 12})).unwrap();
    assert!(s["actual"].as_array().unwrap().len() >= 6, "{s}");
    assert!(s["forecast_base_month"].is_string());
}

/// Retiree in the US, in English: dollar, no 13th salary, ETFs in place of Brazilian stocks and the
/// texts in English.
#[test]
fn retiree_in_the_us_in_english() {
    let (_d, api) = setup();
    crate::i18n::set_current(crate::i18n::Locale::En);
    let p = json!({
        "months": 12,
        "kind": "retiree",
        "country": "US",
        "adults": [{"name": "Walter", "income": 320_000, "retired": true}],
        "bank": "Chase",
        "card_limit": 500_000,
        "housing": "own",
        "utilities": 25_000,
        "groceries": 60_000,
        "transport": "car",
        "transport_amount": 25_000,
        "health": 45_000,
        "leisure": 30_000,
        "reserve": 2_500_000,
        "reserve_monthly": 10_000,
        "stocks": 15_000_000,
        "fiis": 2_000_000,
        "fixed_income": 6_000_000,
        "seed": 11
    });
    let r = api
        .call(
            "vaults.new_demo",
            json!({"name": "Retiree", "household_name": "Walter", "profile": p}),
        )
        .unwrap();
    crate::i18n::set_current(crate::i18n::Locale::PtBr);
    let r = &r["summary"];
    assert!(r["transactions"].as_u64().unwrap() > 100, "{r}");
    let info = api.call("household.get", json!({})).unwrap();
    assert_eq!(info["base_currency"], "USD", "{info}");
    let d = descriptions(&api);
    assert!(d.iter().any(|x| x == "Social Security retirement"), "{d:?}");
    assert!(!d.iter().any(|x| x.contains("13")));
    let assets = api.call("assets.list", json!({})).unwrap();
    let tickers = assets.to_string();
    assert!(
        tickers.contains("VTI") && !tickers.contains("PETR4"),
        "{tickers}"
    );
    let slips = api.call("income.list_payslips", json!({})).unwrap();
    assert!(!slips.as_array().unwrap().is_empty());
}

/// The five assistant profiles (features/demo/presets.ts) generate without error in Brazil, in
/// Portugal (in Spanish) and in the US.
#[test]
fn every_profile_generates_in_every_country() {
    let presets = [
        json!({"kind": "comfortable", "adults": [{"name": "Ana", "income": 1_200_000}, {"name": "Bruno", "income": 850_000}],
            "children": ["Clara"], "housing": "mortgage", "housing_amount": 420_000, "condo": 90_000, "utilities": 90_000,
            "groceries": 220_000, "transport": "car", "transport_amount": 130_000, "school_per_child": 240_000,
            "health": 180_000, "leisure": 150_000, "subscriptions": 15_000, "shopping": 90_000, "card_limit": 2_500_000,
            "reserve": 6_000_000, "reserve_monthly": 150_000, "stocks": 8_000_000, "fiis": 4_000_000,
            "fixed_income": 5_000_000, "invest_monthly": 250_000, "pension": 3_000_000, "pension_monthly": 80_000}),
        json!({"kind": "low_income", "adults": [{"name": "Maria", "income": 190_000}, {"name": "João", "income": 160_000, "variable": true}],
            "children": ["Davi", "Elisa"], "benefit": 60_000, "housing": "rent", "housing_amount": 90_000,
            "utilities": 42_000, "groceries": 145_000, "transport": "public", "transport_amount": 38_000,
            "leisure": 15_000, "subscriptions": 4_000, "shopping": 28_000, "card_limit": 120_000,
            "reserve": 150_000, "reserve_monthly": 3_000}),
        json!({"kind": "struggling", "adults": [{"name": "Rosa", "income": 230_000}, {"name": "Carlos", "income": 90_000, "variable": true}],
            "children": ["Davi", "Elisa"], "housing": "rent", "housing_amount": 110_000, "utilities": 38_000,
            "groceries": 130_000, "transport": "public", "transport_amount": 30_000, "leisure": 6_000,
            "subscriptions": 5_000, "shopping": 20_000, "card_limit": 180_000, "personal_loan": 45_000,
            "personal_loan_left": 18, "overdraft_limit": 150_000, "late_bills": true, "card_revolving": true}),
        json!({"kind": "single", "adults": [{"name": "Lucas", "income": 420_000}], "housing": "rent",
            "housing_amount": 150_000, "condo": 35_000, "utilities": 28_000, "groceries": 70_000, "transport": "app",
            "transport_amount": 45_000, "leisure": 70_000, "subscriptions": 9_000, "shopping": 40_000,
            "card_limit": 500_000, "reserve": 300_000, "reserve_monthly": 30_000, "fiis": 200_000,
            "fixed_income": 500_000, "invest_monthly": 30_000}),
        json!({"kind": "retiree", "adults": [{"name": "Helena", "income": 380_000, "retired": true}],
            "other_income": 150_000, "housing": "own", "condo": 60_000, "utilities": 42_000, "groceries": 110_000,
            "transport": "none", "health": 130_000, "leisure": 40_000, "subscriptions": 6_000, "shopping": 20_000,
            "card_limit": 400_000, "reserve": 4_000_000, "reserve_monthly": 20_000, "fixed_income": 8_000_000}),
    ];
    for (country, locale) in [
        ("BR", crate::i18n::Locale::PtBr),
        ("PT", crate::i18n::Locale::Es),
        ("US", crate::i18n::Locale::En),
    ] {
        let (_d, api) = setup();
        crate::i18n::set_current(locale);
        for (i, p) in presets.iter().enumerate() {
            let mut p = p.clone();
            p["months"] = json!(6);
            p["bank"] = json!("Banco");
            p["country"] = json!(country);
            let r = api.call(
                "vaults.new_demo",
                json!({"name": format!("{country} {i}"), "household_name": "Demo", "profile": p}),
            );
            let r = r.unwrap_or_else(|e| panic!("{country} {}: {e}", p["kind"]));
            assert!(r["summary"]["transactions"].as_u64().unwrap() > 20, "{r}");
            api.call("reports.dashboard", json!({}))
                .unwrap_or_else(|e| panic!("{country} {}: {e}", p["kind"]));
        }
        crate::i18n::set_current(crate::i18n::Locale::PtBr);
    }
}

/// The demo stays entirely in the country's currency: dollar in the US, euro in Portugal (accounts,
/// contracts, goals, assets and transactions), and the dashboard and net worth come out in that
/// currency.
#[test]
fn demo_uses_the_country_currency() {
    for (country, currency) in [("US", "USD"), ("PT", "EUR")] {
        let (_d, api) = setup();
        let mut p = profile();
        p["country"] = json!(country);
        p["personal_loan"] = json!(45_000);
        p["personal_loan_left"] = json!(18);
        new_demo(&api, country, p);
        let h = api.call("household.get", Value::Null).unwrap();
        assert_eq!(h["base_currency"], currency, "{country}");
        let found: Vec<(String, String)> = api
            .with_session(|s, _| {
                let mut stmt = s.db.conn().prepare(
                    "SELECT 'account', currency FROM account UNION SELECT 'loan', currency FROM loan
                     UNION SELECT 'goal', currency FROM goal UNION SELECT 'asset', currency FROM asset
                     UNION SELECT 'txn', currency FROM txn",
                )?;
                let rows = stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                Ok(rows)
            })
            .unwrap();
        for what in ["account", "loan", "goal", "asset", "txn"] {
            assert!(found.iter().any(|(w, _)| w == what), "{country}: {found:?}");
        }
        assert!(
            found.iter().all(|(_, c)| c == currency),
            "{country}: {found:?}"
        );
        let nw = api.call("reports.net_worth", json!({})).unwrap();
        assert_eq!(nw["base_currency"], currency, "{country}");
        let dash = api.call("reports.dashboard", json!({})).unwrap();
        assert_eq!(dash["currency"], currency, "{country}");
    }
}

/// Currency without cents (Chilean peso, yen): the profile's values arrive in the smallest unit,
/// which in those currencies is the unit itself, and the portfolio is born on the same scale as the
/// accounts.
#[test]
fn demo_portfolio_scale_in_currencies_without_cents() {
    for (country, currency) in [("CL", "CLP"), ("JP", "JPY")] {
        let (_d, api) = setup();
        let mut p = profile();
        p["country"] = json!(country);
        p["months"] = json!(6);
        p["stocks"] = json!(8_000_000);
        p["fiis"] = json!(4_000_000);
        p["fixed_income"] = json!(5_000_000);
        p["invest_monthly"] = json!(250_000);
        p["pension"] = json!(0);
        p["pension_monthly"] = json!(0);
        new_demo(&api, country, p);
        let nw = api.call("reports.net_worth", json!({})).unwrap();
        assert_eq!(nw["base_currency"], currency, "{country}");
        // What went into the portfolio: the three initial amounts and six monthly contributions.
        let invested: i64 = 8_000_000 + 4_000_000 + 5_000_000;
        let investments = nw["investments"].as_i64().unwrap();
        assert!(
            investments > invested * 8 / 10 && investments < invested * 2,
            "{country}: portfolio of {investments} for {invested} invested"
        );
    }
}

/// The first time (no vault) and after "New vault", the demo takes the vault that still has
/// no file, instead of leaving an empty vault in the list.
#[test]
fn demo_takes_the_vault_not_created_yet() {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
    let st = api.call("vault.status", Value::Null).unwrap();
    assert_eq!(st["state"], "no_vault");
    let r = api
        .call(
            "vaults.new_demo",
            json!({"name": "Demonstração", "household_name": "Família Demo", "profile": profile()}),
        )
        .unwrap();
    assert_eq!(r["status"]["state"], "unlocked");
    assert_eq!(r["status"]["vault_count"], 1);
    assert_eq!(r["status"]["demo_no_password"], true);
    assert_eq!(r["status"]["vault_name"], "Demonstração");

    // "New vault" without a password yet, and the demo takes that vault.
    let st = api.call("vaults.new", json!({"name": "Outro"})).unwrap();
    assert_eq!(st["state"], "no_vault");
    let r = api
        .call(
            "vaults.new_demo",
            json!({"name": "Demo 2", "household_name": "Família Demo", "profile": profile()}),
        )
        .unwrap();
    assert_eq!(r["status"]["vault_count"], 2);
    let list = api.call("vaults.list", json!({})).unwrap();
    let names: Vec<&str> = list["vaults"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Demonstração", "Demo 2"]);
}

/// Report from Oct 20, 2026: the demo vault opened with "how much can I spend today" at zero,
/// because the generator put big purchases on the day itself. Now only small ones stay. On the 1st
/// the generator does not move them to the previous month, and the test has nothing to check.
#[test]
fn the_demo_opens_with_today_free_of_big_purchases() {
    use chrono::Datelike;
    if crate::time::today_local().day() == 1 {
        return;
    }
    for seed in [3, 5, 8] {
        let (_d, api) = setup();
        let mut p = profile();
        p["seed"] = json!(seed);
        new_demo(&api, &format!("Demo {seed}"), p);
        let today = crate::time::fmt_date(crate::time::today_local());
        let big: i64 = api
            .with_session(|s, _| {
                Ok(s.db.conn().query_row(
                    "SELECT count(*) FROM txn WHERE date = ?1 AND amount < -3000 AND kind = 'expense'",
                    [&today],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(big, 0, "seed {seed}");
    }
}

/// The demo vault is born with the spending groups from the initial set: the car
/// installment falls under debts, the contributions and the reserve under future.
#[test]
fn demo_categories_come_with_spending_groups() {
    let (_d, api) = setup();
    new_demo(&api, "Demonstração", profile());
    let cats = api.call("categories.list", json!({})).unwrap();
    let cats = cats.as_array().unwrap();
    let group = |path: &str| {
        cats.iter()
            .find(|c| c["path"] == path)
            .unwrap_or_else(|| panic!("{path}"))["spend_group"]
            .clone()
    };
    assert_eq!(group("Pagamentos › Financiamentos"), "debt");
    assert_eq!(group("Poupança e investimentos"), "future");
    assert_eq!(group("Moradia › Aluguel"), "essential");
    assert_eq!(group("Lazer"), "non_essential");
    assert!(group("Salário").is_null());
    let today = crate::time::today_local();
    let month = |delta: i32| {
        let d = crate::time::add_months(today, delta);
        crate::time::fmt_month(chrono::Datelike::year(&d), chrono::Datelike::month(&d))
    };
    let rep = api
        .call(
            "reports.spend_groups",
            json!({"from": month(-3), "to": month(-1), "basis": "accrual"}),
        )
        .unwrap();
    let total = &rep["total"];
    assert!(total["income"].as_i64().unwrap() > 0, "{total}");
    for g in total["groups"].as_array().unwrap() {
        assert!(g["amount"].as_i64().unwrap() > 0, "{g}");
        assert!(g["share_of_income"].as_f64().unwrap() > 0.0, "{g}");
    }
    let future = &total["groups"][2];
    assert_eq!(future["group"], "future");
    assert!(
        future["top"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["category_id"].is_null() && t["label"] == "Aportes"),
        "{future}"
    );
}

/// Age (in completed years) of someone born on `born`, today.
fn age_of(born: &str) -> i32 {
    use chrono::Datelike;
    let b = chrono::NaiveDate::parse_from_str(born, "%Y-%m-%d").unwrap();
    let today = crate::time::today_local();
    let mut age = today.year() - b.year();
    if (today.month(), today.day()) < (b.month(), b.day()) {
        age -= 1;
    }
    age
}

fn members(api: &Api) -> Vec<Value> {
    api.call("household.list_members", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn member<'a>(list: &'a [Value], name: &str) -> &'a Value {
    list.iter().find(|m| m["name"] == name).unwrap()
}

/// Each person is born with an age, sex and relationship that fit the profile: the retiree has a
/// retiree's age, the parents are old enough for the children, and what comes from the assistant
/// holds.
#[test]
fn demo_people_have_ages_that_fit_the_profile() {
    // Retiree with no age given: from 64 to 71 years old.
    let (_d, api) = setup();
    let mut p = profile();
    p["kind"] = json!("retiree");
    p["adults"] = json!([{"name": "Helena", "income": 380_000, "retired": true, "sex": "female"}]);
    p["children"] = json!([]);
    new_demo(&api, "Aposentada", p);
    let list = members(&api);
    let helena = &member(&list, "Helena")["profile"];
    let age = age_of(helena["birth_date"].as_str().unwrap());
    assert!((64..=71).contains(&age), "{helena}");
    assert_eq!(helena["sex"], "female");
    assert_eq!(helena["relationship"], "titular");

    // Family with children and no ages: minor children, parents at least 20 years older than the
    // oldest, spouse marked.
    let (_d, api) = setup();
    let mut p = profile();
    p["children"] = json!(["Davi", "Elisa", "Felipe"]);
    new_demo(&api, "Família", p);
    let list = members(&api);
    let kids: Vec<i32> = ["Davi", "Elisa", "Felipe"]
        .iter()
        .map(|n| age_of(member(&list, n)["profile"]["birth_date"].as_str().unwrap()))
        .collect();
    assert!(kids.iter().all(|a| (0..=17).contains(a)), "{kids:?}");
    assert!(kids[0] > kids[1] && kids[1] > kids[2], "{kids:?}");
    for n in ["Ana", "Bruno"] {
        let a = age_of(member(&list, n)["profile"]["birth_date"].as_str().unwrap());
        assert!(a >= kids[0] + 20 && a <= 60, "{n}: {a}, children {kids:?}");
    }
    assert_eq!(member(&list, "Bruno")["profile"]["relationship"], "conjuge");
    assert_eq!(member(&list, "Davi")["profile"]["relationship"], "filho");
    // Without a sex given, it stays unspecified.
    assert!(member(&list, "Ana")["profile"]["sex"].is_null());

    // What comes from the assistant holds as given.
    let (_d, api) = setup();
    let mut p = profile();
    p["adults"] = json!([
        {"name": "Ana", "income": 900_000, "age": 41, "sex": "female"},
        {"name": "Bruno", "income": 600_000, "age": 43, "sex": "male"}
    ]);
    p["child_details"] = json!([{"age": 9, "sex": "female"}]);
    new_demo(&api, "Informado", p);
    let list = members(&api);
    let ages: Vec<(i32, Value)> = ["Ana", "Bruno", "Clara"]
        .iter()
        .map(|n| {
            let pr = &member(&list, n)["profile"];
            (
                age_of(pr["birth_date"].as_str().unwrap()),
                pr["sex"].clone(),
            )
        })
        .collect();
    assert_eq!(
        ages,
        vec![
            (41, json!("female")),
            (43, json!("male")),
            (9, json!("female"))
        ]
    );
}

/// Day-to-day of the ready-made profiles in Brazil: the week's groceries with the item-by-item
/// receipt, recognized subscriptions (with a price increase), Pix to the housekeeper and the
/// allowance, the installment purchase on the card and the FGTS of whoever has a formal job, with
/// the JAM by Caixa's rule and the deposits equal to the payslip's.
#[test]
fn comfortable_family_has_an_everyday_life() {
    let (_d, api) = setup();
    let mut p = profile();
    p["kind"] = json!("comfortable");
    p["months"] = json!(12);
    p["adults"] = json!([
        {"name": "Ana", "income": 1_200_000, "age": 41, "sex": "female"},
        {"name": "Bruno", "income": 850_000, "age": 43, "sex": "male"}
    ]);
    p["child_details"] = json!([{"age": 9, "sex": "female"}]);
    new_demo(&api, "Classe média", p);

    // Grocery receipts from recent months, with the items summing to the amount paid.
    let receipts = api.call("receipts.list", json!({})).unwrap();
    let receipts = receipts.as_array().unwrap();
    assert!(receipts.len() >= 8, "{} cupons", receipts.len());
    for r in receipts {
        if r["paid"].as_i64().unwrap() > 15_000 {
            assert!(r["item_count"].as_u64().unwrap() >= 5, "{r}");
        }
        assert_eq!(r["items_total"], r["paid"], "{r}");
        assert!(r["txn_id"].is_string(), "{r}");
    }
    let one = api
        .call("receipts.get", json!({"id": receipts[0]["id"]}))
        .unwrap();
    let sum: i64 = one["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["total"].as_i64().unwrap())
        .sum();
    assert_eq!(sum, one["paid"].as_i64().unwrap(), "{one}");

    // Recognized subscriptions (streaming, gym, phone...), and one with a price increase.
    let subs = api.call("subscriptions.list", json!({})).unwrap();
    let names: Vec<&str> = subs["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert!(names.len() >= 5, "{names:?}");
    assert!(subs["increases"].as_u64().unwrap() >= 1, "{subs}");

    // Pix to the housekeeper and Clara's allowance; the installment purchase.
    let d = descriptions(&api);
    assert!(d.iter().any(|x| x.starts_with("Pix para Cida")), "{d:?}");
    assert!(d.iter().any(|x| x == "Pix para Clara"), "{d:?}");
    let plans = api.call("cards.list_installment_plans", json!({})).unwrap();
    assert_eq!(plans.as_array().unwrap().len(), 1, "{plans}");

    // FGTS of both: JAM checked with no difference and deposits equal to the payslips.
    let fgts = api.call("fgts.list", json!({})).unwrap();
    let accounts = fgts.as_array().unwrap();
    assert_eq!(accounts.len(), 2, "{fgts}");
    for acc in accounts {
        assert!(acc["balance"].as_i64().unwrap() > 0, "{acc}");
        let check = api
            .call("fgts.check", json!({"account_id": acc["account_id"]}))
            .unwrap();
        assert!(check["jam_checked"].as_u64().unwrap() >= 6, "{check}");
        assert!(
            check["jam_mismatches"].as_array().unwrap().is_empty(),
            "{check}"
        );
        assert!(check["missing"].as_array().unwrap().is_empty(), "{check}");
        let short = check["deposits"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["status"] == "short")
            .count();
        assert_eq!(short, 0, "{check}");
    }
}

/// The hardship and low-income profiles have cooking gas; the family in hardship
/// pays the bank fee and gets help from the family; the young person splits the bar bill.
#[test]
fn each_profile_has_its_own_habits() {
    let (_d, api) = setup();
    let mut p = profile();
    p["kind"] = json!("struggling");
    p["overdraft_limit"] = json!(150_000);
    new_demo(&api, "Dificuldade", p);
    let d = descriptions(&api);
    assert!(d.iter().any(|x| x == "Gás de cozinha (botijão)"), "{d:?}");
    assert!(
        d.iter().any(|x| x == "Tarifa do pacote de serviços"),
        "{d:?}"
    );
    assert!(d.iter().any(|x| x.starts_with("Pix de Tia")), "{d:?}");

    let (_d, api) = setup();
    let mut p = profile();
    p["kind"] = json!("single");
    p["adults"] = json!([{"name": "Lucas", "income": 420_000, "age": 28, "sex": "male"}]);
    p["children"] = json!([]);
    new_demo(&api, "Sozinho", p);
    let d = descriptions(&api);
    assert!(d.iter().any(|x| x == "Academia"), "{d:?}");
    assert!(
        d.iter()
            .any(|x| x.starts_with("Pix de Marcos") || x.starts_with("Pix de Juliana")),
        "{d:?}"
    );
}

/// Believable profiles: the middle class with financing has the apartment and the car as assets
/// (linked to the contracts) and positive net worth; the low-income family saves little and does
/// not get the best band in financial health.
#[test]
fn comfortable_owns_what_it_finances_and_low_income_saves_little() {
    let (_d, api) = setup();
    let p = json!({"kind": "comfortable", "months": 12, "bank": "Itaú",
        "adults": [{"name": "Ana", "income": 1_200_000, "age": 41}, {"name": "Bruno", "income": 850_000, "age": 43}],
        "children": ["Clara"], "child_details": [{"age": 9}], "housing": "mortgage", "housing_amount": 420_000,
        "condo": 90_000, "utilities": 90_000, "groceries": 220_000, "transport": "car", "transport_amount": 130_000,
        "school_per_child": 240_000, "health": 180_000, "leisure": 130_000, "subscriptions": 15_000,
        "shopping": 70_000, "card_limit": 2_500_000, "reserve": 6_000_000, "reserve_monthly": 100_000,
        "stocks": 8_000_000, "fiis": 4_000_000, "fixed_income": 5_000_000, "invest_monthly": 150_000,
        "pension": 3_000_000, "pension_monthly": 50_000, "car_installment": 130_000, "car_installments_left": 24});
    new_demo(&api, "Classe média", p);
    let goods = api.call("physical.list", json!({})).unwrap();
    let goods = goods.as_array().unwrap();
    assert_eq!(goods.len(), 2, "{goods:?}");
    for g in goods {
        assert_eq!(g["loans"].as_array().map(Vec::len), Some(1), "{g}");
    }
    let nw = api.call("reports.net_worth", json!({})).unwrap();
    assert!(nw["net"].as_i64().unwrap() > 0, "{nw}");
    // Comfortable: no month in overdraft.
    let h = api.call("health.score", json!({})).unwrap();
    let caro = h["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == "credito_caro")
        .unwrap();
    assert_eq!(caro["status"], "good", "{caro}");

    let (_d, api) = setup();
    let p = json!({"kind": "low_income", "months": 12, "bank": "Caixa",
        "adults": [{"name": "Maria", "income": 190_000}, {"name": "João", "income": 160_000, "variable": true}],
        "children": ["Davi", "Elisa"], "benefit": 60_000, "housing": "rent", "housing_amount": 90_000,
        "utilities": 42_000, "groceries": 145_000, "transport": "public", "transport_amount": 38_000,
        "leisure": 15_000, "subscriptions": 4_000, "shopping": 28_000, "card_limit": 120_000,
        "reserve": 150_000, "reserve_monthly": 3_000, "personal_loan": 15_000,
        "personal_loan_left": 10, "level": "iniciante"});
    new_demo(&api, "Baixa renda", p);
    let h = api.call("health.score", json!({})).unwrap();
    let rate = h["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == "taxa_poupanca")
        .and_then(|m| m["value"].as_f64());
    if let Some(r) = rate {
        assert!(r < 0.10, "savings {r}: {h}");
    }
    let best = h["bands"].as_array().unwrap().last().unwrap()["label"].clone();
    assert_ne!(h["band"]["label"], best, "{h}");
}
