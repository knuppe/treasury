//! Frameworks da literatura (§6) como funções puras sobre exercícios anuais:
//! Piotroski F-Score, Altman Z'', Beneish M-Score e o retorno sobre o capital da Magic
//! Formula. Não se aplicam a bancos e seguradoras (o motor nem chama).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Um exercício (valores em unidades da moeda; custos e saídas em módulo).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Year {
    pub end: Option<NaiveDate>,
    pub revenue: Option<f64>,
    pub cogs: Option<f64>,
    pub gross_profit: Option<f64>,
    pub sga: Option<f64>,
    pub ebit: Option<f64>,
    /// Resultado antes dos tributos (para estimar o EBIT quando ele não vem informado).
    pub ebt: Option<f64>,
    pub da: Option<f64>,
    pub net_income: Option<f64>,
    pub cfo: Option<f64>,
    /// Investimento em imobilizado e intangível (módulo).
    pub capex: Option<f64>,
    pub dividends_paid: Option<f64>,
    pub total_assets: Option<f64>,
    pub current_assets: Option<f64>,
    pub current_liabilities: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub cash: Option<f64>,
    pub receivables: Option<f64>,
    pub inventories: Option<f64>,
    pub ppe: Option<f64>,
    pub suppliers: Option<f64>,
    pub debt_st: Option<f64>,
    pub debt_lt: Option<f64>,
    pub leases: Option<f64>,
    pub gross_debt: Option<f64>,
    pub equity: Option<f64>,
    pub minority: Option<f64>,
    pub retained: Option<f64>,
    pub shares: Option<f64>,
    pub fin_income: Option<f64>,
    pub fin_expense: Option<f64>,
}

impl Year {
    pub fn gross(&self) -> Option<f64> {
        self.gross_profit
            .or_else(|| Some(self.revenue? - self.cogs?))
    }
    pub fn fcf(&self) -> Option<f64> {
        Some(self.cfo? - self.capex.unwrap_or(0.0))
    }
    pub fn ebitda(&self) -> Option<f64> {
        Some(self.ebit? + self.da.unwrap_or(0.0))
    }
    /// Dívida bruta + arrendamentos − caixa.
    pub fn net_debt(&self) -> Option<f64> {
        let debt = self
            .gross_debt
            .or_else(|| match (self.debt_st, self.debt_lt) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(0.0) + b.unwrap_or(0.0)),
            })?;
        Some(debt + self.leases.unwrap_or(0.0) - self.cash.unwrap_or(0.0))
    }
}

fn div(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    let (a, b) = (a?, b?);
    (b != 0.0 && a.is_finite() && b.is_finite()).then(|| a / b)
}

