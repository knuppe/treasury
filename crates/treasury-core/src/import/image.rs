//! `import.analyze_image`: o que uma imagem é, lida no próprio computador antes de qualquer IA
//! (janela única de importação). O texto vem do OCR embutido (Apple Vision no Mac, pelo app,
//! ou o PaddleOCR do núcleo), o leitor de linhas dos cupons monta os itens e confere as contas,
//! e palavras do texto separam o resto:
//!
//! - **extrato por período da Caixa** (foto ou print) → importação de extratos, sem IA;
//! - **cupom fiscal ou recibo de compra** → a compra abre já preenchida (Consumo);
//! - **comprovante de pagamento** (Pix, transferência, TED, depósito) e **boleto** → o agente
//!   registra, com o tipo já dito;
//! - **outro documento** com texto, ou imagem sem texto legível → o agente, como último
//!   recurso.
//!
//! O QR do cupom (link da consulta da NFC-e) é lido antes, pela interface: com ele a consulta
//! oficial vale mais que o OCR. Nada é gravado aqui e a imagem não sai do computador.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::api::Api;
use crate::error::Result;
use crate::i18n::t;
use crate::receipts::ocr::{PhotoArgs, PhotoReading, lines::Tier};

/// O que a imagem parece ser, pelo texto lido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ImageKind {
    /// Extrato de conta que o app lê (o "Extrato por período" da Caixa): importação de
    /// extratos.
    Statement,
    /// Cupom fiscal ou recibo de compra com itens: abre a compra preenchida.
    Receipt,
    /// Comprovante de Pix, transferência, TED, DOC, depósito ou pagamento.
    PaymentProof,
    /// Boleto (linha digitável, beneficiário e vencimento).
    Bill,
    /// Tem texto, mas não é nada que o app importe sozinho.
    Document,
    /// Sem texto legível (ou sem OCR neste sistema).
    Unreadable,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImageAnalysis {
    pub kind: ImageKind,
    /// O que foi reconhecido ("Cupom fiscal: 12 itens, total R$ 87,40").
    pub label: String,
    /// O que fazer ou por que não deu.
    pub note: Option<String>,
    /// A leitura do cupom (para abrir a compra), quando é cupom.
    pub reading: Option<PhotoReading>,
    /// Linhas de texto lidas.
    #[ts(type = "number")]
    pub lines: i64,
}

fn has(text: &str, words: &[&str]) -> bool {
    words.iter().any(|w| text.contains(w))
}

/// Tipo pelo texto (dobrado: minúsculas, sem acentos) e pela leitura do cupom.
pub fn classify(text: &str, items: usize, total: bool, tier: Tier) -> ImageKind {
    if text.trim().is_empty() {
        return ImageKind::Unreadable;
    }
    // Antes do comprovante: o extrato tem "PIX" nos históricos.
    if super::caixa::looks_like(text) {
        return ImageKind::Statement;
    }
    let proof = has(text, &["comprovante", "recibo de transferencia", "receipt"])
        && has(
            text,
            &[
                "pix",
                "transferencia",
                "ted",
                " doc ",
                "deposito",
                "pagamento efetuado",
                "pagamento realizado",
                "transfer",
            ],
        )
        || has(text, &["id da transacao", "end to end", "e2e", "chave pix"]);
    if proof {
        return ImageKind::PaymentProof;
    }
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    let bill = has(text, &["linha digitavel", "boleto"])
        || (has(text, &["beneficiario", "cedente"]) && has(text, &["vencimento"]))
        || (digits.len() >= 47 && has(text, &["vencimento"]));
    if bill {
        return ImageKind::Bill;
    }
    let receipt_words = crate::receipts::nfce::looks_like(text)
        || has(
            text,
            &[
                "cupom fiscal",
                "documento auxiliar",
                "nota fiscal",
                "extrato do cupom",
                "sat no",
                "total a pagar",
                "valor a pagar",
                "subtotal",
            ],
        );
    if receipt_words && (items > 0 || total) || items >= 2 && total || tier >= Tier::Balanced {
        return ImageKind::Receipt;
    }
    ImageKind::Document
}

