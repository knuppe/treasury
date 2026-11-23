//! Indicators checked against hand-calculated values.

use rust_decimal_macros::dec;

use super::*;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

fn fact(item: &str, pt: PeriodType, start: Option<&str>, end: &str, v: Decimal) -> Fact {
    Fact {
        item: item.into(),
        period_type: pt,
        start: start.map(d),
        end: d(end),
        value: v,
        currency: if item.starts_with("shares") {
            "shares".into()
        } else {
            "BRL".into()
        },
        source: "cvm_dfp".into(),
        document: format!("DFP {end}"),
        priority: 2,
    }
}

fn annual(item: &str, year: i32, v: Decimal) -> Fact {
    fact(
        item,
        PeriodType::Annual,
        Some(&format!("{year}-01-01")),
        &format!("{year}-12-31"),
        v,
    )
}

fn inst(item: &str, end: &str, v: Decimal) -> Fact {
    fact(item, PeriodType::Instant, None, end, v)
}

fn price(v: Decimal) -> Option<(Decimal, PriceOn)> {
    Some((
        v,
        PriceOn {
            close: dstr(v),
            date: "2026-10-13".into(),
            source: "yahoo".into(),
            stale: true,
            intraday: None,
        },
    ))
}

fn subject(class: AssetClass, layout: Layout, kind: &str) -> Subject {
    Subject {
        asset_id: Some("a1".into()),
        ticker: Some("XMPL4".into()),
        name: "Companhia Exemplo PN".into(),
        class,
        currency: "BRL".into(),
        issuer_id: Some("11222333000181".into()),
        issuer_name: Some("COMPANHIA EXEMPLO S.A.".into()),
        issuer_code: Some("11.222.333/0001-81".into()),
        sector: Some("Energia Elétrica".into()),
        segment: None,
        mandate: None,
        layout: Some(layout),
        share_kind: Some(kind.into()),
    }
}

/// Companhia Exemplo: revenue 1,000,000, profit 120,000, equity 800,000, 100 thousand shares
/// (60 thousand common at R$ 10, 40 thousand preferred at R$ 12).
fn exemplo() -> Data {
    let mut facts = vec![
        annual(items::REVENUE, 2025, dec!(1_000_000)),
        annual(items::GROSS_PROFIT, 2025, dec!(400_000)),
        annual(items::EBIT, 2025, dec!(200_000)),
        annual(items::DA, 2025, dec!(50_000)),
        annual(items::NET_INCOME, 2025, dec!(120_000)),
        annual(items::DIVIDENDS_DECLARED, 2025, dec!(50_000)),
        annual(items::DIVIDENDS_PAID, 2025, dec!(60_000)),
        inst(items::TOTAL_ASSETS, "2025-12-31", dec!(2_000_000)),
        inst(items::CURRENT_ASSETS, "2025-12-31", dec!(600_000)),
        inst(items::CASH, "2025-12-31", dec!(200_000)),
        inst(items::CURRENT_LIABILITIES, "2025-12-31", dec!(400_000)),
        inst(items::GROSS_DEBT, "2025-12-31", dec!(400_000)),
        inst(items::EQUITY, "2025-12-31", dec!(800_000)),
        inst(items::SHARES, "2025-12-31", dec!(100_000)),
        inst(items::SHARES_ON, "2025-12-31", dec!(60_000)),
        inst(items::SHARES_PN, "2025-12-31", dec!(40_000)),
        // History: 2020 with half the revenue and profit; 2021 with a loss.
        annual(items::REVENUE, 2020, dec!(500_000)),
        annual(items::NET_INCOME, 2020, dec!(60_000)),
        annual(items::NET_INCOME, 2021, dec!(-10_000)),
    ];
    facts.sort_by_key(|f| f.end);
    let ev = |s: &str, a: Decimal| DividendPoint {
        date: d(s),
        amount: a,
    };
    Data {
        subject: subject(AssetClass::StockBr, Layout::Industrial, "PN"),
        facts,
        price: price(dec!(12)),
        per_unit: Some(Decimal::ONE),
        siblings: vec![
            Sibling {
                ticker: "XMPL3".into(),
                share_kind: "ON".into(),
                close: dec!(10),
            },
            Sibling {
                ticker: "XMPL4".into(),
                share_kind: "PN".into(),
                close: dec!(12),
            },
        ],
        dividends: vec![
            ev("2021-05-01", dec!(0.8)),
            ev("2022-05-01", dec!(0.9)),
            ev("2023-05-01", dec!(1.0)),
            ev("2024-05-01", dec!(1.1)),
            ev("2025-11-15", dec!(1.2)),
            ev("2026-03-10", dec!(0.30)),
            ev("2026-08-20", dec!(0.42)),
        ],
        dividends_source: Some("agenda de proventos (yahoo)".into()),
        share_events: vec![],
        liquidity: Some((
            dec!(2_500_000),
            "média de 21 pregão(ões), B3 COTAHIST".into(),
        )),
        today: d("2026-10-14"),
    }
}

