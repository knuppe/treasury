//! Score de renda fixa de ponta a ponta (cofre temporário): CDB de banco com IF.data, CDB de
//! emissor desconhecido, Tesouro IPCA+ com histórico de taxas e debênture sem emissor.

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::api::Api;
use crate::market::tesouro::{self, Quote, Rates};
use crate::paths::TreasuryPaths;
use crate::research::ifdata::items as it;
use crate::time::{add_months, fmt_date, today_local};

struct Env {
    _dir: tempfile::TempDir,
    api: Arc<Api>,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Arc::new(Api::new_unlocked_process(paths).unwrap());
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família Teste", "members": ["Ana", "Beto"]}),
    )
    .unwrap();
    Env { _dir: dir, api }
}

fn dec(v: f64) -> Decimal {
    Decimal::from_f64_retain(v).unwrap().round_dp(8)
}

/// Banco Exemplo (S3, CNPJ 22333444) e mais 9 bancos S3 com Basileia de 12% a 20%.
fn seed_banks(api: &Api) {
    let today = today_local();
    let q = add_months(today, -4);
    let period = format!("{}", q.format("%Y-%m"));
    api.with_session(|s, _| {
        let c = s.db.conn();
        let mut inst = c.prepare(
            "INSERT INTO bank_institution (cod_inst, name, cnpj8, tcb, segment, prudential_code, active, fetched_at)
             VALUES (?1, ?2, ?3, 'B1', 'S3', ?1, 1, 'x')",
        )?;
        let mut val = c.prepare(
            "INSERT INTO bank_prudential (cod_inst, period, item, value, source, fetched_at)
             VALUES (?1, ?2, ?3, ?4, 'bacen_ifdata', 'x')",
        )?;
        for i in 0..10 {
            let (cod, name, cnpj) = if i == 0 {
                ("C0080001".to_string(), "BANCO EXEMPLO - PRUDENCIAL".to_string(), "22333444".to_string())
            } else {
                (format!("C00800{:02}", 10 + i), format!("BANCO PAR {i} - PRUDENCIAL"), format!("{:08}", 10_000_000 + i))
            };
            inst.execute(rusqlite::params![cod, name, cnpj])?;
            let basel = 0.12 + 0.008 * i as f64;
            for (item, v) in [
                (it::BASEL_RATIO, basel),
                (it::CET1_RATIO, basel - 0.03),
                (it::LEVERAGE_RATIO, 0.07),
                (it::NPL90, 120.0),
                (it::CREDIT_TOTAL, 4000.0),
                (it::EXPECTED_LOSS, 240.0),
            ] {
                val.execute(rusqlite::params![cod, period, item, crate::research::dstr(dec(v))])?;
            }
        }
        Ok(())
    })
    .unwrap();
}

fn member_ids(api: &Api) -> Vec<String> {
    api.with_session(|s, _| {
        let mut st = s.db.conn().prepare("SELECT id FROM member ORDER BY name")?;
        let v = st
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        Ok(v)
    })
    .unwrap()
}

fn create(api: &Api, v: Value) -> String {
    api.call("assets.create", v).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn cdb(cnpj: Option<&str>, issuer: &str) -> Value {
    let today = today_local();
    json!({
        "name": format!("CDB {issuer}"),
        "class": "fixed_income",
        "cnpj": cnpj,
        "fixed_income": {
            "kind": "cdb", "indexer": "cdi", "rate": "110",
            "issue_date": fmt_date(add_months(today, -2)),
            "maturity_date": fmt_date(add_months(today, 22)),
            "liquidity": "at_maturity", "ir_exempt": false, "issuer": issuer
        }
    })
}

fn metric<'a>(s: &'a Value, key: &str) -> &'a Value {
    s["dimensions"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|d| d["metrics"].as_array().unwrap())
        .find(|m| m["key"] == key)
        .unwrap_or_else(|| panic!("métrica {key}"))
}