/// Um critério do Piotroski (ou uma variável do Beneish), com o resultado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ScoreFrameworkItem")]
pub struct FrameworkItem {
    pub label: String,
    /// Piotroski: 1 ou 0. Beneish/Altman: valor da variável.
    #[ts(type = "number | null")]
    pub value: Option<f64>,
    pub passed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ScoreFramework")]
pub struct Framework {
    pub key: String,
    pub label: String,
    #[ts(type = "number | null")]
    pub value: Option<f64>,
    /// Leitura (ex.: "zona segura", "forte").
    pub reading: Option<String>,
    pub items: Vec<FrameworkItem>,
    pub reference: String,
    pub note: Option<String>,
}

/// Piotroski F-Score entre `t` (atual), `p` (anterior) e `pp` (dois antes, para o ROA do
/// ano anterior sobre o ativo inicial). Precisa de pelo menos 7 dos 9 critérios.
pub fn piotroski(t: &Year, p: &Year, pp: Option<&Year>) -> Framework {
    let roa = |y: &Year, begin: Option<&Year>| {
        div(
            y.net_income,
            begin.and_then(|b| b.total_assets).or(y.total_assets),
        )
    };
    let roa_t = roa(t, Some(p));
    let roa_p = roa(p, pp);
    let avg_assets =
        |a: &Year, b: Option<&Year>| match (a.total_assets, b.and_then(|b| b.total_assets)) {
            (Some(x), Some(y)) => Some((x + y) / 2.0),
            (x, _) => x,
        };
    let lev = |y: &Year, b: Option<&Year>| div(y.debt_lt, avg_assets(y, b));
    let cr = |y: &Year| div(y.current_assets, y.current_liabilities);
    let gm = |y: &Year| div(y.gross(), y.revenue);
    let turn = |y: &Year, b: Option<&Year>| {
        div(y.revenue, b.and_then(|b| b.total_assets).or(y.total_assets))
    };
    let gt = |a: Option<f64>, b: Option<f64>| Some(a? > b?);
    // Rótulos: chaves do catálogo (`nucleo-score-piotroski-*`).
    let criteria: Vec<(&str, Option<bool>)> = vec![
        ("nucleo-score-piotroski-roa", roa_t.map(|r| r > 0.0)),
        ("nucleo-score-piotroski-fco", t.cfo.map(|c| c > 0.0)),
        ("nucleo-score-piotroski-roa-maior", gt(roa_t, roa_p)),
        (
            "nucleo-score-piotroski-accrual",
            gt(div(t.cfo, p.total_assets.or(t.total_assets)), roa_t),
        ),
        (
            "nucleo-score-piotroski-alavancagem",
            gt(lev(p, pp), lev(t, Some(p))),
        ),
        ("nucleo-score-piotroski-liquidez", gt(cr(t), cr(p))),
        (
            "nucleo-score-piotroski-sem-emissao",
            match (t.shares, p.shares) {
                (Some(a), Some(b)) => Some(a <= b * 1.001),
                _ => None,
            },
        ),
        ("nucleo-score-piotroski-margem", gt(gm(t), gm(p))),
        (
            "nucleo-score-piotroski-giro",
            gt(turn(t, Some(p)), turn(p, pp)),
        ),
    ];
    let known = criteria.iter().filter(|c| c.1.is_some()).count();
    let score = criteria.iter().filter(|c| c.1 == Some(true)).count() as f64;
    let value = (known >= 7).then_some(score);
    Framework {
        key: "piotroski".into(),
        label: "Piotroski F-Score".into(),
        value,
        reading: value.map(|v| {
            crate::i18n::t(if v >= 8.0 {
                "nucleo-score-leitura-forte"
            } else if v <= 2.0 {
                "nucleo-score-leitura-fraco"
            } else {
                "nucleo-score-leitura-intermediario"
            })
        }),
        items: criteria
            .into_iter()
            .map(|(l, r)| FrameworkItem {
                label: crate::i18n::t(l),
                value: r.map(|b| if b { 1.0 } else { 0.0 }),
                passed: r,
            })
            .collect(),
        reference: "Piotroski (2000), Journal of Accounting Research".into(),
        note: (known < 9).then(|| {
            crate::i18n::t_args(
                "nucleo-score-piotroski-nota",
                &crate::i18n::args(&[("n", (known as i64).into())]),
            )
        }),
    }
}

/// Altman Z'' (não industriais e emergentes; Altman, Hartzell e Peck, 1995). Lucros retidos
/// = reservas de lucros + lucros/prejuízos acumulados (convenção BR, ver ASSUMPTIONS.md).
pub fn altman_z2(y: &Year) -> Framework {
    let ta = y.total_assets;
    let x1 = div(
        match (y.current_assets, y.current_liabilities) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        },
        ta,
    );
    let x2 = div(y.retained, ta);
    let x3 = div(y.ebit, ta);
    let x4 = div(y.equity, y.total_liabilities);
    let value = match (x1, x2, x3, x4) {
        (Some(a), Some(b), Some(c), Some(d)) => Some(6.56 * a + 3.26 * b + 6.72 * c + 1.05 * d),
        _ => None,
    };
    let item = |l: &str, v: Option<f64>| FrameworkItem {
        label: l.into(),
        value: v,
        passed: None,
    };
    Framework {
        key: "altman_z2".into(),
        label: "Altman Z''".into(),
        value,
        reading: value.map(|z| {
            crate::i18n::t(if z > 2.6 {
                "nucleo-score-leitura-zona-segura"
            } else if z >= 1.1 {
                "nucleo-score-leitura-zona-cinza"
            } else {
                "nucleo-score-leitura-zona-estresse"
            })
        }),
        items: vec![
            item(&crate::i18n::t("nucleo-score-altman-x1"), x1),
            item(&crate::i18n::t("nucleo-score-altman-x2"), x2),
            item(&crate::i18n::t("nucleo-score-altman-x3"), x3),
            item(&crate::i18n::t("nucleo-score-altman-x4"), x4),
        ],
        reference: crate::i18n::t("nucleo-score-altman-referencia"),
        note: None,
    }
}

