//! Períodos das demonstrações: exercício, acumulado no ano, trimestres e últimos 12 meses.
//!
//! LTM de um fluxo na data E (fim do último documento):
//! - se há exercício terminando em E, é ele;
//! - senão, `exercício anterior + acumulado até E − acumulado até E − 1 ano`
//!   (ex.: DFP 2025 + ITR 2T26 − 2T25, este último do comparativo do próprio ITR).
//!
//! Trimestres saem das diferenças entre acumulados do mesmo exercício (o 4º trimestre é o
//! exercício menos o acumulado de 9 meses). Datas comparadas com tolerância de 10 dias.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

use super::{Fact, PeriodType, items};
use crate::time::add_months;

const TOLERANCE_DAYS: i64 = 10;

pub fn near(a: NaiveDate, b: NaiveDate) -> bool {
    (a - b).num_days().abs() <= TOLERANCE_DAYS
}

fn flows<'a>(facts: &'a [Fact], item: &'a str) -> impl Iterator<Item = &'a Fact> + 'a {
    facts.iter().filter(move |f| {
        f.item == item && matches!(f.period_type, PeriodType::Annual | PeriodType::Ytd)
    })
}

fn days(f: &Fact) -> i64 {
    f.start.map(|s| (f.end - s).num_days()).unwrap_or(0)
}

/// Data de referência do LTM: o fim mais recente de um exercício ou acumulado do lucro (ou
/// da receita, ou de qualquer fluxo).
pub fn reference_date(facts: &[Fact]) -> Option<NaiveDate> {
    for item in [items::NET_INCOME, items::REVENUE] {
        if let Some(d) = flows(facts, item).map(|f| f.end).max() {
            return Some(d);
        }
    }
    facts
        .iter()
        .filter(|f| matches!(f.period_type, PeriodType::Annual | PeriodType::Ytd))
        .map(|f| f.end)
        .max()
}

/// Últimos 12 meses de um fluxo terminando em `at`.
pub fn ltm_at(facts: &[Fact], item: &str, at: NaiveDate) -> Option<Fact> {
    let fl: Vec<&Fact> = flows(facts, item).collect();
    let mk = |value: Decimal, document: String, base: &Fact| Fact {
        item: item.to_string(),
        period_type: PeriodType::Ltm,
        start: Some(add_months(at, -12).succ_opt().expect("data válida")),
        end: at,
        value,
        currency: base.currency.clone(),
        source: "calculado".into(),
        document,
        priority: 0,
    };
    if let Some(a) = fl
        .iter()
        .find(|f| f.period_type == PeriodType::Annual && near(f.end, at))
    {
        let doc = crate::i18n::t_args(
            "nucleo-pesquisa-doc-exercicio",
            &crate::i18n::args(&[("documento", a.document.clone().into())]),
        );
        return Some(mk(a.value, doc, a));
    }
    let c = fl
        .iter()
        .filter(|f| f.period_type == PeriodType::Ytd && near(f.end, at))
        .max_by_key(|f| days(f))?;
    let s = c.start?;
    let a = fl.iter().find(|f| {
        f.period_type == PeriodType::Annual && near(f.end, s.pred_opt().expect("data válida"))
    })?;
    let prev_end = add_months(at, -12);
    let prev_start = add_months(s, -12);
    let p = fl.iter().find(|f| {
        f.period_type == PeriodType::Ytd
            && near(f.end, prev_end)
            && f.start.is_some_and(|ps| near(ps, prev_start))
    })?;
    Some(mk(
        a.value + c.value - p.value,
        crate::i18n::t_args(
            "nucleo-pesquisa-doc-ltm",
            &crate::i18n::args(&[
                ("exercicio", a.document.clone().into()),
                ("ate", c.end.to_string().into()),
                ("ate_anterior", p.end.to_string().into()),
                ("documento", c.document.clone().into()),
                ("documento_anterior", p.document.clone().into()),
            ]),
        ),
        c,
    ))
}

/// LTM de todos os fluxos na data de referência do emissor (sem acumulado para o item,
/// usa o último exercício dele).
pub fn compute_ltm(facts: &[Fact]) -> Vec<Fact> {
    let Some(at) = reference_date(facts) else {
        return vec![];
    };
    let mut names: Vec<&str> = facts
        .iter()
        .filter(|f| items::is_flow(&f.item))
        .map(|f| f.item.as_str())
        .collect();
    names.sort();
    names.dedup();
    names
        .into_iter()
        .filter_map(|i| {
            ltm_at(facts, i, at).or_else(|| {
                // Sem os acumulados do ITR: fica o último exercício (12 meses até o fim dele),
                // se for recente (um item que a companhia deixou de informar não vale).
                let a = latest(facts, i, PeriodType::Annual)
                    .filter(|a| a.end >= add_months(at, -15))?;
                Some(Fact {
                    period_type: PeriodType::Ltm,
                    source: "calculado".into(),
                    document: crate::i18n::t_args(
                        "nucleo-pesquisa-doc-ultimo-exercicio",
                        &crate::i18n::args(&[("documento", a.document.clone().into())]),
                    ),
                    priority: 0,
                    ..a.clone()
                })
            })
        })
        .collect()
}

