//! Consulta de lançamentos: filtro completo, paginação, totais por moeda e montagem de
//! `TxnView` em lote (divisões e tags carregadas de uma vez para a página inteira).

use std::collections::HashMap;

use rusqlite::functions::FunctionFlags;
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, params_from_iter};

use super::util::placeholders;
use super::{
    CurrencyTotal, Split, Tag, TxnFilter, TxnKind, TxnPage, TxnSort, TxnStatus, TxnView,
    fold_accent,
};
use crate::budget::Basis;
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::Id;

/// Página padrão e máxima de `query`.
pub const DEFAULT_LIMIT: i64 = 100;
pub const MAX_LIMIT: i64 = 2000;

/// Lotes de ids por consulta `IN (...)`.
const CHUNK: usize = 500;

/// Texto comparável na busca: minúsculas e sem acentos (mantém dígitos e pontuação).
pub fn search_fold(s: &str) -> String {
    s.chars()
        .map(fold_accent)
        .collect::<String>()
        .to_lowercase()
}

/// Registra `tr_fold(texto)` na conexão. Idempotente: o SQLite recusa redefinir uma
/// função com consultas em andamento (SQLITE_BUSY), e nesse caso ela já existe.
pub(crate) fn register_functions(conn: &Connection) {
    let _ = conn.create_scalar_function(
        "tr_fold",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let s: Option<String> = ctx.get(0)?;
            Ok(s.map(|s| search_fold(&s)))
        },
    );
}

const VIEW_SELECT: &str = "SELECT t.id, t.account_id, a.name, t.date, t.amount, t.currency, t.kind, t.status,
       t.description, t.payee, t.notes, t.category_id, t.member_id, m.name, t.transfer_id,
       (SELECT o.account_id FROM txn o WHERE o.transfer_id = t.transfer_id AND o.id <> t.id ORDER BY o.id LIMIT 1),
       (SELECT oa.name FROM txn o JOIN account oa ON oa.id = o.account_id
         WHERE o.transfer_id = t.transfer_id AND o.id <> t.id ORDER BY o.id LIMIT 1),
       t.statement_id, s.due_date, t.installment_plan_id, t.installment_number, ip.installments,
       t.recurrence_id, t.import_id, t.external_id,
       (SELECT count(*) FROM attachment x WHERE x.txn_id = t.id),
       t.created_at, t.updated_at, t.project_id, pr.name, t.card_plastic_id, cp.last4, t.orig_amount, t.orig_currency,
       (SELECT rp.receipt_id FROM receipt_payment rp WHERE rp.txn_id = t.id ORDER BY rp.created_at LIMIT 1)
  FROM txn t
  JOIN account a ON a.id = t.account_id
  LEFT JOIN member m ON m.id = t.member_id
  LEFT JOIN card_statement s ON s.id = t.statement_id
  LEFT JOIN installment_plan ip ON ip.id = t.installment_plan_id
  LEFT JOIN project pr ON pr.id = t.project_id
  LEFT JOIN card_plastic cp ON cp.id = t.card_plastic_id";

