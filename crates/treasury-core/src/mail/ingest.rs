//! O caminho de cada mensagem (docs/email-ingest.md §Pipeline): filtros (com o corpo, depois
//! do download) → autenticação do remetente → já vista? (Message-ID, hash) → limite do dia →
//! peças (anexos, o .eml anexado e o corpo) → identificação pelo `import.sniff` e pelos
//! leitores determinísticos (OFX, cupom fiscal, aviso de compra e de Pix, modelo da regra) →
//! repetida? (hash, chave da nota, banco e mês da fatura) → fila de revisão. O que pode ser
//! aplicado sozinho e o que vai para a IA sai daqui numa lista; quem aplica é `apply`, fora
//! da trava do cofre.

use std::sync::OnceLock;

use base64::Engine as _;
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::config::{self, AccountKind, AiMode, DAILY_AI, DAILY_MESSAGES, MailAccount, MailRule};
use super::filter::{self, AccountFilter, Envelope, RuleAction, RuleSpec};
use super::message::{self, Mail, MailHeaders, Piece};
use super::store::{self, MailStatus};
use super::{auth, body};
use crate::error::Result;
use crate::i18n::{self, args, t_args};
use crate::ids::{Id, new_id};

/// Texto do catálogo com um argumento.
fn with(key: &str, name: &'static str, value: impl Into<String>) -> String {
    t_args(key, &args(&[(name, value.into().into())]))
}
use crate::time::now_utc;

pub fn sha256_hex(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}

/// A conta com as regras dela, carregadas uma vez por busca.
pub struct Prepared {
    pub account: MailAccount,
    pub rules: Vec<MailRule>,
    filter: AccountFilter,
    specs: Vec<RuleSpec>,
    brazil: bool,
}

impl Prepared {
    pub fn load(conn: &Connection, account_id: &str) -> Result<Self> {
        let account = config::get_account(conn, None, account_id)?;
        let rules = config::rules_for(conn, account_id)?;
        Ok(Self {
            filter: config::account_filter(&account),
            specs: config::specs(&rules),
            rules,
            account,
            brazil: crate::country::is_brazil(conn),
        })
    }

    fn rule(&self, id: Option<&str>) -> Option<&MailRule> {
        id.and_then(|id| self.rules.iter().find(|r| r.id == id))
    }

    /// Decide pelos cabeçalhos (IMAP, antes de baixar). Com `needs_body`, baixa e decide de
    /// novo em [`store_incoming`].
    pub fn pre_filter(&self, env: &Envelope) -> filter::Decision {
        filter::decide(&self.filter, &self.specs, env)
    }
}

/// O que chegou: uma mensagem (IMAP ou .eml da pasta local) ou um arquivo solto da pasta.
pub struct Incoming {
    pub folder: String,
    pub folder_flags: Vec<String>,
    pub uid: Option<u32>,
    pub uidvalidity: Option<u32>,
    /// Pasta local: o nome do arquivo (dentro da subpasta `folder`).
    pub file_name: Option<String>,
    pub raw: Vec<u8>,
    /// Mensagem de e-mail; senão, um arquivo solto (extrato, nota, cupom…).
    pub is_message: bool,
    /// Data do arquivo solto (`YYYY-MM-DD`).
    pub file_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Recusada pelos filtros (só a contagem fica).
    Rejected(String),
    /// Já vista (mesmo Message-ID ou mesmo conteúdo).
    Known,
    /// Limite de mensagens do dia: fica para amanhã.
    Limit,
    Stored {
        message_id: Id,
        /// Peças que podem ser aplicadas sozinhas.
        auto: Vec<Id>,
        /// Peças que vão para a IA (regra com IA automática).
        ai: Vec<Id>,
    },
}

/// O que a identificação achou numa peça.
#[derive(Debug, Clone)]
pub struct Identified {
    pub kind: &'static str,
    pub label: String,
    pub layout_id: Option<String>,
    pub status: MailStatus,
    pub detail: Option<Value>,
    pub dedup_key: Option<String>,
    /// Leitor determinístico que dá para aplicar sozinho (OFX, cupom, aviso do corpo).
    pub deterministic: bool,
}

