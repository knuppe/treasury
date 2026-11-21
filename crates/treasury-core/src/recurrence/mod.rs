//! Recorrências (salário, aluguel, assinaturas): regra RRULE (subconjunto) que gera
//! lançamentos previstos até um horizonte.

pub mod api;
pub mod rrule;

use std::collections::HashSet;

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::{AccountKind, currency_and_kind};
use crate::changes::{self, Author};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::storage::Db;
use crate::time::{fmt_date, now_utc, parse_date, today_local};
use crate::transactions::util::{clean_opt, require_member};
use crate::transactions::{NewTxn, TxnKind, TxnStatus};

/// Horizonte padrão para gerar lançamentos previstos.
pub const DEFAULT_HORIZON_DAYS: i64 = 62;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Forma estruturada da regra (a interface usa esta; o banco guarda a RRULE em texto).
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct RuleSpec {
    pub freq: Freq,
    /// A cada N períodos (padrão 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub interval: Option<u32>,
    /// Dia do mês (MONTHLY/YEARLY). 31 = último dia em meses curtos.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub by_month_day: Option<u32>,
    /// Dias da semana (WEEKLY): `MO`, `TU`, … `SU`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub by_day: Option<Vec<String>>,
    /// Mês (YEARLY), 1–12.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub by_month: Option<u32>,
    /// Número de ocorrências.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Recurrence {
    pub id: Id,
    pub description: String,
    pub account_id: Id,
    pub account_name: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    pub kind: TxnKind,
    pub category_id: Option<Id>,
    pub member_id: Option<Id>,
    pub to_account_id: Option<Id>,
    /// Texto RRULE, ex.: `FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=5`.
    pub rrule: String,
    pub spec: RuleSpec,
    /// Descrição legível: "todo mês no dia 5".
    pub human: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub active: bool,
    /// Próximas datas (até 3).
    pub next_dates: Vec<String>,
    /// Valor mensal equivalente (para "quanto está comprometido por mês").
    #[ts(type = "number")]
    pub monthly_equivalent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecurrenceInput {
    pub description: String,
    pub account_id: Id,
    /// Com sinal, como o lançamento.
    #[ts(type = "number")]
    pub amount: i64,
    pub kind: TxnKind,
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Para transferências recorrentes.
    #[ts(optional)]
    pub to_account_id: Option<Id>,
    pub spec: RuleSpec,
    pub start_date: String,
    #[ts(optional)]
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecurrencePatch {
    pub id: Id,
    #[ts(optional)]
    pub description: Option<String>,
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub category_id: Option<Option<Id>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub member_id: Option<Option<Id>>,
    #[ts(optional)]
    pub spec: Option<RuleSpec>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub end_date: Option<Option<String>>,
    #[ts(optional)]
    pub active: Option<bool>,
    /// Reaplicar a mudança aos previstos futuros já gerados (padrão: sim).
    #[ts(optional)]
    pub update_planned: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecurrenceListArgs {
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Só ativas (`true`) ou só pausadas (`false`). Padrão: todas.
    #[ts(optional)]
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecurrenceDeleteArgs {
    pub id: Id,
    /// Apagar também os lançamentos previstos gerados (padrão: sim). Os já efetivados
    /// ficam, sem vínculo com a recorrência.
    #[ts(optional)]
    pub delete_planned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecurrenceDeleteResult {
    /// Lançamentos previstos apagados junto.
    #[ts(type = "number")]
    pub deleted_planned: i64,
    pub change_set_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PreviewArgs {
    pub spec: RuleSpec,
    pub start_date: String,
    /// Quantas datas (padrão 12, máximo 366).
    #[ts(optional)]
    pub count: Option<u32>,
    #[ts(optional)]
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecurrencePreview {
    pub rrule: String,
    pub human: String,
    pub dates: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MaterializeArgs {
    /// Dias à frente (padrão 62, máximo 731).
    #[ts(optional, type = "number")]
    pub horizon_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MaterializeResult {
    /// Lançamentos previstos criados (transferências contam as duas pernas).
    #[ts(type = "number")]
    pub created: i64,
    pub change_set_id: Option<Id>,
}

/// Horizonte máximo aceito em `materialize`.
pub const MAX_HORIZON_DAYS: i64 = 731;

/// Linha crua de `recurrence`.
#[derive(Debug, Clone)]
struct RecRow {
    id: Id,
    description: String,
    account_id: Id,
    account_name: String,
    amount: i64,
    currency: String,
    kind: TxnKind,
    category_id: Option<Id>,
    member_id: Option<Id>,
    to_account_id: Option<Id>,
    rrule: String,
    start_date: String,
    end_date: Option<String>,
    generated_until: Option<String>,
    active: bool,
}

const REC_SQL: &str =
    "SELECT r.id, r.description, r.account_id, a.name, r.amount, r.currency, r.kind, r.category_id,
       r.member_id, r.to_account_id, r.rrule, r.start_date, r.end_date, r.generated_until, r.active,
       (SELECT kind FROM account WHERE id = r.to_account_id)
  FROM recurrence r JOIN account a ON a.id = r.account_id";

fn row_to_rec(r: &rusqlite::Row<'_>) -> rusqlite::Result<RecRow> {
    let kind: String = r.get(6)?;
    let to_kind: Option<String> = r.get(15)?;
    let kind = match kind.as_str() {
        "income" => TxnKind::Income,
        "expense" => TxnKind::Expense,
        "investment" => TxnKind::Investment,
        // Transferência para cartão é pagamento de fatura.
        _ if to_kind.as_deref() == Some("credit_card") => TxnKind::CardPayment,
        _ => TxnKind::Transfer,
    };
    Ok(RecRow {
        id: r.get(0)?,
        description: r.get(1)?,
        account_id: r.get(2)?,
        account_name: r.get(3)?,
        amount: r.get(4)?,
        currency: r.get(5)?,
        kind,
        category_id: r.get(7)?,
        member_id: r.get(8)?,
        to_account_id: r.get(9)?,
        rrule: r.get(10)?,
        start_date: r.get(11)?,
        end_date: r.get(12)?,
        generated_until: r.get(13)?,
        active: r.get(14)?,
    })
}

fn load(conn: &Connection, id: &str) -> Result<RecRow> {
    conn.query_row(&format!("{REC_SQL} WHERE r.id = ?1"), [id], row_to_rec)
        .optional()?
        .ok_or_else(|| {
            Error::not_found(t_args(
                "nucleo-lancamentos-ref-recorrencia",
                &args(&[("id", id.to_string().into())]),
            ))
        })
}

fn db_kind(k: TxnKind) -> &'static str {
    match k {
        TxnKind::Income => "income",
        TxnKind::Expense => "expense",
        TxnKind::Investment => "investment",
        _ => "transfer",
    }
}

fn is_two_legged(k: TxnKind) -> bool {
    k.is_transfer_like()
}

fn to_view(r: RecRow, today: NaiveDate) -> Result<Recurrence> {
    let start = parse_date(&r.start_date)?;
    let end = r.end_date.as_deref().map(parse_date).transpose()?;
    let spec = rrule::normalize(&rrule::parse(&r.rrule)?, start)?;
    let next_dates = if r.active {
        rrule::next_n(&spec, start, end, today, 3)?
            .into_iter()
            .map(fmt_date)
            .collect()
    } else {
        vec![]
    };
    let mut human = rrule::human(&spec);
    if let Some(e) = &r.end_date {
        human = t_args(
            "nucleo-lancamentos-rrule-ate",
            &args(&[
                ("regra", human.into()),
                ("data", crate::transactions::util::date_text(e).into()),
            ]),
        );
    }
    Ok(Recurrence {
        monthly_equivalent: rrule::monthly_equivalent(&spec, r.amount),
        id: r.id,
        description: r.description,
        account_id: r.account_id,
        account_name: r.account_name,
        amount: r.amount,
        currency: r.currency,
        kind: r.kind,
        category_id: r.category_id,
        member_id: r.member_id,
        to_account_id: r.to_account_id,
        rrule: r.rrule,
        spec,
        human,
        start_date: r.start_date,
        end_date: r.end_date,
        active: r.active,
        next_dates,
    })
}

pub fn list(
    conn: &Connection,
    args: &RecurrenceListArgs,
    today: NaiveDate,
) -> Result<Vec<Recurrence>> {
    let mut stmt = conn.prepare(&format!(
        "{REC_SQL} WHERE (?1 IS NULL OR r.account_id = ?1 OR r.to_account_id = ?1)
           AND (?2 IS NULL OR r.active = ?2)
         ORDER BY r.active DESC, r.description COLLATE NOCASE, r.id"
    ))?;
    let rows = stmt
        .query_map(params![args.account_id, args.active], row_to_rec)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.into_iter().map(|r| to_view(r, today)).collect()
}

pub fn get(conn: &Connection, id: &str, today: NaiveDate) -> Result<Recurrence> {
    to_view(load(conn, id)?, today)
}

pub fn preview(args: &PreviewArgs) -> Result<RecurrencePreview> {
    let start = parse_date(&args.start_date)?;
    let end = args.end_date.as_deref().map(parse_date).transpose()?;
    let spec = rrule::normalize(&args.spec, start)?;
    let n = args.count.unwrap_or(12).clamp(1, 366) as usize;
    Ok(RecurrencePreview {
        rrule: rrule::to_rrule(&spec),
        human: rrule::human(&spec),
        dates: rrule::next_n(&spec, start, end, start, n)?
            .into_iter()
            .map(fmt_date)
            .collect(),
    })
}

/// Valida valor, tipo, contas e categoria; devolve o valor com o sinal normalizado
/// (despesa e transferências negativas na conta de origem; receita positiva).
fn check_fields(
    conn: &Connection,
    account_id: &str,
    kind: TxnKind,
    amount: i64,
    to_account_id: Option<&str>,
    category_id: Option<&str>,
) -> Result<i64> {
    if amount == 0 {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-recorrencia-erro-valor-zero",
        )));
    }
    let (cur, _) = currency_and_kind(conn, account_id)?;
    match kind {
        TxnKind::Income | TxnKind::Expense => {
            if to_account_id.is_some() {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-recorrencia-erro-destino-so-transferencia",
                )));
            }
            crate::transactions::check_category(conn, category_id, kind)?;
            Ok(if kind == TxnKind::Income {
                amount.abs()
            } else {
                -amount.abs()
            })
        }
        TxnKind::Transfer | TxnKind::Investment | TxnKind::CardPayment => {
            let to = to_account_id.ok_or_else(|| {
                Error::invalid(t("nucleo-lancamentos-recorrencia-erro-informe-destino"))
            })?;
            if to == account_id {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-transferencia-mesma-conta",
                )));
            }
            let (to_cur, to_kind) = currency_and_kind(conn, to)?;
            if to_cur != cur {
                return Err(Error::invalid(t_args(
                    "nucleo-lancamentos-recorrencia-erro-moedas",
                    &args(&[
                        ("origem", cur.clone().into()),
                        ("destino", to_cur.clone().into()),
                    ]),
                )));
            }
            if kind == TxnKind::CardPayment && to_kind != AccountKind::CreditCard {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-recorrencia-erro-pagamento-cartao",
                )));
            }
            if category_id.is_some() {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-recorrencia-erro-transferencia-categoria",
                )));
            }
            Ok(-amount.abs())
        }
        TxnKind::Adjustment => Err(Error::invalid(t(
            "nucleo-lancamentos-recorrencia-erro-ajuste",
        ))),
    }
}

