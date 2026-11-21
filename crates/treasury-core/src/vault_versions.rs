//! Versões do app que abriram o cofre (ajuste `vault.versions`, fora do histórico de
//! alterações): a primeira e a última, a mais nova que já abriu, e cada mudança de versão do
//! app ou do esquema, com a data. Serve para saber de que versão um cofre veio numa
//! atualização e para avisar quando um app mais velho abre um cofre que um mais novo já usou
//! (o esquema é o mesmo, mas o app novo pode ter gravado ajustes que o velho não conhece).

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;

pub const SETTING: &str = "vault.versions";
/// Mudanças guardadas (as mais antigas saem primeiro).
const MAX_HISTORY: usize = 200;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VersionStep {
    pub at: String,
    pub app_version: String,
    #[ts(type = "number")]
    pub schema_from: i32,
    #[ts(type = "number")]
    pub schema_to: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VaultVersions {
    /// Primeira versão do app que gravou este registro (cofres de antes dele: a versão que o
    /// abriu depois da atualização).
    pub first_app_version: String,
    pub last_app_version: String,
    /// A versão mais nova que já abriu o cofre.
    pub newest_app_version: String,
    pub last_opened_at: String,
    pub history: Vec<VersionStep>,
}

/// Compara versões `MAIOR.MENOR.CORREÇÃO` (o que vem depois de `-` ou `+` não conta).
pub fn newer(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> {
        v.split(['-', '+'])
            .next()
            .unwrap_or("")
            .split('.')
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    parts(a) > parts(b)
}

pub fn get(conn: &Connection) -> Result<Option<VaultVersions>> {
    crate::settings::get(conn, SETTING)
}

/// Registra a abertura pela versão `app` do app, com o esquema antes e depois das migrações.
/// Uma linha nova no histórico só quando a versão do app ou o esquema mudou.
pub fn record(
    conn: &Connection,
    app: &str,
    schema_from: i32,
    schema_to: i32,
) -> Result<VaultVersions> {
    let now = crate::time::now_utc();
    let mut v = get(conn)?.unwrap_or_else(|| VaultVersions {
        first_app_version: app.to_string(),
        last_app_version: String::new(),
        newest_app_version: app.to_string(),
        ..Default::default()
    });
    if v.last_app_version != app || schema_from != schema_to {
        v.history.push(VersionStep {
            at: now.clone(),
            app_version: app.to_string(),
            schema_from,
            schema_to,
        });
        let extra = v.history.len().saturating_sub(MAX_HISTORY);
        v.history.drain(..extra);
    }
    if newer(app, &v.newest_app_version) {
        v.newest_app_version = app.to_string();
    }
    v.last_app_version = app.to_string();
    v.last_opened_at = now;
    crate::settings::set(conn, SETTING, &v)?;
    Ok(v)
}

/// A versão mais nova que já abriu o cofre, quando é mais nova que `app`.
pub fn opened_by_newer(conn: &Connection, app: &str) -> Option<String> {
    let v = get(conn).ok().flatten()?;
    newer(&v.newest_app_version, app).then_some(v.newest_app_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_by_number() {
        assert!(newer("0.10.0", "0.9.9"));
        assert!(newer("1.0.0", "0.99.1"));
        assert!(!newer("0.8.9", "0.8.9"));
        assert!(!newer("0.8.9-dev", "0.8.9"));
        assert!(newer("0.8.10", "0.8.9+abc"));
    }
}