/// Beneish M-Score de 8 variáveis (t contra t−1). Variáveis sem dado entram como 1 (TATA
/// como 0), anotadas; com mais de 2 faltando não há valor.
pub fn beneish(t: &Year, p: &Year) -> Framework {
    let gm = |y: &Year| div(y.gross(), y.revenue);
    let aq = |y: &Year| {
        let ca = y.current_assets?;
        let ppe = y.ppe?;
        let ta = y.total_assets?;
        (ta != 0.0).then(|| 1.0 - (ca + ppe) / ta)
    };
    let dep = |y: &Year| {
        let d = y.da?;
        let ppe = y.ppe?;
        (d + ppe != 0.0).then(|| d / (d + ppe))
    };
    let lev = |y: &Year| {
        let cl = y.current_liabilities?;
        let ltd = y.debt_lt.unwrap_or(0.0);
        div(Some(cl + ltd), y.total_assets)
    };
    let r = |a: Option<f64>, b: Option<f64>| div(a, b);
    // Rótulos: chaves do catálogo (`nucleo-score-beneish-*`).
    let vars: Vec<(&str, Option<f64>, f64, f64)> = vec![
        (
            "nucleo-score-beneish-dsri",
            r(div(t.receivables, t.revenue), div(p.receivables, p.revenue)),
            0.920,
            1.0,
        ),
        ("nucleo-score-beneish-gmi", r(gm(p), gm(t)), 0.528, 1.0),
        ("nucleo-score-beneish-aqi", r(aq(t), aq(p)), 0.404, 1.0),
        (
            "nucleo-score-beneish-sgi",
            r(t.revenue, p.revenue),
            0.892,
            1.0,
        ),
        ("nucleo-score-beneish-depi", r(dep(p), dep(t)), 0.115, 1.0),
        (
            "nucleo-score-beneish-sgai",
            r(div(t.sga, t.revenue), div(p.sga, p.revenue)),
            -0.172,
            1.0,
        ),
        (
            "nucleo-score-beneish-tata",
            match (t.net_income, t.cfo) {
                (Some(n), Some(c)) => div(Some(n - c), t.total_assets),
                _ => None,
            },
            4.679,
            0.0,
        ),
        ("nucleo-score-beneish-lvgi", r(lev(t), lev(p)), -0.327, 1.0),
    ];
    let missing: Vec<String> = vars
        .iter()
        .filter(|v| v.1.is_none())
        .map(|v| crate::i18n::t(v.0))
        .collect();
    let m = -4.84
        + vars
            .iter()
            .map(|(_, v, coef, neutral)| coef * v.unwrap_or(*neutral))
            .sum::<f64>();
    let value = (missing.len() <= 2).then_some(m);
    Framework {
        key: "beneish".into(),
        label: "Beneish M-Score".into(),
        value,
        reading: value.map(|m| {
            crate::i18n::t(if m > -1.78 {
                "nucleo-score-leitura-manipulacao"
            } else {
                "nucleo-score-leitura-sem-manipulacao"
            })
        }),
        items: vars
            .iter()
            .map(|(l, v, _, _)| FrameworkItem {
                label: crate::i18n::t(l),
                value: *v,
                passed: None,
            })
            .collect(),
        reference: "Beneish (1999), Financial Analysts Journal".into(),
        note: (!missing.is_empty()).then(|| {
            crate::i18n::t_args(
                "nucleo-score-beneish-nota",
                &crate::i18n::args(&[("variaveis", missing.join(", ").into())]),
            )
        }),
    }
}