fn row_to_view(r: &rusqlite::Row<'_>, paths: &HashMap<Id, String>) -> rusqlite::Result<TxnView> {
    let kind: String = r.get(6)?;
    let status: String = r.get(7)?;
    let category_id: Option<Id> = r.get(11)?;
    let conv = |e: Error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    };
    Ok(TxnView {
        id: r.get(0)?,
        account_id: r.get(1)?,
        account_name: r.get(2)?,
        date: r.get(3)?,
        amount: r.get(4)?,
        currency: r.get(5)?,
        kind: TxnKind::parse(&kind).map_err(conv)?,
        status: TxnStatus::parse(&status).map_err(conv)?,
        description: r.get(8)?,
        payee: r.get(9)?,
        notes: r.get(10)?,
        category_path: category_id.as_ref().and_then(|c| paths.get(c).cloned()),
        category_id,
        member_id: r.get(12)?,
        member_name: r.get(13)?,
        transfer_id: r.get(14)?,
        counterpart_account_id: r.get(15)?,
        counterpart_account_name: r.get(16)?,
        statement_id: r.get(17)?,
        statement_due_date: r.get(18)?,
        installment_plan_id: r.get(19)?,
        installment_number: r.get(20)?,
        installment_total: r.get(21)?,
        recurrence_id: r.get(22)?,
        import_id: r.get(23)?,
        external_id: r.get(24)?,
        splits: vec![],
        tags: vec![],
        attachment_count: r.get(25)?,
        created_at: r.get(26)?,
        updated_at: r.get(27)?,
        project_id: r.get(28)?,
        project_name: r.get(29)?,
        card_plastic_id: r.get(30)?,
        card_last4: r.get(31)?,
        orig_amount: r.get(32)?,
        orig_currency: r.get(33)?,
        loan_id: None,
        loan_name: None,
        loan_installment_id: None,
        loan_installment_number: None,
        operation: None,
        receipt_id: r.get(34)?,
    })
}

/// Preenche divisões, tags e a parcela de dívida paga de vários lançamentos com três
/// consultas por lote.
fn attach_details(conn: &Connection, views: &mut [TxnView]) -> Result<()> {
    if views.is_empty() {
        return Ok(());
    }
    let index: HashMap<String, usize> = views
        .iter()
        .enumerate()
        .map(|(i, v)| (v.id.clone(), i))
        .collect();
    let ids: Vec<String> = views.iter().map(|v| v.id.clone()).collect();
    for chunk in ids.chunks(CHUNK) {
        let ph = placeholders(chunk.len());
        let mut stmt = conn.prepare(&format!(
            "SELECT txn_id, id, category_id, member_id, amount, notes FROM txn_split
              WHERE txn_id IN ({ph}) ORDER BY txn_id, rowid"
        ))?;
        let mut rows = stmt.query(params_from_iter(chunk.iter()))?;
        while let Some(r) = rows.next()? {
            let txn_id: String = r.get(0)?;
            if let Some(&i) = index.get(&txn_id) {
                views[i].splits.push(Split {
                    id: r.get(1)?,
                    category_id: r.get(2)?,
                    member_id: r.get(3)?,
                    amount: r.get(4)?,
                    notes: r.get(5)?,
                });
            }
        }
        let mut stmt = conn.prepare(&format!(
            "SELECT tt.txn_id, g.id, g.name, g.color FROM txn_tag tt JOIN tag g ON g.id = tt.tag_id
              WHERE tt.txn_id IN ({ph}) ORDER BY g.name COLLATE NOCASE"
        ))?;
        let mut rows = stmt.query(params_from_iter(chunk.iter()))?;
        while let Some(r) = rows.next()? {
            let txn_id: String = r.get(0)?;
            if let Some(&i) = index.get(&txn_id) {
                views[i].tags.push(Tag {
                    id: r.get(1)?,
                    name: r.get(2)?,
                    color: r.get(3)?,
                });
            }
        }
        // Parcela paga pelo lançamento: a de menor número, se uma antecipação pagou várias.
        let mut stmt = conn.prepare(&format!(
            "SELECT li.txn_id, l.id, l.name, li.id, li.number
               FROM loan_installment li JOIN loan l ON l.id = li.loan_id
              WHERE li.txn_id IN ({ph}) ORDER BY li.txn_id, li.number"
        ))?;
        let mut rows = stmt.query(params_from_iter(chunk.iter()))?;
        while let Some(r) = rows.next()? {
            let txn_id: String = r.get(0)?;
            if let Some(&i) = index.get(&txn_id)
                && views[i].loan_installment_id.is_none()
            {
                let v = &mut views[i];
                v.loan_id = Some(r.get(1)?);
                v.loan_name = Some(r.get(2)?);
                v.loan_installment_id = Some(r.get(3)?);
                v.loan_installment_number = Some(r.get(4)?);
            }
        }
        let mut stmt = conn.prepare(&format!(
            "SELECT txn_id FROM txn_link WHERE txn_id IN ({ph})"
        ))?;
        let linked: Vec<String> = stmt
            .query_map(params_from_iter(chunk.iter()), |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for txn_id in linked {
            if let Some(&i) = index.get(&txn_id) {
                views[i].operation = super::operations::get(conn, &txn_id)?;
            }
        }
    }
    Ok(())
}

/// Lançamentos completos, na ordem dos ids pedidos (ids inexistentes são ignorados).
pub fn views(conn: &Connection, ids: &[Id]) -> Result<Vec<TxnView>> {
    let paths = crate::categories::path_map(conn)?;
    let mut found: HashMap<String, TxnView> = HashMap::with_capacity(ids.len());
    for chunk in ids.chunks(CHUNK) {
        let sql = format!(
            "{VIEW_SELECT} WHERE t.id IN ({})",
            placeholders(chunk.len())
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params_from_iter(chunk.iter()), |r| row_to_view(r, &paths))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for v in rows {
            found.insert(v.id.clone(), v);
        }
    }
    let mut out: Vec<TxnView> = ids.iter().filter_map(|id| found.remove(id)).collect();
    attach_details(conn, &mut out)?;
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> Result<TxnView> {
    views(conn, &[id.to_string()])?
        .pop()
        .ok_or_else(|| Error::not_found(super::util::txn_ref(id)))
}

// ---------------------------------------------------------------------------
// Filtro

#[derive(Default)]
struct Where {
    clauses: Vec<String>,
    params: Vec<SqlValue>,
}

impl Where {
    fn push(&mut self, clause: impl Into<String>, params: impl IntoIterator<Item = SqlValue>) {
        self.clauses.push(clause.into());
        self.params.extend(params);
    }
    fn sql(&self) -> String {
        if self.clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.clauses.join(" AND "))
        }
    }
}

