fn main() {
    translation_bridge();
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
            "translation_status",
            "translate_text",
        ]),
    ))
    .expect("tauri-build");
}

/// macOS: compiles native/translate.swift (Apple's Translation framework, Swift-only API) into a
/// static library and links it. Translation and the Swift concurrency runtime are weak, so the
/// app still starts on macOS 11 to 25; the Swift side checks for macOS 26 before using them.
fn translation_bridge() {
    println!("cargo:rerun-if-changed=native/translate.swift");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "arm64",
        Ok("x86_64") => "x86_64",
        other => panic!("translation bridge: unsupported arch {other:?}"),
    };
    let min = std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "11.0".into());
    let lib = out.join("libtreasury_translate.a");
    let status = std::process::Command::new("xcrun")
        .args([
            "swiftc",
            "-parse-as-library",
            "-emit-library",
            "-static",
            "-O",
        ])
        .args(["-module-name", "TreasuryTranslate", "-target"])
        .arg(format!("{arch}-apple-macos{min}"))
        .arg("native/translate.swift")
        .arg("-o")
        .arg(&lib)
        .status()
        .expect("translation bridge: xcrun swiftc");
    assert!(status.success(), "translation bridge: swiftc failed");
    let swiftc = std::process::Command::new("xcrun")
        .args(["-f", "swiftc"])
        .output()
        .expect("xcrun -f swiftc");
    let swiftc = std::path::PathBuf::from(String::from_utf8_lossy(&swiftc.stdout).trim());
    // <toolchain>/usr/bin/swiftc -> <toolchain>/usr/lib/swift/macosx (compatibility libraries).
    let toolchain = swiftc.parent().and_then(|p| p.parent()).expect("toolchain");
    println!(
        "cargo:rustc-link-search=native={}",
        toolchain.join("lib/swift/macosx").display()
    );
    let sdk = std::process::Command::new("xcrun")
        .args(["--show-sdk-path"])
        .output()
        .expect("xcrun --show-sdk-path");
    println!(
        "cargo:rustc-link-search=native={}/usr/lib/swift",
        String::from_utf8_lossy(&sdk.stdout).trim()
    );
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=treasury_translate");
    println!("cargo:rustc-link-arg=-Wl,-weak_framework,Translation");
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
}
