//! Compra cadastrada à mão (docs/consumption.md, seção 4): feira, loja sem nota, outro país, cupom
//! apagado. Vira um cupom de origem `manual`, com os itens ligados aos produtos e a categoria
//! escolhida em cada um, e o lançamento: um que já existe (o valor tem de bater), um criado junto
//! (editar a compra ajusta o valor dele; apagar a compra apaga ele) ou nenhum (fica à parte e não
//! entra no gasto). Vale em qualquer país, na moeda da família; o imposto estimado é só do Brasil.

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Category, categories, pack, product_exists, root_of, store_key};
use crate::error::{Error, Result};
use crate::i18n::{args, money, t, t_args};
use crate::ids::{Id, new_id};
use crate::import::text::fold;
use crate::receipts::nfce::ParsedItem;
use crate::receipts::{self, GroupChoice, ItemGroup, Receipt, ReceiptOrigin, TxnMatch};
use crate::time::{fmt_date, now_utc, parse_date};
use crate::transactions::{TxnInput, TxnKind, TxnPatch};

/// Unidades da compra manual.
pub const UNITS: &[&str] = &["un", "kg", "g", "l", "ml", "lb", "oz", "gal", "qt", "floz"];

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, rename = "PurchaseItemInput")]
pub struct ItemInput {
    pub description: String,
    /// Decimal com ponto ou vírgula (`1`, `0,5`).
    #[serde(default)]
    #[ts(optional)]
    pub quantity: Option<String>,
    /// `un`, `kg`, `g`, `l` ou `ml`.
    #[serde(default)]
    #[ts(optional)]
    pub unit: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub unit_price: Option<i64>,
    #[ts(type = "number")]
    pub total: i64,
    /// Produto já comprado (escolhido na sugestão); sem ele, o núcleo procura pelo nome.
    #[serde(default)]
    #[ts(optional)]
    pub product_id: Option<Id>,
    /// Categoria de produto (vale para o produto, nesta e nas próximas compras).
    #[serde(default)]
    #[ts(optional)]
    pub category_id: Option<Id>,
    /// Código de barras (EAN/GTIN) digitado ou lido da foto; liga o item ao produto.
    #[serde(default)]
    #[ts(optional)]
    pub gtin: Option<String>,
    /// Marca de imposto do item, onde o imposto é somado no fim (EUA, Canadá).
    #[serde(default)]
    #[ts(optional)]
    pub taxable: Option<bool>,
}

/// Como a compra foi paga.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[serde(tag = "mode", rename_all = "snake_case")]
#[ts(export, rename = "PurchasePayment")]
pub enum Payment {
    /// Sem lançamento: a compra fica à parte e não entra no gasto.
    None,
    /// Um lançamento de despesa que já existe, com o mesmo valor da compra.
    Link { txn_id: Id },
    /// Cria a despesa junto, nesta conta (categoria padrão: Mercado, se for quase tudo comida).
    Create {
        account_id: Id,
        #[serde(default)]
        #[ts(optional)]
        category_id: Option<Id>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, rename = "PurchaseInput")]
pub struct PurchaseInput {
    /// Edita a compra manual com este id.
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<Id>,
    /// `YYYY-MM-DD`.
    pub date: String,
    #[serde(default)]
    #[ts(optional)]
    pub store_name: Option<String>,
    /// Padrão: o do lançamento.
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    pub items: Vec<ItemInput>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub discount: Option<i64>,
    /// Imposto somado no fim do cupom (EUA, Canadá): o pago é itens − desconto + imposto. Nos
    /// países em que o preço já tem o imposto, é ignorado.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub tax: Option<i64>,
    pub payment: Payment,
}

fn invalid(key: &str) -> Error {
    Error::invalid(t(key))
}

fn qty_of(q: Option<&str>) -> Result<Option<String>> {
    match q.map(str::trim).filter(|q| !q.is_empty()) {
        None => Ok(None),
        Some(q) => {
            let v: f64 = q
                .replace(',', ".")
                .parse()
                .map_err(|_| invalid("nucleo-alimentacao-erro-quantidade"))?;
            if !(v > 0.0 && v.is_finite()) {
                return Err(invalid("nucleo-alimentacao-erro-quantidade"));
            }
            Ok(Some(pack::qty_text(v)))
        }
    }
}

