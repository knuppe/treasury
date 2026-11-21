//! Índice da sua cesta (docs/gtin-inflation.md §5): o mesmo produto, na mesma loja, mês a mês,
//! pelo preço por kg, L ou unidade líquido de desconto, ao lado do índice oficial de alimentos
//! (no Brasil, o IPCA "Alimentação no domicílio").
//!
//! - **Preço do mês** por produto e loja: a mediana das compras do mês. Trocar de mercado não
//!   conta como inflação.
//! - **Variação**: contra o último mês com compra, até 3 meses antes.
//! - **Agregação**: dentro da categoria, média geométrica (Jevons); entre categorias, pesos pelo
//!   gasto da família no período (Laspeyres encadeado mês a mês).
//! - **Cobertura**: o mês só entra com pelo menos 40% do gasto em produtos que têm par; senão o
//!   índice fica igual e a tela diz que houve poucas compras repetidas.
//! - Fica de fora: compras em outra moeda, saltos de mais de 5× (quase sempre tamanho mal lido).

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::time::parse_date;

/// Gasto mínimo coberto por produtos com par para o mês entrar.
const MIN_COVERAGE: f64 = 0.4;
/// Quantos meses para trás procurar o preço anterior.
const LOOKBACK: i32 = 3;
/// Variação maior que isso (ou menor que o inverso) é tamanho mal lido.
const MAX_JUMP: f64 = 5.0;

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export, rename = "BasketArgs")]
pub struct BasketArgs {
    /// Meses mostrados (padrão 12).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub months: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "BasketMonth")]
pub struct Month {
    /// `AAAA-MM`.
    pub month: String,
    /// Sua cesta, base 100 no primeiro mês.
    pub index: f64,
    /// Variação do mês (fração); `null` quando o mês não teve cobertura.
    pub change: Option<f64>,
    /// Parte do gasto do mês em produtos com par (0 a 1).
    pub coverage: f64,
    /// O índice oficial de alimentos na mesma base (`null` sem dado).
    pub official: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "BasketMover")]
pub struct Mover {
    pub product_id: String,
    pub name: String,
    pub base_unit: Option<String>,
    /// Preço por kg, L ou unidade antes e agora (centavos).
    #[ts(type = "number")]
    pub before: i64,
    #[ts(type = "number")]
    pub now: i64,
    pub change: f64,
    /// Quanto a variação pesou no bolso nas compras dos últimos 3 meses (centavos).
    #[ts(type = "number")]
    pub impact: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "BasketInflation")]
pub struct BasketInflation {
    pub months: Vec<Month>,
    /// Sua cesta e o índice oficial no período (fração).
    pub change: Option<f64>,
    pub official_change: Option<f64>,
    /// Nome do índice oficial ("IPCA · Alimentação no domicílio").
    pub official_name: Option<String>,
    /// As compras dos últimos 3 meses (só as que têm preço de um ano antes) a preços de hoje e
    /// de um ano antes (centavos): "as mesmas compras custariam R$ X há um ano".
    #[ts(type = "number | null")]
    pub same_basket_now: Option<i64>,
    #[ts(type = "number | null")]
    pub same_basket_year_ago: Option<i64>,
    pub risers: Vec<Mover>,
    pub fallers: Vec<Mover>,
    /// Há meses com cobertura suficiente.
    pub enough: bool,
}

/// Uma compra com preço por unidade de base, líquida de desconto.
struct Buy {
    product: String,
    store: String,
    category: String,
    month: (i32, u32),
    price: f64,
    spent: f64,
    qty: f64,
}

fn key(m: (i32, u32)) -> i32 {
    m.0 * 12 + m.1 as i32 - 1
}

