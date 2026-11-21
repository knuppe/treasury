use chrono::Datelike;
use serde_json::{Value, json};

use super::dataset;
use super::mapping::{self, GroupKind};
use crate::api::Api;
use crate::paths::TreasuryPaths;

fn api() -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking", "opening_date": "2024-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, acc)
}

fn cat(api: &Api, name: &str) -> String {
    api.call("categories.list", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("categoria {name}"))["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn i(v: &Value, ptr: &str) -> i64 {
    v.pointer(ptr)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("{ptr} em {v}"))
}

#[test]
fn datasets_load_with_sources_for_every_number() {
    let all = dataset::all().unwrap();
    assert_eq!(
        all.iter().map(|d| d.country.as_str()).collect::<Vec<_>>(),
        ["BR", "US", "EU"]
    );
    let br = dataset::get("br").unwrap();
    assert_eq!(br.coverage, "completo");
    assert!(
        br.payroll
            .iter()
            .any(|p| p.from == "2026-01" && p.reducer.is_some())
    );
    // Toda categoria do conjunto inicial cai num grupo que o Brasil conhece.
    for (name, _, children) in crate::categories::seed_expenses() {
        for n in std::iter::once(name).chain(children.iter()) {
            let g = mapping::default_group(n).unwrap();
            assert!(
                mapping::kind_of(g) != GroupKind::Consumption
                    || br.consumption.iter().any(|c| c.key == g),
                "{n} → {g}"
            );
        }
    }
    // Os três países usam os mesmos grupos.
    for d in &all {
        assert_eq!(d.consumption.len(), br.consumption.len(), "{}", d.country);
    }
}

#[test]
fn estimate_combines_consumption_direct_taxes_and_payroll() {
    let (_d, api, acc) = api();
    let year = 2025;
    let spend = |name: &str, cents: i64, date: &str| {
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": date, "amount": -cents, "description": name,
                   "category_id": cat(&api, name)}),
        )
        .unwrap();
    };
    spend("Mercado", 100_000, "2025-03-10");
    spend("Restaurantes", 50_000, "2025-03-11");
    spend("Combustível", 30_000, "2025-03-12");
    spend("IPTU", 20_000, "2025-02-10");
    spend("Plano de saúde", 40_000, "2025-03-05");
    spend("Aportes em investimentos", 90_000, "2025-03-06");
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2025-03-20", "amount": -10_000, "description": "?"}),
    )
    .unwrap();
    let salario = cat(&api, "Salário");
    for m in ["2025-06-05", "2025-07-05", "2025-08-05"] {
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": m, "amount": 450_000, "description": "Salário",
                   "category_id": salario}),
        )
        .unwrap();
    }

    let e = api
        .call("tax_burden.estimate", json!({"year": year}))
        .unwrap();
    assert_eq!(e["dataset"]["country"], "BR");
    let br = dataset::get("BR").unwrap();
    let rate = |k: &str| match br.consumption.iter().find(|g| g.key == k).unwrap().central {
        Some(dataset::RateSpec::Percent(p)) => p,
        _ => panic!("{k}"),
    };
    let expected = [
        ("mercado", 100_000),
        ("restaurantes", 50_000),
        ("gasolina", 30_000),
    ]
    .iter()
    .map(|(k, v)| (*v as f64 * rate(k) / 100.0).round() as i64)
    .sum::<i64>();
    let consumption = e["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kind"] == "consumption")
        .unwrap();
    assert_eq!(i(consumption, "/amount/probable"), expected);
    assert!(i(consumption, "/amount/conservative") < expected);
    assert!(i(consumption, "/amount/high") > expected);
    assert_eq!(i(&e, "/consumption_spent"), 180_000);

    // IPTU lançado entra pelo valor; plano de saúde, aporte e sem categoria ficam de fora.
    let property = e["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kind"] == "property")
        .unwrap();
    assert_eq!(i(property, "/amount/probable"), 20_000);
    let reasons: Vec<(&str, i64)> = e["unestimated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| (u["reason"].as_str().unwrap(), u["amount"].as_i64().unwrap()))
        .collect();
    assert!(reasons.contains(&("no_rate", 40_000)), "{reasons:?}");
    assert!(
        reasons.contains(&("not_consumption", 90_000)),
        "{reasons:?}"
    );
    assert!(reasons.contains(&("uncategorized", 10_000)), "{reasons:?}");

    // Salário líquido de R$ 4.500: bruto estimado pelas tabelas de 2025.
    let p = &e["payroll"][0];
    assert_eq!(p["origin"], "estimated");
    assert_eq!(i(p, "/net"), 1_350_000);
    assert_eq!(i(p, "/months"), 3);
    let rules = br.payroll.iter().find(|r| r.from == "2025-05").unwrap();
    let g = super::rules::gross_from_net(450_000, 0, rules);
    let (inss, ir) = super::rules::payroll(g, 0, rules);
    assert_eq!(i(p, "/gross"), g * 3);
    assert_eq!(i(p, "/social_security"), inss * 3);
    assert_eq!(i(p, "/income_tax"), ir * 3);
    let total = i(&e, "/totals/probable");
    assert_eq!(total, expected + 20_000 + inss * 3 + ir * 3);
    assert!(e["share_of_income"]["probable"].as_f64().unwrap() > 0.0);

    // O informe de rendimentos substitui a estimativa.
    api.call(
        "tax_burden.save_income_report",
        json!({"year": 2025, "payer": "Empresa X", "gross": 1_600_000, "social_security": 150_000,
               "income_tax": 120_000}),
    )
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": year}))
        .unwrap();
    assert_eq!(e["payroll"][0]["origin"], "reported");
    assert_eq!(i(&e, "/payroll/0/income_tax"), 120_000);
}

