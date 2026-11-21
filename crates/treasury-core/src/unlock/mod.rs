//! Métodos extras de recuperação do cofre (docs/unlock-options.md): recuperação dividida
//! (Shamir 2 de 3) e chave física FIDO2. Os dois precisam continuar valendo depois que a
//! senha mestra muda, mas a chave do banco muda junto com a senha (`PRAGMA rekey`). Por
//! isso existe uma **chave de recuperação estável** `R`:
//!
//! - `R` (32 bytes aleatórios) fica **dentro** do banco, como segredo cifrado
//!   (`vault:recovery_root`), e é recifrada automaticamente na troca de senha.
//! - Ao lado do banco, o arquivo `treasury.db.keys` guarda a chave atual do banco cifrada
//!   com `R` (XChaCha20-Poly1305) e os slots FIDO2 (cada um com `R` cifrada pela saída
//!   hmac-secret da chave física). Esse arquivo é reescrito a cada desbloqueio.
//! - As partes de Shamir são `R` dividida em 3; duas reconstroem `R`, que abre a chave do
//!   banco pelo arquivo `.keys`. Uma chave física reconstrói `R` pelo slot dela.
//!
//! Sem o arquivo `.keys`, nem as partes nem a chave física abrem o cofre (o kit de
//! recuperação continua sendo a chave do banco em si). O arquivo não é secreto sozinho:
//! sem `R` ele não revela nada.

pub mod pin;
pub mod shamir;

use std::path::{Path, PathBuf};

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use ts_rs::TS;
use zeroize::{Zeroize, Zeroizing};

use crate::ai::secret;
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::vault::{DbKey, KEY_LEN, SALT_LEN};

const ROOT_ID: &str = "vault:recovery_root";
const ROOT_LEN: usize = 32;
const NONCE_LEN: usize = 24;
const SHARES_AT: &str = "vault.recovery_shares_at";
/// Quando o kit de recuperação foi mostrado (o código é a chave do banco: muda com a senha).
pub const KIT_AT: &str = "vault.recovery_kit_at";
/// O kit gerado antes da última troca de senha não abre mais o cofre.
pub const KIT_OUTDATED: &str = "vault.recovery_kit_outdated";
/// Quando a primeira chave física foi cadastrada.
const FIDO2_AT: &str = "vault.recovery_fido2_at";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Wrapped {
    nonce: String,
    ct: String,
}

/// Slot de chave física (público: nada aqui abre o cofre sem a chave física).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Fido2Slot {
    pub id: String,
    pub label: String,
    pub rp_id: String,
    /// Credencial (base64url) e salt do hmac-secret (hex), para pedir a asserção.
    pub credential_id: String,
    pub salt: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(skip)]
    wrap: Option<Wrapped>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct KeyFile {
    version: u8,
    db_salt: String,
    /// Chave atual do banco (versão do KDF + 32 bytes) cifrada com `R`.
    root_wrap: Option<Wrapped>,
    #[serde(default)]
    fido2: Vec<Fido2Slot>,
}

pub fn keys_path(db_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.keys", db_path.display()))
}

fn load_file(db_path: &Path) -> Result<Option<KeyFile>> {
    let p = keys_path(db_path);
    if !p.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&p)?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|_| Error::Crypto(t("nucleo-geral-erro-chaves-ilegivel")))
}

fn save_file(db_path: &Path, f: &KeyFile) -> Result<()> {
    let p = keys_path(db_path);
    let tmp = p.with_extension("keys.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(f)?)?;
    crate::paths::set_mode(&tmp, 0o600)?;
    std::fs::rename(&tmp, &p)?;
    Ok(())
}

fn cipher(kek: &[u8], info: &[u8]) -> XChaCha20Poly1305 {
    let hk = Hkdf::<Sha256>::new(None, kek);
    let mut k = Zeroizing::new([0u8; 32]);
    hk.expand(info, k.as_mut()).expect("32 bytes no HKDF");
    XChaCha20Poly1305::new_from_slice(k.as_ref()).expect("chave de 32 bytes")
}

fn wrap(kek: &[u8], info: &[u8], aad: &[u8], plain: &[u8]) -> Result<Wrapped> {
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce).map_err(|e| Error::Crypto(e.to_string()))?;
    let ct = cipher(kek, info)
        .encrypt(&XNonce::from(nonce), Payload { msg: plain, aad })
        .map_err(|_| Error::Crypto(t("nucleo-geral-erro-cifrar")))?;
    Ok(Wrapped {
        nonce: hex::encode(nonce),
        ct: hex::encode(ct),
    })
}

