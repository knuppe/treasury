//! Cópias entre contas: o mesmo identificador do banco (`external_id`) gravado em duas contas.
//! Acontece quando o arquivo de uma conta é importado em outra (o CSV do cartão do Nubank
//! importado na conta à vista, por exemplo): a compra aparece duas vezes, entra duas vezes em
//! "sem categoria" e, categorizada, dobra o gasto no fluxo de caixa.
//!
//! Regras:
//! - Só conta como cópia entre contas da mesma instituição, ou com identificador longo: 16
//!   caracteres ou mais quando uma das contas não tem instituição, e do tamanho de um UUID
//!   (32 ou mais, como os do Nubank) entre quaisquer contas. FITIDs curtos de bancos
//!   diferentes podem coincidir por acaso.
//! - As duas pernas de uma mesma transferência não são cópia uma da outra.
//! - Fica o lançamento do cartão quando a outra conta não é cartão (a compra é do cartão);
//!   senão, o gravado primeiro. A cópia é a outra.
//! - Nada é apagado sozinho: a pessoa revisa e confirma (`remove_copies`, desfazível pelo
//!   histórico). Cópia ligada como perna de transferência a outro lançamento tem a ligação
//!   desfeita antes (o outro lado volta a ser despesa ou receita) e só então sai.

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use rusqlite::{Connection, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::TxnView;
use crate::error::Result;
use crate::ids::Id;

/// Identificador longo o bastante para valer quando uma das contas não tem instituição.
const LONG_ID: usize = 16;
/// Do tamanho de um UUID (32 dígitos hexadecimais ou mais): vale entre quaisquer contas, mesmo
/// com instituições diferentes (o cartão e a conta do Nubank às vezes ficam em cadastros
/// separados, "Nubank" e "Nu Pagamentos").
const GLOBAL_ID: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CrossAccountDuplicate {
    /// O lançamento que fica (da conta certa).
    pub keep: TxnView,
    /// A cópia em outra conta.
    pub copy: TxnView,
    /// A cópia está ligada como transferência ou pagamento de fatura a outro lançamento: ao
    /// apagar, a ligação é desfeita e o outro lado volta a ser despesa ou receita.
    pub copy_is_transfer_leg: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CrossAccountDuplicatesArgs {
    /// Só as cópias gravadas nesta conta.
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

struct Row {
    id: Id,
    account_id: Id,
    account_kind: String,
    institution_id: Option<String>,
    transfer_id: Option<String>,
    created_at: String,
}

/// O par vale como cópia? (mesma instituição, ou id longo sem instituição numa das contas.)
fn same_source(a: &Row, b: &Row, ext: &str) -> bool {
    if ext.chars().count() >= GLOBAL_ID {
        return true;
    }
    match (&a.institution_id, &b.institution_id) {
        (Some(x), Some(y)) => x == y,
        _ => ext.chars().count() >= LONG_ID,
    }
}

/// Quem fica: o do cartão (se só um dos dois é cartão), senão o mais antigo.
fn keeper<'a>(a: &'a Row, b: &'a Row) -> (&'a Row, &'a Row) {
    let card = |r: &Row| r.account_kind == "credit_card";
    match (card(a), card(b)) {
        (true, false) => (a, b),
        (false, true) => (b, a),
        _ if (&a.created_at, &a.id) <= (&b.created_at, &b.id) => (a, b),
        _ => (b, a),
    }
}

/// Pares (fica, cópia) do mesmo `external_id` em contas diferentes.
pub(crate) fn pairs(conn: &Connection) -> Result<Vec<(Id, Id, bool)>> {
    let mut stmt = conn.prepare(
        "SELECT t.external_id, t.id, t.account_id, a.kind, a.institution_id, t.transfer_id,
                t.created_at
           FROM txn t JOIN account a ON a.id = t.account_id
          WHERE t.external_id IN (
                SELECT external_id FROM txn
                 WHERE external_id IS NOT NULL AND external_id <> ''
                 GROUP BY external_id HAVING COUNT(DISTINCT account_id) > 1)
          ORDER BY t.external_id, t.created_at, t.id",
    )?;
    let mut groups: Vec<(String, Vec<Row>)> = vec![];
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            Row {
                id: r.get(1)?,
                account_id: r.get(2)?,
                account_kind: r.get(3)?,
                institution_id: r.get(4)?,
                transfer_id: r.get(5)?,
                created_at: r.get(6)?,
            },
        ))
    })?;
    for x in rows {
        let (ext, row) = x?;
        match groups.last_mut() {
            Some((e, v)) if *e == ext => v.push(row),
            _ => groups.push((ext, vec![row])),
        }
    }
    let mut out = vec![];
    for (ext, rows) in groups {
        // Um lançamento por conta (o primeiro); o par é entre contas diferentes.
        let mut by_account: Vec<&Row> = vec![];
        for r in &rows {
            if !by_account.iter().any(|x| x.account_id == r.account_id) {
                by_account.push(r);
            }
        }
        for (i, a) in by_account.iter().enumerate() {
            for b in &by_account[i + 1..] {
                let linked = a.transfer_id.is_some() && a.transfer_id == b.transfer_id;
                if linked || !same_source(a, b, &ext) {
                    continue;
                }
                let (keep, copy) = keeper(a, b);
                out.push((keep.id.clone(), copy.id.clone(), copy.transfer_id.is_some()));
            }
        }
    }
    Ok(out)
}

