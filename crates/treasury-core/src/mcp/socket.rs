//! Socket local do app (`~/.treasury/run/mcp.sock`) para o `treasury mcp` e CLIs locais.
//!
//! Segurança: a pasta `run/` é `0700`, o socket e o token são `0600`, o token é aleatório
//! (256 bits) e novo a cada sessão do app, e só conexões do mesmo usuário (credencial do
//! par no socket) são aceitas.
//!
//! Protocolo: o cliente manda uma primeira linha JSON `{"token":"...","client":"nome"}`.
//! Token errado (ou linha inválida, ou mais de 5 s) fecha a conexão sem resposta. Token
//! certo: o servidor responde uma linha `{"ok":true,"server":"treasury","version":"..."}` e
//! daí em diante o fluxo é MCP puro (JSON-RPC, uma mensagem por linha), atendido como
//! cliente externo (consultas; escrita só com o ajuste `mcp.allow_writes`). Com o cofre
//! bloqueado, as ferramentas respondem o erro de cofre bloqueado.
//!
//! Tokens de agente: quando o app executa uma CLI local como motor, ele registra um token
//! de sessão só daquele job (`ai::jobs::Runtime::grant`) e passa o caminho de um arquivo
//! `0600` com ele em [`TOKEN_FILE_ENV`]. Com esse token o cliente é o agente do app: recebe
//! as ferramentas de escrita, sempre sob a fila de aprovação.

use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub(crate) use super::access::key_access;
pub use super::access::{
    API_KEY_ENV, Handshake, HandshakeReply, SOCKET_FILE, TOKEN_FILE, TOKEN_FILE_ENV,
};
use super::{SERVER_NAME, TreasuryMcp, sanitize_client};
use crate::api::SharedApi;
use crate::error::{Error, Result};

const HANDSHAKE_MAX_BYTES: usize = 4096;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

#[cfg(target_os = "macos")]
const MAX_SOCKET_PATH: usize = 104;
#[cfg(not(target_os = "macos"))]
const MAX_SOCKET_PATH: usize = 108;

/// Servidor do socket em execução. Parar (ou soltar) encerra as conexões e apaga o socket
/// e o token.
pub struct SocketServer {
    socket_path: PathBuf,
    token_path: PathBuf,
    shutdown: watch::Sender<bool>,
    task: Option<JoinHandle<()>>,
}

impl SocketServer {
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn token_path(&self) -> &Path {
        &self.token_path
    }

    /// Para de aceitar conexões, encerra as abertas e apaga os arquivos.
    pub async fn shutdown(mut self) {
        let _ = self.shutdown.send(true);
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
        self.remove_files();
    }

    fn remove_files(&self) {
        let _ = std::fs::remove_file(&self.socket_path);
        let _ = std::fs::remove_file(&self.token_path);
    }
}

impl Drop for SocketServer {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
        if let Some(task) = self.task.take() {
            task.abort();
        }
        self.remove_files();
    }
}

/// Cria `run_dir/mcp.sock` (0600; apaga um socket órfão antes) e grava um token aleatório
/// novo em `run_dir/mcp.token` (0600). Precisa de um runtime tokio.
pub async fn start_socket_server(api: SharedApi, run_dir: &Path) -> Result<SocketServer> {
    crate::ai::jobs::Runtime::attach(&api);
    crate::paths::create_private_dir(run_dir)?;
    let socket_path = run_dir.join(SOCKET_FILE);
    let token_path = run_dir.join(TOKEN_FILE);
    remove_stale_socket(&socket_path)?;
    let _ = std::fs::remove_file(&token_path);

    // sockaddr_un limita o caminho (~104 bytes no macOS, 108 no Linux).
    if socket_path.as_os_str().len() >= MAX_SOCKET_PATH {
        return Err(Error::Other(format!(
            "caminho do socket do MCP longo demais ({} bytes; o limite é {}): use uma pasta de \
             dados mais curta",
            socket_path.as_os_str().len(),
            MAX_SOCKET_PATH - 1
        )));
    }
    let listener = UnixListener::bind(&socket_path).map_err(|e| {
        Error::Other(crate::i18n::t_args(
            "mcp-servidor-socket-erro",
            &crate::i18n::args(&[
                ("caminho", socket_path.display().to_string().into()),
                ("detalhe", e.to_string().into()),
            ]),
        ))
    })?;
    crate::paths::set_mode(&socket_path, 0o600)?;
    let token = new_token()?;
    write_private(&token_path, &token)?;
    let owner_uid = std::fs::metadata(run_dir)?.uid();

    let (tx, rx) = watch::channel(false);
    let task = tokio::spawn(accept_loop(listener, api, Arc::from(token), owner_uid, rx));
    tracing::info!("MCP: socket local pronto");
    Ok(SocketServer {
        socket_path,
        token_path,
        shutdown: tx,
        task: Some(task),
    })
}

