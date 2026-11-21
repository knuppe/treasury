//! Ações no exterior (SEC EDGAR), REITs dos EUA e BDRs. Mesmas métricas das ações BR
//! (`equity::compute` sobre os fatos da SEC, em dólar, deflacionados pelo CPI dos EUA), com
//! perfis e âncoras do mercado americano em `methodology/v0.1.0/classes/intl.json`. O
//! segmento sai do SIC do cadastro da SEC; o BDR herda o score do subjacente.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::Connection;

use super::config::Methodology;
use super::engine::{self, Peer};
use super::metrics::Statements;
use super::{AssetScore, equity, taxonomy};
use crate::assets::AssetClass;
use crate::error::Result;
use crate::research::data::{self, Cache, SubjectRef};
use crate::research::indicators::{self, Data};
use crate::research::screen::UniverseRow;
use crate::research::{IssuerKind, store, to_f64};

const CLASS: &str = "equity_intl";

/// Ticker americano subjacente de um BDR: tabela `bdr_underlying` da metodologia, senão o
/// código sem os dígitos finais (AAPL34 → AAPL).
pub fn bdr_underlying(_conn: &Connection, ticker: &str) -> Result<Option<String>> {
    let t = ticker.trim().to_uppercase();
    let m = super::config::methodology();
    if let Some(u) = m.aliases.get("bdr_underlying").and_then(|a| a.get(&t)) {
        return Ok(Some(u.clone()));
    }
    let base = t.trim_end_matches(|c: char| c.is_ascii_digit());
    Ok((base.len() >= 2 && base.len() < t.len()).then(|| base.to_string()))
}

/// Segmento pelo código SIC (faixas em `aliases.sic_segment`) ou pela classe REIT.
pub fn segment_of(m: &Methodology, sic: Option<&str>, reit: bool) -> String {
    let default = m
        .class_segments
        .get(CLASS)
        .map(|c| c.default_segment.clone())
        .unwrap_or_else(|| "default".into());
    if reit {
        return "reit".into();
    }
    let Some(code) = sic.and_then(|s| s.trim().parse::<u32>().ok()) else {
        return default;
    };
    for (range, seg) in m.aliases.get("sic_segment").into_iter().flatten() {
        let mut it = range
            .split('-')
            .filter_map(|x| x.trim().parse::<u32>().ok());
        if let (Some(a), Some(b)) = (it.next(), it.next())
            && (a..=b).contains(&code)
        {
            return seg.clone();
        }
    }
    default
}

fn us_inflation(conn: &Connection) -> equity::Inflation<'_> {
    equity::Inflation::with(conn, crate::indices::Deflator::UsCpi)
}

/// Pares: empresas americanas com fatos da SEC no universo (só as cadastradas e os
/// subjacentes de BDRs: costumam ser poucas, e o motor cai para a âncora absoluta).
pub fn universe(
    conn: &Connection,
    m: &Methodology,
    rows: &[UniverseRow],
    today: NaiveDate,
) -> Result<Vec<Peer>> {
    let issuers = store::all_issuers(conn)?;
    let mut all = store::load_all_facts(conn)?;
    let inflation = us_inflation(conn);
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    let mut out = vec![];
    for r in rows
        .iter()
        .filter(|r| matches!(r.class, AssetClass::StockUs | AssetClass::Reit))
    {
        let Some(id) = &r.issuer_id else { continue };
        if seen.insert(id.clone(), ()).is_some() {
            continue;
        }
        let Some(issuer) = issuers.get(id).filter(|i| i.kind == IssuerKind::CompanyUs) else {
            continue;
        };
        let Some(facts) = all.remove(id).filter(|f| !f.is_empty()) else {
            continue;
        };
        let mc = r
            .metrics
            .get("market_cap")
            .map(|v| (*v, "cotação".to_string()));
        let dy = r.metrics.get("dy").map(|v| (*v, "proventos".to_string()));
        let o = equity::compute(m, &facts, today, mc, dy, None, &vec![], &[], &inflation);
        out.push(Peer {
            id: id.clone(),
            ticker: Some(r.ticker.clone()),
            name: issuer.name.clone(),
            sector: issuer.sector.as_deref().map(crate::research::norm),
            segment: segment_of(m, issuer.segment.as_deref(), r.class == AssetClass::Reit),
            metrics: o.metrics,
        });
    }
    Ok(out)
}