/// Fato mais recente de um item num tipo de período.
pub fn latest<'a>(facts: &'a [Fact], item: &str, pt: PeriodType) -> Option<&'a Fact> {
    facts
        .iter()
        .filter(|f| f.item == item && f.period_type == pt)
        .max_by_key(|f| f.end)
}

/// Saldo mais recente (até `at`).
pub fn instant_at<'a>(facts: &'a [Fact], item: &str, at: NaiveDate) -> Option<&'a Fact> {
    facts
        .iter()
        .filter(|f| f.item == item && f.period_type == PeriodType::Instant && f.end <= at)
        .max_by_key(|f| f.end)
}

/// Série anual (fim do exercício → fato), mais recente por último.
pub fn annual(facts: &[Fact], item: &str) -> BTreeMap<NaiveDate, Fact> {
    facts
        .iter()
        .filter(|f| f.item == item && f.period_type == PeriodType::Annual)
        .map(|f| (f.end, f.clone()))
        .collect()
}

/// Valor de um trimestre isolado.
#[derive(Debug, Clone, PartialEq)]
pub struct QuarterValue {
    pub start: NaiveDate,
    pub end: NaiveDate,
    /// "2T26" (trimestre do exercício e ano do fim).
    pub label: String,
    pub value: Decimal,
}

/// Trimestres isolados de um fluxo, a partir dos acumulados e do exercício.
pub fn quarterly(facts: &[Fact], item: &str) -> Vec<QuarterValue> {
    // Exercício (pelo início) → acumulados ordenados pelo fim.
    let mut by_year: BTreeMap<NaiveDate, Vec<&Fact>> = BTreeMap::new();
    for f in flows(facts, item) {
        if let Some(s) = f.start {
            by_year.entry(s).or_default().push(f);
        }
    }
    let mut out = vec![];
    for (start, mut cums) in by_year {
        cums.sort_by_key(|f| f.end);
        cums.dedup_by_key(|f| f.end);
        let mut prev: Option<&Fact> = None;
        for c in cums {
            let months = ((c.end.year() - start.year()) * 12 + c.end.month() as i32
                - start.month() as i32)
                + 1;
            let n = ((months + 1) / 3).clamp(1, 4);
            let q = match prev {
                Some(p) if n > 1 => {
                    let pm = ((p.end.year() - start.year()) * 12 + p.end.month() as i32
                        - start.month() as i32)
                        + 1;
                    if months - pm != 3 {
                        prev = Some(c);
                        continue;
                    }
                    QuarterValue {
                        start: p.end.succ_opt().expect("data válida"),
                        end: c.end,
                        label: String::new(),
                        value: c.value - p.value,
                    }
                }
                None if months <= 3 => QuarterValue {
                    start,
                    end: c.end,
                    label: String::new(),
                    value: c.value,
                },
                _ => {
                    prev = Some(c);
                    continue;
                }
            };
            out.push(QuarterValue {
                label: format!("{n}T{:02}", c.end.year() % 100),
                ..q
            });
            prev = Some(c);
        }
    }
    out.sort_by_key(|q| q.end);
    out
}

