//! Google Gemini (`streamGenerateContent` com SSE) e function declarations.

use serde_json::{Map, Value, json};

use super::*;

/// Gemini aceita um subconjunto do OpenAPI: sem `$ref`, `additionalProperties` nem tipos
/// em lista. Resolve referências e converte `["string","null"]` em `nullable`.
pub fn schema(v: &Value) -> Value {
    let defs = v
        .get("$defs")
        .or_else(|| v.get("definitions"))
        .cloned()
        .unwrap_or(Value::Null);
    clean(v, &defs, 0)
}

fn clean(v: &Value, defs: &Value, depth: usize) -> Value {
    let Value::Object(map) = v else {
        return v.clone();
    };
    if depth > 12 {
        return json!({"type": "object"});
    }
    if let Some(r) = map.get("$ref").and_then(Value::as_str) {
        let name = r.rsplit('/').next().unwrap_or("");
        if let Some(d) = defs.get(name) {
            let mut resolved = clean(d, defs, depth + 1);
            if let (Some(desc), Value::Object(o)) = (map.get("description"), &mut resolved) {
                o.insert("description".into(), desc.clone());
            }
            return resolved;
        }
    }
    let mut out = Map::new();
    for (k, val) in map {
        match k.as_str() {
            "type" => match val {
                Value::Array(ts) => {
                    let non_null: Vec<&Value> = ts.iter().filter(|t| *t != "null").collect();
                    if ts.len() != non_null.len() {
                        out.insert("nullable".into(), json!(true));
                    }
                    out.insert(
                        "type".into(),
                        non_null
                            .first()
                            .map(|t| (*t).clone())
                            .unwrap_or(json!("string")),
                    );
                }
                other => {
                    out.insert("type".into(), other.clone());
                }
            },
            "properties" => {
                let props: Map<String, Value> = val
                    .as_object()
                    .map(|p| {
                        p.iter()
                            .map(|(n, s)| (n.clone(), clean(s, defs, depth + 1)))
                            .collect()
                    })
                    .unwrap_or_default();
                out.insert("properties".into(), Value::Object(props));
            }
            "items" => {
                out.insert("items".into(), clean(val, defs, depth + 1));
            }
            "anyOf" | "oneOf" => {
                let opts: Vec<Value> = val
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|o| o.get("type") != Some(&json!("null")))
                    .map(|o| clean(o, defs, depth + 1))
                    .collect();
                if opts.len() == 1 {
                    if let Value::Object(o) = &opts[0] {
                        for (k2, v2) in o {
                            out.entry(k2.clone()).or_insert(v2.clone());
                        }
                    }
                    out.insert("nullable".into(), json!(true));
                } else if !opts.is_empty() {
                    out.insert("anyOf".into(), Value::Array(opts));
                }
            }
            "description" | "enum" | "required" | "format" | "minimum" | "maximum" | "minItems"
            | "maxItems" | "nullable" => {
                out.insert(k.clone(), val.clone());
            }
            _ => {}
        }
    }
    if out.contains_key("enum") && !out.contains_key("type") {
        out.insert("type".into(), json!("string"));
    }
    Value::Object(out)
}

fn parts(m: &Msg) -> Value {
    if let Some(raw) = &m.raw {
        return raw.clone();
    }
    Value::Array(
        m.blocks
            .iter()
            .filter(|b| !matches!(b, Block::Thinking { .. }))
            .map(|b| match b {
                Block::Text(t) => json!({"text": t}),
                Block::Thinking { .. } => Value::Null,
                Block::ToolUse { name, input, .. } => {
                    json!({"functionCall": {"name": name, "args": input}})
                }
                Block::ToolResult {
                    name,
                    content,
                    is_error,
                    ..
                } => {
                    let response: Value = serde_json::from_str(content)
                        .ok()
                        .filter(Value::is_object)
                        .unwrap_or_else(|| json!({"result": content}));
                    let response = if *is_error {
                        json!({"error": response})
                    } else {
                        response
                    };
                    json!({"functionResponse": {"name": name, "response": response}})
                }
                Block::Image { mime, data, .. } | Block::Document { mime, data, .. } => {
                    json!({"inlineData": {"mimeType": mime, "data": attachments::b64(data)}})
                }
                Block::Attachment(a) => json!({"text": unresolved(a)}),
            })
            .collect(),
    )
}

