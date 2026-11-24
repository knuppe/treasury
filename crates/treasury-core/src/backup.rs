//! Backups gerenciados (D13): cópias cifradas, verificadas (`integrity_check` + SHA-256),
//! com rotação avô-pai-filho e restauração segura.
//!
//! - Mesma chave do banco: API de backup online do SQLite (cópia página a página).
//! - Senha de backup própria: `sqlcipher_export()` para um banco anexado com a outra chave.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::settings;
use crate::storage::{Db, apply_key};
use crate::time::now_utc;
use crate::vault::{self, DbKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BackupKind {
    AutoDaily,
    Manual,
    PreMigration,
    PreImport,
    PreAi,
    PreRestore,
    PreRekey,
    OnClose,
}

impl BackupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BackupKind::AutoDaily => "auto_daily",
            BackupKind::Manual => "manual",
            BackupKind::PreMigration => "pre_migration",
            BackupKind::PreImport => "pre_import",
            BackupKind::PreAi => "pre_ai",
            BackupKind::PreRestore => "pre_restore",
            BackupKind::PreRekey => "pre_rekey",
            BackupKind::OnClose => "on_close",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "manual" => BackupKind::Manual,
            "pre_migration" => BackupKind::PreMigration,
            "pre_import" => BackupKind::PreImport,
            "pre_ai" => BackupKind::PreAi,
            "pre_restore" => BackupKind::PreRestore,
            "pre_rekey" => BackupKind::PreRekey,
            "on_close" => BackupKind::OnClose,
            _ => BackupKind::AutoDaily,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum KeyMode {
    /// Mesma senha do cofre.
    Same,
    /// Senha de backup própria.
    Separate,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackupRecord {
    pub id: Id,
    pub created_at: String,
    pub kind: BackupKind,
    pub path: String,
    pub destination: String,
    pub key_mode: KeyMode,
    #[ts(type = "number")]
    pub size_bytes: i64,
    pub sha256: Option<String>,
    pub verified: bool,
    /// Por que a verificação falhou, no idioma da instalação no momento da verificação.
    pub verify_error: Option<String>,
    /// O arquivo ainda existe no disco?
    pub exists: bool,
    /// A verificação que acabou de rodar parou na senha (a de backup ou a usada quando o
    /// backup foi feito): a interface pede a senha. Só vale na resposta de `backup.verify`.
    pub needs_password: bool,
}

/// Configuração de backup (guardada dentro do banco, chave `backup`).
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct BackupSettings {
    /// Hora local (0–23) do backup automático diário.
    pub daily_hour: u32,
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
    /// Pastas adicionais (disco externo, pasta sincronizada). Sempre cifrado.
    pub extra_destinations: Vec<String>,
    /// Backup ao fechar o app quando houve mudanças.
    pub on_close: bool,
    pub key_mode: KeyMode,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            daily_hour: 3,
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
            extra_destinations: vec![],
            on_close: true,
            key_mode: KeyMode::Same,
        }
    }
}

pub const SETTINGS_KEY: &str = "backup";
/// Chave+salt derivados da senha de backup própria (dentro do banco cifrado).
const SEPARATE_KEY_SETTING: &str = "backup.separate_key";

pub fn load_settings(conn: &Connection) -> Result<BackupSettings> {
    Ok(settings::get(conn, SETTINGS_KEY)?.unwrap_or_default())
}

pub fn save_settings(conn: &Connection, s: &BackupSettings) -> Result<()> {
    if s.daily_hour > 23 {
        return Err(Error::invalid(t("nucleo-geral-erro-backup-hora")));
    }
    if s.key_mode == KeyMode::Separate && separate_key(conn)?.is_none() {
        return Err(Error::invalid(t("nucleo-geral-erro-backup-senha-antes")));
    }
    settings::set(conn, SETTINGS_KEY, s)
}

/// Define (ou troca) a senha de backup própria.
pub fn set_separate_password(conn: &Connection, password: &str) -> Result<()> {
    if password.chars().count() < 8 {
        return Err(Error::invalid(t_args(
            "nucleo-geral-erro-backup-senha-curta",
            &args(&[("minimo", 8.into())]),
        )));
    }
    let key = DbKey::derive(password, DbKey::random_salt()?, vault::default_kdf())?;
    settings::set(conn, SEPARATE_KEY_SETTING, &key.to_secret_hex().as_str())
}

fn separate_key(conn: &Connection) -> Result<Option<DbKey>> {
    let hex: Option<String> = settings::get(conn, SEPARATE_KEY_SETTING)?;
    hex.map(|h| DbKey::from_secret_hex(&h)).transpose()
}

fn file_name(now: DateTime<Utc>, kind: BackupKind) -> String {
    format!(
        "treasury-{}-{}.db.enc",
        now.format("%Y-%m-%dT%H-%M-%S%.3fZ"),
        kind.as_str()
    )
}

