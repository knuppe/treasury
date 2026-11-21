//! Adapters against fake HTTP servers in the format of each provider.

use serde_json::json;

use super::fake::{serve, sse};
use super::*;

fn tools() -> Vec<ToolDef> {
    vec![ToolDef {
        name: "get_dashboard".into(),
        description: "Painel do mês".into(),
        schema: json!({"type": "object", "properties": {"month": {"type": ["string", "null"], "description": "Mês"}}, "additionalProperties": false}),
    }]
}

fn never() -> bool {
    false
}

fn run(provider: Provider, url: &str, msgs: &[Msg]) -> (Result<TurnOutput, ProviderError>, String) {
    let (r, text, _) = run_model(provider, url, msgs, "modelo-x");
    (r, text)
}

/// Like `run`, also returning the reasoning received while streaming.
fn run_model(
    provider: Provider,
    url: &str,
    msgs: &[Msg],
    model: &str,
) -> (Result<TurnOutput, ProviderError>, String, String) {
    let tools = tools();
    let ep = Endpoint {
        provider,
        base_url: url,
        api_key: Some("chave-secreta-123"),
        cancelled: &never,
    };
    let mut streamed = String::new();
    let mut reasoning = String::new();
    let r = turn(
        &ep,
        &Turn {
            model,
            system: "Você é o gestor.",
            messages: msgs,
            tools: &tools,
            max_tokens: 256,
            effort: Default::default(),
        },
        &mut |d| match d {
            Delta::Text(t) => streamed.push_str(t),
            Delta::Reasoning(t) => reasoning.push_str(t),
        },
    );
    (r, streamed, reasoning)
}

#[test]
fn anthropic_tool_use_then_text() {
    let call = sse(&[
        (
            Some("message_start"),
            json!({"type": "message_start", "message": {"usage": {"input_tokens": 120, "output_tokens": 1, "cache_read_input_tokens": 20}}}),
        ),
        (
            Some("content_block_start"),
            json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}),
        ),
        (
            Some("content_block_delta"),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Vou consultar."}}),
        ),
        (
            Some("content_block_start"),
            json!({"type": "content_block_start", "index": 1, "content_block": {"type": "tool_use", "id": "toolu_1", "name": "get_dashboard", "input": {}}}),
        ),
        (
            Some("content_block_delta"),
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "{\"month\":"}}),
        ),
        (
            Some("content_block_delta"),
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "\"2026-09\"}"}}),
        ),
        (
            Some("message_delta"),
            json!({"type": "message_delta", "delta": {"stop_reason": "tool_use"}, "usage": {"output_tokens": 30}}),
        ),
        (Some("message_stop"), json!({"type": "message_stop"})),
    ]);
    let srv = serve(vec![(200, "text/event-stream", call)]);
    let (r, streamed) = run(
        Provider::Anthropic,
        &srv.url,
        &[Msg::user("Como foi setembro?")],
    );
    let out = r.unwrap();
    assert_eq!(streamed, "Vou consultar.");
    assert_eq!(
        out.tool_uses(),
        vec![(
            "toolu_1".into(),
            "get_dashboard".into(),
            json!({"month": "2026-09"})
        )]
    );
    assert_eq!(
        out.tokens,
        Tokens {
            input: 140,
            output: 30,
            cached: 20,
            cache_write: 0
        }
    );
    let req = &srv.requests.lock().unwrap()[0];
    assert_eq!(req.path, "/v1/messages");
    assert_eq!(req.header("x-api-key"), Some("chave-secreta-123"));
    assert_eq!(req.header("anthropic-version"), Some("2023-06-01"));
    assert_eq!(req.body["tools"][0]["name"], "get_dashboard");
    // Prompt cache: on the instructions (with the tools before) and at the end of the conversation.
    assert_eq!(req.body["system"][0]["text"], "Você é o gestor.");
    assert_eq!(req.body["system"][0]["cache_control"]["type"], "ephemeral");
    let last = req.body["messages"].as_array().unwrap().last().unwrap()["content"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(last["cache_control"]["type"], "ephemeral");
    assert_eq!(req.body["stream"], true);
}

