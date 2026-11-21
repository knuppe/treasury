use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::Deflator;
use super::providers::{DataProvider, DataProviderInfo};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::{fmt_date, parse_date};

pub const READ_ONLY: &[&str] = &[
    "list_series",
    "get_series",
    "factor",
    "catalog",
    "providers",
    "references",
];

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IndexSeries {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub unit: String,
    pub frequency: String,
    pub country: String,
    pub source: String,
    pub last_fetched_at: Option<String>,
    pub last_official_date: Option<String>,
    pub last_date: Option<String>,
    #[ts(type = "number")]
    pub count: i64,
    pub provider: Option<DataProvider>,
    /// Coletada hoje: séries de sempre (BCB, FRED, Yahoo) ou do país de referência.
    pub in_use: bool,
}

/// Situação de um provedor de dados para Ajustes › Fontes de dados.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProviderStatus {
    pub info: DataProviderInfo,
    /// Alguma série do provedor é coletada (sempre ou pelo país de referência).
    pub in_use: bool,
    #[ts(type = "number")]
    pub series_total: i64,
    #[ts(type = "number")]
    pub series_in_use: i64,
    pub last_fetched_at: Option<String>,
    /// Data do dado mais recente entre as séries em uso.
    pub last_date: Option<String>,
    /// `ok`, `error` (a última coleta de alguma série falhou), `never` ou `idle` (fora de uso).
    pub status: String,
    pub error: Option<String>,
    pub error_at: Option<String>,
}

#[derive(Deserialize)]
struct CountryArgs {
    country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IndexValuePoint {
    pub date: String,
    /// Decimal em texto.
    pub value: String,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FactorResult {
    pub deflator: Deflator,
    /// Série usada (para `country`, a principal de inflação do país de referência).
    pub series_id: Option<String>,
    pub from: String,
    pub to: String,
    /// Fator acumulado (1 + inflação no período), decimal em texto.
    pub factor: Option<String>,
    pub inflation: Option<f64>,
    pub estimated: bool,
}

#[derive(Deserialize)]
struct SeriesArgs {
    id: String,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Deserialize)]
struct FactorArgs {
    deflator: Deflator,
    from: String,
    to: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list_series" => to_value(list_series(ctx.conn())?),
        "catalog" => to_value(super::catalog::catalog(ctx.conn())?),
        "providers" => to_value(providers_status(ctx.conn())?),
        // Taxas de referência das calculadoras (juro básico, CDI, inflação 12 meses, poupança).
        "references" => to_value(super::references::references(ctx.conn())?),
        "set_country" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-ativos-erro-pais-so-interface",
                )));
            }
            let a: CountryArgs = parse_args(args)?;
            super::catalog::set_reference_country(ctx.conn(), a.country.as_deref())?;
            to_value(super::catalog::catalog(ctx.conn())?)
        }
        "get_series" => {
            let a: SeriesArgs = parse_args(args)?;
            let to =
                a.to.as_deref()
                    .map(parse_date)
                    .transpose()?
                    .unwrap_or_else(crate::time::today_local);
            let from = a
                .from
                .as_deref()
                .map(parse_date)
                .transpose()?
                .unwrap_or_else(|| crate::time::add_months(to, -24));
            let pts = super::points(ctx.conn(), &a.id, from, to)?;
            to_value(
                pts.into_iter()
                    .map(|p| IndexValuePoint {
                        date: fmt_date(p.date),
                        value: p.value.normalize().to_string(),
                        estimated: p.estimated,
                    })
                    .collect::<Vec<_>>(),
            )
        }
        "factor" => {
            let a: FactorArgs = parse_args(args)?;
            let f = super::deflator_factor(
                ctx.conn(),
                a.deflator,
                parse_date(&a.from)?,
                parse_date(&a.to)?,
            )?;
            use rust_decimal::prelude::ToPrimitive;
            to_value(FactorResult {
                series_id: a.deflator.resolve(ctx.conn())?,
                deflator: a.deflator,
                from: a.from,
                to: a.to,
                factor: f
                    .as_ref()
                    .map(|f| f.factor.round_dp(10).normalize().to_string()),
                inflation: f
                    .as_ref()
                    .and_then(|f| (f.factor - rust_decimal::Decimal::ONE).to_f64()),
                estimated: f.is_some_and(|f| f.estimated),
            })
        }
        _ => Err(Error::UnknownCommand(format!("indices.{action}"))),
    }
}

pub fn list_series(conn: &rusqlite::Connection) -> Result<Vec<IndexSeries>> {
    let scope: std::collections::HashSet<String> =
        super::catalog::sync_scope(conn)?.into_iter().collect();
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.kind, s.unit, s.frequency, s.country, s.source, s.last_fetched_at,
                (SELECT max(date) FROM index_value v WHERE v.series_id = s.id AND v.estimated = 0),
                (SELECT max(date) FROM index_value v WHERE v.series_id = s.id),
                (SELECT count(*) FROM index_value v WHERE v.series_id = s.id)
         FROM index_series s ORDER BY s.country, s.id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            let id: String = r.get(0)?;
            let source: String = r.get(6)?;
            let provider = DataProvider::of_source(&source).map(|(p, _)| p);
            Ok(IndexSeries {
                in_use: provider.is_some_and(|p| p.is_legacy()) || scope.contains(&id),
                provider,
                id,
                name: r.get(1)?,
                kind: r.get(2)?,
                unit: r.get(3)?,
                frequency: r.get(4)?,
                country: r.get(5)?,
                source,
                last_fetched_at: r.get(7)?,
                last_official_date: r.get(8)?,
                last_date: r.get(9)?,
                count: r.get(10)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Um item por provedor: séries, uso, última coleta e o último erro das séries em uso.
pub fn providers_status(conn: &rusqlite::Connection) -> Result<Vec<ProviderStatus>> {
    let series = list_series(conn)?;
    let errors = super::sync::SyncErrors::load(conn)?;
    let mut out = vec![];
    for p in DataProvider::ALL {
        let mine: Vec<&IndexSeries> = series.iter().filter(|s| s.provider == Some(p)).collect();
        let used: Vec<&&IndexSeries> = mine.iter().filter(|s| s.in_use).collect();
        let last_fetched_at = used.iter().filter_map(|s| s.last_fetched_at.clone()).max();
        let last_date = used.iter().filter_map(|s| s.last_date.clone()).max();
        let err = used
            .iter()
            .filter_map(|s| errors.0.get(&s.id).map(|e| (s, e)))
            .max_by(|a, b| a.1.at.cmp(&b.1.at));
        let status = if used.is_empty() {
            "idle"
        } else if err.is_some() {
            "error"
        } else if last_fetched_at.is_none() {
            "never"
        } else {
            "ok"
        };
        out.push(ProviderStatus {
            info: p.info(),
            in_use: !used.is_empty(),
            series_total: mine.len() as i64,
            series_in_use: used.len() as i64,
            last_fetched_at,
            last_date,
            status: status.into(),
            error: err.map(|(s, e)| format!("{}: {}", s.name, e.error)),
            error_at: err.map(|(_, e)| e.at.clone()),
        });
    }
    Ok(out)
}