fn ind<'a>(v: &'a IndicatorsView, key: &str) -> &'a Indicator {
    v.indicators
        .iter()
        .find(|i| i.key == key)
        .unwrap_or_else(|| panic!("{key} missing"))
}

fn approx(v: &IndicatorsView, key: &str, expected: f64) {
    let got = ind(v, key)
        .value
        .unwrap_or_else(|| panic!("{key}: {:?}", ind(v, key).unavailable_reason));
    assert!(
        (got - expected).abs() < 1e-9,
        "{key}: expected {expected}, got {got}"
    );
}

#[test]
fn stock_indicators_match_hand_calculations() {
    let v = compute(&exemplo(), BazinParams::default());
    // Market value by class: 60,000 × 10 + 40,000 × 12.
    approx(&v, "market_cap", 1_080_000.0);
    assert!(ind(&v, "market_cap").formula.contains("ON × cotação ON"));
    // EV = 1,080,000 + (400,000 − 200,000).
    approx(&v, "ev", 1_280_000.0);
    approx(&v, "pl", 9.0);
    approx(&v, "pvp", 1.35);
    approx(&v, "ev_ebitda", 5.12);
    approx(&v, "ev_ebit", 6.4);
    approx(&v, "psr", 1.08);
    approx(&v, "p_ativo", 0.54);
    approx(&v, "roe", 0.15);
    approx(&v, "roa", 0.06);
    // 200,000 × 0.66 ÷ (800,000 + 200,000)
    approx(&v, "roic", 0.132);
    approx(&v, "gross_margin", 0.4);
    approx(&v, "ebitda_margin", 0.25);
    approx(&v, "ebit_margin", 0.2);
    approx(&v, "net_margin", 0.12);
    approx(&v, "net_debt_equity", 0.25);
    approx(&v, "net_debt_ebitda", 0.8);
    approx(&v, "current_ratio", 1.5);
    approx(&v, "lpa", 1.2);
    approx(&v, "vpa", 8.0);
    // 50,000 declared ÷ 120,000 of profit.
    approx(&v, "payout", 50_000.0 / 120_000.0);
    // (1,000,000 ÷ 500,000)^(1/5) − 1
    approx(&v, "revenue_cagr_5y", 2f64.powf(0.2) - 1.0);
    approx(&v, "earnings_cagr_5y", 2f64.powf(0.2) - 1.0);
    approx(&v, "loss_years_10y", 1.0);
    // Dividends with ex-date in the last 12 months: 1.20 + 0.30 + 0.42 = 1.92 ÷ 12.
    approx(&v, "dy", 0.16);
    approx(&v, "liquidity", 2_500_000.0);
    approx(&v, "price", 12.0);
    // Graham: √(22.5 × 1.2 × 8) = √216 = 14.70; Bazin: 2021–2025 average = 1.00 ÷ 6% = 16.67.
    approx(&v, "graham", 14.70);
    approx(&v, "graham_upside", 14.70 / 12.0 - 1.0);
    approx(&v, "bazin", 16.67);
    approx(&v, "bazin_upside", 16.67 / 12.0 - 1.0);

    let pl = ind(&v, "pl");
    assert_eq!(pl.period.as_deref(), Some("12 meses até 31/12/2025"));
    assert!(pl.source.as_deref().unwrap().contains("DFP 2025-12-31"));
    assert!(
        pl.source
            .as_deref()
            .unwrap()
            .contains("cotação yahoo de 2026-10-13")
    );
    assert_eq!(ind(&v, "lpa").exact.as_deref(), Some("1.2"));
    assert_eq!(ind(&v, "graham").exact.as_deref(), Some("14.7"));
    assert_eq!(v.reference_date.as_deref(), Some("2025-12-31"));
    assert!(
        v.indicators.iter().all(|i| i.key != "basel"),
        "Basel only for banks"
    );
    assert!(v.indicators.iter().all(|i| !i.formula.is_empty()));
}