/// Grupo do cupom pela categoria escolhida: o da primeira categoria-semente da V135 no caminho
/// até a raiz (Hortifrúti, Limpeza…), para o imposto do Brasil.
pub(crate) fn group_of(cats: &HashMap<String, Category>, id: &str) -> Option<ItemGroup> {
    let mut cur = cats.get(id);
    while let Some(c) = cur {
        if let Some(g) = c
            .seed_key
            .as_deref()
            .and_then(|k| k.split('.').next())
            .and_then(|k| ItemGroup::ALL.iter().find(|g| g.key() == k))
        {
            return Some(*g);
        }
        cur = c.parent_id.as_ref().and_then(|p| cats.get(p));
    }
    None
}

/// Categoria de lançamento padrão da despesa criada junto: Mercado quando mais da metade do
/// valor é comida.
fn default_txn_category(conn: &Connection, food: i64, total: i64) -> Result<Option<Id>> {
    if food * 2 <= total {
        return Ok(None);
    }
    Ok(conn
        .query_row(
            "SELECT id FROM category WHERE seed_key = 'categorias-mercado'",
            [],
            |r| r.get(0),
        )
        .optional()?)
}

struct Old {
    txn_id: Option<Id>,
    txn_created: bool,
}

fn manual_purchase(conn: &Connection, id: &str) -> Result<Old> {
    let (origin, txn_id, txn_created): (String, Option<Id>, bool) = conn
        .query_row(
            "SELECT origin, txn_id, txn_created FROM receipt WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-cupom")))?;
    if origin != ReceiptOrigin::Manual.as_str() {
        return Err(invalid("nucleo-alimentacao-erro-so-manual"));
    }
    Ok(Old {
        txn_id,
        txn_created,
    })
}

/// Quanto de um lançamento já está detalhado em outras compras (fora a `except`).
pub(crate) fn detailed(conn: &Connection, txn_id: &str, except: Option<&str>) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(x), 0) FROM (
            SELECT p.amount AS x FROM receipt_payment p
             WHERE p.txn_id = ?1 AND p.receipt_id IS NOT ?2
            UNION ALL
            SELECT r.paid FROM receipt r
             WHERE r.txn_id = ?1 AND r.id IS NOT ?2
               AND NOT EXISTS (SELECT 1 FROM receipt_payment p
                                WHERE p.receipt_id = r.id AND p.txn_id = ?1))",
        params![txn_id, except],
        |r| r.get(0),
    )?)
}

/// O que o pagamento precisa saber da compra.
pub(crate) struct PayCtx<'a> {
    /// A compra que está sendo editada (`None`: nova).
    pub receipt_id: Option<&'a str>,
    /// O lançamento que nasceu com esta compra, se houver.
    pub own: Option<&'a Id>,
    pub paid: i64,
    pub date: &'a str,
    pub description: &'a str,
    pub store: Option<&'a str>,
    pub member_id: Option<&'a Id>,
    /// Quanto dos itens é comida (para a categoria padrão do lançamento criado).
    pub food: i64,
    pub items_total: i64,
}

