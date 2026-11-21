//! Testes do motor (pares sintéticos) e do score de ponta a ponta (cofre temporário).

use std::sync::Arc;

use chrono::NaiveDate;
use serde_json::{Value, json};

use super::config::methodology;
use super::engine::{self, Peer};
use super::metrics::tests::{company, d};
use super::metrics::{EquityInput, Metrics, Statements, equity_metrics};
use super::{ScoreFlag, ScoreStatus, Severity};
use crate::api::Api;
use crate::paths::TreasuryPaths;
use crate::research::{Fact, Issuer, IssuerKind, Layout, Security, store};

fn metrics_of(facts: &[Fact], mc: f64) -> Metrics {
    let st = Statements::new(facts, d("2026-10-16"));
    let one = |_: NaiveDate, _: NaiveDate| Some(1.0);
    equity_metrics(&EquityInput {
        st: &st,
        today: d("2026-10-16"),
        market_cap: Some((mc, "teste".into())),
        dividend_yield: None,
        listing: Some((60.0, "Nível 1".into())),
        filings: &[],
        prices: &[],
        inflation: &one,
        tax_rate: 0.34,
        history_years: 5,
    })
    .metrics
}

/// 12 pares no mesmo setor com margens de 2% a 13% e alavancagem crescente.
fn peers() -> Vec<Peer> {
    (0..12)
        .map(|i| {
            let margin = 0.02 + 0.01 * i as f64;
            let f = company(1.0 + i as f64 * 0.3, margin, 0.4 - 0.03 * i as f64);
            Peer {
                id: format!("p{i}"),
                ticker: Some(format!("PEER{i}")),
                name: format!("Par {i}"),
                sector: Some("energia eletrica".into()),
                segment: "utilities".into(),
                metrics: metrics_of(&f, 3000.0 * (1.0 + i as f64 * 0.3)),
            }
        })
        .collect()
}

fn run(metrics: &Metrics, flags: Vec<ScoreFlag>) -> engine::Output {
    let m = methodology();
    engine::run(engine::Input {
        m,
        profile: &m.profiles["equity_br/utilities"],
        id: Some("eu"),
        sector: Some("energia eletrica"),
        sector_label: Some("Energia Elétrica"),
        segment: "utilities",
        segment_label: "Utilities",
        metrics,
        peers: &peers(),
        flags,
        data_date: Some(d("2026-06-30")),
        today: d("2026-10-16"),
        extra_confidence: vec![],
    })
}

#[test]
fn strong_company_scores_high_and_explains_why() {
    let good = metrics_of(&company(2.0, 0.18, -0.2), 3000.0);
    let weak = metrics_of(&company(2.0, 0.01, 0.8), 3000.0);
    let g = run(&good, vec![]);
    let w = run(&weak, vec![]);
    assert_eq!(g.status, ScoreStatus::Ok);
    assert_eq!(w.status, ScoreStatus::Ok);
    let (gs, ws) = (g.score.unwrap(), w.score.unwrap());
    assert!(gs > ws + 15.0, "bom {gs} × fraco {ws}");
    assert!(
        g.top_drivers.iter().any(|t| t.contains("pares do setor")),
        "{:?}",
        g.top_drivers
    );
    assert!(!w.top_detractors.is_empty());
    assert_eq!(g.peers.sector_size, 12);
    // Pares no mesmo setor e dados recentes: confiança alta.
    assert!(g.confidence > 0.8, "{}", g.confidence);
    // Sub-scores: utilities não tem momentum; atratividade = valuation.
    assert!(g.health.is_some() && g.attractiveness.is_some());
    let roe = g.dimensions[0]
        .metrics
        .iter()
        .find(|x| x.key == "roe")
        .unwrap();
    assert_eq!(roe.peers, Some(12));
    assert!(roe.percentile.unwrap() > 90.0);
    assert!(roe.average.is_some());
}

#[test]
fn critical_metric_missing_means_no_score() {
    let mut ms = metrics_of(&company(2.0, 0.18, -0.2), 3000.0);
    ms.remove("net_debt_ebitda");
    let o = run(&ms, vec![]);
    assert_eq!(o.status, ScoreStatus::InsufficientData);
    assert_eq!(o.score, None);
    assert_eq!(o.band_label, "Dados insuficientes");
    assert!(o.band_meaning.contains("Dívida líquida / EBITDA"));
    assert!(
        o.missing
            .iter()
            .any(|x| x.critical && x.key == "net_debt_ebitda")
    );
    assert!(o.dimensions.iter().all(|d| d.score.is_none()));
}

