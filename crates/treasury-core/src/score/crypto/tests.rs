//! Tests of the crypto and stablecoin metrics (synthetic data) and of the end-to-end score
//! (temporary vault, no network).

use std::sync::Arc;

use serde_json::{Value, json};

use super::*;
use crate::api::Api;
use crate::market::{PriceRow, upsert_prices};
use crate::paths::TreasuryPaths;

fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

fn facts_for(c: &coingecko::Coin, today: NaiveDate) -> Vec<Fact> {
    let mut v = vec![];
    let mut push = |k: &str, x: Option<f64>, cur: &str| {
        if let Some(f) = x.and_then(|x| fact(k, today, x, cur, "coingecko", "teste")) {
            v.push(f);
        }
    };
    push(keys::MCAP, c.market_cap_usd, "USD");
    push(keys::FDV, c.fdv_usd, "USD");
    push(keys::VOLUME, c.volume_usd, "USD");
    push(keys::RANK, c.rank, "count");
    push(keys::CIRCULATING, c.circulating, "count");
    push(keys::MAX_SUPPLY, c.max_supply, "count");
    push(keys::WATCHLIST, c.watchlist_users, "count");
    push(keys::COMMITS, c.commits_4w, "count");
    push(keys::STARS, c.stars, "count");
    push(keys::ATH_DRAWDOWN, c.ath_drawdown, "ratio");
    v
}

/// Daily prices swinging ±3% (high volatility) for 365 days.
fn daily(today: NaiveDate, base: f64, swing: f64) -> Vec<Point> {
    (0..365)
        .map(|i| {
            let k = if i % 2 == 0 { 1.0 + swing } else { 1.0 };
            (today - chrono::Duration::days(364 - i), base * k)
        })
        .collect()
}

#[test]
fn crypto_metrics_by_hand() {
    let today = d("2026-10-16");
    let c = coingecko::parse_coin(coingecko::tests::COIN.as_bytes()).unwrap();
    let mut facts = facts_for(&c, today);
    facts.push(
        fact(
            keys::SUPPLY_GROWTH,
            today,
            0.008,
            "ratio",
            "coingecko",
            "teste",
        )
        .unwrap(),
    );
    facts.push(fact(keys::TVL, today, 4.3e9, "USD", "defillama", "teste").unwrap());
    let meta = CoinMeta {
        genesis_date: Some("2009-01-03".into()),
        ..Default::default()
    };
    let prices = daily(today, 400_000.0, 0.03);
    let o = crypto_metrics(&CryptoInput {
        facts: &facts,
        meta: Some(&meta),
        stable: None,
        prices: &prices,
        price_currency: "BRL",
        usd_brl: Some((5.0, "2026-10-16".into())),
        today,
    });
    let v = |k: &str| o.metrics[k].value;
    // US$ 1.6 trillion × 5 = R$ 8 trillion.
    assert!((v("crypto_market_cap").unwrap() - 8e12).abs() < 1.0);
    assert!((v("crypto_fdv_to_mcap").unwrap() - 1.05).abs() < 1e-12);
    assert!((v("crypto_supply_emitted").unwrap() - 19.9 / 21.0).abs() < 1e-12);
    assert!((v("crypto_turnover").unwrap() - 0.025).abs() < 1e-12);
    assert!((v("crypto_tvl").unwrap() - 2.15e10).abs() < 1.0);
    assert!((v("crypto_ath_drawdown").unwrap() - 0.365).abs() < 1e-12);
    let age = (today - d("2009-01-03")).num_days() as f64 / 365.25;
    assert!((v("crypto_age_years").unwrap() - age).abs() < 1e-12);
    // Rises 3% and comes back every day: drawdown 1 − 1/1.03.
    assert!((v("crypto_max_drawdown").unwrap() - (1.0 - 1.0 / 1.03)).abs() < 1e-9);
    assert!(v("crypto_volatility").unwrap() > 0.5);
    assert!(o.notes.iter().any(|n| n.contains("câmbio")));
    // No exchange rate: the amounts in reais are missing, with the reason.
    let o2 = crypto_metrics(&CryptoInput {
        facts: &facts,
        meta: None,
        stable: None,
        prices: &[],
        price_currency: "BRL",
        usd_brl: None,
        today,
    });
    assert!(
        o2.metrics["crypto_market_cap"]
            .missing
            .as_deref()
            .unwrap()
            .contains("dólar")
    );
    assert!(o2.metrics["crypto_volatility"].missing.is_some());
}

