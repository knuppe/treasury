//! Histórico do agente (`ai_conversation`, `ai_message`), opcional (`ai.keep_history`).

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::attachments::{self, AttachmentInfo, Saved};
use super::providers::{Block, Msg};
use super::routing::AiTask;
use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

/// Mensagens recentes levadas por inteiro (as mais antigas viram o resumo).
pub const RECENT_MESSAGES: usize = 12;
/// Teto de caracteres das mensagens recentes (as que passam saem para o resumo).
pub const RECENT_CHARS: usize = 48_000;
/// Uma mensagem longa vai cortada no meio (começo e fim), com o número para ler inteira.
pub const MESSAGE_CHARS: usize = 8_000;
/// Teto do resumo das mensagens antigas.
pub const SUMMARY_CHARS: usize = 10_000;
/// A fronteira entre resumo e mensagens inteiras anda de tantas em tantas mensagens: o
/// começo do contexto fica igual por vários turnos e o cache de prompt do provider (e o
/// dos modelos locais) continua valendo.
pub const BOUNDARY_STEP: usize = 6;
/// Anexos de texto e PDFs voltam inteiros só da mensagem anterior; imagens, das últimas
/// mensagens ([`attachments::HISTORY_ATTACHMENT_MESSAGES`]). Os demais viram referência
/// (o agente relê com `read_attachment`).
pub const FULL_TEXT_ATTACHMENT_MESSAGES: usize = 2;
/// Ferramentas citadas por resposta na trilha do histórico.
const TOOL_TRAIL: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Conversation {
    pub id: Id,
    pub title: String,
    pub task: AiTask,
    pub last_engine_id: Option<Id>,
    pub created_at: String,
    pub updated_at: String,
    #[ts(type = "number")]
    pub message_count: i64,
    /// Fixada: não entra na faxina por idade.
    pub pinned: bool,
    /// Começada na página de um ativo (docs/asset-page.md): aparece lá.
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolCallBrief {
    pub name: String,
    /// Parâmetros resumidos (vazio nas mensagens gravadas antes da 0.4).
    #[serde(default)]
    pub args: String,
    pub summary: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Message {
    pub id: Id,
    /// `user` ou `assistant`.
    pub role: String,
    pub content: String,
    pub engine_id: Option<Id>,
    pub engine_name: Option<String>,
    pub tool_calls: Vec<ToolCallBrief>,
    pub proposal_ids: Vec<Id>,
    /// Raciocínio do modelo e passos intermediários (mostrados recolhidos; nunca voltam ao
    /// modelo como histórico).
    pub reasoning: Option<String>,
    /// Tempo até a resposta começar, em milissegundos.
    #[ts(type = "number | null")]
    pub reasoning_ms: Option<i64>,
    pub created_at: String,
    /// Anexos da mensagem (metadados; os bytes só se a retenção guardar).
    #[serde(default)]
    pub attachments: Vec<AttachmentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConversationDetail {
    #[serde(flatten)]
    pub conversation: Conversation,
    pub messages: Vec<Message>,
}

fn title_of(text: &str) -> String {
    let t: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() > 80 {
        format!("{}…", t.chars().take(79).collect::<String>())
    } else if t.is_empty() {
        "Conversa".into()
    } else {
        t
    }
}

pub fn create(conn: &Connection, task: AiTask, first_message: &str) -> Result<Id> {
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO ai_conversation (id, title, task, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, title_of(first_message), task.as_str(), now],
    )?;
    Ok(id)
}

pub fn exists(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM ai_conversation WHERE id = ?1)",
        [id],
        |r| r.get(0),
    )?)
}

pub struct NewMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
    pub engine_id: Option<&'a str>,
    pub engine_name: Option<&'a str>,
    pub tool_calls: &'a [ToolCallBrief],
    pub proposal_ids: &'a [Id],
    pub reasoning: Option<&'a str>,
    pub reasoning_ms: Option<i64>,
}

pub fn add(conn: &Connection, conversation_id: &str, m: &NewMessage<'_>) -> Result<Id> {
    let seq: i64 = conn.query_row(
        "SELECT COALESCE(MAX(seq), 0) + 1 FROM ai_message WHERE conversation_id = ?1",
        [conversation_id],
        |r| r.get(0),
    )?;
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO ai_message (id, conversation_id, seq, role, content, engine_id, engine_name, tool_calls, proposal_ids, created_at, reasoning, reasoning_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            id,
            conversation_id,
            seq,
            m.role,
            m.content,
            m.engine_id,
            m.engine_name,
            serde_json::to_string(m.tool_calls)?,
            serde_json::to_string(m.proposal_ids)?,
            now,
            m.reasoning.filter(|r| !r.is_empty()),
            m.reasoning_ms
        ],
    )?;
    conn.execute(
        "UPDATE ai_conversation SET updated_at = ?2, last_engine_id = COALESCE(?3, last_engine_id) WHERE id = ?1",
        params![conversation_id, now, m.engine_id],
    )?;
    Ok(id)
}

