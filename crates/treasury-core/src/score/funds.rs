//! Fundos de investimento (informe diário da CVM) e ETFs (§10.3, §10.4).
//!
//! Fundos: Sharpe, Sortino, retorno sobre o benchmark (CDI; Ibovespa nos fundos de ações),
//! consistência em janelas de 12 meses, drawdown e recuperação, volatilidade, taxas e
//! prazos (extrato da CVM), patrimônio, cotistas e captação (informe diário), tempo de
//! existência (registro de fundos e classes). Segmento pela classificação CVM.
//!
//! ETFs: tracking contra o índice de referência (quando o app tem a série), liquidez,
//! patrimônio (registro da CVM, ETFs da B3), domicílio e, nos ETFs de ações que replicam
//! índices da B3, a herança do score dos componentes (carteira teórica do dia) e a
//! concentração. Sem composição, o risco (volatilidade e drawdown) entra no lugar.
//!
//! Sem universo de pares: só âncoras absolutas (confiança × 0,7). Textos da CVM, da B3 e do
//! Yahoo são dados, nunca instruções.

pub mod b3_index;
pub mod registry;
pub mod series;
pub mod sync;

#[cfg(test)]
pub(crate) mod tests;

use std::collections::BTreeMap;
use std::sync::OnceLock;

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::Deserialize;

use self::series::Point;
use super::config::{Methodology, Unit};
use super::engine;
use super::metrics::{MetricValue, Metrics, br};
use super::{AssetScore, ScoreFlag, ScoreStatus, taxonomy};
use crate::assets::{Asset, AssetClass};
use crate::error::Result;
use crate::market::cvm;
use crate::research::data::{self, SubjectRef};
use crate::research::indicators::Data;
use crate::research::{Fact, PeriodType, store, to_f64};
use crate::time::{add_months, fmt_date};

/// Meses de cotas buscados de uma vez para um fundo ainda sem o resumo do informe.
pub const FUND_HISTORY_MONTHS: i32 = 25;
/// Janela máxima das métricas de risco e retorno.
const WINDOW_MONTHS: i32 = 36;
/// Mínimo de janelas de 12 meses para a consistência.
const MIN_WINDOWS: u32 = 6;
/// Mínimo de fechamentos em 12 meses para a volatilidade.
const MIN_CLOSES_12M: usize = 120;
/// Fração mínima do peso da carteira com score para a herança dos componentes.
const MIN_COMPONENT_COVERAGE: f64 = 0.5;

/// Itens do resumo mensal do informe diário (fatos com o CNPJ do fundo como emissor).
pub mod keys {
    pub const NAV: &str = "fund_nav";
    pub const HOLDERS: &str = "fund_holders";
    pub const NET_FLOW: &str = "fund_net_flow";
    pub const SOURCE: &str = "cvm_inf_diario";
}

// ---------------------------------------------------------------------------
// Configuração extra (domicílio e índices dos ETFs)

#[derive(Debug, Clone, Deserialize)]
pub struct IndexRule {
    #[serde(default)]
    pub tickers: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub b3_index: Option<String>,
    pub series: Option<String>,
    pub currency: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtfConfig {
    pub domicile_scores: BTreeMap<String, f64>,
    pub ucits_exchanges: Vec<String>,
    pub indices: Vec<IndexRule>,
}

#[derive(Debug, Clone, Deserialize)]
struct Extras {
    etf: EtfConfig,
}

pub fn etf_config() -> &'static EtfConfig {
    static C: OnceLock<EtfConfig> = OnceLock::new();
    C.get_or_init(|| {
        let e: Extras = serde_json::from_str(include_str!(
            "../../../../methodology/v0.1.0/classes/funds.json"
        ))
        .expect("classes/funds.json válido");
        e.etf
    })
}

/// Índice de referência de um ETF: ticker conhecido, ou palavra-chave no nome do ativo ou
/// na denominação da CVM.
pub fn index_rule<'a>(
    c: &'a EtfConfig,
    ticker: Option<&str>,
    names: &[&str],
) -> Option<&'a IndexRule> {
    let t = ticker.map(|t| t.trim().to_uppercase());
    if let Some(t) = &t
        && let Some(r) = c.indices.iter().find(|r| r.tickers.iter().any(|x| x == t))
    {
        return Some(r);
    }
    let text: Vec<String> = names.iter().map(|n| n.to_uppercase()).collect();
    c.indices.iter().find(|r| {
        r.keywords
            .iter()
            .any(|k| text.iter().any(|n| n.contains(&k.to_uppercase())))
    })
}

// ---------------------------------------------------------------------------
// Resumo mensal do informe diário

/// Já há resumo do informe para o fundo (senão a coleta busca o histórico de novo).
pub fn has_fund_months(conn: &Connection, cnpj: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM fundamental WHERE issuer_id = ?1 AND item = ?2)",
        rusqlite::params![cnpj, keys::NAV],
        |r| r.get(0),
    )?)
}

fn month_bounds(y: i32, m: u32) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(y, m, 1).expect("mês válido");
    (start, add_months(start, 1) - chrono::Duration::days(1))
}

