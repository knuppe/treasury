//! Os dois modelos de nota sobre os mesmos fatos: as leituras (`read_*`) medem o valor e
//! dizem como foi calculado; cada modelo dá os pontos (curvas lineares por trechos), o
//! estado (bom, atenção, ruim) e as sugestões.
//!
//! Os textos saem do catálogo (`locales/*/nucleo-saude.ftl`) no idioma da instalação, com os
//! números e os meses no formato dele.

use super::facts::{Facts, GRACE_DAYS, MIN_MONTHS, month_label};
use super::{
    HealthAmount, HealthBand, HealthDimension, HealthMetric, HealthModel, HealthReference,
    HealthReport, HealthStatus, HealthUnit, NATIONAL_AVERAGE, SOURCE_URL,
};
use crate::i18n::{self, args, t, t_args};
use fluent_bundle::FluentValue;

// ---------------------------------------------------------------------------
// Utilitários

/// Curva linear por trechos (`pts` em ordem crescente de x; fora dela, o extremo).
fn curve(x: f64, pts: &[(f64, f64)]) -> f64 {
    let (x0, y0) = pts[0];
    if x <= x0 {
        return y0;
    }
    for w in pts.windows(2) {
        let ((a, ya), (b, yb)) = (w[0], w[1]);
        if x <= b {
            return ya + (yb - ya) * (x - a) / (b - a);
        }
    }
    pts[pts.len() - 1].1
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Número com uma casa no idioma da instalação: `62.4` → `62,4` (pt-BR) ou `62.4` (en).
pub(crate) fn score_text(x: f64) -> String {
    crate::ai::alerts::decimal(i18n::current(), &format!("{x:.1}"))
}

/// Percentual com uma casa e o sinal de menos tipográfico: `−30,0%` (pt-BR), `−30.0%` (en).
fn percent(r: f64) -> String {
    crate::ai::alerts::pct(i18n::current(), r, 1).replace('-', "−")
}

fn months_text(x: f64) -> String {
    let key = if (x - 1.0).abs() < 0.05 {
        "nucleo-saude-valor-mes"
    } else {
        "nucleo-saude-valor-meses"
    };
    ta(key, &[("n", score_text(x).into())])
}

/// `n de total meses` (`key`: `nucleo-saude-valor-de-meses` ou `nucleo-saude-valor-em-dia`).
fn of(n: u32, total: u32, key: &str) -> String {
    ta(key, &[("n", n.into()), ("total", total.into())])
}

/// Texto do catálogo com argumentos (atalho).
fn ta(key: &str, list: &[(&'static str, FluentValue<'static>)]) -> String {
    t_args(key, &args(list))
}

/// Limiar de um estado: o valor precisa ser ≥ (ou ≤) o limite.
#[derive(Clone, Copy)]
enum Cmp {
    AtLeast(f64),
    AtMost(f64),
}

impl Cmp {
    fn ok(self, v: f64) -> bool {
        match self {
            Cmp::AtLeast(t) => v >= t - 1e-9,
            Cmp::AtMost(t) => v <= t + 1e-9,
        }
    }
}

/// Pontos e limiares de um indicador num modelo.
struct Scale {
    points: &'static [(f64, f64)],
    good: Cmp,
    ok: Cmp,
}

impl Scale {
    fn grade(&self, v: f64) -> (f64, HealthStatus) {
        let status = if self.good.ok(v) {
            HealthStatus::Good
        } else if self.ok.ok(v) {
            HealthStatus::Ok
        } else {
            HealthStatus::Bad
        };
        (round1(curve(v, self.points).clamp(0.0, 100.0)), status)
    }
}

/// Valor medido de um indicador (ou o motivo de não haver dados).
struct Reading {
    value: f64,
    /// Valor pontuado, quando difere do exibido (crédito caro: fração dos meses).
    graded: Option<f64>,
    text: String,
    computation: String,
    amounts: Vec<HealthAmount>,
}

type Read = Result<Reading, String>;

/// Descrição fixa de um indicador (os textos são chaves do catálogo).
struct Spec {
    key: &'static str,
    label: &'static str,
    unit: HealthUnit,
    target: &'static str,
    target_value: f64,
    explanation: &'static str,
    /// Sugestões quando não está bom (até 3 entram).
    advice: &'static [&'static str],
    /// Sugestão quando faltam dados.
    no_data: &'static str,
}