fn month_of(k: i32) -> (i32, u32) {
    (k.div_euclid(12), (k.rem_euclid(12) + 1) as u32)
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Variação mensal (fração) da série oficial de alimentos do país, por mês `AAAA-MM`: a
/// primeira série do perfil de consumo que tem dados (a principal ou a reserva). Séries de
/// taxa mensal valem como estão; as de nível (índice) viram a variação de um mês para o outro.
fn official_rates(
    conn: &Connection,
    country: &str,
    start: NaiveDate,
    today: NaiveDate,
) -> Result<Option<(String, HashMap<String, f64>)>> {
    use rust_decimal::prelude::ToPrimitive;
    for series in super::profile::of(country).official_series {
        let Some(kind) = conn
            .query_row(
                "SELECT kind FROM index_series WHERE id = ?1",
                [&series],
                |r| r.get::<_, String>(0),
            )
            .ok()
        else {
            continue;
        };
        let by_month: HashMap<String, f64> = if kind == "monthly_rate" {
            crate::indices::monthly_rates(conn, &series, start, today)?
                .iter()
                .filter_map(|r| {
                    Some((
                        format!("{:04}-{:02}", r.year, r.month),
                        r.pct.to_f64()? / 100.0,
                    ))
                })
                .collect()
        } else {
            // Um mês antes do começo, para o primeiro mês ter de onde variar.
            let from = start - chrono::Months::new(1);
            let levels: Vec<(String, f64)> = crate::indices::points(conn, &series, from, today)?
                .into_iter()
                .filter_map(|p| {
                    Some((
                        format!("{:04}-{:02}", p.date.year(), p.date.month()),
                        p.value.to_f64()?,
                    ))
                })
                .collect();
            levels
                .windows(2)
                .filter(|w| w[0].1 > 0.0 && w[0].0 != w[1].0)
                .map(|w| (w[1].0.clone(), w[1].1 / w[0].1 - 1.0))
                .collect()
        };
        if !by_month.is_empty() {
            return Ok(Some((series, by_month)));
        }
    }
    Ok(None)
}

fn buys(conn: &Connection, from: NaiveDate, to: NaiveDate) -> Result<Vec<Buy>> {
    let base = crate::household::base_currency(conn).unwrap_or_else(|_| "BRL".into());
    let mut stmt = conn.prepare(
        "SELECT i.product_id, COALESCE(r.store_cnpj, r.store_name, '?'),
                COALESCE(p.category_id, i.item_group), r.date, i.base_price, i.total,
                i.discount, i.base_qty
           FROM receipt_item i
           JOIN receipt r ON r.id = i.receipt_id
           JOIN product p ON p.id = i.product_id
          WHERE i.base_price IS NOT NULL AND i.base_price > 0 AND i.total > 0
            AND r.date BETWEEN ?1 AND ?2
            AND (r.currency IS NULL OR r.currency = ?3)",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![crate::time::fmt_date(from), crate::time::fmt_date(to), base],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, Option<String>>(7)?,
            ))
        },
    )?;
    let mut out = vec![];
    for row in rows {
        let (product, store, category, date, base_price, total, discount, qty) = row?;
        let Ok(d) = parse_date(&date) else {
            continue;
        };
        // Preço líquido: o desconto do item abate na mesma proporção.
        let net = (total - discount.clamp(0, total)) as f64;
        let factor = net / total as f64;
        out.push(Buy {
            product,
            store,
            category,
            month: (d.year(), d.month()),
            price: base_price as f64 * factor,
            spent: net,
            qty: qty.and_then(|q| q.parse().ok()).unwrap_or(0.0),
        });
    }
    Ok(out)
}

