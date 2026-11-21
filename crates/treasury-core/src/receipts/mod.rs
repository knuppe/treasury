//! Cupons fiscais (docs/receipts.md): a NFC-e lida pelo link do QR do cupom (consulta pública
//! da SEFAZ), pelo texto dessa página ou pelo PDF do DANFE NFC-e; e o cupom que o agente lê de
//! uma foto. Cada item ganha um grupo (hortifrúti, carnes, limpeza…) e o imposto: o impresso na
//! nota (Lei 12.741/2012), rateado pelos itens, ou o estimado pelo percentual do grupo de consumo. O cupom liga ao
//! lançamento da compra, e o membro vem dele: dá para ver o que cada um compra e quanto de
//! imposto vai em cada grupo. A NFC-e e o imposto são do Brasil; os itens informados direto e a
//! compra cadastrada à mão (`food::purchase`, docs/consumption.md) valem em qualquer país.

pub mod classify;
pub mod classify_words;
pub mod foreign;
pub mod nfce;
pub mod ocr;

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use self::classify::ItemGroup;
use self::nfce::ParsedReceipt;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{now_utc, parse_date};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ReceiptOrigin {
    /// Página da consulta pela URL do QR.
    Nfce,
    Pdf,
    /// XML da NF-e ou NFC-e (o arquivo que a loja manda por e-mail).
    Xml,
    /// Texto da página ou do cupom.
    Text,
    /// Itens lidos pelo agente (foto do cupom).
    Agent,
    /// Compra cadastrada à mão na tela Consumo (docs/consumption.md): dá para editar.
    Manual,
}

