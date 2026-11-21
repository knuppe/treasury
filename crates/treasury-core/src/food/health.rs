//! Saúde da cesta (docs/gtin-inflation.md §7.2), a partir do Open Food Facts e do que a família
//! **comprou** (não do que comeu): parte do gasto em ultraprocessados (NOVA 4), produtos "alto
//! em" pelos limites da Anvisa, proteína por real e alergias e restrições de cada membro.
//!
//! Tudo é indicativo: o Open Food Facts pode ter erros e não serve para uso médico. Produto sem
//! dados fica sem informação (nunca se chuta pela descrição). Alergias e restrições são dado
//! sensível: ficam só no cofre.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::open_data::OpenData;
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::time::{fmt_date, now_utc};

/// Cobertura mínima (parte do gasto em comida com dados) para mostrar os números.
const MIN_COVERAGE: f64 = 0.3;

/// Restrições que a pessoa pode marcar no membro (as mesmas do CHECK da V138).
pub const RESTRICTIONS: &[&str] = &[
    "gluten",
    "milk",
    "eggs",
    "peanuts",
    "nuts",
    "soybeans",
    "fish",
    "crustaceans",
    "sesame",
    "vegan",
    "vegetarian",
];

/// O que o produto tem "alto em", pelos limites da rotulagem frontal da Anvisa (RDC 429/2020,
/// IN 75/2020): por 100 g, açúcar adicionado ≥ 15 g, gordura saturada ≥ 6 g, sódio ≥ 600 mg; por
/// 100 ml, a metade. Só no Brasil (os outros países têm regra própria ou nenhuma).
pub fn high_in(d: &OpenData, country: &str) -> Vec<String> {
    if country != "BR" {
        return vec![];
    }
    let liquid = d.per.as_deref() == Some("100ml");
    let f = if liquid { 0.5 } else { 1.0 };
    let mut out = vec![];
    if d.added_sugars.is_some_and(|v| v >= 15.0 * f) {
        out.push("added_sugar".to_string());
    }
    if d.saturated_fat.is_some_and(|v| v >= 6.0 * f) {
        out.push("saturated_fat".to_string());
    }
    if d.sodium_mg.is_some_and(|v| v >= 600.0 * f) {
        out.push("sodium".to_string());
    }
    out
}

/// A restrição bate com o produto: pelo alérgeno declarado, pelos traços ("pode conter") ou,
/// para vegano e vegetariano, pelos ingredientes de origem animal que aparecem nos alérgenos e
/// nas categorias.
fn conflicts(d: &OpenData, restriction: &str) -> Option<bool> {
    let tag = match restriction {
        "sesame" => "sesame-seeds",
        other => other,
    };
    let animal = |vegan: bool| {
        let mut tags = vec!["fish", "crustaceans", "molluscs"];
        if vegan {
            tags.extend(["milk", "eggs"]);
        }
        let in_allergens = d.allergens.iter().any(|a| tags.contains(&a.as_str()));
        let meat = d
            .categories
            .iter()
            .any(|c| c.contains("meats") || c.contains("fishes") || c.contains("seafood"));
        let labelled = d
            .labels
            .iter()
            .any(|l| l == "vegan" || (!vegan && l == "vegetarian"));
        (in_allergens || meat) && !labelled
    };
    match restriction {
        "vegan" => animal(true).then_some(false),
        "vegetarian" => animal(false).then_some(false),
        _ if d.allergens.iter().any(|a| a == tag) => Some(false),
        _ if d.traces.iter().any(|a| a == tag) => Some(true),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "HealthProduct")]
pub struct HealthProduct {
    pub product_id: String,
    pub name: String,
    /// Gasto no período (centavos).
    #[ts(type = "number")]
    pub spent: i64,
    pub nova: Option<u8>,
    /// `added_sugar`, `saturated_fat`, `sodium`.
    pub high_in: Vec<String>,
    /// Gramas de proteína por 1 unidade da moeda (R$ 1).
    pub protein_per_money: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "HealthAllergenAlert")]
pub struct AllergenAlert {
    pub member_id: String,
    pub member_name: String,
    pub product_id: String,
    pub product_name: String,
    pub restriction: String,
    /// Só "pode conter" (traços).
    pub trace: bool,
    pub last_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "HealthSummary")]
