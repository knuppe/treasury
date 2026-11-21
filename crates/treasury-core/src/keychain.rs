//! Chave do banco no Keychain do macOS / Secret Service (libsecret) no Linux, para
//! desbloqueio rápido e para o `treasury mcp` funcionar com o app fechado (só leitura).
//!
//! - Serviço [`SERVICE`]; conta = hash curto do caminho do banco ([`account_for`]), então
//!   cada cofre tem o seu item.
//! - O segredo guardado é [`DbKey::to_secret_hex`] (chave + salt + versão do KDF).
//! - [`is_available`] e [`has_key`] nunca leem o segredo: só buscam atributos, o que não
//!   abre o diálogo de acesso do Keychain. Só [`load_key`] lê o segredo.
//!
//! O armazenamento fica atrás de [`SecretStore`]. O padrão é o do sistema ([`OsStore`]);
//! os testes do núcleo usam [`MemoryStore`] e nunca tocam o keychain real do usuário.
//! `TREASURY_KEYCHAIN=memory` (em memória) ou `TREASURY_KEYCHAIN=off` (indisponível)
//! trocam o padrão, para desenvolvimento e testes E2E; [`set_store`] troca em código.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{Error, Result};
use crate::vault::DbKey;

pub const SERVICE: &str = "io.github.knuppe.treasury.vault";

/// Serviço que nunca tem itens: usado só para conferir se o armazenamento responde.
const PROBE_SERVICE: &str = "io.github.knuppe.treasury.probe";

/// Serviço de antes do identificador `io.github.knuppe.treasury` (docs/decisions.md, D30). Um
/// item que só existe nele é copiado para o serviço novo na primeira leitura e apagado.
pub const LEGACY_SERVICE: &str = "dev.treasury.vault";

