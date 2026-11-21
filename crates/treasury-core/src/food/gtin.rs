//! GTIN (EAN-13, UPC-A, EAN-8, GTIN-14) dos produtos (docs/gtin-inflation.md §3): o código de
//! barras com o dígito verificador certo, guardado sempre com 14 dígitos (zeros à esquerda), é
//! a chave natural do produto em `product_gtin` (V138). Códigos de uso interno ou restrito não
//! são GTIN de produto e ficam como código da loja.

/// O dígito verificador (o último) confere.
fn check_digit_ok(digits: &[u32]) -> bool {
    let (body, check) = digits.split_at(digits.len() - 1);
    let sum: u32 = body
        .iter()
        .rev()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { d * 3 } else { *d })
        .sum();
    (10 - sum % 10) % 10 == check[0]
}

/// GTIN-14 normalizado de um código de 8, 12, 13 ou 14 dígitos com o dígito verificador certo,
/// ou `None` para o que não é GTIN de produto:
/// - peso variável e balança (indicador 9; prefixos 02x e 200–299);
/// - uso interno da empresa (04x), cupons (05x), vales e recibos (980–999);
/// - EAN-8 de circulação restrita (começa com 0 ou 2) e o código só de zeros.
///
/// Depois de normalizar, `0789…` e `789…` são o mesmo GTIN.
pub fn gtin14(code: &str) -> Option<String> {
    let c = code.trim();
    if !matches!(c.len(), 8 | 12 | 13 | 14) || !c.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let digits: Vec<u32> = c.bytes().map(|b| (b - b'0') as u32).collect();
    if digits.iter().all(|d| *d == 0) || !check_digit_ok(&digits) {
        return None;
    }
    if c.len() == 8 && matches!(c.as_bytes()[0], b'0' | b'2') {
        return None;
    }
    let g = format!("{c:0>14}");
    let prefix = &g[1..4];
    let restricted = g.starts_with('9')
        || prefix.starts_with('2')
        || ["02", "04", "05"].iter().any(|p| prefix.starts_with(p))
        || ("980"..="999").contains(&prefix);
    (!restricted).then_some(g)
}

/// Confere o código digitado pela pessoa e devolve o GTIN-14, ou um erro que diz o que está
/// errado: o tamanho, o dígito verificador ou um código de uso interno da loja.
pub fn check(code: &str) -> crate::error::Result<String> {
    use crate::error::Error;
    use crate::i18n::t;
    let c: String = code
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    if !matches!(c.len(), 8 | 12 | 13 | 14) || !c.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::invalid(t("nucleo-alimentacao-gtin-erro-tamanho")));
    }
    let digits: Vec<u32> = c.bytes().map(|b| (b - b'0') as u32).collect();
    if digits.iter().all(|d| *d == 0) || !check_digit_ok(&digits) {
        return Err(Error::invalid(t("nucleo-alimentacao-gtin-erro-digito")));
    }
    gtin14(&c).ok_or_else(|| Error::invalid(t("nucleo-alimentacao-gtin-erro-interno")))
}

/// Como mostrar: o EAN-13 (ou UPC-A, EAN-8) sem os zeros que o GTIN-14 acrescentou.
pub fn display(g14: &str) -> String {
    let t = g14.trim_start_matches('0');
    let len = [8, 12, 13]
        .into_iter()
        .find(|n| *n >= t.len())
        .unwrap_or(14);
    format!("{t:0>len$}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_rejects_internal_codes() {
        // Made-up GTINs, with the check digit computed.
        assert_eq!(gtin14("7890000000017").as_deref(), Some("07890000000017"));
        assert_eq!(gtin14("07890000000017").as_deref(), Some("07890000000017"));
        assert_eq!(gtin14("7890000000018"), None);
        // Scale (2…), internal use (04…), coupon (05…), voucher (98…).
        assert_eq!(gtin14("2000000000015"), None);
        assert_eq!(gtin14("0400000000004"), None);
        assert_eq!(gtin14("0500000000009"), None);
        assert_eq!(gtin14("9800000000009"), None);
        assert_eq!(gtin14("0000000000000"), None);
        assert_eq!(gtin14("12345"), None);
        // US UPC-A becomes a GTIN-14 with two zeros.
        assert_eq!(gtin14("036000291452").as_deref(), Some("00036000291452"));
        assert_eq!(display("07890000000017"), "7890000000017");
        // What the person types: spaces and dashes do not count; each error has its own message.
        assert_eq!(check("789 0000 00001-7").unwrap(), "07890000000017");
        let msg = |c: &str| check(c).unwrap_err().message();
        assert_ne!(msg("78900"), msg("7890000000018"));
        assert_ne!(msg("7890000000018"), msg("2000000000015"));
        assert_eq!(display("00036000291452"), "036000291452");
    }
}