impl Identified {
    fn new(kind: &'static str, label: impl Into<String>) -> Self {
        Self {
            kind,
            label: label.into(),
            layout_id: None,
            status: MailStatus::AwaitingReview,
            detail: None,
            dedup_key: None,
            deterministic: false,
        }
    }
    fn unknown(label: impl Into<String>) -> Self {
        Self {
            status: MailStatus::Unrecognized,
            ..Self::new("unknown", label)
        }
    }
}

fn mime_of(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    match n.rsplit('.').next().unwrap_or("") {
        "pdf" => "application/pdf",
        "ofx" | "qfx" => "application/x-ofx",
        "csv" => "text/csv",
        "xml" => "application/xml",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "heic" => "image/heic",
        "webp" => "image/webp",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

/// Link da consulta da NFC-e no corpo (o mesmo do QR do cupom).
fn nfce_link(body: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#"(?i)https?://[a-z0-9.\-]+\.gov\.br/[^\s"'<>)\]]*"#).expect("regex")
    });
    re.find_iter(body)
        .map(|m| m.as_str().trim_end_matches(['.', ',', ';']).to_string())
        .find(|u| {
            let l = u.to_ascii_lowercase();
            l.contains("nfce") && (l.contains("p=") || l.contains("chnfe"))
        })
}

/// Identifica uma peça pelo conteúdo (nunca pelo nome só).
pub fn identify(conn: &Connection, piece: &Piece, brazil: bool) -> Identified {
    let b64 = base64::engine::general_purpose::STANDARD.encode(&piece.bytes);
    let sniff = crate::import::sniff::sniff(&crate::import::sniff::SniffArgs {
        filename: piece.filename.clone(),
        content_base64: b64,
    });
    let Ok(s) = sniff else {
        return Identified::unknown(i18n::t("nucleo-email-peca-nao-reconhecida"));
    };
    use crate::import::sniff::FileKind;
    let layout_id = s.layout.as_ref().map(|l| l.id.clone());
    let mut id = match s.kind {
        FileKind::Statement if s.format.as_deref() == Some("ofx") => Identified {
            deterministic: true,
            ..Identified::new("statement", s.label.clone())
        },
        FileKind::Statement if s.format.as_deref() == Some("pdf") => {
            let mut i = Identified::new("card_statement", s.label.clone());
            if let Ok(text) = crate::pdf::text(&piece.bytes, None) {
                let c = crate::import::card_pdf::parse(&text);
                if let (Some(issuer), Some(due)) = (&c.issuer, c.due_date) {
                    i.dedup_key = Some(format!(
                        "card:{}:{}",
                        issuer.to_lowercase(),
                        crate::time::month_of(due)
                    ));
                }
            }
            i
        }
        FileKind::Statement => Identified::new("statement", s.label.clone()),
        FileKind::BrokerageNote => Identified::new("brokerage_note", s.label.clone()),
        FileKind::B3Report => Identified::new("b3_report", s.label.clone()),
        FileKind::PensionStatement => Identified::new("pension_statement", s.label.clone()),
        FileKind::Image => identify_image(piece, &s.label),
        // Cupom (PDF/XML de NFC-e): o caminho de antes, que reconhece a nota e a chave.
        FileKind::Unknown | FileKind::Receipt => identify_unknown(piece, brazil, &s.label),
    };
    if id.layout_id.is_none() {
        id.layout_id = layout_id;
    }
    if let Some(note) = &s.note
        && id.status == MailStatus::Unrecognized
        && id.detail.is_none()
    {
        id.detail = Some(json!({ "note": note }));
    }
    // Repetida: o mesmo arquivo, a mesma nota ou a mesma fatura já entrou.
    let seen = |sql: &str, key: &str| -> bool {
        conn.query_row(sql, [key], |_| Ok(()))
            .optional()
            .ok()
            .flatten()
            .is_some()
    };
    let sha = sha256_hex(&piece.bytes);
    let dup = seen(
        "SELECT 1 FROM mail_part WHERE sha256 = ?1 AND status IN ('imported', 'duplicate') LIMIT 1",
        &sha,
    ) || id.dedup_key.as_deref().is_some_and(|k| {
        seen(
            "SELECT 1 FROM mail_part WHERE dedup_key = ?1 AND status = 'imported' LIMIT 1",
            k,
        ) || k
            .strip_prefix("nfe:")
            .is_some_and(|key| seen("SELECT 1 FROM receipt WHERE access_key = ?1 LIMIT 1", key))
    });
    if dup {
        id.status = MailStatus::Duplicate;
    }
    id
}

