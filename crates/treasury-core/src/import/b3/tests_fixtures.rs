//! Acceptance test with the two anonymized reports of the B3 Investor Area (movements and
//! position as the B3 delivers them: same sheets, headers, styles and formats; quantities,
//! prices, dates, account number and contract codes are synthetic). Keep the position
//! file's name: the check date comes from it.

use base64::Engine as _;
use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

const MOV_NAME: &str = "movimentacao-2026-11-03-22-07-23.xlsx";
const POS_NAME: &str = "posicao-2026-11-03-22-03-37.xlsx";
const MOV: &[u8] = include_bytes!("fixtures/movimentacao-2026-11-03-22-07-23.xlsx");
const POS: &[u8] = include_bytes!("fixtures/posicao-2026-11-03-22-03-37.xlsx");

fn file(name: &str, bytes: &[u8]) -> Value {
    json!({"filename": name, "content_base64": base64::engine::general_purpose::STANDARD.encode(bytes)})
}

fn files() -> Value {
    json!([file(MOV_NAME, MOV), file(POS_NAME, POS)])
}

struct Env {
    _dir: tempfile::TempDir,
    api: Api,
    nu: String,
    via: String,
    itau: String,
}

fn setup() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    let acc = |name: &str, kind: &str| {
        api.call(
            "accounts.create",
            json!({"name": name, "kind": kind, "opening_date": "2025-01-01"}),
        )
        .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let nu = acc("NuInvest", "brokerage");
    let via = acc("Viacredi", "checking");
    let itau = acc("Itaú", "checking");
    Env {
        _dir: dir,
        api,
        nu,
        via,
        itau,
    }
}

fn rows<'a>(v: &'a Value, list: &str) -> &'a Vec<Value> {
    v[list].as_array().unwrap()
}

fn by_status(v: &Value, list: &str, status: &str) -> usize {
    rows(v, list)
        .iter()
        .filter(|r| r["status"] == status)
        .count()
}

