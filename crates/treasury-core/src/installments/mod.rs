//! Parcelas detectadas pela descrição (docs/design/detected-installments.md).
//!
//! Lançamentos que dizem na descrição que são uma parcela ("Parcela 8/10", "PARC 03/12",
//! "PREST.EMPREST 003/005") viram uma **série**: mesma conta, mesma descrição sem a parte da
//! parcela, mesmo total de parcelas e valor parecido. Da última parcela vista saem as que
//! faltam, uma por mês, até a n/n. Nada é gravado: a série é calculada na hora; só o "não é
//! parcela" fica guardado (V163).
//!
//! `8/10` sozinho no fim da descrição pode ser uma data: só conta quando a série se confirma
//! (a mesma compra com a parcela seguinte num mês seguinte). Ficam de fora os parcelados do
//! Treasury, os lançamentos ligados a contratos de Dívidas (ou na categoria de um contrato) e
//! as séries ignoradas.

pub mod api;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use chrono::{Datelike, NaiveDate};
use regex::Regex;
use rusqlite::{Connection, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::ids::Id;
use crate::time::{add_months, fmt_date, month_of, now_utc, parse_date};

/// Parcela lida da descrição.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub number: u32,
    pub total: u32,
    /// Veio com palavra-chave ("parcela", "parc", "prest").
    pub keyword: bool,
    /// Prestação de empréstimo ("prest", "emprest", "financ").
    pub loan: bool,
    /// Parcelamento da fatura do cartão ("parc fatura", "parcelamento fat").
    pub bill: bool,
    /// Descrição sem a parte da parcela, para mostrar.
    pub base: String,
}

fn keyword_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)[\s\-–:]*\b(?:parcela|parc|prest(?:acao|ação)?|prest\.?\s*emprest|pmt)\b\.?\s*(\d{1,3})\s*(?:/|de)\s*(\d{1,3})\b",
        )
        .expect("regex da parcela com palavra")
    })
}

fn bare_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"[\s\-–:]+(\d{1,3})\s*/\s*(\d{1,3})\s*$").expect("regex da parcela solta")
    })
}

