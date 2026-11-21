//! Família (cofre), membros (titulares, sem login próprio) e instituições.

pub mod api;
pub mod banks;
pub mod overview;
#[cfg(test)]
mod tests_members;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::now_utc;
use crate::transactions::util::{clean_color, clean_name, clean_opt};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Household {
    pub id: Id,
    pub name: String,
    pub base_currency: String,
    /// País onde a família mora (ISO 3166-1 alfa-2; docs/countries.md).
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HouseholdPatch {
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub base_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Member {
    pub id: Id,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
    pub archived: bool,
    #[ts(type = "number")]
    pub sort_order: i64,
    pub profile: MemberProfile,
}

/// Relação com quem cuida das finanças da família.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MemberRelationship {
    Titular,
    Conjuge,
    Filho,
    PaiMae,
    Outro,
}

/// Sexo do membro para a tábua de mortalidade (docs/life-expectancy.md). `unspecified` =
/// prefiro não informar (usa a tábua de ambos os sexos, como o não informado).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MemberSex {
    Male,
    Female,
    Unspecified,
}

impl MemberSex {
    pub fn as_str(self) -> &'static str {
        match self {
            MemberSex::Male => "male",
            MemberSex::Female => "female",
            MemberSex::Unspecified => "unspecified",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "male" => Some(MemberSex::Male),
            "female" => Some(MemberSex::Female),
            "unspecified" => Some(MemberSex::Unspecified),
            _ => None,
        }
    }
}

