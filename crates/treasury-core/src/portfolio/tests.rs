use rusqlite::params;
use rust_decimal_macros::dec;

use super::*;
use crate::assets::{
    self, AssetClass, AssetInput, FixedIncomeKind, FixedIncomeTerms, Indexer, Liquidity,
};
use crate::changes::{Author, record};
use crate::indices::Deflator;
use crate::market::{PriceRow, upsert_prices};
use crate::storage::Db;
use crate::time::parse_date;
use crate::vault::KDF_TEST;

fn db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
        crate::household::seed(tx, "Família", "BRL", &["Ana".into()])?;
        crate::categories::seed_defaults(tx)?;
        for (id, name, cur) in [("xp", "XP Investimentos", "BRL"), ("avenue", "Avenue", "USD")] {
            tx.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
                 VALUES (?1, ?2, 'brokerage', ?3, '2024-01-01', 'x', 'x')",
                params![id, name, cur],
            )?;
        }
        Ok(())
    })
    .unwrap();
    (dir, db)
}

fn asset(db: &mut Db, ticker: &str, class: AssetClass, fi: Option<FixedIncomeTerms>) -> String {
    let input = AssetInput {
        ticker: Some(ticker.into()),
        name: format!("{ticker} S.A."),
        official_name: None,
        account_id: None,
        class,
        market: None,
        currency: None,
        country: None,
        isin: None,
        cnpj: None,
        treasury_code: None,
        sector: None,
        segment: None,
        price_provider: None,
        provider_symbol: None,
        notes: None,
        fixed_income: fi,
        pension: None,
    };
    record(db.conn_mut(), &Author::user(), "ativo", |tx| {
        assets::create(tx, &input)
    })
    .unwrap()
    .value
}

#[allow(clippy::too_many_arguments)]
fn trade(
    db: &mut Db,
    asset_id: &str,
    account: &str,
    date: &str,
    side: TradeSide,
    qty: &str,
    price: &str,
    fees: i64,
) -> crate::error::Result<String> {
    let t = TradeInput {
        asset_id: asset_id.into(),
        account_id: account.into(),
        date: date.into(),
        side,
        quantity: qty.into(),
        price: price.into(),
        fees: Some(fees),
        taxes: None,
        fx_rate: None,
        day_trade: None,
        note_number: None,
        notes: None,
        cash_movement: None,
        external_id: None,
    };
    record(db.conn_mut(), &Author::user(), "op", |tx| {
        records::create_trade(tx, &t)
    })
    .map(|r| r.value)
}

fn price(db: &Db, asset_id: &str, date: &str, close: rust_decimal::Decimal) {
    upsert_prices(
        db.conn(),
        asset_id,
        &[PriceRow {
            date: parse_date(date).unwrap(),
            open: None,
            high: None,
            low: None,
            close,
            adj_close: None,
            volume: None,
        }],
        "test",
    )
    .unwrap();
}

fn args(as_of: &str) -> PortfolioArgs {
    PortfolioArgs {
        as_of: Some(as_of.into()),
        account_ids: None,
        deflator: Some(Deflator::None),
        by_account: None,
        include_closed: None,
    }
}

#[test]
fn stock_position_average_cost_split_sale_income_and_cash() {
    let (_d, mut db) = db();
    let petr = asset(&mut db, "PETR4", AssetClass::StockBr, None);
    trade(
        &mut db,
        &petr,
        "xp",
        "2025-01-10",
        TradeSide::Buy,
        "100",
        "30",
        1000,
    )
    .unwrap();
    trade(
        &mut db,
        &petr,
        "xp",
        "2025-06-10",
        TradeSide::Buy,
        "100",
        "34",
        1000,
    )
    .unwrap();
    record(db.conn_mut(), &Author::user(), "split", |tx| {
        records::create_action(
            tx,
            &CorporateActionInput {
                asset_id: petr.clone(),
                date: "2025-08-01".into(),
                kind: CorporateActionKind::Split,
                factor: Some("2".into()),
                unit_cost: None,
                amount: None,
                new_asset_id: None,
                notes: None,
            },
        )
    })
    .unwrap();
    // Selling more than you have fails.
    assert!(
        trade(
            &mut db,
            &petr,
            "xp",
            "2025-09-01",
            TradeSide::Sell,
            "500",
            "20",
            0
        )
        .is_err()
    );
    trade(
        &mut db,
        &petr,
        "xp",
        "2025-09-01",
        TradeSide::Sell,
        "100",
        "20",
        500,
    )
    .unwrap();
    record(db.conn_mut(), &Author::user(), "div", |tx| {
        records::create_income(
            tx,
            &IncomeInput {
                asset_id: petr.clone(),
                account_id: "xp".into(),
                kind: IncomeKind::Dividend,
                ex_date: None,
                pay_date: "2025-12-15".into(),
                quantity: None,
                gross: 10000,
                tax_withheld: None,
                fx_rate: None,
                status: None,
                cash_movement: None,
            },
        )
    })
    .unwrap();
    price(&db, &petr, "2026-10-13", dec!(18));

    let s = engine::summary(db.conn(), &args("2026-10-14")).unwrap();
    let p = &s.positions[0];
    assert_eq!(p.quantity, "300");
    assert_eq!(p.avg_price.as_deref(), Some("16.05"));
    assert_eq!(p.cost_base, 481500);
    assert_eq!(p.market_value_base, Some(540000));
    assert_eq!(p.unrealized_base, Some(58500));
    assert_eq!(p.realized_base, 39000); // 2000 − 5 − 1605
    assert_eq!(p.income_base, 10000);
    assert_eq!(p.total_result_base, Some(58500 + 39000 + 10000));
    assert_eq!(p.valuation, Valuation::Market);
    assert!(p.xirr.unwrap() > 0.0);
    assert_eq!(p.weight, Some(1.0));
    assert_eq!(s.market_value, 540000);

    // Brokerage cash: −3.010 −3.410 +1.995 +100 = −4.325
    let bal = crate::accounts::balance(db.conn(), "xp", "2026-10-14", false).unwrap();
    assert_eq!(bal, -432500);
    // The income came in as revenue under the category Rendimentos › Dividendos.
    let cat: Option<String> = db
        .conn()
        .query_row("SELECT c.name FROM txn t JOIN category c ON c.id = t.category_id WHERE t.kind = 'income'", [], |r| r.get(0))
        .ok();
    assert_eq!(cat.as_deref(), Some("Dividendos"));
}

#[test]
fn foreign_stock_uses_fx_for_cost_and_value() {
    let (_d, mut db) = db();
    let c = db.conn();
    for (d, r) in [("2025-01-01", "5.00"), ("2026-10-01", "5.50")] {
        crate::fx::set_manual(
            c,
            &crate::fx::FxRateInput {
                base: "USD".into(),
                quote: "BRL".into(),
                date: d.into(),
                rate: r.into(),
            },
        )
        .unwrap();
    }
    let aapl = asset(&mut db, "AAPL", AssetClass::StockUs, None);
    trade(
        &mut db,
        &aapl,
        "avenue",
        "2025-01-10",
        TradeSide::Buy,
        "10",
        "200",
        0,
    )
    .unwrap();
    price(&db, &aapl, "2026-10-13", dec!(250));
    let s = engine::summary(db.conn(), &args("2026-10-14")).unwrap();
    let p = &s.positions[0];
    assert_eq!(p.cost, 200000, "USD 2.000,00");
    assert_eq!(
        p.cost_base, 1000000,
        "R$ 10.000,00 at the purchase exchange rate"
    );
    assert_eq!(p.market_value, Some(250000));
    assert_eq!(p.market_value_base, Some(1375000));
    assert_eq!(p.unrealized_base, Some(375000));
}

/// RDB (Nubank, Inter): fixed income like the CDB, stored with its own type, on the curve, with
/// IR (not exempt); the FGC is covered in the fixed-income score test.
#[test]
fn rdb_is_a_bank_deposit_like_a_cdb() {
    let (_d, mut db) = db();
    let terms = FixedIncomeTerms {
        kind: FixedIncomeKind::Rdb,
        indexer: Indexer::Cdi,
        rate: "100".into(),
        issue_date: Some("2026-10-09".into()),
        maturity_date: Some("2028-10-09".into()),
        grace_until: None,
        liquidity: Liquidity::Daily,
        ir_exempt: FixedIncomeKind::Rdb.default_ir_exempt(),
        issuer: Some("Banco Y".into()),
    };
    let rdb = asset(&mut db, "RDB-Y-100", AssetClass::FixedIncome, Some(terms));
    let a = assets::get(db.conn(), &rdb).unwrap();
    let t = a.fixed_income.unwrap();
    assert_eq!((t.kind, t.ir_exempt), (FixedIncomeKind::Rdb, false));
    trade(
        &mut db,
        &rdb,
        "xp",
        "2026-10-09",
        TradeSide::Buy,
        "1000",
        "1",
        0,
    )
    .unwrap();
    let s = engine::summary(db.conn(), &args("2026-10-14")).unwrap();
    assert_eq!(s.positions[0].valuation, Valuation::Curve);
}

#[test]
fn cdb_on_the_curve_with_net_redemption() {
    let (_d, mut db) = db();
    let terms = FixedIncomeTerms {
        kind: FixedIncomeKind::Cdb,
        indexer: Indexer::Cdi,
        rate: "110".into(),
        issue_date: Some("2026-10-09".into()),
        maturity_date: Some("2028-10-09".into()),
        grace_until: None,
        liquidity: Liquidity::Daily,
        ir_exempt: false,
        issuer: Some("Banco X".into()),
    };
    let cdb = asset(&mut db, "CDB-X-110", AssetClass::FixedIncome, Some(terms));
    trade(
        &mut db,
        &cdb,
        "xp",
        "2026-10-09",
        TradeSide::Buy,
        "1",
        "1000",
        0,
    )
    .unwrap();
    crate::indices::upsert_values(
        db.conn(),
        "cdi",
        &[
            (parse_date("2026-10-13").unwrap(), dec!(0.055131), false),
            (parse_date("2026-10-14").unwrap(), dec!(0.055131), false),
        ],
        "test",
    )
    .unwrap();
    let s = engine::summary(db.conn(), &args("2026-10-14")).unwrap();
    let p = &s.positions[0];
    assert_eq!(p.valuation, Valuation::Curve);
    // 1000 × 1,000606441² ≈ 1001,213
    assert_eq!(p.market_value_base, Some(100121));
    let net = p.net_redemption_base.unwrap();
    assert!(
        (100000..100121).contains(&net),
        "IOF and IR reduce the net amount: {net}"
    );
}

#[test]
fn ticker_change_moves_position() {
    let (_d, mut db) = db();
    let old = asset(&mut db, "BRML3", AssetClass::StockBr, None);
    let new = asset(&mut db, "ALOS3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &old,
        "xp",
        "2024-03-01",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    record(db.conn_mut(), &Author::user(), "inc", |tx| {
        records::create_action(
            tx,
            &CorporateActionInput {
                asset_id: old.clone(),
                date: "2024-06-01".into(),
                kind: CorporateActionKind::Merger,
                factor: Some("0.5".into()),
                unit_cost: None,
                amount: None,
                new_asset_id: Some(new.clone()),
                notes: None,
            },
        )
    })
    .unwrap();
    price(&db, &new, "2026-10-13", dec!(25));
    let s = engine::summary(db.conn(), &args("2026-10-14")).unwrap();
    assert_eq!(s.positions.len(), 1);
    let p = &s.positions[0];
    assert_eq!(p.asset.ticker.as_deref(), Some("ALOS3"));
    assert_eq!(p.quantity, "50");
    assert_eq!(p.avg_price.as_deref(), Some("20"));
    assert_eq!(p.market_value_base, Some(125000));
    assert_eq!(p.realized_base, 0);
}

#[test]
fn rebalance_distributes_contribution_to_underweight_classes() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "BOVA11", AssetClass::EtfBr, None);
    let b = asset(&mut db, "IVVB11", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "100",
        "70",
        0,
    )
    .unwrap(); // 7.000
    trade(
        &mut db,
        &b,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "10",
        "300",
        0,
    )
    .unwrap(); // 3.000
    price(&db, &a, "2026-10-13", dec!(70));
    price(&db, &b, "2026-10-13", dec!(300));
    let targets = vec![
        TargetInput {
            scope: TargetScope::Class,
            key: "etf_br".into(),
            target_pct: "50".into(),
            tolerance_pct: None,
        },
        TargetInput {
            scope: TargetScope::Class,
            key: "stock_br".into(),
            target_pct: "50".into(),
            tolerance_pct: None,
        },
    ];
    record(db.conn_mut(), &Author::user(), "alvos", |tx| {
        records::set_targets(tx, TargetScope::Class, &targets)
    })
    .unwrap();
    let r = engine::rebalance(
        db.conn(),
        &RebalanceArgs {
            contribution: Some(200000),
        },
    )
    .unwrap();
    assert!(!r.targets_incomplete);
    let stock = r
        .lines
        .iter()
        .find(|l| l.class == AssetClass::StockBr)
        .unwrap();
    let etf = r
        .lines
        .iter()
        .find(|l| l.class == AssetClass::EtfBr)
        .unwrap();
    assert_eq!(
        stock.suggested, 200000,
        "the whole contribution goes to the underweight class"
    );
    assert_eq!(etf.suggested, 0);
    assert!(
        stock.out_of_band && etf.out_of_band,
        "70/30 is outside the 5 p.p. band"
    );
    // Targets above 100% are refused.
    let bad = vec![TargetInput {
        scope: TargetScope::Class,
        key: "etf_br".into(),
        target_pct: "120".into(),
        tolerance_pct: None,
    }];
    assert!(
        record(db.conn_mut(), &Author::user(), "x", |tx| {
            records::set_targets(tx, TargetScope::Class, &bad)
        })
        .is_err()
    );
}

