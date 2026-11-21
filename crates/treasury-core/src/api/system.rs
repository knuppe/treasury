//! Comandos de sistema: cofre (criar, desbloquear, bloquear), app, backups e ajustes.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::{Actor, Api, Ctx, Scratch, Session, State, parse_args, parse_ref_args, to_value};
use crate::backup::{self, BackupKind, BackupSettings};
use crate::changes::{self, Author};
use crate::error::{Error, Result};
use crate::i18n::{t, t_args};
use crate::storage::Db;
use crate::vault::{self, DbKey};
use crate::{keychain, settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum VaultState {
    NoVault,
    Locked,
    Unlocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VaultStatus {
    pub state: VaultState,
    pub db_path: String,
    #[ts(type = "number")]
    pub auto_lock_minutes: u64,
    pub keychain_available: bool,
    pub keychain_enabled: bool,
    /// PIN ligado neste computador (desbloqueio rápido).
    pub pin_enabled: bool,
    /// Chaves físicas cadastradas (recuperação).
    #[ts(type = "number")]
    pub fido2_keys: u32,
    pub household_name: Option<String>,
    /// Cofre ativo (docs/vaults.md).
    pub vault_id: String,
    pub vault_name: String,
    /// Cofres na lista (1: só o principal).
    #[ts(type = "number")]
    pub vault_count: u32,
    /// Cofre de demonstração (dados fictícios do assistente; docs/demo.md).
    pub demo: bool,
    /// O cofre ativo é uma demonstração que abre sem senha (`vault.unlock_demo`).
    pub demo_no_password: bool,
    /// Uma versão mais nova do app já abriu este cofre (o esquema é compatível, mas ela pode
    /// ter gravado ajustes que esta versão não conhece): a interface sugere atualizar.
    pub newer_app_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AppInfo {
    pub version: String,
    #[ts(type = "number")]
    pub schema_version: i32,
    pub db_path: String,
    pub backups_dir: String,
    pub data_root: String,
    pub platform: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct CreateVaultArgs {
    #[ts(type = "string")]
    pub password: vault::Secret,
    pub household_name: String,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub base_currency: Option<String>,
    /// País onde a família mora (ISO 3166-1 alfa-2; padrão: Brasil). Dá a moeda base quando
    /// ela não vem, as fontes de dados, as categorias e as regras prontas (docs/countries.md).
    #[serde(default)]
    #[ts(optional)]
    pub country: Option<String>,
}

#[derive(Deserialize)]
struct PasswordArg {
    password: vault::Secret,
}

#[derive(Deserialize)]
struct RecoveryArgs {
    code: vault::Secret,
    new_password: vault::Secret,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ChangePasswordArgs {
    #[ts(type = "string")]
    pub current: vault::Secret,
    #[ts(type = "string")]
    pub new: vault::Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecoveryKit {
    #[ts(type = "string")]
    pub code: crate::vault::Secret,
    pub created_at: String,
    pub db_path: String,
}

/// Força mínima da senha mestra (a interface mostra um medidor mais completo).
pub fn validate_master_password(pw: &str) -> Result<()> {
    if pw.chars().count() < 10 {
        return Err(Error::WeakPassword(t("nucleo-base-erro-senha-curta")));
    }
    Ok(())
}

pub(super) fn status(api: &Api) -> VaultStatus {
    let paths = api.paths();
    let (state, household_name, demo, newer_app_version) = match &*api.state() {
        State::Open(s) => (
            VaultState::Unlocked,
            s.db.conn()
                .query_row("SELECT name FROM household LIMIT 1", [], |r| {
                    r.get::<_, String>(0)
                })
                .ok(),
            settings::get_or(s.db.conn(), crate::demo::SETTING, false).unwrap_or(false),
            crate::vault_versions::opened_by_newer(s.db.conn(), crate::config::app_version()),
        ),
        State::Closed if paths.db_path.exists() => (VaultState::Locked, None, false, None),
        State::Closed => (VaultState::NoVault, None, false, None),
    };
    let vault_list = vault_list(api);
    let demo_no_password = vault_list.vaults.iter().any(|v| v.active && v.demo);
    VaultStatus {
        state,
        db_path: paths.db_path.to_string_lossy().into_owned(),
        auto_lock_minutes: api.auto_lock_after().as_secs() / 60,
        keychain_available: keychain::is_available(),
        keychain_enabled: keychain::has_key(&paths.db_path).unwrap_or(false),
        pin_enabled: crate::unlock::pin::is_enabled(&paths.db_path),
        fido2_keys: crate::unlock::fido2_slots(&paths.db_path)
            .map(|v| v.len() as u32)
            .unwrap_or(0),
        household_name,
        vault_count: vault_list.vaults.len() as u32,
        vault_name: vault_list
            .vaults
            .iter()
            .find(|v| v.active)
            .map(|v| v.name.clone())
            .unwrap_or_default(),
        vault_id: paths.vault_id.clone(),
        demo,
        demo_no_password,
        newer_app_version,
    }
}

fn load_config(api: &Api) -> Result<crate::config::Config> {
    crate::config::Config::load(&api.paths().config_path)
}

fn vault_list(api: &Api) -> crate::vaults::VaultList {
    let paths = api.paths();
    let cfg = load_config(api).unwrap_or_default();
    crate::vaults::list(&cfg, &paths.root)
}

/// Muda a lista de cofres no `config.toml` e, se o cofre ativo mudou, troca de cofre
/// (bloqueando o atual, com o backup de fechamento).
fn edit_vaults(
    api: &Api,
    f: impl FnOnce(&mut crate::config::Config, &std::path::Path) -> Result<()>,
) -> Result<()> {
    let paths = api.paths();
    let mut cfg = load_config(api)?;
    f(&mut cfg, &paths.root)?;
    let next = crate::vaults::active(&cfg, &paths.root);
    if next.id != paths.vault_id && api.is_unlocked() {
        lock(api, true)?;
    }
    cfg.save(&paths.config_path)?;
    if next.id != paths.vault_id || next.path != paths.db_path {
        api.set_paths(crate::paths::TreasuryPaths::with_root(&paths.root)?)?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct VaultIdArg {
    id: String,
}

#[derive(Deserialize)]
struct VaultAddArgs {
    path: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    purpose: Option<String>,
    /// Já abre este cofre (padrão: sim).
    #[serde(default)]
    select: Option<bool>,
}

#[derive(Deserialize)]
struct VaultNewArgs {
    name: String,
    #[serde(default)]
    purpose: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Deserialize)]
struct VaultCopyArgs {
    id: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Deserialize)]
struct VaultRemoveArgs {
    id: String,
    /// Também manda o arquivo (e o `.keys` dele) para a Lixeira; os backups ficam.
    #[serde(default)]
    delete_file: bool,
}

/// Lixeira do sistema (nos testes, uma pasta dentro dos dados).
fn to_trash(path: &std::path::Path, root: &std::path::Path) -> Result<std::path::PathBuf> {
    if cfg!(test) {
        crate::trash::move_into(path, &root.join("lixeira-testes"))
    } else {
        crate::trash::move_to_trash(path, &root.join("removidos"))
    }
}

/// Copia o arquivo de um cofre: o aberto, pela API de backup do SQLite (cópia consistente com
/// a mesma chave); um fechado, o arquivo como está. O `.keys` (recuperação) vai junto.
fn copy_vault_file(api: &Api, src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let paths = api.paths();
    let open_here = match &*api.state() {
        State::Open(s) if src == paths.db_path => {
            crate::backup::write_copy(s.db.conn(), s.db.key(), dest, s.db.key())?;
            true
        }
        _ => false,
    };
    if !open_here {
        std::fs::copy(src, dest)?;
        crate::paths::set_mode(dest, 0o600)?;
    }
    let keys = crate::unlock::keys_path(src);
    if keys.exists() {
        let to = crate::unlock::keys_path(dest);
        std::fs::copy(&keys, &to)?;
        crate::paths::set_mode(&to, 0o600)?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct DemoVaultArgs {
    name: String,
    household_name: String,
    /// Sem senha (o padrão), o cofre abre pela chave fixa do app e fica marcado como
    /// demonstração sem senha na lista de cofres.
    #[serde(default)]
    password: Option<vault::Secret>,
    profile: crate::demo::DemoProfile,
}

/// Cofre de demonstração criado e preenchido.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DemoVaultResult {
    pub status: VaultStatus,
    pub summary: crate::demo::DemoSummary,
}

#[derive(Deserialize)]
struct VaultUpdateArgs {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    purpose: Option<Option<String>>,
}

fn handle_vaults(api: &Api, action: &str, args: &Value) -> Result<Value> {
    use crate::vaults;
    match action {
        "list" => to_value(vault_list(api)),
        "select" => {
            let a: VaultIdArg = parse_args(args.clone())?;
            edit_vaults(api, |cfg, root| vaults::select(cfg, root, &a.id))?;
            to_value(status(api))
        }
        "add" => {
            let a: VaultAddArgs = parse_args(args.clone())?;
            edit_vaults(api, |cfg, root| {
                let e = vaults::add_existing(
                    cfg,
                    root,
                    std::path::Path::new(a.path.trim()),
                    a.name.as_deref(),
                    a.purpose.as_deref(),
                )?;
                if a.select.unwrap_or(true) {
                    vaults::select(cfg, root, &e.id)?;
                }
                Ok(())
            })?;
            to_value(status(api))
        }
        // Cofre novo: registra e passa a ser o ativo; a tela segue para `vault.create`.
        "new" => {
            let a: VaultNewArgs = parse_args(args.clone())?;
            edit_vaults(api, |cfg, root| {
                let e = vaults::add_new(
                    cfg,
                    root,
                    &a.name,
                    a.purpose.as_deref(),
                    a.path.as_deref().map(str::trim).map(std::path::Path::new),
                )?;
                vaults::select(cfg, root, &e.id)
            })?;
            to_value(status(api))
        }
        // Cofre de demonstração (docs/demo.md): cofre novo e separado (o atual fecha com
        // backup), criado com a senha informada e preenchido pelo assistente.
        "new_demo" => {
            let a: DemoVaultArgs = parse_args(args.clone())?;
            let password = match a.password.as_deref().filter(|p| !p.is_empty()) {
                Some(p) => {
                    validate_master_password(p)?;
                    p.to_string()
                }
                None => crate::demo::DEMO_PASSWORD.to_string(),
            };
            let no_password = a.password.as_ref().is_none_or(|p| p.is_empty());
            let purpose = crate::demo::purpose(a.profile.kind);
            crate::demo::validate(&a.profile)?;
            if a.household_name.trim().is_empty() {
                return Err(Error::invalid(t("nucleo-base-erro-nome-familia")));
            }
            edit_vaults(api, |cfg, root| {
                // O cofre escolhido ainda sem arquivo (primeira vez, ou "Novo cofre" antes da
                // senha) vira a demonstração; senão, ela entra como cofre novo na lista.
                let active = vaults::active(cfg, root);
                let id = if active.path.exists() {
                    vaults::add_new(cfg, root, &a.name, Some(&purpose), None)?.id
                } else {
                    vaults::update(cfg, root, &active.id, Some(&a.name), Some(Some(&purpose)))?;
                    active.id
                };
                if no_password {
                    vaults::mark_demo(cfg, &id);
                }
                vaults::select(cfg, root, &id)
            })?;
            let members = a
                .profile
                .adults
                .iter()
                .map(|x| x.name.trim().to_string())
                .chain(a.profile.children.iter().map(|c| c.trim().to_string()))
                .collect();
            create(
                api,
                CreateVaultArgs {
                    password: password.into(),
                    household_name: a.household_name.trim().to_string(),
                    members,
                    base_currency: None,
                    country: a.profile.country.clone(),
                },
            )?;
            let summary = api.call("demo.generate", serde_json::to_value(&a.profile)?)?;
            to_value(DemoVaultResult {
                status: status(api),
                summary: serde_json::from_value(summary)?,
            })
        }
        "update" => {
            let a: VaultUpdateArgs = parse_args(args.clone())?;
            edit_vaults(api, |cfg, root| {
                vaults::update(
                    cfg,
                    root,
                    &a.id,
                    a.name.as_deref(),
                    a.purpose.as_ref().map(|p| p.as_deref()),
                )
            })?;
            to_value(vault_list(api))
        }
        // Cópia de um cofre (mesma senha), na lista com o nome "… (cópia)"; não troca de cofre.
        "copy" => {
            let a: VaultCopyArgs = parse_args(args.clone())?;
            edit_vaults(api, |cfg, root| {
                let (src, copy) = vaults::add_copy(cfg, root, &a.id, a.name.as_deref())?;
                if let Some(dir) = copy.path.parent() {
                    crate::paths::create_private_dir(dir)?;
                }
                copy_vault_file(api, &src.path, &copy.path).inspect_err(|_| {
                    let _ = std::fs::remove_file(&copy.path);
                })
            })?;
            to_value(vault_list(api))
        }
        // Tira da lista e, se pedido, manda o arquivo para a Lixeira (o cofre em uso, nunca).
        "remove" => {
            let a: VaultRemoveArgs = parse_args(args.clone())?;
            edit_vaults(api, |cfg, root| {
                let entry = vaults::entries(cfg, root)
                    .into_iter()
                    .find(|v| v.id == a.id)
                    .ok_or_else(|| {
                        Error::not_found(t_args(
                            "nucleo-base-cofre",
                            &crate::i18n::args(&[("id", a.id.clone().into())]),
                        ))
                    })?;
                vaults::remove(cfg, root, &a.id)?;
                if a.delete_file && entry.path.exists() {
                    to_trash(&entry.path, root)?;
                    let keys = crate::unlock::keys_path(&entry.path);
                    if keys.exists() {
                        to_trash(&keys, root)?;
                    }
                    // Desbloqueio rápido deste arquivo: não serve mais para nada.
                    let _ = keychain::delete_key(&entry.path);
                    let _ = crate::unlock::pin::disable(&entry.path);
                }
                Ok(())
            })?;
            to_value(vault_list(api))
        }
        _ => Err(Error::UnknownCommand(format!("vaults.{action}"))),
    }
}

fn app_info(api: &Api) -> AppInfo {
    let paths = api.paths();
    let schema_version = match &*api.state() {
        State::Open(s) => s.db.schema_version().unwrap_or(0),
        State::Closed => 0,
    };
    AppInfo {
        version: crate::config::app_version().to_string(),
        schema_version,
        db_path: paths.db_path.to_string_lossy().into_owned(),
        backups_dir: paths.backups_dir.to_string_lossy().into_owned(),
        data_root: paths.root.to_string_lossy().into_owned(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
    }
}

/// Comandos atendidos sem sessão aberta (ou que mexem na própria sessão).
pub(super) fn handle_unlocked(
    api: &Api,
    actor: &Actor,
    module: &str,
    action: &str,
    args: &Value,
) -> Option<Result<Value>> {
    let user_only = |f: &dyn Fn() -> Result<Value>| -> Result<Value> {
        if *actor != Actor::User {
            return Err(Error::Forbidden(t("nucleo-base-erro-cofre-so-interface")));
        }
        f()
    };
    Some(match (module, action) {
        ("app", "info") => to_value(app_info(api)),
        // Tarefas em segundo plano (docs/background-tasks.md): em memória, sem o banco.
        ("tasks", "list") => to_value(crate::tasks::list()),
        ("tasks", "cancel") => user_only(&|| {
            #[derive(Deserialize)]
            struct A {
                id: String,
            }
            let a: A = parse_ref_args(args)?;
            crate::tasks::cancel(&a.id).and_then(to_value)
        }),
        // Idioma da instalação (docs/i18n-design.md): no config.toml, antes de qualquer cofre.
        ("system", "locale") => to_value(crate::i18n::state(&api.paths().config_path)),
        ("system", "set_locale") => user_only(&|| {
            let a: crate::i18n::SetLocaleArgs = parse_ref_args(args)?;
            to_value(crate::i18n::set(&api.paths().config_path, &a)?)
        }),
        // Atualização automática (docs/auto-update.md): por instalação, vale com o cofre
        // fechado.
        ("system", "update_settings") => {
            to_value(crate::config::update_settings(&api.paths().config_path))
        }
        ("system", "set_update_settings") => user_only(&|| {
            let a: crate::config::UpdateSettings = parse_ref_args(args)?;
            to_value(crate::config::set_update_settings(
                &api.paths().config_path,
                &a,
            )?)
        }),
        // Termos de uso (docs/terms.md): antes de existir cofre, por instalação.
        ("terms", "status") => to_value(crate::terms::status(&api.paths().root)),
        ("terms", "accept") => user_only(&|| {
            let a: crate::terms::TermsAcceptArgs = parse_ref_args(args)?;
            to_value(crate::terms::accept(&api.paths().root, &a.version)?)
        }),
        // Países para a criação do cofre (docs/countries.md).
        ("app", "countries") => to_value(crate::country::choices()),
        // Vários cofres (docs/vaults.md): a lista vale com o cofre bloqueado.
        ("vaults", a) => user_only(&|| handle_vaults(api, a, args)),
        ("vault", "status") => to_value(status(api)),
        ("vault", "create") => user_only(&|| create(api, parse_ref_args(args)?).and_then(to_value)),
        ("vault", "unlock") => user_only(&|| {
            let a: PasswordArg = parse_ref_args(args)?;
            match unlock(api, |path| Db::open(path, &a.password)) {
                // A senha não abre o cofre, mas abre o backup mais novo: o cofre pode estar
                // danificado (docs/backup-recovery.md).
                Err(Error::WrongPassword)
                    if backup::newest_opens_with(&api.paths().backups_dir, &a.password) =>
                {
                    Err(Error::VaultDamaged(t("nucleo-base-erro-cofre-danificado")))
                }
                r => r.and_then(to_value),
            }
        }),
        // Recuperação por backup, com o cofre fechado ou quebrado (docs/backup-recovery.md).
        ("backup", "scan") => user_only(&|| to_value(backup::scan(&api.paths().backups_dir))),
        ("backup", "restore_file") => {
            user_only(&|| restore_file(api, parse_ref_args(args)?).and_then(to_value))
        }
        ("backup", "export_file") => user_only(&|| {
            let a: ExportFileArgs = parse_ref_args(args)?;
            backup::export_file(
                &api.paths().backups_dir,
                std::path::Path::new(&a.path),
                std::path::Path::new(&a.dest),
            )?;
            Ok(Value::Null)
        }),
        ("backup", "import_file") => user_only(&|| {
            let a: ImportFileArgs = parse_ref_args(args)?;
            to_value(backup::import_file(
                &api.paths().backups_dir,
                std::path::Path::new(&a.source),
            )?)
        }),
        // Demonstração sem senha: abre pela chave fixa do app, só no cofre marcado assim.
        ("vault", "unlock_demo") => user_only(&|| {
            if !vault_list(api).vaults.iter().any(|v| v.active && v.demo) {
                return Err(Error::invalid(t("nucleo-base-erro-demo-com-senha")));
            }
            unlock(api, |path| Db::open(path, crate::demo::DEMO_PASSWORD)).and_then(to_value)
        }),
        ("vault", "unlock_recovery") => {
            user_only(&|| unlock_recovery(api, parse_ref_args(args)?).and_then(to_value))
        }
        ("vault", "lock") => user_only(&|| {
            lock(api, true)?;
            to_value(status(api))
        }),
        ("vault", "touch") => {
            if let State::Open(s) = &mut *api.state() {
                s.last_activity = Instant::now();
            }
            Ok(Value::Null)
        }
        ("vault", "change_password") => {
            user_only(&|| change_password(api, parse_ref_args(args)?).and_then(to_value))
        }
        ("vault", "recovery_kit") => user_only(&|| {
            let a: PasswordArg = parse_ref_args(args)?;
            recovery_kit(api, &a.password).and_then(to_value)
        }),
        ("vault", "keychain_enable") => user_only(&|| {
            let a: PasswordArg = parse_ref_args(args)?;
            keychain_enable(api, &a.password)?;
            to_value(status(api))
        }),
        ("vault", "keychain_disable") => user_only(&|| {
            keychain::delete_key(&api.paths().db_path)?;
            to_value(status(api))
        }),
        ("vault", "unlock_keychain") => user_only(&|| {
            let path = api.paths().db_path.clone();
            unlock(api, |p| {
                let key = keychain::load_key(&path)?
                    .ok_or_else(|| Error::not_found(t("nucleo-base-chave-keychain")))?;
                Db::open_with_key(p, key, false)
            })
            .and_then(to_value)
        }),
        // Recuperação dividida e chave física (docs/unlock-options.md).
        ("vault", "recovery_status") => user_only(&|| {
            api.with_session(|s, paths| crate::unlock::status(s.db.conn(), &paths.db_path))
                .and_then(to_value)
        }),
        ("vault", "recovery_shares_create") => user_only(&|| {
            #[derive(Deserialize)]
            struct A {
                password: vault::Secret,
                #[serde(default)]
                rotate: bool,
            }
            let a: A = parse_ref_args(args)?;
            api.with_session(|s, paths| {
                verify_password(s, &a.password)?;
                crate::unlock::create_shares(s.db.conn(), s.db.key(), &paths.db_path, a.rotate)
            })
            .and_then(to_value)
        }),
        ("vault", "unlock_shares") => user_only(&|| {
            #[derive(Deserialize)]
            struct A {
                shares: Vec<vault::Secret>,
                new_password: vault::Secret,
            }
            let a: A = parse_ref_args(args)?;
            let path = api.paths().db_path.clone();
            let key = crate::unlock::key_from_shares(&path, &a.shares)?;
            recover_with_key(api, key, &a.new_password).and_then(to_value)
        }),
        ("vault", "pin_enable") => user_only(&|| {
            #[derive(Deserialize)]
            struct A {
                password: vault::Secret,
                pin: vault::Secret,
            }
            let a: A = parse_ref_args(args)?;
            crate::unlock::pin::validate(&a.pin)?;
            api.with_session(|s, paths| {
                verify_password(s, &a.password)?;
                let r = crate::unlock::ensure_root(s.db.conn(), s.db.key(), &paths.db_path)?;
                crate::unlock::pin::enable(&paths.db_path, &r, &a.pin, pin_kdf())
            })?;
            to_value(status(api))
        }),
        ("vault", "pin_disable") => user_only(&|| {
            crate::unlock::pin::disable(&api.paths().db_path)?;
            to_value(status(api))
        }),
        ("vault", "unlock_pin") => user_only(&|| {
            let a: PinArg = parse_ref_args(args)?;
            let path = api.paths().db_path.clone();
            unlock(api, |p| {
                let key = crate::unlock::pin::unlock(&path, &a.pin)?;
                Db::open_with_key(p, key, false)
            })
            .and_then(to_value)
        }),
        ("vault", "fido2_slots") => {
            user_only(&|| to_value(crate::unlock::fido2_slots(&api.paths().db_path)?))
        }
        ("vault", "fido2_add") => user_only(&|| {
            #[derive(Deserialize)]
            struct A {
                label: String,
                rp_id: String,
                credential_id: String,
                salt: String,
                hmac_secret: vault::Secret,
            }
            let a: A = parse_ref_args(args)?;
            let secret = zeroize::Zeroizing::new(
                hex::decode(&a.hmac_secret)
                    .map_err(|_| Error::invalid(t("nucleo-geral-erro-resposta-chave-fisica")))?,
            );
            api.with_session(|s, paths| {
                crate::unlock::add_fido2(
                    s.db.conn(),
                    s.db.key(),
                    &paths.db_path,
                    &a.label,
                    &a.rp_id,
                    &a.credential_id,
                    &a.salt,
                    &secret,
                )
            })
            .and_then(to_value)
        }),
        ("vault", "fido2_remove") => user_only(&|| {
            #[derive(Deserialize)]
            struct A {
                id: String,
                password: vault::Secret,
            }
            let a: A = parse_ref_args(args)?;
            api.with_session(|s, paths| {
                verify_password(s, &a.password)?;
                crate::unlock::remove_fido2(&paths.db_path, &a.id)
            })?;
            Ok(Value::Null)
        }),
        ("vault", "unlock_fido2") => user_only(&|| {
            #[derive(Deserialize)]
            struct A {
                slot_id: String,
                hmac_secret: vault::Secret,
                new_password: vault::Secret,
            }
            let a: A = parse_ref_args(args)?;
            let secret = zeroize::Zeroizing::new(
                hex::decode(&a.hmac_secret)
                    .map_err(|_| Error::invalid(t("nucleo-geral-erro-resposta-chave-fisica")))?,
            );
            let path = api.paths().db_path.clone();
            let key = crate::unlock::key_from_fido2(&path, &a.slot_id, &secret)?;
            recover_with_key(api, key, &a.new_password).and_then(to_value)
        }),
        ("backup", "restore") => {
            user_only(&|| restore(api, parse_ref_args(args)?).and_then(to_value))
        }
        // Coleta de índices e câmbio: libera o banco durante a rede.
        ("indices", "sync") => user_only(&|| crate::indices::sync::run(api).and_then(to_value)),
        ("market", "sync") => user_only(&|| crate::market::sync::run_all(api).and_then(to_value)),
        // Cotação intradiária e busca de tickers no Yahoo: rede sem segurar o banco.
        ("market", "refresh_quotes") => user_only(&|| {
            let a: crate::market::api::QuotesArgs = parse_ref_args(args)?;
            crate::market::intraday::refresh(api, &a.asset_ids.unwrap_or_default())
                .and_then(to_value)
        }),
        ("market", "search") => user_only(&|| {
            crate::market::search::run(api, &parse_ref_args(args)?).and_then(to_value)
        }),
        // Taxas de juros de mercado (BCB): rede fora da trava, só quando os dados estão velhos.
        ("debt", "market_rates") => {
            #[derive(serde::Deserialize, Default)]
            struct A {
                #[serde(default)]
                refresh: bool,
            }
            match parse_args::<A>(args.clone()) {
                Ok(a) => {
                    crate::debt::market_rates::refresh_and_get(api, a.refresh).and_then(to_value)
                }
                Err(e) => Err(e),
            }
        }
        // Open Food Facts pelo código de barras: a rede fora da trava do banco.
        ("food", "open_data_sync") => {
            user_only(&|| crate::food::open_data::sync(api).and_then(to_value))
        }
        // Cupom pela foto (docs/ocr-receipts.md): o OCR embutido e a IA fora da trava do banco.
        // Janela de importação: o que uma imagem é, lida no computador (OCR), antes da IA.
        // O PDF só com imagem passa pelo OCR: fora da trava do banco.
        ("import", "detect") => match parse_args::<crate::import::DetectArgs>(args.clone()) {
            Ok(a) => crate::import::detect(&a).and_then(to_value),
            Err(e) => Err(e),
        },
        ("import", "sniff") => match parse_args::<crate::import::sniff::SniffArgs>(args.clone()) {
            Ok(a) => crate::import::sniff::sniff(&a).and_then(to_value),
            Err(e) => Err(e),
        },
        ("import", "analyze_image") => user_only(&|| {
            crate::import::image::analyze(api, &parse_ref_args(args)?).and_then(to_value)
        }),
        ("receipts", "read_photo") => user_only(&|| {
            crate::receipts::ocr::read_photo(api, &parse_ref_args(args)?).and_then(to_value)
        }),
        // Texto de imagens fora do cupom (docs/ocr-other-documents.md): o OCR sem o banco.
        ("ocr", "read_text") => user_only(&|| {
            let a: crate::receipts::ocr::PhotoArgs = parse_ref_args(args)?;
            let bytes = a
                .image_base64
                .as_deref()
                .map(crate::transactions::b64::decode)
                .transpose()?;
            crate::ocr::read_text(
                a.lines.as_deref(),
                crate::ocr::aspect(a.width, a.height),
                bytes.as_deref(),
            )
            .and_then(to_value)
        }),
        // Boleto, Pix e comprovante lidos no computador: o OCR fora da trava do banco.
        ("payments", "read") => {
            user_only(&|| crate::payments::read(api, &parse_ref_args(args)?).and_then(to_value))
        }
        // E-mail de ingestão: IMAP, autoconfig, buscas e IA fora da trava do banco.
        ("mail", a) if crate::mail::API_LEVEL.contains(&a) => {
            crate::mail::handle_api_level(api, actor, a, args)
        }
        // Agente: jobs em threads próprias, sem segurar o banco durante a rede.
        ("ai", a) if crate::ai::api::API_LEVEL.contains(&a) => {
            crate::ai::api::handle_api_level(api, actor, a, args)
        }
        // Fundamentus: cache local primeiro; a rede (se precisar) não segura o banco.
        ("research", "fundamentus") => crate::research::fundamentus::get_snapshot(
            api,
            &match parse_ref_args(args) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            },
        )
        .and_then(to_value),
        // Dados dos ativos: consulta às fontes públicas da rede (B3, GLEIF, BrasilAPI) fora
        // da trava; grava só leituras, nunca o ativo.
        ("assets", "enrich_lookup") => crate::assets::lookup::run(
            api,
            &match parse_ref_args(args) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            },
        )
        .and_then(to_value),
        // Estudo com IA: o pacote chama outros comandos e o motor roda num job.
        ("assets", "study_state") => {
            #[derive(serde::Deserialize)]
            struct A {
                asset_id: String,
            }
            match parse_args::<A>(args.clone()) {
                Ok(a) => crate::assets::study::state(api, &a.asset_id).and_then(to_value),
                Err(e) => Err(e),
            }
        }
        ("assets", "study_package") => match parse_ref_args(args) {
            Ok(a) => crate::assets::study::preview(api, &a).and_then(to_value),
            Err(e) => Err(e),
        },
        // Notícias do ativo e texto dos documentos do IPE: rede fora da trava do banco
        // (docs/asset-overview.md).
        ("research", "news") => match parse_ref_args(args) {
            Ok(a) => crate::research::news::run(api, &a).and_then(to_value),
            Err(e) => Err(e),
        },
        ("research", "document_read") => match parse_ref_args(args) {
            Ok(a) => crate::research::doc_read::read(api, &a).and_then(to_value),
            Err(e) => Err(e),
        },
        ("assets", "study_start") => {
            user_only(&|| crate::assets::study::start(api, &parse_ref_args(args)?))
        }
        // brapi.dev: lê o plano e o consumo com o token (rede fora da trava).
        ("market", "brapi_test") => user_only(&|| {
            let token = api
                .with_session(|s, _| crate::research::brapi::token(s.db.conn()))?
                .ok_or_else(|| {
                    crate::error::Error::invalid(crate::i18n::t("nucleo-ativos-brapi-sem-token"))
                })?;
            let u = crate::research::brapi::fetch_usage(
                &crate::research::net::HttpFetcher::new(),
                &token,
            )
            .map_err(crate::error::Error::invalid)?;
            api.with_session(|s, _| {
                crate::research::brapi::store_usage(s.db.conn(), &u, &crate::time::now_utc())?;
                to_value(crate::market::api::brapi_view(s.db.conn())?)
            })
        }),
        ("assets", "enrich_lookup_all") => user_only(&|| {
            crate::assets::lookup::run_all(api, &parse_ref_args(args)?).and_then(to_value)
        }),
        // Proventos anunciados de ações e FIIs pelo Fundamentus (rede fora da trava).
        ("portfolio", "sync_fundamentus_income") => user_only(&|| {
            crate::research::fundamentus::sync_income(api, &parse_ref_args(args)?)
                .and_then(to_value)
        }),
        // Cotação de referência do câmbio (PTAX do dia pelo BCB): rede fora da trava.
        ("fx", "reference_rate") => crate::fx::exchange::reference_rate(
            api,
            &match parse_ref_args(args) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            },
        )
        .and_then(to_value),
        ("research", "sync") => user_only(&|| {
            crate::research::sync::run(api, &parse_ref_args(args)?).and_then(to_value)
        }),
        // Tabela FIPE: rede sem segurar o banco (a consulta grava uma avaliação).
        ("physical", "fipe_options") => user_only(&|| {
            api.with_session(|s, _| {
                crate::sources::ensure_enabled(s.db.conn(), crate::sources::SourceId::Fipe)
            })?;
            crate::physical::fipe::fetch_options(&parse_ref_args(args)?).and_then(to_value)
        }),
        ("physical", "refresh_fipe") => user_only(&|| {
            #[derive(serde::Deserialize)]
            struct IdArg {
                id: String,
            }
            let a: IdArg = parse_ref_args(args)?;
            crate::physical::fipe::refresh(api, &a.id).and_then(to_value)
        }),
        ("vault", _) | ("app", _) => Err(Error::UnknownCommand(format!("{module}.{action}"))),
        _ => return None,
    })
}

fn create(api: &Api, a: CreateVaultArgs) -> Result<VaultStatus> {
    let paths = api.paths();
    if paths.db_path.exists() {
        return Err(Error::VaultExists(paths.db_path.display().to_string()));
    }
    validate_master_password(&a.password)?;
    let household_name = a.household_name.trim();
    if household_name.is_empty() {
        return Err(Error::invalid(t("nucleo-base-erro-nome-familia")));
    }
    let country = a
        .country
        .as_deref()
        .unwrap_or("BR")
        .trim()
        .to_ascii_uppercase();
    paths.ensure_dirs()?;
    // Criado ao lado e movido no fim (`relocate`): uma criação interrompida (app fechado, queda
    // de energia) não deixa no lugar do cofre um arquivo pela metade.
    let creating = creating_path(&paths.db_path);
    for leftover in [
        creating.clone(),
        side_file(&creating, "-journal"),
        side_file(&creating, "-wal"),
    ] {
        if leftover.exists() {
            let _ = std::fs::remove_file(&leftover);
        }
    }
    let mut db = Db::create(&creating, &a.password, vault::default_kdf())?;
    let checked = if crate::country::choices().iter().any(|x| x.code == country) {
        crate::country::info(db.conn(), &country)
    } else {
        Err(Error::invalid(t_args(
            "nucleo-base-erro-pais",
            &crate::i18n::args(&[("pais", country.clone().into())]),
        )))
    };
    let country_info = match checked {
        Ok(i) => i,
        Err(e) => {
            drop(db);
            let _ = std::fs::remove_file(&creating);
            return Err(e);
        }
    };
    let base_currency = match crate::money::validate_currency(
        a.base_currency.as_deref().unwrap_or(&country_info.currency),
    ) {
        Ok(c) => c,
        Err(e) => {
            drop(db);
            let _ = std::fs::remove_file(&creating);
            return Err(e);
        }
    };
    let members: Vec<String> = a
        .members
        .iter()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect();
    let res = changes::record(
        db.conn_mut(),
        &Author::system(t("nucleo-base-autor-criacao")),
        &t("nucleo-base-motivo-criacao"),
        |tx| {
            crate::household::seed_in(tx, household_name, &base_currency, &country, &members)?;
            crate::categories::seed_defaults_for(tx, &country)?;
            crate::rules::builtin::seed_for(tx, &country)?;
            crate::country::apply_sources(tx, &country)?;
            Ok(())
        },
    )
    .and_then(|_| crate::rules::builtin::mark_seeded(db.conn()));
    if let Err(e) = res {
        drop(db);
        let _ = std::fs::remove_file(&creating);
        return Err(e);
    }
    let db = db.relocate(&paths.db_path)?;
    open_session(api, db)?;
    Ok(status(api))
}

/// Arquivo onde o cofre é criado antes de ir para o lugar (`treasury.db.criando`).
fn creating_path(db: &std::path::Path) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}.criando", db.display()))
}

fn side_file(p: &std::path::Path, ext: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}{ext}", p.display()))
}

fn open_session(api: &Api, mut db: Db) -> Result<()> {
    let schema_from = db.schema_version()?;
    if db.needs_migration()? {
        // Backup verificado antes de qualquer migração: voltar de versão é restaurá-lo.
        let rec = backup::create(&db, &api.paths().backups_dir, BackupKind::PreMigration)?;
        if !rec.verified {
            return Err(Error::Other(t_args(
                "nucleo-base-erro-backup-migracao",
                &crate::i18n::args(&[("erro", rec.verify_error.unwrap_or_default().into())]),
            )));
        }
        db.migrate()?;
    }
    // Versões do app que abriram o cofre (de onde ele veio numa atualização).
    if !db.is_read_only()
        && let Err(e) = crate::vault_versions::record(
            db.conn(),
            crate::config::app_version(),
            schema_from,
            db.schema_version()?,
        )
    {
        tracing::warn!("versões do cofre: {e}");
    }
    if let Err(e) = backup::sync_dir(db.conn(), &api.paths().backups_dir) {
        tracing::warn!("pasta de backups: {e}");
    }
    // Mantém o arquivo de chaves (partes de recuperação, chave física) com a chave atual.
    if let Err(e) = crate::unlock::refresh(db.conn(), db.key(), &api.paths().db_path) {
        tracing::warn!("arquivo de chaves do cofre: {e}");
    }
    let minutes: u64 = settings::get_or(
        db.conn(),
        "auto_lock_minutes",
        super::DEFAULT_AUTO_LOCK_MINUTES,
    )?;
    api.set_auto_lock_after(Duration::from_secs(minutes * 60));
    // Cofres de antes da V127: as categorias padrão ganham a chave de origem (uma vez).
    if !db.is_read_only()
        && let Err(e) = crate::categories::backfill_seed_keys(db.conn())
    {
        tracing::warn!("chave das categorias padrão: {e}");
    }
    // Cofres de antes da V129: as categorias padrão ganham o grupo de gasto da semente (uma vez,
    // depois da chave de origem).
    if !db.is_read_only()
        && let Err(e) = crate::categories::backfill_spend_groups(db.conn())
    {
        tracing::warn!("grupo das categorias padrão: {e}");
    }
    // Alimentação (V135): categorias-semente e produtos dos cupons antigos (uma vez).
    if !db.is_read_only()
        && let Err(e) = crate::food::on_open(db.conn())
    {
        tracing::warn!("produtos dos cupons: {e}");
    }
    // Rotinas ao desbloquear (ex.: gerar lançamentos previstos de recorrências).
    if let Err(e) = crate::recurrence::on_unlock(&mut db) {
        tracing::warn!("recorrências: {e}");
    }
    if !db.is_read_only()
        && let Err(e) = crate::rules::builtin::seed_once(db.conn_mut())
    {
        tracing::warn!("regras prontas: {e}");
    }
    *api.state() = State::Open(Box::new(Session {
        db,
        last_activity: Instant::now(),
        scratch: Scratch::default(),
        last_backup_at: Some(crate::time::now_utc()),
    }));
    Ok(())
}

fn unlock(api: &Api, open: impl FnOnce(&std::path::Path) -> Result<Db>) -> Result<VaultStatus> {
    if api.is_unlocked() {
        return Ok(status(api));
    }
    let path = api.paths().db_path.clone();
    if !path.exists() {
        return Err(Error::NoVault);
    }
    let db = open(&path)?;
    open_session(api, db)?;
    Ok(status(api))
}

fn unlock_recovery(api: &Api, a: RecoveryArgs) -> Result<VaultStatus> {
    let path = api.paths().db_path.clone();
    let salt = vault::read_salt(&path)?;
    let key = DbKey::from_recovery_code(&a.code, salt).map_err(|_| Error::WrongRecoveryCode)?;
    recover_with_key(api, key, &a.new_password)
}

/// Recuperação (kit, partes ou chave física): abre com a chave obtida, faz backup e exige
/// uma senha mestra nova, porque quem recupera esqueceu a antiga.
fn recover_with_key(api: &Api, key: DbKey, new_password: &str) -> Result<VaultStatus> {
    validate_master_password(new_password)?;
    if api.is_unlocked() {
        return Err(Error::conflict(t("nucleo-base-erro-cofre-aberto")));
    }
    let path = api.paths().db_path.clone();
    let mut db = Db::open_with_key(&path, key, false).map_err(|e| match e {
        Error::WrongPassword => Error::WrongRecoveryCode,
        e => e,
    })?;
    backup::create(&db, &api.paths().backups_dir, BackupKind::PreRekey)?;
    db.change_password(new_password, vault::default_kdf())?;
    open_session(api, db)?;
    Ok(status(api))
}

/// Bloqueia: backup ao fechar (se houve mudanças e estiver ligado), fecha o banco e
/// zera as chaves da memória.
pub(super) fn lock(api: &Api, backup_on_close: bool) -> Result<()> {
    // Nenhum job do agente (nem CLI filha) continua com o cofre bloqueado.
    api.ai_runtime().cancel_all();
    // "Permitir nesta conversa" vale só enquanto o cofre fica aberto.
    api.ai_runtime().forget_external_grants(None);
    let mut guard = api.state();
    if let State::Open(s) = &mut *guard
        && backup_on_close
    {
        let wants = backup::load_settings(s.db.conn())
            .map(|c| c.on_close)
            .unwrap_or(true);
        let changed =
            changes::count_since(s.db.conn(), s.last_backup_at.as_deref()).unwrap_or(1) > 0;
        if wants
            && changed
            && let Err(e) = backup::create(&s.db, &api.paths().backups_dir, BackupKind::OnClose)
        {
            tracing::warn!("backup ao fechar falhou: {e}");
        }
    }
    let old = std::mem::replace(&mut *guard, State::Closed);
    drop(guard);
    if let State::Open(s) = old {
        let Session { db, .. } = *s;
        db.close()?;
    }
    Ok(())
}

fn change_password(api: &Api, a: ChangePasswordArgs) -> Result<VaultStatus> {
    validate_master_password(&a.new)?;
    let paths = api.paths().clone();
    api.with_session(|s, _| {
        // Confere a senha atual derivando com o salt do arquivo.
        let check = DbKey::derive(
            &a.current,
            s.db.key().salt(),
            kdf_for(s.db.key().kdf_version()),
        )?;
        if check.key_bytes() != s.db.key().key_bytes() {
            return Err(Error::WrongPassword);
        }
        backup::create(&s.db, &paths.backups_dir, BackupKind::PreRekey)?;
        s.db.change_password(&a.new, vault::default_kdf())?;
        crate::unlock::refresh(s.db.conn(), s.db.key(), &paths.db_path)?;
        crate::unlock::password_changed(s.db.conn())?;
        if keychain::has_key(&paths.db_path).unwrap_or(false) {
            keychain::store_key(&paths.db_path, s.db.key())?;
        }
        Ok(())
    })?;
    Ok(status(api))
}

#[derive(Deserialize)]
struct PinArg {
    pin: vault::Secret,
}

/// Custo do Argon2id do PIN: o mesmo da senha mestra (no build de testes, o rápido).
fn pin_kdf() -> vault::KdfParams {
    #[cfg(any(test, feature = "fast-kdf"))]
    {
        vault::KDF_TEST
    }
    #[cfg(not(any(test, feature = "fast-kdf")))]
    {
        vault::KDF_V1
    }
}

fn kdf_for(version: u8) -> vault::KdfParams {
    vault::kdf_versions()
        .into_iter()
        .find(|k| k.version == version)
        .unwrap_or_else(vault::default_kdf)
}

fn verify_password(s: &Session, password: &str) -> Result<()> {
    let check = DbKey::derive(
        password,
        s.db.key().salt(),
        kdf_for(s.db.key().kdf_version()),
    )?;
    if check.key_bytes() != s.db.key().key_bytes() {
        return Err(Error::WrongPassword);
    }
    Ok(())
}

fn recovery_kit(api: &Api, password: &str) -> Result<RecoveryKit> {
    api.with_session(|s, paths| {
        verify_password(s, password)?;
        let created_at = crate::unlock::kit_shown(s.db.conn())?;
        Ok(RecoveryKit {
            code: s.db.key().recovery_code(),
            created_at,
            db_path: paths.db_path.to_string_lossy().into_owned(),
        })
    })
}

fn keychain_enable(api: &Api, password: &str) -> Result<()> {
    if !keychain::is_available() {
        return Err(Error::invalid(t("nucleo-geral-erro-keychain-indisponivel")));
    }
    api.with_session(|s, paths| {
        verify_password(s, password)?;
        keychain::store_key(&paths.db_path, s.db.key())
    })
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct RestoreArgs {
    pub id: String,
    /// Senha mestra atual: o banco restaurado é recifrado com ela.
    #[ts(type = "string")]
    pub master_password: vault::Secret,
    /// Senha do backup, se for diferente da mestra (senha de backup própria ou antiga).
    #[serde(default)]
    #[ts(optional, type = "string")]
    pub backup_password: Option<vault::Secret>,
}

fn restore(api: &Api, a: RestoreArgs) -> Result<VaultStatus> {
    let paths = api.paths().clone();
    let staged = api.with_session(|s, _| {
        verify_password(s, &a.master_password)?;
        let rec = backup::get(s.db.conn(), &a.id)?;
        let bpw = a.backup_password.as_deref().unwrap_or(&a.master_password);
        let staged = backup::prepare_restore(
            std::path::Path::new(&rec.path),
            bpw,
            &a.master_password,
            &paths.db_path,
        )?;
        // Backup do estado atual antes de trocar.
        backup::create(&s.db, &paths.backups_dir, BackupKind::PreRestore)?;
        Ok(staged)
    })?;
    lock(api, false)?;
    backup::swap_in(&staged, &paths.db_path)?;
    unlock(api, |p| Db::open(p, &a.master_password))
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct RestoreFileArgs {
    /// Arquivo da lista de `backup.scan`.
    pub path: String,
    /// Senha mestra: vale para o cofre restaurado.
    pub master_password: String,
    /// Senha do backup, se for outra (a senha antiga do cofre ou a de backup própria).
    #[serde(default)]
    #[ts(optional)]
    pub backup_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RestoreFileResult {
    pub status: VaultStatus,
    /// Onde ficou o cofre que não abria (nunca apagado), quando havia um.
    pub moved_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExportFileArgs {
    path: String,
    dest: String,
}

#[derive(Debug, Deserialize)]
struct ImportFileArgs {
    source: String,
}

/// Restaura um arquivo da pasta de backups. Com o cofre aberto, é a restauração de sempre
/// (confere a senha e faz o backup `pre_restore`); fechado, o cofre que não abre vai para o
/// lado (nunca é apagado) e o backup entra no lugar.
fn restore_file(api: &Api, a: RestoreFileArgs) -> Result<RestoreFileResult> {
    let paths = api.paths().clone();
    let file = backup::ensure_in_dir(&paths.backups_dir, std::path::Path::new(&a.path))?;
    let bpw = a.backup_password.as_deref().unwrap_or(&a.master_password);
    if api.is_unlocked() {
        let staged = api.with_session(|s, _| {
            verify_password(s, &a.master_password)?;
            let staged = backup::prepare_restore(&file, bpw, &a.master_password, &paths.db_path)?;
            backup::create(&s.db, &paths.backups_dir, BackupKind::PreRestore)?;
            Ok(staged)
        })?;
        lock(api, false)?;
        backup::swap_in(&staged, &paths.db_path)?;
        let status = unlock(api, |p| Db::open(p, &a.master_password))?;
        return Ok(RestoreFileResult {
            status,
            moved_to: None,
        });
    }
    validate_master_password(&a.master_password)?;
    let staged = backup::prepare_restore(&file, bpw, &a.master_password, &paths.db_path)?;
    let moved_to = if paths.db_path.exists() {
        Some(backup::move_aside(&paths.db_path).inspect_err(|_| {
            let _ = std::fs::remove_file(&staged);
        })?)
    } else {
        None
    };
    if let Err(e) = backup::swap_in(&staged, &paths.db_path) {
        // Devolve o cofre ao lugar antes de desistir.
        if let Some(m) = &moved_to {
            let _ = std::fs::rename(m, &paths.db_path);
        }
        return Err(e);
    }
    let status = unlock(api, |p| Db::open(p, &a.master_password))?;
    Ok(RestoreFileResult {
        status,
        moved_to: moved_to.map(|p| p.to_string_lossy().into_owned()),
    })
}

// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct BackupIdArgs {
    id: String,
    #[serde(default)]
    backup_password: Option<vault::Secret>,
}

pub(super) fn handle_backup(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    if *ctx.actor != Actor::User && !matches!(action, "list" | "health") {
        return Err(Error::Forbidden(t("nucleo-base-erro-backups-interface")));
    }
    match action {
        "list" => to_value(backup::list(ctx.conn())?),
        "health" => to_value(backup::health(ctx.conn(), chrono::Local::now())?),
        "create" => to_value(backup::create(
            ctx.db,
            &ctx.paths.backups_dir,
            BackupKind::Manual,
        )?),
        "verify" => {
            let a: BackupIdArgs = parse_args(args)?;
            to_value(backup::reverify(
                ctx.db,
                &a.id,
                a.backup_password.as_deref().map(String::as_str),
            )?)
        }
        "get_settings" => to_value(backup::load_settings(ctx.conn())?),
        "set_settings" => {
            let s: BackupSettings = parse_args(args)?;
            backup::save_settings(ctx.conn(), &s)?;
            to_value(backup::load_settings(ctx.conn())?)
        }
        "set_password" => {
            let a: PasswordArg = parse_args(args)?;
            backup::set_separate_password(ctx.conn(), &a.password)?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("backup.{action}"))),
    }
}

/// Ajustes gerais guardados no banco.
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct AppSettings {
    #[ts(type = "number")]
    pub auto_lock_minutes: u64,
    /// Visão padrão de orçamento e relatórios: competência (data da compra) ou caixa.
    pub budget_basis: crate::budget::Basis,
    /// Visão padrão do diagrama de fluxo (Sankey): caixa por padrão, porque resgates e aportes
    /// são dinheiro que se mexe; em caixa a fatura do cartão entra no mês do vencimento, aberta
    /// nas categorias das compras.
    #[serde(default = "cash")]
    pub flow_basis: crate::budget::Basis,
    /// Manter rotinas no menu bar/bandeja ao fechar a janela.
    pub close_to_tray: bool,
    pub locale: String,
    /// Opções de desenvolvedor (modo desenvolvedor): o feedback do agente ao desenvolvedor
    /// (ferramentas MCP `*_dev_feedback` e a tela Desenvolvedor).
    pub developer_tools: bool,
    /// Acompanhar a pasta Downloads: arquivo novo que o Treasury importa (extrato, nota de
    /// corretagem, relatório da B3) abre um aviso oferecendo a importação. Desligado por
    /// padrão; o app só olha a pasta com o cofre aberto.
    pub watch_downloads: bool,
    /// Depois de uma importação bem-sucedida de um arquivo oferecido pela pasta Downloads,
    /// mandar o arquivo para a Lixeira (desligado por padrão).
    pub downloads_delete_after_import: bool,
    /// Bloquear o cofre quando o computador dorme, a tela é bloqueada ou outra pessoa entra
    /// na sessão (ligado por padrão; a casca do app avisa o núcleo). docs/local-security.md,
    /// item 3.
    pub lock_on_sleep: bool,
    /// Deixar o Treasury aparecer em capturas e no compartilhamento de tela mesmo com o
    /// modo privacidade ligado (desligado por padrão: com ele, a janela some das capturas).
    /// docs/local-security.md, item 11.
    #[serde(default)]
    pub allow_screen_capture: bool,
    /// Rentabilidade da carteira pelo retorno diário encadeado em vez do mensal (Modified
    /// Dietz). Opção avançada, desligada por padrão.
    #[serde(default)]
    pub twr_daily: bool,
    /// Acompanhar a caixa de entrada de arquivos (`inbox_folder`, padrão `~/Treasury/Inbox`):
    /// o que chega é lido, guardado no cofre e sai da pasta (ligado por padrão).
    #[serde(default = "yes")]
    pub watch_inbox: bool,
    /// Pasta da caixa de entrada (vazio: `~/Treasury/Inbox`).
    #[serde(default)]
    pub inbox_folder: String,
}

fn yes() -> bool {
    true
}

fn cash() -> crate::budget::Basis {
    crate::budget::Basis::Cash
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_lock_minutes: super::DEFAULT_AUTO_LOCK_MINUTES,
            budget_basis: crate::budget::Basis::Accrual,
            flow_basis: crate::budget::Basis::Cash,
            close_to_tray: false,
            locale: "pt-BR".into(),
            developer_tools: false,
            watch_downloads: false,
            downloads_delete_after_import: false,
            lock_on_sleep: true,
            allow_screen_capture: false,
            twr_daily: false,
            watch_inbox: true,
            inbox_folder: String::new(),
        }
    }
}

/// Chave do ajuste da base do diagrama de fluxo (Sankey).
pub const FLOW_BASIS_SETTING: &str = "flow_basis";
/// Chave do ajuste "acompanhar a pasta Downloads" (lido também pela casca do app).
pub const WATCH_DOWNLOADS_SETTING: &str = "watch_downloads";
/// Chave do ajuste "apagar de Downloads depois de importar" (lido também pela casca).
pub const DOWNLOADS_DELETE_SETTING: &str = "downloads_delete_after_import";
/// Chave do ajuste "bloquear ao dormir ou bloquear a tela" (lido pela casca do app).
pub const LOCK_ON_SLEEP_SETTING: &str = "lock_on_sleep";
/// Chave do ajuste "permitir capturas de tela".
pub const ALLOW_SCREEN_CAPTURE_SETTING: &str = "allow_screen_capture";

pub fn load_app_settings(conn: &rusqlite::Connection) -> Result<AppSettings> {
    let mut s = AppSettings::default();
    s.auto_lock_minutes = settings::get_or(conn, "auto_lock_minutes", s.auto_lock_minutes)?;
    s.budget_basis = settings::get_or(conn, "budget_basis", s.budget_basis)?;
    s.flow_basis = settings::get_or(conn, FLOW_BASIS_SETTING, s.flow_basis)?;
    s.close_to_tray = settings::get_or(conn, "close_to_tray", s.close_to_tray)?;
    s.developer_tools = settings::get_or(conn, "developer_tools", s.developer_tools)?;
    s.locale = settings::get_or(conn, "locale", s.locale)?;
    s.watch_downloads = settings::get_or(conn, WATCH_DOWNLOADS_SETTING, s.watch_downloads)?;
    s.downloads_delete_after_import = settings::get_or(
        conn,
        DOWNLOADS_DELETE_SETTING,
        s.downloads_delete_after_import,
    )?;
    s.lock_on_sleep = settings::get_or(conn, LOCK_ON_SLEEP_SETTING, s.lock_on_sleep)?;
    s.allow_screen_capture =
        settings::get_or(conn, ALLOW_SCREEN_CAPTURE_SETTING, s.allow_screen_capture)?;
    s.twr_daily = settings::get_or(
        conn,
        crate::portfolio::engine::DAILY_TWR_SETTING,
        s.twr_daily,
    )?;
    s.watch_inbox = settings::get_or(conn, crate::import::inbox::WATCH_SETTING, s.watch_inbox)?;
    s.inbox_folder = settings::get_or(conn, crate::import::inbox::FOLDER_SETTING, s.inbox_folder)?;
    Ok(s)
}

pub(super) fn handle_settings(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "get" => to_value(load_app_settings(ctx.conn())?),
        "set" => {
            if *ctx.actor != Actor::User {
                return Err(Error::Forbidden(t("nucleo-base-erro-ajustes-interface")));
            }
            let patch: serde_json::Map<String, Value> = parse_args(args)?;
            let mut cur = serde_json::to_value(load_app_settings(ctx.conn())?)?;
            for (k, v) in patch {
                if cur.get(&k).is_none() {
                    return Err(Error::invalid(t_args(
                        "nucleo-base-erro-ajuste-desconhecido",
                        &crate::i18n::args(&[("ajuste", k.clone().into())]),
                    )));
                }
                cur[&k] = v;
            }
            let new: AppSettings =
                serde_json::from_value(cur).map_err(|e| Error::invalid(e.to_string()))?;
            if !(0..=24 * 60).contains(&new.auto_lock_minutes) {
                return Err(Error::invalid(t("nucleo-base-erro-bloqueio-automatico")));
            }
            settings::set(ctx.conn(), "auto_lock_minutes", &new.auto_lock_minutes)?;
            settings::set(ctx.conn(), "budget_basis", &new.budget_basis)?;
            settings::set(ctx.conn(), FLOW_BASIS_SETTING, &new.flow_basis)?;
            settings::set(ctx.conn(), "close_to_tray", &new.close_to_tray)?;
            settings::set(ctx.conn(), "developer_tools", &new.developer_tools)?;
            // O loopback sem chave do MCP HTTP é uma opção de desenvolvedor: sai junto.
            if !new.developer_tools {
                settings::set(
                    ctx.conn(),
                    crate::mcp::http_server::LOOPBACK_NO_AUTH_SETTING,
                    &false,
                )?;
            }
            settings::set(ctx.conn(), "locale", &new.locale)?;
            settings::set(ctx.conn(), WATCH_DOWNLOADS_SETTING, &new.watch_downloads)?;
            settings::set(
                ctx.conn(),
                crate::import::inbox::WATCH_SETTING,
                &new.watch_inbox,
            )?;
            settings::set(
                ctx.conn(),
                crate::import::inbox::FOLDER_SETTING,
                &new.inbox_folder.trim(),
            )?;
            settings::set(
                ctx.conn(),
                DOWNLOADS_DELETE_SETTING,
                &new.downloads_delete_after_import,
            )?;
            settings::set(ctx.conn(), LOCK_ON_SLEEP_SETTING, &new.lock_on_sleep)?;
            settings::set(
                ctx.conn(),
                ALLOW_SCREEN_CAPTURE_SETTING,
                &new.allow_screen_capture,
            )?;
            settings::set(
                ctx.conn(),
                crate::portfolio::engine::DAILY_TWR_SETTING,
                &new.twr_daily,
            )?;
            to_value(new)
        }
        _ => Err(Error::UnknownCommand(format!("settings.{action}"))),
    }
}
