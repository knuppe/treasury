//! Exportar uma conversa do agente (`ai.conversations.export`, só a interface): Markdown
//! com o título, as datas, cada mensagem (quem falou, motor, hora), as ferramentas usadas,
//! as propostas e os nomes dos anexos; ou um PDF montado desse Markdown no próprio
//! computador (escritor de PDF dos relatórios).
//!
//! Nada de segredo sai no arquivo: os valores guardados no cofre (chaves de API dos
//! motores, senhas de e-mail e outros segredos) são trocados por "[segredo]" onde
//! aparecerem, e também o que tem cara de chave (`sk-…`, `gsk_…`, `AIza…`, `Bearer …`,
//! `senha: …`). O raciocínio interno do modelo e os parâmetros completos das ferramentas
//! não entram; os bytes dos anexos também não.

use std::sync::LazyLock;

use regex::Regex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::conversation::{ConversationDetail, Message};
use crate::error::Result;
use crate::i18n::{args, t, t_args};
use crate::reports::pdf_writer::{A4_H, A4_W, Document, Font, Page, Rgb, encode, wrap};
use crate::vault::DbKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ConversationExportFormat")]
pub enum Format {
    Markdown,
    Pdf,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ConversationExportArgs {
    pub id: String,
    pub format: Format,
    /// Destino escolhido no diálogo "Salvar como" (conferido como nas outras exportações).
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ConversationExportResult {
    pub path: String,
    #[ts(type = "number")]
    pub bytes: i64,
    /// Páginas do PDF (0 no Markdown).
    #[ts(type = "number")]
    pub pages: i64,
    /// Quantos trechos com segredo foram trocados por "[segredo]".
    #[ts(type = "number")]
    pub redacted: i64,
}

const MASK: &str = "[segredo]";

/// O que tem cara de chave ou de senha, mesmo que não esteja guardado no cofre.
static KEYLIKE: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"\bsk-(?:ant-|proj-)?[A-Za-z0-9_\-]{16,}",
        r"\bgsk_[A-Za-z0-9]{20,}",
        r"\bxai-[A-Za-z0-9]{20,}",
        r"\bAIza[0-9A-Za-z_\-]{30,}",
        r"\b(?:ghp|gho|ghs|github_pat)_[A-Za-z0-9_]{20,}",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"(?i)\bbearer\s+[A-Za-z0-9._\-~+/]{20,}=*",
    ]
    .iter()
    .map(|p| Regex::new(p).expect("regex"))
    .collect()
});

/// "senha: xyz", "api key = xyz", "token: xyz": mantém o rótulo e esconde o valor.
static LABELED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(senha|password|contraseña|api[ _-]?key|chave de api|token|secret|segredo)(\s*[:=]\s*)(\S{4,})",
    )
    .expect("regex")
});

/// Valores dos segredos guardados no cofre (decifrados só aqui, para achar e esconder).
fn stored_secrets(conn: &Connection, key: &DbKey) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM secret")?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for id in ids {
        // Segredo que não decifra (precisa ser cadastrado de novo) não tem como aparecer.
        if let Ok(Some(v)) = super::secret::get(conn, key, &id) {
            let v = v.trim().to_string();
            if v.chars().count() >= 6 {
                out.push(v);
            }
        }
    }
    // Os maiores primeiro (um segredo pode conter outro).
    out.sort_by_key(|s| std::cmp::Reverse(s.len()));
    Ok(out)
}

/// Esconde os segredos do texto; devolve o texto e quantas trocas foram feitas.
pub fn scrub(text: &str, secrets: &[String]) -> (String, i64) {
    let mut n = 0i64;
    let mut out = text.to_string();
    for s in secrets {
        let c = out.matches(s.as_str()).count() as i64;
        if c > 0 {
            n += c;
            out = out.replace(s.as_str(), MASK);
        }
    }
    for re in KEYLIKE.iter() {
        let c = re.find_iter(&out).count() as i64;
        if c > 0 {
            n += c;
            out = re.replace_all(&out, MASK).into_owned();
        }
    }
    let c = LABELED
        .captures_iter(&out)
        .filter(|c| &c[3] != MASK)
        .count() as i64;
    if c > 0 {
        n += c;
        out = LABELED
            .replace_all(&out, |c: &regex::Captures<'_>| {
                format!("{}{}{MASK}", &c[1], &c[2])
            })
            .into_owned();
    }
    (out, n)
}

