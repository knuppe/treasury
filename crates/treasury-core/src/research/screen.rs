//! Universo diário, rankings, screener e telas salvas.
//!
//! O universo são as ações e FIIs com dados baixados (FCA/informe de FII) mais os ativos
//! cadastrados de classes com fundamentos. Os indicadores de cada papel são recalculados uma
//! vez por dia (na coleta, ou na primeira consulta do dia) e gravados em `research_universe`,
//! para que rankings e screener funcionem offline e rápido.
//!
//! Filtros do screener: `campo op valor`, combinados com E/OU num nível de grupos
//! (`(A e B) ou (C e D)`). Percentuais em fração (0,06 = 6%).

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::data::{self, Cache, SubjectRef};
use super::indicators::{self, BazinParams, CATALOG, IndicatorInfo};
use super::norm;
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc};

/// Um papel do universo com os indicadores do dia.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UniverseRow {
    pub ticker: String,
    pub asset_id: Option<String>,
    pub issuer_id: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub sector: Option<String>,
    pub segment: Option<String>,
    pub currency: String,
    /// Indicador → valor (só os disponíveis). Inclui os campos do score em lote
    /// (`score`, `health`, `attractiveness`, `confidence`, `piotroski`…), quando houver.
    #[ts(type = "Record<string, number>")]
    pub metrics: BTreeMap<String, f64>,
    /// Faixa do score em lote (`robusto`, `saudavel`, `atencao`, `risco`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub score_band: Option<String>,
    /// Red flags do score em lote (chaves separadas por vírgula).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub red_flags: Option<String>,
}

/// Campos do score em lote (`market_score`) que entram nos indicadores do universo.
pub const SCORE_FIELDS: &[&str] = &[
    "score",
    "health",
    "attractiveness",
    "confidence",
    "piotroski",
    "altman_z",
    "beneish",
    "momentum_12_1",
    "volatility_1y",
];

/// Junta o score em lote de cada papel aos indicadores do dia.
fn attach_scores(conn: &Connection, rows: &mut [UniverseRow]) -> Result<()> {
    let scores = crate::score::market::load(conn)?;
    for u in rows.iter_mut() {
        let Some(m) = scores.get(&u.ticker) else {
            continue;
        };
        for (k, v) in [
            ("score", m.score),
            ("health", m.health),
            ("attractiveness", m.attractiveness),
            ("confidence", m.confidence),
            ("piotroski", m.piotroski),
            ("altman_z", m.altman_z),
            ("beneish", m.beneish),
            ("momentum_12_1", m.momentum_12_1),
            ("volatility_1y", m.volatility_1y),
        ] {
            if let Some(v) = v.filter(|v| v.is_finite()) {
                u.metrics.insert(k.to_string(), v);
            }
        }
        u.score_band = m.band.clone();
        u.red_flags = (!m.red_flags.is_empty()).then(|| m.red_flags.join(","));
    }
    Ok(())
}

const FUNDAMENTAL_CLASSES: &[AssetClass] = &[
    AssetClass::StockBr,
    AssetClass::Fii,
    AssetClass::Bdr,
    AssetClass::StockUs,
    AssetClass::Reit,
];

/// Calcula o universo (sem gravar).
pub fn compute_universe(
    conn: &Connection,
    today: NaiveDate,
    bazin: BazinParams,
) -> Result<Vec<UniverseRow>> {
    let mut refs: BTreeMap<String, SubjectRef> = BTreeMap::new();
    for a in crate::assets::list(conn, &Default::default())? {
        if let Some(t) = &a.ticker
            && FUNDAMENTAL_CLASSES.contains(&a.class)
        {
            refs.insert(t.to_uppercase(), SubjectRef::asset(&a.id));
        }
    }
    let mut stmt = conn.prepare(
        "SELECT s.ticker FROM research_security s WHERE s.active = 1 AND s.class IN ('stock_br', 'fii')
           AND EXISTS (SELECT 1 FROM fundamental f WHERE f.issuer_id = s.issuer_id)",
    )?;
    let tickers: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for t in tickers {
        refs.entry(t.clone())
            .or_insert_with(|| SubjectRef::ticker(&t));
    }
    let mut cache = Cache::with_facts(super::store::load_all_facts(conn)?);
    let mut out = vec![];
    for (ticker, r) in refs {
        let Ok(d) = data::load(conn, &r, today, &mut cache) else {
            continue;
        };
        let v = indicators::compute(&d, bazin);
        let metrics: BTreeMap<String, f64> = v
            .indicators
            .iter()
            .filter_map(|i| i.value.map(|x| (i.key.clone(), x)))
            .collect();
        out.push(UniverseRow {
            ticker,
            asset_id: v.subject.asset_id.clone(),
            issuer_id: v.subject.issuer_id.clone(),
            name: v.subject.name.clone(),
            class: v.subject.class,
            sector: v.subject.sector.clone(),
            segment: v.subject.segment.clone(),
            currency: v.subject.currency.clone(),
            metrics,
            score_band: None,
            red_flags: None,
        });
    }
    Ok(out)
}

/// Recalcula e grava o universo do dia.
pub fn recompute_universe(
    tx: &Transaction<'_>,
    today: NaiveDate,
    bazin: BazinParams,
) -> Result<usize> {
    let rows = compute_universe(tx, today, bazin)?;
    store_universe(tx, &rows, today)?;
    Ok(rows.len())
}

