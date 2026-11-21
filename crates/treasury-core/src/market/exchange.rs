//! Catálogo de bolsas (tabela `exchange`, semeada pela V85): país, moeda, sufixo do Yahoo,
//! fuso e pregão regular. O código da bolsa é o `market` do ativo; daí vêm a moeda e o país
//! padrão, o símbolo do Yahoo (ticker + sufixo) e o horário em que a coleta intradiária roda.

use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Utc};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::market::zone::Zone;
use crate::time::now_utc;

/// Moedas com PTAX no Banco Central (`Moedas` da API Olinda).
pub const PTAX_CURRENCIES: &[&str] = &[
    "AUD", "CAD", "CHF", "DKK", "EUR", "GBP", "JPY", "NOK", "SEK", "USD",
];

/// Moedas com câmbio de referência do BCE contra o euro (séries `fx_eur_<moeda>` do catálogo).
/// As que não têm PTAX chegam ao real cruzando pelo euro (moeda → EUR pelo BCE, EUR → BRL pela
/// PTAX; `fx::rate`). As que não estão em nenhuma das duas listas precisam de câmbio manual.
pub const ECB_CURRENCIES: &[&str] = &[
    "AUD", "CAD", "CHF", "CNY", "CZK", "DKK", "GBP", "HKD", "HUF", "IDR", "ILS", "INR", "ISK",
    "JPY", "KRW", "MXN", "MYR", "NOK", "NZD", "PHP", "PLN", "RON", "SEK", "SGD", "THB", "TRY",
    "USD", "ZAR",
];

/// O app acha o câmbio da moeda sozinho (PTAX, ou BCE cruzado pelo euro).
pub fn fx_automatic(currency: &str) -> bool {
    currency == "BRL" || PTAX_CURRENCIES.contains(&currency) || ECB_CURRENCIES.contains(&currency)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Exchange {
    pub code: String,
    pub name: String,
    pub country: String,
    pub currency: String,
    /// Sufixo do símbolo no Yahoo (".SA", ".L"); vazio nos EUA.
    pub yahoo_suffix: String,
    pub mic: Option<String>,
    /// Fuso IANA (America/Sao_Paulo).
    pub timezone: String,
    /// Pregão regular, HH:MM no fuso da bolsa.
    pub open_time: String,
    pub close_time: String,
    pub lunch_start: Option<String>,
    pub lunch_end: Option<String>,
    /// Dias de pregão na numeração ISO (1 = segunda … 7 = domingo).
    pub trading_days: String,
    pub aliases: Vec<String>,
    /// Do catálogo do app (não pode ser apagada).
    pub builtin: bool,
    pub notes: Option<String>,
    /// A moeda tem PTAX no Banco Central (câmbio automático para BRL).
    pub fx_automatic: bool,
}

/// Bolsa nova ou alterada pelo usuário.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ExchangeInput {
    pub code: String,
    pub name: String,
    pub country: String,
    pub currency: String,
    #[ts(optional)]
    pub yahoo_suffix: Option<String>,
    #[ts(optional)]
    pub mic: Option<String>,
    pub timezone: String,
    pub open_time: String,
    pub close_time: String,
    #[ts(optional)]
    pub lunch_start: Option<String>,
    #[ts(optional)]
    pub lunch_end: Option<String>,
    #[ts(optional)]
    pub trading_days: Option<String>,
    #[ts(optional)]
    pub aliases: Option<Vec<String>>,
    #[ts(optional)]
    pub notes: Option<String>,
}

/// Situação do pregão num instante.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SessionState {
    /// Antes da abertura, no dia de pregão.
    Pre,
    Open,
    /// Intervalo de almoço (Tóquio, Hong Kong…).
    Lunch,
    /// Depois do fechamento, no dia de pregão.
    Post,
    /// Fim de semana ou dia sem pregão.
    Closed,
}

