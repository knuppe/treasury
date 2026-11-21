//! Anexos pela API: preparação na memória, envio com a mensagem, motores que não leem
//! imagens (pular na rota, avisar, enviar sem), o que fica gravado em cada retenção, a
//! repetição no turno seguinte e as CLIs (Claude Code pela entrada `stream-json`, Codex
//! com `--image` em arquivo temporário apagado). Sem rede: servidores e CLIs falsos.

use base64::Engine as _;
use serde_json::{Value, json};

use super::tests_agent::{anthropic_text, engine, openai_text, route, setup, text_of, wait};
use crate::ai::attachments::tests::{pdf_with_lines, png};
use crate::ai::providers::fake::serve;

fn b64(b: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(b)
}

fn prepare(api: &crate::api::Api, name: &str, bytes: &[u8]) -> Value {
    api.call(
        "ai.attachments.prepare",
        json!({"name": name, "data": b64(bytes)}),
    )
    .unwrap()
}

fn anthropic(api: &crate::api::Api, url: &str) -> String {
    engine(
        api,
        json!({"provider": "anthropic", "endpoint": url, "api_key": "sk-teste-1234"}),
    )
}

fn count(api: &crate::api::Api, sql: &str) -> i64 {
    api.with_session(|s, _| Ok(s.db.conn().query_row(sql, [], |r| r.get(0))?))
        .unwrap()
}

#[test]
fn image_goes_natively_and_only_metadata_is_saved_by_default() {
    let t = setup();
    let srv = serve(vec![
        (200, "text/event-stream", anthropic_text("Vejo um recibo.")),
        (
            200,
            "text/event-stream",
            anthropic_text("Não tenho mais a imagem."),
        ),
    ]);
    let id = anthropic(&t.api, &srv.url);
    route(&t.api, "chat", &[&id]);
    let img = png(640, 480);
    let info = prepare(&t.api, "recibo.png", &img);
    assert_eq!(info["kind"], "image");
    assert_eq!(info["width"], 640);
    assert_eq!(info["stored"], false);
    assert_eq!(t.api.ai_runtime().attachments.len(), 1);

    // Só anexo, sem texto: vale (a conversa ganha o nome do arquivo).
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "", "attachment_ids": [info["id"]]}),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done", "{events:?}");
    assert_eq!(text_of(&events), "Vejo um recibo.");
    // Saiu da memória ao começar o job.
    assert!(t.api.ai_runtime().attachments.is_empty());
    let req = srv.requests.lock().unwrap()[0].body.clone();
    let content = req["messages"][0]["content"].as_array().unwrap().clone();
    let image = content.iter().find(|c| c["type"] == "image").unwrap();
    assert_eq!(image["source"]["data"], b64(&img));
    assert!(
        content.last().unwrap()["text"]
            .as_str()
            .unwrap()
            .contains("without text")
    );

    // Gravado: só os metadados (nome, tipo, tamanho, sha-256).
    let conv = t
        .api
        .call("ai.conversations.get", json!({"id": s["conversation_id"]}))
        .unwrap();
    assert_eq!(conv["title"], "Anexo: recibo.png");
    let saved = &conv["messages"][0]["attachments"][0];
    assert_eq!(saved["name"], "recibo.png");
    assert_eq!(saved["size"], img.len());
    assert_eq!(saved["stored"], false);
    assert_eq!(
        count(
            &t.api,
            "SELECT count(*) FROM ai_attachment WHERE data IS NULL AND text IS NULL"
        ),
        1
    );
    let st = t.api.call("ai.history.stats", json!({})).unwrap();
    assert_eq!(st["attachments"], 1);
    assert_eq!(st["stored_attachments"], 0);

    // Turno seguinte, com o cofre aberto: a imagem continua na memória da conversa e volta
    // ao motor sem ser anexada de novo (nada foi gravado no cofre).
    let s2 = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "E o total?", "conversation_id": s["conversation_id"]}),
        )
        .unwrap();
    wait(&t.api, s2["job_id"].as_str().unwrap());
    let req = srv.requests.lock().unwrap()[1].body.clone();
    assert!(req.to_string().contains("\"type\":\"image\""), "{req}");
    assert_eq!(
        req["messages"][0]["content"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["text"],
        "(mensagem sem texto, só com anexos)"
    );
    assert_eq!(
        count(
            &t.api,
            "SELECT count(*) FROM ai_attachment WHERE data IS NULL AND text IS NULL"
        ),
        1
    );

    // Depois de bloquear o cofre (a memória é esvaziada), no lugar da imagem vai o aviso
    // "não guardado".
    t.api.ai_runtime().cancel_all();
    let srv2 = serve(vec![(
        200,
        "text/event-stream",
        anthropic_text("Sem a imagem."),
    )]);
    let id2 = anthropic(&t.api, &srv2.url);
    route(&t.api, "chat", &[&id2]);
    let s3 = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "E agora?", "conversation_id": s["conversation_id"]}),
        )
        .unwrap();
    wait(&t.api, s3["job_id"].as_str().unwrap());
    let req = srv2.requests.lock().unwrap()[0].body.clone();
    let first = req["messages"][0]["content"].to_string();
    assert!(
        first.contains("attachment sent earlier: recibo.png"),
        "{first}"
    );
    assert!(!req.to_string().contains("\"type\":\"image\""));
}

