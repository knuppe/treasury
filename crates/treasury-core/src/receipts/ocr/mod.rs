//! Leitura de cupons por foto, sem QR (docs/ocr-receipts.md): o OCR local dá as linhas com a
//! posição, o leitor de linhas monta os itens e confere as contas, e a IA só entra no que não
//! fechou. Vale em qualquer país; o resultado vira uma compra informada (`ManualReceipt`).

pub mod lines;
#[cfg(feature = "ocr-paddle")]
pub mod paddle;

use std::sync::{Arc, LazyLock};

use base64::Engine as _;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use self::lines::{LinesReading, Tier};
use super::{ManualItem, ManualReceipt};
use crate::ai::agent::{self, AgentRequest};
use crate::ai::jobs::{JobKind, JobStatus};
use crate::ai::routing::{self, AiTask};
use crate::api::Api;
use crate::error::{Error, Result};
use crate::i18n::t;

/// Uma linha (ou trecho de linha) lida pelo OCR. Coordenadas normalizadas de 0 a 1, com a
/// origem embaixo à esquerda, como no Apple Vision.
#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct OcrLine {
    pub text: String,
    /// Confiança do OCR, de 0 a 1.
    pub conf: f32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Cantos de cima (esquerdo e direito), para endireitar foto torta.
    #[serde(default)]
    #[ts(optional)]
    pub tl: Option<[f32; 2]>,
    #[serde(default)]
    #[ts(optional)]
    pub tr: Option<[f32; 2]>,
}

/// Motor que leu o cupom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum OcrEngine {
    /// Apple Vision (no Mac, pelo app).
    Vision,
    /// PaddleOCR embutido no app (Rust puro).
    Paddle,
    /// Um motor de IA de Ajustes › IA, com a foto.
    Ai,
    /// Texto colado pela pessoa (lista de itens, recibo que chegou por e-mail): sem OCR.
    Text,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct PhotoArgs {
    /// Linhas já lidas no app (Apple Vision, no Mac).
    #[serde(default)]
    #[ts(optional)]
    pub lines: Option<Vec<OcrLine>>,
    /// Largura e altura da foto (para endireitar as linhas do Vision).
    #[serde(default)]
    #[ts(optional)]
    pub width: Option<f32>,
    #[serde(default)]
    #[ts(optional)]
    pub height: Option<f32>,
    /// A foto (para o PaddleOCR e para a IA).
    #[serde(default)]
    #[ts(optional)]
    pub image_base64: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub filename: Option<String>,
    /// Pedir à IA quando a conta não fechar (a foto sai do computador se o motor for remoto).
    #[serde(default)]
    #[ts(optional)]
    pub use_ai: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhotoReading {
    pub engine: OcrEngine,
    pub tier: Tier,
    pub reading: LinesReading,
    /// A compra pronta para a prévia (`receipts.preview` com `manual`).
    pub receipt: ManualReceipt,
    /// Há motor de IA que lê imagens (para oferecer a ajuda quando a conta não fecha).
    pub ai_available: bool,
    pub warnings: Vec<String>,
}

static DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{1,2})[/.-](\d{1,2})[/.-](\d{4}|\d{2})\b|\b(\d{4})-(\d{2})-(\d{2})\b")
        .unwrap()
});

/// Data do cupom: a primeira data válida, dia antes do mês (EUA e Canadá: mês antes, quando
/// o dia não cabe como mês).
fn find_date(rows: &[String], country: &str) -> Option<String> {
    for r in rows {
        for c in DATE.captures_iter(r) {
            let ymd = if let Some(y) = c.get(4) {
                (
                    y.as_str().parse().ok()?,
                    c[5].parse().ok()?,
                    c[6].parse().ok()?,
                )
            } else {
                let (a, b): (u32, u32) = (c[1].parse().ok()?, c[2].parse().ok()?);
                let y: i32 = match c[3].len() {
                    2 => 2000 + c[3].parse::<i32>().ok()?,
                    _ => c[3].parse().ok()?,
                };
                let us = matches!(country, "US" | "CA") && a <= 12 && b > 12;
                if us { (y, a, b) } else { (y, b, a) }
            };
            if let Some(d) = chrono::NaiveDate::from_ymd_opt(ymd.0, ymd.1, ymd.2)
                && (2000..=2100).contains(&ymd.0)
            {
                return Some(crate::time::fmt_date(d));
            }
        }
    }
    None
}

/// Loja: a primeira linha do topo com cara de nome (letras, sem CNPJ, endereço ou valor).
fn find_store(rows: &[String]) -> Option<String> {
    rows.iter()
        .take(6)
        .map(|r| r.trim())
        .find(|r| {
            let f = crate::import::text::fold(r);
            r.chars().filter(|c| c.is_alphabetic()).count() >= 4
                && !r.chars().any(|c| c.is_ascii_digit())
                && ![
                    "cnpj",
                    "danfe",
                    "documento",
                    "cupom",
                    "nota fiscal",
                    "rua",
                    "avenida",
                    "tel",
                ]
                .iter()
                .any(|k| f.contains(k))
        })
        .map(str::to_string)
}

