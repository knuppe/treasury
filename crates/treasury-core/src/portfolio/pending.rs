//! Proventos a confirmar: um candidato por provento, cruzando as fontes. A agenda do ativo
//! (`dividend_event`: Yahoo, Fundamentus), os eventos oficiais da B3 (`research_dividend`) e os
//! avisos oficiais dos fundos no Fundos.NET (`research_fund_notice`) descrevem os mesmos
//! pagamentos com datas e valores às vezes diferentes; aqui eles viram um só, com a conferência
//! de cada fonte.
//!
//! - **Official sources**: for funds, the Fundos.NET notice (exact amount, whole history) comes
//!   first and B3's row of the same distribution becomes a check; without a notice, B3.
//! - **Dates**: the record date ("data-com") of the official sources and the ex date of Yahoo
//!   are one session apart, which can be five calendar days around Carnival: dates are compared
//!   in business days of the B3 calendar ([`crate::finance::calendar`]).
//!
//! - **Mesmo provento**: mesmo ativo, data-com até [`MAX_DAYS_APART`] dias (o Yahoo usa a data
//!   ex, um pregão depois da data-com). O Yahoo não distingue JCP de dividendo e às vezes soma
//!   os dois do mesmo dia: o valor dele é comparado com cada evento da B3 e com a soma.
//! - **Referência**: a B3 (oficial) quando tem o provento; sem ela, a agenda (com Fundamentus e
//!   Yahoo divergindo, o Fundamentus, que copia os avisos das companhias).
//! - **Fontes da agenda com tipos diferentes**: o Yahoo chama de rendimento o que o
//!   Fundamentus chama de amortização (e usa a data ex, sem pagamento). De fontes diferentes,
//!   com data até [`MAX_BUSINESS_DAYS_APART`] dias úteis e valor até [`TOLERANCE`], é o mesmo
//!   provento, com o tipo e o pagamento do Fundamentus.
//! - **Conferência**: `confirmed` (outra fonte concorda até [`TOLERANCE`]), `divergent` (valor
//!   ou data diferentes: usa a referência e diz o que a outra fonte dizia) ou `single` (uma
//!   fonte só).

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::IncomeKind;
use super::records::text;
use crate::error::Result;
use crate::i18n::{self, t};
use crate::time::{fmt_date, parse_date};

/// Diferença de valor aceita entre as fontes (fração).
pub const TOLERANCE: f64 = 0.02;
/// Business days between the dates of the same distribution (record date x ex date, plus one
/// of slack).
pub const MAX_DAYS_APART: i64 = 2;
/// Entre fontes da agenda com tipos diferentes: dias úteis entre as datas.
pub const MAX_BUSINESS_DAYS_APART: i64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "PendingCheckStatus")]
pub enum CheckStatus {
    /// Outra fonte confirma valor e data.
    Confirmed,
    /// As fontes discordam (valor por ação ou data de pagamento).
    Divergent,
    /// Uma fonte só.
    Single,
}

/// O que uma fonte diz do provento.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "PendingSource")]
pub struct SourceValue {
    pub source: String,
    pub kind: IncomeKind,
    pub ex_date: String,
    pub pay_date: Option<String>,
    pub amount_per_share: String,
}

/// Conferência do provento sugerido entre as fontes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "PendingCheck")]
pub struct Check {
    pub status: CheckStatus,
    /// Fonte usada na sugestão.
    pub reference: String,
    pub sources: Vec<SourceValue>,
    /// O que diverge e o que foi usado, em uma frase.
    pub note: Option<String>,
}

/// Provento a sugerir (antes de saber quem tinha direito).
#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: String,
    pub asset_id: String,
    pub kind: IncomeKind,
    /// Data-com da referência (para a B3) ou data ex (Yahoo).
    pub ex_date: NaiveDate,
    pub pay_date: Option<NaiveDate>,
    pub per_share: Decimal,
    pub currency: String,
    pub source: String,
    pub check: Check,
    /// The reference is an official source (Fundos.NET or B3).
    pub official: bool,
    /// Fund income tax exempt for individuals, as the notice says (`None`: not informed).
    pub exempt: Option<bool>,
    /// Reference month, as the fund's notice writes it.
    pub period: Option<String>,
}

