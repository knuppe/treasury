//! Full flow through the dispatcher (`Api::call`): detect → preview → commit →
//! `portfolio.summary` → `changes.undo`, with the made-up reports in `fixtures/`.

use base64::Engine as _;
use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

const NEGOCIACAO: &[u8] = include_bytes!("fixtures/negociacao.csv");
const NEGOCIACAO_XLSX: &[u8] = include_bytes!("fixtures/negociacao.xlsx");
const MOVIMENTACAO: &[u8] = include_bytes!("fixtures/movimentacao.csv");
const POSICAO: &[u8] = include_bytes!("fixtures/posicao-2024-05-02.xlsx");

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn file(name: &str, bytes: &[u8]) -> Value {
    json!({"filename": name, "content_base64": b64(bytes)})
}

fn all_files() -> Value {
    json!([
        file("negociacao.csv", NEGOCIACAO),
        file("movimentacao.csv", MOVIMENTACAO),
        file("posicao-2024-05-02.xlsx", POSICAO),
    ])
}

struct Env {
    _dir: tempfile::TempDir,
    api: Api,
    xp: String,
    nu: String,
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
            json!({"name": name, "kind": kind, "opening_date": "2023-12-01"}),
        )
        .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let xp = acc("XP Investimentos", "brokerage");
    let nu = acc("Nu Invest", "brokerage");
    // Checking account with the same word: the suggestion prefers the brokerage one.
    acc("Nu Conta", "checking");
    Env {
        _dir: dir,
        api,
        xp,
        nu,
    }
}

fn count(api: &Api, sql: &str) -> i64 {
    api.with_session(|s, _| Ok(s.db.conn().query_row(sql, [], |r| r.get::<_, i64>(0))?))
        .unwrap()
}

fn rows<'a>(v: &'a Value, list: &str) -> &'a Vec<Value> {
    v[list].as_array().unwrap()
}

fn find<'a>(v: &'a Value, list: &str, f: impl Fn(&Value) -> bool) -> &'a Value {
    rows(v, list)
        .iter()
        .find(|x| f(x))
        .unwrap_or_else(|| panic!("nothing found in {list}"))
}

fn position<'a>(check: &'a Value, key: &str, status: &str) -> &'a Value {
    check["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["asset_key"] == key && l["status"] == status)
        .unwrap_or_else(|| panic!("check without {key} {status}: {check:#}"))
}

#[test]
fn detect_recognizes_each_report() {
    let env = setup();
    let neg = env
        .api
        .call("b3.detect", file("negociacao.csv", NEGOCIACAO))
        .unwrap();
    assert_eq!(neg["format"], "csv");
    assert_eq!(neg["kinds"], json!(["trades"]));
    assert_eq!(neg["period_from"], "2024-01-10");
    assert_eq!(neg["period_to"], "2024-03-20");
    assert_eq!(neg["institutions"].as_array().unwrap().len(), 3);
    assert_eq!(neg["sheets"][0]["row_count"], 14);

    let mov = env
        .api
        .call("b3.detect", file("movimentacao.csv", MOVIMENTACAO))
        .unwrap();
    assert_eq!(mov["kinds"], json!(["movements"]));
    assert_eq!(mov["encoding"], "windows-1252");
    assert_eq!(mov["sheets"][0]["headers"][0], "Entrada/Saída");

    let pos = env
        .api
        .call("b3.detect", file("posicao.xlsx", POSICAO))
        .unwrap();
    assert_eq!(pos["format"], "xlsx");
    assert_eq!(pos["kinds"], json!(["position"]));
    let sheets = pos["sheets"].as_array().unwrap();
    let class_of =
        |name: &str| sheets.iter().find(|s| s["name"] == name).unwrap()["position_class"].clone();
    assert_eq!(class_of("Acoes"), "stock_br");
    assert_eq!(class_of("BDR"), "bdr");
    assert_eq!(class_of("ETF"), "etf_br");
    assert_eq!(class_of("Fundo de Investimento"), "fii");
    assert_eq!(class_of("Tesouro Direto"), "treasury_br");
    // Loans are not positions.
    let emp = sheets
        .iter()
        .find(|s| s["name"] == "Empréstimo de Ativos")
        .unwrap();
    assert!(emp["kind"].is_null());
    assert!(pos["warnings"][0].as_str().unwrap().contains("Empréstimo"));

    let xlsx = env
        .api
        .call("b3.detect", file("negociacao.xlsx", NEGOCIACAO_XLSX))
        .unwrap();
    assert_eq!(xlsx["kinds"], json!(["trades"]));
    assert_eq!(xlsx["sheets"][0]["header_line"], 3);
    assert_eq!(xlsx["sheets"][0]["row_count"], 14);
}