/// "2026-10-21T13:05:09Z" → data e hora no idioma do app (UTC → horário local).
fn when(iso: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(iso) {
        Ok(d) => {
            let local = d.with_timezone(&chrono::Local);
            format!(
                "{} {}",
                crate::i18n::date(crate::i18n::current(), local.date_naive()),
                local.format("%H:%M")
            )
        }
        Err(_) => iso.to_string(),
    }
}

fn message_md(m: &Message, out: &mut String) {
    let who = if m.role == "user" {
        t("nucleo-agente-exportar-voce")
    } else {
        match &m.engine_name {
            Some(e) => t_args(
                "nucleo-agente-exportar-agente-motor",
                &args(&[("motor", e.clone().into())]),
            ),
            None => t("nucleo-agente-exportar-agente"),
        }
    };
    out.push_str(&format!("## {who} · {}\n\n", when(&m.created_at)));
    let body = m.content.trim();
    if !body.is_empty() {
        out.push_str(body);
        out.push_str("\n\n");
    }
    if !m.attachments.is_empty() {
        let names: Vec<&str> = m.attachments.iter().map(|a| a.name.as_str()).collect();
        out.push_str(&format!(
            "> {}\n\n",
            t_args(
                "nucleo-agente-exportar-anexos",
                &args(&[("nomes", names.join(", ").into())]),
            )
        ));
    }
    if !m.tool_calls.is_empty() {
        out.push_str(&format!("> {}\n", t("nucleo-agente-exportar-ferramentas")));
        for c in &m.tool_calls {
            let status = if c.ok {
                String::new()
            } else {
                format!(" ({})", t("nucleo-agente-exportar-falhou"))
            };
            let summary = c.summary.trim();
            if summary.is_empty() {
                out.push_str(&format!("> - `{}`{status}\n", c.name));
            } else {
                out.push_str(&format!("> - `{}`: {summary}{status}\n", c.name));
            }
        }
        out.push('\n');
    }
    if !m.proposal_ids.is_empty() {
        out.push_str(&format!(
            "> {}\n\n",
            t_args(
                "nucleo-agente-exportar-propostas",
                &args(&[("n", (m.proposal_ids.len() as i64).into())]),
            )
        ));
    }
}

/// A conversa em Markdown (já sem segredos) e quantas trocas foram feitas.
pub fn markdown(c: &ConversationDetail, secrets: &[String]) -> (String, i64) {
    let mut out = format!("# {}\n\n", c.conversation.title.trim());
    out.push_str(&format!(
        "- {}\n",
        t_args(
            "nucleo-agente-exportar-iniciada",
            &args(&[("data", when(&c.conversation.created_at).into())]),
        )
    ));
    out.push_str(&format!(
        "- {}\n",
        t_args(
            "nucleo-agente-exportar-mensagens",
            &args(&[("n", (c.messages.len() as i64).into())]),
        )
    ));
    out.push_str(&format!(
        "- {}\n\n",
        t_args(
            "nucleo-agente-exportar-gerada",
            &args(&[("data", when(&crate::time::now_utc()).into())]),
        )
    ));
    out.push_str(&format!(
        "_{}_\n\n---\n\n",
        t("nucleo-agente-exportar-aviso")
    ));
    for m in &c.messages {
        message_md(m, &mut out);
    }
    scrub(&out, secrets)
}

// --- PDF ------------------------------------------------------------------------------

const MARGIN: f32 = 48.0;
const INK: Rgb = Rgb(28, 28, 30);
const DIM: Rgb = Rgb(110, 110, 118);
const ACCENT: Rgb = Rgb(70, 90, 200);
const RULE: Rgb = Rgb(215, 215, 222);

/// Só o que a fonte do PDF (WinAnsi) desenha; emoji e outros símbolos saem.
fn pdf_safe(s: &str) -> String {
    s.chars()
        .filter(|c| *c == '?' || encode(&c.to_string()) != b"?")
        .collect()
}

/// Tira a marcação de ênfase e de código do Markdown para o PDF.
fn plain(s: &str) -> String {
    pdf_safe(&s.replace("**", "").replace('`', "").replace("__", ""))
}

