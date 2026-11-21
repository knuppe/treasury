//! History retention: cleanup by age (pinned ones stay), expired attachments, `VACUUM`
//! after a big cleanup, and attachment retention applied to what is already stored.

use std::sync::Arc;

use chrono::{Duration, TimeZone, Utc};

use super::*;
use crate::ai::attachments::{self, Attachment, Retention, tests::png};
use crate::ai::conversation::{self, NewMessage};
use crate::ai::routing::AiTask;
use crate::ai::settings::{AiSettingsPatch, apply};
use crate::storage::Db;
use crate::vault::KDF_TEST;

fn db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let db = Db::create(&dir.path().join("t.db"), "senha", KDF_TEST).unwrap();
    (dir, db)
}

fn msg<'a>(role: &'a str, content: &'a str) -> NewMessage<'a> {
    NewMessage {
        role,
        content,
        engine_id: None,
        engine_name: None,
        tool_calls: &[],
        proposal_ids: &[],
        reasoning: None,
        reasoning_ms: None,
    }
}

/// Conversation with one question (with attachments) and one answer, last used at `at`.
fn conversation_at(
    conn: &Connection,
    title: &str,
    at: &str,
    atts: &[Arc<Attachment>],
    retention: Retention,
) -> String {
    let id = conversation::create(conn, AiTask::Chat, title).unwrap();
    let m = conversation::add(conn, &id, &msg("user", title)).unwrap();
    attachments::save(conn, &id, &m, atts, retention).unwrap();
    conversation::add(conn, &id, &msg("assistant", "ok")).unwrap();
    touch(conn, &id, at).unwrap();
    id
}

fn image(bytes: usize) -> Arc<Attachment> {
    let mut b = png(100, 100);
    b.resize(bytes, 7);
    Arc::new(attachments::prepare("foto.png", None, b).unwrap())
}

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 15, 12, 0, 0).unwrap()
}

