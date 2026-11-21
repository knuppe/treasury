//! Comprovante de Pix, TED/DOC ou pagamento de boleto, pelo texto do OCR
//! (docs/ocr-other-documents.md › 3.4): valor, data e hora, quem recebeu, quem pagou, instituição
//! e o ID fim a fim do Pix (`E` + ISPB de 8 dígitos + `AAAAMMDDHHMM` + 11 caracteres), que
//! confere a leitura e já dá a data, a hora e o banco de origem. Os rótulos variam por banco
//! ("Para", "Destino", "Quem recebeu", "Favorecido"): um leitor genérico de rótulo e valor.
//! O CPF mascarado não é guardado.

use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::import::text::fold;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProofKind {
    Pix,
    /// TED, DOC ou transferência entre contas.
    Transfer,
    /// Pagamento de boleto ou conta.
    Bill,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Direction {
    /// Dinheiro que saiu (Pix enviado, pagamento).
    Out,
    /// Dinheiro que entrou (Pix recebido).
    In,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Proof {
    pub kind: ProofKind,
    pub direction: Direction,
    /// Valor em centavos (sempre positivo).
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[ts(optional)]
    pub date: Option<String>,
    /// Hora (`HH:MM`).
    #[ts(optional)]
    pub time: Option<String>,
    /// Quem recebeu (ou quem pagou, num Pix recebido).
    #[ts(optional)]
    pub counterpart: Option<String>,
    /// Instituição da outra ponta, quando o comprovante diz.
    #[ts(optional)]
    pub institution: Option<String>,
    /// ID fim a fim do Pix, conferido pelo formato.
    #[ts(optional)]
    pub e2e_id: Option<String>,
    /// Banco de origem pelo ISPB do ID fim a fim.
    #[ts(optional)]
    pub origin_bank: Option<String>,
    /// Código de barras do boleto pago (fecha o lançamento previsto).
    #[ts(optional)]
    pub barcode: Option<String>,
    /// Comprovante de **agendamento**: o dinheiro ainda não saiu da conta (vira previsto).
    pub scheduled: bool,
    /// Valor, data e (no Pix) o ID fim a fim lidos e coerentes.
    pub checked: bool,
}

static MONEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:R\$|RS|R5)?\s*(\d{1,3}(?:[.\s]\d{3})*,\d{2})\b").unwrap());
static DATE_NUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{2})/(\d{2})/(\d{4}|\d{2})\b").unwrap());
static DATE_WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d{1,2})\s+(?:de\s+)?(jan|fev|mar|abr|mai|jun|jul|ago|set|out|nov|dez)[a-zç]*\.?\s+(?:de\s+)?(\d{4})\b").unwrap()
});
static TIME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([01]\d|2[0-3])[:h]([0-5]\d)(?::[0-5]\d)?\b").unwrap());
static E2E: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[Ee](\d{8})(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})([A-Za-z0-9]{11})").unwrap()
});

fn cents(s: &str) -> Option<i64> {
    let clean: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    clean.parse().ok()
}

/// O primeiro valor de uma linha.
fn money_in(row: &str) -> Option<i64> {
    MONEY
        .captures(row)
        .and_then(|c| cents(&c[1]))
        .filter(|v| *v > 0)
}

fn date_in(row: &str) -> Option<NaiveDate> {
    if let Some(c) = DATE_NUM.captures(row) {
        let y: i32 = match c[3].len() {
            2 => 2000 + c[3].parse::<i32>().ok()?,
            _ => c[3].parse().ok()?,
        };
        return NaiveDate::from_ymd_opt(y, c[2].parse().ok()?, c[1].parse().ok()?);
    }
    let c = DATE_WORDS.captures(row)?;
    let m = [
        "jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez",
    ]
    .iter()
    .position(|m| c[2].eq_ignore_ascii_case(m))? as u32
        + 1;
    NaiveDate::from_ymd_opt(c[3].parse().ok()?, m, c[1].parse().ok()?)
}

/// Instituições mais comuns pelo ISPB (oito primeiros dígitos do CNPJ).
fn ispb_name(ispb: &str) -> Option<&'static str> {
    Some(match ispb {
        "00000000" => "Banco do Brasil",
        "00360305" => "Caixa",
        "00416968" => "Banco Inter",
        "60701190" => "Itaú",
        "60746948" => "Bradesco",
        "90400888" => "Santander",
        "18236120" => "Nubank",
        "31872495" => "C6 Bank",
        "10573521" => "Mercado Pago",
        "22896431" => "PicPay",
        "08561701" => "PagBank",
        "30306294" => "BTG Pactual",
        "92702067" => "Banrisul",
        "01181521" => "Sicredi",
        "02038232" => "Sicoob",
        _ => return None,
    })
}

