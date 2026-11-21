//! Estimativa da aposentadoria do INSS (RGPS) pelas regras da Reforma da Previdência
//! (EC 103/2019), para o planejador de aposentadoria (`goals.retirement` com `inss_calc`).
//!
//! Entram a idade (ou a data de nascimento), o sexo, o tempo de contribuição de hoje (e o de
//! 13/11/2019, para as transições), a média dos salários de contribuição desde 07/1994 e até
//! quando a pessoa continua contribuindo. Para cada regra, mês a mês: a primeira data em que
//! ela é cumprida e o benefício nessa data; e o benefício na idade em que o plano começa a
//! receber o INSS (a da aposentadoria, ou a da regra se for depois).
//!
//! Regras (art. da EC 103/2019):
//! - permanente (art. 19), para quem começou a contribuir depois da reforma: 65 anos (homem)
//!   ou 62 (mulher), com 20 ou 15 anos de contribuição;
//! - transição por idade (art. 18): 65 anos (homem) ou 62 (mulher; 60 em 2019, mais 6 meses
//!   por ano), com 15 anos de contribuição;
//! - pontos (art. 15): 35 anos de contribuição (homem) ou 30 (mulher) e idade + contribuição
//!   de 96 ou 86 pontos em 2019, mais 1 por ano até 105 ou 100;
//! - idade mínima progressiva (art. 16): 35 ou 30 anos de contribuição e 61 ou 56 anos de
//!   idade em 2019, mais 6 meses por ano até 65 ou 62;
//! - pedágio de 50% (art. 17): quem tinha mais de 33 ou 28 anos de contribuição na reforma;
//!   35 ou 30 anos mais metade do que faltava; benefício pelo fator previdenciário (não
//!   calculado aqui: depende da tábua do IBGE);
//! - pedágio de 100% (art. 20): 60 anos (homem) ou 57 (mulher), 35 ou 30 anos de
//!   contribuição mais o tempo que faltava na reforma; benefício de 100% da média.
//!
//! Benefício: 60% da média mais 2% por ano de contribuição acima de 20 (homem) ou 15
//! (mulher), exceto no pedágio de 100%; entre o salário mínimo e o teto do ano da tabela.
//! Valores em reais de hoje (o plano é em valores reais). É uma estimativa: períodos
//! especiais, professor, deficiência e direito adquirido antes da reforma ficam de fora.

use chrono::{Datelike, Months, NaiveDate};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::time::{fmt_date, parse_date};

/// Data da reforma (publicação da EC 103/2019).
pub const REFORM_DATE: (i32, u32, u32) = (2019, 11, 13);

/// Salário mínimo e teto do RGPS (centavos), com a fonte.
pub struct Table {
    pub year: i32,
    pub min_wage: i64,
    pub ceiling: i64,
    pub source: &'static str,
}

/// Tabela em uso. Atualizar todo janeiro (portaria interministerial dos reajustes; o piso é
/// o salário mínimo do decreto do ano).
pub const TABLE: Table = Table {
    year: 2026,
    min_wage: 162_100,
    ceiling: 847_555,
    source: "Portaria Interministerial MPS/MF nº 13, de 09/01/2026",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "InssSex")]
pub enum Sex {
    Male,
    Female,
}

/// Dados para calcular o INSS. Valores em centavos de hoje.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "InssInput")]
pub struct Input {
    pub sex: Sex,
    /// Data de nascimento (`YYYY-MM-DD`); sem ela, vale a idade de hoje do plano.
    #[ts(optional)]
    #[serde(default)]
    pub birth_date: Option<String>,
    /// Tempo de contribuição até hoje, em anos (aceita fração: 12,5 = 12 anos e 6 meses).
    pub contribution_years: f64,
    /// Tempo de contribuição em 13/11/2019 (padrão: o de hoje menos o tempo desde a reforma,
    /// como se tivesse contribuído sem parar; 0 = começou depois da reforma).
    #[ts(optional)]
    #[serde(default)]
    pub contribution_at_reform: Option<f64>,
    /// Média mensal dos salários de contribuição desde 07/1994, corrigidos.
    #[ts(type = "number")]
    pub average_salary: i64,
    /// Idade até a qual continua contribuindo (padrão: a da aposentadoria do plano).
    #[ts(optional)]
    #[serde(default)]
    pub contribute_until_age: Option<u32>,
}