#[test]
fn fair_price_shows_assumptions_and_refuses_negative_inputs() {
    let data = exemplo();
    let (g, b) = fair_price(&data, BazinParams::default());
    assert_eq!(g.value.as_deref(), Some("14.7"));
    let labels: Vec<&str> = g.assumptions.iter().map(|a| a.label.as_str()).collect();
    assert_eq!(labels, vec!["LPA (12 meses)", "VPA", "Ações em circulação"]);
    assert_eq!(g.assumptions[0].value, "1.2");
    assert_eq!(b.value.as_deref(), Some("16.67"));
    assert!(
        b.assumptions
            .iter()
            .any(|a| a.label == "Taxa mínima" && a.value == "6%")
    );
    assert!(
        b.assumptions
            .iter()
            .any(|a| a.label == "Média anual" && a.value == "1")
    );
    assert_eq!(
        b.assumptions
            .iter()
            .filter(|a| a.label.starts_with("Dividendos por ação em"))
            .count(),
        5
    );

    // 8% rate and 3 years (2023–2025): (1.0 + 1.1 + 1.2) ÷ 3 = 1.10 ÷ 8% = 13.75.
    let (_, b) = fair_price(
        &data,
        BazinParams {
            rate: dec!(0.08),
            years: 3,
        },
    );
    assert_eq!(b.value.as_deref(), Some("13.75"));

    // Loss: Graham unavailable with a reason; Bazin continues.
    let mut loss = exemplo();
    for f in loss
        .facts
        .iter_mut()
        .filter(|f| f.item == items::NET_INCOME && f.end == d("2025-12-31"))
    {
        f.value = dec!(-5000);
    }
    let (g, b) = fair_price(&loss, BazinParams::default());
    assert!(g.value.is_none());
    assert!(g.unavailable_reason.unwrap().contains("LPA negativo"));
    assert!(b.value.is_some());
    // Negative equity: same.
    let mut neg = exemplo();
    for f in neg.facts.iter_mut().filter(|f| f.item == items::EQUITY) {
        f.value = dec!(-1);
    }
    let (g, _) = fair_price(&neg, BazinParams::default());
    assert!(g.unavailable_reason.unwrap().contains("VPA negativo"));
    let v = compute(&neg, BazinParams::default());
    assert!(
        ind(&v, "pvp")
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("negativo")
    );
    // No dividends: Bazin unavailable.
    let mut none = exemplo();
    none.dividends.clear();
    none.facts.retain(|f| f.item != items::DIVIDENDS_PAID);
    let (_, b) = fair_price(&none, BazinParams::default());
    assert_eq!(
        b.unavailable_reason.as_deref(),
        Some("sem histórico de dividendos")
    );
}

#[test]
fn bazin_and_dy_fall_back_to_statements_without_agenda() {
    let mut data = exemplo();
    data.dividends.clear();
    // Dividends paid per fiscal year: 2021–2025 (DFC) ÷ 100 thousand shares.
    for (y, paid) in [
        (2021, 40_000),
        (2022, 50_000),
        (2023, 60_000),
        (2024, 70_000),
    ] {
        data.facts
            .push(annual(items::DIVIDENDS_PAID, y, Decimal::from(paid)));
        data.facts
            .push(inst(items::SHARES, &format!("{y}-12-31"), dec!(100_000)));
    }
    let (_, b) = fair_price(&data, BazinParams::default());
    // (0.4 + 0.5 + 0.6 + 0.7 + 0.6) ÷ 5 = 0.56 ÷ 6% = 9.33.
    assert_eq!(b.value.as_deref(), Some("9.33"));
    let v = compute(&data, BazinParams::default());
    let dy = ind(&v, "dy");
    // 60,000 paid in the last 12 months ÷ 100 thousand shares = 0.60 ÷ 12.
    assert!((dy.value.unwrap() - 0.05).abs() < 1e-9);
    assert!(dy.estimated);
    assert!(dy.source.as_deref().unwrap().contains("DFC"));
}

