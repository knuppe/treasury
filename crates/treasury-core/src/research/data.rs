//! Carrega do banco o que os indicadores precisam: ativo (cadastrado ou só no universo
//! baixado), emissor, fatos, cotação (`market::close_on`, senão B3 COTAHIST), cotações das
//! outras classes, proventos ([`super::dividends::select`]: agenda `dividend_event`, B3
//! oficial `research_dividend`, lançamentos em `income_event`) e liquidez.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde::Deserialize;

use super::indicators::{Data, Sibling, Subject};
use super::{Fact, Issuer, IssuerKind, Security, cnpj_digits, dstr, format_cnpj, sec, store};
use crate::assets::{Asset, AssetClass};
use crate::error::{Error, Result};
use crate::market::PriceOn;
use crate::time::{fmt_date, parse_date};

/// Ativo cadastrado (`asset_id`) ou papel do universo baixado (`ticker`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SubjectRef {
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub ticker: Option<String>,
}

impl SubjectRef {
    pub fn asset(id: &str) -> Self {
        Self {
            asset_id: Some(id.to_string()),
            ticker: None,
        }
    }
    pub fn ticker(t: &str) -> Self {
        Self {
            asset_id: None,
            ticker: Some(t.to_string()),
        }
    }
}

/// Fatos já carregados por emissor (o universo tem várias classes do mesmo emissor).
#[derive(Default)]
pub struct Cache {
    facts: HashMap<String, Vec<Fact>>,
}

impl Cache {
    pub fn with_facts(facts: HashMap<String, Vec<Fact>>) -> Self {
        Self { facts }
    }
}

fn is_us(class: AssetClass) -> bool {
    class.is_us()
}

/// Código de negociação do ativo no universo (ticker da SEC para ativos dos EUA).
pub fn security_of(
    conn: &Connection,
    ticker: &str,
    class: Option<AssetClass>,
) -> Result<Option<Security>> {
    let t = ticker.trim().to_uppercase();
    if class.is_some_and(is_us) {
        return Ok(
            store::security(conn, &sec::normalize_ticker(&t))?.filter(|s| s.class == "stock_us")
        );
    }
    Ok(store::security(conn, &t)?.filter(|s| class.is_none() || s.class != "stock_us"))
}

/// Emissor de um ativo cadastrado (CNPJ do fundo, ou pelo código de negociação).
pub fn issuer_of_asset(conn: &Connection, a: &Asset) -> Result<Option<String>> {
    if let Some(c) = a.cnpj.as_deref().map(cnpj_digits).filter(|c| c.len() == 14)
        && store::issuer(conn, &c)?.is_some()
    {
        return Ok(Some(c));
    }
    let Some(t) = a.ticker.as_deref() else {
        return Ok(None);
    };
    Ok(security_of(conn, t, Some(a.class))?.map(|s| s.issuer_id))
}

/// Ativos com operações ou na watchlist.
pub fn held_or_watched(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT a.id FROM asset a WHERE a.archived = 0 AND (
            EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id) OR EXISTS (SELECT 1 FROM watchlist_item w WHERE w.asset_id = a.id))
         ORDER BY a.ticker",
    )?;
    let ids = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(ids)
}

/// Emissores dos ativos cadastrados (não arquivados).
pub fn registered_issuers(conn: &Connection) -> Result<HashSet<String>> {
    let mut out = HashSet::new();
    for a in crate::assets::list(conn, &Default::default())? {
        if let Some(i) = issuer_of_asset(conn, &a)? {
            out.insert(i);
        }
    }
    Ok(out)
}

fn price_of_asset(
    conn: &Connection,
    id: &str,
    today: NaiveDate,
) -> Result<Option<(Decimal, PriceOn)>> {
    crate::market::close_on(conn, id, today)
}

fn price_of_ticker(
    conn: &Connection,
    ticker: &str,
    today: NaiveDate,
) -> Result<Option<(Decimal, PriceOn)>> {
    if let Some(a) = crate::assets::find_by_ticker(conn, ticker)?
        && let Some(p) = price_of_asset(conn, &a.id, today)?
    {
        return Ok(Some(p));
    }
    Ok(store::latest_quote(conn, ticker, today)?.map(|q| {
        (
            q.close,
            PriceOn {
                close: dstr(q.close),
                date: fmt_date(q.date),
                source: q.source,
                stale: q.date != today,
                intraday: None,
            },
        )
    }))
}

