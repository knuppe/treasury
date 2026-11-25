//! FIIs (§7.2, §10.1): métricas a partir dos informes mensal e trimestral da CVM e da
//! cotação, classificação pela composição do ativo (tijolo, papel, fundo de fundos,
//! híbrido), pares do dia e red flags.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::Connection;

use super::config::Methodology;
use super::engine::{self, Peer};
use super::metrics::{MetricValue, Metrics, br};
use super::taxonomy::{self, ReitComposition};
use super::{AssetScore, ScoreFlag, equity};
use crate::assets::AssetClass;
use crate::error::Result;
use crate::research::indicators::{self, Data};
use crate::research::screen::UniverseRow;
use crate::research::{Fact, IssuerKind, PeriodType, items, store, to_f64};
use crate::time::add_months;

/// Itens dos informes de FII gravados pela coleta da CVM.
pub mod keys {
    pub const TOTAL_ASSETS: &str = "fii_total_assets";
    pub const LIABILITIES: &str = "fii_liabilities";
    pub const ACQ_OBLIGATIONS: &str = "fii_acq_obligations";
    pub const REAL_ESTATE: &str = "fii_real_estate";
    pub const CRI: &str = "fii_cri";
    pub const FII_SHARES: &str = "fii_fii_shares";
    pub const ADMIN_FEE: &str = "fii_admin_fee_pct";
    pub const VACANCY_FIN: &str = "fii_vacancy_financial";
    pub const VACANCY_PHYS: &str = "fii_vacancy_physical";
    pub const DEFAULT_RATE: &str = "fii_default_rate";
    pub const DEFAULT_RATE_AREA: &str = "fii_default_rate_area";
    pub const TOP_PROPERTY: &str = "fii_top_property_share";
    pub const TOP_PROPERTY_AREA: &str = "fii_top_property_share_area";
    pub const PROPERTIES: &str = "fii_properties";
    pub const TOP_TENANT: &str = "fii_top_tenant_share";
    pub const WALT: &str = "fii_walt_years";
    pub const RECURRING: &str = "fii_recurring_result";
    pub const DECLARED: &str = "fii_declared_income";
    pub const CRI_TOP_ISSUER: &str = "fii_cri_top_issuer_share";
    pub const SALES: &str = "fii_sales_result";
    pub const LEASE_OVER_36M: &str = "fii_lease_over_36m_share";
    pub const WALT_MIN: &str = "fii_walt_min_years";
}

/// Two decimals in the installation's language (`1,34`, `1.34`).
fn dec2(v: f64) -> String {
    crate::i18n::number(crate::i18n::current(), (v * 100.0).round() as i64)
}

/// Share of the 4-quarter result from sales of assets above which the coverage gets a note.
const SALES_NOTE_SHARE: f64 = 0.10;

/// Quantas vezes a taxa de administração do informe mensal é anualizada (o informe traz a
/// despesa do mês sobre o patrimônio).
pub const ADMIN_FEE_ANNUALIZE: f64 = 12.0;

fn latest<'a>(facts: &'a [Fact], item: &str, pt: PeriodType) -> Option<&'a Fact> {
    facts
        .iter()
        .filter(|f| f.item == item && f.period_type == pt)
        .max_by_key(|f| f.end)
}

fn month_at<'a>(facts: &'a [Fact], item: &str, at: NaiveDate) -> Option<&'a Fact> {
    facts
        .iter()
        .filter(|f| f.item == item && f.period_type == PeriodType::Month && f.end <= at)
        .max_by_key(|f| f.end)
}

pub struct FiiInput<'a> {
    pub facts: &'a [Fact],
    pub today: NaiveDate,
    pub pvp: Option<(f64, String)>,
    pub dy: Option<(f64, String)>,
    pub liquidity: Option<(f64, String)>,
    pub cdi_12m: Option<f64>,
    pub inflation: &'a dyn Fn(NaiveDate, NaiveDate) -> Option<f64>,
}

pub struct FiiOutput {
    pub metrics: Metrics,
    pub composition: ReitComposition,
    pub data_date: Option<NaiveDate>,
}

/// `missing`: chave do motivo quando falta o fato.
fn from_fact(f: Option<&Fact>, period: impl Fn(&Fact) -> String, missing: &str) -> MetricValue {
    match f {
        Some(f) => MetricValue {
            value: Some(to_f64(f.value)),
            period: Some(period(f)),
            period_end: Some(f.end),
            source: Some(f.document.clone()),
            ..Default::default()
        },
        None => MetricValue::missing(crate::i18n::t(missing)),
    }
}