#[test]
fn monthly_performance_tracks_price_change() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ITSA4", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-02",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &a, "2026-01-30", dec!(10));
    price(&db, &a, "2026-02-27", dec!(11));
    price(&db, &a, "2026-03-31", dec!(11));
    crate::indices::upsert_values(
        db.conn(),
        "ibov",
        &[
            (parse_date("2025-12-30").unwrap(), dec!(100000), false),
            (parse_date("2026-02-27").unwrap(), dec!(110000), false),
            (parse_date("2026-03-31").unwrap(), dec!(99000), false),
        ],
        "test",
    )
    .unwrap();
    let perf = engine::performance(
        db.conn(),
        &PerformanceArgs {
            from_month: Some("2026-01".into()),
            to_month: Some("2026-03".into()),
            deflator: Some(Deflator::None),
            asset_id: None,
            class: None,
            method: None,
        },
    )
    .unwrap();
    assert_eq!(perf.months, vec!["2026-01", "2026-02", "2026-03"]);
    assert!(
        perf.monthly[0].unwrap().abs() < 1e-9,
        "bought at 10 and closed at 10"
    );
    assert!((perf.monthly[1].unwrap() - 0.10).abs() < 1e-9);
    assert!((perf.cumulative[2] - 0.10).abs() < 1e-9);
    assert_eq!(perf.value_base, vec![100000, 110000, 110000]);
    // January without a session on the 10-day base: 30/12 counts for 31/12, but 30/01 has no point.
    assert_eq!(perf.ibov_cumulative[0], None);
    assert!((perf.ibov_cumulative[1].unwrap() - 0.10).abs() < 1e-9);
    assert!((perf.ibov_cumulative[2].unwrap() + 0.01).abs() < 1e-9);
    assert!(perf.sp500_brl_cumulative.iter().all(Option::is_none));
    assert!(perf.usd_cumulative.iter().all(Option::is_none));
    assert_eq!(perf.net_contributions, vec![100000, 100000, 100000]);
}

#[test]
fn real_xirr_is_lower_with_inflation() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "TAEE11", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-01",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &a, "2026-07-01", dec!(11));
    let vals: Vec<_> = (1..=7)
        .map(|m| {
            (
                parse_date(&format!("2026-{m:02}-01")).unwrap(),
                dec!(0.5),
                false,
            )
        })
        .collect();
    crate::indices::upsert_values(db.conn(), "ipca", &vals, "test").unwrap();
    let s = engine::summary(
        db.conn(),
        &PortfolioArgs {
            as_of: Some("2026-07-01".into()),
            account_ids: None,
            deflator: Some(Deflator::Ipca),
            by_account: None,
            include_closed: None,
        },
    )
    .unwrap();
    let (n, r) = (s.xirr.unwrap(), s.xirr_real.unwrap());
    assert!(n > r && r > 0.0, "nominal {n} > real {r} > 0");
    // (1+n)/(1+r) ≈ annualized inflation of 0,5% per month ≈ 6,17% per year.
    assert!(((1.0 + n) / (1.0 + r) - 1.0 - 0.0617).abs() < 0.003);
    // Real result: the R$ 1.000 of the purchase, adjusted to July (≈ 5 to 6 months of 0,5%)
    // cost some R$ 25 to 31 more; the nominal gain of R$ 100 becomes about R$ 70 in today's money.
    assert_eq!(s.total_result, 10_000);
    let effect = s.inflation_effect.unwrap();
    assert!((2_400..=3_100).contains(&effect), "effect {effect}");
    assert_eq!(s.real_result, Some(10_000 - effect));
    let p = &s.positions[0];
    assert_eq!(p.inflation_effect_base, Some(effect));
    assert_eq!(p.real_result_base, Some(10_000 - effect));
    // Nominal: real equals nominal, zero effect.
    let nominal = engine::summary(
        db.conn(),
        &PortfolioArgs {
            as_of: Some("2026-07-01".into()),
            account_ids: None,
            deflator: Some(Deflator::None),
            by_account: None,
            include_closed: None,
        },
    )
    .unwrap();
    assert_eq!(nominal.inflation_effect, Some(0));
    assert_eq!(nominal.real_result, Some(nominal.total_result));
}

#[test]
fn real_result_counts_income_and_needs_the_index() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "BBAS3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-01",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &a, "2026-07-01", dec!(10));
    let args = |d| PortfolioArgs {
        as_of: Some("2026-07-01".into()),
        account_ids: None,
        deflator: Some(d),
        by_account: None,
        include_closed: None,
    };
    // Without the IPCA series: real has no value, nominal stays.
    let s = engine::summary(db.conn(), &args(Deflator::Ipca)).unwrap();
    assert_eq!(s.real_result, None);
    assert_eq!(s.positions[0].real_result_base, None);
    let vals: Vec<_> = (1..=7)
        .map(|m| {
            (
                parse_date(&format!("2026-{m:02}-01")).unwrap(),
                dec!(0.5),
                false,
            )
        })
        .collect();
    crate::indices::upsert_values(db.conn(), "ipca", &vals, "test").unwrap();
    // Flat price: nominal is zero and real is only the inflation loss on the purchase.
    let s = engine::summary(db.conn(), &args(Deflator::Ipca)).unwrap();
    assert_eq!(s.total_result, 0);
    let real = s.real_result.unwrap();
    assert!(real < 0 && real == -s.inflation_effect.unwrap());
}

#[test]
fn undo_trade_restores_cash_and_position() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "WEGE3", AssetClass::StockBr, None);
    let t = TradeInput {
        asset_id: a.clone(),
        account_id: "xp".into(),
        date: "2026-03-02".into(),
        side: TradeSide::Buy,
        quantity: "10".into(),
        price: "40".into(),
        fees: None,
        taxes: None,
        fx_rate: None,
        day_trade: None,
        note_number: None,
        notes: None,
        cash_movement: None,
        external_id: None,
    };
    let r = record(db.conn_mut(), &Author::user(), "op", |tx| {
        records::create_trade(tx, &t)
    })
    .unwrap();
    assert_eq!(
        crate::accounts::balance(db.conn(), "xp", "2026-10-14", false).unwrap(),
        -40000
    );
    crate::changes::undo(db.conn_mut(), r.change_set_id.as_ref().unwrap()).unwrap();
    assert_eq!(
        crate::accounts::balance(db.conn(), "xp", "2026-10-14", false).unwrap(),
        0
    );
    assert!(
        engine::summary(db.conn(), &args("2026-10-14"))
            .unwrap()
            .positions
            .is_empty()
    );
}

#[test]
fn net_worth_includes_portfolio_market_value() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "BBAS3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-02-02",
        TradeSide::Buy,
        "100",
        "25",
        0,
    )
    .unwrap();
    price(&db, &a, "2026-10-13", dec!(30));
    let nw = crate::reports::net_worth(db.conn(), "2026-10-14").unwrap();
    assert_eq!(nw.investments, 300000);
    // Brokerage cash ended at −2.500 (purchase without balance), assets = 3.000 in shares.
    assert_eq!(nw.assets, 300000);
    assert_eq!(nw.liabilities, 250000);
    assert_eq!(nw.net, 50000);
    let hist = crate::reports::net_worth_history(db.conn(), 3, "2026-10").unwrap();
    assert_eq!(hist.last().unwrap().investments, 300000);
}

#[test]
fn ticker_change_does_not_double_count_sales_before_the_change() {
    let (_d, mut db) = db();
    let old = asset(&mut db, "VVAR3", AssetClass::StockBr, None);
    let new = asset(&mut db, "BHIA3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &old,
        "xp",
        "2024-01-10",
        TradeSide::Buy,
        "200",
        "10",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &old,
        "xp",
        "2024-02-10",
        TradeSide::Sell,
        "100",
        "12",
        0,
    )
    .unwrap(); // +200
    record(db.conn_mut(), &Author::user(), "ticker", |tx| {
        records::create_action(
            tx,
            &CorporateActionInput {
                asset_id: old.clone(),
                date: "2024-03-01".into(),
                kind: CorporateActionKind::TickerChange,
                factor: None,
                unit_cost: None,
                amount: None,
                new_asset_id: Some(new.clone()),
                notes: None,
            },
        )
    })
    .unwrap();
    trade(
        &mut db,
        &new,
        "xp",
        "2024-04-10",
        TradeSide::Sell,
        "50",
        "14",
        0,
    )
    .unwrap(); // +200
    price(&db, &new, "2026-10-13", dec!(15));
    let s = engine::summary(
        db.conn(),
        &PortfolioArgs {
            as_of: Some("2026-10-14".into()),
            account_ids: None,
            deflator: Some(Deflator::None),
            by_account: None,
            include_closed: Some(true),
        },
    )
    .unwrap();
    assert_eq!(
        s.realized, 40000,
        "200 before the change + 200 after, without double counting"
    );
    let p = s.positions.iter().find(|p| p.asset.id == new).unwrap();
    assert_eq!(p.quantity, "50");
    assert_eq!(p.avg_price.as_deref(), Some("10"));
    assert_eq!(s.market_value, 75000);
    assert!(
        p.xirr.is_some(),
        "the new asset's XIRR includes the old asset's purchases"
    );
}

#[test]
fn pending_income_suggests_unrecorded_dividends_once() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "AAPL", AssetClass::StockUs, None);
    trade(
        &mut db,
        &a,
        "avenue",
        "2026-03-02",
        TradeSide::Buy,
        "10",
        "200",
        0,
    )
    .unwrap();
    // Bought on the ex day itself: those 5 are not entitled.
    trade(
        &mut db,
        &a,
        "avenue",
        "2026-05-11",
        TradeSide::Buy,
        "5",
        "210",
        0,
    )
    .unwrap();
    for (ex, est) in [("2026-05-11", 0), ("2026-06-01", 1), ("2099-01-02", 0)] {
        db.conn()
            .execute(
                "INSERT INTO dividend_event (id, asset_id, kind, ex_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES (?1, ?2, 'dividend', ?3, '0.26', 'USD', ?4, 'yahoo', 'x')",
                params![format!("ev-{ex}"), a, ex, est],
            )
            .unwrap();
    }
    let args = PendingIncomeArgs {
        since: Some("2026-01-01".into()),
        account_id: None,
    };
    let pend = engine::pending_income(db.conn(), &args).unwrap();
    assert_eq!(pend.len(), 1, "{pend:?}");
    let p = &pend[0];
    assert_eq!(p.account_id, "avenue");
    assert_eq!(p.quantity, "10");
    assert_eq!(p.gross, 260);
    assert_eq!(p.tax_withheld, 78);
    assert_eq!(p.net, 182);
    assert!(p.pay_date_guessed);
    let input = p.input.clone();
    record(db.conn_mut(), &Author::user(), "provento", |tx| {
        records::create_income(tx, &input)
    })
    .unwrap();
    assert!(engine::pending_income(db.conn(), &args).unwrap().is_empty());
}

