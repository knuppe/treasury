//! Contas: corrente, poupança, dinheiro, cartão (com faturas), corretora e exterior.

pub mod api;
pub mod capital;
pub mod freshness;
pub mod overdraft;
pub mod sync;

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc, parse_date};
use crate::transactions::util::{
    clean_color, clean_name, clean_opt, date_text, dedup_ids, exists, require_member,
};

/// "conta {id}" (detalhe do erro "não encontrado").
pub(crate) fn account_ref(id: &str) -> String {
    t_args(
        "nucleo-contas-ref-conta",
        &args(&[("id", id.to_string().into())]),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AccountKind {
    Checking,
    Savings,
    Cash,
    CreditCard,
    Brokerage,
    Foreign,
    /// Cartão de benefício (vale-alimentação e vale-refeição), com saldo próprio.
    Benefit,
    /// Conta vinculada do FGTS (uma por vínculo de emprego; só no Brasil). Patrimônio não
    /// líquido: fora da previsão do dia a dia e do orçamento (docs/brazil/fgts.md).
    Fgts,
    /// Conta de aposentadoria com limite anual de aportes (401(k), IRA, PPR, plan de
    /// pensiones…; só nos países com planos no catálogo; docs/international.md).
    Retirement,
    Other,
}

impl AccountKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AccountKind::Checking => "checking",
            AccountKind::Savings => "savings",
            AccountKind::Cash => "cash",
            AccountKind::CreditCard => "credit_card",
            AccountKind::Brokerage => "brokerage",
            AccountKind::Foreign => "foreign",
            AccountKind::Benefit => "benefit",
            AccountKind::Fgts => "fgts",
            AccountKind::Retirement => "retirement",
            AccountKind::Other => "other",
        }
    }
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "checking" => AccountKind::Checking,
            "savings" => AccountKind::Savings,
            "cash" => AccountKind::Cash,
            "credit_card" => AccountKind::CreditCard,
            "brokerage" => AccountKind::Brokerage,
            "foreign" => AccountKind::Foreign,
            "benefit" => AccountKind::Benefit,
            "fgts" => AccountKind::Fgts,
            "retirement" => AccountKind::Retirement,
            "other" => AccountKind::Other,
            _ => {
                return Err(Error::invalid(t_args(
                    "nucleo-contas-erro-tipo-invalido",
                    &args(&[("valor", s.to_string().into())]),
                )));
            }
        })
    }
    /// Contas cujo saldo negativo é dívida (passivo).
    pub fn is_liability(self) -> bool {
        matches!(self, AccountKind::CreditCard)
    }
    /// Tipos que só existem no Brasil (só aparecem e só se criam num cofre do Brasil).
    pub fn brazil_only(self) -> bool {
        matches!(self, AccountKind::Fgts)
    }
}