#[test]
fn market_cap_falls_back_and_units_scale_per_share_values() {
    // No quote for the common share: total shares × the security's quote.
    let mut data = exemplo();
    data.siblings.retain(|s| s.share_kind == "PN");
    let v = compute(&data, BazinParams::default());
    approx(&v, "market_cap", 1_200_000.0);
    approx(&v, "pl", 10.0);

    // Unit with 3 shares at R$ 33: R$ 11 per share; market value by class.
    let mut unit = exemplo();
    unit.subject.share_kind = Some("UNIT".into());
    unit.subject.ticker = Some("XMPL11".into());
    unit.per_unit = Some(dec!(3));
    unit.price = price(dec!(33));
    let v = compute(&unit, BazinParams::default());
    approx(&v, "market_cap", 1_080_000.0);
    approx(&v, "lpa", 3.6);
    approx(&v, "vpa", 24.0);
    // √(22.5 × 3.6 × 24) = √1944 = 44.09
    approx(&v, "graham", 44.09);
    assert!(v.notes.iter().any(|n| n.contains("Unit com 3 ações")));
    unit.siblings.clear();
    let v = compute(&unit, BazinParams::default());
    approx(&v, "market_cap", 1_100_000.0);

    // Unknown composition: per-share indicators unavailable.
    unit.per_unit = None;
    let v = compute(&unit, BazinParams::default());
    assert!(
        ind(&v, "lpa")
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("unit")
    );
}

#[test]
fn ltm_combines_dfp_and_itr_in_indicators() {
    let mut data = exemplo();
    // ITR Q2 2026: year-to-date profit 70,000; Q2 2025: 50,000 → LTM = 120,000 + 70,000 − 50,000.
    data.facts.push(fact(
        items::NET_INCOME,
        PeriodType::Ytd,
        Some("2026-01-01"),
        "2026-06-30",
        dec!(70_000),
    ));
    data.facts.push(fact(
        items::NET_INCOME,
        PeriodType::Ytd,
        Some("2025-01-01"),
        "2025-06-30",
        dec!(50_000),
    ));
    data.facts
        .push(inst(items::EQUITY, "2026-06-30", dec!(900_000)));
    let v = compute(&data, BazinParams::default());
    approx(&v, "lpa", 1.4);
    approx(&v, "roe", 140_000.0 / 900_000.0);
    assert_eq!(
        ind(&v, "lpa").period.as_deref(),
        Some("12 meses até 30/06/2026")
    );
    assert_eq!(v.reference_date.as_deref(), Some("2026-06-30"));
    // Revenue has no ITR: the last fiscal year holds, with the period in sight.
    approx(&v, "psr", 1.08);
    assert_eq!(
        ind(&v, "psr").period.as_deref(),
        Some("12 meses até 31/12/2025")
    );
}

#[test]
fn banks_have_their_own_set() {
    let mut data = exemplo();
    data.subject = subject(AssetClass::StockBr, Layout::Bank, "ON");
    data.facts = vec![
        annual(items::REVENUE, 2025, dec!(5_000_000)),
        annual(items::NET_INCOME, 2025, dec!(800_000)),
        inst(items::EQUITY, "2025-12-31", dec!(5_000_000)),
        inst(items::TOTAL_ASSETS, "2025-12-31", dec!(60_000_000)),
        inst(items::CASH, "2025-12-31", dec!(900_000)),
        inst(items::SHARES, "2025-12-31", dec!(200_000)),
    ];
    data.siblings.clear();
    data.price = price(dec!(30));
    let v = compute(&data, BazinParams::default());
    approx(&v, "market_cap", 6_000_000.0);
    approx(&v, "pl", 7.5);
    approx(&v, "pvp", 1.2);
    approx(&v, "roe", 0.16);
    approx(&v, "roa", 800_000.0 / 60_000_000.0);
    for k in [
        "ev",
        "ev_ebitda",
        "net_debt_ebitda",
        "current_ratio",
        "net_margin",
        "psr",
        "roic",
    ] {
        assert_eq!(
            ind(&v, k).unavailable_reason.as_deref(),
            Some(financial_na().as_str()),
            "{k}"
        );
    }
    assert!(
        ind(&v, "basel")
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("Basileia")
    );
    assert!(v.notes.iter().any(|n| n.contains("Instituição financeira")));
}

