//! Dados de mercado: cotações diárias com fonte e data de coleta. Os adaptadores (B3
//! COTAHIST, brapi, Yahoo, CoinGecko, Tesouro Transparente, CVM) gravam com
//! [`upsert_prices`]; a carteira lê com [`close_on`].

pub mod api;
pub mod cvm;
pub mod defillama;
pub mod eodhd;
pub mod exchange;
pub mod intraday;
pub mod search;
pub mod sync;
pub mod tesouro;
pub mod tiingo;
pub mod yahoo;
pub mod zone;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::time::{fmt_date, now_utc, parse_date};

#[derive(Debug, Clone, PartialEq)]
pub struct PriceRow {
    pub date: NaiveDate,
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    /// Fechamento não ajustado por proventos.
    pub close: Decimal,
    pub adj_close: Option<Decimal>,
    pub volume: Option<Decimal>,
}

/// Preço usado numa avaliação, com origem ("medido, nunca inventado").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PriceOn {
    /// Decimal em texto.
    pub close: String,
    pub date: String,
    pub source: String,
    /// Cotação de outro dia (a mais recente antes da data pedida).
    pub stale: bool,
    /// Cotação intradiária (Yahoo, ~15 min de atraso) usada no lugar do fechamento de hoje.
    pub intraday: Option<intraday::IntradayMark>,
}

fn bad_stored_price(v: &str) -> Error {
    Error::invalid(crate::i18n::t_args(
        "nucleo-ativos-erro-preco-banco",
        &crate::i18n::args(&[("preco", v.to_string().into())]),
    ))
}

/// Fechamento na data ou o último anterior.
pub fn close_on(
    conn: &Connection,
    asset_id: &str,
    date: NaiveDate,
) -> Result<Option<(Decimal, PriceOn)>> {
    let row: Option<(String, String, String)> = conn
        .query_row(
            "SELECT close, date, source FROM price WHERE asset_id = ?1 AND date <= ?2 ORDER BY date DESC LIMIT 1",
            params![asset_id, fmt_date(date)],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((close, d, source)) = row else {
        return Ok(None);
    };
    let v: Decimal = close.parse().map_err(|_| bad_stored_price(&close))?;
    let stale = parse_date(&d)? != date;
    Ok(Some((
        v,
        PriceOn {
            close: v.normalize().to_string(),
            date: d,
            source,
            stale,
            intraday: None,
        },
    )))
}

/// Preço para avaliar uma posição na data: o fechamento diário (ou o último anterior) e,
/// para hoje, a cotação intradiária do Yahoo quando ela é mais nova que o último fechamento.
/// Datas passadas usam só os fechamentos (a fonte do histórico).
pub fn price_on(
    conn: &Connection,
    asset_id: &str,
    date: NaiveDate,
) -> Result<Option<(Decimal, PriceOn)>> {
    let daily = close_on(conn, asset_id, date)?;
    if date < crate::time::today_local() {
        return Ok(daily);
    }
    let Some(q) = intraday::quote(conn, asset_id)? else {
        return Ok(daily);
    };
    let qd = parse_date(&q.market_date)?;
    // A bolsa pode estar um dia à frente (Tóquio, Sydney), nunca mais.
    if qd > date + chrono::Duration::days(1) {
        return Ok(daily);
    }
    if let Some((_, p)) = &daily
        && parse_date(&p.date)? > qd
    {
        return Ok(daily);
    }
    let v: Decimal = q.price.parse().map_err(|_| bad_stored_price(&q.price))?;
    Ok(Some((
        v,
        PriceOn {
            close: v.normalize().to_string(),
            date: q.market_date.clone(),
            source: q.source.clone(),
            stale: qd < date,
            intraday: Some(intraday::IntradayMark {
                quoted_at: q.market_time.clone(),
                previous_close: q.previous_close.clone(),
                change_pct: q.change_pct,
                market_state: q.market_state,
                delay_minutes: q.delay_minutes,
            }),
        },
    )))
}

pub fn upsert_prices(
    conn: &Connection,
    asset_id: &str,
    rows: &[PriceRow],
    source: &str,
) -> Result<usize> {
    let now = now_utc();
    let mut stmt = conn.prepare(
        "INSERT INTO price (asset_id, date, open, high, low, close, adj_close, volume, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT (asset_id, date) DO UPDATE SET open = excluded.open, high = excluded.high, low = excluded.low,
             close = excluded.close, adj_close = excluded.adj_close, volume = excluded.volume,
             source = excluded.source, fetched_at = excluded.fetched_at",
    )?;
    let s = |v: &Option<Decimal>| v.map(|d| d.normalize().to_string());
    let mut n = 0;
    for r in rows {
        n += stmt.execute(params![
            asset_id,
            fmt_date(r.date),
            s(&r.open),
            s(&r.high),
            s(&r.low),
            r.close.normalize().to_string(),
            s(&r.adj_close),
            s(&r.volume),
            source,
            now
        ])?;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;
    use rust_decimal_macros::dec;

    #[test]
    fn close_on_uses_last_known() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        c.execute(
            "INSERT INTO asset (id, ticker, name, class, market, currency, created_at, updated_at)
             VALUES ('a1', 'PETR4', 'Petrobras PN', 'stock_br', 'B3', 'BRL', 'x', 'x')",
            [],
        )
        .unwrap();
        let row = |d: &str, c: Decimal| PriceRow {
            date: parse_date(d).unwrap(),
            open: None,
            high: None,
            low: None,
            close: c,
            adj_close: None,
            volume: None,
        };
        upsert_prices(
            c,
            "a1",
            &[
                row("2026-10-09", dec!(38.10)),
                row("2026-10-13", dec!(38.55)),
            ],
            "b3_cotahist",
        )
        .unwrap();
        let (v, p) = close_on(c, "a1", parse_date("2026-10-13").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(v, dec!(38.55));
        assert!(!p.stale);
        let (v, p) = close_on(c, "a1", parse_date("2026-10-12").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(v, dec!(38.10));
        assert!(p.stale);
        assert_eq!(p.source, "b3_cotahist");
        assert!(
            close_on(c, "a1", parse_date("2026-01-01").unwrap())
                .unwrap()
                .is_none()
        );
    }
}
