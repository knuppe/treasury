//! Pensão de velhice da Segurança Social de Portugal (regime geral), para o planejador
//! quando a família mora em Portugal (`goals.retirement` com `pt_pension_calc`).
//!
//! Pelo Guia Prático da Pensão de Velhice (ISS, 2026) e o Decreto-Lei 187/2007:
//! - prazo de garantia de 15 anos com registo de remunerações;
//! - idade normal de acesso de 66 anos e 9 meses em 2026 e 66 anos e 11 meses em 2027
//!   (Portaria 476/2025/1; depois sobe com a esperança de vida aos 65, o app mantém a de
//!   2027); a idade pessoal desce 4 meses por ano de carreira acima de 40;
//! - pensão = remuneração de referência (RR, a média mensal da carreira em 14 pagamentos)
//!   × taxa global (2% por ano até 20 anos; com 21 ou mais, de 2,3% a 2% por escalões da RR
//!   em IAS), no máximo 40 anos;
//! - antecipação por flexibilização a partir dos 60 com 40 anos de carreira: menos 0,5% por
//!   mês antes da idade pessoal, e o fator de sustentabilidade (0,8237 em 2026) se os 40 anos
//!   não estavam feitos aos 60; carreiras muito longas (48 anos, ou 46 começando antes dos 17)
//!   a partir dos 60 sem corte;
//! - bonificação por mês com descontos depois da idade pessoal até os 70 (0,33% a 1% conforme
//!   a carreira), até 92% da RR;
//! - pensão mínima pela carreira (valores de 2026).
//!
//! Valores em euros de hoje; a pensão é paga 14 vezes por ano. Fora: a fórmula antiga (P1)
//! de quem descontou antes de 2002, regimes especiais, desemprego de longa duração, pensão
//! unificada e as regras de 2018 quando dão mais.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::inss::{Estimate, Rule, add_months, birth_of, choose, fmt_years, years_between};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::time::fmt_date;

/// IAS (centavos) e os valores do ano, com a fonte.
pub struct Table {
    pub year: i32,
    pub ias: i64,
    pub sustainability: f64,
    /// Pensão mínima por carreira: 15 a 20 anos, 21 a 30, 31 ou mais.
    pub minimum: [i64; 3],
    pub source: &'static str,
}

/// Atualizar todo janeiro (portaria do IAS, da idade de acesso e do fator de
/// sustentabilidade; valores mínimos das pensões).
pub const TABLE: Table = Table {
    year: 2026,
    ias: 53_713,
    sustainability: 0.8237,
    minimum: [35_780, 39_482, 49_352],
    source: "ISS, Guia Prático Pensão de Velhice (2026); Portaria 476/2025/1",
};

/// Idade normal de acesso (meses) para quem começa a receber no ano.
pub fn normal_age_months(year: i32) -> u32 {
    match year {
        ..=2023 => 66 * 12 + 4,
        2024 => 66 * 12 + 4,
        2025 => 66 * 12 + 7,
        2026 => 66 * 12 + 9,
        _ => 66 * 12 + 11,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "PtPensionInput")]
pub struct Input {
    /// Data de nascimento (`YYYY-MM-DD`); sem ela, vale a idade de hoje do plano.
    #[ts(optional)]
    #[serde(default)]
    pub birth_date: Option<String>,
    /// Anos civis com registo de remunerações até hoje (aceita fração).
    pub contribution_years: f64,
    /// Remuneração de referência: soma dos salários da carreira ÷ (14 × anos), em centavos
    /// de hoje.
    #[ts(type = "number")]
    pub reference_salary: i64,
    /// Idade até a qual continua descontando (padrão: a da aposentadoria do plano).
    #[ts(optional)]
    #[serde(default)]
    pub contribute_until_age: Option<u32>,
    /// Começou a descontar antes dos 17 (carreira muito longa com 46 anos).
    #[ts(optional)]
    #[serde(default)]
    pub started_before_17: Option<bool>,
}

