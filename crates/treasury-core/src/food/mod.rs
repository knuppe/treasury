//! Consumo (docs/consumption.md; a fase 1 foi a Alimentação, docs/consumption.md): os itens das
//! compras (cupons fiscais NFC-e, docs/receipts.md, e compras cadastradas à mão, [`purchase`])
//! viram **produtos** (o mesmo item em compras diferentes, pelo GTIN, pelo código e pela
//! descrição da loja), com o preço por kg, L ou unidade ao longo do tempo, numa árvore de
//! **categorias de produto** editável com classe (essencial, complementar, supérfluo), cujas
//! raízes são os segmentos (Alimentação, Vestuário, Eletrônicos…). O resumo da tela soma os itens
//! das compras que têm lançamento (o lançamento é o dinheiro, a compra é o detalhe) e o comer
//! fora pelas categorias de lançamento. Vale em qualquer país; a leitura da NFC-e é do Brasil.

pub mod api;
pub mod basket;
pub mod classify;
pub mod gtin;
pub mod health;
pub mod line;
pub mod open_data;
pub mod pack;
pub mod profile;
pub mod purchase;
pub mod seed;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Datelike, Months, NaiveDate};
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::import::text::fold;
use crate::receipts::classify::ItemGroup;
use crate::time::{fmt_date, now_utc};

/// Classe do produto ou da categoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "FoodClass")]
pub enum FoodClass {
    Essential,
    Complementary,
    Superfluous,
}

impl FoodClass {
    pub fn as_str(self) -> &'static str {
        match self {
            FoodClass::Essential => "essential",
            FoodClass::Complementary => "complementary",
            FoodClass::Superfluous => "superfluous",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "essential" => FoodClass::Essential,
            "complementary" => FoodClass::Complementary,
            "superfluous" => FoodClass::Superfluous,
            _ => return None,
        })
    }
    pub fn label(self) -> String {
        t(match self {
            FoodClass::Essential => "nucleo-alimentacao-classe-essencial",
            FoodClass::Complementary => "nucleo-alimentacao-classe-complementar",
            FoodClass::Superfluous => "nucleo-alimentacao-classe-superfluo",
        })
    }
}

// ---------------------------------------------------------------------------
// Categorias de produto

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ProductCategory")]
pub struct Category {
    pub id: Id,
    pub parent_id: Option<Id>,
    pub seed_key: Option<String>,
    pub name: String,
    /// "Hortifrúti › Frutas".
    pub path: String,
    pub depth: u32,
    pub is_food: bool,
    /// A classe da própria categoria (`null`: herda a do pai).
    pub class: Option<FoodClass>,
    /// A que vale (a própria ou a herdada).
    pub effective_class: Option<FoodClass>,
    #[ts(type = "number")]
    pub products: u32,
}

/// Linha de `product_category`: id, pai, chave da semente, nome, alimento, classe, produtos.
type CatRow = (
    String,
    Option<String>,
    Option<String>,
    String,
    bool,
    Option<String>,
    u32,
);

/// Todas as categorias, em árvore (pai antes dos filhos, na ordem).
pub fn categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.parent_id, c.seed_key, c.name, c.is_food, c.class,
                (SELECT count(*) FROM product p WHERE p.category_id = c.id)
           FROM product_category c ORDER BY c.sort, c.name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, bool>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, u32>(6)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let by_id: HashMap<&str, usize> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| (r.0.as_str(), i))
        .collect();
    let mut children: BTreeMap<Option<String>, Vec<usize>> = BTreeMap::new();
    for (i, r) in rows.iter().enumerate() {
        let parent = r.1.clone().filter(|p| by_id.contains_key(p.as_str()));
        children.entry(parent).or_default().push(i);
    }
    let mut out = vec![];
    fn walk(
        rows: &[CatRow],
        children: &BTreeMap<Option<String>, Vec<usize>>,
        parent: Option<String>,
        prefix: &str,
        inherited: Option<FoodClass>,
        depth: u32,
        out: &mut Vec<Category>,
    ) {
        for &i in children.get(&parent).map(Vec::as_slice).unwrap_or(&[]) {
            let r = &rows[i];
            let class = r.5.as_deref().and_then(FoodClass::parse);
            let effective = class.or(inherited);
            let path = if prefix.is_empty() {
                r.3.clone()
            } else {
                format!("{prefix} › {}", r.3)
            };
            out.push(Category {
                id: r.0.clone(),
                parent_id: r.1.clone(),
                seed_key: r.2.clone(),
                name: r.3.clone(),
                path: path.clone(),
                depth,
                is_food: r.4,
                class,
                effective_class: effective,
                products: r.6,
            });
            walk(
                rows,
                children,
                Some(r.0.clone()),
                &path,
                effective,
                depth + 1,
                out,
            );
        }
    }
    walk(&rows, &children, None, "", None, 0, &mut out);
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "ProductCategoryInput")]
pub struct CategoryInput {
    #[serde(default)]
    #[ts(optional)]
    pub parent_id: Option<Id>,
    pub name: String,
    /// Padrão: o do pai (ou sim).
    #[serde(default)]
    #[ts(optional)]
    pub is_food: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub class: Option<FoodClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "ProductCategoryPatch")]
pub struct CategoryPatch {
    pub id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub parent_id: Option<Option<Id>>,
    #[serde(default)]
    #[ts(optional)]
    pub is_food: Option<bool>,
    /// `null`: volta a herdar a do pai.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(
        optional,
        type = "\"essential\" | \"complementary\" | \"superfluous\" | null"
    )]
    pub class: Option<Option<FoodClass>>,
}

fn clean_name(name: &str) -> Result<String> {
    let n = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if n.is_empty() {
        return Err(Error::invalid(t("nucleo-alimentacao-erro-nome")));
    }
    Ok(n.chars().take(80).collect())
}

fn category_exists(conn: &Connection, id: &str) -> Result<(bool, i64)> {
    conn.query_row(
        "SELECT is_food, sort FROM product_category WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()?
    .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-categoria")))
}

pub fn create_category(tx: &Transaction<'_>, i: &CategoryInput) -> Result<Id> {
    let name = clean_name(&i.name)?;
    let parent_food = match &i.parent_id {
        Some(p) => Some(category_exists(tx, p)?.0),
        None => None,
    };
    let sort: i64 = tx.query_row(
        "SELECT COALESCE(MAX(sort) + 1, 0) FROM product_category WHERE parent_id IS ?1",
        [&i.parent_id],
        |r| r.get(0),
    )?;
    let id = new_id();
    tx.execute(
        "INSERT INTO product_category (id, parent_id, name, is_food, class, sort, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            i.parent_id,
            name,
            i.is_food.or(parent_food).unwrap_or(true),
            i.class.map(FoodClass::as_str),
            sort,
            now_utc()
        ],
    )?;
    Ok(id)
}

