//! Subconjunto de RRULE (RFC 5545): `FREQ`, `INTERVAL`, `BYMONTHDAY`, `BYDAY` (sem
//! prefixo numérico), `BYMONTH` e `COUNT`. Parser, serializador, geração de datas,
//! descrição legível (no idioma da instalação) e valor mensal equivalente.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use fluent_bundle::FluentValue;

use super::{Freq, RuleSpec};
use crate::error::{Error, Result};
use crate::i18n::{self, args, t, t_args};
use crate::time::{add_months, clamp_day};
use crate::transactions::util::div_round;

/// Limite de segurança de períodos percorridos na geração de datas.
const MAX_PERIODS: i64 = 200_000;
pub const MAX_COUNT: u32 = 10_000;
pub const MAX_INTERVAL: u32 = 999;

const DAYS: [(&str, Weekday); 7] = [
    ("MO", Weekday::Mon),
    ("TU", Weekday::Tue),
    ("WE", Weekday::Wed),
    ("TH", Weekday::Thu),
    ("FR", Weekday::Fri),
    ("SA", Weekday::Sat),
    ("SU", Weekday::Sun),
];

fn weekday(code: &str) -> Result<Weekday> {
    let c = code.trim().to_ascii_uppercase();
    if c.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit() || ch == '-' || ch == '+')
    {
        return Err(invalid_with(
            "nucleo-lancamentos-rrule-erro-byday-posicao",
            "valor",
            code,
        ));
    }
    DAYS.iter()
        .find(|(k, _)| *k == c)
        .map(|(_, d)| *d)
        .ok_or_else(|| invalid_with("nucleo-lancamentos-rrule-erro-dia-semana", "valor", code))
}

fn code(d: Weekday) -> &'static str {
    DAYS.iter()
        .find(|(_, w)| *w == d)
        .map(|(k, _)| *k)
        .expect("dia válido")
}

fn freq_str(f: Freq) -> &'static str {
    match f {
        Freq::Daily => "DAILY",
        Freq::Weekly => "WEEKLY",
        Freq::Monthly => "MONTHLY",
        Freq::Yearly => "YEARLY",
    }
}

/// Erro do catálogo com um argumento de texto.
fn invalid_with(key: &str, name: &'static str, value: &str) -> Error {
    Error::invalid(t_args(key, &args(&[(name, value.to_string().into())])))
}

fn parse_u32(key: &str, v: &str) -> Result<u32> {
    v.trim().parse().map_err(|_| {
        Error::invalid(t_args(
            "nucleo-lancamentos-rrule-erro-valor",
            &args(&[
                ("parte", key.to_string().into()),
                ("valor", v.to_string().into()),
            ]),
        ))
    })
}

/// Interpreta o texto RRULE (aceita o prefixo `RRULE:`).
pub fn parse(s: &str) -> Result<RuleSpec> {
    let body = s.trim();
    let body = body
        .strip_prefix("RRULE:")
        .or_else(|| body.strip_prefix("rrule:"))
        .unwrap_or(body);
    let mut freq = None;
    let mut spec = RuleSpec {
        freq: Freq::Monthly,
        interval: None,
        by_month_day: None,
        by_day: None,
        by_month: None,
        count: None,
    };
    for part in body.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| invalid_with("nucleo-lancamentos-rrule-erro-parte", "parte", part))?;
        let key = k.trim().to_ascii_uppercase();
        match key.as_str() {
            "FREQ" => {
                freq = Some(match v.trim().to_ascii_uppercase().as_str() {
                    "DAILY" => Freq::Daily,
                    "WEEKLY" => Freq::Weekly,
                    "MONTHLY" => Freq::Monthly,
                    "YEARLY" => Freq::Yearly,
                    other => {
                        return Err(invalid_with(
                            "nucleo-lancamentos-rrule-erro-frequencia",
                            "valor",
                            other,
                        ));
                    }
                })
            }
            "INTERVAL" => spec.interval = Some(parse_u32(&key, v)?),
            "BYMONTHDAY" => {
                if v.contains(',') {
                    return Err(Error::invalid(t(
                        "nucleo-lancamentos-rrule-erro-varios-dias",
                    )));
                }
                spec.by_month_day = Some(parse_u32(&key, v)?);
            }
            "BYMONTH" => {
                if v.contains(',') {
                    return Err(Error::invalid(t(
                        "nucleo-lancamentos-rrule-erro-varios-meses",
                    )));
                }
                spec.by_month = Some(parse_u32(&key, v)?);
            }
            "BYDAY" => {
                spec.by_day = Some(
                    v.split(',')
                        .map(|d| weekday(d).map(|w| code(w).to_string()))
                        .collect::<Result<Vec<_>>>()?,
                )
            }
            "COUNT" => spec.count = Some(parse_u32(&key, v)?),
            "WKST" if v.trim().eq_ignore_ascii_case("MO") => {}
            other => {
                return Err(invalid_with(
                    "nucleo-lancamentos-rrule-erro-parte-nao-suportada",
                    "parte",
                    other,
                ));
            }
        }
    }
    spec.freq = freq.ok_or_else(|| Error::invalid(t("nucleo-lancamentos-rrule-erro-sem-freq")))?;
    validate(&spec)?;
    Ok(spec)
}