fn quarter_label(f: &Fact) -> String {
    crate::i18n::t_args(
        "nucleo-score-fii-periodo-trimestral",
        &crate::i18n::args(&[("data", br(f.end).into())]),
    )
}

/// "informe mensal de <mm/aaaa>".
fn month_label(d: NaiveDate) -> String {
    crate::i18n::t_args(
        "nucleo-score-fii-periodo-mensal",
        &crate::i18n::args(&[("mes", d.format("%m/%Y").to_string().into())]),
    )
}

/// Meses que um dado do informe trimestral pode ficar atrás do informe mais recente do fundo
/// (um campo deixado em branco no último trimestre não puxa o de um ano antes).
const QUARTERLY_STALE_MONTHS: i32 = 6;

pub fn fii_metrics(i: &FiiInput<'_>) -> FiiOutput {
    let f = i.facts;
    let mut m = Metrics::new();
    // Último informe trimestral do fundo, e cada dado só se for dele (ou de pouco antes).
    let report_end = f
        .iter()
        .filter(|x| x.source == "cvm_fii_trimestral")
        .map(|x| x.end)
        .max();
    let inst = |k: &str| {
        latest(f, k, PeriodType::Instant)
            .filter(|x| report_end.is_none_or(|r| x.end >= add_months(r, -QUARTERLY_STALE_MONTHS)))
    };
    // O informe chegou mas sem o campo (muitos deixam a receita por imóvel zerada).
    let missing = || match report_end {
        Some(r) => crate::i18n::t_args(
            "nucleo-score-fii-trimestral-sem-item",
            &crate::i18n::args(&[("data", br(r).into())]),
        ),
        None => crate::i18n::t("nucleo-score-fii-sem-trimestral"),
    };
    // Sem a participação de cada imóvel na receita, a vacância e a inadimplência saem
    // ponderadas pela área, do mesmo informe (antes de recorrer a uma fonte secundária).
    let by_area = |k: &str, area: &str| -> MetricValue {
        match (inst(k), inst(area)) {
            (Some(x), _) => from_fact(Some(x), quarter_label, ""),
            (None, Some(x)) => MetricValue {
                note: Some(crate::i18n::t("nucleo-score-fii-nota-por-area")),
                ..from_fact(Some(x), quarter_label, "")
            },
            (None, None) => MetricValue::missing(missing()),
        }
    };
    m.insert(
        "vacancy_financial".into(),
        by_area(keys::VACANCY_FIN, keys::VACANCY_PHYS),
    );
    m.insert(
        "default_rate".into(),
        by_area(keys::DEFAULT_RATE, keys::DEFAULT_RATE_AREA),
    );
    // Maior imóvel: zero na receita é campo não preenchido; aí vale o maior pela área.
    let top_rev = inst(keys::TOP_PROPERTY).filter(|x| !x.value.is_zero());
    m.insert(
        "top_property_share".into(),
        match (top_rev, inst(keys::TOP_PROPERTY_AREA)) {
            (Some(x), _) => from_fact(Some(x), quarter_label, ""),
            (None, Some(x)) => MetricValue {
                note: Some(crate::i18n::t("nucleo-score-fii-nota-por-area")),
                ..from_fact(Some(x), quarter_label, "")
            },
            (None, None) => MetricValue::missing(missing()),
        },
    );
    for (key, item) in [
        ("walt_years", keys::WALT),
        ("properties", keys::PROPERTIES),
        ("top_tenant_share", keys::TOP_TENANT),
        ("cri_top_issuer_share", keys::CRI_TOP_ISSUER),
    ] {
        // Participação zero no maior imóvel/inquilino é campo não preenchido, não dado.
        let fact = inst(item).filter(|x| item != keys::TOP_TENANT || !x.value.is_zero());
        m.insert(
            key.into(),
            match fact {
                Some(x) => from_fact(Some(x), quarter_label, ""),
                None => MetricValue::missing(missing()),
            },
        );
    }
    // WALT: the report stops at "above 36 months", counted as 5 years (premise). When part of
    // the revenue is there, the note gives that share and the WALT with 3 years (the least).
    if let Some(w) = m.get_mut("walt_years")
        && let Some(end) = w.period_end
        && let Some(over) = inst(keys::LEASE_OVER_36M)
            .filter(|x| x.end == end && x.value > rust_decimal::Decimal::ZERO)
        && let Some(min) = inst(keys::WALT_MIN).filter(|x| x.end == end)
    {
        w.note = Some(crate::i18n::t_args(
            "nucleo-score-fii-nota-walt-acima-36",
            &crate::i18n::args(&[
                ("pct", format!("{:.0}%", to_f64(over.value) * 100.0).into()),
                ("minimo", dec2(to_f64(min.value)).into()),
            ]),
        ));
    }
    // Resultado recorrente × rendimentos declarados nos últimos 4 trimestres.
    let quarters = |k: &str| -> Vec<&Fact> {
        let mut v: Vec<&Fact> = f
            .iter()
            .filter(|x| x.item == k && x.period_type == PeriodType::Quarter)
            .collect();
        v.sort_by_key(|x| x.end);
        v.into_iter().rev().take(4).collect()
    };
    let (rec, dec) = (quarters(keys::RECURRING), quarters(keys::DECLARED));
    m.insert(
        "income_coverage".into(),
        if rec.len() == 4 && dec.len() == 4 {
            let r: f64 = rec.iter().map(|x| to_f64(x.value)).sum();
            let d: f64 = dec.iter().map(|x| to_f64(x.value)).sum();
            // Sales of assets in the same quarters (not recurring): a note when they weigh.
            let sales: f64 = rec
                .iter()
                .filter_map(|q| {
                    f.iter()
                        .find(|x| x.item == keys::SALES && x.end == q.end)
                        .map(|x| to_f64(x.value))
                })
                .sum();
            let note = (r > 0.0 && sales / r > SALES_NOTE_SHARE).then(|| {
                crate::i18n::t_args(
                    "nucleo-score-fii-nota-cobertura-vendas",
                    &crate::i18n::args(&[
                        ("pct", format!("{:.0}%", sales / r * 100.0).into()),
                        ("sem", dec2((r - sales) / d).into()),
                    ]),
                )
            });
            if d > 0.0 {
                MetricValue {
                    note,
                    period: Some(crate::i18n::t_args(
                        "nucleo-score-fii-periodo-4-trimestres",
                        &crate::i18n::args(&[("data", br(rec[0].end).into())]),
                    )),
                    period_end: Some(rec[0].end),
                    source: Some(crate::i18n::t("nucleo-score-fii-fonte-trimestrais")),
                    ..MetricValue::ok(r / d)
                }
            } else {
                MetricValue::missing(crate::i18n::t(
                    "nucleo-score-fii-sem-rendimentos-declarados",
                ))
            }
        } else {
            MetricValue::missing(crate::i18n::t("nucleo-score-fii-poucos-trimestrais"))
        },
    );
    let nav = latest(f, items::NAV, PeriodType::Month);
    let month = nav.map(|n| n.end);
    let ratio_to_nav = |k: &str, missing: &str| match (month.and_then(|d| month_at(f, k, d)), nav) {
        (Some(x), Some(n)) if n.value > rust_decimal::Decimal::ZERO => MetricValue {
            period: Some(month_label(x.end)),
            period_end: Some(x.end),
            source: Some(x.document.clone()),
            ..MetricValue::ok(to_f64(x.value) / to_f64(n.value))
        },
        _ => MetricValue::missing(crate::i18n::t(missing)),
    };
    m.insert(
        "liabilities_to_nav".into(),
        ratio_to_nav(keys::LIABILITIES, "nucleo-score-fii-sem-passivo"),
    );
    m.insert(
        "acq_obligations_to_nav".into(),
        ratio_to_nav(keys::ACQ_OBLIGATIONS, "nucleo-score-fii-sem-obrigacoes"),
    );
    // `period` e `missing`: chaves do catálogo.
    let market = |x: &Option<(f64, String)>, period: &str, missing: &str| match x {
        Some((v, s)) => MetricValue {
            period: Some(crate::i18n::t(period)),
            source: Some(s.clone()),
            ..MetricValue::ok(*v)
        },
        None => MetricValue::missing(crate::i18n::t(missing)),
    };
    m.insert(
        "pvp_fii".into(),
        market(
            &i.pvp,
            "nucleo-score-fii-periodo-pvp",
            "nucleo-score-fii-sem-pvp",
        ),
    );
    m.insert(
        "dy_12m".into(),
        market(
            &i.dy,
            "nucleo-score-periodo-12-meses",
            "nucleo-score-fii-sem-dy",
        ),
    );
    m.insert(
        "avg_volume".into(),
        market(
            &i.liquidity,
            "nucleo-score-periodo-ultimos-pregoes",
            "nucleo-pesquisa-ind-sem-volume",
        ),
    );
    m.insert(
        "dy_spread_cdi".into(),
        match (&i.dy, i.cdi_12m) {
            (Some((dy, _)), Some(cdi)) => MetricValue {
                period: Some(crate::i18n::t("nucleo-score-periodo-12-meses")),
                source: Some(crate::i18n::t("nucleo-score-fii-fonte-dy-cdi")),
                ..MetricValue::ok(dy - cdi)
            },
            (None, _) => MetricValue::missing(crate::i18n::t("nucleo-score-fii-sem-dy-fundo")),
            (_, None) => MetricValue::missing(crate::i18n::t("nucleo-score-fii-sem-cdi")),
        },
    );
    // Taxa de administração: média dos últimos 12 informes, anualizada.
    let fees: Vec<f64> = {
        let mut v: Vec<&Fact> = f
            .iter()
            .filter(|x| x.item == keys::ADMIN_FEE && x.period_type == PeriodType::Month)
            .collect();
        v.sort_by_key(|x| x.end);
        v.iter().rev().take(12).map(|x| to_f64(x.value)).collect()
    };
    m.insert(
        "admin_fee".into(),
        if fees.len() >= 3 {
            MetricValue {
                period: Some(crate::i18n::t_args(
                    "nucleo-score-fii-periodo-taxa",
                    &crate::i18n::args(&[("n", (fees.len() as i64).into())]),
                )),
                source: Some(crate::i18n::t("nucleo-score-fii-fonte-taxa")),
                ..MetricValue::ok(
                    fees.iter().sum::<f64>() / fees.len() as f64 * ADMIN_FEE_ANNUALIZE,
                )
            }
        } else {
            MetricValue::missing(crate::i18n::t("nucleo-score-fii-poucas-taxas"))
        },
    );
    let nps_now = latest(f, items::NAV_PER_SHARE, PeriodType::Month);
    let nps_then = nps_now.and_then(|n| month_at(f, items::NAV_PER_SHARE, add_months(n.end, -12)));
    m.insert(
        "nav_per_share_real_growth".into(),
        match (nps_now, nps_then) {
            (Some(a), Some(b))
                if b.value > rust_decimal::Decimal::ZERO && (a.end - b.end).num_days() > 300 =>
            {
                let nominal = to_f64(a.value) / to_f64(b.value);
                let (v, note) = match (i.inflation)(b.end, a.end) {
                    Some(k) if k > 0.0 => (nominal / k - 1.0, "nucleo-score-fii-ipca-descontado"),
                    _ => (nominal - 1.0, "nucleo-score-fii-sem-ipca"),
                };
                MetricValue {
                    period: Some(crate::i18n::t_args(
                        "nucleo-score-periodo-de-a",
                        &crate::i18n::args(&[
                            ("de", b.end.format("%m/%Y").to_string().into()),
                            ("ate", a.end.format("%m/%Y").to_string().into()),
                        ]),
                    )),
                    period_end: Some(a.end),
                    source: Some(crate::i18n::t("nucleo-score-fii-fonte-vp-cota")),
                    note: Some(crate::i18n::t(note)),
                    ..MetricValue::ok(v)
                }
            }
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-fii-sem-vp-anterior")),
        },
    );
    m.insert(
        "holders".into(),
        from_fact(
            latest(f, items::HOLDERS, PeriodType::Month),
            |x| month_label(x.end),
            "nucleo-score-fii-sem-cotistas",
        ),
    );
    let share = |k: &str| -> Option<f64> {
        let d = month?;
        let ta = month_at(f, keys::TOTAL_ASSETS, d)?;
        let x = month_at(f, k, d)?;
        (ta.value > rust_decimal::Decimal::ZERO).then(|| to_f64(x.value) / to_f64(ta.value))
    };
    let composition = ReitComposition {
        cri: share(keys::CRI),
        fii: share(keys::FII_SHARES),
        real_estate: share(keys::REAL_ESTATE),
    };
    let data_date = [inst(keys::VACANCY_FIN).map(|x| x.end), month]
        .into_iter()
        .flatten()
        .max();
    FiiOutput {
        metrics: m,
        composition,
        data_date,
    }
}

