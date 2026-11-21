//! Open Food Facts pelo código de barras (docs/gtin-inflation.md §7): nome, marca, quantidade,
//! nutrientes por 100 g ou 100 ml, grupo NOVA, Nutri-Score (só como texto), alérgenos e selos.
//!
//! - Fonte desligada por padrão (`sources::OPT_IN`); ligada em Ajustes › Fontes de dados.
//! - Vai **só o GTIN**, nunca loja, data, preço, cupom ou dado de saúde da família; sem conta.
//! - Cada GTIN é consultado uma vez e guardado no cofre (`product_open_data`); revalida a cada
//!   6 meses, só para produtos comprados no último ano. Consultas espaçadas, em ordem
//!   embaralhada, poucas por vez (o limite de leitura do Open Food Facts é por minuto).
//! - O que vem de fora é sugestão: preenche a marca e a linha vazias, não troca o que a pessoa
//!   editou. Os dados são indicativos e não servem para uso médico (termos do Open Food Facts).

use std::time::Duration;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::gtin;
use crate::api::Api;
use crate::error::Result;
use crate::sources::{self, SourceId};
use crate::time::{fmt_date, now_utc, today_local};

/// Consultas por vez (o resto fica para a próxima).
const BATCH: usize = 40;
/// Intervalo entre consultas.
const GAP: Duration = Duration::from_millis(1500);
/// Revalidar depois de tantos dias.
const STALE_DAYS: i64 = 180;

const FIELDS: &str = "product_name,product_name_pt,product_name_es,product_name_en,brands,quantity,\
categories_tags,nova_group,nutriscore_grade,nutriments,allergens_tags,traces_tags,labels_tags,\
last_modified_t";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ProductOpenData")]
pub struct OpenData {
    pub gtin: String,
    pub found: bool,
    pub fetched_at: String,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub quantity: Option<String>,
    /// Etiquetas de categoria sem o prefixo de idioma ("breakfast-cereals").
    pub categories: Vec<String>,
    /// Grupo NOVA (1 a 4; 4 = ultraprocessado).
    pub nova: Option<u8>,
    /// Nutri-Score segundo o Open Food Facts (texto; o selo tem dono e não é desenhado).
    pub nutriscore: Option<String>,
    /// `100g` ou `100ml`.
    pub per: Option<String>,
    pub energy_kcal: Option<f64>,
    pub sugars: Option<f64>,
    pub added_sugars: Option<f64>,
    pub saturated_fat: Option<f64>,
    pub sodium_mg: Option<f64>,
    pub protein: Option<f64>,
    pub fiber: Option<f64>,
    pub allergens: Vec<String>,
    pub traces: Vec<String>,
    pub labels: Vec<String>,
    /// Página do produto no Open Food Facts (o crédito "ver o produto").
    pub page_url: Option<String>,
    pub source_updated: Option<String>,
}

fn tags(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|t| t.split_once(':').map_or(t, |(_, r)| r).to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn num(v: &Value, key: &str) -> Option<f64> {
    match v.get(key)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.replace(',', ".").parse().ok(),
        _ => None,
    }
    .filter(|x: &f64| x.is_finite() && *x >= 0.0)
}

/// Lê a resposta da API (v2, `product/<código>.json`).
pub fn parse(gtin14: &str, body: &str, fetched_at: &str) -> OpenData {
    let mut d = OpenData {
        gtin: gtin14.to_string(),
        fetched_at: fetched_at.to_string(),
        ..OpenData::default()
    };
    let Ok(v) = serde_json::from_str::<Value>(body) else {
        return d;
    };
    let found = v.get("status").and_then(Value::as_i64) == Some(1);
    let Some(p) = v.get("product").filter(|_| found) else {
        return d;
    };
    d.found = true;
    d.name = [
        "product_name_pt",
        "product_name_es",
        "product_name",
        "product_name_en",
    ]
    .iter()
    .find_map(|k| text(p, k));
    d.brand = text(p, "brands").map(|b| b.split(',').next().unwrap_or(&b).trim().to_string());
    d.quantity = text(p, "quantity");
    d.categories = tags(p, "categories_tags");
    d.nova = p
        .get("nova_group")
        .and_then(|n| n.as_u64().or_else(|| n.as_str()?.parse().ok()))
        .filter(|n| (1..=4).contains(n))
        .map(|n| n as u8);
    d.nutriscore = text(p, "nutriscore_grade")
        .map(|g| g.to_lowercase())
        .filter(|g| matches!(g.as_str(), "a" | "b" | "c" | "d" | "e"));
    let liquid = d.quantity.as_deref().is_some_and(|q| {
        let q = q.to_lowercase();
        q.contains("ml") || q.contains(" l") || q.ends_with('l') || q.contains("litro")
    });
    if let Some(n) = p.get("nutriments") {
        d.energy_kcal = num(n, "energy-kcal_100g");
        d.sugars = num(n, "sugars_100g");
        d.added_sugars = num(n, "added-sugars_100g");
        d.saturated_fat = num(n, "saturated-fat_100g");
        // O Open Food Facts guarda o sódio em gramas.
        d.sodium_mg = num(n, "sodium_100g").map(|g| (g * 1000.0).round());
        d.protein = num(n, "proteins_100g");
        d.fiber = num(n, "fiber_100g");
        let any = [
            d.energy_kcal,
            d.sugars,
            d.saturated_fat,
            d.sodium_mg,
            d.protein,
        ]
        .iter()
        .any(Option::is_some);
        if any {
            d.per = Some(if liquid { "100ml" } else { "100g" }.into());
        }
    }
    d.allergens = tags(p, "allergens_tags");
    d.traces = tags(p, "traces_tags");
    d.labels = tags(p, "labels_tags");
    d.page_url = Some(format!(
        "https://world.openfoodfacts.org/product/{}",
        gtin::display(gtin14)
    ));
    d.source_updated = p
        .get("last_modified_t")
        .and_then(Value::as_i64)
        .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
        .map(|t| fmt_date(t.date_naive()));
    d
}

