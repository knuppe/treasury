//! E-mail de ingestão de ponta a ponta, sem rede: a pasta local (fase 1), a fila, aplicar
//! sozinho, os avisos do corpo, a retenção com o modo de teste e quem pode configurar.

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde_json::{Value, json};

use super::ingest::{self, Incoming, Outcome, Prepared};
use super::message::tests::eml;
use super::store::MailStatus;
use super::{config, retention, sync};
use crate::api::{Actor, Api, SharedApi};
use crate::paths::TreasuryPaths;

const CARTAO_OFX: &[u8] = include_bytes!("../import/fixtures/cartao_credito.ofx");

fn setup() -> (tempfile::TempDir, SharedApi) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap().into_shared();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    (dir, api)
}

/// Cartão com o final do OFX de teste (5555********4444).
fn card(api: &Api) -> String {
    let id = crate::ids::new_id();
    api.with_session(|s, _| {
        let c = s.db.conn();
        c.execute(
            "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
             VALUES (?1, 'Cartão Roxo', 'credit_card', 'BRL', '2026-01-01', 'x', 'x')",
            [&id],
        )?;
        c.execute(
            "INSERT INTO credit_card (account_id, closing_day, due_day, last4, created_at, updated_at)
             VALUES (?1, 25, 5, '4444', 'x', 'x')",
            [&id],
        )?;
        Ok(())
    })
    .unwrap();
    id
}

fn count(api: &Api, sql: &str) -> i64 {
    api.with_session(|s, _| Ok(s.db.conn().query_row(sql, [], |r| r.get::<_, i64>(0))?))
        .unwrap()
}

/// Arquivo "antigo" (a busca ignora o que acabou de ser mexido).
fn put(dir: &Path, name: &str, bytes: &[u8]) {
    std::fs::create_dir_all(dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, bytes).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(120))
        .unwrap();
}

