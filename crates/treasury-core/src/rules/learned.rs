//! Regras aprendidas (docs/learned-rules.md): quando a pessoa já deu a mesma categoria três
//! vezes ao mesmo estabelecimento, o app oferece a regra pronta. Categorizar é a tarefa mais
//! repetitiva do Treasury; a regra só é **oferecida**, nunca criada sozinha.
//!
//! Uma sugestão aparece quando, nos últimos 12 meses:
//!
//! - pelo menos [`MIN_SAME`] lançamentos de receita ou despesa com o mesmo trecho significativo
//!   da descrição (`significant_run`) têm categoria, e **todos a mesma**;
//! - nenhuma regra ativa já define a categoria de um lançamento desses;
//! - a pessoa não dispensou o trecho (ajuste `rules.learned_ignored`).
//!
//! A regra oferecida é a mesma de "criar regra a partir do lançamento" (`suggest_from_txn`),
//! que escolhe o menor trecho que não pega lançamentos de outra categoria.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{RuleInput, RuleSet, facts_for, significant_run, suggest_from_txn};
use crate::error::{Error, Result};
use crate::ids::Id;
use crate::time::fmt_date;

/// Lançamentos com a mesma categoria para o app oferecer a regra.
pub const MIN_SAME: i64 = 3;
/// Janela dos lançamentos examinados.
pub const WINDOW_DAYS: i64 = 365;
/// Máximo de sugestões devolvidas.
pub const LIMIT: usize = 20;
/// Ajuste com os trechos que a pessoa dispensou.
pub const IGNORED_SETTING: &str = "rules.learned_ignored";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LearnedRule {
    /// O trecho da descrição que identifica o estabelecimento (minúsculas, sem acentos).
    pub key: String,
    pub category_id: Id,
    /// Caminho da categoria (`Alimentação › Mercado`).
    pub category_path: String,
    /// Lançamentos já com essa categoria.
    #[ts(type = "number")]
    pub count: i64,
    /// Lançamentos com o mesmo trecho ainda sem categoria (a regra resolveria agora).
    #[ts(type = "number")]
    pub uncategorized: i64,
    /// Um lançamento de exemplo (o mais recente com categoria).
    pub sample_txn_id: Id,
    pub sample_description: String,
    /// A regra pronta para `rules.create`.
    pub rule: RuleInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LearnedIgnoreArgs {
    pub key: String,
}

#[derive(Default)]
struct Group {
    by_category: BTreeMap<Id, i64>,
    uncategorized: i64,
    /// O mais recente com categoria.
    sample: Option<(Id, String)>,
}

fn ignored(conn: &Connection) -> Result<BTreeSet<String>> {
    crate::settings::get_or(conn, IGNORED_SETTING, BTreeSet::new())
}

/// Não oferecer mais a regra para este trecho.
pub fn ignore(conn: &Connection, key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() || key.len() > 120 {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-lancamentos-regra-erro-sem-palavras",
        )));
    }
    let mut all = ignored(conn)?;
    all.insert(key.to_string());
    crate::settings::set(conn, IGNORED_SETTING, &all)
}