/// Official B3 dividends that the calendar does not bring also become suggestions; what the
/// calendar already has (ex date one session later) is not repeated.
#[test]
fn pending_income_includes_official_b3_dividends_missing_from_the_agenda() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "HGLG11", AssetClass::Fii, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "100",
        "160",
        0,
    )
    .unwrap();
    for (ex, pay, seq) in [
        ("2026-03-31", "2026-04-14", 0),
        ("2026-04-30", "2026-05-15", 1),
    ] {
        db.conn()
            .execute(
                "INSERT INTO research_dividend (ticker, kind, ex_date, value, per_shares, pay_date, seq, source, fetched_at)
                 VALUES ('HGLG11', 'fii_income', ?1, '1.10', 1, ?2, ?3, 'b3_listados', 'x')",
                params![ex, pay, seq],
            )
            .unwrap();
    }
    // The calendar (Yahoo) already has March's, with the ex date one day later.
    db.conn()
        .execute(
            "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
             VALUES ('ev1', ?1, 'fii_income', '2026-04-01', '2026-04-14', '1.10', 'BRL', 0, 'yahoo', 'x')",
            [&a],
        )
        .unwrap();
    let args = PendingIncomeArgs {
        since: Some("2026-01-01".into()),
        account_id: None,
    };
    let pend = engine::pending_income(db.conn(), &args).unwrap();
    assert_eq!(pend.len(), 2, "{pend:?}");
    // March: B3 and Yahoo agree (just one, checked); April: B3 only.
    assert_eq!(pend[0].source, "b3_listados");
    assert_eq!(pend[0].ex_date, "2026-03-31");
    assert_eq!(pend[0].check.status, pending::CheckStatus::Confirmed);
    assert_eq!(pend[0].check.sources.len(), 2);
    assert_eq!(pend[1].source, "b3_listados");
    assert_eq!(pend[1].check.status, pending::CheckStatus::Single);
    assert_eq!(pend[1].ex_date, "2026-04-30");
    assert_eq!(pend[1].pay_date, "2026-05-15");
    assert_eq!(pend[1].net, 11000);
    // Posted from B3's (record date), the calendar's (ex date) does not come back.
    let b3 = pend[0].input.clone();
    record(db.conn_mut(), &Author::user(), "provento", |tx| {
        records::create_income(tx, &b3)
    })
    .unwrap();
    let left = engine::pending_income(db.conn(), &args).unwrap();
    assert_eq!(left.len(), 1, "{left:?}");
    assert_eq!(left[0].ex_date, "2026-04-30");
}

#[test]
fn rebalance_splits_class_money_by_asset_targets_and_funds_empty_classes() {
    let (_d, mut db) = db();
    let wege = asset(&mut db, "WEGE3", AssetClass::StockBr, None);
    let itsa = asset(&mut db, "ITSA4", AssetClass::StockBr, None);
    let fii = asset(&mut db, "HGLG11", AssetClass::Fii, None);
    trade(
        &mut db,
        &wege,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "100",
        "50",
        0,
    )
    .unwrap(); // 5.000
    price(&db, &wege, "2026-10-13", dec!(50));
    let class_targets = vec![
        TargetInput {
            scope: TargetScope::Class,
            key: "stock_br".into(),
            target_pct: "50".into(),
            tolerance_pct: None,
        },
        TargetInput {
            scope: TargetScope::Class,
            key: "fii".into(),
            target_pct: "50".into(),
            tolerance_pct: None,
        },
    ];
    let asset_targets = vec![
        TargetInput {
            scope: TargetScope::Asset,
            key: wege.clone(),
            target_pct: "50".into(),
            tolerance_pct: None,
        },
        TargetInput {
            scope: TargetScope::Asset,
            key: itsa.clone(),
            target_pct: "50".into(),
            tolerance_pct: None,
        },
    ];
    record(db.conn_mut(), &Author::user(), "alvos", |tx| {
        records::set_targets(tx, TargetScope::Class, &class_targets)?;
        records::set_targets(tx, TargetScope::Asset, &asset_targets)
    })
    .unwrap();
    // Contribution of 5.000: total becomes 10.000; stocks have 5.000 (on target), FII nothing.
    let r = engine::rebalance(
        db.conn(),
        &RebalanceArgs {
            contribution: Some(500000),
        },
    )
    .unwrap();
    let fii_line = r.lines.iter().find(|l| l.class == AssetClass::Fii).unwrap();
    assert_eq!(fii_line.current, 0);
    assert_eq!(fii_line.suggested, 500000);
    let stock = r
        .lines
        .iter()
        .find(|l| l.class == AssetClass::StockBr)
        .unwrap();
    assert_eq!(stock.suggested, 0);
    let w = r.assets.iter().find(|l| l.asset.id == wege).unwrap();
    let i = r.assets.iter().find(|l| l.asset.id == itsa).unwrap();
    assert_eq!((w.current, w.gap), (500000, 250000 - 500000));
    assert_eq!((i.current, i.gap), (0, 250000));
    assert!(w.out_of_band && i.out_of_band);
    assert_eq!(i.suggested, 0, "the class received no contribution");
    assert!(r.assets.iter().all(|l| l.asset.id != fii));

    // With a larger contribution, the stocks' share all goes to the asset below target.
    let r = engine::rebalance(
        db.conn(),
        &RebalanceArgs {
            contribution: Some(1500000),
        },
    )
    .unwrap();
    let stock = r
        .lines
        .iter()
        .find(|l| l.class == AssetClass::StockBr)
        .unwrap();
    let i = r.assets.iter().find(|l| l.asset.id == itsa).unwrap();
    assert_eq!(stock.suggested, 500000);
    assert_eq!(i.suggested, 500000);

    // Asset targets of a class summing more than 100% are refused.
    let mut bad = asset_targets.clone();
    bad[0].target_pct = "80".into();
    assert!(
        record(db.conn_mut(), &Author::user(), "x", |tx| {
            records::set_targets(tx, TargetScope::Asset, &bad)
        })
        .is_err()
    );
}

#[test]
fn bulk_deletes_are_one_undoable_change_set() {
    use serde_json::{Value, json};
    let dir = tempfile::tempdir().unwrap();
    let paths = crate::paths::TreasuryPaths::with_root(dir.path().join(".t")).unwrap();
    let api = crate::api::Api::new(paths).unwrap();
    let call = |cmd: &str, args: Value| {
        api.call(cmd, args)
            .unwrap_or_else(|e| panic!("{cmd} failed: {e}"))
    };
    call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "C"}),
    );
    let acc = call(
        "accounts.create",
        json!({"name": "XP", "kind": "brokerage", "opening_date": "2026-01-01"}),
    )["id"]
        .clone();
    let a = call(
        "assets.create",
        json!({"ticker": "WEGE3", "name": "WEG", "class": "stock_br"}),
    )["id"]
        .clone();
    let ids = |list: Value| -> Vec<Value> {
        list.as_array()
            .unwrap()
            .iter()
            .map(|x| x["id"].clone())
            .collect()
    };
    let trades = || ids(call("portfolio.list_trades", json!({})));
    let t: Vec<Value> = ["2026-03-02", "2026-03-03", "2026-03-04"]
        .iter()
        .map(|d| {
            call(
                "portfolio.create_trade",
                json!({"asset_id": a, "account_id": acc, "date": d, "side": "buy",
                       "quantity": "10", "price": "40"}),
            )["id"]
                .clone()
        })
        .collect();
    let balance = || call("accounts.get", json!({"id": acc}))["balance"].clone();
    assert_eq!(balance(), -120000);

    // Duplicates count once; the operations and the cash transactions go out together.
    let r = call(
        "portfolio.delete_trades",
        json!({"ids": [t[0], t[1], t[1]]}),
    );
    assert_eq!(r["updated"], 2);
    assert_eq!(trades(), [t[2].clone()]);
    assert_eq!(balance(), -40000);
    let last = &call("changes.list", json!({"limit": 1}))[0];
    assert_eq!(last["id"], r["change_set_id"]);
    assert_eq!(last["reason"], "2 operações removidas");

    // One undo restores both.
    call("changes.undo", json!({"id": r["change_set_id"]}));
    assert_eq!(trades().len(), 3);
    assert_eq!(balance(), -120000);

    // An id that does not exist cancels the whole batch; an empty list is refused.
    let err = api
        .call(
            "portfolio.delete_trades",
            json!({"ids": [t[0], "nao-existe"]}),
        )
        .unwrap_err();
    assert_eq!(err.code(), "not_found");
    assert_eq!(trades().len(), 3);
    let err = api
        .call("portfolio.delete_trades", json!({"ids": []}))
        .unwrap_err();
    assert_eq!(err.code(), "invalid");

    let income: Vec<Value> = ["2026-04-10", "2026-05-10"]
        .iter()
        .map(|d| {
            call(
                "portfolio.create_income",
                json!({"asset_id": a, "account_id": acc, "kind": "dividend",
                       "pay_date": d, "gross": 5000}),
            )["id"]
                .clone()
        })
        .collect();
    let r = call("portfolio.delete_incomes", json!({"ids": income}));
    assert_eq!(r["updated"], 2);
    assert!(ids(call("portfolio.list_income", json!({}))).is_empty());

    let action: Vec<Value> = ["2026-06-01", "2026-07-01"]
        .iter()
        .map(|d| {
            call(
                "portfolio.create_action",
                json!({"asset_id": a, "date": d, "kind": "split", "factor": "2"}),
            )["id"]
                .clone()
        })
        .collect();
    let r = call("portfolio.delete_actions", json!({"ids": [action[0]]}));
    assert_eq!(r["updated"], 1);
    assert_eq!(
        call("changes.list", json!({"limit": 1}))[0]["reason"],
        "Evento corporativo removido"
    );
    assert_eq!(
        ids(call("portfolio.list_actions", json!({}))),
        [action[1].clone()]
    );

    // For the AI, bulk delete is sensitive (item-by-item approval), like deleting a single one.
    for cmd in [
        "portfolio.delete_trades",
        "portfolio.delete_incomes",
        "portfolio.delete_actions",
    ] {
        assert_eq!(
            crate::api::policy::level(cmd),
            Some(crate::api::policy::Level::Sensitive)
        );
    }
}

