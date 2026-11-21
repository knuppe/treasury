//! Rotativo do cartão (docs/design/cards.md, seção 5.3): fatura vencida que ficou com saldo a
//! pagar entra no rotativo e os juros, a multa, a mora e o IOF chegam nas faturas seguintes.
//!
//! No Brasil, o rotativo só pode durar até a fatura seguinte (Res. CMN 4.549/2017) e os juros e
//! encargos do rotativo e do parcelamento da fatura não podem passar de 100% do valor original
//! da dívida (Lei 14.690/2023). O app só compara o que foi lançado com esse teto; nunca calcula
//! juros que o banco não informou.

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::ids::Id;
use crate::time::fmt_date;

/// Situação do rotativo de um cartão.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RevolvingStatus {
    /// Primeira fatura vencida com saldo (início do episódio).
    pub statement_id: Id,
    pub reference_month: String,
    pub due_date: String,
    /// O que ficou sem pagar nessa fatura (base do teto).
    #[ts(type = "number")]
    pub unpaid: i64,
    /// Faturas vencidas seguidas com saldo a pagar desde então.
    pub statements: u32,
    /// Juros, encargos, multa, mora e IOF lançados nas faturas seguintes.
    #[ts(type = "number")]
    pub charges: i64,
    /// `charges ÷ unpaid` em pontos-base.
    pub charges_bp: u32,
    /// Encargos acima do valor original (o teto da Lei 14.690/2023, no Brasil).
    pub over_cap: bool,
    /// Ficou mais de uma fatura no rotativo (no Brasil, o banco deve oferecer parcelamento).
    pub beyond_next_statement: bool,
    pub currency: String,
}

/// Lançamento de encargo: juros, encargos, rotativo, multa, mora ou IOF.
pub fn is_charge(desc: &str) -> bool {
    let d = crate::transactions::normalize_description(desc);
    let words: Vec<&str> = d.split_whitespace().collect();
    words.iter().any(|w| {
        matches!(
            *w,
            "juros" | "encargo" | "encargos" | "rotativo" | "multa" | "mora" | "iof"
        )
    }) && !d.contains("compra internacional")
        && !d.contains("transacoes no exterior")
}

/// Rotativo em curso: as faturas vencidas mais recentes, seguidas, com saldo a pagar.
/// Só em cartões com pagamento de fatura registrado: sem nenhum, uma fatura "em aberto" é falta
/// de dado, não rotativo.
pub fn status(
    conn: &Connection,
    account_id: &str,
    today: NaiveDate,
) -> Result<Option<RevolvingStatus>> {
    let tracked: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM txn WHERE account_id = ?1 AND kind = 'card_payment'
                          AND status <> 'planned')",
        [account_id],
        |r| r.get(0),
    )?;
    if !tracked {
        return Ok(None);
    }
    let today_s = fmt_date(today);
    let mut views = super::list_statements(
        conn,
        &super::ListStatementsArgs {
            account_id: Some(account_id.to_string()),
            from_month: None,
            to_month: None,
        },
        today,
    )?;
    views.retain(|v| v.due_date < today_s);
    // Da mais recente para trás, enquanto a fatura vencida tem saldo.
    let mut run: Vec<&super::StatementView> = vec![];
    for v in views.iter().rev() {
        if v.remaining > 0 {
            run.push(v);
        } else {
            break;
        }
    }
    let Some(first) = run.last() else {
        return Ok(None);
    };
    let first = (*first).clone();
    // Encargos lançados nas faturas depois da primeira (inclusive a em aberto).
    let mut stmt = conn.prepare_cached(
        "SELECT t.description, t.amount FROM txn t JOIN card_statement s ON s.id = t.statement_id
          WHERE t.account_id = ?1 AND s.reference_month > ?2 AND t.kind <> 'card_payment' AND t.amount < 0",
    )?;
    let mut q = stmt.query(rusqlite::params![account_id, first.reference_month])?;
    let mut charges = 0i64;
    while let Some(r) = q.next()? {
        let (desc, amount): (String, i64) = (r.get(0)?, r.get(1)?);
        if is_charge(&desc) {
            charges -= amount;
        }
    }
    let unpaid = first.remaining;
    let charges_bp = if unpaid > 0 {
        (charges as i128 * 10_000 / unpaid as i128).clamp(0, u32::MAX as i128) as u32
    } else {
        0
    };
    Ok(Some(RevolvingStatus {
        statement_id: first.id.clone(),
        reference_month: first.reference_month.clone(),
        due_date: first.due_date.clone(),
        unpaid,
        statements: run.len() as u32,
        charges,
        charges_bp,
        over_cap: charges > unpaid,
        beyond_next_statement: run.len() > 1,
        currency: first.currency.clone(),
    }))
}