fn text_values(ids: &[String]) -> Vec<SqlValue> {
    ids.iter().map(|s| SqlValue::Text(s.clone())).collect()
}

fn non_empty(v: &Option<Vec<String>>) -> Option<&[String]> {
    v.as_deref().filter(|v| !v.is_empty())
}

fn like_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn build_where(conn: &Connection, f: &TxnFilter) -> Result<Where> {
    let mut w = Where::default();
    for d in [&f.date_from, &f.date_to].into_iter().flatten() {
        crate::time::parse_date(d)?;
    }
    if let (Some(a), Some(b)) = (&f.date_from, &f.date_to)
        && a > b
    {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-periodo-invertido",
        )));
    }
    match f.basis.unwrap_or_default() {
        Basis::Accrual => {
            if let Some(d) = &f.date_from {
                w.push("t.date >= ?", [SqlValue::Text(d.clone())]);
            }
            if let Some(d) = &f.date_to {
                w.push("t.date <= ?", [SqlValue::Text(d.clone())]);
            }
        }
        Basis::Cash => {
            // Caixa: compras no cartão contam no vencimento da fatura; o resto, na data.
            let mut own = vec!["(t.statement_id IS NULL OR t.kind = 'card_payment')".to_string()];
            let mut due = Vec::new();
            let mut p_own = Vec::new();
            let mut p_due = Vec::new();
            if let Some(d) = &f.date_from {
                own.push("t.date >= ?".into());
                due.push("due_date >= ?");
                p_own.push(SqlValue::Text(d.clone()));
                p_due.push(SqlValue::Text(d.clone()));
            }
            if let Some(d) = &f.date_to {
                own.push("t.date <= ?".into());
                due.push("due_date <= ?");
                p_own.push(SqlValue::Text(d.clone()));
                p_due.push(SqlValue::Text(d.clone()));
            }
            if !due.is_empty() {
                let clause = format!(
                    "(({}) OR (t.kind <> 'card_payment' AND t.statement_id IN \
                     (SELECT id FROM card_statement WHERE {})))",
                    own.join(" AND "),
                    due.join(" AND ")
                );
                p_own.extend(p_due);
                w.push(clause, p_own);
            }
        }
    }
    if let Some(ids) = non_empty(&f.account_ids) {
        w.push(
            format!("t.account_id IN ({})", placeholders(ids.len())),
            text_values(ids),
        );
    }
    if let Some(ids) = non_empty(&f.category_ids) {
        let ids = if f.include_subcategories.unwrap_or(true) {
            crate::categories::with_descendants(conn, ids)?
        } else {
            ids.to_vec()
        };
        if ids.is_empty() {
            w.push("0", []);
        } else {
            let ph = placeholders(ids.len());
            let mut p = text_values(&ids);
            p.extend(text_values(&ids));
            w.push(
                format!(
                    "(t.category_id IN ({ph}) OR t.id IN (SELECT txn_id FROM txn_split WHERE category_id IN ({ph})))"
                ),
                p,
            );
        }
    }
    if let Some(ids) = non_empty(&f.member_ids) {
        let ph = placeholders(ids.len());
        let mut p = text_values(ids);
        p.extend(text_values(ids));
        w.push(
            format!(
                "(t.member_id IN ({ph}) OR t.id IN (SELECT txn_id FROM txn_split WHERE member_id IN ({ph})))"
            ),
            p,
        );
    }
    if let Some(kinds) = f.kinds.as_deref().filter(|k| !k.is_empty()) {
        w.push(
            format!("t.kind IN ({})", placeholders(kinds.len())),
            kinds.iter().map(|k| SqlValue::Text(k.as_str().into())),
        );
    }
    if let Some(st) = f.statuses.as_deref().filter(|k| !k.is_empty()) {
        w.push(
            format!("t.status IN ({})", placeholders(st.len())),
            st.iter().map(|k| SqlValue::Text(k.as_str().into())),
        );
    }
    if let Some(text) = &f.text {
        // Cada palavra precisa aparecer na descrição, no favorecido ou nas notas.
        for word in search_fold(text).split_whitespace() {
            w.push(
                "tr_fold(t.description || char(1) || COALESCE(t.payee, '') || char(1) || COALESCE(t.notes, '')) \
                 LIKE ? ESCAPE '\\'",
                [SqlValue::Text(format!("%{}%", like_escape(word)))],
            );
        }
    }
    if let Some(v) = f.amount_min {
        w.push("abs(t.amount) >= ?", [SqlValue::Integer(v.abs())]);
    }
    if let Some(v) = f.amount_max {
        w.push("abs(t.amount) <= ?", [SqlValue::Integer(v.abs())]);
    }
    if let Some(ids) = non_empty(&f.tag_ids) {
        w.push(
            format!(
                "EXISTS (SELECT 1 FROM txn_tag tt WHERE tt.txn_id = t.id AND tt.tag_id IN ({}))",
                placeholders(ids.len())
            ),
            text_values(ids),
        );
    }
    if let Some(ids) = non_empty(&f.project_ids) {
        w.push(
            format!("t.project_id IN ({})", placeholders(ids.len())),
            text_values(ids),
        );
    }
    if let Some(id) = &f.import_id {
        w.push("t.import_id = ?", [SqlValue::Text(id.clone())]);
    }
    if let Some(id) = &f.statement_id {
        w.push("t.statement_id = ?", [SqlValue::Text(id.clone())]);
    }
    if let Some(u) = f.uncategorized {
        // Transferência de uma perna só (PIX para terceiros) também espera categoria.
        let cond = "((t.kind IN ('income', 'expense') OR (t.kind = 'transfer' AND t.transfer_id IS NULL))
                    AND t.category_id IS NULL AND (
                      NOT EXISTS (SELECT 1 FROM txn_split sp WHERE sp.txn_id = t.id)
                      OR EXISTS (SELECT 1 FROM txn_split sp WHERE sp.txn_id = t.id AND sp.category_id IS NULL)))";
        w.push(
            if u {
                cond.to_string()
            } else {
                format!("NOT {cond}")
            },
            [],
        );
    }
    Ok(w)
}

