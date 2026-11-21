//! Cálculo (docs/life-expectancy.md §4): curva de sobrevivência de uma geração a partir da
//! idade de hoje, pela diagonal dos anos (idade `a` hoje, ano `t`: a taxa da idade `k` é a do
//! ano `t + (k − a)`), e os números que saem dela.
//!
//! ```text
//! S(a) = 1;  S(x+1) = S(x) × (1 − q(x))
//! e(a) = 0,5 + Σ_{x>a} S(x)        (mortes no meio do ano)
//! ```
//!
//! Os limites (depois de 2100, depois dos 100) ficam na fonte das taxas ([`Mortality`]).

/// Fonte das taxas: q(ano, idade), a probabilidade de morrer entre `idade` e `idade + 1`.
pub trait Mortality {
    fn q(&self, year: i32, age: u32) -> f64;
}

/// Idade máxima da curva (bem depois de a sobrevivência virar zero na prática).
const MAX_AGE: u32 = 200;
/// Abaixo disso a sobrevivência conta como zero.
const TINY: f64 = 1e-12;

/// Sobrevivência de quem tem `from` anos hoje: `s[k]` = chance de chegar vivo aos `from + k`.
#[derive(Debug, Clone, PartialEq)]
pub struct Curve {
    pub from: u32,
    pub s: Vec<f64>,
}

/// Curva a partir da idade `age` no ano `year`, seguindo a geração.
pub fn curve(m: &dyn Mortality, age: u32, year: i32) -> Curve {
    let mut s = vec![1.0];
    let mut cur = 1.0f64;
    let mut k = 0u32;
    while age + k < MAX_AGE {
        let q = m.q(year + k as i32, age + k).clamp(0.0, 1.0);
        cur *= 1.0 - q;
        s.push(cur);
        if cur < TINY {
            break;
        }
        k += 1;
    }
    Curve { from: age, s }
}

/// Curva de período: as taxas de um ano só, a vida inteira (a expectativa "do jornal").
pub fn period_curve(m: &dyn Mortality, age: u32, year: i32) -> Curve {
    struct Fixed<'a>(&'a dyn Mortality, i32);
    impl Mortality for Fixed<'_> {
        fn q(&self, _year: i32, age: u32) -> f64 {
            self.0.q(self.1, age)
        }
    }
    curve(&Fixed(m, year), age, year)
}

impl Curve {
    /// Chance de chegar vivo à idade `age` (1 antes de hoje, 0 depois do fim).
    pub fn at(&self, age: u32) -> f64 {
        if age <= self.from {
            return 1.0;
        }
        self.s
            .get((age - self.from) as usize)
            .copied()
            .unwrap_or(0.0)
    }

    /// Daqui a `k` anos.
    pub fn after(&self, k: u32) -> f64 {
        self.s.get(k as usize).copied().unwrap_or(0.0)
    }

    /// Idade em que a sobrevivência cai a `p` (interpolada entre as idades inteiras).
    pub fn quantile(&self, p: f64) -> f64 {
        years_until(|k| self.after(k), self.s.len(), p) + f64::from(self.from)
    }

    /// Anos que ainda restam, em média: 0,5 + Σ_{k≥1} S(a+k).
    pub fn remaining_mean(&self) -> f64 {
        0.5 + self.s.iter().skip(1).sum::<f64>()
    }
}

/// Anos (fracionários) até `f(k)` cair a `p`, com `f(0) = 1` e `f` decrescente; `len` limita a
/// busca.
fn years_until(f: impl Fn(u32) -> f64, len: usize, p: f64) -> f64 {
    let mut prev = f(0);
    if prev <= p {
        return 0.0;
    }
    for k in 1..len.max(2) as u32 {
        let cur = f(k);
        if cur <= p {
            let span = prev - cur;
            let frac = if span > 0.0 { (prev - p) / span } else { 0.0 };
            return f64::from(k - 1) + frac;
        }
        prev = cur;
    }
    (len.max(1) - 1) as f64
}

/// Expectativa ao nascer de quem nasceu em `birth_year`, pela geração (§4.3).
pub fn cohort_e0(m: &dyn Mortality, birth_year: i32) -> f64 {
    curve(m, 0, birth_year).remaining_mean()
}

/// Expectativa ao nascer de período (as taxas de `year`).
pub fn period_e0(m: &dyn Mortality, year: i32) -> f64 {
    period_curve(m, 0, year).remaining_mean()
}

/// Chance de pelo menos um dos dois estar vivo daqui a `k` anos, supondo independência
/// (§4.4): 1 − (1 − S₁) × (1 − S₂).
pub fn either_alive(a: &Curve, b: &Curve, k: u32) -> f64 {
    1.0 - (1.0 - a.after(k)) * (1.0 - b.after(k))
}

/// Anos (fracionários) até a chance de pelo menos um dos dois estar vivo cair a `p`.
pub fn either_alive_until(a: &Curve, b: &Curve, p: f64) -> f64 {
    let len = a.s.len().max(b.s.len());
    years_until(|k| either_alive(a, b, k), len, p)
}
