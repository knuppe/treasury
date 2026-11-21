//! Bancos (§5.1, CAMELS): indicadores prudenciais do Bacen IF.data (Basileia, capital
//! principal, alavancagem, inadimplência, cobertura, custo do crédito, eficiência) somados
//! aos da CVM e da cotação. Os bancos do IF.data sem ações na B3 entram como pares só nas
//! métricas prudenciais.

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;

use super::ScoreFlag;
use super::config::Methodology;
use super::engine::Peer;
use super::metrics::{MetricValue, Metrics};
use crate::error::Result;
use crate::research::ifdata::{self, items as it};
use crate::research::to_f64;

pub struct BankOut {
    pub metrics: Metrics,
    pub flags: Vec<ScoreFlag>,
    pub notes: Vec<String>,
    pub period: Option<NaiveDate>,
    /// Porte prudencial (grupo de pares) e o id do próprio banco entre os pares do IF.data.
    pub group: Option<String>,
    pub peer_id: Option<String>,
}

/// Nome do grupo de pares para mostrar, no idioma da instalação (o grupo em si é um
/// identificador estável em pt-BR, gravado com os pares do dia).
pub fn group_label(group: &str) -> String {
    match group {
        "Bancos grandes (S1–S2)" => crate::i18n::t("nucleo-score-banco-grupo-grandes"),
        "Bancos médios (S3)" => crate::i18n::t("nucleo-score-banco-grupo-medios"),
        "Bancos pequenos (S4)" => crate::i18n::t("nucleo-score-banco-grupo-pequenos"),
        other => other.to_string(),
    }
}

/// Grupo de pares pelo segmento prudencial (S1–S2 juntos: são poucos bancos).
pub fn size_group(segment: Option<&str>, cod_inst: &str) -> Option<String> {
    match segment {
        Some("S1") | Some("S2") => Some("Bancos grandes (S1–S2)".into()),
        Some("S3") => Some("Bancos médios (S3)".into()),
        Some("S4") => Some("Bancos pequenos (S4)".into()),
        None if cod_inst.starts_with('C') => Some("Bancos grandes (S1–S2)".into()),
        _ => None,
    }
}

type Series = BTreeMap<String, HashMap<String, Decimal>>;

/// Número com `dp` casas no separador decimal do idioma.
fn dec(x: f64, dp: usize) -> String {
    crate::ai::alerts::decimal(crate::i18n::current(), &format!("{x:.dp$}"))
}

const PRUDENTIAL: &[&str] = &[
    "basel_ratio",
    "cet1_ratio",
    "leverage_ratio",
    "npl90",
    "coverage",
    "cost_of_credit",
    "problem_assets_ratio",
    "loan_growth",
    "efficiency_ratio",
];

fn shift(period: &str, months: i32) -> Option<String> {
    let d = NaiveDate::parse_from_str(&format!("{period}-01"), "%Y-%m-%d").ok()?;
    let d = crate::time::add_months(d, months);
    Some(d.format("%Y-%m").to_string())
}

fn period_end(period: &str) -> Option<NaiveDate> {
    let d = NaiveDate::parse_from_str(&format!("{period}-01"), "%Y-%m-%d").ok()?;
    Some(crate::time::add_months(d, 1) - chrono::Duration::days(1))
}

