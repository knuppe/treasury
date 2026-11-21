//! Leitura de uma mensagem (.eml ou o que o IMAP baixou), com `mail-parser`: os cabeçalhos
//! (assunto decodificado da RFC 2047, remetente, destinatários, data, Message-ID), o
//! Authentication-Results, as peças (anexos e o .eml anexado, até um nível) e o corpo como
//! texto (HTML vira texto, sem rede). Limites: 25 MB por mensagem, 20 anexos, nada compactado.
//! O nome do arquivo de um anexo nunca vira caminho.

use mail_parser::{Address, MessageParser, MimeHeaders};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::filter::clean_subject;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

pub const MAX_MESSAGE_BYTES: usize = 25 * 1024 * 1024;
pub const MAX_ATTACHMENTS: usize = 20;
const MAX_BODY_CHARS: usize = 200_000;
const FORWARD_PREFIXES: [&str; 5] = ["fwd:", "fw:", "enc:", "tr:", "encaminhada:"];

/// Uma peça da mensagem: um anexo (ou o corpo, quando ele mesmo é o documento).
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub filename: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailHeaders {
    pub from: String,
    pub from_name: Option<String>,
    pub to: Vec<String>,
    pub subject: String,
    /// `YYYY-MM-DD`.
    pub date: Option<String>,
    pub message_id: Option<String>,
    /// Valores do Authentication-Results, de cima para baixo.
    pub auth_results: Vec<String>,
    /// O assunto original começa com `Fwd:`/`Enc:`/`RES:` (encaminhada, provavelmente).
    #[serde(skip)]
    #[ts(skip)]
    pub forward_hint: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Mail {
    pub headers: MailHeaders,
    /// Encaminhada (assunto `Fwd:`/`Enc:`, bloco "Mensagem encaminhada" ou .eml anexado).
    pub forwarded: bool,
    /// Remetente original de uma encaminhada (quando aparece no corpo ou no .eml anexado).
    pub original_from: Option<String>,
    pub pieces: Vec<Piece>,
    pub body: String,
    pub warnings: Vec<String>,
}

fn first_addr(a: Option<&Address<'_>>) -> (String, Option<String>) {
    match a.and_then(|a| a.first()) {
        Some(x) => (
            x.address().unwrap_or_default().trim().to_ascii_lowercase(),
            x.name()
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty()),
        ),
        None => (String::new(), None),
    }
}

fn all_addrs(a: Option<&Address<'_>>) -> Vec<String> {
    a.map(|a| {
        a.iter()
            .filter_map(|x| x.address().map(|s| s.trim().to_ascii_lowercase()))
            .collect()
    })
    .unwrap_or_default()
}

fn date_of(d: Option<&mail_parser::DateTime>) -> Option<String> {
    d.map(|d| format!("{:04}-{:02}-{:02}", d.year, d.month, d.day))
}

fn headers_of(m: &mail_parser::Message<'_>) -> MailHeaders {
    let (from, from_name) = first_addr(m.from());
    let auth_results = m
        .headers()
        .iter()
        .filter(|h| {
            h.name
                .as_str()
                .eq_ignore_ascii_case("Authentication-Results")
        })
        .filter_map(|h| {
            std::str::from_utf8(&m.raw_message()[h.offset_start as usize..h.offset_end as usize])
                .ok()
                .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        })
        .collect();
    let raw_subject = m.subject().unwrap_or_default().trim().to_ascii_lowercase();
    MailHeaders {
        from,
        from_name,
        to: all_addrs(m.to()),
        subject: clean_subject(m.subject().unwrap_or_default()),
        date: date_of(m.date()),
        message_id: m.message_id().map(|s| s.trim().to_string()),
        auth_results,
        forward_hint: FORWARD_PREFIXES.iter().any(|p| raw_subject.starts_with(p)),
    }
}

/// Só os cabeçalhos (o IMAP baixa só eles antes dos filtros).
pub fn parse_headers(raw: &[u8]) -> Option<MailHeaders> {
    MessageParser::default()
        .parse_headers(raw)
        .map(|m| headers_of(&m))
}

/// Nome de arquivo seguro: só o último componente, sem caracteres de controle nem separadores.
pub fn safe_name(name: &str, fallback: &str) -> String {
    let last = name.rsplit(['/', '\\']).next().unwrap_or("");
    let clean: String = last
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect::<String>()
        .trim()
        .trim_start_matches('.')
        .chars()
        .take(120)
        .collect();
    if clean.is_empty() {
        fallback.to_string()
    } else {
        clean
    }
}

