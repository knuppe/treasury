//! Câmbio. Na fase 1 as taxas entram manualmente; na fase 2 chegam da PTAX (BCB).
//! Taxas são decimais exatos (`rust_decimal`) gravados como texto.

pub mod api;
pub mod exchange;

use std::str::FromStr;

use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::money::minor_units;
use crate::time::{now_utc, parse_date};

/// Taxas mais antigas que isto em relação à data pedida são marcadas como estimadas.
pub const STALE_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FxRate {
    pub id: Id,
    pub base: String,
    pub quote: String,
    pub date: String,
    /// Decimal exato em texto: 1 `base` = `rate` `quote`.
    pub rate: String,
    pub source: String,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FxRateInput {
    pub base: String,
    pub quote: String,
    pub date: String,
    pub rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct Converted {
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    /// Taxa efetivamente usada (de → para), decimal em texto.
    pub rate: String,
    pub rate_date: Option<String>,
    pub source: Option<String>,
    /// Taxa de outra data (mais de 7 dias antes) ou inversa/derivada.
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConvertArgs {
    #[ts(type = "number")]
    pub amount: i64,
    pub from: String,
    pub to: String,
    pub date: String,
}

fn parse_rate(s: &str) -> Result<Decimal> {
    let d = Decimal::from_str(s.trim().replace(',', ".").as_str()).map_err(|_| {
        Error::invalid(crate::i18n::t_args(
            "nucleo-contas-cambio-erro-taxa",
            &crate::i18n::args(&[("valor", s.to_string().into())]),
        ))
    })?;
    if d <= Decimal::ZERO {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-contas-cambio-erro-taxa-positiva",
        )));
    }
    Ok(d)
}

/// Taxa `from → to` na data (ou a última anterior). Tenta par direto e inverso.
pub fn rate(
    conn: &Connection,
    from: &str,
    to: &str,
    date: &str,
) -> Result<Option<(Decimal, String, String, bool)>> {
    if from == to {
        return Ok(Some((
            Decimal::ONE,
            date.to_string(),
            "identity".into(),
            false,
        )));
    }
    let lookup = |b: &str, q: &str| -> Result<Option<(String, String, String)>> {
        Ok(conn
            .query_row(
                "SELECT rate, date, source FROM fx_rate WHERE base = ?1 AND quote = ?2 AND date <= ?3
                 ORDER BY date DESC, CASE source WHEN 'bcb_ptax_venda' THEN 0 WHEN 'manual' THEN 2 ELSE 1 END LIMIT 1",
                params![b, q, date],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?)
    };
    let target = parse_date(date)?;
    let stale = |d: &str| {
        parse_date(d)
            .map(|rd| (target - rd).num_days() > STALE_DAYS)
            .unwrap_or(true)
    };
    if let Some((r, d, s)) = lookup(from, to)? {
        let est = stale(&d);
        return Ok(Some((parse_rate(&r)?, d, s, est)));
    }
    if let Some((r, d, s)) = lookup(to, from)? {
        let inv = Decimal::ONE / parse_rate(&r)?;
        let est = stale(&d);
        return Ok(Some((inv, d, s, est)));
    }
    // Sem o par: pela moeda do BCE ou pelo dólar (GBP→USD = GBP→EUR × EUR→USD). Família fora
    // do Brasil com conta em outra moeda (docs/countries.md).
    for pivot in ["EUR", "USD", "BRL"] {
        if pivot == from || pivot == to {
            continue;
        }
        let (Some(a), Some(b)) = (
            direct(conn, from, pivot, date)?,
            direct(conn, pivot, to, date)?,
        ) else {
            continue;
        };
        let older = if a.1 < b.1 { a.1.clone() } else { b.1.clone() };
        let est = stale(&older);
        return Ok(Some((a.0 * b.0, older, format!("{} × {}", a.2, b.2), est)));
    }
    Ok(None)
}

/// Câmbio direto ou inverso (sem pivô).
fn direct(
    conn: &Connection,
    from: &str,
    to: &str,
    date: &str,
) -> Result<Option<(Decimal, String, String)>> {
    let lookup = |b: &str, q: &str| -> Result<Option<(String, String, String)>> {
        Ok(conn
            .query_row(
                "SELECT rate, date, source FROM fx_rate WHERE base = ?1 AND quote = ?2 AND date <= ?3
                 ORDER BY date DESC, CASE source WHEN 'bcb_ptax_venda' THEN 0 WHEN 'manual' THEN 2 ELSE 1 END LIMIT 1",
                params![b, q, date],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?)
    };
    if let Some((r, d, s)) = lookup(from, to)? {
        return Ok(Some((parse_rate(&r)?, d, s)));
    }
    if let Some((r, d, s)) = lookup(to, from)? {
        return Ok(Some((Decimal::ONE / parse_rate(&r)?, d, s)));
    }
    Ok(None)
}

/// Converte um valor em menor unidade de `from` para `to`, arredondando metade para longe do zero.
pub fn convert(
    conn: &Connection,
    amount: i64,
    from: &str,
    to: &str,
    date: &str,
) -> Result<Option<Converted>> {
    let Some((r, rate_date, source, estimated)) = rate(conn, from, to, date)? else {
        return Ok(None);
    };
    Ok(Some(Converted {
        amount: apply_rate(amount, from, to, r)?,
        currency: to.to_string(),
        rate: r.normalize().to_string(),
        rate_date: Some(rate_date),
        source: Some(source),
        estimated,
    }))
}

pub fn apply_rate(amount: i64, from: &str, to: &str, r: Decimal) -> Result<i64> {
    let major = Decimal::from(amount) / Decimal::from(10i64.pow(minor_units(from)));
    let out = (major * r * Decimal::from(10i64.pow(minor_units(to))))
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    out.to_i64()
        .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-contas-cambio-erro-intervalo")))
}

pub fn set_manual(conn: &Connection, input: &FxRateInput) -> Result<FxRate> {
    let base = crate::money::validate_currency(&input.base)?;
    let quote = crate::money::validate_currency(&input.quote)?;
    if base == quote {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-contas-cambio-erro-moedas-iguais",
        )));
    }
    parse_date(&input.date)?;
    let r = parse_rate(&input.rate)?;
    let now = now_utc();
    let id = new_id();
    conn.execute(
        "INSERT INTO fx_rate (id, base, quote, date, rate, source, fetched_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'manual', ?6, ?6, ?6)
         ON CONFLICT (base, quote, date, source) DO UPDATE SET rate = excluded.rate, updated_at = excluded.updated_at",
        params![id, base, quote, input.date, r.normalize().to_string(), now],
    )?;
    let rec = conn.query_row(
        "SELECT id, base, quote, date, rate, source, fetched_at FROM fx_rate WHERE base = ?1 AND quote = ?2 AND date = ?3 AND source = 'manual'",
        params![base, quote, input.date],
        row_to_rate,
    )?;
    Ok(rec)
}

