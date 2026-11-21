//! Norma 43 (Cuaderno 43 da AEB/CSB): o extrato de conta que todo banco espanhol exporta
//! (docs/international.md). Registros de 80 posições: `11` abre a conta (banco, agência,
//! conta, período e saldo inicial), `22` é um movimento, `23` traz os conceitos
//! complementares do movimento anterior, `33` fecha a conta (totais e saldo final) e `88` fecha
//! o arquivo. Vira o mesmo documento do OFX, para a importação seguir o mesmo caminho.

use chrono::NaiveDate;

use super::ofx::{OfxDocument, OfxStatement, OfxTxn};
use crate::error::{Error, Result};
use crate::i18n::t;

/// Parece um arquivo Norma 43: a primeira linha é um `11` e há movimentos `22`.
pub fn looks_like(text: &str) -> bool {
    let mut lines = text.lines().map(str::trim_end).filter(|l| !l.is_empty());
    let Some(first) = lines.next() else {
        return false;
    };
    first.starts_with("11")
        && first.len() >= 50
        && first[2..20].chars().all(|c| c.is_ascii_digit())
        && lines.any(|l| l.starts_with("22") || l.starts_with("33"))
}

/// Campo de posição fixa (sem quebrar em caractere de mais de um byte).
fn field(line: &str, from: usize, to: usize) -> &str {
    let to = to.min(line.len());
    if from >= to {
        return "";
    }
    line.get(from..to).unwrap_or("").trim()
}

/// `AAMMDD` → data (anos 00–69 = 2000–2069).
fn date(s: &str) -> Option<NaiveDate> {
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let y: i32 = s[0..2].parse().ok()?;
    let year = if y < 70 { 2000 + y } else { 1900 + y };
    NaiveDate::from_ymd_opt(year, s[2..4].parse().ok()?, s[4..6].parse().ok()?)
}

/// Valor de 14 dígitos com 2 decimais e a chave débito (`1`) ou crédito (`2`), como texto
/// com sinal ("-12.34").
fn amount(digits: &str, debit_credit: &str) -> Option<String> {
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let cents: i64 = digits.parse().ok()?;
    let sign = if debit_credit == "1" { "-" } else { "" };
    Some(format!("{sign}{}.{:02}", cents / 100, cents % 100))
}

/// Moeda pelo código numérico ISO 4217.
fn currency(code: &str) -> Option<String> {
    Some(
        match code {
            "978" => "EUR",
            "840" => "USD",
            "826" => "GBP",
            "756" => "CHF",
            "986" => "BRL",
            _ => return None,
        }
        .to_string(),
    )
}

/// Sem referência nem conceito complementar, o nome do conceito comum.
fn finish(mut s: OfxStatement) -> OfxStatement {
    for t in &mut s.txns {
        let common = t.ref_num.take().unwrap_or_default();
        if t.name.is_none() {
            t.name = Some(concept_name(&common));
        }
    }
    s
}