fn iso(t: chrono::DateTime<Utc>) -> String {
    t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn defaults_are_no_attachments_and_ninety_days() {
    let (_d, db) = db();
    let s = settings::load(db.conn()).unwrap();
    assert_eq!(s.attachment_retention, Retention::None);
    assert_eq!(s.history_retention_days, 90);
    assert!(
        apply(
            db.conn(),
            &AiSettingsPatch {
                history_retention_days: Some(45),
                ..Default::default()
            }
        )
        .is_err(),
        "only the options from the list"
    );
}

#[test]
fn cleanup_removes_old_unpinned_conversations_and_keeps_pinned() {
    let (_d, db) = db();
    let conn = db.conn();
    let old = iso(now() - Duration::days(120));
    let recent = iso(now() - Duration::days(10));
    let gone = conversation_at(conn, "antiga", &old, &[image(2000)], Retention::None);
    let pinned = conversation_at(conn, "fixada", &old, &[], Retention::None);
    conversation::set_pinned(conn, &pinned, true).unwrap();
    let kept = conversation_at(conn, "recente", &recent, &[], Retention::None);
    let before = stats(conn).unwrap();
    assert_eq!(
        (before.conversations, before.pinned, before.messages),
        (3, 1, 6)
    );
    assert_eq!((before.attachments, before.stored_attachments), (1, 0));

    let r = cleanup(conn, now()).unwrap();
    assert_eq!(r.conversations_deleted, 1);
    assert_eq!(r.messages_deleted, 2);
    assert!(!conversation::exists(conn, &gone).unwrap());
    assert!(conversation::exists(conn, &pinned).unwrap());
    assert!(conversation::exists(conn, &kept).unwrap());
    // The attachment metadata travels with the conversation.
    assert_eq!(count(conn, "SELECT count(*) FROM ai_attachment"), 0);
    let after = stats(conn).unwrap();
    assert_eq!(after.messages, 4);
    assert!(after.last_cleanup.is_some());

    // Pinned lists first.
    let list = conversation::list(conn, 10).unwrap();
    assert_eq!(list[0].id, pinned);
    assert!(list[0].pinned);

    // "Never": nothing goes, however old.
    apply(
        conn,
        &AiSettingsPatch {
            history_retention_days: Some(0),
            ..Default::default()
        },
    )
    .unwrap();
    conversation::set_pinned(conn, &pinned, false).unwrap();
    touch(conn, &pinned, &iso(now() - Duration::days(4000))).unwrap();
    assert_eq!(cleanup(conn, now()).unwrap().conversations_deleted, 0);
}

#[test]
fn seven_day_attachments_expire_but_metadata_stays() {
    let (_d, db) = db();
    let conn = db.conn();
    let id = conversation_at(
        conn,
        "foto",
        &iso(Utc::now()),
        &[image(4000)],
        Retention::Days7,
    );
    let st = stats(conn).unwrap();
    assert_eq!(st.stored_attachments, 1);
    assert!(st.attachment_bytes >= 4000);
    // Before the deadline nothing changes.
    assert_eq!(cleanup(conn, Utc::now()).unwrap().attachments_expired, 0);
    let r = cleanup(conn, Utc::now() + Duration::days(8)).unwrap();
    assert_eq!(r.attachments_expired, 1);
    let st = stats(conn).unwrap();
    assert_eq!(
        (st.attachments, st.stored_attachments, st.attachment_bytes),
        (1, 0, 0)
    );
    // The conversation shows the attachment as "not stored" and the engine gets the notice instead.
    let detail = conversation::get(conn, &id).unwrap();
    let info = &detail.messages[0].attachments[0];
    assert_eq!(info.name, "foto.png");
    assert!(!info.stored);
    let ctx = conversation::context(conn, &id, &|_| None).unwrap();
    assert!(
        ctx[0]
            .text()
            .contains("[attachment sent earlier: foto.png (image) — not kept")
    );
}

#[test]
fn stored_attachments_replay_and_retention_change_purges() {
    let (_d, db) = db();
    let conn = db.conn();
    let id = conversation_at(
        conn,
        "foto",
        &iso(Utc::now()),
        &[image(3000)],
        Retention::Always,
    );
    let ctx = conversation::context(conn, &id, &|_| None).unwrap();
    assert!(
        ctx[0].blocks.iter().any(
            |b| matches!(b, crate::ai::providers::Block::Attachment(a) if a.data.len() == 3000)
        ),
        "stored: comes back as an attachment"
    );
    let aid: String = conn
        .query_row("SELECT id FROM ai_attachment", [], |r| r.get(0))
        .unwrap();
    assert_eq!(attachments::get_data(conn, &aid).unwrap().mime, "image/png");

    // Switching to "do not store" deletes the bytes already stored.
    apply(
        conn,
        &AiSettingsPatch {
            attachment_retention: Some(Retention::None),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(stats(conn).unwrap().stored_attachments, 0);
    assert!(attachments::get_data(conn, &aid).is_err());
    let ctx = conversation::context(conn, &id, &|_| None).unwrap();
    assert!(ctx[0].text().contains("not kept"));
}

#[test]
fn big_cleanup_vacuums_and_the_file_shrinks() {
    let (dir, db) = db();
    let conn = db.conn();
    let old = iso(now() - Duration::days(400));
    // About 12 MB of attachments stored in an old conversation.
    let atts: Vec<Arc<Attachment>> = (0..4).map(|_| image(3_000_000)).collect();
    conversation_at(conn, "pesada", &old, &atts, Retention::Always);
    let (db_before, _) = db_size(conn).unwrap();
    assert!(db_before > 12_000_000, "{db_before}");
    let file_before = std::fs::metadata(dir.path().join("t.db")).unwrap().len();

    let t0 = std::time::Instant::now();
    let r = cleanup(conn, now()).unwrap();
    eprintln!("faxina com VACUUM: {r:?} em {:?}", t0.elapsed());
    assert!(r.vacuumed, "{r:?}");
    // Only the empty vault schema is left (about 2 MB after the card tables, V155–V158).
    assert!(r.db_bytes_after < 2_500_000, "{r:?}");
    let file_after = std::fs::metadata(dir.path().join("t.db")).unwrap().len();
    // The file goes back to the schema size (which grows with each migration).
    assert!(
        file_after < 2_500_000 && file_after < file_before / 5,
        "{file_before} → {file_after}"
    );
    // The vault still opens and is intact after the VACUUM.
    let ok: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(ok, "ok");
    assert_eq!(stats(conn).unwrap().free_bytes, 0);
}

#[test]
fn small_cleanups_do_not_vacuum_and_run_once_a_day() {
    assert!(!should_vacuum(100 * 1024 * 1024, 2 * 1024 * 1024));
    assert!(should_vacuum(100 * 1024 * 1024, 9 * 1024 * 1024));
    assert!(should_vacuum(4 * 1024 * 1024, 1024 * 1024 + 1));
    assert!(!should_vacuum(4 * 1024 * 1024, 100 * 1024));

    let (_d, db) = db();
    let conn = db.conn();
    assert!(cleanup_due(conn, now()).unwrap());
    let r = run_due(conn).unwrap().expect("runs the first time");
    assert!(!r.vacuumed);
    assert!(run_due(conn).unwrap().is_none(), "not on the same day");
    assert!(cleanup_due(conn, Utc::now() + Duration::hours(25)).unwrap());
}

// --- Context carried to the engine ---------------------------------------------------

/// Conversation with `n` question/answer pairs (`#1` question, `#2` answer...).
fn long_conversation(conn: &Connection, n: usize) -> String {
    let id = conversation::create(conn, AiTask::Chat, "longa").unwrap();
    let tools = [conversation::ToolCallBrief {
        name: "get_portfolio".into(),
        args: "deflator=ipca".into(),
        summary: "Carteira de R$ 100.000,00 em 12 ativos.".into(),
        ok: true,
    }];
    for i in 0..n {
        conversation::add(
            conn,
            &id,
            &msg("user", &format!("pergunta {i} sobre o orçamento")),
        )
        .unwrap();
        conversation::add(
            conn,
            &id,
            &NewMessage {
                tool_calls: &tools,
                ..msg("assistant", &format!("resposta {i}"))
            },
        )
        .unwrap();
    }
    id
}

#[test]
fn long_history_is_summarized_with_a_stable_boundary_and_tool_trail() {
    let (_d, db) = db();
    let conn = db.conn();
    let id = long_conversation(conn, 15); // 30 messages
    let ctx = conversation::context(conn, &id, &|_| None).unwrap();
    // 30 − 12 = 18, a multiple of 6: the last 12 whole, the first 18 in the summary.
    assert_eq!(ctx.len(), 12);
    let first = ctx[0].text();
    assert!(first.starts_with("<historico_resumido>"), "{first}");
    assert!(first.contains("#1 user: pergunta 0 sobre o orçamento"));
    assert!(first.contains("#18 agent: resposta 8 [tools: get_portfolio]"));
    assert!(
        first.contains("pergunta 9 sobre o orçamento"),
        "message #19 comes whole"
    );
    assert!(!first.contains("#19 user"));
    // The answers carry the trail of what the tools returned.
    assert!(ctx[1].text().contains(
        "[tools used in this answer: get_portfolio(deflator=ipca) → Carteira de R$ 100.000,00 em 12 ativos.]"
    ));
    // One more pair: the boundary does not move (same beginning, good for the prompt cache).
    conversation::add(conn, &id, &msg("user", "mais uma")).unwrap();
    conversation::add(conn, &id, &msg("assistant", "ok")).unwrap();
    let ctx2 = conversation::context(conn, &id, &|_| None).unwrap();
    assert_eq!(ctx2[0].text(), first);
    assert_eq!(ctx2.len(), 14);
}

#[test]
fn huge_messages_are_cut_in_the_middle_and_push_the_boundary() {
    let (_d, db) = db();
    let conn = db.conn();
    let id = conversation::create(conn, AiTask::Chat, "grande").unwrap();
    let big = format!("INÍCIO {} FIM", "x".repeat(30_000));
    for _ in 0..4 {
        conversation::add(conn, &id, &msg("user", "me dê a tabela")).unwrap();
        conversation::add(conn, &id, &msg("assistant", &big)).unwrap();
    }
    let ctx = conversation::context(conn, &id, &|_| None).unwrap();
    let last = ctx.last().unwrap().text();
    assert!(last.starts_with("INÍCIO") && last.ends_with("FIM"));
    assert!(last.contains("characters omitted; the whole message #8 is in `read_chat_history`"));
    assert!(last.chars().count() < 9_000);
    // Nothing is lost for good: the beginning becomes a summary.
    assert!(ctx[0].text().contains("<historico_resumido>"));
}

#[test]
fn old_text_attachments_become_references_and_recent_ones_come_from_memory() {
    let (_d, db) = db();
    let conn = db.conn();
    let pdf = Arc::new(
        attachments::prepare(
            "extrato.pdf",
            None,
            crate::ai::attachments::tests::pdf_with_lines(&["Saldo final 1.234,56"]),
        )
        .unwrap(),
    );
    let id = conversation::create(conn, AiTask::Chat, "extrato").unwrap();
    let m = conversation::add(conn, &id, &msg("user", "leia o extrato")).unwrap();
    attachments::save(conn, &id, &m, std::slice::from_ref(&pdf), Retention::None).unwrap();
    conversation::add(conn, &id, &msg("assistant", "li")).unwrap();
    let mem = pdf.clone();
    let cached = move |aid: &str| (aid == mem.id).then(|| mem.clone());
    // Right after: the PDF is still in the conversation memory and comes back whole.
    let ctx = conversation::context(conn, &id, &cached).unwrap();
    assert!(
        ctx[0]
            .blocks
            .iter()
            .any(|b| matches!(b, crate::ai::providers::Block::Attachment(a) if a.id == pdf.id))
    );
    // Two messages later: only the reference (the agent rereads with read_attachment).
    conversation::add(conn, &id, &msg("user", "e o saldo?")).unwrap();
    conversation::add(conn, &id, &msg("assistant", "1.234,56")).unwrap();
    let ctx = conversation::context(conn, &id, &cached).unwrap();
    let t = ctx[0].text();
    assert!(
        t.contains("[attachment sent earlier (message #1): extrato.pdf (PDF, 1 page(s)"),
        "{t}"
    );
    assert!(t.contains(&format!("id {}", pdf.id)));
    assert!(t.contains("read_attachment"));
    // No memory and no stored copy: the "not stored" notice.
    let ctx = conversation::context(conn, &id, &|_| None).unwrap();
    assert!(ctx[0].text().contains("not kept"));

    // The tool reads the text from memory (the app injects it), in parts.
    let r = attachments::read_text(
        conn,
        &attachments::AttachmentReadArgs {
            conversation_id: Some(id.clone()),
            id: Some("EXTRATO.PDF".into()),
            memory_text: pdf.text.as_deref().map(|t| t.to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    match r {
        attachments::AttachmentRead::Text(t) => {
            assert!(t.text.contains("Saldo final 1.234,56"));
            assert_eq!(t.next_offset, None);
        }
        other => panic!("{other:?}"),
    }
    // Without the text (vault locked after sending): clear error.
    let e = attachments::read_text(
        conn,
        &attachments::AttachmentReadArgs {
            conversation_id: Some(id.clone()),
            id: Some(pdf.id.clone()),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(e.to_string().contains("anexar de novo"), "{e}");
    // Without an id: the list with availability.
    let list = attachments::read_text(
        conn,
        &attachments::AttachmentReadArgs {
            conversation_id: Some(id.clone()),
            memory_ids: Some(vec![pdf.id.clone()]),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        serde_json::to_string(&list)
            .unwrap()
            .contains("\"available\":\"memory\"")
    );

    // The history by number and by text.
    let h = conversation::history(
        conn,
        &conversation::ChatHistoryArgs {
            conversation_id: Some(id.clone()),
            query: Some("SALDO".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(h.total_messages, 4);
    assert_eq!(h.messages.len(), 1);
    assert_eq!(h.messages[0].seq, 3);
    let h = conversation::history(
        conn,
        &conversation::ChatHistoryArgs {
            conversation_id: Some(id),
            from_seq: Some(1),
            to_seq: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(h.messages[0].attachments.len(), 1);
}
