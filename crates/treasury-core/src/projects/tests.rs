use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;
use crate::time::{add_months, fmt_date, today_local};

fn api() -> (tempfile::TempDir, Api, String) {
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
            json!({"name": "Conta", "kind": "checking", "opening_date": "2024-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, acc)
}

fn months_ago(n: i32) -> String {
    fmt_date(add_months(today_local(), -n))
}

fn id(v: &Value) -> String {
    v["id"].as_str().unwrap().to_string()
}

fn i(v: &Value, ptr: &str) -> i64 {
    v.pointer(ptr)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("{ptr} em {v}"))
}

#[test]
fn project_groups_spending_and_drives_the_improvement_of_the_asset() {
    let (_d, api, acc) = api();
    let house = id(&api
        .call(
            "physical.create",
            json!({"name": "Casa", "kind": "real_estate", "purchase_date": months_ago(24),
                   "purchase_price": 50_000_000, "valuation_method": "none"}),
        )
        .unwrap());
    // Benfeitoria nova já com o projeto dela (sem valor: vale o orçamento até ter gastos).
    let err = api
        .call(
            "physical.add_improvement",
            json!({"asset_id": house, "date": months_ago(3), "description": "Varanda",
                   "category": "expansion", "amount": 0, "create_project": true}),
        )
        .unwrap_err();
    assert!(err.to_string().contains("valor"), "{err}");
    let v = api
        .call(
            "physical.add_improvement",
            json!({"asset_id": house, "date": months_ago(3), "description": "Varanda",
                   "category": "expansion", "amount": 1_000_000, "create_project": true}),
        )
        .unwrap();
    let imp = &v["improvements"][0];
    let project = imp["project_id"].as_str().unwrap().to_string();
    assert_eq!(imp["project_name"], "Varanda");
    assert_eq!(i(imp, "/linked_count"), 0);
    assert_eq!(i(&v, "/figures/cost_basis"), 51_000_000);

    let t1 = id(&api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": months_ago(3), "amount": -400_000,
                   "description": "Madeira", "project_id": project}),
        )
        .unwrap());
    let t2 = id(&api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": months_ago(2), "amount": -900_000,
                   "description": "PIX marceneiro"}),
        )
        .unwrap());
    let t3 = id(&api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": months_ago(2), "amount": 50_000,
                   "kind": "expense", "description": "Estorno loja"}),
        )
        .unwrap());
    let r = api
        .call(
            "projects.link",
            json!({"txn_ids": [t2, t3, t2], "project_id": project}),
        )
        .unwrap();
    assert_eq!(i(&r, "/updated"), 2);

    let p = api.call("projects.get", json!({"id": project})).unwrap();
    assert_eq!(i(&p, "/spent"), 1_250_000, "saídas menos o estorno");
    assert_eq!(i(&p, "/txn_count"), 3);
    assert_eq!(p["physical_asset_name"], "Casa");
    assert_eq!(p["improvement_ids"].as_array().unwrap().len(), 1);
    assert_eq!(p["first_date"], months_ago(3).as_str());

    // A benfeitoria passa a valer o gasto do projeto; o valor informado fica guardado.
    let v = api.call("physical.get", json!({"id": house})).unwrap();
    let imp = &v["improvements"][0];
    assert_eq!(i(imp, "/amount"), 1_250_000);
    assert_eq!(i(imp, "/entered_amount"), 1_000_000);
    assert_eq!(i(imp, "/linked_count"), 3);
    assert_eq!(i(&v, "/figures/cost_basis"), 51_250_000);

    // Lançamentos do projeto pelo filtro e com o nome do projeto.
    let q = api
        .call("transactions.query", json!({"project_ids": [project]}))
        .unwrap();
    assert_eq!(i(&q, "/total"), 3);
    assert_eq!(q["items"][0]["project_name"], "Varanda");
    let t = api.call("transactions.get", json!({"id": t1})).unwrap();
    assert_eq!(t["project_id"], project.as_str());

    // Tirar um lançamento pela edição.
    api.call("transactions.update", json!({"id": t1, "project_id": null}))
        .unwrap();
    let p = api.call("projects.get", json!({"id": project})).unwrap();
    assert_eq!(i(&p, "/spent"), 850_000);

    // Orçamento e lista.
    let p = api
        .call(
            "projects.update",
            json!({"id": project, "budget": 2_000_000, "status": "done"}),
        )
        .unwrap();
    assert_eq!(i(&p, "/remaining"), 1_150_000);
    let open = api
        .call("projects.list", json!({"include_done": false}))
        .unwrap();
    assert!(open.as_array().unwrap().is_empty());
    let all = api
        .call("projects.list", json!({"physical_asset_id": house}))
        .unwrap();
    assert_eq!(all.as_array().unwrap().len(), 1);

    // Apagar o projeto: lançamentos e benfeitoria ficam, sem o projeto.
    api.call("projects.delete", json!({"id": project})).unwrap();
    let t = api.call("transactions.get", json!({"id": t3})).unwrap();
    assert!(t["project_id"].is_null());
    let v = api.call("physical.get", json!({"id": house})).unwrap();
    assert!(v["improvements"][0]["project_id"].is_null());
    assert_eq!(i(&v, "/improvements/0/amount"), 1_000_000);
}

