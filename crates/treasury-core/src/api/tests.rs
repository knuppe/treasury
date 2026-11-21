use serde_json::{Value, json};

use super::*;
use crate::api::system::VaultState;

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    (dir, api)
}

fn create(api: &Api) -> Value {
    api.call("vault.create", json!({"password": "senha-mestra-forte", "household_name": "Família Exemplo", "members": ["Carlos", "Ana"]}))
        .unwrap()
}

#[test]
fn vault_lifecycle() {
    let (_d, api) = api();
    let st: VaultStatus =
        serde_json::from_value(api.call("vault.status", Value::Null).unwrap()).unwrap();
    assert_eq!(st.state, VaultState::NoVault);
    assert!(matches!(
        api.call("household.get", Value::Null),
        Err(Error::NoVault)
    ));

    assert!(
        api.call(
            "vault.create",
            json!({"password": "curta", "household_name": "x"})
        )
        .is_err()
    );
    let st: VaultStatus = serde_json::from_value(create(&api)).unwrap();
    assert_eq!(st.state, VaultState::Unlocked);
    assert_eq!(st.household_name.as_deref(), Some("Família Exemplo"));
    assert!(matches!(
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "x"})
        ),
        Err(Error::VaultExists(_))
    ));

    let h = api.call("household.get", Value::Null).unwrap();
    assert_eq!(h["base_currency"], "BRL");

    let st: VaultStatus =
        serde_json::from_value(api.call("vault.lock", Value::Null).unwrap()).unwrap();
    assert_eq!(st.state, VaultState::Locked);
    assert!(matches!(
        api.call("household.get", Value::Null),
        Err(Error::Locked)
    ));
    assert!(matches!(
        api.call("vault.unlock", json!({"password": "errada-errada"})),
        Err(Error::WrongPassword)
    ));
    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["name"],
        "Família Exemplo"
    );
}

#[test]
fn single_instance_lock() {
    let (d, _api) = api();
    let paths = TreasuryPaths::with_root(d.path().join(".treasury")).unwrap();
    assert!(matches!(Api::new(paths), Err(Error::AlreadyOpen)));
}

#[test]
fn creation_is_recorded_and_undoable_history_exists() {
    let (_d, api) = api();
    create(&api);
    let list: Vec<changes::ChangeSetSummary> =
        serde_json::from_value(api.call("changes.list", json!({})).unwrap()).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].reason, "Criação do cofre");
    assert!(list[0].summary["insert"]["category"].as_i64().unwrap() > 50);
    let detail = api.call("changes.get", json!({"id": list[0].id})).unwrap();
    assert!(detail["items"].as_array().unwrap().len() > 50);
}

#[test]
fn password_change_recovery_kit_and_recovery_unlock() {
    let (_d, api) = api();
    create(&api);
    assert!(matches!(
        api.call("vault.recovery_kit", json!({"password": "nao-e-esta"})),
        Err(Error::WrongPassword)
    ));
    let kit = api
        .call(
            "vault.recovery_kit",
            json!({"password": "senha-mestra-forte"}),
        )
        .unwrap();
    let code = kit["code"].as_str().unwrap().to_string();

    api.call(
        "vault.change_password",
        json!({"current": "senha-mestra-forte", "new": "outra-senha-forte"}),
    )
    .unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    assert!(matches!(
        api.call("vault.unlock", json!({"password": "senha-mestra-forte"})),
        Err(Error::WrongPassword)
    ));
    api.call("vault.unlock", json!({"password": "outra-senha-forte"}))
        .unwrap();
    api.call("vault.lock", Value::Null).unwrap();

    // O kit antigo não serve mais depois da troca de senha (a chave mudou).
    assert!(
        api.call(
            "vault.unlock_recovery",
            json!({"code": code, "new_password": "terceira-senha-x"})
        )
        .is_err()
    );
    api.call("vault.unlock", json!({"password": "outra-senha-forte"}))
        .unwrap();
    let kit2 = api
        .call(
            "vault.recovery_kit",
            json!({"password": "outra-senha-forte"}),
        )
        .unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    api.call(
        "vault.unlock_recovery",
        json!({"code": kit2["code"], "new_password": "terceira-senha-x"}),
    )
    .unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({"password": "terceira-senha-x"}))
        .unwrap();
}