fn join(v: &[String]) -> Option<String> {
    (!v.is_empty()).then(|| v.join(","))
}

fn split(v: Option<String>) -> Vec<String> {
    v.map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default()
}

/// Grava a resposta e preenche a marca vazia dos produtos com esse GTIN.
pub fn store(conn: &Connection, d: &OpenData) -> Result<()> {
    conn.execute(
        "INSERT INTO product_open_data (gtin, source, found, fetched_at, name, brand, quantity,
             categories, nova, nutriscore, per, energy_kcal, sugars, added_sugars, saturated_fat,
             sodium_mg, protein, fiber, allergens, traces, labels, page_url, source_updated)
         VALUES (?1, 'openfoodfacts', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                 ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)
         ON CONFLICT (gtin) DO UPDATE SET
             found = excluded.found, fetched_at = excluded.fetched_at, name = excluded.name,
             brand = excluded.brand, quantity = excluded.quantity,
             categories = excluded.categories, nova = excluded.nova,
             nutriscore = excluded.nutriscore, per = excluded.per,
             energy_kcal = excluded.energy_kcal, sugars = excluded.sugars,
             added_sugars = excluded.added_sugars, saturated_fat = excluded.saturated_fat,
             sodium_mg = excluded.sodium_mg, protein = excluded.protein, fiber = excluded.fiber,
             allergens = excluded.allergens, traces = excluded.traces, labels = excluded.labels,
             page_url = excluded.page_url, source_updated = excluded.source_updated",
        params![
            d.gtin,
            d.found,
            d.fetched_at,
            d.name,
            d.brand,
            d.quantity,
            join(&d.categories),
            d.nova,
            d.nutriscore,
            d.per,
            d.energy_kcal,
            d.sugars,
            d.added_sugars,
            d.saturated_fat,
            d.sodium_mg,
            d.protein,
            d.fiber,
            join(&d.allergens),
            join(&d.traces),
            join(&d.labels),
            d.page_url,
            d.source_updated,
        ],
    )?;
    if let Some(b) = &d.brand {
        conn.execute(
            "UPDATE product SET brand = ?2
              WHERE brand IS NULL AND id IN (SELECT product_id FROM product_gtin WHERE gtin = ?1)",
            params![d.gtin, b],
        )?;
    }
    // A categoria dos dados abertos vale como sugestão: só entra no produto que a pessoa não
    // classificou e que as palavras da descrição deixaram em Outros (ou sem categoria).
    if let Some(key) = category_key(&d.categories) {
        conn.execute(
            "UPDATE product SET category_id = (SELECT id FROM product_category WHERE seed_key = ?2),
                    updated_at = ?3
              WHERE category_origin = 'auto'
                AND id IN (SELECT product_id FROM product_gtin WHERE gtin = ?1)
                AND (category_id IS NULL
                     OR category_id IN (SELECT id FROM product_category WHERE seed_key = 'outros'))
                AND EXISTS (SELECT 1 FROM product_category WHERE seed_key = ?2)",
            params![d.gtin, key, crate::time::now_utc()],
        )?;
    }
    Ok(())
}

