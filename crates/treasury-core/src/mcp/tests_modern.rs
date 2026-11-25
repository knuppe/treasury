//! MCP 2026-07-28 (no `initialize`, per-request `_meta`, stateless HTTP) next to the legacy
//! session-based era: version pinning, discovery, and the guarantees every client keeps
//! (developer-mode gating, writes as proposals with diff, undo and audit, API key per request,
//! loopback and browser rules). Requests are raw JSON-RPC, as a client of either era sends them.

use std::sync::Arc;
use std::time::Duration;

use rmcp::model::ProtocolVersion;
use serde_json::{Value, json};
use tokio::io::{
    AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, Lines, ReadHalf, WriteHalf,
};

use super::keys::{McpKeyCreateArgs, McpScope};
use super::*;
use crate::api::Api;
use crate::paths::TreasuryPaths;

const MODERN: &str = "2026-07-28";
const VERSIONS: [&str; 5] = [
    "2024-11-05",
    "2025-03-26",
    "2025-06-18",
    "2025-11-25",
    "2026-07-28",
];

fn vault() -> (tempfile::TempDir, SharedApi) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Arc::new(Api::new_unlocked_process(paths).unwrap());
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
    )
    .unwrap();
    (dir, api)
}

/// The `_meta` a 2026-07-28 client sends with every request.
fn meta(client: &str) -> Value {
    meta_for(MODERN, client)
}

fn meta_for(version: &str, client: &str) -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": version,
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/clientInfo": {"name": client, "version": "1.0"}
    })
}

fn tool_names(list: &Value) -> Vec<String> {
    list["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("no tools: {list}"))
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect()
}

fn set_dev(api: &Api, on: bool) {
    api.call("settings.set", json!({"developer_tools": on}))
        .unwrap();
}

/// Raw JSON-RPC over an in-memory pipe, like `treasury mcp` over stdin/stdout.
struct Stdio {
    lines: Lines<BufReader<ReadHalf<DuplexStream>>>,
    out: WriteHalf<DuplexStream>,
    next_id: i64,
}

impl Stdio {
    fn start(server: TreasuryMcp) -> Self {
        let (server_io, client_io) = tokio::io::duplex(1 << 20);
        tokio::spawn(async move {
            let _ = server.serve_transport(server_io).await;
        });
        let (rd, out) = tokio::io::split(client_io);
        Self {
            lines: BufReader::new(rd).lines(),
            out,
            next_id: 0,
        }
    }

    /// Sends a request and waits for its response (skipping anything else).
    async fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        self.out
            .write_all(format!("{msg}\n").as_bytes())
            .await
            .unwrap();
        loop {
            let line = tokio::time::timeout(Duration::from_secs(20), self.lines.next_line())
                .await
                .expect("response in time")
                .unwrap()
                .expect("server still open");
            let v: Value = serde_json::from_str(&line).unwrap();
            if v["id"] == id {
                return v;
            }
        }
    }

    async fn call(&mut self, client: &str, tool: &str, args: Value) -> Value {
        let r = self
            .request(
                "tools/call",
                json!({"name": tool, "arguments": args, "_meta": meta(client)}),
            )
            .await;
        r["result"].clone()
    }
}

