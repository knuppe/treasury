//! Cofre: chave do banco derivada da senha mestra com Argon2id e entregue ao SQLCipher
//! como chave bruta. O salt (16 bytes, não secreto) fica no cabeçalho do próprio arquivo,
//! que o SQLCipher já reserva para isso: o banco abre só com a senha e o arquivo.

use std::alloc::Layout;
use std::fmt;
use std::io::Read;
use std::path::Path;
use std::ptr::NonNull;

use zeroize::{Zeroize, Zeroizing};

use crate::error::{Error, Result};
use crate::i18n::t;

pub const KEY_LEN: usize = 32;
pub const SALT_LEN: usize = 16;

/// Parâmetros de custo do Argon2id, fixados por versão. Novas versões entram no início
/// de [`kdf_versions`]; versões antigas continuam abrindo bancos existentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub version: u8,
    pub m_kib: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

/// v1: 64 MiB, 4 passagens, 1 via (~0,3–0,6 s numa máquina comum).
pub const KDF_V1: KdfParams = KdfParams {
    version: 1,
    m_kib: 64 * 1024,
    t_cost: 4,
    p_cost: 1,
};

/// Versão rápida e insegura, só para testes automatizados e desenvolvimento.
#[cfg(any(test, feature = "fast-kdf"))]
pub const KDF_TEST: KdfParams = KdfParams {
    version: 0,
    m_kib: 64,
    t_cost: 1,
    p_cost: 1,
};

/// Versões aceitas ao abrir, da mais nova para a mais antiga.
pub fn kdf_versions() -> Vec<KdfParams> {
    #[allow(unused_mut)]
    let mut v = vec![KDF_V1];
    #[cfg(any(test, feature = "fast-kdf"))]
    v.push(KDF_TEST);
    v
}

/// Versão usada para criar cofres novos.
pub fn default_kdf() -> KdfParams {
    #[cfg(any(test, feature = "fast-kdf"))]
    if std::env::var_os("TREASURY_SECURE_KDF").is_none() {
        return KDF_TEST;
    }
    KDF_V1
}

/// Senha, PIN ou código digitado: zerado da memória ao sair de escopo (docs/local-security.md,
/// item 6). Vem dos argumentos dos comandos (`serde`), sem cópia comum no caminho.
pub type Secret = Zeroizing<String>;

/// Os 32 bytes da chave numa página só dela, travada na memória (`mlock`: nunca vai para o
/// swap; no Linux também fora de core dumps) e zerada ao sair (docs/local-security.md,
/// item 8). Página própria porque o `mlock` vale por página e não conta referências: com a
/// chave numa caixa comum, soltar outra coisa na mesma página destravaria a chave. No
/// Windows o equivalente é `VirtualLock`; até ele entrar, lá a chave só é zerada.
struct LockedKey(NonNull<[u8; KEY_LEN]>);

// SAFETY: a página é só desta chave (como uma `Box`); ninguém mais tem o ponteiro.
unsafe impl Send for LockedKey {}
unsafe impl Sync for LockedKey {}

fn key_page() -> Layout {
    #[cfg(unix)]
    // SAFETY: consulta sem efeito colateral.
    let page = usize::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).unwrap_or(4096);
    #[cfg(not(unix))]
    let page = 4096;
    let page = page.max(KEY_LEN).next_power_of_two();
    Layout::from_size_align(page, page).unwrap_or_else(|_| Layout::new::<[u8; KEY_LEN]>())
}

impl LockedKey {
    fn new(bytes: &[u8; KEY_LEN]) -> Self {
        let layout = key_page();
        // SAFETY: layout com tamanho não nulo; a página é zerada, recebe os 32 bytes e só é
        // solta no `Drop`, com o mesmo layout.
        unsafe {
            let Some(page) = NonNull::new(std::alloc::alloc_zeroed(layout)) else {
                std::alloc::handle_alloc_error(layout)
            };
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), page.as_ptr(), KEY_LEN);
            page_lock(page.as_ptr(), layout.size());
            Self(page.cast())
        }
    }
}

impl Clone for LockedKey {
    fn clone(&self) -> Self {
        Self::new(self)
    }
}

impl Drop for LockedKey {
    fn drop(&mut self) {
        let layout = key_page();
        // SAFETY: a página alocada em `new` com este layout, ainda viva e só desta chave.
        unsafe {
            (*self.0.as_ptr()).zeroize();
            page_unlock(self.0.as_ptr().cast(), layout.size());
            std::alloc::dealloc(self.0.as_ptr().cast(), layout);
        }
    }
}

impl std::ops::Deref for LockedKey {
    type Target = [u8; KEY_LEN];
    fn deref(&self) -> &[u8; KEY_LEN] {
        // SAFETY: página viva enquanto a chave existir.
        unsafe { self.0.as_ref() }
    }
}