fn liquidity_of(
    conn: &Connection,
    ticker: Option<&str>,
    asset_id: Option<&str>,
) -> Result<Option<(Decimal, String)>> {
    if let Some(t) = ticker
        && let Some((avg, n)) = store::avg_quote_volume(conn, t, 21)?
    {
        return Ok(Some((
            avg,
            crate::i18n::t_args(
                "nucleo-pesquisa-liquidez-cotahist",
                &crate::i18n::args(&[("n", i64::from(n).into())]),
            ),
        )));
    }
    let Some(id) = asset_id else {
        return Ok(None);
    };
    let mut stmt = conn.prepare_cached(
        "SELECT close, volume, source FROM price WHERE asset_id = ?1 AND volume IS NOT NULL ORDER BY date DESC LIMIT 21",
    )?;
    let rows: Vec<(String, String, String)> = stmt
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let vals: Vec<Decimal> = rows
        .iter()
        .filter_map(|(c, v, _)| Some(c.parse::<Decimal>().ok()? * v.parse::<Decimal>().ok()?))
        .collect();
    if vals.is_empty() {
        return Ok(None);
    }
    let avg = vals.iter().copied().sum::<Decimal>() / Decimal::from(vals.len());
    Ok(Some((
        avg,
        crate::i18n::t_args(
            "nucleo-pesquisa-liquidez-cotacoes",
            &crate::i18n::args(&[
                ("n", (vals.len() as i64).into()),
                ("fonte", rows[0].2.clone().into()),
            ]),
        ),
    )))
}

fn class_of_security(s: &Security) -> AssetClass {
    match s.class.as_str() {
        "fii" => AssetClass::Fii,
        "stock_us" => AssetClass::StockUs,
        "bdr" => AssetClass::Bdr,
        _ => AssetClass::StockBr,
    }
}

pub fn subject_of(
    asset: Option<&Asset>,
    security: Option<&Security>,
    issuer: Option<&Issuer>,
    ticker: Option<String>,
) -> Subject {
    let class = asset
        .map(|a| a.class)
        .or_else(|| security.map(class_of_security))
        .unwrap_or(AssetClass::StockBr);
    Subject {
        asset_id: asset.map(|a| a.id.clone()),
        ticker: ticker.clone(),
        name: asset
            .map(|a| a.name.clone())
            .or_else(|| issuer.map(|i| i.name.clone()))
            .or(ticker)
            .unwrap_or_default(),
        class,
        currency: asset
            .map(|a| a.currency.clone())
            .or_else(|| issuer.map(|i| i.currency.clone()))
            // Sem cadastro: papel da B3, como a classe padrão acima.
            .unwrap_or_else(|| "BRL".into()),
        issuer_id: issuer.map(|i| i.id.clone()),
        issuer_name: issuer.map(|i| i.name.clone()),
        issuer_code: issuer.map(|i| {
            if i.id.starts_with("cik:") {
                i.id.clone()
            } else {
                format_cnpj(&i.id)
            }
        }),
        sector: asset
            .and_then(|a| a.sector.clone())
            .or_else(|| issuer.and_then(|i| i.sector.clone())),
        segment: asset
            .and_then(|a| a.segment.clone())
            .or_else(|| issuer.and_then(|i| i.segment.clone())),
        mandate: issuer.and_then(|i| i.mandate.clone()),
        layout: issuer.map(|i| i.layout),
        share_kind: security.and_then(|s| s.share_kind.clone()),
    }
}