/// Expressões de ordenação por texto (sem acentos nem maiúsculas; `tr_fold` precisa de
/// [`register_functions`]). Só usam `t`, para valer em qualquer consulta sobre `txn t`.
const BY_DESCRIPTION: &str = "tr_fold(t.description)";
const BY_ACCOUNT: &str = "tr_fold((SELECT name FROM account WHERE id = t.account_id))";
const BY_CATEGORY: &str =
    "tr_fold((SELECT CASE WHEN p.name IS NULL THEN c.name ELSE p.name || ' ' || c.name END
     FROM category c LEFT JOIN category p ON p.id = c.parent_id WHERE c.id = t.category_id))";
const BY_MEMBER: &str = "tr_fold((SELECT name FROM member WHERE id = t.member_id))";
const BY_STATUS: &str = "CASE t.status WHEN 'planned' THEN 0 WHEN 'cleared' THEN 1 ELSE 2 END";

pub(crate) fn order_by(sort: Option<TxnSort>) -> String {
    // Texto: vazios por último nos dois sentidos; empate pela data mais recente.
    let text = |expr: &str, dir: &str| {
        format!(" ORDER BY ({expr}) IS NULL, {expr} {dir}, t.date DESC, t.id DESC")
    };
    match sort.unwrap_or_default() {
        TxnSort::DateDesc => " ORDER BY t.date DESC, t.created_at DESC, t.id DESC".into(),
        TxnSort::DateAsc => " ORDER BY t.date ASC, t.created_at ASC, t.id ASC".into(),
        TxnSort::AmountDesc => " ORDER BY t.amount DESC, t.date DESC, t.id DESC".into(),
        TxnSort::AmountAsc => " ORDER BY t.amount ASC, t.date DESC, t.id DESC".into(),
        TxnSort::DescriptionAsc => text(BY_DESCRIPTION, "ASC"),
        TxnSort::DescriptionDesc => text(BY_DESCRIPTION, "DESC"),
        TxnSort::AccountAsc => text(BY_ACCOUNT, "ASC"),
        TxnSort::AccountDesc => text(BY_ACCOUNT, "DESC"),
        TxnSort::CategoryAsc => text(BY_CATEGORY, "ASC"),
        TxnSort::CategoryDesc => text(BY_CATEGORY, "DESC"),
        TxnSort::MemberAsc => text(BY_MEMBER, "ASC"),
        TxnSort::MemberDesc => text(BY_MEMBER, "DESC"),
        TxnSort::StatusAsc => format!(" ORDER BY {BY_STATUS} ASC, t.date DESC, t.id DESC"),
        TxnSort::StatusDesc => format!(" ORDER BY {BY_STATUS} DESC, t.date DESC, t.id DESC"),
    }
}