/// Cadastro do membro além do nome (docs/income.md). Sem CPF: o app não guarda documentos.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct MemberProfile {
    /// `YYYY-MM-DD`.
    pub birth_date: Option<String>,
    pub relationship: Option<MemberRelationship>,
    /// Dependente no IR de quem declara (dedução mensal no IRRF).
    pub irpf_dependent: bool,
    /// Profissão ou ocupação.
    pub occupation: Option<String>,
    pub notes: Option<String>,
    /// Sexo para a linha da vida e o INSS (`null` = não informado).
    pub sex: Option<MemberSex>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberInput {
    pub name: String,
    #[ts(optional)]
    pub color: Option<String>,
    #[ts(optional)]
    pub is_primary: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub profile: Option<MemberProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub color: Option<Option<String>>,
    #[ts(optional)]
    pub is_primary: Option<bool>,
    #[ts(optional)]
    pub archived: Option<bool>,
    #[ts(optional, type = "number")]
    pub sort_order: Option<i64>,
    /// Substitui o cadastro inteiro.
    #[serde(default)]
    #[ts(optional)]
    pub profile: Option<MemberProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InstitutionKind {
    Bank,
    Broker,
    Issuer,
    Other,
}

impl InstitutionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            InstitutionKind::Bank => "bank",
            InstitutionKind::Broker => "broker",
            InstitutionKind::Issuer => "issuer",
            InstitutionKind::Other => "other",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "broker" => InstitutionKind::Broker,
            "issuer" => InstitutionKind::Issuer,
            "other" => InstitutionKind::Other,
            _ => InstitutionKind::Bank,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Institution {
    pub id: Id,
    pub name: String,
    pub kind: InstitutionKind,
    pub code: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InstitutionInput {
    pub name: String,
    #[ts(optional)]
    pub kind: Option<InstitutionKind>,
    #[ts(optional)]
    pub code: Option<String>,
    #[ts(optional)]
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InstitutionPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub kind: Option<InstitutionKind>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub code: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub website: Option<Option<String>>,
}

/// Dados iniciais do cofre: a família e seus membros. O primeiro membro é o principal.
pub fn seed(
    tx: &Transaction<'_>,
    name: &str,
    base_currency: &str,
    members: &[String],
) -> Result<Id> {
    seed_in(tx, name, base_currency, "BR", members)
}

/// Como [`seed`], com o país da família.
pub fn seed_in(
    tx: &Transaction<'_>,
    name: &str,
    base_currency: &str,
    country: &str,
    members: &[String],
) -> Result<Id> {
    let now = now_utc();
    let id = new_id();
    tx.execute(
        "INSERT INTO household (id, name, base_currency, country, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, name, base_currency, country, now],
    )?;
    for (i, m) in members.iter().enumerate() {
        tx.execute(
            "INSERT INTO member (id, name, is_primary, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![new_id(), m, (i == 0) as i64, i as i64, now],
        )?;
    }
    Ok(id)
}

pub fn get(conn: &Connection) -> Result<Household> {
    Ok(conn.query_row(
        "SELECT id, name, base_currency, country FROM household LIMIT 1",
        [],
        |r| {
            Ok(Household {
                id: r.get(0)?,
                name: r.get(1)?,
                base_currency: r.get(2)?,
                country: r.get(3)?,
            })
        },
    )?)
}

/// Moeda base da família (vem do país na criação do cofre). Sem a linha da família, BRL, a
/// dos cofres de antes do país.
pub fn base_currency(conn: &Connection) -> Result<String> {
    Ok(conn
        .query_row("SELECT base_currency FROM household LIMIT 1", [], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "BRL".into()))
}

/// Atualiza o nome e/ou a moeda base da família.
pub fn update(tx: &Transaction<'_>, patch: &HouseholdPatch) -> Result<Household> {
    let cur = get(tx)?;
    let name = match &patch.name {
        Some(n) => clean_name(n, "da família")?,
        None => cur.name.clone(),
    };
    let base = match &patch.base_currency {
        Some(c) => crate::money::validate_currency(c)?,
        None => cur.base_currency.clone(),
    };
    if name != cur.name || base != cur.base_currency {
        tx.execute(
            "UPDATE household SET name = ?2, base_currency = ?3, updated_at = ?4 WHERE id = ?1",
            params![cur.id, name, base, now_utc()],
        )?;
    }
    get(tx)
}

// ---------------------------------------------------------------------------
// Membros

const MEMBER_COLS: &str = "id, name, color, is_primary, archived, sort_order, birth_date, relationship, irpf_dependent, occupation, notes, sex";

fn row_to_member(r: &rusqlite::Row<'_>) -> rusqlite::Result<Member> {
    let rel: Option<String> = r.get(7)?;
    Ok(Member {
        id: r.get(0)?,
        name: r.get(1)?,
        color: r.get(2)?,
        is_primary: r.get(3)?,
        archived: r.get(4)?,
        sort_order: r.get(5)?,
        profile: MemberProfile {
            birth_date: r.get(6)?,
            relationship: rel
                .and_then(|v| serde_json::from_value(serde_json::Value::String(v)).ok()),
            irpf_dependent: r.get(8)?,
            occupation: r.get(9)?,
            notes: r.get(10)?,
            sex: r
                .get::<_, Option<String>>(11)?
                .as_deref()
                .and_then(MemberSex::parse),
        },
    })
}

/// Grava o cadastro do membro (datas e textos conferidos).
fn save_profile(tx: &Transaction<'_>, id: &str, p: &MemberProfile) -> Result<()> {
    if let Some(d) = &p.birth_date {
        crate::time::parse_date(d)?;
    }
    let rel = p.relationship.and_then(|r| {
        serde_json::to_value(r)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
    });
    tx.execute(
        "UPDATE member SET birth_date = ?2, relationship = ?3, irpf_dependent = ?4, occupation = ?5, notes = ?6,
                           sex = ?8, updated_at = ?7
         WHERE id = ?1 AND (birth_date IS NOT ?2 OR relationship IS NOT ?3 OR irpf_dependent IS NOT ?4
                            OR occupation IS NOT ?5 OR notes IS NOT ?6 OR sex IS NOT ?8)",
        params![
            id,
            p.birth_date,
            rel,
            p.irpf_dependent,
            clean_opt(p.occupation.clone()),
            clean_opt(p.notes.clone()),
            now_utc(),
            p.sex.map(MemberSex::as_str),
        ],
    )?;
    Ok(())
}

pub fn list_members(conn: &Connection, include_archived: bool) -> Result<Vec<Member>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {MEMBER_COLS} FROM member WHERE (?1 OR archived = 0)
         ORDER BY archived, sort_order, name COLLATE NOCASE"
    ))?;
    let rows = stmt
        .query_map([include_archived], row_to_member)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_member(conn: &Connection, id: &str) -> Result<Member> {
    conn.query_row(
        &format!("SELECT {MEMBER_COLS} FROM member WHERE id = ?1"),
        [id],
        row_to_member,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-geral-membro",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

fn ensure_unique_member_name(conn: &Connection, name: &str, except: Option<&str>) -> Result<()> {
    let clash: Option<String> = conn
        .query_row(
            "SELECT id FROM member WHERE name = ?1 COLLATE NOCASE AND (?2 IS NULL OR id <> ?2) LIMIT 1",
            params![name, except],
            |r| r.get(0),
        )
        .optional()?;
    if clash.is_some() {
        return Err(Error::conflict(t_args(
            "nucleo-geral-erro-membro-repetido",
            &args(&[("nome", name.to_string().into())]),
        )));
    }
    Ok(())
}

fn clear_primary(tx: &Transaction<'_>, except: &str, now: &str) -> Result<()> {
    tx.execute(
        "UPDATE member SET is_primary = 0, updated_at = ?2 WHERE is_primary = 1 AND id <> ?1",
        params![except, now],
    )?;
    Ok(())
}

pub fn create_member(tx: &Transaction<'_>, input: &MemberInput) -> Result<Member> {
    let name = clean_name(&input.name, "do membro")?;
    ensure_unique_member_name(tx, &name, None)?;
    let color = clean_color(input.color.clone())?;
    let has_primary: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM member WHERE is_primary = 1 AND archived = 0)",
        [],
        |r| r.get(0),
    )?;
    // O primeiro membro vira o principal automaticamente.
    let primary = input.is_primary.unwrap_or(!has_primary);
    let sort: i64 = tx.query_row(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM member",
        [],
        |r| r.get(0),
    )?;
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO member (id, name, color, is_primary, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, name, color, primary, sort, now],
    )?;
    if primary {
        clear_primary(tx, &id, &now)?;
    }
    if let Some(p) = &input.profile {
        save_profile(tx, &id, p)?;
    }
    get_member(tx, &id)
}

