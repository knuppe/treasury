//! `treasury-dev mcp` (o mesmo `run_mcp_cli` do `treasury mcp`) como processo filho,
//! falando JSON-RPC por stdin/stdout: proxy para o socket do "app" e modo sem acesso.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use serde_json::{Value, json};
use treasury_core::keychain::{MemoryStore, set_store};
use treasury_core::paths::TreasuryPaths;
use treasury_core::{Api, mcp};

struct McpChild {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: mpsc::Receiver<String>,
}

impl McpChild {
    fn spawn(home: &std::path::Path) -> Self {
        Self::spawn_with_key(home, None)
    }

    /// Com `key`, o cliente informa a chave de API em `TREASURY_API_KEY`.
    fn spawn_with_key(home: &std::path::Path, key: Option<&str>) -> Self {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_treasury-dev"));
        cmd.args(["mcp", "--home"])
            .arg(home)
            .env("RUST_LOG", "warn")
            .env("TREASURY_KEYCHAIN", "off")
            .env_remove("TREASURY_API_KEY");
        if let Some(k) = key {
            cmd.env("TREASURY_API_KEY", k);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Self {
            stdin: child.stdin.take(),
            child,
            lines: rx,
        }
    }

    fn send(&mut self, msg: Value) {
        let stdin = self.stdin.as_mut().unwrap();
        writeln!(stdin, "{msg}").unwrap();
        stdin.flush().unwrap();
    }

    /// Lê até a resposta com este `id` (ignora notificações).
    fn response(&self, id: i64) -> Value {
        loop {
            let line = self
                .lines
                .recv_timeout(Duration::from_secs(20))
                .expect("resposta do treasury-dev mcp");
            let v: Value = serde_json::from_str(&line).unwrap();
            if v["id"] == id {
                return v;
            }
        }
    }

    fn initialize(&mut self) -> Value {
        self.send(json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "teste-e2e", "version": "1.0"}
            }
        }));
        let r = self.response(1);
        self.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        r
    }

    fn call_tool(&mut self, id: i64, name: &str, args: Value) -> Value {
        self.send(json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }));
        self.response(id)["result"].clone()
    }

    /// Fecha o stdin e espera o processo terminar sozinho.
    fn finish(mut self) {
        drop(self.stdin.take());
        for _ in 0..100 {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success(), "saída {status}");
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = self.child.kill();
        panic!("treasury-dev mcp não terminou ao fechar o stdin");
    }
}

#[test]
fn without_app_or_keychain_tools_explain_how_to_unlock() {
    let dir = tempfile::tempdir().unwrap();
    let mut mcp = McpChild::spawn(&dir.path().join("home"));
    let init = mcp.initialize();
    assert_eq!(init["result"]["serverInfo"]["name"], "treasury");
    assert!(
        init["result"]["instructions"]
            .as_str()
            .unwrap()
            .contains("integers in cents")
    );
    mcp.send(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let tools = mcp.response(2);
    assert!(tools["result"]["tools"].as_array().unwrap().len() >= 27);
    let r = mcp.call_tool(3, "list_accounts", json!({}));
    assert_eq!(r["isError"], true);
    assert!(
        r["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Nenhum cofre")
    );
    mcp.finish();
}

#[test]
fn proxies_to_the_app_socket() {
    set_store(Arc::new(MemoryStore::default()));
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let paths = TreasuryPaths::with_root(&home).unwrap();
    // O "app": API com a trava exclusiva e o socket MCP.
    let api = Arc::new(Api::new(paths.clone()).unwrap());
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
    )
    .unwrap();
    // A chave de API é sempre exigida: o cliente recebe uma, como na tela de Integrações MCP.
    let made = api
        .call(
            "mcp.key_create",
            json!({"name": "Cliente de teste", "scopes": ["read"]}),
        )
        .unwrap();
    let key = made["key"].as_str().unwrap().to_string();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let socket = rt
        .block_on(mcp::start_socket_server(api.clone(), &paths.run_dir))
        .unwrap();

    let mut child = McpChild::spawn_with_key(&home, Some(&key));
    let init = child.initialize();
    assert_eq!(init["result"]["serverInfo"]["name"], "treasury");
    let r = child.call_tool(2, "get_change_history", json!({"limit": 3}));
    assert_eq!(r["isError"], false, "{r}");
    assert_eq!(r["structuredContent"]["count"], 1);

    // Bloqueado no app: o proxy repassa o erro amigável.
    api.call("vault.lock", Value::Null).unwrap();
    let r = child.call_tool(3, "get_change_history", json!({}));
    assert_eq!(r["isError"], true);
    assert_eq!(r["content"][0]["text"], mcp::locked_message());
    child.finish();

    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
    let audit = api.call("mcp.audit_list", json!({})).unwrap();
    let entries = audit.as_array().unwrap();
    assert!(
        entries
            .iter()
            .all(|e| e["client"] == "teste-e2e via treasury mcp · chave Cliente de teste"),
        "{audit}"
    );
    assert_eq!(entries.len(), 2);
    rt.block_on(socket.shutdown());
}

/// App aberto exigindo chave (padrão desde a V139) e cliente sem chave: a resposta diz que a
/// chave é exigida, não que o app está fechado.
#[test]
fn app_open_without_key_says_the_key_is_required() {
    set_store(Arc::new(MemoryStore::default()));
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let paths = TreasuryPaths::with_root(&home).unwrap();
    let api = Arc::new(Api::new(paths.clone()).unwrap());
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
    )
    .unwrap();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let socket = rt
        .block_on(mcp::start_socket_server(api.clone(), &paths.run_dir))
        .unwrap();

    let mut child = McpChild::spawn(&home);
    child.initialize();
    let r = child.call_tool(2, "list_accounts", json!({}));
    assert_eq!(r["isError"], true, "{r}");
    let text = r["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("exige chave de API"), "{text}");
    assert!(text.contains("TREASURY_API_KEY"), "{text}");
    assert!(!text.contains("fechado"), "{text}");
    child.finish();
    rt.block_on(socket.shutdown());
}
