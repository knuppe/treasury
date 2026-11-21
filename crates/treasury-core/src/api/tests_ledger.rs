//! Fluxos do livro-caixa pelo despachante (`Api::call`): família, contas, categorias,
//! lançamentos, transferências, cartões, parcelamentos, recorrências e anexos.

use chrono::Duration;
use serde_json::{Value, json};

use super::*;
use crate::time::{fmt_date, today_local};

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Carlos", "Ana"]}),
    )
    .unwrap();
    (dir, api)
}

fn day(offset: i64) -> String {
    fmt_date(today_local() + Duration::days(offset))
}

fn call(api: &Api, cmd: &str, args: Value) -> Value {
    api.call(cmd, args)
        .unwrap_or_else(|e| panic!("{cmd} falhou: {e}"))
}

fn id(v: &Value) -> String {
    v["id"].as_str().unwrap().to_string()
}

fn category(api: &Api, name: &str) -> String {
    let list = call(api, "categories.list", json!({}));
    list.as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .map(id)
        .unwrap_or_else(|| panic!("categoria {name}"))
}

fn last_change(api: &Api) -> Value {
    call(api, "changes.list", json!({"limit": 1}))[0].clone()
}

#[test]
fn household_members_and_institutions() {
    let (_d, api) = api();
    let h = call(&api, "household.update", json!({"name": "Família Exemplo"}));
    assert_eq!(h["name"], "Família Exemplo");
    let m = call(&api, "household.create_member", json!({"name": "Bia"}));
    assert_eq!(m["is_primary"], false);
    call(&api, "household.archive_member", json!({"id": m["id"]}));
    assert_eq!(
        call(&api, "household.list_members", json!({}))
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        call(
            &api,
            "household.list_members",
            json!({"include_archived": true})
        )
        .as_array()
        .unwrap()
        .len(),
        3
    );
    let inst = call(
        &api,
        "household.create_institution",
        json!({"name": "Banco Inter", "kind": "bank", "code": "077"}),
    );
    let acc = call(
        &api,
        "accounts.create",
        json!({"name": "Inter", "kind": "checking", "institution_id": inst["id"]}),
    );
    assert_eq!(acc["institution_name"], "Banco Inter");
    assert_eq!(
        api.call("household.delete_institution", json!({"id": inst["id"]}))
            .unwrap_err()
            .code(),
        "conflict"
    );
    assert_eq!(last_change(&api)["reason"], "Conta criada: Inter");
}

