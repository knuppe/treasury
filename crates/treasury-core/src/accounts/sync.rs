//! Marco de sincronia de saldo: o usuário vê no banco (ou no sistema do contrato) que o saldo
//! numa data é X e o Treasury calcula outro valor, sem saber por quê. O marco grava o saldo
//! informado, lança a diferença como **ajuste** (fora de receitas e despesas) e marca como
//! conciliado o que veio antes, para os saldos voltarem a bater com o banco.
//!
//! A diferença pode ser explicada depois: [`hints`] procura causas prováveis (duplicados,
//! lançamento do mesmo valor, previsto não efetivado, sinal trocado) para o usuário ou o
//! agente investigarem, e [`resolve`] grava o motivo achado.

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{self, args, t_args};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc, parse_date};
use crate::transactions::{NewTxn, TxnKind, TxnStatus};

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct SyncBalanceArgs {
    pub account_id: Id,
    /// Data do saldo informado (`YYYY-MM-DD`).
    pub date: String,
    /// Saldo que o banco mostra na data, em centavos.
    #[ts(type = "number")]
    pub balance: i64,
    #[serde(default)]
    #[ts(optional)]
    pub note: Option<String>,
    /// Marcar como conciliados os efetivados até a data (padrão: sim).
    #[serde(default)]
    #[ts(optional)]
    pub reconcile: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BalanceSyncStatus {
    /// Diferença ainda sem motivo.
    Open,
    /// Motivo achado e anotado.
    Explained,
    /// O usuário aceitou a diferença sem investigar.
    Ignored,
}

impl BalanceSyncStatus {
    fn as_str(self) -> &'static str {
        match self {
            BalanceSyncStatus::Open => "open",
            BalanceSyncStatus::Explained => "explained",
            BalanceSyncStatus::Ignored => "ignored",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "explained" => BalanceSyncStatus::Explained,
            "ignored" => BalanceSyncStatus::Ignored,
            _ => BalanceSyncStatus::Open,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BalanceSync {
    pub id: Id,
    pub account_id: Id,
    pub account_name: String,
    pub currency: String,
    pub date: String,
    #[ts(type = "number")]
    pub statement_balance: i64,
    #[ts(type = "number")]
    pub computed_balance: i64,
    /// Informado − calculado (o valor do ajuste).
    #[ts(type = "number")]
    pub difference: i64,
    pub adjustment_txn_id: Option<Id>,
    pub status: BalanceSyncStatus,
    pub note: Option<String>,
    pub explanation: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BalanceSyncHintKind {
    /// Dois lançamentos iguais (data, valor e descrição parecida).
    Duplicate,
    /// Um lançamento com o mesmo valor da diferença: pode estar sobrando ou faltando.
    SameAmount,
    /// Previsto que não foi efetivado com o valor da diferença.
    PlannedNotCleared,
    /// Um lançamento com metade da diferença: sinal trocado.
    SignFlipped,
    /// Lançamentos depois da última conciliação, para conferir um a um.
    Unreconciled,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BalanceSyncHint {
    pub kind: BalanceSyncHintKind,
    pub message: String,
    pub txn_ids: Vec<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BalanceSyncHints {
    pub sync: BalanceSync,
    /// Marco ou conciliação anterior (de onde vem o período investigado).
    pub since: Option<String>,
    #[ts(type = "number")]
    pub txns_in_period: i64,
    pub hints: Vec<BalanceSyncHint>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ResolveSyncArgs {
    pub id: Id,
    pub status: BalanceSyncStatus,
    #[serde(default)]
    #[ts(optional)]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct BalanceSyncListArgs {
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Só os ainda sem motivo.
    #[serde(default)]
    #[ts(optional)]
    pub open_only: Option<bool>,
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<BalanceSync> {
    Ok(BalanceSync {
        id: r.get("id")?,
        account_id: r.get("account_id")?,
        account_name: r.get("account_name")?,
        currency: r.get("currency")?,
        date: r.get("date")?,
        statement_balance: r.get("statement_balance")?,
        computed_balance: r.get("computed_balance")?,
        difference: r.get("difference")?,
        adjustment_txn_id: r.get("adjustment_txn_id")?,
        status: BalanceSyncStatus::parse(&r.get::<_, String>("status")?),
        note: r.get("note")?,
        explanation: r.get("explanation")?,
        created_at: r.get("created_at")?,
    })
}

const SELECT: &str = "SELECT s.*, a.name AS account_name, a.currency AS currency
                        FROM balance_sync s JOIN account a ON a.id = s.account_id";

pub fn get(conn: &Connection, id: &str) -> Result<BalanceSync> {
    conn.query_row(&format!("{SELECT} WHERE s.id = ?1"), [id], row)
        .optional()?
        .ok_or_else(|| Error::not_found(sync_ref(id)))
}

/// "sincronia de saldo {id}" (detalhe do erro "não encontrado").
fn sync_ref(id: &str) -> String {
    t_args(
        "nucleo-contas-ref-sincronia",
        &args(&[("id", id.to_string().into())]),
    )
}

pub fn list(conn: &Connection, a: &BalanceSyncListArgs) -> Result<Vec<BalanceSync>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE (?1 IS NULL OR s.account_id = ?1) AND (?2 = 0 OR s.status = 'open')
          ORDER BY s.date DESC, s.created_at DESC"
    ))?;
    Ok(stmt
        .query_map(
            params![a.account_id, a.open_only.unwrap_or(false) as i64],
            row,
        )?
        .collect::<rusqlite::Result<_>>()?)
}

fn money(cents: i64, currency: &str) -> String {
    crate::transactions::util::money_text(cents, currency)
}

/// Nota do ajuste (no idioma da instalação; fica gravada).
fn adjustment_note(
    date: &str,
    currency: &str,
    (bank, computed): (i64, i64),
    note: Option<&str>,
) -> String {
    let summary = t_args(
        "nucleo-contas-sincronia-nota",
        &args(&[
            ("data", crate::transactions::util::date_text(date).into()),
            ("banco", money(bank, currency).into()),
            ("calculado", money(computed, currency).into()),
        ]),
    );
    format!(
        "{summary}{}",
        note.map(|n| format!(" {n}")).unwrap_or_default()
    )
}

/// Lançamento de ajuste de um marco (conciliado), com a diferença informado − calculado.
fn insert_adjustment(
    tx: &Transaction<'_>,
    account_id: &str,
    date: &str,
    currency: &str,
    (bank, computed): (i64, i64),
    note: Option<&str>,
) -> Result<Id> {
    let mut t = NewTxn::new(
        account_id,
        date,
        bank - computed,
        currency,
        TxnKind::Adjustment,
        &i18n::t("nucleo-contas-sincronia-descricao"),
    );
    t.status = TxnStatus::Reconciled;
    t.notes = Some(adjustment_note(date, currency, (bank, computed), note));
    crate::transactions::insert(tx, &t)
}

/// Refaz os marcos de saldo de uma conta a partir de `from` (inclusive), em ordem, depois de
/// apagar lançamentos que já estavam no saldo calculado deles (as cópias entre contas): o
/// ajuste de cada marco passa a ser o saldo informado menos o calculado sem as cópias (some se
/// a diferença zerar). Devolve quantos marcos mudaram.
pub(crate) fn recompute_from(tx: &Transaction<'_>, account_id: &str, from: &str) -> Result<usize> {
    let (currency, _) = super::currency_and_kind(tx, account_id)?;
    /// Id, data, saldo informado, ajuste, status e nota de um marco.
    type Mark = (Id, String, i64, Option<Id>, String, Option<String>);
    let syncs: Vec<Mark> = {
        let mut st = tx.prepare(
            "SELECT id, date, statement_balance, adjustment_txn_id, status, note FROM balance_sync
              WHERE account_id = ?1 AND date >= ?2 ORDER BY date, created_at, id",
        )?;
        st.query_map(params![account_id, from], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let mut changed = 0;
    for (id, date, bank, adj, status, note) in syncs {
        let own: Option<(Id, i64)> = match &adj {
            Some(a) => tx
                .query_row("SELECT id, amount FROM txn WHERE id = ?1", [a], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .optional()?,
            None => None,
        };
        let computed =
            super::balance(tx, account_id, &date, false)? - own.as_ref().map_or(0, |o| o.1);
        let diff = bank - computed;
        if own.as_ref().map(|o| o.1) == Some(diff) || (own.is_none() && diff == 0) {
            continue;
        }
        let new_adj = match (own, diff) {
            (Some((tid, _)), 0) => {
                tx.execute("DELETE FROM txn WHERE id = ?1", [&tid])?;
                None
            }
            (Some((tid, _)), d) => {
                tx.execute(
                    "UPDATE txn SET amount = ?2, notes = ?3, updated_at = ?4 WHERE id = ?1",
                    params![
                        tid,
                        d,
                        adjustment_note(&date, &currency, (bank, computed), note.as_deref()),
                        now_utc()
                    ],
                )?;
                Some(tid)
            }
            (None, _) => Some(insert_adjustment(
                tx,
                account_id,
                &date,
                &currency,
                (bank, computed),
                note.as_deref(),
            )?),
        };
        let status = if diff == 0 && status == "open" {
            "explained"
        } else {
            status.as_str()
        };
        tx.execute(
            "UPDATE balance_sync SET computed_balance = ?2, difference = ?3, adjustment_txn_id = ?4,
                    status = ?5,
                    explanation = CASE WHEN ?3 = 0 AND explanation IS NULL THEN ?6 ELSE explanation END,
                    updated_at = ?7
              WHERE id = ?1",
            params![
                id,
                computed,
                diff,
                new_adj,
                status,
                i18n::t("nucleo-contas-sincronia-copias-removidas"),
                now_utc()
            ],
        )?;
        changed += 1;
    }
    Ok(changed)
}

/// Grava o marco e o ajuste. Sem diferença, só o marco (e a conciliação).
pub fn sync_balance(tx: &Transaction<'_>, a: &SyncBalanceArgs, today: NaiveDate) -> Result<Id> {
    let date = parse_date(&a.date)?;
    if date > today {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-sincronia-erro-futuro",
        )));
    }
    let date = fmt_date(date);
    let (currency, _) = super::currency_and_kind(tx, &a.account_id)?;
    let computed = super::balance(tx, &a.account_id, &date, false)?;
    let diff = a.balance - computed;
    let note = a
        .note
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string);
    let adjustment = if diff != 0 {
        Some(insert_adjustment(
            tx,
            &a.account_id,
            &date,
            &currency,
            (a.balance, computed),
            note.as_deref(),
        )?)
    } else {
        None
    };
    if a.reconcile.unwrap_or(true) {
        super::mark_reconciled(tx, &a.account_id, &date)?;
    }
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO balance_sync (id, account_id, date, statement_balance, computed_balance, difference,
                                   adjustment_txn_id, status, note, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![
            id,
            a.account_id,
            date,
            a.balance,
            computed,
            diff,
            adjustment,
            if diff == 0 { "explained" } else { "open" },
            note,
            now
        ],
    )?;
    Ok(id)
}

pub fn resolve(tx: &Transaction<'_>, a: &ResolveSyncArgs) -> Result<()> {
    let explanation = a
        .explanation
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty());
    if a.status == BalanceSyncStatus::Explained && explanation.is_none() {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-sincronia-erro-motivo",
        )));
    }
    let n = tx.execute(
        "UPDATE balance_sync SET status = ?2, explanation = COALESCE(?3, explanation), updated_at = ?4
          WHERE id = ?1",
        params![a.id, a.status.as_str(), explanation, now_utc()],
    )?;
    if n == 0 {
        return Err(Error::not_found(sync_ref(&a.id)));
    }
    Ok(())
}