/// Taxa composta de crescimento anual entre dois valores positivos.
pub fn cagr(first: Decimal, last: Decimal, years: u32) -> Option<f64> {
    if first <= Decimal::ZERO || last <= Decimal::ZERO || years == 0 {
        return None;
    }
    let r = super::to_f64(last) / super::to_f64(first);
    Some(r.powf(1.0 / years as f64) - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn f(item: &str, pt: PeriodType, start: &str, end: &str, v: Decimal, doc: &str) -> Fact {
        Fact {
            item: item.into(),
            period_type: pt,
            start: (!start.is_empty()).then(|| d(start)),
            end: d(end),
            value: v,
            currency: "BRL".into(),
            source: "cvm".into(),
            document: doc.into(),
            priority: 2,
        }
    }

    #[test]
    fn ltm_from_dfp_and_itr() {
        let r = items::REVENUE;
        let facts = vec![
            f(
                r,
                PeriodType::Annual,
                "2025-01-01",
                "2025-12-31",
                dec!(1000),
                "DFP 2025",
            ),
            f(
                r,
                PeriodType::Ytd,
                "2026-01-01",
                "2026-06-30",
                dec!(600),
                "ITR 2T26",
            ),
            f(
                r,
                PeriodType::Ytd,
                "2025-01-01",
                "2025-06-30",
                dec!(450),
                "ITR 2T26 comparativo",
            ),
            f(
                r,
                PeriodType::Ytd,
                "2026-01-01",
                "2026-03-31",
                dec!(280),
                "ITR 1T26",
            ),
            f(
                r,
                PeriodType::Ytd,
                "2025-01-01",
                "2025-03-31",
                dec!(200),
                "ITR 1T25",
            ),
            f(
                r,
                PeriodType::Ytd,
                "2025-01-01",
                "2025-09-30",
                dec!(720),
                "ITR 3T25",
            ),
        ];
        assert_eq!(
            reference_date(&facts),
            Some(d("2026-06-30")),
            "no profit, falls back to revenue"
        );
        let ltm = ltm_at(&facts, r, d("2026-06-30")).unwrap();
        // 1000 + 600 − 450 = 1150
        assert_eq!(ltm.value, dec!(1150));
        assert_eq!(ltm.period_type, PeriodType::Ltm);
        assert_eq!(ltm.start, Some(d("2025-07-01")));
        assert!(ltm.document.contains("DFP 2025") && ltm.document.contains("ITR 2T26"));
        // Fiscal year end: LTM is the fiscal year itself.
        assert_eq!(
            ltm_at(&facts, r, d("2025-12-31")).unwrap().value,
            dec!(1000)
        );
        // Without the comparative, there is no LTM.
        assert!(ltm_at(&facts[..2], r, d("2026-06-30")).is_none());

        let q = quarterly(&facts, r);
        let labels: Vec<(&str, Decimal)> = q.iter().map(|q| (q.label.as_str(), q.value)).collect();
        assert_eq!(
            labels,
            vec![
                ("1T25", dec!(200)),
                ("2T25", dec!(250)),
                ("3T25", dec!(270)),
                ("4T25", dec!(280)),
                ("1T26", dec!(280)),
                ("2T26", dec!(320)),
            ]
        );
    }

    #[test]
    fn compute_ltm_uses_latest_reference() {
        let n = items::NET_INCOME;
        let facts = vec![
            f(
                n,
                PeriodType::Annual,
                "2025-01-01",
                "2025-12-31",
                dec!(100),
                "DFP",
            ),
            f(
                n,
                PeriodType::Ytd,
                "2026-01-01",
                "2026-03-31",
                dec!(30),
                "ITR",
            ),
            f(
                n,
                PeriodType::Ytd,
                "2025-01-01",
                "2025-03-31",
                dec!(20),
                "ITR",
            ),
            f(
                items::CFO,
                PeriodType::Annual,
                "2025-01-01",
                "2025-12-31",
                dec!(90),
                "DFP",
            ),
            f(
                items::EQUITY,
                PeriodType::Instant,
                "",
                "2026-03-31",
                dec!(500),
                "ITR",
            ),
        ];
        let ltm = compute_ltm(&facts);
        assert_eq!(ltm.len(), 2);
        let ni = ltm.iter().find(|f| f.item == n).unwrap();
        assert_eq!((ni.value, ni.end), (dec!(110), d("2026-03-31")));
        // The CFO with no ITR keeps the last fiscal year.
        let cfo = ltm.iter().find(|f| f.item == items::CFO).unwrap();
        assert_eq!((cfo.value, cfo.end), (dec!(90), d("2025-12-31")));
        assert!(cfo.document.starts_with("último exercício"));
        // An old fiscal year (item that stopped being reported) does not become LTM.
        let mut old = facts.clone();
        old.push(f(
            items::DIVIDENDS_DECLARED,
            PeriodType::Annual,
            "2018-01-01",
            "2018-12-31",
            dec!(5),
            "DFP 2018",
        ));
        assert!(
            compute_ltm(&old)
                .iter()
                .all(|f| f.item != items::DIVIDENDS_DECLARED)
        );
        assert_eq!(
            instant_at(&facts, items::EQUITY, d("2026-10-01"))
                .unwrap()
                .value,
            dec!(500)
        );
    }

    #[test]
    fn cagr_needs_positive_ends() {
        let g = cagr(dec!(100), dec!(161.051), 5).unwrap();
        assert!((g - 0.10).abs() < 1e-9);
        assert!(cagr(dec!(-1), dec!(10), 5).is_none());
        assert!(cagr(dec!(10), dec!(0), 5).is_none());
    }
}
