//! Bloqueio do cofre quando o computador dorme, a tela é bloqueada ou outra pessoa entra na
//! sessão (docs/local-security.md, item 3). O núcleo decide pelo ajuste `lock_on_sleep`
//! (ligado por padrão); aqui só chegam os avisos do sistema.
//!
//! - macOS: `NSWorkspaceWillSleepNotification` e `NSWorkspaceSessionDidResignActiveNotification`
//!   (centro do NSWorkspace) e `com.apple.screenIsLocked` (centro distribuído).
//! - Windows: a fazer com `WTSRegisterSessionNotification` (`WM_WTSSESSION_CHANGE`,
//!   `WTS_SESSION_LOCK`) e `WM_POWERBROADCAST` (`PBT_APMSUSPEND`).
//! - Linux: a fazer com o `PrepareForSleep` e o `Lock` do logind (D-Bus).
//!
//! Até lá, nos outros sistemas vale só o bloqueio por inatividade.

use tauri::{AppHandle, Emitter, Manager};

use crate::AppState;

/// Bloqueia (se o ajuste mandar) e avisa a interface, fora da thread do sistema. Por enquanto só
/// o macOS tem quem a chame (Windows e Linux: a fazer, ver o topo do arquivo).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn on_event(app: &AppHandle, what: &'static str) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let api = app.state::<AppState>().api.clone();
        match tauri::async_runtime::spawn_blocking(move || api.lock_on_sleep()).await {
            Ok(Ok(true)) => {
                tracing::info!("cofre bloqueado ({what})");
                let _ = app.emit("vault://locked", ());
            }
            Ok(Err(e)) => tracing::warn!("bloqueio ao dormir ({what}): {e}"),
            _ => {}
        }
    });
}

#[cfg(target_os = "macos")]
pub fn start(app: &AppHandle) {
    mac::watch(app.clone());
}

#[cfg(not(target_os = "macos"))]
pub fn start(_app: &AppHandle) {
    tracing::debug!("bloqueio ao dormir: não disponível neste sistema (só por inatividade)");
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ptr::NonNull;

    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;
    use tauri::AppHandle;

    /// Avisos do centro do NSWorkspace.
    const WORKSPACE: [&str; 2] = [
        "NSWorkspaceWillSleepNotification",
        "NSWorkspaceSessionDidResignActiveNotification",
    ];
    /// Avisos do centro distribuído (tela bloqueada pelo usuário, pelo protetor de tela ou
    /// pelo canto ativo).
    const DISTRIBUTED: [&str; 1] = ["com.apple.screenIsLocked"];

    pub fn watch(app: AppHandle) {
        // SAFETY: mensagens documentadas do AppKit e do Foundation, na thread principal (o
        // `setup` do Tauri roda nela); os observadores e blocos ficam vivos até o app sair.
        unsafe {
            let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
            let ws_center: *mut AnyObject = msg_send![workspace, notificationCenter];
            let dist_center: *mut AnyObject =
                msg_send![class!(NSDistributedNotificationCenter), defaultCenter];
            let queue: *mut AnyObject = msg_send![class!(NSOperationQueue), mainQueue];
            let pairs = WORKSPACE
                .iter()
                .map(|n| (ws_center, *n))
                .chain(DISTRIBUTED.iter().map(|n| (dist_center, *n)));
            for (center, name) in pairs {
                if center.is_null() {
                    continue;
                }
                let app = app.clone();
                let block = RcBlock::new(move |_note: NonNull<AnyObject>| {
                    super::on_event(&app, name);
                });
                let ns_name = NSString::from_str(name);
                let token: *mut AnyObject = msg_send![
                    center,
                    addObserverForName: &*ns_name,
                    object: std::ptr::null_mut::<AnyObject>(),
                    queue: queue,
                    usingBlock: &*block
                ];
                if let Some(t) = Retained::retain(token) {
                    std::mem::forget(t);
                }
                std::mem::forget(block);
            }
        }
    }
}