/// CDI acumulado nos 12 meses até `today` (fração).
pub fn cdi_12m(conn: &Connection, today: NaiveDate) -> Option<f64> {
    let rates = crate::indices::daily_rates(conn, "cdi", add_months(today, -12), today).ok()?;
    if rates.len() < 200 {
        return None;
    }
    Some(
        rates
            .iter()
            .map(|(_, r)| 1.0 + to_f64(*r) / 100.0)
            .product::<f64>()
            - 1.0,
    )
}

fn row_value(r: &UniverseRow, k: &str, src: &str) -> Option<(f64, String)> {
    r.metrics.get(k).map(|v| (*v, src.to_string()))
}

/// Pares: todos os FIIs listados com informe.
pub fn universe(
    conn: &Connection,
    m: &Methodology,
    rows: &[UniverseRow],
    today: NaiveDate,
) -> Result<Vec<Peer>> {
    let issuers = store::all_issuers(conn)?;
    let mut all = store::load_all_facts(conn)?;
    let inflation = equity::Inflation::new(conn);
    let infl = |a: NaiveDate, b: NaiveDate| inflation.factor(a, b);
    let cdi = cdi_12m(conn, today);
    let overrides = taxonomy::overrides(conn)?;
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    let mut out = vec![];
    for r in rows.iter().filter(|r| r.class == AssetClass::Fii) {
        let Some(id) = &r.issuer_id else { continue };
        if seen.insert(id.clone(), ()).is_some() {
            continue;
        }
        let Some(issuer) = issuers.get(id).filter(|i| i.kind == IssuerKind::Fii) else {
            continue;
        };
        let Some(facts) = all.remove(id) else {
            continue;
        };
        let o = fii_metrics(&FiiInput {
            facts: &facts,
            today,
            pvp: row_value(r, "pvp", "B3 COTAHIST"),
            dy: row_value(r, "dy", "proventos"),
            liquidity: row_value(r, "liquidity", "B3 COTAHIST"),
            cdi_12m: cdi,
            inflation: &infl,
        });
        let segment = overrides
            .get(&r.ticker)
            .cloned()
            .unwrap_or_else(|| taxonomy::reit_segment(m, o.composition));
        out.push(Peer {
            id: id.clone(),
            ticker: Some(r.ticker.clone()),
            name: issuer.name.clone(),
            sector: taxonomy::reit_subsegment(m, issuer.segment.as_deref()),
            segment,
            metrics: o.metrics,
        });
    }
    Ok(out)
}

