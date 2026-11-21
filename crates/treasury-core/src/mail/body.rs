//! Layouts do corpo do e-mail (fase 3, docs/email-ingest.md): aviso de compra no cartão e
//! aviso de Pix recebido ou enviado, e o modelo do usuário (uma expressão com os grupos
//! `valor`, `descricao` e `data`). Cada um vira o rascunho de um lançamento; quem grava é a
//! revisão (ou "aplicar sozinho", com remetente autenticado).

use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::i18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "MailDraftKind")]
pub enum DraftKind {
    /// Compra no cartão (saída).
    Purchase,
    PixIn,
    PixOut,
    /// Pelo modelo da regra.
    Template,
}

/// O lançamento que o aviso descreve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailTxnDraft")]
pub struct TxnDraft {
    pub kind: DraftKind,
    /// Centavos com sinal (saída negativa).
    #[ts(type = "number")]
    pub amount: i64,
    /// `YYYY-MM-DD` (do aviso ou da mensagem).
    pub date: Option<String>,
    pub description: String,
    /// Final do cartão, quando o aviso traz.
    pub card_last4: Option<String>,
    /// Conta escolhida (pelo final do cartão ou pelo modelo).
    pub account_id: Option<String>,
}

/// Modelo do usuário para o corpo (por regra).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailBodyTemplate")]
pub struct BodyTemplate {
    /// Expressão com os grupos nomeados `valor` (obrigatório), `descricao` e `data`.
    pub pattern: String,
    /// `expense` (padrão) ou `income`.
    #[serde(default)]
    #[ts(optional)]
    pub direction: Option<String>,
    /// Conta dos lançamentos.
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<String>,
}

fn money_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"R\$\s*(\d{1,3}(?:\.\d{3})*,\d{2}|\d+,\d{2})").expect("regex"))
}

fn cents(s: &str) -> Option<i64> {
    crate::money::parse_amount(s.trim(), "BRL").ok()
}

/// Data `dd/mm/aaaa` ou `dd/mm` (com o ano da mensagem).
fn date_in(text: &str, year: Option<i32>) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\b(\d{2})/(\d{2})(?:/(\d{4}))?\b").expect("regex"));
    let c = re.captures(text)?;
    let (d, m): (u32, u32) = (c[1].parse().ok()?, c[2].parse().ok()?);
    let y: i32 = match c.get(3) {
        Some(y) => y.as_str().parse().ok()?,
        None => year?,
    };
    chrono::NaiveDate::from_ymd_opt(y, m, d).map(crate::time::fmt_date)
}

fn clean_name(s: &str) -> String {
    s.trim()
        .trim_end_matches(['.', ',', ';'])
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect()
}

/// Aviso de compra no cartão ("Compra de R$ 45,90 aprovada em PADARIA X no cartão final 1234").
fn purchase(text: &str, year: Option<i32>) -> Option<TxnDraft> {
    static CTX: OnceLock<Regex> = OnceLock::new();
    let ctx = CTX.get_or_init(|| {
        Regex::new(r"(?i)\b(compra|transa[cç][aã]o)\b.{0,40}\b(aprovad[ao]|realizad[ao]|efetuad[ao])\b|\bvoc[eê] fez uma compra\b|\bcompra no cr[eé]dito\b|\bcompra no d[eé]bito\b")
            .expect("regex")
    });
    if !ctx.is_match(text) {
        return None;
    }
    let m = money_re().captures(text)?;
    let amount = cents(&m[1])?;
    static MERCHANT: OnceLock<Regex> = OnceLock::new();
    let merchant = MERCHANT.get_or_init(|| {
        Regex::new(r"(?i)\b(?:em|no estabelecimento|estabelecimento:?|local:?)\s+([A-Z0-9][A-Z0-9 *&'.\-/]{2,60}?)(?:\s+(?:no|com o|com|para o|pelo)\s+cart[aã]o|\s+em\s+\d{2}/|\s+às|\s+as\s+\d|[.,;\n]|$)")
            .expect("regex")
    });
    // O estabelecimento vem depois do valor, na maioria dos avisos.
    let after = &text[m.get(0)?.end()..];
    let description = merchant
        .captures(after)
        .or_else(|| merchant.captures(text))
        .map(|c| clean_name(&c[1]))
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| i18n::t("nucleo-email-descricao-compra"));
    static FINAL: OnceLock<Regex> = OnceLock::new();
    let fin = FINAL.get_or_init(|| {
        Regex::new(r"(?i)\b(?:final|terminado em|com final|com o final)\s*:?\s*(\d{4})\b")
            .expect("regex")
    });
    Some(TxnDraft {
        kind: DraftKind::Purchase,
        amount: -amount.abs(),
        date: date_in(text, year),
        description,
        card_last4: fin.captures(text).map(|c| c[1].to_string()),
        account_id: None,
    })
}

