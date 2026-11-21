//! Cartões adicionais e virtuais (docs/design/cards.md, fase 1): o lançamento herda o membro
//! do plástico; a importação acha o plástico pelo final; a fatura soma por plástico.

use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana", "Beto", "Clara"]}),
    )
    .unwrap();
    (dir, api)
}

fn id(v: &Value) -> String {
    v["id"].as_str().unwrap().to_string()
}

fn member(api: &Api, name: &str) -> String {
    api.call("household.list_members", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == name)
        .map(id)
        .unwrap()
}

fn member_of(api: &Api, txn: &str) -> (Option<String>, Option<String>) {
    api.with_session(|s, _| {
        Ok(s.db.conn().query_row(
            "SELECT member_id, card_plastic_id FROM txn WHERE id = ?1",
            [txn],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?)
    })
    .unwrap()
}

/// Cartão da Ana (fecha dia 3, vence dia 10) com final 1111 no cadastro.
fn card(api: &Api, ana: &str) -> String {
    id(&api
        .call(
            "accounts.create",
            json!({"name": "Cartão Azul", "kind": "credit_card", "opening_date": "2026-01-01",
                   "member_ids": [ana],
                   "card": {"closing_day": 3, "due_day": 10, "credit_limit": 500_000, "last4": "1111"}}),
        )
        .unwrap())
}

fn buy(api: &Api, card: &str, desc: &str, amount: i64, plastic: Option<&str>) -> String {
    let mut a =
        json!({"account_id": card, "date": "2026-10-10", "amount": -amount, "description": desc});
    if let Some(p) = plastic {
        a["card_plastic_id"] = json!(p);
    }
    id(&api.call("transactions.create", a).unwrap())
}

#[test]
fn purchases_take_the_member_of_the_card_used() {
    let (_d, api) = api();
    let (ana, beto, clara) = (
        member(&api, "Ana"),
        member(&api, "Beto"),
        member(&api, "Clara"),
    );
    let card = card(&api, &ana);
    // O final do cadastro virou o plástico do titular, sem membro próprio.
    let list = api
        .call("cards.list_plastics", json!({"account_id": card}))
        .unwrap();
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1, "{list:?}");
    assert_eq!(list[0]["kind"], "holder");
    assert_eq!(list[0]["last4"], "1111");
    assert!(list[0]["member_id"].is_null());
    let holder = id(&list[0]);

    let adicional = id(&api
        .call(
            "cards.create_plastic",
            json!({"account_id": card, "kind": "additional", "last4": "**** 2222",
                   "member_id": beto, "spend_limit": 50_000}),
        )
        .unwrap());
    let titular = buy(&api, &card, "Padaria", 1_000, Some(&holder));
    let do_beto = buy(&api, &card, "Farmácia", 2_000, Some(&adicional));
    let sem = buy(&api, &card, "Posto", 3_000, None);
    assert_eq!(member_of(&api, &titular).0.as_deref(), Some(ana.as_str()));
    assert_eq!(
        member_of(&api, &do_beto),
        (Some(beto.clone()), Some(adicional.clone()))
    );
    assert_eq!(member_of(&api, &sem).0.as_deref(), Some(ana.as_str()));

    // O adicional passa para a Clara: a compra dele acompanha.
    api.call(
        "cards.update_plastic",
        json!({"id": adicional, "member_id": clara}),
    )
    .unwrap();
    assert_eq!(member_of(&api, &do_beto).0.as_deref(), Some(clara.as_str()));
    // O cartão vira conjunto: o adicional continua da Clara; o resto fica da família.
    api.call(
        "accounts.update",
        json!({"id": card, "member_ids": [ana, beto]}),
    )
    .unwrap();
    assert_eq!(member_of(&api, &do_beto).0.as_deref(), Some(clara.as_str()));
    assert_eq!(member_of(&api, &titular).0, None);
    // Membro escolhido à mão não muda com o plástico.
    api.call(
        "transactions.set_member",
        json!({"ids": [do_beto], "member_id": beto}),
    )
    .unwrap();
    api.call(
        "cards.update_plastic",
        json!({"id": adicional, "member_id": null}),
    )
    .unwrap();
    assert_eq!(member_of(&api, &do_beto).0.as_deref(), Some(beto.as_str()));

    // Trocar o plástico de uma compra refaz o membro herdado.
    api.call("accounts.update", json!({"id": card, "member_ids": [ana]}))
        .unwrap();
    let virtual_ = id(&api
        .call(
            "cards.create_plastic",
            json!({"account_id": card, "kind": "virtual", "last4": "3333", "member_id": clara}),
        )
        .unwrap());
    api.call(
        "transactions.update",
        json!({"id": sem, "card_plastic_id": virtual_}),
    )
    .unwrap();
    assert_eq!(member_of(&api, &sem).0.as_deref(), Some(clara.as_str()));
    // Apagar o plástico: a compra volta para o titular da conta.
    let r = api
        .call("cards.delete_plastic", json!({"id": virtual_}))
        .unwrap();
    assert_eq!(r["updated"], 1);
    assert_eq!(member_of(&api, &sem), (Some(ana.clone()), None));
}

