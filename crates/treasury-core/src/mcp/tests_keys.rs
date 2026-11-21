//! External clients with an API key: scopes on the socket/stdio and on the local HTTP server.

use std::sync::Arc;

use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientConfig, Implementation};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};
use serde_json::{Value, json};

use super::keys::{McpKeyCreateArgs, McpScope};
use super::*;
use crate::api::Api;
use crate::paths::TreasuryPaths;

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

async fn connect(server: TreasuryMcp) -> RunningService<RoleClient, ClientConfig> {
    let (server_io, client_io) = tokio::io::duplex(1 << 16);
    tokio::spawn(async move {
        let _ = server.serve_transport(server_io).await;
    });
    ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new("cliente_externo", "1.0"),
    )
    .serve(client_io)
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn key_scopes_decide_tools_and_writes_become_proposals() {
    let (_d, api) = vault();
    let (read_key, read) = key(&api, &[]);
    let (write_key, write) = key(&api, &[McpScope::Write]);
    let ro = connect(TreasuryMcp::for_key(
        api.clone(),
        "socket",
        crate::vault::Secret::new(read_key),
        Some(read),
    ))
    .await;
    let names: Vec<String> = ro
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert!(names.contains(&"list_accounts".to_string()));
    assert!(!names.contains(&"create_transaction".to_string()));
    let rw = connect(TreasuryMcp::for_key(
        api.clone(),
        "socket",
        crate::vault::Secret::new(write_key),
        Some(write),
    ))
    .await;
    let names: Vec<String> = rw
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert!(names.contains(&"create_goal".to_string()));
    let mut p = CallToolRequestParams::new("create_goal");
    p.arguments = json!({"name": "Viagem", "kind": "travel", "target_amount": 500000})
        .as_object()
        .cloned();
    let r = rw.call_tool(p).await.unwrap();
    let text = serde_json::to_string(&r.structured_content).unwrap();
    assert!(text.contains("pending_approval"), "{text}");
    let proposals = api.call("ai.list_proposals", json!({})).unwrap();
    assert!(
        proposals.to_string().contains("chave Agente externo"),
        "{proposals}"
    );
}

fn text_of(r: &rmcp::model::CallToolResult) -> String {
    serde_json::to_string(&r.content).unwrap()
}

async fn call(
    c: &RunningService<RoleClient, ClientConfig>,
    tool: &str,
) -> rmcp::model::CallToolResult {
    c.call_tool(CallToolRequestParams::new(tool.to_string()))
        .await
        .unwrap()
}

/// An open connection does not hold the key: revoked, deleted or from before the unlock, it is
/// checked again on every call (security review of 2026-10-25).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_sessions_recheck_the_key_on_every_call() {
    let (_d, api) = vault();
    let (k, g) = key(&api, &[]);
    let c = connect(TreasuryMcp::for_key(
        api.clone(),
        "socket",
        crate::vault::Secret::new(k.clone()),
        Some(g.clone()),
    ))
    .await;
    assert_eq!(call(&c, "list_accounts").await.is_error, Some(false));
    api.call("mcp.key_revoke", json!({"id": g.id})).unwrap();
    let r = call(&c, "list_accounts").await;
    assert_eq!(r.is_error, Some(true));
    assert!(
        text_of(&r).contains(&crate::i18n::t("mcp-chave-invalida")),
        "{}",
        text_of(&r)
    );
    // Reading a resource is also refused.
    assert!(
        c.read_resource(rmcp::model::ReadResourceRequestParams::new(
            "treasury://accounts"
        ))
        .await
        .is_err()
    );

    // Connected with the vault locked: no check yet, and it holds after the unlock.
    let (k2, _) = key(&api, &[]);
    api.call("vault.lock", Value::Null).unwrap();
    let late = connect(TreasuryMcp::for_key(
        api.clone(),
        "socket",
        crate::vault::Secret::new(k2),
        None,
    ))
    .await;
    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
    assert_eq!(call(&late, "list_accounts").await.is_error, Some(false));
    // A key that does not exist in this vault never gets in, even connected with it locked.
    let fake = connect(TreasuryMcp::for_key(
        api.clone(),
        "socket",
        crate::vault::Secret::new("trsk_inventada".into()),
        None,
    ))
    .await;
    assert_eq!(call(&fake, "list_accounts").await.is_error, Some(true));
}

/// No key, with the key required (default), no tool answers, even if the connection came in
/// with the vault locked.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn keyless_sessions_get_nothing_when_the_key_is_required() {
    let (_d, api) = vault();
    let c = connect(TreasuryMcp::new(api.clone(), "socket")).await;
    let r = call(&c, "list_accounts").await;
    assert_eq!(r.is_error, Some(true));
    assert!(
        text_of(&r).contains(&crate::i18n::t("mcp-chave-exigida")),
        "{}",
        text_of(&r)
    );
}

