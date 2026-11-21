//! Redação de dados sensíveis em textos que o agente grava (notas de procedimentos e
//! feedback ao desenvolvedor): CPF, CNPJ, números de cartão (com Luhn), conta/agência,
//! e-mail, telefone e sequências longas de dígitos. Mantém os 2 últimos dígitos onde ajudam
//! a conferir.

use regex::Regex;
use std::sync::LazyLock;

static CPF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{3}\.?\d{3}\.?\d{3}-?\d{2}\b").expect("regex"));
static CNPJ: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{2}\.?\d{3}\.?\d{3}/?\d{4}-?\d{2}\b").expect("regex"));
static CARD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:\d[ -]?){13,19}\b").expect("regex"));
static EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}\b").expect("regex"));
static PHONE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(?\b\d{2}\)?\s?9?\d{4}-?\d{4}\b").expect("regex"));
static ACCOUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(ag[eê]ncia|ag\.?|conta(?:\s+corrente)?|c/c|cc)\s*:?\s*(\d[\d.\-/ ]{2,14}\d)",
    )
    .expect("regex")
});

fn luhn(digits: &str) -> bool {
    let mut sum = 0;
    let mut double = false;
    for c in digits.chars().rev() {
        let Some(mut d) = c.to_digit(10) else {
            return false;
        };
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }
    sum % 10 == 0
}

fn tail(s: &str, n: usize) -> String {
    let d: Vec<char> = s.chars().filter(|c| c.is_ascii_digit()).collect();
    d[d.len().saturating_sub(n)..].iter().collect()
}

/// Devolve o texto redigido e quantas trocas foram feitas.
pub fn redact(text: &str) -> (String, u32) {
    let mut count = 0u32;
    let mut out = EMAIL
        .replace_all(text, |_: &regex::Captures<'_>| {
            count += 1;
            "[e-mail]".to_string()
        })
        .into_owned();
    out = CARD
        .replace_all(&out, |c: &regex::Captures<'_>| {
            let m = &c[0];
            let digits: String = m.chars().filter(|c| c.is_ascii_digit()).collect();
            if (13..=19).contains(&digits.len()) && luhn(&digits) {
                count += 1;
                format!("[cartão final {}]", tail(m, 4))
            } else {
                m.to_string()
            }
        })
        .into_owned();
    out = CNPJ
        .replace_all(&out, |_: &regex::Captures<'_>| {
            count += 1;
            "[CNPJ]".to_string()
        })
        .into_owned();
    out = CPF
        .replace_all(&out, |_: &regex::Captures<'_>| {
            count += 1;
            "[CPF]".to_string()
        })
        .into_owned();
    out = ACCOUNT
        .replace_all(&out, |c: &regex::Captures<'_>| {
            count += 1;
            format!("{} [final {}]", &c[1], tail(&c[2], 2))
        })
        .into_owned();
    out = PHONE
        .replace_all(&out, |_: &regex::Captures<'_>| {
            count += 1;
            "[telefone]".to_string()
        })
        .into_owned();
    (out, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_sensitive_data_and_keeps_amounts() {
        let src = "Titular: Ana, CPF 123.456.789-09, e-mail ana@exemplo.com, tel (11) 98765-4321.\n\
                   Agência 1234 Conta 56789-0 · Cartão 4111 1111 1111 1111 · CNPJ 12.345.678/0001-95\n\
                   Compra PADARIA R$ 1.234,56 em 14/10/2026; protocolo 2026101400012345";
        let (out, n) = redact(src);
        assert!(!out.contains("123.456.789-09"), "{out}");
        assert!(out.contains("[CPF]"));
        assert!(out.contains("[e-mail]") && out.contains("[telefone]"));
        assert!(out.contains("[cartão final 1111]"));
        assert!(out.contains("[CNPJ]"));
        assert!(
            out.contains("Agência [final 34]") && out.contains("Conta [final 90]"),
            "{out}"
        );
        assert!(
            out.contains("R$ 1.234,56") && out.contains("14/10/2026"),
            "valores e datas ficam"
        );
        assert!(n >= 6);
        // Número que não passa no Luhn não é tratado como cartão.
        let (o2, _) = redact("Pedido 1234567890123");
        assert!(o2.contains("1234567890123"));
    }
}
