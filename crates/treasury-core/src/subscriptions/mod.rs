//! Assinaturas e cobranças recorrentes (docs/subscriptions.md): acha, nos lançamentos dos
//! últimos 2 anos, as despesas que se repetem no mesmo estabelecimento com valor parecido
//! a cada mês ou a cada ano (streaming, academia, software, seguro, anuidade), mesmo sem
//! recorrência cadastrada. Para cada uma: custo por mês e por ano, próxima cobrança
//! esperada, aumento de preço (última × anterior), cobrança nova (começou há pouco) e
//! cobrança atrasada (a esperada não veio: cancelada?). Sugere a recorrência a cadastrar.
//!
//! Compras parceladas, parcelas de contratos (empréstimo, financiamento, pré-pagamento, ou
//! lançadas na categoria ligada a um contrato), parcelas do IPTU, transferências e pagamentos
//! de fatura ficam de fora, assim como as
//! cópias entre contas (`transactions::duplicates`) e as descrições que só dizem o meio de
//! pagamento ("Débito em conta", "Pix enviado"). A mesma série achada em duas contas vira uma
//! só. O usuário pode dispensar uma detecção ("não é assinatura"); a chave fica no ajuste
//! `subscriptions.dismissed`.
//!
//! Cada série é uma **assinatura** (serviço: streaming, software, academia) ou uma **conta
//! fixa** (escola, condomínio, boleto: gasto essencial da casa); os totais da assinatura somam
//! só as assinaturas, e as contas fixas têm o total delas.

pub mod api;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashSet};

use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::ids::Id;
use crate::recurrence::{Freq, RecurrenceInput, RuleSpec};
use crate::time::{add_months, fmt_date, parse_date};
use crate::transactions::{TxnKind, normalize_description};

pub const DISMISSED_SETTING: &str = "subscriptions.dismissed";

/// Janela analisada.
const LOOKBACK_DAYS: i64 = 760;
/// Uma cobrança "nova" começou há no máximo isso.
const NEW_DAYS: i64 = 70;
/// Variação de preço que conta como aumento (ou queda): 1% e R$ 1,00.
const CHANGE_MIN_PCT: f64 = 0.01;
const CHANGE_MIN_ABS: i64 = 100;
/// Descrições (já normalizadas) que só dizem o meio de pagamento, sem o estabelecimento:
/// juntariam cobranças diferentes numa série falsa.
const GENERIC: &[&str] = &[
    "debito em conta",
    "debito automatico",
    "debito",
    "compra no debito",
    "compra debito",
    "compra no credito",
    "compra",
    "pagamento",
    "pagamento de boleto",
    "pagamento boleto",
    "boleto",
    "pix",
    "pix enviado",
    "transferencia",
    "transferencia enviada",
    "ted",
    "doc",
    "saque",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SubscriptionCadence {
    Monthly,
    Yearly,
}

/// Assinatura de serviço ou conta fixa da casa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SubscriptionKind {
    /// Serviço contratado (streaming, software, academia, clube).
    Subscription,
    /// Conta fixa da casa (escola, condomínio, boleto; categoria essencial).
    FixedBill,
}