/// Palavras muito longas (links, caminhos) quebradas para caberem na linha.
fn breakable(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            if w.chars().count() <= 60 {
                w.to_string()
            } else {
                w.chars()
                    .collect::<Vec<_>>()
                    .chunks(60)
                    .map(|c| c.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

struct Writer {
    doc: Document,
    y: f32,
}

impl Writer {
    fn page(&mut self) -> &mut Page {
        self.doc.pages.last_mut().expect("página")
    }

    fn ensure(&mut self, h: f32) {
        if self.y + h > A4_H - MARGIN {
            self.doc.new_page();
            self.y = MARGIN;
        }
    }

    fn lines(&mut self, text: &str, x: f32, size: f32, font: Font, color: Rgb) {
        let width = A4_W - x - MARGIN;
        for line in wrap(&breakable(text), width, size, font) {
            self.ensure(size * 1.35);
            let y = self.y + size;
            self.page().text(x, y, size, font, color, &line);
            self.y += size * 1.35;
        }
    }
}

/// O PDF da conversa, montado a partir do Markdown já sem segredos.
pub fn pdf(md: &str, title: &str) -> (Vec<u8>, i64) {
    let mut w = Writer {
        doc: Document {
            pages: vec![],
            title: pdf_safe(title),
        },
        y: MARGIN,
    };
    w.doc.new_page();
    let mut code = false;
    for raw in md.lines() {
        let line = raw.trim_end();
        if line.trim_start().starts_with("```") {
            code = !code;
            continue;
        }
        if code {
            w.lines(&pdf_safe(line), MARGIN + 12.0, 9.0, Font::Regular, DIM);
            continue;
        }
        if line.is_empty() {
            w.y += 5.0;
        } else if let Some(h) = line.strip_prefix("# ") {
            w.lines(&plain(h), MARGIN, 17.0, Font::Bold, INK);
            w.y += 4.0;
        } else if let Some(h) = line.strip_prefix("## ") {
            w.ensure(30.0);
            w.y += 6.0;
            let y = w.y;
            w.page().line(MARGIN, y, A4_W - MARGIN, y, 0.6, RULE);
            w.y += 6.0;
            w.lines(&plain(h), MARGIN, 11.0, Font::Bold, ACCENT);
        } else if line.starts_with("### ") || line.starts_with("#### ") {
            w.lines(
                &plain(line.trim_start_matches('#').trim()),
                MARGIN,
                10.5,
                Font::Bold,
                INK,
            );
        } else if line == "---" {
            w.y += 4.0;
        } else if let Some(q) = line.strip_prefix("> ") {
            let q = q
                .strip_prefix("- ")
                .map(|r| format!("• {r}"))
                .unwrap_or(q.to_string());
            w.lines(&plain(&q), MARGIN + 12.0, 9.0, Font::Regular, DIM);
        } else if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            w.lines(
                &format!("• {}", plain(item)),
                MARGIN + 8.0,
                10.0,
                Font::Regular,
                INK,
            );
        } else if let Some(i) = line.strip_prefix('_').and_then(|r| r.strip_suffix('_')) {
            w.lines(&plain(i), MARGIN, 9.0, Font::Regular, DIM);
        } else {
            w.lines(&plain(line), MARGIN, 10.0, Font::Regular, INK);
        }
    }
    // Rodapé com a página.
    let n = w.doc.pages.len();
    for (i, p) in w.doc.pages.iter_mut().enumerate() {
        let label = pdf_safe(&t_args(
            "nucleo-agente-exportar-pagina",
            &args(&[("n", ((i + 1) as i64).into()), ("total", (n as i64).into())]),
        ));
        p.text(MARGIN, A4_H - 24.0, 8.0, Font::Regular, DIM, &label);
    }
    (w.doc.to_bytes(), n as i64)
}

/// Grava a conversa no destino escolhido.
pub fn export(
    conn: &Connection,
    key: &DbKey,
    paths: &crate::paths::TreasuryPaths,
    a: &ConversationExportArgs,
) -> Result<ConversationExportResult> {
    let dest = crate::export::check_destination(&a.path, paths)?;
    let detail = super::conversation::get(conn, &a.id)?;
    let secrets = stored_secrets(conn, key)?;
    let (md, redacted) = markdown(&detail, &secrets);
    let (bytes, pages) = match a.format {
        Format::Markdown => (md.into_bytes(), 0),
        Format::Pdf => pdf(&md, &detail.conversation.title),
    };
    std::fs::write(&dest, &bytes)?;
    Ok(ConversationExportResult {
        path: dest.display().to_string(),
        bytes: bytes.len() as i64,
        pages,
        redacted,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ai::conversation::{NewMessage, ToolCallBrief};
    use crate::ai::routing::AiTask;
    use crate::api::{Actor, Api};
    use crate::paths::TreasuryPaths;

    const KEY: &str = "sk-proj-AbCdEf0123456789xyzXYZ";

    fn setup() -> (tempfile::TempDir, Api, String) {
        let dir = tempfile::tempdir().unwrap();
        let api =
            Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
                .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        let id = api
            .with_session(|s, _| {
                let c = s.db.conn();
                crate::ai::secret::set(c, s.db.key(), "ai_engine:x", KEY)?;
                let id = crate::ai::conversation::create(c, AiTask::Chat, "Quanto gastei em mercado?")?;
                let none: &[ToolCallBrief] = &[];
                crate::ai::conversation::add(
                    c,
                    &id,
                    &NewMessage {
                        role: "user",
                        content: &format!(
                            "Quanto gastei em mercado? Minha chave é {KEY} e a do Groq gsk_abcdefghijklmnopqrstuvwxyz0123; senha: hunter2222 🙂"
                        ),
                        engine_id: None,
                        engine_name: None,
                        tool_calls: none,
                        proposal_ids: &[],
                        reasoning: None,
                        reasoning_ms: None,
                    },
                )?;
                let tools = [ToolCallBrief {
                    name: "get_spending".into(),
                    args: "{\"month\":\"2026-10\"}".into(),
                    summary: "R$ 1.234,56 em Mercado".into(),
                    ok: true,
                }];
                crate::ai::conversation::add(
                    c,
                    &id,
                    &NewMessage {
                        role: "assistant",
                        content: "Em outubro você gastou **R$ 1.234,56** em mercado.",
                        engine_id: None,
                        engine_name: Some("Motor de teste"),
                        tool_calls: &tools,
                        proposal_ids: &[],
                        reasoning: Some("pensamento interno"),
                        reasoning_ms: Some(900),
                    },
                )?;
                Ok(id)
            })
            .unwrap();
        (dir, api, id)
    }

    #[test]
    fn exports_markdown_and_pdf_without_secrets() {
        let (dir, api, id) = setup();
        let md_path = dir.path().join("conversa.md");
        let r = api
            .call(
                "ai.conversations.export",
                json!({"id": id, "format": "markdown", "path": md_path.display().to_string()}),
            )
            .unwrap();
        let md = std::fs::read_to_string(&md_path).unwrap();
        assert!(md.starts_with("# Quanto gastei em mercado?"), "{md}");
        assert!(
            md.contains("## Você ·") && md.contains("## Agente (Motor de teste) ·"),
            "{md}"
        );
        assert!(md.contains("R$ 1.234,56"), "{md}");
        assert!(
            md.contains("`get_spending`: R$ 1.234,56 em Mercado"),
            "{md}"
        );
        // Segredos fora: o guardado no cofre, o que tem cara de chave e a senha rotulada.
        assert!(
            !md.contains(KEY) && !md.contains("gsk_") && !md.contains("hunter2222"),
            "{md}"
        );
        assert!(md.contains("senha: [segredo]"), "{md}");
        assert_eq!(r["redacted"], 3);
        // Raciocínio e parâmetros das ferramentas não entram.
        assert!(
            !md.contains("pensamento interno") && !md.contains("2026-10\""),
            "{md}"
        );

        let pdf_path = dir.path().join("conversa.pdf");
        let r = api
            .call(
                "ai.conversations.export",
                json!({"id": id, "format": "pdf", "path": pdf_path.display().to_string()}),
            )
            .unwrap();
        let bytes = std::fs::read(&pdf_path).unwrap();
        assert!(bytes.starts_with(b"%PDF-1.4"));
        assert_eq!(r["pages"], 1);
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("sk-proj") && !text.contains("hunter2222"));
        assert!(text.contains("Quanto gastei em mercado"));

        // Só a interface exporta, e nunca para dentro das pastas do Treasury.
        let mcp = Actor::Mcp {
            client: "agente".into(),
            read_only: false,
        };
        let e = api
            .call_as(
                &mcp,
                "ai.conversations.export",
                json!({"id": id, "format": "markdown", "path": dir.path().join("x.md").display().to_string()}),
            )
            .unwrap_err();
        assert!(matches!(e, crate::Error::Forbidden(_)), "{e}");
        let inside = dir.path().join(".t").join("x.md");
        assert!(
            api.call(
                "ai.conversations.export",
                json!({"id": id, "format": "markdown", "path": inside.display().to_string()}),
            )
            .is_err()
        );
    }

    #[test]
    fn long_words_and_many_lines_paginate() {
        let mut md = String::from("# Título\n\n");
        for i in 0..200 {
            md.push_str(&format!(
                "Linha {i} com um link https://exemplo.com/{}\n\n",
                "a".repeat(150)
            ));
        }
        let (bytes, pages) = pdf(&md, "Título");
        assert!(pages > 3, "{pages}");
        assert!(bytes.starts_with(b"%PDF"));
    }
}
