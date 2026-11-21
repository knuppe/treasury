use rusqlite::params;

use super::*;
use crate::assets::{self, AssetClass, AssetInput};
use crate::changes::{Author, record};
use crate::portfolio::{TradeInput, TradeSide, records};
use crate::storage::Db;
use crate::vault::KDF_TEST;

fn db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
        crate::household::seed(tx, "Família", "BRL", &["Ana".into(), "Bruno".into()])?;
        crate::categories::seed_defaults(tx)?;
        for (id, name) in [("xp", "XP"), ("btg", "BTG")] {
            tx.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
                 VALUES (?1, ?2, 'brokerage', 'BRL', '2024-01-01', 'x', 'x')",
                params![id, name],
            )?;
        }
        Ok(())
    })
    .unwrap();
    (dir, db)
}

fn asset(db: &mut Db, ticker: &str, class: AssetClass) -> String {
    let input = AssetInput {
        ticker: Some(ticker.into()),
        name: ticker.into(),
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
        fixed_income: None,
        pension: None,
    };
    record(db.conn_mut(), &Author::user(), "a", |tx| {
        assets::create(tx, &input)
    })
    .unwrap()
    .value
}

#[allow(clippy::too_many_arguments)]
fn trade(
    db: &mut Db,
    a: &str,
    acc: &str,
    date: &str,
    side: TradeSide,
    qty: &str,
    price: &str,
    fees: i64,
    irrf: i64,
) {
    let t = TradeInput {
        asset_id: a.into(),
        account_id: acc.into(),
        date: date.into(),
        side,
        quantity: qty.into(),
        price: price.into(),
        fees: Some(fees),
        taxes: Some(irrf),
        fx_rate: None,
        day_trade: None,
        note_number: None,
        notes: None,
        cash_movement: Some(false),
        external_id: None,
    };
    record(db.conn_mut(), &Author::user(), "op", |tx| {
        records::create_trade(tx, &t)
    })
    .unwrap();
}

fn pool(m: &TaxMonth, p: LossPool) -> &PoolMonth {
    m.pools.iter().find(|x| x.pool == p).unwrap()
}

#[test]
fn stock_sales_up_to_20k_are_exempt() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "ITSA4", AssetClass::StockBr);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "1000",
        "10",
        0,
        0,
    );
    trade(
        &mut db,
        &a,
        "xp",
        "2026-02-10",
        TradeSide::Sell,
        "1000",
        "15",
        0,
        0,
    ); // vende 15 mil, ganho 5 mil
    let m = engine::month(db.conn(), None, "2026-02").unwrap();
    assert!(m.stock_exempt);
    assert_eq!(m.stock_sales, 1_500_000);
    assert_eq!(m.categories[0].exempt_gain, 500_000);
    assert_eq!(m.tax, 0);
    assert_eq!(m.status, MonthStatus::NoTax);
}

#[test]
fn stock_sales_above_20k_pay_15_percent_minus_irrf() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "PETR4", AssetClass::StockBr);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "1000",
        "27",
        1000,
        0,
    ); // custo 27.010
    trade(
        &mut db,
        &a,
        "xp",
        "2026-03-10",
        TradeSide::Sell,
        "1000",
        "30",
        1000,
        150,
    ); // líquido 29.990
    let m = engine::month(db.conn(), None, "2026-03").unwrap();
    assert!(!m.stock_exempt);
    // Ganho 29.990 − 27.010 = 2.980 → 15% = 447,00; IRRF retido 1,50 → 445,50.
    assert_eq!(pool(&m, LossPool::Common).base, 298_000);
    assert_eq!(m.tax, 44_700);
    assert_eq!(m.irrf, 150);
    assert_eq!(m.due, 44_550);
    assert_eq!(m.darf_amount, 44_550);
    assert_eq!(m.status, MonthStatus::ToPay);
    assert_eq!(m.revenue_code, "6015");
    assert_eq!(m.due_date, "2026-04-30");
}

#[test]
fn losses_carry_and_compensate_and_exempt_month_loss_counts() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "VALE3", AssetClass::StockBr);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "2000",
        "60",
        0,
        0,
    );
    // Fevereiro: vende 200 a 50 = 10 mil (isento), prejuízo de 2 mil continua compensável.
    trade(
        &mut db,
        &a,
        "xp",
        "2026-02-10",
        TradeSide::Sell,
        "200",
        "50",
        0,
        0,
    );
    // Março: vende 500 a 70 = 35 mil, ganho de 5 mil; compensa os 2 mil → base 3 mil.
    trade(
        &mut db,
        &a,
        "xp",
        "2026-03-10",
        TradeSide::Sell,
        "500",
        "70",
        0,
        0,
    );
    let feb = engine::month(db.conn(), None, "2026-02").unwrap();
    assert_eq!(pool(&feb, LossPool::Common).loss_after, 200_000);
    let mar = engine::month(db.conn(), None, "2026-03").unwrap();
    let p = pool(&mar, LossPool::Common);
    assert_eq!(
        (p.loss_before, p.loss_used, p.base, p.loss_after),
        (200_000, 200_000, 300_000, 0)
    );
    assert_eq!(mar.tax, 45_000);
    assert!(mar.irrf_estimated, "sem IRRF informado, estima 0,005%");
    assert_eq!(mar.irrf, 175); // 0,005% de 35.000 = 1,75 (acima de R$ 1, retido)
    assert_eq!(feb.irrf, 0, "0,005% de 10 mil = 0,50: retenção dispensada");
}

#[test]
fn fii_losses_do_not_offset_stock_gains_and_fii_pays_20() {
    let (_d, mut db) = db();
    let f = asset(&mut db, "HGLG11", AssetClass::Fii);
    let s = asset(&mut db, "BBDC4", AssetClass::StockBr);
    trade(
        &mut db,
        &f,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "100",
        "160",
        0,
        0,
    );
    trade(
        &mut db,
        &f,
        "xp",
        "2026-02-10",
        TradeSide::Sell,
        "50",
        "150",
        0,
        0,
    ); // −500
    trade(
        &mut db,
        &s,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "2000",
        "12",
        0,
        0,
    );
    trade(
        &mut db,
        &s,
        "xp",
        "2026-02-10",
        TradeSide::Sell,
        "2000",
        "13",
        0,
        0,
    ); // vende 26 mil, +2.000
    trade(
        &mut db,
        &f,
        "xp",
        "2026-03-10",
        TradeSide::Sell,
        "50",
        "180",
        0,
        0,
    ); // +1.000, compensa 500
    let feb = engine::month(db.conn(), None, "2026-02").unwrap();
    assert_eq!(pool(&feb, LossPool::Common).tax, 30_000);
    assert_eq!(pool(&feb, LossPool::Fii).loss_after, 50_000);
    let mar = engine::month(db.conn(), None, "2026-03").unwrap();
    let p = pool(&mar, LossPool::Fii);
    assert_eq!((p.base, p.tax), (50_000, 10_000)); // 20% de 500
}

