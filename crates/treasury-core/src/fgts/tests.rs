//! FGTS tests with a synthetic statement (`fixtures/extrato_sintetico.txt`): employer,
//! CNPJ, dates, amounts and entries made up, in the Caixa layout.

use serde_json::json;
use sha2::{Digest, Sha256};

use super::statement::{expected_jam, parse};
use super::{EntryKind, data};
use crate::api::Api;
use crate::paths::TreasuryPaths;

const FIXTURE: &str = include_str!("fixtures/extrato_sintetico.txt");

fn api() -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let member = api.call("household.list_members", json!({})).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, member)
}

#[test]
fn reads_the_header_and_every_line_of_the_statement() {
    let (h, lines) = parse(FIXTURE).unwrap();
    assert_eq!(
        h.employer_name.as_deref(),
        Some("EMPRESA EXEMPLO DE TESTES S.A.")
    );
    assert_eq!(h.employer_cnpj.as_deref(), Some("98.765.432/0001-10"));
    assert_eq!(h.admission_date.as_deref(), Some("2023-02-01"));
    assert_eq!(h.option_date.as_deref(), Some("2023-02-01"));
    assert_eq!(h.leave_date, None);
    assert_eq!(h.interest_rate.as_deref(), Some("3"));
    assert_eq!(h.rescission_base, Some(876_543));
    assert_eq!(h.category.as_deref(), Some("101"));
    assert_eq!(h.statement_date.as_deref(), Some("2024-10-01"));
    assert!(h.has_account_number && h.account_hash.is_some());
    // The worker's name and PIS are not read.
    assert!(!format!("{h:?}").contains("FICTICIA"));
    assert_eq!(lines.len(), 47);
    assert_eq!(lines[0].kind, EntryKind::Deposit);
    assert_eq!(lines[0].competence.as_deref(), Some("2023-02"));
    assert_eq!(lines[1].kind, EntryKind::Jam);
    assert_eq!(lines[1].coefficient.as_deref(), Some("0.002466"));
    // Printed order ≠ date order: the JAM of the 10th comes before the deposit of the 6th.
    assert!(lines[1].date > lines[2].date);
    let extra = lines
        .iter()
        .find(|l| l.kind == EntryKind::DepositExtra)
        .unwrap();
    assert_eq!(extra.competence.as_deref(), Some("2023-06"));
    // Distribution on two lines, with the base year after the amounts.
    let dist = lines
        .iter()
        .find(|l| l.kind == EntryKind::Distribution)
        .unwrap();
    assert_eq!(dist.base_year, Some(2023));
    assert_eq!(dist.amount, 7_504);
    assert!(dist.history.contains("DIST RESULTADO"));
    let w: Vec<_> = lines
        .iter()
        .filter(|l| l.kind == EntryKind::Withdrawal)
        .collect();
    assert_eq!(w.len(), 4);
    assert_eq!(w[2].withdrawal_code.as_deref(), Some("99"));
    assert_eq!(w[2].withdrawal_ref.as_deref(), Some("123456"));
    assert_eq!(w[2].group, w[3].group);
    assert!(lines.iter().any(|l| l.kind == EntryKind::Reversal));
    assert_eq!(
        lines
            .iter()
            .filter(|l| l.kind == EntryKind::JamAdjustment)
            .count(),
        2
    );
    // Two credits in the month of the change (April 10 and 21, 2024).
    assert!(
        lines
            .iter()
            .any(|l| l.kind == EntryKind::Jam && l.date == "2024-04-10")
    );
    assert!(
        lines
            .iter()
            .any(|l| l.kind == EntryKind::Jam && l.date == "2024-04-21")
    );
}

#[test]
fn every_jam_credit_is_recomputed_to_the_cent() {
    let (_, lines) = parse(FIXTURE).unwrap();
    let opening = lines[0].total - lines[0].amount;
    let exp = expected_jam(&lines, opening);
    let mut n = 0;
    for (l, e) in lines.iter().zip(&exp) {
        if let Some(e) = e {
            assert_eq!(*e, l.amount, "JAM of {}", l.date);
            n += 1;
        }
    }
    // The first credit has no previous one for the base.
    assert_eq!(
        n,
        lines.iter().filter(|l| l.kind == EntryKind::Jam).count() - 1
    );
    // Examples from the design: truncates, does not round.
    assert_eq!((30_000f64 * 0.002634 + 1e-6).floor() as i64, 79);
    assert_eq!((1_000_000f64 * 0.004150 + 1e-6).floor() as i64, 4_150);
}