/// O lançamento da compra, pela forma de pagamento: o id e se ele nasceu com ela. Vale para a
/// compra digitada e para o cupom fiscal (docs/design/consumption-review.md, A1).
pub(crate) fn resolve_payment(
    tx: &Transaction<'_>,
    payment: &Payment,
    c: &PayCtx<'_>,
    today: NaiveDate,
) -> Result<(Option<Id>, bool)> {
    Ok(match payment {
        Payment::None => (None, false),
        Payment::Link { txn_id } if c.own == Some(txn_id) => {
            // O lançamento nasceu com a compra: acompanha o valor, a data e a loja.
            crate::transactions::update(
                tx,
                &TxnPatch {
                    id: txn_id.clone(),
                    account_id: None,
                    date: Some(c.date.to_string()),
                    amount: Some(-c.paid),
                    kind: None,
                    status: None,
                    description: Some(c.description.to_string()),
                    payee: None,
                    notes: None,
                    category_id: None,
                    member_id: None,
                    splits: None,
                    tags: None,
                    project_id: None,
                    card_plastic_id: None,
                    orig_amount: None,
                    orig_currency: None,
                    transfer_account_id: None,
                    transfer_amount: None,
                },
                today,
            )?;
            (Some(txn_id.clone()), true)
        }
        Payment::Link { txn_id } => {
            let (amount, kind): (i64, String) = tx
                .query_row(
                    "SELECT amount, kind FROM txn WHERE id = ?1",
                    [txn_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?
                .ok_or_else(|| Error::not_found(t("nucleo-alimentacao-erro-pagamento")))?;
            if kind != TxnKind::Expense.as_str() {
                return Err(invalid("nucleo-alimentacao-erro-nao-despesa"));
            }
            // A compra pode detalhar só parte do lançamento (o resto fica na categoria dele),
            // e mais de uma compra pode usar o mesmo lançamento; a soma nunca passa do valor.
            let left = -amount - detailed(tx, txn_id, c.receipt_id)?;
            if c.paid > left {
                let cur = super::purchase_currency(tx);
                let lang = crate::i18n::current();
                return Err(Error::invalid(t_args(
                    "nucleo-alimentacao-erro-compra-maior",
                    &args(&[
                        ("sobra", money(lang, left.max(0), &cur).into()),
                        ("pago", money(lang, c.paid, &cur).into()),
                    ]),
                )));
            }
            (Some(txn_id.clone()), false)
        }
        Payment::Create {
            account_id,
            category_id,
        } => {
            let category = match category_id {
                Some(c) => Some(c.clone()),
                None => default_txn_category(tx, c.food, c.items_total)?,
            };
            let id = crate::transactions::create(
                tx,
                &TxnInput {
                    account_id: account_id.clone(),
                    date: c.date.to_string(),
                    amount: -c.paid,
                    kind: Some(TxnKind::Expense),
                    status: None,
                    description: c.description.to_string(),
                    payee: c.store.map(String::from),
                    notes: None,
                    category_id: category,
                    member_id: c.member_id.cloned(),
                    splits: None,
                    tags: None,
                    project_id: None,
                    card_plastic_id: None,
                    orig_amount: None,
                    orig_currency: None,
                },
                today,
            )?;
            (Some(id), true)
        }
    })
}

/// Grava a compra (nova ou editada) e devolve o id.
pub fn save(tx: &Transaction<'_>, p: &PurchaseInput, today: NaiveDate) -> Result<Id> {
    let date = fmt_date(parse_date(&p.date)?);
    let items: Vec<&ItemInput> = p
        .items
        .iter()
        .filter(|i| !i.description.trim().is_empty())
        .collect();
    if items.is_empty() {
        return Err(invalid("nucleo-alimentacao-erro-compra-vazia"));
    }
    if items.iter().any(|i| i.total <= 0) {
        return Err(invalid("nucleo-alimentacao-erro-item-valor"));
    }
    let mut parsed = vec![];
    for i in &items {
        let unit = i
            .unit
            .as_deref()
            .map(|u| u.trim().to_ascii_lowercase())
            .filter(|u| !u.is_empty());
        if unit.as_deref().is_some_and(|u| !UNITS.contains(&u)) {
            return Err(invalid("nucleo-alimentacao-erro-unidade"));
        }
        parsed.push(ParsedItem {
            code: None,
            gtin: None,
            description: i
                .description
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(120)
                .collect(),
            quantity: qty_of(i.quantity.as_deref())?,
            unit,
            unit_price: i.unit_price.filter(|v| *v > 0),
            total: i.total,
            discount: 0,
        });
    }
    let items_total: i64 = parsed.iter().map(|i| i.total).sum();
    let discount = p.discount.unwrap_or(0).abs();
    if discount >= items_total {
        return Err(invalid("nucleo-alimentacao-erro-desconto"));
    }
    let profile = super::profile::family(tx);
    let tax = match profile.tax {
        super::profile::TaxAtTill::Added => p.tax.unwrap_or(0).abs(),
        super::profile::TaxAtTill::Included => 0,
    };
    let paid = items_total - discount + tax;
    let store = p
        .store_name
        .as_deref()
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(80).collect::<String>());
    let old = match &p.id {
        Some(id) => Some(manual_purchase(tx, id)?),
        None => None,
    };
    if let Some(m) = &p.member_id {
        crate::household::get_member(tx, m)?;
    }
    for i in &items {
        if let Some(pid) = &i.product_id {
            product_exists(tx, pid)?;
        }
    }
    let cats: HashMap<String, Category> = categories(tx)?
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();
    for i in &items {
        if let Some(c) = &i.category_id
            && !cats.contains_key(c)
        {
            return Err(Error::not_found(t("nucleo-alimentacao-erro-categoria")));
        }
    }
    // Grupo e imposto (estimado pelo percentual do grupo de consumo, só no Brasil).
    let choices: Vec<GroupChoice> = items
        .iter()
        .enumerate()
        .filter_map(|(k, i)| {
            let g = group_of(&cats, i.category_id.as_deref()?)?;
            Some(GroupChoice {
                line: k as u32 + 1,
                group: g,
            })
        })
        .collect();
    let mut lines = receipts::with_tax(&parsed, None, &choices, &super::profile::family(tx).words);
    if !crate::country::is_brazil(tx) {
        receipts::without_tax(&mut lines);
    }
    if tax > 0 {
        let flags: Vec<Option<bool>> = items.iter().map(|i| i.taxable).collect();
        let food: Vec<bool> = items
            .iter()
            .zip(&lines)
            .map(
                |(i, l)| match i.category_id.as_ref().and_then(|c| cats.get(c)) {
                    Some(c) => c.is_food,
                    None => super::seed::ROOTS
                        .iter()
                        .any(|r| r.group == l.group && r.is_food),
                },
            )
            .collect();
        receipts::spread_printed(&mut lines, tax, &flags, &food);
    }
    let food: i64 = items
        .iter()
        .zip(&lines)
        .filter(
            |(i, l)| match i.category_id.as_ref().and_then(|c| cats.get(c)) {
                Some(c) => c.is_food,
                None => super::seed::ROOTS
                    .iter()
                    .any(|r| r.group == l.group && r.is_food),
            },
        )
        .map(|(_, l)| l.total)
        .sum();

    // Lançamento.
    let own = old
        .as_ref()
        .filter(|o| o.txn_created)
        .and_then(|o| o.txn_id.clone());
    let description = store
        .clone()
        .unwrap_or_else(|| t("nucleo-alimentacao-compra-sem-loja"));
    let (txn_id, created) = resolve_payment(
        tx,
        &p.payment,
        &PayCtx {
            receipt_id: p.id.as_deref(),
            own: own.as_ref(),
            paid,
            date: &date,
            description: &description,
            store: store.as_deref(),
            member_id: p.member_id.as_ref(),
            food,
            items_total,
        },
        today,
    )?;
    let member_id = match &p.member_id {
        Some(m) => Some(m.clone()),
        None => match &txn_id {
            Some(t) => {
                tx.query_row("SELECT member_id FROM txn WHERE id = ?1", [t], |r| r.get(0))?
            }
            None => None,
        },
    };

    // O cupom.
    let now = now_utc();
    let id = match &p.id {
        Some(id) => {
            tx.execute(
                "UPDATE receipt SET date = ?2, store_name = ?3, items_total = ?4, discount = ?5,
                        paid = ?6, txn_id = ?7, member_id = ?8, txn_created = ?9, updated_at = ?10,
                        tax_total = ?11
                  WHERE id = ?1",
                params![
                    id,
                    date,
                    store,
                    items_total,
                    discount,
                    paid,
                    txn_id,
                    member_id,
                    created,
                    now,
                    (tax > 0).then_some(tax)
                ],
            )?;
            tx.execute("DELETE FROM receipt_item WHERE receipt_id = ?1", [id])?;
            tx.execute("DELETE FROM receipt_payment WHERE receipt_id = ?1", [id])?;
            id.clone()
        }
        None => {
            let id = new_id();
            tx.execute(
                "INSERT INTO receipt (id, date, store_name, items_total, discount, paid, origin,
                                      txn_id, member_id, txn_created, created_at, updated_at,
                                      tax_total)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, ?12)",
                params![
                    id,
                    date,
                    store,
                    items_total,
                    discount,
                    paid,
                    ReceiptOrigin::Manual.as_str(),
                    txn_id,
                    member_id,
                    created,
                    now,
                    (tax > 0).then_some(tax)
                ],
            )?;
            id
        }
    };
    // O lançamento criado com a compra e que ela deixou de usar sai junto.
    if let Some(o) = own.filter(|o| txn_id.as_ref() != Some(o)) {
        crate::transactions::delete(tx, &[o], today)?;
    }
    if let Some(t) = &txn_id {
        tx.execute(
            "INSERT INTO receipt_payment (id, receipt_id, txn_id, amount, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![new_id(), id, t, paid, now],
        )?;
    }

    // Os itens, já com o produto quando ele é conhecido (escolhido, pelo código de barras ou
    // com o mesmo nome).
    let skey = store_key(None, store.as_deref());
    let codes: Vec<Option<String>> = items
        .iter()
        .map(|i| {
            i.gtin
                .as_deref()
                .filter(|g| !g.trim().is_empty())
                .map(super::gtin::check)
                .transpose()
        })
        .collect::<Result<_>>()?;
    for (k, (it, line)) in items.iter().zip(&lines).enumerate() {
        let by_code: Option<Id> = match &codes[k] {
            Some(g) => tx
                .query_row(
                    "SELECT product_id FROM product_gtin WHERE gtin = ?1",
                    [g],
                    |r| r.get(0),
                )
                .optional()?,
            None => None,
        };
        let product: Option<Id> = match (&it.product_id, by_code) {
            (Some(p), _) => Some(p.clone()),
            (None, Some(p)) => Some(p),
            (None, None) => tx
                .query_row(
                    "SELECT p.id FROM product p WHERE lower(p.name) = lower(?1)
                     UNION ALL
                     SELECT a.product_id FROM product_alias a WHERE a.description = ?2
                     LIMIT 1",
                    params![line.description, fold(&line.description)],
                    |r| r.get(0),
                )
                .optional()?,
        };
        let base = product.as_ref().and_then(|_| {
            pack::base_of(
                &line.description,
                line.quantity.as_deref(),
                line.unit.as_deref(),
            )
        });
        tx.execute(
            "INSERT INTO receipt_item (id, receipt_id, line, description, quantity, unit,
                                       unit_price, total, item_group, group_origin, ibpt_product,
                                       tax_rate, tax, tax_origin, product_id, base_qty, base_unit,
                                       base_price, gtin, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                     ?18, ?20, ?19, ?19)",
            params![
                new_id(),
                id,
                k as u32 + 1,
                line.description,
                line.quantity,
                line.unit,
                line.unit_price,
                line.total,
                line.group.key(),
                line.group_origin,
                Option::<String>::None,
                line.tax_rate,
                line.tax,
                crate::receipts::origin_to_db(&line.tax_origin),
                product,
                base.map(|b| pack::qty_text(b.0)),
                base.map(|b| b.1),
                base.and_then(|b| pack::base_price(line.total, b.0)),
                now,
                codes[k],
            ],
        )?;
        // Produto já conhecido: o código de barras passa a ser dele.
        if let (Some(pid), Some(g)) = (&product, &codes[k]) {
            super::add_gtin(tx, pid, g)?;
        }
        if let Some(pid) = &product {
            tx.execute(
                "INSERT INTO product_alias (id, product_id, store_key, code, description, created_at)
                 VALUES (?1, ?2, ?3, '', ?4, ?5) ON CONFLICT (store_key, code, description) DO NOTHING",
                params![new_id(), pid, skey, fold(&line.description), now],
            )?;
        }
    }
    super::link_items(tx, Some(&id))?;
    // A categoria escolhida vale para o produto.
    for (k, it) in items.iter().enumerate() {
        if let Some(c) = &it.category_id {
            tx.execute(
                "UPDATE product SET category_id = ?3, category_origin = 'manual', updated_at = ?4
                  WHERE id = (SELECT product_id FROM receipt_item WHERE receipt_id = ?1 AND line = ?2)
                    AND category_id IS NOT ?3",
                params![id, k as u32 + 1, c, now],
            )?;
        }
    }
    Ok(id)
}