#[test]
fn heavy_imputation_gives_a_partial_score_without_band() {
    let full = metrics_of(&company(2.0, 0.18, -0.2), 3000.0);
    let keep = ["roe", "net_margin", "net_debt_ebitda", "loss_years_10y"];
    let mut ms = Metrics::new();
    for (k, v) in &full {
        let dim_rent_or_solv = [
            "roic",
            "gross_profitability",
            "ebit_margin",
            "interest_coverage",
            "altman_z2",
            "current_ratio",
            "cash_to_st_debt",
        ];
        if keep.contains(&k.as_str()) || !dim_rent_or_solv.contains(&k.as_str()) {
            ms.insert(k.clone(), v.clone());
        }
    }
    let o = run(&ms, vec![]);
    assert_eq!(o.status, ScoreStatus::Partial);
    assert!(o.score.is_some());
    assert_eq!(o.band_label, "Dados insuficientes");
    assert!(o.imputed_share > 0.3);
    let rent = o
        .dimensions
        .iter()
        .find(|d| d.key == "rentabilidade")
        .unwrap();
    assert!(rent.imputed);
    // Imputado = mediana dos pares × 0,9.
    assert!(rent.score.unwrap() < 60.0);
}

#[test]
fn flags_cap_and_penalize() {
    let good = metrics_of(&company(2.0, 0.18, -0.2), 3000.0);
    let base = run(&good, vec![]).score.unwrap();
    let cap = ScoreFlag {
        key: "altman_distress".into(),
        label: "x".into(),
        severity: Severity::High,
        cap: Some(45.0),
        penalty: None,
        detail: String::new(),
    };
    let pen = ScoreFlag {
        key: "piotroski_low".into(),
        label: "y".into(),
        severity: Severity::Medium,
        cap: None,
        penalty: Some(5.0),
        detail: String::new(),
    };
    assert_eq!(run(&good, vec![cap.clone()]).score, Some(45.0f64.min(base)));
    let p = run(&good, vec![pen]).score.unwrap();
    assert!((base - p - 5.0).abs() < 0.11, "{base} → {p}");
    let o = run(&good, vec![cap]);
    assert_eq!(o.raw_score.map(|x| x.round()), Some(base.round()));
}

#[test]
fn without_peers_uses_absolute_anchor_and_lowers_confidence() {
    let m = methodology();
    let ms = metrics_of(&company(2.0, 0.18, -0.2), 3000.0);
    let o = engine::run(engine::Input {
        m,
        profile: &m.profiles["equity_br/default"],
        id: None,
        sector: None,
        sector_label: None,
        segment: "default",
        segment_label: "Geral",
        metrics: &ms,
        peers: &[],
        flags: vec![],
        data_date: Some(d("2025-12-31")),
        today: d("2026-10-16"),
        extra_confidence: vec![],
    });
    assert_eq!(o.status, ScoreStatus::Ok);
    let roe = o.dimensions[0]
        .metrics
        .iter()
        .find(|x| x.key == "roe")
        .unwrap();
    assert_eq!(roe.percentile, None);
    assert!(roe.absolute.is_some());
    // Sem pares (0,7) e dados com mais de 200 dias (0,8).
    assert!(o.stale);
    assert!(o.confidence < 0.6, "{}", o.confidence);
}

// ---------------------------------------------------------------------------
// Ponta a ponta

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
        json!({"password": "senha-mestra-forte", "household_name": "Família Teste"}),
    )
    .unwrap();
    Env { _dir: dir, api }
}

