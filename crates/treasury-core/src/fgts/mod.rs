//! Contas vinculadas do FGTS (docs/brazil/fgts.md): uma conta do tipo `fgts` por vínculo de
//! emprego, com o extrato da Caixa importado ([`statement`]), a conferência do JAM e dos
//! depósitos com o holerite, saques (inclusive o uso na moradia), o saque-aniversário, a
//! projeção do saldo e a comparação do rendimento com IPCA, poupança, CDI e M2 ([`compare`]).
//! Só no Brasil: fora dele as ações recusam com uma mensagem clara.
//!
//! Os lançamentos são lançamentos comuns na conta FGTS (saldo, histórico e desfazer já
//! funcionam), do tipo ajuste (fora do orçamento e do "sem categoria"); o saque com conta de
//! destino é uma transferência. A classificação de cada um fica em `fgts_entry`.

pub mod api;
pub mod compare;
pub mod data;
pub mod statement;
#[cfg(test)]
mod tests;

use chrono::{Datelike, Months, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, now_utc, parse_date};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "FgtsEntryKind")]
pub enum EntryKind {
    /// Depósito mensal do empregador (115).
    Deposit,
    /// Depósito complementar ou diferença da mesma competência (650).
    DepositExtra,
    Jam,
    /// Regularização do JAM e o JAM de saque cancelado.
    JamAdjustment,
    /// Distribuição de resultado do ano-base.
    Distribution,
    Withdrawal,
    /// Reposição de um saque desfeito.
    Reversal,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntryKind::Deposit => "deposit",
            EntryKind::DepositExtra => "deposit_extra",
            EntryKind::Jam => "jam",
            EntryKind::JamAdjustment => "jam_adjustment",
            EntryKind::Distribution => "distribution",
            EntryKind::Withdrawal => "withdrawal",
            EntryKind::Reversal => "reversal",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "deposit" => EntryKind::Deposit,
            "deposit_extra" => EntryKind::DepositExtra,
            "jam" => EntryKind::Jam,
            "jam_adjustment" => EntryKind::JamAdjustment,
            "distribution" => EntryKind::Distribution,
            "withdrawal" => EntryKind::Withdrawal,
            "reversal" => EntryKind::Reversal,
            _ => return None,
        })
    }
    fn label_key(self) -> &'static str {
        match self {
            EntryKind::Deposit => "nucleo-fgts-tipo-deposito",
            EntryKind::DepositExtra => "nucleo-fgts-tipo-deposito-extra",
            EntryKind::Jam => "nucleo-fgts-tipo-jam",
            EntryKind::JamAdjustment => "nucleo-fgts-tipo-ajuste-jam",
            EntryKind::Distribution => "nucleo-fgts-tipo-distribuicao",
            EntryKind::Withdrawal => "nucleo-fgts-tipo-saque",
            EntryKind::Reversal => "nucleo-fgts-tipo-reposicao",
        }
    }
}

/// O FGTS só existe no Brasil.
pub fn require_brazil(conn: &Connection) -> Result<()> {
    if crate::country::is_brazil(conn) {
        Ok(())
    } else {
        Err(Error::invalid(t("nucleo-contas-erro-tipo-so-brasil")))
    }
}

/// A linha de `fgts_account` de uma conta nova do tipo `fgts`.
pub fn ensure_account(tx: &Transaction<'_>, account_id: &str) -> Result<()> {
    let now = now_utc();
    tx.execute(
        "INSERT INTO fgts_account (account_id, created_at, updated_at) VALUES (?1, ?2, ?2)
         ON CONFLICT (account_id) DO NOTHING",
        params![account_id, now],
    )?;
    Ok(())
}

fn require_fgts(conn: &Connection, account_id: &str) -> Result<()> {
    let (_, kind) = crate::accounts::currency_and_kind(conn, account_id)?;
    if kind == crate::accounts::AccountKind::Fgts {
        Ok(())
    } else {
        Err(Error::invalid(t("nucleo-fgts-erro-nao-e-fgts")))
    }
}

/// Classificação de um lançamento novo.
#[derive(Debug, Clone)]
pub struct NewEntry {
    pub kind: EntryKind,
    pub competence: Option<String>,
    pub coefficient: Option<String>,
    pub base_year: Option<i32>,
    pub withdrawal_code: Option<String>,
    pub withdrawal_reason: Option<String>,
    pub withdrawal_ref: Option<String>,
    pub statement_total: Option<i64>,
    pub statement_seq: Option<u32>,
}

impl NewEntry {
    pub fn of(kind: EntryKind) -> Self {
        Self {
            kind,
            competence: None,
            coefficient: None,
            base_year: None,
            withdrawal_code: None,
            withdrawal_reason: None,
            withdrawal_ref: None,
            statement_total: None,
            statement_seq: None,
        }
    }
}

