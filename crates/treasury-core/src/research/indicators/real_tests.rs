//! Indicators from real CVM rows (`research/fixtures/cvm/`, ITR of 2026-09-30 and DFP of
//! 2025): the cases found in the November 2026 study, with the values recomputed independently
//! from the full files (`scripts/indicators-audit.py`).

use std::collections::HashMap;

use rust_decimal_macros::dec;

use super::*;
use crate::research::cvm::{self, FilingKind};
use crate::research::unit_count;

const KLABIN: &str = "89637490000145";
const TAESA: &str = "07859971000130";
const CVC: &str = "10760260000119";
const ITAU: &str = "60872504000123";
const WEG: &str = "84429695000111";

macro_rules! fixture {
    ($name:literal) => {
        ($name, include_str!(concat!("../fixtures/cvm/", $name)))
    };
}

fn zip(files: &[(&str, &str)]) -> Vec<u8> {
    let owned: Vec<(&str, String)> = files.iter().map(|(n, c)| (*n, c.to_string())).collect();
    cvm::fixtures::zip_latin1(&owned)
}

/// Facts and layout per company (CNPJ digits), from both documents.
fn load() -> HashMap<String, (Vec<Fact>, Option<Layout>)> {
    let itr = zip(&[
        fixture!("itr_cia_aberta_DRE_con_2026.csv"),
        fixture!("itr_cia_aberta_DRE_ind_2026.csv"),
        fixture!("itr_cia_aberta_BPA_con_2026.csv"),
        fixture!("itr_cia_aberta_BPA_ind_2026.csv"),
        fixture!("itr_cia_aberta_BPP_con_2026.csv"),
        fixture!("itr_cia_aberta_BPP_ind_2026.csv"),
        fixture!("itr_cia_aberta_DFC_MI_con_2026.csv"),
        fixture!("itr_cia_aberta_DVA_con_2026.csv"),
        fixture!("itr_cia_aberta_composicao_capital_2026.csv"),
    ]);
    let dfp = zip(&[
        fixture!("dfp_cia_aberta_DRE_con_2025.csv"),
        fixture!("dfp_cia_aberta_DRE_ind_2025.csv"),
        fixture!("dfp_cia_aberta_BPA_con_2025.csv"),
        fixture!("dfp_cia_aberta_BPA_ind_2025.csv"),
        fixture!("dfp_cia_aberta_BPP_con_2025.csv"),
        fixture!("dfp_cia_aberta_BPP_ind_2025.csv"),
        fixture!("dfp_cia_aberta_DFC_MI_con_2025.csv"),
        fixture!("dfp_cia_aberta_DVA_con_2025.csv"),
        fixture!("dfp_cia_aberta_composicao_capital_2025.csv"),
    ]);
    let mut out: HashMap<String, (Vec<Fact>, Option<Layout>)> = HashMap::new();
    for (bytes, kind) in [(dfp, FilingKind::Dfp), (itr, FilingKind::Itr)] {
        for f in cvm::parse_statements(&bytes, kind, &|_| true).unwrap() {
            let e = out.entry(f.cnpj.clone()).or_default();
            if f.layout.is_some() {
                e.1 = f.layout;
            }
            // Same rule as the store: the original (priority 2) beats the restatement.
            for fact in f.facts {
                let same = e.0.iter().position(|x: &Fact| {
                    x.item == fact.item && x.period_type == fact.period_type && x.end == fact.end
                });
                match same {
                    Some(i) if e.0[i].priority > fact.priority => {}
                    Some(i) => e.0[i] = fact,
                    None => e.0.push(fact),
                }
            }
        }
    }
    out
}

fn data(
    all: &HashMap<String, (Vec<Fact>, Option<Layout>)>,
    cnpj: &str,
    ticker: &str,
    kind: &str,
    close: Decimal,
    per_unit: Decimal,
) -> Data {
    let (facts, layout) = all.get(cnpj).cloned().expect("company in the fixtures");
    Data {
        subject: Subject {
            asset_id: None,
            ticker: Some(ticker.into()),
            name: ticker.into(),
            class: AssetClass::StockBr,
            currency: "BRL".into(),
            issuer_id: Some(cnpj.into()),
            issuer_name: None,
            issuer_code: None,
            sector: None,
            segment: None,
            mandate: None,
            layout,
            share_kind: Some(kind.into()),
        },
        facts,
        price: Some((
            close,
            PriceOn {
                close: dstr(close),
                date: "2026-11-19".into(),
                source: "b3_cotahist".into(),
                stale: true,
                intraday: None,
            },
        )),
        per_unit: Some(per_unit),
        siblings: vec![],
        dividends: vec![],
        dividends_source: Some("B3 (oficial)".into()),
        liquidity: None,
        share_events: vec![],
        today: NaiveDate::from_ymd_opt(2026, 11, 23).unwrap(),
    }
}

fn get<'a>(v: &'a IndicatorsView, key: &str) -> &'a Indicator {
    v.indicators.iter().find(|i| i.key == key).unwrap()
}

fn close_to(v: &IndicatorsView, key: &str, expected: f64) {
    let got = get(v, key).value.unwrap_or_else(|| {
        panic!("{key}: {:?}", get(v, key).unavailable_reason);
    });
    let tol = expected.abs() * 1e-6;
    assert!((got - expected).abs() <= tol, "{key}: {got} != {expected}");
}

fn ltm(d: &Data, item: &str) -> Decimal {
    periods::compute_ltm(&d.facts)
        .into_iter()
        .find(|f| f.item == item)
        .unwrap_or_else(|| panic!("no LTM of {item}"))
        .value
}