/// Imagem: o texto do OCR local diz se é boleto, Pix ou comprovante (docs/ocr-other-documents.md
/// › 3.1); a revisão abre a leitura certa na janela de importação. Sem texto, fica para a IA.
fn identify_image(piece: &Piece, label: &str) -> Identified {
    let unread = || Identified {
        status: MailStatus::Unrecognized,
        ..Identified::new("image", label.to_string())
    };
    let Ok(o) = crate::ocr::read_text(None, 1.0, Some(&piece.bytes)) else {
        return unread();
    };
    if o.is_empty() {
        return unread();
    }
    let today = crate::time::today_local();
    let (kind, bill, pix, proof) = crate::payments::classify(&o.rows, &[], today);
    let folded = crate::import::text::fold(&o.text());
    let looks = crate::import::image::classify(
        &format!(" {folded} "),
        0,
        false,
        crate::receipts::ocr::lines::Tier::Unbalanced,
    );
    use crate::payments::PaymentDocKind as K;
    let (key, dedup, detail) = match kind {
        K::Bill => {
            let b = bill.expect("boleto");
            (
                "nucleo-importacao-imagem-boleto",
                Some(crate::payments::bill_key(&b)),
                json!({"ocr": "bill", "amount": b.amount, "due_date": b.due_date}),
            )
        }
        K::Pix => {
            let p = pix.expect("pix");
            (
                "nucleo-importacao-imagem-pix",
                Some(crate::payments::pix::dedup_key(&p)),
                json!({"ocr": "pix", "amount": p.amount, "name": p.name}),
            )
        }
        K::Proof => {
            let p = proof.expect("comprovante");
            (
                "nucleo-importacao-imagem-comprovante",
                p.e2e_id.as_ref().map(|e| format!("e2e:{e}")),
                json!({"ocr": "proof", "amount": p.amount, "date": p.date}),
            )
        }
        K::None => match looks {
            crate::import::image::ImageKind::Receipt => (
                "nucleo-importacao-imagem-documento",
                None,
                json!({"ocr": "receipt"}),
            ),
            _ => return unread(),
        },
    };
    Identified {
        dedup_key: dedup,
        detail: Some(detail),
        ..Identified::new("image", i18n::t(key))
    }
}

/// PDF, XML ou texto que o `sniff` não reconheceu: cupom fiscal ou holerite.
fn identify_unknown(piece: &Piece, brazil: bool, label: &str) -> Identified {
    let text = if piece.bytes.starts_with(b"%PDF") {
        crate::pdf::text(&piece.bytes, None).ok()
    } else {
        std::str::from_utf8(&piece.bytes).ok().map(str::to_string)
    };
    let Some(text) = text else {
        return Identified::unknown(label);
    };
    if brazil {
        let parsed = if crate::receipts::nfce::looks_like_xml(&text) {
            Some(crate::receipts::nfce::parse_xml(&text))
        } else if crate::receipts::nfce::looks_like(&text) {
            Some(crate::receipts::nfce::parse_text(&text))
        } else {
            None
        };
        if let Some(p) = parsed.filter(|p| !p.items.is_empty()) {
            return Identified {
                deterministic: true,
                dedup_key: p.access_key.as_ref().map(|k| format!("nfe:{k}")),
                detail: Some(json!({
                    "store": p.store_name,
                    "date": p.date,
                    "paid": p.paid,
                    "items": p.items.len(),
                })),
                ..Identified::new(
                    "receipt",
                    match &p.store_name {
                        Some(s) => with("nucleo-email-peca-cupom-loja", "loja", s.clone()),
                        None => i18n::t("nucleo-email-peca-cupom"),
                    },
                )
            };
        }
    }
    if crate::income::payslip::matches_generic(&text) {
        return Identified::new("payslip", i18n::t("nucleo-email-peca-holerite"));
    }
    Identified::unknown(label)
}

