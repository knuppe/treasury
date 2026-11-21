//! Developer feedback through the API: deduplication, order by importance, the developer
//! mode for agents and what only the user can do.

use serde_json::{Value, json};

use super::*;
use crate::api::{Actor, Api, SharedApi};
use crate::paths::TreasuryPaths;

fn setup() -> (tempfile::TempDir, SharedApi) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap()
        .into_shared();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    (dir, api)
}

fn agent() -> Actor {
    Actor::Mcp {
        client: "agente (CLI de teste)".into(),
        read_only: false,
    }
}

fn dev_on(api: &Api) {
    api.call("settings.set", json!({"developer_tools": true}))
        .unwrap();
}

fn add(api: &Api, actor: &Actor, title: &str) -> Value {
    api.call_as(
        actor,
        "devfeedback.add",
        json!({"title": title, "body": "Tentei e não deu.", "area": "mcp", "kind": "ferramenta_faltando"}),
    )
    .unwrap()
}

#[test]
fn similarity_ignores_accents_case_and_small_words() {
    assert_eq!(
        title_similarity("Falta ferramenta de fatura", "falta FERRAMENTA da fatura"),
        1.0
    );
    assert!(
        title_similarity(
            "Falta ferramenta para mover lançamento de fatura",
            "Ferramenta para mover lançamento entre faturas falta"
        ) >= DEDUPE_MIN
    );
    assert!(title_similarity("Falta ferramenta de fatura", "Erro ao importar OFX") < DEDUPE_MIN);
    assert_eq!(title_similarity("", "algo"), 0.0);
}

#[test]
fn add_dedupes_a_similar_open_title_into_a_bump() {
    let (_d, api) = setup();
    dev_on(&api);
    let first = add(
        &api,
        &agent(),
        "Falta ferramenta para mover lançamento de fatura",
    );
    assert_eq!(first["deduplicated"], false);
    let f = &first["feedback"];
    assert_eq!(f["importance"], 1);
    assert_eq!(f["status"], "aberto");
    assert_eq!(f["created_by"], "agente (CLI de teste)");
    assert_eq!(f["bumps"], 0);

    let again = add(
        &api,
        &agent(),
        "falta ferramenta para mover lancamento da fatura",
    );
    assert_eq!(again["deduplicated"], true);
    assert!(again["message"].as_str().unwrap().contains("reforçado"));
    let b = &again["feedback"];
    assert_eq!(b["id"], f["id"]);
    assert_eq!(b["importance"], 2);
    assert_eq!(b["bumps"], 1);
    let note = b["notes"][0]["note"].as_str().unwrap();
    assert!(note.contains("Tentei e não deu."), "{note}");

    // Different title: new. A resolved one gets no automatic bump.
    let other = add(&api, &agent(), "Erro ao importar OFX do Itaú");
    assert_eq!(other["deduplicated"], false);
    api.call(
        "devfeedback.set_status",
        json!({"id": other["feedback"]["id"], "status": "resolvido"}),
    )
    .unwrap();
    let fresh = add(&api, &agent(), "Erro ao importar OFX do Itaú");
    assert_eq!(fresh["deduplicated"], false);
    assert_ne!(fresh["feedback"]["id"], other["feedback"]["id"]);
}