/// Uma regra: quando ela é cumprida e quanto paga.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "InssRule")]
pub struct Rule {
    /// `permanent`, `age`, `points`, `progressive_age`, `toll_50` ou `toll_100`.
    pub id: String,
    pub name: String,
    /// Requisitos, em uma frase.
    pub requirement: String,
    /// Primeira data em que a regra é cumprida (`null` = não cumpre até os 100 anos).
    pub eligible_date: Option<String>,
    /// Idade nessa data (anos, com fração).
    pub eligible_age: Option<f64>,
    /// Benefício mensal se pedir nessa data (`null` = não calculado, ex.: fator
    /// previdenciário).
    #[ts(type = "number | null")]
    pub monthly_at_eligibility: Option<i64>,
    /// Idade inteira em que o plano começa a receber por esta regra (a da aposentadoria do
    /// plano ou a da regra, a que vier depois).
    pub start_age: Option<u32>,
    #[ts(type = "number | null")]
    pub monthly_at_start: Option<i64>,
    /// Percentual da média nessa idade (0,6 = 60%).
    pub coefficient_at_start: Option<f64>,
}

/// Estimativa do benefício público (INSS no Brasil, Social Security nos EUA, pensão de
/// velhice em Portugal): as mesmas colunas para os três.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "InssEstimate")]
pub struct Estimate {
    /// `br_inss`, `us_ssa` ou `pt_ss`.
    pub system: String,
    /// Pagamentos por ano (13 no INSS, com o abono anual; 12 nos EUA; 14 em Portugal). O
    /// plano usa o valor × pagamentos ÷ 12 como renda mensal.
    pub payments_per_year: u32,
    pub rules: Vec<Rule>,
    /// Regra escolhida: a que começa a pagar mais cedo na idade do plano (empate: a que paga
    /// mais).
    pub chosen: Option<String>,
    #[ts(type = "number | null")]
    pub monthly: Option<i64>,
    pub start_age: Option<u32>,
    /// Média usada (limitada ao teto).
    #[ts(type = "number")]
    pub average_used: i64,
    pub table_year: i32,
    /// Piso do benefício (0 = sem piso no cálculo).
    #[ts(type = "number")]
    pub min_wage: i64,
    /// Teto do benefício (0 = sem teto no cálculo).
    #[ts(type = "number")]
    pub ceiling: i64,
    pub source: String,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Permanent,
    Age,
    Points,
    ProgressiveAge,
    Toll50,
    Toll100,
}

impl Kind {
    fn id(self) -> &'static str {
        match self {
            Kind::Permanent => "permanent",
            Kind::Age => "age",
            Kind::Points => "points",
            Kind::ProgressiveAge => "progressive_age",
            Kind::Toll50 => "toll_50",
            Kind::Toll100 => "toll_100",
        }
    }
    fn name(self) -> String {
        t(match self {
            Kind::Permanent => "nucleo-orcamento-inss-regra-permanente",
            Kind::Age => "nucleo-orcamento-inss-regra-idade",
            Kind::Points => "nucleo-orcamento-inss-regra-pontos",
            Kind::ProgressiveAge => "nucleo-orcamento-inss-regra-idade-progressiva",
            Kind::Toll50 => "nucleo-orcamento-inss-regra-pedagio-50",
            Kind::Toll100 => "nucleo-orcamento-inss-regra-pedagio-100",
        })
    }
}

struct Person {
    male: bool,
    birth: NaiveDate,
    today: NaiveDate,
    contrib_today: f64,
    contrib_reform: f64,
    until_age: f64,
    average: i64,
}