#[test]
fn own_rates_groups_and_region_change_the_estimate() {
    let (_d, api, acc) = api();
    let mercado = cat(&api, "Mercado");
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2025-03-10", "amount": -100_000, "description": "M",
               "category_id": mercado}),
    )
    .unwrap();
    // Alíquota própria: vale nos três cenários.
    let cats = api
        .call(
            "tax_burden.set_category",
            json!({"category_id": mercado, "rate": 10.0}),
        )
        .unwrap();
    let row = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["category_id"] == mercado.as_str())
        .unwrap();
    assert_eq!(row["origin"], "default");
    assert_eq!(row["rate"], 10.0);
    let e = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    assert_eq!(i(&e, "/consumption/0/taxes/conservative"), 10_000);
    assert_eq!(i(&e, "/consumption/0/taxes/high"), 10_000);
    // Grupo trocado para fora do consumo: sai da estimativa.
    api.call(
        "tax_burden.set_category",
        json!({"category_id": mercado, "group_key": "financial", "rate": null}),
    )
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    assert!(e["consumption"].as_array().unwrap().is_empty());
    assert!(
        api.call(
            "tax_burden.set_category",
            json!({"category_id": mercado, "group_key": "nao-existe"}),
        )
        .is_err()
    );
    // Voltar ao padrão apaga a escolha.
    api.call(
        "tax_burden.set_category",
        json!({"category_id": mercado, "group_key": null}),
    )
    .unwrap();

    // EUA: imposto sobre vendas do estado, por fora do preço.
    api.call(
        "tax_burden.set_settings",
        json!({"country": "US", "region": "CA"}),
    )
    .unwrap();
    let us = dataset::get("US").unwrap();
    let ca = us
        .regions
        .iter()
        .find(|r| r.code == "CA")
        .unwrap()
        .sales_tax
        .unwrap();
    let restaurante = cat(&api, "Restaurantes");
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2025-04-10", "amount": -50_000, "description": "R",
               "category_id": restaurante}),
    )
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    let line = e["consumption"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["group_key"] == "restaurantes")
        .unwrap();
    let share = ca / (100.0 + ca) * 100.0;
    assert!((line["rates"]["probable"].as_f64().unwrap() - share).abs() < 1e-9);
    // Mercado isento no cenário provável, tributado no elevado.
    let m = e["consumption"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["group_key"] == "mercado")
        .unwrap();
    assert_eq!(i(m, "/taxes/probable"), 0);
    assert!(i(m, "/taxes/high") > 0);
    assert!(
        api.call("tax_burden.set_settings", json!({"region": "ZZ"}))
            .is_err()
    );
}