#[test]
fn list_sorts_by_importance_then_latest_bump_and_filters() {
    let (_d, api) = setup();
    let a = add(&api, &Actor::User, "Tela de carteira lenta")["feedback"]["id"].clone();
    let b = add(
        &api,
        &Actor::User,
        "Descrição de query_transactions confusa",
    )["feedback"]["id"]
        .clone();
    let c = add(&api, &Actor::User, "Falta exportar orçamento em CSV")["feedback"]["id"].clone();
    for id in [&b, &b, &c, &c] {
        api.call("devfeedback.bump", json!({"id": id, "note": "de novo"}))
            .unwrap();
    }
    // Tie at 3: the one bumped last (c) comes first.
    let list = api.call("devfeedback.list", json!({})).unwrap();
    let ids: Vec<&Value> = list.as_array().unwrap().iter().map(|f| &f["id"]).collect();
    assert_eq!(ids, vec![&c, &b, &a]);
    assert_eq!(list[0]["notes"].as_array().unwrap().len(), 2);
    assert_eq!(list[0]["notes"][0]["client"], "user");

    api.call(
        "devfeedback.update",
        json!({"id": a, "area": "carteira", "kind": "bug", "importance": 9}),
    )
    .unwrap();
    let top = api.call("devfeedback.list", json!({})).unwrap();
    assert_eq!(top[0]["id"], a);
    let only = api
        .call("devfeedback.list", json!({"area": "carteira"}))
        .unwrap();
    assert_eq!(only.as_array().unwrap().len(), 1);
    api.call(
        "devfeedback.set_status",
        json!({"id": b, "status": "descartado"}),
    )
    .unwrap();
    let open = api
        .call("devfeedback.list", json!({"open_only": true}))
        .unwrap();
    assert_eq!(open.as_array().unwrap().len(), 2);
    let found = api
        .call("devfeedback.list", json!({"query": "orcamento csv"}))
        .unwrap();
    assert_eq!(found[0]["id"], c);
    assert_eq!(found.as_array().unwrap().len(), 1);
    // Bumping a discarded one reopens it.
    let r = api.call("devfeedback.bump", json!({"id": b})).unwrap();
    assert_eq!(r["status"], "aberto");
    let kinds: Vec<(&str, &Value)> = r["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["kind"].as_str().unwrap(), &n["note"]))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("reforco", &json!("de novo")),
            ("reforco", &json!("de novo")),
            ("status", &json!("Descartado")),
            ("reforco", &Value::Null),
            ("status", &json!("Voltou a aberto")),
        ]
    );
}

#[test]
fn agents_need_developer_mode_and_write_directly_with_history() {
    let (_d, api) = setup();
    let args = json!({"title": "X", "body": "Y", "area": "outro", "kind": "melhoria"});
    for cmd in ["devfeedback.add", "devfeedback.list"] {
        let e = api.call_as(&agent(), cmd, args.clone()).unwrap_err();
        assert!(
            matches!(&e, crate::Error::Forbidden(m) if m == "modo desenvolvedor desligado"),
            "{cmd}: {e}"
        );
    }
    // Readers (read-only clients) do not write either.
    let reader = Actor::Mcp {
        client: "x".into(),
        read_only: true,
    };
    dev_on(&api);
    assert!(matches!(
        api.call_as(&reader, "devfeedback.add", args.clone()),
        Err(crate::Error::Forbidden(_))
    ));
    let before = api.call("changes.list", json!({})).unwrap();
    let added = api.call_as(&agent(), "devfeedback.add", args).unwrap();
    let id = added["feedback"]["id"].clone();
    // Direct, no proposal, but with a change set (authored by the AI).
    assert_eq!(api.call("ai.list_proposals", json!({})).unwrap(), json!([]));
    let bumped = api
        .call_as(
            &agent(),
            "devfeedback.bump",
            json!({"id": id, "note": "senha: 1234"}),
        )
        .unwrap();
    assert_eq!(bumped["importance"], 2);
    assert!(
        !bumped["notes"][0]["note"]
            .as_str()
            .unwrap()
            .contains("1234"),
        "{bumped}"
    );
    let after = api.call("changes.list", json!({})).unwrap();
    assert_eq!(
        after.as_array().unwrap().len(),
        before.as_array().unwrap().len() + 2
    );
    assert_eq!(after[0]["author"]["kind"], "ai");
    assert!(!crate::changes::UNTRACKED_TABLES.contains(&"dev_feedback"));
    assert!(!crate::changes::UNTRACKED_TABLES.contains(&"dev_feedback_note"));
    // The bump is undone through the history.
    api.call("changes.undo", json!({"id": after[0]["id"]}))
        .unwrap();
    let f = api.call("devfeedback.get", json!({"id": id})).unwrap();
    assert_eq!(f["feedback"]["importance"], 1);
    assert_eq!(f["feedback"]["notes"], json!([]));
}

fn status(api: &Api, id: &Value, status: &str) -> Value {
    api.call(
        "devfeedback.set_status",
        json!({"id": id, "status": status}),
    )
    .unwrap()
}

fn column_ids(api: &Api, status: &str) -> Vec<Value> {
    api.call("devfeedback.list", json!({"status": status}))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["id"].clone())
        .collect()
}