#[test]
fn day_trade_is_per_broker_and_pays_20_percent() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "MGLU3", AssetClass::StockBr);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-05-04",
        TradeSide::Buy,
        "1000",
        "10",
        0,
        0,
    );
    trade(
        &mut db,
        &a,
        "xp",
        "2026-05-04",
        TradeSide::Sell,
        "1000",
        "10.60",
        0,
        0,
    ); // day trade +600
    // Compra numa corretora e venda em outra no mesmo dia não é day trade.
    trade(
        &mut db,
        &a,
        "btg",
        "2026-05-05",
        TradeSide::Buy,
        "100",
        "10",
        0,
        0,
    );
    trade(
        &mut db,
        &a,
        "xp",
        "2026-05-05",
        TradeSide::Buy,
        "100",
        "10",
        0,
        0,
    );
    trade(
        &mut db,
        &a,
        "btg",
        "2026-05-06",
        TradeSide::Sell,
        "100",
        "11",
        0,
        0,
    );
    let m = engine::month(db.conn(), None, "2026-05").unwrap();
    let dt = pool(&m, LossPool::DayTrade);
    assert_eq!((dt.base, dt.tax), (60_000, 12_000));
    assert_eq!(
        m.realized
            .iter()
            .filter(|r| r.category == TaxCategory::DayTrade)
            .count(),
        1
    );
    // Operação comum de 1.100 em vendas (isenta): sem imposto comum.
    assert_eq!(pool(&m, LossPool::Common).tax, 0);
    // IRRF estimado de 1% do ganho em day trade = 6,00.
    assert_eq!(m.irrf, 600);
    assert_eq!(m.due, 11_400);
}

#[test]
fn darf_below_minimum_accumulates_and_irrf_carries_within_year() {
    let (_d, mut db) = db();
    let e = asset(&mut db, "BOVA11", AssetClass::EtfBr);
    trade(
        &mut db,
        &e,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "100",
        "100",
        0,
        0,
    );
    // ETF não tem isenção: ganho de 50 → 15% = 7,50 (abaixo de R$ 10).
    trade(
        &mut db,
        &e,
        "xp",
        "2026-02-10",
        TradeSide::Sell,
        "10",
        "105",
        0,
        0,
    );
    // Março: ganho de 50 → 7,50 + 7,50 acumulado − IRRF ≥ R$ 10 → paga.
    trade(
        &mut db,
        &e,
        "xp",
        "2026-03-10",
        TradeSide::Sell,
        "10",
        "105",
        0,
        0,
    );
    let feb = engine::month(db.conn(), None, "2026-02").unwrap();
    assert_eq!(feb.status, MonthStatus::BelowMinimum);
    assert!(feb.carried_out > 0 && feb.darf_amount == 0);
    let mar = engine::month(db.conn(), None, "2026-03").unwrap();
    assert_eq!(mar.carried_in, feb.carried_out);
    assert_eq!(mar.darf_amount, mar.due + feb.carried_out);
    assert_eq!(mar.status, MonthStatus::ToPay);
}

