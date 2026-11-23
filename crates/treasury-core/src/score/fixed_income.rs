//! Renda fixa (§3, §7.2, §10.2): Tesouro Direto (soberano), bancária (CDB, LC, LCI, LCA e
//! poupança; herda a saúde do banco emissor pelo Bacen IF.data e soma a cobertura do FGC) e
//! crédito privado (debêntures, CRI e CRA; herda a saúde da companhia emissora pelo score de
//! ações). Pesos, limiares e parâmetros em `methodology/v0.1.0/classes/fixed_income.json`.
//!
//! Classe pelo tipo do título: `treasury_br` e `tesouro_*` → `fi_sovereign`; CRI, CRA e
//! debêntures → `fi_corporate`; o resto → `fi_bank`. Taxas do Tesouro (medianas de 10 anos e
//! curva do dia) vêm da coleta de cotações (`market::tesouro`), gravadas em `fundamental`.

mod calc;
mod data;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::OnceLock;

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::config::Methodology;
use super::engine::{self, Peer};
use super::metrics::{MetricValue, Metrics, br};
use super::{AssetScore, ScoreFlag, ScoreStatus};
use crate::assets::{Asset, AssetClass, FixedIncomeKind, FixedIncomeTerms, Indexer};
use crate::error::Result;
use crate::market::tesouro::{self, Quote};
use crate::research::indicators::Data;
use crate::research::to_f64;
use crate::time::{add_months, parse_date};

pub use data::{
    RatesNeed, cdi_annual, fgc_covered, matched_bank_cnpj8, monthly, rates_needed, selic_meta,
    twelve_months,
};

const CONFIG: &str = include_str!("../../../../methodology/v0.1.0/classes/fixed_income.json");

