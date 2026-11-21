//! Segredos (API keys) cifrados em nível de campo, dentro do banco já cifrado
//! (docs/security.md §Criptografia): XChaCha20-Poly1305 com uma subchave derivada da chave
//! do banco por HKDF-SHA256, nonce aleatório de 24 bytes por valor e o id do segredo como
//! dado associado. O valor nunca sai daqui para logs ou respostas da API: a interface só
//! vê se está cadastrado e os 4 últimos caracteres.
//!
//! A troca de senha mestra (`Db::rekey`) e a restauração de backup recifram os segredos
//! com a chave nova ([`rewrap`]). Um segredo que não decifra com a chave atual (backup com
//! senha própria restaurado depois de trocar a senha mestra) aparece como
//! `needs_reentry` e precisa ser cadastrado de novo.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use ts_rs::TS;
use zeroize::Zeroizing;

use crate::error::{Error, Result};
use crate::time::now_utc;
use crate::vault::DbKey;

const HKDF_INFO: &[u8] = b"treasury:secret:xchacha20poly1305:v1";
const NONCE_LEN: usize = 24;
/// Limite generoso para uma API key.
const MAX_LEN: usize = 4096;

/// O que a interface vê de um segredo.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SecretStatus {
    /// Há um valor cadastrado.
    pub set: bool,
    /// Últimos 4 caracteres (ex.: `…a1b2`), para conferir qual chave está cadastrada.
    pub hint: Option<String>,
    /// O valor não decifra com a chave atual do cofre: cadastre de novo.
    pub needs_reentry: bool,
    pub updated_at: Option<String>,
}

fn subkey(db_key: &DbKey) -> Zeroizing<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(Some(&db_key.salt()), db_key.key_bytes());
    let mut okm = Zeroizing::new([0u8; 32]);
    hk.expand(HKDF_INFO, okm.as_mut())
        .expect("32 bytes cabem no HKDF-SHA256");
    okm
}

fn cipher(db_key: &DbKey) -> XChaCha20Poly1305 {
    let k = subkey(db_key);
    XChaCha20Poly1305::new_from_slice(k.as_ref()).expect("chave de 32 bytes")
}

fn encrypt(db_key: &DbKey, id: &str, value: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce).map_err(|e| Error::Crypto(e.to_string()))?;
    let ct = cipher(db_key)
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: value.as_bytes(),
                aad: id.as_bytes(),
            },
        )
        .map_err(|_| Error::Crypto("falha ao cifrar o segredo".into()))?;
    Ok((nonce.to_vec(), ct))
}

fn decrypt(db_key: &DbKey, id: &str, nonce: &[u8], ct: &[u8]) -> Option<Zeroizing<String>> {
    let nonce: [u8; NONCE_LEN] = nonce.try_into().ok()?;
    let plain = Zeroizing::new(
        cipher(db_key)
            .decrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: ct,
                    aad: id.as_bytes(),
                },
            )
            .ok()?,
    );
    String::from_utf8(plain.to_vec()).ok().map(Zeroizing::new)
}

fn hint_of(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let tail: String = chars[chars.len().saturating_sub(4)..].iter().collect();
    format!("…{tail}")
}

/// Grava (ou troca) um segredo.
pub fn set(conn: &Connection, db_key: &DbKey, id: &str, value: &str) -> Result<SecretStatus> {
    let value = Zeroizing::new(value.trim().to_string());
    if value.is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ia-chave-obrigatoria",
        )));
    }
    if value.len() > MAX_LEN || value.chars().any(char::is_control) {
        return Err(Error::invalid(crate::i18n::t("nucleo-ia-chave-invalida")));
    }
    let (nonce, ct) = encrypt(db_key, id, &value)?;
    let now = now_utc();
    conn.execute(
        "INSERT INTO secret (id, nonce, ciphertext, hint, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT (id) DO UPDATE SET nonce = excluded.nonce, ciphertext = excluded.ciphertext,
           hint = excluded.hint, updated_at = excluded.updated_at",
        params![id, nonce, ct, hint_of(&value), now],
    )?;
    status(conn, db_key, id)
}

/// Limite para documentos guardados como segredo (ex.: cookies de um site).
const MAX_DOCUMENT_LEN: usize = 1024 * 1024;

/// Guarda um documento (JSON) cifrado como segredo, com uma dica própria (ex.: "12
/// cookies"). Entra na recifragem da troca de senha e da restauração como as chaves.
pub fn set_document(
    conn: &Connection,
    db_key: &DbKey,
    id: &str,
    value: &str,
    hint: &str,
) -> Result<()> {
    if value.len() > MAX_DOCUMENT_LEN {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ia-chave-documento-grande",
        )));
    }
    let (nonce, ct) = encrypt(db_key, id, value)?;
    let now = now_utc();
    conn.execute(
        "INSERT INTO secret (id, nonce, ciphertext, hint, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT (id) DO UPDATE SET nonce = excluded.nonce, ciphertext = excluded.ciphertext,
           hint = excluded.hint, updated_at = excluded.updated_at",
        params![id, nonce, ct, hint, now],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM secret WHERE id = ?1", [id])?;
    Ok(())
}

