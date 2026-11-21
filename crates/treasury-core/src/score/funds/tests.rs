//! Tests of the fund and ETF metrics (synthetic series checked by hand) and of the end-to-end
//! score (temporary vault, no network).

use std::sync::Arc;

use chrono::{Datelike, Weekday};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde_json::{Value, json};

use super::*;
use crate::api::Api;
use crate::market::{PriceRow, upsert_prices};
use crate::paths::TreasuryPaths;

fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

fn dec(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap().round_dp(10)
}

/// Business days (Mon–Fri) from `from` to `to`.
fn business_days(from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
    let mut out = vec![];
    let mut x = from;
    while x <= to {
        if !matches!(x.weekday(), Weekday::Sat | Weekday::Sun) {
            out.push(x);
        }
        x += chrono::Duration::days(1);
    }
    out
}

#[test]
fn fund_segments_and_benchmarks_follow_the_cvm_class() {
    let info = |c: &str, a: &str, b: &str| registry::FundInfo {
        classification: Some(c.into()),
        anbima: Some(a.into()),
        benchmark: Some(b.into()),
        ..Default::default()
    };
    let rf = info("Renda Fixa", "Renda Fixa Simples", "DI de um dia");
    assert_eq!(fund_segment(AssetClass::Fund, Some(&rf)), "renda_fixa");
    assert_eq!(fund_benchmark("renda_fixa", Some(&rf)), ("cdi", "CDI"));
    let acoes = info("Ações", "Ações Livre", "Não se aplica");
    assert_eq!(fund_segment(AssetClass::Fund, Some(&acoes)), "acoes");
    assert_eq!(fund_benchmark("acoes", Some(&acoes)), ("ibov", "Ibovespa"));
    let ls = info("Multimercado", "Multimercados L/S - Neutro", "Ibovespa");
    assert_eq!(fund_segment(AssetClass::Fund, Some(&ls)), "multimercado");
    assert_eq!(fund_benchmark("multimercado", Some(&ls)).0, "ibov");
    let prev = info("Multimercado", "Previdência Multimercado Livre", "");
    assert_eq!(fund_segment(AssetClass::Fund, Some(&prev)), "previdencia");
    assert_eq!(fund_segment(AssetClass::Pension, None), "previdencia");
    assert_eq!(fund_segment(AssetClass::Fund, None), "multimercado");
    // Without a registry entry, by name (before falling back to multimercado).
    let named = |n: &str| fund_segment_named(AssetClass::Fund, None, n);
    assert_eq!(
        named("NU RESERVA IMEDIATA FICFI RF REFERENCIADO DI"),
        "renda_fixa"
    );
    assert_eq!(named("XP INVESTOR FIA"), "acoes");
    assert_eq!(named("BRASILPREV TOP FIC PREV"), "previdencia");
    assert_eq!(named("ALGUM MULTIMERCADO DI"), "multimercado");
    assert_eq!(named("FUNDO QUALQUER"), "multimercado");
    // With a registry entry, the CVM classification beats the name.
    assert_eq!(
        fund_segment_named(AssetClass::Fund, Some(&acoes), "RF DI"),
        "acoes"
    );
}

#[test]
fn etf_index_by_ticker_or_name_and_domicile() {
    let c = etf_config();
    assert_eq!(
        index_rule(c, Some("bova11"), &[]).and_then(|r| r.b3_index.as_deref()),
        Some("IBOV")
    );
    let r = index_rule(c, Some("XPTO11"), &["ISHARES SMALL CAP CLASSE DE ÍNDICE"]).unwrap();
    assert_eq!(r.b3_index.as_deref(), Some("SMLL"));
    assert_eq!(
        index_rule(c, Some("VOO"), &[]).and_then(|r| r.series.as_deref()),
        Some("sp500")
    );
    assert!(index_rule(c, Some("IMAB11"), &["It Now IMA-B"]).is_none());
    // Inheritance from the components: weighted average of only those that have a score.
    let h = |t: &str, w: f64| b3_index::Holding {
        ticker: t.into(),
        weight: w,
    };
    let holdings = [h("A3", 0.5), h("B3", 0.3), h("C3", 0.2)];
    let mut scores = |t: &str| match t {
        "A3" => Some(80.0),
        "B3" => Some(40.0),
        _ => None,
    };
    let (avg, cov, n) = components_score(&holdings, &mut scores);
    assert!((avg.unwrap() - (0.5 * 80.0 + 0.3 * 40.0) / 0.8).abs() < 1e-9);
    assert!((cov - 0.8).abs() < 1e-12);
    assert_eq!(n, 2);
}