/// Official sources: their record date is the last day with the right.
pub fn is_official(source: &str) -> bool {
    source == crate::research::b3_listados::SOURCE || source == crate::research::fundos_net::SOURCE
}

#[derive(Debug, Clone)]
struct Row {
    id: String,
    asset_id: String,
    kind: IncomeKind,
    ex: NaiveDate,
    pay: Option<NaiveDate>,
    per: Decimal,
    currency: String,
    source: String,
    exempt: Option<bool>,
    period: Option<String>,
}

impl Row {
    fn value(&self) -> SourceValue {
        SourceValue {
            source: self.source.clone(),
            kind: self.kind,
            ex_date: fmt_date(self.ex),
            pay_date: self.pay.map(fmt_date),
            amount_per_share: self.per.normalize().to_string(),
        }
    }
}

fn kind_of(s: &str) -> IncomeKind {
    serde_json::from_value(serde_json::Value::String(s.to_string())).unwrap_or(IncomeKind::Other)
}

fn close(a: Decimal, b: Decimal) -> bool {
    if b.is_zero() {
        return a.is_zero();
    }
    let d = ((a - b) / b).abs();
    d <= Decimal::from_f64_retain(TOLERANCE).unwrap_or(Decimal::new(2, 2))
}

/// Same distribution by date: at most [`MAX_DAYS_APART`] business days apart.
pub fn near(a: NaiveDate, b: NaiveDate) -> bool {
    business_days_apart(a, b) <= MAX_DAYS_APART
}

/// Business days of the B3 calendar (weekends and national holidays off) between two dates.
pub fn business_days_apart(a: NaiveDate, b: NaiveDate) -> i64 {
    if (a - b).num_days().abs() > 60 {
        return (a - b).num_days().abs();
    }
    crate::finance::calendar::business_days_between(a, b).abs()
}

/// Kind families that describe the same payment: JCP, amortization, and the other cash
/// distributions (the agenda sometimes calls a fund's income a dividend).
pub fn kind_group(k: IncomeKind) -> u8 {
    match k {
        IncomeKind::Jcp => 1,
        IncomeKind::Amortization => 2,
        _ => 0,
    }
}

/// Mesmo provento na agenda: mesmo tipo e data perto, ou fontes diferentes (o tipo varia de
/// uma para a outra) com data até 3 dias úteis e valor igual (±2%).
fn same_payment(a: &Row, b: &Row) -> bool {
    (a.kind == b.kind && near(a.ex, b.ex))
        || (a.source != b.source
            && business_days_apart(a.ex, b.ex) <= MAX_BUSINESS_DAYS_APART
            && close(a.per, b.per))
}

fn label(source: &str) -> &str {
    match source {
        s if s == crate::research::b3_listados::SOURCE => "B3",
        s if s == crate::research::fundos_net::SOURCE => "Fundos.NET",
        "yahoo" => "Yahoo",
        "tiingo" => "Tiingo",
        "eodhd" => "EODHD",
        "fundamentus" => "Fundamentus",
        s => s,
    }
}

/// Valor por cota no idioma da instalação.
fn money(v: Decimal) -> String {
    super::records::fmt_dec(v)
}

fn br(d: NaiveDate) -> String {
    i18n::date(i18n::current(), d)
}