#[test]
fn transactions_flow_query_update_delete_and_undo() {
    let (_d, api) = api();
    let acc = call(
        &api,
        "accounts.create",
        json!({"name": "Corrente", "kind": "checking", "opening_balance": 100000, "opening_date": day(-60)}),
    );
    let savings = call(
        &api,
        "accounts.create",
        json!({"name": "Poupança", "kind": "savings"}),
    );
    let mercado = category(&api, "Mercado");
    let t = call(
        &api,
        "transactions.create",
        json!({"account_id": acc["id"], "date": day(-1), "amount": -25000, "description": "Supermercado São Jorge",
               "category_id": mercado, "tags": ["casa"]}),
    );
    assert_eq!(t["kind"], "expense");
    assert_eq!(t["category_path"], "Alimentação › Mercado");
    assert_eq!(t["tags"][0]["name"], "casa");
    assert_eq!(
        last_change(&api)["reason"],
        "Lançamento criado: Supermercado São Jorge"
    );
    call(
        &api,
        "transactions.create",
        json!({"account_id": acc["id"], "date": day(-2), "amount": 500000, "description": "Salário"}),
    );
    // Divisão que não fecha é recusada com mensagem em português.
    let err = api
        .call(
            "transactions.create",
            json!({"account_id": acc["id"], "date": day(-1), "amount": -1000, "description": "x",
                   "splits": [{"amount": -600}, {"amount": -300}]}),
        )
        .unwrap_err();
    assert_eq!(err.code(), "invalid");
    assert!(err.to_string().contains("difere do total"));

    let page = call(&api, "transactions.query", json!({"text": "SAO JORGE"}));
    assert_eq!(page["total"], 1);
    let page = call(
        &api,
        "transactions.query",
        json!({"category_ids": [category(&api, "Alimentação")], "limit": 10}),
    );
    assert_eq!(page["items"][0]["id"], t["id"]);
    let page = call(&api, "transactions.query", json!({}));
    assert_eq!(page["total"], 2);
    assert_eq!(page["totals"][0]["net"], 475000);

    let u = call(
        &api,
        "transactions.update",
        json!({"id": t["id"], "amount": -26000, "notes": "com nota", "category_id": null}),
    );
    assert_eq!(u["amount"], -26000);
    assert_eq!(u["category_id"], Value::Null);
    let a = call(&api, "accounts.get", json!({"id": acc["id"]}));
    assert_eq!(a["balance"], 100000 - 26000 + 500000);

    // Transferência, apagar uma perna e desfazer pelo histórico.
    let tr = call(
        &api,
        "transactions.create_transfer",
        json!({"from_account_id": acc["id"], "to_account_id": savings["id"], "date": day(0), "amount": 100000}),
    );
    assert_eq!(tr["to"]["amount"], 100000);
    assert_eq!(
        last_change(&api)["reason"],
        "Transferência: Corrente → Poupança"
    );
    let del = call(
        &api,
        "transactions.delete",
        json!({"ids": [tr["from"]["id"]]}),
    );
    assert_eq!(del["updated"], 2);
    assert!(
        api.call("transactions.get", json!({"id": tr["to"]["id"]}))
            .is_err()
    );
    call(&api, "changes.undo", json!({"id": del["change_set_id"]}));
    assert_eq!(
        call(&api, "transactions.get", json!({"id": tr["to"]["id"]}))["amount"],
        100000
    );
    assert_eq!(
        call(&api, "transactions.get", json!({"id": tr["from"]["id"]}))["amount"],
        -100000
    );

    // Status e categoria em lote.
    let r = call(
        &api,
        "transactions.set_status",
        json!({"ids": [t["id"]], "status": "reconciled"}),
    );
    assert_eq!(r["updated"], 1);
    assert_eq!(
        api.call("transactions.update", json!({"id": t["id"], "amount": -1}))
            .unwrap_err()
            .code(),
        "conflict"
    );
    let r = call(
        &api,
        "transactions.set_category",
        json!({"ids": [t["id"]], "category_id": mercado}),
    );
    assert_eq!(r["updated"], 1);
    let tags = call(&api, "transactions.list_tags", json!({}));
    assert_eq!(tags[0]["txn_count"], 1);
}

#[test]
fn reconcile_and_balance_history() {
    let (_d, api) = api();
    let acc = call(
        &api,
        "accounts.create",
        json!({"name": "Corrente", "kind": "checking", "opening_balance": 1000, "opening_date": day(-30)}),
    );
    call(
        &api,
        "transactions.create",
        json!({"account_id": acc["id"], "date": day(-5), "amount": -300, "description": "a"}),
    );
    let r = call(
        &api,
        "accounts.reconcile",
        json!({"account_id": acc["id"], "statement_date": day(-1), "statement_balance": 800, "apply": true}),
    );
    assert_eq!(
        (r["difference"].as_i64(), r["marked_reconciled"].as_i64()),
        (Some(100), Some(0))
    );
    assert_eq!(r["change_set_id"], Value::Null);
    let r = call(
        &api,
        "accounts.reconcile",
        json!({"account_id": acc["id"], "statement_date": day(-1), "statement_balance": 700, "apply": true}),
    );
    assert_eq!(r["marked_reconciled"], 1);
    assert!(r["change_set_id"].is_string());
    assert_eq!(
        call(&api, "accounts.get", json!({"id": acc["id"]}))["reconciled_balance"],
        700
    );
    let h = call(
        &api,
        "accounts.balance_history",
        json!({"id": acc["id"], "from": day(-6), "to": day(-4), "interval": "day"}),
    );
    let vals: Vec<i64> = h
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["balance"].as_i64().unwrap())
        .collect();
    assert_eq!(vals, vec![1000, 700, 700]);
    assert_eq!(
        api.call("accounts.delete", json!({"id": acc["id"]}))
            .unwrap_err()
            .code(),
        "conflict"
    );
}

