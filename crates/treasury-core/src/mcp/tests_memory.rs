//! Memória de procedimentos do agente pelo MCP: o agente do app grava direto (com
//! histórico, desfazer e a conversa de origem); clientes externos só leem.

use std::sync::Arc;

use rmcp::model::Implementation;
use rmcp::model::{CallToolRequestParams, CallToolResult, ClientCapabilities, ClientConfig};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};
use serde_json::{Value, json};

use super::*;
use crate::ai::jobs::JobKind;
use crate::ai::routing::AiTask;
use crate::api::Api;
use crate::paths::TreasuryPaths;

fn vault() -> (tempfile::TempDir, SharedApi) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Arc::new(Api::new_unlocked_process(paths).unwrap());
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Casa", "members": ["Ana"]}),
    )
    .unwrap();
    (dir, api)
}

async fn connect(server: TreasuryMcp) -> RunningService<RoleClient, ClientConfig> {
    let (server_io, client_io) = tokio::io::duplex(1 << 16);
    tokio::spawn(async move {
        let _ = server.serve_transport(server_io).await;
    });
    ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new("cli-teste", "1.0"),
    )
    .serve(client_io)
    .await
    .unwrap()
}

async fn call(
    client: &RunningService<RoleClient, ClientConfig>,
    tool: &str,
    args: Value,
) -> CallToolResult {
    let mut params = CallToolRequestParams::new(tool.to_string());
    if let Value::Object(map) = args {
        params = params.with_arguments(map);
    }
    client.call_tool(params).await.unwrap()
}

fn data(r: &CallToolResult) -> Value {
    r.structured_content.clone().unwrap_or(Value::Null)
}

async fn names(client: &RunningService<RoleClient, ClientConfig>) -> Vec<String> {
    client
        .list_all_tools()
        .await
        .unwrap()
        .iter()
        .map(|t| t.name.to_string())
        .collect()
}

