use chrono::{DateTime, Utc};
use rusqlite::params;
use rust_decimal_macros::dec;

use super::*;
use crate::changes::{Author, record};
use crate::market::yahoo::{IntradayPoint, YahooQuote};
use crate::storage::Db;
use crate::vault::KDF_TEST;

fn utc(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/yahoo/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// Vault with PETR4 (B3, with a purchase), AAPL (NASDAQ, on the watchlist), VOD (LSE, shares of
/// other exchanges) and an asset with no trades.
fn db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
        tx.execute(
            "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at) VALUES ('xp', 'XP', 'brokerage', 'BRL', '2024-01-01', 'x', 'x')",
            [],
        )?;
        for (id, t, class, market, cur) in [
            ("petr", "PETR4", "stock_br", "B3", "BRL"),
            ("aapl", "AAPL", "stock_us", "NASDAQ", "USD"),
            ("vod", "VOD", "stock_intl", "LSE", "GBP"),
            ("idle", "WEGE3", "stock_br", "B3", "BRL"),
        ] {
            tx.execute(
                "INSERT INTO asset (id, ticker, name, class, market, currency, created_at, updated_at) VALUES (?1, ?2, ?2, ?3, ?4, ?5, 'x', 'x')",
                params![id, t, class, market, cur],
            )?;
        }
        tx.execute(
            "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
             VALUES ('t1', 'petr', 'xp', '2026-01-10', 'buy', '100', '30', 'BRL', 'x', 'x')",
            [],
        )?;
        tx.execute(
            "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, fx_rate, created_at, updated_at)
             VALUES ('t2', 'vod', 'xp', '2026-01-10', 'buy', '1000', '1', 'GBP', '7', 'x', 'x')",
            [],
        )?;
        tx.execute("INSERT INTO watchlist_item (id, asset_id, created_at, updated_at) VALUES ('w', 'aapl', 'x', 'x')", [])?;
        Ok(())
    })
    .unwrap();
    (dir, db)
}

fn fake(
    symbol: &str,
    price: Decimal,
    at: DateTime<Utc>,
    tz: &str,
    state: YahooMarketState,
) -> YahooIntraday {
    YahooIntraday {
        quote: YahooQuote {
            symbol: symbol.into(),
            currency: None,
            price,
            previous_close: Some(dec!(50)),
            day_high: None,
            day_low: None,
            volume: None,
            market_time: at.timestamp(),
            timezone: Some(tz.into()),
            exchange: None,
            exchange_name: None,
            instrument_type: None,
            name: None,
            state,
            regular_session: None,
        },
        points: vec![
            IntradayPoint {
                ts: at.timestamp() - 600,
                price: price - dec!(1),
                volume: None,
            },
            IntradayPoint {
                ts: at.timestamp() - 300,
                price,
                volume: Some(dec!(100)),
            },
        ],
    }
}

#[test]
fn parses_intraday_fixtures_with_quote_summary() {
    // PETR4 (5 min, 1 day), collected after the close.
    let now = utc("2026-10-15T03:11:00Z").timestamp();
    let p = yahoo::parse_intraday(&fixture("chart_petr4_5m_1d.json"), now).unwrap();
    assert_eq!(p.quote.symbol, "PETR4.SA");
    assert_eq!(p.quote.currency.as_deref(), Some("BRL"));
    assert_eq!(p.quote.price, dec!(58.11));
    assert_eq!(p.quote.previous_close, Some(dec!(56.85)));
    assert_eq!(p.quote.timezone.as_deref(), Some("America/Sao_Paulo"));
    assert_eq!(p.quote.exchange.as_deref(), Some("SAO"));
    assert_eq!(p.quote.state, YahooMarketState::Closed);
    assert_eq!(p.quote.day_high, None, "a Yahoo zero becomes missing");
    assert!(p.points.len() > 60, "{}", p.points.len());
    assert!(p.points.windows(2).all(|w| w[1].ts > w[0].ts));
    let ch = p.quote.change_ratio().unwrap();
    assert!((ch - dec!(0.02216)).abs() < dec!(0.0001), "{ch}");
    // During the regular session (13:00–20:00 UTC of the next day, per currentTradingPeriod).
    let open = yahoo::parse_intraday(
        &fixture("chart_petr4_5m_1d.json"),
        utc("2026-10-15T14:00:00Z").timestamp(),
    )
    .unwrap();
    assert_eq!(open.quote.state, YahooMarketState::Regular);
    let pre = yahoo::parse_intraday(
        &fixture("chart_petr4_5m_1d.json"),
        utc("2026-10-15T12:50:00Z").timestamp(),
    )
    .unwrap();
    assert_eq!(pre.quote.state, YahooMarketState::Pre);

    // VOD.L (15 min, 5 days): Yahoo quotes in pence; Treasury stores in pounds.
    let v = yahoo::parse_intraday(&fixture("chart_vod_l_15m_5d.json"), now).unwrap();
    assert_eq!(v.quote.currency.as_deref(), Some("GBP"));
    assert_eq!(v.quote.price, dec!(1.1775));
    assert_eq!(
        v.quote.previous_close,
        Some(dec!(1.177)),
        "previousClose, not the 5-day chartPreviousClose"
    );
    assert!(
        v.points
            .iter()
            .all(|p| p.price > dec!(1) && p.price < dec!(2))
    );
    assert_eq!(
        v.quote.name.as_deref(),
        Some("Vodafone Group Public Limited Company")
    );
}

