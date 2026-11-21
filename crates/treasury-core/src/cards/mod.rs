//! Cartões de crédito: faturas (fechamento e vencimento), parcelamentos e pagamento.
//!
//! Uma compra feita até o dia do fechamento entra na fatura que fecha naquele mês; depois
//! dele, na seguinte. O vencimento cai no mesmo mês do fechamento se `due_day > closing_day`,
//! senão no mês seguinte. `reference_month` é o mês do vencimento.

pub mod annual_fee;
pub mod api;
pub mod limits;
pub mod loyalty;
pub mod plastics;
pub mod revolving;
pub mod rewards;
pub mod value;

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::{AccountKind, CardDetails};
use crate::error::{Error, Result};
use crate::i18n::{self, args, t_args};
use crate::ids::{Id, new_id};
use crate::time::{add_months, clamp_day, fmt_date, month_of, now_utc, parse_date, parse_month};
use crate::transactions::util::{clean_opt, date_text, dedup_ids, placeholders, require_member};

/// "fatura {id}" (detalhe do erro "não encontrado").
fn statement_ref(id: &str) -> String {
    t_args(
        "nucleo-contas-ref-fatura",
        &args(&[("id", id.to_string().into())]),
    )
}

/// "parcelamento {id}" (detalhe do erro "não encontrado").
fn plan_ref(id: &str) -> String {
    t_args(
        "nucleo-contas-ref-parcelamento",
        &args(&[("id", id.to_string().into())]),
    )
}
use crate::transactions::{NewTxn, TransferInput, TxnKind, TxnStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum StatementStatus {
    Open,
    Closed,
    Paid,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementView {
    pub id: Id,
    pub account_id: Id,
    pub account_name: String,
    /// Mês do vencimento (YYYY-MM).
    pub reference_month: String,
    pub period_start: String,
    pub closing_date: String,
    pub due_date: String,
    pub status: StatementStatus,
    /// Valor da fatura (positivo = a pagar): compras − estornos.
    #[ts(type = "number")]
    pub total: i64,
    /// Parte do total que ainda é prevista (parcelas futuras, recorrências).
    #[ts(type = "number")]
    pub planned: i64,
    /// Pagamentos já feitos (positivo). Pagamentos agendados (previstos) não contam.
    #[ts(type = "number")]
    pub paid: i64,
    #[ts(type = "number")]
    pub remaining: i64,
    /// Total informado pelo banco, para conferência.
    #[ts(type = "number | null")]
    pub informed_total: Option<i64>,
    #[ts(type = "number")]
    pub txn_count: i64,
    pub currency: String,
    /// De onde vêm as datas: `calc` (pelo ciclo do cartão), `manual` ou `import` (impressas).
    pub dates_source: DatesSource,
    /// Pagamento mínimo, limite e saldo da fatura anterior que o banco imprimiu.
    #[ts(type = "number | null")]
    pub informed_minimum: Option<i64>,
    #[ts(type = "number | null")]
    pub informed_limit: Option<i64>,
    #[ts(type = "number | null")]
    pub informed_previous_balance: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DatesSource {
    Calc,
    Manual,
    Import,
}

impl DatesSource {
    fn parse(s: &str) -> Self {
        match s {
            "manual" => DatesSource::Manual,
            "import" => DatesSource::Import,
            _ => DatesSource::Calc,
        }
    }
}

/// O que a fatura do banco imprime além das compras.
#[derive(Debug, Clone, Default)]
pub struct Printed {
    pub due_date: Option<String>,
    pub minimum: Option<i64>,
    pub credit_limit: Option<i64>,
    pub previous_balance: Option<i64>,
}

/// Alteração de uma fatura: datas reais (fechamento adiado, vencimento mudado num mês) e o que
/// o banco informou. Datas editadas não são mais refeitas pelo ciclo do cartão.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementPatch {
    pub id: Id,
    #[ts(optional)]
    pub closing_date: Option<String>,
    #[ts(optional)]
    pub due_date: Option<String>,
    /// Volta a calcular as datas pelo ciclo do cartão.
    #[serde(default)]
    #[ts(optional)]
    pub reset_dates: Option<bool>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub informed_total: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub informed_minimum: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub informed_limit: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub informed_previous_balance: Option<Option<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementDetail {
    pub statement: StatementView,
    pub transactions: Vec<crate::transactions::TxnView>,
    /// Subtotais por plástico (titular, adicionais, virtuais); vazio se nenhuma compra da
    /// fatura tem plástico.
    pub by_plastic: Vec<plastics::PlasticSubtotal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayStatementArgs {
    pub statement_id: Id,
    pub from_account_id: Id,
    /// Padrão: o restante da fatura.
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    /// Padrão: vencimento da fatura.
    #[ts(optional)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InstallmentInput {
    pub account_id: Id,
    pub description: String,
    /// Valor total da compra (positivo).
    #[ts(type = "number")]
    pub total_amount: i64,
    pub installments: u32,
    pub purchase_date: String,
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InstallmentPlan {
    pub id: Id,
    pub account_id: Id,
    pub description: String,
    #[ts(type = "number")]
    pub total_amount: i64,
    pub currency: String,
    pub installments: u32,
    pub purchase_date: String,
    pub category_id: Option<Id>,
    pub member_id: Option<Id>,
    /// Parcelas já efetivadas (data ≤ hoje).
    pub paid_installments: u32,
    #[ts(type = "number")]
    pub remaining_amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthAmount {
    pub month: String,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayStatementResult {
    pub statement: StatementView,
    pub transfer: crate::transactions::TransferResult,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ListStatementsArgs {
    /// Padrão: todos os cartões.
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Mês de vencimento inicial (YYYY-MM), inclusive.
    #[ts(optional)]
    pub from_month: Option<String>,
    #[ts(optional)]
    pub to_month: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MoveToStatementArgs {
    pub txn_ids: Vec<Id>,
    /// Fatura de destino…
    #[ts(optional)]
    pub statement_id: Option<Id>,
    /// …ou o mês de vencimento dela (YYYY-MM; criada se ainda não existir). Exige `account_id`.
    #[ts(optional)]
    pub reference_month: Option<String>,
    #[ts(optional)]
    pub account_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FutureCommitmentsArgs {
    /// Padrão: todos os cartões.
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Meses a partir do atual (padrão 12, máximo 120).
    #[ts(optional)]
    pub months: Option<u32>,
}

/// Datas da fatura em que uma compra feita em `date` entra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementDates {
    pub reference_month: String,
    pub period_start: NaiveDate,
    pub closing_date: NaiveDate,
    pub due_date: NaiveDate,
}

pub fn statement_dates(date: NaiveDate, closing_day: u32, due_day: u32) -> StatementDates {
    let this_close = clamp_day(date.year(), date.month(), closing_day);
    let close_month = if date <= this_close {
        date
    } else {
        add_months(date.with_day(1).expect("dia 1"), 1)
    };
    let closing_date = clamp_day(close_month.year(), close_month.month(), closing_day);
    let due_month = if due_day > closing_day {
        closing_date
    } else {
        add_months(closing_date.with_day(1).expect("dia 1"), 1)
    };
    let due_date = clamp_day(due_month.year(), due_month.month(), due_day);
    let prev = add_months(closing_date.with_day(1).expect("dia 1"), -1);
    let prev_close = clamp_day(prev.year(), prev.month(), closing_day);
    StatementDates {
        reference_month: month_of(due_date),
        period_start: prev_close.succ_opt().expect("data válida"),
        closing_date,
        due_date,
    }
}

/// Fatura (criada se preciso) para um lançamento em `date` numa conta de cartão.
/// Devolve `None` se a conta não for cartão.
pub fn statement_for(conn: &Connection, account_id: &str, date: &str) -> Result<Option<Id>> {
    let Some(card) = crate::accounts::card_details(conn, account_id)? else {
        return Ok(None);
    };
    let d = statement_dates(parse_date(date)?, card.closing_day, card.due_day);
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM card_statement WHERE account_id = ?1 AND reference_month = ?2",
            params![account_id, d.reference_month],
            |r| r.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(Some(id));
    }
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO card_statement (id, account_id, reference_month, period_start, closing_date, due_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'open', ?7, ?7)",
        params![id, account_id, d.reference_month, fmt_date(d.period_start), fmt_date(d.closing_date), fmt_date(d.due_date), now],
    )?;
    Ok(Some(id))
}

/// Datas da fatura que vence no mês `reference_month` (YYYY-MM).
pub fn dates_for_reference_month(
    reference_month: &str,
    closing_day: u32,
    due_day: u32,
) -> Result<StatementDates> {
    let (y, m) = parse_month(reference_month)?;
    let due_date = clamp_day(y, m, due_day);
    let due_first = due_date.with_day(1).expect("dia 1");
    let close_month = if due_day > closing_day {
        due_first
    } else {
        add_months(due_first, -1)
    };
    let closing_date = clamp_day(close_month.year(), close_month.month(), closing_day);
    let prev = add_months(close_month, -1);
    let prev_close = clamp_day(prev.year(), prev.month(), closing_day);
    Ok(StatementDates {
        reference_month: reference_month.to_string(),
        period_start: prev_close.succ_opt().expect("data válida"),
        closing_date,
        due_date,
    })
}

pub(crate) fn require_card(conn: &Connection, account_id: &str) -> Result<CardDetails> {
    let (_, kind) = crate::accounts::currency_and_kind(conn, account_id)?;
    if kind != AccountKind::CreditCard {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-cartao-erro-nao-cartao",
        )));
    }
    crate::accounts::card_details(conn, account_id)?
        .ok_or_else(|| Error::invalid(i18n::t("nucleo-contas-cartao-erro-sem-ciclo")))
}

/// Fatura do mês de vencimento indicado, criada se ainda não existir.
pub fn ensure_statement(conn: &Connection, account_id: &str, reference_month: &str) -> Result<Id> {
    let card = require_card(conn, account_id)?;
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM card_statement WHERE account_id = ?1 AND reference_month = ?2",
            params![account_id, reference_month],
            |r| r.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    let d = dates_for_reference_month(reference_month, card.closing_day, card.due_day)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO card_statement (id, account_id, reference_month, period_start, closing_date, due_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'open', ?7, ?7)",
        params![id, account_id, d.reference_month, fmt_date(d.period_start), fmt_date(d.closing_date), fmt_date(d.due_date), now_utc()],
    )?;
    Ok(id)
}

/// Fatura que um pagamento feito em `date` quita: a mais antiga já fechada com saldo a
/// pagar; sem ela, a fatura em aberto na data.
pub fn statement_to_pay(conn: &Connection, account_id: &str, date: &str) -> Result<Option<Id>> {
    if crate::accounts::card_details(conn, account_id)?.is_none() {
        return Ok(None);
    }
    let pending: Option<String> = conn
        .query_row(
            "SELECT s.id FROM card_statement s LEFT JOIN txn t ON t.statement_id = s.id
              WHERE s.account_id = ?1 AND s.closing_date < ?2
              GROUP BY s.id
             HAVING COALESCE(SUM(CASE WHEN t.kind <> 'card_payment' THEN -t.amount
                                      WHEN t.status <> 'planned' THEN -t.amount END), 0) > 0
              ORDER BY s.reference_month LIMIT 1",
            params![account_id, date],
            |r| r.get(0),
        )
        .optional()?;
    match pending {
        Some(id) => Ok(Some(id)),
        None => statement_for(conn, account_id, date),
    }
}

/// Status calculado: `open` até o fechamento, `closed` depois; `paid`/`partial`
/// conforme os pagamentos efetivados.
pub fn compute_status(
    total: i64,
    paid: i64,
    closing_date: &str,
    today: NaiveDate,
) -> StatementStatus {
    let closed = fmt_date(today).as_str() > closing_date;
    let remaining = total - paid;
    if paid > 0 || (closed && total <= 0) {
        if remaining <= 0 {
            StatementStatus::Paid
        } else {
            StatementStatus::Partial
        }
    } else if closed {
        StatementStatus::Closed
    } else {
        StatementStatus::Open
    }
}

impl StatementStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            StatementStatus::Open => "open",
            StatementStatus::Closed => "closed",
            StatementStatus::Paid => "paid",
            StatementStatus::Partial => "partial",
        }
    }
}

