fn main() {
    // Manifesto de ACL do app: cada comando precisa de permissão explícita na capability.
    // Só a webview `main` recebe; qualquer outra webview não chama nada.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "api_call",
            "mcp_http_status",
            "mcp_http_apply",
            "biometric_status",
            "unlock_biometric",
            "fido2_present",
            "fido2_register",
            "fido2_unlock",
            "downloads_read",
            "downloads_trash",
            "open_external",
            "system_notify",
            "quit_app",
            "read_qr_native",
            "read_barcode_native",
            "read_payment_codes_native",
            "read_text_native",
            "sefaz_open",
            "sefaz_capture",
            "secret_copy",
            "secure_input",
            "update_status",
            "update_check",
            "update_install",
        ]),
    ))
    .expect("tauri-build");
}