fn to_manual(r: &LinesReading, rows: &[String], country: &str) -> ManualReceipt {
    ManualReceipt {
        date: find_date(rows, country)
            .unwrap_or_else(|| crate::time::fmt_date(crate::time::today_local())),
        store_name: find_store(rows),
        items: r
            .items
            .iter()
            .map(|i| ManualItem {
                description: i.description.clone(),
                quantity: i.quantity.clone(),
                unit: None,
                unit_price: i.unit_price,
                total: i.total,
                taxable: i.taxable,
            })
            .collect(),
        discount: (r.discount > 0).then_some(r.discount),
        paid: r.total,
        tax_total: r.tax,
        from_photo: Some(true),
    }
}

fn image_bytes(a: &PhotoArgs) -> Result<Option<Vec<u8>>> {
    a.image_base64
        .as_deref()
        .map(|b| {
            base64::engine::general_purpose::STANDARD
                .decode(b.trim())
                .map_err(|_| Error::invalid(t("nucleo-ocr-erro-foto")))
        })
        .transpose()
}

/// Há motor de IA (rota "Leitura de PDF" ou a do chat) que lê imagens.
fn ai_available(api: &Api) -> bool {
    api.with_session(|s, _| {
        Ok(
            routing::candidates(s.db.conn(), s.db.key(), AiTask::Pdf, None, false)?
                .iter()
                .any(|e| e.caps.images),
        )
    })
    .unwrap_or(false)
}

/// A IA lê a foto e devolve o cupom em JSON; o resultado passa pela mesma conferência.
fn read_with_ai(
    api: &Api,
    bytes: Vec<u8>,
    filename: &str,
) -> Result<(LinesReading, ManualReceipt)> {
    let att = Arc::new(crate::ai::attachments::prepare(filename, None, bytes)?);
    let job = api
        .ai_runtime()
        .new_job(JobKind::Routine, AiTask::Pdf, None);
    job.set_title(&t("nucleo-ocr-ia-titulo"));
    let req = AgentRequest {
        task: AiTask::Pdf,
        message: t("nucleo-ocr-ia-pedido"),
        history: vec![],
        engine_id: None,
        strict_route: false,
        tools: false,
        context: None,
        notes: None,
        computer_access: false,
        attachments: vec![att],
        skip_unsupported: false,
        effort: Default::default(),
        member_id: None,
        client: None,
        only_tools: None,
    };
    let out = agent::run(api, &job, &req);
    job.finish(
        if out.is_ok() {
            JobStatus::Done
        } else {
            JobStatus::Failed
        },
        None,
        None,
    );
    let reply = out.map_err(|e| Error::invalid(e.message))?;
    from_ai_json(&reply.text).ok_or_else(|| Error::invalid(t("nucleo-ocr-erro-ia-resposta")))
}

#[derive(Deserialize)]
struct AiItem {
    description: String,
    #[serde(default)]
    quantity: Option<serde_json::Value>,
    #[serde(default)]
    unit_price: Option<f64>,
    total: f64,
}

#[derive(Deserialize)]
struct AiReceipt {
    #[serde(default)]
    store_name: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    items: Vec<AiItem>,
    #[serde(default)]
    discount: Option<f64>,
    #[serde(default)]
    total: Option<f64>,
    #[serde(default)]
    item_count: Option<u32>,
}

fn money(v: f64) -> i64 {
    (v * 100.0).round() as i64
}

/// O JSON da IA (pode vir cercado de texto ou de crases) virando a mesma leitura do OCR.
pub(crate) fn from_ai_json(text: &str) -> Option<(LinesReading, ManualReceipt)> {
    let (a, b) = (text.find('{')?, text.rfind('}')?);
    let r: AiReceipt = serde_json::from_str(text.get(a..=b)?).ok()?;
    let items: Vec<lines::LineItem> = r
        .items
        .into_iter()
        .filter(|i| !i.description.trim().is_empty())
        .map(|i| lines::LineItem {
            description: i.description.trim().to_string(),
            quantity: i.quantity.map(|q| match q {
                serde_json::Value::String(s) => s.replace(',', "."),
                other => other.to_string(),
            }),
            unit_price: i.unit_price.map(money),
            total: money(i.total),
            line_checked: None,
            taxable: None,
        })
        .collect();
    let discount = r.discount.map(|d| money(d.abs())).unwrap_or(0)
        + items
            .iter()
            .filter(|i| i.total < 0)
            .map(|i| -i.total)
            .sum::<i64>();
    let items: Vec<_> = items.into_iter().filter(|i| i.total >= 0).collect();
    let sum: i64 = items.iter().map(|i| i.total).sum();
    let total = r.total.map(money);
    let balanced = total.is_some_and(|t| (sum - discount - t).abs() <= 2);
    let tier = match (balanced, r.item_count) {
        (true, Some(n)) if n as usize == items.len() => Tier::Checked,
        (true, _) => Tier::Balanced,
        _ => Tier::Unbalanced,
    };
    let date = r
        .date
        .as_deref()
        .and_then(|d| crate::time::parse_date(d).ok())
        .map(crate::time::fmt_date);
    let reading = LinesReading {
        items,
        discount,
        total,
        items_sum: sum,
        tier,
        printed_count: r.item_count,
        tax: None,
        subtotal: None,
        rows: vec![],
    };
    let mut receipt = to_manual(&reading, &[], "");
    if let Some(d) = date {
        receipt.date = d;
    }
    receipt.store_name = r.store_name.filter(|s| !s.trim().is_empty());
    Some((reading, receipt))
}