#[test]
fn darf_lifecycle_and_year_report() {
    use crate::api::Api;
    use crate::paths::TreasuryPaths;
    use serde_json::json;
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
            json!({"name": "XP", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let cc = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let a = api
        .call(
            "assets.create",
            json!({"ticker": "WEGE3", "name": "WEG", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    for (date, side, qty, price) in [
        ("2026-01-05", "buy", "1000", "40"),
        ("2026-06-10", "sell", "1000", "45"),
    ] {
        api.call(
            "portfolio.create_trade",
            json!({"asset_id": a, "account_id": acc, "date": date, "side": side, "quantity": qty, "price": price, "cash_movement": false}),
        )
        .unwrap();
    }
    api.call("portfolio.create_income", json!({"asset_id": a, "account_id": acc, "kind": "dividend", "pay_date": "2026-08-15", "gross": 12345})).unwrap();
    let m = api.call("tax.month", json!({"month": "2026-06"})).unwrap();
    assert_eq!(m["tax"], 75_000); // 5 mil × 15%
    let d = api
        .call("tax.generate_darf", json!({"month": "2026-06"}))
        .unwrap();
    assert_eq!(d["status"], "pending");
    assert!(
        api.call("tax.generate_darf", json!({"month": "2026-06"}))
            .is_err(),
        "um DARF por mês"
    );
    let paid = api
        .call(
            "tax.mark_darf_paid",
            json!({"id": d["id"], "paid_at": "2026-07-20", "from_account_id": cc}),
        )
        .unwrap();
    assert_eq!(paid["status"], "paid");
    assert!(paid["txn_id"].is_string());
    let m = api.call("tax.month", json!({"month": "2026-06"})).unwrap();
    assert_eq!(m["status"], "paid");
    // MCP somente leitura não gera DARF.
    let mcp = crate::Actor::Mcp {
        client: "x".into(),
        read_only: false,
    };
    assert!(
        api.call_as(&mcp, "tax.generate_darf", json!({"month": "2026-06"}))
            .is_err()
    );

    let y = api.call("tax.year", json!({"year": 2026})).unwrap();
    assert_eq!(y["total_tax"], 75_000);
    assert_eq!(y["dividends"][0]["amount"], 12_345);
    assert!(
        y["holdings"].as_array().unwrap().is_empty(),
        "comprado e vendido no mesmo ano: sem saldo a declarar"
    );
    let y25 = api.call("tax.year", json!({"year": 2025})).unwrap();
    assert_eq!(y25["total_tax"], 0);
}

#[test]
fn year_report_lists_holdings_with_previous_and_current_cost() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "TAEE11", AssetClass::StockBr);
    let f = asset(&mut db, "KNRI11", AssetClass::Fii);
    trade(
        &mut db,
        &a,
        "xp",
        "2025-03-05",
        TradeSide::Buy,
        "100",
        "35",
        0,
        0,
    );
    trade(
        &mut db,
        &a,
        "xp",
        "2026-03-05",
        TradeSide::Buy,
        "100",
        "37",
        0,
        0,
    );
    trade(
        &mut db,
        &f,
        "xp",
        "2026-04-05",
        TradeSide::Buy,
        "10",
        "150",
        0,
        0,
    );
    let y = engine::year(db.conn(), None, 2026).unwrap();
    let t = y
        .holdings
        .iter()
        .find(|h| h.ticker.as_deref() == Some("TAEE11"))
        .unwrap();
    assert_eq!(
        (
            t.quantity_prev.as_str(),
            t.cost_prev,
            t.quantity.as_str(),
            t.cost
        ),
        ("100", 350_000, "200", 720_000)
    );
    assert_eq!(t.group_code, "03.01");
    let k = y
        .holdings
        .iter()
        .find(|h| h.ticker.as_deref() == Some("KNRI11"))
        .unwrap();
    assert_eq!(
        (k.group_code.as_str(), k.cost_prev, k.cost),
        ("07.03", 0, 150_000)
    );
    assert!(y.disclaimer.contains("Receita"));
}

/// Ações de outras bolsas entram como aplicação no exterior (Lei 14.754/2023), com o país da
/// bolsa em Bens e Direitos; não entram na apuração mensal. EUA continua como antes.
#[test]
fn other_exchanges_are_foreign_with_the_exchange_country() {
    let (_d, mut db) = db();
    let mk = |db: &mut Db, ticker: &str, class: AssetClass, market: &str| {
        let input: AssetInput = serde_json::from_value(serde_json::json!({
            "ticker": ticker, "name": ticker, "class": class, "market": market
        }))
        .unwrap();
        record(db.conn_mut(), &Author::user(), "a", |tx| {
            assets::create(tx, &input)
        })
        .unwrap()
        .value
    };
    let vod = mk(&mut db, "VOD", AssetClass::StockIntl, "LSE");
    let aapl = mk(&mut db, "AAPL", AssetClass::StockUs, "NASDAQ");
    for (a, fx) in [(&vod, "7"), (&aapl, "5")] {
        for (date, side, qty, price) in [
            ("2026-02-02", TradeSide::Buy, "100", "2"),
            ("2026-06-02", TradeSide::Sell, "50", "3"),
        ] {
            let t = TradeInput {
                asset_id: a.clone(),
                account_id: "xp".into(),
                date: date.into(),
                side,
                quantity: qty.into(),
                price: price.into(),
                fees: Some(0),
                taxes: Some(0),
                fx_rate: Some(fx.into()),
                day_trade: None,
                note_number: None,
                notes: None,
                cash_movement: Some(false),
                external_id: None,
            };
            record(db.conn_mut(), &Author::user(), "op", |tx| {
                records::create_trade(tx, &t)
            })
            .unwrap();
        }
    }
    let y = engine::year(db.conn(), None, 2026).unwrap();
    // Ganho: 50 × (3 − 2) × câmbio → VOD 350, AAPL 250 (BRL).
    assert_eq!(y.foreign.realized, 60_000);
    let h = |t: &str| {
        y.holdings
            .iter()
            .find(|h| h.ticker.as_deref() == Some(t))
            .unwrap()
    };
    assert_eq!(
        (h("VOD").country.as_str(), h("VOD").group_code.as_str()),
        ("Reino Unido", "03.01")
    );
    assert_eq!(h("AAPL").country, "Estados Unidos");
    // Nada na apuração mensal de renda variável.
    assert_eq!(y.total_tax, 0);
    let jun = engine::month(db.conn(), None, "2026-06").unwrap();
    assert!(jun.realized.is_empty(), "{:?}", jun.realized);
}

#[test]
fn notes_and_labels_follow_the_locale() {
    use crate::i18n::{Locale, set_current};
    let (_d, mut db) = db();
    let a = asset(&mut db, "ITSA4", AssetClass::StockBr);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-01-05",
        TradeSide::Buy,
        "1000",
        "10",
        0,
        0,
    );
    trade(
        &mut db,
        &a,
        "xp",
        "2026-02-10",
        TradeSide::Sell,
        "1000",
        "15",
        0,
        0,
    );
    let pt = engine::month(db.conn(), None, "2026-02").unwrap();
    set_current(Locale::En);
    let en = engine::month(db.conn(), None, "2026-02").unwrap();
    let year = engine::year(db.conn(), None, 2026).unwrap();
    set_current(Locale::PtBr);
    assert_eq!(
        pt.notes[0],
        "Vendas de ações no mês de R$ 15.000,00 (até R$ 20 mil): ganho isento; prejuízo continua compensável."
    );
    assert_eq!(
        en.notes[0],
        "Stock sales in the month of R$15,000.00 (up to R$ 20,000): exempt gain; a loss can still be offset."
    );
    assert_eq!(en.categories[0].label, "Stocks (regular trades)");
    assert!(
        year.disclaimer.starts_with("A helper calculation"),
        "{}",
        year.disclaimer
    );
}

/// Códigos de Bens e Direitos pela tabela do programa IRPF 2026 (docs/brazil/income-tax-validation.md, E6):
/// poupança 04.01, isentos 04.03, tributados 04.02, ETF 07.06, Bitcoin 08.01, altcoins 08.02,
/// stablecoins 08.03, outros 99.99.
#[test]
fn bens_e_direitos_codes_follow_the_irpf_2026_table() {
    let (_d, mut db) = db();
    let fixed = |db: &mut Db, ticker: &str, kind: crate::assets::FixedIncomeKind| {
        let input = AssetInput {
            ticker: Some(ticker.into()),
            name: ticker.into(),
            official_name: None,
            account_id: None,
            class: AssetClass::FixedIncome,
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
            fixed_income: Some(crate::assets::FixedIncomeTerms {
                kind,
                indexer: crate::assets::Indexer::Cdi,
                rate: "100".into(),
                issue_date: Some("2026-01-02".into()),
                maturity_date: Some("2030-01-02".into()),
                grace_until: None,
                liquidity: crate::assets::Liquidity::Daily,
                ir_exempt: kind.default_ir_exempt(),
                issuer: None,
            }),
            pension: None,
        };
        record(db.conn_mut(), &Author::user(), "a", |tx| {
            assets::create(tx, &input)
        })
        .unwrap()
        .value
    };
    use crate::assets::FixedIncomeKind as K;
    let ids = [
        fixed(&mut db, "POUP", K::Poupanca),
        fixed(&mut db, "LCI1", K::Lci),
        fixed(&mut db, "CRA1", K::Cra),
        fixed(&mut db, "CDB1", K::Cdb),
        asset(&mut db, "BOVA11", AssetClass::EtfBr),
        asset(&mut db, "BTC", AssetClass::Crypto),
        asset(&mut db, "ETH", AssetClass::Crypto),
        asset(&mut db, "USDT", AssetClass::Crypto),
        asset(&mut db, "OBRA", AssetClass::Other),
    ];
    for id in &ids {
        trade(
            &mut db,
            id,
            "xp",
            "2026-02-02",
            TradeSide::Buy,
            "1",
            "1000",
            0,
            0,
        );
    }
    let y = engine::year(db.conn(), None, 2026).unwrap();
    let code = |t: &str| {
        y.holdings
            .iter()
            .find(|h| h.ticker.as_deref() == Some(t))
            .unwrap()
            .group_code
            .clone()
    };
    for (t, c) in [
        ("POUP", "04.01"),
        ("LCI1", "04.03"),
        ("CRA1", "04.03"),
        ("CDB1", "04.02"),
        ("BOVA11", "07.06"),
        ("BTC", "08.01"),
        ("ETH", "08.02"),
        ("USDT", "08.03"),
        ("OBRA", "99.99"),
    ] {
        assert_eq!(code(t), c, "{t}");
    }
}

/// Exterior pela Lei 14.754/2023: 15% sobre o rendimento bruto (art. 2º §1º, "não será aplicada
/// nenhuma dedução"), crédito do imposto pago lá fora até o IR brasileiro daquele rendimento
/// (art. 4º §1º; Perguntas e Respostas IRPF 2026, perguntas 140 e 142: EUA por reciprocidade)
/// e prejuízo compensado nos anos seguintes (art. 9º §2º; pergunta 476).
#[test]
fn foreign_income_is_gross_with_credit_and_losses_carry_forward() {
    let (_d, mut db) = db();
    let aapl = asset(&mut db, "AAPL", AssetClass::StockUs);
    let op = |db: &mut Db, date: &str, side: TradeSide, qty: &str, price: &str| {
        let t = TradeInput {
            asset_id: aapl.clone(),
            account_id: "xp".into(),
            date: date.into(),
            side,
            quantity: qty.into(),
            price: price.into(),
            fees: Some(0),
            taxes: Some(0),
            fx_rate: Some("5".into()),
            day_trade: None,
            note_number: None,
            notes: None,
            cash_movement: Some(false),
            external_id: None,
        };
        record(db.conn_mut(), &Author::user(), "op", |tx| {
            records::create_trade(tx, &t)
        })
        .unwrap();
    };
    // 2025: compra 300 a US$ 10 e vende 100 a US$ 8 → prejuízo de US$ 200 = R$ 1.000.
    op(&mut db, "2025-02-03", TradeSide::Buy, "300", "10");
    op(&mut db, "2025-08-04", TradeSide::Sell, "100", "8");
    // 2026: vende 100 a US$ 11,20 → ganho de US$ 120 = R$ 600.
    op(&mut db, "2026-05-04", TradeSide::Sell, "100", "11.2");
    // Dividendo de US$ 100 brutos com 30% retidos nos EUA (câmbio 5).
    let div = crate::portfolio::IncomeInput {
        asset_id: aapl.clone(),
        account_id: "xp".into(),
        kind: crate::portfolio::IncomeKind::Dividend,
        ex_date: None,
        pay_date: "2026-06-10".into(),
        quantity: None,
        gross: 10_000,
        tax_withheld: Some(3_000),
        fx_rate: Some("5".into()),
        status: None,
        cash_movement: Some(false),
    };
    record(db.conn_mut(), &Author::user(), "div", |tx| {
        records::create_income(tx, &div)
    })
    .unwrap();

    let y25 = engine::year(db.conn(), None, 2025).unwrap();
    assert_eq!(y25.foreign.realized, -100_000);
    assert_eq!(y25.foreign.loss_carried_out, 100_000);
    assert_eq!(y25.foreign.estimated_tax, 0);

    let f = engine::year(db.conn(), None, 2026).unwrap().foreign;
    assert_eq!(f.realized, 60_000);
    assert_eq!((f.loss_carried_in, f.loss_carried_out), (100_000, 40_000));
    // Bruto R$ 500, R$ 150 pagos lá fora; o crédito para nos R$ 75 do IR brasileiro.
    assert_eq!(
        (f.income, f.tax_paid_abroad, f.tax_credit),
        (50_000, 15_000, 7_500)
    );
    // Ganho todo compensado pelo prejuízo; 15% × 500 − 75 = 0.
    assert_eq!(f.estimated_tax, 0);
}

/// Cripto em corretora no Brasil: ganho de capital mensal, isento quando o total vendido em
/// cripto no mês (Brasil e exterior) não passa de R$ 35 mil; acima, 15% do ganho em DARF 4600
/// até o último dia útil do mês seguinte. Fontes: Lei 9.250/1995 art. 22, II; Lei 8.981/1995
/// art. 21; Perguntas e Respostas IRPF 2026, pergunta 653. Cripto em conta fora do real vai
/// para o resumo do exterior (Lei 14.754/2023 art. 3º §1º).
#[test]
fn crypto_in_brazil_is_monthly_capital_gain_with_the_35k_exemption() {
    let (_d, mut db) = db();
    db.conn()
        .execute(
            "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
             VALUES ('bin', 'Exchange', 'brokerage', 'USD', '2024-01-01', 'x', 'x')",
            [],
        )
        .unwrap();
    let input: AssetInput = serde_json::from_value(serde_json::json!({
        "ticker": "BTC", "name": "Bitcoin", "class": "crypto", "currency": "BRL"
    }))
    .unwrap();
    let btc = record(db.conn_mut(), &Author::user(), "a", |tx| {
        assets::create(tx, &input)
    })
    .unwrap()
    .value;
    let op = |db: &mut Db, acc: &str, date: &str, side: TradeSide, qty: &str, price: &str| {
        trade(db, &btc, acc, date, side, qty, price, 0, 0);
    };
    op(&mut db, "xp", "2026-02-02", TradeSide::Buy, "1", "300000");
    op(&mut db, "bin", "2026-02-02", TradeSide::Buy, "1", "300000");
    // Março: R$ 34.000 vendidos, ganho de R$ 4.000, isento.
    op(
        &mut db,
        "xp",
        "2026-03-10",
        TradeSide::Sell,
        "0.1",
        "340000",
    );
    // Abril: R$ 70.000 vendidos, ganho de R$ 10.000 → 15% = R$ 1.500.
    op(
        &mut db,
        "xp",
        "2026-04-10",
        TradeSide::Sell,
        "0.2",
        "350000",
    );
    // Maio: R$ 20.000 no Brasil + R$ 20.000 no exterior → passa de R$ 35 mil; o ganho do
    // Brasil (R$ 1.250) paga 15%; o do exterior fica para o ajuste anual.
    op(
        &mut db,
        "xp",
        "2026-05-11",
        TradeSide::Sell,
        "0.0625",
        "320000",
    );
    op(
        &mut db,
        "bin",
        "2026-05-11",
        TradeSide::Sell,
        "0.0625",
        "320000",
    );

    let c = |m: &str| engine::month(db.conn(), None, m).unwrap().crypto.unwrap();
    let mar = c("2026-03");
    assert_eq!(
        (mar.sales, mar.gain, mar.exempt, mar.tax),
        (3_400_000, 400_000, true, 0)
    );
    let abr = c("2026-04");
    assert_eq!(
        (abr.sales, abr.gain, abr.exempt),
        (7_000_000, 1_000_000, false)
    );
    assert_eq!((abr.tax, abr.darf_amount), (150_000, 150_000));
    assert_eq!(abr.revenue_code, "4600");
    assert_eq!(abr.due_date, "2026-05-29");
    let mai = c("2026-05");
    assert_eq!((mai.sales, mai.sales_brazil), (4_000_000, 2_000_000));
    assert_eq!((mai.gain, mai.exempt, mai.tax), (125_000, false, 18_750));
    // Nada disso entra no DARF 6015 de renda variável.
    assert_eq!(engine::month(db.conn(), None, "2026-04").unwrap().tax, 0);
    // No exterior, só a venda da conta em dólar: 0,0625 × (320.000 − 300.000) = R$ 1.250.
    let y = engine::year(db.conn(), None, 2026).unwrap();
    assert_eq!(y.foreign.realized, 125_000);
}

/// DARF 4600 de cripto entra na lista ao lado do 6015 (o índice único é por código) e a conta
/// pode ser marcada como exterior (ou Brasil), o que tira as vendas dela do ganho mensal.
#[test]
fn crypto_darf_4600_and_custody_choice_per_account() {
    use crate::api::Api;
    use crate::paths::TreasuryPaths;
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let id = |v: serde_json::Value| v["id"].as_str().unwrap().to_string();
    let acc = id(api
        .call(
            "accounts.create",
            json!({"name": "Corretora cripto", "kind": "brokerage"}),
        )
        .unwrap());
    let btc = id(api
        .call(
            "assets.create",
            json!({"ticker": "BTC", "name": "Bitcoin", "class": "crypto", "currency": "BRL"}),
        )
        .unwrap());
    for (date, side, qty, price) in [
        ("2026-02-02", "buy", "1", "300000"),
        ("2026-04-10", "sell", "0.2", "350000"),
    ] {
        api.call(
            "portfolio.create_trade",
            json!({"asset_id": btc, "account_id": acc, "date": date, "side": side, "quantity": qty, "price": price, "cash_movement": false}),
        )
        .unwrap();
    }
    let custody = api.call("tax.crypto_custody", json!({})).unwrap();
    let mine = custody
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["account_id"] == acc.as_str())
        .unwrap();
    assert_eq!(
        (mine["abroad"].as_bool(), mine["chosen"].as_bool()),
        (Some(false), Some(false))
    );
    assert_eq!(mine["has_crypto"], true);

    // DARF 4600 de abril (R$ 1.500); o 6015 do mês não tem imposto.
    assert!(
        api.call("tax.generate_darf", json!({"month": "2026-04"}))
            .is_err()
    );
    let d = api
        .call(
            "tax.generate_darf",
            json!({"month": "2026-04", "revenue_code": "4600"}),
        )
        .unwrap();
    assert_eq!(
        (d["revenue_code"].as_str(), d["total"].as_i64()),
        (Some("4600"), Some(150_000))
    );
    let m = api.call("tax.month", json!({"month": "2026-04"})).unwrap();
    assert_eq!(m["crypto"]["darf_id"], d["id"]);
    assert_eq!(m["darf_id"], serde_json::Value::Null);
    assert!(
        api.call(
            "tax.generate_darf",
            json!({"month": "2026-04", "revenue_code": "4600"})
        )
        .is_err(),
        "um DARF 4600 por mês"
    );

    // Pago 5 dias depois do vencimento (29/05): multa de 1,65% e juros de 1% no total.
    let charges = api
        .call(
            "tax.darf_charges",
            json!({"id": d["id"], "paid_at": "2026-06-03"}),
        )
        .unwrap();
    assert_eq!(
        (charges["fine"].as_i64(), charges["interest"].as_i64()),
        (Some(2_475), Some(1_500))
    );
    let paid = api
        .call(
            "tax.mark_darf_paid",
            json!({"id": d["id"], "paid_at": "2026-06-03"}),
        )
        .unwrap();
    assert_eq!(
        (paid["fine"].as_i64(), paid["total"].as_i64()),
        (Some(2_475), Some(153_975))
    );

    // Marcada como exterior: a venda sai do ganho mensal e vai para o resumo anual.
    api.call(
        "tax.set_crypto_custody",
        json!({"account_id": acc, "abroad": true}),
    )
    .unwrap();
    let m = api.call("tax.month", json!({"month": "2026-04"})).unwrap();
    assert_eq!(
        (
            m["crypto"]["sales_brazil"].as_i64(),
            m["crypto"]["gain"].as_i64()
        ),
        (Some(0), Some(0))
    );
    let y = api.call("tax.year", json!({"year": 2026})).unwrap();
    assert_eq!(y["foreign"]["realized"], 1_000_000);
    // De volta ao automático (conta em reais = Brasil).
    api.call(
        "tax.set_crypto_custody",
        json!({"account_id": acc, "abroad": null}),
    )
    .unwrap();
    let m = api.call("tax.month", json!({"month": "2026-04"})).unwrap();
    assert_eq!(m["crypto"]["gain"], 1_000_000);
}

/// Venda maior que a posição registrada (falta histórico de compras): o excedente fica fora
/// da apuração, mas o mês avisa em vez de sumir em silêncio.
#[test]
fn selling_more_than_the_position_is_flagged_in_the_month() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "PETR4", AssetClass::StockBr);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-03-02",
        TradeSide::Buy,
        "100",
        "30",
        0,
        0,
    );
    trade(
        &mut db,
        &a,
        "xp",
        "2026-04-06",
        TradeSide::Sell,
        "100",
        "35",
        0,
        0,
    );
    // A interface recusa vender mais do que há; aqui a compra é editada depois (como numa
    // importação parcial), e sobram 40 sem custo.
    db.conn()
        .execute("UPDATE trade SET quantity = '60' WHERE side = 'buy'", [])
        .unwrap();
    let m = engine::month(db.conn(), None, "2026-04").unwrap();
    assert_eq!(m.stock_sales, 210_000, "só as 60 com custo entram");
    assert!(
        m.notes
            .iter()
            .any(|n| n.contains("PETR4") && n.contains("40")),
        "{:?}",
        m.notes
    );
    let mar = engine::month(db.conn(), None, "2026-03").unwrap();
    assert!(mar.notes.iter().all(|n| !n.contains("PETR4")));
}

