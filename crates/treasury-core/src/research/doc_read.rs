//! Texto de um documento do IPE (fato relevante, comunicado…) para o agente
//! (docs/asset-overview.md §3): o PDF baixado pelo link oficial da CVM (só de
//! `rad.cvm.gov.br`, com a fonte CVM ligada; os de fundos imobiliários, do Fundos.NET, com a
//! fonte Fundos.NET ligada), o texto extraído (ou lido pelo OCR local, quando o
//! PDF é só imagem) e devolvido em partes. Fica na memória enquanto o app está aberto: nada vai
//! para o cofre (o documento é público). Texto da companhia: dado, nunca instrução.

use std::sync::{LazyLock, Mutex};

use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::net::{Fetched, Fetcher};
use crate::error::{Error, Result};
use crate::i18n::t;

/// Até quantos caracteres uma parte devolve.
pub const MAX_CHUNK: usize = 40_000;
const DEFAULT_CHUNK: usize = 20_000;
/// Maior PDF baixado.
const MAX_BYTES: usize = 25 * 1024 * 1024;
/// Documentos guardados na memória.
const KEEP: usize = 24;

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct DocumentReadArgs {
    /// Id do documento (`list_corporate_documents`).
    pub id: String,
    /// Caractere de onde começar (padrão 0).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub offset: Option<usize>,
    /// Quantos caracteres (padrão 20 mil, até 40 mil).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentText {
    pub id: String,
    pub company_name: String,
    pub category: String,
    #[ts(optional)]
    pub subject: Option<String>,
    pub delivered_at: String,
    pub link: String,
    /// O trecho pedido.
    pub text: String,
    #[ts(type = "number")]
    pub offset: usize,
    /// Onde continuar (`None`: acabou).
    #[ts(optional, type = "number")]
    pub next_offset: Option<usize>,
    #[ts(type = "number")]
    pub total_chars: usize,
    /// O texto veio do OCR local (PDF só com imagem): pode ter erros.
    pub ocr: bool,
}

/// Só o link oficial da CVM (RAD/ENET).
pub fn official_link(link: &str) -> bool {
    link.strip_prefix("https://")
        .and_then(|r| r.split('/').next())
        .is_some_and(|host| host == "www.rad.cvm.gov.br" || host == "rad.cvm.gov.br")
}

/// Texto do arquivo baixado: PDF (camada de texto ou OCR) ou HTML (sem as marcações).
pub fn extract(bytes: &[u8]) -> Result<(String, bool)> {
    if bytes.starts_with(b"%PDF") {
        // Em Markdown (títulos e tabelas); só imagem: o OCR local.
        let doc = crate::pdf_markdown::markdown_with(bytes, None, false).unwrap_or(
            crate::pdf_markdown::MdDocument {
                pages: vec![],
                engine: crate::pdf_markdown::MdEngine::Plain,
                plain_pages: vec![],
            },
        );
        let text = doc.markdown();
        if text.chars().filter(|c| c.is_alphanumeric()).count() >= 40 {
            return Ok((text, false));
        }
        let o = crate::ocr::read_scanned(bytes)?;
        return Ok((o.text(), true));
    }
    let html = String::from_utf8_lossy(bytes);
    let mut out = String::with_capacity(html.len());
    let mut inside = false;
    for c in html.chars() {
        match c {
            '<' => {
                inside = true;
                out.push(' ');
            }
            '>' => inside = false,
            c if !inside => out.push(c),
            _ => {}
        }
    }
    Ok((clean(&out), false))
}

/// Linhas sem espaços sobrando e sem linhas vazias repetidas.
fn clean(s: &str) -> String {
    let mut out: Vec<String> = vec![];
    for l in s.lines() {
        let l = l.split_whitespace().collect::<Vec<_>>().join(" ");
        if l.is_empty() && out.last().is_none_or(|x| x.is_empty()) {
            continue;
        }
        out.push(l);
    }
    out.join("\n").trim().to_string()
}

static TEXTS: LazyLock<Mutex<Vec<(String, String, bool)>>> = LazyLock::new(|| Mutex::new(vec![]));

/// O texto do documento: da memória ou baixado e extraído.
pub fn text_of(f: &dyn Fetcher, id: &str, link: &str) -> Result<(String, bool)> {
    if let Some((_, t, ocr)) = TEXTS
        .lock()
        .ok()
        .and_then(|v| v.iter().find(|(k, ..)| k == id).cloned())
    {
        return Ok((t, ocr));
    }
    if !official_link(link) && !super::fundos_net::official_link(link) {
        return Err(Error::invalid(t("nucleo-pesquisa-documento-link")));
    }
    super::net::pace(link);
    let bytes = match f.get(link, None, "").map_err(Error::invalid)? {
        Fetched::Body { bytes, .. } => bytes,
        _ => return Err(Error::not_found(t("nucleo-pesquisa-documento-sumiu"))),
    };
    if bytes.len() > MAX_BYTES {
        return Err(Error::invalid(t("nucleo-pesquisa-documento-grande")));
    }
    let (text, ocr) = extract(&bytes)?;
    if text.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-pesquisa-documento-sem-texto")));
    }
    if let Ok(mut v) = TEXTS.lock() {
        v.retain(|(k, ..)| k != id);
        v.push((id.to_string(), text.clone(), ocr));
        if v.len() > KEEP {
            v.remove(0);
        }
    }
    Ok((text, ocr))
}

