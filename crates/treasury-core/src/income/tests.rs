use serde_json::json;

use super::payslip::{self, PayslipKind};
use crate::api::Api;
use crate::paths::TreasuryPaths;

pub(super) const HOLERITE: &str = "\
EMPRESA EXEMPLO LTDA CNPJ: 12.345.678/0001-90
RECIBO DE PAGAMENTO DE SALÁRIO
Referência: Setembro/2026
Funcionário: FULANO DE TAL  CPF 123.456.789-00
Cód Descrição Ref. Vencimentos Descontos
001 SALARIO BASE 30,00 5.000,00
150 HORAS EXTRAS 50% 10,00 340,91
903 INSS 11,68 584,05
904 IRRF 7,50 263,67
910 VALE TRANSPORTE 6,00 300,00
920 PLANO DE SAUDE 150,00
Total de Vencimentos 5.340,91
Total de Descontos 1.297,72
Líquido a Receber 4.043,19
Salário Base 5.000,00  Base Cálc. INSS 5.340,91  FGTS do Mês 427,27
";

#[test]
fn reads_the_generic_payslip() {
    assert!(payslip::looks_like(HOLERITE));
    let p = payslip::parse(HOLERITE);
    assert_eq!(p.reference_month.as_deref(), Some("2026-09"));
    assert_eq!(p.kind, PayslipKind::Monthly);
    assert_eq!(p.employer_cnpj.as_deref(), Some("12345678000190"));
    assert_eq!(p.employer.as_deref(), Some("EMPRESA EXEMPLO LTDA"));
    assert_eq!((p.gross, p.net), (534_091, 404_319));
    assert_eq!((p.social_security, p.income_tax), (58_405, 26_367));
    assert_eq!(p.other_deductions, 45_000);
    assert_eq!(p.fgts, Some(42_727));
    assert_eq!(p.base_salary, Some(500_000));
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    // Nada do funcionário nas linhas.
    assert!(p.items.iter().all(|i| !i.description.contains("FULANO")));
    assert_eq!(p.items.len(), 6);
    assert!(payslip::matches_generic(HOLERITE));
    let thirteenth = HOLERITE.replace(
        "RECIBO DE PAGAMENTO DE SALÁRIO",
        "RECIBO DE PAGAMENTO - 13º SALÁRIO 2ª PARCELA",
    );
    assert_eq!(payslip::parse(&thirteenth).kind, PayslipKind::Thirteenth2);
}

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana", "Beto"]}),
    )
    .unwrap();
    (dir, api)
}