#[test]
fn quick_income_fills_kind_account_and_tax_from_announced_dividends() {
    use quick_income::{QuickIncomeArgs, QuickIncomeRow, QuickIncomeState as S};
    let (_d, mut db) = db();
    db.conn()
        .execute(
            "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
             VALUES ('rico', 'Rico', 'brokerage', 'BRL', '2024-01-01', 'x', 'x')",
            [],
        )
        .unwrap();
    let itsa = asset(&mut db, "ITSA4", AssetClass::StockBr, None);
    let hglg = asset(&mut db, "HGLG11", AssetClass::Fii, None);
    trade(
        &mut db,
        &itsa,
        "xp",
        "2026-01-10",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &itsa,
        "rico",
        "2026-02-01",
        TradeSide::Buy,
        "300",
        "10",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &hglg,
        "xp",
        "2026-01-10",
        TradeSide::Buy,
        "10",
        "160",
        0,
    )
    .unwrap();
    for (kind, value) in [("jcp", "0.10"), ("dividend", "0.05")] {
        db.conn()
            .execute(
                "INSERT INTO research_dividend (ticker, kind, ex_date, value, per_shares, pay_date, source, fetched_at)
                 VALUES ('ITSA4', ?1, '2026-03-02', ?2, 1, '2026-04-01', 'b3_listados', 'x')",
                params![kind, value],
            )
            .unwrap();
    }
    let row = |date: &str, ticker: &str, amount: i64| QuickIncomeRow {
        date: date.into(),
        ticker: ticker.into(),
        amount,
        kind: None,
        account_id: None,
    };
    let args = QuickIncomeArgs {
        rows: vec![
            // Net JCP on the 300 Rico shares: 300 × 0,10 × 0,825 (17,5% withheld in 2026).
            row("2026-04-01", "ITSA4", 2475),
            // Dividend on the 100 XP shares.
            row("2026-04-01", "itsa4", 500),
            // Gross XP JCP typed one day later.
            row("2026-04-02", "ITSA4", 1000),
            row("2026-04-01", "XXXX9", 100),
            row("2026-04-01", "ITSA4", 2475),
            // FII with no announced income: kind by class, account with a position.
            row("2026-05-15", "HGLG11", 1100),
        ],
    };
    let today = parse_date("2026-06-01").unwrap();
    let lines = quick_income::preview(db.conn(), &args, today).unwrap();
    let st: Vec<S> = lines.iter().map(|l| l.state).collect();
    assert_eq!(
        st,
        [
            S::Ready,
            S::Ready,
            S::Ready,
            S::Error,
            S::Duplicate,
            S::Ready
        ]
    );
    let a = &lines[0];
    assert_eq!(a.kind, Some(IncomeKind::Jcp));
    assert_eq!(a.account_id.as_deref(), Some("rico"));
    assert_eq!(
        (a.gross, a.tax_withheld, a.net),
        (Some(3000), Some(525), Some(2475))
    );
    assert_eq!(a.ex_date.as_deref(), Some("2026-03-02"));
    assert_eq!(a.quantity.as_deref(), Some("300"));
    assert!(a.verified, "{:?}", a.notes);
    let b = &lines[1];
    assert_eq!(b.kind, Some(IncomeKind::Dividend));
    assert_eq!(b.account_id.as_deref(), Some("xp"));
    assert_eq!((b.gross, b.tax_withheld), (Some(500), Some(0)));
    let c = &lines[2];
    assert_eq!(c.kind, Some(IncomeKind::Jcp));
    assert_eq!(c.account_id.as_deref(), Some("xp"));
    assert_eq!(
        (c.gross, c.tax_withheld, c.net),
        (Some(1000), Some(175), Some(825))
    );
    assert!(lines[3].error.as_deref().unwrap().contains("XXXX9"));
    let f = &lines[5];
    assert_eq!(f.kind, Some(IncomeKind::FiiIncome));
    assert_eq!(f.account_id.as_deref(), Some("xp"));
    assert_eq!(f.ex_date, None);
    assert!(!f.verified);

    let report = record(db.conn_mut(), &Author::user(), "lote", |tx| {
        quick_income::apply(tx, &args, today)
    })
    .unwrap()
    .value;
    assert_eq!(report.created, 4);
    let n: i64 = db
        .conn()
        .query_row(
            "SELECT count(*) FROM income_event WHERE txn_id IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 4);
    // Again: everything already recorded.
    let again = quick_income::preview(db.conn(), &args, today).unwrap();
    assert!(again.iter().all(|l| l.state != S::Ready));

    // Future payment stays as announced; a forced account applies even without a position.
    let future = QuickIncomeArgs {
        rows: vec![QuickIncomeRow {
            account_id: Some("rico".into()),
            ..row("2026-07-01", "HGLG11", 1100)
        }],
    };
    let l = &quick_income::preview(db.conn(), &future, today).unwrap()[0];
    assert_eq!(l.state, S::Ready);
    assert_eq!(l.status, Some(IncomeStatus::Announced));
    assert_eq!(l.account_id.as_deref(), Some("rico"));
}

/// Sources that disagree become a single income, with the official value and the warning;
/// Yahoo, which adds JCP and dividend of the same day, matches B3's sum.
#[test]
fn pending_income_reconciles_divergent_sources() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ITSA4", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "1000",
        "10",
        0,
    )
    .unwrap();
    for (kind, ex, value, pay, seq) in [
        ("jcp", "2026-03-02", "0.20", "2026-04-01", 0),
        ("dividend", "2026-03-02", "0.05", "2026-04-01", 1),
        ("dividend", "2026-05-29", "0.10", "2026-07-01", 2),
    ] {
        db.conn()
            .execute(
                "INSERT INTO research_dividend (ticker, kind, ex_date, value, per_shares, pay_date, seq, source, fetched_at)
                 VALUES ('ITSA4', ?1, ?2, ?3, 1, ?4, ?5, 'b3_listados', 'x')",
                params![kind, ex, value, pay, seq],
            )
            .unwrap();
    }
    for (id, ex, amount, pay, source) in [
        // Yahoo adds JCP + dividend of the day (0,25): matches.
        ("y1", "2026-03-03", "0.25", None, "yahoo"),
        // May: Yahoo with a wrong value and Fundamentus with another payment date.
        ("y2", "2026-06-01", "0.12", None, "yahoo"),
        (
            "f2",
            "2026-05-29",
            "0.10",
            Some("2026-07-03"),
            "fundamentus",
        ),
        // Only in the calendar, two sources disagreeing.
        ("y3", "2026-08-10", "0.30", None, "yahoo"),
        (
            "f3",
            "2026-08-10",
            "0.33",
            Some("2026-08-20"),
            "fundamentus",
        ),
    ] {
        db.conn()
            .execute(
                "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES (?1, ?2, 'dividend', ?3, ?4, ?5, 'BRL', 0, ?6, 'x')",
                params![id, a, ex, pay, amount, source],
            )
            .unwrap();
    }
    let args = PendingIncomeArgs {
        since: Some("2026-01-01".into()),
        account_id: None,
    };
    let pend: Vec<_> = engine::pending_income(db.conn(), &args)
        .unwrap()
        .into_iter()
        .filter(|p| p.pay_date.as_str() <= "2026-12-31")
        .collect();
    let summary: Vec<(String, String, pending::CheckStatus)> = pend
        .iter()
        .map(|p| {
            (
                p.ex_date.clone(),
                p.amount_per_share.clone(),
                p.check.status,
            )
        })
        .collect();
    use pending::CheckStatus::*;
    assert_eq!(
        summary,
        vec![
            ("2026-03-02".into(), "0.2".into(), Confirmed),
            ("2026-03-02".into(), "0.05".into(), Confirmed),
            ("2026-05-29".into(), "0.1".into(), Divergent),
            ("2026-08-10".into(), "0.33".into(), Divergent),
        ],
        "{pend:#?}"
    );
    let may = &pend[2];
    assert_eq!(may.pay_date, "2026-07-01");
    assert!(
        may.check
            .note
            .as_deref()
            .unwrap()
            .contains("Fundamentus dizia pagamento em 03/07/2026")
    );
    let aug = &pend[3];
    assert_eq!(aug.source, "fundamentus");
    assert_eq!(aug.pay_date, "2026-08-20");
}

/// FII without B3 events: Yahoo (ex date, yield, no payment) and Fundamentus
/// (record date, amortization, with payment) describe the same income: a single suggestion, with
/// the kind and payment from Fundamentus. And the warning for an asset with no quote and no trades.
#[test]
fn pending_income_pairs_yahoo_and_fundamentus_with_different_kinds() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "XPTO11", AssetClass::Fii, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "100",
        "100",
        0,
    )
    .unwrap();
    for (id, kind, ex, amount, pay, source) in [
        // Friday (record date) × Monday (ex date): 1 business day.
        (
            "f1",
            "amortization",
            "2026-06-05",
            "0.505",
            Some("2026-06-15"),
            "fundamentus",
        ),
        ("y1", "fii_income", "2026-06-08", "0.50", None, "yahoo"),
        // Another month, Yahoo only.
        ("y2", "fii_income", "2026-07-08", "0.80", None, "yahoo"),
    ] {
        db.conn()
            .execute(
                "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'BRL', 0, ?7, 'x')",
                params![id, a, kind, ex, pay, amount, source],
            )
            .unwrap();
    }
    db.conn()
        .execute(
            "INSERT INTO price (asset_id, date, close, volume, source, fetched_at)
             VALUES (?1, '2026-03-02', '90', '0', 'yahoo', 'x')",
            [&a],
        )
        .unwrap();
    let pend = engine::pending_income(
        db.conn(),
        &PendingIncomeArgs {
            since: Some("2026-01-01".into()),
            account_id: None,
        },
    )
    .unwrap();
    assert_eq!(pend.len(), 2, "{pend:#?}");
    let june = &pend[0];
    assert_eq!(june.kind, IncomeKind::Amortization);
    assert_eq!(
        (june.source.as_str(), june.pay_date.as_str()),
        ("fundamentus", "2026-06-15")
    );
    assert_eq!(june.check.status, pending::CheckStatus::Confirmed);
    assert_eq!(june.check.sources.len(), 2);
    assert!(
        june.check
            .note
            .as_deref()
            .unwrap()
            .contains("tipos diferentes")
    );
    assert_eq!(june.gross, 5_050);
    assert!(
        june.asset_warning
            .as_deref()
            .unwrap()
            .contains("deslistado ou incorporado")
    );
    assert_eq!(pend[1].check.status, pending::CheckStatus::Single);
}

/// Come-cotas in May and November: 15% of the yield per lot, shares redeemed, total cost
/// kept; a rule chosen by the user (exempt) turns it off.
#[test]
fn come_cotas_takes_shares_in_may_and_november() {
    let (_d, mut db) = db();
    let f = asset(&mut db, "FUNDO-RF", AssetClass::Fund, None);
    trade(
        &mut db,
        &f,
        "xp",
        "2025-01-02",
        TradeSide::Buy,
        "1000",
        "1",
        0,
    )
    .unwrap();
    price(&db, &f, "2025-05-30", dec!(1.10));
    price(&db, &f, "2025-11-28", dec!(1.20));
    let (rule, evs, _) =
        come_cotas::for_asset(db.conn(), &f, None, parse_date("2026-01-10").unwrap()).unwrap();
    assert_eq!(rule.regime, come_cotas::Regime::Long);
    assert_eq!(evs.len(), 2, "{evs:?}");
    assert_eq!(evs[0].date, "2025-05-30");
    assert_eq!(evs[0].income, "100");
    assert_eq!(evs[0].tax, "15");
    // November: only the yield since May (1,10 → 1,20) on the remaining shares.
    let left = dec!(1000) - dec!(15) / dec!(1.10);
    let nov_tax = (dec!(0.10) * left * dec!(0.15)).round_dp(2);
    assert_eq!(evs[1].tax, nov_tax.normalize().to_string());
    let book = engine::load_book(db.conn(), &f, None).unwrap();
    let st = engine::replay(db.conn(), &book, parse_date("2026-01-10").unwrap()).unwrap();
    let expected = left - dec!(0.10) * left * dec!(0.15) / dec!(1.20);
    assert!(
        (st.quantity() - expected).abs() < dec!(0.0001),
        "{}",
        st.quantity()
    );
    assert_eq!(st.cost(), dec!(1000));
    // The statement shows the synthetic lines.
    let lines = statement::statement(
        db.conn(),
        &statement::StatementArgs {
            asset_id: f.clone(),
            account_id: None,
        },
    )
    .unwrap();
    assert_eq!(lines.iter().filter(|l| l.kind == "come_cotas").count(), 2);
    // Equity fund (chosen rule): no come-cotas.
    come_cotas::set_rule(db.conn(), &f, Some(come_cotas::Regime::Exempt)).unwrap();
    let book = engine::load_book(db.conn(), &f, None).unwrap();
    let st = engine::replay(db.conn(), &book, parse_date("2026-01-10").unwrap()).unwrap();
    assert_eq!(st.quantity(), dec!(1000));
}

#[test]
fn risk_from_monthly_returns_with_today_weights() {
    use chrono::Datelike;
    let (_d, mut db) = db();
    let today = crate::time::today_local();
    let first = crate::time::add_months(today.with_day(1).unwrap(), -16);
    let month_end = |k: i32| crate::time::add_months(first, k + 1).pred_opt().unwrap();
    let a = asset(&mut db, "AAAA3", AssetClass::StockBr, None);
    let b = asset(&mut db, "BBBB11", AssetClass::Fii, None);
    let buy = crate::time::fmt_date(first);
    trade(&mut db, &a, "xp", &buy, TradeSide::Buy, "100", "10", 0).unwrap();
    trade(&mut db, &b, "xp", &buy, TradeSide::Buy, "100", "10", 0).unwrap();
    // A rises and falls 5% alternately; B does the opposite (correlation −1), with half the
    // amplitude.
    let (mut pa, mut pb) = (dec!(10), dec!(10));
    price(&db, &a, &buy, pa);
    price(&db, &b, &buy, pb);
    for k in 0..17 {
        let up = k % 2 == 0;
        pa *= if up { dec!(1.05) } else { dec!(0.95) };
        pb *= if up { dec!(0.975) } else { dec!(1.025) };
        let d = crate::time::fmt_date(month_end(k));
        price(&db, &a, &d, pa);
        price(&db, &b, &d, pb);
    }
    let r = risk::risk(db.conn(), &risk::RiskArgs { months: Some(12) }).unwrap();
    assert_eq!(r.months.len(), 12);
    let ma = r.assets.iter().find(|x| x.asset_id == a).unwrap();
    let mb = r.assets.iter().find(|x| x.asset_id == b).unwrap();
    let (va, vb) = (
        ma.metrics.as_ref().unwrap().volatility,
        mb.metrics.as_ref().unwrap().volatility,
    );
    assert!((va / vb - 2.0).abs() < 0.05, "{va} {vb}");
    // Correlation between the two and between the classes: −1.
    assert_eq!(r.asset_correlation.values[0][1], Some(-1.0));
    assert_eq!(r.class_correlation.values[0][1], Some(-1.0));
    // The combined portfolio has less risk than the most volatile asset, and the parts sum to 1.
    let p = r.portfolio.as_ref().unwrap();
    assert!(p.volatility < va, "{} {va}", p.volatility);
    let shares: f64 = r.by_class.iter().filter_map(|c| c.risk_share).sum();
    assert!((shares - 1.0).abs() < 0.01, "{shares}");
    // A's worst drop: −5% in the month and the slow retreat of 1,05 × 0,95 = 0,9975 every two.
    let dd = ma.metrics.as_ref().unwrap().max_drawdown;
    assert!((-0.08..=-0.05).contains(&dd), "{dd}");
    assert!(r.realized.is_some());
    assert_eq!(p.sharpe, None, "no CDI in the test vault");
}