/// Categorias do Open Food Facts → categoria da semente (docs/design/consumption-review.md, D1).
/// A taxonomia dele é a mesma em todos os países (`en:dairies`, `en:breads`…), então um mapa
/// só serve em qualquer idioma. As específicas vêm antes das gerais.
pub(crate) const CATEGORY_MAP: &[(&str, &str)] = &[
    ("en:baby-foods", "bebe"),
    ("en:alcoholic-beverages", "bebidas_alcoolicas"),
    ("en:beers", "bebidas_alcoolicas"),
    ("en:wines", "bebidas_alcoolicas"),
    ("en:frozen-foods", "congelados"),
    ("en:ice-creams-and-sorbets", "congelados"),
    ("en:waters", "bebidas.agua"),
    ("en:sodas", "bebidas.refrigerantes"),
    ("en:juices-and-nectars", "bebidas.sucos"),
    ("en:fruit-juices", "bebidas.sucos"),
    ("en:cheeses", "laticinios.queijos"),
    ("en:yogurts", "laticinios.iogurtes"),
    ("en:milks", "laticinios.leite"),
    ("en:eggs", "carnes.aves_ovos"),
    ("en:dairies", "laticinios"),
    ("en:fishes", "carnes.peixes"),
    ("en:seafood", "carnes.peixes"),
    ("en:poultries", "carnes.aves_ovos"),
    ("en:prepared-meats", "carnes.frios"),
    ("en:meats", "carnes"),
    ("en:breads", "padaria.paes"),
    ("en:cakes", "padaria.bolos"),
    ("en:biscuits", "doces.biscoitos"),
    ("en:chocolates", "doces.doces"),
    ("en:candies", "doces.doces"),
    ("en:confectioneries", "doces.doces"),
    ("en:salty-snacks", "doces.salgadinhos"),
    ("en:snacks", "doces_lanches"),
    ("en:fruits", "hortifruti.frutas"),
    ("en:vegetables", "hortifruti.legumes"),
    ("en:pastas", "mercearia.cereais"),
    ("en:rices", "mercearia.cereais"),
    ("en:legumes", "mercearia.cereais"),
    ("en:breakfast-cereals", "mercearia.cereais"),
    ("en:flours", "mercearia.farinhas"),
    ("en:vegetable-oils", "mercearia.oleos"),
    ("en:fats", "mercearia.oleos"),
    ("en:sugars", "mercearia.acucar_cafe"),
    ("en:coffees", "mercearia.acucar_cafe"),
    ("en:teas", "mercearia.acucar_cafe"),
    ("en:spices", "mercearia.temperos"),
    ("en:sauces", "mercearia.temperos"),
    ("en:condiments", "mercearia.temperos"),
    ("en:canned-foods", "mercearia.conservas"),
    ("en:beverages", "bebidas"),
    ("en:groceries", "mercearia"),
];

/// A chave da categoria da semente para as categorias do Open Food Facts de um produto.
pub fn category_key(tags: &[String]) -> Option<&'static str> {
    CATEGORY_MAP
        .iter()
        // As categorias ficam guardadas sem o prefixo do idioma ("dairies").
        .find(|(tag, _)| {
            let bare = tag.trim_start_matches("en:");
            tags.iter().any(|t| t.trim_start_matches("en:") == bare)
        })
        .map(|(_, key)| *key)
}

/// Os dados abertos do produto (pelo GTIN mais recente que tenha resposta).
pub fn of_product(conn: &Connection, product_id: &str) -> Result<Option<OpenData>> {
    Ok(conn
        .query_row(
            "SELECT o.gtin, o.found, o.fetched_at, o.name, o.brand, o.quantity, o.categories,
                    o.nova, o.nutriscore, o.per, o.energy_kcal, o.sugars, o.added_sugars,
                    o.saturated_fat, o.sodium_mg, o.protein, o.fiber, o.allergens, o.traces,
                    o.labels, o.page_url, o.source_updated
               FROM product_gtin g JOIN product_open_data o ON o.gtin = g.gtin
              WHERE g.product_id = ?1 AND o.found = 1
              ORDER BY g.last_seen DESC LIMIT 1",
            [product_id],
            row,
        )
        .optional()?)
}

pub(super) fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<OpenData> {
    Ok(OpenData {
        gtin: r.get(0)?,
        found: r.get(1)?,
        fetched_at: r.get(2)?,
        name: r.get(3)?,
        brand: r.get(4)?,
        quantity: r.get(5)?,
        categories: split(r.get(6)?),
        nova: r.get(7)?,
        nutriscore: r.get(8)?,
        per: r.get(9)?,
        energy_kcal: r.get(10)?,
        sugars: r.get(11)?,
        added_sugars: r.get(12)?,
        saturated_fat: r.get(13)?,
        sodium_mg: r.get(14)?,
        protein: r.get(15)?,
        fiber: r.get(16)?,
        allergens: split(r.get(17)?),
        traces: split(r.get(18)?),
        labels: split(r.get(19)?),
        page_url: r.get(20)?,
        source_updated: r.get(21)?,
    })
}

