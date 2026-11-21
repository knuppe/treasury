//! Cotas capital de cooperativa de crédito, ligadas à conta corrente do cooperado.
//!
//! A remuneração das cotas não segue uma regra única entre cooperativas (juros sobre o
//! capital limitados à Selic, sobras distribuídas, atualização anual), então o Treasury não
//! tenta calcular: o usuário informa o saldo das cotas de tempos em tempos (do app ou do
//! extrato da cooperativa) e a rentabilidade de cada período é
//! `saldo novo − saldo anterior − aportes do período`. Aportes são os lançamentos da conta
//! que batem com o texto configurado (ex.: "COTA CAPITAL", "INTEGRALIZAÇÃO") ou estão na
//! categoria escolhida; devoluções (entradas) contam como aporte negativo.
//!
//! O saldo mais recente entra no patrimônio (como investimento), se o usuário quiser.

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc, parse_date};

/// Texto padrão dos aportes no extrato (separados por `|`).
pub const DEFAULT_PATTERN: &str = "cota capital|cotas capital|integraliz|capital social";

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct CapitalShareInput {
    pub account_id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub started_on: Option<String>,
    /// Textos que identificam os aportes no extrato, separados por `|`.
    #[serde(default)]
    #[ts(optional)]
    pub contribution_pattern: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub contribution_category_id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub include_in_net_worth: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct CapitalBalanceInput {
    pub account_id: Id,
    pub date: String,
    /// Saldo das cotas na data, em centavos.
    #[ts(type = "number")]
    pub balance: i64,
    #[serde(default)]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CapitalContribution {
    pub txn_id: Id,
    pub date: String,
    /// Positivo = aporte; negativo = devolução.
    #[ts(type = "number")]
    pub amount: i64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CapitalBalancePoint {
    pub id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub balance: i64,
    pub note: Option<String>,
    /// Aportes desde o saldo anterior (no primeiro, até a data).
    #[ts(type = "number")]
    pub contributions: i64,
    /// Variação − aportes; nulo no primeiro saldo.
    #[ts(type = "number | null")]
    pub yield_amount: Option<i64>,
    /// Rentabilidade do período sobre o saldo anterior + metade dos aportes (%), nula no primeiro.
    pub yield_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CapitalShareView {
    pub account_id: Id,
    pub currency: String,
    pub started_on: Option<String>,
    pub contribution_pattern: String,
    pub contribution_category_id: Option<Id>,
    pub include_in_net_worth: bool,
    pub notes: Option<String>,
    /// Do mais antigo ao mais recente.
    pub balances: Vec<CapitalBalancePoint>,
    #[ts(type = "number | null")]
    pub current_balance: Option<i64>,
    pub current_date: Option<String>,
    /// Aportes totais identificados (desde o início ou o primeiro saldo).
    #[ts(type = "number")]
    pub total_contributions: i64,
    /// Soma das rentabilidades dos períodos com saldo informado.
    #[ts(type = "number")]
    pub total_yield: i64,
    /// Rentabilidade dos últimos 12 meses (% sobre o saldo de 12 meses atrás + aportes), se
    /// houver saldo informado há mais de 11 meses.
    pub yield_12m_pct: Option<f64>,
    /// Aportes depois do último saldo informado (ainda sem rentabilidade apurada).
    #[ts(type = "number")]
    pub contributions_since_last: i64,
    pub contributions: Vec<CapitalContribution>,
}

fn fold(s: &str) -> String {
    crate::import::text::fold(s)
}

fn terms(pattern: &str) -> Vec<String> {
    pattern
        .split('|')
        .map(fold)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Aportes (e devoluções) da conta até `until`, pelo texto ou pela categoria.
fn contributions(
    conn: &Connection,
    account_id: &str,
    pattern: &str,
    category: Option<&str>,
) -> Result<Vec<CapitalContribution>> {
    let ts = terms(pattern);
    let mut stmt = conn.prepare(
        "SELECT id, date, amount, description, category_id FROM txn
          WHERE account_id = ?1 AND status IN ('cleared', 'reconciled') AND amount <> 0
          ORDER BY date, id",
    )?;
    let rows = stmt.query_map([account_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
        ))
    })?;
    // Vínculos explícitos (Lançamentos › Vincular a uma operação): o lançamento vinculado a
    // estas cotas entra mesmo sem bater com o texto, e o vinculado a outra operação sai.
    let mut stmt = conn.prepare(
        "SELECT l.txn_id, l.target_kind = 'capital_share' AND l.target_id = ?1 FROM txn_link l
           JOIN txn t ON t.id = l.txn_id
          WHERE t.account_id = ?1 OR (l.target_kind = 'capital_share' AND l.target_id = ?1)",
    )?;
    let links: std::collections::HashMap<String, bool> = stmt
        .query_map([account_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.date, t.amount, t.description, t.category_id FROM txn_link l
           JOIN txn t ON t.id = l.txn_id
          WHERE l.target_kind = 'capital_share' AND l.target_id = ?1 AND t.account_id <> ?1
            AND t.status IN ('cleared', 'reconciled') AND t.amount <> 0",
    )?;
    let others: Vec<(String, String, i64, String, Option<String>)> = stmt
        .query_map([account_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    let rows: Vec<_> = rows.collect::<rusqlite::Result<_>>()?;
    for (id, date, amount, desc, cat) in rows.into_iter().chain(others) {
        let by_cat = category.is_some() && cat.as_deref() == category;
        let d = fold(&desc);
        let by_text = ts.iter().any(|t| d.contains(t.as_str()));
        let counts = match links.get(&id) {
            Some(linked_here) => *linked_here,
            None => by_cat || by_text,
        };
        if counts {
            out.push(CapitalContribution {
                txn_id: id,
                date,
                amount: -amount,
                description: desc,
            });
        }
    }
    out.sort_by(|a, b| (&a.date, &a.txn_id).cmp(&(&b.date, &b.txn_id)));
    Ok(out)
}

fn exists(conn: &Connection, account_id: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM capital_share WHERE account_id = ?1)",
        [account_id],
        |r| r.get(0),
    )?)
}

pub fn set(tx: &Transaction<'_>, a: &CapitalShareInput) -> Result<()> {
    let (_, kind) = super::currency_and_kind(tx, &a.account_id)?;
    if !matches!(
        kind,
        super::AccountKind::Checking | super::AccountKind::Savings | super::AccountKind::Other
    ) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-contas-cotas-erro-tipo-conta",
        )));
    }
    if let Some(d) = &a.started_on {
        parse_date(d)?;
    }
    if let Some(c) = &a.contribution_category_id {
        crate::transactions::check_category(tx, Some(c), crate::transactions::TxnKind::Expense)?;
    }
    let pattern = a
        .contribution_pattern
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_string);
    let now = now_utc();
    tx.execute(
        "INSERT INTO capital_share (account_id, started_on, contribution_pattern, contribution_category_id,
                                    include_in_net_worth, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT (account_id) DO UPDATE SET started_on = ?2, contribution_pattern = ?3,
             contribution_category_id = ?4, include_in_net_worth = ?5, notes = ?6, updated_at = ?7",
        params![
            a.account_id,
            a.started_on,
            pattern,
            a.contribution_category_id,
            a.include_in_net_worth.unwrap_or(true) as i64,
            a.notes.as_deref().map(str::trim).filter(|n| !n.is_empty()),
            now
        ],
    )?;
    Ok(())
}

