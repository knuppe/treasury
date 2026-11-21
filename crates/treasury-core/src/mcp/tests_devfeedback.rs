//! Developer feedback through the MCP: the tools only show up (and only work) with
//! developer mode on, for the app's agent and for external clients.

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

const DEV: [&str; 11] = [
    "list_dev_feedback",
    "get_dev_feedback",
    "add_dev_feedback",
    "bump_dev_feedback",
    "resolve_dev_feedback",
    "update_dev_feedback",
    "set_dev_feedback_status",
    "set_dev_feedback_priority",
    "link_dev_feedback",
    "delete_dev_feedback",
    "export_dev_feedback",
];

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

fn set_dev(api: &Api, on: bool) {
    api.call("settings.set", json!({"developer_tools": on}))
        .unwrap();
}

#[tokio::test]
async fn external_clients_get_feedback_tools_only_in_developer_mode() {
    let (_d, api) = vault();
    super::tests::allow_keyless(&api);
    let client = connect(TreasuryMcp::new(api.clone(), "stdio")).await;
    let off = names(&client).await;
    assert!(DEV.iter().all(|d| !off.contains(&d.to_string())));
    let args = json!({"title": "Falta ferramenta de metas", "body": "Precisei e não havia.",
                      "area": "mcp", "kind": "ferramenta_faltando"});
    let r = call(&client, "add_dev_feedback", args.clone()).await;
    assert_eq!(data(&r)["error"]["code"], "forbidden");

    set_dev(&api, true);
    let tools = client.list_all_tools().await.unwrap();
    let on: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    assert_eq!(on.len(), off.len() + DEV.len());
    let add = tools.iter().find(|t| t.name == "add_dev_feedback").unwrap();
    assert_eq!(
        add.annotations.as_ref().unwrap().read_only_hint,
        Some(false)
    );
    assert!(
        add.description
            .as_deref()
            .unwrap()
            .contains("list_dev_feedback")
    );

    let r = call(&client, "add_dev_feedback", args.clone()).await;
    assert_eq!(r.is_error, Some(false), "{r:?}");
    let f = &data(&r)["feedback"];
    assert_eq!(f["created_by"], "cli-teste via stdio");
    let id = f["id"].clone();
    // Same title again: reinforces instead of duplicating.
    let r = call(&client, "add_dev_feedback", args).await;
    assert_eq!(data(&r)["deduplicated"], true);
    let r = call(
        &client,
        "bump_dev_feedback",
        json!({"id": id, "note": "de novo ao cadastrar meta"}),
    )
    .await;
    assert_eq!(data(&r)["importance"], 3);
    let r = call(&client, "list_dev_feedback", json!({"query": "metas"})).await;
    assert_eq!(data(&r)["count"], 1);
    assert_eq!(data(&r)["feedback"][0]["bumps"], 2);
    // Resolved by the agent, with the reason in the text; the reason is required and a closed one
    // is not resolved again.
    let r = call(&client, "resolve_dev_feedback", json!({"id": id})).await;
    assert_eq!(r.is_error, Some(true), "{r:?}");
    let r = call(
        &client,
        "resolve_dev_feedback",
        json!({"id": id, "reason": "ferramenta de metas criada na 0.9.6"}),
    )
    .await;
    assert_eq!(r.is_error, Some(false), "{r:?}");
    assert_eq!(data(&r)["status"], "resolvido");
    assert!(
        data(&r)["body"]
            .as_str()
            .unwrap()
            .ends_with("Resolvido por cli-teste via stdio: ferramenta de metas criada na 0.9.6"),
        "{r:?}"
    );
    let r = call(
        &client,
        "resolve_dev_feedback",
        json!({"id": id, "reason": "de novo"}),
    )
    .await;
    assert_eq!(r.is_error, Some(true), "{r:?}");
    // No proposal in the queue.
    assert_eq!(api.call("ai.pending_count", json!({})).unwrap(), 0);

    set_dev(&api, false);
    let r = call(&client, "list_dev_feedback", json!({})).await;
    assert_eq!(data(&r)["error"]["code"], "forbidden");
    client.cancel().await.unwrap();
}