/// Lê o cupom da foto: as linhas do Vision (no Mac) ou o PaddleOCR embutido; o PaddleOCR
/// também lê de novo quando o Vision não confere; a IA só se a pessoa pediu e a conta não
/// fechou. Roda fora da trava do banco (o OCR e a IA levam tempo).
pub fn read_photo(api: &Api, a: &PhotoArgs) -> Result<PhotoReading> {
    let country = api.with_session(|s, _| Ok(crate::country::family_country(s.db.conn())))?;
    let bytes = image_bytes(a)?;
    let aspect = match (a.width, a.height) {
        (Some(w), Some(h)) if h > 0.0 => w / h,
        _ => 1.0,
    };
    let mut warnings = vec![];
    let mut best: Option<(OcrEngine, LinesReading)> = a
        .lines
        .as_ref()
        .filter(|l| !l.is_empty())
        .map(|l| (OcrEngine::Vision, lines::read(l, &country, aspect)));
    #[cfg(feature = "ocr-paddle")]
    if best.as_ref().is_none_or(|(_, r)| r.tier < Tier::Checked)
        && let Some(b) = &bytes
    {
        match paddle::recognize(b) {
            Ok(l) => {
                let r = lines::read(&l, &country, 1.0);
                if best.as_ref().is_none_or(|(_, cur)| r.tier > cur.tier) {
                    best = Some((OcrEngine::Paddle, r));
                }
            }
            Err(e) if best.is_none() => return Err(e),
            Err(_) => {}
        }
    }
    let Some((mut engine, mut reading)) = best else {
        return Err(Error::invalid(t("nucleo-ocr-erro-sem-motor")));
    };
    let mut receipt = to_manual(&reading, &reading.rows, &country);
    let ai_ok = ai_available(api);
    if reading.tier == Tier::Unbalanced && a.use_ai == Some(true) {
        match (&bytes, ai_ok) {
            (Some(b), true) => {
                let name = a.filename.clone().unwrap_or_else(|| "cupom.jpg".into());
                match read_with_ai(api, b.clone(), &name) {
                    Ok((r, mut m)) if r.tier >= reading.tier => {
                        if m.store_name.is_none() {
                            m.store_name = receipt.store_name.take();
                        }
                        (engine, reading, receipt) = (OcrEngine::Ai, r, m);
                    }
                    Ok(_) => {}
                    Err(e) => warnings.push(e.message()),
                }
            }
            _ => warnings.push(t("nucleo-ocr-aviso-sem-ia")),
        }
    }
    // A faixa já diz se a conta fechou; o aviso fica para a foto sem nenhum item.
    if reading.items.is_empty() {
        warnings.push(t("nucleo-ocr-aviso-sem-itens"));
    }
    Ok(PhotoReading {
        engine,
        tier: reading.tier,
        reading,
        receipt,
        ai_available: ai_ok,
        warnings,
    })
}

/// Lê os itens de um texto colado (um item por linha: "Arroz 5kg 24,90", "Leite 2 x 5,49"),
/// com o mesmo leitor de linhas e a mesma conferência da foto. Sem linha de total, a soma dos
/// itens vale como total.
pub fn read_text(conn: &rusqlite::Connection, text: &str) -> Result<PhotoReading> {
    let country = crate::country::family_country(conn);
    let rows: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(400)
        .collect();
    if rows.is_empty() {
        return Err(Error::invalid(t("nucleo-ocr-aviso-sem-itens")));
    }
    let n = rows.len() as f32;
    let lines: Vec<OcrLine> = rows
        .iter()
        .enumerate()
        .map(|(i, l)| OcrLine {
            text: l.chars().take(200).collect(),
            conf: 1.0,
            x: 0.0,
            // As caixas do OCR contam de baixo para cima.
            y: 1.0 - (i as f32 + 1.0) / n,
            w: 1.0,
            h: 0.8 / n,
            tl: None,
            tr: None,
        })
        .collect();
    let reading = lines::read(&lines, &country, 1.0);
    let mut receipt = to_manual(&reading, &reading.rows, &country);
    receipt.from_photo = None;
    let mut warnings = vec![];
    if reading.items.is_empty() {
        warnings.push(t("nucleo-ocr-aviso-sem-itens"));
    }
    Ok(PhotoReading {
        engine: OcrEngine::Text,
        tier: reading.tier,
        reading,
        receipt,
        ai_available: false,
        warnings,
    })
}

#[cfg(test)]
mod tests;
