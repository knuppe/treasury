//! `import.sniff`: qual importador serve para um arquivo (janela única de importação). Lê só o
//! necessário e nada é gravado:
//!
//! - **OFX** ou **CSV** de extrato/fatura → importação de extratos;
//! - **PDF** com "Nota de corretagem/negociação" (SINACOR) → importação de notas (PDF
//!   protegido por senha também vai para lá, que pede a senha);
//! - **PDF sem texto** (a página é uma imagem) → OCR local; o extrato por período da Caixa vai
//!   para a importação de extratos;
//! - planilha ou CSV da **Área do Investidor B3** (negociação, movimentação, posição) →
//!   importação da B3;
//! - **cupom fiscal** em PDF (DANFE NFC-e) ou o XML da NF-e/NFC-e → leitura do cupom
//!   (Consumo);
//! - **imagem** (PNG, JPEG, WebP, GIF, HEIC) → a interface lê o QR e o texto no computador
//!   (`import.analyze_image`): cupom abre a compra; comprovante, boleto e o resto vão ao
//!   agente, por último;
//! - o resto → desconhecido (a interface oferece analisar com o agente).

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::i18n::{self, args, t_args};

/// Texto do catálogo (os rótulos e as dicas de cada tipo de arquivo).
fn tr(key: &str) -> String {
    i18n::t(key)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FileKind {
    Statement,
    BrokerageNote,
    B3Report,
    /// Extrato de previdência (fundo de pensão).
    PensionStatement,
    /// Cupom fiscal em PDF (DANFE NFC-e) ou XML da NF-e/NFC-e: abre a leitura do cupom.
    Receipt,
    /// Imagem: lida no computador (QR e OCR) antes de qualquer IA.
    Image,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SniffArgs {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SniffResult {
    pub filename: String,
    pub kind: FileKind,
    /// `ofx`, `csv`, `pdf`, `xlsx`…
    pub format: Option<String>,
    /// O que foi reconhecido, em português ("Extrato OFX (Nubank)").
    pub label: String,
    /// Por que não foi reconhecido (desconhecido) ou um aviso.
    pub note: Option<String>,
    /// PDF protegido: a importação de notas pede a senha.
    pub needs_password: bool,
    /// Layout reconhecido (docs/layouts.md).
    pub layout: Option<crate::layouts::LayoutInfo>,
}

fn result(
    a: &SniffArgs,
    kind: FileKind,
    format: Option<&str>,
    label: String,
    note: Option<String>,
) -> SniffResult {
    SniffResult {
        filename: a.filename.clone(),
        kind,
        format: format.map(str::to_string),
        label,
        note,
        needs_password: false,
        layout: None,
    }
}

/// Formato de imagem pelos primeiros bytes.
fn image_format(b: &[u8]) -> Option<&'static str> {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpeg")
    } else if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        Some("gif")
    } else if b.len() >= 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        Some("webp")
    } else if b.len() >= 12
        && &b[4..8] == b"ftyp"
        && matches!(
            &b[8..12],
            b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"msf1" | b"heim" | b"heis"
        )
    {
        Some("heic")
    } else {
        None
    }
}

/// PDF sem texto (escaneado, ou gerado como imagem, como o extrato por período da Caixa): as
/// imagens das páginas pelo OCR local (docs/brazil/caixa-statement-by-period.md).
fn scanned(a: &SniffArgs, bytes: &[u8]) -> SniffResult {
    let unknown =
        |note: String| result(a, FileKind::Unknown, Some("pdf"), "PDF".into(), Some(note));
    match crate::ocr::read_scanned(bytes) {
        Ok(o) if !o.is_empty() => {
            let text = o.text();
            let id = crate::layouts::identify(&crate::layouts::Doc {
                format: crate::layouts::LayoutFormat::Pdf,
                text: &text,
                headers: &[],
            });
            if id.layout.as_ref().map(|l| l.subject)
                == Some(crate::layouts::LayoutSubject::BankStatement)
            {
                SniffResult {
                    layout: id.layout,
                    ..result(
                        a,
                        FileKind::Statement,
                        Some("pdf"),
                        tr("nucleo-importacao-tipo-extrato-caixa"),
                        Some(tr("nucleo-importacao-dica-extrato-caixa")),
                    )
                }
            } else {
                unknown(tr("nucleo-importacao-dica-pdf-imagem"))
            }
        }
        Ok(_) => unknown(tr("nucleo-importacao-dica-pdf-ilegivel")),
        Err(e) => unknown(e.message()),
    }
}