#[test]
fn stablecoin_parity_from_usd_history() {
    let today = d("2026-10-16");
    let mut facts = vec![];
    for i in 0..300 {
        let px = if i == 150 {
            0.96
        } else {
            1.0 + if i % 2 == 0 { 0.001 } else { -0.001 }
        };
        facts.push(
            fact(
                keys::PRICE_DAILY,
                today - chrono::Duration::days(299 - i),
                px,
                "USD",
                "defillama",
                "teste",
            )
            .unwrap(),
        );
    }
    let stable = defillama::parse_stables(defillama::tests::STABLES.as_bytes()).unwrap()[0].clone();
    let o = crypto_metrics(&CryptoInput {
        facts: &facts,
        meta: None,
        stable: Some(&stable),
        prices: &[],
        price_currency: "BRL",
        usd_brl: Some((5.0, "2026-10-16".into())),
        today,
    });
    let v = |k: &str| o.metrics[k].value.unwrap();
    assert!((v("stable_max_deviation") - 0.04).abs() < 1e-9);
    // One day above 0.5% in 300 (the window is 12 months).
    assert!((v("stable_depeg_days") - 1.0 / 300.0).abs() < 1e-9);
    assert!((v("stable_mean_deviation") - (299.0 * 0.001 + 0.04) / 300.0).abs() < 1e-9);
    assert_eq!(v("stable_backing"), 75.0);
    // Market cap from DefiLlama's circulating supply (no CoinGecko).
    assert!((v("crypto_market_cap") - 183489807303.1 * 0.9993 * 5.0).abs() < 1.0);
    let m = crate::score::methodology();
    let flags = crypto_flags(m, true, Some("algorithmic"), &o.metrics);
    let keys: Vec<&str> = flags.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(keys, vec!["stable_depeg", "stable_algorithmic"]);
}

