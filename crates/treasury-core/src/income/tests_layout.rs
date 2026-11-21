//! The generic payslip with Reference / Earnings / Deductions / Informational columns and
//! signed deductions ("-R$ 400,00"), as in the agents' report (Oct 19, 2026): deduction rows
//! with no deduction word (Crédito do Trabalhador, Fundacao Exemplo, Clinica Exemplo), the
//! "Totais" row with both columns, "1/3 Férias", the 13th-salary advance inside a monthly
//! payslip and the employer with the "Empresa:" label.

use super::payslip::{self, PayslipKind};

const MENSAL: &str = "\
Empresa: COOPERATIVA DE CREDITO EXEMPLO DO VALE CNPJ: 11.222.333/0001-81
Demonstrativo de Pagamento de Salário
Competência: 06/2026
Funcionário: FULANO DE TAL
Código Descrição Referência Proventos Descontos Informativas
1 Salário 30,00 R$ 5.200,00
150 Horas Extras 50% 8,00 R$ 283,64
220 1/3 Ferias R$ 1.733,33
230 Ferias 10,00 R$ 1.733,33
240 13 Salario Adiantado R$ 2.600,00
903 INSS 14,00 -R$ 951,63
904 IRRF 27,50 -R$ 1.287,40
950 Credito do Trabalhador -R$ 640,15
951 Fundacao Exemplo -R$ 312,00
952 Clinica Exemplo -R$ 245,70
990 FGTS 8,00 R$ 924,02
Totais R$ 11.550,30 -R$ 3.436,88
Líquido R$ 8.113,42
";

const PLR: &str = "\
Empregador: COOPERATIVA EXEMPLO
CNPJ 12.345.678/0001-90
Demonstrativo de Pagamento - Participação nos Lucros (PLR)
Competência: 03/2026
Código Descrição Referência Proventos Descontos Informativas
310 PLR R$ 8.000,00
904 IRRF PLR 7,50 -R$ 159,54
Totais R$ 8.000,00 -R$ 159,54
Líquido R$ 7.840,46
";

#[test]
fn signed_deductions_columns_and_the_totals_row() {
    let p = payslip::parse(MENSAL);
    let desc = |s: &str| {
        p.items
            .iter()
            .find(|i| i.description == s)
            .unwrap_or_else(|| panic!("{s} in {:#?}", p.items))
    };
    for d in [
        "INSS",
        "IRRF",
        "Credito do Trabalhador",
        "Fundacao Exemplo",
        "Clinica Exemplo",
    ] {
        assert!(desc(d).deduction, "{d}");
    }
    for e in [
        "Salário",
        "Horas Extras 50%",
        "1/3 Ferias",
        "Ferias",
        "13 Salario Adiantado",
    ] {
        assert!(!desc(e).deduction, "{e}");
    }
    assert_eq!(desc("Credito do Trabalhador").amount, 64_015);
    // Totals and FGTS (informational column) are not items.
    assert!(
        p.items
            .iter()
            .all(|i| !i.description.to_lowercase().starts_with("tota"))
    );
    assert!(p.items.iter().all(|i| !i.description.contains("FGTS")));
    assert_eq!(p.items.len(), 10, "{:#?}", p.items);
    // Gross and deductions from the printed totals, checked against the rows.
    assert_eq!(p.gross, 1_155_030);
    assert_eq!(p.social_security, 95_163);
    assert_eq!(p.income_tax, 128_740);
    assert_eq!(p.other_deductions, 64_015 + 31_200 + 24_570);
    assert_eq!(p.net, 811_342);
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    assert_eq!(p.fgts, Some(92_402));
    // The 13th-salary advance on one row does not change the payslip type.
    assert_eq!(p.kind, PayslipKind::Monthly);
    assert_eq!(p.reference_month.as_deref(), Some("2026-06"));
    assert_eq!(
        p.employer.as_deref(),
        Some("COOPERATIVA DE CREDITO EXEMPLO DO VALE")
    );
    assert_eq!(p.employer_cnpj.as_deref(), Some("11222333000181"));
    assert!(payslip::matches_generic(MENSAL));
}

#[test]
fn plr_with_totals_row_and_employer_on_its_own_line() {
    let p = payslip::parse(PLR);
    assert_eq!(p.kind, PayslipKind::Plr);
    assert_eq!(p.gross, 800_000);
    assert_eq!(p.income_tax, 15_954);
    assert_eq!(p.net, 784_046);
    assert_eq!(p.items.len(), 2, "{:#?}", p.items);
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    assert_eq!(p.employer.as_deref(), Some("COOPERATIVA EXEMPLO"));
    assert_eq!(p.employer_cnpj.as_deref(), Some("12345678000190"));
}