#[test]
fn preview_import_dedupe_check_and_withdrawals() {
    let (_d, api, member) = api();
    let p = api
        .call("fgts.statement_preview", json!({"text": FIXTURE}))
        .unwrap();
    assert_eq!(p["balance_ok"], true, "{}", p["warnings"]);
    assert_eq!(p["jam_mismatches"], 0);
    assert_eq!(p["account_id"], serde_json::Value::Null);
    assert_eq!(p["new_count"], 47);
    let groups = p["withdrawals"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["reason"], "emergencial");
    assert_eq!(groups[0]["amount"], 30_512);
    assert_eq!(groups[1]["reason"], "moradia");
    assert!(p["warnings"].to_string().contains("99"));

    let checking = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking", "opening_date": "2023-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let key = groups[0]["key"].as_str().unwrap().to_string();
    let r = api
        .call(
            "fgts.statement_import",
            json!({"text": FIXTURE, "member_id": member, "destinations": {key: checking}}),
        )
        .unwrap();
    assert_eq!(r["inserted"], 47);
    let id = r["account_id"].as_str().unwrap().to_string();
    let list = api.call("fgts.list", json!({})).unwrap();
    let acc = &list[0];
    assert_eq!(acc["balance"], 633_775);
    assert_eq!(acc["employer_name"], "EMPRESA EXEMPLO DE TESTES S.A.");
    assert_eq!(acc["member_id"], member.as_str());
    assert_eq!(acc["dismissal_fine"], 876_543 * 40 / 100);
    // The immediate withdrawal went to the checking account (transfer, outside the budget).
    let bal = api.call("accounts.get", json!({"id": checking})).unwrap()["balance"]
        .as_i64()
        .unwrap();
    assert_eq!(bal, 30_512);
    // Re-importing does not duplicate.
    let again = api
        .call("fgts.statement_import", json!({"text": FIXTURE}))
        .unwrap();
    assert_eq!(again["account_id"], id.as_str());
    assert_eq!(again["inserted"], 0);
    assert_eq!(again["skipped"], 47);
    // Check: JAM to the cent, deposits by competence.
    let c = api.call("fgts.check", json!({"account_id": id})).unwrap();
    assert_eq!(c["jam_mismatches"].as_array().unwrap().len(), 0);
    let deps = c["deposits"].as_array().unwrap();
    assert_eq!(deps[0]["competence"], "2023-02");
    assert_eq!(deps[0]["deposited"], 40_000);
    let june = deps.iter().find(|d| d["competence"] == "2023-06").unwrap();
    assert_eq!(june["deposited"], 41_234);
    // The entries stay out of the budget and of "uncategorized".
    let unc = api
        .call(
            "transactions.query",
            json!({"uncategorized": true, "account_ids": [id]}),
        )
        .unwrap();
    assert_eq!(unc["items"].as_array().map_or(0, Vec::len), 0, "{unc}");
    let e = api.call("fgts.entries", json!({"account_id": id})).unwrap();
    assert_eq!(e.as_array().unwrap().len(), 47);
    assert_eq!(e.as_array().unwrap().last().unwrap()["balance"], 633_775);

    // A new withdrawal with a destination becomes a transfer.
    api.call(
        "fgts.record_withdrawal",
        json!({"account_id": id, "date": "2024-10-05", "amount": 10_000, "code": "60",
               "destination_account_id": checking}),
    )
    .unwrap();
    let list = api.call("fgts.list", json!({})).unwrap();
    assert_eq!(list[0]["balance"], 623_775);
    assert!(
        api.call(
            "fgts.record_withdrawal",
            json!({"account_id": id, "date": "2024-10-05", "amount": 99_999_999}),
        )
        .is_err()
    );
}