/// O índice da cesta nos últimos `months` meses até `today`.
pub fn basket(conn: &Connection, a: &BasketArgs, today: NaiveDate) -> Result<BasketInflation> {
    let months = a.months.unwrap_or(12).clamp(3, 36) as i32;
    let last = key((today.year(), today.month()));
    let first = last - months + 1;
    // Mais 15 meses antes, para o preço anterior e para "um ano antes".
    let (fy, fm) = month_of(first - 15);
    let from = NaiveDate::from_ymd_opt(fy, fm, 1).expect("dia 1");
    let all = buys(conn, from, today)?;

    // Preço do mês por produto e loja; gasto do mês e quantidade.
    let mut prices: HashMap<(String, String), BTreeMap<i32, Vec<f64>>> = HashMap::new();
    let mut spent_month: HashMap<i32, f64> = HashMap::new();
    let mut cat_of: HashMap<String, String> = HashMap::new();
    let mut weight: HashMap<String, f64> = HashMap::new();
    for b in &all {
        let k = key(b.month);
        prices
            .entry((b.product.clone(), b.store.clone()))
            .or_default()
            .entry(k)
            .or_default()
            .push(b.price);
        *spent_month.entry(k).or_default() += b.spent;
        cat_of.insert(b.product.clone(), b.category.clone());
        if k >= first {
            *weight.entry(b.category.clone()).or_default() += b.spent;
        }
    }
    let monthly: HashMap<(String, String), BTreeMap<i32, f64>> = prices
        .into_iter()
        .map(|(k, m)| {
            (
                k,
                m.into_iter()
                    .map(|(mk, mut v)| (mk, median(&mut v)))
                    .collect(),
            )
        })
        .collect();
    // Gasto do mês por produto e loja (para a cobertura).
    let mut spent_key: HashMap<(String, String, i32), f64> = HashMap::new();
    for b in &all {
        *spent_key
            .entry((b.product.clone(), b.store.clone(), key(b.month)))
            .or_default() += b.spent;
    }

    let mut out_months = vec![];
    let mut index = 100.0;
    let mut enough = false;
    for m in first..=last {
        // Variações elementares por categoria.
        let mut by_cat: HashMap<&str, Vec<f64>> = HashMap::new();
        let mut covered = 0.0;
        for ((product, store), series) in &monthly {
            let Some(now) = series.get(&m) else {
                continue;
            };
            let Some((_, prev)) = series.range(m - LOOKBACK..m).next_back() else {
                continue;
            };
            let ratio = now / prev;
            if !(1.0 / MAX_JUMP..=MAX_JUMP).contains(&ratio) {
                continue;
            }
            let cat = cat_of.get(product).map(String::as_str).unwrap_or("?");
            by_cat.entry(cat).or_default().push(ratio.ln());
            covered += spent_key
                .get(&(product.clone(), store.clone(), m))
                .copied()
                .unwrap_or(0.0);
        }
        let total = spent_month.get(&m).copied().unwrap_or(0.0);
        let coverage = if total > 0.0 { covered / total } else { 0.0 };
        let change = if coverage >= MIN_COVERAGE && m > first {
            let (mut sum, mut w) = (0.0, 0.0);
            for (cat, logs) in &by_cat {
                let cw = weight.get(*cat).copied().unwrap_or(0.0).max(1.0);
                let jevons = (logs.iter().sum::<f64>() / logs.len() as f64).exp();
                sum += cw * jevons;
                w += cw;
            }
            (w > 0.0).then(|| sum / w - 1.0)
        } else {
            None
        };
        if let Some(c) = change {
            index *= 1.0 + c;
            enough = true;
        }
        let (y, mo) = month_of(m);
        out_months.push(Month {
            month: format!("{y:04}-{mo:02}"),
            index,
            change,
            coverage,
            official: None,
        });
    }

    // Índice oficial na mesma base.
    let country = crate::country::family_country(conn);
    let (mut official_change, mut official_name) = (None, None);
    let (y, mo) = month_of(first);
    let start = NaiveDate::from_ymd_opt(y, mo, 1).expect("dia 1");
    if let Some((series, by_month)) = official_rates(conn, &country, start, today)? {
        let mut level = 100.0;
        let mut any = false;
        for (i, m) in out_months.iter_mut().enumerate() {
            if i > 0 {
                match by_month.get(&m.month) {
                    Some(r) => {
                        level *= 1.0 + r;
                        any = true;
                    }
                    None => continue,
                }
            }
            m.official = Some(level);
        }
        if any {
            official_change = Some(level / 100.0 - 1.0);
            official_name = conn
                .query_row(
                    "SELECT name FROM index_series WHERE id = ?1",
                    [&series],
                    |r| r.get(0),
                )
                .ok();
        }
    }

    // "As mesmas compras de 3 meses, um ano antes": preço da mesma loja perto de 12 meses antes.
    let (mut now_sum, mut then_sum) = (0.0, 0.0);
    for b in &all {
        let k = key(b.month);
        if k <= last - 3 || b.qty <= 0.0 {
            continue;
        }
        let Some(series) = monthly.get(&(b.product.clone(), b.store.clone())) else {
            continue;
        };
        let target = k - 12;
        let then = (0..=LOOKBACK)
            .flat_map(|d| [target - d, target + d])
            .find_map(|t| series.get(&t));
        if let Some(p) = then
            && (1.0 / MAX_JUMP..=MAX_JUMP).contains(&(b.price / p))
        {
            now_sum += b.spent;
            then_sum += p * b.qty;
        }
    }
    let recent: f64 = (last - 2..=last)
        .map(|m| spent_month.get(&m).copied().unwrap_or(0.0))
        .sum();
    let (same_now, same_then) = if recent > 0.0 && now_sum / recent >= 0.2 {
        (Some(now_sum.round() as i64), Some(then_sum.round() as i64))
    } else {
        (None, None)
    };

    // O que mais subiu e caiu: o primeiro e o último preço do período, na mesma loja.
    let mut movers: HashMap<String, Mover> = HashMap::new();
    for ((product, store), series) in &monthly {
        let in_period: Vec<(&i32, &f64)> = series.range(first..=last).collect();
        let (Some((k0, p0)), Some((k1, p1))) = (in_period.first(), in_period.last()) else {
            continue;
        };
        if **k1 - **k0 < 2 || **p0 <= 0.0 {
            continue;
        }
        let change = **p1 / **p0 - 1.0;
        if !(1.0 / MAX_JUMP..=MAX_JUMP).contains(&(**p1 / **p0)) {
            continue;
        }
        let qty: f64 = all
            .iter()
            .filter(|b| &b.product == product && &b.store == store && key(b.month) > last - 3)
            .map(|b| b.qty)
            .sum();
        let m = Mover {
            product_id: product.clone(),
            name: String::new(),
            base_unit: None,
            before: p0.round() as i64,
            now: p1.round() as i64,
            change,
            impact: ((**p1 - **p0) * qty).round() as i64,
        };
        // Um produto em várias lojas: fica a loja com mais impacto.
        match movers.get(product) {
            Some(cur) if cur.impact.abs() >= m.impact.abs() => {}
            _ => {
                movers.insert(product.clone(), m);
            }
        }
    }
    let mut movers: Vec<Mover> = movers.into_values().filter(|m| m.change != 0.0).collect();
    for m in &mut movers {
        if let Ok((name, unit)) = conn.query_row(
            "SELECT p.name, (SELECT base_unit FROM receipt_item WHERE product_id = p.id
                              AND base_unit IS NOT NULL LIMIT 1)
               FROM product p WHERE p.id = ?1",
            [&m.product_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
        ) {
            m.name = super::clean_description(&name);
            m.base_unit = unit;
        }
    }
    movers.sort_by(|a, b| b.change.total_cmp(&a.change));
    let risers: Vec<Mover> = movers
        .iter()
        .filter(|m| m.change > 0.0)
        .take(8)
        .cloned()
        .collect();
    let mut fallers: Vec<Mover> = movers.into_iter().filter(|m| m.change < 0.0).collect();
    fallers.reverse();
    fallers.truncate(8);

    let change = enough.then(|| index / 100.0 - 1.0);
    Ok(BasketInflation {
        months: out_months,
        change,
        official_change,
        official_name,
        same_basket_now: same_now,
        same_basket_year_ago: same_then,
        risers,
        fallers,
        enough,
    })
}

#[cfg(test)]
mod tests;