fn build(
    spec: &Spec,
    dimension: Option<&str>,
    weight: f64,
    reading: Read,
    scale: &Scale,
) -> HealthMetric {
    let (value, value_text, points, status, computation, amounts) = match reading {
        Ok(r) => {
            let (p, s) = scale.grade(r.graded.unwrap_or(r.value));
            (
                Some(r.value),
                Some(r.text),
                Some(p),
                s,
                r.computation,
                r.amounts,
            )
        }
        Err(why) => (None, None, None, HealthStatus::Unknown, why, vec![]),
    };
    let suggestions = match status {
        HealthStatus::Good => vec![],
        HealthStatus::Unknown => vec![t(spec.no_data)],
        HealthStatus::Ok => spec.advice.iter().take(2).map(|k| t(k)).collect(),
        HealthStatus::Bad => spec.advice.iter().take(3).map(|k| t(k)).collect(),
    };
    HealthMetric {
        key: spec.key.into(),
        label: t(spec.label),
        dimension: dimension.map(str::to_string),
        value,
        unit: spec.unit,
        value_text,
        target: t(spec.target),
        target_value: Some(spec.target_value),
        status,
        points,
        weight,
        explanation: t(spec.explanation),
        computation,
        suggestions,
        amounts,
    }
}

/// Valor da conta com o rótulo do catálogo (`key`).
fn amount(f: &Facts, key: &str, amount: i64) -> HealthAmount {
    HealthAmount {
        label: t(key),
        amount,
        currency: f.currency.clone(),
    }
}

fn need_months(f: &Facts) -> Result<(), String> {
    if f.enough() {
        Ok(())
    } else {
        Err(ta(
            "nucleo-saude-sem-dados-meses",
            &[("minimo", MIN_MONTHS.into()), ("n", f.n().into())],
        ))
    }
}

/// Média ponderada dos pontos dos indicadores com dados (`None` sem nenhum).
fn weighted(items: &[(Option<f64>, f64)]) -> Option<(f64, f64)> {
    let known: Vec<(f64, f64)> = items
        .iter()
        .filter_map(|(p, w)| p.map(|p| (p, *w)))
        .collect();
    let wsum: f64 = known.iter().map(|(_, w)| w).sum();
    if known.is_empty() || wsum <= 0.0 {
        return None;
    }
    let total: f64 = items.iter().map(|(_, w)| w).sum();
    Some((
        round1(known.iter().map(|(p, w)| p * w).sum::<f64>() / wsum),
        wsum / total,
    ))
}

fn band_of(bands: &[HealthBand], score: f64) -> Option<HealthBand> {
    let s = score.round().clamp(0.0, 100.0) as u32;
    bands.iter().find(|b| s >= b.min && s <= b.max).cloned()
}

