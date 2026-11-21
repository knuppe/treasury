//! Motores de IA configurados (Ajustes › IA): APIs de providers, CLIs locais e modelos
//! locais. Ver docs/ai-agent.md §Providers e motores de IA.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::attachments::{EngineCaps, caps_for};
use super::secret::{self, SecretStatus};
use crate::error::{Error, Result};
use crate::i18n;
use crate::ids::{Id, new_id};
use crate::time::now_utc;
use crate::vault::DbKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EngineKind {
    /// API de um provider (custo por token).
    Api,
    /// CLI local já autenticada (usa a assinatura do usuário).
    Cli,
    /// Modelo local (Ollama, LM Studio): sem internet, custo zero.
    Local,
}

impl EngineKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EngineKind::Api => "api",
            EngineKind::Cli => "cli",
            EngineKind::Local => "local",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Provider {
    Anthropic,
    Openai,
    Gemini,
    Xai,
    Mistral,
    Openrouter,
    /// Groq (API compatível com a da OpenAI), com plano gratuito: modo enxuto (`lean`).
    Groq,
    /// Qualquer endpoint compatível com a API de chat da OpenAI.
    OpenaiCompatible,
    ClaudeCode,
    Codex,
    GeminiCli,
    GrokCli,
    /// OpenCode (opencode.ai): qualquer provedor que o usuário configurou nele.
    OpencodeCli,
    CustomCli,
    Ollama,
    LmStudio,
}

/// Formato da conversa com o motor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    AnthropicMessages,
    OpenaiChat,
    GeminiGenerate,
    /// CLI com saída `stream-json` (Claude Code).
    CliStreamJson,
    /// CLI com saída em texto.
    CliText,
    /// OpenCode com `--format json` (um evento por linha).
    CliOpencodeJson,
}

pub const ALL_PROVIDERS: &[Provider] = &[
    Provider::Anthropic,
    Provider::Openai,
    Provider::Gemini,
    Provider::Xai,
    Provider::Mistral,
    Provider::Openrouter,
    Provider::Groq,
    Provider::OpenaiCompatible,
    Provider::ClaudeCode,
    Provider::Codex,
    Provider::GeminiCli,
    Provider::GrokCli,
    Provider::OpencodeCli,
    Provider::CustomCli,
    Provider::Ollama,
    Provider::LmStudio,
];

/// Claude Code: só as ferramentas do Treasury (MCP estrito), sem as nativas (`--tools ""`:
/// nada de arquivos, shell ou web). O app reforça isso em `cli::restrict_argv` mesmo se o
/// template for editado.
pub const CLAUDE_CODE_TEMPLATE: &str = "claude -p {prompt} [--model {model}] --mcp-config {mcp_config} --strict-mcp-config --tools \"\" --allowedTools \"mcp__treasury__*\" --output-format stream-json --verbose";
/// Codex: sandbox somente leitura, MCP do Treasury por `-c` (TOML em linha).
pub const CODEX_TEMPLATE: &str = "codex exec --sandbox read-only --skip-git-repo-check --cd {workdir} [--model {model}] -c mcp_servers.treasury={mcp_config} {prompt}";
/// Gemini CLI: configuração MCP em `{workdir}/.gemini/settings.json` (ferramentas nativas
/// excluídas) e só o servidor `treasury` liberado.
pub const GEMINI_CLI_TEMPLATE: &str =
    "gemini [-m {model}] --allowed-mcp-server-names treasury -p {prompt}";
/// Resposta máxima no modo enxuto (o limite de tokens por minuto conta a resposta pedida).
pub const LEAN_MAX_TOKENS: u32 = 1536;

/// CLI do Grok: o MCP do Treasury entra pelo `.grok/config.toml` da pasta de trabalho, e o
/// Treasury acrescenta `--trust`, as ferramentas liberadas e as negadas (`cli::restrict_argv`).
pub const GROK_CLI_TEMPLATE: &str = "grok [--model {model}] -p {prompt}";

