//! Leitura de texto de imagens fora do cupom (docs/ocr-other-documents.md › 2.1): o mesmo OCR local
//! do cupom (Apple Vision pelo app, no Mac; PaddleOCR embutido nos outros sistemas, ou quando o
//! Vision não mandou linhas) e a mesma junção das caixas em linhas. Nada sai do computador. O
//! texto serve aos leitores de documentos (boleto, Pix, comprovante; `crate::payments`), ao
//! agente de IA (texto no lugar da imagem para motor sem visão; `ai::attachments`) e à ingestão
//! por e-mail.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::receipts::ocr::{OcrEngine, OcrLine, lines};

/// O texto lido de uma imagem.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OcrText {
    /// Motor que leu (`None` sem texto).
    #[ts(optional)]
    pub engine: Option<OcrEngine>,
    /// Linhas de cima para baixo (trechos da mesma altura separados por dois espaços).
    pub rows: Vec<String>,
    /// Confiança média do OCR, de 0 a 1.
    pub confidence: f32,
}

impl OcrText {
    /// As linhas num texto só.
    pub fn text(&self) -> String {
        self.rows.join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.rows.iter().all(|r| r.trim().is_empty())
    }
}

/// Texto de linhas já lidas (Vision, no Mac, ou PaddleOCR).
pub fn from_lines(engine: OcrEngine, ls: &[OcrLine], aspect: f32) -> OcrText {
    let useful: Vec<&OcrLine> = ls.iter().filter(|l| !l.text.trim().is_empty()).collect();
    let confidence = if useful.is_empty() {
        0.0
    } else {
        useful.iter().map(|l| l.conf).sum::<f32>() / useful.len() as f32
    };
    let rows = lines::row_texts(ls, aspect);
    OcrText {
        engine: (!rows.is_empty()).then_some(engine),
        rows,
        confidence,
    }
}

/// Lê o texto: as linhas do Vision, quando vieram, ou o PaddleOCR com os bytes da imagem.
/// `aspect` é largura/altura da foto (para endireitar as linhas do Vision).
pub fn read_text(vision: Option<&[OcrLine]>, aspect: f32, bytes: Option<&[u8]>) -> Result<OcrText> {
    if let Some(ls) = vision.filter(|l| !l.is_empty()) {
        return Ok(from_lines(OcrEngine::Vision, ls, aspect));
    }
    #[cfg(feature = "ocr-paddle")]
    if let Some(b) = bytes {
        let ls = crate::receipts::ocr::paddle::recognize(b)?;
        return Ok(from_lines(OcrEngine::Paddle, &ls, 1.0));
    }
    let _ = bytes;
    Err(Error::invalid(t("nucleo-ocr-erro-sem-motor")))
}

/// Texto de um PDF sem camada de texto (as imagens JPEG das páginas, em ordem) ou de uma
/// imagem solta, pelo OCR embutido. O resultado fica guardado por alguns arquivos (pelo
/// sha256), porque a mesma importação lê o arquivo várias vezes (identificar, prévia, gravar).
pub fn read_scanned(bytes: &[u8]) -> Result<OcrText> {
    use std::sync::{LazyLock, Mutex};
    static CACHE: LazyLock<Mutex<Vec<(String, OcrText)>>> = LazyLock::new(|| Mutex::new(vec![]));
    let key = {
        use sha2::Digest;
        hex::encode(sha2::Sha256::digest(bytes))
    };
    if let Some((_, o)) = CACHE
        .lock()
        .ok()
        .and_then(|c| c.iter().find(|(k, _)| *k == key).cloned())
    {
        return Ok(o);
    }
    let out = if bytes.starts_with(b"%PDF") {
        let pages = crate::pdf::page_jpegs(bytes).map_err(Error::invalid)?;
        if pages.jpegs.is_empty() {
            return Err(Error::invalid(t(if pages.other_images > 0 {
                "nucleo-ocr-erro-pdf-formato"
            } else {
                "nucleo-ocr-erro-pdf-sem-imagem"
            })));
        }
        let mut all = OcrText::default();
        let mut confs = vec![];
        for jpeg in &pages.jpegs {
            let o = read_text(None, 1.0, Some(jpeg))?;
            all.engine = all.engine.or(o.engine);
            confs.push(o.confidence);
            all.rows.extend(o.rows);
        }
        all.confidence = confs.iter().sum::<f32>() / confs.len().max(1) as f32;
        all
    } else {
        read_text(None, 1.0, Some(bytes))?
    };
    if let Ok(mut c) = CACHE.lock() {
        c.retain(|(k, _)| *k != key);
        c.push((key, out.clone()));
        if c.len() > 4 {
            c.remove(0);
        }
    }
    Ok(out)
}

/// Largura/altura a partir das dimensões informadas pela interface (1 sem elas).
pub fn aspect(width: Option<f32>, height: Option<f32>) -> f32 {
    match (width, height) {
        (Some(w), Some(h)) if h > 0.0 => w / h,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str, y: f32, x: f32) -> OcrLine {
        OcrLine {
            text: text.into(),
            conf: 0.9,
            x,
            y,
            w: 0.3,
            h: 0.02,
            ..Default::default()
        }
    }

    #[test]
    fn joins_boxes_in_rows_top_down() {
        let t = from_lines(
            OcrEngine::Vision,
            &[
                line("Valor", 0.5, 0.1),
                line("R$ 50,00", 0.5, 0.6),
                line("Comprovante", 0.9, 0.1),
                line("", 0.2, 0.1),
            ],
            1.0,
        );
        assert_eq!(t.rows, vec!["Comprovante", "Valor  R$ 50,00"]);
        assert_eq!(t.engine, Some(OcrEngine::Vision));
        assert!((t.confidence - 0.9).abs() < 1e-6);
        assert_eq!(t.text(), "Comprovante\nValor  R$ 50,00");
        let empty = from_lines(OcrEngine::Vision, &[], 1.0);
        assert!(empty.is_empty() && empty.engine.is_none());
    }
}
