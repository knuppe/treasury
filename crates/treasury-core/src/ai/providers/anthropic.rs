//! Anthropic Messages API com tool use, em streaming.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::*;

pub const API_VERSION: &str = "2023-06-01";

fn content(m: &Msg) -> Vec<Value> {
    m.blocks
        .iter()
        .filter_map(|b| match b {
            // Sem assinatura a API recusa o bloco: só volta o que veio assinado.
            Block::Thinking { signature, .. } if signature.is_empty() => None,
            other => Some(other),
        })
        .map(|b| match b {
            Block::Text(t) => json!({"type": "text", "text": t}),
            Block::Thinking { text, signature } => {
                json!({"type": "thinking", "thinking": text, "signature": signature})
            }
            Block::ToolUse { id, name, input } => {
                json!({"type": "tool_use", "id": id, "name": name, "input": input})
            }
            Block::ToolResult {
                id,
                content,
                is_error,
                ..
            } => json!({"type": "tool_result", "tool_use_id": id, "content": content, "is_error": is_error}),
            Block::Image { mime, data, .. } => json!({
                "type": "image",
                "source": {"type": "base64", "media_type": mime, "data": attachments::b64(data)},
            }),
            Block::Document { mime, name, data, .. } => json!({
                "type": "document",
                "source": {"type": "base64", "media_type": mime, "data": attachments::b64(data)},
                "title": name,
            }),
            Block::Attachment(a) => json!({"type": "text", "text": unresolved(a)}),
        })
        .collect()
}

/// Cache de prompt: um ponto no fim das instruções (guarda ferramentas + instruções, iguais
/// em todo turno) e outro no último bloco da conversa (o próximo passo do loop de
/// ferramentas e o próximo turno leem tudo até ali do cache, a 10% do preço).
const CACHE: &str = "ephemeral";

pub fn body(t: &Turn<'_>) -> Value {
    let mut messages: Vec<Value> = t
        .messages
        .iter()
        .map(|m| {
            json!({
                "role": if m.role == Role::User { "user" } else { "assistant" },
                "content": content(m),
            })
        })
        .collect();
    if let Some(last) = messages
        .last_mut()
        .and_then(|m| m["content"].as_array_mut())
        .and_then(|c| c.iter_mut().rev().find(|b| b["type"] != "thinking"))
    {
        last["cache_control"] = json!({"type": CACHE});
    }
    let mut b = json!({
        "model": t.model,
        "max_tokens": t.max_tokens,
        "system": [{"type": "text", "text": t.system, "cache_control": {"type": CACHE}}],
        "messages": messages,
        "stream": true,
    });
    if let Some(budget) = crate::ai::effort::anthropic_budget(t.effort, t.model) {
        // O orçamento do raciocínio conta dentro de `max_tokens`: sobra espaço para a resposta.
        b["thinking"] = json!({"type": "enabled", "budget_tokens": budget});
        b["max_tokens"] = json!(t.max_tokens.max(budget + 4_096));
    }
    if !t.tools.is_empty() {
        b["tools"] = Value::Array(
            t.tools
                .iter()
                .map(|d| json!({"name": d.name, "description": d.description, "input_schema": d.schema}))
                .collect(),
        );
    }
    b
}

enum Partial {
    Text(String),
    Thinking {
        text: String,
        signature: String,
    },
    Tool {
        id: String,
        name: String,
        json: String,
    },
}

