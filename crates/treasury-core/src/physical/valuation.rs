//! Motor de avaliação do patrimônio físico (puro, sem banco).
//!
//! O valor numa data parte de uma **âncora**: a última avaliação até a data ou, sem
//! avaliação, o custo de aquisição (compra + custos). A âncora e as benfeitorias de
//! capital posteriores a ela passam pela regra do bem:
//!
//! - `Flat`: sem estimativa (vale a última avaliação; benfeitorias posteriores pelo custo);
//! - `Index`: correção pelo índice escolhido (IPCA, IGP-M, INCC), pró-rata no mês;
//! - `Declining`: depreciação composta à taxa anual, `(1 − r)^anos` (veículos; também a
//!   ponte entre duas consultas à tabela FIPE);
//! - `StraightLine`: depreciação linear até o valor residual ao fim da vida útil.
//!
//! Todo resultado leva a fórmula no idioma da instalação e diz se é estimado.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::finance::inflation::{self, MonthlyRate};
use crate::finance::pow_dec;
use crate::i18n::{self, t, t_args};
use crate::portfolio::records::text;
use crate::time::add_months;

/// De onde vem o valor do bem numa data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ValueBasis {
    /// Última avaliação (laudo, FIPE, estimativa de mercado), sem ajuste.
    Appraisal,
    /// Estimativa automática a partir da âncora (índice, depreciação).
    Automatic,
    /// Sem avaliação nem estimativa: o custo.
    Cost,
    /// Vendido até a data (vale zero no patrimônio).
    Sold,
    /// Ainda não comprado na data.
    NotOwned,
}

/// Variações mensais de um índice de correção.
#[derive(Debug, Clone, Default)]
pub struct IndexTable {
    pub rates: Vec<MonthlyRate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexFactor {
    pub factor: Decimal,
    pub estimated: bool,
    /// Observação quando o índice não cobre o período todo.
    pub note: Option<String>,
}

fn month_start(y: i32, m: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, 1).expect("dia 1")
}

fn br_month(d: NaiveDate) -> String {
    format!("{:02}/{}", d.month(), d.year())
}

/// Data no idioma da instalação (`14/10/2026`, `Oct 14, 2026`).
pub fn br_date(d: NaiveDate) -> String {
    i18n::date(i18n::current(), d)
}

/// Decimal com `dp` casas no idioma da instalação (vírgula ou ponto).
pub fn br_dec(v: Decimal, dp: u32) -> String {
    crate::ai::alerts::decimal(i18n::current(), &v.round_dp(dp).normalize().to_string())
}

