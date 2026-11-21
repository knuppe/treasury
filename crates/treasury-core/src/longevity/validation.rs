//! Validação das tábuas embutidas com números de fontes oficiais (docs/life-expectancy.md §9),
//! com as tolerâncias e as diferenças registradas em docs/life-expectancy-validation.md.
//!
//! - IBGE 2024 (período, Brasil): a referência embutida reproduz os números publicados, e a
//!   tábua de período da ONU para 2024 fica perto deles.
//! - ONS (Reino Unido): expectativa por geração dos nascidos em 2023 e a de período.
//! - Propriedades nas tábuas reais de todos os países: S decrescente, mediana acima da idade,
//!   casal ≥ cada um, coorte ≥ período.
//!
//! SSA (EUA, tábuas de coorte de 2024): pendente; o app não tem os números da SSA (ver o
//! documento de validação).

use super::data::{self, TableSex, Tables};
use super::math;

/// Números publicados: (sexo, e0, e60, e80) do IBGE 2024 (Tábuas Completas de Mortalidade
/// 2024; docs/life-expectancy.md §3.2: 76,6 / 73,3 / 79,9 ao nascer; 22,6 / 20,8 / 24,2 aos 60;
/// 8,3 e 9,5 aos 80).
const IBGE_2024: &[(TableSex, f64, f64, Option<f64>)] = &[
    (TableSex::Both, 76.6, 22.6, None),
    (TableSex::Male, 73.3, 20.8, Some(8.3)),
    (TableSex::Female, 79.9, 24.2, Some(9.5)),
];
/// A referência embutida contra o publicado (arredondado a uma casa).
const IBGE_ROUNDING: f64 = 0.06;
/// Período da ONU (2024) contra o IBGE 2024: métodos e dados diferentes (a ONU modela as taxas
/// do Brasil); diferença registrada no documento.
const WPP_VS_IBGE: f64 = 1.0;

/// ONS, base 2022 (docs/life-expectancy.md §3.3): nascidos em 2023, por geração, 86,7 meninos e
/// 90,0 meninas; período (2021–2023) 78,8 e 82,8.
const ONS_COHORT_2023: &[(TableSex, f64)] = &[(TableSex::Male, 86.7), (TableSex::Female, 90.0)];
const ONS_PERIOD: &[(TableSex, f64)] = &[(TableSex::Male, 78.8), (TableSex::Female, 82.8)];
/// Projeções diferentes (ONS 2022 contra ONU 2024): até 1,5 ano na coorte e 1 no período.
const ONS_COHORT_TOLERANCE: f64 = 1.5;
const ONS_PERIOD_TOLERANCE: f64 = 1.0;

fn tables() -> &'static Tables {
    let t = data::embedded().unwrap();
    assert!(!t.synthetic, "validação só com as tábuas reais (WPP 2024)");
    t
}

#[test]
fn ibge_reference_matches_the_published_table() {
    let t = tables();
    let ibge = t.ibge.as_ref().expect("referência do IBGE embutida");
    for (sex, e0, e60, e80) in IBGE_2024 {
        let tab = ibge.of(*sex);
        assert_eq!(tab.ex.len(), 91, "0 a 90+");
        assert!(
            (tab.ex[0] - e0).abs() <= IBGE_ROUNDING,
            "{sex:?} e0 {}",
            tab.ex[0]
        );
        assert!(
            (tab.ex[60] - e60).abs() <= IBGE_ROUNDING,
            "{sex:?} e60 {}",
            tab.ex[60]
        );
        if let Some(e80) = e80 {
            assert!(
                (tab.ex[80] - e80).abs() <= IBGE_ROUNDING,
                "{sex:?} e80 {}",
                tab.ex[80]
            );
        }
    }
}

