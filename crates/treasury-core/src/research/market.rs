//! Tela Mercado (docs/design/market-explorer.md, seção 7): o panorama por setor, o
//! mapa Saúde × Atratividade e o Observar, que põe um papel do universo na watchlist sem
//! lançar operação.
//!
//! Tudo vem do universo do dia (`research_universe` com o score em lote de
//! `market_score`): nada de rede aqui. Nada disso é recomendação de investimento: mostra quem
//! cumpre critérios públicos (CVM Res. 20/2021).

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use rusqlite::{Connection, Transaction};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::indicators::BazinParams;
use super::screen::{self, UniverseRow};
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::ids::Id;

/// Ativos criados pelo Observar (somem do cadastro quando deixam de ser observados sem nunca
/// ter tido operação).
pub const OBSERVED_SETTING: &str = "market.observed_assets";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MarketSector")]
pub struct SectorStats {
    pub sector: String,
    pub count: u32,
    /// Papéis com score em lote.
    pub scored: u32,
    /// Faixas Robusto e Saudável.
    pub healthy: u32,
    pub median_pl: Option<f64>,
    pub median_pvp: Option<f64>,
    pub median_dy: Option<f64>,
    pub median_score: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MarketOverview {
    pub as_of: String,
    pub stocks: u32,
    pub fiis: u32,
    pub scored: u32,
    /// Data do último score em lote.
    pub scores_as_of: Option<String>,
    pub last_sync_at: Option<String>,
    /// Ações por setor e FIIs por segmento, do setor com mais papéis ao com menos.
    pub stock_sectors: Vec<SectorStats>,
    pub fii_segments: Vec<SectorStats>,
}

/// Um papel no mapa Saúde × Atratividade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MarketMapPoint {
    pub ticker: String,
    pub name: String,
    pub class: AssetClass,
    pub sector: Option<String>,
    pub asset_id: Option<String>,
    #[ts(type = "number")]
    pub health: f64,
    #[ts(type = "number")]
    pub attractiveness: f64,
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    pub band: Option<String>,
    #[ts(type = "number | null")]
    pub market_cap: Option<f64>,
    pub held: bool,
    pub watched: bool,
}

fn median(mut v: Vec<f64>) -> Option<f64> {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    Some(if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    })
}

fn stats(sector: String, rows: &[&UniverseRow]) -> SectorStats {
    let m = |k: &str, positive: bool| {
        median(
            rows.iter()
                .filter_map(|r| r.metrics.get(k).copied())
                .filter(|v| !positive || *v > 0.0)
                .collect(),
        )
    };
    SectorStats {
        sector,
        count: rows.len() as u32,
        scored: rows
            .iter()
            .filter(|r| r.metrics.contains_key("score"))
            .count() as u32,
        healthy: rows
            .iter()
            .filter(|r| matches!(r.score_band.as_deref(), Some("robusto" | "saudavel")))
            .count() as u32,
        median_pl: m("pl", true),
        median_pvp: m("pvp", true),
        median_dy: m("dy", false),
        median_score: m("score", false),
    }
}

fn group_by(
    rows: &[&UniverseRow],
    key: impl Fn(&UniverseRow) -> Option<String>,
) -> Vec<SectorStats> {
    let mut by: BTreeMap<String, Vec<&UniverseRow>> = BTreeMap::new();
    for r in rows {
        by.entry(key(r).unwrap_or_else(|| "—".into()))
            .or_default()
            .push(r);
    }
    let mut out: Vec<SectorStats> = by.into_iter().map(|(k, v)| stats(k, &v)).collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then(a.sector.cmp(&b.sector)));
    out
}

/// Panorama do mercado: quantos papéis, quantos com score e as medianas por setor.
pub fn overview(conn: &Connection, today: NaiveDate, bazin: BazinParams) -> Result<MarketOverview> {
    let (rows, as_of) = screen::universe(conn, today, bazin)?;
    let stocks: Vec<&UniverseRow> = rows
        .iter()
        .filter(|r| r.class == AssetClass::StockBr)
        .collect();
    let fiis: Vec<&UniverseRow> = rows.iter().filter(|r| r.class == AssetClass::Fii).collect();
    Ok(MarketOverview {
        as_of,
        stocks: stocks.len() as u32,
        fiis: fiis.len() as u32,
        scored: rows
            .iter()
            .filter(|r| r.metrics.contains_key("score"))
            .count() as u32,
        scores_as_of: crate::score::market::last_as_of(conn)?,
        last_sync_at: crate::settings::get(conn, super::config::LAST_SYNC_SETTING)?,
        stock_sectors: group_by(&stocks, |r| r.sector.clone()),
        fii_segments: group_by(&fiis, |r| r.segment.clone()),
    })
}

