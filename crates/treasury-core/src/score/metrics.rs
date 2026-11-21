//! Métricas brutas por classe (funções puras sobre fatos, cotação e séries já carregados).
//! Cada métrica sai com valor, período, fonte e, nas métricas-chave, a média de 5 anos e a
//! inclinação anual; ou com o motivo de faltar. A nota (0–100) vem depois, no motor.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::frameworks::{self, Framework, Year};
use super::normalize::{mean, slope, stdev};
use crate::research::{Fact, PeriodType, items, periods, to_f64};
use crate::time::add_months;

/// Valor bruto de uma métrica.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: Option<f64>,
    /// Média dos últimos exercícios (métricas-chave).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg: Option<f64>,
    /// Inclinação anual da série (métricas-chave).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trend: Option<f64>,
    /// Denominador negativo ou zero: recebe a pior nota explicitamente (§7.3 item 6).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub worst: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_end: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Por que não há valor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing: Option<String>,
}

impl MetricValue {
    pub fn ok(v: f64) -> Self {
        if v.is_finite() {
            Self {
                value: Some(v),
                ..Default::default()
            }
        } else {
            Self::missing(crate::i18n::t("nucleo-score-metrica-divisao-zero"))
        }
    }
    pub fn missing(why: impl Into<String>) -> Self {
        Self {
            missing: Some(why.into()),
            ..Default::default()
        }
    }
    pub fn worst(why: impl Into<String>) -> Self {
        Self {
            worst: true,
            note: Some(why.into()),
            ..Default::default()
        }
    }
    pub fn is_available(&self) -> bool {
        self.value.is_some() || self.worst
    }
    fn at(
        mut self,
        period: impl Into<String>,
        end: Option<NaiveDate>,
        source: Option<String>,
    ) -> Self {
        self.period = Some(period.into());
        self.period_end = end;
        self.source = source;
        self
    }
    fn noted(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

pub type Metrics = BTreeMap<String, MetricValue>;

/// Chaves dos itens das demonstrações (as novas vêm da coleta ampliada da CVM).
pub mod keys {
    pub use crate::research::items::*;
    pub const COGS: &str = "cogs";
    pub const SGA: &str = "sga";
    pub const FIN_INCOME: &str = "fin_income";
    pub const FIN_EXPENSE: &str = "fin_expense";
    pub const RECEIVABLES: &str = "receivables";
    pub const INVENTORIES: &str = "inventories";
    pub const PPE: &str = "ppe";
    pub const SUPPLIERS: &str = "suppliers";
    pub const DEBT_ST: &str = "debt_st";
    pub const DEBT_LT: &str = "debt_lt";
    pub const LEASES: &str = "leases";
    pub const RETAINED: &str = "retained";
    pub const MINORITY: &str = "minority";
}

/// Data no formato do idioma da instalação (14/10/2026, Oct 14, 2026).
pub(crate) fn br(d: NaiveDate) -> String {
    crate::i18n::date(crate::i18n::current(), d)
}

fn f(d: Decimal) -> f64 {
    to_f64(d)
}

fn div(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    let (a, b) = (a?, b?);
    (b != 0.0).then(|| a / b).filter(|x| x.is_finite())
}

fn avg2(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some((x + y) / 2.0),
        (x, None) => x,
        (None, y) => y,
    }
}

/// Fatos de um emissor organizados por exercício e nos últimos 12 meses.
pub struct Statements<'a> {
    facts: &'a [Fact],
    /// Fim do LTM (data do último documento com fluxos).
    pub reference: Option<NaiveDate>,
    /// Exercícios (mais antigo primeiro), com fluxos anuais e saldos do fim do exercício.
    pub years: Vec<Year>,
    /// Fluxos dos últimos 12 meses e saldos do último balanço.
    pub ltm: Year,
    /// Saldos de 12 meses antes do último balanço.
    pub year_ago: Year,
    /// Documentos usados no LTM e no último balanço.
    pub ltm_source: Option<String>,
    pub balance_date: Option<NaiveDate>,
}

const FLOW_ITEMS: &[&str] = &[
    keys::REVENUE,
    keys::COGS,
    keys::GROSS_PROFIT,
    keys::SGA,
    keys::EBIT,
    keys::DA,
    keys::NET_INCOME,
    keys::CFO,
    keys::CAPEX,
    keys::DIVIDENDS_PAID,
    keys::FIN_INCOME,
    keys::FIN_EXPENSE,
    keys::EBT,
];

fn set_flow(y: &mut Year, item: &str, v: f64) {
    match item {
        keys::REVENUE => y.revenue = Some(v),
        keys::COGS => y.cogs = Some(v.abs()),
        keys::GROSS_PROFIT => y.gross_profit = Some(v),
        keys::SGA => y.sga = Some(v.abs()),
        keys::EBIT => y.ebit = Some(v),
        keys::DA => y.da = Some(v.abs()),
        keys::NET_INCOME => y.net_income = Some(v),
        keys::CFO => y.cfo = Some(v),
        // A CVM traz o capex como saída (negativo); aqui fica o módulo.
        keys::CAPEX => y.capex = Some(v.abs()),
        keys::DIVIDENDS_PAID => y.dividends_paid = Some(v.abs()),
        keys::FIN_INCOME => y.fin_income = Some(v.abs()),
        keys::FIN_EXPENSE => y.fin_expense = Some(v.abs()),
        keys::EBT => y.ebt = Some(v),
        _ => {}
    }
}