#[test]
fn un_period_2024_is_close_to_ibge_2024() {
    let t = tables();
    let br = t.country("BR").unwrap();
    for (sex, e0, e60, _) in IBGE_2024 {
        let v = t.view(br, *sex);
        let p0 = math::period_e0(&v, 2024);
        let p60 = math::period_curve(&v, 60, 2024).remaining_mean();
        assert!(
            (p0 - e0).abs() <= WPP_VS_IBGE,
            "{sex:?}: ONU {p0:.2} × IBGE {e0}"
        );
        assert!(
            (p60 - e60).abs() <= WPP_VS_IBGE,
            "{sex:?}: ONU {p60:.2} × IBGE {e60}"
        );
    }
}

#[test]
fn uk_cohort_and_period_are_close_to_ons() {
    let t = tables();
    let gb = t.country("GB").unwrap();
    for (sex, ons) in ONS_COHORT_2023 {
        let e = t.cohort_e0(gb, *sex, 2023).unwrap();
        assert!(
            (e - ons).abs() <= ONS_COHORT_TOLERANCE,
            "{sex:?}: ONU {e:.2} × ONS {ons}"
        );
    }
    for (sex, ons) in ONS_PERIOD {
        let e = math::period_e0(&t.view(gb, *sex), 2024);
        assert!(
            (e - ons).abs() <= ONS_PERIOD_TOLERANCE,
            "{sex:?}: ONU {e:.2} × ONS {ons}"
        );
    }
}

/// As propriedades do §9 em todos os países e sexos.
#[test]
fn properties_hold_in_every_country() {
    let t = tables();
    let sexes = [TableSex::Both, TableSex::Female, TableSex::Male];
    let mut cohort_below_period = vec![];
    for (ci, code) in t.countries.iter().enumerate() {
        for s in sexes {
            let v = t.view(ci, s);
            for age in [0, 18, 41, 65, 90, 100, 105] {
                let c = math::curve(&v, age, 2026);
                assert!(c.s.windows(2).all(|w| w[1] <= w[0]), "{code} {s:?} {age}");
                assert!(c.quantile(0.5) > f64::from(age), "{code} {s:?} {age}");
                let other = math::curve(&t.view(ci, TableSex::Both), 38, 2026);
                for k in [0, 10, 30, 50] {
                    let both = math::either_alive(&c, &other, k);
                    assert!(both + 1e-12 >= c.after(k) && both + 1e-12 >= other.after(k));
                }
            }
            // Coorte ≥ período: nascidos de 2024 a 2026 contra a tábua de período de 2024.
            let period = math::period_e0(&v, 2024);
            for b in 2024..=2026 {
                let cohort = t.cohort_e0(ci, s, b).unwrap();
                if cohort + 1e-9 < period {
                    cohort_below_period.push(format!("{code} {s:?} {b}: {cohort} < {period:.2}"));
                }
            }
        }
    }
    assert!(cohort_below_period.is_empty(), "{cohort_below_period:#?}");
}

/// Os números do documento de validação (rode com `--nocapture` para atualizar a tabela).
#[test]
fn print_the_validation_table() {
    let t = tables();
    let br = t.country("BR").unwrap();
    let gb = t.country("GB").unwrap();
    let ibge = t.ibge.as_ref().unwrap();
    for (sex, ..) in IBGE_2024 {
        let v = t.view(br, *sex);
        println!(
            "BR {:?}: ONU período 2024 e0 {:.2} e60 {:.2} | IBGE e0 {:.2} e60 {:.2} | coorte 1985 {:.2} 2023 {:.2}",
            sex,
            math::period_e0(&v, 2024),
            math::period_curve(&v, 60, 2024).remaining_mean(),
            ibge.of(*sex).ex[0],
            ibge.of(*sex).ex[60],
            t.cohort_e0(br, *sex, 1985).unwrap(),
            t.cohort_e0(br, *sex, 2023).unwrap(),
        );
    }
    for (sex, _) in ONS_COHORT_2023 {
        println!(
            "GB {:?}: coorte 2023 {:.2} | período 2024 {:.2}",
            sex,
            t.cohort_e0(gb, *sex, 2023).unwrap(),
            math::period_e0(&t.view(gb, *sex), 2024),
        );
    }
}