/// Onde os segredos ficam guardados. Implementações não podem registrar o segredo em log.
pub trait SecretStore: Send + Sync {
    /// Nome curto para diagnóstico (`os`, `memory`, `off`).
    fn name(&self) -> &'static str;
    /// O armazenamento existe e responde. Deve ser rápido e nunca abrir diálogo.
    fn is_available(&self) -> bool;
    /// Existe um item para a conta (sem ler o segredo).
    fn exists(&self, account: &str) -> Result<bool>;
    fn get(&self, account: &str) -> Result<Option<Zeroizing<String>>>;
    fn set(&self, account: &str, secret: &str) -> Result<()>;
    /// Apaga o item; não é erro se ele não existir.
    fn delete(&self, account: &str) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Funções usadas pelo resto do núcleo

/// O keychain do sistema está disponível (resultado guardado em cache; sem diálogo).
pub fn is_available() -> bool {
    store().is_available()
}

/// Existe uma chave guardada para este banco.
pub fn has_key(db_path: &Path) -> Result<bool> {
    let s = store();
    if !s.is_available() {
        return Ok(false);
    }
    s.exists(&account_for(db_path))
}

fn unavailable() -> Error {
    Error::invalid(crate::i18n::t("nucleo-geral-erro-keychain-indisponivel"))
}

/// Guarda (ou substitui) a chave do banco.
pub fn store_key(db_path: &Path, key: &DbKey) -> Result<()> {
    let s = store();
    if !s.is_available() {
        return Err(unavailable());
    }
    s.set(&account_for(db_path), &key.to_secret_hex())
}

/// Lê a chave do banco, se houver. Pode pedir autorização ao usuário (Touch ID, senha
/// do keychain), conforme a política do sistema.
pub fn load_key(db_path: &Path) -> Result<Option<DbKey>> {
    let s = store();
    if !s.is_available() {
        return Ok(None);
    }
    match s.get(&account_for(db_path))? {
        Some(hex) => DbKey::from_secret_hex(&hex).map(Some),
        None => Ok(None),
    }
}

/// Remove a chave do banco (não é erro se não houver).
pub fn delete_key(db_path: &Path) -> Result<()> {
    let s = store();
    if !s.is_available() {
        return Ok(());
    }
    s.delete(&account_for(db_path))
}

/// Conta do item: `db-` + 16 primeiros dígitos hex do SHA-256 do caminho absoluto do
/// banco. Caminhos diferentes para o mesmo arquivo (relativo, symlink da pasta) dão a
/// mesma conta.
pub fn account_for(db_path: &Path) -> String {
    let canonical = canonical_path(db_path);
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    format!("db-{}", &hex::encode(digest)[..16])
}

fn canonical_path(p: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(p) {
        return c;
    }
    // O arquivo pode ainda não existir: normaliza a pasta e mantém o nome.
    match (p.parent(), p.file_name()) {
        (Some(dir), Some(name)) if !dir.as_os_str().is_empty() => std::fs::canonicalize(dir)
            .map(|d| d.join(name))
            .unwrap_or_else(|_| p.to_path_buf()),
        _ => std::env::current_dir()
            .map(|d| d.join(p))
            .unwrap_or_else(|_| p.to_path_buf()),
    }
}

// ---------------------------------------------------------------------------
// Seleção do armazenamento

static STORE: RwLock<Option<Arc<dyn SecretStore>>> = RwLock::new(None);

/// Armazenamento em uso (cria o padrão na primeira chamada).
pub fn store() -> Arc<dyn SecretStore> {
    if let Some(s) = STORE.read().unwrap_or_else(|p| p.into_inner()).as_ref() {
        return s.clone();
    }
    let mut guard = STORE.write().unwrap_or_else(|p| p.into_inner());
    guard.get_or_insert_with(default_store).clone()
}

/// Troca o armazenamento do processo (testes de outros crates, servidor de desenvolvimento).
pub fn set_store(store: Arc<dyn SecretStore>) {
    *STORE.write().unwrap_or_else(|p| p.into_inner()) = Some(store);
}

fn default_store() -> Arc<dyn SecretStore> {
    if cfg!(test) {
        return Arc::new(MemoryStore::default());
    }
    match std::env::var("TREASURY_KEYCHAIN").ok().as_deref() {
        Some("memory") => Arc::new(MemoryStore::default()),
        Some("off") | Some("disabled") | Some("none") => Arc::new(MemoryStore::unavailable()),
        _ => Arc::new(MigratingStore::new(
            Arc::new(OsStore::default()),
            Arc::new(OsStore::for_service(LEGACY_SERVICE)),
        )),
    }
}

// ---------------------------------------------------------------------------
// Migração do serviço antigo

/// Armazenamento com um serviço antigo atrás: lê do novo e, se a chave só existir no antigo,
/// copia para o novo e apaga a antiga (uma vez, na primeira leitura). Gravar ou apagar no
/// novo também apaga o antigo, para não sobrar cópia.
pub struct MigratingStore {
    current: Arc<dyn SecretStore>,
    legacy: Arc<dyn SecretStore>,
}

impl MigratingStore {
    pub fn new(current: Arc<dyn SecretStore>, legacy: Arc<dyn SecretStore>) -> Self {
        Self { current, legacy }
    }

    fn forget_legacy(&self, account: &str) {
        if let Err(e) = self.legacy.delete(account) {
            tracing::warn!("keychain: não apaguei o item do serviço antigo: {e}");
        }
    }
}

impl SecretStore for MigratingStore {
    fn name(&self) -> &'static str {
        self.current.name()
    }

    fn is_available(&self) -> bool {
        self.current.is_available()
    }

    fn exists(&self, account: &str) -> Result<bool> {
        Ok(self.current.exists(account)? || self.legacy.exists(account).unwrap_or(false))
    }

    fn get(&self, account: &str) -> Result<Option<Zeroizing<String>>> {
        if let Some(s) = self.current.get(account)? {
            return Ok(Some(s));
        }
        let Some(old) = self.legacy.get(account)? else {
            return Ok(None);
        };
        match self.current.set(account, &old) {
            Ok(()) => {
                self.forget_legacy(account);
                tracing::info!("keychain: chave do cofre migrada para {SERVICE}");
            }
            Err(e) => tracing::warn!("keychain: não migrei a chave para o serviço novo: {e}"),
        }
        Ok(Some(old))
    }