/// OpenCode: servidor próprio da execução (`--standalone`), para valer a configuração que o
/// Treasury passa em `OPENCODE_CONFIG_CONTENT` (só o MCP do Treasury, ferramentas nativas
/// negadas; `cli::opencode_config`). O modelo vai como `provedor/modelo`.
pub const OPENCODE_CLI_TEMPLATE: &str =
    "opencode run --standalone --format json [--model {model}] {prompt}";

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::Openai => "openai",
            Provider::Gemini => "gemini",
            Provider::Xai => "xai",
            Provider::Mistral => "mistral",
            Provider::Openrouter => "openrouter",
            Provider::Groq => "groq",
            Provider::OpenaiCompatible => "openai_compatible",
            Provider::ClaudeCode => "claude_code",
            Provider::Codex => "codex",
            Provider::GeminiCli => "gemini_cli",
            Provider::GrokCli => "grok_cli",
            Provider::OpencodeCli => "opencode_cli",
            Provider::CustomCli => "custom_cli",
            Provider::Ollama => "ollama",
            Provider::LmStudio => "lm_studio",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        ALL_PROVIDERS
            .iter()
            .copied()
            .find(|p| p.as_str() == s)
            .ok_or_else(|| {
                Error::invalid(i18n::t_args(
                    "nucleo-ia-provider-desconhecido",
                    &i18n::args(&[("provider", s.to_string().into())]),
                ))
            })
    }

    pub fn kind(self) -> EngineKind {
        match self {
            Provider::Anthropic
            | Provider::Openai
            | Provider::Gemini
            | Provider::Xai
            | Provider::Mistral
            | Provider::Openrouter
            | Provider::Groq
            | Provider::OpenaiCompatible => EngineKind::Api,
            Provider::ClaudeCode
            | Provider::Codex
            | Provider::GeminiCli
            | Provider::GrokCli
            | Provider::OpencodeCli
            | Provider::CustomCli => EngineKind::Cli,
            Provider::Ollama | Provider::LmStudio => EngineKind::Local,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic (Claude)",
            Provider::Openai => "OpenAI",
            Provider::Gemini => "Google Gemini",
            Provider::Xai => "xAI (Grok)",
            Provider::Mistral => "Mistral",
            Provider::Openrouter => "OpenRouter",
            Provider::Groq => "Groq",
            Provider::OpenaiCompatible => "Endpoint compatível com OpenAI",
            Provider::ClaudeCode => "Claude Code (CLI)",
            Provider::Codex => "OpenAI Codex (CLI)",
            Provider::GeminiCli => "Gemini CLI",
            Provider::GrokCli => "Grok (CLI)",
            Provider::OpencodeCli => "OpenCode (CLI)",
            Provider::CustomCli => "CLI personalizada",
            Provider::Ollama => "Ollama",
            Provider::LmStudio => "LM Studio",
        }
    }

    pub fn protocol(self) -> Protocol {
        match self {
            Provider::Anthropic => Protocol::AnthropicMessages,
            Provider::Gemini => Protocol::GeminiGenerate,
            Provider::Openai
            | Provider::Xai
            | Provider::Mistral
            | Provider::Openrouter
            | Provider::Groq
            | Provider::OpenaiCompatible
            | Provider::Ollama
            | Provider::LmStudio => Protocol::OpenaiChat,
            Provider::ClaudeCode => Protocol::CliStreamJson,
            Provider::OpencodeCli => Protocol::CliOpencodeJson,
            Provider::Codex | Provider::GeminiCli | Provider::GrokCli | Provider::CustomCli => {
                Protocol::CliText
            }
        }
    }

    /// URL base padrão (APIs e modelos locais).
    pub fn default_endpoint(self) -> Option<&'static str> {
        Some(match self {
            Provider::Anthropic => "https://api.anthropic.com",
            Provider::Openai => "https://api.openai.com/v1",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Provider::Xai => "https://api.x.ai/v1",
            Provider::Mistral => "https://api.mistral.ai/v1",
            Provider::Openrouter => "https://openrouter.ai/api/v1",
            Provider::Groq => "https://api.groq.com/openai/v1",
            Provider::Ollama => "http://127.0.0.1:11434/v1",
            Provider::LmStudio => "http://127.0.0.1:1234/v1",
            _ => return None,
        })
    }

    /// Sugestão de modelo (o nome atual de cada modelo muda: o campo é editável).
    pub fn default_model(self) -> Option<&'static str> {
        Some(match self {
            Provider::Anthropic => "claude-sonnet-4-5",
            Provider::Openai => "gpt-5-mini",
            Provider::Gemini => "gemini-2.5-flash",
            Provider::Xai => "grok-4",
            Provider::Mistral => "mistral-large-latest",
            Provider::Openrouter => "anthropic/claude-sonnet-4.5",
            // Modelo de produção do Groq com chamada de ferramentas (também em paralelo) e o
            // maior limite de tokens por minuto do plano gratuito entre os que chamam
            // ferramentas bem (conferido em 20/10/2026; docs/ai-agent.md › Groq).
            Provider::Groq => "llama-3.3-70b-versatile",
            // Modelo local sugerido: o Qwen 3 4B acertou 79% das categorias num teste com 85
            // descrições de extrato (docs/ai-agent.md); precisa estar baixado no Ollama/LM Studio.
            Provider::Ollama => "qwen3:4b",
            Provider::LmStudio => "qwen/qwen3-4b",
            _ => return None,
        })
    }

    pub fn default_template(self) -> Option<&'static str> {
        Some(match self {
            Provider::ClaudeCode => CLAUDE_CODE_TEMPLATE,
            Provider::Codex => CODEX_TEMPLATE,
            Provider::GeminiCli => GEMINI_CLI_TEMPLATE,
            Provider::GrokCli => GROK_CLI_TEMPLATE,
            Provider::OpencodeCli => OPENCODE_CLI_TEMPLATE,
            _ => return None,
        })
    }

    /// Executável das CLIs conhecidas (detecção no PATH).
    pub fn binary(self) -> Option<&'static str> {
        Some(match self {
            Provider::ClaudeCode => "claude",
            Provider::Codex => "codex",
            Provider::GeminiCli => "gemini",
            Provider::GrokCli => "grok",
            Provider::OpencodeCli => "opencode",
            _ => return None,
        })
    }

    /// Exige API key (APIs remotas; o endpoint compatível e os locais podem dispensar).
    pub fn needs_key(self) -> bool {
        matches!(
            self,
            Provider::Anthropic
                | Provider::Openai
                | Provider::Gemini
                | Provider::Xai
                | Provider::Mistral
                | Provider::Openrouter
                | Provider::Groq
        )
    }

    /// Modo enxuto: o plano gratuito limita os tokens por minuto (cada pedido conta as
    /// instruções, as ferramentas e a resposta máxima), então o agente vai com instruções
    /// curtas, um conjunto pequeno de ferramentas e resposta menor, e espera o limite liberar
    /// em vez de desistir.
    pub fn lean(self) -> bool {
        matches!(self, Provider::Groq)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProviderInfo {
    pub provider: Provider,
    pub kind: EngineKind,
    pub label: String,
    pub default_endpoint: Option<String>,
    pub default_model: Option<String>,
    pub default_template: Option<String>,
    pub binary: Option<String>,
    pub needs_key: bool,
}

pub fn providers() -> Vec<ProviderInfo> {
    ALL_PROVIDERS
        .iter()
        .map(|&p| ProviderInfo {
            provider: p,
            kind: p.kind(),
            label: p.label().into(),
            default_endpoint: p.default_endpoint().map(Into::into),
            default_model: p.default_model().map(Into::into),
            default_template: p.default_template().map(Into::into),
            binary: p.binary().map(Into::into),
            needs_key: p.needs_key(),
        })
        .collect()
}

/// Motor como a interface vê (nunca com o valor da chave).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Engine {
    pub id: Id,
    pub name: String,
    pub kind: EngineKind,
    pub provider: Provider,
    pub provider_label: String,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub command_template: Option<String>,
    /// Endpoint/template em uso (o configurado ou o padrão do provider).
    pub effective_endpoint: Option<String>,
    pub effective_template: Option<String>,
    pub enabled: bool,
    #[ts(type = "number | null")]
    pub max_output_tokens: Option<i64>,
    /// Ajuste "lê imagens" do motor (None = automático, pelo provider e pelo modelo).
    pub vision: Option<bool>,
    /// O que o motor lê além de texto (efetivo: o ajuste ou o automático).
    pub caps: EngineCaps,
    /// API key: só "cadastrada" e os 4 últimos caracteres.
    pub secret: SecretStatus,
    pub needs_key: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Engine {
    /// Pronto para uso: ligado e, se exigir chave, com chave válida.
    pub fn ready(&self) -> std::result::Result<(), String> {
        if !self.enabled {
            return Err(i18n::t("nucleo-ia-motor-desligado"));
        }
        if self.needs_key && (!self.secret.set || self.secret.needs_reentry) {
            return Err(i18n::t("nucleo-ia-motor-sem-chave"));
        }
        if self.kind == EngineKind::Cli && self.effective_template.is_none() {
            return Err(i18n::t("nucleo-ia-motor-sem-template"));
        }
        if self.kind != EngineKind::Cli && self.model.as_deref().is_none_or(str::is_empty) {
            return Err(i18n::t("nucleo-ia-motor-sem-modelo"));
        }
        Ok(())
    }

    pub fn max_tokens(&self) -> u32 {
        self.max_output_tokens
            .filter(|n| *n > 0)
            .map(|n| n.min(64_000) as u32)
            .unwrap_or(if self.provider.lean() {
                LEAN_MAX_TOKENS
            } else {
                4096
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EngineInput {
    /// Padrão: o nome do provider.
    #[ts(optional)]
    pub name: Option<String>,
    pub provider: Provider,
    #[ts(optional)]
    pub model: Option<String>,
    #[ts(optional)]
    pub endpoint: Option<String>,
    #[ts(optional)]
    pub command_template: Option<String>,
    #[ts(optional)]
    pub enabled: Option<bool>,
    #[ts(optional, type = "number")]
    pub max_output_tokens: Option<i64>,
    /// Lê imagens? Ausente = automático.
    #[ts(optional)]
    pub vision: Option<bool>,
    /// API key (gravada cifrada; nunca devolvida).
    #[ts(optional)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EnginePatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub model: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub endpoint: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub command_template: Option<Option<String>>,
    #[ts(optional)]
    pub enabled: Option<bool>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "number | null")]
    pub max_output_tokens: Option<Option<i64>>,
    /// `null` volta ao automático.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "boolean | null")]
    pub vision: Option<Option<bool>>,
}

pub fn secret_id(engine_id: &str) -> String {
    format!("ai_engine:{engine_id}")
}

fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Endpoint: http(s) sem credenciais; http só para o próprio computador (a chave não pode
/// viajar em texto puro pela rede).
pub fn validate_endpoint(url: &str) -> Result<()> {
    let lower = url.to_ascii_lowercase();
    let rest = if let Some(r) = lower.strip_prefix("https://") {
        r
    } else if let Some(r) = lower.strip_prefix("http://") {
        let host = r.split(['/', '?', '#']).next().unwrap_or("");
        let host = host.rsplit_once(':').map_or(host, |(h, p)| {
            if p.chars().all(|c| c.is_ascii_digit()) {
                h
            } else {
                host
            }
        });
        if !matches!(host, "localhost" | "127.0.0.1" | "[::1]") {
            return Err(Error::invalid(i18n::t("nucleo-ia-endpoint-so-local")));
        }
        r
    } else {
        return Err(Error::invalid(i18n::t("nucleo-ia-endpoint-https")));
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains('@') || url.chars().any(char::is_whitespace) {
        return Err(Error::invalid(i18n::t("nucleo-ia-endpoint-invalido")));
    }
    Ok(())
}

fn validate_template(t: &str) -> Result<()> {
    let argv = super::cli::split_template(t)?;
    if argv.is_empty() {
        return Err(Error::invalid(i18n::t("nucleo-ia-template-vazio")));
    }
    if !t.contains("{prompt}") {
        return Err(Error::invalid(i18n::t("nucleo-ia-template-sem-prompt")));
    }
    Ok(())
}

fn row_to_engine(r: &rusqlite::Row<'_>) -> rusqlite::Result<(Engine, String)> {
    let provider: String = r.get("provider")?;
    let provider = Provider::parse(&provider).unwrap_or(Provider::OpenaiCompatible);
    let endpoint: Option<String> = r.get("endpoint")?;
    let template: Option<String> = r.get("command_template")?;
    let id: String = r.get("id")?;
    let model: Option<String> = r.get("model")?;
    let vision: Option<bool> = r.get::<_, Option<i64>>("vision")?.map(|v| v == 1);
    Ok((
        Engine {
            id: id.clone(),
            name: r.get("name")?,
            kind: provider.kind(),
            provider,
            provider_label: provider.label().into(),
            caps: caps_for(provider, model.as_deref(), vision),
            vision,
            model,
            effective_endpoint: endpoint
                .clone()
                .or_else(|| provider.default_endpoint().map(Into::into)),
            effective_template: template
                .clone()
                .or_else(|| provider.default_template().map(Into::into)),
            endpoint,
            command_template: template,
            enabled: r.get::<_, i64>("enabled")? == 1,
            max_output_tokens: r.get("max_output_tokens")?,
            secret: SecretStatus::default(),
            needs_key: provider.needs_key(),
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
        },
        id,
    ))
}

pub fn list(conn: &Connection, key: &DbKey) -> Result<Vec<Engine>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_engine ORDER BY created_at, id")?;
    let rows: Vec<(Engine, String)> = stmt
        .query_map([], row_to_engine)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(|(mut e, id)| {
            e.secret = secret::status(conn, key, &secret_id(&id))?;
            Ok(e)
        })
        .collect()
}

pub fn get(conn: &Connection, key: &DbKey, id: &str) -> Result<Engine> {
    let row = conn
        .query_row("SELECT * FROM ai_engine WHERE id = ?1", [id], row_to_engine)
        .optional()?
        .ok_or_else(|| {
            Error::not_found(i18n::t_args(
                "nucleo-ia-motor-id",
                &i18n::args(&[("id", id.to_string().into())]),
            ))
        })?;
    let (mut e, id) = row;
    e.secret = secret::status(conn, key, &secret_id(&id))?;
    Ok(e)
}

pub fn create(conn: &Connection, key: &DbKey, input: &EngineInput) -> Result<Engine> {
    let p = input.provider;
    let name = clean(input.name.clone()).unwrap_or_else(|| p.label().to_string());
    let model = clean(input.model.clone()).or_else(|| p.default_model().map(Into::into));
    let endpoint = clean(input.endpoint.clone());
    if let Some(e) = &endpoint {
        validate_endpoint(e)?;
    } else if p == Provider::OpenaiCompatible {
        return Err(Error::invalid(i18n::t("nucleo-ia-endpoint-obrigatorio")));
    }
    let template = clean(input.command_template.clone());
    if let Some(t) = &template {
        if p.kind() != EngineKind::Cli {
            return Err(Error::invalid(i18n::t("nucleo-ia-template-so-cli")));
        }
        validate_template(t)?;
    } else if p == Provider::CustomCli {
        return Err(Error::invalid(i18n::t("nucleo-ia-template-obrigatorio")));
    }
    if name.chars().count() > 80 {
        return Err(Error::invalid(i18n::t_args(
            "nucleo-ia-nome-longo",
            &i18n::args(&[("maximo", "80".into())]),
        )));
    }
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO ai_engine (id, name, kind, provider, model, endpoint, command_template, enabled, max_output_tokens, vision, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?11, ?10, ?10)",
        params![
            id,
            name,
            p.kind().as_str(),
            p.as_str(),
            model,
            endpoint,
            template,
            input.enabled.unwrap_or(true) as i64,
            input.max_output_tokens,
            now,
            input.vision.map(i64::from)
        ],
    )?;
    if let Some(k) = input.api_key.as_deref().filter(|k| !k.trim().is_empty()) {
        secret::set(conn, key, &secret_id(&id), k)?;
    }
    get(conn, key, &id)
}

pub fn update(conn: &Connection, key: &DbKey, p: &EnginePatch) -> Result<Engine> {
    let cur = get(conn, key, &p.id)?;
    let name = match &p.name {
        Some(n) if n.trim().is_empty() => {
            return Err(Error::invalid(i18n::t("nucleo-ia-nome-obrigatorio")));
        }
        Some(n) => n.trim().to_string(),
        None => cur.name.clone(),
    };
    let model = match &p.model {
        Some(m) => clean(m.clone()),
        None => cur.model.clone(),
    };
    let endpoint = match &p.endpoint {
        Some(e) => clean(e.clone()),
        None => cur.endpoint.clone(),
    };
    if let Some(e) = &endpoint {
        validate_endpoint(e)?;
    }
    let template = match &p.command_template {
        Some(t) => clean(t.clone()),
        None => cur.command_template.clone(),
    };
    if let Some(t) = &template {
        if cur.kind != EngineKind::Cli {
            return Err(Error::invalid(i18n::t("nucleo-ia-template-so-cli")));
        }
        validate_template(t)?;
    }
    let max = match p.max_output_tokens {
        Some(m) => m,
        None => cur.max_output_tokens,
    };
    let vision = match p.vision {
        Some(v) => v,
        None => cur.vision,
    };
    conn.execute(
        "UPDATE ai_engine SET name = ?2, model = ?3, endpoint = ?4, command_template = ?5, enabled = ?6,
           max_output_tokens = ?7, updated_at = ?8, vision = ?9 WHERE id = ?1",
        params![
            p.id,
            name,
            model,
            endpoint,
            template,
            p.enabled.unwrap_or(cur.enabled) as i64,
            max,
            now_utc(),
            vision.map(i64::from)
        ],
    )?;
    get(conn, key, &p.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    if conn.execute("DELETE FROM ai_engine WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(i18n::t("nucleo-ia-motor")));
    }
    secret::delete(conn, &secret_id(id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_need_https_outside_localhost() {
        assert!(validate_endpoint("https://api.anthropic.com").is_ok());
        assert!(validate_endpoint("http://127.0.0.1:11434/v1").is_ok());
        assert!(validate_endpoint("http://localhost:1234/v1").is_ok());
        assert!(validate_endpoint("http://api.exemplo.com/v1").is_err());
        assert!(validate_endpoint("https://user:pw@exemplo.com").is_err());
        assert!(validate_endpoint("ftp://x").is_err());
    }

    #[test]
    fn provider_catalog_is_consistent() {
        for p in ALL_PROVIDERS {
            assert_eq!(Provider::parse(p.as_str()).unwrap(), *p);
            let json = serde_json::to_value(p).unwrap();
            assert_eq!(json, p.as_str());
            if p.kind() == EngineKind::Cli && *p != Provider::CustomCli {
                assert!(p.default_template().unwrap().contains("{prompt}"));
                assert!(p.binary().is_some());
            }
        }
        // Claude Code: MCP estrito e só as ferramentas do Treasury.
        assert!(CLAUDE_CODE_TEMPLATE.contains("--strict-mcp-config"));
        assert!(CLAUDE_CODE_TEMPLATE.contains("mcp__treasury__*"));
    }
}