/// Métricas prudenciais a partir da série do IF.data (puro).
pub fn prudential(series: &Series, name: &str) -> (Metrics, Option<String>) {
    let mut m = Metrics::new();
    let Some(p) = series
        .iter()
        .rev()
        .find(|(_, v)| v.contains_key(it::BASEL_RATIO))
        .map(|(k, _)| k.clone())
    else {
        for k in PRUDENTIAL {
            m.insert(
                (*k).into(),
                MetricValue::missing(crate::i18n::t("nucleo-score-banco-sem-dados")),
            );
        }
        return (m, None);
    };
    let at = |per: &str, item: &str| {
        series
            .get(per)
            .and_then(|v| v.get(item))
            .map(|x| to_f64(*x))
    };
    let now = |item: &str| at(&p, item);
    let year_ago = shift(&p, -12).unwrap_or_default();
    let src = format!("Bacen IF.data, {name}, {}", p);
    let label = crate::i18n::t_args(
        "nucleo-score-banco-trimestre",
        &crate::i18n::args(&[(
            "data",
            period_end(&p)
                .map(super::metrics::br)
                .unwrap_or_default()
                .into(),
        )]),
    );
    let mk = |v: Option<f64>, missing: &str| -> MetricValue {
        match v.filter(|x| x.is_finite()) {
            Some(x) => MetricValue {
                period: Some(label.clone()),
                period_end: period_end(&p),
                source: Some(src.clone()),
                ..MetricValue::ok(x)
            },
            None => MetricValue::missing(if missing.is_empty() {
                String::new()
            } else {
                crate::i18n::t(missing)
            }),
        }
    };
    let div = |a: Option<f64>, b: Option<f64>| match (a, b) {
        (Some(a), Some(b)) if b != 0.0 => Some(a / b),
        _ => None,
    };
    let twelve = |item: &str| ifdata::twelve_months(series, &p, item).map(to_f64);
    m.insert(
        "basel_ratio".into(),
        mk(now(it::BASEL_RATIO), "nucleo-score-banco-sem-basileia"),
    );
    m.insert(
        "cet1_ratio".into(),
        mk(
            now(it::CET1_RATIO),
            "nucleo-score-banco-sem-capital-principal",
        ),
    );
    m.insert(
        "leverage_ratio".into(),
        mk(
            now(it::LEVERAGE_RATIO),
            "nucleo-score-banco-sem-alavancagem",
        ),
    );
    m.insert(
        "npl90".into(),
        mk(
            div(now(it::NPL90), now(it::CREDIT_TOTAL)),
            "nucleo-score-banco-sem-inadimplencia",
        ),
    );
    m.insert(
        "problem_assets_ratio".into(),
        mk(
            div(now(it::PROBLEM_ASSETS), now(it::CREDIT_TOTAL)),
            "nucleo-score-banco-sem-ativos-problematicos",
        ),
    );
    m.insert(
        "coverage".into(),
        match (now(it::EXPECTED_LOSS), now(it::NPL90)) {
            (Some(_), Some(n)) if n <= 0.0 => mk(Some(99.0), ""),
            (e, n) => mk(div(e, n), "nucleo-score-banco-sem-provisao"),
        },
    );
    let avg_credit = match (
        now(it::CREDIT_PORTFOLIO),
        at(&year_ago, it::CREDIT_PORTFOLIO),
    ) {
        (Some(a), Some(b)) => Some((a + b) / 2.0),
        (a, _) => a,
    };
    m.insert(
        "cost_of_credit".into(),
        mk(
            div(twelve(it::PROVISION_EXPENSE), avg_credit),
            "nucleo-score-banco-sem-despesa-provisao",
        ),
    );
    m.insert(
        "loan_growth".into(),
        mk(
            div(now(it::CREDIT_TOTAL), at(&year_ago, it::CREDIT_TOTAL)).map(|x| x - 1.0),
            "nucleo-score-banco-sem-carteira-anterior",
        ),
    );
    let costs = match (twelve(it::PERSONNEL_EXPENSE), twelve(it::ADMIN_EXPENSE)) {
        (Some(a), Some(b)) => Some(a + b),
        _ => None,
    };
    // Receita antes das provisões (o resultado de intermediação do IF.data já desconta a
    // perda esperada): margem financeira + tarifas e serviços.
    let revenue = match (
        twelve(it::INTERMEDIATION_RESULT),
        twelve(it::FEE_INCOME),
        twelve(it::PROVISION_EXPENSE),
    ) {
        (Some(a), Some(b), p) => Some(a + b + p.unwrap_or(0.0)),
        _ => None,
    };
    m.insert(
        "efficiency_ratio".into(),
        mk(
            div(costs, revenue.filter(|r| *r > 0.0)),
            "nucleo-score-banco-sem-despesas-receitas",
        ),
    );
    let avg_equity = match (now(it::EQUITY), at(&year_ago, it::EQUITY)) {
        (Some(a), Some(b)) => Some((a + b) / 2.0),
        (a, _) => a,
    };
    let ni = twelve(it::NET_INCOME);
    if let Some(r) = div(ni, avg_equity.filter(|e| *e > 0.0)) {
        m.insert("roe".into(), mk(Some(r), ""));
    }
    let avg_assets = match (now(it::TOTAL_ASSETS), at(&year_ago, it::TOTAL_ASSETS)) {
        (Some(a), Some(b)) => Some((a + b) / 2.0),
        (a, _) => a,
    };
    if let Some(r) = div(ni, avg_assets) {
        m.insert("roa".into(), mk(Some(r), ""));
    }
    for k in ["coverage", "cost_of_credit", "efficiency_ratio"] {
        if m[k].missing.is_some() {
            m.get_mut(k).expect("inserido").note =
                Some(crate::i18n::t("nucleo-score-banco-nota-relatorios"));
        }
    }
    m.insert(
        "lcr".into(),
        MetricValue::missing(crate::i18n::t("nucleo-score-sem-fonte")),
    );
    (m, Some(p))
}

