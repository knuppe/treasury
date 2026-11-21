//! Boleto bancário e conta com código de arrecadação (FEBRABAN), pelos 44 dígitos do código de
//! barras ou pela linha digitável (47 dígitos no boleto, 48 na arrecadação). Tudo é conferido
//! pelos dígitos verificadores: o erro do OCR num dígito não passa (docs/ocr-other-documents.md ›
//! 3.2).
//!
//! - **Boleto bancário:** banco (3), moeda (1), DV geral (módulo 11), fator de vencimento (4),
//!   valor em centavos (10), campo livre (25). Na linha digitável, cada um dos três primeiros
//!   campos tem DV módulo 10. O fator chegou a 9999 em 21/02/2025 e recomeçou em 1000 em
//!   22/02/2025: a data é a do ciclo mais perto de hoje.
//! - **Arrecadação** (começa com 8: contas de consumo, tributos, multas, carnês): segmento (1),
//!   identificador do valor (6 ou 8: valor em reais; 7 ou 9: referência), DV geral (módulo 10
//!   com 6/7, módulo 11 com 8/9), valor (11). Linha digitável: quatro blocos de 11 dígitos, cada
//!   um com o seu DV. O vencimento não vem no código: sai do texto.

use std::sync::LazyLock;

use chrono::{Duration, NaiveDate};
use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BillKind {
    /// Boleto de cobrança bancária.
    Bank,
    /// Código de arrecadação (concessionárias, prefeituras, órgãos do governo).
    Collection,
}

/// Um boleto ou conta lido e conferido.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Bill {
    pub kind: BillKind,
    /// Os 44 dígitos do código de barras.
    pub barcode: String,
    /// A linha digitável formatada (para copiar e pagar no banco).
    pub line: String,
    /// Valor em centavos (sem valor no código: em aberto ou só referência).
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    /// Vencimento (boleto: pelo fator; arrecadação: pelo texto, quando houver).
    #[ts(optional)]
    pub due_date: Option<String>,
    /// Banco emissor (boleto): código e nome, quando conhecido.
    #[ts(optional)]
    pub bank_code: Option<String>,
    #[ts(optional)]
    pub bank_name: Option<String>,
    /// Segmento da arrecadação (1 prefeituras, 2 saneamento, 3 energia e gás, 4 telecomunicações,
    /// 5 órgãos do governo, 6 carnês, 7 multas de trânsito, 9 uso exclusivo do banco).
    #[ts(optional)]
    pub segment: Option<u8>,
    /// Beneficiário, quando o texto do OCR diz.
    #[ts(optional)]
    pub beneficiary: Option<String>,
}

fn digits_only(s: &str) -> Option<Vec<u32>> {
    s.chars().map(|c| c.to_digit(10)).collect()
}

/// Módulo 10 (pesos 2 e 1 da direita para a esquerda, somando os algarismos do produto).
pub fn mod10(d: &[u32]) -> u32 {
    let sum: u32 = d
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &x)| {
            let p = x * if i % 2 == 0 { 2 } else { 1 };
            p / 10 + p % 10
        })
        .sum();
    (10 - sum % 10) % 10
}

fn mod11_sum(d: &[u32]) -> u32 {
    d.iter()
        .rev()
        .enumerate()
        .map(|(i, &x)| x * (2 + (i as u32 % 8)))
        .sum()
}

/// Módulo 11 do boleto bancário (resto 0, 1 ou 10 dá 1).
pub fn mod11_bank(d: &[u32]) -> u32 {
    match 11 - mod11_sum(d) % 11 {
        0 | 10 | 11 => 1,
        v => v,
    }
}

/// Módulo 11 da arrecadação (resto 0 ou 1 dá 0; resto 10 dá 1).
pub fn mod11_collection(d: &[u32]) -> u32 {
    match mod11_sum(d) % 11 {
        0 | 1 => 0,
        r => 11 - r,
    }
}

fn s(d: &[u32]) -> String {
    d.iter().map(|x| char::from(b'0' + *x as u8)).collect()
}