/// IRRF que sobra no ano: o "dedo-duro" de operações comuns (Lei 11.033/2004) vai para a
/// declaração de ajuste (art. 2º §7º, III; Perguntas e Respostas IRPF 2026, pergunta 706); o
/// de day trade só compensa até dezembro e depois só por restituição (perguntas 714 e 715).
/// Os dois ficam separados no relatório anual.
#[test]
fn leftover_withholding_splits_lei_11033_from_day_trade_at_year_end() {
    let (_d, mut db) = db();
    let a = asset(&mut db, "VALE3", AssetClass::StockBr);
    let b = asset(&mut db, "ITUB4", AssetClass::StockBr);
    trade(
        &mut db,
        &a,
        "xp",
        "2026-02-02",
        TradeSide::Buy,
        "1000",
        "60",
        0,
        0,
    );
    // Venda comum com prejuízo e R$ 3,00 de IRRF retido na nota.
    trade(
        &mut db,
        &a,
        "xp",
        "2026-03-02",
        TradeSide::Sell,
        "1000",
        "55",
        0,
        300,
    );
    // Day trade com prejuízo e R$ 2,00 de IRRF retido.
    trade(
        &mut db,
        &b,
        "btg",
        "2026-04-06",
        TradeSide::Buy,
        "100",
        "30",
        0,
        0,
    );
    trade(
        &mut db,
        &b,
        "btg",
        "2026-04-06",
        TradeSide::Sell,
        "100",
        "29",
        0,
        200,
    );
    let y = engine::year(db.conn(), None, 2026).unwrap();
    assert_eq!((y.irrf_lei_11033_left, y.irrf_day_trade_left), (300, 200));
    // No ano seguinte, nada passa para a apuração mensal.
    let jan = engine::month(db.conn(), None, "2027-01").unwrap();
    assert_eq!(jan.irrf_carried_in, 0);
}