const BALANCE_ITEMS: &[&str] = &[
    keys::TOTAL_ASSETS,
    keys::CURRENT_ASSETS,
    keys::CURRENT_LIABILITIES,
    keys::TOTAL_LIABILITIES,
    keys::CASH,
    keys::RECEIVABLES,
    keys::INVENTORIES,
    keys::PPE,
    keys::SUPPLIERS,
    keys::DEBT_ST,
    keys::DEBT_LT,
    keys::LEASES,
    keys::GROSS_DEBT,
    keys::EQUITY,
    keys::MINORITY,
    keys::RETAINED,
    keys::SHARES,
];

fn set_balance(y: &mut Year, item: &str, v: f64) {
    let slot = match item {
        keys::TOTAL_ASSETS => &mut y.total_assets,
        keys::CURRENT_ASSETS => &mut y.current_assets,
        keys::CURRENT_LIABILITIES => &mut y.current_liabilities,
        keys::TOTAL_LIABILITIES => &mut y.total_liabilities,
        keys::CASH => &mut y.cash,
        keys::RECEIVABLES => &mut y.receivables,
        keys::INVENTORIES => &mut y.inventories,
        keys::PPE => &mut y.ppe,
        keys::SUPPLIERS => &mut y.suppliers,
        keys::DEBT_ST => &mut y.debt_st,
        keys::DEBT_LT => &mut y.debt_lt,
        keys::LEASES => &mut y.leases,
        keys::GROSS_DEBT => &mut y.gross_debt,
        keys::EQUITY => &mut y.equity,
        keys::MINORITY => &mut y.minority,
        keys::RETAINED => &mut y.retained,
        keys::SHARES => &mut y.shares,
        _ => return,
    };
    *slot = Some(v);
}

impl<'a> Statements<'a> {
    pub fn new(facts: &'a [Fact], today: NaiveDate) -> Self {
        let reference = periods::reference_date(facts);
        // Exercícios: pelos fins de período anuais do lucro ou da receita.
        let mut ends: Vec<NaiveDate> = facts
            .iter()
            .filter(|x| {
                x.period_type == PeriodType::Annual
                    && (x.item == keys::NET_INCOME || x.item == keys::REVENUE)
            })
            .map(|x| x.end)
            .collect();
        ends.sort();
        ends.dedup();
        let mut years = vec![];
        for end in ends {
            let mut y = Year {
                end: Some(end),
                ..Default::default()
            };
            for item in FLOW_ITEMS {
                if let Some(x) = facts.iter().find(|x| {
                    x.item == *item
                        && x.period_type == PeriodType::Annual
                        && periods::near(x.end, end)
                }) {
                    set_flow(&mut y, item, f(x.value));
                }
            }
            Self::fill_balance(facts, &mut y, end, true);
            estimate_ebit(&mut y);
            years.push(y);
        }
        let mut ltm = Year::default();
        let mut srcs: Vec<String> = vec![];
        if let Some(at) = reference {
            ltm.end = Some(at);
            for item in FLOW_ITEMS {
                let x = periods::ltm_at(facts, item, at).or_else(|| {
                    let a = periods::latest(facts, item, PeriodType::Annual)
                        .filter(|a| a.end >= add_months(at, -15))?;
                    Some(a.clone())
                });
                if let Some(x) = x {
                    set_flow(&mut ltm, item, f(x.value));
                    if *item == keys::NET_INCOME {
                        srcs.push(x.document.clone());
                    }
                }
            }
        }
        let balance_date = periods::instant_at(facts, keys::TOTAL_ASSETS, today)
            .or_else(|| periods::instant_at(facts, keys::EQUITY, today))
            .map(|x| x.end);
        estimate_ebit(&mut ltm);
        let mut year_ago = Year::default();
        if let Some(b) = balance_date {
            Self::fill_balance(facts, &mut ltm, b, true);
            if let Some(doc) =
                periods::instant_at(facts, keys::EQUITY, today).map(|x| x.document.clone())
            {
                srcs.push(doc);
            }
            let back = add_months(b, -12);
            year_ago.end = Some(back);
            Self::fill_balance(facts, &mut year_ago, back, true);
        }
        srcs.dedup();
        Self {
            facts,
            reference,
            years,
            ltm,
            year_ago,
            ltm_source: (!srcs.is_empty()).then(|| srcs.join("; ")),
            balance_date,
        }
    }

    /// Saldos numa data (com tolerância de 10 dias; ações em circulação aceitam o último
    /// número até a data, pois a composição do capital nem sempre coincide).
    fn fill_balance(facts: &[Fact], y: &mut Year, at: NaiveDate, exact: bool) {
        for item in BALANCE_ITEMS {
            let x = facts
                .iter()
                .filter(|x| {
                    x.item == *item
                        && x.period_type == PeriodType::Instant
                        && (if exact && *item != keys::SHARES {
                            periods::near(x.end, at)
                        } else {
                            x.end <= at + chrono::Duration::days(10)
                        })
                })
                .max_by_key(|x| (x.end, x.priority));
            if let Some(x) = x {
                set_balance(y, item, f(x.value));
            }
        }
    }

    pub fn has_facts(&self) -> bool {
        !self.facts.is_empty()
    }