fn member(api: &Api, name: &str) -> String {
    api.call("household.list_members", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == name)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn sources_payslips_and_projection() {
    let (_d, api) = api();
    let ana = member(&api, "Ana");
    let beto = member(&api, "Beto");
    // Cadastro do membro.
    let m = api
        .call(
            "household.update_member",
            json!({"id": ana, "profile": {"birth_date": "1990-05-10", "relationship": "titular", "occupation": "Engenheira"}}),
        )
        .unwrap();
    assert_eq!(m["profile"]["occupation"], "Engenheira");
    assert_eq!(m["profile"]["relationship"], "titular");
    // Renda informal do Beto, sem holerite.
    api.call(
        "income.create_source",
        json!({"member_id": beto, "name": "Bicos de pintura", "kind": "informal", "net_monthly": 150_000}),
    )
    .unwrap();
    // Conta e o salário líquido de outubro, para ligar ao holerite de setembro.
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking", "opening_date": "2026-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let salary = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-05", "amount": 404_319, "description": "SALARIO EMPRESA EXEMPLO"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // Prévia sem renda: pede o membro; com o membro, cria a renda do empregador.
    let p = api
        .call("income.payslip_preview", json!({"text": HOLERITE}))
        .unwrap();
    assert!(p["source_id"].is_null());
    assert!(p["warnings"].to_string().contains("escolha a renda"));
    assert_eq!(p["layout"]["id"], "payslip.generico.v1");
    let r = api
        .call(
            "income.payslip_import",
            json!({"text": HOLERITE, "member_id": ana}),
        )
        .unwrap();
    assert_eq!(r["created_source"], true);
    assert_eq!(r["txn_id"], salary.as_str());
    // A segunda vez: a mesma renda pelo CNPJ, e já existe.
    let p = api
        .call("income.payslip_preview", json!({"text": HOLERITE}))
        .unwrap();
    assert_eq!(p["source_id"], r["source_id"]);
    assert_eq!(p["duplicate"], true);
    assert!(
        api.call("income.payslip_import", json!({"text": HOLERITE}))
            .is_err()
    );
    let src = api
        .call("income.list_sources", json!({"member_id": ana}))
        .unwrap();
    assert_eq!(src[0]["gross_monthly"], 534_091);
    assert_eq!(src[0]["net_monthly"], 404_319);
    assert_eq!(src[0]["employer_cnpj"], "12345678000190");
    assert_eq!(src[0]["has_thirteenth"], true);
    // Resumo: previsão de 12 meses com 13º em novembro e dezembro.
    let s = api
        .with_session(|s, _| {
            super::summary(
                s.db.conn(),
                &super::IncomeSummaryArgs {
                    months: Some(12),
                    member_id: None,
                },
                crate::time::parse_date("2026-10-15").unwrap(),
            )
        })
        .unwrap();
    let ana_inc = s.members.iter().find(|m| m.member_name == "Ana").unwrap();
    let proj = &ana_inc.projection[0];
    assert_eq!(proj.basis, "payslip");
    let nov = &proj.months[0];
    assert_eq!(nov.month, "2026-11");
    assert_eq!(nov.gross, 534_091 + 534_091 / 2);
    assert_eq!(nov.net, 404_319 + 534_091 / 2);
    let dec = &proj.months[1];
    assert!(dec.extras.iter().any(|e| e.contains("13º")));
    assert!(dec.income_tax > 26_367 && dec.social_security > 58_405);
    let jan = &proj.months[2];
    assert_eq!((jan.gross, jan.net), (534_091, 404_319));
    assert_eq!(ana_inc.last_12m.gross, 534_091);
    assert!((ana_inc.effective_tax_rate.unwrap() - (58_405.0 + 26_367.0) / 534_091.0).abs() < 1e-9);
    let beto_inc = s.members.iter().find(|m| m.member_name == "Beto").unwrap();
    assert_eq!(beto_inc.projection[0].basis, "declared");
    assert_eq!(beto_inc.projection[0].months[0].net, 150_000);
    assert_eq!(beto_inc.projection[0].months[0].income_tax, 0);
    assert_eq!(s.monthly[2].net, 404_319 + 150_000);
    // CLT só com o bruto: INSS e IRRF pelas regras do ano.
    let c = api
        .call(
            "income.create_source",
            json!({"member_id": beto, "name": "Emprego novo", "kind": "clt", "gross_monthly": 300_000, "start_date": "2027-01-01"}),
        )
        .unwrap();
    assert_eq!(c["has_thirteenth"], true);
    let s = api
        .with_session(|s, _| {
            super::summary(
                s.db.conn(),
                &super::IncomeSummaryArgs {
                    months: Some(4),
                    member_id: Some(beto.clone()),
                },
                crate::time::parse_date("2026-10-15").unwrap(),
            )
        })
        .unwrap();
    let novo = s.members[0]
        .projection
        .iter()
        .find(|p| p.name == "Emprego novo")
        .unwrap();
    assert_eq!(novo.months[0].gross, 0, "começa em janeiro");
    let jan = &novo.months[2];
    assert_eq!(jan.gross, 300_000);
    assert!(jan.social_security > 0);
    assert_eq!(jan.net, 300_000 - jan.social_security - jan.income_tax);
    // Validação.
    assert!(api
        .call(
            "income.create_source",
            json!({"member_id": beto, "name": "x", "kind": "clt", "gross_monthly": 100, "net_monthly": 200}),
        )
        .is_err());
}

/// Os meses passados com holerite aparecem como "real", ao lado do que a previsão de hoje
/// (pelo último holerite) daria para o mesmo mês.
#[test]
fn summary_shows_past_payslips_next_to_the_forecast() {
    let (_d, api) = api();
    let ana = member(&api, "Ana");
    api.call(
        "income.payslip_import",
        json!({"text": HOLERITE, "member_id": ana}),
    )
    .unwrap();
    // Julho com salário menor (antes do aumento): o mesmo empregador, outra competência.
    let july = HOLERITE
        .replace("Setembro/2026", "Julho/2026")
        .replace("5.340,91", "4.340,91")
        .replace("Líquido a Receber 4.043,19", "Líquido a Receber 3.043,19");
    let r = api.call("income.payslip_import", json!({"text": july}));
    assert!(r.is_ok(), "{r:?}");
    let s = api
        .with_session(|s, _| {
            super::summary(
                s.db.conn(),
                &super::IncomeSummaryArgs {
                    months: Some(12),
                    member_id: None,
                },
                crate::time::parse_date("2026-10-15").unwrap(),
            )
        })
        .unwrap();
    assert_eq!(s.forecast_base_month.as_deref(), Some("2026-09"));
    let months: Vec<&str> = s.actual.iter().map(|a| a.real.month.as_str()).collect();
    assert_eq!(months, ["2026-07", "2026-09"]);
    let jul = &s.actual[0];
    assert_eq!(jul.real.net, 304_319);
    // A previsão de hoje para julho usa o holerite de setembro.
    assert_eq!(jul.expected.net, 404_319);
    let sep = &s.actual[1];
    assert_eq!(sep.real.net, sep.expected.net);
    assert_eq!(sep.real.gross, 534_091);
    // Outubro (o mês de hoje) ainda sem holerite: já vem previsto, fora dos próximos 12 meses.
    let cur = s.current_month.as_ref().expect("mês corrente previsto");
    assert_eq!(cur.month, "2026-10");
    assert_eq!(cur.gross, 534_091);
    assert_eq!(s.monthly[0].month, "2026-11");
    assert_eq!(s.monthly.len(), 12);
    // Com o holerite do mês corrente, não há previsão para ele.
    let s = api
        .with_session(|s, _| {
            super::summary(
                s.db.conn(),
                &super::IncomeSummaryArgs {
                    months: Some(12),
                    member_id: None,
                },
                crate::time::parse_date("2026-09-20").unwrap(),
            )
        })
        .unwrap();
    assert!(s.current_month.is_none(), "{:?}", s.current_month);
}