pub fn create(tx: &Transaction<'_>, input: &RecurrenceInput, today: NaiveDate) -> Result<Id> {
    let description = input.description.trim().to_string();
    if description.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-recorrencia-erro-descricao",
        )));
    }
    let start = parse_date(&input.start_date)?;
    let end = clean_opt(input.end_date.clone());
    if let Some(e) = &end
        && parse_date(e)? < start
    {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-recorrencia-erro-fim-antes",
        )));
    }
    let category_id = clean_opt(input.category_id.clone());
    let to_account_id = clean_opt(input.to_account_id.clone());
    let amount = check_fields(
        tx,
        &input.account_id,
        input.kind,
        input.amount,
        to_account_id.as_deref(),
        category_id.as_deref(),
    )?;
    let member_id = clean_opt(input.member_id.clone());
    if let Some(m) = &member_id {
        require_member(tx, m)?;
    }
    let spec = rrule::normalize(&input.spec, start)?;
    let (currency, _) = currency_and_kind(tx, &input.account_id)?;
    let id = new_id();
    tx.execute(
        "INSERT INTO recurrence (id, description, account_id, amount, currency, kind, category_id, member_id,
                                 to_account_id, rrule, start_date, end_date, active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1, ?13, ?13)",
        params![
            id,
            description,
            input.account_id,
            amount,
            currency,
            db_kind(input.kind),
            category_id,
            member_id,
            to_account_id,
            rrule::to_rrule(&spec),
            input.start_date,
            end,
            now_utc()
        ],
    )?;
    materialize_one(tx, &load(tx, &id)?, today, DEFAULT_HORIZON_DAYS)?;
    Ok(id)
}

