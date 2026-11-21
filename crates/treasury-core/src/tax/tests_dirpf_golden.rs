//! Reference cases for the annual adjustment return (IRPF 2026), copied from the Imposto de
//! Renda MCP (AGPL-3.0-only; attribution in `tests/fixtures/ir-br/dirpf-2026/README.md`).
//! Treasury does not compute the annual adjustment yet: here we only check that the cases load
//! and are consistent, so they can serve as a target when the calculation exists.

use std::path::Path;

use serde_json::Value;

/// Simplified discount cap for exercise 2026 (calendar year 2025), in cents.
const SIMPLIFIED_CAP_2026: i64 = 1_675_434;

fn cases() -> Vec<Value> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ir-br/dirpf-2026");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("cases folder")
        .map(|e| e.expect("file").path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
        .iter()
        .flat_map(|p| {
            let v: Value = serde_json::from_str(&std::fs::read_to_string(p).expect("read"))
                .unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            match v {
                Value::Array(a) => a,
                v => vec![v],
            }
        })
        .collect()
}

fn cents(c: &Value, field: &str) -> i64 {
    c["esperado"][field]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("{}: {field}", c["id"]))
}

#[test]
fn dirpf_2026_reference_cases_load_and_agree_with_themselves() {
    let all = cases();
    assert_eq!(all.len(), 65);
    for c in &all {
        let income = cents(c, "rendimentosTributaveis");
        let full = cents(c, "impostoDevidoCompleto");
        let simple = cents(c, "impostoDevidoSimplificado");
        let id = &c["id"];
        assert_eq!(
            cents(c, "baseCalculoCompleto"),
            (income - cents(c, "deducoesCompleto")).max(0),
            "{id}"
        );
        assert_eq!(
            cents(c, "descontoSimplificado"),
            (income * 20 / 100).min(SIMPLIFIED_CAP_2026),
            "{id}"
        );
        assert_eq!(
            cents(c, "baseCalculoSimplificado"),
            (income - cents(c, "descontoSimplificado")).max(0),
            "{id}"
        );
        let best = c["esperado"]["modeloMaisVantajoso"]
            .as_str()
            .unwrap_or_default();
        if full != simple {
            let expected = if full < simple {
                "completo"
            } else {
                "simplificado"
            };
            assert_eq!(best, expected, "{id}");
        }
    }
}

/// Builds the annual calculation input from a case; `None` when the case uses something the
/// calculation does not cover (rural activity).
fn input_of(c: &Value) -> Option<super::annual::Input> {
    let e = &c["entrada"];
    if e.get("atividadeRural").is_some() || e.get("livroCaixaRural").is_some() {
        return None;
    }
    let n = |v: &Value| -> i64 { v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0) };
    let mut input = super::annual::Input {
        year: 2025,
        ..Default::default()
    };
    for pj in e["rendimentosPJ"].as_array().into_iter().flatten() {
        input.employment.push(super::annual::Employment {
            income: n(&pj["rendimento"]),
            social_security: n(&pj["previdenciaOficial"]),
            withheld: n(&pj["irrf"]),
        });
    }
    for m in e["rendimentosPF"]["meses"].as_array().into_iter().flatten() {
        let se = &mut input.self_employed;
        se.income += n(&m["pessoaFisica"]) + n(&m["exterior"]);
        se.abroad += n(&m["exterior"]);
        se.social_security += n(&m["previdenciaOficial"]);
        se.alimony += n(&m["pensao"]);
        se.livro_caixa += n(&m["livroCaixa"]);
        se.carne_leao += n(&m["carneLeao"]);
    }
    input.dependents = e["dependentes"].as_array().map_or(0, |d| d.len() as u32);
    // Education per person: holder ("T") and each dependent by CPF.
    let mut education: std::collections::BTreeMap<String, i64> = Default::default();
    for p in e["pagamentos"].as_array().into_iter().flatten() {
        let code: u32 = p["codigo"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let value = n(&p["valor"]) - n(&p["parcelaNaoDedutivel"]);
        match code {
            1 | 2 => {
                let who = p["cpfDependente"].as_str().unwrap_or("T").to_string();
                *education.entry(who).or_default() += value;
            }
            9..=29 => input.medical += value,
            30..=34 => input.alimony += value,
            36 | 37 => input.private_pension += value,
            other => panic!("{}: payment with code {other} not mapped", c["id"]),
        }
    }
    input.education = education.into_values().collect();
    input.donation_children = n(&e["doacoesDiretas"]["eca"]);
    input.donation_elderly = n(&e["doacoesDiretas"]["idoso"]);
    input.complementary_tax = n(&e["impostoPago"]["complementar"]);
    input.foreign_tax_paid = n(&e["impostoPago"]["exterior"]);
    input.irrf_lei_11033 = n(&e["impostoPago"]["irrfLei11033"]);
    Some(input)
}

