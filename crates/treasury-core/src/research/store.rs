//! Leitura e gravação das tabelas técnicas da análise de ativos (fora do histórico de
//! alterações: dados baixados, sempre com fonte, documento e data de coleta).

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Row, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{Fact, Issuer, IssuerKind, Layout, PeriodType, Security, dstr};
use crate::error::{Error, Result};
use crate::time::{fmt_date, parse_date};

pub fn upsert_issuer(conn: &Connection, i: &Issuer, fetched_at: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO research_issuer (id, kind, name, cvm_code, sector, segment, mandate, layout, currency, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT (id) DO UPDATE SET
             kind = excluded.kind,
             name = CASE WHEN excluded.name <> '' THEN excluded.name ELSE research_issuer.name END,
             cvm_code = coalesce(excluded.cvm_code, research_issuer.cvm_code),
             sector = coalesce(excluded.sector, research_issuer.sector),
             segment = coalesce(excluded.segment, research_issuer.segment),
             mandate = coalesce(excluded.mandate, research_issuer.mandate),
             layout = excluded.layout,
             currency = excluded.currency,
             source = excluded.source,
             fetched_at = excluded.fetched_at",
        params![
            i.id,
            i.kind.as_str(),
            i.name,
            i.cvm_code,
            i.sector,
            i.segment,
            i.mandate,
            i.layout.as_str(),
            i.currency,
            i.source,
            fetched_at
        ],
    )?;
    Ok(())
}

/// Atualiza o leiaute (vindo da DRE) e o nome sem mexer no setor do FCA.
pub fn set_layout(conn: &Connection, id: &str, layout: Layout) -> Result<()> {
    conn.execute(
        "UPDATE research_issuer SET layout = ?2 WHERE id = ?1",
        params![id, layout.as_str()],
    )?;
    Ok(())
}

pub fn upsert_security(conn: &Connection, s: &Security, fetched_at: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO research_security (ticker, issuer_id, class, share_kind, unit_composition, isin, listing_segment, active, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT (ticker) DO UPDATE SET issuer_id = excluded.issuer_id, class = excluded.class,
             share_kind = excluded.share_kind, unit_composition = excluded.unit_composition,
             isin = coalesce(excluded.isin, research_security.isin),
             listing_segment = coalesce(excluded.listing_segment, research_security.listing_segment),
             active = excluded.active, source = excluded.source, fetched_at = excluded.fetched_at",
        params![
            s.ticker,
            s.issuer_id,
            s.class,
            s.share_kind,
            s.unit_composition,
            s.isin,
            s.listing_segment,
            s.active as i64,
            s.source,
            fetched_at
        ],
    )?;
    Ok(())
}

/// Grava fatos de um emissor. Um fato só substitui outro do mesmo período com prioridade
/// igual ou maior (o original vence a reapresentação no documento seguinte).
pub fn upsert_facts(
    conn: &Connection,
    issuer_id: &str,
    facts: &[Fact],
    fetched_at: &str,
) -> Result<usize> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO fundamental (issuer_id, item, period_type, period_end, period_start, value, currency, source, document, priority, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT (issuer_id, item, period_type, period_end) DO UPDATE SET
             period_start = excluded.period_start, value = excluded.value, currency = excluded.currency,
             source = excluded.source, document = excluded.document, priority = excluded.priority,
             fetched_at = excluded.fetched_at
         WHERE excluded.priority >= fundamental.priority",
    )?;
    let mut n = 0;
    for f in facts {
        n += stmt.execute(params![
            issuer_id,
            f.item,
            f.period_type.as_str(),
            fmt_date(f.end),
            f.start.map(fmt_date),
            dstr(f.value),
            f.currency,
            f.source,
            f.document,
            f.priority,
            fetched_at
        ])?;
    }
    Ok(n)
}

