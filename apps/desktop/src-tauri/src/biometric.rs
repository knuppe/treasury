//! Touch ID / Apple Watch / senha do Mac antes do desbloqueio pelo Keychain (macOS,
//! LocalAuthentication com `DeviceOwnerAuthentication`). A interface não chama
//! `vault.unlock_keychain` direto: o `api_call` recusa, e só [`unlock_biometric`] desbloqueia,
//! depois da confirmação do sistema. Em outros sistemas não há biometria aqui.

use serde_json::{Value, json};
use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Manager;
use treasury_core::ApiError;

#[cfg(target_os = "macos")]
use crate::AppState;

fn err(code: &str, message: impl Into<String>) -> ApiError {
    ApiError {
        code: code.into(),
        message: message.into(),
        details: None,
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::sync::mpsc;
    use std::time::Duration;

    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSError, NSString};
    use objc2_local_authentication::{LABiometryType, LAContext, LAPolicy};

    const POLICY: LAPolicy = LAPolicy::DeviceOwnerAuthentication;

    /// `Some("touch_id" | "password")` se dá para confirmar a presença do dono.
    pub fn kind() -> Option<&'static str> {
        let ctx = unsafe { LAContext::new() };
        unsafe { ctx.canEvaluatePolicy_error(POLICY) }.ok()?;
        let bio = unsafe { ctx.biometryType() };
        Some(
            if bio == LABiometryType::TouchID || bio == LABiometryType::FaceID {
                "touch_id"
            } else {
                "password"
            },
        )
    }

    pub fn authenticate(reason: &str) -> Result<(), String> {
        let ctx = unsafe { LAContext::new() };
        unsafe { ctx.canEvaluatePolicy_error(POLICY) }
            .map_err(|e| e.localizedDescription().to_string())?;
        let (tx, rx) = mpsc::channel::<Option<String>>();
        let block = RcBlock::new(move |ok: Bool, error: *mut NSError| {
            let msg = if ok.as_bool() {
                None
            } else {
                Some(
                    unsafe { error.as_ref() }
                        .map(|e| e.localizedDescription().to_string())
                        .unwrap_or_else(|| "confirmação cancelada".into()),
                )
            };
            let _ = tx.send(msg);
        });
        unsafe {
            ctx.evaluatePolicy_localizedReason_reply(POLICY, &NSString::from_str(reason), &block)
        };
        match rx.recv_timeout(Duration::from_secs(120)) {
            Ok(None) => Ok(()),
            Ok(Some(m)) => Err(m),
            Err(_) => Err("tempo esgotado para a confirmação".into()),
        }
    }
}

/// Se o desbloqueio com confirmação do sistema está disponível neste computador.
#[tauri::command]
pub fn biometric_status() -> Value {
    #[cfg(target_os = "macos")]
    let kind = mac::kind();
    #[cfg(not(target_os = "macos"))]
    let kind: Option<&str> = None;
    json!({ "available": kind.is_some(), "kind": kind })
}

/// Pede Touch ID (ou a senha do Mac) e, confirmado, desbloqueia pelo Keychain.
#[tauri::command]
pub async fn unlock_biometric(app: AppHandle) -> Result<Value, ApiError> {
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(|| {
            mac::authenticate("desbloquear o cofre do Treasury")
        })
        .await
        .map_err(|e| err("internal", e.to_string()))?
        .map_err(|m| err("auth_failed", m))?;
        let api = app.state::<AppState>().api.clone();
        tauri::async_runtime::spawn_blocking(move || api.call("vault.unlock_keychain", Value::Null))
            .await
            .map_err(|e| err("internal", e.to_string()))?
            .map_err(ApiError::from)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        Err(err(
            "unsupported",
            "o desbloqueio por biometria só existe no macOS; use o PIN",
        ))
    }
}
