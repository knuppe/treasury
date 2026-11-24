//! Fundos.NET (the CVM and B3 disclosure system for funds): the structured "Rendimentos e
//! Amortizações" notice that every listed real estate or agribusiness fund (FII, Fiagro)
//! files for each distribution. It is the primary source for what a unit holder was entitled
//! to: record date ("data-base"), exact amount per unit (up to nine decimals), payment date,
//! reference month and whether the income is tax exempt for individuals, for the whole life of
//! the fund (B3's own API only keeps the last ~12 months).
//!
//! Requests (public pages, no key):
//! - list: `pesquisarGerenciadorDocumentosDados` by fund CNPJ, category 14 ("Aviso aos
//!   Cotistas - Estruturado"), type 41 ("Rendimentos e Amortizações"), active documents only
//!   (`situacao=A`: a corrected filing replaces the old one), pages of [`PAGE_SIZE`];
//! - document: `downloadDocumento?id=…`, an XML of about 1 KB. Two layouts: until 2022 one
//!   document per security, with the codes in `DadosGerais`; since then one `Provento` block
//!   per security (unit and subscription receipts).
//!
//! Considerate use, as with the rest of B3: only the funds the family holds or held, one
//! request at a time at the site's pace ([`super::net::min_gap`], 1.5 s), each document
//! downloaded once (a filed notice does not change; a correction is a new document) and the
//! list refreshed every ~[`REFRESH_HOURS`] hours. Nothing leaves the vault. The XML also names
//! the person in charge at the administrator: that is not read.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde::Deserialize;

use super::net::Fetched;
use super::sync::{Runner, SyncItem, SyncSource, SyncStatus};
use crate::error::Result;
use crate::time::{fmt_date, now_utc};

/// `source` value in the tables and in the fetch log.
pub const SOURCE: &str = "fundos_net";
const BASE: &str = "https://fnet.bmfbovespa.com.br/fnet/publico";
/// Largest page the list accepts.
pub const PAGE_SIZE: u32 = 100;
/// Page limit per fund (the oldest funds have ~2 pages of notices).
pub const MAX_PAGES: u32 = 6;
/// A fund's list is read again after ~20 h.
pub const REFRESH_HOURS: i64 = 20;
/// Error: try again after a day.
pub const ERROR_RETRY_HOURS: i64 = 24;
/// Network errors in a row before the collection stops (the rest waits for the next one).
const MAX_CONSECUTIVE_ERRORS: u32 = 5;
/// Notices with a record date this many days before the first purchase are not downloaded.
const MARGIN_DAYS: i64 = 40;

/// Public page of a fund's documents (for the "open on the site" link).
pub fn site_url(cnpj: &str) -> String {
    format!("{BASE}/abrirGerenciadorDocumentosCVM?cnpjFundo={cnpj}")
}

/// One page of the fund's structured distribution notices, newest first.
pub fn list_url(cnpj: &str, start: u32) -> String {
    format!(
        "{BASE}/pesquisarGerenciadorDocumentosDados?d=1&s={start}&l={PAGE_SIZE}\
         &o%5B0%5D%5BdataEntrega%5D=desc&cnpjFundo={cnpj}&idCategoriaDocumento=14\
         &idTipoDocumento=41&idEspecieDocumento=0&situacao=A"
    )
}

pub fn document_url(id: i64) -> String {
    format!("{BASE}/downloadDocumento?id={id}")
}

// ---------------------------------------------------------------------------------------
// Parse (bytes → structures; tested with samples in the real format)
// ---------------------------------------------------------------------------------------

/// A document in the list.
#[derive(Debug, Clone, PartialEq)]
pub struct DocRef {
    pub id: i64,
    pub version: i64,
    /// Record date of the notice (`dataReferencia`).
    pub ref_date: Option<NaiveDate>,
    /// Filing date and time as the site writes it (`dd/mm/aaaa hh:mm`).
    pub delivered_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListPage {
    pub total: u32,
    pub docs: Vec<DocRef>,
}

#[derive(Deserialize)]
struct RawList {
    #[serde(rename = "recordsTotal", default)]
    total: u32,
    #[serde(default)]
    data: Vec<RawDoc>,
}

#[derive(Deserialize)]
struct RawDoc {
    id: i64,
    #[serde(default)]
    versao: Option<i64>,
    #[serde(rename = "dataReferencia", default)]
    data_referencia: Option<String>,
    #[serde(rename = "dataEntrega", default)]
    data_entrega: Option<String>,
    #[serde(rename = "situacaoDocumento", default)]
    situacao: Option<String>,
}

fn br_date(s: &str) -> Option<NaiveDate> {
    let d = s.trim().get(..10)?;
    NaiveDate::parse_from_str(d, "%d/%m/%Y").ok()
}

pub fn parse_list(bytes: &[u8]) -> std::result::Result<ListPage, String> {
    let raw: RawList = serde_json::from_slice(bytes).map_err(|e| format!("Fundos.NET: {e}"))?;
    Ok(ListPage {
        total: raw.total,
        docs: raw
            .data
            .into_iter()
            // The query already asks for active documents; a cancelled one is left out anyway.
            .filter(|d| d.situacao.as_deref().is_none_or(|s| s == "A"))
            .map(|d| DocRef {
                id: d.id,
                version: d.versao.unwrap_or(1),
                ref_date: d.data_referencia.as_deref().and_then(br_date),
                delivered_at: d.data_entrega,
            })
            .collect(),
    })
}

/// A distribution as the notice reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    pub ticker: String,
    pub isin: Option<String>,
    /// `fii_income` or `amortization`.
    pub kind: &'static str,
    /// Amortization type ("Total", "Parcial").
    pub amortization: Option<String>,
    pub ex_date: NaiveDate,
    pub value: Decimal,
    pub pay_date: Option<NaiveDate>,
    pub period: Option<String>,
    pub exempt: Option<bool>,
}

