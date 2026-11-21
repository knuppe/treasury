//! Chaves de API para clientes MCP externos: agentes fora do Treasury que consultam (ou
//! propõem mudanças na) carteira. Cada chave tem escopos:
//!
//! - `read`: ferramentas de consulta (sempre incluído);
//! - `write`: ferramentas de escrita, **sempre como proposta** na fila de aprovação.
//!
//! Só o SHA-256 da chave fica no banco; a chave aparece uma vez, ao ser criada. Chaves
//! revogadas ou vencidas param de valer na hora. Uma revogada continua na lista por
//! [`REVOKED_KEEP_DAYS`] dias (a pessoa pode excluir antes) e depois sai sozinha; a
//! auditoria das chamadas fica. Nenhum escopo aprova propostas, mexe em ajustes, chaves ou
//! cofre: isso é só da interface do app.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::time::now_utc;

pub const PREFIX: &str = "trsk_";
/// Ajuste antigo de recusar conexões MCP externas sem chave de API (o agente interno não
/// precisa). Desde 23/10/2026 a chave é sempre exigida e a tela não tem o interruptor: o
/// token do socket é legível por qualquer processo do mesmo usuário, então só a chave separa
/// um cliente conhecido (docs/local-security.md, item 1). O ajuste só vale nos testes do
/// núcleo, para exercitar o caminho sem chave.
pub const REQUIRE_KEY_SETTING: &str = "mcp.require_api_key";
pub const REQUIRE_KEY_DEFAULT: bool = true;
/// Aviso único, para cofres de antes do padrão novo (a V139 liga a chave neles), de que a
/// chave passou a ser exigida. Some quando a pessoa dispensa.
pub const REQUIRE_KEY_NOTICE_SETTING: &str = "mcp.require_api_key_notice";

/// Dias que uma chave revogada fica na lista antes de sair sozinha.
pub const REVOKED_KEEP_DAYS: i64 = 7;

