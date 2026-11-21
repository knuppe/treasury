//! Conferência da própria assinatura ao abrir (docs/local-security.md, item 15). Um app
//! instalado que alguém trocou ou alterou (para capturar a senha no próximo desbloqueio)
//! deixa de bater com a assinatura do bundle; o Treasury avisa antes de pedir a senha.
//!
//! Só vale no build de release, dentro de um `.app` (o binário de desenvolvimento não é
//! assinado como bundle). Com assinatura ad hoc, prova que o bundle não mudou desde que foi
//! assinado, não quem assinou; com Developer ID, prova os dois. No Linux e no Windows, nada
//! por enquanto (no Windows: `WinVerifyTrust` no executável).

use tauri::AppHandle;

/// Confere e, se a assinatura não bater, mostra um aviso do sistema.
pub fn check_on_start(app: &AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    if let Some(problem) = problem() {
        tracing::warn!("assinatura do app: {problem}");
        use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
        app.dialog()
            .message(treasury_core::i18n::t("casca-assinatura-alterada"))
            .title(treasury_core::i18n::t("casca-assinatura-titulo"))
            .kind(MessageDialogKind::Warning)
            .show(|_| {});
    }
}

/// `Some(motivo)` quando o bundle não confere com a própria assinatura.
#[cfg(target_os = "macos")]
fn problem() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    // …/Treasury.app/Contents/MacOS/treasury → …/Treasury.app
    let bundle = exe.parent()?.parent()?.parent()?;
    if bundle.extension().and_then(|e| e.to_str()) != Some("app") {
        return None;
    }
    mac::check(bundle).err()
}

#[cfg(not(target_os = "macos"))]
fn problem() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::c_void;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    type CFTypeRef = *const c_void;
    type OSStatus = i32;

    /// Confere tudo o que estiver selado no bundle, inclusive código aninhado.
    const K_SEC_CS_CHECK_ALL_ARCHITECTURES: u32 = 1 << 0;
    const K_SEC_CS_CHECK_NESTED_CODE: u32 = 1 << 3;
    const K_SEC_CS_STRICT_VALIDATE: u32 = 1 << 4;
    /// `errSecCSUnsigned` e `errSecCSResourcesNotSealed`.
    const ERR_SEC_CS_UNSIGNED: OSStatus = -67062;
    const ERR_SEC_CS_RESOURCES_NOT_SEALED: OSStatus = -67056;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFURLCreateFromFileSystemRepresentation(
            allocator: CFTypeRef,
            buffer: *const u8,
            len: isize,
            is_directory: u8,
        ) -> CFTypeRef;
        fn CFRelease(cf: CFTypeRef);
    }

    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        fn SecStaticCodeCreateWithPath(
            path: CFTypeRef,
            flags: u32,
            static_code: *mut CFTypeRef,
        ) -> OSStatus;
        fn SecStaticCodeCheckValidity(
            static_code: CFTypeRef,
            flags: u32,
            requirement: CFTypeRef,
        ) -> OSStatus;
    }

    pub fn check(bundle: &Path) -> Result<(), String> {
        let bytes = bundle.as_os_str().as_bytes();
        // SAFETY: chamadas do Security e do CoreFoundation com objetos que este bloco cria e
        // solta; nenhum ponteiro sai daqui.
        unsafe {
            let url = CFURLCreateFromFileSystemRepresentation(
                std::ptr::null(),
                bytes.as_ptr(),
                bytes.len() as isize,
                1,
            );
            if url.is_null() {
                return Err("caminho do app inválido".into());
            }
            let mut code: CFTypeRef = std::ptr::null();
            let status = SecStaticCodeCreateWithPath(url, 0, &mut code);
            CFRelease(url);
            if status != 0 || code.is_null() {
                return Err(format!("SecStaticCodeCreateWithPath: {status}"));
            }
            let flags = K_SEC_CS_CHECK_ALL_ARCHITECTURES
                | K_SEC_CS_CHECK_NESTED_CODE
                | K_SEC_CS_STRICT_VALIDATE;
            let status = SecStaticCodeCheckValidity(code, flags, std::ptr::null());
            CFRelease(code);
            match status {
                0 => Ok(()),
                // Bundle que nunca foi assinado como bundle (só a assinatura do linker, como
                // sai de um `tauri build` sem o scripts/release.sh): não há o que conferir.
                ERR_SEC_CS_UNSIGNED | ERR_SEC_CS_RESOURCES_NOT_SEALED => {
                    tracing::info!(
                        "assinatura do app: bundle sem selo ({status}); nada a conferir"
                    );
                    Ok(())
                }
                _ => Err(format!("SecStaticCodeCheckValidity: {status}")),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        /// Uma pasta com nome .app e sem nada dentro não abre como código.
        #[test]
        fn a_folder_that_is_not_a_bundle_is_reported() {
            let dir = tempfile_dir();
            let app = dir.join("Falso.app");
            std::fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
            assert!(super::check(&app).is_err());
            let _ = std::fs::remove_dir_all(&dir);
        }

        /// Conferência à mão de um bundle de verdade: `TREASURY_APP_BUNDLE=/caminho/Treasury.app`.
        #[test]
        #[ignore]
        fn bundle_from_env() {
            let Ok(p) = std::env::var("TREASURY_APP_BUNDLE") else {
                return;
            };
            println!("{p}: {:?}", super::check(std::path::Path::new(&p)));
        }

        fn tempfile_dir() -> std::path::PathBuf {
            let d = std::env::temp_dir().join(format!("treasury-integrity-{}", std::process::id()));
            std::fs::create_dir_all(&d).unwrap();
            d
        }
    }
}