fn agenda(conn: &Connection, since: NaiveDate) -> Result<Vec<Row>> {
    let mut stmt = conn.prepare(
        "SELECT id, asset_id, kind, ex_date, pay_date, amount_per_share, currency, source FROM dividend_event
         WHERE estimated = 0 AND ex_date IS NOT NULL AND ex_date >= ?1",
    )?;
    let rows = stmt
        .query_map([fmt_date(since)], |r| {
            let ex: String = r.get(3)?;
            let pay: Option<String> = r.get(4)?;
            let per: String = r.get(5)?;
            let kind: String = r.get(2)?;
            Ok((|| {
                Some(Row {
                    id: r.get(0).ok()?,
                    asset_id: r.get(1).ok()?,
                    kind: kind_of(&kind),
                    ex: parse_date(&ex).ok()?,
                    pay: pay.and_then(|p| parse_date(&p).ok()),
                    per: per.parse().ok()?,
                    currency: r.get(6).ok()?,
                    source: r.get(7).ok()?,
                    exempt: None,
                    period: None,
                })
            })())
        })?
        .collect::<rusqlite::Result<Vec<Option<Row>>>>()?;
    Ok(rows.into_iter().flatten().collect())
}

fn official(conn: &Connection, since: NaiveDate) -> Result<Vec<Row>> {
    let src = crate::research::b3_listados::SOURCE;
    let mut stmt = conn.prepare(
        "SELECT a.id, a.currency, d.ticker, d.kind, d.ex_date, d.value, d.per_shares, d.pay_date, d.seq
           FROM research_dividend d JOIN asset a ON upper(a.ticker) = d.ticker AND a.archived = 0
          WHERE d.source = ?1 AND d.ex_date >= ?2",
    )?;
    let rows = stmt
        .query_map(params![src, fmt_date(since)], |r| {
            let ticker: String = r.get(2)?;
            let kind: String = r.get(3)?;
            let ex: String = r.get(4)?;
            let value: String = r.get(5)?;
            let per: i64 = r.get(6)?;
            let pay: Option<String> = r.get(7)?;
            let seq: i64 = r.get(8)?;
            Ok((|| {
                let v: Decimal = value.parse().ok()?;
                Some(Row {
                    id: format!("{src}:{ticker}:{ex}:{kind}:{seq}"),
                    asset_id: r.get(0).ok()?,
                    kind: kind_of(&kind),
                    ex: parse_date(&ex).ok()?,
                    pay: pay.and_then(|p| parse_date(&p).ok()),
                    per: (v / Decimal::from(per.max(1))).normalize(),
                    currency: r.get(1).ok()?,
                    source: src.to_string(),
                    exempt: None,
                    period: None,
                })
            })())
        })?
        .collect::<rusqlite::Result<Vec<Option<Row>>>>()?;
    Ok(rows.into_iter().flatten().collect())
}

/// Fundos.NET notices of the registered fund assets.
fn fund_notices(conn: &Connection, since: NaiveDate) -> Result<Vec<Row>> {
    let assets: Vec<(String, String, String)> = conn
        .prepare("SELECT id, upper(ticker), currency FROM asset WHERE class = 'fii' AND ticker IS NOT NULL")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let src = crate::research::fundos_net::SOURCE;
    let mut out = vec![];
    for (asset_id, ticker, currency) in assets {
        for n in crate::research::fundos_net::notices(conn, &ticker, since)? {
            out.push(Row {
                id: format!(
                    "{src}:{}:{}:{}:{}",
                    n.document_id,
                    n.ticker,
                    n.kind,
                    fmt_date(n.ex_date)
                ),
                asset_id: asset_id.clone(),
                kind: kind_of(&n.kind),
                ex: n.ex_date,
                pay: n.pay_date,
                per: n.value,
                currency: currency.clone(),
                source: src.to_string(),
                exempt: n.exempt,
                period: n.period,
            });
        }
    }
    Ok(out)
}

/// Candidatos já pagos (ou com a data-com passada, sem data de pagamento) até hoje.
pub fn candidates(conn: &Connection, since: NaiveDate, today: NaiveDate) -> Result<Vec<Candidate>> {
    let mut out = all_candidates(conn, since)?;
    out.retain(|c| c.pay_date.unwrap_or(c.ex_date) <= today);
    Ok(out)
}