pub fn turn(
    ep: &Endpoint<'_>,
    t: &Turn<'_>,
    on_delta: &mut dyn FnMut(Delta<'_>),
) -> Result<TurnOutput, ProviderError> {
    let url = if ep.base_url.trim_end_matches('/').ends_with("/v1") {
        join_url(ep.base_url, "messages")
    } else {
        join_url(ep.base_url, "v1/messages")
    };
    let headers = vec![
        ("x-api-key", ep.api_key.unwrap_or_default().to_string()),
        ("anthropic-version", API_VERSION.to_string()),
    ];
    let mut parts: BTreeMap<u64, Partial> = BTreeMap::new();
    let mut out = TurnOutput::default();
    let mut stream_error: Option<ProviderError> = None;
    post_lines(ep, &url, &headers, &body(t), &mut |line| {
        let Some(data) = sse_data(line) else {
            return Ok(());
        };
        let v = parse_json(data)?;
        match v["type"].as_str().unwrap_or("") {
            "message_start" => {
                let u = &v["message"]["usage"];
                let cached = u["cache_read_input_tokens"].as_i64().unwrap_or(0);
                let written = u["cache_creation_input_tokens"].as_i64().unwrap_or(0);
                out.tokens.input = u["input_tokens"].as_i64().unwrap_or(0) + cached + written;
                out.tokens.cached = cached;
                out.tokens.cache_write = written;
                out.tokens.output = u["output_tokens"].as_i64().unwrap_or(0);
                out.usage_reported = true;
            }
            "content_block_start" => {
                let i = v["index"].as_u64().unwrap_or(0);
                let cb = &v["content_block"];
                let p = match cb["type"].as_str() {
                    Some("tool_use") => Partial::Tool {
                        id: cb["id"].as_str().unwrap_or("").to_string(),
                        name: cb["name"].as_str().unwrap_or("").to_string(),
                        json: String::new(),
                    },
                    Some("thinking") => {
                        let t = cb["thinking"].as_str().unwrap_or("").to_string();
                        if !t.is_empty() {
                            on_delta(Delta::Reasoning(&t));
                        }
                        Partial::Thinking {
                            text: t,
                            signature: cb["signature"].as_str().unwrap_or("").to_string(),
                        }
                    }
                    _ => {
                        let t = cb["text"].as_str().unwrap_or("").to_string();
                        if !t.is_empty() {
                            on_delta(Delta::Text(&t));
                        }
                        Partial::Text(t)
                    }
                };
                parts.insert(i, p);
            }
            "content_block_delta" => {
                let i = v["index"].as_u64().unwrap_or(0);
                let d = &v["delta"];
                match (parts.get_mut(&i), d["type"].as_str()) {
                    (Some(Partial::Text(s)), Some("text_delta")) => {
                        let t = d["text"].as_str().unwrap_or("");
                        s.push_str(t);
                        on_delta(Delta::Text(t));
                    }
                    (Some(Partial::Thinking { text, .. }), Some("thinking_delta")) => {
                        let t = d["thinking"].as_str().unwrap_or("");
                        text.push_str(t);
                        on_delta(Delta::Reasoning(t));
                    }
                    (Some(Partial::Thinking { signature, .. }), Some("signature_delta")) => {
                        signature.push_str(d["signature"].as_str().unwrap_or(""));
                    }
                    (Some(Partial::Tool { json, .. }), Some("input_json_delta")) => {
                        json.push_str(d["partial_json"].as_str().unwrap_or(""));
                    }
                    _ => {}
                }
            }
            "message_delta" => {
                if let Some(o) = v["usage"]["output_tokens"].as_i64() {
                    out.tokens.output = o;
                }
            }
            "error" => {
                let msg = v["error"]["message"].as_str().map_or_else(
                    || crate::i18n::t("nucleo-ia-provider-streaming"),
                    str::to_string,
                );
                let kind = match v["error"]["type"].as_str() {
                    Some("overloaded_error") | Some("api_error") => ErrKind::Server,
                    Some("rate_limit_error") => ErrKind::RateLimit,
                    Some("authentication_error") | Some("permission_error") => ErrKind::Auth,
                    _ => ErrKind::Invalid,
                };
                stream_error = Some(ProviderError::new(kind, format!("Anthropic: {msg}")));
            }
            _ => {}
        }
        Ok(())
    })?;
    if let Some(e) = stream_error {
        return Err(e);
    }
    for (_, p) in parts {
        match p {
            Partial::Text(s) if !s.is_empty() => out.blocks.push(Block::Text(s)),
            Partial::Text(_) => {}
            Partial::Thinking { text, signature } => {
                out.blocks.push(Block::Thinking { text, signature })
            }
            Partial::Tool { id, name, json } => {
                let input = if json.trim().is_empty() {
                    json!({})
                } else {
                    parse_json(&json)?
                };
                out.blocks.push(Block::ToolUse { id, name, input });
            }
        }
    }
    Ok(out)
}
