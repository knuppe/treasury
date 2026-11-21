//! Etiquetas de ativos da família (V161): nomes livres ligados a ativos para agrupar. Uma
//! etiqueta pode ser **grupo de concentração**, com limite próprio (sem ele, o padrão de grupo).
//! As fontes de meta por etiqueta usam [`assets_with`].

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::{Id, new_id};
use crate::time::now_utc;

/// Teto de etiquetas por ativo e tamanho do nome.
const MAX_PER_ASSET: usize = 20;
const MAX_NAME: usize = 40;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetLabel {
    pub id: Id,
    pub name: String,
    pub is_concentration_group: bool,
    /// % da carteira; `null` = o padrão de grupo.
    pub concentration_limit_pct: Option<f64>,
    /// Ativos com a etiqueta.
    pub asset_ids: Vec<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetAssetLabelsArgs {
    pub asset_id: Id,
    /// Nomes das etiquetas do ativo (substitui a lista; as novas são criadas).
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateAssetLabelArgs {
    /// Nome da etiqueta (criada se não existir).
    pub name: String,
    #[serde(default)]
    #[ts(optional)]
    pub is_concentration_group: Option<bool>,
    /// % da carteira; `null` volta ao padrão de grupo.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub concentration_limit_pct: Option<Option<f64>>,
}

pub(crate) fn clean_name(s: &str) -> Result<String> {
    let n: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if n.is_empty() || n.chars().count() > MAX_NAME {
        return Err(Error::invalid(t("nucleo-ativos-etiqueta-invalida")));
    }
    Ok(n)
}

pub(crate) fn check_pct(p: Option<f64>) -> Result<Option<f64>> {
    match p {
        Some(v) if !(v > 0.0 && v <= 100.0) => {
            Err(Error::invalid(t("nucleo-carteira-limite-invalido")))
        }
        other => Ok(other),
    }
}

fn find(conn: &Connection, name: &str) -> Result<Option<Id>> {
    Ok(conn
        .query_row(
            "SELECT id FROM asset_label WHERE name = ?1 COLLATE NOCASE",
            [name],
            |r| r.get(0),
        )
        .optional()?)
}

fn ensure(conn: &Connection, name: &str) -> Result<Id> {
    let name = clean_name(name)?;
    if let Some(id) = find(conn, &name)? {
        return Ok(id);
    }
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO asset_label (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, name, now],
    )?;
    Ok(id)
}

pub fn list(conn: &Connection) -> Result<Vec<AssetLabel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, is_concentration_group, concentration_limit_pct FROM asset_label
         ORDER BY name COLLATE NOCASE",
    )?;
    let rows: Vec<(Id, String, i64, Option<f64>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(|(id, name, g, limit)| {
            Ok(AssetLabel {
                asset_ids: assets_of_label(conn, &id)?,
                id,
                name,
                is_concentration_group: g == 1,
                concentration_limit_pct: limit,
            })
        })
        .collect()
}

fn assets_of_label(conn: &Connection, label_id: &str) -> Result<Vec<Id>> {
    let mut stmt = conn
        .prepare("SELECT asset_id FROM asset_label_link WHERE label_id = ?1 ORDER BY asset_id")?;
    let v = stmt
        .query_map([label_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Ativos com a etiqueta (por nome, sem diferenciar maiúsculas); vazio se ela não existe.
pub fn assets_with(conn: &Connection, name: &str) -> Result<Vec<Id>> {
    match find(conn, name.trim())? {
        Some(id) => assets_of_label(conn, &id),
        None => Ok(vec![]),
    }
}

/// Nomes das etiquetas de um ativo.
pub fn of_asset(conn: &Connection, asset_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT l.name FROM asset_label l JOIN asset_label_link k ON k.label_id = l.id
         WHERE k.asset_id = ?1 ORDER BY l.name COLLATE NOCASE",
    )?;
    let v = stmt
        .query_map([asset_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Troca as etiquetas do ativo; as que ficam sem ativo e sem papel de grupo somem.
pub fn set_for_asset(conn: &Connection, a: &SetAssetLabelsArgs) -> Result<Vec<String>> {
    crate::assets::get(conn, &a.asset_id)?;
    if a.labels.len() > MAX_PER_ASSET {
        return Err(Error::invalid(t("nucleo-ativos-etiqueta-invalida")));
    }
    let ids = a
        .labels
        .iter()
        .map(|n| ensure(conn, n))
        .collect::<Result<Vec<_>>>()?;
    conn.execute(
        "DELETE FROM asset_label_link WHERE asset_id = ?1",
        [&a.asset_id],
    )?;
    for id in ids {
        conn.execute(
            "INSERT OR IGNORE INTO asset_label_link (asset_id, label_id) VALUES (?1, ?2)",
            params![a.asset_id, id],
        )?;
    }
    conn.execute(
        "DELETE FROM asset_label WHERE is_concentration_group = 0 AND concentration_limit_pct IS NULL
           AND id NOT IN (SELECT label_id FROM asset_label_link)",
        [],
    )?;
    of_asset(conn, &a.asset_id)
}

/// Marca (ou desmarca) a etiqueta como grupo de concentração e ajusta o limite dela.
pub fn update(conn: &Connection, a: &UpdateAssetLabelArgs) -> Result<AssetLabel> {
    let id = ensure(conn, &a.name)?;
    if let Some(g) = a.is_concentration_group {
        conn.execute(
            "UPDATE asset_label SET is_concentration_group = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, g as i64, now_utc()],
        )?;
    }
    if let Some(limit) = a.concentration_limit_pct {
        conn.execute(
            "UPDATE asset_label SET concentration_limit_pct = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, check_pct(limit)?, now_utc()],
        )?;
    }
    list(conn)?
        .into_iter()
        .find(|l| l.id == id)
        .ok_or_else(|| Error::NotFound(t("nucleo-ativos-etiqueta-invalida")))
}
