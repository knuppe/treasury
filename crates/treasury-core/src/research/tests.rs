//! Coleta completa com fixtures (sem rede) e os comandos `research.*` pelo despachante.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use super::net::{FetchResult, Fetched, Fetcher, FileMeta};
use super::sync::{SyncArgs, SyncSource, SyncStatus, run_with};
use super::{b3, cvm, sec};
use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;
use crate::time::{fmt_date, parse_date, today_local};

/// Servidor de arquivos inventado: URL → bytes, com ETag "v1" e falhas programadas.
#[derive(Default)]
struct FixtureFetcher {
    files: HashMap<String, Vec<u8>>,
    errors: HashMap<String, String>,
    calls: Mutex<Vec<(String, String)>>,
}

impl Fetcher for FixtureFetcher {
    fn get(&self, url: &str, cond: Option<&FileMeta>, ua: &str) -> FetchResult {
        self.calls
            .lock()
            .unwrap()
            .push((url.to_string(), ua.to_string()));
        if let Some(e) = self.errors.get(url) {
            return Err(e.clone());
        }
        match self.files.get(url) {
            None => Ok(Fetched::NotFound),
            Some(_) if cond.and_then(|c| c.etag.as_deref()) == Some("v1") => {
                Ok(Fetched::NotModified)
            }
            Some(b) => Ok(Fetched::Body {
                bytes: b.clone(),
                meta: FileMeta {
                    etag: Some("v1".into()),
                    last_modified: None,
                },
            }),
        }
    }
}

fn fetcher() -> FixtureFetcher {
    let mut f = FixtureFetcher::default();
    f.files.insert(cvm::fca_url(2026), cvm::fixtures::fca_zip());
    for (y, rev, ni) in [
        (2020, 500, 60),
        (2021, 600, -10),
        (2022, 700, 80),
        (2023, 800, 90),
        (2024, 900, 100),
        (2025, 1000, 120),
    ] {
        f.files
            .insert(cvm::dfp_url(y), cvm::fixtures::dfp_zip(y, rev, ni));
    }
    f.files.insert(
        cvm::itr_url(2026),
        cvm::fixtures::itr_zip(2026, 600, 70, 500, 50),
    );
    f.files
        .insert(cvm::fii_url(2026), cvm::fixtures::fii_zip(2026));
    f.files.insert(
        cvm::fii_quarterly_url(2026),
        cvm::fixtures::fii_quarterly_zip(),
    );
    f.errors.insert(cvm::fii_url(2025), "tempo esgotado".into());
    f.files.insert(cvm::ipe_url(2026), cvm::fixtures::ipe_zip());
    let day = |s: &str| parse_date(s).unwrap();
    f.files.insert(
        b3::daily_url(day("2026-10-13")),
        b3::fixtures::zip(
            "20261013",
            &[
                ("02", "XMPL3", 1000, 500_000_000),
                ("02", "XMPL4", 1200, 300_000_000),
                ("02", "XMPL11", 3300, 100_000_000),
                ("02", "BXMP3", 3000, 900_000_000),
                ("12", "XPTO11", 9500, 200_000_000),
            ],
        ),
    );
    f.files.insert(
        b3::daily_url(day("2026-10-09")),
        b3::fixtures::zip("20261009", &[("02", "XMPL3", 950, 300_000_000)]),
    );
    f.files.insert(
        sec::TICKERS_URL.to_string(),
        br#"{"0":{"cik_str":1234,"ticker":"EXMP","title":"Example Corp"}}"#.to_vec(),
    );
    f.files
        .insert(sec::facts_url(1234), sec::fixtures::companyfacts());
    f
}

struct Env {
    _dir: tempfile::TempDir,
    api: Arc<Api>,
    xmpl4: String,
    exmp: String,
    xpto: String,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Arc::new(Api::new_unlocked_process(paths).unwrap());
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família Teste"}),
    )
    .unwrap();
    let mk = |v: Value| {
        api.call("assets.create", v).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let xmpl4 = mk(json!({"ticker": "XMPL4", "name": "Exemplo PN", "class": "stock_br"}));
    let exmp = mk(json!({"ticker": "EXMP", "name": "Example Corp", "class": "stock_us"}));
    let xpto = mk(json!({"ticker": "XPTO11", "name": "XPTO Logística", "class": "fii"}));
    for id in [&xmpl4, &xpto] {
        api.call("portfolio.watch", json!({"asset_id": id}))
            .unwrap();
    }
    api.call(
        "market.set_price",
        json!({"asset_id": exmp, "date": "2026-10-13", "close": "50"}),
    )
    .unwrap();
    // Rendimentos mensais do FII (agenda), para o DY e o radar.
    api.with_session(|s, _| {
        for (ex, v) in [
            ("2026-05-29", "0.80"),
            ("2026-06-30", "0.80"),
            ("2026-07-31", "0.82"),
            ("2026-08-31", "0.84"),
            ("2026-09-30", "0.86"),
        ] {
            s.db.conn().execute(
                "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES (?1, ?2, 'fii_income', ?3, NULL, ?4, 'BRL', 0, 'yahoo', 'x')",
                rusqlite::params![crate::ids::new_id(), xpto, ex, v],
            )?;
        }
        Ok(())
    })
    .unwrap();
    Env {
        _dir: dir,
        api,
        xmpl4,
        exmp,
        xpto,
    }
}

fn today() -> chrono::NaiveDate {
    parse_date("2026-10-14").unwrap()
}

fn ind(v: &Value, key: &str) -> Value {
    v["indicators"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["key"] == key)
        .cloned()
        .unwrap_or_else(|| panic!("{key} ausente"))
}

