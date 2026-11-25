//! Consultas da análise de ativos: fundamentos brutos com documento de origem, histórico de
//! 10 anos e trimestres, comparador, documentos IPE e estado da coleta.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::data::{self, Cache, SubjectRef};
use super::indicators::{self, BazinParams, IndicatorGroup, IndicatorUnit, Subject};
use super::{Issuer, PeriodType, Security, cvm, dstr, items, periods, store, to_f64};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::portfolio::records::to_minor;
use crate::time::{add_months, fmt_date, parse_date};

/// Link do documento de origem, quando dá para montar.
pub fn document_url(issuer_id: &str, document: &str) -> Option<String> {
    if let Some(i) = document.find("doc. CVM ") {
        let id: String = document[i + 9..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if !id.is_empty() {
            return Some(cvm::document_link(&id));
        }
    }
    if let Some(cik) = issuer_id.strip_prefix("cik:") {
        let accn = document
            .split_whitespace()
            .find(|w| w.len() == 20 && w.matches('-').count() == 2)?;
        let cik: u64 = cik.parse().ok()?;
        return Some(format!(
            "https://www.sec.gov/Archives/edgar/data/{cik}/{}/",
            accn.replace('-', "")
        ));
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FundamentalValue {
    pub item: String,
    pub label: String,
    pub period_type: PeriodType,
    pub period_start: Option<String>,
    pub period_end: String,
    /// Decimal em texto, na moeda (`currency`) ou em ações/cotas.
    pub value: String,
    pub currency: String,
    pub source: String,
    pub document: String,
    pub document_url: Option<String>,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FundamentalsView {
    pub subject: Subject,
    pub issuer: Option<Issuer>,
    pub securities: Vec<Security>,
    /// Data de referência dos últimos 12 meses.
    pub reference_date: Option<String>,
    /// Último valor de cada item: LTM (fluxos), saldo mais recente, informe mais recente.
    pub latest: Vec<FundamentalValue>,
    /// Último exercício completo.
    pub annual: Vec<FundamentalValue>,
    pub last_sync_at: Option<String>,
}

fn values(conn: &Connection, issuer_id: &str) -> Result<Vec<FundamentalValue>> {
    let mut stmt = conn.prepare(
        "SELECT item, period_type, period_start, period_end, value, currency, source, document, fetched_at
         FROM fundamental WHERE issuer_id = ?1 ORDER BY item, period_type, period_end",
    )?;
    let rows = stmt
        .query_map([issuer_id], |r| {
            let item: String = r.get(0)?;
            let pt: String = r.get(1)?;
            let document: String = r.get(7)?;
            Ok(FundamentalValue {
                label: items::label(&item),
                item,
                period_type: PeriodType::parse(&pt).unwrap_or(PeriodType::Instant),
                period_start: r.get(2)?,
                period_end: r.get(3)?,
                value: r.get(4)?,
                currency: r.get(5)?,
                source: r.get(6)?,
                document_url: document_url(issuer_id, &document),
                document,
                fetched_at: r.get(8)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn fundamentals(
    conn: &Connection,
    r: &SubjectRef,
    today: NaiveDate,
) -> Result<FundamentalsView> {
    let d = data::load(conn, r, today, &mut Cache::default())?;
    let issuer = match &d.subject.issuer_id {
        Some(i) => store::issuer(conn, i)?,
        None => None,
    };
    let (securities, all) = match &issuer {
        Some(i) => (store::securities_of(conn, &i.id)?, values(conn, &i.id)?),
        None => (vec![], vec![]),
    };
    let mut latest: BTreeMap<String, FundamentalValue> = BTreeMap::new();
    for v in all.iter().filter(|v| {
        matches!(
            v.period_type,
            PeriodType::Ltm | PeriodType::Instant | PeriodType::Month
        )
    }) {
        let replace = latest.get(&v.item).is_none_or(|old| {
            // LTM antes de tudo; senão o mais recente.
            (v.period_type == PeriodType::Ltm && old.period_type != PeriodType::Ltm)
                || (v.period_type == old.period_type && v.period_end > old.period_end)
        });
        if replace {
            latest.insert(v.item.clone(), v.clone());
        }
    }
    let last_fy = all
        .iter()
        .filter(|v| v.period_type == PeriodType::Annual)
        .map(|v| v.period_end.clone())
        .max();
    let annual: Vec<FundamentalValue> = all
        .iter()
        .filter(|v| {
            (v.period_type == PeriodType::Annual || v.period_type == PeriodType::Instant)
                && Some(&v.period_end) == last_fy.as_ref()
        })
        .cloned()
        .collect();
    Ok(FundamentalsView {
        reference_date: periods::reference_date(&d.facts).map(fmt_date),
        subject: d.subject,
        issuer,
        securities,
        latest: latest.into_values().collect(),
        annual,
        last_sync_at: crate::settings::get(conn, super::config::LAST_SYNC_SETTING)?,
    })
}

// ---------------------------------------------------------------------------
// Histórico

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct HistoryArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
    /// Exercícios (padrão 10, máximo 15).
    #[serde(default)]
    #[ts(optional)]
    pub years: Option<u32>,
}

/// Um exercício. Dinheiro em centavos (menor unidade de `currency`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct YearRow {
    pub year: i32,
    pub period_end: String,
    #[ts(type = "number | null")]
    pub revenue: Option<i64>,
    #[ts(type = "number | null")]
    pub net_income: Option<i64>,
    #[ts(type = "number | null")]
    pub ebitda: Option<i64>,
    #[ts(type = "number | null")]
    pub dividends_declared: Option<i64>,
    #[ts(type = "number | null")]
    pub dividends_paid: Option<i64>,
    #[ts(type = "number | null")]
    pub equity: Option<i64>,
    /// Lucro por ação/papel (decimal texto).
    pub eps: Option<String>,
    /// Dividendos por ação/papel no ano (decimal texto).
    pub dps: Option<String>,
    pub net_margin: Option<f64>,
    pub roe: Option<f64>,
    /// Fechamento do último pregão do ano (ativos cadastrados com cotação).
    pub price: Option<String>,
    pub dividend_yield: Option<f64>,
    pub document: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuarterRow {
    /// "2T26".
    pub label: String,
    pub period_start: String,
    pub period_end: String,
    #[ts(type = "number | null")]
    pub revenue: Option<i64>,
    #[ts(type = "number | null")]
    pub net_income: Option<i64>,
    #[ts(type = "number | null")]
    pub ebitda: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthRow {
    /// `YYYY-MM`.
    pub month: String,
    pub nav_per_share: Option<String>,
    pub income_per_share: Option<String>,
    pub dy_month: Option<f64>,
    #[ts(type = "number | null")]
    pub holders: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HistoryView {
    pub subject: Subject,
    pub currency: String,
    pub years: Vec<YearRow>,
    /// Últimos 8 trimestres isolados (diferenças entre acumulados).
    pub quarters: Vec<QuarterRow>,
    /// FIIs: informes mensais.
    pub months: Vec<MonthRow>,
    /// Notas para quem lê (no idioma da instalação).
    pub notes: Vec<String>,
    /// Unidade dos valores ("valores monetários em centavos…"), para o agente e o MCP; a tela
    /// não mostra.
    pub units_note: String,
}

pub fn history(conn: &Connection, a: &HistoryArgs, today: NaiveDate) -> Result<HistoryView> {
    let r = SubjectRef {
        asset_id: a.asset_id.clone(),
        ticker: a.ticker.clone(),
    };
    let years = a.years.unwrap_or(10).clamp(1, 15) as usize;
    let d = data::load(conn, &r, today, &mut Cache::default())?;
    let cur = d.subject.currency.clone();
    let cents = |v: Decimal| to_minor(v, &cur);
    let unit = d.per_unit.unwrap_or(Decimal::ONE);
    let series: HashMap<&str, BTreeMap<NaiveDate, super::Fact>> = [
        items::REVENUE,
        items::NET_INCOME,
        items::EBIT,
        items::DA,
        items::DIVIDENDS_DECLARED,
        items::DIVIDENDS_PAID,
        items::DPS_DECLARED,
    ]
    .into_iter()
    .map(|i| (i, periods::annual(&d.facts, i)))
    .collect();
    let mut ends: Vec<NaiveDate> = series.values().flat_map(|s| s.keys().copied()).collect();
    ends.sort();
    ends.dedup();
    let ends: Vec<NaiveDate> = ends.into_iter().rev().take(years).rev().collect();
    // Proventos por ano civil da agenda do papel.
    let mut ev_year: BTreeMap<i32, Decimal> = BTreeMap::new();
    for x in &d.dividends {
        *ev_year.entry(x.date.year()).or_default() += x.amount;
    }
    let reference_shares = indicators::reference_shares(&d);
    let mut rows = vec![];
    for end in ends {
        let g = |i: &str| series.get(i).and_then(|s| s.get(&end)).map(|f| f.value);
        let shares =
            periods::instant_at(&d.facts, items::SHARES, end + chrono::Duration::days(100))
                .filter(|s| !s.value.is_zero() && (s.end - end).num_days().abs() <= 100)
                .map(|s| match reference_shares {
                    Some(r) => indicators::consistent_shares(s.value, r),
                    None => s.value,
                });
        let equity = periods::instant_at(&d.facts, items::EQUITY, end)
            .filter(|f| periods::near(f.end, end))
            .map(|f| f.value);
        let ni = g(items::NET_INCOME);
        let rev = g(items::REVENUE);
        let dps = ev_year
            .get(&end.year())
            .copied()
            .or_else(|| g(items::DPS_DECLARED).map(|v| v * unit))
            .or_else(|| match (g(items::DIVIDENDS_PAID), shares) {
                (Some(p), Some(s)) => Some(p / s * unit),
                _ => None,
            });
        let price = match &d.subject.asset_id {
            Some(id) => crate::market::close_on(conn, id, end)?
                .filter(|(_, p)| parse_date(&p.date).is_ok_and(|pd| (end - pd).num_days() <= 10))
                .map(|(v, _)| v),
            None => None,
        };
        rows.push(YearRow {
            year: end.year(),
            period_end: fmt_date(end),
            revenue: rev.map(cents),
            net_income: ni.map(cents),
            ebitda: match (g(items::EBIT), g(items::DA)) {
                (Some(e), Some(a)) => Some(cents(e + a)),
                _ => None,
            },
            dividends_declared: g(items::DIVIDENDS_DECLARED).map(cents),
            dividends_paid: g(items::DIVIDENDS_PAID).map(cents),
            equity: equity.map(cents),
            eps: match (ni, shares) {
                (Some(n), Some(s)) => Some(dstr((n / s * unit).round_dp(4))),
                _ => None,
            },
            dps: dps.map(|v| dstr(v.round_dp(4))),
            net_margin: match (ni, rev) {
                (Some(n), Some(r)) if !r.is_zero() => Some(to_f64(n / r)),
                _ => None,
            },
            roe: match (ni, equity) {
                (Some(n), Some(e)) if e > Decimal::ZERO => Some(to_f64(n / e)),
                _ => None,
            },
            dividend_yield: match (dps, price) {
                (Some(dv), Some(p)) if !p.is_zero() => Some(to_f64(dv / p)),
                _ => None,
            },
            price: price.map(dstr),
            document: series
                .get(items::NET_INCOME)
                .and_then(|s| s.get(&end))
                .or_else(|| series.get(items::REVENUE).and_then(|s| s.get(&end)))
                .map(|f| f.document.clone()),
        });
    }
    let q_rev = periods::quarterly(&d.facts, items::REVENUE);
    let q_ni = periods::quarterly(&d.facts, items::NET_INCOME);
    let q_ebit = periods::quarterly(&d.facts, items::EBIT);
    let q_da = periods::quarterly(&d.facts, items::DA);
    let mut q_ends: Vec<(NaiveDate, NaiveDate, String)> = q_rev
        .iter()
        .chain(q_ni.iter())
        .map(|q| (q.end, q.start, q.label.clone()))
        .collect();
    q_ends.sort();
    q_ends.dedup_by_key(|q| q.0);
    let find = |v: &[periods::QuarterValue], end: NaiveDate| {
        v.iter().find(|q| q.end == end).map(|q| q.value)
    };
    let quarters = q_ends
        .into_iter()
        .rev()
        .take(8)
        .rev()
        .map(|(end, start, label)| QuarterRow {
            label,
            period_start: fmt_date(start),
            period_end: fmt_date(end),
            revenue: find(&q_rev, end).map(cents),
            net_income: find(&q_ni, end).map(cents),
            ebitda: match (find(&q_ebit, end), find(&q_da, end)) {
                (Some(e), Some(a)) => Some(cents(e + a)),
                _ => None,
            },
        })
        .collect();
    let mut months: BTreeMap<String, MonthRow> = BTreeMap::new();
    for f in d
        .facts
        .iter()
        .filter(|f| f.period_type == PeriodType::Month)
    {
        let m = f.end.format("%Y-%m").to_string();
        let row = months.entry(m.clone()).or_insert(MonthRow {
            month: m,
            nav_per_share: None,
            income_per_share: None,
            dy_month: None,
            holders: None,
        });
        match f.item.as_str() {
            items::NAV_PER_SHARE => row.nav_per_share = Some(dstr(f.value.round_dp(4))),
            items::INCOME_PER_SHARE => row.income_per_share = Some(dstr(f.value.round_dp(4))),
            items::DY_MONTH => row.dy_month = Some(to_f64(f.value)),
            items::HOLDERS => row.holders = rust_decimal::prelude::ToPrimitive::to_i64(&f.value),
            _ => {}
        }
    }
    let mut notes = vec![];
    if d.facts.is_empty() {
        notes.push(t("nucleo-pesquisa-hist-sem-demonstracoes"));
    }
    if unit > Decimal::ONE {
        notes.push(t_args(
            "nucleo-pesquisa-hist-unit",
            &args(&[("acoes", dstr(unit).into())]),
        ));
    }
    Ok(HistoryView {
        subject: d.subject,
        currency: cur,
        years: rows,
        quarters,
        months: months.into_values().rev().take(24).rev().collect(),
        notes,
        units_note: t("nucleo-pesquisa-hist-centavos"),
    })
}

// ---------------------------------------------------------------------------
// Comparador

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct CompareAssetsArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_ids: Option<Vec<String>>,
    /// Papéis do universo (sem cadastro).
    #[serde(default)]
    #[ts(optional)]
    pub tickers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompareRow {
    pub key: String,
    pub label: String,
    pub group: IndicatorGroup,
    pub unit: IndicatorUnit,
    /// Um valor por ativo, na ordem de `subjects`.
    pub values: Vec<Option<f64>>,
    /// Índice do melhor valor (pela ordem natural do indicador).
    pub best: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SeriesPoint {
    pub date: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompareSeries {
    pub ticker: Option<String>,
    /// Cotação normalizada (primeiro dia = 100), últimos 12 meses.
    pub points: Vec<SeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompareView {
    pub as_of: String,
    pub subjects: Vec<Subject>,
    pub rows: Vec<CompareRow>,
    pub price_index: Vec<CompareSeries>,
}

pub const MAX_COMPARE: usize = 6;

pub fn compare(
    conn: &Connection,
    a: &CompareAssetsArgs,
    today: NaiveDate,
    bazin: BazinParams,
) -> Result<CompareView> {
    let mut refs: Vec<SubjectRef> = a
        .asset_ids
        .iter()
        .flatten()
        .map(|i| SubjectRef::asset(i))
        .collect();
    refs.extend(a.tickers.iter().flatten().map(|t| SubjectRef::ticker(t)));
    if refs.len() < 2 {
        return Err(Error::invalid(t("nucleo-pesquisa-erro-comparar-minimo")));
    }
    if refs.len() > MAX_COMPARE {
        return Err(Error::invalid(t_args(
            "nucleo-pesquisa-erro-comparar-maximo",
            &args(&[("max", (MAX_COMPARE as i64).into())]),
        )));
    }
    let mut cache = Cache::default();
    let mut views = vec![];
    for r in &refs {
        let d = data::load(conn, r, today, &mut cache)?;
        views.push(indicators::compute(&d, bazin));
    }
    let mut keys: Vec<&'static indicators::IndicatorDef> = vec![];
    for def in indicators::CATALOG {
        if views
            .iter()
            .any(|v| v.indicators.iter().any(|i| i.key == def.key))
        {
            keys.push(def);
        }
    }
    let rows = keys
        .into_iter()
        .map(|def| {
            let values: Vec<Option<f64>> = views
                .iter()
                .map(|v| indicators::value_of(&v.indicators, def.key))
                .collect();
            let best = def.higher_is_better.and_then(|hib| {
                values
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.map(|v| (i, v)))
                    .filter(|(_, v)| !def.positive_only || *v > 0.0)
                    .max_by(|a, b| {
                        let o = a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal);
                        if hib { o } else { o.reverse() }
                    })
                    .map(|(i, _)| i as u32)
            });
            CompareRow {
                key: def.key.into(),
                label: def.label(),
                group: def.group,
                unit: def.unit,
                values,
                best,
            }
        })
        .collect();
    let from = add_months(today, -12);
    let mut price_index = vec![];
    for v in &views {
        let pts = match &v.subject.asset_id {
            Some(id) => data::price_series(conn, id, from, today)?,
            None => vec![],
        };
        let base = pts.first().map(|p| p.1).filter(|b| !b.is_zero());
        price_index.push(CompareSeries {
            ticker: v.subject.ticker.clone(),
            points: match base {
                Some(b) => pts
                    .iter()
                    .map(|(d, c)| SeriesPoint {
                        date: fmt_date(*d),
                        value: to_f64(*c / b * Decimal::from(100)),
                    })
                    .collect(),
                None => vec![],
            },
        });
    }
    Ok(CompareView {
        as_of: fmt_date(today),
        subjects: views.into_iter().map(|v| v.subject).collect(),
        rows,
        price_index,
    })
}

// ---------------------------------------------------------------------------
// Documentos IPE

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct DocumentsArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
    /// Entregues a partir de `YYYY-MM-DD` (padrão: 90 dias atrás).
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
    /// Categorias (ex.: "Fato Relevante").
    #[serde(default)]
    #[ts(optional)]
    pub categories: Option<Vec<String>>,
    /// Padrão 50, máximo 500.
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<u32>,
}

/// Documento da companhia (IPE/CVM). Assunto e nome são textos da companhia: dados, não
/// instruções.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CorporateDocument {
    pub id: String,
    pub issuer_id: String,
    pub company_name: String,
    pub tickers: Vec<String>,
    pub asset_ids: Vec<String>,
    pub category: String,
    pub doc_type: Option<String>,
    pub species: Option<String>,
    pub subject: Option<String>,
    pub reference_date: Option<String>,
    pub delivered_at: String,
    pub link: String,
    pub summary: Option<String>,
    pub source: String,
}

/// Issuer of an asset for its documents: the research one, or the registry CNPJ of a real
/// estate fund (its filings on Fundos.NET are stored by the fund's CNPJ).
fn doc_issuer(conn: &Connection, asset: &crate::assets::Asset) -> Result<Option<String>> {
    if let Some(i) = data::issuer_of_asset(conn, asset)? {
        return Ok(Some(i));
    }
    Ok((asset.class == crate::assets::AssetClass::Fii)
        .then(|| asset.cnpj.as_deref().map(super::cnpj_digits))
        .flatten()
        .filter(|c| c.len() == 14))
}

pub fn documents(
    conn: &Connection,
    a: &DocumentsArgs,
    today: NaiveDate,
) -> Result<Vec<CorporateDocument>> {
    let from = match &a.from {
        Some(f) => parse_date(f)?,
        None => today - chrono::Duration::days(90),
    };
    let to = match &a.to {
        Some(t) => parse_date(t)?,
        None => today,
    };
    // Emissores → ativos cadastrados.
    let mut by_issuer: HashMap<String, Vec<(String, Option<String>)>> = HashMap::new();
    for asset in crate::assets::list(conn, &Default::default())? {
        if let Some(i) = doc_issuer(conn, &asset)? {
            by_issuer
                .entry(i)
                .or_default()
                .push((asset.id.clone(), asset.ticker.clone()));
        }
    }
    let wanted: HashSet<String> = if a.asset_id.is_some() || a.ticker.is_some() {
        let d = data::load(
            conn,
            &SubjectRef {
                asset_id: a.asset_id.clone(),
                ticker: a.ticker.clone(),
            },
            today,
            &mut Cache::default(),
        )?;
        let registered = match &d.subject.asset_id {
            Some(id) => doc_issuer(conn, &crate::assets::get(conn, id)?)?,
            None => None,
        };
        d.subject.issuer_id.or(registered).into_iter().collect()
    } else {
        let mine: HashSet<String> = data::held_or_watched(conn)?.into_iter().collect();
        by_issuer
            .iter()
            .filter(|(_, v)| v.iter().any(|(id, _)| mine.contains(id)))
            .map(|(i, _)| i.clone())
            .collect()
    };
    let cats: Vec<String> = a
        .categories
        .iter()
        .flatten()
        .map(|c| super::norm(c))
        .collect();
    let limit = a.limit.unwrap_or(50).clamp(1, 500) as usize;
    let mut stmt = conn.prepare(
        "SELECT id, issuer_id, company_name, category, doc_type, species, subject, reference_date, delivered_at, link, summary, source
         FROM corporate_document WHERE delivered_at BETWEEN ?1 AND ?2 ORDER BY delivered_at DESC, id",
    )?;
    let rows = stmt.query_map(params![fmt_date(from), fmt_date(to)], |r| {
        Ok(CorporateDocument {
            id: r.get(0)?,
            issuer_id: r.get(1)?,
            company_name: r.get(2)?,
            tickers: vec![],
            asset_ids: vec![],
            category: r.get(3)?,
            doc_type: r.get(4)?,
            species: r.get(5)?,
            subject: r.get(6)?,
            reference_date: r.get(7)?,
            delivered_at: r.get(8)?,
            link: r.get(9)?,
            summary: r.get(10)?,
            source: r.get(11)?,
        })
    })?;
    let mut out = vec![];
    for d in rows {
        let mut d = d?;
        if !wanted.contains(&d.issuer_id) {
            continue;
        }
        if !cats.is_empty() && !cats.contains(&super::norm(&d.category)) {
            continue;
        }
        if let Some(v) = by_issuer.get(&d.issuer_id) {
            d.asset_ids = v.iter().map(|x| x.0.clone()).collect();
            d.tickers = v.iter().filter_map(|x| x.1.clone()).collect();
        }
        out.push(d);
        if out.len() >= limit {
            break;
        }
    }
    Ok(out)
}

/// Por que um ativo tem (ou não) documentos, para a aba Documentos e o agente
/// (docs/design/market-explorer.md, seção 3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DocumentsReason {
    /// Há documentos guardados.
    Ok,
    /// Classe sem fonte de documentos ainda (ETFs e fundos; os FIIs vêm do Fundos.NET).
    NotCovered,
    /// Companhia aberta sem emissor encontrado (sem CNPJ no cadastro e ticker fora do FCA).
    NoIssuer,
    /// A coleta da CVM ainda não rodou.
    NotCollected,
    /// Emissor encontrado, coleta feita, nenhum documento das categorias guardadas.
    Quiet,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentsCategoryCount {
    pub category: String,
    #[ts(type = "number")]
    pub count: u64,
    pub latest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentsStatus {
    pub reason: DocumentsReason,
    /// Ativo cadastrado (para a ação de informar o CNPJ).
    pub asset_id: Option<String>,
    pub class: Option<crate::assets::AssetClass>,
    pub issuer_id: Option<String>,
    pub issuer_name: Option<String>,
    /// Fonte consultada (`cvm_ipe`; FIIs: `fundos_net`).
    pub source: String,
    /// Categorias guardadas pela coleta (as demais do IPE são descartadas).
    pub kept_categories: Vec<String>,
    /// Desde quando a coleta guarda documentos (1º de janeiro do ano anterior).
    pub collected_since: String,
    pub last_sync_at: Option<String>,
    #[ts(type = "number")]
    pub stored: u64,
    pub latest: Option<String>,
    pub by_category: Vec<DocumentsCategoryCount>,
}

/// Diagnóstico dos documentos de um ativo: fonte, emissor, coleta e o que está guardado.
pub fn documents_status(
    conn: &Connection,
    a: &DocumentsArgs,
    today: NaiveDate,
) -> Result<DocumentsStatus> {
    use crate::assets::AssetClass as C;
    let asset = match (&a.asset_id, &a.ticker) {
        (Some(id), _) if !id.trim().is_empty() => Some(crate::assets::get(conn, id)?),
        (_, Some(t)) if !t.trim().is_empty() => crate::assets::find_by_ticker(conn, t)?,
        _ => None,
    };
    // Papel fora do universo e do cadastro: sem emissor, como um ticker que o FCA não lista.
    let subject = data::load(
        conn,
        &SubjectRef {
            asset_id: a.asset_id.clone(),
            ticker: a.ticker.clone(),
        },
        today,
        &mut Cache::default(),
    )
    .ok()
    .map(|d| d.subject);
    let class = asset
        .as_ref()
        .map(|x| x.class)
        .or(subject.as_ref().map(|s| s.class));
    let issuer_id = match subject.as_ref().and_then(|s| s.issuer_id.clone()) {
        Some(i) => Some(i),
        None => match &asset {
            Some(a) => doc_issuer(conn, a)?,
            None => None,
        },
    };
    let issuer_name = subject.as_ref().and_then(|s| s.issuer_name.clone());
    // Real estate funds: Fundos.NET (management reports, material facts, announcements).
    let fund = matches!(class, Some(C::Fii));
    let covered = fund || matches!(class, Some(C::StockBr | C::Bdr) | None);
    let mut by_category = vec![];
    if let Some(i) = &issuer_id {
        let mut stmt = conn.prepare(
            "SELECT category, count(*), max(delivered_at) FROM corporate_document
              WHERE issuer_id = ?1 GROUP BY category ORDER BY max(delivered_at) DESC",
        )?;
        by_category = stmt
            .query_map([i], |r| {
                Ok(DocumentsCategoryCount {
                    category: r.get(0)?,
                    count: r.get::<_, i64>(1)? as u64,
                    latest: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
    }
    let stored: u64 = by_category.iter().map(|c| c.count).sum();
    let latest = by_category.iter().map(|c| c.latest.clone()).max();
    let last_sync_at: Option<String> = if fund {
        // The fund's last list on Fundos.NET.
        issuer_id.as_ref().and_then(|i| {
            conn.query_row(
                "SELECT fetched_at FROM research_fetch_log WHERE source = ?1 AND key = ?2 AND status = 'ok'",
                params![super::fundos_net::SOURCE, format!("docs:{i}")],
                |r| r.get(0),
            )
            .ok()
        })
    } else {
        crate::settings::get(conn, super::config::LAST_SYNC_SETTING)?
    };
    let reason = if !covered {
        DocumentsReason::NotCovered
    } else if issuer_id.is_none() {
        DocumentsReason::NoIssuer
    } else if stored > 0 {
        DocumentsReason::Ok
    } else if last_sync_at.is_none() {
        DocumentsReason::NotCollected
    } else {
        DocumentsReason::Quiet
    };
    Ok(DocumentsStatus {
        reason,
        asset_id: asset.map(|x| x.id),
        class,
        issuer_id,
        issuer_name,
        source: if fund {
            super::fundos_net::DOC_SOURCE
        } else {
            "cvm_ipe"
        }
        .into(),
        kept_categories: if fund {
            super::fundos_net::DOC_CATEGORIES
                .iter()
                .map(|(_, c, _)| c.to_string())
                .collect()
        } else {
            cvm::IPE_CATEGORIES.iter().map(|c| c.to_string()).collect()
        },
        collected_since: format!("{}-01-01", today.year() - 1),
        last_sync_at,
        stored,
        latest,
        by_category,
    })
}

// ---------------------------------------------------------------------------
// Estado

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResearchStatus {
    pub last_sync_at: Option<String>,
    #[ts(type = "number")]
    pub issuers: u64,
    #[ts(type = "number")]
    pub securities: u64,
    #[ts(type = "number")]
    pub facts: u64,
    #[ts(type = "number")]
    pub documents: u64,
    pub quotes_last_date: Option<String>,
    pub universe_as_of: Option<String>,
    #[ts(type = "number")]
    pub universe_size: u64,
    pub last_report: Option<super::sync::ResearchSyncReport>,
}

pub fn status(conn: &Connection) -> Result<ResearchStatus> {
    let count =
        |sql: &str| -> Result<u64> { Ok(conn.query_row(sql, [], |r| r.get::<_, i64>(0))? as u64) };
    Ok(ResearchStatus {
        last_sync_at: crate::settings::get(conn, super::config::LAST_SYNC_SETTING)?,
        issuers: count("SELECT count(*) FROM research_issuer")?,
        securities: count("SELECT count(*) FROM research_security")?,
        facts: count("SELECT count(*) FROM fundamental")?,
        documents: count("SELECT count(*) FROM corporate_document")?,
        quotes_last_date: conn
            .query_row("SELECT max(date) FROM research_quote", [], |r| r.get(0))?,
        universe_as_of: conn
            .query_row("SELECT max(as_of) FROM research_universe", [], |r| r.get(0))?,
        universe_size: count("SELECT count(*) FROM research_universe")?,
        last_report: crate::settings::get(conn, super::config::LAST_REPORT_SETTING)
            .ok()
            .flatten(),
    })
}