/// Body of the first `<name>` (or `<name …>`) element and its attributes; a self-closing
/// element has an empty body. `None` when there is no such element.
fn element<'a>(s: &'a str, name: &str) -> Option<(&'a str, &'a str, usize)> {
    let open = format!("<{name}");
    let mut from = 0;
    while let Some(i) = s[from..].find(&open) {
        let start = from + i + open.len();
        let next = s[start..].chars().next()?;
        if next == '>' || next == '/' || next.is_whitespace() {
            let gt = start + s[start..].find('>')?;
            let attrs = &s[start..gt];
            if attrs.trim_end().ends_with('/') {
                return Some((attrs, "", gt + 1));
            }
            let close = format!("</{name}>");
            let end = gt + 1 + s[gt + 1..].find(&close)?;
            return Some((attrs, &s[gt + 1..end], end + close.len()));
        }
        from = start;
    }
    None
}

/// Every `<name>` element in `s`: (attributes, body).
fn elements<'a>(s: &'a str, name: &str) -> Vec<(&'a str, &'a str)> {
    let mut out = vec![];
    let mut rest = s;
    while let Some((attrs, body, end)) = element(rest, name) {
        out.push((attrs, body));
        rest = &rest[end..];
    }
    out
}

fn text(s: &str, name: &str) -> Option<String> {
    let (_, body, _) = element(s, name)?;
    let t = body
        .replace("<![CDATA[", "")
        .replace("]]>", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    let t = t.trim();
    (!t.is_empty()).then(|| t.to_string())
}

fn attr(attrs: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let i = attrs.find(&key)? + key.len();
    let v = attrs[i..].split('"').next()?.trim();
    (!v.is_empty()).then(|| v.to_string())
}

fn iso_date(s: Option<String>) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s?.trim().get(..10)?, "%Y-%m-%d").ok()
}

fn amount(s: Option<String>) -> Option<Decimal> {
    let s = s?;
    let s = s.trim();
    // XML uses the decimal point; a comma (rare) is read as the decimal separator.
    let s = if s.contains('.') {
        s.to_string()
    } else {
        s.replace(',', ".")
    };
    s.parse().ok()
}

fn exempt(s: Option<String>) -> Option<bool> {
    match crate::import::text::fold(&s?).as_str() {
        "sim" | "true" | "s" | "1" => Some(true),
        "nao" | "false" | "n" | "0" => Some(false),
        _ => None,
    }
}

/// Reference month: the newer layout writes "Setembro-2025"; the older one splits the month
/// and the year.
fn period(block: &str, year: Option<&str>) -> Option<String> {
    let p = text(block, "PeriodoReferencia")?;
    match (p.contains(char::is_numeric), year) {
        (false, Some(y)) => Some(format!("{p}-{y}")),
        _ => Some(p),
    }
}

fn read_block(
    kind: &'static str,
    attrs: &str,
    block: &str,
    ticker: &str,
    isin: Option<&str>,
) -> Option<Notice> {
    let ex_date = iso_date(text(block, "DataBase"))?;
    let value = amount(text(block, "ValorProvento").or_else(|| text(block, "ValorProventoCota")))?;
    if value <= Decimal::ZERO {
        return None;
    }
    let year = text(block, "Ano");
    Some(Notice {
        ticker: ticker.to_string(),
        isin: isin.map(str::to_string),
        kind,
        amortization: (kind == "amortization")
            .then(|| attr(attrs, "tipo"))
            .flatten(),
        ex_date,
        value: value.normalize(),
        pay_date: iso_date(text(block, "DataPagamento")),
        period: period(block, year.as_deref()),
        exempt: (kind == "fii_income")
            .then(|| exempt(text(block, "RendimentoIsentoIR")))
            .flatten(),
    })
}

fn read_securities(scope: &str, ticker: &str, isin: Option<&str>, out: &mut Vec<Notice>) {
    for (attrs, block) in elements(scope, "Rendimento") {
        out.extend(read_block("fii_income", attrs, block, ticker, isin));
    }
    for (attrs, block) in elements(scope, "Amortizacao") {
        out.extend(read_block("amortization", attrs, block, ticker, isin));
    }
}

