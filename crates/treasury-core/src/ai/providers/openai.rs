//! OpenAI Chat Completions com function calling, em streaming. Serve também xAI, Mistral,
//! OpenRouter, endpoints compatíveis, Ollama e LM Studio (`/v1/chat/completions`).

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::*;

fn messages(t: &Turn<'_>) -> Vec<Value> {
    let mut out = vec![json!({"role": "system", "content": t.system})];
    for m in t.messages {
        match m.role {
            Role::User => {
                // Só texto: `content` em texto (aceito por todo servidor compatível). Com
                // imagem ou PDF: lista de partes (`text`, `image_url` com URL `data:`, `file`).
                let mut parts: Vec<Value> = vec![];
                let mut text = String::new();
                let mut rich = false;
                let flush = |text: &mut String, parts: &mut Vec<Value>| {
                    if !text.is_empty() {
                        parts.push(json!({"type": "text", "text": std::mem::take(text)}));
                    }
                };
                for b in &m.blocks {
                    match b {
                        Block::ToolResult { id, content, .. } => {
                            out.push(
                                json!({"role": "tool", "tool_call_id": id, "content": content}),
                            );
                        }
                        Block::Text(s) => {
                            if !text.is_empty() {
                                text.push_str("\n\n");
                            }
                            text.push_str(s);
                        }
                        Block::Image { mime, data, .. } => {
                            rich = true;
                            flush(&mut text, &mut parts);
                            parts.push(json!({"type": "image_url", "image_url": {"url": data_url(mime, data)}}));
                        }
                        Block::Document {
                            mime, name, data, ..
                        } => {
                            rich = true;
                            flush(&mut text, &mut parts);
                            parts.push(json!({"type": "file", "file": {"filename": name, "file_data": data_url(mime, data)}}));
                        }
                        Block::Attachment(a) => {
                            if !text.is_empty() {
                                text.push_str("\n\n");
                            }
                            text.push_str(&unresolved(a));
                        }
                        Block::ToolUse { .. } | Block::Thinking { .. } => {}
                    }
                }
                if rich {
                    flush(&mut text, &mut parts);
                    out.push(json!({"role": "user", "content": parts}));
                } else if !text.is_empty() {
                    out.push(json!({"role": "user", "content": text}));
                }
            }
            Role::Assistant => {
                let text = m.text();
                let calls: Vec<Value> = m
                    .blocks
                    .iter()
                    .filter_map(|b| match b {
                        Block::ToolUse { id, name, input } => Some(json!({
                            "id": id, "type": "function",
                            "function": {"name": name, "arguments": input.to_string()},
                        })),
                        _ => None,
                    })
                    .collect();
                let mut msg = json!({"role": "assistant", "content": if text.is_empty() { Value::Null } else { Value::String(text) }});
                if !calls.is_empty() {
                    msg["tool_calls"] = Value::Array(calls);
                }
                out.push(msg);
            }
        }
    }
    out
}

pub fn body(provider: Provider, t: &Turn<'_>) -> Value {
    let mut b = json!({
        "model": t.model,
        "messages": messages(t),
        "stream": true,
    });
    // Modelos novos da OpenAI só aceitam `max_completion_tokens`.
    if provider == Provider::Openai {
        b["max_completion_tokens"] = json!(t.max_tokens);
    } else {
        b["max_tokens"] = json!(t.max_tokens);
    }
    if matches!(
        provider,
        Provider::Openai | Provider::Openrouter | Provider::Xai
    ) {
        b["stream_options"] = json!({"include_usage": true});
    }
    if let Some((k, v)) = crate::ai::effort::openai_fields(provider, t.model, t.effort) {
        b[k] = v;
    }
    if !t.tools.is_empty() {
        b["tools"] = Value::Array(
            t.tools
                .iter()
                .map(|d| {
                    json!({"type": "function", "function": {
                        "name": d.name, "description": d.description, "parameters": d.schema,
                    }})
                })
                .collect(),
        );
    }
    b
}

#[derive(Default)]
struct Call {
    id: String,
    name: String,
    args: String,
}

