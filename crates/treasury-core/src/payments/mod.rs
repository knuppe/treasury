//! Boletos, Pix "copia e cola" e comprovantes lidos no computador (docs/ocr-other-documents.md ›
//! passos 2 e 3), antes de qualquer IA:
//!
//! - **boleto, conta de arrecadação e Pix** (código de barras, linha digitável, BR Code; tudo
//!   conferido por dígito verificador ou CRC) → um **lançamento previsto** com o vencimento, que
//!   entra no saldo previsto e no lembrete de contas a vencer (`notify`, "due_soon"). O app não
//!   paga nada: a linha digitável ou o código Pix ficam na nota do lançamento para copiar.
//! - **comprovante de Pix, TED ou boleto pago** → o lançamento do extrato com o mesmo valor
//!   (data ±2 dias) ganha quem recebeu e, na nota, os dados lidos (ID fim a fim, banco, valor,
//!   data); sem lançamento, cria um. Comprovante de boleto fecha o previsto. A imagem **não** é
//!   guardada (o cofre cresceria demais): só os dados, mais o sha256 do arquivo na nota, para
//!   reconhecer o mesmo comprovante (ou boleto) lido de novo.
//!
//! `payments.read` lê (OCR fora da trava do banco) e não grava; `payments.plan` e
//! `payments.save_proof` gravam o que a pessoa revisou. Sem migração: o previsto usa
//! `txn.status = 'planned'` e `external_id` (`boleto:<44 dígitos>` ou `pix:<hash>`) para não
//! duplicar.

