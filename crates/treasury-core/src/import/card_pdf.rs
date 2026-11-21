//! Fatura de cartão em PDF (docs/card-statement-pdf.md): o texto do PDF vira lançamentos do
//! cartão, ligados à fatura do vencimento impresso e com o total da fatura para conferir.
//!
//! O layout `card.generico.v1` lê as linhas no formato comum às faturas brasileiras:
//! data (`12/10`, `12/10/26` ou `12 OUT`), descrição e valor no fim (`1.234,56`, com `-`
//! ou `CR` nos créditos). O ano vem do vencimento. Parcelas (`02/10`, `PARC 2/10`) ficam
//! na descrição e vão para o mês em que são cobradas. Pagamentos da fatura anterior ficam
//! de fora (entram pela conta que pagou), e totais, subtotais e saldos são ignorados.
//! Cartões adicionais: as faturas separam os lançamentos em blocos por cartão ("Cartão final
//! 1234", "**** 1234 - NOME"); o final do bloco vai em cada linha dele.
//! Layouts próprios de cada banco entram aqui quando houver um PDF de exemplo.

use std::sync::OnceLock;

use chrono::{Datelike, NaiveDate};
use regex::Regex;

use crate::import::text::{fold, fold_keep_punct};
use crate::time::{add_months, clamp_day};

#[derive(Debug, Clone, PartialEq)]
pub struct CardPdfRow {
    pub line: u32,
    pub date: NaiveDate,
    pub description: String,
    /// Com sinal: compra negativa, estorno ou crédito positivo.
    pub amount: i64,
    pub installment: Option<(u32, u32)>,
    /// Final do cartão do bloco em que a linha está (adicionais), se a fatura separar.
    pub card_last4: Option<String>,
    /// Compra no exterior: valor e moeda de origem impressos na descrição ("USD 80,00").
    pub orig: Option<(i64, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CardPdf {
    pub issuer: Option<String>,
    pub due_date: Option<NaiveDate>,
    /// Total da fatura impresso (positivo = a pagar).
    pub total: Option<i64>,
    pub rows: Vec<CardPdfRow>,
    /// Pagamentos da fatura anterior ignorados.
    pub payments_skipped: u32,
    /// Pagamento mínimo, limite total e saldo da fatura anterior impressos (para conferir).
    pub minimum: Option<i64>,
    pub credit_limit: Option<i64>,
    pub previous_balance: Option<i64>,
}

const ISSUERS: &[(&str, &str)] = &[
    ("nu pagamentos", "Nubank"),
    ("nubank", "Nubank"),
    ("itau", "Itaú"),
    ("bradesco", "Bradesco"),
    ("santander", "Santander"),
    ("banco inter", "Inter"),
    ("c6 bank", "C6 Bank"),
    ("ourocard", "Banco do Brasil"),
    ("banco do brasil", "Banco do Brasil"),
    ("caixa economica", "Caixa"),
    ("btg", "BTG Pactual"),
    ("xp visa", "XP"),
    ("mercado pago", "Mercado Pago"),
    ("picpay", "PicPay"),
    ("porto seguro", "Porto Seguro"),
    ("sicredi", "Sicredi"),
    ("sicoob", "Sicoob"),
    ("neon", "Neon"),
];

/// Parece uma fatura de cartão (marcadores do assunto, sem layout).
pub fn looks_like(text: &str) -> bool {
    let t = fold(text);
    t.contains("fatura")
        && (t.contains("cartao") || t.contains("vencimento"))
        && !t.contains("nota de corretagem")
        && !t.contains("extrato financeiro do participante")
}

fn line_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)^(?P<d>\d{1,2})(?:[/.-](?P<m>\d{1,2})(?:[/.-](?P<y>\d{2}|\d{4}))?|\s+(?P<mn>jan|fev|mar|abr|mai|jun|jul|ago|set|out|nov|dez)[a-z]*)\s+(?P<desc>.+?)\s+(?P<sign>[-−]\s*)?(?:R\$\s*)?(?P<amt>\d{1,3}(?:\.\d{3})*,\d{2})(?:\s*(?P<suf>-|(?:cr|c|d)\b))?\s*$",
        )
        .expect("regex da linha")
    })
}

