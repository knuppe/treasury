//! Indicadores conferidos com valores calculados à mão.

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

/// Companhia Exemplo: receita 1.000.000, lucro 120.000, PL 800.000, 100 mil ações
/// (60 mil ON a R$ 10, 40 mil PN a R$ 12).
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
        // Histórico: 2020 com metade da receita e do lucro; 2021 com prejuízo.
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
        .unwrap_or_else(|| panic!("{key} ausente"))
}

fn approx(v: &IndicatorsView, key: &str, expected: f64) {
    let got = ind(v, key)
        .value
        .unwrap_or_else(|| panic!("{key}: {:?}", ind(v, key).unavailable_reason));
    assert!(
        (got - expected).abs() < 1e-9,
        "{key}: esperado {expected}, veio {got}"
    );
}

#[test]
fn stock_indicators_match_hand_calculations() {
    let v = compute(&exemplo(), BazinParams::default());
    // Valor de mercado por classe: 60.000 × 10 + 40.000 × 12.
    approx(&v, "market_cap", 1_080_000.0);
    assert!(ind(&v, "market_cap").formula.contains("ON × cotação ON"));
    // EV = 1.080.000 + (400.000 − 200.000).
    approx(&v, "ev", 1_280_000.0);
    approx(&v, "pl", 9.0);
    approx(&v, "pvp", 1.35);
    approx(&v, "ev_ebitda", 5.12);
    approx(&v, "ev_ebit", 6.4);
    approx(&v, "psr", 1.08);
    approx(&v, "p_ativo", 0.54);
    approx(&v, "roe", 0.15);
    approx(&v, "roa", 0.06);
    // 200.000 × 0,66 ÷ (800.000 + 200.000)
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
    // 50.000 declarados ÷ 120.000 de lucro.
    approx(&v, "payout", 50_000.0 / 120_000.0);
    // (1.000.000 ÷ 500.000)^(1/5) − 1
    approx(&v, "revenue_cagr_5y", 2f64.powf(0.2) - 1.0);
    approx(&v, "earnings_cagr_5y", 2f64.powf(0.2) - 1.0);
    approx(&v, "loss_years_10y", 1.0);
    // Proventos com data-com nos últimos 12 meses: 1,20 + 0,30 + 0,42 = 1,92 ÷ 12.
    approx(&v, "dy", 0.16);
    approx(&v, "liquidity", 2_500_000.0);
    approx(&v, "price", 12.0);
    // Graham: √(22,5 × 1,2 × 8) = √216 = 14,70; Bazin: média 2021–2025 = 1,00 ÷ 6% = 16,67.
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
        "Basileia só para bancos"
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

    // Taxa de 8% e 3 anos (2023–2025): (1,0 + 1,1 + 1,2) ÷ 3 = 1,10 ÷ 8% = 13,75.
    let (_, b) = fair_price(
        &data,
        BazinParams {
            rate: dec!(0.08),
            years: 3,
        },
    );
    assert_eq!(b.value.as_deref(), Some("13.75"));

    // Prejuízo: Graham indisponível com motivo; Bazin continua.
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
    // PL negativo: idem.
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
    // Sem proventos: Bazin indisponível.
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
    // Dividendos pagos por exercício: 2021–2025 (DFC) ÷ 100 mil ações.
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
    // (0,4 + 0,5 + 0,6 + 0,7 + 0,6) ÷ 5 = 0,56 ÷ 6% = 9,33.
    assert_eq!(b.value.as_deref(), Some("9.33"));
    let v = compute(&data, BazinParams::default());
    let dy = ind(&v, "dy");
    // 60.000 pagos nos últimos 12 meses ÷ 100 mil ações = 0,60 ÷ 12.
    assert!((dy.value.unwrap() - 0.05).abs() < 1e-9);
    assert!(dy.estimated);
    assert!(dy.source.as_deref().unwrap().contains("DFC"));
}