#[test]
fn project_validation() {
    let (_d, api, acc) = api();
    assert!(api.call("projects.create", json!({"name": "  "})).is_err());
    assert!(
        api.call(
            "projects.create",
            json!({"name": "Festa", "start_date": "2026-05-10", "end_date": "2026-05-01"}),
        )
        .is_err()
    );
    assert!(
        api.call(
            "projects.create",
            json!({"name": "Festa", "physical_asset_id": "nao-existe"}),
        )
        .is_err()
    );
    let t = id(&api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-05-01", "amount": -100, "description": "x"}),
        )
        .unwrap());
    assert!(
        api.call(
            "projects.link",
            json!({"txn_ids": [t], "project_id": "nao-existe"})
        )
        .is_err()
    );
    let p = api
        .call("projects.create", json!({"name": "Festa de 15 anos"}))
        .unwrap();
    assert_eq!(p["status"], "active");
    assert_eq!(i(&p, "/spent"), 0);
    assert!(p["remaining"].is_null());
}

#[test]
fn manual_items_and_income_enter_the_totals_and_the_entries() {
    let (_d, api, acc) = api();
    let house = id(&api
        .call(
            "physical.create",
            json!({"name": "Casa", "kind": "real_estate", "purchase_date": months_ago(24),
                   "purchase_price": 50_000_000, "valuation_method": "none"}),
        )
        .unwrap());
    let v = api
        .call(
            "physical.add_improvement",
            json!({"asset_id": house, "date": months_ago(3), "description": "Edícula",
                   "category": "expansion", "amount": 1_000_000, "create_project": true}),
        )
        .unwrap();
    let project = v["improvements"][0]["project_id"]
        .as_str()
        .unwrap()
        .to_string();
    let t1 = id(&api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": months_ago(3), "amount": -400_000,
                   "description": "Cimento", "project_id": project}),
        )
        .unwrap());
    // Receita das contas: venda de sobras de material.
    let t2 = id(&api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": months_ago(1), "amount": 30_000,
                   "description": "Venda de sobras", "project_id": project}),
        )
        .unwrap());

    // Itens à mão: despesa paga em dinheiro e uma receita.
    let e = api
        .call(
            "projects.add_item",
            json!({"project_id": project, "date": months_ago(2), "flow": "expense",
                   "amount": 250_000, "description": "Pedreiro (dinheiro)", "notes": "  "}),
        )
        .unwrap();
    assert_eq!(e["source"], "item");
    assert_eq!(e["flow"], "expense");
    assert_eq!(i(&e, "/amount"), -250_000);
    assert!(e["notes"].is_null());
    assert!(e["account_id"].is_null());
    assert_eq!(e["currency"], "BRL");
    let item = id(&e);
    let gift = id(&api
        .call(
            "projects.add_item",
            json!({"project_id": project, "date": months_ago(1), "flow": "income",
                   "amount": 100_000, "description": "Ajuda do sogro"}),
        )
        .unwrap());

    let p = api.call("projects.get", json!({"id": project})).unwrap();
    assert_eq!(i(&p, "/spent"), 650_000, "lançamento + item de despesa");
    assert_eq!(i(&p, "/income"), 130_000, "lançamento de receita + item");
    assert_eq!(i(&p, "/net"), -520_000);
    assert_eq!(i(&p, "/txn_count"), 2);
    assert_eq!(i(&p, "/item_count"), 2);
    assert_eq!(p["first_date"], months_ago(3).as_str());
    assert_eq!(p["last_date"], months_ago(1).as_str());
    assert_eq!(i(&p, "/by_category/0/spent"), 650_000);
    assert_eq!(i(&p, "/income_by_category/0/spent"), 130_000);

    // A benfeitoria vale as despesas (as receitas não abatem).
    let v = api.call("physical.get", json!({"id": house})).unwrap();
    assert_eq!(i(&v, "/improvements/0/amount"), 650_000);
    assert_eq!(i(&v, "/improvements/0/linked_count"), 2);

    // Lista em ordem de data, com a origem de cada entrada.
    let list = api
        .call("projects.entries", json!({"id": project}))
        .unwrap();
    let list = list.as_array().unwrap();
    let got: Vec<(&str, &str, i64)> = list
        .iter()
        .map(|e| {
            (
                e["source"].as_str().unwrap(),
                e["description"].as_str().unwrap(),
                e["amount"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            ("txn", "Cimento", -400_000),
            ("item", "Pedreiro (dinheiro)", -250_000),
            ("txn", "Venda de sobras", 30_000),
            ("item", "Ajuda do sogro", 100_000),
        ]
    );
    assert_eq!(list[0]["account_name"], "Conta");
    assert_eq!(list[2]["flow"], "income");

    // Editar: vira receita com outro valor; categoria inexistente e valor zero são recusados.
    let e = api
        .call(
            "projects.update_item",
            json!({"id": item, "flow": "income", "amount": 50_000}),
        )
        .unwrap();
    assert_eq!(i(&e, "/amount"), 50_000);
    assert!(
        api.call(
            "projects.update_item",
            json!({"id": item, "category_id": "nao-existe"})
        )
        .is_err()
    );
    assert!(
        api.call("projects.update_item", json!({"id": item, "amount": 0}))
            .is_err()
    );
    let p = api.call("projects.get", json!({"id": project})).unwrap();
    assert_eq!(i(&p, "/spent"), 400_000);
    assert_eq!(i(&p, "/income"), 180_000);

    // Tirar do projeto: o lançamento fica na conta, o item é apagado; desfazer devolve tudo.
    let r = api
        .call(
            "projects.remove_entries",
            json!({"txn_ids": [t1], "item_ids": [gift, gift]}),
        )
        .unwrap();
    assert_eq!(i(&r, "/updated"), 2);
    let t = api.call("transactions.get", json!({"id": t1})).unwrap();
    assert!(t["project_id"].is_null());
    let p = api.call("projects.get", json!({"id": project})).unwrap();
    assert_eq!(i(&p, "/item_count"), 1);
    assert_eq!(i(&p, "/spent"), 0);
    api.call(
        "changes.undo",
        json!({"id": r["change_set_id"].as_str().unwrap()}),
    )
    .unwrap();
    let p = api.call("projects.get", json!({"id": project})).unwrap();
    assert_eq!(i(&p, "/item_count"), 2);
    assert_eq!(i(&p, "/spent"), 400_000);

    // Apagar o projeto apaga os itens; desfazer volta com eles.
    api.call("projects.delete", json!({"id": project})).unwrap();
    assert!(
        api.call("projects.update_item", json!({"id": item, "amount": 1}))
            .is_err()
    );
    let t = api.call("transactions.get", json!({"id": t2})).unwrap();
    assert!(t["project_id"].is_null());
}

#[test]
fn item_validation() {
    let (_d, api, _acc) = api();
    let p = id(&api
        .call("projects.create", json!({"name": "Festa"}))
        .unwrap());
    let base = json!({"project_id": p, "date": "2026-05-01", "flow": "expense",
                      "amount": 1_000, "description": "Bolo"});
    let with = |k: &str, v: Value| {
        let mut a = base.clone();
        a[k] = v;
        a
    };
    assert!(api.call("projects.add_item", base.clone()).is_ok());
    for bad in [
        with("amount", json!(0)),
        with("amount", json!(-5)),
        with("description", json!("  ")),
        with("date", json!("2026-13-01")),
        with("project_id", json!("nao-existe")),
        with("category_id", json!("nao-existe")),
        with("flow", json!("transfer")),
    ] {
        assert!(api.call("projects.add_item", bad.clone()).is_err(), "{bad}");
    }
    assert!(
        api.call("projects.entries", json!({"id": "nao-existe"}))
            .is_err()
    );
}
