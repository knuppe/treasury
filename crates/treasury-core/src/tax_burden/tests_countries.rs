//! Imposto de renda do trabalho fora do Brasil contra casos de resultado conhecido
//! (`tests/fixtures/ir-paises/casos-2026.json`, cada um com a fonte) e as propriedades que a
//! estimativa deve manter: o líquido nunca cai quando o bruto sobe e o imposto não dá saltos.

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

/// Imposto do ano para um bruto anual pago em partes iguais.
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
            other => panic!("estado civil desconhecido: {other}"),
        };
        let got = year_tax(&r, cents(&c["bruto"]), dependents);
        let (want, tol) = (cents(&c["imposto"]), cents(&c["tolerancia"]));
        assert!(
            (got - want).abs() <= tol,
            "{country} bruto {}: {got} ≠ {want} ({})",
            c["bruto"],
            c["fonte"]
        );
    }
}

/// O bruto sobe de 10 em 10 (na moeda) até bem acima da última faixa: o imposto nunca cai, o
/// líquido nunca cai e o imposto a mais nunca passa do bruto a mais (sem degraus).
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
            // Chile e Argentina têm salários com muito mais dígitos.
            let (step, top) = match country {
                "AR" | "CL" => (500_000, 6_000_000_000i64),
                _ => (1_000, 9_000_000),
            };
            let mut prev = (0i64, 0i64, 0i64);
            let mut gross = step;
            while gross <= top {
                let (ss, tax) = rules::payroll(gross, dependents, &r);
                let net = gross - ss - tax;
                // 2 centavos de folga pelo arredondamento de cada parcela.
                assert!(tax + 2 >= prev.1, "{country}: imposto cai em {gross}");
                assert!(net + 2 >= prev.2, "{country}: líquido cai em {gross}");
                // O crédito por filho dos EUA cai em degraus de US$ 50 por ano (lei).
                let slack = if country == "US" { 500 } else { 2 };
                assert!(
                    tax - prev.1 <= step + slack,
                    "{country}: salto de imposto em {gross} ({} → {tax})",
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
    // Até o salário mínimo (920 € × 14 = valor de referência), nenhum IRS.
    for gross in [50_000, 80_000, 92_000] {
        assert_eq!(rules::payroll(gross, 0, &r).1, 0, "{gross}");
    }
    // Logo acima, o imposto cresce 3,6 × 12,2% do que passa da referência (alínea b).
    let tax = rules::payroll(93_000, 0, &r).1;
    assert!((tax - 439).abs() <= 2, "{tax}");
    // O abatimento acaba perto de 15 886 € por ano: daí em diante é a tabela menos os 250 €.
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
    // Um salário só: a declaração conjunta paga menos que head of household, e este menos que
    // a individual.
    let tax = |r: &PayrollRules| rules::payroll(800_000, 1, r).1;
    assert!(tax(&joint) < tax(&hoh) && tax(&hoh) < tax(&single));
    // Mesma renda, um filho: a tabela de head of household dá menos imposto.
    let g = 600_000;
    assert!(rules::payroll(g, 1, &hoh).1 < rules::payroll(g, 1, &single).1);
    // FICA igual nos dois.
    assert_eq!(
        rules::payroll(g, 1, &hoh).0,
        rules::payroll(g, 1, &single).0
    );
    // Crédito por filho: inteiro até US$ 200 mil por ano, zero a partir de US$ 244 mil (um filho).
    let credit = |gross_year: i64| {
        let g = gross_year * 100 / 12;
        rules::payroll(g, 0, &single).1 - rules::payroll(g, 1, &single).1
    };
    assert_eq!(credit(180_000), 18_333);
    assert_eq!(credit(220_000), 18_333 - 8_333);
    assert_eq!(credit(250_000), 0);
    // Na conjunta a retirada começa em US$ 400 mil.
    let joint_credit = |gross_year: i64| {
        let g = gross_year * 100 / 12;
        rules::payroll(g, 0, &joint).1 - rules::payroll(g, 1, &joint).1
    };
    assert_eq!(joint_credit(250_000), 18_333);
    assert_eq!(joint_credit(420_000), 18_333 - 8_333);
    // País sem regras por estado civil: as mesmas nos três.
    let de = rules_of("DE");
    assert_eq!(de.for_single_parent(), de);
    assert_eq!(de.for_joint(), de);
    // Portugal: a família monoparental só muda o limite das despesas gerais (335 € no ano).
    let pt = rules_of("PT");
    assert_eq!(pt.for_single_parent().personal_credit, Some(2_393));
    assert_eq!(pt.for_single_parent().ir_monthly, pt.ir_monthly);
}