#[test]
fn cards_installments_payment_and_commitments() {
    let (_d, api) = api();
    let bank = call(
        &api,
        "accounts.create",
        json!({"name": "Corrente", "kind": "checking", "opening_balance": 1000000}),
    );
    let card = call(
        &api,
        "accounts.create",
        json!({"name": "Cartão", "kind": "credit_card",
               "card": {"credit_limit": 1000000, "closing_day": 3, "due_day": 10,
                        "payment_account_id": bank["id"], "network": "Visa", "last4": "9876"}}),
    );
    assert!(
        api.call(
            "accounts.create",
            json!({"name": "Sem dados", "kind": "credit_card"})
        )
        .is_err()
    );
    let plan = call(
        &api,
        "cards.create_installments",
        json!({"account_id": card["id"], "description": "Notebook", "total_amount": 1000000,
               "installments": 10, "purchase_date": day(0), "category_id": category(&api, "Eletrônicos")}),
    );
    assert_eq!(plan["installments"], 10);
    assert_eq!(plan["paid_installments"], 1);
    assert_eq!(plan["remaining_amount"], 900000);
    assert_eq!(
        last_change(&api)["reason"],
        "Compra parcelada em 10x: Notebook"
    );
    let c = call(&api, "accounts.get", json!({"id": card["id"]}));
    assert_eq!(
        c["available_credit"], 0,
        "parcelas futuras comprometem o limite"
    );
    let page = call(
        &api,
        "transactions.query",
        json!({"account_ids": [card["id"]], "sort": "date_asc", "limit": 2}),
    );
    assert_eq!(page["items"][0]["description"], "Notebook 1/10");
    assert_eq!(page["items"][1]["status"], "planned");
    assert_eq!(page["items"][1]["installment_total"], 10);

    let statements = call(
        &api,
        "cards.list_statements",
        json!({"account_id": card["id"]}),
    );
    let statements = statements.as_array().unwrap();
    assert_eq!(statements.len(), 10);
    let first = &statements[0];
    assert_eq!(first["total"], 100000);
    let paid = call(
        &api,
        "cards.pay_statement",
        json!({"statement_id": first["id"], "from_account_id": "", "date": day(0)}),
    );
    assert_eq!(paid["statement"]["status"], "paid");
    assert_eq!(paid["statement"]["remaining"], 0);
    assert_eq!(
        paid["transfer"]["from"]["account_id"], bank["id"],
        "conta pagadora do cartão"
    );
    assert_eq!(paid["transfer"]["to"]["kind"], "card_payment");
    let detail = call(&api, "cards.get_statement", json!({"id": first["id"]}));
    assert_eq!(detail["transactions"].as_array().unwrap().len(), 2);
    assert_eq!(
        detail["statement"]["total"], 100000,
        "pagamento não entra no total"
    );

    let fc = call(
        &api,
        "cards.future_commitments",
        json!({"account_id": card["id"], "months": 14}),
    );
    let total: i64 = fc
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["amount"].as_i64().unwrap())
        .sum();
    assert_eq!(total, 900000);
    let plans = call(&api, "cards.list_installment_plans", json!({}));
    assert_eq!(plans.as_array().unwrap().len(), 1);
    let r = call(
        &api,
        "cards.delete_installment_plan",
        json!({"id": plan["id"]}),
    );
    assert_eq!(r["updated"], 10);
    assert_eq!(
        call(
            &api,
            "transactions.query",
            json!({"account_ids": [card["id"]]})
        )["total"],
        1,
        "sobra só a perna do pagamento"
    );
}

