//! Leitura do cupom fiscal (docs/receipts.md): a página da consulta pública da NFC-e (o link do
//! QR do cupom, no leiaute das SEFAZ: `txtTit`, `Rqtd`, `RUN`, `RvlUnit`, `valor`,
//! `totalNumb`), o texto dessa página copiado, ou o texto do PDF do DANFE NFC-e. Tira a loja, a
//! chave de acesso, a data, os itens, os totais e os tributos aproximados (Lei 12.741/2012).
//! Nada do consumidor (CPF, nome) é lido.

use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::i18n::{args, t, t_args};
use crate::import::text::fold_keep_punct as fold;
use crate::transactions::util::money_text;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ParsedItem {
    pub code: Option<String>,
    pub description: String,
    /// Quantidade como no cupom (`1`, `0,535`).
    pub quantity: Option<String>,
    pub unit: Option<String>,
    #[ts(type = "number | null")]
    pub unit_price: Option<i64>,
    #[ts(type = "number")]
    pub total: i64,
    /// GTIN/EAN do item: o `cEAN` do XML ou o código da loja quando ele é um GTIN válido.
    #[serde(default)]
    #[ts(optional)]
    pub gtin: Option<String>,
    /// Desconto do próprio item (`vDesc` do XML), em centavos.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub discount: i64,
}

/// GTIN válido (8, 12, 13 ou 14 dígitos, dígito verificador conferido), como veio no cupom.
/// Códigos de uso interno ou restrito (balança, peso variável, cupons, vales) não são GTIN
/// (`food::gtin::gtin14`, docs/gtin-inflation.md §3.3).
pub fn valid_gtin(code: &str) -> Option<String> {
    crate::food::gtin::gtin14(code).map(|_| code.trim().to_string())
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ParsedReceipt {
    pub access_key: Option<String>,
    pub store_name: Option<String>,
    pub store_cnpj: Option<String>,
    pub uf: Option<String>,
    /// `YYYY-MM-DD`.
    pub date: Option<String>,
    /// `YYYY-MM-DDTHH:MM:SS`, quando o cupom traz a hora.
    pub issued_at: Option<String>,
    pub items: Vec<ParsedItem>,
    #[ts(type = "number")]
    pub items_total: i64,
    #[ts(type = "number")]
    pub discount: i64,
    #[ts(type = "number")]
    pub paid: i64,
    #[ts(type = "number | null")]
    pub tax_total: Option<i64>,
    #[ts(type = "number | null")]
    pub tax_federal: Option<i64>,
    #[ts(type = "number | null")]
    pub tax_state: Option<i64>,
    #[ts(type = "number | null")]
    pub tax_municipal: Option<i64>,
    pub warnings: Vec<String>,
}

/// UF pelos dois primeiros dígitos da chave de acesso (código do IBGE).
pub fn uf_of_key(key: &str) -> Option<&'static str> {
    Some(match key.get(..2)? {
        "11" => "RO",
        "12" => "AC",
        "13" => "AM",
        "14" => "RR",
        "15" => "PA",
        "16" => "AP",
        "17" => "TO",
        "21" => "MA",
        "22" => "PI",
        "23" => "CE",
        "24" => "RN",
        "25" => "PB",
        "26" => "PE",
        "27" => "AL",
        "28" => "SE",
        "29" => "BA",
        "31" => "MG",
        "32" => "ES",
        "33" => "RJ",
        "35" => "SP",
        "41" => "PR",
        "42" => "SC",
        "43" => "RS",
        "50" => "MS",
        "51" => "MT",
        "52" => "GO",
        "53" => "DF",
        _ => return None,
    })
}

/// Chave de acesso (44 dígitos) de um link do QR (`?p=CHAVE|2|1|…`) ou de um texto.
pub fn access_key(s: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(\d{4}\s?){10}\d{4}").expect("regex"));
    re.find_iter(s)
        .map(|m| {
            m.as_str()
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
        })
        .find(|k| k.len() == 44 && key_check_digit_ok(k))
}

/// Dígito verificador da chave (módulo 11, pesos 2 a 9 da direita para a esquerda).
fn key_check_digit_ok(key: &str) -> bool {
    let digits: Vec<u32> = key.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() != 44 {
        return false;
    }
    let mut sum = 0;
    let mut w = 2;
    for d in digits[..43].iter().rev() {
        sum += d * w;
        w = if w == 9 { 2 } else { w + 1 };
    }
    let r = sum % 11;
    let dv = if r < 2 { 0 } else { 11 - r };
    dv == digits[43]
}

