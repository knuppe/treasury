//! Quanto o saldo de cada conta está em dia (docs/account-freshness.md). Sem o banco ligado,
//! o saldo é o que foi lançado ou importado: uma conta com o último lançamento há semanas mostra
//! um saldo que parece atual. A conta fica `fresh`, `aging` ou `possibly_incomplete` pelos dias
//! desde o último lançamento, a última importação de extrato ou a última conferência de saldo
//! (o mais recente dos três), com limites por tipo de conta nos ajustes. Também fica
//! `possibly_incomplete` a conta que costuma ter lançamentos todo mês e fechou o mês passado
//! sem nenhum.

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::AccountKind;
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::Id;
use crate::time::{add_months, fmt_date, month_bounds, month_of, parse_date};

/// Ajuste com os limites trocados pela família, por tipo de conta.
pub const SETTING: &str = "accounts.staleness";

/// Meses fechados antes do mês passado que, todos com lançamentos, fazem a conta "de todo mês".
const HABIT_MONTHS: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BalanceStatus {
    Fresh,
    Aging,
    PossiblyIncomplete,
}

/// Dias sem novidade a partir dos quais a conta fica `aging` e `possibly_incomplete`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StalenessLimits {
    pub aging_days: u32,
    pub incomplete_days: u32,
}

/// Os limites padrão: conta corrente e afins em 7 e 20 dias; cartão pela fatura do mês;
/// benefício e contas de investimento com menos movimento, mais folga.
pub fn default_limits(kind: AccountKind) -> StalenessLimits {
    use AccountKind::*;
    let (aging_days, incomplete_days) = match kind {
        Checking | Savings | Cash | Foreign | Other => (7, 20),
        CreditCard => (35, 50),
        Benefit => (20, 45),
        Brokerage | Retirement | Fgts => (45, 90),
    };
    StalenessLimits {
        aging_days,
        incomplete_days,
    }
}

const KINDS: [AccountKind; 10] = [
    AccountKind::Checking,
    AccountKind::Savings,
    AccountKind::Cash,
    AccountKind::CreditCard,
    AccountKind::Benefit,
    AccountKind::Foreign,
    AccountKind::Brokerage,
    AccountKind::Retirement,
    AccountKind::Fgts,
    AccountKind::Other,
];

/// Os limites de um tipo de conta nos ajustes.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StalenessSetting {
    pub kind: AccountKind,
    pub limits: StalenessLimits,
    pub default_limits: StalenessLimits,
    /// A família trocou o padrão.
    pub custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetStalenessArgs {
    pub kind: AccountKind,
    /// Sem eles, volta ao padrão do tipo.
    #[ts(optional)]
    pub limits: Option<StalenessLimits>,
}

fn custom(conn: &Connection) -> Result<HashMap<String, StalenessLimits>> {
    crate::settings::get_or(conn, SETTING, HashMap::new())
}

/// Os limites de cada tipo (o da família ou o padrão).
pub fn limits(conn: &Connection) -> Result<HashMap<AccountKind, StalenessLimits>> {
    let c = custom(conn)?;
    Ok(KINDS
        .iter()
        .map(|k| (*k, c.get(k.as_str()).copied().unwrap_or(default_limits(*k))))
        .collect())
}

/// Os limites de todos os tipos, para a tela de ajustes.
pub fn settings(conn: &Connection) -> Result<Vec<StalenessSetting>> {
    let c = custom(conn)?;
    Ok(KINDS
        .iter()
        .map(|k| {
            let d = default_limits(*k);
            let own = c.get(k.as_str()).copied();
            StalenessSetting {
                kind: *k,
                limits: own.unwrap_or(d),
                default_limits: d,
                custom: own.is_some_and(|o| o != d),
            }
        })
        .collect())
}

pub fn set(conn: &Connection, a: &SetStalenessArgs) -> Result<Vec<StalenessSetting>> {
    let mut c = custom(conn)?;
    match a.limits {
        Some(l) => {
            if l.aging_days == 0 || l.incomplete_days <= l.aging_days || l.incomplete_days > 3650 {
                return Err(Error::invalid(t("nucleo-contas-erro-limites-atualizacao")));
            }
            c.insert(a.kind.as_str().into(), l);
        }
        None => {
            c.remove(a.kind.as_str());
        }
    }
    crate::settings::set(conn, SETTING, &c)?;
    settings(conn)
}

/// As contas do dia a dia (corrente, poupança, dinheiro, cartão, benefício e exterior) que
/// entram no patrimônio e estão com o saldo possivelmente incompleto. É a mesma lista para o
/// lembrete semanal, a saúde financeira e a cobertura dos números (`attention`).
pub fn everyday_incomplete(conn: &Connection, today: NaiveDate) -> Result<Vec<super::AccountView>> {
    Ok(everyday(conn, today)?
        .into_iter()
        .filter(|a| a.balance_status == Some(BalanceStatus::PossiblyIncomplete))
        .collect())
}