/// O trecho `[offset, offset + limit)` em caracteres.
pub fn chunk(text: &str, offset: usize, limit: usize) -> (String, Option<usize>, usize) {
    let total = text.chars().count();
    let start = offset.min(total);
    let limit = limit.clamp(500, MAX_CHUNK);
    let piece: String = text.chars().skip(start).take(limit).collect();
    let end = start + piece.chars().count();
    (piece, (end < total).then_some(end), total)
}

#[allow(clippy::type_complexity)]
fn meta(
    conn: &rusqlite::Connection,
    id: &str,
) -> Result<(String, String, Option<String>, String, String)> {
    conn.query_row(
        "SELECT company_name, category, subject, delivered_at, link FROM corporate_document WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
    )
    .optional()?
    .ok_or_else(|| Error::not_found(t("nucleo-pesquisa-documento-nao-encontrado")))
}

/// `research.document_read` (rede fora da trava do banco).
pub fn read(api: &crate::api::Api, a: &DocumentReadArgs) -> Result<DocumentText> {
    let (company_name, category, subject, delivered_at, link) = api.with_session(|s, _| {
        let m = meta(s.db.conn(), &a.id)?;
        // A fund's filing comes from Fundos.NET; a company's from the CVM.
        let source = if super::fundos_net::official_link(&m.4) {
            crate::sources::SourceId::FundosNet
        } else {
            crate::sources::SourceId::Cvm
        };
        crate::sources::ensure_enabled(s.db.conn(), source)?;
        Ok(m)
    })?;
    let f = super::net::HttpFetcher::new();
    let (text, ocr) = text_of(&f, &a.id, &link)?;
    let (piece, next_offset, total_chars) = chunk(
        &text,
        a.offset.unwrap_or(0),
        a.limit.unwrap_or(DEFAULT_CHUNK),
    );
    Ok(DocumentText {
        id: a.id.clone(),
        company_name,
        category,
        subject,
        delivered_at,
        link,
        text: piece,
        offset: a.offset.unwrap_or(0).min(total_chars),
        next_offset,
        total_chars,
        ocr,
    })
}

#[cfg(test)]
pub(crate) fn remember(id: &str, text: &str) {
    if let Ok(mut v) = TEXTS.lock() {
        v.push((id.to_string(), text.to_string(), false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_the_official_link_and_text_in_parts() {
        assert!(official_link(
            "https://www.rad.cvm.gov.br/ENET/frmDownloadDocumento.aspx?Tela=ext&numProtocolo=1"
        ));
        assert!(!official_link("http://www.rad.cvm.gov.br/x"));
        assert!(!official_link("https://rad.cvm.gov.br.exemplo.invalid/x"));
        let (html, ocr) =
            extract(b"<html><body><p>Fato   relevante</p>\n\n\n<p>Texto</p></body></html>")
                .unwrap();
        assert!(!ocr);
        assert!(html.contains("Fato relevante") && html.contains("Texto"));
        let text = "a".repeat(1200);
        let (p, next, total) = chunk(&text, 0, 500);
        assert_eq!((p.len(), next, total), (500, Some(500), 1200));
        let (p, next, _) = chunk(&text, 1000, 500);
        assert_eq!((p.len(), next), (200, None));
    }

    #[test]
    fn reads_a_captured_document_for_the_agent() {
        let dir = tempfile::tempdir().unwrap();
        let api = crate::api::Api::new_unlocked_process(
            crate::paths::TreasuryPaths::with_root(dir.path().join(".t")).unwrap(),
        )
        .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        api.with_session(|s, _| {
            s.db.conn().execute(
                "INSERT INTO corporate_document (id, issuer_id, company_name, category, subject, delivered_at, link, source, fetched_at)
                 VALUES ('doc-1', '00000000000100', 'EXEMPLO ENERGIA S.A.', 'Fato Relevante', 'Aprovação de dividendos', '2026-10-26',
                         'https://www.rad.cvm.gov.br/ENET/frmDownloadDocumento.aspx?numProtocolo=1', 'cvm_ipe', 'x')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        remember("doc-1", &format!("Fato relevante.\n{}", "x".repeat(30_000)));
        let r = api
            .call(
                "research.document_read",
                json!({"id": "doc-1", "limit": 1000}),
            )
            .unwrap();
        assert_eq!(r["category"], "Fato Relevante");
        assert!(r["text"].as_str().unwrap().starts_with("Fato relevante."));
        assert_eq!(r["next_offset"], 1000);
        assert_eq!(r["total_chars"], 30_016);
        // The agent (read access) also reads.
        let ai = crate::api::Actor::Mcp {
            client: "agente".into(),
            read_only: true,
        };
        let r = api
            .call_as(
                &ai,
                "research.document_read",
                json!({"id": "doc-1", "offset": 30_000}),
            )
            .unwrap();
        assert!(r["next_offset"].is_null());
        assert!(
            api.call("research.document_read", json!({"id": "nao-existe"}))
                .is_err()
        );
    }
}
