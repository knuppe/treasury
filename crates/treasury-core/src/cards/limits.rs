//! Limite do cartão (docs/design/cards.md, fase 2): uso, histórico, melhor dia de compra e o
//! resumo do cartão que junta tudo (limite, ciclo, anuidade).

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc, parse_date};

/// Faixas de uso do limite que geram aviso (em pontos-base: 8000 = 80%).
pub const WARN_BP: [u32; 2] = [8000, 9500];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LimitSource {
    Manual,
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LimitChange {
    pub id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub credit_limit: i64,
    pub source: LimitSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LimitChangeInput {
    pub account_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub credit_limit: i64,
}

/// Resumo do cartão: limite e uso, ciclo da fatura em aberto, melhor dia de compra, anuidade.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardOverview {
    pub account_id: Id,
    pub account_name: String,
    pub currency: String,
    #[ts(type = "number | null")]
    pub credit_limit: Option<i64>,
    /// Saldo devedor, incluindo as parcelas futuras já comprometidas (positivo = devido).
    #[ts(type = "number")]
    pub used: i64,
    /// Parte de `used` que são parcelas futuras.
    #[ts(type = "number")]
    pub committed_installments: i64,
    #[ts(type = "number | null")]
    pub available: Option<i64>,
    /// Uso do limite em pontos-base (10000 = 100%); `null` sem limite.
    pub utilization_bp: Option<u32>,
    /// Compras a partir desta data entram na próxima fatura (dia seguinte ao fechamento).
    pub best_purchase_date: String,
    /// Vencimento da fatura dessas compras.
    pub best_purchase_due_date: String,
    /// Dias entre o melhor dia de compra e o vencimento.
    #[ts(type = "number")]
    pub best_purchase_days: i64,
    pub open_statement: Option<super::StatementView>,
    pub limit_history: Vec<LimitChange>,
    pub annual_fee: Option<super::annual_fee::AnnualFeeStatus>,
    /// Fatura vencida com saldo a pagar (rotativo) e os encargos que vieram depois.
    pub revolving: Option<super::revolving::RevolvingStatus>,
    /// País da família é o Brasil (as regras do rotativo citadas valem só lá).
    pub brazil_rules: bool,
}

fn map(r: &rusqlite::Row<'_>) -> rusqlite::Result<LimitChange> {
    let source: String = r.get(3)?;
    Ok(LimitChange {
        id: r.get(0)?,
        date: r.get(1)?,
        credit_limit: r.get(2)?,
        source: if source == "import" {
            LimitSource::Import
        } else {
            LimitSource::Manual
        },
    })
}

pub fn history(conn: &Connection, account_id: &str) -> Result<Vec<LimitChange>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, credit_limit, source FROM card_limit_change WHERE account_id = ?1
          ORDER BY date DESC, created_at DESC",
    )?;
    let rows = stmt
        .query_map([account_id], map)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Limite em vigor numa data (pelo histórico), para o uso de meses passados.
pub fn limit_at(conn: &Connection, account_id: &str, date: &str) -> Result<Option<i64>> {
    Ok(conn
        .prepare_cached(
            "SELECT credit_limit FROM card_limit_change WHERE account_id = ?1 AND date <= ?2
              ORDER BY date DESC, created_at DESC LIMIT 1",
        )?
        .query_row(params![account_id, date], |r| r.get(0))
        .optional()?)
}

fn insert(
    tx: &Transaction<'_>,
    account_id: &str,
    date: &str,
    limit: i64,
    source: &str,
) -> Result<Id> {
    let id = new_id();
    tx.execute(
        "INSERT INTO card_limit_change (id, account_id, date, credit_limit, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, account_id, date, limit, source, now_utc()],
    )?;
    Ok(id)
}

/// O mais recente do histórico vira o limite em vigor do cartão.
fn sync_current(tx: &Transaction<'_>, account_id: &str) -> Result<()> {
    let latest: Option<i64> = tx
        .query_row(
            "SELECT credit_limit FROM card_limit_change WHERE account_id = ?1
              ORDER BY date DESC, created_at DESC LIMIT 1",
            [account_id],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(l) = latest {
        tx.execute(
            "UPDATE credit_card SET credit_limit = ?2, updated_at = ?3
              WHERE account_id = ?1 AND credit_limit IS NOT ?2",
            params![account_id, l, now_utc()],
        )?;
    }
    Ok(())
}

/// O limite mudou no cadastro do cartão: registra no histórico (na abertura da conta, se é o
/// primeiro; senão hoje).
pub fn record_manual(
    tx: &Transaction<'_>,
    account_id: &str,
    old: Option<i64>,
    new: Option<i64>,
) -> Result<()> {
    let Some(new) = new else { return Ok(()) };
    if old == Some(new) {
        return Ok(());
    }
    let has_history: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM card_limit_change WHERE account_id = ?1)",
        [account_id],
        |r| r.get(0),
    )?;
    let date = if has_history {
        fmt_date(crate::time::today_local())
    } else {
        tx.query_row(
            "SELECT opening_date FROM account WHERE id = ?1",
            [account_id],
            |r| r.get(0),
        )?
    };
    // Mesmo dia: corrige o registro em vez de somar outro.
    let same_day: Option<String> = tx
        .query_row(
            "SELECT id FROM card_limit_change WHERE account_id = ?1 AND date = ?2 AND source = 'manual'",
            params![account_id, date],
            |r| r.get(0),
        )
        .optional()?;
    match same_day {
        Some(id) => {
            tx.execute(
                "UPDATE card_limit_change SET credit_limit = ?2, updated_at = ?3 WHERE id = ?1",
                params![id, new, now_utc()],
            )?;
        }
        None => {
            insert(tx, account_id, &date, new, "manual")?;
        }
    }
    Ok(())
}