/// Dados do cartão de crédito (extensão da conta do tipo `credit_card`).
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct CardDetails {
    #[ts(type = "number | null")]
    pub credit_limit: Option<i64>,
    /// Dia do fechamento da fatura (1–31; limitado ao fim do mês).
    pub closing_day: u32,
    /// Dia do vencimento (1–31). Se ≤ fechamento, vence no mês seguinte ao fechamento.
    pub due_day: u32,
    /// Conta que paga a fatura.
    pub payment_account_id: Option<Id>,
    pub network: Option<String>,
    pub last4: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Account {
    pub id: Id,
    pub name: String,
    pub kind: AccountKind,
    pub currency: String,
    pub institution_id: Option<Id>,
    pub institution_name: Option<String>,
    /// Titulares.
    pub member_ids: Vec<Id>,
    #[ts(type = "number")]
    pub opening_balance: i64,
    pub opening_date: String,
    pub include_in_net_worth: bool,
    pub archived: bool,
    pub color: Option<String>,
    pub notes: Option<String>,
    #[ts(type = "number")]
    pub sort_order: i64,
    pub card: Option<CardDetails>,
    /// Cheque especial (limite de crédito) da conta corrente.
    pub overdraft: Option<overdraft::OverdraftTerms>,
    /// Agência e número da conta como no extrato do banco (achar a conta num OFX).
    pub branch: Option<String>,
    pub account_number: Option<String>,
    /// Plano da conta de aposentadoria (`us_401k`, `pt_ppr`…) e a cobertura (HSA).
    #[serde(default)]
    pub retirement_plan: Option<String>,
    #[serde(default)]
    pub retirement_coverage: Option<String>,
}

/// Conta com saldos calculados ("medido, nunca inventado": saldo = inicial + lançamentos).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountView {
    #[serde(flatten)]
    pub account: Account,
    /// Saldo hoje (efetivados + conciliados), na moeda da conta.
    #[ts(type = "number")]
    pub balance: i64,
    /// Saldo incluindo previstos até o fim do mês corrente.
    #[ts(type = "number")]
    pub projected_balance: i64,
    /// Saldo conciliado (só lançamentos conciliados).
    #[ts(type = "number")]
    pub reconciled_balance: i64,
    /// Saldo convertido para a moeda base da família, quando há câmbio.
    #[ts(type = "number | null")]
    pub balance_base: Option<i64>,
    /// O câmbio usado é de outra data ou estimado.
    pub fx_estimated: bool,
    /// Data do último lançamento efetivado até hoje.
    pub last_txn_date: Option<String>,
    /// Última importação de extrato ou conferência de saldo (`YYYY-MM-DD`).
    pub last_import_at: Option<String>,
    /// Dias desde a novidade mais recente (lançamento, importação ou conferência; sem
    /// nenhuma, a abertura da conta).
    #[ts(type = "number | null")]
    pub staleness_days: Option<i64>,
    /// Quanto o saldo está em dia pelos limites do tipo de conta; `null` nas arquivadas.
    pub balance_status: Option<freshness::BalanceStatus>,
    /// O mês passado (`YYYY-MM`) sem nenhum lançamento numa conta que costuma ter todo mês.
    pub missed_month: Option<String>,
    /// Cartão: limite − saldo devedor (faturas não pagas, incluindo parcelas futuras já
    /// comprometidas). Negativo = acima do limite.
    #[ts(type = "number | null")]
    pub available_credit: Option<i64>,
    /// Cheque especial: uso do limite agora e o custo estimado de 30 dias.
    pub overdraft_status: Option<overdraft::OverdraftStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountListArgs {
    /// Incluir contas arquivadas (padrão: não).
    #[ts(optional)]
    pub include_archived: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BalanceInterval {
    /// Um ponto por dia.
    #[default]
    Day,
    /// Um ponto no último dia de cada mês (o último ponto é `to`).
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BalanceHistoryArgs {
    pub id: Id,
    pub from: String,
    pub to: String,
    #[ts(optional)]
    pub interval: Option<BalanceInterval>,
    /// Incluir lançamentos previstos (projeção). Padrão: não.
    #[ts(optional)]
    pub include_planned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountInput {
    pub name: String,
    pub kind: AccountKind,
    #[ts(optional)]
    pub currency: Option<String>,
    #[ts(optional)]
    pub institution_id: Option<Id>,
    /// Sem `institution_id`: usa a instituição com este nome (sem diferenciar maiúsculas) ou
    /// cria uma nova na mesma alteração. Permite cadastrar banco e conta numa proposta só.
    #[serde(default)]
    #[ts(optional)]
    pub institution_name: Option<String>,
    #[ts(optional)]
    pub member_ids: Option<Vec<Id>>,
    #[ts(optional, type = "number")]
    pub opening_balance: Option<i64>,
    /// Padrão: hoje.
    #[ts(optional)]
    pub opening_date: Option<String>,
    #[ts(optional)]
    pub include_in_net_worth: Option<bool>,
    #[ts(optional)]
    pub color: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
    /// Obrigatório quando `kind = credit_card`.
    #[ts(optional)]
    pub card: Option<CardDetails>,
    /// Cheque especial (contas correntes).
    #[serde(default)]
    #[ts(optional)]
    pub overdraft: Option<overdraft::OverdraftTerms>,
    /// Agência (só dígitos e hífen).
    #[serde(default)]
    #[ts(optional)]
    pub branch: Option<String>,
    /// Número da conta com dígito.
    #[serde(default)]
    #[ts(optional)]
    pub account_number: Option<String>,
    /// Conta de aposentadoria: o plano do país e a cobertura (HSA: `self` ou `family`).
    #[serde(default)]
    #[ts(optional)]
    pub retirement_plan: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub retirement_coverage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub institution_id: Option<Option<Id>>,
    #[ts(optional)]
    pub member_ids: Option<Vec<Id>>,
    #[ts(optional, type = "number")]
    pub opening_balance: Option<i64>,
    #[ts(optional)]
    pub opening_date: Option<String>,
    #[ts(optional)]
    pub include_in_net_worth: Option<bool>,
    #[ts(optional)]
    pub archived: Option<bool>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub color: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
    #[ts(optional, type = "number")]
    pub sort_order: Option<i64>,
    #[ts(optional)]
    pub card: Option<CardDetails>,
    /// Cheque especial: `null` remove o limite.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub overdraft: Option<Option<overdraft::OverdraftTerms>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub branch: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub account_number: Option<Option<String>>,
    /// Troca o plano da conta de aposentadoria (e a cobertura).
    #[serde(default)]
    #[ts(optional)]
    pub retirement_plan: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub retirement_coverage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BalancePoint {
    pub date: String,
    #[ts(type = "number")]
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReconcileArgs {
    pub account_id: Id,
    /// Data do extrato.
    pub statement_date: String,
    /// Saldo informado no extrato do banco.
    #[ts(type = "number")]
    pub statement_balance: i64,
    /// Marca como conciliados os efetivados até a data se a diferença for zero.
    #[ts(optional)]
    pub apply: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReconcileResult {
    #[ts(type = "number")]
    pub computed_balance: i64,
    #[ts(type = "number")]
    pub statement_balance: i64,
    /// extrato − calculado. Zero = conciliado.
    #[ts(type = "number")]
    pub difference: i64,
    #[ts(type = "number")]
    pub marked_reconciled: i64,
    pub change_set_id: Option<Id>,
}

/// Limite de pontos de `balance_history`.
pub const MAX_HISTORY_POINTS: i64 = 3700;

/// Saldo da conta numa data: saldo inicial + soma dos lançamentos com status em `statuses`
/// e data ≤ `as_of`. Lançamentos de divisão não alteram o saldo (o total está no pai).
pub fn balance(
    conn: &Connection,
    account_id: &str,
    as_of: &str,
    include_planned: bool,
) -> Result<i64> {
    let opening: i64 = conn
        .query_row(
            "SELECT opening_balance FROM account WHERE id = ?1",
            [account_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(account_ref(account_id)))?;
    let sum: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM txn WHERE account_id = ?1 AND date <= ?2
           AND (status IN ('cleared', 'reconciled') OR (?3 AND status = 'planned'))",
        rusqlite::params![account_id, as_of, include_planned],
        |r| r.get(0),
    )?;
    Ok(opening + sum)
}

/// Moeda e tipo da conta.
pub fn currency_and_kind(conn: &Connection, account_id: &str) -> Result<(String, AccountKind)> {
    let (cur, kind): (String, String) = conn
        .query_row(
            "SELECT currency, kind FROM account WHERE id = ?1",
            [account_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(account_ref(account_id)))?;
    Ok((cur, AccountKind::parse(&kind)?))
}

pub fn card_details(conn: &Connection, account_id: &str) -> Result<Option<CardDetails>> {
    Ok(conn
        .query_row(
            "SELECT credit_limit, closing_day, due_day, payment_account_id, network, last4 FROM credit_card WHERE account_id = ?1",
            [account_id],
            |r| {
                Ok(CardDetails {
                    credit_limit: r.get(0)?,
                    closing_day: r.get(1)?,
                    due_day: r.get(2)?,
                    payment_account_id: r.get(3)?,
                    network: r.get(4)?,
                    last4: r.get(5)?,
                })
            },
        )
        .optional()?)
}

// ---------------------------------------------------------------------------
// Leitura

const ACCOUNT_SQL: &str = "SELECT a.id, a.name, a.kind, a.currency, a.institution_id, i.name, a.opening_balance,
       a.opening_date, a.include_in_net_worth, a.archived, a.color, a.notes, a.sort_order,
       c.account_id, c.credit_limit, c.closing_day, c.due_day, c.payment_account_id, c.network, c.last4,
       COALESCE(x.bal, 0), COALESCE(x.proj, 0), COALESCE(x.rec, 0), x.last_date, COALESCE(x.committed, 0),
       ra.plan, ra.coverage
  FROM account a
  LEFT JOIN institution i ON i.id = a.institution_id
  LEFT JOIN credit_card c ON c.account_id = a.id
  LEFT JOIN retirement_account ra ON ra.account_id = a.id
  LEFT JOIN (SELECT account_id,
                    SUM(CASE WHEN status IN ('cleared', 'reconciled') AND date <= ?1 THEN amount ELSE 0 END) AS bal,
                    SUM(CASE WHEN date <= ?2 THEN amount ELSE 0 END) AS proj,
                    SUM(CASE WHEN status = 'reconciled' THEN amount ELSE 0 END) AS rec,
                    MAX(CASE WHEN status <> 'planned' AND date <= ?1 THEN date END) AS last_date,
                    SUM(CASE WHEN status <> 'planned' OR installment_plan_id IS NOT NULL THEN amount ELSE 0 END) AS committed
               FROM txn {TXN_WHERE} GROUP BY account_id) x ON x.account_id = a.id";

struct Sums {
    bal: i64,
    proj: i64,
    rec: i64,
    last_date: Option<String>,
    committed: i64,
}

fn row_to_account(r: &rusqlite::Row<'_>) -> rusqlite::Result<(Account, Sums)> {
    let kind: String = r.get(2)?;
    let card = match r.get::<_, Option<String>>(13)? {
        Some(_) => Some(CardDetails {
            credit_limit: r.get(14)?,
            closing_day: r.get(15)?,
            due_day: r.get(16)?,
            payment_account_id: r.get(17)?,
            network: r.get(18)?,
            last4: r.get(19)?,
        }),
        None => None,
    };
    Ok((
        Account {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: AccountKind::parse(&kind).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
            currency: r.get(3)?,
            institution_id: r.get(4)?,
            institution_name: r.get(5)?,
            member_ids: vec![],
            opening_balance: r.get(6)?,
            opening_date: r.get(7)?,
            include_in_net_worth: r.get(8)?,
            archived: r.get(9)?,
            color: r.get(10)?,
            notes: r.get(11)?,
            sort_order: r.get(12)?,
            card,
            overdraft: None,
            branch: None,
            account_number: None,
            retirement_plan: r.get(25)?,
            retirement_coverage: r.get(26)?,
        },
        Sums {
            bal: r.get(20)?,
            proj: r.get(21)?,
            rec: r.get(22)?,
            last_date: r.get(23)?,
            committed: r.get(24)?,
        },
    ))
}

fn load_views(
    conn: &Connection,
    only: Option<&str>,
    include_archived: bool,
    today: NaiveDate,
) -> Result<Vec<AccountView>> {
    let (y, m) = (
        chrono::Datelike::year(&today),
        chrono::Datelike::month(&today),
    );
    let month_end = crate::time::month_bounds(y, m).1;
    let sql = ACCOUNT_SQL.replace(
        "{TXN_WHERE}",
        if only.is_some() {
            "WHERE account_id = ?3"
        } else {
            ""
        },
    );
    let sql = format!(
        "{sql} WHERE (?4 OR a.archived = 0) {} ORDER BY a.archived, a.sort_order, a.name COLLATE NOCASE",
        if only.is_some() {
            "AND a.id = ?3"
        } else {
            "AND ?3 IS NULL"
        }
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(
            params![fmt_date(today), fmt_date(month_end), only, include_archived],
            row_to_account,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut members: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT am.account_id, am.member_id FROM account_member am JOIN member m ON m.id = am.member_id
              ORDER BY m.sort_order, m.name",
        )?;
        let mut q = stmt.query([])?;
        while let Some(r) = q.next()? {
            members.entry(r.get(0)?).or_default().push(r.get(1)?);
        }
    }
    let base = crate::household::base_currency(conn)?;
    let today_s = fmt_date(today);
    let fresh_inputs = freshness::inputs(conn, today)?;
    let limits = freshness::limits(conn)?;
    let mut out = Vec::with_capacity(rows.len());
    for (mut account, sums) in rows {
        account.member_ids = members.remove(&account.id).unwrap_or_default();
        account.overdraft = overdraft::load(conn, &account.id)?;
        (account.branch, account.account_number) = bank_details(conn, &account.id)?;
        let balance = account.opening_balance + sums.bal;
        let converted = crate::fx::convert(conn, balance, &account.currency, &base, &today_s)?;
        let available_credit = account
            .card
            .as_ref()
            .and_then(|c| c.credit_limit)
            .map(|limit| limit + account.opening_balance + sums.committed);
        let inputs = fresh_inputs.get(&account.id);
        let (staleness_days, status, missed) = freshness::assess(
            account.kind,
            &account.opening_date,
            sums.last_date.as_deref(),
            inputs,
            &limits,
            today,
        );
        let balance_status = (!account.archived).then_some(status);
        out.push(AccountView {
            balance,
            projected_balance: account.opening_balance + sums.proj,
            reconciled_balance: account.opening_balance + sums.rec,
            balance_base: converted.as_ref().map(|c| c.amount),
            fx_estimated: converted.as_ref().is_some_and(|c| c.estimated),
            last_txn_date: sums.last_date,
            last_import_at: inputs.and_then(|i| i.last_import.clone()),
            staleness_days,
            balance_status,
            missed_month: inputs
                .and_then(|i| i.missed_month.clone())
                .filter(|_| missed),
            available_credit,
            overdraft_status: account.overdraft.as_ref().map(|t| {
                let index = overdraft::index_reading(conn, t.indexer, crate::time::today_local())
                    .ok()
                    .flatten();
                overdraft::OverdraftStatus {
                    uncertain: balance_status.is_some_and(|s| s != freshness::BalanceStatus::Fresh),
                    ..overdraft::status(t, balance, index)
                }
            }),
            account,
        });
    }
    Ok(out)
}

/// Contas com saldos calculados, na ordem de exibição.
pub fn list(
    conn: &Connection,
    include_archived: bool,
    today: NaiveDate,
) -> Result<Vec<AccountView>> {
    load_views(conn, None, include_archived, today)
}

pub fn get(conn: &Connection, id: &str, today: NaiveDate) -> Result<AccountView> {
    load_views(conn, Some(id), true, today)?
        .pop()
        .ok_or_else(|| Error::not_found(account_ref(id)))
}

/// Saldo dia a dia (ou no fim de cada mês) entre `from` e `to`.
pub fn balance_history(conn: &Connection, args: &BalanceHistoryArgs) -> Result<Vec<BalancePoint>> {
    let from = parse_date(&args.from)?;
    let to = parse_date(&args.to)?;
    if from > to {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-periodo-invertido",
        )));
    }
    let interval = args.interval.unwrap_or_default();
    if interval == BalanceInterval::Day && (to - from).num_days() + 1 > MAX_HISTORY_POINTS {
        return Err(Error::invalid(t_args(
            "nucleo-contas-erro-historico-longo",
            &crate::i18n::args(&[("max", MAX_HISTORY_POINTS.into())]),
        )));
    }
    let planned = args.include_planned.unwrap_or(false);
    let day_before = from.pred_opt().expect("data válida");
    let mut running = balance(conn, &args.id, &fmt_date(day_before), planned)?;
    let mut stmt = conn.prepare(
        "SELECT date, SUM(amount) FROM txn WHERE account_id = ?1 AND date >= ?2 AND date <= ?3
           AND (status IN ('cleared', 'reconciled') OR (?4 AND status = 'planned'))
         GROUP BY date ORDER BY date",
    )?;
    let deltas: Vec<(String, i64)> = stmt
        .query_map(params![args.id, args.from, args.to, planned], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut deltas = deltas.into_iter().peekable();
    let mut out = Vec::new();
    let mut d = from;
    while d <= to {
        let ds = fmt_date(d);
        while let Some((date, v)) = deltas.peek() {
            if *date <= ds {
                running += v;
                deltas.next();
            } else {
                break;
            }
        }
        let is_point = match interval {
            BalanceInterval::Day => true,
            BalanceInterval::Month => {
                d == to || d.succ_opt().is_some_and(|n| chrono::Datelike::day(&n) == 1)
            }
        };
        if is_point {
            out.push(BalancePoint {
                date: ds,
                balance: running,
            });
        }
        d = d.succ_opt().expect("data válida");
    }
    Ok(out)
}

/// Compara o saldo calculado até a data do extrato com o saldo informado pelo banco.
pub fn reconcile_check(conn: &Connection, args: &ReconcileArgs) -> Result<ReconcileResult> {
    parse_date(&args.statement_date)?;
    let computed = balance(conn, &args.account_id, &args.statement_date, false)?;
    Ok(ReconcileResult {
        computed_balance: computed,
        statement_balance: args.statement_balance,
        difference: args.statement_balance - computed,
        marked_reconciled: 0,
        change_set_id: None,
    })
}

/// Marca como conciliados os efetivados até a data do extrato. Devolve quantos mudaram.
pub fn mark_reconciled(tx: &Transaction<'_>, account_id: &str, until: &str) -> Result<usize> {
    Ok(tx.execute(
        "UPDATE txn SET status = 'reconciled', updated_at = ?3
          WHERE account_id = ?1 AND status = 'cleared' AND date <= ?2",
        params![account_id, until, now_utc()],
    )?)
}

pub fn account_name(conn: &Connection, id: &str) -> Result<String> {
    conn.query_row("SELECT name FROM account WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| Error::not_found(account_ref(id)))
}

// ---------------------------------------------------------------------------
// Escrita

fn validate_card(
    conn: &Connection,
    card: &CardDetails,
    self_id: Option<&str>,
) -> Result<CardDetails> {
    if !(1..=31).contains(&card.closing_day) {
        return Err(Error::invalid(t("nucleo-contas-cartao-erro-fechamento")));
    }
    if !(1..=31).contains(&card.due_day) {
        return Err(Error::invalid(t("nucleo-contas-cartao-erro-vencimento")));
    }
    if card.credit_limit.is_some_and(|l| l < 0) {
        return Err(Error::invalid(t("nucleo-contas-cartao-erro-limite")));
    }
    let payment = clean_opt(card.payment_account_id.clone());
    if let Some(p) = &payment {
        if Some(p.as_str()) == self_id {
            return Err(Error::invalid(t("nucleo-contas-cartao-erro-paga-propria")));
        }
        let (_, kind) = currency_and_kind(conn, p).map_err(|_| {
            Error::not_found(t_args(
                "nucleo-contas-ref-conta-pagadora",
                &args(&[("id", p.to_string().into())]),
            ))
        })?;
        if kind == AccountKind::CreditCard {
            return Err(Error::invalid(t(
                "nucleo-contas-cartao-erro-pagadora-cartao",
            )));
        }
    }
    let last4 = clean_opt(card.last4.clone());
    if let Some(l) = &last4
        && (l.len() != 4 || !l.chars().all(|c| c.is_ascii_digit()))
    {
        return Err(Error::invalid(t("nucleo-contas-cartao-erro-final")));
    }
    Ok(CardDetails {
        credit_limit: card.credit_limit,
        closing_day: card.closing_day,
        due_day: card.due_day,
        payment_account_id: payment,
        network: clean_opt(card.network.clone()),
        last4,
    })
}

fn validate_members(conn: &Connection, ids: &[Id]) -> Result<Vec<Id>> {
    let ids = dedup_ids(ids);
    for m in &ids {
        require_member(conn, m)?;
    }
    Ok(ids)
}

fn validate_institution(conn: &Connection, id: Option<String>) -> Result<Option<String>> {
    let id = clean_opt(id);
    if let Some(i) = &id
        && !exists(conn, "institution", i)?
    {
        return Err(Error::not_found(t_args(
            "nucleo-contas-ref-instituicao",
            &args(&[("id", i.to_string().into())]),
        )));
    }
    Ok(id)
}

/// Instituição pelo nome (sem diferenciar maiúsculas) ou uma nova, com o tipo deduzido da
/// conta (corretora → `broker`, cartão → `issuer`, demais → `bank`).
fn institution_by_name(tx: &Transaction<'_>, name: &str, kind: AccountKind) -> Result<Id> {
    if let Some(id) = tx
        .query_row(
            "SELECT id FROM institution WHERE lower(name) = lower(?1) ORDER BY created_at LIMIT 1",
            [name.trim()],
            |r| r.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    let kind = match kind {
        AccountKind::Brokerage => crate::household::InstitutionKind::Broker,
        AccountKind::CreditCard => crate::household::InstitutionKind::Issuer,
        _ => crate::household::InstitutionKind::Bank,
    };
    let inst = crate::household::create_institution(
        tx,
        &crate::household::InstitutionInput {
            name: name.trim().to_string(),
            kind: Some(kind),
            code: None,
            website: None,
        },
    )?;
    Ok(inst.id)
}

fn replace_members(tx: &Transaction<'_>, account_id: &str, ids: &[Id]) -> Result<()> {
    let current: Vec<String> = {
        let mut stmt = tx.prepare("SELECT member_id FROM account_member WHERE account_id = ?1")?;
        stmt.query_map([account_id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    for m in current.iter().filter(|m| !ids.contains(m)) {
        tx.execute(
            "DELETE FROM account_member WHERE account_id = ?1 AND member_id = ?2",
            params![account_id, m],
        )?;
    }
    for m in ids.iter().filter(|m| !current.contains(m)) {
        tx.execute(
            "INSERT INTO account_member (account_id, member_id) VALUES (?1, ?2)",
            params![account_id, m],
        )?;
    }
    Ok(())
}

fn bank_details(conn: &Connection, id: &str) -> Result<(Option<String>, Option<String>)> {
    Ok(conn.query_row(
        "SELECT branch, account_number FROM account WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}

/// Agência e conta: dígitos, hífen e "X" (dígito verificador), sem espaços. `branch`: é a
/// agência (senão, o número da conta), para a mensagem de erro.
fn clean_bank_ref(v: Option<String>, branch: bool) -> Result<Option<String>> {
    let Some(v) = clean_opt(v) else {
        return Ok(None);
    };
    let s: String = v
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '.')
        .collect();
    if s.len() > 30
        || !s
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == 'x' || c == 'X')
    {
        return Err(Error::invalid(t(if branch {
            "nucleo-contas-erro-agencia"
        } else {
            "nucleo-contas-erro-numero-conta"
        })));
    }
    Ok(Some(s.to_uppercase()))
}

fn set_bank_details(
    tx: &Transaction<'_>,
    id: &str,
    branch: Option<String>,
    number: Option<String>,
) -> Result<()> {
    tx.execute(
        "UPDATE account SET branch = ?2, account_number = ?3, updated_at = ?4
          WHERE id = ?1 AND (branch IS NOT ?2 OR account_number IS NOT ?3)",
        params![id, branch, number, now_utc()],
    )?;
    Ok(())
}

/// Cheque especial só em conta corrente (ou "outra", para contas de pagamento com limite).
fn check_overdraft_kind(kind: AccountKind) -> Result<()> {
    if matches!(kind, AccountKind::Checking | AccountKind::Other) {
        Ok(())
    } else {
        Err(Error::invalid(t("nucleo-contas-erro-cheque-especial-tipo")))
    }
}

fn upsert_card(tx: &Transaction<'_>, account_id: &str, c: &CardDetails) -> Result<()> {
    let now = now_utc();
    let old_limit: Option<i64> = tx
        .query_row(
            "SELECT credit_limit FROM credit_card WHERE account_id = ?1",
            [account_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    tx.execute(
        "INSERT INTO credit_card (account_id, credit_limit, closing_day, due_day, payment_account_id, network, last4,
                                  created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
         ON CONFLICT (account_id) DO UPDATE SET
            credit_limit = excluded.credit_limit, closing_day = excluded.closing_day, due_day = excluded.due_day,
            payment_account_id = excluded.payment_account_id, network = excluded.network, last4 = excluded.last4,
            updated_at = excluded.updated_at
          WHERE credit_limit IS NOT excluded.credit_limit OR closing_day IS NOT excluded.closing_day
             OR due_day IS NOT excluded.due_day OR payment_account_id IS NOT excluded.payment_account_id
             OR network IS NOT excluded.network OR last4 IS NOT excluded.last4",
        params![account_id, c.credit_limit, c.closing_day, c.due_day, c.payment_account_id, c.network, c.last4, now],
    )?;
    crate::cards::plastics::sync_holder_last4(tx, account_id, c.last4.as_deref())?;
    crate::cards::limits::record_manual(tx, account_id, old_limit, c.credit_limit)?;
    Ok(())
}

pub fn create(tx: &Transaction<'_>, input: &AccountInput, today: NaiveDate) -> Result<Id> {
    let name = clean_name(&input.name, "da conta")?;
    if input.kind.brazil_only() && !crate::country::is_brazil(tx) {
        return Err(Error::invalid(t("nucleo-contas-erro-tipo-so-brasil")));
    }
    let currency = match &input.currency {
        // O FGTS é sempre em reais.
        _ if input.kind == AccountKind::Fgts => "BRL".to_string(),
        Some(c) => crate::money::validate_currency(c)?,
        None => crate::household::base_currency(tx)?,
    };
    let institution = match (
        validate_institution(tx, input.institution_id.clone())?,
        clean_opt(input.institution_name.clone()),
    ) {
        (Some(id), _) => Some(id),
        (None, Some(name)) => Some(institution_by_name(tx, &name, input.kind)?),
        (None, None) => None,
    };
    let members = validate_members(tx, input.member_ids.as_deref().unwrap_or_default())?;
    let opening_date = match &input.opening_date {
        Some(d) => {
            parse_date(d)?;
            d.clone()
        }
        None => fmt_date(today),
    };
    let card = match (input.kind, &input.card) {
        (AccountKind::CreditCard, Some(c)) => Some(validate_card(tx, c, None)?),
        (AccountKind::CreditCard, None) => {
            return Err(Error::invalid(t("nucleo-contas-cartao-erro-dados")));
        }
        (_, Some(_)) => {
            return Err(Error::invalid(t("nucleo-contas-cartao-erro-so-cartao")));
        }
        (_, None) => None,
    };
    let sort: i64 = tx.query_row(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM account",
        [],
        |r| r.get(0),
    )?;
    let id = new_id();
    tx.execute(
        "INSERT INTO account (id, name, kind, currency, institution_id, opening_balance, opening_date,
                              include_in_net_worth, color, notes, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
        params![
            id,
            name,
            input.kind.as_str(),
            currency,
            institution,
            input.opening_balance.unwrap_or(0),
            opening_date,
            input.include_in_net_worth.unwrap_or(true),
            clean_color(input.color.clone())?,
            clean_opt(input.notes.clone()),
            sort,
            now_utc()
        ],
    )?;
    replace_members(tx, &id, &members)?;
    let branch = clean_bank_ref(input.branch.clone(), true)?;
    let number = clean_bank_ref(input.account_number.clone(), false)?;
    if branch.is_some() || number.is_some() {
        set_bank_details(tx, &id, branch, number)?;
    }
    if let Some(o) = &input.overdraft {
        check_overdraft_kind(input.kind)?;
        overdraft::upsert(tx, &id, &overdraft::validate(o)?)?;
    }
    if let Some(c) = &card {
        upsert_card(tx, &id, c)?;
    }
    if input.kind == AccountKind::Fgts {
        crate::fgts::ensure_account(tx, &id)?;
    }
    if input.kind != AccountKind::Retirement
        && (input.retirement_plan.is_some() || input.retirement_coverage.is_some())
    {
        return Err(Error::invalid(t(
            "nucleo-aposentadoria-erro-so-aposentadoria",
        )));
    }
    if input.kind == AccountKind::Retirement {
        let country = crate::country::family_country(tx);
        let plan = match &input.retirement_plan {
            Some(p) => p.clone(),
            None => crate::retirement::PLANS
                .iter()
                .find(|p| p.country == country)
                .map(|p| p.id.to_string())
                .ok_or_else(|| Error::invalid(t("nucleo-aposentadoria-erro-plano")))?,
        };
        crate::retirement::set_plan(tx, &id, &plan, input.retirement_coverage.as_deref())?;
    }
    Ok(id)
}

pub fn update(tx: &Transaction<'_>, patch: &AccountPatch, today: NaiveDate) -> Result<()> {
    let cur = get(tx, &patch.id, today)?.account;
    let name = match &patch.name {
        Some(n) => clean_name(n, "da conta")?,
        None => cur.name.clone(),
    };
    let institution = match &patch.institution_id {
        Some(i) => validate_institution(tx, i.clone())?,
        None => cur.institution_id.clone(),
    };
    let opening_date = match &patch.opening_date {
        Some(d) => {
            parse_date(d)?;
            d.clone()
        }
        None => cur.opening_date.clone(),
    };
    let color = match &patch.color {
        Some(c) => clean_color(c.clone())?,
        None => cur.color.clone(),
    };
    let notes = match &patch.notes {
        Some(n) => clean_opt(n.clone()),
        None => cur.notes.clone(),
    };
    tx.execute(
        "UPDATE account SET name = ?2, institution_id = ?3, opening_balance = ?4, opening_date = ?5,
                include_in_net_worth = ?6, archived = ?7, color = ?8, notes = ?9, sort_order = ?10, updated_at = ?11
          WHERE id = ?1 AND (name IS NOT ?2 OR institution_id IS NOT ?3 OR opening_balance IS NOT ?4
                OR opening_date IS NOT ?5 OR include_in_net_worth IS NOT ?6 OR archived IS NOT ?7
                OR color IS NOT ?8 OR notes IS NOT ?9 OR sort_order IS NOT ?10)",
        params![
            cur.id,
            name,
            institution,
            patch.opening_balance.unwrap_or(cur.opening_balance),
            opening_date,
            patch.include_in_net_worth.unwrap_or(cur.include_in_net_worth),
            patch.archived.unwrap_or(cur.archived),
            color,
            notes,
            patch.sort_order.unwrap_or(cur.sort_order),
            now_utc()
        ],
    )?;
    if let Some(m) = &patch.member_ids {
        let members = validate_members(tx, m)?;
        replace_members(tx, &cur.id, &members)?;
        // Os lançamentos com o membro herdado acompanham (os escolhidos à mão, não).
        crate::transactions::reinherit_members(tx, &cur.id)?;
    }
    if patch.branch.is_some() || patch.account_number.is_some() {
        let branch = match &patch.branch {
            Some(b) => clean_bank_ref(b.clone(), true)?,
            None => cur.branch.clone(),
        };
        let number = match &patch.account_number {
            Some(n) => clean_bank_ref(n.clone(), false)?,
            None => cur.account_number.clone(),
        };
        set_bank_details(tx, &cur.id, branch, number)?;
    }
    match &patch.overdraft {
        Some(Some(o)) => {
            check_overdraft_kind(cur.kind)?;
            overdraft::upsert(tx, &cur.id, &overdraft::validate(o)?)?;
        }
        Some(None) => overdraft::remove(tx, &cur.id)?,
        None => {}
    }
    if let Some(c) = &patch.card {
        if cur.kind != AccountKind::CreditCard {
            return Err(Error::invalid(t("nucleo-contas-cartao-erro-so-cartao")));
        }
        let c = validate_card(tx, c, Some(&cur.id))?;
        let cycle_changed = cur
            .card
            .as_ref()
            .is_none_or(|old| old.closing_day != c.closing_day || old.due_day != c.due_day);
        upsert_card(tx, &cur.id, &c)?;
        if cycle_changed {
            crate::cards::recompute_statements(tx, &cur.id, today)?;
        }
    }
    if patch.retirement_plan.is_some() || patch.retirement_coverage.is_some() {
        if cur.kind != AccountKind::Retirement {
            return Err(Error::invalid(t(
                "nucleo-aposentadoria-erro-so-aposentadoria",
            )));
        }
        let plan = patch
            .retirement_plan
            .clone()
            .or(cur.retirement_plan.clone())
            .ok_or_else(|| Error::invalid(t("nucleo-aposentadoria-erro-plano")))?;
        let coverage = patch
            .retirement_coverage
            .clone()
            .or(cur.retirement_coverage.clone());
        crate::retirement::set_plan(tx, &cur.id, &plan, coverage.as_deref())?;
    }
    Ok(())
}

/// Apaga uma conta sem lançamentos (e o que só existe por causa dela: faturas vazias,
/// dados do cartão, titulares, vínculos com metas e lotes de importação vazios).
pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<String> {
    let name = account_name(tx, id)?;
    let count = |sql: &str| -> Result<i64> { Ok(tx.query_row(sql, [id], |r| r.get(0))?) };
    let txns = count("SELECT count(*) FROM txn WHERE account_id = ?1")?;
    if txns > 0 {
        return Err(Error::conflict(t_args(
            "nucleo-contas-erro-apagar-lancamentos",
            &args(&[("nome", name.clone().into()), ("n", txns.into())]),
        )));
    }
    let recs =
        count("SELECT count(*) FROM recurrence WHERE account_id = ?1 OR to_account_id = ?1")?;
    if recs > 0 {
        return Err(Error::conflict(t_args(
            "nucleo-contas-erro-apagar-recorrencias",
            &args(&[("nome", name.clone().into()), ("n", recs.into())]),
        )));
    }
    let plans = count("SELECT count(*) FROM installment_plan WHERE account_id = ?1")?;
    if plans > 0 {
        return Err(Error::conflict(t_args(
            "nucleo-contas-erro-apagar-parcelamentos",
            &args(&[("nome", name.clone().into()), ("n", plans.into())]),
        )));
    }
    let now = now_utc();
    tx.execute(
        "UPDATE credit_card SET payment_account_id = NULL, updated_at = ?2 WHERE payment_account_id = ?1",
        params![id, now],
    )?;
    tx.execute("DELETE FROM goal_account WHERE account_id = ?1", [id])?;
    tx.execute("DELETE FROM import_batch WHERE account_id = ?1", [id])?;
    tx.execute("DELETE FROM card_statement WHERE account_id = ?1", [id])?;
    tx.execute("DELETE FROM card_plastic WHERE account_id = ?1", [id])?;
    tx.execute("DELETE FROM card_limit_change WHERE account_id = ?1", [id])?;
    tx.execute("DELETE FROM card_annual_fee WHERE account_id = ?1", [id])?;
    tx.execute(
        "DELETE FROM card_reward_program WHERE account_id = ?1",
        [id],
    )?;
    tx.execute("DELETE FROM credit_card WHERE account_id = ?1", [id])?;
    overdraft::remove(tx, id)?;
    tx.execute("DELETE FROM account_member WHERE account_id = ?1", [id])?;
    tx.execute("DELETE FROM account WHERE id = ?1", [id])?;
    Ok(name)
}

/// Motivo legível da conciliação.
pub fn reconcile_reason(conn: &Connection, args: &ReconcileArgs) -> Result<String> {
    Ok(t_args(
        "nucleo-contas-historico-conciliacao",
        &crate::i18n::args(&[
            ("nome", account_name(conn, &args.account_id)?.into()),
            ("data", date_text(&args.statement_date).into()),
        ]),
    ))
}

#[cfg(test)]
mod tests;

/// O que um OFX diz da conta (`BANKID`, `BRANCHID`, `ACCTID`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OfxMatchArgs {
    #[serde(default)]
    #[ts(optional)]
    pub bank_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub branch_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<String>,
    /// Extrato de cartão de crédito (`CCSTMTRS`): compara o final do cartão.
    #[serde(default)]
    #[ts(optional)]
    pub credit_card: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OfxMatch {
    /// Conta reconhecida (uma só).
    pub account_id: Option<Id>,
    /// Banco do OFX na lista do Banco Central.
    pub bank: Option<crate::household::banks::Bank>,
    /// Por que não reconheceu, quando não reconheceu.
    pub reason: Option<String>,
    /// A conta veio da escolha feita à mão na última importação deste arquivo.
    pub remembered: bool,
}

fn digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}

fn same_number(ofx: &str, ours: &str) -> bool {
    let (a, b) = (digits(ofx), digits(ours));
    if a.len() < 3 || b.len() < 3 {
        return false;
    }
    let trim = |x: &str| x.trim_start_matches('0').to_string();
    let (a, b) = (trim(&a), trim(&b));
    // Com ou sem o dígito verificador, com ou sem a agência na frente.
    a == b
        || a.ends_with(&b)
        || b.ends_with(&a)
        || (b.len() > 1 && a.ends_with(&b[..b.len() - 1]))
        || (a.len() > 1 && b.ends_with(&a[..a.len() - 1]))
}

/// Ajuste técnico: contas escolhidas à mão para OFX que o cadastro não reconhece sozinho
/// (`banco|agência|ACCTID|cartao` ou `…|conta` → id da conta), para a próxima importação
/// do mesmo arquivo já vir com a conta certa (ex.: cartão cujo `ACCTID` é um UUID).
pub const OFX_REFS_SETTING: &str = "accounts.ofx_refs";

/// Chave da conta do OFX na lembrança: banco e agência sem zeros à esquerda, `ACCTID`
/// aparado e o tipo do extrato. Sem `ACCTID`, não há o que lembrar.
fn ofx_ref_key(a: &OfxMatchArgs) -> Option<String> {
    let acct = a
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    let part = |v: &Option<String>| {
        v.as_deref()
            .map(|s| s.trim().trim_start_matches('0').to_lowercase())
            .unwrap_or_default()
    };
    let kind = if a.credit_card.unwrap_or(false) {
        "cartao"
    } else {
        "conta"
    };
    Some(format!(
        "{}|{}|{}|{kind}",
        part(&a.bank_id),
        part(&a.branch_id),
        acct.to_lowercase()
    ))
}

/// Conta lembrada para o OFX, se ainda existe, não está arquivada e é do tipo do extrato
/// (cartão ou não); senão a lembrança é ignorada.
fn remembered_ofx(conn: &Connection, a: &OfxMatchArgs) -> Result<Option<Id>> {
    let Some(key) = ofx_ref_key(a) else {
        return Ok(None);
    };
    let refs: HashMap<String, Id> =
        crate::settings::get_or(conn, OFX_REFS_SETTING, HashMap::new())?;
    let Some(id) = refs.get(&key) else {
        return Ok(None);
    };
    let kind: Option<String> = conn
        .query_row(
            "SELECT kind FROM account WHERE id = ?1 AND archived = 0",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let card = a.credit_card.unwrap_or(false);
    Ok(kind
        .filter(|k| (k == AccountKind::CreditCard.as_str()) == card)
        .map(|_| id.clone()))
}

/// Guarda a conta em que o OFX foi importado quando o cadastro não a reconhece sozinho
/// (nenhuma, mais de uma ou outra conta); quando reconhece a mesma, a lembrança sai e
/// volta a valer o cadastro. A última escolha vence.
pub fn remember_ofx(
    conn: &Connection,
    a: &OfxMatchArgs,
    account_id: &str,
    today: NaiveDate,
) -> Result<()> {
    let Some(key) = ofx_ref_key(a) else {
        return Ok(());
    };
    let card = a.credit_card.unwrap_or(false);
    let (_, kind) = currency_and_kind(conn, account_id)?;
    if (kind == AccountKind::CreditCard) != card {
        return Ok(());
    }
    let auto = auto_match_ofx(conn, a, ofx_bank(conn, a)?.as_ref(), today)?;
    let mut refs: HashMap<String, Id> =
        crate::settings::get_or(conn, OFX_REFS_SETTING, HashMap::new())?;
    let changed = if matches!(auto.as_slice(), [only] if only == account_id) {
        refs.remove(&key).is_some()
    } else {
        refs.insert(key, account_id.to_string()).as_deref() != Some(account_id)
    };
    if changed {
        crate::settings::set(conn, OFX_REFS_SETTING, &refs)?;
    }
    Ok(())
}

/// Banco do OFX na lista do Banco Central.
fn ofx_bank(conn: &Connection, a: &OfxMatchArgs) -> Result<Option<crate::household::banks::Bank>> {
    match a.bank_id.as_deref() {
        Some(b) => crate::household::banks::by_ofx_bank_id(conn, b),
        None => Ok(None),
    }
}

/// Contas que combinam com o OFX pelo cadastro: banco (código COMPE da instituição),
/// agência e número; no cartão, o final do cartão.
fn auto_match_ofx(
    conn: &Connection,
    a: &OfxMatchArgs,
    bank: Option<&crate::household::banks::Bank>,
    today: NaiveDate,
) -> Result<Vec<Id>> {
    let Some(acct) = a.account_id.as_deref().filter(|x| !x.trim().is_empty()) else {
        return Ok(vec![]);
    };
    let card = a.credit_card.unwrap_or(false);
    let code_of = |inst: &Option<Id>| -> Option<String> {
        let id = inst.as_ref()?;
        conn.query_row("SELECT code FROM institution WHERE id = ?1", [id], |r| {
            r.get::<_, Option<String>>(0)
        })
        .ok()
        .flatten()
        .map(|c| digits(&c).trim_start_matches('0').to_string())
    };
    let ofx_bank = a
        .bank_id
        .as_deref()
        .map(|b| digits(b).trim_start_matches('0').to_string())
        .filter(|b| !b.is_empty());
    let bank_code = bank
        .and_then(|b| b.code.clone())
        .map(|c| c.trim_start_matches('0').to_string());
    Ok(list(conn, false, today)?
        .into_iter()
        .filter(|v| (v.account.kind == AccountKind::CreditCard) == card)
        .filter(|v| {
            let acc = &v.account;
            if card {
                let last4 = acc
                    .card
                    .as_ref()
                    .and_then(|c| c.last4.clone())
                    .unwrap_or_default();
                return !last4.is_empty() && digits(acct).ends_with(&last4);
            }
            let Some(num) = acc.account_number.as_deref() else {
                return false;
            };
            if !same_number(acct, num) {
                return false;
            }
            // Banco e agência só eliminam quando os dois lados têm o dado.
            if let (Some(ours), Some(theirs)) = (
                code_of(&acc.institution_id),
                bank_code.clone().or(ofx_bank.clone()),
            ) && !ours.is_empty()
                && ours != theirs
            {
                return false;
            }
            if let (Some(ours), Some(theirs)) = (acc.branch.as_deref(), a.branch_id.as_deref()) {
                let (o, t) = (digits(ours), digits(theirs));
                if !o.is_empty()
                    && !t.is_empty()
                    && o.trim_start_matches('0') != t.trim_start_matches('0')
                {
                    return false;
                }
            }
            true
        })
        .map(|v| v.account.id)
        .collect())
}

/// Acha a conta do OFX: primeiro a escolhida à mão na última importação do mesmo arquivo
/// (`OFX_REFS_SETTING`); depois pelo banco (código COMPE da instituição), agência e número;
/// no cartão, pelo final do cartão.
pub fn match_ofx(conn: &Connection, a: &OfxMatchArgs, today: NaiveDate) -> Result<OfxMatch> {
    let bank = ofx_bank(conn, a)?;
    if a.account_id.as_deref().is_none_or(|x| x.trim().is_empty()) {
        return Ok(OfxMatch {
            account_id: None,
            bank,
            reason: Some(t("nucleo-contas-ofx-sem-numero")),
            remembered: false,
        });
    }
    if let Some(id) = remembered_ofx(conn, a)? {
        return Ok(OfxMatch {
            account_id: Some(id),
            bank,
            reason: None,
            remembered: true,
        });
    }
    let card = a.credit_card.unwrap_or(false);
    let hits = auto_match_ofx(conn, a, bank.as_ref(), today)?;
    Ok(match hits.as_slice() {
        [id] => OfxMatch {
            account_id: Some(id.clone()),
            bank,
            reason: None,
            remembered: false,
        },
        [] => OfxMatch {
            account_id: None,
            bank,
            reason: Some(t(if card {
                "nucleo-contas-ofx-sem-cartao"
            } else {
                "nucleo-contas-ofx-sem-conta"
            })),
            remembered: false,
        },
        _ => OfxMatch {
            account_id: None,
            bank,
            reason: Some(t("nucleo-contas-ofx-varias")),
            remembered: false,
        },
    })
}