/// Texto RRULE canônico: `FREQ;INTERVAL;BYMONTH;BYMONTHDAY;BYDAY;COUNT`.
pub fn to_rrule(spec: &RuleSpec) -> String {
    let mut parts = vec![
        format!("FREQ={}", freq_str(spec.freq)),
        format!("INTERVAL={}", spec.interval.unwrap_or(1)),
    ];
    if let Some(m) = spec.by_month {
        parts.push(format!("BYMONTH={m}"));
    }
    if let Some(d) = spec.by_month_day {
        parts.push(format!("BYMONTHDAY={d}"));
    }
    if let Some(days) = spec.by_day.as_ref().filter(|d| !d.is_empty()) {
        parts.push(format!("BYDAY={}", days.join(",")));
    }
    if let Some(c) = spec.count {
        parts.push(format!("COUNT={c}"));
    }
    parts.join(";")
}

fn days_of(spec: &RuleSpec) -> Result<Vec<Weekday>> {
    let mut out: Vec<Weekday> = spec
        .by_day
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|d| weekday(d))
        .collect::<Result<_>>()?;
    out.sort_by_key(|d| d.num_days_from_monday());
    out.dedup();
    Ok(out)
}

pub fn validate(spec: &RuleSpec) -> Result<()> {
    let interval = spec.interval.unwrap_or(1);
    if !(1..=MAX_INTERVAL).contains(&interval) {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-rrule-erro-intervalo",
            &args(&[("max", MAX_INTERVAL.into())]),
        )));
    }
    if let Some(c) = spec.count
        && !(1..=MAX_COUNT).contains(&c)
    {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-rrule-erro-ocorrencias",
            &args(&[("max", MAX_COUNT.into())]),
        )));
    }
    if let Some(d) = spec.by_month_day {
        if !(1..=31).contains(&d) {
            return Err(Error::invalid(t("nucleo-lancamentos-rrule-erro-dia-mes")));
        }
        if !matches!(spec.freq, Freq::Monthly | Freq::Yearly) {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-rrule-erro-dia-mes-frequencia",
            )));
        }
    }
    if let Some(m) = spec.by_month {
        if !(1..=12).contains(&m) {
            return Err(Error::invalid(t("nucleo-lancamentos-rrule-erro-mes")));
        }
        if spec.freq != Freq::Yearly {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-rrule-erro-mes-frequencia",
            )));
        }
    }
    if let Some(days) = &spec.by_day {
        if days.is_empty() {
            return Err(Error::invalid(t("nucleo-lancamentos-rrule-erro-sem-dias")));
        }
        if !matches!(spec.freq, Freq::Weekly | Freq::Daily) {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-rrule-erro-dias-frequencia",
            )));
        }
        days_of(spec)?;
    }
    Ok(())
}

