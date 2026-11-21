//! Fase 3: a IA por regra (docs/email-ingest.md §IA). Desligada por padrão; por regra, nunca,
//! a pedido (na fila de revisão) ou sozinha (até 10 por dia por conta). A IA recebe só a peça,
//! como dado de terceiros, e só as ferramentas de prévia e de importação (mais as leituras de
//! contas, categorias e lançamentos, para não duplicar); o que ela gravar vira proposta, nunca
//! aplicada sozinha (nem no modo autônomo). Vai pela rota "Leitura de PDF", só com motores de
//! API ou modelos locais (as CLIs veem todas as ferramentas). O texto do corpo vai com CPF,
//! cartão, conta e e-mail mascarados.

use std::sync::Arc;

use serde_json::json;

use super::store::{self, MailStatus};
use crate::ai::agent::{self, AgentRequest};
use crate::ai::jobs::{JobKind, JobStatus};
use crate::ai::routing::AiTask;
use crate::api::{Api, SharedApi};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;

/// Prefixo do cliente das propostas que a IA faz a partir de um e-mail.
pub const AI_CLIENT_PREFIX: &str = "E-mail: ";

/// O que a IA pode usar ao ler uma peça.
pub const TOOLS: &[&str] = &[
    "list_accounts",
    "list_categories",
    "query_transactions",
    "preview_card_statement_pdf",
    "preview_brokerage_notes",
    "preview_payslip",
    "preview_receipt",
    "preview_pension_statement",
    "import_transactions",
    "import_statement_file",
    "import_brokerage_notes",
    "import_payslip",
    "import_receipt",
    "import_pension_statement",
];

struct Job {
    part_id: Id,
    account_id: Id,
    rule: String,
    sender: String,
    subject: String,
    filename: String,
    mime: Option<String>,
    bytes: Vec<u8>,
    body_text: bool,
}

fn load(api: &Api, part_id: &str) -> Result<Job> {
    api.with_session(|s, _| {
        let conn = s.db.conn();
        let content = store::part_content(conn, part_id)?;
        let msg = store::message_of_part(conn, part_id)?;
        let m = store::get(conn, &msg)?;
        Ok(Job {
            part_id: part_id.into(),
            account_id: m.account_id,
            rule: m.rule_name.unwrap_or_else(|| t("nucleo-email-sem-regra")),
            sender: m.original_sender.or(m.sender).unwrap_or_default(),
            subject: m.subject.unwrap_or_default(),
            body_text: content.mime.as_deref() == Some("text/plain"),
            filename: content.filename,
            mime: content.mime,
            bytes: content.bytes,
        })
    })
}

/// Confere se a regra da mensagem deixa pedir à IA (a pedido ou sozinha).
pub fn allowed(conn: &rusqlite::Connection, part_id: &str) -> Result<()> {
    let msg = store::message_of_part(conn, part_id)?;
    let rule_id: Option<String> = conn.query_row(
        "SELECT rule_id FROM mail_message WHERE id = ?1",
        [&msg],
        |r| r.get(0),
    )?;
    let ai: Option<String> = match rule_id {
        Some(r) => conn
            .query_row("SELECT ai FROM mail_rule WHERE id = ?1", [r], |r| r.get(0))
            .ok(),
        None => None,
    };
    match ai.as_deref() {
        Some("on_request") | Some("auto") => Ok(()),
        _ => Err(Error::Forbidden(t("nucleo-email-erro-ia-desligada"))),
    }
}