pub fn body(t: &Turn<'_>) -> Value {
    let contents: Vec<Value> = t
        .messages
        .iter()
        .map(|m| {
            json!({
                "role": if m.role == Role::User { "user" } else { "model" },
                "parts": parts(m),
            })
        })
        .collect();
    let mut b = json!({
        "systemInstruction": {"parts": [{"text": t.system}]},
        "contents": contents,
        "generationConfig": {"maxOutputTokens": t.max_tokens},
    });
    if thinks(t.model) {
        // Resumos do raciocínio (sem custo extra: o pensamento já é cobrado nesses modelos).
        b["generationConfig"]["thinkingConfig"] = json!({"includeThoughts": true});
        if let Some(budget) = crate::ai::effort::gemini_budget(t.model, t.effort) {
            b["generationConfig"]["thinkingConfig"]["thinkingBudget"] = json!(budget);
        }
    }
    if !t.tools.is_empty() {
        b["tools"] = json!([{"functionDeclarations": t.tools.iter().map(|d| json!({
            "name": d.name, "description": d.description, "parameters": schema(&d.schema),
        })).collect::<Vec<_>>()}]);
    }
    b
}

/// Modelos que pensam e aceitam `thinkingConfig` (2.5 e posteriores). Nos outros o campo
/// seria recusado.
fn thinks(model: &str) -> bool {
    let m = model.trim_start_matches("models/");
    m.starts_with("gemini-2.5") || m.starts_with("gemini-3") || m.starts_with("gemini-4")
}

pub fn turn(
    ep: &Endpoint<'_>,
    t: &Turn<'_>,
    on_delta: &mut dyn FnMut(Delta<'_>),
) -> Result<TurnOutput, ProviderError> {
    let model = t.model.trim_start_matches("models/");
    let url = join_url(
        ep.base_url,
        &format!("models/{model}:streamGenerateContent?alt=sse"),
    );
    let headers = vec![("x-goog-api-key", ep.api_key.unwrap_or_default().to_string())];
    let mut out = TurnOutput::default();
    let mut raw_parts: Vec<Value> = vec![];
    let mut text = String::new();
    let mut calls = 0usize;
    post_lines(ep, &url, &headers, &body(t), &mut |line| {
        let Some(data) = sse_data(line) else {
            return Ok(());
        };
        let v = parse_json(data)?;
        if let Some(u) = v.get("usageMetadata") {
            out.tokens.input = u["promptTokenCount"].as_i64().unwrap_or(0);
            out.tokens.output = u["candidatesTokenCount"].as_i64().unwrap_or(0)
                + u["thoughtsTokenCount"].as_i64().unwrap_or(0);
            out.tokens.cached = u["cachedContentTokenCount"].as_i64().unwrap_or(0);
            out.usage_reported = true;
        }
        for p in v
            .pointer("/candidates/0/content/parts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            raw_parts.push(p.clone());
            if let Some(s) = p["text"].as_str() {
                if p["thought"] == json!(true) {
                    on_delta(Delta::Reasoning(s));
                } else {
                    text.push_str(s);
                    on_delta(Delta::Text(s));
                }
            }
            if let Some(fc) = p.get("functionCall") {
                if !text.is_empty() {
                    out.blocks.push(Block::Text(std::mem::take(&mut text)));
                }
                calls += 1;
                out.blocks.push(Block::ToolUse {
                    id: fc["id"]
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("gemini_{calls}")),
                    name: fc["name"].as_str().unwrap_or("").to_string(),
                    input: fc.get("args").cloned().unwrap_or(json!({})),
                });
            }
        }
        Ok(())
    })?;
    if !text.is_empty() {
        out.blocks.push(Block::Text(text));
    }
    out.raw = Some(Value::Array(raw_parts));
    Ok(out)
}