/// Board: drag between columns (with reason and position), reorder within a column without
/// touching the bumps, undo the move, close and reopen.
#[test]
fn board_moves_reorders_and_undoes() {
    let (_d, api) = setup();
    let a = add(&api, &Actor::User, "Tela de carteira lenta")["feedback"]["id"].clone();
    let b = add(
        &api,
        &Actor::User,
        "Descrição confusa de query_transactions",
    )["feedback"]["id"]
        .clone();
    let c = add(&api, &Actor::User, "Falta exportar orçamento em CSV")["feedback"]["id"].clone();
    api.call("devfeedback.bump", json!({"id": c})).unwrap();
    assert_eq!(column_ids(&api, "aberto")[0], c);

    // Manual order: a goes to the top, even with less importance.
    let r = api
        .call("devfeedback.set_priority", json!({"id": a, "position": 1}))
        .unwrap();
    assert_eq!(r["position"], 1);
    assert_eq!(r["importance"], 1);
    assert!(r["change_set_id"].is_string());
    assert_eq!(
        column_ids(&api, "aberto"),
        vec![a.clone(), c.clone(), b.clone()]
    );
    // A position past the end goes to the end.
    api.call("devfeedback.set_priority", json!({"id": a, "position": 99}))
        .unwrap();
    assert_eq!(
        column_ids(&api, "aberto"),
        vec![c.clone(), b.clone(), a.clone()]
    );
    assert!(
        api.call("devfeedback.set_priority", json!({"id": a}))
            .is_err()
    );
    assert!(
        api.call("devfeedback.set_priority", json!({"id": a, "position": 0}))
            .is_err()
    );

    // Drag to Em andamento, with the reason, and undo.
    let moved = api
        .call(
            "devfeedback.set_status",
            json!({"id": b, "status": "em_andamento", "reason": "começando", "position": 1}),
        )
        .unwrap();
    assert_eq!(moved["status"], "em_andamento");
    assert_eq!(moved["position"], 1);
    let note = moved["notes"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(note["kind"], "status");
    assert_eq!(note["note"], "Em andamento: começando");
    api.call("changes.undo", json!({"id": moved["change_set_id"]}))
        .unwrap();
    let back = api.call("devfeedback.get", json!({"id": b})).unwrap();
    assert_eq!(back["feedback"]["status"], "aberto");
    assert_eq!(back["feedback"]["notes"], json!([]));
    // The item's history shows the move (undone) and the column repositioning.
    assert!(!back["history"].as_array().unwrap().is_empty());

    // Closing sets the date; reopening clears it; the `closed_since` filter finds the
    // just-closed one.
    let before = crate::time::now_utc();
    let done = status(&api, &c, "resolvido");
    assert!(done["closed_at"].is_string());
    assert_eq!(done["position"], Value::Null);
    let since = api
        .call("devfeedback.list", json!({"closed_since": before}))
        .unwrap();
    assert_eq!(since.as_array().unwrap().len(), 1);
    let reopened = status(&api, &c, "aberto");
    assert_eq!(reopened["closed_at"], Value::Null);
    // Filters by importance.
    let hi = api
        .call("devfeedback.list", json!({"min_importance": 2}))
        .unwrap();
    assert_eq!(hi.as_array().unwrap().len(), 1);
    assert_eq!(hi[0]["id"], c);
}

/// Structured fields: the agent fills them in when registering (no sensitive data) and the
/// user edits; empty text clears.
#[test]
fn structured_fields_are_saved_scrubbed_and_editable() {
    let (_d, api) = setup();
    dev_on(&api);
    let r = api
        .call_as(
            &agent(),
            "devfeedback.add",
            json!({
                "title": "list_subscriptions conta parcelas",
                "body": "Parcelas de empréstimo entram como assinatura.",
                "area": "lancamentos",
                "kind": "bug",
                "proposal": "Excluir parcelas ligadas a empréstimos.",
                "acceptance_criteria": "Teste com parcela + streaming.",
                "code_pointers": ["crates/treasury-core/src/subscriptions/mod.rs:40", " ", "crates/treasury-core/src/subscriptions/mod.rs:40"],
                "labels": ["assinaturas"],
                "assignee": "cli-teste",
                "expected": "Só streaming",
                "actual": "Streaming + parcela; senha: 9876",
                "user_impact": "alto",
                "confirmed_by_user": true,
                "tool_calls": [{"tool": "list_subscriptions", "outcome": "3 itens"}],
            }),
        )
        .unwrap();
    let f = &r["feedback"];
    assert_eq!(f["proposal"], "Excluir parcelas ligadas a empréstimos.");
    assert_eq!(
        f["code_pointers"],
        json!(["crates/treasury-core/src/subscriptions/mod.rs:40"])
    );
    assert_eq!(f["labels"], json!(["assinaturas"]));
    assert_eq!(f["assignee"], "cli-teste");
    assert_eq!(f["user_impact"], "alto");
    assert_eq!(f["confirmed_by_user"], true);
    assert_eq!(f["tool_calls"][0]["tool"], "list_subscriptions");
    assert!(!f["actual"].as_str().unwrap().contains("9876"), "{f}");
    assert_eq!(f["app_version"], crate::config::app_version());
    let id = f["id"].clone();

    let u = api
        .call(
            "devfeedback.update",
            json!({"id": id, "proposal": "", "labels": ["assinaturas", "dívidas"], "assignee": "usuario"}),
        )
        .unwrap();
    assert_eq!(u["proposal"], Value::Null);
    assert_eq!(u["labels"], json!(["assinaturas", "dívidas"]));
    assert_eq!(u["acceptance_criteria"], "Teste com parcela + streaming.");
    let by_label = api
        .call("devfeedback.list", json!({"label": "Dividas"}))
        .unwrap();
    assert_eq!(by_label.as_array().unwrap().len(), 1);
    let by_assignee = api
        .call("devfeedback.list", json!({"assignee": "cli-teste"}))
        .unwrap();
    assert_eq!(by_assignee, json!([]));
    let too_many: Vec<String> = (0..30).map(|i| format!("arquivo{i}.rs")).collect();
    assert!(
        api.call(
            "devfeedback.update",
            json!({"id": id, "code_pointers": too_many})
        )
        .is_err()
    );
}

/// Links: related in both directions; duplicate merges the bumps into the main item and
/// discards the duplicate; undo restores everything.
#[test]
fn links_and_duplicate_merge() {
    let (_d, api) = setup();
    let main =
        add(&api, &Actor::User, "Importação do OFX do banco falha")["feedback"]["id"].clone();
    let dup = add(&api, &Actor::User, "Erro na conciliação de cartões")["feedback"]["id"].clone();
    let other = add(&api, &Actor::User, "Tela de metas sem gráfico")["feedback"]["id"].clone();
    api.call("devfeedback.bump", json!({"id": dup, "note": "de novo"}))
        .unwrap();

    let rel = api
        .call(
            "devfeedback.link",
            json!({"id": other, "related_id": main, "kind": "relacionado"}),
        )
        .unwrap();
    assert_eq!(rel["merged"], false);
    assert_eq!(rel["feedback"]["links"][0]["related_id"], main);
    assert_eq!(rel["feedback"]["links"][0]["incoming"], false);
    assert_eq!(rel["related"]["links"][0]["incoming"], true);
    assert!(
        api.call(
            "devfeedback.link",
            json!({"id": main, "related_id": main, "kind": "relacionado"})
        )
        .is_err()
    );

    let m = api
        .call(
            "devfeedback.link",
            json!({"id": dup, "related_id": main, "kind": "duplicado"}),
        )
        .unwrap();
    assert_eq!(m["merged"], true);
    assert_eq!(m["feedback"]["status"], "descartado");
    let principal = &m["related"];
    // 1 (the main one) + 2 (the duplicate's importance).
    assert_eq!(principal["importance"], 3);
    assert_eq!(principal["bumps"], 2);
    let kinds: Vec<&str> = principal["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, vec!["reforco", "mescla"]);
    assert_eq!(m["feedback"]["links"][0]["kind"], "duplicado");

    api.call("changes.undo", json!({"id": m["change_set_id"]}))
        .unwrap();
    let back = api.call("devfeedback.get", json!({"id": main})).unwrap();
    assert_eq!(back["feedback"]["importance"], 1);
    assert_eq!(back["feedback"]["links"].as_array().unwrap().len(), 1);
    let d = api.call("devfeedback.get", json!({"id": dup})).unwrap();
    assert_eq!(d["feedback"]["status"], "aberto");
    assert_eq!(d["feedback"]["bumps"], 1);

    api.call(
        "devfeedback.unlink",
        json!({"id": main, "related_id": other}),
    )
    .unwrap();
    assert!(
        api.call(
            "devfeedback.unlink",
            json!({"id": main, "related_id": other})
        )
        .is_err()
    );
    // Deleting takes the links with it.
    api.call(
        "devfeedback.link",
        json!({"id": other, "related_id": main, "kind": "depende_de"}),
    )
    .unwrap();
    api.call("devfeedback.delete", json!({"id": main})).unwrap();
    let o = api.call("devfeedback.get", json!({"id": other})).unwrap();
    assert_eq!(o["feedback"]["links"], json!([]));
}

#[test]
fn agents_manage_the_board_but_delete_and_discard_are_proposals() {
    let (_d, api) = setup();
    dev_on(&api);
    let id = add(&api, &agent(), "Bug na importação")["feedback"]["id"].clone();
    let moved = api
        .call_as(
            &agent(),
            "devfeedback.set_status",
            json!({"id": id, "status": "em_andamento"}),
        )
        .unwrap();
    assert_eq!(moved["status"], "em_andamento");
    let edited = api
        .call_as(
            &agent(),
            "devfeedback.update",
            json!({"id": id, "title": "Outro título"}),
        )
        .unwrap();
    assert_eq!(edited["title"], "Outro título");
    for (cmd, args) in [
        (
            "devfeedback.set_status",
            json!({"id": id, "status": "descartado"}),
        ),
        ("devfeedback.delete", json!({"id": id})),
    ] {
        let r = api.call_as(&agent(), cmd, args).unwrap();
        assert_eq!(r["status"], "pending_approval", "{cmd}: {r}");
    }
    // Writing the Markdown to a file remains interface-only.
    assert!(matches!(
        api.call_as(
            &agent(),
            "devfeedback.export",
            json!({"path": "/tmp/x.md", "text": "x"})
        ),
        Err(crate::Error::Forbidden(_))
    ));
    api.call(
        "devfeedback.set_status",
        json!({"id": id, "status": "aberto"}),
    )
    .unwrap();
    let s = api
        .call(
            "devfeedback.set_status",
            json!({"id": id, "status": "em_andamento"}),
        )
        .unwrap();
    assert_eq!(s["status"], "em_andamento");
    let u = api
        .call(
            "devfeedback.update",
            json!({"id": id, "title": "  Bug na prévia da importação  "}),
        )
        .unwrap();
    assert_eq!(u["title"], "Bug na prévia da importação");
    assert!(
        api.call("devfeedback.update", json!({"id": id, "importance": 0}))
            .is_err()
    );
    assert!(
        api.call("devfeedback.update", json!({"id": id, "title": " "}))
            .is_err()
    );
    api.call("devfeedback.bump", json!({"id": id})).unwrap();
    api.call("devfeedback.delete", json!({"id": id})).unwrap();
    assert!(matches!(
        api.call("devfeedback.get", json!({"id": id})),
        Err(crate::Error::NotFound(_))
    ));
    assert_eq!(api.call("devfeedback.list", json!({})).unwrap(), json!([]));
}

/// Export: the core writes the Markdown to the chosen path (the app's WebKit does not download
/// a Blob link); interface only; inside the vault folder it is not allowed.
#[test]
fn export_writes_the_markdown_where_the_user_chose() {
    let (dir, api) = setup();
    let out = dir.path().join("feedback.md");
    let r = api
        .call(
            "devfeedback.export",
            json!({"path": out.display().to_string(), "text": "# Feedback\n\n- um item\n"}),
        )
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        "# Feedback\n\n- um item\n"
    );
    assert_eq!(r["bytes"], 22);
    dev_on(&api);
    let e = api
        .call_as(
            &agent(),
            "devfeedback.export",
            json!({"path": dir.path().join("x.md").display().to_string(), "text": "x"}),
        )
        .unwrap_err();
    assert!(matches!(e, crate::Error::Forbidden(_)), "{e}");
    let inside = dir.path().join(".t").join("x.md");
    assert!(
        api.call(
            "devfeedback.export",
            json!({"path": inside.display().to_string(), "text": "x"})
        )
        .is_err()
    );
}