/// An MCP client triages the whole backlog without opening the screen: board, edit, move, reorder,
/// merge duplicate, export; discard and delete become a proposal.
#[tokio::test]
async fn external_client_manages_the_whole_board() {
    let (_d, api) = vault();
    super::tests::allow_keyless(&api);
    set_dev(&api, true);
    let client = connect(TreasuryMcp::new(api.clone(), "stdio")).await;
    let mut ids = vec![];
    for (title, area) in [
        ("Tela do quadro confusa", "ui"),
        ("Falta exportar feedback em Markdown", "mcp"),
        ("Gráfico de metas sem legenda", "ui"),
        ("Importação OFX ignora tarifa", "importacao"),
    ] {
        let r = call(
            &client,
            "add_dev_feedback",
            json!({"title": title, "body": "Exemplo inventado.", "area": area, "kind": "melhoria",
                   "acceptance_criteria": "Teste e2e com 15 itens.",
                   "code_pointers": ["apps/desktop/src/features/devfeedback/DevFeedbackScreen.tsx"],
                   "tool_calls": [{"tool": "list_dev_feedback", "outcome": "texto corrido"}]}),
        )
        .await;
        assert_eq!(r.is_error, Some(false), "{r:?}");
        ids.push(data(&r)["feedback"]["id"].as_str().unwrap().to_string());
    }
    let (a, b, c, d) = (&ids[0], &ids[1], &ids[2], &ids[3]);

    // Edit, prioritize and reorder.
    let r = call(
        &client,
        "update_dev_feedback",
        json!({"id": a, "proposal": "Quadro kanban.", "priority": 5, "assignee": "cli-teste",
               "labels": ["tela"]}),
    )
    .await;
    assert_eq!(r.is_error, Some(false), "{r:?}");
    assert_eq!(data(&r)["importance"], 5);
    assert_eq!(data(&r)["proposal"], "Quadro kanban.");
    let r = call(
        &client,
        "set_dev_feedback_priority",
        json!({"id": d, "position": 1}),
    )
    .await;
    assert_eq!(data(&r)["position"], 1);
    let r = call(&client, "set_dev_feedback_priority", json!({"id": d})).await;
    assert_eq!(r.is_error, Some(true), "{r:?}");

    // Move to "Em andamento" (in progress), with a reason.
    let r = call(
        &client,
        "set_dev_feedback_status",
        json!({"id": a, "status": "em_andamento", "reason": "começando pelo quadro"}),
    )
    .await;
    assert_eq!(data(&r)["status"], "em_andamento");

    // Merge duplicate: the reinforcements go to the main one.
    let r = call(
        &client,
        "link_dev_feedback",
        json!({"id": c, "related_id": a, "kind": "duplicado"}),
    )
    .await;
    assert_eq!(data(&r)["merged"], true, "{r:?}");
    assert_eq!(data(&r)["feedback"]["status"], "descartado");
    assert_eq!(data(&r)["related"]["importance"], 6);

    // The board: columns by status, in queue order.
    let r = call(&client, "list_dev_feedback", json!({"group_by": "status"})).await;
    let board = data(&r);
    assert_eq!(board["count"], 4);
    let col = |s: &str| {
        board["columns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["status"] == s)
            .unwrap()
            .clone()
    };
    assert_eq!(col("aberto")["items"][0]["id"], json!(d));
    assert_eq!(col("em_andamento")["count"], 1);
    assert_eq!(col("descartado")["count"], 1);
    let r = call(
        &client,
        "list_dev_feedback",
        json!({"min_priority": 5, "assignee": "cli-teste"}),
    )
    .await;
    assert_eq!(data(&r)["count"], 1);

    // See a whole item, with links and history.
    let r = call(&client, "get_dev_feedback", json!({"id": a})).await;
    let full = data(&r);
    assert_eq!(full["feedback"]["links"][0]["kind"], "duplicado");
    assert!(full["history"].as_array().unwrap().len() >= 3, "{full}");

    // Markdown for the coding agent.
    let r = call(&client, "export_dev_feedback", json!({"id": a})).await;
    let md = data(&r)["markdown"].as_str().unwrap().to_string();
    assert!(md.starts_with("# Tela do quadro confusa\n"), "{md}");
    assert!(
        md.contains("## Critério de aceite\n\nTeste e2e com 15 itens."),
        "{md}"
    );
    assert!(md.contains("`apps/desktop/src/features/devfeedback/DevFeedbackScreen.tsx`"));
    assert!(md.contains("`list_dev_feedback`: texto corrido"), "{md}");
    assert!(md.contains(&format!("`id` = `{a}`")));
    let r = call(&client, "export_dev_feedback", json!({})).await;
    assert_eq!(data(&r)["count"], 3);

    // Discard and delete become a proposal; nothing changes until approval.
    assert_eq!(api.call("ai.pending_count", json!({})).unwrap(), 0);
    let r = call(
        &client,
        "set_dev_feedback_status",
        json!({"id": b, "status": "descartado", "reason": "já existe"}),
    )
    .await;
    assert_eq!(data(&r)["status"], "pending_approval", "{r:?}");
    let r = call(&client, "delete_dev_feedback", json!({"id": d})).await;
    assert_eq!(data(&r)["status"], "pending_approval", "{r:?}");
    assert_eq!(api.call("ai.pending_count", json!({})).unwrap(), 2);
    let still = api.call("devfeedback.get", json!({"id": b})).unwrap();
    assert_eq!(still["feedback"]["status"], "aberto");
    api.call("devfeedback.get", json!({"id": d})).unwrap();

    // Request for a tool that already exists: the answer points to the existing ones.
    let r = call(
        &client,
        "add_dev_feedback",
        json!({"title": "Falta ferramenta para quitar empréstimo antes do prazo",
               "body": "Exemplo inventado.", "area": "mcp", "kind": "ferramenta_faltando"}),
    )
    .await;
    let out = data(&r);
    let found: Vec<&str> = out["existing_tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(found.contains(&"simulate_loan_prepayment"), "{found:?}");
    assert!(found.contains(&"plan_debt_payoff"), "{found:?}");
    let text = format!("{r:?}");
    assert!(text.contains("simulate_loan_prepayment"), "{text}");
    // A bug gets no suggestion.
    let r = call(
        &client,
        "add_dev_feedback",
        json!({"title": "Erro interno ao simular quitação de empréstimo",
               "body": "Exemplo inventado.", "area": "mcp", "kind": "bug"}),
    )
    .await;
    assert!(data(&r).get("existing_tools").is_none());

    // Resolve and see what closed since the last lookup.
    let since = crate::time::now_utc();
    call(
        &client,
        "resolve_dev_feedback",
        json!({"id": a, "reason": "quadro na feat/feedback-quadro"}),
    )
    .await;
    let r = call(&client, "list_dev_feedback", json!({"closed_since": since})).await;
    assert_eq!(data(&r)["count"], 1);
    assert_eq!(data(&r)["feedback"][0]["id"], json!(a));
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn app_agent_follows_developer_mode_and_the_prompt_mentions_it() {
    let (_d, api) = vault();
    let job = api.ai_runtime().new_job(JobKind::Chat, AiTask::Chat, None);
    let client = connect(TreasuryMcp::for_agent(
        api.clone(),
        "agente (CLI de teste)",
        job.id.clone(),
    ))
    .await;
    assert!(!names(&client).await.contains(&"add_dev_feedback".into()));
    assert!(!inproc::agent_access(&api).dev);
    set_dev(&api, true);
    assert!(names(&client).await.contains(&"add_dev_feedback".into()));
    assert!(inproc::agent_access(&api).dev);
    let r = call(
        &client,
        "add_dev_feedback",
        json!({"title": "Resultado de get_dashboard confuso", "body": "Campo sem moeda.",
               "area": "dados", "kind": "dados"}),
    )
    .await;
    assert_eq!(r.is_error, Some(false), "{r:?}");
    let list = api.call("devfeedback.list", json!({})).unwrap();
    assert_eq!(list[0]["created_by"], "cli-teste via agente (CLI de teste)");
    client.cancel().await.unwrap();

    // In process: the tool only goes through with developer mode access.
    let actor = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let o = inproc::call_tool(
        &api,
        &actor,
        "list_dev_feedback",
        json!({}),
        inproc::AGENT_ACCESS,
    );
    assert!(o.is_error);
    let o = inproc::call_tool(
        &api,
        &actor,
        "list_dev_feedback",
        json!({}),
        inproc::agent_access(&api),
    );
    assert!(!o.is_error, "{}", o.content);
    assert!(o.content.starts_with("1 feedback."), "{}", o.content);
}