/// Every distribution since `since`, paid or announced, one per payment, with the reference
/// and the check between the sources.
pub fn all_candidates(conn: &Connection, since: NaiveDate) -> Result<Vec<Candidate>> {
    #[derive(Default)]
    struct PerAsset {
        agenda: Vec<Row>,
        funds: Vec<Row>,
        b3: Vec<Row>,
    }
    let mut by_asset: BTreeMap<String, PerAsset> = BTreeMap::new();
    for r in agenda(conn, since)? {
        by_asset
            .entry(r.asset_id.clone())
            .or_default()
            .agenda
            .push(r);
    }
    for r in fund_notices(conn, since)? {
        by_asset
            .entry(r.asset_id.clone())
            .or_default()
            .funds
            .push(r);
    }
    for r in official(conn, since)? {
        by_asset.entry(r.asset_id.clone()).or_default().b3.push(r);
    }
    let mut out = vec![];
    for (_, a) in by_asset {
        // Official distributions: the fund's notice first; B3's row of the same payment checks it.
        let mut official: Vec<(Row, Vec<Row>)> = a.funds.into_iter().map(|r| (r, vec![])).collect();
        for b in a.b3 {
            match official
                .iter_mut()
                .find(|(o, _)| kind_group(o.kind) == kind_group(b.kind) && near(o.ex, b.ex))
            {
                Some((_, extra)) => extra.push(b),
                None => official.push((b, vec![])),
            }
        }
        let mut used = vec![false; a.agenda.len()];
        // 1) Cada provento oficial, conferido com a agenda do mesmo dia.
        for (o, extra) in &official {
            let same_day: Vec<&Row> = official
                .iter()
                .map(|(x, _)| x)
                .filter(|x| x.ex == o.ex)
                .collect();
            let sum: Decimal = same_day.iter().map(|x| x.per).sum();
            let mut others: Vec<&Row> = extra.iter().collect();
            for (i, r) in a.agenda.iter().enumerate() {
                // A JCP never pairs with an amortization; same-day dividend and JCP both see
                // Yahoo's sum.
                let compatible = kind_group(r.kind) == kind_group(o.kind)
                    || (kind_group(r.kind) == 0 && o.kind == IncomeKind::Jcp);
                if compatible && near(r.ex, o.ex) {
                    used[i] = true;
                    others.push(r);
                }
            }
            out.push(official_candidate(o, &others, sum, same_day.len()));
        }
        // 2) O que só a agenda traz, agrupado por provento.
        let rest: Vec<&Row> = a
            .agenda
            .iter()
            .zip(used)
            .filter(|(_, u)| !u)
            .map(|(r, _)| r)
            .collect();
        let mut groups: Vec<Vec<&Row>> = vec![];
        for r in rest {
            match groups
                .iter_mut()
                .find(|g| g.iter().any(|x| same_payment(x, r)))
            {
                Some(g) => g.push(r),
                None => groups.push(vec![r]),
            }
        }
        for g in groups {
            out.push(agenda_candidate(&g));
        }
    }
    out.sort_by(|a, b| {
        a.pay_date
            .unwrap_or(a.ex_date)
            .cmp(&b.pay_date.unwrap_or(b.ex_date))
            .then(a.asset_id.cmp(&b.asset_id))
    });
    Ok(out)
}

