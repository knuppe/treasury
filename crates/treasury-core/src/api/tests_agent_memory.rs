//! Agent through the API: procedure memory (validation, export, notes in the request) and the
//! reply split into reasoning, tools and final text when the conversation is saved.

use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::*;
use crate::ai::providers::fake::{serve, sse};

struct T {
    _dir: tempfile::TempDir,
    api: SharedApi,
    acc: String,
}

fn setup() -> T {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap()
        .into_shared();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    let inst = api
        .call(
            "household.create_institution",
            json!({"name": "Nubank", "kind": "bank"}),
        )
        .unwrap()["id"]
        .clone();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta Nu", "kind": "checking", "institution_id": inst}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    T {
        _dir: dir,
        api,
        acc,
    }
}

fn wait(api: &Api, job: &str) -> (String, Vec<Value>) {
    let started = Instant::now();
    loop {
        let p = api
            .call("ai.chat_poll", json!({"job_id": job, "after_seq": 0}))
            .unwrap();
        if p["status"] != "running" {
            return (
                p["status"].as_str().unwrap().to_string(),
                p["events"].as_array().unwrap().clone(),
            );
        }
        assert!(started.elapsed() < Duration::from_secs(20), "{p}");
        std::thread::sleep(Duration::from_millis(30));
    }
}

#[test]
fn procedures_are_validated_and_go_into_the_export() {
    let t = setup();
    let bad = [
        json!({"topic": "statement", "steps": "x"}),
        json!({"topic": "statement", "title": "x", "steps": "y", "site": "não é site"}),
        json!({"title": "x", "steps": "y"}),
        json!({"topic": "statement", "title": "  ", "steps": "y"}),
        json!({"topic": "statement", "title": "x", "steps": "y".repeat(2001)}),
    ];
    for b in bad {
        assert!(
            matches!(
                t.api.call("ai.procedures.save", b.clone()),
                Err(Error::Invalid(_))
            ),
            "{b}"
        );
    }
    assert!(matches!(
        t.api.call(
            "ai.procedures.save",
            json!({"topic": "statement", "title": "x", "steps": "y", "account_id": "nao-existe"})
        ),
        Err(Error::NotFound(_))
    ));
    let saved = t
        .api
        .call(
            "ai.procedures.save",
            json!({"topic": "card_bill", "title": "Fatura do Nubank", "steps": "App › Cartão › Faturas › PDF", "account_id": t.acc}),
        )
        .unwrap();
    assert_eq!(saved["created"], true);
    let id = saved["id"].clone();
    let p = t.api.call("ai.procedures.get", json!({"id": id})).unwrap();
    assert_eq!(p["created_by"], "user");
    assert_eq!(p["institution_name"], "Nubank");
    // Edit: `null` clears, absent keeps.
    t.api
        .call(
            "ai.procedures.save",
            json!({"id": id, "file_format": "pdf", "account_id": null}),
        )
        .unwrap();
    let p = t.api.call("ai.procedures.get", json!({"id": id})).unwrap();
    assert_eq!(p["file_format"], "PDF");
    assert!(p["account_id"].is_null());
    assert_eq!(p["title"], "Fatura do Nubank");
    t.api
        .call(
            "ai.procedures.outcome",
            json!({"id": id, "outcome": "worked"}),
        )
        .unwrap();
    // Filters and search.
    let found = t
        .api
        .call("ai.procedures.list", json!({"query": "fatura nubank"}))
        .unwrap();
    assert_eq!(found.as_array().unwrap().len(), 1);
    let none = t
        .api
        .call("ai.procedures.list", json!({"topic": "tax"}))
        .unwrap();
    assert_eq!(none.as_array().unwrap().len(), 0);

    // Open export: the notes and their use come out together with the other data.
    let out = tempfile::tempdir().unwrap();
    let path = out.path().join("t.json");
    t.api
        .call("export.json", json!({"path": path.to_str().unwrap()}))
        .unwrap();
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(v["tables"]["ai_procedure"][0]["title"], "Fatura do Nubank");
    assert_eq!(v["tables"]["ai_procedure_use"][0]["worked"], 1);
}

