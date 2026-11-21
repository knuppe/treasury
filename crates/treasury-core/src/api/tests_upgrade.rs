//! Upgrading through the app: a vault left by an old version opens, gets a backup verified
//! before migrating, records which version it came from and stays usable; a backup from
//! a newer version is rejected without touching the current vault.

use serde_json::{Value, json};

use super::*;

const PASSWORD: &str = "senha-de-teste-forte";

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    (dir, api)
}

fn versions(api: &Api) -> crate::vault_versions::VaultVersions {
    api.with_session(|s, _| Ok(crate::vault_versions::get(s.db.conn())?.unwrap()))
        .unwrap()
}

fn pre_migration_backups(api: &Api) -> Vec<Value> {
    api.call("backup.list", Value::Null)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["kind"] == "pre_migration")
        .cloned()
        .collect()
}

/// A 0.9.38 vault (the last one with the old migration chain) opens through the app: backup
/// verified first, history replaced by V1, record of where it came from and everyday reads.
#[test]
fn a_0_9_38_vault_opens_backed_up_and_records_where_it_came_from() {
    let (_d, api) = api();
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/vault-v0.9.38.db"
        ),
        &api.paths().db_path,
    )
    .unwrap();
    let password = "cofre-da-versao-0.9.38";
    let st = api
        .call("vault.unlock", json!({ "password": password }))
        .unwrap();
    assert_eq!(st["state"], "unlocked");
    assert!(st["newer_app_version"].is_null());
    let backups = pre_migration_backups(&api);
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0]["verified"], true);
    let v = versions(&api);
    assert_eq!(v.history.len(), 1);
    assert_eq!((v.history[0].schema_from, v.history[0].schema_to), (164, 1));
    assert_eq!(v.last_app_version, crate::config::app_version());
    for cmd in [
        "household.get",
        "accounts.list",
        "transactions.query",
        "categories.list",
        "backup.health",
    ] {
        api.call(cmd, json!({}))
            .unwrap_or_else(|e| panic!("{cmd}: {e}"));
    }
    // The earlier backup (old history) is still restorable.
    let b = &backups[0];
    api.call(
        "backup.restore",
        json!({"id": b["id"], "master_password": password}),
    )
    .unwrap();
    // Opening again neither migrates nor makes another backup.
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({ "password": password }))
        .unwrap();
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["name"],
        "Família 0.9.38"
    );
}

#[test]
fn an_older_app_opening_the_vault_is_flagged() {
    let (_d, api) = api();
    api.call(
        "vault.create",
        json!({"password": PASSWORD, "household_name": "Família Teste"}),
    )
    .unwrap();
    let st = api.call("vault.status", Value::Null).unwrap();
    assert!(st["newer_app_version"].is_null());
    let v = versions(&api);
    assert_eq!(v.first_app_version, crate::config::app_version());
    // A newer version opened the vault before (same schema): this one warns.
    api.with_session(|s, _| {
        crate::vault_versions::record(
            s.db.conn(),
            "99.0.0",
            v.history[0].schema_to,
            v.history[0].schema_to,
        )?;
        Ok(())
    })
    .unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    let st = api
        .call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
    assert_eq!(st["newer_app_version"], "99.0.0");
    let v = versions(&api);
    assert_eq!(v.newest_app_version, "99.0.0");
    assert_eq!(v.last_app_version, crate::config::app_version());
    // Going back to this version enters the history.
    assert_eq!(
        v.history
            .iter()
            .map(|h| h.app_version.as_str())
            .collect::<Vec<_>>(),
        [
            crate::config::app_version(),
            "99.0.0",
            crate::config::app_version()
        ]
    );
}

#[test]
fn a_backup_from_a_newer_version_is_refused_without_touching_the_vault() {
    let (_d, api) = api();
    api.call(
        "vault.create",
        json!({"password": PASSWORD, "household_name": "Família Teste"}),
    )
    .unwrap();
    // The backup comes with a migration this app does not know (made by a newer version).
    let insert = "INSERT INTO refinery_schema_history (version, name, applied_on, checksum)
                  VALUES (9999, 'futuro', '2030-01-01', '0')";
    api.with_session(|s, _| Ok(s.db.conn().execute(insert, [])?))
        .unwrap();
    let b = api.call("backup.create", Value::Null).unwrap();
    api.with_session(|s, _| {
        Ok(s.db.conn().execute(
            "DELETE FROM refinery_schema_history WHERE version = 9999",
            [],
        )?)
    })
    .unwrap();
    let e = api
        .call(
            "backup.restore",
            json!({"id": b["id"], "master_password": PASSWORD}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "schema_too_new", "{e}");
    // The vault stays open, unchanged, and opens again after locking.
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["name"],
        "Família Teste"
    );
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
}