/// Conecta ao socket do app e faz o handshake. Devolve as metades da conexão, já prontas
/// para MCP. Falha se o app não estiver aberto ou recusar o token.
pub async fn connect_socket(
    run_dir: &Path,
    client: &str,
) -> Result<(BufReader<OwnedReadHalf>, OwnedWriteHalf)> {
    let token_path = match std::env::var_os(TOKEN_FILE_ENV) {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => run_dir.join(TOKEN_FILE),
    };
    let token = std::fs::read_to_string(token_path)?;
    let stream = tokio::time::timeout(
        CONNECT_TIMEOUT,
        UnixStream::connect(run_dir.join(SOCKET_FILE)),
    )
    .await
    .map_err(|_| Error::Other(crate::i18n::t("mcp-servidor-socket-tempo")))??;
    let (rd, mut wr) = stream.into_split();
    let mut line = serde_json::to_string(&Handshake {
        token: token.trim().to_string(),
        client: Some(client.to_string()),
        api_key: std::env::var(API_KEY_ENV)
            .ok()
            .filter(|k| !k.trim().is_empty()),
    })?;
    line.push('\n');
    wr.write_all(line.as_bytes()).await?;
    let mut rd = BufReader::new(rd);
    let reply = tokio::time::timeout(HANDSHAKE_TIMEOUT, read_line_limited(&mut rd))
        .await
        .ok()
        .and_then(|r| r.ok().flatten())
        .and_then(|l| serde_json::from_str::<HandshakeReply>(l.trim()).ok());
    match reply {
        Some(r) if r.ok => Ok((rd, wr)),
        Some(HandshakeReply {
            error: Some(why), ..
        }) => Err(Error::Forbidden(why)),
        _ => Err(Error::Forbidden(crate::i18n::t("mcp-cli-app-recusou"))),
    }
}