#[test]
fn plastic_validation() {
    let (_d, api) = api();
    let ana = member(&api, "Ana");
    let card = card(&api, &ana);
    let create = |last4: &str| {
        api.call(
            "cards.create_plastic",
            json!({"account_id": card, "kind": "additional", "last4": last4}),
        )
    };
    // Nunca o número inteiro.
    assert_eq!(create("4111 1111 1111 1111").unwrap_err().code(), "invalid");
    assert_eq!(create("12").unwrap_err().code(), "invalid");
    // Final repetido no mesmo cartão.
    assert_eq!(create("1111").unwrap_err().code(), "conflict");
    // Plástico de outro cartão não serve.
    let other = id(&api
        .call(
            "accounts.create",
            json!({"name": "Cartão Verde", "kind": "credit_card", "opening_date": "2026-01-01",
                   "card": {"closing_day": 3, "due_day": 10}}),
        )
        .unwrap());
    let p = id(&create("5555").unwrap());
    let e = api
        .call(
            "transactions.create",
            json!({"account_id": other, "date": "2026-10-10", "amount": -100,
                   "description": "X", "card_plastic_id": p}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    // Conta que não é cartão não tem plástico.
    let conta = id(&api
        .call(
            "accounts.create",
            json!({"name": "Corrente", "kind": "checking", "opening_date": "2026-01-01"}),
        )
        .unwrap());
    let e = api
        .call(
            "cards.create_plastic",
            json!({"account_id": conta, "kind": "virtual"}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
}

#[test]
fn import_by_card_ending_and_statement_subtotals() {
    let (_d, api) = api();
    let (ana, beto) = (member(&api, "Ana"), member(&api, "Beto"));
    let card = card(&api, &ana);
    api.call(
        "cards.create_plastic",
        json!({"account_id": card, "kind": "additional", "last4": "2222", "member_id": beto}),
    )
    .unwrap();
    let r = api
        .call(
            "import.rows",
            json!({"account_id": card, "source": "fatura na página", "card_last4": "1111",
            "rows": [
                {"date": "2026-09-20", "amount": -5_000, "description": "MERCADO"},
                {"date": "2026-09-21", "amount": -7_000, "description": "LIVRARIA", "card_last4": "2222"},
                {"date": "2026-09-22", "amount": -9_000, "description": "CINEMA", "card_last4": "9999"}
            ]}),
        )
        .unwrap();
    assert_eq!(r["status"], "applied", "{r}");
    assert!(
        r.to_string().contains("9999"),
        "aviso do final desconhecido: {r}"
    );
    let rows: Vec<(String, Option<String>, Option<String>)> = api
        .with_session(|s, _| {
            let mut st = s.db.conn().prepare(
                "SELECT t.description, p.last4, t.member_id FROM txn t
                  LEFT JOIN card_plastic p ON p.id = t.card_plastic_id ORDER BY t.description",
            )?;
            let v = st
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(v)
        })
        .unwrap();
    assert_eq!(
        rows,
        vec![
            ("CINEMA".into(), None, Some(ana.clone())),
            ("LIVRARIA".into(), Some("2222".into()), Some(beto.clone())),
            ("MERCADO".into(), Some("1111".into()), Some(ana.clone())),
        ]
    );
    // Final com mais de 4 dígitos é erro da linha, não vai para o banco.
    let r = api
        .call(
            "import.rows",
            json!({"account_id": card, "source": "fatura",
                   "rows": [{"date": "2026-09-23", "amount": -100, "description": "Z",
                             "card_last4": "4111111111111111"}]}),
        )
        .unwrap();
    assert!(!r.to_string().contains("4111111111111111\""), "{r}");

    let st = api
        .call("cards.list_statements", json!({"account_id": card}))
        .unwrap();
    let st = id(&st[0]);
    let detail = api.call("cards.get_statement", json!({"id": st})).unwrap();
    let subs: Vec<(Value, i64)> = detail["by_plastic"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (s["last4"].clone(), s["total"].as_i64().unwrap()))
        .collect();
    assert_eq!(
        subs,
        vec![
            (json!("1111"), 5_000),
            (json!("2222"), 7_000),
            (Value::Null, 9_000),
        ]
    );
    let tx = detail["transactions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["description"] == "LIVRARIA")
        .unwrap()
        .clone();
    assert_eq!(tx["card_last4"], "2222");
}

#[test]
fn account_last4_keeps_the_holder_card() {
    let (_d, api) = api();
    let ana = member(&api, "Ana");
    let card = card(&api, &ana);
    api.call(
        "accounts.update",
        json!({"id": card, "card": {"closing_day": 3, "due_day": 10, "last4": "7777"}}),
    )
    .unwrap();
    let list = api
        .call("cards.list_plastics", json!({"account_id": card}))
        .unwrap();
    let finals: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["last4"].as_str().unwrap())
        .collect();
    assert_eq!(finals, ["7777"]);
}