#[test]
fn full_flow_preview_commit_summary_undo() {
    let env = setup();
    let api = &env.api;
    // Operation entered by hand earlier (same date, asset, quantity and price as the B3).
    let hglg = api
        .call(
            "assets.create",
            json!({"ticker": "HGLG11", "name": "CSHG Logística", "class": "fii"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let manual = api
        .call(
            "portfolio.create_trade",
            json!({"asset_id": hglg, "account_id": env.nu, "date": "2024-01-10", "side": "buy",
                   "quantity": "10", "price": "160", "cash_movement": false}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let p = api
        .call("b3.preview", json!({"files": all_files()}))
        .unwrap();

    // Institutions: suggestion by name, preferring the brokerage account.
    let inst = |name: &str| {
        find(&p, "institutions", |i| {
            i["name"].as_str().unwrap().starts_with(name)
        })
        .clone()
    };
    assert_eq!(inst("XP")["account_id"], env.xp.as_str());
    assert_eq!(inst("XP")["from_suggestion"], true);
    assert_eq!(inst("NU")["account_id"], env.nu.as_str());
    assert!(inst("CORRETORA FICTICIA")["account_id"].is_null());
    assert!(
        p["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("CORRETORA FICTICIA"))
    );
    assert_eq!(p["uses_settlements"], false);

    // Operations.
    let trades = rows(&p, "trades");
    let status_of = |ticker: &str, side: &str, date: &str| {
        trades
            .iter()
            .filter(|t| t["ticker"] == ticker && t["side"] == side && t["date"] == date)
            .map(|t| t["status"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    // Sale listed before the purchase of the same day: it is recorded because purchases go first.
    assert_eq!(status_of("ITSA4", "sell", "2024-01-12"), vec!["new"]);
    // March sale listed on the first line (the file comes newest to oldest).
    assert_eq!(status_of("PETR4", "sell", "2024-03-20"), vec!["new"]);
    // Fractional becomes the same asset.
    let frac = find(&p, "trades", |t| t["date"] == "2024-03-15");
    assert_eq!(frac["ticker"], "PETR4");
    assert!(frac["note"].as_str().unwrap().contains("PETR4F"));
    // Two identical trades on the same day: two different ids.
    let twins: Vec<&str> = trades
        .iter()
        .filter(|t| t["ticker"] == "ITSA4" && t["date"] == "2024-01-16")
        .map(|t| t["external_id"].as_str().unwrap())
        .collect();
    assert_eq!(twins.len(), 2);
    assert_ne!(twins[0], twins[1]);
    assert!(twins[1].ends_with(":2"));
    assert_eq!(status_of("BBAS3", "buy", "2024-01-15"), vec!["no_account"]);
    let dup = find(&p, "trades", |t| t["ticker"] == "HGLG11");
    assert_eq!(dup["status"], "possible_duplicate");
    assert_eq!(dup["duplicate_of"], manual.as_str());
    // Treasury comes from settlement even with the trade report.
    let tesouro = find(&p, "trades", |t| t["asset_key"] == "tesouro:selic:2029");
    assert_eq!(tesouro["origin"], "movement");
    assert_eq!(tesouro["quantity"], "0.5");
    assert_eq!(tesouro["price"], "14000");
    // Fraction auction: sale of the 0.7 left over from the bonus shares.
    let leilao = find(&p, "trades", |t| t["origin"] == "fraction_auction");
    assert_eq!(leilao["quantity"], "0.7");
    assert_eq!(leilao["price"], "40");
    assert_eq!(leilao["status"], "new");

    // Income.
    let jcp = find(&p, "incomes", |i| i["kind"] == "jcp");
    assert_eq!(
        (
            jcp["gross"].as_i64(),
            jcp["tax_withheld"].as_i64(),
            jcp["net"].as_i64()
        ),
        (Some(400), Some(60), Some(340))
    );
    let fii = find(&p, "incomes", |i| i["ticker"] == "HGLG11");
    assert_eq!(fii["kind"], "fii_income");
    assert_eq!(fii["net"], 1100);
    let div = find(&p, "incomes", |i| i["ticker"] == "PETR4");
    assert_eq!(
        (
            div["kind"].as_str(),
            div["net"].as_i64(),
            div["status"].as_str()
        ),
        (Some("dividend"), Some(5000), Some("new"))
    );
    assert_eq!(
        find(&p, "incomes", |i| i["ticker"] == "BBAS3")["status"],
        "no_account"
    );
    assert_eq!(rows(&p, "incomes").len(), 5);

    // Events with a factor from the quantities.
    let split = find(&p, "actions", |a| a["kind"] == "split");
    assert_eq!(
        (
            split["ticker"].as_str(),
            split["held_quantity"].as_str(),
            split["factor"].as_str()
        ),
        (Some("ITSA4"), Some("20"), Some("2"))
    );
    assert_eq!(split["status"], "new");
    let bonus = find(&p, "actions", |a| a["kind"] == "bonus");
    assert_eq!(
        (
            bonus["held_quantity"].as_str(),
            bonus["file_quantity"].as_str(),
            bonus["factor"].as_str()
        ),
        (Some("57"), Some("5"), Some("0.1"))
    );
    assert!(bonus["note"].as_str().is_some());

    // Ignored lines, with the reason.
    let rules: Vec<&str> = rows(&p, "ignored")
        .iter()
        .map(|i| i["rule"].as_str().unwrap())
        .collect();
    for r in [
        "option",
        "forward",
        "settlement_duplicate",
        "fraction",
        "custody_transfer",
        "lending",
        "subscription_right",
        "balance_update",
        "forwarded_income",
        "manual_event",
    ] {
        assert!(rules.contains(&r), "missing rule {r}: {rules:?}");
    }
    assert_eq!(
        rules.iter().filter(|r| **r == "custody_transfer").count(),
        2
    );
    assert!(
        rows(&p, "ignored")
            .iter()
            .all(|i| !i["reason"].as_str().unwrap().is_empty())
    );

    // Assets: class from the position tab, suffix and name.
    let asset = |key: &str| find(&p, "assets", |a| a["key"] == key).clone();
    assert_eq!(asset("AAPL34")["class"], "bdr");
    assert_eq!(asset("IVVB11")["class"], "etf_br");
    assert_eq!(asset("KLBN11")["class"], "stock_br");
    assert_eq!(asset("KLBN11")["needs_review"], false);
    assert_eq!(asset("MXRF11")["class"], "fii");
    assert_eq!(asset("tesouro:selic:2029")["class"], "treasury_br");
    assert_eq!(asset("HGLG11")["create"], false);
    assert!(rows(&p, "assets").iter().all(|a| a["key"] != "VALE3"));
    assert_eq!(p["counts"]["new_assets"], 7);

    // Check against the position of 2024-05-02.
    let check = &p["position_check"];
    assert_eq!(check["date"], "2024-05-02");
    for key in [
        "PETR4",
        "ITSA4",
        "KLBN11",
        "IVVB11",
        "HGLG11",
        "tesouro:selic:2029",
    ] {
        position(check, key, "match");
    }
    assert_eq!(
        position(check, "VALE3", "divergent")["computed_quantity"],
        "0"
    );
    position(check, "MXRF11", "divergent");
    // Custody transfer XP → Nu: the B3 shows it at Nu, the app still has it at XP.
    let moved = position(check, "AAPL34", "divergent");
    assert!(
        moved["note"]
            .as_str()
            .unwrap()
            .contains("transferência de custódia")
    );
    assert_eq!(
        position(check, "AAPL34", "missing_in_b3")["account_id"],
        env.xp.as_str()
    );
    position(check, "BBAS3", "no_account");
    assert_eq!(check["matched"], 6);

    // Commit: nothing recorded by the preview.
    assert_eq!(count(api, "SELECT count(*) FROM trade"), 1);
    let r = api.call("b3.commit", json!({"token": p["token"]})).unwrap();
    assert_eq!(r["trades"], 12);
    assert_eq!(r["incomes"], 4);
    assert_eq!(r["actions"], 2);
    assert_eq!(r["assets_created"], 7);
    assert_eq!(r["errors"], json!([]));
    let cs = r["change_set_id"].as_str().unwrap().to_string();
    assert_eq!(count(api, "SELECT count(*) FROM trade"), 13);
    // No cash movement by default (only the manual operation's, which has none either).
    assert_eq!(count(api, "SELECT count(*) FROM txn"), 0);
    assert_eq!(
        count(
            api,
            "SELECT count(*) FROM income_event WHERE source = 'b3' AND external_id LIKE 'b3:prov:%'"
        ),
        4
    );
    assert_eq!(
        count(
            api,
            "SELECT count(*) FROM corporate_action WHERE source = 'b3'"
        ),
        2
    );
    assert_eq!(
        count(
            api,
            "SELECT count(*) FROM asset WHERE ticker IS NULL AND class = 'treasury_br' AND price_provider = 'tesouro'"
        ),
        1
    );
    let author: String = api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT author_kind FROM change_set WHERE id = ?1",
                [&cs],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(author, "import");
    // The preview was consumed.
    assert!(api.call("b3.commit", json!({"token": p["token"]})).is_err());

    // Portfolio through the engine.
    let s = api
        .call(
            "portfolio.summary",
            json!({"as_of": "2024-05-02", "by_account": true}),
        )
        .unwrap();
    let qty = |ticker: &str, acc: &str| {
        s["positions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["asset"]["ticker"] == ticker && p["account_id"] == acc)
            .map(|p| p["quantity"].as_str().unwrap().to_string())
    };
    assert_eq!(qty("PETR4", &env.xp).as_deref(), Some("62"));
    assert_eq!(qty("ITSA4", &env.xp).as_deref(), Some("40"));
    assert_eq!(qty("HGLG11", &env.nu).as_deref(), Some("10"));
    assert!(s["income"].as_i64().unwrap() > 0);
    let incomes = api.call("portfolio.list_income", json!({})).unwrap();
    let jcp = incomes
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "jcp")
        .unwrap();
    assert_eq!(
        (
            jcp["gross"].as_i64(),
            jcp["tax_withheld"].as_i64(),
            jcp["net"].as_i64()
        ),
        (Some(400), Some(60), Some(340))
    );

    // The same import again: everything duplicated.
    let again = api
        .call("b3.preview", json!({"files": all_files()}))
        .unwrap();
    assert_eq!(again["counts"]["trades"], 0, "{:#}", again["counts"]);
    assert_eq!(again["counts"]["incomes"], 0);
    assert_eq!(again["counts"]["actions"], 0);
    assert_eq!(again["counts"]["new_assets"], 0);
    assert!(
        rows(&again, "trades")
            .iter()
            .filter(|t| t["ticker"] == "PETR4")
            .all(|t| t["status"] == "duplicate")
    );
    assert!(
        rows(&again, "actions")
            .iter()
            .all(|a| a["status"] == "duplicate")
    );

    // Undo the whole import.
    api.call("changes.undo", json!({"id": cs})).unwrap();
    assert_eq!(count(api, "SELECT count(*) FROM trade"), 1);
    assert_eq!(count(api, "SELECT count(*) FROM income_event"), 0);
    assert_eq!(count(api, "SELECT count(*) FROM corporate_action"), 0);
    assert_eq!(count(api, "SELECT count(*) FROM asset"), 1);
    let s = api
        .call("portfolio.summary", json!({"as_of": "2024-05-02"}))
        .unwrap();
    assert_eq!(s["positions"].as_array().unwrap().len(), 1);
}

#[test]
fn settlements_only_without_trades_report_and_never_both() {
    let env = setup();
    let api = &env.api;
    // Movement only: settlement becomes the purchase, with an estimated date.
    let p = api
        .call(
            "b3.preview",
            json!({"files": [file("movimentacao.csv", MOVIMENTACAO)]}),
        )
        .unwrap();
    assert_eq!(p["uses_settlements"], true);
    let petr = find(&p, "trades", |t| {
        t["ticker"] == "PETR4" && t["origin"] == "settlement"
    });
    // Settlement on 2024-01-12 (Friday) → trade on 2024-01-10.
    assert_eq!(petr["date"], "2024-01-10");
    assert!(petr["external_id"].as_str().unwrap().starts_with("b3:liq:"));
    assert!(
        p["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("liquidações"))
    );
    // ITSA4 has no operations in this file: the split is left for review.
    let split = find(&p, "actions", |a| a["kind"] == "split");
    assert_eq!(split["status"], "review");
    assert!(split["note"].as_str().unwrap().contains("sem posição"));

    // With the trade already recorded, the settlement of the same trade is a duplicate.
    let n = api
        .call(
            "b3.preview",
            json!({"files": [file("negociacao.csv", NEGOCIACAO)]}),
        )
        .unwrap();
    api.call("b3.commit", json!({"token": n["token"]})).unwrap();
    let p = api
        .call(
            "b3.preview",
            json!({"files": [file("movimentacao.csv", MOVIMENTACAO)]}),
        )
        .unwrap();
    let petr = find(&p, "trades", |t| {
        t["ticker"] == "PETR4" && t["origin"] == "settlement"
    });
    assert_eq!(petr["status"], "duplicate");
    assert!(petr["note"].as_str().unwrap().contains("negociação"));
    // And the split now has a position to compute the factor.
    let split = find(&p, "actions", |a| a["kind"] == "split");
    assert_eq!(
        (split["status"].as_str(), split["factor"].as_str()),
        (Some("new"), Some("2"))
    );

    // The reverse: a trade after the settlement does not duplicate either.
    let env = setup();
    let api = &env.api;
    let m = api
        .call(
            "b3.preview",
            json!({"files": [file("movimentacao.csv", MOVIMENTACAO)]}),
        )
        .unwrap();
    api.call("b3.commit", json!({"token": m["token"]})).unwrap();
    let n = api
        .call(
            "b3.preview",
            json!({"files": [file("negociacao.csv", NEGOCIACAO)]}),
        )
        .unwrap();
    let petr = find(&n, "trades", |t| {
        t["ticker"] == "PETR4" && t["date"] == "2024-01-10"
    });
    assert_eq!(petr["status"], "duplicate");
    assert!(petr["note"].as_str().unwrap().contains("liquidação"));
}

#[test]
fn account_choice_review_approval_and_cash_option() {
    let env = setup();
    let api = &env.api;
    // The user rejects Nu (null) and points the fictional brokerage to XP.
    let p = api
        .call(
            "b3.preview",
            json!({
                "files": [file("movimentacao.csv", MOVIMENTACAO)],
                "accounts": [
                    {"institution": "nu invest corretora de valores s.a.", "account_id": null},
                    {"institution": "CORRETORA FICTICIA DTVM LTDA", "account_id": env.xp},
                ],
            }),
        )
        .unwrap();
    let nu = find(&p, "institutions", |i| {
        i["name"].as_str().unwrap().starts_with("NU")
    });
    assert!(nu["account_id"].is_null());
    assert_eq!(nu["from_suggestion"], false);
    assert_eq!(nu["suggested_account_id"], env.nu.as_str());
    assert_eq!(
        find(&p, "incomes", |i| i["ticker"] == "HGLG11")["status"],
        "no_account"
    );
    assert_eq!(
        find(&p, "incomes", |i| i["ticker"] == "BBAS3")["status"],
        "new"
    );

    // A nonexistent account is rejected.
    assert!(
        api.call(
            "b3.preview",
            json!({"files": [file("movimentacao.csv", MOVIMENTACAO)],
                   "accounts": [{"institution": "XP INVESTIMENTOS CCTVM S/A", "account_id": "nao-existe"}]}),
        )
        .is_err()
    );

    // Event under review only enters approved, with the given factor; cash on.
    let split = find(&p, "actions", |a| a["kind"] == "split");
    assert_eq!(split["status"], "review");
    let idx = split["index"].as_u64().unwrap();
    assert!(
        api.call(
            "b3.commit",
            json!({"token": p["token"], "approve_actions": [{"index": 9999}]}),
        )
        .is_err()
    );
    let r = api
        .call(
            "b3.commit",
            json!({"token": p["token"], "approve_actions": [{"index": idx, "factor": "2"}], "cash_movement": true}),
        )
        .unwrap();
    assert_eq!(r["actions"], 2, "{r:#}");
    assert_eq!(
        count(
            api,
            "SELECT count(*) FROM corporate_action WHERE kind = 'split' AND factor = '2'"
        ),
        1
    );
    // With cash on, income and operations post to the brokerage cash.
    assert!(count(api, "SELECT count(*) FROM txn") > 0);
}

#[test]
fn xlsx_and_csv_give_the_same_trades() {
    let env = setup();
    let ids = |name: &str, bytes: &[u8]| {
        let p = env
            .api
            .call("b3.preview", json!({"files": [file(name, bytes)]}))
            .unwrap();
        let mut v: Vec<(String, String, String)> = rows(&p, "trades")
            .iter()
            .map(|t| {
                (
                    t["external_id"].as_str().unwrap().to_string(),
                    t["quantity"].as_str().unwrap().to_string(),
                    t["price"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        v.sort();
        (v, rows(&p, "ignored").len())
    };
    let csv = ids("negociacao.csv", NEGOCIACAO);
    let xlsx = ids("negociacao.xlsx", NEGOCIACAO_XLSX);
    assert_eq!(csv.0.len(), 12);
    assert_eq!(csv, xlsx);
}

#[test]
fn same_file_twice_in_one_preview_counts_once() {
    let env = setup();
    let p = env
        .api
        .call(
            "b3.preview",
            json!({"files": [file("a.csv", NEGOCIACAO), file("b.xlsx", NEGOCIACAO_XLSX)]}),
        )
        .unwrap();
    let dups = rows(&p, "trades")
        .iter()
        .filter(|t| t["status"] == "duplicate")
        .count();
    assert_eq!(dups, 12);
    assert_eq!(p["counts"]["trades"], 11);
}

#[test]
fn ai_client_can_preview_and_propose_the_import() {
    let env = setup();
    let api = &env.api;
    let agent = crate::api::Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let p = api
        .call_as(
            &agent,
            "b3.preview",
            json!({"files": [file("negociacao.csv", NEGOCIACAO)]}),
        )
        .unwrap();
    assert_eq!(p["counts"]["trades"], 11);
    let r = api
        .call_as(&agent, "b3.commit", json!({"token": p["token"]}))
        .unwrap();
    assert_eq!(r["status"], "pending_approval");
    assert_eq!(
        count(api, "SELECT count(*) FROM trade"),
        0,
        "nothing recorded before approval"
    );
    let list = api
        .call("ai.list_proposals", json!({"status": "pending"}))
        .unwrap();
    assert_eq!(list[0]["command"], "b3.commit");
    assert_eq!(list[0]["level"], "sensitive");
    api.call("ai.approve", json!({"ids": [r["proposal_id"]]}))
        .unwrap();
    assert_eq!(count(api, "SELECT count(*) FROM trade"), 11);
    // Read-only client only detects.
    let ro = crate::api::Actor::Mcp {
        client: "leitor".into(),
        read_only: true,
    };
    assert!(
        api.call_as(&ro, "b3.detect", file("negociacao.csv", NEGOCIACAO))
            .is_ok()
    );
    assert!(
        api.call_as(
            &ro,
            "b3.preview",
            json!({"files": [file("negociacao.csv", NEGOCIACAO)]})
        )
        .is_err()
    );
}

#[test]
fn income_recorded_from_the_official_notice_is_not_imported_again() {
    let env = setup();
    let api = &env.api;
    let mxrf = api
        .call(
            "assets.create",
            json!({"ticker": "MXRF11", "name": "Maxi Renda", "class": "fii"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // Recorded by the income ledger one day off the statement's credit date.
    let id = api
        .call(
            "portfolio.create_income",
            json!({"asset_id": mxrf, "account_id": env.nu, "kind": "fii_income", "ex_date": "2024-01-31",
                   "pay_date": "2024-02-19", "gross": 1000, "cash_movement": false}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE income_event SET source = 'official' WHERE id = ?1",
            [&id],
        )?;
        Ok(())
    })
    .unwrap();
    let p = api
        .call("b3.preview", json!({"files": all_files()}))
        .unwrap();
    let row = find(&p, "incomes", |i| i["ticker"] == "MXRF11");
    assert_eq!(row["status"], "duplicate", "{row:#}");
    assert_eq!(row["duplicate_of"], id.as_str());
}