/// Séries de todas as instituições (uma consulta).
fn all_series(conn: &Connection) -> Result<HashMap<String, Series>> {
    let mut out: HashMap<String, Series> = HashMap::new();
    let mut stmt = conn.prepare("SELECT cod_inst, period, item, value FROM bank_prudential")?;
    for r in stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    })? {
        let (c, p, i, v) = r?;
        if let Ok(v) = v.parse::<Decimal>() {
            out.entry(c).or_default().entry(p).or_default().insert(i, v);
        }
    }
    Ok(out)
}

/// Métricas prudenciais do banco emissor (pelo CNPJ da companhia aberta) e os ajustes de
/// valuation que dependem do ROE.
pub fn metrics(
    conn: &Connection,
    m: &Methodology,
    issuer_id: &str,
    equity: &Metrics,
    _today: NaiveDate,
) -> Result<BankOut> {
    let mut notes = vec![];
    let inst = ifdata::institution_for_cnpj8(conn, issuer_id)?;
    let (mut ms, period) = match &inst {
        Some(i) => prudential(&ifdata::series(conn, &i.cod_inst)?, &i.name),
        None => (prudential(&Series::new(), "").0, None),
    };
    if period.is_none() {
        // O motivo exato (fonte desligada, nada baixado, CNPJ fora do cadastro…) vai para a
        // tela e para o feedback, no lugar do genérico.
        let why = ifdata::diagnose(conn, issuer_id)?;
        for v in ms.values_mut() {
            if v.missing.is_some() {
                v.missing = Some(why.clone());
            }
        }
    }
    match (&inst, &period) {
        (Some(i), Some(p)) => notes.push(crate::i18n::t_args(
            "nucleo-score-banco-nota-prudenciais",
            &crate::i18n::args(&[
                ("banco", i.name.clone().into()),
                ("periodo", p.clone().into()),
                ("convencao", ifdata::income_convention().into()),
            ]),
        )),
        _ => notes.push(crate::i18n::t(
            "nucleo-score-banco-nota-sem-correspondencia",
        )),
    }
    // ROE pelo IF.data quando houver; senão o da CVM (já em `equity`).
    let roe = ms
        .get("roe")
        .and_then(|v| v.value)
        .or_else(|| equity.get("roe").and_then(|v| v.value));
    if ms.get("roe").is_some_and(|v| v.value.is_none()) {
        ms.remove("roe");
    }
    if ms.get("roa").is_some_and(|v| v.value.is_none()) {
        ms.remove("roa");
    }
    let bv = &m.bank_valuation;
    ms.insert(
        "pb_vs_justified".into(),
        match (equity.get("pb").and_then(|v| v.value), roe) {
            (Some(pb), Some(r)) => {
                let fair = (r - bv.growth) / (bv.cost_of_equity - bv.growth);
                if fair <= 0.0 {
                    MetricValue::worst(crate::i18n::t("nucleo-score-banco-roe-abaixo"))
                } else {
                    MetricValue {
                        note: Some(crate::i18n::t_args(
                            "nucleo-score-banco-pvp-justo",
                            &crate::i18n::args(&[
                                ("pvp", dec(fair, 2).into()),
                                ("ke", dec(bv.cost_of_equity * 100.0, 0).into()),
                                ("g", dec(bv.growth * 100.0, 0).into()),
                            ]),
                        )),
                        ..MetricValue::ok(pb / fair)
                    }
                }
            }
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-banco-sem-pvp-roe")),
        },
    );
    // Crescimento do lucro pelos exercícios da CVM.
    if let Some(g) = equity.get("net_income_growth").cloned() {
        ms.insert("net_income_growth".into(), g);
    }
    let mut flags = vec![];
    if let Some(b) = ms.get("basel_ratio").and_then(|v| v.value)
        && let Some(f) = m.flags.get("bank_basel_low")
        && b < f.threshold.unwrap_or(0.115)
    {
        flags.push(ScoreFlag {
            key: "bank_basel_low".into(),
            label: f.label.clone(),
            severity: f.severity,
            cap: f.cap,
            penalty: f.penalty,
            detail: crate::i18n::t_args(
                "nucleo-score-banco-flag-basileia",
                &crate::i18n::args(&[("valor", dec(b * 100.0, 1).into())]),
            ),
        });
    }
    Ok(BankOut {
        metrics: ms,
        flags,
        notes,
        period: period.as_deref().and_then(period_end),
        group: inst
            .as_ref()
            .and_then(|i| size_group(i.segment.as_deref(), &i.cod_inst)),
        peer_id: inst.as_ref().map(|i| format!("ifdata:{}", i.cod_inst)),
    })
}

