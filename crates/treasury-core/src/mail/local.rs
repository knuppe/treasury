//! Fase 1: a pasta local vigiada (docs/email-ingest.md §Pasta local). Sem credencial: o
//! usuário salva ali anexos, extratos e mensagens `.eml` (ou aponta para a pasta onde o
//! cliente de e-mail salva os anexos). Cada subpasta é uma "pasta" dos filtros (a raiz é
//! `INBOX`); o `.eml` passa pelos mesmos filtros de uma caixa IMAP. A retenção move o arquivo
//! para `Processados` (padrão) ou manda para a Lixeira do sistema.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::config::LocalRetention;
use super::filter;
use super::ingest::Incoming;
use super::message::MAX_MESSAGE_BYTES;
use super::retention::Candidate;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

/// Texto do catálogo com um argumento.
fn with(key: &str, name: &'static str, value: String) -> String {
    t_args(key, &args(&[(name, value.into())]))
}

/// Subpasta para onde a retenção move o que já foi tratado (nunca é lida).
pub const PROCESSED_DIR: &str = "Processados";
const EXTENSIONS: &[&str] = &[
    "eml", "pdf", "csv", "ofx", "qfx", "xml", "xlsx", "xls", "ods", "txt", "png", "jpg", "jpeg",
    "heic", "webp",
];
const MAX_DEPTH: usize = 4;
/// Arquivos por busca (o resto fica para a próxima).
const MAX_FILES: usize = 500;
/// Arquivo mexido há menos que isso pode estar sendo copiado ainda.
const SETTLE: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFile {
    /// Subpasta relativa à raiz (`INBOX` na raiz, `Bancos/Nubank`…).
    pub folder: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    /// `YYYY-MM-DD` da modificação.
    pub date: Option<String>,
}

impl LocalFile {
    /// Marca do arquivo recusado (para não contar de novo a cada busca).
    pub fn seen_key(&self) -> String {
        format!("{}/{}:{}", self.folder, self.name, self.size)
    }

    pub fn is_message(&self) -> bool {
        self.name.to_ascii_lowercase().ends_with(".eml")
    }
}

fn wanted(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.rsplit_once('.')
        .is_some_and(|(stem, ext)| !stem.is_empty() && EXTENSIONS.contains(&ext))
}

/// Os arquivos das pastas permitidas (sem seguir atalhos, sem ocultos, sem `Processados`).
pub fn scan(root: &Path, allowed: &[String], blocked: &[String]) -> Result<Vec<LocalFile>> {
    if !root.is_dir() {
        return Err(Error::invalid(with(
            "nucleo-email-local-erro-pasta",
            "pasta",
            root.display().to_string(),
        )));
    }
    let mut out = vec![];
    let now = SystemTime::now();
    let mut stack: Vec<(PathBuf, Vec<String>)> = vec![(root.to_path_buf(), vec![])];
    while let Some((dir, rel)) = stack.pop() {
        let folder = if rel.is_empty() {
            "INBOX".to_string()
        } else {
            rel.join("/")
        };
        let folder_ok = allowed
            .iter()
            .any(|f| filter::folder_matches(f, &folder, &[]))
            && !blocked
                .iter()
                .any(|f| filter::folder_matches(f, &folder, &[]));
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                if rel.len() + 1 < MAX_DEPTH && !(rel.is_empty() && name == PROCESSED_DIR) {
                    let mut r = rel.clone();
                    r.push(name);
                    stack.push((e.path(), r));
                }
                continue;
            }
            if !folder_ok || !ft.is_file() || !wanted(&name) {
                continue;
            }
            let Ok(meta) = e.metadata() else { continue };
            if meta.len() == 0 || meta.len() > MAX_MESSAGE_BYTES as u64 {
                continue;
            }
            let modified = meta.modified().ok();
            if modified.is_some_and(|m| now.duration_since(m).unwrap_or_default() < SETTLE) {
                continue;
            }
            out.push(LocalFile {
                folder: folder.clone(),
                name,
                path: e.path(),
                size: meta.len(),
                date: modified.map(|m| {
                    chrono::DateTime::<chrono::Local>::from(m)
                        .date_naive()
                        .format("%Y-%m-%d")
                        .to_string()
                }),
            });
            if out.len() >= MAX_FILES {
                return Ok(out);
            }
        }
    }
    Ok(out)
}