/// Causas prováveis da diferença no período desde o marco (ou conciliação) anterior.
pub fn hints(conn: &Connection, id: &str) -> Result<BalanceSyncHints> {
    let s = get(conn, id)?;
    let diff = s.difference;
    // Início do período: o marco anterior ou, sem ele, 90 dias antes.
    let prev: Option<String> = conn
        .query_row(
            "SELECT max(date) FROM balance_sync WHERE account_id = ?1 AND date < ?2",
            params![s.account_id, s.date],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    let start = prev.clone().unwrap_or_else(|| {
        parse_date(&s.date)
            .map(|d| fmt_date(d - chrono::Duration::days(90)))
            .unwrap_or_default()
    });
    let adj = s.adjustment_txn_id.clone().unwrap_or_default();
    let in_period: Vec<(Id, String, i64, String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT id, date, amount, description, status FROM txn
              WHERE account_id = ?1 AND date > ?2 AND date <= ?3 AND id <> ?4 ORDER BY date",
        )?;
        stmt.query_map(params![s.account_id, start, s.date, adj], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let mut hints = vec![];
    if diff != 0 {
        let fmt = |c: i64| money(c, &s.currency);
        // Mesmo valor da diferença (sobrando ou faltando).
        let same: Vec<Id> = in_period
            .iter()
            .filter(|t| t.4 != "planned" && (t.2 == -diff || t.2 == diff))
            .map(|t| t.0.clone())
            .collect();
        if !same.is_empty() {
            hints.push(BalanceSyncHint {
                kind: BalanceSyncHintKind::SameAmount,
                message: t_args(
                    "nucleo-contas-sincronia-dica-mesmo-valor",
                    &args(&[("n", same.len().into()), ("valor", fmt(diff.abs()).into())]),
                ),
                txn_ids: same,
            });
        }
        // Previsto do valor da diferença que não virou efetivado.
        let planned: Vec<Id> = in_period
            .iter()
            .filter(|t| t.4 == "planned" && t.2 == diff)
            .map(|t| t.0.clone())
            .collect();
        if !planned.is_empty() {
            hints.push(BalanceSyncHint {
                kind: BalanceSyncHintKind::PlannedNotCleared,
                message: i18n::t("nucleo-contas-sincronia-dica-previsto"),
                txn_ids: planned,
            });
        }
        // Metade da diferença: entrada lançada como saída (ou o contrário).
        if diff % 2 == 0 {
            let half: Vec<Id> = in_period
                .iter()
                .filter(|t| t.4 != "planned" && t.2 == -diff / 2)
                .map(|t| t.0.clone())
                .collect();
            if !half.is_empty() {
                hints.push(BalanceSyncHint {
                    kind: BalanceSyncHintKind::SignFlipped,
                    message: t_args(
                        "nucleo-contas-sincronia-dica-sinal",
                        &args(&[("valor", fmt((diff / 2).abs()).into())]),
                    ),
                    txn_ids: half,
                });
            }
        }
        // Duplicados: mesma data e valor, descrição parecida.
        let mut dups: Vec<Id> = vec![];
        for (i, a) in in_period.iter().enumerate() {
            for b in &in_period[i + 1..] {
                if a.1 == b.1
                    && a.2 == b.2
                    && a.4 != "planned"
                    && b.4 != "planned"
                    && crate::import::text::very_similar(&a.3, &b.3)
                {
                    for id in [&a.0, &b.0] {
                        if !dups.contains(id) {
                            dups.push(id.clone());
                        }
                    }
                }
            }
        }
        if !dups.is_empty() {
            hints.push(BalanceSyncHint {
                kind: BalanceSyncHintKind::Duplicate,
                message: t_args(
                    "nucleo-contas-sincronia-dica-duplicados",
                    &args(&[("n", dups.len().into())]),
                ),
                txn_ids: dups,
            });
        }
        let open: Vec<Id> = in_period
            .iter()
            .filter(|t| t.4 == "cleared")
            .map(|t| t.0.clone())
            .collect();
        if hints.is_empty() && !open.is_empty() {
            hints.push(BalanceSyncHint {
                kind: BalanceSyncHintKind::Unreconciled,
                message: t_args(
                    "nucleo-contas-sincronia-dica-conferir",
                    &args(&[("n", in_period.len().into())]),
                ),
                txn_ids: open,
            });
        }
    }
    Ok(BalanceSyncHints {
        txns_in_period: in_period.len() as i64,
        since: prev,
        hints,
        sync: s,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    #[test]
    fn sync_posts_the_difference_as_adjustment_and_suggests_causes() {
        let dir = tempfile::tempdir().unwrap();
        let api =
            Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
                .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        let acc = api
            .call(
                "accounts.create",
                json!({"name": "Itaú", "kind": "checking", "opening_balance": 100_000,
                       "opening_date": "2026-09-01"}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        let mk = |date: &str, amount: i64, desc: &str| {
            api.call(
                "transactions.create",
                json!({"account_id": acc, "date": date, "amount": amount, "description": desc}),
            )
            .unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string()
        };
        mk("2026-09-05", -20_000, "MERCADO");
        let dup = mk("2026-09-10", -4_590, "NETFLIX");
        mk("2026-09-10", -4_590, "NETFLIX");
        // The bank shows 75,590: Treasury computes 70,820 (Netflix was counted twice).
        let s = api
            .call(
                "accounts.sync_balance",
                json!({"account_id": acc, "date": "2026-09-30", "balance": 75_410}),
            )
            .unwrap();
        assert_eq!(s["computed_balance"], 70_820);
        assert_eq!(s["difference"], 4_590);
        assert_eq!(s["status"], "open");
        let adj = api
            .call("transactions.get", json!({"id": s["adjustment_txn_id"]}))
            .unwrap();
        assert_eq!(adj["kind"], "adjustment");
        assert_eq!(adj["amount"], 4_590);
        // With the adjustment, the balance matches the bank.
        let bal = api.call("accounts.get", json!({"id": acc})).unwrap();
        assert_eq!(bal["balance"], 75_410);
        let h = api
            .call("accounts.sync_hints", json!({"id": s["id"]}))
            .unwrap();
        let kinds: Vec<&str> = h["hints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["kind"].as_str().unwrap())
            .collect();
        assert!(kinds.contains(&"duplicate"), "{h}");
        assert!(kinds.contains(&"same_amount"), "{h}");
        assert!(h["hints"].to_string().contains(&dup));
        // Explaining requires the reason.
        assert!(
            api.call(
                "accounts.resolve_sync",
                json!({"id": s["id"], "status": "explained"})
            )
            .is_err()
        );
        api.call(
            "accounts.resolve_sync",
            json!({"id": s["id"], "status": "explained", "explanation": "Netflix duplicada"}),
        )
        .unwrap();
        let l = api
            .call("accounts.list_syncs", json!({"open_only": true}))
            .unwrap();
        assert!(l.as_array().unwrap().is_empty());
        // No difference: only the milestone, already explained, and no adjustment.
        let same = api
            .call(
                "accounts.sync_balance",
                json!({"account_id": acc, "date": "2026-10-01", "balance": 75_410}),
            )
            .unwrap();
        assert_eq!(same["difference"], 0);
        assert!(same["adjustment_txn_id"].is_null());
        assert!(
            api.call(
                "accounts.sync_balance",
                json!({"account_id": acc, "date": "2099-01-01", "balance": 1}),
            )
            .is_err()
        );
    }
}