const VIEW_SQL: &str = "SELECT s.id, s.account_id, a.name, s.reference_month, s.period_start, s.closing_date, s.due_date,
       s.informed_total, a.currency, s.dates_source, s.informed_minimum, s.informed_limit,
       s.informed_previous_balance,
       COALESCE(SUM(CASE WHEN t.kind <> 'card_payment' THEN -t.amount END), 0),
       COALESCE(SUM(CASE WHEN t.kind <> 'card_payment' AND t.status = 'planned' THEN -t.amount END), 0),
       COALESCE(SUM(CASE WHEN t.kind = 'card_payment' AND t.status <> 'planned' THEN t.amount END), 0),
       COUNT(CASE WHEN t.kind <> 'card_payment' THEN 1 END)
  FROM card_statement s
  JOIN account a ON a.id = s.account_id
  LEFT JOIN txn t ON t.statement_id = s.id";

fn row_to_view(r: &rusqlite::Row<'_>, today: NaiveDate) -> rusqlite::Result<StatementView> {
    let closing_date: String = r.get(5)?;
    let total: i64 = r.get(13)?;
    let paid: i64 = r.get(15)?;
    Ok(StatementView {
        id: r.get(0)?,
        account_id: r.get(1)?,
        account_name: r.get(2)?,
        reference_month: r.get(3)?,
        period_start: r.get(4)?,
        status: compute_status(total, paid, &closing_date, today),
        closing_date,
        due_date: r.get(6)?,
        informed_total: r.get(7)?,
        currency: r.get(8)?,
        dates_source: DatesSource::parse(&r.get::<_, String>(9)?),
        informed_minimum: r.get(10)?,
        informed_limit: r.get(11)?,
        informed_previous_balance: r.get(12)?,
        total,
        planned: r.get(14)?,
        paid,
        remaining: total - paid,
        txn_count: r.get(16)?,
    })
}