#[test]
fn detect_reads_both_reports() {
    let env = setup();
    let mov = env.api.call("b3.detect", file(MOV_NAME, MOV)).unwrap();
    assert_eq!(mov["kinds"], json!(["movements"]));
    assert_eq!(mov["institutions"].as_array().unwrap().len(), 4);
    assert_eq!(mov["period_from"], "2025-11-03");
    assert_eq!(mov["period_to"], "2026-10-26");
    let pos = env.api.call("b3.detect", file(POS_NAME, POS)).unwrap();
    let sheets: Vec<(String, u64)> = pos["sheets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["kind"] == "position")
        .map(|s| {
            (
                s["position_class"].as_str().unwrap().to_string(),
                s["row_count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        sheets,
        vec![
            ("stock_br".into(), 3),
            ("fii".into(), 2),
            ("fixed_income".into(), 9),
            ("treasury_br".into(), 2),
        ]
    );
}

#[test]
fn preview_in_an_empty_vault() {
    let env = setup();
    let p = env
        .api
        .call("b3.preview", json!({"files": files()}))
        .unwrap();

    // Custodians: the two names of Nu are one.
    let inst = rows(&p, "institutions");
    assert_eq!(inst.len(), 3, "{inst:#?}");
    let of = |id: &str| inst.iter().find(|i| i["account_id"] == id).unwrap();
    assert_eq!(of(&env.nu)["aliases"].as_array().unwrap().len(), 2);
    assert_eq!(of(&env.via)["account_name"], "Viacredi");
    assert_eq!(of(&env.itau)["account_name"], "Itaú");
    assert_eq!(p["counts"]["no_account"], 0);

    // Trades: 4 settlements, 1 "COMPRA / VENDA", 2 Tesouro purchases, 7 RDB investments and
    // 6 redemptions; 7 are sales whose purchase is older than the file: imported all the
    // same, with a pending item.
    let trades = rows(&p, "trades");
    assert_eq!(trades.len(), 20, "{trades:#?}");
    assert_eq!(by_status(&p, "trades", "new"), 13);
    assert_eq!(by_status(&p, "trades", "warning"), 7);
    assert_eq!(by_status(&p, "trades", "error"), 0);
    assert_eq!(p["counts"]["trades"], 20);
    assert_eq!(p["counts"]["warnings"], 7);
    let settlements: Vec<&Value> = trades
        .iter()
        .filter(|t| t["origin"] == "settlement")
        .collect();
    assert_eq!(settlements.len(), 4);
    assert!(settlements.iter().all(|t| t["note"].is_string()));

    // Incomes: 35 FII, 12 JCP, 9 dividends, 2 Tesouro interest with tax.
    let incomes = rows(&p, "incomes");
    assert_eq!(incomes.len(), 58);
    let interest: Vec<&Value> = incomes.iter().filter(|i| i["kind"] == "interest").collect();
    assert_eq!(interest.len(), 2);
    let mut gross: Vec<i64> = interest
        .iter()
        .map(|i| i["gross"].as_i64().unwrap())
        .collect();
    gross.sort();
    assert_eq!(gross, vec![7804, 7917]);
    assert!(
        interest
            .iter()
            .all(|i| i["tax_withheld"].as_i64().unwrap() > 0
                && i["net"].as_i64().unwrap() < i["gross"].as_i64().unwrap())
    );

    // The ITSA4 bonus: 30 shares, by quantity, without review.
    let actions = rows(&p, "actions");
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0]["asset_key"], "ITSA4");
    assert_eq!(actions[0]["file_quantity"], "30");
    assert_eq!(actions[0]["status"], "new");

    // Ignored: 4 custody snapshots (VALE3) and 12 subscription rows; nothing unknown.
    let ignored = rows(&p, "ignored");
    assert_eq!(ignored.len(), 16, "{ignored:#?}");
    assert_eq!(
        ignored
            .iter()
            .filter(|i| i["rule"] == "custody_snapshot")
            .count(),
        4
    );
    assert_eq!(
        ignored
            .iter()
            .filter(|i| i["rule"] == "subscription_right")
            .count(),
        12
    );
    assert_eq!(
        trades.len() + incomes.len() + actions.len() + ignored.len(),
        95
    );

    // Position check: 16 lines, 9 match.
    let check = &p["position_check"];
    let lines = check["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 16, "{check:#}");
    assert_eq!(check["matched"], 9);
    let matched: Vec<&str> = lines
        .iter()
        .filter(|l| l["status"] == "match")
        .map(|l| l["asset_key"].as_str().unwrap())
        .collect();
    for key in [
        "PSSA3",
        "rf:CDBU24FGBLH",
        "rf:RDB00786TZO",
        "rf:RDB33853SCX",
        "rf:RDB36193CTA",
        "rf:RDB8099808P",
        "rf:RDBY4709WK2",
        "rf:RDBY7445VO7",
    ] {
        assert!(matched.contains(&key), "{key} should match: {matched:?}");
    }
    let fixed: i64 = lines
        .iter()
        .filter(|l| l["asset_key"].as_str().unwrap().starts_with("rf:"))
        .map(|l| l["b3_value"].as_i64().unwrap())
        .sum();
    assert_eq!(fixed, 1_364_501);

    // One asset per contract.
    let assets = rows(&p, "assets");
    let keys: Vec<&str> = assets.iter().map(|a| a["key"].as_str().unwrap()).collect();
    let mut unique = keys.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), keys.len());
    // 7 listed, 2 Tesouro, 2 CDB and 9 RDB. The two RDBs only in the position report
    // (RDB48884ZYQ, RDBY9629CZB) have nothing to record yet: they are created with their
    // opening positions (next test).
    assert_eq!(p["counts"]["new_assets"], 20, "{keys:?}");
}

#[test]
fn with_the_opening_positions_everything_matches() {
    let env = setup();
    let p = env
        .api
        .call("b3.preview", json!({"files": files()}))
        .unwrap();
    let suggested = rows(&p, "openings");
    let want = [
        ("ITSA4", "1500"),
        ("TAEE11", "238"),
        ("MXRF11", "1939"),
        ("RZTR11", "669"),
        ("RECR11", "109"),
        ("VALE3", "111"),
        ("tesouro:ipca-js:2035", "0.51"),
        ("rf:CDB2326Y6KV", "304639"),
        ("rf:RDB1232CWCO", "3000000"),
        ("rf:RDB05928TN9", "280812"),
        ("rf:RDB48884ZYQ", "3441"),
        ("rf:RDBY9629CZB", "3724"),
    ];
    for (key, qty) in want {
        let o = suggested
            .iter()
            .find(|o| o["asset_key"] == key)
            .unwrap_or_else(|| panic!("no opening for {key}: {suggested:#?}"));
        assert_eq!(o["quantity"], qty, "{key}");
        assert_eq!(o["date"], "2025-11-02");
    }
    let openings: Vec<Value> = suggested
        .iter()
        .map(|o| {
            json!({"asset_key": o["asset_key"], "account_id": o["account_id"],
                   "quantity": o["quantity"], "unit_cost": "1"})
        })
        .collect();
    let p = env
        .api
        .call(
            "b3.preview",
            json!({"files": files(), "openings": openings}),
        )
        .unwrap();
    assert_eq!(p["counts"]["errors"], 0, "{:#?}", p["trades"]);
    let check = &p["position_check"];
    assert_eq!(check["matched"], 16, "{check:#}");
    assert_eq!(check["divergent"], 0);
    assert_eq!(p["counts"]["new_assets"], 22);
    // With 1500 ITSA4 before the bonus, the factor is derived (30 new shares per 1500).
    assert_eq!(rows(&p, "actions")[0]["factor"], "0.02");
}

#[test]
fn without_accounts_the_import_creates_them_and_remembers() {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    let p = api.call("b3.preview", json!({"files": files()})).unwrap();
    let inst = rows(&p, "institutions");
    assert_eq!(inst.len(), 3);
    let choices: Vec<Value> = inst
        .iter()
        .map(|i| {
            assert!(i["proposed_new_account"]["name"].is_string());
            assert!(i["proposed_new_account"]["kind"].is_string());
            assert!(i["proposed_new_account"]["institution_name"].is_string());
            json!({"institution": i["name"], "account_id": null, "new_account": i["proposed_new_account"]})
        })
        .collect();
    let p = api
        .call("b3.preview", json!({"files": files(), "accounts": choices}))
        .unwrap();
    let r = api.call("b3.commit", json!({"token": p["token"]})).unwrap();
    assert_eq!(r["accounts_created"].as_array().unwrap().len(), 3);
    // The 7 sales without a purchase in the file were written and wait for their purchase.
    assert_eq!(r["trades"], 20);
    assert_eq!(r["pending_sales"], 7);
    let pending = api.call("portfolio.uncovered_sales", json!({})).unwrap();
    assert_eq!(pending["open"], 7, "{pending:#}");
    let p2 = api.call("b3.preview", json!({"files": files()})).unwrap();
    assert!(
        rows(&p2, "institutions")
            .iter()
            .all(|i| i["mapped"] == true)
    );
    for list in ["trades", "incomes", "actions"] {
        assert!(
            rows(&p2, list).iter().all(|x| x["status"] == "duplicate"),
            "{list}: {:#?}",
            p2[list]
        );
    }
}