/// Já tratado (na fila) ou já recusado?
pub fn known(conn: &rusqlite::Connection, account_id: &str, f: &LocalFile) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM mail_message WHERE account_id = ?1 AND folder = ?2 AND file_name = ?3
                          AND size = ?4 AND deleted_at IS NULL)
             OR EXISTS (SELECT 1 FROM mail_seen WHERE account_id = ?1 AND key = ?5)",
        rusqlite::params![account_id, f.folder, f.name, f.size as i64, f.seen_key()],
        |r| r.get(0),
    )?)
}

pub fn mark_seen(conn: &rusqlite::Connection, account_id: &str, f: &LocalFile) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO mail_seen (account_id, key, at) VALUES (?1, ?2, ?3)",
        rusqlite::params![account_id, f.seen_key(), crate::time::now_utc()],
    )?;
    Ok(())
}

pub fn incoming(f: &LocalFile) -> Result<Incoming> {
    let raw = std::fs::read(&f.path)?;
    Ok(Incoming {
        folder: f.folder.clone(),
        folder_flags: vec![],
        uid: None,
        uidvalidity: None,
        file_name: Some(f.name.clone()),
        is_message: f.is_message(),
        file_date: f.date.clone(),
        raw,
    })
}

/// Caminho do arquivo de uma mensagem, conferindo que continua dentro da pasta da conta.
fn path_of(root: &Path, c: &Candidate) -> Result<PathBuf> {
    let name = c
        .file_name
        .as_deref()
        .ok_or_else(|| Error::invalid(t("nucleo-email-local-erro-sem-arquivo")))?;
    if name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(Error::invalid(t("nucleo-email-local-erro-nome")));
    }
    let dir = if c.folder == "INBOX" {
        root.to_path_buf()
    } else {
        let mut d = root.to_path_buf();
        for part in c.folder.split('/') {
            if part.is_empty() || part == "." || part == ".." {
                return Err(Error::invalid(t("nucleo-email-local-erro-subpasta")));
            }
            d.push(part);
        }
        d
    };
    let p = dir.join(name);
    let real = p.canonicalize().map_err(|_| {
        Error::not_found(with(
            "nucleo-email-local-ref-arquivo",
            "arquivo",
            name.to_string(),
        ))
    })?;
    let real_root = root.canonicalize()?;
    if !real.starts_with(&real_root) || std::fs::symlink_metadata(&p)?.file_type().is_symlink() {
        return Err(Error::invalid(t("nucleo-email-local-erro-fora")));
    }
    Ok(real)
}

/// Tira o arquivo da pasta: move para `Processados/<subpasta>` ou manda para a Lixeira.
pub fn remove(
    root: &Path,
    c: &Candidate,
    fallback_trash: &Path,
) -> std::result::Result<String, String> {
    let path = path_of(root, c).map_err(|e| e.message())?;
    match c.retention.local {
        LocalRetention::Trash => crate::trash::move_to_trash(&path, fallback_trash)
            .map(|_| t("nucleo-email-local-lixeira"))
            .map_err(|e| e.message()),
        LocalRetention::Move => {
            let mut dest = root.join(PROCESSED_DIR);
            if c.folder != "INBOX" {
                for part in c.folder.split('/') {
                    dest.push(part);
                }
            }
            move_free(&path, &dest)
                .map(|p| {
                    with(
                        "nucleo-email-local-movido",
                        "pasta",
                        p.strip_prefix(root).unwrap_or(&p).display().to_string(),
                    )
                })
                .map_err(|e| e.message())
        }
    }
}

/// Move para a pasta com um nome livre (`extrato.ofx`, `extrato (2).ofx`…).
fn move_free(path: &Path, dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| Error::invalid(t("nucleo-email-local-erro-caminho")))?;
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), format!(".{e}")),
        _ => (name.clone(), String::new()),
    };
    let dest = std::iter::once(dir.join(&name))
        .chain((2..1000).map(|n| dir.join(format!("{stem} ({n}){ext}"))))
        .find(|p| !p.exists())
        .ok_or_else(|| Error::conflict(t("nucleo-email-local-erro-repetidos")))?;
    std::fs::rename(path, &dest)?;
    Ok(dest)
}