#[test]
fn daily_series_in_pence_are_converted() {
    let body = r#"{"chart":{"result":[{"meta":{"currency":"GBp","gmtoffset":3600,"priceHint":2},
      "timestamp":[1791961200],
      "indicators":{"quote":[{"open":[117.0],"high":[118.0],"low":[116.5],"close":[117.75],"volume":[5200]}]},
      "events":{"dividends":{"1791961200":{"amount":2.25,"date":1791961200}}}}],"error":null}}"#;
    let s = yahoo::parse(body).unwrap();
    assert_eq!(s.currency.as_deref(), Some("GBP"));
    assert_eq!(s.bars[0].close, dec!(1.1775));
    assert_eq!(
        s.bars[0].volume,
        Some(dec!(5200)),
        "volume is not converted"
    );
    assert_eq!(s.dividends[0].amount, dec!(0.0225));
}

#[test]
fn parses_search_fixtures_and_drops_non_spot_b3_codes() {
    let t = yahoo::parse_search(&fixture("search_toyota.json")).unwrap();
    let syms: Vec<&str> = t.iter().map(|h| h.symbol.as_str()).collect();
    assert!(
        syms.contains(&"7203.T") && syms.contains(&"TM") && syms.contains(&"TMH"),
        "{syms:?}"
    );
    let tmh = t.iter().find(|h| h.symbol == "TMH").unwrap();
    assert_eq!(
        (tmh.quote_type.as_str(), tmh.exchange.as_deref()),
        ("ETF", Some("PCX"))
    );
    let p = yahoo::parse_search(&fixture("search_petr4.json")).unwrap();
    assert_eq!(
        p.iter().map(|h| h.symbol.as_str()).collect::<Vec<_>>(),
        vec!["PETR4.SA"],
        "fractional lots (PETR4F) and the like are left out"
    );
    assert_eq!(p[0].name, "Petróleo Brasileiro S.A. - Petrobras");
    assert!(yahoo::parse_search("<html>").is_err());
    assert!(
        yahoo::parse_search(r#"{"quotes":[{"symbol":"^BVSP","quoteType":"INDEX"}]}"#)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn plan_follows_trading_hours_and_tracked_assets() {
    let (_d, db) = db();
    let c = db.conn();
    // Wednesday, 13:15 UTC: B3 (10:15) and London (14:15) open, Nasdaq still closed (9:15 in NY).
    let now = utc("2026-10-14T13:15:00Z");
    let p = plan(c, now, None).unwrap();
    let ids: Vec<&str> = p.iter().map(|t| t.asset.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["petr", "vod"],
        "only tracked assets with an open session"
    );
    assert_eq!(p[1].symbol, "VOD.L");
    assert_eq!(p[0].range, "5d", "no points yet: fetches 5 days");
    // 15:00 UTC: Nasdaq opened (11:00 in NY).
    let ids: Vec<String> = plan(c, utc("2026-10-14T15:00:00Z"), None)
        .unwrap()
        .into_iter()
        .map(|t| t.asset.id)
        .collect();
    assert!(ids.contains(&"aapl".to_string()));
    // Saturday: nothing. Screen request: ignores the hours.
    assert!(
        plan(c, utc("2026-10-17T15:00:00Z"), None)
            .unwrap()
            .is_empty()
    );
    let forced = plan(c, utc("2026-10-17T15:00:00Z"), Some(&["idle".to_string()])).unwrap();
    assert_eq!(forced.len(), 1);
    assert_eq!(forced[0].symbol, "WEGE3.SA");
}

#[test]
fn apply_stores_quote_points_and_prunes() {
    let (_d, db) = db();
    let c = db.conn();
    let now = utc("2026-10-14T14:00:00Z");
    // Old point (more than 7 days) is deleted on collection.
    c.execute(
        "INSERT INTO intraday_price (asset_id, ts, price) VALUES ('petr', ?1, '40')",
        [now.timestamp() - 8 * 86_400],
    )
    .unwrap();
    let tasks = plan(c, now, None).unwrap();
    let petr = tasks.iter().find(|t| t.asset.id == "petr").unwrap().clone();
    let vod = tasks.iter().find(|t| t.asset.id == "vod").unwrap().clone();
    let mut wrong = fake(
        "VOD.L",
        dec!(1.2),
        now,
        "Europe/London",
        YahooMarketState::Regular,
    );
    wrong.quote.currency = Some("USD".into());
    let rep = apply(
        c,
        &[
            (
                petr.clone(),
                Ok(fake(
                    "PETR4.SA",
                    dec!(51),
                    now,
                    "America/Sao_Paulo",
                    YahooMarketState::Regular,
                )),
            ),
            (vod.clone(), Ok(wrong)),
        ],
        now,
    )
    .unwrap();
    assert_eq!(rep.refreshed, vec!["petr"]);
    assert!(rep.errors[0].error.contains("moeda"), "{:?}", rep.errors);
    let q = quote(c, "petr").unwrap().unwrap();
    assert_eq!(
        (q.price.as_str(), q.market_date.as_str()),
        ("51", "2026-10-14")
    );
    assert_eq!(q.market_state, SessionState::Open);
    assert_eq!(q.change_pct, Some(0.02));
    assert_eq!(q.delay_minutes, 15);
    let n: i64 = c
        .query_row(
            "SELECT count(*) FROM intraday_price WHERE asset_id = 'petr'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 2, "the old point was deleted");
    let s = series(c, "petr", "1d", now).unwrap();
    assert_eq!(s.points.len(), 2);
    assert_eq!(s.exchange.as_deref(), Some("B3"));
    assert_eq!(s.session_now, Some(SessionState::Open));
    assert!(series(c, "petr", "1y", now).is_err());
    // Collected a moment ago: the routine does not repeat.
    assert!(
        plan(c, now + chrono::Duration::minutes(2), None)
            .unwrap()
            .iter()
            .all(|t| t.asset.id != "petr")
    );
    // With recent points, the next collection asks for only 1 day.
    let next = plan(c, now + chrono::Duration::minutes(6), None).unwrap();
    assert_eq!(
        next.iter().find(|t| t.asset.id == "petr").unwrap().range,
        "1d"
    );
    // Technical tables: nothing in the change history.
    assert!(crate::changes::UNTRACKED_TABLES.contains(&"market_quote"));
    assert!(crate::changes::UNTRACKED_TABLES.contains(&"intraday_price"));
}

#[test]
fn five_day_series_keeps_last_sessions_in_exchange_time() {
    let (_d, db) = db();
    let c = db.conn();
    // Bars over 6 sessions: 1d keeps the last day (in São Paulo time), 5d keeps five.
    for day in 8..=14 {
        for (h, p) in [("13:00", "10"), ("19:55", "11")] {
            let ts = utc(&format!("2026-10-{day:02}T{h}:00Z")).timestamp();
            c.execute(
                "INSERT INTO intraday_price (asset_id, ts, price) VALUES ('petr', ?1, ?2)",
                params![ts, p],
            )
            .unwrap();
        }
    }
    let now = utc("2026-10-14T21:00:00Z");
    assert_eq!(series(c, "petr", "1d", now).unwrap().points.len(), 2);
    let five = series(c, "petr", "5d", now).unwrap();
    assert_eq!(five.points.len(), 10);
    assert_eq!(five.timezone.as_deref(), Some("America/Sao_Paulo"));
}

#[test]
fn portfolio_uses_intraday_quote_only_for_today() {
    let (_d, db) = db();
    let c = db.conn();
    let today = crate::time::today_local();
    let now = Utc::now();
    let yesterday = today - chrono::Duration::days(1);
    crate::market::upsert_prices(
        c,
        "petr",
        &[crate::market::PriceRow {
            date: yesterday,
            open: None,
            high: None,
            low: None,
            close: dec!(50),
            adj_close: None,
            volume: None,
        }],
        "yahoo",
    )
    .unwrap();
    let task = QuoteTask {
        asset: crate::assets::get(c, "petr").unwrap(),
        symbol: "PETR4.SA".into(),
        exchange: exchange::get(c, "B3").unwrap(),
        range: "1d",
    };
    let mut data = fake(
        "PETR4.SA",
        dec!(52.5),
        now,
        "America/Sao_Paulo",
        YahooMarketState::Regular,
    );
    // Forces the session date to be today in the app's local time zone.
    data.quote.timezone = Some("UTC".into());
    data.quote.market_time = today.and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp();
    apply(c, &[(task.clone(), Ok(data.clone()))], now).unwrap();

    let book = crate::portfolio::engine::load_book(c, "petr", None).unwrap();
    let st = crate::portfolio::engine::replay(c, &book, today).unwrap();
    let v = crate::portfolio::engine::value_state(c, &book, &st, today).unwrap();
    assert_eq!(v.value, Some(dec!(5250)));
    let p = v.price.unwrap();
    assert_eq!(p.source, "yahoo_intraday");
    let mark = p.intraday.unwrap();
    assert_eq!(mark.delay_minutes, 15);
    assert_eq!(mark.change_pct, Some(0.05));
    assert!(v.estimated, "session in progress: estimated value");
    // Past dates use only the daily close.
    let v = crate::portfolio::engine::value_state(c, &book, &st, yesterday).unwrap();
    assert_eq!(v.value, Some(dec!(5000)));
    assert!(v.price.unwrap().intraday.is_none());
    // Watchlist too (AAPL without a quote; PETR4 is not on the watchlist).
    let w = crate::portfolio::records::list_watch(c).unwrap();
    assert!(w[0].price.is_none());

    // An old intraday quote does not beat a newer close.
    let mut old = data;
    old.quote.market_time = (today - chrono::Duration::days(3))
        .and_hms_opt(12, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    old.quote.state = YahooMarketState::Closed;
    apply(c, &[(task, Ok(old))], now).unwrap();
    let v = crate::portfolio::engine::value_state(c, &book, &st, today).unwrap();
    assert_eq!(v.value, Some(dec!(5000)));
    assert_eq!(v.price.unwrap().source, "yahoo");
}

#[test]
fn backoff_grows_per_symbol_and_globally() {
    let now = utc("2026-10-14T14:00:00Z");
    let (_d, db) = db();
    let tasks = plan(db.conn(), now, None).unwrap();
    let mut b = Backoff::default();
    let fail_all = QuoteRefreshReport {
        refreshed: vec![],
        errors: tasks
            .iter()
            .map(|t| QuoteError {
                asset_id: t.asset.id.clone(),
                ticker: None,
                symbol: t.symbol.clone(),
                error: "rede".into(),
            })
            .collect(),
        finished_at: String::new(),
    };
    b.record(&fail_all, &tasks, now);
    assert!(!b.global_allows(now + chrono::Duration::minutes(4)));
    assert!(b.global_allows(now + chrono::Duration::minutes(5)));
    b.record(&fail_all, &tasks, now);
    b.record(&fail_all, &tasks, now);
    assert!(
        !b.global_allows(now + chrono::Duration::minutes(19)),
        "3rd failure: 20 min"
    );
    for _ in 0..10 {
        b.record(&fail_all, &tasks, now);
    }
    assert!(
        b.global_allows(now + chrono::Duration::minutes(MAX_BACKOFF_MINUTES)),
        "60 min cap"
    );
    // One symbol failing while the others are fine: only it backs off, and the global one resets.
    let one = QuoteRefreshReport {
        refreshed: vec!["petr".into()],
        errors: vec![QuoteError {
            asset_id: "vod".into(),
            ticker: None,
            symbol: "VOD.L".into(),
            error: "404".into(),
        }],
        finished_at: String::new(),
    };
    b.record(&one, &tasks, now);
    assert!(b.global_allows(now));
    assert!(b.allows("PETR4.SA", now));
    assert!(!b.allows("VOD.L", now + chrono::Duration::minutes(1)));
}

#[test]
#[ignore = "usa a rede: cargo test -- --ignored live_yahoo_intraday"]
fn live_yahoo_intraday() {
    for (sym, cur) in [
        ("PETR4.SA", "BRL"),
        ("AAPL", "USD"),
        ("VOD.L", "GBP"),
        ("7203.T", "JPY"),
    ] {
        let d = yahoo::fetch_intraday(sym, "5m", "5d").unwrap();
        assert_eq!(d.quote.currency.as_deref(), Some(cur), "{sym}");
        assert!(d.quote.price > Decimal::ZERO, "{sym}");
        assert!(d.points.len() > 20, "{sym}: {}", d.points.len());
        assert!(d.quote.previous_close.is_some(), "{sym}");
        println!(
            "{sym}: {} {cur} ({:?}) às {} — {} pontos, {:?}",
            d.quote.price,
            d.quote.change_ratio(),
            d.quote.market_time,
            d.points.len(),
            d.quote.state
        );
    }
    let d = yahoo::fetch_intraday("BTC-USD", "15m", "1d").unwrap();
    assert!(d.quote.price > Decimal::from(1000));
    let s = yahoo::search("vodafone", 10).unwrap();
    assert!(s.iter().any(|h| h.symbol == "VOD.L"), "{s:?}");
    let s = yahoo::search("PETR4", 10).unwrap();
    assert_eq!(s[0].symbol, "PETR4.SA");
}
