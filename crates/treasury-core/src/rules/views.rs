//! Consulta mínima de lançamentos para as amostras de regras e a exportação.
//!
//! Provisório: o módulo `transactions` terá a consulta completa (`transactions.query` /
//! `TxnView`); depois da integração, [`load`] e [`filter_ids`] podem ser trocados por ela.

use std::collections::HashMap;

use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, OptionalExtension};

use crate::budget::Basis;
use crate::error::Result;
use crate::ids::Id;
use crate::import::text::fold;
use crate::transactions::{Split, Tag, TxnFilter, TxnKind, TxnStatus, TxnView};

/// Caminho de cada categoria: `Alimentação › Mercado`.
pub fn category_paths(conn: &Connection) -> Result<HashMap<Id, String>> {
    let mut stmt = conn.prepare("SELECT id, parent_id, name FROM category")?;
    let rows: Vec<(Id, Option<Id>, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let by_id: HashMap<&str, (Option<&str>, &str)> = rows
        .iter()
        .map(|(id, p, n)| (id.as_str(), (p.as_deref(), n.as_str())))
        .collect();
    let mut out = HashMap::with_capacity(rows.len());
    for (id, _, _) in &rows {
        let mut parts = Vec::new();
        let mut cur = Some(id.as_str());
        let mut guard = 0;
        while let Some(c) = cur {
            let Some((parent, name)) = by_id.get(c) else {
                break;
            };
            parts.push(*name);
            cur = *parent;
            guard += 1;
            if guard > 16 {
                break;
            }
        }
        parts.reverse();
        out.insert(id.clone(), parts.join(" › "));
    }
    Ok(out)
}

/// Lançamentos como `TxnView`, na ordem dos ids pedidos (ids inexistentes são omitidos).
pub fn load(conn: &Connection, ids: &[Id]) -> Result<Vec<TxnView>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let paths = category_paths(conn)?;
    let account_names: HashMap<Id, String> = conn
        .prepare("SELECT id, name FROM account")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut main = conn.prepare_cached(
        "SELECT t.id, t.account_id, t.date, t.amount, t.currency, t.kind, t.status, t.description, t.payee,
                t.notes, t.category_id, t.member_id, m.name, t.transfer_id,
                (SELECT o.account_id FROM txn o WHERE o.transfer_id = t.transfer_id AND o.id <> t.id LIMIT 1),
                t.statement_id, cs.due_date, t.installment_plan_id, t.installment_number, ip.installments,
                t.recurrence_id, t.import_id, t.external_id,
                (SELECT count(*) FROM attachment x WHERE x.txn_id = t.id), t.created_at, t.updated_at,
                t.card_plastic_id, cp.last4, t.orig_amount, t.orig_currency
         FROM txn t
         LEFT JOIN member m ON m.id = t.member_id
         LEFT JOIN card_statement cs ON cs.id = t.statement_id
         LEFT JOIN installment_plan ip ON ip.id = t.installment_plan_id
         LEFT JOIN card_plastic cp ON cp.id = t.card_plastic_id
         WHERE t.id = ?1",
    )?;
    let mut splits = conn.prepare_cached(
        "SELECT id, category_id, member_id, amount, notes FROM txn_split WHERE txn_id = ?1 ORDER BY created_at, id",
    )?;
    let mut tags = conn.prepare_cached(
        "SELECT g.id, g.name, g.color FROM txn_tag tt JOIN tag g ON g.id = tt.tag_id WHERE tt.txn_id = ?1 ORDER BY g.name",
    )?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        let row = main
            .query_row([id], |r| {
                let kind: String = r.get(5)?;
                let status: String = r.get(6)?;
                let account_id: Id = r.get(1)?;
                let category_id: Option<Id> = r.get(10)?;
                let counterpart: Option<Id> = r.get(14)?;
                Ok(TxnView {
                    id: r.get(0)?,
                    account_name: account_names.get(&account_id).cloned().unwrap_or_default(),
                    account_id,
                    date: r.get(2)?,
                    amount: r.get(3)?,
                    currency: r.get(4)?,
                    kind: TxnKind::parse(&kind).unwrap_or(TxnKind::Adjustment),
                    status: TxnStatus::parse(&status).unwrap_or(TxnStatus::Cleared),
                    description: r.get(7)?,
                    payee: r.get(8)?,
                    notes: r.get(9)?,
                    category_path: category_id.as_ref().and_then(|c| paths.get(c).cloned()),
                    category_id,
                    member_id: r.get(11)?,
                    member_name: r.get(12)?,
                    transfer_id: r.get(13)?,
                    counterpart_account_name: counterpart
                        .as_ref()
                        .and_then(|a| account_names.get(a).cloned()),
                    counterpart_account_id: counterpart,
                    statement_id: r.get(15)?,
                    statement_due_date: r.get(16)?,
                    installment_plan_id: r.get(17)?,
                    installment_number: r.get(18)?,
                    installment_total: r.get(19)?,
                    recurrence_id: r.get(20)?,
                    import_id: r.get(21)?,
                    external_id: r.get(22)?,
                    splits: vec![],
                    tags: vec![],
                    attachment_count: r.get(23)?,
                    created_at: r.get(24)?,
                    updated_at: r.get(25)?,
                    project_id: None,
                    project_name: None,
                    card_plastic_id: r.get(26)?,
                    card_last4: r.get(27)?,
                    orig_amount: r.get(28)?,
                    orig_currency: r.get(29)?,
                    loan_id: None,
                    loan_name: None,
                    loan_installment_id: None,
                    loan_installment_number: None,
                    operation: None,
                    receipt_id: None,
                })
            })
            .optional()?;
        let Some(mut v) = row else { continue };
        v.splits = splits
            .query_map([id], |r| {
                Ok(Split {
                    id: r.get(0)?,
                    category_id: r.get(1)?,
                    member_id: r.get(2)?,
                    amount: r.get(3)?,
                    notes: r.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        v.tags = tags
            .query_map([id], |r| {
                Ok(Tag {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        out.push(v);
    }
    Ok(out)
}

fn push_in(sql: &mut String, p: &mut Vec<SqlValue>, expr: &str, values: &[String]) {
    let marks = vec!["?"; values.len()].join(",");
    sql.push_str(&format!(" AND {expr} IN ({marks})"));
    p.extend(values.iter().cloned().map(SqlValue::Text));
}

/// Ids dos lançamentos que atendem ao filtro (subconjunto de `transactions.query`).
pub fn filter_ids(conn: &Connection, f: &TxnFilter) -> Result<Vec<Id>> {
    let date_expr = match f.basis.unwrap_or_default() {
        Basis::Cash => "COALESCE(cs.due_date, t.date)",
        Basis::Accrual => "t.date",
    };
    let mut sql = String::from(
        "SELECT t.id, t.description, t.payee, t.notes FROM txn t
         LEFT JOIN card_statement cs ON cs.id = t.statement_id WHERE 1 = 1",
    );
    let mut p: Vec<SqlValue> = Vec::new();
    if let Some(d) = &f.date_from {
        sql.push_str(&format!(" AND {date_expr} >= ?"));
        p.push(SqlValue::Text(d.clone()));
    }
    if let Some(d) = &f.date_to {
        sql.push_str(&format!(" AND {date_expr} <= ?"));
        p.push(SqlValue::Text(d.clone()));
    }
    if let Some(ids) = f.account_ids.as_ref().filter(|v| !v.is_empty()) {
        push_in(&mut sql, &mut p, "t.account_id", ids);
    }
    if let Some(ids) = f.category_ids.as_ref().filter(|v| !v.is_empty()) {
        let ids = if f.include_subcategories.unwrap_or(true) {
            crate::categories::with_descendants(conn, ids)?
        } else {
            ids.clone()
        };
        let marks = vec!["?"; ids.len()].join(",");
        sql.push_str(&format!(
            " AND (t.category_id IN ({marks}) OR EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id AND s.category_id IN ({marks})))"
        ));
        for _ in 0..2 {
            p.extend(ids.iter().cloned().map(SqlValue::Text));
        }
    }
    if let Some(ids) = f.member_ids.as_ref().filter(|v| !v.is_empty()) {
        let marks = vec!["?"; ids.len()].join(",");
        sql.push_str(&format!(
            " AND (t.member_id IN ({marks}) OR EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id AND s.member_id IN ({marks})))"
        ));
        for _ in 0..2 {
            p.extend(ids.iter().cloned().map(SqlValue::Text));
        }
    }
    if let Some(k) = f.kinds.as_ref().filter(|v| !v.is_empty()) {
        let v: Vec<String> = k.iter().map(|k| k.as_str().to_string()).collect();
        push_in(&mut sql, &mut p, "t.kind", &v);
    }
    if let Some(s) = f.statuses.as_ref().filter(|v| !v.is_empty()) {
        let v: Vec<String> = s.iter().map(|s| s.as_str().to_string()).collect();
        push_in(&mut sql, &mut p, "t.status", &v);
    }
    if let Some(a) = f.amount_min {
        sql.push_str(" AND abs(t.amount) >= ?");
        p.push(SqlValue::Integer(a));
    }
    if let Some(a) = f.amount_max {
        sql.push_str(" AND abs(t.amount) <= ?");
        p.push(SqlValue::Integer(a));
    }
    if let Some(ids) = f.tag_ids.as_ref().filter(|v| !v.is_empty()) {
        let marks = vec!["?"; ids.len()].join(",");
        sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM txn_tag tt WHERE tt.txn_id = t.id AND tt.tag_id IN ({marks}))"
        ));
        p.extend(ids.iter().cloned().map(SqlValue::Text));
    }
    if let Some(id) = &f.import_id {
        sql.push_str(" AND t.import_id = ?");
        p.push(SqlValue::Text(id.clone()));
    }
    if let Some(id) = &f.statement_id {
        sql.push_str(" AND t.statement_id = ?");
        p.push(SqlValue::Text(id.clone()));
    }
    match f.uncategorized {
        Some(true) => sql.push_str(
            " AND t.category_id IS NULL AND NOT EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)",
        ),
        Some(false) => sql.push_str(
            " AND (t.category_id IS NOT NULL OR EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id))",
        ),
        None => {}
    }
    sql.push_str(&crate::transactions::query::order_by(f.sort));
    crate::transactions::query::register_functions(conn);
    let needle = f.text.as_deref().map(fold).filter(|t| !t.is_empty());
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(p.iter()), |r| {
        Ok((
            r.get::<_, Id>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    let offset = f.offset.unwrap_or(0).max(0) as usize;
    let limit = f.limit.map(|l| l.max(0) as usize).unwrap_or(usize::MAX);
    let mut out = Vec::new();
    let mut skipped = 0usize;
    for row in rows {
        let (id, desc, payee, notes) = row?;
        if let Some(n) = &needle {
            let hay = format!(
                "{} {} {}",
                fold(&desc),
                fold(payee.as_deref().unwrap_or("")),
                fold(notes.as_deref().unwrap_or(""))
            );
            if !hay.contains(n.as_str()) {
                continue;
            }
        }
        if skipped < offset {
            skipped += 1;
            continue;
        }
        if out.len() >= limit {
            break;
        }
        out.push(id);
    }
    Ok(out)
}
