//! Ligação manual entre uma parcela do contrato e o lançamento que a pagou: candidatos
//! (saídas perto do vencimento), a ligação em si (a parcela fica paga com a data e o valor
//! do lançamento) e a limpeza quando o lançamento é apagado.

use rusqlite::{Connection, OptionalExtension, Transaction, params, params_from_iter};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n;
use crate::ids::Id;
use crate::time::now_utc;
use crate::transactions::{TxnKind, TxnStatus};

/// Distância máxima (dias) entre o lançamento e o vencimento da parcela nos candidatos.
pub const CANDIDATE_DAYS: i64 = 20;
/// Quantos candidatos mostrar.
pub const CANDIDATE_LIMIT: i64 = 20;

/// Lançamento que pode ser o pagamento de uma parcela (diálogo "Ligar a um lançamento").
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoanPaymentCandidate {
    pub txn_id: Id,
    pub date: String,
    pub description: String,
    pub account_id: Id,
    pub account_name: String,
    /// Valor do lançamento, com o sinal da conta (saída: negativo).
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    /// Na conta de débito do contrato.
    pub in_debit_account: bool,
}

/// Parcela, contrato e o que a ligação precisa saber dele.
struct Target {
    loan_id: Id,
    loan_name: String,
    number: u32,
    due_date: String,
    total: i64,
    currency: String,
    debit_account_id: Option<Id>,
    category_id: Option<Id>,
}