fn draft_part(d: &body::TxnDraft) -> Identified {
    let (kind, label) = match d.kind {
        body::DraftKind::Purchase => (
            "purchase",
            with(
                "nucleo-email-peca-aviso-compra",
                "descricao",
                d.description.clone(),
            ),
        ),
        body::DraftKind::PixIn | body::DraftKind::PixOut => ("pix", d.description.clone()),
        body::DraftKind::Template => (
            "template",
            with(
                "nucleo-email-peca-modelo",
                "descricao",
                d.description.clone(),
            ),
        ),
    };
    Identified {
        deterministic: true,
        detail: serde_json::to_value(d).ok(),
        ..Identified::new(kind, label)
    }
}

/// Conta do aviso: o cartão pelo final, se houver um só com ele.
fn draft_account(conn: &Connection, d: &mut body::TxnDraft) -> Result<()> {
    if d.account_id.is_some() {
        return Ok(());
    }
    if let Some(l4) = &d.card_last4 {
        let ids: Vec<String> = conn
            .prepare(
                "SELECT a.id FROM account a JOIN credit_card c ON c.account_id = a.id
                  WHERE c.last4 = ?1 AND a.archived = 0",
            )?
            .query_map([l4], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        if ids.len() == 1 {
            d.account_id = ids.into_iter().next();
        }
    }
    Ok(())
}

/// O remetente pode ter o resultado aplicado sozinho?
fn trusted(
    acc: &MailAccount,
    rule: Option<&MailRule>,
    mail: &Mail,
    check: &auth::AuthCheck,
) -> bool {
    if !check.authenticated {
        return false;
    }
    let Some(orig) = mail.original_from.as_deref().filter(|_| mail.forwarded) else {
        // Encaminhada sem o remetente original à vista: não dá para confiar.
        return !mail.forwarded;
    };
    // Encaminhada por um membro autenticado: só vale com o remetente original na lista
    // de permitidos da própria regra (não basta "membros").
    let member = filter::sender_matches("membros", &mail.headers.from, &acc.family_addresses);
    member
        && rule.is_some_and(|r| {
            r.conditions
                .from
                .iter()
                .filter(|p| !p.trim().to_ascii_lowercase().starts_with("membros"))
                .any(|p| filter::sender_matches(p, orig, &[]))
        })
}

/// Grava a mensagem (se passar nos filtros) e as peças dela, já identificadas.
pub fn store_incoming(conn: &Connection, p: &Prepared, inc: Incoming) -> Result<Outcome> {
    let acc = &p.account;
    let raw_hash = sha256_hex(&inc.raw);
    let mail = if inc.is_message {
        match message::parse(&inc.raw) {
            Ok(m) => m,
            Err(e) => {
                return Ok(Outcome::Rejected(with(
                    "nucleo-email-erro-ler-mensagem",
                    "erro",
                    e.message(),
                )));
            }
        }
    } else {
        let name = inc.file_name.clone().unwrap_or_else(|| "arquivo".into());
        Mail {
            headers: MailHeaders {
                subject: name.clone(),
                date: inc.file_date.clone(),
                ..Default::default()
            },
            pieces: vec![Piece {
                mime: mime_of(&name).into(),
                filename: name,
                bytes: inc.raw.clone(),
            }],
            ..Default::default()
        }
    };
    let env = Envelope {
        folder: inc.folder.clone(),
        folder_flags: inc.folder_flags.clone(),
        from: mail.headers.from.clone(),
        original_from: mail.original_from.clone(),
        to: mail.headers.to.clone(),
        subject: mail.headers.subject.clone(),
        date: mail.headers.date.clone(),
        size: inc.raw.len() as u64,
        attachments: mail.pieces.iter().map(|x| x.filename.clone()).collect(),
        body: Some(mail.body.clone()),
    };
    let decision = filter::decide(&p.filter, &p.specs, &env);
    if !decision.passed {
        return Ok(Outcome::Rejected(decision.reason));
    }
    let known: Option<String> = conn
        .query_row(
            "SELECT id FROM mail_message WHERE account_id = ?1 AND (raw_hash = ?2 OR message_id = ?3) LIMIT 1",
            params![acc.id, raw_hash, mail.headers.message_id],
            |r| r.get(0),
        )
        .optional()?;
    if known.is_some() {
        return Ok(Outcome::Known);
    }
    if store::messages_today(conn, &acc.id)? >= DAILY_MESSAGES {
        return Ok(Outcome::Limit);
    }
    let rule = p.rule(decision.rule_id.as_deref());
    let check = if inc.is_message {
        auth::check(
            &mail.headers.auth_results,
            acc.authserv_id.as_deref(),
            &mail.headers.from,
        )
    } else {
        auth::AuthCheck {
            authenticated: false,
            detail: i18n::t("nucleo-email-auth-pasta-local"),
        }
    };
    // Arquivo solto na pasta do próprio usuário: quem pôs foi ele.
    let is_trusted = if inc.is_message {
        trusted(acc, rule, &mail, &check)
    } else {
        acc.kind == AccountKind::Folder
    };
    let now = now_utc();
    let msg_id = new_id();
    conn.execute(
        "INSERT INTO mail_message (id, account_id, folder, uid, uidvalidity, file_name, message_id, sender,
            original_sender, subject, date, size, authenticated, trusted, auth_detail, rule_id, rule_name,
            status, raw_hash, raw, error, received_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
            'awaiting_review', ?18, ?19, ?20, ?21, ?21, ?21)",
        params![
            msg_id,
            acc.id,
            inc.folder,
            inc.uid,
            inc.uidvalidity,
            inc.file_name,
            mail.headers.message_id,
            Some(mail.headers.from.clone()).filter(|s| !s.is_empty()),
            mail.original_from,
            mail.headers.subject,
            mail.headers.date,
            inc.raw.len() as i64,
            check.authenticated as i64,
            is_trusted as i64,
            check.detail,
            decision.rule_id,
            decision.rule_name,
            raw_hash,
            acc.keep_original.then_some(&inc.raw),
            (!mail.warnings.is_empty()).then(|| mail.warnings.join("; ")),
            now
        ],
    )?;
    // Peças: os anexos (e o .eml anexado) e, sem documento reconhecido, o corpo.
    let mut parts: Vec<(Identified, String, String, Vec<u8>)> = vec![];
    for piece in &mail.pieces {
        let id = identify(conn, piece, p.brazil);
        parts.push((
            id,
            piece.filename.clone(),
            piece.mime.clone(),
            piece.bytes.clone(),
        ));
    }
    let year = mail
        .headers
        .date
        .as_deref()
        .and_then(|d| d.get(..4))
        .and_then(|y| y.parse().ok())
        .or_else(|| now.get(..4).and_then(|y| y.parse().ok()));
    let recognized = parts
        .iter()
        .any(|(i, ..)| i.kind != "unknown" && i.kind != "image");
    if inc.is_message && !mail.body.trim().is_empty() {
        let text = mail.body.as_bytes().to_vec();
        let drafted = match rule.and_then(|r| r.template.as_ref()) {
            Some(t) => body::read_template(t, &mail.body, year),
            None if !recognized => body::read(&mail.body, year),
            None => None,
        };
        if let Some(mut d) = drafted {
            if d.date.is_none() {
                d.date = mail.headers.date.clone();
            }
            draft_account(conn, &mut d)?;
            parts.push((
                draft_part(&d),
                i18n::t("nucleo-email-peca-corpo"),
                "text/plain".into(),
                text,
            ));
        } else if p.brazil
            && !parts.iter().any(|(i, ..)| i.kind == "receipt")
            && let Some(url) = nfce_link(&mail.body)
        {
            let key = crate::receipts::nfce::access_key(&url);
            let mut i = Identified {
                deterministic: true,
                dedup_key: key.as_ref().map(|k| format!("nfe:{k}")),
                detail: Some(json!({ "url": url })),
                ..Identified::new("receipt", i18n::t("nucleo-email-peca-cupom-link"))
            };
            if key.is_some_and(|k| {
                conn.query_row("SELECT 1 FROM receipt WHERE access_key = ?1", [k], |_| {
                    Ok(())
                })
                .optional()
                .ok()
                .flatten()
                .is_some()
            }) {
                i.status = MailStatus::Duplicate;
            }
            parts.push((
                i,
                i18n::t("nucleo-email-peca-link-cupom"),
                "text/uri-list".into(),
                url.into_bytes(),
            ));
        } else if !recognized {
            parts.push((
                Identified::unknown(i18n::t("nucleo-email-peca-corpo-nao-reconhecido")),
                i18n::t("nucleo-email-peca-corpo"),
                "text/plain".into(),
                text,
            ));
        }
    }
    // Mensagem: só com a regra restrita a remetentes dados (um "pass" sozinho não basta:
    // já houve remetente falsificado com DKIM e DMARC válidos).
    let names_senders = rule.is_some_and(|r| {
        r.conditions
            .from
            .iter()
            .any(|p| !p.trim().to_ascii_lowercase().starts_with("membros"))
    });
    let auto_ok = decision.action == Some(RuleAction::Auto)
        && is_trusted
        && (!inc.is_message || names_senders);
    let ai_auto = rule.is_some_and(|r| r.ai == AiMode::Auto);
    let mut ai_left = if ai_auto {
        (DAILY_AI - store::count_today(conn, &acc.id, "ai")?).max(0)
    } else {
        0
    };
    let mut auto = vec![];
    let mut ai = vec![];
    for (idx, (mut i, filename, mime, bytes)) in parts.into_iter().enumerate() {
        let part_id = new_id();
        if ai_auto && i.status == MailStatus::Unrecognized && ai_left > 0 {
            i.status = MailStatus::AiPending;
            ai_left -= 1;
            ai.push(part_id.clone());
        }
        if auto_ok && i.deterministic && i.status == MailStatus::AwaitingReview {
            auto.push(part_id.clone());
        }
        conn.execute(
            "INSERT INTO mail_part (id, message_id, idx, filename, mime, size, sha256, kind, label, layout_id,
                status, detail, dedup_key, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
            params![
                part_id,
                msg_id,
                idx as i64,
                filename,
                mime,
                bytes.len() as i64,
                sha256_hex(&bytes),
                i.kind,
                i.label,
                i.layout_id,
                i.status.as_str(),
                i.detail.as_ref().map(|d| d.to_string()),
                i.dedup_key,
                bytes,
                now
            ],
        )?;
    }
    store::refresh_message(conn, &msg_id)?;
    store::audit(
        conn,
        Some(&acc.id),
        "received",
        Some(&msg_id),
        &format!(
            "{}{}",
            decision
                .rule_name
                .as_deref()
                .map(|r| format!("regra \"{r}\": "))
                .unwrap_or_default(),
            mail.headers.subject
        ),
    )?;
    Ok(Outcome::Stored {
        message_id: msg_id,
        auto,
        ai,
    })
}

