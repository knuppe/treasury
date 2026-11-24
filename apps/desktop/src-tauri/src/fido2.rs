//! Chave física FIDO2 (YubiKey, SoloKey…) para recuperar o cofre, com a extensão
//! hmac-secret: a chave devolve um segredo de 32 bytes para um salt guardado no arquivo
//! `.keys`, e esse segredo abre a chave de recuperação do cofre (ver `treasury_core::unlock`).
//! O segredo nunca vai para a interface: este módulo fala com a chave pelo USB e repassa
//! direto ao núcleo, em Rust.

use std::sync::{Mutex, OnceLock, mpsc};

use base64::Engine as _;
use ctap_hid_fido2::fidokey::{
    AssertionExtension as Gext, CredentialExtension as Mext, GetAssertionArgsBuilder,
    MakeCredentialArgsBuilder,
};
use ctap_hid_fido2::{FidoKeyHid, FidoKeyHidFactory, LibCfg};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
use treasury_core::ApiError;
use zeroize::Zeroizing;

use crate::AppState;

/// Identificador do "site" para a credencial (nunca usado na web).
const RP_ID: &str = "treasury.local";

fn err(code: &str, message: impl Into<String>) -> ApiError {
    ApiError {
        code: code.into(),
        message: message.into(),
        details: None,
    }
}

type HidJob = Box<dyn FnOnce() + Send>;

/// Todo acesso ao USB (hidapi/IOHIDManager) acontece numa única thread que vive o app
/// inteiro: no macOS o gerenciador HID fica preso ao run loop da thread que o criou, e
/// usá-lo de novo a partir de outra thread (ou depois que a primeira morreu) derruba o app.
fn hid_sender() -> &'static Mutex<mpsc::Sender<HidJob>> {
    static TX: OnceLock<Mutex<mpsc::Sender<HidJob>>> = OnceLock::new();
    TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<HidJob>();
        std::thread::Builder::new()
            .name("treasury-hid".into())
            .spawn(move || {
                for job in rx {
                    job();
                }
            })
            .expect("thread do USB");
        Mutex::new(tx)
    })
}

async fn on_hid<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T, ApiError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    hid_sender()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .send(Box::new(move || {
            let _ = tx.send(f());
        }))
        .map_err(|_| err("internal", "a thread do USB parou"))?;
    rx.await
        .map_err(|_| err("internal", "a thread do USB parou"))
}

fn device() -> Result<FidoKeyHid, ApiError> {
    FidoKeyHidFactory::create(&LibCfg::init()).map_err(|_| {
        err(
            "not_found",
            "nenhuma chave física encontrada: conecte a chave (YubiKey ou outra FIDO2) e tente de novo",
        )
    })
}

fn random32() -> Result<[u8; 32], ApiError> {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).map_err(|e| err("internal", e.to_string()))?;
    Ok(b)
}

/// The key's answer, in the person's terms. `pin_given`: a PIN was sent with the request, so
/// "the key asked for the PIN" would be wrong; the real code goes in the message instead.
fn fido_err(e: impl std::fmt::Display, pin_given: bool) -> ApiError {
    let m = e.to_string();
    let u = m.to_uppercase();
    if u.contains("PIN_INVALID") {
        err(
            "pin_invalid",
            "PIN incorreto: confira e tente de novo (a chave bloqueia o PIN depois de várias tentativas erradas)",
        )
    } else if u.contains("PIN_BLOCKED") || u.contains("PIN_AUTH_BLOCKED") {
        err(
            "pin_blocked",
            "a chave física bloqueou o PIN depois de tentativas erradas: tire a chave, coloque de novo e tente outra vez",
        )
    } else if !pin_given
        && (u.contains("PIN_REQUIRED") || u.contains("PUAT_REQUIRED") || u.contains("PIN NOT SET"))
    {
        err(
            "pin_required",
            "a chave física pediu o PIN dela: informe e tente de novo",
        )
    } else {
        err(
            "fido2",
            format!("a chave física recusou ({m}); toque na chave quando ela piscar"),
        )
    }
}

/// Precisa do PIN para responder (chave com "sempre verificar o usuário")?
fn needs_pin(e: &ApiError) -> bool {
    e.code == "pin_required"
}

/// The hmac-secret of a credential. First without the PIN: the secret does not need user
/// verification, and asking for the PIN token here makes the library open a second key
/// agreement after the one that encrypted the salt, which the key rejects (`PIN_AUTH_INVALID`
/// with the right PIN). Only a key that always demands verification gets the PIN.
fn hmac_secret(
    dev: &FidoKeyHid,
    credential_id: &[u8],
    salt: [u8; 32],
    pin: Option<&str>,
) -> Result<Zeroizing<[u8; 32]>, ApiError> {
    match hmac_secret_with(dev, credential_id, salt, None) {
        Err(e) if needs_pin(&e) && pin.is_some() => hmac_secret_with(dev, credential_id, salt, pin),
        other => other,
    }
}

