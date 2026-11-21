//! Caixa de entrada de arquivos (docs/inbox.md).
//!
//! Uma pasta local (padrão `~/Treasury/Inbox`, ajuste `inbox_folder`) que o app acompanha com o
//! cofre aberto. O arquivo que chega é lido (até 20 MB), reconhecido pelo conteúdo (o mesmo
//! `import.sniff` da importação), guardado em `inbox_item` (dentro do cofre cifrado) e sai da
//! pasta. O mesmo arquivo (mesmo SHA-256) não vira dois itens. Nada sai do computador.
//!
//! Agentes externos (MCP) gravam o arquivo na pasta e chamam a prévia ou a importação com
//! `inbox_id` em vez de mandar o conteúdo em base64 (`mcp::inbox_files`).

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use base64::Engine;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use super::sniff::{FileKind, SniffArgs, sniff};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

/// Maior arquivo aceito.
pub const MAX_BYTES: u64 = 20 * 1024 * 1024;
/// Arquivo mexido há menos que isso ainda pode estar sendo gravado: fica para a próxima leitura.
pub const SETTLE: Duration = Duration::from_secs(1);
/// Ajuste da pasta (vazio: a padrão).
pub const FOLDER_SETTING: &str = "inbox_folder";
/// Ajuste "acompanhar a caixa de entrada" (ligado por padrão).
pub const WATCH_SETTING: &str = "watch_inbox";

/// Arquivos parciais e temporários (navegadores, editores, cópias em andamento).
const PARTIAL: &[&str] = &[
    "crdownload",
    "part",
    "partial",
    "download",
    "opdownload",
    "tmp",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InboxStatus {
    /// Reconhecido, esperando a prévia e a importação.
    Pending,
    Imported,
    /// A pessoa descartou.
    Dismissed,
    /// Tipo que o Treasury não lê.
    Unrecognized,
}

impl InboxStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Imported => "imported",
            Self::Dismissed => "dismissed",
            Self::Unrecognized => "unrecognized",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "imported" => Self::Imported,
            "dismissed" => Self::Dismissed,
            "unrecognized" => Self::Unrecognized,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InboxItem {
    pub id: Id,
    pub filename: String,
    #[ts(type = "number")]
    pub size: i64,
    pub sha256: String,
    /// O tipo pelo conteúdo: `statement`, `brokerage_note`, `b3_report`, `pension_statement`,
    /// `receipt`, `image` ou `unknown`.
    pub detected_kind: FileKind,
    /// O que foi reconhecido ("Extrato OFX (Nubank)").
    pub label: String,
    pub status: InboxStatus,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InboxScan {
    /// A pasta acompanhada.
    pub folder: String,
    /// Itens novos desta leitura.
    pub added: Vec<InboxItem>,
    /// Arquivos iguais a um item que já existe (saíram da pasta sem criar outro).
    pub duplicates: u32,
    /// Arquivos maiores que o limite (ficam na pasta).
    pub too_large: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct InboxListArgs {
    /// Também os já importados e descartados (padrão: só os pendentes e os não reconhecidos).
    #[serde(default)]
    #[ts(optional)]
    pub include_done: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InboxFile {
    pub id: Id,
    pub filename: String,
    pub content_base64: String,
}

/// A pasta padrão: `~/Treasury/Inbox`.
pub fn default_folder() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join("Treasury").join("Inbox"))
}

/// A pasta acompanhada (o ajuste ou a padrão).
pub fn folder(conn: &Connection) -> Result<Option<PathBuf>> {
    let custom: String = crate::settings::get_or(conn, FOLDER_SETTING, String::new())?;
    Ok(match custom.trim() {
        "" => default_folder(),
        p => Some(PathBuf::from(p)),
    })
}

/// O nome serve? (ocultos e parciais: não)
pub fn candidate(name: &str) -> bool {
    if name.starts_with('.') || name.starts_with("~$") {
        return false;
    }
    let lower = name.to_lowercase();
    match lower.rsplit_once('.') {
        Some((stem, ext)) => !stem.is_empty() && !PARTIAL.contains(&ext),
        None => true,
    }
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

const COLUMNS: &str = "id, filename, size, sha256, detected_kind, label, status, note, created_at";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<InboxItem> {
    let kind: String = r.get(4)?;
    Ok(InboxItem {
        id: r.get(0)?,
        filename: r.get(1)?,
        size: r.get(2)?,
        sha256: r.get(3)?,
        detected_kind: serde_json::from_value(serde_json::Value::String(kind))
            .unwrap_or(FileKind::Unknown),
        label: r.get(5)?,
        status: InboxStatus::parse(&r.get::<_, String>(6)?),
        note: r.get(7)?,
        created_at: r.get(8)?,
    })
}

fn kind_str(k: FileKind) -> String {
    serde_json::to_value(k)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

/// Guarda um arquivo na caixa (ou devolve `None` se o mesmo conteúdo já está lá).
pub fn add(conn: &Connection, filename: &str, bytes: &[u8]) -> Result<Option<InboxItem>> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(Error::invalid(t("nucleo-caixa-erro-grande")));
    }
    let sha = hex(bytes);
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM inbox_item WHERE sha256 = ?1)",
        [&sha],
        |r| r.get(0),
    )?;
    if exists {
        return Ok(None);
    }
    let content_base64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    // Um arquivo que nem a leitura entende não quebra nada: fica como não reconhecido.
    let (kind, label, note) = match sniff(&SniffArgs {
        filename: filename.to_string(),
        content_base64,
    }) {
        Ok(s) => (s.kind, s.label, s.note),
        Err(e) => (
            FileKind::Unknown,
            t("nucleo-caixa-desconhecido"),
            Some(e.to_string()),
        ),
    };
    let status = if kind == FileKind::Unknown {
        InboxStatus::Unrecognized
    } else {
        InboxStatus::Pending
    };
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO inbox_item (id, filename, size, sha256, detected_kind, label, status, note,
                                 content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![
            id,
            filename,
            bytes.len() as i64,
            sha,
            kind_str(kind),
            label,
            status.as_str(),
            note,
            bytes,
            now
        ],
    )?;
    get(conn, &id).map(Some)
}

