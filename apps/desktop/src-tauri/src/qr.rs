//! QR do cupom fiscal numa foto pelo Apple Vision (macOS, `VNDetectBarcodesRequest`): o mesmo
//! leitor da câmera do sistema, que aguenta sombra, inclinação e papel amassado, lê HEIC e
//! respeita a rotação EXIF da foto do iPhone. Roda no computador (a imagem não sai dele). Fora
//! do Mac devolve `None` e a interface usa o leitor em JavaScript. Também lê os códigos de um
//! boleto ou de uma conta (QR do Pix e o intercalado 2 de 5 de 44 dígitos), sem dependência
//! nova (docs/ocr-other-documents.md › 2.3).

use base64::Engine as _;
use treasury_core::ApiError;

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
    use objc2_foundation::{NSArray, NSData, NSDictionary};
    use objc2_vision::{
        VNBarcodeSymbology, VNBarcodeSymbologyEAN8, VNBarcodeSymbologyEAN13,
        VNBarcodeSymbologyI2of5, VNBarcodeSymbologyITF14, VNBarcodeSymbologyQR,
        VNBarcodeSymbologyUPCE, VNDetectBarcodesRequest, VNImageRequestHandler, VNRequest,
    };

    /// O texto do primeiro QR da imagem.
    pub fn read(bytes: &[u8]) -> Result<Option<String>, String> {
        read_with(bytes, &[unsafe { VNBarcodeSymbologyQR }])
    }

    /// O código de barras de produto (EAN-13, EAN-8, UPC-E, ITF-14) da imagem.
    pub fn read_product_code(bytes: &[u8]) -> Result<Option<String>, String> {
        read_with(
            bytes,
            &[
                unsafe { VNBarcodeSymbologyEAN13 },
                unsafe { VNBarcodeSymbologyEAN8 },
                unsafe { VNBarcodeSymbologyUPCE },
                unsafe { VNBarcodeSymbologyITF14 },
            ],
        )
    }

    /// Todos os códigos de pagamento da imagem: QR (Pix) e intercalado 2 de 5 (boleto e
    /// arrecadação, 44 dígitos). O núcleo confere o DV e o CRC.
    pub fn read_payment_codes(bytes: &[u8]) -> Result<Vec<String>, String> {
        read_all(
            bytes,
            &[unsafe { VNBarcodeSymbologyQR }, unsafe {
                VNBarcodeSymbologyI2of5
            }],
        )
    }

    fn read_with(
        bytes: &[u8],
        symbologies: &[Option<&'static VNBarcodeSymbology>],
    ) -> Result<Option<String>, String> {
        Ok(read_all(bytes, symbologies)?.into_iter().next())
    }

    fn read_all(
        bytes: &[u8],
        symbologies: &[Option<&'static VNBarcodeSymbology>],
    ) -> Result<Vec<String>, String> {
        let data = NSData::with_bytes(bytes);
        let options = NSDictionary::new();
        let handler = VNImageRequestHandler::initWithData_options(
            VNImageRequestHandler::alloc(),
            &data,
            &options,
        );
        let request = unsafe { VNDetectBarcodesRequest::new() };
        let wanted: Vec<&VNBarcodeSymbology> = symbologies.iter().flatten().copied().collect();
        if !wanted.is_empty() {
            unsafe { request.setSymbologies(&NSArray::from_slice(&wanted)) };
        }
        // VNDetectBarcodesRequest → VNImageBasedRequest → VNRequest.
        let as_request: Retained<VNRequest> =
            Retained::into_super(Retained::into_super(request.clone()));
        handler
            .performRequests_error(&NSArray::from_retained_slice(&[as_request]))
            .map_err(|e| e.localizedDescription().to_string())?;
        let results = unsafe { request.results() };
        Ok(results
            .map(|list| {
                list.iter()
                    .filter_map(|o| unsafe { o.payloadStringValue() }.map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default())
    }
}

/// O texto do QR na foto (`content_base64`: os bytes do arquivo, JPEG, PNG ou HEIC), ou
/// `None` se não achar ou fora do Mac.
#[tauri::command]
pub async fn read_qr_native(content_base64: String) -> Result<Option<String>, ApiError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(content_base64.as_bytes())
        .map_err(|e| err(e.to_string()))?;
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(move || mac::read(&bytes))
            .await
            .map_err(|e| err(e.to_string()))?
            .map_err(err)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bytes;
        Ok(None)
    }
}

/// O código de barras de produto na foto (EAN-13, EAN-8, UPC-E, ITF-14), só os dígitos, ou
/// `None` se não achar ou fora do Mac. O núcleo confere o dígito verificador.
#[tauri::command]
pub async fn read_barcode_native(content_base64: String) -> Result<Option<String>, ApiError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(content_base64.as_bytes())
        .map_err(|e| err(e.to_string()))?;
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(move || mac::read_product_code(&bytes))
            .await
            .map_err(|e| err(e.to_string()))?
            .map_err(err)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bytes;
        Ok(None)
    }
}

