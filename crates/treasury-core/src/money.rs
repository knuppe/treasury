//! Dinheiro: inteiro na menor unidade da moeda (centavos) + código ISO 4217.
//! Nunca ponto flutuante.

use crate::error::{Error, Result};
use crate::i18n::{args, t_args};

/// Erro de um campo com o texto digitado (`valor`).
fn invalid(key: &str, value: &str) -> Error {
    Error::invalid(t_args(key, &args(&[("valor", value.to_string().into())])))
}

/// Casas decimais da menor unidade por moeda (ISO 4217). Padrão 2.
pub fn minor_units(currency: &str) -> u32 {
    match currency {
        "JPY" | "KRW" | "CLP" | "PYG" | "VND" => 0,
        "BHD" | "KWD" | "OMR" | "TND" => 3,
        _ => 2,
    }
}

pub fn validate_currency(code: &str) -> Result<String> {
    let c = code.trim().to_ascii_uppercase();
    if c.len() == 3 && c.chars().all(|ch| ch.is_ascii_uppercase()) {
        Ok(c)
    } else {
        Err(invalid("nucleo-base-erro-moeda", code))
    }
}

/// Formata no padrão pt-BR: `R$ 1.234,56`, `USD 1.234,56`, sinal `−` para negativos.
pub fn format_brl_style(amount_minor: i64, currency: &str) -> String {
    let units = minor_units(currency);
    let neg = amount_minor < 0;
    let abs = amount_minor.unsigned_abs();
    let div = 10u64.pow(units);
    let int = abs / div;
    let frac = abs % div;
    let int_str = group_thousands(int);
    let num = if units == 0 {
        int_str
    } else {
        format!("{int_str},{frac:0width$}", width = units as usize)
    };
    let prefix = if currency == "BRL" {
        "R$".to_string()
    } else {
        currency.to_string()
    };
    format!("{}{prefix} {num}", if neg { "−" } else { "" })
}

fn group_thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(ch);
    }
    out
}

/// Interpreta valores escritos como `1.234,56`, `-1234,5`, `1234.56` (ponto decimal
/// só quando não há vírgula e há no máximo `units` casas depois do último ponto),
/// `R$ 10`, `(12,00)` (negativo contábil). Devolve a menor unidade.
pub fn parse_amount(input: &str, currency: &str) -> Result<i64> {
    let units = minor_units(currency) as usize;
    let err = || invalid("nucleo-base-erro-valor", input);
    let mut s: String = input
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}')
        .collect();
    let mut neg = false;
    if s.starts_with('(') && s.ends_with(')') {
        neg = true;
        s = s[1..s.len() - 1].to_string();
    }
    // Sinal antes do símbolo da moeda ("-R$ 40,90", como nos extratos de cartões de benefício).
    if let Some(rest) = s.strip_prefix(['-', '−'])
        && ["R$", "US$", "$"].iter().any(|p| rest.starts_with(p))
    {
        neg = !neg;
        s = rest.to_string();
    }
    for p in ["R$", "US$", "$"] {
        if let Some(rest) = s.strip_prefix(p) {
            s = rest.to_string();
        }
    }
    if s.len() > 3 && s.is_char_boundary(3) && s[..3].chars().all(|c| c.is_ascii_uppercase()) {
        s = s[3..].to_string();
    }
    if let Some(rest) = s.strip_prefix(['-', '−']) {
        neg = !neg;
        s = rest.to_string();
    } else if let Some(rest) = s.strip_prefix('+') {
        s = rest.to_string();
    }
    if let Some(rest) = s.strip_suffix('-') {
        neg = !neg;
        s = rest.to_string();
    }
    if s.is_empty() {
        return Err(err());
    }
    let (int_part, frac_part) = if let Some(idx) = s.rfind(',') {
        (s[..idx].replace('.', ""), s[idx + 1..].to_string())
    } else if let Some(idx) = s.rfind('.') {
        let frac = &s[idx + 1..];
        let dots = s.matches('.').count();
        if dots == 1 && frac.len() <= units.max(2) && (frac.len() != 3 || units >= 3) {
            (s[..idx].to_string(), frac.to_string())
        } else {
            (s.replace('.', ""), String::new())
        }
    } else {
        (s.clone(), String::new())
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return Err(err());
    }
    if !int_part.chars().all(|c| c.is_ascii_digit())
        || !frac_part.chars().all(|c| c.is_ascii_digit())
    {
        return Err(err());
    }
    if frac_part.len() > units {
        return Err(invalid("nucleo-base-erro-casas-decimais", input));
    }
    let int: i64 = if int_part.is_empty() {
        0
    } else {
        int_part.parse().map_err(|_| err())?
    };
    let frac: i64 = if frac_part.is_empty() {
        0
    } else {
        format!("{frac_part:0<units$}").parse().map_err(|_| err())?
    };
    let v = int
        .checked_mul(10i64.pow(units as u32))
        .and_then(|x| x.checked_add(frac))
        .ok_or_else(err)?;
    Ok(if neg { -v } else { v })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_decimal_currencies_keep_their_decimals() {
        assert_eq!(parse_amount("1.234", "KWD").unwrap(), 1_234);
        assert_eq!(parse_amount("12.345", "BHD").unwrap(), 12_345);
        // In 2-decimal currencies, "1.234" is still one thousand two hundred and thirty-four.
        assert_eq!(parse_amount("1.234", "BRL").unwrap(), 123_400);
    }

    #[test]
    fn formats_pt_br() {
        assert_eq!(format_brl_style(123456, "BRL"), "R$ 1.234,56");
        assert_eq!(format_brl_style(-5, "BRL"), "−R$ 0,05");
        assert_eq!(format_brl_style(100000000, "USD"), "USD 1.000.000,00");
        assert_eq!(format_brl_style(1500, "JPY"), "JPY 1.500");
    }

    #[test]
    fn parses_common_inputs() {
        assert_eq!(parse_amount("1.234,56", "BRL").unwrap(), 123456);
        assert_eq!(parse_amount("R$ 10", "BRL").unwrap(), 1000);
        assert_eq!(parse_amount("-1234,5", "BRL").unwrap(), -123450);
        assert_eq!(parse_amount("1234.56", "BRL").unwrap(), 123456);
        assert_eq!(parse_amount("1.234", "BRL").unwrap(), 123400);
        assert_eq!(parse_amount("(12,00)", "BRL").unwrap(), -1200);
        assert_eq!(parse_amount("−3,00", "BRL").unwrap(), -300);
        assert_eq!(parse_amount("50,00-", "BRL").unwrap(), -5000);
        assert!(parse_amount("abc", "BRL").is_err());
        assert!(parse_amount("1,234", "BRL").is_err());
        assert!(
            parse_amount("Débito", "BRL").is_err(),
            "non-ASCII text does not panic"
        );
        assert_eq!(parse_amount("USD10", "USD").unwrap(), 1000);
        assert_eq!(parse_amount("-R$ 40,90", "BRL").unwrap(), -4090);
        assert_eq!(parse_amount("R$ 1.365,00", "BRL").unwrap(), 136500);
        assert_eq!(parse_amount("-R$ 0,00", "BRL").unwrap(), 0);
    }
}