/// Lê a parcela da descrição, se houver.
pub fn parse(desc: &str) -> Option<Parsed> {
    let (caps, keyword) = match keyword_re().captures(desc) {
        Some(c) => (c, true),
        None => (bare_re().captures(desc)?, false),
    };
    let number: u32 = caps[1].parse().ok()?;
    let total: u32 = caps[2].parse().ok()?;
    let max = if keyword { 120 } else { 72 };
    if number == 0 || total < 2 || number > total || total > max {
        return None;
    }
    let whole = caps.get(0)?;
    let mut base = String::new();
    base.push_str(desc[..whole.start()].trim_end_matches([' ', '-', '–', ':', '*']));
    let rest = desc[whole.end()..].trim();
    if !rest.is_empty() {
        base.push(' ');
        base.push_str(rest);
    }
    let base = base.trim().to_string();
    let folded = crate::transactions::normalize_description(desc);
    let loan = ["emprest", "financ", "prest", "consig"]
        .iter()
        .any(|w| folded.contains(w));
    let bill = folded.contains("fatura")
        || folded.contains("parc fat")
        || folded.contains("parcelamento fat");
    Some(Parsed {
        number,
        total,
        keyword,
        loan: loan && !bill,
        bill,
        // Só a palavra ("PREST.EMPREST 003/005"): a base é a palavra, sem os números.
        base: if base.is_empty() {
            desc[whole.start()..caps.get(1)?.start()]
                .trim_matches([' ', '-', '–', ':', '*', '.'])
                .to_string()
        } else {
            base
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SeriesKind {
    /// Compra parcelada no cartão.
    Card,
    /// Prestação de empréstimo ou financiamento na conta.
    Loan,
    /// Parcelamento da fatura do cartão.
    Bill,
    /// Outra parcela na conta (carnê, crediário).
    Account,
}

/// Uma parcela projetada.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectedInstallment {
    pub number: u32,
    /// Data prevista do lançamento (no cartão, o fechamento da fatura).
    pub date: String,
    /// No cartão, o vencimento da fatura em que a parcela cai.
    pub due_date: Option<String>,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DetectedSeries {
    /// Chave estável (conta + descrição sem a parcela + total).
    pub key: String,
    pub account_id: Id,
    pub account_name: String,
    pub kind: SeriesKind,
    pub description: String,
    pub member_id: Option<Id>,
    pub member_name: Option<String>,
    /// Última parcela vista e o total.
    pub number: u32,
    pub total: u32,
    /// Valor da parcela (positivo).
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    pub last_date: String,
    /// Quanto falta pagar (parcelas restantes × valor).
    #[ts(type = "number")]
    pub remaining_amount: i64,
    pub remaining: u32,
    /// Mês da última parcela (n/n).
    pub end_month: String,
    /// A parcela seguinte já devia ter aparecido e não apareceu.
    pub missing: bool,
    /// Confirmada por palavra-chave ou por parcelas seguidas.
    pub confirmed_by_sequence: bool,
    pub projected: Vec<ProjectedInstallment>,
    pub txn_ids: Vec<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DetectedInstallments {
    pub currency: String,
    pub series: Vec<DetectedSeries>,
    /// Total que falta, na moeda base.
    #[ts(type = "number")]
    pub remaining_total: i64,
    /// O que sai por mês (moeda base), nos próximos 12 meses.
    pub by_month: Vec<crate::cards::MonthAmount>,
}

struct Row {
    id: Id,
    account_id: Id,
    account_name: String,
    card: bool,
    currency: String,
    date: NaiveDate,
    amount: i64,
    member_id: Option<Id>,
    member_name: Option<String>,
    statement_month: Option<String>,
    parsed: Parsed,
}

/// Chave da série: conta, descrição sem a parcela (normalizada) e total de parcelas.
pub fn series_key(account_id: &str, base: &str, total: u32) -> String {
    let norm = crate::transactions::normalize_description(base);
    format!("{account_id}|{norm}|{total}")
}

fn ignored(conn: &Connection) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare_cached("SELECT series_key FROM detected_installment_ignore")?;
    let rows = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Lançamentos com parcela na descrição nos últimos 13 meses, fora de parcelados do Treasury,
/// de contratos de Dívidas e de transferências.
fn candidates(conn: &Connection, today: NaiveDate) -> Result<Vec<Row>> {
    let from = fmt_date(add_months(today, -13));
    let mut stmt = conn.prepare(
        "SELECT t.id, t.account_id, a.name, a.kind, t.currency, t.date, t.amount, t.description,
                t.member_id, m.name, s.reference_month
           FROM txn t
           JOIN account a ON a.id = t.account_id
           LEFT JOIN member m ON m.id = t.member_id
           LEFT JOIN card_statement s ON s.id = t.statement_id
          WHERE t.date >= ?1 AND t.date <= ?2 AND t.amount < 0 AND t.status <> 'planned'
            AND t.installment_plan_id IS NULL AND t.transfer_id IS NULL
            AND t.kind IN ('expense', 'adjustment')
            AND NOT EXISTS (SELECT 1 FROM loan_installment li WHERE li.txn_id = t.id)
            AND NOT EXISTS (SELECT 1 FROM loan l WHERE l.category_id IS NOT NULL
                              AND l.category_id = t.category_id)
          ORDER BY t.date, t.id",
    )?;
    let mut q = stmt.query(params![from, fmt_date(today)])?;
    let mut out = vec![];
    while let Some(r) = q.next()? {
        let desc: String = r.get(7)?;
        let Some(parsed) = parse(&desc) else { continue };
        out.push(Row {
            id: r.get(0)?,
            account_id: r.get(1)?,
            account_name: r.get(2)?,
            card: r.get::<_, String>(3)? == "credit_card",
            currency: r.get(4)?,
            date: parse_date(&r.get::<_, String>(5)?)?,
            amount: -r.get::<_, i64>(6)?,
            member_id: r.get(8)?,
            member_name: r.get(9)?,
            statement_month: r.get(10)?,
            parsed,
        });
    }
    Ok(out)
}

/// Valor parecido: até `total` centavos (o resto da divisão) ou 1%.
fn close(a: i64, b: i64, total: u32) -> bool {
    let diff = (a - b).abs();
    diff <= i64::from(total) || diff * 100 <= a.max(b)
}

/// Mês da parcela seguinte (sequência confirmada: k e k+1 em meses seguidos).
fn consecutive(a: &Row, b: &Row) -> bool {
    if b.parsed.number != a.parsed.number + 1 {
        return false;
    }
    match (&a.statement_month, &b.statement_month) {
        (Some(x), Some(y)) => {
            let next = parse_date(&format!("{x}-01"))
                .map(|d| month_of(add_months(d, 1)))
                .unwrap_or_default();
            *y == next
        }
        _ => {
            let days = (b.date - a.date).num_days();
            (20..=45).contains(&days)
        }
    }
}

/// Séries em andamento, com as parcelas restantes projetadas.
pub fn detect(conn: &Connection, today: NaiveDate) -> Result<DetectedInstallments> {
    let base = crate::household::base_currency(conn)?;
    let skip = ignored(conn)?;
    let mut groups: BTreeMap<String, Vec<Row>> = BTreeMap::new();
    for r in candidates(conn, today)? {
        let key = series_key(&r.account_id, &r.parsed.base, r.parsed.total);
        // Ignorada antes de separar as compras (V163): vale para todas as da loja.
        if skip.contains(&key) {
            continue;
        }
        groups.entry(key).or_default().push(r);
    }
    let mut cards: HashMap<Id, Option<(u32, u32)>> = HashMap::new();
    let mut series = vec![];
    for (group_key, rows) in groups {
        let mut seen: HashMap<String, u32> = HashMap::new();
        for (origin, chain) in split_purchases(rows) {
            // Duas compras iguais no mesmo mês: a segunda ganha "#2" (chave única na tela).
            let n = seen.entry(origin.clone()).or_default();
            *n += 1;
            let key = match *n {
                1 => format!("{group_key}|{origin}"),
                n => format!("{group_key}|{origin}#{n}"),
            };
            if skip.contains(&key) {
                continue;
            }
            if let Some(s) = series_of(conn, today, key, &chain, &mut cards)? {
                series.push(s);
            }
        }
    }
    finish(conn, today, base, series)
}

/// Mês da 1ª parcela de uma linha: o da fatura (ou da data), menos as parcelas já vistas.
fn origin_month(r: &Row) -> Result<NaiveDate> {
    let month = match &r.statement_month {
        Some(m) => parse_date(&format!("{m}-01"))?,
        None => r.date.with_day(1).expect("dia 1"),
    };
    Ok(add_months(month, -(r.parsed.number as i32 - 1)))
}

/// Separa as compras de um grupo (mesma conta, loja e total de parcelas): duas compras em 10x
/// na mesma loja são duas séries. Linhas da mesma compra começam no mesmo mês (com um mês de
/// folga para datas na virada), têm valor parecido e não repetem o número da parcela.
fn split_purchases(mut rows: Vec<Row>) -> Vec<(String, Vec<Row>)> {
    let origins: Vec<Option<NaiveDate>> = rows.iter().map(|r| origin_month(r).ok()).collect();
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| {
        origins[a]
            .cmp(&origins[b])
            .then(rows[a].date.cmp(&rows[b].date))
            .then(rows[a].parsed.number.cmp(&rows[b].parsed.number))
    });
    // (mês de origem, índices das linhas)
    let mut chains: Vec<(NaiveDate, Vec<usize>)> = vec![];
    for i in order {
        let Some(origin) = origins[i] else { continue };
        let r = &rows[i];
        let fits = chains.iter_mut().find(|(o, idx)| {
            let months = (origin.year() - o.year()) * 12 + origin.month() as i32 - o.month() as i32;
            months.abs() <= 1
                && idx
                    .iter()
                    .all(|&j| rows[j].parsed.number != r.parsed.number)
                && idx
                    .iter()
                    .any(|&j| close(rows[j].amount, r.amount, r.parsed.total))
        });
        match fits {
            Some((_, idx)) => idx.push(i),
            None => chains.push((origin, vec![i])),
        }
    }
    let mut slots: Vec<Option<Row>> = rows.drain(..).map(Some).collect();
    chains
        .into_iter()
        .map(|(origin, idx)| {
            let mut chain: Vec<Row> = idx.into_iter().filter_map(|i| slots[i].take()).collect();
            chain.sort_by(|a, b| {
                a.date
                    .cmp(&b.date)
                    .then(a.parsed.number.cmp(&b.parsed.number))
            });
            (month_of(origin), chain)
        })
        .collect()
}

/// A série de uma compra, se ainda tem parcela a vir.
fn series_of(
    conn: &Connection,
    today: NaiveDate,
    key: String,
    chain: &[Row],
    cards: &mut HashMap<Id, Option<(u32, u32)>>,
) -> Result<Option<DetectedSeries>> {
    {
        let chain: Vec<&Row> = chain.iter().collect();
        let last = *chain.last().expect("compra com linha");
        let confirmed_by_sequence = chain.windows(2).any(|w| consecutive(w[0], w[1]));
        if !last.parsed.keyword && !confirmed_by_sequence {
            return Ok(None);
        }
        let (k, n) = (last.parsed.number, last.parsed.total);
        if k >= n {
            return Ok(None);
        }
        let kind = if last.card {
            if last.parsed.bill {
                SeriesKind::Bill
            } else {
                SeriesKind::Card
            }
        } else if last.parsed.loan {
            SeriesKind::Loan
        } else {
            SeriesKind::Account
        };
        // Projeção: no cartão, uma fatura por mês depois da da última parcela; na conta, o
        // mesmo dia do mês.
        let cycle = if last.card {
            *cards.entry(last.account_id.clone()).or_insert(
                crate::accounts::card_details(conn, &last.account_id)?
                    .map(|c| (c.closing_day, c.due_day)),
            )
        } else {
            None
        };
        let mut projected = vec![];
        for i in 1..=(n - k) {
            let (date, due) = match (cycle, &last.statement_month) {
                (Some((closing, due_day)), Some(month)) => {
                    let first = parse_date(&format!("{month}-01"))?;
                    let m = month_of(add_months(first, i as i32));
                    let d = crate::cards::dates_for_reference_month(&m, closing, due_day)?;
                    (d.closing_date, Some(d.due_date))
                }
                (Some((closing, due_day)), None) => {
                    let d = add_months(last.date, i as i32);
                    let s = crate::cards::statement_dates(d, closing, due_day);
                    (d, Some(s.due_date))
                }
                _ => (add_months(last.date, i as i32), None),
            };
            projected.push(ProjectedInstallment {
                number: k + i,
                date: fmt_date(date),
                due_date: due.map(fmt_date),
                amount: last.amount,
            });
        }
        // A parcela seguinte já devia ter vindo (15 dias de folga). Dois meses sem ela: a
        // série sai (quitada ou estornada).
        let expected = projected
            .first()
            .map(|p| parse_date(&p.date))
            .transpose()?
            .expect("há parcela restante");
        let late = (today - expected).num_days();
        if late > 45 {
            return Ok(None);
        }
        let missing = late > 15;
        let end_month = projected
            .last()
            .map(|p| p.due_date.as_deref().unwrap_or(&p.date)[..7].to_string())
            .unwrap_or_else(|| month_of(last.date));
        // No cartão vale o vencimento: a fatura fechada e ainda não importada continua a pagar.
        let today_s = fmt_date(today);
        projected
            .retain(|p| p.due_date.as_deref().unwrap_or(&p.date) > today_s.as_str() || missing);
        let remaining = n - k;
        Ok(Some(DetectedSeries {
            key,
            account_id: last.account_id.clone(),
            account_name: last.account_name.clone(),
            kind,
            description: last.parsed.base.clone(),
            member_id: last.member_id.clone(),
            member_name: last.member_name.clone(),
            number: k,
            total: n,
            amount: last.amount,
            currency: last.currency.clone(),
            last_date: fmt_date(last.date),
            remaining_amount: last.amount * i64::from(remaining),
            remaining,
            end_month,
            missing,
            confirmed_by_sequence,
            projected,
            txn_ids: chain.iter().map(|r| r.id.clone()).collect(),
        }))
    }
}

fn finish(
    conn: &Connection,
    today: NaiveDate,
    base: String,
    mut series: Vec<DetectedSeries>,
) -> Result<DetectedInstallments> {
    series.sort_by(|a, b| {
        b.remaining_amount
            .cmp(&a.remaining_amount)
            .then(a.description.cmp(&b.description))
    });
    // Totais na moeda base.
    let today_s = fmt_date(today);
    let first = today.with_day(1).expect("dia 1");
    let months: Vec<String> = (0..12).map(|i| month_of(add_months(first, i))).collect();
    let mut by_month: BTreeMap<String, i64> = months.iter().map(|m| (m.clone(), 0)).collect();
    let mut remaining_total = 0;
    for s in &series {
        let conv = |v: i64| -> Result<Option<i64>> {
            if s.currency == base {
                Ok(Some(v))
            } else {
                Ok(crate::fx::convert(conn, v, &s.currency, &base, &today_s)?.map(|c| c.amount))
            }
        };
        if let Some(v) = conv(s.remaining_amount)? {
            remaining_total += v;
        }
        for p in &s.projected {
            let when = p.due_date.as_deref().unwrap_or(&p.date);
            if let (Some(v), Some(slot)) = (conv(p.amount)?, by_month.get_mut(&when[..7])) {
                *slot += v;
            }
        }
    }
    Ok(DetectedInstallments {
        currency: base,
        series,
        remaining_total,
        by_month: by_month
            .into_iter()
            .map(|(month, amount)| crate::cards::MonthAmount { month, amount })
            .collect(),
    })
}

/// "Não é parcela": a série sai da lista e da previsão.
pub fn ignore(tx: &Transaction<'_>, s: &DetectedSeries) -> Result<()> {
    let now = now_utc();
    tx.execute(
        "INSERT INTO detected_installment_ignore (series_key, account_id, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4) ON CONFLICT (series_key) DO NOTHING",
        params![s.key, s.account_id, s.description, now],
    )?;
    Ok(())
}

pub fn unignore(tx: &Transaction<'_>, key: &str) -> Result<usize> {
    Ok(tx.execute(
        "DELETE FROM detected_installment_ignore WHERE series_key = ?1",
        [key],
    )?)
}

/// Série ignorada (para trazer de volta).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IgnoredSeries {
    pub key: String,
    pub account_id: Option<Id>,
    pub description: String,
}

pub fn list_ignored(conn: &Connection) -> Result<Vec<IgnoredSeries>> {
    let mut stmt = conn.prepare(
        "SELECT series_key, account_id, description FROM detected_installment_ignore ORDER BY description",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(IgnoredSeries {
                key: r.get(0)?,
                account_id: r.get(1)?,
                description: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}