/// Valor decifrado, para uso imediato (cabeçalho de uma chamada). Nunca registre.
pub fn get(conn: &Connection, db_key: &DbKey, id: &str) -> Result<Option<Zeroizing<String>>> {
    let row: Option<(Vec<u8>, Vec<u8>)> = conn
        .query_row(
            "SELECT nonce, ciphertext FROM secret WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match row {
        None => Ok(None),
        Some((n, c)) => decrypt(db_key, id, &n, &c)
            .map(Some)
            .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-ia-chave-nao-abre"))),
    }
}

pub fn status(conn: &Connection, db_key: &DbKey, id: &str) -> Result<SecretStatus> {
    let row: Option<(Vec<u8>, Vec<u8>, String, String)> = conn
        .query_row(
            "SELECT nonce, ciphertext, hint, updated_at FROM secret WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    Ok(match row {
        None => SecretStatus::default(),
        Some((n, c, hint, updated_at)) => SecretStatus {
            set: true,
            needs_reentry: decrypt(db_key, id, &n, &c).is_none(),
            hint: Some(hint),
            updated_at: Some(updated_at),
        },
    })
}

/// Recifra todos os segredos com `new_key`, tentando decifrar com cada chave candidata.
/// Devolve quantos foram recifrados. Sem a tabela (esquema antigo), não faz nada.
pub fn rewrap(conn: &Connection, candidates: &[&DbKey], new_key: &DbKey) -> Result<usize> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'secret')",
        [],
        |r| r.get(0),
    )?;
    if !exists {
        return Ok(0);
    }
    let mut stmt = conn.prepare("SELECT id, nonce, ciphertext FROM secret")?;
    let rows: Vec<(String, Vec<u8>, Vec<u8>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut n = 0;
    for (id, nonce, ct) in rows {
        let Some(plain) = candidates.iter().find_map(|k| decrypt(k, &id, &nonce, &ct)) else {
            continue;
        };
        let (nonce, ct) = encrypt(new_key, &id, &plain)?;
        conn.execute(
            "UPDATE secret SET nonce = ?2, ciphertext = ?3 WHERE id = ?1",
            params![id, nonce, ct],
        )?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn roundtrip_binding_and_rekey() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.db");
        let mut db = Db::create(&path, "senha", KDF_TEST).unwrap();
        let key = "sk-ant-api03-SEGREDO-abcd1234";
        let st = set(db.conn(), db.key(), "ai_engine:x", key).unwrap();
        assert_eq!(st.hint.as_deref(), Some("…1234"));
        assert!(st.set && !st.needs_reentry);
        assert_eq!(
            get(db.conn(), db.key(), "ai_engine:x")
                .unwrap()
                .unwrap()
                .as_str(),
            key
        );
        // A cifra vale só para a linha dela (AAD).
        let (n, c): (Vec<u8>, Vec<u8>) = db
            .conn()
            .query_row(
                "SELECT nonce, ciphertext FROM secret WHERE id = 'ai_engine:x'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(decrypt(db.key(), "ai_engine:y", &n, &c).is_none());
        assert!(!c.windows(key.len()).any(|w| w == key.as_bytes()));
        // Mesmo valor, nonce novo a cada gravação.
        set(db.conn(), db.key(), "ai_engine:x", key).unwrap();
        let n2: Vec<u8> = db
            .conn()
            .query_row("SELECT nonce FROM secret", [], |r| r.get(0))
            .unwrap();
        assert_ne!(n, n2);
        // Troca de senha mestra recifra os segredos.
        db.change_password("outra senha", KDF_TEST).unwrap();
        assert_eq!(
            get(db.conn(), db.key(), "ai_engine:x")
                .unwrap()
                .unwrap()
                .as_str(),
            key
        );
        drop(db);
        let db = Db::open(&path, "outra senha").unwrap();
        assert!(
            !status(db.conn(), db.key(), "ai_engine:x")
                .unwrap()
                .needs_reentry
        );
        // Chave errada: aparece como "cadastre de novo", sem erro na listagem.
        let other = DbKey::derive("x", db.key().salt(), KDF_TEST).unwrap();
        assert!(
            status(db.conn(), &other, "ai_engine:x")
                .unwrap()
                .needs_reentry
        );
        assert!(get(db.conn(), &other, "ai_engine:x").is_err());
    }
}