    fn set(&self, account: &str, secret: &str) -> Result<()> {
        self.current.set(account, secret)?;
        self.forget_legacy(account);
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<()> {
        self.current.delete(account)?;
        self.forget_legacy(account);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Keychain do sistema

/// Keychain do macOS (login keychain) ou Secret Service no Linux, via crate `keyring`.
pub struct OsStore {
    service: &'static str,
    available: OnceLock<bool>,
}

impl Default for OsStore {
    fn default() -> Self {
        Self::for_service(SERVICE)
    }
}

/// Nome do atributo de conta na busca de cada armazenamento.
#[cfg(target_os = "macos")]
const USER_ATTR: &str = "user";
#[cfg(not(target_os = "macos"))]
const USER_ATTR: &str = "username";

impl OsStore {
    /// Itens de outro serviço (o antigo, na migração).
    pub fn for_service(service: &'static str) -> Self {
        Self {
            service,
            available: OnceLock::new(),
        }
    }

    fn entry(&self, account: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(self.service, account).map_err(map_keyring_error)
    }
}

impl SecretStore for OsStore {
    fn name(&self) -> &'static str {
        "os"
    }

    fn is_available(&self) -> bool {
        *self.available.get_or_init(|| {
            if keyring::Entry::store_status().is_err() {
                return false;
            }
            // Busca por um serviço que não tem itens: confere que o armazenamento
            // responde sem ler nem destravar nada.
            let spec = HashMap::from([("service", PROBE_SERVICE)]);
            match keyring_core::Entry::search(&spec) {
                Ok(_) => true,
                Err(e) => {
                    tracing::debug!("keychain indisponível: {e}");
                    false
                }
            }
        })
    }

    fn exists(&self, account: &str) -> Result<bool> {
        let spec = HashMap::from([("service", self.service), (USER_ATTR, account)]);
        match keyring_core::Entry::search(&spec) {
            Ok(found) => Ok(!found.is_empty()),
            Err(keyring_core::Error::NoEntry) => Ok(false),
            Err(e) => Err(map_keyring_error(e)),
        }
    }

    fn get(&self, account: &str) -> Result<Option<Zeroizing<String>>> {
        match self.entry(account)?.get_password() {
            Ok(s) => Ok(Some(Zeroizing::new(s))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(map_keyring_error(e)),
        }
    }

    fn set(&self, account: &str, secret: &str) -> Result<()> {
        self.entry(account)?
            .set_password(secret)
            .map_err(map_keyring_error)
    }

    fn delete(&self, account: &str) -> Result<()> {
        match self.entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(map_keyring_error(e)),
        }
    }
}

fn map_keyring_error(e: keyring::Error) -> Error {
    match e {
        keyring::Error::NoStorageAccess(_) => {
            Error::Forbidden(crate::i18n::t("nucleo-geral-erro-keychain-negado"))
        }
        keyring::Error::NoDefaultStore => unavailable(),
        // Mensagens do sistema não carregam o segredo.
        other => Error::Other(crate::i18n::t_args(
            "nucleo-geral-erro-keychain",
            &crate::i18n::args(&[("erro", other.to_string().into())]),
        )),
    }
}

// ---------------------------------------------------------------------------
// Em memória

/// Armazenamento em memória do processo: testes e desenvolvimento. Nada é persistido.
pub struct MemoryStore {
    available: bool,
    items: Mutex<HashMap<String, Zeroizing<String>>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            available: true,
            items: Mutex::new(HashMap::new()),
        }
    }
}

impl MemoryStore {
    /// Simula um sistema sem keychain.
    pub fn unavailable() -> Self {
        Self {
            available: false,
            ..Self::default()
        }
    }

    fn items(&self) -> std::sync::MutexGuard<'_, HashMap<String, Zeroizing<String>>> {
        self.items.lock().unwrap_or_else(|p| p.into_inner())
    }
}

impl SecretStore for MemoryStore {
    fn name(&self) -> &'static str {
        if self.available { "memory" } else { "off" }
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn exists(&self, account: &str) -> Result<bool> {
        Ok(self.items().contains_key(account))
    }

    fn get(&self, account: &str) -> Result<Option<Zeroizing<String>>> {
        Ok(self.items().get(account).cloned())
    }

    fn set(&self, account: &str, secret: &str) -> Result<()> {
        if !self.available {
            return Err(unavailable());
        }
        self.items()
            .insert(account.to_string(), Zeroizing::new(secret.to_string()));
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<()> {
        self.items().remove(account);
        Ok(())
    }
}

#[cfg(all(test, target_os = "macos"))]
mod enclave_poc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::KDF_TEST;