/// The annual calculation reproduces, to the cent, the official program's summary (IRPF 2026) in
/// the reference cases it covers (all, except the rural activity ones).
#[test]
fn annual_adjustment_matches_the_official_program_cases() {
    let mut checked = 0;
    let mut failures = vec![];
    for c in cases() {
        let Some(input) = input_of(&c) else { continue };
        let out = super::annual::compute(&input);
        let got = [
            ("rendimentosTributaveis", out.taxable_income),
            ("deducoesCompleto", out.full.deductions),
            ("baseCalculoCompleto", out.full.base),
            ("impostoDevidoCompleto", out.full.tax_due),
            ("descontoSimplificado", out.simplified.deductions),
            ("baseCalculoSimplificado", out.simplified.base),
            ("impostoDevidoSimplificado", out.simplified.tax_due),
            ("impostoRetidoTotal", out.full.tax_paid),
            ("saldoPagarCompleto", out.full.to_pay),
            ("restituirCompleto", out.full.refund),
            ("saldoPagarSimplificado", out.simplified.to_pay),
            ("restituirSimplificado", out.simplified.refund),
            ("aliquotaEfetiva", out.effective_rate_bp),
        ];
        for (field, value) in got {
            let expected = cents(&c, field);
            if value != expected {
                failures.push(format!("{} {field}: {value} ≠ {expected}", c["id"]));
            }
        }
        let best = c["esperado"]["modeloMaisVantajoso"]
            .as_str()
            .unwrap_or_default();
        if out.best != best {
            failures.push(format!("{} model: {} ≠ {best}", c["id"], out.best));
        }
        checked += 1;
    }
    assert!(
        failures.is_empty(),
        "{checked} cases; failures:\n{}",
        failures.join("\n")
    );
    assert_eq!(checked, 60);
}

/// Capital gain (real estate, movable assets and holdings in Brazil) reproduces to the cent the
/// GCAP program in the reference cases: gain, Lei 7.713 reductions and FR1/FR2 with the
/// percentages, only property, small value and tax by brackets.
#[test]
fn capital_gains_match_the_official_gcap_cases() {
    use super::gcap::{AssetKind, Input, compute};
    let date = |s: &str| format!("{}-{}-{}", &s[4..8], &s[2..4], &s[0..2]);
    let n = |v: &Value| -> i64 { v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0) };
    let mut checked = 0;
    for c in cases() {
        let Some(sales) = c["entrada"]["ganhosCapital"].as_array() else {
            continue;
        };
        for (k, g) in sales.iter().enumerate() {
            let kind = match g["tipo"].as_str() {
                Some("imovel") => AssetKind::RealEstate,
                Some("movel") => AssetKind::Movable,
                Some("participacao") => AssetKind::Holding,
                _ => continue,
            };
            if g["exterior"].as_bool() == Some(true) {
                continue;
            }
            let out = compute(&Input {
                kind,
                acquired_on: date(g["dataAquisicao"].as_str().unwrap()),
                sold_on: date(g["dataAlienacao"].as_str().unwrap()),
                sale_value: n(&g["valorAlienacao"]),
                brokerage: n(&g["custoCorretagem"]),
                cost: n(&g["custoAquisicao"]),
                only_property: g["unicoImovel"].as_bool().unwrap_or(false),
                month_sales_same_kind: None,
                reinvested: 0,
            })
            .unwrap();
            let e = &c["esperadoGcap"]["alienacoes"][k];
            let id = &c["id"];
            assert_eq!(out.net_value, n(&e["valorLiquido"]), "{id}");
            assert_eq!(out.gross_gain, n(&e["ganhoBruto"]), "{id}");
            let r = &e["reducoes"];
            assert_eq!(out.reductions.lei_7713, n(&r["lei7713"]), "{id}");
            assert_eq!(out.reductions.fr1, n(&r["fr1"]), "{id}");
            assert_eq!(out.reductions.fr2, n(&r["fr2"]), "{id}");
            assert_eq!(out.reductions.only_property, n(&r["unicoImovel"]), "{id}");
            let p = &e["percentuaisReducao"];
            if kind == AssetKind::RealEstate {
                assert_eq!(out.reductions.fr2_pct.replace('.', ","), p["fr2"], "{id}");
                assert_eq!(out.reductions.fr1_pct.replace('.', ","), p["fr1"], "{id}");
            }
            assert_eq!(out.taxable_gain, n(&e["ganhoTributavel"]), "{id}");
            assert_eq!(out.tax, n(&e["impostoDevido"]), "{id}");
            checked += 1;
        }
    }
    assert_eq!(checked, 7);
}
