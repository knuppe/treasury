//! Leitor de linhas (docs/ocr-receipts.md §6): as caixas do OCR viram linhas do cupom, as
//! linhas viram itens, desconto e total, e as contas dizem o quanto dá para confiar.
//!
//! - **Conferido**: itens − descontos = total, e a quantidade de itens impressa ou o pagamento
//!   batem também. No banco de testes, 100% de total certo.
//! - **Fechou**: só a soma bate (revisão rápida).
//! - **Não fechou**: vai para a IA, se a pessoa deixou, ou para a revisão.
//!
//! O OCR troca vírgula por ponto, `0` por `O`, `1` por `l`, e cola a letra da situação
//! tributária no valor (`1,49G`); o leitor aceita tudo isso. Foto torta é endireitada pela
//! inclinação mediana das linhas.

use std::sync::LazyLock;

use regex::Regex;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::OcrLine;
use crate::import::text::fold_keep_punct;

/// Países que escrevem o decimal com vírgula (o OCR erra, então o ponto também vale).
const COMMA: &[&str] = &["BR", "PT", "ES", "FR", "AR", "DE"];

/// Palavras de total, da mais forte para a mais fraca.
const TOTAL_KW: &[&str] = &[
    "valor a pagar",
    "valor total",
    "total a pagar",
    "total r$",
    "total pago",
    "valor pago",
    "a pagar",
    "total eur",
    "total €",
    "importe total",
    "total compra",
    "total venta",
    "total ttc",
    "net a payer",
    "montant",
    "summe",
    "zu zahlen",
    "gesamt",
    "balance due",
    "amount due",
    "total due",
    "grand total",
    "total",
    "otal r$",
    "otal",
];
/// A partir daqui, palavra de total fraca: só vale com o valor na mesma linha.
const STRONG_TOTALS: usize = 21;

const NOT_TOTAL: &[&str] = &[
    "subtotal",
    "sub total",
    "sub-total",
    "total de itens",
    "qtd",
    "total items",
    "items",
    "artigos",
    "articulos",
    "tributos",
    "impostos",
    "iva",
    "tax",
    "poupan",
    "ahorro",
    "saving",
    "desconto",
    "descuento",
    "troco",
    "cambio",
    "vuelto",
    "change",
    "pontos",
    "points",
    "puntos",
];
const PAY_KW: &[&str] = &[
    "dinheiro",
    "cartao",
    "debito",
    "credito",
    "tef",
    "pix",
    "visa",
    "master",
    "cash",
    "card",
    "efectivo",
    "tarjeta",
    "multibanco",
    "numerario",
    "contactless",
];
const CHANGE_KW: &[&str] = &["troco", "change", "cambio", "vuelto"];
const DISC_KW: &[&str] = &[
    "desconto",
    "desc.",
    "descuento",
    "dto",
    "promo",
    "ahorro",
    "poupanca",
    "saving",
    "you saved",
    "remise",
    "rabatt",
    "coupon",
    "cupom",
    "cupon",
    "oferta",
];
const SUBTOTAL_KW: &[&str] = &["subtotal", "sub total", "sub-total", "subtotai"];
/// Linhas que não são item (cabeçalho, rodapé, pagamento, impostos).
const SKIP_KW: &[&str] = &[
    "cnpj",
    "cpf",
    "nif",
    "cuit:",
    "c.u.i.t",
    "rut",
    "ie:",
    "im:",
    "rua ",
    "av.",
    "avenida",
    "tel",
    "fone",
    "data",
    "hora",
    "caixa",
    "operador",
    "cupom fiscal",
    "nfc-e",
    "danfe",
    "chave",
    "protocolo",
    "consulte",
    "www",
    "http",
    "iva incl",
    "base",
    "vat",
    "tax",
    "tributos",
    "lei 12.741",
    "qtd. total",
    "qtde total",
    "quantidade total",
    "codigo",
    "descri",
    "vl unit",
    "vl item",
    "vl total",
    "valor unit",
    "item cod",
    "#",
    "pontos",
    "points",
    "puntos",
    "obrigado",
    "gracias",
    "thank",
    "merci",
    "danke",
    "card",
    "aid:",
    "auth",
    "aut:",
    "nsu",
    "iva contenido",
    "impuestos",
    "redondeo",
    "dinheiro",
    "tef ",
    "trib aprox",
    "valor dos impostos",
    "ccf:",
    "coo:",
    "gnf",
    "goods",
    "net ",
    "acumula",
    "cartao",
    "saldo",
    "disponivel",
    "tarjeta",
    "club",
];