fn anthropic(events: &[(&str, Value)]) -> String {
    let mut all: Vec<(Option<&str>, Value)> = vec![(
        Some("message_start"),
        json!({"type": "message_start", "message": {"usage": {"input_tokens": 100, "output_tokens": 1}}}),
    )];
    all.extend(events.iter().map(|(e, v)| (Some(*e), v.clone())));
    sse(&all)
}

#[test]
fn relevant_notes_go_with_the_turn_and_reasoning_is_kept_apart_from_the_answer() {
    let t = setup();
    t.api
        .call(
            "ai.procedures.save",
            json!({"topic": "statement", "title": "Extrato OFX da Conta Nu", "steps": "App › Extrato › Exportar OFX", "site": "nubank.com.br", "account_id": t.acc}),
        )
        .unwrap();
    t.api
        .call(
            "ai.procedures.save",
            json!({"topic": "tax", "title": "Informe de rendimentos da corretora", "steps": "Área do cliente › IR"}),
        )
        .unwrap();
    let first = anthropic(&[
        (
            "content_block_start",
            json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": ""}}),
        ),
        (
            "content_block_delta",
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "Preciso do saldo da conta."}}),
        ),
        (
            "content_block_delta",
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "signature_delta", "signature": "sig"}}),
        ),
        (
            "content_block_start",
            json!({"type": "content_block_start", "index": 1, "content_block": {"type": "text", "text": ""}}),
        ),
        (
            "content_block_delta",
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "text_delta", "text": "Vou ver o saldo."}}),
        ),
        (
            "content_block_start",
            json!({"type": "content_block_start", "index": 2, "content_block": {"type": "tool_use", "id": "toolu_1", "name": "get_account_balance", "input": {}}}),
        ),
        (
            "content_block_delta",
            json!({"type": "content_block_delta", "index": 2, "delta": {"type": "input_json_delta", "partial_json": json!({"account_id": t.acc}).to_string()}}),
        ),
    ]);
    let second = anthropic(&[
        (
            "content_block_start",
            json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}),
        ),
        (
            "content_block_delta",
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Saldo em dia."}}),
        ),
    ]);
    let srv = serve(vec![
        (200, "text/event-stream", first),
        (200, "text/event-stream", second),
        (200, "text/event-stream", anthropic(&[])),
    ]);
    let id = t
        .api
        .call(
            "ai.engines.create",
            json!({"provider": "anthropic", "endpoint": srv.url, "api_key": "sk-teste-1234"}),
        )
        .unwrap()["id"]
        .clone();
    t.api
        .call("ai.routes.set", json!({"task": "chat", "engine_ids": [id]}))
        .unwrap();
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({
                "message": "Como está o saldo aqui?",
                "page_context": {"screen": "contas", "title": "03 · Contas", "selection": {"account_id": t.acc}},
            }),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done", "{events:?}");
    assert!(events.iter().any(|e| e["event"]["type"] == "reasoning"));

    let reqs = srv.requests.lock().unwrap().clone();
    let user = reqs[0].body["messages"].as_array().unwrap().last().unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(user.starts_with("Como está o saldo aqui?"), "{user}");
    assert_eq!(user.matches("<notas_do_agente>").count(), 1, "{user}");
    assert!(user.contains("Extrato OFX da Conta Nu"));
    assert!(
        !user.contains("Informe de rendimentos"),
        "unrelated note stays out"
    );
    assert!(user.ends_with("</notas_do_agente>"));
    // The signed thinking comes back on the next turn.
    let assistant = &reqs[1].body["messages"][1]["content"];
    assert_eq!(assistant[0]["type"], "thinking");

    // The conversation saves the final reply; reasoning and intermediate steps apart.
    let conv = t
        .api
        .call("ai.conversations.get", json!({"id": s["conversation_id"]}))
        .unwrap();
    let m = &conv["messages"][1];
    assert_eq!(m["content"], "Saldo em dia.");
    let reasoning = m["reasoning"].as_str().unwrap();
    assert!(
        reasoning.contains("Preciso do saldo da conta."),
        "{reasoning}"
    );
    assert!(reasoning.contains("Vou ver o saldo."), "{reasoning}");
    assert!(m["reasoning_ms"].is_number());
    assert_eq!(m["tool_calls"][0]["name"], "get_account_balance");
    assert!(
        m["tool_calls"][0]["args"]
            .as_str()
            .unwrap()
            .contains(&t.acc)
    );
    assert_eq!(m["tool_calls"][0]["ok"], true);

    // Request unrelated to the notes: nothing goes along. And only the final reply comes back as
    // history (not the reasoning).
    let s2 = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "obrigado", "conversation_id": s["conversation_id"]}),
        )
        .unwrap();
    wait(&t.api, s2["job_id"].as_str().unwrap());
    let req = srv.requests.lock().unwrap()[2].clone();
    let body = req.body.to_string();
    assert!(!body.contains("notas_do_agente>\\n"), "{body}");
    assert!(!body.contains("Preciso do saldo"));
    assert!(body.contains("Saldo em dia."));
}