    /// Últimos `n` exercícios (mais antigo primeiro).
    pub fn last_years(&self, n: usize) -> &[Year] {
        let k = self.years.len().saturating_sub(n);
        &self.years[k..]
    }

    fn period_ltm(&self) -> String {
        self.reference
            .map(crate::research::indicators::ltm_until)
            .unwrap_or_else(|| crate::i18n::t("nucleo-score-periodo-12-meses"))
    }
}

/// Sem EBIT informado (REITs e algumas empresas americanas): resultado antes dos tributos
/// + despesas financeiras − receitas financeiras.
fn estimate_ebit(y: &mut Year) {
    if y.ebit.is_none()
        && let (Some(ebt), Some(fe)) = (y.ebt, y.fin_expense)
    {
        y.ebit = Some(ebt + fe - y.fin_income.unwrap_or(0.0));
    }
}

/// Uma série anual de uma razão (fim do exercício, valor).
fn annual_series(
    years: &[Year],
    f: impl Fn(&Year, Option<&Year>) -> Option<f64>,
) -> Vec<(NaiveDate, f64)> {
    let mut out = vec![];
    for (i, y) in years.iter().enumerate() {
        let prev = i.checked_sub(1).map(|j| &years[j]);
        if let (Some(end), Some(v)) = (y.end, f(y, prev)) {
            out.push((end, v));
        }
    }
    out
}

/// Média e inclinação anual dos últimos `n` pontos de uma série.
fn history(series: &[(NaiveDate, f64)], n: usize) -> (Option<f64>, Option<f64>) {
    let k = series.len().saturating_sub(n);
    let s = &series[k..];
    let vals: Vec<f64> = s.iter().map(|p| p.1).collect();
    let pts: Vec<(f64, f64)> = s
        .iter()
        .map(|(d, v)| (d.year() as f64 + d.ordinal0() as f64 / 365.0, *v))
        .collect();
    (
        (vals.len() >= 3).then(|| mean(&vals)).flatten(),
        slope(&pts),
    )
}

/// Entradas das métricas de ações não financeiras.
pub struct EquityInput<'a> {
    pub st: &'a Statements<'a>,
    pub today: NaiveDate,
    /// Valor de mercado na moeda das demonstrações e a fonte da cotação.
    pub market_cap: Option<(f64, String)>,
    pub dividend_yield: Option<(f64, String)>,
    /// Pontuação do segmento de listagem (0–100) e o nome do segmento.
    pub listing: Option<(f64, String)>,
    /// Entregas de DFP/ITR: (tipo `dfp`/`itr`, data de referência, data de entrega).
    pub filings: &'a [(String, NaiveDate, NaiveDate)],
    /// Fechamentos diários (para volatilidade e momentum; vazio nos pares).
    pub prices: &'a [(NaiveDate, f64)],
    /// Fator de inflação (IPCA) entre duas datas.
    pub inflation: &'a dyn Fn(NaiveDate, NaiveDate) -> Option<f64>,
    pub tax_rate: f64,
    pub history_years: usize,
}

/// Resultado das métricas de ações: as métricas e os frameworks (diagnóstico e flags).
pub struct EquityOutput {
    pub metrics: Metrics,
    pub frameworks: Vec<Framework>,
    /// DSRI do Beneish (flag própria).
    pub dsri: Option<f64>,
    /// Cobertura de juros nos dois últimos trimestres (LTM em cada um).
    pub coverage_last_quarters: Vec<f64>,
    /// FCF anual dos últimos exercícios (mais antigo primeiro).
    pub fcf_years: Vec<f64>,
    /// Dividendos pagos − FCF por exercício (positivo = pagou mais que o caixa livre).
    pub dividends_over_fcf_years: Vec<bool>,
    pub negative_equity: bool,
}

fn roic_of(y: &Year, tax: f64) -> Option<f64> {
    let capital = y.net_debt()? + y.equity? + y.minority.unwrap_or(0.0);
    if capital <= 0.0 {
        return None;
    }
    Some(y.ebit? * (1.0 - tax) / capital)
}

fn roe_of(y: &Year, prev: Option<&Year>) -> Option<f64> {
    let eq = avg2(y.equity, prev.and_then(|p| p.equity))?;
    if eq <= 0.0 {
        return None;
    }
    Some(y.net_income? / eq)
}

fn cagr(first: f64, last: f64, years: f64) -> Option<f64> {
    (first > 0.0 && last > 0.0 && years > 0.0).then(|| (last / first).powf(1.0 / years) - 1.0)
}