/// Papéis com Saúde e Atratividade (o mapa), com a marca de carteira e watchlist.
pub fn map(conn: &Connection, today: NaiveDate, bazin: BazinParams) -> Result<Vec<MarketMapPoint>> {
    let (rows, _) = screen::universe(conn, today, bazin)?;
    let held: HashSet<String> = conn
        .prepare("SELECT DISTINCT asset_id FROM trade")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let watched: HashSet<String> = conn
        .prepare("SELECT asset_id FROM watchlist_item")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            let h = *r.metrics.get("health")?;
            let a = *r.metrics.get("attractiveness")?;
            Some(MarketMapPoint {
                held: r.asset_id.as_ref().is_some_and(|id| held.contains(id)),
                watched: r.asset_id.as_ref().is_some_and(|id| watched.contains(id)),
                ticker: r.ticker,
                name: r.name,
                class: r.class,
                sector: r.sector,
                asset_id: r.asset_id,
                health: h,
                attractiveness: a,
                score: r.metrics.get("score").copied(),
                band: r.score_band,
                market_cap: r.metrics.get("market_cap").copied(),
            })
        })
        .collect())
}

fn observed(conn: &Connection) -> Result<Vec<String>> {
    Ok(crate::settings::get::<Vec<String>>(conn, OBSERVED_SETTING)?.unwrap_or_default())
}

/// Observar um papel do universo: o ativo cadastrado (ou criado agora, com o nome, a classe e
/// o CNPJ do fundo vindos da CVM) entra na watchlist, sem operação. Devolve o id do ativo.
pub fn observe(tx: &Transaction<'_>, ticker: &str) -> Result<Id> {
    let ticker = ticker.trim().to_uppercase();
    if ticker.is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-asset-ou-ticker",
        )));
    }
    let id = match crate::assets::find_by_ticker(tx, &ticker)? {
        Some(a) => a.id,
        None => {
            let sec = super::data::security_of(tx, &ticker, None)?.ok_or_else(|| {
                Error::not_found(crate::i18n::t_args(
                    "nucleo-pesquisa-erro-fora-do-universo",
                    &crate::i18n::args(&[("ticker", ticker.clone().into())]),
                ))
            })?;
            let issuer = super::store::issuer(tx, &sec.issuer_id)?;
            let class = AssetClass::parse(&sec.class).unwrap_or(AssetClass::StockBr);
            let name = issuer
                .as_ref()
                .map(|i| i.name.split_whitespace().collect::<Vec<_>>().join(" "))
                .unwrap_or_else(|| ticker.clone());
            // FII: o CNPJ do fundo é o do emissor (cotas na CVM); ação: o emissor fica no FCA.
            let cnpj = (class == AssetClass::Fii).then(|| sec.issuer_id.clone());
            let id = crate::assets::create(
                tx,
                &crate::assets::AssetInput {
                    ticker: Some(ticker.clone()),
                    name,
                    official_name: None,
                    account_id: None,
                    class,
                    market: Some("B3".into()),
                    currency: None,
                    country: None,
                    isin: sec.isin.clone(),
                    cnpj,
                    treasury_code: None,
                    sector: issuer.as_ref().and_then(|i| i.sector.clone()),
                    segment: issuer.as_ref().and_then(|i| i.segment.clone()),
                    price_provider: None,
                    provider_symbol: None,
                    notes: None,
                    fixed_income: None,
                    pension: None,
                },
            )?;
            let mut list = observed(tx)?;
            list.push(id.clone());
            crate::settings::set(tx, OBSERVED_SETTING, &list)?;
            id
        }
    };
    crate::portfolio::records::upsert_watch(
        tx,
        &crate::portfolio::WatchlistInput {
            asset_id: id.clone(),
            target_price: None,
            alert_above: None,
            alert_below: None,
            notes: None,
        },
    )?;
    Ok(id)
}

/// Deixar de observar: sai da watchlist e, se o ativo foi criado pelo Observar e nunca teve
/// operação nem provento, sai também do cadastro. Devolve `true` quando o ativo foi apagado.
pub fn unobserve(tx: &Transaction<'_>, asset_id: &str) -> Result<bool> {
    tx.execute("DELETE FROM watchlist_item WHERE asset_id = ?1", [asset_id])?;
    let mut list = observed(tx)?;
    if !list.iter().any(|x| x == asset_id) {
        return Ok(false);
    }
    // Com operação ou provento lançado, o ativo virou da família: fica no cadastro.
    let removed = crate::assets::delete(tx, asset_id).is_ok();
    list.retain(|x| x != asset_id);
    crate::settings::set(tx, OBSERVED_SETTING, &list)?;
    Ok(removed)
}

/// O ativo foi criado pelo Observar (para a interface marcar "observado pelo Mercado").
pub fn is_observed(conn: &Connection, asset_id: &str) -> Result<bool> {
    Ok(observed(conn)?.iter().any(|x| x == asset_id))
}