fn row_to_fact(r: &Row<'_>) -> rusqlite::Result<(String, Fact)> {
    let conv = |e: Error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    };
    let pt: String = r.get("period_type")?;
    let end: String = r.get("period_end")?;
    let start: Option<String> = r.get("period_start")?;
    let value: String = r.get("value")?;
    Ok((
        r.get("issuer_id")?,
        Fact {
            item: r.get("item")?,
            period_type: PeriodType::parse(&pt).map_err(conv)?,
            start: start.map(|s| parse_date(&s)).transpose().map_err(conv)?,
            end: parse_date(&end).map_err(conv)?,
            value: value.parse::<Decimal>().map_err(|_| {
                conv(Error::invalid(crate::i18n::t_args(
                    "nucleo-pesquisa-erro-valor-banco",
                    &crate::i18n::args(&[("valor", value.clone().into())]),
                )))
            })?,
            currency: r.get("currency")?,
            source: r.get("source")?,
            document: r.get("document")?,
            priority: r.get("priority")?,
        },
    ))
}

pub fn load_facts(conn: &Connection, issuer_id: &str) -> Result<Vec<Fact>> {
    let mut stmt = conn.prepare_cached(
        "SELECT * FROM fundamental WHERE issuer_id = ?1 ORDER BY item, period_type, period_end",
    )?;
    let rows = stmt
        .query_map([issuer_id], row_to_fact)?
        .map(|r| r.map(|(_, f)| f))
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Fatos de todos os emissores (para recalcular o universo de uma vez).
pub fn load_all_facts(conn: &Connection) -> Result<HashMap<String, Vec<Fact>>> {
    let mut stmt = conn
        .prepare("SELECT * FROM fundamental ORDER BY issuer_id, item, period_type, period_end")?;
    let mut out: HashMap<String, Vec<Fact>> = HashMap::new();
    for r in stmt.query_map([], row_to_fact)? {
        let (id, f) = r?;
        out.entry(id).or_default().push(f);
    }
    Ok(out)
}

/// Troca os fatos LTM de um emissor pelos recalculados.
pub fn replace_ltm(
    conn: &Connection,
    issuer_id: &str,
    ltm: &[Fact],
    fetched_at: &str,
) -> Result<()> {
    conn.execute(
        "DELETE FROM fundamental WHERE issuer_id = ?1 AND period_type = 'ltm'",
        [issuer_id],
    )?;
    upsert_facts(conn, issuer_id, ltm, fetched_at)?;
    Ok(())
}

fn row_to_issuer(r: &Row<'_>) -> rusqlite::Result<Issuer> {
    Ok(Issuer {
        id: r.get("id")?,
        kind: IssuerKind::parse(&r.get::<_, String>("kind")?),
        name: r.get("name")?,
        cvm_code: r.get("cvm_code")?,
        sector: r.get("sector")?,
        segment: r.get("segment")?,
        mandate: r.get("mandate")?,
        layout: Layout::parse(&r.get::<_, String>("layout")?),
        currency: r.get("currency")?,
        source: r.get("source")?,
    })
}

pub fn issuer(conn: &Connection, id: &str) -> Result<Option<Issuer>> {
    Ok(conn
        .query_row(
            "SELECT * FROM research_issuer WHERE id = ?1",
            [id],
            row_to_issuer,
        )
        .optional()?)
}

pub fn all_issuers(conn: &Connection) -> Result<HashMap<String, Issuer>> {
    let mut stmt = conn.prepare("SELECT * FROM research_issuer")?;
    let rows = stmt
        .query_map([], row_to_issuer)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows.into_iter().map(|i| (i.id.clone(), i)).collect())
}

fn row_to_security(r: &Row<'_>) -> rusqlite::Result<Security> {
    Ok(Security {
        ticker: r.get("ticker")?,
        issuer_id: r.get("issuer_id")?,
        class: r.get("class")?,
        share_kind: r.get("share_kind")?,
        unit_composition: r.get("unit_composition")?,
        isin: r.get("isin")?,
        listing_segment: r.get("listing_segment")?,
        active: r.get::<_, i64>("active")? == 1,
        source: r.get("source")?,
    })
}

pub fn security(conn: &Connection, ticker: &str) -> Result<Option<Security>> {
    Ok(conn
        .query_row(
            "SELECT * FROM research_security WHERE ticker = upper(?1)",
            [ticker.trim()],
            row_to_security,
        )
        .optional()?)
}

