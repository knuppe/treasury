//! Escritas da IA viram propostas com diff; só o usuário aprova.

use serde_json::{Value, json};

use super::*;

fn setup() -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, acc)
}

fn agent() -> Actor {
    Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    }
}

fn txn_count(api: &Api) -> i64 {
    api.call("transactions.query", json!({})).unwrap()["total"]
        .as_i64()
        .unwrap()
}

#[test]
fn ai_write_becomes_pending_proposal_until_approved() {
    let (_d, api, acc) = setup();
    let r = api
        .call_as(&agent(), "transactions.create", json!({"account_id": acc, "date": "2026-10-01", "amount": -5000, "description": "Padaria"}))
        .unwrap();
    assert_eq!(r["status"], "pending_approval");
    assert_eq!(txn_count(&api), 0, "nada gravado antes da aprovação");
    let list = api
        .call("ai.list_proposals", json!({"status": "pending"}))
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["command"], "transactions.create");
    assert_eq!(list[0]["level"], "reversible");
    let detail = api
        .call("ai.get_proposal", json!({"id": r["proposal_id"]}))
        .unwrap();
    assert!(
        detail["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["table_name"] == "txn" && i["after"]["description"] == "Padaria")
    );
    // A IA não aprova a própria proposta.
    assert!(
        api.call_as(&agent(), "ai.approve", json!({"ids": [r["proposal_id"]]}))
            .is_err()
    );
    let res = api
        .call("ai.approve", json!({"ids": [r["proposal_id"]]}))
        .unwrap();
    assert_eq!(res[0]["status"], "approved");
    assert_eq!(txn_count(&api), 1);
    // Fica no histórico como autoria da IA e com backup antes do lote.
    let hist = api
        .call("changes.list", json!({"author_kind": "ai"}))
        .unwrap();
    assert_eq!(hist[0]["author"]["detail"], "agente");
    let backups = api.call("backup.list", Value::Null).unwrap();
    assert!(
        backups
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["kind"] == "pre_ai")
    );
}

#[test]
fn rejected_proposal_changes_nothing() {
    let (_d, api, acc) = setup();
    let r = api
        .call_as(
            &agent(),
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-01", "amount": -100, "description": "X"}),
        )
        .unwrap();
    api.call(
        "ai.reject",
        json!({"ids": [r["proposal_id"]], "note": "não"}),
    )
    .unwrap();
    assert_eq!(txn_count(&api), 0);
    assert!(
        api.call("ai.approve", json!({"ids": [r["proposal_id"]]}))
            .is_err()
    );
}

#[test]
fn ai_cannot_touch_vault_settings_backups_or_unlisted_writes() {
    let (_d, api, acc) = setup();
    for (cmd, args) in [
        ("settings.set", json!({"auto_lock_minutes": 0})),
        ("backup.create", Value::Null),
        ("market.set_price", json!({})),
        ("market.set_fred_key", json!({"key": "x"})),
        ("export.json", json!({"path": "/tmp/x.json"})),
        ("fx.delete", json!({"id": "x"})),
        ("tax.generate_darf", json!({"month": "2026-01"})),
        ("ai.forget", json!({"id": "x"})),
    ] {
        let e = api.call_as(&agent(), cmd, args).unwrap_err();
        assert!(matches!(e, Error::Forbidden(_)), "{cmd}: {e:?}");
    }
    assert!(api.call_as(&agent(), "vault.lock", Value::Null).is_err());
    // Apagar uma conta agora é possível, mas sempre como proposta sensível.
    let p = api
        .call_as(&agent(), "accounts.delete", json!({"id": acc}))
        .unwrap();
    assert_eq!(p["status"], "pending_approval");
    let prop = api
        .call("ai.get_proposal", json!({"id": p["proposal_id"]}))
        .unwrap();
    assert_eq!(prop["level"], "sensitive");
}

