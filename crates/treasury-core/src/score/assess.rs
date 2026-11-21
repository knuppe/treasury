//! Avaliação de um ativo: carrega os dados, escolhe o perfil, calcula as métricas, busca os
//! pares do dia, roda o motor e grava o snapshot. Faltando dado crítico, registra um
//! feedback para o desenvolvedor.

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::config::{Methodology, Profile, methodology};
use super::engine::{self, Peer};
use super::frameworks::Framework;
use super::metrics::Metrics;
use super::{
    AssetScore, INSUFFICIENT_BAND, MissingItem, ScoreChange, ScoreFlag, ScoreStatus, disclaimer,
    equity, feedback, insufficient_label, store, taxonomy,
};
use crate::ai::alerts::{Arg, Msg, Part};
use crate::assets::AssetClass;
use crate::error::Result;
use crate::research::data::{self, Cache, SubjectRef};
use crate::research::indicators::{self, Data, IndicatorsView};
use crate::research::to_f64;
use crate::time::fmt_date;

/// Universo de pares de uma classe (o gravado do dia, ou recalculado).
pub fn peers(
    conn: &Connection,
    m: &Methodology,
    class: &str,
    today: NaiveDate,
) -> Result<Vec<Peer>> {
    if let Some(p) = store::cached_universe(conn, class, today, &m.version)? {
        return Ok(p);
    }
    let bazin = crate::research::config::load(conn)?.bazin();
    let (rows, _) = crate::research::screen::universe(conn, today, bazin)?;
    let peers = match class {
        "equity_br" => equity::universe(conn, m, &rows, today)?,
        "reit" => super::fii::universe(conn, m, &rows, today)?,
        "equity_intl" => super::intl::universe(conn, m, &rows, today)?,
        _ => vec![],
    };
    store::store_universe(conn, class, &peers, today, &m.version);
    Ok(peers)
}

fn value_of(v: &IndicatorsView, key: &str) -> Option<(f64, String)> {
    v.indicators
        .iter()
        .find(|i| i.key == key)
        .and_then(|i| Some((i.value?, i.source.clone().unwrap_or_default())))
}

/// Base do resultado (identificação e metodologia).
#[allow(clippy::too_many_arguments)]
pub(crate) fn shell_for(
    m: &Methodology,
    d: &Data,
    class: &str,
    segment: &str,
    segment_label: &str,
    profile_key: &str,
    profile: Option<&Profile>,
    today: NaiveDate,
) -> AssetScore {
    AssetScore {
        asset_id: d.subject.asset_id.clone(),
        ticker: d.subject.ticker.clone(),
        name: d.subject.name.clone(),
        asset_class: class.to_string(),
        class_label: m.class_label(class),
        segment: segment.to_string(),
        segment_label: m.segment_label(class, segment, segment_label),
        profile: profile_key.to_string(),
        profile_label: profile
            .map(|_| m.profile_label(profile_key))
            .unwrap_or_default(),
        sector: d.subject.sector.clone(),
        as_of: fmt_date(today),
        methodology_version: m.version.clone(),
        status: ScoreStatus::NotSupported,
        score: None,
        band: INSUFFICIENT_BAND.into(),
        band_label: insufficient_label(),
        band_meaning: String::new(),
        confidence: 0.0,
        health: None,
        attractiveness: None,
        raw_score: None,
        health_raw: None,
        imputed_share: 0.0,
        dimensions: vec![],
        frameworks: vec![],
        flags: vec![],
        missing: vec![],
        top_drivers: vec![],
        top_detractors: vec![],
        peers: None,
        data_date: None,
        stale: false,
        reconciliation: vec![],
        change: None,
        events: vec![],
        event_suggestions: vec![],
        notes: vec![],
        disclaimer: disclaimer(),
    }
}

/// Classe ainda sem metodologia implementada.
pub(crate) fn not_supported(
    m: &Methodology,
    d: &Data,
    class: &str,
    today: NaiveDate,
) -> AssetScore {
    let mut s = shell_for(m, d, class, "", "", "", None, today);
    s.band_meaning = crate::i18n::t_args(
        "nucleo-score-nao-implementado",
        &crate::i18n::args(&[("classe", s.class_label.to_lowercase().into())]),
    );
    s
}