/// Cabeçalho de bloco de um cartão ("Cartão final 1234", "FINAL 1234 - NOME", "**** 1234"):
/// devolve o final.
fn section_last4(line: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?i)(?:\bfinal\s*(?:n[ºo°.]?\s*)?[:\-]?\s*|(?:[x*•]{4}[\s.-]?){1,3})(\d{4})\b")
            .expect("regex do final do cartão")
    });
    let f = fold(line);
    if !(f.contains("final") || f.contains("cartao") || line.contains("****") || f.contains("xxxx"))
    {
        return None;
    }
    re.captures(line).map(|c| c[1].to_string())
}

/// Valor de origem na descrição: "AMAZON USD 80,00", "US$ 12,50", "EUR 1.200,00".
fn orig_in(desc: &str) -> Option<(i64, String)> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?i)\b(USD|US\$|EUR|GBP|ARS|CLP|CAD|CHF|JPY|UYU|PYG|MXN|COP|PEN|AUD)\s*(\d{1,3}(?:\.\d{3})*,\d{2})\b")
            .expect("regex da moeda de origem")
    });
    let c = re.captures(desc)?;
    let cur = match c[1].to_ascii_uppercase().as_str() {
        "US$" => "USD".to_string(),
        x => x.to_string(),
    };
    Some((amount(&c[2])?, cur))
}

fn installment_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(?:parc(?:ela)?\.?\s*)?\b(\d{1,2})\s*(?:/|de)\s*(\d{1,2})\s*$")
            .expect("regex da parcela")
    })
}

fn amount(s: &str) -> Option<i64> {
    crate::money::parse_amount(s, "BRL").ok()
}

/// Data `DD/MM/AAAA` depois de um rótulo (texto já em minúsculas e sem acentos).
fn date_after(t: &str, labels: &[&str]) -> Option<NaiveDate> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(\d{2})[/.](\d{2})[/.](\d{4})").expect("regex"));
    for l in labels {
        let mut from = 0;
        while let Some(i) = t[from..].find(l) {
            let start = from + i + l.len();
            let window: String = t[start..].chars().take(60).collect();
            if let Some(c) = re.captures(&window) {
                let (d, m, y) = (c[1].parse().ok()?, c[2].parse().ok()?, c[3].parse().ok()?);
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
            from = start;
        }
    }
    None
}

/// Primeiro valor depois de um dos rótulos (na ordem dada).
fn amount_after(t: &str, labels: &[&str]) -> Option<i64> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?:r\$\s*)?(\d{1,3}(?:\.\d{3})*,\d{2})").expect("regex do total")
    });
    for l in labels {
        if let Some(i) = t.find(l) {
            let window: String = t[i + l.len()..].chars().take(50).collect();
            if let Some(c) = re.captures(&window) {
                return amount(&c[1]);
            }
        }
    }
    None
}

fn total_after(t: &str) -> Option<i64> {
    amount_after(
        t,
        &[
            "total desta fatura",
            "total da fatura",
            "total da sua fatura",
            "valor total da fatura",
            "total a pagar",
            "valor total",
        ],
    )
}

fn month_num(name: &str) -> Option<u32> {
    let n = fold(name);
    [
        "jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez",
    ]
    .iter()
    .position(|m| n.starts_with(m))
    .map(|i| i as u32 + 1)
}

/// Linhas que não são lançamentos.
fn ignored(desc: &str) -> bool {
    let f = fold(desc);
    [
        "saldo anterior",
        "total",
        "subtotal",
        "limite",
        "saldo em aberto",
        "pagamento minimo",
        "valor minimo",
    ]
    .iter()
    .any(|k| f.starts_with(k))
}

fn is_payment(desc: &str) -> bool {
    let f = fold(desc);
    f.starts_with("pagamento") || f.starts_with("pagto") || f.contains("pagamento recebido")
}