fn compressed(name: &str, mime: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [".zip", ".rar", ".7z", ".gz", ".tgz", ".tar", ".bz2", ".xz"]
        .iter()
        .any(|e| n.ends_with(e))
        || ["zip", "x-rar", "x-7z", "gzip", "x-tar", "x-bzip"]
            .iter()
            .any(|t| mime.contains(t))
}

/// "De:"/"From:" dentro de um bloco de mensagem encaminhada no corpo.
fn forwarded_from(body: &str) -> Option<String> {
    let low = body.to_lowercase();
    let start = [
        "forwarded message",
        "mensagem encaminhada",
        "mensaje reenviado",
        "original message",
        "mensagem original",
    ]
    .iter()
    .filter_map(|m| low.find(m))
    .min()?;
    let block: String = body[start..].chars().take(2000).collect();
    for line in block.lines().take(12) {
        let l = line.trim();
        let ll = l.to_lowercase();
        if ll.starts_with("from:") || ll.starts_with("de:") {
            return email_in(l);
        }
    }
    None
}

fn email_in(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").ok()?;
    re.find(s).map(|m| m.as_str().to_ascii_lowercase())
}

/// Lê a mensagem inteira.
pub fn parse(raw: &[u8]) -> Result<Mail> {
    if raw.len() > MAX_MESSAGE_BYTES {
        return Err(Error::invalid(t("nucleo-email-mensagem-erro-grande")));
    }
    let m = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| Error::invalid(t("nucleo-email-mensagem-erro-ler")))?;
    let mut mail = Mail {
        headers: headers_of(&m),
        ..Default::default()
    };
    mail.forwarded = mail.headers.forward_hint;
    mail.body = m
        .body_text(0)
        .map(|b| b.chars().take(MAX_BODY_CHARS).collect())
        .unwrap_or_default();
    // Imagens do próprio corpo (logotipos, ícones, pixel de rastreio) não são peças.
    let html = m.body_html(0).map(|h| h.to_lowercase()).unwrap_or_default();
    let decorative = |part: &mail_parser::MessagePart<'_>, mime: &str| -> bool {
        if !mime.starts_with("image/") {
            return false;
        }
        let size = part.contents().len();
        let referenced = part
            .content_id()
            .map(|c| c.trim_matches(['<', '>']).to_lowercase())
            .is_some_and(|c| !c.is_empty() && html.contains(&format!("cid:{c}")));
        size < 12 * 1024 || (referenced && size < 300 * 1024)
    };
    let mut count = 0;
    let mut add = |mail: &mut Mail, filename: String, mime: String, bytes: &[u8]| {
        if compressed(&filename, &mime) {
            mail.warnings.push(t_args(
                "nucleo-email-mensagem-aviso-compactado",
                &args(&[("arquivo", filename.into())]),
            ));
            return;
        }
        count += 1;
        if count > MAX_ATTACHMENTS {
            if count == MAX_ATTACHMENTS + 1 {
                mail.warnings.push(t_args(
                    "nucleo-email-mensagem-aviso-anexos",
                    &args(&[("max", MAX_ATTACHMENTS.into())]),
                ));
            }
            return;
        }
        mail.pieces.push(Piece {
            filename,
            mime,
            bytes: bytes.to_vec(),
        });
    };
    for (i, part) in m.attachments().enumerate() {
        let mime = part
            .content_type()
            .map(|c| {
                format!(
                    "{}/{}",
                    c.c_type,
                    c.c_subtype.as_deref().unwrap_or("octet-stream")
                )
                .to_ascii_lowercase()
            })
            .unwrap_or_else(|| "application/octet-stream".into());
        if let Some(inner) = part.message() {
            // .eml anexado: um nível só (os anexos dele entram; os .eml dele, não).
            mail.forwarded = true;
            let (orig, _) = first_addr(inner.from());
            if !orig.is_empty() {
                mail.original_from.get_or_insert(orig);
            }
            if mail.body.trim().is_empty()
                && let Some(b) = inner.body_text(0)
            {
                mail.body = b.chars().take(MAX_BODY_CHARS).collect();
            }
            for (j, ip) in inner.attachments().enumerate() {
                if ip.message().is_some() {
                    mail.warnings
                        .push(t("nucleo-email-mensagem-aviso-aninhada"));
                    continue;
                }
                let imime = ip
                    .content_type()
                    .map(|c| {
                        format!(
                            "{}/{}",
                            c.c_type,
                            c.c_subtype.as_deref().unwrap_or("octet-stream")
                        )
                        .to_ascii_lowercase()
                    })
                    .unwrap_or_else(|| "application/octet-stream".into());
                let name = safe_name(
                    ip.attachment_name().unwrap_or_default(),
                    &format!("anexo-{}-{}", i + 1, j + 1),
                );
                add(&mut mail, name, imime, ip.contents());
            }
            continue;
        }
        if decorative(part, &mime) {
            continue;
        }
        let name = safe_name(
            part.attachment_name().unwrap_or_default(),
            &format!("anexo-{}", i + 1),
        );
        add(&mut mail, name, mime, part.contents());
    }
    if mail.original_from.is_none()
        && let Some(f) = forwarded_from(&mail.body)
    {
        mail.forwarded = true;
        mail.original_from = Some(f);
    }
    if mail.original_from.as_deref() == Some(mail.headers.from.as_str()) {
        mail.original_from = None;
    }
    Ok(mail)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn eml(
        from: &str,
        subject: &str,
        body: &str,
        attachment: Option<(&str, &str, &[u8])>,
    ) -> Vec<u8> {
        use base64::Engine as _;
        let mut s = format!(
            "Authentication-Results: mx.google.com;\r\n       dkim=pass header.i=@{d} header.s=s1;\r\n       spf=pass smtp.mailfrom={from};\r\n       dmarc=pass (p=REJECT) header.from={d}\r\nFrom: Banco <{from}>\r\nTo: familia@gmail.com\r\nSubject: {subject}\r\nDate: Mon, 19 Oct 2026 10:00:00 -0300\r\nMessage-ID: <{id}@exemplo>\r\nMIME-Version: 1.0\r\n",
            d = from.rsplit_once('@').map_or("", |x| x.1),
            id = subject.len() + body.len(),
        );
        match attachment {
            None => s.push_str(&format!(
                "Content-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n"
            )),
            Some((name, mime, bytes)) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                s.push_str(&format!(
                    "Content-Type: multipart/mixed; boundary=\"XX\"\r\n\r\n--XX\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n--XX\r\nContent-Type: {mime}; name=\"{name}\"\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{b64}\r\n--XX--\r\n"
                ));
            }
        }
        s.into_bytes()
    }

    #[test]
    fn reads_headers_parts_and_forwarding() {
        let raw = eml(
            "faturas@nubank.com.br",
            "=?UTF-8?Q?Fwd:_Sua_fatura_de_outubro_chegou?=",
            "---------- Forwarded message ---------\r\nDe: Nubank <todomundo@nubank.com.br>\r\nAssunto: fatura\r\n\r\nSegue a fatura.",
            Some((
                "../../etc/fatura.ofx",
                "application/x-ofx",
                b"OFXHEADER:100\r\n",
            )),
        );
        let m = parse(&raw).unwrap();
        assert_eq!(m.headers.from, "faturas@nubank.com.br");
        assert_eq!(m.headers.subject, "Sua fatura de outubro chegou");
        assert_eq!(m.headers.date.as_deref(), Some("2026-10-19"));
        assert_eq!(m.headers.to, ["familia@gmail.com"]);
        assert_eq!(m.headers.auth_results.len(), 1);
        assert!(m.forwarded);
        assert_eq!(m.original_from.as_deref(), Some("todomundo@nubank.com.br"));
        assert_eq!(m.pieces.len(), 1);
        // The name never becomes a path.
        assert_eq!(m.pieces[0].filename, "fatura.ofx");
        assert!(m.pieces[0].bytes.starts_with(b"OFXHEADER"));
        assert!(parse_headers(&raw).unwrap().message_id.is_some());
        // Compressed ones are left out.
        let z = eml(
            "a@b.com",
            "x",
            "y",
            Some(("extratos.zip", "application/zip", b"PK\x03\x04")),
        );
        let m = parse(&z).unwrap();
        assert!(m.pieces.is_empty() && !m.warnings.is_empty());
        assert_eq!(safe_name("..\\..\\x.pdf", "f"), "x.pdf");
        assert_eq!(safe_name("", "anexo-1"), "anexo-1");
    }
}