fn unwrap(kek: &[u8], info: &[u8], aad: &[u8], w: &Wrapped) -> Option<Zeroizing<Vec<u8>>> {
    let nonce: [u8; NONCE_LEN] = hex::decode(&w.nonce).ok()?.try_into().ok()?;
    let ct = hex::decode(&w.ct).ok()?;
    cipher(kek, info)
        .decrypt(&XNonce::from(nonce), Payload { msg: &ct, aad })
        .ok()
        .map(Zeroizing::new)
}

const ROOT_INFO: &[u8] = b"treasury:unlock:root-wrap:v1";
const FIDO_INFO: &[u8] = b"treasury:unlock:fido2-hmac-secret:v1";

fn root(conn: &Connection, key: &DbKey) -> Result<Option<Zeroizing<Vec<u8>>>> {
    let Some(h) = secret::get(conn, key, ROOT_ID)? else {
        return Ok(None);
    };
    let bytes = hex::decode(h.as_str())
        .map_err(|_| Error::Crypto(t("nucleo-geral-erro-recuperacao-ilegivel")))?;
    Ok(Some(Zeroizing::new(bytes)))
}

fn new_root(conn: &Connection, key: &DbKey) -> Result<Zeroizing<Vec<u8>>> {
    let mut r = Zeroizing::new(vec![0u8; ROOT_LEN]);
    getrandom::fill(&mut r).map_err(|e| Error::Crypto(e.to_string()))?;
    let hexed = Zeroizing::new(hex::encode(r.as_slice()));
    secret::set_document(conn, key, ROOT_ID, &hexed, "chave de recuperação")?;
    Ok(r)
}

fn key_plain(key: &DbKey) -> Zeroizing<Vec<u8>> {
    let mut v = Zeroizing::new(Vec::with_capacity(KEY_LEN + 1));
    v.push(key.kdf_version());
    v.extend_from_slice(key.key_bytes());
    v
}

/// Reescreve no `.keys` a chave atual do banco cifrada com `R` (se houver `R`). Chamado a
/// cada desbloqueio e depois de trocar a senha ou restaurar um backup.
pub fn refresh(conn: &Connection, key: &DbKey, db_path: &Path) -> Result<()> {
    let Some(r) = root(conn, key)? else {
        return Ok(());
    };
    let mut f = load_file(db_path)?.unwrap_or_default();
    f.version = 1;
    f.db_salt = hex::encode(key.salt());
    f.root_wrap = Some(wrap(&r, ROOT_INFO, &key.salt(), &key_plain(key))?);
    save_file(db_path, &f)
}

/// `R` deste cofre, criando-a (e o `.keys`) se ainda não existir.
pub(crate) fn ensure_root(
    conn: &Connection,
    key: &DbKey,
    db_path: &Path,
) -> Result<Zeroizing<Vec<u8>>> {
    let r = match root(conn, key)? {
        Some(r) => r,
        None => new_root(conn, key)?,
    };
    refresh(conn, key, db_path)?;
    Ok(r)
}