#[test]
fn income_report_validation_and_history() {
    let (_d, api, _acc) = api();
    assert!(
        api.call(
            "tax_burden.save_income_report",
            json!({"year": 2025, "payer": " ", "gross": 1}),
        )
        .is_err()
    );
    assert!(
        api.call(
            "tax_burden.save_income_report",
            json!({"year": 2025, "payer": "X", "gross": -1}),
        )
        .is_err()
    );
    let list = api
        .call(
            "tax_burden.save_income_report",
            json!({"year": 2025, "payer": "X", "gross": 100}),
        )
        .unwrap();
    let id = list[0]["id"].as_str().unwrap().to_string();
    let r = api
        .call("tax_burden.delete_income_report", json!({"id": id}))
        .unwrap();
    assert!(r.is_null());
    assert!(
        api.call("tax_burden.income_reports", json!({"year": 2025}))
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

/// Os percentuais de consumo são nacionais (Contas Nacionais do IBGE): o estado não os muda.
#[test]
fn the_state_does_not_change_consumption_rates() {
    let (_d, api, acc) = api();
    for (name, cents) in [("Vestuário", 100_000), ("Restaurantes", 100_000)] {
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": "2025-05-10", "amount": -cents, "description": name,
                   "category_id": cat(&api, name)}),
        )
        .unwrap();
    }
    let rate = |e: &Value, group: &str| {
        e["consumption"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["group_key"] == group)
            .unwrap()["rates"]["probable"]
            .as_f64()
            .unwrap()
    };
    let national = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    assert!(
        national["notes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().contains("estado (UF)"))
    );
    api.call("tax_burden.set_settings", json!({"region": "RJ"}))
        .unwrap();
    let rj = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    let br = dataset::get("BR").unwrap();
    assert!(br.reference_icms.is_none() && br.consumption.iter().all(|g| !g.icms_modal));
    assert_eq!(rate(&rj, "vestuario"), rate(&national, "vestuario"));
    assert_eq!(rate(&rj, "restaurantes"), rate(&national, "restaurantes"));
    assert_eq!(rj["region_name"], "Rio de Janeiro");
}

#[test]
fn ipva_is_estimated_by_vehicle_value_unless_paid_or_exempt_by_age() {
    let (_d, api, acc) = api();
    api.call("tax_burden.set_settings", json!({"region": "SP"}))
        .unwrap();
    let car = api
        .call(
            "physical.create",
            json!({"name": "Onix", "kind": "vehicle", "purchase_date": "2022-03-01",
                   "purchase_price": 8_000_000, "valuation_method": "none"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let year = crate::time::today_local().year();
    let e = api
        .call("tax_burden.estimate", json!({"year": year}))
        .unwrap();
    let line = &e["property"][0];
    let value = i(line, "/value");
    assert_eq!(line["rate"], 4.0);
    assert_eq!(i(line, "/amount"), (value as f64 * 0.04).round() as i64);
    // Fabricado há 20 anos ou mais: imune desde 2026 (EC 137/2025).
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE physical_asset SET fipe_year_code = ?2 WHERE id = ?1",
            rusqlite::params![car, format!("{}-1", year - 21)],
        )?;
        Ok(())
    })
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": year}))
        .unwrap();
    assert!(e["property"].as_array().unwrap().is_empty());
    // IPVA lançado: vale o lançado, sem estimativa.
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE physical_asset SET fipe_year_code = NULL WHERE id = ?1",
            [&car],
        )?;
        Ok(())
    })
    .unwrap();
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": format!("{year}-01-15"), "amount": -150_000,
               "description": "IPVA", "category_id": cat(&api, "IPVA e licenciamento")}),
    )
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": year}))
        .unwrap();
    assert!(e["property"].as_array().unwrap().is_empty());
    let property = e["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kind"] == "property")
        .unwrap();
    assert_eq!(i(property, "/amount/probable"), 150_000);
}

#[test]
fn thirteenth_salary_is_taxed_apart() {
    let (_d, api, acc) = api();
    let id = cat(&api, "13º salário");
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2025-12-15", "amount": 300_000, "description": "13º",
               "category_id": id}),
    )
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    let p = &e["payroll"][0];
    assert_eq!(p["origin"], "estimated");
    assert_eq!(i(p, "/net"), 300_000);
    assert_eq!(i(p, "/months"), 0, "o 13º não conta como mês de salário");
    assert!(i(p, "/gross") > 300_000);
    let cats = api.call("tax_burden.categories", json!({})).unwrap();
    let mercado = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["path"].as_str().unwrap().ends_with("Mercado"))
        .unwrap();
    assert_eq!(mercado["default_group_key"], "mercado");
}