/// Emissor com ticker, fatos e cotação de ontem no universo.
fn seed_company(api: &Api, n: usize, ticker: &str, facts: &[Fact], close: &str, sector: &str) {
    let today = crate::time::today_local();
    api.with_session(|s, _| {
        let c = s.db.conn();
        let id = format!("{:014}", n);
        store::upsert_issuer(
            c,
            &Issuer {
                id: id.clone(),
                kind: IssuerKind::CompanyBr,
                name: format!("Companhia {ticker}"),
                cvm_code: None,
                sector: Some(sector.into()),
                segment: None,
                mandate: None,
                layout: Layout::Industrial,
                currency: "BRL".into(),
                source: "teste".into(),
            },
            "2026-10-01T00:00:00Z",
        )?;
        store::upsert_security(
            c,
            &Security {
                ticker: ticker.into(),
                issuer_id: id.clone(),
                class: "stock_br".into(),
                share_kind: Some("ON".into()),
                unit_composition: None,
                isin: None,
                listing_segment: Some("Novo Mercado".into()),
                active: true,
                source: "teste".into(),
            },
            "2026-10-01T00:00:00Z",
        )?;
        store::upsert_facts(c, &id, facts, "2026-10-01T00:00:00Z")?;
        c.execute(
            "INSERT INTO research_quote (ticker, date, close, volume, trades, source, fetched_at)
             VALUES (?1, ?2, ?3, '50000000', 100, 'b3_cotahist', 'x')",
            rusqlite::params![
                ticker,
                crate::time::fmt_date(today - chrono::Duration::days(1)),
                close
            ],
        )?;
        Ok(())
    })
    .unwrap();
}

fn create_asset(api: &Api, ticker: &str) -> String {
    let v = api
        .call(
            "assets.create",
            json!({"ticker": ticker, "name": ticker, "class": "stock_br"}),
        )
        .unwrap();
    let id = v["id"].as_str().unwrap().to_string();
    api.call("portfolio.watch", json!({"asset_id": id}))
        .unwrap();
    id
}

fn seed_sector(api: &Api) {
    for i in 0..10 {
        let f = company(
            1.0 + i as f64 * 0.2,
            0.03 + 0.01 * i as f64,
            0.3 - 0.03 * i as f64,
        );
        seed_company(
            api,
            i + 1,
            &format!("ENRG{}3", i),
            &f,
            &format!("{}", 2 + i),
            "Energia Elétrica",
        );
    }
}

/// Leitura por cliente MCP calcula o score, mas não grava snapshot, alertas nem feedback.
#[test]
fn mcp_reads_do_not_write_snapshots() {
    let e = env();
    seed_sector(&e.api);
    let id = create_asset(&e.api, "ENRG93");
    let reader = crate::api::Actor::Mcp {
        client: "teste".into(),
        read_only: true,
    };
    for (cmd, args) in [
        ("score.get", json!({"asset_id": id})),
        ("score.portfolio", json!({})),
        ("score.assets", json!({"asset_ids": [id]})),
    ] {
        let v = e.api.call_as(&reader, cmd, args).unwrap();
        assert!(!v.is_null(), "{cmd}");
    }
    let h = e
        .api
        .call("score.history", json!({"asset_id": id}))
        .unwrap();
    assert_eq!(h.as_array().unwrap().len(), 0, "{h}");
    // A interface grava o snapshot do dia.
    e.api.call("score.get", json!({"asset_id": id})).unwrap();
    let h = e
        .api
        .call("score.history", json!({"asset_id": id}))
        .unwrap();
    assert_eq!(h.as_array().unwrap().len(), 1, "{h}");
}

#[test]
fn scores_a_registered_stock_against_its_sector() {
    let e = env();
    seed_sector(&e.api);
    let id = create_asset(&e.api, "ENRG93");
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["asset_class"], "equity_br");
    assert_eq!(s["segment"], "utilities");
    assert_eq!(
        s["status"],
        "ok",
        "{}",
        serde_json::to_string_pretty(&s).unwrap()
    );
    if let Ok(p) = std::env::var("SCORE_SAMPLE_OUT") {
        std::fs::write(p, serde_json::to_string_pretty(&s).unwrap()).unwrap();
    }
    assert!(s["score"].as_f64().unwrap() > 0.0);
    assert_eq!(s["peers"]["sector_size"], 9);
    assert!(s["dimensions"].as_array().unwrap().len() >= 9);
    // Snapshot do dia e histórico.
    let h = e
        .api
        .call("score.history", json!({"asset_id": id}))
        .unwrap();
    assert_eq!(h.as_array().unwrap().len(), 1);
    // Pela carteira.
    let p = e.api.call("score.portfolio", json!({})).unwrap();
    assert_eq!(p[0]["ticker"], "ENRG93");
    assert!(p[0]["score"].is_number());
    // Pela lista de Ativos: o snapshot do dia, na ordem pedida; id desconhecido fica de fora.
    let l = e
        .api
        .call(
            "score.assets",
            json!({"asset_ids": ["nao-existe", id.clone()]}),
        )
        .unwrap();
    assert_eq!(l.as_array().unwrap().len(), 1);
    assert_eq!(l[0]["ticker"], "ENRG93");
    assert_eq!(l[0]["score"], s["score"]);
    // O universo do dia fica gravado.
    let n: i64 = e
        .api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT count(*) FROM score_universe WHERE class = 'equity_br'",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(n, 10);
    // Metodologia exposta para a interface.
    let m = e.api.call("score.methodology", json!({})).unwrap();
    assert_eq!(m["version"], "0.1.0");
}