/// Pensão estatutária pela fórmula (RR × taxa global), em centavos.
pub fn formula(rr: i64, years: u32) -> i64 {
    let n = years.min(40) as f64;
    let rr = rr.max(0) as f64;
    if years <= 20 {
        return (rr * 0.02 * n).round() as i64;
    }
    let ias = TABLE.ias as f64;
    let steps = [(1.1, 0.023), (2.0, 0.0225), (4.0, 0.022), (8.0, 0.021)];
    let mut total = 0.0;
    let mut low = 0.0;
    for (up, rate) in steps {
        let hi = up * ias;
        total += (rr.min(hi) - low).max(0.0) * rate * n;
        low = hi;
    }
    total += (rr - low).max(0.0) * 0.02 * n;
    total.round() as i64
}

/// Taxa mensal da bonificação (%) pela carreira.
fn bonus_rate(years: u32) -> f64 {
    match years {
        0..=14 => 0.0,
        15..=24 => 0.33,
        25..=34 => 0.5,
        35..=39 => 0.65,
        _ => 1.0,
    }
}

fn minimum(years: u32) -> i64 {
    match years {
        0..=14 => 0,
        15..=20 => TABLE.minimum[0],
        21..=30 => TABLE.minimum[1],
        _ => TABLE.minimum[2],
    }
}

struct Person {
    birth: NaiveDate,
    today: NaiveDate,
    contrib_today: f64,
    until_age: f64,
    rr: i64,
    before17: bool,
}

impl Person {
    fn months(&self, d: NaiveDate) -> f64 {
        years_between(self.birth, d) * 12.0
    }
    fn contrib_on(&self, d: NaiveDate) -> f64 {
        let now = years_between(self.birth, self.today);
        let worked = (years_between(self.birth, d).min(self.until_age) - now).max(0.0);
        self.contrib_today + worked
    }
    fn years_on(&self, d: NaiveDate) -> u32 {
        (self.contrib_on(d) + 1e-6).floor().max(0.0) as u32
    }
    /// Idade pessoal de acesso (meses) para quem começa a receber em `d`.
    fn personal_months(&self, d: NaiveDate) -> f64 {
        let normal = normal_age_months(d.year()) as f64;
        let extra = self.years_on(d).saturating_sub(40) as f64;
        (normal - 4.0 * extra).max(60.0 * 12.0)
    }
    fn meets(&self, k: Kind, d: NaiveDate) -> bool {
        let age = self.months(d) + 1e-6;
        let c = self.contrib_on(d) + 1e-6;
        c >= 15.0
            && match k {
                Kind::Normal => age >= self.personal_months(d),
                Kind::Flex => age >= 720.0 && c >= 40.0,
                Kind::LongCareer => age >= 720.0 && (c >= 48.0 || (self.before17 && c >= 46.0)),
            }
    }
    /// Pensão mensal (14 por ano) se começar em `d` pela regra.
    fn benefit(&self, k: Kind, d: NaiveDate) -> (i64, f64) {
        let years = self.years_on(d);
        let base = formula(self.rr, years);
        let age = self.months(d);
        let personal = self.personal_months(d);
        if age + 1e-6 < personal && k == Kind::Flex {
            let early = (personal - age).ceil();
            let mut f = (1.0 - 0.005 * early).max(0.0);
            // Fator de sustentabilidade se os 40 anos não estavam feitos aos 60.
            if self.contrib_on(add_months(self.birth, 720)) + 1e-6 < 40.0 {
                f *= TABLE.sustainability;
            }
            return ((base as f64 * f).round() as i64, f);
        }
        if age + 1e-6 < personal {
            // Carreira muito longa: sem corte.
            return (base.max(minimum(years)), 1.0);
        }
        // Bonificação: meses com descontos entre a idade pessoal e o início, até os 70.
        let worked_until = (self.until_age * 12.0).min(age).min(70.0 * 12.0);
        let months = (worked_until - personal).max(0.0).floor();
        let f = 1.0 + bonus_rate(years) * months / 100.0;
        let mut v = (base as f64 * f).round() as i64;
        if f > 1.0 {
            v = v.min(base.max((self.rr as f64 * 0.92).round() as i64));
        }
        (v.max(minimum(years)), f)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Normal,
    Flex,
    LongCareer,
}

impl Kind {
    fn id(self) -> &'static str {
        match self {
            Kind::Normal => "pt_normal",
            Kind::Flex => "pt_flex",
            Kind::LongCareer => "pt_long_career",
        }
    }
}