/// Palavras (na chave normalizada) de conta fixa, mesmo sem categoria.
const FIXED_WORDS: &[&str] = &[
    "escola",
    "colegio",
    "faculdade",
    "universidade",
    "mensalidade",
    "condominio",
    "boleto",
    "aluguel",
    "iptu",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SubscriptionStatus {
    /// Cobrando em dia.
    Active,
    /// Começou nos últimos 70 dias.
    New,
    /// A cobrança esperada não veio (cancelada ou atrasada).
    Late,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SubscriptionCharge {
    pub txn_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Subscription {
    /// Estabelecimento normalizado + conta: identifica a assinatura.
    pub key: String,
    pub name: String,
    pub account_id: Id,
    pub account_name: String,
    pub currency: String,
    pub category_id: Option<Id>,
    pub category: Option<String>,
    pub cadence: SubscriptionCadence,
    pub status: SubscriptionStatus,
    /// Assinatura (serviço) ou conta fixa (escola, condomínio, boleto).
    pub kind: SubscriptionKind,
    /// Valor da última cobrança (positivo).
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(type = "number | null")]
    pub previous_amount: Option<i64>,
    /// Última ÷ anterior − 1, quando mudou (≥ 1% e R$ 1,00).
    pub change_pct: Option<f64>,
    #[ts(type = "number")]
    pub monthly_cost: i64,
    #[ts(type = "number")]
    pub annual_cost: i64,
    pub first_date: String,
    pub last_date: String,
    pub next_expected: String,
    /// Já existe recorrência cadastrada para ela.
    pub recurrence_id: Option<Id>,
    /// Recorrência pronta para cadastrar (quando não há).
    pub suggested_recurrence: Option<RecurrenceInput>,
    pub charges: Vec<SubscriptionCharge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SubscriptionReport {
    pub as_of: String,
    pub items: Vec<Subscription>,
    /// Soma por mês das assinaturas ativas e novas, na moeda base (outras moedas e as contas
    /// fixas ficam de fora).
    #[ts(type = "number")]
    pub monthly_total: i64,
    #[ts(type = "number")]
    pub annual_total: i64,
    /// O mesmo das contas fixas (escola, condomínio, boleto).
    #[ts(type = "number")]
    pub fixed_monthly_total: i64,
    #[ts(type = "number")]
    pub fixed_annual_total: i64,
    pub currency: String,
    pub increases: u32,
    pub new_count: u32,
    pub late_count: u32,
    pub dismissed: Vec<String>,
}

struct Row {
    id: Id,
    date: NaiveDate,
    amount: i64,
    description: String,
    account_id: Id,
    account_name: String,
    currency: String,
    category_id: Option<Id>,
    category: Option<String>,
    recurrence_id: Option<Id>,
    /// Grupo de gasto da categoria (`essential`, `debt`…), herdado da mãe quando falta.
    spend_group: Option<String>,
}

/// Chave do estabelecimento: descrição (ou favorecido) normalizada, sem números.
pub fn merchant_key(payee: Option<&str>, description: &str) -> String {
    let base = payee
        .filter(|p| !p.trim().is_empty())
        .unwrap_or(description);
    normalize_description(base)
        .split(' ')
        .filter(|w| !w.is_empty() && !w.chars().all(|c| c.is_ascii_digit()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn median(v: &mut [i64]) -> i64 {
    v.sort_unstable();
    v[v.len() / 2]
}

pub fn dismissed(conn: &Connection) -> Result<Vec<String>> {
    crate::settings::get_or(conn, DISMISSED_SETTING, Vec::new())
}

pub fn detect(conn: &Connection, today: NaiveDate) -> Result<SubscriptionReport> {
    let from = fmt_date(today - Duration::days(LOOKBACK_DAYS));
    let mut stmt = conn.prepare(
        "SELECT t.id, t.date, t.amount, t.description, t.payee, t.account_id, a.name, t.currency,
                t.category_id, c.name, t.recurrence_id, COALESCE(c.spend_group, p.spend_group)
         FROM txn t JOIN account a ON a.id = t.account_id
         LEFT JOIN category c ON c.id = t.category_id
         LEFT JOIN category p ON p.id = c.parent_id
         WHERE t.kind = 'expense' AND t.status <> 'planned' AND t.amount < 0
           AND t.date >= ?1 AND t.date <= ?2
           -- A closed account charges nothing any more: its series would only show as late.
           AND a.closed_on IS NULL
           AND t.installment_plan_id IS NULL AND t.transfer_id IS NULL
           -- Parcelas de contratos e do IPTU não são assinatura.
           AND NOT EXISTS (SELECT 1 FROM loan_installment li WHERE li.txn_id = t.id)
           AND NOT EXISTS (SELECT 1 FROM loan_extra_payment le WHERE le.txn_id = t.id)
           AND NOT EXISTS (SELECT 1 FROM property_tax_installment pt WHERE pt.txn_id = t.id)
           AND (t.category_id IS NULL OR t.category_id NOT IN
                (SELECT category_id FROM loan WHERE category_id IS NOT NULL))
         ORDER BY t.date, t.id",
    )?;
    // A cópia de um lançamento em outra conta não é uma segunda cobrança.
    let copies: HashSet<Id> = crate::transactions::duplicates::pairs(conn)?
        .into_iter()
        .map(|(_, copy, _)| copy)
        .collect();
    let mut groups: BTreeMap<String, Vec<Row>> = BTreeMap::new();
    let rows = stmt.query_map([&from, &fmt_date(today)], |r| {
        Ok((
            Row {
                id: r.get(0)?,
                date: NaiveDate::default(),
                amount: -r.get::<_, i64>(2)?,
                description: r.get(3)?,
                account_id: r.get(5)?,
                account_name: r.get(6)?,
                currency: r.get(7)?,
                category_id: r.get(8)?,
                category: r.get(9)?,
                recurrence_id: r.get(10)?,
                spend_group: r.get(11)?,
            },
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(4)?,
        ))
    })?;
    for row in rows {
        let (mut row, date, payee) = row?;
        if copies.contains(&row.id) {
            continue;
        }
        row.date = parse_date(&date)?;
        let key = merchant_key(payee.as_deref(), &row.description);
        if key.len() < 3 || GENERIC.contains(&key.as_str()) {
            continue;
        }
        groups
            .entry(format!("{key}|{}", row.account_id))
            .or_default()
            .push(row);
    }
    let gone: HashSet<String> = dismissed(conn)?.into_iter().collect();
    let recurrences = recurrence_keys(conn)?;
    let base = crate::household::base_currency(conn)?;
    let cards: HashSet<Id> = {
        let mut stmt = conn.prepare("SELECT id FROM account WHERE kind = 'credit_card'")?;
        stmt.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    // Cada série com as chaves de todas as contas em que apareceu: dispensar qualquer uma
    // dispensa a série.
    let mut series: Vec<(Subscription, Vec<String>)> = vec![];
    for (key, rows) in groups {
        let Some(s) = classify(&key, rows, today, &recurrences) else {
            continue;
        };
        match series.iter_mut().find(|(k, _)| same_series(k, &s)) {
            Some((kept, keys)) => {
                keys.push(key);
                if preference(&s, &cards) > preference(kept, &cards) {
                    *kept = s;
                }
            }
            None => series.push((s, vec![key])),
        }
    }
    let mut items: Vec<Subscription> = series
        .into_iter()
        .filter(|(_, keys)| keys.iter().all(|k| !gone.contains(k)))
        .map(|(s, _)| s)
        .collect();
    items.sort_by(|a, b| {
        b.monthly_cost
            .cmp(&a.monthly_cost)
            .then(a.name.cmp(&b.name))
    });
    let sum = |kind: SubscriptionKind, annual: bool| -> i64 {
        items
            .iter()
            .filter(|s| {
                s.kind == kind && s.status != SubscriptionStatus::Late && s.currency == base
            })
            .map(|s| {
                if annual {
                    s.annual_cost
                } else {
                    s.monthly_cost
                }
            })
            .sum()
    };
    let monthly_total = sum(SubscriptionKind::Subscription, false);
    let annual_total = sum(SubscriptionKind::Subscription, true);
    let fixed_monthly_total = sum(SubscriptionKind::FixedBill, false);
    let fixed_annual_total = sum(SubscriptionKind::FixedBill, true);
    Ok(SubscriptionReport {
        as_of: fmt_date(today),
        increases: items
            .iter()
            .filter(|s| s.change_pct.is_some_and(|c| c > 0.0))
            .count() as u32,
        new_count: items
            .iter()
            .filter(|s| s.status == SubscriptionStatus::New)
            .count() as u32,
        late_count: items
            .iter()
            .filter(|s| s.status == SubscriptionStatus::Late)
            .count() as u32,
        items,
        monthly_total,
        annual_total,
        fixed_monthly_total,
        fixed_annual_total,
        currency: base,
        dismissed: {
            let mut d: Vec<String> = gone.into_iter().collect();
            d.sort();
            d
        },
    })
}

/// A mesma série de cobranças achada duas vezes (o extrato de uma conta importado em outra,
/// sem o id do banco para casar as cópias): mesmo nome, ritmo e valor, e as duas últimas
/// cobranças nas mesmas datas.
fn same_series(a: &Subscription, b: &Subscription) -> bool {
    let before_last = |s: &Subscription| s.charges.iter().rev().nth(1).map(|c| c.date.clone());
    normalize_description(&a.name) == normalize_description(&b.name)
        && a.cadence == b.cadence
        && a.amount == b.amount
        && a.last_date == b.last_date
        && before_last(a) == before_last(b)
}

/// Qual das duas fica: a do cartão (a compra é dele), a que já tem recorrência, a de mais
/// cobranças.
fn preference(s: &Subscription, cards: &HashSet<Id>) -> (bool, bool, usize) {
    (
        cards.contains(&s.account_id),
        s.recurrence_id.is_some(),
        s.charges.len(),
    )
}

/// Recorrências de despesa ativas: chave (estabelecimento|conta) → id.
fn recurrence_keys(conn: &Connection) -> Result<BTreeMap<String, Id>> {
    let mut stmt = conn.prepare(
        "SELECT id, description, account_id FROM recurrence WHERE active = 1 AND kind = 'expense'",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, Id>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Id>(2)?,
        ))
    })?;
    let mut out = BTreeMap::new();
    for r in rows {
        let (id, desc, acc) = r?;
        out.insert(format!("{}|{acc}", merchant_key(None, &desc)), id);
    }
    Ok(out)
}

fn classify(
    key: &str,
    mut rows: Vec<Row>,
    today: NaiveDate,
    recurrences: &BTreeMap<String, Id>,
) -> Option<Subscription> {
    // Duas cobranças no mesmo dia (estorno e nova cobrança, ou duplicata): fica a última.
    rows.dedup_by(|b, a| a.date == b.date && a.amount == b.amount);
    if rows.len() < 2 {
        return None;
    }
    let gaps: Vec<i64> = rows
        .windows(2)
        .map(|w| (w[1].date - w[0].date).num_days())
        .collect();
    let mut sorted = gaps.clone();
    let gap = median(&mut sorted);
    let cadence = if (26..=35).contains(&gap) {
        SubscriptionCadence::Monthly
    } else if (350..=380).contains(&gap) {
        SubscriptionCadence::Yearly
    } else {
        return None;
    };
    let (lo, hi) = match cadence {
        SubscriptionCadence::Monthly => (20, 40),
        SubscriptionCadence::Yearly => (330, 400),
    };
    // Ritmo regular: no máximo um intervalo fora (um mês pulado vira 2 intervalos curtos).
    if gaps.iter().filter(|g| !(lo..=hi).contains(*g)).count() > 1 {
        return None;
    }
    let amounts: Vec<i64> = rows.iter().map(|r| r.amount).collect();
    let (min, max) = (*amounts.iter().min()?, *amounts.iter().max()?);
    if max as f64 > min as f64 * 1.5 {
        return None;
    }
    let first = rows.first()?;
    let last = rows.last()?;
    let new = (today - first.date).num_days() <= NEW_DAYS;
    // Mensal com duas cobranças só se for nova e de valor igual (senão é coincidência).
    if cadence == SubscriptionCadence::Monthly
        && rows.len() < 3
        && !(new && (max - min) * 20 <= max)
    {
        return None;
    }
    let next = match cadence {
        SubscriptionCadence::Monthly => add_months(last.date, 1),
        SubscriptionCadence::Yearly => add_months(last.date, 12),
    };
    let grace = match cadence {
        SubscriptionCadence::Monthly => 10,
        SubscriptionCadence::Yearly => 30,
    };
    let late = today > next + Duration::days(grace);
    // Atrasada há muito tempo: não é mais assinatura.
    if late && today > next + Duration::days(grace + 60) {
        return None;
    }
    let status = if late {
        SubscriptionStatus::Late
    } else if new {
        SubscriptionStatus::New
    } else {
        SubscriptionStatus::Active
    };
    let previous = rows.len().checked_sub(2).map(|i| rows[i].amount);
    let change_pct = previous.and_then(|p| {
        let d = last.amount - p;
        (p > 0 && d.abs() >= CHANGE_MIN_ABS && (d.abs() as f64 / p as f64) >= CHANGE_MIN_PCT)
            .then(|| last.amount as f64 / p as f64 - 1.0)
    });
    let (monthly_cost, annual_cost) = match cadence {
        SubscriptionCadence::Monthly => (last.amount, last.amount * 12),
        SubscriptionCadence::Yearly => ((last.amount as f64 / 12.0).round() as i64, last.amount),
    };
    let recurrence_id = rows
        .iter()
        .rev()
        .find_map(|r| r.recurrence_id.clone())
        .or_else(|| recurrences.get(key).cloned());
    let suggested_recurrence = recurrence_id.is_none().then(|| RecurrenceInput {
        description: last.description.clone(),
        account_id: last.account_id.clone(),
        amount: -last.amount,
        kind: TxnKind::Expense,
        category_id: last.category_id.clone(),
        member_id: None,
        to_account_id: None,
        spec: RuleSpec {
            freq: match cadence {
                SubscriptionCadence::Monthly => Freq::Monthly,
                SubscriptionCadence::Yearly => Freq::Yearly,
            },
            interval: None,
            by_month_day: Some(last.date.day()),
            by_day: None,
            by_month: (cadence == SubscriptionCadence::Yearly).then(|| last.date.month()),
            count: None,
        },
        start_date: fmt_date(next.max(today)),
        end_date: None,
    });
    Some(Subscription {
        key: key.to_string(),
        name: display_name(&last.description),
        account_id: last.account_id.clone(),
        account_name: last.account_name.clone(),
        currency: last.currency.clone(),
        category_id: last.category_id.clone(),
        category: last.category.clone(),
        cadence,
        status,
        kind: kind_of(key, &rows),
        amount: last.amount,
        previous_amount: previous,
        change_pct,
        monthly_cost,
        annual_cost,
        first_date: fmt_date(first.date),
        last_date: fmt_date(last.date),
        next_expected: fmt_date(next),
        recurrence_id,
        suggested_recurrence,
        charges: rows
            .iter()
            .map(|r| SubscriptionCharge {
                txn_id: r.id.clone(),
                date: fmt_date(r.date),
                amount: r.amount,
            })
            .collect(),
    })
}

/// Conta fixa: categoria essencial (ou de dívida) na maioria das cobranças, ou uma palavra de
/// conta da casa no estabelecimento (escola, condomínio, boleto…); senão, assinatura.
fn kind_of(key: &str, rows: &[Row]) -> SubscriptionKind {
    let essential = rows
        .iter()
        .filter(|r| matches!(r.spend_group.as_deref(), Some("essential" | "debt")))
        .count();
    let merchant = key.split('|').next().unwrap_or_default();
    let fixed_word = merchant.split(' ').any(|w| FIXED_WORDS.contains(&w));
    if fixed_word || essential * 2 > rows.len() {
        SubscriptionKind::FixedBill
    } else {
        SubscriptionKind::Subscription
    }
}

/// Nome para mostrar: a descrição sem os números soltos no fim (parcela, código).
fn display_name(desc: &str) -> String {
    let words: Vec<&str> = desc.split_whitespace().collect();
    let keep = words
        .iter()
        .rposition(|w| !w.chars().all(|c| c.is_ascii_digit() || "/-#*".contains(c)))
        .map_or(words.len(), |i| i + 1);
    let s = words[..keep].join(" ");
    if s.is_empty() {
        desc.trim().to_string()
    } else {
        s
    }
}