/// Sem demonstrações ou informes baixados: dado crítico ausente.
pub(crate) fn without_data_for(s: AssetScore, what: &str, why: &str) -> AssetScore {
    without_data_keyed(s, "source_data", what, why)
}

/// Como [`without_data_for`], com a chave do item (a interface oferece a correção de alguns,
/// como `sec_contact`: informar o e-mail exigido pela SEC).
pub(crate) fn without_data_keyed(
    mut s: AssetScore,
    key: &str,
    what: &str,
    why: &str,
) -> AssetScore {
    s.status = ScoreStatus::InsufficientData;
    s.band_meaning = crate::i18n::t_args(
        "nucleo-score-sem-score",
        &crate::i18n::args(&[("motivo", why.to_string().into())]),
    );
    s.missing.push(MissingItem {
        key: key.into(),
        label: what.into(),
        dimension: crate::i18n::t("nucleo-score-dimensao-dados"),
        reason: why.into(),
        critical: true,
    });
    s
}

/// Avalia um ativo (cadastrado ou do universo) hoje.
pub fn assess(conn: &Connection, r: &SubjectRef, today: NaiveDate) -> Result<AssetScore> {
    assess_with(conn, r, today, true)
}

/// Como [`assess`]; `persist: false` (leituras de clientes MCP) calcula as mudanças desde o
/// snapshot anterior sem gravar snapshot, alertas nem feedback.
pub fn assess_with(
    conn: &Connection,
    r: &SubjectRef,
    today: NaiveDate,
    persist: bool,
) -> Result<AssetScore> {
    let (mut s, known) = evaluate_with(conn, r, today)?;
    if known {
        finish(conn, methodology(), &mut s, persist);
    }
    Ok(s)
}

/// Avalia sem gravar snapshot, alertas nem feedback (ex.: componentes de um ETF).
pub(crate) fn evaluate(conn: &Connection, r: &SubjectRef, today: NaiveDate) -> Result<AssetScore> {
    evaluate_with(conn, r, today).map(|x| x.0)
}

fn evaluate_with(
    conn: &Connection,
    r: &SubjectRef,
    today: NaiveDate,
) -> Result<(AssetScore, bool)> {
    let m = methodology();
    let mut cache = Cache::default();
    let d = data::load(conn, r, today, &mut cache)?;
    let class = taxonomy::class_of(d.subject.class);
    let mut s = match (class, d.subject.class) {
        ("equity_br", _) => assess_equity(conn, m, &d, today)?,
        ("reit", AssetClass::Fii) => super::fii::assess(conn, m, &d, today)?,
        ("reit", _) | ("equity_intl", _) => super::intl::assess(conn, m, &d, today)?,
        ("fi_bank", _) | ("fi_sovereign", _) => super::fixed_income::assess(conn, m, &d, today)?,
        ("fund", _) | ("etf", _) => super::funds::assess(conn, m, &d, today)?,
        ("crypto", _) => super::crypto::assess(conn, m, &d, today)?,
        _ => return Ok((not_supported(m, &d, class, today), false)),
    };
    // Eventos qualitativos aceitos (e as sugestões dos fatos relevantes da CVM).
    let evs = super::events::list(conn, m, s.asset_id.as_deref(), s.ticker.as_deref(), today)?;
    super::events::apply(m, &mut s, &evs);
    if let Some(issuer) = &d.subject.issuer_id {
        s.event_suggestions = super::events::suggestions(conn, m, issuer, &evs, today)?;
    }
    s.events = evs;
    Ok((s, true))
}

/// Nome da faixa no idioma de agora, para os resultados gravados (snapshot do dia): as faixas
/// da metodologia e "Dados insuficientes" pela chave; outro nome (ex.: título vencido) fica
/// como foi gravado.
pub(crate) fn band_label_now(band: &str, stored: &str) -> String {
    use crate::i18n::{AVAILABLE, tr};
    match band {
        "robusto" | "saudavel" | "atencao" | "risco" => {
            crate::i18n::t(&format!("score-faixa-{band}"))
        }
        INSUFFICIENT_BAND
            if AVAILABLE
                .iter()
                .any(|l| tr(*l, "score-dados-insuficientes", None) == stored) =>
        {
            insufficient_label()
        }
        _ => stored.to_string(),
    }
}