/// Apaga a compra; o lançamento criado junto com ela sai também.
pub fn delete(tx: &Transaction<'_>, id: &str, today: NaiveDate) -> Result<()> {
    let old = manual_purchase(tx, id)?;
    let _ = old;
    receipts::delete(tx, id, today)
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, rename = "PurchaseMatchArgs")]
pub struct MatchArgs {
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// A compra sendo editada (o lançamento dela também serve).
    #[serde(default)]
    #[ts(optional)]
    pub purchase_id: Option<Id>,
}

/// Despesas perto da data (de 2 dias antes a 5 depois) em que a compra ainda cabe (o valor do
/// lançamento menos o que outras compras já detalharam): as candidatas a pagar a compra, a de
/// valor igual primeiro, depois a mais perto.
pub fn candidates(conn: &Connection, a: &MatchArgs) -> Result<Vec<TxnMatch>> {
    let date = fmt_date(parse_date(&a.date)?);
    let mut stmt = conn.prepare(
        "SELECT t.id, t.date, t.amount, t.description, a.name, t.member_id FROM txn t
           JOIN account a ON a.id = t.account_id
          WHERE t.kind = 'expense' AND t.status <> 'planned'
            AND t.date BETWEEN date(?2, '-2 days') AND date(?2, '+5 days')
            AND -t.amount
                - COALESCE((SELECT SUM(x.amount) FROM receipt_payment x
                             WHERE x.txn_id = t.id AND x.receipt_id IS NOT ?3), 0)
                - COALESCE((SELECT SUM(r.paid) FROM receipt r
                             WHERE r.txn_id = t.id AND r.id IS NOT ?3
                               AND NOT EXISTS (SELECT 1 FROM receipt_payment x
                                                WHERE x.receipt_id = r.id AND x.txn_id = t.id)), 0)
                >= -?1
          ORDER BY (t.amount <> ?1), abs(julianday(t.date) - julianday(?2)), t.date LIMIT 5",
    )?;
    let rows = stmt
        .query_map(params![-a.amount.abs(), date, a.purchase_id], |r| {
            Ok(TxnMatch {
                txn_id: r.get(0)?,
                date: r.get(1)?,
                amount: r.get(2)?,
                description: r.get(3)?,
                account_name: r.get(4)?,
                member_id: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// As últimas lojas (para escolher na compra manual).
pub fn stores(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT store_name FROM receipt WHERE store_name IS NOT NULL AND trim(store_name) <> ''
          GROUP BY lower(store_name) ORDER BY MAX(date) DESC LIMIT 20",
    )?;
    let rows = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Lista de compras

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export, rename = "PurchasesArgs")]
pub struct PurchasesArgs {
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Loja ou item.
    #[serde(default)]
    #[ts(optional)]
    pub query: Option<String>,
    /// Só as compras com itens deste segmento (raiz da árvore de categorias).
    #[serde(default)]
    #[ts(optional)]
    pub segment_id: Option<Id>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "PurchaseSegment")]
pub struct SegmentShare {
    /// `null`: sem categoria.
    pub category_id: Option<Id>,
    pub name: String,
    #[ts(type = "number")]
    pub spent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "Purchase")]
pub struct Purchase {
    pub receipt: Receipt,
    /// Quanto foi em cada segmento (maior primeiro).
    pub segments: Vec<SegmentShare>,
}

pub fn list(conn: &Connection, a: &PurchasesArgs) -> Result<Vec<Purchase>> {
    let receipts = receipts::list(
        conn,
        &receipts::ReceiptListArgs {
            from: a.from.clone(),
            to: a.to.clone(),
            member_id: a.member_id.clone(),
            limit: Some(1000),
        },
    )?;
    let cats: HashMap<String, Category> = categories(conn)?
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();
    let q = a.query.as_deref().map(fold).filter(|q| !q.is_empty());
    let mut stmt = conn.prepare(
        "SELECT i.description, i.total, p.category_id FROM receipt_item i
           LEFT JOIN product p ON p.id = i.product_id WHERE i.receipt_id = ?1",
    )?;
    let mut out = vec![];
    for r in receipts {
        let items: Vec<(String, i64, Option<String>)> = stmt
            .query_map([&r.id], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        if let Some(q) = &q {
            let store = r.store_name.as_deref().map(fold).unwrap_or_default();
            if !store.contains(q.as_str()) && !items.iter().any(|i| fold(&i.0).contains(q.as_str()))
            {
                continue;
            }
        }
        let mut by: HashMap<Option<String>, i64> = HashMap::new();
        for (_, total, cat) in &items {
            *by.entry(cat.as_deref().map(|c| root_of(&cats, c)))
                .or_default() += total;
        }
        if a.segment_id
            .as_ref()
            .is_some_and(|s| !by.contains_key(&Some(s.clone())))
        {
            continue;
        }
        let mut segments: Vec<SegmentShare> = by
            .into_iter()
            .map(|(id, spent)| SegmentShare {
                name: id
                    .as_ref()
                    .and_then(|i| cats.get(i))
                    .map_or_else(|| t("nucleo-alimentacao-sem-categoria"), |c| c.name.clone()),
                category_id: id,
                spent,
            })
            .collect();
        segments.sort_by_key(|s| -s.spent);
        out.push(Purchase {
            receipt: r,
            segments,
        });
    }
    Ok(out)
}
