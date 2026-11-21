//! Comparar a rentabilidade com qualquer série (docs/portfolio-compare.md): um ativo do cadastro
//! ou da watchlist, uma série do catálogo de índices ou o CDI (o juro de referência do país fora
//! do Brasil) mais uma taxa ao ano. O acumulado sai nos mesmos meses da rentabilidade, com a
//! mesma base: o fim do mês anterior ao primeiro mês.
//!
//! Só desempenho passado lado a lado, sem ranking (CVM Res. 20/2021).

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::fx_to_base;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::indices;
use crate::time::{month_bounds, parse_month};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum BenchmarkTarget {
    /// Um ativo do cadastro (inclusive da watchlist), pelo fechamento na moeda base.
    Asset { id: String },
    /// Uma série do catálogo de índices (nível, variação mensal, taxa diária ou anual).
    Index { id: String },
    /// O CDI (fora do Brasil, o juro de referência do país) mais `spread` ao ano (0,02 = 2%).
    CdiPlus { spread: f64 },
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct BenchmarkArgs {
    /// Os meses da rentabilidade (`YYYY-MM`), em ordem.
    pub months: Vec<String>,
    pub target: BenchmarkTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BenchmarkSeries {
    pub label: String,
    /// Acumulado no fim de cada mês (razão); `null` sem dado.
    pub cumulative: Vec<Option<f64>>,
    pub estimated: bool,
}

/// De onde sai cada ponto da série.
enum Source {
    /// Fechamento do ativo convertido para a moeda base.
    Asset { id: String, currency: String },
    /// Índice em pontos (Ibovespa, IFIX, S&P 500…).
    Level { id: String },
    /// Variação mensal (IPCA, IGP-M…).
    Monthly { id: String },
    /// Taxa diária em % a.d. (CDI), com um extra ao ano por dia útil.
    Daily {
        points: Vec<(NaiveDate, Decimal)>,
        extra: f64,
    },
    /// Taxa anual em % a.a. (Selic meta, juro de referência), com um extra ao ano.
    Annual {
        points: Vec<(NaiveDate, Decimal)>,
        extra: f64,
    },
}

fn level(conn: &Connection, s: &Source, d: NaiveDate) -> Result<Option<Decimal>> {
    match s {
        Source::Asset { id, currency } => {
            let Some((close, _)) = crate::market::close_on(conn, id, d)? else {
                return Ok(None);
            };
            Ok(fx_to_base(conn, currency, d)?.map(|(fx, _)| close * fx))
        }
        Source::Level { id } => indices::level_on(conn, id, d),
        _ => Ok(None),
    }
}

/// Taxa diária (% a.d.) acumulada entre `from` (exclusive) e `to` (inclusive), com `extra` ao
/// ano distribuído por dia útil (252).
fn daily_growth(
    points: &[(NaiveDate, Decimal)],
    from: NaiveDate,
    to: NaiveDate,
    extra: f64,
) -> Option<f64> {
    let day_extra = (1.0 + extra).powf(1.0 / 252.0);
    let mut acc = 1.0f64;
    let mut any = false;
    for (_, v) in points.iter().filter(|(d, _)| *d > from && *d <= to) {
        acc *= (1.0 + v.to_f64()? / 100.0) * day_extra;
        any = true;
    }
    any.then_some(acc - 1.0)
}

/// Acumulado da série entre `base` e `end` (razão) e se veio de dado estimado.
fn growth(
    conn: &Connection,
    s: &Source,
    base: NaiveDate,
    end: NaiveDate,
) -> Result<Option<(f64, bool)>> {
    Ok(match s {
        Source::Asset { .. } | Source::Level { .. } => {
            match (level(conn, s, base)?, level(conn, s, end)?) {
                (Some(b), Some(n)) if !b.is_zero() => {
                    (n / b - Decimal::ONE).to_f64().map(|g| (g, false))
                }
                _ => None,
            }
        }
        Source::Monthly { id } => indices::series_factor(
            conn,
            id,
            "monthly_rate",
            base + chrono::Duration::days(1),
            end + chrono::Duration::days(1),
        )?
        .and_then(|f| f.factor.to_f64().map(|x| (x - 1.0, f.estimated))),
        Source::Daily { points, extra } => {
            daily_growth(points, base, end, *extra).map(|g| (g, false))
        }
        Source::Annual { points, extra } => {
            indices::annual_rate_growth(points, base, end).map(|g| {
                let years = (end - base).num_days() as f64 / 365.0;
                ((1.0 + g) * (1.0 + extra).powf(years) - 1.0, false)
            })
        }
    })
}

fn series_info(conn: &Connection, id: &str) -> Result<(String, String)> {
    conn.query_row(
        "SELECT kind, name FROM index_series WHERE id = ?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()?
    .ok_or_else(|| Error::not_found(t("nucleo-carteira-comparar-serie-desconhecida")))
}

/// "2,00%" no idioma da instalação.
fn pct_text(rate: f64) -> String {
    format!(
        "{}%",
        crate::i18n::number(crate::i18n::current(), (rate * 10_000.0).round() as i64)
    )
}

pub fn benchmark_series(conn: &Connection, a: &BenchmarkArgs) -> Result<BenchmarkSeries> {
    let months: Vec<(i32, u32)> = a
        .months
        .iter()
        .map(|m| parse_month(m))
        .collect::<Result<_>>()?;
    let Some(&(y0, m0)) = months.first() else {
        return Ok(BenchmarkSeries {
            label: String::new(),
            cumulative: vec![],
            estimated: false,
        });
    };
    let today = crate::time::today_local();
    let base = month_bounds(y0, m0).0 - chrono::Duration::days(1);
    // Taxas anuais valem desde a última mudança, que pode ser bem antes do período.
    let rates_from = base - chrono::Duration::days(800);
    let (label, source) = match &a.target {
        BenchmarkTarget::Asset { id } => {
            let asset = crate::assets::get(conn, id)?;
            (
                asset.ticker.clone().unwrap_or_else(|| asset.name.clone()),
                Source::Asset {
                    id: id.clone(),
                    currency: asset.currency.clone(),
                },
            )
        }
        BenchmarkTarget::Index { id } => {
            let (kind, name) = series_info(conn, id)?;
            let source = match kind.as_str() {
                "level" => Source::Level { id: id.clone() },
                "monthly_rate" => Source::Monthly { id: id.clone() },
                "daily_rate" => Source::Daily {
                    points: indices::daily_rates(conn, id, base, today)?,
                    extra: 0.0,
                },
                _ => Source::Annual {
                    points: indices::daily_rates(conn, id, rates_from, today)?,
                    extra: 0.0,
                },
            };
            (name, source)
        }
        BenchmarkTarget::CdiPlus { spread } => {
            if crate::country::is_brazil(conn) {
                (
                    t_args(
                        "nucleo-carteira-comparar-cdi-mais",
                        &args(&[("indice", "CDI".into()), ("taxa", pct_text(*spread).into())]),
                    ),
                    Source::Daily {
                        points: indices::daily_rates(conn, "cdi", base, today)?,
                        extra: *spread,
                    },
                )
            } else {
                let Some((name, id)) = indices::catalog::reference_rate(conn)? else {
                    return Err(Error::invalid(t("nucleo-carteira-comparar-sem-juro")));
                };
                (
                    t_args(
                        "nucleo-carteira-comparar-cdi-mais",
                        &args(&[("indice", name.into()), ("taxa", pct_text(*spread).into())]),
                    ),
                    Source::Annual {
                        points: indices::daily_rates(conn, &id, rates_from, today)?,
                        extra: *spread,
                    },
                )
            }
        }
    };
    let mut cumulative = Vec::with_capacity(months.len());
    let mut estimated = false;
    for (y, m) in months {
        let end = month_bounds(y, m).1.min(today);
        let v = growth(conn, &source, base, end)?;
        estimated |= v.is_some_and(|(_, e)| e);
        cumulative.push(v.map(|(g, _)| g));
    }
    Ok(BenchmarkSeries {
        label,
        cumulative,
        estimated,
    })
}