/// Prévia da declaração anual a partir do cofre: holerites do ano (sem 13º e PLR), parcela
/// isenta de aposentadoria a partir dos 65 anos, saúde e instrução dedutíveis pelas
/// categorias (farmácia e idiomas ficam de fora) e o cálculo dos dois modelos.
#[test]
fn annual_draft_reads_payslips_and_deductible_expenses() {
    let (_d, mut db) = db();
    let ana: String = db
        .conn()
        .query_row("SELECT id FROM member WHERE name = 'Ana'", [], |r| r.get(0))
        .unwrap();
    let conn = db.conn();
    conn.execute(
        "UPDATE member SET birth_date = '1960-06-15' WHERE id = ?1",
        [&ana],
    )
    .unwrap();
    for (id, kind) in [("emp", "clt"), ("inss", "retirement")] {
        conn.execute(
            "INSERT INTO income_source (id, member_id, name, kind, dependents, created_at, updated_at)
             VALUES (?1, ?2, ?1, ?3, 1, 'x', 'x')",
            params![id, ana, kind],
        )
        .unwrap();
    }
    for m in 1..=12 {
        let month = format!("2025-{m:02}");
        conn.execute(
            "INSERT INTO payslip (id, source_id, reference_month, kind, gross, social_security, income_tax, net, created_at, updated_at)
             VALUES (?1, 'emp', ?2, 'monthly', 800000, 80000, 90000, 630000, 'x', 'x')",
            params![format!("p{m}"), month],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO payslip (id, source_id, reference_month, kind, gross, social_security, income_tax, net, created_at, updated_at)
             VALUES (?1, 'inss', ?2, 'monthly', 300000, 0, 0, 300000, 'x', 'x')",
            params![format!("a{m}"), month],
        )
        .unwrap();
    }
    // 13º e PLR ficam fora (tributação exclusiva).
    conn.execute(
        "INSERT INTO payslip (id, source_id, reference_month, kind, gross, social_security, income_tax, net, created_at, updated_at)
         VALUES ('t13', 'emp', '2025-12', 'thirteenth_2', 800000, 80000, 90000, 630000, 'x', 'x')",
        [],
    )
    .unwrap();
    let cat = |db: &Db, path: &str| {
        crate::categories::find_br_path(
            db.conn(),
            Some(crate::categories::CategoryKind::Expense),
            path,
        )
        .unwrap()
        .unwrap()
    };
    let sep = crate::categories::PATH_SEP;
    for (path, amount) in [
        (format!("Saúde{sep}Plano de saúde"), 600_000),
        (format!("Saúde{sep}Farmácia"), 200_000),
        (format!("Educação{sep}Escola"), 500_000),
        (format!("Educação{sep}Idiomas"), 300_000),
    ] {
        let id = cat(&db, &path);
        record(db.conn_mut(), &Author::user(), "t", |tx| {
            let mut n = crate::transactions::NewTxn::new(
                "xp",
                "2025-05-10",
                -amount,
                "BRL",
                crate::transactions::TxnKind::Expense,
                "despesa",
            );
            n.category_id = Some(id.clone());
            crate::transactions::insert(tx, &n)
        })
        .unwrap();
    }
    let d = super::annual_draft::draft(db.conn(), 2025, None).unwrap();
    // Salário 96 mil + aposentadoria: 5 meses antes dos 65 (15 mil) e 7 a partir do mês do
    // aniversário (junho), com R$ 1.903,98 isentos por mês (7 × 1.096,02 = 7.672,14).
    let employment: i64 = d.input.employment.iter().map(|e| e.income).sum();
    assert_eq!(employment, 9_600_000 + 1_500_000 + 767_214);
    assert_eq!(d.input.dependents, 1);
    assert_eq!(d.input.medical, 600_000);
    assert_eq!(d.input.education, vec![500_000]);
    assert_eq!(d.result.taxable_income, employment);
    let paid: i64 = d.input.employment.iter().map(|e| e.withheld).sum();
    assert_eq!(paid, 12 * 90_000);
    assert!(d.notes.iter().any(|n| n.contains("65")), "{:?}", d.notes);
}