#[test]
fn many_notes_go_by_relevance_and_the_graph_finds_neighbors() {
    let t = setup();
    // 13 unrelated notes + 2 about the Conta Nu account (over the instructions limit).
    for i in 0..13 {
        t.api
            .call(
                "ai.remember",
                json!({"kind": "note", "content": format!("Preferência genérica número {i} sobre viagens")}),
            )
            .unwrap();
    }
    t.api
        .call(
            "ai.remember",
            json!({"kind": "note", "content": "A Conta Nu recebe o salário da Ana todo dia 5."}),
        )
        .unwrap();
    t.api
        .call(
            "ai.remember",
            json!({"kind": "note", "content": "A Ana prefere investir pela corretora, não pelo banco."}),
        )
        .unwrap();
    t.api
        .call(
            "ai.remember",
            json!({"kind": "family_rule", "content": "Não sugerir cripto."}),
        )
        .unwrap();
    let srv = serve(vec![(200, "text/event-stream", anthropic(&[]))]);
    let id = t
        .api
        .call(
            "ai.engines.create",
            json!({"provider": "anthropic", "endpoint": srv.url, "api_key": "sk-teste-1234"}),
        )
        .unwrap()["id"]
        .clone();
    t.api
        .call("ai.routes.set", json!({"task": "chat", "engine_ids": [id]}))
        .unwrap();
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Quando cai o dinheiro na Conta Nu?"}),
        )
        .unwrap();
    wait(&t.api, s["job_id"].as_str().unwrap());
    let req = srv.requests.lock().unwrap()[0].body.clone();
    let system = req["system"][0]["text"].as_str().unwrap();
    // Family rules always; the notes no longer fit in the instructions.
    assert!(system.contains("Não sugerir cripto"));
    assert!(
        system.contains("There are 15 notes in the manager memory"),
        "{system}"
    );
    assert!(!system.contains("Preferência genérica"));
    let user = req["messages"].as_array().unwrap().last().unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(user.contains("<memoria_relevante>"), "{user}");
    assert!(user.contains("recebe o salário da Ana todo dia 5"));
    assert!(user.contains("(about: "));
    assert!(!user.contains("Preferência genérica"), "{user}");

    // Graph search: the account's note and, as a neighbor, the other note about Ana.
    let o = crate::mcp::inproc::call_tool_in(
        &t.api,
        &Actor::Mcp {
            client: "agente".into(),
            read_only: false,
        },
        "recall_memory",
        json!({"query": "conta nu"}),
        crate::mcp::inproc::AGENT_ACCESS,
        None,
    );
    assert!(!o.is_error, "{}", o.content);
    let v: Value = serde_json::from_str(&o.content).unwrap_or(Value::Null);
    let text = if v.is_null() {
        o.content.clone()
    } else {
        v.to_string()
    };
    assert!(text.contains("recebe o salário"), "{text}");
    assert!(
        text.contains("prefere investir pela corretora"),
        "neighbor through Ana: {text}"
    );
}