/// Tira os aportes de despesas: viram "aporte" (investimento), sem categoria, como as
/// transferências para a corretora. Devolve quantos mudaram.
pub fn mark_contributions(tx: &Transaction<'_>, account_id: &str) -> Result<u32> {
    let Some(v) = view(tx, account_id)? else {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-contas-cotas-erro-ative",
        )));
    };
    let mut n = 0;
    for c in v.contributions.iter().filter(|c| c.amount > 0) {
        n += tx.execute(
            "UPDATE txn SET kind = 'investment', category_id = NULL, updated_at = ?2
              WHERE id = ?1 AND kind = 'expense' AND transfer_id IS NULL",
            params![c.txn_id, now_utc()],
        )? as u32;
    }
    Ok(n)
}

pub fn remove(tx: &Transaction<'_>, account_id: &str) -> Result<()> {
    tx.execute(
        "DELETE FROM capital_share WHERE account_id = ?1",
        [account_id],
    )?;
    Ok(())
}

pub fn add_balance(tx: &Transaction<'_>, a: &CapitalBalanceInput) -> Result<Id> {
    if !exists(tx, &a.account_id)? {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-contas-cotas-erro-ative",
        )));
    }
    if a.balance < 0 {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-contas-cotas-erro-saldo-negativo",
        )));
    }
    let date = fmt_date(parse_date(&a.date)?);
    let now = now_utc();
    let note = a
        .note
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string);
    let id: Option<String> = tx
        .query_row(
            "SELECT id FROM capital_share_balance WHERE account_id = ?1 AND date = ?2",
            params![a.account_id, date],
            |r| r.get(0),
        )
        .optional()?;
    match id {
        Some(id) => {
            tx.execute(
                "UPDATE capital_share_balance SET balance = ?2, note = ?3, updated_at = ?4 WHERE id = ?1",
                params![id, a.balance, note, now],
            )?;
            Ok(id)
        }
        None => {
            let id = new_id();
            tx.execute(
                "INSERT INTO capital_share_balance (id, account_id, date, balance, note, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![id, a.account_id, date, a.balance, note, now],
            )?;
            Ok(id)
        }
    }
}