/// Ano-calendário 2026 (exercício 2027): tabela anual nova, teto do simplificado de
/// R$ 17.640,00 e a redução anual da Lei 15.270/2025 (Lei 9.250 art. 10, X, e art. 11-A;
/// Receita, tabelas 2026).
#[test]
fn annual_2026_has_the_new_cap_and_the_lei_15270_reduction() {
    use super::annual::{Employment, Input, compute};
    let one = |income: i64| Input {
        year: 2026,
        employment: vec![Employment {
            income,
            social_security: 0,
            withheld: 0,
        }],
        ..Default::default()
    };
    // R$ 50 mil: base simplificada 40 mil → 1.270,09 na tabela, zerado pela redução.
    let r = compute(&one(5_000_000));
    assert_eq!((r.simplified.table_tax, r.simplified.tax_due), (127_009, 0));
    // R$ 70 mil: redução de 8.429,73 − 0,095575 × 70.000 = 1.739,48 nos dois modelos.
    let r = compute(&one(7_000_000));
    assert_eq!(r.full.table_tax, 834_534);
    assert_eq!((r.full.reduction, r.full.tax_due), (173_948, 660_586));
    assert_eq!(
        (r.simplified.deductions, r.simplified.tax_due),
        (1_400_000, 275_586)
    );
    // R$ 100 mil: sem redução; desconto simplificado no teto de 2026.
    let r = compute(&one(10_000_000));
    assert_eq!((r.full.reduction, r.simplified.deductions), (0, 1_764_000));
    // Em 2025 o teto é o antigo e não há redução.
    let mut old = one(10_000_000);
    old.year = 2025;
    assert_eq!(compute(&old).simplified.deductions, 1_675_434);
}