pub fn equity_metrics(i: &EquityInput<'_>) -> EquityOutput {
    let st = i.st;
    let t = &st.ltm;
    let ya = &st.year_ago;
    let ltm_p = st.period_ltm();
    let bal_p = st
        .balance_date
        .map(crate::research::indicators::balance_at)
        .unwrap_or_else(|| crate::i18n::t("nucleo-score-metrica-periodo-ultimo-balanco"));
    let src = st.ltm_source.clone();
    let mut m = Metrics::new();
    let hy = i.history_years;
    let yrs = st.last_years(10);
    let put = |m: &mut Metrics, k: &str, v: MetricValue, p: &str| {
        m.insert(k.into(), v.at(p.to_string(), st.reference, src.clone()));
    };

    // Rentabilidade.
    let roic_series = annual_series(yrs, |y, _| roic_of(y, i.tax_rate));
    let (ra, rt) = history(&roic_series, hy);
    let roic = match roic_of(t, i.tax_rate) {
        Some(v) => MetricValue {
            avg: ra,
            trend: rt,
            ..MetricValue::ok(v)
        },
        None if t.ebit.is_none() => {
            MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-ebit"))
        }
        None if t.equity.is_none() => {
            MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-patrimonio"))
        }
        None => MetricValue::worst(crate::i18n::t("nucleo-score-metrica-capital-negativo")),
    };
    put(&mut m, "roic", roic, &ltm_p);
    let roic_norm = match ra {
        Some(v) => MetricValue {
            avg: ra,
            trend: rt,
            ..MetricValue::ok(v)
        }
        .noted(crate::i18n::t_args(
            "nucleo-score-metrica-media-exercicios",
            &crate::i18n::args(&[("n", (roic_series.len().min(hy) as i64).into())]),
        )),
        None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-poucos-roic")),
    };
    put(
        &mut m,
        "roic_norm",
        roic_norm,
        &crate::i18n::t("nucleo-score-metrica-periodo-media-exercicios"),
    );

    let roe_series = annual_series(yrs, roe_of);
    let (ea, et) = history(&roe_series, hy);
    let roe = match (t.net_income, avg2(t.equity, ya.equity)) {
        _ if t.equity.is_some_and(|e| e <= 0.0) => {
            MetricValue::worst(crate::i18n::t("nucleo-score-metrica-patrimonio-negativo"))
        }
        (Some(ni), Some(eq)) if eq > 0.0 => MetricValue {
            avg: ea,
            trend: et,
            ..MetricValue::ok(ni / eq)
        },
        (Some(_), Some(_)) => {
            MetricValue::worst(crate::i18n::t("nucleo-score-metrica-patrimonio-negativo"))
        }
        (None, _) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-lucro-12m")),
        (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-patrimonio")),
    };
    put(&mut m, "roe", roe, &ltm_p);
    put(
        &mut m,
        "roa",
        match div(t.net_income, avg2(t.total_assets, ya.total_assets)) {
            Some(v) => MetricValue::ok(v),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-lucro-ativo")),
        },
        &ltm_p,
    );
    put(
        &mut m,
        "gross_profitability",
        match div(t.gross(), t.total_assets) {
            Some(v) => MetricValue::ok(v),
            None => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-lucro-bruto-ativo"))
            }
        },
        &ltm_p,
    );
    // `what`: chave do motivo quando falta o numerador.
    let margin = |num: fn(&Year) -> Option<f64>, what: &str| match (num(t), t.revenue) {
        (Some(n), Some(r)) if r > 0.0 => MetricValue::ok(n / r),
        (None, _) => MetricValue::missing(crate::i18n::t(what)),
        _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-receita-positiva")),
    };
    put(
        &mut m,
        "ebit_margin",
        margin(|y| y.ebit, "nucleo-score-metrica-sem-ebit"),
        &ltm_p,
    );
    let nm_series = annual_series(yrs, |y, _| {
        div(y.net_income, y.revenue.filter(|r| *r > 0.0))
    });
    let (na, nt) = history(&nm_series, hy);
    let mut nm = margin(|y| y.net_income, "nucleo-score-metrica-sem-lucro-liquido");
    nm.avg = na;
    nm.trend = nt;
    put(&mut m, "net_margin", nm, &ltm_p);

    // Endividamento e liquidez.
    let nd_series = annual_series(yrs, |y, _| {
        let e = y.ebitda()?;
        (e > 0.0).then(|| y.net_debt().map(|n| n / e)).flatten()
    });
    let (da, dt) = history(&nd_series, hy);
    let nd_e = match (t.net_debt(), t.ebitda()) {
        (Some(nd), Some(e)) if e > 0.0 => MetricValue {
            avg: da,
            trend: dt,
            ..MetricValue::ok(nd / e)
        },
        (Some(_), Some(_)) => MetricValue::worst("EBITDA ≤ 0"),
        (None, _) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-divida-caixa")),
        (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-ebit-12m")),
    };
    put(&mut m, "net_debt_ebitda", nd_e, &ltm_p);
    let ebitda_avg = {
        let v: Vec<f64> = st.last_years(hy).iter().filter_map(Year::ebitda).collect();
        (v.len() >= 3).then(|| mean(&v)).flatten()
    };
    put(
        &mut m,
        "net_debt_ebitda_norm",
        match (t.net_debt(), ebitda_avg) {
            (Some(nd), Some(e)) if e > 0.0 => MetricValue {
                avg: da,
                trend: dt,
                ..MetricValue::ok(nd / e)
            }
            .noted(crate::i18n::t("nucleo-score-metrica-nota-ebitda-medio")),
            (Some(_), Some(_)) => {
                MetricValue::worst(crate::i18n::t("nucleo-score-metrica-ebitda-medio-negativo"))
            }
            (None, _) => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-divida-caixa"))
            }
            (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-poucos-ebitda")),
        },
        &crate::i18n::t("nucleo-score-metrica-nota-divida-ebitda"),
    );
    let coverage = |y: &Year| -> Option<MetricValue> {
        let ebit = y.ebit?;
        let net = y.fin_expense? - y.fin_income.unwrap_or(0.0);
        Some(if net <= 0.0 {
            MetricValue::ok(99.0).noted(crate::i18n::t(
                "nucleo-score-metrica-nota-receitas-financeiras",
            ))
        } else if ebit <= 0.0 {
            MetricValue::ok(0.0).noted("EBIT ≤ 0")
        } else {
            MetricValue::ok(ebit / net)
        })
    };
    put(
        &mut m,
        "interest_coverage",
        coverage(t).unwrap_or_else(|| {
            MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-ebit-despesas"))
        }),
        &ltm_p,
    );
    let z = frameworks::altman_z2(&Year {
        ebit: t.ebit,
        ..t.clone()
    });
    put(
        &mut m,
        "altman_z2",
        match z.value {
            Some(v) => MetricValue::ok(v),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-altman")),
        },
        &ltm_p,
    );
    put(
        &mut m,
        "current_ratio",
        match div(t.current_assets, t.current_liabilities) {
            Some(v) => MetricValue::ok(v),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-circulante")),
        },
        &bal_p,
    );
    put(
        &mut m,
        "cash_to_st_debt",
        match (t.cash, t.debt_st) {
            (Some(_), Some(d)) if d <= 0.0 => {
                MetricValue::ok(99.0).noted(crate::i18n::t("nucleo-score-metrica-sem-divida-cp"))
            }
            (Some(c), Some(d)) => MetricValue::ok(c / d),
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-caixa-divida-cp")),
        },
        &bal_p,
    );
    put(
        &mut m,
        "equity_to_assets",
        match div(t.equity, t.total_assets) {
            Some(v) => MetricValue::ok(v),
            None => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-patrimonio-ativo"))
            }
        },
        &bal_p,
    );

    // Qualidade dos lucros.
    let fcf = t.fcf();
    put(
        &mut m,
        "fcf_to_net_income",
        match (fcf, t.net_income) {
            (Some(c), Some(n)) if n > 0.0 => MetricValue::ok(c / n),
            (Some(_), Some(_)) => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-lucro-negativo-razao"))
            }
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-fco-lucro")),
        },
        &ltm_p,
    );
    let fm_series = annual_series(yrs, |y, _| div(y.fcf(), y.revenue.filter(|r| *r > 0.0)));
    let (fa, ft) = history(&fm_series, hy);
    put(
        &mut m,
        "fcf_margin",
        match (fcf, t.revenue) {
            (Some(c), Some(r)) if r > 0.0 => MetricValue {
                avg: fa,
                trend: ft,
                ..MetricValue::ok(c / r)
            },
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-fco-receita")),
        },
        &ltm_p,
    );
    put(
        &mut m,
        "accruals",
        match (t.net_income, t.cfo, avg2(t.total_assets, ya.total_assets)) {
            (Some(n), Some(c), Some(a)) if a > 0.0 => MetricValue::ok((n - c) / a),
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-lucro-fco-ativo")),
        },
        &ltm_p,
    );
    let mut fws = vec![];
    let n = st.years.len();
    let (mut piot, mut ben) = (None, None);
    if n >= 2 {
        let (tt, pp) = (&st.years[n - 1], &st.years[n - 2]);
        let ppp = n.checked_sub(3).map(|k| &st.years[k]);
        let p = frameworks::piotroski(tt, pp, ppp);
        let b = frameworks::beneish(tt, pp);
        piot = Some(p.clone());
        ben = Some(b.clone());
        fws.push(p);
        fws.push(frameworks::altman_z2(t));
        fws.push(b);
    } else {
        fws.push(frameworks::altman_z2(t));
    }
    let last_fy = st.years.last().and_then(|y| y.end);
    put(
        &mut m,
        "piotroski",
        match piot.as_ref().and_then(|p| p.value) {
            Some(v) => MetricValue::ok(v),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-piotroski")),
        },
        &last_fy
            .map(|d| {
                crate::i18n::t_args(
                    "nucleo-score-metrica-periodo-exercicio",
                    &crate::i18n::args(&[("data", br(d).into())]),
                )
            })
            .unwrap_or_default(),
    );
    let dsri = ben
        .as_ref()
        .and_then(|b| b.items.first())
        .and_then(|i| i.value);

    // Crescimento.
    let rev: Vec<(NaiveDate, f64)> = yrs
        .iter()
        .filter_map(|y| Some((y.end?, y.revenue?)))
        .collect();
    let real_cagr = |n: usize| -> MetricValue {
        if rev.len() < n + 1 {
            return MetricValue::missing(crate::i18n::t_args(
                "nucleo-score-metrica-poucos-exercicios-receita",
                &crate::i18n::args(&[("n", ((n + 1) as i64).into())]),
            ));
        }
        let (d0, r0) = rev[rev.len() - 1 - n];
        let (d1, r1) = rev[rev.len() - 1];
        let Some(nominal) = cagr(r0, r1, n as f64) else {
            return MetricValue::missing(crate::i18n::t("nucleo-score-metrica-receita-negativa"));
        };
        match (i.inflation)(d0, d1) {
            Some(k) if k > 0.0 => MetricValue::ok((1.0 + nominal) / k.powf(1.0 / n as f64) - 1.0)
                .noted(crate::i18n::t_args(
                    "nucleo-score-metrica-nota-nominal",
                    &crate::i18n::args(&[(
                        "pct",
                        crate::ai::alerts::decimal(
                            crate::i18n::current(),
                            &format!("{:.1}", nominal * 100.0),
                        )
                        .into(),
                    )]),
                )),
            _ => MetricValue::ok(nominal).noted(crate::i18n::t("nucleo-score-metrica-sem-ipca")),
        }
        .at(
            crate::i18n::t_args(
                "nucleo-score-periodo-de-a",
                &crate::i18n::args(&[
                    ("de", d0.year().to_string().into()),
                    ("ate", d1.year().to_string().into()),
                ]),
            ),
            Some(d1),
            None,
        )
    };
    m.insert("revenue_cagr_5y_real".into(), real_cagr(5));
    m.insert("revenue_cagr_3y_real".into(), real_cagr(3));
    let eps: Vec<(NaiveDate, f64)> = yrs
        .iter()
        .filter_map(|y| Some((y.end?, div(y.net_income, y.shares.filter(|s| *s > 0.0))?)))
        .collect();
    m.insert(
        "eps_cagr_5y".into(),
        if eps.len() >= 6 {
            let (d0, e0) = eps[eps.len() - 6];
            let (d1, e1) = eps[eps.len() - 1];
            match cagr(e0, e1, 5.0) {
                Some(v) => MetricValue::ok(v),
                None if e1 <= 0.0 => {
                    MetricValue::worst(crate::i18n::t("nucleo-score-metrica-lpa-final-negativo"))
                }
                None => MetricValue::missing(crate::i18n::t(
                    "nucleo-score-metrica-lpa-inicial-negativo",
                )),
            }
            .at(
                crate::i18n::t_args(
                    "nucleo-score-periodo-de-a",
                    &crate::i18n::args(&[
                        ("de", d0.year().to_string().into()),
                        ("ate", d1.year().to_string().into()),
                    ]),
                ),
                Some(d1),
                None,
            )
        } else {
            MetricValue::missing(crate::i18n::t("nucleo-score-metrica-poucos-lpa"))
        },
    );
    let ni_years: Vec<(NaiveDate, f64)> = yrs
        .iter()
        .filter_map(|y| Some((y.end?, y.net_income?)))
        .collect();
    m.insert(
        "net_income_growth".into(),
        match ni_years.len() {
            n if n >= 2 => {
                let ((_, a), (e, b)) = (ni_years[n - 2], ni_years[n - 1]);
                if a > 0.0 {
                    MetricValue::ok(b / a - 1.0).at(
                        crate::i18n::t_args(
                            "nucleo-score-metrica-periodo-exercicio",
                            &crate::i18n::args(&[("data", br(e).into())]),
                        ),
                        Some(e),
                        Some(crate::i18n::t("nucleo-score-metrica-fonte-dfp")),
                    )
                } else {
                    MetricValue::missing(crate::i18n::t(
                        "nucleo-score-metrica-lucro-anterior-negativo",
                    ))
                }
            }
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-poucos-lucros-2")),
        },
    );
    put(
        &mut m,
        "asset_growth",
        match div(t.total_assets, ya.total_assets.filter(|a| *a > 0.0)) {
            Some(v) => MetricValue::ok(v - 1.0),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-ativo-anterior")),
        },
        &bal_p,
    );
    let rev_growth = {
        let n = rev.len();
        (n >= 2)
            .then(|| div(Some(rev[n - 1].1), Some(rev[n - 2].1)).map(|x| x - 1.0))
            .flatten()
    };
    put(
        &mut m,
        "rule_of_40",
        match (rev_growth, div(fcf, t.revenue.filter(|r| *r > 0.0))) {
            (Some(g), Some(fm)) => MetricValue::ok(g + fm),
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-regra-40")),
        },
        &ltm_p,
    );

    // Eficiência.
    put(
        &mut m,
        "asset_turnover",
        match div(t.revenue, avg2(t.total_assets, ya.total_assets)) {
            Some(v) => MetricValue::ok(v),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-receita-ativo")),
        },
        &ltm_p,
    );
    put(
        &mut m,
        "cash_cycle_days",
        match (t.receivables, t.inventories, t.suppliers, t.revenue, t.cogs) {
            (Some(r), inv, Some(s), Some(rv), Some(c)) if rv > 0.0 && c > 0.0 => {
                MetricValue::ok(r / rv * 365.0 + inv.unwrap_or(0.0) / c * 365.0 - s / c * 365.0)
            }
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-ciclo")),
        },
        &ltm_p,
    );

    // Valuation (precisa da cotação).
    let mc = i.market_cap.as_ref().map(|x| x.0);
    let price_src = i.market_cap.as_ref().map(|x| x.1.clone());
    let with_price = |v: MetricValue| MetricValue {
        source: match (&src, &price_src) {
            (Some(a), Some(b)) => Some(format!("{a}; {b}")),
            (a, b) => a.clone().or(b.clone()),
        },
        period: Some(ltm_p.clone()),
        period_end: st.reference,
        ..v
    };
    let no_price =
        || MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-cotacao-acoes"));
    let ev = mc.and_then(|c| Some(c + t.net_debt()? + t.minority.unwrap_or(0.0)));
    m.insert(
        "earnings_yield".into(),
        match (ev, t.ebit) {
            (None, _) if mc.is_none() => no_price(),
            (None, _) => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-divida-liquida"))
            }
            (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-ebit")),
            (Some(e), Some(_)) if e <= 0.0 => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-ev-negativo"))
            }
            (Some(e), Some(b)) => with_price(MetricValue::ok(b / e)),
        },
    );
    m.insert(
        "fcf_yield".into(),
        match (mc, fcf) {
            (None, _) => no_price(),
            (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-fcl")),
            (Some(c), Some(x)) => with_price(MetricValue::ok(x / c)),
        },
    );
    m.insert(
        "earnings_to_price".into(),
        match (mc, t.net_income) {
            (None, _) => no_price(),
            (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-lucro")),
            (Some(c), Some(x)) => with_price(MetricValue::ok(x / c)),
        },
    );
    m.insert(
        "pb".into(),
        match (mc, t.equity) {
            (None, _) => no_price(),
            (_, None) => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-patrimonio"))
            }
            (Some(_), Some(e)) if e <= 0.0 => with_price(MetricValue::worst(crate::i18n::t(
                "nucleo-score-metrica-patrimonio-negativo",
            ))),
            (Some(c), Some(e)) => with_price(MetricValue::ok(c / e)),
        },
    );
    m.insert(
        "ev_ebitda".into(),
        match (ev, t.ebitda()) {
            (None, _) => no_price(),
            (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-ebitda")),
            (Some(_), Some(e)) if e <= 0.0 => with_price(MetricValue::worst("EBITDA ≤ 0")),
            (Some(v), Some(e)) => with_price(MetricValue::ok(v / e)),
        },
    );
    m.insert(
        "dy".into(),
        match &i.dividend_yield {
            Some((v, s)) => MetricValue {
                source: Some(s.clone()),
                period: Some("12 meses".into()),
                ..MetricValue::ok(*v)
            },
            None => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-proventos-cotacao"))
            }
        },
    );

    // Governança.
    m.insert(
        "listing_segment".into(),
        match &i.listing {
            Some((s, name)) => MetricValue {
                source: Some(crate::i18n::t("nucleo-score-metrica-fonte-fca")),
                ..MetricValue::ok(*s)
            }
            .noted(name.clone()),
            None => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-segmento-listagem"))
            }
        },
    );
    m.insert("timely_filing".into(), timely_filing(i.filings, i.today));

    // Estabilidade.
    let roe_vals: Vec<f64> = roe_series.iter().rev().take(10).map(|p| p.1).collect();
    m.insert(
        "roe_stability".into(),
        match (roe_vals.len() >= 4).then(|| stdev(&roe_vals)).flatten() {
            Some(s) => MetricValue::ok(s).at(
                crate::i18n::t_args(
                    "nucleo-score-metrica-periodo-n-exercicios",
                    &crate::i18n::args(&[("n", (roe_vals.len() as i64).into())]),
                ),
                last_fy,
                None,
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-poucos-roe")),
        },
    );
    let ni: Vec<f64> = yrs.iter().filter_map(|y| y.net_income).collect();
    m.insert(
        "loss_years_10y".into(),
        if ni.len() >= 3 {
            MetricValue::ok(ni.iter().filter(|x| **x < 0.0).count() as f64).at(
                crate::i18n::t_args(
                    "nucleo-score-metrica-periodo-n-exercicios",
                    &crate::i18n::args(&[("n", (ni.len() as i64).into())]),
                ),
                last_fy,
                Some(crate::i18n::t("nucleo-score-metrica-fonte-dfp")),
            )
        } else {
            MetricValue::missing(crate::i18n::t("nucleo-score-metrica-poucos-lucros-3"))
        },
    );
    let (vol, mom) = price_stats(i.prices, i.today);
    m.insert("volatility_1y".into(), vol);
    m.insert("momentum_12_1".into(), mom);

    // Remuneração.
    let declared = st_ltm_declared(st);
    put(
        &mut m,
        "payout",
        match (declared.or(t.dividends_paid), t.net_income) {
            (None, _) => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-dividendos"))
            }
            (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-lucro")),
            (Some(d), Some(n)) if n <= 0.0 && d > 0.0 => MetricValue::worst(crate::i18n::t(
                "nucleo-score-metrica-proventos-com-prejuizo",
            )),
            (Some(_), Some(n)) if n <= 0.0 => MetricValue::missing(crate::i18n::t(
                "nucleo-score-metrica-lucro-negativo-sem-proventos",
            )),
            (Some(d), Some(n)) => MetricValue::ok(d / n),
        },
        &ltm_p,
    );
    put(
        &mut m,
        "dividend_fcf_coverage",
        match (fcf, t.dividends_paid) {
            (_, None) => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-dividendos-pagos"))
            }
            (_, Some(d)) if d <= 0.0 => {
                MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-dividendos-12m"))
            }
            (None, _) => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-fcl")),
            (Some(c), Some(d)) => MetricValue::ok(c / d),
        },
        &ltm_p,
    );
    put(
        &mut m,
        "dilution",
        match div(t.shares, ya.shares.filter(|s| *s > 0.0)) {
            Some(v) => MetricValue::ok(v - 1.0),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-acoes-anterior")),
        },
        &bal_p,
    );

    // Para as flags.
    let fcf_years: Vec<f64> = yrs.iter().filter_map(Year::fcf).collect();
    let dividends_over_fcf_years = yrs
        .iter()
        .filter_map(|y| Some(y.dividends_paid? > y.fcf()?.max(0.0) && y.dividends_paid? > 0.0))
        .collect();
    let coverage_last_quarters = quarter_coverages(st);
    EquityOutput {
        metrics: m,
        frameworks: fws,
        dsri,
        coverage_last_quarters,
        fcf_years,
        dividends_over_fcf_years,
        negative_equity: t.equity.is_some_and(|e| e < 0.0),
    }
}