pub mod boleto;
pub mod br_code;
pub mod pix;
pub mod proof;

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use self::boleto::Bill;
use self::pix::PixCode;
use self::proof::{Direction, Proof};
use crate::api::{Api, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::import::text::fold;
use crate::ocr::OcrText;
use crate::receipts::ocr::OcrLine;
use crate::time::{fmt_date, parse_date};

pub const READ_ONLY: &[&str] = &[];

// ---------------------------------------------------------------------------
// Rótulos no texto do OCR

/// Palavras que encerram um nome lido depois de um rótulo.
const NAME_STOP: &[&str] = &[
    "cpf",
    "cnpj",
    "ag",
    "agencia",
    "conta",
    "cc",
    "chave",
    "instituicao",
    "banco",
    "tipo",
    "valor",
    "data",
];

/// O que vem depois do rótulo no começo da linha (palavras originais, com acento e caixa).
fn after_label(row: &str, label: &str) -> Option<String> {
    let words: Vec<&str> = row.split_whitespace().collect();
    let lab: Vec<&str> = label.split(' ').collect();
    if words.len() < lab.len() {
        return None;
    }
    let ok = lab.iter().enumerate().all(|(k, l)| fold(words[k]) == *l);
    ok.then(|| words[lab.len()..].join(" "))
}

/// Um nome: até o CPF/CNPJ, um número ou outro rótulo; ao menos três letras.
fn clean_name(s: &str) -> Option<String> {
    let s = s.trim_start_matches([':', '-', ' ', '.']);
    let mut out = vec![];
    for w in s.split_whitespace() {
        let f = fold(w);
        if w.chars().any(|c| c.is_ascii_digit())
            || w.contains('*')
            || NAME_STOP.contains(&f.as_str())
        {
            break;
        }
        out.push(w);
    }
    let name = out
        .join(" ")
        .trim_matches(|c: char| !c.is_alphanumeric() && c != ')' && c != '.')
        .trim()
        .to_string();
    (name.chars().filter(|c| c.is_alphabetic()).count() >= 3).then_some(name)
}

/// O valor do primeiro rótulo achado (na ordem dada): o resto da linha, ou uma das linhas de
/// baixo ("Nome Fulano", ou só o nome), quando o rótulo está sozinho.
pub(crate) fn labeled(rows: &[String], labels: &[&str]) -> Option<String> {
    for label in labels {
        for (i, r) in rows.iter().enumerate() {
            let Some(rest) = after_label(r, label) else {
                continue;
            };
            if let Some(v) = clean_name(&rest) {
                return Some(v);
            }
            for n in rows.iter().skip(i + 1).take(3) {
                if let Some(v) = after_label(n, "nome").and_then(|v| clean_name(&v)) {
                    return Some(v);
                }
                let f = fold(n);
                // "Nome" sozinho: o nome vem na linha de baixo.
                if f == "nome" {
                    continue;
                }
                if !NAME_STOP.iter().any(|s| f.starts_with(s))
                    && let Some(v) = clean_name(n)
                {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Um rótulo procurado só depois do bloco de outro (ex.: a instituição de quem recebeu).
pub(crate) fn labeled_after(rows: &[String], first: &[&str], then: &[&str]) -> Option<String> {
    let start = rows
        .iter()
        .position(|r| first.iter().any(|l| after_label(r, l).is_some()))?;
    let block = &rows[start + 1..rows.len().min(start + 8)];
    // "001 - BANCO EXEMPLO" vira "BANCO EXEMPLO".
    let value = |v: &str| -> Option<String> {
        let v = v
            .trim_start_matches([':', ' '])
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .trim_start_matches([' ', '-', ':'])
            .trim();
        (v.chars().filter(|c| c.is_alphabetic()).count() >= 3).then(|| v.to_string())
    };
    for label in then {
        for (i, r) in block.iter().enumerate() {
            if let Some(v) = after_label(r, label) {
                if let Some(v) = value(&v) {
                    return Some(v);
                }
                if let Some(v) = block.get(i + 1).and_then(|n| value(n)) {
                    return Some(v);
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Leitura

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct PaymentReadArgs {
    /// Linhas já lidas no app (Apple Vision, no Mac).
    #[serde(default)]
    #[ts(optional)]
    pub lines: Option<Vec<OcrLine>>,
    #[serde(default)]
    #[ts(optional)]
    pub width: Option<f32>,
    #[serde(default)]
    #[ts(optional)]
    pub height: Option<f32>,
    /// A imagem (para o PaddleOCR, fora do Mac).
    #[serde(default)]
    #[ts(optional)]
    pub image_base64: Option<String>,
    /// Textos de QR e códigos de barras lidos na interface (Pix, 44 dígitos do boleto).
    #[serde(default)]
    #[ts(optional)]
    pub codes: Option<Vec<String>>,
    /// Texto colado (linha digitável, Pix "copia e cola" ou o texto do comprovante).
    #[serde(default)]
    #[ts(optional)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PaymentDocKind {
    Bill,
    Pix,
    Proof,
    /// Nada que o app leia sozinho (fica para o agente, se a pessoa quiser).
    None,
}

/// Lançamento que combina com o documento.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PaymentMatch {
    pub txn_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub description: String,
    #[ts(optional)]
    pub payee: Option<String>,
    pub account_id: Id,
    pub account_name: String,
    /// `planned` (previsto) ou `cleared`/`reconciled` (do extrato).
    pub status: String,
    #[ts(type = "number")]
    pub attachments: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PaymentReading {
    pub kind: PaymentDocKind,
    #[ts(optional)]
    pub bill: Option<Bill>,
    #[ts(optional)]
    pub pix: Option<PixCode>,
    #[ts(optional)]
    pub proof: Option<Proof>,
    /// Descrição sugerida para o lançamento.
    pub description: String,
    /// Categoria sugerida (pelo segmento da conta ou "Contas e boletos").
    #[ts(optional)]
    pub category_id: Option<Id>,
    /// O previsto que já existe para este boleto ou Pix (não lança de novo).
    #[ts(optional)]
    pub existing: Option<PaymentMatch>,
    /// Comprovante: lançamentos do extrato que combinam (mais perto primeiro).
    pub matches: Vec<PaymentMatch>,
    /// Comprovante: previstos que ele paga (mesmo boleto, ou mesmo valor perto da data).
    pub planned: Vec<PaymentMatch>,
    /// Comprovante já ligado a um lançamento (mesmo arquivo ou mesmo ID fim a fim).
    #[ts(optional)]
    pub linked: Option<PaymentMatch>,
    /// sha256 do arquivo lido (vai na nota do lançamento, para não ler duas vezes).
    #[ts(optional)]
    pub sha256: Option<String>,
    /// O texto lido (para revisar e para o agente, se a pessoa quiser).
    pub ocr: OcrText,
    pub warnings: Vec<String>,
}

/// Dedupe do previsto: o código do boleto ou o hash do Pix.
pub fn bill_key(b: &Bill) -> String {
    format!("boleto:{}", b.barcode)
}

fn match_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PaymentMatch> {
    Ok(PaymentMatch {
        txn_id: r.get(0)?,
        date: r.get(1)?,
        amount: r.get(2)?,
        description: r.get(3)?,
        payee: r.get(4)?,
        account_id: r.get(5)?,
        account_name: r.get(6)?,
        status: r.get(7)?,
        attachments: r.get(8)?,
    })
}

const MATCH_COLS: &str =
    "t.id, t.date, t.amount, t.description, t.payee, t.account_id, a.name, t.status,
     (SELECT count(*) FROM attachment x WHERE x.txn_id = t.id)";

/// O marcador do arquivo na nota do lançamento.
pub fn file_mark(sha256: &str) -> String {
    format!("sha256:{}", &sha256[..sha256.len().min(16)])
}

/// Lançamento cuja nota tem o trecho (marcador do arquivo ou ID fim a fim).
fn by_note(conn: &Connection, needle: &str) -> Result<Option<PaymentMatch>> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {MATCH_COLS} FROM txn t JOIN account a ON a.id = t.account_id
                 WHERE instr(t.notes, ?1) > 0 ORDER BY t.date LIMIT 1"
            ),
            [needle],
            match_row,
        )
        .optional()?)
}

fn by_external(conn: &Connection, key: &str) -> Result<Option<PaymentMatch>> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {MATCH_COLS} FROM txn t JOIN account a ON a.id = t.account_id
                 WHERE t.external_id = ?1 ORDER BY t.date LIMIT 1"
            ),
            [key],
            match_row,
        )
        .optional()?)
}

/// Lançamentos do extrato com o valor do comprovante, a até 2 dias da data.
pub fn proof_matches(
    conn: &Connection,
    amount: i64,
    date: NaiveDate,
    dir: Direction,
) -> Result<Vec<PaymentMatch>> {
    let signed = match dir {
        Direction::Out => -amount,
        Direction::In => amount,
    };
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {MATCH_COLS} FROM txn t JOIN account a ON a.id = t.account_id
          WHERE t.status IN ('cleared', 'reconciled') AND t.amount = ?1
            AND t.date BETWEEN ?2 AND ?3 AND t.kind <> 'card_payment'
          ORDER BY abs(julianday(t.date) - julianday(?4)), t.id LIMIT 6"
    ))?;
    let rows = stmt.query_map(
        params![
            signed,
            fmt_date(date - Duration::days(2)),
            fmt_date(date + Duration::days(2)),
            fmt_date(date)
        ],
        match_row,
    )?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Previstos que o comprovante paga: o mesmo boleto, ou o mesmo valor a até 10 dias.
pub fn planned_matches(
    conn: &Connection,
    amount: Option<i64>,
    date: Option<NaiveDate>,
    barcode: Option<&str>,
) -> Result<Vec<PaymentMatch>> {
    let mut out = vec![];
    if let Some(b) = barcode
        && let Some(m) = by_external(conn, &format!("boleto:{b}"))?
        && m.status == "planned"
    {
        out.push(m);
    }
    if let (Some(a), Some(d)) = (amount, date) {
        let mut stmt = conn.prepare_cached(&format!(
            "SELECT {MATCH_COLS} FROM txn t JOIN account a ON a.id = t.account_id
              WHERE t.status = 'planned' AND t.amount = ?1 AND t.date BETWEEN ?2 AND ?3
              ORDER BY abs(julianday(t.date) - julianday(?4)) LIMIT 4"
        ))?;
        let rows = stmt.query_map(
            params![
                -a,
                fmt_date(d - Duration::days(10)),
                fmt_date(d + Duration::days(10)),
                fmt_date(d)
            ],
            match_row,
        )?;
        for m in rows {
            let m = m?;
            if !out.iter().any(|o| o.txn_id == m.txn_id) {
                out.push(m);
            }
        }
    }
    Ok(out)
}

fn category_by_seed(conn: &Connection, seed: &str) -> Result<Option<Id>> {
    Ok(conn
        .query_row(
            "SELECT id FROM category WHERE seed_key = ?1 AND archived = 0 AND kind = 'expense'
             ORDER BY sort_order LIMIT 1",
            [seed],
            |r| r.get(0),
        )
        .optional()?)
}

/// O que o documento é, só pelo texto e pelos códigos (sem banco de dados).
pub fn classify(
    rows: &[String],
    codes: &[String],
    today: NaiveDate,
) -> (PaymentDocKind, Option<Bill>, Option<PixCode>, Option<Proof>) {
    let mut bill = codes.iter().find_map(|c| boleto::from_barcode(c, today));
    let mut pix = codes.iter().find_map(|c| pix::parse(c));
    let text = fold(&rows.join(" "));
    let image_kind = crate::import::image::classify(
        &format!(" {text} "),
        0,
        false,
        crate::receipts::ocr::lines::Tier::Unbalanced,
    );
    if bill.is_none() {
        bill = boleto::find_in_text(rows, today)
            .or_else(|| codes.iter().find_map(|c| boleto::from_line(c, today)));
    } else if let Some(b) = bill.as_mut() {
        let from_text = boleto::find_in_text(rows, today);
        if b.due_date.is_none() {
            b.due_date = from_text.as_ref().and_then(|t| t.due_date.clone());
        }
        b.beneficiary = from_text.and_then(|t| t.beneficiary);
    }
    if pix.is_none() {
        pix = pix::find_in_text(rows);
    }
    let paid = text.contains("comprovante")
        && ["pagamento", "pago", "paga ", "quitado", "autenticacao"]
            .iter()
            .any(|w| text.contains(w));
    if image_kind == crate::import::image::ImageKind::PaymentProof || paid {
        let p = proof::read(rows, bill.as_ref().map(|b| b.barcode.clone()));
        return (PaymentDocKind::Proof, None, None, Some(p));
    }
    if let Some(b) = bill {
        return (PaymentDocKind::Bill, Some(b), pix, None);
    }
    if let Some(p) = pix {
        return (PaymentDocKind::Pix, None, Some(p), None);
    }
    (PaymentDocKind::None, None, None, None)
}

fn describe_bill(b: &Bill) -> String {
    let who = b.beneficiary.clone().or_else(|| {
        b.bank_name.clone().map(|n| {
            t_args(
                "nucleo-pagamentos-boleto-do-banco",
                &args(&[("banco", n.into())]),
            )
        })
    });
    match (b.kind, who) {
        (_, Some(w)) => w,
        (boleto::BillKind::Collection, None) => t(match b.segment {
            Some(1) => "nucleo-pagamentos-seg-prefeitura",
            Some(2) => "nucleo-pagamentos-seg-agua",
            Some(3) => "nucleo-pagamentos-seg-energia",
            Some(4) => "nucleo-pagamentos-seg-telefone",
            Some(5) => "nucleo-pagamentos-seg-governo",
            Some(7) => "nucleo-pagamentos-seg-multa",
            _ => "nucleo-pagamentos-seg-conta",
        }),
        (boleto::BillKind::Bank, None) => t("nucleo-pagamentos-boleto"),
    }
}

/// `payments.read`: o OCR (fora da trava do banco), a leitura e as combinações.
pub fn read(api: &Api, a: &PaymentReadArgs) -> Result<PaymentReading> {
    let today = crate::time::today_local();
    let mut warnings = vec![];
    let mut rows: Vec<String> = a
        .text
        .as_deref()
        .map(|t| t.lines().map(str::to_string).collect())
        .unwrap_or_default();
    let has_image = a.lines.as_ref().is_some_and(|l| !l.is_empty()) || a.image_base64.is_some();
    let mut ocr = OcrText::default();
    let bytes = a
        .image_base64
        .as_deref()
        .map(crate::transactions::b64::decode)
        .transpose()?;
    let sha256 = bytes.as_deref().map(|b| {
        use sha2::Digest;
        hex::encode(sha2::Sha256::digest(b))
    });
    if has_image {
        match crate::ocr::read_text(
            a.lines.as_deref(),
            crate::ocr::aspect(a.width, a.height),
            bytes.as_deref(),
        ) {
            Ok(o) => {
                rows.extend(o.rows.iter().cloned());
                ocr = o;
            }
            Err(e) => warnings.push(e.message()),
        }
    }
    let codes = a.codes.clone().unwrap_or_default();
    let (kind, bill, pix, proof) = classify(&rows, &codes, today);
    if ocr.rows.is_empty() && !rows.is_empty() {
        ocr.rows = rows.clone();
    }
    api.with_session(|s, _| {
        let conn = s.db.conn();
        let mut r = PaymentReading {
            kind,
            bill: None,
            pix: None,
            proof: None,
            description: String::new(),
            category_id: None,
            existing: None,
            matches: vec![],
            planned: vec![],
            linked: None,
            sha256: sha256.clone(),
            ocr: ocr.clone(),
            warnings: warnings.clone(),
        };
        let same_file = match &sha256 {
            Some(h) => by_note(conn, &file_mark(h))?,
            None => None,
        };
        match kind {
            PaymentDocKind::Bill => {
                let b = bill.clone().expect("boleto");
                r.description = describe_bill(&b);
                r.category_id = category_by_seed(conn, boleto::category_seed(&b))?;
                r.existing = match by_external(conn, &bill_key(&b))? {
                    Some(m) => Some(m),
                    None => same_file.clone(),
                };
                if b.amount.is_none() {
                    r.warnings.push(t("nucleo-pagamentos-aviso-sem-valor"));
                }
                if b.due_date.is_none() {
                    r.warnings.push(t("nucleo-pagamentos-aviso-sem-vencimento"));
                }
                r.bill = Some(b);
                r.pix = pix.clone();
            }
            PaymentDocKind::Pix => {
                let p = pix.clone().expect("pix");
                r.description = p.name.clone().unwrap_or_else(|| t("nucleo-pagamentos-pix"));
                r.category_id = category_by_seed(conn, "categorias-contas-e-boletos")?;
                r.existing = match by_external(conn, &pix::dedup_key(&p))? {
                    Some(m) => Some(m),
                    None => same_file.clone(),
                };
                if p.amount.is_none() {
                    r.warnings.push(t("nucleo-pagamentos-aviso-sem-valor"));
                }
                if p.dynamic {
                    r.warnings.push(t("nucleo-pagamentos-aviso-pix-dinamico"));
                }
                r.pix = Some(p);
            }
            PaymentDocKind::Proof => {
                let p = proof.clone().expect("comprovante");
                let date = p.date.as_deref().and_then(|d| parse_date(d).ok());
                if let (Some(amount), Some(d), false) = (p.amount, date, p.scheduled) {
                    r.matches = proof_matches(conn, amount, d, p.direction)?;
                }
                if p.scheduled {
                    r.warnings.push(t("nucleo-pagamentos-aviso-agendamento"));
                }
                if p.direction == Direction::Out && !p.scheduled {
                    r.planned = planned_matches(conn, p.amount, date, p.barcode.as_deref())?;
                }
                r.description = p.counterpart.clone().unwrap_or_else(|| {
                    t(match p.kind {
                        proof::ProofKind::Pix => "nucleo-pagamentos-pix",
                        proof::ProofKind::Bill => "nucleo-pagamentos-boleto",
                        _ => "nucleo-pagamentos-transferencia",
                    })
                });
                if !p.checked {
                    r.warnings.push(t("nucleo-pagamentos-aviso-revisar"));
                }
                r.linked = match &p.e2e_id {
                    Some(e) => by_note(conn, e)?,
                    None => None,
                }
                .or(same_file);
                r.proof = Some(p);
            }
            PaymentDocKind::None => {}
        }
        Ok(r)
    })
}

// ---------------------------------------------------------------------------
// Gravação

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct PlanArgs {
    pub account_id: Id,
    pub description: String,
    /// Valor a pagar, em centavos (positivo).
    #[ts(type = "number")]
    pub amount: i64,
    pub due_date: String,
    #[serde(default)]
    #[ts(optional)]
    pub category_id: Option<Id>,
    /// Os 44 dígitos do boleto (ou o código Pix inteiro, com `pix: true`).
    pub code: String,
    #[serde(default)]
    #[ts(optional)]
    pub pix: Option<bool>,
    /// sha256 do arquivo lido (`payments.read`), para reconhecê-lo depois.
    #[serde(default)]
    #[ts(optional)]
    pub sha256: Option<String>,
}

/// Cria o lançamento previsto do boleto ou do Pix (uma vez só por código).
pub fn plan(tx: &Transaction<'_>, a: &PlanArgs) -> Result<Id> {
    let today = crate::time::today_local();
    let (key, note) = if a.pix == Some(true) {
        let p = pix::parse(&a.code)
            .ok_or_else(|| Error::invalid(t("nucleo-pagamentos-erro-codigo")))?;
        (
            pix::dedup_key(&p),
            t_args(
                "nucleo-pagamentos-nota-pix",
                &args(&[("codigo", p.payload.into())]),
            ),
        )
    } else {
        let b = boleto::from_barcode(&a.code, today)
            .or_else(|| boleto::from_line(&a.code, today))
            .ok_or_else(|| Error::invalid(t("nucleo-pagamentos-erro-codigo")))?;
        (
            bill_key(&b),
            t_args(
                "nucleo-pagamentos-nota-boleto",
                &args(&[("linha", b.line.into())]),
            ),
        )
    };
    let note = match a.sha256.as_deref() {
        Some(h) => format!("{note}\n{}", file_mark(h)),
        None => note,
    };
    if by_external(tx, &key)?.is_some() {
        return Err(Error::conflict(t("nucleo-pagamentos-erro-ja-lancado")));
    }
    if a.amount <= 0 {
        return Err(Error::invalid(t("nucleo-pagamentos-erro-valor")));
    }
    let id = crate::transactions::create(
        tx,
        &crate::transactions::TxnInput {
            account_id: a.account_id.clone(),
            date: a.due_date.clone(),
            amount: -a.amount,
            kind: Some(crate::transactions::TxnKind::Expense),
            status: Some(crate::transactions::TxnStatus::Planned),
            description: a.description.trim().to_string(),
            payee: None,
            notes: Some(note),
            category_id: a.category_id.clone(),
            member_id: None,
            splits: None,
            tags: None,
            project_id: None,
            card_plastic_id: None,
            orig_amount: None,
            orig_currency: None,
        },
        today,
    )?;
    tx.execute(
        "UPDATE txn SET external_id = ?2 WHERE id = ?1",
        params![id, key],
    )?;
    Ok(id)
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct NewProofTxn {
    pub account_id: Id,
    /// Com sinal (− saiu, + entrou), em centavos.
    #[ts(type = "number")]
    pub amount: i64,
    pub description: String,
    #[serde(default)]
    #[ts(optional)]
    pub category_id: Option<Id>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct SaveProofArgs {
    /// O lançamento do extrato (ou o previsto) que o comprovante comprova.
    #[serde(default)]
    #[ts(optional)]
    pub txn_id: Option<Id>,
    /// Sem lançamento: o que criar.
    #[serde(default)]
    #[ts(optional)]
    pub create: Option<NewProofTxn>,
    /// Data do pagamento (o previsto passa a valer nela).
    pub date: String,
    /// Quem recebeu (ou pagou): vai para o favorecido, se estiver vazio.
    #[serde(default)]
    #[ts(optional)]
    pub payee: Option<String>,
    /// Previsto que este pagamento quita (some, porque o lançamento do extrato vale por ele).
    #[serde(default)]
    #[ts(optional)]
    pub close_planned_id: Option<Id>,
    /// O que o comprovante diz (vai para a nota do lançamento; a imagem não é guardada).
    #[serde(default)]
    #[ts(optional)]
    pub proof: Option<Proof>,
    /// sha256 do arquivo lido, para reconhecer o mesmo comprovante depois.
    #[serde(default)]
    #[ts(optional)]
    pub sha256: Option<String>,
}

fn empty_patch(id: &str) -> crate::transactions::TxnPatch {
    crate::transactions::TxnPatch {
        id: id.to_string(),
        account_id: None,
        date: None,
        amount: None,
        kind: None,
        status: None,
        description: None,
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
    }
}

/// A linha da nota com o que o comprovante diz ("Comprovante Pix · Banco · ID E2E … ·
/// sha256:…"), sem CPF.
fn proof_note(p: Option<&Proof>, sha256: Option<&str>) -> String {
    let mut parts: Vec<String> = vec![];
    if let Some(p) = p {
        parts.push(t(match p.kind {
            proof::ProofKind::Pix => "nucleo-pagamentos-nota-comprovante-pix",
            proof::ProofKind::Bill => "nucleo-pagamentos-nota-comprovante-boleto",
            proof::ProofKind::Transfer => "nucleo-pagamentos-nota-comprovante-transferencia",
            proof::ProofKind::Other => "nucleo-pagamentos-nota-comprovante",
        }));
        if let Some(i) = p.institution.as_ref().or(p.origin_bank.as_ref()) {
            parts.push(i.clone());
        }
        if let Some(time) = &p.time {
            parts.push(time.clone());
        }
        if let Some(e) = &p.e2e_id {
            parts.push(format!("E2E {e}"));
        }
    }
    if let Some(h) = sha256 {
        if parts.is_empty() {
            parts.push(t("nucleo-pagamentos-nota-comprovante"));
        }
        parts.push(file_mark(h));
    }
    parts.join(" · ")
}

/// Liga o comprovante ao lançamento (ou cria um): favorecido, os dados lidos na nota e,
/// quando ele paga um previsto, o previsto vira o pagamento (ou sai, se o extrato já trouxe o
/// lançamento).
pub fn save_proof(tx: &Transaction<'_>, a: &SaveProofArgs) -> Result<Id> {
    let today = crate::time::today_local();
    parse_date(&a.date)?;
    let payee = a.payee.as_deref().map(str::trim).filter(|p| !p.is_empty());
    let id = match (&a.txn_id, &a.create) {
        (Some(id), _) => {
            let (status, cur_payee): (String, Option<String>) = tx
                .query_row("SELECT status, payee FROM txn WHERE id = ?1", [id], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .optional()?
                .ok_or_else(|| Error::not_found(crate::transactions::util::txn_ref(id)))?;
            let planned = status == "planned";
            let patch = crate::transactions::TxnPatch {
                id: id.clone(),
                date: planned.then(|| a.date.clone()),
                status: planned.then_some(crate::transactions::TxnStatus::Cleared),
                payee: payee
                    .filter(|_| cur_payee.as_deref().is_none_or(|p| p.trim().is_empty()))
                    .map(|p| Some(p.to_string())),
                ..empty_patch(id)
            };
            crate::transactions::update(tx, &patch, today)?;
            id.clone()
        }
        (None, Some(n)) => crate::transactions::create(
            tx,
            &crate::transactions::TxnInput {
                account_id: n.account_id.clone(),
                date: a.date.clone(),
                amount: n.amount,
                kind: None,
                // Agendamento: o dinheiro ainda não saiu (entra no saldo previsto).
                status: a
                    .proof
                    .as_ref()
                    .filter(|p| p.scheduled)
                    .map(|_| crate::transactions::TxnStatus::Planned),
                description: n.description.trim().to_string(),
                payee: payee.map(str::to_string),
                notes: None,
                category_id: n.category_id.clone(),
                member_id: None,
                splits: None,
                tags: None,
                project_id: None,
                card_plastic_id: None,
                orig_amount: None,
                orig_currency: None,
            },
            today,
        )?,
        (None, None) => return Err(Error::invalid(t("nucleo-pagamentos-erro-sem-lancamento"))),
    };
    if let Some(p) = a.close_planned_id.as_ref().filter(|p| **p != id) {
        let status: Option<String> = tx
            .query_row("SELECT status FROM txn WHERE id = ?1", [p], |r| r.get(0))
            .optional()?;
        if status.as_deref() == Some("planned") {
            crate::transactions::delete(tx, std::slice::from_ref(p), today)?;
        }
    }
    let note = proof_note(a.proof.as_ref(), a.sha256.as_deref());
    if !note.is_empty() {
        let cur: Option<String> =
            tx.query_row("SELECT notes FROM txn WHERE id = ?1", [&id], |r| r.get(0))?;
        let notes = match cur.filter(|c| !c.trim().is_empty()) {
            Some(c) if c.contains(&note) => c,
            Some(c) => format!("{c}\n{note}"),
            None => note,
        };
        let mut patch = empty_patch(&id);
        patch.notes = Some(Some(notes));
        crate::transactions::update(tx, &patch, today)?;
    }
    Ok(id)
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "plan" => {
            let a: PlanArgs = parse_args(args)?;
            let reason = crate::transactions::api::with_desc(
                "nucleo-pagamentos-historico-previsto",
                &a.description,
            );
            to_value(ctx.write(&reason, |tx| plan(tx, &a))?.value)
        }
        "save_proof" => {
            let a: SaveProofArgs = parse_args(args)?;
            let desc = a
                .payee
                .clone()
                .or_else(|| a.create.as_ref().map(|c| c.description.clone()))
                .unwrap_or_default();
            let reason = crate::transactions::api::with_desc(
                "nucleo-pagamentos-historico-comprovante",
                &desc,
            );
            to_value(ctx.write(&reason, |tx| save_proof(tx, &a))?.value)
        }
        _ => Err(Error::UnknownCommand(format!("payments.{action}"))),
    }
}

#[cfg(test)]
mod tests;
