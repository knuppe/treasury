//! Partes do acesso local ao MCP que valem em qualquer sistema: nomes dos arquivos e das
//! variáveis, o aperto de mão do socket e a decisão pela chave de API. O socket em si
//! ([`super::socket`]) é Unix; no Windows o `treasury mcp` abre o cofre em modo leitura.

use serde::{Deserialize, Serialize};

pub const SOCKET_FILE: &str = "mcp.sock";
pub const TOKEN_FILE: &str = "mcp.token";
/// Variável com o caminho de um arquivo de token alternativo (token de agente de um job).
pub const TOKEN_FILE_ENV: &str = "TREASURY_MCP_TOKEN_FILE";

/// Primeira linha enviada pelo cliente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    pub token: String,
    #[serde(default)]
    pub client: Option<String>,
    /// Chave de API do cliente externo (escopos); obrigatória se o ajuste pedir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Variável com a chave de API que o `treasury mcp` repassa ao app.
pub const API_KEY_ENV: &str = "TREASURY_API_KEY";

/// Decide o acesso de um cliente externo pela chave (ou falta dela). `Err` = recusar.
pub(crate) fn key_access(
    api: &crate::SharedApi,
    api_key: Option<&str>,
) -> std::result::Result<Option<super::keys::KeyGrant>, String> {
    let key = api_key.map(str::trim).filter(|k| !k.is_empty());
    let r = api.with_session(|s, _| {
        let require = super::keys::require_key(s.db.conn())?;
        let grant = match key {
            Some(k) => super::keys::verify(s.db.conn(), k, !s.db.is_read_only())?,
            None => None,
        };
        Ok((require, grant))
    });
    match r {
        // Cofre bloqueado: com chave, a conexão entra e a chave é conferida a cada chamada
        // (`TreasuryMcp::refusal`), quando o cofre abrir; sem chave, recusa (a chave é sempre
        // exigida; só os testes do núcleo desligam).
        Err(_) if key.is_some() || cfg!(test) => Ok(None),
        Err(_) => Err(crate::i18n::t("mcp-chave-exigida")),
        Ok((_, Some(g))) => Ok(Some(g)),
        Ok((_, None)) if key.is_some() => Err(crate::i18n::t("mcp-chave-invalida")),
        Ok((true, None)) => Err(crate::i18n::t("mcp-chave-exigida")),
        Ok((false, None)) => Ok(None),
    }
}

/// Resposta do servidor quando o token confere. Com `ok: false`, `error` diz por que a chave
/// de API foi recusada, para o `treasury mcp` repassar ao cliente (token errado continua
/// fechando a conexão sem resposta).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeReply {
    pub ok: bool,
    pub server: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