/// Daily quotas with a fixed yield per day and a 10% drop in the middle, and a constant CDI.
fn synthetic_fund(today: NaiveDate) -> (Vec<Point>, Vec<Point>) {
    let days = business_days(add_months(today, -25), today);
    let mid = days.len() / 2;
    let mut q = vec![];
    let mut lvl = 1.0;
    for (i, day) in days.iter().enumerate() {
        if i == mid {
            lvl *= 0.9;
        } else if i > 0 {
            lvl *= 1.0008;
        }
        q.push((*day, lvl));
    }
    let rates: Vec<(NaiveDate, f64)> = days.iter().map(|x| (*x, 0.05)).collect();
    (q, series::level_from_daily_rates(&rates))
}

#[test]
fn fund_metrics_from_quotes_and_cdi() {
    let today = d("2026-10-16");
    let (q, cdi) = synthetic_fund(today);
    let info = registry::FundInfo {
        admin_fee: Some("0.015".into()),
        perf_fee: Some("0.2".into()),
        perf_benchmark: Some("CDI".into()),
        redemption_days: Some(31),
        inception: Some("2016-10-16".into()),
        ..Default::default()
    };
    let o = fund_metrics(&FundInput {
        quotes: &q,
        cdi: &cdi,
        bench: &cdi,
        bench_label: "CDI",
        info: Some(&info),
        facts: &[],
        today,
    });
    let v = |k: &str| o.metrics[k].value;
    // The 10% drop in a single day: 10% drawdown; recovery at 0.08%/day takes
    // ln(1/0.9)/ln(1.0008) ≈ 132 business days.
    assert!((v("fund_max_drawdown").unwrap() - 0.1).abs() < 1e-9);
    let rec = v("fund_recovery_days").unwrap();
    assert!((180.0..=190.0).contains(&rec), "{rec}");
    // Daily return 0.08% against CDI 0.05%: above CDI only in the windows without the drop
    // (the one at the end; the drop is in the middle of the 25 months).
    let c = v("fund_consistency").unwrap();
    assert!(c > 0.0 && c < 0.2, "{c}");
    assert!(v("fund_sharpe").is_some() && v("fund_sortino").is_some());
    assert_eq!(v("fund_admin_fee"), Some(0.015));
    assert_eq!(v("fund_perf_fee"), Some(0.2));
    assert_eq!(v("fund_redemption_days"), Some(31.0));
    assert!((v("fund_age_years").unwrap() - 10.0).abs() < 0.01);
    assert!(
        o.metrics["fund_perf_fee"]
            .note
            .as_deref()
            .unwrap()
            .contains("CDI")
    );
    // The benchmark is CDI: no information ratio.
    assert!(o.metrics["fund_information_ratio"].missing.is_some());
    // Excess over annualized CDI: (1.0008/1.0005)^(observations per year) − 1, minus the
    // drop; positive.
    assert!(v("fund_excess_return").unwrap() > 0.0);
    // Without the daily-report summary: net assets and shareholders are missing, with the reason.
    assert!(o.metrics["fund_nav"].missing.is_some());
    assert!(o.notes[0].starts_with("Retorno da cota: 12 meses"));
    // Less than 12 months of quotas: the critical ones are missing.
    let short: Vec<Point> = q
        .iter()
        .filter(|p| p.0 > add_months(today, -6))
        .copied()
        .collect();
    let o = fund_metrics(&FundInput {
        quotes: &short,
        cdi: &cdi,
        bench: &cdi,
        bench_label: "CDI",
        info: None,
        facts: &[],
        today,
    });
    assert!(
        o.metrics["fund_sharpe"]
            .missing
            .as_deref()
            .unwrap()
            .contains("12 meses")
    );
    assert!(o.metrics["fund_max_drawdown"].missing.is_some());
}

// ---------------------------------------------------------------------------
// End to end

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