/// Grava um lançamento na conta FGTS: ajuste (fora do orçamento) ou, num saque com conta de
/// destino, a transferência. Devolve o lançamento da conta FGTS.
#[allow(clippy::too_many_arguments)]
pub fn insert_entry(
    tx: &Transaction<'_>,
    account_id: &str,
    date: &str,
    amount: i64,
    description: &str,
    e: &NewEntry,
    destination: Option<&str>,
    today: NaiveDate,
) -> Result<Id> {
    let txn_id = match destination {
        Some(dest) if amount < 0 => {
            crate::transactions::create_transfer(
                tx,
                &crate::transactions::TransferInput {
                    from_account_id: account_id.to_string(),
                    to_account_id: dest.to_string(),
                    date: date.to_string(),
                    amount: -amount,
                    to_amount: None,
                    description: Some(description.to_string()),
                    kind: None,
                    status: None,
                    member_id: None,
                },
                today,
            )?
            .from_id
        }
        _ => crate::transactions::insert(
            tx,
            &crate::transactions::NewTxn::new(
                account_id,
                date,
                amount,
                "BRL",
                crate::transactions::TxnKind::Adjustment,
                description,
            ),
        )?,
    };
    tx.execute(
        "INSERT INTO fgts_entry (txn_id, account_id, kind, competence, coefficient, base_year,
                                 withdrawal_code, withdrawal_reason, withdrawal_ref,
                                 statement_total, statement_seq, computed, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12)",
        params![
            txn_id,
            account_id,
            e.kind.as_str(),
            e.competence,
            e.coefficient,
            e.base_year,
            e.withdrawal_code,
            e.withdrawal_reason,
            e.withdrawal_ref,
            e.statement_total,
            e.statement_seq,
            now_utc()
        ],
    )?;
    Ok(txn_id)
}

// ---------------------------------------------------------------------------
// Contas e lançamentos

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsAccountView")]
pub struct AccountView {
    pub account_id: Id,
    pub name: String,
    pub member_id: Option<Id>,
    #[ts(type = "number")]
    pub balance: i64,
    pub employer_name: Option<String>,
    pub employer_cnpj: Option<String>,
    pub admission_date: Option<String>,
    pub option_date: Option<String>,
    pub leave_date: Option<String>,
    pub leave_code: Option<String>,
    pub category: Option<String>,
    pub interest_rate: String,
    /// Conta ativa: vínculo sem data de afastamento.
    pub active: bool,
    #[ts(type = "number | null")]
    pub rescission_base: Option<i64>,
    pub rescission_base_date: Option<String>,
    /// Multa de 40% numa demissão sem justa causa, pelo valor para fins rescisórios.
    #[ts(type = "number | null")]
    pub dismissal_fine: Option<i64>,
    pub last_statement_date: Option<String>,
    pub income_source_id: Option<Id>,
    /// Modalidade de saque registrada do titular.
    pub modality: Option<String>,
    pub archived: bool,
}

pub fn list(conn: &Connection, today: NaiveDate) -> Result<Vec<AccountView>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.archived, f.employer_name, f.employer_cnpj, f.admission_date,
                f.option_date, f.leave_date, f.leave_code, f.category, f.interest_rate,
                f.rescission_base, f.rescission_base_date, f.last_statement_date, f.income_source_id,
                (SELECT CASE WHEN count(*) = 1 THEN min(am.member_id) END FROM account_member am
                  WHERE am.account_id = a.id)
           FROM account a JOIN fgts_account f ON f.account_id = a.id
          WHERE a.kind = 'fgts' ORDER BY a.archived, a.sort_order, a.name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
                [
                    r.get::<_, Option<String>>(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                ],
                r.get::<_, String>(10)?,
                r.get::<_, Option<i64>>(11)?,
                r.get::<_, Option<String>>(12)?,
                r.get::<_, Option<String>>(13)?,
                r.get::<_, Option<String>>(14)?,
                r.get::<_, Option<String>>(15)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let day = fmt_date(today);
    rows.into_iter()
        .map(
            |(id, name, archived, f, rate, rb, rbd, last, income, member)| {
                let [
                    employer,
                    cnpj,
                    admission,
                    option,
                    leave,
                    leave_code,
                    category,
                ] = f;
                let modality = match &member {
                    Some(m) => conn
                        .query_row(
                            "SELECT modality FROM fgts_modality WHERE member_id = ?1",
                            [m],
                            |r| r.get(0),
                        )
                        .optional()?,
                    None => None,
                };
                Ok(AccountView {
                    balance: crate::accounts::balance(conn, &id, &day, false)?,
                    account_id: id,
                    name,
                    member_id: member,
                    active: leave.is_none(),
                    employer_name: employer,
                    employer_cnpj: cnpj,
                    admission_date: admission,
                    option_date: option,
                    leave_date: leave,
                    leave_code,
                    category,
                    interest_rate: rate,
                    rescission_base: rb,
                    rescission_base_date: rbd,
                    dismissal_fine: rb.map(|v| v * 40 / 100),
                    last_statement_date: last,
                    income_source_id: income,
                    modality,
                    archived,
                })
            },
        )
        .collect()
}

