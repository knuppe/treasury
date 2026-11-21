//! PIN para o dia a dia: só vale **neste computador**. A chave de recuperação estável `R`
//! fica no Keychain/Secret Service cifrada com Argon2id(PIN) — nunca no arquivo do banco —
//! e abre a chave do banco pelo arquivo `.keys`. Por usar `R` (e não a chave do banco), o
//! PIN continua valendo depois de trocar a senha mestra.
//!
//! Cinco PINs errados apagam o item: daí em diante só a senha mestra (ou a recuperação)
//! abre o cofre. Quem copiar o item do Keychain e o arquivo `.keys` ainda precisa
//! adivinhar o PIN contra o Argon2id (64 MiB por tentativa).

use std::path::Path;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::keychain;
use crate::vault::{DbKey, KdfParams, SALT_LEN};

pub const MAX_FAILURES: u8 = 5;

#[derive(Serialize, Deserialize)]
struct PinItem {
    v: u8,
    m_kib: u32,
    t_cost: u32,
    p_cost: u32,
    salt: String,
    nonce: String,
    ct: String,
    failures: u8,
}

fn account(db_path: &Path) -> String {
    format!("{}-pin", keychain::account_for(db_path))
}

pub fn validate(pin: &str) -> Result<()> {
    if !(6..=12).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(Error::invalid(t("nucleo-geral-erro-pin-tamanho")));
    }
    let digits: Vec<u8> = pin.bytes().collect();
    let all_same = digits.windows(2).all(|w| w[0] == w[1]);
    let sequence =
        digits.windows(2).all(|w| w[1] == w[0] + 1) || digits.windows(2).all(|w| w[0] == w[1] + 1);
    if all_same || sequence {
        return Err(Error::invalid(t("nucleo-geral-erro-pin-facil")));
    }
    Ok(())
}

fn kek(pin: &str, salt: [u8; SALT_LEN], p: KdfParams) -> Result<DbKey> {
    DbKey::derive(pin, salt, p)
}

fn cipher(k: &DbKey) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new_from_slice(k.key_bytes()).expect("chave de 32 bytes")
}

pub fn is_enabled(db_path: &Path) -> bool {
    let s = keychain::store();
    s.is_available() && s.exists(&account(db_path)).unwrap_or(false)
}

/// Liga (ou troca) o PIN: guarda `R` cifrada com o PIN no Keychain.
pub fn enable(db_path: &Path, root: &[u8], pin: &str, params: KdfParams) -> Result<()> {
    validate(pin)?;
    let s = keychain::store();
    if !s.is_available() {
        return Err(Error::invalid(t("nucleo-geral-erro-pin-sem-keychain")));
    }
    let salt = DbKey::random_salt()?;
    let k = kek(pin, salt, params)?;
    let mut nonce = [0u8; 24];
    getrandom::fill(&mut nonce).map_err(|e| Error::Crypto(e.to_string()))?;
    let ct = cipher(&k)
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: root,
                aad: b"treasury:pin:v1",
            },
        )
        .map_err(|_| Error::Crypto(t("nucleo-geral-erro-cifrar")))?;
    let item = PinItem {
        v: 1,
        m_kib: params.m_kib,
        t_cost: params.t_cost,
        p_cost: params.p_cost,
        salt: hex::encode(salt),
        nonce: hex::encode(nonce),
        ct: hex::encode(ct),
        failures: 0,
    };
    s.set(&account(db_path), &serde_json::to_string(&item)?)
}

pub fn disable(db_path: &Path) -> Result<()> {
    let s = keychain::store();
    if !s.is_available() {
        return Ok(());
    }
    s.delete(&account(db_path))
}

/// Chave do banco a partir do PIN. Erros contam tentativa; na quinta, o PIN é apagado.
pub fn unlock(db_path: &Path, pin: &str) -> Result<DbKey> {
    let s = keychain::store();
    let acc = account(db_path);
    let raw = s
        .get(&acc)?
        .ok_or_else(|| Error::invalid(t("nucleo-geral-erro-pin-desligado")))?;
    let mut item: PinItem = serde_json::from_str(&raw)
        .map_err(|_| Error::Crypto(t("nucleo-geral-erro-pin-ilegivel-keychain")))?;
    let salt: [u8; SALT_LEN] = hex::decode(&item.salt)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| Error::Crypto(t("nucleo-geral-erro-pin-ilegivel")))?;
    let params = KdfParams {
        version: 1,
        m_kib: item.m_kib,
        t_cost: item.t_cost,
        p_cost: item.p_cost,
    };
    let k = kek(pin, salt, params)?;
    let nonce: [u8; 24] = hex::decode(&item.nonce)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| Error::Crypto(t("nucleo-geral-erro-pin-ilegivel")))?;
    let ct =
        hex::decode(&item.ct).map_err(|_| Error::Crypto(t("nucleo-geral-erro-pin-ilegivel")))?;
    let plain = cipher(&k).decrypt(
        &XNonce::from(nonce),
        Payload {
            msg: &ct,
            aad: b"treasury:pin:v1",
        },
    );
    match plain {
        Ok(root) => {
            let root = zeroize::Zeroizing::new(root);
            if item.failures > 0 {
                item.failures = 0;
                s.set(&acc, &serde_json::to_string(&item)?)?;
            }
            super::key_from_root(db_path, &root)
        }
        Err(_) => {
            item.failures += 1;
            if item.failures >= MAX_FAILURES {
                s.delete(&acc)?;
                return Err(Error::WrongPin(t_args(
                    "nucleo-geral-erro-pin-bloqueado",
                    &args(&[("tentativas", MAX_FAILURES.into())]),
                )));
            }
            s.set(&acc, &serde_json::to_string(&item)?)?;
            let left = MAX_FAILURES - item.failures;
            Err(Error::WrongPin(t_args(
                "nucleo-geral-erro-pin-incorreto",
                &args(&[("restam", left.into())]),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn pin_rules() {
        assert!(validate("123456").is_err(), "sequência");
        assert!(validate("000000").is_err(), "repetido");
        assert!(validate("12a456").is_err());
        assert!(validate("4071").is_err(), "curto");
        assert!(validate("407193").is_ok());
    }

    #[test]
    fn pin_unlocks_survives_password_change_and_locks_after_five_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("treasury.db");
        let mut db = Db::create(&p, "senha", KDF_TEST).unwrap();
        let r = super::super::ensure_root(db.conn(), db.key(), &p).unwrap();
        enable(&p, &r, "407193", KDF_TEST).unwrap();
        assert!(is_enabled(&p));
        db.change_password("outra", KDF_TEST).unwrap();
        super::super::refresh(db.conn(), db.key(), &p).unwrap();
        drop(db);
        let k = unlock(&p, "407193").unwrap();
        Db::open_with_key(&p, k, false).unwrap();
        for i in 1..MAX_FAILURES {
            let e = unlock(&p, "999999").unwrap_err();
            assert!(
                e.to_string()
                    .contains(&format!("Restam {}", MAX_FAILURES - i)),
                "{e}"
            );
        }
        let e = unlock(&p, "999999").unwrap_err();
        assert!(e.to_string().contains("o PIN foi desligado"), "{e}");
        assert!(!is_enabled(&p));
        assert!(unlock(&p, "407193").is_err());
    }
}