    /// Chave que só existe no serviço antigo (`dev.treasury.vault`): a primeira leitura copia
    /// para o novo e apaga a antiga; depois tudo vem do novo.
    #[test]
    fn a_key_only_in_the_old_service_moves_on_first_read() {
        let current = Arc::new(MemoryStore::default());
        let legacy = Arc::new(MemoryStore::default());
        legacy.set("db-inventada", "segredo-inventado").unwrap();
        let s = MigratingStore::new(current.clone(), legacy.clone());
        assert!(s.exists("db-inventada").unwrap());
        assert_eq!(
            s.get("db-inventada")
                .unwrap()
                .as_deref()
                .map(String::as_str),
            Some("segredo-inventado")
        );
        assert!(current.exists("db-inventada").unwrap());
        assert!(!legacy.exists("db-inventada").unwrap());
        // Gravar e apagar valem para os dois serviços.
        legacy.set("db-outra", "velha").unwrap();
        s.set("db-outra", "nova").unwrap();
        assert!(!legacy.exists("db-outra").unwrap());
        s.delete("db-inventada").unwrap();
        assert!(!s.exists("db-inventada").unwrap());
        assert_eq!(s.get("nenhuma").unwrap(), None);
    }

    #[test]
    fn default_store_in_tests_is_memory() {
        assert_eq!(store().name(), "memory");
        assert!(is_available());
    }

    #[test]
    fn store_load_delete_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("treasury.db");
        let key = DbKey::derive("senha", [7u8; 16], KDF_TEST).unwrap();

        assert!(!has_key(&db).unwrap());
        assert!(load_key(&db).unwrap().is_none());
        store_key(&db, &key).unwrap();
        assert!(has_key(&db).unwrap());
        let back = load_key(&db).unwrap().unwrap();
        assert_eq!(back.key_bytes(), key.key_bytes());
        assert_eq!(back.salt(), key.salt());
        assert_eq!(back.kdf_version(), key.kdf_version());

        // Outro banco não enxerga a chave.
        assert!(!has_key(&dir.path().join("outro.db")).unwrap());

        delete_key(&db).unwrap();
        assert!(!has_key(&db).unwrap());
        delete_key(&db).unwrap();
    }

    #[test]
    fn account_is_short_stable_and_path_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("treasury.db");
        std::fs::write(&db, b"x").unwrap();
        let a = account_for(&db);
        assert!(a.starts_with("db-"));
        assert_eq!(a.len(), 3 + 16);
        assert_eq!(a, account_for(&dir.path().join(".").join("treasury.db")));
        assert_ne!(a, account_for(&dir.path().join("outro.db")));
        assert!(!a.contains("treasury"), "a conta não expõe o caminho");
    }

    #[test]
    fn vault_unlocks_with_keychain_through_the_api() {
        use serde_json::{Value, json};
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::paths::TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
        let api = crate::Api::new_unlocked_process(paths).unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
        )
        .unwrap();
        let st = api.call("vault.status", Value::Null).unwrap();
        assert_eq!(st["keychain_available"], true);
        assert_eq!(st["keychain_enabled"], false);
        assert!(
            api.call(
                "vault.keychain_enable",
                json!({"password": "errada-errada"})
            )
            .is_err()
        );
        let st = api
            .call(
                "vault.keychain_enable",
                json!({"password": "senha-mestra-forte"}),
            )
            .unwrap();
        assert_eq!(st["keychain_enabled"], true);

        api.call("vault.lock", Value::Null).unwrap();
        let st = api.call("vault.unlock_keychain", Value::Null).unwrap();
        assert_eq!(st["state"], "unlocked");

        // A chave do keychain também abre o banco em modo leitura (treasury mcp).
        let key = load_key(&api.paths().db_path).unwrap().unwrap();
        let ro = crate::storage::Db::open_with_key(&api.paths().db_path, key, true).unwrap();
        assert!(ro.is_read_only());
        drop(ro);

        let st = api.call("vault.keychain_disable", Value::Null).unwrap();
        assert_eq!(st["keychain_enabled"], false);
        api.call("vault.lock", Value::Null).unwrap();
        assert!(api.call("vault.unlock_keychain", Value::Null).is_err());
    }

    #[test]
    fn unavailable_store_refuses_and_reports_nothing() {
        let s = MemoryStore::unavailable();
        assert!(!s.is_available());
        assert!(s.set("db-x", "segredo").is_err());
        assert_eq!(s.name(), "off");
    }
}
