//! Estudo com IA de ponta a ponta com um motor falso: o aviso, o job, a resposta no esquema,
//! a conferência dos números, o histórico e apagar.

use serde_json::{Value, json};

use super::tests_agent::{engine, openai_text, route, setup, wait};
use crate::ai::providers::fake::serve;

#[test]
fn study_end_to_end() {
    let t = setup();
    let id = t
        .api
        .call(
            "assets.create",
            json!({"ticker": "TAEE11", "name": "Taesa", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let reply = json!({
        "summary": "Transmissora de energia com receita regulada.",
        "strengths": [{"text": "Receita previsível", "data": "record.sector"}],
        "risks": [{"text": "Juros altos", "data": null}],
        "key_numbers": [
            {"label": "Inventado", "value": 42, "ref": "score.nao_existe"}
        ],
        "manual_checks": [{"item": "Dívida líquida", "where": "DFP 2025 na CVM"}],
        "sources": [{"name": "CVM", "date": null}],
        "confidence": "low"
    });
    let srv = serve(vec![(
        200,
        "text/event-stream",
        openai_text(&format!("```json\n{reply}\n```")),
    )]);
    let e = engine(
        &t.api,
        json!({"provider": "openai", "endpoint": srv.url, "api_key": "sk-teste-1234"}),
    );
    route(&t.api, "study", &[&e]);
    t.api.call("assets.study_accept", json!({})).unwrap();
    let st = t
        .api
        .call("assets.study_state", json!({"asset_id": id}))
        .unwrap();
    assert_eq!(st["engine"]["id"], e.as_str());
    let s = t
        .api
        .call(
            "assets.study_start",
            json!({"asset_id": id, "question": "O que mudou?"}),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done", "{events:?}");
    // A lista de tarefas leva ao estudo gravado.
    let jobs = t.api.call("ai.jobs.list", Value::Null).unwrap();
    let link = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["job_id"] == s["job_id"])
        .unwrap()["link"]
        .clone();
    assert_eq!(link["screen"], "asset_study");
    assert_eq!(link["asset_id"], id.as_str());
    // O pedido levou o pacote delimitado e nada da memória ou do nome da família.
    let body = srv.requests.lock().unwrap()[0].body.to_string();
    assert!(body.contains("pacote_do_estudo"), "{body}");
    assert!(body.contains("O que mudou?"));
    assert!(!body.contains("Household: Família"), "{body}");
    let list = t
        .api
        .call("assets.study_list", json!({"asset_id": id}))
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["unchecked"], 1);
    assert_eq!(list[0]["confidence"], "low");
    let full = t
        .api
        .call("assets.study_get", json!({"id": list[0]["id"]}))
        .unwrap();
    assert_eq!(full["checks"][0]["status"], "not_found");
    assert_eq!(
        full["output"]["manual_checks"][0]["where"],
        "DFP 2025 na CVM"
    );
    assert_eq!(full["disclaimer_version"], "1");
    assert_eq!(link["id"], list[0]["id"]);
    // Apagar (só a interface) e desfazer.
    t.api
        .call("assets.study_delete", json!({"id": list[0]["id"]}))
        .unwrap();
    let after: Value = t
        .api
        .call("assets.study_list", json!({"asset_id": id}))
        .unwrap();
    assert!(after.as_array().unwrap().is_empty());
}

/// O agente pede um estudo pelo chat (`start_asset_study`): vira proposta; aprovada, o app
/// dispara o estudo, que fica gravado no ativo (docs/asset-page.md).
#[test]
fn study_requested_by_the_agent_runs_only_after_approval() {
    let t = setup();
    let id = t
        .api
        .call(
            "assets.create",
            json!({"ticker": "TAEE11", "name": "Taesa", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let reply = json!({"summary": "Transmissora com receita regulada.", "confidence": "medium"});
    let srv = serve(vec![(
        200,
        "text/event-stream",
        openai_text(&reply.to_string()),
    )]);
    let e = engine(
        &t.api,
        json!({"provider": "openai", "endpoint": srv.url, "api_key": "sk-teste-1234"}),
    );
    route(&t.api, "study", &[&e]);
    let ai = crate::api::Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    // Sem o aviso aceito, o pedido é recusado na hora (não fica uma proposta que falharia).
    let refused = t
        .api
        .call_as(&ai, "assets.study_request", json!({"asset_id": id}));
    assert!(refused.is_err());
    t.api.call("assets.study_accept", json!({})).unwrap();
    let r = t
        .api
        .call_as(
            &ai,
            "assets.study_request",
            json!({"asset_id": id, "question": "O que mudou?"}),
        )
        .unwrap();
    assert_eq!(r["status"], "pending_approval");
    let proposal_id = r["proposal_id"].as_str().unwrap().to_string();
    // Nada roda antes da aprovação.
    assert!(srv.requests.lock().unwrap().is_empty());
    let studies = |api: &crate::api::Api| {
        api.call("assets.study_list", json!({"asset_id": id}))
            .unwrap()
            .as_array()
            .unwrap()
            .len()
    };
    assert_eq!(studies(&t.api), 0);
    t.api
        .call("ai.approve", json!({"ids": [proposal_id]}))
        .unwrap();
    let job = t
        .api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT job_id, question FROM asset_study_request WHERE asset_id = ?1",
                [&id],
                |r| {
                    Ok((
                        r.get::<_, Option<String>>(0)?,
                        r.get::<_, Option<String>>(1)?,
                    ))
                },
            )?)
        })
        .unwrap();
    assert_eq!(job.1.as_deref(), Some("O que mudou?"));
    let (status, events) = wait(&t.api, job.0.as_deref().expect("estudo disparado"));
    assert_eq!(status, "done", "{events:?}");
    assert_eq!(studies(&t.api), 1);
}

#[test]
fn price_history_tool_summarizes_and_thins_the_points() {
    let t = setup();
    let id = t
        .api
        .call(
            "assets.create",
            json!({"ticker": "TAEE11", "name": "Taesa", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    for (d, c) in [
        ("2026-01-05", "30.00"),
        ("2026-01-06", "31.00"),
        ("2026-02-02", "33.00"),
        ("2026-06-01", "36.00"),
        ("2026-10-01", "34.50"),
    ] {
        t.api
            .call(
                "market.set_price",
                json!({"asset_id": id, "date": d, "close": c}),
            )
            .unwrap();
    }
    let raw = t
        .api
        .call(
            "market.prices",
            json!({"asset_id": id, "from": "2026-01-01", "to": "2026-10-31"}),
        )
        .unwrap();
    let points = raw.as_array().unwrap().clone();
    assert_eq!(points.len(), 5);
    // Período de 9 meses: uma por semana (as duas de janeiro na mesma semana viram uma).
    let v = crate::mcp::inputs_research::summarize_prices(&id, &points, None);
    assert_eq!(v["interval"], "weekly");
    assert_eq!(v["count"], 4);
    assert_eq!(v["recorded"], 5);
    assert_eq!(v["first_close"], points[0]["close"]);
    assert_eq!(v["last_close"], points[4]["close"]);
    assert_eq!(v["change"], 0.15);
    assert_eq!(v["min_close"], 30.0);
    assert_eq!(v["max_close"], 36.0);
    let monthly = crate::mcp::inputs_research::summarize_prices(
        &id,
        &points,
        Some(crate::mcp::inputs_research::PriceInterval::Monthly),
    );
    assert_eq!(monthly["count"], 4);
    assert_eq!(monthly["points"][0]["date"], "2026-01-06");
}
