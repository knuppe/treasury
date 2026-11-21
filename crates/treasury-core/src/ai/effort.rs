//! Esforço de raciocínio escolhido na conversa ("Esforço: Padrão / Baixo / Médio / Alto /
//! Máximo"), traduzido para o que cada motor entende. Onde o modelo não tem esse controle,
//! nada é enviado (a interface mostra a opção desativada, ver `EngineCaps::effort`).
//!
//! - Anthropic: raciocínio estendido (`thinking.budget_tokens`): Baixo sem raciocínio,
//!   Médio 4 mil, Alto 16 mil, Máximo 32 mil tokens.
//! - OpenAI (o1/o3/o4, gpt-5): `reasoning_effort` low/medium/high (Máximo = high).
//! - xAI (grok-3-mini): `reasoning_effort` low/high.
//! - OpenRouter: `reasoning.effort` (o OpenRouter repassa a quem aceita).
//! - Gemini 2.5+: `thinkingConfig.thinkingBudget` 1.024 / 8.192 / 24.576 / 32.768 (Flash
//!   até 24.576).
//! - CLIs: Claude Code `--effort`, Grok `--reasoning-effort`, Codex
//!   `-c model_reasoning_effort=…`. Gemini CLI e modelos locais: sem controle.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ts_rs::TS;

use super::engine::Provider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Effort {
    /// O padrão do motor e do modelo (nada é enviado).
    #[default]
    Default,
    Low,
    Medium,
    High,
    Max,
}

impl Effort {
    pub fn label(self) -> &'static str {
        match self {
            Effort::Default => "padrão",
            Effort::Low => "baixo",
            Effort::Medium => "médio",
            Effort::High => "alto",
            Effort::Max => "máximo",
        }
    }

    /// `low`/`medium`/`high` (Máximo vira `high`, o teto da maioria das APIs).
    fn lmh(self) -> Option<&'static str> {
        match self {
            Effort::Default => None,
            Effort::Low => Some("low"),
            Effort::Medium => Some("medium"),
            Effort::High | Effort::Max => Some("high"),
        }
    }
}

fn lower(model: &str) -> String {
    model.to_ascii_lowercase()
}

/// Modelos Claude com raciocínio estendido (3.7 em diante; os 3.0/3.5 não têm).
fn anthropic_thinks(model: &str) -> bool {
    let m = lower(model);
    !(m.starts_with("claude-3-") && !m.starts_with("claude-3-7")) && !m.starts_with("claude-2")
}

fn openai_reasons(model: &str) -> bool {
    let m = lower(model);
    let m = m.rsplit('/').next().unwrap_or(&m);
    ["o1", "o3", "o4", "gpt-5"].iter().any(|p| m.starts_with(p))
}

fn xai_reasons(model: &str) -> bool {
    lower(model).contains("grok-3-mini")
}

fn gemini_thinks(model: &str) -> bool {
    let m = lower(model);
    !(m.contains("gemini-1") || m.contains("gemini-2.0"))
}

/// O motor tem controle de esforço com este modelo?
pub fn supported(provider: Provider, model: &str) -> bool {
    match provider {
        Provider::Anthropic => anthropic_thinks(model),
        Provider::Openai => openai_reasons(model),
        Provider::Xai => xai_reasons(model),
        Provider::Openrouter => true,
        Provider::Gemini => gemini_thinks(model),
        Provider::ClaudeCode | Provider::GrokCli | Provider::Codex => true,
        _ => false,
    }
}

/// Orçamento do raciocínio estendido da Anthropic (`None` = sem raciocínio estendido).
pub fn anthropic_budget(effort: Effort, model: &str) -> Option<u32> {
    if !anthropic_thinks(model) {
        return None;
    }
    match effort {
        Effort::Default | Effort::Low => None,
        Effort::Medium => Some(4_096),
        Effort::High => Some(16_384),
        Effort::Max => Some(32_000),
    }
}