/// Dispara a leitura de uma peça num job (a fila mostra o andamento pelo estado da peça).
pub fn start(api: &SharedApi, part_id: &str) -> Result<Id> {
    let job_data = load(api, part_id)?;
    let rt = api.ai_runtime();
    let job = rt.new_job(JobKind::Routine, AiTask::Pdf, None);
    job.set_title(&format!("E-mail: ler {}", job_data.filename));
    let id = job.id.clone();
    api.with_session(|s, _| {
        let conn = s.db.conn();
        store::set_part(conn, part_id, MailStatus::AiPending, None, None)?;
        store::audit(
            conn,
            Some(&job_data.account_id),
            "ai",
            Some(&store::message_of_part(conn, part_id)?),
            &t_args(
                "nucleo-email-auditoria-ia-lendo",
                &args(&[
                    ("arquivo", job_data.filename.clone().into()),
                    ("regra", job_data.rule.clone().into()),
                ]),
            ),
        )
    })?;
    agent::spawn(api.clone(), job, move |api, job| {
        let outcome = run(api, job, &job_data);
        let _ = api.with_session(|s, _| {
            let conn = s.db.conn();
            match &outcome {
                Ok((text, proposals)) => {
                    let detail = json!({
                        "ai": {
                            "summary": text.chars().take(800).collect::<String>(),
                            "proposals": proposals,
                        }
                    });
                    store::set_part_detail(conn, &job_data.part_id, &detail)?;
                    // Com proposta, a revisão é na fila de aprovação; sem, segue não reconhecida.
                    let status = if proposals.is_empty() {
                        MailStatus::Unrecognized
                    } else {
                        MailStatus::AwaitingReview
                    };
                    store::set_part(conn, &job_data.part_id, status, None, None)?;
                }
                Err(e) => {
                    store::set_part(
                        conn,
                        &job_data.part_id,
                        MailStatus::Unrecognized,
                        None,
                        Some(&t_args(
                            "nucleo-email-erro-ia",
                            &args(&[("erro", e.clone().into())]),
                        )),
                    )?;
                }
            }
            Ok(())
        });
        match outcome {
            Ok(_) => job.finish(JobStatus::Done, None, None),
            Err(_) => job.finish(JobStatus::Failed, None, None),
        }
    });
    Ok(id)
}

fn run(
    api: &Api,
    job: &crate::ai::jobs::Job,
    d: &Job,
) -> std::result::Result<(String, Vec<Id>), String> {
    let (name, bytes) = if d.body_text {
        let text = String::from_utf8_lossy(&d.bytes);
        let (masked, _) = crate::ai::redact::redact(&text);
        ("corpo-do-email.txt".to_string(), masked.into_bytes())
    } else {
        (d.filename.clone(), d.bytes.clone())
    };
    let mut att = crate::ai::attachments::prepare(&name, d.mime.as_deref(), bytes)
        .map_err(|e| e.message())?;
    // Imagem: o texto do OCR local.
    crate::ai::attachments::add_ocr(&mut att, None);
    // E-mail: só o texto, quando o OCR leu (a imagem fica no computador).
    if att.ocr_confidence.is_some() {
        att.image_mode = crate::ai::attachments::ImageMode::Text;
    }
    let att = Arc::new(att);
    // Pedido ao modelo (vai como a mensagem do usuário), no idioma da instalação.
    let message = t_args(
        "nucleo-email-ia-pedido",
        &args(&[
            ("regra", d.rule.clone().into()),
            ("anexo", name.clone().into()),
            ("remetente", format!("{:?}", d.sender).into()),
            ("assunto", format!("{:?}", d.subject).into()),
        ]),
    );
    let key = agent::turn_files_key(job);
    api.ai_runtime().set_turn_files(&key, vec![att.clone()]);
    let req = AgentRequest {
        task: AiTask::Pdf,
        message,
        history: vec![],
        engine_id: None,
        strict_route: true,
        tools: true,
        context: None,
        notes: None,
        computer_access: false,
        attachments: vec![att],
        skip_unsupported: false,
        effort: Default::default(),
        member_id: None,
        client: Some(format!("{AI_CLIENT_PREFIX}{}", d.rule)),
        only_tools: Some(TOOLS),
    };
    let out = agent::run(api, job, &req);
    api.ai_runtime().drop_turn_files(&key);
    let reply = out.map_err(|e| e.message)?;
    let parts = crate::ai::activity::split(&job.events_after(0));
    let text = if parts.answer.is_empty() {
        reply.text
    } else {
        parts.answer
    };
    Ok((text.trim().to_string(), parts.proposals))
}
