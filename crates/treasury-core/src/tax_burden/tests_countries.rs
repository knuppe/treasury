//! Income tax on work outside Brazil against cases with a known result
//! (`tests/fixtures/ir-paises/casos-2026.json`, each with its source) and the properties the
//! estimate must keep: net pay never falls when gross rises and the tax does not jump.

use serde_json::Value;

use super::dataset;
use super::rules::{self, PayrollRules};

fn rules_of(country: &str) -> PayrollRules {
    let month = "2026-11";
    match country {
        "US" => super::rules_for(&dataset::get("US").unwrap(), month, None)
            .unwrap()
            .clone(),
        "PT" | "ES" | "FR" | "DE" => {
            super::rules_for(&dataset::get("EU").unwrap(), month, Some(country))
                .unwrap()
                .clone()
        }
        _ => dataset::payroll_only()
            .unwrap()
            .1
            .into_iter()
            .find(|r| r.region.as_deref() == Some(country))
            .unwrap(),
    }
}

fn cents(v: &Value) -> i64 {
    (v.as_f64().unwrap() * 100.0).round() as i64
}

/// Tax for the year for an annual gross paid in equal parts.
fn year_tax(r: &PayrollRules, gross_year: i64, dependents: u32) -> i64 {
    let periods = r.periods as i64;
    let gross = (gross_year as f64 / periods as f64).round() as i64;
    rules::payroll(gross, dependents, r).1 * periods
}

#[test]
fn income_tax_matches_the_official_cases_of_each_country() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ir-paises/casos-2026.json");
    let fx: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let cases = fx["casos"].as_array().unwrap();
    assert!(cases.len() >= 30);
    for c in cases {
        let country = c["pais"].as_str().unwrap();
        let dependents = c["dependentes"].as_u64().unwrap() as u32;
        let base = rules_of(country);
        let r = match c["estado"].as_str().unwrap() {
            "individual" => base.for_single(),
            "conjunta" => base.for_joint(),
            "sozinho_com_dependentes" => base.for_single_parent(),
            other => panic!("unknown marital status: {other}"),
        };
        let got = year_tax(&r, cents(&c["bruto"]), dependents);
        let (want, tol) = (cents(&c["imposto"]), cents(&c["tolerancia"]));
        assert!(
            (got - want).abs() <= tol,
            "{country} gross {}: {got} ≠ {want} ({})",
            c["bruto"],
            c["fonte"]
        );
    }
}

/// Gross rises in steps of 10 (in the currency) to well above the last bracket: the tax never
/// falls, the
/// net never falls and the extra tax never exceeds the extra gross (no cliffs).
#[test]
fn net_pay_never_falls_when_gross_rises() {
    for country in ["US", "PT", "ES", "FR", "DE", "GB", "CA", "AR", "CL"] {
        let base = rules_of(country);
        for (r, dependents) in [
            (base.for_single(), 0),
            (base.for_single(), 2),
            (base.for_single_parent(), 1),
            (base.for_joint(), 2),
        ] {
            // Chile and Argentina have salaries with many more digits.
            let (step, top) = match country {
                "AR" | "CL" => (500_000, 6_000_000_000i64),
                _ => (1_000, 9_000_000),
            };
            let mut prev = (0i64, 0i64, 0i64);
            let mut gross = step;
            while gross <= top {
                let (ss, tax) = rules::payroll(gross, dependents, &r);
                let net = gross - ss - tax;
                // 2 cents of slack for the rounding of each installment.
                assert!(tax + 2 >= prev.1, "{country}: tax falls at {gross}");
                assert!(net + 2 >= prev.2, "{country}: net falls at {gross}");
                // The US child credit drops in steps of US$ 50 per year (law).
                let slack = if country == "US" { 500 } else { 2 };
                assert!(
                    tax - prev.1 <= step + slack,
                    "{country}: tax jump at {gross} ({} → {tax})",
                    prev.1
                );
                prev = (gross, tax, net);
                gross += step;
            }
        }
    }
}

#[test]
fn portugal_minimum_existence_follows_article_70() {
    let r = rules_of("PT");
    // Up to the minimum wage (920 € × 14 = reference value), no IRS.
    for gross in [50_000, 80_000, 92_000] {
        assert_eq!(rules::payroll(gross, 0, &r).1, 0, "{gross}");
    }
    // Just above, the tax grows 3.6 × 12.2% of what exceeds the reference (item b).
    let tax = rules::payroll(93_000, 0, &r).1;
    assert!((tax - 439).abs() <= 2, "{tax}");
    // The deduction ends near 15,886 € per year: from there on it is the table minus the 250 €.
    let gross = 120_000;
    let ss = rules::contribution(gross, &r.inss_bands);
    let table = rules::ir_table(gross - ss.max(r.simplified_discount), &r.ir_monthly);
    assert_eq!(
        rules::monthly_irrf(gross, ss, 0, &r),
        table - r.personal_credit.unwrap()
    );
}

#[test]
fn united_states_head_of_household_and_child_credit_phase_out() {
    let us = rules_of("US");
    let (single, hoh, joint) = (us.for_single(), us.for_single_parent(), us.for_joint());
    // One salary only: the joint return pays less than head of household, and that less than
    // the individual one.
    let tax = |r: &PayrollRules| rules::payroll(800_000, 1, r).1;
    assert!(tax(&joint) < tax(&hoh) && tax(&hoh) < tax(&single));
    // Same income, one child: the head of household table gives less tax.
    let g = 600_000;
    assert!(rules::payroll(g, 1, &hoh).1 < rules::payroll(g, 1, &single).1);
    // FICA the same in both.
    assert_eq!(
        rules::payroll(g, 1, &hoh).0,
        rules::payroll(g, 1, &single).0
    );
    // Child credit: whole up to US$ 200 thousand per year, zero from US$ 244 thousand (one child).
    let credit = |gross_year: i64| {
        let g = gross_year * 100 / 12;
        rules::payroll(g, 0, &single).1 - rules::payroll(g, 1, &single).1
    };
    assert_eq!(credit(180_000), 18_333);
    assert_eq!(credit(220_000), 18_333 - 8_333);
    assert_eq!(credit(250_000), 0);
    // On the joint return the phase-out starts at US$ 400 thousand.
    let joint_credit = |gross_year: i64| {
        let g = gross_year * 100 / 12;
        rules::payroll(g, 0, &joint).1 - rules::payroll(g, 1, &joint).1
    };
    assert_eq!(joint_credit(250_000), 18_333);
    assert_eq!(joint_credit(420_000), 18_333 - 8_333);
    // Country with no rules by marital status: the same in all three.
    let de = rules_of("DE");
    assert_eq!(de.for_single_parent(), de);
    assert_eq!(de.for_joint(), de);
    // Portugal: the single-parent family only changes the general expenses limit (335 € per year).
    let pt = rules_of("PT");
    assert_eq!(pt.for_single_parent().personal_credit, Some(2_393));
    assert_eq!(pt.for_single_parent().ir_monthly, pt.ir_monthly);
}
