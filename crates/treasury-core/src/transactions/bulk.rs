//! Ações em lote da tela 03 que não cabem em `write`: achar os lançamentos com a mesma
//! descrição (para categorizar de uma vez) e pôr ou tirar tags de vários lançamentos.

use std::collections::HashSet;

use rusqlite::{Connection, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::TxnFilter;
use super::query::{query_descriptions, search_fold};
use super::util::dedup_ids;
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::Id;
use crate::time::now_utc;

/// Lançamentos com a mesma descrição dos escolhidos, dentro de um filtro.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SameDescriptionArgs {
    pub ids: Vec<Id>,
    /// Onde procurar (padrão: tudo). Paginação é ignorada.
    #[serde(default)]
    #[ts(optional)]
    pub filter: Option<TxnFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SameDescription {
    /// Os escolhidos e os que têm a mesma descrição.
    pub ids: Vec<Id>,
    /// Descrições procuradas, como estão no primeiro lançamento de cada uma.
    pub descriptions: Vec<String>,
}

/// Descrição comparável: sem maiúsculas, acentos e espaços repetidos.
fn key(s: &str) -> String {
    search_fold(s)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn same_description(conn: &Connection, a: &SameDescriptionArgs) -> Result<SameDescription> {
    let ids = dedup_ids(&a.ids);
    if ids.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-escolha-um")));
    }
    let mut keys = HashSet::new();
    let mut descriptions = vec![];
    for id in &ids {
        let d: String = conn
            .query_row("SELECT description FROM txn WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .map_err(|_| Error::not_found(super::util::txn_ref(id)))?;
        let k = key(&d);
        if !k.is_empty() && keys.insert(k) {
            descriptions.push(d.trim().to_string());
        }
    }
    let mut filter = a.filter.clone().unwrap_or_default();
    filter.limit = None;
    filter.offset = None;
    let mut out = ids;
    let mut seen: HashSet<Id> = out.iter().cloned().collect();
    for (id, d) in query_descriptions(conn, &filter)? {
        if keys.contains(&key(&d)) && seen.insert(id.clone()) {
            out.push(id);
        }
    }
    Ok(SameDescription {
        ids: out,
        descriptions,
    })
}

/// Põe e tira tags (por nome) de vários lançamentos.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EditTagsArgs {
    pub ids: Vec<Id>,
    /// Tags a pôr (criadas se não existirem).
    #[serde(default)]
    #[ts(optional)]
    pub add: Option<Vec<String>>,
    /// Tags a tirar.
    #[serde(default)]
    #[ts(optional)]
    pub remove: Option<Vec<String>>,
}

pub fn edit_tags(tx: &Transaction<'_>, a: &EditTagsArgs) -> Result<usize> {
    let names = |v: &Option<Vec<String>>| -> Vec<String> {
        v.iter()
            .flatten()
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .collect()
    };
    let (add, remove) = (names(&a.add), names(&a.remove));
    if add.is_empty() && remove.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-tags-lote")));
    }
    let add_ids = add
        .iter()
        .map(|n| super::ensure_tag(tx, n))
        .collect::<Result<Vec<_>>>()?;
    let mut n = 0;
    for id in dedup_ids(&a.ids) {
        let exists: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM txn WHERE id = ?1)",
            [&id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(Error::not_found(super::util::txn_ref(&id)));
        }
        let mut changed = 0;
        for t in &add_ids {
            changed += tx.execute(
                "INSERT OR IGNORE INTO txn_tag (txn_id, tag_id) VALUES (?1, ?2)",
                params![id, t],
            )?;
        }
        for name in &remove {
            changed += tx.execute(
                "DELETE FROM txn_tag WHERE txn_id = ?1
                   AND tag_id IN (SELECT id FROM tag WHERE name = ?2 COLLATE NOCASE)",
                params![id, name],
            )?;
        }
        if changed > 0 {
            tx.execute(
                "UPDATE txn SET updated_at = ?2 WHERE id = ?1",
                params![id, now_utc()],
            )?;
            n += 1;
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transactions::testkit::Fx;

    #[test]
    fn same_description_member_and_tags_in_bulk() {
        let mut fx = Fx::new();
        let acc = fx.checking("Conta", 0);
        let other = fx.checking("Outra", 0);
        let a = fx.txn(&acc, "2026-08-03", -3_000, "UBER *TRIP");
        let b = fx.txn(&acc, "2026-09-12", -2_500, "Uber  *trip ");
        let c = fx.txn(&other, "2026-09-20", -1_800, "uber *trip");
        let _x = fx.txn(&acc, "2026-09-21", -900, "Uber Eats");

        // Same description (ignoring case, accents and spaces), in the filter accounts.
        let r = same_description(
            fx.conn(),
            &SameDescriptionArgs {
                ids: vec![a.clone()],
                filter: Some(TxnFilter {
                    account_ids: Some(vec![acc.clone()]),
                    limit: Some(1),
                    ..Default::default()
                }),
            },
        )
        .unwrap();
        assert_eq!(r.ids.len(), 2);
        assert!(r.ids.contains(&b) && !r.ids.contains(&c));
        assert_eq!(r.descriptions, vec!["UBER *TRIP".to_string()]);
        let all = same_description(
            fx.conn(),
            &SameDescriptionArgs {
                ids: vec![a.clone()],
                filter: None,
            },
        )
        .unwrap();
        assert_eq!(all.ids.len(), 3);

        // Holder in bulk.
        let ana = fx.member("Ana");
        let n = fx
            .write(|tx| crate::transactions::set_member(tx, &all.ids, Some(&ana)))
            .unwrap();
        assert_eq!(n, 3);
        let v = crate::transactions::get(fx.conn(), &c).unwrap();
        assert_eq!(v.member_id.as_deref(), Some(ana.as_str()));

        // Tags in bulk: add and remove.
        let n = fx
            .write(|tx| {
                edit_tags(
                    tx,
                    &EditTagsArgs {
                        ids: all.ids.clone(),
                        add: Some(vec!["Transporte".into(), "Trabalho".into()]),
                        remove: None,
                    },
                )
            })
            .unwrap();
        assert_eq!(n, 3);
        let n = fx
            .write(|tx| {
                edit_tags(
                    tx,
                    &EditTagsArgs {
                        ids: vec![a.clone(), b.clone()],
                        add: None,
                        remove: Some(vec!["trabalho".into()]),
                    },
                )
            })
            .unwrap();
        assert_eq!(n, 2);
        let tags = |id: &str| {
            crate::transactions::get(fx.conn(), id)
                .unwrap()
                .tags
                .into_iter()
                .map(|t| t.name)
                .collect::<Vec<_>>()
        };
        assert_eq!(tags(&a), vec!["Transporte"]);
        assert_eq!(tags(&c), vec!["Trabalho", "Transporte"]);
        assert!(
            fx.write(|tx| edit_tags(
                tx,
                &EditTagsArgs {
                    ids: vec![a.clone()],
                    add: None,
                    remove: None
                }
            ))
            .is_err()
        );
    }
}
