//! Comandos `mcp.*` do despachante: auditoria (tela Agente) e configuração para clientes
//! externos (Ajustes › IA).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};
use ts_rs::TS;

use super::SERVER_NAME;
use super::audit::{self, AuditListArgs};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};

pub const READ_ONLY: &[&str] = &["audit_list", "client_config"];

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct McpClientConfigArgs {
    /// Caminho do executável do Treasury (padrão: o executável em execução).
    #[serde(default)]
    #[ts(optional)]
    pub command: Option<String>,
}

/// Ajustes do acesso MCP externo (Ajustes › Integrações MCP).
#[derive(Debug, Clone, serde::Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpServerSettings {
    /// Recusar clientes externos sem chave de API.
    pub require_api_key: bool,
    /// Servidor MCP HTTP local (127.0.0.1) para agentes que não usam stdio.
    pub http_enabled: bool,
    /// HTTP: aceitar pedidos sem chave vindos do próprio computador (loopback), fora de
    /// navegadores. Desligado por padrão; só vale (e só liga) no modo desenvolvedor.
    #[serde(default)]
    pub http_loopback_no_auth: bool,
    #[ts(type = "number")]
    pub http_port: u16,
    /// Clientes sem chave: ferramentas de escrita (propostas).
    pub allow_writes: bool,
    /// Cofre de antes do padrão novo: a chave passou a ser exigida (aviso até dispensar).
    #[serde(default)]
    pub require_key_notice: bool,
    /// `treasury mcp` com o app fechado: abre o cofre em modo leitura pelo keychain, e só
    /// para um cliente com chave de API válida. Desligado por padrão (docs/local-security.md,
    /// item 2): sem ele, qualquer processo do usuário rodaria o binário e leria o cofre.
    #[serde(default)]
    pub cli_when_closed: bool,
    /// Clientes externos que conectaram pela primeira vez (aviso até dispensar).
    #[serde(default)]
    pub new_clients: Vec<super::clients::McpClientNotice>,
}

pub const HTTP_ENABLED_SETTING: &str = "mcp.http_enabled";
pub const CLI_WHEN_CLOSED_SETTING: &str = "mcp.cli_when_closed";
pub const HTTP_PORT_SETTING: &str = "mcp.http_port";
pub const DEFAULT_HTTP_PORT: u16 = 7655;

pub fn server_settings(conn: &rusqlite::Connection) -> Result<McpServerSettings> {
    use crate::settings::get_or;
    Ok(McpServerSettings {
        require_api_key: super::keys::require_key(conn)?,
        require_key_notice: get_or(conn, super::keys::REQUIRE_KEY_NOTICE_SETTING, false)?,
        cli_when_closed: get_or(conn, CLI_WHEN_CLOSED_SETTING, false)?,
        http_enabled: get_or(conn, HTTP_ENABLED_SETTING, false)?,
        http_loopback_no_auth: super::http_server::loopback_no_auth(conn)?,
        http_port: get_or(conn, HTTP_PORT_SETTING, DEFAULT_HTTP_PORT)?,
        allow_writes: get_or(conn, super::ALLOW_WRITES_SETTING, false)?,
        new_clients: super::clients::notices(conn)?,
    })
}