/// Chave do banco a partir de `R` e do arquivo `.keys`.
pub(crate) fn key_from_root(db_path: &Path, r: &[u8]) -> Result<DbKey> {
    let f = load_file(db_path)?
        .ok_or_else(|| Error::invalid(t("nucleo-geral-erro-sem-arquivo-chaves")))?;
    let salt_v = hex::decode(&f.db_salt)
        .map_err(|_| Error::Crypto(t("nucleo-geral-erro-chaves-ilegivel")))?;
    let salt: [u8; SALT_LEN] = salt_v
        .try_into()
        .map_err(|_| Error::Crypto(t("nucleo-geral-erro-chaves-ilegivel")))?;
    let w = f
        .root_wrap
        .ok_or_else(|| Error::invalid(t("nucleo-geral-erro-sem-recuperacao-dividida")))?;
    let plain = unwrap(r, ROOT_INFO, &salt, &w).ok_or(Error::WrongRecoveryCode)?;
    if plain.len() != KEY_LEN + 1 {
        return Err(Error::Crypto(t("nucleo-geral-erro-chave-cofre-ilegivel")));
    }
    let mut k = [0u8; KEY_LEN];
    k.copy_from_slice(&plain[1..]);
    let out = DbKey::from_parts(k, salt, plain[0]);
    k.zeroize();
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecoveryShares {
    /// Três partes; quaisquer duas recuperam o cofre.
    pub shares: Vec<String>,
    pub created_at: String,
    /// Chaves físicas cadastradas que deixaram de valer (rotação da chave de recuperação).
    #[ts(type = "number")]
    pub fido2_removed: u32,
}

/// Gera as 3 partes. Com `rotate`, troca `R`: partes antigas e chaves físicas deixam de
/// valer (as chaves físicas precisam ser cadastradas de novo).
pub fn create_shares(
    conn: &Connection,
    key: &DbKey,
    db_path: &Path,
    rotate: bool,
) -> Result<RecoveryShares> {
    let mut removed = 0;
    let r = match (root(conn, key)?, rotate) {
        (Some(r), false) => r,
        _ => {
            let r = new_root(conn, key)?;
            if let Some(mut f) = load_file(db_path)? {
                removed = f.fido2.len() as u32;
                f.fido2.clear();
                save_file(db_path, &f)?;
            }
            r
        }
    };
    refresh(conn, key, db_path)?;
    let now = crate::time::now_utc();
    crate::settings::set(conn, SHARES_AT, &now)?;
    Ok(RecoveryShares {
        shares: shamir::split(&r, 2, 3)?,
        created_at: now,
        fido2_removed: removed,
    })
}

/// Chave do banco a partir de duas (ou mais) partes.
pub fn key_from_shares<S: AsRef<str>>(db_path: &Path, shares: &[S]) -> Result<DbKey> {
    let r = shamir::combine(shares)?;
    key_from_root(db_path, &r)
}

/// Cadastra uma chave física. `hmac_secret` é a saída do hmac-secret (32 bytes) para o
/// `salt` deste slot; nunca é guardada.
#[allow(clippy::too_many_arguments)]
pub fn add_fido2(
    conn: &Connection,
    key: &DbKey,
    db_path: &Path,
    label: &str,
    rp_id: &str,
    credential_id: &str,
    salt_hex: &str,
    hmac_secret: &[u8],
) -> Result<Fido2Slot> {
    if hmac_secret.len() < 32 {
        return Err(Error::invalid(t("nucleo-geral-erro-resposta-chave-fisica")));
    }
    let r = match root(conn, key)? {
        Some(r) => r,
        None => new_root(conn, key)?,
    };
    refresh(conn, key, db_path)?;
    let id = crate::ids::new_id();
    let mut f = load_file(db_path)?.unwrap_or_default();
    let slot = Fido2Slot {
        id: id.clone(),
        label: label.trim().chars().take(60).collect(),
        rp_id: rp_id.into(),
        credential_id: credential_id.into(),
        salt: salt_hex.into(),
        created_at: crate::time::now_utc(),
        wrap: Some(wrap(hmac_secret, FIDO_INFO, id.as_bytes(), &r)?),
    };
    f.fido2.push(slot.clone());
    save_file(db_path, &f)?;
    crate::settings::set(conn, FIDO2_AT, &slot.created_at)?;
    Ok(Fido2Slot { wrap: None, ..slot })
}

pub fn fido2_slots(db_path: &Path) -> Result<Vec<Fido2Slot>> {
    Ok(load_file(db_path)?
        .map(|f| f.fido2)
        .unwrap_or_default()
        .into_iter()
        .map(|s| Fido2Slot { wrap: None, ..s })
        .collect())
}

pub fn remove_fido2(db_path: &Path, id: &str) -> Result<()> {
    let mut f =
        load_file(db_path)?.ok_or_else(|| Error::not_found(t("nucleo-geral-chave-fisica")))?;
    let before = f.fido2.len();
    f.fido2.retain(|s| s.id != id);
    if f.fido2.len() == before {
        return Err(Error::not_found(t("nucleo-geral-chave-fisica")));
    }
    save_file(db_path, &f)
}

/// Chave do banco a partir da resposta de uma chave física cadastrada.
pub fn key_from_fido2(db_path: &Path, slot_id: &str, hmac_secret: &[u8]) -> Result<DbKey> {
    let f =
        load_file(db_path)?.ok_or_else(|| Error::not_found(t("nucleo-geral-arquivo-chaves")))?;
    let slot = f
        .fido2
        .iter()
        .find(|s| s.id == slot_id)
        .ok_or_else(|| Error::not_found(t("nucleo-geral-chave-fisica")))?;
    let w = slot.wrap.as_ref().ok_or(Error::WrongRecoveryCode)?;
    let r =
        unwrap(hmac_secret, FIDO_INFO, slot.id.as_bytes(), w).ok_or(Error::WrongRecoveryCode)?;
    key_from_root(db_path, &r)
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecoveryStatus {
    /// Quando as partes de recuperação foram geradas (`null` = nunca).
    pub shares_created_at: Option<String>,
    pub fido2: Vec<Fido2Slot>,
    /// Quando o kit de recuperação válido foi mostrado (`null` = nunca, ou a senha mudou depois).
    #[serde(default)]
    pub kit_created_at: Option<String>,
    /// Havia um kit, mas a senha mudou depois dele: o código antigo não abre mais o cofre.
    #[serde(default)]
    pub kit_outdated: bool,
}

pub fn status(conn: &Connection, db_path: &Path) -> Result<RecoveryStatus> {
    Ok(RecoveryStatus {
        shares_created_at: crate::settings::get::<String>(conn, SHARES_AT)?,
        fido2: fido2_slots(db_path)?,
        kit_created_at: crate::settings::get::<String>(conn, KIT_AT)?,
        kit_outdated: crate::settings::get::<bool>(conn, KIT_OUTDATED)?.unwrap_or(false),
    })
}

/// O kit foi mostrado agora.
pub fn kit_shown(conn: &Connection) -> Result<String> {
    let now = crate::time::now_utc();
    crate::settings::set(conn, KIT_AT, &now)?;
    crate::settings::delete(conn, KIT_OUTDATED)?;
    Ok(now)
}

/// A senha mudou: o kit anterior (se havia) deixa de valer. As partes e as chaves físicas
/// continuam valendo (`refresh` regrava o `.keys`).
pub fn password_changed(conn: &Connection) -> Result<()> {
    if crate::settings::get::<String>(conn, KIT_AT)?.is_some() {
        crate::settings::delete(conn, KIT_AT)?;
        crate::settings::set(conn, KIT_OUTDATED, &true)?;
    }
    Ok(())
}

/// Alguma forma de abrir o cofre sem a senha foi preparada (kit, partes ou chave física).
/// Sem o caminho do arquivo: a chave física conta pelo registro do cadastro.
pub fn has_recovery(conn: &Connection) -> Result<bool> {
    Ok(crate::settings::get::<String>(conn, KIT_AT)?.is_some()
        || crate::settings::get::<String>(conn, SHARES_AT)?.is_some()
        || crate::settings::get::<String>(conn, FIDO2_AT)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn shares_survive_a_password_change() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("treasury.db");
        let mut db = Db::create(&p, "senha-antiga", KDF_TEST).unwrap();
        let rs = create_shares(db.conn(), db.key(), &p, false).unwrap();
        assert_eq!(rs.shares.len(), 3);
        // A senha muda (a chave do banco muda); o desbloqueio reescreve o .keys.
        db.change_password("senha-nova", KDF_TEST).unwrap();
        refresh(db.conn(), db.key(), &p).unwrap();
        drop(db);
        let key = key_from_shares(&p, &[rs.shares[2].clone(), rs.shares[0].clone()]).unwrap();
        let db = Db::open_with_key(&p, key, false).unwrap();
        assert!(db.schema_version().unwrap() > 0);
        // Partes de outro conjunto não abrem.
        let other = shamir::split(&[7u8; 32], 2, 3).unwrap();
        assert!(key_from_shares(&p, &other[..2]).is_err());
    }

    #[test]
    fn rotating_invalidates_old_shares_and_fido2() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("treasury.db");
        let db = Db::create(&p, "s", KDF_TEST).unwrap();
        let old = create_shares(db.conn(), db.key(), &p, false).unwrap();
        let hmac = [5u8; 32];
        let slot = add_fido2(
            db.conn(),
            db.key(),
            &p,
            "YubiKey",
            "treasury.local",
            "cred",
            "00",
            &hmac,
        )
        .unwrap();
        assert!(key_from_fido2(&p, &slot.id, &hmac).is_ok());
        assert!(
            key_from_fido2(&p, &slot.id, &[6u8; 32]).is_err(),
            "resposta errada"
        );
        let new = create_shares(db.conn(), db.key(), &p, true).unwrap();
        assert_eq!(new.fido2_removed, 1);
        assert!(key_from_shares(&p, &old.shares[..2]).is_err());
        assert!(key_from_shares(&p, &new.shares[1..]).is_ok());
        assert!(fido2_slots(&p).unwrap().is_empty());
    }

    #[test]
    fn keys_file_reveals_nothing_without_r() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("treasury.db");
        let db = Db::create(&p, "s", KDF_TEST).unwrap();
        create_shares(db.conn(), db.key(), &p, false).unwrap();
        let text = std::fs::read_to_string(keys_path(&p)).unwrap();
        assert!(!text.contains(&hex::encode(db.key().key_bytes())));
        assert!(fido2_slots(&p).unwrap().is_empty());
        let st = status(db.conn(), &p).unwrap();
        assert!(st.shares_created_at.is_some());
    }
}