#[test]
fn fii_indicators() {
    let mut facts = vec![];
    for m in 1..=12 {
        let mm = format!("2025-{m:02}-01");
        facts.push(fact(
            items::INCOME_PER_SHARE,
            PeriodType::Month,
            None,
            &mm,
            dec!(0.8),
        ));
    }
    for m in 1..=9 {
        let mm = format!("2026-{m:02}-01");
        facts.push(fact(
            items::INCOME_PER_SHARE,
            PeriodType::Month,
            None,
            &mm,
            dec!(0.8),
        ));
    }
    facts.push(fact(
        items::NAV_PER_SHARE,
        PeriodType::Month,
        None,
        "2026-08-01",
        dec!(99),
    ));
    facts.push(fact(
        items::NAV_PER_SHARE,
        PeriodType::Month,
        None,
        "2026-09-01",
        dec!(100),
    ));
    facts.push(fact(
        items::FII_SHARES,
        PeriodType::Month,
        None,
        "2026-09-01",
        dec!(1_000_000),
    ));
    facts.push(fact(
        items::HOLDERS,
        PeriodType::Month,
        None,
        "2026-09-01",
        dec!(52_000),
    ));
    let mut s = subject(AssetClass::Fii, Layout::Fund, "CTF");
    s.ticker = Some("XPTO11".into());
    s.segment = Some("Logística".into());
    let data = Data {
        subject: s,
        facts,
        price: price(dec!(95)),
        per_unit: Some(Decimal::ONE),
        siblings: vec![],
        dividends: vec![],
        dividends_source: None,
        share_events: vec![],
        liquidity: None,
        today: d("2026-10-14"),
    };
    let v = compute(&data, BazinParams::default());
    approx(&v, "pvp", 0.95);
    approx(&v, "market_cap", 95_000_000.0);
    approx(&v, "nav_per_share", 100.0);
    approx(&v, "holders", 52_000.0);
    // 12 reports × 0.80 = 9.60 ÷ 95.
    approx(&v, "dy", 9.6 / 95.0);
    assert!(ind(&v, "dy").estimated);
    approx(&v, "last_dividend", 0.8);
    assert!(ind(&v, "last_dividend").estimated);
    assert!(ind(&v, "vacancy").unavailable_reason.is_some());
    assert!(ind(&v, "liquidity").unavailable_reason.is_some());
    assert!(
        v.indicators
            .iter()
            .all(|i| i.key != "graham" && i.key != "pl")
    );
    // Bazin from the report: only 2025 has the 12 months → 9.60 ÷ 6% = 160.
    approx(&v, "bazin", 160.0);
    let (g, b) = fair_price(&data, BazinParams::default());
    assert_eq!(
        g.unavailable_reason.as_deref(),
        Some("não se aplica a FIIs")
    );
    assert!(
        b.assumptions
            .iter()
            .any(|a| a.source.as_deref().is_some_and(|s| s.contains("1 ano")))
    );
}

/// DY of a FII without cash dividends: B3 with no income counts as zero; without B3, the
/// income declared in the quarterly reports; without those, the monthly reports, except the
/// implausible month, with the months in the formula. Vacancy comes from the quarterly (by area).
#[test]
fn fii_dy_sources_and_vacancy() {
    let mut facts = vec![];
    for (y, m) in (10..=12)
        .map(|m| (2025, m))
        .chain((1..=9).map(|m| (2026, m)))
    {
        let mm = format!("{y}-{m:02}-01");
        // May: 30 per unit on a book value of 100 (30% in the month) is not income.
        let v = if (y, m) == (2026, 5) {
            dec!(30)
        } else {
            dec!(0.8)
        };
        facts.push(fact(
            items::INCOME_PER_SHARE,
            PeriodType::Month,
            None,
            &mm,
            v,
        ));
        facts.push(fact(
            items::NAV_PER_SHARE,
            PeriodType::Month,
            None,
            &mm,
            dec!(100),
        ));
    }
    facts.push(fact(
        items::FII_SHARES,
        PeriodType::Month,
        None,
        "2026-09-01",
        dec!(1_000_000),
    ));
    facts.push(fact(
        items::FII_VACANCY_PHYSICAL,
        PeriodType::Instant,
        None,
        "2026-06-30",
        dec!(0.12),
    ));
    let mut s = subject(AssetClass::Fii, Layout::Fund, "CTF");
    s.ticker = Some("XPTO11".into());
    let data = Data {
        subject: s,
        facts,
        price: price(dec!(95)),
        per_unit: Some(Decimal::ONE),
        siblings: vec![],
        dividends: vec![],
        dividends_source: None,
        share_events: vec![],
        liquidity: None,
        today: d("2026-10-14"),
    };
    // Monthly reports: 11 × 0.80 (without May).
    let v = compute(&data, BazinParams::default());
    approx(&v, "dy", 8.8 / 95.0);
    let dy = ind(&v, "dy");
    assert!(dy.estimated);
    let formula = dy.formula.clone();
    assert!(
        formula.contains("09/2026") && !formula.contains("05/2026"),
        "{formula}"
    );
    approx(&v, "vacancy", 0.12);
    // Declared in the 4 quarters: 8 million ÷ 1 million units.
    let mut q = data.clone();
    for end in ["2025-09-30", "2025-12-31", "2026-03-31", "2026-06-30"] {
        q.facts.push(fact(
            items::FII_DECLARED_INCOME,
            PeriodType::Quarter,
            None,
            end,
            dec!(2_000_000),
        ));
    }
    let v = compute(&q, BazinParams::default());
    approx(&v, "dy", 8.0 / 95.0);
    assert!(
        ind(&v, "dy")
            .source
            .as_deref()
            .unwrap()
            .contains("trimestrais")
    );
    // Missing a quarter in the middle: goes back to the monthly reports.
    q.facts
        .retain(|f| !(f.item == items::FII_DECLARED_INCOME && f.end == d("2025-12-31")));
    approx(&compute(&q, BazinParams::default()), "dy", 8.8 / 95.0);
    // B3 answered with no income: zero, not estimated, and no last income.
    let mut b3 = data.clone();
    b3.dividends_source = Some("B3 (oficial): nenhum rendimento".into());
    let v = compute(&b3, BazinParams::default());
    approx(&v, "dy", 0.0);
    assert!(!ind(&v, "dy").estimated);
    assert!(ind(&v, "last_dividend").unavailable_reason.is_some());
}