/// Bancos mais comuns pelo código da compensação (só para mostrar o nome).
fn bank_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "001" => "Banco do Brasil",
        "004" => "Banco do Nordeste",
        "021" => "Banestes",
        "033" => "Santander",
        "041" => "Banrisul",
        "070" => "BRB",
        "077" => "Banco Inter",
        "104" => "Caixa",
        "208" => "BTG Pactual",
        "212" => "Banco Original",
        "237" => "Bradesco",
        "260" => "Nubank",
        "290" => "PagBank",
        "323" => "Mercado Pago",
        "336" => "C6 Bank",
        "341" => "Itaú",
        "380" => "PicPay",
        "389" => "Banco Mercantil",
        "422" => "Safra",
        "655" => "Banco Votorantim",
        "748" => "Sicredi",
        "756" => "Sicoob",
        _ => return None,
    })
}

/// Data do fator de vencimento: base 07/10/1997, ciclos de 9.000 dias (o fator 1000 de
/// 22/02/2025 é o 10.000º dia), o mais perto de `today`. Fator 0: sem vencimento.
pub fn due_from_factor(factor: u32, today: NaiveDate) -> Option<NaiveDate> {
    if factor < 1000 {
        return None;
    }
    let base = NaiveDate::from_ymd_opt(1997, 10, 7)?;
    (0..4)
        .filter_map(|k| base.checked_add_signed(Duration::days(factor as i64 + 9000 * k)))
        .min_by_key(|d| (*d - today).num_days().abs())
}

/// Lê os 44 dígitos do código de barras (conferindo o DV geral).
pub fn from_barcode(code: &str, today: NaiveDate) -> Option<Bill> {
    let d = digits_only(code.trim())?;
    if d.len() != 44 {
        return None;
    }
    if d[0] == 8 {
        let ten = matches!(d[2], 6 | 7);
        let eleven = matches!(d[2], 8 | 9);
        if !ten && !eleven {
            return None;
        }
        let mut rest = d.clone();
        rest.remove(3);
        let dv = if ten {
            mod10(&rest)
        } else {
            mod11_collection(&rest)
        };
        if dv != d[3] {
            return None;
        }
        let amount: i64 = s(&d[4..15]).parse().ok()?;
        let line = d
            .chunks(11)
            .map(|c| {
                let dv = if ten { mod10(c) } else { mod11_collection(c) };
                format!("{}-{dv}", s(c))
            })
            .collect::<Vec<_>>()
            .join(" ");
        return Some(Bill {
            kind: BillKind::Collection,
            barcode: s(&d),
            line,
            amount: (matches!(d[2], 6 | 8) && amount > 0).then_some(amount),
            due_date: None,
            bank_code: None,
            bank_name: None,
            segment: Some(d[1] as u8),
            beneficiary: None,
        });
    }
    let mut rest = d.clone();
    rest.remove(4);
    if mod11_bank(&rest) != d[4] {
        return None;
    }
    let factor: u32 = s(&d[5..9]).parse().ok()?;
    let amount: i64 = s(&d[9..19]).parse().ok()?;
    let f1 = [&d[0..4], &d[19..24]].concat();
    let f2 = d[24..34].to_vec();
    let f3 = d[34..44].to_vec();
    let fmt = |f: &[u32], split: usize| {
        let dv = mod10(f);
        format!("{}.{}{dv}", s(&f[..split]), s(&f[split..]))
    };
    let line = format!(
        "{} {} {} {} {}",
        fmt(&f1, 5),
        fmt(&f2, 5),
        fmt(&f3, 5),
        d[4],
        s(&d[5..19])
    );
    let bank = s(&d[0..3]);
    Some(Bill {
        kind: BillKind::Bank,
        barcode: s(&d),
        line,
        amount: (amount > 0).then_some(amount),
        due_date: due_from_factor(factor, today).map(crate::time::fmt_date),
        bank_name: bank_name(&bank).map(str::to_string),
        bank_code: Some(bank),
        segment: None,
        beneficiary: None,
    })
}

/// Lê a linha digitável (47 ou 48 dígitos, com ou sem pontos e espaços), conferindo o DV de
/// cada campo e o geral.
pub fn from_line(line: &str, today: NaiveDate) -> Option<Bill> {
    let d: Vec<u32> = line.chars().filter_map(|c| c.to_digit(10)).collect();
    match d.len() {
        47 => {
            for (a, b) in [(0, 9), (10, 20), (21, 31)] {
                if mod10(&d[a..b]) != d[b] {
                    return None;
                }
            }
            let code = [
                &d[0..4],
                &d[32..33],
                &d[33..47],
                &d[4..9],
                &d[10..20],
                &d[21..31],
            ]
            .concat();
            from_barcode(&s(&code), today)
        }
        48 if d[0] == 8 => {
            let ten = matches!(d[2], 6 | 7);
            let mut code = vec![];
            for b in d.chunks(12) {
                let (body, dv) = (&b[..11], b[11]);
                let want = if ten {
                    mod10(body)
                } else {
                    mod11_collection(body)
                };
                if want != dv {
                    return None;
                }
                code.extend_from_slice(body);
            }
            from_barcode(&s(&code), today)
        }
        _ => None,
    }
}