fn user_only(ctx: &Ctx<'_>) -> Result<()> {
    if matches!(ctx.actor, Actor::User) {
        Ok(())
    } else {
        Err(Error::Forbidden(crate::i18n::t("mcp-so-interface")))
    }
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "keys_list" => {
            user_only(ctx)?;
            if !ctx.db.is_read_only() {
                super::keys::purge_revoked(ctx.conn())?;
            }
            to_value(super::keys::list(ctx.conn())?)
        }
        // Só chaves revogadas; a auditoria das chamadas feitas com ela continua.
        "key_delete" => {
            user_only(ctx)?;
            #[derive(Deserialize)]
            struct A {
                id: String,
            }
            let a: A = parse_args(args)?;
            super::keys::delete(ctx.conn(), &a.id)?;
            to_value(super::keys::list(ctx.conn())?)
        }
        "key_create" => {
            user_only(ctx)?;
            let a: super::keys::McpKeyCreateArgs = parse_args(args)?;
            to_value(super::keys::create(ctx.conn(), &a)?)
        }
        "key_revoke" => {
            user_only(ctx)?;
            #[derive(Deserialize)]
            struct A {
                id: String,
            }
            let a: A = parse_args(args)?;
            super::keys::revoke(ctx.conn(), &a.id)?;
            to_value(super::keys::list(ctx.conn())?)
        }
        "server_settings" => {
            user_only(ctx)?;
            to_value(server_settings(ctx.conn())?)
        }
        "dismiss_require_key_notice" => {
            user_only(ctx)?;
            crate::settings::set(ctx.conn(), super::keys::REQUIRE_KEY_NOTICE_SETTING, &false)?;
            to_value(server_settings(ctx.conn())?)
        }
        "dismiss_client_notices" => {
            user_only(ctx)?;
            super::clients::dismiss(ctx.conn())?;
            to_value(server_settings(ctx.conn())?)
        }
        "set_server_settings" => {
            user_only(ctx)?;
            let a: McpServerSettings = parse_args(args)?;
            if a.http_port < 1024 {
                return Err(Error::invalid(crate::i18n::t("mcp-porta-baixa")));
            }
            let c = ctx.conn();
            crate::settings::set(c, super::keys::REQUIRE_KEY_SETTING, &a.require_api_key)?;
            crate::settings::set(c, HTTP_ENABLED_SETTING, &a.http_enabled)?;
            // Fora do modo desenvolvedor, o loopback sem chave não liga.
            crate::settings::set(
                c,
                super::http_server::LOOPBACK_NO_AUTH_SETTING,
                &(a.http_loopback_no_auth && crate::devfeedback::developer_mode(c)?),
            )?;
            crate::settings::set(c, HTTP_PORT_SETTING, &a.http_port)?;
            crate::settings::set(c, super::ALLOW_WRITES_SETTING, &a.allow_writes)?;
            crate::settings::set(c, CLI_WHEN_CLOSED_SETTING, &a.cli_when_closed)?;
            to_value(server_settings(c)?)
        }
        // audit_list AuditListArgs -> Vec<AuditEntry> (mais recentes primeiro).
        "audit_list" => {
            let a: AuditListArgs = parse_args(args)?;
            if !ctx.db.is_read_only() {
                audit::import_pending(ctx.conn(), &ctx.paths.run_dir);
            }
            to_value(audit::list(ctx.conn(), &a)?)
        }
        // client_config McpClientConfigArgs -> {"mcpServers": {"treasury": {...}}}
        "client_config" => {
            let a: McpClientConfigArgs = parse_args(args)?;
            let command = match a.command.as_deref().map(str::trim) {
                Some(c) if !c.is_empty() => PathBuf::from(c),
                _ => std::env::current_exe()?,
            };
            let home = (!is_default_root(&ctx.paths.root)).then(|| ctx.paths.root.clone());
            Ok(client_config(&command, home.as_deref()))
        }
        _ => Err(Error::UnknownCommand(format!("mcp.{action}"))),
    }
}

/// Configuração MCP para Claude Desktop/Claude Code e outros clientes:
/// `{"mcpServers":{"treasury":{"command":"<binário>","args":["mcp"]}}}`. Com uma raiz de
/// dados fora do padrão, inclui `env.TREASURY_HOME`.
pub fn client_config(command: &Path, home: Option<&Path>) -> Value {
    let mut server = json!({
        "command": command.to_string_lossy(),
        "args": ["mcp"],
    });
    if let Some(h) = home {
        server["env"] = json!({ "TREASURY_HOME": h.to_string_lossy() });
    }
    json!({ "mcpServers": { SERVER_NAME: server } })
}

fn is_default_root(root: &Path) -> bool {
    dirs::home_dir().is_some_and(|h| h.join(".treasury") == root)
}