impl SessionState {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionState::Pre => "pre",
            SessionState::Open => "open",
            SessionState::Lunch => "lunch",
            SessionState::Post => "post",
            SessionState::Closed => "closed",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "pre" => SessionState::Pre,
            "open" => SessionState::Open,
            "lunch" => SessionState::Lunch,
            "post" => SessionState::Post,
            _ => SessionState::Closed,
        }
    }
}

fn row_to_exchange(r: &Row<'_>) -> rusqlite::Result<Exchange> {
    let currency: String = r.get("currency")?;
    let aliases: String = r.get("aliases")?;
    Ok(Exchange {
        code: r.get("code")?,
        name: r.get("name")?,
        country: r.get("country")?,
        fx_automatic: fx_automatic(&currency),
        currency,
        yahoo_suffix: r.get("yahoo_suffix")?,
        mic: r.get("mic")?,
        timezone: r.get("timezone")?,
        open_time: r.get("open_time")?,
        close_time: r.get("close_time")?,
        lunch_start: r.get("lunch_start")?,
        lunch_end: r.get("lunch_end")?,
        trading_days: r.get("trading_days")?,
        aliases: aliases
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        builtin: r.get::<_, i64>("builtin")? == 1,
        notes: r.get("notes")?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Exchange>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM exchange ORDER BY country = 'BR' DESC, country = 'US' DESC, name",
    )?;
    let rows = stmt
        .query_map([], row_to_exchange)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, code: &str) -> Result<Option<Exchange>> {
    Ok(conn
        .query_row(
            "SELECT * FROM exchange WHERE code = upper(?1)",
            [code.trim()],
            row_to_exchange,
        )
        .optional()?)
}

/// Bolsa pelo código, por um apelido (nome por extenso, código do Yahoo) ou pelo nome.
pub fn resolve(conn: &Connection, market: &str) -> Result<Option<Exchange>> {
    let m = market.trim();
    if m.is_empty() {
        return Ok(None);
    }
    if let Some(e) = get(conn, m)? {
        return Ok(Some(e));
    }
    let up = m.to_uppercase();
    Ok(list(conn)?
        .into_iter()
        .find(|e| e.name.to_uppercase() == up || e.aliases.iter().any(|a| a.to_uppercase() == up)))
}

/// Bolsa de um símbolo do Yahoo: pelo sufixo (".L"); sem sufixo, pelo código de bolsa do
/// Yahoo (NMS, NYQ…) entre os apelidos.
pub fn for_yahoo(
    conn: &Connection,
    symbol: &str,
    yahoo_exchange: Option<&str>,
) -> Result<Option<Exchange>> {
    let all = list(conn)?;
    if let Some(dot) = symbol.rfind('.') {
        let suffix = &symbol[dot..];
        if let Some(e) = all
            .iter()
            .find(|e| !e.yahoo_suffix.is_empty() && e.yahoo_suffix.eq_ignore_ascii_case(suffix))
        {
            return Ok(Some(e.clone()));
        }
    }
    let Some(code) = yahoo_exchange.map(str::trim).filter(|c| !c.is_empty()) else {
        return Ok(None);
    };
    let up = code.to_uppercase();
    Ok(all
        .into_iter()
        .find(|e| e.code == up || e.aliases.iter().any(|a| a.to_uppercase() == up)))
}

/// Ticker da bolsa a partir do símbolo do Yahoo (tira o sufixo; nos EUA, "BRK-B" → "BRK.B").
pub fn ticker_from_yahoo(symbol: &str, ex: Option<&Exchange>) -> String {
    let s = symbol.trim().to_uppercase();
    match ex {
        Some(e) if !e.yahoo_suffix.is_empty() => s
            .strip_suffix(&e.yahoo_suffix.to_uppercase())
            .unwrap_or(&s)
            .to_string(),
        Some(e) if e.country == "US" => s.replace('-', "."),
        _ => s,
    }
}

/// Símbolo do Yahoo: ticker + sufixo; nos EUA, ponto vira hífen ("BRK.B" → "BRK-B").
pub fn yahoo_symbol(ticker: &str, ex: &Exchange) -> String {
    let t = ticker.trim().to_uppercase();
    if ex.yahoo_suffix.is_empty() {
        t.replace('.', "-")
    } else {
        format!("{t}{}", ex.yahoo_suffix)
    }
}

fn hhmm(s: &str) -> Result<NaiveTime> {
    NaiveTime::parse_from_str(s.trim(), "%H:%M").map_err(|_| {
        Error::invalid(crate::i18n::t_args(
            "nucleo-ativos-erro-horario",
            &crate::i18n::args(&[("horario", s.to_string().into())]),
        ))
    })
}

fn unknown_tz(name: &str) -> Error {
    Error::invalid(crate::i18n::t_args(
        "nucleo-ativos-erro-fuso",
        &crate::i18n::args(&[("fuso", name.to_string().into())]),
    ))
}

pub fn tz(ex: &Exchange) -> Result<Zone> {
    ex.timezone
        .parse::<Zone>()
        .map_err(|_| unknown_tz(&ex.timezone))
}

/// Situação do pregão regular no instante `at` (feriados não entram: contam como pregão).
pub fn session_at(ex: &Exchange, at: DateTime<Utc>) -> Result<SessionState> {
    let local = at.with_timezone(&tz(ex)?);
    let day = local.weekday().number_from_monday();
    if !ex.trading_days.chars().any(|c| c.to_digit(10) == Some(day)) {
        return Ok(SessionState::Closed);
    }
    let t = local.time();
    let (open, close) = (hhmm(&ex.open_time)?, hhmm(&ex.close_time)?);
    if t < open {
        return Ok(SessionState::Pre);
    }
    if t >= close {
        return Ok(SessionState::Post);
    }
    if let (Some(a), Some(b)) = (&ex.lunch_start, &ex.lunch_end)
        && t >= hhmm(a)?
        && t < hhmm(b)?
    {
        return Ok(SessionState::Lunch);
    }
    Ok(SessionState::Open)
}

/// A coleta intradiária deve rodar agora: pregão aberto ou fechado há menos de
/// `grace_minutes` (o Yahoo publica com ~15 min de atraso, então o último preço do dia
/// chega depois do fechamento).
pub fn should_poll(ex: &Exchange, at: DateTime<Utc>, grace_minutes: i64) -> Result<bool> {
    match session_at(ex, at)? {
        SessionState::Open => Ok(true),
        SessionState::Lunch => Ok(false),
        SessionState::Pre | SessionState::Closed => Ok(false),
        SessionState::Post => {
            let earlier = at - chrono::Duration::minutes(grace_minutes);
            Ok(matches!(
                session_at(ex, earlier)?,
                SessionState::Open | SessionState::Lunch
            ))
        }
    }
}

/// Data local do pregão de um instante, no fuso da bolsa (ou do nome IANA informado).
pub fn local_date(timezone: &str, at: DateTime<Utc>) -> chrono::NaiveDate {
    match timezone.parse::<Zone>() {
        Ok(tz) => at.with_timezone(&tz).date_naive(),
        Err(_) => at.date_naive(),
    }
}

/// Abertura do pregão regular na data local, em UTC (para gráficos).
pub fn open_at(ex: &Exchange, date: chrono::NaiveDate) -> Result<Option<DateTime<Utc>>> {
    let t = hhmm(&ex.open_time)?;
    Ok(tz(ex)?
        .from_local_datetime(&date.and_time(t))
        .earliest()
        .map(|d| d.with_timezone(&Utc)))
}

fn validate(i: &ExchangeInput) -> Result<ExchangeInput> {
    let code = i.code.trim().to_uppercase();
    if code.is_empty()
        || code.len() > 12
        || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-codigo",
        )));
    }
    if i.name.trim().is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-nome",
        )));
    }
    let country = i.country.trim().to_uppercase();
    if country.len() != 2 || !country.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-pais",
        )));
    }
    let currency = crate::money::validate_currency(&i.currency)?;
    i.timezone
        .trim()
        .parse::<Zone>()
        .map_err(|_| unknown_tz(&i.timezone))?;
    let (open, close) = (hhmm(&i.open_time)?, hhmm(&i.close_time)?);
    if open >= close {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-abertura",
        )));
    }
    match (&i.lunch_start, &i.lunch_end) {
        (Some(a), Some(b)) if !a.trim().is_empty() || !b.trim().is_empty() => {
            let (a, b) = (hhmm(a)?, hhmm(b)?);
            if !(open < a && a < b && b < close) {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-ativos-erro-bolsa-almoco",
                )));
            }
        }
        _ => {}
    }
    let days = i.trading_days.clone().unwrap_or_else(|| "12345".into());
    if days.is_empty() || !days.chars().all(|c| ('1'..='7').contains(&c)) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-dias",
        )));
    }
    let suffix = i
        .yahoo_suffix
        .clone()
        .unwrap_or_default()
        .trim()
        .to_uppercase();
    if !suffix.is_empty() && (!suffix.starts_with('.') || suffix.len() > 5) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-sufixo",
        )));
    }
    let clean = |s: &Option<String>| {
        s.as_ref()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    Ok(ExchangeInput {
        code,
        name: i.name.trim().to_string(),
        country,
        currency,
        yahoo_suffix: Some(suffix),
        mic: clean(&i.mic).map(|m| m.to_uppercase()),
        timezone: i.timezone.trim().to_string(),
        open_time: open.format("%H:%M").to_string(),
        close_time: close.format("%H:%M").to_string(),
        lunch_start: clean(&i.lunch_start),
        lunch_end: clean(&i.lunch_end),
        trading_days: Some(days),
        aliases: i.aliases.clone(),
        notes: clean(&i.notes),
    })
}