pub fn sniff(a: &SniffArgs) -> Result<SniffResult> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(a.content_base64.trim())
        .map_err(|_| crate::error::Error::invalid(tr("nucleo-importacao-erro-base64-curto")))?;
    let lower = a.filename.to_lowercase();
    if let Some(fmt) = image_format(&bytes) {
        return Ok(result(
            a,
            FileKind::Image,
            Some(fmt),
            t_args(
                "nucleo-importacao-tipo-imagem",
                &args(&[("formato", fmt.to_uppercase().into())]),
            ),
            Some(tr("nucleo-importacao-dica-imagem")),
        ));
    }
    if bytes.starts_with(b"%PDF") {
        return Ok(match crate::pdf::text(&bytes, None) {
            // Sem texto (a página é uma imagem): o OCR local diz o que é.
            Ok(text)
                if super::text::fold(&text)
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .count()
                    < 40 =>
            {
                scanned(a, &bytes)
            }
            Ok(text) => {
                let t = super::notes::parse::normalize(&text);
                let id = crate::layouts::identify(&crate::layouts::Doc {
                    format: crate::layouts::LayoutFormat::Pdf,
                    text: &text,
                    headers: &[],
                });
                if id.subject == Some(crate::layouts::LayoutSubject::PensionStatement) {
                    return Ok(SniffResult {
                        layout: id.layout.clone(),
                        ..result(
                            a,
                            FileKind::PensionStatement,
                            Some("pdf"),
                            tr("nucleo-importacao-tipo-previdencia"),
                            id.problem
                                .or(Some(tr("nucleo-importacao-dica-previdencia"))),
                        )
                    });
                }
                if crate::receipts::nfce::looks_like(&text) {
                    return Ok(result(
                        a,
                        FileKind::Receipt,
                        Some("pdf"),
                        tr("nucleo-importacao-tipo-cupom-pdf"),
                        Some(tr("nucleo-importacao-dica-cupom")),
                    ));
                }
                if id.subject == Some(crate::layouts::LayoutSubject::CardStatement) {
                    let issuer = super::card_pdf::parse(&text).issuer;
                    return Ok(SniffResult {
                        layout: id.layout.clone(),
                        ..result(
                            a,
                            FileKind::Statement,
                            Some("pdf"),
                            match issuer {
                                Some(i) => t_args(
                                    "nucleo-importacao-tipo-fatura-pdf-emissor",
                                    &args(&[("emissor", i.into())]),
                                ),
                                None => tr("nucleo-importacao-tipo-fatura-pdf"),
                            },
                            id.problem.or(Some(tr("nucleo-importacao-dica-fatura-pdf"))),
                        )
                    });
                }
                // Extrato de conta com texto (o mesmo leiaute, se um dia vier com camada de texto).
                if id.layout.as_ref().map(|l| l.subject)
                    == Some(crate::layouts::LayoutSubject::BankStatement)
                {
                    return Ok(SniffResult {
                        layout: id.layout.clone(),
                        ..result(
                            a,
                            FileKind::Statement,
                            Some("pdf"),
                            tr("nucleo-importacao-tipo-extrato-caixa"),
                            Some(tr("nucleo-importacao-dica-extrato-caixa")),
                        )
                    });
                }
                if t.contains("NOTA DE CORRETAGEM") || t.contains("NOTA DE NEGOCIACAO") {
                    result(
                        a,
                        FileKind::BrokerageNote,
                        Some("pdf"),
                        tr("nucleo-importacao-tipo-nota"),
                        None,
                    )
                } else {
                    result(
                        a,
                        FileKind::Unknown,
                        Some("pdf"),
                        "PDF".into(),
                        Some(tr("nucleo-importacao-dica-pdf-desconhecido")),
                    )
                }
            }
            Err(e) => {
                let m = e.to_string().to_lowercase();
                if m.contains("password") || m.contains("decrypt") || m.contains("encrypt") {
                    SniffResult {
                        needs_password: true,
                        ..result(
                            a,
                            FileKind::BrokerageNote,
                            Some("pdf"),
                            tr("nucleo-importacao-tipo-pdf-senha"),
                            Some(tr("nucleo-importacao-dica-pdf-senha")),
                        )
                    }
                } else {
                    scanned(a, &bytes)
                }
            }
        });
    }
    // XML da NF-e / NFC-e (o arquivo que a loja manda por e-mail).
    if let Ok(xml) = std::str::from_utf8(&bytes)
        && crate::receipts::nfce::looks_like_xml(xml)
    {
        return Ok(result(
            a,
            FileKind::Receipt,
            Some("xml"),
            tr("nucleo-importacao-tipo-cupom-xml"),
            Some(tr("nucleo-importacao-dica-cupom")),
        ));
    }
    // Planilha ou CSV da Área do Investidor B3.
    let b3 = super::b3::detect(&super::b3::B3File {
        filename: a.filename.clone(),
        content_base64: a.content_base64.clone(),
    });
    if let Ok(d) = &b3
        && !d.kinds.is_empty()
    {
        let what: Vec<String> = d
            .kinds
            .iter()
            .map(|k| {
                serde_json::to_value(k)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default()
            })
            .collect();
        let fmt = serde_json::to_value(d.format)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string));
        return Ok(result(
            a,
            FileKind::B3Report,
            fmt.as_deref(),
            t_args(
                "nucleo-importacao-tipo-b3",
                &args(&[("relatorios", what.join(", ").into())]),
            ),
            None,
        ));
    }
    // Extrato OFX ou CSV.
    if let Ok(d) = super::detect(&super::DetectArgs {
        filename: a.filename.clone(),
        content_base64: a.content_base64.clone(),
        encoding: None,
        country: None,
    }) {
        match d.format {
            super::ImportFormat::Ofx => {
                let card = d.ofx.as_ref().is_some_and(|o| o.credit_card);
                return Ok(result(
                    a,
                    FileKind::Statement,
                    Some("ofx"),
                    tr(if card {
                        "nucleo-importacao-tipo-fatura-ofx"
                    } else {
                        "nucleo-importacao-tipo-extrato-ofx"
                    }),
                    None,
                ));
            }
            super::ImportFormat::Csv
                if d.row_count > 0
                    && (d.suggested_mapping.is_some() || looks_like_statement(&d.headers)) =>
            {
                return Ok(result(
                    a,
                    FileKind::Statement,
                    Some("csv"),
                    match &d.suggested_preset_id {
                        Some(p) => t_args(
                            "nucleo-importacao-tipo-extrato-csv-preset",
                            &args(&[("preset", p.clone().into())]),
                        ),
                        None => tr("nucleo-importacao-tipo-extrato-csv"),
                    },
                    None,
                ));
            }
            _ => {}
        }
    }
    let ext = lower
        .rsplit('.')
        .next()
        .filter(|e| *e != lower)
        .map(str::to_string);
    Ok(result(
        a,
        FileKind::Unknown,
        ext.as_deref(),
        tr("nucleo-importacao-tipo-desconhecido"),
        Some(tr("nucleo-importacao-dica-desconhecido")),
    ))
}

