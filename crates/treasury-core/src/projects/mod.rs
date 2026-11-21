//! Projetos: agrupador genérico de despesas e receitas (benfeitoria no imóvel, reforma do
//! carro, festa, viagem…). Entram no projeto os lançamentos ligados a ele (`txn.project_id`)
//! e os itens lançados à mão, fora das contas (`project_item`, V106). Lançamento de receita
//! (`kind = 'income'`) e item positivo são receitas; o resto é despesa (estorno de despesa
//! abate o gasto). O projeto pode ser de um bem (05 · Patrimônio) e as benfeitorias ligadas a
//! ele passam a valer o que o projeto gastou (as receitas não abatem).

pub mod api;
mod entries;
#[cfg(test)]
mod tests;

pub use entries::{
    ProjectEntry, ProjectEntrySource, ProjectFlow, ProjectItemInput, ProjectItemPatch,
    ProjectRemoveArgs, add_item, entries, remove_entries, update_item,
};

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{now_utc, parse_date};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProjectStatus {
    Planned,
    #[default]
    Active,
    Done,
}

impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Done => "done",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "planned" => Self::Planned,
            "done" => Self::Done,
            _ => Self::Active,
        }
    }
}

/// Despesa (ou receita) do projeto numa categoria.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectCategorySpend {
    pub category_id: Option<Id>,
    /// `Casa › Reforma`; `null` = sem categoria.
    pub category_path: Option<String>,
    /// Valor positivo: o gasto (em `by_category`) ou a receita (em `income_by_category`).
    #[ts(type = "number")]
    pub spent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectView {
    pub id: Id,
    pub name: String,
    /// Bem do projeto (imóvel, veículo…).
    pub physical_asset_id: Option<Id>,
    pub physical_asset_name: Option<String>,
    pub status: ProjectStatus,
    /// Orçamento em centavos.
    #[ts(type = "number | null")]
    pub budget: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    /// Gasto: despesas menos estornos, dos lançamentos ligados e dos itens (positivo = gastou).
    #[ts(type = "number")]
    pub spent: i64,
    /// Receitas: lançamentos de receita e itens positivos.
    #[ts(type = "number")]
    pub income: i64,
    /// Resultado: receitas − gasto.
    #[ts(type = "number")]
    pub net: i64,
    /// Lançamentos ligados (das contas).
    #[ts(type = "number")]
    pub txn_count: i64,
    /// Itens lançados à mão no projeto.
    #[ts(type = "number")]
    pub item_count: i64,
    /// Primeira e última data (lançamentos e itens).
    pub first_date: Option<String>,
    pub last_date: Option<String>,
    /// Orçamento − gasto (sem orçamento: `null`).
    #[ts(type = "number | null")]
    pub remaining: Option<i64>,
    /// Gasto por categoria, do maior para o menor.
    pub by_category: Vec<ProjectCategorySpend>,
    /// Receitas por categoria, da maior para a menor.
    pub income_by_category: Vec<ProjectCategorySpend>,
    /// Benfeitorias ligadas ao projeto.
    pub improvement_ids: Vec<Id>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectListArgs {
    /// Incluir os concluídos (padrão: sim).
    #[ts(optional)]
    pub include_done: Option<bool>,
    /// Só os projetos deste bem.
    #[ts(optional)]
    pub physical_asset_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectInput {
    pub name: String,
    #[ts(optional)]
    pub physical_asset_id: Option<Id>,
    #[ts(optional)]
    pub status: Option<ProjectStatus>,
    #[ts(optional, type = "number")]
    pub budget: Option<i64>,
    #[ts(optional)]
    pub start_date: Option<String>,
    #[ts(optional)]
    pub end_date: Option<String>,
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub physical_asset_id: Option<Option<Id>>,
    #[ts(optional)]
    pub status: Option<ProjectStatus>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub budget: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub start_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub end_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub notes: Option<Option<String>>,
}

/// Liga (ou desliga, com `project_id` nulo) lançamentos a um projeto.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectLinkArgs {
    pub txn_ids: Vec<Id>,
    pub project_id: Option<Id>,
}

#[derive(Debug, Clone, PartialEq)]
struct ProjectRow {
    id: Id,
    name: String,
    physical_asset_id: Option<Id>,
    status: ProjectStatus,
    budget: Option<i64>,
    start_date: Option<String>,
    end_date: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

const COLS: &str = "id, name, physical_asset_id, status, budget, start_date, end_date, notes, created_at, updated_at";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok(ProjectRow {
        id: r.get(0)?,
        name: r.get(1)?,
        physical_asset_id: r.get(2)?,
        status: ProjectStatus::parse(&r.get::<_, String>(3)?),
        budget: r.get(4)?,
        start_date: r.get(5)?,
        end_date: r.get(6)?,
        notes: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}

fn load(conn: &Connection, id: &str) -> Result<ProjectRow> {
    conn.query_row(
        &format!("SELECT {COLS} FROM project WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-orcamento-projeto",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

/// Nome do projeto (para mensagens do histórico).
pub fn name_of(conn: &Connection, id: &str) -> Result<String> {
    Ok(load(conn, id)?.name)
}

/// Lançamentos ligados e itens do projeto, cada um com o valor (com sinal) e se é receita.
/// Parâmetro `?1` = o projeto.
pub(crate) const FLOWS: &str =
    "SELECT date, amount, category_id, kind = 'income' AS income, 1 AS from_txn
       FROM txn WHERE project_id = ?1
     UNION ALL
     SELECT date, amount, category_id, amount > 0, 0 FROM project_item WHERE project_id = ?1";

/// Totais do projeto: gasto, receitas, quantos lançamentos e itens, e as datas.
struct Totals {
    spent: i64,
    income: i64,
    txn_count: i64,
    item_count: i64,
    first_date: Option<String>,
    last_date: Option<String>,
}

fn totals(conn: &Connection, id: &str) -> Result<Totals> {
    Ok(conn.query_row(
        &format!(
            "SELECT coalesce(-sum(CASE WHEN income THEN 0 ELSE amount END), 0),
                    coalesce(sum(CASE WHEN income THEN amount ELSE 0 END), 0),
                    coalesce(sum(from_txn), 0), coalesce(sum(1 - from_txn), 0), min(date), max(date)
               FROM ({FLOWS})"
        ),
        [id],
        |r| {
            Ok(Totals {
                spent: r.get(0)?,
                income: r.get(1)?,
                txn_count: r.get(2)?,
                item_count: r.get(3)?,
                first_date: r.get(4)?,
                last_date: r.get(5)?,
            })
        },
    )?)
}

/// Gasto do projeto (despesas menos estornos, dos lançamentos e dos itens).
pub(crate) fn spent(conn: &Connection, id: &str) -> Result<i64> {
    Ok(totals(conn, id)?.spent)
}

/// Despesas do projeto (data, valor gasto), em ordem de data: o que as benfeitorias ligadas
/// valem. Receitas não entram; estornos de despesa abatem.
pub(crate) fn outflows(conn: &Connection, id: &str) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT date, -amount FROM ({FLOWS}) WHERE NOT income ORDER BY date, from_txn"
    ))?;
    Ok(stmt
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?)
}

/// Despesas (`income = false`) ou receitas por categoria, positivas, da maior para a menor.
fn by_category(
    conn: &Connection,
    id: &str,
    income: bool,
    paths: &std::collections::HashMap<Id, String>,
) -> Result<Vec<ProjectCategorySpend>> {
    let sign = if income { "" } else { "-" };
    let mut stmt = conn.prepare(&format!(
        "SELECT category_id, {sign}sum(amount) AS v FROM ({FLOWS}) WHERE income = ?2
          GROUP BY category_id HAVING v <> 0 ORDER BY v DESC"
    ))?;
    let rows = stmt
        .query_map(params![id, income], |r| {
            Ok((r.get::<_, Option<Id>>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .map(|(category_id, spent)| ProjectCategorySpend {
            category_path: category_id.as_ref().and_then(|c| paths.get(c).cloned()),
            category_id,
            spent,
        })
        .collect())
}

fn view(conn: &Connection, p: ProjectRow) -> Result<ProjectView> {
    let t = totals(conn, &p.id)?;
    let physical_asset_name = match &p.physical_asset_id {
        Some(a) => conn
            .query_row("SELECT name FROM physical_asset WHERE id = ?1", [a], |r| {
                r.get(0)
            })
            .optional()?,
        None => None,
    };
    let paths = crate::categories::path_map(conn)?;
    let mut stmt = conn
        .prepare("SELECT id FROM physical_improvement WHERE project_id = ?1 ORDER BY date, id")?;
    let improvement_ids = stmt
        .query_map([&p.id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(ProjectView {
        remaining: p.budget.map(|b| b - t.spent),
        by_category: by_category(conn, &p.id, false, &paths)?,
        income_by_category: by_category(conn, &p.id, true, &paths)?,
        id: p.id,
        name: p.name,
        physical_asset_id: p.physical_asset_id,
        physical_asset_name,
        status: p.status,
        budget: p.budget,
        start_date: p.start_date,
        end_date: p.end_date,
        notes: p.notes,
        spent: t.spent,
        income: t.income,
        net: t.income - t.spent,
        txn_count: t.txn_count,
        item_count: t.item_count,
        first_date: t.first_date,
        last_date: t.last_date,
        improvement_ids,
        created_at: p.created_at,
        updated_at: p.updated_at,
    })
}

pub fn get(conn: &Connection, id: &str) -> Result<ProjectView> {
    view(conn, load(conn, id)?)
}

/// Projetos: em andamento e planejados primeiro, depois os concluídos; dentro de cada
/// grupo, o mais recente primeiro.
pub fn list(conn: &Connection, a: &ProjectListArgs) -> Result<Vec<ProjectView>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM project
          WHERE (?1 OR status <> 'done') AND (?2 IS NULL OR physical_asset_id = ?2)
          ORDER BY status = 'done', coalesce(start_date, substr(created_at, 1, 10)) DESC, name"
    ))?;
    let rows = stmt
        .query_map(
            params![a.include_done.unwrap_or(true), a.physical_asset_id],
            row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.into_iter().map(|p| view(conn, p)).collect()
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn validate(conn: &Connection, p: &ProjectRow) -> Result<()> {
    if p.name.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-orcamento-erro-projeto-sem-nome")));
    }
    if p.budget.is_some_and(|b| b < 0) {
        return Err(Error::invalid(t("nucleo-orcamento-erro-projeto-negativo")));
    }
    for d in [&p.start_date, &p.end_date].into_iter().flatten() {
        parse_date(d)?;
    }
    if let (Some(s), Some(e)) = (&p.start_date, &p.end_date)
        && e < s
    {
        return Err(Error::invalid(t("nucleo-orcamento-erro-projeto-fim-antes")));
    }
    if let Some(a) = &p.physical_asset_id {
        let ok: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM physical_asset WHERE id = ?1)",
            [a],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(Error::not_found(t_args(
                "nucleo-orcamento-bem",
                &args(&[("id", a.clone().into())]),
            )));
        }
    }
    Ok(())
}

pub fn create(tx: &Transaction<'_>, i: &ProjectInput) -> Result<Id> {
    let now = now_utc();
    let p = ProjectRow {
        id: new_id(),
        name: i.name.trim().to_string(),
        physical_asset_id: clean(i.physical_asset_id.clone()),
        status: i.status.unwrap_or_default(),
        budget: i.budget,
        start_date: clean(i.start_date.clone()),
        end_date: clean(i.end_date.clone()),
        notes: clean(i.notes.clone()),
        created_at: now.clone(),
        updated_at: now,
    };
    validate(tx, &p)?;
    tx.execute(
        &format!("INSERT INTO project ({COLS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"),
        params![
            p.id,
            p.name,
            p.physical_asset_id,
            p.status.as_str(),
            p.budget,
            p.start_date,
            p.end_date,
            p.notes,
            p.created_at,
            p.updated_at
        ],
    )?;
    Ok(p.id)
}

pub fn update(tx: &Transaction<'_>, patch: &ProjectPatch) -> Result<()> {
    let old = load(tx, &patch.id)?;
    let mut p = old.clone();
    if let Some(n) = &patch.name {
        p.name = n.trim().to_string();
    }
    if let Some(a) = &patch.physical_asset_id {
        p.physical_asset_id = clean(a.clone());
    }
    if let Some(s) = patch.status {
        p.status = s;
    }
    if let Some(b) = patch.budget {
        p.budget = b;
    }
    if let Some(d) = &patch.start_date {
        p.start_date = clean(d.clone());
    }
    if let Some(d) = &patch.end_date {
        p.end_date = clean(d.clone());
    }
    if let Some(n) = &patch.notes {
        p.notes = clean(n.clone());
    }
    validate(tx, &p)?;
    if p == old {
        return Ok(());
    }
    tx.execute(
        "UPDATE project SET name = ?2, physical_asset_id = ?3, status = ?4, budget = ?5, start_date = ?6,
                end_date = ?7, notes = ?8, updated_at = ?9 WHERE id = ?1",
        params![
            p.id,
            p.name,
            p.physical_asset_id,
            p.status.as_str(),
            p.budget,
            p.start_date,
            p.end_date,
            p.notes,
            now_utc()
        ],
    )?;
    Ok(())
}

/// Apaga o projeto e os itens lançados nele; os lançamentos e as benfeitorias continuam, só
/// sem o projeto.
pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    load(tx, id)?;
    // Explícito (e não pela cascata) para o histórico guardar cada item e o desfazer voltar.
    tx.execute("DELETE FROM project_item WHERE project_id = ?1", [id])?;
    tx.execute(
        "UPDATE txn SET project_id = NULL, updated_at = ?2 WHERE project_id = ?1",
        params![id, now_utc()],
    )?;
    tx.execute(
        "UPDATE physical_improvement SET project_id = NULL, updated_at = ?2 WHERE project_id = ?1",
        params![id, now_utc()],
    )?;
    tx.execute("DELETE FROM project WHERE id = ?1", [id])?;
    Ok(())
}

/// Confere que o projeto existe (ou nenhum).
pub(crate) fn check(conn: &Connection, id: Option<&str>) -> Result<()> {
    if let Some(id) = id {
        load(conn, id)?;
    }
    Ok(())
}

/// Liga os lançamentos ao projeto (ou desliga, com `None`). Devolve quantos mudaram.
pub fn link(tx: &Transaction<'_>, txn_ids: &[Id], project_id: Option<&str>) -> Result<usize> {
    check(tx, project_id)?;
    let mut n = 0;
    let now = now_utc();
    let mut seen = std::collections::HashSet::new();
    for id in txn_ids.iter().filter(|id| seen.insert(id.as_str())) {
        let cur: Option<Option<String>> = tx
            .query_row("SELECT project_id FROM txn WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .optional()?;
        let Some(cur) = cur else {
            return Err(Error::not_found(t_args(
                "nucleo-orcamento-lancamento",
                &args(&[("id", id.to_string().into())]),
            )));
        };
        if cur.as_deref() != project_id {
            tx.execute(
                "UPDATE txn SET project_id = ?2, updated_at = ?3 WHERE id = ?1",
                params![id, project_id, now],
            )?;
            n += 1;
        }
    }
    Ok(n)
}