/// Lê a imagem no computador e diz o que ela é (rede nenhuma, IA nenhuma).
pub fn analyze(api: &Api, a: &PhotoArgs) -> Result<ImageAnalysis> {
    let args = PhotoArgs {
        use_ai: Some(false),
        ..a.clone()
    };
    let reading = match crate::receipts::ocr::read_photo(api, &args) {
        Ok(r) => r,
        Err(e) => {
            return Ok(ImageAnalysis {
                kind: ImageKind::Unreadable,
                label: t("nucleo-importacao-imagem-sem-texto"),
                note: Some(e.message()),
                reading: None,
                lines: 0,
            });
        }
    };
    let text = crate::import::text::fold(&reading.reading.rows.join("\n"));
    let kind = classify(
        &format!(" {text} "),
        reading.reading.items.len(),
        reading.reading.total.is_some(),
        reading.tier,
    );
    let lines = reading.reading.rows.len() as i64;
    let (label, note) = match kind {
        ImageKind::Statement => (
            t("nucleo-importacao-tipo-extrato-caixa"),
            Some(t("nucleo-importacao-dica-extrato-caixa")),
        ),
        ImageKind::Receipt => (
            crate::i18n::t_args(
                "nucleo-importacao-imagem-cupom",
                &crate::i18n::args(&[("itens", (reading.reading.items.len() as i64).into())]),
            ),
            Some(t(match reading.tier {
                Tier::Checked | Tier::Balanced => "nucleo-importacao-imagem-cupom-fechou",
                Tier::Unbalanced => "nucleo-importacao-imagem-cupom-revisar",
            })),
        ),
        ImageKind::PaymentProof => (
            t("nucleo-importacao-imagem-comprovante"),
            Some(t("nucleo-importacao-imagem-comprovante-dica")),
        ),
        ImageKind::Bill => (
            t("nucleo-importacao-imagem-boleto"),
            Some(t("nucleo-importacao-imagem-boleto-dica")),
        ),
        ImageKind::Document => (
            t("nucleo-importacao-imagem-documento"),
            Some(t("nucleo-importacao-imagem-documento-dica")),
        ),
        ImageKind::Unreadable => (t("nucleo-importacao-imagem-sem-texto"), None),
    };
    Ok(ImageAnalysis {
        kind,
        label,
        note,
        reading: (kind == ImageKind::Receipt).then_some(reading),
        lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(text: &str, items: usize, total: bool, tier: Tier) -> ImageKind {
        classify(
            &format!(" {} ", crate::import::text::fold(text)),
            items,
            total,
            tier,
        )
    }

    #[test]
    fn caixa_statement_photo_is_a_statement_not_a_proof() {
        let text = include_str!("fixtures/caixa-extrato-periodo.ocr.txt");
        assert_eq!(k(text, 0, false, Tier::Unbalanced), ImageKind::Statement);
    }

    #[test]
    fn classifies_by_the_text_read() {
        assert_eq!(k("", 0, false, Tier::Unbalanced), ImageKind::Unreadable);
        assert_eq!(
            k(
                "Comprovante de transferência\nPix enviado\nValor R$ 50,00\nID da transação E2E",
                1,
                true,
                Tier::Unbalanced
            ),
            ImageKind::PaymentProof
        );
        assert_eq!(
            k(
                "Banco X\nBeneficiário: Escola\nVencimento 10/11/2026\n23790.12345 60000.123456 78901.234567 8 12345678901234",
                0,
                false,
                Tier::Unbalanced
            ),
            ImageKind::Bill
        );
        assert_eq!(
            k(
                "MERCADO BOM PRECO\nDOCUMENTO AUXILIAR DA NOTA FISCAL DE CONSUMIDOR ELETRONICA\nARROZ 5KG 25,90\nTOTAL 25,90",
                1,
                true,
                Tier::Checked
            ),
            ImageKind::Receipt
        );
        // Receipt from another country, no Brazilian words: the sum matched.
        assert_eq!(
            k(
                "CORNER SHOP\nMILK 2.10\nBREAD 3.40\nTOTAL 5.50",
                2,
                true,
                Tier::Balanced
            ),
            ImageKind::Receipt
        );
        assert_eq!(
            k(
                "Contrato de locação\nCláusula primeira",
                0,
                false,
                Tier::Unbalanced
            ),
            ImageKind::Document
        );
    }
}
