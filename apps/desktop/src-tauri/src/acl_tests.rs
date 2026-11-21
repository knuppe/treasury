//! A ACL do app (capability `main-window` + manifesto de comandos do `build.rs`): só a
//! webview `main`, numa página do próprio app, chama comandos. Qualquer outra webview fica
//! sem nada, mesmo numa página local; páginas remotas também não chegam a nada.

use tauri::WebviewUrl;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;

/// Mesmo nome do comando real (a ACL é por nome); aqui só responde.
#[tauri::command]
fn api_call() -> &'static str {
    "ok"
}

/// Idem para a leitura de arquivos da pasta Downloads.
#[tauri::command]
fn downloads_read() -> &'static str {
    "ok"
}

/// Idem para mandar um arquivo importado da pasta Downloads para a Lixeira.
#[tauri::command]
fn downloads_trash() -> &'static str {
    "ok"
}

/// Idem para abrir um link no navegador do sistema.
#[tauri::command]
fn open_external() -> &'static str {
    "ok"
}

/// Idem para a notificação de teste.
#[tauri::command]
fn system_notify() -> &'static str {
    "ok"
}

/// Idem para fechar o app (tela dos termos de uso).
#[tauri::command]
fn quit_app() -> &'static str {
    "ok"
}

fn request(cmd: &str, url: &str) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: url.parse().unwrap(),
        body: InvokeBody::default(),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

#[test]
fn only_the_main_webview_reaches_commands() {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            api_call,
            downloads_read,
            downloads_trash,
            open_external,
            system_notify,
            quit_app
        ])
        .build(tauri::generate_context!(test = true))
        .unwrap();
    let main = tauri::WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
        .build()
        .unwrap();
    let local = "tauri://localhost";
    let commands = [
        "api_call",
        "downloads_read",
        "downloads_trash",
        "open_external",
        "system_notify",
        "quit_app",
    ];
    for cmd in commands {
        assert!(
            get_ipc_response(&main, request(cmd, local)).is_ok(),
            "a interface chama {cmd}"
        );
    }
    // Página remota na webview principal (nunca acontece, mas não ganharia nada).
    assert!(get_ipc_response(&main, request("api_call", "https://golpe.com/")).is_err());

    // Outra webview (outro rótulo), mesmo com uma página local: nada.
    let other = tauri::WebviewWindowBuilder::new(
        &app,
        "outra",
        WebviewUrl::External("https://www.banco.com.br/".parse().unwrap()),
    )
    .build()
    .unwrap();
    for url in ["https://www.banco.com.br/", local] {
        let e = get_ipc_response(&other, request("api_call", url)).unwrap_err();
        assert!(e.to_string().contains("not allowed"), "{url}: {e}");
        for cmd in commands {
            assert!(
                get_ipc_response(&other, request(cmd, url)).is_err(),
                "{cmd} em {url}"
            );
        }
        // Plugins (eventos, diálogos) também não.
        for cmd in ["plugin:event|listen", "plugin:dialog|open"] {
            assert!(
                get_ipc_response(&other, request(cmd, url)).is_err(),
                "{cmd} em {url}"
            );
        }
    }
}