pub fn update_member(tx: &Transaction<'_>, patch: &MemberPatch) -> Result<Member> {
    let cur = get_member(tx, &patch.id)?;
    let name = match &patch.name {
        Some(n) => {
            let n = clean_name(n, "do membro")?;
            ensure_unique_member_name(tx, &n, Some(&cur.id))?;
            n
        }
        None => cur.name.clone(),
    };
    let color = match &patch.color {
        Some(c) => clean_color(c.clone())?,
        None => cur.color.clone(),
    };
    let archived = patch.archived.unwrap_or(cur.archived);
    let primary = patch.is_primary.unwrap_or(cur.is_primary);
    if archived && primary {
        return Err(Error::invalid(t(if cur.archived {
            "nucleo-geral-erro-arquivado-principal"
        } else {
            "nucleo-geral-erro-principal-arquivado"
        })));
    }
    let sort = patch.sort_order.unwrap_or(cur.sort_order);
    let now = now_utc();
    tx.execute(
        "UPDATE member SET name = ?2, color = ?3, is_primary = ?4, archived = ?5, sort_order = ?6, updated_at = ?7
         WHERE id = ?1 AND (name IS NOT ?2 OR color IS NOT ?3 OR is_primary IS NOT ?4 OR archived IS NOT ?5
                            OR sort_order IS NOT ?6)",
        params![cur.id, name, color, primary, archived, sort, now],
    )?;
    if primary && !cur.is_primary {
        clear_primary(tx, &cur.id, &now)?;
    }
    if let Some(p) = &patch.profile {
        save_profile(tx, &cur.id, p)?;
    }
    get_member(tx, &cur.id)
}

