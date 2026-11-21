//! Texto dos arquivos importados: detecção de codificação e comparação de descrições.
//!
//! Bancos brasileiros ainda geram muitos arquivos em Windows-1252/ISO-8859-1, às vezes
//! declarando outra codificação no cabeçalho. A regra é: BOM manda; depois, um arquivo que é
//! UTF-8 válido é UTF-8; senão é Windows-1252 (superconjunto prático do ISO-8859-1).

use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};

use crate::error::{Error, Result};
use crate::transactions::fold_accent;

#[derive(Debug, Clone)]
pub struct Decoded {
    pub text: String,
    /// `utf-8`, `windows-1252`, `iso-8859-1`, `utf-16le`, `utf-16be`.
    pub encoding: String,
}

fn canonical_label(label: &str) -> Option<(&'static Encoding, &'static str)> {
    let l = label.trim().to_ascii_lowercase().replace('_', "-");
    Some(match l.as_str() {
        "utf-8" | "utf8" | "unicode-1-1-utf-8" => (UTF_8, "utf-8"),
        "windows-1252" | "cp1252" | "1252" | "win-1252" | "ansi" => (WINDOWS_1252, "windows-1252"),
        "iso-8859-1" | "iso8859-1" | "8859-1" | "latin1" | "latin-1" | "l1" => {
            (WINDOWS_1252, "iso-8859-1")
        }
        "utf-16le" | "utf-16" => (UTF_16LE, "utf-16le"),
        "utf-16be" => (UTF_16BE, "utf-16be"),
        "usascii" | "us-ascii" | "ascii" | "none" => return None,
        other => {
            let enc = Encoding::for_label(other.as_bytes())?;
            (enc, enc.name())
        }
    })
}

/// Decodifica com uma codificação escolhida pelo usuário.
pub fn decode_with(bytes: &[u8], label: &str) -> Result<Decoded> {
    let (enc, name) = canonical_label(label).ok_or_else(|| {
        Error::invalid(crate::i18n::t_args(
            "nucleo-importacao-erro-codificacao",
            &crate::i18n::args(&[("valor", label.to_string().into())]),
        ))
    })?;
    let bytes = strip_bom_for(bytes, enc);
    let (text, _, _) = enc.decode(bytes);
    Ok(Decoded {
        text: text.into_owned(),
        encoding: name.to_lowercase(),
    })
}

fn strip_bom_for<'a>(bytes: &'a [u8], enc: &'static Encoding) -> &'a [u8] {
    if enc == UTF_8 {
        bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes)
    } else {
        bytes
    }
}

/// Detecta a codificação. `declared` é o que o próprio arquivo diz (cabeçalho OFX), usado
/// só para nomear a codificação de 8 bits quando o conteúdo não é UTF-8.
pub fn decode_auto(bytes: &[u8], declared: Option<&str>) -> Decoded {
    if let Some(rest) = bytes.strip_prefix(b"\xEF\xBB\xBF") {
        return Decoded {
            text: String::from_utf8_lossy(rest).into_owned(),
            encoding: "utf-8".into(),
        };
    }
    if bytes.starts_with(b"\xFF\xFE") {
        let (t, _, _) = UTF_16LE.decode(bytes);
        return Decoded {
            text: t.into_owned(),
            encoding: "utf-16le".into(),
        };
    }
    if bytes.starts_with(b"\xFE\xFF") {
        let (t, _, _) = UTF_16BE.decode(bytes);
        return Decoded {
            text: t.into_owned(),
            encoding: "utf-16be".into(),
        };
    }
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Decoded {
            text: s.to_string(),
            encoding: "utf-8".into(),
        };
    }
    let name = match declared.and_then(canonical_label) {
        Some((enc, name)) if enc != UTF_8 && enc != UTF_16LE && enc != UTF_16BE => {
            let (t, _, _) = enc.decode(bytes);
            return Decoded {
                text: t.into_owned(),
                encoding: name.to_lowercase(),
            };
        }
        _ => "windows-1252",
    };
    let (t, _, _) = WINDOWS_1252.decode(bytes);
    Decoded {
        text: t.into_owned(),
        encoding: name.into(),
    }
}

/// Decodifica conforme a escolha do usuário ou detecta.
pub fn decode(bytes: &[u8], hint: Option<&str>, declared: Option<&str>) -> Result<Decoded> {
    match hint.map(str::trim).filter(|h| !h.is_empty()) {
        Some(h) => decode_with(bytes, h),
        None => Ok(decode_auto(bytes, declared)),
    }
}