/// Faturas com totais calculados, em ordem de vencimento.
pub fn list_statements(
    conn: &Connection,
    args: &ListStatementsArgs,
    today: NaiveDate,
) -> Result<Vec<StatementView>> {
    for m in [&args.from_month, &args.to_month].into_iter().flatten() {
        parse_month(m)?;
    }
    let mut stmt = conn.prepare(&format!(
        "{VIEW_SQL}
         WHERE (?1 IS NULL OR s.account_id = ?1) AND (?2 IS NULL OR s.reference_month >= ?2)
           AND (?3 IS NULL OR s.reference_month <= ?3)
         GROUP BY s.id ORDER BY s.reference_month, a.sort_order, a.name"
    ))?;
    let rows = stmt
        .query_map(
            params![args.account_id, args.from_month, args.to_month],
            |r| row_to_view(r, today),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn statement_view(conn: &Connection, id: &str, today: NaiveDate) -> Result<StatementView> {
    conn.query_row(
        &format!("{VIEW_SQL} WHERE s.id = ?1 GROUP BY s.id"),
        [id],
        |r| row_to_view(r, today),
    )
    .optional()?
    .ok_or_else(|| Error::not_found(statement_ref(id)))
}

/// Fatura com os lançamentos (compras, estornos e pagamentos), em ordem de data.
pub fn get_statement(conn: &Connection, id: &str, today: NaiveDate) -> Result<StatementDetail> {
    let statement = statement_view(conn, id, today)?;
    let ids = crate::transactions::query_ids(
        conn,
        &crate::transactions::TxnFilter {
            statement_id: Some(id.to_string()),
            sort: Some(crate::transactions::TxnSort::DateAsc),
            ..Default::default()
        },
    )?;
    Ok(StatementDetail {
        statement,
        transactions: crate::transactions::views(conn, &ids)?,
        by_plastic: plastics::statement_subtotals(conn, id)?,
    })
}

/// Grava o status calculado na tabela (para consultas SQL de outros módulos). A visão
/// (`StatementView`) sempre recalcula na hora.
pub fn sync_statuses(conn: &Connection, ids: &[Id], today: NaiveDate) -> Result<()> {
    for id in ids {
        let Some(v) = statement_view(conn, id, today).ok() else {
            continue;
        };
        conn.execute(
            "UPDATE card_statement SET status = ?2, updated_at = ?3 WHERE id = ?1 AND status <> ?2",
            params![id, v.status.as_str(), now_utc()],
        )?;
    }
    Ok(())
}

/// Saldo devedor do cartão (positivo = devido): saldo inicial e lançamentos efetivados,
/// mais as parcelas futuras já comprometidas. Base do limite disponível.
pub fn outstanding(conn: &Connection, account_id: &str) -> Result<i64> {
    let v: i64 = conn.query_row(
        "SELECT a.opening_balance + COALESCE((SELECT SUM(t.amount) FROM txn t WHERE t.account_id = a.id
                  AND (t.status <> 'planned' OR t.installment_plan_id IS NOT NULL)), 0)
           FROM account a WHERE a.id = ?1",
        [account_id],
        |r| r.get(0),
    )?;
    Ok(-v)
}