/// Cadastra ou altera uma bolsa (as do catálogo também podem ter horário e apelidos ajustados).
pub fn save(tx: &Transaction<'_>, input: &ExchangeInput) -> Result<String> {
    let i = validate(input)?;
    let aliases = i
        .aliases
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|a| a.trim().replace(',', " "))
        .filter(|a| !a.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    let now = now_utc();
    tx.execute(
        "INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time,
                               lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0, ?14, ?15, ?15)
         ON CONFLICT (code) DO UPDATE SET name = excluded.name, country = excluded.country, currency = excluded.currency,
             yahoo_suffix = excluded.yahoo_suffix, mic = excluded.mic, timezone = excluded.timezone,
             open_time = excluded.open_time, close_time = excluded.close_time, lunch_start = excluded.lunch_start,
             lunch_end = excluded.lunch_end, trading_days = excluded.trading_days, aliases = excluded.aliases,
             notes = excluded.notes, updated_at = excluded.updated_at",
        params![
            i.code,
            i.name,
            i.country,
            i.currency,
            i.yahoo_suffix.clone().unwrap_or_default(),
            i.mic,
            i.timezone,
            i.open_time,
            i.close_time,
            i.lunch_start,
            i.lunch_end,
            i.trading_days.clone().unwrap_or_default(),
            aliases,
            i.notes,
            now
        ],
    )?;
    Ok(i.code)
}

