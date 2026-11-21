//! Taxas de referência para as calculadoras (docs/calculators.md): o juro básico do país de
//! referência, o CDI anualizado, a inflação de 12 meses e a poupança, cada uma com a data do
//! dado e a série de onde veio. Só leitura e só o que já foi coletado: sem dado, `None`, e a
//! calculadora pede a taxa à pessoa.

use chrono::{Datelike, Months, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::time::{fmt_date, parse_date};

/// Uma taxa anual de referência (decimal: 0,15 = 15% a.a.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RateReference {
    /// Nome da série (ex.: "Selic meta", "IPCA").
    pub name: String,
    pub series_id: String,
    pub rate: f64,
    /// Data do dado (para a inflação, o último mês oficial).
    pub date: String,
    /// Veio de projeção ou repetição do último dado.
    pub estimated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RateReferences {
    /// Juro básico do país de referência (Selic meta no Brasil).
    pub policy_rate: Option<RateReference>,
    /// CDI do último dia útil, anualizado por 252 dias (só Brasil).
    pub cdi: Option<RateReference>,
    /// Inflação acumulada nos 12 últimos meses oficiais.
    pub inflation_12m: Option<RateReference>,
    /// Poupança pela regra do Banco Central (só Brasil, sem a TR).
    pub savings: Option<RateReference>,
}

fn latest(conn: &Connection, series: &str) -> Result<Option<(String, f64, bool)>> {
    Ok(conn
        .query_row(
            "SELECT date, value, estimated FROM index_value
              WHERE series_id = ?1 AND estimated = 0 ORDER BY date DESC LIMIT 1",
            params![series],
            |r| {
                let v: String = r.get(1)?;
                Ok((r.get(0)?, v.parse::<f64>().unwrap_or(f64::NAN), r.get(2)?))
            },
        )
        .optional()?
        .filter(|(_, v, _)| v.is_finite()))
}

fn series_name(conn: &Connection, id: &str) -> Result<String> {
    Ok(conn
        .query_row("SELECT name FROM index_series WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?
        .unwrap_or_else(|| id.to_string()))
}

/// Poupança no Brasil: 0,5% ao mês com a Selic acima de 8,5% a.a.; senão 70% da Selic.
pub fn savings_rate(selic: f64) -> f64 {
    if selic > 0.085 {
        1.005_f64.powi(12) - 1.0
    } else {
        0.7 * selic
    }
}

pub fn references(conn: &Connection) -> Result<RateReferences> {
    let mut out = RateReferences::default();
    if let Some((name, id)) = super::catalog::reference_rate(conn)?
        && let Some((date, v, est)) = latest(conn, &id)?
    {
        out.policy_rate = Some(RateReference {
            name,
            series_id: id,
            rate: v / 100.0,
            date,
            estimated: est,
        });
    }
    let (country, _) = super::catalog::reference_country(conn)?;
    let brazil = country == "BR";
    if brazil && let Some((date, v, est)) = latest(conn, "cdi")? {
        out.cdi = Some(RateReference {
            name: series_name(conn, "cdi")?,
            series_id: "cdi".into(),
            rate: (1.0 + v / 100.0).powi(252) - 1.0,
            date,
            estimated: est,
        });
    }
    if brazil && let Some(p) = &out.policy_rate {
        out.savings = Some(RateReference {
            name: "poupanca".into(),
            series_id: p.series_id.clone(),
            rate: savings_rate(p.rate),
            date: p.date.clone(),
            estimated: p.estimated,
        });
    }
    // Inflação: a primeira série do país com o último mês oficial e os 11 anteriores.
    for (id, kind) in super::catalog::inflation_series(conn)? {
        let Some((last, _, _)) = latest(conn, &id)? else {
            continue;
        };
        let Ok(last) = parse_date(&last) else {
            continue;
        };
        let end = NaiveDate::from_ymd_opt(last.year(), last.month(), 1)
            .and_then(|d| d.checked_add_months(Months::new(1)));
        let Some(end) = end else { continue };
        let Some(start) = end.checked_sub_months(Months::new(12)) else {
            continue;
        };
        if let Some(f) = super::series_factor(conn, &id, &kind, start, end)?
            && let Some(rate) = f.factor.to_f64().map(|x| x - 1.0)
        {
            out.inflation_12m = Some(RateReference {
                name: series_name(conn, &id)?,
                series_id: id,
                rate,
                date: fmt_date(last),
                estimated: f.estimated,
            });
            break;
        }
    }
    Ok(out)
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
    fn savings_follows_the_central_bank_rule() {
        assert!((savings_rate(0.15) - 0.061_677_8).abs() < 1e-6);
        assert!((savings_rate(0.07) - 0.049).abs() < 1e-12);
    }

    #[test]
    fn references_come_from_the_collected_series_with_their_dates() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        // Nothing collected: everything empty (the calculator asks for the rate).
        assert_eq!(references(c).unwrap(), RateReferences::default());

        let months: Vec<_> = (1..=12)
            .map(|m| (d(&format!("2026-{m:02}-01")), dec!(0.4), false))
            .collect();
        crate::indices::upsert_values(c, "ipca", &months, "bcb").unwrap();
        // The Focus projection after the last official one does not count.
        crate::indices::upsert_values(c, "ipca", &[(d("2027-01-01"), dec!(9), true)], "focus")
            .unwrap();
        crate::indices::upsert_values(c, "cdi", &[(d("2026-10-30"), dec!(0.055131), false)], "bcb")
            .unwrap();
        crate::indices::upsert_values(
            c,
            "selic_meta",
            &[(d("2026-09-18"), dec!(15), false)],
            "bcb",
        )
        .unwrap();
        let r = references(c).unwrap();
        let inf = r.inflation_12m.expect("12-month IPCA");
        assert_eq!(inf.series_id, "ipca");
        assert_eq!(inf.date, "2026-12-01");
        assert!(
            (inf.rate - (1.004_f64.powi(12) - 1.0)).abs() < 1e-6,
            "{}",
            inf.rate
        );
        let cdi = r.cdi.expect("CDI");
        assert_eq!(cdi.date, "2026-10-30");
        assert!((cdi.rate - 0.149).abs() < 0.001, "{}", cdi.rate);
        let selic = r.policy_rate.expect("Selic");
        assert!((selic.rate - 0.15).abs() < 1e-12);
        assert_eq!(selic.date, "2026-09-18");
        assert!((r.savings.unwrap().rate - 0.061_677_8).abs() < 1e-6);
    }
}