#[test]
fn stored_attachments_are_resent_in_the_next_turn() {
    let t = setup();
    t.api
        .call("ai.settings.set", json!({"attachment_retention": "always"}))
        .unwrap();
    let srv = serve(vec![
        (200, "text/event-stream", anthropic_text("ok")),
        (200, "text/event-stream", anthropic_text("ok de novo")),
    ]);
    let id = anthropic(&t.api, &srv.url);
    route(&t.api, "chat", &[&id]);
    let pdf = pdf_with_lines(&["Fatura total 321,00"]);
    let info = prepare(&t.api, "fatura.pdf", &pdf);
    assert_eq!(info["pages"], 1);
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Leia", "attachment_ids": [info["id"]]}),
        )
        .unwrap();
    wait(&t.api, s["job_id"].as_str().unwrap());
    let s2 = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Qual o total?", "conversation_id": s["conversation_id"]}),
        )
        .unwrap();
    wait(&t.api, s2["job_id"].as_str().unwrap());
    let reqs = srv.requests.lock().unwrap();
    for r in reqs.iter() {
        let doc = r.body["messages"][0]["content"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["type"] == "document")
            .cloned()
            .expect("PDF nativo nos dois turnos");
        assert_eq!(doc["source"]["data"], b64(&pdf));
    }
    let st = t.api.call("ai.history.stats", json!({})).unwrap();
    assert_eq!(st["stored_attachments"], 1);
    assert!(st["attachment_bytes"].as_i64().unwrap() >= pdf.len() as i64);
    // A interface consegue abrir o anexo guardado.
    let aid = t
        .api
        .call("ai.conversations.get", json!({"id": s["conversation_id"]}))
        .unwrap()["messages"][0]["attachments"][0]["id"]
        .clone();
    let data = t
        .api
        .call("ai.attachments.get", json!({"id": aid}))
        .unwrap();
    assert_eq!(data["mime"], "application/pdf");
    assert_eq!(data["data"], b64(&pdf));
}

#[test]
fn engines_that_cannot_read_images_are_skipped_or_warned() {
    let t = setup();
    // Rota: modelo local só de texto, depois Anthropic (lê imagens).
    let local = serve(vec![(200, "text/event-stream", openai_text("texto"))]);
    let text_only = engine(
        &t.api,
        json!({"provider": "openai_compatible", "endpoint": local.url, "model": "llama3.1:8b"}),
    );
    let srv = serve(vec![(
        200,
        "text/event-stream",
        anthropic_text("vi a foto"),
    )]);
    let vision = anthropic(&t.api, &srv.url);
    let list = t.api.call("ai.engines.list", json!({})).unwrap();
    assert_eq!(list[0]["caps"]["images"], false);
    assert_eq!(list[1]["caps"]["images"], true);
    route(&t.api, "chat", &[&text_only, &vision]);

    let info = prepare(&t.api, "foto.png", &png(10, 10));
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Veja", "attachment_ids": [info["id"]]}),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done", "{events:?}");
    let fb = events
        .iter()
        .find(|e| e["event"]["type"] == "fallback")
        .unwrap();
    assert!(
        fb["event"]["reason"]
            .as_str()
            .unwrap()
            .contains("não lê imagens")
    );
    assert!(
        local.requests.lock().unwrap().is_empty(),
        "nada foi ao motor sem visão"
    );
    assert_eq!(text_of(&events), "vi a foto");

    // Só o motor sem visão: falha clara...
    route(&t.api, "chat", &[&text_only]);
    let info = prepare(&t.api, "foto.png", &png(10, 10));
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Veja", "attachment_ids": [info["id"]]}),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "failed");
    let err = events
        .iter()
        .find(|e| e["event"]["type"] == "error")
        .unwrap();
    assert_eq!(err["event"]["code"], "attachment_unsupported");
    // ...e o usuário pode escolher enviar sem a imagem (vai um aviso no lugar).
    let info = prepare(&t.api, "foto.png", &png(10, 10));
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Veja", "attachment_ids": [info["id"]], "skip_unsupported": true}),
        )
        .unwrap();
    let (status, _) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done");
    let body = local.requests.lock().unwrap()[0].body.clone();
    let user = body["messages"][1]["content"].as_str().unwrap();
    assert!(
        user.contains("foto.png") && user.contains("does not read images"),
        "{user}"
    );

    // O ajuste do motor muda o que ele lê.
    let e = t
        .api
        .call(
            "ai.engines.update",
            json!({"id": text_only, "vision": true}),
        )
        .unwrap();
    assert_eq!(e["caps"]["images"], true);
    assert_eq!(e["vision"], true);
    let e = t
        .api
        .call(
            "ai.engines.update",
            json!({"id": text_only, "vision": null}),
        )
        .unwrap();
    assert_eq!(e["vision"], Value::Null);
    assert_eq!(e["caps"]["images"], false);
}

