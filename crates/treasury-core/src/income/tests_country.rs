//! Income outside Brazil (docs/countries.md): payroll deductions for the United States and
//! Portugal, vacation and Christmas subsidies and reading the paystub and the recibo de vencimento.

use serde_json::{Value, json};

use super::payslip::{self, PayslipKind, Style};
use crate::api::Api;
use crate::paths::TreasuryPaths;

fn api(country: &str) -> (tempfile::TempDir, Api) {
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

fn ana(api: &Api) -> String {
    api.call("household.list_members", json!({})).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn projection(api: &Api) -> Vec<Value> {
    let s = api.call("income.summary", json!({"months": 12})).unwrap();
    s["members"][0]["projection"][0]["months"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn united_states_withholds_fica_and_federal_tax() {
    let (_d, api) = api("US");
    let id = ana(&api);
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Salary", "kind": "clt", "gross_monthly": 500_000, "has_thirteenth": true, "vacation_month": 7}),
    )
    .unwrap();
    let months = projection(&api);
    for m in &months {
        // FICA: 7.65% (below the Social Security cap).
        assert_eq!(m["social_security"], 38_250, "{m}");
        // Federal: (5000 − 16100/12) = 3658.33 in the 12% bracket (deduction 20.67).
        assert_eq!(m["income_tax"], 41_833, "{m}");
        assert_eq!(
            m["gross"], 500_000,
            "in the US there is no 13th salary or vacation third"
        );
    }
    let s = api.call("income.summary", json!({"months": 12})).unwrap();
    assert!(s["notes"].to_string().contains("FICA"), "{}", s["notes"]);
}

/// Marital status comes from the family record, as on the W-4 form: with a child and no
/// spouse, head of household; with a spouse without income, joint return; with both with
/// income, the individual table.
#[test]
fn united_states_filing_status_follows_the_family_registry() {
    let (_d, api) = api("US");
    let id = ana(&api);
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Salary", "kind": "clt", "gross_monthly": 500_000, "dependents": 1}),
    )
    .unwrap();
    // (5000 − 24150/12) = 2987.50: 12% − 29.50 = 329.00, minus 183.33 of the child credit.
    assert_eq!(projection(&api)[0]["income_tax"], 14_567);
    let spouse = api
        .call("household.create_member", json!({"name": "Bruno"}))
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "household.update_member",
        json!({"id": spouse, "profile": {"relationship": "conjuge"}}),
    )
    .unwrap();
    // Joint: (5000 − 32200/12) = 2316.67: 12% − 41.33 = 236.67, minus 183.33.
    assert_eq!(projection(&api)[0]["income_tax"], 5_334);
    api.call(
        "income.create_source",
        json!({"member_id": spouse, "name": "Wages", "kind": "clt", "gross_monthly": 300_000}),
    )
    .unwrap();
    // Both with income: individual table, 418.33 − 183.33.
    let s = api.call("income.summary", json!({"months": 12})).unwrap();
    let ana_tax = s["members"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["projection"].as_array().unwrap().clone())
        .find(|p| p["name"] == "Salary")
        .unwrap()["months"][0]["income_tax"]
        .clone();
    assert_eq!(ana_tax, 23_500);
}

#[test]
fn portugal_pays_fourteen_salaries() {
    let (_d, api) = api("PT");
    let id = ana(&api);
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Salário", "kind": "clt", "gross_monthly": 200_000, "has_thirteenth": true, "vacation_month": 7}),
    )
    .unwrap();
    let months = projection(&api);
    let normal = months
        .iter()
        .find(|m| m["extras"].as_array().unwrap().is_empty())
        .unwrap();
    // Segurança Social 11%; IRS from the annual tax of 14 × €2000 ÷ 14 (2026 brackets):
    // €292.02 from the table minus €17.86 of the general family expenses deduction (€250 ÷ 14).
    assert_eq!(normal["social_security"], 22_000);
    assert_eq!(normal["income_tax"], 27_416);
    let dec = months
        .iter()
        .find(|m| m["month"].as_str().unwrap().ends_with("-12"))
        .unwrap();
    assert_eq!(dec["gross"], 400_000);
    assert_eq!(dec["social_security"], 44_000);
    assert!(dec["extras"].to_string().contains("Natal"));
    let jul = months
        .iter()
        .find(|m| m["month"].as_str().unwrap().ends_with("-07"))
        .unwrap();
    assert_eq!(jul["gross"], 400_000);
    assert!(jul["extras"].to_string().contains("férias"));
    // Minimum wage: the existence minimum zeroes the IRS.
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Part-time", "kind": "clt", "gross_monthly": 92_000}),
    )
    .unwrap();
    let s = api.call("income.summary", json!({"months": 12})).unwrap();
    // An ordinary month (without the Christmas or the vacation subsidy), whatever today's date is.
    let low = s["members"][0]["projection"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "Part-time")
        .unwrap()["months"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| {
            let m = m["month"].as_str().unwrap();
            !m.ends_with("-07") && !m.ends_with("-12")
        })
        .unwrap()
        .clone();
    assert_eq!(low["income_tax"], 0);
    assert_eq!(low["social_security"], 10_120);
}

