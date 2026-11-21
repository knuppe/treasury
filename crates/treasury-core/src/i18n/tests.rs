//! Languages: codes, fallback, catalog parity (every pt-BR key in en and es, with the
//! same arguments) and per-language formatting.

use std::collections::{BTreeMap, BTreeSet};

use super::*;

#[test]
fn codes_and_fallback_chain() {
    assert_eq!(Locale::parse("pt"), Some(Locale::PtBr));
    assert_eq!(Locale::parse("pt_BR"), Some(Locale::PtBr));
    assert_eq!(Locale::parse("pt-PT"), Some(Locale::PtPt));
    assert_eq!(Locale::parse("en-US"), Some(Locale::En));
    assert_eq!(Locale::parse("es-419"), Some(Locale::Es));
    assert_eq!(Locale::parse("fr"), None);
    assert_eq!(Locale::Es.chain(), &[Locale::Es, Locale::En, Locale::PtBr]);
    assert_eq!(tr(Locale::En, "comum-salvar", None), "Save");
    assert_eq!(tr(Locale::Es, "comum-salvar", None), "Guardar");
    // pt-PT without the key: falls back entirely to pt-BR.
    assert_eq!(tr(Locale::PtPt, "comum-cancelar", None), "Cancelar");
    assert_eq!(tr(Locale::PtPt, "comum-salvar", None), "Guardar");
    assert_eq!(
        tr(Locale::En, "chave-que-nao-existe", None),
        "chave-que-nao-existe"
    );
}

/// `{ $nome }` of each message in an `.ftl`.
fn variables(src: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in src.lines() {
        if line.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            current = line.split_once('=').map(|(k, _)| k.trim().to_string());
        } else if line.trim().is_empty() || line.starts_with('#') {
            // A comment or blank line does not close the message (continuation is indented).
        }
        let Some(k) = &current else { continue };
        let set = out.entry(k.clone()).or_default();
        let mut rest = line;
        while let Some(i) = rest.find("$") {
            rest = &rest[i + 1..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if !name.is_empty() {
                set.insert(name);
            }
        }
    }
    out
}

fn vars_of(l: Locale) -> BTreeMap<String, BTreeSet<String>> {
    sources(l).iter().flat_map(|(_, s)| variables(s)).collect()
}

#[test]
fn en_and_es_are_complete_with_the_same_arguments() {
    let base = catalog_keys(Locale::PtBr);
    assert!(!base.is_empty());
    let base_vars = vars_of(Locale::PtBr);
    for l in [Locale::En, Locale::Es] {
        let keys = catalog_keys(l);
        let missing: Vec<_> = base.difference(keys).collect();
        assert!(missing.is_empty(), "{} missing: {missing:?}", l.code());
        let extra: Vec<_> = keys.difference(base).collect();
        assert!(extra.is_empty(), "{} has extra keys: {extra:?}", l.code());
        let vars = vars_of(l);
        for (k, v) in &base_vars {
            assert_eq!(vars.get(k), Some(v), "{}: arguments of {k}", l.code());
        }
    }
    // pt-PT may be partial, but with no key that pt-BR lacks.
    let extra: Vec<_> = catalog_keys(Locale::PtPt).difference(base).collect();
    assert!(extra.is_empty(), "pt-PT has extra keys: {extra:?}");
}

#[test]
fn every_file_exists_in_every_complete_locale() {
    let files = |l: Locale| -> BTreeSet<&str> { sources(l).iter().map(|(f, _)| *f).collect() };
    assert_eq!(files(Locale::En), files(Locale::PtBr));
    assert_eq!(files(Locale::Es), files(Locale::PtBr));
}

#[test]
fn numbers_money_and_dates_follow_the_locale() {
    assert_eq!(number(Locale::PtBr, 123_456), "1.234,56");
    assert_eq!(number(Locale::En, 123_456), "1,234.56");
    assert_eq!(number(Locale::Es, -123_456), "\u{2212}1.234,56");
    assert_eq!(number(Locale::PtPt, 123_456), "1234,56");
    assert_eq!(number(Locale::PtPt, 1_234_567), "12\u{202F}345,67");
    assert_eq!(money(Locale::PtBr, 123_456, "BRL"), "R$ 1.234,56");
    assert_eq!(money(Locale::En, 123_456, "USD"), "$1,234.56");
    assert_eq!(money(Locale::En, -500, "USD"), "\u{2212}$5.00");
    assert_eq!(money(Locale::PtBr, 123_456, "USD"), "USD 1.234,56");
    assert_eq!(money(Locale::PtPt, 123_456, "EUR"), "1234,56 €");
    let d = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap();
    assert_eq!(date(Locale::PtBr, d), "14/10/2026");
    assert_eq!(date(Locale::En, d), "Oct 14, 2026");
    assert_eq!(month_names(Locale::Es, false)[9], "octubre");
}