#[cfg(unix)]
fn page_lock(ptr: *mut u8, len: usize) {
    // SAFETY: página válida e viva; falhar (limite do sistema) não é erro, a chave só fica
    // sem a trava.
    unsafe {
        libc::mlock(ptr.cast(), len);
        #[cfg(target_os = "linux")]
        libc::madvise(ptr.cast(), len, libc::MADV_DONTDUMP);
    }
}

#[cfg(unix)]
fn page_unlock(ptr: *mut u8, len: usize) {
    // SAFETY: a mesma página travada em `page_lock`, ainda viva.
    unsafe {
        libc::munlock(ptr.cast(), len);
    }
}

#[cfg(not(unix))]
fn page_lock(_ptr: *mut u8, _len: usize) {}

#[cfg(not(unix))]
fn page_unlock(_ptr: *mut u8, _len: usize) {}

/// Endurece o processo no build de release (docs/local-security.md, "Linux depois"): sem
/// core dump em nenhum unix e, no Linux, `PR_SET_DUMPABLE = 0`, que impede outro processo do
/// mesmo usuário de ler a memória (`ptrace`, `/proc/<pid>/mem`) sem root. No macOS o mesmo
/// vem do hardened runtime (scripts/release.sh). No Windows, quando o build voltar:
/// `SetProcessMitigationPolicy` (só DLLs assinadas pela Microsoft) e `SetErrorMode` sem
/// relatório de falha com memória. No build de desenvolvimento não faz nada, para o
/// depurador continuar funcionando.
pub fn harden_process() {
    // Off Unix there is nothing to do yet (an early `return` would be needless there).
    #[cfg(unix)]
    if !cfg!(debug_assertions) {
        // SAFETY: chamadas de sistema sem ponteiros guardados; falhar só deixa o processo como
        // estava.
        unsafe {
            let none = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            libc::setrlimit(libc::RLIMIT_CORE, &none);
            #[cfg(target_os = "linux")]
            libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
        }
    }
}

/// Chave bruta do banco + salt do arquivo. Zerada da memória ao sair de escopo.
#[derive(Clone)]
pub struct DbKey {
    key: LockedKey,
    salt: [u8; SALT_LEN],
    kdf_version: u8,
}

impl fmt::Debug for DbKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DbKey")
            .field("kdf_version", &self.kdf_version)
            .finish_non_exhaustive()
    }
}

impl DbKey {
    pub fn derive(password: &str, salt: [u8; SALT_LEN], params: KdfParams) -> Result<Self> {
        let p = argon2::Params::new(params.m_kib, params.t_cost, params.p_cost, Some(KEY_LEN))
            .map_err(|e| Error::Crypto(e.to_string()))?;
        let argon = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, p);
        let mut key = Zeroizing::new([0u8; KEY_LEN]);
        argon
            .hash_password_into(password.as_bytes(), &salt, key.as_mut())
            .map_err(|e| Error::Crypto(e.to_string()))?;
        Ok(Self {
            key: LockedKey::new(&key),
            salt,
            kdf_version: params.version,
        })
    }

    pub fn from_parts(key: [u8; KEY_LEN], salt: [u8; SALT_LEN], kdf_version: u8) -> Self {
        let key = Zeroizing::new(key);
        Self {
            key: LockedKey::new(&key),
            salt,
            kdf_version,
        }
    }

    pub fn random_salt() -> Result<[u8; SALT_LEN]> {
        let mut s = [0u8; SALT_LEN];
        getrandom::fill(&mut s).map_err(|e| Error::Crypto(e.to_string()))?;
        Ok(s)
    }

    pub fn salt(&self) -> [u8; SALT_LEN] {
        self.salt
    }

    pub fn kdf_version(&self) -> u8 {
        self.kdf_version
    }

    pub fn key_bytes(&self) -> &[u8; KEY_LEN] {
        &self.key
    }

    /// Valor para `PRAGMA key`/`rekey`: `x'<chave 32 bytes><salt 16 bytes>'` em hex.
    pub fn pragma_value(&self) -> Zeroizing<String> {
        let mut raw = Zeroizing::new(Vec::with_capacity(KEY_LEN + SALT_LEN));
        raw.extend_from_slice(self.key.as_ref());
        raw.extend_from_slice(&self.salt);
        Zeroizing::new(format!("x'{}'", hex::encode_upper(raw.as_slice())))
    }

    /// Mesma chave com outro salt (usado quando uma senha é aplicada a outro arquivo).
    pub fn with_salt(&self, salt: [u8; SALT_LEN]) -> Self {
        Self {
            key: self.key.clone(),
            salt,
            kdf_version: self.kdf_version,
        }
    }

    /// Kit de recuperação: a chave em base32, em grupos de 4 caracteres.
    pub fn recovery_code(&self) -> Secret {
        let mut raw = Zeroizing::new(Vec::with_capacity(KEY_LEN + 1));
        raw.push(self.kdf_version);
        raw.extend_from_slice(self.key.as_ref());
        let enc = Zeroizing::new(base32::encode(
            base32::Alphabet::Rfc4648 { padding: false },
            &raw,
        ));
        Zeroizing::new(
            enc.as_bytes()
                .chunks(4)
                .map(|c| std::str::from_utf8(c).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("-"),
        )
    }

    pub fn from_recovery_code(code: &str, salt: [u8; SALT_LEN]) -> Result<Self> {
        let clean: String = code
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_uppercase();
        let mut raw = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &clean)
            .ok_or_else(|| Error::invalid(t("nucleo-cofre-erro-codigo-recuperacao")))?;
        if raw.len() != KEY_LEN + 1 {
            raw.zeroize();
            return Err(Error::invalid(t("nucleo-cofre-erro-codigo-recuperacao")));
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&raw[1..]);
        let v = raw[0];
        raw.zeroize();
        let out = Self::from_parts(key, salt, v);
        key.zeroize();
        Ok(out)
    }

    /// Representação hex (chave + salt + versão), usada só para guardar no keychain.
    pub fn to_secret_hex(&self) -> Zeroizing<String> {
        let mut raw = Zeroizing::new(Vec::with_capacity(KEY_LEN + SALT_LEN + 1));
        raw.push(self.kdf_version);
        raw.extend_from_slice(self.key.as_ref());
        raw.extend_from_slice(&self.salt);
        Zeroizing::new(hex::encode(raw.as_slice()))
    }

    pub fn from_secret_hex(s: &str) -> Result<Self> {
        let raw = Zeroizing::new(
            hex::decode(s.trim())
                .map_err(|_| Error::Crypto(t("nucleo-cofre-erro-segredo-keychain")))?,
        );
        if raw.len() != KEY_LEN + SALT_LEN + 1 {
            return Err(Error::Crypto(t("nucleo-cofre-erro-segredo-keychain")));
        }
        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&raw[1..1 + KEY_LEN]);
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&raw[1 + KEY_LEN..]);
        let out = Self::from_parts(key, salt, raw[0]);
        key.zeroize();
        Ok(out)
    }
}