#[test]
fn a_generic_country_does_not_compute_withholding() {
    let (_d, api) = api("MX");
    let id = ana(&api);
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Sueldo", "kind": "clt", "gross_monthly": 3_000_000, "net_monthly": 2_500_000}),
    )
    .unwrap();
    let m = &projection(&api)[0];
    assert_eq!(
        (m["social_security"].as_i64(), m["income_tax"].as_i64()),
        (Some(0), Some(0))
    );
    assert_eq!(m["other_deductions"], 500_000);
}

const RECIBO: &str = "\
EMPRESA EXEMPLO, LDA.  NIF 501234567
RECIBO DE VENCIMENTO
Período: Setembro 2026
Trabalhador: FULANO DE TAL  NIF 234567890  NISS 12345678901
Vencimento Base 1.800,00
Subsídio de Alimentação 22 dias 135,30
Segurança Social 11% 198,00
IRS 17,4% 313,20
Quota Sindical 18,00
Total Ilíquido 1.935,30
Total Descontos 529,20
Líquido a Receber 1.406,10
";

const PAYSTUB: &str = "\
ACME CORP  EIN 12-3456789
EARNINGS STATEMENT
Pay Period 09/01/2026 - 09/15/2026   Pay Date 09/19/2026
Employee: JOHN DOE  SSN XXX-XX-1234
Earnings Hours Rate Current YTD
Regular 80.00 31.25 2,500.00 45,000.00
Gross Pay 2,500.00 45,000.00
Federal Income Tax 210.00 3,780.00
Social Security 155.00 2,790.00
Medicare 36.25 652.50
CA State Income Tax 95.00 1,710.00
401(k) Pre-Tax 125.00 2,250.00
Net Pay 1,878.75 33,817.50
";

#[test]
fn reads_a_portuguese_recibo_de_vencimento() {
    assert_eq!(payslip::style_of(RECIBO), Style::Pt);
    assert!(payslip::looks_like(RECIBO));
    let p = payslip::parse(RECIBO);
    assert_eq!(p.reference_month.as_deref(), Some("2026-09"));
    assert_eq!((p.gross, p.net), (193_530, 140_610));
    assert_eq!((p.social_security, p.income_tax), (19_800, 31_320));
    assert_eq!(p.other_deductions, 1_800);
    assert_eq!(p.base_salary, Some(180_000));
    assert_eq!(p.fgts, None);
    // Nobody's NIF is stored.
    assert_eq!(
        (p.employer.as_deref(), p.employer_cnpj.as_deref()),
        (None, None)
    );
    assert!(p.items.iter().all(|i| !i.description.contains("FULANO")));
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    let natal = RECIBO.replace(
        "RECIBO DE VENCIMENTO",
        "RECIBO DE VENCIMENTO - SUBSÍDIO DE NATAL",
    );
    assert_eq!(payslip::parse(&natal).kind, PayslipKind::Thirteenth2);
}

#[test]
fn reads_an_american_paystub() {
    assert_eq!(payslip::style_of(PAYSTUB), Style::Us);
    assert!(payslip::looks_like(PAYSTUB));
    let p = payslip::parse(PAYSTUB);
    assert_eq!(p.reference_month.as_deref(), Some("2026-09"));
    // The period column, not the year one (YTD).
    assert_eq!((p.gross, p.net), (250_000, 187_875));
    // Social Security + Medicare; federal + state tax.
    assert_eq!((p.social_security, p.income_tax), (19_125, 30_500));
    assert_eq!(p.other_deductions, 12_500);
    assert_eq!(
        (p.employer.as_deref(), p.employer_cnpj.as_deref()),
        (None, None)
    );
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    // The Brazilian holerite stays the same.
    assert_eq!(payslip::style_of(super::tests::HOLERITE), Style::Br);
}

