//! Subcomando `treasury mcp`: servidor MCP via stdio para clientes locais (Claude
//! Desktop, Claude Code, outras CLIs).
//!
//! 1. App aberto: conecta ao socket do app (`run/mcp.sock` + token) e vira um proxy
//!    stdio ⇄ socket (bytes puros depois do handshake). Escritas futuras passam pela fila
//!    de aprovação do app e o cofre não é aberto duas vezes.
//! 2. App fechado: só com o ajuste "Responder com o app fechado" ligado (desligado por
//!    padrão, docs/local-security.md, item 2) e com uma chave de API válida em
//!    `TREASURY_API_KEY`. Lê a chave do keychain (se o desbloqueio pelo keychain estiver
//!    habilitado) e abre o banco **em modo leitura**, sem a trava exclusiva do app, que pode
//!    abrir normalmente enquanto este cliente está conectado. O ajuste fica dentro do cofre:
//!    outro programa não o liga sem o cofre aberto.
//! 3. Sem app, sem o ajuste, sem chave ou sem keychain: o servidor sobe mesmo assim e toda
//!    ferramenta explica o que fazer.

use std::sync::Arc;

use crate::api::Api;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::keychain;
use crate::paths::TreasuryPaths;
use crate::storage::Db;

use super::TreasuryMcp;

/// Nome do cliente enviado no handshake do socket.
pub const CLI_CLIENT: &str = "treasury mcp";

/// Rótulo do cliente quando o `treasury mcp` atende em processo (app fechado).
const READ_ONLY_CLIENT: &str = "treasury mcp (app fechado)";

/// Executa o `treasury mcp` até o cliente fechar o stdin (ou o app fechar a conexão).
///
/// Logs só em stderr: o chamador deve configurar o `tracing` para stderr (o stdout é o
/// protocolo). Ao retornar, encerre o processo.
pub async fn run_mcp_cli(paths: TreasuryPaths) -> Result<()> {
    // Idioma da instalação para os textos que as ferramentas devolvem (o app fechado não
    // passa pelo `Api::new`, que é quem o lê normalmente).
    crate::i18n::set_current(crate::i18n::load(&paths.config_path));
    let key = std::env::var(super::access::API_KEY_ENV)
        .ok()
        .filter(|k| !k.trim().is_empty());
    #[cfg(unix)]
    match super::socket::connect_socket(&paths.run_dir, CLI_CLIENT).await {
        Ok((rd, wr)) => {
            tracing::info!("treasury mcp: conectado ao app pelo socket local");
            return proxy::run(rd, wr).await;
        }
        // O app está aberto e recusou (chave de API exigida ou inválida): dizer isso, e não
        // que o app está fechado.
        Err(Error::Forbidden(why)) => {
            tracing::info!("treasury mcp: o app recusou a conexão");
            return TreasuryMcp::unavailable(
                refused_message(why),
                CLI_CLIENT,
                Some(paths.run_dir.clone()),
            )
            .serve_stdio()
            .await;
        }
        Err(e) => tracing::info!(
            "treasury mcp: app indisponível ({}); tentando o modo leitura",
            e.code()
        ),
    }
    let server = match open_read_only_api(&paths) {
        Ok(api) => {
            let api: crate::SharedApi = Arc::new(api);
            match closed_app_access(&api, key.as_deref()) {
                Ok(g) => {
                    tracing::info!("treasury mcp: cofre aberto em modo leitura (app fechado)");
                    let key = crate::vault::Secret::new(key.clone().unwrap_or_default());
                    TreasuryMcp::for_key(api, READ_ONLY_CLIENT, key, Some(g))
                }
                Err(message) => {
                    TreasuryMcp::unavailable(message, READ_ONLY_CLIENT, Some(paths.run_dir.clone()))
                }
            }
        }
        Err(message) => {
            tracing::info!(
                "treasury mcp: sem acesso ao cofre; ferramentas explicarão como liberar"
            );
            TreasuryMcp::unavailable(message, READ_ONLY_CLIENT, Some(paths.run_dir.clone()))
        }
    };
    server.serve_stdio().await
}