struct Stored {
    id: Id,
    seq: i64,
    role: String,
    content: String,
    tools: Vec<ToolCallBrief>,
}

/// Anexo de uma mensagem gravada (metadados).
struct StoredAttachment {
    id: Id,
    name: String,
    kind: attachments::AttachmentKind,
    pages: Option<u32>,
    text_chars: Option<i64>,
    stored: bool,
}

fn stored_attachments(conn: &Connection, message_id: &str) -> Result<Vec<StoredAttachment>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, kind, pages, length(text), data IS NOT NULL OR text IS NOT NULL
         FROM ai_attachment WHERE message_id = ?1 ORDER BY position",
    )?;
    Ok(stmt
        .query_map([message_id], |r| {
            Ok(StoredAttachment {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: attachments::AttachmentKind::parse(&r.get::<_, String>(2)?),
                pages: r.get(3)?,
                text_chars: r.get(4)?,
                stored: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

/// Corta no meio, mantendo o começo e o fim.
fn clip_middle(text: &str, max: usize, seq: i64) -> String {
    let n = text.chars().count();
    if n <= max {
        return text.to_string();
    }
    let head: String = text.chars().take(max * 2 / 3).collect();
    let tail: String = text.chars().skip(n - max / 3).collect();
    format!(
        "{head}\n[… {} characters omitted; the whole message #{seq} is in `read_chat_history` …]\n{tail}",
        n - head.chars().count() - tail.chars().count()
    )
}

fn first_chars(text: &str, max: usize) -> String {
    let t: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() <= max {
        t
    } else {
        format!("{}…", t.chars().take(max).collect::<String>())
    }
}

/// O que as ferramentas trouxeram na resposta, em uma linha cada (memória barata do que o
/// agente já consultou).
fn tool_trail(tools: &[ToolCallBrief]) -> Option<String> {
    if tools.is_empty() {
        return None;
    }
    let mut lines: Vec<String> = tools
        .iter()
        .take(TOOL_TRAIL)
        .map(|t| {
            let args = first_chars(&t.args, 80);
            let summary = first_chars(&t.summary, 160);
            let mark = if t.ok { "" } else { " (failed)" };
            if args.is_empty() {
                format!("{}{mark} → {summary}", t.name)
            } else {
                format!("{}({args}){mark} → {summary}", t.name)
            }
        })
        .collect();
    if tools.len() > TOOL_TRAIL {
        lines.push(format!("… and {} more", tools.len() - TOOL_TRAIL));
    }
    Some(format!("[tools used in this answer: {}]", lines.join("; ")))
}

/// Referência a um anexo que não volta inteiro ao motor.
fn attachment_reference(a: &StoredAttachment, seq: i64, available: bool) -> String {
    if !available {
        return attachments::placeholder(&a.name, a.kind);
    }
    let mut what = a.kind.label().to_string();
    if let Some(p) = a.pages {
        what.push_str(&format!(", {p} page(s)"));
    }
    if let Some(c) = a.text_chars.filter(|c| *c > 0) {
        what.push_str(&format!(", {}k characters", (c + 999) / 1000));
    }
    if a.kind == attachments::AttachmentKind::Image {
        format!(
            "[image sent earlier (message #{seq}): {} ({what}) — not resent to save space; if you need to see it again, ask the user to attach it again]",
            a.name
        )
    } else {
        format!(
            "[attachment sent earlier (message #{seq}): {} ({what}), id {} — its text was not resent to save space; read it with `read_attachment` (id or name) if needed]",
            a.name, a.id
        )
    }
}

/// Resumo das mensagens antigas (fora da janela das recentes), feito pelo app sem chamar o
/// motor: o começo de cada pedido e de cada resposta, com as ferramentas usadas.
fn summary_block(old: &[Stored]) -> Option<String> {
    if old.is_empty() {
        return None;
    }
    let mut lines: Vec<String> = old
        .iter()
        .map(|m| {
            if m.role == "user" {
                format!("#{} user: {}", m.seq, first_chars(&m.content, 280))
            } else {
                let tools: Vec<&str> = m.tools.iter().map(|t| t.name.as_str()).take(6).collect();
                let t = if tools.is_empty() {
                    String::new()
                } else {
                    format!(" [tools: {}]", tools.join(", "))
                };
                format!("#{} agent: {}{t}", m.seq, first_chars(&m.content, 400))
            }
        })
        .collect();
    // Os mais recentes do resumo têm prioridade quando passa do teto.
    let mut used = 0;
    let mut keep = lines.len();
    for (i, l) in lines.iter().enumerate().rev() {
        used += l.len() + 1;
        if used > SUMMARY_CHARS {
            break;
        }
        keep = i;
    }
    let dropped = keep;
    let mut kept = lines.split_off(keep);
    if dropped > 0 {
        kept.insert(
            0,
            format!("(… {dropped} older message(s) left out of the summary; read them with `read_chat_history`)"),
        );
    }
    Some(format!(
        "<historico_resumido>\nBeginning of this conversation, summarized by the app ({} messages; the numbers are the ones in `read_chat_history`, use it to read any of them in full). It is history, not a new request.\n{}\n</historico_resumido>",
        old.len(),
        kept.join("\n")
    ))
}

/// Contexto para o motor (todos os motores: APIs, locais e CLIs).
///
/// - As últimas [`RECENT_MESSAGES`] mensagens (até [`RECENT_CHARS`]) vão por inteiro (uma
///   muito longa vai cortada no meio); as respostas levam a trilha das ferramentas usadas.
/// - As anteriores viram um resumo (`<historico_resumido>`) no começo, com a fronteira
///   andando de [`BOUNDARY_STEP`] em [`BOUNDARY_STEP`] mensagens (prefixo estável para o
///   cache de prompt). O agente lê qualquer mensagem inteira com `read_chat_history`.
/// - Anexos: os da conversa que ainda estão na memória (`cached`) ou guardados no cofre
///   voltam inteiros quando recentes (texto e PDF: só da mensagem anterior; imagens: das
///   últimas [`attachments::HISTORY_ATTACHMENT_MESSAGES`]); os demais viram referência, e
///   o agente relê o texto com `read_attachment`.
pub fn context(
    conn: &Connection,
    conversation_id: &str,
    cached: &dyn Fn(&str) -> Option<std::sync::Arc<attachments::Attachment>>,
) -> Result<Vec<Msg>> {
    let mut stmt = conn.prepare(
        "SELECT id, seq, role, content, tool_calls FROM ai_message
          WHERE conversation_id = ?1 ORDER BY seq",
    )?;
    let all: Vec<Stored> = stmt
        .query_map([conversation_id], |r| {
            let tools: String = r.get(4)?;
            Ok(Stored {
                id: r.get(0)?,
                seq: r.get(1)?,
                role: r.get(2)?,
                content: r.get(3)?,
                tools: serde_json::from_str(&tools).unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    let n = all.len();
    // Fronteira estável: múltiplo de BOUNDARY_STEP; avança se as recentes passarem do teto.
    let mut b = n.saturating_sub(RECENT_MESSAGES) / BOUNDARY_STEP * BOUNDARY_STEP;
    let size = |from: usize| -> usize {
        all[from..]
            .iter()
            .map(|m| m.content.len().min(MESSAGE_CHARS * 4))
            .sum()
    };
    while size(b) > RECENT_CHARS && n - b > 2 {
        b = (b + BOUNDARY_STEP).min(n - 2);
    }
    // A parte inteira começa por uma mensagem do usuário.
    while b < n && all[b].role != "user" {
        b += 1;
    }
    let mut out: Vec<Msg> = vec![];
    for (i, m) in all[b..].iter().enumerate() {
        let dist = n - b - i; // 1 = a última gravada
        let text = clip_middle(&m.content, MESSAGE_CHARS, m.seq);
        if m.role != "user" {
            let mut t = text;
            if let Some(trail) = tool_trail(&m.tools) {
                t.push_str("\n\n");
                t.push_str(&trail);
            }
            out.push(Msg::assistant(t));
            continue;
        }
        // Mensagem só com anexos: um texto curto (as APIs recusam bloco de texto vazio).
        let text = if text.trim().is_empty() {
            "(mensagem sem texto, só com anexos)".to_string()
        } else {
            text
        };
        let mut msg = Msg::user(text);
        let atts = stored_attachments(conn, &m.id)?;
        if !atts.is_empty() {
            let mut blocks = vec![];
            for a in &atts {
                let window = if a.kind == attachments::AttachmentKind::Image {
                    attachments::HISTORY_ATTACHMENT_MESSAGES
                } else {
                    FULL_TEXT_ATTACHMENT_MESSAGES
                };
                let full = if dist <= window {
                    cached(&a.id).or_else(|| {
                        a.stored
                            .then(|| {
                                attachments::load_for_message(conn, &m.id, true)
                                    .ok()?
                                    .into_iter()
                                    .find_map(|s| match s {
                                        Saved::Stored(x) if x.id == a.id => Some(x),
                                        _ => None,
                                    })
                            })
                            .flatten()
                    })
                } else {
                    None
                };
                blocks.push(match full {
                    Some(x) => Block::Attachment(x),
                    None => Block::Text(attachment_reference(
                        a,
                        m.seq,
                        a.stored || cached(&a.id).is_some(),
                    )),
                });
            }
            blocks.append(&mut msg.blocks);
            msg.blocks = blocks;
        }
        out.push(msg);
    }
    if let Some(summary) = summary_block(&all[..b])
        && let Some(first) = out.first_mut()
    {
        first.blocks.insert(0, Block::Text(summary));
    }
    // A conversa precisa começar pelo usuário.
    while out
        .first()
        .is_some_and(|m| m.role == super::providers::Role::Assistant)
    {
        out.remove(0);
    }
    Ok(out)
}

fn conv_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    let task: String = r.get(2)?;
    Ok(Conversation {
        id: r.get(0)?,
        title: r.get(1)?,
        task: AiTask::parse(&task).unwrap_or(AiTask::Chat),
        last_engine_id: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
        message_count: r.get(6)?,
        pinned: r.get::<_, i64>(7)? == 1,
        asset_id: r.get(8)?,
    })
}

const CONV_SELECT: &str =
    "SELECT c.id, c.title, c.task, c.last_engine_id, c.created_at, c.updated_at,
    (SELECT count(*) FROM ai_message m WHERE m.conversation_id = c.id), c.pinned, c.asset_id
    FROM ai_conversation c";

pub fn list(conn: &Connection, limit: i64) -> Result<Vec<Conversation>> {
    let mut stmt = conn.prepare(&format!(
        "{CONV_SELECT} ORDER BY c.pinned DESC, c.updated_at DESC LIMIT ?1"
    ))?;
    Ok(stmt
        .query_map([limit.clamp(1, 500)], conv_row)?
        .collect::<rusqlite::Result<_>>()?)
}

/// As conversas começadas na página de um ativo, as mais recentes primeiro.
pub fn list_for_asset(conn: &Connection, asset_id: &str, limit: i64) -> Result<Vec<Conversation>> {
    let mut stmt = conn.prepare(&format!(
        "{CONV_SELECT} WHERE c.asset_id = ?1 ORDER BY c.updated_at DESC LIMIT ?2"
    ))?;
    Ok(stmt
        .query_map(params![asset_id, limit.clamp(1, 500)], conv_row)?
        .collect::<rusqlite::Result<_>>()?)
}

/// Liga a conversa ao ativo (só se ainda não for de outro ativo e o ativo existir).
pub fn link_asset(conn: &Connection, conversation_id: &str, asset_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE ai_conversation SET asset_id = ?2
         WHERE id = ?1 AND asset_id IS NULL AND EXISTS (SELECT 1 FROM asset WHERE id = ?2)",
        params![conversation_id, asset_id],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<ConversationDetail> {
    let conversation = conn
        .query_row(&format!("{CONV_SELECT} WHERE c.id = ?1"), [id], conv_row)
        .optional()?
        .ok_or_else(|| Error::not_found(crate::i18n::t("nucleo-ia-conversa")))?;
    let mut stmt = conn.prepare(
        "SELECT id, role, content, engine_id, engine_name, tool_calls, proposal_ids, created_at, reasoning, reasoning_ms
         FROM ai_message WHERE conversation_id = ?1 ORDER BY seq",
    )?;
    let mut atts = attachments::infos_for_conversation(conn, id)?;
    let messages = stmt
        .query_map([id], |r| {
            let tools: String = r.get(5)?;
            let props: String = r.get(6)?;
            Ok(Message {
                id: r.get(0)?,
                role: r.get(1)?,
                content: r.get(2)?,
                engine_id: r.get(3)?,
                engine_name: r.get(4)?,
                tool_calls: serde_json::from_str(&tools).unwrap_or_default(),
                proposal_ids: serde_json::from_str(&props).unwrap_or_default(),
                created_at: r.get(7)?,
                reasoning: r.get(8)?,
                reasoning_ms: r.get(9)?,
                attachments: vec![],
            })
        })?
        .collect::<rusqlite::Result<Vec<Message>>>()?
        .into_iter()
        .map(|mut m| {
            m.attachments = atts.remove(&m.id).unwrap_or_default();
            m
        })
        .collect();
    Ok(ConversationDetail {
        conversation,
        messages,
    })
}

/// Fixa ou solta uma conversa (fixadas não entram na faxina por idade).
pub fn set_pinned(conn: &Connection, id: &str, pinned: bool) -> Result<()> {
    if conn.execute(
        "UPDATE ai_conversation SET pinned = ?2 WHERE id = ?1",
        params![id, pinned as i64],
    )? == 0
    {
        return Err(Error::not_found(crate::i18n::t("nucleo-ia-conversa")));
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    if conn.execute("DELETE FROM ai_conversation WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(crate::i18n::t("nucleo-ia-conversa")));
    }
    Ok(())
}

pub fn clear(conn: &Connection) -> Result<i64> {
    Ok(conn.execute("DELETE FROM ai_conversation", [])? as i64)
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct ChatHistoryArgs {
    /// Conversa (o agente do app recebe a dele; não dá para escolher outra).
    #[serde(default)]
    #[ts(optional)]
    pub conversation_id: Option<Id>,
    /// Texto a procurar (sem diferenciar maiúsculas e acentos).
    #[serde(default)]
    #[ts(optional)]
    pub query: Option<String>,
    /// Faixa de mensagens pelo número (`#`).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub from_seq: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub to_seq: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatHistoryMessage {
    #[ts(type = "number")]
    pub seq: i64,
    pub role: String,
    pub created_at: String,
    pub content: String,
    /// O texto foi cortado (peça a mensagem sozinha com `from_seq` = `to_seq`).
    pub truncated: bool,
    pub tool_calls: Vec<ToolCallBrief>,
    /// Anexos da mensagem (nome e id; o texto com `read_attachment`).
    pub attachments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatHistory {
    #[ts(type = "number")]
    pub total_messages: i64,
    pub messages: Vec<ChatHistoryMessage>,
}

/// Mensagens gravadas da conversa, por faixa de números ou por texto (para o agente reler o
/// que saiu do contexto).
pub fn history(conn: &Connection, a: &ChatHistoryArgs) -> Result<ChatHistory> {
    let Some(cid) = a.conversation_id.as_deref() else {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ia-conversa-sem-historico",
        )));
    };
    if !exists(conn, cid)? {
        return Err(Error::not_found(crate::i18n::t("nucleo-ia-conversa")));
    }
    let limit = a.limit.unwrap_or(20).clamp(1, 50) as usize;
    let mut stmt = conn.prepare(
        "SELECT id, seq, role, content, tool_calls, created_at FROM ai_message
          WHERE conversation_id = ?1 AND seq >= ?2 AND seq <= ?3 ORDER BY seq",
    )?;
    let rows: Vec<(Id, i64, String, String, String, String)> = stmt
        .query_map(
            params![cid, a.from_seq.unwrap_or(0), a.to_seq.unwrap_or(i64::MAX)],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )?
        .collect::<rusqlite::Result<_>>()?;
    let total: i64 = conn.query_row(
        "SELECT count(*) FROM ai_message WHERE conversation_id = ?1",
        [cid],
        |r| r.get(0),
    )?;
    let q = a
        .query
        .as_deref()
        .map(crate::import::text::fold)
        .filter(|q| !q.trim().is_empty());
    let mut atts = attachments::infos_for_conversation(conn, cid)?;
    let picked: Vec<_> = rows
        .into_iter()
        .filter(|r| {
            q.as_ref()
                .is_none_or(|q| crate::import::text::fold(&r.3).contains(q.as_str()))
        })
        .collect();
    // Sem faixa nem busca: as mais recentes.
    let skip = if a.from_seq.is_none() && q.is_none() {
        picked.len().saturating_sub(limit)
    } else {
        0
    };
    let single = picked.len() == 1;
    let max = if single { 40_000 } else { 6_000 };
    Ok(ChatHistory {
        total_messages: total,
        messages: picked
            .into_iter()
            .skip(skip)
            .take(limit)
            .map(|(id, seq, role, content, tools, created_at)| {
                let truncated = content.chars().count() > max;
                ChatHistoryMessage {
                    seq,
                    role,
                    created_at,
                    content: if truncated {
                        format!("{}…", content.chars().take(max).collect::<String>())
                    } else {
                        content
                    },
                    truncated,
                    tool_calls: serde_json::from_str(&tools).unwrap_or_default(),
                    attachments: atts
                        .remove(&id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|a| format!("{} (id {})", a.name, a.id))
                        .collect(),
                }
            })
            .collect(),
    })
}
