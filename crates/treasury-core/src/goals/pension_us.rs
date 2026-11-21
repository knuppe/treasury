//! Social Security dos EUA (aposentadoria do trabalhador), para o planejador quando a família
//! mora nos EUA (`goals.retirement` com `us_ssa_calc`).
//!
//! Entra o PIA (o benefício na idade plena que o extrato da SSA mostra) ou o AIME (a média
//! mensal dos 35 melhores anos corrigidos, ÷ 420), de onde sai o PIA pela fórmula de 90%,
//! 32% e 15% nas faixas dos bend points. A idade plena (FRA) depende do ano de nascimento:
//! 66 para 1943–1954, mais 2 meses por ano de 1955 a 1959, 67 de 1960 em diante. Pedir antes
//! (a partir dos 62) reduz 5/9 de 1% por mês nos primeiros 36 meses e 5/12 de 1% nos
//! seguintes; depois da idade plena, 2/3 de 1% por mês (8% ao ano) até os 70. Direito com 40
//! créditos (até 4 por ano trabalhado).
//!
//! Fonte: SSA, Office of the Chief Actuary (fórmula do PIA, bend points de 2027 anunciados em
//! 14/10/2026; redução por antecipação e crédito por adiamento, 20 CFR 404.410 e 404.313).
//! Valores em dólares de hoje. Fora: benefício de cônjuge e de sobrevivente, o teste de renda
//! antes da idade plena e o imposto sobre o benefício.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::inss::{Estimate, Rule, add_months, birth_of, choose, fmt_years, years_between};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::time::fmt_date;

/// Bend points do PIA (centavos), com o ano de elegibilidade e a fonte.
pub struct Table {
    pub year: i32,
    pub bend1: i64,
    pub bend2: i64,
    pub source: &'static str,
}

/// Atualizar todo outubro (SSA, "Benefit Formula Bend Points").
pub const TABLE: Table = Table {
    year: 2027,
    bend1: 134_000,
    bend2: 807_700,
    source: "SSA, PIA bend points 2027 (14/10/2026)",
};

/// Créditos para ter direito.
const CREDITS_NEEDED: u32 = 40;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "UsSsaInput")]
pub struct Input {
    /// Data de nascimento (`YYYY-MM-DD`); sem ela, vale a idade de hoje do plano.
    #[ts(optional)]
    #[serde(default)]
    pub birth_date: Option<String>,
    /// Benefício mensal na idade plena (PIA) do extrato da SSA, em centavos de hoje.
    #[ts(optional, type = "number")]
    #[serde(default)]
    pub pia: Option<i64>,
    /// Média mensal dos 35 melhores anos corrigidos (AIME), se não houver o PIA.
    #[ts(optional, type = "number")]
    #[serde(default)]
    pub aime: Option<i64>,
    /// Créditos (quarters of coverage) até hoje; padrão 40.
    #[ts(optional)]
    #[serde(default)]
    pub credits: Option<u32>,
    /// Idade até a qual continua trabalhando (padrão: a da aposentadoria do plano).
    #[ts(optional)]
    #[serde(default)]
    pub work_until_age: Option<u32>,
}

/// PIA pela fórmula, truncado na dezena de centavos (como a SSA).
pub fn pia_of(aime: i64) -> i64 {
    let a = aime.max(0) as f64;
    let (b1, b2) = (TABLE.bend1 as f64, TABLE.bend2 as f64);
    let pia = 0.9 * a.min(b1) + 0.32 * (a.min(b2) - b1).max(0.0) + 0.15 * (a - b2).max(0.0);
    (pia as i64) / 10 * 10
}

/// Idade plena em meses pelo ano de nascimento.
pub fn full_retirement_months(birth_year: i32) -> u32 {
    match birth_year {
        ..=1937 => 65 * 12,
        1938..=1942 => 65 * 12 + 2 * (birth_year - 1937) as u32,
        1943..=1954 => 66 * 12,
        1955..=1959 => 66 * 12 + 2 * (birth_year - 1954) as u32,
        _ => 67 * 12,
    }
}

/// Fator do benefício ao pedir `months` meses depois (positivo) ou antes (negativo) da idade
/// plena.
pub fn claim_factor(months: i32) -> f64 {
    if months >= 0 {
        1.0 + months as f64 * 2.0 / 3.0 / 100.0
    } else {
        let m = -months as f64;
        1.0 - (m.min(36.0) * 5.0 / 9.0 + (m - 36.0).max(0.0) * 5.0 / 12.0) / 100.0
    }
}