fn folder_account(api: &Api, path: &Path) -> String {
    std::fs::create_dir_all(path).unwrap();
    api.call(
        "mail.accounts.save",
        json!({"name": "Pasta da família", "kind": "folder", "folder_path": path.to_string_lossy()}),
    )
    .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn rule(api: &Api, v: Value) -> String {
    api.call("mail.rules.save", v).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn messages(api: &Api) -> Vec<Value> {
    api.call("mail.list", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn local_folder_imports_reviews_and_skips_what_it_saw() {
    let (dir, api) = setup();
    let cartao = card(&api);
    let inbox = dir.path().join("Recebidos");
    put(&inbox, "fatura.ofx", CARTAO_OFX);
    // Mesmo conteúdo com outro nome: repetido.
    put(&inbox, "fatura (1).ofx", CARTAO_OFX);
    put(&inbox, "notas.txt", b"lista do mercado");
    put(&inbox.join("Bancos"), "outro.ofx", CARTAO_OFX);
    put(&inbox.join(".escondida"), "x.ofx", CARTAO_OFX);
    let acc = folder_account(&api, &inbox);
    // O arquivo solto que a regra pega vai sozinho (quem pôs na pasta foi o usuário).
    rule(
        &api,
        json!({"name": "Faturas OFX", "action": "auto",
               "conditions": {"advanced": {"type": "attachment", "names": ["*.ofx"]}}}),
    );
    let st = sync::sync_account(&api, &acc).unwrap();
    assert!(st.ok, "{st:?}");
    // Só a raiz é lida (Bancos/ não está nas pastas permitidas); a pasta oculta, nunca.
    assert_eq!((st.seen, st.passed, st.applied), (3, 2, 1), "{st:?}");
    let txns = count(
        &api,
        &format!("SELECT COUNT(*) FROM txn WHERE account_id = '{cartao}'"),
    );
    assert!(txns >= 3, "lançamentos da fatura: {txns}");
    // O histórico assina a importação com a regra.
    let by: String = api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT author_detail FROM change_set WHERE reason LIKE 'Importação de %' ORDER BY created_at DESC LIMIT 1",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert!(by.starts_with("E-mail: Faturas OFX"), "{by}");
    let list = messages(&api);
    // O primeiro dos dois iguais entra; o outro é repetido e nem vira mensagem.
    let ofx = list
        .iter()
        .find(|m| m["file_name"].as_str().is_some_and(|f| f.ends_with(".ofx")))
        .unwrap();
    assert_eq!(ofx["status"], "imported");
    assert_eq!(ofx["parts"][0]["kind"], "statement");
    // O .txt sem regra que o pegue cai no padrão da pasta (revisar) e não é reconhecido.
    let txt = list.iter().find(|m| m["file_name"] == "notas.txt").unwrap();
    assert_eq!(txt["status"], "unrecognized");
    // Segunda busca: nada de novo (o repetido e o visto não contam de novo).
    let st = sync::sync_account(&api, &acc).unwrap();
    assert_eq!((st.seen, st.passed), (0, 0), "{st:?}");
    // Liberar a subpasta: o OFX igual ao já importado entra como repetido.
    api.call(
        "mail.accounts.save",
        json!({"id": acc, "name": "Pasta da família", "allowed_folders": ["INBOX", "Bancos/*", "Bancos"]}),
    )
    .unwrap();
    let st = sync::sync_account(&api, &acc).unwrap();
    assert_eq!(st.seen, 1, "{st:?}");
    let audit = api.call("mail.audit", json!({"account_id": acc})).unwrap();
    assert!(
        audit
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["action"] == "sync")
    );
}

#[test]
fn eml_in_the_folder_goes_through_the_filters_and_the_body_layouts() {
    let (dir, api) = setup();
    let cartao = card(&api);
    let inbox = dir.path().join("Caixa");
    put(
        &inbox,
        "compra.eml",
        &eml(
            "avisos@nubank.com.br",
            "Compra aprovada",
            "Compra de R$ 45,90 APROVADA em PADARIA PAO QUENTE no cartão final 4444 em 18/10/2026.",
            None,
        ),
    );
    put(
        &inbox,
        "promo.eml",
        &eml("ofertas@loja.com", "Promoção imperdível", "Compre já", None),
    );
    let acc = folder_account(&api, &inbox);
    // Sem regra, o .eml segue o padrão da conta (pasta local: revisar); a promoção é barrada.
    api.call(
        "mail.accounts.save",
        json!({"id": acc, "name": "Caixa", "block_senders": ["loja.com"]}),
    )
    .unwrap();
    rule(
        &api,
        json!({"name": "Nubank", "action": "auto",
               "conditions": {"from": ["nubank.com.br"], "subject": {"contains": ["compra"]}}}),
    );
    let st = sync::sync_account(&api, &acc).unwrap();
    assert_eq!((st.seen, st.passed, st.rejected), (2, 1, 1), "{st:?}");
    let m = messages(&api).into_iter().next().unwrap();
    // .eml de pasta local não tem provedor que autentique: não aplica sozinho.
    assert_eq!(m["authenticated"], false);
    assert_eq!(m["trusted"], false);
    assert_eq!(m["status"], "awaiting_review");
    let part = &m["parts"][0];
    assert_eq!(part["kind"], "purchase");
    assert_eq!(part["detail"]["amount"], -4590);
    assert_eq!(part["detail"]["account_id"], cartao.as_str());
    // Na revisão, o usuário aplica: o lançamento entra no cartão.
    let r = api
        .call("mail.apply_part", json!({"part_id": part["id"]}))
        .unwrap();
    assert_eq!(r["status"], "imported", "{r}");
    assert_eq!(
        count(
            &api,
            &format!("SELECT COUNT(*) FROM txn WHERE account_id = '{cartao}' AND amount = -4590")
        ),
        1
    );
    // A promoção recusada ficou só na contagem (e não é lida de novo).
    assert_eq!(count(&api, "SELECT COUNT(*) FROM mail_seen"), 1);
}

#[test]
fn retention_waits_for_the_test_mode_and_the_confirmation() {
    let (dir, api) = setup();
    card(&api);
    let inbox = dir.path().join("Entrada");
    put(&inbox, "fatura.ofx", CARTAO_OFX);
    let acc = folder_account(&api, &inbox);
    rule(
        &api,
        json!({"name": "OFX", "action": "auto",
               "conditions": {"advanced": {"type": "attachment", "names": ["*.ofx"]}},
               "retention": {"mode": "after_success", "grace_days": 0}}),
    );
    let a = api.call("mail.accounts.get", json!({"id": acc})).unwrap();
    assert_eq!(a["retention_state"], "testing");
    sync::sync_account(&api, &acc).unwrap();
    // Modo de teste: o arquivo continua lá e a auditoria diz o que teria saído.
    assert!(inbox.join("fatura.ofx").exists());
    let would = api
        .call("mail.would_delete", json!({"account_id": acc}))
        .unwrap();
    assert_eq!(would.as_array().unwrap().len(), 1, "{would}");
    // Confirmar antes do fim do teste não vale.
    assert!(
        api.call("mail.accounts.confirm_retention", json!({"id": acc}))
            .is_err()
    );
    // Fim do teste: espera a confirmação (e continua sem apagar).
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE mail_account SET retention_test_until = '2020-01-01T00:00:00.000Z'",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    let a = api.call("mail.accounts.get", json!({"id": acc})).unwrap();
    assert_eq!(a["retention_state"], "awaiting_confirmation");
    sync::sync_account(&api, &acc).unwrap();
    assert!(inbox.join("fatura.ofx").exists());
    api.call("mail.accounts.confirm_retention", json!({"id": acc}))
        .unwrap();
    sync::sync_account(&api, &acc).unwrap();
    assert!(!inbox.join("fatura.ofx").exists());
    assert!(inbox.join("Processados").join("fatura.ofx").exists());
    let audit = api.call("mail.audit", json!({"account_id": acc})).unwrap();
    assert!(
        audit
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["action"] == "deleted")
    );
    // A pasta Processados nunca é lida.
    let st = sync::sync_account(&api, &acc).unwrap();
    assert_eq!(st.seen, 0);
}

#[test]
fn imap_account_trust_forwarding_and_credentials() {
    let (_dir, api) = setup();
    let acc = api
        .call(
            "mail.accounts.save",
            json!({"name": "Caixa da família", "kind": "imap", "provider": "gmail",
                   "username": "familia.treasury@gmail.com",
                   "family_addresses": ["ana@gmail.com"]}),
        )
        .unwrap();
    let id = acc["id"].as_str().unwrap().to_string();
    assert_eq!(acc["host"], "imap.gmail.com");
    assert_eq!(acc["authserv_id"], "mx.google.com");
    assert_eq!(acc["allowed_folders"], json!(["INBOX"]));
    assert_eq!(acc["default_action"], "ignore");
    // A senha nunca volta: só o "cadastrada" e a dica.
    api.call(
        "mail.accounts.set_password",
        json!({"id": id, "password": "abcd efgh ijkl mnop"}),
    )
    .unwrap();
    let got = api.call("mail.accounts.get", json!({"id": id})).unwrap();
    assert_eq!(got["password"]["set"], true);
    assert!(!got.to_string().contains("abcd efgh"));
    // Trocar o servidor apaga a senha.
    let moved = api
        .call(
            "mail.accounts.save",
            json!({"id": id, "name": "Caixa da família", "host": "imap.exemplo.com"}),
        )
        .unwrap();
    assert_eq!(moved["password"]["set"], false);
    api.call(
        "mail.accounts.save",
        json!({"id": id, "name": "Caixa da família", "host": "imap.gmail.com"}),
    )
    .unwrap();
    rule(
        &api,
        json!({"name": "Nubank", "action": "auto", "account_id": id,
               "conditions": {"from": ["nubank.com.br"]}}),
    );
    let inc = |raw: Vec<u8>| Incoming {
        folder: "INBOX".into(),
        folder_flags: vec![],
        uid: Some(1),
        uidvalidity: Some(7),
        file_name: None,
        raw,
        is_message: true,
        file_date: None,
    };
    let store = |raw: Vec<u8>| {
        api.with_session(|s, _| {
            let p = Prepared::load(s.db.conn(), &id)?;
            ingest::store_incoming(s.db.conn(), &p, inc(raw))
        })
        .unwrap()
    };
    // Direto do banco, autenticado pelo Gmail: confiável.
    let o = store(eml(
        "todomundo@nubank.com.br",
        "Sua fatura",
        "Segue a fatura.",
        Some(("fatura.ofx", "application/x-ofx", CARTAO_OFX)),
    ));
    let Outcome::Stored { auto, .. } = o else {
        panic!("{o:?}")
    };
    assert_eq!(
        auto.len(),
        1,
        "OFX de remetente autenticado na regra com nomes"
    );
    // A mesma mensagem de novo: já vista.
    let again = store(eml(
        "todomundo@nubank.com.br",
        "Sua fatura",
        "Segue a fatura.",
        Some(("fatura.ofx", "application/x-ofx", CARTAO_OFX)),
    ));
    assert_eq!(again, Outcome::Known);
    // Encaminhada por um membro (autenticado) com o original na regra: confiável.
    let fwd = eml(
        "ana@gmail.com",
        "Fwd: Fatura de outubro",
        "---------- Forwarded message ---------\r\nDe: Nubank <todomundo@nubank.com.br>\r\n\r\nSegue.",
        Some((
            "outubro.ofx",
            "application/x-ofx",
            b"OFXHEADER:100\r\n<OFX></OFX>",
        )),
    );
    let Outcome::Stored { message_id, .. } = store(fwd) else {
        panic!()
    };
    let m = api.call("mail.get", json!({"id": message_id})).unwrap();
    assert_eq!(m["original_sender"], "todomundo@nubank.com.br");
    assert_eq!(m["trusted"], true, "{m}");
    // Encaminhada por quem não é da família: não passa nem do filtro (o original casa a regra,
    // mas o encaminhador não é confiável) — fica, sem aplicar sozinho.
    let stranger = eml(
        "alguem@gmail.com",
        "Fwd: Fatura",
        "---------- Forwarded message ---------\r\nDe: Nubank <todomundo@nubank.com.br>\r\n\r\nSegue.",
        None,
    );
    if let Outcome::Stored {
        message_id, auto, ..
    } = store(stranger)
    {
        assert!(auto.is_empty());
        let m = api.call("mail.get", json!({"id": message_id})).unwrap();
        assert_eq!(m["trusted"], false);
    }
    // Remetente que a regra não cobre e sem padrão de revisão: recusado (só contagem).
    let other = store(eml("x@itau.com.br", "Fatura", "oi", None));
    assert!(matches!(other, Outcome::Rejected(_)), "{other:?}");
}

#[test]
fn only_the_app_configures_and_the_queue_is_readable() {
    let (dir, api) = setup();
    let acc = folder_account(&api, &dir.path().join("pasta"));
    let mcp = Actor::Mcp {
        client: "externo".into(),
        read_only: false,
    };
    let ro = Actor::Mcp {
        client: "externo".into(),
        read_only: true,
    };
    for (cmd, args) in [
        (
            "mail.accounts.save",
            json!({"name": "x", "kind": "folder", "folder_path": "/tmp"}),
        ),
        (
            "mail.accounts.set_password",
            json!({"id": acc, "password": "x"}),
        ),
        ("mail.rules.save", json!({"name": "x"})),
        ("mail.part_content", json!({"id": "x"})),
        ("mail.sync_now", json!({"id": acc})),
        ("mail.apply_part", json!({"part_id": "x"})),
        ("mail.accounts.confirm_retention", json!({"id": acc})),
    ] {
        assert!(api.call_as(&mcp, cmd, args).is_err(), "{cmd}");
    }
    assert!(api.call_as(&ro, "mail.list", json!({})).is_ok());
    assert!(api.call_as(&ro, "mail.accounts.list", json!({})).is_ok());
    // A pasta não pode ser a dos dados do Treasury.
    let inside = api.call(
        "mail.accounts.save",
        json!({"name": "x", "kind": "folder", "folder_path": api.paths().root.to_string_lossy()}),
    );
    assert!(inside.is_err());
    // Outlook: sem senha de app.
    assert!(
        api.call("mail.discover", json!({"address": "a@hotmail.com"}))
            .is_err()
    );
    let g = api
        .call("mail.discover", json!({"address": "a@gmail.com"}))
        .unwrap();
    assert_eq!(g["host"], "imap.gmail.com");
}

#[test]
fn rule_test_and_validation() {
    let (dir, api) = setup();
    let acc = folder_account(&api, &dir.path().join("pasta"));
    let bad = api.call(
        "mail.rules.save",
        json!({"name": "x", "conditions": {"subject": {"regex": "fatura ("}}}),
    );
    assert!(bad.unwrap_err().to_string().contains("inválida"));
    let t = api
        .call(
            "mail.rules.test",
            json!({"account_id": acc,
                   "rule": {"name": "Itaú", "conditions": {"from": ["*.itau.com.br"], "subject": {"contains": ["fatura"]}}},
                   "sample": {"folder": "INBOX", "folder_flags": [], "from": "faturas@mail.itau.com.br",
                              "to": [], "subject": "RES: Sua FATURA", "size": 1000, "attachments": []}}),
        )
        .unwrap();
    assert_eq!(t["sample"]["rule_name"], "Itaú", "{t}");
    assert_eq!(t["sample"]["passed"], true);
    // Testar não grava nada.
    assert_eq!(count(&api, "SELECT COUNT(*) FROM mail_rule"), 0);
    assert_eq!(config::DAILY_AI, 10);
}

#[test]
fn ai_on_rule_marks_parts_and_respects_the_rule() {
    let (dir, api) = setup();
    let inbox = dir.path().join("Fotos");
    put(
        &inbox,
        "cupom.png",
        b"\x89PNG\r\n\x1a\nnot really an image but enough for sniff",
    );
    let acc = folder_account(&api, &inbox);
    let r = rule(
        &api,
        json!({"name": "Fotos", "ai": "never",
               "conditions": {"advanced": {"type": "attachment", "names": ["*.png"]}}}),
    );
    let p = api
        .with_session(|s, _| Prepared::load(s.db.conn(), &acc))
        .unwrap();
    let raw = std::fs::read(inbox.join("cupom.png")).unwrap();
    let o = api
        .with_session(|s, _| {
            ingest::store_incoming(
                s.db.conn(),
                &p,
                Incoming {
                    folder: "INBOX".into(),
                    folder_flags: vec![],
                    uid: None,
                    uidvalidity: None,
                    file_name: Some("cupom.png".into()),
                    raw: raw.clone(),
                    is_message: false,
                    file_date: None,
                },
            )
        })
        .unwrap();
    let Outcome::Stored { message_id, ai, .. } = o else {
        panic!()
    };
    assert!(ai.is_empty());
    let m = api.call("mail.get", json!({"id": message_id})).unwrap();
    let part = m["parts"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(m["parts"][0]["kind"], "image");
    // A regra não deixa: pedir à IA é recusado.
    assert!(api.call("mail.ask_ai", json!({"part_id": part})).is_err());
    // Com IA automática, a peça já sai marcada para a IA.
    api.call(
        "mail.rules.save",
        json!({"id": r, "name": "Fotos", "ai": "auto",
               "conditions": {"advanced": {"type": "attachment", "names": ["*.png"]}}}),
    )
    .unwrap();
    let p = api
        .with_session(|s, _| Prepared::load(s.db.conn(), &acc))
        .unwrap();
    let mut raw2 = raw.clone();
    raw2.extend_from_slice(b"2");
    let o = api
        .with_session(|s, _| {
            ingest::store_incoming(
                s.db.conn(),
                &p,
                Incoming {
                    folder: "INBOX".into(),
                    folder_flags: vec![],
                    uid: None,
                    uidvalidity: None,
                    file_name: Some("cupom2.png".into()),
                    raw: raw2,
                    is_message: false,
                    file_date: None,
                },
            )
        })
        .unwrap();
    let Outcome::Stored { ai, .. } = o else {
        panic!()
    };
    assert_eq!(ai.len(), 1);
    let st: String = api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT status FROM mail_part WHERE id = ?1",
                [&ai[0]],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(MailStatus::parse(&st), MailStatus::AiPending);
}

#[test]
fn retention_never_deletes_without_a_rule_or_outside_the_folders() {
    let (dir, api) = setup();
    let inbox = dir.path().join("E");
    put(&inbox, "a.txt", b"qualquer coisa");
    let acc = folder_account(&api, &inbox);
    // Retenção da conta ligada, mas a mensagem entrou pelo padrão (sem regra): nunca sai.
    api.call(
        "mail.accounts.save",
        json!({"id": acc, "name": "E", "retention": {"mode": "after_days", "days": 1}}),
    )
    .unwrap();
    sync::sync_account(&api, &acc).unwrap();
    api.with_session(|s, _| {
        s.db.conn().execute(
            "UPDATE mail_message SET received_at = '2020-01-01T00:00:00.000Z'",
            [],
        )?;
        s.db.conn()
            .execute("UPDATE mail_account SET retention_confirmed = 1", [])?;
        Ok(())
    })
    .unwrap();
    let p = api
        .with_session(|s, _| Prepared::load(s.db.conn(), &acc))
        .unwrap();
    let list = api
        .with_session(|s, _| retention::candidates(s.db.conn(), &p, chrono::Utc::now()))
        .unwrap();
    assert!(list.is_empty());
}