fn store_universe(conn: &Connection, rows: &[UniverseRow], today: NaiveDate) -> Result<()> {
    conn.execute("DELETE FROM research_universe", [])?;
    let now = now_utc();
    let mut stmt = conn.prepare(
        "INSERT INTO research_universe (ticker, asset_id, issuer_id, name, class, sector, segment, currency, metrics, as_of, computed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?;
    for r in rows {
        stmt.execute(params![
            r.ticker,
            r.asset_id,
            r.issuer_id,
            r.name,
            r.class.as_str(),
            r.sector,
            r.segment,
            r.currency,
            serde_json::to_string(&r.metrics)?,
            fmt_date(today),
            now
        ])?;
    }
    Ok(())
}

fn load_snapshot(conn: &Connection) -> Result<Vec<UniverseRow>> {
    let mut stmt = conn.prepare("SELECT * FROM research_universe ORDER BY ticker")?;
    let rows = stmt
        .query_map([], |r| {
            let class: String = r.get("class")?;
            let metrics: String = r.get("metrics")?;
            Ok((
                UniverseRow {
                    ticker: r.get("ticker")?,
                    asset_id: r.get("asset_id")?,
                    issuer_id: r.get("issuer_id")?,
                    name: r.get("name")?,
                    class: AssetClass::parse(&class).unwrap_or(AssetClass::Other),
                    sector: r.get("sector")?,
                    segment: r.get("segment")?,
                    currency: r.get("currency")?,
                    metrics: BTreeMap::new(),
                    score_band: None,
                    red_flags: None,
                },
                metrics,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.into_iter()
        .map(|(mut u, m)| {
            u.metrics = serde_json::from_str(&m)?;
            Ok(u)
        })
        .collect()
}

/// Universo do dia: o gravado, se for de hoje e posterior às últimas coletas; senão
/// recalcula (e grava, se o banco aceitar escrita).
pub fn universe(
    conn: &Connection,
    today: NaiveDate,
    bazin: BazinParams,
) -> Result<(Vec<UniverseRow>, String)> {
    let meta: Option<(String, String)> = conn
        .query_row(
            "SELECT as_of, min(computed_at) FROM research_universe GROUP BY as_of ORDER BY as_of DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let last_sync = [
        super::config::LAST_SYNC_SETTING,
        crate::indices::sync::LAST_SYNC_SETTING,
    ]
    .iter()
    .filter_map(|k| crate::settings::get::<String>(conn, k).ok().flatten())
    .max();
    // Ativo cadastrado depois do cálculo também deixa o universo velho.
    let missing_asset: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM asset a WHERE a.archived = 0 AND a.ticker IS NOT NULL
            AND a.class IN ('stock_br', 'fii', 'bdr', 'stock_us', 'reit')
            AND NOT EXISTS (SELECT 1 FROM research_universe u WHERE u.asset_id = a.id))",
        [],
        |r| r.get(0),
    )?;
    if let Some((as_of, computed)) = &meta
        && *as_of == fmt_date(today)
        && last_sync.as_ref().is_none_or(|l| computed >= l)
        && !missing_asset
    {
        let mut rows = load_snapshot(conn)?;
        attach_scores(conn, &mut rows)?;
        return Ok((rows, as_of.clone()));
    }
    let mut rows = compute_universe(conn, today, bazin)?;
    // Grava numa transação; com o banco somente leitura (MCP com o app fechado) só usa.
    if let Ok(tx) = conn.unchecked_transaction()
        && store_universe(&tx, &rows, today).is_ok()
    {
        let _ = tx.commit();
    }
    attach_scores(conn, &mut rows)?;
    Ok((rows, fmt_date(today)))
}

// ---------------------------------------------------------------------------
// Filtros

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

/// Recorte do universo (classes, setores, liquidez).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UniverseFilter {
    #[serde(default)]
    #[ts(optional)]
    pub classes: Option<Vec<AssetClass>>,
    /// Trechos do setor (sem diferenciar acentos e maiúsculas).
    #[serde(default)]
    #[ts(optional)]
    pub sectors: Option<Vec<String>>,
    /// Trechos do segmento (FIIs: logística, shoppings…).
    #[serde(default)]
    #[ts(optional)]
    pub segments: Option<Vec<String>>,
    /// Liquidez média diária mínima, na moeda do ativo.
    #[serde(default)]
    #[ts(optional)]
    pub min_liquidity: Option<f64>,
    /// Só ativos da carteira e da watchlist.
    #[serde(default)]
    #[ts(optional)]
    pub only_mine: Option<bool>,
}

fn mine(conn: &Connection) -> Result<HashSet<String>> {
    Ok(data::held_or_watched(conn)?.into_iter().collect())
}

fn passes(u: &UniverseRow, f: &UniverseFilter, mine: &HashSet<String>) -> bool {
    if let Some(c) = f.classes.as_ref().filter(|c| !c.is_empty())
        && !c.contains(&u.class)
    {
        return false;
    }
    let has = |field: &Option<String>, wanted: &Option<Vec<String>>| match wanted
        .as_ref()
        .filter(|w| !w.is_empty())
    {
        None => true,
        Some(w) => field
            .as_deref()
            .is_some_and(|v| w.iter().any(|x| norm(v).contains(&norm(x)))),
    };
    if !has(&u.sector, &f.sectors) || !has(&u.segment, &f.segments) {
        return false;
    }
    if let Some(min) = f.min_liquidity
        && u.metrics.get("liquidity").is_none_or(|l| *l < min)
    {
        return false;
    }
    if f.only_mine.unwrap_or(false) && u.asset_id.as_ref().is_none_or(|a| !mine.contains(a)) {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Ranking

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RankArgs {
    /// Chave do indicador (`research.catalog`).
    pub indicator: String,
    #[serde(default)]
    #[ts(optional)]
    pub filters: Option<UniverseFilter>,
    /// Padrão: a ordem natural do indicador (menor P/L primeiro, maior DY primeiro).
    #[serde(default)]
    #[ts(optional)]
    pub order: Option<SortOrder>,
    /// Padrão 20, máximo 500.
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RankItem {
    pub position: u32,
    pub ticker: String,
    pub asset_id: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub sector: Option<String>,
    pub segment: Option<String>,
    pub value: f64,
    pub price: Option<f64>,
    pub liquidity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RankResult {
    pub indicator: IndicatorInfo,
    pub order: SortOrder,
    pub as_of: String,
    /// Papéis do recorte com o indicador disponível.
    #[ts(type = "number")]
    pub total: u64,
    /// Papéis do recorte sem o indicador (ou com valor ≤ 0 em múltiplos).
    #[ts(type = "number")]
    pub without_value: u64,
    pub items: Vec<RankItem>,
}

fn unknown_indicator(key: &str) -> Error {
    Error::invalid(crate::i18n::t_args(
        "nucleo-pesquisa-erro-indicador-desconhecido",
        &crate::i18n::args(&[("indicador", key.to_string().into())]),
    ))
}

fn info(key: &str) -> Result<IndicatorInfo> {
    indicators::catalog()
        .into_iter()
        .find(|i| i.key == key)
        .ok_or_else(|| unknown_indicator(key))
}

pub fn rank(
    conn: &Connection,
    rows: &[UniverseRow],
    as_of: &str,
    a: &RankArgs,
) -> Result<RankResult> {
    let def = indicators::def(&a.indicator).ok_or_else(|| unknown_indicator(&a.indicator))?;
    let order = a.order.unwrap_or(match def.higher_is_better {
        Some(false) => SortOrder::Asc,
        _ => SortOrder::Desc,
    });
    let f = a.filters.clone().unwrap_or_default();
    let mine = mine(conn)?;
    let scoped: Vec<&UniverseRow> = rows.iter().filter(|u| passes(u, &f, &mine)).collect();
    let mut with: Vec<(&UniverseRow, f64)> = scoped
        .iter()
        .filter_map(|u| u.metrics.get(def.key).map(|v| (*u, *v)))
        .filter(|(_, v)| v.is_finite() && (!def.positive_only || *v > 0.0))
        .collect();
    with.sort_by(|x, y| {
        let o = x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal);
        let o = if order == SortOrder::Asc {
            o
        } else {
            o.reverse()
        };
        o.then_with(|| x.0.ticker.cmp(&y.0.ticker))
    });
    let limit = a.limit.unwrap_or(20).clamp(1, 500) as usize;
    let total = with.len() as u64;
    Ok(RankResult {
        indicator: info(def.key)?,
        order,
        as_of: as_of.to_string(),
        total,
        without_value: scoped.len() as u64 - total,
        items: with
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(i, (u, v))| RankItem {
                position: i as u32 + 1,
                ticker: u.ticker.clone(),
                asset_id: u.asset_id.clone(),
                name: u.name.clone(),
                class: u.class,
                sector: u.sector.clone(),
                segment: u.segment.clone(),
                value: v,
                price: u.metrics.get("price").copied(),
                liquidity: u.metrics.get("liquidity").copied(),
            })
            .collect(),
    })
}

// ---------------------------------------------------------------------------
// Screener

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Logic {
    /// Todas as condições (E).
    #[default]
    And,
    /// Qualquer condição (OU).
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CmpOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Ne,
    /// `value: [mín, máx]` (inclusive).
    Between,
    /// `value: [...]`.
    In,
    NotIn,
    /// Texto contém (sem diferenciar acentos e maiúsculas).
    Contains,
    /// O indicador está disponível.
    Exists,
    /// O indicador está indisponível.
    Missing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Condition {
    /// Indicador (`pl`, `dy`…) ou campo de texto (`ticker`, `name`, `class`, `sector`,
    /// `segment`, `currency`) ou `held`/`watched` (booleanos).
    pub field: String,
    pub op: CmpOp,
    #[serde(default)]
    #[ts(optional, type = "unknown")]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConditionGroup {
    #[serde(default)]
    pub logic: Logic,
    pub conditions: Vec<Condition>,
}

/// Filtro do screener: condições e grupos combinados por `logic`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenQuery {
    #[serde(default)]
    pub logic: Logic,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub groups: Vec<ConditionGroup>,
    #[serde(default)]
    #[ts(optional)]
    pub universe: Option<UniverseFilter>,
    /// Características (tags do catálogo, `research.tags`): o papel precisa ter todas.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub tags: Vec<String>,
}

const TEXT_FIELDS: &[&str] = &[
    "ticker",
    "name",
    "class",
    "sector",
    "segment",
    "currency",
    "score_band",
    "red_flags",
];
const BOOL_FIELDS: &[&str] = &["held", "watched"];

fn is_numeric_field(f: &str) -> bool {
    CATALOG.iter().any(|d| d.key == f)
}

fn label_of(field: &str) -> String {
    indicators::def(field)
        .map(|d| d.label())
        .unwrap_or_else(|| field.to_string())
}

fn validate_condition(c: &Condition) -> Result<()> {
    let f = c.field.as_str();
    let numeric = is_numeric_field(f);
    if !numeric && !TEXT_FIELDS.contains(&f) && !BOOL_FIELDS.contains(&f) {
        return Err(Error::invalid(crate::i18n::t_args(
            "nucleo-pesquisa-erro-campo-desconhecido",
            &crate::i18n::args(&[
                ("campo", f.to_string().into()),
                (
                    "campos",
                    [TEXT_FIELDS, BOOL_FIELDS].concat().join(", ").into(),
                ),
            ]),
        )));
    }
    let v = c.value.as_ref();
    let num = |v: Option<&Value>| v.and_then(Value::as_f64);
    let bad = |key: &str| {
        Err(Error::invalid(crate::i18n::t_args(
            "nucleo-pesquisa-erro-filtro",
            &crate::i18n::args(&[
                ("campo", f.to_string().into()),
                ("motivo", crate::i18n::t(key).into()),
            ]),
        )))
    };
    match c.op {
        CmpOp::Exists | CmpOp::Missing => Ok(()),
        CmpOp::Gt | CmpOp::Gte | CmpOp::Lt | CmpOp::Lte => {
            if !numeric {
                return bad("nucleo-pesquisa-filtro-comparacao");
            }
            if num(v).is_none() {
                return bad("nucleo-pesquisa-filtro-numero");
            }
            Ok(())
        }
        CmpOp::Between => match v.and_then(Value::as_array) {
            Some(a) if numeric && a.len() == 2 && a.iter().all(|x| x.as_f64().is_some()) => Ok(()),
            _ => bad("nucleo-pesquisa-filtro-between"),
        },
        CmpOp::In | CmpOp::NotIn => match v.and_then(Value::as_array) {
            Some(a) if !a.is_empty() => Ok(()),
            _ => bad("nucleo-pesquisa-filtro-in"),
        },
        CmpOp::Contains => {
            if numeric || v.and_then(Value::as_str).is_none() {
                return bad("nucleo-pesquisa-filtro-contains");
            }
            Ok(())
        }
        CmpOp::Eq | CmpOp::Ne => {
            if v.is_none() {
                return bad("nucleo-pesquisa-filtro-value");
            }
            Ok(())
        }
    }
}

pub fn validate(q: &ScreenQuery) -> Result<()> {
    for c in q
        .conditions
        .iter()
        .chain(q.groups.iter().flat_map(|g| g.conditions.iter()))
    {
        validate_condition(c)?;
    }
    if q.groups.iter().any(|g| g.conditions.is_empty()) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-grupo-vazio",
        )));
    }
    Ok(())
}

