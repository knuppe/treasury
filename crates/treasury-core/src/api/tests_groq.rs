//! Groq como motor do agente (sem rede, sem chave real): o provider pronto, o modo enxuto
//! (poucas ferramentas, instruções curtas, resposta menor) e a espera do limite por minuto
//! do plano gratuito antes de continuar.

use serde_json::{Value, json};

use super::tests_agent::{engine, openai_text, route, setup, text_of, wait};
use crate::ai::providers::fake::serve;

#[test]
fn groq_is_ready_with_a_free_tier_model_that_calls_tools() {
    let t = setup();
    let providers = t.api.call("ai.engines.providers", Value::Null).unwrap();
    let g = providers
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["provider"] == "groq")
        .unwrap();
    assert_eq!(g["kind"], "api");
    assert_eq!(g["needs_key"], true);
    assert_eq!(g["default_endpoint"], "https://api.groq.com/openai/v1");
    assert_eq!(g["default_model"], "llama-3.3-70b-versatile");
}

#[test]
fn lean_mode_and_waiting_for_the_per_minute_limit() {
    let t = setup();
    let srv = serve(vec![
        (
            429,
            "application/json",
            json!({"error": {"message": "Rate limit reached for model `llama-3.3-70b-versatile` on tokens per minute (TPM)", "type": "tokens", "code": "rate_limit_exceeded"}}).to_string(),
        ),
        (200, "text/event-stream", openai_text("Tudo certo.")),
    ]);
    let id = engine(
        &t.api,
        json!({"provider": "groq", "endpoint": srv.url, "api_key": "gsk_teste_1234"}),
    );
    route(&t.api, "chat", &[&id]);
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Como estão os gastos do mês?"}),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done", "{events:?}");
    assert_eq!(text_of(&events), "Tudo certo.");
    // Esperou o limite (o 429 com Retry-After) em vez de desistir.
    assert!(
        events.iter().any(|e| e["event"]["type"] == "progress"
            && e["event"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("esperando 1 s")),
        "{events:?}"
    );
    let reqs = srv.requests.lock().unwrap();
    assert_eq!(reqs.len(), 2);
    let body = &reqs[1].body;
    assert_eq!(reqs[1].path, "/chat/completions");
    assert_eq!(
        reqs[1].header("authorization"),
        Some("Bearer gsk_teste_1234")
    );
    assert_eq!(body["model"], "llama-3.3-70b-versatile");
    assert_eq!(body["max_tokens"], 1536);
    assert!(body.get("stream_options").is_none());
    // Só as ferramentas do modo enxuto, e as instruções curtas.
    let tools: Vec<&str> = body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["function"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        tools.len(),
        crate::mcp::inproc::LEAN_TOOLS.len(),
        "{tools:?}"
    );
    assert!(tools.contains(&"get_dashboard") && !tools.contains(&"import_investments"));
    let system = body["messages"][0]["content"].as_str().unwrap();
    assert!(system.contains("small per-minute limit"), "{system}");
    assert!(system.len() < 4_000, "{}", system.len());
    let payload = serde_json::to_string(body).unwrap().len();
    assert!(
        payload < 30_000,
        "pedido do modo enxuto com {payload} bytes"
    );
}