// ---------------------------------------------------------------------------
// Aplicar sozinho (fora da trava do cofre: cada comando segura o banco só o tempo dele)

/// Resultado de aplicar uma peça sozinha.
#[derive(Debug, Clone, PartialEq)]
pub enum Applied {
    Imported(Option<String>),
    Duplicate,
    /// Fica na revisão (com o motivo: aviso na prévia, conta não reconhecida…).
    Review(String),
}

/// Quem aplica: a regra (sozinho, com a prévia sem avisos) ou o usuário (na revisão).
pub enum By {
    Rule,
    User,
}

/// Aplica uma peça: OFX pela importação de extrato, cupom pelo `receipts.import`, aviso do
/// corpo pelo lançamento. Sozinho, só com a prévia sem avisos; senão, fica na revisão.
pub fn apply_part(
    api: &crate::Api,
    part_id: &str,
    by: By,
    account_override: Option<&str>,
) -> Result<Applied> {
    let (content, detail, rule) = api.with_session(|s, _| {
        let conn = s.db.conn();
        let c = store::part_content(conn, part_id)?;
        let detail: Option<String> = conn.query_row(
            "SELECT detail FROM mail_part WHERE id = ?1",
            [part_id],
            |r| r.get(0),
        )?;
        let rule: Option<String> = conn.query_row(
            "SELECT m.rule_name FROM mail_message m JOIN mail_part p ON p.message_id = m.id WHERE p.id = ?1",
            [part_id],
            |r| r.get(0),
        )?;
        Ok((c, detail.and_then(|d| serde_json::from_str::<Value>(&d).ok()), rule))
    })?;
    let strict = matches!(by, By::Rule);
    let actor = match by {
        By::Rule => crate::api::Actor::Mail {
            rule: rule
                .unwrap_or_else(|| i18n::t("nucleo-email-sem-regra"))
                .chars()
                .take(80)
                .collect(),
        },
        By::User => crate::api::Actor::User,
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&content.bytes);
    match content.kind.as_str() {
        "statement" => {
            let account = match account_override {
                Some(a) => Ok(a.to_string()),
                None => {
                    api.with_session(|s, _| ofx_account(s.db.conn(), &content.filename, &b64))?
                }
            };
            let account_id = match account {
                Ok(a) => a,
                Err(why) => {
                    return Ok(Applied::Review(with(
                        "nucleo-email-revisao-conta-extrato",
                        "motivo",
                        why,
                    )));
                }
            };
            let prev = api.call_as(
                &actor,
                "import.preview",
                json!({ "account_id": account_id, "filename": content.filename, "content_base64": b64 }),
            )?;
            let counts = &prev["counts"];
            let n = |k: &str| counts[k].as_u64().unwrap_or(0);
            if prev["already_imported"].as_bool() == Some(true) || n("new") == 0 && n("error") == 0
            {
                return Ok(Applied::Duplicate);
            }
            let warnings = prev["warnings"].as_array().map_or(0, Vec::len);
            if strict && (warnings > 0 || n("error") > 0 || n("possible_duplicate") > 0) {
                return Ok(Applied::Review(t_args(
                    "nucleo-email-revisao-previa",
                    &args(&[
                        ("avisos", warnings.into()),
                        ("erros", n("error").into()),
                        ("repetidos", n("possible_duplicate").into()),
                    ]),
                )));
            }
            if n("new") == 0 {
                return Ok(Applied::Review(i18n::t("nucleo-email-revisao-nada-novo")));
            }
            let res = api.call_as(&actor, "import.commit", json!({ "token": prev["token"] }))?;
            Ok(Applied::Imported(
                res["batch_id"].as_str().map(str::to_string),
            ))
        }
        "receipt" => {
            let mut args = match detail.as_ref().and_then(|d| d["url"].as_str()) {
                Some(url) => json!({ "url": url }),
                None => {
                    json!({ "files": [{ "filename": content.filename, "content_base64": b64 }] })
                }
            };
            let prev = api.call_as(&actor, "receipts.preview", args.clone())?;
            if prev["duplicate"].is_string() {
                return Ok(Applied::Duplicate);
            }
            let warnings: Vec<String> = prev["warnings"]
                .as_array()
                .map(|w| {
                    w.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            if strict && !warnings.is_empty() {
                return Ok(Applied::Review(warnings.join("; ")));
            }
            if let Some(t) = prev["txn"]["txn_id"].as_str() {
                args["txn_id"] = json!(t);
            }
            let r = api.call_as(&actor, "receipts.import", args)?;
            Ok(Applied::Imported(r["id"].as_str().map(str::to_string)))
        }
        "purchase" | "pix" | "template" => {
            let Some(mut d) = detail.and_then(|d| serde_json::from_value::<body::TxnDraft>(d).ok())
            else {
                return Ok(Applied::Review(i18n::t(
                    "nucleo-email-revisao-sem-lancamento",
                )));
            };
            if let Some(a) = account_override {
                d.account_id = Some(a.to_string());
            }
            let Some(account_id) = d.account_id.clone() else {
                return Ok(Applied::Review(match &d.card_last4 {
                    Some(l) => with("nucleo-email-revisao-cartao-final", "final", l.clone()),
                    None => i18n::t("nucleo-email-revisao-sem-conta"),
                }));
            };
            let Some(date) = d.date.clone() else {
                return Ok(Applied::Review(i18n::t("nucleo-email-revisao-sem-data")));
            };
            // Repetido: o mesmo valor na mesma conta, perto da data.
            let dup = api.with_session(|s, _| {
                Ok(s.db
                    .conn()
                    .query_row(
                        "SELECT 1 FROM txn WHERE account_id = ?1 AND amount = ?2
                            AND date BETWEEN date(?3, '-2 days') AND date(?3, '+2 days') LIMIT 1",
                        params![account_id, d.amount, date],
                        |_| Ok(()),
                    )
                    .optional()?
                    .is_some())
            })?;
            if dup && strict {
                return Ok(Applied::Duplicate);
            }
            let t = api.call_as(
                &actor,
                "transactions.create",
                json!({
                    "account_id": account_id,
                    "date": date,
                    "amount": d.amount,
                    "description": d.description,
                }),
            )?;
            Ok(Applied::Imported(t["id"].as_str().map(str::to_string)))
        }
        _ => Ok(Applied::Review(i18n::t("nucleo-email-revisao-sempre"))),
    }
}

/// Conta de um OFX pelo banco, agência e número (`accounts::match_ofx`).
fn ofx_account(
    conn: &Connection,
    filename: &str,
    b64: &str,
) -> Result<std::result::Result<String, String>> {
    let d = crate::import::detect(&crate::import::DetectArgs {
        filename: filename.to_string(),
        content_base64: b64.to_string(),
        encoding: None,
        country: None,
    })?;
    let Some(ofx) = d.ofx else {
        return Ok(Err(i18n::t("nucleo-email-revisao-nao-ofx")));
    };
    let m = crate::accounts::match_ofx(
        conn,
        &crate::accounts::OfxMatchArgs {
            bank_id: ofx.bank_id,
            branch_id: ofx.branch_id,
            account_id: ofx.account_id,
            credit_card: Some(ofx.credit_card),
        },
        crate::time::today_local(),
    )?;
    Ok(m.account_id.ok_or_else(|| {
        m.reason
            .unwrap_or_else(|| i18n::t("nucleo-importacao-conta-nao-reconhecida"))
    }))
}

/// Grava o resultado de aplicar uma peça (o que não deu fica na revisão, com o motivo).
pub fn record_applied(
    conn: &Connection,
    part_id: &str,
    res: &Result<Applied>,
    who: &str,
) -> Result<MailStatus> {
    let msg = store::message_of_part(conn, part_id)?;
    let account: Option<String> = conn
        .query_row(
            "SELECT account_id FROM mail_message WHERE id = ?1",
            [&msg],
            |r| r.get(0),
        )
        .optional()?;
    Ok(match res {
        Ok(Applied::Imported(r)) => {
            let s = store::set_part(conn, part_id, MailStatus::Imported, r.as_deref(), None)?;
            store::audit(conn, account.as_deref(), "imported", Some(&msg), who)?;
            s
        }
        Ok(Applied::Duplicate) => {
            let s = store::set_part(conn, part_id, MailStatus::Duplicate, None, None)?;
            store::audit(
                conn,
                account.as_deref(),
                "duplicate",
                Some(&msg),
                &i18n::t("nucleo-email-ja-estava"),
            )?;
            s
        }
        Ok(Applied::Review(why)) => {
            store::set_part(conn, part_id, MailStatus::AwaitingReview, None, Some(why))?
        }
        Err(e) => store::set_part(
            conn,
            part_id,
            MailStatus::AwaitingReview,
            None,
            Some(&e.message()),
        )?,
    })
}

/// Aplica sozinho as peças (pela regra) e devolve quantas entraram.
pub fn apply_all(api: &crate::Api, parts: &[Id]) -> u32 {
    let mut applied = 0;
    for pid in parts {
        let res = apply_part(api, pid, By::Rule, None);
        if matches!(res, Ok(Applied::Imported(_))) {
            applied += 1;
        }
        let _ = api.with_session(|s, _| {
            record_applied(
                s.db.conn(),
                pid,
                &res,
                &i18n::t("nucleo-email-auditoria-aplicado-regra"),
            )
        });
    }
    applied
}