/// Anos entre duas datas pelo calendário, como o INSS conta: meses completos mais a fração
/// do mês corrente (no aniversário a idade é exata; 64 anos e 6 meses = 64,5).
pub(super) fn years_between(a: NaiveDate, b: NaiveDate) -> f64 {
    if b <= a {
        return -((a - b).num_days() as f64) / 365.25;
    }
    let mut months = (b.year() - a.year()) * 12 + b.month() as i32 - a.month() as i32;
    if b.day() < a.day() {
        months -= 1;
    }
    let months = months.max(0) as u32;
    let anchor = add_months(a, months);
    let next = add_months(a, months + 1);
    let frac = (b - anchor).num_days() as f64 / (next - anchor).num_days().max(1) as f64;
    (months as f64 + frac) / 12.0
}

impl Person {
    fn age_on(&self, d: NaiveDate) -> f64 {
        years_between(self.birth, d)
    }
    /// Contribuição numa data futura: a de hoje mais o tempo até `until_age`.
    fn contrib_on(&self, d: NaiveDate) -> f64 {
        let age_now = self.age_on(self.today);
        let worked = (self.age_on(d).min(self.until_age) - age_now).max(0.0);
        self.contrib_today + worked
    }
    fn full(&self) -> f64 {
        if self.male { 35.0 } else { 30.0 }
    }
    /// Exigência da regra numa data; `None` = não cumpre.
    fn meets(&self, k: Kind, d: NaiveDate) -> bool {
        let age = self.age_on(d);
        let c = self.contrib_on(d);
        let y = (d.year() - 2019) as f64;
        // Pequena folga para o arredondamento dos meses.
        let eps = 1e-6;
        match k {
            Kind::Permanent => {
                age + eps >= if self.male { 65.0 } else { 62.0 }
                    && c + eps >= if self.male { 20.0 } else { 15.0 }
            }
            Kind::Age => {
                let min_age = if self.male {
                    65.0
                } else {
                    (60.0 + 0.5 * y).min(62.0)
                };
                age + eps >= min_age && c + eps >= 15.0
            }
            Kind::Points => {
                let need = if self.male {
                    (96.0 + y).min(105.0)
                } else {
                    (86.0 + y).min(100.0)
                };
                c + eps >= self.full() && age + c + eps >= need
            }
            Kind::ProgressiveAge => {
                let min_age = if self.male {
                    (61.0 + 0.5 * y).min(65.0)
                } else {
                    (56.0 + 0.5 * y).min(62.0)
                };
                c + eps >= self.full() && age + eps >= min_age
            }
            Kind::Toll50 => {
                let missing = self.full() - self.contrib_reform;
                missing < 2.0 && missing > 0.0 && c + eps >= self.full() + missing / 2.0
            }
            Kind::Toll100 => {
                let missing = (self.full() - self.contrib_reform).max(0.0);
                age + eps >= if self.male { 60.0 } else { 57.0 } && c + eps >= self.full() + missing
            }
        }
    }
    fn coefficient(&self, k: Kind, d: NaiveDate) -> Option<f64> {
        match k {
            Kind::Toll50 => None,
            Kind::Toll100 => Some(1.0),
            _ => {
                let base = if self.male { 20.0 } else { 15.0 };
                let extra = (self.contrib_on(d) - base).max(0.0).floor();
                Some(0.6 + 0.02 * extra)
            }
        }
    }
    fn benefit(&self, coef: Option<f64>) -> Option<i64> {
        coef.map(|c| {
            ((self.average as f64 * c).round() as i64).clamp(TABLE.min_wage, TABLE.ceiling)
        })
    }
}

pub(super) fn add_months(d: NaiveDate, n: u32) -> NaiveDate {
    d.checked_add_months(Months::new(n)).unwrap_or(d)
}

/// Data de nascimento informada ou a de quem tem `current_age` anos hoje.
pub(super) fn birth_of(
    birth_date: Option<&str>,
    current_age: Option<u32>,
    today: NaiveDate,
) -> Result<NaiveDate> {
    let birth = match birth_date {
        Some(b) => parse_date(b)?,
        None => {
            let age = current_age.ok_or_else(|| {
                Error::invalid(t("nucleo-orcamento-inss-erro-idade-ou-nascimento"))
            })?;
            today
                .checked_sub_months(Months::new(age * 12))
                .ok_or_else(|| Error::invalid(t("nucleo-orcamento-inss-erro-idade")))?
        }
    };
    if birth >= today {
        return Err(Error::invalid(t(
            "nucleo-orcamento-inss-erro-nascimento-futuro",
        )));
    }
    Ok(birth)
}