/// Cópias entre contas, com os dois lançamentos para a revisão (mais recentes primeiro).
pub fn list(
    conn: &Connection,
    a: &CrossAccountDuplicatesArgs,
) -> Result<Vec<CrossAccountDuplicate>> {
    let pairs = pairs(conn)?;
    let ids: Vec<Id> = pairs
        .iter()
        .flat_map(|(k, c, _)| [k.clone(), c.clone()])
        .collect();
    let views: HashMap<Id, TxnView> = super::views(conn, &ids)?
        .into_iter()
        .map(|v| (v.id.clone(), v))
        .collect();
    let mut out: Vec<CrossAccountDuplicate> = pairs
        .into_iter()
        .filter_map(|(k, c, leg)| {
            Some(CrossAccountDuplicate {
                keep: views.get(&k)?.clone(),
                copy: views.get(&c)?.clone(),
                copy_is_transfer_leg: leg,
            })
        })
        .filter(|d| {
            a.account_id
                .as_ref()
                .is_none_or(|acc| d.copy.account_id == *acc)
        })
        .collect();
    out.sort_by(|x, y| {
        y.copy
            .date
            .cmp(&x.copy.date)
            .then(x.copy.id.cmp(&y.copy.id))
    });
    if let Some(l) = a.limit {
        out.truncate(l.clamp(1, 5000) as usize);
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RemoveCopiesResult {
    #[ts(type = "number")]
    pub removed: i64,
    /// Marcos de saldo refeitos sem as cópias (o ajuste deles diminui ou some).
    #[serde(default)]
    #[ts(type = "number")]
    pub balance_syncs_adjusted: i64,
    /// Ids pedidos que não são cópia (ou são perna de transferência) e ficaram.
    pub skipped: Vec<Id>,
    pub change_set_id: Option<Id>,
}

/// Apaga só os ids que são de fato cópias (e não pernas de transferência). O resto fica.
pub(crate) fn removable(conn: &Connection, ids: &[Id]) -> Result<(Vec<Id>, Vec<Id>)> {
    let copies: HashMap<Id, bool> = pairs(conn)?
        .into_iter()
        .map(|(_, c, leg)| (c, leg))
        .collect();
    let (ok, skip): (Vec<Id>, Vec<Id>) =
        ids.iter().cloned().partition(|id| copies.contains_key(id));
    Ok((ok, skip))
}

/// Apaga as cópias, mesmo conciliadas: o marco de saldo que as conciliou também contava com
/// elas no saldo calculado (o ajuste compensou a mais), então os marcos da conta desde a cópia
/// mais antiga são refeitos sem elas. Devolve (lançamentos apagados, marcos refeitos).
pub(crate) fn remove(tx: &Transaction<'_>, ids: &[Id], today: NaiveDate) -> Result<(usize, usize)> {
    let mut first: BTreeMap<Id, String> = BTreeMap::new();
    for id in ids {
        let (acc, date): (Id, String) = tx.query_row(
            "SELECT account_id, date FROM txn WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let e = first.entry(acc).or_insert_with(|| date.clone());
        if date < *e {
            *e = date;
        }
    }
    // Cópia ligada como transferência ou pagamento de fatura a outro lançamento (a ligação
    // automática juntou "IOF de X" da cópia com "IOF de volta de X" no cartão): desfaz a
    // ligação e o outro lado volta a ser despesa ou receita pelo sinal, antes de apagar.
    for id in ids {
        let linked: Option<String> =
            tx.query_row("SELECT transfer_id FROM txn WHERE id = ?1", [id], |r| {
                r.get(0)
            })?;
        if linked.is_none() {
            continue;
        }
        for other in super::transfers::unlink(tx, id, today)? {
            if other == *id {
                continue;
            }
            let desc: String =
                tx.query_row("SELECT description FROM txn WHERE id = ?1", [&other], |r| {
                    r.get(0)
                })?;
            if crate::import::is_bill_payment(&desc, true) {
                continue;
            }
            tx.execute(
                "UPDATE txn SET kind = CASE WHEN amount < 0 THEN 'expense' ELSE 'income' END,
                        updated_at = ?2
                  WHERE id = ?1",
                params![other, crate::time::now_utc()],
            )?;
        }
    }
    let n = super::write::delete_checked(tx, ids, today, false)?;
    let mut syncs = 0;
    for (acc, from) in first {
        syncs += crate::accounts::sync::recompute_from(tx, &acc, &from)?;
    }
    Ok((n, syncs))
}

/// Quantas cópias entre contas existem (para o aviso na tela de lançamentos).
pub fn count(conn: &Connection) -> Result<i64> {
    Ok(pairs(conn)?.len() as i64)
}

/// Outra conta (da mesma instituição, ou qualquer uma com id longo) que já tem este
/// `external_id`: o id do lançamento e o nome da conta. Para a importação marcar a linha.
pub(crate) fn in_other_account(
    conn: &Connection,
    account_id: &str,
    external_id: &str,
) -> Result<Option<(Id, String)>> {
    let n = external_id.chars().count();
    let long = n >= LONG_ID;
    let global = n >= GLOBAL_ID;
    let mut stmt = conn.prepare_cached(
        "SELECT t.id, a.name FROM txn t JOIN account a ON a.id = t.account_id
           JOIN account me ON me.id = ?1
          WHERE t.external_id = ?2 AND t.account_id <> ?1
            AND (?4 = 1
                 OR ?3 = 1 AND (a.institution_id IS NULL OR me.institution_id IS NULL)
                 OR a.institution_id = me.institution_id)
          ORDER BY t.created_at, t.id LIMIT 1",
    )?;
    let mut rows = stmt.query(params![account_id, external_id, long as i64, global as i64])?;
    Ok(match rows.next()? {
        Some(r) => Some((r.get(0)?, r.get(1)?)),
        None => None,
    })
}
