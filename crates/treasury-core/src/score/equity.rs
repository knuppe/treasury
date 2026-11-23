//! Ações BR: universo de pares (todas as companhias com demonstrações na CVM, métricas do
//! dia em `score_universe`), métricas do ativo, red flags e conferência com o Fundamentus.

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};

use super::config::{FlagDef, Methodology};
use super::engine::Peer;
use super::metrics::{self, EquityInput, EquityOutput, Metrics, Statements};
use super::taxonomy;
use super::{ReconItem, ScoreFlag};
use crate::assets::AssetClass;
use crate::error::Result;
use crate::research::screen::UniverseRow;
use crate::research::{Fact, IssuerKind, Layout, Security, store};
use crate::time::{add_months, parse_date};

/// Datas de entrega das demonstrações de um emissor: (tipo, referência, entrega).
pub type Filings = Vec<(String, NaiveDate, NaiveDate)>;

fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// Entregas de DFP/ITR (tabela da coleta da CVM; vazia em cofres sem a coleta nova).
pub fn filings(conn: &Connection, issuer_id: Option<&str>) -> Result<HashMap<String, Filings>> {
    let mut out: HashMap<String, Filings> = HashMap::new();
    if !table_exists(conn, "research_filing")? {
        return Ok(out);
    }
    let mut stmt = conn.prepare(
        "SELECT issuer_id, kind, reference, filed_at FROM research_filing
          WHERE kind IN ('dfp', 'itr') AND (?1 IS NULL OR issuer_id = ?1)",
    )?;
    let rows = stmt.query_map(params![issuer_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    for row in rows {
        let (id, kind, reference, filed) = row?;
        if let (Ok(r), Some(Ok(f))) = (parse_date(&reference), filed.as_deref().map(parse_date)) {
            out.entry(id).or_default().push((kind, r, f));
        }
    }
    Ok(out)
}

/// Auditores do emissor: (nome, início, fim), mais antigo primeiro.
pub fn auditors(
    conn: &Connection,
    issuer_id: &str,
) -> Result<Vec<(String, NaiveDate, Option<NaiveDate>)>> {
    if !table_exists(conn, "research_auditor")? {
        return Ok(vec![]);
    }
    let mut stmt = conn.prepare(
        "SELECT auditor, start, \"end\" FROM research_auditor WHERE issuer_id = ?1 ORDER BY start",
    )?;
    let rows: Vec<(String, String, Option<String>)> = stmt
        .query_map([issuer_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(a, s, e)| {
            Some((a, parse_date(&s).ok()?, e.and_then(|e| parse_date(&e).ok())))
        })
        .collect())
}

/// Pontuação do segmento de listagem (Novo Mercado 100…).
pub fn listing_score(m: &Methodology, listing: Option<&str>) -> Option<(f64, String)> {
    let l = listing?.trim();
    let n = crate::research::norm(l);
    m.equity_br
        .listing_scores
        .get(&n)
        .map(|s| (*s, l.to_string()))
}

/// Fator do IPCA entre duas datas (memorizado: os pares repetem os mesmos exercícios).
pub struct Inflation<'c> {
    conn: &'c Connection,
    deflator: crate::indices::Deflator,
    memo: std::cell::RefCell<HashMap<(NaiveDate, NaiveDate), Option<f64>>>,
}

impl<'c> Inflation<'c> {
    /// IPCA (empresas brasileiras).
    pub fn new(conn: &'c Connection) -> Self {
        Self::with(conn, crate::indices::Deflator::Ipca)
    }
    /// Outro deflator (CPI dos EUA para empresas americanas).
    pub fn with(conn: &'c Connection, deflator: crate::indices::Deflator) -> Self {
        Self {
            conn,
            deflator,
            memo: Default::default(),
        }
    }
    pub fn factor(&self, from: NaiveDate, to: NaiveDate) -> Option<f64> {
        if let Some(v) = self.memo.borrow().get(&(from, to)) {
            return *v;
        }
        let v = crate::indices::deflator_factor(self.conn, self.deflator, from, to)
            .ok()
            .flatten()
            .map(|f| crate::research::to_f64(f.factor));
        self.memo.borrow_mut().insert((from, to), v);
        v
    }
}

/// Métricas de um emissor (ativo ou par).
#[allow(clippy::too_many_arguments)]
pub fn compute(
    m: &Methodology,
    facts: &[Fact],
    today: NaiveDate,
    market_cap: Option<(f64, String)>,
    dividend_yield: Option<(f64, String)>,
    listing: Option<(f64, String)>,
    filings: &Filings,
    prices: &[(NaiveDate, f64)],
    inflation: &Inflation<'_>,
) -> EquityOutput {
    let st = Statements::new(facts, today);
    let f = |a: NaiveDate, b: NaiveDate| inflation.factor(a, b);
    metrics::equity_metrics(&EquityInput {
        st: &st,
        today,
        market_cap,
        dividend_yield,
        listing,
        filings,
        prices,
        inflation: &f,
        tax_rate: m.normalization.tax_rate_br,
        history_years: m.normalization.history_years as usize,
    })
}

/// Pares: um por emissor com ações negociadas (o papel mais líquido dá o valor de mercado).
pub fn universe(
    conn: &Connection,
    m: &Methodology,
    rows: &[UniverseRow],
    today: NaiveDate,
) -> Result<Vec<Peer>> {
    let mut best: BTreeMap<String, &UniverseRow> = BTreeMap::new();
    for r in rows.iter().filter(|r| r.class == AssetClass::StockBr) {
        let Some(id) = &r.issuer_id else { continue };
        let liq = |x: &UniverseRow| x.metrics.get("liquidity").copied().unwrap_or(0.0);
        match best.get(id) {
            Some(b) if liq(b) >= liq(r) => {}
            _ => {
                best.insert(id.clone(), r);
            }
        }
    }
    let issuers = store::all_issuers(conn)?;
    let securities: HashMap<String, Security> = store::all_securities(conn)?
        .into_iter()
        .map(|s| (s.ticker.clone(), s))
        .collect();
    let all_filings = filings(conn, None)?;
    let overrides = taxonomy::overrides(conn)?;
    let inflation = Inflation::new(conn);
    let mut all_facts = store::load_all_facts(conn)?;
    // Fechamentos do COTAHIST (13 meses) para volatilidade e momentum dos pares.
    let mut series = store::quote_series(conn, None, today - chrono::Duration::days(400))?;
    let mut out = vec![];
    for (id, row) in best {
        let Some(issuer) = issuers.get(&id).filter(|i| i.kind == IssuerKind::CompanyBr) else {
            continue;
        };
        let Some(facts) = all_facts.remove(&id).filter(|f| !f.is_empty()) else {
            continue;
        };
        let sec = securities.get(&row.ticker);
        let mc = row
            .metrics
            .get("market_cap")
            .map(|v| (*v, "B3 COTAHIST".to_string()));
        let dy = row.metrics.get("dy").map(|v| (*v, "proventos".to_string()));
        let listing = listing_score(m, sec.and_then(|s| s.listing_segment.as_deref()));
        let empty = vec![];
        let fl = all_filings.get(&id).unwrap_or(&empty);
        let prices = series.remove(&row.ticker).unwrap_or_default();
        let o = compute(m, &facts, today, mc, dy, listing, fl, &prices, &inflation);
        let segment = overrides.get(&row.ticker).cloned().unwrap_or_else(|| {
            taxonomy::equity_segment(m, issuer.sector.as_deref(), Some(issuer.layout))
        });
        out.push(Peer {
            id,
            ticker: Some(row.ticker.clone()),
            name: issuer.name.clone(),
            sector: issuer.sector.as_deref().map(taxonomy::normalize_sector),
            segment,
            metrics: o.metrics,
        });
    }
    Ok(out)
}

fn flag(m: &Methodology, key: &str, detail: String) -> Option<ScoreFlag> {
    let d: &FlagDef = m.flags.get(key)?;
    Some(ScoreFlag {
        key: key.into(),
        label: m.flag_label(key),
        severity: d.severity,
        cap: d.cap,
        penalty: d.penalty,
        detail,
    })
}

fn thr(m: &Methodology, key: &str, default: f64) -> f64 {
    m.flags
        .get(key)
        .and_then(|f| f.threshold)
        .unwrap_or(default)
}

fn value(ms: &Metrics, key: &str) -> Option<f64> {
    ms.get(key).and_then(|v| v.value)
}

/// Red flags quantitativas automáticas (§8.1) de uma ação.
pub fn flags(
    m: &Methodology,
    out: &EquityOutput,
    layout: Layout,
    filings: &Filings,
    auditors: &[(String, NaiveDate, Option<NaiveDate>)],
    liquidity: Option<f64>,
    today: NaiveDate,
) -> Vec<ScoreFlag> {
    let mut v = vec![];
    let fmt = |x: f64| format!("{x:.2}").replace('.', ",");
    if !layout.is_financial() {
        if let Some(z) = value(&out.metrics, "altman_z2")
            && z < thr(m, "altman_distress", 1.1)
        {
            v.extend(flag(m, "altman_distress", format!("Z'' = {}", fmt(z))));
        }
        if let Some(b) = out
            .frameworks
            .iter()
            .find(|f| f.key == "beneish")
            .and_then(|f| f.value)
            && b > thr(m, "beneish_manipulation", -1.78)
        {
            v.extend(flag(
                m,
                "beneish_manipulation",
                crate::i18n::t_args(
                    "nucleo-score-flag-beneish",
                    &crate::i18n::args(&[("valor", fmt(b).into())]),
                ),
            ));
        }
        if let Some(p) = value(&out.metrics, "piotroski")
            && p <= thr(m, "piotroski_low", 2.0)
        {
            v.extend(flag(m, "piotroski_low", format!("F = {p:.0}")));
        }
        let lim = thr(m, "interest_coverage_low", 1.5);
        if out.coverage_last_quarters.len() >= 2
            && out.coverage_last_quarters.iter().all(|c| *c < lim)
        {
            v.extend(flag(
                m,
                "interest_coverage_low",
                crate::i18n::t_args(
                    "nucleo-score-flag-cobertura",
                    &crate::i18n::args(&[
                        ("ultimo", fmt(out.coverage_last_quarters[0]).into()),
                        ("anterior", fmt(out.coverage_last_quarters[1]).into()),
                    ]),
                ),
            ));
        }
        let n = out.fcf_years.len();
        if n >= 3 && out.fcf_years[n - 3..].iter().all(|x| *x < 0.0) {
            v.extend(flag(
                m,
                "fcf_negative_3y",
                crate::i18n::t("nucleo-score-flag-fcf-negativo"),
            ));
        }
        let d = &out.dividends_over_fcf_years;
        if d.len() >= 2 && d[d.len() - 2..].iter().all(|x| *x) {
            v.extend(flag(
                m,
                "dividends_above_fcf",
                crate::i18n::t("nucleo-score-flag-dividendos-acima"),
            ));
        }
        if let Some(s) = out.dsri
            && s > thr(m, "dsri_high", 1.4)
        {
            v.extend(flag(m, "dsri_high", format!("DSRI = {}", fmt(s))));
        }
    }
    if let Some(x) = value(&out.metrics, "dilution")
        && x > thr(m, "dilution_high", 0.05)
    {
        v.extend(flag(
            m,
            "dilution_high",
            crate::i18n::t_args(
                "nucleo-score-flag-diluicao",
                &crate::i18n::args(&[(
                    "pct",
                    crate::ai::alerts::decimal(
                        crate::i18n::current(),
                        &format!("{:.1}", x * 100.0),
                    )
                    .into(),
                )]),
            ),
        ));
    }
    if out.negative_equity {
        v.extend(flag(
            m,
            "negative_equity",
            crate::i18n::t("nucleo-score-flag-patrimonio-negativo"),
        ));
    }
    let late: Vec<String> = filings
        .iter()
        .filter(|(k, r, f)| *f >= add_months(today, -12) && *f > metrics::deadline(k, *r))
        .map(|(k, r, f)| {
            crate::i18n::t_args(
                "nucleo-score-flag-entrega-atrasada",
                &crate::i18n::args(&[
                    ("documento", k.to_uppercase().into()),
                    ("referencia", metrics::br(*r).into()),
                    ("entrega", metrics::br(*f).into()),
                ]),
            )
        })
        .collect();
    if !late.is_empty() {
        v.extend(flag(m, "late_filing", late.join("; ")));
    }
    if let Some(min_years) = m
        .flags
        .get("auditor_change")
        .and_then(|f| f.min_tenure_years)
        && auditors.len() >= 2
    {
        let (cur, prev) = (&auditors[auditors.len() - 1], &auditors[auditors.len() - 2]);
        let tenure = (prev.2.unwrap_or(cur.1) - prev.1).num_days() as f64 / 365.25;
        if cur.1 >= add_months(today, -12)
            && tenure < min_years
            && crate::research::norm(&cur.0) != crate::research::norm(&prev.0)
        {
            v.extend(flag(
                m,
                "auditor_change",
                crate::i18n::t_args(
                    "nucleo-score-flag-auditor",
                    &crate::i18n::args(&[
                        ("antiga", prev.0.clone().into()),
                        ("nova", cur.0.clone().into()),
                        (
                            "anos",
                            crate::ai::alerts::decimal(
                                crate::i18n::current(),
                                &format!("{tenure:.1}"),
                            )
                            .into(),
                        ),
                    ]),
                ),
            ));
        }
    }
    if let Some(l) = liquidity
        && l < thr(m, "low_liquidity", 1_000_000.0)
    {
        v.extend(flag(
            m,
            "low_liquidity",
            crate::i18n::t_args(
                "nucleo-score-flag-volume-medio",
                &crate::i18n::args(&[(
                    "valor",
                    super::engine::fmt_value(super::Unit::Money, l).into(),
                )]),
            ),
        ));
    }
    v
}

/// Conferência dos proventos com a B3 (oficial), para ações e FIIs cadastrados com agenda
/// (Yahoo, Fundamentus): um item com a soma de 12 meses e um por provento divergente (até 6),
/// e uma nota. Com divergência material os indicadores (DY, Bazin) já usam a B3; a conferência
/// não reduz a confiança, porque o score usa a fonte oficial.
pub fn reconcile_dividends(
    conn: &Connection,
    d: &crate::research::indicators::Data,
    today: NaiveDate,
) -> Result<(Vec<ReconItem>, Option<String>)> {
    use crate::research::dividends::{self, DividendCheckStatus as St};
    let (Some(id), Some(t)) = (&d.subject.asset_id, &d.subject.ticker) else {
        return Ok((vec![], None));
    };
    let r = crate::research::data::SubjectRef::asset(id);
    let Ok(c) = dividends::check_subject(conn, &r, today) else {
        return Ok((vec![], None));
    };
    if !c.b3_available || c.other_sources.is_empty() {
        return Ok((vec![], None));
    }
    let f = |s: &Option<String>| {
        s.as_deref()
            .and_then(|x| x.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let agenda = c.other_sources.join(", ");
    let mut out = vec![];
    if let Some(diff) = c.diff_12m.filter(|x| *x > dividends::TOLERANCE) {
        out.push(ReconItem {
            metric: "dividends_12m".into(),
            label: crate::i18n::t_args(
                "nucleo-score-recon-proventos-12m",
                &crate::i18n::args(&[("agenda", agenda.clone().into())]),
            ),
            ours: f(&c.other_12m),
            theirs: f(&c.b3_12m),
            diff,
            source: crate::i18n::t("nucleo-score-fonte-b3-oficial"),
        });
    }
    for i in c.items.iter().filter(|i| i.status != St::Match).take(6) {
        let date = i
            .b3_date
            .clone()
            .or(i.other_date.clone())
            .unwrap_or_default();
        let src = i.other_source.clone().unwrap_or_default();
        let what = match i.status {
            St::ValueDiff => crate::i18n::t("nucleo-score-recon-valor-diferente"),
            St::OnlyB3 => crate::i18n::t("nucleo-score-recon-so-b3"),
            St::OnlyOther => crate::i18n::t("nucleo-score-recon-fora-b3"),
            St::Match => String::new(),
        };
        out.push(ReconItem {
            metric: format!("dividend:{date}:{src}:{}", i.family),
            label: crate::i18n::t_args(
                "nucleo-score-recon-provento",
                &crate::i18n::args(&[
                    (
                        "tipo",
                        (if i.family == "amortização" {
                            "amortizacao"
                        } else {
                            "provento"
                        })
                        .into(),
                    ),
                    (
                        "data",
                        parse_date(&date)
                            .map(metrics::br)
                            .unwrap_or(date.clone())
                            .into(),
                    ),
                    ("fonte", src.clone().into()),
                    ("situacao", what.into()),
                ]),
            ),
            ours: f(&i.other_amount),
            theirs: if i.b3_adjusted.is_some() {
                f(&i.b3_adjusted)
            } else {
                f(&i.b3_amount)
            },
            diff: i.diff.unwrap_or(1.0),
            source: crate::i18n::t("nucleo-score-fonte-b3-oficial"),
        });
    }
    let note = if c.material {
        let why: Vec<&str> = c
            .notes
            .iter()
            .filter(|n| crate::research::dividends::is_epoch_note(n))
            .map(String::as_str)
            .collect();
        crate::i18n::t_args(
            "nucleo-score-recon-nota-divergente",
            &crate::i18n::args(&[
                ("ticker", t.clone().into()),
                ("iguais", i64::from(c.matched).into()),
                ("diferentes", i64::from(c.value_diffs).into()),
                ("so_b3", i64::from(c.only_b3).into()),
                ("so_agenda", i64::from(c.only_other).into()),
                ("agenda", agenda.clone().into()),
            ]),
        ) + &why.iter().map(|w| format!(" {w}")).collect::<String>()
    } else {
        crate::i18n::t_args(
            "nucleo-score-recon-nota-igual",
            &crate::i18n::args(&[
                ("ticker", t.clone().into()),
                ("iguais", i64::from(c.matched).into()),
                ("agenda", agenda.clone().into()),
            ]),
        )
    };
    Ok((out, Some(note)))
}

/// Valores das fontes secundárias (brapi.dev, se houver token, e Fundamentus; só os retratos
/// guardados, nada é buscado aqui), na unidade do score: métrica → [(fonte, valor)].
fn secondary_values(
    conn: &Connection,
    ticker: &str,
    fii: bool,
) -> Result<BTreeMap<String, Vec<(String, f64)>>> {
    let mut out: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
    if let Some(b) = crate::research::brapi::cached(conn, ticker)? {
        let when = b.fetched_at.get(..10).unwrap_or("").to_string();
        for (k, v) in &b.metrics {
            out.entry(k.clone()).or_default().push((
                crate::i18n::t_args(
                    "nucleo-score-fonte-brapi",
                    &crate::i18n::args(&[("data", when.clone().into())]),
                ),
                *v,
            ));
        }
    }
    if let Some(snap) = crate::research::fundamentus::cached(conn, ticker)?
        && snap.found
    {
        let when = snap.fetched_at.get(..10).unwrap_or("").to_string();
        let theirs = |label: &str| {
            let l = crate::research::norm(label);
            snap.indicators
                .iter()
                .find(|i| crate::research::norm(&i.label) == l)
                .and_then(|i| i.value)
        };
        type Conv = fn(f64) -> Option<f64>;
        let map: &[(&str, &str, Conv)] = if fii {
            &[
                ("pvp_fii", "P/VP", |x| Some(x)),
                ("dy_12m", "Div. Yield", |x| Some(x / 100.0)),
                ("vacancy_financial", "Vacância Média", |x| Some(x / 100.0)),
            ]
        } else {
            &[
                ("roe", "ROE", |x| Some(x / 100.0)),
                ("net_margin", "Marg. Líquida", |x| Some(x / 100.0)),
                ("ebit_margin", "Marg. EBIT", |x| Some(x / 100.0)),
                ("pb", "P/VP", |x| (x > 0.0).then_some(x)),
                ("current_ratio", "Liquidez Corr", |x| Some(x)),
                ("ev_ebitda", "EV / EBITDA", |x| (x > 0.0).then_some(x)),
                ("dy", "Div. Yield", |x| Some(x / 100.0)),
                ("earnings_to_price", "P/L", |x| (x != 0.0).then(|| 1.0 / x)),
            ]
        };
        for (key, label, conv) in map {
            if let Some(v) = theirs(label).and_then(conv).filter(|v| v.is_finite()) {
                out.entry((*key).to_string()).or_default().push((
                    crate::i18n::t_args(
                        "nucleo-score-fonte-fundamentus",
                        &crate::i18n::args(&[
                            ("indicador", label.to_string().into()),
                            ("data", when.clone().into()),
                        ]),
                    ),
                    v,
                ));
            }
        }
    }
    // Métricas que não são da classe ficam de fora (ex.: pvp_fii numa ação).
    let keep: &[&str] = if fii {
        &["pvp_fii", "dy_12m", "vacancy_financial"]
    } else {
        &[
            "roe",
            "roa",
            "net_margin",
            "ebit_margin",
            "pb",
            "current_ratio",
            "ev_ebitda",
            "dy",
            "earnings_to_price",
            "net_debt_ebitda",
            "fcf_margin",
        ]
    };
    out.retain(|k, _| keep.contains(&k.as_str()));
    Ok(out)
}

/// Concordância entre valores (diferença relativa até `tol`).
fn agree(a: f64, b: f64, tol: f64) -> bool {
    let base = a.abs().max(b.abs());
    base == 0.0 || (a - b).abs() / base <= tol
}

/// Métricas que a fonte oficial não tem: preenchidas pelas fontes secundárias, marcadas.
/// Com duas fontes concordando, o valor é a média e a confiança cai pouco; com uma só ou
/// com as duas divergindo, cai mais. Devolve (rótulo, fator de confiança) por métrica.
pub fn fill_from_secondary(
    m: &Methodology,
    conn: &Connection,
    ticker: Option<&str>,
    ms: &mut Metrics,
    fii: bool,
) -> Result<Vec<String>> {
    let Some(t) = ticker else { return Ok(vec![]) };
    let cands = secondary_values(conn, t, fii)?;
    let tol = m.reconciliation.confidence_tolerance.min(0.10);
    let mut filled = vec![];
    for (key, vals) in cands {
        if !m.metrics.contains_key(&key) || m.metric(&key).unavailable.is_some() {
            continue;
        }
        let missing = ms.get(&key).is_none_or(|v| v.value.is_none() && !v.worst);
        if !missing || vals.is_empty() {
            continue;
        }
        let why = ms.get(&key).and_then(|x| x.missing.clone());
        let (value, note) = match vals.as_slice() {
            [(_, a), (_, b), ..] if agree(*a, *b, tol) => (
                (a + b) / 2.0,
                crate::i18n::t("nucleo-score-secundaria-concordam"),
            ),
            [(s1, a), (s2, b), ..] => (
                *a,
                crate::i18n::t_args(
                    "nucleo-score-secundaria-divergem",
                    &crate::i18n::args(&[
                        ("fonte1", s1.clone().into()),
                        ("valor1", fmt(*a).into()),
                        ("fonte2", s2.clone().into()),
                        ("valor2", fmt(*b).into()),
                    ]),
                ),
            ),
            [(_, a)] => (*a, crate::i18n::t("nucleo-score-secundaria-uma")),
            [] => continue,
        };
        let sources: Vec<String> = vals.iter().map(|(s, _)| s.clone()).collect();
        ms.insert(
            key.clone(),
            super::metrics::MetricValue {
                value: Some(value),
                source: Some(format!(
                    "{} ({})",
                    sources.join("; "),
                    crate::i18n::t("nucleo-score-secundaria-marca")
                )),
                note: Some(match why {
                    Some(w) => crate::i18n::t_args(
                        "nucleo-score-secundaria-nota-motivo",
                        &crate::i18n::args(&[("motivo", w.into()), ("nota", note.into())]),
                    ),
                    None => crate::i18n::t_args(
                        "nucleo-score-secundaria-nota",
                        &crate::i18n::args(&[("nota", note.into())]),
                    ),
                }),
                period: Some(crate::i18n::t("nucleo-score-secundaria-periodo")),
                ..Default::default()
            },
        );
        filled.push(m.metric_label(&key));
    }
    Ok(filled)
}

fn fmt(x: f64) -> String {
    crate::ai::alerts::decimal(crate::i18n::current(), &format!("{x:.4}"))
}

/// Os nossos valores refeitos na convenção do Fundamentus, a partir dos nossos dados oficiais
/// (docs/indicators.md, "Why the numbers differ from other sites"): EBIT como lucro bruto −
/// vendas − gerais e administrativas, margem líquida com o lucro consolidado total e valor de
/// mercado pelo total de ações do capital (com a tesouraria). Assim a conferência compara dado
/// com dado: diferença que sobra é dado errado de um dos lados, não fórmula.
pub fn fundamentus_convention(
    d: &crate::research::indicators::Data,
    market_cap: Option<f64>,
) -> BTreeMap<String, f64> {
    use crate::research::{items, periods, to_f64};
    let mut out = BTreeMap::new();
    let ltm = periods::compute_ltm(&d.facts);
    let flow = |item: &str| {
        d.facts
            .iter()
            .find(|f| f.item == item && f.period_type == crate::research::PeriodType::Ltm)
            .or_else(|| ltm.iter().find(|f| f.item == item))
            .map(|f| to_f64(f.value))
    };
    let inst =
        |item: &str| periods::instant_at(&d.facts, item, d.today).map(|f| (f.end, to_f64(f.value)));
    let (Some(rev), Some(gp)) = (flow(items::REVENUE), flow(items::GROSS_PROFIT)) else {
        return out;
    };
    if rev <= 0.0 {
        return out;
    }
    let ebit = gp - flow(items::SGA).unwrap_or(0.0);
    out.insert("ebit_margin".into(), ebit / rev);
    if let Some(n) = flow(items::NET_INCOME_TOTAL) {
        out.insert("net_margin".into(), n / rev);
    }
    // Valor de mercado pelo total de ações (com tesouraria): o do Fundamentus.
    let outstanding = inst(items::SHARES).map(|x| x.1);
    let total = inst(items::SHARES_B3)
        .or_else(|| inst(items::SHARES_TOTAL))
        .map(|x| x.1);
    let mc = match (market_cap, outstanding, total) {
        (Some(mc), Some(o), Some(t)) if o > 0.0 && t > 0.0 && (t / o - 1.0).abs() < 0.2 => {
            Some(mc * t / o)
        }
        (Some(mc), _, _) => Some(mc),
        _ => None,
    };
    if let Some(mc) = mc {
        if let (Some(debt), Some(da)) = (inst(items::GROSS_DEBT), flow(items::DA)) {
            let cash = inst(items::CASH).map(|x| x.1).unwrap_or(0.0);
            let ebitda = ebit + da;
            if ebitda > 0.0 {
                out.insert("ev_ebitda".into(), (mc + debt.1 - cash) / ebitda);
            }
        }
        if let Some(e) = inst(items::EQUITY).map(|x| x.1).filter(|e| *e > 0.0) {
            out.insert("pb".into(), mc / e);
        }
        if let Some(n) = flow(items::NET_INCOME).filter(|_| mc > 0.0) {
            out.insert("earnings_to_price".into(), n / mc);
        }
    }
    out
}

/// Conferência com as fontes secundárias (brapi.dev e Fundamentus; só os retratos guardados):
/// diferenças acima da tolerância viram itens da conferência. Com o Fundamentus, as métricas
/// de `conv` (a nossa conta na convenção dele, [`fundamentus_convention`]) entram no lugar das
/// nossas.
pub fn reconcile(
    m: &Methodology,
    conn: &Connection,
    ticker: Option<&str>,
    ms: &Metrics,
    fii: bool,
    conv: &BTreeMap<String, f64>,
) -> Result<Vec<ReconItem>> {
    let Some(t) = ticker else { return Ok(vec![]) };
    let mut out = vec![];
    for (key, vals) in secondary_values(conn, t, fii)? {
        let Some(mv) = ms.get(&key) else { continue };
        // O que já veio da fonte secundária não se confere com ela mesma.
        if mv
            .source
            .as_deref()
            .is_some_and(|s| s.contains(&crate::i18n::t("nucleo-score-secundaria-marca")))
        {
            continue;
        }
        let Some(mine) = mv.value else { continue };
        for (source, theirs) in vals {
            if theirs == 0.0 {
                continue;
            }
            // A fonte é o nome do site em qualquer idioma ("Fundamentus (P/L), lido em …").
            let same_rule = source.starts_with("Fundamentus") && conv.contains_key(&key);
            let ours = if same_rule { conv[&key] } else { mine };
            let diff = (ours - theirs).abs() / theirs.abs();
            if diff > m.reconciliation.tolerance {
                out.push(ReconItem {
                    metric: key.clone(),
                    label: if same_rule {
                        crate::i18n::t_args(
                            "nucleo-score-conferencia-mesma-regra",
                            &crate::i18n::args(&[("metrica", m.metric_label(&key).into())]),
                        )
                    } else {
                        m.metric_label(&key)
                    },
                    ours,
                    theirs,
                    diff,
                    source,
                });
            }
        }
    }
    Ok(out)
}