/// Paga uma fatura: transferência `card_payment` da conta pagadora para o cartão, com a
/// perna do cartão ligada à fatura paga.
pub fn pay_statement(
    tx: &Transaction<'_>,
    args: &PayStatementArgs,
    today: NaiveDate,
) -> Result<crate::transactions::TransferIds> {
    let st = statement_view(tx, &args.statement_id, today)?;
    let card = require_card(tx, &st.account_id)?;
    let from = match clean_opt(Some(args.from_account_id.clone())) {
        Some(f) => f,
        None => card
            .payment_account_id
            .clone()
            .ok_or_else(|| Error::invalid(i18n::t("nucleo-contas-fatura-erro-pagadora")))?,
    };
    let (from_cur, from_kind) = crate::accounts::currency_and_kind(tx, &from)?;
    if from_kind == AccountKind::CreditCard {
        return Err(Error::invalid(i18n::t(
            "nucleo-lancamentos-erro-pagamento-de-cartao",
        )));
    }
    if from_cur != st.currency {
        return Err(Error::invalid(t_args(
            "nucleo-contas-fatura-erro-moeda",
            &i18n::args(&[
                ("moeda_conta", from_cur.clone().into()),
                ("moeda_cartao", st.currency.clone().into()),
            ]),
        )));
    }
    let amount = match args.amount {
        Some(a) if a > 0 => a,
        Some(_) => {
            return Err(Error::invalid(i18n::t(
                "nucleo-contas-fatura-erro-valor-positivo",
            )));
        }
        None if st.remaining > 0 => st.remaining,
        None => {
            return Err(Error::invalid(i18n::t(
                "nucleo-contas-fatura-erro-nada-a-pagar",
            )));
        }
    };
    let date = args.date.clone().unwrap_or_else(|| st.due_date.clone());
    let d = parse_date(&date)?;
    let (y, m) = parse_month(&st.reference_month)?;
    let input = TransferInput {
        from_account_id: from,
        to_account_id: st.account_id.clone(),
        date,
        amount,
        to_amount: None,
        // Descrição no idioma da instalação (fica gravada no lançamento).
        description: Some(t_args(
            "nucleo-contas-fatura-descricao-pagamento",
            &i18n::args(&[
                ("cartao", st.account_name.clone().into()),
                ("mes", format!("{m:02}/{y}").into()),
            ]),
        )),
        kind: Some(TxnKind::CardPayment),
        status: Some(if d > today {
            TxnStatus::Planned
        } else {
            TxnStatus::Cleared
        }),
        member_id: None,
    };
    crate::transactions::create_transfer_with(tx, &input, Some(st.id.clone()), today)
}