/// Parâmetros da renda fixa (regras do FGC, da B3 e dos títulos), na extensão da metodologia.
#[derive(Debug, Clone, Deserialize)]
pub struct Params {
    pub fgc: FgcParams,
    pub custody: Custody,
    pub coupons: BTreeMap<String, f64>,
    pub amortizing_months: BTreeMap<String, u32>,
    pub rate_history_years: u32,
    pub rate_history_min_points: usize,
    pub rates_refresh_days: i64,
    pub poupanca: Savings,
    pub issuer_stopwords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FgcParams {
    pub per_cpf_per_institution: String,
    pub per_cpf_global: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Custody {
    pub b3_fee: f64,
    pub selic_exempt_upto: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Savings {
    pub selic_threshold: f64,
    pub monthly_rate: f64,
    pub selic_share: f64,
    pub ir_equivalent: f64,
}

fn dec(s: &str) -> Decimal {
    s.parse().expect("parâmetro decimal válido (testado)")
}

impl Params {
    pub fn fgc_limit(&self) -> Decimal {
        dec(&self.fgc.per_cpf_per_institution)
    }
    pub fn fgc_global(&self) -> Decimal {
        dec(&self.fgc.per_cpf_global)
    }
    pub fn selic_exempt(&self) -> Decimal {
        dec(&self.custody.selic_exempt_upto)
    }
}

#[derive(Deserialize)]
struct File {
    fixed_income: Params,
}

pub fn params() -> &'static Params {
    static P: OnceLock<Params> = OnceLock::new();
    P.get_or_init(|| {
        serde_json::from_str::<File>(CONFIG)
            .expect("parâmetros da renda fixa válidos")
            .fixed_income
    })
}

/// Classe e segmento da metodologia de um título.
pub fn classify(a: &Asset) -> (&'static str, &'static str) {
    use FixedIncomeKind::*;
    let kind = a.fixed_income.as_ref().map(|t| t.kind);
    let sovereign = a.class == AssetClass::TreasuryBr
        || matches!(
            kind,
            Some(TesouroSelic | TesouroPrefixado | TesouroIpca | TesouroRenda)
        );
    if sovereign {
        let k = data::title_of(a, &[]).map(|t| t.kind).unwrap_or_default();
        let seg = match k.as_str() {
            "selic" => "selic",
            "prefixado" | "prefixado_juros" => "prefixado",
            _ => "ipca",
        };
        return ("fi_sovereign", seg);
    }
    match kind {
        Some(Cri | Cra) => ("fi_corporate", "cri_cra"),
        Some(Debenture) => ("fi_corporate", "debenture"),
        Some(DebentureIncentivada) => ("fi_corporate", "debenture_incentivada"),
        Some(Lci | Lca) => ("fi_bank", "lci_lca"),
        Some(Poupanca) => ("fi_bank", "poupanca"),
        _ => ("fi_bank", "cdb"),
    }
}

fn r1(x: f64) -> String {
    crate::ai::alerts::decimal(crate::i18n::current(), &format!("{:.1}", x * 100.0))
}

fn r2(x: f64) -> String {
    crate::ai::alerts::decimal(crate::i18n::current(), &format!("{:.2}", x * 100.0))
}

fn money(v: Decimal) -> String {
    super::engine::fmt_value(super::Unit::Money, to_f64(v))
}

fn metric(v: f64, period: Option<String>, end: Option<NaiveDate>, source: &str) -> MetricValue {
    MetricValue {
        period,
        period_end: end,
        source: Some(source.to_string()),
        ..MetricValue::ok(v)
    }
}

fn noted(mut v: MetricValue, note: impl Into<String>) -> MetricValue {
    v.note = Some(note.into());
    v
}

fn flag(m: &Methodology, key: &str, detail: String) -> Option<ScoreFlag> {
    let f = m.flags.get(key)?;
    Some(ScoreFlag {
        key: key.into(),
        label: f.label.clone(),
        severity: f.severity,
        cap: f.cap,
        penalty: f.penalty,
        detail,
    })
}

fn date_of(s: Option<&str>) -> Option<NaiveDate> {
    s.and_then(|d| parse_date(d).ok())
}

fn rate_of(t: &FixedIncomeTerms) -> Option<f64> {
    t.rate_dec().ok().map(to_f64)
}

/// Prazo para ter o dinheiro.
fn liquidity_metric(
    t: Option<&FixedIncomeTerms>,
    maturity: Option<NaiveDate>,
    today: NaiveDate,
    default_daily: bool,
) -> MetricValue {
    let liq = match t {
        Some(t) => t.liquidity,
        None if default_daily => crate::assets::Liquidity::Daily,
        None => {
            return MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-termos-liquidez"));
        }
    };
    let grace = date_of(t.and_then(|t| t.grace_until.as_deref()));
    match calc::liquidity_days(liq, grace, maturity, today) {
        Some(v) => metric(
            v,
            None,
            None,
            &crate::i18n::t("nucleo-score-rf-fonte-cadastro"),
        ),
        None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-vencimento")),
    }
}

/// Vencido: red flag.
fn matured_flag(
    m: &Methodology,
    maturity: Option<NaiveDate>,
    today: NaiveDate,
) -> Option<ScoreFlag> {
    let mt = maturity.filter(|d| *d < today)?;
    flag(
        m,
        "fi_matured",
        crate::i18n::t_args(
            "nucleo-score-rf-flag-vencido",
            &crate::i18n::args(&[("data", br(mt).into())]),
        ),
    )
}

/// Duration de um título sem cronograma conhecido: pós-fixado ≈ 0, senão o prazo.
fn bullet_duration(
    t: &FixedIncomeTerms,
    maturity: Option<NaiveDate>,
    today: NaiveDate,
) -> MetricValue {
    if matches!(t.indexer, Indexer::Cdi | Indexer::Selic) {
        return noted(
            metric(
                0.0,
                None,
                None,
                &crate::i18n::t("nucleo-score-rf-fonte-cadastro"),
            ),
            crate::i18n::t("nucleo-score-rf-nota-pos-fixado"),
        );
    }
    match maturity {
        Some(mt) => noted(
            metric(
                calc::years_between(today, mt).max(0.0),
                None,
                None,
                &crate::i18n::t("nucleo-score-rf-fonte-cadastro"),
            ),
            crate::i18n::t("nucleo-score-rf-nota-aproximacao"),
        ),
        None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-vencimento")),
    }
}

/// Curva do Tesouro no prazo: (prefixado, IPCA+) em fração a.a. e a data da curva.
struct CurveAt {
    pre: Option<f64>,
    real: Option<f64>,
    date: Option<NaiveDate>,
}

fn curve_at(curve: &[Quote], years: f64, today: NaiveDate) -> CurveAt {
    CurveAt {
        pre: calc::curve_rate(&data::curve_points(curve, "prefixado", today), years),
        real: calc::curve_rate(&data::curve_points(curve, "ipca", today), years),
        date: curve.iter().map(|q| q.date).max(),
    }
}

pub fn assess(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    let Some(id) = d.subject.asset_id.clone() else {
        let class = super::taxonomy::class_of(d.subject.class);
        return Ok(super::assess::not_supported(m, d, class, today));
    };
    let a = crate::assets::get(conn, &id)?;
    let (class, segment) = classify(&a);
    let seg = &m.class_segments[class].segments[segment];
    let profile = &m.profiles[&seg.profile];
    let s = super::assess::shell_for(
        m,
        d,
        class,
        segment,
        &seg.label,
        &seg.profile,
        Some(profile),
        today,
    );
    match class {
        "fi_sovereign" => sovereign(conn, m, &a, s, today),
        "fi_corporate" => corporate(conn, m, &a, s, today),
        _ => bank(conn, m, &a, s, today),
    }
}

/// Roda o motor e copia o resultado.
#[allow(clippy::too_many_arguments)]
fn finish(
    m: &Methodology,
    mut s: AssetScore,
    metrics: &Metrics,
    peers: &[Peer],
    id: Option<&str>,
    sector: Option<&str>,
    flags: Vec<ScoreFlag>,
    data_date: Option<NaiveDate>,
    today: NaiveDate,
    notes: Vec<String>,
) -> AssetScore {
    let profile = &m.profiles[&s.profile];
    let segment = s.segment.clone();
    let segment_label = s.segment_label.clone();
    // O setor dos pares é o porte do banco (identificador); o rótulo sai no idioma.
    let sector_label = sector.map(super::bank::group_label);
    let o = engine::run(engine::Input {
        m,
        profile,
        id,
        sector,
        sector_label: sector_label.as_deref(),
        segment: &segment,
        segment_label: &segment_label,
        metrics,
        peers,
        flags,
        data_date,
        today,
        extra_confidence: vec![],
    });
    super::assess::apply(&mut s, o, vec![], vec![], data_date);
    if sector.is_some() {
        s.sector = sector_label;
    }
    s.notes.extend(notes);
    s
}

// ---------------------------------------------------------------------------
// Tesouro Direto

fn sovereign(
    conn: &Connection,
    m: &Methodology,
    a: &Asset,
    s: AssetScore,
    today: NaiveDate,
) -> Result<AssetScore> {
    let p = params();
    let curve = data::curve(conn)?;
    if let Some(when) = data::matured(a, &curve, today) {
        // Vencido não tem saúde a avaliar (nem dado a faltar): sem score e sem aviso.
        let mut s = s;
        s.status = ScoreStatus::NotSupported;
        s.band_label = crate::i18n::t("nucleo-score-rf-titulo-vencido");
        s.band_meaning = crate::i18n::t_args(
            "nucleo-score-rf-titulo-vencido-sentido",
            &crate::i18n::args(&[("data", when.to_string().into())]),
        );
        return Ok(s);
    }
    let mut ms = Metrics::new();
    let mut notes = vec![];
    let title = data::title_of(a, &curve);
    let kind = title.as_ref().map(|t| t.kind.clone()).unwrap_or_default();
    let quote = title.as_ref().and_then(|t| t.quote.clone());
    let maturity = title.as_ref().and_then(|t| t.maturity);
    let curve_date = quote.as_ref().map(|q| q.date);
    let src = |q: &Quote| format!("Tesouro Transparente, {}", br(q.date));
    let period = |q: &Quote| {
        Some(crate::i18n::t_args(
            "nucleo-score-rf-periodo-taxa",
            &crate::i18n::args(&[("data", br(q.date).into())]),
        ))
    };
    ms.insert(
        "fi_sovereign_credit".into(),
        noted(
            metric(
                1.0,
                None,
                None,
                &crate::i18n::t("nucleo-score-rf-fonte-premissa"),
            ),
            crate::i18n::t("nucleo-score-rf-nota-tesouro-reais"),
        ),
    );
    // Taxa de hoje: a de compra quando o título está à venda, senão a de resgate.
    let rate = quote.as_ref().map(|q| {
        if q.pu_buy > Decimal::ZERO {
            to_f64(q.rate_buy)
        } else {
            to_f64(q.rate_sell)
        }
    });
    let no_rates = &crate::i18n::t("nucleo-score-rf-sem-taxas");
    let ipca = data::monthly(
        conn,
        "ipca",
        add_months(today, -12 * p.rate_history_years as i32 - 14),
        today,
    )?;
    let ipca12 = data::twelve_months(&ipca, calc::ym(today));
    let ipca_note = |x: (f64, (i32, u32))| {
        crate::i18n::t_args(
            "nucleo-score-rf-nota-ipca",
            &crate::i18n::args(&[
                ("mes", format!("{:02}/{}", x.1.1, x.1.0).into()),
                ("pct", r2(x.0).into()),
            ]),
        )
    };
    let from = add_months(today, -12 * p.rate_history_years as i32);
    if kind == "selic" {
        // Juro real da Selic agora e no histórico mensal.
        let selic = data::selic_meta(conn, today)?;
        let spread = rate.unwrap_or(0.0);
        match (selic, ipca12) {
            (Some(sm), Some(i)) => {
                let real = calc::real_rate(sm + spread, i.0);
                let hist =
                    super::normalize::sorted(data::real_selic_history(conn, &ipca, from, today)?);
                let note = crate::i18n::t_args(
                    "nucleo-score-rf-nota-selic",
                    &crate::i18n::args(&[
                        ("selic", r2(sm).into()),
                        ("agio", r2(spread).into()),
                        ("ipca", ipca_note(i).into()),
                    ]),
                );
                ms.insert(
                    "fi_real_rate".into(),
                    noted(
                        metric(
                            real,
                            Some(crate::i18n::t("nucleo-score-rf-periodo-hoje")),
                            Some(today),
                            &crate::i18n::t("nucleo-score-rf-fonte-bcb-selic-ipca"),
                        ),
                        note.clone(),
                    ),
                );
                ms.insert(
                    "fi_rate_percentile".into(),
                    if hist.len() >= 60 {
                        let pct = super::normalize::percentile_rank(&hist, real).unwrap_or(50.0);
                        noted(
                            metric(
                                pct,
                                Some(crate::i18n::t_args(
                                    "nucleo-score-rf-periodo-meses",
                                    &crate::i18n::args(&[("n", (hist.len() as i64).into())]),
                                )),
                                Some(today),
                                &crate::i18n::t("nucleo-score-rf-fonte-bcb-selic-ipca"),
                            ),
                            crate::i18n::t_args(
                                "nucleo-score-rf-nota-juro-real-selic",
                                &crate::i18n::args(&[
                                    ("pct", r2(real).into()),
                                    ("meses", (hist.len() as i64).into()),
                                    ("anos", i64::from(p.rate_history_years).into()),
                                ]),
                            ),
                        )
                    } else {
                        MetricValue::missing(crate::i18n::t(
                            "nucleo-score-rf-historico-selic-curto",
                        ))
                    },
                );
            }
            _ => {
                let why = &crate::i18n::t("nucleo-score-rf-sem-selic-ipca");
                ms.insert("fi_real_rate".into(), MetricValue::missing(why));
                ms.insert("fi_rate_percentile".into(), MetricValue::missing(why));
            }
        }
    } else {
        let family = tesouro::family_of(&kind).unwrap_or("real");
        match (&quote, rate) {
            (Some(q), Some(r)) => {
                let hist = super::normalize::sorted(data::rate_history(conn, family, from, today)?);
                let fam = crate::i18n::t(if family == "real" {
                    "nucleo-score-rf-familia-ipca"
                } else {
                    "nucleo-score-rf-familia-prefixados"
                });
                ms.insert(
                    "fi_rate_percentile".into(),
                    if hist.len() >= p.rate_history_min_points {
                        let pct = super::normalize::percentile_rank(&hist, r).unwrap_or(50.0);
                        noted(
                            metric(pct, period(q), Some(q.date), &src(q)),
                            crate::i18n::t_args(
                                "nucleo-score-rf-nota-percentil-taxa",
                                &crate::i18n::args(&[
                                    ("pct", r2(r).into()),
                                    ("medianas", (hist.len() as i64).into()),
                                    ("familia", fam.clone().into()),
                                    ("anos", i64::from(p.rate_history_years).into()),
                                    (
                                        "mediana",
                                        r2(super::normalize::median(&hist).unwrap_or(0.0)).into(),
                                    ),
                                ]),
                            ),
                        )
                    } else {
                        MetricValue::missing(crate::i18n::t_args(
                            "nucleo-score-rf-historico-curto",
                            &crate::i18n::args(&[
                                ("familia", fam.clone().into()),
                                ("dias", (p.rate_history_min_points as i64).into()),
                            ]),
                        ))
                    },
                );
                let real = if family == "real" {
                    Some(noted(
                        metric(r, period(q), Some(q.date), &src(q)),
                        format!(
                            "{} + {}%",
                            if kind.starts_with("igpm") {
                                "IGP-M"
                            } else {
                                "IPCA"
                            },
                            r2(r)
                        ),
                    ))
                } else {
                    ipca12.map(|i| {
                        noted(
                            metric(calc::real_rate(r, i.0), period(q), Some(q.date), &src(q)),
                            crate::i18n::t_args(
                                "nucleo-score-rf-nota-prefixado-real",
                                &crate::i18n::args(&[
                                    ("pct", r2(r).into()),
                                    ("ipca", ipca_note(i).into()),
                                ]),
                            ),
                        )
                    })
                };
                ms.insert(
                    "fi_real_rate".into(),
                    real.unwrap_or_else(|| {
                        MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-ipca"))
                    }),
                );
            }
            _ => {
                ms.insert("fi_rate_percentile".into(), MetricValue::missing(no_rates));
                ms.insert("fi_real_rate".into(), MetricValue::missing(no_rates));
            }
        }
    }
    // Duration pelo fluxo do título.
    let y = rate.unwrap_or(0.06);
    let dur = match (kind.as_str(), maturity) {
        ("selic", Some(_)) => Some((0.0, crate::i18n::t("nucleo-score-rf-nota-selic-duration"))),
        (k, Some(mt)) if p.coupons.contains_key(k) => {
            let c = p.coupons[k];
            calc::coupon_duration(today, mt, c, y).map(|x| {
                (
                    x,
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-cupom",
                        &crate::i18n::args(&[("pct", r1(c).into()), ("data", br(mt).into())]),
                    ),
                )
            })
        }
        (k, Some(mt)) if p.amortizing_months.contains_key(k) => {
            let n = p.amortizing_months[k];
            calc::annuity_duration(today, mt, n, y).map(|x| {
                (
                    x,
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-parcelas",
                        &crate::i18n::args(&[("n", i64::from(n).into()), ("data", br(mt).into())]),
                    ),
                )
            })
        }
        (_, Some(mt)) => Some((
            calc::years_between(today, mt).max(0.0),
            crate::i18n::t_args(
                "nucleo-score-rf-nota-sem-cupom",
                &crate::i18n::args(&[("data", br(mt).into())]),
            ),
        )),
        _ => None,
    };
    ms.insert(
        "fi_duration".into(),
        match dur {
            Some((x, note)) => noted(
                metric(
                    x,
                    None,
                    None,
                    quote
                        .as_ref()
                        .map(src)
                        .as_deref()
                        .unwrap_or(&crate::i18n::t("nucleo-score-rf-fonte-cadastro")),
                ),
                note,
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-vencimento-desconhecido")),
        },
    );
    let pu = data::prices(conn, &a.id, today - chrono::Duration::days(365), today)?;
    ms.insert(
        "fi_price_volatility".into(),
        match calc::annualized_volatility(&pu) {
            Some(v) => metric(
                v,
                Some(crate::i18n::t_args(
                    "nucleo-score-rf-periodo-pregoes",
                    &crate::i18n::args(&[("n", (pu.len() as i64).into())]),
                )),
                Some(today),
                &crate::i18n::t("nucleo-score-rf-fonte-pu"),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-poucos-precos")),
        },
    );
    ms.insert(
        "fi_liquidity_days".into(),
        match a.fixed_income.as_ref() {
            Some(t) => liquidity_metric(Some(t), maturity, today, true),
            None => noted(
                metric(
                    1.0,
                    None,
                    None,
                    &crate::i18n::t("nucleo-score-rf-fonte-regra-tesouro"),
                ),
                crate::i18n::t("nucleo-score-rf-nota-recompra"),
            ),
        },
    );
    // Custódia da B3; Tesouro Selic isento até a faixa.
    let pos: Decimal = data::position_by_holder(conn, &a.id, today)?
        .values()
        .copied()
        .sum();
    let fee = p.custody.b3_fee;
    let custody = if kind == "selic" {
        let ex = p.selic_exempt();
        let eff = if pos > Decimal::ZERO {
            fee * to_f64((pos - ex).max(Decimal::ZERO) / pos)
        } else {
            0.0
        };
        noted(
            metric(
                eff,
                None,
                None,
                &crate::i18n::t("nucleo-score-rf-fonte-regra-b3"),
            ),
            crate::i18n::t_args(
                "nucleo-score-rf-nota-custodia-selic",
                &crate::i18n::args(&[
                    ("pct", r2(fee).into()),
                    ("isento", money(ex).into()),
                    ("posicao", money(pos).into()),
                ]),
            ),
        )
    } else {
        noted(
            metric(
                fee,
                None,
                None,
                &crate::i18n::t("nucleo-score-rf-fonte-regra-b3"),
            ),
            crate::i18n::t_args(
                "nucleo-score-rf-nota-custodia",
                &crate::i18n::args(&[("pct", r2(fee).into())]),
            ),
        )
    };
    ms.insert("fi_custody_fee".into(), custody);
    ms.insert(
        "fi_exit_cost".into(),
        match &quote {
            Some(q) if q.pu_buy > Decimal::ZERO => noted(
                metric(
                    1.0 - to_f64(q.pu_sell / q.pu_buy),
                    period(q),
                    Some(q.date),
                    &src(q),
                ),
                crate::i18n::t_args(
                    "nucleo-score-rf-nota-pu",
                    &crate::i18n::args(&[
                        (
                            "compra",
                            crate::ai::alerts::decimal(
                                crate::i18n::current(),
                                &q.pu_buy.round_dp(2).to_string(),
                            )
                            .into(),
                        ),
                        (
                            "venda",
                            crate::ai::alerts::decimal(
                                crate::i18n::current(),
                                &q.pu_sell.round_dp(2).to_string(),
                            )
                            .into(),
                        ),
                    ]),
                ),
            ),
            Some(_) => MetricValue::missing(crate::i18n::t("nucleo-score-rf-fora-de-venda")),
            None => MetricValue::missing(no_rates),
        },
    );
    let flags: Vec<ScoreFlag> = matured_flag(m, maturity, today).into_iter().collect();
    if title.is_none() {
        notes.push(crate::i18n::t("nucleo-score-rf-nota-tipo-desconhecido"));
    }
    notes.push(crate::i18n::t("nucleo-score-rf-nota-soberano"));
    Ok(finish(
        m,
        s,
        &ms,
        &[],
        None,
        None,
        flags,
        curve_date.or(Some(today)),
        today,
        notes,
    ))
}

// ---------------------------------------------------------------------------
// Renda fixa bancária

fn bank(
    conn: &Connection,
    m: &Methodology,
    a: &Asset,
    s: AssetScore,
    today: NaiveDate,
) -> Result<AssetScore> {
    let p = params();
    let Some(t) = a.fixed_income.clone() else {
        return Ok(super::assess::without_data_for(
            s,
            &crate::i18n::t("nucleo-score-rf-dado-termos"),
            &crate::i18n::t("nucleo-score-rf-sem-termos"),
        ));
    };
    let mut ms = Metrics::new();
    let mut notes = vec![];
    let mut flags = vec![];
    let maturity = date_of(t.maturity_date.as_deref());
    // Saúde do emissor: indicadores prudenciais do banco (herança, §7.2).
    let issuer = data::bank_issuer(conn, a)?;
    let mut period_end = None;
    let mut sector = None;
    let mut own_id = None;
    let issuer_keys = [
        "basel_ratio",
        "cet1_ratio",
        "leverage_ratio",
        "npl90",
        "coverage",
        "roe",
        "efficiency_ratio",
    ];
    match &issuer {
        Some((inst, how)) => {
            let series = crate::research::ifdata::series(conn, &inst.cod_inst)?;
            let (pm, per) = super::bank::prudential(&series, &inst.name);
            for k in issuer_keys {
                if let Some(v) = pm.get(k) {
                    ms.insert(k.into(), v.clone());
                }
            }
            if !ms.contains_key("roe") {
                ms.insert(
                    "roe".into(),
                    MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-roe-ifdata")),
                );
            }
            period_end = per
                .as_deref()
                .and_then(|p| parse_date(&format!("{p}-01")).ok())
                .map(|d| add_months(d, 1) - chrono::Duration::days(1));
            sector = super::bank::size_group(inst.segment.as_deref(), &inst.cod_inst);
            own_id = Some(format!("ifdata:{}", inst.cod_inst));
            notes.push(match how {
                data::Match::Cnpj => crate::i18n::t_args(
                    "nucleo-score-rf-nota-emissor-banco-cnpj",
                    &crate::i18n::args(&[("emissor", inst.name.clone().into())]),
                ),
                data::Match::Name(n) => crate::i18n::t_args(
                    "nucleo-score-rf-nota-emissor-banco-nome",
                    &crate::i18n::args(&[
                        ("emissor", inst.name.clone().into()),
                        ("nome", n.clone().into()),
                    ]),
                ),
            });
            if !inst.active {
                flags.extend(flag(
                    m,
                    "fi_issuer_inactive",
                    crate::i18n::t_args(
                        "nucleo-score-rf-flag-inativa",
                        &crate::i18n::args(&[("emissor", inst.name.clone().into())]),
                    ),
                ));
                let why = crate::i18n::t_args(
                    "nucleo-score-rf-fora-de-funcionamento",
                    &crate::i18n::args(&[("emissor", inst.name.clone().into())]),
                );
                for k in issuer_keys {
                    if ms.get(k).is_some_and(|v| v.value.is_none()) {
                        ms.insert(k.into(), MetricValue::missing(why.clone()));
                    }
                }
            }
            if let Some(b) = ms.get("basel_ratio").and_then(|v| v.value)
                && let Some(f) = m.flags.get("bank_basel_low")
                && b < f.threshold.unwrap_or(0.115)
            {
                flags.extend(flag(
                    m,
                    "bank_basel_low",
                    crate::i18n::t_args(
                        "nucleo-score-banco-flag-basileia",
                        &crate::i18n::args(&[("valor", r1(b).into())]),
                    ),
                ));
            }
        }
        None => {
            let why = match t.issuer.as_deref().filter(|x| !x.trim().is_empty()) {
                Some(n) => crate::i18n::t_args(
                    "nucleo-score-rf-banco-nao-identificado",
                    &crate::i18n::args(&[("nome", n.to_string().into())]),
                ),
                None => crate::i18n::t("nucleo-score-rf-banco-nao-informado"),
            };
            for k in issuer_keys {
                ms.insert(k.into(), MetricValue::missing(why.clone()));
            }
        }
    }
    // FGC.
    ms.insert(
        "fgc_coverage".into(),
        match data::fgc_covered(t.kind) {
            None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-fgc-tipo-outro")),
            Some(false) => noted(
                metric(
                    0.0,
                    None,
                    None,
                    &crate::i18n::t("nucleo-score-rf-fonte-regra-fgc"),
                ),
                crate::i18n::t("nucleo-score-rf-sem-fgc"),
            ),
            Some(true) => {
                let key = data::institution_key(conn, a)?;
                let pos = data::position_by_holder(conn, &a.id, today)?;
                let ex = data::exposure(conn, &key, today)?;
                let o = calc::fgc_coverage(
                    &pos,
                    &ex.at_institution,
                    &ex.covered_total,
                    p.fgc_limit(),
                    p.fgc_global(),
                );
                if o.excess > Decimal::ZERO {
                    flags.extend(flag(
                        m,
                        "fi_fgc_over_limit",
                        crate::i18n::t_args(
                            "nucleo-score-rf-flag-fgc",
                            &crate::i18n::args(&[
                                ("excesso", money(o.excess).into()),
                                ("limite", money(p.fgc_limit()).into()),
                            ]),
                        ),
                    ));
                }
                let names = data::holder_names(conn)?;
                let who: Vec<String> = pos
                    .keys()
                    .map(|h| {
                        names.get(h).cloned().unwrap_or_else(|| {
                            crate::i18n::t("nucleo-score-rf-titular-desconhecido")
                        })
                    })
                    .collect();
                let attribution =
                    crate::i18n::t(if pos.keys().any(|h| h == data::UNKNOWN_HOLDER) {
                        "nucleo-score-rf-fgc-sem-titulares"
                    } else {
                        "nucleo-score-rf-fgc-por-titular"
                    });
                let note = if pos.is_empty() {
                    crate::i18n::t("nucleo-score-rf-sem-posicao")
                } else {
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-fgc",
                        &crate::i18n::args(&[
                            ("titulos", (ex.titles as i64).into()),
                            ("exposicao", money(o.max_exposure).into()),
                            ("titulares", who.join(", ").into()),
                            ("atribuicao", attribution.into()),
                        ]),
                    )
                };
                noted(
                    metric(
                        o.coverage,
                        Some(crate::i18n::t("nucleo-score-rf-periodo-hoje")),
                        Some(today),
                        &crate::i18n::t("nucleo-score-rf-fonte-fgc"),
                    ),
                    note,
                )
            }
        },
    );
    // Taxa vs CDI.
    ms.insert(
        "fi_rate_vs_cdi".into(),
        rate_vs_cdi(conn, &t, maturity, today)?,
    );
    ms.insert(
        "fi_liquidity_days".into(),
        if t.kind == FixedIncomeKind::Poupanca {
            noted(
                metric(
                    1.0,
                    None,
                    None,
                    &crate::i18n::t("nucleo-score-rf-fonte-regra-poupanca"),
                ),
                crate::i18n::t("nucleo-score-rf-nota-liquidez-diaria"),
            )
        } else {
            liquidity_metric(Some(&t), maturity, today, false)
        },
    );
    flags.extend(matured_flag(m, maturity, today));
    // Pares: os bancos do IF.data no mesmo porte prudencial.
    let segment = s.segment.clone();
    let peers: Vec<Peer> = if issuer.is_some() {
        super::bank::peers(conn, m, today)?
            .into_iter()
            .map(|mut x| {
                x.segment = segment.clone();
                x
            })
            .collect()
    } else {
        vec![]
    };
    if issuer.is_some() {
        notes.push(crate::i18n::t("nucleo-score-rf-nota-heranca-banco"));
    }
    Ok(finish(
        m,
        s,
        &ms,
        &peers,
        own_id.as_deref(),
        sector.as_deref(),
        flags,
        period_end,
        today,
        notes,
    ))
}

/// Taxa contratada em % do CDI (equivalente tributado).
fn rate_vs_cdi(
    conn: &Connection,
    t: &FixedIncomeTerms,
    maturity: Option<NaiveDate>,
    today: NaiveDate,
) -> Result<MetricValue> {
    let p = params();
    let cdi = data::cdi_annual(conn, today)?;
    let issue = date_of(t.issue_date.as_deref());
    let days = match (issue, maturity) {
        (Some(i), Some(mt)) => (mt - i).num_days(),
        (None, Some(mt)) => (mt - today).num_days(),
        _ => 721,
    };
    let ir = to_f64(crate::finance::fixed_income::ir_rate(days));
    let exempt = t.ir_exempt || t.kind.default_ir_exempt();
    let no_cdi = &crate::i18n::t("nucleo-score-rf-sem-cdi");
    if t.kind == FixedIncomeKind::Poupanca {
        let (Some((c, _)), Some(sm)) = (cdi, data::selic_meta(conn, today)?) else {
            return Ok(MetricValue::missing(no_cdi));
        };
        let tr = data::monthly(conn, "tr", add_months(today, -14), today)?;
        let tr12 = data::twelve_months(&tr, calc::ym(today))
            .map(|x| x.0)
            .unwrap_or(0.0);
        let sp = &p.poupanca;
        let y = calc::savings_yield(
            sm,
            tr12,
            sp.selic_threshold,
            sp.monthly_rate,
            sp.selic_share,
        );
        let Some(pct) = calc::pct_of(y, c) else {
            return Ok(MetricValue::missing(no_cdi));
        };
        return Ok(noted(
            metric(
                calc::gross_up(pct, sp.ir_equivalent),
                Some(crate::i18n::t("nucleo-score-rf-periodo-hoje")),
                Some(today),
                &crate::i18n::t("nucleo-score-rf-fonte-bcb-poupanca"),
            ),
            crate::i18n::t_args(
                "nucleo-score-rf-nota-poupanca",
                &crate::i18n::args(&[
                    ("rende", r2(y).into()),
                    ("selic", r2(sm).into()),
                    ("tr", r2(tr12).into()),
                    ("ir", r1(sp.ir_equivalent).into()),
                ]),
            ),
        ));
    }
    let Some(rate) = rate_of(t) else {
        return Ok(MetricValue::missing(crate::i18n::t(
            "nucleo-score-rf-taxa-invalida",
        )));
    };
    let years = maturity
        .map(|mt| calc::years_between(today, mt).max(0.0))
        .unwrap_or(1.0);
    let curve = data::curve(conn)?;
    let c = curve_at(&curve, years, today);
    let ipca12 = data::twelve_months(
        &data::monthly(conn, "ipca", add_months(today, -14), today)?,
        calc::ym(today),
    )
    .map(|x| x.0);
    let (pct, how, src) = match t.indexer {
        Indexer::Cdi | Indexer::Selic => (
            Some(rate / 100.0),
            crate::i18n::t_args(
                "nucleo-score-rf-pct-do-indice",
                &crate::i18n::args(&[
                    ("pct", r1(rate / 100.0).into()),
                    (
                        "indice",
                        (if t.indexer == Indexer::Cdi {
                            "CDI"
                        } else {
                            "Selic"
                        })
                        .into(),
                    ),
                ]),
            ),
            crate::i18n::t("nucleo-score-rf-fonte-cadastro"),
        ),
        Indexer::Prefixado => {
            let r = rate / 100.0;
            match (c.pre, cdi) {
                (Some(pre), _) => (
                    calc::pct_of(r, pre),
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-prefixado-curva",
                        &crate::i18n::args(&[("pct", r2(r).into()), ("curva", r2(pre).into())]),
                    ),
                    format!(
                        "Tesouro Transparente, {}",
                        c.date.map(br).unwrap_or_default()
                    ),
                ),
                (None, Some((cd, _))) => (
                    calc::pct_of(r, cd),
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-prefixado-cdi",
                        &crate::i18n::args(&[("pct", r2(r).into()), ("cdi", r2(cd).into())]),
                    ),
                    crate::i18n::t("nucleo-score-rf-fonte-bcb-cdi"),
                ),
                _ => {
                    return Ok(MetricValue::missing(crate::i18n::t(
                        "nucleo-score-rf-sem-curva-cdi",
                    )));
                }
            }
        }
        Indexer::Ipca | Indexer::Igpm => {
            let r = rate / 100.0;
            match (c.pre, c.real, cdi, ipca12) {
                (Some(pre), Some(real), _, _) => (
                    calc::pct_of(calc::real_to_nominal(r, pre, real), pre),
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-ipca-curva",
                        &crate::i18n::args(&[
                            ("pct", r2(r).into()),
                            ("real", r2(real).into()),
                            ("pre", r2(pre).into()),
                        ]),
                    ),
                    format!(
                        "Tesouro Transparente, {}",
                        c.date.map(br).unwrap_or_default()
                    ),
                ),
                (_, _, Some((cd, _)), Some(i)) => (
                    calc::pct_of((1.0 + r) * (1.0 + i) - 1.0, cd),
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-ipca-cdi",
                        &crate::i18n::args(&[
                            ("pct", r2(r).into()),
                            ("ipca", r2(i).into()),
                            ("cdi", r2(cd).into()),
                        ]),
                    ),
                    crate::i18n::t("nucleo-score-rf-fonte-bcb-cdi-ipca"),
                ),
                _ => {
                    return Ok(MetricValue::missing(crate::i18n::t(
                        "nucleo-score-rf-sem-curva-cdi-ipca",
                    )));
                }
            }
        }
    };
    let Some(pct) = pct else {
        return Ok(MetricValue::missing(no_cdi));
    };
    let (v, tax) = if exempt {
        (
            calc::gross_up(pct, ir),
            crate::i18n::t_args(
                "nucleo-score-rf-nota-isento",
                &crate::i18n::args(&[
                    ("pct", r1(calc::gross_up(pct, ir)).into()),
                    ("ir", r1(ir).into()),
                    ("dias", days.into()),
                ]),
            ),
        )
    } else {
        (pct, String::new())
    };
    Ok(noted(
        metric(
            v,
            Some(crate::i18n::t("nucleo-score-rf-periodo-hoje")),
            Some(today),
            &src,
        ),
        format!("{how}{tax}"),
    ))
}