/// A regra escolhida: a que começa a pagar mais cedo na idade do plano (empate: a que paga
/// mais).
pub(super) fn choose(rules: &[Rule]) -> Option<Rule> {
    rules
        .iter()
        .filter(|r| r.start_age.is_some() && r.monthly_at_start.is_some())
        .min_by(|a, b| {
            a.start_age
                .cmp(&b.start_age)
                .then(b.monthly_at_start.cmp(&a.monthly_at_start))
        })
        .cloned()
}

/// Estimativa do INSS. `current_age` vale quando não há data de nascimento; `start_from_age`
/// é a idade em que o plano para de trabalhar (o benefício começa nela ou depois).
pub fn estimate(
    input: &Input,
    current_age: Option<u32>,
    start_from_age: u32,
    today: NaiveDate,
) -> Result<Estimate> {
    let birth = birth_of(input.birth_date.as_deref(), current_age, today)?;
    let age_now = years_between(birth, today);
    if input.contribution_years < 0.0 || input.contribution_years > (age_now - 12.0).max(0.0) + 1e-9
    {
        return Err(Error::invalid(t("nucleo-orcamento-inss-erro-contribuicao")));
    }
    if input.average_salary < 0 {
        return Err(Error::invalid(t(
            "nucleo-orcamento-inss-erro-media-negativa",
        )));
    }
    let reform =
        NaiveDate::from_ymd_opt(REFORM_DATE.0, REFORM_DATE.1, REFORM_DATE.2).expect("data");
    let since_reform = years_between(reform, today).max(0.0);
    let contrib_reform = input
        .contribution_at_reform
        .unwrap_or((input.contribution_years - since_reform).max(0.0))
        .clamp(0.0, input.contribution_years);
    let until = input.contribute_until_age.unwrap_or(start_from_age) as f64;
    let p = Person {
        male: input.sex == Sex::Male,
        birth,
        today,
        contrib_today: input.contribution_years,
        contrib_reform,
        until_age: until,
        average: input.average_salary.min(TABLE.ceiling),
    };
    let before_reform = contrib_reform > 0.0;
    let kinds: Vec<Kind> = if before_reform {
        vec![
            Kind::Age,
            Kind::Points,
            Kind::ProgressiveAge,
            Kind::Toll50,
            Kind::Toll100,
        ]
    } else {
        vec![Kind::Permanent]
    };
    let (full, base) = if p.male { (35, 20) } else { (30, 15) };
    let start_floor = add_months(birth, start_from_age * 12);
    let mut rules = vec![];
    for k in kinds {
        // Números como texto: anos e idades sem separador de milhar em nenhum idioma.
        let n = |v: i32| -> fluent_bundle::FluentValue<'static> { v.to_string().into() };
        let year = today.year();
        let requirement = match k {
            Kind::Permanent => t_args(
                "nucleo-orcamento-inss-req-permanente",
                &args(&[
                    ("idade", n(if p.male { 65 } else { 62 })),
                    ("contribuicao", n(base)),
                ]),
            ),
            Kind::Age => t_args(
                "nucleo-orcamento-inss-req-idade",
                &args(&[("idade", n(if p.male { 65 } else { 62 }))]),
            ),
            Kind::Points => t_args(
                "nucleo-orcamento-inss-req-pontos",
                &args(&[
                    ("contribuicao", n(full)),
                    (
                        "pontos",
                        n(if p.male {
                            (96 + year - 2019).min(105)
                        } else {
                            (86 + year - 2019).min(100)
                        }),
                    ),
                    ("ano", n(year)),
                    ("maximo", n(if p.male { 105 } else { 100 })),
                ]),
            ),
            Kind::ProgressiveAge => {
                let y = (year - 2019) as f64;
                let m = if p.male {
                    (61.0 + 0.5 * y).min(65.0)
                } else {
                    (56.0 + 0.5 * y).min(62.0)
                };
                t_args(
                    "nucleo-orcamento-inss-req-idade-progressiva",
                    &args(&[
                        ("contribuicao", n(full)),
                        ("idade", fmt_years(m).into()),
                        ("ano", n(year)),
                        ("maximo", n(if p.male { 65 } else { 62 })),
                    ]),
                )
            }
            Kind::Toll50 => t_args(
                "nucleo-orcamento-inss-req-pedagio-50",
                &args(&[("minimo", n(full - 2)), ("contribuicao", n(full))]),
            ),
            Kind::Toll100 => t_args(
                "nucleo-orcamento-inss-req-pedagio-100",
                &args(&[
                    ("idade", n(if p.male { 60 } else { 57 })),
                    ("contribuicao", n(full)),
                    (
                        "total",
                        fmt_years(full as f64 + (full as f64 - p.contrib_reform).max(0.0)).into(),
                    ),
                ]),
            ),
        };
        // Primeira data em que cumpre: mês a mês até os 100 anos e, no mês achado, dia a dia
        // (a idade e o tempo vencem num dia certo, não no dia do mês de hoje).
        let mut eligible = None;
        let mut d = today;
        let mut prev = None;
        while p.age_on(d) <= 100.0 {
            if p.meets(k, d) {
                let mut day = prev.map_or(d, |x: NaiveDate| x + chrono::Duration::days(1));
                while day < d && !p.meets(k, day) {
                    day += chrono::Duration::days(1);
                }
                eligible = Some(day);
                break;
            }
            prev = Some(d);
            d = add_months(d, 1);
        }
        let (start_age, monthly_at_start, coef_start) = match eligible {
            Some(e) => {
                let start = e.max(start_floor);
                // O simulador anda por idade inteira: a mais próxima.
                let age = p.age_on(start).round().max(start_from_age as f64) as u32;
                let at = add_months(birth, age * 12).max(start);
                let c = p.coefficient(k, at);
                (Some(age), p.benefit(c), c)
            }
            None => (None, None, None),
        };
        rules.push(Rule {
            id: k.id().into(),
            name: k.name(),
            requirement,
            eligible_date: eligible.map(fmt_date),
            eligible_age: eligible.map(|e| (p.age_on(e) * 10.0).round() / 10.0),
            monthly_at_eligibility: eligible.and_then(|e| p.benefit(p.coefficient(k, e))),
            start_age,
            monthly_at_start,
            coefficient_at_start: coef_start.map(|c| (c * 1000.0).round() / 1000.0),
        });
    }
    let chosen = choose(&rules);
    let mut notes = vec![t_args(
        "nucleo-orcamento-inss-nota-estimativa",
        &args(&[
            ("ano", TABLE.year.to_string().into()),
            ("fonte", TABLE.source.into()),
        ]),
    )];
    if before_reform && input.contribution_at_reform.is_none() {
        notes.push(t_args(
            "nucleo-orcamento-inss-nota-reforma-suposta",
            &args(&[("tempo", fmt_years(contrib_reform).into())]),
        ));
    }
    if !before_reform {
        notes.push(t("nucleo-orcamento-inss-nota-so-permanente"));
    }
    if input.average_salary > TABLE.ceiling {
        notes.push(t("nucleo-orcamento-inss-nota-teto"));
    }
    notes.push(t("nucleo-orcamento-inss-nota-13"));
    notes.push(t("nucleo-orcamento-inss-nota-fora"));
    Ok(Estimate {
        system: "br_inss".into(),
        payments_per_year: 13,
        chosen: chosen.as_ref().map(|r| r.id.clone()),
        monthly: chosen.as_ref().and_then(|r| r.monthly_at_start),
        start_age: chosen.as_ref().and_then(|r| r.start_age),
        rules,
        average_used: p.average,
        table_year: TABLE.year,
        min_wage: TABLE.min_wage,
        ceiling: TABLE.ceiling,
        source: TABLE.source.into(),
        notes,
    })
}