#[test]
fn payroll_categories_include_overtime_and_commissions() {
    for name in ["Salário", "Férias", "Horas extras", "Comissões"] {
        assert_eq!(super::wage_kind(name), Some("monthly"), "{name}");
    }
    assert_eq!(super::wage_kind("13º salário"), Some("thirteenth"));
    // Rescisão (parte indenizatória isenta) e PLR (tabela própria) ficam de fora.
    for name in ["Rescisão", "PLR e bônus", "Bicos e diárias"] {
        assert_eq!(super::wage_kind(name), None, "{name}");
    }
}

/// Os textos dos dados de cada país têm a tradução no catálogo, com o pt-BR igual ao do arquivo
/// (dados atualizados sem atualizar `nucleo-impostos.ftl` voltam a aparecer só em pt-BR).
#[test]
fn dataset_texts_are_in_the_catalog() {
    use crate::i18n::{Locale, tr};
    let mut stale = vec![];
    for d in dataset::all().unwrap() {
        let c = d.country.to_lowercase();
        let key = |what: &str| format!("nucleo-impostos-dados-{c}-{what}");
        let mut texts = vec![
            (key("nome"), d.name.clone()),
            (key("aviso"), d.disclaimer.clone()),
        ];
        for (i, m) in d.methodology.iter().enumerate() {
            texts.push((key(&format!("metodo-{}", i + 1)), m.clone()));
        }
        if let Some(x) = &d.context {
            texts.push((key("contexto"), x.label.clone()));
        }
        if let Some(w) = &d.work_days {
            texts.push((key("dias"), w.label.clone()));
        }
        for g in &d.consumption {
            texts.push((
                format!("nucleo-impostos-grupo-{}", g.key.replace('_', "-")),
                g.label.clone(),
            ));
        }
        for (k, text) in texts {
            if tr(Locale::PtBr, &k, None) != text {
                stale.push(k);
            }
        }
    }
    assert!(
        stale.is_empty(),
        "textos dos dados fora do catálogo: {stale:?}"
    );
}

#[test]
fn estimate_and_data_follow_the_locale() {
    use crate::i18n::{Locale, set_current};
    let (_d, api, acc) = api();
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2025-03-10", "amount": -100_000, "description": "Mercado",
               "category_id": cat(&api, "Mercado")}),
    )
    .unwrap();
    set_current(Locale::En);
    let e = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    set_current(Locale::PtBr);
    assert_eq!(e["dataset"]["name"], "Brazil");
    let consumption = e["consumption"].as_array().unwrap();
    assert_eq!(consumption[0]["group_label"], "Groceries (mixed basket)");
    let parts = e["parts"].as_array().unwrap();
    assert_eq!(parts[0]["label"], "Consumption (built into prices)");
    assert!(
        e["notes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().starts_with("Choose the state (UF)")),
        "{}",
        e["notes"]
    );
    let br = dataset::get("BR").unwrap();
    assert_eq!(br.name, "Brasil", "em pt-BR, o texto do arquivo");
}

/// Os cinco exemplos oficiais da Receita para a Lei 15.270/2025 (IRRF de janeiro de 2026), em
/// `tests/fixtures/ir-br/irrf-mensal-2026.json`, com a fonte no próprio arquivo.
#[test]
fn irrf_2026_matches_the_receita_examples_for_lei_15270() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ir-br/irrf-mensal-2026.json");
    let fx: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let month = fx["mes"].as_str().unwrap();
    let br = dataset::get("br").unwrap();
    let r = br
        .payroll
        .iter()
        .filter(|p| p.from.as_str() <= month)
        .max_by(|a, b| a.from.cmp(&b.from))
        .unwrap();
    let cases = fx["casos"].as_array().unwrap();
    assert_eq!(cases.len(), 5);
    for c in cases {
        let (gross, inss) = (i(c, "/bruto"), i(c, "/inss"));
        assert_eq!(
            super::rules::monthly_irrf(gross, inss, 0, r),
            i(c, "/irrf"),
            "bruto {gross}"
        );
    }
}