pub fn parse(text: &str) -> CardPdf {
    let low = fold_keep_punct(text);
    let due_date = date_after(&low, &["vencimento", "vence em", "data de vencimento"]);
    let folded = fold(text);
    let issuer = ISSUERS
        .iter()
        .find(|(k, _)| folded.contains(k))
        .map(|(_, v)| v.to_string());
    let total = total_after(&low);
    let minimum = amount_after(
        &low,
        &[
            "pagamento minimo",
            "valor minimo",
            "minimo a pagar",
            "pagamento mínimo",
        ],
    );
    let credit_limit = amount_after(
        &low,
        &[
            "limite total",
            "limite de credito",
            "limite do cartao",
            "seu limite",
        ],
    );
    let previous_balance = amount_after(
        &low,
        &[
            "total da fatura anterior",
            "fatura anterior",
            "saldo anterior",
        ],
    );
    let mut rows = vec![];
    let mut payments_skipped = 0;
    let mut section: Option<String> = None;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let Some(c) = line_re().captures(line) else {
            if let Some(l) = section_last4(line) {
                section = Some(l);
            }
            continue;
        };
        let desc = c["desc"].trim().to_string();
        if ignored(&desc) {
            continue;
        }
        if is_payment(&desc) {
            payments_skipped += 1;
            continue;
        }
        let Some(value) = amount(&c["amt"]) else {
            continue;
        };
        let day: u32 = c["d"].parse().unwrap_or(0);
        let month = match (c.name("m"), c.name("mn")) {
            (Some(m), _) => m.as_str().parse().unwrap_or(0),
            (None, Some(n)) => month_num(n.as_str()).unwrap_or(0),
            _ => 0,
        };
        if !(1..=31).contains(&day) || !(1..=12).contains(&month) {
            continue;
        }
        let year = match c.name("y") {
            Some(y) if y.as_str().len() == 2 => 2000 + y.as_str().parse::<i32>().unwrap_or(0),
            Some(y) => y.as_str().parse().unwrap_or(0),
            // Sem ano: o do vencimento, ou o anterior se o mês passa do vencimento.
            None => match due_date {
                Some(d) if month > d.month() => d.year() - 1,
                Some(d) => d.year(),
                None => chrono::Local::now().year(),
            },
        };
        let mut date = clamp_day(year, month, day);
        let credit = c.name("sign").is_some()
            || c.name("suf").is_some_and(|s| {
                let s = s.as_str().to_ascii_lowercase();
                s == "-" || s == "cr" || s == "c"
            })
            || {
                let f = fold(&desc);
                f.contains("estorno") || f.starts_with("credito") || f.contains("devolucao")
            };
        let installment = installment_re().captures(&desc).and_then(|p| {
            let (k, n): (u32, u32) = (p[1].parse().ok()?, p[2].parse().ok()?);
            (k >= 1 && n >= 2 && k <= n && n <= 72).then_some((k, n))
        });
        // A parcela k é cobrada k−1 meses depois da compra.
        if let Some((k, _)) = installment
            && k > 1
        {
            date = add_months(date, k as i32 - 1);
        }
        rows.push(CardPdfRow {
            line: i as u32 + 1,
            date,
            description: desc.chars().take(300).collect(),
            amount: if credit { value } else { -value },
            installment,
            card_last4: section.clone(),
            orig: orig_in(&desc),
        });
    }
    CardPdf {
        issuer,
        due_date,
        total,
        rows,
        payments_skipped,
        minimum,
        credit_limit,
        previous_balance,
    }
}

