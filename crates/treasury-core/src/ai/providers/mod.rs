//! Adaptadores dos motores de API e modelos locais: Anthropic Messages (tool use), OpenAI
//! Chat Completions (function calling; serve OpenAI, xAI, Mistral, OpenRouter, endpoints
//! compatíveis, Ollama e LM Studio) e Gemini (function declarations). Todos em streaming
//! (SSE lido linha a linha com `ureq`), sem segurar o banco: o chamador já leu o que
//! precisava e grava o uso depois.
//!
//! O streaming entrega dois tipos de trecho ([`Delta`]): texto da resposta e raciocínio do
//! modelo (blocos `thinking` da Anthropic, `reasoning_content`/`reasoning` dos compatíveis
//! com OpenAI, `<think>…</think>` de modelos locais e resumos de pensamento do Gemini). Quem
//! não manda raciocínio só manda texto; a interface mostra o raciocínio recolhido.

mod anthropic;
mod gemini;
mod openai;

#[cfg(test)]
pub(crate) use gemini::schema as gemini_schema;

use std::io::{BufRead, BufReader, Read};
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use super::attachments::{self, Attachment};
use super::engine::{Protocol, Provider};
use super::usage::Tokens;

/// Ferramenta oferecida ao modelo (nome, descrição, esquema JSON da entrada).
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub schema: Value,
}