// ---------------------------------------------------------------------------
// Crédito privado

fn corporate(
    conn: &Connection,
    m: &Methodology,
    a: &Asset,
    s: AssetScore,
    today: NaiveDate,
) -> Result<AssetScore> {
    let Some(t) = a.fixed_income.clone() else {
        return Ok(super::assess::without_data_for(
            s,
            &crate::i18n::t("nucleo-score-rf-dado-termos"),
            &crate::i18n::t("nucleo-score-rf-sem-termos"),
        ));
    };
    let mut ms = Metrics::new();
    let mut notes = vec![];
    let mut flags = vec![];
    let maturity = date_of(t.maturity_date.as_deref());
    let mut data_date = None;
    // Saúde do emissor: sub-score Saúde da ação da companhia (herança, §7.2).
    let issuer = data::company_issuer(conn, a)?;
    ms.insert(
        "fi_issuer_health".into(),
        match &issuer {
            Some((iss, how)) => {
                let eq = issuer_equity(conn, m, iss, today)?;
                data_date = eq.data_date.as_deref().and_then(|d| parse_date(d).ok());
                let via = match how {
                    data::Match::Cnpj => crate::i18n::t("nucleo-score-rf-via-cnpj"),
                    data::Match::Name(n) => crate::i18n::t_args(
                        "nucleo-score-rf-via-nome",
                        &crate::i18n::args(&[("nome", n.clone().into())]),
                    ),
                };
                notes.push(crate::i18n::t_args(
                    "nucleo-score-rf-nota-emissor-cvm",
                    &crate::i18n::args(&[
                        ("emissor", iss.name.clone().into()),
                        ("como", via.into()),
                    ]),
                ));
                match (eq.status, eq.health) {
                    (ScoreStatus::Ok | ScoreStatus::Partial, Some(h)) => noted(
                        metric(
                            h,
                            eq.data_date.clone().map(|d| {
                                crate::i18n::t_args(
                                    "nucleo-score-rf-periodo-dados-ate",
                                    &crate::i18n::args(&[("data", d.into())]),
                                )
                            }),
                            data_date,
                            &crate::i18n::t("nucleo-score-rf-fonte-score-acoes"),
                        ),
                        crate::i18n::t_args(
                            "nucleo-score-rf-nota-score-emissor",
                            &crate::i18n::args(&[
                                (
                                    "emissor",
                                    eq.ticker.clone().unwrap_or(iss.name.clone()).into(),
                                ),
                                (
                                    "score",
                                    eq.score
                                        .map(|x| format!("{x:.0}"))
                                        .unwrap_or("—".into())
                                        .into(),
                                ),
                                ("faixa", eq.band_label.clone().into()),
                                ("saude", format!("{h:.0}").into()),
                                (
                                    "parcial",
                                    (if eq.status == ScoreStatus::Partial {
                                        "sim"
                                    } else {
                                        "nao"
                                    })
                                    .into(),
                                ),
                            ]),
                        ),
                    ),
                    _ => MetricValue::missing(crate::i18n::t_args(
                        "nucleo-score-rf-score-emissor-falhou",
                        &crate::i18n::args(&[
                            ("emissor", iss.name.clone().into()),
                            (
                                "faltam",
                                eq.missing
                                    .iter()
                                    .filter(|x| x.critical)
                                    .map(|x| x.label.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                                    .into(),
                            ),
                        ]),
                    )),
                }
            }
            None => {
                MetricValue::missing(match t.issuer.as_deref().filter(|x| !x.trim().is_empty()) {
                    Some(n) => crate::i18n::t_args(
                        "nucleo-score-rf-companhia-nao-identificada",
                        &crate::i18n::args(&[("nome", n.to_string().into())]),
                    ),
                    None => crate::i18n::t("nucleo-score-rf-companhia-nao-informada"),
                })
            }
        },
    );
    // Spread sobre o Tesouro.
    let years = maturity
        .map(|mt| calc::years_between(today, mt).max(0.0))
        .unwrap_or(1.0);
    let curve = data::curve(conn)?;
    let c = curve_at(&curve, years, today);
    let curve_src = format!(
        "Tesouro Transparente, {}",
        c.date.map(br).unwrap_or_default()
    );
    ms.insert(
        "fi_credit_spread".into(),
        match (rate_of(&t), t.indexer) {
            (None, _) => MetricValue::missing(crate::i18n::t("nucleo-score-rf-taxa-invalida")),
            (Some(r), Indexer::Ipca | Indexer::Igpm) => match c.real {
                Some(real) => noted(
                    metric(
                        r / 100.0 - real,
                        Some(crate::i18n::t("nucleo-score-rf-periodo-hoje")),
                        c.date,
                        &curve_src,
                    ),
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-spread-ipca",
                        &crate::i18n::args(&[
                            ("pct", r2(r / 100.0).into()),
                            ("curva", r2(real).into()),
                        ]),
                    ),
                ),
                None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-curva-ipca")),
            },
            (Some(r), Indexer::Prefixado) => match c.pre {
                Some(pre) => noted(
                    metric(
                        r / 100.0 - pre,
                        Some(crate::i18n::t("nucleo-score-rf-periodo-hoje")),
                        c.date,
                        &curve_src,
                    ),
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-prefixado-curva",
                        &crate::i18n::args(&[
                            ("pct", r2(r / 100.0).into()),
                            ("curva", r2(pre).into()),
                        ]),
                    ),
                ),
                None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-curva-prefixado")),
            },
            (Some(r), Indexer::Cdi | Indexer::Selic) => match data::cdi_annual(conn, today)? {
                Some((cd, d)) => noted(
                    metric(
                        (r / 100.0 - 1.0) * cd,
                        Some(crate::i18n::t("nucleo-score-rf-periodo-hoje")),
                        Some(d),
                        &crate::i18n::t("nucleo-score-rf-fonte-bcb-cdi"),
                    ),
                    crate::i18n::t_args(
                        "nucleo-score-rf-nota-pct-cdi",
                        &crate::i18n::args(&[
                            ("pct", r1(r / 100.0).into()),
                            ("cdi", r2(cd).into()),
                        ]),
                    ),
                ),
                None => MetricValue::missing(crate::i18n::t("nucleo-score-rf-sem-cdi")),
            },
        },
    );
    ms.insert("fi_duration".into(), bullet_duration(&t, maturity, today));
    ms.insert(
        "fi_liquidity_days".into(),
        liquidity_metric(Some(&t), maturity, today, false),
    );
    let exempt = t.ir_exempt || t.kind.default_ir_exempt();
    ms.insert(
        "fi_tax_exempt".into(),
        noted(
            metric(
                if exempt { 1.0 } else { 0.0 },
                None,
                None,
                &crate::i18n::t("nucleo-score-rf-fonte-cadastro"),
            ),
            crate::i18n::t(if exempt {
                "nucleo-score-rf-isento"
            } else {
                "nucleo-score-rf-tributado"
            }),
        ),
    );
    flags.extend(matured_flag(m, maturity, today));
    notes.push(crate::i18n::t("nucleo-score-rf-nota-credito-privado"));
    Ok(finish(
        m,
        s,
        &ms,
        &[],
        None,
        None,
        flags,
        data_date.or(Some(today)),
        today,
        notes,
    ))
}

/// Score de ações do emissor (sem gravar snapshot nem feedback: é só a herança).
fn issuer_equity(
    conn: &Connection,
    m: &Methodology,
    iss: &crate::research::Issuer,
    today: NaiveDate,
) -> Result<AssetScore> {
    use crate::research::data as rd;
    use crate::research::store;
    let ticker = store::securities_of(conn, &iss.id)?
        .into_iter()
        .filter(|x| x.active && x.class == "stock_br")
        .map(|x| x.ticker)
        .next();
    let d = match ticker {
        Some(t) => rd::load(
            conn,
            &rd::SubjectRef::ticker(&t),
            today,
            &mut rd::Cache::default(),
        )?,
        None => Data {
            subject: rd::subject_of(None, None, Some(iss), None),
            facts: store::load_facts(conn, &iss.id)?,
            price: None,
            per_unit: Some(Decimal::ONE),
            siblings: vec![],
            dividends: vec![],
            dividends_source: None,
            share_events: vec![],
            liquidity: None,
            today,
        },
    };
    super::assess::assess_equity(conn, m, &d, today)
}