#[test]
fn texts_follow_the_locale() {
    use crate::i18n::{Locale, set_current};
    use quick_income::{QuickIncomeArgs, QuickIncomeRow};
    let (_d, mut db) = db();
    let itsa = asset(&mut db, "ITSA4", AssetClass::StockBr, None);
    let row = |ticker: &str| QuickIncomeRow {
        date: "2026-04-01".into(),
        ticker: ticker.into(),
        amount: 500,
        kind: None,
        account_id: None,
    };
    let args = QuickIncomeArgs {
        rows: vec![row("ITSA4"), row("XXXX9")],
    };
    let today = parse_date("2026-06-01").unwrap();
    // No operation on the asset: the UI offers picking the account via the field, not the text.
    let pt = quick_income::preview(db.conn(), &args, today).unwrap();
    assert_eq!(
        pt[0].error.as_deref(),
        Some("nenhuma operação de ITSA4 registrada: escolha a conta")
    );
    assert!(pt[0].needs_account && !pt[1].needs_account);
    set_current(Locale::En);
    let en = quick_income::preview(db.conn(), &args, today).unwrap();
    trade(
        &mut db,
        &itsa,
        "xp",
        "2026-01-10",
        TradeSide::Buy,
        "10",
        "9.5",
        0,
    )
    .unwrap();
    let lines = statement::statement(
        db.conn(),
        &statement::StatementArgs {
            asset_id: itsa.clone(),
            account_id: None,
        },
    )
    .unwrap();
    let kind = IncomeKind::FiiIncome.label();
    set_current(Locale::PtBr);
    assert_eq!(
        en[0].error.as_deref(),
        Some("no trades of ITSA4 recorded: choose the account")
    );
    assert!(en[0].needs_account);
    assert_eq!(
        en[1].error.as_deref(),
        Some("XXXX9 isn't registered in the portfolio")
    );
    assert_eq!(
        (lines[0].label.as_str(), lines[0].origin.as_str()),
        ("Buy", "manual")
    );
    assert_eq!(kind, "FII income");
    let desc: String = db
        .conn()
        .query_row(
            "SELECT t.description FROM txn t JOIN trade r ON r.txn_id = t.id",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(desc, "Buy 10 ITSA4 @ R$9.50");
}

/// The agent batch's operations show up as the agent's in the statement, in any language
/// the note was recorded in.
#[test]
fn agent_batch_notes_are_recognized_in_any_language() {
    use crate::i18n::{Locale, set_current};
    let (_d, mut db) = db();
    let a = asset(&mut db, "AAAA3", AssetClass::StockBr, None);
    let mut add = |date: &str, notes: &str| {
        let t = TradeInput {
            asset_id: a.clone(),
            account_id: "xp".into(),
            date: date.into(),
            side: TradeSide::Buy,
            quantity: "1".into(),
            price: "10".into(),
            fees: None,
            taxes: None,
            fx_rate: None,
            day_trade: None,
            note_number: None,
            notes: Some(notes.into()),
            cash_movement: Some(false),
            external_id: None,
        };
        record(db.conn_mut(), &Author::user(), "op", |tx| {
            records::create_trade(tx, &t)
        })
        .unwrap();
    };
    add("2026-01-05", "Lote do agente: extrato da XP");
    add("2026-01-06", "Agent batch: XP statement");
    add("2026-01-07", "Lote del agente: extracto de XP");
    add("2026-01-08", "comprado no home broker");
    set_current(Locale::En);
    let lines = statement::statement(
        db.conn(),
        &statement::StatementArgs {
            asset_id: a.clone(),
            account_id: None,
        },
    )
    .unwrap();
    set_current(Locale::PtBr);
    let origins: Vec<&str> = lines.iter().map(|l| l.origin.as_str()).collect();
    assert_eq!(origins, ["agent", "agent", "agent", "manual"]);
}

/// JCP: 15% withheld until 2025 and 17,5% from 01/01/2026, by the credit or payment date
/// (whichever comes first). Sources: LC 224/2025 art. 8º (Lei 9.249/1995 art. 9º §2º) and art.
/// 14, III; Receita, Nota Cetad/Coest nº 009/2026.
#[test]
fn jcp_withholding_is_17_5_percent_from_2026_by_credit_or_payment_date() {
    use crate::assets::AssetClass;
    use crate::portfolio::records::{credit_date, default_withholding, jcp_rate};
    let d = |s: &str| parse_date(s).unwrap();
    assert_eq!(jcp_rate(d("2025-12-31")), dec!(0.15));
    assert_eq!(jcp_rate(d("2026-01-01")), dec!(0.175));
    // Credited in December 2025 and paid in 2026: the credit date applies.
    let on = credit_date(Some(d("2025-12-15")), d("2026-02-10"));
    assert_eq!(
        default_withholding(IncomeKind::Jcp, AssetClass::StockBr, 10_000, on),
        1_500
    );
    // Credited and paid in 2026.
    let on = credit_date(Some(d("2026-01-05")), d("2026-02-10"));
    assert_eq!(
        default_withholding(IncomeKind::Jcp, AssetClass::StockBr, 10_000, on),
        1_750
    );
}

/// Account where the asset is held (custody): bank or brokerage, never a card; switch and turn off.
#[test]
fn asset_keeps_the_account_where_it_is_held() {
    let (_d, mut db) = db();
    record(db.conn_mut(), &Author::system("t"), "contas", |tx| {
        for (id, kind) in [("itau", "checking"), ("cartao", "credit_card")] {
            tx.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
                 VALUES (?1, ?1, ?2, 'BRL', '2024-01-01', 'x', 'x')",
                params![id, kind],
            )?;
        }
        Ok(())
    })
    .unwrap();
    let cdb = asset(&mut db, "CDBITAU", AssetClass::StockBr, None);
    let patch = |db: &mut Db, acc: Option<Option<&str>>| {
        let p: assets::AssetPatch = serde_json::from_value(match acc {
            Some(Some(a)) => serde_json::json!({"id": cdb, "account_id": a}),
            Some(None) => serde_json::json!({"id": cdb, "account_id": null}),
            None => serde_json::json!({"id": cdb}),
        })
        .unwrap();
        record(db.conn_mut(), &Author::user(), "ativo", |tx| {
            assets::update(tx, &p)
        })
    };
    assert_eq!(assets::get(db.conn(), &cdb).unwrap().account_id, None);
    patch(&mut db, Some(Some("itau"))).unwrap();
    assert_eq!(
        assets::get(db.conn(), &cdb).unwrap().account_id.as_deref(),
        Some("itau")
    );
    // Without the field: stays as it was.
    patch(&mut db, None).unwrap();
    assert_eq!(
        assets::get(db.conn(), &cdb).unwrap().account_id.as_deref(),
        Some("itau")
    );
    assert!(patch(&mut db, Some(Some("cartao"))).is_err());
    assert!(patch(&mut db, Some(Some("nao-existe"))).is_err());
    patch(&mut db, Some(None)).unwrap();
    assert_eq!(assets::get(db.conn(), &cdb).unwrap().account_id, None);
}

/// Total return on the sale: the income from the units sold counts alongside the price result
/// (the IR one), by record date and in the same fraction as the cost; a sale on the record date
/// does not carry the income; amortization does not count.
#[test]
fn sales_ledger_adds_income_received_by_the_sold_units() {
    let (_d, mut db) = db();
    let f = asset(&mut db, "XPTO11", AssetClass::Fii, None);
    let buy = |db: &mut Db, date: &str| {
        trade(db, &f, "xp", date, TradeSide::Buy, "100", "100", 0).unwrap();
    };
    buy(&mut db, "2025-01-02");
    buy(&mut db, "2025-05-02");
    trade(
        &mut db,
        &f,
        "xp",
        "2025-07-15",
        TradeSide::Sell,
        "100",
        "95",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &f,
        "xp",
        "2025-09-01",
        TradeSide::Sell,
        "100",
        "96",
        0,
    )
    .unwrap();
    let income = |db: &mut Db, kind: &str, ex: &str, pay: &str, net: i64| {
        record(db.conn_mut(), &Author::system("t"), "provento", |tx| {
            tx.execute(
                "INSERT INTO income_event (id, asset_id, account_id, kind, ex_date, pay_date, gross,
                                           net, currency, status, created_at, updated_at)
                 VALUES (?1, ?2, 'xp', ?3, ?4, ?5, ?6, ?6, 'BRL', 'received', 'x', 'x')",
                params![crate::ids::new_id(), f, kind, ex, pay, net],
            )?;
            Ok(())
        })
        .unwrap();
    };
    // 100 shares: R$ 80; 200 shares: R$ 160; at the 1st sale's record date: R$ 50 (stays with those
    // that remain); amortization: out.
    income(&mut db, "fii_income", "2025-03-31", "2025-04-14", 8_000);
    income(&mut db, "fii_income", "2025-06-30", "2025-07-14", 16_000);
    income(&mut db, "fii_income", "2025-07-15", "2025-07-25", 5_000);
    income(&mut db, "amortization", "2025-08-01", "2025-08-10", 90_000);
    let r = sales::sales(
        db.conn(),
        &sales::SalesArgs {
            to: Some("2025-12-31".into()),
            deflator: Some(Deflator::None),
            ..Default::default()
        },
    )
    .unwrap();
    let (late, early) = (&r.rows[0], &r.rows[1]);
    // 1st sale: half of the position carries half of R$ 240.
    assert_eq!((early.date.as_str(), early.result), ("2025-07-15", -50_000));
    assert_eq!(early.income, 12_000);
    assert_eq!(early.total_result, -38_000);
    assert_eq!(early.real_total_result, Some(-38_000));
    // 2nd sale: the rest (R$ 120) plus the income of the 1st sale's record date (R$ 50).
    assert_eq!((late.result, late.income), (-40_000, 17_000));
    assert_eq!(late.total_result, -23_000);
    assert_eq!((r.totals.result, r.totals.income), (-90_000, 29_000));
    assert_eq!(r.totals.total_result, -61_000);
}

#[test]
fn sales_ledger_shows_nominal_and_real_result_of_each_sale() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ABCD3", AssetClass::StockBr, None);
    let b = asset(&mut db, "EFGH3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2025-01-02",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &a,
        "xp",
        "2025-07-01",
        TradeSide::Buy,
        "100",
        "12",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-02",
        TradeSide::Sell,
        "100",
        "13",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &b,
        "xp",
        "2025-06-02",
        TradeSide::Buy,
        "10",
        "50",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &b,
        "xp",
        "2025-12-01",
        TradeSide::Sell,
        "10",
        "45",
        0,
    )
    .unwrap();
    let vals: Vec<_> = (0..14)
        .map(|i| {
            let (y, m) = (2025 + i / 12, i % 12 + 1);
            (
                parse_date(&format!("{y}-{m:02}-01")).unwrap(),
                dec!(1),
                false,
            )
        })
        .collect();
    crate::indices::upsert_values(db.conn(), "ipca", &vals, "test").unwrap();
    let run = |deflator: Deflator, from: Option<&str>| {
        sales::sales(
            db.conn(),
            &sales::SalesArgs {
                from: from.map(str::to_string),
                to: Some("2026-02-01".into()),
                deflator: Some(deflator),
                ..Default::default()
            },
        )
        .unwrap()
    };
    let r = run(Deflator::Ipca, None);
    assert_eq!(r.rows.len(), 2);
    // Most recent first: ABCD3, average price 11 → cost R$ 1.100, sale R$ 1.300.
    let s = &r.rows[0];
    assert_eq!(
        (s.ticker.as_deref(), s.cost, s.proceeds, s.result),
        (Some("ABCD3"), 110_000, 130_000, 20_000)
    );
    // Half of each purchase: R$ 500 from Jan/2025 and R$ 600 from Jul/2025, weighted by cost.
    assert_eq!(s.holding_days, Some(267));
    let real = s.real_cost.unwrap();
    assert!(real > s.cost && real < 125_000, "{real}");
    assert_eq!(s.real_result, Some(s.proceeds - real));
    assert!(s.real_result.unwrap() < s.result);
    assert!(s.index_change.unwrap() > 0.05);
    // EFGH3: nominal loss and a larger real loss.
    let l = &r.rows[1];
    assert_eq!(l.result, -5_000);
    assert!(l.real_result.unwrap() < l.result);
    assert_eq!(
        (r.totals.gains, r.totals.losses, r.totals.missing),
        (1, 1, 0)
    );
    assert_eq!(r.totals.result, 15_000);
    assert_eq!(
        r.totals.inflation_effect,
        Some(r.totals.real_cost.unwrap() - r.totals.cost)
    );
    assert_eq!(
        r.by_month
            .iter()
            .map(|g| g.key.as_str())
            .collect::<Vec<_>>(),
        ["2025-12", "2026-01"]
    );
    // Nominal: the real is the result itself.
    let n = run(Deflator::None, None);
    assert_eq!(n.rows[0].real_result, Some(n.rows[0].result));
    // Period: only the 2026 sale.
    assert_eq!(run(Deflator::Ipca, Some("2026-01-01")).rows.len(), 1);
    // No index in the period (M2 not stored): real stays blank and counts as missing.
    let m = run(Deflator::M2Br, None);
    assert_eq!((m.rows[0].real_result, m.totals.missing), (None, 2));
}