#[test]
fn recurrences_categories_and_attachments() {
    let (_d, api) = api();
    let acc = call(
        &api,
        "accounts.create",
        json!({"name": "Corrente", "kind": "checking"}),
    );
    let rec = call(
        &api,
        "recurrences.create",
        json!({"description": "Academia", "account_id": acc["id"], "amount": -12000, "kind": "expense",
               "category_id": category(&api, "Academia"),
               "spec": {"freq": "MONTHLY", "by_month_day": 31}, "start_date": day(0)}),
    );
    assert_eq!(rec["human"], "todo mês no último dia");
    assert_eq!(rec["rrule"], "FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=31");
    let planned = call(
        &api,
        "transactions.query",
        json!({"statuses": ["planned"], "account_ids": [acc["id"]]}),
    );
    assert!(planned["total"].as_i64().unwrap() >= 2);
    assert_eq!(
        call(&api, "recurrences.materialize", json!({}))["created"],
        0,
        "idempotente"
    );
    let prev = call(
        &api,
        "recurrences.preview",
        json!({"spec": {"freq": "WEEKLY", "interval": 2, "by_day": ["MO"]}, "start_date": "2026-10-12", "count": 2}),
    );
    assert_eq!(prev["dates"], json!(["2026-10-12", "2026-10-26"]));
    assert_eq!(prev["human"], "a cada 2 semanas às segundas");
    assert!(
        api.call(
            "recurrences.preview",
            json!({"spec": {"freq": "MONTHLY", "by_day": ["1MO"]}, "start_date": "2026-10-12"})
        )
        .is_err()
    );
    let upd = call(
        &api,
        "recurrences.update",
        json!({"id": rec["id"], "amount": -13000}),
    );
    assert_eq!(upd["amount"], -13000);
    let list = call(&api, "recurrences.list", json!({}));
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Categoria em uso: só apaga com reatribuição.
    let academia = category(&api, "Academia");
    assert_eq!(
        api.call("categories.delete", json!({"id": academia}))
            .unwrap_err()
            .code(),
        "conflict"
    );
    let usage = call(&api, "categories.usage", json!({"id": academia}));
    assert_eq!(usage["recurrences"], 1);
    let moved = call(
        &api,
        "categories.delete",
        json!({"id": academia, "reassign_to": category(&api, "Cabelo e estética")}),
    );
    assert_eq!(moved["recurrences"], 1);
    let tree = call(&api, "categories.tree", json!({"kind": "income"}));
    assert!(
        tree.as_array()
            .unwrap()
            .iter()
            .all(|n| n["kind"] == "income")
    );

    let del = call(&api, "recurrences.delete", json!({"id": rec["id"]}));
    assert!(del["deleted_planned"].as_i64().unwrap() >= 2);

    // Anexos em base64.
    let t = call(
        &api,
        "transactions.create",
        json!({"account_id": acc["id"], "date": day(0), "amount": -100, "description": "Farmácia"}),
    );
    let a = call(
        &api,
        "transactions.add_attachment",
        json!({"txn_id": t["id"], "filename": "recibo.txt", "mime": "text/plain", "content_base64": "UmVjaWJv"}),
    );
    assert_eq!(a["size_bytes"], 6);
    let got = call(&api, "transactions.get_attachment", json!({"id": a["id"]}));
    assert_eq!(got["content_base64"], "UmVjaWJv");
    assert_eq!(got["filename"], "recibo.txt");
    // Salvar no disco (o app pergunta onde, o núcleo grava); agente não grava arquivos.
    let out = tempfile::tempdir().unwrap();
    let dest = out.path().join("recibo.txt");
    let saved = call(
        &api,
        "transactions.save_attachment",
        json!({"id": a["id"], "path": dest.display().to_string()}),
    );
    assert_eq!(saved["bytes"], 6);
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "Recibo");
    let agent = crate::api::Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(matches!(
        api.call_as(
            &agent,
            "transactions.save_attachment",
            json!({"id": a["id"], "path": out.path().join("x.txt").display().to_string()}),
        ),
        Err(crate::Error::Forbidden(_))
    ));
    assert!(
        api.call(
            "transactions.save_attachment",
            json!({"id": "nao-existe", "path": out.path().join("y.txt").display().to_string()}),
        )
        .is_err()
    );
    assert_eq!(
        call(
            &api,
            "transactions.list_attachments",
            json!({"txn_id": t["id"]})
        )
        .as_array()
        .unwrap()
        .len(),
        1
    );
    call(
        &api,
        "transactions.delete_attachment",
        json!({"id": a["id"]}),
    );
    assert_eq!(
        call(&api, "transactions.get", json!({"id": t["id"]}))["attachment_count"],
        0
    );
}

#[test]
fn unlock_clears_due_installments_as_system() {
    let (_d, api) = api();
    let card = call(
        &api,
        "accounts.create",
        json!({"name": "Cartão", "kind": "credit_card", "card": {"closing_day": 3, "due_day": 10}}),
    );
    call(
        &api,
        "cards.create_installments",
        json!({"account_id": card["id"], "description": "Sofá", "total_amount": 3000,
               "installments": 3, "purchase_date": day(0)}),
    );
    // Simula a passagem do tempo: a 2ª parcela "chegou".
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE txn SET date = ?1 WHERE installment_number = 2",
            [day(0)],
        )?;
        Ok(())
    })
    .unwrap();
    call(&api, "vault.lock", Value::Null);
    call(
        &api,
        "vault.unlock",
        json!({"password": "senha-mestra-forte"}),
    );
    let page = call(
        &api,
        "transactions.query",
        json!({"account_ids": [card["id"]], "statuses": ["planned"]}),
    );
    assert_eq!(page["total"], 1);
    let ch = last_change(&api);
    assert_eq!(ch["reason"], "Parcelas do cartão efetivadas");
    assert_eq!(ch["author"]["kind"], "system");
}