#[test]
fn shares_reported_in_thousands_are_corrected_by_price() {
    // Document with 100 shares (actually 100 thousand): P/B of 0.00135 is impossible.
    let mut data = exemplo();
    for f in data
        .facts
        .iter_mut()
        .filter(|f| f.item.starts_with("shares"))
    {
        f.value /= dec!(1000);
    }
    let v = compute(&data, BazinParams::default());
    approx(&v, "market_cap", 1_080_000.0);
    approx(&v, "pvp", 1.35);
    approx(&v, "lpa", 1.2);
    assert!(v.notes.iter().any(|n| n.contains("milhares")));
    // History on different scales stays on the reference's scale.
    assert_eq!(consistent_shares(dec!(100), dec!(100_000)), dec!(100_000));
    assert_eq!(
        consistent_shares(dec!(100_000_000), dec!(100_000)),
        dec!(100_000)
    );
    assert_eq!(consistent_shares(dec!(90_000), dec!(100_000)), dec!(90_000));
    // Right scale: nothing changes.
    let v = compute(&exemplo(), BazinParams::default());
    assert!(v.notes.iter().all(|n| !n.contains("milhares")));
}

/// Exemplo with the quote of 2026-10-13 and only the common class (no class sum), so market
/// cap = shares × price.
fn exemplo_single_class() -> Data {
    let mut data = exemplo();
    data.siblings.clear();
    // Balance sheet less than 200 days old (no staleness warning in the checks).
    data.today = d("2026-03-01");
    data
}