#[test]
fn sales_ledger_uses_the_country_cost_method_and_shows_the_irr() {
    use cost_method::{CostMethod, SetCostMethodArgs};
    let (_d, mut db) = db();
    let a = asset(&mut db, "ABCD3", AssetClass::StockBr, None);
    for (date, side, qty, price) in [
        ("2025-01-02", TradeSide::Buy, "100", "10"),
        ("2025-07-01", TradeSide::Buy, "100", "12"),
        ("2026-01-02", TradeSide::Sell, "100", "13"),
    ] {
        trade(&mut db, &a, "xp", date, side, qty, price, 0).unwrap();
    }
    let run = |db: &Db| {
        sales::sales(
            db.conn(),
            &sales::SalesArgs {
                to: Some("2026-02-01".into()),
                deflator: Some(Deflator::None),
                ..Default::default()
            },
        )
        .unwrap()
    };
    // Brazil: average price, without the average columns.
    let r = run(&db);
    assert_eq!(
        (r.cost_method, r.cost_method_automatic),
        (CostMethod::Average, true)
    );
    let s = &r.rows[0];
    assert_eq!(
        (s.cost, s.average_cost, s.average_result),
        (110_000, None, None)
    );
    assert!(s.irr.unwrap() > 0.0);
    assert_eq!(r.totals.average_result, None);
    // US: FIFO by country; the sale takes the whole Jan/2025 lot.
    db.conn()
        .execute("UPDATE household SET country = 'US'", [])
        .unwrap();
    let r = run(&db);
    assert_eq!(
        (r.cost_method, r.cost_method_automatic),
        (CostMethod::Fifo, true)
    );
    let s = &r.rows[0];
    assert_eq!(
        (s.cost, s.result, s.average_cost, s.average_result),
        (100_000, 30_000, Some(110_000), Some(20_000))
    );
    assert_eq!(s.holding_days, Some(365));
    // Exactly one year between purchase and sale: +30% a year.
    assert!((s.irr.unwrap() - 0.30).abs() < 1e-6, "{:?}", s.irr);
    assert_eq!(r.totals.average_result, Some(20_000));
    // The family's choice overrides the country's rule.
    cost_method::set(
        db.conn(),
        &SetCostMethodArgs {
            method: Some(CostMethod::Average),
        },
    )
    .unwrap();
    let st = cost_method::state(db.conn()).unwrap();
    assert_eq!(
        (st.method, st.automatic, st.country_method),
        (CostMethod::Average, false, CostMethod::Fifo)
    );
    assert_eq!(run(&db).rows[0].cost, 110_000);
    // An explicit request overrides the choice.
    let pool = sales::sales(
        db.conn(),
        &sales::SalesArgs {
            to: Some("2026-02-01".into()),
            cost_method: Some(CostMethod::UkPool),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (pool.cost_method, pool.rows[0].cost),
        (CostMethod::UkPool, 110_000)
    );
}

fn perf_args(from: &str, to: &str, method: Option<TwrMethod>) -> PerformanceArgs {
    PerformanceArgs {
        from_month: Some(from.into()),
        to_month: Some(to.into()),
        deflator: Some(Deflator::None),
        asset_id: None,
        class: None,
        method,
    }
}

#[test]
fn bridge_goes_from_start_to_end_value_line_by_line() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ABCD3", AssetClass::StockBr, None);
    // Buys 100 at R$ 10 (R$ 10 brokerage), sells 50 at R$ 11 (R$ 5), income of R$ 10
    // with R$ 1,50 IR withheld, and the rest closes the period at R$ 12.
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-02",
        TradeSide::Buy,
        "100",
        "10",
        1000,
    )
    .unwrap();
    price(&db, &a, "2026-01-30", dec!(10));
    price(&db, &a, "2026-02-27", dec!(11));
    trade(
        &mut db,
        &a,
        "xp",
        "2026-03-02",
        TradeSide::Sell,
        "50",
        "11",
        500,
    )
    .unwrap();
    let income = IncomeInput {
        asset_id: a.clone(),
        account_id: "xp".into(),
        kind: IncomeKind::Jcp,
        ex_date: None,
        pay_date: "2026-03-16".into(),
        quantity: None,
        gross: 1000,
        tax_withheld: Some(150),
        fx_rate: None,
        status: None,
        cash_movement: None,
    };
    record(db.conn_mut(), &Author::user(), "provento", |tx| {
        records::create_income(tx, &income)
    })
    .unwrap();
    price(&db, &a, "2026-03-31", dec!(12));
    let b = bridge::bridge(db.conn(), &perf_args("2026-01", "2026-03", None)).unwrap();
    let l = &b.lines;
    assert_eq!(b.from, "2026-01-01");
    assert_eq!(b.to, "2026-03-31");
    assert_eq!(l.start_value, 0);
    assert_eq!(l.contributions, 101_000);
    assert_eq!(l.withdrawals, 54_500 + 850);
    assert_eq!(l.income, 1_000);
    assert_eq!(l.fees, 1_500);
    assert_eq!(l.taxes, 150);
    assert_eq!(l.end_value, 60_000);
    // Price: +R$ 50 on those sold (10 → 11) and +R$ 100 on those kept (10 → 12).
    assert_eq!(l.price_gain, 15_000);
    assert_eq!(l.fx_gain, 0);
    assert_eq!(l.delta, 60_000 - 101_000 + 55_350);
    assert_eq!(
        l.start_value + l.price_gain + l.income - l.fees - l.taxes + l.contributions
            - l.withdrawals,
        l.end_value
    );
    assert_eq!(b.assets.len(), 1);
    assert_eq!(b.assets[0].lines, *l);
    assert_eq!(b.inflation_loss, None, "no deflator");
    // February only: starts at R$ 1.000, rises to R$ 1.100, no flows.
    let feb = bridge::bridge(db.conn(), &perf_args("2026-02", "2026-02", None)).unwrap();
    assert_eq!(feb.lines.start_value, 100_000);
    assert_eq!(feb.lines.price_gain, 10_000);
    assert_eq!(feb.lines.delta, 10_000);
    assert_eq!(feb.lines.contributions + feb.lines.withdrawals, 0);
}

#[test]
fn daily_twr_neutralizes_a_large_contribution_in_a_volatile_month() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "WXYZ3", AssetClass::StockBr, None);
    // R$ 1.000 rise 10% until the 14th; on the 15th a large contribution comes in, price back to R$ 10.
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-02",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    for (day, close) in [
        ("2026-01-02", dec!(10)),
        ("2026-01-09", dec!(10)),
        ("2026-01-14", dec!(11)),
        ("2026-01-15", dec!(11)),
        ("2026-01-22", dec!(11)),
        ("2026-01-30", dec!(10)),
    ] {
        price(&db, &a, day, close);
    }
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-15",
        TradeSide::Buy,
        "900",
        "11",
        0,
    )
    .unwrap();
    let daily = engine::performance(
        db.conn(),
        &perf_args("2026-01", "2026-01", Some(TwrMethod::Daily)),
    )
    .unwrap();
    assert_eq!(daily.method, TwrMethod::Daily);
    // Daily: 1,10 × (10/11) − 1 = 0.
    assert!(
        daily.monthly[0].unwrap().abs() < 1e-9,
        "{:?}",
        daily.monthly
    );
    let risk = daily.risk.unwrap();
    assert!(
        (risk.max_drawdown - (10.0 / 11.0 - 1.0)).abs() < 1e-6,
        "{risk:?}"
    );
    assert_eq!(risk.drawdown_peak.as_deref(), Some("2026-01-14"));
    assert!(risk.drawdown_recovered.is_none());
    assert!(risk.longest_drawdown_ongoing);
    assert!(risk.semideviation > 0.0);
    // Monthly (Modified Dietz): the mid-month contribution distorts the return.
    let monthly = engine::performance(db.conn(), &perf_args("2026-01", "2026-01", None)).unwrap();
    assert_eq!(monthly.method, TwrMethod::Monthly);
    assert!(monthly.monthly[0].unwrap() < -0.03, "{:?}", monthly.monthly);
    // The advanced setting turns on daily without passing the method.
    crate::settings::set(db.conn(), engine::DAILY_TWR_SETTING, &true).unwrap();
    let by_setting =
        engine::performance(db.conn(), &perf_args("2026-01", "2026-01", None)).unwrap();
    assert_eq!(by_setting.method, TwrMethod::Daily);
}

#[test]
fn return_risk_measures_downside_and_the_longest_drawdown() {
    let d = |s: &str| parse_date(s).unwrap();
    let series = vec![
        (d("2026-01-31"), 0.10),
        (d("2026-02-28"), -0.20),
        (d("2026-03-31"), 0.05),
        (d("2026-04-30"), 0.25),
        (d("2026-05-31"), 0.01),
    ];
    let r = engine::return_risk(&series, d("2025-12-31"), 12.0).unwrap();
    assert_eq!(r.periods, 5);
    assert!((r.max_drawdown + 0.20).abs() < 1e-9);
    assert_eq!(r.drawdown_peak.as_deref(), Some("2026-01-31"));
    assert_eq!(r.drawdown_trough.as_deref(), Some("2026-02-28"));
    // 1,1 × 0,8 × 1,05 × 1,25 = 1,155 ≥ 1,1: recovered in April.
    assert_eq!(r.drawdown_recovered.as_deref(), Some("2026-04-30"));
    assert_eq!(r.longest_drawdown_days, 89, "31/01 to 30/04");
    assert!(!r.longest_drawdown_ongoing);
    // Only returns below the mean (0,042) enter the semideviation.
    let mean = 0.21 / 5.0;
    let down = (((-0.20f64 - mean).powi(2) + (0.01f64 - mean).powi(2)) / 5.0).sqrt() * 12f64.sqrt();
    assert!(
        (r.semideviation - down).abs() < 1e-5,
        "{} × {down}",
        r.semideviation
    );
    assert!(engine::return_risk(&series[..1], d("2025-12-31"), 12.0).is_none());
}

#[test]
fn bridge_splits_the_exchange_rate_out_of_a_foreign_price_gain() {
    let (_d, mut db) = db();
    for (d, r) in [("2026-01-01", "5.00"), ("2026-03-01", "5.50")] {
        crate::fx::set_manual(
            db.conn(),
            &crate::fx::FxRateInput {
                base: "USD".into(),
                quote: "BRL".into(),
                date: d.into(),
                rate: r.into(),
            },
        )
        .unwrap();
    }
    let a = asset(&mut db, "EXUS", AssetClass::StockUs, None);
    // USD 2.000 at 5,00 become USD 2.200 at 5,50: R$ 10.000 → R$ 12.100.
    trade(
        &mut db,
        &a,
        "avenue",
        "2026-01-05",
        TradeSide::Buy,
        "10",
        "200",
        0,
    )
    .unwrap();
    price(&db, &a, "2026-03-31", dec!(220));
    let b = bridge::bridge(db.conn(), &perf_args("2026-01", "2026-03", None)).unwrap();
    let l = &b.lines;
    assert_eq!(l.contributions, 1_000_000);
    assert_eq!(l.end_value, 1_210_000);
    assert_eq!(l.price_gain, 210_000);
    // USD 200 of price are worth R$ 1.100 at the end rate; the rest (USD 2.000 × 0,50) is FX.
    assert_eq!(l.fx_gain, 100_000);
}

#[test]
fn benchmark_series_follows_asset_index_and_cdi_plus() {
    use super::benchmark::{BenchmarkArgs, BenchmarkTarget, benchmark_series};
    let (_d, mut db) = db();
    let a = asset(&mut db, "ITSA4", AssetClass::StockBr, None);
    price(&db, &a, "2026-01-30", dec!(10));
    price(&db, &a, "2026-02-27", dec!(11));
    price(&db, &a, "2026-03-31", dec!(12));
    crate::indices::upsert_values(
        db.conn(),
        "ibov",
        &[
            (parse_date("2026-01-30").unwrap(), dec!(100000), false),
            (parse_date("2026-02-27").unwrap(), dec!(110000), false),
            (parse_date("2026-03-31").unwrap(), dec!(99000), false),
        ],
        "test",
    )
    .unwrap();
    crate::indices::upsert_values(
        db.conn(),
        "cdi",
        &[
            (parse_date("2026-02-02").unwrap(), dec!(0.1), false),
            (parse_date("2026-03-02").unwrap(), dec!(0.1), false),
        ],
        "test",
    )
    .unwrap();
    let months = vec!["2026-02".to_string(), "2026-03".to_string()];
    let run = |target| {
        benchmark_series(
            db.conn(),
            &BenchmarkArgs {
                months: months.clone(),
                target,
            },
        )
        .unwrap()
    };
    let close = |a: Option<f64>, b: f64| (a.unwrap() - b).abs() < 1e-9;

    // Base at the end of January: the 30/01 close counts for 31/01.
    let s = run(BenchmarkTarget::Asset { id: a.clone() });
    assert_eq!(s.label, "ITSA4");
    assert!(
        close(s.cumulative[0], 0.10) && close(s.cumulative[1], 0.20),
        "{s:?}"
    );

    let s = run(BenchmarkTarget::Index { id: "ibov".into() });
    assert!(
        close(s.cumulative[0], 0.10) && close(s.cumulative[1], -0.01),
        "{s:?}"
    );

    let s = run(BenchmarkTarget::CdiPlus { spread: 0.0 });
    assert!(s.label.starts_with("CDI +"), "{}", s.label);
    assert!(close(s.cumulative[0], 0.001), "{s:?}");
    assert!(close(s.cumulative[1], 1.001f64.powi(2) - 1.0), "{s:?}");
    // The extra enters per business day with a rate: two days, each with 1,1^(1/252).
    let s = run(BenchmarkTarget::CdiPlus { spread: 0.10 });
    let day = 1.001 * 1.1f64.powf(1.0 / 252.0);
    assert!(close(s.cumulative[1], day * day - 1.0), "{s:?}");

    assert!(
        benchmark_series(
            db.conn(),
            &BenchmarkArgs {
                months: months.clone(),
                target: BenchmarkTarget::Index {
                    id: "nao-existe".into()
                },
            },
        )
        .is_err()
    );
}