fn official_candidate(o: &Row, others: &[&Row], day_sum: Decimal, day_events: usize) -> Candidate {
    let mut sources = vec![o.value()];
    sources.extend(others.iter().map(|r| r.value()));
    let mut notes = vec![];
    let pay = o.pay.or_else(|| others.iter().find_map(|r| r.pay));
    if o.pay.is_none() && pay.is_some() {
        notes.push(t("nucleo-carteira-pendente-data-agenda"));
    }
    let status = if others.is_empty() {
        CheckStatus::Single
    } else {
        // O Yahoo não separa JCP de dividendo: vale o valor do evento ou a soma do dia.
        let value_ok = others
            .iter()
            .any(|r| close(r.per, o.per) || (day_events > 1 && close(r.per, day_sum)));
        let pay_diff = others.iter().find_map(|r| match (r.pay, o.pay) {
            (Some(a), Some(b)) if a != b => Some((r, a, b)),
            _ => None,
        });
        if !value_ok {
            let r = others[0];
            notes.push(text(
                "nucleo-carteira-pendente-valor-divergente",
                &[
                    ("fonte", label(&r.source).to_string()),
                    ("valor", money(r.per)),
                    ("b3", money(o.per)),
                ],
            ));
        }
        if let Some((r, theirs, ours)) = pay_diff {
            notes.push(text(
                "nucleo-carteira-pendente-data-divergente",
                &[
                    ("fonte", label(&r.source).to_string()),
                    ("data", br(theirs)),
                    ("b3", br(ours)),
                ],
            ));
        }
        if value_ok && pay_diff.is_none() {
            CheckStatus::Confirmed
        } else {
            CheckStatus::Divergent
        }
    };
    Candidate {
        id: o.id.clone(),
        asset_id: o.asset_id.clone(),
        kind: o.kind,
        ex_date: o.ex,
        pay_date: pay,
        per_share: o.per,
        currency: o.currency.clone(),
        source: o.source.clone(),
        check: Check {
            status,
            reference: o.source.clone(),
            sources,
            note: (!notes.is_empty()).then(|| notes.join("; ")),
        },
        official: true,
        exempt: o.exempt,
        period: o.period.clone(),
    }
}

fn agenda_candidate(g: &[&Row]) -> Candidate {
    // Referência: com divergência de valor ou de tipo, o Fundamentus (avisos das companhias,
    // com o tipo e o pagamento); senão o primeiro.
    let all_agree = g.iter().all(|r| close(r.per, g[0].per));
    let kinds_agree = g.iter().all(|r| r.kind == g[0].kind);
    let reference = if all_agree && kinds_agree {
        g[0]
    } else {
        g.iter()
            .find(|r| r.source == "fundamentus")
            .copied()
            .unwrap_or(g[0])
    };
    let pay = reference.pay.or_else(|| g.iter().find_map(|r| r.pay));
    let pay_agree = g.iter().filter_map(|r| r.pay).all(|p| Some(p) == pay);
    let status = if g.len() == 1 {
        CheckStatus::Single
    } else if all_agree && pay_agree {
        CheckStatus::Confirmed
    } else {
        CheckStatus::Divergent
    };
    let kind_note = (!kinds_agree).then(|| {
        text(
            "nucleo-carteira-pendente-tipos-divergentes",
            &[
                (
                    "lista",
                    g.iter()
                        .map(|r| format!("{} {}", label(&r.source), r.kind.label().to_lowercase()))
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                ("fonte", label(&reference.source).to_string()),
            ],
        )
    });
    let note = match status {
        CheckStatus::Divergent => Some(text(
            "nucleo-carteira-pendente-fontes-discordam",
            &[
                (
                    "lista",
                    g.iter()
                        .map(|r| format!("{} {}", label(&r.source), money(r.per)))
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                ("fonte", label(&reference.source).to_string()),
            ],
        )),
        CheckStatus::Single => Some(text(
            "nucleo-carteira-pendente-so-uma-fonte",
            &[("fonte", label(&reference.source).to_string())],
        )),
        CheckStatus::Confirmed => None,
    };
    let note = match (note, kind_note) {
        (Some(a), Some(b)) => Some(format!("{a}; {b}")),
        (a, b) => a.or(b),
    };
    Candidate {
        id: reference.id.clone(),
        asset_id: reference.asset_id.clone(),
        kind: reference.kind,
        ex_date: reference.ex,
        pay_date: pay,
        per_share: reference.per,
        currency: reference.currency.clone(),
        source: reference.source.clone(),
        check: Check {
            status,
            reference: reference.source.clone(),
            sources: g.iter().map(|r| r.value()).collect(),
            note,
        },
        official: false,
        exempt: None,
        period: None,
    }
}