/// Grava o resumo mensal (patrimônio e cotistas do último dia, captação líquida do mês).
pub fn store_fund_months(conn: &Connection, months: &cvm::Months) -> Result<()> {
    let now = crate::time::now_utc();
    for (cnpj, by) in months {
        let mut facts = vec![];
        for (&(y, m), x) in by {
            let (start, end) = month_bounds(y, m);
            let doc = crate::i18n::t_args(
                "nucleo-score-fundo-doc-informe",
                &crate::i18n::args(&[("data", br(x.last).into())]),
            );
            let mut push = |item: &str, value: Decimal, currency: &str| {
                facts.push(Fact {
                    item: item.into(),
                    period_type: PeriodType::Month,
                    start: Some(start),
                    end,
                    value,
                    currency: currency.into(),
                    source: keys::SOURCE.into(),
                    document: doc.clone(),
                    priority: 2,
                })
            };
            if let Some(n) = x.nav {
                push(keys::NAV, n, "BRL");
            }
            if let Some(h) = x.holders {
                push(keys::HOLDERS, Decimal::from(h), "count");
            }
            push(keys::NET_FLOW, x.inflow - x.outflow, "BRL");
        }
        store::upsert_facts(conn, cnpj, &facts, &now)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Auxiliares

fn ok(v: f64, period: impl Into<String>, source: impl Into<String>) -> MetricValue {
    MetricValue {
        period: Some(period.into()),
        source: Some(source.into()),
        ..MetricValue::ok(v)
    }
}

fn noted(mut v: MetricValue, note: impl Into<String>) -> MetricValue {
    v.note = Some(note.into());
    v
}

fn pct(x: f64) -> String {
    crate::ai::alerts::pct(crate::i18n::current(), x, 1)
}

fn num(x: f64) -> String {
    crate::ai::alerts::decimal(crate::i18n::current(), &format!("{x:.2}"))
}

/// "registro de <data>" (data do registro de fundos da CVM).
fn registry_on(d: String) -> String {
    crate::i18n::t_args(
        "nucleo-score-fundo-periodo-registro",
        &crate::i18n::args(&[("data", d.into())]),
    )
}

fn dec_f(s: &Option<String>) -> Option<f64> {
    s.as_deref()
        .and_then(|x| x.parse::<Decimal>().ok())
        .map(to_f64)
}

fn prices(conn: &Connection, asset_id: &str, from: NaiveDate, to: NaiveDate) -> Result<Vec<Point>> {
    Ok(data::price_series(conn, asset_id, from, to)?
        .into_iter()
        .map(|(d, p)| (d, to_f64(p)))
        .filter(|p| p.1 > 0.0 && p.1.is_finite())
        .collect())
}

fn index_levels(
    conn: &Connection,
    series: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<Point>> {
    Ok(crate::indices::points(conn, series, from, to)?
        .into_iter()
        .map(|p| (p.date, to_f64(p.value)))
        .filter(|p| p.1 > 0.0)
        .collect())
}

/// CDI acumulado como série de níveis.
fn cdi_levels(conn: &Connection, from: NaiveDate, to: NaiveDate) -> Result<Vec<Point>> {
    let rates: Vec<(NaiveDate, f64)> = crate::indices::daily_rates(conn, "cdi", from, to)?
        .into_iter()
        .map(|(d, r)| (d, to_f64(r)))
        .collect();
    Ok(series::level_from_daily_rates(&rates))
}

/// Série de níveis convertida de moeda pelo câmbio de cada data (sem câmbio, o ponto sai).
fn in_currency(conn: &Connection, pts: &[Point], from: &str, to: &str) -> Result<Vec<Point>> {
    let mut out = Vec::with_capacity(pts.len());
    for (d, v) in pts {
        if let Some((r, rd, _, _)) = crate::fx::rate(conn, from, to, &fmt_date(*d))?
            && (*d - crate::time::parse_date(&rd)?).num_days() <= 7
        {
            out.push((*d, v * to_f64(r)));
        }
    }
    Ok(out)
}

/// A série de referência cobre a janela (começa até 10 dias depois e termina até 10 dias
/// antes do ativo).
fn covers(bench: &[Point], pts: &[Point]) -> bool {
    match (bench.first(), bench.last(), pts.first(), pts.last()) {
        (Some(b0), Some(b1), Some(p0), Some(p1)) => {
            (b0.0 - p0.0).num_days() <= 10 && (p1.0 - b1.0).num_days() <= 10
        }
        _ => false,
    }
}

fn flag(m: &Methodology, key: &str, detail: String) -> Option<ScoreFlag> {
    let d = m.flags.get(key)?;
    Some(ScoreFlag {
        key: key.into(),
        label: m.flag_label(key),
        severity: d.severity,
        cap: d.cap,
        penalty: d.penalty,
        detail,
    })
}

fn threshold(m: &Methodology, key: &str) -> Option<f64> {
    m.flags.get(key).and_then(|f| f.threshold)
}

/// Volume médio em reais (ETFs no exterior convertidos pela PTAX do dia).
fn liquidity_brl(
    conn: &Connection,
    d: &Data,
    currency: &str,
    today: NaiveDate,
) -> Result<MetricValue> {
    let Some((v, src)) = &d.liquidity else {
        return Ok(MetricValue::missing(crate::i18n::t(
            "nucleo-score-fundo-sem-volume",
        )));
    };
    let v = to_f64(*v);
    if currency == "BRL" {
        return Ok(ok(
            v,
            crate::i18n::t("nucleo-score-periodo-ultimos-pregoes"),
            src.clone(),
        ));
    }
    Ok(
        match crate::fx::rate(conn, currency, "BRL", &fmt_date(today))? {
            Some((r, rd, _, _)) => ok(
                v * to_f64(r),
                crate::i18n::t("nucleo-score-periodo-ultimos-pregoes"),
                crate::i18n::t_args(
                    "nucleo-score-fundo-fonte-convertido",
                    &crate::i18n::args(&[
                        ("fonte", src.clone().into()),
                        ("moeda", currency.to_string().into()),
                        ("data", rd.to_string().into()),
                    ]),
                ),
            ),
            None => MetricValue::missing(crate::i18n::t_args(
                "nucleo-score-fundo-sem-cambio",
                &crate::i18n::args(&[("moeda", currency.to_string().into())]),
            )),
        },
    )
}

/// Shell do resultado com o segmento e o perfil da classe.
fn shell(
    m: &Methodology,
    d: &Data,
    class: &str,
    segment: &str,
    today: NaiveDate,
) -> (AssetScore, String) {
    let cs = &m.class_segments[class];
    let seg = cs
        .segments
        .get(segment)
        .unwrap_or(&cs.segments[&cs.default_segment]);
    let key = if cs.segments.contains_key(segment) {
        segment.to_string()
    } else {
        cs.default_segment.clone()
    };
    let profile = &m.profiles[&seg.profile];
    (
        super::assess::shell_for(
            m,
            d,
            class,
            &key,
            &seg.label,
            &seg.profile,
            Some(profile),
            today,
        ),
        key,
    )
}

fn override_for(conn: &Connection, d: &Data) -> Result<Option<String>> {
    let o = taxonomy::overrides(conn)?;
    Ok(d.subject.ticker.as_ref().and_then(|t| o.get(t).cloned()))
}

/// Roda o motor sem pares e copia para o resultado.
fn finish(
    m: &Methodology,
    s: &mut AssetScore,
    metrics: &Metrics,
    flags: Vec<ScoreFlag>,
    data_date: Option<NaiveDate>,
    today: NaiveDate,
) {
    let profile = &m.profiles[&s.profile];
    let o = engine::run(engine::Input {
        m,
        profile,
        id: None,
        sector: None,
        sector_label: None,
        segment: &s.segment.clone(),
        segment_label: &s.segment_label.clone(),
        metrics,
        peers: &[],
        flags,
        data_date,
        today,
        extra_confidence: vec![],
    });
    super::assess::apply(s, o, vec![], vec![], data_date);
}

pub fn assess(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    match d.subject.class {
        AssetClass::EtfBr | AssetClass::EtfUs | AssetClass::EtfIntl => {
            assess_etf(conn, m, d, today)
        }
        _ => assess_fund(conn, m, d, today),
    }
}

// ---------------------------------------------------------------------------
// Fundos

/// Segmento de um fundo pela classe do cadastro e pela classificação da CVM.
pub fn fund_segment(class: AssetClass, info: Option<&registry::FundInfo>) -> &'static str {
    let low = |s: &Option<String>| s.as_deref().map(crate::research::norm).unwrap_or_default();
    if class == AssetClass::Pension {
        return "previdencia";
    }
    let Some(i) = info else {
        return "multimercado";
    };
    let (c, a) = (low(&i.classification), low(&i.anbima));
    if a.starts_with("previdencia") {
        "previdencia"
    } else if c.starts_with("renda fixa") || (c.is_empty() && a.starts_with("renda fixa")) {
        "renda_fixa"
    } else if c.starts_with("acoes") || (c.is_empty() && a.starts_with("acoes")) {
        "acoes"
    } else {
        "multimercado"
    }
}

/// Como [`fund_segment`], e sem o registro da CVM pelo nome do fundo ("RF", "Referenciado DI",
/// "Ações", "FIA", "Prev"), antes de cair em multimercado.
pub fn fund_segment_named(
    class: AssetClass,
    info: Option<&registry::FundInfo>,
    name: &str,
) -> &'static str {
    if class == AssetClass::Pension || info.is_some() {
        return fund_segment(class, info);
    }
    let key = format!(" {} ", registry::fund_key(name));
    let has = |w: &str| key.contains(&format!(" {w} "));
    if has("prev") || has("previdencia") || has("previdenciario") || has("pgbl") || has("vgbl") {
        "previdencia"
    } else if has("multimercado") {
        "multimercado"
    } else if has("renda fixa") || has("referenciado") || has("di") || has("simples") {
        "renda_fixa"
    } else if has("acoes") {
        "acoes"
    } else {
        "multimercado"
    }
}

/// Benchmark de um fundo: Ibovespa nos de ações (ou com Ibovespa/IBrX declarado), senão CDI.
pub fn fund_benchmark(
    segment: &str,
    info: Option<&registry::FundInfo>,
) -> (&'static str, &'static str) {
    let b = info
        .and_then(|i| i.benchmark.as_deref())
        .map(crate::research::norm)
        .unwrap_or_default();
    if segment == "acoes" || b.contains("ibovespa") || b.starts_with("ibrx") {
        ("ibov", "Ibovespa")
    } else {
        ("cdi", "CDI")
    }
}

/// Entradas puras das métricas de um fundo.
pub struct FundInput<'a> {
    pub quotes: &'a [Point],
    pub cdi: &'a [Point],
    /// Benchmark (o próprio CDI ou o Ibovespa) e o nome dele.
    pub bench: &'a [Point],
    pub bench_label: &'a str,
    pub info: Option<&'a registry::FundInfo>,
    /// Resumo mensal do informe (fatos do fundo).
    pub facts: &'a [Fact],
    pub today: NaiveDate,
}