pub fn parse(text: &str) -> Result<OfxDocument> {
    let mut doc = OfxDocument::default();
    let mut cur: Option<OfxStatement> = None;
    for raw in text.lines() {
        let line = raw.trim_end_matches(['\r', '\n']);
        if line.len() < 2 {
            continue;
        }
        match &line[..2] {
            "11" => {
                if let Some(s) = cur.take() {
                    doc.statements.push(finish(s));
                }
                cur = Some(OfxStatement {
                    credit_card: false,
                    currency: currency(field(line, 47, 50)),
                    bank_id: Some(field(line, 2, 6).to_string()),
                    branch_id: Some(field(line, 6, 10).to_string()),
                    account_id: Some(field(line, 10, 20).to_string()),
                    account_type: Some("CHECKING".into()),
                    date_start: date(field(line, 20, 26)),
                    date_end: date(field(line, 26, 32)),
                    ..Default::default()
                });
            }
            "22" => {
                let Some(s) = cur.as_mut() else { continue };
                let op = field(line, 10, 16);
                let common = field(line, 22, 24);
                let doc_no = field(line, 42, 52);
                let refs: Vec<&str> = [field(line, 52, 64), field(line, 64, 80)]
                    .into_iter()
                    .filter(|r| !r.is_empty() && !r.chars().all(|c| c == '0'))
                    .collect();
                let raw_amount = amount(field(line, 28, 42), field(line, 27, 28));
                // Identificador estável: data, valor, documento, referências e a ordem no dia.
                let base = format!(
                    "n43-{}-{op}-{}-{doc_no}-{}",
                    s.account_id.as_deref().unwrap_or(""),
                    field(line, 27, 42),
                    refs.join("-")
                );
                let seq = s
                    .txns
                    .iter()
                    .filter(|t| {
                        t.fitid
                            .as_deref()
                            .is_some_and(|f| f.starts_with(&format!("{base}#")))
                    })
                    .count();
                s.txns.push(OfxTxn {
                    trn_type: Some(if field(line, 27, 28) == "1" {
                        "DEBIT".into()
                    } else {
                        "CREDIT".into()
                    }),
                    date: date(op),
                    date_raw: Some(op.to_string()),
                    amount_raw: raw_amount,
                    fitid: Some(format!("{base}#{seq}")),
                    check_num: (!doc_no.is_empty() && !doc_no.chars().all(|c| c == '0'))
                        .then(|| doc_no.to_string()),
                    // O conceito comum, até o fim (vira o nome se nada melhor vier).
                    ref_num: Some(common.to_string()),
                    memo: None,
                    // As referências do banco; o conceito complementar (`23`) vem por cima.
                    name: (!refs.is_empty()).then(|| refs.join(" ")),
                    payee: None,
                });
            }
            "23" => {
                let Some(t) = cur.as_mut().and_then(|s| s.txns.last_mut()) else {
                    continue;
                };
                let text = [field(line, 4, 42), field(line, 42, 80)]
                    .into_iter()
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                if text.is_empty() {
                    continue;
                }
                // O primeiro conceito vira o nome (as referências passam para a observação); os
                // seguintes, a observação.
                if field(line, 2, 4) == "01" {
                    t.memo = t.name.take();
                    t.name = Some(text);
                } else {
                    t.memo = Some(match t.memo.take() {
                        Some(m) => format!("{m} {text}"),
                        None => text,
                    });
                }
            }
            "33" => {
                if let Some(s) = cur.as_mut() {
                    s.ledger_balance_raw = amount(field(line, 59, 73), field(line, 58, 59));
                    s.balance_date = s.date_end;
                    if s.currency.is_none() {
                        s.currency = currency(field(line, 73, 76));
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(s) = cur.take() {
        doc.statements.push(finish(s));
    }
    if doc.statements.is_empty() {
        return Err(Error::invalid(t("nucleo-importacao-erro-norma43")));
    }
    Ok(doc)
}

/// Nome do conceito comum da AEB (quando o banco não manda texto).
fn concept_name(code: &str) -> String {
    let key = match code {
        "01" => "nucleo-importacao-n43-reintegro",
        "02" => "nucleo-importacao-n43-abonare",
        "03" => "nucleo-importacao-n43-recibo",
        "04" => "nucleo-importacao-n43-transferencia",
        "12" => "nucleo-importacao-n43-tarjeta",
        "15" => "nucleo-importacao-n43-nomina",
        "17" => "nucleo-importacao-n43-comision",
        _ => "nucleo-importacao-descricao-padrao",
    };
    t(key)
}

/// Extrato inventado (testes): banco 0000, agência 1111, conta 2222222222, outubro de 2026.
#[cfg(test)]
pub(crate) fn sample() -> String {
    let lines = [
        format!(
            "11{:4}{:4}{:10}{}{}{}{:014}{}{}{:26}{:3}",
            "0000",
            "1111",
            "2222222222",
            "261001",
            "261031",
            "2",
            150_000,
            "978",
            "3",
            "FAMILIA EJEMPLO",
            ""
        ),
        format!(
            "22{:4}{:4}{}{}{}{}{}{:014}{:010}{:12}{:16}",
            "", "1111", "261005", "261005", "12", "000", "1", 2_340, 0, "", ""
        ),
        format!(
            "23{}{:38}{:38}",
            "01", "COMPRA TARJ. MERCADONA VALENCIA", ""
        ),
        format!(
            "22{:4}{:4}{}{}{}{}{}{:014}{:010}{:12}{:16}",
            "",
            "1111",
            "261027",
            "261027",
            "15",
            "000",
            "2",
            185_000,
            0,
            "NOMINA",
            "EMPRESA EJEMPLO"
        ),
        format!(
            "22{:4}{:4}{}{}{}{}{}{:014}{:010}{:12}{:16}",
            "", "1111", "261027", "261027", "04", "000", "1", 2_340, 0, "", ""
        ),
        format!("23{}{:38}{:38}", "01", "BIZUM ENVIADO", "CENA"),
        format!(
            "33{:4}{:4}{:10}{:05}{:014}{:05}{:014}{}{:014}{}{:4}",
            "0000", "1111", "2222222222", 2, 4_680, 1, 185_000, "2", 330_320, "978", ""
        ),
        format!("88{:78}", ""),
    ];
    lines.join("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_invented_norma43_statement() {
        let text = sample();
        assert!(looks_like(&text));
        assert!(!looks_like("Fecha;Concepto;Importe\n01/10/2026;X;1,00\n"));
        let doc = parse(&text).unwrap();
        assert_eq!(doc.statements.len(), 1);
        let s = &doc.statements[0];
        assert_eq!(s.account_id.as_deref(), Some("2222222222"));
        assert_eq!(s.currency.as_deref(), Some("EUR"));
        assert_eq!(s.ledger_balance_raw.as_deref(), Some("3303.20"));
        assert_eq!(s.date_end, NaiveDate::from_ymd_opt(2026, 10, 31));
        assert_eq!(s.txns.len(), 3);
        let t0 = &s.txns[0];
        assert_eq!(t0.amount_raw.as_deref(), Some("-23.40"));
        assert_eq!(t0.name.as_deref(), Some("COMPRA TARJ. MERCADONA VALENCIA"));
        assert_eq!(t0.date, NaiveDate::from_ymd_opt(2026, 10, 5));
        let t1 = &s.txns[1];
        assert_eq!(t1.amount_raw.as_deref(), Some("1850.00"));
        assert_eq!(t1.name.as_deref(), Some("NOMINA EMPRESA EJEMPLO"));
        // Identificadores diferentes e estáveis (a mesma leitura dá os mesmos).
        let t2 = &s.txns[2];
        assert_eq!(t2.name.as_deref(), Some("BIZUM ENVIADO CENA"));
        assert_eq!(t2.memo, None);
        assert_ne!(t0.fitid, t2.fitid);
        assert_eq!(parse(&text).unwrap().statements[0].txns[2].fitid, t2.fitid);
    }
}