#[test]
fn autonomous_mode_applies_only_reversible_writes() {
    let (_d, api, acc) = setup();
    api.with_session(|s, _| crate::settings::set(s.db.conn(), policy::AUTONOMOUS_SETTING, &true))
        .unwrap();
    let r = api
        .call_as(
            &agent(),
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-01", "amount": -100, "description": "Café"}),
        )
        .unwrap();
    assert_ne!(
        r["status"], "pending_approval",
        "reversível aplica direto no modo autônomo"
    );
    assert_eq!(txn_count(&api), 1);
    // Sensível continua pedindo aprovação.
    let a = api
        .call(
            "assets.create",
            json!({"ticker": "PETR4", "name": "Petrobras", "class": "stock_br"}),
        )
        .unwrap();
    let brk = api
        .call(
            "accounts.create",
            json!({"name": "XP", "kind": "brokerage"}),
        )
        .unwrap();
    let t = api
        .call_as(
            &agent(),
            "portfolio.create_trade",
            json!({"asset_id": a["id"], "account_id": brk["id"], "date": "2026-10-01", "side": "buy", "quantity": "10", "price": "30"}),
        )
        .unwrap();
    assert_eq!(t["status"], "pending_approval");
    let p = api
        .call("ai.list_proposals", json!({"status": "pending"}))
        .unwrap();
    assert_eq!(p[0]["level"], "sensitive");
}