/// Dados de um ativo para os indicadores.
pub fn load(
    conn: &Connection,
    r: &SubjectRef,
    today: NaiveDate,
    cache: &mut Cache,
) -> Result<Data> {
    let asset = match (&r.asset_id, &r.ticker) {
        (Some(id), _) if !id.trim().is_empty() => Some(crate::assets::get(conn, id)?),
        (_, Some(t)) if !t.trim().is_empty() => crate::assets::find_by_ticker(conn, t)?,
        _ => {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-pesquisa-erro-asset-ou-ticker",
            )));
        }
    };
    let ticker = asset
        .as_ref()
        .and_then(|a| a.ticker.clone())
        .or_else(|| r.ticker.as_ref().map(|t| t.trim().to_uppercase()));
    let security = match &ticker {
        Some(t) => security_of(conn, t, asset.as_ref().map(|a| a.class))?,
        None => None,
    };
    let issuer_id = match &asset {
        Some(a) => issuer_of_asset(conn, a)?,
        None => security.as_ref().map(|s| s.issuer_id.clone()),
    };
    let issuer = match &issuer_id {
        Some(i) => store::issuer(conn, i)?,
        None => None,
    };
    if asset.is_none() && security.is_none() {
        return Err(Error::not_found(crate::i18n::t_args(
            "nucleo-pesquisa-erro-fora-do-universo",
            &crate::i18n::args(&[("ticker", ticker.unwrap_or_default().into())]),
        )));
    }
    let subject = subject_of(
        asset.as_ref(),
        security.as_ref(),
        issuer.as_ref(),
        ticker.clone(),
    );
    let facts = match &issuer {
        Some(i) => match cache.facts.get(&i.id) {
            Some(f) => f.clone(),
            None => {
                let f = store::load_facts(conn, &i.id)?;
                cache.facts.insert(i.id.clone(), f.clone());
                f
            }
        },
        None => vec![],
    };
    let price = match (&asset, &ticker) {
        (Some(a), _) => match price_of_asset(conn, &a.id, today)? {
            Some(p) => Some(p),
            None => match &ticker {
                Some(t) if !is_us(a.class) => price_of_ticker(conn, t, today)?,
                _ => None,
            },
        },
        (None, Some(t)) => price_of_ticker(conn, t, today)?,
        (None, None) => None,
    };
    let per_unit = security
        .as_ref()
        .map(|s| s.shares_per_unit())
        .unwrap_or(Some(Decimal::ONE));
    let mut siblings = vec![];
    if let Some(i) = issuer.as_ref().filter(|i| i.kind == IssuerKind::CompanyBr) {
        for s in store::securities_of(conn, &i.id)? {
            let Some(kind) = s.share_kind.clone().filter(|k| k == "ON" || k == "PN") else {
                continue;
            };
            if !s.active || siblings.iter().any(|x: &Sibling| x.share_kind == kind) {
                continue;
            }
            let p = if Some(&s.ticker) == ticker.as_ref() {
                price.as_ref().map(|p| p.0)
            } else {
                price_of_ticker(conn, &s.ticker, today)?.map(|p| p.0)
            };
            if let Some(close) = p {
                siblings.push(Sibling {
                    ticker: s.ticker.clone(),
                    share_kind: kind,
                    close,
                });
            }
        }
    }
    // Ações e FIIs brasileiros têm os proventos oficiais da B3.
    let br = matches!(subject.class, AssetClass::StockBr | AssetClass::Fii);
    let (dividends, dividends_source) = super::dividends::select(
        conn,
        asset.as_ref().map(|a| a.id.as_str()),
        ticker.as_deref().filter(|_| br),
        today,
    )?;
    let liquidity = liquidity_of(
        conn,
        ticker.as_deref(),
        asset.as_ref().map(|a| a.id.as_str()),
    )?;
    Ok(Data {
        subject,
        facts,
        price,
        per_unit,
        siblings,
        dividends,
        dividends_source,
        liquidity,
        today,
    })
}

/// Último fechamento conhecido de um ativo cadastrado num dia (para gráficos normalizados).
pub fn price_series(
    conn: &Connection,
    asset_id: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<(NaiveDate, Decimal)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT date, close FROM price WHERE asset_id = ?1 AND date BETWEEN ?2 AND ?3 ORDER BY date",
    )?;
    let rows: Vec<(String, String)> = stmt
        .query_map(params![asset_id, fmt_date(from), fmt_date(to)], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(d, c)| Some((parse_date(&d).ok()?, c.parse().ok()?)))
        .collect())
}
