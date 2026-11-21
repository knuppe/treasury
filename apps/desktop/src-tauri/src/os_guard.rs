//! Proteções do sistema para segredos na tela (docs/local-security.md, itens 9 e 12).
//!
//! - `secret_copy`: copia uma parte de Shamir, chave de API ou código marcando o conteúdo
//!   como oculto (`org.nspasteboard.ConcealedType`, que os gerenciadores de histórico da área
//!   de transferência respeitam), só neste Mac (fora da área de transferência universal) e
//!   apagado depois de 60 s se ninguém copiou outra coisa.
//! - `secure_input`: liga o Secure Event Input enquanto um campo de senha tem o foco, inclusive
//!   com a senha à mostra (o WebKit só liga sozinho em `type=password`). Com ele ligado, outro
//!   processo não lê as teclas por event tap.
//!
//! No Windows e no Linux, `secret_copy` responde que não há e a interface copia do jeito
//! normal; `secure_input` não faz nada. No Windows, o caminho é gravar junto os formatos
//! `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory = 0` e
//! `CanUploadToCloudClipboard = 0` (features `Win32_System_DataExchange` do crate `windows`).

use tauri::AppHandle;
use treasury_core::vault::Secret;

/// Tempo até apagar o que foi copiado.
#[cfg(target_os = "macos")]
const CLEAR_AFTER: std::time::Duration = std::time::Duration::from_secs(60);

#[tauri::command]
pub fn secret_copy(app: AppHandle, text: Secret) -> Result<(), String> {
    imp::copy(&app, &text)
}

