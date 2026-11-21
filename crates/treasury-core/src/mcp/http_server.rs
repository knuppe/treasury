//! Servidor MCP por HTTP (Streamable HTTP) para agentes que não usam stdio: escuta só em
//! `127.0.0.1`, recusa `Host` que não seja local (proteção contra DNS rebinding, do rmcp) e
//! exige `Authorization: Bearer trsk_…` em **toda** requisição. Os escopos da chave valem
//! por requisição (ver [`super::TreasuryMcp::for_http`]).
//!
//! Exceção opcional ([`LOOPBACK_NO_AUTH_SETTING`], desligada por padrão): pedidos **sem**
//! chave vindos do próprio computador. Vale só quando o endereço de quem conectou (o do
//! socket, nunca cabeçalhos como `X-Forwarded-For`) é de loopback e o pedido não vem de um
//! navegador (`Origin`, `Sec-Fetch-*` ou `Referer` presentes = recusado), para que uma
//! página aberta no navegador não use o servidor. "Exigir chave de API" ligado anula a
//! exceção. O cliente sem chave recebe o mesmo acesso de um cliente MCP local sem chave
//! (leitura; escritas como proposta só com o ajuste próprio).
//!
//! Desligado por padrão (Ajustes › Integrações MCP). O app liga, desliga e troca a porta
//! conforme os ajustes, no laço de rotinas.

use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use std::sync::Arc;

use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio_util::sync::CancellationToken;

use super::TreasuryMcp;
use super::keys::{self, KeyGrant};
use crate::SharedApi;
use crate::error::{Error, Result};

/// Servidor em execução; [`HttpServer::shutdown`] (ou soltar) encerra.
pub struct HttpServer {
    pub addr: SocketAddr,
    cancel: CancellationToken,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl HttpServer {
    pub async fn shutdown(mut self) {
        self.cancel.cancel();
        if let Some(t) = self.task.take() {
            let _ = t.await;
        }
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// Ajuste: aceitar pedidos sem chave do loopback (padrão: não).
pub const LOOPBACK_NO_AUTH_SETTING: &str = "mcp.http_loopback_no_auth";

/// O loopback sem chave vale agora: o ajuste ligado e o modo desenvolvedor também.
pub fn loopback_no_auth(conn: &rusqlite::Connection) -> crate::error::Result<bool> {
    Ok(
        crate::settings::get_or(conn, LOOPBACK_NO_AUTH_SETTING, false)?
            && crate::devfeedback::developer_mode(conn)?,
    )
}

/// Marca do pedido aceito sem chave (vale o acesso de cliente local sem chave).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopbackAnonymous;

/// Cabeçalhos que só navegadores mandam: um pedido com eles nunca entra sem chave.
const BROWSER_HEADERS: &[&str] = &[
    "origin",
    "referer",
    "sec-fetch-site",
    "sec-fetch-mode",
    "sec-fetch-dest",
];

fn forbidden(msg: &str) -> Response {
    (StatusCode::FORBIDDEN, msg.to_string()).into_response()
}

fn unauthorized(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
        msg.to_string(),
    )
        .into_response()
}

async fn auth(api: SharedApi, mut req: Request, next: Next) -> Response {
    let key = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|k| k.trim().to_string());
    let Some(key) = key else {
        let peer = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip());
        if !peer.is_some_and(|ip| ip.is_loopback()) {
            return unauthorized(&crate::i18n::t("mcp-http-informe-chave"));
        }
        let from_browser = BROWSER_HEADERS
            .iter()
            .any(|h| req.headers().contains_key(*h));
        let allowed = tokio::task::spawn_blocking(move || {
            api.with_session(|s, _| {
                let c = s.db.conn();
                // "Exigir chave de API" vence: sem exceção para o loopback. E só vale no modo
                // desenvolvedor (um ajuste antigo ligado não abre o cofre fora dele).
                Ok(loopback_no_auth(c)? && !super::keys::require_key(c)?)
            })
            .unwrap_or(false)
        })
        .await
        .unwrap_or(false);
        if !allowed {
            return unauthorized(&crate::i18n::t("mcp-http-informe-chave"));
        }
        if from_browser {
            return forbidden(&crate::i18n::t("mcp-http-navegador"));
        }
        req.extensions_mut().insert(LoopbackAnonymous);
        return next.run(req).await;
    };
    let grant: Option<KeyGrant> = tokio::task::spawn_blocking(move || {
        api.with_session(|s, _| keys::verify(s.db.conn(), &key, !s.db.is_read_only()))
            .ok()
            .flatten()
    })
    .await
    .ok()
    .flatten();
    match grant {
        Some(g) => {
            req.extensions_mut().insert(g);
            next.run(req).await
        }
        // Cofre bloqueado também cai aqui: sem cofre aberto não há como conferir a chave.
        None => unauthorized(&crate::i18n::t("mcp-http-chave-invalida")),
    }
}

/// `GET /events`: avisos de dados alterados (SSE, evento `data_changed`), com a mesma
/// autenticação do `/mcp`. O aviso diz só o comando, o módulo e quem gravou.
/// O fluxo termina quando o servidor para (sem isso o desligamento esperaria o cliente).
fn events_sse(
    stop: CancellationToken,
) -> Sse<impl futures_util::Stream<Item = std::result::Result<Event, std::convert::Infallible>>> {
    use tokio::sync::broadcast::error::RecvError;
    let rx = crate::events::subscribe();
    let stream = futures_util::stream::unfold((rx, stop), |(mut rx, stop)| async move {
        loop {
            let next = tokio::select! {
                r = rx.recv() => r,
                () = stop.cancelled() => return None,
            };
            match next {
                Ok(e) => {
                    let ev = Event::default()
                        .event("data_changed")
                        .id(e.seq.to_string())
                        .json_data(&e)
                        .unwrap_or_default();
                    return Some((Ok(ev), (rx, stop)));
                }
                // Cliente lento: perde os mais velhos e segue.
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return None,
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Sobe o servidor em `127.0.0.1:port`.
pub async fn start(api: SharedApi, port: u16) -> Result<HttpServer> {
    let cancel = CancellationToken::new();
    let config =
        StreamableHttpServerConfig::default().with_cancellation_token(cancel.child_token());
    let factory_api = api.clone();
    let service = StreamableHttpService::new(
        move || Ok(TreasuryMcp::for_http(factory_api.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    let auth_api = api.clone();
    let router = axum::Router::new()
        .route(
            "/events",
            axum::routing::get({
                let stop = cancel.child_token();
                move || std::future::ready(events_sse(stop.clone()))
            }),
        )
        .nest_service("/mcp", service)
        .layer(middleware::from_fn(move |req, next| {
            auth(auth_api.clone(), req, next)
        }));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| {
            Error::Other(crate::i18n::t_args(
                "mcp-servidor-porta",
                &crate::i18n::args(&[
                    ("porta", port.to_string().into()),
                    ("detalhe", e.to_string().into()),
                ]),
            ))
        })?;
    let addr = listener.local_addr()?;
    let stop = cancel.clone();
    let task = tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move { stop.cancelled().await })
        .await;
    });
    tracing::info!("MCP HTTP em http://{addr}/mcp");
    Ok(HttpServer {
        addr,
        cancel,
        task: Some(task),
    })
}