/// Grava o que a fatura impressa mostra: o vencimento (as datas passam a ser `import`, se
/// diferentes das calculadas), mínimo, limite (vira registro no histórico do limite) e saldo
/// anterior.
pub fn apply_printed(tx: &Transaction<'_>, id: &str, p: &Printed) -> Result<()> {
    let (account_id, due, source): (String, String, String) = tx.query_row(
        "SELECT account_id, due_date, dates_source FROM card_statement WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let now = now_utc();
    if let Some(d) = &p.due_date
        && *d != due
        && source != "manual"
    {
        parse_date(d)?;
        tx.execute(
            "UPDATE card_statement SET due_date = ?2, dates_source = 'import', updated_at = ?3 WHERE id = ?1",
            params![id, d, now],
        )?;
    }
    tx.execute(
        "UPDATE card_statement SET informed_minimum = COALESCE(?2, informed_minimum),
                informed_limit = COALESCE(?3, informed_limit),
                informed_previous_balance = COALESCE(?4, informed_previous_balance), updated_at = ?5
          WHERE id = ?1 AND (?2 IS NOT NULL OR ?3 IS NOT NULL OR ?4 IS NOT NULL)",
        params![id, p.minimum, p.credit_limit, p.previous_balance, now],
    )?;
    if let Some(limit) = p.credit_limit {
        let date = p.due_date.clone().unwrap_or(due);
        limits::record_import(tx, &account_id, &date, limit)?;
    }
    Ok(())
}

/// Altera datas e valores informados de uma fatura.
pub fn update_statement(
    tx: &Transaction<'_>,
    patch: &StatementPatch,
    today: NaiveDate,
) -> Result<()> {
    let cur = statement_view(tx, &patch.id, today)?;
    let now = now_utc();
    if patch.reset_dates == Some(true) {
        let card = require_card(tx, &cur.account_id)?;
        let d = dates_for_reference_month(&cur.reference_month, card.closing_day, card.due_day)?;
        tx.execute(
            "UPDATE card_statement SET period_start = ?2, closing_date = ?3, due_date = ?4,
                    dates_source = 'calc', updated_at = ?5 WHERE id = ?1",
            params![
                cur.id,
                fmt_date(d.period_start),
                fmt_date(d.closing_date),
                fmt_date(d.due_date),
                now
            ],
        )?;
    } else if patch.closing_date.is_some() || patch.due_date.is_some() {
        let closing = patch
            .closing_date
            .clone()
            .unwrap_or(cur.closing_date.clone());
        let due = patch.due_date.clone().unwrap_or(cur.due_date.clone());
        let (c, d) = (parse_date(&closing)?, parse_date(&due)?);
        if d < c {
            return Err(Error::invalid(i18n::t("nucleo-contas-fatura-erro-datas")));
        }
        if c < parse_date(&cur.period_start)? {
            return Err(Error::invalid(i18n::t(
                "nucleo-contas-fatura-erro-fechamento",
            )));
        }
        tx.execute(
            "UPDATE card_statement SET closing_date = ?2, due_date = ?3, dates_source = 'manual', updated_at = ?4
              WHERE id = ?1",
            params![cur.id, closing, due, now],
        )?;
        // A fatura seguinte começa no dia depois do fechamento novo (se ainda é calculada).
        if closing != cur.closing_date {
            let next_start = fmt_date(c.succ_opt().expect("data válida"));
            tx.execute(
                "UPDATE card_statement SET period_start = ?3, updated_at = ?4
                  WHERE id = (SELECT id FROM card_statement WHERE account_id = ?1 AND reference_month > ?2
                               ORDER BY reference_month LIMIT 1)",
                params![cur.account_id, cur.reference_month, next_start, now],
            )?;
        }
    }
    let money = |v: &Option<Option<i64>>| -> Result<()> {
        if v.is_some_and(|x| x.is_some_and(|x| x < 0)) {
            return Err(Error::invalid(i18n::t(
                "nucleo-contas-fatura-erro-valor-negativo",
            )));
        }
        Ok(())
    };
    for v in [
        &patch.informed_total,
        &patch.informed_minimum,
        &patch.informed_limit,
        &patch.informed_previous_balance,
    ] {
        money(v)?;
    }
    let cols = [
        ("informed_total", patch.informed_total),
        ("informed_minimum", patch.informed_minimum),
        ("informed_limit", patch.informed_limit),
        ("informed_previous_balance", patch.informed_previous_balance),
    ];
    for (col, v) in cols {
        if let Some(v) = v {
            tx.execute(
                &format!("UPDATE card_statement SET {col} = ?2, updated_at = ?3 WHERE id = ?1 AND {col} IS NOT ?2"),
                params![cur.id, v, now],
            )?;
        }
    }
    sync_statuses(tx, std::slice::from_ref(&cur.id), today)?;
    Ok(())
}

pub fn set_informed_total(
    tx: &Transaction<'_>,
    id: &str,
    informed_total: Option<i64>,
) -> Result<()> {
    let n = tx.execute(
        "UPDATE card_statement SET informed_total = ?2, updated_at = ?3
          WHERE id = ?1 AND informed_total IS NOT ?2",
        params![id, informed_total, now_utc()],
    )?;
    if n == 0 && !crate::transactions::util::exists(tx, "card_statement", id)? {
        return Err(Error::not_found(statement_ref(id)));
    }
    Ok(())
}

/// Move lançamentos do cartão para outra fatura (ex.: compra lançada pelo banco no mês
/// seguinte). Devolve quantos mudaram.
pub fn move_to_statement(
    tx: &Transaction<'_>,
    args: &MoveToStatementArgs,
    today: NaiveDate,
) -> Result<usize> {
    let target = match (&args.statement_id, &args.reference_month, &args.account_id) {
        (Some(id), _, _) => {
            if !crate::transactions::util::exists(tx, "card_statement", id)? {
                return Err(Error::not_found(statement_ref(id)));
            }
            id.clone()
        }
        (None, Some(month), Some(acc)) => ensure_statement(tx, acc, month)?,
        _ => {
            return Err(Error::invalid(i18n::t("nucleo-contas-fatura-erro-destino")));
        }
    };
    let account: String = tx.query_row(
        "SELECT account_id FROM card_statement WHERE id = ?1",
        [&target],
        |r| r.get(0),
    )?;
    let mut touched = vec![target.clone()];
    let mut n = 0;
    for id in dedup_ids(&args.txn_ids) {
        let row = crate::transactions::load_row(tx, &id)?;
        if row.account_id != account {
            return Err(Error::invalid(t_args(
                "nucleo-contas-fatura-erro-outro-cartao",
                &i18n::args(&[("descricao", row.description.clone().into())]),
            )));
        }
        if row.statement_id.as_deref() == Some(target.as_str()) {
            continue;
        }
        touched.extend(row.statement_id.iter().cloned());
        tx.execute(
            "UPDATE txn SET statement_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, target, now_utc()],
        )?;
        n += 1;
    }
    sync_statuses(tx, &touched, today)?;
    Ok(n)
}

/// Compra parcelada: N lançamentos no cartão, um por mês a partir da data da compra;
/// o centavo que sobra da divisão vai na 1ª parcela. Parcelas futuras ficam previstas.
pub fn create_installments(
    tx: &Transaction<'_>,
    input: &InstallmentInput,
    today: NaiveDate,
) -> Result<Id> {
    require_card(tx, &input.account_id)?;
    let (currency, _) = crate::accounts::currency_and_kind(tx, &input.account_id)?;
    let n = input.installments;
    if !(1..=120).contains(&n) {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-parcelas-erro-numero",
        )));
    }
    if input.total_amount <= 0 {
        return Err(Error::invalid(i18n::t("nucleo-contas-parcelas-erro-total")));
    }
    if input.total_amount < i64::from(n) {
        return Err(Error::invalid(i18n::t("nucleo-contas-parcelas-erro-menor")));
    }
    let purchase = parse_date(&input.purchase_date)?;
    let description = input.description.trim().to_string();
    if description.is_empty() {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-parcelas-erro-descricao",
        )));
    }
    let category_id = clean_opt(input.category_id.clone());
    crate::transactions::check_category(tx, category_id.as_deref(), TxnKind::Expense)?;
    let member_id = clean_opt(input.member_id.clone());
    if let Some(m) = &member_id {
        require_member(tx, m)?;
    }
    let plan_id = new_id();
    tx.execute(
        "INSERT INTO installment_plan (id, account_id, description, total_amount, currency, installments,
                                       purchase_date, category_id, member_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![plan_id, input.account_id, description, input.total_amount, currency, n,
                input.purchase_date, category_id, member_id, now_utc()],
    )?;
    let base = input.total_amount / i64::from(n);
    let rest = input.total_amount - base * i64::from(n);
    let mut touched = Vec::new();
    for i in 1..=n {
        let date = add_months(purchase, i as i32 - 1);
        let value = base + if i == 1 { rest } else { 0 };
        let mut t = NewTxn::new(
            &input.account_id,
            &fmt_date(date),
            -value,
            &currency,
            TxnKind::Expense,
            &format!("{description} {i}/{n}"),
        );
        t.status = if date > today {
            TxnStatus::Planned
        } else {
            TxnStatus::Cleared
        };
        t.category_id = category_id.clone();
        t.member_id = member_id.clone();
        t.installment_plan_id = Some(plan_id.clone());
        t.installment_number = Some(i64::from(i));
        let id = crate::transactions::insert(tx, &t)?;
        if let Some(tags) = &input.tags {
            crate::transactions::replace_tags(tx, &id, tags)?;
        }
        if let Some(s) = tx.query_row("SELECT statement_id FROM txn WHERE id = ?1", [&id], |r| {
            r.get::<_, Option<String>>(0)
        })? {
            touched.push(s);
        }
    }
    touched.dedup();
    sync_statuses(tx, &touched, today)?;
    Ok(plan_id)
}