/// Apaga uma bolsa cadastrada pelo usuário e sem ativos.
pub fn delete(tx: &Transaction<'_>, code: &str) -> Result<()> {
    let Some(e) = get(tx, code)? else {
        return Err(Error::not_found(crate::i18n::t_args(
            "nucleo-ativos-ref-bolsa",
            &crate::i18n::args(&[("bolsa", code.to_string().into())]),
        )));
    };
    if e.builtin {
        return Err(Error::conflict(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-do-catalogo",
        )));
    }
    let used: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM asset WHERE upper(market) = ?1)",
        [&e.code],
        |r| r.get(0),
    )?;
    if used {
        return Err(Error::conflict(crate::i18n::t(
            "nucleo-ativos-erro-bolsa-com-ativos",
        )));
    }
    tx.execute("DELETE FROM exchange WHERE code = ?1", [&e.code])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        (dir, db)
    }

    #[test]
    fn catalog_is_seeded_and_valid() {
        let (_d, db) = db();
        let all = list(db.conn()).unwrap();
        assert!(all.len() >= 40, "{}", all.len());
        assert_eq!(all[0].code, "B3", "Brasil primeiro");
        for e in &all {
            tz(e).unwrap_or_else(|_| panic!("fuso de {}", e.code));
            assert!(
                hhmm(&e.open_time).unwrap() < hhmm(&e.close_time).unwrap(),
                "{}",
                e.code
            );
            assert!(e.builtin);
        }
        let suffixes: Vec<&str> = all
            .iter()
            .map(|e| e.yahoo_suffix.as_str())
            .filter(|s| !s.is_empty())
            .collect();
        let mut dedup = suffixes.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(suffixes.len(), dedup.len(), "sufixos do Yahoo únicos");
        for (sym, ex) in [
            ("PETR4.SA", "B3"),
            ("VOD.L", "LSE"),
            ("SAP.DE", "XETRA"),
            ("MC.PA", "EPA"),
            ("ASML.AS", "AMS"),
            ("EDP.LS", "ELI"),
            ("SAN.MC", "BME"),
            ("ENI.MI", "MIL"),
            ("NESN.SW", "SIX"),
            ("SHOP.TO", "TSX"),
            ("ABC.V", "TSXV"),
            ("BHP.AX", "ASX"),
            ("7203.T", "TSE"),
            ("0700.HK", "HKEX"),
            ("RELIANCE.NS", "NSE"),
            ("500325.BO", "BSE"),
            ("WALMEX.MX", "BMV"),
            ("GGAL.BA", "BYMA"),
            ("SQM-B.SN", "BCS"),
            ("005930.KS", "KRX"),
            ("2330.TW", "TWSE"),
            ("D05.SI", "SGX"),
            ("NPN.JO", "JSE"),
        ] {
            assert_eq!(
                for_yahoo(db.conn(), sym, None).unwrap().map(|e| e.code),
                Some(ex.to_string()),
                "{sym}"
            );
        }
        assert_eq!(
            for_yahoo(db.conn(), "AAPL", Some("NMS"))
                .unwrap()
                .unwrap()
                .code,
            "NASDAQ"
        );
        assert_eq!(
            for_yahoo(db.conn(), "BRK-B", Some("NYQ"))
                .unwrap()
                .unwrap()
                .code,
            "NYSE"
        );
        assert!(for_yahoo(db.conn(), "XYZ", Some("PNK")).unwrap().is_none());
        assert_eq!(
            resolve(db.conn(), "NYSE Arca").unwrap().unwrap().code,
            "ARCA"
        );
        assert_eq!(resolve(db.conn(), "lse").unwrap().unwrap().code, "LSE");
        let lse = get(db.conn(), "LSE").unwrap().unwrap();
        assert_eq!((lse.currency.as_str(), lse.fx_automatic), ("GBP", true));
        // Sem PTAX, mas com câmbio de referência do BCE (cruza pelo euro).
        assert!(get(db.conn(), "HKEX").unwrap().unwrap().fx_automatic);
        // Nem PTAX nem BCE: câmbio manual.
        assert!(!fx_automatic("TWD") && !fx_automatic("ARS") && fx_automatic("BRL"));
        // Toda moeda do BCE tem a série no catálogo.
        for c in ECB_CURRENCIES {
            let id = format!("fx_eur_{}", c.to_ascii_lowercase());
            let n: i64 = db
                .conn()
                .query_row(
                    "SELECT count(*) FROM index_series WHERE id = ?1",
                    [&id],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "{id} fora do catálogo");
        }
    }

    #[test]
    fn yahoo_symbols_round_trip() {
        let (_d, db) = db();
        let c = db.conn();
        let nyse = get(c, "NYSE").unwrap().unwrap();
        let tse = get(c, "TSE").unwrap().unwrap();
        assert_eq!(yahoo_symbol("BRK.B", &nyse), "BRK-B");
        assert_eq!(ticker_from_yahoo("BRK-B", Some(&nyse)), "BRK.B");
        assert_eq!(yahoo_symbol("7203", &tse), "7203.T");
        assert_eq!(ticker_from_yahoo("7203.T", Some(&tse)), "7203");
    }

    #[test]
    fn trading_hours_follow_each_timezone_and_dst() {
        let (_d, db) = db();
        let c = db.conn();
        let nyse = get(c, "NYSE").unwrap().unwrap();
        let lse = get(c, "LSE").unwrap().unwrap();
        let b3 = get(c, "B3").unwrap().unwrap();
        let tse = get(c, "TSE").unwrap().unwrap();
        let asx = get(c, "ASX").unwrap().unwrap();
        let tadawul = get(c, "TADAWUL").unwrap().unwrap();
        // NYSE abre 9h30 em Nova York: 13h30 UTC no verão (EDT), 14h30 UTC no inverno (EST).
        assert_eq!(
            session_at(&nyse, utc("2026-07-15T13:29:00Z")).unwrap(),
            SessionState::Pre
        );
        assert_eq!(
            session_at(&nyse, utc("2026-07-15T13:30:00Z")).unwrap(),
            SessionState::Open
        );
        assert_eq!(
            session_at(&nyse, utc("2026-01-15T14:00:00Z")).unwrap(),
            SessionState::Pre
        );
        assert_eq!(
            session_at(&nyse, utc("2026-01-15T14:30:00Z")).unwrap(),
            SessionState::Open
        );
        assert_eq!(
            session_at(&nyse, utc("2026-01-15T21:00:00Z")).unwrap(),
            SessionState::Post
        );
        // Semana de março em que só os EUA mudaram de horário (8/3/2026; Londres muda em 29/3):
        // NYSE abre às 13h30 UTC e Londres continua abrindo às 8h UTC.
        assert_eq!(
            session_at(&nyse, utc("2026-03-10T13:45:00Z")).unwrap(),
            SessionState::Open
        );
        assert_eq!(
            session_at(&lse, utc("2026-03-10T08:00:00Z")).unwrap(),
            SessionState::Open
        );
        assert_eq!(
            session_at(&lse, utc("2026-03-31T07:30:00Z")).unwrap(),
            SessionState::Open,
            "BST: 8h30 local"
        );
        assert_eq!(
            session_at(&lse, utc("2026-03-31T15:30:00Z")).unwrap(),
            SessionState::Post,
            "BST: 16h30 local"
        );
        // B3 (sem horário de verão desde 2019): 10h–18h em São Paulo = 13h–21h UTC.
        assert_eq!(
            session_at(&b3, utc("2026-10-14T12:59:00Z")).unwrap(),
            SessionState::Pre
        );
        assert_eq!(
            session_at(&b3, utc("2026-10-14T13:00:00Z")).unwrap(),
            SessionState::Open
        );
        assert_eq!(
            session_at(&b3, utc("2026-10-17T15:00:00Z")).unwrap(),
            SessionState::Closed,
            "sábado"
        );
        // Tóquio: almoço 11h30–12h30 local (02h30–03h30 UTC).
        assert_eq!(
            session_at(&tse, utc("2026-10-14T02:45:00Z")).unwrap(),
            SessionState::Lunch
        );
        assert_eq!(
            session_at(&tse, utc("2026-10-14T03:45:00Z")).unwrap(),
            SessionState::Open
        );
        // Segunda em Tóquio ainda é domingo no Brasil: o dia da semana é o local da bolsa.
        assert_eq!(
            session_at(&tse, utc("2026-10-19T00:30:00Z")).unwrap(),
            SessionState::Open
        );
        // Sydney em horário de verão (AEDT, +11) desde 4/10/2026: 10h local = 23h UTC do dia anterior.
        assert_eq!(
            session_at(&asx, utc("2026-10-13T23:05:00Z")).unwrap(),
            SessionState::Open
        );
        assert_eq!(
            session_at(&asx, utc("2026-07-14T00:05:00Z")).unwrap(),
            SessionState::Open,
            "AEST, +10"
        );
        assert_eq!(
            session_at(&asx, utc("2026-07-13T23:30:00Z")).unwrap(),
            SessionState::Pre
        );
        // Tadawul: domingo a quinta.
        assert_eq!(
            session_at(&tadawul, utc("2026-10-18T08:00:00Z")).unwrap(),
            SessionState::Open,
            "domingo"
        );
        assert_eq!(
            session_at(&tadawul, utc("2026-10-16T08:00:00Z")).unwrap(),
            SessionState::Closed,
            "sexta"
        );
        // Coleta: pregão aberto e até 20 min depois do fechamento.
        assert!(should_poll(&nyse, utc("2026-01-15T21:15:00Z"), 20).unwrap());
        assert!(!should_poll(&nyse, utc("2026-01-15T21:25:00Z"), 20).unwrap());
        assert!(!should_poll(&tse, utc("2026-10-14T02:45:00Z"), 20).unwrap());
        assert_eq!(
            local_date("Asia/Tokyo", utc("2026-10-18T23:30:00Z")).to_string(),
            "2026-10-19"
        );
        assert_eq!(
            open_at(&nyse, chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap())
                .unwrap()
                .unwrap(),
            utc("2026-07-15T13:30:00Z")
        );
    }

    #[test]
    fn user_exchanges_are_validated_and_protected() {
        let (_d, mut db) = db();
        let input = ExchangeInput {
            code: "pse".into(),
            name: "Philippine Stock Exchange".into(),
            country: "ph".into(),
            currency: "php".into(),
            yahoo_suffix: Some(".ps".into()),
            mic: Some("xphs".into()),
            timezone: "Asia/Manila".into(),
            open_time: "9:30".into(),
            close_time: "15:00".into(),
            lunch_start: Some("12:00".into()),
            lunch_end: Some("13:00".into()),
            trading_days: None,
            aliases: Some(vec!["PHS".into()]),
            notes: None,
        };
        crate::changes::record(db.conn_mut(), &crate::changes::Author::user(), "t", |tx| {
            assert_eq!(save(tx, &input)?, "PSE");
            Ok(())
        })
        .unwrap();
        let e = get(db.conn(), "PSE").unwrap().unwrap();
        assert_eq!(
            (
                e.yahoo_suffix.as_str(),
                e.open_time.as_str(),
                e.currency.as_str()
            ),
            (".PS", "09:30", "PHP")
        );
        assert!(!e.builtin && e.fx_automatic);
        assert_eq!(
            for_yahoo(db.conn(), "JFC.PS", None).unwrap().unwrap().code,
            "PSE"
        );
        let mut try_save = |f: &dyn Fn(&mut ExchangeInput)| {
            let mut i = input.clone();
            f(&mut i);
            crate::changes::record(db.conn_mut(), &crate::changes::Author::user(), "t", |tx| {
                save(tx, &i).map(|_| ())
            })
            .unwrap_err()
        };
        let e = try_save(&|i| i.timezone = "Marte/Olympus".into());
        assert!(e.to_string().contains("fuso"), "{e}");
        let e = try_save(&|i| i.lunch_start = Some("08:00".into()));
        assert!(e.to_string().contains("almoço"), "{e}");
        let e = try_save(&|i| i.trading_days = Some("189".into()));
        assert!(e.to_string().contains("dias"), "{e}");
        let user = crate::changes::Author::user();
        let e =
            crate::changes::record(db.conn_mut(), &user, "t", |tx| delete(tx, "LSE")).unwrap_err();
        assert_eq!(e.code(), "conflict");
        crate::changes::record(db.conn_mut(), &user, "t", |tx| delete(tx, "pse")).unwrap();
        assert!(get(db.conn(), "PSE").unwrap().is_none());
    }
}
