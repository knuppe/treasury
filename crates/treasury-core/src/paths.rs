//! Caminhos em disco. Padrão `~/.treasury`, igual em macOS e Linux.
//! `TREASURY_HOME` sobrescreve a raiz (testes, desenvolvimento).

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct TreasuryPaths {
    pub root: PathBuf,
    /// Cofre ativo (docs/vaults.md): id na lista do `config.toml` (`principal` sem lista).
    pub vault_id: String,
    pub db_path: PathBuf,
    pub backups_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub run_dir: PathBuf,
    pub config_path: PathBuf,
}

impl TreasuryPaths {
    /// Resolve a raiz (`TREASURY_HOME` ou `~/.treasury`) e aplica o `config.toml`.
    pub fn resolve() -> Result<Self> {
        let root = match std::env::var_os("TREASURY_HOME") {
            Some(p) if !p.is_empty() => PathBuf::from(p),
            _ => dirs::home_dir()
                .ok_or_else(|| crate::error::Error::Other("pasta pessoal não encontrada".into()))?
                .join(".treasury"),
        };
        Self::with_root(root)
    }

    pub fn with_root(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let config_path = root.join("config.toml");
        let config = Config::load(&config_path)?;
        let vault = crate::vaults::active(&config, &root);
        let backups_dir = crate::vaults::backups_dir(&config, &root, &vault);
        Ok(Self {
            logs_dir: root.join("logs"),
            run_dir: root.join("run"),
            vault_id: vault.id,
            db_path: vault.path,
            root,
            backups_dir,
            config_path,
        })
    }

    pub fn data_dir(&self) -> &Path {
        self.db_path.parent().unwrap_or(&self.root)
    }

    /// Cria as pastas com permissão 0700.
    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [&self.root, &self.backups_dir, &self.logs_dir, &self.run_dir] {
            create_private_dir(dir)?;
        }
        create_private_dir(self.data_dir())?;
        Ok(())
    }

    pub fn lock_path(&self) -> PathBuf {
        self.run_dir.join("treasury.lock")
    }
}

pub fn create_private_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    set_mode(dir, 0o700)
}

#[cfg(unix)]
pub fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
pub fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}