pub fn delete_balance(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM capital_share_balance WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(crate::i18n::t(
            "nucleo-contas-ref-saldo-cotas",
        )));
    }
    Ok(())
}

fn pct(num: i64, den: i64) -> Option<f64> {
    (den > 0).then(|| (num as f64 / den as f64 * 10_000.0).round() / 100.0)
}

/// (início, texto dos aportes, categoria, entra no patrimônio, notas).
type ShareRow = (
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
);

pub fn view(conn: &Connection, account_id: &str) -> Result<Option<CapitalShareView>> {
    let row: Option<ShareRow> = conn
        .query_row(
            "SELECT started_on, contribution_pattern, contribution_category_id, include_in_net_worth, notes
               FROM capital_share WHERE account_id = ?1",
            [account_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?;
    let Some((started_on, pattern, category, nw, notes)) = row else {
        return Ok(None);
    };
    let (currency, _) = super::currency_and_kind(conn, account_id)?;
    let pattern = pattern.unwrap_or_else(|| DEFAULT_PATTERN.to_string());
    let all = contributions(conn, account_id, &pattern, category.as_deref())?;
    let since =
        |c: &CapitalContribution| started_on.as_deref().is_none_or(|s| c.date.as_str() >= s);
    let contribs: Vec<CapitalContribution> = all.into_iter().filter(since).collect();
    let raw: Vec<(String, String, i64, Option<String>)> = conn
        .prepare(
            "SELECT id, date, balance, note FROM capital_share_balance WHERE account_id = ?1 ORDER BY date",
        )?
        .query_map([account_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let sum_between = |from: Option<&str>, to: &str| -> i64 {
        contribs
            .iter()
            .filter(|c| from.is_none_or(|f| c.date.as_str() > f) && c.date.as_str() <= to)
            .map(|c| c.amount)
            .sum()
    };
    let mut points = vec![];
    let mut prev: Option<(String, i64)> = None;
    let mut total_yield = 0;
    for (id, date, balance, note) in raw {
        let c = sum_between(prev.as_ref().map(|p| p.0.as_str()), &date);
        let (y, yp) = match &prev {
            Some((_, pb)) => {
                let y = balance - pb - c;
                total_yield += y;
                (Some(y), pct(y, pb + c / 2))
            }
            None => (None, None),
        };
        points.push(CapitalBalancePoint {
            id,
            date: date.clone(),
            balance,
            note,
            contributions: c,
            yield_amount: y,
            yield_pct: yp,
        });
        prev = Some((date, balance));
    }
    let last = points.last().cloned();
    // 12 meses: do saldo mais próximo de um ano antes do último até o último.
    let yield_12m_pct = last.as_ref().and_then(|l| {
        let target = parse_date(&l.date).ok()? - chrono::Duration::days(365);
        let base = points.iter().rfind(|p| {
            parse_date(&p.date).is_ok_and(|d| d <= target + chrono::Duration::days(31))
        })?;
        if base.date == l.date {
            return None;
        }
        let c = sum_between(Some(&base.date), &l.date);
        pct(l.balance - base.balance - c, base.balance + c / 2)
    });
    let contributions_since_last = match &last {
        Some(l) => contribs
            .iter()
            .filter(|c| c.date > l.date)
            .map(|c| c.amount)
            .sum(),
        None => contribs.iter().map(|c| c.amount).sum(),
    };
    Ok(Some(CapitalShareView {
        account_id: account_id.to_string(),
        currency,
        started_on,
        contribution_pattern: pattern,
        contribution_category_id: category,
        include_in_net_worth: nw == 1,
        notes,
        current_balance: last.as_ref().map(|l| l.balance),
        current_date: last.as_ref().map(|l| l.date.clone()),
        total_contributions: contribs.iter().map(|c| c.amount).sum(),
        total_yield,
        yield_12m_pct,
        contributions_since_last,
        balances: points,
        contributions: contribs,
    }))
}

/// Saldo das cotas na data (o último informado até ela) para o patrimônio, na moeda base.
pub fn net_worth_at(conn: &Connection, as_of: &str) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT a.currency,
                (SELECT b.balance FROM capital_share_balance b
                  WHERE b.account_id = s.account_id AND b.date <= ?1 ORDER BY b.date DESC LIMIT 1)
           FROM capital_share s JOIN account a ON a.id = s.account_id
          WHERE s.include_in_net_worth = 1 AND a.include_in_net_worth = 1",
    )?;
    let rows = stmt.query_map([as_of], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?))
    })?;
    let mut out = vec![];
    for r in rows {
        if let (c, Some(b)) = r? {
            out.push((c, b));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    #[test]
    fn yield_is_the_balance_change_minus_monthly_contributions() {
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
                json!({"name": "Cooperativa", "kind": "checking", "opening_date": "2025-01-01",
                       "opening_balance": 1_000_000}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        api.call("accounts.capital_share_set", json!({"account_id": acc}))
            .unwrap();
        // Contribution of R$ 50 every month ("INTEGRALIZACAO COTA CAPITAL").
        for m in 1..=12 {
            api.call(
                "transactions.create",
                json!({"account_id": acc, "date": format!("2025-{m:02}-05"), "amount": -5_000,
                       "description": "INTEGRALIZACAO COTA CAPITAL"}),
            )
            .unwrap();
        }
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": "2025-03-10", "amount": -9_990, "description": "MERCADO"}),
        )
        .unwrap();
        let bal = |date: &str, b: i64| {
            api.call(
                "accounts.capital_share_add_balance",
                json!({"account_id": acc, "date": date, "balance": b}),
            )
            .unwrap();
        };
        bal("2025-01-31", 105_000); // R$ 1.000 from before + January's contribution
        bal("2025-06-30", 135_000); // + 5 contributions (250) + 50 of yield
        bal("2025-12-31", 172_000); // + 6 contributions (300) + 70
        let v = api
            .call("accounts.capital_share", json!({"account_id": acc}))
            .unwrap();
        let b = v["balances"].as_array().unwrap();
        assert!(b[0]["yield_amount"].is_null());
        assert_eq!(b[1]["contributions"], 25_000);
        assert_eq!(b[1]["yield_amount"], 5_000);
        assert_eq!(b[2]["yield_amount"], 7_000);
        assert_eq!(v["total_yield"], 12_000);
        assert_eq!(v["total_contributions"], 60_000);
        assert_eq!(v["current_balance"], 172_000);
        assert_eq!(
            v["contributions"].as_array().unwrap().len(),
            12,
            "the market does not count"
        );
        // Contributions leave the expenses (they become contributions); the return does not change.
        let m = api
            .call(
                "accounts.capital_share_mark_contributions",
                json!({"account_id": acc}),
            )
            .unwrap();
        assert_eq!(m["updated"], 12);
        let v2 = api
            .call("accounts.capital_share", json!({"account_id": acc}))
            .unwrap();
        assert_eq!(v2["total_yield"], 12_000);
        // Counts toward net worth as an investment.
        let nw = api
            .call("reports.net_worth", json!({"as_of": "2025-12-31"}))
            .unwrap();
        assert!(nw["investments"].as_i64().unwrap() >= 172_000, "{nw}");
        // A card has no capital shares.
        let card = api
            .call(
                "accounts.create",
                json!({"name": "C", "kind": "credit_card",
                       "card": {"closing_day": 1, "due_day": 10}}),
            )
            .unwrap();
        assert!(
            api.call(
                "accounts.capital_share_set",
                json!({"account_id": card["id"]})
            )
            .is_err()
        );
    }
}
