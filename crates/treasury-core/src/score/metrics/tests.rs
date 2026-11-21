//! Testes das métricas (fatos sintéticos conferidos à mão).

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::*;
use crate::research::{Fact, PeriodType};

pub(crate) fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

pub(crate) fn fact(item: &str, pt: PeriodType, start: Option<&str>, end: &str, v: f64) -> Fact {
    Fact {
        item: item.into(),
        period_type: pt,
        start: start.map(d),
        end: d(end),
        value: Decimal::from_f64_retain(v).unwrap().round_dp(6),
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

/// Companhia sintética: receita crescendo 10% ao ano, margem líquida de 10%, balanço estável.
/// `scale` multiplica tudo (pares de tamanhos diferentes); `ni_margin` muda a margem.
pub(crate) fn company(scale: f64, ni_margin: f64, net_debt_factor: f64) -> Vec<Fact> {
    let mut v = vec![];
    for (i, y) in (2018..=2025).enumerate() {
        let start = format!("{y}-01-01");
        let end = format!("{y}-12-31");
        let rev = 1000.0 * scale * 1.1f64.powi(i as i32);
        let a = |item: &str, x: f64| fact(item, PeriodType::Annual, Some(&start), &end, x);
        let b = |item: &str, x: f64| fact(item, PeriodType::Instant, None, &end, x);
        v.push(a(keys::REVENUE, rev));
        v.push(a(keys::COGS, -0.6 * rev));
        v.push(a(keys::GROSS_PROFIT, 0.4 * rev));
        v.push(a(keys::SGA, -0.15 * rev));
        // EBIT coerente com o lucro: LL = (EBIT − resultado financeiro de 3%) × (1 − 34%).
        v.push(a(keys::EBIT, (ni_margin / 0.66 + 0.03) * rev));
        v.push(a(keys::DA, 0.05 * rev));
        v.push(a(keys::NET_INCOME, ni_margin * rev));
        v.push(a(keys::CFO, (ni_margin + 0.05) * rev));
        v.push(a(keys::CAPEX, -0.04 * rev));
        v.push(a(keys::DIVIDENDS_PAID, 0.5 * ni_margin * rev));
        v.push(a(keys::FIN_EXPENSE, -0.04 * rev));
        v.push(a(keys::FIN_INCOME, 0.01 * rev));
        v.push(b(keys::TOTAL_ASSETS, 1.2 * rev));
        v.push(b(keys::CURRENT_ASSETS, 0.5 * rev));
        v.push(b(keys::CURRENT_LIABILITIES, 0.3 * rev));
        v.push(b(keys::TOTAL_LIABILITIES, 0.6 * rev));
        v.push(b(keys::CASH, 0.1 * rev));
        v.push(b(keys::RECEIVABLES, 0.15 * rev));
        v.push(b(keys::INVENTORIES, 0.1 * rev));
        v.push(b(keys::PPE, 0.5 * rev));
        v.push(b(keys::SUPPLIERS, 0.08 * rev));
        v.push(b(keys::DEBT_ST, 0.05 * rev));
        v.push(b(keys::DEBT_LT, 0.1 * rev + net_debt_factor * rev));
        v.push(b(keys::GROSS_DEBT, 0.15 * rev + net_debt_factor * rev));
        v.push(b(keys::EQUITY, 0.6 * rev));
        v.push(b(keys::RETAINED, 0.3 * rev));
        v.push(b(keys::SHARES, 1000.0 * scale));
    }
    v
}

fn no_inflation(_: NaiveDate, _: NaiveDate) -> Option<f64> {
    Some(1.0)
}

fn run(facts: &[Fact], mc: Option<f64>) -> EquityOutput {
    let st = Statements::new(facts, d("2026-10-16"));
    equity_metrics(&EquityInput {
        st: &st,
        today: d("2026-10-16"),
        market_cap: mc.map(|c| (c, "teste".into())),
        dividend_yield: None,
        listing: Some((100.0, "Novo Mercado".into())),
        filings: &[],
        prices: &[],
        inflation: &no_inflation,
        tax_rate: 0.34,
        history_years: 5,
    })
}

fn v(o: &EquityOutput, k: &str) -> f64 {
    o.metrics[k]
        .value
        .unwrap_or_else(|| panic!("{k}: {:?}", o.metrics[k]))
}

#[test]
fn ratios_by_hand() {
    let f = company(1.0, 0.1, 0.0);
    let o = run(&f, Some(2000.0));
    let rev25 = 1000.0 * 1.1f64.powi(7);
    // ROE = LL / PL médio: 0,1·R25 ÷ média(0,6·R25, 0,6·R24).
    let roe = 0.1 * rev25 / ((0.6 * rev25 + 0.6 * rev25 / 1.1) / 2.0);
    assert!((v(&o, "roe") - roe).abs() < 1e-6);
    // EBIT = (0,1/0,66 + 0,03)R; dívida líquida = 0,15R − 0,1R = 0,05R; EBITDA = EBIT + 0,05R.
    let em = 0.1 / 0.66 + 0.03;
    assert!((v(&o, "net_debt_ebitda") - 0.05 / (em + 0.05)).abs() < 1e-6);
    // ROIC = EBIT·0,66 ÷ (0,05R + 0,6R).
    assert!((v(&o, "roic") - em * 0.66 / 0.65).abs() < 1e-6);
    assert!((v(&o, "net_margin") - 0.1).abs() < 1e-6);
    assert!((v(&o, "ebit_margin") - em).abs() < 1e-6);
    // Cobertura = EBIT ÷ (0,04R − 0,01R).
    assert!((v(&o, "interest_coverage") - em / 0.03).abs() < 1e-6);
    assert!((v(&o, "current_ratio") - 0.5 / 0.3).abs() < 1e-6);
    // FCF = 0,15R − 0,04R = 0,11R.
    assert!((v(&o, "fcf_margin") - 0.11).abs() < 1e-6);
    assert!((v(&o, "fcf_to_net_income") - 1.1).abs() < 1e-6);
    // Crescimento real de 10% (IPCA neutro); LPA cresce 10% (ações constantes).
    assert!((v(&o, "revenue_cagr_5y_real") - 0.1).abs() < 1e-6);
    assert!((v(&o, "eps_cagr_5y") - 0.1).abs() < 1e-6);
    assert_eq!(v(&o, "loss_years_10y"), 0.0);
    assert!(v(&o, "roe_stability") < 0.01);
    assert_eq!(v(&o, "dilution"), 0.0);
    // Payout = dividendos pagos ÷ lucro (sem DVA) = 50%; cobertura = 0,11R ÷ 0,05R.
    assert!((v(&o, "payout") - 0.5).abs() < 1e-6);
    assert!((v(&o, "dividend_fcf_coverage") - 2.2).abs() < 1e-6);
    // Valuation: E/P = 0,1R ÷ 2000.
    assert!((v(&o, "earnings_to_price") - 0.1 * rev25 / 2000.0).abs() < 1e-6);
    assert!((v(&o, "pb") - 2000.0 / (0.6 * rev25)).abs() < 1e-6);
    // Ciclo de caixa = 0,15·365 + (0,1/0,6)·365 − (0,08/0,6)·365.
    let cc = 0.15 * 365.0 + 0.1 / 0.6 * 365.0 - 0.08 / 0.6 * 365.0;
    assert!((v(&o, "cash_cycle_days") - cc).abs() < 1e-6);
    // Métricas-chave com média e tendência (ROE constante: inclinação ≈ 0).
    let roe_m = &o.metrics["roe"];
    assert!(roe_m.avg.is_some() && roe_m.trend.unwrap().abs() < 1e-3);
    assert_eq!(v(&o, "listing_segment"), 100.0);
    assert!(o.metrics["momentum_12_1"].missing.is_some());
    // Piotroski calculável e Altman em zona segura.
    assert!(o.metrics["piotroski"].value.is_some());
    assert!(v(&o, "altman_z2") > 2.6);
}

#[test]
fn negative_denominators_get_the_worst_score() {
    let mut f = company(1.0, -0.05, 0.0);
    // EBIT negativo no último exercício.
    for x in f
        .iter_mut()
        .filter(|x| x.item == keys::EBIT && x.end == d("2025-12-31"))
    {
        x.value = Decimal::from(-100);
    }
    for x in f
        .iter_mut()
        .filter(|x| x.item == keys::EQUITY && x.end == d("2025-12-31"))
    {
        x.value = Decimal::from(-50);
    }
    let o = run(&f, Some(1000.0));
    assert!(o.metrics["net_debt_ebitda"].worst);
    assert!(o.metrics["roe"].worst);
    assert!(o.metrics["pb"].worst);
    assert!(o.negative_equity);
    assert_eq!(v(&o, "loss_years_10y"), 8.0);
    assert!(o.metrics["fcf_to_net_income"].missing.is_some());
}

#[test]
fn missing_history_is_reported() {
    let f: Vec<Fact> = company(1.0, 0.1, 0.0)
        .into_iter()
        .filter(|x| x.end >= d("2024-12-31"))
        .collect();
    let o = run(&f, None);
    assert!(o.metrics["loss_years_10y"].missing.is_some());
    assert!(o.metrics["revenue_cagr_5y_real"].missing.is_some());
    assert!(
        o.metrics["earnings_yield"]
            .missing
            .as_deref()
            .unwrap()
            .contains("cotação")
    );
    assert!(o.metrics["roe"].value.is_some());
}

#[test]
fn timely_filing_and_price_stats() {
    let fl = vec![
        ("dfp".to_string(), d("2025-12-31"), d("2026-03-20")),
        ("itr".to_string(), d("2026-03-31"), d("2026-05-10")),
        ("itr".to_string(), d("2026-06-30"), d("2026-08-30")),
    ];
    let t = timely_filing(&fl, d("2026-10-16"));
    assert!((t.value.unwrap() - 2.0 / 3.0).abs() < 1e-6);
    let prices: Vec<(NaiveDate, f64)> = (0..400)
        .map(|i| {
            (
                d("2025-07-01") + chrono::Duration::days(i),
                10.0 * 1.001f64.powi(i as i32),
            )
        })
        .collect();
    let (vol, mom) = price_stats(&prices, d("2026-08-01"));
    assert!(
        vol.value.unwrap() < 0.01,
        "retornos constantes: volatilidade ~0"
    );
    assert!(mom.value.unwrap() > 0.2);
}