fn money(s: &str) -> Option<i64> {
    crate::money::parse_amount(s.trim(), "BRL").ok()
}

fn amount_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\d{1,3}(?:\.\d{3})*,\d{2}|\d+,\d{2}").expect("regex"))
}

/// HTML → linhas de texto, mantendo as marcas das classes que importam como prefixos.
fn html_to_lines(html: &str) -> Vec<String> {
    static TAG: OnceLock<Regex> = OnceLock::new();
    let tag = TAG.get_or_init(|| Regex::new(r"(?s)<[^>]*>").expect("regex"));
    static CLS: OnceLock<Regex> = OnceLock::new();
    let cls = CLS.get_or_init(|| {
        Regex::new(r#"(?i)<(?:span|td|div|label)[^>]*class="([^"]*)"[^>]*>"#).expect("regex")
    });
    // As quebras de linha do código da página não contam: as linhas vêm das tags.
    let flat = html.replace(['\n', '\r', '\t'], " ");
    let no_script = Regex::new(r"(?is)<(script|style)[^>]*>.*?</(script|style)>")
        .expect("regex")
        .replace_all(&flat, " ");
    // Cada classe vira um marcador `⟦classe⟧`; quebras nas linhas da tabela e dos blocos.
    let marked = cls.replace_all(&no_script, |c: &regex::Captures<'_>| {
        let first = c[1].split_whitespace().next().unwrap_or("").to_string();
        format!(" ⟦{first}⟧ ")
    });
    // Quebra nas linhas da tabela e nos blocos; o `<br>` dentro do item não quebra (a
    // descrição e o valor ficam na mesma linha).
    let broken = Regex::new(r"(?i)</(tr|div|li|p)\s*>|<tr[^>]*>")
        .expect("regex")
        .replace_all(&marked, "\n");
    let text = tag.replace_all(&broken, " ");
    decode_entities(&text)
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect()
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&ccedil;", "ç")
        .replace("&atilde;", "ã")
        .replace("&otilde;", "õ")
        .replace("&aacute;", "á")
        .replace("&eacute;", "é")
        .replace("&iacute;", "í")
        .replace("&oacute;", "ó")
        .replace("&uacute;", "ú")
        .replace("&ecirc;", "ê")
        .replace("&ocirc;", "ô")
        .replace("&acirc;", "â")
        .replace("&Ccedil;", "Ç")
        .replace("&Atilde;", "Ã")
}

/// É a página (ou o texto) de uma NFC-e.
pub fn looks_like(text: &str) -> bool {
    let f = fold(text);
    (f.contains("nfc-e") || f.contains("nfce") || f.contains("consumidor eletronica"))
        || (f.contains("qtde") && f.contains("vl. unit"))
        || f.contains("txttit")
}

/// Lê a página HTML da consulta da NFC-e.
pub fn parse_html(html: &str) -> ParsedReceipt {
    let lines = html_to_lines(html);
    parse_lines(&lines, true)
}

/// Lê o texto do cupom (página copiada ou PDF do DANFE NFC-e).
pub fn parse_text(text: &str) -> ParsedReceipt {
    let lines: Vec<String> = text
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect();
    parse_lines(&lines, false)
}

fn after_amount(line: &str) -> Option<i64> {
    amount_re()
        .find_iter(line)
        .last()
        .and_then(|m| money(m.as_str()))
}