#[test]
fn anthropic_sends_tool_results_and_errors_are_scrubbed() {
    let srv = serve(vec![(
        401,
        "application/json",
        json!({"type": "error", "error": {"type": "authentication_error", "message": "invalid x-api-key chave-secreta-123"}}).to_string(),
    )]);
    let msgs = vec![
        Msg::user("oi"),
        Msg {
            role: Role::Assistant,
            blocks: vec![Block::ToolUse {
                id: "t1".into(),
                name: "get_dashboard".into(),
                input: json!({}),
            }],
            raw: None,
        },
        Msg {
            role: Role::User,
            blocks: vec![Block::ToolResult {
                id: "t1".into(),
                name: "get_dashboard".into(),
                content: "{\"ok\":1}".into(),
                is_error: false,
            }],
            raw: None,
        },
    ];
    let (r, _) = run(Provider::Anthropic, &srv.url, &msgs);
    let e = r.unwrap_err();
    assert_eq!(e.kind, ErrKind::Auth);
    assert!(!e.message.contains("chave-secreta-123"), "{}", e.message);
    let req = &srv.requests.lock().unwrap()[0];
    assert_eq!(req.body["messages"][2]["content"][0]["type"], "tool_result");
    assert_eq!(req.body["messages"][2]["content"][0]["tool_use_id"], "t1");
}

#[test]
fn openai_function_call_streamed_in_pieces() {
    let body = sse(&[
        (
            None,
            json!({"choices": [{"index": 0, "delta": {"role": "assistant", "content": null, "tool_calls": [{"index": 0, "id": "call_9", "type": "function", "function": {"name": "get_dashboard", "arguments": ""}}]}}]}),
        ),
        (
            None,
            json!({"choices": [{"index": 0, "delta": {"tool_calls": [{"index": 0, "function": {"arguments": "{\"month\":\"2026-09\"}"}}]}}]}),
        ),
        (
            None,
            json!({"choices": [{"index": 0, "delta": {}, "finish_reason": "tool_calls"}]}),
        ),
        (
            None,
            json!({"choices": [], "usage": {"prompt_tokens": 50, "completion_tokens": 12, "prompt_tokens_details": {"cached_tokens": 0}}}),
        ),
    ]) + "data: [DONE]\n\n";
    let srv = serve(vec![(200, "text/event-stream", body)]);
    let (r, _) = run(Provider::Openai, &srv.url, &[Msg::user("oi")]);
    let out = r.unwrap();
    assert_eq!(out.tool_uses()[0].0, "call_9");
    assert_eq!(out.tool_uses()[0].2, json!({"month": "2026-09"}));
    assert_eq!(out.tokens.input, 50);
    let req = &srv.requests.lock().unwrap()[0];
    assert_eq!(req.path, "/chat/completions");
    assert_eq!(
        req.header("authorization"),
        Some("Bearer chave-secreta-123")
    );
    assert_eq!(req.body["messages"][0]["role"], "system");
    assert_eq!(req.body["tools"][0]["function"]["name"], "get_dashboard");
    assert_eq!(req.body["max_completion_tokens"], 256);
    assert_eq!(req.body["stream_options"]["include_usage"], true);
}

#[test]
fn openai_compatible_local_text_without_usage_is_estimated() {
    let body = sse(&[
        (
            None,
            json!({"choices": [{"delta": {"content": "Gastos "}}]}),
        ),
        (
            None,
            json!({"choices": [{"delta": {"content": "em dia."}, "finish_reason": "stop"}]}),
        ),
    ]) + "data: [DONE]\n\n";
    let srv = serve(vec![(200, "text/event-stream", body)]);
    let (r, streamed) = run(Provider::Ollama, &srv.url, &[Msg::user("resumo")]);
    let out = r.unwrap();
    assert_eq!(streamed, "Gastos em dia.");
    assert!(!out.usage_reported && out.tokens.input > 0);
    let req = &srv.requests.lock().unwrap()[0];
    assert_eq!(req.body["max_tokens"], 256);
    assert!(req.body.get("stream_options").is_none());
}

