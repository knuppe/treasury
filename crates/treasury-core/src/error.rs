//! Erros do núcleo. Toda mensagem voltada ao usuário é em pt-BR; o `code` é estável
//! e pode ser usado pela interface e por clientes MCP.

use serde::Serialize;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("o cofre não existe; crie um cofre primeiro")]
    NoVault,
    #[error("o cofre já existe em {0}")]
    VaultExists(String),
    #[error("o cofre está bloqueado")]
    Locked,
    #[error("senha incorreta")]
    WrongPassword,
    /// A senha não abre o cofre, mas abre o backup mais novo: o cofre pode estar danificado
    /// (ou a senha mudou depois do backup). A interface oferece a recuperação por backup.
    #[error("{0}")]
    VaultDamaged(String),
    #[error("código de recuperação incorreto")]
    WrongRecoveryCode,
    #[error("{0}")]
    WeakPassword(String),
    #[error("o Treasury já está aberto em outro processo")]
    AlreadyOpen,
    #[error("{0}")]
    Invalid(String),
    #[error("não encontrado: {0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("comando desconhecido: {0}")]
    UnknownCommand(String),
    #[error("ainda não implementado: {0}")]
    NotImplemented(String),
    #[error("permissão negada: {0}")]
    Forbidden(String),
    /// Nenhum motor de IA na rota da tarefa (a interface leva o usuário aos ajustes).
    #[error("{0}")]
    NoEngine(String),
    /// PIN errado (a mensagem diz quantas tentativas restam).
    #[error("{0}")]
    WrongPin(String),
    /// O banco foi aberto por uma versão mais nova do Treasury (migrações desconhecidas).
    #[error("{0}")]
    SchemaTooNew(String),
    #[error("erro de banco de dados: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("erro de migração: {0}")]
    Migration(String),
    #[error("erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("dados inválidos: {0}")]
    Json(#[from] serde_json::Error),
    #[error("erro de criptografia: {0}")]
    Crypto(String),
    #[error("{0}")]
    Other(String),
    /// Tarefa em segundo plano cancelada a pedido (docs/background-tasks.md); o que já
    /// tinha sido gravado fica.
    #[error("cancelado")]
    Cancelled,
    /// A escrita virou uma proposta aguardando aprovação do usuário (agente de IA, MCP).
    #[error("alteração proposta, aguardando aprovação do usuário")]
    Proposed {
        proposal_id: String,
        change_set_id: String,
    },
}

impl Error {
    pub fn invalid(msg: impl Into<String>) -> Self {
        Error::Invalid(msg.into())
    }
    pub fn not_found(what: impl Into<String>) -> Self {
        Error::NotFound(what.into())
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Error::Conflict(msg.into())
    }

    /// Código estável para a interface e para o MCP.
    pub fn code(&self) -> &'static str {
        match self {
            Error::NoVault => "no_vault",
            Error::VaultExists(_) => "vault_exists",
            Error::Locked => "vault_locked",
            Error::WrongPassword => "wrong_password",
            Error::VaultDamaged(_) => "vault_damaged",
            Error::WrongRecoveryCode => "wrong_recovery_code",
            Error::WeakPassword(_) => "weak_password",
            Error::AlreadyOpen => "already_open",
            Error::Invalid(_) | Error::Json(_) => "invalid",
            Error::NotFound(_) => "not_found",
            Error::Conflict(_) => "conflict",
            Error::UnknownCommand(_) => "unknown_command",
            Error::NotImplemented(_) => "not_implemented",
            Error::Forbidden(_) => "forbidden",
            Error::NoEngine(_) => "no_engine",
            Error::SchemaTooNew(_) => "schema_too_new",
            Error::WrongPin(_) => "wrong_pin",
            Error::Db(rusqlite::Error::QueryReturnedNoRows) => "not_found",
            Error::Db(_) => "db",
            Error::Migration(_) => "migration",
            Error::Io(_) => "io",
            Error::Crypto(_) => "crypto",
            Error::Other(_) => "other",
            Error::Cancelled => "cancelled",
            Error::Proposed { .. } => "pending_approval",
        }
    }

    /// Mensagem para quem usa (interface, agente), no idioma da instalação: os erros fixos vêm
    /// do catálogo (`locales/<idioma>/erros.ftl`, chave `erro-<código>`); o detalhe e os textos
    /// livres passam como estão. O `Display` continua em pt-BR, para os logs.
    pub fn message(&self) -> String {
        use crate::i18n::{args, t, t_args};
        let with = |key: &str, name: &'static str, value: String| {
            t_args(key, &args(&[(name, value.into())]))
        };
        let detail = |key: &str, d: String| with(key, "detalhe", d);
        match self {
            Error::NoVault => t("erro-no_vault"),
            Error::VaultExists(p) => with("erro-vault_exists", "caminho", p.clone()),
            Error::Locked => t("erro-vault_locked"),
            Error::WrongPassword => t("erro-wrong_password"),
            Error::WrongRecoveryCode => t("erro-wrong_recovery_code"),
            Error::AlreadyOpen => t("erro-already_open"),
            Error::NotFound(d) => detail("erro-not_found", d.clone()),
            Error::UnknownCommand(d) => detail("erro-unknown_command", d.clone()),
            Error::NotImplemented(d) => detail("erro-not_implemented", d.clone()),
            Error::Forbidden(d) => detail("erro-forbidden", d.clone()),
            Error::Db(e) => detail("erro-db", e.to_string()),
            Error::Migration(d) => detail("erro-migration", d.clone()),
            Error::Io(e) => detail("erro-io", e.to_string()),
            Error::Json(e) => detail("erro-json", e.to_string()),
            Error::Crypto(d) => detail("erro-crypto", d.clone()),
            Error::Cancelled => t("erro-cancelled"),
            Error::Proposed { .. } => t("erro-pending_approval"),
            Error::WeakPassword(m)
            | Error::Invalid(m)
            | Error::Conflict(m)
            | Error::NoEngine(m)
            | Error::WrongPin(m)
            | Error::SchemaTooNew(m)
            | Error::VaultDamaged(m)
            | Error::Other(m) => m.clone(),
        }
    }

    pub fn to_api(&self) -> ApiError {
        ApiError {
            code: self.code().to_string(),
            message: self.message(),
            details: None,
        }
    }
}

/// Forma serializada de um erro, devolvida pela API (Tauri, servidor de desenvolvimento).
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[ts(optional, type = "unknown")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        e.to_api()
    }
}