/// Crédito do imposto pago lá fora só para países com acordo ou reciprocidade (Lei 14.754/2023
/// art. 4º; Receita, lista de acordos atualizada em 28/08/2026; Perguntas e Respostas IRPF
/// 2026, perguntas 136 e 142: Alemanha, EUA e Reino Unido por reciprocidade). A Austrália não
/// tem acordo com o Brasil: o imposto retido lá não vira crédito.
#[test]
fn foreign_tax_credit_only_for_treaty_or_reciprocity_countries() {
    let (_d, mut db) = db();
    let mk = |db: &mut Db, ticker: &str, json: serde_json::Value| {
        let input: AssetInput = serde_json::from_value(json).unwrap();
        let _ = ticker;
        record(db.conn_mut(), &Author::user(), "a", |tx| {
            assets::create(tx, &input)
        })
        .unwrap()
        .value
    };
    let bhp = mk(
        &mut db,
        "BHP",
        serde_json::json!({"ticker": "BHP", "name": "BHP", "class": "stock_intl", "market": "ASX"}),
    );
    let vod = mk(
        &mut db,
        "VOD",
        serde_json::json!({"ticker": "VOD", "name": "VOD", "class": "stock_intl", "market": "LSE"}),
    );
    for (a, fx) in [(&bhp, "3.5"), (&vod, "7")] {
        let div = crate::portfolio::IncomeInput {
            asset_id: a.clone(),
            account_id: "xp".into(),
            kind: crate::portfolio::IncomeKind::Dividend,
            ex_date: None,
            pay_date: "2026-06-10".into(),
            quantity: None,
            gross: 10_000,
            tax_withheld: Some(1_500),
            fx_rate: Some(fx.into()),
            status: None,
            cash_movement: Some(false),
        };
        record(db.conn_mut(), &Author::user(), "div", |tx| {
            records::create_income(tx, &div)
        })
        .unwrap();
    }
    let f = engine::year(db.conn(), None, 2026).unwrap().foreign;
    // Bruto: 350 (Austrália) + 700 (Reino Unido); pago lá fora: 52,50 + 105,00.
    assert_eq!((f.income, f.tax_paid_abroad), (105_000, 15_750));
    // Só o do Reino Unido vira crédito (até 15% de 700 = 105).
    assert_eq!((f.tax_credit, f.tax_without_credit), (10_500, 5_250));
    assert_eq!(f.estimated_tax, 15_750 - 10_500);
}

/// IRPFM (Lei 9.250/1995 art. 16-A, Lei 15.270/2025): acima de R$ 600 mil, alíquota de
/// (rendimentos ÷ 60.000 − 10)% até 10% a partir de R$ 1,2 milhão, menos os impostos do §3º e,
/// depois, os 10% retidos sobre dividendos (§5º). Só a partir de 2026.
#[test]
fn minimum_tax_on_high_incomes_follows_the_law_formula() {
    use super::annual::{MinimumTaxInput, minimum_tax};
    let base = |total: i64| MinimumTaxInput {
        year: 2026,
        total_income: total,
        annual_tax: 2_000_000,
        exclusive_tax: 500_000,
        definitive_tax: 500_000,
        dividend_withheld: 0,
    };
    assert!(
        !minimum_tax(&base(60_000_000)).applies,
        "exatamente R$ 600 mil não entra"
    );
    // R$ 900 mil: 900/60 − 10 = 5% → 45.000; menos 30.000 de impostos = 15.000.
    let r = minimum_tax(&base(90_000_000));
    assert_eq!(
        (r.rate_bp, r.gross, r.credits, r.due),
        (500, 4_500_000, 3_000_000, 1_500_000)
    );
    // Com 20.000 retidos sobre dividendos: sobram 5.000 a compensar.
    let r = minimum_tax(&MinimumTaxInput {
        dividend_withheld: 2_000_000,
        ..base(90_000_000)
    });
    assert_eq!(r.due, -500_000);
    // R$ 1,5 milhão: 10%.
    assert_eq!(minimum_tax(&base(150_000_000)).rate_bp, 1000);
    // Antes de 2026 não existe.
    assert!(
        !minimum_tax(&MinimumTaxInput {
            year: 2025,
            ..base(90_000_000)
        })
        .applies
    );
}

/// Dividendos de uma mesma empresa acima de R$ 50 mil no mês: retenção de 10% sobre o total,
/// a partir de 2026 (Lei 9.250/1995 art. 6º-A, §§ 1º e 2º). PETR3 e PETR4 são a mesma empresa.
#[test]
fn dividends_above_50k_a_month_from_one_company_are_listed() {
    let (_d, mut db) = db();
    let p3 = asset(&mut db, "PETR3", AssetClass::StockBr);
    let p4 = asset(&mut db, "PETR4", AssetClass::StockBr);
    let vale = asset(&mut db, "VALE3", AssetClass::StockBr);
    let div = |db: &mut Db, a: &str, date: &str, gross: i64| {
        let i = crate::portfolio::IncomeInput {
            asset_id: a.into(),
            account_id: "xp".into(),
            kind: crate::portfolio::IncomeKind::Dividend,
            ex_date: None,
            pay_date: date.into(),
            quantity: None,
            gross,
            tax_withheld: None,
            fx_rate: None,
            status: None,
            cash_movement: Some(false),
        };
        record(db.conn_mut(), &Author::user(), "div", |tx| {
            records::create_income(tx, &i)
        })
        .unwrap();
    };
    div(&mut db, &p3, "2026-05-20", 3_000_000);
    div(&mut db, &p4, "2026-05-21", 3_000_000);
    div(&mut db, &vale, "2026-05-20", 4_000_000);
    div(&mut db, &p3, "2025-05-20", 9_000_000);
    let y = engine::year(db.conn(), None, 2026).unwrap();
    assert_eq!(y.dividends_over_limit.len(), 1);
    let d = &y.dividends_over_limit[0];
    assert_eq!(
        (d.month.as_str(), d.amount, d.expected_withholding),
        ("2026-05", 6_000_000, 600_000)
    );
    assert_eq!(d.tickers, vec!["PETR3".to_string(), "PETR4".to_string()]);
    assert!(
        engine::year(db.conn(), None, 2025)
            .unwrap()
            .dividends_over_limit
            .is_empty()
    );
}