/// Lançamentos previstos da recorrência com data ≥ `from`.
fn planned_rows(
    tx: &Transaction<'_>,
    id: &str,
    from: Option<&str>,
) -> Result<Vec<crate::transactions::TxnRow>> {
    let ids: Vec<String> = {
        let mut stmt = tx.prepare(
            "SELECT id FROM txn WHERE recurrence_id = ?1 AND status = 'planned' AND (?2 IS NULL OR date >= ?2)",
        )?;
        stmt.query_map(params![id, from], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    ids.iter()
        .map(|i| crate::transactions::load_row(tx, i))
        .collect()
}

pub fn update(tx: &Transaction<'_>, patch: &RecurrencePatch, today: NaiveDate) -> Result<()> {
    let cur = load(tx, &patch.id)?;
    let start = parse_date(&cur.start_date)?;
    let description = match &patch.description {
        Some(d) if d.trim().is_empty() => {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-recorrencia-erro-descricao",
            )));
        }
        Some(d) => d.trim().to_string(),
        None => cur.description.clone(),
    };
    let category_id = match &patch.category_id {
        Some(c) => clean_opt(c.clone()),
        None => cur.category_id.clone(),
    };
    let member_id = match &patch.member_id {
        Some(m) => clean_opt(m.clone()),
        None => cur.member_id.clone(),
    };
    if let Some(m) = &member_id {
        require_member(tx, m)?;
    }
    let amount = check_fields(
        tx,
        &cur.account_id,
        cur.kind,
        patch.amount.unwrap_or(cur.amount),
        cur.to_account_id.as_deref(),
        if category_id != cur.category_id {
            category_id.as_deref()
        } else {
            None
        },
    )?;
    let rrule_text = match &patch.spec {
        Some(s) => rrule::to_rrule(&rrule::normalize(s, start)?),
        None => cur.rrule.clone(),
    };
    let end_date = match &patch.end_date {
        Some(e) => clean_opt(e.clone()),
        None => cur.end_date.clone(),
    };
    if let Some(e) = &end_date
        && parse_date(e)? < start
    {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-recorrencia-erro-fim-antes",
        )));
    }
    let active = patch.active.unwrap_or(cur.active);
    let dates_changed = rrule_text != cur.rrule || end_date != cur.end_date;
    let values_changed = amount != cur.amount
        || description != cur.description
        || category_id != cur.category_id
        || member_id != cur.member_id;
    let yesterday = fmt_date(today - Duration::days(1));
    let mut generated_until = cur.generated_until.clone();
    let update_planned = patch.update_planned.unwrap_or(true);
    let today_s = fmt_date(today);
    if update_planned && (dates_changed || (cur.active && !active)) {
        // Datas mudaram (ou pausou): refaz os previstos futuros.
        let rows = planned_rows(tx, &cur.id, Some(&today_s))?;
        crate::transactions::delete_rows(tx, &rows, today)?;
        if generated_until
            .as_deref()
            .is_some_and(|g| g > yesterday.as_str())
        {
            generated_until = Some(yesterday.clone());
        }
    } else if update_planned && values_changed {
        for row in planned_rows(tx, &cur.id, Some(&today_s))? {
            let leg_amount = if row.account_id == cur.account_id {
                amount
            } else {
                -amount
            };
            let cat = if row.kind.is_cash_flow() {
                category_id.clone()
            } else {
                None
            };
            tx.execute(
                // Sem membro na recorrência: o lançamento fica com o da conta (herdado).
                "UPDATE txn SET amount = ?2, description = ?3, category_id = ?4,
                        member_id = COALESCE(?5, CASE WHEN member_origin = 'auto' THEN member_id END),
                        member_origin = CASE WHEN ?5 IS NOT NULL THEN 'manual'
                                             WHEN member_origin = 'auto' THEN 'auto' END,
                        fingerprint = ?6, updated_at = ?7
                  WHERE id = ?1",
                params![
                    row.id,
                    leg_amount,
                    description,
                    cat,
                    member_id,
                    crate::transactions::fingerprint(&row.date, leg_amount, &description),
                    now_utc()
                ],
            )?;
            if let Some(s) = &row.statement_id {
                crate::cards::sync_statuses(tx, std::slice::from_ref(s), today)?;
            }
        }
    }
    if !cur.active && active {
        // Reativada: não gera o período em que ficou pausada.
        if generated_until
            .as_deref()
            .is_none_or(|g| g < yesterday.as_str())
        {
            generated_until = Some(yesterday);
        }
    }
    tx.execute(
        "UPDATE recurrence SET description = ?2, amount = ?3, category_id = ?4, member_id = ?5, rrule = ?6,
                end_date = ?7, active = ?8, generated_until = ?9, updated_at = ?10
          WHERE id = ?1 AND (description IS NOT ?2 OR amount IS NOT ?3 OR category_id IS NOT ?4
                OR member_id IS NOT ?5 OR rrule IS NOT ?6 OR end_date IS NOT ?7 OR active IS NOT ?8
                OR generated_until IS NOT ?9)",
        params![
            cur.id,
            description,
            amount,
            category_id,
            member_id,
            rrule_text,
            end_date,
            active,
            generated_until,
            now_utc()
        ],
    )?;
    if active {
        materialize_one(tx, &load(tx, &cur.id)?, today, DEFAULT_HORIZON_DAYS)?;
    }
    Ok(())
}