#[test]
fn items_that_do_not_add_up_to_the_printed_totals_are_flagged() {
    // A deduction with no sign and no deduction word: the printed totals hold, and the warning
    // points out that the rows do not add up.
    let text = MENSAL.replace("-R$ 312,00", "R$ 312,00");
    let p = payslip::parse(&text);
    assert_eq!(p.gross, 1_155_030);
    assert_eq!(p.net, 811_342);
    assert!(
        p.warnings.iter().any(|w| w.contains("Fundacao Exemplo")),
        "{:?}",
        p.warnings
    );
    // And the leftover row in the earnings is treated as a deduction, to match the total.
    assert!(
        p.items
            .iter()
            .find(|i| i.description == "Fundacao Exemplo")
            .unwrap()
            .deduction
    );
}

/// PLR from the same employer and the same month as the salary (report of Oct 19, 2026): stored in
/// the CLT income next to the monthly one, but left out of "actual × forecast" and shown in the
/// summary with its type.
#[test]
fn plr_is_listed_with_its_kind_and_stays_out_of_the_monthly_series() {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;
    use crate::time::{add_months, month_of, today_local};

    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let ana = api.call("household.list_members", json!({})).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let month = month_of(add_months(today_local(), -1));
    let (y, m) = month.split_once('-').unwrap();
    let competencia = format!("Competência: {m}/{y}");
    let mensal = MENSAL.replace("Competência: 06/2026", &competencia);
    let plr = PLR
        .replace("Competência: 03/2026", &competencia)
        .replace("CNPJ 12.345.678/0001-90", "CNPJ 11.222.333/0001-81");
    let r = api
        .call(
            "income.payslip_import",
            json!({"text": mensal, "member_id": ana}),
        )
        .unwrap();
    // The PLR is not a duplicate of the monthly: it is another type in the same income.
    let p = api
        .call("income.payslip_preview", json!({"text": plr}))
        .unwrap();
    assert_eq!(p["data"]["kind"], "plr");
    assert_eq!(p["duplicate"], false);
    assert_eq!(p["source_id"], r["source_id"]);
    api.call("income.payslip_import", json!({"text": plr}))
        .unwrap();

    let s = api.call("income.summary", json!({})).unwrap();
    // The summary list: month, type, gross and net.
    let slips = s["payslips"].as_array().unwrap();
    assert_eq!(slips.len(), 2, "{s}");
    let plr_row = slips.iter().find(|x| x["kind"] == "plr").unwrap();
    assert_eq!(plr_row["month"], month.as_str());
    assert_eq!(plr_row["label"], "PLR");
    assert_eq!(plr_row["gross"], 800_000);
    assert_eq!(plr_row["net"], 784_046);
    assert_eq!(plr_row["extra"], true);
    let monthly_row = slips.iter().find(|x| x["kind"] == "monthly").unwrap();
    assert_eq!(monthly_row["extra"], false);
    // "Actual × forecast" for the month: only the monthly.
    let real = s["actual"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["real"]["month"] == month.as_str())
        .unwrap();
    assert_eq!(real["real"]["gross"], 1_155_030, "{real}");
    // The 12 months sum everything and say what came in beyond the salary.
    let mi = &s["members"][0];
    assert_eq!(mi["last_12m"]["gross"], 1_155_030 + 800_000);
    assert_eq!(mi["last_12m"]["extras"], json!(["PLR"]));
}

/// PLR: the payslip's IRRF is checked against the PLR's exclusive table (Lei 10.101/2000 art. 3;
/// Receita, table since May 2025): R$ 8,000 in 2026 is exempt, so the R$ 159.54
/// withheld becomes a warning; with the right IRRF, nothing.
#[test]
fn plr_withholding_is_checked_against_the_plr_table() {
    use crate::changes::{Author, record};
    let dir = tempfile::tempdir().unwrap();
    let mut db =
        crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST).unwrap();
    record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
        crate::household::seed(tx, "Família", "BRL", &["Ana".into()])
    })
    .unwrap();
    let mut p = payslip::parse(PLR);
    p.reference_month = Some("2026-03".into());
    let w = payslip::plr_check(db.conn(), None, &p).unwrap().unwrap();
    assert!(w.contains("159,54") && w.contains("0,00"), "{w}");
    p.income_tax = 0;
    assert!(payslip::plr_check(db.conn(), None, &p).unwrap().is_none());
}