#[tokio::test]
async fn external_clients_only_read_the_procedure_memory() {
    let (_d, api) = vault();
    api.call(
        "ai.procedures.save",
        json!({"topic": "statement", "title": "Extrato do banco", "steps": "Menu Extrato › Exportar OFX", "site": "banco.com.br"}),
    )
    .unwrap();
    super::tests::allow_keyless(&api);
    let client = connect(TreasuryMcp::new(api.clone(), "stdio")).await;
    // Nem com as escritas liberadas o cliente externo grava na memória do agente.
    api.call("ai.settings.set", json!({"mcp_allow_writes": true}))
        .unwrap();
    let tools = names(&client).await;
    assert!(tools.contains(&"recall_procedures".to_string()));
    assert!(!tools.contains(&"remember_procedure".to_string()));
    assert!(!tools.contains(&"record_procedure_outcome".to_string()));
    let r = call(
        &client,
        "remember_procedure",
        json!({"topic": "other", "title": "x", "steps": "y"}),
    )
    .await;
    assert_eq!(r.is_error, Some(true));
    assert_eq!(data(&r)["error"]["code"], "forbidden");
    let r = call(&client, "recall_procedures", json!({"query": "extrato"})).await;
    assert_eq!(r.is_error, Some(false));
    assert_eq!(data(&r)["count"], 1);
    assert_eq!(data(&r)["procedures"][0]["site"], "banco.com.br");
    // E pelo despachante, um escritor externo também não chega lá.
    let writer = Actor::Mcp {
        client: "externo".into(),
        read_only: false,
    };
    assert!(matches!(
        api.call_as(&writer, "ai.procedures.delete", json!({"id": "x"})),
        Err(Error::Forbidden(_))
    ));
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn app_agent_records_procedures_directly_with_history_and_origin() {
    let (_d, api) = vault();
    let inst = api
        .call(
            "household.create_institution",
            json!({"name": "Itaú", "kind": "bank"}),
        )
        .unwrap()["id"]
        .clone();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Itaú Corrente", "kind": "checking", "institution_id": inst}),
        )
        .unwrap()["id"]
        .clone();
    let conv = api
        .with_session(|s, _| {
            crate::ai::conversation::create(s.db.conn(), AiTask::Chat, "Extrato do Itaú")
        })
        .unwrap();
    let job = api
        .ai_runtime()
        .new_job(JobKind::Chat, AiTask::Chat, Some(conv.clone()));
    let client = connect(TreasuryMcp::for_agent(
        api.clone(),
        "agente (CLI de teste)",
        job.id.clone(),
    ))
    .await;
    let tools = names(&client).await;
    assert!(tools.contains(&"remember_procedure".to_string()));
    assert!(tools.contains(&"record_procedure_outcome".to_string()));

    let r = call(
        &client,
        "remember_procedure",
        json!({
            "topic": "statement",
            "title": "Extrato OFX da conta Itaú",
            "steps": "Entrar (senha: 123456). Menu Conta corrente › Extrato › Exportar › OFX, período de 90 dias.",
            "site": "https://www.itau.com.br/extrato?sessao=abc",
            "file_format": "ofx",
            "account_id": acc,
        }),
    )
    .await;
    assert_eq!(r.is_error, Some(false), "{r:?}");
    let saved = data(&r);
    assert_eq!(saved["created"], true);
    assert!(saved["redactions"].as_u64().unwrap() >= 1);
    let id = saved["id"].as_str().unwrap().to_string();
    // Direto: nenhuma proposta na fila; o histórico tem a alteração com autoria da IA.
    assert_eq!(api.call("ai.pending_count", json!({})).unwrap(), 0);
    let changes = api.call("changes.list", json!({"limit": 1})).unwrap();
    let cs = &changes.as_array().unwrap()[0];
    assert_eq!(cs["author"]["kind"], "ai");
    assert!(
        cs["reason"]
            .as_str()
            .unwrap()
            .contains("Memória de procedimentos")
    );

    let list = api.call("ai.procedures.list", json!({})).unwrap();
    let p = &list[0];
    assert_eq!(p["site"], "www.itau.com.br");
    assert_eq!(p["file_format"], "OFX");
    assert_eq!(p["institution_id"], inst, "instituição vem da conta");
    assert_eq!(p["institution_name"], "Itaú");
    assert_eq!(p["account_name"], "Itaú Corrente");
    assert_eq!(p["created_by"], "ai");
    assert!(
        p["client"]
            .as_str()
            .unwrap()
            .contains("agente (CLI de teste)")
    );
    assert_eq!(p["conversation_id"], conv.as_str());
    assert_eq!(p["conversation_title"], "Extrato do Itaú");
    assert!(!p["steps"].as_str().unwrap().contains("123456"));

    // Uso: falhou, depois funcionou; a observação fica.
    let r = call(
        &client,
        "record_procedure_outcome",
        json!({"id": id, "outcome": "failed", "note": "o menu agora é Extratos › Exportar"}),
    )
    .await;
    assert_eq!(data(&r)["failed"], 1);
    let r = call(
        &client,
        "record_procedure_outcome",
        json!({"id": id, "outcome": "worked"}),
    )
    .await;
    let used = data(&r);
    assert_eq!(
        (used["worked"].as_i64(), used["failed"].as_i64()),
        (Some(1), Some(1))
    );
    assert_eq!(used["last_outcome"], "worked");
    assert_eq!(used["last_note"], "o menu agora é Extratos › Exportar");
    // A estatística de uso não entra no histórico.
    let after = api.call("changes.list", json!({"limit": 1})).unwrap();
    assert_eq!(after[0]["id"], cs["id"]);

    // Mesmo assunto, título e conta: atualiza em vez de duplicar.
    let r = call(
        &client,
        "remember_procedure",
        json!({"topic": "statement", "title": "Extrato OFX da conta Itaú", "steps": "Menu Extratos › Exportar › OFX", "account_id": acc}),
    )
    .await;
    assert_eq!(data(&r)["created"], false);
    let list = api
        .call("ai.procedures.list", json!({"account_id": acc}))
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["steps"], "Menu Extratos › Exportar › OFX");
    assert_eq!(
        list[0]["site"], "www.itau.com.br",
        "campo ausente fica como estava"
    );

    // O usuário desfaz pelo histórico e apaga pela interface; o agente não apaga.
    let last = api.call("changes.list", json!({"limit": 1})).unwrap()[0]["id"].clone();
    api.call("changes.undo", json!({"id": last})).unwrap();
    let restored = api.call("ai.procedures.get", json!({"id": id})).unwrap();
    assert!(restored["steps"].as_str().unwrap().contains("90 dias"));
    let agent = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(
        api.call_as(&agent, "ai.procedures.delete", json!({"id": id}))
            .is_err()
    );
    api.call("ai.procedures.delete", json!({"id": id})).unwrap();
    assert_eq!(
        api.call("ai.procedures.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        0
    );
    client.cancel().await.unwrap();
}
