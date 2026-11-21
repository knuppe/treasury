//! Tamanho da embalagem e quantidade em kg, L ou unidades (docs/consumption.md, seção 5.1): sai
//! da unidade do cupom (KG nos itens de balança) e da descrição ("5KG", "500G", "LT 350ML",
//! "C/12", "2L", "6X1L"). Sem isso não há preço por kg, então é central.

use std::sync::LazyLock;

use regex::Regex;

static SIZE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(\d+(?:[.,]\d+)?)\s*(kg|kgs|g|gr|grs|gramas|l|lt|lts|litro|litros|ml|fl\.?\s?oz|floz|oz|lbs?|gal|gallons?|qt)\b",
    )
    .unwrap()
});
/// "12 ct", "6 pk", "pack of 12", "24 count" (cupons em inglês).
static MULTI_EN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d{1,3})\s*(?:ct|pk|pack|count)\b|\bpack\s+of\s+(\d{1,3})\b").unwrap()
});
static BARE_GAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(half\s+|1/2\s*)?gal(?:lon)?\b").unwrap());
static DOZEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(?:dozen|dz)\b").unwrap());

/// Libra, onça, galão, quarto e onça fluida em kg e L (o preço por unidade fica sempre
/// guardado em kg, L ou unidade; a tela mostra por libra e por galão onde se usa isso).
pub const KG_PER_LB: f64 = 0.453_592_37;
pub const KG_PER_OZ: f64 = 0.028_349_523_125;
pub const L_PER_GAL: f64 = 3.785_411_784;
pub const L_PER_QT: f64 = 0.946_352_946;
pub const L_PER_FLOZ: f64 = 0.029_573_529_562_5;
static MULTI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:c/|cx\s*c/|com\s+)(\d{1,3})\b|\b(\d{1,3})\s*x\s*\d").unwrap()
});

/// Unidade base: `kg`, `l` ou `un`.
pub type BaseUnit = &'static str;

fn num(s: &str) -> Option<f64> {
    s.replace(',', ".").parse().ok().filter(|v: &f64| *v > 0.0)
}

/// Tamanho de uma embalagem pela descrição: (quantidade em kg, L ou unidades, unidade).
pub fn pack_of(description: &str) -> Option<(f64, BaseUnit)> {
    let count = MULTI
        .captures(description)
        .or_else(|| MULTI_EN.captures(description))
        .and_then(|c| c.get(1).or(c.get(2)))
        .and_then(|m| num(m.as_str()))
        .or_else(|| DOZEN.is_match(description).then_some(12.0))
        .unwrap_or(1.0);
    let size = SIZE.captures_iter(description).last();
    match size {
        Some(c) => {
            let v = num(&c[1])?;
            let unit: String = c[2]
                .to_ascii_lowercase()
                .chars()
                .filter(|ch| ch.is_ascii_alphabetic())
                .collect();
            let (q, u) = match unit.as_str() {
                "kg" | "kgs" => (v, "kg"),
                "g" | "gr" | "grs" | "gramas" => (v / 1000.0, "kg"),
                "lb" | "lbs" => (v * KG_PER_LB, "kg"),
                "oz" => (v * KG_PER_OZ, "kg"),
                "ml" => (v / 1000.0, "l"),
                "floz" => (v * L_PER_FLOZ, "l"),
                "gal" | "gallon" | "gallons" => (v * L_PER_GAL, "l"),
                "qt" => (v * L_PER_QT, "l"),
                _ => (v, "l"),
            };
            Some((q * count, u))
        }
        // "MILK GAL", "HALF GAL": o galão sem número na frente.
        None if BARE_GAL.is_match(description) => {
            let half = BARE_GAL
                .captures(description)
                .is_some_and(|c| c.get(1).is_some());
            Some((L_PER_GAL * if half { 0.5 } else { 1.0 } * count, "l"))
        }
        None if count > 1.0 => Some((count, "un")),
        None => None,
    }
}

