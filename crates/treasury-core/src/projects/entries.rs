//! Entradas do projeto numa lista só: os lançamentos ligados (das contas) e os itens lançados
//! à mão, fora das contas (`project_item`, V106). Aqui também o cadastro dos itens e o "tirar
//! do projeto" de várias entradas de uma vez.

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{now_utc, parse_date};

/// Receita ou despesa no projeto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProjectFlow {
    Income,
    Expense,
}

/// De onde vem a entrada: lançamento de uma conta ou item lançado à mão no projeto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProjectEntrySource {
    Txn,
    Item,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectEntry {
    pub source: ProjectEntrySource,
    pub id: Id,
    pub project_id: Id,
    pub date: String,
    /// Com sinal: + entra (receita ou estorno), − sai.
    #[ts(type = "number")]
    pub amount: i64,
    /// Receita ou despesa (estorno de despesa é despesa, com valor positivo).
    pub flow: ProjectFlow,
    pub description: String,
    pub category_id: Option<Id>,
    pub category_path: Option<String>,
    /// Conta do lançamento; `null` nos itens.
    pub account_id: Option<Id>,
    pub account_name: Option<String>,
    pub currency: String,
    pub notes: Option<String>,
    /// Lançamento previsto (ainda não efetivado).
    pub planned: bool,
}

/// Item lançado à mão no projeto, fora das contas.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectItemInput {
    pub project_id: Id,
    pub date: String,
    pub flow: ProjectFlow,
    /// Valor positivo em centavos; o sinal sai de `flow`.
    #[ts(type = "number")]
    pub amount: i64,
    pub description: String,
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectItemPatch {
    pub id: Id,
    /// Passa o item para outro projeto.
    #[ts(optional)]
    pub project_id: Option<Id>,
    #[ts(optional)]
    pub date: Option<String>,
    #[ts(optional)]
    pub flow: Option<ProjectFlow>,
    /// Valor positivo em centavos.
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub category_id: Option<Option<Id>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
}

/// Tira entradas do projeto: os lançamentos continuam nas contas, só sem o projeto; os itens
/// (que só existem no projeto) são apagados.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectRemoveArgs {
    #[serde(default)]
    pub txn_ids: Vec<Id>,
    #[serde(default)]
    pub item_ids: Vec<Id>,
}

#[derive(Debug, Clone, PartialEq)]
struct ItemRow {
    id: Id,
    project_id: Id,
    date: String,
    amount: i64,
    description: String,
    category_id: Option<Id>,
    notes: Option<String>,
}