pub fn list_installment_plans(
    conn: &Connection,
    account_id: Option<&str>,
    today: NaiveDate,
) -> Result<Vec<InstallmentPlan>> {
    let today = fmt_date(today);
    let mut stmt = conn.prepare(
        "SELECT p.id, p.account_id, p.description, p.total_amount, p.currency, p.installments, p.purchase_date,
                p.category_id, p.member_id,
                (SELECT count(*) FROM txn t WHERE t.installment_plan_id = p.id AND t.date <= ?2),
                (SELECT COALESCE(SUM(-t.amount), 0) FROM txn t WHERE t.installment_plan_id = p.id AND t.date > ?2)
           FROM installment_plan p
          WHERE (?1 IS NULL OR p.account_id = ?1)
          ORDER BY p.purchase_date DESC, p.id DESC",
    )?;
    let rows = stmt
        .query_map(params![account_id, today], |r| {
            Ok(InstallmentPlan {
                id: r.get(0)?,
                account_id: r.get(1)?,
                description: r.get(2)?,
                total_amount: r.get(3)?,
                currency: r.get(4)?,
                installments: r.get(5)?,
                purchase_date: r.get(6)?,
                category_id: r.get(7)?,
                member_id: r.get(8)?,
                paid_installments: r.get(9)?,
                remaining_amount: r.get(10)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_installment_plan(
    conn: &Connection,
    id: &str,
    today: NaiveDate,
) -> Result<InstallmentPlan> {
    list_installment_plans(conn, None, today)?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| Error::not_found(plan_ref(id)))
}

/// Apaga o parcelamento e todas as suas parcelas. Devolve quantas parcelas saíram.
pub fn delete_installment_plan(tx: &Transaction<'_>, id: &str, today: NaiveDate) -> Result<usize> {
    if !crate::transactions::util::exists(tx, "installment_plan", id)? {
        return Err(Error::not_found(plan_ref(id)));
    }
    let ids: Vec<String> = {
        let mut stmt = tx.prepare("SELECT id FROM txn WHERE installment_plan_id = ?1")?;
        stmt.query_map([id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let rows = ids
        .iter()
        .map(|i| crate::transactions::load_row(tx, i))
        .collect::<Result<Vec<_>>>()?;
    if let Some(r) = rows.iter().find(|r| r.status == TxnStatus::Reconciled) {
        return Err(Error::conflict(t_args(
            "nucleo-contas-parcelas-erro-conciliada",
            &args(&[
                ("descricao", r.description.clone().into()),
                ("data", date_text(&r.date).into()),
            ]),
        )));
    }
    let n = crate::transactions::delete_rows(tx, &rows, today)?;
    tx.execute("DELETE FROM installment_plan WHERE id = ?1", [id])?;
    Ok(n)
}

/// Quanto vence por mês nos próximos `months` meses (a partir do atual): o que falta
/// pagar de cada fatura, incluindo parcelas futuras. Valores na moeda base.
pub fn future_commitments(
    conn: &Connection,
    account_id: Option<&str>,
    months: u32,
    today: NaiveDate,
) -> Result<Vec<MonthAmount>> {
    let months = months.clamp(1, 120);
    let first = today.with_day(1).expect("dia 1");
    let keys: Vec<String> = (0..months)
        .map(|i| month_of(add_months(first, i as i32)))
        .collect();
    let mut out: BTreeMap<String, i64> = keys.iter().map(|k| (k.clone(), 0)).collect();
    let base = crate::household::base_currency(conn)?;
    let views = list_statements(
        conn,
        &ListStatementsArgs {
            account_id: account_id.map(str::to_string),
            from_month: keys.first().cloned(),
            to_month: keys.last().cloned(),
        },
        today,
    )?;
    let today_s = fmt_date(today);
    // Parcelas detectadas pela descrição, no mês do vencimento da fatura em que caem.
    for s in crate::installments::detect(conn, today)?
        .series
        .iter()
        .filter(|s| !s.missing && account_id.is_none_or(|a| a == s.account_id))
    {
        for p in &s.projected {
            let Some(due) = p.due_date.as_deref() else {
                continue;
            };
            let amount = if s.currency == base {
                Some(p.amount)
            } else {
                crate::fx::convert(conn, p.amount, &s.currency, &base, &today_s)?.map(|c| c.amount)
            };
            if let (Some(a), Some(slot)) = (amount, out.get_mut(&due[..7])) {
                *slot += a;
            }
        }
    }
    for v in views {
        if v.remaining <= 0 {
            continue;
        }
        let amount = if v.currency == base {
            Some(v.remaining)
        } else {
            crate::fx::convert(conn, v.remaining, &v.currency, &base, &today_s)?.map(|c| c.amount)
        };
        if let (Some(a), Some(slot)) = (amount, out.get_mut(&v.reference_month)) {
            *slot += a;
        }
    }
    Ok(out
        .into_iter()
        .map(|(month, amount)| MonthAmount { month, amount })
        .collect())
}

/// Parcelas previstas cuja data já chegou viram efetivadas. Devolve quantas mudaram.
pub fn clear_due_installments(conn: &Connection, today: NaiveDate) -> Result<usize> {
    Ok(conn.execute(
        "UPDATE txn SET status = 'cleared', updated_at = ?2
          WHERE installment_plan_id IS NOT NULL AND status = 'planned' AND date <= ?1",
        params![fmt_date(today), now_utc()],
    )?)
}

/// Depois de mudar o fechamento/vencimento do cartão: recalcula as datas das faturas
/// ainda não fechadas e reatribui os lançamentos delas. Faturas fechadas não mudam.
pub fn recompute_statements(
    tx: &Transaction<'_>,
    account_id: &str,
    today: NaiveDate,
) -> Result<()> {
    let card = require_card(tx, account_id)?;
    let today_s = fmt_date(today);
    let open: Vec<(String, String, String)> = {
        let mut stmt = tx.prepare(
            "SELECT id, reference_month, period_start FROM card_statement
              WHERE account_id = ?1 AND closing_date >= ?2 AND dates_source = 'calc'",
        )?;
        stmt.query_map(params![account_id, today_s], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let now = now_utc();
    let mut cutoff = fmt_date(statement_dates(today, card.closing_day, card.due_day).period_start);
    for (id, month, start) in &open {
        cutoff = cutoff.min(start.clone());
        let d = dates_for_reference_month(month, card.closing_day, card.due_day)?;
        tx.execute(
            "UPDATE card_statement SET period_start = ?2, closing_date = ?3, due_date = ?4, updated_at = ?5
              WHERE id = ?1 AND (period_start <> ?2 OR closing_date <> ?3 OR due_date <> ?4)",
            params![id, fmt_date(d.period_start), fmt_date(d.closing_date), fmt_date(d.due_date), now],
        )?;
    }
    let candidates: Vec<(String, String, Option<String>)> = {
        let mut stmt = tx.prepare(
            "SELECT t.id, t.date, t.statement_id FROM txn t LEFT JOIN card_statement s ON s.id = t.statement_id
              WHERE t.account_id = ?1 AND t.kind <> 'card_payment' AND t.date >= ?2
                AND (s.id IS NULL OR s.closing_date >= ?3)",
        )?;
        stmt.query_map(params![account_id, cutoff, today_s], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let mut touched: Vec<Id> = open.iter().map(|(id, _, _)| id.clone()).collect();
    for (id, date, cur) in candidates {
        let d = statement_dates(parse_date(&date)?, card.closing_day, card.due_day);
        let existing_close: Option<String> = tx
            .query_row(
                "SELECT closing_date FROM card_statement WHERE account_id = ?1 AND reference_month = ?2",
                params![account_id, d.reference_month],
                |r| r.get(0),
            )
            .optional()?;
        let target_closed = match &existing_close {
            Some(c) => c.as_str() < today_s.as_str(),
            None => d.closing_date < today,
        };
        if target_closed {
            continue; // cairia numa fatura já fechada: fica onde está
        }
        let target = statement_for(tx, account_id, &date)?;
        if target != cur {
            tx.execute(
                "UPDATE txn SET statement_id = ?2, updated_at = ?3 WHERE id = ?1",
                params![id, target, now],
            )?;
            touched.extend(target);
        }
    }
    // Faturas futuras que ficaram vazias.
    tx.execute(
        &format!(
            "DELETE FROM card_statement WHERE account_id = ?1 AND closing_date >= ?2 AND informed_total IS NULL
                AND NOT EXISTS (SELECT 1 FROM txn t WHERE t.statement_id = card_statement.id)
                AND id IN ({})",
            if touched.is_empty() { "NULL".to_string() } else { placeholders(touched.len()) }
        ),
        rusqlite::params_from_iter(
            [account_id.to_string(), today_s.clone()]
                .into_iter()
                .chain(touched.iter().cloned()),
        ),
    )?;
    touched
        .retain(|id| crate::transactions::util::exists(tx, "card_statement", id).unwrap_or(false));
    sync_statuses(tx, &touched, today)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dates(date: &str, c: u32, d: u32) -> (String, String, String, String) {
        let s = statement_dates(parse_date(date).unwrap(), c, d);
        (
            s.reference_month,
            fmt_date(s.period_start),
            fmt_date(s.closing_date),
            fmt_date(s.due_date),
        )
    }

    #[test]
    fn purchase_before_and_after_closing() {
        // Closes on the 3rd, due on the 10th (same month).
        assert_eq!(
            dates("2026-10-02", 3, 10),
            (
                "2026-10".into(),
                "2026-09-04".into(),
                "2026-10-03".into(),
                "2026-10-10".into()
            )
        );
        assert_eq!(dates("2026-10-03", 3, 10).0, "2026-10");
        assert_eq!(
            dates("2026-10-04", 3, 10),
            (
                "2026-11".into(),
                "2026-10-04".into(),
                "2026-11-03".into(),
                "2026-11-10".into()
            )
        );
        // Closes on the 25th, due on the 5th (next month).
        assert_eq!(
            dates("2026-10-20", 25, 5),
            (
                "2026-11".into(),
                "2026-09-26".into(),
                "2026-10-25".into(),
                "2026-11-05".into()
            )
        );
        assert_eq!(dates("2026-12-26", 25, 5).0, "2027-02");
    }

    #[test]
    fn short_months_clamp() {
        // Closes on the 31st: in February it closes on the 28th.
        assert_eq!(
            dates("2026-02-28", 31, 10),
            (
                "2026-03".into(),
                "2026-02-01".into(),
                "2026-02-28".into(),
                "2026-03-10".into()
            )
        );
        assert_eq!(dates("2026-03-01", 31, 10).2, "2026-03-31");
    }
}

#[cfg(test)]
mod tests_domain;

#[cfg(test)]
mod tests_plastics;

#[cfg(test)]
mod tests_limits;

#[cfg(test)]
mod tests_charges;

#[cfg(test)]
mod tests_rewards;

#[cfg(test)]
mod tests_loyalty;

#[cfg(test)]
mod tests_value;
