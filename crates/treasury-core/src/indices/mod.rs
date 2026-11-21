//! Índices (IPCA, CDI, Selic, M2, US CPI, HICP, CPI de outros países…): leitura e gravação
//! das séries e fatores de deflação usados pela carteira. Os adaptadores de coleta (BCB SGS,
//! Olinda, FRED e os provedores de [`providers`]) gravam com [`upsert_values`]; tudo que é
//! lido aqui é medido e tem fonte.

pub mod api;
pub mod catalog;
pub mod providers;
pub mod references;
pub mod sync;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::finance::inflation::{self, Factor, MonthlyRate};
use crate::time::{fmt_date, now_utc, parse_date};

/// Deflator escolhido pelo usuário para o resultado real.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Deflator {
    /// IPCA (padrão para BRL).
    #[default]
    Ipca,
    /// IGP-M (FGV), variação mensal.
    Igpm,
    /// US CPI-U (ativos em USD na moeda original).
    UsCpi,
    /// Expansão monetária do Brasil.
    M2Br,
    /// Expansão monetária dos EUA.
    M2Us,
    /// Sem deflação (nominal).
    None,
    /// Inflação do país de referência (catálogo de países). No Brasil é o IPCA.
    Country,
}

impl Deflator {
    pub fn series_id(self) -> Option<&'static str> {
        match self {
            Deflator::Ipca => Some("ipca"),
            Deflator::Igpm => Some("igpm"),
            Deflator::UsCpi => Some("us_cpi"),
            Deflator::M2Br => Some("m2_br"),
            Deflator::M2Us => Some("us_m2"),
            Deflator::None | Deflator::Country => None,
        }
    }

    /// Série efetiva: para `Country`, a principal de inflação do país de referência.
    pub fn resolve(self, conn: &Connection) -> Result<Option<String>> {
        match self {
            Deflator::Country => Ok(catalog::inflation_series(conn)?
                .into_iter()
                .next()
                .map(|(id, _)| id)),
            d => Ok(d.series_id().map(str::to_string)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexPoint {
    pub date: NaiveDate,
    pub value: Decimal,
    pub estimated: bool,
}

fn parse_dec(s: &str) -> Result<Decimal> {
    s.parse::<Decimal>().map_err(|_| {
        Error::invalid(crate::i18n::t_args(
            "nucleo-ativos-erro-decimal-banco",
            &crate::i18n::args(&[("valor", s.to_string().into())]),
        ))
    })
}

/// Pontos da série no intervalo [from, to].
pub fn points(
    conn: &Connection,
    series_id: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<IndexPoint>> {
    let mut stmt = conn.prepare(
        "SELECT date, value, estimated FROM index_value WHERE series_id = ?1 AND date >= ?2 AND date <= ?3 ORDER BY date",
    )?;
    let rows = stmt
        .query_map(params![series_id, fmt_date(from), fmt_date(to)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.into_iter()
        .map(|(d, v, e)| {
            Ok(IndexPoint {
                date: parse_date(&d)?,
                value: parse_dec(&v)?,
                estimated: e == 1,
            })
        })
        .collect()
}

/// Os `n` últimos pontos até `until`, em ordem de data.
fn last_points(
    conn: &Connection,
    series_id: &str,
    until: NaiveDate,
    n: i64,
) -> Result<Vec<IndexPoint>> {
    let mut stmt = conn.prepare(
        "SELECT date, value, estimated FROM index_value WHERE series_id = ?1 AND date <= ?2 ORDER BY date DESC LIMIT ?3",
    )?;
    let rows = stmt
        .query_map(params![series_id, fmt_date(until), n], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut out = rows
        .into_iter()
        .map(|(d, v, e)| {
            Ok(IndexPoint {
                date: parse_date(&d)?,
                value: parse_dec(&v)?,
                estimated: e == 1,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    out.reverse();
    Ok(out)
}

/// Último nível publicado até `date` (no máximo 10 dias antes), para séries diárias de
/// pontos como Ibovespa e S&P 500.
pub fn level_on(conn: &Connection, series_id: &str, date: NaiveDate) -> Result<Option<Decimal>> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM index_value WHERE series_id = ?1 AND date <= ?2 AND date >= ?3 ORDER BY date DESC LIMIT 1",
            params![
                series_id,
                fmt_date(date),
                fmt_date(date - chrono::Duration::days(10))
            ],
            |r| r.get(0),
        )
        .optional()?;
    v.map(|s| parse_dec(&s)).transpose()
}

/// Variações mensais (séries `monthly_rate`, ex.: IPCA) cobrindo [from, to].
pub fn monthly_rates(
    conn: &Connection,
    series_id: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<MonthlyRate>> {
    let first = from.with_day(1).expect("dia 1");
    Ok(points(conn, series_id, first, to)?
        .into_iter()
        .map(|p| MonthlyRate {
            year: p.date.year(),
            month: p.date.month(),
            pct: p.value,
            estimated: p.estimated,
        })
        .collect())
}

/// Taxas diárias (séries `daily_rate`, ex.: CDI em % a.d.).
pub fn daily_rates(
    conn: &Connection,
    series_id: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<(NaiveDate, Decimal)>> {
    Ok(points(conn, series_id, from, to)?
        .into_iter()
        .map(|p| (p.date, p.value))
        .collect())
}

/// Quanto rendeu uma taxa anual (% a.a., diária ou mensal) entre `from` (exclusive) e `to`
/// (inclusive), dia a dia pela última taxa conhecida: (1 + taxa)^(1/365) por dia corrido.
/// `None` sem taxa até `from`.
pub fn annual_rate_growth(
    points: &[(NaiveDate, Decimal)],
    from: NaiveDate,
    to: NaiveDate,
) -> Option<f64> {
    use rust_decimal::prelude::ToPrimitive;
    if to <= from {
        return Some(0.0);
    }
    let mut i = points.partition_point(|(d, _)| *d <= from);
    let mut rate = points.get(i.checked_sub(1)?)?.1.to_f64()?;
    let mut acc = 1.0f64;
    let mut day = from;
    while day < to {
        day = day.succ_opt()?;
        while i < points.len() && points[i].0 <= day {
            rate = points[i].1.to_f64()?;
            i += 1;
        }
        acc *= (1.0 + rate / 100.0).powf(1.0 / 365.0);
    }
    Some(acc - 1.0)
}

/// O IPCA é o padrão das telas; fora do Brasil (docs/countries.md), vale a inflação do país.
pub fn effective_deflator(conn: &Connection, d: Deflator) -> Deflator {
    if d == Deflator::Ipca && !crate::country::is_brazil(conn) {
        Deflator::Country
    } else {
        d
    }
}

/// Fator de deflação entre duas datas: (1 + inflação no período). `None` sem dados.
/// IPCA usa variações mensais pró-rata; CPI e M2 usam níveis com interpolação geométrica
/// (mensais datados no 1º dia do mês de referência). `Country` usa a inflação do país de
/// referência e, se faltar dado, as alternativas do catálogo (ex.: Banco Mundial).
pub fn deflator_factor(
    conn: &Connection,
    deflator: Deflator,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Option<Factor>> {
    let deflator = effective_deflator(conn, deflator);
    if deflator == Deflator::None || to <= from {
        return Ok(Some(Factor {
            factor: Decimal::ONE,
            estimated: false,
        }));
    }
    if deflator == Deflator::Country {
        for (id, kind) in catalog::inflation_series(conn)? {
            if let Some(f) = series_factor_with(conn, &id, &kind, from, to, true)? {
                return Ok(Some(f));
            }
        }
        return Ok(None);
    }
    let series = deflator.series_id().expect("deflator com série");
    let kind = if matches!(deflator, Deflator::Ipca | Deflator::Igpm) {
        "monthly_rate"
    } else {
        "level"
    };
    series_factor(conn, series, kind, from, to)
}

/// Fator de uma série: `monthly_rate` acumula as variações mensais pró-rata; as demais
/// (níveis) dividem o nível final pelo inicial.
pub fn series_factor(
    conn: &Connection,
    series: &str,
    kind: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Option<Factor>> {
    series_factor_with(conn, series, kind, from, to, false)
}

/// Meses que um nível pode ser projetado além do último dado (só no deflator do país).
pub const MAX_PROJECTED_MONTHS: i32 = 24;

/// Como [`series_factor`]; com `stale_tail`, um nível cujo último dado ficou antes da
/// janela (inflação anual do Banco Mundial, publicada com atraso de mais de um ano) ainda
/// é projetado pela última variação mensal, por até [`MAX_PROJECTED_MONTHS`] meses.
fn series_factor_with(
    conn: &Connection,
    series: &str,
    kind: &str,
    from: NaiveDate,
    to: NaiveDate,
    stale_tail: bool,
) -> Result<Option<Factor>> {
    if to <= from {
        return Ok(Some(Factor {
            factor: Decimal::ONE,
            estimated: false,
        }));
    }
    if kind == "monthly_rate" {
        return Ok(inflation::accumulated_factor(
            &monthly_rates(conn, series, from, to)?,
            from,
            to,
        ));
    }
    // Níveis: inclui um mês antes e um depois para interpolar.
    let pts = points(
        conn,
        series,
        crate::time::add_months(from, -4),
        crate::time::add_months(to, 2),
    )?;
    let mut est = pts
        .iter()
        .any(|p| p.estimated && p.date >= from.with_day(1).expect("dia 1"));
    let mut levels: Vec<(NaiveDate, Decimal)> = pts.iter().map(|p| (p.date, p.value)).collect();
    if stale_tail && levels.len() < 2 {
        let tail = last_points(conn, series, crate::time::add_months(to, 2), 2)?;
        if tail.len() == 2 && crate::time::add_months(tail[1].date, MAX_PROJECTED_MONTHS) >= to {
            est |= tail.iter().any(|p| p.estimated);
            for p in tail {
                if !levels.iter().any(|(d, _)| *d == p.date) {
                    levels.push((p.date, p.value));
                }
            }
            levels.sort_by_key(|(d, _)| *d);
        }
    }
    // Meses ainda não publicados: repete a última variação mensal (estimado).
    if levels.len() >= 2 {
        let (d1, l1) = levels[levels.len() - 1];
        let (_, l0) = levels[levels.len() - 2];
        if d1 < to && !l0.is_zero() {
            let ratio = l1 / l0;
            let mut d = d1;
            let mut l = l1;
            while d < to {
                d = crate::time::add_months(d, 1);
                l *= ratio;
                levels.push((d, l));
            }
            est = true;
        }
    }
    Ok(inflation::level_factor(&levels, from, to).map(|f| Factor {
        factor: f,
        estimated: est,
    }))
}

/// Grava (ou atualiza) valores de uma série. Valores oficiais substituem estimados.
pub fn upsert_values(
    conn: &Connection,
    series_id: &str,
    values: &[(NaiveDate, Decimal, bool)],
    source: &str,
) -> Result<usize> {
    let now = now_utc();
    let mut stmt = conn.prepare(
        "INSERT INTO index_value (series_id, date, value, estimated, source, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (series_id, date) DO UPDATE SET value = excluded.value, estimated = excluded.estimated,
             source = excluded.source, fetched_at = excluded.fetched_at
         WHERE index_value.estimated = 1 OR excluded.estimated = 0",
    )?;
    let mut n = 0;
    for (d, v, est) in values {
        n += stmt.execute(params![
            series_id,
            fmt_date(*d),
            v.normalize().to_string(),
            *est as i64,
            source,
            now
        ])?;
    }
    conn.execute(
        "UPDATE index_series SET last_fetched_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![series_id, now],
    )?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn ipca_factor_and_official_replaces_estimate() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        upsert_values(
            c,
            "ipca",
            &[
                (d("2026-01-01"), dec!(0.5), false),
                (d("2026-02-01"), dec!(1.0), true),
            ],
            "test",
        )
        .unwrap();
        let f = deflator_factor(c, Deflator::Ipca, d("2026-01-01"), d("2026-03-01"))
            .unwrap()
            .unwrap();
        assert_eq!(f.factor, dec!(1.005) * dec!(1.01));
        assert!(f.estimated);
        // Official replaces estimated; estimated does not replace official.
        upsert_values(c, "ipca", &[(d("2026-02-01"), dec!(0.8), false)], "bcb").unwrap();
        upsert_values(c, "ipca", &[(d("2026-02-01"), dec!(9.9), true)], "focus").unwrap();
        let f = deflator_factor(c, Deflator::Ipca, d("2026-01-01"), d("2026-03-01"))
            .unwrap()
            .unwrap();
        assert_eq!(f.factor, dec!(1.005) * dec!(1.008));
        assert!(!f.estimated);
        assert!(
            deflator_factor(c, Deflator::Ipca, d("2026-01-01"), d("2026-06-01"))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            deflator_factor(c, Deflator::None, d("2020-01-01"), d("2026-01-01"))
                .unwrap()
                .unwrap()
                .factor,
            Decimal::ONE
        );
    }

    #[test]
    fn level_based_deflators() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        upsert_values(
            c,
            "us_cpi",
            &[
                (d("2026-01-01"), dec!(320), false),
                (d("2026-02-01"), dec!(321.6), false),
            ],
            "fred",
        )
        .unwrap();
        let f = deflator_factor(c, Deflator::UsCpi, d("2026-01-01"), d("2026-02-01"))
            .unwrap()
            .unwrap();
        assert_eq!(f.factor, dec!(1.005));
    }

    #[test]
    fn country_deflator_follows_reference_country() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        upsert_values(
            c,
            "ipca",
            &[
                (d("2026-01-01"), dec!(0.5), false),
                (d("2026-02-01"), dec!(1.0), false),
            ],
            "bcb",
        )
        .unwrap();
        // Brazil (default): identical to IPCA.
        let (from, to) = (d("2026-01-10"), d("2026-02-20"));
        assert_eq!(
            deflator_factor(c, Deflator::Country, from, to).unwrap(),
            deflator_factor(c, Deflator::Ipca, from, to).unwrap()
        );
        assert_eq!(
            Deflator::Country.resolve(c).unwrap().as_deref(),
            Some("ipca")
        );
        let ipca_before = deflator_factor(c, Deflator::Ipca, from, to).unwrap();

        // Portugal: HICP (level); without HICP in the period, falls back to the World Bank.
        catalog::set_reference_country(c, Some("PT")).unwrap();
        assert_eq!(
            Deflator::Country.resolve(c).unwrap().as_deref(),
            Some("hicp_pt")
        );
        assert!(
            deflator_factor(c, Deflator::Country, d("2026-01-01"), d("2026-02-01"))
                .unwrap()
                .is_none()
        );
        upsert_values(
            c,
            "wb_cpi_pt",
            &[
                (d("2025-12-01"), dec!(200), true),
                (d("2026-01-01"), dec!(201), true),
                (d("2026-02-01"), dec!(202), true),
            ],
            "worldbank:PT",
        )
        .unwrap();
        let wb = deflator_factor(c, Deflator::Country, d("2026-01-01"), d("2026-02-01"))
            .unwrap()
            .unwrap();
        assert!(wb.estimated);
        assert_eq!(wb.factor.round_dp(6), (dec!(202) / dec!(201)).round_dp(6));
        // World Bank lagging: the last data point is before the window and is projected by the
        // last monthly change (up to 24 months); the old deflator's US CPI does not project.
        let late = deflator_factor(c, Deflator::Country, d("2026-09-01"), d("2026-10-01"))
            .unwrap()
            .unwrap();
        assert!(late.estimated);
        assert_eq!(late.factor.round_dp(6), (dec!(202) / dec!(201)).round_dp(6));
        assert!(
            deflator_factor(c, Deflator::Country, d("2028-09-01"), d("2028-10-01"))
                .unwrap()
                .is_none()
        );
        upsert_values(
            c,
            "hicp_pt",
            &[
                (d("2026-01-01"), dec!(100), false),
                (d("2026-02-01"), dec!(101), false),
            ],
            "ecb:HICP/M.PT.N.000000.4D0.INX",
        )
        .unwrap();
        let f = deflator_factor(c, Deflator::Country, d("2026-01-01"), d("2026-02-01"))
            .unwrap()
            .unwrap();
        assert_eq!(f.factor, dec!(1.01));
        assert!(!f.estimated);
        // The earlier deflators do not change with the country.
        assert_eq!(
            deflator_factor(c, Deflator::Ipca, from, to).unwrap(),
            ipca_before
        );
        assert_eq!(
            serde_json::to_string(&Deflator::Country).unwrap(),
            "\"country\""
        );
    }
}