pub fn turn(
    ep: &Endpoint<'_>,
    t: &Turn<'_>,
    on_delta: &mut dyn FnMut(Delta<'_>),
) -> Result<TurnOutput, ProviderError> {
    let url = join_url(ep.base_url, "chat/completions");
    let mut headers = vec![];
    if let Some(k) = ep.api_key.filter(|k| !k.is_empty()) {
        headers.push(("Authorization", format!("Bearer {k}")));
    }
    if ep.provider == Provider::Openrouter {
        headers.push(("X-Title", "Treasury".to_string()));
    }
    let mut text = String::new();
    let mut reasoning = String::new();
    let mut think = ThinkSplitter::default();
    // Texto e raciocínio já separados (o `<think>` de modelos locais sai do texto).
    let mut deliver = |d: Delta<'_>, text: &mut String, reasoning: &mut String| match d {
        Delta::Text(s) => {
            // A resposta não começa com as linhas em branco que sobram depois do `</think>`.
            let s = if text.is_empty() { s.trim_start() } else { s };
            if !s.is_empty() {
                text.push_str(s);
                on_delta(Delta::Text(s));
            }
        }
        Delta::Reasoning(s) => {
            reasoning.push_str(s);
            on_delta(Delta::Reasoning(s));
        }
    };
    let mut calls: BTreeMap<u64, Call> = BTreeMap::new();
    let mut out = TurnOutput::default();
    let mut stream_error: Option<ProviderError> = None;
    post_lines(ep, &url, &headers, &body(ep.provider, t), &mut |line| {
        let Some(data) = sse_data(line) else {
            return Ok(());
        };
        let v = parse_json(data)?;
        if let Some(e) = v.get("error").filter(|e| !e.is_null()) {
            let msg = e["message"].as_str().map_or_else(
                || crate::i18n::t("nucleo-ia-provider-streaming"),
                str::to_string,
            );
            stream_error = Some(ProviderError::new(
                ErrKind::Server,
                format!("{}: {msg}", ep.provider.label()),
            ));
            return Ok(());
        }
        // O Groq manda o uso no último pedaço, em `x_groq.usage`.
        if let Some(u) = v.get("usage").filter(|u| u.is_object()).or_else(|| {
            v.get("x_groq")
                .and_then(|x| x.get("usage"))
                .filter(|u| u.is_object())
        }) {
            out.tokens.input = u["prompt_tokens"].as_i64().unwrap_or(0);
            out.tokens.output = u["completion_tokens"].as_i64().unwrap_or(0);
            out.tokens.cached = u["prompt_tokens_details"]["cached_tokens"]
                .as_i64()
                .unwrap_or(0);
            out.usage_reported = true;
        }
        for ch in v["choices"].as_array().into_iter().flatten() {
            let d = &ch["delta"];
            // `reasoning_content` (DeepSeek, LM Studio, vLLM, xAI) ou `reasoning` (OpenRouter,
            // Ollama).
            for key in ["reasoning_content", "reasoning"] {
                if let Some(s) = d[key].as_str()
                    && !s.is_empty()
                {
                    deliver(Delta::Reasoning(s), &mut text, &mut reasoning);
                }
            }
            if let Some(s) = d["content"].as_str()
                && !s.is_empty()
            {
                let mut parts: Vec<(bool, String)> = vec![];
                think.push(s, &mut |p| match p {
                    Delta::Text(t) => parts.push((false, t.to_string())),
                    Delta::Reasoning(t) => parts.push((true, t.to_string())),
                });
                for (is_reasoning, p) in parts {
                    let d = if is_reasoning {
                        Delta::Reasoning(&p)
                    } else {
                        Delta::Text(&p)
                    };
                    deliver(d, &mut text, &mut reasoning);
                }
            }
            for tc in d["tool_calls"].as_array().into_iter().flatten() {
                let i = tc["index"].as_u64().unwrap_or(calls.len() as u64);
                let c = calls.entry(i).or_default();
                if let Some(id) = tc["id"].as_str() {
                    c.id = id.to_string();
                }
                if let Some(n) = tc["function"]["name"].as_str() {
                    c.name.push_str(n);
                }
                if let Some(a) = tc["function"]["arguments"].as_str() {
                    c.args.push_str(a);
                } else if tc["function"]["arguments"].is_object() {
                    // Alguns servidores locais mandam o objeto pronto.
                    c.args = tc["function"]["arguments"].to_string();
                }
            }
        }
        Ok(())
    })?;
    if let Some(e) = stream_error {
        return Err(e);
    }
    let mut rest: Vec<(bool, String)> = vec![];
    think.finish(&mut |p| match p {
        Delta::Text(t) => rest.push((false, t.to_string())),
        Delta::Reasoning(t) => rest.push((true, t.to_string())),
    });
    for (is_reasoning, p) in rest {
        let d = if is_reasoning {
            Delta::Reasoning(&p)
        } else {
            Delta::Text(&p)
        };
        deliver(d, &mut text, &mut reasoning);
    }
    if !reasoning.is_empty() {
        out.blocks.push(Block::Thinking {
            text: reasoning,
            signature: String::new(),
        });
    }
    if !text.is_empty() {
        out.blocks.push(Block::Text(text));
    }
    for (i, c) in calls {
        let input = if c.args.trim().is_empty() {
            json!({})
        } else {
            parse_json(&c.args)?
        };
        let id = if c.id.is_empty() {
            format!("call_{i}")
        } else {
            c.id
        };
        out.blocks.push(Block::ToolUse {
            id,
            name: c.name,
            input,
        });
    }
    Ok(out)
}