fn bands(list: &[(&str, &str, u32, u32)]) -> Vec<HealthBand> {
    list.iter()
        .map(|(key, label, min, max)| HealthBand {
            key: (*key).into(),
            label: t(label),
            min: *min,
            max: *max,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Leituras

fn no_income(f: &Facts) -> String {
    ta(
        "nucleo-saude-sem-dados-receita",
        &[("periodo", f.period_label().into())],
    )
}

fn read_savings_rate(f: &Facts) -> Read {
    need_months(f)?;
    if f.income <= 0 {
        return Err(no_income(f));
    }
    let net = f.income - f.expense;
    let r = net as f64 / f.income as f64;
    Ok(Reading {
        graded: None,
        value: r,
        text: percent(r),
        computation: ta(
            "nucleo-saude-conta-poupanca",
            &[
                ("periodo", f.period_label().into()),
                ("n", f.n().into()),
                ("base", basis_label(f).into()),
            ],
        ),
        amounts: vec![
            amount(f, "nucleo-saude-quantia-receitas", f.income),
            amount(f, "nucleo-saude-quantia-despesas", f.expense),
            amount(f, "nucleo-saude-quantia-sobra", net),
        ],
    })
}

fn read_regular_saving(f: &Facts) -> Read {
    need_months(f)?;
    let with_income: Vec<_> = f.flows.iter().filter(|m| m.income > 0).collect();
    if with_income.is_empty() {
        return Err(no_income(f));
    }
    let n = with_income.len() as u32;
    let good = with_income
        .iter()
        .filter(|m| m.savings_rate.is_some_and(|r| r >= 0.10))
        .count() as u32;
    Ok(Reading {
        graded: None,
        value: good as f64 / n as f64,
        text: of(good, n, "nucleo-saude-valor-de-meses"),
        computation: ta(
            "nucleo-saude-conta-regular",
            &[("periodo", f.period_label().into())],
        ),
        amounts: vec![],
    })
}

fn read_reserve(f: &Facts) -> Read {
    if f.essential_months < MIN_MONTHS {
        return Err(ta(
            "nucleo-saude-sem-dados-gastos",
            &[
                ("minimo", MIN_MONTHS.into()),
                ("n", f.essential_months.into()),
            ],
        ));
    }
    if f.essential_avg <= 0 {
        return Err(t("nucleo-saude-sem-dados-essenciais"));
    }
    let months = f.reserve.max(0) as f64 / f.essential_avg as f64;
    let source = if f.reserve_from_goal {
        t("nucleo-saude-fonte-meta")
    } else {
        t("nucleo-saude-fonte-saldos")
    };
    Ok(Reading {
        graded: None,
        value: months,
        text: months_text(months),
        computation: ta(
            "nucleo-saude-conta-reserva",
            &[("fonte", source.into()), ("n", f.essential_months.into())],
        ),
        amounts: vec![
            amount(f, "nucleo-saude-quantia-reserva", f.reserve),
            amount(f, "nucleo-saude-quantia-gasto-essencial", f.essential_avg),
        ],
    })
}

fn read_debt_service(f: &Facts) -> Read {
    need_months(f)?;
    if f.income <= 0 {
        return Err(no_income(f));
    }
    let r = f.debt_service as f64 / f.income as f64;
    let what = if f.loans == 0 {
        t("nucleo-saude-contratos-nenhum")
    } else {
        ta("nucleo-saude-contratos", &[("n", f.loans.into())])
    };
    Ok(Reading {
        graded: None,
        value: r,
        text: percent(r),
        computation: ta(
            "nucleo-saude-conta-comprometimento",
            &[
                ("periodo", f.period_label().into()),
                ("contratos", what.into()),
            ],
        ),
        amounts: vec![
            amount(f, "nucleo-saude-quantia-parcelas", f.debt_service),
            amount(f, "nucleo-saude-quantia-receitas", f.income),
        ],
    })
}

fn read_housing(f: &Facts) -> Read {
    need_months(f)?;
    let Some(h) = f.housing else {
        return Err(t("nucleo-saude-sem-dados-moradia"));
    };
    if f.income <= 0 {
        return Err(no_income(f));
    }
    let r = h as f64 / f.income as f64;
    Ok(Reading {
        graded: None,
        value: r,
        text: percent(r),
        computation: ta(
            "nucleo-saude-conta-moradia",
            &[("periodo", f.period_label().into())],
        ),
        amounts: vec![
            amount(f, "nucleo-saude-quantia-moradia", h),
            amount(f, "nucleo-saude-quantia-receitas", f.income),
        ],
    })
}

fn read_expensive_credit(f: &Facts) -> Read {
    need_months(f)?;
    if f.checking_accounts == 0 && f.cards_tracked == 0 {
        return Err(t("nucleo-saude-sem-dados-credito"));
    }
    let months: std::collections::BTreeSet<&String> = f
        .overdraft_months
        .iter()
        .chain(f.revolving_months.iter())
        .collect();
    let n = months.len() as u32;
    let mut parts = vec![];
    if !f.overdraft_months.is_empty() {
        parts.push(ta(
            "nucleo-saude-credito-cheque",
            &[("meses", plural_months(f.overdraft_months.len()).into())],
        ));
    }
    if !f.revolving_months.is_empty() {
        parts.push(ta(
            "nucleo-saude-credito-rotativo",
            &[("meses", plural_months(f.revolving_months.len()).into())],
        ));
    }
    if !f.overdraft_suspect.is_empty() {
        parts.push(ta(
            "nucleo-saude-credito-suspeito",
            &[("contas", f.overdraft_suspect.join(", ").into())],
        ));
    }
    let detail = if parts.is_empty() {
        t("nucleo-saude-credito-nenhum")
    } else {
        parts.join("; ")
    };
    let mut computation = ta(
        "nucleo-saude-conta-credito",
        &[
            ("periodo", f.period_label().into()),
            ("dias", GRACE_DAYS.into()),
            ("detalhe", detail.into()),
        ],
    );
    if f.cards_tracked == 0 {
        computation.push(' ');
        computation.push_str(&t("nucleo-saude-conta-credito-sem-cartoes"));
    }
    Ok(Reading {
        // Pontuado pela fração dos meses (1 mês em 12 pesa como 2 em 24).
        graded: Some(n as f64 / f.n().max(1) as f64),
        value: n as f64,
        text: if n == 0 {
            t("nucleo-saude-nenhum-mes")
        } else {
            of(n, f.n(), "nucleo-saude-valor-de-meses")
        },
        computation,
        amounts: vec![],
    })
}

fn plural_months(n: usize) -> String {
    ta("nucleo-saude-n-meses", &[("n", n.into())])
}

fn read_net_worth(f: &Facts) -> Read {
    let pts = &f.net_worth;
    if pts.len() < (MIN_MONTHS + 1) as usize {
        return Err(ta(
            "nucleo-saude-sem-dados-patrimonio",
            &[("minimo", (MIN_MONTHS + 1).into()), ("n", pts.len().into())],
        ));
    }
    let (a, b) = (&pts[0], &pts[pts.len() - 1]);
    let delta = b.net - a.net;
    // Patrimônio inicial pequeno (ou negativo): a variação é medida contra o gasto do ano.
    let base = a.net.abs().max(f.expense).max(1);
    let g = delta as f64 / base as f64;
    let scale = if base == a.net.abs() {
        t("nucleo-saude-base-patrimonio-inicial")
    } else {
        t("nucleo-saude-base-despesas")
    };
    let worth_in = |month: &str, net: i64| HealthAmount {
        label: ta(
            "nucleo-saude-quantia-patrimonio-em",
            &[("mes", month_label(month).into())],
        ),
        amount: net,
        currency: f.currency.clone(),
    };
    Ok(Reading {
        graded: None,
        value: g,
        text: format!("{}{}", if g > 0.0 { "+" } else { "" }, percent(g)),
        computation: ta(
            "nucleo-saude-conta-patrimonio",
            &[
                ("de", month_label(&a.month).into()),
                ("ate", month_label(&b.month).into()),
                ("base", scale.into()),
            ],
        ),
        amounts: vec![
            worth_in(&a.month, a.net),
            worth_in(&b.month, b.net),
            amount(f, "nucleo-saude-quantia-variacao", delta),
        ],
    })
}

fn read_on_time(f: &Facts) -> Read {
    need_months(f)?;
    let on = f.installments_on_time + f.bills_on_time;
    let late = f.installments_late + f.bills_late;
    let total = on + late;
    if total == 0 {
        return Err(t("nucleo-saude-sem-dados-parcelas"));
    }
    Ok(Reading {
        graded: None,
        value: on as f64 / total as f64,
        text: of(on, total, "nucleo-saude-valor-em-dia"),
        computation: ta(
            "nucleo-saude-conta-em-dia",
            &[
                ("parcelas_em_dia", f.installments_on_time.into()),
                ("parcelas_atraso", f.installments_late.into()),
                ("faturas_pagas", f.bills_on_time.into()),
                ("faturas_nao", f.bills_late.into()),
                ("periodo", f.period_label().into()),
                ("dias", GRACE_DAYS.into()),
            ],
        ),
        amounts: vec![],
    })
}

fn read_budget(f: &Facts) -> Read {
    if f.budget_months == 0 {
        return Err(ta(
            "nucleo-saude-sem-dados-orcamento",
            &[("periodo", f.period_label().into())],
        ));
    }
    Ok(Reading {
        graded: None,
        value: f.budget_within as f64 / f.budget_months as f64,
        text: of(
            f.budget_within,
            f.budget_months,
            "nucleo-saude-valor-de-meses",
        ),
        computation: ta(
            "nucleo-saude-conta-orcamento",
            &[
                ("periodo", f.period_label().into()),
                ("base", basis_label(f).into()),
            ],
        ),
        amounts: vec![],
    })
}

fn read_positive_months(f: &Facts) -> Read {
    need_months(f)?;
    let active: Vec<_> = f
        .flows
        .iter()
        .filter(|m| m.income != 0 || m.expense != 0)
        .collect();
    if active.is_empty() {
        return Err(t("nucleo-saude-sem-dados-movimento"));
    }
    let n = active.len() as u32;
    let pos = active.iter().filter(|m| m.net >= 0).count() as u32;
    Ok(Reading {
        graded: None,
        value: pos as f64 / n as f64,
        text: of(pos, n, "nucleo-saude-valor-de-meses"),
        computation: ta(
            "nucleo-saude-conta-azul",
            &[
                ("periodo", f.period_label().into()),
                ("base", basis_label(f).into()),
            ],
        ),
        amounts: vec![],
    })
}

fn basis_label(f: &Facts) -> String {
    t(match f.basis {
        crate::budget::Basis::Accrual => "nucleo-saude-base-competencia",
        crate::budget::Basis::Cash => "nucleo-saude-base-caixa",
    })
}

// ---------------------------------------------------------------------------
// Indicadores (textos)

const SAVINGS: Spec = Spec {
    key: "taxa_poupanca",
    label: "nucleo-saude-poupanca",
    unit: HealthUnit::Percent,
    target: "nucleo-saude-poupanca-meta",
    target_value: 0.20,
    explanation: "nucleo-saude-poupanca-explicacao",
    advice: &[
        "nucleo-saude-poupanca-dica-1",
        "nucleo-saude-poupanca-dica-2",
        "nucleo-saude-poupanca-dica-3",
    ],
    no_data: "nucleo-saude-sem-dados-3-meses",
};

const REGULAR: Spec = Spec {
    key: "poupanca_regular",
    label: "nucleo-saude-regular",
    unit: HealthUnit::Ratio,
    target: "nucleo-saude-regular-meta",
    target_value: 1.0,
    explanation: "nucleo-saude-regular-explicacao",
    advice: &[
        "nucleo-saude-regular-dica-1",
        "nucleo-saude-regular-dica-2",
        "nucleo-saude-regular-dica-3",
    ],
    no_data: "nucleo-saude-sem-dados-3-meses",
};

const RESERVE: Spec = Spec {
    key: "reserva",
    label: "nucleo-saude-reserva",
    unit: HealthUnit::Months,
    target: "nucleo-saude-reserva-meta",
    target_value: 6.0,
    explanation: "nucleo-saude-reserva-explicacao",
    advice: &[
        "nucleo-saude-reserva-dica-1",
        "nucleo-saude-reserva-dica-2",
        "nucleo-saude-reserva-dica-3",
    ],
    no_data: "nucleo-saude-reserva-sem-dados",
};

const DEBT: Spec = Spec {
    key: "comprometimento",
    label: "nucleo-saude-comprometimento",
    unit: HealthUnit::Percent,
    target: "nucleo-saude-comprometimento-meta",
    target_value: 0.15,
    explanation: "nucleo-saude-comprometimento-explicacao",
    advice: &[
        "nucleo-saude-comprometimento-dica-1",
        "nucleo-saude-comprometimento-dica-2",
        "nucleo-saude-comprometimento-dica-3",
    ],
    no_data: "nucleo-saude-comprometimento-sem-dados",
};

const HOUSING: Spec = Spec {
    key: "moradia",
    label: "nucleo-saude-moradia",
    unit: HealthUnit::Percent,
    target: "nucleo-saude-moradia-meta",
    target_value: 0.30,
    explanation: "nucleo-saude-moradia-explicacao",
    advice: &[
        "nucleo-saude-moradia-dica-1",
        "nucleo-saude-moradia-dica-2",
        "nucleo-saude-moradia-dica-3",
    ],
    no_data: "nucleo-saude-moradia-sem-dados",
};

const CREDIT: Spec = Spec {
    key: "credito_caro",
    label: "nucleo-saude-credito",
    unit: HealthUnit::Count,
    target: "nucleo-saude-credito-meta",
    target_value: 0.0,
    explanation: "nucleo-saude-credito-explicacao",
    advice: &[
        "nucleo-saude-credito-dica-1",
        "nucleo-saude-credito-dica-2",
        "nucleo-saude-credito-dica-3",
    ],
    no_data: "nucleo-saude-credito-sem-dados",
};

const NET_WORTH: Spec = Spec {
    key: "patrimonio",
    label: "nucleo-saude-patrimonio",
    unit: HealthUnit::Percent,
    target: "nucleo-saude-patrimonio-meta",
    target_value: 0.05,
    explanation: "nucleo-saude-patrimonio-explicacao",
    advice: &[
        "nucleo-saude-patrimonio-dica-1",
        "nucleo-saude-patrimonio-dica-2",
        "nucleo-saude-patrimonio-dica-3",
    ],
    no_data: "nucleo-saude-patrimonio-sem-dados",
};

const ON_TIME: Spec = Spec {
    key: "contas_em_dia",
    label: "nucleo-saude-em-dia",
    unit: HealthUnit::Ratio,
    target: "nucleo-saude-em-dia-meta",
    target_value: 1.0,
    explanation: "nucleo-saude-em-dia-explicacao",
    advice: &[
        "nucleo-saude-em-dia-dica-1",
        "nucleo-saude-em-dia-dica-2",
        "nucleo-saude-em-dia-dica-3",
    ],
    no_data: "nucleo-saude-em-dia-sem-dados",
};

const BUDGET: Spec = Spec {
    key: "orcamento",
    label: "nucleo-saude-orcamento",
    unit: HealthUnit::Ratio,
    target: "nucleo-saude-meta-todos-os-meses",
    target_value: 1.0,
    explanation: "nucleo-saude-orcamento-explicacao",
    advice: &[
        "nucleo-saude-orcamento-dica-1",
        "nucleo-saude-orcamento-dica-2",
        "nucleo-saude-orcamento-dica-3",
    ],
    no_data: "nucleo-saude-orcamento-sem-dados",
};

const POSITIVE: Spec = Spec {
    key: "meses_no_azul",
    label: "nucleo-saude-azul",
    unit: HealthUnit::Ratio,
    target: "nucleo-saude-meta-todos-os-meses",
    target_value: 1.0,
    explanation: "nucleo-saude-azul-explicacao",
    advice: &[
        "nucleo-saude-azul-dica-1",
        "nucleo-saude-azul-dica-2",
        "nucleo-saude-azul-dica-3",
    ],
    no_data: "nucleo-saude-sem-dados-3-meses",
};

// ---------------------------------------------------------------------------
// I-SFB

/// (chave, rótulo no catálogo, de, até).
const ISF_BANDS: &[(&str, &str, u32, u32)] = &[
    ("ruim", "nucleo-saude-faixa-ruim", 0, 36),
    ("muito_baixa", "nucleo-saude-faixa-muito-baixa", 37, 49),
    ("baixa", "nucleo-saude-faixa-baixa", 50, 56),
    ("ok", "nucleo-saude-faixa-ok", 57, 60),
    ("boa", "nucleo-saude-faixa-boa", 61, 68),
    ("muito_boa", "nucleo-saude-faixa-muito-boa", 69, 82),
    ("otima", "nucleo-saude-faixa-otima", 83, 100),
];

/// Dimensão do I-SFB (rótulo e descrição são chaves do catálogo).
struct Dim {
    key: &'static str,
    label: &'static str,
    description: &'static str,
}

const DIMS: [Dim; 4] = [
    Dim {
        key: "comportamento",
        label: "nucleo-saude-dimensao-comportamento",
        description: "nucleo-saude-dimensao-comportamento-descricao",
    },
    Dim {
        key: "habilidade",
        label: "nucleo-saude-dimensao-habilidade",
        description: "nucleo-saude-dimensao-habilidade-descricao",
    },
    Dim {
        key: "liberdade",
        label: "nucleo-saude-dimensao-liberdade",
        description: "nucleo-saude-dimensao-liberdade-descricao",
    },
    Dim {
        key: "seguranca",
        label: "nucleo-saude-dimensao-seguranca",
        description: "nucleo-saude-dimensao-seguranca-descricao",
    },
];

/// Estado de uma nota de 0 a 100 (alinhado às faixas do I-SFB: "Muito boa" ou melhor é bom;
/// "Ok" e "Boa" pedem atenção).
fn score_status(score: Option<f64>) -> HealthStatus {
    match score {
        None => HealthStatus::Unknown,
        Some(s) if s.round() >= 69.0 => HealthStatus::Good,
        Some(s) if s.round() >= 57.0 => HealthStatus::Ok,
        Some(_) => HealthStatus::Bad,
    }
}

pub(crate) fn isf(f: &Facts) -> HealthReport {
    let c = Some("comportamento");
    let h = Some("habilidade");
    let l = Some("liberdade");
    let s = Some("seguranca");
    let metrics = vec![
        build(
            &ON_TIME,
            c,
            0.4,
            read_on_time(f),
            &Scale {
                points: &[(0.7, 0.0), (0.9, 50.0), (1.0, 100.0)],
                good: Cmp::AtLeast(1.0),
                ok: Cmp::AtLeast(0.9),
            },
        ),
        build(
            &BUDGET,
            c,
            0.3,
            read_budget(f),
            &Scale {
                points: &[(0.0, 10.0), (0.5, 55.0), (0.83, 85.0), (1.0, 100.0)],
                good: Cmp::AtLeast(0.83),
                ok: Cmp::AtLeast(0.5),
            },
        ),
        build(
            &POSITIVE,
            c,
            0.3,
            read_positive_months(f),
            &Scale {
                points: &[(0.25, 0.0), (0.5, 40.0), (0.75, 75.0), (1.0, 100.0)],
                good: Cmp::AtLeast(0.83),
                ok: Cmp::AtLeast(0.6),
            },
        ),
        build(
            &SAVINGS,
            h,
            0.6,
            read_savings_rate(f),
            &Scale {
                points: &[
                    (-0.1, 0.0),
                    (0.0, 30.0),
                    (0.1, 60.0),
                    (0.2, 85.0),
                    (0.3, 100.0),
                ],
                good: Cmp::AtLeast(0.2),
                ok: Cmp::AtLeast(0.1),
            },
        ),
        build(
            &REGULAR,
            h,
            0.4,
            read_regular_saving(f),
            &Scale {
                points: &[(0.0, 0.0), (0.5, 50.0), (0.83, 85.0), (1.0, 100.0)],
                good: Cmp::AtLeast(0.75),
                ok: Cmp::AtLeast(0.5),
            },
        ),
        build(
            &DEBT,
            l,
            0.6,
            read_debt_service(f),
            &Scale {
                points: &[
                    (0.0, 100.0),
                    (0.15, 85.0),
                    (0.3, 50.0),
                    (0.4, 25.0),
                    (0.6, 0.0),
                ],
                good: Cmp::AtMost(0.15),
                ok: Cmp::AtMost(0.30),
            },
        ),
        build(
            &CREDIT,
            l,
            0.4,
            read_expensive_credit(f),
            &Scale {
                points: &[
                    (0.0, 100.0),
                    (1.0 / 12.0, 75.0),
                    (0.25, 45.0),
                    (0.5, 15.0),
                    (1.0, 0.0),
                ],
                good: Cmp::AtMost(0.0),
                ok: Cmp::AtMost(1.0 / 6.0),
            },
        ),
        build(
            &RESERVE,
            s,
            0.6,
            read_reserve(f),
            &Scale {
                points: &[
                    (0.0, 0.0),
                    (1.0, 30.0),
                    (3.0, 60.0),
                    (6.0, 90.0),
                    (12.0, 100.0),
                ],
                good: Cmp::AtLeast(6.0),
                ok: Cmp::AtLeast(3.0),
            },
        ),
        build(
            &NET_WORTH,
            s,
            0.4,
            read_net_worth(f),
            &Scale {
                points: &[(-0.1, 0.0), (0.0, 40.0), (0.05, 70.0), (0.1, 100.0)],
                good: Cmp::AtLeast(0.05),
                ok: Cmp::AtLeast(0.0),
            },
        ),
    ];
    let dimensions: Vec<HealthDimension> = DIMS
        .iter()
        .map(|d| {
            let mine: Vec<&HealthMetric> = metrics
                .iter()
                .filter(|m| m.dimension.as_deref() == Some(d.key))
                .collect();
            let score = weighted(
                &mine
                    .iter()
                    .map(|m| (m.points, m.weight))
                    .collect::<Vec<_>>(),
            )
            .map(|(s, _)| s);
            HealthDimension {
                key: d.key.into(),
                label: t(d.label),
                description: t(d.description),
                weight: 0.25,
                score,
                status: score_status(score),
                metrics: mine.iter().map(|m| m.key.clone()).collect(),
            }
        })
        .collect();
    let known = dimensions.iter().filter(|d| d.score.is_some()).count();
    let score = if f.enough() && known >= 2 {
        weighted(
            &dimensions
                .iter()
                .map(|d| (d.score, d.weight))
                .collect::<Vec<_>>(),
        )
        .map(|(s, _)| s)
    } else {
        None
    };
    let all_bands = bands(ISF_BANDS);
    let band = score.and_then(|s| band_of(&all_bands, s));
    let summary = match (score, &band) {
        (Some(s), Some(b)) => {
            let cmp = if s > NATIONAL_AVERAGE + 0.05 {
                "nucleo-saude-resumo-acima"
            } else if s < NATIONAL_AVERAGE - 0.05 {
                "nucleo-saude-resumo-abaixo"
            } else {
                "nucleo-saude-resumo-igual"
            };
            let weakest = dimensions
                .iter()
                .filter_map(|d| d.score.map(|s| (s, d)))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, d)| d.label.clone());
            let mut text = ta(
                cmp,
                &[
                    ("nota", score_text(s).into()),
                    ("faixa", b.label.clone().into()),
                    ("media", score_text(NATIONAL_AVERAGE).into()),
                ],
            );
            if let Some(w) = weakest {
                text.push(' ');
                text.push_str(&ta(
                    "nucleo-saude-resumo-dimensao-fraca",
                    &[("dimensao", w.into())],
                ));
            }
            text
        }
        _ => insufficient(f),
    };
    let mut notes = vec![t("nucleo-saude-aviso-isf")];
    common_notes(f, &mut notes);
    HealthReport {
        model: HealthModel::Isf,
        enabled: true,
        as_of: f.as_of.clone(),
        period: f.period(),
        months_analyzed: f.n(),
        score,
        band,
        bands: all_bands,
        reference: Some(HealthReference {
            label: t("nucleo-saude-referencia"),
            short_label: t("nucleo-saude-referencia-curta"),
            score: NATIONAL_AVERAGE,
            source_url: SOURCE_URL.into(),
        }),
        dimensions,
        metrics,
        summary,
        notes,
        currency: f.currency.clone(),
        missing_fx: f.missing_fx.iter().cloned().collect(),
        estimated: f.estimated,
        stale_inputs: f.stale.clone(),
    }
}

fn insufficient(f: &Facts) -> String {
    if !f.enough() {
        ta(
            "nucleo-saude-insuficiente-meses",
            &[("minimo", MIN_MONTHS.into()), ("n", f.n().into())],
        )
    } else {
        t("nucleo-saude-insuficiente-indicadores")
    }
}

fn common_notes(f: &Facts, notes: &mut Vec<String>) {
    if !f.stale.is_empty() {
        let names: Vec<&str> = f.stale.iter().map(|s| s.name.as_str()).collect();
        notes.push(ta(
            "nucleo-saude-aviso-contas-desatualizadas",
            &[("contas", names.join(", ").into())],
        ));
    }
    if !f.enough() {
        notes.push(ta(
            "nucleo-saude-aviso-pouco-historico",
            &[("minimo", MIN_MONTHS.into())],
        ));
    }
    if !f.missing_fx.is_empty() {
        notes.push(ta(
            "nucleo-saude-aviso-sem-cambio",
            &[(
                "moedas",
                f.missing_fx
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
                    .into(),
            )],
        ));
    }
}

// ---------------------------------------------------------------------------
// Indicadores clássicos

const RATIO_BANDS: &[(&str, &str, u32, u32)] = &[
    ("critica", "nucleo-saude-faixa-critica", 0, 39),
    ("fragil", "nucleo-saude-faixa-fragil", 40, 59),
    ("estavel", "nucleo-saude-faixa-estavel", 60, 79),
    ("saudavel", "nucleo-saude-faixa-saudavel", 80, 100),
];

pub(crate) fn ratios(f: &Facts) -> HealthReport {
    let metrics = vec![
        build(
            &SAVINGS,
            None,
            0.25,
            read_savings_rate(f),
            &Scale {
                points: &[(0.0, 0.0), (0.1, 60.0), (0.2, 100.0)],
                good: Cmp::AtLeast(0.2),
                ok: Cmp::AtLeast(0.1),
            },
        ),
        build(
            &RESERVE,
            None,
            0.25,
            read_reserve(f),
            &Scale {
                points: &[(0.0, 0.0), (3.0, 60.0), (6.0, 100.0)],
                good: Cmp::AtLeast(6.0),
                ok: Cmp::AtLeast(3.0),
            },
        ),
        build(
            &DEBT,
            None,
            0.20,
            read_debt_service(f),
            &Scale {
                points: &[(0.15, 100.0), (0.3, 60.0), (0.4, 20.0), (0.5, 0.0)],
                good: Cmp::AtMost(0.15),
                ok: Cmp::AtMost(0.40),
            },
        ),
        build(
            &HOUSING,
            None,
            0.10,
            read_housing(f),
            &Scale {
                points: &[(0.3, 100.0), (0.4, 50.0), (0.5, 0.0)],
                good: Cmp::AtMost(0.30),
                ok: Cmp::AtMost(0.40),
            },
        ),
        build(
            &CREDIT,
            None,
            0.10,
            read_expensive_credit(f),
            &Scale {
                points: &[(0.0, 100.0), (1.0 / 12.0, 50.0), (0.25, 20.0), (0.5, 0.0)],
                good: Cmp::AtMost(0.0),
                ok: Cmp::AtMost(-1.0),
            },
        ),
        build(
            &NET_WORTH,
            None,
            0.10,
            read_net_worth(f),
            &Scale {
                points: &[(-0.1, 0.0), (0.0, 50.0), (0.05, 80.0), (0.1, 100.0)],
                good: Cmp::AtLeast(0.05),
                ok: Cmp::AtLeast(0.0),
            },
        ),
    ];
    let score = weighted(
        &metrics
            .iter()
            .map(|m| (m.points, m.weight))
            .collect::<Vec<_>>(),
    )
    .filter(|(_, coverage)| f.enough() && *coverage >= 0.5 - 1e-9)
    .map(|(s, _)| s);
    let all_bands = bands(RATIO_BANDS);
    let band = score.and_then(|s| band_of(&all_bands, s));
    let summary = match (score, &band) {
        (Some(s), Some(b)) => {
            let weakest = metrics
                .iter()
                .filter(|m| m.status != HealthStatus::Good)
                .filter_map(|m| m.points.map(|p| (p, m)))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, m)| m.label.clone());
            let tail = match weakest {
                Some(w) => ta(
                    "nucleo-saude-resumo-ponto-fraco",
                    &[("indicador", w.into())],
                ),
                None => t("nucleo-saude-resumo-tudo-na-meta"),
            };
            format!(
                "{} {tail}",
                ta(
                    "nucleo-saude-resumo-nota",
                    &[
                        ("nota", score_text(s).into()),
                        ("faixa", b.label.clone().into()),
                    ],
                )
            )
        }
        _ => insufficient(f),
    };
    let mut notes = vec![t("nucleo-saude-aviso-classicos")];
    common_notes(f, &mut notes);
    HealthReport {
        model: HealthModel::Ratios,
        enabled: true,
        as_of: f.as_of.clone(),
        period: f.period(),
        months_analyzed: f.n(),
        score,
        band,
        bands: all_bands,
        reference: None,
        dimensions: vec![],
        metrics,
        summary,
        notes,
        currency: f.currency.clone(),
        missing_fx: f.missing_fx.iter().cloned().collect(),
        estimated: f.estimated,
        stale_inputs: f.stale.clone(),
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn curve_interpolates_and_clamps() {
        let pts = &[(0.0, 0.0), (0.1, 60.0), (0.2, 100.0)];
        assert_eq!(curve(-1.0, pts), 0.0);
        assert!((curve(0.05, pts) - 30.0).abs() < 1e-9);
        assert!((curve(0.15, pts) - 80.0).abs() < 1e-9);
        assert_eq!(curve(0.5, pts), 100.0);
    }

    #[test]
    fn bands_cover_the_scale() {
        let b = bands(ISF_BANDS);
        assert_eq!(band_of(&b, 56.7).unwrap().label, "Ok");
        assert_eq!(band_of(&b, 36.4).unwrap().label, "Ruim");
        assert_eq!(band_of(&b, 36.5).unwrap().label, "Muito baixa");
        assert_eq!(band_of(&b, 100.0).unwrap().label, "Ótima");
        for s in 0..=100 {
            assert!(band_of(&b, s as f64).is_some(), "{s}");
            assert!(band_of(&bands(RATIO_BANDS), s as f64).is_some(), "{s}");
        }
    }

    #[test]
    fn weighted_ignores_unknown() {
        assert_eq!(
            weighted(&[(Some(80.0), 0.6), (None, 0.4)]),
            Some((80.0, 0.6))
        );
        assert_eq!(weighted(&[(None, 1.0)]), None);
    }
}