/// Apaga a recorrência. Devolve quantos previstos foram apagados.
pub fn delete(
    tx: &Transaction<'_>,
    id: &str,
    delete_planned: bool,
    today: NaiveDate,
) -> Result<usize> {
    load(tx, id)?;
    let removed = if delete_planned {
        let rows = planned_rows(tx, id, None)?;
        crate::transactions::delete_rows(tx, &rows, today)?
    } else {
        0
    };
    tx.execute(
        "UPDATE txn SET recurrence_id = NULL, updated_at = ?2 WHERE recurrence_id = ?1",
        params![id, now_utc()],
    )?;
    tx.execute("DELETE FROM recurrence WHERE id = ?1", [id])?;
    Ok(removed)
}

/// Gera os previstos de uma recorrência até `today + horizon_days` (ou a data final).
/// Idempotente: começa depois de `generated_until` (previstos apagados pelo usuário
/// não voltam) e pula datas que já têm lançamento da recorrência.
fn materialize_one(
    tx: &Transaction<'_>,
    r: &RecRow,
    today: NaiveDate,
    horizon_days: i64,
) -> Result<usize> {
    if !r.active {
        return Ok(0);
    }
    let start = parse_date(&r.start_date)?;
    let end = r.end_date.as_deref().map(parse_date).transpose()?;
    let mut until = today + Duration::days(horizon_days.clamp(0, MAX_HORIZON_DAYS));
    if let Some(e) = end {
        until = until.min(e);
    }
    let from = match &r.generated_until {
        Some(g) => parse_date(g)? + Duration::days(1),
        // Primeira geração: nada no passado (já deve ter sido lançado ou importado).
        None => today,
    }
    .max(start);
    if from > until {
        return Ok(0);
    }
    let spec = rrule::parse(&r.rrule)?;
    let dates = rrule::occurrences_between(&spec, start, end, from, until)?;
    if dates.is_empty() {
        return Ok(0);
    }
    let existing: HashSet<String> = {
        let mut stmt = tx.prepare(
            "SELECT DISTINCT date FROM txn WHERE recurrence_id = ?1 AND date >= ?2 AND date <= ?3",
        )?;
        stmt.query_map(params![r.id, fmt_date(from), fmt_date(until)], |row| {
            row.get(0)
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let mut created = 0;
    let mut touched = Vec::new();
    for d in dates
        .into_iter()
        .map(fmt_date)
        .filter(|d| !existing.contains(d))
    {
        let mut t = NewTxn::new(
            &r.account_id,
            &d,
            r.amount,
            &r.currency,
            r.kind,
            &r.description,
        );
        t.status = TxnStatus::Planned;
        t.category_id = r.category_id.clone().filter(|_| r.kind.is_cash_flow());
        t.member_id = r.member_id.clone();
        t.recurrence_id = Some(r.id.clone());
        if is_two_legged(r.kind) {
            let to = r.to_account_id.clone().ok_or_else(|| {
                Error::invalid(crate::i18n::t(
                    "nucleo-lancamentos-recorrencia-erro-sem-destino",
                ))
            })?;
            let transfer_id = new_id();
            t.transfer_id = Some(transfer_id.clone());
            let mut leg = NewTxn::new(&to, &d, -r.amount, &r.currency, r.kind, &r.description);
            leg.status = TxnStatus::Planned;
            leg.member_id = r.member_id.clone();
            leg.recurrence_id = Some(r.id.clone());
            leg.transfer_id = Some(transfer_id);
            if r.kind == TxnKind::CardPayment {
                leg.statement_id = crate::cards::statement_to_pay(tx, &to, &d)?;
            }
            crate::transactions::insert(tx, &t)?;
            crate::transactions::insert(tx, &leg)?;
            created += 2;
            touched.extend(leg.statement_id);
        } else {
            let id = crate::transactions::insert(tx, &t)?;
            created += 1;
            if let Some(s) =
                tx.query_row("SELECT statement_id FROM txn WHERE id = ?1", [&id], |row| {
                    row.get::<_, Option<String>>(0)
                })?
            {
                touched.push(s);
            }
        }
    }
    tx.execute(
        "UPDATE recurrence SET generated_until = ?2 WHERE id = ?1 AND (generated_until IS NULL OR generated_until < ?2)",
        params![r.id, fmt_date(until)],
    )?;
    touched.sort();
    touched.dedup();
    crate::cards::sync_statuses(tx, &touched, today)?;
    Ok(created)
}

/// Gera os previstos de todas as recorrências ativas. Devolve quantos lançamentos criou.
pub fn materialize_all(tx: &Transaction<'_>, today: NaiveDate, horizon_days: i64) -> Result<usize> {
    let rows: Vec<RecRow> = {
        let mut stmt = tx.prepare(&format!("{REC_SQL} WHERE r.active = 1 ORDER BY r.id"))?;
        stmt.query_map([], row_to_rec)?
            .collect::<rusqlite::Result<_>>()?
    };
    let mut n = 0;
    for r in &rows {
        n += materialize_one(tx, r, today, horizon_days)?;
    }
    Ok(n)
}

/// Rotinas de desbloqueio com uma data de referência (testável): previstos das
/// recorrências e parcelas do cartão que venceram. Devolve (previstos, parcelas).
pub fn run_unlock_routines(conn: &mut Connection, today: NaiveDate) -> Result<(usize, usize)> {
    let created = changes::record(
        conn,
        &Author::system(t("nucleo-lancamentos-autor-recorrencias")),
        &t("nucleo-lancamentos-historico-previstos"),
        |tx| materialize_all(tx, today, DEFAULT_HORIZON_DAYS),
    )?
    .value;
    let cleared = changes::record(
        conn,
        &Author::system(t("nucleo-lancamentos-autor-cartoes")),
        &t("nucleo-lancamentos-historico-parcelas-efetivadas"),
        |tx| crate::cards::clear_due_installments(tx, today),
    )?
    .value;
    Ok((created, cleared))
}

/// Chamado ao desbloquear o cofre: gera os previstos até o horizonte e efetiva as
/// parcelas do cartão cuja data chegou.
pub fn on_unlock(db: &mut Db) -> Result<()> {
    if db.is_read_only() {
        return Ok(());
    }
    run_unlock_routines(db.conn_mut(), today_local())?;
    Ok(())
}

#[cfg(test)]
mod tests;
