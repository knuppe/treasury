use chrono::NaiveDate;
use serde_json::{Value, json};

use super::regressive_rate;
use crate::api::Api;
use crate::paths::TreasuryPaths;

fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

fn setup() -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Seguradora", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let plan = api
        .call(
            "assets.create",
            json!({"name": "VGBL Brasilprev RV", "class": "pension"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // Old contribution (more than 10 years: 10%) and a recent one (35%), quota 1 → 2.
    for (date, qty) in [("2014-03-10", "10000"), ("2026-01-10", "5000")] {
        api.call(
            "portfolio.create_trade",
            json!({"asset_id": plan, "account_id": acc, "date": date, "side": "buy",
                   "quantity": qty, "price": "1", "cash_movement": false}),
        )
        .unwrap();
    }
    api.call(
        "market.set_price",
        json!({"asset_id": plan, "date": "2026-10-15", "close": "2"}),
    )
    .unwrap();
    (dir, api, plan)
}

#[test]
fn regressive_bands_by_age_of_each_contribution() {
    assert_eq!(
        regressive_rate(d("2026-01-10"), d("2026-10-16")),
        (35, Some((30, d("2028-01-11"))))
    );
    assert_eq!(regressive_rate(d("2020-01-10"), d("2026-10-16")).0, 20);
    assert_eq!(
        regressive_rate(d("2014-03-10"), d("2026-10-16")),
        (10, None)
    );
}

/// Law 11.053/2004 art. 1, I to VI, and the IRPF 2026 Q&A, question 187: each bracket
/// holds up to the term "less than or equal to" 2, 4, 6, 8 and 10 years; the next one only the day
/// after.
#[test]
fn regressive_band_holds_on_the_anniversary_day() {
    let aporte = d("2016-03-15");
    for (anos, rate) in [(2, 35u32), (4, 30), (6, 25), (8, 20), (10, 15)] {
        let dia = d(&format!("{}-03-15", 2016 + anos));
        assert_eq!(regressive_rate(aporte, dia).0, rate, "{anos} exact years");
        assert_eq!(
            regressive_rate(aporte, dia + chrono::Duration::days(1)).0,
            rate - 5,
            "{anos} years and one day"
        );
    }
    assert_eq!(regressive_rate(aporte, d("2026-03-16")), (10, None));
}

#[test]
fn vgbl_regressive_view_and_withdrawals() {
    let (_d, api, plan) = setup();
    // No plan: warns and assumes regressive VGBL.
    let v = api.call("pension.get", json!({"asset_id": plan})).unwrap();
    assert!(!v["warnings"].as_array().unwrap().is_empty());
    api.call(
        "pension.set_plan",
        json!({"asset_id": plan, "plan_type": "vgbl", "tax_regime": "regressive",
               "insurer": "Brasilprev", "admin_fee_pct": "1,0"}),
    )
    .unwrap();
    let v = api.call("pension.get", json!({"asset_id": plan})).unwrap();
    assert_eq!(v["contributed"], 1_500_000);
    assert_eq!(v["value"], 3_000_000);
    assert_eq!(v["gain"], 1_500_000);
    assert_eq!(v["plan"]["admin_fee_pct"], "1");
    let lots = v["lots"].as_array().unwrap();
    assert_eq!(lots[0]["regressive_rate_pct"], 10);
    assert_eq!(lots[1]["regressive_rate_pct"], 35);
    // Everything: IR only on the earnings, by the age of each contribution.
    // 10% × 10,000 + 35% × 5,000 = 2,750.00.
    let full = &v["full_withdrawal"];
    assert_eq!(full["gross"], 3_000_000);
    assert_eq!(full["tax_base"], 1_500_000);
    assert_eq!(full["tax"], 275_000);
    assert_eq!(full["net"], 2_725_000);
    // Partial: the oldest contribution goes out first (10%).
    let s = api
        .call(
            "pension.simulate",
            json!({"asset_id": plan, "amount": 2_000_000}),
        )
        .unwrap();
    assert_eq!(
        (s["tax_base"].as_i64(), s["tax"].as_i64()),
        (Some(1_000_000), Some(100_000))
    );
    assert!(
        api.call(
            "pension.simulate",
            json!({"asset_id": plan, "amount": 9_000_000})
        )
        .is_err()
    );
}

#[test]
fn pgbl_and_progressive_tax_bases() {
    let (_d, api, plan) = setup();
    let sim = |api: &Api, t: &str, r: &str| -> Value {
        api.call(
            "pension.set_plan",
            json!({"asset_id": plan, "plan_type": t, "tax_regime": r}),
        )
        .unwrap()["full_withdrawal"]
            .clone()
    };
    // Regressive PGBL: the base is the whole amount. 10% × 20,000 + 35% × 10,000 = 5,500.00.
    let p = sim(&api, "pgbl", "regressive");
    assert_eq!(
        (p["tax_base"].as_i64(), p["tax"].as_i64()),
        (Some(3_000_000), Some(550_000))
    );
    // Progressive VGBL: 15% of the earnings withheld at source.
    let v = sim(&api, "vgbl", "progressive");
    assert_eq!(v["tax"], 225_000);
    // Only assets of the Previdência class have a plan.
    let stock = api
        .call(
            "assets.create",
            json!({"ticker": "PETR4", "name": "Petrobras", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        api.call(
            "pension.set_plan",
            json!({"asset_id": stock, "plan_type": "vgbl", "tax_regime": "regressive"}),
        )
        .is_err()
    );
}

#[test]
fn ir_report_lists_vgbl_in_group_99_and_explains_pgbl() {
    let (_d, api, plan) = setup();
    let year = api.call("tax.year", json!({"year": 2026})).unwrap();
    let h = year["holdings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["asset_id"] == plan.as_str())
        .unwrap()
        .clone();
    assert_eq!(h["group_code"], "99.06");
    assert_eq!(h["cost"], 1_500_000, "VGBL goes by the amount invested");
    api.call(
        "pension.set_plan",
        json!({"asset_id": plan, "plan_type": "pgbl", "tax_regime": "regressive"}),
    )
    .unwrap();
    let year = api.call("tax.year", json!({"year": 2026})).unwrap();
    let h = year["holdings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["asset_id"] == plan.as_str())
        .unwrap()
        .clone();
    assert!(h["description"].as_str().unwrap().contains("PGBL"));
}

#[test]
fn plan_is_saved_with_the_asset() {
    let (_d, api, _) = setup();
    let id = api
        .call(
            "assets.create",
            json!({"name": "PGBL Icatu", "class": "pension", "cnpj": "12.345.678/0001-90",
                   "pension": {"plan_type": "pgbl", "tax_regime": "progressive",
                               "insurer": " Icatu ", "admin_fee_pct": "0,8",
                               "start_date": "2020-05-01"}}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let v = api.call("pension.get", json!({"asset_id": id})).unwrap();
    let p = &v["plan"];
    assert_eq!(
        (p["plan_type"].as_str(), p["tax_regime"].as_str()),
        (Some("pgbl"), Some("progressive"))
    );
    assert_eq!(p["insurer"], "Icatu");
    assert_eq!(p["admin_fee_pct"], "0.8");
    assert_eq!(p["start_date"], "2020-05-01");

    // A plan registered with no contribution stays in "Em carteira" (to receive contributions), not
    // in the
    // closed positions nor in the ones with trades.
    let names = |position: &str| {
        api.call("assets.list", json!({"position": position}))
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(names("open"), ["PGBL Icatu", "VGBL Brasilprev RV"]);
    assert!(names("closed").is_empty());
    assert_eq!(names("traded"), ["VGBL Brasilprev RV"]);

    // Plan only on pension; nothing is written when it fails.
    let err = api
        .call(
            "assets.create",
            json!({"name": "Ação", "class": "stock_br", "ticker": "ABCD3",
                   "pension": {"plan_type": "vgbl", "tax_regime": "regressive"}}),
        )
        .unwrap_err();
    assert!(err.to_string().contains("Previdência"), "{err}");
    let list = api.call("assets.list", json!({})).unwrap();
    assert!(!list.to_string().contains("ABCD3"));
    // An invalid date in the plan undoes the whole registration.
    assert!(
        api.call(
            "assets.create",
            json!({"name": "VGBL ruim", "class": "pension",
                   "pension": {"plan_type": "vgbl", "tax_regime": "regressive",
                               "start_date": "31/02/2020"}}),
        )
        .is_err()
    );
    assert!(
        !api.call("assets.list", json!({}))
            .unwrap()
            .to_string()
            .contains("VGBL ruim")
    );
}

const QUANTA: &str = include_str!("fixtures/quanta_extrato.txt");

#[test]
fn reads_and_imports_a_pension_fund_statement() {
    let (_d, api, _) = setup();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Previdência Quanta", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let p = api
        .call("pension.statement_preview", json!({"text": QUANTA}))
        .unwrap();
    let h = &p["header"];
    assert_eq!(h["plan_name"], "PLANOPREV-Arrojado");
    assert_eq!(h["plan_cnpj"], "11.444.777/0001-61");
    assert_eq!(h["cnpb"], "2010.0042-19");
    assert_eq!(h["adhesion"], "2022-03-14");
    assert_eq!(h["admin_fee_pct"], "0.25");
    assert_eq!(h["loading_fee_pct"], "0.00");
    assert_eq!(h["tax_regime"], "regressive");
    let kinds: Vec<&str> = p["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "participant",
            "risk_disability",
            "risk_death",
            "employer",
            "participant"
        ]
    );
    let r0 = &p["rows"][0];
    assert_eq!(
        (r0["paid"].as_i64(), r0["retirement"].as_i64()),
        (Some(18000), Some(18000))
    );
    assert_eq!(r0["quotas"], "140.12345678");
    assert_eq!(r0["quota_price"], "1.2845814979");
    // A payment before the due date counts as the date.
    assert_eq!(p["rows"][4]["paid_date"], "2022-08-12");
    assert_eq!(p["rows"][1]["status"], "info");
    let t = &p["totals"];
    assert_eq!(
        (
            t["participant"].as_i64(),
            t["employer"].as_i64(),
            t["risk"].as_i64()
        ),
        (Some(108000), Some(21600), Some(2500))
    );
    assert_eq!(t["quota_value"], "1.63284017");
    assert_eq!(t["quota_date"], "2026-10-14");
    assert_eq!(
        (t["balance"].as_i64(), t["computed_balance"].as_i64()),
        (Some(155384), Some(155384))
    );
    assert_eq!(p["warnings"], json!([]));
    assert!(p["asset_id"].is_null());

    // Saving asks for the account; with it, creates the plan (pension fund) and the quotas.
    assert!(
        api.call("pension.statement_import", json!({"text": QUANTA}))
            .is_err()
    );
    let r = api
        .call(
            "pension.statement_import",
            json!({"text": QUANTA, "account_id": acc, "risk_account_id": acc}),
        )
        .unwrap();
    assert_eq!(r["created_asset"], true);
    assert_eq!(
        (r["contributions"].as_u64(), r["risk_expenses"].as_u64()),
        (Some(3), Some(2))
    );
    let id = r["asset_id"].as_str().unwrap().to_string();
    let v = api.call("pension.get", json!({"asset_id": id})).unwrap();
    assert_eq!(v["plan"]["plan_type"], "efpc");
    assert_eq!(v["plan"]["start_date"], "2022-03-14");
    assert_eq!(v["plan"]["admin_fee_pct"], "0.25");
    assert_eq!(v["contributed"], 129600);
    assert_eq!(v["value"], 155384);

    // Again: the same plan (by CNPJ) and nothing duplicated.
    let again = api
        .call(
            "pension.statement_preview",
            json!({"text": QUANTA, "risk_account_id": acc}),
        )
        .unwrap();
    assert_eq!(again["asset_id"], id.as_str());
    assert!(
        again["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["kind"] == "participant" || r["kind"] == "employer")
            .all(|r| r["status"] == "duplicate")
    );
    let r = api
        .call(
            "pension.statement_import",
            json!({"text": QUANTA, "account_id": acc, "risk_account_id": acc}),
        )
        .unwrap();
    assert_eq!(
        (r["contributions"].as_u64(), r["risk_expenses"].as_u64()),
        (Some(0), Some(0))
    );
    // Text without a table: a clear error.
    assert!(
        api.call("pension.statement_preview", json!({"text": "nada"}))
            .is_err()
    );
}

/// How the app's PDF reader delivers the Quanta statement: numbers stuck together and reversed.
#[test]
fn reads_the_statement_as_the_pdf_reader_delivers_it() {
    let (h, rows, t) =
        super::statement::parse(include_str!("fixtures/quanta_pdf_extract.txt")).unwrap();
    assert_eq!(h.plan_name.as_deref(), Some("PLANOPREV-Arrojado"));
    assert_eq!(h.cnpb.as_deref(), Some("2010.0042-19"));
    assert_eq!(h.plan_cnpj.as_deref(), Some("11.444.777/0001-61"));
    assert_eq!(
        (h.admin_fee_pct.as_deref(), h.loading_fee_pct.as_deref()),
        (Some("0.25"), Some("0.00"))
    );
    assert_eq!(h.adhesion.as_deref(), Some("2022-03-14"));
    let kinds: Vec<_> = rows.iter().map(|r| r.kind).collect();
    use super::statement::PensionRowKind as K;
    assert_eq!(
        kinds,
        [
            K::Participant,
            K::RiskDisability,
            K::RiskDeath,
            K::Employer,
            K::Participant
        ]
    );
    let last = &rows[4];
    assert_eq!(
        (
            last.competence.as_str(),
            last.due_date.as_str(),
            last.paid_date.as_str()
        ),
        ("2022-08", "2022-08-16", "2022-08-12")
    );
    assert_eq!((last.retirement, last.current_value), (90000, 106296));
    assert_eq!(rows[2].risk, 1100);
    assert_eq!((t.participant, t.employer, t.risk), (108000, 21600, 2500));
    assert_eq!(t.quota_balance.as_deref(), Some("951.615432"));
    assert_eq!(
        (t.balance, t.computed_balance),
        (Some(155384), Some(155384))
    );
    assert_eq!(t.quota_date.as_deref(), Some("2026-10-14"));
}

/// Cost of the plan's fees over 10/20/30 years (portfolio.fee_cost), against 0.5% per year.
#[test]
fn fee_cost_projects_plan_fees() {
    let (_d, api, plan) = setup();
    api.call(
        "pension.set_plan",
        json!({"asset_id": plan, "plan_type": "vgbl", "tax_regime": "regressive",
               "admin_fee_pct": "2", "loading_fee_pct": "1"}),
    )
    .unwrap();
    let v = api
        .call(
            "portfolio.fee_cost",
            json!({"monthly_contribution": 100_000}),
        )
        .unwrap();
    let h = &v["holdings"].as_array().unwrap()[0];
    assert_eq!(h["asset"]["id"], plan);
    assert_eq!(h["admin_fee"], 0.02);
    assert_eq!(h["loading_fee"], 0.01);
    let p30 = &h["projections"][2];
    assert_eq!(p30["years"], 30);
    assert!(p30["cost"].as_i64().unwrap() > 0);
    assert!(p30["savings"].as_i64().unwrap() > 0);
    assert!(p30["current"].as_i64().unwrap() < p30["alternative"].as_i64().unwrap());
    assert_eq!(v["totals"][2]["cost"], p30["cost"]);
}

#[test]
fn agent_preview_is_compact_unless_rows_are_asked_and_attachments_resolve_in_any_engine() {
    use std::sync::Arc;

    use crate::mcp::inproc::{AGENT_ACCESS, attach_turn_files, call_tool_in};
    let (_d, api, _) = setup();
    let actor = crate::api::Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let call = |args: Value| {
        call_tool_in(
            &api,
            &actor,
            "preview_pension_statement",
            args,
            AGENT_ACCESS,
            None,
        )
    };
    let full = call(json!({"text": QUANTA, "rows": true}));
    let compact = call(json!({"text": QUANTA}));
    assert!(!full.is_error && !compact.is_error, "{}", compact.content);
    // Content: the summary on one line and the JSON (the full one may come cut at the ceiling).
    let v: Value = serde_json::from_str(compact.content.split_once('\n').unwrap().1).unwrap();
    assert_eq!(v["row_summary"]["count"], 5);
    assert!(
        v["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["status"] != "new")
    );
    assert!(v["totals"].is_object() && v["header"].is_object());
    assert!(compact.content.len() < full.content.len());
    assert!(
        compact.note.summary.contains("5 linha(s)"),
        "{}",
        compact.note.summary
    );

    // `attachments` becomes `files` through the job's conversation (app loop and CLIs through the
    // socket); an attached text (no native bytes) goes through the text.
    let att = Arc::new(
        crate::ai::attachments::prepare("extrato.txt", None, QUANTA.as_bytes().to_vec()).unwrap(),
    );
    api.ai_runtime().set_turn_files("conv-1", vec![att]);
    let input = attach_turn_files(
        &api,
        "conv-1",
        "preview_pension_statement",
        json!({"attachments": ["extrato.txt"]}),
    )
    .unwrap();
    assert!(input.get("attachments").is_none());
    assert!(
        !input["files"][0]["content_base64"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    // A name that is not in the conversation: a clear error, with the attachments that are.
    let e = attach_turn_files(
        &api,
        "conv-1",
        "preview_pension_statement",
        json!({"attachments": ["outro.pdf"]}),
    )
    .unwrap_err();
    assert!(
        e.contains("não encontrado nesta conversa") && e.contains("1) extrato.txt"),
        "{e}"
    );
    // The reference directly in `files` (what models try) and the only attachment with nothing
    // else.
    for raw in [
        json!({"files": ["1"]}),
        json!({"files": [{"filename": "extrato.txt"}]}),
        json!({}),
    ] {
        let input = attach_turn_files(&api, "conv-1", "preview_pension_statement", raw).unwrap();
        assert_eq!(input["files"][0]["filename"], "extrato.txt", "{input}");
    }
    let o = call(json!({"attachments": ["outro.pdf"]}));
    assert!(o.is_error, "{}", o.content);
}

#[test]
fn friendly_name_keeps_the_official_one_and_both_are_searched() {
    let (_d, api, plan) = setup();
    api.call(
        "assets.update",
        json!({"id": plan, "name": "Meu VGBL", "official_name": "BRASILPREV FIC PREV 100 RF"}),
    )
    .unwrap();
    let a = api.call("assets.get", json!({"id": plan})).unwrap();
    assert_eq!(a["name"], "Meu VGBL");
    assert_eq!(a["official_name"], "BRASILPREV FIC PREV 100 RF");
    for text in ["meu vgbl", "fic prev 100"] {
        let found = api.call("assets.list", json!({"text": text})).unwrap();
        assert_eq!(found.as_array().unwrap().len(), 1, "{text}");
    }
    // Equal to the displayed name is not stored; empty clears it.
    let id = api
        .call(
            "assets.create",
            json!({"name": "PGBL X", "official_name": " pgbl x ", "class": "pension"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let a = api.call("assets.get", json!({"id": id})).unwrap();
    assert!(a["official_name"].is_null());
    api.call("assets.update", json!({"id": plan, "official_name": null}))
        .unwrap();
    let a = api.call("assets.get", json!({"id": plan})).unwrap();
    assert!(a["official_name"].is_null());
}

#[test]
fn bulk_import_reuses_the_plan_by_official_name_or_cnpj() {
    let (_d, api, plan) = setup();
    api.call(
        "assets.update",
        json!({"id": plan, "official_name": "BRASILPREV FIC PREV 100 RF", "cnpj": "11.444.777/0001-61"}),
    )
    .unwrap();
    let acc = api.call("accounts.list", json!({})).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    // The fund name on the page and, in another batch, only the CNPJ: the registered plan is the
    // same.
    for (name, date) in [
        ("BRASILPREV FIC PREV 100 RF", "2026-02-10"),
        ("Fundo da página", "2026-03-10"),
    ] {
        let r = api
            .call(
                "portfolio.bulk",
                json!({"source": "teste", "account_id": acc,
                       "assets": [{"name": name, "class": "pension", "cnpj": "11444777000161"}],
                       "trades": [{"asset": name, "date": date, "side": "buy",
                                   "quantity": "100", "price": "1"}]}),
            )
            .unwrap();
        assert!(
            r["report"]["assets_created"]
                .as_array()
                .is_some_and(Vec::is_empty),
            "{r}"
        );
    }
    let pension = api
        .call("assets.list", json!({"classes": ["pension"]}))
        .unwrap();
    assert_eq!(pension.as_array().unwrap().len(), 1);
    let trades = api
        .call("portfolio.list_trades", json!({"asset_id": plan}))
        .unwrap();
    assert_eq!(trades.as_array().unwrap().len(), 4);
}

#[test]
fn payment_history_becomes_units_at_the_cvm_price_of_the_day() {
    let (_d, api, _) = setup();
    let plan = api
        .call(
            "assets.create",
            json!({"name": "Caixa VGBL", "class": "pension", "cnpj": "11.444.777/0001-61",
                   "pension": {"plan_type": "vgbl", "tax_regime": "progressive",
                               "loading_fee_pct": "1"}}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // CVM quota on Monday (payment on Saturday) and on the day of the second payment.
    for (date, close) in [("2026-03-02", "1.5"), ("2026-04-01", "2")] {
        api.call(
            "market.set_price",
            json!({"asset_id": plan, "date": date, "close": close}),
        )
        .unwrap();
    }
    let payments = json!([
        {"date": "2026-02-28", "amount": 30000},
        {"date": "2026-04-01", "amount": 30000},
        {"date": "2025-01-10", "amount": 10000},
    ]);
    let p = api
        .call(
            "pension.payments_preview",
            json!({"asset_id": plan, "payments": payments}),
        )
        .unwrap();
    assert_eq!(
        (
            p["new_count"].as_u64(),
            p["with_quote_count"].as_u64(),
            p["no_quote_count"].as_u64()
        ),
        (Some(3), Some(2), Some(1))
    );
    // All three come in (the one with no quota with the pending quantity).
    assert_eq!(p["new_total"], 70000);
    let first = &p["lines"][0];
    assert_eq!(first["quote_date"], "2026-03-02");
    // R$ 300 − 1% loading = R$ 297 ÷ 1.5 = 198 quotas.
    assert_eq!(
        (first["loading_fee"].as_i64(), first["quantity"].as_str()),
        (Some(300), Some("198"))
    );
    assert!(p["hint"].as_str().unwrap().contains("2025-01-10"), "{p}");
    // The plan has no quota account yet: the import creates one along with it.
    assert_eq!(p["account_id"], Value::Null);
    assert_eq!(p["account_to_create"], "Cotas · Caixa VGBL");

    let r = api
        .call(
            "pension.payments_import",
            json!({"asset_id": plan, "payments": payments}),
        )
        .unwrap();
    assert_eq!(r["trades_created"], 2);
    assert_eq!(r["pending_saved"], 1);
    assert_eq!(r["start_date_set"], "2025-01-10");
    let acc = r["account_created"].as_str().unwrap().to_string();
    let plan_view = api.call("pension.get", json!({"asset_id": plan})).unwrap();
    assert_eq!(plan_view["plan"]["start_date"], "2025-01-10");
    assert_eq!(plan_view["accounts"][0]["id"], acc.as_str());
    assert_eq!(plan_view["pending_payments"], 1);
    assert_eq!(plan_view["pending_amount"], 10000);
    let trades = api
        .call("portfolio.list_trades", json!({"asset_id": plan}))
        .unwrap();
    assert_eq!(trades.as_array().unwrap().len(), 2);
    // Repeating does not duplicate even the pending contribution; the account is the same.
    let again = api
        .call(
            "pension.payments_preview",
            json!({"asset_id": plan, "payments": payments}),
        )
        .unwrap();
    assert_eq!(again["duplicate_count"], 3);
    assert_eq!(again["account_id"], acc.as_str());
    assert!(
        api.call(
            "pension.payments_import",
            json!({"asset_id": plan, "payments": payments}),
        )
        .is_err()
    );
    // The collection brings the 2025 quota: the pending contribution becomes a purchase, without
    // duplicating.
    api.call(
        "market.set_price",
        json!({"asset_id": plan, "date": "2025-01-13", "close": "1"}),
    )
    .unwrap();
    let n = api
        .with_session(|s, _| super::payments::fill_pending(s.db.conn_mut()))
        .unwrap();
    assert_eq!(n, 1);
    let trades = api
        .call("portfolio.list_trades", json!({"asset_id": plan}))
        .unwrap();
    assert_eq!(trades.as_array().unwrap().len(), 3);
    let plan_view = api.call("pension.get", json!({"asset_id": plan})).unwrap();
    assert_eq!(plan_view["pending_payments"], 0);
    assert_eq!(
        api.with_session(|s, _| super::payments::fill_pending(s.db.conn_mut()))
            .unwrap(),
        0
    );
    // The collection fetches the plan's quotas since joining.
    let tasks = api
        .with_session(|s, _| crate::market::sync::plan(s.db.conn(), d("2026-10-23")))
        .unwrap();
    let task = tasks.iter().find(|t| t.asset.id == plan).unwrap();
    assert!(task.from <= d("2025-01-10"), "{}", task.from);
    // Outside pension, no.
    let stock = api
        .call(
            "assets.create",
            json!({"name": "Ação", "class": "stock_br", "ticker": "ABCD3"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        api.call(
            "pension.payments_preview",
            json!({"asset_id": stock, "payments": payments}),
        )
        .is_err()
    );
}

#[test]
fn a_plan_without_contributions_is_still_collected() {
    let (_d, api, _) = setup();
    let plan = api
        .call(
            "assets.create",
            json!({"name": "PGBL sem aporte", "class": "pension", "cnpj": "11.444.777/0001-61"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let tasks = api
        .with_session(|s, _| crate::market::sync::plan(s.db.conn(), d("2026-10-23")))
        .unwrap();
    assert!(tasks.iter().any(|t| t.asset.id == plan));
}

/// Text from made-up screenshots of the payment history (amount, status and date), as the OCR
/// returns it: three contributions on the same day, one pending, and the second screenshot repeats
/// the end of the first.
const PRINT_1: &str = "[Text read from the image by on-device OCR (91% average confidence).]
R$ 40,00
Pago
10/03/2022
R$ 120,00
Pendente
10/03/2022
R$ 80,00
Pago
10/02/2022
R$ 40,00
Pago
10/02/2022";
const PRINT_2: &str = "R$ 80,00
Pago
10/02/2022
R$ 40,00
Pago
10/02/2022
R$ 120,00
Pago
10/02/2022
R$ 1.200,00
Pago
10/01/2022";
#[test]
fn payment_screenshots_are_read_without_repeating_the_overlap() {
    use super::payments::{IssueKind, parse_print, read_prints};
    let (one, pending) = parse_print(PRINT_1);
    assert_eq!(pending, 1);
    assert_eq!(
        one.iter()
            .map(|p| (p.date.as_str(), p.amount))
            .collect::<Vec<_>>(),
        [
            ("2022-03-10", 4_000),
            ("2022-02-10", 8_000),
            ("2022-02-10", 4_000)
        ]
    );
    let r = read_prints(&[PRINT_1.into(), PRINT_2.into()], &[]);
    assert_eq!((r.not_paid, r.overlap), (1, 2));
    assert_eq!(
        r.payments
            .iter()
            .map(|p| (p.date.as_str(), p.amount))
            .collect::<Vec<_>>(),
        [
            ("2022-03-10", 4_000),
            ("2022-02-10", 8_000),
            ("2022-02-10", 4_000),
            ("2022-02-10", 12_000),
            ("2022-01-10", 120_000),
        ]
    );
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    // The date before the amount also counts.
    let (p, _) = parse_print("10/05/2022 R$ 40,00 Pago\n10/04/2022 R$ 80,00 Pago");
    assert_eq!(
        p.iter()
            .map(|p| (p.date.as_str(), p.amount))
            .collect::<Vec<_>>(),
        [("2022-05-10", 4_000), ("2022-04-10", 8_000)]
    );

    // Unreadable amount in the middle, the line cut between two screenshots and a missing month.
    let a = "R$ 40,00\nPago\n10/06/2022\nR$ 4O,0O\nPago\n10/05/2022\nR$ 40,00\nPago\n10/04/2022\nR$ 40,00";
    let b = "Pago\n10/03/2022\nR$ 40,00\nPago\n10/01/2022\nR$ 40,00\nPago\n10/12/2021";
    let r = read_prints(
        &[a.into(), b.into()],
        &["print-a.png".into(), "print-b.png".into()],
    );
    // The cut line (amount at the end of one, date at the start of the other) is assembled.
    assert_eq!(
        r.payments
            .iter()
            .map(|p| (p.date.as_str(), p.amount))
            .collect::<Vec<_>>(),
        [
            ("2022-06-10", 4_000),
            ("2022-04-10", 4_000),
            ("2022-03-10", 4_000),
            ("2022-01-10", 4_000),
            ("2021-12-10", 4_000),
        ]
    );
    let kinds: Vec<_> = r
        .issues
        .iter()
        .map(|i| (i.kind, i.date.clone(), i.source.clone()))
        .collect();
    assert!(
        kinds.contains(&(
            IssueKind::UnreadableAmount,
            Some("2022-05-10".into()),
            Some("print-a.png".into())
        )),
        "{kinds:?}"
    );
    // May has the unreadable line (already a pending item); February is missing for good.
    assert!(
        kinds.contains(&(IssueKind::MissingMonth, Some("2022-02".into()), None)),
        "{kinds:?}"
    );
    assert!(r.issues.iter().all(|i| !i.message.is_empty()));
    // Cut at the end of the last screenshot: pending, does not disappear.
    let r = read_prints(
        &["R$ 40,00\nPago\n10/06/2022\nR$ 50,00".into()],
        &["p.png".into()],
    );
    assert_eq!(r.payments.len(), 1);
    assert_eq!(r.issues[0].kind, IssueKind::CutBottom);
    assert_eq!(r.issues[0].amount, Some(5_000));
}

#[test]
fn payment_screenshots_attached_become_the_preview() {
    use std::sync::Arc;

    use crate::mcp::inproc::attach_turn_files;
    let (_d, api, _) = setup();
    let plan = api
        .call(
            "assets.create",
            json!({"name": "VGBL Teste", "class": "pension", "cnpj": "11.444.777/0001-61"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    for date in ["2022-01-10", "2022-02-10", "2022-03-10"] {
        api.call(
            "market.set_price",
            json!({"asset_id": plan, "date": date, "close": "2"}),
        )
        .unwrap();
    }
    let prep = |name: &str, t: &str| {
        Arc::new(crate::ai::attachments::prepare(name, None, t.as_bytes().to_vec()).unwrap())
    };
    // The screenshots in two messages: "1" and "2" in conversation order, besides the last turn.
    api.ai_runtime()
        .set_turn_files("conv-prints", vec![prep("print1.txt", PRINT_1)]);
    api.ai_runtime()
        .set_turn_files("conv-prints", vec![prep("print2.txt", PRINT_2)]);
    let input = attach_turn_files(
        &api,
        "conv-prints",
        "preview_pension_payments",
        json!({"asset_id": plan, "attachments": ["1", "2"]}),
    )
    .unwrap();
    assert_eq!(input["texts"].as_array().unwrap().len(), 2, "{input}");
    assert_eq!(input["sources"][0], "print1.txt", "{input}");
    assert!(input.get("files").is_none());
    let p = api.call("pension.payments_preview", input.clone()).unwrap();
    assert_eq!(p["read_from_prints"], 5);
    assert_eq!(p["overlap_removed"], 2);
    assert_eq!(p["skipped_not_paid"], 1);
    assert_eq!(p["new_count"], 5);
    assert_eq!(p["new_total"], 148_000);
    assert_eq!(p["lines"][0]["source"], "print1.txt");
    // Through the MCP tool: the proposal says how many contributions come in (the account goes
    // along).
    let agent = crate::api::Actor::Mcp {
        client: "teste".into(),
        read_only: false,
    };
    let o = crate::mcp::inproc::call_tool(
        &api,
        &agent,
        "import_pension_payments",
        json!({"asset_id": plan, "texts": [PRINT_1, PRINT_2]}),
        crate::mcp::inproc::AGENT_ACCESS,
    );
    assert!(!o.is_error, "{}", o.content);
    assert!(o.content.contains("5 aporte(s) com cota"), "{}", o.content);
    // With the payments already in the call, the only attachment does not come in on top.
    api.ai_runtime()
        .set_turn_files("conv-um", vec![prep("print.txt", PRINT_2)]);
    let input = attach_turn_files(
        &api,
        "conv-um",
        "preview_pension_payments",
        json!({"asset_id": plan, "payments": [{"date": "2022-01-10", "amount": 100}]}),
    )
    .unwrap();
    assert!(input.get("texts").is_none(), "{input}");
}

#[test]
fn a_payment_left_between_two_screenshots_and_the_month_after_joining() {
    use super::payments::{IssueKind, read_prints};
    // Three contributions per day; March 10 had one in each screenshot and the third in the gap.
    // The
    // joining on Nov 28 and the first debit only in January: December empty on purpose.
    let a = "R$ 50,00\nPago\n10/04/2022\nR$ 100,00\nPago\n10/04/2022\nR$ 150,00\nPago\n10/04/2022\nR$ 50,00\nPago\n10/03/2022";
    let b = "R$ 150,00\nPago\n10/03/2022\nR$ 50,00\nPago\n10/02/2022\nR$ 100,00\nPago\n10/02/2022\nR$ 150,00\nPago\n10/02/2022\nR$ 50,00\nPago\n10/01/2022\nR$ 100,00\nPago\n10/01/2022\nR$ 150,00\nPago\n10/01/2022\nR$ 300,00\nPago\n28/11/2021";
    let r = read_prints(&[a.into(), b.into()], &["p1.png".into(), "p2.png".into()]);
    assert_eq!(r.payments.len(), 12);
    let kinds: Vec<_> = r.issues.iter().map(|i| (i.kind, i.date.clone())).collect();
    assert_eq!(
        kinds,
        [(IssueKind::GapBetweenPrints, Some("2022-03-10".into()))],
        "{:?}",
        r.issues
    );
    assert!(r.issues[0].source.as_deref() == Some("p1.png / p2.png"));
}

#[test]
fn contributions_without_a_unit_price_show_in_the_portfolio_and_become_an_alert() {
    let (_d, api, _) = setup();
    let plan = api
        .call(
            "assets.create",
            json!({"name": "VGBL Sem Cota", "class": "pension", "cnpj": "11.444.777/0001-61"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let before = api.call("reports.net_worth", json!({})).unwrap()["investments"]
        .as_i64()
        .unwrap();
    // No quota at all; an unreadable amount in March.
    let print = "R$ 100,00\nPago\n10/04/2022\nR$ 1OO,OO\nPago\n10/03/2022\nR$ 100,00\nPago\n10/02/2022\nR$ 100,00\nPago\n10/01/2022";
    let r = api
        .call(
            "pension.payments_import",
            json!({"asset_id": plan, "texts": [print], "sources": ["print.png"]}),
        )
        .unwrap();
    assert_eq!(
        (r["trades_created"].as_u64(), r["pending_saved"].as_u64()),
        (Some(0), Some(3))
    );
    // Portfolio and position: the plan shows by the amount contributed.
    let s = api.call("portfolio.summary", json!({})).unwrap();
    let pos = s["positions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["asset"]["id"] == plan.as_str())
        .unwrap();
    assert_eq!(pos["market_value_base"], 30_000, "{pos}");
    assert!(pos["warnings"].to_string().contains("sem cota"), "{pos}");
    assert!(s["by_class"].to_string().contains("pension"));
    let p = api
        .call("portfolio.position", json!({"asset_id": plan}))
        .unwrap();
    assert_eq!(p["cost_base"], 30_000);
    let nw = api.call("reports.net_worth", json!({})).unwrap();
    assert_eq!(nw["investments"].as_i64(), Some(before + 30_000), "{nw}");
    // Plan screen: the contributions, the pending item and the amount contributed.
    let v = api.call("pension.get", json!({"asset_id": plan})).unwrap();
    assert_eq!(v["payments"].as_array().unwrap().len(), 3);
    assert_eq!(v["payments"][0]["status"], "pending_quote");
    assert_eq!(v["contributed"], 30_000);
    assert_eq!(v["payment_issues"][0]["kind"], "unreadable_amount");
    assert_eq!(v["payment_issues"][0]["source"], "print.png");
    let l = api
        .call("pension.payments_list", json!({"asset_id": plan}))
        .unwrap();
    assert_eq!(l["issues"].as_array().unwrap().len(), 1);
    assert_eq!(
        (l["total"].as_u64(), l["pending_quote"].as_u64()),
        (Some(3), Some(3))
    );
    // Filter by date and page: the total is the filtered series'.
    let l = api
        .call(
            "pension.payments_list",
            json!({"asset_id": plan, "date_from": "2022-02-01", "limit": 1}),
        )
        .unwrap();
    assert_eq!(l["total"], 2);
    assert_eq!(l["payments"].as_array().unwrap().len(), 1);
    assert_eq!(l["first_date"], "2022-02-10");
    // The plan's alert, with the pending item.
    let ins = api.call("reports.insights", json!({})).unwrap();
    let it = ins["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "pension_payments")
        .unwrap_or_else(|| panic!("{ins}"));
    assert!(it["body"].as_str().unwrap().contains("10/03/2022"), "{it}");
    // With March's payment reported, the pending item goes away.
    api.call(
        "pension.payments_import",
        json!({"asset_id": plan, "payments": [{"date": "2022-03-10", "amount": 10_000}]}),
    )
    .unwrap();
    let v = api.call("pension.get", json!({"asset_id": plan})).unwrap();
    assert_eq!(v["payment_issues"].as_array().unwrap().len(), 0, "{v}");
    assert_eq!(v["payments"].as_array().unwrap().len(), 4);
}

#[test]
fn the_plan_tax_regime_changes_with_the_asset_update() {
    let (_d, api, _) = setup();
    let plan = api
        .call(
            "assets.create",
            json!({"name": "VGBL Regime", "class": "pension",
                   "pension": {"plan_type": "vgbl", "tax_regime": "regressive", "insurer": "Seguradora Teste",
                               "loading_fee_pct": "1"}}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "assets.update",
        json!({"id": plan, "pension": {"tax_regime": "progressive"}}),
    )
    .unwrap();
    let v = api.call("pension.get", json!({"asset_id": plan})).unwrap();
    assert_eq!(v["plan"]["tax_regime"], "progressive");
    // The rest of the plan stays as it was.
    assert_eq!(v["plan"]["plan_type"], "vgbl");
    assert_eq!(v["plan"]["insurer"], "Seguradora Teste");
    assert_eq!(v["plan"]["loading_fee_pct"], "1");
}

/// Pension with no public quota (closed): with no balance, it asks for the balance instead of
/// entering the
/// ones valued at cost; the reported balance becomes the day's quota and the position's value; an
/// old one
/// (more than 90 days) asks again.
#[test]
fn closed_plan_is_valued_by_the_reported_balance() {
    let (_d, api, _) = setup();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Cooperativa", "kind": "other"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let plan = api
        .call(
            "assets.create",
            json!({"name": "Plano Fechado Exemplo", "class": "pension",
                   "pension": {"plan_type": "efpc", "tax_regime": "progressive"}}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "portfolio.create_trade",
        json!({"asset_id": plan, "account_id": acc, "date": "2025-01-10", "side": "buy",
               "quantity": "1000", "price": "1", "cash_movement": false}),
    )
    .unwrap();
    let at = "2026-10-20";
    let summary = || {
        api.call(
            "portfolio.summary",
            json!({"as_of": at, "deflator": "none"}),
        )
        .unwrap()
    };
    let due = |s: &Value| -> Vec<(String, Option<String>)> {
        s["balance_due"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                (
                    b["name"].as_str().unwrap().to_string(),
                    b["as_of"].as_str().map(str::to_string),
                )
            })
            .collect()
    };
    let s = summary();
    assert_eq!(due(&s), [("Plano Fechado Exemplo".to_string(), None)]);
    assert!(
        !s["missing_prices"].to_string().contains("Plano Fechado"),
        "{}",
        s["missing_prices"]
    );
    // Old balance: it counts, but asks again.
    api.call(
        "pension.set_balance",
        json!({"asset_id": plan, "date": "2026-05-01", "balance": 110_000}),
    )
    .unwrap();
    assert_eq!(
        due(&summary()),
        [(
            "Plano Fechado Exemplo".to_string(),
            Some("2026-05-01".to_string())
        )]
    );
    // Recent balance: the position is worth the balance (earnings = balance − contributions).
    let r = api
        .call(
            "pension.set_balance",
            json!({"asset_id": plan, "date": "2026-10-15", "balance": 123_456}),
        )
        .unwrap();
    assert_eq!(r["price"], "1.23456");
    let s = summary();
    assert!(due(&s).is_empty(), "{}", s["balance_due"]);
    let pos = s["positions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["asset"]["id"] == plan.as_str())
        .unwrap();
    assert_eq!(pos["market_value_base"], 123_456, "{pos}");
    assert_eq!(pos["unrealized_base"], 23_456);
    // Before the first contribution there are no quotas; and only pension accepts a balance.
    assert!(
        api.call(
            "pension.set_balance",
            json!({"asset_id": plan, "date": "2024-01-01", "balance": 1}),
        )
        .is_err()
    );
}
