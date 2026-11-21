//! Radar de dividendos: projeção dos proventos dos próximos 12 meses a partir do histórico
//! do ativo (frequência e valor médio por evento), gravada em `dividend_event` com
//! `estimated = 1` e fonte `radar`. Não duplica proventos já anunciados: uma data projetada
//! perto (meio intervalo) de um anúncio real é descartada. Cada execução troca as projeções
//! anteriores do ativo.

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::dstr;
use crate::error::Result;
use crate::ids::new_id;
use crate::portfolio::AssetBrief;
use crate::time::{add_months, fmt_date, now_utc, parse_date};

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RadarArgs {
    /// Padrão: ativos da carteira e da watchlist.
    #[serde(default)]
    #[ts(optional)]
    pub asset_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RadarProjection {
    pub kind: String,
    pub ex_date: String,
    pub pay_date: Option<String>,
    /// Decimal em texto.
    pub amount_per_share: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RadarAsset {
    pub asset: AssetBrief,
    /// mensal, trimestral, semestral, anual.
    pub frequency: Option<String>,
    #[ts(type = "number | null")]
    pub interval_months: Option<i64>,
    /// Valor médio por evento usado na projeção (decimal texto).
    pub average: Option<String>,
    /// Proventos do histórico considerados (24 meses).
    #[ts(type = "number")]
    pub history_events: u64,
    pub projected: Vec<RadarProjection>,
    /// Datas projetadas descartadas por já haver provento anunciado perto.
    #[ts(type = "number")]
    pub skipped_announced: u64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RadarResult {
    pub horizon_end: String,
    pub assets: Vec<RadarAsset>,
    #[ts(type = "number")]
    pub written: u64,
    #[ts(type = "number")]
    pub removed: u64,
}

struct Event {
    kind: String,
    ex: NaiveDate,
    pay: Option<NaiveDate>,
    amount: Decimal,
}

fn history(conn: &Connection, asset_id: &str) -> Result<Vec<Event>> {
    let mut stmt = conn.prepare_cached(
        "SELECT kind, coalesce(ex_date, pay_date), pay_date, amount_per_share FROM dividend_event
         WHERE asset_id = ?1 AND estimated = 0 AND source <> 'radar' AND coalesce(ex_date, pay_date) IS NOT NULL
         ORDER BY 2",
    )?;
    let rows: Vec<(String, String, Option<String>, String)> = stmt
        .query_map([asset_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(kind, ex, pay, a)| {
            Some(Event {
                kind,
                ex: parse_date(&ex).ok()?,
                pay: pay.and_then(|p| parse_date(&p).ok()),
                amount: a.parse().ok()?,
            })
        })
        .collect())
}

fn median(mut v: Vec<i64>) -> Option<i64> {
    if v.is_empty() {
        return None;
    }
    v.sort();
    Some(v[v.len() / 2])
}

/// Projeção de um ativo (antes de gravar).
struct Projection {
    /// (meses entre proventos, rótulo).
    freq: Option<(i64, &'static str)>,
    avg: Option<Decimal>,
    history: u64,
    events: Vec<Event>,
    reason: Option<String>,
}

impl Projection {
    fn none(history: u64, reason: String) -> Self {
        Self {
            freq: None,
            avg: None,
            history,
            events: vec![],
            reason: Some(reason),
        }
    }
}

/// Projeta os proventos de um ativo (sem gravar).
fn project(events: &[Event], today: NaiveDate) -> Projection {
    let since = add_months(today, -24);
    let recent: Vec<&Event> = events
        .iter()
        .filter(|e| e.ex >= since && e.ex <= today)
        .collect();
    let n = recent.len() as u64;
    if recent.len() < 2 {
        return Projection::none(n, crate::i18n::t("nucleo-pesquisa-radar-poucos-proventos"));
    }
    // Intervalo típico entre datas-com distintas, em dias.
    let mut dates: Vec<NaiveDate> = recent.iter().map(|e| e.ex).collect();
    dates.dedup();
    let gaps: Vec<i64> = dates.windows(2).map(|w| (w[1] - w[0]).num_days()).collect();
    let Some(gap) = median(gaps) else {
        return Projection::none(n, crate::i18n::t("nucleo-pesquisa-radar-insuficiente"));
    };
    let (months, label) = match gap {
        0..=45 => (1, "mensal"),
        46..=120 => (3, "trimestral"),
        121..=240 => (6, "semestral"),
        _ => (12, "anual"),
    };
    // Valor médio por data-com nos últimos 12 meses (ou 24, se não houver).
    let last12: Vec<&&Event> = recent
        .iter()
        .filter(|e| e.ex > add_months(today, -12))
        .collect();
    let base: Vec<&Event> = if last12.len() >= 2 {
        last12.into_iter().copied().collect()
    } else {
        recent.clone()
    };
    let mut per_date: std::collections::BTreeMap<NaiveDate, Decimal> = Default::default();
    for e in &base {
        *per_date.entry(e.ex).or_default() += e.amount;
    }
    let avg =
        (per_date.values().copied().sum::<Decimal>() / Decimal::from(per_date.len())).round_dp(6);
    // Tipo mais comum e prazo típico entre data-com e pagamento.
    let mut kinds: std::collections::BTreeMap<&str, usize> = Default::default();
    for e in &recent {
        *kinds.entry(e.kind.as_str()).or_default() += 1;
    }
    let kind = kinds
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| "dividend".into());
    let pay_gap = median(
        recent
            .iter()
            .filter_map(|e| e.pay.map(|p| (p - e.ex).num_days()))
            .filter(|d| *d >= 0)
            .collect(),
    );
    // Continua depois do último provento real, inclusive os já anunciados para o futuro.
    let last = events
        .iter()
        .map(|e| e.ex)
        .max()
        .expect("2 ou mais eventos");
    let horizon = add_months(today, 12);
    let mut out = vec![];
    let mut k = 1;
    loop {
        let next = add_months(last, months * k);
        k += 1;
        if next <= today {
            continue;
        }
        if next > horizon {
            break;
        }
        out.push(Event {
            kind: kind.clone(),
            ex: next,
            pay: pay_gap.map(|g| next + chrono::Duration::days(g)),
            amount: avg,
        });
    }
    Projection {
        freq: Some((months as i64, label)),
        avg: Some(avg),
        history: n,
        events: out,
        reason: None,
    }
}

/// Projeta e grava o radar dos ativos pedidos.
pub fn run(conn: &Connection, asset_ids: &[String], today: NaiveDate) -> Result<RadarResult> {
    let horizon = add_months(today, 12);
    let mut res = RadarResult {
        horizon_end: fmt_date(horizon),
        assets: vec![],
        written: 0,
        removed: 0,
    };
    let now = now_utc();
    for id in asset_ids {
        let brief = crate::portfolio::records::asset_brief(conn, id)?;
        res.removed += conn.execute(
            "DELETE FROM dividend_event WHERE asset_id = ?1 AND estimated = 1 AND source = 'radar'",
            [id],
        )? as u64;
        let events = history(conn, id)?;
        let Projection {
            freq,
            avg,
            history: n,
            events: projected,
            reason,
        } = project(&events, today);
        // Anunciados (não estimados) a partir de hoje: a projeção não duplica.
        let half = freq.map(|(m, _)| (m * 30 / 2).max(10)).unwrap_or(10);
        let mut skipped = 0;
        let mut kept = vec![];
        for p in projected {
            let dup = events
                .iter()
                .any(|e| (e.ex - p.ex).num_days().abs() <= half);
            if dup {
                skipped += 1;
                continue;
            }
            res.written += conn.execute(
                "INSERT OR IGNORE INTO dividend_event (id, asset_id, kind, declared_date, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, 1, 'radar', ?8)",
                params![new_id(), id, p.kind, fmt_date(p.ex), p.pay.map(fmt_date), dstr(p.amount), brief.currency, now],
            )? as u64;
            kept.push(RadarProjection {
                kind: p.kind,
                ex_date: fmt_date(p.ex),
                pay_date: p.pay.map(fmt_date),
                amount_per_share: dstr(p.amount),
            });
        }
        res.assets.push(RadarAsset {
            asset: brief,
            frequency: freq.map(|(_, l)| l.to_string()),
            interval_months: freq.map(|(m, _)| m),
            average: avg.map(dstr),
            history_events: n,
            projected: kept,
            skipped_announced: skipped,
            reason,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::{Author, record};
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn projects_quarterly_without_duplicating_announced() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            tx.execute(
                "INSERT INTO asset (id, ticker, name, class, currency, created_at, updated_at) VALUES ('a1', 'XMPL3', 'Exemplo', 'stock_br', 'BRL', 'x', 'x')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        let c = db.conn();
        let ins = |ex: &str, pay: Option<&str>, amount: &str, est: i64, src: &str| {
            c.execute(
                "INSERT INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES (?1, 'a1', 'dividend', ?2, ?3, ?4, 'BRL', ?5, ?6, 'x')",
                params![new_id(), ex, pay, amount, est, src],
            )
            .unwrap();
        };
        // Quarterly: Oct/2025, Jan, Apr and Jul of 2026, payment 15 days later.
        ins("2025-10-15", Some("2025-10-30"), "0.40", 0, "yahoo");
        ins("2026-01-02", Some("2026-01-17"), "0.50", 0, "yahoo");
        ins("2026-04-01", Some("2026-04-16"), "0.50", 0, "yahoo");
        ins("2026-07-01", Some("2026-07-16"), "0.60", 0, "yahoo");
        // Already announced for 10/20 (future): the radar does not project October again and
        // continues the sequence from it.
        ins("2026-10-20", Some("2026-11-04"), "0.70", 0, "yahoo");
        // Old radar projection: it will be replaced.
        ins("2026-12-01", None, "9.99", 1, "radar");
        let today = d("2026-10-14");
        let r = run(c, &["a1".to_string()], today).unwrap();
        let a = &r.assets[0];
        assert_eq!(a.frequency.as_deref(), Some("trimestral"));
        assert_eq!(r.removed, 1);
        // Average of the events of the last 12 months: (0.40 + 0.50 + 0.50 + 0.60) ÷ 4 = 0.50.
        assert_eq!(a.average.as_deref(), Some("0.5"));
        // From the announced one (10/20/2026): 1/20, 4/20 and 7/20/2027 (10/20/2027 is past the
        // 12 months).
        let dates: Vec<&str> = a.projected.iter().map(|p| p.ex_date.as_str()).collect();
        assert_eq!(dates, vec!["2027-01-20", "2027-04-20", "2027-07-20"]);
        assert_eq!(a.projected[0].pay_date.as_deref(), Some("2027-02-04"));
        assert_eq!(r.written, 3);
        // Running again does not duplicate anything.
        let r2 = run(c, &["a1".to_string()], today).unwrap();
        assert_eq!((r2.removed, r2.written), (3, 3));
        let n: i64 = c
            .query_row(
                "SELECT count(*) FROM dividend_event WHERE asset_id = 'a1' AND source = 'radar'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 3);
        // The agenda shows the projections as estimated.
        let cal = crate::portfolio::engine::dividend_calendar(
            c,
            &crate::portfolio::DividendCalendarArgs {
                from: "2027-01-01".into(),
                to: "2027-12-31".into(),
                only_mine: Some(false),
            },
        )
        .unwrap();
        assert!(cal.iter().all(|e| e.estimated && e.source == "radar"));
        assert_eq!(cal.len(), 3);
    }

    #[test]
    fn needs_history() {
        let ev = vec![Event {
            kind: "dividend".into(),
            ex: d("2026-05-01"),
            pay: None,
            amount: Decimal::ONE,
        }];
        let Projection {
            freq: f,
            history: n,
            events: p,
            reason,
            ..
        } = project(&ev, d("2026-10-14"));
        assert!(f.is_none() && p.is_empty());
        assert_eq!(n, 1);
        assert!(reason.unwrap().contains("insuficiente"));
    }
}