fn target(conn: &Connection, installment_id: &str) -> Result<Target> {
    conn.query_row(
        "SELECT l.id, l.name, li.number, li.due_date, li.total, l.currency, l.debit_account_id, l.category_id
           FROM loan_installment li JOIN loan l ON l.id = li.loan_id
          WHERE li.id = ?1",
        [installment_id],
        |r| {
            Ok(Target {
                loan_id: r.get(0)?,
                loan_name: r.get(1)?,
                number: r.get(2)?,
                due_date: r.get(3)?,
                total: r.get(4)?,
                currency: r.get(5)?,
                debit_account_id: r.get(6)?,
                category_id: r.get(7)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| Error::not_found(i18n::t("nucleo-planejamento-divida-parcela")))
}

/// Contrato e número da parcela (para o motivo da alteração).
pub fn describe(conn: &Connection, installment_id: &str) -> Result<(Id, String, u32)> {
    let t = target(conn, installment_id)?;
    Ok((t.loan_id, t.loan_name, t.number))
}

/// Saídas efetivadas, fora de transferências entre contas e ainda sem parcela, até
/// [`CANDIDATE_DAYS`] dias do vencimento: as de valor mais próximo do da parcela primeiro,
/// depois as de data mais próxima. Compras no cartão e movimentos de corretora ficam de
/// fora da lista (parcela se paga da conta), mas [`link_payment`] aceita qualquer saída.
pub fn payment_candidates(
    conn: &Connection,
    installment_id: &str,
) -> Result<Vec<LoanPaymentCandidate>> {
    let t = target(conn, installment_id)?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.date, t.description, t.account_id, a.name, t.amount, t.currency
           FROM txn t JOIN account a ON a.id = t.account_id
          WHERE t.amount < 0 AND t.transfer_id IS NULL AND t.status <> 'planned' AND t.currency = ?1
            AND t.kind NOT IN ('investment', 'card_payment') AND a.kind <> 'credit_card'
            AND t.date BETWEEN date(?2, ?3) AND date(?2, ?4)
            AND NOT EXISTS (SELECT 1 FROM loan_installment li WHERE li.txn_id = t.id)
            AND NOT EXISTS (SELECT 1 FROM loan_extra_payment e WHERE e.txn_id = t.id)
          ORDER BY abs(-t.amount - ?5), abs(julianday(t.date) - julianday(?2)), t.date, t.id
          LIMIT ?6",
    )?;
    let rows = stmt
        .query_map(
            params![
                t.currency,
                t.due_date,
                format!("-{CANDIDATE_DAYS} days"),
                format!("+{CANDIDATE_DAYS} days"),
                t.total,
                CANDIDATE_LIMIT
            ],
            |r| {
                let account_id: Id = r.get(3)?;
                Ok(LoanPaymentCandidate {
                    txn_id: r.get(0)?,
                    date: r.get(1)?,
                    description: r.get(2)?,
                    in_debit_account: t.debit_account_id.as_deref() == Some(account_id.as_str()),
                    account_id,
                    account_name: r.get(4)?,
                    amount: r.get(5)?,
                    currency: r.get(6)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Liga a parcela ao lançamento que a pagou: a parcela fica paga na data e pelo valor do
/// lançamento (uma ligação anterior da parcela é trocada). Lançamento sem categoria recebe
/// a dos pagamentos do contrato, quando ela serve para ele.
pub fn link_payment(tx: &Transaction<'_>, installment_id: &str, txn_id: &str) -> Result<()> {
    let t = target(tx, installment_id)?;
    let crate::transactions::TxnRow {
        date,
        amount,
        currency,
        kind,
        status,
        transfer_id,
        category_id,
        ..
    } = crate::transactions::load_row(tx, txn_id)?;
    crate::transactions::operations::ensure_free(tx, txn_id)?;
    if amount >= 0 {
        return Err(Error::invalid(i18n::t(
            "nucleo-planejamento-divida-erro-pagamento-saida",
        )));
    }
    if transfer_id.is_some() {
        return Err(Error::invalid(i18n::t(
            "nucleo-planejamento-divida-erro-pagamento-transferencia",
        )));
    }
    if status == TxnStatus::Planned {
        return Err(Error::invalid(i18n::t(
            "nucleo-planejamento-divida-erro-pagamento-previsto",
        )));
    }
    if currency != t.currency {
        return Err(Error::invalid(i18n::t_args(
            "nucleo-planejamento-divida-erro-pagamento-moeda",
            &i18n::args(&[
                ("moeda", currency.clone().into()),
                ("moeda_contrato", t.currency.clone().into()),
            ]),
        )));
    }
    let other: Option<(u32, String)> = tx
        .query_row(
            "SELECT li.number, l.name FROM loan_installment li JOIN loan l ON l.id = li.loan_id
              WHERE li.txn_id = ?1 AND li.id <> ?2 ORDER BY li.number LIMIT 1",
            params![txn_id, installment_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((n, name)) = other {
        return Err(Error::conflict(i18n::t_args(
            "nucleo-planejamento-divida-erro-pagamento-ja-ligado",
            &i18n::args(&[("numero", n.to_string().into()), ("contrato", name.into())]),
        )));
    }
    let extra: Option<String> = tx
        .query_row(
            "SELECT l.name FROM loan_extra_payment e JOIN loan l ON l.id = e.loan_id
              WHERE e.txn_id = ?1 LIMIT 1",
            [txn_id],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(name) = extra {
        return Err(Error::conflict(i18n::t_args(
            "nucleo-planejamento-divida-erro-pagamento-amortizacao",
            &i18n::args(&[("contrato", name.into())]),
        )));
    }
    let now = now_utc();
    tx.execute(
        "UPDATE loan_installment SET status = 'paid', paid_date = ?2, paid_amount = ?3, txn_id = ?4, updated_at = ?5
          WHERE id = ?1",
        params![installment_id, date, -amount, txn_id, now],
    )?;
    // Sem categoria: recebe a dos pagamentos do contrato (receita/despesa sem divisão).
    if let Some(cat) = &t.category_id
        && category_id.is_none()
    {
        let split: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM txn_split WHERE txn_id = ?1)",
            [txn_id],
            |r| r.get(0),
        )?;
        if matches!(kind, TxnKind::Income | TxnKind::Expense)
            && !split
            && crate::transactions::check_category(tx, Some(cat), kind).is_ok()
        {
            tx.execute(
                "UPDATE txn SET category_id = ?2, updated_at = ?3 WHERE id = ?1 AND category_id IS NULL",
                params![txn_id, cat, now],
            )?;
        }
    }
    Ok(())
}

/// Lançamentos apagados: as parcelas pagas por eles voltam a ficar em aberto e as
/// amortizações extraordinárias perdem a ligação (continuam registradas).
pub fn forget_txns(tx: &Transaction<'_>, txn_ids: &[Id]) -> Result<()> {
    let linked: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM loan_installment WHERE txn_id IS NOT NULL)
             OR EXISTS (SELECT 1 FROM loan_extra_payment WHERE txn_id IS NOT NULL)",
        [],
        |r| r.get(0),
    )?;
    if !linked {
        return Ok(());
    }
    let now = now_utc();
    for chunk in txn_ids.chunks(500) {
        let ph = vec!["?"; chunk.len()].join(", ");
        let mut values: Vec<&str> = vec![now.as_str()];
        values.extend(chunk.iter().map(String::as_str));
        tx.execute(
            &format!(
                "UPDATE loan_installment SET status = 'planned', paid_date = NULL, paid_amount = NULL,
                        txn_id = NULL, updated_at = ?1
                  WHERE txn_id IN ({ph})"
            ),
            params_from_iter(values.iter()),
        )?;
        tx.execute(
            &format!(
                "UPDATE loan_extra_payment SET txn_id = NULL, updated_at = ?1 WHERE txn_id IN ({ph})"
            ),
            params_from_iter(values.iter()),
        )?;
    }
    Ok(())
}