/// As regras que o app pode oferecer na data `today`: primeiro as que resolvem mais
/// lançamentos sem categoria, depois as mais repetidas.
pub fn learned(conn: &Connection, today: NaiveDate) -> Result<Vec<LearnedRule>> {
    let since = fmt_date(today - Duration::days(WINDOW_DAYS));
    let mut stmt = conn.prepare(
        "SELECT t.id, t.description, t.category_id FROM txn t
          WHERE t.kind IN ('income', 'expense') AND t.status != 'planned'
            AND t.date BETWEEN ?1 AND ?2 AND t.transfer_id IS NULL
            AND NOT EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)
          ORDER BY t.date DESC, t.created_at DESC, t.id DESC",
    )?;
    let rows: Vec<(Id, String, Option<Id>)> = stmt
        .query_map([&since, &fmt_date(today)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?;

    let mut groups: BTreeMap<String, Group> = BTreeMap::new();
    for (id, description, category) in rows {
        let key = significant_run(&description).join(" ");
        // Trecho curto demais pega coisa que não é o mesmo estabelecimento.
        if key.len() < 4 {
            continue;
        }
        let g = groups.entry(key).or_default();
        match category {
            Some(c) => {
                *g.by_category.entry(c).or_default() += 1;
                if g.sample.is_none() {
                    g.sample = Some((id, description));
                }
            }
            None => g.uncategorized += 1,
        }
    }

    let off = ignored(conn)?;
    let rules = RuleSet::load_active(conn)?;
    let paths = crate::categories::path_map(conn)?;
    let mut out = vec![];
    for (key, g) in groups {
        // Sempre a mesma categoria, pelo menos três vezes.
        if g.by_category.len() != 1 || off.contains(&key) {
            continue;
        }
        let Some((category_id, count)) = g.by_category.into_iter().next() else {
            continue;
        };
        let Some((sample_txn_id, sample_description)) = g.sample else {
            continue;
        };
        if count < MIN_SAME {
            continue;
        }
        // Uma regra ativa já cuida deste estabelecimento.
        let Some(facts) = facts_for(conn, Some(std::slice::from_ref(&sample_txn_id)))?
            .into_iter()
            .next()
        else {
            continue;
        };
        if rules.evaluate(&facts.subject()).category_id.is_some() {
            continue;
        }
        let Ok(rule) = suggest_from_txn(conn, &sample_txn_id) else {
            continue;
        };
        // Sem categoria na regra (tipo que segue o vínculo), não há o que oferecer.
        if rule.actions.category_id.is_none() {
            continue;
        }
        out.push(LearnedRule {
            category_path: paths.get(&category_id).cloned().unwrap_or_default(),
            key,
            category_id,
            count,
            uncategorized: g.uncategorized,
            sample_txn_id,
            sample_description,
            rule,
        });
    }
    out.sort_by(|a, b| (b.uncategorized, b.count, &a.key).cmp(&(a.uncategorized, a.count, &b.key)));
    out.truncate(LIMIT);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::fixture::{self, cat, expense, new_db, write};
    use crate::time::parse_date;

    #[test]
    fn three_times_the_same_category_offers_the_rule() {
        let today = parse_date("2026-10-15").unwrap();
        let mut t = new_db();
        let (market, bakery) = write(&mut t.db, |tx| {
            let acc = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
            let market = cat(tx, "Alimentação › Mercado");
            let bakery = cat(tx, "Alimentação › Padaria");
            // Mesmo mercado, três vezes na mesma categoria, e duas compras ainda sem categoria.
            for d in ["2026-07-05", "2026-08-05", "2026-09-05"] {
                expense(
                    tx,
                    &acc,
                    d,
                    -20_000,
                    Some(&market),
                    None,
                    "SUPERMERCADO ZAFFARI 0123",
                );
            }
            expense(
                tx,
                &acc,
                "2026-10-02",
                -18_000,
                None,
                None,
                "Supermercado Zaffari 0456",
            );
            expense(
                tx,
                &acc,
                "2026-10-09",
                -9_000,
                None,
                None,
                "SUPERMERCADO ZAFFARI 0123",
            );
            // Só duas vezes: ainda não.
            for d in ["2026-08-10", "2026-09-10"] {
                expense(
                    tx,
                    &acc,
                    d,
                    -1_500,
                    Some(&bakery),
                    None,
                    "PADARIA DO BAIRRO",
                );
            }
            // Três vezes, mas em categorias diferentes: não há padrão.
            expense(
                tx,
                &acc,
                "2026-07-20",
                -5_000,
                Some(&market),
                None,
                "LOJA MISTA CENTRO",
            );
            expense(
                tx,
                &acc,
                "2026-08-20",
                -5_000,
                Some(&bakery),
                None,
                "LOJA MISTA CENTRO",
            );
            expense(
                tx,
                &acc,
                "2026-09-20",
                -5_000,
                Some(&market),
                None,
                "LOJA MISTA CENTRO",
            );
            Ok((market, bakery))
        });
        let conn = t.db.conn();
        let found = learned(conn, today).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        let l = &found[0];
        assert_eq!(l.key, "supermercado zaffari");
        assert_eq!(l.category_id, market);
        assert_eq!(l.category_path, "Alimentação › Mercado");
        assert_eq!((l.count, l.uncategorized), (3, 2));
        assert_eq!(l.rule.actions.category_id.as_deref(), Some(market.as_str()));
        assert_eq!(l.rule.conditions.len(), 1);
        let _ = bakery;

        // Criada a regra, a sugestão some: ela já cuida do estabelecimento.
        let rule = l.rule.clone();
        write(&mut t.db, |tx| super::super::create(tx, &rule));
        assert!(learned(t.db.conn(), today).unwrap().is_empty());
    }

    #[test]
    fn an_ignored_merchant_is_not_offered_again() {
        let today = parse_date("2026-10-15").unwrap();
        let mut t = new_db();
        write(&mut t.db, |tx| {
            let acc = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
            let c = cat(tx, "Transporte");
            for d in ["2026-07-05", "2026-08-05", "2026-09-05"] {
                expense(
                    tx,
                    &acc,
                    d,
                    -3_000,
                    Some(&c),
                    None,
                    "POSTO IPIRANGA AV BRASIL",
                );
            }
            // Fora da janela de 12 meses não conta.
            for d in ["2025-01-05", "2025-02-05", "2025-03-05"] {
                expense(tx, &acc, d, -3_000, Some(&c), None, "ACADEMIA ANTIGA");
            }
            Ok(())
        });
        let conn = t.db.conn();
        let found = learned(conn, today).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        ignore(conn, &found[0].key).unwrap();
        assert!(learned(conn, today).unwrap().is_empty());
        assert!(ignore(conn, " ").is_err());
    }
}