#[test]
fn cdb_inherits_the_bank_health_and_checks_the_fgc_limit() {
    let e = env();
    seed_banks(&e.api);
    let members = member_ids(&e.api);
    let acc = e
        .api
        .call(
            "accounts.create",
            json!({"name": "XP", "kind": "brokerage", "member_ids": members}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let id = create(&e.api, cdb(Some("22.333.444/0001-55"), "Banco Exemplo"));
    // Conta conjunta de Ana e Beto com R$ 600 mil: R$ 300 mil por CPF (limite R$ 250 mil).
    e.api
        .call(
            "portfolio.create_trade",
            json!({"asset_id": id, "account_id": acc, "date": fmt_date(add_months(today_local(), -2)),
                   "side": "buy", "quantity": "1", "price": "600000", "cash_movement": false}),
        )
        .unwrap();
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    let dump = serde_json::to_string_pretty(&s).unwrap();
    assert_eq!(s["asset_class"], "fi_bank", "{dump}");
    assert_eq!(s["segment"], "cdb");
    assert_eq!(s["status"], "ok", "{dump}");
    assert_eq!(s["sector"], "Bancos médios (S3)");
    // Basileia de 12% é a menor dos 10: percentil baixo entre os 9 pares do porte.
    let basel = metric(&s, "basel_ratio");
    assert!((basel["value"].as_f64().unwrap() - 0.12).abs() < 1e-9);
    assert_eq!(basel["peers"], 9);
    assert!(basel["percentile"].as_f64().unwrap() < 10.0, "{dump}");
    // Cobertura do FGC: a valor de curva (≈ R$ 600 mil) → 250 ÷ 300 por titular.
    let fgc = metric(&s, "fgc_coverage")["value"].as_f64().unwrap();
    assert!(fgc > 0.8 && fgc < 0.84, "{fgc}");
    let flags: Vec<&str> = s["flags"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["key"].as_str())
        .collect();
    assert!(flags.contains(&"fi_fgc_over_limit"), "{flags:?}");
    // 110% do CDI, tributado.
    assert!((metric(&s, "fi_rate_vs_cdi")["value"].as_f64().unwrap() - 1.1).abs() < 1e-9);
    assert!(s["score"].as_f64().unwrap() > 0.0);
    assert!(s["data_date"].is_string());

    // Pelo nome (sem CNPJ) também acha o banco; e a coleta do IF.data passa a pedi-lo.
    let by_name = create(&e.api, cdb(None, "Banco Exemplo S.A."));
    e.api
        .call("portfolio.watch", json!({"asset_id": by_name}))
        .unwrap();
    let s2 = e
        .api
        .call("score.get", json!({"asset_id": by_name}))
        .unwrap();
    assert_eq!(
        s2["status"],
        "ok",
        "{}",
        serde_json::to_string_pretty(&s2).unwrap()
    );
    assert!(
        s2["notes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().contains("identificado pelo nome"))
    );
    let targets = e
        .api
        .with_session(|s, _| crate::research::ifdata::target_cnpj8(s.db.conn()))
        .unwrap();
    assert!(targets.contains("22333444"));
}

#[test]
fn cdb_without_identifiable_issuer_has_no_score_and_records_feedback() {
    let e = env();
    seed_banks(&e.api);
    let id = create(&e.api, cdb(None, "Banco Que Não Existe"));
    e.api
        .call("portfolio.watch", json!({"asset_id": id}))
        .unwrap();
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["status"], "insufficient_data");
    assert!(s["score"].is_null());
    let missing = s["missing"].as_array().unwrap();
    let basel = missing
        .iter()
        .find(|x| x["key"] == "basel_ratio")
        .expect("Basileia em falta");
    assert_eq!(basel["critical"], true);
    assert!(
        basel["reason"]
            .as_str()
            .unwrap()
            .contains("não identificado")
    );
    let fb = e.api.call("devfeedback.list", json!({})).unwrap();
    assert!(fb.as_array().unwrap().iter().any(|x| {
        x["title"]
            .as_str()
            .unwrap()
            .contains("dados críticos ausentes")
    }));
}

/// 300 dias de medianas de IPCA+ entre 4% e 7% e a curva de hoje (IPCA+ 2035 a 6,84%).
fn seed_tesouro(api: &Api) -> NaiveDate {
    let today = today_local();
    let day = today - chrono::Duration::days(1);
    let mut medians = BTreeMap::new();
    for i in 0..300 {
        let v = 0.04 + 0.03 * (i as f64) / 299.0;
        medians.insert(
            ("real".to_string(), day - chrono::Duration::days(300 - i)),
            dec(v),
        );
    }
    let q = |kind: &str, mat: &str, rate: f64, pb: f64, ps: f64| Quote {
        kind: kind.into(),
        maturity: crate::time::parse_date(mat).unwrap(),
        date: day,
        rate_buy: dec(rate),
        rate_sell: dec(rate + 0.0012),
        pu_buy: dec(pb),
        pu_sell: dec(ps),
    };
    let rates = Rates {
        medians,
        curve: vec![
            q("ipca", "2029-05-15", 0.0677, 4034.07, 4021.13),
            q("ipca", "2035-05-15", 0.0684, 2713.48, 2686.72),
            q("prefixado", "2029-01-01", 0.1264, 770.85, 768.69),
        ],
    };
    api.with_session(|s, _| tesouro::store_rates(s.db.conn(), &rates, "x"))
        .unwrap();
    day
}

#[test]
fn tesouro_ipca_scores_rate_against_ten_years_and_duration() {
    let e = env();
    let day = seed_tesouro(&e.api);
    let id = create(
        &e.api,
        json!({"name": "Tesouro IPCA+ 2035", "class": "treasury_br", "treasury_code": "Tesouro IPCA+ 2035"}),
    );
    e.api
        .call("portfolio.watch", json!({"asset_id": id}))
        .unwrap();
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    let dump = serde_json::to_string_pretty(&s).unwrap();
    assert_eq!(s["asset_class"], "fi_sovereign", "{dump}");
    assert_eq!(s["segment"], "ipca");
    assert_eq!(s["status"], "ok", "{dump}");
    // 6,84% acima de ~94% das medianas de 4% a 7%.
    let pct = metric(&s, "fi_rate_percentile")["value"].as_f64().unwrap();
    assert!(pct > 90.0 && pct < 97.0, "{pct}");
    assert!((metric(&s, "fi_real_rate")["value"].as_f64().unwrap() - 0.0684).abs() < 1e-9);
    // Sem cupom: duration = prazo até 15/05/2035.
    let dur = metric(&s, "fi_duration")["value"].as_f64().unwrap();
    let years =
        (crate::time::parse_date("2035-05-15").unwrap() - today_local()).num_days() as f64 / 365.25;
    assert!((dur - years).abs() < 1e-9);
    // Custo de sair hoje = 1 − 2686,72 ÷ 2713,48.
    let exit = metric(&s, "fi_exit_cost")["value"].as_f64().unwrap();
    assert!((exit - (1.0 - 2686.72 / 2713.48)).abs() < 1e-6, "{exit}");
    assert_eq!(s["data_date"], fmt_date(day));
    // Sem preços gravados: a volatilidade falta, mas não é crítica.
    assert!(
        s["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["key"] == "fi_price_volatility" && x["critical"] == false)
    );
    assert!(s["attractiveness"].as_f64().unwrap() > 70.0);
}

#[test]
fn matured_tesouro_has_no_score_and_no_feedback() {
    let e = env();
    seed_tesouro(&e.api);
    let year = chrono::Datelike::year(&today_local());
    for (name, maturity) in [
        // Ano no nome antes do ano corrente.
        ("Tesouro Prefixado 2025".to_string(), None),
        // Ano corrente, fora da curva de hoje (já venceu).
        (format!("Tesouro IPCA+ com Juros Semestrais {year}"), None),
        // Vencimento nos termos.
        ("Tesouro IPCA+ 2024".to_string(), Some("2024-08-15")),
    ] {
        let mut v = json!({"name": name, "class": "treasury_br", "treasury_code": name});
        if let Some(m) = maturity {
            v["fixed_income"] = json!({"kind": "tesouro_ipca", "indexer": "ipca",
                "rate": "6", "issue_date": "2018-01-01", "maturity_date": m,
                "liquidity": "daily", "ir_exempt": false});
        }
        let id = create(&e.api, v);
        let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
        assert_eq!(s["status"], "not_supported", "{name}: {s}");
        assert_eq!(s["band_label"], "Título vencido", "{name}");
        assert!(s["missing"].as_array().unwrap().is_empty(), "{name}: {s}");
    }
    let fb = e.api.call("devfeedback.list", json!({})).unwrap();
    assert!(
        !fb.to_string().contains("Tesouro"),
        "título vencido não vira aviso: {fb}"
    );
    // O que está na curva segue com score.
    let id = create(
        &e.api,
        json!({"name": "Tesouro IPCA+ 2029", "class": "treasury_br", "treasury_code": "Tesouro IPCA+ 2029"}),
    );
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_ne!(s["status"], "not_supported", "{s}");
}

#[test]
fn debenture_without_issuer_is_private_credit_without_score() {
    let e = env();
    let today = today_local();
    let id = create(
        &e.api,
        json!({"name": "Debênture XYZ", "class": "fixed_income", "fixed_income": {
            "kind": "debenture_incentivada", "indexer": "ipca", "rate": "7.5",
            "maturity_date": fmt_date(add_months(today, 60)), "liquidity": "at_maturity",
            "ir_exempt": true, "issuer": "Companhia Desconhecida"}}),
    );
    e.api
        .call("portfolio.watch", json!({"asset_id": id}))
        .unwrap();
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["asset_class"], "fi_corporate");
    assert_eq!(s["segment"], "debenture_incentivada");
    assert_eq!(s["status"], "insufficient_data");
    assert!(
        s["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["key"] == "fi_issuer_health" && x["critical"] == true)
    );
    // Estrutura sem fonte aparece como indisponível (não crítica), sem feedback (lacuna
    // conhecida da metodologia); o aviso aberto por versões anteriores fecha sozinho.
    assert!(
        s["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["key"] == "fi_guarantees" && x["critical"] == false)
    );
    e.api
        .with_session(|s, _| {
            s.db.conn().execute(
                "INSERT INTO dev_feedback (id, title, body, area, kind, importance, status,
                     created_by, bumps, created_at, updated_at)
                 VALUES ('fb-antigo', 'Score de ativos: sem fonte para Garantias', 'x', 'dados',
                     'dados', 1, 'aberto', 'score de ativos', 0, '2026-10-01T00:00:00Z',
                     '2026-10-01T00:00:00Z')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    e.api.call("score.get", json!({"asset_id": id})).unwrap();
    let fb = e.api.call("devfeedback.list", json!({})).unwrap();
    let all = fb.as_array().unwrap();
    let titles: Vec<&str> = all.iter().filter_map(|x| x["title"].as_str()).collect();
    let gap = all
        .iter()
        .find(|x| x["id"] == "fb-antigo")
        .unwrap_or_else(|| panic!("{titles:?}"));
    assert_eq!(gap["status"], "resolvido", "{gap}");
    assert!(
        titles.iter().any(|t| t.contains("Debênture XYZ")),
        "{titles:?}"
    );
}

/// Emissores pelo nome da marca: "Nubank" (Nu Financeira, pelo CNPJ conhecido e conferido
/// pelo nome do IF.data) e "Credivale" (palavra única no nome da cooperativa); marca cujo CNPJ
/// não está no cadastro não casa com outro banco.
#[test]
fn bank_issuer_by_brand_and_distinctive_word() {
    let e = env();
    e.api
        .with_session(|s, _| {
            let c = s.db.conn();
            for (cod, name, cnpj) in [
                ("C0011", "NU FINANCEIRA S.A. - SOCIEDADE DE CREDITO FINANCIAMENTO E INVESTIMENTO", "30680829"),
                ("C0022", "COOPERATIVA DE CREDITO EXEMPLAR - CREDIVALE", "99887766"),
                ("C0033", "BANCO INTERESSANTE S.A.", "11223344"),
            ] {
                c.execute(
                    "INSERT INTO bank_institution (cod_inst, name, cnpj8, tcb, segment, prudential_code, active, fetched_at)
                     VALUES (?1, ?2, ?3, 'B1', 'S4', ?1, 1, 'x')",
                    rusqlite::params![cod, name, cnpj],
                )?;
            }
            let by = |t: &str| super::data::bank_by_name(c, t).unwrap().map(|i| i.cod_inst);
            assert_eq!(by("Nubank").as_deref(), Some("C0011"));
            assert_eq!(by("Credivale").as_deref(), Some("C0022"));
            // "Inter": o CNPJ do Inter não está no cadastro e "inter" não é palavra do outro.
            assert_eq!(by("Banco Inter"), None);
            Ok(())
        })
        .unwrap();
}

/// RDB tem a garantia do FGC, como o CDB.
#[test]
fn rdb_is_covered_by_the_fgc() {
    use crate::assets::FixedIncomeKind;
    assert_eq!(super::data::fgc_covered(FixedIncomeKind::Rdb), Some(true));
    assert_eq!(super::data::fgc_covered(FixedIncomeKind::Cdb), Some(true));
}