/// Texto para comparação: minúsculas, sem acentos, só letras e dígitos separados por um
/// espaço. Diferente de `normalize_description`, mantém números (úteis em regras).
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = true;
    for ch in s.chars() {
        let c = fold_accent(ch);
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            space = false;
        } else if !space {
            out.push(' ');
            space = true;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Só tira acentos e põe em minúsculas, preservando pontuação e espaços (para regex).
pub fn fold_keep_punct(s: &str) -> String {
    s.chars()
        .map(fold_accent)
        .flat_map(char::to_lowercase)
        .collect()
}

/// Semelhança entre duas descrições (0–1): coeficiente de Dice sobre bigramas de
/// caracteres das descrições normalizadas, com atalho para igualdade e contenção.
pub fn similarity(a: &str, b: &str) -> f64 {
    let na = crate::transactions::normalize_description(a);
    let nb = crate::transactions::normalize_description(b);
    if na.is_empty() || nb.is_empty() {
        return 0.0;
    }
    if na == nb {
        return 1.0;
    }
    let (short, long) = if na.len() <= nb.len() {
        (&na, &nb)
    } else {
        (&nb, &na)
    };
    if short.len() >= 5 && long.contains(short.as_str()) {
        return 0.9;
    }
    let bigrams = |s: &str| {
        let cs: Vec<char> = s.chars().filter(|c| *c != ' ').collect();
        let mut v: Vec<(char, char)> = cs.windows(2).map(|w| (w[0], w[1])).collect();
        v.sort_unstable();
        v
    };
    let ba = bigrams(&na);
    let bb = bigrams(&nb);
    if ba.is_empty() || bb.is_empty() {
        return 0.0;
    }
    // Interseção de multiconjuntos ordenados.
    let (mut i, mut j, mut common) = (0, 0, 0usize);
    while i < ba.len() && j < bb.len() {
        match ba[i].cmp(&bb[j]) {
            std::cmp::Ordering::Equal => {
                common += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    2.0 * common as f64 / (ba.len() + bb.len()) as f64
}

/// Palavras que identificam o estabelecimento (sem meios de pagamento e preposições).
fn key_words(s: &str) -> Vec<String> {
    crate::transactions::normalize_description(s)
        .split(' ')
        .filter(|w| w.len() >= 4 && crate::rules::significant(w))
        .map(str::to_string)
        .collect()
}

/// Descrições "muito parecidas" para apontar possível duplicado (usado junto com a mesma
/// data e o mesmo valor). Com palavras-chave dos dois lados, decide pela metade delas em
/// comum ("Compra no débito - NETFLIX.COM" × "Netflix.com assinatura" sim; "Compra no
/// débito - PADARIA" × "Compra no débito - FARMÁCIA" não); senão, pelos bigramas.
pub fn very_similar(a: &str, b: &str) -> bool {
    let sim = similarity(a, b);
    let (ka, kb) = (key_words(a), key_words(b));
    if ka.is_empty() || kb.is_empty() {
        return sim >= 0.6;
    }
    let common = ka.iter().filter(|w| kb.contains(w)).count();
    common * 2 >= ka.len().min(kb.len()) || sim >= 0.9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_utf8_latin1_and_bom() {
        let latin1 = b"PADARIA S\xc3O JO\xc3O"; // "SÃO JOÃO" em Windows-1252
        let d = decode_auto(latin1, None);
        assert_eq!(d.encoding, "windows-1252");
        assert_eq!(d.text, "PADARIA SÃO JOÃO");
        let d = decode_auto(latin1, Some("8859-1"));
        assert_eq!(d.encoding, "iso-8859-1");
        assert_eq!(d.text, "PADARIA SÃO JOÃO");

        let d = decode_auto("Ação".as_bytes(), Some("1252"));
        assert_eq!(d.encoding, "utf-8", "UTF-8 válido vence o cabeçalho");
        assert_eq!(d.text, "Ação");

        let d = decode_auto(b"\xEF\xBB\xBFData;Valor", None);
        assert_eq!(d.text, "Data;Valor");
        assert_eq!(d.encoding, "utf-8");

        let d = decode_with(b"caf\xe9", "latin1").unwrap();
        assert_eq!(d.text, "café");
        assert!(decode_with(b"x", "klingon").is_err());
    }

    #[test]
    fn folds_and_compares() {
        assert_eq!(fold("PAG*Padaria São João  12"), "pag padaria sao joao 12");
        assert_eq!(fold_keep_punct("Pix ENVIADO: Joã"), "pix enviado: joa");
        assert!(very_similar("UBER *TRIP 123456789", "Uber Trip"));
        assert!(very_similar("PADARIA SAO JOAO", "PADARIA S JOAO LTDA"));
        assert!(!very_similar("NETFLIX", "SPOTIFY"));
        assert!(very_similar(
            "Compra no débito - NETFLIX.COM",
            "Netflix.com assinatura"
        ));
        assert!(!very_similar(
            "Compra no débito - PADARIA",
            "Compra no débito - FARMACIA"
        ));
        assert!(similarity("", "x") == 0.0);
    }
}