/// Ids da subárvore de uma categoria (ela inclusa).
fn subtree(conn: &Connection, id: &str) -> Result<HashSet<String>> {
    let all = categories(conn)?;
    let mut set: HashSet<String> = [id.to_string()].into();
    let mut grew = true;
    while grew {
        grew = false;
        for c in &all {
            if c.parent_id.as_ref().is_some_and(|p| set.contains(p)) && set.insert(c.id.clone()) {
                grew = true;
            }
        }
    }
    Ok(set)
}

pub fn update_category(tx: &Transaction<'_>, p: &CategoryPatch) -> Result<()> {
    category_exists(tx, &p.id)?;
    if let Some(n) = &p.name {
        tx.execute(
            "UPDATE product_category SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, clean_name(n)?, now_utc()],
        )?;
    }
    if let Some(parent) = &p.parent_id {
        if let Some(pid) = parent {
            category_exists(tx, pid)?;
            if subtree(tx, &p.id)?.contains(pid) {
                return Err(Error::invalid(t("nucleo-alimentacao-erro-ciclo")));
            }
        }
        tx.execute(
            "UPDATE product_category SET parent_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, parent, now_utc()],
        )?;
    }
    if let Some(f) = p.is_food {
        tx.execute(
            "UPDATE product_category SET is_food = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, f, now_utc()],
        )?;
    }
    if let Some(c) = &p.class {
        tx.execute(
            "UPDATE product_category SET class = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, c.map(FoodClass::as_str), now_utc()],
        )?;
    }
    Ok(())
}

/// Apaga a categoria e as filhas; os produtos delas vão para a categoria-pai.
pub fn delete_category(tx: &Transaction<'_>, id: &str) -> Result<()> {
    category_exists(tx, id)?;
    let parent: Option<String> = tx.query_row(
        "SELECT parent_id FROM product_category WHERE id = ?1",
        [id],
        |r| r.get(0),
    )?;
    for c in subtree(tx, id)? {
        tx.execute(
            "UPDATE product SET category_id = ?2, updated_at = ?3 WHERE category_id = ?1",
            params![c, parent, now_utc()],
        )?;
    }
    tx.execute("DELETE FROM product_category WHERE id = ?1", [id])?;
    Ok(())
}

/// Categoria automática de um item: a subcategoria da semente cujas palavras casam, ou a
/// categoria do grupo do cupom.
pub(crate) fn auto_category(
    conn: &Connection,
    group: ItemGroup,
    description: &str,
) -> Result<Option<Id>> {
    let d = fold(description);
    let root = seed::root_key(group);
    let matches = |s: &&seed::Sub| Regex::new(s.pattern).is_ok_and(|r| r.is_match(&d));
    let key = seed::SUBS
        .iter()
        .filter(|s| s.parent == root)
        .find(matches)
        // Sem grupo do mercado (roupa, eletrônico…): as subcategorias dos segmentos.
        .or_else(|| {
            matches!(group, ItemGroup::Outros | ItemGroup::Casa)
                .then(|| {
                    seed::SUBS
                        .iter()
                        .filter(|s| s.parent.starts_with("seg."))
                        .find(matches)
                })
                .flatten()
        })
        .map(|s| s.key.to_string())
        .unwrap_or(root);
    Ok(conn
        .query_row(
            "SELECT id FROM product_category WHERE seed_key = ?1",
            [&key],
            |r| r.get(0),
        )
        .optional()?
        .or(conn
            .query_row(
                "SELECT id FROM product_category WHERE seed_key = ?1",
                [seed::root_key(group)],
                |r| r.get(0),
            )
            .optional()?))
}

// ---------------------------------------------------------------------------
// Produtos

/// Chave da loja: o CNPJ (só dígitos) ou o nome normalizado.
pub(crate) fn store_key(cnpj: Option<&str>, name: Option<&str>) -> String {
    let digits: String = cnpj
        .unwrap_or("")
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    if !digits.is_empty() {
        digits
    } else {
        fold(name.unwrap_or("?"))
    }
}

static QTY_TAIL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)\s+(?:\d+[.,]\d{1,3}\s*(?:un|und|unid|kg|g|l|lt|ml|pc|pç|cx|ea|lb)\b|\d+\s*(?:un|und|unid|pc|pç|cx)\s*[x×*]).*$",
    )
    .unwrap()
});
static PRICE_TAIL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?:\s+[+\-]*\s*[$€£]?\s*\d{1,6}[.,]\d{2}\s*[A-Z]?)+\s*$").unwrap()
});

/// Tira da descrição do cupom o que não é nome: o número do item e o código no começo
/// ("001 7891234567895 ARROZ…", "51354 CAMISETA…") e a quantidade, a unidade e os valores no
/// fim ("… 1,00 UN X 24,90 24,90", "… 1,00 Un +-53,48"). O peso da embalagem fica ("5KG").
pub(crate) fn clean_description(description: &str) -> String {
    let words: Vec<&str> = description.split_whitespace().collect();
    let has_letters = |w: &&str| w.chars().any(char::is_alphabetic);
    let first_text = words.iter().position(has_letters).unwrap_or(0);
    let mut s = words[first_text..].join(" ");
    s = QTY_TAIL.replace(&s, "").into_owned();
    s = PRICE_TAIL.replace(&s, "").into_owned();
    let s = s.trim().trim_end_matches(['-', '+', '*']).trim();
    if s.chars().any(char::is_alphabetic) {
        s.to_string()
    } else {
        description.trim().to_string()
    }
}

