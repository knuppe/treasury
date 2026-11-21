//! Anuidade do cartão (docs/design/cards.md, seção 5.2): a regra combinada com o banco, a
//! isenção, a conferência das cobranças e o lembrete antes da renovação.
//!
//! Cobrança de anuidade = lançamento do cartão com "anuidade" na descrição (o banco imprime
//! assim); estorno de anuidade abate. Isenção por gasto: a parcela cobrada numa fatura é isenta
//! quando as compras dessa mesma fatura chegam ao valor combinado. Isenção por investimento: o
//! saldo das contas da mesma instituição chega ao valor.

use std::str::FromStr;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::{Id, new_id};
use crate::time::{add_months, clamp_day, fmt_date, now_utc, parse_date};
use crate::transactions::util::clean_opt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum WaiverKind {
    /// Sem isenção.
    None,
    /// Isenta sempre.
    Full,
    /// Isenta a parcela da fatura cujas compras chegam ao valor.
    Spend,
    /// Isenta quando o saldo na instituição chega ao valor.
    Investment,
    /// Desconto fixo (`waiver_discount`, 0.5 = 50%).
    Discount,
}

impl WaiverKind {
    fn as_str(self) -> &'static str {
        match self {
            WaiverKind::None => "none",
            WaiverKind::Full => "full",
            WaiverKind::Spend => "spend",
            WaiverKind::Investment => "investment",
            WaiverKind::Discount => "discount",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "full" => WaiverKind::Full,
            "spend" => WaiverKind::Spend,
            "investment" => WaiverKind::Investment,
            "discount" => WaiverKind::Discount,
            _ => WaiverKind::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualFee {
    pub id: Id,
    pub account_id: Id,
    /// Valor cheio do ano.
    #[ts(type = "number")]
    pub amount: i64,
    pub installments: u32,
    pub anniversary_month: Option<u32>,
    pub waiver_kind: WaiverKind,
    #[ts(type = "number | null")]
    pub waiver_threshold: Option<i64>,
    /// Decimal em texto (`"0.5"` = 50%).
    pub waiver_discount: Option<String>,
    pub starts_on: String,
    pub ends_on: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualFeeInput {
    pub account_id: Id,
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(optional)]
    pub installments: Option<u32>,
    #[ts(optional)]
    pub anniversary_month: Option<u32>,
    #[ts(optional)]
    pub waiver_kind: Option<WaiverKind>,
    #[ts(optional, type = "number")]
    pub waiver_threshold: Option<i64>,
    #[ts(optional)]
    pub waiver_discount: Option<String>,
    /// Padrão: hoje.
    #[ts(optional)]
    pub starts_on: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FeeIssue {
    /// Cobrada numa fatura que tinha atingido a isenção (ou com isenção total).
    ChargedWhileWaived,
    /// Valor diferente da parcela combinada.
    AmountDiffers,
    /// Mais parcelas em 12 meses do que o combinado.
    ExtraInstallment,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeeCharge {
    pub txn_id: Id,
    pub date: String,
    pub description: String,
    /// Positivo = cobrado; negativo = estorno.
    #[ts(type = "number")]
    pub amount: i64,
    pub statement_id: Option<Id>,
    /// Parcela esperada nessa fatura (0 quando isenta).
    #[ts(type = "number | null")]
    pub expected: Option<i64>,
    pub issue: Option<FeeIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WaiverProgress {
    pub kind: WaiverKind,
    #[ts(type = "number | null")]
    pub threshold: Option<i64>,
    /// Gasto da fatura em aberto (isenção por gasto) ou saldo na instituição (por investimento).
    #[ts(type = "number")]
    pub current: i64,
    /// Quanto falta para isentar (0 = isenta).
    #[ts(type = "number")]
    pub remaining: i64,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualFeeStatus {
    pub rule: AnnualFee,
    /// Parcela cheia (valor ÷ parcelas).
    #[ts(type = "number")]
    pub installment_amount: i64,
    pub waiver: WaiverProgress,
    /// Pago de anuidade nos últimos 12 meses (cobranças − estornos).
    #[ts(type = "number")]
    pub paid_12m: i64,
    pub charges: Vec<FeeCharge>,
    /// Próxima renovação (1º dia do mês de aniversário), se o mês foi informado.
    pub next_renewal: Option<String>,
    pub issues: u32,
}

fn fee_ref(id: &str) -> String {
    crate::i18n::t_args(
        "nucleo-contas-ref-anuidade",
        &crate::i18n::args(&[("id", id.to_string().into())]),
    )
}

const COLS: &str =
    "id, account_id, amount, installments, anniversary_month, waiver_kind, waiver_threshold,
                    waiver_discount, starts_on, ends_on, notes";

fn map(r: &rusqlite::Row<'_>) -> rusqlite::Result<AnnualFee> {
    Ok(AnnualFee {
        id: r.get(0)?,
        account_id: r.get(1)?,
        amount: r.get(2)?,
        installments: r.get(3)?,
        anniversary_month: r.get(4)?,
        waiver_kind: WaiverKind::parse(&r.get::<_, String>(5)?),
        waiver_threshold: r.get(6)?,
        waiver_discount: r.get(7)?,
        starts_on: r.get(8)?,
        ends_on: r.get(9)?,
        notes: r.get(10)?,
    })
}

/// Regra em vigor numa data.
pub fn active(conn: &Connection, account_id: &str, date: &str) -> Result<Option<AnnualFee>> {
    Ok(conn
        .prepare_cached(&format!(
            "SELECT {COLS} FROM card_annual_fee WHERE account_id = ?1 AND starts_on <= ?2
               AND (ends_on IS NULL OR ends_on >= ?2) ORDER BY starts_on DESC LIMIT 1"
        ))?
        .query_row(params![account_id, date], map)
        .optional()?)
}

pub fn rules(conn: &Connection, account_id: &str) -> Result<Vec<AnnualFee>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLS} FROM card_annual_fee WHERE account_id = ?1 ORDER BY starts_on DESC"
    ))?;
    let rows = stmt
        .query_map([account_id], map)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn discount(rule: &AnnualFee) -> Result<Decimal> {
    let d = rule
        .waiver_discount
        .as_deref()
        .map(Decimal::from_str)
        .transpose()
        .map_err(|_| Error::invalid(t("nucleo-contas-anuidade-erro-desconto")))?
        .unwrap_or(Decimal::ZERO);
    if d < Decimal::ZERO || d > Decimal::ONE {
        return Err(Error::invalid(t("nucleo-contas-anuidade-erro-desconto")));
    }
    Ok(d)
}

/// Grava uma regra nova; a anterior em vigor termina no dia anterior (ou é trocada, se começou
/// no mesmo dia).
pub fn set(tx: &Transaction<'_>, input: &AnnualFeeInput, today: NaiveDate) -> Result<Id> {
    super::require_card(tx, &input.account_id)?;
    if input.amount < 0 {
        return Err(Error::invalid(t("nucleo-contas-anuidade-erro-valor")));
    }
    let installments = input.installments.unwrap_or(12);
    if !(1..=12).contains(&installments) {
        return Err(Error::invalid(t("nucleo-contas-anuidade-erro-parcelas")));
    }
    if input
        .anniversary_month
        .is_some_and(|m| !(1..=12).contains(&m))
    {
        return Err(Error::invalid(t("nucleo-contas-anuidade-erro-mes")));
    }
    let kind = input.waiver_kind.unwrap_or(WaiverKind::None);
    let threshold = input.waiver_threshold;
    if matches!(kind, WaiverKind::Spend | WaiverKind::Investment)
        && threshold.is_none_or(|v| v <= 0)
    {
        return Err(Error::invalid(t("nucleo-contas-anuidade-erro-isencao")));
    }
    let starts = input.starts_on.clone().unwrap_or_else(|| fmt_date(today));
    let start_date = parse_date(&starts)?;
    let rule = AnnualFee {
        id: new_id(),
        account_id: input.account_id.clone(),
        amount: input.amount,
        installments,
        anniversary_month: input.anniversary_month,
        waiver_kind: kind,
        waiver_threshold: if matches!(kind, WaiverKind::Spend | WaiverKind::Investment) {
            threshold
        } else {
            None
        },
        waiver_discount: if kind == WaiverKind::Discount {
            clean_opt(input.waiver_discount.clone())
        } else {
            None
        },
        starts_on: starts.clone(),
        ends_on: None,
        notes: clean_opt(input.notes.clone()),
    };
    if kind == WaiverKind::Discount {
        if rule.waiver_discount.is_none() {
            return Err(Error::invalid(t("nucleo-contas-anuidade-erro-desconto")));
        }
        discount(&rule)?;
    }
    let now = now_utc();
    tx.execute(
        "DELETE FROM card_annual_fee WHERE account_id = ?1 AND starts_on = ?2",
        params![input.account_id, starts],
    )?;
    let day_before = fmt_date(start_date.pred_opt().expect("data válida"));
    tx.execute(
        "UPDATE card_annual_fee SET ends_on = ?3, updated_at = ?4
          WHERE account_id = ?1 AND starts_on < ?2 AND (ends_on IS NULL OR ends_on >= ?2)",
        params![input.account_id, starts, day_before, now],
    )?;
    tx.execute(
        &format!(
            "INSERT INTO card_annual_fee ({COLS}, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, ?11, ?11)"
        ),
        params![
            rule.id,
            rule.account_id,
            rule.amount,
            rule.installments,
            rule.anniversary_month,
            kind.as_str(),
            rule.waiver_threshold,
            rule.waiver_discount,
            rule.starts_on,
            rule.notes,
            now
        ],
    )?;
    Ok(rule.id)
}

/// Encerra a anuidade do cartão (cartão sem anuidade a partir de hoje).
pub fn end(tx: &Transaction<'_>, account_id: &str, today: NaiveDate) -> Result<usize> {
    let today_s = fmt_date(today);
    // Regra que ainda nem começou sai inteira.
    tx.execute(
        "DELETE FROM card_annual_fee WHERE account_id = ?1 AND starts_on > ?2",
        params![account_id, today_s],
    )?;
    let n = tx.execute(
        "UPDATE card_annual_fee SET ends_on = ?2, updated_at = ?3
          WHERE account_id = ?1 AND (ends_on IS NULL OR ends_on > ?2)",
        params![account_id, today_s, now_utc()],
    )?;
    if n == 0
        && !tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM card_annual_fee WHERE account_id = ?1)",
            [account_id],
            |r| r.get::<_, bool>(0),
        )?
    {
        return Err(Error::not_found(fee_ref(account_id)));
    }
    Ok(n)
}

/// Parcela cheia: valor ÷ parcelas (o centavo que sobra fica na primeira, como no parcelado).
pub fn installment_amount(rule: &AnnualFee) -> i64 {
    rule.amount / i64::from(rule.installments.max(1))
}

/// Parcela esperada numa fatura com `spend` em compras e `invested` na instituição.
fn expected_for(rule: &AnnualFee, spend: i64, invested: i64) -> Result<i64> {
    let full = installment_amount(rule);
    Ok(match rule.waiver_kind {
        WaiverKind::None => full,
        WaiverKind::Full => 0,
        WaiverKind::Spend => {
            if spend >= rule.waiver_threshold.unwrap_or(i64::MAX) {
                0
            } else {
                full
            }
        }
        WaiverKind::Investment => {
            if invested >= rule.waiver_threshold.unwrap_or(i64::MAX) {
                0
            } else {
                full
            }
        }
        WaiverKind::Discount => {
            let d = Decimal::ONE - discount(rule)?;
            (Decimal::from(full) * d).round().try_into().unwrap_or(full)
        }
    })
}

fn is_fee(desc: &str) -> bool {
    crate::transactions::normalize_description(desc).contains("anuidade")
}

/// Compras de uma fatura (sem anuidade, encargos de pagamento e pagamentos).
fn statement_spend(conn: &Connection, statement_id: &str) -> Result<i64> {
    let mut stmt = conn.prepare_cached(
        "SELECT amount, description FROM txn WHERE statement_id = ?1 AND kind = 'expense'",
    )?;
    let mut total = 0;
    let mut q = stmt.query([statement_id])?;
    while let Some(r) = q.next()? {
        let (amount, desc): (i64, String) = (r.get(0)?, r.get(1)?);
        if !is_fee(&desc) {
            total -= amount;
        }
    }
    Ok(total)
}

/// Saldo das outras contas da mesma instituição do cartão, na moeda dele.
fn institution_balance(conn: &Connection, account_id: &str, today: NaiveDate) -> Result<i64> {
    let inst: Option<String> = conn
        .query_row(
            "SELECT institution_id FROM account WHERE id = ?1",
            [account_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    let Some(inst) = inst else { return Ok(0) };
    let (currency, _) = crate::accounts::currency_and_kind(conn, account_id)?;
    let views = crate::accounts::list(conn, false, today)?;
    Ok(views
        .iter()
        .filter(|v| {
            v.account.institution_id.as_deref() == Some(inst.as_str())
                && v.account.id != account_id
                && v.account.kind != crate::accounts::AccountKind::CreditCard
                && v.account.currency == currency
        })
        .map(|v| v.balance.max(0))
        .sum())
}

/// 1º dia do próximo mês de aniversário (a partir de hoje).
pub fn next_renewal(rule: &AnnualFee, today: NaiveDate) -> Option<NaiveDate> {
    let m = rule.anniversary_month?;
    let this = clamp_day(today.year(), m, 1);
    Some(if this > today {
        this
    } else {
        add_months(this, 12)
    })
}

/// Situação da anuidade: regra, isenção da fatura em aberto, cobranças de 12 meses conferidas.
pub fn status(
    conn: &Connection,
    account_id: &str,
    today: NaiveDate,
) -> Result<Option<AnnualFeeStatus>> {
    let today_s = fmt_date(today);
    let Some(rule) = active(conn, account_id, &today_s)? else {
        return Ok(None);
    };
    let invested = if rule.waiver_kind == WaiverKind::Investment {
        institution_balance(conn, account_id, today)?
    } else {
        0
    };
    // Isenção da fatura em aberto.
    let open: Option<String> = conn
        .query_row(
            "SELECT id FROM card_statement WHERE account_id = ?1 AND period_start <= ?2 AND closing_date >= ?2",
            params![account_id, today_s],
            |r| r.get(0),
        )
        .optional()?;
    let open_spend = match &open {
        Some(id) => statement_spend(conn, id)?,
        None => 0,
    };
    let current = match rule.waiver_kind {
        WaiverKind::Spend => open_spend,
        WaiverKind::Investment => invested,
        _ => 0,
    };
    let met =
        expected_for(&rule, open_spend, invested)? == 0 && rule.waiver_kind != WaiverKind::None;
    let remaining = match (rule.waiver_kind, rule.waiver_threshold) {
        (WaiverKind::Spend | WaiverKind::Investment, Some(th)) => (th - current).max(0),
        _ => 0,
    };
    // Cobranças dos últimos 12 meses.
    let from = fmt_date(add_months(today, -12));
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, description, amount, statement_id FROM txn
          WHERE account_id = ?1 AND date > ?2 AND date <= ?3 AND kind IN ('expense', 'adjustment')
          ORDER BY date, id",
    )?;
    let raw: Vec<(Id, String, String, i64, Option<Id>)> = stmt
        .query_map(params![account_id, from, today_s], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let full = installment_amount(&rule);
    let mut charges: Vec<FeeCharge> = vec![];
    let mut paid = 0;
    for (id, date, desc, amount, statement_id) in raw {
        if !is_fee(&desc) {
            continue;
        }
        let charged = -amount;
        paid += charged;
        let mut c = FeeCharge {
            txn_id: id,
            date: date.clone(),
            description: desc,
            amount: charged,
            statement_id: statement_id.clone(),
            expected: None,
            issue: None,
        };
        // Só as cobranças da regra em vigor são conferidas.
        if charged > 0 && date >= rule.starts_on {
            let spend = match &statement_id {
                Some(s) => statement_spend(conn, s)?,
                None => 0,
            };
            let expected = expected_for(&rule, spend, invested)?;
            c.expected = Some(expected);
            let count_12m = charges
                .iter()
                .filter(|x| x.amount > 0 && x.date >= rule.starts_on)
                .count() as u32
                + 1;
            c.issue = if expected == 0 {
                Some(FeeIssue::ChargedWhileWaived)
            } else if count_12m > rule.installments {
                Some(FeeIssue::ExtraInstallment)
            } else if (charged - expected).abs() > 1
                && charged != rule.amount - full * i64::from(rule.installments - 1)
            {
                Some(FeeIssue::AmountDiffers)
            } else {
                None
            };
        }
        charges.push(c);
    }
    let issues = charges.iter().filter(|c| c.issue.is_some()).count() as u32;
    Ok(Some(AnnualFeeStatus {
        installment_amount: full,
        waiver: WaiverProgress {
            kind: rule.waiver_kind,
            threshold: rule.waiver_threshold,
            current,
            remaining,
            met,
        },
        paid_12m: paid,
        charges,
        next_renewal: next_renewal(&rule, today).map(fmt_date),
        issues,
        rule,
    }))
}
