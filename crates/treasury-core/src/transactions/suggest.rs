//! Autocompletar do lançamento: ao digitar a descrição, sugere lançamentos parecidos da
//! história da família com a categoria mais usada, o favorecido, o último valor e a conta,
//! para preencher o resto com um toque. Busca sem acento e sem maiúsculas, por início de
//! palavra; ordena por casamento, frequência e uso recente.

use std::collections::HashMap;

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::normalize_description;
use crate::error::Result;
use crate::ids::Id;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnSuggestArgs {
    pub text: String,
    /// Conta do lançamento sendo digitado (prioriza o que foi lançado nela).
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// `expense` ou `income` (prioriza o mesmo tipo).
    #[serde(default)]
    #[ts(optional)]
    pub kind: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnSuggestion {
    pub description: String,
    pub payee: Option<String>,
    pub category_id: Option<Id>,
    pub category_path: Option<String>,
    pub account_id: Id,
    pub kind: String,
    /// Último valor usado (centavos, com sinal).
    #[ts(type = "number")]
    pub last_amount: i64,
    pub last_date: String,
    #[ts(type = "number")]
    pub count: u32,
}

struct Acc {
    description: String,
    payee: Option<String>,
    account_id: String,
    kind: String,
    last_amount: i64,
    last_date: String,
    count: u32,
    categories: HashMap<String, u32>,
    same_account: u32,
}

fn score(name: &str, query: &str, count: u32, recent_rank: usize, same_account: u32) -> f64 {
    let words: Vec<&str> = name.split(' ').collect();
    let q_words: Vec<&str> = query.split(' ').filter(|w| !w.is_empty()).collect();
    let mut s = 0.0;
    if name.starts_with(query) {
        s += 6.0;
    }
    for q in &q_words {
        if words.iter().any(|w| w.starts_with(q)) {
            s += 3.0;
        } else if name.contains(q) {
            s += 1.0;
        } else {
            return 0.0;
        }
    }
    s + (count as f64).ln_1p() * 1.5
        + 2.0 / (1.0 + recent_rank as f64)
        + (same_account as f64).min(3.0)
}

pub fn suggest(conn: &Connection, a: &TxnSuggestArgs) -> Result<Vec<TxnSuggestion>> {
    let query = normalize_description(&a.text);
    if query.chars().count() < 2 {
        return Ok(vec![]);
    }
    let limit = a.limit.unwrap_or(8).clamp(1, 20) as usize;
    let mut stmt = conn.prepare(
        "SELECT description, payee, category_id, account_id, kind, amount, date FROM txn
         WHERE kind IN ('income', 'expense') AND description <> ''
           AND date >= date('now', '-2 years')
         ORDER BY date DESC, created_at DESC LIMIT 5000",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, String>(6)?,
        ))
    })?;
    // Agrupa pela descrição normalizada, na ordem do mais recente.
    let mut order: Vec<String> = vec![];
    let mut groups: HashMap<String, Acc> = HashMap::new();
    for row in rows {
        let (desc, payee, cat, acc, kind, amount, date) = row?;
        let key = normalize_description(&desc);
        if key.is_empty() {
            continue;
        }
        let g = groups.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            Acc {
                description: desc.clone(),
                payee: payee.clone(),
                account_id: acc.clone(),
                kind: kind.clone(),
                last_amount: amount,
                last_date: date.clone(),
                count: 0,
                categories: HashMap::new(),
                same_account: 0,
            }
        });
        g.count += 1;
        if let Some(c) = cat {
            *g.categories.entry(c).or_default() += 1;
        }
        if a.account_id.as_deref() == Some(acc.as_str()) {
            g.same_account += 1;
        }
    }
    let mut scored: Vec<(f64, &String)> = order
        .iter()
        .enumerate()
        .filter_map(|(rank, key)| {
            let g = &groups[key];
            let mut s = score(key, &query, g.count, rank, g.same_account);
            if s <= 0.0 {
                return None;
            }
            if a.kind.as_deref().is_some_and(|k| k == g.kind) {
                s += 1.0;
            }
            Some((s, key))
        })
        .collect();
    scored.sort_by(|x, y| y.0.total_cmp(&x.0));
    let mut out = vec![];
    for (_, key) in scored.into_iter().take(limit) {
        let g = &groups[key];
        let category_id = g
            .categories
            .iter()
            .max_by_key(|(id, n)| (**n, std::cmp::Reverse((*id).clone())))
            .map(|(id, _)| id.clone());
        let category_path = match &category_id {
            Some(id) => conn
                .query_row(
                    "WITH RECURSIVE up(id, name, parent_id, depth) AS (
                        SELECT id, name, parent_id, 0 FROM category WHERE id = ?1
                        UNION ALL SELECT c.id, c.name, c.parent_id, up.depth + 1 FROM category c JOIN up ON c.id = up.parent_id)
                     SELECT group_concat(name, ' › ') FROM (SELECT name FROM up ORDER BY depth DESC)",
                    params![id],
                    |r| r.get::<_, Option<String>>(0),
                )
                .ok()
                .flatten(),
            None => None,
        };
        out.push(TxnSuggestion {
            description: g.description.clone(),
            payee: g.payee.clone(),
            category_id,
            category_path,
            account_id: g.account_id.clone(),
            kind: g.kind.clone(),
            last_amount: g.last_amount,
            last_date: g.last_date.clone(),
            count: g.count,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transactions::testkit::Fx;

    #[test]
    fn suggests_by_word_start_with_most_used_category_and_last_amount() {
        let mut fx = Fx::new();
        let acc = fx.checking("Conta", 0);
        let mercado = fx.cat("Mercado");
        let today = crate::time::today_local();
        let day = |n: i64| crate::time::fmt_date(today - chrono::Duration::days(n));
        for (n, desc, amount) in [
            (40, "Padaria Pão Quente", -1200),
            (20, "Padaria Pão Quente", -1500),
            (5, "padaria pao quente", -1800),
            (3, "Pague Menos Farmácia", -4500),
            (2, "Supermercado Pão de Açúcar", -25000),
        ] {
            let id = fx.txn(&acc, &day(n), amount, desc);
            if desc.starts_with("Super") {
                fx.conn()
                    .execute(
                        "UPDATE txn SET category_id = ?2 WHERE id = ?1",
                        params![id, mercado],
                    )
                    .unwrap();
            }
        }
        let s = suggest(
            fx.conn(),
            &TxnSuggestArgs {
                text: "pada".into(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(s.len(), 1, "variações de acento e caixa viram uma sugestão");
        assert_eq!(s[0].count, 3);
        assert_eq!(s[0].last_amount, -1800, "o valor mais recente");
        // "pao" casa as duas pelo início de palavra; acento não importa.
        let s = suggest(
            fx.conn(),
            &TxnSuggestArgs {
                text: "pão".into(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(s.len(), 2);
        let sup = suggest(
            fx.conn(),
            &TxnSuggestArgs {
                text: "super".into(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(sup[0].category_id.as_deref(), Some(mercado.as_str()));
        assert!(sup[0].category_path.as_deref().unwrap().contains("Mercado"));
        assert!(
            suggest(
                fx.conn(),
                &TxnSuggestArgs {
                    text: "x".into(),
                    ..Default::default()
                }
            )
            .unwrap()
            .is_empty(),
            "uma letra só não sugere"
        );
    }
}