#[test]
fn missing_critical_data_skips_the_score_and_records_feedback() {
    let e = env();
    seed_sector(&e.api);
    // Só dois exercícios: sem histórico de lucros (crítico).
    let short: Vec<Fact> = company(1.0, 0.1, 0.1)
        .into_iter()
        .filter(|x| x.end >= d("2024-12-31"))
        .collect();
    seed_company(&e.api, 50, "NOVA3", &short, "10", "Energia Elétrica");
    let id = create_asset(&e.api, "NOVA3");
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["status"], "insufficient_data");
    assert!(s["score"].is_null());
    assert_eq!(s["band_label"], "Dados insuficientes");
    let missing = s["missing"].as_array().unwrap();
    assert!(
        missing
            .iter()
            .any(|x| x["key"] == "loss_years_10y" && x["critical"] == true)
    );
    let fb = e.api.call("devfeedback.list", json!({})).unwrap();
    let titles: Vec<&str> = fb
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x["title"].as_str())
        .collect();
    assert!(titles.iter().any(|t| t.contains("NOVA3")), "{titles:?}");
    // Consultar de novo não duplica o feedback.
    e.api.call("score.get", json!({"asset_id": id})).unwrap();
    let fb2 = e.api.call("devfeedback.list", json!({})).unwrap();
    assert_eq!(fb.as_array().unwrap().len(), fb2.as_array().unwrap().len());
    // Outro ativo com dado faltando ganha o próprio feedback (não reforça o de NOVA3).
    seed_company(&e.api, 51, "NOVB3", &short, "11", "Energia Elétrica");
    let id_b = create_asset(&e.api, "NOVB3");
    e.api.call("score.get", json!({"asset_id": id_b})).unwrap();
    e.api.call("score.get", json!({"asset_id": id_b})).unwrap();
    let fb3 = e.api.call("devfeedback.list", json!({})).unwrap();
    let mine = |t: &str| {
        fb3.as_array()
            .unwrap()
            .iter()
            .find(|x| x["title"] == format!("Score de ativos: dados críticos ausentes em {t}"))
            .cloned()
            .unwrap_or_else(|| panic!("sem feedback de {t}: {fb3}"))
    };
    assert_eq!(mine("NOVA3")["bumps"], 0, "{fb3}");
    assert_eq!(mine("NOVB3")["bumps"], 0, "{fb3}");
    // Os dados chegam: o score sai e o feedback do ativo fecha sozinho.
    e.api
        .with_session(|s, _| {
            store::upsert_facts(
                s.db.conn(),
                &format!("{:014}", 50),
                &company(1.0, 0.1, 0.1),
                "2026-10-02T00:00:00Z",
            )
        })
        .unwrap();
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["status"], "ok", "{s}");
    let fb4 = e
        .api
        .call("devfeedback.list", json!({"status": "resolvido"}))
        .unwrap();
    let closed = fb4
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["title"] == "Score de ativos: dados críticos ausentes em NOVA3")
        .unwrap_or_else(|| panic!("{fb4}"));
    assert!(
        closed["body"]
            .as_str()
            .unwrap()
            .contains("Resolvido sozinho")
    );
}