/// Dividendos e JCP declarados nos últimos 12 meses (DVA), se houver.
fn st_ltm_declared(st: &Statements<'_>) -> Option<f64> {
    let at = st.reference?;
    periods::ltm_at(st.facts, items::DIVIDENDS_DECLARED, at).map(|x| f(x.value).abs())
}

/// Cobertura de juros LTM nos dois últimos fins de trimestre.
fn quarter_coverages(st: &Statements<'_>) -> Vec<f64> {
    let Some(at) = st.reference else {
        return vec![];
    };
    let mut out = vec![];
    for back in [0, 3] {
        let d = add_months(at, -back);
        let get = |item: &str| periods::ltm_at(st.facts, item, d).map(|x| f(x.value).abs());
        let ebit = periods::ltm_at(st.facts, items::EBIT, d).map(|x| f(x.value));
        if let (Some(e), Some(fe)) = (ebit, get(keys::FIN_EXPENSE)) {
            let net = fe - get(keys::FIN_INCOME).unwrap_or(0.0);
            out.push(if net <= 0.0 { 99.0 } else { e / net });
        }
    }
    out
}

/// Fração das últimas 4 entregas no prazo (DFP: 3 meses após o exercício; ITR: 45 dias).
pub fn timely_filing(filings: &[(String, NaiveDate, NaiveDate)], today: NaiveDate) -> MetricValue {
    let mut fs: Vec<&(String, NaiveDate, NaiveDate)> = filings
        .iter()
        .filter(|(k, r, _)| (k == "dfp" || k == "itr") && *r >= add_months(today, -24))
        .collect();
    fs.sort_by_key(|x| x.1);
    let last: Vec<_> = fs.iter().rev().take(4).collect();
    if last.is_empty() {
        return MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-entregas"));
    }
    let on_time = last
        .iter()
        .filter(|(k, r, filed)| *filed <= deadline(k, *r))
        .count();
    MetricValue::ok(on_time as f64 / last.len() as f64).at(
        crate::i18n::t_args(
            "nucleo-score-metrica-periodo-entregas",
            &crate::i18n::args(&[("n", (last.len() as i64).into())]),
        ),
        last.first().map(|x| x.1),
        Some(crate::i18n::t("nucleo-score-metrica-fonte-recebimento")),
    )
}