pub struct FundOutput {
    pub metrics: Metrics,
    pub notes: Vec<String>,
    pub data_date: Option<NaiveDate>,
}

pub fn fund_metrics(i: &FundInput<'_>) -> FundOutput {
    let mut ms = Metrics::new();
    let mut notes = vec![];
    let quotes = i.quotes;
    let last = quotes.last().map(|p| p.0);
    let src = &crate::i18n::t("nucleo-score-fundo-fonte-cotas");
    // `what`: chave do que falta medir (`nucleo-score-fundo-para-*`).
    let short = |what: &str| {
        MetricValue::missing(crate::i18n::t_args(
            "nucleo-score-fundo-poucas-cotas",
            &crate::i18n::args(&[("para", crate::i18n::t(what).into())]),
        ))
    };
    // Janela de até 36 meses terminando na última cota.
    let win: Vec<Point> = match last {
        Some(l) => series::window(quotes, add_months(l, -WINDOW_MONTHS), l),
        None => vec![],
    };
    let years = series::span_years(&win);
    let enough = years >= 0.95;
    let period = match (win.first(), win.last()) {
        (Some(a), Some(b)) => crate::i18n::t_args(
            "nucleo-score-fundo-periodo-anos",
            &crate::i18n::args(&[
                ("de", br(a.0).into()),
                ("ate", br(b.0).into()),
                (
                    "anos",
                    crate::ai::alerts::decimal(crate::i18n::current(), &format!("{years:.1}"))
                        .into(),
                ),
            ]),
        ),
        _ => String::new(),
    };
    let is_cdi = i.bench_label == "CDI";
    // Sharpe e Sortino contra o CDI.
    let rr = (enough && covers(i.cdi, &win))
        .then(|| series::risk_return(&win, i.cdi))
        .flatten();
    let no_cdi = || {
        if enough {
            MetricValue::missing(crate::i18n::t("nucleo-score-fundo-sem-cdi"))
        } else {
            short("nucleo-score-fundo-para-indice")
        }
    };
    match rr {
        Some(r) => {
            // Métricas de nota (unidade "score"): a explicação mostra a nota da métrica, por
            // isso o valor vem no começo.
            let note = |x: f64| {
                crate::i18n::t_args(
                    "nucleo-score-fundo-nota-sharpe",
                    &crate::i18n::args(&[
                        ("valor", num(x).into()),
                        ("retorno", pct(r.ret).into()),
                        ("cdi", pct(r.rf).into()),
                        ("volatilidade", pct(r.vol).into()),
                    ]),
                )
            };
            ms.insert(
                "fund_sharpe".into(),
                match r.sharpe {
                    Some(x) => noted(ok(x, &period, src), note(x)),
                    None => {
                        MetricValue::missing(crate::i18n::t("nucleo-score-fundo-volatilidade-zero"))
                    }
                },
            );
            ms.insert(
                "fund_sortino".into(),
                match r.sortino {
                    Some(x) => noted(ok(x, &period, src), note(x)),
                    None => noted(
                        ok(10.0, &period, src),
                        crate::i18n::t("nucleo-score-fundo-sortino-teto"),
                    ),
                },
            );
        }
        None => {
            ms.insert("fund_sharpe".into(), no_cdi());
            ms.insert("fund_sortino".into(), no_cdi());
        }
    }
    // Retorno sobre o benchmark e information ratio.
    let rel = (enough && covers(i.bench, &win))
        .then(|| series::relative(&win, i.bench))
        .flatten();
    match rel {
        Some(r) => {
            ms.insert(
                "fund_excess_return".into(),
                noted(
                    ok(r.excess, &period, format!("{src}; {}", i.bench_label)),
                    crate::i18n::t_args(
                        "nucleo-score-fundo-nota-benchmark",
                        &crate::i18n::args(&[("benchmark", i.bench_label.to_string().into())]),
                    ),
                ),
            );
            ms.insert(
                "fund_information_ratio".into(),
                if is_cdi {
                    MetricValue::missing(crate::i18n::t("nucleo-score-fundo-benchmark-cdi"))
                } else {
                    match r.information_ratio {
                        Some(x) => noted(
                            ok(x, &period, src),
                            crate::i18n::t_args(
                                "nucleo-score-fundo-nota-information-ratio",
                                &crate::i18n::args(&[
                                    ("valor", num(x).into()),
                                    ("te", pct(r.tracking_error).into()),
                                ]),
                            ),
                        ),
                        None => {
                            MetricValue::missing(crate::i18n::t("nucleo-score-fundo-tracking-zero"))
                        }
                    }
                },
            );
        }
        None => {
            let why = if enough {
                MetricValue::missing(crate::i18n::t_args(
                    "nucleo-score-fundo-sem-serie",
                    &crate::i18n::args(&[("benchmark", i.bench_label.to_string().into())]),
                ))
            } else {
                short("nucleo-score-fundo-para-benchmark")
            };
            ms.insert("fund_excess_return".into(), why.clone());
            ms.insert("fund_information_ratio".into(), why);
        }
    }
    // Consistência: janelas de 12 meses (até 36 meses de histórico).
    let (n, wins) = series::rolling_12m_wins(&win, i.bench);
    ms.insert(
        "fund_consistency".into(),
        if n >= MIN_WINDOWS {
            noted(
                ok(
                    wins as f64 / n as f64,
                    &period,
                    format!("{src}; {}", i.bench_label),
                ),
                crate::i18n::t_args(
                    "nucleo-score-fundo-nota-janelas",
                    &crate::i18n::args(&[
                        ("acima", (wins as i64).into()),
                        ("janelas", (n as i64).into()),
                        ("benchmark", i.bench_label.to_string().into()),
                    ]),
                ),
            )
        } else {
            MetricValue::missing(crate::i18n::t_args(
                "nucleo-score-fundo-poucas-janelas",
                &crate::i18n::args(&[
                    ("minimo", (MIN_WINDOWS as i64).into()),
                    ("benchmark", i.bench_label.to_string().into()),
                ]),
            ))
        },
    );
    // Drawdown e recuperação.
    match series::max_drawdown(&win).filter(|_| enough) {
        Some(dd) => {
            let note = if dd.depth == 0.0 {
                crate::i18n::t("nucleo-score-fundo-sem-queda")
            } else {
                let base = crate::i18n::t_args(
                    "nucleo-score-nota-pico-fundo",
                    &crate::i18n::args(&[
                        ("pico", br(dd.peak).into()),
                        ("fundo", br(dd.trough).into()),
                    ]),
                );
                let rec = match dd.recovered {
                    Some(r) => crate::i18n::t_args(
                        "nucleo-score-fundo-recuperada",
                        &crate::i18n::args(&[("data", br(r).into())]),
                    ),
                    None => crate::i18n::t("nucleo-score-fundo-nao-recuperada"),
                };
                format!("{base}{rec}")
            };
            ms.insert(
                "fund_max_drawdown".into(),
                noted(ok(dd.depth, &period, src), &note),
            );
            ms.insert(
                "fund_recovery_days".into(),
                noted(ok(dd.recovery_days as f64, &period, src), &note),
            );
        }
        None => {
            ms.insert(
                "fund_max_drawdown".into(),
                short("nucleo-score-fundo-para-drawdown"),
            );
            ms.insert(
                "fund_recovery_days".into(),
                short("nucleo-score-fundo-para-recuperacao"),
            );
        }
    }
    // Volatilidade de 12 meses.
    let y1: Vec<Point> = match last {
        Some(l) => series::window(quotes, add_months(l, -12), l),
        None => vec![],
    };
    ms.insert(
        "fund_volatility".into(),
        match series::volatility(&y1).filter(|_| y1.len() >= MIN_CLOSES_12M) {
            Some(v) => ok(
                v,
                crate::i18n::t_args(
                    "nucleo-score-fundo-periodo-cotas",
                    &crate::i18n::args(&[
                        ("n", (y1.len() as i64).into()),
                        ("data", br(y1[y1.len() - 1].0).into()),
                    ]),
                ),
                src,
            ),
            None => MetricValue::missing(crate::i18n::t_args(
                "nucleo-score-fundo-poucas-cotas-12m",
                &crate::i18n::args(&[("n", (MIN_CLOSES_12M as i64).into())]),
            )),
        },
    );
    // Custos (extrato da CVM).
    let ext_period = i
        .info
        .and_then(|x| x.extrato_date.clone())
        .map(|d| {
            crate::i18n::t_args(
                "nucleo-score-fundo-periodo-extrato",
                &crate::i18n::args(&[("data", d.into())]),
            )
        })
        .unwrap_or_default();
    let ext_src = &crate::i18n::t("nucleo-score-fundo-fonte-extrato");
    let no_extrato = || MetricValue::missing(crate::i18n::t("nucleo-score-fundo-sem-extrato"));
    ms.insert(
        "fund_admin_fee".into(),
        match i.info.and_then(|x| dec_f(&x.admin_fee)) {
            Some(v) => ok(v, &ext_period, ext_src),
            None => no_extrato(),
        },
    );
    ms.insert(
        "fund_perf_fee".into(),
        match i.info.and_then(|x| dec_f(&x.perf_fee)) {
            Some(v) => {
                let mv = ok(v, &ext_period, ext_src);
                match i.info.and_then(|x| x.perf_benchmark.clone()) {
                    Some(b) if v > 0.0 => noted(
                        mv,
                        crate::i18n::t_args(
                            "nucleo-score-fundo-nota-performance",
                            &crate::i18n::args(&[("benchmark", b.into())]),
                        ),
                    ),
                    _ => mv,
                }
            }
            None => no_extrato(),
        },
    );
    ms.insert(
        "fund_redemption_days".into(),
        match i.info.and_then(|x| x.redemption_days) {
            Some(v) => ok(v as f64, &ext_period, ext_src),
            None => no_extrato(),
        },
    );
    // Patrimônio, cotistas e captação (informe diário; patrimônio do registro em último caso).
    let monthly = |item: &str| -> Vec<&Fact> {
        let mut v: Vec<&Fact> = i
            .facts
            .iter()
            .filter(|f| f.item == item && f.period_type == PeriodType::Month)
            .collect();
        v.sort_by_key(|f| f.end);
        v
    };
    let navs = monthly(keys::NAV);
    let nav_now = navs
        .last()
        .map(|f| (to_f64(f.value), f.document.clone(), f.end));
    ms.insert(
        "fund_nav".into(),
        match (&nav_now, i.info.and_then(|x| dec_f(&x.nav))) {
            (Some((v, doc, _)), _) => ok(
                *v,
                doc.clone(),
                crate::i18n::t("nucleo-score-fundo-fonte-patrimonio"),
            ),
            (None, Some(v)) => ok(
                v,
                i.info
                    .and_then(|x| x.nav_date.clone())
                    .map(registry_on)
                    .unwrap_or_default(),
                crate::i18n::t("nucleo-score-fundo-fonte-registro"),
            ),
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-fundo-sem-patrimonio")),
        },
    );
    ms.insert(
        "fund_holders".into(),
        match monthly(keys::HOLDERS).last() {
            Some(f) => ok(
                to_f64(f.value),
                f.document.clone(),
                crate::i18n::t("nucleo-score-fundo-fonte-cotistas"),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-fundo-sem-cotistas")),
        },
    );
    let flows = monthly(keys::NET_FLOW);
    let recent: Vec<&&Fact> = flows.iter().rev().take(12).collect();
    ms.insert(
        "fund_net_flow_12m".into(),
        match &nav_now {
            Some((nav, doc, end)) if recent.len() >= 11 && *nav > 0.0 => {
                let sum: f64 = recent.iter().map(|f| to_f64(f.value)).sum();
                let first = recent.last().map(|f| f.end).unwrap_or(*end);
                noted(
                    ok(
                        sum / nav,
                        crate::i18n::t_args(
                            "nucleo-score-fundo-periodo-captacao",
                            &crate::i18n::args(&[
                                ("meses", (recent.len() as i64).into()),
                                ("de", first.format("%m/%Y").to_string().into()),
                                ("ate", end.format("%m/%Y").to_string().into()),
                                ("documento", doc.clone().into()),
                            ]),
                        ),
                        crate::i18n::t("nucleo-score-fundo-fonte-captacao"),
                    ),
                    crate::i18n::t_args(
                        "nucleo-score-fundo-nota-captacao",
                        &crate::i18n::args(&[(
                            "valor",
                            engine::fmt_value(Unit::Money, sum).into(),
                        )]),
                    ),
                )
            }
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-fundo-pouca-captacao")),
        },
    );
    ms.insert(
        "fund_age_years".into(),
        match i
            .info
            .and_then(|x| x.inception.as_deref())
            .and_then(|s| crate::time::parse_date(s).ok())
        {
            Some(d) => noted(
                ok(
                    (i.today - d).num_days() as f64 / 365.25,
                    crate::i18n::t_args(
                        "nucleo-score-fundo-periodo-constituido",
                        &crate::i18n::args(&[("data", br(d).into())]),
                    ),
                    crate::i18n::t("nucleo-score-fundo-fonte-registro"),
                ),
                crate::i18n::t("nucleo-score-fundo-nota-tempo-gestao"),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-fundo-sem-constituicao")),
        },
    );
    // Retornos para a explicação (não pontuam).
    if let Some(l) = last {
        let mut parts = vec![];
        for months in [12, 24, 36] {
            let w = series::window(quotes, add_months(l, -months), l);
            if series::span_years(&w) < months as f64 / 12.0 - 0.05 {
                continue;
            }
            let b = series::window(i.bench, add_months(l, -months), l);
            let (Some(r), years) = (series::total_return(&w), series::span_years(&w)) else {
                continue;
            };
            let ann = series::annualized(r, years).unwrap_or(r);
            let bench = series::total_return(&b)
                .and_then(|x| series::annualized(x, years))
                .map(|x| format!(" ({} {})", i.bench_label, pct(x)))
                .unwrap_or_default();
            parts.push(crate::i18n::t_args(
                "nucleo-score-fundo-retorno-meses",
                &crate::i18n::args(&[
                    ("meses", i64::from(months).into()),
                    ("pct", pct(ann).into()),
                    ("benchmark", bench.into()),
                ]),
            ));
        }
        if !parts.is_empty() {
            notes.push(crate::i18n::t_args(
                "nucleo-score-fundo-nota-retorno",
                &crate::i18n::args(&[("retornos", parts.join("; ").into())]),
            ));
        }
    }
    FundOutput {
        metrics: ms,
        notes,
        data_date: last,
    }
}

pub fn fund_flags(m: &Methodology, segment: &str, ms: &Metrics) -> Vec<ScoreFlag> {
    let val = |k: &str| ms.get(k).and_then(|x| x.value);
    let mut v = vec![];
    if let (Some(x), Some(t)) = (val("fund_nav"), threshold(m, "fund_small_nav"))
        && x < t
    {
        v.extend(flag(
            m,
            "fund_small_nav",
            crate::i18n::t_args(
                "nucleo-score-fundo-flag-patrimonio",
                &crate::i18n::args(&[("valor", engine::fmt_value(Unit::Money, x).into())]),
            ),
        ));
    }
    if let (Some(x), Some(t)) = (
        val("fund_net_flow_12m"),
        threshold(m, "fund_heavy_outflows"),
    ) && x < t
    {
        v.extend(flag(
            m,
            "fund_heavy_outflows",
            crate::i18n::t_args(
                "nucleo-score-fundo-flag-resgates",
                &crate::i18n::args(&[("pct", pct(x).into())]),
            ),
        ));
    }
    if segment == "renda_fixa"
        && let (Some(x), Some(t)) = (val("fund_max_drawdown"), threshold(m, "fund_deep_drawdown"))
        && x > t
    {
        v.extend(flag(
            m,
            "fund_deep_drawdown",
            crate::i18n::t_args(
                "nucleo-score-fundo-flag-queda",
                &crate::i18n::args(&[("pct", pct(x).into())]),
            ),
        ));
    }
    v
}

/// Por que um fundo cadastrado não tem cotas: pela fonte que a coleta usaria de fato
/// (`market::sync::default_source`), não só pelo provedor escolhido.
fn no_quotes_reason(a: &Asset, cnpj: Option<&str>, reg: &registry::Registry) -> String {
    let source = crate::market::sync::default_source(a);
    let cvm = matches!(source, Some((crate::market::sync::Provider::Cvm, _)));
    match (cnpj, a.price_provider.as_deref()) {
        (_, Some("manual") | Some("none")) => crate::i18n::t("nucleo-score-fundo-cotacao-manual"),
        (None, _) => match reg
            .fund_guesses
            .get(&a.id)
            .map(|c| (c, reg.funds.get(c).map(|f| f.name.as_str()).unwrap_or("")))
        {
            Some((c, name)) => crate::i18n::t_args(
                "nucleo-score-fundo-sem-cnpj-provavel",
                &crate::i18n::args(&[
                    ("cnpj", crate::research::format_cnpj(c).into()),
                    ("nome", name.to_string().into()),
                ]),
            ),
            None => crate::i18n::t("nucleo-score-fundo-sem-cnpj"),
        },
        (Some(_), _) if !cvm => crate::i18n::t("nucleo-score-fundo-fonte-nao-cvm"),
        (Some(c), _)
            if reg.registro_at.is_some()
                && !reg.funds.contains_key(c)
                && reg.fund_guesses.contains_key(&a.id) =>
        {
            let g = &reg.fund_guesses[&a.id];
            crate::i18n::t_args(
                "nucleo-score-fundo-cnpj-plano-provavel",
                &crate::i18n::args(&[
                    ("cnpj", crate::research::format_cnpj(c).into()),
                    ("provavel", crate::research::format_cnpj(g).into()),
                    (
                        "nome",
                        reg.funds
                            .get(g)
                            .map(|f| f.name.clone())
                            .unwrap_or_default()
                            .into(),
                    ),
                ]),
            )
        }
        (Some(c), _) if reg.registro_at.is_some() && !reg.funds.contains_key(c) => {
            let when = reg
                .registro_at
                .as_deref()
                .and_then(|d| d.get(..10))
                .unwrap_or("");
            crate::i18n::t_args(
                "nucleo-score-fundo-cnpj-fora-do-registro",
                &crate::i18n::args(&[
                    ("cnpj", crate::research::format_cnpj(c).into()),
                    ("data", when.to_string().into()),
                    (
                        "tipo",
                        (if a.class == AssetClass::Pension {
                            "previdencia"
                        } else {
                            "fundo"
                        })
                        .into(),
                    ),
                ]),
            )
        }
        _ => crate::i18n::t("nucleo-score-fundo-sem-cotas"),
    }
}

/// Item que falta no fundo de pensão sem extrato importado: dado do usuário, não da coleta.
pub const PLAN_STATEMENT: &str = "plan_statement";

fn assess_fund(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    let asset: Option<Asset> = match &d.subject.asset_id {
        Some(id) => Some(crate::assets::get(conn, id)?),
        None => None,
    };
    let reg = registry::load(conn)?;
    let cnpj = asset
        .as_ref()
        .and_then(|a| registry::cnpj_of(&reg, &a.id, a.cnpj.as_deref()));
    let info = cnpj.as_ref().and_then(|c| reg.funds.get(c));
    // Sem CNPJ, o segmento sai do fundo provável pelo nome (só a classificação) ou do nome.
    let guess = match (&asset, cnpj.is_none()) {
        (Some(a), true) => reg.fund_guesses.get(&a.id).and_then(|c| reg.funds.get(c)),
        _ => None,
    };
    let segment = override_for(conn, d)?.unwrap_or_else(|| {
        fund_segment_named(d.subject.class, info.or(guess), &d.subject.name).to_string()
    });
    let (mut s, segment) = shell(m, d, "fund", &segment, today);
    let Some(a) = asset else {
        return Ok(super::assess::without_data_for(
            s,
            &crate::i18n::t("nucleo-score-fundo-dado-cotas"),
            &crate::i18n::t("nucleo-score-fundo-so-cadastrados"),
        ));
    };
    let quotes = prices(conn, &a.id, add_months(today, -WINDOW_MONTHS - 1), today)?;
    // Fundo de pensão (EFPC): não é fundo da CVM; a cota do plano só vem do extrato.
    if quotes.is_empty() && a.class == AssetClass::Pension && crate::pension::is_efpc(conn, &a.id)?
    {
        return Ok(super::assess::without_data_keyed(
            s,
            PLAN_STATEMENT,
            &crate::i18n::t("nucleo-score-fundo-dado-cotas-extrato"),
            &crate::i18n::t("nucleo-score-fundo-efpc-sem-cotas"),
        ));
    }
    if quotes.is_empty() {
        return Ok(super::assess::without_data_for(
            s,
            &crate::i18n::t("nucleo-score-fundo-dado-cotas-cvm"),
            &no_quotes_reason(&a, cnpj.as_deref(), &reg),
        ));
    }
    let from = quotes[0].0 - chrono::Duration::days(10);
    let cdi = cdi_levels(conn, from, today)?;
    let (bench_id, bench_label) = fund_benchmark(&segment, info);
    let bench = if bench_id == "cdi" {
        cdi.clone()
    } else {
        index_levels(conn, bench_id, from, today)?
    };
    let facts = match &cnpj {
        Some(c) => store::load_facts(conn, c)?,
        None => vec![],
    };
    let o = fund_metrics(&FundInput {
        quotes: &quotes,
        cdi: &cdi,
        bench: &bench,
        bench_label,
        info,
        facts: &facts,
        today,
    });
    let flags = fund_flags(m, &segment, &o.metrics);
    finish(m, &mut s, &o.metrics, flags, o.data_date, today);
    if let Some(i) = info {
        let mut parts = vec![];
        if let Some(c) = &i.classification {
            parts.push(crate::i18n::t_args(
                "nucleo-score-fundo-cadastro-classe",
                &crate::i18n::args(&[("classe", c.clone().into())]),
            ));
        }
        if let Some(b) = &i.benchmark {
            parts.push(crate::i18n::t_args(
                "nucleo-score-fundo-cadastro-indicador",
                &crate::i18n::args(&[("indicador", b.clone().into())]),
            ));
        }
        if let Some(g) = &i.manager {
            parts.push(crate::i18n::t_args(
                "nucleo-score-fundo-cadastro-gestor",
                &crate::i18n::args(&[("gestor", g.clone().into())]),
            ));
        }
        if !parts.is_empty() {
            let text = parts.join("; ");
            s.notes.push(crate::i18n::t_args(
                "nucleo-score-fundo-nota-cadastro",
                &crate::i18n::args(&[("cadastro", text.trim_end_matches('.').to_string().into())]),
            ));
        }
        if i.exclusive {
            s.notes
                .push(crate::i18n::t("nucleo-score-fundo-nota-exclusivo"));
        }
    } else {
        s.notes
            .push(crate::i18n::t("nucleo-score-fundo-nota-sem-cadastro"));
    }
    s.notes.push(crate::i18n::t_args(
        "nucleo-score-fundo-nota-benchmark-pares",
        &crate::i18n::args(&[("benchmark", bench_label.to_string().into())]),
    ));
    s.notes.extend(o.notes);
    Ok(s)
}

// ---------------------------------------------------------------------------
// ETFs

/// Domicílio de um ETF (chave da tabela de notas) e a explicação (no idioma da instalação).
pub fn etf_domicile(c: &EtfConfig, a: &Asset) -> (&'static str, String) {
    let (key, why) = etf_domicile_key(c, a);
    (key, crate::i18n::t(why))
}

fn etf_domicile_key(c: &EtfConfig, a: &Asset) -> (&'static str, &'static str) {
    match a.class {
        AssetClass::EtfBr => ("br", "nucleo-score-etf-domicilio-br"),
        AssetClass::EtfUs => ("us", "nucleo-score-etf-domicilio-us"),
        _ => {
            let code = a
                .exchange
                .as_ref()
                .map(|e| e.code.to_uppercase())
                .or_else(|| a.market.clone().map(|m| m.to_uppercase()))
                .unwrap_or_default();
            if c.ucits_exchanges.contains(&code) {
                ("ucits", "nucleo-score-etf-domicilio-ucits")
            } else {
                ("other", "nucleo-score-etf-domicilio-outro")
            }
        }
    }
}

/// Herança do score dos componentes: média ponderada e fração do peso com score.
pub fn components_score(
    holdings: &[b3_index::Holding],
    score_of: &mut dyn FnMut(&str) -> Option<f64>,
) -> (Option<f64>, f64, usize) {
    let total: f64 = holdings.iter().map(|h| h.weight).sum();
    let (mut acc, mut w, mut n) = (0.0, 0.0, 0);
    for h in holdings {
        if let Some(s) = score_of(&h.ticker) {
            acc += h.weight * s;
            w += h.weight;
            n += 1;
        }
    }
    let coverage = if total > 0.0 { w / total } else { 0.0 };
    ((w > 0.0).then(|| acc / w), coverage, n)
}

fn assess_etf(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    let cfg = etf_config();
    let asset: Option<Asset> = match &d.subject.asset_id {
        Some(id) => Some(crate::assets::get(conn, id)?),
        None => None,
    };
    let reg = registry::load(conn)?;
    let cnpj = asset
        .as_ref()
        .and_then(|a| registry::cnpj_of(&reg, &a.id, a.cnpj.as_deref()));
    let info = cnpj.as_ref().and_then(|c| reg.funds.get(c));
    let mut names: Vec<&str> = vec![d.subject.name.as_str()];
    if let Some(i) = info {
        names.push(i.name.as_str());
    }
    if let Some(n) = asset.as_ref().and_then(|a| reg.etf_names.get(&a.id)) {
        names.push(n.as_str());
    }
    let rule = index_rule(cfg, d.subject.ticker.as_deref(), &names);
    let portfolios = b3_index::load(conn)?;
    let portfolio = rule
        .and_then(|r| r.b3_index.as_ref())
        .and_then(|i| portfolios.get(i).map(|p| (i.clone(), p)));
    let auto = match d.subject.class {
        AssetClass::EtfBr if rule.is_some_and(|r| r.b3_index.is_some()) => "acoes_br",
        AssetClass::EtfUs | AssetClass::EtfIntl => "exterior",
        _ => "outros",
    };
    let segment = override_for(conn, d)?.unwrap_or_else(|| auto.to_string());
    let (mut s, segment) = shell(m, d, "etf", &segment, today);
    let Some(a) = asset else {
        return Ok(super::assess::without_data_for(
            s,
            &crate::i18n::t("nucleo-score-etf-dado-cotacoes"),
            &crate::i18n::t("nucleo-score-etf-so-cadastrados"),
        ));
    };
    let px = prices(conn, &a.id, add_months(today, -WINDOW_MONTHS - 1), today)?;
    let mut ms = Metrics::new();
    let mut notes: Vec<String> = vec![];
    let last = px.last().map(|p| p.0);
    let src = crate::i18n::t_args(
        "nucleo-score-etf-fonte-cotacoes",
        &crate::i18n::args(&[(
            "fonte",
            d.price
                .as_ref()
                .map(|p| p.1.source.clone())
                .unwrap_or_default()
                .into(),
        )]),
    );
    // Tracking contra a série do índice (mesma moeda).
    let y1: Vec<Point> = match last {
        Some(l) => series::window(&px, add_months(l, -12), l),
        None => vec![],
    };
    let tracking = match rule.and_then(|r| r.series.as_deref().map(|s| (r, s))) {
        None => Err(crate::i18n::t("nucleo-score-etf-sem-indice")),
        Some((r, sid)) => {
            let idx = match (y1.first(), last) {
                (Some(f), Some(l)) => index_levels(conn, sid, f.0 - chrono::Duration::days(10), l)?,
                _ => vec![],
            };
            // Índice em outra moeda (ex.: S&P 500 para um ETF da B3): convertido pelo câmbio do dia.
            let converted = r.currency != a.currency;
            let (idx, label) = if !converted {
                (idx, sid.to_string())
            } else {
                (
                    in_currency(conn, &idx, &r.currency, &a.currency)?,
                    crate::i18n::t_args(
                        "nucleo-score-etf-indice-convertido",
                        &crate::i18n::args(&[
                            ("indice", sid.to_string().into()),
                            ("moeda", a.currency.clone().into()),
                        ]),
                    ),
                )
            };
            if series::span_years(&y1) < 0.95 {
                Err(crate::i18n::t("nucleo-score-cripto-menos-12-meses"))
            } else if !covers(&idx, &y1) {
                Err(crate::i18n::t_args(
                    "nucleo-score-etf-sem-serie",
                    &crate::i18n::args(&[("indice", sid.to_string().into())]),
                ))
            } else {
                // Diferença pelo período inteiro; tracking error com retornos semanais (os
                // diários sofrem com fechamentos em horários diferentes).
                let te =
                    series::relative(&series::sample_every(&y1, 5), &idx).map(|w| w.tracking_error);
                series::relative(&y1, &idx)
                    .map(|r| (r, te, label, converted))
                    .ok_or_else(|| crate::i18n::t("nucleo-score-etf-sem-dados"))
            }
        }
    };
    match tracking {
        Ok((r, te, sid, converted)) => {
            let period = crate::i18n::t_args(
                "nucleo-score-periodo-de-a",
                &crate::i18n::args(&[
                    ("de", br(y1[0].0).into()),
                    ("ate", br(y1[y1.len() - 1].0).into()),
                ]),
            );
            let index_src = crate::i18n::t_args(
                "nucleo-score-etf-fonte-indice",
                &crate::i18n::args(&[
                    ("fonte", src.clone().into()),
                    ("indice", sid.clone().into()),
                ]),
            );
            ms.insert(
                "etf_tracking_diff".into(),
                noted(
                    ok(r.total - r.bench_total, &period, &index_src),
                    crate::i18n::t_args(
                        "nucleo-score-etf-nota-tracking",
                        &crate::i18n::args(&[
                            ("etf", pct(r.total).into()),
                            ("indice", pct(r.bench_total).into()),
                        ]),
                    ),
                ),
            );
            ms.insert(
                "etf_tracking_error".into(),
                match te {
                    Some(t) => {
                        let v = ok(
                            t,
                            crate::i18n::t_args(
                                "nucleo-score-etf-periodo-semanal",
                                &crate::i18n::args(&[("periodo", period.clone().into())]),
                            ),
                            &index_src,
                        );
                        if converted {
                            noted(v, crate::i18n::t("nucleo-score-etf-nota-cambio"))
                        } else {
                            v
                        }
                    }
                    None => MetricValue::missing(crate::i18n::t("nucleo-score-etf-poucas-semanas")),
                },
            );
        }
        Err(e) => {
            ms.insert("etf_tracking_diff".into(), MetricValue::missing(e.clone()));
            ms.insert("etf_tracking_error".into(), MetricValue::missing(e));
        }
    }
    ms.insert(
        "etf_avg_volume".into(),
        liquidity_brl(conn, d, &a.currency, today)?,
    );
    ms.insert(
        "etf_aum".into(),
        match info.and_then(|i| dec_f(&i.nav)) {
            Some(v) => ok(
                v,
                info.and_then(|i| i.nav_date.clone())
                    .map(registry_on)
                    .unwrap_or_default(),
                crate::i18n::t("nucleo-score-fundo-fonte-registro"),
            ),
            None if a.class == AssetClass::EtfBr => {
                MetricValue::missing(crate::i18n::t("nucleo-score-etf-sem-cnpj"))
            }
            None => MetricValue::missing(crate::i18n::t("nucleo-score-etf-sem-patrimonio")),
        },
    );
    let (dom, why) = etf_domicile(cfg, &a);
    ms.insert(
        "etf_domicile".into(),
        match cfg.domicile_scores.get(dom) {
            Some(v) => noted(
                ok(
                    *v,
                    crate::i18n::t("nucleo-score-etf-periodo-classe-bolsa"),
                    crate::i18n::t("nucleo-score-etf-fonte-tabela"),
                ),
                why,
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-etf-domicilio-sem-nota")),
        },
    );
    ms.insert(
        "fund_age_years".into(),
        match info
            .and_then(|x| x.inception.as_deref())
            .and_then(|s| crate::time::parse_date(s).ok())
        {
            Some(dt) => ok(
                (today - dt).num_days() as f64 / 365.25,
                crate::i18n::t_args(
                    "nucleo-score-fundo-periodo-constituido",
                    &crate::i18n::args(&[("data", br(dt).into())]),
                ),
                crate::i18n::t("nucleo-score-fundo-fonte-registro"),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-etf-sem-constituicao")),
        },
    );
    // Composição (carteira teórica do índice da B3) e herança dos componentes.
    match &portfolio {
        Some((idx, p)) => {
            let period = crate::i18n::t_args(
                "nucleo-score-etf-periodo-carteira",
                &crate::i18n::args(&[
                    ("indice", idx.clone().into()),
                    ("data", p.date.to_string().into()),
                ]),
            );
            let src_b3 = &crate::i18n::t("nucleo-score-etf-fonte-carteira");
            ms.insert(
                "etf_top10_weight".into(),
                ok(b3_index::top_weight(p, 10), &period, src_b3),
            );
            ms.insert(
                "etf_holdings".into(),
                ok(p.holdings.len() as f64, &period, src_b3),
            );
            let mut score_of = |t: &str| -> Option<f64> {
                let r = super::assess::evaluate(conn, &SubjectRef::ticker(t), today).ok()?;
                matches!(r.status, ScoreStatus::Ok | ScoreStatus::Partial)
                    .then_some(r.score)
                    .flatten()
            };
            let (avg, coverage, n) = components_score(&p.holdings, &mut score_of);
            ms.insert(
                "etf_components_score".into(),
                match avg.filter(|_| coverage >= MIN_COMPONENT_COVERAGE) {
                    Some(v) => noted(
                        ok(
                            v,
                            &period,
                            crate::i18n::t("nucleo-score-etf-fonte-componentes"),
                        ),
                        crate::i18n::t_args(
                            "nucleo-score-etf-nota-componentes",
                            &crate::i18n::args(&[
                                ("com_score", (n as i64).into()),
                                ("total", (p.holdings.len() as i64).into()),
                                ("peso", pct(coverage).into()),
                            ]),
                        ),
                    ),
                    None => MetricValue::missing(crate::i18n::t_args(
                        "nucleo-score-etf-pouca-cobertura",
                        &crate::i18n::args(&[("peso", pct(coverage).into())]),
                    )),
                },
            );
        }
        None => {
            let why = match rule.and_then(|r| r.b3_index.as_ref()) {
                Some(i) => crate::i18n::t_args(
                    "nucleo-score-etf-sem-carteira",
                    &crate::i18n::args(&[("indice", i.clone().into())]),
                ),
                None => crate::i18n::t("nucleo-score-etf-sem-composicao"),
            };
            for k in ["etf_top10_weight", "etf_holdings", "etf_components_score"] {
                ms.insert(k.into(), MetricValue::missing(why.clone()));
            }
        }
    }
    // Risco (perfil sem composição).
    ms.insert(
        "etf_volatility".into(),
        match series::volatility(&y1).filter(|_| y1.len() >= MIN_CLOSES_12M) {
            Some(v) => ok(
                v,
                crate::i18n::t_args(
                    "nucleo-score-etf-periodo-fechamentos",
                    &crate::i18n::args(&[
                        ("n", (y1.len() as i64).into()),
                        ("data", br(y1[y1.len() - 1].0).into()),
                    ]),
                ),
                &src,
            ),
            None => MetricValue::missing(crate::i18n::t_args(
                "nucleo-score-etf-poucos-fechamentos",
                &crate::i18n::args(&[("n", (MIN_CLOSES_12M as i64).into())]),
            )),
        },
    );
    let win: Vec<Point> = match last {
        Some(l) => series::window(&px, add_months(l, -WINDOW_MONTHS), l),
        None => vec![],
    };
    ms.insert(
        "etf_max_drawdown".into(),
        match series::max_drawdown(&win).filter(|_| series::span_years(&win) >= 0.95) {
            Some(dd) => ok(
                dd.depth,
                crate::i18n::t_args(
                    "nucleo-score-periodo-de-a",
                    &crate::i18n::args(&[
                        ("de", br(win[0].0).into()),
                        ("ate", br(win[win.len() - 1].0).into()),
                    ]),
                ),
                &src,
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-menos-12-meses")),
        },
    );
    // Red flags.
    let mut flags = vec![];
    let val = |k: &str| ms.get(k).and_then(|x| x.value);
    if let (Some(x), Some(t)) = (val("etf_avg_volume"), threshold(m, "etf_low_liquidity"))
        && x < t
    {
        flags.extend(flag(
            m,
            "etf_low_liquidity",
            crate::i18n::t_args(
                "nucleo-score-flag-volume-medio",
                &crate::i18n::args(&[("valor", engine::fmt_value(Unit::Money, x).into())]),
            ),
        ));
    }
    if let (Some(x), Some(t)) = (val("etf_aum"), threshold(m, "etf_small_aum"))
        && x < t
    {
        flags.extend(flag(
            m,
            "etf_small_aum",
            crate::i18n::t_args(
                "nucleo-score-fundo-flag-patrimonio",
                &crate::i18n::args(&[("valor", engine::fmt_value(Unit::Money, x).into())]),
            ),
        ));
    }
    finish(m, &mut s, &ms, flags, last, today);
    if let Some(r) = rule {
        let what = r
            .b3_index
            .clone()
            .or_else(|| r.series.clone())
            .unwrap_or_default();
        notes.push(crate::i18n::t_args(
            "nucleo-score-etf-nota-indice",
            &crate::i18n::args(&[("indice", what.into())]),
        ));
    }
    if let Some(c) = &cnpj {
        notes.push(crate::i18n::t_args(
            "nucleo-score-etf-nota-cnpj",
            &crate::i18n::args(&[("cnpj", crate::research::format_cnpj(c).into())]),
        ));
    }
    if segment != "acoes_br" {
        notes.push(crate::i18n::t("nucleo-score-etf-nota-sem-composicao"));
    }
    notes.push(crate::i18n::t("nucleo-score-etf-nota-sem-pares"));
    s.notes.extend(notes);
    Ok(s)
}