/// O quanto o cupom lido merece confiança.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Tier {
    /// A conta não fechou: revisão ou IA.
    Unbalanced,
    /// Itens − descontos = total.
    Balanced,
    /// Fechou e a quantidade de itens impressa ou o pagamento batem também.
    Checked,
}

/// Um item lido. Valores em centavos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LineItem {
    pub description: String,
    /// Quantidade como impressa, com ponto decimal ("0.540", "2").
    pub quantity: Option<String>,
    #[ts(type = "number | null")]
    pub unit_price: Option<i64>,
    #[ts(type = "number")]
    pub total: i64,
    /// Quantidade × unitário = total (quando a linha traz os três).
    pub line_checked: Option<bool>,
    /// Marca de imposto depois do preço, nos cupons em que o imposto é somado no fim ("T",
    /// "X": tributável; outra letra: não). Sem marca legível, ausente.
    #[serde(default)]
    #[ts(optional)]
    pub taxable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LinesReading {
    pub items: Vec<LineItem>,
    #[ts(type = "number")]
    pub discount: i64,
    #[ts(type = "number | null")]
    pub total: Option<i64>,
    #[ts(type = "number")]
    pub items_sum: i64,
    pub tier: Tier,
    /// "Qtd. total de itens", "8 Items"… quando o cupom imprime.
    #[ts(type = "number | null")]
    pub printed_count: Option<u32>,
    /// Imposto somado no fim do cupom (EUA, Canadá): total = itens − desconto + imposto.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub tax: Option<i64>,
    /// O subtotal impresso (antes do imposto somado).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub subtotal: Option<i64>,
    /// O texto das linhas, na ordem do cupom (para a revisão e para a IA).
    pub rows: Vec<String>,
}

struct Row {
    text: String,
}

/// Gira as caixas pela inclinação mediana das linhas largas (foto torta).
fn deskew(lines: &mut [OcrLine], aspect: f32) {
    let mut angs: Vec<f32> = lines
        .iter()
        .filter(|l| l.w > 0.15)
        .filter_map(|l| {
            let (tl, tr) = (l.tl?, l.tr?);
            let dx = (tr[0] - tl[0]) * aspect;
            (dx > 0.0).then(|| (tr[1] - tl[1]).atan2(dx))
        })
        .collect();
    if angs.len() < 3 {
        return;
    }
    angs.sort_by(f32::total_cmp);
    let a = angs[angs.len() / 2];
    if a.abs() < 0.005 {
        return;
    }
    let (c, s) = ((-a).cos(), (-a).sin());
    for l in lines {
        let cx = (l.x + l.w / 2.0) * aspect;
        let cy = l.y + l.h / 2.0;
        let (nx, ny) = (cx * c - cy * s, cx * s + cy * c);
        l.x = nx / aspect - l.w / 2.0;
        l.y = ny - l.h / 2.0;
    }
}