pub fn securities_of(conn: &Connection, issuer_id: &str) -> Result<Vec<Security>> {
    let mut stmt = conn
        .prepare_cached("SELECT * FROM research_security WHERE issuer_id = ?1 ORDER BY ticker")?;
    let rows = stmt
        .query_map([issuer_id], row_to_security)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn all_securities(conn: &Connection) -> Result<Vec<Security>> {
    let mut stmt = conn.prepare("SELECT * FROM research_security ORDER BY ticker")?;
    let rows = stmt
        .query_map([], row_to_security)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Cotação do universo (B3 COTAHIST): fechamento, data e volume financeiro médio.
#[derive(Debug, Clone, PartialEq)]
pub struct UniverseQuote {
    pub close: Decimal,
    pub date: NaiveDate,
    pub source: String,
}

pub fn latest_quote(
    conn: &Connection,
    ticker: &str,
    at: NaiveDate,
) -> Result<Option<UniverseQuote>> {
    let row: Option<(String, String, String)> = conn
        .prepare_cached(
            "SELECT close, date, source FROM research_quote WHERE ticker = ?1 AND date <= ?2 ORDER BY date DESC LIMIT 1",
        )?
        .query_row(params![ticker, fmt_date(at)], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .optional()?;
    let Some((c, d, s)) = row else {
        return Ok(None);
    };
    Ok(Some(UniverseQuote {
        close: c.parse().map_err(|_| {
            Error::invalid(crate::i18n::t_args(
                "nucleo-pesquisa-erro-cotacao-banco",
                &crate::i18n::args(&[("cotacao", c.clone().into())]),
            ))
        })?,
        date: parse_date(&d)?,
        source: s,
    }))
}

/// Volume financeiro médio dos últimos `n` pregões do universo (e quantos pregões entraram).
pub fn avg_quote_volume(conn: &Connection, ticker: &str, n: u32) -> Result<Option<(Decimal, u32)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT volume FROM research_quote WHERE ticker = ?1 ORDER BY date DESC LIMIT ?2",
    )?;
    let vols: Vec<String> = stmt
        .query_map(params![ticker, n], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    if vols.is_empty() {
        return Ok(None);
    }
    let sum: Decimal = vols.iter().filter_map(|v| v.parse::<Decimal>().ok()).sum();
    Ok(Some((sum / Decimal::from(vols.len()), vols.len() as u32)))
}

/// Entrega de um documento periódico (DFP, ITR, informes mensal e trimestral de FII).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilingInfo {
    /// `dfp`, `itr`, `fii_mensal`, `fii_trimestral`.
    pub kind: String,
    pub reference: NaiveDate,
    pub version: u32,
    /// Número do documento na CVM, quando há.
    pub document: Option<String>,
    pub filed_at: NaiveDate,
    pub link: Option<String>,
    pub fetched_at: String,
}

/// Grava entregas; uma versão mais velha não substitui a mais nova já gravada.
pub fn upsert_filings(
    conn: &Connection,
    recs: &[super::cvm::FilingRecord],
    fetched_at: &str,
) -> Result<usize> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO research_filing (issuer_id, kind, reference, version, document, filed_at, link, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (issuer_id, kind, reference) DO UPDATE SET version = excluded.version,
             document = excluded.document, filed_at = excluded.filed_at, link = excluded.link,
             fetched_at = excluded.fetched_at
         WHERE excluded.version >= research_filing.version",
    )?;
    let mut n = 0;
    for r in recs {
        n += stmt.execute(params![
            r.issuer_id,
            r.kind,
            fmt_date(r.reference),
            r.version,
            r.document,
            fmt_date(r.filed_at),
            r.link,
            fetched_at
        ])?;
    }
    Ok(n)
}