/// Lê a pasta: o que chegou vira item da caixa e sai da pasta (o conteúdo fica no cofre).
pub fn scan_folder(conn: &Connection, dir: &Path, now: SystemTime) -> Result<InboxScan> {
    let mut out = InboxScan {
        folder: dir.display().to_string(),
        ..InboxScan::default()
    };
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
        return Ok(out);
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().to_string();
        if !candidate(&name) {
            continue;
        }
        let Ok(meta) = e.metadata() else { continue };
        if meta
            .modified()
            .ok()
            .and_then(|m| now.duration_since(m).ok())
            .is_some_and(|age| age < SETTLE)
        {
            continue;
        }
        if meta.len() > MAX_BYTES {
            out.too_large.push(name);
            continue;
        }
        let Ok(bytes) = std::fs::read(e.path()) else {
            continue;
        };
        if bytes.is_empty() {
            continue;
        }
        match add(conn, &name, &bytes)? {
            Some(item) => out.added.push(item),
            None => out.duplicates += 1,
        }
        // Guardado (ou já estava): sai da pasta.
        if let Err(err) = std::fs::remove_file(e.path()) {
            tracing::warn!("caixa de entrada: não deu para tirar {name} da pasta: {err}");
        }
    }
    Ok(out)
}

/// Lê a pasta do ajuste (com o acompanhamento ligado).
pub fn scan(conn: &Connection) -> Result<InboxScan> {
    if !crate::settings::get_or(conn, WATCH_SETTING, true)? {
        return Ok(InboxScan::default());
    }
    match folder(conn)? {
        Some(dir) => scan_folder(conn, &dir, SystemTime::now()),
        None => Ok(InboxScan::default()),
    }
}

pub fn get(conn: &Connection, id: &str) -> Result<InboxItem> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM inbox_item WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-caixa-item",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

