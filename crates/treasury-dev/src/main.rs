//! `treasury-dev`: ferramentas de desenvolvimento do Treasury (não vai para o usuário).
//!
//! - `treasury-dev serve --home <dir> --port 7777 [--token T]`: API do núcleo por HTTP em
//!   127.0.0.1 para a interface no navegador e testes E2E, mais o socket MCP local (como o
//!   app aberto).
//! - `treasury-dev mcp --home <dir>`: o mesmo que `treasury mcp` (stdio).

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use treasury_core::paths::TreasuryPaths;
use treasury_core::{Api, mcp};
use treasury_dev::{
    configure_keychain, local_addr, new_token, resolve_home, serve_http, spawn_ticker,
};

#[derive(Parser)]
#[command(
    name = "treasury-dev",
    version,
    about = "Ferramentas de desenvolvimento do Treasury (servidor HTTP da API e MCP)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Servidor HTTP da API (só 127.0.0.1) e socket MCP local.
    Serve {
        /// Raiz dos dados (padrão: TREASURY_HOME ou ./.treasury-dev; nunca ~/.treasury).
        #[arg(long)]
        home: Option<PathBuf>,
        #[arg(long, default_value_t = 7777)]
        port: u16,
        /// Token do `Authorization: Bearer` (padrão: gerado e impresso no stderr).
        #[arg(long)]
        token: Option<String>,
        /// Keychain: memory (padrão), off ou os (Keychain/Secret Service reais).
        #[arg(long, default_value = "memory")]
        keychain: String,
        /// Não abrir o socket MCP local (run/mcp.sock).
        #[arg(long)]
        no_mcp_socket: bool,
    },
    /// Servidor MCP via stdio, igual ao `treasury mcp`.
    Mcp {
        /// Raiz dos dados (padrão: TREASURY_HOME ou ./.treasury-dev; nunca ~/.treasury).
        #[arg(long)]
        home: Option<PathBuf>,
        /// Keychain para o modo app fechado: off (padrão), memory ou os.
        #[arg(long, default_value = "off")]
        keychain: String,
    },
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // stdout é do protocolo (MCP) ou do usuário: logs só em stderr, sem cores fora de um
    // terminal (os clientes MCP guardam o stderr em arquivo).
    use std::io::IsTerminal;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init();
}

fn main() -> anyhow::Result<()> {
    init_logging();
    let cli = Cli::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    match cli.cmd {
        Cmd::Serve {
            home,
            port,
            token,
            keychain,
            no_mcp_socket,
        } => rt.block_on(serve(home, port, token, &keychain, !no_mcp_socket)),
        Cmd::Mcp { home, keychain } => {
            let result = rt.block_on(async {
                let home = resolve_home(home)?;
                configure_keychain(&keychain)?;
                let paths = TreasuryPaths::with_root(&home)?;
                mcp::run_mcp_cli(paths).await.map_err(anyhow::Error::from)
            });
            // O stdin pode continuar aberto numa leitura bloqueante: encerra o processo.
            rt.shutdown_background();
            match result {
                Ok(()) => std::process::exit(0),
                Err(e) => {
                    eprintln!("treasury-dev mcp: {e}");
                    std::process::exit(1)
                }
            }
        }
    }
}

async fn serve(
    home: Option<PathBuf>,
    port: u16,
    token: Option<String>,
    keychain: &str,
    mcp_socket: bool,
) -> anyhow::Result<()> {
    let home = resolve_home(home)?;
    configure_keychain(keychain)?;
    let paths = TreasuryPaths::with_root(&home)?;
    let api = Api::new(paths.clone())
        .context("abrindo a pasta de dados")?
        .into_shared();
    let (token, generated) = match token {
        Some(t) if !t.trim().is_empty() => (t.trim().to_string(), false),
        _ => (new_token()?, true),
    };
    let addr = local_addr(port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("porta {port} ocupada?"))?;
    let socket = if mcp_socket {
        match mcp::start_socket_server(api.clone(), &paths.run_dir).await {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("treasury-dev: sem socket MCP ({e}); a API HTTP continua");
                None
            }
        }
    } else {
        None
    };
    let ticker = spawn_ticker(api.clone());

    eprintln!(
        "treasury-dev: API em http://{addr}/api (dados em {})",
        home.display()
    );
    if generated {
        eprintln!("treasury-dev: token {token}");
    }
    if socket.is_some() {
        eprintln!(
            "treasury-dev: socket MCP em {} (cliente: treasury-dev mcp --home {})",
            paths.run_dir.join(mcp::SOCKET_FILE).display(),
            home.display()
        );
    }

    serve_http(listener, api.clone(), token, shutdown_signal()).await?;

    ticker.abort();
    if let Some(s) = socket {
        s.shutdown().await;
    }
    let api2 = api.clone();
    tokio::task::spawn_blocking(move || api2.shutdown())
        .await?
        .context("ao fechar o cofre")?;
    eprintln!("treasury-dev: encerrado");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = term => {}
    }
}