#[test]
fn contributions_split_the_period_result_by_asset() {
    use super::contrib::{ContributionArgs, contributions};
    let (_d, mut db) = db();
    let a = asset(&mut db, "ITSA4", AssetClass::StockBr, None);
    let b = asset(&mut db, "PETR4", AssetClass::StockBr, None);
    for (id, p) in [(&a, "10"), (&b, "20")] {
        trade(&mut db, id, "xp", "2026-01-02", TradeSide::Buy, "100", p, 0).unwrap();
    }
    for (id, j, f, m) in [
        (&a, dec!(10), dec!(11), dec!(12)),
        (&b, dec!(20), dec!(18), dec!(18)),
    ] {
        price(&db, id, "2026-01-30", j);
        price(&db, id, "2026-02-27", f);
        price(&db, id, "2026-03-31", m);
    }
    // A purchase mid-period enters as money invested, not as result.
    trade(
        &mut db,
        &a,
        "xp",
        "2026-03-31",
        TradeSide::Buy,
        "10",
        "12",
        0,
    )
    .unwrap();
    let r = contributions(
        db.conn(),
        &ContributionArgs {
            from_month: Some("2026-02".into()),
            to_month: Some("2026-03".into()),
        },
    )
    .unwrap();
    assert_eq!(
        (r.from.as_str(), r.to.as_str()),
        ("2026-01-31", "2026-03-31")
    );
    assert_eq!(r.rows.len(), 2);
    assert_eq!(r.rows[0].asset.id, a);
    assert_eq!(r.rows[0].result, 20000, "1.000 → 1.320 with 120 invested");
    assert_eq!(r.rows[1].result, -20000);
    assert_eq!(r.total, 0);
    // The 31/03 purchase weighs zero days: the denominator stays at the initial 3.000.
    assert!((r.rows[0].points.unwrap() - 200.0 / 3000.0).abs() < 1e-9);
    assert!(r.total_points.unwrap().abs() < 1e-9);
}

#[test]
fn scheduled_contributions_wait_for_confirmation() {
    use super::plans::{self, InvestmentPlanInput, PlanConfirmArgs, PlanSkipArgs};
    let (_dir, mut db) = db();
    let petr = asset(&mut db, "PETR4", AssetClass::StockBr, None);
    price(&db, &petr, "2026-08-10", dec!(30));
    price(&db, &petr, "2026-09-09", dec!(32.10));
    let input = InvestmentPlanInput {
        id: None,
        name: "PETR4 todo mês".into(),
        asset_id: petr.clone(),
        account_id: "xp".into(),
        amount: 50_000,
        day_of_month: 10,
        start_date: "2026-08-01".into(),
        end_date: None,
        goal_id: None,
        active: None,
        notes: None,
    };
    let id = record(db.conn_mut(), &Author::user(), "plano", |tx| {
        plans::save(tx, &input)
    })
    .unwrap()
    .value;
    let today = parse_date("2026-10-20").unwrap();
    // Nothing is stored on its own: the three contributions wait for a decision, with the purchase.
    let due = plans::pending(db.conn(), today).unwrap();
    let dates: Vec<&str> = due.iter().map(|d| d.due_date.as_str()).collect();
    assert_eq!(dates, ["2026-08-10", "2026-09-10", "2026-10-10"]);
    assert_eq!(
        (due[0].price.as_deref(), due[0].quantity.as_deref()),
        (Some("30"), Some("16"))
    );
    // No close on the day: the last earlier one (09/09) and the whole quantity.
    assert_eq!(
        (
            due[1].price.as_deref(),
            due[1].price_date.as_deref(),
            due[1].quantity.as_deref()
        ),
        (Some("32.1"), Some("2026-09-09"), Some("15"))
    );
    assert!(
        records::list_trades(db.conn(), &TradeFilter::default())
            .unwrap()
            .is_empty()
    );

    // Confirm stores the purchase; skip removes the date from the list; neither comes back.
    let confirm = |db: &mut Db, due: &str| {
        record(db.conn_mut(), &Author::user(), "aporte", |tx| {
            plans::confirm(
                tx,
                &PlanConfirmArgs {
                    plan_id: id.clone(),
                    due_date: due.into(),
                    quantity: "16".into(),
                    price: "30".into(),
                    date: None,
                    fees: None,
                },
            )
        })
    };
    let trade_id = confirm(&mut db, "2026-08-10").unwrap().value;
    let tr = records::get_trade(db.conn(), &trade_id).unwrap();
    assert_eq!(
        (tr.date.as_str(), tr.quantity.as_str()),
        ("2026-08-10", "16")
    );
    assert!(tr.notes.unwrap().contains("PETR4 todo mês"));
    assert!(confirm(&mut db, "2026-08-10").is_err());
    assert!(confirm(&mut db, "2026-08-11").is_err());
    record(db.conn_mut(), &Author::user(), "pular", |tx| {
        plans::skip(
            tx,
            &PlanSkipArgs {
                plan_id: id.clone(),
                due_date: "2026-09-10".into(),
            },
        )
    })
    .unwrap();
    let left = plans::pending(db.conn(), today).unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].due_date, "2026-10-10");
    let v = &plans::list(db.conn(), today).unwrap()[0];
    assert_eq!((v.pending, v.confirmed), (1, 1));
    assert_eq!(v.next_due.as_deref(), Some("2026-11-10"));

    // Paused: nothing pending; a day outside 1–28 is refused.
    let mut paused = input.clone();
    paused.id = Some(id.clone());
    paused.active = Some(false);
    record(db.conn_mut(), &Author::user(), "pausa", |tx| {
        plans::save(tx, &paused)
    })
    .unwrap();
    assert!(plans::pending(db.conn(), today).unwrap().is_empty());
    let mut bad = input.clone();
    bad.day_of_month = 31;
    assert!(
        record(db.conn_mut(), &Author::user(), "x", |tx| plans::save(
            tx, &bad
        ))
        .is_err()
    );
    // Deleting the plan keeps the confirmed operation.
    record(db.conn_mut(), &Author::user(), "apagar", |tx| {
        plans::delete(tx, &id)
    })
    .unwrap();
    assert!(records::get_trade(db.conn(), &trade_id).is_ok());
}

fn position<'a>(s: &'a PortfolioSummary, ticker: &str) -> &'a PositionView {
    s.positions
        .iter()
        .find(|p| p.asset.ticker.as_deref() == Some(ticker))
        .unwrap()
}

#[test]
fn recent_positions_show_the_period_return_instead_of_an_exploding_xirr() {
    let (_d, mut db) = db();
    // 5 days, −6%: no annualized.
    let a = asset(&mut db, "NOVA3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-10-01",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &a, "2026-10-06", dec!(9.40));
    // 90 days: period highlighted and annualized beside it.
    let b = asset(&mut db, "MEIA3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &b,
        "xp",
        "2026-07-08",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &b, "2026-10-06", dec!(11));
    // 400 days: annualized.
    let c = asset(&mut db, "VELH3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &c,
        "xp",
        "2025-09-01",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &c, "2026-10-06", dec!(12));
    // Closed out and bought again 10 days ago: recent again.
    let r = asset(&mut db, "VOLT3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &r,
        "xp",
        "2025-01-02",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &r,
        "xp",
        "2025-06-02",
        TradeSide::Sell,
        "100",
        "12",
        0,
    )
    .unwrap();
    trade(
        &mut db,
        &r,
        "xp",
        "2026-09-26",
        TradeSide::Buy,
        "50",
        "10",
        0,
    )
    .unwrap();
    price(&db, &r, "2026-10-06", dec!(10.5));
    let s = engine::summary(db.conn(), &args("2026-10-06")).unwrap();
    let n = position(&s, "NOVA3");
    assert_eq!(n.return_basis, ReturnBasis::Period);
    assert_eq!(n.xirr, None);
    assert_eq!(n.xirr_real, None);
    assert_eq!(n.holding_days, Some(5));
    assert!(
        (n.period_return.unwrap() + 0.06).abs() < 1e-9,
        "{:?}",
        n.period_return
    );
    let m = position(&s, "MEIA3");
    assert_eq!(m.return_basis, ReturnBasis::PeriodAndAnnualized);
    assert!(m.xirr.is_some());
    assert!((m.period_return.unwrap() - 0.10).abs() < 1e-9);
    assert_eq!(position(&s, "VELH3").return_basis, ReturnBasis::Annualized);
    let v = position(&s, "VOLT3");
    assert_eq!(v.holding_days, Some(10), "account of the repurchase");
    assert_eq!(v.return_basis, ReturnBasis::Period);
    assert_eq!(v.xirr, None);
    // Single constant for the limit.
    assert_eq!(MIN_DAYS_FOR_ANNUALIZED, 30);
}

#[test]
fn class_average_xirr_ignores_a_one_day_position() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ANTI3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &a,
        "xp",
        "2025-01-02",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &a, "2026-10-06", dec!(13));
    let b = asset(&mut db, "OUTR3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &b,
        "xp",
        "2025-06-02",
        TradeSide::Buy,
        "100",
        "20",
        0,
    )
    .unwrap();
    price(&db, &b, "2026-10-06", dec!(19));
    let class_xirr = |db: &Db| {
        engine::summary(db.conn(), &args("2026-10-06"))
            .unwrap()
            .by_class
            .into_iter()
            .find(|c| c.class == AssetClass::StockBr)
            .unwrap()
            .xirr
            .unwrap()
    };
    let before = class_xirr(&db);
    // A 1-day position with +10% (absurd annualized) does not move the class average.
    let n = asset(&mut db, "ONTM3", AssetClass::StockBr, None);
    trade(
        &mut db,
        &n,
        "xp",
        "2026-10-05",
        TradeSide::Buy,
        "100",
        "10",
        0,
    )
    .unwrap();
    price(&db, &n, "2026-10-06", dec!(11));
    let after = class_xirr(&db);
    assert!((before - after).abs() < 1e-12, "{before} × {after}");
}

/// Portfolio of R$ 100.000 with known weights: A 32%, B 22%, C 18%, D 4%, E 24%.
fn concentrated(db: &mut Db) -> Vec<String> {
    let mut ids = vec![];
    for (t, v) in [
        ("AAAA3", 32),
        ("BBBB3", 22),
        ("CCCC3", 18),
        ("DDDD3", 4),
        ("EEEE3", 24),
    ] {
        let a = asset(db, t, AssetClass::StockBr, None);
        trade(
            db,
            &a,
            "xp",
            "2025-01-02",
            TradeSide::Buy,
            "100",
            &format!("{}", v * 10),
            0,
        )
        .unwrap();
        price(db, &a, "2026-10-06", rust_decimal::Decimal::from(v * 10));
        ids.push(a);
    }
    ids
}