/// Aviso de Pix ("Você recebeu um Pix de R$ 100,00 de FULANO", "Pix enviado de R$ 50,00 para…").
fn pix(text: &str, year: Option<i32>) -> Option<TxnDraft> {
    static IN: OnceLock<Regex> = OnceLock::new();
    let inn = IN.get_or_init(|| {
        Regex::new(r"(?i)\b(?:voc[eê] recebeu um pix|pix recebido|recebimento de pix|transfer[eê]ncia pix recebida)\b").expect("regex")
    });
    static OUT: OnceLock<Regex> = OnceLock::new();
    let out = OUT.get_or_init(|| {
        Regex::new(r"(?i)\b(?:voc[eê] (?:fez|enviou) um pix|pix enviado|pix realizado|comprovante de pix|transfer[eê]ncia pix enviada)\b").expect("regex")
    });
    let incoming = inn.is_match(text);
    if !incoming && !out.is_match(text) {
        return None;
    }
    let m = money_re().captures(text)?;
    let amount = cents(&m[1])?;
    static WHO: OnceLock<Regex> = OnceLock::new();
    let who = WHO.get_or_init(|| {
        Regex::new(r"(?i)\b(?:de|para|pagador:?|recebedor:?|destinat[aá]rio:?)\s+([A-ZÀ-Ú][A-Za-zÀ-ú.' ]{2,60}?)(?:\s+(?:em|no dia|às|as|via|com)\b|[.,;\n]|$)")
            .expect("regex")
    });
    let after = &text[m.get(0)?.end()..];
    let name = who.captures(after).map(|c| clean_name(&c[1]));
    // Descrição no idioma da instalação (fica gravada no lançamento).
    let named = |key: &str, n: String| i18n::t_args(key, &i18n::args(&[("nome", n.into())]));
    let description = match (incoming, name) {
        (true, Some(n)) => named("nucleo-email-descricao-pix-recebido-de", n),
        (false, Some(n)) => named("nucleo-email-descricao-pix-para", n),
        (true, None) => i18n::t("nucleo-email-descricao-pix-recebido"),
        (false, None) => i18n::t("nucleo-email-descricao-pix-enviado"),
    };
    Some(TxnDraft {
        kind: if incoming {
            DraftKind::PixIn
        } else {
            DraftKind::PixOut
        },
        amount: if incoming {
            amount.abs()
        } else {
            -amount.abs()
        },
        date: date_in(text, year),
        description,
        card_last4: None,
        account_id: None,
    })
}

/// O aviso que o corpo descreve, pelos layouts prontos.
pub fn read(body: &str, year: Option<i32>) -> Option<TxnDraft> {
    // Os avisos são curtos; o resto (propaganda, rodapé) só atrapalha.
    let text: String = body.chars().take(4000).collect();
    pix(&text, year).or_else(|| purchase(&text, year))
}

/// Confere o modelo ao salvar (grupo `valor` obrigatório).
pub fn validate_template(t: &BodyTemplate) -> Result<()> {
    let re = super::filter::compile_regex(&t.pattern)?;
    if !re.capture_names().any(|n| n == Some("valor")) {
        return Err(crate::error::Error::invalid(i18n::t(
            "nucleo-email-erro-modelo-valor",
        )));
    }
    Ok(())
}

/// O lançamento pelo modelo do usuário.
pub fn read_template(t: &BodyTemplate, body: &str, year: Option<i32>) -> Option<TxnDraft> {
    let re = super::filter::compile_regex(&t.pattern).ok()?;
    let c = re.captures(body)?;
    let raw = c.name("valor")?.as_str().replace("R$", "");
    let amount = cents(&raw)?;
    let income = t.direction.as_deref() == Some("income");
    Some(TxnDraft {
        kind: DraftKind::Template,
        amount: if income { amount.abs() } else { -amount.abs() },
        date: c
            .name("data")
            .and_then(|d| date_in(d.as_str(), year))
            .or_else(|| date_in(body, year)),
        description: c
            .name("descricao")
            .map(|d| clean_name(d.as_str()))
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| i18n::t("nucleo-email-descricao-padrao")),
        card_last4: None,
        account_id: t.account_id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purchase_and_pix_notices() {
        let d = read(
            "Olá! Compra de R$ 45,90 APROVADA em PADARIA PAO QUENTE no cartão final 1234 em 18/10/2026 às 10:21.",
            Some(2026),
        )
        .unwrap();
        assert_eq!(d.kind, DraftKind::Purchase);
        assert_eq!(d.amount, -4_590);
        assert_eq!(d.description, "PADARIA PAO QUENTE");
        assert_eq!(d.card_last4.as_deref(), Some("1234"));
        assert_eq!(d.date.as_deref(), Some("2026-10-18"));

        let d = read(
            "Você fez uma compra de R$ 1.234,56 em MAGAZINE EXEMPLO",
            Some(2026),
        )
        .unwrap();
        assert_eq!(d.amount, -123_456);
        assert_eq!(d.description, "MAGAZINE EXEMPLO");

        let p = read(
            "Você recebeu um Pix de R$ 100,00 de Maria Souza em 19/10.",
            Some(2026),
        )
        .unwrap();
        assert_eq!(p.kind, DraftKind::PixIn);
        assert_eq!(p.amount, 10_000);
        assert_eq!(p.description, "Pix recebido de Maria Souza");
        assert_eq!(p.date.as_deref(), Some("2026-10-19"));

        let o = read("Pix enviado: R$ 50,00 para João Lima.", Some(2026)).unwrap();
        assert_eq!((o.kind, o.amount), (DraftKind::PixOut, -5_000));
        assert_eq!(o.description, "Pix para João Lima");

        assert!(read("Sua fatura de R$ 900,00 está disponível", Some(2026)).is_none());
    }

    #[test]
    fn user_template() {
        let t = BodyTemplate {
            pattern: r"Débito de (?P<valor>R\$\s*[\d.,]+) - (?P<descricao>[^\n]+)".into(),
            direction: None,
            account_id: Some("acc".into()),
        };
        validate_template(&t).unwrap();
        let d = read_template(
            &t,
            "Aviso: Débito de R$ 89,90 - CONTA DE LUZ\nem 05/10/2026",
            Some(2026),
        )
        .unwrap();
        assert_eq!((d.amount, d.description.as_str()), (-8_990, "CONTA DE LUZ"));
        assert_eq!(d.date.as_deref(), Some("2026-10-05"));
        assert!(
            validate_template(&BodyTemplate {
                pattern: "sem grupo".into(),
                ..Default::default()
            })
            .is_err()
        );
    }
}