#[test]
fn housing_use_goes_out_of_the_fgts_account_into_the_mortgage() {
    let (_d, api, member) = api();
    let r = api
        .call(
            "fgts.statement_import",
            json!({"text": FIXTURE, "member_id": member}),
        )
        .unwrap();
    let id = r["account_id"].as_str().unwrap().to_string();
    let l = api
        .call(
            "debt.create",
            json!({"name": "Apartamento", "kind": "mortgage", "principal": 30_000_000, "annual_rate": "9",
                   "system": "sac", "contract_date": "2024-01-01", "first_due_date": "2024-02-01", "term_months": 360}),
        )
        .unwrap();
    let before = api.call("debt.get", json!({"id": l["id"]})).unwrap()["balance"]
        .as_i64()
        .unwrap();
    let h = api
        .call(
            "fgts.use_for_housing",
            json!({"account_id": id, "loan_id": l["id"], "date": "2024-10-02", "amount": 200_000}),
        )
        .unwrap();
    // There was housing use (code 99) in 06/2024: the 2-year notice.
    assert_eq!(h["warnings"].as_array().unwrap().len(), 1, "{h}");
    let after = api.call("debt.get", json!({"id": l["id"]})).unwrap()["balance"]
        .as_i64()
        .unwrap();
    assert!(after < before);
    assert_eq!(
        api.call("fgts.list", json!({})).unwrap()[0]["balance"],
        633_775 - 200_000
    );
    // Real-estate contracts only.
    let car = api
        .call(
            "debt.create",
            json!({"name": "Carro", "kind": "vehicle", "principal": 1_000_000, "annual_rate": "20",
                   "system": "price", "contract_date": "2024-01-01", "first_due_date": "2024-02-01", "term_months": 12}),
        )
        .unwrap();
    assert!(
        api.call(
            "fgts.use_for_housing",
            json!({"account_id": id, "loan_id": car["id"], "date": "2024-10-02", "amount": 1_000}),
        )
        .is_err()
    );
}

#[test]
fn anniversary_modality_projection_and_comparison() {
    let (_d, api, member) = api();
    api.call(
        "household.update_member",
        json!({"id": member, "profile": {"birth_date": "1990-05-15"}}),
    )
    .unwrap();
    let r = api
        .call(
            "fgts.statement_import",
            json!({"text": FIXTURE, "member_id": member}),
        )
        .unwrap();
    let id = r["account_id"].as_str().unwrap().to_string();
    let a = api
        .call("fgts.anniversary_simulate", json!({"member_id": member}))
        .unwrap();
    // 6,337.75 in the 5,000.01 to 10,000 bracket: 20% + 650.
    assert_eq!(a["amount"], data::anniversary_amount(633_775).0);
    assert_eq!(a["amount"], 191_755);
    assert_eq!(a["window_start"].as_str().unwrap()[5..], *"05-01");
    let m = api
        .call(
            "fgts.set_modality",
            json!({"member_id": member, "modality": "aniversario", "since": "2025-05-01",
                   "return_requested_at": "2026-03-10"}),
        )
        .unwrap();
    assert_eq!(m["modality"], "aniversario");
    assert_eq!(m["earliest_return"], "2028-04-01");
    assert_eq!(m["blocked_on_dismissal"], 633_775);
    // Member that does not exist: "Não encontrado: membro x" (not "Membro não encontrado."
    // repeated).
    let e = api
        .call(
            "fgts.set_modality",
            json!({"member_id": "nao-existe", "modality": "rescisao"}),
        )
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("nao-existe") && !e.contains("encontrado."),
        "{e}"
    );

    // Projection to age 65 with 8% of a R$ 5,000 gross.
    let src = api
        .call(
            "income.create_source",
            json!({"member_id": member, "name": "Salário", "kind": "clt", "gross_monthly": 500_000}),
        )
        .unwrap();
    api.call(
        "fgts.update_account",
        json!({"account_id": id, "income_source_id": src["id"]}),
    )
    .unwrap();
    let p = api
        .call("fgts.project", json!({"member_id": member, "age": 65}))
        .unwrap();
    assert_eq!(p["date"], "2055-05-15");
    assert!(p["balance"].as_i64().unwrap() > p["balance_real"].as_i64().unwrap());
    assert!(p["deposits"].as_i64().unwrap() > 40_000 * 12 * 28);

    // Comparison with IPCA of 0.4% per month and CDI of 0.04% per day (synthetic).
    api.with_session(|s, _| {
        let c = s.db.conn();
        let mut d = chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        while d <= chrono::NaiveDate::from_ymd_opt(2024, 10, 1).unwrap() {
            c.execute(
                "INSERT INTO index_value (series_id, date, value, source, fetched_at)
                 VALUES ('ipca', ?1, '0.4', 'test', 'x')",
                [crate::time::fmt_date(d)],
            )?;
            d = d.checked_add_months(chrono::Months::new(1)).unwrap();
        }
        let mut d = chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        while d <= chrono::NaiveDate::from_ymd_opt(2024, 10, 1).unwrap() {
            c.execute(
                "INSERT INTO index_value (series_id, date, value, source, fetched_at)
                 VALUES ('cdi', ?1, '0.04', 'test', 'x')",
                [crate::time::fmt_date(d)],
            )?;
            d = d.succ_opt().unwrap();
        }
        Ok(())
    })
    .unwrap();
    let c = api
        .call(
            "fgts.yield_compare",
            json!({"account_id": id, "from": "2023-03-01", "to": "2024-10-01"}),
        )
        .unwrap();
    assert_eq!(c["final_balance"], 633_775);
    let irr = c["irr"].as_f64().unwrap();
    assert!((0.02..0.08).contains(&irr), "{irr}");
    let b = c["benchmarks"].as_array().unwrap();
    let ipca = b.iter().find(|x| x["id"] == "ipca").unwrap();
    assert!(ipca["final_value"].as_i64().is_some());
    let cdi = b.iter().find(|x| x["id"] == "cdi").unwrap();
    let cdi85 = b.iter().find(|x| x["id"] == "cdi_85").unwrap();
    assert!(cdi["final_value"].as_i64() > cdi85["final_value"].as_i64());
    assert_eq!(
        b.iter().find(|x| x["id"] == "m2").unwrap()["final_value"],
        serde_json::Value::Null
    );
    let y2023 = c["years"]
        .as_array()
        .unwrap()
        .iter()
        .find(|y| y["year"] == 2023)
        .unwrap();
    assert!(y2023["jam_distribution"].as_f64().unwrap() > y2023["jam"].as_f64().unwrap());
}

