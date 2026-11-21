//! Proventos por papel: a agenda do app (`dividend_event`: Yahoo, Fundamentus), os oficiais
//! da B3 (`research_dividend`, [`super::b3_listados`]) e os lançados pela família
//! (`income_event`). Daqui saem a série dos indicadores (DY, último provento, Bazin) e a
//! conferência entre as fontes (`research.dividends_check` e o score de saúde).
//!
//! Regra: a agenda do ativo cadastrado continua valendo enquanto concorda com a B3; com
//! divergência material (valor acima de 2%, evento numa fonte e não na outra, ou soma de 12
//! meses diferente), a B3, fonte oficial, vira a referência. Sem agenda, a B3 vem antes dos
//! proventos lançados. Papéis só do universo usam a B3.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, NaiveDate, Weekday};
use rusqlite::{Connection, OptionalExtension};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::b3_listados::{self, SOURCE};
use super::data::SubjectRef;
use super::indicators::DividendPoint;
use super::to_f64;
use crate::error::{Error, Result};
use crate::time::{add_months, fmt_date, parse_date};

/// Diferença de valor aceita entre as fontes (fração).
pub const TOLERANCE: f64 = 0.02;
/// Distância máxima entre as datas do mesmo provento, em dias úteis (o Yahoo usa a data
/// ex, um pregão depois da data-com da B3).
pub const MAX_DAYS_APART: i64 = 3;
/// Janela da conferência.
pub const CHECK_MONTHS: i32 = 24;

/// Provento por ação/cota numa data.
#[derive(Debug, Clone, PartialEq)]
pub struct DivEvent {
    pub date: NaiveDate,
    /// dividend, jcp, fii_income, amortization, other.
    pub kind: String,
    pub amount: Decimal,
    /// yahoo, fundamentus, b3_listados…
    pub source: String,
}

/// Desdobramento, grupamento ou bonificação (data-com e ações novas por ação antiga).
#[derive(Debug, Clone, PartialEq)]
pub struct StockEvent {
    pub date: NaiveDate,
    pub kind: String,
    pub multiplier: Decimal,
}

/// Agenda do ativo cadastrado (sem as projeções do radar).
pub fn agenda(conn: &Connection, asset_id: &str) -> Result<Vec<DivEvent>> {
    let mut stmt = conn.prepare_cached(
        "SELECT coalesce(ex_date, pay_date), kind, amount_per_share, source FROM dividend_event
         WHERE asset_id = ?1 AND estimated = 0 AND coalesce(ex_date, pay_date) IS NOT NULL ORDER BY 1",
    )?;
    let rows: Vec<(String, String, String, String)> = stmt
        .query_map([asset_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(d, kind, a, source)| {
            Some(DivEvent {
                date: parse_date(&d).ok()?,
                kind,
                amount: a.parse().ok()?,
                source,
            })
        })
        .collect())
}

/// Proventos e eventos em ações oficiais (B3) de um papel.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Official {
    pub cash: Vec<DivEvent>,
    pub events: Vec<StockEvent>,
    /// Amortizações (devolução de capital) com a cota de antes: (data-com, fração do valor
    /// que fica na cota, (cotação − amortização) ÷ cotação na data-com).
    pub capital: Vec<(NaiveDate, Decimal)>,
}

/// Dias antes da data-com em que ainda vale a última cotação, para a amortização.
const CAPITAL_PRICE_DAYS: i64 = 10;
/// Amortizações que deixam menos que isso na cota mudam o DY o bastante para a fonte avisar.
const CAPITAL_NOTE_BELOW: Decimal = Decimal::from_parts(98, 0, 0, false, 2);