fn parse_lines(lines: &[String], html: bool) -> ParsedReceipt {
    let mut r = ParsedReceipt::default();
    let all = lines.join("\n");
    r.access_key = access_key(&all);
    r.uf = r
        .access_key
        .as_deref()
        .and_then(uf_of_key)
        .map(String::from);

    // Loja: a primeira linha com letras antes do CNPJ (no HTML, `txtTopo`).
    static CNPJ: OnceLock<Regex> = OnceLock::new();
    let cnpj = CNPJ.get_or_init(|| {
        Regex::new(r"\b(\d{2})\.?(\d{3})\.?(\d{3})/?(\d{4})-?(\d{2})\b").expect("regex")
    });
    if let Some((i, c)) = lines.iter().enumerate().find_map(|(i, l)| {
        fold(l)
            .contains("cnpj")
            .then(|| cnpj.captures(l).map(|c| (i, c)))
            .flatten()
    }) {
        r.store_cnpj = Some(format!("{}{}{}{}{}", &c[1], &c[2], &c[3], &c[4], &c[5]));
        let topo = lines
            .iter()
            .find_map(|l| l.split("⟦txtTopo⟧").nth(1).map(clean));
        let before = lines[..i]
            .iter()
            .rev()
            .map(|l| clean(l))
            .find(|l| l.chars().filter(|c| c.is_alphabetic()).count() >= 3);
        r.store_name = topo.filter(|s| !s.is_empty()).or(before).map(|s| {
            let s = s
                .trim_end_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            s.split(" CNPJ").next().unwrap_or(&s).trim().to_string()
        });
    }

    // Emissão: "Emissão: 19/10/2026 10:21:33".
    static EMI: OnceLock<Regex> = OnceLock::new();
    let emi = EMI.get_or_init(|| {
        Regex::new(r"(\d{2})/(\d{2})/(20\d{2})(?:\s+(\d{2}):(\d{2})(?::(\d{2}))?)?").expect("regex")
    });
    let emission = lines
        .iter()
        .find(|l| fold(l).contains("emissao"))
        .and_then(|l| emi.captures(l))
        .or_else(|| lines.iter().find_map(|l| emi.captures(l)));
    if let Some(c) = emission {
        let date = format!("{}-{}-{}", &c[3], &c[2], &c[1]);
        if let (Some(h), Some(m)) = (c.get(4), c.get(5)) {
            let s = c.get(6).map_or("00", |s| s.as_str());
            r.issued_at = Some(format!("{date}T{}:{}:{s}", h.as_str(), m.as_str()));
        }
        r.date = Some(date);
    }

    r.items = if html {
        items_html(lines)
    } else {
        items_text(lines)
    };

    // Totais e tributos.
    for l in lines {
        let f = fold(l);
        let v = after_amount(l);
        if f.contains("tributos")
            && (f.contains("12.741")
                || f.contains("12741")
                || f.contains("incidentes")
                || f.contains("aproximad"))
        {
            // "Tributos Totais Incidentes (Lei Federal 12.741/2012) R$ 10,45" ou, em alguns
            // estados, "Trib aprox R$ 4,10 Fed 2,10 Est 1,90 Mun 0,10".
            if r.tax_total.is_none() {
                r.tax_total = tax_total_of(l);
            }
            split_taxes(l, &mut r);
        } else if f.contains("desconto") && !f.contains("acrescimo") {
            if let Some(v) = v {
                r.discount = r.discount.max(v);
            }
        } else if f.contains("valor a pagar") {
            if let Some(v) = v {
                r.paid = v;
            }
        } else if (f.contains("valor total") || f.starts_with("total r$"))
            && !f.contains("tributo")
            && let Some(v) = v
        {
            r.items_total = r.items_total.max(v);
        }
    }
    let sum: i64 = r.items.iter().map(|i| i.total).sum();
    if r.items_total == 0 {
        r.items_total = sum;
    }
    if r.paid == 0 {
        r.paid = r.items_total - r.discount;
    }
    if r.items.is_empty() {
        r.warnings
            .push(t("nucleo-importacao-cupom-aviso-sem-itens"));
    } else if (sum - r.items_total).abs() > 1 {
        r.warnings.push(t_args(
            "nucleo-importacao-cupom-aviso-soma",
            &args(&[
                ("soma", money_text(sum, "BRL").into()),
                ("total", money_text(r.items_total, "BRL").into()),
            ]),
        ));
    }
    if r.date.is_none() {
        r.warnings.push(t("nucleo-importacao-cupom-aviso-sem-data"));
    }
    r
}