#[test]
fn read_only_mcp_can_query_but_not_write() {
    let (_d, api) = api();
    let mcp = Actor::Mcp {
        client: "cli-teste".into(),
        read_only: true,
    };
    for (cmd, args) in [
        ("accounts.list", json!({})),
        ("transactions.query", json!({})),
        ("transactions.list_tags", json!({})),
        ("categories.tree", json!({})),
        ("household.list_members", json!({})),
        ("recurrences.list", json!({})),
        ("cards.list_statements", json!({})),
        ("cards.future_commitments", json!({})),
    ] {
        assert!(api.call_as(&mcp, cmd, args).is_ok(), "{cmd}");
    }
    for cmd in [
        "accounts.create",
        "accounts.reconcile",
        "transactions.create",
        "transactions.delete",
        "categories.merge",
        "recurrences.materialize",
        "cards.pay_statement",
        "household.update",
    ] {
        assert!(
            matches!(api.call_as(&mcp, cmd, json!({})), Err(Error::Forbidden(_))),
            "{cmd}"
        );
    }
    // Um cliente MCP com escrita não mexe no que está fora da política da IA (câmbio
    // manual, ajustes, cofre) e, nos comandos permitidos (inclusive cadastrar conta), gera
    // propostas que gravam com autor IA depois da aprovação.
    let writer = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(matches!(
        api.call_as(
            &writer,
            "fx.set",
            json!({"base": "USD", "quote": "BRL", "date": "2026-01-01", "rate": "5"})
        ),
        Err(Error::Forbidden(_))
    ));
    let p = api
        .call_as(
            &writer,
            "accounts.create",
            json!({"name": "Carteira", "kind": "cash"}),
        )
        .unwrap();
    assert_eq!(p["status"], "pending_approval");
    api.call("ai.reject", json!({"ids": [p["proposal_id"]]}))
        .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Corrente", "kind": "checking"}),
        )
        .unwrap()["id"]
        .clone();
    let r = api
        .call_as(
            &writer,
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-01", "amount": -100, "description": "Café"}),
        )
        .unwrap();
    api.call("ai.approve", json!({"ids": [r["proposal_id"]]}))
        .unwrap();
    let ch = last_change(&api);
    assert_eq!(ch["author"]["kind"], "ai");
    assert_eq!(ch["author"]["detail"], "agente");
}

#[test]
fn unknown_actions_in_ledger_modules() {
    let (_d, api) = api();
    for cmd in [
        "accounts.nada",
        "transactions.nada",
        "categories.nada",
        "cards.nada",
        "recurrences.nada",
        "household.nada",
    ] {
        assert!(
            matches!(api.call(cmd, json!({})), Err(Error::UnknownCommand(_))),
            "{cmd}"
        );
    }
}