/// Letras que o OCR confunde com algarismos, num trecho que é quase todo de números.
fn fix_digit(c: char) -> char {
    match c {
        'O' | 'o' | 'D' | 'Q' => '0',
        'I' | 'l' | '|' | 'i' | '!' => '1',
        'Z' | 'z' => '2',
        'S' | 's' => '5',
        'G' | 'b' => '6',
        'B' => '8',
        'g' => '9',
        c => c,
    }
}

/// Sequências de dígitos de um texto: os trechos com pontos, hífens e espaços entre os blocos
/// (como a linha digitável sai impressa) viram uma só.
fn digit_runs(text: &str) -> Vec<String> {
    let mut runs = vec![];
    let mut cur = String::new();
    for tok in text.split_whitespace() {
        let core: String = tok.chars().filter(|c| !matches!(c, '.' | '-')).collect();
        let digits = core.chars().filter(char::is_ascii_digit).count();
        // Um bloco da linha digitável: quase todo algarismo (o OCR troca um ou outro).
        if !core.is_empty() && digits * 10 >= core.chars().count() * 7 {
            cur.extend(core.chars().map(fix_digit).filter(char::is_ascii_digit));
        } else if !cur.is_empty() {
            runs.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        runs.push(cur);
    }
    runs
}

static DUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)venc[a-z.]*\D{0,20}?(\d{2})[/.-](\d{2})[/.-](\d{4})").unwrap()
});

/// Vencimento escrito no texto ("Vencimento 10/11/2026").
fn due_in_text(text: &str) -> Option<String> {
    DUE.captures_iter(text).find_map(|c| {
        NaiveDate::from_ymd_opt(c[3].parse().ok()?, c[2].parse().ok()?, c[1].parse().ok()?)
            .map(crate::time::fmt_date)
    })
}

/// Beneficiário ("Beneficiário: Escola Exemplo Ltda  CNPJ …"): o resto da linha do rótulo, ou
/// a linha de baixo quando o rótulo está sozinho.
fn beneficiary(rows: &[String]) -> Option<String> {
    super::labeled(rows, &["beneficiario", "cedente", "favorecido"])
}

/// Procura o boleto no texto do OCR: a linha digitável (47 ou 48 dígitos) ou os 44 do código
/// impressos, conferidos pelo DV. Uma linha, ou duas seguidas (a linha digitável quebra).
pub fn find_in_text(rows: &[String], today: NaiveDate) -> Option<Bill> {
    let mut texts: Vec<String> = rows.to_vec();
    for w in rows.windows(2) {
        texts.push(format!("{} {}", w[0], w[1]));
    }
    let mut found = None;
    'outer: for t in &texts {
        for run in digit_runs(t) {
            for len in [47, 48, 44] {
                if run.len() < len {
                    continue;
                }
                for i in 0..=run.len() - len {
                    let w = &run[i..i + len];
                    let b = if len == 44 {
                        from_barcode(w, today)
                    } else {
                        from_line(w, today)
                    };
                    if b.is_some() {
                        found = b;
                        break 'outer;
                    }
                }
            }
        }
    }
    let mut b = found?;
    let all = rows.join("\n");
    if b.due_date.is_none() {
        b.due_date = due_in_text(&all);
    }
    b.beneficiary = beneficiary(rows);
    Some(b)
}

/// A categoria padrão (chave de `locales/*/categorias.ftl`) pelo tipo do boleto.
pub fn category_seed(b: &Bill) -> &'static str {
    match (b.kind, b.segment) {
        (BillKind::Collection, Some(1)) => "categorias-iptu",
        (BillKind::Collection, Some(2)) => "categorias-agua",
        (BillKind::Collection, Some(3)) => "categorias-energia",
        (BillKind::Collection, Some(4)) => "categorias-telefone",
        (BillKind::Collection, Some(5)) => "categorias-impostos-e-taxas",
        (BillKind::Collection, Some(7)) => "categorias-juros-e-multas",
        _ => "categorias-contas-e-boletos",
    }
}
