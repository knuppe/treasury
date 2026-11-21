//! Servidor de desenvolvimento do Treasury: expõe o despachante do núcleo por HTTP para a
//! interface React rodar no navegador (Vite em `localhost:1420`) e para testes E2E.
//!
//! - `POST /api` com `{"cmd": "modulo.acao", "args": {...}}` e `Authorization: Bearer
//!   <token>`; resposta sempre no envelope `{"ok": true, "data": ...}` ou `{"ok": false,
//!   "error": {"code", "message", "details?"}}` (o status HTTP acompanha o código).
//! - `GET /api/health` sem token: `{"ok": true, "data": {"version": ...}}`.
//! - CORS só para `http://localhost:1420` e `http://127.0.0.1:1420`; escuta só em 127.0.0.1.
//!
//! Não vai para o usuário final.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine;
use serde::Deserialize;
use serde_json::{Value, json};
use tower_http::cors::{AllowOrigin, CorsLayer};
use treasury_core::{ApiError, SharedApi};

/// Origens do Vite (`npm run dev` do app desktop).
pub const ALLOWED_ORIGINS: [&str; 2] = ["http://localhost:1420", "http://127.0.0.1:1420"];

/// Intervalo das rotinas periódicas (bloqueio automático, backup diário).
pub const TICK_INTERVAL: Duration = Duration::from_secs(30);

/// Limite do corpo: importações mandam arquivos em base64.
const BODY_LIMIT: usize = 64 * 1024 * 1024;

#[derive(Clone)]
struct DevState {
    api: SharedApi,
    token: Arc<str>,
}

#[derive(Deserialize)]
struct CallBody {
    cmd: String,
    #[serde(default)]
    args: Value,
}

/// Rotas do servidor de desenvolvimento.
pub fn router(api: SharedApi, token: impl Into<Arc<str>>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(
            ALLOWED_ORIGINS.map(HeaderValue::from_static),
        ))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(Duration::from_secs(600));
    Router::new()
        .route("/api", post(call_api))
        .route("/api/health", get(health))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(cors)
        .with_state(DevState {
            api,
            token: token.into(),
        })
}

/// Token aleatório (256 bits, base64 URL sem preenchimento).
pub fn new_token() -> anyhow::Result<String> {
    let mut raw = [0u8; 32];
    getrandom::fill(&mut raw).map_err(|e| anyhow::anyhow!("gerador aleatório: {e}"))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw))
}

/// Chama `api.tick()` a cada [`TICK_INTERVAL`] (bloqueio automático e backup diário).
pub fn spawn_ticker(api: SharedApi) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            let api = api.clone();
            match tokio::task::spawn_blocking(move || api.tick()).await {
                Ok(Ok(out)) if out.locked => tracing::info!("cofre bloqueado por inatividade"),
                Ok(Ok(out)) if out.backup_created => tracing::info!("backup diário criado"),
                Ok(Ok(_)) => {}
                Ok(Err(e)) => tracing::warn!("rotina periódica falhou ({})", e.code()),
                Err(e) => tracing::warn!("rotina periódica interrompida: {e}"),
            }
        }
    })
}

/// Atende HTTP no `listener` até `shutdown` terminar.
pub async fn serve_http(
    listener: tokio::net::TcpListener,
    api: SharedApi,
    token: impl Into<Arc<str>>,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, router(api, token))
        .with_graceful_shutdown(shutdown)
        .await
}

/// Endereço local (só 127.0.0.1).
pub fn local_addr(port: u16) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], port))
}