#[test]
fn unsupported_classes_say_so_and_segment_override_applies() {
    let e = env();
    seed_sector(&e.api);
    let v = e
        .api
        .call(
            "assets.create",
            json!({"ticker": "OBRA", "name": "Obra de arte", "class": "other"}),
        )
        .unwrap();
    let s = e
        .api
        .call("score.get", json!({"asset_id": v["id"]}))
        .unwrap();
    assert_eq!(s["status"], "not_supported");
    // Override do segmento.
    let id = create_asset(&e.api, "ENRG53");
    e.api
        .call(
            "score.set_segment",
            json!({"ticker": "ENRG53", "class": "equity_br", "segment": "industrials"}),
        )
        .unwrap();
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["segment"], "industrials");
    assert!(
        e.api
            .call(
                "score.set_segment",
                json!({"ticker": "ENRG53", "class": "equity_br", "segment": "naoexiste"})
            )
            .is_err()
    );
    // Discordo, com motivo.
    let f = e
        .api
        .call("score.feedback.add", json!({"asset_id": id, "verdict": "discordo", "reason": "Pagou dividendos com dívida."}))
        .unwrap();
    assert!(f.is_string());
    let l = e
        .api
        .call("score.feedback.list", json!({"asset_id": id}))
        .unwrap();
    assert_eq!(l.as_array().unwrap().len(), 1);
}

#[test]
fn accepted_events_cap_the_score_and_pending_ones_do_not() {
    let e = env();
    seed_sector(&e.api);
    let id = create_asset(&e.api, "ENRG93");
    let before = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    let base = before["score"].as_f64().unwrap();
    assert!(base > 25.0);
    let today = crate::time::fmt_date(crate::time::today_local());
    let ev = e
        .api
        .call(
            "score.events.add",
            json!({"asset_id": id, "kind": "fraude", "title": "Reapresentação das demonstrações", "event_date": today, "evidence_url": "https://www.rad.cvm.gov.br/x", "accepted": false}),
        )
        .unwrap();
    let pending = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(pending["score"].as_f64().unwrap(), base);
    assert_eq!(pending["events"][0]["status"], "pending");
    e.api
        .call(
            "score.events.set_status",
            json!({"id": ev, "status": "accepted"}),
        )
        .unwrap();
    let after = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(after["score"].as_f64().unwrap(), 25.0);
    assert_eq!(after["band_label"], "Risco");
    assert!(
        after["flags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["key"].as_str().unwrap().starts_with("event:"))
    );
    e.api
        .call(
            "score.events.set_status",
            json!({"id": ev, "status": "resolved"}),
        )
        .unwrap();
    let resolved = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(resolved["score"].as_f64().unwrap(), base);
}

#[test]
fn secondary_source_fills_a_missing_official_metric() {
    let e = env();
    seed_sector(&e.api);
    // Sem receita no último exercício: margem líquida (crítica) some na fonte oficial.
    let mut f = company(1.0, 0.1, 0.1);
    f.retain(|x| !(x.item == "revenue" && x.end >= d("2024-01-01")));
    seed_company(&e.api, 60, "SECU3", &f, "10", "Energia Elétrica");
    let id = create_asset(&e.api, "SECU3");
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["status"], "insufficient_data");
    // Cache do Fundamentus com a margem líquida: preenche como fonte secundária.
    e.api
        .with_session(|s, _| {
            let snap = crate::research::fundamentus::FundamentusSnapshot {
                ticker: "SECU3".into(),
                fii: false,
                found: true,
                name: None,
                sector: None,
                subsector: None,
                price: Some(10.0),
                price_date: None,
                indicators: vec![crate::research::fundamentus::FundamentusItem {
                    label: "Marg. Líquida".into(),
                    text: "9,5%".into(),
                    value: Some(9.5),
                    hint: None,
                }],
                oscillations: vec![],
                dividends: vec![],
                fetched_at: crate::time::now_utc(),
                from_cache: false,
                source_url: String::new(),
            };
            crate::research::fundamentus::store(s.db.conn(), &snap)
        })
        .unwrap();
    let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_ne!(s["status"], "insufficient_data", "{}", s["band_meaning"]);
    let nm = s["dimensions"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|d| d["metrics"].as_array().unwrap().clone())
        .find(|m| m["key"] == "net_margin")
        .unwrap();
    assert!((nm["value"].as_f64().unwrap() - 0.095).abs() < 1e-9);
    assert!(nm["source"].as_str().unwrap().contains("fonte secundária"));
    assert!(
        s["notes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().contains("fonte secundária"))
    );
}

/// Alerta só quando a saúde piora entre dois scores definitivos.
#[test]
fn alerts_only_on_real_deterioration() {
    let e = env();
    seed_sector(&e.api);
    let id = create_asset(&e.api, "ENRG93");
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["status"], "ok");
    let score = s["score"].as_f64().unwrap();
    let yesterday = crate::time::fmt_date(crate::time::today_local() - chrono::Duration::days(1));
    // Snapshot de ontem com outro status/score/faixa (o payload é o de hoje ajustado).
    let set_previous = |status: &str, prev_score: Option<f64>, band: &str, label: &str| {
        let mut p = s.clone();
        p["status"] = json!(status);
        p["score"] = json!(prev_score);
        p["band"] = json!(band);
        p["band_label"] = json!(label);
        p["as_of"] = json!(yesterday);
        e.api
            .with_session(|ss, _| {
                let c = ss.db.conn();
                c.execute("DELETE FROM score_snapshot WHERE as_of = ?1", [&yesterday])?;
                c.execute("DELETE FROM alert", [])?;
                c.execute(
                    "INSERT INTO score_snapshot (subject, asset_id, ticker, as_of, methodology_version, status, score, band, confidence, payload, computed_at)
                     VALUES (?1, ?1, 'ENRG93', ?2, ?3, ?4, ?5, ?6, 1, ?7, 'x')",
                    rusqlite::params![id, yesterday, s["methodology_version"].as_str().unwrap(), status, prev_score, band, p.to_string()],
                )?;
                Ok(())
            })
            .unwrap();
    };
    let alerts = || -> Vec<String> {
        e.api.call("score.get", json!({"asset_id": id})).unwrap();
        e.api
            .with_session(|ss, _| {
                let c = ss.db.conn();
                let mut st = c.prepare("SELECT body FROM alert WHERE kind = 'asset_score'")?;
                Ok(st
                    .query_map([], |r| r.get(0))?
                    .collect::<rusqlite::Result<Vec<String>>>()?)
            })
            .unwrap()
    };
    // Saía de "Dados insuficientes": não é piora.
    set_previous(
        "insufficient_data",
        None,
        "dados_insuficientes",
        "Dados insuficientes",
    );
    assert!(alerts().is_empty());
    // Melhorou: não alerta.
    set_previous("ok", Some(score - 15.0), "risco", "Risco");
    assert!(alerts().is_empty());
    // Caiu 15 pontos entre dois scores definitivos: alerta.
    set_previous("ok", Some(score + 15.0), "robusto", "Robusto");
    let a = alerts();
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].contains("Score caiu 15,0 pontos"), "{a:?}");
}