/// Reads one notice. `fallback`: the fund's ticker, for the rare old document without codes.
pub fn parse_notice(bytes: &[u8], fallback: &str) -> std::result::Result<Vec<Notice>, String> {
    let s = std::str::from_utf8(bytes).map_err(|_| "Fundos.NET: XML sem UTF-8".to_string())?;
    let Some((_, report, _)) = element(s, "InformeRendimentos") else {
        return Err("Fundos.NET: documento sem <InformeRendimentos>".into());
    };
    let mut out = vec![];
    let blocks = elements(report, "Provento");
    if blocks.is_empty() {
        // Layout until 2022: one security per document, codes in the header.
        let head = element(s, "DadosGerais").map(|(_, b, _)| b).unwrap_or("");
        let ticker = text(head, "CodNegociacaoCota").unwrap_or_else(|| fallback.to_string());
        let isin = text(head, "CodISINCota");
        read_securities(report, &ticker.to_uppercase(), isin.as_deref(), &mut out);
    } else {
        for (_, p) in blocks {
            let Some(ticker) = text(p, "CodNegociacao") else {
                continue;
            };
            let isin = text(p, "CodISIN");
            read_securities(p, &ticker.to_uppercase(), isin.as_deref(), &mut out);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------
// Targets and storage
// ---------------------------------------------------------------------------------------

/// A fund to collect: the CNPJ, the ticker of the unit and the oldest record date that matters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    pub cnpj: String,
    pub ticker: String,
    pub since: NaiveDate,
}

impl Target {
    pub fn key(&self) -> String {
        format!("fund:{}", self.cnpj)
    }
}

/// CNPJ of a fund asset: the one in the asset's registry or the CVM's (by the ticker).
pub fn fund_cnpj(
    conn: &Connection,
    ticker: &str,
    registered: Option<&str>,
) -> Result<Option<String>> {
    if let Some(c) = registered.map(super::cnpj_digits)
        && c.len() == 14
    {
        return Ok(Some(c));
    }
    let id: Option<String> = conn
        .query_row(
            "SELECT s.issuer_id FROM research_security s JOIN research_issuer i ON i.id = s.issuer_id
              WHERE s.ticker = ?1 AND i.kind = 'fii'",
            [ticker.to_uppercase()],
            |r| r.get(0),
        )
        .ok();
    Ok(id.filter(|c| c.len() == 14 && c.chars().all(|x| x.is_ascii_digit())))
}

/// Funds the family holds or held (FII class, with a trade or received through a ticker
/// change), with the date of the first trade.
pub fn targets(conn: &Connection) -> Result<Vec<Target>> {
    let rows: Vec<(String, Option<String>, String)> = conn
        .prepare(
            "SELECT a.ticker, a.cnpj, min(f.date) FROM asset a
               JOIN (SELECT asset_id, date FROM trade
                     UNION ALL
                     SELECT ca.new_asset_id, t.date FROM corporate_action ca JOIN trade t ON t.asset_id = ca.asset_id
                      WHERE ca.new_asset_id IS NOT NULL AND ca.kind IN ('ticker_change', 'merger')) f
                 ON f.asset_id = a.id
              WHERE a.class = 'fii' AND a.ticker IS NOT NULL
              GROUP BY a.id",
        )?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut by_cnpj: HashMap<String, Target> = HashMap::new();
    for (ticker, cnpj, first) in rows {
        let Some(cnpj) = fund_cnpj(conn, &ticker, cnpj.as_deref())? else {
            continue;
        };
        let Ok(first) = crate::time::parse_date(&first) else {
            continue;
        };
        let since = first - chrono::Duration::days(MARGIN_DAYS);
        by_cnpj
            .entry(cnpj.clone())
            .and_modify(|t| t.since = t.since.min(since))
            .or_insert(Target {
                cnpj,
                ticker: ticker.to_uppercase(),
                since,
            });
    }
    let mut out: Vec<Target> = by_cnpj.into_values().collect();
    out.sort();
    Ok(out)
}

fn log_fetch(
    conn: &Connection,
    key: &str,
    status: &str,
    detail: Option<&str>,
    now: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO research_fetch_log (source, key, fetched_at, status, detail) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (source, key) DO UPDATE SET fetched_at = excluded.fetched_at, status = excluded.status,
             detail = excluded.detail",
        params![SOURCE, key, now, status, detail],
    )?;
    Ok(())
}

fn load_log(conn: &Connection) -> Result<HashMap<String, (DateTime<Utc>, String)>> {
    let rows: Vec<(String, String, String)> = conn
        .prepare("SELECT key, fetched_at, status FROM research_fetch_log WHERE source = ?1")?
        .query_map([SOURCE], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(k, at, st)| {
            let at = DateTime::parse_from_rfc3339(&at).ok()?.with_timezone(&Utc);
            Some((k, (at, st)))
        })
        .collect())
}