/// DARF pago em atraso (Lei 9.430/1996 art. 61): multa de 0,33% ao dia até 20% e juros pela
/// Selic dos meses entre o vencimento e o pagamento, mais 1% no mês do pagamento
/// (Perguntas e Respostas IRPF 2026, perguntas 063 e 064).
#[test]
fn late_darf_has_fine_capped_at_20_percent_and_selic_interest() {
    use rust_decimal_macros::dec;
    let d = |s: &str| crate::time::parse_date(s).unwrap();
    let selic = |y: i32, m: u32| match (y, m) {
        (2026, 4) => Some(dec!(1.0)),
        (2026, 5) => Some(dec!(1.1)),
        _ => None,
    };
    // 5 dias: multa 1,65%; juros só o 1% do mês do pagamento.
    let c = super::late::compute(100_000, d("2026-03-31"), d("2026-04-05"), &selic);
    assert_eq!(
        (c.days, c.fine, c.interest, c.total),
        (5, 1_650, 1_000, 102_650)
    );
    // 71 dias: multa no teto de 20%; juros de abril e maio (2,1%) + 1%.
    let c = super::late::compute(100_000, d("2026-03-31"), d("2026-06-10"), &selic);
    assert_eq!((c.fine_pct.as_str(), c.fine), ("20", 20_000));
    assert_eq!(
        (c.interest_pct.as_str(), c.interest, c.total),
        ("3.1", 3_100, 123_100)
    );
    assert!(!c.estimated);
    // Em dia: nada.
    let c = super::late::compute(100_000, d("2026-03-31"), d("2026-03-31"), &selic);
    assert_eq!(c.total, 100_000);
    // Mês sem Selic conhecida: usa o último conhecido e marca como estimado.
    let c = super::late::compute(100_000, d("2026-03-31"), d("2026-07-10"), &selic);
    assert!(c.estimated);
    assert_eq!(c.interest_pct, "4.2");
}

/// Ganho de capital de um bem do 02 · Bens: custo com ITBI e registro, venda com corretagem,
/// FR2 pelos meses da compra à venda (Lei 11.196/2005 art. 40) e 15% (Lei 8.981 art. 21).
/// Mesmo cálculo conferido nos casos do programa GCAP (`tests_dirpf_golden`).
#[test]
fn physical_asset_sale_has_capital_gain_with_fr2() {
    use crate::api::Api;
    use crate::paths::TreasuryPaths;
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let a = api
        .call(
            "physical.create",
            json!({"name": "Apartamento", "kind": "real_estate", "purchase_date": "2020-02-10",
                   "purchase_price": 40_000_000, "valuation_method": "none",
                   "acquisition_costs": [{"kind": "itbi", "amount": 1_200_000}]}),
        )
        .unwrap();
    let id = a["id"].as_str().unwrap().to_string();
    // Simulação sem vender: preço e data nos argumentos.
    let g = api
        .call(
            "tax.physical_gain",
            json!({"asset_id": id, "sale_date": "2025-03-18", "sale_price": 60_000_000, "sale_costs": 1_800_000}),
        )
        .unwrap();
    // Ganho bruto: 600.000 − 18.000 − 412.000 = 170.000; FR2 de fev/2020 a mar/2025 (62
    // meses): 19,476501% → 33.110,05; tributável 136.889,95; 15% = 20.533,49.
    assert_eq!(g["gross_gain"], 17_000_000);
    assert_eq!(g["reductions"]["fr2_pct"], "19.476501");
    assert_eq!(g["reductions"]["fr2"], 3_311_005);
    assert_eq!(
        (g["taxable_gain"].as_i64(), g["tax"].as_i64()),
        (Some(13_688_995), Some(2_053_349))
    );
    assert_eq!(
        (g["revenue_code"].as_str(), g["due_date"].as_str()),
        (Some("4600"), Some("2025-04-30"))
    );
    // Único imóvel por até R$ 440 mil: isento.
    let g = api
        .call(
            "tax.physical_gain",
            json!({"asset_id": id, "sale_date": "2025-03-18", "sale_price": 44_000_000, "only_property": true}),
        )
        .unwrap();
    assert_eq!(g["tax"], 0);
    // Sem venda e sem argumentos: pede os dados.
    assert!(
        api.call("tax.physical_gain", json!({"asset_id": id}))
            .is_err()
    );
}

/// PLR pela tabela exclusiva (Lei 10.101/2000 art. 3º e anexo; Receita, tabelas 2025 e
/// 2026; Perguntas e Respostas IRPF 2026, pergunta 179), recalculada sobre o total do ano.
#[test]
fn plr_uses_its_own_table_recalculated_on_the_year_total() {
    use super::exclusive::{PlrArgs, plr};
    let a = |month: &str, gross: i64, before: i64, withheld: i64| PlrArgs {
        month: month.into(),
        gross,
        paid_before: before,
        withheld_before: withheld,
        alimony: 0,
    };
    // R$ 8.214,40: isenta (tabela desde maio de 2025).
    assert_eq!(plr(&a("2026-03", 821_440, 0, 0)).unwrap().tax, 0);
    // R$ 10.000: 15% − 1.360,25 = 139,75.
    assert_eq!(plr(&a("2026-03", 1_000_000, 0, 0)).unwrap().tax, 13_975);
    // Segunda PLR de R$ 5.000 no ano: total 15.000 → 1.027,22, menos os 139,75 já retidos.
    assert_eq!(
        plr(&a("2026-09", 500_000, 1_000_000, 13_975)).unwrap().tax,
        88_747
    );
    // Janeiro de 2025, tabela antiga: R$ 8.000 → 7,5% − 573,06 = 26,94.
    assert_eq!(plr(&a("2025-01", 800_000, 0, 0)).unwrap().tax, 2_694);
}

/// Carnê-leão: tabela mensal, deduções legais ou o simplificado mensal, a redução da Lei
/// 15.270/2025 desde 2026 e DARF 0190 no último dia útil do mês seguinte (Perguntas e
/// Respostas IRPF 2026, perguntas 266 e 267).
#[test]
fn carne_leao_uses_the_monthly_table_and_the_2026_reduction() {
    use super::exclusive::{CarneLeaoArgs, carne_leao};
    let a = |income: i64, inss: i64| CarneLeaoArgs {
        month: "2026-03".into(),
        income,
        social_security: inss,
        dependents: 0,
        alimony: 0,
        livro_caixa: 0,
        foreign_tax: 0,
    };
    let r = carne_leao(&a(1_000_000, 0)).unwrap();
    assert_eq!(
        (r.tax, r.revenue_code.as_str(), r.due_date.as_str()),
        (167_429, "0190", "2026-04-30")
    );
    // R$ 5.000: a redução zera.
    assert_eq!(carne_leao(&a(500_000, 0)).unwrap().tax, 0);
    // R$ 6.000 com R$ 700 de INSS: 548,77 − redução de 179,75 = 369,02.
    assert_eq!(carne_leao(&a(600_000, 70_000)).unwrap().tax, 36_902);
}