#[test]
fn staged_attachments_expire_with_the_vault_and_can_be_discarded() {
    let t = setup();
    let a = prepare(&t.api, "a.txt", b"um");
    let b = prepare(&t.api, "b.csv", b"x;y\n1;2\n");
    assert_eq!(b["kind"], "text");
    t.api
        .call("ai.attachments.discard", json!({"ids": [a["id"]]}))
        .unwrap();
    assert_eq!(t.api.ai_runtime().attachments.len(), 1);
    // Anexo descartado ou inexistente: o envio recusa, sem gravar nada.
    let e = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "x", "attachment_ids": [a["id"]]}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "not_found");
    let bad = t.api.call(
        "ai.attachments.prepare",
        json!({"name": "virus.exe", "data": b64(b"MZ\0\0")}),
    );
    assert!(bad.unwrap_err().to_string().contains("não aceito"));
    t.api.call("vault.lock", Value::Null).unwrap();
    assert!(
        t.api.ai_runtime().attachments.is_empty(),
        "somem ao bloquear"
    );
}

#[cfg(unix)]
#[test]
fn claude_code_gets_images_through_stream_json_stdin() {
    use super::tests_agent::script;
    let t = setup();
    let out = tempfile::tempdir().unwrap();
    let o = out.path().to_string_lossy().into_owned();
    let bin = script(
        out.path(),
        "fake-claude",
        &format!(
            r#"for a in "$@"; do printf '[%s]\n' "$a"; done > "{o}/args.txt"
cat > "{o}/stdin.txt"
echo '{{"type":"assistant","message":{{"content":[{{"type":"text","text":"Imagem recebida."}}]}}}}'
echo '{{"type":"result","subtype":"success","is_error":false,"result":"Imagem recebida.","usage":{{"input_tokens":1700,"output_tokens":3}}}}'"#
        ),
    );
    t.api
        .ai_runtime()
        .set_mcp_command(Some("/opt/treasury-teste".into()));
    let id = engine(
        &t.api,
        json!({"provider": "claude_code", "command_template": format!("{bin} -p {{prompt}} --mcp-config {{mcp_config}} --output-format stream-json --verbose")}),
    );
    route(&t.api, "chat", &[&id]);
    let img = png(32, 32);
    let pic = prepare(&t.api, "tela.png", &img);
    let note = prepare(&t.api, "nota.md", b"# Lembrete\npagar IPVA");
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "O que é isso?", "attachment_ids": [pic["id"], note["id"]]}),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done", "{events:?}");
    let args = std::fs::read_to_string(out.path().join("args.txt")).unwrap();
    assert!(args.contains("[--input-format]\n[stream-json]\n"), "{args}");
    assert!(
        !args.contains("O que é isso?"),
        "o prompt saiu da linha de comando"
    );
    let stdin = std::fs::read_to_string(out.path().join("stdin.txt")).unwrap();
    let msg: Value = serde_json::from_str(stdin.trim()).unwrap();
    assert_eq!(msg["type"], "user");
    let content = msg["message"]["content"].as_array().unwrap();
    assert_eq!(content[0]["type"], "image");
    assert_eq!(content[0]["source"]["data"], b64(&img));
    let prompt = content.last().unwrap()["text"].as_str().unwrap();
    assert!(prompt.contains("O que é isso?") && prompt.contains("pagar IPVA"));
    assert!(prompt.contains("<anexo-") && prompt.contains("name=\"nota.md\""));
}

