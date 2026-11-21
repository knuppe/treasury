//! O classificador único dos itens (docs/design/consumption-review.md, A2): a categoria de produto
//! de um item e de onde ela veio. A mesma função serve à prévia do cupom, à compra digitada e à
//! gravação, para a pessoa ver antes de gravar o que o app vai escolher.
//!
//! Ordem: o código de barras já ligado a um produto; o apelido da loja (a mesma descrição
//! naquela loja); um produto com o mesmo nome; as palavras da descrição (`receipts::classify`,
//! que dá o grupo, e as subcategorias da semente). Sem nada disso, o item fica em Outros.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{auto_category, fold, gtin, store_key};
use crate::error::Result;
use crate::ids::Id;
use crate::receipts::classify::{self as words, ItemGroup};

/// De onde veio a categoria prevista.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "FoodGuessReason")]
pub enum Reason {
    /// Produto já comprado (código de barras, apelido da loja ou o mesmo nome).
    Known,
    /// Categoria dos dados abertos do código de barras.
    OpenData,
    /// Palavras da descrição.
    Words,
    /// Nada casou: fica em Outros até a pessoa escolher.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodGuess")]
pub struct Guess {
    pub category_id: Option<Id>,
    pub reason: Reason,
    /// O produto já comprado, quando a categoria veio dele.
    pub product_id: Option<Id>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export, rename = "FoodGuessArgs")]
pub struct GuessArgs {
    pub description: String,
    #[serde(default)]
    #[ts(optional)]
    pub store_name: Option<String>,
    /// Identificação fiscal da loja (CNPJ), quando o cupom traz.
    #[serde(default)]
    #[ts(optional)]
    pub store_tax_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub gtin: Option<String>,
}

fn category_of(conn: &Connection, product: &str) -> Result<Option<Id>> {
    Ok(conn
        .query_row(
            "SELECT category_id FROM product WHERE id = ?1",
            [product],
            |r| r.get(0),
        )
        .optional()?
        .flatten())
}

/// A categoria prevista de um item.
pub fn guess(conn: &Connection, a: &GuessArgs) -> Result<Guess> {
    let known = |product: Id| -> Result<Guess> {
        Ok(Guess {
            category_id: category_of(conn, &product)?,
            reason: Reason::Known,
            product_id: Some(product),
        })
    };
    if let Some(g) = a.gtin.as_deref().and_then(gtin::gtin14)
        && let Some(p) = conn
            .query_row(
                "SELECT product_id FROM product_gtin WHERE gtin = ?1",
                [&g],
                |r| r.get::<_, Id>(0),
            )
            .optional()?
    {
        return known(p);
    }
    // Código de barras novo, mas com os dados abertos já no cofre: a categoria deles.
    if let Some(g) = a.gtin.as_deref().and_then(gtin::gtin14)
        && let Some(tags) = conn
            .query_row(
                "SELECT categories FROM product_open_data WHERE gtin = ?1 AND found",
                [&g],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten()
        && let Some(key) = super::open_data::category_key(
            &tags
                .split(',')
                .map(|t| t.trim().to_string())
                .collect::<Vec<_>>(),
        )
        && let Some(id) = conn
            .query_row(
                "SELECT id FROM product_category WHERE seed_key = ?1",
                [key],
                |r| r.get::<_, Id>(0),
            )
            .optional()?
    {
        return Ok(Guess {
            category_id: Some(id),
            reason: Reason::OpenData,
            product_id: None,
        });
    }
    let desc = fold(&a.description);
    if desc.is_empty() {
        return Ok(Guess {
            category_id: None,
            reason: Reason::None,
            product_id: None,
        });
    }
    let store = store_key(a.store_tax_id.as_deref(), a.store_name.as_deref());
    // O apelido da loja primeiro; depois o mesmo nome em qualquer loja.
    let by_alias: Option<Id> = conn
        .query_row(
            "SELECT product_id FROM product_alias WHERE description = ?1
              ORDER BY (store_key = ?2) DESC LIMIT 1",
            params![desc, store],
            |r| r.get(0),
        )
        .optional()?;
    let by_name: Option<Id> = match by_alias {
        Some(p) => Some(p),
        None => conn
            .query_row(
                "SELECT id FROM product WHERE lower(name) = lower(?1) LIMIT 1",
                [a.description.trim()],
                |r| r.get(0),
            )
            .optional()?,
    };
    if let Some(p) = by_name {
        return known(p);
    }
    let (group, _) = words::classify_in(&a.description, &super::profile::family(conn).words);
    let category_id = auto_category(conn, group, &a.description)?;
    // "Outros" sem subcategoria é o mesmo que não ter achado nada.
    let found = group != ItemGroup::Outros
        || category_id.as_ref().is_some_and(|c| {
            conn.query_row(
                "SELECT seed_key FROM product_category WHERE id = ?1",
                [c],
                |r| r.get::<_, Option<String>>(0),
            )
            .ok()
            .flatten()
            .is_some_and(|k| k != super::seed::root_key(ItemGroup::Outros))
        });
    Ok(Guess {
        category_id,
        reason: if found { Reason::Words } else { Reason::None },
        product_id: None,
    })
}
