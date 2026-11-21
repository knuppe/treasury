//! Base64 (RFC 4648) para anexos trafegando em JSON. Aceita o alfabeto padrão e o
//! "URL safe", com ou sem `=`, ignora espaços/quebras de linha e um prefixo `data:…;base64,`.

use crate::error::{Error, Result};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

fn value(c: u8) -> Option<u32> {
    Some(match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' | b'-' => 62,
        b'/' | b'_' => 63,
        _ => return None,
    } as u32)
}

/// Tamanho decodificado aproximado (para recusar arquivos grandes antes de decodificar).
pub fn decoded_len_estimate(s: &str) -> usize {
    s.len() / 4 * 3 + 3
}

pub fn decode(input: &str) -> Result<Vec<u8>> {
    let err = || Error::invalid(crate::i18n::t("nucleo-lancamentos-erro-base64"));
    let mut s = input.trim();
    if let Some(rest) = s.strip_prefix("data:") {
        s = rest.split_once(";base64,").ok_or_else(err)?.1;
    }
    let mut out = Vec::with_capacity(decoded_len_estimate(s));
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    let mut padding = 0usize;
    let mut symbols = 0usize;
    for &c in s.as_bytes() {
        if c.is_ascii_whitespace() {
            continue;
        }
        if c == b'=' {
            padding += 1;
            continue;
        }
        if padding > 0 {
            // Nada depois do preenchimento.
            return Err(err());
        }
        let v = value(c).ok_or_else(err)?;
        symbols += 1;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    if symbols % 4 == 1 || padding > 2 || (padding > 0 && !(symbols + padding).is_multiple_of(4)) {
        return Err(err());
    }
    // Bits que sobram precisam ser zero (forma canônica).
    if acc != 0 {
        return Err(err());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_rfc_vectors() {
        for (raw, enc) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(encode(raw.as_bytes()), enc);
            assert_eq!(decode(enc).unwrap(), raw.as_bytes());
        }
        let bin: Vec<u8> = (0..=255u8).collect();
        assert_eq!(decode(&encode(&bin)).unwrap(), bin);
        // Sem preenchimento, com quebras de linha, URL safe e data URL.
        assert_eq!(decode("Zm9vYg").unwrap(), b"foob");
        assert_eq!(decode("Zm9v\nYmFy\n").unwrap(), b"foobar");
        assert_eq!(decode("-_8=").unwrap(), decode("+/8=").unwrap());
        assert_eq!(
            decode("data:text/plain;base64,Zm9v").unwrap(),
            b"foo".to_vec()
        );
        assert!(decode("Zm9v!").is_err());
        assert!(decode("Z").is_err());
        assert!(decode("Zg==Zg==").is_err());
        assert!(decode("Zh==").is_err(), "bits sobrando");
    }
}