/// Por que faltam as demonstrações da SEC: fonte desligada, e-mail de contato não informado
/// (a SEC recusa pedidos sem identificação; chave `sec_contact`) ou coleta ainda sem o ativo.
fn sec_gap(conn: &Connection, what: &str) -> (&'static str, String) {
    use crate::sources::{SourceId, enabled};
    if !enabled(conn, SourceId::Sec) {
        return (
            "source_data",
            crate::i18n::t_args(
                "nucleo-score-intl-sec-desligada",
                &crate::i18n::args(&[("falta", what.to_string().into())]),
            ),
        );
    }
    let default_ua = crate::research::config::load(conn)
        .map(|c| c.sec_user_agent_is_default())
        .unwrap_or(true);
    if default_ua {
        return (
            "sec_contact",
            crate::i18n::t_args(
                "nucleo-score-intl-sec-email",
                &crate::i18n::args(&[("falta", what.to_string().into())]),
            ),
        );
    }
    (
        "source_data",
        crate::i18n::t_args(
            "nucleo-score-intl-sec-coleta",
            &crate::i18n::args(&[("falta", what.to_string().into())]),
        ),
    )
}

/// Chave do item "sem histórico" (empresa recém-listada): falta do ativo, não da coleta.
pub const SHORT_HISTORY: &str = "short_history";

pub fn assess(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    match d.subject.class {
        AssetClass::Bdr => assess_bdr(conn, m, d, today),
        AssetClass::StockIntl => {
            let s = super::assess::shell_for(
                m,
                d,
                CLASS,
                "default",
                "Não financeiras (geral)",
                "equity_intl/default",
                m.profiles.get("equity_intl/default"),
                today,
            );
            Ok(super::assess::without_data_for(
                s,
                &crate::i18n::t("nucleo-score-intl-dado-demonstracoes"),
                &crate::i18n::t("nucleo-score-intl-sem-fonte"),
            ))
        }
        _ => assess_us(conn, m, d, today),
    }
}

fn assess_us(conn: &Connection, m: &Methodology, d: &Data, today: NaiveDate) -> Result<AssetScore> {
    let issuer = d
        .subject
        .issuer_id
        .as_deref()
        .and_then(|i| store::issuer(conn, i).ok().flatten());
    let overrides = taxonomy::overrides(conn)?;
    let segment = d
        .subject
        .ticker
        .as_ref()
        .and_then(|t| overrides.get(t).cloned())
        .unwrap_or_else(|| {
            segment_of(
                m,
                issuer.as_ref().and_then(|i| i.segment.as_deref()),
                d.subject.class == AssetClass::Reit,
            )
        });
    let segs = &m.class_segments[CLASS].segments;
    let seg = segs.get(&segment).unwrap_or(&segs["default"]);
    let profile = &m.profiles[&seg.profile];
    let mut s = super::assess::shell_for(
        m,
        d,
        CLASS,
        &segment,
        &seg.label,
        &seg.profile,
        Some(profile),
        today,
    );
    let Some(issuer_id) = d.subject.issuer_id.clone().filter(|_| !d.facts.is_empty()) else {
        let (key, why) = sec_gap(conn, &crate::i18n::t("nucleo-score-intl-sem-sec"));
        return Ok(super::assess::without_data_keyed(
            s,
            key,
            &crate::i18n::t("nucleo-score-intl-dado-sec"),
            &why,
        ));
    };
    // Recém-listada: ainda sem exercício completo (10-K) com receita ou lucro. Não é lacuna
    // da coleta, é falta de história; o score sai depois do primeiro balanço anual.
    let annual = |item: &str| {
        d.facts
            .iter()
            .any(|f| f.item == item && f.period_type == crate::research::PeriodType::Annual)
    };
    if !annual(crate::research::items::REVENUE) && !annual(crate::research::items::NET_INCOME) {
        return Ok(super::assess::without_data_keyed(
            s,
            SHORT_HISTORY,
            &crate::i18n::t("nucleo-score-intl-dado-historico"),
            &crate::i18n::t("nucleo-score-intl-listada-ha-pouco"),
        ));
    }
    let bazin = crate::research::config::load(conn)?.bazin();
    let view = indicators::compute(d, bazin);
    let get = |k: &str| {
        view.indicators
            .iter()
            .find(|i| i.key == k)
            .and_then(|i| Some((i.value?, i.source.clone().unwrap_or_default())))
    };
    let prices: Vec<(NaiveDate, f64)> = match &d.subject.asset_id {
        Some(a) => data::price_series(conn, a, today - chrono::Duration::days(400), today)?
            .into_iter()
            .map(|(dt, p)| (dt, to_f64(p)))
            .collect(),
        None => vec![],
    };
    let inflation = us_inflation(conn);
    let out = equity::compute(
        m,
        &d.facts,
        today,
        get("market_cap"),
        get("dy"),
        None,
        &vec![],
        &prices,
        &inflation,
    );
    let liquidity = get("liquidity").map(|x| x.0);
    let lay = crate::research::Layout::Industrial;
    let mut flags = equity::flags(m, &out, lay, &vec![], &[], None, today);
    if segment == "financial" {
        flags.retain(|f| {
            !matches!(
                f.key.as_str(),
                "altman_distress"
                    | "beneish_manipulation"
                    | "piotroski_low"
                    | "interest_coverage_low"
                    | "dsri_high"
                    | "fcf_negative_3y"
                    | "dividends_above_fcf"
            )
        });
    }
    let _ = liquidity;
    let peers = super::assess::peers(conn, m, CLASS, today)?;
    let st = Statements::new(&d.facts, today);
    let data_date = st.reference.into_iter().chain(st.balance_date).max();
    let sector = issuer
        .as_ref()
        .and_then(|i| i.sector.as_deref())
        .map(crate::research::norm);
    let sector_label = issuer.as_ref().and_then(|i| i.sector.clone());
    let o = engine::run(engine::Input {
        m,
        profile,
        id: Some(&issuer_id),
        sector: sector.as_deref(),
        sector_label: sector_label.as_deref(),
        segment: &segment,
        segment_label: &s.segment_label.clone(),
        metrics: &out.metrics,
        peers: &peers,
        flags,
        data_date,
        today,
        extra_confidence: vec![],
    });
    let fws = if segment == "financial" {
        vec![]
    } else {
        out.frameworks
    };
    super::assess::apply(&mut s, o, fws, vec![], data_date);
    s.sector = sector_label;
    s.notes.push(crate::i18n::t("nucleo-score-intl-nota-sec"));
    if segment == "financial" {
        s.notes
            .push(crate::i18n::t("nucleo-score-intl-nota-financeira"));
    }
    Ok(s)
}