/// A faixa no alerta: pela chave do catálogo (`score-faixa-*`, no idioma de quem lê), senão o
/// nome da metodologia.
fn band_arg(band: &str, label: &str) -> Arg {
    match band {
        "robusto" | "saudavel" | "atencao" | "risco" => {
            Msg::new(&format!("score-faixa-{band}")).into()
        }
        _ => label.into(),
    }
}

/// Mudanças desde o snapshot anterior, alertas, snapshot do dia e feedback de dado ausente.
fn finish(conn: &Connection, m: &Methodology, s: &mut AssetScore, persist: bool) {
    let subject = store::subject_key(s.asset_id.as_deref(), s.ticker.as_deref());
    if let Ok(Some(prev)) = store::previous(conn, &subject, &s.as_of, &s.methodology_version) {
        let delta = match (s.score, prev.score) {
            (Some(a), Some(b)) => Some(((a - b) * 10.0).round() / 10.0),
            _ => None,
        };
        let band_changed = prev.band != s.band;
        let new_flags: Vec<String> = s
            .flags
            .iter()
            .filter(|f| !prev.flags.iter().any(|p| p.key == f.key))
            .map(|f| f.label.clone())
            .collect();
        // Pela chave: o alerta é montado no idioma de quem lê; `change.alerts`, no de agora.
        let mut parts: Vec<Msg> = vec![];
        // Só entre dois scores definitivos: sair de "Dados insuficientes", de um parcial ou
        // virar título vencido não é piora da saúde.
        let comparable = s.status == ScoreStatus::Ok && prev.status == ScoreStatus::Ok;
        let rank = |band: &str| m.bands.iter().position(|b| b.key == band);
        let floor = m.alerts.band_floor.as_deref().and_then(rank);
        let band_worse = comparable
            && match (rank(&prev.band), rank(&s.band)) {
                (Some(p), Some(n)) => n > p && floor.is_none_or(|f| n >= f),
                _ => false,
            };
        if comparable && delta.is_some_and(|x| -x >= m.alerts.drop_points) {
            parts.push(
                Msg::new("alertas-score-caiu")
                    .arg("pontos", Arg::dec(-delta.unwrap_or(0.0), 1))
                    .arg("desde", prev.as_of.clone()),
            );
        }
        if band_worse {
            parts.push(
                Msg::new("alertas-score-faixa-piorou")
                    .arg("antes", band_arg(&prev.band, &prev.band_label))
                    .arg("agora", band_arg(&s.band, &s.band_label)),
            );
        }
        for f in s.flags.iter().filter(|f| {
            s.status != ScoreStatus::NotSupported
                && f.severity >= super::Severity::High
                && !prev.flags.iter().any(|p| p.key == f.key)
        }) {
            parts.push(Msg::new("alertas-score-red-flag").arg("flag", f.label.clone()));
        }
        let alerts: Vec<String> = parts
            .iter()
            .map(|p| p.render(crate::i18n::current()))
            .collect();
        s.change = Some(ScoreChange {
            previous_as_of: prev.as_of.clone(),
            previous_score: prev.score,
            previous_band: band_label_now(&prev.band, &prev.band_label),
            delta,
            band_changed,
            new_flags,
            alerts: alerts.clone(),
        });
        if persist && !alerts.is_empty() && s.asset_id.is_some() {
            let _ = crate::ai::alerts::create(
                conn,
                &crate::ai::alerts::NewAlert {
                    kind: "asset_score",
                    severity: crate::ai::alerts::AlertSeverity::Warning,
                    title: match s.score {
                        Some(x) => Msg::new("alertas-score-titulo").arg("nota", format!("{x:.0}")),
                        None => Msg::new("alertas-score-titulo-sem-nota"),
                    }
                    .arg("ativo", s.ticker.clone().unwrap_or_else(|| s.name.clone()))
                    .arg("faixa", band_arg(&s.band, &s.band_label)),
                    // As frases separadas por ". ", com um ponto no fim.
                    body: {
                        let mut b: Vec<Part> = vec![];
                        for (i, m) in parts.iter().enumerate() {
                            if i > 0 {
                                b.push(". ".into());
                            }
                            b.push(m.clone().into());
                        }
                        b.push(".".into());
                        b
                    },
                    source: "score",
                    dedupe_key: Some(format!("asset_score:{subject}:{}", s.as_of)),
                    ref_table: Some("asset"),
                    ref_id: s.asset_id.clone(),
                    data: serde_json::json!({"score": s.score, "previous": prev.score, "band": s.band}),
                },
            );
        }
    }
    if !persist {
        return;
    }
    store::save_snapshot(conn, s);
    // Registra o que falta (e fecha o aviso do ativo quando o score volta a ser calculado).
    feedback::record_missing(conn, m, s);
}