impl IndexTable {
    /// Fator acumulado entre `from` e `to`. Meses ainda não publicados ficam sem correção
    /// (e o resultado é estimado); sem dados, o fator é 1.
    pub fn factor(&self, from: NaiveDate, to: NaiveDate) -> IndexFactor {
        if to <= from {
            return IndexFactor {
                factor: Decimal::ONE,
                estimated: false,
                note: None,
            };
        }
        let (Some(first), Some(last)) = (self.rates.first(), self.rates.last()) else {
            return IndexFactor {
                factor: Decimal::ONE,
                estimated: true,
                note: Some(t("nucleo-carteira-bem-indice-sem-dados")),
            };
        };
        let start = month_start(first.year, first.month);
        let end = add_months(month_start(last.year, last.month), 1);
        let f = from.max(start);
        let t = to.min(end);
        let mut note = None;
        if t < to {
            note = Some(text(
                "nucleo-carteira-bem-indice-publicado-ate",
                &[("mes", br_month(month_start(last.year, last.month)))],
            ));
        } else if f > from {
            note = Some(text(
                "nucleo-carteira-bem-indice-desde",
                &[("mes", br_month(start))],
            ));
        }
        let clamped = note.is_some();
        if t <= f {
            return IndexFactor {
                factor: Decimal::ONE,
                estimated: true,
                note,
            };
        }
        match inflation::accumulated_factor(&self.rates, f, t) {
            Some(x) => IndexFactor {
                factor: x.factor,
                estimated: clamped || x.estimated,
                note,
            },
            None => IndexFactor {
                factor: Decimal::ONE,
                estimated: true,
                note: Some(i18n::t("nucleo-carteira-bem-indice-faltando")),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Rule {
    Flat,
    Index {
        /// Nome exibido (ex.: "IGP-M").
        name: String,
        table: IndexTable,
    },
    Declining {
        /// % a.a.
        rate_pct: Decimal,
        /// Ponte entre consultas à tabela FIPE (muda só o texto).
        fipe: bool,
    },
    StraightLine {
        life_years: u32,
        /// % do valor que resta ao fim da vida útil.
        residual_pct: Decimal,
    },
}

/// Uma avaliação registrada.
#[derive(Debug, Clone)]
pub struct Appraisal {
    pub date: NaiveDate,
    pub value: i64,
    /// Rótulo da fonte (ex.: "laudo", "FIPE").
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub currency: String,
    pub purchase_date: NaiveDate,
    /// Compra + custos de aquisição.
    pub initial_cost: i64,
    /// Benfeitorias que entram no custo (data, valor), em ordem.
    pub capital: Vec<(NaiveDate, i64)>,
    /// Avaliações em ordem de data.
    pub appraisals: Vec<Appraisal>,
    pub rule: Rule,
    /// Venda: (data, valor líquido).
    pub sale: Option<(NaiveDate, i64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub value: i64,
    pub basis: ValueBasis,
    pub estimated: bool,
    pub formula: String,
    /// Fonte curta (ex.: "avaliação · laudo", "IGP-M (BCB)", "depreciação 12% a.a.").
    pub source: String,
    pub anchor_date: NaiveDate,
    pub anchor_value: i64,
    /// Fator aplicado à âncora (1 sem estimativa).
    pub factor: Decimal,
}

fn years_between(from: NaiveDate, to: NaiveDate) -> Decimal {
    let days = (to - from).num_days().max(0);
    Decimal::from(days) / Decimal::new(36525, 2)
}

fn apply(amount: i64, f: Decimal) -> i64 {
    inflation::adjust(amount, f).max(0)
}

impl Model {
    /// Custo (compra + custos + benfeitorias de capital) até a data.
    pub fn cost_basis(&self, at: NaiveDate) -> i64 {
        self.initial_cost
            + self
                .capital
                .iter()
                .filter(|(d, _)| *d <= at)
                .map(|(_, v)| v)
                .sum::<i64>()
    }

    fn money(&self, v: i64) -> String {
        i18n::money(i18n::current(), v, &self.currency)
    }

    /// Fator da regra para um valor datado em `from` (âncora ou benfeitoria), até `to`.
    /// `appraisal_from`: a âncora é uma avaliação (na linear, a vida útil restante conta
    /// a partir dela).
    fn rule_factor(&self, from: NaiveDate, to: NaiveDate, appraisal_from: bool) -> IndexFactor {
        let plain = |factor: Decimal| IndexFactor {
            factor,
            estimated: to > from,
            note: None,
        };
        match &self.rule {
            Rule::Flat => IndexFactor {
                factor: Decimal::ONE,
                estimated: false,
                note: None,
            },
            Rule::Index { table, .. } => {
                let mut f = table.factor(from, to);
                f.estimated |= to > from;
                f
            }
            Rule::Declining { rate_pct, .. } => {
                let base = Decimal::ONE - *rate_pct / Decimal::ONE_HUNDRED;
                plain(pow_dec(base.max(Decimal::ZERO), years_between(from, to)))
            }
            Rule::StraightLine {
                life_years,
                residual_pct,
            } => {
                let life = Decimal::from(*life_years);
                let span = if appraisal_from {
                    life - years_between(self.purchase_date, from)
                } else {
                    life
                };
                if span <= Decimal::ZERO {
                    return IndexFactor {
                        factor: Decimal::ONE,
                        estimated: false,
                        note: Some(t("nucleo-carteira-bem-vida-encerrada")),
                    };
                }
                let used = (years_between(from, to) / span).min(Decimal::ONE);
                let lost = Decimal::ONE - *residual_pct / Decimal::ONE_HUNDRED;
                plain(Decimal::ONE - lost.max(Decimal::ZERO) * used)
            }
        }
    }

    fn rule_text(&self, from: NaiveDate, to: NaiveDate, f: Decimal) -> String {
        let s = match &self.rule {
            Rule::Flat => return String::new(),
            Rule::Index { name, .. } => text(
                "nucleo-carteira-bem-formula-indice",
                &[
                    ("indice", name.clone()),
                    ("de", br_date(from)),
                    ("ate", br_date(to)),
                    ("fator", br_dec(f, 4)),
                ],
            ),
            Rule::Declining { rate_pct, .. } => text(
                "nucleo-carteira-bem-formula-depreciacao",
                &[
                    ("taxa", br_dec(*rate_pct, 2)),
                    ("anos", br_dec(years_between(from, to), 1)),
                    ("fator", br_dec(f, 4)),
                ],
            ),
            Rule::StraightLine {
                life_years,
                residual_pct,
            } => t_args(
                "nucleo-carteira-bem-formula-linear",
                &i18n::args(&[
                    ("anos", i64::from(*life_years).into()),
                    ("residual", br_dec(*residual_pct, 2).into()),
                    ("fator", br_dec(f, 4).into()),
                ]),
            ),
        };
        format!(" {s}")
    }

    fn rule_source(&self) -> String {
        match &self.rule {
            Rule::Flat => t("nucleo-carteira-bem-fonte-sem-estimativa"),
            Rule::Index { name, .. } => text(
                "nucleo-carteira-bem-fonte-indice-bcb",
                &[("indice", name.clone())],
            ),
            Rule::Declining { rate_pct, fipe } => text(
                if *fipe {
                    "nucleo-carteira-bem-fonte-fipe-depreciacao"
                } else {
                    "nucleo-carteira-bem-fonte-depreciacao"
                },
                &[("taxa", br_dec(*rate_pct, 2))],
            ),
            Rule::StraightLine { life_years, .. } => t_args(
                "nucleo-carteira-bem-fonte-linear",
                &i18n::args(&[("anos", i64::from(*life_years).into())]),
            ),
        }
    }

    /// Valor estimado do bem na data `at`.
    pub fn value_at(&self, at: NaiveDate) -> Estimate {
        let zero = |basis, formula: String, source: String| Estimate {
            value: 0,
            basis,
            estimated: false,
            formula,
            source,
            anchor_date: at,
            anchor_value: 0,
            factor: Decimal::ONE,
        };
        if at < self.purchase_date {
            return zero(
                ValueBasis::NotOwned,
                text(
                    "nucleo-carteira-bem-formula-comprado",
                    &[("data", br_date(self.purchase_date))],
                ),
                t("nucleo-carteira-bem-fonte-sem-posse"),
            );
        }
        if let Some((d, v)) = self.sale
            && d <= at
        {
            return zero(
                ValueBasis::Sold,
                text(
                    "nucleo-carteira-bem-formula-vendido",
                    &[("data", br_date(d)), ("valor", self.money(v))],
                ),
                t("nucleo-carteira-bem-fonte-venda"),
            );
        }
        let anchor = self.appraisals.iter().rfind(|a| a.date <= at);
        let (anchor_date, anchor_value, is_appraisal) = match anchor {
            Some(a) => (a.date, a.value, true),
            None => (self.purchase_date, self.initial_cost, false),
        };
        let anchor_desc = match anchor {
            Some(a) => text(
                "nucleo-carteira-bem-formula-avaliacao",
                &[
                    ("data", br_date(a.date)),
                    ("fonte", a.source.clone()),
                    ("valor", self.money(a.value)),
                ],
            ),
            None => text(
                "nucleo-carteira-bem-formula-custo",
                &[
                    ("valor", self.money(self.initial_cost)),
                    ("data", br_date(self.purchase_date)),
                ],
            ),
        };
        let af = self.rule_factor(anchor_date, at, is_appraisal);
        let mut value = apply(anchor_value, af.factor);
        let mut estimated = af.estimated;
        let mut notes: Vec<String> = af.note.iter().cloned().collect();
        // Benfeitorias de capital depois da âncora (com a avaliação, as da mesma data já
        // estão no laudo).
        let later: Vec<&(NaiveDate, i64)> = self
            .capital
            .iter()
            .filter(|(d, _)| *d <= at && if is_appraisal { *d > anchor_date } else { true })
            .collect();
        let mut later_cost = 0;
        let mut later_value = 0;
        for (d, v) in &later {
            let from = (*d).max(self.purchase_date);
            let f = self.rule_factor(from, at, false);
            estimated |= f.estimated;
            if let Some(n) = f.note
                && !notes.contains(&n)
            {
                notes.push(n);
            }
            later_cost += v;
            later_value += apply(*v, f.factor);
        }
        value += later_value;
        let flat = matches!(self.rule, Rule::Flat);
        let basis = if !is_appraisal && flat {
            ValueBasis::Cost
        } else if is_appraisal && (flat || at == anchor_date) && later.is_empty() {
            ValueBasis::Appraisal
        } else {
            ValueBasis::Automatic
        };
        if basis == ValueBasis::Cost || (flat && !later.is_empty()) {
            estimated = true;
        }
        let mut formula = format!(
            "{anchor_desc}{}",
            self.rule_text(anchor_date, at, af.factor)
        );
        if !later.is_empty() {
            let what = t_args(
                "nucleo-carteira-bem-benfeitorias",
                &i18n::args(&[("n", (later.len() as i64).into())]),
            );
            let added = if flat || later_value == later_cost {
                text(
                    "nucleo-carteira-bem-formula-benfeitorias-custo",
                    &[("benfeitorias", what), ("valor", self.money(later_cost))],
                )
            } else {
                text(
                    "nucleo-carteira-bem-formula-benfeitorias-regra",
                    &[
                        ("benfeitorias", what),
                        ("custo", self.money(later_cost)),
                        ("valor", self.money(later_value)),
                    ],
                )
            };
            formula.push_str(&format!(" + {added}"));
        }
        if basis != ValueBasis::Appraisal || !later.is_empty() || !flat {
            formula.push_str(&format!(" = {}", self.money(value)));
        }
        if basis == ValueBasis::Cost {
            formula = text(
                "nucleo-carteira-bem-formula-sem-avaliacao",
                &[("formula", formula)],
            );
        }
        for n in &notes {
            formula.push_str(&format!(" ({n})"));
        }
        let source = match (basis, anchor) {
            (ValueBasis::Appraisal, Some(a)) => text(
                "nucleo-carteira-bem-fonte-avaliacao-de",
                &[("fonte", a.source.clone())],
            ),
            (ValueBasis::Cost, _) => t("nucleo-carteira-bem-fonte-custo"),
            (_, Some(a)) => text(
                "nucleo-carteira-bem-fonte-avaliacao-regra",
                &[("fonte", a.source.clone()), ("regra", self.rule_source())],
            ),
            (_, None) => self.rule_source(),
        };
        Estimate {
            value,
            basis,
            estimated,
            formula,
            source,
            anchor_date,
            anchor_value,
            factor: af.factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn model(rule: Rule) -> Model {
        Model {
            currency: "BRL".into(),
            purchase_date: d("2024-01-01"),
            initial_cost: 50_000_000,
            capital: vec![],
            appraisals: vec![],
            rule,
            sale: None,
        }
    }

    fn table(pct: Decimal, from: (i32, u32), months: u32) -> IndexTable {
        let mut rates = vec![];
        let (mut y, mut m) = from;
        for _ in 0..months {
            rates.push(MonthlyRate {
                year: y,
                month: m,
                pct,
                estimated: false,
            });
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
        IndexTable { rates }
    }

    #[test]
    fn index_correction_of_cost_and_improvements() {
        // 1% a month for 12 months: factor 1.01^12.
        let mut m = model(Rule::Index {
            name: "IGP-M".into(),
            table: table(dec!(1), (2024, 1), 24),
        });
        let e = m.value_at(d("2025-01-01"));
        let f = dec!(1.01).powi(12);
        assert_eq!(e.value, inflation::adjust(50_000_000, f));
        assert_eq!(e.basis, ValueBasis::Automatic);
        assert!(e.estimated);
        assert!(e.formula.contains("IGP-M"), "{}", e.formula);
        // An improvement 6 months earlier corrects by 1.01^6 and goes into the cost.
        m.capital.push((d("2024-07-01"), 10_000_000));
        let e2 = m.value_at(d("2025-01-01"));
        assert_eq!(
            e2.value,
            inflation::adjust(50_000_000, f) + inflation::adjust(10_000_000, dec!(1.01).powi(6))
        );
        assert_eq!(m.cost_basis(d("2025-01-01")), 60_000_000);
        assert_eq!(m.cost_basis(d("2024-06-30")), 50_000_000);
    }

    #[test]
    fn index_without_recent_months_is_estimated_with_note() {
        let m = model(Rule::Index {
            name: "INCC".into(),
            table: table(dec!(0.5), (2024, 1), 6),
        });
        let e = m.value_at(d("2025-01-01"));
        assert_eq!(e.value, inflation::adjust(50_000_000, dec!(1.005).powi(6)));
        assert!(e.estimated);
        assert!(e.formula.contains("publicado até 06/2024"), "{}", e.formula);
        // No data: stays at cost, estimated.
        let empty = model(Rule::Index {
            name: "INCC".into(),
            table: IndexTable::default(),
        });
        let e = empty.value_at(d("2025-01-01"));
        assert_eq!(e.value, 50_000_000);
        assert!(e.estimated && e.formula.contains("sem dados"));
    }

    #[test]
    fn appraisal_anchors_the_estimate() {
        let mut m = model(Rule::Index {
            name: "IPCA".into(),
            table: table(dec!(1), (2024, 1), 24),
        });
        m.appraisals.push(Appraisal {
            date: d("2025-01-01"),
            value: 70_000_000,
            source: "laudo".into(),
        });
        let on = m.value_at(d("2025-01-01"));
        assert_eq!(on.value, 70_000_000);
        assert_eq!(on.basis, ValueBasis::Appraisal);
        assert!(!on.estimated);
        // After the appraisal, corrects from it.
        let after = m.value_at(d("2025-04-01"));
        assert_eq!(
            after.value,
            inflation::adjust(70_000_000, dec!(1.01).powi(3))
        );
        assert_eq!(after.anchor_value, 70_000_000);
        assert!(after.source.contains("laudo"));
        // Before the appraisal, the anchor is the cost.
        assert_eq!(m.value_at(d("2024-01-01")).value, 50_000_000);
        // No rule: the appraisal holds, and the later improvement goes in at cost (estimated).
        m.rule = Rule::Flat;
        m.capital.push((d("2025-02-01"), 5_000_000));
        let flat = m.value_at(d("2025-06-01"));
        assert_eq!(flat.value, 75_000_000);
        assert!(flat.estimated);
        assert!(flat.formula.contains("pelo custo"), "{}", flat.formula);
    }

    #[test]
    fn vehicle_declining_balance() {
        let m = model(Rule::Declining {
            rate_pct: dec!(12),
            fipe: false,
        });
        // Two years (731 days) at 12% a year.
        let e = m.value_at(d("2026-01-01"));
        let years = Decimal::from(731) / dec!(365.25);
        let expected = inflation::adjust(50_000_000, pow_dec(dec!(0.88), years));
        assert_eq!(e.value, expected);
        assert!(
            (e.value - 38_720_000).abs() < 20_000,
            "≈ 0.88² × 500k: {}",
            e.value
        );
        assert!(e.formula.contains("12% a.a."), "{}", e.formula);
    }

    #[test]
    fn straight_line_until_residual() {
        let m = model(Rule::StraightLine {
            life_years: 5,
            residual_pct: dec!(10),
        });
        let half = m.value_at(d("2026-07-02"));
        // 2.5 years of 5: loses 45% (half of 90%).
        assert!((half.value - 27_500_000).abs() < 10_000, "{}", half.value);
        let end = m.value_at(d("2035-01-01"));
        assert_eq!(end.value, 5_000_000, "stops at the residual");
    }

    #[test]
    fn flat_without_appraisal_is_cost_and_sale_zeroes() {
        let mut m = model(Rule::Flat);
        let e = m.value_at(d("2025-01-01"));
        assert_eq!(e.basis, ValueBasis::Cost);
        assert_eq!(e.value, 50_000_000);
        assert!(e.estimated);
        assert_eq!(m.value_at(d("2023-12-31")).basis, ValueBasis::NotOwned);
        m.sale = Some((d("2025-06-01"), 60_000_000));
        assert_eq!(m.value_at(d("2025-05-31")).value, 50_000_000);
        let sold = m.value_at(d("2025-06-01"));
        assert_eq!((sold.value, sold.basis), (0, ValueBasis::Sold));
    }

    #[test]
    fn formula_follows_the_locale() {
        use crate::i18n::{Locale, set_current};
        let m = model(Rule::Declining {
            rate_pct: dec!(12),
            fipe: false,
        });
        set_current(Locale::En);
        let en = m.value_at(d("2025-01-01"));
        set_current(Locale::PtBr);
        assert!(
            en.formula.starts_with(
                "acquisition cost of R$500,000.00 on Jan 1, 2024 depreciated at 12% a year"
            ),
            "{}",
            en.formula
        );
        assert_eq!(en.source, "depreciation of 12% a year");
        let pt = m.value_at(d("2025-01-01"));
        assert!(
            pt.formula.starts_with(
                "custo de aquisição de R$ 500.000,00 em 01/01/2024 depreciado a 12% a.a."
            ),
            "{}",
            pt.formula
        );
    }
}