#[test]
fn market_cap_falls_back_and_units_scale_per_share_values() {
    // Sem cotação da ON: total de ações × cotação do papel.
    let mut data = exemplo();
    data.siblings.retain(|s| s.share_kind == "PN");
    let v = compute(&data, BazinParams::default());
    approx(&v, "market_cap", 1_200_000.0);
    approx(&v, "pl", 10.0);

    // Unit com 3 ações a R$ 33: por ação R$ 11; valor de mercado por classe.
    let mut unit = exemplo();
    unit.subject.share_kind = Some("UNIT".into());
    unit.subject.ticker = Some("XMPL11".into());
    unit.per_unit = Some(dec!(3));
    unit.price = price(dec!(33));
    let v = compute(&unit, BazinParams::default());
    approx(&v, "market_cap", 1_080_000.0);
    approx(&v, "lpa", 3.6);
    approx(&v, "vpa", 24.0);
    // √(22,5 × 3,6 × 24) = √1944 = 44,09
    approx(&v, "graham", 44.09);
    assert!(v.notes.iter().any(|n| n.contains("Unit com 3 ações")));
    unit.siblings.clear();
    let v = compute(&unit, BazinParams::default());
    approx(&v, "market_cap", 1_100_000.0);

    // Composição desconhecida: indicadores por ação indisponíveis.
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
    // ITR 2T26: lucro acumulado 70.000; 2T25: 50.000 → LTM = 120.000 + 70.000 − 50.000.
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
    // A receita não tem ITR: vale o último exercício, com o período à vista.
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
        liquidity: None,
        today: d("2026-10-14"),
    };
    let v = compute(&data, BazinParams::default());
    approx(&v, "pvp", 0.95);
    approx(&v, "market_cap", 95_000_000.0);
    approx(&v, "nav_per_share", 100.0);
    approx(&v, "holders", 52_000.0);
    // 12 informes × 0,80 = 9,60 ÷ 95.
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
    // Bazin pelo informe: só 2025 tem os 12 meses → 9,60 ÷ 6% = 160.
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

/// DY de FII sem proventos em dinheiro: a B3 sem rendimentos vale zero; sem a B3, os
/// rendimentos declarados nos informes trimestrais; sem eles, os informes mensais, fora o mês
/// implausível, com os meses na fórmula. A vacância vem do trimestral (pela área).
#[test]
fn fii_dy_sources_and_vacancy() {
    let mut facts = vec![];
    for (y, m) in (10..=12)
        .map(|m| (2025, m))
        .chain((1..=9).map(|m| (2026, m)))
    {
        let mm = format!("{y}-{m:02}-01");
        // Maio: 30 por cota sobre VP de 100 (30% no mês) não é rendimento.
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
        liquidity: None,
        today: d("2026-10-14"),
    };
    // Informes mensais: 11 × 0,80 (sem maio).
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
    // Declarados nos 4 trimestres: 8 milhões ÷ 1 milhão de cotas.
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
    // Faltando um trimestre no meio: volta aos informes mensais.
    q.facts
        .retain(|f| !(f.item == items::FII_DECLARED_INCOME && f.end == d("2025-12-31")));
    approx(&compute(&q, BazinParams::default()), "dy", 8.8 / 95.0);
    // A B3 respondeu sem rendimento: zero, não estimado, e sem último rendimento.
    let mut b3 = data.clone();
    b3.dividends_source = Some("B3 (oficial): nenhum rendimento".into());
    let v = compute(&b3, BazinParams::default());
    approx(&v, "dy", 0.0);
    assert!(!ind(&v, "dy").estimated);
    assert!(ind(&v, "last_dividend").unavailable_reason.is_some());
}

#[test]
fn shares_reported_in_thousands_are_corrected_by_price() {
    // Documento com 100 ações (na verdade 100 mil): P/VP de 0,00135 é impossível.
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
    // Histórico em escalas diferentes fica na escala da referência.
    assert_eq!(consistent_shares(dec!(100), dec!(100_000)), dec!(100_000));
    assert_eq!(
        consistent_shares(dec!(100_000_000), dec!(100_000)),
        dec!(100_000)
    );
    assert_eq!(consistent_shares(dec!(90_000), dec!(100_000)), dec!(90_000));
    // Escala certa: nada muda.
    let v = compute(&exemplo(), BazinParams::default());
    assert!(v.notes.iter().all(|n| !n.contains("milhares")));
}