#[test]
fn memory_from_ai_needs_approval_and_user_can_forget() {
    let (_d, api, _acc) = setup();
    let r = api
        .call_as(
            &agent(),
            "ai.remember",
            json!({"kind": "family_rule", "content": "O aluguel vence dia 5"}),
        )
        .unwrap();
    assert_eq!(r["status"], "pending_approval");
    api.call("ai.approve", json!({"ids": [r["proposal_id"]]}))
        .unwrap();
    let mem = api.call("ai.memory", Value::Null).unwrap();
    assert_eq!(mem[0]["content"], "O aluguel vence dia 5");
    assert_eq!(mem[0]["created_by"], "ai");
    api.call("ai.forget", json!({"id": mem[0]["id"]})).unwrap();
    assert!(
        api.call("ai.memory", Value::Null)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn stale_proposal_fails_without_blocking_others() {
    let (_d, api, acc) = setup();
    let t = api.call("transactions.create", json!({"account_id": acc, "date": "2026-10-01", "amount": -100, "description": "Mercado"})).unwrap();
    let p1 = api
        .call_as(
            &agent(),
            "transactions.update",
            json!({"id": t["id"], "description": "Mercado do bairro"}),
        )
        .unwrap();
    let p2 = api
        .call_as(&agent(), "transactions.create", json!({"account_id": acc, "date": "2026-10-02", "amount": -200, "description": "Farmácia"}))
        .unwrap();
    // O usuário mudou o lançamento depois da proposta 1: ela sai da fila sozinha.
    api.call(
        "transactions.update",
        json!({"id": t["id"], "description": "Supermercado"}),
    )
    .unwrap();
    let res = api
        .call(
            "ai.approve",
            json!({"ids": [p1["proposal_id"], p2["proposal_id"]]}),
        )
        .unwrap();
    let st: Vec<&str> = res
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["status"].as_str().unwrap())
        .collect();
    assert_eq!(st, vec!["withdrawn", "approved"]);
    assert_eq!(txn_count(&api), 2);
    let t = api
        .call("transactions.get", json!({"id": t["id"]}))
        .unwrap();
    assert_eq!(t["description"], "Supermercado", "a mudança à mão fica");
}

#[test]
fn manual_change_to_the_same_data_withdraws_the_proposal() {
    let (_d, api, acc) = setup();
    let cat = |name: &str| {
        api.call("categories.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (mercado, padaria) = (cat("Mercado"), cat("Padaria"));
    let new = |d: &str| {
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-01", "amount": -100, "description": d}),
        )
        .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (a, b, c) = (new("Pão"), new("Feira"), new("Leite"));
    let propose = |ids: &[&str], cat: &str| {
        api.call_as(
            &agent(),
            "transactions.set_category",
            json!({"ids": ids, "category_id": cat}),
        )
        .unwrap()["proposal_id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let p1 = propose(&[&a], &padaria);
    let p2 = propose(&[&b], &mercado);
    let p3 = propose(&[&c], &mercado);
    let p4 = propose(&[&c], &padaria);
    let status = |id: &str| {
        api.call("ai.get_proposal", json!({"id": id})).unwrap()["status"]
            .as_str()
            .unwrap()
            .to_string()
    };

    // Outra coluna da mesma linha (descrição): a proposta continua valendo.
    api.call(
        "transactions.update",
        json!({"id": a, "description": "Pão francês"}),
    )
    .unwrap();
    assert_eq!(status(&p1), "pending");
    // A mesma coluna (categoria posta à mão): sai da fila, com o motivo.
    api.call(
        "transactions.set_category",
        json!({"ids": [b], "category_id": padaria}),
    )
    .unwrap();
    assert_eq!(status(&p2), "withdrawn");
    let note = api.call("ai.get_proposal", json!({"id": p2})).unwrap()["note"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(note.contains("você mudou os mesmos dados"), "{note}");
    let pending = api
        .call("ai.list_proposals", json!({"status": "pending"}))
        .unwrap();
    assert_eq!(pending.as_array().unwrap().len(), 3);

    // Duas propostas para a mesma categoria: aprovar uma retira a outra (no mesmo lote).
    let res = api
        .call("ai.approve", json!({"ids": [p3, p4, p1]}))
        .unwrap();
    let st: Vec<&str> = res
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["status"].as_str().unwrap())
        .collect();
    // Aprovadas na ordem de criação: p1, p3 e então p4 (já retirada pela p3).
    assert_eq!(st, vec!["approved", "approved", "withdrawn"]);
    let t = api.call("transactions.get", json!({"id": c})).unwrap();
    assert_eq!(t["category_id"], mercado.as_str());
    let t = api.call("transactions.get", json!({"id": b})).unwrap();
    assert_eq!(
        t["category_id"],
        padaria.as_str(),
        "a escolha do usuário fica"
    );

    // Linha apagada à mão: a proposta sobre ela sai da fila.
    let d = new("Café");
    let p5 = propose(&[&d], &padaria);
    api.call("transactions.delete", json!({"ids": [d]}))
        .unwrap();
    assert_eq!(status(&p5), "withdrawn");
    // Rejeitar uma retirada não dá erro.
    let r = api.call("ai.reject", json!({"ids": [p5]})).unwrap();
    assert_eq!(r[0]["status"], "withdrawn");
}

#[test]
fn agent_withdraws_only_its_own_pending_proposal() {
    let (_d, api, acc) = setup();
    let other = Actor::Mcp {
        client: "outro".into(),
        read_only: false,
    };
    let make = |actor: &Actor| {
        api.call_as(
            actor,
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-01", "amount": -100, "description": "Teste"}),
        )
        .unwrap()["proposal_id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let mine = make(&agent());
    let theirs = make(&other);
    // De outro cliente, não; da própria, sai da fila sem aplicar nada.
    assert!(
        api.call_as(&agent(), "ai.withdraw_proposal", json!({"id": theirs}))
            .is_err()
    );
    let r = api
        .call_as(
            &agent(),
            "ai.withdraw_proposal",
            json!({"id": mine, "reason": "teste de campo"}),
        )
        .unwrap();
    assert_eq!(r["status"], "withdrawn");
    let p = api.call("ai.get_proposal", json!({"id": mine})).unwrap();
    assert_eq!(p["status"], "withdrawn");
    assert!(p["note"].as_str().unwrap().contains("teste de campo"));
    assert_eq!(txn_count(&api), 0);
    // Já decidida, não muda; só leitura, nem tenta.
    assert!(
        api.call_as(&agent(), "ai.withdraw_proposal", json!({"id": mine}))
            .is_err()
    );
    let reader = Actor::Mcp {
        client: "agente".into(),
        read_only: true,
    };
    assert!(
        api.call_as(&reader, "ai.withdraw_proposal", json!({"id": theirs}))
            .is_err()
    );
    // A interface rejeita pela fila.
    assert!(
        api.call("ai.withdraw_proposal", json!({"id": theirs}))
            .is_err()
    );
}

#[test]
fn retirement_plan_only_on_retirement_accounts_and_known_plans() {
    let (_d, api, _) = setup();
    for args in [
        json!({"name": "x", "kind": "checking", "retirement_plan": "us_401k"}),
        json!({"name": "x", "kind": "retirement", "retirement_plan": "qualquer"}),
        json!({"name": "x", "kind": "retirement", "retirement_plan": "us_hsa", "retirement_coverage": "casal"}),
    ] {
        assert!(api.call("accounts.create", args.clone()).is_err(), "{args}");
    }
}