#[test]
fn require_key_setting_and_bad_keys_are_refused() {
    let (_d, api) = vault();
    // Default (V139): key required; a newly created vault does not show the change notice.
    let s = api.call("mcp.server_settings", json!({})).unwrap();
    assert_eq!(
        (
            s["require_api_key"].clone(),
            s["require_key_notice"].clone()
        ),
        (json!(true), json!(false))
    );
    // The local socket only exists on macOS and Linux.
    #[cfg(unix)]
    assert!(socket::key_access(&api, None).is_err());
    // Turned off by the person: no key gets in; a fake key never.
    let mut off = s.clone();
    off["require_api_key"] = json!(false);
    api.call("mcp.set_server_settings", off).unwrap();
    #[cfg(unix)]
    {
        assert_eq!(socket::key_access(&api, None).unwrap(), None);
        assert!(socket::key_access(&api, Some("trsk_falsa")).is_err());
    }
    api.call("mcp.set_server_settings", s).unwrap();
    #[cfg(unix)]
    assert!(socket::key_access(&api, None).is_err());
    let (k, _) = key(&api, &[McpScope::Write]);
    #[cfg(unix)]
    {
        let g = socket::key_access(&api, Some(&k)).unwrap().unwrap();
        assert!(g.has(McpScope::Write) && g.has(McpScope::Read));
    }
    let _ = &k;
    // The AI does not manage keys.
    let ai = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert_eq!(
        api.call_as(&ai, "mcp.keys_list", json!({}))
            .unwrap_err()
            .code(),
        "forbidden"
    );
}

/// JSON-RPC over HTTP; the reply may come as SSE (`data: {...}`).
fn post(
    port: u16,
    key: Option<&str>,
    session: Option<&str>,
    body: Value,
) -> (u16, Option<String>, Value) {
    post_with(port, key, session, &[], body)
}

fn post_with(
    port: u16,
    key: Option<&str>,
    session: Option<&str>,
    extra: &[(&str, &str)],
    body: Value,
) -> (u16, Option<String>, Value) {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let mut req = agent
        .post(&format!("http://127.0.0.1:{port}/mcp"))
        .header("Accept", "application/json, text/event-stream")
        .header("Content-Type", "application/json");
    for (k, v) in extra {
        req = req.header(*k, *v);
    }
    if let Some(k) = key {
        req = req.header("Authorization", &format!("Bearer {k}"));
    }
    if let Some(s) = session {
        req = req.header("Mcp-Session-Id", s);
    }
    let mut resp = req.send(body.to_string()).unwrap();
    let status = resp.status().as_u16();
    let sid = resp
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let text = resp.body_mut().read_to_string().unwrap_or_default();
    let json = text
        .lines()
        .filter_map(|l| l.strip_prefix("data:"))
        .filter_map(|l| serde_json::from_str::<Value>(l.trim()).ok())
        .find(|v| v.get("id").is_some())
        .or_else(|| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null);
    (status, sid, json)
}

#[test]
fn http_server_requires_a_key_and_honors_scopes() {
    let (_d, api) = vault();
    let (k, _) = key(&api, &[]);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let server = rt.block_on(http_server::start(api.clone(), 0)).unwrap();
    let port = server.addr.port();
    let init = json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "cliente_http", "version": "1"}}
    });
    let (st, _, _) = post(port, None, None, init.clone());
    assert_eq!(st, 401);
    let (st, _, _) = post(port, Some("trsk_errada"), None, init.clone());
    assert_eq!(st, 401);
    let (st, sid, body) = post(port, Some(&k), None, init);
    assert_eq!(st, 200, "{body}");
    assert_eq!(body["result"]["serverInfo"]["name"], "treasury");
    let sid = sid.expect("session");
    let _ = post(
        port,
        Some(&k),
        Some(&sid),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    );
    let (st, _, list) = post(
        port,
        Some(&k),
        Some(&sid),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    );
    assert_eq!(st, 200);
    let names = list["result"]["tools"].to_string();
    assert!(
        names.contains("list_accounts") && !names.contains("create_transaction"),
        "{names}"
    );
    rt.block_on(server.shutdown());
}