/// Bancos do IF.data sem ações na B3 como pares adicionais (só métricas prudenciais).
pub fn peers(conn: &Connection, _m: &Methodology, _today: NaiveDate) -> Result<Vec<Peer>> {
    let latest: Option<String> = conn
        .query_row(
            "SELECT max(period) FROM bank_prudential WHERE item = 'basel_ratio'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(None);
    let Some(latest) = latest else {
        return Ok(vec![]);
    };
    let listed: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT substr(id, 1, 8) FROM research_issuer WHERE layout = 'bank'")?;
        stmt.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let mut series = all_series(conn)?;
    let mut out = vec![];
    for (i, _) in ifdata::peers(conn, &latest)? {
        let Some(s) = series.remove(&i.cod_inst) else {
            continue;
        };
        let (mut metrics, _) = prudential(&s, &i.name);
        // Bancos listados já estão entre os pares com o ROE da CVM: aqui só o prudencial.
        if i.cnpj8.as_ref().is_some_and(|c| listed.contains(c)) {
            metrics.remove("roe");
            metrics.remove("roa");
        }
        out.push(Peer {
            id: format!("ifdata:{}", i.cod_inst),
            ticker: None,
            name: i.name.clone(),
            sector: size_group(i.segment.as_deref(), &i.cod_inst),
            segment: "banks".into(),
            metrics,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: f64) -> Decimal {
        Decimal::from_f64_retain(v).unwrap()
    }

    #[test]
    fn prudential_metrics_from_semester_to_date_series() {
        let mut s = Series::new();
        let mut put = |p: &str, kv: &[(&str, f64)]| {
            let e = s.entry(p.to_string()).or_default();
            for (k, v) in kv {
                e.insert((*k).to_string(), d(*v));
            }
        };
        // Lucro semestral até a data: 2025-12 (jul–dez) = 60; 2026-06 (jan–jun) = 70.
        put(
            "2025-06",
            &[
                (it::NET_INCOME, 50.0),
                (it::EQUITY, 900.0),
                (it::CREDIT_TOTAL, 4000.0),
                (it::CREDIT_PORTFOLIO, 4000.0),
                (it::PROVISION_EXPENSE, 40.0),
                (it::PERSONNEL_EXPENSE, 30.0),
                (it::ADMIN_EXPENSE, 20.0),
                (it::INTERMEDIATION_RESULT, 150.0),
                (it::FEE_INCOME, 50.0),
                (it::TOTAL_ASSETS, 10000.0),
            ],
        );
        put(
            "2025-12",
            &[
                (it::NET_INCOME, 60.0),
                (it::PROVISION_EXPENSE, 45.0),
                (it::PERSONNEL_EXPENSE, 32.0),
                (it::ADMIN_EXPENSE, 22.0),
                (it::INTERMEDIATION_RESULT, 160.0),
                (it::FEE_INCOME, 55.0),
            ],
        );
        put(
            "2026-06",
            &[
                (it::BASEL_RATIO, 0.16),
                (it::CET1_RATIO, 0.12),
                (it::LEVERAGE_RATIO, 0.08),
                (it::NET_INCOME, 70.0),
                (it::EQUITY, 1100.0),
                (it::CREDIT_TOTAL, 4600.0),
                (it::CREDIT_PORTFOLIO, 4600.0),
                (it::NPL90, 115.0),
                (it::PROBLEM_ASSETS, 230.0),
                (it::EXPECTED_LOSS, 230.0),
                (it::PROVISION_EXPENSE, 50.0),
                (it::PERSONNEL_EXPENSE, 35.0),
                (it::ADMIN_EXPENSE, 25.0),
                (it::INTERMEDIATION_RESULT, 170.0),
                (it::FEE_INCOME, 60.0),
                (it::TOTAL_ASSETS, 12000.0),
            ],
        );
        let (m, p) = prudential(&s, "Banco Teste");
        assert_eq!(p.as_deref(), Some("2026-06"));
        let v = |k: &str| m[k].value.unwrap();
        assert!((v("npl90") - 115.0 / 4600.0).abs() < 1e-12);
        assert!((v("coverage") - 2.0).abs() < 1e-12);
        assert!((v("loan_growth") - 0.15).abs() < 1e-12);
        // 12 meses = jun/26 + dez/25 (semestres).
        assert!((v("cost_of_credit") - 95.0 / 4300.0).abs() < 1e-12);
        assert!(
            (v("efficiency_ratio")
                - (35.0 + 25.0 + 32.0 + 22.0) / (170.0 + 60.0 + 160.0 + 55.0 + 95.0))
                .abs()
                < 1e-12
        );
        assert!((v("roe") - 130.0 / 1000.0).abs() < 1e-12);
        assert!(m["lcr"].missing.is_some());
        let (empty, none) = prudential(&Series::new(), "");
        assert!(none.is_none() && empty["basel_ratio"].missing.is_some());
    }
}