/// Os textos do score saem do catálogo: em pt-BR exatamente como antes (inclusive as mensagens
/// com seletor e com várias linhas) e no idioma da instalação.
#[test]
fn score_texts_follow_the_locale() {
    use crate::i18n::{Locale, args, set_current, t, t_args};
    let body = t_args(
        "nucleo-score-feedback-corpo",
        &args(&[
            ("ativo", "PETR4".into()),
            ("classe", "Ações BR".into()),
            ("perfil", "Geral".into()),
            ("data", "2026-10-20".into()),
            (
                "itens",
                "- ROE (Rentabilidade): sem dado\n- ROIC (Rentabilidade): sem dado".into(),
            ),
            ("versao", "0.1.0".into()),
        ]),
    );
    assert_eq!(
        body,
        "O score de saúde de PETR4 (Ações BR, perfil Geral) não foi calculado em 2026-10-20 porque faltam dados críticos:\n- ROE (Rentabilidade): sem dado\n- ROIC (Rentabilidade): sem dado\n\nMetodologia 0.1.0. Ver o ativo em Ativos › PETR4 › Saúde."
    );
    let hacks = |n: i64| {
        t_args(
            "nucleo-score-cripto-nota-ataques",
            &args(&[("n", n.into())]),
        )
    };
    assert_eq!(hacks(0), "nenhum ataque à rede na lista da DefiLlama");
    assert_eq!(hacks(1), "1 ataque à rede na lista da DefiLlama");
    assert_eq!(hacks(3), "3 ataques à rede na lista da DefiLlama");
    let issuer = |parcial: &str| {
        t_args(
            "nucleo-score-rf-nota-score-emissor",
            &args(&[
                ("emissor", "ITUB4".into()),
                ("score", "71".into()),
                ("faixa", "Saudável".into()),
                ("saude", "68".into()),
                ("parcial", parcial.to_string().into()),
            ]),
        )
    };
    assert_eq!(issuer("nao"), "ITUB4: score 71 (Saudável), saúde 68");
    assert_eq!(
        issuer("sim"),
        "ITUB4: score 71 (Saudável), saúde 68; score parcial (muito peso imputado)"
    );
    let reg = |tipo: &str| {
        t_args(
            "nucleo-score-fundo-cnpj-fora-do-registro",
            &args(&[
                ("cnpj", "00.000.000/0001-00".into()),
                ("data", "2026-10-01".into()),
                ("tipo", tipo.to_string().into()),
            ]),
        )
    };
    assert_eq!(
        reg("fundo"),
        "o CNPJ 00.000.000/0001-00 não apareceu no registro de fundos da CVM baixado em 2026-10-01: confira o CNPJ no ativo"
    );
    assert!(reg("previdencia").ends_with("não o do plano na Susep"));
    assert_eq!(engine::fmt_value(super::Unit::Money, 1.5e9), "R$ 1,5 bi");
    let m = methodology();
    assert_eq!(m.band(80.0).label_text(), "Robusto");

    set_current(Locale::En);
    assert_eq!(engine::fmt_value(super::Unit::Money, 1.5e9), "R$ 1.5 bn");
    assert_eq!(engine::fmt_value(super::Unit::Days, 3.0), "3 days");
    assert_eq!(m.band(80.0).label_text(), "Robust");
    assert!(
        m.band(80.0)
            .meaning_text()
            .starts_with("Solid fundamentals")
    );
    assert_eq!(hacks(1), "1 network hack on the DefiLlama list");
    assert!(super::disclaimer().ends_with("Not investment advice."));
    assert_eq!(t("score-dados-insuficientes"), super::insufficient_label());
    assert_eq!(
        engine::join_pt(&["ROE".into(), "ROA".into(), "ROIC".into()]),
        "ROE, ROA and ROIC"
    );
    set_current(Locale::PtBr);
}