struct Ctx<'a> {
    held: &'a HashSet<String>,
    watched: &'a HashSet<String>,
}

fn text_of(u: &UniverseRow, f: &str) -> Option<String> {
    match f {
        "ticker" => Some(u.ticker.clone()),
        "name" => Some(u.name.clone()),
        "class" => Some(u.class.as_str().to_string()),
        "sector" => u.sector.clone(),
        "segment" => u.segment.clone(),
        "currency" => Some(u.currency.clone()),
        "score_band" => u.score_band.clone(),
        "red_flags" => u.red_flags.clone(),
        _ => None,
    }
}

fn eval(u: &UniverseRow, c: &Condition, cx: &Ctx<'_>) -> bool {
    let f = c.field.as_str();
    let v = c.value.as_ref();
    if BOOL_FIELDS.contains(&f) {
        let actual = u.asset_id.as_ref().is_some_and(|a| {
            if f == "held" {
                cx.held.contains(a)
            } else {
                cx.watched.contains(a)
            }
        });
        let want = v.and_then(Value::as_bool).unwrap_or(true);
        return match c.op {
            CmpOp::Ne => actual != want,
            CmpOp::Missing => !actual,
            CmpOp::Exists => actual,
            _ => actual == want,
        };
    }
    if is_numeric_field(f) {
        let x = u.metrics.get(f).copied();
        let n = || v.and_then(Value::as_f64);
        return match (c.op, x) {
            (CmpOp::Exists, x) => x.is_some(),
            (CmpOp::Missing, x) => x.is_none(),
            (_, None) => false,
            (CmpOp::Gt, Some(x)) => n().is_some_and(|n| x > n),
            (CmpOp::Gte, Some(x)) => n().is_some_and(|n| x >= n),
            (CmpOp::Lt, Some(x)) => n().is_some_and(|n| x < n),
            (CmpOp::Lte, Some(x)) => n().is_some_and(|n| x <= n),
            (CmpOp::Eq, Some(x)) => n().is_some_and(|n| (x - n).abs() < 1e-9),
            (CmpOp::Ne, Some(x)) => n().is_some_and(|n| (x - n).abs() >= 1e-9),
            (CmpOp::Between, Some(x)) => v.and_then(Value::as_array).is_some_and(|a| {
                let lo = a.first().and_then(Value::as_f64).unwrap_or(f64::MIN);
                let hi = a.get(1).and_then(Value::as_f64).unwrap_or(f64::MAX);
                x >= lo && x <= hi
            }),
            (CmpOp::In | CmpOp::NotIn, Some(x)) => {
                let hit = v.and_then(Value::as_array).is_some_and(|a| {
                    a.iter()
                        .filter_map(Value::as_f64)
                        .any(|n| (x - n).abs() < 1e-9)
                });
                hit == (c.op == CmpOp::In)
            }
            (CmpOp::Contains, _) => false,
        };
    }
    let t = text_of(u, f);
    let same = |a: &str, b: &Value| b.as_str().is_some_and(|b| norm(a) == norm(b));
    match (c.op, t) {
        (CmpOp::Exists, t) => t.is_some(),
        (CmpOp::Missing, t) => t.is_none(),
        (_, None) => false,
        (CmpOp::Eq, Some(t)) => v.is_some_and(|v| same(&t, v)),
        (CmpOp::Ne, Some(t)) => v.is_some_and(|v| !same(&t, v)),
        (CmpOp::Contains, Some(t)) => v
            .and_then(Value::as_str)
            .is_some_and(|s| norm(&t).contains(&norm(s))),
        (CmpOp::In | CmpOp::NotIn, Some(t)) => {
            let hit = v
                .and_then(Value::as_array)
                .is_some_and(|a| a.iter().any(|x| same(&t, x)));
            hit == (c.op == CmpOp::In)
        }
        _ => false,
    }
}

