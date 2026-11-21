//! The app's ACL (capability `main-window` + the command manifest from `build.rs`): only the
//! `main` webview, on a page of the app itself, can call commands. Any other webview gets
//! nothing, even on a local page; remote pages reach nothing either.

use tauri::WebviewUrl;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;

/// Same name as the real command (the ACL works by name); here it only replies.
#[tauri::command]
fn api_call() -> &'static str {
    "ok"
}

/// Same, for reading files from the Downloads folder.
#[tauri::command]
fn downloads_read() -> &'static str {
    "ok"
}

/// Same, for sending an imported file from Downloads to the Trash.
#[tauri::command]
fn downloads_trash() -> &'static str {
    "ok"
}

/// Same, for opening a link in the system browser.
#[tauri::command]
fn open_external() -> &'static str {
    "ok"
}

/// Same, for the test notification.
#[tauri::command]
fn system_notify() -> &'static str {
    "ok"
}

/// Same, for closing the app (terms of use screen).
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
            "the interface calls {cmd}"
        );
    }
    // Remote page in the main webview (never happens, but it would gain nothing).
    assert!(get_ipc_response(&main, request("api_call", "https://golpe.com/")).is_err());

    // Another webview (different label), even with a local page: nothing.
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
                "{cmd} on {url}"
            );
        }
        // Plugins (events, dialogs) are not reachable either.
        for cmd in ["plugin:event|listen", "plugin:dialog|open"] {
            assert!(
                get_ipc_response(&other, request(cmd, url)).is_err(),
                "{cmd} on {url}"
            );
        }
    }
}