/// Junta as caixas em linhas do cupom pela altura (origem embaixo, como no Vision).
fn rows(lines: &[OcrLine], aspect: f32) -> Vec<Row> {
    let mut ls: Vec<OcrLine> = lines
        .iter()
        .filter(|l| !l.text.trim().is_empty())
        .cloned()
        .collect();
    if ls.is_empty() {
        return vec![];
    }
    deskew(&mut ls, aspect);
    let mut hs: Vec<f32> = ls.iter().map(|l| l.h).collect();
    hs.sort_by(f32::total_cmp);
    let med = if hs[hs.len() / 2] > 0.0 {
        hs[hs.len() / 2]
    } else {
        0.01
    };
    let center = |l: &OcrLine| l.y + l.h / 2.0;
    ls.sort_by(|a, b| center(b).total_cmp(&center(a)));
    let mut groups: Vec<Vec<OcrLine>> = vec![];
    let mut cy = 0.0;
    for l in ls {
        let c = center(&l);
        match groups.last_mut() {
            Some(cur) if (c - cy).abs() <= 0.6 * med => {
                cur.push(l);
                cy = cur.iter().map(center).sum::<f32>() / cur.len() as f32;
            }
            _ => {
                cy = c;
                groups.push(vec![l]);
            }
        }
    }
    groups
        .into_iter()
        .map(|mut g| {
            g.sort_by(|a, b| a.x.total_cmp(&b.x));
            Row {
                text: g
                    .iter()
                    .map(|l| l.text.trim())
                    .collect::<Vec<_>>()
                    .join("  "),
            }
        })
        .collect()
}

/// O texto das linhas, de cima para baixo (caixas juntas pela altura, endireitadas): base
/// da leitura de texto comum (`crate::ocr`), fora do cupom.
pub fn row_texts(lines: &[OcrLine], aspect: f32) -> Vec<String> {
    rows(lines, aspect.max(0.01))
        .into_iter()
        .map(|r| r.text)
        .collect()
}

fn is_confusable(c: char) -> bool {
    matches!(c, 'O' | 'o' | 'D' | 'l' | 'I' | '|' | 'S' | 'B' | 'Z')
}

fn fix_digit(c: char) -> char {
    match c {
        'U' | 'Q' | 'O' | 'o' | 'D' => '0',
        'l' | 'I' | '|' => '1',
        'S' => '5',
        'B' => '8',
        'Z' => '2',
        other => other,
    }
}

static CURRENCY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:GBP|EUR|CAD|USD|ARS|CLP|BRL|R\$|RS)\s?([\d-])").unwrap());
static SEMICOLON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d)\s?;\s?(\d{2})\b").unwrap());
static SPLIT_CENTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d[.,])\s(\d{2})\b").unwrap());

/// Arruma o texto para achar os valores: tira a moeda, junta "9, 17", troca letras que o OCR
/// pôs no lugar de dígitos ("U,35", "1,5O", "63,0l").
fn normalize_numbers(text: &str) -> String {
    let t = CURRENCY.replace_all(text, " $1");
    let t = SEMICOLON.replace_all(&t, "$1,$2");
    let t = SPLIT_CENTS.replace_all(&t, "$1$2");
    let cs: Vec<char> = t.chars().collect();
    let mut out = String::with_capacity(t.len());
    for i in 0..cs.len() {
        let c = cs[i];
        let prev = i.checked_sub(1).map(|j| cs[j]);
        let prev2 = i.checked_sub(2).map(|j| cs[j]);
        let digit_ahead = cs[i + 1..]
            .iter()
            .take_while(|x| x.is_ascii_digit() || **x == '.' || **x == ',')
            .any(char::is_ascii_digit);
        let between = is_confusable(c)
            && prev.is_some_and(|p| p.is_ascii_digit() || p == '.' || p == ',')
            && digit_ahead;
        let leading = matches!(c, 'O' | 'o' | 'U' | 'Q')
            && !prev.is_some_and(char::is_alphabetic)
            && cs.get(i + 1).is_some_and(|n| *n == '.' || *n == ',')
            && cs.get(i + 2).is_some_and(char::is_ascii_digit);
        let trailing = is_confusable(c)
            && prev.is_some_and(|p| p.is_ascii_digit())
            && prev2.is_some_and(|p| p == '.' || p == ',');
        out.push(if between || leading || trailing {
            fix_digit(c)
        } else {
            c
        });
    }
    out
}

static MONEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^-?[$€£]?\s?((?:\d{1,3}(?:[.,\s]\d{3})+|\d{1,7})[.,]\s?\d{2})").unwrap()
});
static MONEY_CL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-?\$?\s?(\d{1,3}(?:\.\d{3})+|\d{2,7})").unwrap());
static TAX_LETTER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s?[A-Z]\b").unwrap());

/// Um valor achado na linha: o número e onde começa.
struct Money {
    value: Decimal,
    start: usize,
}

/// Valor em decimal a partir do número impresso ("1.234,56", "1,234.56", "31.92", "1.990").
fn to_decimal(s: &str, country: &str) -> Option<Decimal> {
    let s: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '$' | '€' | '£'))
        .collect();
    if country == "CL" {
        return s.replace(['.', ','], "").parse().ok();
    }
    let cut = s.rfind(['.', ','])?;
    let (int, frac) = (&s[..cut], &s[cut + 1..]);
    let int: String = int.chars().filter(char::is_ascii_digit).collect();
    match frac.len() {
        2 => format!("{int}.{frac}").parse().ok(),
        3 if COMMA.contains(&country) && s[cut..].starts_with(',') => {
            format!("{int}.{frac}").parse().ok()
        }
        _ => format!("{int}{frac}").parse().ok(),
    }
}

fn find_money(t: &str, country: &str) -> Vec<Money> {
    let re: &Regex = if country == "CL" { &MONEY_CL } else { &MONEY };
    let mut out = vec![];
    let mut i = 0;
    while i < t.len() {
        if !t.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let prev = t[..i].chars().next_back();
        if prev.is_some_and(|p| p.is_alphanumeric() || p == '.' || p == ',' || p == '_') {
            i += 1;
            continue;
        }
        let Some(m) = re.captures(&t[i..]) else {
            i += 1;
            continue;
        };
        let whole = m.get(0).unwrap();
        let mut end = i + whole.end();
        let rest = &t[end..];
        // Sinal, moeda ou letra da alíquota depois do valor ("4,00-", "3,99 €", "1.50 A").
        let mut negative = whole.as_str().starts_with('-');
        if let Some(r) = rest.strip_prefix('-').or_else(|| rest.strip_prefix(" -")) {
            negative = true;
            end = t.len() - r.len();
        } else if let Some(r) = rest.strip_prefix('€').or_else(|| rest.strip_prefix(" €")) {
            end = t.len() - r.len();
        } else if let Some(l) = TAX_LETTER.find(rest) {
            end += l.end();
        } else if let Some(c) = rest.chars().next()
            && "0O)GTFIN8".contains(c)
            && rest[c.len_utf8()..].trim().is_empty()
        {
            // Letra da situação tributária grudada no fim da linha ("10,390", "6,99)").
            end += c.len_utf8();
        }
        let next = t[end..].chars().next();
        let bad_next = if country == "CL" {
            next.is_some_and(|n| n.is_alphanumeric() || matches!(n, '.' | ',' | '%'))
        } else {
            next.is_some_and(|n| n.is_ascii_digit() || n == '%')
        };
        if bad_next {
            i += 1;
            continue;
        }
        if let Some(v) = to_decimal(m.get(1).unwrap().as_str(), country) {
            out.push(Money {
                value: if negative { -v } else { v },
                start: i,
            });
        }
        i = end.max(i + 1);
    }
    out
}

static QTY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(\d+(?:[.,]\d{1,3})?)\s*(?i:un|und|unid|kg|g|l|lt|pc|pç|lb|ea)?\s*[xX×@*]\s*\$?\s*(\d+[.,]\d{2,3})",
    )
    .unwrap()
});
static COLUMN_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:vl|valor|v\.)\s*total\b.*\b(?:unit|item)|\b(?:unit|item).*\btotal").unwrap()
});
static COUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:qtd\.?|qtde\.?|quantidade)\s*total\s*de\s*itens\D{0,6}(\d{1,3})|(\d{1,3})\s*(?:itens|items?|articulos|artigos|art\.)\b|items? sold\D{0,3}(\d{1,3})",
    )
    .unwrap()
});