#[test]
fn protocol_versions_are_pinned() {
    let supported: Vec<&str> = supported_protocol_versions()
        .iter()
        .map(ProtocolVersion::as_str)
        .collect();
    assert_eq!(supported, VERSIONS);
    // A newer revision in the SDK is not served by accident: raising MAX_PROTOCOL_VERSION means
    // reading its changelog, extending these tests and docs/architecture.md.
    assert_eq!(
        ProtocolVersion::KNOWN_VERSIONS.last(),
        Some(&MAX_PROTOCOL_VERSION),
        "the MCP SDK knows a newer protocol revision; review it before raising \
         MAX_PROTOCOL_VERSION"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn modern_client_discovers_the_server_and_its_versions() {
    let (_d, api) = vault();
    super::tests::allow_keyless(&api);
    let mut c = Stdio::start(TreasuryMcp::new(api.clone(), "stdio"));
    let d = c
        .request("server/discover", json!({"_meta": meta("agente-novo")}))
        .await;
    let r = &d["result"];
    assert_eq!(r["supportedVersions"], json!(VERSIONS), "{d}");
    assert_eq!(r["resultType"], "complete");
    let info = &r["_meta"]["io.modelcontextprotocol/serverInfo"];
    assert_eq!(info["name"], SERVER_NAME);
    assert!(
        !info["description"].as_str().unwrap().contains("read-only"),
        "{info}"
    );
    assert!(
        r["instructions"]
            .as_str()
            .unwrap()
            .contains("integers in cents")
    );
    assert!(r["capabilities"]["tools"].is_object());
    // An unknown revision is refused with the list to retry with.
    let bad = c
        .request(
            "tools/list",
            json!({"_meta": meta_for("2099-01-01", "agente-novo")}),
        )
        .await;
    assert_eq!(bad["error"]["code"], -32022, "{bad}");
    assert_eq!(bad["error"]["data"]["supported"], json!(VERSIONS));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_clients_still_negotiate_with_initialize() {
    let (_d, api) = vault();
    super::tests::allow_keyless(&api);
    for (asked, agreed) in [
        ("2024-11-05", "2024-11-05"),
        ("2025-06-18", "2025-06-18"),
        ("2025-11-25", "2025-11-25"),
        // `initialize` naming a stateless revision falls back to the newest one with a handshake.
        (MODERN, "2025-11-25"),
    ] {
        let mut c = Stdio::start(TreasuryMcp::new(api.clone(), "stdio"));
        let init = c
            .request(
                "initialize",
                json!({"protocolVersion": asked, "capabilities": {},
                       "clientInfo": {"name": "legado", "version": "1"}}),
            )
            .await;
        assert_eq!(init["result"]["protocolVersion"], agreed, "{init}");
        assert_eq!(init["result"]["serverInfo"]["name"], SERVER_NAME);
        let list = c.request("tools/list", json!({})).await;
        assert!(tool_names(&list).contains(&"list_accounts".to_string()));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn modern_tool_list_is_private_stable_and_gates_developer_tools() {
    let (_d, api) = vault();
    super::tests::allow_keyless(&api);
    let mut c = Stdio::start(TreasuryMcp::new(api.clone(), "stdio"));
    let list = c
        .request("tools/list", json!({"_meta": meta("agente-novo")}))
        .await;
    // The list depends on the key, the settings and developer mode: never shared or cached.
    assert_eq!(
        list["result"]["cacheScope"], "private",
        "{}",
        list["result"]["cacheScope"]
    );
    assert_eq!(list["result"]["ttlMs"], 0);
    let names = tool_names(&list);
    let again = c
        .request("tools/list", json!({"_meta": meta("agente-novo")}))
        .await;
    assert_eq!(names, tool_names(&again), "deterministic order");
    assert!(names.iter().all(|n| !n.contains("dev_feedback")));

    let feedback = json!({"title": "Falta ferramenta de metas", "body": "Precisei e não havia.",
                          "area": "mcp", "kind": "ferramenta_faltando"});
    let r = c
        .call("agente-novo", "add_dev_feedback", feedback.clone())
        .await;
    assert_eq!(r["structuredContent"]["error"]["code"], "forbidden", "{r}");

    set_dev(&api, true);
    let list = c
        .request("tools/list", json!({"_meta": meta("agente-novo")}))
        .await;
    assert!(tool_names(&list).contains(&"add_dev_feedback".to_string()));
    let r = c
        .call("agente-novo", "add_dev_feedback", feedback.clone())
        .await;
    assert_eq!(r["isError"], false, "{r}");
    // Authorship comes from the per-request client info (there is no `initialize`).
    assert_eq!(
        r["structuredContent"]["feedback"]["created_by"],
        "agente-novo via stdio"
    );

    set_dev(&api, false);
    let r = c.call("agente-novo", "list_dev_feedback", json!({})).await;
    assert_eq!(r["structuredContent"]["error"]["code"], "forbidden", "{r}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn modern_writes_become_proposals_with_diff_undo_and_audit() {
    let (_d, api) = vault();
    super::tests::allow_keyless(&api);
    let mut c = Stdio::start(TreasuryMcp::new(api.clone(), "stdio"));
    let goal = json!({"name": "Viagem", "kind": "travel", "target_amount": 500000});
    // Writes off: refused, and the refusal is audited.
    let r = c.call("agente-novo", "create_goal", goal.clone()).await;
    assert_eq!(r["structuredContent"]["error"]["code"], "forbidden", "{r}");

    api.call("ai.settings.set", json!({"mcp_allow_writes": true}))
        .unwrap();
    let r = c.call("agente-novo", "create_goal", goal).await;
    assert_eq!(r["isError"], false, "{r}");
    let data = &r["structuredContent"];
    assert_eq!(data["status"], "pending_approval", "{data}");
    let p = api
        .call("ai.get_proposal", json!({"id": data["proposal_id"]}))
        .unwrap();
    assert_eq!(p["client"], "agente-novo via stdio");
    assert!(!p["items"].as_array().unwrap().is_empty(), "diff: {p}");
    let goals = |api: &Api| {
        api.call("goals.list", json!({}))
            .unwrap()
            .as_array()
            .map_or(0, Vec::len)
    };
    assert_eq!(goals(&api), 0, "nothing changes before approval");

    api.call("ai.approve", json!({"ids": [data["proposal_id"]]}))
        .unwrap();
    assert_eq!(goals(&api), 1);
    api.call("changes.undo", json!({"id": p["change_set_id"]}))
        .unwrap();
    assert_eq!(goals(&api), 0, "undone through the history");

    let audit = api.call("mcp.audit_list", json!({})).unwrap();
    let rows: Vec<&Value> = audit
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["tool"] == "create_goal")
        .collect();
    assert_eq!(rows.len(), 2, "{audit}");
    assert!(rows.iter().all(|a| a["client"] == "agente-novo via stdio"));
    assert!(rows.iter().any(|a| a["ok"] == false));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn modern_socket_client_keeps_its_key_scopes() {
    let (_d, api) = vault();
    let (k, grant) = key(&api, &[]);
    let mut c = Stdio::start(TreasuryMcp::for_key(
        api.clone(),
        "socket",
        crate::vault::Secret::new(k),
        Some(grant),
    ));
    let names = tool_names(
        &c.request("tools/list", json!({"_meta": meta("agente-novo")}))
            .await,
    );
    assert!(names.contains(&"list_accounts".to_string()));
    assert!(!names.contains(&"create_goal".to_string()));
    let r = c.call("agente-novo", "list_accounts", json!({})).await;
    assert_eq!(r["isError"], false, "{r}");
    let audit = api.call("mcp.audit_list", json!({})).unwrap();
    assert_eq!(
        audit[0]["client"], "agente-novo via socket · chave Agente externo",
        "{audit}"
    );
}

fn key(api: &Api, scopes: &[McpScope]) -> (String, keys::KeyGrant) {
    let made = api
        .call(
            "mcp.key_create",
            serde_json::to_value(McpKeyCreateArgs {
                name: "Agente externo".into(),
                scopes: scopes.to_vec(),
                expires_days: None,
            })
            .unwrap(),
        )
        .unwrap();
    let k = made["key"].as_str().unwrap().to_string();
    let g = api
        .with_session(|s, _| keys::verify(s.db.conn(), &k, false))
        .unwrap()
        .unwrap();
    (k, g)
}

struct HttpReply {
    status: u16,
    session: Option<String>,
    body: Value,
}

/// One JSON-RPC POST to the local server; the reply may come as SSE (`data: {...}`).
fn post(port: u16, key: Option<&str>, headers: &[(&str, &str)], body: &Value) -> HttpReply {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let mut req = agent
        .post(&format!("http://127.0.0.1:{port}/mcp"))
        .header("Accept", "application/json, text/event-stream")
        .header("Content-Type", "application/json");
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    if let Some(k) = key {
        req = req.header("Authorization", &format!("Bearer {k}"));
    }
    let mut resp = req.send(body.to_string()).unwrap();
    let status = resp.status().as_u16();
    let session = resp
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let text = resp.body_mut().read_to_string().unwrap_or_default();
    let body = text
        .lines()
        .filter_map(|l| l.strip_prefix("data:"))
        .filter_map(|l| serde_json::from_str::<Value>(l.trim()).ok())
        .find(|v| v.get("id").is_some())
        .or_else(|| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null);
    HttpReply {
        status,
        session,
        body,
    }
}

/// The headers a 2026-07-28 client sends with each POST (SEP-2243).
fn headers<'a>(method: &'a str, name: Option<&'a str>) -> Vec<(&'a str, &'a str)> {
    let mut h = vec![("MCP-Protocol-Version", MODERN), ("Mcp-Method", method)];
    if let Some(n) = name {
        h.push(("Mcp-Name", n));
    }
    h
}

fn modern_request(id: i64, method: &str, params: Value, client: &str) -> Value {
    let mut params = params;
    params["_meta"] = meta(client);
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
}

fn http_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn http_modern_checks_the_key_and_headers_on_every_request() {
    let (_d, api) = vault();
    let (read, read_grant) = key(&api, &[]);
    let (write, _) = key(&api, &[McpScope::Write]);
    api.call("ai.settings.set", json!({"mcp_allow_writes": true}))
        .unwrap();
    let rt = http_runtime();
    let server = rt.block_on(http_server::start(api.clone(), 0)).unwrap();
    let port = server.addr.port();

    let discover = modern_request(1, "server/discover", json!({}), "http-novo");
    let h = headers("server/discover", None);
    assert_eq!(post(port, None, &h, &discover).status, 401);
    assert_eq!(post(port, Some("trsk_errada"), &h, &discover).status, 401);
    let r = post(port, Some(&read), &h, &discover);
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.body["result"]["supportedVersions"], json!(VERSIONS));
    assert_eq!(r.session, None, "2026-07-28 has no sessions");

    // SEP-2243 headers: required and checked against the body.
    let list = modern_request(2, "tools/list", json!({}), "http-novo");
    let r = post(port, Some(&read), &[], &list);
    assert_eq!(
        (r.status, &r.body["error"]["code"]),
        (400, &json!(-32020)),
        "{}",
        r.body
    );
    let r = post(port, Some(&read), &headers("tools/call", None), &list);
    assert_eq!(
        (r.status, &r.body["error"]["code"]),
        (400, &json!(-32020)),
        "{}",
        r.body
    );
    let call = |id, tool: &str, args: Value, client: &str| {
        modern_request(
            id,
            "tools/call",
            json!({"name": tool, "arguments": args}),
            client,
        )
    };
    let accounts = call(3, "list_accounts", json!({}), "http-novo");
    let r = post(port, Some(&read), &headers("tools/call", None), &accounts);
    assert_eq!(
        r.status, 400,
        "Mcp-Name is required for tools/call: {}",
        r.body
    );

    // Key scopes per request.
    let r = post(port, Some(&read), &headers("tools/list", None), &list);
    assert_eq!(r.status, 200);
    let names = tool_names(&r.body);
    assert!(names.contains(&"list_accounts".to_string()));
    assert!(!names.contains(&"create_goal".to_string()));
    assert!(names.iter().all(|n| !n.contains("dev_feedback")));
    let r = post(
        port,
        Some(&read),
        &headers("tools/call", Some("list_accounts")),
        &accounts,
    );
    assert_eq!(r.body["result"]["isError"], false, "{}", r.body);

    let goal = call(
        4,
        "create_goal",
        json!({"name": "Viagem", "kind": "travel", "target_amount": 500000}),
        "http-escrita",
    );
    let r = post(
        port,
        Some(&write),
        &headers("tools/call", Some("create_goal")),
        &goal,
    );
    let data = &r.body["result"]["structuredContent"];
    assert_eq!(data["status"], "pending_approval", "{}", r.body);
    let p = api
        .call("ai.get_proposal", json!({"id": data["proposal_id"]}))
        .unwrap();
    assert_eq!(p["client"], "http-escrita via http · chave Agente externo");
    let audit = api.call("mcp.audit_list", json!({})).unwrap().to_string();
    assert!(
        audit.contains("http-novo via http · chave Agente externo"),
        "{audit}"
    );

    // Developer tools follow developer mode for key clients too.
    set_dev(&api, true);
    let r = post(port, Some(&read), &headers("tools/list", None), &list);
    assert!(tool_names(&r.body).contains(&"add_dev_feedback".to_string()));
    set_dev(&api, false);

    // A revoked key fails on the very next request: there is no session to outlive it.
    api.call("mcp.key_revoke", json!({"id": read_grant.id}))
        .unwrap();
    assert_eq!(
        post(port, Some(&read), &headers("tools/list", None), &list).status,
        401
    );

    // No server-to-client stream in 2026-07-28.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let get = agent
        .get(&format!("http://127.0.0.1:{port}/mcp"))
        .header("Authorization", &format!("Bearer {write}"))
        .header("Accept", "text/event-stream")
        .header("MCP-Protocol-Version", MODERN)
        .call()
        .unwrap();
    assert_eq!(get.status().as_u16(), 405);
    rt.block_on(server.shutdown());
}

#[test]
fn http_modern_loopback_without_key_keeps_the_browser_rule() {
    let (_d, api) = vault();
    let rt = http_runtime();
    let server = rt.block_on(http_server::start(api.clone(), 0)).unwrap();
    let port = server.addr.port();
    let list = modern_request(1, "tools/list", json!({}), "script-novo");
    let h = headers("tools/list", None);
    assert_eq!(post(port, None, &h, &list).status, 401, "off by default");

    set_dev(&api, true);
    let mut s = api.call("mcp.server_settings", json!({})).unwrap();
    s["http_loopback_no_auth"] = json!(true);
    s["require_api_key"] = json!(false);
    api.call("mcp.set_server_settings", s).unwrap();
    for browser in [
        ("Origin", "https://golpe.example"),
        ("Sec-Fetch-Mode", "cors"),
        ("Referer", "https://golpe.example/"),
    ] {
        let mut hb = h.clone();
        hb.push(browser);
        assert_eq!(post(port, None, &hb, &list).status, 403, "{browser:?}");
    }
    let r = post(port, None, &h, &list);
    assert_eq!(r.status, 200, "{}", r.body);
    assert!(!tool_names(&r.body).contains(&"create_goal".to_string()));
    let accounts = modern_request(
        2,
        "tools/call",
        json!({"name": "list_accounts", "arguments": {}}),
        "script-novo",
    );
    let r = post(
        port,
        None,
        &headers("tools/call", Some("list_accounts")),
        &accounts,
    );
    assert_eq!(r.status, 200, "{}", r.body);
    let audit = api.call("mcp.audit_list", json!({})).unwrap();
    assert_eq!(
        audit[0]["client"], "script-novo via http · sem chave (loopback)",
        "{audit}"
    );
    // Turning developer mode off closes the exception on the next request.
    set_dev(&api, false);
    assert_eq!(post(port, None, &h, &list).status, 401);
    rt.block_on(server.shutdown());
}

#[test]
fn http_serves_legacy_and_modern_clients_at_once() {
    let (_d, api) = vault();
    let (k, _) = key(&api, &[]);
    let rt = http_runtime();
    let server = rt.block_on(http_server::start(api.clone(), 0)).unwrap();
    let port = server.addr.port();

    let init = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-11-25", "capabilities": {},
                   "clientInfo": {"name": "legado", "version": "1"}}});
    let r = post(port, Some(&k), &[], &init);
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.body["result"]["protocolVersion"], "2025-11-25");
    let sid = r.session.expect("legacy clients keep their session");
    let session = [("Mcp-Session-Id", sid.as_str())];
    let _ = post(
        port,
        Some(&k),
        &session,
        &json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    );

    let modern = modern_request(2, "tools/list", json!({}), "http-novo");
    let r = post(port, Some(&k), &headers("tools/list", None), &modern);
    assert_eq!(r.status, 200, "{}", r.body);
    let modern_names = tool_names(&r.body);

    let legacy = json!({"jsonrpc": "2.0", "id": 3, "method": "tools/list"});
    let r = post(port, Some(&k), &session, &legacy);
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(tool_names(&r.body), modern_names, "same tools in both eras");

    let call = json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call",
        "params": {"name": "list_accounts", "arguments": {}}});
    let r = post(port, Some(&k), &session, &call);
    assert_eq!(r.body["result"]["isError"], false, "{}", r.body);
    let audit = api.call("mcp.audit_list", json!({})).unwrap();
    assert_eq!(audit[0]["client"], "legado via http · chave Agente externo");
    rt.block_on(server.shutdown());
}