/// Lê o salt gravado nos 16 primeiros bytes do arquivo do banco.
pub fn read_salt(path: &Path) -> Result<[u8; SALT_LEN]> {
    let mut f = std::fs::File::open(path)?;
    let mut salt = [0u8; SALT_LEN];
    f.read_exact(&mut salt)
        .map_err(|_| Error::invalid(t("nucleo-cofre-erro-banco-corrompido")))?;
    Ok(salt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivation_is_deterministic_and_salted() {
        let s1 = [1u8; SALT_LEN];
        let s2 = [2u8; SALT_LEN];
        let a = DbKey::derive("senha forte", s1, KDF_TEST).unwrap();
        let b = DbKey::derive("senha forte", s1, KDF_TEST).unwrap();
        let c = DbKey::derive("senha forte", s2, KDF_TEST).unwrap();
        assert_eq!(a.key_bytes(), b.key_bytes());
        assert_ne!(a.key_bytes(), c.key_bytes());
        assert!(a.pragma_value().starts_with("x'"));
        assert_eq!(a.pragma_value().len(), 2 + 96 + 1);
    }

    #[test]
    fn recovery_code_roundtrip() {
        let k = DbKey::derive("x", [9u8; SALT_LEN], KDF_TEST).unwrap();
        let code = k.recovery_code();
        let back = DbKey::from_recovery_code(&code.to_lowercase(), [9u8; SALT_LEN]).unwrap();
        assert_eq!(back.key_bytes(), k.key_bytes());
        assert_eq!(back.kdf_version(), k.kdf_version());
        assert!(DbKey::from_recovery_code("AAAA-BBBB", [0; SALT_LEN]).is_err());
    }

    #[test]
    fn locked_key_lives_on_its_own_page() {
        let a = LockedKey::new(&[7u8; KEY_LEN]);
        let b = a.clone();
        let page = key_page().align();
        assert_eq!(a.0.as_ptr() as usize % page, 0);
        assert_ne!(a.0.as_ptr() as usize / page, b.0.as_ptr() as usize / page);
        drop(a);
        assert_eq!(*b, [7u8; KEY_LEN]);
    }

    #[test]
    fn secret_hex_roundtrip() {
        let k = DbKey::derive("y", [3u8; SALT_LEN], KDF_TEST).unwrap();
        let back = DbKey::from_secret_hex(&k.to_secret_hex()).unwrap();
        assert_eq!(back.key_bytes(), k.key_bytes());
        assert_eq!(back.salt(), k.salt());
    }
}