#[test]
fn b3_share_count_confirms_or_corrects_the_balance_sheet() {
    // CVM: 102,000 total with 2,000 in treasury → 100,000 outstanding.
    let base = |b3: Decimal, total: Decimal, treasury: Decimal| {
        let mut d = exemplo_single_class();
        d.facts.push(inst(items::SHARES_TOTAL, "2025-12-31", total));
        d.facts
            .push(inst(items::SHARES_TREASURY, "2025-12-31", treasury));
        d.facts.push(inst(items::SHARES_B3, "2026-02-20", b3));
        d
    };
    // B3 agrees with the total (with treasury): verified, outstanding stays.
    let v = compute(
        &base(dec!(102_000), dec!(102_000), dec!(2_000)),
        BazinParams::default(),
    );
    approx(&v, "market_cap", 100_000.0 * 12.0);
    assert_eq!(ind(&v, "market_cap").check, InputCheck::Verified);
    // Vale-like: the CVM "total" was already net of treasury (B3 = total + treasury).
    let mut d = base(dec!(104_000), dec!(102_000), dec!(2_000));
    if let Some(f) = d.facts.iter_mut().find(|f| f.item == items::SHARES) {
        f.value = dec!(100_000); // what the reader stores: total − treasury
    }
    let v = compute(&d, BazinParams::default());
    approx(&v, "market_cap", 102_000.0 * 12.0);
    assert_eq!(ind(&v, "market_cap").check, InputCheck::Corrected);
    // Shares in thousands at the CVM: the B3 count gives the scale.
    let mut d = base(dec!(102_000_000), dec!(102_000), dec!(2_000));
    d.facts
        .retain(|f| f.item != items::SHARES_ON && f.item != items::SHARES_PN);
    let v = compute(&d, BazinParams::default());
    approx(&v, "market_cap", 100_000_000.0 * 12.0);
    assert_eq!(ind(&v, "market_cap").check, InputCheck::Corrected);
    // A 2-for-1 split after the balance sheet explains the B3 count.
    let mut d = base(dec!(204_000), dec!(102_000), dec!(2_000));
    d.share_events = vec![(d_("2026-02-02"), dec!(2))];
    let v = compute(&d, BazinParams::default());
    approx(&v, "market_cap", 200_000.0 * 12.0);
    assert_eq!(ind(&v, "market_cap").check, InputCheck::Corrected);
    // Unexplained 5%: value kept, with a warning; above 10%: blocked.
    let v = compute(
        &base(dec!(107_100), dec!(102_000), dec!(2_000)),
        BazinParams::default(),
    );
    approx(&v, "market_cap", 100_000.0 * 12.0);
    assert_eq!(ind(&v, "market_cap").check, InputCheck::Warning);
    let v = compute(
        &base(dec!(130_000), dec!(102_000), dec!(2_000)),
        BazinParams::default(),
    );
    let mc = ind(&v, "market_cap");
    assert_eq!(mc.check, InputCheck::Blocked);
    assert!(mc.value.is_none());
    assert!(mc.unavailable_reason.as_deref().unwrap().contains("B3"));
    assert!(ind(&v, "pl").value.is_none());
    // ROE does not depend on the share count.
    assert!(ind(&v, "roe").value.is_some());
}

fn d_(s: &str) -> NaiveDate {
    d(s)
}

#[test]
fn balances_come_from_the_latest_balance_sheet_only() {
    let mut data = exemplo();
    // Non-controlling interests reported only in an old balance sheet: not used.
    data.facts
        .push(inst(items::MINORITY, "2020-12-31", dec!(999_999)));
    let v = compute(&data, BazinParams::default());
    approx(&v, "ev", 1_080_000.0 + 200_000.0);
    // In the latest balance sheet: EV and ROIC include them.
    data.facts
        .push(inst(items::MINORITY, "2025-12-31", dec!(100_000)));
    let v = compute(&data, BazinParams::default());
    approx(&v, "ev", 1_080_000.0 + 200_000.0 + 100_000.0);
    approx(
        &v,
        "roic",
        200_000.0 * 0.66 / (800_000.0 + 100_000.0 + 200_000.0),
    );
}

#[test]
fn net_margin_and_roa_use_the_consolidated_total() {
    let mut data = exemplo();
    data.facts
        .push(annual(items::NET_INCOME_TOTAL, 2025, dec!(150_000)));
    let v = compute(&data, BazinParams::default());
    approx(&v, "net_margin", 0.15);
    approx(&v, "roa", 150_000.0 / 2_000_000.0);
    // P/L and ROE stay with the controllers' income.
    approx(&v, "roe", 120_000.0 / 800_000.0);
}

#[test]
fn roic_is_unavailable_when_net_cash_shrinks_the_capital() {
    let mut data = exemplo();
    data.facts.retain(|f| f.item != items::CASH);
    // Cash of 1,150,000: capital = 800,000 + 400,000 − 1,150,000 = 50,000 < 10% of assets.
    data.facts
        .push(inst(items::CASH, "2025-12-31", dec!(1_150_000)));
    let v = compute(&data, BazinParams::default());
    assert!(ind(&v, "roic").value.is_none());
    assert!(
        ind(&v, "roic")
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("10%")
    );
}

#[test]
fn holding_companies_skip_operating_indicators() {
    let mut data = exemplo();
    // Equity method above half of EBIT (Itaúsa-like).
    data.facts
        .push(annual(items::EQUITY_INCOME, 2025, dec!(150_000)));
    let v = compute(&data, BazinParams::default());
    for key in [
        "ev_ebitda",
        "ebit_margin",
        "roic",
        "p_ebit",
        "asset_turnover",
    ] {
        assert!(ind(&v, key).value.is_none(), "{key}");
    }
    assert!(ind(&v, "pl").value.is_some());
    assert!(ind(&v, "net_debt_equity").value.is_some());
    assert!(v.notes.iter().any(|n| n.contains("equivalência")));
    // Insurer by the registry sector, with an industrial-format DRE.
    let mut data = exemplo();
    data.subject.sector = Some("Seguradoras e Corretoras".into());
    let v = compute(&data, BazinParams::default());
    assert!(ind(&v, "ev").value.is_none());
}