pub(crate) fn assess_equity(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    let layout = d.subject.layout;
    let overrides = taxonomy::overrides(conn)?;
    let ticker = d.subject.ticker.clone();
    let segment = ticker
        .as_ref()
        .and_then(|t| overrides.get(t).cloned())
        .unwrap_or_else(|| taxonomy::equity_segment(m, d.subject.sector.as_deref(), layout));
    let seg = &m.equity_br.segments[&segment];
    let profile = &m.profiles[&seg.profile];
    let mut s = shell_for(
        m,
        d,
        "equity_br",
        &segment,
        &seg.label,
        &seg.profile,
        Some(profile),
        today,
    );
    let Some(issuer_id) = d.subject.issuer_id.clone().filter(|_| !d.facts.is_empty()) else {
        return Ok(without_data_for(
            s,
            &crate::i18n::t("nucleo-score-dado-demonstracoes"),
            &crate::i18n::t("nucleo-score-sem-demonstracoes"),
        ));
    };
    let bazin = crate::research::config::load(conn)?.bazin();
    let view = indicators::compute(d, bazin);
    let liquidity = value_of(&view, "liquidity").map(|x| x.0);
    let security = match &ticker {
        Some(t) => data::security_of(conn, t, Some(d.subject.class))?,
        None => None,
    };
    let listing = equity::listing_score(
        m,
        security.as_ref().and_then(|x| x.listing_segment.as_deref()),
    );
    let fl = equity::filings(conn, Some(&issuer_id))?
        .remove(&issuer_id)
        .unwrap_or_default();
    let mut prices: Vec<(NaiveDate, f64)> = match &d.subject.asset_id {
        Some(a) => data::price_series(conn, a, today - chrono::Duration::days(400), today)?
            .into_iter()
            .map(|(dt, p)| (dt, to_f64(p)))
            .collect(),
        None => vec![],
    };
    // Sem cotações próprias (papel só do universo, ou ativo recém-cadastrado): o COTAHIST.
    if prices.len() < 120
        && let Some(t) = &ticker
    {
        let from = today - chrono::Duration::days(400);
        if let Some(q) = crate::research::store::quote_series(conn, Some(t), from)?.remove(t)
            && q.len() > prices.len()
        {
            prices = q;
        }
    }
    let inflation = equity::Inflation::new(conn);
    let price_src = d.price.as_ref().map(|(_, p)| {
        crate::i18n::t_args(
            "nucleo-pesquisa-fonte-cotacao",
            &crate::i18n::args(&[
                ("fonte", p.source.clone().into()),
                ("data", p.date.clone().into()),
            ]),
        )
    });
    let mc = value_of(&view, "market_cap").map(|(v, _)| (v, price_src.clone().unwrap_or_default()));
    let out = equity::compute(
        m,
        &d.facts,
        today,
        mc,
        value_of(&view, "dy"),
        listing,
        &fl,
        &prices,
        &inflation,
    );
    let mut metrics: Metrics = out.metrics.clone();
    let lay = layout.unwrap_or(crate::research::Layout::Industrial);
    let auditors = equity::auditors(conn, &issuer_id)?;
    let mut flags: Vec<ScoreFlag> = equity::flags(m, &out, lay, &fl, &auditors, liquidity, today);
    let mut notes = vec![];
    let mut bank_date = None;
    let mut bank_group = None;
    let mut bank_peer = None;
    if segment == "banks" {
        let b = super::bank::metrics(conn, m, &issuer_id, &metrics, today)?;
        bank_date = b.period;
        bank_group = b.group;
        bank_peer = b.peer_id;
        notes.extend(b.notes);
        flags.extend(b.flags);
        for (k, v) in b.metrics {
            metrics.insert(k, v);
        }
    }
    let peers = peers(conn, m, "equity_br", today)?;
    let peers = if segment == "banks" {
        let mut p = peers;
        p.extend(
            super::bank::peers(conn, m, today)?
                .into_iter()
                .filter(|x| Some(&x.id) != bank_peer.as_ref()),
        );
        p
    } else {
        peers
    };
    let filled = equity::fill_from_secondary(m, conn, ticker.as_deref(), &mut metrics, false)?;
    let recon = equity::reconcile(m, conn, ticker.as_deref(), &metrics, false)?;
    let mut extra = vec![];
    if !filled.is_empty() {
        extra.push((
            crate::i18n::t_args(
                "nucleo-score-confianca-fonte-secundaria",
                &crate::i18n::args(&[("metricas", engine::join_pt(&filled).into())]),
            ),
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
    notes.extend(div_note);
    let st = super::metrics::Statements::new(&d.facts, today);
    let data_date = st
        .reference
        .into_iter()
        .chain(st.balance_date)
        .chain(bank_date)
        .max();
    // Bancos: o "setor" dos pares é o porte prudencial (S1–S2, S3, S4).
    let sector = bank_group
        .clone()
        .or_else(|| d.subject.sector.as_deref().map(taxonomy::normalize_sector));
    let sector_label = bank_group
        .as_deref()
        .map(super::bank::group_label)
        .or_else(|| d.subject.sector.clone());
    let o = engine::run(engine::Input {
        m,
        profile,
        id: Some(&issuer_id),
        sector: sector.as_deref(),
        sector_label: sector_label.as_deref(),
        segment: &segment,
        segment_label: &seg.label,
        metrics: &metrics,
        peers: &peers,
        flags,
        data_date,
        today,
        extra_confidence: extra,
    });
    let mut fws: Vec<Framework> = out.frameworks;
    if !lay.is_financial() {
        let roc = super::frameworks::magic_roc(st.ltm.ebit, &st.ltm);
        fws.push(Framework {
            key: "magic_formula".into(),
            label: "Magic Formula (Greenblatt)".into(),
            value: None,
            reading: None,
            items: vec![
                super::frameworks::FrameworkItem {
                    label: crate::i18n::t("nucleo-score-magic-roc"),
                    value: roc,
                    passed: None,
                },
                super::frameworks::FrameworkItem {
                    label: "Earnings yield (EBIT ÷ EV)".into(),
                    value: metrics.get("earnings_yield").and_then(|v| v.value),
                    passed: None,
                },
            ],
            reference: "Greenblatt (2005), The Little Book That Beats the Market".into(),
            note: Some(crate::i18n::t("nucleo-score-magic-nota")),
        });
    } else {
        fws.clear();
        notes.push(crate::i18n::t("nucleo-score-nota-financeira"));
    }
    apply(&mut s, o, fws, recon, data_date);
    s.notes.extend(notes);
    Ok(s)
}

/// Copia a saída do motor para o resultado.
pub(crate) fn apply(
    s: &mut AssetScore,
    o: engine::Output,
    frameworks: Vec<Framework>,
    recon: Vec<super::ReconItem>,
    data_date: Option<NaiveDate>,
) {
    s.status = o.status;
    s.score = o.score;
    s.raw_score = o.raw_score;
    s.health_raw = o.health_raw;
    s.band = o.band;
    s.band_label = o.band_label;
    s.band_meaning = o.band_meaning;
    s.health = o.health;
    s.attractiveness = o.attractiveness;
    s.confidence = o.confidence;
    s.imputed_share = o.imputed_share;
    s.dimensions = o.dimensions;
    s.flags = o.flags;
    s.missing.extend(o.missing);
    s.top_drivers = o.top_drivers;
    s.top_detractors = o.top_detractors;
    s.peers = Some(o.peers);
    s.stale = o.stale;
    s.notes.extend(o.notes);
    s.frameworks = frameworks;
    s.reconciliation = recon;
    s.data_date = data_date.map(fmt_date);
}

/// Linha do resumo da carteira.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetScoreSummary {
    pub asset_id: Option<String>,
    pub ticker: Option<String>,
    pub name: String,
    pub asset_class: String,
    pub segment_label: String,
    pub status: ScoreStatus,
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    pub band: String,
    pub band_label: String,
    #[ts(type = "number")]
    pub confidence: f64,
    #[ts(type = "number | null")]
    pub health: Option<f64>,
    #[ts(type = "number | null")]
    pub attractiveness: Option<f64>,
    #[ts(type = "number | null")]
    pub delta: Option<f64>,
    pub flags: Vec<String>,
    pub missing_critical: Vec<String>,
    pub alerts: Vec<String>,
    pub error: Option<String>,
}

impl AssetScoreSummary {
    pub fn of(s: &AssetScore) -> Self {
        Self {
            asset_id: s.asset_id.clone(),
            ticker: s.ticker.clone(),
            name: s.name.clone(),
            asset_class: s.asset_class.clone(),
            segment_label: s.segment_label.clone(),
            status: s.status,
            score: s.score,
            band: s.band.clone(),
            band_label: band_label_now(&s.band, &s.band_label),
            confidence: s.confidence,
            health: s.health,
            attractiveness: s.attractiveness,
            delta: s.change.as_ref().and_then(|c| c.delta),
            flags: s.flags.iter().map(|f| f.label.clone()).collect(),
            missing_critical: s
                .missing
                .iter()
                .filter(|x| x.critical)
                .map(|x| x.label.clone())
                .collect(),
            alerts: s
                .change
                .as_ref()
                .map(|c| c.alerts.clone())
                .unwrap_or_default(),
            error: None,
        }
    }
}

/// Linha de um ativo cujo score não pôde ser calculado.
fn failed_summary(a: &crate::assets::Asset, e: &crate::error::Error) -> AssetScoreSummary {
    AssetScoreSummary {
        asset_id: Some(a.id.clone()),
        ticker: a.ticker.clone(),
        name: a.name.clone(),
        asset_class: taxonomy::class_of(a.class).into(),
        segment_label: String::new(),
        status: ScoreStatus::NotSupported,
        score: None,
        band: INSUFFICIENT_BAND.into(),
        band_label: insufficient_label(),
        confidence: 0.0,
        health: None,
        attractiveness: None,
        delta: None,
        flags: vec![],
        missing_critical: vec![],
        alerts: vec![],
        error: Some(e.to_string()),
    }
}

/// Máximo de ativos por consulta de [`summaries`].
pub const MAX_SUMMARIES: usize = 500;

/// Resumo do score de vários ativos cadastrados, na ordem pedida (lista de Ativos). Usa o
/// cálculo de hoje quando já existe (snapshot do dia, mesma metodologia) e calcula o resto.
pub fn summaries(
    conn: &Connection,
    ids: &[String],
    today: NaiveDate,
    persist: bool,
) -> Result<Vec<AssetScoreSummary>> {
    let version = methodology().version.clone();
    let day = fmt_date(today);
    let mut out = vec![];
    for id in ids.iter().take(MAX_SUMMARIES) {
        let Ok(a) = crate::assets::get(conn, id) else {
            continue;
        };
        if let Some(s) = super::store::snapshot_on(conn, id, &day, &version)? {
            out.push(AssetScoreSummary::of(&s));
            continue;
        }
        out.push(
            match assess_with(conn, &SubjectRef::asset(id), today, persist) {
                Ok(s) => AssetScoreSummary::of(&s),
                Err(e) => failed_summary(&a, &e),
            },
        );
    }
    Ok(out)
}

/// Scores dos ativos com posição ou na watchlist (ações e FIIs primeiro).
pub fn portfolio(
    conn: &Connection,
    today: NaiveDate,
    persist: bool,
) -> Result<Vec<AssetScoreSummary>> {
    let mut out = vec![];
    for id in data::held_or_watched(conn)? {
        let a = crate::assets::get(conn, &id)?;
        match assess_with(conn, &SubjectRef::asset(&id), today, persist) {
            Ok(s) => out.push(AssetScoreSummary::of(&s)),
            Err(e) => out.push(failed_summary(&a, &e)),
        }
    }
    let rank = |s: &AssetScoreSummary| match s.status {
        ScoreStatus::Ok | ScoreStatus::Partial => 0,
        ScoreStatus::InsufficientData => 1,
        ScoreStatus::NotSupported => 2,
    };
    out.sort_by(|a, b| {
        rank(a)
            .cmp(&rank(b))
            .then(b.score.unwrap_or(-1.0).total_cmp(&a.score.unwrap_or(-1.0)))
    });
    Ok(out)
}