#[test]
fn gemini_function_call_and_schema_cleanup() {
    let body = sse(&[
        (
            None,
            json!({"candidates": [{"content": {"role": "model", "parts": [{"functionCall": {"name": "get_dashboard", "args": {"month": "2026-09"}}, "thoughtSignature": "abc"}]}}]}),
        ),
        (
            None,
            json!({"candidates": [{"content": {"role": "model", "parts": []}, "finishReason": "STOP"}], "usageMetadata": {"promptTokenCount": 70, "candidatesTokenCount": 9}}),
        ),
    ]);
    let srv = serve(vec![(200, "text/event-stream", body)]);
    let (r, _) = run(Provider::Gemini, &srv.url, &[Msg::user("oi")]);
    let out = r.unwrap();
    assert_eq!(out.tool_uses()[0].1, "get_dashboard");
    assert_eq!(out.tokens.input, 70);
    // The signature comes back on the next turn.
    assert_eq!(out.raw.as_ref().unwrap()[0]["thoughtSignature"], "abc");
    let req = &srv.requests.lock().unwrap()[0];
    assert_eq!(req.path, "/models/modelo-x:streamGenerateContent?alt=sse");
    assert_eq!(req.header("x-goog-api-key"), Some("chave-secreta-123"));
    let params = &req.body["tools"][0]["functionDeclarations"][0]["parameters"];
    assert!(params.get("additionalProperties").is_none());
    assert_eq!(params["properties"]["month"]["type"], "string");
    assert_eq!(params["properties"]["month"]["nullable"], true);
}

#[test]
fn rate_limit_is_classified_for_fallback() {
    let srv = serve(vec![(
        429,
        "application/json",
        json!({"error": {"message": "Rate limit"}}).to_string(),
    )]);
    let (r, _) = run(Provider::Xai, &srv.url, &[Msg::user("oi")]);
    assert_eq!(r.unwrap_err().kind, ErrKind::RateLimit);
}

#[test]
fn anthropic_thinking_is_reasoning_and_goes_back_signed() {
    let body = sse(&[
        (
            Some("content_block_start"),
            json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": ""}}),
        ),
        (
            Some("content_block_delta"),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "Preciso do painel."}}),
        ),
        (
            Some("content_block_delta"),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "signature_delta", "signature": "sig-1"}}),
        ),
        (
            Some("content_block_start"),
            json!({"type": "content_block_start", "index": 1, "content_block": {"type": "text", "text": ""}}),
        ),
        (
            Some("content_block_delta"),
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "text_delta", "text": "Tudo certo."}}),
        ),
    ]);
    let srv = serve(vec![
        (200, "text/event-stream", body),
        (200, "text/event-stream", sse(&[])),
    ]);
    let (r, text, reasoning) = run_model(
        Provider::Anthropic,
        &srv.url,
        &[Msg::user("oi")],
        "claude-x",
    );
    let out = r.unwrap();
    assert_eq!(text, "Tudo certo.");
    assert_eq!(reasoning, "Preciso do painel.");
    assert_eq!(
        out.blocks[0],
        Block::Thinking {
            text: "Preciso do painel.".into(),
            signature: "sig-1".into()
        }
    );
    // On the next turn the block comes back with the signature; without a signature, it does not.
    let history = vec![
        Msg::user("oi"),
        Msg {
            role: Role::Assistant,
            blocks: vec![
                out.blocks[0].clone(),
                Block::Thinking {
                    text: "sem assinatura".into(),
                    signature: String::new(),
                },
                Block::Text("Tudo certo.".into()),
            ],
            raw: None,
        },
        Msg::user("e agora?"),
    ];
    let _ = run(Provider::Anthropic, &srv.url, &history);
    let req = &srv.requests.lock().unwrap()[1];
    let content = req.body["messages"][1]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "thinking");
    assert_eq!(content[0]["signature"], "sig-1");
}