// ---------------------------------------------------------------------------
// Instituições

const INSTITUTION_COLS: &str = "id, name, kind, code, website";

fn row_to_institution(r: &rusqlite::Row<'_>) -> rusqlite::Result<Institution> {
    Ok(Institution {
        id: r.get(0)?,
        name: r.get(1)?,
        kind: InstitutionKind::parse(&r.get::<_, String>(2)?),
        code: r.get(3)?,
        website: r.get(4)?,
    })
}

pub fn list_institutions(conn: &Connection) -> Result<Vec<Institution>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {INSTITUTION_COLS} FROM institution ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt
        .query_map([], row_to_institution)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_institution(conn: &Connection, id: &str) -> Result<Institution> {
    conn.query_row(
        &format!("SELECT {INSTITUTION_COLS} FROM institution WHERE id = ?1"),
        [id],
        row_to_institution,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-geral-instituicao",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

fn clean_code(code: Option<String>) -> Result<Option<String>> {
    let c = clean_opt(code);
    if let Some(v) = &c
        && v.len() > 20
    {
        return Err(Error::invalid(t_args(
            "nucleo-geral-erro-codigo-instituicao",
            &args(&[("codigo", v.clone().into())]),
        )));
    }
    Ok(c)
}

fn clean_website(site: Option<String>) -> Result<Option<String>> {
    let w = clean_opt(site);
    if let Some(v) = &w
        && (v.len() > 300 || v.chars().any(char::is_whitespace))
    {
        return Err(Error::invalid(t_args(
            "nucleo-geral-erro-site",
            &args(&[("site", v.clone().into())]),
        )));
    }
    Ok(w)
}

pub fn create_institution(tx: &Transaction<'_>, input: &InstitutionInput) -> Result<Institution> {
    let name = clean_name(&input.name, "da instituição")?;
    let id = new_id();
    tx.execute(
        "INSERT INTO institution (id, name, kind, code, website, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            id,
            name,
            input.kind.unwrap_or(InstitutionKind::Bank).as_str(),
            clean_code(input.code.clone())?,
            clean_website(input.website.clone())?,
            now_utc()
        ],
    )?;
    get_institution(tx, &id)
}

pub fn update_institution(tx: &Transaction<'_>, patch: &InstitutionPatch) -> Result<Institution> {
    let cur = get_institution(tx, &patch.id)?;
    let name = match &patch.name {
        Some(n) => clean_name(n, "da instituição")?,
        None => cur.name.clone(),
    };
    let kind = patch.kind.unwrap_or(cur.kind);
    let code = match &patch.code {
        Some(c) => clean_code(c.clone())?,
        None => cur.code.clone(),
    };
    let website = match &patch.website {
        Some(w) => clean_website(w.clone())?,
        None => cur.website.clone(),
    };
    tx.execute(
        "UPDATE institution SET name = ?2, kind = ?3, code = ?4, website = ?5, updated_at = ?6
         WHERE id = ?1 AND (name IS NOT ?2 OR kind IS NOT ?3 OR code IS NOT ?4 OR website IS NOT ?5)",
        params![cur.id, name, kind.as_str(), code, website, now_utc()],
    )?;
    get_institution(tx, &cur.id)
}

/// Apaga uma instituição que nenhuma conta usa.
pub fn delete_institution(tx: &Transaction<'_>, id: &str) -> Result<()> {
    let inst = get_institution(tx, id)?;
    let used: i64 = tx.query_row(
        "SELECT count(*) FROM account WHERE institution_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    if used > 0 {
        return Err(Error::conflict(t_args(
            "nucleo-geral-erro-instituicao-em-uso",
            &args(&[("instituicao", inst.name.into()), ("n", used.into())]),
        )));
    }
    tx.execute("DELETE FROM institution WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests;