#[test]
fn backups_via_api_and_restore() {
    let (_d, api) = api();
    create(&api);
    let b = api.call("backup.create", Value::Null).unwrap();
    assert_eq!(b["verified"], true);
    let health = api.call("backup.health", Value::Null).unwrap();
    assert_eq!(health["warning"], false);

    // Alteração depois do backup.
    api.with_session(|s, _| {
        s.db.conn()
            .execute("UPDATE household SET name = 'Depois do backup'", [])?;
        Ok(())
    })
    .unwrap();
    assert!(
        api.call(
            "backup.restore",
            json!({"id": b["id"], "master_password": "errada-errada"})
        )
        .is_err()
    );
    let st = api
        .call(
            "backup.restore",
            json!({"id": b["id"], "master_password": "senha-mestra-forte"}),
        )
        .unwrap();
    assert_eq!(st["state"], "unlocked");
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["name"],
        "Família Exemplo"
    );
    // Existe um backup pre_restore com o estado anterior.
    let list = api.call("backup.list", Value::Null).unwrap();
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .any(|b| b["kind"] == "pre_restore")
    );
    // E a senha mestra continua a mesma.
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
}

#[test]
fn lock_makes_on_close_backup_only_with_changes() {
    let (_d, api) = api();
    create(&api);
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
    let list = api.call("backup.list", Value::Null).unwrap();
    let on_close = list
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["kind"] == "on_close")
        .count();
    // Criar o cofre gera um change_set antes do bloqueio? O backup de sessão marca o início,
    // então só há backup ao fechar se houver alterações depois do desbloqueio.
    assert!(on_close <= 1);
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
    let list2 = api.call("backup.list", Value::Null).unwrap();
    let on_close2 = list2
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["kind"] == "on_close")
        .count();
    assert_eq!(on_close, on_close2, "sem alterações, sem backup ao fechar");
}

#[test]
fn settings_roundtrip_and_validation() {
    let (_d, api) = api();
    create(&api);
    let s = api.call("settings.get", Value::Null).unwrap();
    assert_eq!(s["auto_lock_minutes"], 15);
    assert_eq!(s["budget_basis"], "accrual");
    let s = api
        .call(
            "settings.set",
            json!({"auto_lock_minutes": 5, "budget_basis": "cash"}),
        )
        .unwrap();
    assert_eq!(s["budget_basis"], "cash");
    // Pasta Downloads: desligada por padrão, liga pela interface.
    assert_eq!(s["watch_downloads"], false);
    let s = api
        .call("settings.set", json!({"watch_downloads": true}))
        .unwrap();
    assert_eq!(s["watch_downloads"], true);
    assert_eq!(
        api.call("settings.get", Value::Null).unwrap()["watch_downloads"],
        true
    );
    // Apagar de Downloads depois de importar: desligado por padrão.
    assert_eq!(s["downloads_delete_after_import"], false);
    let s = api
        .call(
            "settings.set",
            json!({"downloads_delete_after_import": true}),
        )
        .unwrap();
    assert_eq!(s["downloads_delete_after_import"], true);
    assert!(api.call("settings.set", json!({"nao_existe": 1})).is_err());
    assert!(
        api.call("settings.set", json!({"auto_lock_minutes": 99999}))
            .is_err()
    );
}

#[test]
fn mcp_read_only_actor_cannot_write_or_manage_vault() {
    let (_d, api) = api();
    create(&api);
    let mcp = Actor::Mcp {
        client: "claude-desktop".into(),
        read_only: true,
    };
    assert!(api.call_as(&mcp, "household.get", Value::Null).is_ok());
    assert!(matches!(
        api.call_as(&mcp, "fx.set", json!({})),
        Err(Error::Forbidden(_))
    ));
    assert!(matches!(
        api.call_as(&mcp, "vault.lock", Value::Null),
        Err(Error::Forbidden(_))
    ));
    assert!(matches!(
        api.call_as(&mcp, "backup.create", Value::Null),
        Err(Error::Forbidden(_))
    ));
    let writer = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(matches!(
        api.call_as(&writer, "vault.lock", Value::Null),
        Err(Error::Forbidden(_))
    ));
}