pub fn list(conn: &Connection, a: &InboxListArgs) -> Result<Vec<InboxItem>> {
    let all = a.include_done.unwrap_or(false);
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM inbox_item
          WHERE ?1 OR status IN ('pending', 'unrecognized')
          ORDER BY created_at DESC, id"
    ))?;
    let rows = stmt
        .query_map([all], row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// O arquivo guardado, para a prévia e a importação.
pub fn file(conn: &Connection, id: &str) -> Result<InboxFile> {
    let item = get(conn, id)?;
    let bytes: Vec<u8> =
        conn.query_row("SELECT content FROM inbox_item WHERE id = ?1", [id], |r| {
            r.get(0)
        })?;
    Ok(InboxFile {
        id: item.id,
        filename: item.filename,
        content_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}

pub fn set_status(conn: &Connection, id: &str, status: InboxStatus) -> Result<InboxItem> {
    get(conn, id)?;
    conn.execute(
        "UPDATE inbox_item SET status = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, status.as_str(), now_utc()],
    )?;
    get(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> (tempfile::TempDir, crate::storage::Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST)
            .unwrap();
        (dir, db)
    }

    const OFX: &str = "OFXHEADER:100\nDATA:OFXSGML\n<OFX><BANKMSGSRSV1><STMTTRNRS><STMTRS><CURDEF>BRL\
        <BANKACCTFROM><BANKID>0260<ACCTID>12345</BANKACCTFROM><BANKTRANLIST>\
        <STMTTRN><TRNTYPE>DEBIT<DTPOSTED>20261001<TRNAMT>-10.00<FITID>A1<MEMO>PADARIA EXEMPLO\
        </STMTTRN></BANKTRANLIST></STMTRS></STMTTRNRS></BANKMSGSRSV1></OFX>";

    #[test]
    fn files_in_the_folder_become_items_once_and_leave_the_folder() {
        let (_d, db) = conn();
        let inbox = tempfile::tempdir().unwrap();
        let later = SystemTime::now() + Duration::from_secs(10);
        std::fs::write(inbox.path().join("extrato.ofx"), OFX).unwrap();
        std::fs::write(inbox.path().join("copia.ofx"), OFX).unwrap();
        std::fs::write(inbox.path().join("lista.bin"), b"\x00\x01\x02 nada").unwrap();
        std::fs::write(inbox.path().join("baixando.pdf.crdownload"), b"%PDF").unwrap();
        std::fs::write(inbox.path().join(".oculto"), b"x").unwrap();
        let r = scan_folder(db.conn(), inbox.path(), later).unwrap();
        // O mesmo conteúdo duas vezes vira um item só.
        assert_eq!((r.added.len(), r.duplicates), (2, 1));
        let ofx = r.added.iter().find(|i| i.filename == "copia.ofx").unwrap();
        assert_eq!(
            (ofx.detected_kind, ofx.status),
            (FileKind::Statement, InboxStatus::Pending)
        );
        // Tipo desconhecido: guardado como não reconhecido, sem quebrar a leitura.
        let bin = r.added.iter().find(|i| i.filename == "lista.bin").unwrap();
        assert_eq!(bin.status, InboxStatus::Unrecognized);
        // Os lidos saíram da pasta; o parcial e o oculto ficaram.
        let mut left: Vec<String> = std::fs::read_dir(inbox.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        left.sort();
        assert_eq!(left, [".oculto", "baixando.pdf.crdownload"]);
        // O conteúdo guardado é o mesmo do arquivo.
        let f = file(db.conn(), &ofx.id).unwrap();
        let back = base64::engine::general_purpose::STANDARD
            .decode(f.content_base64)
            .unwrap();
        assert_eq!(back, OFX.as_bytes());
        // Copiado de novo depois: não cria outro item.
        std::fs::write(inbox.path().join("de-novo.ofx"), OFX).unwrap();
        let r = scan_folder(db.conn(), inbox.path(), later).unwrap();
        assert_eq!((r.added.len(), r.duplicates), (0, 1));
        assert_eq!(list(db.conn(), &InboxListArgs::default()).unwrap().len(), 2);
        set_status(db.conn(), &ofx.id, InboxStatus::Imported).unwrap();
        assert_eq!(list(db.conn(), &InboxListArgs::default()).unwrap().len(), 1);
    }

    #[test]
    fn a_file_still_being_written_waits() {
        let (_d, db) = conn();
        let inbox = tempfile::tempdir().unwrap();
        std::fs::write(inbox.path().join("extrato.ofx"), OFX).unwrap();
        let r = scan_folder(db.conn(), inbox.path(), SystemTime::now()).unwrap();
        assert!(r.added.is_empty());
        assert!(inbox.path().join("extrato.ofx").exists());
        // Pasta que não existe é criada.
        let fresh = inbox.path().join("nova");
        scan_folder(db.conn(), &fresh, SystemTime::now()).unwrap();
        assert!(fresh.is_dir());
    }
}