fn num(v: &Value, key: &str) -> f64 {
    ind(v, key)["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("{key}: {}", ind(v, key)))
}

#[test]
fn full_sync_with_fixtures_and_commands() {
    let e = env();
    // Sem e-mail de contato a SEC fica de fora da coleta (ela recusa com 403).
    let f = fetcher();
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    let sec_items: Vec<_> = rep
        .items
        .iter()
        .filter(|i| i.source == SyncSource::Sec)
        .collect();
    assert_eq!(sec_items.len(), 1, "{sec_items:?}");
    assert_eq!(sec_items[0].status, SyncStatus::Skipped);
    assert!(
        rep.warnings.iter().any(|w| w.contains("contato@exemplo")),
        "aviso do User-Agent da SEC"
    );
    // Com o e-mail, num cofre novo, a coleta completa.
    let e = {
        drop(e);
        env()
    };
    e.api
        .call(
            "research.set_settings",
            json!({"sec_user_agent": "Treasury/0.6 eu@email.com"}),
        )
        .unwrap();
    let f = fetcher();
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    // A falha do informe de FII de 2025 vira item do relatório; o resto segue.
    assert!(!rep.ok);
    let item = |label: &str| {
        rep.items
            .iter()
            .find(|i| i.label == label)
            .unwrap_or_else(|| panic!("{label}"))
    };
    assert_eq!(
        item("CVM informe mensal de FII 2025").error.as_deref(),
        Some("tempo esgotado")
    );
    assert_eq!(item("CVM FCA 2026").status, SyncStatus::Updated);
    assert_eq!(item("CVM FCA 2025").status, SyncStatus::NotFound);
    assert_eq!(item("CVM DFP 2025").status, SyncStatus::Updated);
    assert_eq!(item("CVM ITR 2026").status, SyncStatus::Updated);
    assert_eq!(
        item("CVM IPE 2026").rows,
        4,
        "emissores cadastrados e companhias com ação negociada (o estatuto fica de fora)"
    );
    assert_eq!(item("B3 COTAHIST 2026-10-13").rows, 5);
    assert!(
        rep.items
            .iter()
            .any(|i| i.source == SyncSource::Sec && i.status == SyncStatus::Updated)
    );
    assert!(rep.bytes > 0);
    assert!(rep.universe >= 6);
    assert!(
        !rep.warnings.iter().any(|w| w.contains("contato@exemplo")),
        "com e-mail, sem aviso da SEC"
    );
    let calls = f.calls.lock().unwrap().clone();
    let sec_ua = calls.iter().find(|(u, _)| u.contains("sec.gov")).unwrap();
    assert_eq!(sec_ua.1, "Treasury/0.6 eu@email.com");
    assert!(
        calls
            .iter()
            .filter(|(u, _)| u.contains("cvm"))
            .all(|(_, ua)| ua.is_empty())
    );

    let api = &e.api;
    // Datas de entrega (índice da DFP) e auditores (FCA) do Exemplo.
    api.with_session(|s, _| {
        let c = s.db.conn();
        let f = super::store::filings(c, "11222333000181")?;
        let dfp = f
            .iter()
            .find(|x| x.kind == "dfp" && x.reference == parse_date("2025-12-31").unwrap())
            .expect("entrega da DFP 2025");
        assert_eq!(dfp.filed_at, parse_date("2026-03-01").unwrap());
        assert_eq!(dfp.document.as_deref(), Some("92025"));
        let fii = super::store::filings(c, "33444555000102")?;
        assert_eq!(
            fii.iter().map(|x| x.kind.as_str()).collect::<Vec<_>>(),
            vec!["fii_trimestral", "fii_trimestral", "fii_mensal", "fii_mensal"],
            "mais recentes primeiro"
        );
        let walt: String = c.query_row(
            "SELECT value FROM fundamental WHERE issuer_id = '33444555000102' AND item = 'fii_walt_years'",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(walt, "1.8438");
        let a = super::store::auditors(c, "11222333000181")?;
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].auditor, "AUDITORES NOVOS S.S.", "o atual primeiro");
        assert_eq!(a[1].end, Some(parse_date("2025-12-31").unwrap()));
        Ok(())
    })
    .unwrap();
    // Score em lote do universo (explorador de mercado): um resumo por ação e FII do
    // universo, fora do histórico; os campos viram campos do screener.
    assert!(rep.market_scores.is_some(), "{:?}", rep.warnings);
    let (in_universe, in_batch): (i64, i64) = e
        .api
        .with_session(|s, _| {
            let c = s.db.conn();
            Ok((
                c.query_row(
                    "SELECT count(*) FROM research_universe WHERE class IN ('stock_br', 'fii')",
                    [],
                    |r| r.get(0),
                )?,
                c.query_row("SELECT count(*) FROM market_score", [], |r| r.get(0))?,
            ))
        })
        .unwrap();
    assert!(in_universe > 0);
    assert_eq!(in_batch, in_universe);
    let scored = api
        .call(
            "research.screen",
            json!({"filters": {"conditions": [{"field": "score_band", "op": "exists"}]}}),
        )
        .unwrap();
    assert_eq!(
        scored["total"].as_u64().unwrap_or(0),
        rep.market_scores.unwrap(),
        "{scored}"
    );
    // O screener filtra pelo score; rankings também ordenam por ele.
    let high = api
        .call(
            "research.screen",
            json!({"filters": {"conditions": [{"field": "score", "op": "gte", "value": 0}]}}),
        )
        .unwrap();
    assert_eq!(high["total"], scored["total"]);
    // Tela Mercado: panorama por setor e Observar sem operação.
    let ov = api.call("research.market_overview", json!({})).unwrap();
    assert_eq!(
        ov["stocks"].as_u64().unwrap() + ov["fiis"].as_u64().unwrap(),
        in_universe as u64
    );
    assert!(!ov["stock_sectors"].as_array().unwrap().is_empty());
    let map = api.call("research.market_map", json!({})).unwrap();
    assert_eq!(
        map.as_array().unwrap().len() as u64,
        rep.market_scores.unwrap()
    );
    let obs = api
        .call("research.observe", json!({"ticker": "xmpl3"}))
        .unwrap();
    let new_id = obs["asset_id"].as_str().unwrap().to_string();
    assert_ne!(new_id, e.xmpl4);
    let watch = api.call("portfolio.list_watchlist", json!({})).unwrap();
    assert!(watch.to_string().contains(&new_id), "{watch}");
    assert_eq!(
        api.call("research.observed", json!({"id": new_id}))
            .unwrap(),
        true
    );
    // Deixar de observar apaga o ativo criado pelo Observar (sem operação)…
    let un = api
        .call("research.unobserve", json!({"asset_id": new_id}))
        .unwrap();
    assert_eq!(un["removed"], true);
    assert!(api.call("assets.get", json!({"id": new_id})).is_err());
    // …mas não um ativo que já era da família.
    let mine = api
        .call("research.observe", json!({"ticker": "XMPL4"}))
        .unwrap();
    assert_eq!(mine["asset_id"], e.xmpl4.as_str());
    let un = api
        .call("research.unobserve", json!({"asset_id": e.xmpl4}))
        .unwrap();
    assert_eq!(un["removed"], false);
    assert!(api.call("assets.get", json!({"id": e.xmpl4})).is_ok());
    // Volta para a watchlist (os testes abaixo contam com ele).
    api.call("research.observe", json!({"ticker": "XMPL4"}))
        .unwrap();
    assert!(
        api.call("research.observe", json!({"ticker": "ZZZZ3"}))
            .is_err()
    );
    // Tags: catálogo com contagem; o screener filtra por todas e cada linha traz as suas.
    let tags = api.call("research.tags", json!({})).unwrap();
    let tags = tags.as_array().unwrap();
    assert!(tags.len() >= 20);
    let used = tags
        .iter()
        .find(|t| t["count"].as_u64().unwrap_or(0) > 0)
        .expect("alguma tag no universo do teste");
    let key = used["key"].as_str().unwrap();
    let r = api
        .call(
            "research.screen",
            json!({"filters": {"tags": [key]}, "limit": 500}),
        )
        .unwrap();
    assert_eq!(r["total"], used["count"], "{r}");
    for i in r["items"].as_array().unwrap() {
        assert!(
            i["tags"].as_array().unwrap().iter().any(|t| t == key),
            "{i}"
        );
    }
    assert!(
        r["description"]
            .as_str()
            .unwrap()
            .contains(used["label"].as_str().unwrap())
    );
    assert!(
        api.call(
            "research.screen",
            json!({"filters": {"tags": ["inventada"]}})
        )
        .is_err()
    );
    // Métodos: receitas, contagem, rodar e explicar.
    let methods = api.call("research.methods", json!({})).unwrap();
    assert_eq!(methods.as_array().unwrap().len(), 5);
    let counts = api.call("research.method_counts", json!({})).unwrap();
    assert!(counts.get("bazin").is_some(), "{counts}");
    let run = api
        .call("research.method_run", json!({"key": "bazin"}))
        .unwrap();
    assert_eq!(run["passing"], counts["bazin"]);
    let ex = api
        .call(
            "research.method_explain",
            json!({"key": "bazin", "ticker": "XMPL4"}),
        )
        .unwrap();
    assert!(
        ex["excluded"].is_string() || ex["checks"].as_array().is_some_and(|c| !c.is_empty()),
        "{ex}"
    );
    assert!(
        api.call("research.method_run", json!({"key": "nada"}))
            .is_err()
    );
    // Alerta de tela: a primeira passada só registra; quem entra depois vira alerta.
    let scr = api
        .call(
            "research.screens.create",
            json!({"name": "Alerta teste", "filters": {"conditions": [{"field": "pl", "op": "lt", "value": 8}]}, "alert": true}),
        )
        .unwrap();
    assert_eq!(scr["alert"], true);
    let alerts_before = api
        .call("ai.alerts.list", json!({}))
        .map(|v| v.to_string())
        .unwrap_or_default();
    e.api
        .with_session(|s, _| {
            let b = crate::research::config::load(s.db.conn())?.bazin();
            let r = crate::research::screen_alerts::run(s.db.conn(), today(), b)?;
            assert_eq!(r.entered, 0, "linha de base");
            s.db.conn()
                .execute("DELETE FROM screen_match WHERE ticker = 'XMPL3'", [])?;
            let r = crate::research::screen_alerts::run(s.db.conn(), today(), b)?;
            assert_eq!(r.entered, 1, "XMPL3 entrou");
            let r = crate::research::screen_alerts::run(s.db.conn(), today(), b)?;
            assert_eq!(r.entered, 0, "um alerta por episódio");
            Ok(())
        })
        .unwrap();
    let _ = alerts_before;
    api.call("research.screens.delete", json!({"id": scr["id"]}))
        .unwrap();
    let again = api.call("research.score_batch", json!({})).unwrap();
    assert_eq!(again["scored"].as_u64(), rep.market_scores);
    let status = api.call("research.status", json!({})).unwrap();
    assert!(status["facts"].as_u64().unwrap() > 50);
    assert_eq!(status["documents"], 4, "P1, P3, P5 e o outro emissor (P4)");

    // Ação cadastrada: LTM = DFP 2025 + ITR 2T26 − 2T25 = 120 + 70 − 50 (mil).
    let v = api
        .call("research.indicators", json!({"asset_id": e.xmpl4}))
        .unwrap();
    assert_eq!(v["reference_date"], "2026-06-30");
    assert_eq!(v["price"]["source"], "b3_cotahist");
    // 60 mil ON × 10 + 40 mil PN × 12.
    assert!((num(&v, "market_cap") - 1_080_000.0).abs() < 1e-6);
    assert!((num(&v, "pl") - 1_080_000.0 / 140_000.0).abs() < 1e-9);
    assert!((num(&v, "pvp") - 1.2).abs() < 1e-9);
    assert!((num(&v, "revenue_cagr_5y") - (2f64.powf(0.2) - 1.0)).abs() < 1e-9);
    assert_eq!(num(&v, "loss_years_10y"), 1.0);
    // Liquidez: média dos pregões com volume no COTAHIST.
    assert!((num(&v, "liquidity") - 3_000_000.0).abs() < 1e-6);
    let pl = ind(&v, "pl");
    assert!(pl["source"].as_str().unwrap().contains("doc. CVM"), "{pl}");
    assert_eq!(v["subject"]["issuer_code"], "11.222.333/0001-81");
    assert_eq!(v["subject"]["sector"], "Energia Elétrica");

    // Papel só do universo (sem cadastro): banco com ações informadas em milhares.
    let b = api
        .call("research.indicators", json!({"ticker": "BXMP3"}))
        .unwrap();
    assert_eq!(b["subject"]["layout"], "bank");
    assert!((num(&b, "pl") - 7.5).abs() < 1e-9);
    assert!(
        ind(&b, "ev_ebitda")["unavailable_reason"]
            .as_str()
            .unwrap()
            .contains("financeiras")
    );
    assert!(
        api.call("research.indicators", json!({"ticker": "NADA3"}))
            .is_err()
    );

    // Ação americana (SEC).
    let us = api
        .call("research.indicators", json!({"asset_id": e.exmp}))
        .unwrap();
    assert!((num(&us, "pl") - 5000.0 / 270.0).abs() < 1e-9);
    assert_eq!(us["subject"]["issuer_code"], "cik:0000001234");

    // FII: agenda do papel para DY e último rendimento; VP do informe.
    let fii = api
        .call("research.indicators", json!({"asset_id": e.xpto}))
        .unwrap();
    assert!((num(&fii, "pvp") - 95.0 / 101.0).abs() < 1e-9);
    assert!((num(&fii, "last_dividend") - 0.86).abs() < 1e-9);
    assert!((num(&fii, "dy") - 4.12 / 95.0).abs() < 1e-9);
    assert_eq!(fii["subject"]["segment"], "Logística");

    // Fundamentos brutos com documento de origem e link.
    let fu = api
        .call("research.fundamentals", json!({"asset_id": e.xmpl4}))
        .unwrap();
    let ni = fu["latest"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["item"] == "net_income")
        .unwrap()
        .clone();
    assert_eq!(ni["period_type"], "ltm");
    assert_eq!(ni["value"], "140000");
    let rev = fu["annual"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["item"] == "revenue")
        .unwrap()
        .clone();
    assert_eq!(rev["value"], "1000000");
    assert!(
        rev["document_url"]
            .as_str()
            .unwrap()
            .contains("NumeroSequencialDocumento=92025")
    );
    assert_eq!(fu["securities"].as_array().unwrap().len(), 3);

    // Preço justo: Graham √(22,5 × 1,4 × 9) = 16,84; Bazin 0,60/ano ÷ 8% = 7,50.
    let fp = api
        .call(
            "research.fair_price",
            json!({"asset_id": e.xmpl4, "bazin_rate": 8, "years": 5}),
        )
        .unwrap();
    assert_eq!(fp["graham"]["value"], "16.84");
    assert_eq!(fp["bazin"]["value"], "7.5");
    assert!(
        fp["bazin"]["assumptions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["value"] == "8%")
    );
    assert!(
        api.call(
            "research.fair_price",
            json!({"asset_id": e.xmpl4, "bazin_rate": -1})
        )
        .is_err()
    );

    // Histórico de 10 anos.
    let h = api
        .call(
            "research.history",
            json!({"asset_id": e.xmpl4, "years": 10}),
        )
        .unwrap();
    let years = h["years"].as_array().unwrap();
    assert_eq!(years.len(), 6);
    assert_eq!(years[5]["year"], 2025);
    assert_eq!(years[5]["revenue"], 100_000_000, "centavos");
    assert_eq!(years[1]["net_income"], -1_000_000);
    assert_eq!(years[5]["dps"], "0.6");

    // Ranking por P/L (menor primeiro, múltiplos negativos fora).
    let r = api
        .call("research.rank", json!({"indicator": "pl", "limit": 10}))
        .unwrap();
    let tickers: Vec<&str> = r["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["ticker"].as_str().unwrap())
        .collect();
    assert_eq!(tickers, vec!["BXMP3", "XMPL11", "XMPL3", "XMPL4", "EXMP"]);
    assert_eq!(r["order"], "asc");
    let r = api
        .call(
            "research.rank",
            json!({"indicator": "dy", "filters": {"classes": ["fii"]}}),
        )
        .unwrap();
    assert_eq!(r["items"][0]["ticker"], "XPTO11");
    let r = api
        .call(
            "research.rank",
            json!({"indicator": "market_cap", "filters": {"min_liquidity": 5_000_000}}),
        )
        .unwrap();
    let tickers: Vec<&str> = r["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["ticker"].as_str().unwrap())
        .collect();
    assert_eq!(tickers, vec!["BXMP3"], "liquidez mínima");
    assert!(
        api.call("research.rank", json!({"indicator": "nada"}))
            .is_err()
    );

    // Screener: P/L < 8 E ROE > 10%.
    let s = api
        .call(
            "research.screen",
            json!({"filters": {"conditions": [
                {"field": "pl", "op": "lt", "value": 8},
                {"field": "roe", "op": "gt", "value": 0.1}
            ]}, "sort_by": "pl"}),
        )
        .unwrap();
    let tickers: Vec<&str> = s["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["ticker"].as_str().unwrap())
        .collect();
    assert_eq!(tickers, vec!["BXMP3", "XMPL11", "XMPL3", "XMPL4"]);
    assert_eq!(s["description"], "P/L < 8 E ROE > 0.1");
    // OU com "held"/"watched".
    let s = api
        .call(
            "research.screen",
            json!({"filters": {"logic": "or", "conditions": [
                {"field": "watched", "op": "eq", "value": true},
                {"field": "ticker", "op": "eq", "value": "EXMP"}
            ]}}),
        )
        .unwrap();
    assert_eq!(s["total"], 3);

    // Telas salvas: criar, listar, rodar, desfazer.
    let scr = api
        .call(
            "research.screens.create",
            json!({"name": "Baratas e rentáveis", "filters": {"conditions": [{"field": "pl", "op": "lt", "value": 8}]}}),
        )
        .unwrap();
    let sid = scr["id"].as_str().unwrap().to_string();
    assert_eq!(scr["summary"], "P/L < 8");
    assert!(
        api.call(
            "research.screens.create",
            json!({"name": "baratas e rentáveis", "filters": {}})
        )
        .is_err()
    );
    let run = api
        .call("research.screen", json!({"screen_id": sid}))
        .unwrap();
    assert_eq!(run["total"], 4);
    api.call(
        "research.screens.update",
        json!({"id": sid, "name": "Baratas"}),
    )
    .unwrap();
    let hist = api.call("changes.list", json!({"limit": 1})).unwrap();
    assert_eq!(hist[0]["reason"], "Tela do screener alterada");
    api.call("changes.undo", json!({"id": hist[0]["id"]}))
        .unwrap();
    assert_eq!(
        api.call("research.screens.get", json!({"id": sid}))
            .unwrap()["name"],
        "Baratas e rentáveis"
    );
    api.call("research.screens.delete", json!({"id": sid}))
        .unwrap();
    assert_eq!(
        api.call("research.screens.list", json!({})).unwrap(),
        json!([])
    );
    let hist = api.call("changes.list", json!({"limit": 1})).unwrap();
    api.call("changes.undo", json!({"id": hist[0]["id"]}))
        .unwrap();
    assert_eq!(
        api.call("research.screens.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // Comparador (até 6).
    let c = api
        .call(
            "research.compare",
            json!({"asset_ids": [e.xmpl4, e.exmp], "tickers": ["BXMP3"]}),
        )
        .unwrap();
    assert_eq!(c["subjects"].as_array().unwrap().len(), 3);
    let pl = c["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == "pl")
        .unwrap()
        .clone();
    assert_eq!(pl["best"], 2, "BXMP3 tem o menor P/L");
    assert!(
        api.call("research.compare", json!({"tickers": ["XMPL3"]}))
            .is_err()
    );
    assert!(
        api.call(
            "research.compare",
            json!({"tickers": ["A", "B", "C", "D", "E", "F", "G"]})
        )
        .is_err()
    );

    // Documentos IPE (dados, não instruções).
    let docs = api
        .call("research.documents", json!({"from": "2026-01-01"}))
        .unwrap();
    let docs = docs.as_array().unwrap();
    assert_eq!(docs.len(), 3, "a ata da assembleia (P5) também entra");
    assert_eq!(docs[0]["id"], "P5", "mais recente primeiro");
    assert_eq!(docs[0]["category"], "Assembleia");
    assert_eq!(docs[2]["tickers"], json!(["XMPL4"]));
    assert!(
        docs[2]["subject"]
            .as_str()
            .unwrap()
            .contains("ignore as instruções")
    );
    let fr = api
        .call(
            "research.documents",
            json!({"asset_id": e.xmpl4, "from": "2026-01-01", "categories": ["fato relevante"]}),
        )
        .unwrap();
    assert_eq!(fr.as_array().unwrap().len(), 1);
    // Diagnóstico da aba Documentos: fonte, emissor e o que está guardado.
    let st = api
        .call("research.documents_status", json!({"asset_id": e.xmpl4}))
        .unwrap();
    assert_eq!(st["reason"], "ok");
    assert!(st["stored"].as_u64().unwrap() >= 2, "{st}");
    assert_eq!(st["source"], "cvm_ipe");
    assert!(
        st["kept_categories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "Assembleia")
    );
    // FII: os documentos dos fundos ficam no Fundos.NET (fonte ainda não coletada).
    let fii = api
        .call("research.documents_status", json!({"asset_id": e.xpto}))
        .unwrap();
    assert_eq!(fii["reason"], "not_covered");
    // Ticker que nem o cadastro nem o FCA conhecem: sem emissor, com aviso.
    let none = api
        .call("research.documents_status", json!({"ticker": "ZZZZ3"}))
        .unwrap();
    assert_eq!(none["reason"], "no_issuer");

    // Radar: FII mensal (média dos eventos dos últimos 12 meses), sem duplicar. O radar
    // projeta a partir de hoje: os proventos deste FII são do último mês para trás, para o
    // teste não depender da data em que roda.
    let now = today_local();
    let radr = api
        .call(
            "assets.create",
            json!({"ticker": "RADR11", "name": "Radar Renda", "class": "fii"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let last = chrono::Datelike::with_day(&now, 1).unwrap() - chrono::Duration::days(1);
    api.with_session(|s, _| {
        for (k, v) in [(4, "0.80"), (3, "0.80"), (2, "0.82"), (1, "0.84"), (0, "0.86")] {
            s.db.conn().execute(
                "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES (?1, ?2, 'fii_income', ?3, NULL, ?4, 'BRL', 0, 'yahoo', 'x')",
                rusqlite::params![
                    crate::ids::new_id(),
                    radr,
                    fmt_date(crate::time::add_months(last, -k)),
                    v
                ],
            )?;
        }
        Ok(())
    })
    .unwrap();
    let radar = api
        .call("research.dividend_radar", json!({"asset_ids": [radr]}))
        .unwrap();
    let a = radar["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["asset"]["id"] == radr.as_str())
        .unwrap()
        .clone();
    assert_eq!(a["frequency"], "mensal");
    assert_eq!(a["projected"].as_array().unwrap().len(), 12);
    assert_eq!(a["average"], "0.824");
    let first = (1..)
        .map(|k| crate::time::add_months(last, k))
        .find(|d| *d > now)
        .unwrap();
    assert_eq!(a["projected"][0]["ex_date"], fmt_date(first).as_str());
    let again = api
        .call("research.dividend_radar", json!({"asset_ids": [radr]}))
        .unwrap();
    assert_eq!(again["removed"], 12);
    let cal = api
        .call(
            "portfolio.dividend_calendar",
            json!({
                "from": fmt_date(now + chrono::Duration::days(1)),
                "to": fmt_date(crate::time::add_months(now, 12)),
                "only_mine": false,
            }),
        )
        .unwrap();
    assert_eq!(
        cal.as_array()
            .unwrap()
            .iter()
            .filter(|e| e["estimated"] == true && e["asset"]["id"] == radr.as_str())
            .count(),
        12
    );

    // Cliente MCP somente leitura: consultas liberadas, escritas não.
    let reader = Actor::Mcp {
        client: "teste".into(),
        read_only: true,
    };
    assert!(
        api.call_as(&reader, "research.rank", json!({"indicator": "dy"}))
            .is_ok()
    );
    for cmd in [
        "research.dividend_radar",
        "research.screens.create",
        "research.set_settings",
        "research.refresh_universe",
    ] {
        assert!(
            matches!(
                api.call_as(&reader, cmd, json!({})),
                Err(crate::Error::Forbidden(_))
            ),
            "{cmd}"
        );
    }
    // Cliente MCP com escrita: tela salva vira proposta pendente.
    let writer = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let p = api
        .call_as(
            &writer,
            "research.screens.create",
            json!({"name": "Da IA", "filters": {}}),
        )
        .unwrap();
    assert_eq!(p["status"], "pending_approval");
    assert_eq!(
        api.call("research.screens.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // Segunda coleta: nada mudou (ETag), só os arquivos sem ETag são baixados de novo.
    let rep2 = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    assert_eq!(
        rep2.items
            .iter()
            .find(|i| i.label == "CVM DFP 2025")
            .unwrap()
            .status,
        SyncStatus::Unchanged
    );
    let ipe = rep2
        .items
        .iter()
        .find(|i| i.label == "CVM IPE 2026")
        .unwrap();
    assert_eq!(ipe.status, SyncStatus::Updated);
    // Ativo novo de um emissor sem demonstrações: DFP/ITR são baixados de novo uma vez só.
    api.with_session(|s, _| {
        s.db.conn().execute(
            "INSERT INTO research_issuer (id, kind, name, layout, currency, source, fetched_at)
             VALUES ('99888777000166', 'company_br', 'SEM DFP S.A.', 'industrial', 'BRL', 'cvm_fca', 'x')",
            [],
        )?;
        s.db.conn().execute(
            "INSERT INTO research_security (ticker, issuer_id, class, share_kind, active, source, fetched_at)
             VALUES ('SDFP3', '99888777000166', 'stock_br', 'ON', 1, 'cvm_fca', 'x')",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    api.call(
        "assets.create",
        json!({"ticker": "SDFP3", "name": "Sem DFP", "class": "stock_br"}),
    )
    .unwrap();
    let only_dfp = SyncArgs {
        force: None,
        sources: Some(vec![SyncSource::Dfp]),
    };
    let rep4 = run_with(&e.api, &only_dfp, &f, today()).unwrap();
    assert_eq!(
        rep4.items
            .iter()
            .find(|i| i.label == "CVM DFP 2025")
            .unwrap()
            .status,
        SyncStatus::Updated
    );
    let rep5 = run_with(&e.api, &only_dfp, &f, today()).unwrap();
    assert_eq!(
        rep5.items
            .iter()
            .find(|i| i.label == "CVM DFP 2025")
            .unwrap()
            .status,
        SyncStatus::Unchanged
    );
    // Só uma fonte, forçando.
    let rep3 = run_with(
        &e.api,
        &SyncArgs {
            force: Some(true),
            sources: Some(vec![SyncSource::Fca]),
        },
        &f,
        today(),
    )
    .unwrap();
    assert!(rep3.items.iter().all(|i| i.source == SyncSource::Fca));
    assert_eq!(
        rep3.items
            .iter()
            .find(|i| i.label == "CVM FCA 2026")
            .unwrap()
            .status,
        SyncStatus::Updated
    );
}

#[test]
fn settings_require_contact_email_and_user() {
    let e = env();
    let api = &e.api;
    let v = api.call("research.get_settings", json!({})).unwrap();
    assert_eq!(v["sec_user_agent_is_default"], true);
    assert!(v["sec_hint"].as_str().unwrap().contains("e-mail"));
    assert_eq!(v["settings"]["bazin_rate"], 0.06);
    assert!(
        api.call(
            "research.set_settings",
            json!({"sec_user_agent": "Treasury/0.1 sem-email"})
        )
        .is_err()
    );
    assert!(
        api.call("research.set_settings", json!({"bazin_years": 30}))
            .is_err()
    );
    assert!(
        api.call("research.set_settings", json!({"nada": 1}))
            .is_err()
    );
    let v = api
        .call(
            "research.set_settings",
            json!({"sec_user_agent": "Treasury/0.1 eu@email.com", "bazin_rate": 7}),
        )
        .unwrap();
    assert_eq!(v["sec_user_agent_is_default"], false);
    assert!(v["sec_hint"].is_null());
    assert_eq!(v["settings"]["bazin_rate"], 0.07);
    let mcp = Actor::Mcp {
        client: "x".into(),
        read_only: false,
    };
    assert!(
        api.call_as(&mcp, "research.set_settings", json!({"bazin_years": 3}))
            .is_err()
    );
    // O User-Agent configurado vai para a SEC.
    let f = fetcher();
    run_with(
        api,
        &SyncArgs {
            force: None,
            sources: Some(vec![SyncSource::Sec]),
        },
        &f,
        today(),
    )
    .unwrap();
    assert!(
        f.calls
            .lock()
            .unwrap()
            .iter()
            .all(|(_, ua)| ua == "Treasury/0.1 eu@email.com")
    );
}

#[test]
fn commands_work_before_any_sync() {
    let e = env();
    let api = &e.api;
    // Sem dados baixados: indicadores de cotação ainda saem, com motivos claros.
    let v = api
        .call("research.indicators", json!({"asset_id": e.exmp}))
        .unwrap();
    assert!((num(&v, "price") - 50.0).abs() < 1e-9);
    assert!(ind(&v, "pl")["unavailable_reason"].is_string());
    assert!(v["notes"][0].as_str().unwrap().contains("coleta"));
    let r = api
        .call("research.rank", json!({"indicator": "price"}))
        .unwrap();
    assert_eq!(r["items"][0]["ticker"], "EXMP");
    // O universo calculado na consulta fica gravado para o resto do dia…
    let st = api.call("research.status", json!({})).unwrap();
    assert_eq!(st["universe_size"], 3);
    // …e um ativo cadastrado depois entra na próxima consulta.
    let id = api
        .call(
            "assets.create",
            json!({"ticker": "NOVO3", "name": "Novo", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "market.set_price",
        json!({"asset_id": id, "date": "2026-10-13", "close": "99"}),
    )
    .unwrap();
    let r = api
        .call("research.rank", json!({"indicator": "price"}))
        .unwrap();
    assert_eq!(r["items"][0]["ticker"], "NOVO3");
    assert_eq!(
        api.call("research.catalog", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        super::indicators::CATALOG.len()
    );
    assert_eq!(
        api.call("research.documents", json!({})).unwrap(),
        json!([])
    );
    assert!(matches!(
        api.call("research.nada", json!({})),
        Err(crate::Error::UnknownCommand(_))
    ));
}

#[test]
#[ignore = "usa a rede (CVM, ~13 MB): cargo test -p treasury-core -- --ignored live_cvm_petr4"]
fn live_cvm_petr4() {
    use super::net::HttpFetcher;
    let f = HttpFetcher::new();
    let year = chrono::Datelike::year(&crate::time::today_local()) - 1;
    let fca = match f.get(&cvm::fca_url(year + 1), None, "").unwrap() {
        Fetched::Body { bytes, .. } => cvm::parse_fca(&bytes).unwrap(),
        other => panic!("FCA: {other:?}"),
    };
    let petr = fca
        .securities
        .iter()
        .find(|s| s.ticker == "PETR4")
        .expect("PETR4 no FCA");
    assert_eq!(petr.issuer_id, "33000167000101");
    assert_eq!(petr.share_kind.as_deref(), Some("PN"));
    let issuer = petr.issuer_id.clone();
    let dfp = match f.get(&cvm::dfp_url(year), None, "").unwrap() {
        Fetched::Body { bytes, .. } => {
            cvm::parse_statements(&bytes, cvm::FilingKind::Dfp, &|c| c == issuer).unwrap()
        }
        other => panic!("DFP: {other:?}"),
    };
    let p = dfp.first().expect("DFP da Petrobras");
    assert!(p.consolidated);
    assert_eq!(p.layout, Some(super::Layout::Industrial));
    let get = |item: &str| {
        p.facts
            .iter()
            .find(|x| x.item == item && x.priority == 2)
            .map(|x| x.value)
    };
    assert!(get(super::items::REVENUE).unwrap() > rust_decimal::Decimal::from(100_000_000_000i64));
    assert!(get(super::items::SHARES).unwrap() > rust_decimal::Decimal::from(10_000_000_000i64));
    assert!(get(super::items::EQUITY).is_some() && get(super::items::DA).is_some());
}

#[test]
#[ignore = "usa a rede (coleta completa, ~200 MB em memória): cargo test -p treasury-core -- --ignored live_full_sync --nocapture"]
fn live_full_sync() {
    let e = env();
    let api = &e.api;
    for (t, c) in [
        ("PETR4", "stock_br"),
        ("ITUB4", "stock_br"),
        ("HGLG11", "fii"),
        ("TAEE11", "stock_br"),
    ] {
        let id = api
            .call("assets.create", json!({"ticker": t, "name": t, "class": c}))
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        api.call("portfolio.watch", json!({"asset_id": id}))
            .unwrap();
    }
    let started = std::time::Instant::now();
    let rep = super::sync::run(
        api,
        &SyncArgs {
            force: None,
            sources: Some(vec![
                SyncSource::Fca,
                SyncSource::Dfp,
                SyncSource::Itr,
                SyncSource::Fii,
                SyncSource::Ipe,
                SyncSource::B3,
            ]),
        },
    )
    .unwrap();
    println!(
        "tempo: {:?}, baixado: {} bytes, universo: {}",
        started.elapsed(),
        rep.bytes,
        rep.universe
    );
    for i in &rep.items {
        println!(
            "{:?} {} {} bytes {} linhas {:?}",
            i.status, i.label, i.bytes, i.rows, i.error
        );
    }
    for w in &rep.warnings {
        println!("aviso: {w}");
    }
    println!("{}", api.call("research.status", json!({})).unwrap());
    for t in ["PETR4", "ITUB4", "HGLG11", "TAEE11", "WEGE3", "BBAS3"] {
        let v = api
            .call("research.indicators", json!({"ticker": t}))
            .unwrap();
        let line: Vec<String> = v["indicators"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| format!("{}={}", i["key"].as_str().unwrap(), i["value"]))
            .collect();
        println!("{t} ref {}: {}", v["reference_date"], line.join(" "));
    }
    let fp = api
        .call("research.fair_price", json!({"ticker": "PETR4"}))
        .unwrap();
    println!(
        "PETR4 preço justo: graham {} bazin {}",
        fp["graham"]["value"], fp["bazin"]["value"]
    );
    let r = api
        .call(
            "research.rank",
            json!({"indicator": "dy", "limit": 5, "filters": {"min_liquidity": 1_000_000}}),
        )
        .unwrap();
    println!("ranking DY: {}", r["items"]);
    let docs = api
        .call(
            "research.documents",
            json!({"from": "2026-01-01", "limit": 3}),
        )
        .unwrap();
    println!("documentos: {}", docs);
    let size = std::fs::metadata(api.paths().db_path.clone())
        .unwrap()
        .len();
    println!("banco: {size} bytes");
    assert!(rep.universe > 300);
}

/// IF.data com fixtures: sondagem do trimestre, cadastro, relatórios completos e filtrados,
/// leitura e coleta incremental.
#[test]
fn ifdata_sync_with_fixtures() {
    use super::ifdata::{self as ifd, fixtures as fx};
    let e = env();
    let mut f = fetcher();
    f.files
        .insert(ifd::cadastro_url("2026-09", true), fx::empty());
    f.files
        .insert(ifd::cadastro_url("2026-06", true), fx::cadastro("2026-06"));
    f.files
        .insert(ifd::cadastro_url("2026-06", false), fx::cadastro("2026-06"));
    let targets: Vec<String> = vec!["C0080001".into(), "C0080002".into(), "C0080003".into()];
    for (p, ni) in [
        ("2026-06", "26000"),
        ("2026-03", "13000"),
        ("2025-12", "24000"),
        ("2025-09", "12000"),
        ("2025-06", "22000"),
    ] {
        f.files.insert(
            ifd::values_url(p, "1", None),
            fx::values(
                p,
                "1",
                &[
                    ("C0080001", "79664", "0.151234"),
                    ("C0080001", "140220", "1000000"),
                    ("C0080001", "140246", "100000"),
                    ("C0080001", "141870", ni),
                    ("C0080001", "141873", "500000"),
                    ("C0080001", "140239", "600000"),
                    ("C0080002", "140220", "2000000"),
                    ("C0080003", "140220", "50000"),
                    ("12345678", "140220", "10"),
                ],
            ),
        );
        f.files.insert(
            ifd::values_url(p, "5", None),
            fx::values(
                p,
                "5",
                &[
                    ("C0080001", "79659", "0.12"),
                    ("C0080001", "79660", "0.13"),
                    ("C0080001", "79661", "0.07"),
                    ("C0080001", "79665", "700000"),
                ],
            ),
        );
        f.files.insert(
            ifd::values_url(p, "16", None),
            fx::values(
                p,
                "16",
                &[
                    ("C0080001", "148834", "2000"),
                    ("C0080001", "148833", "3000"),
                    ("C0080001", "24454", "480000"),
                ],
            ),
        );
        f.files.insert(
            ifd::values_url(p, "2", Some(&targets)),
            fx::values(p, "2", &[("C0080001", "140202", "-9000")]),
        );
        f.files.insert(
            ifd::values_url(p, "4", Some(&targets)),
            fx::values(
                p,
                "4",
                &[
                    ("C0080001", "141842", "-1500"),
                    ("C0080001", "141858", "-800"),
                ],
            ),
        );
    }
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    let ifdata: Vec<_> = rep
        .items
        .iter()
        .filter(|i| i.source == SyncSource::Ifdata)
        .collect();
    // Cadastro + 5 trimestres × (3 completos + 2 filtrados).
    assert_eq!(ifdata.len(), 26, "{ifdata:#?}");
    assert!(
        ifdata.iter().all(|i| i.status == SyncStatus::Updated),
        "{ifdata:#?}"
    );
    e.api
        .with_session(|s, _| {
            let c = s.db.conn();
            // Banco Exemplo (CNPJ 22.333.444): o conglomerado prudencial.
            let b = ifd::institution_for_cnpj8(c, "22333444000190")?.unwrap();
            assert_eq!(b.cod_inst, "C0080001");
            let ser = ifd::series(c, "C0080001")?;
            assert_eq!(ser.len(), 5);
            let jun = &ser["2026-06"];
            assert_eq!(jun[ifd::items::BASEL_RATIO].to_string(), "0.151234");
            assert_eq!(jun[ifd::items::EXPECTED_LOSS].to_string(), "9000");
            assert_eq!(jun[ifd::items::PROVISION_EXPENSE].to_string(), "1500");
            assert_eq!(jun[ifd::items::CREDIT_TOTAL].to_string(), "480000");
            assert_eq!(
                ifd::twelve_months(&ser, "2026-06", ifd::items::NET_INCOME).map(|d| d.to_string()),
                Some("50000".into())
            );
            // Pares: bancos S1–S4 e o conglomerado sem segmento; nem a financeira (N1) nem a
            // cooperativa S5.
            let mut peers: Vec<String> = ifd::peers(c, "2026-06")?
                .into_iter()
                .map(|(i, _)| i.cod_inst)
                .collect();
            peers.sort();
            assert_eq!(peers, vec!["C0080001", "C0080002"]);
            Ok(())
        })
        .unwrap();
    // Segunda coleta: nada do IF.data é baixado de novo (nem a sondagem, feita há pouco).
    let before = f.calls.lock().unwrap().len();
    let rep2 = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    let olinda = f.calls.lock().unwrap()[before..]
        .iter()
        .filter(|(u, _)| u.contains("olinda"))
        .count();
    assert_eq!(olinda, 0);
    assert!(rep2.items.iter().all(|i| i.source != SyncSource::Ifdata));
    // Fonte "Banco Central" desligada: o IF.data fica de fora.
    e.api
        .call("sources.set", json!({"id": "bcb", "enabled": false}))
        .unwrap();
    let rep3 = run_with(
        &e.api,
        &SyncArgs {
            force: Some(true),
            sources: Some(vec![SyncSource::Ifdata]),
        },
        &f,
        today(),
    )
    .unwrap();
    assert!(rep3.items.is_empty());
}

/// Parser novo: os arquivos da CVM já processados (sem mudança pelo ETag) são lidos de novo
/// uma vez, e só depois de uma coleta sem erro nessas fontes.
#[test]
fn reparses_once_after_parser_upgrade() {
    let e = env();
    let mut f = fetcher();
    let status = |rep: &super::sync::ResearchSyncReport, label: &str| {
        rep.items
            .iter()
            .find(|i| i.label == label)
            .unwrap_or_else(|| panic!("{label}"))
            .status
    };
    let version = || -> Option<u32> {
        e.api
            .with_session(|s, _| crate::settings::get(s.db.conn(), "research.parser_version"))
            .unwrap()
    };
    let set_version = |v: u32| {
        e.api
            .with_session(|s, _| crate::settings::set(s.db.conn(), "research.parser_version", &v))
            .unwrap()
    };
    // Cofre novo: nada a reler; a versão fica gravada.
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    assert_eq!(status(&rep, "CVM DFP 2025"), SyncStatus::Updated);
    assert_eq!(version(), Some(super::sync::PARSER_VERSION));
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    assert_eq!(status(&rep, "CVM DFP 2025"), SyncStatus::Unchanged);
    // Cofre de antes da atualização: relê; com o informe de FII de 2025 falhando, relê de novo
    // na coleta seguinte.
    set_version(1);
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    assert_eq!(status(&rep, "CVM DFP 2025"), SyncStatus::Updated);
    assert_eq!(status(&rep, "CVM FCA 2026"), SyncStatus::Updated);
    assert_eq!(status(&rep, "CVM ITR 2026"), SyncStatus::Updated);
    assert_eq!(
        status(&rep, "CVM informe mensal de FII 2026"),
        SyncStatus::Updated
    );
    // O trimestral de FII entrou na releitura na versão 5.
    assert_eq!(
        status(&rep, "CVM informe trimestral de FII 2026"),
        SyncStatus::Updated
    );
    assert_eq!(version(), Some(1));
    f.errors.clear();
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    assert_eq!(status(&rep, "CVM DFP 2025"), SyncStatus::Updated);
    assert_eq!(version(), Some(super::sync::PARSER_VERSION));
    let rep = run_with(&e.api, &SyncArgs::default(), &f, today()).unwrap();
    assert_eq!(status(&rep, "CVM DFP 2025"), SyncStatus::Unchanged);
    assert_eq!(status(&rep, "CVM ITR 2026"), SyncStatus::Unchanged);
}

/// Nomes e fórmulas dos indicadores, notas e erros saem no idioma da instalação.
#[test]
fn research_texts_follow_the_locale() {
    use crate::i18n::{Locale, set_current};
    let pl = super::indicators::def("pl").unwrap();
    assert_eq!(pl.label(), "P/L");
    for d in super::indicators::CATALOG {
        for l in [Locale::PtBr, Locale::En, Locale::Es] {
            let k = format!("nucleo-pesquisa-ind-{}", d.key);
            assert!(crate::i18n::has_key(l, &k), "{l:?}: {k}");
            assert!(
                crate::i18n::has_key(l, &format!("{k}-formula")),
                "{l:?}: {k}"
            );
        }
    }
    assert_eq!(super::items::label("revenue"), "Receita líquida");
    assert_eq!(super::items::label("xyz"), "Item");
    set_current(Locale::En);
    assert_eq!(pl.label(), "P/E");
    assert_eq!(super::items::label("revenue"), "Net revenue");
    assert_eq!(
        super::indicators::ltm_until(parse_date("2026-06-30").unwrap()),
        "12 months to Jun 30, 2026"
    );
    let e = super::PeriodType::parse("weekly").unwrap_err();
    assert_eq!(e.message(), "invalid period type: weekly");
    assert!(super::config::sec_hint().starts_with("The SEC requires"));
    set_current(Locale::PtBr);
}