fn hmac_secret_with(
    dev: &FidoKeyHid,
    credential_id: &[u8],
    salt: [u8; 32],
    pin: Option<&str>,
) -> Result<Zeroizing<[u8; 32]>, ApiError> {
    let challenge = random32()?;
    let ext = [Gext::HmacSecret(Some(salt))];
    let builder = GetAssertionArgsBuilder::new(RP_ID, &challenge)
        .credential_id(credential_id)
        .extensions(&ext);
    // Without the PIN, no "uv" option either: the library sends `uv: true` by default, and a key
    // without built-in verification (YubiKey) refuses it (`CTAP2_ERR_INVALID_OPTION`).
    let args = match pin {
        Some(p) => builder.pin(p),
        None => builder.without_pin_and_uv(),
    }
    .build();
    let ass = dev
        .get_assertion_with_args(&args)
        .map_err(|e| fido_err(e, pin.is_some()))?
        .into_iter()
        .next()
        .ok_or_else(|| err("fido2", "a chave física não respondeu"))?;
    ass.extensions
        .iter()
        .find_map(|e| match e {
            Gext::HmacSecret(Some(out)) => Some(Zeroizing::new(*out)),
            _ => None,
        })
        .ok_or_else(|| err("fido2", "esta chave física não suporta hmac-secret"))
}

/// Há uma chave física conectada.
#[tauri::command]
pub async fn fido2_present() -> bool {
    on_hid(|| device().is_ok()).await.unwrap_or(false)
}

/// Cadastra a chave conectada (dois toques: criar a credencial e confirmar o segredo).
#[tauri::command]
pub async fn fido2_register(
    app: AppHandle,
    label: String,
    pin: Option<String>,
) -> Result<Value, ApiError> {
    let (cred, salt, secret) = on_hid(move || {
        let dev = device()?;
        let pin = pin.as_deref().filter(|p| !p.is_empty());
        let challenge = random32()?;
        let ext = [Mext::HmacSecret(Some(true))];
        let builder = MakeCredentialArgsBuilder::new(RP_ID, &challenge).extensions(&ext);
        // The PIN when there is one; without it, no "uv" option (see `hmac_secret_with`).
        let args = match pin {
            Some(p) => builder.pin(p),
            None => builder.without_pin_and_uv(),
        }
        .build();
        let att = dev
            .make_credential_with_args(&args)
            .map_err(|e| fido_err(e, pin.is_some()))?;
        let cred = att.credential_descriptor.id.clone();
        let salt = random32()?;
        let secret = hmac_secret(&dev, &cred, salt, pin)?;
        Ok::<_, ApiError>((cred, salt, secret))
    })
    .await??;
    let api = app.state::<AppState>().api.clone();
    let args = json!({
        "label": if label.trim().is_empty() { "Chave física".to_string() } else { label },
        "rp_id": RP_ID,
        "credential_id": base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&cred),
        "salt": hex::encode(salt),
        "hmac_secret": hex::encode(secret.as_ref()),
    });
    tauri::async_runtime::spawn_blocking(move || api.call("vault.fido2_add", args))
        .await
        .map_err(|e| err("internal", e.to_string()))?
        .map_err(ApiError::from)
}

/// Recupera o cofre com uma chave física cadastrada (exige senha mestra nova).
#[tauri::command]
pub async fn fido2_unlock(
    app: AppHandle,
    slot_id: String,
    new_password: String,
    pin: Option<String>,
) -> Result<Value, ApiError> {
    let api = app.state::<AppState>().api.clone();
    let slots = {
        let api = api.clone();
        tauri::async_runtime::spawn_blocking(move || api.call("vault.fido2_slots", Value::Null))
            .await
            .map_err(|e| err("internal", e.to_string()))?
            .map_err(ApiError::from)?
    };
    let slot = slots
        .as_array()
        .into_iter()
        .flatten()
        .find(|s| s["id"] == slot_id.as_str())
        .cloned()
        .ok_or_else(|| err("not_found", "chave física não cadastrada"))?;
    let cred = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(slot["credential_id"].as_str().unwrap_or_default())
        .map_err(|_| err("invalid", "credencial ilegível"))?;
    let salt: [u8; 32] = hex::decode(slot["salt"].as_str().unwrap_or_default())
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| err("invalid", "salt ilegível"))?;
    let secret = on_hid(move || {
        let dev = device()?;
        hmac_secret(&dev, &cred, salt, pin.as_deref().filter(|p| !p.is_empty()))
    })
    .await??;
    let args = json!({
        "slot_id": slot_id,
        "hmac_secret": hex::encode(secret.as_ref()),
        "new_password": new_password,
    });
    tauri::async_runtime::spawn_blocking(move || api.call("vault.unlock_fido2", args))
        .await
        .map_err(|e| err("internal", e.to_string()))?
        .map_err(ApiError::from)
}
