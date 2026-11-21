//! Secrets and data outside the vault (docs/local-security.md, item 13): runs the flows for
//! password, PIN, recovery kit, Shamir shares, backup and password change with made-up data
//! and looks for the password, the PIN, the hex key, the code, the shares, a description and
//! a value in the logs, the app's files, recent temporary files, error messages and
//! the change history.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use serde_json::{Value, json};

use super::*;

/// Joins the text of every `tracing` event and span of this thread.
struct Capture(Arc<Mutex<String>>);

struct Fields<'a>(&'a mut String);

impl tracing::field::Visit for Fields<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let _ = write!(self.0, " {}={:?}", field.name(), value);
    }
}

impl tracing::Subscriber for Capture {
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        span.record(&mut Fields(&mut self.0.lock().unwrap()));
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        values.record(&mut Fields(&mut self.0.lock().unwrap()));
    }
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        let mut out = self.0.lock().unwrap();
        out.push('\n');
        event.record(&mut Fields(&mut out));
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

/// Contents of the files under `dir` (up to `depth` levels) changed since `since`.
fn files_text(dir: &Path, depth: usize, since: SystemTime, out: &mut Vec<(String, Vec<u8>)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        let Ok(meta) = e.metadata() else { continue };
        if meta.is_dir() {
            if depth > 0 {
                files_text(&path, depth - 1, since, out);
            }
        } else if meta.len() < 8 * 1024 * 1024
            && meta.modified().is_ok_and(|m| m >= since)
            && let Ok(bytes) = std::fs::read(&path)
        {
            out.push((path.display().to_string(), bytes));
        }
    }
}

fn contains(hay: &[u8], needle: &str) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|w| w == needle.as_bytes())
}

#[test]
fn secrets_and_values_stay_out_of_logs_files_and_errors() {
    let start = SystemTime::now() - std::time::Duration::from_secs(2);
    // Unique values per run: nothing from another test or file matches by chance.
    let tag: String = crate::ids::new_id()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(12)
        .collect();
    let password = format!("senha-vazamento-{tag}");
    let new_password = format!("outra-senha-vazamento-{tag}");
    let wrong = format!("errada-vazamento-{tag}");
    let pin = "730518";
    let description = format!("Padaria Inventada {tag}");
    let amount: i64 = -987_654_321;

    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    let logs = Arc::new(Mutex::new(String::new()));
    let mut errors = Vec::new();
    let mut secrets: Vec<String> = vec![password.clone(), new_password.clone(), pin.into()];

    tracing::subscriber::with_default(Capture(logs.clone()), || {
        tracing::info!("captura ligada");
        api.call(
            "vault.create",
            json!({"password": password, "household_name": "Casa Inventada", "members": ["Pessoa"]}),
        )
        .unwrap();
        let key = api
            .with_session(|s, _| Ok(hex::encode(s.db.key().key_bytes())))
            .unwrap();
        secrets.push(key.to_uppercase());
        secrets.push(key);

        let acc = api
            .call(
                "accounts.create",
                json!({"name": "Conta", "kind": "checking"}),
            )
            .unwrap();
        api.call(
            "transactions.create",
            json!({"account_id": acc["id"], "date": "2026-03-02", "amount": amount, "description": description}),
        )
        .unwrap();

        api.call(
            "vault.pin_enable",
            json!({"password": password, "pin": pin}),
        )
        .unwrap();
        let kit = api
            .call("vault.recovery_kit", json!({"password": password}))
            .unwrap();
        secrets.push(kit["code"].as_str().unwrap().to_string());
        let shares = api
            .call(
                "vault.recovery_shares_create",
                json!({"password": password}),
            )
            .unwrap();
        for s in shares["shares"].as_array().unwrap() {
            secrets.push(s.as_str().unwrap().to_string());
        }
        api.call("backup.create", Value::Null).unwrap();
        api.call(
            "vault.change_password",
            json!({"current": password, "new": new_password}),
        )
        .unwrap();

        // Errors with the wrong secret and with bad arguments do not repeat what came in.
        for (cmd, args) in [
            ("vault.recovery_kit", json!({"password": wrong})),
            ("vault.change_password", json!({"current": wrong, "new": 1})),
            ("vault.pin_enable", json!({"password": wrong, "pin": wrong})),
        ] {
            errors.push(api.call(cmd, args).unwrap_err().to_string());
        }
        api.call("vault.lock", Value::Null).unwrap();
        for (cmd, args) in [
            ("vault.unlock", json!({"password": wrong})),
            ("vault.unlock_pin", json!({"pin": "000000"})),
        ] {
            errors.push(api.call(cmd, args).unwrap_err().to_string());
        }
        api.call("vault.unlock_pin", json!({"pin": pin})).unwrap();
        errors.push(
            serde_json::to_string(&api.call("changes.list", json!({"limit": 200})).unwrap())
                .unwrap(),
        );
        api.call("vault.lock", Value::Null).unwrap();
    });
    secrets.push(wrong.clone());

    let logs = logs.lock().unwrap().clone();
    assert!(logs.contains("captura ligada"), "log capture did not work");
    let mut files = Vec::new();
    files_text(dir.path(), 8, start, &mut files);
    assert!(
        files.iter().any(|(p, _)| p.ends_with(".db")),
        "the vault was not read"
    );
    files_text(&std::env::temp_dir(), 2, start, &mut files);

    let plain = [
        description.clone(),
        (-amount).to_string(),
        "9876543.21".into(),
    ];
    for needle in secrets.iter().chain(plain.iter()) {
        assert!(!logs.contains(needle.as_str()), "{needle} in the logs");
        for (path, bytes) in &files {
            assert!(!contains(bytes, needle), "{needle} in {path}");
        }
    }
    for needle in &secrets {
        for e in &errors {
            assert!(!e.contains(needle.as_str()), "{needle} in {e}");
        }
    }
}