/// Quantidade total do item em kg, L ou unidades, pela unidade do cupom e pela descrição.
pub fn base_of(
    description: &str,
    quantity: Option<&str>,
    unit: Option<&str>,
) -> Option<(f64, BaseUnit)> {
    let qty = quantity.and_then(num).unwrap_or(1.0);
    let unit = unit
        .map(|u| u.trim().to_ascii_lowercase())
        .unwrap_or_default();
    match unit.as_str() {
        "kg" | "kgs" | "kilo" => return Some((qty, "kg")),
        "g" | "gr" => return Some((qty / 1000.0, "kg")),
        "l" | "lt" | "lts" => return Some((qty, "l")),
        "ml" => return Some((qty / 1000.0, "l")),
        "lb" | "lbs" => return Some((qty * KG_PER_LB, "kg")),
        "oz" => return Some((qty * KG_PER_OZ, "kg")),
        "gal" => return Some((qty * L_PER_GAL, "l")),
        "qt" => return Some((qty * L_PER_QT, "l")),
        "floz" | "fl oz" => return Some((qty * L_PER_FLOZ, "l")),
        _ => {}
    }
    match pack_of(description) {
        Some((p, u)) => Some((p * qty, u)),
        None => Some((qty, "un")),
    }
}

/// Preço por kg, L ou unidade (centavos), arredondado.
pub fn base_price(total: i64, base_qty: f64) -> Option<i64> {
    (base_qty > 0.0).then(|| (total as f64 / base_qty).round() as i64)
}

/// Quantidade como texto decimal curto ("0.535", "5", "12").
pub fn qty_text(v: f64) -> String {
    let s = format!("{v:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_from_descriptions_and_units() {
        assert_eq!(pack_of("ARROZ TIPO1 EXEMPLO 5KG"), Some((5.0, "kg")));
        assert_eq!(pack_of("CAFE TORRADO 500G"), Some((0.5, "kg")));
        assert_eq!(pack_of("REFRI LT 350ML"), Some((0.35, "l")));
        assert_eq!(pack_of("AGUA MINERAL 1,5L"), Some((1.5, "l")));
        assert_eq!(pack_of("LEITE UHT 6X1L"), Some((6.0, "l")));
        assert_eq!(pack_of("OVOS BRANCOS C/12"), Some((12.0, "un")));
        assert_eq!(pack_of("ALFACE CRESPA"), None);
        // Item de balança: a quantidade do cupom já é em kg.
        assert_eq!(
            base_of("BANANA PRATA", Some("0,535"), Some("KG")),
            Some((0.535, "kg"))
        );
        // Duas embalagens de 5 kg.
        assert_eq!(
            base_of("ARROZ 5KG", Some("2"), Some("UN")),
            Some((10.0, "kg"))
        );
        assert_eq!(
            base_of("ALFACE CRESPA", Some("1"), Some("UN")),
            Some((1.0, "un"))
        );
        assert_eq!(base_price(2_490, 5.0), Some(498));
        // Medidas dos EUA: guardadas em kg e L.
        let near = |got: Option<(f64, BaseUnit)>, want: f64, unit: &str| {
            let (q, u) = got.unwrap();
            assert!((q - want).abs() < 1e-6 && u == unit, "{q} {u}");
        };
        near(pack_of("GROUND BEEF 80/20 2 LB"), 0.907_184_74, "kg");
        near(pack_of("CHEDDAR CHEESE 8 OZ"), 0.226_796_185, "kg");
        near(pack_of("WHOLE MILK 1 GAL"), 3.785_411_784, "l");
        near(pack_of("SODA 12 FL OZ 6 PK"), 6.0 * 12.0 * L_PER_FLOZ, "l");
        near(pack_of("ORANGE JUICE 52 FL. OZ"), 52.0 * L_PER_FLOZ, "l");
        near(pack_of("GV WHL MLK GAL"), L_PER_GAL, "l");
        near(pack_of("2% MILK HALF GAL"), L_PER_GAL / 2.0, "l");
        assert_eq!(pack_of("LARGE EGGS 12 CT"), Some((12.0, "un")));
        assert_eq!(pack_of("EGGS GRADE A DOZEN"), Some((12.0, "un")));
        assert_eq!(pack_of("PAPER TOWELS PACK OF 6"), Some((6.0, "un")));
        // Peso variável em libras ("2.31 lb @ 3.99/lb").
        near(
            base_of("BANANAS", Some("2.31"), Some("lb")),
            2.31 * KG_PER_LB,
            "kg",
        );
        assert_eq!(qty_text(0.535), "0.535");
        assert_eq!(qty_text(5.0), "5");
    }
}