/// ID fim a fim: o formato, com a data dele possível (as letras que o OCR confunde viram
/// algarismos só nas partes numéricas).
fn e2e(rows: &[String]) -> Option<(String, NaiveDate, String, String)> {
    let joined: String = rows
        .iter()
        .map(|r| r.split_whitespace().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ");
    for m in joined.split(|c: char| !c.is_ascii_alphanumeric()) {
        let Some(start) = m.find(['E', 'e']) else {
            continue;
        };
        let cand = &m[start..];
        if cand.len() < 32 {
            continue;
        }
        let cand = &cand[..32];
        // Os 20 primeiros depois do E são números.
        let fixed: String = cand
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if (1..=20).contains(&i) {
                    match c {
                        'O' | 'o' | 'D' | 'Q' => '0',
                        'I' | 'l' | 'i' => '1',
                        'S' | 's' => '5',
                        'B' => '8',
                        c => c,
                    }
                } else if i == 0 {
                    'E'
                } else {
                    c
                }
            })
            .collect();
        let Some(c) = E2E.captures(&fixed) else {
            continue;
        };
        let Some(d) =
            NaiveDate::from_ymd_opt(c[2].parse().ok()?, c[3].parse().ok()?, c[4].parse().ok()?)
        else {
            continue;
        };
        if !(2020..=2100).contains(&chrono::Datelike::year(&d)) {
            continue;
        }
        let hm = format!("{}:{}", &c[5], &c[6]);
        let ispb = c[1].to_string();
        return Some((fixed.clone(), d, hm, ispb));
    }
    None
}

/// Lê o comprovante. `rows` são as linhas do OCR; `barcode` o boleto achado no mesmo texto.
pub fn read(rows: &[String], barcode: Option<String>) -> Proof {
    let all = fold(&rows.join(" "));
    let has = |ws: &[&str]| ws.iter().any(|w| all.contains(w));
    let id = e2e(rows);
    let kind = if id.is_some() || has(&["pix"]) {
        ProofKind::Pix
    } else if barcode.is_some()
        || has(&[
            "boleto",
            "linha digitavel",
            "codigo de barras",
            "pagamento de conta",
        ])
    {
        ProofKind::Bill
    } else if has(&["ted", "doc", "transferencia"]) {
        ProofKind::Transfer
    } else {
        ProofKind::Other
    };
    let direction = if has(&["recebido", "recebida", "voce recebeu"])
        && !has(&[
            "enviado",
            "enviada",
            "voce enviou",
            "pagamento efetuado",
            "pagamento realizado",
        ]) {
        Direction::In
    } else {
        Direction::Out
    };
    // Valor: a linha do rótulo "valor" (ou a de baixo); senão o primeiro valor do texto.
    let mut amount = None;
    for (i, r) in rows.iter().enumerate() {
        let f = fold(r);
        if f.starts_with("valor")
            || f.contains("valor pago")
            || f.contains("valor da transferencia")
            || f.contains("valor do pix")
            || f.contains("valor total")
        {
            amount = money_in(r).or_else(|| rows.get(i + 1).and_then(|n| money_in(n)));
            if amount.is_some() {
                break;
            }
        }
    }
    let amount = amount.or_else(|| rows.iter().find_map(|r| money_in(r)));
    let mut date = rows.iter().find_map(|r| date_in(r));
    let mut time = rows
        .iter()
        .find_map(|r| TIME.captures(r).map(|c| format!("{}:{}", &c[1], &c[2])));
    let mut origin_bank = None;
    let e2e_id = id.map(|(code, d, hm, ispb)| {
        // O ID fim a fim vale mais que a data impressa (que o OCR pode ter lido errado);
        // a hora dele é UTC, então a impressa fica quando houver.
        if date.is_none_or(|x| (x - d).num_days().abs() > 1) {
            date = Some(d);
        }
        if time.is_none() {
            time = Some(hm);
        }
        origin_bank = ispb_name(&ispb).map(str::to_string);
        code
    });
    let (who_labels, inst_labels): (&[&str], &[&str]) = match direction {
        Direction::Out => (
            &[
                "quem recebeu",
                "destinatario",
                "recebedor",
                "favorecido",
                "beneficiario",
                "destino",
                "para",
            ],
            &["instituicao", "banco"],
        ),
        Direction::In => (
            &[
                "quem pagou",
                "quem enviou",
                "pagador",
                "remetente",
                "origem",
                "de",
            ],
            &["instituicao", "banco"],
        ),
    };
    let counterpart = super::labeled(rows, who_labels);
    let institution = super::labeled_after(rows, who_labels, inst_labels);
    let scheduled = has(&[
        "agendamento",
        "agendado",
        "agendada",
        "pagamento programado",
    ]) && !has(&[
        "agendamento efetivado",
        "agendamento realizado com sucesso e debitado",
    ]);
    let checked =
        amount.is_some() && date.is_some() && (kind != ProofKind::Pix || e2e_id.is_some());
    Proof {
        kind,
        direction,
        amount,
        date: date.map(crate::time::fmt_date),
        time,
        counterpart,
        institution,
        e2e_id,
        origin_bank,
        barcode,
        scheduled,
        checked,
    }
}