#[cfg(unix)]
#[test]
fn codex_gets_images_as_private_temp_files_removed_after() {
    use super::tests_agent::script;
    let t = setup();
    let out = tempfile::tempdir().unwrap();
    let o = out.path().to_string_lossy().into_owned();
    let bin = script(
        out.path(),
        "fake-codex",
        &format!(
            r#"for a in "$@"; do printf '[%s]\n' "$a"; done > "{o}/args.txt"
for a in "$@"; do case "$a" in --image=*) f="${{a#--image=}}"; ls -l "$f" > "{o}/perm.txt"; cp "$f" "{o}/img.bin"; echo "$f" > "{o}/path.txt";; esac; done
echo "Vi a imagem.""#
        ),
    );
    let id = engine(
        &t.api,
        json!({"provider": "codex", "command_template": format!("{bin} exec -c mcp_servers.treasury={{mcp_config}} {{prompt}}")}),
    );
    route(&t.api, "chat", &[&id]);
    let img = png(16, 16);
    let pic = prepare(&t.api, "foto.png", &img);
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Veja", "attachment_ids": [pic["id"]]}),
        )
        .unwrap();
    let (status, events) = wait(&t.api, s["job_id"].as_str().unwrap());
    assert_eq!(status, "done", "{events:?}");
    let args = std::fs::read_to_string(out.path().join("args.txt")).unwrap();
    // Logo depois de `exec`, com `=` (o argumento não engole o prompt).
    assert!(args.contains("[exec]\n[--image="), "{args}");
    assert_eq!(std::fs::read(out.path().join("img.bin")).unwrap(), img);
    let perm = std::fs::read_to_string(out.path().join("perm.txt")).unwrap();
    assert!(perm.starts_with("-rw-------"), "{perm}");
    let path = std::fs::read_to_string(out.path().join("path.txt")).unwrap();
    assert!(
        path.contains("/run/agents/"),
        "no diretório privado do app: {path}"
    );
    assert!(
        !std::path::Path::new(path.trim()).exists(),
        "apagado depois"
    );
}

#[test]
fn the_agent_reads_an_earlier_attachment_without_it_being_stored() {
    let t = setup();
    let srv = serve(vec![(
        200,
        "text/event-stream",
        anthropic_text("Li o extrato."),
    )]);
    let id = anthropic(&t.api, &srv.url);
    route(&t.api, "chat", &[&id]);
    let pdf = pdf_with_lines(&["Saldo final 9.876,54"]);
    let info = prepare(&t.api, "extrato.pdf", &pdf);
    let s = t
        .api
        .call(
            "ai.chat_start",
            json!({"message": "Leia", "attachment_ids": [info["id"]]}),
        )
        .unwrap();
    wait(&t.api, s["job_id"].as_str().unwrap());
    let conv = s["conversation_id"].as_str().unwrap();
    let actor = crate::api::Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let call = |name: &str, args: Value, c: Option<&str>| {
        crate::mcp::inproc::call_tool_in(
            &t.api,
            &actor,
            name,
            args,
            crate::mcp::inproc::AGENT_ACCESS,
            c,
        )
    };
    // O texto vem da memória da conversa (nada guardado no cofre).
    let o = call("read_attachment", json!({"id": "extrato.pdf"}), Some(conv));
    assert!(!o.is_error, "{}", o.content);
    assert!(o.content.contains("Saldo final 9.876,54"), "{}", o.content);
    // A conversa vem do job, não do modelo.
    let o = call(
        "read_attachment",
        json!({"id": "extrato.pdf", "conversation_id": "outra"}),
        Some(conv),
    );
    assert!(o.is_error, "parâmetro desconhecido: {}", o.content);
    let o = call("read_chat_history", json!({}), Some(conv));
    assert!(o.content.contains("Li o extrato."), "{}", o.content);
    // Fora de uma conversa: erro claro.
    let o = call("read_chat_history", json!({}), None);
    assert!(o.is_error);
    // Depois de bloquear o cofre, o conteúdo some (os anexos não ficam guardados).
    t.api.ai_runtime().cancel_all();
    let o = call("read_attachment", json!({"id": "extrato.pdf"}), Some(conv));
    assert!(
        o.is_error && o.content.contains("anexar de novo"),
        "{}",
        o.content
    );
}