#[test]
fn income_checks_against_the_individual_statements() {
    // Consolidated controllers' line zeroed and replaced by the individual: corrected.
    let mut data = exemplo_single_class();
    data.facts
        .push(annual(items::NET_INCOME_CONSOLIDATED, 2025, dec!(0)));
    data.facts
        .push(annual(items::NET_INCOME_INDIVIDUAL, 2025, dec!(120_000)));
    let v = compute(&data, BazinParams::default());
    let pl = ind(&v, "pl");
    assert_eq!(pl.check, InputCheck::Corrected);
    assert!(pl.check_notes[0].contains("individual"));
    // Disagreeing by 5%: warning; by more than 20% (and R$ 1 million): blocked.
    let mut data = exemplo_single_class();
    data.facts
        .push(annual(items::NET_INCOME_INDIVIDUAL, 2025, dec!(126_000)));
    let v = compute(&data, BazinParams::default());
    assert_eq!(ind(&v, "pl").check, InputCheck::Warning);
    let mut data = exemplo();
    for f in data
        .facts
        .iter_mut()
        .filter(|f| f.item == items::NET_INCOME)
    {
        f.value *= dec!(100);
    }
    data.facts
        .push(annual(items::NET_INCOME_INDIVIDUAL, 2025, dec!(-5_000_000)));
    let v = compute(&data, BazinParams::default());
    assert_eq!(ind(&v, "pl").check, InputCheck::Blocked);
    assert!(ind(&v, "pl").value.is_none());
    assert!(ind(&v, "roe").value.is_none());
    // Revenue-only indicators are unaffected.
    assert!(ind(&v, "psr").value.is_some());
}

#[test]
fn restatements_are_flagged_but_blank_comparatives_are_not() {
    let ytd = |item: &str, end: &str, v: Decimal, pri: i64| Fact {
        priority: pri,
        ..fact(item, PeriodType::Ytd, Some("2025-01-01"), end, v)
    };
    let mut data = exemplo_single_class();
    data.facts
        .push(ytd(items::NET_INCOME, "2026-06-30", dec!(70_000), 2));
    data.facts
        .push(ytd(items::NET_INCOME, "2025-06-30", dec!(50_000), 2));
    data.facts
        .push(ytd(items::REVENUE, "2026-06-30", dec!(600_000), 2));
    data.facts
        .push(ytd(items::REVENUE, "2025-06-30", dec!(500_000), 2));
    data.facts
        .push(ytd(items::REVENUE_RESTATED, "2025-06-30", dec!(450_000), 1));
    for f in data
        .facts
        .iter_mut()
        .filter(|f| f.end == d("2026-06-30") && f.period_type == PeriodType::Ytd)
    {
        f.start = Some(d("2026-01-01"));
    }
    data.facts
        .push(inst(items::TOTAL_ASSETS, "2026-06-30", dec!(2_000_000)));
    let v = compute(&data, BazinParams::default());
    let psr = ind(&v, "psr");
    assert_eq!(psr.check, InputCheck::Warning, "{:?}", psr.check_notes);
    assert!(psr.check_notes.iter().any(|n| n.contains("reapresentou")));
    // A zero comparative is a blank field.
    for f in data
        .facts
        .iter_mut()
        .filter(|f| f.item == items::REVENUE_RESTATED)
    {
        f.value = Decimal::ZERO;
    }
    let v = compute(&data, BazinParams::default());
    assert!(
        !ind(&v, "psr")
            .check_notes
            .iter()
            .any(|n| n.contains("reapresentou"))
    );
}

#[test]
fn estimated_dividends_stay_out_of_rankings() {
    assert!(uses_dividends("dy"));
    assert!(uses_dividends("bazin_upside"));
    assert!(!uses_dividends("pl"));
    let mut data = exemplo();
    data.dividends.clear();
    data.dividends_source = None;
    let v = compute(&data, BazinParams::default());
    let dy = ind(&v, "dy");
    assert!(dy.estimated);
    assert_eq!(dy.check, InputCheck::Warning);
}