/// Extra payments, warnings, history reasons and errors come out in the installation language.
#[test]
fn texts_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let (_d, api) = api("PT");
    let id = ana(&api);
    set_current(Locale::En);
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Salário", "kind": "clt", "gross_monthly": 200_000, "has_thirteenth": true, "vacation_month": 7}),
    )
    .unwrap();
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Bicos", "kind": "informal"}),
    )
    .unwrap();
    let s = api.call("income.summary", json!({"months": 12})).unwrap();
    let e = api
        .call(
            "income.create_source",
            json!({"member_id": id, "name": "X", "kind": "clt", "pay_day": 40}),
        )
        .unwrap_err()
        .message();
    let history = api.call("changes.list", json!({"limit": 5})).unwrap();
    set_current(Locale::PtBr);
    let extras = s["monthly"].to_string();
    assert!(extras.contains("Christmas allowance"), "{extras}");
    assert!(extras.contains("holiday allowance"), "{extras}");
    let notes = s["notes"].to_string();
    assert!(
        notes.contains("Bicos: enter the gross or net amount"),
        "{notes}"
    );
    assert!(
        notes.contains("Segurança Social and IRS estimated"),
        "{notes}"
    );
    assert_eq!(e, "payday from 1 to 31");
    assert!(
        history.to_string().contains("Income added: Bicos"),
        "{history}"
    );
}

#[test]
fn spain_pays_fourteen_salaries_with_irpf_and_seguridad_social() {
    let (_d, api) = api("ES");
    let id = ana(&api);
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Salario", "kind": "clt", "gross_monthly": 200_000, "has_thirteenth": true}),
    )
    .unwrap();
    let months = projection(&api);
    let normal = months
        .iter()
        .find(|m| m["extras"].as_array().unwrap().is_empty())
        .unwrap();
    // Seguridad Social 6.50%; IRPF: base = 2000 − 130 − 142.86 (expenses) = 1727.14 at 30%
    // (deduction 131.03) minus the personal minimum (75.32 per payment).
    assert_eq!(normal["social_security"], 13_000);
    assert_eq!(normal["income_tax"], 31_179);
    for end in ["-06", "-12"] {
        let m = months
            .iter()
            .find(|m| m["month"].as_str().unwrap().ends_with(end))
            .unwrap();
        assert_eq!(m["gross"], 400_000, "{m}");
        assert!(m["extras"].to_string().contains("paga extra"), "{m}");
    }
    // Low salary: the reduction for earned income lowers the base.
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Media jornada", "kind": "clt", "gross_monthly": 100_000}),
    )
    .unwrap();
    let s = api.call("income.summary", json!({"months": 12})).unwrap();
    let low = s["members"][0]["projection"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "Media jornada")
        .unwrap()["months"][0]
        .clone();
    assert_eq!(low["income_tax"], 0, "{low}");
    assert!(s["notes"].to_string().contains("IRPF"), "{}", s["notes"]);
}

const NOMINA: &str = "\
EMPRESA EJEMPLO, S.L.  CIF B12345678
RECIBO INDIVIDUAL JUSTIFICATIVO DEL PAGO DE SALARIOS
Periodo de liquidación: del 01/10/2026 al 31/10/2026
Trabajador: FULANITO DE TAL  NIF 12345678Z  Nº afiliación 28/1234567890
I. DEVENGOS
Salario base 1.900,00
Plus transporte 100,00
II. DEDUCCIONES
Contingencias comunes 4,70% 94,00
Desempleo 1,55% 31,00
Formación profesional 0,10% 2,00
MEI 0,15% 3,00
Retención IRPF 15,00% 300,00
A. TOTAL DEVENGADO 2.000,00
B. TOTAL A DEDUCIR 430,00
LÍQUIDO TOTAL A PERCIBIR (A-B) 1.570,00
";