#[test]
fn installation_locale_lives_in_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("config.toml");
    assert!(!state(&cfg).chosen);
    let s = set(
        &cfg,
        &SetLocaleArgs {
            locale: "es".into(),
        },
    )
    .unwrap();
    assert!(s.chosen);
    assert_eq!(s.locale, Locale::Es);
    assert_eq!(load(&cfg), Locale::Es);
    assert!(
        std::fs::read_to_string(&cfg)
            .unwrap()
            .contains("locale = \"es\"")
    );
    assert!(
        set(
            &cfg,
            &SetLocaleArgs {
                locale: "fr".into()
            }
        )
        .is_err()
    );
    let names: Vec<_> = s.available.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Português (Brasil)",
            "English",
            "Español",
            "Português (Portugal)"
        ]
    );
    set_current(Locale::PtBr);
}

/// Every key written in the core code (and in the native shell) exists in pt-BR: a literal with
/// an area prefix (`"erro-…"`, `"casca-…"`) that is not in the catalog is a typo.
#[test]
fn keys_used_in_code_exist() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let areas: BTreeSet<String> = sources(Locale::PtBr)
        .iter()
        .map(|(file, _)| file.trim_end_matches(".ftl").to_string())
        .collect();
    // Whole literal that looks like a key; the area is the file name (`nucleo-saude`, `erros`).
    let key = regex::Regex::new(r#""([a-z][a-z0-9_]*(?:-[a-z0-9_]+)+)""#).unwrap();
    let known = catalog_keys(Locale::PtBr);
    let mut missing = Vec::new();
    let mut stack = vec![
        root.join("src"),
        root.join("../../apps/desktop/src-tauri/src"),
    ];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                // Test-only files have names that look like keys ("agente-externo").
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("tests"))
            {
                let src = std::fs::read_to_string(&path).unwrap();
                for c in key.captures_iter(&src) {
                    let whole = &c[1];
                    let of_area = areas.iter().any(|a| whole.starts_with(&format!("{a}-")));
                    if of_area && !known.contains(whole) {
                        missing.push(format!("{}: {whole}", path.display()));
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "keys without text in pt-BR: {missing:#?}"
    );
}

/// Fixed errors come out in the installation language; free texts pass through as they are.
#[test]
fn errors_follow_the_locale() {
    use crate::error::Error;
    set_current(Locale::En);
    assert_eq!(Error::Locked.to_api().message, "the vault is locked");
    assert_eq!(Error::not_found("conta").message(), "not found: conta");
    assert_eq!(Error::invalid("valor inválido").message(), "valor inválido");
    set_current(Locale::Es);
    assert_eq!(Error::WrongPassword.message(), "contraseña incorrecta");
    set_current(Locale::PtBr);
    assert_eq!(Error::Locked.message(), Error::Locked.to_string());
    assert_eq!(Error::NoVault.message(), Error::NoVault.to_string());
}

/// Each key exists in a single file, per language: a repeated id across two `.ftl` files breaks
/// the catalog load (each area has its own file and prefix).
#[test]
fn ids_are_unique_across_files() {
    for l in [Locale::PtBr, Locale::En, Locale::Es, Locale::PtPt] {
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        let mut dups = Vec::new();
        for (file, src) in sources(l) {
            for id in message_ids(src) {
                if let Some(first) = seen.insert(id.clone(), file) {
                    dups.push(format!("{}: {id} em {first} e {file}", l.code()));
                }
            }
        }
        assert!(dups.is_empty(), "repeated keys: {dups:#?}");
    }
}

/// Money with the currency's decimals: yen without cents, Kuwaiti dinar with three.
#[test]
fn money_uses_the_currency_minor_units() {
    assert_eq!(money(Locale::PtBr, 123456, "BRL"), "R$ 1.234,56");
    assert_eq!(money(Locale::En, 1234, "JPY"), "JPY 1,234");
    assert_eq!(money(Locale::PtBr, 1234567, "KWD"), "KWD 1.234,567");
    assert_eq!(money(Locale::En, -500, "USD"), "\u{2212}$5.00");
}

#[test]
fn catalogs_parse_cleanly() {
    for l in AVAILABLE {
        let mut bundle = FluentBundle::new_concurrent(vec![l.langid()]);
        for (file, src) in sources(l) {
            let res = FluentResource::try_new(src.to_string())
                .unwrap_or_else(|(_, e)| panic!("{}/{file}: {e:?}", l.code()));
            bundle
                .add_resource(res)
                .unwrap_or_else(|e| panic!("{}/{file}: {e:?}", l.code()));
        }
    }
}

#[test]
fn plurals_and_arguments_are_formatted() {
    let n = |v: i64| args(&[("n", v.into())]);
    let key = "alertas-rotina-alertas-novos";
    assert_eq!(tr(Locale::En, key, Some(&n(1))), "1 new alert.");
    assert_eq!(tr(Locale::En, key, Some(&n(3))), "3 new alerts.");
    // Missing argument: the message has an error and falls back to its own key (no fallback).
    assert_eq!(tr(Locale::PtBr, key, None), key);
}
