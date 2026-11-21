//! `config.toml`: só o mínimo para iniciar (caminho do banco e dos backups, idioma).
//! Nunca guarda segredos; o restante das configurações fica dentro do banco cifrado.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

static APP_VERSION: OnceLock<&'static str> = OnceLock::new();

/// Versão do app que usa o núcleo (a do binário do Tauri, que a informa ao iniciar). O
/// núcleo tem a versão dele fixa no Cargo.toml: trocar a versão do app a cada release não
/// invalida a compilação do núcleo.
pub fn app_version() -> &'static str {
    APP_VERSION
        .get()
        .copied()
        .unwrap_or(env!("CARGO_PKG_VERSION"))
}

/// Informa a versão do app (uma vez, ao iniciar).
pub fn set_app_version(version: &'static str) {
    let _ = APP_VERSION.set(version);
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub db_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backups_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Cofres conhecidos (docs/vaults.md). Vazio: só o principal, em `db_path` ou no padrão.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vaults: Vec<VaultEntry>,
    /// Cofre aberto por padrão (id de `vaults`; vazio: o principal).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_vault: Option<String>,
    /// Atualização automática (docs/auto-update.md). Fica aqui, fora do cofre, porque a
    /// checagem roda com o cofre fechado. Sem os campos: baixa sozinho. `auto_update` é o
    /// ajuste antigo (ligado/desligado), lido só quando não há `update_mode`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_mode: Option<UpdateMode>,
}

/// O que o app faz sozinho com uma versão nova. Instalar sempre espera a pessoa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum UpdateMode {
    /// Procura ao abrir e a cada 6 h, baixa e confere a assinatura, e avisa que está pronta.
    Download,
    /// Procura no mesmo ritmo e só avisa que há versão nova; baixa quando a pessoa pedir.
    Notify,
    /// Não procura sozinho: só com "Procurar agora".
    Off,
}

/// Ajustes da atualização automática (Ajustes › Atualizações).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateSettings {
    pub mode: UpdateMode,
}

/// Lê os ajustes de atualização do `config.toml`.
pub fn update_settings(config_path: &Path) -> UpdateSettings {
    let c = Config::load(config_path).ok();
    let mode = match c {
        Some(Config {
            update_mode: Some(m),
            ..
        }) => m,
        Some(Config {
            auto_update: Some(false),
            ..
        }) => UpdateMode::Off,
        _ => UpdateMode::Download,
    };
    UpdateSettings { mode }
}

/// Grava os ajustes de atualização no `config.toml` (e tira o ajuste antigo).
pub fn set_update_settings(config_path: &Path, a: &UpdateSettings) -> Result<UpdateSettings> {
    let mut c = Config::load(config_path)?;
    c.update_mode = Some(a.mode);
    c.auto_update = None;
    c.save(config_path)?;
    Ok(update_settings(config_path))
}

/// Um cofre: um arquivo .db cifrado, com senha, desbloqueio rápido e backups próprios.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct VaultEntry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    /// Para que serve (ex.: "Empresa", "Pais"), em texto livre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_at: Option<String>,
    /// Cofre de demonstração que abre sem senha, pela chave fixa do app (docs/demo.md): só
    /// dados fictícios, nunca dados reais.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub demo: bool,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).map_err(|e| {
                Error::invalid(crate::i18n::t_args(
                    "nucleo-geral-erro-config",
                    &crate::i18n::args(&[("erro", e.to_string().into())]),
                ))
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let s = toml::to_string_pretty(self).map_err(|e| Error::Other(e.to_string()))?;
        if let Some(dir) = path.parent() {
            crate::paths::create_private_dir(dir)?;
        }
        std::fs::write(path, s)?;
        crate::paths::set_mode(path, 0o600)
    }
}