/// Ativo vendido (posição zerada): dado faltando nele não é lacuna a investigar; o aviso
/// aberto fecha sozinho e não volta (caso de um FII incorporado e fora da bolsa).
#[test]
fn closed_position_does_not_keep_a_missing_data_feedback() {
    let e = env();
    seed_sector(&e.api);
    let short: Vec<Fact> = company(1.0, 0.1, 0.1)
        .into_iter()
        .filter(|x| x.end >= d("2024-12-31"))
        .collect();
    seed_company(&e.api, 52, "VEND3", &short, "12", "Energia Elétrica");
    let id = create_asset(&e.api, "VEND3");
    e.api.call("score.get", json!({"asset_id": id})).unwrap();
    let open = |api: &Api| {
        api.call("devfeedback.list", json!({"open_only": true}))
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["title"].as_str().is_some_and(|t| t.contains("VEND3")))
            .count()
    };
    assert_eq!(open(&e.api), 1);
    let acc = e
        .api
        .call(
            "accounts.create",
            json!({"name": "Corretora", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .clone();
    for (date, side) in [("2025-03-03", "buy"), ("2025-06-02", "sell")] {
        e.api
            .call(
                "portfolio.create_trade",
                json!({"asset_id": id, "account_id": acc, "date": date, "side": side,
                       "quantity": "10", "price": "20"}),
            )
            .unwrap();
    }
    e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(open(&e.api), 0);
    let done = e
        .api
        .call("devfeedback.list", json!({"status": "resolvido"}))
        .unwrap();
    assert!(
        done.as_array().unwrap().iter().any(|x| x["body"]
            .as_str()
            .unwrap_or("")
            .contains("posição foi encerrada")),
        "{done}"
    );
    // Recalcular não reabre.
    e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(open(&e.api), 0);
}
