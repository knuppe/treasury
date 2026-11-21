//! Contas de aposentadoria com titulares, datas de nascimento e valores inventados.

use chrono::NaiveDate;
use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

fn api(country: &str) -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"], "country": country}),
    )
    .unwrap();
    let member = api.call("household.list_members", json!({})).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, member)
}

fn born(api: &Api, member: &str, date: &str) {
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE member SET birth_date = ?2 WHERE id = ?1",
            [member, date],
        )?;
        Ok(())
    })
    .unwrap();
}

fn account(api: &Api, member: &str, name: &str, plan: &str, coverage: Option<&str>) -> String {
    let mut input = json!({"name": name, "kind": "retirement", "opening_date": "2026-01-01",
                           "member_ids": [member], "retirement_plan": plan});
    if let Some(c) = coverage {
        input["retirement_coverage"] = json!(c);
    }
    api.call("accounts.create", input).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn deposit(api: &Api, account: &str, date: &str, amount: i64) {
    api.call(
        "transactions.create",
        json!({"account_id": account, "date": date, "amount": amount, "description": "Aporte",
               "kind": "income"}),
    )
    .unwrap();
}

fn status(api: &Api, account: &str, today: NaiveDate) -> super::Status {
    api.with_session(|s, _| super::status(s.db.conn(), account, today))
        .unwrap()
}

#[test]
fn us_limits_are_per_person_and_shared_by_group() {
    let (_d, api, ana) = api("US");
    born(&api, &ana, "1970-03-10");
    let k = account(&api, &ana, "401(k) Acme", "us_401k", None);
    let roth = account(&api, &ana, "Roth 401(k) Acme", "us_roth_401k", None);
    deposit(&api, &k, "2026-02-15", 1_000_000);
    deposit(&api, &roth, "2026-03-15", 500_000);
    // Do ano anterior não conta.
    deposit(&api, &k, "2025-12-15", 900_000);
    let today = NaiveDate::from_ymd_opt(2026, 10, 22).unwrap();
    let s = status(&api, &k, today);
    // 56 anos no fim de 2026: 24.500 + 8.000 de catch-up.
    assert_eq!(s.age, Some(56));
    assert_eq!(s.limit, Some(3_250_000));
    assert_eq!(s.contributed, 1_500_000);
    assert_eq!(s.remaining, Some(1_750_000));
    assert_eq!(s.shared_with, vec!["Roth 401(k) Acme".to_string()]);
    assert_eq!(s.limit_year, None);
    // HSA familiar, sem catch-up (menos de 55 anos de outra titular).
    let bia: String = api
        .call("household.create_member", json!({"name": "Bia"}))
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    born(&api, &bia, "1990-06-01");
    let hsa = account(&api, &bia, "HSA", "us_hsa", Some("family"));
    assert_eq!(status(&api, &hsa, today).limit, Some(875_000));
    // 529: sem teto anual.
    let edu = account(&api, &bia, "529 Ana", "us_529", None);
    assert_eq!(status(&api, &edu, today).limit, None);
    // Ano sem números publicados: vale o último conhecido, avisado.
    let next = status(&api, &k, NaiveDate::from_ymd_opt(2027, 2, 1).unwrap());
    assert_eq!(next.limit_year, Some(2026));
    // A conta mostra o plano; aporte para ela é aporte, não transferência.
    let view = api.call("accounts.get", json!({"id": k})).unwrap();
    assert_eq!(view["retirement_plan"], "us_401k");
    let plans = api.call("retirement.plans", Value::Null).unwrap();
    assert!(
        plans
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["country"] == "US")
    );
}

#[test]
fn portugal_ppr_limit_depends_on_age_and_brazil_has_no_plans() {
    let (_d, api, ana) = api("PT");
    born(&api, &ana, "1995-01-01");
    let ppr = account(&api, &ana, "PPR", "pt_ppr", None);
    let today = NaiveDate::from_ymd_opt(2026, 10, 22).unwrap();
    let s = status(&api, &ppr, today);
    // 31 anos: o aporte de 2.000 € dá o abatimento máximo de 400 €.
    assert_eq!(s.limit, Some(200_000));
    assert_eq!(s.plan.kind, super::LimitKind::TaxBenefit);
    // Trocar o plano por um de outro grupo.
    let e = api
        .call(
            "accounts.update",
            json!({"id": ppr, "retirement_plan": "nao_existe"}),
        )
        .unwrap_err();
    assert!(!e.message().is_empty());
    let (_d2, br, ana) = self::api("BR");
    assert!(
        br.call(
            "accounts.create",
            json!({"name": "X", "kind": "retirement", "member_ids": [ana]}),
        )
        .is_err()
    );
    assert!(!super::has_plans("BR") && super::has_plans("ES"));
}

#[test]
fn uk_isa_follows_the_tax_year_from_6_april() {
    let (_d, api, ana) = api("GB");
    let isa = account(&api, &ana, "ISA", "gb_isa", None);
    deposit(&api, &isa, "2026-03-01", 500_000);
    deposit(&api, &isa, "2026-05-01", 300_000);
    let s = status(&api, &isa, NaiveDate::from_ymd_opt(2026, 10, 22).unwrap());
    assert_eq!(s.year, 2026);
    assert_eq!(s.contributed, 300_000);
    assert_eq!(s.limit, Some(2_000_000));
    // Em março, ainda é o ano fiscal que começou em abril de 2025.
    let s = status(&api, &isa, NaiveDate::from_ymd_opt(2026, 3, 20).unwrap());
    assert_eq!(s.year, 2025);
    assert_eq!(s.contributed, 500_000);
}
