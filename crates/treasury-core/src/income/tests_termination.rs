//! Simulação de demissão: verbas por motivo conferidas à mão (valores inventados), descontos
//! pelas tabelas do mês, FGTS e seguro-desemprego, e só em cofres do Brasil.

use serde_json::{Value, json};

use super::termination::unemployment_installment;
use crate::api::Api;
use crate::paths::TreasuryPaths;
use crate::tax_burden::{dataset, rules, rules_for};

fn vault(country: &str) -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"], "country": country}),
    )
    .unwrap();
    (dir, api)
}

fn sim(api: &Api, args: Value) -> Value {
    api.call("income.simulate_termination", args).unwrap()
}

fn amount(r: &Value, code: &str) -> i64 {
    r["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["code"] == code)
        .map(|l| l["amount"].as_i64().unwrap())
        .unwrap_or(0)
}

/// INSS e IRRF de um valor pelas tabelas do mês.
fn taxes(gross: i64, month: &str) -> (i64, i64) {
    let ds = dataset::get("BR").unwrap();
    rules::payroll(gross, 0, rules_for(&ds, month, None).unwrap())
}

#[test]
fn dismissal_without_cause_with_paid_notice() {
    let (_d, api) = vault("BR");
    let r = sim(
        &api,
        json!({"gross_monthly": 300_000, "admission_date": "2023-03-10",
               "termination_date": "2026-08-20", "reason": "sem_justa_causa",
               "fgts_balance": 1_000_000}),
    );
    // 3 anos completos: 30 + 9 dias de aviso, que projetam o contrato até 28/09.
    assert_eq!(r["notice"], "indenizado");
    assert_eq!(r["notice_days"], 39);
    assert_eq!(r["projected_end"], "2026-09-28");
    assert_eq!(amount(&r, "saldo_salario"), 200_000);
    assert_eq!(amount(&r, "aviso_previo"), 390_000);
    // 13º: jan a set (setembro com 28 dias) = 9/12.
    assert_eq!(amount(&r, "decimo_terceiro"), 225_000);
    // Férias: de 10/03 a 28/09 = 6 meses + 19 dias = 7/12, com 1/3.
    assert_eq!(amount(&r, "ferias_proporcionais"), 175_000 + 58_333);
    assert_eq!(r["gross"], 200_000 + 390_000 + 225_000 + 233_333);
    let (i, ir) = taxes(200_000, "2026-08");
    let (i13, ir13) = taxes(225_000, "2026-08");
    assert_eq!(amount(&r, "inss"), -i);
    assert_eq!(r["deductions"], i + ir + i13 + ir13);
    assert_eq!(
        r["net"].as_i64().unwrap(),
        1_048_333 - (i + ir + i13 + ir13)
    );
    // FGTS: 8% de saldo + 13º + aviso; multa de 40% sobre saldo + depósito; saque integral.
    let f = &r["fgts"];
    assert_eq!(f["deposit"], 65_200);
    assert_eq!(f["fine_rate"], 40);
    assert_eq!(f["fine"], 426_080);
    assert_eq!(f["withdrawable"], 1_065_200 + 426_080);
    // Seguro: 1º pedido, 36 meses ou mais: 5 parcelas; média de R$ 3.000 na 2ª faixa de 2026.
    let u = &r["unemployment"];
    assert_eq!(u["eligible"], true);
    assert_eq!(u["installments"], 5);
    assert_eq!(u["installment"], 177_774 + (300_000 - 222_217) / 2);
    assert!(u["table"].as_str().unwrap().contains("11/01/2026"));
    assert_eq!(
        r["total_available"].as_i64().unwrap(),
        r["net"].as_i64().unwrap() + 1_491_280 + 216_665 * 5
    );
}

#[test]
fn resignation_without_serving_notice_owes_the_month() {
    let (_d, api) = vault("BR");
    let r = sim(
        &api,
        json!({"gross_monthly": 200_000, "admission_date": "2025-11-05",
               "termination_date": "2026-02-14", "reason": "pedido_demissao",
               "notice": "nao_cumprido", "fgts_balance": 50_000}),
    );
    assert_eq!(amount(&r, "saldo_salario"), 93_333);
    // Fevereiro com 14 dias não conta: 1/12 de 13º; férias de 3/12 (pedido mantém).
    assert_eq!(amount(&r, "decimo_terceiro"), 16_666);
    assert_eq!(amount(&r, "ferias_proporcionais"), 50_000 + 16_666);
    assert_eq!(amount(&r, "aviso_previo"), 0);
    assert_eq!(amount(&r, "desconto_aviso"), -200_000);
    assert!(r["net"].as_i64().unwrap() < 0);
    assert_eq!(r["fgts"]["fine"], 0);
    assert_eq!(r["fgts"]["withdrawable"], 0);
    assert_eq!(r["unemployment"]["eligible"], false);
}

#[test]
fn just_cause_keeps_only_salary_and_overdue_vacation() {
    let (_d, api) = vault("BR");
    let r = sim(
        &api,
        json!({"gross_monthly": 300_000, "admission_date": "2022-01-03",
               "termination_date": "2026-05-31", "reason": "justa_causa",
               "overdue_vacation_days": 30}),
    );
    assert_eq!(amount(&r, "saldo_salario"), 300_000);
    assert_eq!(amount(&r, "decimo_terceiro"), 0);
    assert_eq!(amount(&r, "ferias_proporcionais"), 0);
    assert_eq!(amount(&r, "ferias_vencidas"), 400_000);
    assert_eq!(r["notice_days"], 0);
    assert_eq!(r["fgts"]["withdrawal_allowed"], false);
    // Sem saldo do FGTS informado, avisa.
    assert!(r["warnings"].as_array().unwrap().len() >= 2);
}

#[test]
fn agreement_pays_half_notice_and_fine_and_80_percent() {
    let (_d, api) = vault("BR");
    let r = sim(
        &api,
        json!({"gross_monthly": 300_000, "admission_date": "2024-04-01",
               "termination_date": "2026-04-30", "reason": "acordo", "fgts_balance": 500_000}),
    );
    // 2 anos: aviso cheio de 36 dias, metade no acordo.
    assert_eq!(r["notice_days"], 18);
    assert_eq!(amount(&r, "aviso_previo"), 180_000);
    let f = &r["fgts"];
    assert_eq!(f["fine_rate"], 20);
    assert_eq!(f["withdrawal_rate"], 80);
    let deposit = f["deposit"].as_i64().unwrap();
    let base = 500_000 + deposit;
    assert_eq!(f["fine"], (base * 20 + 50) / 100);
    assert_eq!(
        f["withdrawable"].as_i64().unwrap(),
        (base * 80 + 50) / 100 + (base * 20 + 50) / 100
    );
    assert_eq!(r["unemployment"]["eligible"], false);
}

#[test]
fn trial_contract_broken_by_employer_pays_half_of_the_rest() {
    let (_d, api) = vault("BR");
    let r = sim(
        &api,
        json!({"gross_monthly": 300_000, "admission_date": "2026-05-01",
               "termination_date": "2026-06-10", "reason": "quebra_experiencia_empregador",
               "contract_end": "2026-07-29"}),
    );
    // 49 dias até o fim combinado: metade.
    assert_eq!(amount(&r, "indenizacao_479"), 245_000);
    assert_eq!(r["fgts"]["fine_rate"], 40);
    // Pouco tempo de carteira: sem carência para o seguro.
    let u = &r["unemployment"];
    assert_eq!(u["eligible"], false);
    assert!(u["reason"].as_str().unwrap().contains("12 meses"));
    // Pelo empregado: vira desconto; e sem o fim do contrato, erro claro.
    let e = api
        .call(
            "income.simulate_termination",
            json!({"gross_monthly": 300_000, "admission_date": "2026-05-01",
                   "termination_date": "2026-06-10", "reason": "quebra_experiencia_empregado"}),
        )
        .unwrap_err();
    assert!(e.to_string().contains("experiência"), "{e}");
}

#[test]
fn unemployment_insurance_tables_and_waiting_period() {
    // 2025: piso no mínimo de R$ 1.518; 2026: teto de R$ 2.518,65.
    assert_eq!(unemployment_installment(150_000, "2025-06-01"), 151_800);
    assert_eq!(unemployment_installment(200_000, "2025-06-01"), 160_000);
    assert_eq!(unemployment_installment(500_000, "2025-06-01"), 242_411);
    assert_eq!(unemployment_installment(500_000, "2026-03-01"), 251_865);
    // Antes de 11/01/2026 ainda vale a de 2025.
    assert_eq!(unemployment_installment(500_000, "2026-01-05"), 242_411);
    let (_d, api) = vault("BR");
    // 2º pedido com 10 meses: 3 parcelas, pela média dos últimos salários.
    let r = sim(
        &api,
        json!({"gross_monthly": 300_000, "admission_date": "2025-09-01",
               "termination_date": "2026-07-15", "reason": "sem_justa_causa",
               "previous_claims": 1, "last_salaries": [200_000, 200_000, 260_000]}),
    );
    let u = &r["unemployment"];
    assert_eq!(u["average_salary"], 220_000);
    assert_eq!(u["installment"], 176_000);
    assert_eq!(u["installments"], 3);
    // 1º pedido com 10 meses: sem carência.
    let r = sim(
        &api,
        json!({"gross_monthly": 300_000, "admission_date": "2025-09-01",
               "termination_date": "2026-07-15", "reason": "sem_justa_causa"}),
    );
    assert_eq!(r["unemployment"]["eligible"], false);
}

#[test]
fn prefills_from_the_income_and_is_brazil_only() {
    let (_d, api) = vault("BR");
    let member = api.call("household.list_members", json!({})).unwrap()[0]["id"].clone();
    let src = api
        .call(
            "income.create_source",
            json!({"member_id": member, "name": "Emprego", "kind": "clt",
                   "gross_monthly": 300_000, "start_date": "2023-03-10", "dependents": 1}),
        )
        .unwrap();
    let r = sim(
        &api,
        json!({"source_id": src["id"], "termination_date": "2026-08-20",
               "reason": "sem_justa_causa", "fgts_balance": 0}),
    );
    assert_eq!(amount(&r, "aviso_previo"), 390_000);
    assert!(r["months_covered"].as_f64().unwrap() > 1.0);
    assert!(
        api.call(
            "income.simulate_termination",
            json!({"termination_date": "2026-08-20", "reason": "sem_justa_causa"})
        )
        .is_err()
    );
    let (_d2, us) = vault("US");
    assert!(
        us.call(
            "income.simulate_termination",
            json!({"gross_monthly": 300_000, "admission_date": "2023-03-10",
                   "termination_date": "2026-08-20", "reason": "sem_justa_causa"})
        )
        .is_err()
    );
}