fn clean(s: &str) -> String {
    s.split('⟦')
        .map(|p| p.split_once('⟧').map_or(p, |(_, rest)| rest))
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tax_total_of(line: &str) -> Option<i64> {
    // O primeiro valor depois do rótulo (ou do "R$"); "12.741/2012" não é valor.
    let f = line.replace("12.741/2012", "").replace("12741/2012", "");
    let amounts: Vec<i64> = amount_re()
        .find_iter(&f)
        .filter_map(|m| money(m.as_str()))
        .collect();
    let lf = fold(&f);
    if lf.contains("fed") && amounts.len() >= 2 {
        // "Trib aprox R$ 4,10 (Fed 2,10 Est 1,90 Mun 0,10)": o total vem primeiro.
        return amounts.first().copied();
    }
    amounts.last().copied()
}

pub(crate) fn split_taxes(line: &str, r: &mut ParsedReceipt) {
    static PART: OnceLock<Regex> = OnceLock::new();
    let part = PART.get_or_init(|| {
        Regex::new(r"(?i)(federal|fed|estadual|est|municipal|mun)\.?\s*:?\s*(?:r\$\s*)?(\d+,\d{2})")
            .expect("regex")
    });
    let f = fold(line);
    // O texto impresso costuma pôr o valor antes ("R$ 2,00 Federal, 1,00 Estadual, 0,10
    // Municipal"); outros cupons, o rótulo antes ("Fed 2,10 Est 1,90 Mun 0,10").
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    let before = BEFORE.get_or_init(|| {
        Regex::new(r"(?i)(\d+,\d{2})\s*\(?\s*(federal|fed|estadual|est|municipal|mun)\b")
            .expect("regex")
    });
    // Fica o formato que acha mais partes; no empate, o rótulo antes (o valor solto na
    // frente é o total).
    if before.captures_iter(&f).count() > part.captures_iter(&f).count() {
        for c in before.captures_iter(&f) {
            set_tax(r, &c[2], money(&c[1]));
        }
        return;
    }
    for c in part.captures_iter(&f) {
        set_tax(r, &c[1], money(&c[2]));
    }
}

fn set_tax(r: &mut ParsedReceipt, label: &str, v: Option<i64>) {
    match &label[..3] {
        "fed" => r.tax_federal = v,
        "est" => r.tax_state = v,
        "mun" => r.tax_municipal = v,
        _ => {}
    }
}

/// Itens da página da consulta: `⟦txtTit⟧ DESCRIÇÃO ⟦RCod⟧ (Código: 123) ⟦Rqtd⟧ Qtde.:1
/// ⟦RUN⟧ UN: KG ⟦RvlUnit⟧ Vl. Unit.: 5,99 … ⟦valor⟧ 5,99`.
fn items_html(lines: &[String]) -> Vec<ParsedItem> {
    let mut out = vec![];
    for l in lines {
        if !l.contains("⟦txtTit⟧") || !l.contains("⟦valor⟧") {
            continue;
        }
        let seg = |name: &str| -> Option<String> {
            let rest = l.split(&format!("⟦{name}⟧")).nth(1)?;
            Some(rest.split('⟦').next().unwrap_or("").trim().to_string())
        };
        let Some(description) =
            seg("txtTit").filter(|d| !d.is_empty() && !fold(d).starts_with("vl. total"))
        else {
            continue;
        };
        let total = seg("valor").and_then(|v| money(&v));
        let Some(total) = total else {
            continue;
        };
        let code = seg("RCod").and_then(|c| {
            let d: String = c.chars().filter(char::is_ascii_digit).collect();
            (!d.is_empty()).then_some(d)
        });
        let after_colon = |s: Option<String>| {
            s.and_then(|v| v.split_once(':').map(|(_, x)| x.trim().to_string()))
                .filter(|v| !v.is_empty())
        };
        out.push(ParsedItem {
            gtin: code.as_deref().and_then(valid_gtin),
            code,
            description: description.chars().take(120).collect(),
            quantity: after_colon(seg("Rqtd")),
            unit: after_colon(seg("RUN")),
            unit_price: after_colon(seg("RvlUnit")).and_then(|v| money(&v)),
            total,
            discount: 0,
        });
    }
    out
}

/// Itens do texto: o da página copiada ("DESCRIÇÃO (Código: 123) Qtde.:1 UN: UN Vl. Unit.: 5,99
/// Vl. Total 5,99") ou a tabela do DANFE ("123 DESCRIÇÃO 1 UN 5,99 5,99").
fn items_text(lines: &[String]) -> Vec<ParsedItem> {
    static PAGE: OnceLock<Regex> = OnceLock::new();
    let page = PAGE.get_or_init(|| {
        Regex::new(r"(?i)^(?P<desc>.+?)\s*\(C[oó]digo:\s*(?P<code>\w+)\s*\)\s*Qtde\.?:\s*(?P<qty>[\d,\.]+)\s*UN:?\s*(?P<un>\w+)\s*Vl\.?\s*Unit\.?:?\s*(?P<unit>[\d\.]+,\d{2,4})\s*Vl\.?\s*Total:?\s*(?P<total>[\d\.]+,\d{2})")
            .expect("regex")
    });
    static TABLE: OnceLock<Regex> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        Regex::new(r"(?i)^(?:(?P<n>\d{1,3})\s+)?(?P<code>\d{4,14})\s+(?P<desc>.+?)\s+(?P<qty>\d+(?:[,\.]\d{1,4})?)\s*(?P<un>UN|KG|L|LT|PC|PCT|CX|G|ML|M|DZ|FD|PT|BD|SC|GF|FR|BJ)\.?\s+(?:X\s+)?(?P<unit>[\d\.]*\d,\d{2,4})\s+(?P<total>[\d\.]*\d,\d{2})$")
            .expect("regex")
    });
    let mut out = vec![];
    let mut i = 0;
    while i < lines.len() {
        // A página copiada pode quebrar o item em duas linhas.
        let joined = if i + 1 < lines.len() {
            format!("{} {}", lines[i], lines[i + 1])
        } else {
            lines[i].clone()
        };
        let (cap, used) = match page.captures(&lines[i]) {
            Some(c) => (Some(c), 1),
            None => match page.captures(&joined) {
                Some(c) => (Some(c), 2),
                None => (table.captures(&lines[i]), 1),
            },
        };
        if let Some(c) = cap {
            let unit_price = c.name("unit").and_then(|m| {
                let v = m.as_str();
                // Preço unitário com 3 ou 4 casas: arredonda para centavos.
                let (int, dec) = v.rsplit_once(',')?;
                let d = format!("{:0<2}", &dec[..dec.len().min(2)]);
                money(&format!("{int},{d}"))
            });
            if let Some(total) = c.name("total").and_then(|m| money(m.as_str())) {
                out.push(ParsedItem {
                    gtin: c.name("code").and_then(|m| valid_gtin(m.as_str())),
                    code: c.name("code").map(|m| m.as_str().to_string()),
                    description: c["desc"].trim().chars().take(120).collect(),
                    quantity: c.name("qty").map(|m| m.as_str().to_string()),
                    unit: c.name("un").map(|m| m.as_str().to_uppercase()),
                    unit_price,
                    total,
                    discount: 0,
                });
            }
            i += used;
            continue;
        }
        i += 1;
    }
    out
}

