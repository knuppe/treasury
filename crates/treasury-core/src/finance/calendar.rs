//! Calendário de dias úteis brasileiro (feriados nacionais, como o calendário ANBIMA/B3):
//! base para renda fixa (base 252) e contagens do mercado.

use chrono::{Datelike, Duration, NaiveDate, Weekday};

/// Domingo de Páscoa (algoritmo de Meeus/Jones/Butcher).
pub fn easter(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).expect("data de Páscoa válida")
}

/// Feriados nacionais que fecham o mercado financeiro no ano (inclui Carnaval e Corpus
/// Christi, como a ANBIMA; Consciência Negra a partir de 2024).
pub fn national_holidays(year: i32) -> Vec<(NaiveDate, &'static str)> {
    let fixed = |m, d| NaiveDate::from_ymd_opt(year, m, d).expect("data válida");
    let e = easter(year);
    let mut v = vec![
        (fixed(1, 1), "Confraternização Universal"),
        (e - Duration::days(48), "Carnaval"),
        (e - Duration::days(47), "Carnaval"),
        (e - Duration::days(2), "Sexta-feira Santa"),
        (fixed(4, 21), "Tiradentes"),
        (fixed(5, 1), "Dia do Trabalho"),
        (e + Duration::days(60), "Corpus Christi"),
        (fixed(9, 7), "Independência do Brasil"),
        (fixed(10, 12), "Nossa Senhora Aparecida"),
        (fixed(11, 2), "Finados"),
        (fixed(11, 15), "Proclamação da República"),
        (fixed(12, 25), "Natal"),
    ];
    if year >= 2024 {
        v.push((
            fixed(11, 20),
            "Dia Nacional de Zumbi e da Consciência Negra",
        ));
    }
    v.sort();
    v
}

pub fn is_holiday(d: NaiveDate) -> bool {
    national_holidays(d.year()).iter().any(|(h, _)| *h == d)
}

pub fn is_business_day(d: NaiveDate) -> bool {
    !matches!(d.weekday(), Weekday::Sat | Weekday::Sun) && !is_holiday(d)
}

/// Dias úteis no intervalo (início, fim] — convenção de mercado para "du" entre duas datas.
pub fn business_days_between(start: NaiveDate, end: NaiveDate) -> i64 {
    if end == start {
        return 0;
    }
    if end < start {
        return -business_days_between(end, start);
    }
    let mut n = 0;
    let mut d = start.succ_opt().expect("data válida");
    while d <= end {
        if is_business_day(d) {
            n += 1;
        }
        d = d.succ_opt().expect("data válida");
    }
    n
}

/// Próximo dia útil a partir de `d` (inclusive).
pub fn next_business_day(mut d: NaiveDate) -> NaiveDate {
    while !is_business_day(d) {
        d = d.succ_opt().expect("data válida");
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn easter_dates() {
        assert_eq!(easter(2024), d("2024-03-31"));
        assert_eq!(easter(2025), d("2025-04-20"));
        assert_eq!(easter(2026), d("2026-04-05"));
        assert_eq!(easter(2027), d("2027-03-28"));
    }

    #[test]
    fn holidays_2026() {
        let h: Vec<NaiveDate> = national_holidays(2026)
            .into_iter()
            .map(|(d, _)| d)
            .collect();
        for s in [
            "2026-02-16",
            "2026-02-17",
            "2026-04-03",
            "2026-06-04",
            "2026-11-20",
        ] {
            assert!(h.contains(&d(s)), "{s}");
        }
        assert!(!is_business_day(d("2026-10-12")));
        assert!(is_business_day(d("2026-10-14")));
        assert!(!is_business_day(d("2026-10-17")), "sábado");
    }

    #[test]
    fn business_days_in_a_year() {
        // 2026: 261 dias de semana − 12 feriados em dias de semana = 249.
        assert_eq!(business_days_between(d("2025-12-31"), d("2026-12-31")), 249);
        assert_eq!(
            business_days_between(d("2026-10-09"), d("2026-10-13")),
            1,
            "sexta → terça com feriado na segunda"
        );
        assert_eq!(business_days_between(d("2026-10-13"), d("2026-10-09")), -1);
        assert_eq!(next_business_day(d("2026-10-10")), d("2026-10-13"));
        assert_eq!(business_days_between(d("2026-10-14"), d("2026-10-14")), 0);
    }
}