/// No key, only with the setting on, only from loopback and never from a browser; access is that
/// of a local client with no key (read, writes per the settings).
#[test]
fn http_loopback_without_key_only_with_the_setting_and_outside_browsers() {
    let (_d, api) = vault();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let server = rt.block_on(http_server::start(api.clone(), 0)).unwrap();
    let port = server.addr.port();
    let init = json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "script-local", "version": "1"}}
    });
    // Default: off.
    let s = api.call("mcp.server_settings", json!({})).unwrap();
    assert_eq!(s["http_loopback_no_auth"], false);
    assert_eq!(post(port, None, None, init.clone()).0, 401);
    let mut s2 = s.clone();
    s2["http_loopback_no_auth"] = json!(true);
    // Loopback with no key only counts with "Exigir chave de API" turned off.
    s2["require_api_key"] = json!(false);
    // Outside developer mode, it does not turn on.
    let r = api.call("mcp.set_server_settings", s2.clone()).unwrap();
    assert_eq!(r["http_loopback_no_auth"], false);
    assert_eq!(post(port, None, None, init.clone()).0, 401);
    api.call("settings.set", json!({"developer_tools": true}))
        .unwrap();
    let r = api.call("mcp.set_server_settings", s2).unwrap();
    assert_eq!(r["http_loopback_no_auth"], true);
    // Browser (Origin / Sec-Fetch): rejected even on loopback.
    for h in [
        ("Origin", "https://golpe.example"),
        ("Sec-Fetch-Mode", "cors"),
        ("Referer", "https://golpe.example/"),
    ] {
        assert_eq!(
            post_with(port, None, None, &[h], init.clone()).0,
            403,
            "{h:?}"
        );
    }
    let (st, sid, body) = post(port, None, None, init);
    assert_eq!(st, 200, "{body}");
    let sid = sid.expect("session");
    let _ = post(
        port,
        None,
        Some(&sid),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    );
    let (st, _, list) = post(
        port,
        None,
        Some(&sid),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    );
    assert_eq!(st, 200);
    let names = list["result"]["tools"].to_string();
    assert!(
        names.contains("list_accounts") && !names.contains("create_transaction"),
        "{names}"
    );
    // An invalid key stays rejected (does not fall to keyless access).
    assert_eq!(
        post(
            port,
            Some("trsk_errada"),
            None,
            json!({"jsonrpc": "2.0", "id": 3, "method": "tools/list"})
        )
        .0,
        401
    );
    // "Exigir chave de API" overrides the exception.
    let mut s3 = api.call("mcp.server_settings", json!({})).unwrap();
    s3["require_api_key"] = json!(true);
    api.call("mcp.set_server_settings", s3.clone()).unwrap();
    let init2 = json!({"jsonrpc": "2.0", "id": 9, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "x", "version": "1"}}});
    assert_eq!(post(port, None, None, init2).0, 401);
    s3["require_api_key"] = json!(false);
    api.call("mcp.set_server_settings", s3).unwrap();
    // The audit marks the client with no key.
    let (st, _, _) = post(
        port,
        None,
        Some(&sid),
        json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call",
               "params": {"name": "list_accounts", "arguments": {}}}),
    );
    assert_eq!(st, 200);
    let audit = api.call("mcp.audit_list", json!({})).unwrap().to_string();
    assert!(audit.contains("sem chave (loopback)"), "{audit}");
    // Turning developer mode off turns loopback off immediately.
    let init3 = json!({"jsonrpc": "2.0", "id": 10, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "y", "version": "1"}}});
    api.call("settings.set", json!({"developer_tools": false}))
        .unwrap();
    let s4 = api.call("mcp.server_settings", json!({})).unwrap();
    assert_eq!(s4["http_loopback_no_auth"], false);
    assert_eq!(post(port, None, None, init3.clone()).0, 401);
    // An old setting that is on (recorded before this rule) does not open the vault outside
    // developer mode.
    api.with_session(|s, _| {
        crate::settings::set(s.db.conn(), http_server::LOOPBACK_NO_AUTH_SETTING, &true)
    })
    .unwrap();
    assert_eq!(post(port, None, None, init3).0, 401);
    assert_eq!(
        api.call("mcp.server_settings", json!({})).unwrap()["http_loopback_no_auth"],
        false
    );
    rt.block_on(server.shutdown());
}

/// `GET /events`: SSE with the same key as `/mcp`; a write becomes `data_changed`.
#[test]
fn http_events_stream_announces_writes() {
    use std::io::{BufRead, BufReader, Write};
    let (_d, api) = vault();
    let (k, _) = key(&api, &[]);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let server = rt.block_on(http_server::start(api.clone(), 0)).unwrap();
    let port = server.addr.port();
    let get = |auth: Option<&str>| {
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let auth = auth.map_or(String::new(), |k| format!("Authorization: Bearer {k}\r\n"));
        write!(
            s,
            "GET /events HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept: text/event-stream\r\n{auth}\r\n"
        )
        .unwrap();
        BufReader::new(s)
    };
    let mut first = String::new();
    get(None).read_line(&mut first).unwrap();
    assert!(first.contains("401"), "{first}");
    let mut r = get(Some(&k));
    let mut line = String::new();
    r.read_line(&mut line).unwrap();
    assert!(line.contains("200"), "{line}");
    // Waits for the headers and only then writes (the listener is already subscribed).
    loop {
        line.clear();
        r.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(150));
    api.call(
        "accounts.create",
        json!({"name": "SSE", "kind": "checking"}),
    )
    .unwrap();
    let mut seen = String::new();
    // Other tests write in parallel and also show up in the stream: reads until finding ours.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        line.clear();
        if r.read_line(&mut line).is_err() {
            break;
        }
        seen.push_str(&line);
        if seen.contains("\"cmd\":\"accounts.create\"") {
            break;
        }
    }
    assert!(seen.contains("event: data_changed"), "{seen}");
    assert!(seen.contains("\"cmd\":\"accounts.create\""), "{seen}");
    rt.block_on(server.shutdown());
}