#[test]
fn clean_company_matches_the_independent_recalculation() {
    let all = load();
    let d = data(&all, WEG, "WEGE3", "ON", dec!(51.13), Decimal::ONE);
    assert_eq!(ltm(&d, items::NET_INCOME), dec!(6_268_804_000));
    assert_eq!(ltm(&d, items::NET_INCOME_TOTAL), dec!(6_690_278_000));
    assert_eq!(ltm(&d, items::EBIT), dec!(7_884_487_000));
    assert_eq!(ltm(&d, items::REVENUE), dec!(40_719_467_000));
    let v = compute(&d, BazinParams::default());
    let shares = 4_197_317_998.0 - 1_488_307.0;
    let mc = shares * 51.13;
    close_to(&v, "market_cap", mc);
    close_to(&v, "pl", mc / 6_268_804_000.0);
    close_to(&v, "pvp", mc / 19_956_834_000.0);
    // Net margin with the consolidated total, like revenue.
    close_to(&v, "net_margin", 6_690_278_000.0 / 40_719_467_000.0);
    // EV with non-controlling interests.
    let ev = mc + 3_754_445_000.0 - 6_339_447_000.0 + 1_410_309_000.0;
    close_to(&v, "ev", ev);
    close_to(&v, "ev_ebitda", ev / (7_884_487_000.0 + 1_068_117_000.0));
    // New indicators.
    close_to(&v, "p_ebit", mc / 7_884_487_000.0);
    close_to(
        &v,
        "p_working_capital",
        mc / (28_416_086_000.0 - 18_461_100_000.0),
    );
    close_to(&v, "p_ncav", mc / (28_416_086_000.0 - 24_268_303_000.0));
    close_to(&v, "asset_turnover", 40_719_467_000.0 / 45_635_446_000.0);
    close_to(&v, "gross_debt_equity", 3_754_445_000.0 / 19_956_834_000.0);
    assert_eq!(get(&v, "pl").check, InputCheck::Verified);
    assert!(get(&v, "pl").check_notes.is_empty());
}

#[test]
fn zeroed_controllers_income_is_taken_from_the_individual_statements() {
    let all = load();
    // Taesa: 9M26 with the controllers' line at zero.
    let d = data(&all, TAESA, "TAEE11", "UNIT", dec!(40.17), dec!(3));
    assert_eq!(ltm(&d, items::NET_INCOME), dec!(1_645_594_000));
    let v = compute(&d, BazinParams::default());
    let mc = 1_033_497_000.0 * 40.17 / 3.0;
    close_to(&v, "market_cap", mc);
    close_to(&v, "pl", mc / 1_645_594_000.0);
    let pl = get(&v, "pl");
    assert_eq!(pl.check, InputCheck::Corrected);
    assert!(
        pl.check_notes.iter().any(|n| n.contains("individual")),
        "{:?}",
        pl.check_notes
    );
    // CVC: zeroed in 9M26, filled in the other periods.
    let d = data(&all, CVC, "CVCB3", "ON", dec!(0.72), Decimal::ONE);
    assert_eq!(ltm(&d, items::NET_INCOME), dec!(-194_062_000));
    let v = compute(&d, BazinParams::default());
    close_to(
        &v,
        "pl",
        (525_591_097.0 - 4_792_509.0) * 0.72 / -194_062_000.0,
    );
    assert_eq!(get(&v, "roe").check, InputCheck::Corrected);
}

#[test]
fn klabin_unit_counts_five_shares_and_uses_the_individual_income() {
    let all = load();
    assert_eq!(unit_count("1 KLBN3 + 4 KLBN4"), 5);
    let d = data(&all, KLABIN, "KLBN11", "UNIT", dec!(16.94), dec!(5));
    assert_eq!(ltm(&d, items::NET_INCOME), dec!(69_421_000));
    assert_eq!(ltm(&d, items::NET_INCOME_TOTAL), dec!(451_225_000));
    let v = compute(&d, BazinParams::default());
    let mc = (6_241_478_850.0 - 124_802_017.0) * 16.94 / 5.0;
    close_to(&v, "market_cap", mc);
    close_to(&v, "pl", mc / 69_421_000.0);
    close_to(&v, "net_margin", 451_225_000.0 / 20_827_973_000.0);
    // ROIC capital with non-controlling interests (6.6 billion).
    let capital = 9_882_846_000.0 + 6_620_837_000.0 + 34_993_433_000.0 - 9_475_806_000.0;
    close_to(&v, "roic", 2_654_035_000.0 * 0.66 / capital);
}

#[test]
fn banks_use_the_central_bank_standard_individual_statements() {
    let all = load();
    let d = data(&all, ITAU, "ITUB4", "PN", dec!(40.07), Decimal::ONE);
    assert_eq!(d.subject.layout, Some(Layout::Bank));
    assert_eq!(ltm(&d, items::NET_INCOME), dec!(46_777_000_000));
    let v = compute(&d, BazinParams::default());
    let shares = (11_026_869.0 - 4_843.0) * 1000.0;
    let mc = shares * 40.07;
    close_to(&v, "pl", mc / 46_777_000_000.0);
    close_to(&v, "pvp", mc / 218_927_000_000.0);
    assert!(
        v.notes.iter().any(|n| n.contains("Banco Central")),
        "{:?}",
        v.notes
    );
    // Bank figures are a method, not a correction.
    assert_eq!(get(&v, "pl").check, InputCheck::Verified);
    assert!(get(&v, "ev").value.is_none());
}