/// Limite impresso numa fatura importada: registra quando muda e, se é o mais recente, vira o
/// limite em vigor.
pub fn record_import(tx: &Transaction<'_>, account_id: &str, date: &str, limit: i64) -> Result<()> {
    if limit < 0 {
        return Ok(());
    }
    let at = limit_at(tx, account_id, date)?;
    if at == Some(limit) {
        return Ok(());
    }
    insert(tx, account_id, date, limit, "import")?;
    sync_current(tx, account_id)
}

/// Registro de limite com data (aumento ou redução que o banco avisou).
pub fn add_change(tx: &Transaction<'_>, input: &LimitChangeInput) -> Result<Id> {
    super::require_card(tx, &input.account_id)?;
    parse_date(&input.date)?;
    if input.credit_limit < 0 {
        return Err(Error::invalid(t("nucleo-contas-cartao-erro-limite")));
    }
    let id = insert(
        tx,
        &input.account_id,
        &input.date,
        input.credit_limit,
        "manual",
    )?;
    sync_current(tx, &input.account_id)?;
    Ok(id)
}

pub fn delete_change(tx: &Transaction<'_>, id: &str) -> Result<()> {
    let account: Option<String> = tx
        .query_row(
            "SELECT account_id FROM card_limit_change WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(account) = account else {
        return Err(Error::not_found(t("nucleo-contas-ref-limite")));
    };
    tx.execute("DELETE FROM card_limit_change WHERE id = ?1", [id])?;
    sync_current(tx, &account)
}

/// Parcelas futuras (previstas) ainda não cobradas do cartão.
fn future_installments(conn: &Connection, account_id: &str) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(-amount), 0) FROM txn WHERE account_id = ?1
            AND installment_plan_id IS NOT NULL AND status = 'planned'",
        [account_id],
        |r| r.get(0),
    )?)
}

/// Uso em pontos-base (sem limite ou limite zero: `None`).
pub fn utilization_bp(used: i64, limit: Option<i64>) -> Option<u32> {
    match limit {
        Some(l) if l > 0 => {
            Some((used.max(0) as i128 * 10_000 / l as i128).min(u32::MAX as i128) as u32)
        }
        _ => None,
    }
}

pub fn overview(conn: &Connection, account_id: &str, today: NaiveDate) -> Result<CardOverview> {
    let card = super::require_card(conn, account_id)?;
    let (currency, _) = crate::accounts::currency_and_kind(conn, account_id)?;
    let name = crate::accounts::account_name(conn, account_id)?;
    let used = super::outstanding(conn, account_id)?;
    let best = super::statement_dates(today, card.closing_day, card.due_day);
    // Melhor dia: o seguinte ao fechamento da fatura em aberto.
    let best_date = best.closing_date.succ_opt().expect("data válida");
    let best_due = super::statement_dates(best_date, card.closing_day, card.due_day).due_date;
    let open_id: Option<String> = conn
        .query_row(
            "SELECT id FROM card_statement WHERE account_id = ?1 AND period_start <= ?2 AND closing_date >= ?2
              ORDER BY reference_month LIMIT 1",
            params![account_id, fmt_date(today)],
            |r| r.get(0),
        )
        .optional()?;
    let open_statement = open_id
        .map(|id| super::statement_view(conn, &id, today))
        .transpose()?;
    Ok(CardOverview {
        account_id: account_id.to_string(),
        account_name: name,
        currency,
        credit_limit: card.credit_limit,
        used,
        committed_installments: future_installments(conn, account_id)?,
        available: card.credit_limit.map(|l| l - used),
        utilization_bp: utilization_bp(used, card.credit_limit),
        best_purchase_date: fmt_date(best_date),
        best_purchase_due_date: fmt_date(best_due),
        best_purchase_days: (best_due - best_date).num_days(),
        open_statement,
        limit_history: history(conn, account_id)?,
        annual_fee: super::annual_fee::status(conn, account_id, today)?,
        revolving: super::revolving::status(conn, account_id, today)?,
        brazil_rules: crate::country::is_brazil(conn),
    })
}