/// Campos extras do corpo nas APIs compatíveis com a OpenAI.
pub fn openai_fields(
    provider: Provider,
    model: &str,
    effort: Effort,
) -> Option<(&'static str, Value)> {
    let v = effort.lmh()?;
    match provider {
        Provider::Openai if openai_reasons(model) => Some(("reasoning_effort", json!(v))),
        Provider::Xai if xai_reasons(model) => Some((
            "reasoning_effort",
            json!(if v == "low" { "low" } else { "high" }),
        )),
        Provider::Openrouter => Some(("reasoning", json!({ "effort": v }))),
        // Groq: os GPT-OSS raciocinam com `reasoning_effort` (low, medium, high).
        Provider::Groq if lower(model).contains("gpt-oss") => Some(("reasoning_effort", json!(v))),
        _ => None,
    }
}

/// `thinkingBudget` do Gemini.
pub fn gemini_budget(model: &str, effort: Effort) -> Option<i64> {
    if !gemini_thinks(model) {
        return None;
    }
    let cap = if lower(model).contains("flash") {
        24_576
    } else {
        32_768
    };
    let b = match effort {
        Effort::Default => return None,
        Effort::Low => 1_024,
        Effort::Medium => 8_192,
        Effort::High => 24_576,
        Effort::Max => 32_768,
    };
    Some(b.min(cap))
}

/// Argumentos das CLIs (acrescentados depois de `cli::restrict_argv`).
pub fn cli_args(provider: Provider, effort: Effort) -> Vec<String> {
    if effort == Effort::Default {
        return vec![];
    }
    match provider {
        Provider::ClaudeCode => vec![
            "--effort".into(),
            match effort {
                Effort::Low => "low",
                Effort::Medium => "medium",
                Effort::High => "high",
                _ => "max",
            }
            .into(),
        ],
        Provider::GrokCli => vec![
            "--reasoning-effort".into(),
            effort.lmh().unwrap_or("medium").into(),
        ],
        Provider::Codex => vec![
            "-c".into(),
            format!(
                "model_reasoning_effort={}",
                effort.lmh().unwrap_or("medium")
            ),
        ],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_effort_per_engine() {
        assert!(supported(Provider::Anthropic, "claude-sonnet-4-5"));
        assert!(!supported(Provider::Anthropic, "claude-3-5-haiku-latest"));
        assert_eq!(
            anthropic_budget(Effort::High, "claude-opus-5-5"),
            Some(16_384)
        );
        assert_eq!(anthropic_budget(Effort::Low, "claude-opus-5-5"), None);
        assert_eq!(
            openai_fields(Provider::Openai, "gpt-5-mini", Effort::Max),
            Some(("reasoning_effort", json!("high")))
        );
        assert_eq!(
            openai_fields(Provider::Openai, "gpt-4o", Effort::High),
            None
        );
        assert_eq!(
            openai_fields(Provider::Xai, "grok-3-mini", Effort::Medium),
            Some(("reasoning_effort", json!("high")))
        );
        assert_eq!(openai_fields(Provider::Xai, "grok-4", Effort::High), None);
        assert_eq!(
            openai_fields(Provider::Openrouter, "x/y", Effort::Low),
            Some(("reasoning", json!({"effort": "low"})))
        );
        assert_eq!(gemini_budget("gemini-2.5-flash", Effort::Max), Some(24_576));
        assert_eq!(gemini_budget("gemini-2.5-pro", Effort::Max), Some(32_768));
        assert_eq!(gemini_budget("gemini-2.0-flash", Effort::High), None);
        assert_eq!(
            cli_args(Provider::ClaudeCode, Effort::Max),
            vec!["--effort", "max"]
        );
        assert_eq!(
            cli_args(Provider::GrokCli, Effort::Low),
            vec!["--reasoning-effort", "low"]
        );
        assert_eq!(
            cli_args(Provider::Codex, Effort::High),
            vec!["-c", "model_reasoning_effort=high"]
        );
        assert!(cli_args(Provider::GeminiCli, Effort::High).is_empty());
        assert!(cli_args(Provider::ClaudeCode, Effort::Default).is_empty());
        assert!(!supported(Provider::Ollama, "qwen3"));
    }
}