/// BDR: o score do subjacente, com a identificação do BDR.
fn assess_bdr(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    let ticker = d.subject.ticker.clone().unwrap_or_default();
    let shell = || {
        super::assess::shell_for(
            m,
            d,
            CLASS,
            "default",
            "Não financeiras (geral)",
            "equity_intl/default",
            m.profiles.get("equity_intl/default"),
            today,
        )
    };
    let Some(under) = bdr_underlying(conn, &ticker)? else {
        return Ok(super::assess::without_data_for(
            shell(),
            &crate::i18n::t("nucleo-score-intl-dado-subjacente"),
            &crate::i18n::t("nucleo-score-intl-bdr-sem-empresa"),
        ));
    };
    let u = crate::research::sec::normalize_ticker(&under);
    let found = store::security(conn, &u)?.filter(|s| s.class == "stock_us");
    let Some(sec) = found else {
        let (key, why) = sec_gap(
            conn,
            &crate::i18n::t_args(
                "nucleo-score-intl-bdr-sem-sec",
                &crate::i18n::args(&[("ticker", u.clone().into())]),
            ),
        );
        return Ok(super::assess::without_data_keyed(
            shell(),
            key,
            &crate::i18n::t("nucleo-score-intl-dado-subjacente-sec"),
            &why,
        ));
    };
    let mut ud = data::load(
        conn,
        &SubjectRef::ticker(&sec.ticker),
        today,
        &mut Cache::default(),
    )?;
    ud.subject.class = AssetClass::StockUs;
    let mut s = assess_us(conn, m, &ud, today)?;
    s.asset_id = d.subject.asset_id.clone();
    s.ticker = d.subject.ticker.clone();
    s.name = d.subject.name.clone();
    s.notes.insert(
        0,
        crate::i18n::t_args(
            "nucleo-score-intl-nota-bdr",
            &crate::i18n::args(&[
                ("empresa", ud.subject.name.clone().into()),
                ("ticker", u.clone().into()),
            ]),
        ),
    );
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::config::methodology;

    #[test]
    fn segments_from_sic_and_bdr_underlying() {
        let m = methodology();
        assert_eq!(segment_of(m, Some("7372"), false), "tech");
        assert_eq!(segment_of(m, Some("6021"), false), "financial");
        assert_eq!(segment_of(m, Some("4911"), false), "utilities");
        assert_eq!(segment_of(m, Some("9999"), false), "default");
        assert_eq!(segment_of(m, None, true), "reit");
        let c = rusqlite::Connection::open_in_memory().unwrap();
        assert_eq!(
            bdr_underlying(&c, "AAPL34").unwrap().as_deref(),
            Some("AAPL")
        );
        assert_eq!(
            bdr_underlying(&c, "GOGL34").unwrap().as_deref(),
            Some("GOOGL")
        );
        assert_eq!(bdr_underlying(&c, "XPTO").unwrap(), None);
    }
}