#[test]
fn reads_a_spanish_nomina() {
    assert_eq!(payslip::style_of(NOMINA), Style::Es);
    assert!(payslip::looks_like(NOMINA));
    let p = payslip::parse(NOMINA);
    assert_eq!(p.reference_month.as_deref(), Some("2026-10"));
    assert_eq!((p.gross, p.net), (200_000, 157_000));
    assert_eq!((p.social_security, p.income_tax), (13_000, 30_000));
    assert_eq!(p.other_deductions, 0);
    assert_eq!(p.base_salary, Some(190_000));
    // Neither anyone's NIF nor affiliation number is stored.
    assert_eq!(
        (p.employer.as_deref(), p.employer_cnpj.as_deref()),
        (None, None)
    );
    assert!(p.items.iter().all(|i| !i.description.contains("FULANITO")));
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    let extra = NOMINA.replace(
        "RECIBO INDIVIDUAL JUSTIFICATIVO DEL PAGO DE SALARIOS",
        "RECIBO DE SALARIOS - PAGA EXTRA DE NAVIDAD",
    );
    assert_eq!(payslip::parse(&extra).kind, PayslipKind::Thirteenth2);
    // The others stay the same.
    assert_eq!(payslip::style_of(RECIBO), Style::Pt);
}

/// First month of the projection without an extra payment (contribution, tax).
fn normal_month(country: &str, gross: i64, thirteenth: bool) -> (Value, Vec<Value>) {
    let (_d, api) = api(country);
    let id = ana(&api);
    api.call(
        "income.create_source",
        json!({"member_id": id, "name": "Salary", "kind": "clt", "gross_monthly": gross, "has_thirteenth": thirteenth}),
    )
    .unwrap();
    let months = projection(&api);
    let normal = months
        .iter()
        .find(|m| m["extras"].as_array().unwrap().is_empty())
        .unwrap()
        .clone();
    (normal, months)
}

#[test]
fn round_two_countries_withhold_their_own_way() {
    // United Kingdom: £3,000 per month; tax (3000 − 1047.50) × 20% and NI × 8%.
    let (m, _) = normal_month("GB", 300_000, false);
    assert_eq!(
        (m["social_security"].as_i64(), m["income_tax"].as_i64()),
        (Some(15_620), Some(39_050))
    );
    // Canada (Ontario): $5,000 per month; CPP + EI and the tax summed with the credits.
    let (m, _) = normal_month("CA", 500_000, false);
    assert!(m["social_security"].as_i64().unwrap() > 30_000, "{m}");
    assert!(m["income_tax"].as_i64().unwrap() > 50_000, "{m}");
    // Germany: €4,000 per month, class I.
    let (m, _) = normal_month("DE", 400_000, false);
    assert_eq!(m["social_security"], 84_600);
    assert!(
        (40_000..70_000).contains(&m["income_tax"].as_i64().unwrap()),
        "{m}"
    );
    // France: €3,000 per month, one share. Cotisations of 20.84% (old age 7.30%, Agirc-Arrco
    // 4.01%, CSG and CRDS 9.53%); the tax base adds back the 2.85% non-deductible:
    // (2460.27 × 12 × 0.9 − 11,600) × 11% = 1,646.80 a year, minus the décote of 151.82.
    let (m, _) = normal_month("FR", 300_000, false);
    assert_eq!(m["social_security"], 62_521);
    assert_eq!(m["income_tax"], 12_458);
    // Chile: 900,000 pesos per month stay below 13.5 UTM after the cotizaciones.
    let (m, _) = normal_month("CL", 900_000, false);
    assert_eq!(m["social_security"], 168_300);
    assert_eq!(m["income_tax"], 0);
    // Argentina: the aguinaldo comes in June and December, half a salary each.
    let (_, months) = normal_month("AR", 200_000_000, true);
    let jun = months
        .iter()
        .find(|m| m["month"].as_str().unwrap().ends_with("-06"))
        .unwrap();
    assert_eq!(jun["gross"], 300_000_000);
    assert!(jun["extras"].to_string().contains("aguinaldo"), "{jun}");
}