fn letters(s: &str) -> usize {
    s.chars().filter(char::is_ascii_alphabetic).count()
}

fn has_any(f: &str, kws: &[&str]) -> bool {
    kws.iter().any(|k| f.contains(k))
}

fn total_rank(f: &str) -> Option<usize> {
    let (i, k) = TOTAL_KW.iter().enumerate().find(|(_, k)| f.contains(*k))?;
    let bad = NOT_TOTAL.iter().any(|n| f.contains(n) && !k.contains(n));
    (!bad).then_some(i)
}

fn cents(d: Decimal) -> i64 {
    (d * Decimal::ONE_HUNDRED).round().to_i64().unwrap_or(0)
}

fn close(a: Decimal, b: Decimal) -> bool {
    (a - b).abs() <= Decimal::new(2, 2)
}

struct Info {
    text: String,
    fold: String,
    money: Vec<Money>,
    fixed: String,
}

impl Info {
    fn last(&self) -> Option<Decimal> {
        self.money.last().map(|m| m.value)
    }
}

struct Item {
    description: String,
    qty: Option<Decimal>,
    unit: Option<Decimal>,
    total: Decimal,
    line_checked: Option<bool>,
    taxable: Option<bool>,
}

/// Linha do imposto somado no fim ("TAX 1 7.25% 1.45", "SALES TAX", "GST", "HST"); fora o que
/// só fala de imposto ("TAXABLE", "TAX ID", "TAX EXEMPT").
static TAX_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:sales\s+tax|tax|gst|hst|pst|qst|tps|tvq)\b").unwrap());
static NOT_TAX_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"taxable|tax\s*id|tax\s*exempt|tax\s*free|before\s+tax").unwrap());
/// A letra de imposto depois do preço do item ("3.49 T", "2.99 F", "12.00 TX").
static TAX_FLAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d[.,]\d{2}\s?-?\s?([A-Z]{1,2})\s*$").unwrap());