/// Primeira data em que `meets` vale: mês a mês até os 100 anos e, no mês achado, dia a dia.
fn first_date(p: &Person, k: Kind) -> Option<NaiveDate> {
    let mut d = p.today;
    let mut prev: Option<NaiveDate> = None;
    while years_between(p.birth, d) <= 100.0 {
        if p.meets(k, d) {
            let mut day = prev.map_or(d, |x| x + chrono::Duration::days(1));
            while day < d && !p.meets(k, day) {
                day += chrono::Duration::days(1);
            }
            return Some(day);
        }
        prev = Some(d);
        d = add_months(d, 1);
    }
    None
}

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
    if input.reference_salary < 0 {
        return Err(Error::invalid(t(
            "nucleo-orcamento-inss-erro-media-negativa",
        )));
    }
    let p = Person {
        birth,
        today,
        contrib_today: input.contribution_years,
        until_age: input.contribute_until_age.unwrap_or(start_from_age) as f64,
        rr: input.reference_salary,
        before17: input.started_before_17.unwrap_or(false),
    };
    let start_floor = add_months(birth, start_from_age * 12);
    let normal_at = first_date(&p, Kind::Normal);
    let mut rules = vec![];
    for k in [Kind::Normal, Kind::Flex, Kind::LongCareer] {
        let eligible = first_date(&p, k);
        // A antecipação só entra se vier antes da idade pessoal.
        if k != Kind::Normal && eligible.is_some_and(|e| normal_at.is_some_and(|n| e >= n)) {
            continue;
        }
        let (name, requirement) = match k {
            Kind::Normal => (
                t("nucleo-orcamento-pt-regra-normal"),
                t_args(
                    "nucleo-orcamento-pt-req-normal",
                    &args(&[
                        (
                            "idade",
                            fmt_years(normal_age_months(today.year()) as f64 / 12.0).into(),
                        ),
                        ("ano", today.year().to_string().into()),
                    ]),
                ),
            ),
            Kind::Flex => (
                t("nucleo-orcamento-pt-regra-flexibilizacao"),
                t("nucleo-orcamento-pt-req-flexibilizacao"),
            ),
            Kind::LongCareer => (
                t("nucleo-orcamento-pt-regra-carreira-longa"),
                t("nucleo-orcamento-pt-req-carreira-longa"),
            ),
        };
        let (start_age, monthly_at_start, coef) = match eligible {
            Some(e) => {
                let start = e.max(start_floor);
                let age = years_between(birth, start)
                    .round()
                    .max(start_from_age as f64) as u32;
                let at = add_months(birth, age * 12).max(start);
                let (v, f) = p.benefit(k, at);
                (Some(age), Some(v), Some((f * 1000.0).round() / 1000.0))
            }
            None => (None, None, None),
        };
        rules.push(Rule {
            id: k.id().into(),
            name,
            requirement,
            eligible_date: eligible.map(fmt_date),
            eligible_age: eligible.map(|e| (years_between(birth, e) * 10.0).round() / 10.0),
            monthly_at_eligibility: eligible.map(|e| p.benefit(k, e).0),
            start_age,
            monthly_at_start,
            coefficient_at_start: coef,
        });
    }
    let chosen = choose(&rules);
    let mut notes = vec![
        t_args(
            "nucleo-orcamento-pt-nota-estimativa",
            &args(&[
                ("ano", TABLE.year.to_string().into()),
                ("fonte", TABLE.source.into()),
            ]),
        ),
        t("nucleo-orcamento-pt-nota-14"),
        t("nucleo-orcamento-pt-nota-idade-futura"),
    ];
    // Carreira que começou antes de 2002: a Segurança Social mistura a fórmula antiga.
    let since_2002 = years_between(NaiveDate::from_ymd_opt(2002, 1, 1).expect("data"), today);
    if input.contribution_years > since_2002 + 1e-6 {
        notes.push(t("nucleo-orcamento-pt-nota-antes-2002"));
    }
    notes.push(t("nucleo-orcamento-pt-nota-fora"));
    Ok(Estimate {
        system: "pt_ss".into(),
        payments_per_year: 14,
        chosen: chosen.as_ref().map(|r| r.id.clone()),
        monthly: chosen.as_ref().and_then(|r| r.monthly_at_start),
        start_age: chosen.as_ref().and_then(|r| r.start_age),
        rules,
        average_used: input.reference_salary,
        table_year: TABLE.year,
        min_wage: TABLE.minimum[0],
        ceiling: 0,
        source: TABLE.source.into(),
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::parse_date;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    fn rule<'a>(e: &'a Estimate, id: &str) -> &'a Rule {
        e.rules.iter().find(|r| r.id == id).unwrap()
    }

    #[test]
    fn formula_by_ias_brackets() {
        // 20 anos ou menos: 2% por ano.
        assert_eq!(formula(100_000, 20), 40_000);
        // 30 anos e RR de 1.500 € (≈ 2,79 IAS): 1,1 IAS × 2,3% + 0,9 IAS × 2,25% + resto ×
        // 2,2%, × 30.
        let ias: f64 = 537.13;
        let rr = 1500.0;
        let expected = (1.1 * ias * 0.023 + 0.9 * ias * 0.0225 + (rr - 2.0 * ias) * 0.022) * 30.0;
        assert_eq!(formula(150_000, 30), (expected * 100.0).round() as i64);
        // A taxa global para nos 40 anos.
        assert_eq!(formula(150_000, 45), formula(150_000, 40));
    }

    #[test]
    fn normal_age_with_bonus_and_penalty() {
        // Nascida em 1970 com 30 anos de carreira hoje, descontando até os 67 (2037): idade
        // normal de 2027 em diante, 66 anos e 11 meses; carreira de ~40,6 anos então → a
        // pessoal fica igual (só desce acima de 40 anos completos: 41 → 4 meses).
        let i = Input {
            birth_date: Some("1970-04-01".into()),
            contribution_years: 30.0,
            reference_salary: 150_000,
            contribute_until_age: Some(67),
            started_before_17: None,
        };
        let e = estimate(&i, None, 67, d("2026-10-21")).unwrap();
        assert_eq!(e.system, "pt_ss");
        assert_eq!(e.payments_per_year, 14);
        let n = rule(&e, "pt_normal");
        assert_eq!(n.eligible_date.as_deref(), Some("2037-03-01"));
        assert_eq!(n.monthly_at_eligibility, Some(formula(150_000, 40)));
        // Antecipação: 40 anos de carreira aos 66 anos e ~7 meses, 5 meses antes da idade
        // pessoal (−2,5%) e com o fator de sustentabilidade (aos 60 tinha só ~33 anos).
        let f = rule(&e, "pt_flex");
        assert_eq!(f.eligible_age.map(|a| a.floor()), Some(66.0));
        let expected = formula(150_000, 40) as f64 * 0.975 * TABLE.sustainability;
        assert_eq!(f.monthly_at_eligibility, Some(expected.round() as i64));
        // Pedindo aos 67: 1 mês de bonificação de 1% (40 anos de carreira).
        assert_eq!(e.chosen.as_deref(), Some("pt_normal"));
        assert_eq!(e.start_age, Some(67));
        assert_eq!(
            e.monthly,
            Some((formula(150_000, 40) as f64 * 1.01).round() as i64)
        );
    }

    #[test]
    fn short_career_needs_15_years_and_gets_the_minimum() {
        let i = Input {
            birth_date: Some("1975-01-15".into()),
            contribution_years: 10.0,
            reference_salary: 60_000,
            contribute_until_age: Some(57),
            started_before_17: None,
        };
        // 51 anos hoje, para aos 57: 15 anos de carreira. 2% × 15 × 600 € = 180 € < mínimo
        // de 357,80 € (15 a 20 anos).
        let e = estimate(&i, None, 57, d("2026-10-21")).unwrap();
        let n = rule(&e, "pt_normal");
        assert_eq!(n.monthly_at_eligibility, Some(TABLE.minimum[0]));
        let mut short = i.clone();
        short.contribute_until_age = Some(53);
        let e = estimate(&short, None, 53, d("2026-10-21")).unwrap();
        assert!(e.rules.iter().all(|r| r.eligible_date.is_none()));
    }
}
