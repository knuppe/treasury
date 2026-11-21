//! Texto do cupom numa foto pelo Apple Vision (macOS, `VNRecognizeTextRequest`, nível
//! "accurate", sem correção de idioma, que estraga códigos e valores): docs/ocr-receipts.md. Roda
//! no computador (a imagem não sai dele). Devolve as linhas com a posição; o núcleo monta os
//! itens e confere as contas (`receipts.read_photo`). Fora do Mac devolve `None` e o núcleo usa
//! o PaddleOCR embutido.

use base64::Engine as _;
use treasury_core::ApiError;
use treasury_core::receipts::ocr::OcrLine;

fn err(message: impl Into<String>) -> ApiError {
    ApiError {
        code: "invalid".into(),
        message: message.into(),
        details: None,
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use objc2::AnyThread;
    use objc2::rc::Retained;
    use objc2_foundation::{NSArray, NSData, NSDictionary, NSString};
    use objc2_vision::{
        VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
    };

    use super::OcrLine;

    /// Idiomas dos países do Treasury (o Vision usa a lista como preferência, não filtro).
    const LANGS: &[&str] = &["pt-BR", "es-ES", "fr-FR", "de-DE", "en-US"];

    pub fn read(bytes: &[u8]) -> Result<Vec<OcrLine>, String> {
        let data = NSData::with_bytes(bytes);
        let options = NSDictionary::new();
        let handler = VNImageRequestHandler::initWithData_options(
            VNImageRequestHandler::alloc(),
            &data,
            &options,
        );
        let request = VNRecognizeTextRequest::new();
        request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
        request.setUsesLanguageCorrection(false);
        let langs: Vec<Retained<NSString>> = LANGS.iter().map(|l| NSString::from_str(l)).collect();
        request.setRecognitionLanguages(&NSArray::from_retained_slice(&langs));
        // VNRecognizeTextRequest → VNImageBasedRequest → VNRequest.
        let as_request: Retained<VNRequest> =
            Retained::into_super(Retained::into_super(request.clone()));
        handler
            .performRequests_error(&NSArray::from_retained_slice(&[as_request]))
            .map_err(|e| e.localizedDescription().to_string())?;
        let Some(results) = request.results() else {
            return Ok(vec![]);
        };
        Ok(results
            .iter()
            .filter_map(|o| {
                let top = o.topCandidates(1).firstObject()?;
                let b = unsafe { o.boundingBox() };
                let (tl, tr) = unsafe { (o.topLeft(), o.topRight()) };
                Some(OcrLine {
                    text: top.string().to_string(),
                    conf: top.confidence(),
                    x: b.origin.x as f32,
                    y: b.origin.y as f32,
                    w: b.size.width as f32,
                    h: b.size.height as f32,
                    tl: Some([tl.x as f32, tl.y as f32]),
                    tr: Some([tr.x as f32, tr.y as f32]),
                })
            })
            .collect())
    }
}

/// As linhas de texto da foto (`content_base64`: os bytes do arquivo, JPEG, PNG ou HEIC), ou
/// `None` fora do Mac.
#[tauri::command]
pub async fn read_text_native(content_base64: String) -> Result<Option<Vec<OcrLine>>, ApiError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(content_base64.as_bytes())
        .map_err(|e| err(e.to_string()))?;
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(move || mac::read(&bytes))
            .await
            .map_err(|e| err(e.to_string()))?
            .map(Some)
            .map_err(err)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bytes;
        Ok(None)
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    /// Foto particular de cupom (`TREASURY_OCR_SAMPLE`), nunca no repositório.
    #[test]
    #[ignore]
    fn reads_a_private_sample() {
        let Ok(path) = std::env::var("TREASURY_OCR_SAMPLE") else {
            return;
        };
        let lines = super::mac::read(&std::fs::read(path).unwrap()).unwrap();
        assert!(!lines.is_empty());
        for l in &lines {
            println!("{:.3} {:.3} {}", l.x, l.y, l.text);
        }
    }
}
