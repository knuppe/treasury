//! Lista os catálogos de idioma (`locales/<idioma>/*.ftl`) para o módulo `i18n` (testes de
//! paridade e chaves), e recompila quando um catálogo muda ou ganha arquivo novo.
//!
//! A pasta vem de `CARGO_MANIFEST_DIR` lido ao rodar (não `env!`, fixado ao compilar o script) e
//! o texto é incluído relativo ao crate que compila: com o `target/` dividido entre checkouts
//! (worktrees), um script compilado num checkout não lê nem embute os catálogos de outro.

use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = Path::new(&manifest).join("../../locales");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut out = String::from("&[\n");
    let mut langs: Vec<_> = std::fs::read_dir(&dir)
        .map(|r| r.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    langs.sort();
    for lang in langs.iter().filter(|p| p.is_dir()) {
        let code = lang.file_name().unwrap().to_string_lossy().into_owned();
        let mut files: Vec<_> = std::fs::read_dir(lang)
            .map(|r| r.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        files.sort();
        for f in files
            .iter()
            .filter(|p| p.extension().is_some_and(|e| e == "ftl"))
        {
            println!("cargo:rerun-if-changed={}", f.display());
            let name = f.file_name().unwrap().to_string_lossy();
            let _ = writeln!(
                out,
                "    ({code:?}, {name:?}, include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {:?}))),",
                format!("/../../locales/{code}/{name}")
            );
        }
    }
    out.push(']');
    let target = Path::new(&std::env::var("OUT_DIR").unwrap()).join("locales.rs");
    std::fs::write(target, out).unwrap();
}