/// Lê data e tipo do nome do arquivo (`treasury-2026-10-14T03-00-00.000Z-auto_daily.db.enc`).
fn parse_file_name(name: &str) -> Option<(DateTime<Utc>, BackupKind)> {
    let rest = name.strip_prefix("treasury-")?.strip_suffix(".db.enc")?;
    let (ts, kind) = rest.split_once("Z-")?;
    let t = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H-%M-%S%.3f").ok()?;
    Some((t.and_utc(), BackupKind::parse(kind)))
}

/// Registra arquivos de backup da pasta que não estão no banco (ex.: depois de uma
/// restauração, os backups feitos após aquele ponto continuam no disco).
pub fn sync_dir(conn: &Connection, dir: &Path) -> Result<u32> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(0);
    };
    let mut added = 0;
    for e in entries.flatten() {
        let path = e.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some((ts, kind)) = parse_file_name(name) else {
            continue;
        };
        let path_s = path.to_string_lossy().into_owned();
        let known: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM backup_record WHERE path = ?1)",
            [&path_s],
            |r| r.get(0),
        )?;
        if known {
            continue;
        }
        let size = e.metadata().map(|m| m.len() as i64).unwrap_or(0);
        conn.execute(
            "INSERT INTO backup_record (id, created_at, kind, path, destination, key_mode, size_bytes, sha256, verified, verify_error)
             VALUES (?1, ?2, ?3, ?4, 'primary', 'same', ?5, ?6, 0, ?7)",
            params![new_id(), ts.to_rfc3339_opts(chrono::SecondsFormat::Millis, true), kind.as_str(), path_s, size, sha256_file(&path).ok(), t("nucleo-geral-backup-encontrado")],
        )?;
        added += 1;
    }
    Ok(added)
}

fn sha256_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

/// Copia o banco aberto para `dest` cifrado com `key` (que já traz o salt do destino).
pub(crate) fn write_copy(
    src: &Connection,
    src_key: &DbKey,
    dest: &Path,
    key: &DbKey,
) -> Result<()> {
    if key.key_bytes() == src_key.key_bytes() && key.salt() == src_key.salt() {
        let mut dst = Connection::open(dest)?;
        apply_key(&dst, key)?;
        {
            let b = rusqlite::backup::Backup::new(src, &mut dst)?;
            b.run_to_completion(256, std::time::Duration::ZERO, None)?;
        }
        dst.close().map_err(|(_, e)| Error::Db(e))?;
    } else {
        let v = key.pragma_value();
        src.execute(
            "ATTACH DATABASE ?1 AS backup_target KEY ?2",
            params![dest.to_string_lossy(), v.as_str()],
        )
        .map_err(|e| {
            Error::Other(t_args(
                "nucleo-geral-erro-backup-criar",
                &args(&[("erro", e.to_string().into())]),
            ))
        })?;
        let res = src.query_row("SELECT sqlcipher_export('backup_target')", [], |_| Ok(()));
        let _ = src.execute("DETACH DATABASE backup_target", []);
        res?;
    }
    crate::paths::set_mode(dest, 0o600)?;
    Ok(())
}

/// Dados públicos baixados (CVM, SEC, B3) que a próxima coleta refaz: ficam fora dos
/// backups para não multiplicar dezenas de MB em cada cópia da rotação. `research_file`
/// (ETags) sai junto, para a coleta depois de uma restauração baixar tudo de novo.
pub const PUBLIC_DATA_TABLES: &[&str] = &[
    "fundamental",
    "corporate_document",
    "research_quote",
    "research_universe",
    "research_file",
    "score_universe",
    "market_score",
    "research_dividend",
    "research_stock_event",
    "research_fetch_log",
    "research_fund_notice",
    "research_filing",
    "research_auditor",
    "bank_institution",
    "bank_prudential",
    "bank_ifdata_fetch",
];

/// Esvazia as tabelas de dados públicos da cópia e compacta o arquivo.
fn slim_copy(path: &Path, key: &DbKey) -> Result<()> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    apply_key(&conn, key)?;
    let mut removed = 0;
    for t in PUBLIC_DATA_TABLES {
        let exists: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [t],
            |r| r.get(0),
        )?;
        if exists {
            removed += conn.execute(&format!("DELETE FROM {t}"), [])?;
        }
    }
    if removed > 0 {
        conn.execute_batch("VACUUM")?;
    }
    conn.close().map_err(|(_, e)| Error::Db(e))?;
    Ok(())
}