#[test]
fn concentration_alerts_by_asset_group_and_class() {
    let (_d, mut db) = db();
    let ids = concentrated(&mut db);
    let s = engine::summary(db.conn(), &args("2026-10-06")).unwrap();
    assert_eq!(s.market_value, 10_000_000);
    let of = |s: &PortfolioSummary, r: &str| {
        s.concentration_alerts
            .iter()
            .find(|a| a.reference == r)
            .map(|a| (a.severity, a.excess_base))
    };
    // No configuration: 32% ≥ 1,5 × 20% is critical; 22% and 24% warn; 18% and 4% nothing.
    assert_eq!(
        of(&s, &ids[0]),
        Some((concentration::Severity::Critical, 1_200_000)),
        "(0,32 − 0,20) × R$ 100.000"
    );
    assert_eq!(
        of(&s, &ids[1]).map(|x| x.0),
        Some(concentration::Severity::Warning)
    );
    assert_eq!(of(&s, &ids[2]), None);
    assert_eq!(of(&s, &ids[3]), None);
    // No class target, no class alert.
    assert!(
        s.concentration_alerts
            .iter()
            .all(|a| a.kind != concentration::Kind::Class)
    );
    // Group "controlador" with A (32%) and D (4%): 36% against 30%, and A keeps its own alert.
    record(db.conn_mut(), &Author::user(), "etiquetas", |tx| {
        for id in [&ids[0], &ids[3]] {
            crate::assets::labels::set_for_asset(
                tx,
                &crate::assets::labels::SetAssetLabelsArgs {
                    asset_id: id.clone(),
                    labels: vec!["controlador".into()],
                },
            )?;
        }
        crate::assets::labels::update(
            tx,
            &crate::assets::labels::UpdateAssetLabelArgs {
                name: "Controlador".into(),
                is_concentration_group: Some(true),
                concentration_limit_pct: None,
            },
        )
    })
    .unwrap();
    let s = engine::summary(db.conn(), &args("2026-10-06")).unwrap();
    let g = s
        .concentration_alerts
        .iter()
        .find(|a| a.kind == concentration::Kind::Group)
        .unwrap();
    assert!((g.weight - 0.36).abs() < 1e-9 && (g.limit - 0.30).abs() < 1e-9);
    assert_eq!(g.severity, concentration::Severity::Warning);
    assert_eq!(g.assets.len(), 2);
    assert!(of(&s, &ids[0]).is_some());
    // Own limit of 40% on asset A: no individual alert.
    record(db.conn_mut(), &Author::user(), "limite", |tx| {
        concentration::set_limit(
            tx,
            &concentration::SetConcentrationLimitArgs {
                scope: concentration::Scope::Asset,
                asset_id: Some(ids[0].clone()),
                pct: Some(40.0),
            },
        )
    })
    .unwrap();
    let s = engine::summary(db.conn(), &args("2026-10-06")).unwrap();
    assert_eq!(of(&s, &ids[0]), None);
    // With a class target (stocks 60% ± 5%), the class at 100% raises a critical alert.
    record(db.conn_mut(), &Author::user(), "alvo", |tx| {
        records::set_targets(
            tx,
            TargetScope::Class,
            &[TargetInput {
                scope: TargetScope::Class,
                key: "stock_br".into(),
                target_pct: "60".into(),
                tolerance_pct: Some("5".into()),
            }],
        )
    })
    .unwrap();
    let s = engine::summary(db.conn(), &args("2026-10-06")).unwrap();
    let c = s
        .concentration_alerts
        .iter()
        .find(|a| a.kind == concentration::Kind::Class)
        .unwrap();
    assert!((c.limit - 0.65).abs() < 1e-9);
    assert_eq!(c.severity, concentration::Severity::Critical);
}

#[test]
fn rebalance_without_class_targets_never_feeds_an_asset_over_its_limit() {
    let (_d, mut db) = db();
    let ids = concentrated(&mut db);
    let r = engine::rebalance(
        db.conn(),
        &RebalanceArgs {
            contribution: Some(1_000_000),
        },
    )
    .unwrap();
    assert!(r.lines.is_empty(), "no class target");
    let line = |id: &str| r.concentration.iter().find(|l| l.asset.id == id).unwrap();
    // A (32%) and E (24%) and B (22%) are already above 20%: nothing for them, and no selling.
    for id in [&ids[0], &ids[1], &ids[4]] {
        assert_eq!(line(id).suggested, 0, "{id}");
        assert!(line(id).over_limit);
    }
    assert!(r.concentration.iter().all(|l| l.suggested >= 0));
    // The contribution goes to C (18%) and D (4%), without going over 20% after the contribution.
    let total: i64 = r.concentration.iter().map(|l| l.suggested).sum();
    assert_eq!(total, 1_000_000);
    for id in [&ids[2], &ids[3]] {
        assert!(line(id).weight <= 0.20 + 1e-9, "{}", line(id).weight);
    }
}

#[test]
fn rebalance_caps_each_asset_at_its_room_and_reports_what_is_left() {
    let (_d, mut db) = db();
    let ids = concentrated(&mut db);
    record(db.conn_mut(), &Author::user(), "limite", |tx| {
        concentration::set_limit(
            tx,
            &concentration::SetConcentrationLimitArgs {
                scope: concentration::Scope::AssetDefault,
                asset_id: None,
                pct: Some(15.0),
            },
        )
    })
    .unwrap();
    // R$ 100 thousand + R$ 30 thousand: 15% = R$ 19.500. Only C (R$ 18 thousand) and D (R$ 4 th.)
    // have room, R$ 1.500 and R$ 15.500; the rest of the contribution fits nobody.
    let r = engine::rebalance(
        db.conn(),
        &RebalanceArgs {
            contribution: Some(3_000_000),
        },
    )
    .unwrap();
    let line = |id: &str| r.concentration.iter().find(|l| l.asset.id == id).unwrap();
    assert_eq!(line(&ids[2]).suggested, 150_000);
    assert_eq!(line(&ids[3]).suggested, 1_550_000);
    for l in &r.concentration {
        assert!(
            l.suggested <= l.room,
            "{}: {} > {}",
            l.asset.id,
            l.suggested,
            l.room
        );
    }
    assert_eq!(r.unallocated, 1_300_000);
    let total: i64 = r.concentration.iter().map(|l| l.suggested).sum();
    assert_eq!(total + r.unallocated, 3_000_000);
}

#[test]
fn distribute_capped_moves_what_a_cap_cuts_to_who_has_room() {
    // Weights 3:1, caps 100 and 1000: the first stops at 100 and the rest goes to the second.
    assert_eq!(
        engine::distribute_capped_for_tests(1000, &[3, 1], &[100, 1000]),
        [100, 900]
    );
    // Without weight, the room decides; what does not fit is left out.
    assert_eq!(
        engine::distribute_capped_for_tests(500, &[0, 0], &[100, 200]),
        [100, 200]
    );
    assert_eq!(engine::distribute_capped_for_tests(0, &[1], &[10]), [0]);
}

#[test]
fn treasury_and_fgc_covered_fixed_income_are_outside_the_default_limit() {
    let (_d, mut db) = db();
    concentrated(&mut db); // R$ 100 thousand in stocks
    let tesouro = asset(&mut db, "TESOURO-SELIC-2029", AssetClass::TreasuryBr, None);
    trade(
        &mut db,
        &tesouro,
        "xp",
        "2025-01-02",
        TradeSide::Buy,
        "100",
        "4000",
        0,
    )
    .unwrap();
    price(
        &db,
        &tesouro,
        "2026-10-06",
        rust_decimal::Decimal::from(4000),
    );
    let cdb = |db: &mut Db, name: &str, value: &str| {
        let terms = FixedIncomeTerms {
            kind: FixedIncomeKind::Cdb,
            indexer: Indexer::Prefixado,
            rate: "0".into(),
            issue_date: Some("2026-10-01".into()),
            maturity_date: Some("2030-10-01".into()),
            grace_until: None,
            liquidity: Liquidity::Daily,
            ir_exempt: false,
            issuer: Some(format!("Banco {name}")),
        };
        let id = asset(db, name, AssetClass::FixedIncome, Some(terms));
        trade(db, &id, "xp", "2026-10-01", TradeSide::Buy, "1", value, 0).unwrap();
        id
    };
    // Within the FGC guarantee (R$ 240 thousand) and above it (R$ 300 thousand).
    let coberto = cdb(&mut db, "CDB-A", "240000");
    let acima = cdb(&mut db, "CDB-B", "300000");
    let s = engine::summary(db.conn(), &args("2026-10-06")).unwrap();
    let alerted: Vec<&str> = s
        .concentration_alerts
        .iter()
        .filter(|a| a.kind == concentration::Kind::Asset)
        .map(|a| a.reference.as_str())
        .collect();
    assert!(
        !alerted.contains(&tesouro.as_str()),
        "Tesouro Direto: {alerted:?}"
    );
    assert!(
        !alerted.contains(&coberto.as_str()),
        "CDB within the FGC limit"
    );
    assert!(alerted.contains(&acima.as_str()), "CDB above the FGC limit");
    let r = concentration::report(db.conn(), &s).unwrap();
    let item = |id: &str| r.assets.iter().find(|i| i.reference == id).unwrap();
    assert!(item(&tesouro).exempt && item(&coberto).exempt && !item(&acima).exempt);
    assert_eq!(item(&tesouro).limit, 1.0);
    // An own limit applies even to the Treasury bonds.
    record(db.conn_mut(), &Author::user(), "limite", |tx| {
        concentration::set_limit(
            tx,
            &concentration::SetConcentrationLimitArgs {
                scope: concentration::Scope::Asset,
                asset_id: Some(tesouro.clone()),
                pct: Some(30.0),
            },
        )
    })
    .unwrap();
    let s = engine::summary(db.conn(), &args("2026-10-06")).unwrap();
    assert!(
        s.concentration_alerts
            .iter()
            .any(|a| a.reference == tesouro)
    );
}

#[test]
fn trade_summary_totals_both_sides_and_counts_events() {
    let (_d, mut db) = db();
    let fii = asset(&mut db, "ABCD11", AssetClass::Fii, None);
    for (date, side, qty, price, fees) in [
        ("2025-01-10", TradeSide::Buy, "100", "10", 300),
        ("2025-03-10", TradeSide::Buy, "50", "13", 150),
        ("2025-05-10", TradeSide::Sell, "30", "12.5", 90),
    ] {
        trade(&mut db, &fii, "xp", date, side, qty, price, fees).unwrap();
    }
    let s = trade_summary::summary(
        db.conn(),
        &trade_summary::TradeSummaryArgs {
            asset_id: fii.clone(),
            account_id: None,
        },
    )
    .unwrap();
    assert_eq!(s.buys.count, 2);
    assert_eq!(s.buys.quantity, "150");
    assert_eq!(s.buys.gross, 165000);
    assert_eq!(s.buys.fees, 450);
    assert_eq!(s.buys.avg_price.as_deref(), Some("11"));
    assert_eq!(s.sells.count, 1);
    assert_eq!(s.sells.gross, 37500);
    assert_eq!(s.sells.avg_price.as_deref(), Some("12.5"));
    assert_eq!(s.costs, 540);
    assert_eq!(s.volume, 202500);
    assert!((s.cost_share.unwrap() - 540.0 / 202500.0).abs() < 1e-12);
    assert_eq!(s.net_quantity, "120");
    assert_eq!(s.events, 0);
    assert_eq!(s.first_date.as_deref(), Some("2025-01-10"));
    assert_eq!(s.last_date.as_deref(), Some("2025-05-10"));
    assert!(!s.by_value);

    assert!(!s.split_adjusted);

    let action = |db: &mut Db, date: &str, kind, factor: &str, unit_cost: Option<&str>| {
        record(db.conn_mut(), &Author::user(), "event", |tx| {
            records::create_action(
                tx,
                &CorporateActionInput {
                    asset_id: fii.clone(),
                    date: date.into(),
                    kind,
                    factor: Some(factor.into()),
                    unit_cost: unit_cost.map(Into::into),
                    amount: None,
                    new_asset_id: None,
                    notes: None,
                },
            )
        })
        .unwrap();
    };
    let summary = |db: &Db, account_id: Option<&str>| {
        trade_summary::summary(
            db.conn(),
            &trade_summary::TradeSummaryArgs {
                asset_id: fii.clone(),
                account_id: account_id.map(Into::into),
            },
        )
        .unwrap()
    };

    // A later split puts the quantities in today's units; the gross stays.
    action(&mut db, "2025-06-01", CorporateActionKind::Split, "2", None);
    let s = summary(&db, None);
    assert!(s.split_adjusted);
    assert_eq!(s.buys.quantity, "300");
    assert_eq!(s.buys.gross, 165000);
    assert_eq!(s.buys.avg_price.as_deref(), Some("5.5"));
    assert_eq!(s.sells.avg_price.as_deref(), Some("6.25"));
    assert_eq!(s.net_quantity, "240");
    assert_eq!(s.events, 0);

    // A bonus is not a trade: it counts as an event that explains the difference.
    action(
        &mut db,
        "2025-07-01",
        CorporateActionKind::Bonus,
        "0.1",
        Some("5"),
    );
    assert_eq!(summary(&db, None).events, 1);

    // Another account: no trades, but the events are still the asset's.
    let s = summary(&db, Some("avenue"));
    assert_eq!(
        s.buys,
        trade_summary::TradeSideTotals {
            quantity: "0".into(),
            ..Default::default()
        }
    );
    assert_eq!(s.cost_share, None);
    assert_eq!(s.first_date, None);
    assert_eq!(s.events, 1);
}