/// Consulta paginada com totais (por moeda) de todos os resultados do filtro.
pub fn query(conn: &Connection, f: &TxnFilter) -> Result<TxnPage> {
    register_functions(conn);
    let w = build_where(conn, f)?;
    let where_sql = w.sql();

    let mut stmt = conn.prepare(&format!(
        "SELECT t.currency,
                COALESCE(SUM(CASE WHEN t.amount > 0 THEN t.amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN t.amount < 0 THEN -t.amount ELSE 0 END), 0),
                count(*)
           FROM txn t{where_sql} GROUP BY t.currency ORDER BY t.currency"
    ))?;
    let mut total = 0i64;
    let totals = stmt
        .query_map(params_from_iter(w.params.iter()), |r| {
            Ok((
                CurrencyTotal {
                    currency: r.get(0)?,
                    inflow: r.get(1)?,
                    outflow: r.get(2)?,
                    net: r.get::<_, i64>(1)? - r.get::<_, i64>(2)?,
                },
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .map(|(t, n)| {
            total += n;
            t
        })
        .collect();

    let limit = f.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = f.offset.unwrap_or(0).max(0);
    let paths = crate::categories::path_map(conn)?;
    let sql = format!(
        "{VIEW_SELECT}{where_sql}{} LIMIT ? OFFSET ?",
        order_by(f.sort)
    );
    let mut params = w.params.clone();
    params.push(SqlValue::Integer(limit));
    params.push(SqlValue::Integer(offset));
    let mut stmt = conn.prepare(&sql)?;
    let mut items = stmt
        .query_map(params_from_iter(params.iter()), |r| row_to_view(r, &paths))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    attach_details(conn, &mut items)?;
    Ok(TxnPage {
        items,
        total,
        totals,
    })
}

/// Id e descrição de todos os lançamentos do filtro, sem paginação (ações em lote pela
/// descrição).
pub(crate) fn query_descriptions(conn: &Connection, f: &TxnFilter) -> Result<Vec<(Id, String)>> {
    register_functions(conn);
    let w = build_where(conn, f)?;
    let sql = format!(
        "SELECT t.id, t.description FROM txn t{}{}",
        w.sql(),
        order_by(f.sort)
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_from_iter(w.params.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Ids de todos os lançamentos do filtro, na ordem pedida, sem paginação
/// (regras, exportação, ações em lote).
pub fn query_ids(conn: &Connection, f: &TxnFilter) -> Result<Vec<Id>> {
    register_functions(conn);
    let w = build_where(conn, f)?;
    let sql = format!("SELECT t.id FROM txn t{}{}", w.sql(), order_by(f.sort));
    let mut stmt = conn.prepare(&sql)?;
    let ids = stmt
        .query_map(params_from_iter(w.params.iter()), |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transactions::testkit::Fx;

    fn plan(fx: &Fx, f: &TxnFilter) -> String {
        register_functions(fx.conn());
        let w = build_where(fx.conn(), f).unwrap();
        let sql = format!(
            "EXPLAIN QUERY PLAN {VIEW_SELECT}{}{} LIMIT 100",
            w.sql(),
            order_by(f.sort)
        );
        let mut stmt = fx.conn().prepare(&sql).unwrap();
        let rows: Vec<String> = stmt
            .query_map(params_from_iter(w.params.iter()), |r| r.get::<_, String>(3))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        rows.join("\n")
    }

    #[test]
    fn filters_use_existing_indexes() {
        let mut fx = Fx::new();
        let acc = fx.checking("Corrente", 0);
        let by_date = plan(
            &fx,
            &TxnFilter {
                date_from: Some("2026-10-01".into()),
                date_to: Some("2026-10-31".into()),
                ..Default::default()
            },
        );
        assert!(by_date.contains("INDEX txn_date"), "{by_date}");
        let by_account = plan(
            &fx,
            &TxnFilter {
                account_ids: Some(vec![acc]),
                date_from: Some("2026-10-01".into()),
                ..Default::default()
            },
        );
        assert!(by_account.contains("INDEX txn_"), "{by_account}");
        let by_statement = plan(
            &fx,
            &TxnFilter {
                statement_id: Some("x".into()),
                ..Default::default()
            },
        );
        assert!(
            by_statement.contains("INDEX txn_statement"),
            "{by_statement}"
        );
        // Divisões, tags e anexos são buscados por índice, não por varredura.
        assert!(!by_date.contains("SCAN x"), "{by_date}");
    }

    #[test]
    fn text_search_works_with_statements_in_progress() {
        let mut fx = Fx::new();
        let acc = fx.checking("Corrente", 0);
        fx.txn(&acc, "2026-10-01", -100, "Pão de queijo");
        let f = TxnFilter {
            text: Some("PAO".into()),
            ..Default::default()
        };
        assert_eq!(query(fx.conn(), &f).unwrap().total, 1);
        let mut stmt = fx.conn().prepare("SELECT id FROM account").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while rows.next().unwrap().is_some() {
            assert_eq!(query(fx.conn(), &f).unwrap().total, 1);
            assert_eq!(query_ids(fx.conn(), &f).unwrap().len(), 1);
        }
    }
}