/// Abre um arquivo de backup e confere integridade criptográfica e lógica.
pub fn verify_file(path: &Path, key: &DbKey) -> Result<()> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    apply_key(&conn, key)?;
    let ok: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|_| Error::WrongPassword)?;
    if ok != "ok" {
        return Err(Error::Other(format!("integrity_check: {ok}")));
    }
    let mut stmt = conn.prepare("PRAGMA cipher_integrity_check")?;
    let problems: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    if !problems.is_empty() {
        return Err(Error::Other(format!(
            "cipher_integrity_check: {}",
            problems.join("; ")
        )));
    }
    Ok(())
}

/// Cria, verifica e registra um backup. Aplica a rotação depois.
pub fn create(db: &Db, dir: &Path, kind: BackupKind) -> Result<BackupRecord> {
    let conn = db.conn();
    let s = load_settings(conn)?;
    let (key_mode, base_key) = match (s.key_mode, separate_key(conn)?) {
        (KeyMode::Separate, Some(k)) => (KeyMode::Separate, k),
        _ => (KeyMode::Same, db.key().clone()),
    };
    let rec = create_in(db, dir, kind, key_mode, &base_key, "primary")?;
    for extra in &s.extra_destinations {
        // Destinos adicionais são best-effort: falha registrada, não interrompe.
        let _ = create_in(db, Path::new(extra), kind, key_mode, &base_key, extra);
    }
    rotate(conn, &s)?;
    Ok(rec)
}

fn create_in(
    db: &Db,
    dir: &Path,
    kind: BackupKind,
    key_mode: KeyMode,
    key: &DbKey,
    destination: &str,
) -> Result<BackupRecord> {
    crate::paths::create_private_dir(dir)?;
    let now = Utc::now();
    let final_path = dir.join(file_name(now, kind));
    let partial = final_path.with_extension("partial");
    let _ = std::fs::remove_file(&partial);
    write_copy(db.conn(), db.key(), &partial, key)?;
    slim_copy(&partial, key)?;
    let (verified, verify_error) = match verify_file(&partial, key) {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.message())),
    };
    std::fs::rename(&partial, &final_path)?;
    let size = std::fs::metadata(&final_path)?.len() as i64;
    let sha = sha256_file(&final_path)?;
    let rec = BackupRecord {
        id: new_id(),
        created_at: now_utc(),
        kind,
        path: final_path.to_string_lossy().into_owned(),
        destination: destination.to_string(),
        key_mode,
        size_bytes: size,
        sha256: Some(sha),
        verified,
        verify_error,
        exists: true,
        needs_password: false,
    };
    db.conn().execute(
        "INSERT INTO backup_record (id, created_at, kind, path, destination, key_mode, size_bytes, sha256, verified, verify_error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            rec.id,
            rec.created_at,
            kind.as_str(),
            rec.path,
            rec.destination,
            if key_mode == KeyMode::Same { "same" } else { "separate" },
            rec.size_bytes,
            rec.sha256,
            verified as i64,
            rec.verify_error,
        ],
    )?;
    Ok(rec)
}

