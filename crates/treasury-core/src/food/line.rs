//! Linha do produto e reduflação (docs/gtin-inflation.md §4): produto = um tamanho; a linha
//! (marca + nome sem o tamanho) junta os tamanhos do mesmo item ("o mesmo café em 500 g e
//! 250 g") e mostra a reduflação, quando a embalagem encolhe e o preço por kg sobe. Também as
//! sugestões de juntar produtos com nome parecido na mesma categoria (nunca junta sozinho).

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::import::text::{fold, similarity};

static SIZE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:\d+\s*[x×]\s*)?\d+(?:[.,]\d+)?\s*(?:kg|g|gr|grs|mg|ml|l|lt|lts|litros?|un|und|unid|cx|pct|c/?\s*\d+)\b|\bc/\s*\d+\b|\b\d+\s*[x×]\b",
    )
    .unwrap()
});

/// Marca + nome sem o tamanho, para comparação ("Café Exemplo Torrado 500g" → "cafe exemplo
/// torrado").
pub fn line_key(brand: Option<&str>, name: &str) -> String {
    let clean = super::clean_description(name);
    let base = SIZE.replace_all(&clean, " ");
    let key = fold(&format!("{} {base}", brand.unwrap_or("")));
    key.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Preenche a linha dos produtos que ainda não têm (ao abrir o cofre e ao criar o produto).
pub fn fill(conn: &Connection) -> Result<u32> {
    let rows: Vec<(String, Option<String>, String)> = {
        let mut stmt =
            conn.prepare("SELECT id, brand, name FROM product WHERE line_key IS NULL")?;
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?
    };
    for (id, brand, name) in &rows {
        conn.execute(
            "UPDATE product SET line_key = ?2 WHERE id = ?1",
            params![id, line_key(brand.as_deref(), name)],
        )?;
    }
    Ok(rows.len() as u32)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodLineSibling")]
pub struct Sibling {
    pub id: String,
    pub name: String,
    pub pack_qty: Option<String>,
    pub pack_unit: Option<String>,
    /// Último preço por kg, L ou unidade (centavos).
    #[ts(type = "number | null")]
    pub last_price: Option<i64>,
    pub base_unit: Option<String>,
    pub last_date: Option<String>,
}

/// Os outros tamanhos da mesma linha.
pub fn siblings(conn: &Connection, product_id: &str) -> Result<Vec<Sibling>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.pack_qty, p.pack_unit,
                (SELECT i.base_price FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
                  WHERE i.product_id = p.id AND i.base_price IS NOT NULL
                  ORDER BY r.date DESC LIMIT 1),
                (SELECT i.base_unit FROM receipt_item i WHERE i.product_id = p.id
                  AND i.base_unit IS NOT NULL LIMIT 1),
                (SELECT max(r.date) FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
                  WHERE i.product_id = p.id)
           FROM product p
          WHERE p.line_key = (SELECT line_key FROM product WHERE id = ?1)
            AND p.line_key <> '' AND p.id <> ?1
          ORDER BY p.pack_qty",
    )?;
    let out = stmt
        .query_map([product_id], |r| {
            Ok(Sibling {
                id: r.get(0)?,
                name: super::clean_description(&r.get::<_, String>(1)?),
                pack_qty: r.get(2)?,
                pack_unit: r.get(3)?,
                last_price: r.get(4)?,
                base_unit: r.get(5)?,
                last_date: r.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(out)
}

fn qty(s: &Option<String>) -> Option<f64> {
    s.as_deref()?.parse().ok()
}

/// Reduflação: o produto novo é da mesma linha de um maior, com o preço por kg, L ou unidade
/// mais alto. Cria um alerta por linha e episódio.
pub fn check_shrinkflation(conn: &Connection, product_id: &str) -> Result<bool> {
    let me = siblings_with_self(conn, product_id)?;
    let Some(new) = me.iter().find(|s| s.id == product_id) else {
        return Ok(false);
    };
    let (Some(nq), Some(np)) = (qty(&new.pack_qty), new.last_price) else {
        return Ok(false);
    };
    let Some(old) = me.iter().filter(|s| s.id != product_id).find(|s| {
        s.pack_unit == new.pack_unit
            && qty(&s.pack_qty).is_some_and(|q| q > nq)
            && s.last_price.is_some_and(|p| (np as f64) > p as f64 * 1.01)
    }) else {
        return Ok(false);
    };
    let line: String = conn.query_row(
        "SELECT line_key FROM product WHERE id = ?1",
        [product_id],
        |r| r.get(0),
    )?;
    let change = np as f64 / old.last_price.unwrap_or(np) as f64 - 1.0;
    use crate::ai::alerts::{AlertSeverity, Msg, NewAlert, create};
    create(
        conn,
        &NewAlert {
            kind: "shrinkflation",
            severity: AlertSeverity::Info,
            title: Msg::new("alertas-reduflacao-titulo").arg("produto", new.name.as_str()),
            body: vec![
                Msg::new("alertas-reduflacao-corpo")
                    .arg("produto", new.name.as_str())
                    .arg("antes", old.pack_qty.clone().unwrap_or_default().as_str())
                    .arg("agora", new.pack_qty.clone().unwrap_or_default().as_str())
                    .arg(
                        "unidade",
                        new.pack_unit.clone().unwrap_or_default().as_str(),
                    )
                    .arg("variacao", format!("{:+.0}%", change * 100.0).as_str())
                    .into(),
            ],
            source: "food",
            dedupe_key: Some(format!(
                "shrinkflation:{line}:{}",
                new.pack_qty.clone().unwrap_or_default()
            )),
            ref_table: Some("product"),
            ref_id: Some(product_id.to_string()),
            data: serde_json::json!({"product_id": product_id, "from_id": old.id}),
        },
    )?;
    Ok(true)
}

fn siblings_with_self(conn: &Connection, product_id: &str) -> Result<Vec<Sibling>> {
    let mut all = siblings(conn, product_id)?;
    // O próprio produto, com o mesmo formato.
    let me: Vec<Sibling> = {
        let mut stmt = conn.prepare(
            "SELECT p.id, p.name, p.pack_qty, p.pack_unit,
                    (SELECT i.base_price FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
                      WHERE i.product_id = p.id AND i.base_price IS NOT NULL
                      ORDER BY r.date DESC LIMIT 1),
                    NULL, NULL
               FROM product p WHERE p.id = ?1",
        )?;
        stmt.query_map([product_id], |r| {
            Ok(Sibling {
                id: r.get(0)?,
                name: super::clean_description(&r.get::<_, String>(1)?),
                pack_qty: r.get(2)?,
                pack_unit: r.get(3)?,
                last_price: r.get(4)?,
                base_unit: r.get(5)?,
                last_date: r.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    all.extend(me);
    Ok(all)
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FoodMergeSuggestion")]
pub struct MergeSuggestion {
    pub a_id: String,
    pub a_name: String,
    pub b_id: String,
    pub b_name: String,
    /// Semelhança dos nomes (0 a 1).
    pub score: f64,
}

/// Produtos com nome parecido na mesma categoria e do mesmo tamanho, sem GTINs diferentes:
/// talvez sejam o mesmo (a pessoa decide).
pub fn merge_suggestions(conn: &Connection, limit: usize) -> Result<Vec<MergeSuggestion>> {
    type Row = (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    );
    let rows: Vec<Row> = {
        let mut stmt = conn.prepare(
            "SELECT p.id, p.name, p.category_id, p.pack_qty, p.pack_unit,
                    (SELECT group_concat(gtin) FROM product_gtin g WHERE g.product_id = p.id)
               FROM product p",
        )?;
        stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let mut by_cat: HashMap<Option<String>, Vec<usize>> = HashMap::new();
    for (i, r) in rows.iter().enumerate() {
        by_cat.entry(r.2.clone()).or_default().push(i);
    }
    let mut out = vec![];
    for idx in by_cat.values() {
        for (n, &i) in idx.iter().enumerate() {
            for &j in &idx[n + 1..] {
                let (a, b) = (&rows[i], &rows[j]);
                if a.3 != b.3 || a.4 != b.4 || (a.5.is_some() && b.5.is_some()) {
                    continue;
                }
                let (an, bn) = (
                    super::clean_description(&a.1),
                    super::clean_description(&b.1),
                );
                let score = similarity(&an, &bn);
                if score >= 0.75 {
                    out.push(MergeSuggestion {
                        a_id: a.0.clone(),
                        a_name: an,
                        b_id: b.0.clone(),
                        b_name: bn,
                        score,
                    });
                }
            }
        }
    }
    out.sort_by(|x, y| y.score.total_cmp(&x.score));
    out.truncate(limit);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_key_drops_the_size() {
        assert_eq!(
            line_key(None, "Café Exemplo Torrado 500g"),
            "cafe exemplo torrado"
        );
        assert_eq!(
            line_key(None, "CAFE EXEMPLO TORRADO 250G"),
            "cafe exemplo torrado"
        );
        assert_eq!(
            line_key(Some("Marca"), "Leite Integral 1L C/12"),
            "marca leite integral"
        );
        assert_eq!(line_key(None, "Refrigerante 6x2L"), "refrigerante");
    }
}