/// É o XML de uma NF-e ou NFC-e (`<infNFe>`, do Portal Fiscal).
pub fn looks_like_xml(text: &str) -> bool {
    let head: String = text.chars().take(4000).collect();
    head.contains("<infNFe") || head.contains("portalfiscal.inf.br/nfe")
}

/// Conteúdo da primeira tag `<name>` (ou `<name …>`) em `s`.
fn xml_tag<'a>(s: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}");
    let mut from = 0;
    while let Some(i) = s[from..].find(&open) {
        let start = from + i + open.len();
        let next = s[start..].chars().next()?;
        if next == '>' || next.is_whitespace() {
            let body = start + s[start..].find('>')? + 1;
            let close = format!("</{name}>");
            let end = body + s[body..].find(&close)?;
            return Some(&s[body..end]);
        }
        from = start;
    }
    None
}

/// Todos os blocos `<name …>…</name>` em `s`.
fn xml_blocks<'a>(s: &'a str, name: &str) -> Vec<&'a str> {
    let mut out = vec![];
    let mut rest = s;
    while let Some(b) = xml_tag(rest, name) {
        out.push(b);
        let Some(pos) = rest.find(b) else { break };
        rest = &rest[pos + b.len()..];
    }
    out
}

/// "25.90" (ponto decimal do XML) em centavos.
fn xml_money(s: &str) -> Option<i64> {
    let d: rust_decimal::Decimal = s.trim().parse().ok()?;
    (d * rust_decimal::Decimal::from(100))
        .round()
        .to_string()
        .parse()
        .ok()
}

/// "1.0000" → "1"; "0.5350" → "0,535" (como no cupom).
fn xml_qty(s: &str) -> String {
    let t = s.trim();
    let t = if t.contains('.') {
        t.trim_end_matches('0').trim_end_matches('.')
    } else {
        t
    };
    t.replace('.', ",")
}