/// Códigos de pagamento na foto (QR do Pix, código de barras do boleto), na ordem em que o
/// Vision os achou; vazio se nada ou fora do Mac.
#[tauri::command]
pub async fn read_payment_codes_native(content_base64: String) -> Result<Vec<String>, ApiError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(content_base64.as_bytes())
        .map_err(|e| err(e.to_string()))?;
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(move || mac::read_payment_codes(&bytes))
            .await
            .map_err(|e| err(e.to_string()))?
            .map_err(err)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bytes;
        Ok(vec![])
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    /// O mesmo QR inventado dos testes da interface (chave e CNPJ de exemplo).
    const FIXTURE: &str =
        include_str!("../../src/features/receipts/fixtures/qr-nfce-inventado.txt");
    const TEXT: &str = "https://www.fazenda.pr.gov.br/nfce/qrcode?p=41261011222333000181650010000012341000012346|2|1|1|0A1B2C3D4E5F60718293A4B5C6D7E8F901234567";

    /// "Foto" do cupom: papel cinza, QR inclinado 7°, sombra forte e ondulada em metade e ruído,
    /// gravada como BMP de 24 bits (o ImageIO do sistema lê).
    fn photo() -> Vec<u8> {
        let m: Vec<&str> = FIXTURE
            .lines()
            .filter(|l| l.chars().all(|c| c == '0' || c == '1') && !l.is_empty())
            .collect();
        let (w, h, size) = (640usize, 760usize, 360.0f64);
        let n = m.len() as f64;
        let a = (7.0f64).to_radians();
        let mut seed = 7u32;
        let mut px = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let dx = x as f64 - w as f64 / 2.0;
                let dy = y as f64 - h as f64 / 2.0;
                let u = dx * (-a).cos() - dy * (-a).sin();
                let v = dx * (-a).sin() + dy * (-a).cos();
                let mx = ((u + size / 2.0) / size * n).floor();
                let my = ((v + size / 2.0) / size * n).floor();
                let mut val = 205.0;
                if mx >= 0.0
                    && my >= 0.0
                    && mx < n
                    && my < n
                    && m[my as usize].as_bytes()[mx as usize] == b'1'
                {
                    val = 45.0;
                }
                let edge = w as f64 * 0.52 + (y as f64 / 37.0).sin() * 40.0;
                let soft = ((edge - x as f64) / 18.0).clamp(0.0, 1.0);
                val *= 1.0 - 0.7 * soft;
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345) & 0x7fff_ffff;
                val += (seed as f64 / 0x7fff_ffff as f64 - 0.5) * 24.0;
                px[y * w + x] = val.clamp(0.0, 255.0) as u8;
            }
        }
        let row = (w * 3).div_ceil(4) * 4;
        let size_img = row * h;
        let mut b = Vec::with_capacity(54 + size_img);
        b.extend_from_slice(b"BM");
        b.extend_from_slice(&((54 + size_img) as u32).to_le_bytes());
        b.extend_from_slice(&[0, 0, 0, 0]);
        b.extend_from_slice(&54u32.to_le_bytes());
        b.extend_from_slice(&40u32.to_le_bytes());
        b.extend_from_slice(&(w as i32).to_le_bytes());
        b.extend_from_slice(&(h as i32).to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&24u16.to_le_bytes());
        b.extend_from_slice(&[0; 4]);
        b.extend_from_slice(&(size_img as u32).to_le_bytes());
        b.extend_from_slice(&[0; 16]);
        for y in (0..h).rev() {
            for x in 0..w {
                let v = px[y * w + x];
                b.extend_from_slice(&[v, v, v]);
            }
            b.resize(b.len() + (row - w * 3), 0);
        }
        b
    }

    #[test]
    fn vision_reads_the_qr_with_shadow_and_tilt() {
        assert_eq!(super::mac::read(&photo()).unwrap().as_deref(), Some(TEXT));
        assert_eq!(super::mac::read(b"nada").ok().flatten(), None);
    }
}
