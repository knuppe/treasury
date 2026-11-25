//! On-device translation of the release notes (Settings › Updates): Apple's Translation
//! framework through the Swift bridge in native/translate.swift. Only macOS 26 or later with
//! the language pair downloaded; everywhere else `translation_status` is false and the screen
//! offers the browser translator instead, which the person opens with an explicit click.
//! The release notes are written in English, so the source language is fixed.
//!
//! The framework answers through the main run loop: call it from a worker thread of the running
//! app, never from the main thread (it would wait forever). For the same reason it cannot run
//! under `cargo test`, whose main thread has no run loop; it was checked in the app itself.

use treasury_core::ApiError;

#[cfg(target_os = "macos")]
const SOURCE: &str = "en";
/// The interface languages other than English.
const TARGETS: &[&str] = &["pt-BR", "pt-PT", "es"];
/// Release notes are a few kilobytes; anything far larger is not ours to translate.
const MAX_TEXT: usize = 20_000;

fn err(code: &str, message: impl Into<String>) -> ApiError {
    ApiError {
        code: code.into(),
        message: message.into(),
        details: None,
    }
}

fn checked_target(target: &str) -> Result<&'static str, ApiError> {
    TARGETS
        .iter()
        .find(|t| **t == target)
        .copied()
        .ok_or_else(|| err("invalid", format!("unsupported language: {target}")))
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::{CStr, CString, c_char};

    unsafe extern "C" {
        fn treasury_translation_status(source: *const c_char, target: *const c_char) -> i32;
        fn treasury_translate(
            source: *const c_char,
            target: *const c_char,
            text: *const c_char,
            ok: *mut i32,
        ) -> *mut c_char;
        fn treasury_translation_free(ptr: *mut c_char);
    }

    fn c(s: &str) -> CString {
        // Interior NULs cannot come from the release notes; drop them rather than fail.
        CString::new(s.replace('\0', "")).unwrap_or_default()
    }

    pub fn installed(source: &str, target: &str) -> bool {
        let (s, t) = (c(source), c(target));
        // SAFETY: both pointers are valid NUL-terminated strings for the call's duration.
        unsafe { treasury_translation_status(s.as_ptr(), t.as_ptr()) == 1 }
    }

    pub fn translate(source: &str, target: &str, text: &str) -> Result<String, String> {
        let (s, t, x) = (c(source), c(target), c(text));
        let mut ok = 0i32;
        // SAFETY: valid C strings in, and the returned buffer is freed by its own allocator.
        unsafe {
            let ptr = treasury_translate(s.as_ptr(), t.as_ptr(), x.as_ptr(), &mut ok);
            if ptr.is_null() {
                return Err("no result".into());
            }
            let out = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            treasury_translation_free(ptr);
            if ok == 1 { Ok(out) } else { Err(out) }
        }
    }
}

/// Whether this computer translates English into `target` on the device right now.
/// Runs off the main thread: the framework may need the main run loop while it answers.
#[tauri::command]
pub async fn translation_status(target: String) -> Result<bool, ApiError> {
    let target = checked_target(&target)?;
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(move || mac::installed(SOURCE, target))
            .await
            .map_err(|e| err("internal", e.to_string()))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = target;
        Ok(false)
    }
}

/// Translates the release notes on the device (line by line, keeping the layout).
#[tauri::command]
pub async fn translate_text(text: String, target: String) -> Result<String, ApiError> {
    let target = checked_target(&target)?;
    if text.len() > MAX_TEXT {
        return Err(err("invalid", "text too long to translate"));
    }
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(move || mac::translate(SOURCE, target, &text))
            .await
            .map_err(|e| err("internal", e.to_string()))?
            .map_err(|e| err("unavailable", e))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (text, target);
        Err(err(
            "unavailable",
            "on-device translation is only available on macOS",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::checked_target;

    #[test]
    fn only_the_interface_languages_are_targets() {
        for ok in ["pt-BR", "pt-PT", "es"] {
            assert!(checked_target(ok).is_ok(), "{ok}");
        }
        for bad in ["en", "pt", "fr", "", "pt-BR\0"] {
            assert!(checked_target(bad).is_err(), "{bad}");
        }
    }
}