fn load_item(conn: &Connection, id: &str) -> Result<ItemRow> {
    conn.query_row(
        "SELECT id, project_id, date, amount, description, category_id, notes
           FROM project_item WHERE id = ?1",
        [id],
        |r| {
            Ok(ItemRow {
                id: r.get(0)?,
                project_id: r.get(1)?,
                date: r.get(2)?,
                amount: r.get(3)?,
                description: r.get(4)?,
                category_id: r.get(5)?,
                notes: r.get(6)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-orcamento-item-projeto",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

/// Descrição do item (para mensagens do histórico).
pub fn item_description(conn: &Connection, id: &str) -> Result<String> {
    Ok(load_item(conn, id)?.description)
}

/// Lançamentos e itens do projeto, em ordem de data (lançamentos antes dos itens no mesmo dia).
pub fn entries(conn: &Connection, project_id: &str) -> Result<Vec<ProjectEntry>> {
    super::check(conn, Some(project_id))?;
    let paths = crate::categories::path_map(conn)?;
    let path = |c: &Option<Id>| c.as_ref().and_then(|c| paths.get(c).cloned());
    let mut stmt = conn.prepare(
        "SELECT t.id, t.date, t.amount, t.kind, t.description, t.category_id, t.account_id,
                a.name, t.currency, t.notes, t.status
           FROM txn t JOIN account a ON a.id = t.account_id
          WHERE t.project_id = ?1
          ORDER BY t.date, t.id",
    )?;
    let mut out = stmt
        .query_map([project_id], |r| {
            let category_id: Option<Id> = r.get(5)?;
            Ok(ProjectEntry {
                source: ProjectEntrySource::Txn,
                id: r.get(0)?,
                project_id: project_id.to_string(),
                date: r.get(1)?,
                amount: r.get(2)?,
                flow: if r.get::<_, String>(3)? == "income" {
                    ProjectFlow::Income
                } else {
                    ProjectFlow::Expense
                },
                description: r.get(4)?,
                category_path: path(&category_id),
                category_id,
                account_id: r.get(6)?,
                account_name: r.get(7)?,
                currency: r.get(8)?,
                notes: r.get(9)?,
                planned: r.get::<_, String>(10)? == "planned",
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let currency = crate::household::base_currency(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, date, amount, description, category_id, notes
           FROM project_item WHERE project_id = ?1 ORDER BY date, id",
    )?;
    let items = stmt
        .query_map([project_id], |r| {
            let category_id: Option<Id> = r.get(4)?;
            let amount: i64 = r.get(2)?;
            Ok(ProjectEntry {
                source: ProjectEntrySource::Item,
                id: r.get(0)?,
                project_id: project_id.to_string(),
                date: r.get(1)?,
                amount,
                flow: if amount > 0 {
                    ProjectFlow::Income
                } else {
                    ProjectFlow::Expense
                },
                description: r.get(3)?,
                category_path: path(&category_id),
                category_id,
                account_id: None,
                account_name: None,
                currency: currency.clone(),
                notes: r.get(5)?,
                planned: false,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    out.extend(items);
    // Estável: no mesmo dia, os lançamentos (já em ordem) vêm antes dos itens.
    out.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(out)
}

/// O item como entrada do projeto (depois de criar ou alterar).
pub fn item_entry(conn: &Connection, id: &str) -> Result<ProjectEntry> {
    let item = load_item(conn, id)?;
    entries(conn, &item.project_id)?
        .into_iter()
        .find(|e| e.source == ProjectEntrySource::Item && e.id == id)
        .ok_or_else(|| {
            Error::not_found(t_args(
                "nucleo-orcamento-item-projeto",
                &args(&[("id", id.to_string().into())]),
            ))
        })
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn signed(flow: ProjectFlow, amount: i64) -> i64 {
    match flow {
        ProjectFlow::Income => amount,
        ProjectFlow::Expense => -amount,
    }
}

fn validate(conn: &Connection, i: &ItemRow) -> Result<()> {
    super::check(conn, Some(&i.project_id))?;
    if i.description.trim().is_empty() {
        return Err(Error::invalid(t(
            "nucleo-orcamento-erro-item-sem-descricao",
        )));
    }
    parse_date(&i.date)?;
    if let Some(c) = &i.category_id {
        let ok: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM category WHERE id = ?1)",
            [c],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(Error::not_found(t_args(
                "nucleo-orcamento-categoria",
                &args(&[("id", c.to_string().into())]),
            )));
        }
    }
    Ok(())
}

fn positive(amount: i64) -> Result<i64> {
    if amount <= 0 {
        return Err(Error::invalid(t("nucleo-orcamento-erro-item-valor")));
    }
    Ok(amount)
}

pub fn add_item(tx: &Transaction<'_>, i: &ProjectItemInput) -> Result<Id> {
    let row = ItemRow {
        id: new_id(),
        project_id: i.project_id.clone(),
        date: i.date.trim().to_string(),
        amount: signed(i.flow, positive(i.amount)?),
        description: i.description.trim().to_string(),
        category_id: clean(i.category_id.clone()),
        notes: clean(i.notes.clone()),
    };
    validate(tx, &row)?;
    let now = now_utc();
    tx.execute(
        "INSERT INTO project_item (id, project_id, date, amount, description, category_id, notes,
                                   created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            row.id,
            row.project_id,
            row.date,
            row.amount,
            row.description,
            row.category_id,
            row.notes,
            now
        ],
    )?;
    Ok(row.id)
}

pub fn update_item(tx: &Transaction<'_>, p: &ProjectItemPatch) -> Result<()> {
    let old = load_item(tx, &p.id)?;
    let mut row = old.clone();
    if let Some(project) = &p.project_id {
        row.project_id = project.clone();
    }
    if let Some(d) = &p.date {
        row.date = d.trim().to_string();
    }
    let flow = p.flow.unwrap_or(if old.amount > 0 {
        ProjectFlow::Income
    } else {
        ProjectFlow::Expense
    });
    let amount = match p.amount {
        Some(a) => positive(a)?,
        None => old.amount.abs(),
    };
    row.amount = signed(flow, amount);
    if let Some(d) = &p.description {
        row.description = d.trim().to_string();
    }
    if let Some(c) = &p.category_id {
        row.category_id = clean(c.clone());
    }
    if let Some(n) = &p.notes {
        row.notes = clean(n.clone());
    }
    validate(tx, &row)?;
    if row == old {
        return Ok(());
    }
    tx.execute(
        "UPDATE project_item SET project_id = ?2, date = ?3, amount = ?4, description = ?5,
                category_id = ?6, notes = ?7, updated_at = ?8 WHERE id = ?1",
        params![
            row.id,
            row.project_id,
            row.date,
            row.amount,
            row.description,
            row.category_id,
            row.notes,
            now_utc()
        ],
    )?;
    Ok(())
}

/// Tira as entradas do projeto (lançamentos ficam sem projeto, itens são apagados). Devolve
/// quantas mudaram.
pub fn remove_entries(tx: &Transaction<'_>, a: &ProjectRemoveArgs) -> Result<usize> {
    let mut n = super::link(tx, &a.txn_ids, None)?;
    let mut seen = std::collections::HashSet::new();
    for id in a.item_ids.iter().filter(|id| seen.insert(id.as_str())) {
        load_item(tx, id)?;
        n += tx.execute("DELETE FROM project_item WHERE id = ?1", [id])?;
    }
    Ok(n)
}
