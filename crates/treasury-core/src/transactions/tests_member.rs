//! Membro herdado da conta (docs/members.md): lançamento sem membro numa conta de um membro só
//! recebe esse membro; mudar os membros da conta leva junto os herdados, não os escolhidos à mão.

use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana", "Beto"]}),
    )
    .unwrap();
    (dir, api)
}

fn id(v: &Value) -> String {
    v["id"].as_str().unwrap().to_string()
}

fn member_of(api: &Api, txn: &str) -> (Option<String>, Option<String>) {
    api.with_session(|s, _| {
        Ok(s.db.conn().query_row(
            "SELECT member_id, member_origin FROM txn WHERE id = ?1",
            [txn],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?)
    })
    .unwrap()
}

#[test]
fn transactions_inherit_the_account_member() {
    let (_d, api) = api();
    let members = api.call("household.list_members", json!({})).unwrap();
    let who = |n: &str| {
        members
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == n)
            .map(id)
            .unwrap()
    };
    let (ana, beto) = (who("Ana"), who("Beto"));
    let acc = id(&api
        .call(
            "accounts.create",
            json!({"name": "Conta da Ana", "kind": "checking", "opening_date": "2026-01-01", "member_ids": [ana]}),
        )
        .unwrap());
    let create = |desc: &str, member: Option<&str>| {
        let mut a =
            json!({"account_id": acc, "date": "2026-10-10", "amount": -1_000, "description": desc});
        if let Some(m) = member {
            a["member_id"] = json!(m);
        }
        id(&api.call("transactions.create", a).unwrap())
    };
    let herdado = create("mercado", None);
    let escolhido = create("presente", Some(&beto));
    let familia = create("conta de luz", None);
    assert_eq!(
        member_of(&api, &herdado),
        (Some(ana.clone()), Some("auto".into()))
    );
    assert_eq!(
        member_of(&api, &escolhido),
        (Some(beto.clone()), Some("manual".into()))
    );
    // "Família" escolhido à mão: sem membro, e a conta não muda mais.
    api.call(
        "transactions.set_member",
        json!({"ids": [familia], "member_id": null}),
    )
    .unwrap();
    assert_eq!(member_of(&api, &familia), (None, Some("manual".into())));
    // A conta passa a ser do Beto: o herdado acompanha; os escolhidos ficam.
    api.call("accounts.update", json!({"id": acc, "member_ids": [beto]}))
        .unwrap();
    assert_eq!(
        member_of(&api, &herdado),
        (Some(beto.clone()), Some("auto".into()))
    );
    assert_eq!(member_of(&api, &escolhido).0, Some(beto.clone()));
    assert_eq!(member_of(&api, &familia).0, None);
    // Conta conjunta: o herdado fica sem membro (da família) e volta quando há um membro só.
    api.call(
        "accounts.update",
        json!({"id": acc, "member_ids": [ana, beto]}),
    )
    .unwrap();
    assert_eq!(member_of(&api, &herdado), (None, None));
    api.call("accounts.update", json!({"id": acc, "member_ids": [ana]}))
        .unwrap();
    assert_eq!(
        member_of(&api, &herdado),
        (Some(ana.clone()), Some("auto".into()))
    );
    // Lançamento movido para outra conta com o membro herdado: o da conta nova.
    let acc2 = id(&api
        .call(
            "accounts.create",
            json!({"name": "Cartão do Beto", "kind": "checking", "opening_date": "2026-01-01", "member_ids": [beto]}),
        )
        .unwrap());
    api.call(
        "transactions.update",
        json!({"id": herdado, "account_id": acc2}),
    )
    .unwrap();
    assert_eq!(member_of(&api, &herdado).0, Some(beto));
}
