//! Leitura de texto de PDFs (anexos do agente, notas de corretagem, extratos), num lugar só:
//! corrige arquivos com bytes depois do `%%EOF` (alguns geradores e downloads completam com
//! zeros, e o leitor procura o `startxref` no fim do arquivo) e protege contra pânicos do
//! leitor.

/// O PDF sem o que vem depois do último `%%EOF`.
pub fn sanitize(bytes: &[u8]) -> &[u8] {
    match bytes.windows(5).rposition(|w| w == b"%%EOF") {
        Some(i) => &bytes[..i + 5],
        None => bytes,
    }
}

fn guard<T>(
    f: impl FnOnce() -> Result<T, pdf_extract::OutputError> + std::panic::UnwindSafe,
) -> Result<T, String> {
    std::panic::catch_unwind(f)
        .map_err(|_| crate::i18n::t("nucleo-base-erro-pdf"))?
        .map_err(|e| e.to_string())
}

/// Texto de cada página.
pub fn pages(bytes: &[u8]) -> Result<Vec<String>, String> {
    let b = sanitize(bytes).to_vec();
    guard(move || pdf_extract::extract_text_from_mem_by_pages(&b))
}

/// Texto do PDF inteiro (com a senha, se protegido).
pub fn text(bytes: &[u8], password: Option<&str>) -> Result<String, String> {
    let b = sanitize(bytes).to_vec();
    match password.filter(|p| !p.is_empty()) {
        Some(p) => {
            let p = p.to_string();
            guard(move || pdf_extract::extract_text_from_mem_encrypted(&b, &p))
        }
        None => guard(move || pdf_extract::extract_text_from_mem(&b)),
    }
}

/// As imagens de um PDF sem texto (escaneado ou gerado como imagem, como o extrato por período
/// da Caixa), na ordem das páginas.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ScannedPdf {
    /// JPEGs (`/DCTDecode`): os bytes do fluxo já são o arquivo JPEG.
    pub jpegs: Vec<Vec<u8>>,
    /// Imagens noutro formato (ainda não lidas).
    pub other_images: usize,
}

/// Tira as imagens de cada página sem desenhar nada (o leitor de OCR lê os JPEGs).
pub fn page_jpegs(bytes: &[u8]) -> Result<ScannedPdf, String> {
    let b = sanitize(bytes).to_vec();
    std::panic::catch_unwind(move || -> Result<ScannedPdf, String> {
        let doc = lopdf::Document::load_mem(&b).map_err(|e| e.to_string())?;
        let mut out = ScannedPdf::default();
        for (_, page) in doc.get_pages() {
            let Ok(images) = doc.get_page_images(page) else {
                continue;
            };
            for img in images {
                let filters = img.filters.clone().unwrap_or_default();
                if filters.len() == 1 && filters[0] == "DCTDecode" {
                    out.jpegs.push(img.content.to_vec());
                } else {
                    out.other_images += 1;
                }
            }
        }
        Ok(out)
    })
    .map_err(|_| crate::i18n::t("nucleo-base-erro-pdf"))?
}

#[cfg(test)]
mod tests {
    #[test]
    fn trims_bytes_after_eof() {
        let mut v = b"%PDF-1.4 ... startxref 10 %%EOF".to_vec();
        v.extend([0u8; 64]);
        assert_eq!(super::sanitize(&v), b"%PDF-1.4 ... startxref 10 %%EOF");
        assert_eq!(super::sanitize(b"sem fim"), b"sem fim");
    }
}