/// Preenche o que a regra deixa implícito a partir da data inicial (dia do mês, mês,
/// dia da semana) e ordena os dias da semana.
pub fn normalize(spec: &RuleSpec, start: NaiveDate) -> Result<RuleSpec> {
    validate(spec)?;
    let mut s = spec.clone();
    s.interval = Some(spec.interval.unwrap_or(1));
    match s.freq {
        Freq::Daily => {
            if s.by_day.is_some() {
                s.by_day = Some(days_of(spec)?.into_iter().map(|d| code(d).into()).collect());
            }
        }
        Freq::Weekly => {
            let days = if s.by_day.is_some() {
                days_of(spec)?
            } else {
                vec![start.weekday()]
            };
            s.by_day = Some(days.into_iter().map(|d| code(d).into()).collect());
        }
        Freq::Monthly => s.by_month_day = Some(s.by_month_day.unwrap_or(start.day())),
        Freq::Yearly => {
            s.by_month = Some(s.by_month.unwrap_or(start.month()));
            s.by_month_day = Some(s.by_month_day.unwrap_or(start.day()));
        }
    }
    Ok(s)
}

/// Percorre as ocorrências a partir de `start`, em ordem, respeitando `COUNT` e
/// `end`. `f` devolve `false` para parar.
fn walk(
    spec: &RuleSpec,
    start: NaiveDate,
    end: Option<NaiveDate>,
    mut f: impl FnMut(NaiveDate) -> bool,
) -> Result<()> {
    let s = normalize(spec, start)?;
    let interval = i64::from(s.interval.unwrap_or(1));
    let mut emitted: u32 = 0;
    // Devolve false quando deve parar.
    let mut emit = |d: NaiveDate| -> bool {
        if end.is_some_and(|e| d > e) {
            return false;
        }
        if s.count.is_some_and(|c| emitted >= c) {
            return false;
        }
        emitted += 1;
        f(d)
    };
    let days = days_of(&s)?;
    match s.freq {
        Freq::Daily => {
            for k in 0..MAX_PERIODS {
                let Some(d) = start.checked_add_signed(Duration::days(k * interval)) else {
                    break;
                };
                if d.year() > 9999 {
                    break;
                }
                if !days.is_empty() && !days.contains(&d.weekday()) {
                    if end.is_some_and(|e| d > e) {
                        break;
                    }
                    continue;
                }
                if !emit(d) {
                    break;
                }
            }
        }
        Freq::Weekly => {
            let week0 = start - Duration::days(i64::from(start.weekday().num_days_from_monday()));
            'weeks: for k in 0..MAX_PERIODS {
                let Some(monday) = week0.checked_add_signed(Duration::weeks(k * interval)) else {
                    break;
                };
                if monday.year() > 9999 {
                    break;
                }
                for wd in &days {
                    let d = monday + Duration::days(i64::from(wd.num_days_from_monday()));
                    if d < start {
                        continue;
                    }
                    if !emit(d) {
                        break 'weeks;
                    }
                }
            }
        }
        Freq::Monthly => {
            let day = s.by_month_day.unwrap_or(start.day());
            let first = start.with_day(1).expect("dia 1");
            for k in 0..MAX_PERIODS {
                if i64::from(first.year()) + k * interval / 12 > 9999 {
                    break;
                }
                let m = add_months(first, (k * interval) as i32);
                let d = clamp_day(m.year(), m.month(), day);
                if d < start {
                    continue;
                }
                if !emit(d) {
                    break;
                }
            }
        }
        Freq::Yearly => {
            let month = s.by_month.unwrap_or(start.month());
            let day = s.by_month_day.unwrap_or(start.day());
            for k in 0..MAX_PERIODS {
                let Ok(y) = i32::try_from(i64::from(start.year()) + k * interval) else {
                    break;
                };
                if y > 9999 {
                    break;
                }
                let d = clamp_day(y, month, day);
                if d < start {
                    continue;
                }
                if !emit(d) {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// Ocorrências entre `from` e `until` (inclusive).
pub fn occurrences_between(
    spec: &RuleSpec,
    start: NaiveDate,
    end: Option<NaiveDate>,
    from: NaiveDate,
    until: NaiveDate,
) -> Result<Vec<NaiveDate>> {
    let mut out = Vec::new();
    walk(spec, start, end, |d| {
        if d > until {
            return false;
        }
        if d >= from {
            out.push(d);
        }
        true
    })?;
    Ok(out)
}

/// As próximas `n` ocorrências a partir de `from` (inclusive).
pub fn next_n(
    spec: &RuleSpec,
    start: NaiveDate,
    end: Option<NaiveDate>,
    from: NaiveDate,
    n: usize,
) -> Result<Vec<NaiveDate>> {
    let mut out = Vec::new();
    if n == 0 {
        return Ok(out);
    }
    walk(spec, start, end, |d| {
        if d >= from {
            out.push(d);
        }
        out.len() < n
    })?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Descrição legível, no idioma da instalação (`nucleo-lancamentos-rrule-*`)

/// (chave do dia no catálogo, feminino em português?). O gênero escolhe "às"/"aos" no pt-BR;
/// os outros idiomas ignoram.
fn day_key(d: Weekday) -> (&'static str, bool) {
    match d {
        Weekday::Mon => ("nucleo-lancamentos-rrule-dia-mo", true),
        Weekday::Tue => ("nucleo-lancamentos-rrule-dia-tu", true),
        Weekday::Wed => ("nucleo-lancamentos-rrule-dia-we", true),
        Weekday::Thu => ("nucleo-lancamentos-rrule-dia-th", true),
        Weekday::Fri => ("nucleo-lancamentos-rrule-dia-fr", true),
        Weekday::Sat => ("nucleo-lancamentos-rrule-dia-sa", false),
        Weekday::Sun => ("nucleo-lancamentos-rrule-dia-su", false),
    }
}

/// "a, b e c" no idioma.
fn join_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [a] => a.clone(),
        [init @ .., last] => t_args(
            "nucleo-lancamentos-rrule-lista",
            &args(&[
                ("inicio", init.join(", ").into()),
                ("ultimo", last.clone().into()),
            ]),
        ),
    }
}

/// "às segundas e quartas", "aos sábados", "às sextas e aos sábados".
fn days_phrase(days: &[Weekday]) -> String {
    let keys: Vec<(&str, bool)> = days.iter().map(|d| day_key(*d)).collect();
    let feminine = keys.iter().all(|(_, f)| *f);
    let masculine = keys.iter().all(|(_, f)| !*f);
    let (gender, list) = if feminine || masculine {
        (
            if feminine { "f" } else { "m" },
            join_list(&keys.iter().map(|(k, _)| t(k)).collect::<Vec<_>>()),
        )
    } else {
        // Gêneros misturados: cada dia com o seu artigo.
        let items: Vec<String> = keys
            .iter()
            .map(|(k, f)| {
                t_args(
                    "nucleo-lancamentos-rrule-dia-com-artigo",
                    &args(&[
                        ("genero", if *f { "f" } else { "m" }.into()),
                        ("dia", t(k).into()),
                    ]),
                )
            })
            .collect();
        ("misto", join_list(&items))
    };
    t_args(
        "nucleo-lancamentos-rrule-dias",
        &args(&[("genero", gender.into()), ("lista", list.into())]),
    )
}

/// Monta um pedaço com a base ("todo mês", "a cada 2 semanas") e mais argumentos.
fn with_base(key: &str, base: &str, more: &[(&'static str, FluentValue<'static>)]) -> String {
    let mut list = vec![("base", FluentValue::from(base.to_string()))];
    list.extend(more.iter().cloned());
    t_args(key, &args(&list))
}

/// Descrição legível: "todo mês no dia 5", "a cada 2 semanas às segundas".
pub fn human(spec: &RuleSpec) -> String {
    let interval = spec.interval.unwrap_or(1).max(1);
    let days = days_of(spec).unwrap_or_default();
    let every = |key: &str| t_args(key, &args(&[("n", interval.into())]));
    let s = match spec.freq {
        Freq::Daily => {
            let base = every("nucleo-lancamentos-rrule-diaria");
            let weekdays = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ];
            if days.is_empty() || days.len() == 7 {
                base
            } else if interval == 1 && days == weekdays {
                t("nucleo-lancamentos-rrule-dia-util")
            } else {
                with_base(
                    "nucleo-lancamentos-rrule-so-dias",
                    &base,
                    &[("dias", days_phrase(&days).into())],
                )
            }
        }
        Freq::Weekly => {
            let base = every("nucleo-lancamentos-rrule-semanal");
            if days.is_empty() {
                base
            } else {
                with_base(
                    "nucleo-lancamentos-rrule-nos-dias",
                    &base,
                    &[("dias", days_phrase(&days).into())],
                )
            }
        }
        Freq::Monthly => {
            let base = every("nucleo-lancamentos-rrule-mensal");
            match spec.by_month_day {
                Some(31) => with_base("nucleo-lancamentos-rrule-ultimo-dia", &base, &[]),
                Some(d) => with_base(
                    "nucleo-lancamentos-rrule-no-dia",
                    &base,
                    &[("dia", d.into())],
                ),
                None => base,
            }
        }
        Freq::Yearly => {
            let base = every("nucleo-lancamentos-rrule-anual");
            let month = spec.by_month.and_then(|m| {
                i18n::month_names(i18n::current(), false)
                    .get(m as usize - 1)
                    .cloned()
            });
            match (spec.by_month_day, month) {
                (Some(d), Some(m)) => with_base(
                    "nucleo-lancamentos-rrule-em-data",
                    &base,
                    &[("dia", d.into()), ("mes", m.into())],
                ),
                (None, Some(m)) => with_base(
                    "nucleo-lancamentos-rrule-em-mes",
                    &base,
                    &[("mes", m.into())],
                ),
                (Some(31), None) => {
                    with_base("nucleo-lancamentos-rrule-ultimo-dia-mes", &base, &[])
                }
                (Some(d), None) => with_base(
                    "nucleo-lancamentos-rrule-no-dia",
                    &base,
                    &[("dia", d.into())],
                ),
                (None, None) => base,
            }
        }
    };
    match spec.count {
        Some(c) => t_args(
            "nucleo-lancamentos-rrule-vezes",
            &args(&[("regra", s.into()), ("n", c.into())]),
        ),
        None => s,
    }
}

/// Valor mensal equivalente (semanal × 52 ÷ 12, diário × 365 ÷ 12, anual ÷ 12…),
/// arredondado metade para longe do zero. Mantém o sinal do valor.
pub fn monthly_equivalent(spec: &RuleSpec, amount: i64) -> i64 {
    let interval = i128::from(spec.interval.unwrap_or(1).max(1));
    let a = i128::from(amount);
    let n_days = days_of(spec).map(|d| d.len()).unwrap_or(0) as i128;
    let v = match spec.freq {
        Freq::Daily if n_days > 0 && n_days < 7 => div_round(a * n_days * 52, 12 * interval),
        Freq::Daily => div_round(a * 365, 12 * interval),
        Freq::Weekly => div_round(a * n_days.max(1) * 52, 12 * interval),
        Freq::Monthly => div_round(a, interval),
        Freq::Yearly => div_round(a, 12 * interval),
    };
    v as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::{fmt_date, parse_date};

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    fn spec(s: &str) -> RuleSpec {
        parse(s).unwrap()
    }

    fn dates(v: Vec<NaiveDate>) -> Vec<String> {
        v.into_iter().map(fmt_date).collect()
    }

    #[test]
    fn parses_and_serializes_roundtrip() {
        let s = spec("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=5");
        assert_eq!(s.freq, Freq::Monthly);
        assert_eq!(s.by_month_day, Some(5));
        assert_eq!(to_rrule(&s), "FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=5");
        let w = spec("rrule:freq=weekly;interval=2;byday=MO,fr;count=10");
        assert_eq!(w.by_day, Some(vec!["MO".into(), "FR".into()]));
        assert_eq!(to_rrule(&w), "FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,FR;COUNT=10");
        assert_eq!(parse(&to_rrule(&w)).unwrap(), w);
        let y = spec("FREQ=YEARLY;BYMONTH=3;BYMONTHDAY=10");
        assert_eq!(
            to_rrule(&y),
            "FREQ=YEARLY;INTERVAL=1;BYMONTH=3;BYMONTHDAY=10"
        );
        // Without an explicit INTERVAL, the canonical form includes INTERVAL=1.
        assert_eq!(to_rrule(&spec("FREQ=DAILY")), "FREQ=DAILY;INTERVAL=1");
    }

    #[test]
    fn rejects_unsupported_parts() {
        for bad in [
            "",
            "INTERVAL=2",
            "FREQ=HOURLY",
            "FREQ=MONTHLY;BYDAY=1MO",
            "FREQ=WEEKLY;BYDAY=-1FR",
            "FREQ=WEEKLY;BYDAY=XX",
            "FREQ=MONTHLY;UNTIL=20261231",
            "FREQ=MONTHLY;BYMONTHDAY=32",
            "FREQ=MONTHLY;BYMONTHDAY=0",
            "FREQ=MONTHLY;BYMONTHDAY=5,20",
            "FREQ=WEEKLY;BYMONTHDAY=5",
            "FREQ=MONTHLY;BYMONTH=3",
            "FREQ=MONTHLY;BYDAY=MO",
            "FREQ=DAILY;INTERVAL=0",
            "FREQ=DAILY;COUNT=0",
            "FREQ=DAILY;INTERVAL=x",
        ] {
            assert!(parse(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn monthly_day_31_becomes_last_day() {
        let s = spec("FREQ=MONTHLY;BYMONTHDAY=31");
        let v = next_n(&s, d("2026-01-31"), None, d("2026-01-01"), 5).unwrap();
        assert_eq!(
            dates(v),
            vec![
                "2026-01-31",
                "2026-02-28",
                "2026-03-31",
                "2026-04-30",
                "2026-05-31"
            ]
        );
        // Leap year.
        let v = next_n(&s, d("2028-02-01"), None, d("2028-02-01"), 1).unwrap();
        assert_eq!(dates(v), vec!["2028-02-29"]);
        // Implicit day from the start date.
        let implicit = spec("FREQ=MONTHLY");
        let v = next_n(&implicit, d("2026-01-30"), None, d("2026-01-01"), 2).unwrap();
        assert_eq!(dates(v), vec!["2026-01-30", "2026-02-28"]);
    }

    #[test]
    fn monthly_start_after_day_skips_to_next_month_and_interval() {
        let s = spec("FREQ=MONTHLY;INTERVAL=2;BYMONTHDAY=5");
        let v = next_n(&s, d("2026-01-10"), None, d("2026-01-01"), 3).unwrap();
        assert_eq!(dates(v), vec!["2026-03-05", "2026-05-05", "2026-07-05"]);
    }

    #[test]
    fn weekly_daily_yearly_count_and_end() {
        let w = spec("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,TH");
        // 2026-10-14 is a Wednesday.
        let v = next_n(&w, d("2026-10-14"), None, d("2026-10-14"), 4).unwrap();
        assert_eq!(
            dates(v),
            vec!["2026-10-15", "2026-10-26", "2026-10-29", "2026-11-09"]
        );
        let implicit = spec("FREQ=WEEKLY");
        let v = next_n(&implicit, d("2026-10-14"), None, d("2026-10-14"), 2).unwrap();
        assert_eq!(dates(v), vec!["2026-10-14", "2026-10-21"]);

        let weekdays = spec("FREQ=DAILY;BYDAY=MO,TU,WE,TH,FR;COUNT=4");
        let v = next_n(&weekdays, d("2026-10-16"), None, d("2026-10-01"), 10).unwrap();
        assert_eq!(
            dates(v),
            vec!["2026-10-16", "2026-10-19", "2026-10-20", "2026-10-21"]
        );

        let y = spec("FREQ=YEARLY;BYMONTH=2;BYMONTHDAY=29");
        let v = next_n(&y, d("2027-01-01"), None, d("2027-01-01"), 2).unwrap();
        assert_eq!(dates(v), vec!["2027-02-28", "2028-02-29"]);

        // COUNT counts from the start, even with a later window.
        let c = spec("FREQ=MONTHLY;BYMONTHDAY=1;COUNT=3");
        let v = occurrences_between(&c, d("2026-01-01"), None, d("2026-02-15"), d("2027-01-01"))
            .unwrap();
        assert_eq!(dates(v), vec!["2026-03-01"]);
        // End date inclusive.
        let m = spec("FREQ=MONTHLY;BYMONTHDAY=5");
        let v = occurrences_between(
            &m,
            d("2026-01-05"),
            Some(d("2026-03-05")),
            d("2026-01-01"),
            d("2027-01-01"),
        )
        .unwrap();
        assert_eq!(dates(v), vec!["2026-01-05", "2026-02-05", "2026-03-05"]);
    }

    #[test]
    fn human_descriptions_in_portuguese() {
        assert_eq!(
            human(&spec("FREQ=MONTHLY;BYMONTHDAY=5")),
            "todo mês no dia 5"
        );
        assert_eq!(
            human(&spec("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO")),
            "a cada 2 semanas às segundas"
        );
        assert_eq!(
            human(&spec("FREQ=WEEKLY;BYDAY=MO,WE,FR")),
            "toda semana às segundas, quartas e sextas"
        );
        assert_eq!(
            human(&spec("FREQ=WEEKLY;BYDAY=SA,SU")),
            "toda semana aos sábados e domingos"
        );
        assert_eq!(
            human(&spec("FREQ=WEEKLY;BYDAY=FR,SA")),
            "toda semana às sextas e aos sábados"
        );
        assert_eq!(
            human(&spec("FREQ=MONTHLY;BYMONTHDAY=31")),
            "todo mês no último dia"
        );
        assert_eq!(
            human(&spec("FREQ=MONTHLY;INTERVAL=3;BYMONTHDAY=10;COUNT=4")),
            "a cada 3 meses no dia 10, 4 vezes"
        );
        assert_eq!(
            human(&spec("FREQ=YEARLY;BYMONTH=3;BYMONTHDAY=1")),
            "todo ano em 1º de março"
        );
        assert_eq!(
            human(&spec("FREQ=YEARLY;INTERVAL=2;BYMONTH=12;BYMONTHDAY=25")),
            "a cada 2 anos em 25 de dezembro"
        );
        assert_eq!(human(&spec("FREQ=DAILY")), "todo dia");
        assert_eq!(human(&spec("FREQ=DAILY;INTERVAL=3")), "a cada 3 dias");
        assert_eq!(
            human(&spec("FREQ=DAILY;BYDAY=MO,TU,WE,TH,FR")),
            "todo dia útil (segunda a sexta)"
        );
    }

    #[test]
    fn monthly_equivalents() {
        assert_eq!(monthly_equivalent(&spec("FREQ=MONTHLY"), -150000), -150000);
        assert_eq!(
            monthly_equivalent(&spec("FREQ=MONTHLY;INTERVAL=2"), -10001),
            -5001
        );
        // Weekly: 100.00 × 52 ÷ 12 = 433.33
        assert_eq!(
            monthly_equivalent(&spec("FREQ=WEEKLY;BYDAY=MO"), 10000),
            43333
        );
        // Twice a week every 2 weeks: 100 × 2 × 52 ÷ 24 = 433.33
        assert_eq!(
            monthly_equivalent(&spec("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,TH"), 10000),
            43333
        );
        // Annual: 1,200.00 ÷ 12
        assert_eq!(monthly_equivalent(&spec("FREQ=YEARLY"), -120000), -10000);
        // Daily: 10.00 × 365 ÷ 12 = 304.17
        assert_eq!(monthly_equivalent(&spec("FREQ=DAILY"), 1000), 30417);
    }

    /// The description comes out in the installation language (day gender is Portuguese-only).
    #[test]
    fn human_descriptions_follow_the_language() {
        use crate::i18n::{Locale, set_current};
        set_current(Locale::En);
        assert_eq!(
            human(&spec("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO")),
            "every 2 weeks on Mondays"
        );
        assert_eq!(
            human(&spec("FREQ=YEARLY;BYMONTH=3;BYMONTHDAY=1")),
            "every year on March 1"
        );
        assert_eq!(
            human(&spec("FREQ=MONTHLY;INTERVAL=3;BYMONTHDAY=10;COUNT=4")),
            "every 3 months on day 10, 4 times"
        );
        assert_eq!(
            human(&spec("FREQ=DAILY;BYDAY=MO,TU,WE,TH,FR")),
            "every weekday (Monday to Friday)"
        );
        set_current(Locale::Es);
        assert_eq!(
            human(&spec("FREQ=WEEKLY;BYDAY=FR,SA")),
            "todas las semanas los viernes y los sábados"
        );
        assert!(
            parse("FREQ=HOURLY")
                .unwrap_err()
                .message()
                .starts_with("frecuencia no compatible")
        );
        set_current(Locale::PtBr);
        assert_eq!(
            human(&spec("FREQ=WEEKLY;BYDAY=FR,SA")),
            "toda semana às sextas e aos sábados"
        );
    }
}