/// Raiz de dados do desenvolvimento: `--home`, senão `TREASURY_HOME`, senão
/// `./.treasury-dev` (no `.gitignore`). Nunca `~/.treasury`. Devolve o caminho absoluto.
pub fn resolve_home(arg: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let home = arg
        .or_else(|| {
            std::env::var_os("TREASURY_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from(".treasury-dev"));
    if let Some(user) = dirs_home()
        && home == user.join(".treasury")
    {
        anyhow::bail!(
            "o treasury-dev não usa ~/.treasury (dados reais); escolha outra pasta com --home"
        );
    }
    treasury_core::paths::create_private_dir(&home)?;
    let home = std::fs::canonicalize(&home)?;
    let real = dirs_home().and_then(|u| std::fs::canonicalize(u.join(".treasury")).ok());
    if real.is_some_and(|r| r == home) {
        anyhow::bail!("o treasury-dev não usa ~/.treasury (dados reais); escolha outra pasta");
    }
    Ok(home)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Escolhe o keychain do processo: `memory` (padrão: nunca toca o keychain real), `off`
/// ou `os` (Keychain do macOS / Secret Service).
pub fn configure_keychain(mode: &str) -> anyhow::Result<()> {
    use treasury_core::keychain::{MemoryStore, OsStore, set_store};
    match mode {
        "memory" => set_store(Arc::new(MemoryStore::default())),
        "off" => set_store(Arc::new(MemoryStore::unavailable())),
        "os" => set_store(Arc::new(OsStore::default())),
        other => anyhow::bail!("keychain inválido: {other} (use memory, off ou os)"),
    }
    Ok(())
}

async fn health() -> Response {
    envelope_ok(json!({ "version": env!("CARGO_PKG_VERSION") }))
}

async fn call_api(State(state): State<DevState>, headers: HeaderMap, body: Bytes) -> Response {
    if !authorized(&headers, &state.token) {
        return envelope_err(
            StatusCode::UNAUTHORIZED,
            ApiError {
                code: "unauthorized".into(),
                message: "token ausente ou inválido (Authorization: Bearer <token>)".into(),
                details: None,
            },
        );
    }
    let call: CallBody = match serde_json::from_slice(&body) {
        Ok(c) => c,
        Err(e) => {
            return envelope_err(
                StatusCode::BAD_REQUEST,
                ApiError {
                    code: "invalid".into(),
                    message: format!("corpo inválido: esperado {{\"cmd\", \"args\"}} ({e})"),
                    details: None,
                },
            );
        }
    };
    let api = state.api.clone();
    let cmd = call.cmd.clone();
    match tokio::task::spawn_blocking(move || api.call(&call.cmd, call.args)).await {
        Ok(Ok(data)) => envelope_ok(data),
        Ok(Err(e)) => {
            tracing::debug!("{cmd}: {}", e.code());
            envelope_err(status_for(e.code()), e.to_api())
        }
        Err(e) => envelope_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError {
                code: "other".into(),
                message: format!("tarefa interrompida: {e}"),
                details: None,
            },
        ),
    }
}

fn authorized(headers: &HeaderMap, token: &str) -> bool {
    let Some(given) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    else {
        return false;
    };
    let (a, b) = (given.trim().as_bytes(), token.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Status HTTP de cada código de erro do núcleo (o corpo traz sempre o envelope).
pub fn status_for(code: &str) -> StatusCode {
    match code {
        "invalid" | "no_engine" => StatusCode::BAD_REQUEST,
        "wrong_password" | "wrong_pin" => StatusCode::UNAUTHORIZED,
        "forbidden" => StatusCode::FORBIDDEN,
        "not_found" | "unknown_command" | "no_vault" => StatusCode::NOT_FOUND,
        "conflict" | "vault_exists" | "already_open" => StatusCode::CONFLICT,
        "vault_locked" => StatusCode::LOCKED,
        "not_implemented" => StatusCode::NOT_IMPLEMENTED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn envelope_ok(data: Value) -> Response {
    (
        StatusCode::OK,
        axum::Json(json!({ "ok": true, "data": data })),
    )
        .into_response()
}

fn envelope_err(status: StatusCode, error: ApiError) -> Response {
    (status, axum::Json(json!({ "ok": false, "error": error }))).into_response()
}