#[test]
fn fgts_accounts_only_in_a_brazilian_vault() {
    let (_d, api, _) = api();
    api.call("country.set", json!({"country": "US"})).unwrap();
    assert!(
        api.call("accounts.create", json!({"name": "FGTS", "kind": "fgts"}))
            .is_err()
    );
    assert!(api.call("fgts.list", json!({})).is_err());
}

#[test]
fn missing_deposit_alert_and_withdrawal_at_retirement() {
    let (_d, api, member) = api();
    // The same statement without the May 2024 deposit (deadline 2024-06-20, before the statement).
    let text: String = FIXTURE
        .lines()
        .filter(|l| !l.contains("115-DEPOSITO MAIO 2024"))
        .collect::<Vec<_>>()
        .join("\n");
    api.call(
        "fgts.statement_import",
        json!({"text": text, "member_id": member}),
    )
    .unwrap();
    let ins = api.call("reports.insights", json!({})).unwrap();
    let fgts: Vec<_> = ins["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "fgts_missing_deposit")
        .collect();
    assert_eq!(fgts.len(), 1, "{ins}");
    assert_eq!(fgts[0]["data"]["competences"][0], "2024-05");

    // Retirement plan: the FGTS balance enters as a withdrawal at retirement.
    let sim = api
        .call(
            "goals.retirement",
            json!({"current_age": 40, "retirement_age": 65, "current_wealth": 0,
                   "monthly_spending": 0}),
        )
        .unwrap();
    assert!(sim["fgts_at_retirement"].as_i64().unwrap() > 0, "{sim}");
    assert!(
        sim["plan"]["events"].is_null() || sim["plan"]["events"].as_array().unwrap().is_empty()
    );
    let off = api
        .call(
            "goals.retirement",
            json!({"current_age": 40, "retirement_age": 65, "current_wealth": 0,
                   "monthly_spending": 0, "include_fgts": false}),
        )
        .unwrap();
    assert!(off["fgts_at_retirement"].is_null());
    let p50 = |v: &serde_json::Value| {
        v["bands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["age"] == 66)
            .unwrap()["p50"]
            .as_i64()
    };
    assert!(p50(&off) < p50(&sim));
}

/// Another made-up layout: labels alone on the line, CNPJ without punctuation, the employment
/// dates and the category on a single line, and the distribution with the date, the description
/// and the amounts each on its own line.
const COLUMNS: &str = "CAIXA ECONOMICA FEDERAL
FGTS - EXTRATO DE CONTA VINCULADA
Emitido em 15/09/2025
EMPREGADOR
COMERCIO FICTICIO DE
PECAS LTDA
INSCRICAO DO EMPREGADOR
11222333000181
N DA CONTA
2222 0001112223334445
DATA DE ADMISSAO DATA DE OPCAO CATEGORIA
05/01/2015 05/01/2015 1
TAXA DE JUROS VALOR PARA FINS RESCISORIOS
3.0% R$ 2.000,00
DATA LANCAMENTOS VALOR TOTAL
06/02/2015 115-DEPOSITO JANEIRO 2015 R$ 500,00 R$ 500,00
10/03/2015 CREDITO DE JAM 0,002466 R$ 1,23 R$ 501,23
06/03/2015 115-DEPOSITO FEVEREIRO 2015 R$ 500,00 R$ 1.001,23
21/03/2015
AC CRED DIST RESULTADO ANO BASE
R$ 10,00 R$ 1.011,23
12/2014
10/04/2015 CREDITO DE JAM 0,002466 R$ 2,46 R$ 1.013,69
06/04/2015 115-DEPOSITO MARCO 2015 R$ 500,00 R$ 1.513,69
";

#[test]
fn reads_the_header_and_the_split_distribution_of_another_layout() {
    let (h, lines) = parse(COLUMNS).unwrap();
    assert_eq!(
        h.employer_name.as_deref(),
        Some("COMERCIO FICTICIO DE PECAS LTDA")
    );
    assert_eq!(h.employer_cnpj.as_deref(), Some("11.222.333/0001-81"));
    assert_eq!(h.admission_date.as_deref(), Some("2015-01-05"));
    assert_eq!(h.option_date.as_deref(), Some("2015-01-05"));
    assert_eq!(h.leave_date, None);
    // The category is not the piece of the option date.
    assert_eq!(h.category.as_deref(), Some("1"));
    assert!(h.has_account_number);
    assert_eq!(h.statement_date.as_deref(), Some("2025-09-15"));
    assert_eq!(h.rescission_base, Some(200_000));
    assert_eq!(lines.len(), 6);
    let dist = &lines[3];
    assert_eq!(dist.kind, EntryKind::Distribution);
    assert_eq!(dist.date, "2015-03-21");
    assert_eq!(dist.amount, 1_000);
    assert_eq!(dist.total, 101_123);
    assert_eq!(dist.base_year, Some(2014));
    assert_eq!(lines[4].kind, EntryKind::Jam);
    let opening = lines[0].total - lines[0].amount;
    let exp = expected_jam(&lines, opening);
    assert_eq!(exp[4], Some(246));
}

#[test]
fn split_distribution_enters_the_balance_and_the_jam_matches() {
    let (_d, api, _) = api();
    let p = api
        .call("fgts.statement_preview", json!({"text": COLUMNS}))
        .unwrap();
    assert_eq!(p["balance_ok"], true, "{}", p["warnings"]);
    assert_eq!(p["jam_mismatches"], 0);
    assert_eq!(p["new_count"], 6);
    assert!(!p["warnings"].to_string().contains("cabeçalho"));
    let r = api
        .call("fgts.statement_import", json!({"text": COLUMNS}))
        .unwrap();
    let id = r["account_id"].as_str().unwrap().to_string();
    let acc = api.call("fgts.list", json!({})).unwrap()[0].clone();
    assert_eq!(acc["balance"], 151_369);
    assert_eq!(acc["category"], "1");
    let e = api.call("fgts.entries", json!({"account_id": id})).unwrap();
    assert!(
        e.as_array()
            .unwrap()
            .iter()
            .any(|x| x["kind"] == "distribution" && x["amount"] == 1_000),
        "{e}"
    );
}

/// The statement without a legible account number or CNPJ.
fn without_ids() -> String {
    COLUMNS
        .lines()
        .filter(|l| !l.starts_with("11222333") && !l.starts_with("2222 "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn importing_the_same_statement_again_reuses_the_account() {
    let (_d, api, member) = api();
    let text = without_ids();
    let first = api
        .call("fgts.statement_import", json!({"text": text}))
        .unwrap();
    let id = first["account_id"].as_str().unwrap().to_string();
    assert_eq!(first["inserted"], 6);
    // The second time, with holder and name: the same account, which receives both.
    let p = api
        .call("fgts.statement_preview", json!({"text": text}))
        .unwrap();
    assert_eq!(p["account_id"], id.as_str());
    assert_eq!(p["existing_count"], 6);
    let again = api
        .call(
            "fgts.statement_import",
            json!({"text": text, "member_id": member, "name": "FGTS da Ana"}),
        )
        .unwrap();
    assert_eq!(again["account_id"], id.as_str());
    assert_eq!(again["inserted"], 0);
    let list = api.call("fgts.list", json!({})).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["member_id"], member.as_str());
    assert_eq!(list[0]["name"], "FGTS da Ana");
}

#[test]
fn text_with_the_pdf_completes_the_header_and_fields_override_it() {
    let (_d, api, _) = api();
    use base64::Engine as _;
    let file = base64::engine::general_purpose::STANDARD.encode(without_ids());
    let p = api
        .call(
            "fgts.statement_preview",
            json!({
                "files": [{"filename": "extrato.txt", "content_base64": file}],
                "text": "INSCRICAO DO EMPREGADOR 11.222.333/0001-81",
                "admission_date": "2015-01-02",
            }),
        )
        .unwrap();
    // The lines come from the file; the text only completes the header.
    assert_eq!(p["new_count"], 6);
    assert_eq!(p["header"]["employer_cnpj"], "11.222.333/0001-81");
    assert_eq!(p["header"]["admission_date"], "2015-01-02");
    assert_eq!(p["header"]["option_date"], "2015-01-05");
    // No CNPJ anywhere: the incomplete-header warning.
    let p = api
        .call("fgts.statement_preview", json!({"text": without_ids()}))
        .unwrap();
    assert!(
        p["warnings"].to_string().contains("cabeçalho"),
        "{}",
        p["warnings"]
    );
    // Invalid date in the given data: error.
    assert!(
        api.call(
            "fgts.statement_preview",
            json!({"text": COLUMNS, "admission_date": "2015-13-40"}),
        )
        .is_err()
    );
}

#[test]
fn archived_account_leaves_the_mcp_list_and_its_total() {
    let (_d, api, _) = api();
    let id = api
        .call("fgts.statement_import", json!({"text": COLUMNS}))
        .unwrap()["account_id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call("accounts.update", json!({"id": id, "archived": true}))
        .unwrap();
    // The interface sees all of them (with `archived`); the MCP, only the active ones, unless
    // asked.
    assert_eq!(
        api.call("fgts.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        api.call("fgts.list", json!({"active_only": true}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let agent = crate::api::Actor::Mcp {
        client: "teste".into(),
        read_only: false,
    };
    let o = crate::mcp::inproc::call_tool(
        &api,
        &agent,
        "list_fgts_accounts",
        json!({}),
        crate::mcp::inproc::AGENT_ACCESS,
    );
    assert!(!o.is_error, "{}", o.content);
    assert!(!o.content.contains("1.513,69"), "{}", o.content);
    let o = crate::mcp::inproc::call_tool(
        &api,
        &agent,
        "list_fgts_accounts",
        json!({"include_archived": true}),
        crate::mcp::inproc::AGENT_ACCESS,
    );
    assert!(o.content.contains("archived"), "{}", o.content);
}

#[test]
fn import_with_nothing_in_the_call_does_not_take_the_attachment() {
    let (_d, api, _) = api();
    let att = std::sync::Arc::new(
        crate::ai::attachments::prepare("extrato.txt", None, COLUMNS.as_bytes().to_vec()).unwrap(),
    );
    api.ai_runtime().set_turn_files("conv-fgts", vec![att]);
    use crate::mcp::inproc::attach_turn_files;
    // The preview uses the only attachment; the import asks for the named attachment.
    let input = attach_turn_files(&api, "conv-fgts", "preview_fgts_statement", json!({})).unwrap();
    assert_eq!(input["files"][0]["filename"], "extrato.txt");
    let e = attach_turn_files(&api, "conv-fgts", "import_fgts_statement", json!({})).unwrap_err();
    assert!(e.contains("1) extrato.txt"), "{e}");
    let input = attach_turn_files(
        &api,
        "conv-fgts",
        "import_fgts_statement",
        json!({"attachments": ["1"]}),
    )
    .unwrap();
    assert_eq!(input["files"][0]["filename"], "extrato.txt");
}

/// The "Histórico" issued by the Caixa app and website in pages (`fixtures/
/// extrato_historico_paginas.txt`, made up): the employer name breaks before the admission
/// date, the CNPJ comes without punctuation and without the leading zero, next to the work
/// permit and the account number, each page repeats the header and the distribution carries
/// the base year "12/AAAA" on the amounts line.
const PAGES: &str = include_str!("fixtures/extrato_historico_paginas.txt");

#[test]
fn reads_the_paged_history_with_the_cnpj_without_its_leading_zero() {
    let (h, lines) = parse(PAGES).unwrap();
    assert_eq!(
        h.employer_name.as_deref(),
        Some("COOPERATIVA FICTICIA DE CREDITO EXEMPLO")
    );
    assert_eq!(h.employer_cnpj.as_deref(), Some("07.654.321/0001-59"));
    assert_eq!(h.admission_date.as_deref(), Some("2020-02-03"));
    assert_eq!(h.option_date.as_deref(), Some("2020-02-03"));
    assert_eq!(h.leave_date, None);
    assert_eq!(h.category.as_deref(), Some("1"));
    assert_eq!(h.interest_rate.as_deref(), Some("3"));
    assert_eq!(h.rescission_base, Some(320_000));
    assert_eq!(h.statement_date.as_deref(), Some("2025-09-15"));
    // The account number, not the 13-digit CNPJ that comes before it.
    let hash = hex::encode(Sha256::digest(b"123456789012345678"));
    assert_eq!(h.account_hash.as_deref(), Some(hash.as_str()));
    assert_eq!(lines.len(), 17);
    let dist = lines
        .iter()
        .find(|l| l.kind == EntryKind::Distribution)
        .unwrap();
    assert_eq!(dist.date, "2020-07-10");
    assert_eq!(dist.amount, 1_500);
    assert_eq!(dist.base_year, Some(2019));
    assert_eq!(lines[5].kind, EntryKind::DepositExtra);
    assert_eq!(lines[8].withdrawal_code.as_deref(), Some("50"));
    assert_eq!(lines[14].group, lines[15].group);
    assert_eq!(lines.last().unwrap().total, 80_000);
    let exp = expected_jam(&lines, 0);
    assert_eq!(exp.iter().flatten().count(), 4);
    for (l, e) in lines.iter().zip(&exp) {
        if let Some(e) = e {
            assert_eq!(*e, l.amount, "{}", l.date);
        }
    }
}

#[test]
fn paged_history_preview_and_import() {
    let (_d, api, member) = api();
    let p = api
        .call("fgts.statement_preview", json!({"text": PAGES}))
        .unwrap();
    assert_eq!(p["balance_ok"], true, "{}", p["warnings"]);
    assert_eq!(p["jam_mismatches"], 0);
    assert_eq!(p["new_count"], 17);
    assert_eq!(p["opening_balance"], 0);
    assert_eq!(p["final_balance"], 80_000);
    let groups = p["withdrawals"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[1]["amount"], 455_563);
    assert_eq!(groups[1]["reason"], "moradia");
    let r = api
        .call(
            "fgts.statement_import",
            json!({"text": PAGES, "member_id": member}),
        )
        .unwrap();
    assert_eq!(r["inserted"], 17);
    let acc = api.call("fgts.list", json!({})).unwrap()[0].clone();
    assert_eq!(acc["balance"], 80_000);
    assert_eq!(acc["employer_cnpj"], "07.654.321/0001-59");
    // Again: the same account, nothing new.
    let again = api
        .call("fgts.statement_preview", json!({"text": PAGES}))
        .unwrap();
    assert_eq!(again["account_id"], r["account_id"]);
    assert_eq!(again["new_count"], 0);
}