fn remove_stale_socket(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_socket() => {
            // Outro processo ainda atende? (Não deveria: o app é instância única.)
            if std::os::unix::net::UnixStream::connect(path).is_ok() {
                return Err(Error::AlreadyOpen);
            }
            std::fs::remove_file(path)?;
            Ok(())
        }
        Ok(_) => Err(Error::Other(crate::i18n::t_args(
            "mcp-servidor-nao-socket",
            &crate::i18n::args(&[("caminho", path.display().to_string().into())]),
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn new_token() -> Result<String> {
    let mut raw = [0u8; 32];
    getrandom::fill(&mut raw).map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw))
}

fn write_private(path: &Path, content: &str) -> Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    std::io::Write::write_all(&mut f, content.as_bytes())?;
    crate::paths::set_mode(path, 0o600)
}

/// Comparação em tempo constante (para o tamanho esperado).
fn token_eq(given: &str, expected: &str) -> bool {
    let (a, b) = (given.as_bytes(), expected.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn read_line_limited<R: AsyncBufRead + Unpin>(r: &mut R) -> std::io::Result<Option<String>> {
    let mut buf = Vec::new();
    let n = (&mut *r)
        .take(HANDSHAKE_MAX_BYTES as u64)
        .read_until(b'\n', &mut buf)
        .await?;
    if n == 0 || !buf.ends_with(b"\n") {
        return Ok(None);
    }
    Ok(String::from_utf8(buf).ok())
}

async fn accept_loop(
    listener: UnixListener,
    api: SharedApi,
    token: Arc<str>,
    owner_uid: u32,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    tokio::spawn(handle_connection(
                        stream,
                        api.clone(),
                        token.clone(),
                        owner_uid,
                        shutdown.clone(),
                    ));
                }
                Err(e) => {
                    tracing::warn!("MCP: falha ao aceitar conexão: {e}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
        }
    }
}

/// Cliente externo novo neste cofre: aviso na tela de Integrações MCP (item 14 de
/// docs/local-security.md).
fn note_client(api: &SharedApi, client: &str, pid: Option<i32>) {
    let (program, pid) = super::clients::peer_program(pid);
    match api
        .with_session(|s, _| super::clients::note(s.db.conn(), client, program.as_deref(), pid))
    {
        Ok(true) => {
            tracing::warn!("MCP: cliente novo conectado ({client})");
            crate::events::emit_system("mcp.client_new");
        }
        Ok(false) => {}
        Err(e) => tracing::debug!("MCP: aviso de cliente novo: {e}"),
    }
}

async fn handle_connection(
    stream: UnixStream,
    api: SharedApi,
    token: Arc<str>,
    owner_uid: u32,
    mut shutdown: watch::Receiver<bool>,
) {
    let peer_pid = match stream.peer_cred() {
        Ok(cred) if cred.uid() == owner_uid => cred.pid(),
        Ok(_) => {
            tracing::warn!("MCP: conexão de outro usuário recusada");
            return;
        }
        Err(e) => {
            tracing::warn!("MCP: credencial do cliente indisponível: {e}");
            return;
        }
    };
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);
    let handshake = tokio::time::timeout(HANDSHAKE_TIMEOUT, read_line_limited(&mut rd))
        .await
        .ok()
        .and_then(|r| r.ok().flatten())
        .and_then(|l| serde_json::from_str::<Handshake>(l.trim()).ok());
    let Some(hs) = handshake else {
        tracing::debug!("MCP: handshake ausente ou inválido");
        return;
    };
    let grant = api.ai_runtime().grant_for(hs.token.trim());
    if grant.is_none() && !token_eq(hs.token.trim(), &token) {
        tracing::warn!("MCP: token inválido; conexão fechada");
        return;
    }
    let client = hs
        .client
        .as_deref()
        .and_then(sanitize_client)
        .unwrap_or_else(|| "socket".into());
    // Clientes externos: chave de API (escopos) ou recusa, conforme o ajuste.
    let key_grant = if grant.is_some() {
        None
    } else {
        let api2 = api.clone();
        let key = hs.api_key.clone();
        match tokio::task::spawn_blocking(move || key_access(&api2, key.as_deref())).await {
            Ok(Ok(g)) => g,
            Ok(Err(why)) => {
                tracing::warn!("MCP: conexão recusada ({why})");
                let refused = HandshakeReply {
                    ok: false,
                    server: SERVER_NAME.into(),
                    version: crate::config::app_version().into(),
                    error: Some(why),
                };
                let mut line = serde_json::to_string(&refused).unwrap_or_default();
                line.push('\n');
                let _ = wr.write_all(line.as_bytes()).await;
                return;
            }
            Err(_) => return,
        }
    };
    let reply = HandshakeReply {
        ok: true,
        server: SERVER_NAME.into(),
        version: crate::config::app_version().into(),
        error: None,
    };
    let mut line = serde_json::to_string(&reply).unwrap_or_default();
    line.push('\n');
    if wr.write_all(line.as_bytes()).await.is_err() {
        return;
    }
    tracing::info!("MCP: cliente conectado pelo socket");
    if grant.is_none() {
        let (api2, client2) = (api.clone(), client.clone());
        tokio::task::spawn_blocking(move || note_client(&api2, &client2, peer_pid));
    }
    let api_key = hs
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(|k| crate::vault::Secret::new(k.to_string()));
    let server = match (grant, api_key) {
        (Some(g), _) => TreasuryMcp::for_agent(api, g.client, g.job_id),
        (None, Some(k)) => TreasuryMcp::for_key(api, client, k, key_grant),
        // Sem chave só chega aqui nos testes do núcleo (`key_access`).
        (None, None) => TreasuryMcp::new(api, client),
    };
    tokio::select! {
        res = server.serve_transport((rd, wr)) => {
            if let Err(e) = res {
                tracing::debug!("MCP: sessão do socket encerrada: {e}");
            }
        }
        _ = shutdown.changed() => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_comparison() {
        assert!(token_eq("abc", "abc"));
        assert!(!token_eq("abd", "abc"));
        assert!(!token_eq("ab", "abc"));
        let t = new_token().unwrap();
        assert_eq!(t.len(), 43);
        assert_ne!(t, new_token().unwrap());
    }
}
