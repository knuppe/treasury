//! Tags e anexos (comprovantes guardados como BLOB dentro do banco cifrado).

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use super::util::{clean_color, clean_name};
use super::{AttachmentMeta, Tag, b64};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

/// Tamanho máximo de um anexo (depois de decodificado).
pub const MAX_ATTACHMENT_BYTES: usize = 20 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagSummary {
    pub id: Id,
    pub name: String,
    pub color: Option<String>,
    /// Lançamentos com esta tag.
    #[ts(type = "number")]
    pub txn_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagInput {
    pub name: String,
    #[ts(optional)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub color: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttachmentInput {
    pub txn_id: Id,
    pub filename: String,
    /// Tipo MIME, ex.: `application/pdf`, `image/jpeg`.
    pub mime: String,
    /// Conteúdo em base64 (até 20 MB depois de decodificado).
    pub content_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttachmentContent {
    #[serde(flatten)]
    pub meta: AttachmentMeta,
    pub content_base64: String,
}

// ---------------------------------------------------------------------------
// Tags

pub fn list_tags(conn: &Connection) -> Result<Vec<TagSummary>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.color, (SELECT count(*) FROM txn_tag tt WHERE tt.tag_id = g.id)
           FROM tag g ORDER BY g.name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(TagSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                txn_count: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_tag(conn: &Connection, id: &str) -> Result<Tag> {
    conn.query_row("SELECT id, name, color FROM tag WHERE id = ?1", [id], |r| {
        Ok(Tag {
            id: r.get(0)?,
            name: r.get(1)?,
            color: r.get(2)?,
        })
    })
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-lancamentos-ref-tag",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

fn ensure_unique_tag(conn: &Connection, name: &str, except: Option<&str>) -> Result<()> {
    let target = super::query::search_fold(name);
    let mut stmt = conn.prepare("SELECT id, name FROM tag WHERE (?1 IS NULL OR id <> ?1)")?;
    let clash = stmt
        .query_map([except], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .any(|(_, n)| super::query::search_fold(&n) == target);
    if clash {
        return Err(Error::conflict(t_args(
            "nucleo-lancamentos-erro-tag-repetida",
            &args(&[("nome", name.to_string().into())]),
        )));
    }
    Ok(())
}

pub fn create_tag(tx: &Transaction<'_>, input: &TagInput) -> Result<Tag> {
    let name = clean_name(&input.name, "da tag")?;
    ensure_unique_tag(tx, &name, None)?;
    let id = new_id();
    tx.execute(
        "INSERT INTO tag (id, name, color, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, name, clean_color(input.color.clone())?, now_utc()],
    )?;
    get_tag(tx, &id)
}

pub fn update_tag(tx: &Transaction<'_>, patch: &TagPatch) -> Result<Tag> {
    let cur = get_tag(tx, &patch.id)?;
    let name = match &patch.name {
        Some(n) => {
            let n = clean_name(n, "da tag")?;
            ensure_unique_tag(tx, &n, Some(&cur.id))?;
            n
        }
        None => cur.name.clone(),
    };
    let color = match &patch.color {
        Some(c) => clean_color(c.clone())?,
        None => cur.color.clone(),
    };
    tx.execute(
        "UPDATE tag SET name = ?2, color = ?3, updated_at = ?4
          WHERE id = ?1 AND (name IS NOT ?2 OR color IS NOT ?3)",
        params![cur.id, name, color, now_utc()],
    )?;
    get_tag(tx, &cur.id)
}

/// Apaga a tag e a tira de todos os lançamentos. Devolve o nome apagado.
pub fn delete_tag(tx: &Transaction<'_>, id: &str) -> Result<String> {
    let tag = get_tag(tx, id)?;
    tx.execute("DELETE FROM txn_tag WHERE tag_id = ?1", [id])?;
    tx.execute("DELETE FROM tag WHERE id = ?1", [id])?;
    Ok(tag.name)
}

// ---------------------------------------------------------------------------
// Anexos

const META_COLS: &str = "id, txn_id, filename, mime, size_bytes, sha256, created_at";

fn row_to_meta(r: &rusqlite::Row<'_>) -> rusqlite::Result<AttachmentMeta> {
    Ok(AttachmentMeta {
        id: r.get(0)?,
        txn_id: r.get(1)?,
        filename: r.get(2)?,
        mime: r.get(3)?,
        size_bytes: r.get(4)?,
        sha256: r.get(5)?,
        created_at: r.get(6)?,
    })
}

pub fn list_attachments(conn: &Connection, txn_id: &str) -> Result<Vec<AttachmentMeta>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {META_COLS} FROM attachment WHERE txn_id = ?1 ORDER BY created_at, id"
    ))?;
    let rows = stmt
        .query_map([txn_id], row_to_meta)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn attachment_meta(conn: &Connection, id: &str) -> Result<AttachmentMeta> {
    conn.query_row(
        &format!("SELECT {META_COLS} FROM attachment WHERE id = ?1"),
        [id],
        row_to_meta,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-lancamentos-ref-anexo",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

/// Só o nome do arquivo (sem pastas), sem caracteres de controle.
fn clean_filename(name: &str) -> Result<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or_default();
    let clean: String = base.chars().filter(|c| !c.is_control()).collect();
    let clean = clean.trim();
    if clean.is_empty() || clean == "." || clean == ".." {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-arquivo-nome")));
    }
    if clean.chars().count() > 255 {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-arquivo-nome-longo",
        )));
    }
    Ok(clean.to_string())
}

fn clean_mime(mime: &str) -> Result<String> {
    let m = mime.trim().to_ascii_lowercase();
    if m.is_empty() {
        return Ok("application/octet-stream".into());
    }
    let valid = m.len() <= 127
        && m.split_once('/').is_some_and(|(a, b)| {
            let ok = |s: &str| {
                !s.is_empty()
                    && s.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "!#$&-^_.+".contains(c))
            };
            ok(a) && ok(b.split(';').next().unwrap_or_default().trim())
        });
    if !valid {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-arquivo-tipo",
            &args(&[("tipo", mime.to_string().into())]),
        )));
    }
    Ok(m)
}

