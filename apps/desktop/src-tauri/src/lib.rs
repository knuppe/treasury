//! Casca Tauri do Treasury: janela, comando `api_call` (ponte para o núcleo), rotinas em
//! segundo plano (bloqueio automático, backup diário) e instância única.

#[cfg(test)]
mod acl_tests;
mod biometric;
mod downloads;
mod external;
mod fido2;
mod inbox;
mod integrity;
mod notify;
mod ocr;
mod os_guard;
mod qr;
mod sefaz;
mod session_watch;
mod translate;
mod update_sig;
mod updater;

use std::time::Duration;

use serde_json::Value;
use tauri::{Emitter, Manager, RunEvent};
use treasury_core::paths::TreasuryPaths;
use treasury_core::{Api, ApiError, SharedApi};

pub(crate) struct AppState {
    pub(crate) api: SharedApi,
    /// Socket local do MCP (`~/.treasury/run/mcp.sock`), para o `treasury mcp` de clientes externos.
    #[cfg(unix)]
    mcp_socket: std::sync::Mutex<Option<treasury_core::mcp::SocketServer>>,
    /// Servidor MCP HTTP local (opcional, Ajustes › Integrações MCP) e o último erro.
    mcp_http: tokio::sync::Mutex<McpHttpState>,
}

#[derive(Default)]
pub(crate) struct McpHttpState {
    server: Option<treasury_core::mcp::http_server::HttpServer>,
    port: Option<u16>,
    error: Option<String>,
}

/// Estado do servidor MCP HTTP (para a tela de Ajustes).
#[tauri::command]
async fn mcp_http_status(state: tauri::State<'_, AppState>) -> Result<Value, ApiError> {
    let st = state.mcp_http.lock().await;
    Ok(serde_json::json!({
        "running": st.server.is_some(),
        "url": st.server.as_ref().map(|s| format!("http://{}/mcp", s.addr)),
        "error": st.error,
    }))
}

/// Aplica os ajustes do MCP HTTP agora (a tela chama depois de salvar).
#[tauri::command]
async fn mcp_http_apply(app: tauri::AppHandle) -> Result<Value, ApiError> {
    reconcile_mcp_http(&app).await;
    mcp_http_status(app.state::<AppState>()).await
}

/// Liga, desliga ou troca a porta do servidor MCP HTTP conforme os ajustes do cofre.
async fn reconcile_mcp_http(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let api = state.api.clone();
    if !api.is_unlocked() {
        return;
    }
    let settings =
        tauri::async_runtime::spawn_blocking(move || api.call("mcp.server_settings", Value::Null))
            .await;
    let Ok(Ok(s)) = settings else { return };
    let want = s["http_enabled"]
        .as_bool()
        .unwrap_or(false)
        .then(|| s["http_port"].as_u64().unwrap_or(7655) as u16);
    let mut st = state.mcp_http.lock().await;
    if st.port == want && (want.is_none() || st.server.is_some()) {
        return;
    }
    if let Some(old) = st.server.take() {
        old.shutdown().await;
    }
    st.port = want;
    st.error = None;
    if let Some(port) = want {
        match treasury_core::mcp::http_server::start(state.api.clone(), port).await {
            Ok(server) => st.server = Some(server),
            Err(e) => {
                tracing::warn!("MCP HTTP: {e}");
                st.error = Some(e.message());
            }
        }
    }
}

/// Fecha o app (o "Sair" da tela de termos de uso, docs/terms.md).
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// macOS: o item "Apoiar o projeto…" no menu do app, no idioma da instalação.
#[cfg(target_os = "macos")]
fn relabel_menu(app: &tauri::AppHandle) {
    let item = app
        .menu()
        .and_then(|m| m.items().ok())
        .and_then(|items| items.first().and_then(|m| m.as_submenu().cloned()))
        .and_then(|app_menu| app_menu.get("support"));
    if let Some(item) = item.as_ref().and_then(|i| i.as_menuitem()) {
        let _ = item.set_text(treasury_core::i18n::t("casca-menu-apoiar"));
    }
}