fn flag(m: &Methodology, key: &str, detail: String) -> Option<ScoreFlag> {
    let d = m.flags.get(key)?;
    Some(ScoreFlag {
        key: key.into(),
        label: d.label.clone(),
        severity: d.severity,
        cap: d.cap,
        penalty: d.penalty,
        detail,
    })
}

pub fn flags(m: &Methodology, ms: &Metrics) -> Vec<ScoreFlag> {
    let mut v = vec![];
    let val = |k: &str| ms.get(k).and_then(|x| x.value);
    let thr = |k: &str, d: f64| m.flags.get(k).and_then(|f| f.threshold).unwrap_or(d);
    let pct = |x: f64| crate::ai::alerts::pct(crate::i18n::current(), x, 1);
    if let Some(x) = val("vacancy_financial")
        && x > thr("fii_vacancy_high", 0.25)
    {
        v.extend(flag(
            m,
            "fii_vacancy_high",
            crate::i18n::t_args(
                "nucleo-score-fii-flag-vacancia",
                &crate::i18n::args(&[("pct", pct(x).into())]),
            ),
        ));
    }
    if let Some(x) = val("top_tenant_share")
        && x > thr("fii_tenant_concentration", 0.4)
    {
        v.extend(flag(
            m,
            "fii_tenant_concentration",
            crate::i18n::t_args(
                "nucleo-score-fii-flag-inquilino",
                &crate::i18n::args(&[("pct", pct(x).into())]),
            ),
        ));
    }
    if let Some(x) = val("avg_volume")
        && x < thr("low_liquidity", 1_000_000.0)
    {
        v.extend(flag(
            m,
            "low_liquidity",
            crate::i18n::t_args(
                "nucleo-score-flag-volume-medio",
                &crate::i18n::args(&[("valor", engine::fmt_value(super::Unit::Money, x).into())]),
            ),
        ));
    }
    v
}