fn view_of(conn: &Connection, account_id: &str, today: NaiveDate) -> Result<AccountView> {
    list(conn, today)?
        .into_iter()
        .find(|a| a.account_id == account_id)
        .ok_or_else(|| Error::invalid(t("nucleo-fgts-erro-nao-e-fgts")))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsEntryView")]
pub struct EntryView {
    pub txn_id: Id,
    pub date: String,
    pub description: String,
    pub kind: EntryKind,
    pub kind_label: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// Saldo corrido pela data.
    #[ts(type = "number")]
    pub balance: i64,
    #[ts(type = "number | null")]
    pub statement_total: Option<i64>,
    pub competence: Option<String>,
    pub coefficient: Option<String>,
    pub base_year: Option<i32>,
    pub withdrawal_code: Option<String>,
    pub withdrawal_reason: Option<String>,
    pub withdrawal_ref: Option<String>,
    /// Conta que recebeu o saque (transferência).
    pub destination_account_id: Option<Id>,
    pub computed: bool,
}

/// Lançamentos classificados da conta, em ordem de data (e de linha do extrato).
pub fn entries(conn: &Connection, account_id: &str) -> Result<Vec<EntryView>> {
    require_fgts(conn, account_id)?;
    let opening: i64 = conn.query_row(
        "SELECT opening_balance FROM account WHERE id = ?1",
        [account_id],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.date, t.description, e.kind, t.amount, e.statement_total, e.competence,
                e.coefficient, e.base_year, e.withdrawal_code, e.withdrawal_reason,
                e.withdrawal_ref, e.computed,
                (SELECT o.account_id FROM txn o WHERE o.transfer_id = t.transfer_id AND o.id <> t.id)
           FROM fgts_entry e JOIN txn t ON t.id = e.txn_id
          WHERE e.account_id = ?1
          ORDER BY t.date, COALESCE(e.statement_seq, 0), t.created_at",
    )?;
    let mut bal = opening;
    let rows = stmt
        .query_map([account_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, Option<i32>>(8)?,
                r.get::<_, Option<String>>(9)?,
                r.get::<_, Option<String>>(10)?,
                r.get::<_, Option<String>>(11)?,
                r.get::<_, bool>(12)?,
                r.get::<_, Option<String>>(13)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .map(|r| {
            bal += r.4;
            let kind = EntryKind::parse(&r.3).unwrap_or(EntryKind::JamAdjustment);
            EntryView {
                txn_id: r.0,
                date: r.1,
                description: r.2,
                kind,
                kind_label: t(kind.label_key()),
                amount: r.4,
                balance: bal,
                statement_total: r.5,
                competence: r.6,
                coefficient: r.7,
                base_year: r.8,
                withdrawal_code: r.9,
                withdrawal_reason: r.10,
                withdrawal_ref: r.11,
                computed: r.12,
                destination_account_id: r.13,
            }
        })
        .collect())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsAccountPatch")]
pub struct AccountPatch {
    pub account_id: Id,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub employer_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub employer_cnpj: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub admission_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub option_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub leave_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub leave_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub income_source_id: Option<Option<Id>>,
}

pub fn update_account(tx: &Transaction<'_>, p: &AccountPatch) -> Result<()> {
    require_fgts(tx, &p.account_id)?;
    for d in [&p.admission_date, &p.option_date, &p.leave_date]
        .into_iter()
        .flatten()
        .flatten()
    {
        parse_date(d)?;
    }
    if let Some(Some(src)) = &p.income_source_id {
        let ok: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM income_source WHERE id = ?1)",
            [src],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(Error::not_found(format!(
                "{} {src}",
                t("nucleo-fgts-erro-renda")
            )));
        }
    }
    let set = |col: &str, v: &Option<Option<String>>| -> Result<()> {
        if let Some(v) = v {
            let v = v
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            tx.execute(
                &format!(
                    "UPDATE fgts_account SET {col} = ?2, updated_at = ?3 WHERE account_id = ?1"
                ),
                params![p.account_id, v, now_utc()],
            )?;
        }
        Ok(())
    };
    set("employer_name", &p.employer_name)?;
    set("employer_cnpj", &p.employer_cnpj)?;
    set("admission_date", &p.admission_date)?;
    set("option_date", &p.option_date)?;
    set("leave_date", &p.leave_date)?;
    set("leave_code", &p.leave_code)?;
    set("income_source_id", &p.income_source_id)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Saques e moradia

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsWithdrawalInput")]
pub struct WithdrawalInput {
    pub account_id: Id,
    pub date: String,
    /// Valor sacado (positivo).
    #[ts(type = "number")]
    pub amount: i64,
    #[serde(default)]
    #[ts(optional)]
    pub code: Option<String>,
    /// rescisao, aposentadoria, moradia, aniversario, doenca, emergencial, outros.
    #[serde(default)]
    #[ts(optional)]
    pub reason: Option<String>,
    /// Conta que recebeu (vira transferência); sem ela, só sai da conta FGTS.
    #[serde(default)]
    #[ts(optional)]
    pub destination_account_id: Option<Id>,
}

fn check_reason(r: Option<&str>) -> Result<()> {
    match r {
        Some(r) if !data::REASONS.contains(&r) => Err(Error::invalid(t_args(
            "nucleo-fgts-erro-motivo",
            &args(&[("motivo", r.to_string().into())]),
        ))),
        _ => Ok(()),
    }
}

pub fn record_withdrawal(
    tx: &Transaction<'_>,
    w: &WithdrawalInput,
    today: NaiveDate,
) -> Result<Id> {
    require_brazil(tx)?;
    require_fgts(tx, &w.account_id)?;
    parse_date(&w.date)?;
    if w.amount <= 0 {
        return Err(Error::invalid(t("nucleo-fgts-erro-valor-positivo")));
    }
    let balance = crate::accounts::balance(tx, &w.account_id, &w.date, false)?;
    if w.amount > balance {
        return Err(Error::invalid(t("nucleo-fgts-erro-saldo-insuficiente")));
    }
    let reason = w.reason.clone().or_else(|| {
        w.code
            .as_deref()
            .and_then(data::reason_of_code)
            .map(str::to_string)
    });
    check_reason(reason.as_deref())?;
    let mut e = NewEntry::of(EntryKind::Withdrawal);
    e.withdrawal_code = w.code.clone();
    e.withdrawal_reason = reason.clone();
    let desc = t_args(
        "nucleo-fgts-lancamento-saque",
        &args(&[(
            "motivo",
            t(&format!(
                "nucleo-fgts-saque-motivo-{}",
                reason.as_deref().unwrap_or("outros")
            ))
            .into(),
        )]),
    );
    insert_entry(
        tx,
        &w.account_id,
        &w.date,
        -w.amount,
        &desc,
        &e,
        w.destination_account_id.as_deref(),
        today,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsHousingInput")]
pub struct HousingInput {
    pub account_id: Id,
    pub loan_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// `reduce_term` (padrão) ou `reduce_installment`.
    #[serde(default)]
    #[ts(optional)]
    pub mode: Option<crate::debt::PrepayMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsHousingResult")]
pub struct HousingResult {
    pub txn_id: Id,
    /// Aviso do intervalo de 2 anos entre usos para amortizar (Lei 8.036, art. 20).
    pub warnings: Vec<String>,
}

/// Amortização de contrato imobiliário com o saldo FGTS: a saída fica na conta FGTS (saque com
/// motivo moradia) e o contrato recebe o pagamento extra.
pub fn use_for_housing(
    tx: &Transaction<'_>,
    h: &HousingInput,
    today: NaiveDate,
) -> Result<HousingResult> {
    require_brazil(tx)?;
    require_fgts(tx, &h.account_id)?;
    parse_date(&h.date)?;
    let (kind, name): (String, String) = tx
        .query_row(
            "SELECT kind, name FROM loan WHERE id = ?1",
            [&h.loan_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| {
            Error::not_found(format!("{} {}", t("nucleo-fgts-erro-contrato"), h.loan_id))
        })?;
    if kind != "mortgage" {
        return Err(Error::invalid(t("nucleo-fgts-erro-contrato-imobiliario")));
    }
    if h.amount <= 0 {
        return Err(Error::invalid(t("nucleo-fgts-erro-valor-positivo")));
    }
    let balance = crate::accounts::balance(tx, &h.account_id, &h.date, false)?;
    if h.amount > balance {
        return Err(Error::invalid(t("nucleo-fgts-erro-saldo-insuficiente")));
    }
    let mut warnings = vec![];
    let two_years = parse_date(&h.date)?
        .checked_sub_months(Months::new(24))
        .map(fmt_date)
        .unwrap_or_default();
    let recent: Option<String> = tx
        .query_row(
            "SELECT MAX(t.date) FROM fgts_entry e JOIN txn t ON t.id = e.txn_id
              JOIN account_member am ON am.account_id = e.account_id
             WHERE e.kind = 'withdrawal' AND e.withdrawal_reason = 'moradia' AND t.date > ?1
               AND am.member_id IN (SELECT member_id FROM account_member WHERE account_id = ?2)",
            params![two_years, h.account_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    if let Some(d) = recent {
        warnings.push(t_args(
            "nucleo-fgts-aviso-dois-anos",
            &args(&[("data", crate::transactions::util::date_text(&d).into())]),
        ));
    }
    let mut e = NewEntry::of(EntryKind::Withdrawal);
    e.withdrawal_reason = Some("moradia".into());
    let desc = t_args(
        "nucleo-planejamento-divida-lancamento-amortizacao",
        &args(&[("contrato", name.into())]),
    );
    let txn_id = insert_entry(
        tx,
        &h.account_id,
        &h.date,
        -h.amount,
        &desc,
        &e,
        None,
        today,
    )?;
    let now = now_utc();
    tx.execute(
        "INSERT INTO loan_extra_payment (id, loan_id, date, amount, mode, txn_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            crate::ids::new_id(),
            h.loan_id,
            h.date,
            h.amount,
            crate::debt::kind_str(&h.mode.unwrap_or(crate::debt::PrepayMode::ReduceTerm)),
            txn_id,
            now
        ],
    )?;
    crate::debt::regenerate(tx, &h.loan_id)?;
    Ok(HousingResult { txn_id, warnings })
}

// ---------------------------------------------------------------------------
// Modalidade e saque-aniversário

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsModalityInput")]
pub struct ModalityInput {
    pub member_id: Id,
    /// `rescisao` ou `aniversario`.
    pub modality: String,
    /// Desde quando (padrão: hoje).
    #[serde(default)]
    #[ts(optional)]
    pub since: Option<String>,
    /// Pedido de volta ao saque-rescisão.
    #[serde(default)]
    #[ts(optional)]
    pub return_requested_at: Option<String>,
    /// Empréstimo de antecipação do saque-aniversário.
    #[serde(default)]
    #[ts(optional)]
    pub anticipation_debt_id: Option<Id>,
}

pub fn set_modality(tx: &Transaction<'_>, m: &ModalityInput, today: NaiveDate) -> Result<()> {
    require_brazil(tx)?;
    if !matches!(m.modality.as_str(), "rescisao" | "aniversario") {
        return Err(Error::invalid(t("nucleo-fgts-erro-modalidade")));
    }
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM member WHERE id = ?1)",
        [&m.member_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(Error::not_found(format!(
            "{} {}",
            t("nucleo-fgts-erro-membro"),
            m.member_id
        )));
    }
    for d in [&m.since, &m.return_requested_at].into_iter().flatten() {
        parse_date(d)?;
    }
    tx.execute(
        "INSERT INTO fgts_modality (member_id, modality, since, return_requested_at,
                                    anticipation_debt_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (member_id) DO UPDATE SET modality = excluded.modality, since = excluded.since,
            return_requested_at = excluded.return_requested_at,
            anticipation_debt_id = excluded.anticipation_debt_id, updated_at = excluded.updated_at",
        params![
            m.member_id,
            m.modality,
            m.since.clone().unwrap_or_else(|| fmt_date(today)),
            m.return_requested_at,
            m.anticipation_debt_id,
            now_utc()
        ],
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsAnniversary")]
pub struct Anniversary {
    pub member_id: Option<Id>,
    /// Saldo somado das contas FGTS do membro.
    #[ts(type = "number")]
    pub total_balance: i64,
    /// Valor do saque deste ano pela tabela.
    #[ts(type = "number")]
    pub amount: i64,
    /// Alíquota (0,3 = 30%) e parcela adicional da faixa.
    pub rate: f64,
    #[ts(type = "number")]
    pub extra: i64,
    /// Janela do saque: do 1º dia do mês do aniversário ao último dia útil do 2º mês seguinte.
    pub window_start: Option<String>,
    pub window_end: Option<String>,
    /// Modalidade registrada (`null`: saque-rescisão, o padrão).
    pub modality: Option<String>,
    /// Numa demissão sem justa causa com o saque-aniversário: o saldo fica bloqueado e só a
    /// multa de 40% sai.
    #[ts(type = "number")]
    pub blocked_on_dismissal: i64,
    #[ts(type = "number")]
    pub fine_on_dismissal: i64,
    /// Data mais cedo para voltar ao saque-rescisão (1º dia do 25º mês depois do pedido).
    pub earliest_return: Option<String>,
    /// Saldo devedor da antecipação ligada (a volta só depois de quitá-la).
    #[ts(type = "number | null")]
    pub anticipation_balance: Option<i64>,
    pub source: String,
    pub notes: Vec<String>,
}

fn last_business_day(year: i32, month: u32) -> NaiveDate {
    let mut d = NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|d| d.checked_add_months(Months::new(1)))
        .and_then(|d| d.pred_opt())
        .expect("data");
    while !crate::finance::calendar::is_business_day(d) {
        d = d.pred_opt().expect("data");
    }
    d
}

pub fn anniversary(
    conn: &Connection,
    member_id: Option<&str>,
    today: NaiveDate,
) -> Result<Anniversary> {
    require_brazil(conn)?;
    let accounts: Vec<AccountView> = list(conn, today)?
        .into_iter()
        .filter(|a| !a.archived && (member_id.is_none() || a.member_id.as_deref() == member_id))
        .collect();
    let total: i64 = accounts.iter().map(|a| a.balance.max(0)).sum();
    let (amount, band) = data::anniversary_amount(total);
    let member = member_id
        .map(str::to_string)
        .or_else(|| accounts.iter().find_map(|a| a.member_id.clone()));
    let birth: Option<NaiveDate> = match &member {
        Some(m) => conn
            .query_row("SELECT birth_date FROM member WHERE id = ?1", [m], |r| {
                r.get::<_, Option<String>>(0)
            })
            .optional()?
            .flatten()
            .and_then(|d| parse_date(&d).ok()),
        None => None,
    };
    let (window_start, window_end) = match birth {
        Some(b) => {
            // A janela deste ano; se já passou, a do ano que vem.
            let mut y = today.year();
            loop {
                let start = NaiveDate::from_ymd_opt(y, b.month(), 1).expect("data");
                let end_month = start.checked_add_months(Months::new(2)).expect("data");
                let end = last_business_day(end_month.year(), end_month.month());
                if end >= today || y > today.year() {
                    break (Some(fmt_date(start)), Some(fmt_date(end)));
                }
                y += 1;
            }
        }
        None => (None, None),
    };
    let (modality, ret, debt): (Option<String>, Option<String>, Option<String>) = match &member {
        Some(m) => conn
            .query_row(
                "SELECT modality, return_requested_at, anticipation_debt_id FROM fgts_modality
                  WHERE member_id = ?1",
                [m],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?
            .map_or((None, None, None), |(a, b, c)| (Some(a), b, c)),
        None => (None, None, None),
    };
    let earliest_return = if modality.as_deref() == Some("aniversario") {
        let from = ret
            .as_deref()
            .and_then(|d| parse_date(d).ok())
            .unwrap_or(today);
        NaiveDate::from_ymd_opt(from.year(), from.month(), 1)
            .and_then(|d| d.checked_add_months(Months::new(25)))
            .map(fmt_date)
    } else {
        None
    };
    let anticipation_balance = match &debt {
        Some(id) => crate::debt::view(conn, id).ok().map(|l| l.balance),
        None => None,
    };
    let fine: i64 = accounts
        .iter()
        .filter(|a| a.active)
        .filter_map(|a| a.dismissal_fine)
        .sum();
    let mut notes = vec![t("nucleo-fgts-nota-aniversario")];
    if anticipation_balance.is_some() {
        notes.push(t("nucleo-fgts-nota-antecipacao"));
    }
    Ok(Anniversary {
        member_id: member,
        total_balance: total,
        amount,
        rate: band.rate,
        extra: band.extra,
        window_start,
        window_end,
        modality,
        blocked_on_dismissal: total,
        fine_on_dismissal: fine,
        earliest_return,
        anticipation_balance,
        source: data::ANNIVERSARY_SOURCE.into(),
        notes,
    })
}

// ---------------------------------------------------------------------------
// Conferência

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsJamCheck")]
pub struct JamCheck {
    pub date: String,
    #[ts(type = "number")]
    pub credited: i64,
    #[ts(type = "number")]
    pub expected: i64,
    pub coefficient: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsDepositCheck")]
pub struct DepositCheck {
    /// Competência (AAAA-MM).
    pub competence: String,
    /// FGTS do holerite (`null`: sem holerite).
    #[ts(type = "number | null")]
    pub payslip: Option<i64>,
    /// Depósitos 115 + 650 da competência.
    #[ts(type = "number")]
    pub deposited: i64,
    /// Prazo do depósito (dia 20 do mês seguinte; antes de 03/2024, dia 7).
    pub due_date: String,
    /// `ok`, `missing` (passou do prazo sem depósito), `short` (depositou menos que o
    /// holerite), `pending` (dentro do prazo).
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsCheck")]
pub struct Check {
    pub account_id: Id,
    pub jam_checked: u32,
    pub jam_mismatches: Vec<JamCheck>,
    pub deposits: Vec<DepositCheck>,
    /// Competências sem depósito e com o prazo vencido.
    pub missing: Vec<String>,
    pub notes: Vec<String>,
}

pub fn check(conn: &Connection, account_id: &str, today: NaiveDate) -> Result<Check> {
    require_brazil(conn)?;
    let list = entries(conn, account_id)?;
    let view = view_of(conn, account_id, today)?;
    // JAM pela regra, com os lançamentos na ordem de data.
    let opening: i64 = conn.query_row(
        "SELECT opening_balance FROM account WHERE id = ?1",
        [account_id],
        |r| r.get(0),
    )?;
    let lines: Vec<statement::StatementLine> = list
        .iter()
        .enumerate()
        .map(|(i, e)| statement::StatementLine {
            seq: i as u32 + 1,
            date: e.date.clone(),
            history: e.description.clone(),
            kind: e.kind,
            amount: e.amount,
            total: e.balance,
            competence: e.competence.clone(),
            coefficient: e.coefficient.clone(),
            base_year: e.base_year,
            withdrawal_code: e.withdrawal_code.clone(),
            withdrawal_ref: e.withdrawal_ref.clone(),
            group: None,
            exists: true,
            jam_expected: None,
        })
        .collect();
    let expected = statement::expected_jam(&lines, opening);
    let mut checked = 0;
    let mut mism = vec![];
    for (l, e) in lines.iter().zip(expected) {
        if let Some(e) = e {
            checked += 1;
            if e != l.amount {
                mism.push(JamCheck {
                    date: l.date.clone(),
                    credited: l.amount,
                    expected: e,
                    coefficient: l.coefficient.clone().unwrap_or_default(),
                });
            }
        }
    }
    // Depósitos por competência.
    let mut by_comp: std::collections::BTreeMap<String, i64> = Default::default();
    for e in list
        .iter()
        .filter(|e| matches!(e.kind, EntryKind::Deposit | EntryKind::DepositExtra))
    {
        if let Some(c) = &e.competence {
            *by_comp.entry(c.clone()).or_default() += e.amount;
        }
    }
    // FGTS do holerite da renda ligada (ou das rendas CLT do titular com o mesmo CNPJ).
    let mut payslips: std::collections::BTreeMap<String, i64> = Default::default();
    let mut stmt = conn.prepare(
        "SELECT p.reference_month, SUM(COALESCE(p.fgts, 0)) FROM payslip p
           JOIN income_source s ON s.id = p.source_id
          WHERE p.fgts IS NOT NULL AND (s.id = ?1 OR (?1 IS NULL AND ?2 IS NOT NULL
                AND replace(replace(replace(s.employer_cnpj, '.', ''), '/', ''), '-', '')
                  = replace(replace(replace(?2, '.', ''), '/', ''), '-', '')))
          GROUP BY p.reference_month",
    )?;
    for r in stmt.query_map(params![view.income_source_id, view.employer_cnpj], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (m, v) = r?;
        payslips.insert(m, v);
    }
    // Competências da admissão (ou do primeiro depósito) até o mês passado, enquanto ativa.
    let first = view
        .admission_date
        .as_deref()
        .map(|d| d[..7].to_string())
        .or_else(|| by_comp.keys().next().cloned());
    let last = match &view.leave_date {
        Some(d) => d[..7].to_string(),
        None => {
            fmt_date(today.checked_sub_months(Months::new(1)).unwrap_or(today))[..7].to_string()
        }
    };
    let mut deposits = vec![];
    let mut missing = vec![];
    if let Some(first) = first {
        let month = |s: &str| -> Result<NaiveDate> {
            let (y, m) = crate::time::parse_month(s)?;
            Ok(NaiveDate::from_ymd_opt(y, m, 1).expect("data"))
        };
        let mut m = month(&first)?;
        let end = month(&last)?;
        while m <= end {
            let comp = fmt_date(m)[..7].to_string();
            let next = m.checked_add_months(Months::new(1)).expect("data");
            let due = NaiveDate::from_ymd_opt(
                next.year(),
                next.month(),
                data::deposit_due_day(next.year(), next.month()),
            )
            .expect("data");
            let deposited = by_comp.get(&comp).copied().unwrap_or(0);
            let slip = payslips.get(&comp).copied();
            let status = if deposited == 0 && today > due {
                "missing"
            } else if deposited == 0 {
                "pending"
            } else if slip.is_some_and(|s| deposited + 100 < s) {
                "short"
            } else {
                "ok"
            };
            if status == "missing" {
                missing.push(comp.clone());
            }
            deposits.push(DepositCheck {
                competence: comp,
                payslip: slip,
                deposited,
                due_date: fmt_date(due),
                status: status.into(),
            });
            m = next;
        }
    }
    let mut notes = vec![t("nucleo-fgts-nota-conferencia")];
    if view.last_statement_date.is_some() && !missing.is_empty() {
        notes.push(t("nucleo-fgts-nota-extrato-recente"));
    }
    Ok(Check {
        account_id: account_id.to_string(),
        jam_checked: checked,
        jam_mismatches: mism,
        deposits,
        missing,
        notes,
    })
}

// ---------------------------------------------------------------------------
// Projeção

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsProjectArgs")]
pub struct ProjectArgs {
    /// Uma conta; sem ela, todas as do membro (ou todas).
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Até a data…
    #[serde(default)]
    #[ts(optional)]
    pub date: Option<String>,
    /// …ou até a idade do titular.
    #[serde(default)]
    #[ts(optional)]
    pub age: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsProjection")]
pub struct Projection {
    pub date: String,
    #[ts(type = "number")]
    pub balance_today: i64,
    /// Saldo projetado em reais da época.
    #[ts(type = "number")]
    pub balance: i64,
    /// O mesmo em reais de hoje (descontada a inflação dos últimos 12 meses, mantida).
    #[ts(type = "number")]
    pub balance_real: i64,
    #[ts(type = "number")]
    pub deposits: i64,
    #[ts(type = "number")]
    pub jam: i64,
    #[ts(type = "number")]
    pub distribution: i64,
    /// TR ao mês e IPCA em 12 meses usados (%).
    pub tr_monthly: f64,
    pub ipca_12m: f64,
    pub notes: Vec<String>,
}

/// Último valor de uma série (por data ≤ hoje).
fn last_value(conn: &Connection, series: &str, today: NaiveDate) -> Result<Option<f64>> {
    let from = today.checked_sub_months(Months::new(6)).unwrap_or(today);
    Ok(crate::indices::points(conn, series, from, today)?
        .pop()
        .and_then(|p| p.value.to_string().parse().ok()))
}

/// Inflação acumulada dos últimos 12 meses (IPCA, % ).
fn ipca_12m(conn: &Connection, today: NaiveDate) -> Result<f64> {
    let from = today.checked_sub_months(Months::new(13)).unwrap_or(today);
    let pts = crate::indices::points(conn, "ipca", from, today)?;
    let last: Vec<f64> = pts
        .iter()
        .rev()
        .take(12)
        .filter_map(|p| p.value.to_string().parse::<f64>().ok())
        .collect();
    Ok(if last.len() == 12 {
        (last.iter().fold(1.0, |a, v| a * (1.0 + v / 100.0)) - 1.0) * 100.0
    } else {
        4.0
    })
}

pub fn project(conn: &Connection, a: &ProjectArgs, today: NaiveDate) -> Result<Projection> {
    require_brazil(conn)?;
    let accounts: Vec<AccountView> = list(conn, today)?
        .into_iter()
        .filter(|x| !x.archived)
        .filter(|x| a.account_id.as_ref().is_none_or(|id| &x.account_id == id))
        .filter(|x| a.member_id.is_none() || x.member_id == a.member_id)
        .collect();
    let target = match (&a.date, a.age) {
        (Some(d), _) => parse_date(d)?,
        (None, Some(age)) => {
            let member = a
                .member_id
                .clone()
                .or_else(|| accounts.iter().find_map(|x| x.member_id.clone()));
            let birth: Option<String> = match member {
                Some(m) => conn
                    .query_row("SELECT birth_date FROM member WHERE id = ?1", [m], |r| {
                        r.get(0)
                    })
                    .optional()?
                    .flatten(),
                None => None,
            };
            let b = birth
                .as_deref()
                .and_then(|d| parse_date(d).ok())
                .ok_or_else(|| Error::invalid(t("nucleo-fgts-erro-nascimento")))?;
            b.checked_add_months(Months::new(age * 12)).unwrap_or(b)
        }
        (None, None) => return Err(Error::invalid(t("nucleo-fgts-erro-data-ou-idade"))),
    };
    let tr = last_value(conn, "tr", today)?.unwrap_or(0.0);
    let ipca = ipca_12m(conn, today)?;
    let coef = (1.0 + tr / 100.0) * (1.0 + data::ANNUAL_INTEREST).powf(1.0 / 12.0) - 1.0;
    let ipca_m = (1.0 + ipca / 100.0).powf(1.0 / 12.0) - 1.0;
    let dist = data::estimated_distribution();
    let mut total = 0.0;
    let mut real = 0.0;
    let mut deposits = 0.0;
    let mut jam = 0.0;
    let mut distribution = 0.0;
    let mut today_total = 0;
    for acc in &accounts {
        today_total += acc.balance;
        // Depósito mensal: 8% do bruto da renda ligada, enquanto o vínculo está ativo.
        let gross: Option<i64> = match (&acc.income_source_id, acc.active) {
            (Some(src), true) => conn
                .query_row(
                    "SELECT gross_monthly FROM income_source WHERE id = ?1 AND archived = 0",
                    [src],
                    |r| r.get(0),
                )
                .optional()?
                .flatten(),
            _ => None,
        };
        let monthly_dep = gross.unwrap_or(0) as f64 * data::DEPOSIT_RATE;
        let mut bal = acc.balance as f64;
        let mut bal_real = acc.balance as f64;
        let mut deflator = 1.0;
        let (mut dec31, mut dec31_real) = (bal, bal_real);
        let mut m = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).expect("data");
        while m < target {
            let j = bal * coef;
            jam += j;
            bal += j;
            bal_real = bal_real * (1.0 + coef) / (1.0 + ipca_m);
            deflator *= 1.0 + ipca_m;
            if m.month() == 8 {
                distribution += dec31 * dist;
                bal += dec31 * dist;
                bal_real += dec31_real * dist;
            }
            // O salário acompanha a inflação: o depósito é constante em reais de hoje.
            bal += monthly_dep * deflator;
            bal_real += monthly_dep;
            deposits += monthly_dep * deflator;
            if m.month() == 12 {
                (dec31, dec31_real) = (bal, bal_real);
            }
            m = m.checked_add_months(Months::new(1)).expect("data");
        }
        total += bal;
        real += bal_real;
    }
    let mut notes = vec![t_args(
        "nucleo-fgts-nota-projecao",
        &args(&[
            (
                "tr",
                crate::ai::alerts::decimal(crate::i18n::current(), &format!("{tr:.4}")).into(),
            ),
            (
                "ipca",
                crate::ai::alerts::decimal(crate::i18n::current(), &format!("{ipca:.2}")).into(),
            ),
        ]),
    )];
    if accounts
        .iter()
        .any(|a| a.income_source_id.is_none() && a.active)
    {
        notes.push(t("nucleo-fgts-nota-sem-renda"));
    }
    Ok(Projection {
        date: fmt_date(target),
        balance_today: today_total,
        balance: total.round() as i64,
        balance_real: real.round() as i64,
        deposits: deposits.round() as i64,
        jam: jam.round() as i64,
        distribution: distribution.round() as i64,
        tr_monthly: tr,
        ipca_12m: (ipca * 100.0).round() / 100.0,
        notes,
    })
}

/// Saldo FGTS do titular projetado até a aposentadoria, em reais de hoje, para o evento de vida
/// "saque do FGTS na aposentadoria" do planejador (docs/brazil/fgts.md, seção 6.7). `None` fora do
/// Brasil ou sem conta FGTS.
pub fn retirement_withdrawal(
    conn: &Connection,
    current_age: Option<u32>,
    retire: u32,
    today: NaiveDate,
) -> Result<Option<i64>> {
    if !crate::country::is_brazil(conn) {
        return Ok(None);
    }
    let primary: Option<String> = conn
        .query_row(
            "SELECT id FROM member WHERE is_primary = 1 AND archived = 0 LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let accounts: Vec<AccountView> = list(conn, today)?
        .into_iter()
        .filter(|a| !a.archived && (a.member_id.is_none() || a.member_id == primary))
        .collect();
    if accounts.is_empty() {
        return Ok(None);
    }
    let years = retire.saturating_sub(current_age.unwrap_or(retire));
    let date = today
        .checked_add_months(Months::new(years * 12))
        .unwrap_or(today);
    let mut total = 0;
    for a in &accounts {
        let p = project(
            conn,
            &ProjectArgs {
                account_id: Some(a.account_id.clone()),
                member_id: None,
                date: Some(fmt_date(date)),
                age: None,
            },
            today,
        )?;
        total += p.balance_real;
    }
    Ok((total > 0).then_some(total))
}