fn create(api: &Api, input: Value) -> String {
    let v = api.call("assets.create", input).unwrap();
    let id = v["id"].as_str().unwrap().to_string();
    api.call("portfolio.watch", json!({"asset_id": id}))
        .unwrap();
    id
}

fn seed_prices(api: &Api, asset: &str, pts: &[Point], volume: Option<f64>) {
    api.with_session(|s, _| {
        let rows: Vec<PriceRow> = pts
            .iter()
            .map(|(d, p)| PriceRow {
                date: *d,
                open: None,
                high: None,
                low: None,
                close: dec(*p),
                adj_close: None,
                volume: volume.map(dec),
            })
            .collect();
        upsert_prices(s.db.conn(), asset, &rows, "teste")?;
        Ok(())
    })
    .unwrap();
}

fn seed_index(api: &Api, series_id: &str, pts: &[(NaiveDate, f64)]) {
    api.with_session(|s, _| {
        let v: Vec<(NaiveDate, Decimal, bool)> =
            pts.iter().map(|(d, x)| (*d, dec(*x), false)).collect();
        crate::indices::upsert_values(s.db.conn(), series_id, &v, "teste")?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn scores_a_fund_from_its_quota_history() {
    let e = env();
    let today = crate::time::today_local();
    let (q, _) = synthetic_fund(today);
    let id = create(
        &e.api,
        json!({"name": "Fundo Teste RF", "class": "fund", "cnpj": "32.893.503/0001-20"}),
    );
    seed_prices(&e.api, &id, &q, None);
    let days: Vec<NaiveDate> = q.iter().map(|p| p.0).collect();
    seed_index(
        &e.api,
        "cdi",
        &days.iter().map(|x| (*x, 0.05)).collect::<Vec<_>>(),
    );
    // CVM registry and the monthly summary of the daily report (12 months of inflows).
    e.api
        .with_session(|s, _| {
            let c = s.db.conn();
            let mut reg = registry::Registry::default();
            reg.funds.insert(
                "32893503000120".into(),
                registry::FundInfo {
                    cnpj: "32893503000120".into(),
                    name: "FUNDO TESTE RF".into(),
                    classification: Some("Renda Fixa".into()),
                    benchmark: Some("DI de um dia".into()),
                    inception: Some("2019-10-03".into()),
                    manager: Some("GESTORA TESTE LTDA.".into()),
                    admin_fee: Some("0.002".into()),
                    perf_fee: Some("0".into()),
                    redemption_days: Some(1),
                    ..Default::default()
                },
            );
            registry::save(c, &reg)?;
            let mut months = cvm::Months::new();
            let by = months.entry("32893503000120".into()).or_default();
            for k in 0..12 {
                let m = add_months(today, -k);
                by.insert(
                    (m.year(), m.month()),
                    cvm::FundMonth {
                        last: m,
                        nav: Some(Decimal::from(1_000_000_000)),
                        holders: Some(30_000),
                        inflow: Decimal::from(20_000_000),
                        outflow: Decimal::from(10_000_000),
                        days: 20,
                    },
                );
            }
            store_fund_months(c, &months)?;
            assert!(has_fund_months(c, "32893503000120")?);
            Ok(())
        })
        .unwrap();
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["asset_class"], "fund");
    assert_eq!(s["segment"], "renda_fixa");
    assert_eq!(
        s["status"],
        "ok",
        "{}",
        serde_json::to_string_pretty(&s).unwrap()
    );
    assert!(s["score"].as_f64().unwrap() > 0.0);
    let metric = |k: &str| -> Value {
        s["dimensions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|d| d["metrics"].as_array().unwrap().iter())
            .find(|m| m["key"] == k)
            .cloned()
            .unwrap_or(Value::Null)
    };
    assert_eq!(metric("fund_holders")["value"], 30000.0);
    assert_eq!(metric("fund_nav")["value"], 1e9);
    // 12 months of R$ 10 million net on R$ 1 billion.
    assert!((metric("fund_net_flow_12m")["value"].as_f64().unwrap() - 0.12).abs() < 1e-9);
    // A 10% drop in a fixed-income fund: below the 20% red-flag limit.
    assert!(s["flags"].as_array().unwrap().is_empty(), "{}", s["flags"]);
    let notes = s["notes"].to_string();
    assert!(notes.contains("Benchmark: CDI"), "{notes}");
    assert!(notes.contains("GESTORA TESTE LTDA."), "{notes}");
    assert!(!notes.contains("LTDA.."), "{notes}");
    // No peers: confidence below 0.7.
    assert!(s["confidence"].as_f64().unwrap() <= 0.7);
}

#[test]
fn fund_without_twelve_months_of_quotas_has_no_score() {
    let e = env();
    let today = crate::time::today_local();
    let (q, _) = synthetic_fund(today);
    let short: Vec<Point> = q
        .iter()
        .filter(|p| p.0 > add_months(today, -5))
        .copied()
        .collect();
    let id = create(
        &e.api,
        json!({"name": "Fundo Novo", "class": "fund", "cnpj": "11.111.111/0001-11"}),
    );
    seed_prices(&e.api, &id, &short, None);
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["status"], "insufficient_data");
    assert!(
        s["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["key"] == "fund_sharpe" && m["critical"] == true)
    );
    // No quotas at all: the reason says what to do.
    let id2 = create(&e.api, json!({"name": "Fundo Sem Cotas", "class": "fund"}));
    let s: Value = e.api.call("score.get", json!({"asset_id": id2})).unwrap();
    assert_eq!(s["status"], "insufficient_data");
    assert!(
        s["band_meaning"].as_str().unwrap().contains("CNPJ"),
        "{}",
        s["band_meaning"]
    );
}

#[test]
fn scores_an_equity_etf_with_tracking_and_composition() {
    let e = env();
    let today = crate::time::today_local();
    let days = business_days(add_months(today, -13), today);
    // ETF yields 0.05%/day; index 0.051%/day (tracking difference ≈ −2.5% over 12 months).
    let mut etf = vec![];
    let mut idx = vec![];
    let (mut a, mut b) = (100.0, 100_000.0);
    for (i, day) in days.iter().enumerate() {
        if i > 0 {
            let wobble = if i % 2 == 0 { 1.01 } else { 1.0 / 1.01 };
            a *= 1.0005 * wobble;
            b *= 1.00051 * wobble;
        }
        etf.push((*day, a));
        idx.push((*day, b));
    }
    let id = create(
        &e.api,
        json!({"ticker": "BOVA11", "name": "iShares Ibovespa", "class": "etf_br", "market": "B3"}),
    );
    seed_prices(&e.api, &id, &etf, Some(5_000_000.0));
    seed_index(&e.api, "ibov", &idx);
    e.api
        .with_session(|s, _| {
            let c = s.db.conn();
            b3_index::apply(
                c,
                "IBOV",
                b3_index::parse(b3_index::tests::FIXTURE.as_bytes())
                    .map_err(crate::error::Error::invalid)?,
                &crate::time::now_utc(),
            )?;
            Ok(())
        })
        .unwrap();
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["asset_class"], "etf");
    assert_eq!(s["segment"], "acoes_br");
    let metric = |k: &str| -> Value {
        s["dimensions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|d| d["metrics"].as_array().unwrap().iter())
            .find(|m| m["key"] == k)
            .cloned()
            .unwrap_or(Value::Null)
    };
    let td = metric("etf_tracking_diff")["value"].as_f64().unwrap();
    assert!(td < 0.0 && td > -0.01, "{td}");
    // Same swings in both: tracking error close to zero.
    assert!(metric("etf_tracking_error")["value"].as_f64().unwrap() < 0.001);
    assert_eq!(metric("etf_holdings")["value"], 3.0);
    assert_eq!(metric("etf_domicile")["value"], 80.0);
    // No component scores (no stock universe): the inheritance is missing, but not critical.
    assert!(metric("etf_components_score")["missing"].is_string());
    assert!(
        metric("etf_ter")["missing"]
            .as_str()
            .unwrap()
            .contains("fonte")
    );
    assert_eq!(
        s["status"],
        "ok",
        "{}",
        serde_json::to_string_pretty(&s).unwrap()
    );
    // Volume: R$ 5 million × price.
    assert!(metric("etf_avg_volume")["value"].as_f64().unwrap() > 1e8);
}

// ---------------------------------------------------------------------------
// Collection (fixtures, no network)

/// Fake downloads by URL (anything not in the map is a 404).
pub(crate) struct Fixtures {
    pub files: std::collections::HashMap<String, Vec<u8>>,
    pub errors: std::collections::HashMap<String, String>,
    pub calls: std::sync::Mutex<Vec<String>>,
}

impl Fixtures {
    pub fn new(files: Vec<(String, Vec<u8>)>) -> Self {
        Self {
            files: files.into_iter().collect(),
            errors: Default::default(),
            calls: Default::default(),
        }
    }
}

impl crate::research::net::Fetcher for Fixtures {
    fn get(
        &self,
        url: &str,
        _cond: Option<&crate::research::net::FileMeta>,
        _ua: &str,
    ) -> crate::research::net::FetchResult {
        self.calls.lock().unwrap().push(url.to_string());
        if let Some(e) = self.errors.get(url) {
            return Err(e.clone());
        }
        Ok(match self.files.get(url) {
            Some(b) => crate::research::net::Fetched::Body {
                bytes: b.clone(),
                meta: Default::default(),
            },
            None => crate::research::net::Fetched::NotFound,
        })
    }
    fn post(&self, url: &str, body: &str, ua: &str) -> crate::research::net::FetchResult {
        self.calls.lock().unwrap().push(format!("POST {body}"));
        self.get(url, None, ua)
    }
}

#[test]
fn sync_fetches_registry_extrato_etf_names_and_portfolios() {
    use crate::research::sync::{SyncArgs, SyncSource, run_with};
    let e = env();
    let today = crate::time::today_local();
    let trend = create(
        &e.api,
        json!({"name": "Trend", "class": "fund", "cnpj": "32.893.503/0001-20"}),
    );
    let bova = create(
        &e.api,
        json!({"ticker": "BOVA11", "name": "iShares Ibovespa", "class": "etf_br", "market": "B3"}),
    );
    let extrato = "TP_FUNDO_CLASSE;CNPJ_FUNDO_CLASSE;DENOM_SOCIAL;DT_COMPTC;TAXA_ADM;EXISTE_TAXA_PERFM;TAXA_PERFM;PARAM_TAXA_PERFM;QT_DIA_CONVERSAO_COTA;QT_DIA_PAGTO_RESGATE\n\
        FI;32.893.503/0001-20;TREND;2021-09-06;0.020000;N;;;0;0\n";
    let yahoo = r#"{"chart":{"result":[{"meta":{"longName":"ISHARES IBOVESPA CLASSE DE ÍNDICE - RESPONSABILIDADE LIMITADA"}}]}}"#;
    let f = Fixtures::new(vec![
        (
            registry::REGISTRO_URL.into(),
            registry::tests::registro_zip(),
        ),
        (registry::EXTRATO_URL.into(), extrato.as_bytes().to_vec()),
        (
            registry::yahoo_meta_url("BOVA11.SA"),
            yahoo.as_bytes().to_vec(),
        ),
        (
            b3_index::url("IBOV"),
            b3_index::tests::FIXTURE.as_bytes().to_vec(),
        ),
    ]);
    let args = SyncArgs {
        force: None,
        sources: Some(vec![SyncSource::Funds]),
    };
    let rep = run_with(&e.api, &args, &f, today).unwrap();
    assert!(rep.ok, "{:?}", rep.items);
    let labels: Vec<&str> = rep.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(labels.len(), 4, "{labels:?}");
    let (reg, ports) = e
        .api
        .with_session(|s, _| Ok((registry::load(s.db.conn())?, b3_index::load(s.db.conn())?)))
        .unwrap();
    assert_eq!(
        reg.etfs.get(&bova).map(String::as_str),
        Some("10406511000161")
    );
    let t = &reg.funds["32893503000120"];
    assert_eq!(t.admin_fee.as_deref(), Some("0.0002"));
    assert_eq!(t.classification.as_deref(), Some("Renda Fixa"));
    assert_eq!(ports["IBOV"].holdings.len(), 3);
    // Next collection on the same day: nothing due, no downloads.
    let n = f.calls.lock().unwrap().len();
    run_with(&e.api, &args, &f, today).unwrap();
    assert_eq!(f.calls.lock().unwrap().len(), n);
    let _ = trend;
}

/// Fund without quotas: the reason follows the source the collection would actually use
/// (pension with the automatic provider already uses the CVM; a CNPJ outside the registry is
/// probably the plan's) and, without a CNPJ, suggests the likely one by name.
#[test]
fn fund_without_quotes_says_the_real_reason() {
    let e = env();
    let reason = |id: &str| {
        let s = e.api.call("score.get", json!({"asset_id": id})).unwrap();
        assert_eq!(s["status"], "insufficient_data", "{s}");
        s["missing"][0]["reason"].as_str().unwrap().to_string()
    };
    let prev = create(
        &e.api,
        json!({"name": "PREV TESTE", "class": "pension", "cnpj": "11.111.111/0001-11"}),
    );
    let nu = create(
        &e.api,
        json!({"name": "NU RESERVA IMEDIATA FICFI RF REFERENCIADO DI", "class": "fund"}),
    );
    // Before the CVM registry: no conclusion about the CNPJ.
    assert_eq!(
        reason(&prev),
        "sem cotas gravadas (rode a coleta de cotações)"
    );
    assert!(
        reason(&nu).starts_with("fundo sem CNPJ: informe"),
        "{}",
        reason(&nu)
    );
    e.api
        .with_session(|s, _| {
            let mut reg = registry::Registry {
                registro_at: Some("2026-10-19T10:00:00Z".into()),
                ..Default::default()
            };
            reg.funds.insert(
                "22222222000122".into(),
                registry::FundInfo {
                    cnpj: "22222222000122".into(),
                    name: "NU RESERVA IMEDIATA FIC FIF RENDA FIXA REFERENCIADO DI".into(),
                    classification: Some("Renda Fixa".into()),
                    ..Default::default()
                },
            );
            reg.fund_guesses.insert(nu.clone(), "22222222000122".into());
            registry::save(s.db.conn(), &reg)?;
            Ok(())
        })
        .unwrap();
    let r = reason(&prev);
    assert!(
        r.contains("11.111.111/0001-11") && r.contains("2026-10-19") && r.contains("FIE"),
        "{r}"
    );
    let r = reason(&nu);
    assert!(
        r.contains("22.222.222/0001-22") && r.contains("confira"),
        "{r}"
    );
    let s = e.api.call("score.get", json!({"asset_id": nu})).unwrap();
    assert_eq!(s["segment"], "renda_fixa", "{s}");
}

/// Pension fund (EFPC, the Quanta Previdência case): without an imported statement, the score
/// asks for the statement's quotas (not the CNPJ of a CVM fund) and opens no developer
/// notice; with the statement's quotas, the gap goes away and the analysis proceeds.
#[test]
fn pension_fund_plan_asks_for_the_statement_and_opens_no_dev_feedback() {
    let e = env();
    let today = crate::time::today_local();
    let id = create(
        &e.api,
        json!({"name": "PLANOPREV-Arrojado", "class": "pension", "cnpj": "11.444.777/0001-61"}),
    );
    e.api
        .with_session(|s, _| {
            Ok(s.db.conn().execute(
                "INSERT INTO pension_plan (asset_id, plan_type, created_at, updated_at)
                 VALUES (?1, 'efpc', 'x', 'x')",
                [&id],
            )?)
        })
        .unwrap();
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert_eq!(s["status"], "insufficient_data");
    let missing = s["missing"].as_array().unwrap();
    assert!(
        missing.iter().any(|m| m["key"] == "plan_statement"),
        "{missing:?}"
    );
    let why = s["band_meaning"].as_str().unwrap();
    assert!(why.contains("extrato") && !why.contains("FIE"), "{why}");
    let open: i64 = e
        .api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT count(*) FROM dev_feedback WHERE status IN ('aberto', 'em_andamento')",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(open, 0, "falta do usuário, não da coleta");
    // With the statement's quotas (manual price), the quotas stop being missing and the analysis
    // proceeds.
    let (q, _) = synthetic_fund(today);
    seed_prices(&e.api, &id, &q, None);
    let s: Value = e.api.call("score.get", json!({"asset_id": id})).unwrap();
    assert!(
        !s["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["key"] == "plan_statement"),
        "{}",
        s["missing"]
    );
}