/// Cabeçalho de CSV com data e valor (extrato genérico, sem preset).
fn looks_like_statement(headers: &[String]) -> bool {
    let h: Vec<String> = headers
        .iter()
        .map(|x| super::notes::parse::normalize(x))
        .collect();
    let has = |w: &[&str]| h.iter().any(|x| w.iter().any(|k| x.contains(k)));
    has(&["DATA", "DATE"]) && has(&["VALOR", "AMOUNT", "QUANTIA", "DEBITO", "CREDITO"])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(name: &str, bytes: &[u8]) -> SniffArgs {
        SniffArgs {
            filename: name.into(),
            content_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
        }
    }

    #[test]
    fn recognizes_each_importer_and_leaves_the_rest_to_the_agent() {
        let ofx = sniff(&args("x.ofx", include_bytes!("fixtures/nubank_xml.ofx"))).unwrap();
        assert_eq!(
            (ofx.kind, ofx.format.as_deref()),
            (FileKind::Statement, Some("ofx"))
        );
        let csv = sniff(&args("x.csv", include_bytes!("fixtures/nubank_conta.csv"))).unwrap();
        assert_eq!(
            (csv.kind, csv.format.as_deref()),
            (FileKind::Statement, Some("csv"))
        );
        let b3 = sniff(&args(
            "n.xlsx",
            include_bytes!("b3/fixtures/negociacao.xlsx"),
        ))
        .unwrap();
        assert_eq!(b3.kind, FileKind::B3Report, "{b3:?}");
        let note = crate::import::notes::tests::pdf_with_lines(
            &crate::import::notes::tests::NOTE
                .lines()
                .collect::<Vec<_>>(),
        );
        let n = sniff(&args("nota.pdf", &note)).unwrap();
        assert_eq!(n.kind, FileKind::BrokerageNote);
        let other =
            crate::import::notes::tests::pdf_with_lines(&["Contrato de aluguel", "Cláusula 1"]);
        assert_eq!(
            sniff(&args("c.pdf", &other)).unwrap().kind,
            FileKind::Unknown
        );
        let txt = sniff(&args("lista.txt", b"comprar pao\nleite\n")).unwrap();
        assert_eq!(txt.kind, FileKind::Unknown);
        assert!(txt.note.is_some());
    }

    #[test]
    fn receipts_in_xml_and_pdf_open_the_receipt_reader() {
        let xml = sniff(&args(
            "nota.xml",
            br#"<?xml version="1.0"?><nfeProc xmlns="http://www.portalfiscal.inf.br/nfe"><NFe><infNFe Id="NFe1"></infNFe></NFe></nfeProc>"#,
        ))
        .unwrap();
        assert_eq!(
            (xml.kind, xml.format.as_deref()),
            (FileKind::Receipt, Some("xml"))
        );
        let pdf = crate::import::notes::tests::pdf_with_lines(&[
            "MERCADO BOM PRECO",
            "Documento Auxiliar da Nota Fiscal de Consumidor Eletronica",
            "NFC-e 12345",
        ]);
        let r = sniff(&args("cupom.pdf", &pdf)).unwrap();
        assert_eq!(r.kind, FileKind::Receipt, "{r:?}");
    }

    #[test]
    fn images_go_to_the_agent() {
        let png = sniff(&args("recibo.png", b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR")).unwrap();
        assert_eq!(
            (png.kind, png.format.as_deref()),
            (FileKind::Image, Some("png"))
        );
        let jpg = sniff(&args("foto.JPG", &[0xFF, 0xD8, 0xFF, 0xE0, 0, 16])).unwrap();
        assert_eq!(jpg.kind, FileKind::Image);
        let heic = sniff(&args("IMG_0001.HEIC", b"\0\0\0\x18ftypheic\0\0\0\0")).unwrap();
        assert_eq!(heic.format.as_deref(), Some("heic"));
        // By content, not by extension.
        let fake = sniff(&args("x.png", b"data,valor\n2026-10-01,10\n")).unwrap();
        assert_ne!(fake.kind, FileKind::Image);
    }
}