/// Nome do produto pela descrição do cupom: sem código, quantidade e preço, espaços limpos, a
/// primeira letra de cada palavra maiúscula ("ARROZ TIPO1 5KG" → "Arroz Tipo1 5kg").
fn product_name(description: &str) -> String {
    clean_description(description)
        .split_whitespace()
        .map(|w| {
            let lower = w.to_lowercase();
            let mut c = lower.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

struct ItemRow {
    id: String,
    code: Option<String>,
    description: String,
    quantity: Option<String>,
    unit: Option<String>,
    total: i64,
    group: ItemGroup,
    gtin: Option<String>,
    store_cnpj: Option<String>,
    store_name: Option<String>,
    date: String,
    /// Origem do cupom (`xml`, `manual`…): diz como o GTIN chegou.
    origin: String,
}

/// O produto do item e se ele acabou de nascer.
fn find_or_create_product(tx: &Connection, it: &ItemRow, gtin: Option<&str>) -> Result<(Id, bool)> {
    let store = store_key(it.store_cnpj.as_deref(), it.store_name.as_deref());
    let desc = fold(&it.description);
    let code = it.code.clone().unwrap_or_default();
    // 1. GTIN-14 normalizado (docs/gtin-inflation.md §3.5); 2. apelido da loja.
    let by_gtin: Option<String> = match gtin {
        Some(g) => tx
            .query_row(
                "SELECT product_id FROM product_gtin WHERE gtin = ?1",
                [g],
                |r| r.get(0),
            )
            .optional()?,
        None => None,
    };
    let found = match by_gtin {
        Some(id) => Some(id),
        None => tx
            .query_row(
                "SELECT product_id FROM product_alias
                  WHERE store_key = ?1 AND description = ?2
                  ORDER BY (code = ?3) DESC LIMIT 1",
                params![store, desc, code],
                |r| r.get(0),
            )
            .optional()?,
    };
    let now = now_utc();
    let created = found.is_none();
    let id = match found {
        Some(id) => id,
        None => {
            let id = new_id();
            let (pack_qty, pack_unit) = match pack::pack_of(&it.description) {
                Some((q, u)) => (Some(pack::qty_text(q)), Some(u)),
                None => (None, None),
            };
            tx.execute(
                "INSERT INTO product (id, name, pack_qty, pack_unit, category_id,
                                      category_origin, line_key, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'auto', ?7, ?6, ?6)",
                params![
                    id,
                    product_name(&it.description),
                    pack_qty,
                    pack_unit,
                    auto_category(tx, it.group, &it.description)?,
                    now,
                    line::line_key(None, &it.description),
                ],
            )?;
            id
        }
    };
    tx.execute(
        "INSERT INTO product_alias (id, product_id, store_key, code, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT (store_key, code, description) DO NOTHING",
        params![new_id(), id, store, code, desc, now],
    )?;
    // O GTIN fica ligado ao produto: o novo, ou o achado pelo apelido quando a loja passou a
    // mandar o código de barras.
    if let Some(g) = gtin {
        tx.execute(
            "INSERT INTO product_gtin (gtin, product_id, origin, first_seen, last_seen, created_at)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5)
             ON CONFLICT (gtin) DO UPDATE SET
                first_seen = min(COALESCE(first_seen, excluded.first_seen), excluded.first_seen),
                last_seen = max(COALESCE(last_seen, excluded.last_seen), excluded.last_seen)",
            params![
                g,
                id,
                match it.origin.as_str() {
                    "xml" => "xml",
                    "manual" => "manual",
                    _ => "store_code",
                },
                it.date,
                now
            ],
        )?;
    }
    Ok((id, created))
}

/// Liga os itens sem produto (de um cupom, ou de todos) e grava a quantidade e o preço por kg,
/// L ou unidade. Devolve quantos itens ligou.
pub fn link_items(tx: &Connection, receipt_id: Option<&str>) -> Result<u32> {
    seed::ensure(tx)?;
    let mut stmt = tx.prepare(
        "SELECT i.id, i.code, i.description, i.quantity, i.unit, i.total, i.item_group, i.gtin,
                r.store_cnpj, r.store_name, r.date, r.origin
           FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
          WHERE i.product_id IS NULL AND (?1 IS NULL OR i.receipt_id = ?1)
          ORDER BY r.date, i.line",
    )?;
    let rows: Vec<ItemRow> = stmt
        .query_map([receipt_id], |r| {
            Ok(ItemRow {
                id: r.get(0)?,
                code: r.get(1)?,
                description: r.get(2)?,
                quantity: r.get(3)?,
                unit: r.get(4)?,
                total: r.get(5)?,
                group: ItemGroup::parse(&r.get::<_, String>(6)?),
                gtin: r.get(7)?,
                store_cnpj: r.get(8)?,
                store_name: r.get(9)?,
                date: r.get(10)?,
                origin: r.get(11)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);
    let mut n = 0;
    for it in &rows {
        let gtin = it
            .gtin
            .as_deref()
            .and_then(gtin::gtin14)
            .or_else(|| it.code.as_deref().and_then(gtin::gtin14));
        let (product, created) = find_or_create_product(tx, it, gtin.as_deref())?;
        let base = pack::base_of(&it.description, it.quantity.as_deref(), it.unit.as_deref());
        tx.execute(
            "UPDATE receipt_item SET product_id = ?2, gtin = ?3, base_qty = ?4, base_unit = ?5,
                    base_price = ?6
              WHERE id = ?1",
            params![
                it.id,
                product,
                gtin,
                base.map(|b| pack::qty_text(b.0)),
                base.map(|b| b.1),
                base.and_then(|b| pack::base_price(it.total, b.0)),
            ],
        )?;
        // Produto novo da mesma linha de um maior, mais caro por kg: reduflação.
        if created {
            line::check_shrinkflation(tx, &product)?;
        }
        n += 1;
    }
    Ok(n)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodProduct")]
pub struct Product {
    pub id: Id,
    pub name: String,
    pub gtin: Option<String>,
    /// Todos os códigos de barras do produto (EAN-13 etc.), o mais recente primeiro.
    #[serde(default)]
    pub gtins: Vec<String>,
    pub pack_qty: Option<String>,
    pub pack_unit: Option<String>,
    pub category_id: Option<Id>,
    pub category_path: Option<String>,
    /// A classe do próprio produto (`null`: a da categoria).
    pub class: Option<FoodClass>,
    pub effective_class: Option<FoodClass>,
    pub is_food: bool,
    #[ts(type = "number")]
    pub purchases: u32,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub stores: u32,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
    /// Unidade do preço (`kg`, `l`, `un`).
    pub base_unit: Option<String>,
    /// Preço por kg, L ou unidade na primeira e na última compra (centavos).
    #[ts(type = "number | null")]
    pub first_price: Option<i64>,
    #[ts(type = "number | null")]
    pub last_price: Option<i64>,
    /// Variação do preço por kg, L ou unidade da primeira para a última compra (fração).
    pub change: Option<f64>,
    /// Preço unitário e unidade da última compra (para sugerir na compra manual).
    #[ts(type = "number | null")]
    pub last_unit_price: Option<i64>,
    pub last_unit: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodProductsArgs")]
pub struct ProductsArgs {
    /// `true`: só alimentos; `false`: só casa e higiene; ausente: todos.
    #[serde(default)]
    #[ts(optional)]
    pub food: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub query: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub category_id: Option<Id>,
}

pub fn products(conn: &Connection, a: &ProductsArgs) -> Result<Vec<Product>> {
    let cats: HashMap<String, Category> = categories(conn)?
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();
    let within: Option<HashSet<String>> = match &a.category_id {
        Some(id) => Some(subtree(conn, id)?),
        None => None,
    };
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name,
                (SELECT group_concat(gtin) FROM (SELECT g.gtin FROM product_gtin g
                  WHERE g.product_id = p.id ORDER BY g.last_seen DESC, g.created_at DESC)),
                p.pack_qty, p.pack_unit, p.category_id, p.class,
                count(i.id), COALESCE(SUM(i.total), 0),
                count(DISTINCT COALESCE(r.store_cnpj, r.store_name)), MIN(r.date), MAX(r.date)
           FROM product p
           LEFT JOIN receipt_item i ON i.product_id = p.id
           LEFT JOIN receipt r ON r.id = i.receipt_id
          GROUP BY p.id ORDER BY SUM(i.total) DESC, p.name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, u32>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, u32>(9)?,
                r.get::<_, Option<String>>(10)?,
                r.get::<_, Option<String>>(11)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let q = a.query.as_deref().map(fold).filter(|q| !q.is_empty());
    let mut out = vec![];
    for r in rows {
        let cat = r.5.as_ref().and_then(|c| cats.get(c));
        let is_food = cat.is_none_or(|c| c.is_food);
        if a.food.is_some_and(|f| f != is_food) {
            continue;
        }
        if within
            .as_ref()
            .is_some_and(|w| r.5.as_ref().is_none_or(|c| !w.contains(c)))
        {
            continue;
        }
        if q.as_ref().is_some_and(|q| !fold(&r.1).contains(q.as_str())) {
            continue;
        }
        let class = r.6.as_deref().and_then(FoodClass::parse);
        let (first, last, unit) = price_ends(conn, &r.0)?;
        let (last_unit_price, last_unit) = conn
            .query_row(
                "SELECT i.unit_price, i.unit FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
                  WHERE i.product_id = ?1 ORDER BY r.date DESC, i.line DESC LIMIT 1",
                [&r.0],
                |x| Ok((x.get(0)?, x.get(1)?)),
            )
            .optional()?
            .unwrap_or((None, None));
        out.push(Product {
            change: match (first, last) {
                (Some(f), Some(l)) if f > 0 => Some(l as f64 / f as f64 - 1.0),
                _ => None,
            },
            id: r.0,
            // Produtos gravados antes da limpeza ainda trazem código e preço no nome.
            name: clean_description(&r.1),
            gtin: r
                .2
                .as_deref()
                .and_then(|g| g.split(',').next())
                .map(gtin::display),
            gtins: r
                .2
                .as_deref()
                .map(|g| g.split(',').map(gtin::display).collect())
                .unwrap_or_default(),
            pack_qty: r.3,
            pack_unit: r.4,
            category_id: r.5.clone(),
            category_path: cat.map(|c| c.path.clone()),
            class,
            effective_class: class.or(cat.and_then(|c| c.effective_class)),
            is_food,
            purchases: r.7,
            spent: r.8,
            stores: r.9,
            first_date: r.10,
            last_date: r.11,
            base_unit: unit,
            first_price: first,
            last_price: last,
            last_unit_price,
            last_unit,
        });
    }
    Ok(out)
}

/// Preço por kg, L ou unidade na primeira e na última compra do produto.
fn price_ends(
    conn: &Connection,
    product: &str,
) -> Result<(Option<i64>, Option<i64>, Option<String>)> {
    let q = |order: &str| -> Result<Option<(i64, String)>> {
        Ok(conn
            .query_row(
                &format!(
                    "SELECT i.base_price, i.base_unit FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
                      WHERE i.product_id = ?1 AND i.base_price IS NOT NULL
                      ORDER BY r.date {order}, i.line {order} LIMIT 1"
                ),
                [product],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    };
    let first = q("ASC")?;
    let last = q("DESC")?;
    Ok((
        first.as_ref().map(|f| f.0),
        last.as_ref().map(|l| l.0),
        last.map(|l| l.1),
    ))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodPricePoint")]
pub struct PricePoint {
    pub date: String,
    pub store_name: Option<String>,
    pub description: String,
    pub quantity: Option<String>,
    pub unit: Option<String>,
    #[ts(type = "number")]
    pub total: i64,
    pub base_qty: Option<String>,
    pub base_unit: Option<String>,
    #[ts(type = "number | null")]
    pub base_price: Option<i64>,
    pub receipt_id: Id,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodProductHistory")]
pub struct ProductHistory {
    pub product: Product,
    pub points: Vec<PricePoint>,
    /// Os outros tamanhos da mesma linha (docs/gtin-inflation.md §4).
    #[serde(default)]
    pub same_line: Vec<line::Sibling>,
    /// Nutrientes, NOVA e alérgenos do Open Food Facts, quando a fonte trouxe.
    #[serde(default)]
    pub health: Option<health::ProductHealth>,
}

pub fn product_history(conn: &Connection, id: &str) -> Result<ProductHistory> {
    let product = products(conn, &ProductsArgs::default())?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-produto")))?;
    let mut stmt = conn.prepare(
        "SELECT r.date, r.store_name, i.description, i.quantity, i.unit, i.total, i.base_qty,
                i.base_unit, i.base_price, r.id
           FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
          WHERE i.product_id = ?1 ORDER BY r.date, i.line",
    )?;
    let points = stmt
        .query_map([id], |r| {
            Ok(PricePoint {
                date: r.get(0)?,
                store_name: r.get(1)?,
                description: r.get(2)?,
                quantity: r.get(3)?,
                unit: r.get(4)?,
                total: r.get(5)?,
                base_qty: r.get(6)?,
                base_unit: r.get(7)?,
                base_price: r.get(8)?,
                receipt_id: r.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ProductHistory {
        same_line: line::siblings(conn, id)?,
        health: health::of_product(conn, id)?,
        product,
        points,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodProductPatch")]
pub struct ProductPatch {
    pub id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub category_id: Option<Option<Id>>,
    /// `null`: volta a usar a classe da categoria.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(
        optional,
        type = "\"essential\" | \"complementary\" | \"superfluous\" | null"
    )]
    pub class: Option<Option<FoodClass>>,
}

fn product_exists(conn: &Connection, id: &str) -> Result<()> {
    conn.query_row("SELECT 1 FROM product WHERE id = ?1", [id], |_| Ok(()))
        .optional()?
        .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-produto")))
}

pub fn update_product(tx: &Transaction<'_>, p: &ProductPatch) -> Result<()> {
    product_exists(tx, &p.id)?;
    let now = now_utc();
    if let Some(n) = &p.name {
        tx.execute(
            "UPDATE product SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, clean_name(n)?, now],
        )?;
    }
    if let Some(c) = &p.category_id {
        if let Some(c) = c {
            category_exists(tx, c)?;
        }
        tx.execute(
            "UPDATE product SET category_id = ?2, category_origin = 'manual', updated_at = ?3
              WHERE id = ?1",
            params![p.id, c, now],
        )?;
    }
    if let Some(c) = &p.class {
        tx.execute(
            "UPDATE product SET class = ?2, updated_at = ?3 WHERE id = ?1",
            params![p.id, c.map(FoodClass::as_str), now],
        )?;
    }
    Ok(())
}

/// Junta `from` em `into`: os itens e os apelidos passam para `into`, e `from` sai.
/// Liga um código de barras digitado (ou lido da foto) ao produto. Código de outro produto não
/// muda de dono sozinho: a pessoa junta os dois, se forem o mesmo.
pub fn add_gtin(tx: &Connection, product_id: &str, code: &str) -> Result<Product> {
    product_exists(tx, product_id)?;
    let g = gtin::check(code)?;
    let owner: Option<(String, String)> = tx
        .query_row(
            "SELECT p.id, p.name FROM product_gtin g JOIN product p ON p.id = g.product_id
              WHERE g.gtin = ?1",
            [&g],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match owner {
        Some((id, _)) if id == product_id => {}
        Some((_, name)) => {
            return Err(Error::conflict(crate::i18n::t_args(
                "nucleo-alimentacao-gtin-erro-outro",
                &crate::i18n::args(&[("produto", clean_description(&name).into())]),
            )));
        }
        None => {
            tx.execute(
                "INSERT INTO product_gtin (gtin, product_id, origin, created_at)
                 VALUES (?1, ?2, 'manual', ?3)",
                params![g, product_id, now_utc()],
            )?;
        }
    }
    products(tx, &ProductsArgs::default())?
        .into_iter()
        .find(|p| p.id == product_id)
        .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-produto")))
}

/// Tira o código de barras do produto (o código errado, por exemplo).
pub fn remove_gtin(tx: &Connection, product_id: &str, code: &str) -> Result<()> {
    let g = gtin::gtin14(code).unwrap_or_else(|| format!("{:0>14}", code.trim()));
    tx.execute(
        "DELETE FROM product_gtin WHERE gtin = ?1 AND product_id = ?2",
        params![g, product_id],
    )?;
    Ok(())
}

pub fn merge_products(tx: &Transaction<'_>, from: &str, into: &str) -> Result<()> {
    if from == into {
        return Err(Error::invalid(t("nucleo-alimentacao-erro-mesmo-produto")));
    }
    product_exists(tx, from)?;
    product_exists(tx, into)?;
    tx.execute(
        "UPDATE receipt_item SET product_id = ?2 WHERE product_id = ?1",
        params![from, into],
    )?;
    tx.execute(
        "UPDATE OR IGNORE product_alias SET product_id = ?2 WHERE product_id = ?1",
        params![from, into],
    )?;
    // Os GTINs do que sai passam todos para o que fica.
    tx.execute(
        "UPDATE product_gtin SET product_id = ?2 WHERE product_id = ?1",
        params![from, into],
    )?;
    tx.execute("DELETE FROM product WHERE id = ?1", [from])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Pagamentos do cupom

fn purchase_currency(conn: &Connection) -> String {
    crate::household::base_currency(conn).unwrap_or_else(|_| "BRL".into())
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "ReceiptPaymentInput")]
pub struct PaymentInput {
    pub txn_id: Id,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "ReceiptPaymentsArgs")]
pub struct PaymentsArgs {
    pub receipt_id: Id,
    /// Os lançamentos que pagaram o cupom (vazio: solta).
    pub payments: Vec<PaymentInput>,
}

/// Troca os pagamentos do cupom (cartão + vale, por exemplo). O primeiro vira o `txn_id`.
pub fn set_payments(tx: &Transaction<'_>, a: &PaymentsArgs) -> Result<()> {
    let paid: i64 = tx
        .query_row(
            "SELECT paid FROM receipt WHERE id = ?1",
            [&a.receipt_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-cupom")))?;
    let mut seen = HashSet::new();
    for p in &a.payments {
        if p.amount <= 0 || !seen.insert(p.txn_id.clone()) {
            return Err(Error::invalid(t("nucleo-alimentacao-erro-pagamento")));
        }
        tx.query_row("SELECT 1 FROM txn WHERE id = ?1", [&p.txn_id], |_| Ok(()))
            .optional()?
            .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-pagamento")))?;
    }
    let sum: i64 = a.payments.iter().map(|p| p.amount).sum();
    if !a.payments.is_empty() && sum != paid {
        return Err(Error::invalid(t_args(
            "nucleo-alimentacao-erro-soma-pagamentos",
            &args(&[
                (
                    "soma",
                    crate::i18n::money(crate::i18n::current(), sum, &purchase_currency(tx)).into(),
                ),
                (
                    "pago",
                    crate::i18n::money(crate::i18n::current(), paid, &purchase_currency(tx)).into(),
                ),
            ]),
        )));
    }
    tx.execute(
        "DELETE FROM receipt_payment WHERE receipt_id = ?1",
        [&a.receipt_id],
    )?;
    let now = now_utc();
    for p in &a.payments {
        tx.execute(
            "INSERT INTO receipt_payment (id, receipt_id, txn_id, amount, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![new_id(), a.receipt_id, p.txn_id, p.amount, now],
        )?;
    }
    tx.execute(
        "UPDATE receipt SET txn_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![a.receipt_id, a.payments.first().map(|p| &p.txn_id), now],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Resumo

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodSummaryArgs")]
pub struct SummaryArgs {
    /// `AAAA-MM` (padrão: o mês corrente).
    #[serde(default)]
    #[ts(optional)]
    pub month: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Detalha um segmento (raiz da árvore): as classes, as categorias e os produtos dele.
    #[serde(default)]
    #[ts(optional)]
    pub segment_id: Option<Id>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodClassSpend")]
pub struct ClassSpend {
    /// `null`: sem classe.
    pub class: Option<FoodClass>,
    pub label: String,
    #[ts(type = "number")]
    pub spent: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodCategorySpend")]
pub struct CategorySpend {
    pub category_id: Option<Id>,
    pub name: String,
    pub class: Option<FoodClass>,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub items: u32,
}

/// Gasto de um segmento no mês e no anterior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodSegmentSpend")]
pub struct SegmentSpend {
    /// `null`: itens sem categoria.
    pub category_id: Option<Id>,
    pub seed_key: Option<String>,
    pub name: String,
    pub is_food: bool,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub items: u32,
    #[ts(type = "number")]
    pub previous: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodProductSpend")]
pub struct ProductSpend {
    pub product_id: Id,
    pub name: String,
    pub category_path: Option<String>,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub items: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodMonth")]
pub struct MonthSpend {
    pub month: String,
    /// Comida em casa (itens das compras + mercado sem cupom).
    #[ts(type = "number")]
    pub home: i64,
    /// Comida fora (restaurantes e delivery).
    #[ts(type = "number")]
    pub out: i64,
    /// O que não é comida (itens das compras).
    #[ts(type = "number")]
    pub non_food: i64,
    /// Itens das compras por segmento.
    pub segments: Vec<purchase::SegmentShare>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodSummary")]
pub struct Summary {
    pub month: String,
    pub segment_id: Option<Id>,
    /// Itens das compras com lançamento no mês, já com o desconto rateado (todos os segmentos).
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub previous_total: i64,
    /// Os segmentos com gasto no mês ou no anterior (maior primeiro).
    pub segments: Vec<SegmentSpend>,
    /// Comida: itens de comida das compras com lançamento.
    #[ts(type = "number")]
    pub receipt_items: i64,
    /// Mercado e padaria sem cupom (pelas categorias de lançamento).
    #[ts(type = "number")]
    pub market_without_receipt: i64,
    #[ts(type = "number")]
    pub home: i64,
    #[ts(type = "number")]
    pub out: i64,
    /// Restaurantes, delivery… (categoria → valor).
    pub out_categories: Vec<CategorySpend>,
    /// Pessoas da família (ativas) e o gasto de comida por pessoa (em casa + fora).
    #[ts(type = "number")]
    pub people: u32,
    #[ts(type = "number")]
    pub per_person: i64,
    /// No segmento escolhido (ou em tudo).
    pub classes: Vec<ClassSpend>,
    /// As categorias do segmento escolhido (ou os segmentos).
    pub categories: Vec<CategorySpend>,
    /// Os produtos que mais pesaram no mês (no segmento escolhido ou em tudo).
    pub top_products: Vec<ProductSpend>,
    /// Comprados no mês, os que mais subiram de preço por kg, L ou unidade desde a primeira compra.
    pub risers: Vec<Product>,
    /// Os seis meses até este.
    pub months: Vec<MonthSpend>,
    /// Compras do mês sem lançamento (não entram no gasto).
    #[ts(type = "number")]
    pub receipts_without_txn: u32,
    #[ts(type = "number")]
    pub receipts_without_txn_amount: i64,
    #[ts(type = "number")]
    pub receipts: u32,
    /// Itens ainda sem produto (cupons de antes da tela Alimentação).
    #[ts(type = "number")]
    pub unlinked_items: u32,
}

fn month_range(month: &str) -> Result<(NaiveDate, NaiveDate)> {
    let (y, m) = crate::time::parse_month(month)?;
    let from = NaiveDate::from_ymd_opt(y, m, 1).expect("data");
    let to = from
        .checked_add_months(Months::new(1))
        .and_then(|d| d.pred_opt())
        .expect("data");
    Ok((from, to))
}

fn month_key(d: NaiveDate) -> String {
    format!("{:04}-{:02}", d.year(), d.month())
}

/// Categorias de lançamento da semente e as filhas delas.
fn txn_categories(conn: &Connection, seeds: &[&str]) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE tree(id, name) AS (
            SELECT id, name FROM category WHERE seed_key IN (SELECT value FROM json_each(?1))
            UNION SELECT c.id, c.name FROM category c JOIN tree t ON c.parent_id = t.id)
         SELECT id, name FROM tree",
    )?;
    let seeds = serde_json::to_string(seeds).unwrap_or_default();
    let rows = stmt
        .query_map([seeds], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Para o lembrete de Para resolver (docs/to-resolve.md): as compras de mercado do período
/// sem itens, a mais recente primeiro. Só para quem usa Consumo (cadastrou alguma compra nos
/// últimos 90 dias); para os demais, nada.
pub(crate) fn market_without_items(
    conn: &Connection,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<Id>> {
    let uses: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM receipt WHERE created_at >= date(?1, '-90 days'))",
        [fmt_date(to)],
        |r| r.get(0),
    )?;
    if !uses {
        return Ok(vec![]);
    }
    let cats = txn_categories(conn, MARKET_SEEDS)?;
    let ids =
        serde_json::to_string(&cats.iter().map(|c| &c.0).collect::<Vec<_>>()).unwrap_or_default();
    let mut stmt = conn.prepare(
        "SELECT t.id FROM txn t
          WHERE t.kind = 'expense' AND t.status IN ('cleared', 'reconciled')
            AND t.date BETWEEN ?1 AND ?2
            AND t.category_id IN (SELECT value FROM json_each(?3))
            AND NOT EXISTS (SELECT 1 FROM receipt_payment p WHERE p.txn_id = t.id)
            AND NOT EXISTS (SELECT 1 FROM receipt r WHERE r.txn_id = t.id)
          ORDER BY t.date DESC, t.id",
    )?;
    let rows = stmt
        .query_map(params![fmt_date(from), fmt_date(to), ids], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<Id>>>()?;
    Ok(rows)
}

/// A parte sem detalhe dos lançamentos destas categorias que têm compra ligada: o valor do
/// lançamento menos o que as compras dele somam.
fn undetailed(
    conn: &Connection,
    cats: &[(String, String)],
    from: NaiveDate,
    to: NaiveDate,
    member: Option<&str>,
) -> Result<i64> {
    if cats.is_empty() {
        return Ok(0);
    }
    let ids =
        serde_json::to_string(&cats.iter().map(|c| &c.0).collect::<Vec<_>>()).unwrap_or_default();
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(MAX(0, -t.amount - (
                    SELECT COALESCE(SUM(p.amount), 0) FROM receipt_payment p
                     WHERE p.txn_id = t.id))), 0)
           FROM txn t
          WHERE t.kind = 'expense' AND t.status IN ('cleared', 'reconciled')
            AND t.date BETWEEN ?1 AND ?2
            AND t.category_id IN (SELECT value FROM json_each(?3))
            AND (?4 IS NULL OR t.member_id = ?4)
            AND EXISTS (SELECT 1 FROM receipt_payment p WHERE p.txn_id = t.id)",
        params![fmt_date(from), fmt_date(to), ids, member],
        |r| r.get(0),
    )?)
}

/// Gasto dos lançamentos destas categorias no período (só despesas realizadas), por categoria.
fn txn_spend(
    conn: &Connection,
    cats: &[(String, String)],
    from: NaiveDate,
    to: NaiveDate,
    member: Option<&str>,
    without_receipt: bool,
) -> Result<BTreeMap<String, i64>> {
    let mut out = BTreeMap::new();
    if cats.is_empty() {
        return Ok(out);
    }
    let ids =
        serde_json::to_string(&cats.iter().map(|c| &c.0).collect::<Vec<_>>()).unwrap_or_default();
    let mut stmt = conn.prepare(&format!(
        "SELECT COALESCE(s.category_id, t.category_id), -SUM(COALESCE(s.amount, t.amount))
           FROM txn t LEFT JOIN txn_split s ON s.txn_id = t.id
          WHERE t.kind = 'expense' AND t.status IN ('cleared', 'reconciled')
            AND t.date BETWEEN ?1 AND ?2
            AND COALESCE(s.category_id, t.category_id) IN (SELECT value FROM json_each(?3))
            AND (?4 IS NULL OR {member})
            {receipt}
          GROUP BY 1",
        member = crate::reports::engine::MEMBER_EXPR.to_string() + " = ?4",
        receipt = if without_receipt {
            "AND NOT EXISTS (SELECT 1 FROM receipt_payment p WHERE p.txn_id = t.id)
             AND NOT EXISTS (SELECT 1 FROM receipt rr WHERE rr.txn_id = t.id)"
        } else {
            ""
        }
    ))?;
    for r in stmt.query_map(params![fmt_date(from), fmt_date(to), ids, member], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })? {
        let (c, v) = r?;
        *out.entry(c).or_default() += v;
    }
    Ok(out)
}

/// A raiz (o segmento) de uma categoria.
pub(crate) fn root_of(cats: &HashMap<String, Category>, id: &str) -> String {
    let mut cur = cats.get(id);
    while let Some(c) = cur {
        match c.parent_id.as_ref().and_then(|p| cats.get(p)) {
            Some(p) => cur = Some(p),
            None => return c.id.clone(),
        }
    }
    id.to_string()
}

/// A categoria do caminho logo abaixo do segmento (ou a própria, se já for filha dele).
fn child_of(cats: &HashMap<String, Category>, segment: &str, id: &str) -> String {
    let mut cur = id.to_string();
    while let Some(c) = cats.get(&cur) {
        match &c.parent_id {
            Some(p) if p != segment && cats.contains_key(p) => cur = p.clone(),
            _ => break,
        }
    }
    cur
}

struct SpendItem {
    product_id: Option<String>,
    category_id: Option<String>,
    /// O segmento (raiz da categoria).
    segment: Option<String>,
    class: Option<FoodClass>,
    is_food: bool,
    net: i64,
}

/// Itens das compras com lançamento no período, com o desconto rateado pelo valor.
fn receipt_items(
    conn: &Connection,
    cats: &HashMap<String, Category>,
    from: NaiveDate,
    to: NaiveDate,
    member: Option<&str>,
) -> Result<Vec<SpendItem>> {
    let mut stmt = conn.prepare(
        "SELECT i.total, r.discount, r.items_total, p.category_id, p.class, i.item_group, p.id
           FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
           LEFT JOIN product p ON p.id = i.product_id
          WHERE r.date BETWEEN ?1 AND ?2 AND (?3 IS NULL OR r.member_id = ?3)
            AND (r.txn_id IS NOT NULL OR EXISTS (SELECT 1 FROM receipt_payment x WHERE x.receipt_id = r.id))",
    )?;
    let rows = stmt
        .query_map(params![fmt_date(from), fmt_date(to), member], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .map(
            |(total, discount, items_total, cat, class, group, product)| {
                let c = cat.as_ref().and_then(|c| cats.get(c));
                let share = if items_total > 0 {
                    (discount as f64 * total as f64 / items_total as f64).round() as i64
                } else {
                    0
                };
                let group = ItemGroup::parse(&group);
                SpendItem {
                    product_id: product,
                    segment: cat.as_deref().map(|c| root_of(cats, c)),
                    category_id: cat.clone(),
                    class: class
                        .as_deref()
                        .and_then(FoodClass::parse)
                        .or(c.and_then(|c| c.effective_class)),
                    is_food: c.map_or_else(
                        || {
                            seed::ROOTS
                                .iter()
                                .find(|r| r.group == group)
                                .is_some_and(|r| r.is_food)
                        },
                        |c| c.is_food,
                    ),
                    net: total - share,
                }
            },
        )
        .collect())
}

const MARKET_SEEDS: &[&str] = &["categorias-mercado", "categorias-padaria"];
const OUT_SEEDS: &[&str] = &["categorias-restaurantes", "categorias-delivery"];

/// Gasto por segmento (id do segmento → valor, itens).
fn by_segment(items: &[SpendItem]) -> BTreeMap<Option<String>, (i64, u32)> {
    let mut out: BTreeMap<Option<String>, (i64, u32)> = BTreeMap::new();
    for i in items {
        let e = out.entry(i.segment.clone()).or_default();
        e.0 += i.net;
        e.1 += 1;
    }
    out
}

fn segment_name(cats: &HashMap<String, Category>, id: Option<&String>) -> String {
    id.and_then(|i| cats.get(i))
        .map_or_else(|| t("nucleo-alimentacao-sem-categoria"), |c| c.name.clone())
}

fn month_totals(
    conn: &Connection,
    cats: &HashMap<String, Category>,
    month: &str,
    member: Option<&str>,
) -> Result<MonthSpend> {
    let (from, to) = month_range(month)?;
    let items = receipt_items(conn, cats, from, to, member)?;
    let market = txn_spend(
        conn,
        &txn_categories(conn, MARKET_SEEDS)?,
        from,
        to,
        member,
        true,
    )?;
    let out = txn_spend(
        conn,
        &txn_categories(conn, OUT_SEEDS)?,
        from,
        to,
        member,
        false,
    )?;
    let mut segments: Vec<purchase::SegmentShare> = by_segment(&items)
        .into_iter()
        .map(|(id, (spent, _))| purchase::SegmentShare {
            name: segment_name(cats, id.as_ref()),
            category_id: id,
            spent,
        })
        .collect();
    segments.sort_by_key(|s| -s.spent);
    Ok(MonthSpend {
        month: month.to_string(),
        home: items
            .iter()
            .filter(|i| i.is_food)
            .map(|i| i.net)
            .sum::<i64>()
            + market.values().sum::<i64>(),
        out: out.values().sum(),
        non_food: items.iter().filter(|i| !i.is_food).map(|i| i.net).sum(),
        segments,
    })
}

pub fn summary(conn: &Connection, a: &SummaryArgs, today: NaiveDate) -> Result<Summary> {
    let month = a.month.clone().unwrap_or_else(|| month_key(today));
    let (from, to) = month_range(&month)?;
    let prev = from.checked_sub_months(Months::new(1)).expect("data");
    let member = a.member_id.as_deref();
    let cats: HashMap<String, Category> = categories(conn)?
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();
    if let Some(s) = &a.segment_id
        && !cats.contains_key(s)
    {
        return Err(Error::not_found(t("nucleo-alimentacao-erro-categoria")));
    }
    let all = receipt_items(conn, &cats, from, to, member)?;
    let before = receipt_items(conn, &cats, prev, from.pred_opt().expect("data"), member)?;

    // Segmentos, com o mês anterior.
    let now_by = by_segment(&all);
    let prev_by = by_segment(&before);
    let mut keys: Vec<&Option<String>> = now_by.keys().chain(prev_by.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut segments: Vec<SegmentSpend> = keys
        .into_iter()
        .map(|id| {
            let c = id.as_ref().and_then(|i| cats.get(i));
            let (spent, items) = now_by.get(id).copied().unwrap_or_default();
            SegmentSpend {
                category_id: id.clone(),
                seed_key: c.and_then(|c| c.seed_key.clone()),
                name: segment_name(&cats, id.as_ref()),
                is_food: c.is_some_and(|c| c.is_food),
                spent,
                items,
                previous: prev_by.get(id).map_or(0, |v| v.0),
            }
        })
        .collect();
    segments.sort_by_key(|s| (-s.spent, -s.previous));

    // Comida: em casa (itens + mercado sem cupom) e fora.
    let food_items: i64 = all.iter().filter(|i| i.is_food).map(|i| i.net).sum();
    let market_cats = txn_categories(conn, MARKET_SEEDS)?;
    let out_cats = txn_categories(conn, OUT_SEEDS)?;
    // Mercado sem cupom, mais a parte sem detalhe dos lançamentos de mercado que têm compra
    // ligada (a compra pode detalhar só alguns itens).
    let market: i64 = txn_spend(conn, &market_cats, from, to, member, true)?
        .values()
        .sum::<i64>()
        + undetailed(conn, &market_cats, from, to, member)?;
    let out_map = txn_spend(conn, &out_cats, from, to, member, false)?;
    let names: HashMap<&String, &String> = out_cats.iter().map(|(i, n)| (i, n)).collect();
    let out_categories: Vec<CategorySpend> = out_map
        .iter()
        .map(|(id, v)| CategorySpend {
            category_id: Some(id.clone()),
            name: names.get(id).map(|n| n.to_string()).unwrap_or_default(),
            class: None,
            spent: *v,
            items: 0,
        })
        .collect();
    let out: i64 = out_map.values().sum();

    // O detalhe: o segmento escolhido ou tudo.
    let items: Vec<&SpendItem> = all
        .iter()
        .filter(|i| a.segment_id.is_none() || i.segment == a.segment_id)
        .collect();
    let mut classes: BTreeMap<Option<FoodClass>, i64> = BTreeMap::new();
    let mut by_cat: BTreeMap<Option<String>, (i64, u32)> = BTreeMap::new();
    let mut by_product: HashMap<String, (i64, u32)> = HashMap::new();
    for i in &items {
        *classes.entry(i.class).or_default() += i.net;
        let key = match &a.segment_id {
            Some(s) => i.category_id.as_deref().map(|c| child_of(&cats, s, c)),
            None => i.segment.clone(),
        };
        let e = by_cat.entry(key).or_default();
        e.0 += i.net;
        e.1 += 1;
        if let Some(p) = &i.product_id {
            let e = by_product.entry(p.clone()).or_default();
            e.0 += i.net;
            e.1 += 1;
        }
    }
    let mut categories: Vec<CategorySpend> = by_cat
        .into_iter()
        .map(|(id, (spent, n))| {
            let c = id.as_ref().and_then(|i| cats.get(i));
            CategorySpend {
                name: c.map_or_else(|| t("nucleo-alimentacao-sem-categoria"), |c| c.name.clone()),
                class: c.and_then(|c| c.effective_class),
                category_id: id,
                spent,
                items: n,
            }
        })
        .collect();
    categories.sort_by_key(|c| -c.spent);
    let classes: Vec<ClassSpend> = classes
        .into_iter()
        .map(|(c, v)| ClassSpend {
            label: c.map_or_else(|| t("nucleo-alimentacao-classe-sem"), FoodClass::label),
            class: c,
            spent: v,
        })
        .collect();
    let catalog = products(
        conn,
        &ProductsArgs {
            category_id: a.segment_id.clone(),
            ..ProductsArgs::default()
        },
    )?;
    let mut top_products: Vec<ProductSpend> = catalog
        .iter()
        .filter_map(|p| {
            let (spent, n) = by_product.get(&p.id)?;
            Some(ProductSpend {
                product_id: p.id.clone(),
                name: p.name.clone(),
                category_path: p.category_path.clone(),
                spent: *spent,
                items: *n,
            })
        })
        .collect();
    top_products.sort_by_key(|p| -p.spent);
    top_products.truncate(8);
    let mut risers: Vec<Product> = catalog
        .into_iter()
        .filter(|p| {
            by_product.contains_key(&p.id) && p.purchases >= 2 && p.change.is_some_and(|c| c > 0.0)
        })
        .collect();
    risers.sort_by(|x, y| y.change.unwrap_or(0.0).total_cmp(&x.change.unwrap_or(0.0)));
    risers.truncate(5);

    let mut months = vec![];
    for k in (0..6).rev() {
        let m = from.checked_sub_months(Months::new(k)).expect("data");
        months.push(month_totals(conn, &cats, &month_key(m), member)?);
    }
    let people: u32 = if member.is_some() {
        1
    } else {
        conn.query_row("SELECT count(*) FROM member WHERE archived = 0", [], |r| {
            r.get(0)
        })?
    };
    let (rcount, without, without_amount): (u32, u32, i64) = conn.query_row(
        "SELECT count(*),
                SUM(CASE WHEN r.txn_id IS NULL AND NOT EXISTS (SELECT 1 FROM receipt_payment x WHERE x.receipt_id = r.id) THEN 1 ELSE 0 END),
                COALESCE(SUM(CASE WHEN r.txn_id IS NULL AND NOT EXISTS (SELECT 1 FROM receipt_payment x WHERE x.receipt_id = r.id) THEN r.paid ELSE 0 END), 0)
           FROM receipt r WHERE r.date BETWEEN ?1 AND ?2 AND (?3 IS NULL OR r.member_id = ?3)",
        params![fmt_date(from), fmt_date(to), member],
        |r| Ok((r.get(0)?, r.get::<_, Option<u32>>(1)?.unwrap_or(0), r.get(2)?)),
    )?;
    let unlinked: u32 = conn.query_row(
        "SELECT count(*) FROM receipt_item WHERE product_id IS NULL",
        [],
        |r| r.get(0),
    )?;
    let home = food_items + market;
    Ok(Summary {
        month,
        segment_id: a.segment_id.clone(),
        total: all.iter().map(|i| i.net).sum(),
        previous_total: before.iter().map(|i| i.net).sum(),
        segments,
        receipt_items: food_items,
        market_without_receipt: market,
        home,
        out,
        out_categories,
        people,
        per_person: if people > 0 {
            (home + out) / people as i64
        } else {
            0
        },
        classes,
        categories,
        top_products,
        risers,
        months,
        receipts_without_txn: without,
        receipts_without_txn_amount: without_amount,
        receipts: rcount,
        unlinked_items: unlinked,
    })
}

/// Ao abrir o cofre: as categorias-semente (e os segmentos da V136) e os produtos dos itens de
/// cupons de antes da V135, fora do histórico (como a chave das categorias,
/// `changes::BACKFILLED_COLUMNS`).
pub fn on_open(conn: &Connection) -> Result<u32> {
    let tx = conn.unchecked_transaction()?;
    seed::ensure(&tx)?;
    let n = link_items(&tx, None)?;
    line::fill(&tx)?;
    tx.commit()?;
    Ok(n)
}
