//! Perfil de consumo do país (docs/design/consumption-review.md, seção 4): o que muda de um país
//! para outro na tela Consumo fica aqui, como dados, em vez de condições espalhadas. O imposto
//! já está no preço (Brasil, Europa) ou é somado no fim do cupom (EUA, Canadá); o imposto de
//! cada item vem do percentual do grupo de consumo, do que o cupom imprime ou de nada; as unidades; o idioma das
//! palavras do classificador; o leitor de documento fiscal; a série oficial de alimentos.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Onde o imposto sobre a venda aparece no cupom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaxAtTill {
    /// O preço de cada item já tem o imposto.
    Included,
    /// O imposto é somado no fim: pago = itens − desconto + imposto.
    Added,
}

/// De onde vem o imposto de cada item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ItemTax {
    /// Percentual do grupo de consumo (ou o impresso na NFC-e, rateado).
    Estimated,
    /// O que o cupom imprime, rateado pelos itens tributáveis.
    Printed,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Units {
    Metric,
    /// Libra, onça, galão.
    Us,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConsumptionProfile {
    pub country: String,
    pub tax: TaxAtTill,
    pub item_tax: ItemTax,
    pub units: Units,
    /// Idiomas das palavras do classificador, na ordem em que são tentados.
    pub words: Vec<String>,
    /// Leitor do documento fiscal pelo QR: `nfce`, `afip`, `atcud`, `verifactu`.
    pub reader: Option<String>,
    /// Série oficial de alimentos do país (`index_series.id`) e a reserva.
    pub official_series: Vec<String>,
    /// Selo "alto em" da rotulagem frontal (só o da Anvisa).
    pub high_in: bool,
}

/// O perfil de um país (ISO de duas letras). País desconhecido: o genérico (imposto no preço,
/// métrico, palavras em inglês e português, sem leitor nem série).
pub fn of(country: &str) -> ConsumptionProfile {
    let c = country.to_ascii_uppercase();
    let (tax, item_tax, units) = match c.as_str() {
        "BR" => (TaxAtTill::Included, ItemTax::Estimated, Units::Metric),
        "US" => (TaxAtTill::Added, ItemTax::Printed, Units::Us),
        "CA" => (TaxAtTill::Added, ItemTax::Printed, Units::Metric),
        _ => (TaxAtTill::Included, ItemTax::None, Units::Metric),
    };
    let words: &[&str] = match c.as_str() {
        "BR" | "PT" => &["pt"],
        "US" | "GB" | "CA" => &["en"],
        "ES" | "AR" | "CL" => &["es", "pt"],
        _ => &["en", "pt"],
    };
    let reader = match c.as_str() {
        "BR" => Some("nfce"),
        "AR" => Some("afip"),
        "PT" => Some("atcud"),
        "ES" => Some("verifactu"),
        _ => None,
    };
    let official: &[&str] = match c.as_str() {
        "BR" => &["ipca_food_home"],
        "US" => &["us_cpi_food_home", "cpi_food_us_oecd"],
        "PT" => &["hicp_food_pt"],
        "ES" => &["hicp_food_es"],
        "FR" => &["hicp_food_fr"],
        "DE" => &["hicp_food_de"],
        "GB" => &["cpi_food_gb_oecd"],
        "CA" => &["cpi_food_ca_oecd"],
        "CL" => &["cpi_food_cl_oecd"],
        _ => &[],
    };
    ConsumptionProfile {
        high_in: c == "BR",
        country: c,
        tax,
        item_tax,
        units,
        words: words.iter().map(|s| s.to_string()).collect(),
        reader: reader.map(String::from),
        official_series: official.iter().map(|s| s.to_string()).collect(),
    }
}

/// O perfil do país da família.
pub fn family(conn: &rusqlite::Connection) -> ConsumptionProfile {
    of(&crate::country::family_country(conn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_by_country() {
        let br = of("br");
        assert_eq!(
            (br.tax, br.item_tax, br.units),
            (TaxAtTill::Included, ItemTax::Estimated, Units::Metric)
        );
        assert!(br.high_in);
        let us = of("US");
        assert_eq!(
            (us.tax, us.item_tax, us.units),
            (TaxAtTill::Added, ItemTax::Printed, Units::Us)
        );
        assert_eq!(us.words, ["en"]);
        // Canadá: imposto somado, mas métrico.
        assert_eq!(
            (of("CA").tax, of("CA").units),
            (TaxAtTill::Added, Units::Metric)
        );
        // País fora da lista: o genérico.
        let jp = of("JP");
        assert_eq!((jp.tax, jp.item_tax), (TaxAtTill::Included, ItemTax::None));
        assert!(jp.official_series.is_empty() && jp.reader.is_none());
    }
}