/// A chave é exigida? Sempre, fora dos testes do núcleo.
pub fn require_key(conn: &rusqlite::Connection) -> Result<bool> {
    if cfg!(test) {
        crate::settings::get_or(conn, REQUIRE_KEY_SETTING, REQUIRE_KEY_DEFAULT)
    } else {
        Ok(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum McpScope {
    Read,
    Write,
}

/// O que uma conexão autenticada por chave pode usar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyGrant {
    pub id: String,
    pub name: String,
    pub scopes: Vec<McpScope>,
}

impl KeyGrant {
    pub fn has(&self, s: McpScope) -> bool {
        self.scopes.contains(&s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpApiKey {
    pub id: String,
    pub name: String,
    /// Início da chave (`trsk_ABCD…`), para reconhecer.
    pub prefix: String,
    pub scopes: Vec<McpScope>,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpKeyCreateArgs {
    pub name: String,
    pub scopes: Vec<McpScope>,
    /// Validade em dias (padrão: sem vencimento).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub expires_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpKeyCreated {
    /// A chave inteira: mostrada só agora.
    pub key: String,
    pub info: McpApiKey,
}

fn hash(key: &str) -> String {
    hex::encode(Sha256::digest(key.trim().as_bytes()))
}

fn normalize_scopes(mut s: Vec<McpScope>) -> Vec<McpScope> {
    if !s.contains(&McpScope::Read) {
        s.push(McpScope::Read);
    }
    let order = |x: &McpScope| match x {
        McpScope::Read => 0,
        McpScope::Write => 1,
    };
    s.sort_by_key(order);
    s.dedup();
    s
}

pub fn create(conn: &Connection, a: &McpKeyCreateArgs) -> Result<McpKeyCreated> {
    let name = a.name.trim();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(Error::invalid(crate::i18n::t("mcp-chave-nome")));
    }
    let mut raw = [0u8; 32];
    getrandom::fill(&mut raw).map_err(|e| Error::Crypto(e.to_string()))?;
    let key = format!(
        "{PREFIX}{}",
        base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &raw)
    );
    let id = crate::ids::new_id();
    let now = now_utc();
    let expires = a.expires_days.map(|d| {
        (chrono::Utc::now() + chrono::Duration::days(d as i64))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    });
    let scopes = normalize_scopes(a.scopes.clone());
    let prefix: String = key.chars().take(PREFIX.len() + 6).collect();
    conn.execute(
        "INSERT INTO mcp_api_key (id, name, prefix, key_hash, scopes, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, prefix, hash(&key), serde_json::to_string(&scopes)?, now, expires],
    )?;
    let info = get(conn, &id)?;
    Ok(McpKeyCreated { key, info })
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<McpApiKey> {
    let scopes: String = r.get("scopes")?;
    Ok(McpApiKey {
        id: r.get("id")?,
        name: r.get("name")?,
        prefix: r.get("prefix")?,
        scopes: parse_scopes(&scopes),
        created_at: r.get("created_at")?,
        expires_at: r.get("expires_at")?,
        last_used_at: r.get("last_used_at")?,
        revoked_at: r.get("revoked_at")?,
    })
}

/// Escopos gravados (JSON). Um escopo que não existe mais (ex.: `browser`, do navegador
/// removido na V108) é ignorado sem derrubar os outros.
fn parse_scopes(raw: &str) -> Vec<McpScope> {
    let list: Vec<serde_json::Value> = serde_json::from_str(raw).unwrap_or_default();
    normalize_scopes(
        list.into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
    )
}

pub fn get(conn: &Connection, id: &str) -> Result<McpApiKey> {
    conn.query_row("SELECT * FROM mcp_api_key WHERE id = ?1", [id], row)
        .optional()?
        .ok_or_else(|| Error::not_found(crate::i18n::t("mcp-chave-de-api")))
}

pub fn list(conn: &Connection) -> Result<Vec<McpApiKey>> {
    let mut stmt =
        conn.prepare("SELECT * FROM mcp_api_key ORDER BY revoked_at IS NOT NULL, created_at DESC")?;
    let rows = stmt.query_map([], row)?.collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Apaga as chaves revogadas há mais de [`REVOKED_KEEP_DAYS`] dias. Devolve quantas.
pub fn purge_revoked(conn: &Connection) -> Result<usize> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(REVOKED_KEEP_DAYS))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    Ok(conn.execute(
        "DELETE FROM mcp_api_key WHERE revoked_at IS NOT NULL AND revoked_at <= ?1",
        [cutoff],
    )?)
}

/// Exclui uma chave já revogada (uma ativa precisa ser revogada antes).
pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute(
        "DELETE FROM mcp_api_key WHERE id = ?1 AND revoked_at IS NOT NULL",
        [id],
    )?;
    if n == 0 {
        get(conn, id)?;
        return Err(Error::invalid(crate::i18n::t("mcp-chave-excluir-ativa")));
    }
    Ok(())
}

pub fn revoke(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute(
        "UPDATE mcp_api_key SET revoked_at = ?2 WHERE id = ?1 AND revoked_at IS NULL",
        params![id, now_utc()],
    )?;
    if n == 0 {
        get(conn, id)?;
    }
    Ok(())
}

/// Confere uma chave (comparação por hash; vencida ou revogada não vale). Marca o uso
/// quando o banco aceita escrita.
pub fn verify(conn: &Connection, key: &str, writable: bool) -> Result<Option<KeyGrant>> {
    if !key.trim().starts_with(PREFIX) {
        return Ok(None);
    }
    let found: Option<McpApiKey> = conn
        .query_row(
            "SELECT * FROM mcp_api_key WHERE key_hash = ?1",
            [hash(key)],
            row,
        )
        .optional()?;
    let Some(k) = found else { return Ok(None) };
    let now = now_utc();
    if k.revoked_at.is_some() || k.expires_at.as_deref().is_some_and(|e| e <= now.as_str()) {
        return Ok(None);
    }
    if writable {
        conn.execute(
            "UPDATE mcp_api_key SET last_used_at = ?2 WHERE id = ?1",
            params![k.id, now],
        )?;
    }
    Ok(Some(KeyGrant {
        id: k.id,
        name: k.name,
        scopes: k.scopes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn revoked_keys_can_be_deleted_and_leave_after_seven_days() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        let make = |name: &str| {
            create(
                c,
                &McpKeyCreateArgs {
                    name: name.into(),
                    scopes: vec![],
                    expires_days: None,
                },
            )
            .unwrap()
            .info
            .id
        };
        let (ativa, recente, antiga) = (make("Ativa"), make("Recente"), make("Antiga"));
        // Ativa não se exclui: precisa revogar antes.
        assert_eq!(delete(c, &ativa).unwrap_err().code(), "invalid");
        assert_eq!(delete(c, "nao-existe").unwrap_err().code(), "not_found");
        revoke(c, &recente).unwrap();
        revoke(c, &antiga).unwrap();
        c.execute(
            "UPDATE mcp_api_key SET revoked_at = '2000-01-01T00:00:00.000Z' WHERE id = ?1",
            [&antiga],
        )
        .unwrap();
        // Revogada há mais de 7 dias sai sozinha; a recente fica até alguém excluir.
        assert_eq!(purge_revoked(c).unwrap(), 1);
        let names: Vec<String> = list(c).unwrap().into_iter().map(|k| k.name).collect();
        assert_eq!(names, ["Ativa", "Recente"]);
        delete(c, &recente).unwrap();
        let names: Vec<String> = list(c).unwrap().into_iter().map(|k| k.name).collect();
        assert_eq!(names, ["Ativa"]);
    }

    #[test]
    fn create_verify_revoke_and_expire() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        let made = create(
            c,
            &McpKeyCreateArgs {
                name: "Agente da corretora".into(),
                scopes: vec![McpScope::Write],
                expires_days: None,
            },
        )
        .unwrap();
        assert!(made.key.starts_with("trsk_"));
        assert_eq!(made.info.scopes, vec![McpScope::Read, McpScope::Write]);
        let stored: String = c
            .query_row("SELECT key_hash FROM mcp_api_key", [], |r| r.get(0))
            .unwrap();
        assert_ne!(stored, made.key, "a chave não é guardada");
        let g = verify(c, &made.key, true).unwrap().unwrap();
        assert!(g.has(McpScope::Write));
        assert!(list(c).unwrap()[0].last_used_at.is_some());
        assert!(verify(c, "trsk_errada", true).unwrap().is_none());
        assert!(verify(c, "outra-coisa", true).unwrap().is_none());
        revoke(c, &made.info.id).unwrap();
        assert!(verify(c, &made.key, true).unwrap().is_none());
        // Vencida.
        let old = create(
            c,
            &McpKeyCreateArgs {
                name: "Temporária".into(),
                scopes: vec![],
                expires_days: Some(1),
            },
        )
        .unwrap();
        c.execute(
            "UPDATE mcp_api_key SET expires_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
            [&old.info.id],
        )
        .unwrap();
        assert!(verify(c, &old.key, true).unwrap().is_none());
        // Escopo que não existe mais (navegador, V108) não derruba os outros.
        assert_eq!(
            parse_scopes(r#"["read", "write", "browser"]"#),
            vec![McpScope::Read, McpScope::Write]
        );
        assert_eq!(parse_scopes("lixo"), vec![McpScope::Read]);
    }
}