#[tauri::command]
pub fn secure_input(on: bool) {
    imp::secure_input(on);
}

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::atomic::{AtomicBool, Ordering};

    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;
    use tauri::AppHandle;

    /// `NSPasteboardTypeString` e o tipo de conteúdo oculto do nspasteboard.org.
    const STRING_TYPE: &str = "public.utf8-plain-text";
    const CONCEALED_TYPE: &str = "org.nspasteboard.ConcealedType";
    /// `NSPasteboardContentsCurrentHostOnly`: não vai para o iPhone nem para outro Mac.
    const CURRENT_HOST_ONLY: usize = 1;

    #[link(name = "Carbon", kind = "framework")]
    unsafe extern "C" {
        fn EnableSecureEventInput() -> i32;
        fn DisableSecureEventInput() -> i32;
        #[cfg(test)]
        fn IsSecureEventInputEnabled() -> u8;
    }

    /// O Secure Event Input conta chamadas por processo; isto garante uma só ligada por vez.
    static SECURE_INPUT: AtomicBool = AtomicBool::new(false);

    pub fn secure_input(on: bool) {
        if SECURE_INPUT.swap(on, Ordering::SeqCst) == on {
            return;
        }
        // SAFETY: funções sem argumentos do HIToolbox, equilibradas pelo `swap` acima.
        unsafe {
            if on {
                EnableSecureEventInput();
            } else {
                DisableSecureEventInput();
            }
        }
    }

    pub fn copy(app: &AppHandle, text: &str) -> Result<(), String> {
        // SAFETY: mensagem documentada do NSPasteboard, na thread principal (comando síncrono).
        let count = unsafe { write_concealed(general()?, text)? };
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(super::CLEAR_AFTER);
            let _ = app.run_on_main_thread(move || {
                // SAFETY: idem, de volta na thread principal.
                unsafe {
                    if let Ok(pb) = general() {
                        clear_if_unchanged(pb, count);
                    }
                }
            });
        });
        Ok(())
    }

    unsafe fn general() -> Result<*mut AnyObject, String> {
        // SAFETY: classe e mensagem documentadas do AppKit.
        let pb: *mut AnyObject = unsafe { msg_send![class!(NSPasteboard), generalPasteboard] };
        if pb.is_null() {
            return Err("área de transferência indisponível".into());
        }
        Ok(pb)
    }

    /// Grava o texto marcado como oculto e devolve o `changeCount` da área de transferência.
    ///
    /// # Safety
    /// `pb` é um NSPasteboard vivo.
    unsafe fn write_concealed(pb: *mut AnyObject, text: &str) -> Result<isize, String> {
        // SAFETY: mensagens documentadas do NSPasteboard; os NSString vivem até o fim do bloco.
        unsafe {
            let _: isize = msg_send![pb, prepareForNewContentsWithOptions: CURRENT_HOST_ONLY];
            let value = NSString::from_str(text);
            let empty = NSString::from_str("");
            let ok: bool =
                msg_send![pb, setString: &*value, forType: &*NSString::from_str(STRING_TYPE)];
            let _: bool =
                msg_send![pb, setString: &*empty, forType: &*NSString::from_str(CONCEALED_TYPE)];
            if !ok {
                return Err("não foi possível copiar".into());
            }
            Ok(msg_send![pb, changeCount])
        }
    }

    /// Apaga só se a área de transferência ainda tem o que o Treasury copiou.
    ///
    /// # Safety
    /// `pb` é um NSPasteboard vivo.
    unsafe fn clear_if_unchanged(pb: *mut AnyObject, count: isize) {
        // SAFETY: mensagens documentadas do NSPasteboard.
        unsafe {
            let now: isize = msg_send![pb, changeCount];
            if now == count {
                let _: isize = msg_send![pb, clearContents];
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        use objc2_foundation::NSString;

        /// Turning it on twice and off once leaves it off: the calls stay balanced.
        /// The state is system-wide: if another program already enables secure input (an open
        /// password field, Terminal's "Secure Keyboard Entry"), it goes back to that, not to 0.
        #[test]
        fn secure_input_calls_stay_balanced() {
            // SAFETY: query with no arguments.
            let before = unsafe { super::IsSecureEventInputEnabled() };
            super::secure_input(true);
            super::secure_input(true);
            assert_eq!(unsafe { super::IsSecureEventInputEnabled() }, 1);
            super::secure_input(false);
            assert_eq!(unsafe { super::IsSecureEventInputEnabled() }, before);
        }

        /// On the test's own pasteboard (does not touch the runner's): the text goes with the
        /// concealed type, disappears if nobody copied afterwards, and stays if someone did.
        #[test]
        fn concealed_copy_is_marked_and_cleared_only_if_unchanged() {
            // SAFETY: private NSPasteboard, created and released here.
            unsafe {
                let pb: *mut AnyObject = msg_send![class!(NSPasteboard), pasteboardWithUniqueName];
                let types = |pb: *mut AnyObject| -> Vec<String> {
                    let list: *mut AnyObject = msg_send![pb, types];
                    if list.is_null() {
                        return vec![];
                    }
                    let n: usize = msg_send![list, count];
                    (0..n)
                        .map(|i| {
                            let s: *mut NSString = msg_send![list, objectAtIndex: i];
                            (*s).to_string()
                        })
                        .collect()
                };
                let count = super::write_concealed(pb, "parte-inventada").unwrap();
                let got = types(pb);
                assert!(got.iter().any(|t| t == super::CONCEALED_TYPE), "{got:?}");
                assert!(got.iter().any(|t| t == super::STRING_TYPE), "{got:?}");
                super::clear_if_unchanged(pb, count);
                assert!(types(pb).is_empty());

                let count = super::write_concealed(pb, "parte-inventada").unwrap();
                let _: isize = msg_send![pb, clearContents];
                let other = NSString::from_str("outra coisa");
                let _: bool = msg_send![pb, setString: &*other, forType: &*NSString::from_str(super::STRING_TYPE)];
                super::clear_if_unchanged(pb, count);
                assert!(!types(pb).is_empty());
                let _: () = msg_send![pb, releaseGlobally];
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use tauri::AppHandle;

    pub fn secure_input(_on: bool) {}

    pub fn copy(_app: &AppHandle, _text: &str) -> Result<(), String> {
        Err("unsupported".into())
    }
}
