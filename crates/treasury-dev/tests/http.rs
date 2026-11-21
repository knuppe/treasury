//! Servidor HTTP de desenvolvimento: rota, token, envelope de erro e CORS.

use std::net::SocketAddr;
use std::sync::Arc;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use treasury_core::keychain::{MemoryStore, set_store};
use treasury_core::paths::TreasuryPaths;
use treasury_core::{Api, SharedApi};

const TOKEN: &str = "token-de-teste";

struct Server {
    addr: SocketAddr,
    _dir: tempfile::TempDir,
    _stop: tokio::sync::oneshot::Sender<()>,
}

async fn start() -> Server {
    // Nunca o keychain real nos testes.
    set_store(Arc::new(MemoryStore::default()));
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join("home")).unwrap();
    let api: SharedApi = Arc::new(Api::new_unlocked_process(paths).unwrap());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(treasury_dev::serve_http(listener, api, TOKEN, async {
        let _ = rx.await;
    }));
    Server {
        addr,
        _dir: dir,
        _stop: tx,
    }
}

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

async fn request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> Reply {
    let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    req.push_str(body);
    s.write_all(req.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).await.unwrap();
    let text = String::from_utf8(raw).unwrap();
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    Reply {
        status,
        headers,
        body: body.to_string(),
    }
}

async fn call(addr: SocketAddr, cmd: &str, args: Value) -> Reply {
    let auth = format!("Bearer {TOKEN}");
    request(
        addr,
        "POST",
        "/api",
        &[
            ("Authorization", &auth),
            ("Content-Type", "application/json"),
        ],
        &json!({"cmd": cmd, "args": args}).to_string(),
    )
    .await
}

#[tokio::test]
async fn token_is_required() {
    let s = start().await;
    let body = json!({"cmd": "vault.status"}).to_string();
    let r = request(
        s.addr,
        "POST",
        "/api",
        &[("Content-Type", "application/json")],
        &body,
    )
    .await;
    assert_eq!(r.status, 401);
    assert_eq!(r.json()["ok"], false);
    assert_eq!(r.json()["error"]["code"], "unauthorized");
    let r = request(
        s.addr,
        "POST",
        "/api",
        &[
            ("Authorization", "Bearer errado"),
            ("Content-Type", "application/json"),
        ],
        &body,
    )
    .await;
    assert_eq!(r.status, 401);
    // Health não exige token.
    let r = request(s.addr, "GET", "/api/health", &[], "").await;
    assert_eq!(r.status, 200);
    assert_eq!(r.json()["ok"], true);
}

#[tokio::test]
async fn dispatches_commands_with_envelope() {
    let s = start().await;
    let r = call(s.addr, "vault.status", Value::Null).await;
    assert_eq!(r.status, 200);
    assert_eq!(r.json(), json!({"ok": true, "data": r.json()["data"]}));
    assert_eq!(r.json()["data"]["state"], "no_vault");

    let r = call(
        s.addr,
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Casa Dev"}),
    )
    .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["state"], "unlocked");
    let r = call(s.addr, "household.get", json!({})).await;
    assert_eq!(r.json()["data"]["name"], "Casa Dev");

    // Erros estruturados, com status HTTP correspondente.
    let r = call(
        s.addr,
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "x"}),
    )
    .await;
    assert_eq!(r.status, 409);
    assert_eq!(r.json()["ok"], false);
    assert_eq!(r.json()["error"]["code"], "vault_exists");
    assert!(
        r.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("já existe")
    );
    let r = call(s.addr, "nada.x", json!({})).await;
    assert_eq!(r.status, 404);
    assert_eq!(r.json()["error"]["code"], "unknown_command");
    let r = call(s.addr, "settings.set", json!({"nao_existe": 1})).await;
    assert_eq!(r.status, 400);
    assert_eq!(r.json()["error"]["code"], "invalid");
    call(s.addr, "vault.lock", json!({})).await;
    let r = call(s.addr, "household.get", json!({})).await;
    assert_eq!(r.status, 423);
    assert_eq!(r.json()["error"]["code"], "vault_locked");

    // Corpo inválido.
    let auth = format!("Bearer {TOKEN}");
    let r = request(
        s.addr,
        "POST",
        "/api",
        &[
            ("Authorization", &auth),
            ("Content-Type", "application/json"),
        ],
        "{\"args\": 1}",
    )
    .await;
    assert_eq!(r.status, 400);
    assert_eq!(r.json()["error"]["code"], "invalid");
}

#[tokio::test]
async fn cors_only_for_the_vite_origins() {
    let s = start().await;
    for origin in ["http://localhost:1420", "http://127.0.0.1:1420"] {
        let r = request(
            s.addr,
            "OPTIONS",
            "/api",
            &[
                ("Origin", origin),
                ("Access-Control-Request-Method", "POST"),
                (
                    "Access-Control-Request-Headers",
                    "authorization,content-type",
                ),
            ],
            "",
        )
        .await;
        assert!(r.status < 300, "{origin}: {}", r.status);
        assert_eq!(r.header("access-control-allow-origin"), Some(origin));
        let allowed = r
            .header("access-control-allow-headers")
            .unwrap()
            .to_lowercase();
        assert!(allowed.contains("authorization"));
    }
    let r = request(
        s.addr,
        "OPTIONS",
        "/api",
        &[
            ("Origin", "https://evil.example"),
            ("Access-Control-Request-Method", "POST"),
        ],
        "",
    )
    .await;
    assert_eq!(r.header("access-control-allow-origin"), None);
    let r = call(s.addr, "vault.status", Value::Null).await;
    assert_eq!(
        r.header("access-control-allow-origin"),
        None,
        "sem Origin, sem CORS"
    );
}

#[test]
fn error_codes_map_to_http_statuses() {
    use treasury_dev::status_for;
    assert_eq!(status_for("vault_locked").as_u16(), 423);
    assert_eq!(status_for("not_implemented").as_u16(), 501);
    assert_eq!(status_for("forbidden").as_u16(), 403);
    assert_eq!(status_for("db").as_u16(), 500);
}

#[test]
fn home_never_defaults_to_the_real_vault() {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap();
    assert!(treasury_dev::resolve_home(Some(home.join(".treasury"))).is_err());
    let dir = tempfile::tempdir().unwrap();
    let h = treasury_dev::resolve_home(Some(dir.path().join("dev"))).unwrap();
    assert!(h.is_absolute() && h.exists());
}