/// Stores the notices of one document (replacing what the same document had).
pub fn store(
    conn: &Connection,
    cnpj: &str,
    doc: &DocRef,
    rows: &[Notice],
    now: &str,
) -> Result<u64> {
    conn.execute(
        "DELETE FROM research_fund_notice WHERE document_id = ?1",
        [doc.id],
    )?;
    let mut n = 0;
    for r in rows {
        n += conn.execute(
            "INSERT OR REPLACE INTO research_fund_notice (document_id, version, cnpj, ticker, isin, kind, amortization,
                 ex_date, value, pay_date, period, exempt, delivered_at, source, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                doc.id,
                doc.version,
                cnpj,
                r.ticker,
                r.isin,
                r.kind,
                r.amortization,
                fmt_date(r.ex_date),
                r.value.normalize().to_string(),
                r.pay_date.map(fmt_date),
                r.period,
                r.exempt.map(i64::from),
                doc.delivered_at,
                SOURCE,
                now
            ],
        )? as u64;
    }
    Ok(n)
}

/// Removes the notices of documents the fund no longer has as active (filed again or
/// cancelled), among those with a record date from `since` on.
pub fn prune(
    conn: &Connection,
    cnpj: &str,
    active: &HashSet<i64>,
    since: NaiveDate,
) -> Result<u64> {
    let ids: Vec<i64> = conn
        .prepare("SELECT DISTINCT document_id FROM research_fund_notice WHERE cnpj = ?1 AND ex_date >= ?2")?
        .query_map(params![cnpj, fmt_date(since)], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut n = 0;
    for id in ids.into_iter().filter(|id| !active.contains(id)) {
        n += conn.execute(
            "DELETE FROM research_fund_notice WHERE document_id = ?1",
            [id],
        )? as u64;
    }
    Ok(n)
}

// ---------------------------------------------------------------------------------------
// Collection
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outcome {
    pub funds: usize,
    pub requests: u32,
    pub documents: u32,
    pub rows: u64,
    pub errors: u32,
    pub stopped: bool,
}

fn get(r: &mut Runner<'_>, url: &str) -> std::result::Result<Option<Vec<u8>>, String> {
    let res = r.fetcher.get(url, None, "");
    let delay = r.fetcher.polite_delay(url);
    if !delay.is_zero() {
        std::thread::sleep(delay);
    }
    match res? {
        Fetched::Body { bytes, .. } => {
            r.rep.bytes += bytes.len() as u64;
            Ok(Some(bytes))
        }
        Fetched::NotFound => Ok(None),
        Fetched::NotModified => Err(crate::i18n::t("nucleo-pesquisa-erro-304")),
    }
}

/// Every active document of a fund (all pages).
fn list_all(
    r: &mut Runner<'_>,
    cnpj: &str,
    requests: &mut u32,
) -> std::result::Result<Vec<DocRef>, String> {
    let mut docs = vec![];
    for page in 0..MAX_PAGES {
        let start = page * PAGE_SIZE;
        *requests += 1;
        let Some(bytes) = get(r, &list_url(cnpj, start))? else {
            break;
        };
        let p = parse_list(&bytes)?;
        let n = p.docs.len() as u32;
        docs.extend(p.docs);
        if n == 0 || start + n >= p.total {
            break;
        }
    }
    Ok(docs)
}

/// Collects the notices of the funds the family holds or held. Network outside the database;
/// each document stored in its own transaction.
pub(crate) fn sync(r: &mut Runner<'_>, _today: NaiveDate) -> Result<Outcome> {
    let force = r.plan.force;
    let now = Utc::now();
    let (targets, log) = r.with(|c| Ok((targets(c)?, load_log(c)?)))?;
    let due: Vec<Target> = targets
        .into_iter()
        .filter(|t| {
            force
                || log.get(&t.key()).is_none_or(|(at, st)| {
                    let wait = if st == "ok" {
                        REFRESH_HOURS
                    } else {
                        ERROR_RETRY_HOURS
                    };
                    (now - *at).num_hours() >= wait
                })
        })
        .collect();
    let mut out = Outcome {
        funds: due.len(),
        ..Default::default()
    };
    let mut consecutive = 0u32;
    let mut errors: Vec<SyncItem> = vec![];
    let mut last_error: Option<String> = None;
    for t in &due {
        if consecutive >= MAX_CONSECUTIVE_ERRORS {
            out.stopped = true;
            break;
        }
        crate::tasks::check()?;
        crate::tasks::detail(format!("Fundos.NET {}", t.ticker));
        let docs = match list_all(r, &t.cnpj, &mut out.requests) {
            Ok(d) => d,
            Err(e) => {
                consecutive += 1;
                out.errors += 1;
                let stamp = now_utc();
                r.with(|c| log_fetch(c, &t.key(), "error", Some(&e), &stamp))?;
                errors.push(SyncItem {
                    source: SyncSource::FundosNet,
                    label: format!("Fundos.NET {}", t.ticker),
                    url: Some(site_url(&t.cnpj)),
                    status: SyncStatus::Error,
                    bytes: 0,
                    rows: 0,
                    error: Some(e.clone()),
                });
                last_error = Some(e);
                continue;
            }
        };
        consecutive = 0;
        let active: HashSet<i64> = docs.iter().map(|d| d.id).collect();
        let fetched: HashSet<i64> = r.with(|c| {
            let ok: Vec<i64> = c
                .prepare("SELECT DISTINCT document_id FROM research_fund_notice WHERE cnpj = ?1")?
                .query_map([&t.cnpj], |x| x.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            Ok(ok.into_iter().collect())
        })?;
        let mut fund_error = false;
        for d in docs
            .iter()
            .filter(|d| d.ref_date.is_none_or(|x| x >= t.since))
        {
            let key = format!("doc:{}", d.id);
            let done = fetched.contains(&d.id)
                || log
                    .get(&key)
                    .is_some_and(|(_, st)| st == "ok" || st == "empty");
            if done && !force {
                continue;
            }
            if consecutive >= MAX_CONSECUTIVE_ERRORS {
                out.stopped = true;
                break;
            }
            crate::tasks::check()?;
            out.requests += 1;
            let res = get(r, &document_url(d.id))
                .and_then(|b| b.map(|b| parse_notice(&b, &t.ticker)).transpose());
            let stamp = now_utc();
            match res {
                Ok(rows) => {
                    consecutive = 0;
                    let rows = rows.unwrap_or_default();
                    out.documents += 1;
                    let n = r.with(|c| {
                        let tx = c.transaction()?;
                        let n = store(&tx, &t.cnpj, d, &rows, &stamp)?;
                        log_fetch(
                            &tx,
                            &key,
                            if rows.is_empty() { "empty" } else { "ok" },
                            None,
                            &stamp,
                        )?;
                        tx.commit()?;
                        Ok(n)
                    })?;
                    out.rows += n;
                }
                Err(e) => {
                    consecutive += 1;
                    out.errors += 1;
                    fund_error = true;
                    r.with(|c| log_fetch(c, &key, "error", Some(&e), &stamp))?;
                    last_error = Some(e);
                }
            }
        }
        let stamp = now_utc();
        r.with(|c| {
            let tx = c.transaction()?;
            prune(&tx, &t.cnpj, &active, t.since)?;
            log_fetch(
                &tx,
                &t.key(),
                if fund_error { "error" } else { "ok" },
                None,
                &stamp,
            )?;
            tx.commit()?;
            Ok(())
        })?;
        if fund_error {
            errors.push(SyncItem {
                source: SyncSource::FundosNet,
                label: format!("Fundos.NET {}", t.ticker),
                url: Some(site_url(&t.cnpj)),
                status: SyncStatus::Error,
                bytes: 0,
                rows: 0,
                error: last_error.clone(),
            });
        }
    }
    let status = if out.funds == 0 {
        SyncStatus::Unchanged
    } else if out.errors == 0 || out.documents > 0 {
        SyncStatus::Updated
    } else {
        SyncStatus::Error
    };
    let mut summary = SyncItem {
        source: SyncSource::FundosNet,
        label: crate::i18n::t_args(
            "nucleo-pesquisa-fnet-resumo",
            &crate::i18n::args(&[
                ("fundos", (out.funds as i64).into()),
                ("documentos", i64::from(out.documents).into()),
            ]),
        ),
        url: None,
        status,
        bytes: 0,
        rows: out.rows,
        error: None,
    };
    if out.stopped {
        summary.error = Some(crate::i18n::t_args(
            "nucleo-pesquisa-fnet-parou",
            &crate::i18n::args(&[("vezes", i64::from(MAX_CONSECUTIVE_ERRORS).into())]),
        ));
    }
    if out.errors > 0 {
        r.rep.ok = false;
    }
    r.rep.items.push(summary);
    r.rep.items.extend(errors);
    if out.requests > 0 {
        let res: std::result::Result<(), String> = if out.errors == 0 || out.documents > 0 {
            Ok(())
        } else {
            Err(last_error.unwrap_or_else(|| "Fundos.NET".into()))
        };
        let today = crate::time::today_local();
        r.with(|c| {
            crate::sources::record_result(c, crate::sources::SourceId::FundosNet, &res, today)
        })?;
    }
    Ok(out)
}

/// Notices of a ticker as stored (one per record date and kind: when two active documents
/// report the same distribution, the latest filing wins).
#[derive(Debug, Clone, PartialEq)]
pub struct StoredNotice {
    pub document_id: i64,
    pub ticker: String,
    pub kind: String,
    pub ex_date: NaiveDate,
    pub value: Decimal,
    pub pay_date: Option<NaiveDate>,
    pub period: Option<String>,
    pub exempt: Option<bool>,
    pub cnpj: String,
}

pub fn notices(conn: &Connection, ticker: &str, since: NaiveDate) -> Result<Vec<StoredNotice>> {
    type Row = (
        i64,
        i64,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<i64>,
        Option<String>,
        String,
    );
    let rows: Vec<Row> = conn
        .prepare_cached(
            "SELECT document_id, version, ticker, kind, ex_date, value, pay_date, period, exempt, delivered_at, cnpj
               FROM research_fund_notice WHERE ticker = ?1 AND ex_date >= ?2 ORDER BY ex_date, kind",
        )?
        .query_map(params![ticker.to_uppercase(), fmt_date(since)], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
                r.get(10)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    // Latest filing per (kind, record date): delivery time, then version, then document id.
    let mut best: HashMap<(String, String), (String, i64, i64, StoredNotice)> = HashMap::new();
    for (doc, version, ticker, kind, ex, value, pay, period, exempt, delivered, cnpj) in rows {
        let (Ok(ex_date), Ok(value)) = (crate::time::parse_date(&ex), value.parse::<Decimal>())
        else {
            continue;
        };
        let rank = delivered
            .as_deref()
            .and_then(|d| chrono::NaiveDateTime::parse_from_str(d.trim(), "%d/%m/%Y %H:%M").ok())
            .map(|d| d.format("%Y%m%d%H%M").to_string())
            .unwrap_or_default();
        let n = StoredNotice {
            document_id: doc,
            ticker,
            kind: kind.clone(),
            ex_date,
            value,
            pay_date: pay.and_then(|p| crate::time::parse_date(&p).ok()),
            period,
            exempt: exempt.map(|e| e == 1),
            cnpj,
        };
        let k = (kind, ex);
        let better = best
            .get(&k)
            .is_none_or(|(r, v, d, _)| (rank.as_str(), version, doc) > (r.as_str(), *v, *d));
        if better {
            best.insert(k, (rank, version, doc, n));
        }
    }
    let mut out: Vec<StoredNotice> = best.into_values().map(|(.., n)| n).collect();
    out.sort_by(|a, b| a.ex_date.cmp(&b.ex_date).then(a.kind.cmp(&b.kind)));
    Ok(out)
}

#[cfg(test)]
pub(crate) mod fixtures {
    //! Samples in the real format of the responses (invented fund, people and amounts).

    pub const LIST: &str = r#"{"draw":1,"recordsFiltered":3,"recordsTotal":3,"data":[
      {"id":900003,"categoriaDocumento":"Aviso aos Cotistas - Estruturado","tipoDocumento":"Rendimentos e Amortizações","dataReferencia":"28/11/2025","dataEntrega":"28/11/2025 18:10","descricaoModalidade":"Apresentação","status":"AC","versao":1,"situacaoDocumento":"A"},
      {"id":900002,"categoriaDocumento":"Aviso aos Cotistas - Estruturado","tipoDocumento":"Rendimentos e Amortizações","dataReferencia":"31/10/2025","dataEntrega":"03/11/2025 15:48","descricaoModalidade":"Reapresentação Espontânea","status":"AC","versao":2,"situacaoDocumento":"A"},
      {"id":100001,"categoriaDocumento":"Aviso aos Cotistas - Estruturado","tipoDocumento":"Rendimentos e Amortizações","dataReferencia":"09/03/2021","dataEntrega":"09/03/2021 11:14","descricaoModalidade":"Apresentação","status":"AC","versao":1,"situacaoDocumento":"A"}]}"#;

    /// Layout since 2022: unit and subscription receipt in the same document.
    pub const NOTICE_NEW: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<DadosEconomicoFinanceiros>
	<DadosGerais>
		<NomeFundo>FUNDO EXEMPLO FII</NomeFundo>
		<CNPJFundo>11222333000144</CNPJFundo>
		<NomeAdministrador>Administradora Exemplo DTVM</NomeAdministrador>
		<CNPJAdministrador>55666777000188</CNPJAdministrador>
		<ResponsavelInformacao>Pessoa Inventada</ResponsavelInformacao>
		<TelefoneContato>(11) 0000-0000</TelefoneContato>
		<DataInformacao>2025-11-28</DataInformacao>
		<Ano>2025</Ano>
	</DadosGerais>
	<InformeRendimentos>
		<Provento>
			<CodISIN>BRXMPLCTF001</CodISIN>
			<CodNegociacao>XMPL11</CodNegociacao>
			<Rendimento>
				<AtoSocietarioAprovacao/>
				<DataBase>2025-11-28</DataBase>
				<ValorProvento>0.100000001</ValorProvento>
				<DataPagamento>2025-12-12</DataPagamento>
				<PeriodoReferencia>Novembro-2025</PeriodoReferencia>
				<RendimentoIsentoIR>Sim</RendimentoIsentoIR>
			</Rendimento>
		</Provento>
		<Provento>
			<CodISIN>BRXMPLR13M17</CodISIN>
			<CodNegociacao>XMPL13</CodNegociacao>
			<Rendimento>
				<AtoSocietarioAprovacao/>
				<DataBase>2025-11-28</DataBase>
				<ValorProvento>0.069235596</ValorProvento>
				<DataPagamento>2025-12-12</DataPagamento>
				<PeriodoReferencia>Novembro-2025</PeriodoReferencia>
				<RendimentoIsentoIR>Não</RendimentoIsentoIR>
			</Rendimento>
		</Provento>
	</InformeRendimentos>
</DadosEconomicoFinanceiros>"#;

    pub const NOTICE_AMORTIZATION: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<DadosEconomicoFinanceiros>
	<DadosGerais><NomeFundo>FUNDO EXEMPLO FII</NomeFundo><CNPJFundo>11222333000144</CNPJFundo></DadosGerais>
	<InformeRendimentos>
		<Provento>
			<CodISIN>BRXMPLCTF001</CodISIN>
			<CodNegociacao>XMPL11</CodNegociacao>
			<Amortizacao tipo="Total">
				<AtoSocietarioAprovacao/>
				<DataBase>2025-10-31</DataBase>
				<ValorProvento>0.330560945</ValorProvento>
				<DataPagamento>2025-11-10</DataPagamento>
				<PeriodoReferencia>Outubro-2025</PeriodoReferencia>
			</Amortizacao>
		</Provento>
	</InformeRendimentos>
</DadosEconomicoFinanceiros>"#;

    /// Layout until 2022: codes in the header, empty amortization element.
    pub const NOTICE_OLD: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<DadosEconomicoFinanceiros xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
	<DadosGerais>
		<NomeFundo>FUNDO EXEMPLO FII</NomeFundo>
		<CNPJFundo>11222333000144</CNPJFundo>
		<ResponsavelInformacao>Pessoa Inventada</ResponsavelInformacao>
		<CodISINCota>BRXMPLCTF001</CodISINCota>
		<CodNegociacaoCota>XMPL11</CodNegociacaoCota>
	</DadosGerais>
	<InformeRendimentos>
		<Rendimento>
			<DataAprovacao>2021-03-09</DataAprovacao>
			<DataBase>2021-03-09</DataBase>
			<DataPagamento>2021-03-16</DataPagamento>
			<ValorProventoCota>0.6</ValorProventoCota>
			<PeriodoReferencia>Fevereiro</PeriodoReferencia>
			<Ano>2021</Ano>
			<RendimentoIsentoIR>true</RendimentoIsentoIR>
		</Rendimento>
		<Amortizacao tipo=""/>
	</InformeRendimentos>
</DadosEconomicoFinanceiros>"#;
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    #[test]
    fn reads_the_list_of_active_documents() {
        let p = parse_list(LIST.as_bytes()).unwrap();
        assert_eq!(p.total, 3);
        assert_eq!(p.docs.len(), 3);
        assert_eq!(p.docs[1].id, 900002);
        assert_eq!(p.docs[1].version, 2);
        assert_eq!(p.docs[1].ref_date, Some(d("2025-10-31")));
        assert_eq!(p.docs[1].delivered_at.as_deref(), Some("03/11/2025 15:48"));
    }

    #[test]
    fn reads_the_current_layout_with_units_and_receipts() {
        let n = parse_notice(NOTICE_NEW.as_bytes(), "XMPL11").unwrap();
        assert_eq!(n.len(), 2);
        assert_eq!(n[0].ticker, "XMPL11");
        assert_eq!(n[0].isin.as_deref(), Some("BRXMPLCTF001"));
        assert_eq!(n[0].kind, "fii_income");
        assert_eq!(n[0].ex_date, d("2025-11-28"));
        assert_eq!(n[0].value.to_string(), "0.100000001");
        assert_eq!(n[0].pay_date, Some(d("2025-12-12")));
        assert_eq!(n[0].period.as_deref(), Some("Novembro-2025"));
        assert_eq!(n[0].exempt, Some(true));
        assert_eq!(n[1].ticker, "XMPL13");
        assert_eq!(n[1].exempt, Some(false));
    }

    #[test]
    fn reads_an_amortization_with_its_type() {
        let n = parse_notice(NOTICE_AMORTIZATION.as_bytes(), "XMPL11").unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].kind, "amortization");
        assert_eq!(n[0].amortization.as_deref(), Some("Total"));
        assert_eq!(n[0].value.to_string(), "0.330560945");
        assert_eq!(n[0].exempt, None);
    }

    #[test]
    fn reads_the_old_layout_and_skips_the_empty_amortization() {
        let n = parse_notice(NOTICE_OLD.as_bytes(), "OTHER11").unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].ticker, "XMPL11");
        assert_eq!(n[0].isin.as_deref(), Some("BRXMPLCTF001"));
        assert_eq!(n[0].value.to_string(), "0.6");
        assert_eq!(n[0].period.as_deref(), Some("Fevereiro-2021"));
        assert_eq!(n[0].exempt, Some(true));
    }

    #[test]
    fn rejects_a_document_without_the_report() {
        assert!(parse_notice(b"<html>erro</html>", "XMPL11").is_err());
    }

    #[test]
    fn the_latest_filing_wins_and_inactive_documents_are_pruned() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST)
            .unwrap();
        let c = db.conn();
        let now = "2025-12-01T00:00:00Z";
        let notice = |v: &str| Notice {
            ticker: "XMPL11".into(),
            isin: None,
            kind: "fii_income",
            amortization: None,
            ex_date: d("2025-10-31"),
            value: v.parse().unwrap(),
            pay_date: Some(d("2025-11-14")),
            period: None,
            exempt: Some(true),
        };
        let first = DocRef {
            id: 1,
            version: 1,
            ref_date: Some(d("2025-10-31")),
            delivered_at: Some("31/10/2025 17:00".into()),
        };
        let again = DocRef {
            id: 2,
            version: 2,
            ref_date: Some(d("2025-10-31")),
            delivered_at: Some("03/11/2025 15:48".into()),
        };
        store(c, "11222333000144", &first, &[notice("0.09")], now).unwrap();
        store(c, "11222333000144", &again, &[notice("0.10")], now).unwrap();
        let got = notices(c, "xmpl11", d("2025-01-01")).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].value.to_string(), "0.1");
        // The fund lists only the corrected document: the old one goes away.
        let active: HashSet<i64> = [2].into_iter().collect();
        assert_eq!(
            prune(c, "11222333000144", &active, d("2025-01-01")).unwrap(),
            1
        );
        assert_eq!(
            notices(c, "XMPL11", d("2025-01-01")).unwrap()[0].document_id,
            2
        );
    }
}