/// Para onde vai o imposto: o total estimado repartido pelas dez funções da despesa do governo
/// geral (Tesouro Nacional, 2024); a soma de cada cenário fecha com o total.
#[test]
fn destination_splits_the_estimated_tax_by_government_function() {
    let (_d, api, acc) = api();
    // Sem imposto estimado, nada a repartir.
    let empty = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    assert!(empty["destination"].is_null());
    for (name, cents) in [("Vestuário", 100_000), ("Restaurantes", 60_000)] {
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": "2025-05-10", "amount": -cents, "description": name,
                   "category_id": cat(&api, name)}),
        )
        .unwrap();
    }
    let e = api
        .call("tax_burden.estimate", json!({"year": 2025}))
        .unwrap();
    let d = &e["destination"];
    assert_eq!(d["year"], 2024);
    assert_eq!(d["source_ids"][0], "tesouro_cofog_2024");
    let lines = d["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 10);
    for scenario in ["conservative", "probable", "high"] {
        let sum: i64 = lines
            .iter()
            .map(|l| i(l, &format!("/amounts/{scenario}")))
            .sum();
        assert_eq!(sum, i(&e, &format!("/totals/{scenario}")), "{scenario}");
    }
    let social = lines
        .iter()
        .find(|l| l["key"] == "protecao_social")
        .unwrap();
    assert_eq!(social["share"], 36.11);
    // Vestuário 14,70% de 1.000 e restaurantes 9,56% de 600: 204,36 no total, 36,11% disso.
    assert_eq!(i(&e, "/totals/probable"), 20_436);
    assert!((i(social, "/amounts/probable") - 7_379).abs() <= 1);
}

/// O que é imposto do Brasil só existe em cofre do Brasil: fora dele, os dados do Brasil não
/// aparecem na lista, não podem ser escolhidos e a estimativa não traz os blocos de lá.
#[test]
fn brazil_tax_data_is_only_offered_to_families_in_brazil() {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"], "country": "US"}),
    )
    .unwrap();
    let list = api.call("tax_burden.datasets", json!({})).unwrap();
    let countries: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["country"].as_str().unwrap())
        .collect();
    assert_eq!(countries, ["US", "EU"]);
    assert!(
        api.call("tax_burden.dataset", json!({"country": "BR"}))
            .is_err()
    );
    assert!(
        api.call("tax_burden.set_settings", json!({"country": "BR"}))
            .is_err()
    );
    let e = api
        .call("tax_burden.estimate", json!({"year": 2026}))
        .unwrap();
    assert_eq!(e["dataset"]["country"], "US");
    assert!(e["destination"].is_null() && e["work_days_reference"].is_null());
    // No Brasil, os três aparecem.
    let (_d, br, _) = self::api();
    assert_eq!(
        br.call("tax_burden.datasets", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

/// A cidade da família (Brasil): só a pessoa escolhe, com o código do IBGE; fora do Brasil não
/// existe. A leitura devolve o que está guardado, sem ir à rede.
#[test]
fn the_family_city_is_set_by_the_person_and_only_in_brazil() {
    let (_d, api, _) = api();
    let s = api.call("tax_burden.city", json!({})).unwrap();
    assert!(s["city"].is_null() && s["revenue"].is_null());
    assert_eq!(s["source_enabled"], true);
    let s = api
        .call(
            "tax_burden.set_city",
            json!({"city": {"code": "9999999", "name": "Vila Exemplo", "uf": "sp"}}),
        )
        .unwrap();
    assert_eq!(s["city"]["uf"], "SP");
    assert!(
        api.call(
            "tax_burden.set_city",
            json!({"city": {"code": "12", "name": "X", "uf": "SP"}})
        )
        .is_err()
    );
    // Um cliente MCP lê, mas não escolhe a cidade nem dispara a coleta.
    let mcp = crate::api::Actor::Mcp {
        client: "teste".into(),
        read_only: false,
    };
    assert!(api.call_as(&mcp, "tax_burden.city", json!({})).is_ok());
    assert!(
        api.call_as(&mcp, "tax_burden.set_city", json!({"city": null}))
            .is_err()
    );
    assert!(
        api.call_as(&mcp, "tax_burden.refresh_city", json!({}))
            .is_err()
    );
    // Fonte desligada: a coleta diz isso, sem ir à rede.
    api.call("sources.set", json!({"id": "siconfi", "enabled": false}))
        .unwrap();
    assert!(api.call("tax_burden.refresh_city", json!({})).is_err());
    assert_eq!(
        api.call("tax_burden.city", json!({})).unwrap()["source_enabled"],
        false
    );
    api.call("tax_burden.set_city", json!({"city": null}))
        .unwrap();
    assert!(api.call("tax_burden.city", json!({})).unwrap()["city"].is_null());
}