#[test]
fn asset_list_filters_open_and_closed_positions() {
    let (_d, api) = api();
    let acc = id(&call(
        &api,
        "accounts.create",
        json!({"name": "Corretora", "kind": "brokerage"}),
    ));
    let asset = |t: &str| {
        id(&call(
            &api,
            "assets.create",
            json!({"ticker": t, "name": t, "class": "stock_br"}),
        ))
    };
    let (open, closed, _watch) = (asset("ITSA4"), asset("MGLU3"), asset("WEGE3"));
    let trade = |a: &str, side: &str, q: &str| {
        call(
            &api,
            "portfolio.create_trade",
            json!({"asset_id": a, "account_id": acc, "date": day(-30), "side": side,
                   "quantity": q, "price": "10", "cash_movement": false}),
        );
    };
    trade(&open, "buy", "100");
    trade(&closed, "buy", "50");
    trade(&closed, "sell", "50");
    let tickers = |pos: Value| -> Vec<String> {
        call(&api, "assets.list", json!({"position": pos}))
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["ticker"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(tickers(json!("open")), ["ITSA4"]);
    assert_eq!(tickers(json!("closed")), ["MGLU3"]);
    assert_eq!(tickers(json!("traded")).len(), 2);
    assert_eq!(tickers(Value::Null).len(), 3);
}

#[test]
fn transactions_sort_by_description_account_and_status() {
    let (_d, api) = api();
    let mk = |name: &str| {
        id(&call(
            &api,
            "accounts.create",
            json!({"name": name, "kind": "checking", "opening_date": "2026-01-01"}),
        ))
    };
    let (a, b) = (mk("Banco B"), mk("Água Conta"));
    for (acc, desc, d, st) in [
        (&a, "Zebra", -3, "cleared"),
        (&b, "árvore", -2, "planned"),
        (&a, "Mercado", -1, "cleared"),
    ] {
        call(
            &api,
            "transactions.create",
            json!({"account_id": acc, "date": day(d), "amount": -100, "description": desc, "status": st}),
        );
    }
    let order = |sort: &str| -> Vec<String> {
        call(&api, "transactions.query", json!({"sort": sort}))["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["description"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(order("description_asc"), ["árvore", "Mercado", "Zebra"]);
    assert_eq!(order("description_desc"), ["Zebra", "Mercado", "árvore"]);
    assert_eq!(order("account_asc")[0], "árvore");
    assert_eq!(order("status_asc")[0], "árvore");
}

/// Cópias entre contas pela API: a lista traz o par e só a interface apaga, e só a cópia.
#[test]
fn cross_account_copies_via_api() {
    let (_d, api) = api();
    let inst = call(
        &api,
        "household.create_institution",
        json!({"name": "Nubank", "kind": "bank"}),
    );
    let card = call(
        &api,
        "accounts.create",
        json!({"name": "Nubank Cartão", "kind": "credit_card", "institution_id": inst["id"],
               "card": {"closing_day": 3, "due_day": 10}}),
    );
    let conta = call(
        &api,
        "accounts.create",
        json!({"name": "Nubank", "kind": "checking", "institution_id": inst["id"]}),
    );
    let mk = |acc: &Value| {
        call(
            &api,
            "transactions.create",
            json!({"account_id": acc["id"], "date": day(0), "amount": -2350, "description": "Padaria"}),
        )["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let keep = mk(&card);
    let copy = mk(&conta);
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE txn SET external_id = 'abc-1' WHERE id IN (?1, ?2)",
            rusqlite::params![keep, copy],
        )?;
        Ok(())
    })
    .unwrap();
    let list = call(&api, "transactions.cross_account_duplicates", json!({}));
    assert_eq!(list.as_array().unwrap().len(), 1, "{list}");
    assert_eq!(list[0]["keep"]["id"], keep.as_str());
    assert_eq!(list[0]["copy"]["id"], copy.as_str());
    let agent = crate::api::Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(matches!(
        api.call_as(&agent, "transactions.remove_copies", json!({"ids": [copy]})),
        Err(crate::Error::Forbidden(_))
    ));
    // A conta foi sincronizada com o banco com a cópia dentro (o ajuste compensou a cópia) e
    // a cópia ficou conciliada: apagar a cópia refaz o marco e o ajuste some.
    let bal = |api: &Api| {
        call(api, "accounts.get", json!({"id": conta["id"]}))["balance"]
            .as_i64()
            .unwrap()
    };
    let bank = bal(&api) + 2350;
    call(
        &api,
        "accounts.sync_balance",
        json!({"account_id": conta["id"], "date": day(0), "balance": bank}),
    );
    let syncs = call(
        &api,
        "accounts.list_syncs",
        json!({"account_id": conta["id"]}),
    );
    assert_eq!(syncs[0]["difference"], 2350, "{syncs}");
    assert_eq!(
        call(&api, "transactions.get", json!({"id": copy}))["status"],
        "reconciled"
    );
    let r = call(
        &api,
        "transactions.remove_copies",
        json!({"ids": [copy, keep]}),
    );
    assert_eq!(r["removed"], 1);
    assert_eq!(r["balance_syncs_adjusted"], 1);
    let syncs = call(
        &api,
        "accounts.list_syncs",
        json!({"account_id": conta["id"]}),
    );
    assert_eq!(syncs[0]["difference"], 0, "{syncs}");
    assert!(syncs[0]["adjustment_txn_id"].is_null(), "{syncs}");
    assert_eq!(syncs[0]["status"], "explained");
    assert_eq!(bal(&api), bank);
    assert_eq!(r["skipped"], json!([keep]));
    assert!(r["change_set_id"].is_string());
    assert!(api.call("transactions.get", json!({"id": copy})).is_err());
    call(&api, "transactions.get", json!({"id": keep}));
}

/// O caso do Nubank: o arquivo do cartão importado também na conta, e a ligação automática
/// entre as duas contas juntou "IOF de X" (cópia na conta) com "IOF de volta de X" (no cartão)
/// como pagamento de fatura. O detector não sugere mais esses pares, e apagar a cópia desfaz a
/// ligação: o "IOF de volta" volta a ser receita (reembolso) no cartão.
#[test]
fn iof_refund_on_the_card_is_not_a_bill_payment() {
    let (_d, api) = api();
    let inst = call(
        &api,
        "household.create_institution",
        json!({"name": "Nubank", "kind": "bank"}),
    );
    let card = call(
        &api,
        "accounts.create",
        json!({"name": "Nubank Cartão", "kind": "credit_card", "institution_id": inst["id"],
               "card": {"closing_day": 3, "due_day": 10}}),
    );
    let conta = call(
        &api,
        "accounts.create",
        json!({"name": "Nubank", "kind": "checking", "institution_id": inst["id"]}),
    );
    let mk = |acc: &Value, amount: i64, desc: &str, ext: &str| {
        let id = call(
            &api,
            "transactions.create",
            json!({"account_id": acc["id"], "date": day(0), "amount": amount, "description": desc}),
        )["id"]
            .as_str()
            .unwrap()
            .to_string();
        api.with_session(|s, _| {
            s.db.conn().execute(
                "UPDATE txn SET external_id = ?2 WHERE id = ?1",
                rusqlite::params![id, ext],
            )?;
            Ok(())
        })
        .unwrap();
        id
    };
    let iof = mk(&card, -907, "IOF de \"Grok Xai\"", "ext-iof-1");
    let back = mk(&card, 907, "IOF de volta de Grok Xai", "ext-iof-2");
    // Sem as cópias: o IOF devolvido no cartão não é pagamento da fatura com um débito da conta.
    let loose = mk(&conta, -907, "Débito avulso", "ext-avulso");
    let pairs = call(&api, "transactions.transfer_candidates", json!({}));
    assert!(
        !pairs
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["to"]["txn_id"] == back.as_str()),
        "{pairs}"
    );
    call(&api, "transactions.delete", json!({"ids": [loose]}));
    // As cópias na conta, ligadas como fizeram as versões antigas.
    let copy_iof = mk(&conta, -907, "IOF de \"Grok Xai\"", "ext-iof-1");
    let copy_back = mk(&conta, 907, "IOF de volta de Grok Xai", "ext-iof-2");
    let pairs = call(&api, "transactions.transfer_candidates", json!({}));
    assert_eq!(
        pairs.as_array().unwrap().len(),
        0,
        "cópias não viram par: {pairs}"
    );
    call(
        &api,
        "transactions.link_transfers",
        json!({"pairs": [{"from_id": copy_iof, "to_id": back}, {"from_id": iof, "to_id": copy_back}]}),
    );
    assert_eq!(
        call(&api, "transactions.get", json!({"id": back}))["kind"],
        "card_payment"
    );
    let dups = call(&api, "transactions.cross_account_duplicates", json!({}));
    assert_eq!(dups.as_array().unwrap().len(), 2, "{dups}");
    let r = call(
        &api,
        "transactions.remove_copies",
        json!({"ids": [copy_iof, copy_back]}),
    );
    assert_eq!(r["removed"], 2, "{r}");
    let b = call(&api, "transactions.get", json!({"id": back}));
    assert_eq!(
        (b["kind"].as_str(), b["transfer_id"].is_null()),
        (Some("income"), true),
        "{b}"
    );
    let i = call(&api, "transactions.get", json!({"id": iof}));
    assert_eq!(
        (i["kind"].as_str(), i["transfer_id"].is_null()),
        (Some("expense"), true),
        "{i}"
    );
}