pub fn assess(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    let bazin = crate::research::config::load(conn)?.bazin();
    let view = indicators::compute(d, bazin);
    let get = |k: &str| {
        view.indicators
            .iter()
            .find(|i| i.key == k)
            .and_then(|i| Some((i.value?, i.source.clone().unwrap_or_default())))
    };
    let inflation = equity::Inflation::new(conn);
    let infl = |a: NaiveDate, b: NaiveDate| inflation.factor(a, b);
    let o = fii_metrics(&FiiInput {
        facts: &d.facts,
        today,
        pvp: get("pvp"),
        dy: get("dy"),
        liquidity: get("liquidity"),
        cdi_12m: cdi_12m(conn, today),
        inflation: &infl,
    });
    let overrides = taxonomy::overrides(conn)?;
    let segment = d
        .subject
        .ticker
        .as_ref()
        .and_then(|t| overrides.get(t).cloned())
        .unwrap_or_else(|| taxonomy::reit_segment(m, o.composition));
    let seg = &m.reit.segments[&segment];
    let profile = &m.profiles[&seg.profile];
    let mut s = super::assess::shell_for(
        m,
        d,
        "reit",
        &segment,
        &seg.label,
        &seg.profile,
        Some(profile),
        today,
    );
    let Some(issuer_id) = d.subject.issuer_id.clone().filter(|_| !d.facts.is_empty()) else {
        return Ok(super::assess::without_data_for(
            s,
            &crate::i18n::t("nucleo-score-fii-dado-informes"),
            &crate::i18n::t("nucleo-score-fii-sem-informes"),
        ));
    };
    let peers = super::assess::peers(conn, m, "reit", today)?;
    let mut o = o;
    let filled =
        equity::fill_from_secondary(m, conn, d.subject.ticker.as_deref(), &mut o.metrics, true)?;
    let flags = flags(m, &o.metrics);
    let recon = equity::reconcile(
        m,
        conn,
        d.subject.ticker.as_deref(),
        &o.metrics,
        true,
        &Default::default(),
    )?;
    let mut extra = vec![];
    if !filled.is_empty() {
        extra.push((
            crate::i18n::t_args(
                "nucleo-score-confianca-fonte-secundaria",
                &crate::i18n::args(&[("metricas", super::engine::join_pt(&filled).into())]),
            ),
            m.confidence.reconcile_factor,
        ));
    }
    // DY sem proventos pagos (estimado pelos informes): conta, mas com menos confiança.
    if view
        .indicators
        .iter()
        .any(|x| x.key == "dy" && x.estimated && x.value.is_some())
    {
        extra.push((
            crate::i18n::t("nucleo-score-confianca-dy-estimado"),
            m.confidence.reconcile_factor,
        ));
    }
    if flags.iter().any(|f| f.key == "low_liquidity") {
        extra.push((
            crate::i18n::t("nucleo-score-confianca-liquidez-baixa"),
            m.confidence.low_liquidity_factor,
        ));
    }
    if recon
        .iter()
        .any(|r| r.diff > m.reconciliation.confidence_tolerance)
    {
        extra.push((
            crate::i18n::t("nucleo-score-confianca-divergencia"),
            m.confidence.reconcile_factor,
        ));
    }
    // Proventos conferidos com a B3 (oficial): só informativo, o DY já usa a B3 se divergir.
    let (div_recon, div_note) = equity::reconcile_dividends(conn, d, today)?;
    let mut recon = recon;
    recon.extend(div_recon);
    let sector = taxonomy::reit_subsegment(m, d.subject.segment.as_deref());
    let out = engine::run(engine::Input {
        m,
        profile,
        id: Some(&issuer_id),
        sector: sector.as_deref(),
        sector_label: sector.as_deref(),
        segment: &segment,
        segment_label: &seg.label,
        metrics: &o.metrics,
        peers: &peers,
        flags,
        data_date: o.data_date,
        today,
        extra_confidence: extra,
    });
    s.sector = sector.or(d.subject.segment.clone());
    super::assess::apply(&mut s, out, vec![], recon, o.data_date);
    s.notes.extend(div_note);
    let c = o.composition;
    let pct = |x: Option<f64>| {
        x.map(|v| format!("{:.0}%", v * 100.0))
            .unwrap_or_else(|| "?".into())
    };
    s.notes.push(crate::i18n::t_args(
        "nucleo-score-fii-nota-composicao",
        &crate::i18n::args(&[
            ("cri", pct(c.cri).into()),
            ("fii", pct(c.fii).into()),
            ("imoveis", pct(c.real_estate).into()),
        ]),
    ));
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn q(item: &str, end: &str, v: Decimal) -> Fact {
        Fact {
            item: item.into(),
            period_type: PeriodType::Instant,
            start: None,
            end: crate::time::parse_date(end).unwrap(),
            value: v,
            currency: "ratio".into(),
            source: "cvm_fii_trimestral".into(),
            document: format!("Informe trimestral FII {end} v1"),
            priority: 2,
        }
    }

    /// Quarterly report without revenue per property (like XPLG11's in 2026): vacancy and
    /// delinquency and largest property by area; WALT from a year earlier is left out.
    #[test]
    fn quarterly_without_revenue_shares() {
        let facts = vec![
            q(keys::VACANCY_PHYS, "2026-06-30", Decimal::new(165, 3)),
            q(keys::DEFAULT_RATE_AREA, "2026-06-30", Decimal::ZERO),
            q(keys::PROPERTIES, "2026-06-30", Decimal::from(30)),
            q(keys::TOP_PROPERTY, "2026-06-30", Decimal::ZERO),
            q(keys::TOP_PROPERTY_AREA, "2026-06-30", Decimal::new(8, 2)),
            q(keys::WALT, "2025-06-30", Decimal::new(38, 1)),
        ];
        let o = fii_metrics(&FiiInput {
            facts: &facts,
            today: crate::time::parse_date("2026-10-27").unwrap(),
            pvp: None,
            dy: None,
            liquidity: None,
            cdi_12m: None,
            inflation: &|_, _| None,
        });
        let m = &o.metrics;
        let vac = &m["vacancy_financial"];
        assert_eq!(vac.value, Some(0.165));
        assert!(vac.note.as_deref().unwrap().contains("área"));
        assert_eq!(m["default_rate"].value, Some(0.0));
        assert_eq!(m["properties"].value, Some(30.0));
        let top = &m["top_property_share"];
        assert_eq!(top.value, Some(0.08));
        assert!(top.note.as_deref().unwrap().contains("área"));
        for k in ["walt_years", "top_tenant_share"] {
            assert!(m[k].value.is_none(), "{k}");
            assert!(
                m[k].missing.as_deref().unwrap().contains("30/06/2026"),
                "{k}"
            );
        }
    }

    /// WALT with most contracts above 36 months: the value keeps the 5-year premise and the
    /// note gives the share and the minimum; a sale of assets in the coverage gets a note.
    #[test]
    fn walt_premise_and_sales_in_the_coverage_are_explained() {
        let quarter = |item: &str, end: &str, v: i64| Fact {
            period_type: PeriodType::Quarter,
            currency: "BRL".into(),
            ..q(item, end, Decimal::from(v))
        };
        let mut facts = vec![
            q(keys::WALT, "2026-09-30", Decimal::new(44, 1)),
            q(keys::LEASE_OVER_36M, "2026-09-30", Decimal::new(8, 1)),
            q(keys::WALT_MIN, "2026-09-30", Decimal::new(28, 1)),
        ];
        for end in ["2025-12-31", "2026-03-31", "2026-06-30", "2026-09-30"] {
            facts.push(quarter(keys::RECURRING, end, 1000));
            facts.push(quarter(keys::DECLARED, end, 900));
        }
        facts.push(quarter(keys::SALES, "2026-09-30", 600));
        let o = fii_metrics(&FiiInput {
            facts: &facts,
            today: crate::time::parse_date("2026-11-24").unwrap(),
            pvp: None,
            dy: None,
            liquidity: None,
            cdi_12m: None,
            inflation: &|_, _| None,
        });
        let w = &o.metrics["walt_years"];
        assert_eq!(w.value, Some(4.4));
        let note = w.note.as_deref().unwrap();
        assert!(note.contains("80%") && note.contains("2,80"), "{note}");
        let c = &o.metrics["income_coverage"];
        assert!((c.value.unwrap() - 4000.0 / 3600.0).abs() < 1e-9);
        let note = c.note.as_deref().unwrap();
        assert!(note.contains("15%") && note.contains("0,94"), "{note}");
    }
}