fn xml_text(s: &str) -> String {
    let no_cdata = s.replace("<![CDATA[", "").replace("]]>", "");
    decode_entities(&no_cdata)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lê o XML da NF-e ou NFC-e (o arquivo que a loja manda por e-mail). Do destinatário
/// (CPF, nome) nada é lido.
pub fn parse_xml(xml: &str) -> ParsedReceipt {
    let mut r = ParsedReceipt::default();
    let Some(inf) = xml_tag(xml, "infNFe") else {
        r.warnings
            .push(t("nucleo-importacao-cupom-aviso-xml-sem-nota"));
        return r;
    };
    r.access_key = xml_tag(xml, "chNFe").and_then(access_key).or_else(|| {
        let id_at = xml.find("<infNFe")?;
        let head = &xml[id_at..id_at + xml[id_at..].find('>')?];
        access_key(head)
    });
    r.uf = r
        .access_key
        .as_deref()
        .and_then(uf_of_key)
        .map(String::from);
    if let Some(emit) = xml_tag(inf, "emit") {
        r.store_cnpj = xml_tag(emit, "CNPJ").map(|c| c.trim().to_string());
        r.store_name = xml_tag(emit, "xFant")
            .or_else(|| xml_tag(emit, "xNome"))
            .map(xml_text)
            .filter(|s| !s.is_empty());
    }
    if let Some(ide) = xml_tag(inf, "ide") {
        let dh = xml_tag(ide, "dhEmi")
            .or_else(|| xml_tag(ide, "dEmi"))
            .unwrap_or("")
            .trim();
        if dh.len() >= 10 && chrono::NaiveDate::parse_from_str(&dh[..10], "%Y-%m-%d").is_ok() {
            r.date = Some(dh[..10].to_string());
            if dh.len() >= 19 {
                r.issued_at = Some(dh[..19].to_string());
            }
        }
    }
    let mut item_taxes = 0;
    for det in xml_blocks(inf, "det") {
        let Some(prod) = xml_tag(det, "prod") else {
            continue;
        };
        let total = xml_tag(prod, "vProd").and_then(xml_money).unwrap_or(0);
        let description = xml_tag(prod, "xProd").map(xml_text).unwrap_or_default();
        if description.is_empty() || total == 0 {
            continue;
        }
        item_taxes += xml_tag(det, "vTotTrib").and_then(xml_money).unwrap_or(0);
        let code = xml_tag(prod, "cProd").map(|c| c.trim().to_string());
        // O XML é a única fonte que sempre traz o GTIN (`cEAN`; "SEM GTIN" quando não há).
        let gtin = xml_tag(prod, "cEAN")
            .and_then(valid_gtin)
            .or_else(|| code.as_deref().and_then(valid_gtin));
        r.items.push(ParsedItem {
            code,
            description: description.chars().take(120).collect(),
            quantity: xml_tag(prod, "qCom").map(xml_qty),
            unit: xml_tag(prod, "uCom").map(|u| u.trim().to_string()),
            unit_price: xml_tag(prod, "vUnCom").and_then(xml_money),
            total,
            gtin,
            discount: xml_tag(prod, "vDesc").and_then(xml_money).unwrap_or(0),
        });
    }
    let tot = xml_tag(inf, "ICMSTot").unwrap_or("");
    r.items_total = xml_tag(tot, "vProd")
        .and_then(xml_money)
        .unwrap_or_else(|| r.items.iter().map(|i| i.total).sum());
    r.discount = xml_tag(tot, "vDesc").and_then(xml_money).unwrap_or(0);
    r.paid = xml_tag(tot, "vNF")
        .and_then(xml_money)
        .unwrap_or(r.items_total - r.discount);
    let printed = xml_tag(tot, "vTotTrib")
        .and_then(xml_money)
        .unwrap_or(item_taxes);
    r.tax_total = (printed > 0).then_some(printed);
    // A divisão federal/estadual/municipal só aparece no texto complementar.
    if let Some(cpl) = xml_tag(inf, "infCpl") {
        split_taxes(&xml_text(cpl), &mut r);
    }
    if r.items.is_empty() {
        r.warnings
            .push(t("nucleo-importacao-cupom-aviso-xml-sem-itens"));
    }
    r
}