/// Ponte única para o núcleo: `invoke('api_call', { cmd, args })`.
/// Roda fora da thread principal (Argon2id leva ~0,5 s ao desbloquear).
#[tauri::command]
async fn api_call(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    cmd: String,
    args: Option<Value>,
) -> Result<Value, ApiError> {
    // O desbloqueio pelo Keychain só passa pela confirmação do sistema (Touch ID).
    if cmd == "vault.unlock_keychain" {
        return Err(ApiError {
            code: "forbidden".into(),
            message: "use o desbloqueio com Touch ID".into(),
            details: None,
        });
    }
    let relabel = cmd == "system.set_locale";
    let api = state.api.clone();
    let out = tauri::async_runtime::spawn_blocking(move || {
        api.call(&cmd, args.unwrap_or(Value::Null))
            .map_err(ApiError::from)
    })
    .await
    .map_err(|e| ApiError {
        code: "internal".into(),
        message: e.to_string(),
        details: None,
    })?;
    // O idioma mudou: o menu nativo acompanha (as telas a interface redesenha).
    #[cfg(target_os = "macos")]
    if relabel && out.is_ok() {
        relabel_menu(&app);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (relabel, app);
    out
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_env("TREASURY_LOG")
        .unwrap_or_else(|_| "warn,treasury=info".into());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

pub fn run() {
    init_tracing();
    let paths = match TreasuryPaths::resolve() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Treasury: {e}");
            std::process::exit(1);
        }
    };

    let app = tauri::Builder::default()
        // Uma segunda execução só traz a janela existente para a frente.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let api: SharedApi = match Api::new(paths.clone()) {
                // `into_shared` registra a Api para os jobs do agente (threads próprias).
                Ok(a) => a.into_shared(),
                Err(e) => {
                    tracing::error!("não foi possível iniciar o núcleo: {e}");
                    return Err(Box::new(e));
                }
            };
            #[cfg(unix)]
            let mcp_socket = {
                let run_dir = api.paths().run_dir.clone();
                match tauri::async_runtime::block_on(treasury_core::mcp::start_socket_server(
                    api.clone(),
                    &run_dir,
                )) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        tracing::warn!("socket MCP indisponível: {e}");
                        None
                    }
                }
            };
            app.manage(AppState {
                api: api.clone(),
                #[cfg(unix)]
                mcp_socket: std::sync::Mutex::new(mcp_socket),
                mcp_http: tokio::sync::Mutex::new(McpHttpState::default()),
            });
            // macOS: Treasury › Apoiar o projeto… (abre Ajustes › Sobre e apoio), no idioma
            // da instalação (o núcleo já leu o `config.toml`).
            #[cfg(target_os = "macos")]
            {
                use tauri::menu::{Menu, MenuItem};
                let menu = Menu::default(app.handle())?;
                if let Some(app_menu) = menu.items()?.first().and_then(|m| m.as_submenu().cloned())
                {
                    let label = treasury_core::i18n::t("casca-menu-apoiar");
                    let item = MenuItem::with_id(app, "support", label, true, None::<&str>)?;
                    app_menu.insert(&item, 1)?;
                }
                app.set_menu(menu)?;
                app.on_menu_event(|app, e| {
                    if e.id() == "support" {
                        let _ = app.emit("menu://support", ());
                    }
                });
            }
            // Restos do navegador assistido (removido, ver docs/decisions.md): arquivos
            // baixados pelas abas e ainda não importados ficavam em `run/downloads`.
            let _ = std::fs::remove_dir_all(api.paths().run_dir.join("downloads"));
            // Pasta Downloads (ajuste `watch_downloads`): oferece importar arquivos novos.
            downloads::spawn(app.handle().clone());
            // Caixa de entrada de arquivos (~/Treasury/Inbox): o que chega entra no cofre.
            inbox::spawn(app.handle().clone());
            // Dormir, bloquear a tela ou trocar de usuário bloqueia o cofre (ajuste
            // `lock_on_sleep`).
            session_watch::start(app.handle());
            // Build de release: o bundle confere com a própria assinatura (aviso se mudou).
            integrity::check_on_start(app.handle());
            // Release: procura versões novas (ajuste `auto_update`, ligado por padrão).
            updater::spawn(app.handle().clone());
            // Tempo real: toda escrita do agente, de clientes MCP e das rotinas chega à
            // interface como `data://changed`, para as telas abertas se atualizarem.
            let events_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tokio::sync::broadcast::error::RecvError;
                let mut rx = treasury_core::events::subscribe();
                loop {
                    match rx.recv().await {
                        Ok(e) => {
                            let _ = events_handle.emit("data://changed", &e);
                        }
                        Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => break,
                    }
                }
            });

            // Tarefas em segundo plano (docs/background-tasks.md): início, fim e pedido de
            // cancelar chegam como `tasks://changed`; o progresso a interface relê sozinha.
            let tasks_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tokio::sync::broadcast::error::RecvError;
                let mut rx = treasury_core::tasks::subscribe();
                loop {
                    match rx.recv().await {
                        Ok(e) => {
                            let _ = tasks_handle.emit("tasks://changed", &e);
                        }
                        Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => break,
                    }
                }
            });

            // Rotinas: bloqueio por inatividade, backup diário, coletas e agente.
            let handle = app.handle().clone();
            let api_sync = api.clone();
            let api_research = api.clone();
            tauri::async_runtime::spawn(async move {
                let mut every = tokio::time::interval(Duration::from_secs(30));
                loop {
                    every.tick().await;
                    let api = api.clone();
                    match tauri::async_runtime::spawn_blocking(move || api.tick()).await {
                        Ok(Ok(out)) if out.locked => {
                            let _ = handle.emit("vault://locked", ());
                        }
                        Ok(Ok(out)) if out.backup_created => {
                            let _ = handle.emit("backup://created", ());
                        }
                        Ok(Err(e)) => tracing::warn!("rotina: {e}"),
                        _ => {}
                    }
                    // Índices, câmbio e cotações: ao desbloquear e depois a cada 12 h.
                    let api = api_sync.clone();
                    match tauri::async_runtime::spawn_blocking(move || {
                        treasury_core::market::sync::run_all_if_due(&api, 12)
                    })
                    .await
                    {
                        Ok(Ok(Some(report))) => {
                            let _ = handle.emit("market://synced", report.ok);
                        }
                        Ok(Err(treasury_core::error::Error::Cancelled)) => {
                            tracing::info!("coleta de índices e cotações cancelada")
                        }
                        Ok(Err(e)) => tracing::warn!("coleta de índices: {e}"),
                        _ => {}
                    }
                    // Servidor MCP HTTP conforme os ajustes (liga, desliga, troca de porta).
                    reconcile_mcp_http(&handle).await;
                    // Cotações intradiárias (~15 min de atraso): a cada 5 min, só com o cofre
                    // aberto e só para ativos acompanhados cuja bolsa está em pregão.
                    let api = api_sync.clone();
                    match tauri::async_runtime::spawn_blocking(move || {
                        treasury_core::market::intraday::refresh_if_due(&api)
                    })
                    .await
                    {
                        Ok(Ok(Some(report))) if !report.refreshed.is_empty() => {
                            let _ = handle.emit("market://quotes", report.refreshed.len());
                        }
                        Ok(Err(e)) => tracing::warn!("cotações intradiárias: {e}"),
                        _ => {}
                    }
                    // Rotinas do agente (revisão, fechamento do mês, alertas): disparam em
                    // threads próprias; aqui só chegam os alertas novos para a interface.
                    let api = api_sync.clone();
                    match tauri::async_runtime::spawn_blocking(move || {
                        treasury_core::ai::run_due(&api)
                    })
                    .await
                    {
                        Ok(Ok(report)) if !report.new_alerts.is_empty() => {
                            let _ = handle.emit("ai://alerts", &report.new_alerts);
                            // Notificações do sistema (conforme Ajustes › Notificações).
                            for n in report.notices.iter().take(5) {
                                if let Err(e) = notify::show(&n.title, &n.body) {
                                    tracing::warn!("notificação do sistema: {e}");
                                }
                            }
                        }
                        Ok(Err(e)) => tracing::warn!("rotinas do agente: {e}"),
                        _ => {}
                    }
                }
            });
            // Fundamentos (CVM, SEC, B3): a primeira coleta leva alguns minutos, por isso
            // roda numa tarefa própria, conferindo a cada 30 min se já passou um dia.
            tauri::async_runtime::spawn(async move {
                let mut every = tokio::time::interval(Duration::from_secs(30 * 60));
                loop {
                    every.tick().await;
                    let api = api_research.clone();
                    match tauri::async_runtime::spawn_blocking(move || {
                        treasury_core::research::sync::run_if_due(&api, 24)
                    })
                    .await
                    {
                        Ok(Err(treasury_core::error::Error::Cancelled)) => {
                            tracing::info!("coleta de fundamentos cancelada")
                        }
                        Ok(Err(e)) => tracing::warn!("coleta de fundamentos: {e}"),
                        Ok(Ok(Some(_))) => tracing::info!("fundamentos atualizados"),
                        _ => {}
                    }
                }
            });
            Ok(())
        })
        .manage(downloads::DownloadsWatch::default())
        .manage(updater::UpdateState::default())
        .invoke_handler(tauri::generate_handler![
            api_call,
            mcp_http_status,
            mcp_http_apply,
            biometric::biometric_status,
            biometric::unlock_biometric,
            fido2::fido2_present,
            fido2::fido2_register,
            fido2::fido2_unlock,
            downloads::downloads_read,
            downloads::downloads_trash,
            external::open_external,
            quit_app,
            notify::system_notify,
            qr::read_qr_native,
            qr::read_barcode_native,
            qr::read_payment_codes_native,
            ocr::read_text_native,
            sefaz::sefaz_open,
            sefaz::sefaz_capture,
            os_guard::secret_copy,
            os_guard::secure_input,
            updater::update_status,
            updater::update_check,
            updater::update_install,
            translate::translation_status,
            translate::translate_text
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Treasury: erro ao iniciar: {e}");
            std::process::exit(1);
        });

    app.run(|handle, event| {
        if let RunEvent::Exit = event
            && let Some(state) = handle.try_state::<AppState>()
        {
            // Backup ao fechar (se houve mudanças) e bloqueio: zera as chaves da memória.
            if let Err(e) = state.api.shutdown() {
                tracing::warn!("ao fechar: {e}");
            }
            #[cfg(unix)]
            if let Some(sock) = state.mcp_socket.lock().ok().and_then(|mut g| g.take()) {
                tauri::async_runtime::block_on(sock.shutdown());
            }
            if let Some(http) = tauri::async_runtime::block_on(state.mcp_http.lock())
                .server
                .take()
            {
                tauri::async_runtime::block_on(http.shutdown());
            }
        }
    });
}

/// `treasury mcp`: servidor MCP via stdio. Com o app aberto, repassa ao socket local
/// (mesma fila e permissões); com o app fechado, lê em modo somente leitura pela chave do
/// keychain, se habilitado. Roda antes de qualquer coisa do Tauri (sem instância única,
/// sem trava do banco).
pub fn run_mcp() -> i32 {
    init_tracing();
    let paths = match TreasuryPaths::resolve() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("treasury mcp: {e}");
            return 1;
        }
    };
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("treasury mcp: {e}");
            return 1;
        }
    };
    let res = rt.block_on(treasury_core::mcp::run_mcp_cli(paths));
    rt.shutdown_background();
    match res {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("treasury mcp: {e}");
            1
        }
    }
}