/// GTINs para consultar: comprados no último ano, nunca consultados ou consultados há mais de
/// 6 meses, em ordem embaralhada.
pub fn pending(conn: &Connection, today: NaiveDate, limit: usize) -> Result<Vec<String>> {
    let year_ago = fmt_date(today - chrono::Duration::days(365));
    let stale = (today - chrono::Duration::days(STALE_DAYS)).to_string();
    let mut stmt = conn.prepare(
        "SELECT g.gtin FROM product_gtin g
           LEFT JOIN product_open_data o ON o.gtin = g.gtin
          WHERE (g.last_seen IS NULL OR g.last_seen >= ?1)
            AND (o.gtin IS NULL OR substr(o.fetched_at, 1, 10) < ?2)
          ORDER BY random() LIMIT ?3",
    )?;
    let out = stmt
        .query_map(params![year_ago, stale, limit as i64], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(out)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "OpenDataSync")]
pub struct SyncReport {
    #[ts(type = "number")]
    pub fetched: u32,
    #[ts(type = "number")]
    pub found: u32,
    #[ts(type = "number")]
    pub failed: u32,
    /// Ainda faltam (ficam para a próxima vez).
    #[ts(type = "number")]
    pub remaining: u32,
}

/// Busca os GTINs pendentes, fora da trava do banco (a rede leva tempo).
pub fn sync(api: &Api) -> Result<SyncReport> {
    let today = today_local();
    let gtins = api.with_session(|s, _| {
        let conn = s.db.conn();
        sources::ensure_enabled(conn, SourceId::OpenFoodFacts)?;
        pending(conn, today, BATCH + 1)
    })?;
    let more = gtins.len() > BATCH;
    let agent = crate::indices::sync::agent();
    let mut rep = SyncReport::default();
    for (i, g) in gtins.iter().take(BATCH).enumerate() {
        if i > 0 {
            std::thread::sleep(GAP);
        }
        let url = format!(
            "https://world.openfoodfacts.org/api/v2/product/{}.json?fields={FIELDS}",
            gtin::display(g)
        );
        match crate::indices::sync::get_text(&agent, &url) {
            Ok(body) => {
                let d = parse(g, &body, &now_utc());
                rep.fetched += 1;
                rep.found += d.found as u32;
                api.with_session(|s, _| store(s.db.conn(), &d))?;
            }
            // 404 é "produto não está na base": guarda como não achado.
            Err(e) if e.contains("404") => {
                let d = parse(g, "{\"status\":0}", &now_utc());
                rep.fetched += 1;
                api.with_session(|s, _| store(s.db.conn(), &d))?;
            }
            Err(_) => rep.failed += 1,
        }
    }
    let result = if rep.failed > 0 && rep.fetched == 0 {
        Err(crate::i18n::t("nucleo-alimentacao-consultas-falharam"))
    } else {
        Ok(())
    };
    api.with_session(|s, _| {
        sources::record_result(s.db.conn(), SourceId::OpenFoodFacts, &result, today)?;
        rep.remaining = if more {
            pending(s.db.conn(), today, 10_000)?.len() as u32
        } else {
            0
        };
        Ok(())
    })?;
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Made-up response in the API v2 format.
    const BODY: &str = r#"{"status":1,"code":"7890000000024","product":{
        "product_name_pt":"Biscoito Exemplo Recheado","brands":"Marca Exemplo, Outra",
        "quantity":"130 g","categories_tags":["en:snacks","en:biscuits"],"nova_group":4,
        "nutriscore_grade":"E","allergens_tags":["en:gluten","en:milk"],"traces_tags":["en:soybeans"],
        "labels_tags":[],"last_modified_t":1790000000,
        "nutriments":{"energy-kcal_100g":480,"sugars_100g":32,"added-sugars_100g":30,
          "saturated-fat_100g":9.5,"sodium_100g":0.32,"proteins_100g":6,"fiber_100g":"2,1"}}}"#;

    #[test]
    fn parses_the_api_answer() {
        let d = parse("07890000000024", BODY, "2026-10-22T10:00:00.000Z");
        assert!(d.found);
        assert_eq!(d.name.as_deref(), Some("Biscoito Exemplo Recheado"));
        assert_eq!(d.brand.as_deref(), Some("Marca Exemplo"));
        assert_eq!(d.nova, Some(4));
        assert_eq!(d.nutriscore.as_deref(), Some("e"));
        assert_eq!(d.per.as_deref(), Some("100g"));
        assert_eq!(d.sodium_mg, Some(320.0));
        assert_eq!(d.fiber, Some(2.1));
        assert_eq!(d.allergens, vec!["gluten", "milk"]);
        assert_eq!(d.categories, vec!["snacks", "biscuits"]);
        assert_eq!(
            d.page_url.as_deref(),
            Some("https://world.openfoodfacts.org/product/7890000000024")
        );
        let none = parse("07890000000024", r#"{"status":0}"#, "x");
        assert!(!none.found && none.name.is_none());
    }
}