/// Trecho do streaming de um turno.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delta<'a> {
    /// Texto da resposta.
    Text(&'a str),
    /// Raciocínio do modelo (mostrado recolhido; nunca vai para o histórico da conversa).
    Reasoning(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Text(String),
    /// Raciocínio do modelo. Na Anthropic volta no turno seguinte com a assinatura (exigida
    /// quando há ferramentas); nos outros providers não é reenviado.
    Thinking {
        text: String,
        signature: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        id: String,
        name: String,
        content: String,
        is_error: bool,
    },
    /// Imagem nativa (motores que leem imagens; ver `ai::attachments::resolve`).
    Image {
        mime: String,
        data: Arc<Vec<u8>>,
        width: Option<u32>,
        height: Option<u32>,
    },
    /// PDF nativo (Anthropic, Gemini, OpenAI).
    Document {
        mime: String,
        name: String,
        data: Arc<Vec<u8>>,
        pages: Option<u32>,
    },
    /// Anexo ainda não convertido para o motor: [`super::attachments::resolve`] troca por
    /// texto, imagem ou documento antes do turno. Se chegar a um adaptador, vira um aviso.
    Attachment(Arc<Attachment>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Msg {
    pub role: Role,
    pub blocks: Vec<Block>,
    /// Partes originais do provider (Gemini: assinaturas de raciocínio que precisam voltar).
    pub raw: Option<Value>,
}

impl Msg {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            blocks: vec![Block::Text(text.into())],
            raw: None,
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            blocks: vec![Block::Text(text.into())],
            raw: None,
        }
    }

    pub fn text(&self) -> String {
        self.blocks
            .iter()
            .filter_map(|b| match b {
                Block::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Um turno: o histórico inteiro vai a cada chamada.
pub struct Turn<'a> {
    pub model: &'a str,
    pub system: &'a str,
    pub messages: &'a [Msg],
    pub tools: &'a [ToolDef],
    pub max_tokens: u32,
    /// Esforço de raciocínio escolhido na conversa (ver [`crate::ai::effort`]).
    pub effort: crate::ai::effort::Effort,
}

#[derive(Debug, Clone, Default)]
pub struct TurnOutput {
    /// Texto e chamadas de ferramenta do modelo.
    pub blocks: Vec<Block>,
    pub tokens: Tokens,
    /// O provider informou o uso (senão, `tokens` é estimado).
    pub usage_reported: bool,
    pub raw: Option<Value>,
}

impl TurnOutput {
    pub fn tool_uses(&self) -> Vec<(String, String, Value)> {
        self.blocks
            .iter()
            .filter_map(|b| match b {
                Block::ToolUse { id, name, input } => {
                    Some((id.clone(), name.clone(), input.clone()))
                }
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrKind {
    /// Chave inválida ou sem permissão.
    Auth,
    /// Limite de requisições (429).
    RateLimit,
    /// Sem créditos/cota.
    Quota,
    Network,
    Server,
    /// Requisição recusada (modelo inexistente, parâmetro…).
    Invalid,
    Cancelled,
    /// Resposta fora do formato esperado.
    Protocol,
}

#[derive(Debug, Clone)]
pub struct ProviderError {
    pub kind: ErrKind,
    pub message: String,
    /// Segundos até o limite liberar (cabeçalho `retry-after` de um 429).
    pub retry_after: Option<u64>,
}

impl ProviderError {
    pub fn new(kind: ErrKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn code(&self) -> &'static str {
        match self.kind {
            ErrKind::Auth => "auth",
            ErrKind::RateLimit => "rate_limit",
            ErrKind::Quota => "quota",
            ErrKind::Network => "network",
            ErrKind::Server => "server",
            ErrKind::Invalid => "invalid",
            ErrKind::Cancelled => "cancelled",
            ErrKind::Protocol => "protocol",
        }
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Onde e como chamar.
pub struct Endpoint<'a> {
    pub provider: Provider,
    pub base_url: &'a str,
    pub api_key: Option<&'a str>,
    pub cancelled: &'a (dyn Fn() -> bool + Sync),
}

/// Um turno no protocolo do provider. `on_delta` recebe o texto e o raciocínio parciais.
pub fn turn(
    ep: &Endpoint<'_>,
    t: &Turn<'_>,
    on_delta: &mut dyn FnMut(Delta<'_>),
) -> Result<TurnOutput, ProviderError> {
    let mut out = match ep.provider.protocol() {
        Protocol::AnthropicMessages => anthropic::turn(ep, t, on_delta),
        Protocol::OpenaiChat => openai::turn(ep, t, on_delta),
        Protocol::GeminiGenerate => gemini::turn(ep, t, on_delta),
        Protocol::CliStreamJson | Protocol::CliText | Protocol::CliOpencodeJson => Err(
            ProviderError::new(ErrKind::Invalid, "CLIs não usam o adaptador de API"),
        ),
    }?;
    if !out.usage_reported {
        let chars: usize = t.system.len()
            + t.messages
                .iter()
                .flat_map(|m| &m.blocks)
                .map(block_len)
                .sum::<usize>();
        out.tokens.input = super::usage::approx_tokens(chars);
        out.tokens.output =
            super::usage::approx_tokens(out.blocks.iter().map(block_len).sum::<usize>());
    }
    Ok(out)
}

/// Tamanho de um bloco em "caracteres" para a estimativa de tokens (imagens e PDFs pela
/// estimativa de tokens de cada um, × 4).
pub(crate) fn block_len(b: &Block) -> usize {
    match b {
        Block::Text(t) => t.len(),
        Block::Thinking { text, .. } => text.len(),
        Block::ToolUse { input, name, .. } => name.len() + input.to_string().len(),
        Block::ToolResult { content, .. } => content.len(),
        Block::Image { .. } | Block::Document { .. } | Block::Attachment(_) => {
            attachments::approx_block_tokens(b) as usize * 4
        }
    }
}

/// Texto de um anexo que chegou sem ser convertido (não deveria acontecer).
pub(crate) fn unresolved(a: &Attachment) -> String {
    format!("[Anexo {}: não enviado a este motor.]", a.name)
}

/// `data:<tipo>;base64,<dados>` (OpenAI e compatíveis).
pub(crate) fn data_url(mime: &str, data: &[u8]) -> String {
    format!("data:{mime};base64,{}", attachments::b64(data))
}

// ---------------------------------------------------------------------------
// `<think>…</think>` no texto (modelos locais)

/// Separa o raciocínio que alguns modelos locais (Qwen, DeepSeek R1 e derivados) escrevem
/// no próprio texto entre `<think>` e `</think>`. Funciona em streaming: uma marca cortada
/// entre dois pedaços fica guardada até o pedaço seguinte.
#[derive(Debug, Default)]
pub(crate) struct ThinkSplitter {
    inside: bool,
    pending: String,
}

const THINK_OPEN: &str = "<think>";
const THINK_CLOSE: &str = "</think>";

impl ThinkSplitter {
    /// Processa um pedaço e entrega os trechos na ordem.
    pub fn push(&mut self, chunk: &str, out: &mut dyn FnMut(Delta<'_>)) {
        self.pending.push_str(chunk);
        loop {
            let tag = if self.inside { THINK_CLOSE } else { THINK_OPEN };
            if let Some(i) = self.pending.find(tag) {
                let before: String = self.pending[..i].to_string();
                self.emit(&before, out);
                self.pending.drain(..i + tag.len());
                self.inside = !self.inside;
                continue;
            }
            // Guarda um possível começo de marca no fim do pedaço.
            let keep = (1..tag.len())
                .rev()
                .find(|&n| self.pending.len() >= n && self.pending.ends_with(&tag[..n]))
                .unwrap_or(0);
            let cut = self.pending.len() - keep;
            let ready: String = self.pending[..cut].to_string();
            self.emit(&ready, out);
            self.pending.drain(..cut);
            return;
        }
    }

    /// Fim do streaming: o que sobrou sai como está.
    pub fn finish(&mut self, out: &mut dyn FnMut(Delta<'_>)) {
        let rest = std::mem::take(&mut self.pending);
        self.emit(&rest, out);
    }

    fn emit(&self, s: &str, out: &mut dyn FnMut(Delta<'_>)) {
        if s.is_empty() {
            return;
        }
        out(if self.inside {
            Delta::Reasoning(s)
        } else {
            Delta::Text(s)
        });
    }
}

// ---------------------------------------------------------------------------
// HTTP em streaming

const USER_AGENT: &str = concat!("Treasury/", env!("CARGO_PKG_VERSION"));

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_recv_response(Some(Duration::from_secs(180)))
        .timeout_recv_body(Some(Duration::from_secs(900)))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

/// Tira a chave de qualquer texto que possa voltar ao usuário (mensagens de erro).
fn scrub(msg: &str, key: Option<&str>) -> String {
    let mut s: String = msg.chars().take(400).collect();
    if let Some(k) = key.filter(|k| k.len() >= 6) {
        s = s.replace(k, "***");
    }
    s
}

fn classify(status: u16, body: &str) -> ErrKind {
    let lower = body.to_ascii_lowercase();
    match status {
        401 | 403 => ErrKind::Auth,
        402 => ErrKind::Quota,
        429 if lower.contains("quota") || lower.contains("credit") || lower.contains("billing") => {
            ErrKind::Quota
        }
        429 => ErrKind::RateLimit,
        500..=599 => ErrKind::Server,
        400 if lower.contains("credit") || lower.contains("billing") => ErrKind::Quota,
        _ => ErrKind::Invalid,
    }
}

/// Extrai a mensagem de erro dos formatos comuns (`{"error":{"message":…}}`, `{"error":"…"}`,
/// `{"message":…}`).
fn error_message(body: &str) -> String {
    let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let v = if let Value::Array(a) = &v {
        a.first().cloned().unwrap_or(Value::Null)
    } else {
        v
    };
    v.pointer("/error/message")
        .or_else(|| v.get("error"))
        .or_else(|| v.get("message"))
        .and_then(|m| m.as_str().map(str::to_string))
        .unwrap_or_else(|| body.trim().chars().take(200).collect())
}

/// POST JSON e entrega cada linha da resposta (SSE ou NDJSON) a `on_line`.
fn post_lines(
    ep: &Endpoint<'_>,
    url: &str,
    headers: &[(&str, String)],
    body: &Value,
    on_line: &mut dyn FnMut(&str) -> Result<(), ProviderError>,
) -> Result<(), ProviderError> {
    if (ep.cancelled)() {
        return Err(ProviderError::new(
            ErrKind::Cancelled,
            crate::i18n::t("nucleo-ia-agente-cancelado"),
        ));
    }
    let mut req = agent()
        .post(url)
        .header("Accept", "text/event-stream, application/json");
    for (k, v) in headers {
        req = req.header(*k, v.as_str());
    }
    let payload = serde_json::to_vec(body)
        .map_err(|e| ProviderError::new(ErrKind::Protocol, e.to_string()))?;
    let resp = req
        .content_type("application/json")
        .send(&payload[..])
        .map_err(|e| {
            ProviderError::new(
                ErrKind::Network,
                scrub(
                    &crate::i18n::t_args(
                        "nucleo-ia-provider-rede",
                        &crate::i18n::args(&[("detalhe", e.to_string().into())]),
                    ),
                    ep.api_key,
                ),
            )
        })?;
    let status = resp.status().as_u16();
    let retry_after = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|s| s.ceil().max(1.0) as u64);
    let reader = resp.into_body().into_reader();
    if !(200..300).contains(&status) {
        let mut text = String::new();
        let _ = reader.take(64 * 1024).read_to_string(&mut text);
        let kind = classify(status, &text);
        return Err(ProviderError {
            retry_after,
            ..ProviderError::new(
                kind,
                scrub(
                    &crate::i18n::t_args(
                        "nucleo-ia-provider-respondeu",
                        &crate::i18n::args(&[
                            ("provider", ep.provider.label().into()),
                            ("status", status.to_string().into()),
                            ("mensagem", error_message(&text).to_string().into()),
                        ]),
                    ),
                    ep.api_key,
                ),
            )
        });
    }
    let mut lines = BufReader::new(reader.take(256 * 1024 * 1024));
    let mut line = String::new();
    loop {
        if (ep.cancelled)() {
            return Err(ProviderError::new(
                ErrKind::Cancelled,
                crate::i18n::t("nucleo-ia-agente-cancelado"),
            ));
        }
        line.clear();
        match lines.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => on_line(line.trim_end_matches(['\r', '\n']))?,
            Err(e) => {
                return Err(ProviderError::new(
                    ErrKind::Network,
                    scrub(
                        &crate::i18n::t_args(
                            "nucleo-ia-provider-conexao",
                            &crate::i18n::args(&[("detalhe", e.to_string().into())]),
                        ),
                        ep.api_key,
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Dados de uma linha SSE (`data: …`), ignorando comentários e `[DONE]`.
fn sse_data(line: &str) -> Option<&str> {
    let d = line.strip_prefix("data:")?.trim_start();
    (d != "[DONE]" && !d.is_empty()).then_some(d)
}

fn parse_json(s: &str) -> Result<Value, ProviderError> {
    serde_json::from_str(s).map_err(|e| {
        ProviderError::new(
            ErrKind::Protocol,
            crate::i18n::t_args(
                "nucleo-ia-provider-resposta-invalida",
                &crate::i18n::args(&[("detalhe", e.to_string().into())]),
            ),
        )
    })
}

fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// Corpo da requisição de um turno no formato do provider (testes).
#[cfg(test)]
pub(crate) fn body_for_tests(provider: Provider, t: &Turn<'_>) -> Value {
    match provider.protocol() {
        Protocol::AnthropicMessages => anthropic::body(t),
        Protocol::GeminiGenerate => gemini::body(t),
        _ => openai::body(provider, t),
    }
}

#[cfg(test)]
pub(crate) mod fake {
    //! Servidor HTTP falso (uma thread com `TcpListener`) para os testes dos adaptadores.

    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    pub struct Captured {
        pub path: String,
        pub headers: Vec<(String, String)>,
        pub body: serde_json::Value,
    }

    impl Captured {
        pub fn header(&self, k: &str) -> Option<&str> {
            self.headers
                .iter()
                .find(|(h, _)| h.eq_ignore_ascii_case(k))
                .map(|(_, v)| v.as_str())
        }
    }

    pub struct FakeServer {
        pub url: String,
        pub requests: Arc<Mutex<Vec<Captured>>>,
    }

    /// Responde, em ordem, `(status, content-type, corpo)`; depois da lista, 500.
    pub fn serve(responses: Vec<(u16, &'static str, String)>) -> FakeServer {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(vec![]));
        let reqs = requests.clone();
        std::thread::spawn(move || {
            let mut queue = responses.into_iter();
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                let mut rd = BufReader::new(stream.try_clone().unwrap());
                let mut first = String::new();
                if rd.read_line(&mut first).is_err() {
                    continue;
                }
                let path = first.split_whitespace().nth(1).unwrap_or("").to_string();
                let mut headers = vec![];
                let mut len = 0usize;
                loop {
                    let mut l = String::new();
                    if rd.read_line(&mut l).unwrap_or(0) == 0 {
                        break;
                    }
                    let l = l.trim_end().to_string();
                    if l.is_empty() {
                        break;
                    }
                    if let Some((k, v)) = l.split_once(':') {
                        if k.eq_ignore_ascii_case("content-length") {
                            len = v.trim().parse().unwrap_or(0);
                        }
                        headers.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
                let mut body = vec![0u8; len];
                let _ = rd.read_exact(&mut body);
                reqs.lock().unwrap().push(Captured {
                    path,
                    headers,
                    body: serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
                });
                let (status, ct, body) = queue.next().unwrap_or((
                    500,
                    "application/json",
                    "{\"error\":{\"message\":\"sem resposta\"}}".into(),
                ));
                // O 429 vem com `Retry-After` de 1 s (como o do Groq, em segundos).
                let retry = if status == 429 {
                    "Retry-After: 1\r\n"
                } else {
                    ""
                };
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status} X\r\nContent-Type: {ct}\r\n{retry}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.flush();
            }
        });
        FakeServer { url, requests }
    }

    /// Corpo SSE a partir de pares (evento, JSON).
    pub fn sse(events: &[(Option<&str>, serde_json::Value)]) -> String {
        let mut s = String::new();
        for (ev, data) in events {
            if let Some(e) = ev {
                s.push_str(&format!("event: {e}\n"));
            }
            s.push_str(&format!("data: {data}\n\n"));
        }
        s
    }
}

#[cfg(test)]
mod tests;