/// Anos e meses por extenso ("64 anos e 6 meses").
pub(super) fn fmt_years(v: f64) -> String {
    let y = v.floor() as i64;
    let m = ((v - y as f64) * 12.0).round() as i64;
    match m {
        0 | 12 => t_args(
            "nucleo-orcamento-inss-anos",
            &args(&[("anos", (y + m / 12).into())]),
        ),
        _ => t_args(
            "nucleo-orcamento-inss-anos-meses",
            &args(&[("anos", y.into()), ("meses", m.into())]),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    fn input(sex: Sex, birth: &str, contrib: f64, avg: i64) -> Input {
        Input {
            sex,
            birth_date: Some(birth.into()),
            contribution_years: contrib,
            contribution_at_reform: None,
            average_salary: avg,
            contribute_until_age: Some(100),
        }
    }

    fn rule<'a>(e: &'a Estimate, id: &str) -> &'a Rule {
        e.rules.iter().find(|r| r.id == id).unwrap()
    }

    #[test]
    fn man_by_points_or_progressive_age() {
        // 55-year-old man with 30 years of contribution in Oct/2026, contributing without a
        // break.
        let e = estimate(
            &input(Sex::Male, "1971-10-01", 30.0, 500_000),
            None,
            55,
            d("2026-10-18"),
        )
        .unwrap();
        // Points: 35 years of contribution at 60 (2031): 60 + 35 = 95 < 105 (points ceiling in
        // 2028); needs 105 → age + contribution rise 2 per year: 85 today → 105 in 10
        // years (age 65, 40 years of contribution).
        let pts = rule(&e, "points");
        assert_eq!(pts.eligible_age.map(|a| a.round()), Some(65.0), "{pts:?}");
        // Progressive age: 65 (the minimum age reaches 65 in 2027), on the birthday, with
        // 39 years and 11 months of contribution: 60% + 2% × 19 full years = 98% of the average.
        let prog = rule(&e, "progressive_age");
        assert_eq!(prog.eligible_date.as_deref(), Some("2036-10-01"));
        assert_eq!(prog.monthly_at_eligibility, Some(490_000));
        // 100% toll: age 60 and 35 + what was missing (35 − ~23 = ~12) → 47 of
        // contribution, at ~72: after the others.
        assert!(rule(&e, "toll_100").eligible_age.unwrap() > 65.0);
        assert!(e.rules.iter().all(|r| r.id != "permanent"));
        assert_eq!(e.start_age, Some(65));
        // At 65 there are still days left until 40 full years of contribution.
        assert_eq!(e.monthly, Some(490_000));
    }

    #[test]
    fn woman_close_to_retiring_gets_toll_100() {
        // Woman born in 1968 with 29 years of contribution at the reform (1 was missing): 100%
        // toll at 57 with 31 of contribution, 100% of the average.
        let mut i = input(Sex::Female, "1968-03-10", 35.9, 400_000);
        i.contribution_at_reform = Some(29.0);
        let e = estimate(&i, None, 50, d("2026-10-18")).unwrap();
        let t = rule(&e, "toll_100");
        assert!(t.eligible_date.as_deref().unwrap() <= "2026-10-18", "{t:?}");
        assert_eq!(t.monthly_at_eligibility, Some(400_000));
        // 50% toll: 1 year was missing (< 2): 30 + 0.5 → already met; benefit not calculated.
        let t50 = rule(&e, "toll_50");
        assert!(t50.eligible_date.is_some());
        assert!(t50.monthly_at_eligibility.is_none());
    }

    #[test]
    fn newcomer_uses_the_permanent_rule_and_the_floor_and_ceiling() {
        // Started in 2020: only the permanent rule; 62 years old and 15 of contribution (woman).
        let mut i = input(Sex::Female, "1990-01-01", 6.0, 100_000);
        i.contribution_at_reform = Some(0.0);
        i.contribute_until_age = Some(65);
        let e = estimate(&i, None, 62, d("2026-10-18")).unwrap();
        assert_eq!(e.rules.len(), 1);
        let p = rule(&e, "permanent");
        assert_eq!(p.eligible_age.map(|a| a.round()), Some(62.0));
        // 60% of R$ 1,000 is below the minimum: pays the minimum.
        assert_eq!(p.monthly_at_eligibility, Some(TABLE.min_wage));
        // Average above the ceiling: capped.
        let mut rich = i.clone();
        rich.average_salary = 3_000_000;
        let e = estimate(&rich, None, 62, d("2026-10-18")).unwrap();
        assert_eq!(e.average_used, TABLE.ceiling);
        assert!(e.monthly.unwrap() <= TABLE.ceiling);
    }

    #[test]
    fn stopping_early_delays_or_lowers_the_benefit() {
        let mut i = input(Sex::Male, "1981-05-05", 20.0, 600_000);
        i.contribute_until_age = Some(50);
        // Stops at 50 with ~25 of contribution: points and progressive age require 35, not
        // met; what is left is the age transition at 65 (15 of contribution), 60% + 2% × 5.
        let e = estimate(&i, None, 50, d("2026-10-18")).unwrap();
        assert!(rule(&e, "points").eligible_date.is_none());
        assert_eq!(e.chosen.as_deref(), Some("age"));
        assert_eq!(e.start_age, Some(65));
        let coef = rule(&e, "age").coefficient_at_start.unwrap();
        assert!((coef - 0.68).abs() < 1e-9, "{coef}");
    }

    #[test]
    fn transition_rules_of_a_man_who_joined_before_the_reform() {
        // Made-up situation, in the format of the Meu INSS simulation (the rules were checked
        // against a simulation on 21/10/2026): man with 18 years, 7 months and 4 days of
        // contribution today and 11 years, 8 months and 26 days on 13/11/2019, contributing
        // until 65.
        let ymd = |y: f64, m: f64, d: f64| y + (m + d / 30.4375) / 12.0;
        let mut i = input(Sex::Male, "1985-03-10", ymd(18.0, 7.0, 4.0), 500_000);
        i.contribution_at_reform = Some(ymd(11.0, 8.0, 26.0));
        i.contribute_until_age = Some(65);
        let e = estimate(&i, None, 65, d("2026-10-21")).unwrap();
        // The 2026 requirements that INSS shows.
        assert!(
            rule(&e, "points")
                .requirement
                .contains("103 pontos em 2026")
        );
        assert!(
            rule(&e, "progressive_age")
                .requirement
                .contains("64 anos e 6 meses de idade em 2026")
        );
        // 100% toll: 35 + (35 − 11y8m26d) = 58 years and 3 months.
        assert!(
            rule(&e, "toll_100")
                .requirement
                .contains("58 anos e 3 meses"),
            "{}",
            rule(&e, "toll_100").requirement
        );
        // Contributing only until 65, he does not reach 58 years and 3 months; the 50% toll
        // does not apply (he was under 33 at the reform).
        assert!(rule(&e, "toll_100").eligible_date.is_none());
        assert!(rule(&e, "toll_50").eligible_date.is_none());
        // Age transition on the day he turns 65 (not on today's day of the month).
        assert_eq!(rule(&e, "age").eligible_date.as_deref(), Some("2050-03-10"));
        assert_eq!(rule(&e, "age").eligible_age, Some(65.0));
        // 35 of contribution in ~2043 and 105 points (ceiling since 2028) when age +
        // contribution add up to 105: 2 × age − 23.0 = 105, ~64 years.
        let pts = rule(&e, "points");
        let at = pts.eligible_age.unwrap();
        assert!((63.9..=64.1).contains(&at), "{pts:?}");
        // 2026 floor and ceiling.
        assert_eq!(TABLE.min_wage, 162_100);
        assert_eq!(TABLE.ceiling, 847_555);
    }

    #[test]
    fn age_counts_by_the_calendar() {
        assert_eq!(years_between(d("1985-03-10"), d("2050-03-10")), 65.0);
        assert_eq!(years_between(d("1985-03-10"), d("2049-09-10")), 64.5);
        assert!(years_between(d("1985-03-10"), d("2050-03-09")) < 65.0);
    }
}