fn too_big() -> Error {
    Error::invalid(t_args(
        "nucleo-lancamentos-erro-anexo-grande",
        &args(&[("mb", (MAX_ATTACHMENT_BYTES / (1024 * 1024)).into())]),
    ))
}

pub fn add_attachment(tx: &Transaction<'_>, input: &AttachmentInput) -> Result<AttachmentMeta> {
    if !super::util::exists(tx, "txn", &input.txn_id)? {
        return Err(Error::not_found(super::util::txn_ref(&input.txn_id)));
    }
    let filename = clean_filename(&input.filename)?;
    let mime = clean_mime(&input.mime)?;
    // Recusa cedo pelo tamanho do texto, antes de decodificar.
    if b64::decoded_len_estimate(input.content_base64.trim()) > MAX_ATTACHMENT_BYTES + 3 * 1024 {
        return Err(too_big());
    }
    let data = b64::decode(&input.content_base64)?;
    if data.len() > MAX_ATTACHMENT_BYTES {
        return Err(too_big());
    }
    if data.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-arquivo-vazio")));
    }
    let sha = hex::encode(Sha256::digest(&data));
    let id = new_id();
    tx.execute(
        "INSERT INTO attachment (id, txn_id, filename, mime, size_bytes, sha256, data, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id,
            input.txn_id,
            filename,
            mime,
            data.len() as i64,
            sha,
            data,
            now_utc()
        ],
    )?;
    attachment_meta(tx, &id)
}

pub fn get_attachment(conn: &Connection, id: &str) -> Result<AttachmentContent> {
    let meta = attachment_meta(conn, id)?;
    let data: Vec<u8> = conn.query_row("SELECT data FROM attachment WHERE id = ?1", [id], |r| {
        r.get(0)
    })?;
    Ok(AttachmentContent {
        meta,
        content_base64: b64::encode(&data),
    })
}

/// Apaga um anexo e devolve o nome do arquivo.
pub fn delete_attachment(tx: &Transaction<'_>, id: &str) -> Result<String> {
    let meta = attachment_meta(tx, id)?;
    tx.execute("DELETE FROM attachment WHERE id = ?1", [id])?;
    Ok(meta.filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_filenames_and_mime() {
        assert_eq!(
            clean_filename("../../etc/recibo.pdf").unwrap(),
            "recibo.pdf"
        );
        assert_eq!(clean_filename("C:\\x\\nota.jpg").unwrap(), "nota.jpg");
        assert!(clean_filename("pasta/").is_err());
        assert_eq!(clean_mime("").unwrap(), "application/octet-stream");
        assert_eq!(clean_mime("Image/JPEG").unwrap(), "image/jpeg");
        assert_eq!(
            clean_mime("text/plain; charset=utf-8").unwrap(),
            "text/plain; charset=utf-8"
        );
        assert!(clean_mime("nada").is_err());
        assert!(clean_mime("a/b c").is_err());
    }
}
