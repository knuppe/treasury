//! Datas e instantes. Datas de lançamento são `YYYY-MM-DD` sem fuso; instantes técnicos
//! são UTC em RFC 3339 com milissegundos.

use chrono::{Datelike, NaiveDate, SecondsFormat, Utc};

use crate::error::{Error, Result};
use crate::i18n::{args, t_args};

pub fn now_utc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn today_local() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        Error::invalid(t_args(
            "nucleo-base-erro-data",
            &args(&[("valor", s.to_string().into())]),
        ))
    })
}

pub fn fmt_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// Mês no formato `YYYY-MM`.
pub fn parse_month(s: &str) -> Result<(i32, u32)> {
    let err = || {
        Error::invalid(t_args(
            "nucleo-base-erro-mes",
            &args(&[("valor", s.to_string().into())]),
        ))
    };
    let (y, m) = s.split_once('-').ok_or_else(err)?;
    let y: i32 = y.parse().map_err(|_| err())?;
    let m: u32 = m.parse().map_err(|_| err())?;
    if !(1..=12).contains(&m) || !(1900..=9999).contains(&y) {
        return Err(err());
    }
    Ok((y, m))
}

pub fn fmt_month(y: i32, m: u32) -> String {
    format!("{y:04}-{m:02}")
}

pub fn month_of(d: NaiveDate) -> String {
    fmt_month(d.year(), d.month())
}

/// Primeiro e último dia do mês.
pub fn month_bounds(y: i32, m: u32) -> (NaiveDate, NaiveDate) {
    let first = NaiveDate::from_ymd_opt(y, m, 1).expect("mês válido");
    let next = add_months(first, 1);
    (first, next.pred_opt().expect("data válida"))
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    month_bounds(y, m).1.day()
}

/// Soma meses mantendo o dia, limitado ao último dia do mês de destino
/// (31/01 + 1 mês = 28/02 ou 29/02).
pub fn add_months(d: NaiveDate, months: i32) -> NaiveDate {
    let total = d.year() * 12 + d.month0() as i32 + months;
    let y = total.div_euclid(12);
    let m = total.rem_euclid(12) as u32 + 1;
    let day = d.day().min(days_in_month_raw(y, m));
    NaiveDate::from_ymd_opt(y, m, day).expect("data válida")
}

/// Data com o dia limitado ao último dia do mês.
pub fn clamp_day(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day.clamp(1, days_in_month_raw(y, m))).expect("data válida")
}

fn days_in_month_raw(y: i32, m: u32) -> u32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .expect("data válida")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_months_clamps_to_month_end() {
        let d = parse_date("2026-01-31").unwrap();
        assert_eq!(fmt_date(add_months(d, 1)), "2026-02-28");
        assert_eq!(fmt_date(add_months(d, 13)), "2027-02-28");
        assert_eq!(
            fmt_date(add_months(parse_date("2024-01-31").unwrap(), 1)),
            "2024-02-29"
        );
        assert_eq!(fmt_date(add_months(d, -2)), "2025-11-30");
    }

    #[test]
    fn errors_follow_the_locale() {
        use crate::i18n::{Locale, set_current};
        set_current(Locale::En);
        let date = parse_date("31/12/2026").unwrap_err().message();
        let month = parse_month("2026-13").unwrap_err().message();
        let amount = crate::money::parse_amount("abc", "BRL")
            .unwrap_err()
            .message();
        set_current(Locale::PtBr);
        assert_eq!(date, "invalid date: 31/12/2026");
        assert_eq!(month, "invalid month (use YYYY-MM): 2026-13");
        assert_eq!(amount, "invalid amount: abc");
        assert_eq!(parse_date("x").unwrap_err().message(), "data inválida: x");
    }

    #[test]
    fn month_parsing() {
        assert_eq!(parse_month("2026-10").unwrap(), (2026, 10));
        assert!(parse_month("2026-13").is_err());
        assert!(parse_month("abc").is_err());
        let (a, b) = month_bounds(2026, 2);
        assert_eq!(
            (fmt_date(a).as_str(), fmt_date(b).as_str()),
            ("2026-02-01", "2026-02-28")
        );
    }
}