/// Retorno sobre o capital da Magic Formula: EBIT ÷ (capital de giro líquido + imobilizado).
pub fn magic_roc(ebit: Option<f64>, y: &Year) -> Option<f64> {
    let nwc = (y.current_assets? - y.cash.unwrap_or(0.0))
        - (y.current_liabilities? - y.debt_st.unwrap_or(0.0));
    let cap = nwc + y.ppe?;
    let ebit = ebit?;
    (cap > 0.0).then(|| ebit / cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn year(f: impl FnOnce(&mut Year)) -> Year {
        let mut y = Year::default();
        f(&mut y);
        y
    }

    #[test]
    fn altman_z2_by_hand() {
        // X1 = (600−400)/2000 = 0.1; X2 = 300/2000 = 0.15; X3 = 200/2000 = 0.1;
        // X4 = 900/1100 = 0.8182 → Z'' = 0.656 + 0.489 + 0.672 + 0.8591 = 2.6761.
        let y = year(|y| {
            y.total_assets = Some(2000.0);
            y.current_assets = Some(600.0);
            y.current_liabilities = Some(400.0);
            y.retained = Some(300.0);
            y.ebit = Some(200.0);
            y.equity = Some(900.0);
            y.total_liabilities = Some(1100.0);
        });
        let z = altman_z2(&y);
        assert!((z.value.unwrap() - 2.676_090_909).abs() < 1e-6);
        assert_eq!(z.reading.as_deref(), Some("zona segura"));
        assert_eq!(altman_z2(&Year::default()).value, None);
    }

    #[test]
    fn piotroski_counts_criteria() {
        let pp = year(|y| y.total_assets = Some(900.0));
        let p = year(|y| {
            y.net_income = Some(50.0);
            y.cfo = Some(60.0);
            y.total_assets = Some(1000.0);
            y.debt_lt = Some(300.0);
            y.current_assets = Some(400.0);
            y.current_liabilities = Some(400.0);
            y.shares = Some(100.0);
            y.revenue = Some(1000.0);
            y.gross_profit = Some(300.0);
        });
        let t = year(|y| {
            y.net_income = Some(80.0);
            y.cfo = Some(120.0);
            y.total_assets = Some(1050.0);
            y.debt_lt = Some(250.0);
            y.current_assets = Some(500.0);
            y.current_liabilities = Some(400.0);
            y.shares = Some(100.0);
            y.revenue = Some(1200.0);
            y.gross_profit = Some(400.0);
        });
        let f = piotroski(&t, &p, Some(&pp));
        assert_eq!(f.value, Some(9.0));
        assert_eq!(f.reading.as_deref(), Some("forte"));
        // Loss, negative cash flow and share issuance bring the score down.
        let bad = year(|y| {
            y.net_income = Some(-10.0);
            y.cfo = Some(-5.0);
            y.total_assets = Some(1100.0);
            y.debt_lt = Some(500.0);
            y.current_assets = Some(300.0);
            y.current_liabilities = Some(400.0);
            y.shares = Some(120.0);
            y.revenue = Some(900.0);
            y.gross_profit = Some(200.0);
        });
        let f = piotroski(&bad, &p, Some(&pp));
        assert_eq!(f.value, Some(1.0)); // only "operating cash flow ÷ assets > ROA"
        assert_eq!(f.reading.as_deref(), Some("fraco"));
    }

    #[test]
    fn beneish_neutral_company_is_below_threshold() {
        // Stable company: all indices = 1 and TATA = 0 → M = −4.84 + 0.92 + 0.528 +
        // 0.404 + 0.892 + 0.115 − 0.172 − 0.327 = −2.48.
        let y = year(|y| {
            y.revenue = Some(1000.0);
            y.cogs = Some(600.0);
            y.receivables = Some(100.0);
            y.current_assets = Some(400.0);
            y.ppe = Some(300.0);
            y.total_assets = Some(1000.0);
            y.da = Some(30.0);
            y.sga = Some(100.0);
            y.net_income = Some(80.0);
            y.cfo = Some(80.0);
            y.current_liabilities = Some(200.0);
            y.debt_lt = Some(100.0);
        });
        let b = beneish(&y, &y);
        assert!((b.value.unwrap() - (-2.48)).abs() < 1e-9);
        assert_eq!(b.note, None);
        // Receivables soaring and profit without cash: above −1.78.
        let t = year(|n| {
            *n = y.clone();
            n.receivables = Some(300.0);
            n.cfo = Some(-200.0);
        });
        let b = beneish(&t, &y);
        assert!(b.value.unwrap() > -1.78, "{:?}", b.value);
    }

    #[test]
    fn magic_roc_and_helpers() {
        let y = year(|y| {
            y.current_assets = Some(500.0);
            y.cash = Some(100.0);
            y.current_liabilities = Some(300.0);
            y.debt_st = Some(50.0);
            y.ppe = Some(750.0);
            y.gross_debt = Some(400.0);
            y.leases = Some(20.0);
        });
        // NWC = 400 − 250 = 150; capital = 900; ROC = 90/900.
        assert!((magic_roc(Some(90.0), &y).unwrap() - 0.1).abs() < 1e-12);
        assert_eq!(y.net_debt(), Some(320.0));
    }
}