impl ReceiptOrigin {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Nfce => "nfce",
            Self::Pdf => "pdf",
            Self::Xml => "xml",
            Self::Text => "text",
            Self::Agent => "agent",
            Self::Manual => "manual",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "nfce" => Self::Nfce,
            "pdf" => Self::Pdf,
            "xml" => Self::Xml,
            "agent" => Self::Agent,
            "manual" => Self::Manual,
            _ => Self::Text,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ReceiptFile {
    pub filename: String,
    pub content_base64: String,
}

/// Item informado direto (o agente, de uma foto do cupom).
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ManualItem {
    pub description: String,
    #[serde(default)]
    #[ts(optional)]
    pub quantity: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub unit: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub unit_price: Option<i64>,
    #[ts(type = "number")]
    pub total: i64,
    /// Marca de imposto do item, onde o imposto é somado no fim do cupom (EUA, Canadá).
    #[serde(default)]
    #[ts(optional)]
    pub taxable: Option<bool>,
}

/// Cupom informado direto (o agente, de uma foto).
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ManualReceipt {
    /// `YYYY-MM-DD`.
    pub date: String,
    #[serde(default)]
    #[ts(optional)]
    pub store_name: Option<String>,
    pub items: Vec<ManualItem>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub discount: Option<i64>,
    /// Valor pago (padrão: itens − desconto).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub paid: Option<i64>,
    /// Tributos aproximados impressos no cupom, se houver. Onde o imposto é somado no fim
    /// (EUA, Canadá), é a linha de imposto: o pago é itens − desconto + este valor.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub tax_total: Option<i64>,
    /// Lida da foto pelo app (OCR, docs/ocr-receipts.md): grava com a origem `text`.
    #[serde(default)]
    #[ts(optional)]
    pub from_photo: Option<bool>,
}

/// Grupo de imposto de um item (pela linha), derivado da categoria escolhida.
#[derive(Debug, Clone)]
pub(crate) struct GroupChoice {
    pub line: u32,
    pub group: ItemGroup,
}

/// Categoria de produto escolhida para um item (pela linha). Vale para o produto, nesta e nas
/// próximas compras.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export, rename = "ReceiptCategoryChoice")]
pub struct CategoryChoice {
    #[ts(type = "number")]
    pub line: u32,
    pub category_id: Id,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ReceiptArgs {
    /// Link do QR do cupom (consulta pública da SEFAZ).
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
    /// Texto da página da consulta ou do cupom.
    #[serde(default)]
    #[ts(optional)]
    pub text: Option<String>,
    /// PDF do DANFE NFC-e ou o XML da NF-e/NFC-e.
    #[serde(default)]
    #[ts(optional)]
    pub files: Option<Vec<ReceiptFile>>,
    #[serde(default)]
    #[ts(optional)]
    pub manual: Option<ManualReceipt>,
    /// Data quando o cupom não traz (`YYYY-MM-DD`).
    #[serde(default)]
    #[ts(optional)]
    pub date: Option<String>,
    /// Lançamento da compra (padrão: o que bate com o valor pago, perto da data).
    #[serde(default)]
    #[ts(optional)]
    pub txn_id: Option<Id>,
    /// Membro (padrão: o do lançamento).
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Categoria de produto escolhida por linha (o resto fica com a prevista).
    #[serde(default)]
    #[ts(optional)]
    pub categories: Option<Vec<CategoryChoice>>,
    /// Como a compra foi paga. Ausente: o lançamento de `txn_id` ou o que bate com o valor.
    #[serde(default)]
    #[ts(optional)]
    pub payment: Option<crate::food::purchase::Payment>,
    /// Substitui o cupom com a mesma chave de acesso.
    #[serde(default)]
    #[ts(optional)]
    pub replace: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReceiptItem {
    #[ts(type = "number")]
    pub line: u32,
    pub code: Option<String>,
    pub description: String,
    pub quantity: Option<String>,
    pub unit: Option<String>,
    #[ts(type = "number | null")]
    pub unit_price: Option<i64>,
    #[ts(type = "number")]
    pub total: i64,
    pub group: ItemGroup,
    pub group_label: String,
    /// `auto` (pela descrição) ou `manual`.
    pub group_origin: String,
    /// % de tributos no preço (o do grupo de consumo do item, pelas Contas Nacionais do IBGE,
    /// ou o impresso na nota rateado).
    pub tax_rate: f64,
    #[ts(type = "number")]
    pub tax: i64,
    /// `nfce`: parte do imposto impresso na nota; `estimated`: estimado pelo percentual do
    /// grupo de consumo.
    pub tax_origin: String,
    /// GTIN/EAN do item (do XML ou do código da loja).
    #[serde(default)]
    pub gtin: Option<String>,
    /// Desconto do próprio item (`vDesc` do XML), em centavos.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub discount: i64,
    /// Produto da tela Alimentação (docs/consumption.md) e o preço por kg, L ou unidade.
    #[serde(default)]
    pub product_id: Option<Id>,
    #[serde(default)]
    pub base_unit: Option<String>,
    #[serde(default)]
    #[ts(type = "number | null")]
    pub base_price: Option<i64>,
    /// Categoria de produto: a do produto (cupom gravado) ou a prevista (prévia).
    #[serde(default)]
    pub category_id: Option<Id>,
    /// De onde veio a categoria prevista (só na prévia).
    #[serde(default)]
    #[ts(optional)]
    pub category_reason: Option<crate::food::classify::Reason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TxnMatch {
    pub txn_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub description: String,
    pub account_name: String,
    pub member_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReceiptPreview {
    pub origin: ReceiptOrigin,
    pub parsed: ParsedReceipt,
    pub items: Vec<ReceiptItem>,
    /// Imposto do cupom: o impresso (`tax_printed`) ou a soma estimada dos itens.
    #[ts(type = "number")]
    pub tax: i64,
    pub tax_printed: bool,
    /// Já existe um cupom com a mesma chave de acesso.
    pub duplicate: Option<Id>,
    pub txn: Option<TxnMatch>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Receipt {
    pub id: Id,
    pub access_key: Option<String>,
    pub date: String,
    pub issued_at: Option<String>,
    pub store_name: Option<String>,
    pub store_cnpj: Option<String>,
    pub uf: Option<String>,
    #[ts(type = "number")]
    pub items_total: i64,
    #[ts(type = "number")]
    pub discount: i64,
    #[ts(type = "number")]
    pub paid: i64,
    /// Imposto do cupom (impresso ou estimado pelos itens).
    #[ts(type = "number")]
    pub tax: i64,
    pub tax_printed: bool,
    pub origin: ReceiptOrigin,
    pub txn_id: Option<Id>,
    /// O lançamento nasceu junto com a compra manual (editar a compra ajusta o valor dele).
    #[serde(default)]
    pub txn_created: bool,
    /// Imposto somado no fim do cupom (pago − itens + desconto), onde o preço do item não
    /// traz o imposto; zero nos demais.
    #[serde(default)]
    #[ts(type = "number")]
    pub tax_added: i64,
    pub member_id: Option<Id>,
    pub member_name: Option<String>,
    #[ts(type = "number")]
    pub item_count: u32,
    /// Só em `receipts.get`.
    pub items: Vec<ReceiptItem>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ReceiptListArgs {
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GroupSpend {
    /// Segmento de consumo (a raiz da categoria de produto); `null`: item sem categoria.
    pub segment_id: Option<Id>,
    pub label: String,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub tax: i64,
    #[ts(type = "number")]
    pub items: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberSpend {
    /// `null`: da família (cupom sem membro).
    pub member_id: Option<Id>,
    pub member_name: String,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub tax: i64,
    pub groups: Vec<GroupSpend>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TopItem {
    pub description: String,
    /// Nome do segmento do item.
    pub segment: String,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub times: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReceiptSummary {
    pub from: String,
    pub to: String,
    #[ts(type = "number")]
    pub receipts: u32,
    #[ts(type = "number")]
    pub spent: i64,
    #[ts(type = "number")]
    pub tax: i64,
    /// Parte do imposto que veio impressa nas notas (o resto, estimado pelos itens).
    #[ts(type = "number")]
    pub tax_printed: i64,
    pub groups: Vec<GroupSpend>,
    pub members: Vec<MemberSpend>,
    pub top_items: Vec<TopItem>,
}

/// A leitura de cupons é do Brasil (NFC-e).
/// "cupom {id}" (detalhe do erro "não encontrado").
fn receipt_ref(id: &str) -> String {
    t_args(
        "nucleo-importacao-cupom-ref",
        &args(&[("id", id.to_string().into())]),
    )
}

fn ensure_brazil(conn: &Connection) -> Result<()> {
    if crate::country::is_brazil(conn) {
        Ok(())
    } else {
        Err(Error::invalid(t("nucleo-importacao-cupom-erro-brasil")))
    }
}

/// Origem "estimado" do imposto de um item, como a API a mostra.
pub(crate) const ESTIMATED: &str = "estimated";
/// A mesma origem como a coluna `tax_origin` a guarda (o `CHECK` da tabela só aceita `nfce` e
/// este valor antigo).
const ESTIMATED_DB: &str = "ibpt";

/// Origem do imposto de um item no formato da coluna.
pub(crate) fn origin_to_db(origin: &str) -> &str {
    if origin == ESTIMATED {
        ESTIMATED_DB
    } else {
        origin
    }
}

fn origin_from_db(origin: String) -> String {
    if origin == ESTIMATED_DB {
        ESTIMATED.into()
    } else {
        origin
    }
}

/// % de tributos do grupo de consumo do item (o "Provável" do Brasil); grupo sem dado oficial
/// fica sem imposto estimado.
fn group_rate(tax_group: &str) -> f64 {
    crate::tax_burden::dataset::get("BR")
        .ok()
        .and_then(|d| {
            d.consumption
                .into_iter()
                .find(|g| g.key == tax_group)
                .and_then(|g| match g.central {
                    Some(crate::tax_burden::dataset::RateSpec::Percent(p)) => Some(p),
                    _ => None,
                })
        })
        .unwrap_or(0.0)
}

/// Grupo e alíquota de cada item; o imposto impresso é rateado pelos itens na
/// proporção do imposto estimado de cada um (a soma bate com a nota).
pub(crate) fn with_tax(
    items: &[nfce::ParsedItem],
    printed: Option<i64>,
    choices: &[GroupChoice],
    langs: &[String],
) -> Vec<ReceiptItem> {
    let mut out: Vec<ReceiptItem> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let line = i as u32 + 1;
            let (auto, _) = classify::classify_in(&it.description, langs);
            let chosen = choices.iter().find(|c| c.line == line).map(|c| c.group);
            let group = chosen.unwrap_or(auto);
            let rate = group_rate(group.tax_group());
            ReceiptItem {
                line,
                code: it.code.clone(),
                description: it.description.clone(),
                quantity: it.quantity.clone(),
                unit: it.unit.clone(),
                unit_price: it.unit_price,
                total: it.total,
                group,
                group_label: group.label(),
                group_origin: if chosen.is_some() { "manual" } else { "auto" }.into(),
                tax_rate: rate,
                tax: (it.total as f64 * rate / 100.0).round() as i64,
                tax_origin: ESTIMATED.into(),
                gtin: it.gtin.clone(),
                discount: it.discount,
                product_id: None,
                base_unit: None,
                base_price: None,
                category_id: None,
                category_reason: None,
            }
        })
        .collect();
    if let Some(total) = printed.filter(|t| *t > 0) {
        let weight: i64 = out.iter().map(|i| i.tax.max(1)).sum();
        let mut given = 0;
        let n = out.len();
        for (k, it) in out.iter_mut().enumerate() {
            let share = if k + 1 == n {
                total - given
            } else {
                (total as f64 * it.tax.max(1) as f64 / weight as f64).round() as i64
            };
            given += share;
            it.tax = share;
            it.tax_origin = "nfce".into();
            if it.total > 0 {
                it.tax_rate = (share as f64 / it.total as f64 * 10_000.0).round() / 100.0;
            }
        }
    }
    out
}

/// O imposto impresso no cupom (a linha de imposto dos EUA e do Canadá) rateado pelos itens
/// tributáveis, na proporção do valor: os marcados como tributáveis; sem marca, os que não são
/// comida (na maioria dos estados a comida para casa é isenta ou tem alíquota reduzida); sem
/// nenhum deles, todos. A soma dos itens bate com o impresso.
pub(crate) fn spread_printed(
    items: &mut [ReceiptItem],
    tax: i64,
    taxable: &[Option<bool>],
    food: &[bool],
) {
    let flag = |k: usize| taxable.get(k).copied().flatten();
    let is_food = |k: usize| food.get(k).copied().unwrap_or(false);
    let n = items.len();
    let mut on: Vec<usize> = (0..n).filter(|k| flag(*k) == Some(true)).collect();
    if on.is_empty() {
        on = (0..n)
            .filter(|k| flag(*k) != Some(false) && !is_food(*k))
            .collect();
    }
    if on.is_empty() {
        on = (0..n).filter(|k| flag(*k) != Some(false)).collect();
    }
    if on.is_empty() {
        on = (0..n).collect();
    }
    let weight: i64 = on.iter().map(|k| items[*k].total.max(1)).sum();
    let mut given = 0;
    for it in items.iter_mut() {
        it.tax_rate = 0.0;
        it.tax = 0;
        // "nfce" é a origem "impresso no documento".
        it.tax_origin = "nfce".into();
    }
    for (pos, k) in on.iter().enumerate() {
        let it = &mut items[*k];
        let share = if pos + 1 == on.len() {
            tax - given
        } else {
            (tax as f64 * it.total.max(1) as f64 / weight as f64).round() as i64
        };
        given += share;
        it.tax = share;
        if it.total > 0 {
            it.tax_rate = (share as f64 / it.total as f64 * 10_000.0).round() / 100.0;
        }
    }
}

/// Fora do Brasil não há percentual por grupo: o item fica sem imposto.
pub(crate) fn without_tax(items: &mut [ReceiptItem]) {
    for it in items {
        it.tax_rate = 0.0;
        it.tax = 0;
    }
}

/// Só links das SEFAZ (domínios `.gov.br`, em https).
fn check_url(url: &str) -> Result<String> {
    let u = url.trim();
    let rest = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))
        .ok_or_else(|| Error::invalid(t("nucleo-importacao-cupom-erro-link")))?;
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !(host.ends_with(".gov.br")) {
        return Err(Error::invalid(t_args(
            "nucleo-importacao-cupom-erro-sefaz",
            &args(&[("host", host.into())]),
        )));
    }
    // Algumas SEFAZ ainda imprimem o QR com http; a consulta é sempre por https.
    Ok(format!("https://{rest}"))
}

/// Páginas da consulta já lidas nesta sessão do app, pela chave de acesso (ou pelo link): a
/// prévia e a gravação usam a mesma página, e o mesmo QR não volta à SEFAZ (que pode pedir
/// captcha na segunda vez). Só entra página com itens.
const PAGE_TTL: std::time::Duration = std::time::Duration::from_secs(12 * 3600);
const PAGE_MAX: usize = 64;

type PageCache = std::sync::Mutex<Vec<(String, std::time::Instant, String)>>;

fn pages() -> &'static PageCache {
    static PAGES: std::sync::OnceLock<PageCache> = std::sync::OnceLock::new();
    PAGES.get_or_init(Default::default)
}

/// Chave do cache: a chave de acesso do link, ou o link em https.
fn page_key(url: &str) -> String {
    nfce::access_key(url).unwrap_or_else(|| check_url(url).unwrap_or_else(|_| url.trim().into()))
}

fn cached_page(key: &str) -> Option<String> {
    let mut list = pages().lock().ok()?;
    list.retain(|(_, at, _)| at.elapsed() < PAGE_TTL);
    list.iter()
        .find(|(k, ..)| k == key)
        .map(|(.., html)| html.clone())
}

pub(crate) fn remember_page(key: &str, html: &str) {
    if let Ok(mut list) = pages().lock() {
        list.retain(|(k, at, _)| k != key && at.elapsed() < PAGE_TTL);
        if list.len() >= PAGE_MAX {
            list.remove(0);
        }
        list.push((key.to_string(), std::time::Instant::now(), html.to_string()));
    }
}

/// O cupom desta chave já foi gravado? Então não consulta a SEFAZ de novo: avisa.
fn already_imported(conn: &Connection, url: &str) -> Result<()> {
    let Some(key) = nfce::access_key(url) else {
        return Ok(());
    };
    let found: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT date, store_name FROM receipt WHERE access_key = ?1",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match found {
        Some((date, store)) => Err(Error::Conflict(t_args(
            "nucleo-importacao-cupom-erro-ja-importado",
            &args(&[
                (
                    "data",
                    parse_date(&date)
                        .map(|d| crate::i18n::date(crate::i18n::current(), d))
                        .unwrap_or(date)
                        .into(),
                ),
                ("loja", store.unwrap_or_default().into()),
            ]),
        ))),
        None => Ok(()),
    }
}

fn fetch(conn: &Connection, url: &str) -> Result<String> {
    crate::sources::ensure_enabled(conn, crate::sources::SourceId::Sefaz)?;
    let url = check_url(url)?;
    let agent = crate::indices::sync::agent();
    crate::indices::sync::get_text(&agent, &url).map_err(|e| {
        Error::invalid(t_args(
            "nucleo-importacao-cupom-erro-consulta",
            &args(&[("erro", e.to_string().into())]),
        ))
    })
}

fn read(
    conn: &Connection,
    a: &ReceiptArgs,
) -> Result<(ReceiptOrigin, ParsedReceipt, Option<String>)> {
    if let Some(m) = &a.manual {
        let date = parse_date(&m.date)?;
        let items: Vec<nfce::ParsedItem> = m
            .items
            .iter()
            .filter(|i| !i.description.trim().is_empty() && i.total != 0)
            .map(|i| nfce::ParsedItem {
                code: None,
                gtin: None,
                description: i.description.trim().chars().take(120).collect(),
                quantity: i.quantity.clone(),
                unit: i.unit.clone(),
                unit_price: i.unit_price,
                total: i.total.abs(),
                discount: 0,
            })
            .collect();
        let items_total: i64 = items.iter().map(|i| i.total).sum();
        let discount = m.discount.unwrap_or(0).abs();
        // Onde o imposto é somado no fim do cupom, ele entra no pago.
        let added = match crate::food::profile::family(conn).tax {
            crate::food::profile::TaxAtTill::Added => m.tax_total.map(i64::abs).unwrap_or(0),
            crate::food::profile::TaxAtTill::Included => 0,
        };
        let p = ParsedReceipt {
            store_name: m.store_name.clone().filter(|s| !s.trim().is_empty()),
            date: Some(crate::time::fmt_date(date)),
            items_total,
            discount,
            paid: m
                .paid
                .map(i64::abs)
                .unwrap_or(items_total - discount + added),
            tax_total: m.tax_total.map(i64::abs),
            items,
            ..ParsedReceipt::default()
        };
        let origin = if m.from_photo == Some(true) {
            ReceiptOrigin::Text
        } else {
            ReceiptOrigin::Agent
        };
        return Ok((origin, p, None));
    }
    if let Some(url) = a.url.as_deref().filter(|u| !u.trim().is_empty()) {
        if a.replace != Some(true) {
            already_imported(conn, url)?;
        }
        let key = page_key(url);
        let html = match cached_page(&key) {
            Some(h) => h,
            None => fetch(conn, url)?,
        };
        let mut p = nfce::parse_html(&html);
        if p.access_key.is_none() {
            p.access_key = nfce::access_key(url);
            p.uf = p
                .access_key
                .as_deref()
                .and_then(nfce::uf_of_key)
                .map(String::from);
        }
        if p.items.is_empty() {
            return Err(Error::invalid(match p.uf.as_deref() {
                Some(uf) => t_args(
                    "nucleo-importacao-cupom-erro-sem-itens-uf",
                    &args(&[("uf", uf.to_string().into())]),
                ),
                None => t("nucleo-importacao-cupom-erro-sem-itens"),
            }));
        }
        remember_page(&key, &html);
        return Ok((ReceiptOrigin::Nfce, p, Some(check_url(url)?)));
    }
    if let Some(f) = a.files.as_ref().and_then(|f| f.first()) {
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(f.content_base64.trim())
            .map_err(|_| {
                Error::invalid(t_args(
                    "nucleo-importacao-cupom-erro-base64",
                    &args(&[("arquivo", f.filename.clone().into())]),
                ))
            })?;
        if let Some(xml) = std::str::from_utf8(&bytes)
            .ok()
            .filter(|t| nfce::looks_like_xml(t))
        {
            return Ok((ReceiptOrigin::Xml, nfce::parse_xml(xml), None));
        }
        let text = crate::pdf::text(&bytes, None).map_err(|e| {
            Error::invalid(t_args(
                "nucleo-importacao-pdf-erro",
                &args(&[("erro", e.to_string().into())]),
            ))
        })?;
        return Ok((ReceiptOrigin::Pdf, nfce::parse_text(&text), None));
    }
    match a.text.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(t) if t.contains('<') && t.to_ascii_lowercase().contains("txttit") => {
            Ok((ReceiptOrigin::Text, nfce::parse_html(t), None))
        }
        Some(t) => Ok((ReceiptOrigin::Text, nfce::parse_text(t), None)),
        None => Err(Error::invalid(t("nucleo-importacao-cupom-erro-sem-origem"))),
    }
}

/// O lançamento de saída que bate com o valor pago, perto da data (cartão lança depois).
fn txn_match(
    conn: &Connection,
    date: &str,
    paid: i64,
    key: Option<&str>,
) -> Result<Option<TxnMatch>> {
    Ok(conn
        .query_row(
            "SELECT t.id, t.date, t.amount, t.description, a.name, t.member_id FROM txn t
               JOIN account a ON a.id = t.account_id
              WHERE t.kind = 'expense' AND t.amount = ?1 AND t.status <> 'planned'
                AND t.date BETWEEN date(?2, '-2 days') AND date(?2, '+5 days')
                AND NOT EXISTS (SELECT 1 FROM receipt r WHERE r.txn_id = t.id
                                 AND r.access_key IS NOT ?3)
              ORDER BY abs(julianday(t.date) - julianday(?2)) LIMIT 1",
            params![-paid, date, key],
            |r| {
                Ok(TxnMatch {
                    txn_id: r.get(0)?,
                    date: r.get(1)?,
                    amount: r.get(2)?,
                    description: r.get(3)?,
                    account_name: r.get(4)?,
                    member_id: r.get(5)?,
                })
            },
        )
        .optional()?)
}

pub fn preview(conn: &Connection, a: &ReceiptArgs) -> Result<ReceiptPreview> {
    // Os itens informados direto (o agente, de uma foto) valem em qualquer país; a NFC-e, não.
    let brazil = crate::country::is_brazil(conn);
    if a.manual.is_none() {
        ensure_brazil(conn)?;
    }
    let (origin, mut parsed, _) = read(conn, a)?;
    if parsed.date.is_none()
        && let Some(d) = &a.date
    {
        parsed.date = Some(crate::time::fmt_date(parse_date(d)?));
    }
    // A categoria escolhida decide o grupo de imposto do item.
    let chosen = a.categories.as_deref().unwrap_or(&[]);
    let cats: std::collections::HashMap<String, crate::food::Category> =
        crate::food::categories(conn)?
            .into_iter()
            .map(|c| (c.id.clone(), c))
            .collect();
    for c in chosen {
        if !cats.contains_key(&c.category_id) {
            return Err(Error::not_found(t("nucleo-alimentacao-erro-categoria")));
        }
    }
    let groups: Vec<GroupChoice> = chosen
        .iter()
        .filter_map(|c| {
            crate::food::purchase::group_of(&cats, &c.category_id).map(|g| GroupChoice {
                line: c.line,
                group: g,
            })
        })
        .collect();
    let langs = crate::food::profile::family(conn).words;
    let mut items = with_tax(&parsed.items, parsed.tax_total, &groups, &langs);
    if !brazil {
        without_tax(&mut items);
    }
    for it in &mut items {
        match chosen.iter().find(|c| c.line == it.line) {
            Some(c) => it.category_id = Some(c.category_id.clone()),
            None => {
                let g = crate::food::classify::guess(
                    conn,
                    &crate::food::classify::GuessArgs {
                        description: it.description.clone(),
                        store_name: parsed.store_name.clone(),
                        store_tax_id: parsed.store_cnpj.clone(),
                        gtin: it.gtin.clone().or_else(|| it.code.clone()),
                    },
                )?;
                it.category_id = g.category_id;
                it.category_reason = Some(g.reason);
            }
        }
    }
    let printed = parsed.tax_total.is_some_and(|t| t > 0);
    let tax = if printed {
        parsed.tax_total.unwrap_or(0)
    } else {
        items.iter().map(|i| i.tax).sum()
    };
    let duplicate = match &parsed.access_key {
        Some(k) => conn
            .query_row("SELECT id FROM receipt WHERE access_key = ?1", [k], |r| {
                r.get(0)
            })
            .optional()?,
        None => None,
    };
    use crate::food::purchase::Payment;
    let wanted = match &a.payment {
        Some(Payment::Link { txn_id }) => Some(txn_id),
        _ => a.txn_id.as_ref(),
    };
    let txn = match (wanted, &parsed.date) {
        // Sem lançamento, ou um criado junto: não procura um que bata.
        _ if matches!(a.payment, Some(Payment::None | Payment::Create { .. })) => None,
        (Some(id), _) => conn
            .query_row(
                "SELECT t.id, t.date, t.amount, t.description, a.name, t.member_id FROM txn t
                   JOIN account a ON a.id = t.account_id WHERE t.id = ?1",
                [id],
                |r| {
                    Ok(TxnMatch {
                        txn_id: r.get(0)?,
                        date: r.get(1)?,
                        amount: r.get(2)?,
                        description: r.get(3)?,
                        account_name: r.get(4)?,
                        member_id: r.get(5)?,
                    })
                },
            )
            .optional()?,
        // A mesma nota de novo (substituir): o lançamento dela continua valendo.
        (None, Some(d)) => txn_match(conn, d, parsed.paid, parsed.access_key.as_deref())?,
        _ => None,
    };
    let mut warnings = parsed.warnings.clone();
    if duplicate.is_some() {
        warnings.push(t("nucleo-importacao-cupom-aviso-repetido"));
    }
    // Fora do Brasil, o imposto de cada item é o que o cupom imprime (onde o perfil do país
    // diz isso), rateado pelos itens tributáveis.
    let profile = crate::food::profile::family(conn);
    if !brazil
        && profile.item_tax == crate::food::profile::ItemTax::Printed
        && let Some(total) = parsed.tax_total.filter(|t| *t > 0)
    {
        let flags: Vec<Option<bool>> = a
            .manual
            .iter()
            .flat_map(|m| &m.items)
            .filter(|i| !i.description.trim().is_empty() && i.total != 0)
            .map(|i| i.taxable)
            .collect();
        let food: Vec<bool> = items
            .iter()
            .map(|i| {
                i.category_id
                    .as_ref()
                    .and_then(|c| cats.get(c))
                    .is_some_and(|c| c.is_food)
            })
            .collect();
        spread_printed(&mut items, total, &flags, &food);
    }
    let (printed, tax) = if !brazil {
        let t: i64 = items.iter().map(|i| i.tax).sum();
        (t > 0, t)
    } else {
        (printed, tax)
    };
    if !printed && a.manual.is_none() {
        warnings.push(t("nucleo-importacao-cupom-aviso-sem-tributos"));
    }
    Ok(ReceiptPreview {
        origin,
        parsed,
        items,
        tax,
        tax_printed: printed,
        duplicate,
        txn,
        warnings,
    })
}

pub fn import(tx: &Transaction<'_>, a: &ReceiptArgs, today: NaiveDate) -> Result<Receipt> {
    crate::food::seed::ensure(tx)?;
    let p = preview(tx, a)?;
    let date = p
        .parsed
        .date
        .clone()
        .ok_or_else(|| Error::invalid(t("nucleo-importacao-cupom-erro-data")))?;
    if p.items.is_empty() {
        return Err(Error::invalid(t("nucleo-importacao-cupom-erro-vazio")));
    }
    if let Some(old) = &p.duplicate {
        if a.replace != Some(true) {
            return Err(Error::Conflict(t("nucleo-importacao-cupom-erro-repetido")));
        }
        tx.execute("DELETE FROM receipt WHERE id = ?1", [old])?;
    }
    // A URL já foi lida na prévia; guarda a forma em https.
    let url = a
        .url
        .as_deref()
        .filter(|u| !u.trim().is_empty())
        .map(check_url)
        .transpose()?;
    if let Some(m) = &a.member_id {
        crate::household::get_member(tx, m)?;
    }
    let pr = &p.parsed;
    // O lançamento: o escolhido na forma de pagamento (com as mesmas regras da compra
    // digitada) ou, sem ela, o que a prévia achou.
    let (txn_id, txn_created) = match &a.payment {
        Some(pay) => {
            let cats: std::collections::HashMap<String, crate::food::Category> =
                crate::food::categories(tx)?
                    .into_iter()
                    .map(|c| (c.id.clone(), c))
                    .collect();
            let food: i64 = p
                .items
                .iter()
                .filter(|i| {
                    i.category_id
                        .as_ref()
                        .and_then(|c| cats.get(c))
                        .is_some_and(|c| c.is_food)
                })
                .map(|i| i.total)
                .sum();
            let description = pr
                .store_name
                .clone()
                .unwrap_or_else(|| t("nucleo-alimentacao-compra-sem-loja"));
            crate::food::purchase::resolve_payment(
                tx,
                pay,
                &crate::food::purchase::PayCtx {
                    receipt_id: None,
                    own: None,
                    paid: pr.paid,
                    date: &date,
                    description: &description,
                    store: pr.store_name.as_deref(),
                    member_id: a.member_id.as_ref(),
                    food,
                    items_total: pr.items_total,
                },
                today,
            )?
        }
        None => (p.txn.as_ref().map(|t| t.txn_id.clone()), false),
    };
    let member_id = match &a.member_id {
        Some(m) => Some(m.clone()),
        None => match &txn_id {
            Some(t) => {
                tx.query_row("SELECT member_id FROM txn WHERE id = ?1", [t], |r| r.get(0))?
            }
            None => None,
        },
    };
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO receipt (id, access_key, date, issued_at, store_name, store_cnpj, uf,
                              items_total, discount, paid, tax_total, tax_federal, tax_state,
                              tax_municipal, origin, url, txn_id, member_id, created_at, updated_at,
                              txn_created)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?19,
                 ?20)",
        params![
            id,
            pr.access_key,
            date,
            pr.issued_at,
            pr.store_name,
            pr.store_cnpj,
            pr.uf,
            pr.items_total,
            pr.discount,
            pr.paid,
            pr.tax_total.filter(|t| *t > 0),
            pr.tax_federal,
            pr.tax_state,
            pr.tax_municipal,
            p.origin.as_str(),
            url,
            txn_id,
            member_id,
            now,
            txn_created
        ],
    )?;
    let mut stmt = tx.prepare(
        "INSERT INTO receipt_item (id, receipt_id, line, code, description, quantity, unit,
                                   unit_price, total, item_group, group_origin, ibpt_product,
                                   tax_rate, tax, tax_origin, gtin, discount, created_at,
                                   updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18,
                 ?18)",
    )?;
    for it in &p.items {
        stmt.execute(params![
            new_id(),
            id,
            it.line,
            it.code,
            it.description,
            it.quantity,
            it.unit,
            it.unit_price,
            it.total,
            it.group.key(),
            it.group_origin,
            Option::<String>::None,
            it.tax_rate,
            it.tax,
            origin_to_db(&it.tax_origin),
            it.gtin,
            it.discount,
            now
        ])?;
    }
    drop(stmt);
    // O pagamento do cupom (o caso comum: um lançamento só) e os produtos dos itens.
    if let Some(txn) = &txn_id {
        tx.execute(
            "INSERT INTO receipt_payment (id, receipt_id, txn_id, amount, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT (receipt_id, txn_id) DO NOTHING",
            params![new_id(), id, txn, pr.paid, now],
        )?;
    }
    crate::food::link_items(tx, Some(&id))?;
    set_categories(tx, &id, a.categories.as_deref().unwrap_or(&[]))?;
    get(tx, &id)
}

/// A categoria escolhida vale para o produto do item; o grupo de imposto do item acompanha.
fn set_categories(tx: &Transaction<'_>, id: &str, chosen: &[CategoryChoice]) -> Result<()> {
    if chosen.is_empty() {
        return Ok(());
    }
    let cats: std::collections::HashMap<String, crate::food::Category> =
        crate::food::categories(tx)?
            .into_iter()
            .map(|c| (c.id.clone(), c))
            .collect();
    let now = now_utc();
    for c in chosen {
        if !cats.contains_key(&c.category_id) {
            return Err(Error::not_found(t("nucleo-alimentacao-erro-categoria")));
        }
        tx.execute(
            "UPDATE product SET category_id = ?3, category_origin = 'manual', updated_at = ?4
              WHERE id = (SELECT product_id FROM receipt_item WHERE receipt_id = ?1 AND line = ?2)
                AND category_id IS NOT ?3",
            params![id, c.line, c.category_id, now],
        )?;
        if let Some(g) = crate::food::purchase::group_of(&cats, &c.category_id) {
            regroup(tx, id, c.line, g)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct ReceiptUpdate {
    pub id: Id,
    /// Ausente: não muda. `null`: da família.
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional, type = "string | null")]
    pub member_id: Option<Option<Id>>,
    #[serde(default)]
    #[ts(optional)]
    pub categories: Option<Vec<CategoryChoice>>,
    /// Ausente: o lançamento não muda.
    #[serde(default)]
    #[ts(optional)]
    pub payment: Option<crate::food::purchase::Payment>,
}

/// Ajusta um cupom já gravado sem tocar no que o documento diz (itens, valores, loja, data):
/// a categoria de cada item, o membro e o lançamento.
pub fn update(tx: &Transaction<'_>, a: &ReceiptUpdate, today: NaiveDate) -> Result<Receipt> {
    let r = get(tx, &a.id)?;
    let now = now_utc();
    if let Some(m) = &a.member_id {
        if let Some(m) = m {
            crate::household::get_member(tx, m)?;
        }
        tx.execute(
            "UPDATE receipt SET member_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![a.id, m, now],
        )?;
    }
    set_categories(tx, &a.id, a.categories.as_deref().unwrap_or(&[]))?;
    if let Some(pay) = &a.payment {
        let own = r.txn_id.clone().filter(|_| r.txn_created);
        let description = r
            .store_name
            .clone()
            .unwrap_or_else(|| t("nucleo-alimentacao-compra-sem-loja"));
        let food: i64 = tx.query_row(
            "SELECT coalesce(sum(i.total), 0) FROM receipt_item i
               JOIN product p ON p.id = i.product_id
               JOIN product_category c ON c.id = p.category_id
              WHERE i.receipt_id = ?1 AND c.is_food",
            [&a.id],
            |x| x.get(0),
        )?;
        let member = match &a.member_id {
            Some(m) => m.clone(),
            None => r.member_id.clone(),
        };
        let (txn_id, created) = crate::food::purchase::resolve_payment(
            tx,
            pay,
            &crate::food::purchase::PayCtx {
                receipt_id: Some(&a.id),
                own: own.as_ref(),
                paid: r.paid,
                date: &r.date,
                description: &description,
                store: r.store_name.as_deref(),
                member_id: member.as_ref(),
                food,
                items_total: r.items_total,
            },
            today,
        )?;
        tx.execute(
            "UPDATE receipt SET txn_id = ?2, txn_created = ?3, updated_at = ?4 WHERE id = ?1",
            params![a.id, txn_id, created, now],
        )?;
        tx.execute("DELETE FROM receipt_payment WHERE receipt_id = ?1", [&a.id])?;
        if let Some(t) = &txn_id {
            tx.execute(
                "INSERT INTO receipt_payment (id, receipt_id, txn_id, amount, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![new_id(), a.id, t, r.paid, now],
            )?;
        }
        // O lançamento criado com o cupom e que ele deixou de usar sai junto.
        if let Some(o) = own.filter(|o| txn_id.as_ref() != Some(o)) {
            crate::transactions::delete(tx, &[o], today)?;
        }
    }
    get(tx, &a.id)
}

const COLS: &str = "r.id, r.access_key, r.date, r.issued_at, r.store_name, r.store_cnpj, r.uf,
    r.items_total, r.discount, r.paid, r.tax_total, r.origin, r.txn_id, r.member_id, m.name,
    (SELECT count(*) FROM receipt_item i WHERE i.receipt_id = r.id),
    (SELECT coalesce(sum(i.tax), 0) FROM receipt_item i WHERE i.receipt_id = r.id),
    r.txn_created";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Receipt> {
    let printed: Option<i64> = r.get(10)?;
    let items_tax: i64 = r.get(16)?;
    Ok(Receipt {
        id: r.get(0)?,
        access_key: r.get(1)?,
        date: r.get(2)?,
        issued_at: r.get(3)?,
        store_name: r.get(4)?,
        store_cnpj: r.get(5)?,
        uf: r.get(6)?,
        items_total: r.get(7)?,
        discount: r.get(8)?,
        paid: r.get(9)?,
        tax: printed.unwrap_or(items_tax),
        tax_printed: printed.is_some(),
        origin: ReceiptOrigin::parse(&r.get::<_, String>(11)?),
        txn_id: r.get(12)?,
        txn_created: r.get(17)?,
        tax_added: {
            let (items, discount, paid): (i64, i64, i64) = (r.get(7)?, r.get(8)?, r.get(9)?);
            (paid - (items - discount)).max(0)
        },
        member_id: r.get(13)?,
        member_name: r.get(14)?,
        item_count: r.get(15)?,
        items: vec![],
    })
}

fn items_of(conn: &Connection, id: &str) -> Result<Vec<ReceiptItem>> {
    let mut stmt = conn.prepare(
        "SELECT line, code, description, quantity, unit, unit_price, total, item_group,
                group_origin, ibpt_product, tax_rate, tax, tax_origin, gtin, product_id,
                base_unit, base_price, discount,
                (SELECT p.category_id FROM product p WHERE p.id = receipt_item.product_id)
           FROM receipt_item WHERE receipt_id = ?1 ORDER BY line",
    )?;
    let rows = stmt
        .query_map([id], |r| {
            let group = ItemGroup::parse(&r.get::<_, String>(7)?);
            Ok(ReceiptItem {
                line: r.get(0)?,
                code: r.get(1)?,
                description: r.get(2)?,
                quantity: r.get(3)?,
                unit: r.get(4)?,
                unit_price: r.get(5)?,
                total: r.get(6)?,
                group,
                group_label: group.label(),
                group_origin: r.get(8)?,
                tax_rate: r.get::<_, Option<f64>>(10)?.unwrap_or(0.0),
                tax: r.get(11)?,
                tax_origin: origin_from_db(r.get(12)?),
                // Guardado como GTIN-14 (V138); mostrado como EAN-13.
                gtin: r
                    .get::<_, Option<String>>(13)?
                    .map(|g| crate::food::gtin::display(&g)),
                product_id: r.get(14)?,
                base_unit: r.get(15)?,
                base_price: r.get(16)?,
                discount: r.get(17)?,
                category_id: r.get(18)?,
                category_reason: None,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: &str) -> Result<Receipt> {
    let mut r = conn
        .query_row(
            &format!("SELECT {COLS} FROM receipt r LEFT JOIN member m ON m.id = r.member_id WHERE r.id = ?1"),
            [id],
            row,
        )
        .optional()?
        .ok_or_else(|| Error::not_found(receipt_ref(id)))?;
    r.items = items_of(conn, id)?;
    Ok(r)
}

fn period(a_from: Option<&str>, a_to: Option<&str>) -> Result<(String, String)> {
    let today = crate::time::today_local();
    let to = match a_to {
        Some(t) => parse_date(t)?,
        None => today,
    };
    let from = match a_from {
        Some(f) => parse_date(f)?,
        None => NaiveDate::from_ymd_opt(chrono::Datelike::year(&to), 1, 1).expect("data"),
    };
    Ok((crate::time::fmt_date(from), crate::time::fmt_date(to)))
}

pub fn list(conn: &Connection, a: &ReceiptListArgs) -> Result<Vec<Receipt>> {
    let (from, to) = period(a.from.as_deref(), a.to.as_deref())?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM receipt r LEFT JOIN member m ON m.id = r.member_id
          WHERE r.date BETWEEN ?1 AND ?2 AND (?3 IS NULL OR r.member_id = ?3)
          ORDER BY r.date DESC, r.created_at DESC LIMIT ?4"
    ))?;
    let rows = stmt
        .query_map(
            params![from, to, a.member_id, a.limit.unwrap_or(200).min(1000)],
            row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Apaga o cupom; o lançamento criado junto com ele sai também.
pub fn delete(tx: &Transaction<'_>, id: &str, today: NaiveDate) -> Result<()> {
    let own: Option<Id> = tx
        .query_row(
            "SELECT CASE WHEN txn_created THEN txn_id END FROM receipt WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(receipt_ref(id)))?;
    tx.execute("DELETE FROM receipt WHERE id = ?1", [id])?;
    if let Some(t) = own {
        crate::transactions::delete(tx, &[t], today)?;
    }
    Ok(())
}

/// Troca o grupo de imposto de um item. O rateio do imposto impresso fica (é a nota); o
/// estimado é refeito pela mediana do grupo novo.
fn regroup(tx: &Transaction<'_>, receipt_id: &str, line: u32, group: ItemGroup) -> Result<()> {
    let Some((total, origin, old)) = tx
        .query_row(
            "SELECT total, tax_origin, item_group FROM receipt_item
              WHERE receipt_id = ?1 AND line = ?2",
            params![receipt_id, line],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?
    else {
        return Ok(());
    };
    if old == group.key() {
        return Ok(());
    }
    let now = now_utc();
    if origin == ESTIMATED_DB && crate::country::is_brazil(tx) {
        let rate = group_rate(group.tax_group());
        tx.execute(
            "UPDATE receipt_item SET item_group = ?3, group_origin = 'manual', ibpt_product = NULL,
                    tax_rate = ?4, tax = ?5, updated_at = ?6
              WHERE receipt_id = ?1 AND line = ?2",
            params![
                receipt_id,
                line,
                group.key(),
                rate,
                (total as f64 * rate / 100.0).round() as i64,
                now
            ],
        )?;
    } else {
        tx.execute(
            "UPDATE receipt_item SET item_group = ?3, group_origin = 'manual', updated_at = ?4
              WHERE receipt_id = ?1 AND line = ?2",
            params![receipt_id, line, group.key(), now],
        )?;
    }
    Ok(())
}

/// O que a família compra (e cada membro), por segmento de consumo, com o imposto, e os itens
/// que mais pesam.
pub fn summary(conn: &Connection, a: &ReceiptListArgs) -> Result<ReceiptSummary> {
    let (from, to) = period(a.from.as_deref(), a.to.as_deref())?;
    let cats: std::collections::HashMap<String, crate::food::Category> =
        crate::food::categories(conn)?
            .into_iter()
            .map(|c| (c.id.clone(), c))
            .collect();
    let mut stmt = conn.prepare(
        "SELECT r.member_id, m.name, p.category_id, i.description, i.total, i.tax, i.tax_origin
           FROM receipt_item i JOIN receipt r ON r.id = i.receipt_id
           LEFT JOIN member m ON m.id = r.member_id
           LEFT JOIN product p ON p.id = i.product_id
          WHERE r.date BETWEEN ?1 AND ?2 AND (?3 IS NULL OR r.member_id = ?3)",
    )?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        Option<Id>,
        Option<String>,
        Option<String>,
        String,
        i64,
        i64,
        String,
    )> = stmt
        .query_map(params![from, to, a.member_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let (receipts, spent): (u32, i64) = conn.query_row(
        "SELECT count(*), coalesce(sum(paid), 0) FROM receipt
          WHERE date BETWEEN ?1 AND ?2 AND (?3 IS NULL OR member_id = ?3)",
        params![from, to, a.member_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    type Segments = BTreeMap<Option<Id>, GroupSpend>;
    let other = t("nucleo-importacao-cupom-grupo-outros");
    let label = |g: &Option<Id>| {
        g.as_ref()
            .and_then(|id| cats.get(id))
            .map(|c| c.name.clone())
            .unwrap_or_else(|| other.clone())
    };
    let add = |m: &mut Segments, g: &Option<Id>, total: i64, tax: i64| {
        let e = m.entry(g.clone()).or_insert_with(|| GroupSpend {
            segment_id: g.clone(),
            label: label(g),
            spent: 0,
            tax: 0,
            items: 0,
        });
        e.spent += total;
        e.tax += tax;
        e.items += 1;
    };
    let mut groups: Segments = BTreeMap::new();
    let mut members: BTreeMap<Option<Id>, (String, Segments)> = BTreeMap::new();
    let mut top: BTreeMap<String, TopItem> = BTreeMap::new();
    let (mut tax, mut tax_printed) = (0, 0);
    for (mid, mname, g, desc, total, t, origin) in rows {
        let g = g
            .filter(|c| cats.contains_key(c))
            .map(|c| crate::food::root_of(&cats, &c));
        add(&mut groups, &g, total, t);
        let entry = members.entry(mid.clone()).or_insert_with(|| {
            (
                mname
                    .clone()
                    .unwrap_or_else(|| crate::i18n::t("nucleo-importacao-cupom-familia")),
                BTreeMap::new(),
            )
        });
        add(&mut entry.1, &g, total, t);
        tax += t;
        if origin == "nfce" {
            tax_printed += t;
        }
        let key = crate::transactions::normalize_description(&desc);
        let e = top.entry(key).or_insert_with(|| TopItem {
            description: desc.clone(),
            segment: label(&g),
            spent: 0,
            times: 0,
        });
        e.spent += total;
        e.times += 1;
    }
    let sorted = |m: Segments| {
        let mut v: Vec<GroupSpend> = m.into_values().collect();
        v.sort_by_key(|g| std::cmp::Reverse(g.spent));
        v
    };
    let mut members: Vec<MemberSpend> = members
        .into_iter()
        .map(|(id, (name, g))| {
            let groups = sorted(g);
            MemberSpend {
                member_id: id,
                member_name: name,
                spent: groups.iter().map(|g| g.spent).sum(),
                tax: groups.iter().map(|g| g.tax).sum(),
                groups,
            }
        })
        .collect();
    members.sort_by_key(|m| std::cmp::Reverse(m.spent));
    let mut top_items: Vec<TopItem> = top.into_values().collect();
    top_items.sort_by_key(|t| std::cmp::Reverse(t.spent));
    top_items.truncate(15);
    Ok(ReceiptSummary {
        from,
        to,
        receipts,
        spent,
        tax,
        tax_printed,
        groups: sorted(groups),
        members,
        top_items,
    })
}

/// Para a estimativa de impostos (docs/taxes.md): os cupons ligados a lançamentos do período,
/// por categoria do lançamento: (categoria, valor pago, imposto do cupom).
pub fn linked_in(
    conn: &Connection,
    from: &str,
    to: &str,
    printed_only: bool,
) -> Result<Vec<(Option<Id>, i64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT t.category_id, sum(r.paid),
                sum(coalesce(r.tax_total,
                    (SELECT coalesce(sum(i.tax), 0) FROM receipt_item i WHERE i.receipt_id = r.id)))
           FROM receipt r JOIN txn t ON t.id = r.txn_id
          WHERE t.date BETWEEN ?1 AND ?2 AND t.kind = 'expense' AND t.status <> 'planned'
            AND (NOT ?3 OR r.tax_total IS NOT NULL)
          GROUP BY t.category_id",
    )?;
    let rows = stmt
        .query_map(params![from, to, printed_only], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

#[derive(Deserialize)]
struct IdArg {
    id: Id,
}

pub const READ_ONLY: &[&str] = &[
    "preview",
    "foreign_qr",
    "list",
    "get",
    "summary",
    "read_text",
];

pub fn handle(
    ctx: &mut crate::api::Ctx<'_>,
    action: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    use crate::api::{parse_args, to_value};
    match action {
        "preview" => to_value(preview(ctx.conn(), &parse_args::<ReceiptArgs>(args)?)?),
        // Nota de fora do Brasil pelo QR (Argentina, AFIP), em qualquer país da família.
        "foreign_qr" => {
            #[derive(serde::Deserialize)]
            struct A {
                url: String,
            }
            let a: A = parse_args(args)?;
            to_value(foreign::read(ctx.conn(), &a.url)?)
        }
        "import" => {
            let a: ReceiptArgs = parse_args(args)?;
            if a.manual.is_none() {
                ensure_brazil(ctx.conn())?;
            }
            to_value(
                ctx.write(&t("nucleo-importacao-cupom-historico-importado"), |tx| {
                    import(tx, &a, crate::time::today_local())
                })?
                .value,
            )
        }
        // Itens colados como texto (um por linha), em qualquer país.
        "read_text" => {
            #[derive(serde::Deserialize)]
            struct A {
                text: String,
            }
            let a: A = parse_args(args)?;
            to_value(ocr::read_text(ctx.conn(), &a.text)?)
        }
        "list" => to_value(list(ctx.conn(), &parse_args::<ReceiptListArgs>(args)?)?),
        "get" => to_value(get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "summary" => to_value(summary(ctx.conn(), &parse_args::<ReceiptListArgs>(args)?)?),
        "update" => {
            let a: ReceiptUpdate = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-importacao-cupom-historico-grupo"), |tx| {
                    update(tx, &a, crate::time::today_local())
                })?
                .value,
            )
        }
        "delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-importacao-cupom-historico-apagado"), |tx| {
                delete(tx, &a.id, crate::time::today_local())
            })?;
            Ok(serde_json::Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("receipts.{action}"))),
    }
}

#[cfg(test)]
mod tests;