/// Made-up receipts (companies, people and sample numbers) for the round 2 countries:
/// (text, country, month, gross, contribution, tax, net).
const ROUND_TWO: &[(&str, &str, &str, i64, i64, i64, i64)] = &[
    (
        "ACME LTD  PAYSLIP
Employee: JANE EXAMPLE   NI number QQ123456C   Tax code 1257L
Pay date 25/10/2026
Payments This period Year to date
Basic salary 3,000.00 21,000.00
Deductions
PAYE Income Tax 390.50 2,733.50
National Insurance 156.20 1,093.40
Pension 150.00 1,050.00
Total gross pay 3,000.00 21,000.00
Total deductions 696.70 4,876.90
Net pay 2,303.30 16,123.10
",
        "GB",
        "2026-10",
        300_000,
        15_620,
        39_050,
        230_330,
    ),
    (
        "MAPLE EXAMPLE INC.  STATEMENT OF EARNINGS
Employee: JOHN EXAMPLE
Pay date 2026-10-30
Earnings Current YTD
Regular pay 5,000.00 50,000.00
Gross pay 5,000.00 50,000.00
Federal tax 620.00 6,200.00
Provincial tax 280.00 2,800.00
CPP 280.13 2,801.30
EI 81.50 815.00
Total deductions 1,261.63 12,616.30
Net pay 3,738.37 37,383.70
",
        "CA",
        "2026-10",
        500_000,
        36_163,
        90_000,
        373_837,
    ),
    (
        "SOCIETE EXEMPLE SAS
BULLETIN DE PAIE  Période : octobre 2026
Salarié : MARIE EXEMPLE
Salaire de base 3 000,00
Salaire brut 3 000,00
Sécurité sociale vieillesse plafonnée 207,00
Retraite complémentaire Agirc-Arrco 120,90
CSG déductible 200,43
CSG/CRDS non déductible 98,24
Total des cotisations et contributions 626,57
Net à payer avant impôt sur le revenu 2 373,43
Impôt sur le revenu prélevé à la source 125,33
Net payé 2 248,10
",
        "FR",
        "2026-10",
        300_000,
        62_657,
        12_533,
        224_810,
    ),
    (
        "BEISPIEL GMBH  Entgeltabrechnung Oktober 2026
Mitarbeiter: MAX MUSTER  Steuerklasse 1
Gehalt 4.000,00
Gesamtbrutto 4.000,00
Lohnsteuer 541,25
Rentenversicherung 372,00
Arbeitslosenversicherung 52,00
Krankenversicherung 350,00
Pflegeversicherung 72,00
Summe der Abzüge 1.387,25
Auszahlungsbetrag 2.612,75
",
        "DE",
        "2026-10",
        400_000,
        84_600,
        54_125,
        261_275,
    ),
    (
        "EMPRESA EJEMPLO S.A.  RECIBO DE SUELDO
Período: octubre 2026   Legajo 123
Sueldo básico 2.000.000,00
Total remunerativo 2.000.000,00
Jubilación 11% 220.000,00
Ley 19.032 3% 60.000,00
Obra social 3% 60.000,00
Impuesto a las Ganancias 0,00
Total descuentos 340.000,00
Neto a cobrar 1.660.000,00
",
        "AR",
        "2026-10",
        200_000_000,
        34_000_000,
        0,
        166_000_000,
    ),
    (
        "EMPRESA EJEMPLO SPA  LIQUIDACIÓN DE SUELDO
Mes: octubre 2026
Sueldo base $ 1.500.000
Gratificación $ 219.114
Total haberes $ 1.719.114
AFP 11,1% $ 190.822
Fonasa 7% $ 120.338
Seguro de cesantía 0,6% $ 10.315
Impuesto único $ 20.000
Total descuentos $ 341.475
Alcance líquido $ 1.377.639
",
        "CL",
        "2026-10",
        1_719_114,
        321_475,
        20_000,
        1_377_639,
    ),
];

#[test]
fn reads_the_round_two_payslips() {
    for (text, country, month, gross, ss, tax, net) in ROUND_TWO {
        let style = payslip::style_of(text);
        assert!(
            matches!(style, Style::Other(s) if s.country == *country),
            "{country}: {style:?}"
        );
        assert!(payslip::looks_like(text), "{country}");
        let p = payslip::parse(text);
        assert_eq!(p.reference_month.as_deref(), Some(*month), "{country}");
        assert_eq!((p.gross, p.net), (*gross, *net), "{country}: {:?}", p.items);
        assert_eq!(
            (p.social_security, p.income_tax),
            (*ss, *tax),
            "{country}: {:?}",
            p.items
        );
        assert_eq!(p.employer_cnpj, None);
    }
    // The others stay the same.
    assert_eq!(payslip::style_of(PAYSTUB), Style::Us);
    assert_eq!(payslip::style_of(RECIBO), Style::Pt);
    assert_eq!(payslip::style_of(NOMINA), Style::Es);
}