/// Prazo de entrega na CVM.
pub fn deadline(kind: &str, reference: NaiveDate) -> NaiveDate {
    if kind == "dfp" {
        add_months(reference, 3)
    } else {
        reference + chrono::Duration::days(45)
    }
}

/// Volatilidade anualizada de 1 ano e momentum 12-1 a partir dos fechamentos.
pub fn price_stats(prices: &[(NaiveDate, f64)], today: NaiveDate) -> (MetricValue, MetricValue) {
    let since = add_months(today, -12);
    let year: Vec<&(NaiveDate, f64)> = prices.iter().filter(|p| p.0 > since && p.1 > 0.0).collect();
    let vol = if year.len() >= 120 {
        let rets: Vec<f64> = year.windows(2).map(|w| (w[1].1 / w[0].1).ln()).collect();
        match stdev(&rets) {
            Some(s) => MetricValue::ok(s * 252f64.sqrt()).at(
                crate::i18n::t_args(
                    "nucleo-score-metrica-periodo-pregoes",
                    &crate::i18n::args(&[
                        ("n", (year.len() as i64).into()),
                        ("data", br(year[year.len() - 1].0).into()),
                    ]),
                ),
                Some(year[year.len() - 1].0),
                Some(crate::i18n::t("nucleo-score-metrica-fonte-fechamentos")),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-retornos")),
        }
    } else {
        MetricValue::missing(crate::i18n::t("nucleo-score-metrica-poucos-fechamentos"))
    };
    let at = |d: NaiveDate| {
        prices
            .iter()
            .filter(|p| p.0 <= d && p.1 > 0.0)
            .max_by_key(|p| p.0)
    };
    let (a, b) = (at(add_months(today, -1)), at(add_months(today, -12)));
    let mom = match (a, b) {
        (Some(a), Some(b)) if (a.0 - b.0).num_days() > 300 && b.0 >= add_months(today, -13) => {
            MetricValue::ok(a.1 / b.1 - 1.0).at(
                crate::i18n::t_args(
                    "nucleo-score-periodo-de-a",
                    &crate::i18n::args(&[("de", br(b.0).into()), ("ate", br(a.0).into())]),
                ),
                Some(a.0),
                Some(crate::i18n::t("nucleo-score-metrica-fonte-fechamentos")),
            )
        }
        _ => MetricValue::missing(crate::i18n::t("nucleo-score-metrica-sem-cotacoes-12m")),
    };
    (vol, mom)
}

#[cfg(test)]
pub(crate) mod tests;