pub fn estimate(
    input: &Input,
    current_age: Option<u32>,
    start_from_age: u32,
    today: NaiveDate,
) -> Result<Estimate> {
    let birth = birth_of(input.birth_date.as_deref(), current_age, today)?;
    let pia = match (input.pia, input.aime) {
        (Some(p), _) if p >= 0 => p,
        (None, Some(a)) if a >= 0 => pia_of(a),
        _ => return Err(Error::invalid(t("nucleo-orcamento-ssa-erro-pia"))),
    };
    // Quem nasceu em 1º de janeiro segue a idade plena do ano anterior (SSA).
    let birth_year = if birth.month() == 1 && birth.day() == 1 {
        birth.year() - 1
    } else {
        birth.year()
    };
    let fra = full_retirement_months(birth_year);
    let until = input.work_until_age.unwrap_or(start_from_age);
    let credits = input.credits.unwrap_or(CREDITS_NEEDED);
    // Primeira idade (meses) com 40 créditos, 4 por ano enquanto trabalha.
    let age_now = years_between(birth, today);
    let credit_age = if credits >= CREDITS_NEEDED {
        Some(0.0)
    } else {
        let years = (CREDITS_NEEDED - credits).div_ceil(4) as f64;
        (age_now + years <= until as f64 + 1e-9).then_some(age_now + years)
    };
    let fmt = |m: u32| fmt_years(m as f64 / 12.0);
    let mut rules = vec![];
    let mut push = |id: &str, name: String, requirement: String, months: u32| {
        let at = add_months(birth, months);
        let ok = credit_age.is_some_and(|c| c <= months as f64 / 12.0 + 1e-9);
        let monthly = ok.then(|| (pia as f64 * claim_factor(months as i32 - fra as i32)) as i64);
        // Idade inteira em que o plano recebe por esta regra (a de pedir, para cima).
        let start = months.div_ceil(12);
        rules.push(Rule {
            id: id.into(),
            name,
            requirement,
            eligible_date: ok.then(|| fmt_date(at)),
            eligible_age: ok.then(|| ((months as f64 / 12.0) * 10.0).round() / 10.0),
            monthly_at_eligibility: monthly,
            start_age: ok.then_some(start),
            monthly_at_start: ok
                .then(|| (pia as f64 * claim_factor((start * 12) as i32 - fra as i32)) as i64),
            coefficient_at_start: ok.then(|| {
                (claim_factor((start * 12) as i32 - fra as i32) * 1000.0).round() / 1000.0
            }),
        });
    };
    push(
        "early",
        t("nucleo-orcamento-ssa-regra-antecipada"),
        t_args(
            "nucleo-orcamento-ssa-req-antecipada",
            &args(&[
                ("plena", fmt(fra).into()),
                (
                    "corte",
                    pct(1.0 - claim_factor(62 * 12 - fra as i32)).into(),
                ),
            ]),
        ),
        62 * 12,
    );
    push(
        "full",
        t("nucleo-orcamento-ssa-regra-plena"),
        t_args(
            "nucleo-orcamento-ssa-req-plena",
            &args(&[("plena", fmt(fra).into())]),
        ),
        fra,
    );
    push(
        "delayed",
        t("nucleo-orcamento-ssa-regra-adiada"),
        t_args(
            "nucleo-orcamento-ssa-req-adiada",
            &args(&[(
                "acrescimo",
                pct(claim_factor(70 * 12 - fra as i32) - 1.0).into(),
            )]),
        ),
        70 * 12,
    );
    // Pedido na idade em que o plano para de trabalhar, se não for uma das três.
    let plan_age = start_from_age.clamp(62, 70);
    if ![62 * 12, fra, 70 * 12].contains(&(plan_age * 12)) {
        push(
            "plan_age",
            t_args(
                "nucleo-orcamento-ssa-regra-idade-plano",
                &args(&[("idade", plan_age.to_string().into())]),
            ),
            t("nucleo-orcamento-ssa-req-idade-plano"),
            plan_age * 12,
        );
    }
    // A escolhida: a do pedido na idade do plano (ou a mais próxima dela).
    let target = start_from_age.clamp(62, 70);
    let chosen = rules
        .iter()
        .filter(|r| r.start_age == Some(target))
        .max_by_key(|r| r.monthly_at_start)
        .cloned()
        .or_else(|| choose(&rules));
    let mut notes = vec![t_args(
        "nucleo-orcamento-ssa-nota-estimativa",
        &args(&[("fonte", TABLE.source.into())]),
    )];
    if input.pia.is_none() {
        notes.push(t_args(
            "nucleo-orcamento-ssa-nota-pia",
            &args(&[
                ("bend1", (TABLE.bend1 / 100).to_string().into()),
                ("bend2", (TABLE.bend2 / 100).to_string().into()),
            ]),
        ));
    }
    if credit_age.is_none() {
        notes.push(t("nucleo-orcamento-ssa-nota-creditos"));
    }
    notes.push(t("nucleo-orcamento-ssa-nota-fora"));
    Ok(Estimate {
        system: "us_ssa".into(),
        payments_per_year: 12,
        chosen: chosen.as_ref().map(|r| r.id.clone()),
        monthly: chosen.as_ref().and_then(|r| r.monthly_at_start),
        start_age: chosen.as_ref().and_then(|r| r.start_age),
        rules,
        average_used: input.aime.unwrap_or(0),
        table_year: TABLE.year,
        min_wage: 0,
        ceiling: 0,
        source: TABLE.source.into(),
        notes,
    })
}