/// O layout genérico reconhece o arquivo: parece fatura e tem lançamentos legíveis.
pub fn matches_generic(text: &str) -> bool {
    looks_like(text) && !parse(text).rows.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
BANCO EXEMPLO S.A. - CARTÃO DE CRÉDITO
Fatura de outubro
Vencimento: 10/11/2026     Total desta fatura R$ 1.391,40
Lançamentos
28/09 PAGAMENTO RECEBIDO 980,00 -
02/10 SUPERMERCADO BOA COMPRA 312,45
05/10 NETFLIX.COM 44,90
15 OUT POSTO SHELL 250,00
20/12/2025 LOJA DE MÓVEIS 11/12 300,00
18/10 ESTORNO LOJA X 30,00 CR
21/10 IFOOD *RESTAURANTE 98,05
Saldo anterior 980,00
Total dos lançamentos 975,40
22/10 IOF COMPRA INTERNACIONAL 4,00
22/10 AMAZON USD 80,00 412,00
";

    #[test]
    fn reads_the_generic_layout() {
        assert!(looks_like(SAMPLE));
        let p = parse(SAMPLE);
        assert_eq!(p.due_date, NaiveDate::from_ymd_opt(2026, 11, 10));
        assert_eq!(p.total, Some(139_140));
        assert_eq!(p.payments_skipped, 1);
        let got: Vec<(String, &str, i64)> = p
            .rows
            .iter()
            .map(|r| (r.date.to_string(), r.description.as_str(), r.amount))
            .collect();
        assert_eq!(
            got,
            [
                ("2026-10-02".to_string(), "SUPERMERCADO BOA COMPRA", -31_245),
                ("2026-10-05".to_string(), "NETFLIX.COM", -4_490),
                ("2026-10-15".to_string(), "POSTO SHELL", -25_000),
                // Parcela 11 de 12 de uma compra de dezembro: cobrada em outubro.
                ("2026-10-20".to_string(), "LOJA DE MÓVEIS 11/12", -30_000),
                ("2026-10-18".to_string(), "ESTORNO LOJA X", 3_000),
                ("2026-10-21".to_string(), "IFOOD *RESTAURANTE", -9_805),
                ("2026-10-22".to_string(), "IOF COMPRA INTERNACIONAL", -400),
                ("2026-10-22".to_string(), "AMAZON USD 80,00", -41_200),
            ]
        );
        assert_eq!(p.rows[3].installment, Some((11, 12)));
        assert_eq!(p.rows[7].orig, Some((8_000, "USD".to_string())));
        assert_eq!(p.rows[0].orig, None);
        // As compras menos o estorno batem com o total impresso.
        assert_eq!(-p.rows.iter().map(|r| r.amount).sum::<i64>(), 139_140);
        assert!(matches_generic(SAMPLE));
        assert!(!matches_generic(
            "Extrato de conta corrente\n02/10 PIX 10,00"
        ));
    }

    #[test]
    fn reads_minimum_limit_and_previous_balance() {
        let p = parse(
            "Fatura do cartão\nVencimento 05/11/2026\nTotal desta fatura R$ 1.000,00\n\
             Pagamento mínimo R$ 150,00\nLimite total R$ 8.000,00   Limite disponível R$ 7.000,00\n\
             Total da fatura anterior R$ 900,00\n02/10 PADARIA 1.000,00",
        );
        assert_eq!(
            (p.minimum, p.credit_limit, p.previous_balance),
            (Some(15_000), Some(800_000), Some(90_000))
        );
        let p = parse(SAMPLE);
        assert_eq!((p.minimum, p.credit_limit), (None, None));
        assert_eq!(p.previous_balance, Some(98_000), "saldo anterior");
    }

    #[test]
    fn reads_the_card_of_each_block() {
        let p = parse(
            "Fatura do cartão\nVencimento 05/11/2026\nCartão final 1111 - ANA EXEMPLO\n\
             02/10 PADARIA 10,00\n**** **** **** 2222 BRUNO EXEMPLO\n03/10 FARMACIA 20,00\n\
             Subtotal 20,00\nCARTAO VIRTUAL FINAL 3333\n04/10 STREAMING 30,00",
        );
        let got: Vec<Option<&str>> = p.rows.iter().map(|r| r.card_last4.as_deref()).collect();
        assert_eq!(got, [Some("1111"), Some("2222"), Some("3333")]);
        // Sem blocos, nenhuma linha tem final.
        assert!(parse(SAMPLE).rows.iter().all(|r| r.card_last4.is_none()));
    }

    #[test]
    fn purchases_after_the_due_month_are_from_the_year_before() {
        let p = parse(
            "Fatura do cartão\nVencimento 05/01/2027\n20/12 PRESENTE 100,00\n02/01 PADARIA 10,00",
        );
        assert_eq!(p.rows[0].date.to_string(), "2026-12-20");
        assert_eq!(p.rows[1].date.to_string(), "2027-01-02");
    }
}
