//! Recovery from a backup with the vault closed (docs/backup-recovery.md): the damaged vault
//! is reported, set aside (never deleted) and the backup takes its place; copy out and
//! backup brought in from outside.

use serde_json::{Value, json};

use super::*;

const PASSWORD: &str = "senha-de-teste-forte";

fn api_with_backup() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": PASSWORD, "household_name": "Casa"}),
    )
    .unwrap();
    api.call("backup.create", Value::Null).unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    (dir, api)
}

/// Corrupts the vault file (the salt stays, the pages do not).
fn damage(api: &Api) {
    let path = &api.paths().db_path;
    let mut bytes = std::fs::read(path).unwrap();
    bytes.truncate(1024);
    std::fs::write(path, bytes).unwrap();
}

fn code(r: Result<Value>) -> String {
    r.unwrap_err().code().to_string()
}

#[test]
fn damaged_vault_is_set_aside_and_restored_from_a_backup() {
    let (_d, api) = api_with_backup();
    // Wrong password is still a wrong password.
    assert_eq!(
        code(api.call("vault.unlock", json!({"password": "outra-senha-qualquer"}))),
        "wrong_password"
    );
    damage(&api);
    // The right password opens the newest backup, but not the vault: damaged-vault notice.
    assert_eq!(
        code(api.call("vault.unlock", json!({"password": PASSWORD}))),
        "vault_damaged"
    );
    let files = api.call("backup.scan", Value::Null).unwrap();
    let files = files.as_array().unwrap();
    assert_eq!(files.len(), 1, "{files:?}");
    assert_eq!(files[0]["kind"], "manual");
    let r = api
        .call(
            "backup.restore_file",
            json!({"path": files[0]["path"], "master_password": PASSWORD}),
        )
        .unwrap();
    assert_eq!(r["status"]["state"], "unlocked");
    assert_eq!(r["status"]["household_name"], "Casa");
    // The damaged vault stayed beside it, with the suffix.
    let moved = r["moved_to"].as_str().unwrap();
    assert!(moved.contains(".danificado-"), "{moved}");
    assert!(std::path::Path::new(moved).exists());
    // And the restored vault opens again after locking.
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
}

#[test]
fn restore_needs_the_backup_password_and_a_file_from_the_folder() {
    let (d, api) = api_with_backup();
    damage(&api);
    let files = api.call("backup.scan", Value::Null).unwrap();
    let path = files[0]["path"].clone();
    // Password that does not open the backup: the vault stays where it was.
    assert_eq!(
        code(api.call(
            "backup.restore_file",
            json!({"path": path, "master_password": "senha-nova-qualquer"})
        )),
        "wrong_password"
    );
    assert!(api.paths().db_path.exists());
    // File outside the backups folder: refused.
    let outside = d.path().join("fora.db.enc");
    std::fs::copy(path.as_str().unwrap(), &outside).unwrap();
    assert_eq!(
        code(api.call(
            "backup.restore_file",
            json!({"path": outside.to_string_lossy(), "master_password": PASSWORD})
        )),
        "invalid"
    );
}

#[test]
fn backup_goes_out_and_comes_back() {
    let (d, api) = api_with_backup();
    let files = api.call("backup.scan", Value::Null).unwrap();
    let src = files[0]["path"].as_str().unwrap().to_string();
    // Copy out (USB stick): same file, encrypted.
    let usb = d.path().join("pendrive");
    std::fs::create_dir_all(&usb).unwrap();
    let dest = usb.join("treasury-copia.db.enc");
    api.call(
        "backup.export_file",
        json!({"path": src, "dest": dest.to_string_lossy()}),
    )
    .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), std::fs::read(&src).unwrap());
    // Any random file is not accepted as a backup.
    let junk = d.path().join("nota.txt");
    std::fs::write(&junk, "não é backup").unwrap();
    assert_eq!(
        code(api.call(
            "backup.import_file",
            json!({"source": junk.to_string_lossy()})
        )),
        "invalid"
    );
    // The copy comes back as imported and restores the damaged vault.
    let imported = api
        .call(
            "backup.import_file",
            json!({"source": dest.to_string_lossy()}),
        )
        .unwrap();
    assert_eq!(imported["imported"], true);
    let files = api.call("backup.scan", Value::Null).unwrap();
    assert_eq!(files.as_array().unwrap().len(), 2);
    damage(&api);
    let r = api
        .call(
            "backup.restore_file",
            json!({"path": imported["path"], "master_password": PASSWORD}),
        )
        .unwrap();
    assert_eq!(r["status"]["state"], "unlocked");
}

#[test]
fn restore_from_a_file_with_the_vault_open_keeps_a_pre_restore_backup() {
    let (_d, api) = api_with_backup();
    api.call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
    let files = api.call("backup.scan", Value::Null).unwrap();
    let r = api
        .call(
            "backup.restore_file",
            json!({"path": files[0]["path"], "master_password": PASSWORD}),
        )
        .unwrap();
    assert_eq!(r["status"]["state"], "unlocked");
    assert!(r["moved_to"].is_null());
    let after = api.call("backup.scan", Value::Null).unwrap();
    assert!(
        after
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["kind"] == "pre_restore"),
        "{after}"
    );
}