/// Com o app fechado, só o ajuste ligado e uma chave de API válida dão acesso (o ajuste
/// "Exigir chave de API" não muda isso). `Err` é a mensagem para o cliente.
pub(crate) fn closed_app_access(
    api: &crate::SharedApi,
    key: Option<&str>,
) -> std::result::Result<super::keys::KeyGrant, String> {
    let enabled = api
        .with_session(|s, _| {
            crate::settings::get_or(s.db.conn(), super::api::CLI_WHEN_CLOSED_SETTING, false)
        })
        .unwrap_or(false);
    if !enabled {
        return Err(t("mcp-cli-app-fechado-desligado"));
    }
    match key.map(|k| super::access::key_access(api, Some(k))) {
        Some(Ok(Some(g))) => Ok(g),
        Some(Err(why)) => Err(refused_message(why)),
        _ => Err(refused_message(t("mcp-chave-exigida"))),
    }
}

/// "Acesso recusado: <motivo>. Crie uma chave… e informe em TREASURY_API_KEY."
fn refused_message(why: String) -> String {
    t_args(
        "mcp-cli-acesso-recusado",
        &args(&[
            ("motivo", why.into()),
            ("variavel", super::access::API_KEY_ENV.into()),
        ]),
    )
}

/// Abre o cofre em modo leitura com a chave do keychain, sem trava de processo. O erro é a
/// explicação para o usuário (como liberar o acesso).
pub fn open_read_only_api(paths: &TreasuryPaths) -> std::result::Result<Api, String> {
    if !paths.db_path.exists() {
        return Err(t("mcp-cli-sem-cofre"));
    }
    if !keychain::is_available() {
        return Err(t("mcp-cli-sem-keychain"));
    }
    let key = match keychain::load_key(&paths.db_path) {
        Ok(Some(k)) => k,
        Ok(None) => return Err(t("mcp-cli-sem-keychain")),
        Err(e) => {
            return Err(t_args(
                "mcp-cli-erro-keychain",
                &args(&[("erro", e.message().into())]),
            ));
        }
    };
    let db = Db::open_with_key(&paths.db_path, key, true).map_err(|e| match e {
        Error::WrongPassword => t("mcp-cli-chave-antiga"),
        other => t_args(
            "mcp-cli-erro-abrir",
            &args(&[("erro", other.message().into())]),
        ),
    })?;
    if db.needs_migration().unwrap_or(true) {
        return Err(t("mcp-cli-migracao"));
    }
    Api::open_read_only(paths.clone(), db).map_err(|e| e.to_string())
}

#[cfg(unix)]
mod proxy {
    use tokio::io::{AsyncWriteExt, BufReader};
    use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
    use tokio::sync::mpsc;

    use crate::error::Result;

    /// stdin → socket e socket → stdout, até o app fechar a conexão (ou o cliente fechar o
    /// stdin e o app terminar de responder).
    pub async fn run(mut rd: BufReader<OwnedReadHalf>, mut wr: OwnedWriteHalf) -> Result<()> {
        let mut input = spawn_stdin_pump();
        let upstream = tokio::spawn(async move {
            while let Some(chunk) = input.recv().await {
                if wr.write_all(&chunk).await.is_err() {
                    break;
                }
            }
            let _ = wr.shutdown().await;
        });
        let mut stdout = tokio::io::stdout();
        let res = tokio::io::copy(&mut rd, &mut stdout).await;
        let _ = stdout.flush().await;
        upstream.abort();
        res?;
        Ok(())
    }

    /// Lê o stdin numa thread própria. O stdin do tokio usa uma leitura bloqueante que não
    /// pode ser cancelada e prenderia o encerramento do processo se o app fechar primeiro.
    fn spawn_stdin_pump() -> mpsc::Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(16);
        std::thread::Builder::new()
            .name("treasury-mcp-stdin".into())
            .spawn(move || {
                use std::io::Read;
                let mut stdin = std::io::stdin().lock();
                let mut buf = vec![0u8; 16 * 1024];
                loop {
                    match stdin.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            if tx.blocking_send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                    }
                }
            })
            .expect("thread do stdin");
        rx
    }
}