/// As contas do dia a dia que entram no patrimônio (em dia ou não).
pub fn everyday(conn: &Connection, today: NaiveDate) -> Result<Vec<super::AccountView>> {
    use AccountKind::*;
    Ok(super::list(conn, false, today)?
        .into_iter()
        .filter(|a| {
            a.account.include_in_net_worth
                && matches!(
                    a.account.kind,
                    Checking | Savings | Cash | CreditCard | Benefit | Foreign
                )
        })
        .collect())
}

/// O status pelos dias sem novidade (ou pelo mês passado sem lançamento).
pub fn classify(staleness_days: i64, limits: StalenessLimits, missed_month: bool) -> BalanceStatus {
    if missed_month || staleness_days > i64::from(limits.incomplete_days) {
        BalanceStatus::PossiblyIncomplete
    } else if staleness_days > i64::from(limits.aging_days) {
        BalanceStatus::Aging
    } else {
        BalanceStatus::Fresh
    }
}

/// O que se sabe da atualização de uma conta.
#[derive(Debug, Clone, Default)]
pub(crate) struct Inputs {
    /// Última importação de extrato ou conferência de saldo (`YYYY-MM-DD`).
    pub last_import: Option<String>,
    /// O mês passado (`YYYY-MM`), quando ficou sem lançamento numa conta de todo mês.
    pub missed_month: Option<String>,
}

/// Importações, conferências de saldo e lançamentos por mês de todas as contas, de uma vez.
pub(crate) fn inputs(conn: &Connection, today: NaiveDate) -> Result<HashMap<Id, Inputs>> {
    let mut out: HashMap<Id, Inputs> = HashMap::new();
    let mut stmt = conn.prepare_cached(
        "SELECT account_id, MAX(d) FROM (
             SELECT account_id, substr(created_at, 1, 10) AS d FROM import_batch
             UNION ALL SELECT account_id, date AS d FROM balance_sync WHERE status <> 'ignored')
          GROUP BY account_id",
    )?;
    let mut q = stmt.query([])?;
    while let Some(r) = q.next()? {
        let id: Id = r.get(0)?;
        out.entry(id).or_default().last_import = r.get(1)?;
    }
    // Lançamentos por mês nos meses fechados: o passado e os três antes dele.
    let this_month = month_bounds(
        chrono::Datelike::year(&today),
        chrono::Datelike::month(&today),
    )
    .0;
    let last = add_months(this_month, -1);
    let first = add_months(this_month, -(HABIT_MONTHS + 1));
    let mut stmt = conn.prepare_cached(
        "SELECT account_id, substr(date, 1, 7), COUNT(*) FROM txn
          WHERE status <> 'planned' AND date >= ?1 AND date < ?2
          GROUP BY account_id, substr(date, 1, 7)",
    )?;
    let mut counts: HashMap<Id, HashMap<String, i64>> = HashMap::new();
    let mut q = stmt.query(params![fmt_date(first), fmt_date(this_month)])?;
    while let Some(r) = q.next()? {
        counts
            .entry(r.get(0)?)
            .or_default()
            .insert(r.get(1)?, r.get(2)?);
    }
    let last_m = month_of(last);
    let habit: Vec<String> = (1..=HABIT_MONTHS)
        .map(|i| month_of(add_months(last, -i)))
        .collect();
    for (id, by_month) in counts {
        let every = habit
            .iter()
            .all(|m| by_month.get(m).is_some_and(|c| *c > 0));
        if every && !by_month.contains_key(&last_m) {
            out.entry(id).or_default().missed_month = Some(last_m.clone());
        }
    }
    Ok(out)
}

/// Os campos de atualização de uma conta: dias desde a novidade mais recente (último
/// lançamento, importação ou conferência; sem nenhum, a abertura da conta) e o status.
pub(crate) fn assess(
    kind: AccountKind,
    opening_date: &str,
    last_txn_date: Option<&str>,
    inputs: Option<&Inputs>,
    limits: &HashMap<AccountKind, StalenessLimits>,
    today: NaiveDate,
) -> (Option<i64>, BalanceStatus, bool) {
    let latest = [last_txn_date, inputs.and_then(|i| i.last_import.as_deref())]
        .into_iter()
        .flatten()
        .filter_map(|d| parse_date(d).ok())
        .max()
        .or_else(|| parse_date(opening_date).ok());
    let days = latest.map(|d| (today - d).num_days().max(0));
    let lim = limits.get(&kind).copied().unwrap_or(default_limits(kind));
    // A regra do mês sem lançamento só vale para as contas de movimento mensal.
    let missed = lim.incomplete_days <= 31 && inputs.is_some_and(|i| i.missed_month.is_some());
    (days, classify(days.unwrap_or(0), lim, missed), missed)
}