#[test]
fn openai_reasoning_fields_and_think_tags_are_reasoning() {
    let body = sse(&[
        (
            None,
            json!({"choices": [{"delta": {"reasoning_content": "Somar as "}}]}),
        ),
        (
            None,
            json!({"choices": [{"delta": {"reasoning": "despesas."}}]}),
        ),
        (None, json!({"choices": [{"delta": {"content": "<thi"}}]})),
        (
            None,
            json!({"choices": [{"delta": {"content": "nk>local</th"}}]}),
        ),
        (
            None,
            json!({"choices": [{"delta": {"content": "ink>\n\nGastos em dia."}, "finish_reason": "stop"}]}),
        ),
    ]) + "data: [DONE]\n\n";
    let srv = serve(vec![(200, "text/event-stream", body)]);
    let (r, text, reasoning) = run_model(Provider::Ollama, &srv.url, &[Msg::user("oi")], "qwen3");
    let out = r.unwrap();
    assert_eq!(text, "Gastos em dia.");
    assert_eq!(reasoning, "Somar as despesas.local");
    assert_eq!(
        out.blocks.last(),
        Some(&Block::Text("Gastos em dia.".into()))
    );
    // The reasoning does not go back to the provider on the next turn.
    let history = vec![
        Msg::user("oi"),
        Msg {
            role: Role::Assistant,
            blocks: out.blocks.clone(),
            raw: None,
        },
        Msg::user("e?"),
    ];
    let srv2 = serve(vec![(200, "text/event-stream", "data: [DONE]\n\n".into())]);
    let _ = run(Provider::Ollama, &srv2.url, &history);
    let req = &srv2.requests.lock().unwrap()[0];
    assert_eq!(req.body["messages"][2]["content"], "Gastos em dia.");
}

#[test]
fn think_splitter_handles_tags_cut_between_chunks() {
    let mut s = ThinkSplitter::default();
    let mut got: Vec<(bool, String)> = vec![];
    for chunk in ["a<", "think>b</", "think", ">c<t", "x"] {
        s.push(chunk, &mut |d| match d {
            Delta::Text(t) => got.push((false, t.into())),
            Delta::Reasoning(t) => got.push((true, t.into())),
        });
    }
    s.finish(&mut |d| match d {
        Delta::Text(t) => got.push((false, t.into())),
        Delta::Reasoning(t) => got.push((true, t.into())),
    });
    let text: String = got
        .iter()
        .filter(|(r, _)| !r)
        .map(|(_, t)| t.as_str())
        .collect();
    let reasoning: String = got
        .iter()
        .filter(|(r, _)| *r)
        .map(|(_, t)| t.as_str())
        .collect();
    assert_eq!(text, "ac<tx");
    assert_eq!(reasoning, "b");
}

#[test]
fn gemini_thoughts_are_reasoning_and_requested_only_on_thinking_models() {
    let body = sse(&[(
        None,
        json!({"candidates": [{"content": {"role": "model", "parts": [{"text": "Olhar o mês.", "thought": true}, {"text": "Mês tranquilo."}]}, "finishReason": "STOP"}]}),
    )]);
    let srv = serve(vec![
        (200, "text/event-stream", body),
        (200, "text/event-stream", sse(&[])),
    ]);
    let (r, text, reasoning) = run_model(
        Provider::Gemini,
        &srv.url,
        &[Msg::user("oi")],
        "gemini-2.5-flash",
    );
    r.unwrap();
    assert_eq!(text, "Mês tranquilo.");
    assert_eq!(reasoning, "Olhar o mês.");
    let _ = run_model(
        Provider::Gemini,
        &srv.url,
        &[Msg::user("oi")],
        "gemini-2.0-flash",
    );
    let reqs = srv.requests.lock().unwrap();
    assert_eq!(
        reqs[0].body["generationConfig"]["thinkingConfig"]["includeThoughts"],
        true
    );
    assert!(
        reqs[1].body["generationConfig"]
            .get("thinkingConfig")
            .is_none()
    );
}