/// Lê o cupom a partir das linhas do OCR. `country` decide o separador decimal; `aspect` é a
/// largura / altura da foto (para endireitar).
pub fn read(lines: &[OcrLine], country: &str, aspect: f32) -> LinesReading {
    let rs = rows(lines, aspect.max(0.01));
    let info: Vec<Info> = rs
        .iter()
        .map(|r| {
            let fixed = normalize_numbers(&r.text);
            Info {
                money: find_money(&fixed, country),
                fold: fold_keep_punct(&r.text),
                text: r.text.clone(),
                fixed,
            }
        })
        .collect();

    // 1. Total: a linha de total de palavra mais forte. Os itens vão até a primeira delas.
    let (mut total, mut ti, mut best, mut tidx) = (None::<Decimal>, info.len(), usize::MAX, 0);
    let mut late_discount = Decimal::ZERO;
    for (idx, r) in info.iter().enumerate() {
        let Some(k) = total_rank(&r.fold) else {
            continue;
        };
        let has_value = !r.money.is_empty();
        if idx < ti && (has_value || k < STRONG_TOTALS) && !COLUMN_HEADER.is_match(&r.fold) {
            ti = idx;
        }
        if !has_value && k < STRONG_TOTALS {
            // "VALOR TOTAL R$" sem valor: o valor caiu na linha de baixo ou de cima.
            for j in [idx + 1, idx.wrapping_sub(1)] {
                if let Some(n) = info.get(j)
                    && let Some(v) = n.last()
                    && k < best
                    && (letters(&n.fold) < 4 || total_rank(&n.fold).is_some())
                {
                    (total, best) = (Some(v), k);
                    break;
                }
            }
        }
        let Some(v) = r.last() else {
            continue;
        };
        if k < best {
            (total, best, tidx) = (Some(v), k, idx);
        } else if k == best
            && total.is_some_and(|t| v < t)
            && info[tidx + 1..idx]
                .iter()
                .any(|x| has_any(&x.fold, DISC_KW))
        {
            // "Total 20,67 / Poupança 1,56 / Total 19,11": vale o total depois do desconto.
            late_discount = info[tidx + 1..idx]
                .iter()
                .filter(|x| has_any(&x.fold, DISC_KW))
                .filter_map(Info::last)
                .map(|v| v.abs())
                .sum();
            (total, tidx) = (Some(v), idx);
        }
    }
    let pays: Vec<Decimal> = info[ti.min(info.len())..]
        .iter()
        .filter(|r| has_any(&r.fold, PAY_KW) && !has_any(&r.fold, CHANGE_KW))
        .filter_map(Info::last)
        .collect();

    // O imposto somado no fim e o subtotal, nos países em que o preço do item não tem o
    // imposto (docs/design/consumption-review.md, seção 4). "TOTAL TAX" vale sozinho.
    let added = crate::food::profile::of(country).tax == crate::food::profile::TaxAtTill::Added;
    let (mut tax, mut subtotal) = (Decimal::ZERO, None::<Decimal>);
    if added {
        let end = if total.is_some() { tidx } else { info.len() };
        let mut whole = None;
        for r in &info[..end.min(info.len())] {
            let Some(v) = r.last() else { continue };
            if has_any(&r.fold, SUBTOTAL_KW) {
                subtotal = Some(v);
            } else if TAX_LINE.is_match(&r.fold) && !NOT_TAX_LINE.is_match(&r.fold) {
                if r.fold.contains("total") {
                    whole = Some(v.abs());
                } else {
                    tax += v.abs();
                }
            }
        }
        if let Some(w) = whole {
            tax = w;
        }
    }

    // 2. Itens, entre o cabeçalho e o total.
    let mut items: Vec<Item> = vec![];
    let mut discount = Decimal::ZERO;
    let mut pending: Option<String> = None;
    let mut pending_qty: Option<(Decimal, Decimal)> = None;
    for r in &info[..ti.min(info.len())] {
        let f = &r.fold;
        if has_any(f, SUBTOTAL_KW) {
            continue;
        }
        if let Some(v) = r.last()
            && has_any(f, DISC_KW)
        {
            discount += v.abs();
            continue;
        }
        let Some(price) = r.last() else {
            if letters(f) >= 3 {
                pending = (!has_any(f, SKIP_KW)).then(|| r.text.clone());
            }
            continue;
        };
        if has_any(f, SKIP_KW) {
            pending = None;
            continue;
        }
        let cut = r.money.last().map_or(r.fixed.len(), |m| m.start);
        let mut desc = r.fixed[..cut].trim().to_string();
        let (mut qty, mut unit) = (None, None);
        if let Some(q) = QTY.captures(&r.fixed)
            && q.get(0).unwrap().start() < cut
        {
            let qs = q.get(1).unwrap().as_str();
            qty = to_decimal(qs, if qs.contains(',') { "BR" } else { "US" });
            unit = to_decimal(q.get(2).unwrap().as_str(), country);
            desc = r.fixed[..q.get(0).unwrap().start()].trim().to_string();
        }
        // Linha só de quantidade ("2,000 x 1.250,00"): o único valor é o unitário.
        let qty_only = qty.is_some() && r.money.len() == 1 && unit == Some(price);
        if letters(&desc) < 3 || qty_only {
            if let (false, Some(p)) = (qty_only, pending.take()) {
                desc = p;
            } else if let (Some(q), Some(u)) = (qty, unit) {
                let tol = Decimal::new(2, 2) + q * Decimal::new(5, 3);
                match items.last_mut() {
                    // Linha de peso ou quantidade do item de cima ("0,540 kg x 1,49   0,80").
                    Some(prev) if prev.qty.is_none() && (q * u - prev.total).abs() <= tol => {
                        (prev.qty, prev.unit, prev.line_checked) = (Some(q), Some(u), Some(true));
                    }
                    // Quantidade antes do item (Argentina, Aldi): vale para a próxima linha.
                    _ => pending_qty = Some((q, u)),
                }
                continue;
            } else {
                continue;
            }
        }
        if qty.is_none()
            && let Some((q, u)) = pending_qty
        {
            (qty, unit) = (Some(q), Some(u));
        }
        pending_qty = None;
        if price < Decimal::ZERO {
            discount += -price;
            pending = None;
            continue;
        }
        let line_checked = match (qty, unit) {
            (Some(q), Some(u)) => {
                Some((q * u - price).abs() <= Decimal::new(2, 2) + q * Decimal::new(5, 3))
            }
            _ => None,
        };
        let taxable = added
            .then(|| TAX_FLAG.captures(r.text.trim()))
            .flatten()
            .map(|c| c[1].contains('T') || c[1].contains('X'));
        items.push(Item {
            description: desc,
            qty,
            unit,
            total: price,
            line_checked,
            taxable,
        });
        pending = None;
    }
    let sum: Decimal = items.iter().map(|i| i.total).sum();
    discount += late_discount;
    // O subtotal impresso confere a soma dos itens; a linha de imposto que o OCR perdeu sai
    // da diferença entre o total e o subtotal.
    let sub_ok = subtotal.is_some_and(|s| close(sum - discount, s));
    if added
        && sub_ok
        && tax.is_zero()
        && let (Some(t), Some(s)) = (total, subtotal)
        && t > s
    {
        tax = t - s;
    }
    if let Some(t) = total
        && sum - discount + tax != t
        && pays.iter().any(|p| close(*p, sum - discount + tax))
    {
        // O total impresso foi mal lido; o pagamento e a soma dos itens concordam.
        total = Some(sum - discount + tax);
    }
    let mut balanced = false;
    if let Some(t) = total {
        if close(sum - discount + tax, t) {
            balanced = true;
        } else if close(sum + tax, t) {
            balanced = true;
            discount = Decimal::ZERO;
        }
    }
    // 3. Segunda conferência: a quantidade de itens impressa ou o pagamento.
    let printed_count = info.iter().find_map(|r| {
        let c = COUNT.captures(&r.fold)?;
        (1..=3)
            .find_map(|i| c.get(i))
            .and_then(|m| m.as_str().parse::<u32>().ok())
    });
    let units: u32 = items
        .iter()
        .map(|i| match i.qty {
            Some(q) if q.fract().is_zero() => q.to_u32().unwrap_or(1),
            _ => 1,
        })
        .sum();
    let count_ok = printed_count.is_some_and(|c| c == items.len() as u32 || c == units);
    let pay_ok = total.is_some_and(|t| pays.contains(&t));
    // Com imposto somado, as duas contas fechando (itens = subtotal, subtotal + imposto =
    // total) valem como a segunda conferência.
    let tier = match (balanced, count_ok || pay_ok || (sub_ok && !tax.is_zero())) {
        (true, true) => Tier::Checked,
        (true, false) => Tier::Balanced,
        _ => Tier::Unbalanced,
    };
    LinesReading {
        items: items
            .into_iter()
            .map(|i| LineItem {
                description: crate::food::clean_description(&i.description),
                quantity: i.qty.map(|q| q.normalize().to_string()),
                unit_price: i.unit.map(cents),
                total: cents(i.total),
                line_checked: i.line_checked,
                taxable: i.taxable,
            })
            .collect(),
        tax: (!tax.is_zero()).then(|| cents(tax)),
        subtotal: subtotal.map(cents),
        discount: cents(discount),
        total: total.map(cents),
        items_sum: cents(sum),
        tier,
        printed_count,
        rows: rs.into_iter().map(|r| r.text).collect(),
    }
}

#[cfg(test)]
mod tests;