#[test]
fn tick_auto_locks_after_inactivity() {
    let (_d, api) = api();
    create(&api);
    api.set_auto_lock_after(std::time::Duration::from_millis(1));
    std::thread::sleep(std::time::Duration::from_millis(5));
    let out = api.tick().unwrap();
    assert!(out.locked);
    assert!(!api.is_unlocked());
}

#[test]
fn unknown_commands() {
    let (_d, api) = api();
    create(&api);
    assert!(matches!(
        api.call("nada", Value::Null),
        Err(Error::UnknownCommand(_))
    ));
    assert!(matches!(
        api.call("nada.x", Value::Null),
        Err(Error::UnknownCommand(_))
    ));
    assert!(matches!(
        api.call("vault.x", Value::Null),
        Err(Error::UnknownCommand(_))
    ));
}

#[test]
fn asset_patch_edits_identity_and_guards_currency() {
    let (_d, api) = api();
    create(&api);
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Corretora", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let a = api
        .call(
            "assets.create",
            json!({"ticker": "IVVB11", "name": "iShares S&P 500", "class": "etf_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "assets.update",
        json!({"id": a, "isin": "br-ivvbci-001", "cnpj": "19.909.560/0001-91", "country": "br", "currency": "usd"}),
    )
    .unwrap();
    let got = api.call("assets.get", json!({"id": a})).unwrap();
    assert_eq!(got["isin"], "BR-IVVBCI-001");
    assert_eq!(got["country"], "BR");
    assert_eq!(got["currency"], "USD");
    api.call(
        "assets.update",
        json!({"id": a, "isin": null, "currency": "BRL"}),
    )
    .unwrap();
    api.call(
        "portfolio.create_trade",
        json!({"asset_id": a, "account_id": acc, "date": "2026-02-02", "side": "buy", "quantity": "1", "price": "300", "cash_movement": false}),
    )
    .unwrap();
    let err = api
        .call("assets.update", json!({"id": a, "currency": "USD"}))
        .unwrap_err();
    assert_eq!(err.code(), "conflict");
    let got = api.call("assets.get", json!({"id": a})).unwrap();
    assert!(got["isin"].is_null());
    assert_eq!(got["currency"], "BRL");
}

#[test]
fn recovery_shares_unlock_after_password_change_and_require_new_password() {
    let (_d, api) = api();
    create(&api);
    let rs = api
        .call(
            "vault.recovery_shares_create",
            json!({"password": "senha-mestra-forte"}),
        )
        .unwrap();
    let shares: Vec<String> = serde_json::from_value(rs["shares"].clone()).unwrap();
    assert_eq!(shares.len(), 3);
    assert!(
        api.call(
            "vault.recovery_shares_create",
            json!({"password": "errada"})
        )
        .is_err()
    );
    api.call(
        "vault.change_password",
        json!({"current": "senha-mestra-forte", "new": "outra-senha-bem-forte"}),
    )
    .unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    let st = api
        .call(
            "vault.unlock_shares",
            json!({"shares": [shares[1], shares[2]], "new_password": "senha-recuperada-forte"}),
        )
        .unwrap();
    assert_eq!(st["state"], "unlocked");
    let rec = api.call("vault.recovery_status", json!({})).unwrap();
    assert!(rec["shares_created_at"].is_string());
    api.call("vault.lock", Value::Null).unwrap();
    // A senha nova é a da recuperação.
    api.call(
        "vault.unlock",
        json!({"password": "senha-recuperada-forte"}),
    )
    .unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    // Uma parte só não basta.
    let e = api
        .call(
            "vault.unlock_shares",
            json!({"shares": [shares[0]], "new_password": "qualquer-senha-forte"}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
}

#[test]
fn fido2_slot_recovers_with_the_right_hmac_secret_only() {
    let (_d, api) = api();
    create(&api);
    let secret = "ab".repeat(32);
    let slot = api
        .call(
            "vault.fido2_add",
            json!({"label": "YubiKey 5C", "rp_id": "treasury.local", "credential_id": "Y3JlZA", "salt": "00".repeat(32), "hmac_secret": secret}),
        )
        .unwrap();
    let slots = api.call("vault.fido2_slots", json!({})).unwrap();
    assert_eq!(slots[0]["label"], "YubiKey 5C");
    assert!(
        slots[0].get("wrap").is_none(),
        "slot público não leva a chave cifrada"
    );
    api.call("vault.lock", Value::Null).unwrap();
    let bad = api
        .call(
            "vault.unlock_fido2",
            json!({"slot_id": slot["id"], "hmac_secret": "cd".repeat(32), "new_password": "senha-nova-bem-forte"}),
        )
        .unwrap_err();
    assert_eq!(bad.code(), "wrong_recovery_code");
    let ok = api
        .call(
            "vault.unlock_fido2",
            json!({"slot_id": slot["id"], "hmac_secret": secret, "new_password": "senha-nova-bem-forte"}),
        )
        .unwrap();
    assert_eq!(ok["state"], "unlocked");
    api.call(
        "vault.fido2_remove",
        json!({"id": slot["id"], "password": "senha-nova-bem-forte"}),
    )
    .unwrap();
    assert_eq!(api.call("vault.fido2_slots", json!({})).unwrap(), json!([]));
}

#[test]
fn pin_unlock_counts_attempts_and_stays_after_password_change() {
    let (_d, api) = api();
    create(&api);
    let st = api
        .call(
            "vault.pin_enable",
            json!({"password": "senha-mestra-forte", "pin": "730418"}),
        )
        .unwrap();
    assert_eq!(st["pin_enabled"], true);
    api.call(
        "vault.change_password",
        json!({"current": "senha-mestra-forte", "new": "outra-senha-bem-forte"}),
    )
    .unwrap();
    api.call("vault.lock", Value::Null).unwrap();
    let e = api
        .call("vault.unlock_pin", json!({"pin": "111222"}))
        .unwrap_err();
    assert_eq!(e.code(), "wrong_pin");
    assert!(e.to_string().contains("Restam 4"), "{e}");
    let ok = api
        .call("vault.unlock_pin", json!({"pin": "730418"}))
        .unwrap();
    assert_eq!(ok["state"], "unlocked");
    api.call("vault.pin_disable", json!({})).unwrap();
    assert_eq!(
        api.call("vault.status", json!({})).unwrap()["pin_enabled"],
        false
    );
}

#[test]
fn reference_country_via_api() {
    let (_d, api) = api();
    create(&api);
    let cat = api.call("indices.catalog", Value::Null).unwrap();
    assert_eq!(cat["country"], "BR");
    assert_eq!(cat["automatic"], true);
    let cat = api
        .call("indices.set_country", json!({"country": "CA"}))
        .unwrap();
    assert_eq!(cat["country"], "CA");
    assert_eq!(cat["automatic"], false);
    let f = api
        .call(
            "indices.factor",
            json!({"deflator": "country", "from": "2026-01-01", "to": "2026-02-01"}),
        )
        .unwrap();
    assert_eq!(f["series_id"], "cpi_ca");
    assert!(f["factor"].is_null(), "sem dados ainda");
    let series = api.call("indices.list_series", Value::Null).unwrap();
    let corra = series
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "corra")
        .unwrap();
    assert_eq!(corra["in_use"], true);
    assert_eq!(corra["provider"], "boc");
    let providers = api.call("indices.providers", Value::Null).unwrap();
    assert!(
        providers
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["info"]["id"] == "boc" && p["status"] == "never")
    );
    let writer = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(matches!(
        api.call_as(&writer, "indices.set_country", json!({"country": "PT"})),
        Err(Error::Forbidden(_))
    ));
    assert!(api.call_as(&writer, "indices.catalog", Value::Null).is_ok());
    let back = api
        .call("indices.set_country", json!({"country": null}))
        .unwrap();
    assert_eq!(back["country"], "BR");
    assert!(
        api.call("indices.set_country", json!({"country": "ZZ"}))
            .is_err()
    );
}

/// Dormir, bloquear a tela ou trocar de usuário bloqueia o cofre (ajuste ligado por padrão).
#[test]
fn auto_update_downloads_by_default_and_lives_outside_the_vault() {
    let (_d, api) = api();
    // Sem cofre: o ajuste já vale (a checagem roda com o cofre fechado).
    let s = api.call("system.update_settings", Value::Null).unwrap();
    assert_eq!(s["mode"], "download");
    for mode in ["notify", "off", "download"] {
        let s = api
            .call("system.set_update_settings", json!({ "mode": mode }))
            .unwrap();
        assert_eq!(s["mode"], mode);
        let toml = std::fs::read_to_string(&api.paths().config_path).unwrap();
        assert!(
            toml.contains(&format!("update_mode = \"{mode}\"")),
            "{toml}"
        );
        assert_eq!(
            api.call("system.update_settings", Value::Null).unwrap()["mode"],
            mode
        );
    }
    assert!(
        api.call("system.set_update_settings", json!({ "mode": "sempre" }))
            .is_err()
    );
}

/// O ajuste antigo (`auto_update = false`) vira "desligado"; gravar o novo tira o antigo.
#[test]
fn the_old_auto_update_switch_still_counts() {
    let (_d, api) = api();
    let path = api.paths().config_path.clone();
    let mut c = crate::config::Config::load(&path).unwrap();
    c.auto_update = Some(false);
    c.save(&path).unwrap();
    assert_eq!(
        api.call("system.update_settings", Value::Null).unwrap()["mode"],
        "off"
    );
    api.call("system.set_update_settings", json!({ "mode": "notify" }))
        .unwrap();
    let toml = std::fs::read_to_string(&path).unwrap();
    assert!(!toml.contains("auto_update"), "{toml}");
}

#[test]
fn screen_capture_is_blocked_by_default_and_can_be_allowed() {
    let (_d, api) = api();
    create(&api);
    let s = api.call("settings.get", Value::Null).unwrap();
    assert_eq!(s["allow_screen_capture"], false);
    let s = api
        .call("settings.set", json!({"allow_screen_capture": true}))
        .unwrap();
    assert_eq!(s["allow_screen_capture"], true);
    let s = api.call("settings.get", Value::Null).unwrap();
    assert_eq!(s["allow_screen_capture"], true);
}

#[test]
fn lock_on_sleep_follows_the_setting() {
    let (_d, api) = api();
    create(&api);
    let s = api.call("settings.get", Value::Null).unwrap();
    assert_eq!(s["lock_on_sleep"], true);
    assert!(api.lock_on_sleep().unwrap());
    let st = api.call("vault.status", Value::Null).unwrap();
    assert_eq!(st["state"], "locked");
    // Já bloqueado: nada a fazer.
    assert!(!api.lock_on_sleep().unwrap());
    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
    api.call("settings.set", json!({"lock_on_sleep": false}))
        .unwrap();
    assert!(!api.lock_on_sleep().unwrap());
    let st = api.call("vault.status", Value::Null).unwrap();
    assert_eq!(st["state"], "unlocked");
}

#[test]
fn an_interrupted_creation_leaves_nothing_in_the_vault_place() {
    let (_d, api) = api();
    let db = api.paths().db_path.clone();
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    // Sobra de uma criação interrompida (app fechado no meio): fica de lado e não impede criar.
    let creating = std::path::PathBuf::from(format!("{}.criando", db.display()));
    std::fs::write(&creating, b"metade de um banco").unwrap();
    assert!(!db.exists());
    let st: VaultStatus = serde_json::from_value(
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "Família Teste", "members": ["Ana"]}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(st.state, VaultState::Unlocked);
    assert!(db.exists() && !creating.exists());
    // O cofre movido abre de novo com a senha.
    api.call("vault.lock", Value::Null).unwrap();
    api.call("vault.unlock", json!({"password": "senha-mestra-forte"}))
        .unwrap();
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["name"],
        "Família Teste"
    );
}