impl Official {
    pub fn load(conn: &Connection, ticker: &str) -> Result<Self> {
        let t = ticker.trim().to_uppercase();
        let mut stmt = conn.prepare_cached(
            "SELECT ex_date, kind, value, per_shares, pay_date FROM research_dividend
             WHERE ticker = ?1 AND source = ?2 ORDER BY ex_date, kind, seq",
        )?;
        type Row = (String, String, String, i64, Option<String>);
        let mut rows: Vec<Row> = stmt
            .query_map(rusqlite::params![t, SOURCE], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        // A B3 às vezes repete a linha do rendimento de um fundo (mesma data-com, valor e
        // pagamento): conta uma vez. Parcelas iguais de JCP de companhia continuam duas.
        rows.dedup_by(|a, b| a.1 == "fii_income" && a == b);
        let cash: Vec<DivEvent> = rows
            .into_iter()
            .filter_map(|(d, kind, v, per, _)| {
                let v: Decimal = v.parse().ok()?;
                Some(DivEvent {
                    date: parse_date(&d).ok()?,
                    kind,
                    amount: (v / Decimal::from(per.max(1))).normalize(),
                    source: SOURCE.into(),
                })
            })
            .collect();
        // Cada amortização com a cotação da data-com (ou de poucos dias antes).
        let mut amort: BTreeMap<NaiveDate, Decimal> = BTreeMap::new();
        for c in cash.iter().filter(|c| c.kind == "amortization") {
            *amort.entry(c.date).or_default() += c.amount;
        }
        let mut price = conn.prepare_cached(
            "SELECT close FROM research_quote WHERE ticker = ?1 AND date <= ?2 AND date >= ?3
             ORDER BY date DESC LIMIT 1",
        )?;
        let mut capital = vec![];
        for (date, a) in amort {
            let p: Option<String> = price
                .query_row(
                    rusqlite::params![
                        t,
                        fmt_date(date),
                        fmt_date(date - chrono::Duration::days(CAPITAL_PRICE_DAYS))
                    ],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(p) = p.and_then(|p| p.parse::<Decimal>().ok()).filter(|p| *p > a) {
                capital.push((date, ((p - a) / p).round_dp(8)));
            }
        }
        let mut stmt = conn.prepare_cached(
            "SELECT ex_date, kind, factor FROM research_stock_event WHERE ticker = ?1 AND source = ?2 ORDER BY ex_date",
        )?;
        let rows: Vec<(String, String, String)> = stmt
            .query_map(rusqlite::params![t, SOURCE], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        let events = rows
            .into_iter()
            .filter_map(|(d, kind, f)| {
                let m = b3_listados::multiplier(&kind, f.parse().ok()?)?;
                Some(StockEvent {
                    date: parse_date(&d).ok()?,
                    kind,
                    multiplier: m,
                })
            })
            .collect();
        Ok(Self {
            cash,
            events,
            capital,
        })
    }

    /// Ações de hoje por ação na data-com (desdobramentos, grupamentos e bonificações
    /// depois dela e já efetivos).
    pub fn factor_after(&self, date: NaiveDate, today: NaiveDate) -> Decimal {
        self.events
            .iter()
            .filter(|e| e.date >= date && e.date < today)
            .fold(Decimal::ONE, |acc, e| acc * e.multiplier)
    }

    /// Fração do valor da cota que ficou depois das amortizações a partir da data-com (já
    /// efetivas): o rendimento pago sobre a cota inteira, na escala da cota de hoje.
    pub fn capital_after(&self, date: NaiveDate, today: NaiveDate) -> Decimal {
        self.capital
            .iter()
            .filter(|(d, _)| *d >= date && *d < today)
            .fold(Decimal::ONE, |acc, (_, f)| acc * f)
    }

    /// Amortizações dos últimos 12 meses que tiraram mais de 2% do valor da cota.
    pub fn capital_returned(&self, today: NaiveDate) -> Vec<NaiveDate> {
        let since = today - chrono::Duration::days(365);
        self.capital
            .iter()
            .filter(|(d, f)| *d > since && *d < today && *f < CAPITAL_NOTE_BELOW)
            .map(|(d, _)| *d)
            .collect()
    }

    /// Série dos indicadores: proventos em dinheiro (sem amortizações), por ação de hoje e
    /// na escala do capital de hoje (depois das amortizações), somados por data-com.
    pub fn points(&self, today: NaiveDate) -> Vec<DividendPoint> {
        let mut by: BTreeMap<NaiveDate, Decimal> = BTreeMap::new();
        for c in self.cash.iter().filter(|c| c.kind != "amortization") {
            *by.entry(c.date).or_default() +=
                c.amount / self.factor_after(c.date, today) * self.capital_after(c.date, today);
        }
        by.into_iter()
            .map(|(date, amount)| DividendPoint {
                date,
                amount: amount.round_dp(8).normalize(),
            })
            .collect()
    }
}

/// A nota da conferência que explica a diferença pelos valores da época (para citá-la em
/// outro texto), no idioma da instalação.
pub fn is_epoch_note(n: &str) -> bool {
    let probe = crate::i18n::t_args(
        "nucleo-pesquisa-div-nota-valores-epoca",
        &crate::i18n::args(&[("agenda", "\u{1}".into()), ("b3", "\u{1}".into())]),
    );
    probe
        .split('\u{1}')
        .next()
        .is_some_and(|p| !p.is_empty() && n.starts_with(p))
}

/// Fonte dos proventos oficiais da B3, no idioma da instalação.
pub fn b3_label() -> String {
    crate::i18n::t("nucleo-pesquisa-div-fonte-b3")
}

/// A fonte da B3, dizendo quando os rendimentos foram ajustados por amortizações.
fn b3_source(off: &Official, today: NaiveDate, base: String) -> String {
    let dates = off.capital_returned(today);
    if dates.is_empty() {
        return base;
    }
    let list: Vec<String> = dates
        .iter()
        .map(|d| d.format("%d/%m/%Y").to_string())
        .collect();
    crate::i18n::t_args(
        "nucleo-pesquisa-div-fonte-b3-amortizacao",
        &crate::i18n::args(&[("fonte", base.into()), ("datas", list.join(", ").into())]),
    )
}

/// Agenda com cada provento uma vez: Yahoo e Fundamentus trazem o mesmo provento, e somar os
/// dois dobrava o DY. A fonte com mais eventos vale; das outras entram só as datas que ela
/// não tem (mesma natureza, até 3 dias úteis e soma do dia a 2%).
pub fn merge_agenda(ev: &[DivEvent]) -> Vec<DivEvent> {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for e in ev {
        *count.entry(e.source.as_str()).or_default() += 1;
    }
    let mut order: Vec<&str> = count.keys().copied().collect();
    order.sort_by_key(|s| std::cmp::Reverse(count[s]));
    let mut kept: Vec<DivEvent> = vec![];
    for src in order {
        let mine: Vec<DivEvent> = ev.iter().filter(|e| e.source == src).cloned().collect();
        let theirs = groups(&kept, None);
        let mut used = vec![false; theirs.len()];
        for g in groups(&mine, None) {
            let dup = theirs.iter().enumerate().position(|(i, o)| {
                !used[i]
                    && o.family == g.family
                    && bdays_apart(o.date, g.date) <= MAX_DAYS_APART
                    && rel(g.amount, o.amount) <= TOLERANCE
            });
            match dup {
                Some(i) => used[i] = true,
                None => kept.extend(
                    mine.iter()
                        .filter(|e| e.date == g.date && family(e) == g.family)
                        .cloned(),
                ),
            }
        }
    }
    kept.sort_by_key(|e| e.date);
    kept
}

fn agenda_points(ev: &[DivEvent]) -> (Vec<DividendPoint>, String) {
    let mut sources: Vec<String> = ev.iter().map(|e| e.source.clone()).collect();
    sources.sort();
    sources.dedup();
    (
        merge_agenda(ev)
            .iter()
            .filter(|e| family(e) != "amortização")
            .map(|e| DividendPoint {
                date: e.date,
                amount: e.amount,
            })
            .collect(),
        crate::i18n::t_args(
            "nucleo-pesquisa-div-fonte-agenda",
            &crate::i18n::args(&[("fontes", sources.join(", ").into())]),
        ),
    )
}

/// Proventos lançados (bruto ÷ quantidade na data-com).
fn income_points(conn: &Connection, asset_id: &str) -> Result<Vec<DividendPoint>> {
    let mut stmt = conn.prepare_cached(
        "SELECT coalesce(ex_date, pay_date), gross, quantity, currency FROM income_event
         WHERE asset_id = ?1 AND quantity IS NOT NULL AND kind IN ('dividend', 'jcp', 'fii_income') ORDER BY 1",
    )?;
    let rows: Vec<(String, i64, String, String)> = stmt
        .query_map([asset_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut by_date: BTreeMap<NaiveDate, Decimal> = BTreeMap::new();
    for (d, gross, q, cur) in rows {
        let (Ok(d), Ok(q)) = (parse_date(&d), q.parse::<Decimal>()) else {
            continue;
        };
        if q.is_zero() {
            continue;
        }
        *by_date.entry(d).or_default() += crate::portfolio::records::from_minor(gross, &cur) / q;
    }
    Ok(by_date
        .into_iter()
        .map(|(date, amount)| DividendPoint {
            date,
            amount: amount.round_dp(8),
        })
        .collect())
}

/// Proventos do papel para os indicadores, com a origem (ver a regra no topo do módulo).
/// `official_ticker`: ticker da B3 (ações e FIIs brasileiros), senão `None`.
pub fn select(
    conn: &Connection,
    asset_id: Option<&str>,
    official_ticker: Option<&str>,
    today: NaiveDate,
) -> Result<(Vec<DividendPoint>, Option<String>)> {
    let off = match official_ticker {
        Some(t) => Official::load(conn, t)?,
        None => Official::default(),
    };
    if let Some(id) = asset_id {
        let ag = agenda(conn, id)?;
        if !ag.is_empty() {
            if !off.cash.is_empty() {
                let c = check(official_ticker.unwrap_or_default(), &off, &ag, today);
                if c.material {
                    return Ok((
                        off.points(today),
                        Some(b3_source(
                            &off,
                            today,
                            crate::i18n::t_args(
                                "nucleo-pesquisa-div-fonte-b3-diverge",
                                &crate::i18n::args(&[
                                    ("b3", b3_label().into()),
                                    ("fontes", c.other_sources.join(", ").into()),
                                ]),
                            ),
                        )),
                    ));
                }
            }
            let (p, s) = agenda_points(&ag);
            return Ok((p, Some(s)));
        }
        if !off.cash.is_empty() {
            return Ok((off.points(today), Some(b3_source(&off, today, b3_label()))));
        }
        let p = income_points(conn, id)?;
        if !p.is_empty() {
            return Ok((
                p,
                Some(crate::i18n::t("nucleo-pesquisa-div-fonte-lancados")),
            ));
        }
    } else if !off.cash.is_empty() {
        return Ok((off.points(today), Some(b3_source(&off, today, b3_label()))));
    }
    // A B3 respondeu há pouco pelo fundo, sem nenhum provento: não houve rendimento (a
    // estimativa do informe mensal não entra no lugar). Fonte sem pontos.
    match official_ticker {
        Some(t) if fund_without_cash(conn, t, today)? => Ok((
            vec![],
            Some(crate::i18n::t(
                "nucleo-pesquisa-div-fonte-b3-sem-rendimentos",
            )),
        )),
        _ => Ok((vec![], None)),
    }
}

/// Dias em que a resposta da B3 a um fundo (que cobre uns 12 meses) vale como "sem
/// rendimentos".
pub const FUND_CHECK_DAYS: i64 = 45;

/// A consulta da B3 ao fundo deste código (`HGLG11` → `fii:HGLG`) deu certo há até
/// [`FUND_CHECK_DAYS`] dias. Só fundos (ações sem proventos na B3 seguem para a DFC).
fn fund_without_cash(conn: &Connection, ticker: &str, today: NaiveDate) -> Result<bool> {
    let code: String = ticker.trim().to_uppercase().chars().take(4).collect();
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT fetched_at, status FROM research_fetch_log WHERE source = ?1 AND key = ?2",
            rusqlite::params![SOURCE, format!("fii:{code}")],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.is_some_and(|(at, status)| {
        status == "ok"
            && chrono::DateTime::parse_from_rfc3339(&at)
                .is_ok_and(|a| (today - a.date_naive()).num_days() <= FUND_CHECK_DAYS)
    }))
}

// ---------------------------------------------------------------------------------------
// Conferência entre as fontes
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DividendCheckStatus {
    /// Mesmo provento nas duas fontes, valor a 2%.
    Match,
    /// Mesmo provento, valor diferente.
    ValueDiff,
    /// Só na B3.
    OnlyB3,
    /// Só na outra fonte.
    OnlyOther,
}

/// Um provento (por data e natureza) comparado entre a B3 e uma fonte da agenda.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DividendCheckItem {
    pub status: DividendCheckStatus,
    /// `provento` (dividendo, JCP, rendimento) ou `amortização`.
    pub family: String,
    pub b3_date: Option<String>,
    pub other_date: Option<String>,
    /// Tipos na B3 (ex.: "dividend + jcp").
    pub b3_kinds: Option<String>,
    /// Soma por ação como a B3 informa na data-com.
    pub b3_amount: Option<String>,
    /// O mesmo por ação de hoje (depois de desdobramentos e bonificações), quando muda.
    pub b3_adjusted: Option<String>,
    pub other_amount: Option<String>,
    /// yahoo, fundamentus…
    pub other_source: Option<String>,
    /// Diferença relativa ao valor da B3.
    #[ts(type = "number | null")]
    pub diff: Option<f64>,
    pub note: Option<String>,
}

/// Conferência dos proventos de um papel: B3 (oficial) × agenda do app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DividendCheck {
    pub ticker: String,
    pub asset_id: Option<String>,
    /// Janela conferida (24 meses até hoje).
    pub from: String,
    pub to: String,
    /// A B3 tem proventos deste papel.
    pub b3_available: bool,
    /// Última consulta à B3 (UTC) e a situação (ok, not_found, error).
    pub b3_fetched_at: Option<String>,
    pub b3_status: Option<String>,
    /// Fontes da agenda (yahoo, fundamentus…).
    pub other_sources: Vec<String>,
    #[ts(type = "number")]
    pub b3_events: u32,
    #[ts(type = "number")]
    pub other_events: u32,
    #[ts(type = "number")]
    pub matched: u32,
    #[ts(type = "number")]
    pub value_diffs: u32,
    #[ts(type = "number")]
    pub only_b3: u32,
    #[ts(type = "number")]
    pub only_other: u32,
    /// Proventos por ação nos últimos 12 meses, sem amortizações (a agenda com cada provento
    /// uma vez).
    pub b3_12m: Option<String>,
    pub other_12m: Option<String>,
    #[ts(type = "number | null")]
    pub diff_12m: Option<f64>,
    /// Divergência que faz a B3 virar a referência dos indicadores.
    pub material: bool,
    /// Origem que os indicadores usam: `b3`, `agenda` ou `none`.
    pub reference: String,
    pub items: Vec<DividendCheckItem>,
    pub notes: Vec<String>,
}

/// Dias úteis (seg–sex) entre duas datas.
fn bdays_apart(a: NaiveDate, b: NaiveDate) -> i64 {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let mut n = 0;
    let mut d = lo;
    while d < hi {
        d += chrono::Duration::days(1);
        if !matches!(d.weekday(), Weekday::Sat | Weekday::Sun) {
            n += 1;
        }
        if n > MAX_DAYS_APART + 1 {
            break;
        }
    }
    n
}

/// Natureza para o pareamento: amortização (devolução de capital) ou provento.
fn family(e: &DivEvent) -> &'static str {
    match (e.kind.as_str(), e.source.as_str()) {
        ("amortization", _) | ("other", "fundamentus") => "amortização",
        _ => "provento",
    }
}

struct Group {
    date: NaiveDate,
    family: &'static str,
    amount: Decimal,
    /// B3: por ação de hoje.
    adjusted: Decimal,
    kinds: BTreeSet<String>,
}

fn groups(ev: &[DivEvent], off: Option<(&Official, NaiveDate)>) -> Vec<Group> {
    let mut by: BTreeMap<(NaiveDate, &'static str), Group> = BTreeMap::new();
    for e in ev {
        let f = family(e);
        let g = by.entry((e.date, f)).or_insert_with(|| Group {
            date: e.date,
            family: f,
            amount: Decimal::ZERO,
            adjusted: Decimal::ZERO,
            kinds: BTreeSet::new(),
        });
        g.amount += e.amount;
        g.adjusted += match off {
            Some((o, today)) => e.amount / o.factor_after(e.date, today),
            None => e.amount,
        };
        g.kinds.insert(e.kind.clone());
    }
    by.into_values().collect()
}

fn rel(a: Decimal, b: Decimal) -> f64 {
    if b.is_zero() {
        return if a.is_zero() { 0.0 } else { 1.0 };
    }
    (to_f64(a) - to_f64(b)).abs() / to_f64(b).abs()
}

fn s(d: Decimal) -> String {
    d.round_dp(8).normalize().to_string()
}

/// Conferência (função pura): pareia por data-com (até 3 dias úteis) e natureza os proventos
/// da B3 com os de cada fonte da agenda nos últimos 24 meses, aponta valores a mais de 2% e
/// proventos numa fonte e não na outra (só no período que cada fonte cobre) e compara a soma
/// de 12 meses que o DY usa. Um valor que só bate depois do desdobramento (ou só antes) é
/// tido como igual, com nota: o Yahoo corrige os valores passados, a B3 e o Fundamentus não.
pub fn check(ticker: &str, off: &Official, other: &[DivEvent], today: NaiveDate) -> DividendCheck {
    let from = add_months(today, -CHECK_MONTHS);
    let in_window = |d: NaiveDate| d > from && d <= today;
    let b3_start = off.cash.iter().map(|e| e.date).min();
    let b3: Vec<DivEvent> = off
        .cash
        .iter()
        .filter(|e| in_window(e.date))
        .cloned()
        .collect();
    let b3_groups = groups(&b3, Some((off, today)));
    let mut sources: Vec<String> = other.iter().map(|e| e.source.clone()).collect();
    sources.sort();
    sources.dedup();
    let mut items: Vec<DividendCheckItem> = vec![];
    for src in &sources {
        let all: Vec<DivEvent> = other.iter().filter(|e| &e.source == src).cloned().collect();
        let start = all.iter().map(|e| e.date).min();
        let mine: Vec<DivEvent> = all.into_iter().filter(|e| in_window(e.date)).collect();
        let og = groups(&mine, None);
        let mut used = vec![false; og.len()];
        for g in &b3_groups {
            let best = og
                .iter()
                .enumerate()
                .filter(|(i, o)| {
                    !used[*i]
                        && o.family == g.family
                        && bdays_apart(o.date, g.date) <= MAX_DAYS_APART
                })
                .min_by_key(|(_, o)| {
                    (
                        bdays_apart(o.date, g.date),
                        (o.date - g.date).num_days().abs(),
                    )
                });
            let b3_adjusted = (g.adjusted != g.amount).then(|| s(g.adjusted));
            let mut item = DividendCheckItem {
                status: DividendCheckStatus::OnlyB3,
                family: g.family.into(),
                b3_date: Some(fmt_date(g.date)),
                other_date: None,
                b3_kinds: Some(g.kinds.iter().cloned().collect::<Vec<_>>().join(" + ")),
                b3_amount: Some(s(g.amount)),
                b3_adjusted,
                other_amount: None,
                other_source: Some(src.clone()),
                diff: None,
                note: None,
            };
            match best {
                Some((i, o)) => {
                    used[i] = true;
                    let (raw, adj) = (rel(o.amount, g.amount), rel(o.amount, g.adjusted));
                    let diff = raw.min(adj);
                    item.other_date = Some(fmt_date(o.date));
                    item.other_amount = Some(s(o.amount));
                    item.diff = Some(diff);
                    item.status = if diff <= TOLERANCE {
                        DividendCheckStatus::Match
                    } else {
                        DividendCheckStatus::ValueDiff
                    };
                    if g.adjusted != g.amount {
                        let fonte = || crate::i18n::args(&[("fonte", src.clone().into())]);
                        item.note = Some(if adj <= TOLERANCE && raw > TOLERANCE {
                            crate::i18n::t_args("nucleo-pesquisa-div-nota-ja-corrige", &fonte())
                        } else if raw <= TOLERANCE && adj > TOLERANCE {
                            crate::i18n::t_args("nucleo-pesquisa-div-nota-nao-corrige", &fonte())
                        } else {
                            crate::i18n::t("nucleo-pesquisa-div-nota-houve-evento")
                        });
                    }
                }
                None if start.is_some_and(|st| g.date >= st - chrono::Duration::days(5)) => {}
                // Antes do começo desta fonte: não conta como faltando.
                None => continue,
            }
            items.push(item);
        }
        for (i, o) in og.iter().enumerate() {
            if used[i] || b3_start.is_none_or(|st| o.date < st - chrono::Duration::days(5)) {
                continue;
            }
            items.push(DividendCheckItem {
                status: DividendCheckStatus::OnlyOther,
                family: o.family.into(),
                b3_date: None,
                other_date: Some(fmt_date(o.date)),
                b3_kinds: None,
                b3_amount: None,
                b3_adjusted: None,
                other_amount: Some(s(o.amount)),
                other_source: Some(src.clone()),
                diff: Some(1.0),
                note: Some(crate::i18n::t_args(
                    "nucleo-pesquisa-div-nota-sem-par",
                    &crate::i18n::args(&[(
                        "tipos",
                        o.kinds
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" + ")
                            .into(),
                    )]),
                )),
            });
        }
    }
    items.sort_by(|a, b| {
        let da = a.b3_date.as_ref().or(a.other_date.as_ref());
        let db = b.b3_date.as_ref().or(b.other_date.as_ref());
        db.cmp(&da)
            .then_with(|| a.other_source.cmp(&b.other_source))
    });
    let count = |st: DividendCheckStatus| items.iter().filter(|i| i.status == st).count() as u32;
    // Soma de 12 meses: a da agenda como o app soma (cada provento uma vez, sem amortizações)
    // × a da B3 sem amortizações, por ação de hoje.
    let since = today - chrono::Duration::days(365);
    let b3_12m: Decimal = off
        .points(today)
        .iter()
        .filter(|p| p.date > since && p.date <= today)
        .map(|p| p.amount)
        .sum();
    let other_12m: Decimal = merge_agenda(other)
        .iter()
        .filter(|e| family(e) != "amortização" && e.date > since && e.date <= today)
        .map(|e| e.amount)
        .sum();
    let both = !off.cash.is_empty() && !other.is_empty();
    let diff_12m = both.then(|| rel(other_12m, b3_12m));
    let (vd, ob, oo) = (
        count(DividendCheckStatus::ValueDiff),
        count(DividendCheckStatus::OnlyB3),
        count(DividendCheckStatus::OnlyOther),
    );
    let material = both && (vd + ob + oo > 0 || diff_12m.is_some_and(|d| d > TOLERANCE));
    let mut notes = vec![];
    if off.cash.is_empty() {
        notes.push(crate::i18n::t("nucleo-pesquisa-div-nota-sem-b3"));
    }
    if other.is_empty() {
        notes.push(crate::i18n::t("nucleo-pesquisa-div-nota-sem-agenda"));
    }
    if let Some(st) = b3_start
        && st > from
        && both
    {
        notes.push(crate::i18n::t_args(
            "nucleo-pesquisa-div-nota-b3-desde",
            &crate::i18n::args(&[("data", fmt_date(st).into())]),
        ));
    }
    // Soma de 12 meses com os valores da época (sem a correção por desdobramentos e
    // bonificações): explica a diferença quando a agenda não corrige (o Yahoo ignora as
    // bonificações).
    let b3_raw_12m: Decimal = off
        .cash
        .iter()
        .filter(|e| e.kind != "amortization" && e.date > since && e.date <= today)
        .map(|e| e.amount)
        .sum();
    if diff_12m.is_some_and(|d| d > TOLERANCE)
        && b3_raw_12m != b3_12m
        && rel(other_12m, b3_raw_12m) <= TOLERANCE
    {
        notes.push(crate::i18n::t_args(
            "nucleo-pesquisa-div-nota-valores-epoca",
            &crate::i18n::args(&[
                ("agenda", s(other_12m).into()),
                ("b3", s(b3_raw_12m).into()),
            ]),
        ));
    }
    if material {
        notes.push(crate::i18n::t("nucleo-pesquisa-div-nota-divergencia"));
    }
    let reference = if off.cash.is_empty() && other.is_empty() {
        "none"
    } else if material || other.is_empty() {
        "b3"
    } else {
        "agenda"
    };
    DividendCheck {
        ticker: ticker.to_string(),
        asset_id: None,
        from: fmt_date(from),
        to: fmt_date(today),
        b3_available: !off.cash.is_empty(),
        b3_fetched_at: None,
        b3_status: None,
        other_sources: sources,
        b3_events: b3.len() as u32,
        other_events: other.iter().filter(|e| in_window(e.date)).count() as u32,
        matched: count(DividendCheckStatus::Match),
        value_diffs: vd,
        only_b3: ob,
        only_other: oo,
        b3_12m: (!off.cash.is_empty()).then(|| s(b3_12m)),
        other_12m: (!other.is_empty()).then(|| s(other_12m)),
        diff_12m,
        material,
        reference: reference.into(),
        items,
        notes,
    }
}

/// Conferência de um ativo cadastrado ou papel do universo (`research.dividends_check`).
pub fn check_subject(conn: &Connection, r: &SubjectRef, today: NaiveDate) -> Result<DividendCheck> {
    let asset = match (&r.asset_id, &r.ticker) {
        (Some(id), _) if !id.trim().is_empty() => Some(crate::assets::get(conn, id)?),
        (_, Some(t)) if !t.trim().is_empty() => crate::assets::find_by_ticker(conn, t)?,
        _ => {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-pesquisa-erro-asset-ou-ticker",
            )));
        }
    };
    let ticker = asset
        .as_ref()
        .and_then(|a| a.ticker.clone())
        .or_else(|| r.ticker.clone())
        .map(|t| t.trim().to_uppercase())
        .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-pesquisa-erro-sem-codigo")))?;
    let br = asset.as_ref().is_none_or(|a| {
        matches!(
            a.class,
            crate::assets::AssetClass::StockBr | crate::assets::AssetClass::Fii
        )
    });
    if !br {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-conferencia-br",
        )));
    }
    let off = Official::load(conn, &ticker)?;
    let other = match &asset {
        Some(a) => agenda(conn, &a.id)?,
        None => vec![],
    };
    let mut c = check(&ticker, &off, &other, today);
    c.asset_id = asset.map(|a| a.id);
    if let Some(code) = b3_listados::code_of(&ticker) {
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT fetched_at, status FROM research_fetch_log WHERE source = ?1 AND key IN (?2, ?3)
                 ORDER BY fetched_at DESC LIMIT 1",
                rusqlite::params![SOURCE, format!("cia:{code}"), format!("fii:{code}")],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        if let Some((at, st)) = row {
            c.b3_fetched_at = Some(at);
            c.b3_status = Some(st);
        }
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }
    fn ev(date: &str, kind: &str, amount: Decimal, source: &str) -> DivEvent {
        DivEvent {
            date: d(date),
            kind: kind.into(),
            amount,
            source: source.into(),
        }
    }
    fn b3(list: &[(&str, &str, Decimal)]) -> Official {
        Official {
            cash: list
                .iter()
                .map(|(dt, k, a)| ev(dt, k, *a, SOURCE))
                .collect(),
            events: vec![],
            capital: vec![],
        }
    }

    #[test]
    fn agreeing_sources_are_not_material() {
        let today = d("2026-10-16");
        // B3: dividendo + JCP na mesma data-com (sexta); Yahoo soma os dois na data ex
        // (segunda); Fundamentus lista separado na data-com.
        let off = b3(&[
            ("2026-08-21", "dividend", dec!(0.47156696)),
            ("2026-08-21", "jcp", dec!(0.67407131)),
            ("2026-06-01", "jcp", dec!(0.35048636)),
            ("2026-06-01", "jcp", dec!(0.35048636)),
        ]);
        let other = vec![
            ev("2026-08-24", "dividend", dec!(1.14563827), "yahoo"),
            ev("2026-06-02", "dividend", dec!(0.70097272), "yahoo"),
        ];
        let c = check("PETR4", &off, &other, today);
        assert_eq!(c.matched, 2, "{c:#?}");
        assert!(!c.material);
        assert_eq!(c.reference, "agenda");
        assert_eq!(c.diff_12m, Some(0.0));
        assert_eq!(c.items[0].b3_kinds.as_deref(), Some("dividend + jcp"));
    }

    #[test]
    fn flags_value_differences_and_one_sided_events() {
        let today = d("2026-10-16");
        let off = b3(&[
            ("2026-09-30", "fii_income", dec!(1.17)),
            ("2026-08-31", "fii_income", dec!(1.17)),
            ("2026-07-31", "fii_income", dec!(1.17)),
            ("2026-06-30", "fii_income", dec!(1.10)),
            ("2026-06-30", "amortization", dec!(2)),
        ]);
        let other = vec![
            ev("2026-09-30", "fii_income", dec!(1.17), "fundamentus"),
            ev("2026-08-31", "fii_income", dec!(1.25), "fundamentus"), // valor errado
            // julho faltando
            ev("2026-06-30", "fii_income", dec!(1.10), "fundamentus"),
            ev("2026-06-30", "other", dec!(2), "fundamentus"), // amortização
            ev("2026-07-15", "fii_income", dec!(0.5), "fundamentus"), // não existe na B3
            // Antes do começo da B3: não conta.
            ev("2025-01-31", "fii_income", dec!(1.0), "fundamentus"),
        ];
        let off = Official {
            cash: off.cash,
            events: vec![],
            capital: vec![],
        };
        let mut off2 = off.clone();
        off2.cash
            .push(ev("2026-05-29", "fii_income", dec!(1.1), SOURCE));
        let c = check("HGLG11", &off2, &other, today);
        assert_eq!(c.matched, 3, "{c:#?}");
        assert_eq!(c.value_diffs, 1);
        assert_eq!(c.only_b3, 2, "31/07 e 29/05");
        assert_eq!(c.only_other, 1);
        assert!(c.material);
        assert_eq!(c.reference, "b3");
        let vd = c
            .items
            .iter()
            .find(|i| i.status == DividendCheckStatus::ValueDiff)
            .unwrap();
        assert_eq!(vd.b3_date.as_deref(), Some("2026-08-31"));
        assert!((vd.diff.unwrap() - 0.08 / 1.17).abs() < 1e-9);
        // 12 meses: a B3 e a agenda sem amortização.
        assert_eq!(c.b3_12m.as_deref(), Some("5.71"));
    }

    /// Yahoo e Fundamentus com o mesmo provento contam uma vez (o Yahoo soma o dia; a data
    /// ex é um pregão depois); a data só de uma fonte entra; amortização fica fora do DY.
    #[test]
    fn agenda_counts_each_distribution_once() {
        let other = vec![
            ev("2026-09-30", "fii_income", dec!(0.82), "fundamentus"),
            ev("2026-08-31", "fii_income", dec!(0.82), "fundamentus"),
            ev("2025-12-30", "fii_income", dec!(0.82), "fundamentus"),
            ev("2025-12-30", "fii_income", dec!(2.03), "fundamentus"),
            ev("2026-07-15", "other", dec!(5), "fundamentus"), // amortização
            ev("2026-10-01", "dividend", dec!(0.82), "yahoo"),
            ev("2026-09-01", "dividend", dec!(0.82), "yahoo"),
            ev("2025-12-31", "dividend", dec!(2.85), "yahoo"),
            ev("2026-07-31", "dividend", dec!(0.82), "yahoo"), // só no Yahoo
        ];
        let merged = merge_agenda(&other);
        assert_eq!(merged.len(), 6, "{merged:#?}");
        let (pts, _) = agenda_points(&other);
        let sum: Decimal = pts.iter().map(|p| p.amount).sum();
        assert_eq!(sum, dec!(0.82) * dec!(4) + dec!(2.03));
        let off = b3(&[
            ("2026-09-30", "fii_income", dec!(0.82)),
            ("2026-08-31", "fii_income", dec!(0.82)),
            ("2026-07-31", "fii_income", dec!(0.82)),
            ("2025-12-30", "fii_income", dec!(0.82)),
            ("2025-12-30", "fii_income", dec!(2.03)),
        ]);
        let c = check("XPLG11", &off, &other, d("2026-10-16"));
        assert_eq!(c.diff_12m, Some(0.0), "{c:#?}");
    }

    /// Rendimento pago antes de uma amortização fica na escala do capital que sobrou na cota.
    #[test]
    fn distributions_scaled_by_capital_returned() {
        let today = d("2026-10-27");
        let mut off = b3(&[
            ("2025-12-30", "fii_income", dec!(15.30)),
            ("2026-04-07", "amortization", dec!(76.50)),
            ("2026-09-30", "fii_income", dec!(0.10)),
        ]);
        // Cota a 80 na data-com da amortização: sobra (80 − 76,50) ÷ 80.
        off.capital.push((d("2026-04-07"), dec!(0.04375)));
        let pts = off.points(today);
        assert_eq!(pts[0].amount, dec!(15.30) * dec!(0.04375));
        assert_eq!(pts[1].amount, dec!(0.10));
        assert_eq!(off.capital_returned(today), vec![d("2026-04-07")]);
        assert!(b3_source(&off, today, "B3".into()).contains("07/04/2026"));
    }

    #[test]
    fn split_explains_per_share_difference() {
        let today = d("2026-10-16");
        let mut off = b3(&[("2026-03-10", "dividend", dec!(1.0))]);
        off.events.push(StockEvent {
            date: d("2026-05-04"),
            kind: "split".into(),
            multiplier: dec!(2),
        });
        // Yahoo corrige pelo desdobramento; o Fundamentus mostra o valor da época.
        let other = vec![
            ev("2026-03-11", "dividend", dec!(0.5), "yahoo"),
            ev("2026-03-10", "dividend", dec!(1.0), "fundamentus"),
        ];
        let c = check("ABCD3", &off, &other, today);
        assert_eq!(c.matched, 2, "{c:#?}");
        let y = c
            .items
            .iter()
            .find(|i| i.other_source.as_deref() == Some("yahoo"))
            .unwrap();
        assert!(y.note.as_deref().unwrap().contains("já corrige"));
        assert_eq!(y.b3_adjusted.as_deref(), Some("0.5"));
        let f = c
            .items
            .iter()
            .find(|i| i.other_source.as_deref() == Some("fundamentus"))
            .unwrap();
        assert!(f.note.as_deref().unwrap().contains("não corrige"));
        // A série dos indicadores fica por ação de hoje; a soma da agenda (Yahoo + Fundamentus
        // do mesmo provento) é o dobro: divergência material.
        assert_eq!(off.points(today)[0].amount, dec!(0.5));
        assert!(c.material, "{c:#?}");
        assert_eq!(c.diff_12m, Some(2.0));
        assert_eq!(bdays_apart(d("2026-02-13"), d("2026-02-18")), 3, "carnaval");
        // Só o Yahoo (que corrige): a soma de 12 meses bate com a B3 corrigida.
        let only_yahoo: Vec<DivEvent> = other
            .iter()
            .filter(|e| e.source == "yahoo")
            .cloned()
            .collect();
        let c = check("ABCD3", &off, &only_yahoo, today);
        assert!(!c.material, "{c:#?}");
        // Só o Fundamentus (valores da época): a nota explica a diferença de 12 meses.
        let only_f: Vec<DivEvent> = other
            .iter()
            .filter(|e| e.source == "fundamentus")
            .cloned()
            .collect();
        let c = check("ABCD3", &off, &only_f, today);
        assert!(c.material);
        assert!(
            c.notes.iter().any(|n| n.contains("valores da época")),
            "{c:#?}"
        );
    }
}