fn combine(logic: Logic, mut results: impl Iterator<Item = bool>) -> Option<bool> {
    let first = results.next()?;
    Some(match logic {
        Logic::And => first && results.all(|x| x),
        Logic::Or => first || results.any(|x| x),
    })
}

/// O papel passa no filtro? `held`: ativos com operações; `watched`: na watchlist.
pub fn matches(
    u: &UniverseRow,
    q: &ScreenQuery,
    held: &HashSet<String>,
    watched: &HashSet<String>,
) -> bool {
    let cx = Ctx { held, watched };
    let parts =
        q.conditions
            .iter()
            .map(|c| eval(u, c, &cx))
            .chain(q.groups.iter().map(|g| {
                combine(g.logic, g.conditions.iter().map(|c| eval(u, c, &cx))).unwrap_or(true)
            }));
    combine(q.logic, parts).unwrap_or(true)
}

fn describe_condition(c: &Condition) -> String {
    let v = c
        .value
        .as_ref()
        .map(|v| match v {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .unwrap_or_default();
    let word = |key: &str| crate::i18n::t(key);
    let op = match c.op {
        CmpOp::Gt => ">".to_string(),
        CmpOp::Gte => "≥".to_string(),
        CmpOp::Lt => "<".to_string(),
        CmpOp::Lte => "≤".to_string(),
        CmpOp::Eq => "=".to_string(),
        CmpOp::Ne => "≠".to_string(),
        CmpOp::Between => word("nucleo-pesquisa-filtro-op-entre"),
        CmpOp::In => word("nucleo-pesquisa-filtro-op-em"),
        CmpOp::NotIn => word("nucleo-pesquisa-filtro-op-fora-de"),
        CmpOp::Contains => word("nucleo-pesquisa-filtro-op-contem"),
        CmpOp::Exists => word("nucleo-pesquisa-filtro-op-disponivel"),
        CmpOp::Missing => word("nucleo-pesquisa-filtro-op-indisponivel"),
    };
    if matches!(c.op, CmpOp::Exists | CmpOp::Missing) {
        format!("{} {op}", label_of(&c.field))
    } else {
        format!("{} {op} {v}", label_of(&c.field))
    }
}

/// Filtro em texto, no idioma da instalação ("P/L < 10 E DY ≥ 0.06").
pub fn describe(q: &ScreenQuery) -> String {
    let (and, or) = (
        crate::i18n::t("nucleo-pesquisa-filtro-e"),
        crate::i18n::t("nucleo-pesquisa-filtro-ou"),
    );
    let join = |l: Logic| format!(" {} ", if l == Logic::And { &and } else { &or });
    let mut parts: Vec<String> = q.conditions.iter().map(describe_condition).collect();
    for g in &q.groups {
        let inner: Vec<String> = g.conditions.iter().map(describe_condition).collect();
        parts.push(format!("({})", inner.join(&join(g.logic))));
    }
    let tags = (!q.tags.is_empty()).then(|| {
        crate::i18n::t_args(
            "nucleo-tag-filtro",
            &crate::i18n::args(&[(
                "tags",
                q.tags
                    .iter()
                    .map(|k| super::tags::label(k))
                    .collect::<Vec<_>>()
                    .join(", ")
                    .into(),
            )]),
        )
    });
    match (parts.is_empty(), tags) {
        (true, None) => crate::i18n::t("nucleo-pesquisa-filtro-todos"),
        (true, Some(t)) => t,
        (false, None) => parts.join(&join(q.logic)),
        (false, Some(t)) => format!(
            "{}{and_sep}{t}",
            parts.join(&join(q.logic)),
            and_sep = join(Logic::And)
        ),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenArgs {
    /// Filtros (ou use `screen_id` de uma tela salva).
    #[serde(default)]
    #[ts(optional)]
    pub filters: Option<ScreenQuery>,
    #[serde(default)]
    #[ts(optional)]
    pub screen_id: Option<String>,
    /// Indicador para ordenar (padrão: valor de mercado).
    #[serde(default)]
    #[ts(optional)]
    pub sort_by: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub order: Option<SortOrder>,
    /// Padrão 50, máximo 500.
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<u32>,
    /// Indicadores a devolver em cada linha (além dos filtrados e do de ordenação).
    #[serde(default)]
    #[ts(optional)]
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenRow {
    pub ticker: String,
    pub asset_id: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub sector: Option<String>,
    pub segment: Option<String>,
    pub currency: String,
    #[ts(type = "Record<string, number>")]
    pub metrics: BTreeMap<String, f64>,
    /// Características do papel (tags do catálogo).
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenResult {
    pub as_of: String,
    /// Filtro em português.
    pub description: String,
    /// Quantos papéis passaram (antes do limite).
    #[ts(type = "number")]
    pub total: u64,
    pub items: Vec<ScreenRow>,
    /// Quantos papéis do recorte (todos, antes do limite) têm cada característica: a contagem
    /// facetada do explorador. Com tags marcadas, as marcadas dão o total.
    #[serde(default)]
    #[ts(type = "Record<string, number>")]
    pub tag_counts: BTreeMap<String, u32>,
}

const DEFAULT_FIELDS: &[&str] = &["price", "market_cap", "liquidity", "pl", "pvp", "dy", "roe"];

pub fn run_screen(
    conn: &Connection,
    rows: &[UniverseRow],
    as_of: &str,
    a: &ScreenArgs,
) -> Result<ScreenResult> {
    let q = match (&a.filters, &a.screen_id) {
        (Some(q), _) => q.clone(),
        (None, Some(id)) => get(conn, id)?.filters,
        (None, None) => ScreenQuery::default(),
    };
    validate(&q)?;
    super::tags::validate(&q.tags)?;
    let tagged = super::tags::of_universe(rows, as_of);
    let sort_by = a.sort_by.clone().unwrap_or_else(|| "market_cap".into());
    let def = indicators::def(&sort_by).ok_or_else(|| {
        Error::invalid(crate::i18n::t_args(
            "nucleo-pesquisa-erro-ordenar-desconhecido",
            &crate::i18n::args(&[("indicador", sort_by.clone().into())]),
        ))
    })?;
    let order = a.order.unwrap_or(match def.higher_is_better {
        Some(false) => SortOrder::Asc,
        _ => SortOrder::Desc,
    });
    let mine = mine(conn)?;
    let ids = |sql: &str| -> Result<HashSet<String>> {
        Ok(conn
            .prepare(sql)?
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?)
    };
    let held = ids("SELECT DISTINCT asset_id FROM trade")?;
    let watched = ids("SELECT asset_id FROM watchlist_item")?;
    let uf = q.universe.clone().unwrap_or_default();
    let mut hits: Vec<&UniverseRow> = rows
        .iter()
        .filter(|u| {
            passes(u, &uf, &mine)
                && matches(u, &q, &held, &watched)
                && q.tags.iter().all(|k| {
                    tagged
                        .get(&u.ticker)
                        .is_some_and(|v| v.iter().any(|x| x == k))
                })
        })
        .collect();
    hits.sort_by(|x, y| {
        let (a, b) = (x.metrics.get(def.key), y.metrics.get(def.key));
        let o = match (a, b) {
            (Some(a), Some(b)) => {
                let o = a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal);
                if order == SortOrder::Asc {
                    o
                } else {
                    o.reverse()
                }
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        };
        o.then_with(|| x.ticker.cmp(&y.ticker))
    });
    let mut keep: HashSet<String> = DEFAULT_FIELDS.iter().map(|s| s.to_string()).collect();
    keep.insert(sort_by.clone());
    for c in q
        .conditions
        .iter()
        .chain(q.groups.iter().flat_map(|g| g.conditions.iter()))
    {
        keep.insert(c.field.clone());
    }
    for f in a.fields.iter().flatten() {
        keep.insert(f.clone());
    }
    let mut tag_counts: BTreeMap<String, u32> = BTreeMap::new();
    for k in hits.iter().filter_map(|u| tagged.get(&u.ticker)).flatten() {
        *tag_counts.entry(k.clone()).or_default() += 1;
    }
    let limit = a.limit.unwrap_or(50).clamp(1, 500) as usize;
    Ok(ScreenResult {
        tag_counts,
        as_of: as_of.to_string(),
        description: describe(&q),
        total: hits.len() as u64,
        items: hits
            .into_iter()
            .take(limit)
            .map(|u| ScreenRow {
                ticker: u.ticker.clone(),
                asset_id: u.asset_id.clone(),
                name: u.name.clone(),
                class: u.class,
                sector: u.sector.clone(),
                segment: u.segment.clone(),
                currency: u.currency.clone(),
                metrics: u
                    .metrics
                    .iter()
                    .filter(|(k, _)| keep.contains(*k))
                    .map(|(k, v)| (k.clone(), *v))
                    .collect(),
                tags: tagged.get(&u.ticker).cloned().unwrap_or_default(),
            })
            .collect(),
    })
}

// ---------------------------------------------------------------------------
// Telas salvas (dados do usuário: passam pelo histórico de alterações)

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Screen {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub filters: ScreenQuery,
    /// Avisa quando um papel novo passa a cumprir a tela (alerta de tela).
    pub alert: bool,
    /// Filtro em português.
    pub summary: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenInput {
    pub name: String,
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
    pub filters: ScreenQuery,
    #[serde(default)]
    #[ts(optional)]
    pub alert: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScreenPatch {
    pub id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub filters: Option<ScreenQuery>,
    #[serde(default)]
    #[ts(optional)]
    pub alert: Option<bool>,
}

fn row_to_screen(r: &rusqlite::Row<'_>) -> rusqlite::Result<Screen> {
    let q: String = r.get("query")?;
    let filters: ScreenQuery = serde_json::from_str(&q).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(Screen {
        id: r.get("id")?,
        name: r.get("name")?,
        description: r.get("description")?,
        alert: r.get::<_, i64>("alert")? == 1,
        summary: describe(&filters),
        filters,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Screen>> {
    let mut stmt = conn.prepare("SELECT * FROM screen ORDER BY lower(name)")?;
    let rows = stmt
        .query_map([], row_to_screen)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: &str) -> Result<Screen> {
    conn.query_row("SELECT * FROM screen WHERE id = ?1", [id], row_to_screen)
        .optional()?
        .ok_or_else(|| {
            Error::not_found(crate::i18n::t_args(
                "nucleo-pesquisa-ref-tela",
                &crate::i18n::args(&[("id", id.to_string().into())]),
            ))
        })
}

fn check_name(conn: &Connection, name: &str, except: Option<&str>) -> Result<String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-tela-nome",
        )));
    }
    let dup: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM screen WHERE lower(name) = lower(?1) AND id <> coalesce(?2, ''))",
        params![n, except],
        |r| r.get(0),
    )?;
    if dup {
        return Err(Error::conflict(crate::i18n::t_args(
            "nucleo-pesquisa-erro-tela-duplicada",
            &crate::i18n::args(&[("nome", n.to_string().into())]),
        )));
    }
    Ok(n.to_string())
}

pub fn create(tx: &Transaction<'_>, i: &ScreenInput) -> Result<Id> {
    let name = check_name(tx, &i.name, None)?;
    validate(&i.filters)?;
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO screen (id, name, description, query, created_at, updated_at, alert) VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
        params![
            id,
            name,
            i.description.as_deref().map(str::trim).filter(|d| !d.is_empty()),
            serde_json::to_string(&i.filters)?,
            now,
            i.alert.unwrap_or(false) as i64
        ],
    )?;
    Ok(id)
}

pub fn update(tx: &Transaction<'_>, p: &ScreenPatch) -> Result<()> {
    get(tx, &p.id)?;
    let now = now_utc();
    if let Some(n) = &p.name {
        let n = check_name(tx, n, Some(&p.id))?;
        tx.execute(
            "UPDATE screen SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, n, now],
        )?;
    }
    if let Some(d) = &p.description {
        tx.execute(
            "UPDATE screen SET description = ?2, updated_at = ?3 WHERE id = ?1",
            params![
                p.id,
                d.as_deref().map(str::trim).filter(|d| !d.is_empty()),
                now
            ],
        )?;
    }
    if let Some(q) = &p.filters {
        validate(q)?;
        tx.execute(
            "UPDATE screen SET query = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, serde_json::to_string(q)?, now],
        )?;
    }
    if let Some(a) = p.alert {
        tx.execute(
            "UPDATE screen SET alert = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, a as i64, now],
        )?;
    }
    Ok(())
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM screen WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(crate::i18n::t_args(
            "nucleo-pesquisa-ref-tela",
            &crate::i18n::args(&[("id", id.to_string().into())]),
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(t: &str, class: AssetClass, sector: &str, m: &[(&str, f64)]) -> UniverseRow {
        UniverseRow {
            ticker: t.into(),
            asset_id: (t == "AAAA3").then(|| "a1".to_string()),
            issuer_id: None,
            name: format!("{t} S.A."),
            class,
            sector: Some(sector.into()),
            segment: None,
            currency: "BRL".into(),
            metrics: m.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            score_band: None,
            red_flags: None,
        }
    }

    fn universe() -> Vec<UniverseRow> {
        vec![
            row(
                "AAAA3",
                AssetClass::StockBr,
                "Energia Elétrica",
                &[
                    ("pl", 5.0),
                    ("dy", 0.09),
                    ("roe", 0.20),
                    ("liquidity", 5e6),
                    ("market_cap", 1e9),
                ],
            ),
            row(
                "BBBB4",
                AssetClass::StockBr,
                "Bancos",
                &[
                    ("pl", 8.0),
                    ("dy", 0.07),
                    ("roe", 0.18),
                    ("liquidity", 9e7),
                    ("market_cap", 9e10),
                ],
            ),
            row(
                "CCCC3",
                AssetClass::StockBr,
                "Varejo",
                &[
                    ("pl", -3.0),
                    ("dy", 0.0),
                    ("liquidity", 2e5),
                    ("market_cap", 5e8),
                ],
            ),
            row(
                "DDDD3",
                AssetClass::StockBr,
                "Energia Elétrica",
                &[
                    ("pl", 12.0),
                    ("dy", 0.04),
                    ("roe", 0.11),
                    ("liquidity", 1e6),
                    ("market_cap", 3e9),
                ],
            ),
            row(
                "FIII11",
                AssetClass::Fii,
                "Fundos imobiliários",
                &[("pvp", 0.9), ("dy", 0.11), ("liquidity", 3e6)],
            ),
        ]
    }

    fn q(v: serde_json::Value) -> ScreenQuery {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn screener_and_or_and_groups() {
        let u = universe();
        let none = HashSet::new();
        let run = |query: ScreenQuery| -> Vec<String> {
            validate(&query).unwrap();
            u.iter()
                .filter(|r| matches(r, &query, &none, &none))
                .map(|r| r.ticker.clone())
                .collect()
        };
        // E: P/L < 10 e DY ≥ 6%.
        let and = q(
            json!({"conditions": [{"field": "pl", "op": "lt", "value": 10}, {"field": "dy", "op": "gte", "value": 0.06}]}),
        );
        assert_eq!(run(and.clone()), vec!["AAAA3", "BBBB4"]);
        assert_eq!(
            describe(&and),
            "P/L < 10 E Dividend yield (12 meses) ≥ 0.06"
        );
        // OU.
        let or = q(
            json!({"logic": "or", "conditions": [{"field": "pl", "op": "gt", "value": 10}, {"field": "class", "op": "eq", "value": "fii"}]}),
        );
        assert_eq!(run(or), vec!["DDDD3", "FIII11"]);
        // (P/L entre 0 e 10 E setor contém "energia") OU (P/VP < 1).
        let groups = q(json!({"logic": "or", "groups": [
            {"conditions": [{"field": "pl", "op": "between", "value": [0, 10]}, {"field": "sector", "op": "contains", "value": "ENERGIA"}]},
            {"conditions": [{"field": "pvp", "op": "lt", "value": 1}]}
        ]}));
        assert_eq!(run(groups.clone()), vec!["AAAA3", "FIII11"]);
        assert!(describe(&groups).contains(") OU ("));
        // Indicador ausente não passa em comparação; `missing` acha.
        assert_eq!(
            run(q(
                json!({"conditions": [{"field": "roe", "op": "missing"}]})
            )),
            vec!["CCCC3", "FIII11"]
        );
        assert_eq!(
            run(q(
                json!({"conditions": [{"field": "ticker", "op": "in", "value": ["bbbb4", "XXXX3"]}]})
            )),
            vec!["BBBB4"]
        );
        // Erros legíveis.
        for bad in [
            json!({"conditions": [{"field": "nao_existe", "op": "gt", "value": 1}]}),
            json!({"conditions": [{"field": "pl", "op": "gt", "value": "dez"}]}),
            json!({"conditions": [{"field": "pl", "op": "between", "value": [1]}]}),
            json!({"conditions": [{"field": "sector", "op": "gt", "value": 1}]}),
            json!({"groups": [{"conditions": []}]}),
        ] {
            assert!(validate(&q(bad.clone())).is_err(), "{bad}");
        }
    }

    #[test]
    fn universe_filter() {
        let u = universe();
        let none = HashSet::new();
        let f = UniverseFilter {
            classes: Some(vec![AssetClass::StockBr]),
            sectors: Some(vec!["energia eletrica".into()]),
            min_liquidity: Some(2e6),
            ..Default::default()
        };
        let hits: Vec<&str> = u
            .iter()
            .filter(|r| passes(r, &f, &none))
            .map(|r| r.ticker.as_str())
            .collect();
        assert_eq!(hits, vec!["AAAA3"]);
        let mine: HashSet<String> = ["a1".to_string()].into();
        let f = UniverseFilter {
            only_mine: Some(true),
            ..Default::default()
        };
        assert_eq!(u.iter().filter(|r| passes(r, &f, &mine)).count(), 1);
    }

    #[test]
    fn tag_counts_are_faceted_by_the_screen() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE asset (id TEXT, ticker TEXT, archived INTEGER);
             CREATE TABLE trade (asset_id TEXT);
             CREATE TABLE watchlist_item (asset_id TEXT);",
        )
        .unwrap();
        let rows = universe();
        let run = |q: serde_json::Value| {
            run_screen(
                &conn,
                &rows,
                "2026-10-30",
                &ScreenArgs {
                    filters: Some(serde_json::from_value(q).unwrap()),
                    limit: Some(1),
                    ..Default::default()
                },
            )
            .unwrap()
        };
        let n = |r: &ScreenResult, k: &str| r.tag_counts.get(k).copied().unwrap_or(0);
        // DY alto: AAAA3, BBBB4 e o FII (10% para FII); dos três, dois têm ROE alto.
        let r = run(json!({"logic": "and", "conditions": [], "groups": [], "tags": ["dy_alto"]}));
        assert_eq!(r.total, 3);
        assert_eq!(r.items.len(), 1, "o limite não corta as contagens");
        assert_eq!((n(&r, "dy_alto"), n(&r, "roe_alto")), (3, 2));
        assert_eq!(n(&r, "divida_baixa"), 0);
        // Duas marcadas: as duas dão o total do recorte.
        let r = run(json!({"logic": "and", "conditions": [], "groups": [],
            "tags": ["dy_alto", "roe_alto"]}));
        assert_eq!(r.total, 2);
        assert_eq!((n(&r, "dy_alto"), n(&r, "roe_alto")), (2, 2));
        // O recorte do universo (setor) também vale para as contagens.
        let r = run(json!({"logic": "and", "conditions": [], "groups": [],
            "universe": {"sectors": ["Bancos"]}, "tags": ["dy_alto"]}));
        assert_eq!((r.total, n(&r, "dy_alto"), n(&r, "roe_alto")), (1, 1, 1));
    }
}