pub(crate) fn row_to_rate(r: &rusqlite::Row<'_>) -> rusqlite::Result<FxRate> {
    Ok(FxRate {
        id: r.get(0)?,
        base: r.get(1)?,
        quote: r.get(2)?,
        date: r.get(3)?,
        rate: r.get(4)?,
        source: r.get(5)?,
        fetched_at: r.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn converts_with_direct_inverse_and_stale_rates() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        set_manual(
            c,
            &FxRateInput {
                base: "USD".into(),
                quote: "BRL".into(),
                date: "2026-10-10".into(),
                rate: "5,4321".into(),
            },
        )
        .unwrap();
        // 100,00 USD → 543,21 BRL
        let v = convert(c, 10000, "USD", "BRL", "2026-10-14")
            .unwrap()
            .unwrap();
        assert_eq!(v.amount, 54321);
        assert!(!v.estimated);
        // Inverso: 543,21 BRL → 100,00 USD
        assert_eq!(
            convert(c, 54321, "BRL", "USD", "2026-10-14")
                .unwrap()
                .unwrap()
                .amount,
            10000
        );
        // Data muito posterior: estimado.
        assert!(
            convert(c, 100, "USD", "BRL", "2026-12-01")
                .unwrap()
                .unwrap()
                .estimated
        );
        // Antes de qualquer taxa: sem conversão.
        assert!(
            convert(c, 100, "USD", "BRL", "2026-01-01")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            convert(c, 123, "BRL", "BRL", "2026-01-01")
                .unwrap()
                .unwrap()
                .amount,
            123
        );
        // JPY sem casas decimais: 1.000 JPY a 0,0365 BRL = R$ 36,50
        set_manual(
            c,
            &FxRateInput {
                base: "JPY".into(),
                quote: "BRL".into(),
                date: "2026-10-10".into(),
                rate: "0.0365".into(),
            },
        )
        .unwrap();
        assert_eq!(
            convert(c, 1000, "JPY", "BRL", "2026-10-10")
                .unwrap()
                .unwrap()
                .amount,
            3650
        );
    }
}