#[cfg(test)]
mod sync_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use super::super::net::{FetchResult, Fetched, Fetcher, FileMeta};
    use super::super::sync::{SyncArgs, SyncSource, run_with};
    use super::fixtures::*;
    use super::*;
    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    #[derive(Default)]
    struct Fixture {
        files: HashMap<String, Vec<u8>>,
        calls: Mutex<Vec<String>>,
    }

    impl Fetcher for Fixture {
        fn get(&self, url: &str, _cond: Option<&FileMeta>, _ua: &str) -> FetchResult {
            self.calls.lock().unwrap().push(url.to_string());
            match self.files.get(url) {
                None => Ok(Fetched::NotFound),
                Some(b) => Ok(Fetched::Body {
                    bytes: b.clone(),
                    meta: FileMeta::default(),
                }),
            }
        }
    }

    #[test]
    fn collects_each_notice_once_for_the_funds_held() {
        let dir = tempfile::tempdir().unwrap();
        let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
        let api = Arc::new(Api::new_unlocked_process(paths).unwrap());
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "Família Teste"}),
        )
        .unwrap();
        let fund = api
            .call(
                "assets.create",
                json!({"ticker": "XMPL11", "name": "Fundo Exemplo", "class": "fii", "cnpj": "11.222.333/0001-44"}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        // A fund without trades is not collected.
        api.call(
            "assets.create",
            json!({"ticker": "OUTR11", "name": "Outro Fundo", "class": "fii", "cnpj": "99888777000166"}),
        )
        .unwrap();
        api.with_session(|s, _| {
            let c = s.db.conn();
            c.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
                 VALUES ('corretora', 'Corretora', 'brokerage', 'BRL', '2025-01-01', 'x', 'x')",
                [],
            )?;
            c.execute(
                "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
                 VALUES ('t1', ?1, 'corretora', '2025-06-10', 'buy', '100', '9', 'BRL', 'x', 'x')",
                [&fund],
            )?;
            Ok(())
        })
        .unwrap();
        let mut f = Fixture::default();
        f.files
            .insert(list_url("11222333000144", 0), LIST.as_bytes().to_vec());
        f.files
            .insert(document_url(900003), NOTICE_NEW.as_bytes().to_vec());
        f.files.insert(
            document_url(900002),
            NOTICE_AMORTIZATION.as_bytes().to_vec(),
        );
        // 100001 (2021) is older than the first purchase: not downloaded.
        let args = SyncArgs {
            force: None,
            sources: Some(vec![SyncSource::FundosNet]),
        };
        let today = crate::time::parse_date("2025-12-01").unwrap();
        run_with(&api, &args, &f, today).unwrap();
        let calls = f.calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 3, "{calls:#?}");
        assert!(!calls.iter().any(|u| u.contains("100001")));
        let got = api
            .with_session(|s, _| {
                notices(s.db.conn(), "XMPL11", today - chrono::Duration::days(400))
            })
            .unwrap();
        assert_eq!(got.len(), 2, "{got:?}");
        assert_eq!(got[0].kind, "amortization");
        assert_eq!(got[1].kind, "fii_income");
        // The receipt's notice is stored under its own code.
        let receipt = api
            .with_session(|s, _| {
                notices(s.db.conn(), "XMPL13", today - chrono::Duration::days(400))
            })
            .unwrap();
        assert_eq!(receipt.len(), 1);
        // Second run within the refresh window: nothing is requested.
        run_with(&api, &args, &f, today).unwrap();
        assert_eq!(f.calls.lock().unwrap().len(), 3);
        // Forced: the list again, but the documents already stored are not downloaded.
        let forced = SyncArgs {
            force: Some(true),
            sources: Some(vec![SyncSource::FundosNet]),
        };
        let before = f.calls.lock().unwrap().len();
        run_with(&api, &forced, &f, today).unwrap();
        let after = f.calls.lock().unwrap().clone();
        assert!(after.len() > before);
    }
}