/// Entregas de um emissor, da mais recente para a mais antiga.
pub fn filings(conn: &Connection, issuer_id: &str) -> Result<Vec<FilingInfo>> {
    let mut stmt = conn.prepare_cached(
        "SELECT kind, reference, version, document, filed_at, link, fetched_at FROM research_filing
         WHERE issuer_id = ?1 ORDER BY reference DESC, kind",
    )?;
    // (tipo, referência, versão, documento, entrega, link, coleta)
    type FilingRow = (
        String,
        String,
        u32,
        Option<String>,
        String,
        Option<String>,
        String,
    );
    let rows: Vec<FilingRow> = stmt
        .query_map([issuer_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(
            |(kind, reference, version, document, filed_at, link, fetched_at)| {
                Ok(FilingInfo {
                    kind,
                    reference: parse_date(&reference)?,
                    version,
                    document,
                    filed_at: parse_date(&filed_at)?,
                    link,
                    fetched_at,
                })
            },
        )
        .collect()
}

/// Auditor independente de uma companhia (FCA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditorInfo {
    pub auditor: String,
    /// CPF/CNPJ do auditor só com dígitos.
    pub auditor_cnpj: String,
    pub start: Option<NaiveDate>,
    /// `None` enquanto atua.
    pub end: Option<NaiveDate>,
    /// Data de referência do FCA de onde veio.
    pub reference: NaiveDate,
    pub fetched_at: String,
}

/// Troca os auditores de cada companhia pelos do FCA mais novo. Um FCA mais velho que o já
/// gravado (arquivo do ano anterior processado depois) é ignorado.
pub fn replace_auditors(
    conn: &Connection,
    recs: &[super::cvm::AuditorRecord],
    fetched_at: &str,
) -> Result<usize> {
    let mut by: HashMap<&str, Vec<&super::cvm::AuditorRecord>> = HashMap::new();
    for r in recs {
        by.entry(r.issuer_id.as_str()).or_default().push(r);
    }
    let mut n = 0;
    for (issuer, list) in by {
        let reference = list
            .iter()
            .map(|r| r.reference)
            .max()
            .expect("lista não vazia");
        let stored: Option<String> = conn
            .prepare_cached("SELECT max(reference) FROM research_auditor WHERE issuer_id = ?1")?
            .query_row([issuer], |r| r.get(0))?;
        if stored.is_some_and(|s| s > fmt_date(reference)) {
            continue;
        }
        conn.prepare_cached("DELETE FROM research_auditor WHERE issuer_id = ?1")?
            .execute([issuer])?;
        let mut stmt = conn.prepare_cached(
            "INSERT INTO research_auditor (issuer_id, auditor, auditor_cnpj, start, end, reference, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT (issuer_id, auditor_cnpj, start) DO NOTHING",
        )?;
        for r in list {
            n += stmt.execute(params![
                issuer,
                r.auditor,
                r.auditor_cnpj,
                r.start.map(fmt_date).unwrap_or_default(),
                r.end.map(fmt_date),
                fmt_date(r.reference),
                fetched_at
            ])?;
        }
    }
    Ok(n)
}

/// Auditores de uma companhia (o atual primeiro: sem fim de atuação, início mais recente).
pub fn auditors(conn: &Connection, issuer_id: &str) -> Result<Vec<AuditorInfo>> {
    let mut stmt = conn.prepare_cached(
        "SELECT auditor, auditor_cnpj, start, end, reference, fetched_at FROM research_auditor
         WHERE issuer_id = ?1 ORDER BY end IS NOT NULL, start DESC",
    )?;
    let rows: Vec<(String, String, String, Option<String>, String, String)> = stmt
        .query_map([issuer_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(
            |(auditor, auditor_cnpj, start, end, reference, fetched_at)| {
                Ok(AuditorInfo {
                    auditor,
                    auditor_cnpj,
                    start: (!start.is_empty())
                        .then(|| parse_date(&start))
                        .transpose()?,
                    end: end.map(|e| parse_date(&e)).transpose()?,
                    reference: parse_date(&reference)?,
                    fetched_at,
                })
            },
        )
        .collect()
}

/// Fechamentos do universo (B3 COTAHIST) desde `from`, por papel, em ordem de data.
pub fn quote_series(
    conn: &Connection,
    ticker: Option<&str>,
    from: NaiveDate,
) -> Result<HashMap<String, Vec<(NaiveDate, f64)>>> {
    let mut stmt = conn.prepare(
        "SELECT ticker, date, close FROM research_quote WHERE date >= ?1 AND (?2 IS NULL OR ticker = ?2) ORDER BY ticker, date",
    )?;
    let mut out: HashMap<String, Vec<(NaiveDate, f64)>> = HashMap::new();
    for r in stmt.query_map(params![fmt_date(from), ticker], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (t, d, c) = r?;
        if let (Ok(d), Ok(c)) = (parse_date(&d), c.parse::<Decimal>()) {
            out.entry(t).or_default().push((d, super::to_f64(c)));
        }
    }
    Ok(out)
}