pub struct HealthSummary {
    /// A fonte Open Food Facts está ligada.
    pub source_enabled: bool,
    /// Parte do gasto em comida (últimos 3 meses) com dados do Open Food Facts.
    pub coverage: f64,
    /// Há cobertura suficiente para mostrar as partes do gasto.
    pub enough: bool,
    /// Parte do gasto coberto em ultraprocessados (NOVA 4), agora e um ano antes.
    pub ultra_share: Option<f64>,
    pub ultra_share_year_ago: Option<f64>,
    /// Produtos comprados com "alto em" algum nutriente (Brasil), do mais gasto ao menos.
    pub high_in: Vec<HealthProduct>,
    /// Melhores fontes de proteína por real gasto.
    pub protein: Vec<HealthProduct>,
    pub allergens: Vec<AllergenAlert>,
    /// GTINs ainda sem consulta (para o botão "buscar dados").
    #[ts(type = "number")]
    pub pending: u32,
}

struct Bought {
    product: String,
    name: String,
    spent: f64,
    base_price: Option<i64>,
    base_unit: Option<String>,
    date: String,
}

fn bought(conn: &Connection, from: &str, to: &str) -> Result<Vec<Bought>> {
    let mut stmt = conn.prepare(
        "SELECT i.product_id, p.name, i.total - i.discount, i.base_price, i.base_unit, r.date
           FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
           JOIN product p ON p.id = i.product_id
          WHERE r.date BETWEEN ?1 AND ?2",
    )?;
    let out = stmt
        .query_map(params![from, to], |r| {
            Ok(Bought {
                product: r.get(0)?,
                name: super::clean_description(&r.get::<_, String>(1)?),
                spent: r.get::<_, i64>(2)? as f64,
                base_price: r.get(3)?,
                base_unit: r.get(4)?,
                date: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(out)
}

fn open_data(conn: &Connection) -> Result<HashMap<String, OpenData>> {
    let mut stmt = conn.prepare(
        "SELECT g.product_id, o.gtin, o.found, o.fetched_at, o.name, o.brand, o.quantity,
                o.categories, o.nova, o.nutriscore, o.per, o.energy_kcal, o.sugars,
                o.added_sugars, o.saturated_fat, o.sodium_mg, o.protein, o.fiber, o.allergens,
                o.traces, o.labels, o.page_url, o.source_updated
           FROM product_gtin g JOIN product_open_data o ON o.gtin = g.gtin
          WHERE o.found = 1 ORDER BY g.last_seen",
    )?;
    let mut out = HashMap::new();
    let rows = stmt.query_map([], |r| {
        let id: String = r.get(0)?;
        // Mesma ordem de colunas de `open_data::row`, deslocada de um.
        Ok((
            id,
            OpenData {
                gtin: r.get(1)?,
                found: r.get(2)?,
                fetched_at: r.get(3)?,
                name: r.get(4)?,
                brand: r.get(5)?,
                quantity: r.get(6)?,
                categories: split(r.get(7)?),
                nova: r.get(8)?,
                nutriscore: r.get(9)?,
                per: r.get(10)?,
                energy_kcal: r.get(11)?,
                sugars: r.get(12)?,
                added_sugars: r.get(13)?,
                saturated_fat: r.get(14)?,
                sodium_mg: r.get(15)?,
                protein: r.get(16)?,
                fiber: r.get(17)?,
                allergens: split(r.get(18)?),
                traces: split(r.get(19)?),
                labels: split(r.get(20)?),
                page_url: r.get(21)?,
                source_updated: r.get(22)?,
            },
        ))
    })?;
    for row in rows {
        let (id, d) = row?;
        out.insert(id, d);
    }
    Ok(out)
}

fn split(v: Option<String>) -> Vec<String> {
    v.map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default()
}

fn food_products(conn: &Connection) -> Result<HashSet<String>> {
    Ok(super::products(
        conn,
        &super::ProductsArgs {
            food: Some(true),
            ..Default::default()
        },
    )?
    .into_iter()
    .map(|p| p.id)
    .collect())
}

fn ultra_share(
    buys: &[Bought],
    data: &HashMap<String, OpenData>,
    food: &HashSet<String>,
) -> (f64, Option<f64>) {
    let (mut total, mut covered, mut ultra) = (0.0, 0.0, 0.0);
    for b in buys.iter().filter(|b| food.contains(&b.product)) {
        total += b.spent;
        if let Some(d) = data.get(&b.product).filter(|d| d.nova.is_some()) {
            covered += b.spent;
            if d.nova == Some(4) {
                ultra += b.spent;
            }
        }
    }
    let coverage = if total > 0.0 { covered / total } else { 0.0 };
    (coverage, (covered > 0.0).then(|| ultra / covered))
}

/// A saúde da cesta nos últimos 3 meses até `today`.
pub fn summary(conn: &Connection, today: NaiveDate) -> Result<HealthSummary> {
    let country = crate::country::family_country(conn);
    let data = open_data(conn)?;
    let food = food_products(conn)?;
    let from = today - chrono::Duration::days(91);
    let recent = bought(conn, &fmt_date(from), &fmt_date(today))?;
    let year = bought(
        conn,
        &fmt_date(from - chrono::Duration::days(365)),
        &fmt_date(today - chrono::Duration::days(365)),
    )?;
    let (coverage, ultra) = ultra_share(&recent, &data, &food);
    let (cov_year, ultra_year) = ultra_share(&year, &data, &food);
    let enough = coverage >= MIN_COVERAGE;

    // Por produto, no período.
    let mut per: HashMap<String, (String, f64, Option<i64>, Option<String>)> = HashMap::new();
    for b in recent.iter().filter(|b| food.contains(&b.product)) {
        let e = per
            .entry(b.product.clone())
            .or_insert((b.name.clone(), 0.0, None, None));
        e.1 += b.spent;
        if b.base_price.is_some() {
            e.2 = b.base_price;
            e.3 = b.base_unit.clone();
        }
    }
    let mut products: Vec<HealthProduct> = per
        .into_iter()
        .filter_map(|(id, (name, spent, price, unit))| {
            let d = data.get(&id)?;
            // Proteína por real: g por kg (×10) dividido pelo preço do kg em reais.
            let protein_per_money = match (d.protein, price, unit.as_deref()) {
                (Some(p), Some(c), Some("kg" | "l")) if c > 0 => {
                    Some(p * 10.0 / (c as f64 / 100.0))
                }
                _ => None,
            };
            Some(HealthProduct {
                product_id: id,
                name,
                spent: spent.round() as i64,
                nova: d.nova,
                high_in: high_in(d, &country),
                protein_per_money,
            })
        })
        .collect();
    products.sort_by_key(|p| std::cmp::Reverse(p.spent));
    let high: Vec<HealthProduct> = products
        .iter()
        .filter(|p| !p.high_in.is_empty())
        .take(15)
        .cloned()
        .collect();
    let mut protein: Vec<HealthProduct> = products
        .iter()
        .filter(|p| p.protein_per_money.is_some_and(|g| g > 0.0))
        .cloned()
        .collect();
    protein.sort_by(|a, b| {
        b.protein_per_money
            .unwrap_or(0.0)
            .total_cmp(&a.protein_per_money.unwrap_or(0.0))
    });
    protein.truncate(10);

    // Alergias e restrições de cada membro contra o que foi comprado nos últimos 3 meses.
    let diets = diet_list(conn)?;
    let mut allergens = vec![];
    let mut seen = HashSet::new();
    for diet in &diets {
        for b in recent.iter().rev() {
            let Some(d) = data.get(&b.product) else {
                continue;
            };
            for r in &diet.restrictions {
                if let Some(trace) = conflicts(d, r)
                    && seen.insert((diet.member_id.clone(), b.product.clone(), r.clone()))
                {
                    allergens.push(AllergenAlert {
                        member_id: diet.member_id.clone(),
                        member_name: diet.member_name.clone(),
                        product_id: b.product.clone(),
                        product_name: b.name.clone(),
                        restriction: r.clone(),
                        trace,
                        last_date: b.date.clone(),
                    });
                }
            }
        }
    }
    let pending = super::open_data::pending(conn, today, 10_000)?.len() as u32;
    let _ = cov_year;
    Ok(HealthSummary {
        source_enabled: crate::sources::enabled(conn, crate::sources::SourceId::OpenFoodFacts),
        coverage,
        enough,
        ultra_share: ultra.filter(|_| enough),
        ultra_share_year_ago: ultra_year.filter(|_| enough),
        high_in: if enough { high } else { vec![] },
        protein,
        allergens,
        pending,
    })
}

/// Os dados de saúde de um produto: os do Open Food Facts e o "alto em".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ProductHealth")]
pub struct ProductHealth {
    pub data: OpenData,
    pub high_in: Vec<String>,
}

pub fn of_product(conn: &Connection, product_id: &str) -> Result<Option<ProductHealth>> {
    let country = crate::country::family_country(conn);
    Ok(
        super::open_data::of_product(conn, product_id)?.map(|d| ProductHealth {
            high_in: high_in(&d, &country),
            data: d,
        }),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "MemberDiet")]
pub struct MemberDiet {
    pub member_id: String,
    pub member_name: String,
    pub restrictions: Vec<String>,
}

/// Restrições de todos os membros (só os que têm alguma).
pub fn diet_list(conn: &Connection) -> Result<Vec<MemberDiet>> {
    let mut stmt = conn.prepare(
        "SELECT d.member_id, m.name, d.restriction FROM member_diet d
           JOIN member m ON m.id = d.member_id ORDER BY m.name, d.restriction",
    )?;
    let mut out: Vec<MemberDiet> = vec![];
    for row in stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (id, name, r) = row?;
        match out.last_mut() {
            Some(d) if d.member_id == id => d.restrictions.push(r),
            _ => out.push(MemberDiet {
                member_id: id,
                member_name: name,
                restrictions: vec![r],
            }),
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, rename = "MemberDietInput")]
pub struct DietInput {
    pub member_id: String,
    pub restrictions: Vec<String>,
}

/// Troca as restrições do membro.
pub fn diet_set(conn: &Connection, a: &DietInput) -> Result<Vec<MemberDiet>> {
    if let Some(bad) = a
        .restrictions
        .iter()
        .find(|r| !RESTRICTIONS.contains(&r.as_str()))
    {
        return Err(Error::invalid(crate::i18n::t_args(
            "nucleo-alimentacao-erro-restricao",
            &crate::i18n::args(&[("restricao", bad.clone().into())]),
        )));
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM member WHERE id = ?1)",
        [&a.member_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(Error::not_found(t("nucleo-alimentacao-erro-membro")));
    }
    conn.execute(
        "DELETE FROM member_diet WHERE member_id = ?1",
        [&a.member_id],
    )?;
    let now = now_utc();
    for r in &a.restrictions {
        conn.execute(
            "INSERT OR IGNORE INTO member_diet (member_id, restriction, created_at)
             VALUES (?1, ?2, ?3)",
            params![a.member_id, r, now],
        )?;
    }
    diet_list(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(per: &str) -> OpenData {
        OpenData {
            gtin: "07890000000024".into(),
            found: true,
            per: Some(per.into()),
            added_sugars: Some(10.0),
            saturated_fat: Some(6.5),
            sodium_mg: Some(320.0),
            allergens: vec!["gluten".into(), "milk".into()],
            traces: vec!["soybeans".into()],
            ..OpenData::default()
        }
    }

    #[test]
    fn anvisa_limits_and_restrictions() {
        assert_eq!(high_in(&data("100g"), "BR"), vec!["saturated_fat"]);
        // Bebida: a metade dos limites.
        assert_eq!(
            high_in(&data("100ml"), "BR"),
            vec!["added_sugar", "saturated_fat", "sodium"]
        );
        assert!(high_in(&data("100g"), "US").is_empty());
        let d = data("100g");
        assert_eq!(conflicts(&d, "gluten"), Some(false));
        assert_eq!(conflicts(&d, "soybeans"), Some(true));
        assert_eq!(conflicts(&d, "peanuts"), None);
        assert_eq!(conflicts(&d, "vegan"), Some(false));
        assert_eq!(conflicts(&d, "vegetarian"), None);
    }
}