fn record_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<BackupRecord> {
    let path: String = r.get("path")?;
    Ok(BackupRecord {
        id: r.get("id")?,
        created_at: r.get("created_at")?,
        kind: BackupKind::parse(&r.get::<_, String>("kind")?),
        exists: Path::new(&path).exists(),
        path,
        destination: r.get("destination")?,
        key_mode: if r.get::<_, String>("key_mode")? == "separate" {
            KeyMode::Separate
        } else {
            KeyMode::Same
        },
        size_bytes: r.get("size_bytes")?,
        sha256: r.get("sha256")?,
        verified: r.get::<_, i64>("verified")? == 1,
        verify_error: r.get("verify_error")?,
        needs_password: false,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<BackupRecord>> {
    let mut stmt = conn
        .prepare("SELECT * FROM backup_record WHERE deleted_at IS NULL ORDER BY created_at DESC")?;
    Ok(stmt
        .query_map([], record_from_row)?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<BackupRecord> {
    conn.query_row(
        "SELECT * FROM backup_record WHERE id = ?1",
        [id],
        record_from_row,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-geral-backup",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

/// Reverifica um backup existente (arquivo, hash e integridade).
pub fn reverify(db: &Db, id: &str, backup_password: Option<&str>) -> Result<BackupRecord> {
    let rec = get(db.conn(), id)?;
    let path = PathBuf::from(&rec.path);
    // Parou na senha: falta a senha certa (a interface pede), não é defeito do arquivo.
    let mut needs_password = false;
    let result = (|| -> Result<()> {
        if !path.exists() {
            return Err(Error::not_found(t("nucleo-geral-arquivo-backup")));
        }
        if let Some(expected) = &rec.sha256
            && &sha256_file(&path)? != expected
        {
            return Err(Error::Other(t("nucleo-geral-erro-backup-hash")));
        }
        let key = backup_key(db, &rec, &path, backup_password).inspect_err(|e| {
            needs_password = matches!(e, Error::Invalid(_) | Error::WrongPassword);
        })?;
        verify_file(&path, &key).inspect_err(|e| {
            needs_password = matches!(e, Error::WrongPassword);
        })
    })();
    let (verified, err) = match &result {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.message())),
    };
    db.conn().execute(
        "UPDATE backup_record SET verified = ?2, verify_error = ?3 WHERE id = ?1",
        params![id, verified as i64, err],
    )?;
    Ok(BackupRecord {
        needs_password,
        ..get(db.conn(), id)?
    })
}

fn backup_key(
    db: &Db,
    rec: &BackupRecord,
    path: &Path,
    backup_password: Option<&str>,
) -> Result<DbKey> {
    let salt = vault::read_salt(path)?;
    if let Some(pw) = backup_password {
        return derive_any(pw, salt, path);
    }
    match rec.key_mode {
        KeyMode::Same => {
            if db.key().salt() == salt {
                Ok(db.key().clone())
            } else {
                Err(Error::invalid(t("nucleo-geral-erro-backup-senha-antiga")))
            }
        }
        KeyMode::Separate => separate_key(db.conn())?
            .filter(|k| k.salt() == salt)
            .ok_or_else(|| Error::invalid(t("nucleo-geral-erro-backup-senha"))),
    }
}

/// Deriva a chave tentando as versões de KDF conhecidas, conferindo no arquivo.
fn derive_any(password: &str, salt: [u8; vault::SALT_LEN], path: &Path) -> Result<DbKey> {
    for params in vault::kdf_versions() {
        let key = DbKey::derive(password, salt, params)?;
        let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        apply_key(&conn, &key)?;
        if conn
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
                r.get::<_, i64>(0)
            })
            .is_ok()
        {
            return Ok(key);
        }
    }
    Err(Error::WrongPassword)
}

/// Prepara a restauração: abre o backup, exporta para um arquivo novo cifrado com a
/// senha mestra atual (salt novo) e verifica. Devolve o caminho pronto para substituir
/// o banco. O chamador fecha o banco atual, faz o backup `pre_restore` e troca os arquivos.
pub fn prepare_restore(
    backup_path: &Path,
    backup_password: &str,
    master_password: &str,
    target_db_path: &Path,
) -> Result<PathBuf> {
    let salt = vault::read_salt(backup_path)?;
    let bkey = derive_any(backup_password, salt, backup_path)?;
    verify_file(backup_path, &bkey)?;
    let staged = target_db_path.with_extension("restore-staging");
    let _ = std::fs::remove_file(&staged);
    let new_key = DbKey::derive(master_password, DbKey::random_salt()?, vault::default_kdf())?;
    // Leitura/escrita: com o principal só-leitura, o ATTACH do destino também seria só-leitura.
    let src = Connection::open_with_flags(
        backup_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    apply_key(&src, &bkey)?;
    write_copy(&src, &bkey, &staged, &new_key)?;
    drop(src);
    {
        // Segredos do backup (cifrados com a chave dele) passam para a chave nova.
        let dst = Connection::open(&staged)?;
        apply_key(&dst, &new_key)?;
        if let Err(e) = crate::ai::secret::rewrap(&dst, &[&bkey], &new_key) {
            tracing::warn!("segredos do backup: falha ao recifrar ({})", e.code());
        }
        // E-mail de ingestão: a conexão de um backup restaurado precisa ser confirmada de novo
        // (Testar) antes de buscar ou apagar algo no provedor. Backup sem a tabela: nada a fazer.
        let _ = dst.execute(
            "UPDATE mail_account SET needs_reconfirm = 1 WHERE kind = 'imap'",
            [],
        );
        dst.close().map_err(|(_, e)| Error::Db(e))?;
    }
    verify_file(&staged, &new_key)?;
    // Backup feito por uma versão mais nova do app: recusar antes de trocar o banco (depois
    // da troca, este app não conseguiria abrir o cofre).
    let compatible = (|| -> Result<()> {
        let conn = Connection::open(&staged)?;
        apply_key(&conn, &new_key)?;
        // Backup de antes da 1.0.0 (histórico V1 a V164): o esquema é o da V1; ao abrir, o
        // histórico é trocado (storage::Db::migrate).
        let schema: i32 = if crate::storage::is_legacy_history(&conn).unwrap_or(false) {
            0
        } else {
            conn.query_row(
                "SELECT COALESCE(MAX(version), 0) FROM refinery_schema_history",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0)
        };
        let known = crate::storage::latest_schema_version();
        if schema > known {
            return Err(Error::SchemaTooNew(t_args(
                "nucleo-base-erro-backup-esquema-novo",
                &args(&[
                    ("esquema", schema.to_string().into()),
                    ("conhecido", known.to_string().into()),
                ]),
            )));
        }
        Ok(())
    })();
    if let Err(e) = compatible {
        let _ = std::fs::remove_file(&staged);
        return Err(e);
    }
    Ok(staged)
}

/// Troca o banco pelo arquivo preparado (rename atômico na mesma pasta).
pub fn swap_in(staged: &Path, db_path: &Path) -> Result<()> {
    std::fs::rename(staged, db_path)?;
    crate::paths::set_mode(db_path, 0o600)
}

/// Rotação avô-pai-filho: mantém o mais recente de cada um dos últimos N dias, semanas
/// e meses. Backups manuais nunca são apagados automaticamente; os 3 mais recentes sempre ficam.
pub fn rotate(conn: &Connection, s: &BackupSettings) -> Result<Vec<Id>> {
    let all = list(conn)?;
    let auto: Vec<&BackupRecord> = all
        .iter()
        .filter(|b| b.kind != BackupKind::Manual && b.destination == "primary")
        .collect();
    let keep = gfs_keep(
        &auto
            .iter()
            .map(|b| (b.id.clone(), b.created_at.clone()))
            .collect::<Vec<_>>(),
        s,
    );
    let mut removed = vec![];
    for b in auto {
        if !keep.contains(&b.id) {
            let _ = std::fs::remove_file(&b.path);
            conn.execute(
                "UPDATE backup_record SET deleted_at = ?2 WHERE id = ?1",
                params![b.id, now_utc()],
            )?;
            removed.push(b.id.clone());
        }
    }
    Ok(removed)
}

/// Decide quais ids manter. `items` em qualquer ordem: (id, created_at RFC 3339).
pub fn gfs_keep(items: &[(Id, String)], s: &BackupSettings) -> std::collections::HashSet<Id> {
    use std::collections::{HashMap, HashSet};
    let mut parsed: Vec<(Id, DateTime<Utc>)> = items
        .iter()
        .filter_map(|(id, ts)| {
            DateTime::parse_from_rfc3339(ts)
                .ok()
                .map(|t| (id.clone(), t.with_timezone(&Utc)))
        })
        .collect();
    parsed.sort_by_key(|p| std::cmp::Reverse(p.1));
    let mut keep: HashSet<Id> = parsed.iter().take(3).map(|(id, _)| id.clone()).collect();
    let mut bucket = |key_fn: &dyn Fn(&DateTime<Utc>) -> (i32, u32), limit: u32| {
        let mut seen: HashMap<(i32, u32), ()> = HashMap::new();
        for (id, t) in &parsed {
            let k = key_fn(t);
            if seen.contains_key(&k) {
                continue;
            }
            if seen.len() as u32 >= limit {
                break;
            }
            seen.insert(k, ());
            keep.insert(id.clone());
        }
    };
    bucket(&|t| (t.year(), t.ordinal()), s.keep_daily);
    bucket(
        &|t| (t.iso_week().year(), t.iso_week().week()),
        s.keep_weekly,
    );
    bucket(&|t| (t.year(), t.month()), s.keep_monthly);
    keep
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackupHealth {
    pub last_valid_at: Option<String>,
    #[ts(type = "number | null")]
    pub hours_since_last_valid: Option<i64>,
    /// Mais de 48 h sem backup válido.
    pub warning: bool,
    pub daily_due: bool,
    #[ts(type = "number")]
    pub count: i64,
    #[ts(type = "number")]
    pub total_bytes: i64,
}

pub fn last_valid(conn: &Connection) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT max(created_at) FROM backup_record WHERE verified = 1 AND deleted_at IS NULL AND destination = 'primary'",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten())
}

pub fn health(
    conn: &Connection,
    now_local: chrono::DateTime<chrono::Local>,
) -> Result<BackupHealth> {
    let last = last_valid(conn)?;
    let hours = last
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|t| (now_local.with_timezone(&Utc) - t.with_timezone(&Utc)).num_hours());
    let (count, total): (i64, i64) = conn.query_row(
        "SELECT count(*), COALESCE(sum(size_bytes), 0) FROM backup_record WHERE deleted_at IS NULL",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok(BackupHealth {
        warning: hours.is_none_or(|h| h > 48),
        daily_due: daily_due(conn, now_local)?,
        last_valid_at: last,
        hours_since_last_valid: hours,
        count,
        total_bytes: total,
    })
}

/// O backup diário está pendente? Sim se já passou da hora configurada hoje e não há
/// backup automático diário feito desde então (ou nenhum backup válido nas últimas 24 h).
pub fn daily_due(conn: &Connection, now_local: chrono::DateTime<chrono::Local>) -> Result<bool> {
    use chrono::TimeZone;
    let s = load_settings(conn)?;
    let today_slot = now_local
        .date_naive()
        .and_hms_opt(s.daily_hour, 0, 0)
        .expect("hora válida");
    let slot = if now_local.naive_local() >= today_slot {
        today_slot
    } else {
        today_slot - chrono::Duration::days(1)
    };
    let slot_utc = chrono::Local
        .from_local_datetime(&slot)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let last: Option<String> = conn
        .query_row(
            "SELECT max(created_at) FROM backup_record WHERE verified = 1 AND deleted_at IS NULL AND destination = 'primary'
             AND kind IN ('auto_daily', 'manual', 'on_close', 'pre_migration', 'pre_import', 'pre_ai', 'pre_restore', 'pre_rekey')",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(
        match last.and_then(|s| DateTime::parse_from_rfc3339(&s).ok()) {
            Some(t) => t.with_timezone(&Utc) < slot_utc,
            None => true,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::KDF_TEST;

    fn setup() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("data/treasury.db"), "mestra", KDF_TEST).unwrap();
        db.conn()
            .execute("INSERT INTO member (id, name, created_at, updated_at) VALUES ('m1', 'Ana', 'a', 'a')", [])
            .unwrap();
        (dir, db)
    }

    #[test]
    fn backup_same_key_is_verified_and_restorable() {
        let (dir, db) = setup();
        let bdir = dir.path().join("backups");
        let rec = create(&db, &bdir, BackupKind::Manual).unwrap();
        assert!(rec.verified, "{:?}", rec.verify_error);
        assert!(rec.path.ends_with(".db.enc"));
        assert_eq!(rec.key_mode, KeyMode::Same);
        let again = reverify(&db, &rec.id, None).unwrap();
        assert!(again.verified);

        // Corrupting the file invalidates the hash.
        let mut bytes = std::fs::read(&rec.path).unwrap();
        let n = bytes.len();
        bytes[n - 100] ^= 0xff;
        std::fs::write(&rec.path, &bytes).unwrap();
        let bad = reverify(&db, &rec.id, None).unwrap();
        assert!(!bad.verified);
    }

    #[test]
    fn backups_leave_out_downloaded_public_data() {
        let (dir, db) = setup();
        db.conn()
            .execute_batch(
                "INSERT INTO research_issuer (id, kind, name, source, fetched_at) VALUES ('33000167000101', 'company_br', 'Petrobras', 'cvm', 'x');
                 INSERT INTO research_file (url, etag, fetched_at) VALUES ('https://dados.cvm.gov.br/x.zip', 'e1', 'x');",
            )
            .unwrap();
        let rec = create(&db, &dir.path().join("backups"), BackupKind::Manual).unwrap();
        assert!(rec.verified, "{:?}", rec.verify_error);
        let conn = Connection::open(&rec.path).unwrap();
        apply_key(&conn, db.key()).unwrap();
        let count = |t: &str| -> i64 {
            conn.query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(
            count("research_file"),
            0,
            "ETags are left out so the collector redoes everything"
        );
        assert_eq!(count("research_issuer"), 1);
        assert_eq!(count("member"), 1);
        // The database in use keeps everything.
        let live: i64 = db
            .conn()
            .query_row("SELECT count(*) FROM research_file", [], |r| r.get(0))
            .unwrap();
        assert_eq!(live, 1);
    }

    #[test]
    fn backup_with_separate_password_and_restore() {
        let (dir, db) = setup();
        set_separate_password(db.conn(), "senha-de-backup").unwrap();
        save_settings(
            db.conn(),
            &BackupSettings {
                key_mode: KeyMode::Separate,
                ..Default::default()
            },
        )
        .unwrap();
        let rec = create(&db, &dir.path().join("backups"), BackupKind::Manual).unwrap();
        assert!(rec.verified, "{:?}", rec.verify_error);
        assert_eq!(rec.key_mode, KeyMode::Separate);
        // The master password does not open the backup.
        assert!(
            derive_any(
                "mestra",
                vault::read_salt(Path::new(&rec.path)).unwrap(),
                Path::new(&rec.path)
            )
            .is_err()
        );

        // A change made after the backup disappears on restore.
        db.conn().execute("DELETE FROM member", []).unwrap();
        let db_path = db.path().to_path_buf();
        let staged = prepare_restore(
            Path::new(&rec.path),
            "senha-de-backup",
            "nova-mestra",
            &db_path,
        )
        .unwrap();
        db.close().unwrap();
        swap_in(&staged, &db_path).unwrap();
        let restored = Db::open(&db_path, "nova-mestra").unwrap();
        let n: i64 = restored
            .conn()
            .query_row("SELECT count(*) FROM member", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn gfs_rotation_keeps_expected_slots() {
        let s = BackupSettings {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
            ..Default::default()
        };
        // One backup per day for 120 days + 3 extras on the last day.
        let base = DateTime::parse_from_rfc3339("2026-10-14T03:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut items = vec![];
        for d in 0..120 {
            items.push((
                format!("d{d}"),
                (base - chrono::Duration::days(d)).to_rfc3339(),
            ));
        }
        for h in 1..=3 {
            items.push((
                format!("x{h}"),
                (base + chrono::Duration::hours(h)).to_rfc3339(),
            ));
        }
        let keep = gfs_keep(&items, &s);
        assert!(
            keep.contains("x3") && keep.contains("x2") && keep.contains("x1"),
            "3 most recent"
        );
        for d in 1..7 {
            assert!(keep.contains(&format!("d{d}")), "daily d{d}");
        }
        assert!(!keep.contains("d10") || keep.len() < 30);
        assert!(keep.len() <= 3 + 7 + 4 + 12);
        // The oldest (about June) is only kept if it is the month's representative.
        assert!(!keep.contains("d118"));
    }

    #[test]
    fn rotation_removes_files_but_keeps_manual() {
        let (dir, db) = setup();
        let bdir = dir.path().join("backups");
        let tight = BackupSettings {
            keep_daily: 1,
            keep_weekly: 0,
            keep_monthly: 0,
            ..Default::default()
        };
        settings::set(db.conn(), SETTINGS_KEY, &tight).unwrap();
        let manual = create(&db, &bdir, BackupKind::Manual).unwrap();
        let mut autos = vec![];
        for _ in 0..5 {
            autos.push(create(&db, &bdir, BackupKind::AutoDaily).unwrap());
        }
        let live = list(db.conn()).unwrap();
        assert!(live.iter().any(|b| b.id == manual.id));
        // The 3 most recent always stay; the oldest automatic ones go.
        assert_eq!(
            live.iter()
                .filter(|b| b.kind == BackupKind::AutoDaily)
                .count(),
            3
        );
        assert!(!Path::new(&autos[0].path).exists());
    }

    #[test]
    fn file_names_roundtrip() {
        let t = DateTime::parse_from_rfc3339("2026-10-14T03:00:00.123Z")
            .unwrap()
            .with_timezone(&Utc);
        let n = file_name(t, BackupKind::PreMigration);
        assert_eq!(n, "treasury-2026-10-14T03-00-00.123Z-pre_migration.db.enc");
        assert_eq!(parse_file_name(&n), Some((t, BackupKind::PreMigration)));
        assert_eq!(parse_file_name("outro.db"), None);
    }

    #[test]
    fn health_warns_without_backups() {
        let (_dir, db) = setup();
        let h = health(db.conn(), chrono::Local::now()).unwrap();
        assert!(h.warning);
        assert!(h.daily_due);
    }
}

#[cfg(test)]
mod reopen_tests {
    use super::*;
    use crate::vault::KDF_TEST;

    #[test]
    fn separate_password_backup_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        let db = Db::create(&p, "mestra", KDF_TEST).unwrap();
        set_separate_password(db.conn(), "senha-de-backup").unwrap();
        save_settings(
            db.conn(),
            &BackupSettings {
                key_mode: KeyMode::Separate,
                ..Default::default()
            },
        )
        .unwrap();
        db.close().unwrap();
        let db = Db::open(&p, "mestra").unwrap();
        let rec = create(&db, &dir.path().join("b"), BackupKind::Manual).unwrap();
        assert!(rec.verified, "{:?}", rec.verify_error);
    }
}

// ---------------------------------------------------------------------------
// Recuperação (docs/backup-recovery.md): com o cofre fechado ou quebrado, a lista vem da
// pasta (a tabela `backup_record` fica dentro do cofre).

/// Subpasta dos backups trazidos de fora (fora da rotação: nunca apagados por ela).
pub const IMPORTED_DIR: &str = "importados";

/// Arquivo de backup achado na pasta do cofre.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackupFile {
    pub path: String,
    pub file_name: String,
    /// Data do backup (do nome do arquivo; nos importados sem o nome de sempre, a do arquivo).
    pub created_at: Option<String>,
    pub kind: Option<BackupKind>,
    #[ts(type = "number")]
    pub size_bytes: i64,
    /// Trazido de fora (`importados/`).
    pub imported: bool,
}

fn file_entry(path: &Path, imported: bool) -> Option<BackupFile> {
    let name = path.file_name()?.to_str()?.to_string();
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || name.ends_with(".partial") || name.starts_with('.') {
        return None;
    }
    let parsed = parse_file_name(&name);
    if parsed.is_none() && !imported {
        return None;
    }
    let created_at = match parsed {
        Some((ts, _)) => Some(ts),
        None => meta.modified().ok().map(DateTime::<Utc>::from),
    }
    .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
    Some(BackupFile {
        path: path.to_string_lossy().into_owned(),
        file_name: name,
        created_at,
        kind: parsed.map(|(_, k)| k),
        size_bytes: meta.len() as i64,
        imported,
    })
}

/// Backups da pasta do cofre e dos importados, do mais novo para o mais antigo.
pub fn scan(dir: &Path) -> Vec<BackupFile> {
    let mut out = vec![];
    for (d, imported) in [(dir.to_path_buf(), false), (dir.join(IMPORTED_DIR), true)] {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        out.extend(
            entries
                .flatten()
                .filter_map(|e| file_entry(&e.path(), imported)),
        );
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out
}

/// Confere que `path` é um arquivo da pasta de backups do cofre (ou de `importados/`).
pub fn ensure_in_dir(dir: &Path, path: &Path) -> Result<PathBuf> {
    let not_here = || Error::invalid(t("nucleo-geral-erro-backup-fora-da-pasta"));
    let dir = dir.canonicalize().map_err(|_| not_here())?;
    let file = path.canonicalize().map_err(|_| not_here())?;
    match file.parent() {
        Some(p) if p == dir || p == dir.join(IMPORTED_DIR) => Ok(file),
        _ => Err(not_here()),
    }
}

/// Copia um backup da pasta para fora (pen drive, HD, nuvem) e confere a cópia. O arquivo
/// continua cifrado como o backup.
pub fn export_file(dir: &Path, src: &Path, dest: &Path) -> Result<()> {
    let src = ensure_in_dir(dir, src)?;
    if dest.canonicalize().ok().as_deref() == Some(src.as_path()) {
        return Err(Error::invalid(t("nucleo-geral-erro-backup-destino-igual")));
    }
    let partial = dest.with_extension("partial");
    let _ = std::fs::remove_file(&partial);
    std::fs::copy(&src, &partial)?;
    if sha256_file(&partial)? != sha256_file(&src)? {
        let _ = std::fs::remove_file(&partial);
        return Err(Error::Other(t("nucleo-geral-erro-backup-copia-diferente")));
    }
    std::fs::rename(&partial, dest)?;
    Ok(())
}

/// Traz um backup de fora para `importados/`. Sem a senha não dá para abrir: confere só que
/// parece um banco cifrado (tamanho em páginas, sem o cabeçalho de um SQLite sem cifra).
pub fn import_file(dir: &Path, src: &Path) -> Result<BackupFile> {
    use std::io::Read;
    let bad = || Error::invalid(t("nucleo-geral-erro-backup-nao-e-backup"));
    let meta = std::fs::metadata(src).map_err(|_| bad())?;
    if !meta.is_file() || meta.len() < 4096 || meta.len() % 512 != 0 {
        return Err(bad());
    }
    let mut head = [0u8; 16];
    std::fs::File::open(src)?.read_exact(&mut head)?;
    if &head == b"SQLite format 3\0" {
        return Err(bad());
    }
    let target_dir = dir.join(IMPORTED_DIR);
    crate::paths::create_private_dir(&target_dir)?;
    let name = src
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| file_name(Utc::now(), BackupKind::Manual));
    let mut target = target_dir.join(&name);
    let mut n = 2;
    while target.exists() {
        target = target_dir.join(format!("{n}-{name}"));
        n += 1;
    }
    let partial = target.with_extension("partial");
    std::fs::copy(src, &partial)?;
    std::fs::rename(&partial, &target)?;
    crate::paths::set_mode(&target, 0o600)?;
    file_entry(&target, true).ok_or_else(bad)
}

/// Põe o cofre quebrado de lado (nunca apaga): `treasury.db.danificado-AAAAMMDD-HHMMSS`, com
/// o `-wal` e o `-shm` junto. Devolve o caminho novo.
pub fn move_aside(db_path: &Path) -> Result<PathBuf> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let name = db_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("treasury.db");
    let aside = db_path.with_file_name(format!("{name}.danificado-{stamp}"));
    std::fs::rename(db_path, &aside)?;
    for ext in ["-wal", "-shm"] {
        let side = db_path.with_file_name(format!("{name}{ext}"));
        if side.exists() {
            let _ = std::fs::rename(
                &side,
                aside.with_file_name(format!("{name}.danificado-{stamp}{ext}")),
            );
        }
    }
    Ok(aside)
}

/// A senha abre o backup mais novo da pasta (não importado)? Usado quando ela não abre o cofre.
/// Fica de fora quando o mais novo é o de antes da troca de senha.
pub fn newest_opens_with(dir: &Path, password: &str) -> bool {
    let Some(newest) = scan(dir).into_iter().find(|f| !f.imported) else {
        return false;
    };
    // O backup de antes de trocar a senha abre com a senha antiga: quem digita a antiga logo
    // depois da troca não tem um cofre danificado.
    if newest.kind == Some(BackupKind::PreRekey) {
        return false;
    }
    let path = Path::new(&newest.path);
    vault::read_salt(path)
        .and_then(|salt| derive_any(password, salt, path))
        .is_ok()
}