#[test]
fn ids_from_the_asset_the_table_and_the_lists() {
    let mut a = crate::assets::Asset {
        id: "a".into(),
        ticker: Some("btc-usd".into()),
        name: "Bitcoin".into(),
        official_name: None,
        account_id: None,
        class: crate::assets::AssetClass::Crypto,
        class_label: String::new(),
        market: None,
        currency: "USD".into(),
        country: None,
        isin: None,
        cnpj: None,
        treasury_code: None,
        sector: None,
        segment: None,
        price_provider: None,
        provider_symbol: None,
        archived: false,
        notes: None,
        fixed_income: None,
        exchange: None,
        quote_symbol: None,
    };
    let reg = CryptoRegistry::default();
    assert_eq!(symbol_of(&a).as_deref(), Some("BTC"));
    assert_eq!(local_id(&a, &reg).as_deref(), Some("bitcoin"));
    a.price_provider = Some("coingecko".into());
    a.provider_symbol = Some("wrapped-bitcoin".into());
    assert_eq!(local_id(&a, &reg).as_deref(), Some("wrapped-bitcoin"));
    // Symbol not in the table: DefiLlama's list of chains.
    let mut reg = CryptoRegistry::default();
    reg.chains
        .insert("hyperliquid".into(), ("HYPE".into(), 1e9));
    a.price_provider = None;
    a.ticker = Some("HYPE".into());
    assert_eq!(local_id(&a, &reg).as_deref(), Some("hyperliquid"));
    assert!(is_stablecoin(&reg, None, Some("USDC")));
    assert!(!is_stablecoin(&reg, Some("hyperliquid"), Some("HYPE")));
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

fn seed(api: &Api, asset: &str, id: &str, facts: &[Fact], prices: &[Point]) {
    api.with_session(|s, _| {
        let c = s.db.conn();
        store::upsert_facts(c, &issuer(id), facts, "2026-10-16T00:00:00Z")?;
        let rows: Vec<PriceRow> = prices
            .iter()
            .map(|(d, p)| PriceRow {
                date: *d,
                open: None,
                high: None,
                low: None,
                close: Decimal::from_f64(*p).unwrap().round_dp(8),
                adj_close: None,
                volume: None,
            })
            .collect();
        upsert_prices(c, asset, &rows, "teste")?;
        crate::fx::set_manual(
            c,
            &crate::fx::FxRateInput {
                base: "USD".into(),
                quote: "BRL".into(),
                date: fmt_date(crate::time::today_local()),
                rate: "5".into(),
            },
        )?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn scores_a_crypto_and_a_stablecoin() {
    let e = env();
    let today = crate::time::today_local();
    let btc = create(
        &e.api,
        json!({"ticker": "BTC", "name": "Bitcoin", "class": "crypto", "market": "CRYPTO"}),
    );
    let c = coingecko::parse_coin(coingecko::tests::COIN.as_bytes()).unwrap();
    let mut facts = facts_for(&c, today);
    facts.push(
        fact(
            keys::SUPPLY_GROWTH,
            today,
            0.008,
            "ratio",
            "coingecko",
            "teste",
        )
        .unwrap(),
    );
    facts.push(fact(keys::TVL, today, 4.3e9, "USD", "defillama", "teste").unwrap());
    seed(
        &e.api,
        &btc,
        "bitcoin",
        &facts,
        &daily(today, 400_000.0, 0.02),
    );
    e.api
        .with_session(|s, _| {
            let mut reg = load(s.db.conn())?;
            reg.coins.insert(
                "bitcoin".into(),
                CoinMeta {
                    name: "Bitcoin".into(),
                    symbol: "BTC".into(),
                    genesis_date: Some("2009-01-03".into()),
                    ..Default::default()
                },
            );
            save(s.db.conn(), &reg)?;
            Ok(())
        })
        .unwrap();
    let s: Value = e.api.call("score.get", json!({"asset_id": btc})).unwrap();
    assert_eq!(s["asset_class"], "crypto");
    assert_eq!(s["segment"], "default");
    assert_eq!(
        s["status"],
        "ok",
        "{}",
        serde_json::to_string_pretty(&s).unwrap()
    );
    assert!(
        s["notes"]
            .to_string()
            .contains("não se compara diretamente com ações")
    );
    // Without the paid on-chain data (active addresses, concentration, hacks…): unavailable.
    let missing: Vec<&str> = s["missing"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["key"].as_str())
        .collect();
    assert!(missing.contains(&"crypto_active_addresses"), "{missing:?}");

    // Stablecoin: its own segment from the symbol table, parity from the dollar prices.
    let usdt = create(
        &e.api,
        json!({"ticker": "USDT", "name": "Tether", "class": "crypto", "market": "CRYPTO"}),
    );
    let t = coingecko::parse_coin(coingecko::tests::STABLE.as_bytes()).unwrap();
    let mut facts = facts_for(&t, today);
    for i in 0..300 {
        facts.push(
            fact(
                keys::PRICE_DAILY,
                today - chrono::Duration::days(299 - i),
                1.0005,
                "USD",
                "defillama",
                "teste",
            )
            .unwrap(),
        );
    }
    seed(&e.api, &usdt, "tether", &facts, &[]);
    e.api
        .with_session(|s, _| {
            let mut reg = load(s.db.conn())?;
            let st = defillama::parse_stables(defillama::tests::STABLES.as_bytes())
                .map_err(crate::error::Error::invalid)?;
            reg.stables.insert("tether".into(), st[0].clone());
            save(s.db.conn(), &reg)?;
            Ok(())
        })
        .unwrap();
    let s: Value = e.api.call("score.get", json!({"asset_id": usdt})).unwrap();
    assert_eq!(s["segment"], "stablecoin");
    assert_eq!(
        s["status"],
        "ok",
        "{}",
        serde_json::to_string_pretty(&s).unwrap()
    );
    let parity = s["dimensions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["key"] == "stable_paridade")
        .unwrap();
    assert!(parity["score"].as_f64().unwrap() > 90.0);
    // No dollar prices: the (critical) parity is missing.
    let usdc = create(
        &e.api,
        json!({"ticker": "USDC", "name": "USD Coin", "class": "crypto", "market": "CRYPTO"}),
    );
    let s: Value = e.api.call("score.get", json!({"asset_id": usdc})).unwrap();
    assert_eq!(s["status"], "insufficient_data");
    assert!(s["band_meaning"].as_str().unwrap().contains("paridade"));
}

#[test]
fn sync_collects_coingecko_and_defillama_and_backs_off() {
    use crate::research::sync::{SyncArgs, SyncSource, run_with};
    use crate::score::funds::tests::Fixtures;
    let e = env();
    let today = crate::time::today_local();
    let btc = create(
        &e.api,
        json!({"ticker": "BTC", "name": "Bitcoin", "class": "crypto", "market": "CRYPTO"}),
    );
    create(
        &e.api,
        json!({"ticker": "USDT", "name": "Tether", "class": "crypto", "market": "CRYPTO"}),
    );
    let day = 86_400_000i64;
    let chart = format!(
        r#"{{"prices":[[0,10.0],[{a},20.0]],"market_caps":[[0,1000.0],[{a},2040.0]]}}"#,
        a = 365 * day
    );
    let llama_chart =
        r#"{"coins":{"coingecko:x":{"prices":[{"timestamp":1792000019,"price":1.0}]}}}"#;
    let mut f = Fixtures::new(vec![
        (
            defillama::CHAINS_URL.into(),
            defillama::tests::CHAINS.as_bytes().to_vec(),
        ),
        (
            defillama::STABLES_URL.into(),
            defillama::tests::STABLES.as_bytes().to_vec(),
        ),
        (
            coingecko::coin_url("bitcoin"),
            coingecko::tests::COIN.as_bytes().to_vec(),
        ),
        (coingecko::chart_url("bitcoin"), chart.into_bytes()),
        (
            defillama::chart_url("bitcoin", today),
            llama_chart.as_bytes().to_vec(),
        ),
        (
            defillama::chart_url("tether", today),
            llama_chart.as_bytes().to_vec(),
        ),
        (
            defillama::MCAPS_URL.into(),
            br#"{"coingecko:bitcoin":{"mcap":1.6e12},"coingecko:tether":{"mcap":1.8e11}}"#.to_vec(),
        ),
        (
            onchain::HACKS_URL.into(),
            br#"[{"date":1546300800,"name":"Ethereum Classic","targetType":"Chain"}]"#.to_vec(),
        ),
        (
            onchain::CM_CATALOG_URL.into(),
            br#"{"data":[{"asset":"btc","metrics":[{"metric":"AdrActCnt","frequencies":[{"frequency":"1d","community":true}]}]}]}"#.to_vec(),
        ),
        (
            onchain::cm_addresses_url(&["btc".into()], today),
            format!(
                r#"{{"data":[{{"asset":"btc","time":"{}T00:00:00.000000000Z","AdrActCnt":"650000"}}]}}"#,
                crate::time::fmt_date(today - chrono::Duration::days(1))
            )
            .into_bytes(),
        ),
        (
            onchain::MEMPOOL_POOLS_URL.into(),
            br#"{"pools":[{"blockCount":600},{"blockCount":408}]}"#.to_vec(),
        ),
    ]);
    // CoinGecko refuses Tether: the rest goes on.
    f.errors
        .insert(coingecko::coin_url("tether"), "HTTP 429".into());
    let args = SyncArgs {
        force: None,
        sources: Some(vec![SyncSource::Crypto]),
    };
    let rep = run_with(&e.api, &args, &f, today).unwrap();
    let errors: Vec<&str> = rep
        .items
        .iter()
        .filter(|i| i.error.is_some())
        .map(|i| i.label.as_str())
        .collect();
    assert_eq!(errors, vec!["CoinGecko: tether"], "{:?}", rep.items);
    let (reg, facts) = e
        .api
        .with_session(|s, _| {
            Ok((
                load(s.db.conn())?,
                store::load_facts(s.db.conn(), &issuer("bitcoin"))?,
            ))
        })
        .unwrap();
    assert_eq!(reg.ids.get(&btc).map(String::as_str), Some("bitcoin"));
    assert!(reg.stables.contains_key("tether"));
    assert!(reg.chains.contains_key("bitcoin"));
    assert_eq!(
        reg.coins["bitcoin"].genesis_date.as_deref(),
        Some("2009-01-03")
    );
    let items: std::collections::BTreeSet<&str> = facts.iter().map(|f| f.item.as_str()).collect();
    for k in [
        keys::MCAP,
        keys::SUPPLY_GROWTH,
        keys::TVL,
        keys::MCAP_LLAMA,
        keys::PRICE_DAILY,
        keys::INCIDENTS,
        keys::ACTIVE_ADDRESSES,
        keys::NAKAMOTO,
    ] {
        assert!(items.contains(k), "{k}: {items:?}");
    }
    let val = |k: &str| {
        facts
            .iter()
            .find(|f| f.item == k)
            .map(|f| f.value.to_string())
            .unwrap_or_default()
    };
    assert_eq!(val(keys::INCIDENTS), "0");
    assert_eq!(val(keys::ACTIVE_ADDRESSES), "650000");
    assert_eq!(val(keys::NAKAMOTO), "1");
    // Tether (stablecoin) is left out of the on-chain data.
    let tether = e
        .api
        .with_session(|s, _| store::load_facts(s.db.conn(), &issuer("tether")))
        .unwrap();
    assert!(!tether.iter().any(|f| f.item == keys::INCIDENTS));
    // After the failure on tether, no other request to CoinGecko in this collection (bitcoin
    // comes first in the id order).
    let calls = f.calls.lock().unwrap().clone();
    let cg_after = calls
        .iter()
        .skip_while(|u| **u != coingecko::coin_url("tether"))
        .filter(|u| u.contains("coingecko.com"))
        .count();
    assert_eq!(cg_after, 1, "{calls:?}");
}

#[test]
fn coingecko_data_older_than_a_day_is_stale() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-11-18T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    assert!(!super::coingecko_stale("2026-11-17T12:00:01Z", now));
    assert!(super::coingecko_stale("2026-11-17T12:00:00Z", now));
    assert!(super::coingecko_stale("2026-11-10T08:00:00Z", now));
    // An unreadable date does not become a warning.
    assert!(!super::coingecko_stale("ontem", now));
}