/// Percentual com uma casa ("30" ou "6,7"), no idioma da instalação.
fn pct(f: f64) -> String {
    let v = (f * 1000.0).round() / 10.0;
    let s = if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    };
    crate::ai::alerts::decimal(crate::i18n::current(), &s)
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
    fn pia_formula_matches_the_ssa_example() {
        // SSA example with the 2026 bend points ($1,286 and $7,749): AIME of $5,825 →
        // PIA of $2,609.80. With the 2027 ones ($1,340 and $8,077): 90% × 1,340 + 32% × 4,485.
        assert_eq!(pia_of(582_500), 264_120);
        // Below the first bend point: 90%.
        assert_eq!(pia_of(100_000), 90_000);
    }

    #[test]
    fn full_age_and_claim_factors() {
        assert_eq!(full_retirement_months(1954), 66 * 12);
        assert_eq!(full_retirement_months(1957), 66 * 12 + 6);
        assert_eq!(full_retirement_months(1960), 67 * 12);
        // At 62 with full age 67: 60 months early → 30% less (SSA).
        assert!((claim_factor(-60) - 0.70).abs() < 1e-9);
        // Full age 66: 48 months early → 25% less.
        assert!((claim_factor(-48) - 0.75).abs() < 1e-9);
        // At 70 with full age 67: 36 months × 2/3% = 24% more.
        assert!((claim_factor(36) - 1.24).abs() < 1e-9);
    }

    #[test]
    fn rules_at_62_full_age_and_70() {
        let i = Input {
            birth_date: Some("1975-06-15".into()),
            pia: Some(250_000),
            aime: None,
            credits: None,
            work_until_age: None,
        };
        let e = estimate(&i, None, 65, d("2026-10-21")).unwrap();
        assert_eq!(e.system, "us_ssa");
        assert_eq!(e.payments_per_year, 12);
        assert_eq!(rule(&e, "early").monthly_at_eligibility, Some(175_000));
        assert_eq!(
            rule(&e, "full").eligible_date.as_deref(),
            Some("2042-06-15")
        );
        assert_eq!(rule(&e, "full").monthly_at_eligibility, Some(250_000));
        assert_eq!(rule(&e, "delayed").monthly_at_eligibility, Some(310_000));
        // Stop at 65: claim at 65, 24 months before full age → 13.3% less.
        assert_eq!(e.chosen.as_deref(), Some("plan_age"));
        assert_eq!(e.start_age, Some(65));
        assert_eq!(e.monthly, Some(216_666));
    }

    #[test]
    fn without_40_credits_there_is_no_benefit() {
        let i = Input {
            birth_date: Some("1990-01-10".into()),
            pia: Some(150_000),
            aime: None,
            credits: Some(12),
            work_until_age: Some(38),
        };
        // 36 years old today, stops at 38: 12 + 8 = 20 credits < 40.
        let e = estimate(&i, None, 38, d("2026-10-21")).unwrap();
        assert!(e.rules.iter().all(|r| r.eligible_date.is_none()));
        assert_eq!(e.monthly, None);
    }
}
