//! Detecção, prévia e gravação de importações.
//!
//! Decisões:
//! - **Duplicado**: mesmo `external_id` (FITID/identificador) já gravado na conta, ou o
//!   mesmo FITID repetido no próprio arquivo com o mesmo conteúdo. FITID repetido com
//!   conteúdo diferente (bancos que reaproveitam o id) vira `FITID#2`, `FITID#3`…
//! - **Possível duplicado**: mesma impressão digital (data + valor + descrição normalizada)
//!   na conta, ou mesma data e valor com descrição muito parecida. Cada lançamento
//!   existente "consome" no máximo uma linha do arquivo. Por padrão não é gravado.
//! - **Sinal**: relativo à conta. Cartão: compras negativas; se a maioria das linhas de um
//!   extrato de cartão vier positiva, o arquivo usa a convenção do banco e os sinais são
//!   invertidos (com aviso). OFX com `TRNTYPE=DEBIT` e valor positivo vira negativo.
//! - **Pagamento de fatura**: no cartão, crédito com "Pagamento recebido"/"Pagamento de
//!   fatura" etc. entra como `card_payment`, positivo e sem categoria; na conta corrente,
//!   débito "Pagamento de fatura"/"Pgto cartão" também vira `card_payment`.
//! - Linhas de saldo e de valor zero são ignoradas.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use base64::Engine as _;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};

use super::csv_file;
use super::ofx;
use super::text::{self, fold};
use super::{
    CsvMapping, DetectArgs, DetectResult, ImportBatch, ImportCommitArgs, ImportFormat,
    ImportPreview, ImportPreviewArgs, ImportResult, ImportRowError, ListBatchesArgs,
    OfxAccountInfo, PreviewCounts, PreviewRow, RowStatus,
};
use crate::accounts::AccountKind;
use crate::api::Ctx;
use crate::backup::BackupKind;
use crate::changes::Author;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::rules::RuleSet;
use crate::time::{fmt_date, now_utc};
use crate::transactions::util::money_text;
use crate::transactions::{self, NewTxn, TxnKind};

/// Tamanho máximo do arquivo importado.
const MAX_FILE_BYTES: usize = 25 * 1024 * 1024;
/// Acima disso, backup `pre_import` antes de gravar.
pub const BACKUP_THRESHOLD: usize = 200;
/// Prévias guardadas por sessão.
const MAX_PREVIEWS: usize = 8;

// ---------------------------------------------------------------------------
// Arquivo

pub(crate) fn decode_base64(s: &str) -> Result<Vec<u8>> {
    let s = s.trim();
    let s = match s.strip_prefix("data:") {
        Some(rest) => rest.split_once(',').map(|(_, b)| b).unwrap_or(rest),
        None => s,
    };
    let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&clean)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&clean))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(&clean))
        .map_err(|_| Error::invalid(t("nucleo-importacao-erro-base64")))?;
    if bytes.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-arquivo-vazio")));
    }
    if bytes.len() > MAX_FILE_BYTES {
        return Err(Error::invalid(t_args(
            "nucleo-importacao-erro-grande",
            &args(&[
                ("mb", (bytes.len() / (1024 * 1024)).into()),
                ("max", (MAX_FILE_BYTES / (1024 * 1024)).into()),
            ]),
        )));
    }
    Ok(bytes)
}

/// Cabeçalhos da amostra de um PDF ou OFX (com o FITID no OFX), no idioma da instalação.
fn sample_headers(fitid: bool) -> Vec<String> {
    let mut h = vec![
        t("nucleo-importacao-coluna-data"),
        t("nucleo-importacao-coluna-valor"),
        t("nucleo-importacao-coluna-descricao"),
    ];
    if fitid {
        h.push("FITID".into());
    }
    h
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Foto ou print (PNG, JPEG): o extrato lido pelo OCR segue o caminho do PDF só com imagem.
fn is_image(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG\r\n\x1a\n") || bytes.starts_with(&[0xFF, 0xD8, 0xFF])
}

fn detect_format(bytes: &[u8], filename: &str) -> Result<ImportFormat> {
    if bytes.starts_with(b"%PDF") || is_image(bytes) {
        return Ok(ImportFormat::Pdf);
    }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"\xD0\xCF\x11\xE0") {
        return Err(Error::invalid(t("nucleo-importacao-erro-excel")));
    }
    let lower = filename.to_ascii_lowercase();
    // Norma 43 (bancos da Espanha) segue o caminho do OFX (docs/international.md).
    if super::norma43::looks_like(&String::from_utf8_lossy(&bytes[..bytes.len().min(4096)])) {
        return Ok(ImportFormat::Ofx);
    }
    if ofx::looks_like_ofx(bytes)
        || ((lower.ends_with(".ofx") || lower.ends_with(".qfx")) && bytes.contains(&b'<'))
    {
        Ok(ImportFormat::Ofx)
    } else {
        Ok(ImportFormat::Csv)
    }
}

// ---------------------------------------------------------------------------
// Linhas lidas

#[derive(Debug, Clone, Default)]
struct RawRow {
    line: Option<u32>,
    date: Option<String>,
    amount: Option<i64>,
    description: String,
    payee: Option<String>,
    notes: Option<String>,
    external_id: Option<String>,
    trn_type: Option<String>,
    error: Option<String>,
    /// Final do cartão (adicional ou virtual) da linha, quando o arquivo ou o agente informa.
    card_last4: Option<String>,
    /// Compra no exterior: valor positivo e moeda de origem.
    orig: Option<(i64, String)>,
}

struct Parsed {
    format: ImportFormat,
    encoding: String,
    rows: Vec<RawRow>,
    warnings: Vec<String>,
    ofx: Option<OfxAccountInfo>,
    mapping: Option<CsvMapping>,
    preset_id: Option<String>,
}

/// Junta dois textos sem repetir: se um contém o outro, fica o mais longo.
fn combine(a: Option<&str>, b: Option<&str>) -> String {
    match (a.filter(|s| !s.is_empty()), b.filter(|s| !s.is_empty())) {
        (Some(a), Some(b)) => {
            let (fa, fb) = (fold(a), fold(b));
            if fa.contains(&fb) {
                a.to_string()
            } else if fb.contains(&fa) {
                b.to_string()
            } else {
                format!("{a} - {b}")
            }
        }
        (Some(a), None) => a.to_string(),
        (None, Some(b)) => b.to_string(),
        (None, None) => String::new(),
    }
}

fn ofx_info(s: &ofx::OfxStatement, count: usize, currency: &str) -> OfxAccountInfo {
    OfxAccountInfo {
        bank_id: s.bank_id.clone(),
        branch_id: s.branch_id.clone(),
        account_id: s.account_id.clone(),
        account_type: s.account_type.clone(),
        credit_card: s.credit_card,
        currency: s.currency.clone(),
        ledger_balance: s
            .ledger_balance_raw
            .as_deref()
            .and_then(|b| ofx::parse_amount(b, s.currency.as_deref().unwrap_or(currency)).ok()),
        balance_date: s.balance_date.map(fmt_date),
        date_start: s.date_start.map(fmt_date),
        date_end: s.date_end.map(fmt_date),
        statements: count as u32,
    }
}

/// Escolhe o extrato: pelo ACCTID pedido; senão o primeiro do mesmo tipo da conta.
fn pick_statement<'a>(
    doc: &'a ofx::OfxDocument,
    wanted: Option<&str>,
    card_account: bool,
) -> Result<&'a ofx::OfxStatement> {
    if let Some(w) = wanted.map(str::trim).filter(|w| !w.is_empty()) {
        return doc
            .statements
            .iter()
            .find(|s| s.account_id.as_deref() == Some(w))
            .ok_or_else(|| {
                Error::not_found(t_args(
                    "nucleo-importacao-ref-extrato",
                    &args(&[("conta", w.to_string().into())]),
                ))
            });
    }
    Ok(doc
        .statements
        .iter()
        .find(|s| s.credit_card == card_account && !s.txns.is_empty())
        .or_else(|| doc.statements.iter().find(|s| !s.txns.is_empty()))
        .unwrap_or(&doc.statements[0]))
}

/// O documento do extrato: OFX ou Norma 43 (lido no mesmo formato).
fn parse_statement(text: &str) -> Result<ofx::OfxDocument> {
    if super::norma43::looks_like(text) {
        super::norma43::parse(text)
    } else {
        ofx::parse(text)
    }
}

fn parse_ofx(
    bytes: &[u8],
    encoding: Option<&str>,
    currency: &str,
    card_account: bool,
    wanted: Option<&str>,
) -> Result<Parsed> {
    let declared = ofx::declared_charset(bytes);
    let dec = text::decode(bytes, encoding, declared.as_deref())?;
    let doc = parse_statement(&dec.text)?;
    let stmt = pick_statement(&doc, wanted, card_account)?;
    let mut warnings = Vec::new();
    if doc.statements.len() > 1 {
        warnings.push(t_args(
            "nucleo-importacao-aviso-varios-extratos",
            &args(&[
                ("n", doc.statements.len().into()),
                (
                    "conta",
                    stmt.account_id
                        .clone()
                        .unwrap_or_else(|| t("nucleo-importacao-sem-numero"))
                        .into(),
                ),
            ]),
        ));
    }
    if let Some(c) = &stmt.currency
        && !c.eq_ignore_ascii_case(currency)
    {
        warnings.push(t_args(
            "nucleo-importacao-aviso-moeda",
            &args(&[
                ("moeda_extrato", c.clone().into()),
                ("moeda_conta", currency.to_string().into()),
            ]),
        ));
    }
    if stmt.credit_card != card_account {
        warnings.push(t(if stmt.credit_card {
            "nucleo-importacao-aviso-arquivo-cartao"
        } else {
            "nucleo-importacao-aviso-conta-cartao"
        }));
    }
    let mut rows = Vec::with_capacity(stmt.txns.len());
    for t in &stmt.txns {
        let mut r = RawRow {
            description: combine(t.name.as_deref().or(t.payee.as_deref()), t.memo.as_deref()),
            payee: t.payee.clone(),
            external_id: t.fitid.clone(),
            trn_type: t.trn_type.clone(),
            // Alguns bancos repetem o FITID em CHECKNUM; só vale como nota quando difere.
            notes: t
                .check_num
                .as_ref()
                .filter(|c| Some(*c) != t.fitid.as_ref())
                .map(|c| {
                    t_args(
                        "nucleo-importacao-nota-documento",
                        &args(&[("numero", c.clone().into())]),
                    )
                }),
            ..Default::default()
        };
        if r.payee.as_deref() == Some(r.description.as_str()) {
            r.payee = None;
        }
        if r.description.is_empty() {
            r.description = t
                .trn_type
                .clone()
                .unwrap_or_else(|| crate::i18n::t("nucleo-importacao-descricao-padrao"));
        }
        match t.date {
            Some(d) => r.date = Some(fmt_date(d)),
            None => {
                r.error = Some(t_args(
                    "nucleo-importacao-erro-data",
                    &args(&[(
                        "valor",
                        t.date_raw
                            .clone()
                            .unwrap_or_else(|| crate::i18n::t("nucleo-importacao-vazia"))
                            .into(),
                    )]),
                ))
            }
        }
        match t.amount_raw.as_deref() {
            Some(raw) => match ofx::parse_amount(raw, currency) {
                Ok(mut a) => {
                    if t.trn_type.as_deref() == Some("DEBIT") && a > 0 {
                        a = -a;
                    }
                    r.amount = Some(a);
                }
                Err(e) => {
                    r.error.get_or_insert_with(|| e.message());
                }
            },
            None => {
                r.error
                    .get_or_insert_with(|| crate::i18n::t("nucleo-importacao-erro-sem-trnamt"));
            }
        }
        rows.push(r);
    }
    Ok(Parsed {
        format: ImportFormat::Ofx,
        encoding: dec.encoding,
        rows,
        warnings,
        ofx: Some(ofx_info(stmt, doc.statements.len(), currency)),
        mapping: None,
        preset_id: None,
    })
}

fn resolve_mapping(
    text: &str,
    explicit: Option<&CsvMapping>,
    preset_id: Option<&str>,
) -> Result<(CsvMapping, Option<String>)> {
    if let Some(m) = explicit {
        return Ok((m.clone(), preset_id.map(str::to_string)));
    }
    if let Some(id) = preset_id {
        return Ok((csv_file::preset(id)?.mapping, Some(id.to_string())));
    }
    let delim = csv_file::detect_delimiter(text);
    let layout = csv_file::detect_layout(text, delim)?;
    match csv_file::suggest_preset(&layout.headers) {
        Some(id) => Ok((csv_file::preset(&id)?.mapping, Some(id))),
        None => Err(Error::invalid(t_args(
            "nucleo-importacao-csv-erro-layout",
            &args(&[("colunas", layout.headers.join(", ").into())]),
        ))),
    }
}

/// Fatura de cartão em PDF: texto, layout (docs/layouts.md) e linhas.
fn parse_card_pdf(bytes: &[u8], password: Option<&str>) -> Result<(Parsed, super::CardPdfInfo)> {
    let text = crate::pdf::text(bytes, password).map_err(|e| {
        let m = e.to_lowercase();
        if m.contains("password") || m.contains("decrypt") || m.contains("encrypt") {
            Error::invalid(t(if password.is_some() {
                "nucleo-importacao-pdf-senha-errada"
            } else {
                "nucleo-importacao-pdf-senha-fatura"
            }))
        } else {
            Error::invalid(t_args(
                "nucleo-importacao-pdf-erro",
                &args(&[("erro", e.to_string().into())]),
            ))
        }
    })?;
    let layout = crate::layouts::require(
        &crate::layouts::Doc {
            format: crate::layouts::LayoutFormat::Pdf,
            text: &text,
            headers: &[],
        },
        crate::layouts::LayoutSubject::CardStatement,
    )?;
    let p = super::card_pdf::parse(&text);
    let rows_total: i64 = -p.rows.iter().map(|r| r.amount).sum::<i64>();
    let total_matches = p.total.map(|t| t == rows_total);
    let mut warnings = vec![];
    if p.due_date.is_none() {
        warnings.push(t("nucleo-importacao-pdf-sem-vencimento"));
    }
    // Os layouts de fatura em PDF são de bancos brasileiros: valores em reais.
    match (p.total, total_matches) {
        (Some(total), Some(false)) => warnings.push(t_args(
            "nucleo-importacao-pdf-total-diferente",
            &args(&[
                ("impresso", money_text(total, "BRL").into()),
                ("lido", money_text(rows_total, "BRL").into()),
            ]),
        )),
        (None, _) => warnings.push(t("nucleo-importacao-pdf-sem-total")),
        _ => {}
    }
    if p.payments_skipped > 0 {
        warnings.push(t_args(
            "nucleo-importacao-pdf-pagamentos-fora",
            &args(&[("n", p.payments_skipped.into())]),
        ));
    }
    let info = super::CardPdfInfo {
        layout,
        issuer: p.issuer.clone(),
        due_date: p.due_date.map(fmt_date),
        reference_month: p.due_date.map(crate::time::month_of),
        total: p.total,
        rows_total,
        total_matches,
        payments_skipped: p.payments_skipped,
        minimum: p.minimum,
        credit_limit: p.credit_limit,
        previous_balance: p.previous_balance,
    };
    let rows = p
        .rows
        .into_iter()
        .map(|r| RawRow {
            line: Some(r.line),
            date: Some(fmt_date(r.date)),
            amount: Some(r.amount),
            description: r.description,
            card_last4: r.card_last4,
            orig: r.orig,
            ..Default::default()
        })
        .collect();
    Ok((
        Parsed {
            format: ImportFormat::Pdf,
            encoding: "pdf".into(),
            rows,
            warnings,
            ofx: None,
            mapping: None,
            preset_id: None,
        },
        info,
    ))
}

/// Texto de um extrato em PDF: o do próprio PDF ou, sem texto (página que é uma imagem, como
/// o extrato por período da Caixa) e na foto, o do OCR local. Devolve a confiança do OCR.
pub(crate) fn statement_text(bytes: &[u8]) -> Result<(String, Option<f32>)> {
    if bytes.starts_with(b"%PDF") {
        let text = crate::pdf::text(bytes, None).unwrap_or_default();
        if fold(&text).chars().filter(|c| c.is_alphanumeric()).count() >= 40 {
            return Ok((text, None));
        }
    }
    let o = crate::ocr::read_scanned(bytes)?;
    Ok((o.text(), Some(o.confidence)))
}

/// Extrato de conta em PDF (hoje, o "Extrato por período" da Caixa; docs/CAIXA-EXTRATO-
/// POR-PERIODO.md) ou a foto dele: lançamentos, CPF/CNPJ descartado, conferência dos saldos de
/// cada dia virando aviso.
fn parse_bank_pdf(bytes: &[u8]) -> Result<Parsed> {
    let (text, confidence) = statement_text(bytes)?;
    let id = crate::layouts::identify(&crate::layouts::Doc {
        format: crate::layouts::LayoutFormat::Pdf,
        text: &text,
        headers: &[],
    });
    if id.layout.as_ref().map(|l| l.id.as_str()) != Some("bank.caixa.extrato-periodo.v1") {
        return Err(Error::invalid(t("nucleo-importacao-pdf-conta-sem-leiaute")));
    }
    let st = super::caixa::parse(&text);
    let mut warnings = vec![];
    if let Some(c) = confidence {
        warnings.push(t_args(
            "nucleo-importacao-caixa-lido-ocr",
            &args(&[("confianca", ((c * 100.0).round() as i64).into())]),
        ));
    }
    for p in &st.problems {
        warnings.push(t_args(
            "nucleo-importacao-caixa-dia-nao-fecha",
            &args(&[
                (
                    "data",
                    crate::i18n::date(crate::i18n::current(), p.date).into(),
                ),
                ("impresso", money_text(p.printed, "BRL").into()),
                ("lido", money_text(p.computed, "BRL").into()),
            ]),
        ));
    }
    if st.total_ok == Some(false)
        && let (Some(o), Some(c)) = (st.opening, st.closing)
    {
        let sum: i64 = st.rows.iter().map(|r| r.amount).sum();
        warnings.push(t_args(
            "nucleo-importacao-caixa-total-nao-fecha",
            &args(&[
                ("impresso", money_text(c, "BRL").into()),
                ("lido", money_text(o + sum, "BRL").into()),
            ]),
        ));
    }
    let rows = st
        .rows
        .iter()
        .map(|r| RawRow {
            line: Some(r.line),
            date: Some(fmt_date(r.date)),
            amount: Some(r.amount),
            description: r.history.clone(),
            payee: r.payee.clone(),
            external_id: Some(r.external_id()),
            ..Default::default()
        })
        .collect();
    Ok(Parsed {
        format: ImportFormat::Pdf,
        encoding: if confidence.is_some() { "ocr" } else { "pdf" }.into(),
        rows,
        warnings,
        ofx: None,
        mapping: None,
        preset_id: None,
    })
}

/// `import.card_pdf_preview`: a fatura em PDF lida, sem conta e sem gravar.
pub fn card_pdf_preview(args: &super::CardPdfArgs) -> Result<super::CardPdfPreview> {
    let bytes = decode_base64(&args.content_base64)?;
    if !bytes.starts_with(b"%PDF") {
        return Err(Error::invalid(t_args(
            "nucleo-importacao-erro-nao-pdf",
            &crate::i18n::args(&[("arquivo", args.filename.clone().into())]),
        )));
    }
    let (parsed, info) = parse_card_pdf(&bytes, args.password.as_deref())?;
    Ok(super::CardPdfPreview {
        info,
        rows: parsed
            .rows
            .into_iter()
            .map(|r| super::CardPdfRowView {
                date: r.date.unwrap_or_default(),
                description: r.description,
                amount: r.amount.unwrap_or_default(),
            })
            .collect(),
        warnings: parsed.warnings,
    })
}

fn parse_csv(
    bytes: &[u8],
    encoding: Option<&str>,
    currency: &str,
    explicit: Option<&CsvMapping>,
    preset_id: Option<&str>,
) -> Result<Parsed> {
    let hint = explicit.and_then(|m| m.encoding.as_deref()).or(encoding);
    let dec = text::decode(bytes, hint, None)?;
    let (mut mapping, preset_id) = resolve_mapping(&dec.text, explicit, preset_id)?;
    let parsed = csv_file::parse(&dec.text, &mapping, currency)?;
    mapping.delimiter = Some(csv_file::delimiter_str(parsed.delimiter));
    mapping.skip_rows = Some(parsed.skip_rows);
    mapping.encoding = Some(dec.encoding.clone());
    let rows = parsed
        .rows
        .into_iter()
        .map(|r| RawRow {
            line: Some(r.line),
            date: r.date.map(fmt_date),
            amount: r.amount,
            description: r.description,
            payee: r.payee,
            external_id: r.external_id,
            error: r.error,
            ..Default::default()
        })
        .collect();
    Ok(Parsed {
        format: ImportFormat::Csv,
        encoding: dec.encoding,
        rows,
        warnings: vec![],
        ofx: None,
        mapping: Some(mapping),
        preset_id,
    })
}

// ---------------------------------------------------------------------------
// Classificação

const BILL_PAYMENT: &[&str] = &[
    "pagamento de fatura",
    "pagamento da fatura",
    "pagamento fatura",
    "pagto fatura",
    "pgto fatura",
    "pag fatura",
    "pagamento cartao de credito",
    "pagamento cartao credito",
    "pagto cartao",
    "pgto cartao",
    "pag cartao credito",
    "fatura cartao",
    "fatura do cartao",
    "fatura de cartao",
];

/// Só no extrato do cartão (crédito do pagamento).
const CARD_SIDE_PAYMENT: &[&str] = &[
    "pagamento recebido",
    "pagamento efetuado",
    "pagamento obrigado",
    "pagamento em ",
    "credito de pagamento",
];

pub(crate) fn is_bill_payment(desc: &str, card_side: bool) -> bool {
    let d = format!("{} ", fold(desc));
    BILL_PAYMENT.iter().any(|p| d.contains(p))
        || (card_side && CARD_SIDE_PAYMENT.iter().any(|p| d.contains(p)))
}

fn is_balance_line(desc: &str) -> bool {
    let d = fold(desc);
    d == "saldo"
        || d.starts_with("saldo ")
        || d.contains("saldo anterior")
        || d.contains("saldo do dia")
        || d.contains("saldo final")
        || d == "s a l d o"
}

struct Classified {
    kind: Option<TxnKind>,
    ignored: Option<String>,
    note: Option<String>,
}

fn classify(
    rows: &mut [RawRow],
    account_kind: AccountKind,
    warnings: &mut Vec<String>,
) -> Vec<Classified> {
    let card = account_kind == AccountKind::CreditCard;
    let mut out: Vec<Classified> = rows
        .iter()
        .map(|r| {
            // Linha de saldo vence o erro (no CSV ela costuma vir sem valor).
            let ignored = if is_balance_line(&r.description) {
                Some(t("nucleo-importacao-ignorada-saldo"))
            } else if r.error.is_some() {
                None
            } else if r.amount == Some(0) {
                Some(t("nucleo-importacao-valor-zero"))
            } else {
                None
            };
            Classified {
                kind: None,
                ignored,
                note: None,
            }
        })
        .collect();
    if card {
        // Convenção do arquivo: compras positivas? Então inverte tudo.
        let purchases: Vec<i64> = rows
            .iter()
            .zip(&out)
            .filter(|(r, c)| r.error.is_none() && c.ignored.is_none())
            .filter(|(r, _)| !is_bill_payment(&r.description, true))
            .filter_map(|(r, _)| r.amount)
            .collect();
        let positives = purchases.iter().filter(|a| **a > 0).count();
        if purchases.len() >= 3 && positives * 3 > purchases.len() * 2 {
            for r in rows.iter_mut() {
                if let Some(a) = r.amount.as_mut() {
                    *a = -*a;
                }
            }
            warnings.push(t("nucleo-importacao-aviso-sinais-invertidos"));
        }
    }
    for (r, c) in rows.iter_mut().zip(out.iter_mut()) {
        let Some(amount) = r.amount else { continue };
        if r.error.is_some() || c.ignored.is_some() {
            continue;
        }
        let payment_type = r.trn_type.as_deref() == Some("PAYMENT");
        c.kind = Some(
            if card && (is_bill_payment(&r.description, true) || (payment_type && amount > 0)) {
                if amount < 0 {
                    r.amount = Some(-amount);
                    c.note = Some(t("nucleo-importacao-nota-pagamento-sinal"));
                }
                TxnKind::CardPayment
            } else if !card && amount < 0 && is_bill_payment(&r.description, false) {
                TxnKind::CardPayment
            } else if amount < 0 {
                TxnKind::Expense
            } else {
                TxnKind::Income
            },
        );
    }
    out
}

// ---------------------------------------------------------------------------
// Prévia guardada na sessão

#[derive(Debug, Clone)]
struct PendingRow {
    row: PreviewRow,
    notes: Option<String>,
    /// Regras que definiram algum campo (acertos no commit).
    applied_rules: Vec<Id>,
    /// Fatura informada para a linha (mês de vencimento), quando o agente a leu da tela.
    statement_month: Option<String>,
}

#[derive(Debug, Clone)]
struct StoredPreview {
    token: String,
    account_id: Id,
    currency: String,
    filename: String,
    format: ImportFormat,
    file_sha256: String,
    rows: Vec<PendingRow>,
    /// Conta lida do OFX, para lembrar a conta escolhida ao gravar.
    ofx: Option<OfxAccountInfo>,
    /// Fatura em PDF: mês de vencimento e total impresso.
    /// Fatura em PDF: o que ela imprime (vencimento, total, mínimo, limite).
    statement: Option<super::CardPdfInfo>,
    created: Instant,
}

#[derive(Default)]
struct ImportScratch {
    previews: Vec<StoredPreview>,
}

const SCRATCH_KEY: &str = "import";

fn already_imported(conn: &Connection, account_id: &str, sha: &str) -> Result<Option<Id>> {
    Ok(conn
        .query_row(
            "SELECT id FROM import_batch WHERE account_id = ?1 AND file_sha256 = ?2 ORDER BY created_at DESC LIMIT 1",
            params![account_id, sha],
            |r| r.get(0),
        )
        .optional()?)
}

/// Marca duplicados e possíveis duplicados.
fn dedupe(
    conn: &Connection,
    account_id: &str,
    rows: &mut [PreviewRow],
    warnings: &mut Vec<String>,
) -> Result<()> {
    // 1. FITID repetido no próprio arquivo. Para cada FITID, as impressões digitais já
    //    vistas: igual a uma delas = duplicado; diferente = id sintético `FITID#n`.
    let mut seen: HashMap<String, Vec<String>> = HashMap::new();
    let mut collisions = 0;
    for row in rows.iter_mut() {
        if row.status != RowStatus::New {
            continue;
        }
        let Some(ext) = row.external_id.clone() else {
            continue;
        };
        let fp = transactions::fingerprint(
            row.date.as_deref().unwrap_or(""),
            row.amount.unwrap_or(0),
            &row.description,
        );
        let fps = seen.entry(ext.clone()).or_default();
        if fps.contains(&fp) {
            row.status = RowStatus::Duplicate;
            row.note = Some(t_args(
                "nucleo-importacao-nota-fitid-repetido",
                &args(&[("fitid", ext.clone().into())]),
            ));
        } else if fps.is_empty() {
            fps.push(fp);
        } else {
            fps.push(fp);
            let new_ext = format!("{ext}#{}", fps.len());
            row.note = Some(t_args(
                "nucleo-importacao-nota-fitid-colisao",
                &args(&[
                    ("fitid", ext.clone().into()),
                    ("novo", new_ext.clone().into()),
                ]),
            ));
            row.external_id = Some(new_ext);
            collisions += 1;
        }
    }
    if collisions > 0 {
        warnings.push(t_args(
            "nucleo-importacao-aviso-fitid-colisoes",
            &args(&[("n", collisions.into())]),
        ));
    }

    // 2. Mesmo id externo já gravado na conta.
    let mut by_ext =
        conn.prepare_cached("SELECT id FROM txn WHERE account_id = ?1 AND external_id = ?2")?;
    for r in rows.iter_mut().filter(|r| r.status == RowStatus::New) {
        if let Some(ext) = &r.external_id
            && let Some(id) = by_ext
                .query_row(params![account_id, ext], |x| x.get::<_, Id>(0))
                .optional()?
        {
            r.status = RowStatus::Duplicate;
            r.duplicate_of = Some(id);
            r.note = Some(t("nucleo-importacao-nota-ja-importado"));
        }
    }

    // 2b. Mesmo id externo em outra conta (da mesma instituição, ou id longo): o arquivo de uma
    // conta importado em outra, como o CSV do cartão na conta à vista. Não grava por padrão.
    let mut elsewhere: HashMap<String, usize> = HashMap::new();
    for r in rows.iter_mut().filter(|r| r.status == RowStatus::New) {
        if let Some(ext) = r.external_id.clone()
            && let Some((id, name)) =
                transactions::duplicates::in_other_account(conn, account_id, &ext)?
        {
            r.status = RowStatus::Duplicate;
            r.duplicate_of = Some(id);
            r.note = Some(t_args(
                "nucleo-importacao-nota-em-outra-conta",
                &args(&[("conta", name.clone().into())]),
            ));
            *elsewhere.entry(name).or_default() += 1;
        }
    }
    if let Some((name, n)) = elsewhere.into_iter().max_by_key(|(_, n)| *n) {
        warnings.push(t_args(
            "nucleo-importacao-aviso-outra-conta",
            &args(&[("n", n.into()), ("conta", name.into())]),
        ));
    }

    // 3. Possíveis duplicados: impressão digital ou data+valor+descrição parecida.
    let mut consumed: HashSet<Id> = HashSet::new();
    let mut by_fp = conn.prepare_cached(
        "SELECT id FROM txn WHERE account_id = ?1 AND fingerprint = ?2 AND status <> 'planned'
          ORDER BY created_at, id",
    )?;
    let mut by_day = conn.prepare_cached(
        "SELECT id, description FROM txn WHERE account_id = ?1 AND date = ?2 AND amount = ?3 AND status <> 'planned'
          ORDER BY created_at, id",
    )?;
    // The cash entry the app created for a portfolio operation (the sale or purchase of an asset,
    // a distribution received): the statement brings the same money under the bank's own text
    // ("Resgate RDB"), so the descriptions never look alike.
    let mut by_operation = conn.prepare_cached(
        "SELECT t.id, t.description FROM txn t
          WHERE t.account_id = ?1 AND t.date = ?2 AND t.amount = ?3 AND t.status <> 'planned'
            AND t.external_id IS NULL
            AND (EXISTS (SELECT 1 FROM trade tr WHERE tr.txn_id = t.id)
              OR EXISTS (SELECT 1 FROM income_event ie WHERE ie.txn_id = t.id))
          ORDER BY t.created_at, t.id",
    )?;
    for r in rows.iter_mut().filter(|r| r.status == RowStatus::New) {
        let (Some(date), Some(amount)) = (r.date.as_deref(), r.amount) else {
            continue;
        };
        let fp = transactions::fingerprint(date, amount, &r.description);
        let ids: Vec<Id> = by_fp
            .query_map(params![account_id, fp], |x| x.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let mut hit = ids.into_iter().find(|id| !consumed.contains(id));
        if hit.is_none() {
            let same_day: Vec<(Id, String)> = by_day
                .query_map(params![account_id, date, amount], |x| {
                    Ok((x.get(0)?, x.get(1)?))
                })?
                .collect::<rusqlite::Result<_>>()?;
            hit = same_day
                .into_iter()
                .find(|(id, d)| !consumed.contains(id) && text::very_similar(d, &r.description))
                .map(|(id, _)| id);
        }
        if let Some(id) = hit {
            consumed.insert(id.clone());
            r.status = RowStatus::PossibleDuplicate;
            r.duplicate_of = Some(id);
            r.note = Some(t("nucleo-importacao-nota-possivel-duplicado"));
            continue;
        }
        let operations: Vec<(Id, String)> = by_operation
            .query_map(params![account_id, date, amount], |x| {
                Ok((x.get(0)?, x.get(1)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        if let Some((id, description)) = operations
            .into_iter()
            .find(|(id, _)| !consumed.contains(id))
        {
            consumed.insert(id.clone());
            r.status = RowStatus::PossibleDuplicate;
            r.duplicate_of = Some(id);
            r.note = Some(t_args(
                "nucleo-importacao-nota-caixa-de-operacao",
                &args(&[("descricao", description.into())]),
            ));
        }
    }
    Ok(())
}

/// Previsto que um lançamento real efetiva: mesma conta, mesmo sentido, receita ou despesa
/// (fora transferências e parcelas do cartão), valor até 5% diferente (mínimo R$ 1,00) e data
/// até 7 dias antes ou depois. Entre vários, o de valor mais perto e depois o de data mais
/// perto; `taken` são os já usados por outras linhas.
pub(crate) fn planned_match(
    conn: &Connection,
    account_id: &str,
    date: &str,
    amount: i64,
    taken: &HashSet<Id>,
) -> Result<Option<(Id, String, String)>> {
    if amount == 0 {
        return Ok(None);
    }
    let mut stmt = conn.prepare_cached(
        "SELECT id, description, date FROM txn
          WHERE account_id = ?1 AND status = 'planned' AND kind IN ('expense', 'income')
            AND transfer_id IS NULL AND installment_plan_id IS NULL
            AND (amount < 0) = (?2 < 0)
            AND abs(amount - ?2) <= max(100, abs(?2) * 5 / 100)
            AND abs(julianday(date) - julianday(?3)) <= 7
          ORDER BY abs(amount - ?2), abs(julianday(date) - julianday(?3)), id",
    )?;
    let rows: Vec<(Id, String, String)> = stmt
        .query_map(params![account_id, amount, date], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows.into_iter().find(|(id, ..)| !taken.contains(id)))
}

/// Na prévia: as linhas novas que vão efetivar um previsto ganham o id e uma nota.
fn mark_planned(conn: &Connection, account_id: &str, rows: &mut [PreviewRow]) -> Result<()> {
    let mut taken: HashSet<Id> = HashSet::new();
    for r in rows.iter_mut().filter(|r| r.status == RowStatus::New) {
        let (Some(date), Some(amount)) = (r.date.clone(), r.amount) else {
            continue;
        };
        if r.kind
            .is_some_and(|k| !matches!(k, TxnKind::Expense | TxnKind::Income))
        {
            continue;
        }
        if let Some((id, desc, when)) = planned_match(conn, account_id, &date, amount, &taken)? {
            taken.insert(id.clone());
            let note = t_args(
                "nucleo-importacao-nota-efetiva-previsto",
                &args(&[
                    ("descricao", desc.into()),
                    ("data", transactions::util::date_text(&when).into()),
                ]),
            );
            r.note = Some(match r.note.take() {
                Some(n) => format!("{n} · {note}"),
                None => note,
            });
            r.planned_id = Some(id);
        }
    }
    Ok(())
}

/// Efetiva o previsto: o lançamento real herda categoria, titular e recorrência dele (quando
/// não tem os seus) e o previsto sai. Fica no mesmo conjunto de alterações da importação, então
/// desfazer a importação traz o previsto de volta.
fn settle_planned(tx: &Transaction<'_>, real_id: &str, planned_id: &str) -> Result<bool> {
    let row: Option<(Option<Id>, Option<Id>, Option<Id>)> = tx
        .query_row(
            "SELECT category_id, member_id, recurrence_id FROM txn WHERE id = ?1 AND status = 'planned'",
            [planned_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((category, member, recurrence)) = row else {
        return Ok(false);
    };
    let now = now_utc();
    tx.execute(
        "UPDATE txn SET category_id = coalesce(category_id, ?2),
                        member_id = coalesce(member_id, ?3),
                        member_origin = CASE WHEN member_id IS NULL AND ?3 IS NOT NULL THEN 'manual' ELSE member_origin END,
                        recurrence_id = coalesce(recurrence_id, ?4),
                        updated_at = ?5
          WHERE id = ?1",
        params![real_id, category, member, recurrence, now],
    )?;
    let planned = transactions::load_row(tx, planned_id)?;
    transactions::delete_rows(tx, &[planned], crate::time::today_local())?;
    Ok(true)
}

/// Final do cartão de cada linha: o plástico cadastrado com ele (o membro vem dele). Final
/// desconhecido vira aviso (uma vez por final), para a pessoa cadastrar o adicional; mais de 4
/// dígitos é erro da linha (o app nunca guarda o número inteiro).
fn resolve_plastics(
    conn: &Connection,
    account_id: &str,
    account_kind: AccountKind,
    raw: &[RawRow],
    rows: &mut [PreviewRow],
    warnings: &mut Vec<String>,
) -> Result<()> {
    if account_kind != AccountKind::CreditCard {
        return Ok(());
    }
    let mut unknown: Vec<String> = vec![];
    for (row, r) in rows.iter_mut().zip(raw) {
        let Some(l) = r
            .card_last4
            .as_deref()
            .map(str::trim)
            .filter(|l| !l.is_empty())
        else {
            continue;
        };
        match crate::cards::plastics::normalize_last4(l) {
            Ok(l) => {
                row.card_plastic_id = crate::cards::plastics::find_by_last4(conn, account_id, &l)?;
                if row.card_plastic_id.is_none() && !unknown.contains(&l) {
                    unknown.push(l.clone());
                }
                row.card_last4 = Some(l);
            }
            Err(e) if row.error.is_none() => {
                row.error = Some(e.message().to_string());
                row.status = RowStatus::Error;
            }
            Err(_) => {}
        }
    }
    for l in unknown {
        warnings.push(t_args(
            "nucleo-importacao-aviso-final-desconhecido",
            &crate::i18n::args(&[("final", l.into())]),
        ));
    }
    Ok(())
}

fn build_rows(
    conn: &Connection,
    account_id: &str,
    parsed: &mut Parsed,
    account_kind: AccountKind,
) -> Result<Vec<PendingRow>> {
    // A valid calendar date outside the plausible range (a typo in the year) is a row error:
    // it would be refused when recorded.
    let today = crate::time::today_local();
    for r in parsed.rows.iter_mut().filter(|r| r.error.is_none()) {
        if let Some(date) = &r.date
            && crate::time::parse_date(date)
                .is_ok_and(|d| !crate::time::plausible_record_date(d, today))
        {
            r.error = Some(t_args(
                "nucleo-base-erro-data-improvavel",
                &args(&[("valor", date.clone().into())]),
            ));
        }
    }
    let classified = classify(&mut parsed.rows, account_kind, &mut parsed.warnings);
    let mut rows: Vec<PreviewRow> = parsed
        .rows
        .iter()
        .zip(&classified)
        .enumerate()
        .map(|(i, (r, c))| PreviewRow {
            index: i as u32,
            line: r.line,
            date: r.date.clone(),
            amount: r.amount,
            description: r.description.clone(),
            payee: r.payee.clone(),
            external_id: r.external_id.clone(),
            status: if c.ignored.is_some() {
                RowStatus::Ignored
            } else if r.error.is_some() {
                RowStatus::Error
            } else {
                RowStatus::New
            },
            duplicate_of: None,
            kind: c.kind,
            suggested_category_id: None,
            suggested_member_id: None,
            suggested_tags: vec![],
            suggested_description: None,
            rule_id: None,
            error: if c.ignored.is_some() {
                None
            } else {
                r.error.clone()
            },
            note: c.ignored.clone().or_else(|| c.note.clone()),
            card_last4: None,
            card_plastic_id: None,
            orig_amount: r.orig.as_ref().map(|o| o.0),
            orig_currency: r.orig.as_ref().map(|o| o.1.clone()),
            planned_id: None,
        })
        .collect();
    resolve_plastics(
        conn,
        account_id,
        account_kind,
        &parsed.rows,
        &mut rows,
        &mut parsed.warnings,
    )?;
    dedupe(conn, account_id, &mut rows, &mut parsed.warnings)?;
    mark_planned(conn, account_id, &mut rows)?;

    let set = RuleSet::load_active(conn)?;
    let mut out = Vec::with_capacity(rows.len());
    for (row, raw) in rows.into_iter().zip(&parsed.rows) {
        let mut p = PendingRow {
            row,
            notes: raw.notes.clone(),
            applied_rules: vec![],
            statement_month: None,
        };
        let eligible = matches!(p.row.status, RowStatus::New | RowStatus::PossibleDuplicate)
            && p.row.kind.is_some_and(|k| k != TxnKind::CardPayment);
        if eligible && !set.is_empty() {
            let mut t = NewTxn::new(
                account_id,
                p.row.date.as_deref().unwrap_or(""),
                p.row.amount.unwrap_or(0),
                "",
                p.row.kind.unwrap_or(TxnKind::Expense),
                &p.row.description,
            );
            t.payee = p.row.payee.clone();
            t.notes = p.notes.clone();
            let outcome = set.apply_to_new(&mut t);
            p.row.kind = Some(t.kind);
            p.row.suggested_category_id = t.category_id;
            p.row.suggested_member_id = t.member_id;
            p.row.suggested_tags = outcome.tags.clone().unwrap_or_default();
            p.row.suggested_description = outcome.rename_to.clone();
            p.row.rule_id = outcome.first_rule().cloned();
            p.applied_rules = outcome.applied;
        }
        out.push(p);
    }
    Ok(out)
}

fn counts(rows: &[PendingRow]) -> PreviewCounts {
    let mut c = PreviewCounts::default();
    for r in rows {
        match r.row.status {
            RowStatus::New => c.new += 1,
            RowStatus::Duplicate => c.duplicate += 1,
            RowStatus::PossibleDuplicate => c.possible_duplicate += 1,
            RowStatus::Error => c.error += 1,
            RowStatus::Ignored => c.ignored += 1,
        }
    }
    c
}

// ---------------------------------------------------------------------------
// Comandos

/// `import.detect`: formato, codificação, delimitador, cabeçalho, amostra e preset.
pub fn detect(args: &DetectArgs) -> Result<DetectResult> {
    let bytes = decode_base64(&args.content_base64)?;
    match detect_format(&bytes, &args.filename)? {
        // Fatura em PDF: a amostra das linhas lidas (sem senha; protegido, só o formato).
        ImportFormat::Pdf => {
            // Extrato de conta (PDF com ou sem texto, ou a foto): as linhas do leitor da Caixa;
            // senão, a fatura de cartão.
            let mut rows = vec![];
            let mut layout = None;
            let mut encoding = "pdf";
            if let Ok(p) = parse_bank_pdf(&bytes) {
                layout = crate::layouts::known(Some(crate::layouts::LayoutSubject::BankStatement))
                    .into_iter()
                    .find(|l| l.id == "bank.caixa.extrato-periodo.v1");
                if p.encoding == "ocr" {
                    encoding = "ocr";
                }
                rows = p
                    .rows
                    .into_iter()
                    .filter_map(|r| {
                        Some(super::card_pdf::CardPdfRow {
                            line: r.line.unwrap_or(0),
                            date: crate::time::parse_date(r.date.as_deref()?).ok()?,
                            amount: r.amount?,
                            description: r.description,
                            installment: None,
                            card_last4: None,
                            orig: None,
                        })
                    })
                    .collect();
            } else {
                rows = crate::pdf::text(&bytes, None)
                    .map(|t| super::card_pdf::parse(&t).rows)
                    .unwrap_or_default();
            }
            Ok(DetectResult {
                format: ImportFormat::Pdf,
                encoding: encoding.into(),
                delimiter: None,
                skip_rows: 0,
                has_header: true,
                headers: sample_headers(false),
                sample_rows: rows
                    .iter()
                    .take(10)
                    .map(|r| {
                        vec![
                            fmt_date(r.date),
                            money_text(r.amount, "BRL"),
                            r.description.clone(),
                        ]
                    })
                    .collect(),
                row_count: rows.len() as u32,
                suggested_preset_id: None,
                suggested_mapping: None,
                ofx: None,
                layout,
            })
        }
        ImportFormat::Ofx => {
            let declared = ofx::declared_charset(&bytes);
            let dec = text::decode(&bytes, args.encoding.as_deref(), declared.as_deref())?;
            let doc = parse_statement(&dec.text)?;
            let stmt = pick_statement(&doc, None, false)?;
            // Sem conta ainda: a moeda só decide as casas decimais do saldo; na importação,
            // vale a moeda da conta escolhida.
            let currency = stmt.currency.clone().unwrap_or_else(|| "BRL".into());
            let sample_rows = stmt
                .txns
                .iter()
                .take(10)
                .map(|t| {
                    vec![
                        t.date.map(fmt_date).unwrap_or_default(),
                        t.amount_raw.clone().unwrap_or_default(),
                        combine(t.name.as_deref().or(t.payee.as_deref()), t.memo.as_deref()),
                        t.fitid.clone().unwrap_or_default(),
                    ]
                })
                .collect();
            Ok(DetectResult {
                format: ImportFormat::Ofx,
                encoding: dec.encoding,
                delimiter: None,
                skip_rows: 0,
                has_header: true,
                headers: sample_headers(true),
                sample_rows,
                row_count: stmt.txns.len() as u32,
                suggested_preset_id: None,
                suggested_mapping: None,
                ofx: Some(ofx_info(stmt, doc.statements.len(), &currency)),
                layout: None,
            })
        }
        ImportFormat::Csv => {
            let dec = text::decode(&bytes, args.encoding.as_deref(), None)?;
            let delim = csv_file::detect_delimiter(&dec.text);
            let layout = csv_file::detect_layout(&dec.text, delim)?;
            let body = csv_file::skip_lines(&dec.text, layout.skip_rows);
            let recs = csv_file::read_records(body, delim)?;
            let data = if layout.has_header {
                &recs[recs.len().min(1)..]
            } else {
                &recs[..]
            };
            let suggested = csv_file::suggest_preset(&layout.headers);
            // Sem preset: as colunas adivinhadas pela amostra, com o padrão do país para datas
            // e vírgula decimal (docs/international.md).
            let country = args.country.as_deref().unwrap_or("BR");
            let guessed = || {
                let rows: Vec<Vec<String>> =
                    data.iter().take(50).map(|r| r.cells.clone()).collect();
                let headers = if layout.has_header {
                    layout.headers.clone()
                } else {
                    vec![]
                };
                csv_file::guess_mapping(
                    &headers,
                    &rows,
                    country == "US",
                    !csv_file::DOT_DECIMAL_COUNTRIES.contains(&country),
                )
            };
            let suggested_mapping = suggested
                .as_deref()
                .map(|id| csv_file::preset(id).expect("preset existente").mapping)
                .or_else(guessed)
                .map(|mut m| {
                    m.delimiter = Some(csv_file::delimiter_str(delim));
                    m.skip_rows = Some(layout.skip_rows);
                    m.has_header = layout.has_header;
                    m.encoding = Some(dec.encoding.clone());
                    m
                });
            Ok(DetectResult {
                format: ImportFormat::Csv,
                encoding: dec.encoding.clone(),
                delimiter: Some(csv_file::delimiter_str(delim)),
                skip_rows: layout.skip_rows,
                has_header: layout.has_header,
                headers: layout.headers,
                sample_rows: data.iter().take(10).map(|r| r.cells.clone()).collect(),
                row_count: data.len() as u32,
                suggested_preset_id: suggested,
                suggested_mapping,
                ofx: None,
                layout: None,
            })
        }
    }
}

/// `import.preview`: lê, deduplica, aplica as regras e guarda a prévia na sessão.
pub fn preview(ctx: &mut Ctx<'_>, args: &ImportPreviewArgs) -> Result<ImportPreview> {
    let filename = args.filename.trim();
    if filename.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-arquivo-nome")));
    }
    let bytes = decode_base64(&args.content_base64)?;
    let conn = ctx.conn();
    let (currency, account_kind) = crate::accounts::currency_and_kind(conn, &args.account_id)?;
    let sha = sha256_hex(&bytes);
    let format = match args.format {
        Some(f) => f,
        None => detect_format(&bytes, filename)?,
    };
    let mut card_pdf: Option<super::CardPdfInfo> = None;
    let mut parsed = match format {
        ImportFormat::Ofx => parse_ofx(
            &bytes,
            args.encoding.as_deref(),
            &currency,
            account_kind == AccountKind::CreditCard,
            args.ofx_account.as_deref(),
        )?,
        ImportFormat::Csv => parse_csv(
            &bytes,
            args.encoding.as_deref(),
            &currency,
            args.csv_mapping.as_ref(),
            args.preset_id.as_deref(),
        )?,
        // Conta corrente: o extrato em PDF (ou a foto dele); cartão: a fatura.
        ImportFormat::Pdf if account_kind != AccountKind::CreditCard => parse_bank_pdf(&bytes)?,
        ImportFormat::Pdf => {
            let (parsed, info) = parse_card_pdf(&bytes, args.password.as_deref())?;
            card_pdf = Some(info);
            parsed
        }
    };
    if parsed.rows.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-importacao-erro-nenhum-lancamento",
        )));
    }
    let rows = build_rows(conn, &args.account_id, &mut parsed, account_kind)?;
    let previous = already_imported(conn, &args.account_id, &sha)?;
    if previous.is_some() {
        parsed
            .warnings
            .insert(0, t("nucleo-importacao-aviso-arquivo-repetido"));
    }
    let token = new_id();
    let result = ImportPreview {
        token: token.clone(),
        format: parsed.format,
        encoding: parsed.encoding.clone(),
        account_id: args.account_id.clone(),
        filename: filename.to_string(),
        file_sha256: sha.clone(),
        already_imported: previous.is_some(),
        already_imported_batch_id: previous,
        currency: currency.clone(),
        rows: rows.iter().map(|r| r.row.clone()).collect(),
        counts: counts(&rows),
        ofx: parsed.ofx.clone(),
        csv_mapping: parsed.mapping.clone(),
        preset_id: parsed.preset_id.clone(),
        warnings: parsed.warnings.clone(),
        card_pdf: card_pdf.clone(),
    };
    let store = ctx.scratch.get_mut::<ImportScratch>(SCRATCH_KEY);
    store.previews.push(StoredPreview {
        token,
        account_id: args.account_id.clone(),
        currency,
        filename: filename.to_string(),
        format: parsed.format,
        file_sha256: sha,
        rows,
        ofx: parsed.ofx,
        statement: card_pdf.filter(|c| c.reference_month.is_some()),
        created: Instant::now(),
    });
    if store.previews.len() > MAX_PREVIEWS {
        store.previews.sort_by_key(|p| p.created);
        let excess = store.previews.len() - MAX_PREVIEWS;
        store.previews.drain(..excess);
    }
    Ok(result)
}

/// Prévia de lançamentos já estruturados (lidos de uma página pelo agente): mesma
/// deduplicação, regras e gravação de um extrato, sem arquivo. Datas `YYYY-MM-DD`, valores
/// em centavos com sinal (negativo = saída).
pub fn preview_rows(ctx: &mut Ctx<'_>, args: &super::RowsImportArgs) -> Result<ImportPreview> {
    let source = args.source.trim();
    if source.is_empty() {
        return Err(Error::invalid(t("nucleo-importacao-linhas-erro-origem")));
    }
    if args.rows.is_empty() {
        return Err(Error::invalid(t("nucleo-importacao-linhas-erro-vazio")));
    }
    if args.rows.len() > super::MAX_AGENT_ROWS {
        return Err(Error::invalid(t_args(
            "nucleo-importacao-linhas-erro-maximo",
            &crate::i18n::args(&[("max", super::MAX_AGENT_ROWS.into())]),
        )));
    }
    let conn = ctx.conn();
    let (currency, account_kind) = crate::accounts::currency_and_kind(conn, &args.account_id)?;
    let rows: Vec<RawRow> = args
        .rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let date = r.date.trim().to_string();
            let error = if crate::time::parse_date(&date).is_err() {
                Some(t_args(
                    "nucleo-importacao-linhas-erro-data",
                    &crate::i18n::args(&[("valor", date.clone().into())]),
                ))
            } else if r.description.trim().is_empty() {
                Some(t("nucleo-importacao-linhas-erro-descricao"))
            } else if r.amount == 0 {
                Some(t("nucleo-importacao-valor-zero"))
            } else {
                None
            };
            RawRow {
                line: Some(i as u32 + 1),
                date: Some(date),
                amount: Some(r.amount),
                description: r.description.trim().chars().take(300).collect(),
                payee: r.payee.clone().filter(|p| !p.trim().is_empty()),
                notes: r.notes.clone().filter(|n| !n.trim().is_empty()),
                external_id: r.external_id.clone().filter(|e| !e.trim().is_empty()),
                trn_type: None,
                error,
                card_last4: r.card_last4.clone().or_else(|| args.card_last4.clone()),
                orig: match (r.orig_amount, &r.orig_currency) {
                    (Some(v), Some(c)) if v != 0 && !c.trim().is_empty() => {
                        Some((v.abs(), c.trim().to_ascii_uppercase()))
                    }
                    _ => None,
                },
            }
        })
        .collect();
    let canonical = serde_json::to_vec(&args.rows)?;
    let sha = sha256_hex(&canonical);
    let mut parsed = Parsed {
        format: ImportFormat::Csv,
        encoding: "utf-8".into(),
        rows,
        warnings: vec![],
        ofx: None,
        mapping: None,
        preset_id: None,
    };
    let mut rows = build_rows(conn, &args.account_id, &mut parsed, account_kind)?;
    // Vencimento da fatura informado (por linha ou para o lote): só em cartão.
    let due_month = |d: &str| -> Result<String> {
        crate::time::parse_date(d.trim())
            .map(crate::time::month_of)
            .map_err(|_| {
                Error::invalid(t_args(
                    "nucleo-importacao-linhas-erro-vencimento",
                    &crate::i18n::args(&[("valor", d.trim().to_string().into())]),
                ))
            })
    };
    let lot = args
        .statement_due_date
        .as_deref()
        .filter(|d| !d.trim().is_empty())
        .map(due_month)
        .transpose()?;
    let any_row = args.rows.iter().any(|r| r.statement_due_date.is_some());
    if (lot.is_some() || any_row) && account_kind != AccountKind::CreditCard {
        return Err(Error::invalid(t(
            "nucleo-importacao-linhas-erro-vencimento-cartao",
        )));
    }
    for (p, r) in rows.iter_mut().zip(&args.rows) {
        p.statement_month = match r
            .statement_due_date
            .as_deref()
            .filter(|d| !d.trim().is_empty())
        {
            Some(d) => Some(due_month(d)?),
            None => lot.clone(),
        };
    }
    let filename = format!("agente — {}", source.chars().take(80).collect::<String>());
    let token = new_id();
    let result = ImportPreview {
        token: token.clone(),
        format: parsed.format,
        encoding: parsed.encoding.clone(),
        account_id: args.account_id.clone(),
        filename: filename.clone(),
        file_sha256: sha.clone(),
        already_imported: false,
        already_imported_batch_id: None,
        currency: currency.clone(),
        rows: rows.iter().map(|r| r.row.clone()).collect(),
        counts: counts(&rows),
        ofx: None,
        csv_mapping: None,
        preset_id: None,
        warnings: parsed.warnings.clone(),
        card_pdf: None,
    };
    let store = ctx.scratch.get_mut::<ImportScratch>(SCRATCH_KEY);
    store.previews.push(StoredPreview {
        token,
        account_id: args.account_id.clone(),
        currency,
        filename,
        format: parsed.format,
        file_sha256: sha,
        rows,
        ofx: parsed.ofx,
        statement: None,
        created: Instant::now(),
    });
    if store.previews.len() > MAX_PREVIEWS {
        store.previews.sort_by_key(|p| p.created);
        let excess = store.previews.len() - MAX_PREVIEWS;
        store.previews.drain(..excess);
    }
    Ok(result)
}

/// `what`: monta o detalhe do "não encontrado" ("categoria {id}").
fn ensure_exists(conn: &Connection, table: &str, id: &str, what: fn(&str) -> String) -> Result<()> {
    let ok: bool = conn.query_row(
        &format!("SELECT EXISTS (SELECT 1 FROM {table} WHERE id = ?1)"),
        [id],
        |r| r.get(0),
    )?;
    if ok {
        Ok(())
    } else {
        Err(Error::not_found(what(id)))
    }
}

struct ToInsert {
    txn: NewTxn,
    original_description: String,
    renamed: bool,
    tags: Vec<String>,
    rules: Vec<Id>,
    statement_month: Option<String>,
    planned_id: Option<Id>,
}

/// `import.commit`: grava a prévia num único conjunto de alterações.
pub fn commit(ctx: &mut Ctx<'_>, args: &ImportCommitArgs) -> Result<ImportResult> {
    let preview = ctx
        .scratch
        .get_mut::<ImportScratch>(SCRATCH_KEY)
        .previews
        .iter()
        .find(|p| p.token == args.token)
        .cloned()
        .ok_or_else(|| Error::not_found(t("nucleo-importacao-ref-previa")))?;
    let conn = ctx.conn();
    crate::accounts::currency_and_kind(conn, &preview.account_id)?;
    let skip: HashSet<u32> = args.skip_indexes.iter().flatten().copied().collect();
    let include_possible = args.include_possible_duplicates.unwrap_or(false);
    let overrides: HashMap<u32, &super::RowOverride> = args
        .overrides
        .iter()
        .flatten()
        .map(|o| (o.index, o))
        .collect();
    for o in overrides.values() {
        if o.index as usize >= preview.rows.len() {
            return Err(Error::invalid(t_args(
                "nucleo-importacao-erro-linha-previa",
                &crate::i18n::args(&[("linha", o.index.into())]),
            )));
        }
        if let Some(c) = &o.category_id {
            ensure_exists(conn, "category", c, crate::categories::category_ref)?;
        }
        if let Some(m) = &o.member_id {
            ensure_exists(conn, "member", m, transactions::util::member_ref)?;
        }
    }
    // Sugestões cujas categorias/titulares foram apagados depois da prévia caem fora.
    let exists_cat =
        |id: &str| ensure_exists(conn, "category", id, crate::categories::category_ref).is_ok();
    let exists_member =
        |id: &str| ensure_exists(conn, "member", id, transactions::util::member_ref).is_ok();

    let mut to_insert = Vec::new();
    let mut duplicates = 0u32;
    let mut errors = Vec::new();
    for p in &preview.rows {
        let r = &p.row;
        match r.status {
            RowStatus::Duplicate => duplicates += 1,
            RowStatus::PossibleDuplicate if !include_possible => duplicates += 1,
            RowStatus::Error => errors.push(ImportRowError {
                index: r.index,
                line: r.line,
                message: r.error.clone().unwrap_or_default(),
            }),
            _ => {}
        }
        let insertable = r.status == RowStatus::New
            || (r.status == RowStatus::PossibleDuplicate && include_possible);
        if !insertable || skip.contains(&r.index) {
            continue;
        }
        let (Some(date), Some(amount), Some(kind)) = (r.date.as_deref(), r.amount, r.kind) else {
            continue;
        };
        let o = overrides.get(&r.index);
        let kind = o.and_then(|o| o.kind).unwrap_or(kind);
        let mut t = NewTxn::new(
            &preview.account_id,
            date,
            amount,
            &preview.currency,
            kind,
            &r.description,
        );
        t.payee = r.payee.clone();
        t.external_id = r.external_id.clone();
        // Compra no exterior: valor e moeda de origem (ignorados se não conferem).
        if let Ok((a, c)) = transactions::check_orig(
            amount,
            &preview.currency,
            r.orig_amount,
            r.orig_currency.as_deref(),
        ) {
            (t.orig_amount, t.orig_currency) = (a, c);
        }
        // O plástico do final (cadastrado até agora, inclusive depois da prévia).
        t.card_plastic_id = match &r.card_last4 {
            Some(l) => crate::cards::plastics::find_by_last4(conn, &preview.account_id, l)?,
            None => None,
        };
        t.category_id = o
            .and_then(|o| o.category_id.clone())
            .or_else(|| r.suggested_category_id.clone().filter(|c| exists_cat(c)));
        // Fatura e aporte não usam categoria; transferência (PIX que paga algo) usa.
        if matches!(kind, TxnKind::CardPayment | TxnKind::Investment)
            && o.and_then(|o| o.category_id.as_ref()).is_none()
        {
            t.category_id = None;
        }
        t.member_id = o
            .and_then(|o| o.member_id.clone())
            .or_else(|| r.suggested_member_id.clone().filter(|m| exists_member(m)));
        let new_description = o
            .and_then(|o| o.description.clone())
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty())
            .or_else(|| r.suggested_description.clone());
        let renamed = new_description
            .as_ref()
            .is_some_and(|d| *d != r.description);
        if let Some(d) = new_description {
            t.description = d;
        }
        let mut notes: Vec<String> = p.notes.iter().cloned().collect();
        if renamed {
            notes.push(t_args(
                "nucleo-importacao-nota-descricao-original",
                &crate::i18n::args(&[("descricao", r.description.clone().into())]),
            ));
        }
        t.notes = (!notes.is_empty()).then(|| notes.join(" · "));
        to_insert.push(ToInsert {
            txn: t,
            original_description: r.description.clone(),
            renamed,
            tags: o
                .and_then(|o| o.tags.clone())
                .unwrap_or_else(|| r.suggested_tags.clone()),
            rules: if o.is_some_and(|o| o.category_id.is_some()) {
                vec![]
            } else {
                p.applied_rules.clone()
            },
            statement_month: p.statement_month.clone(),
            // Efetiva o previsto só quando a linha entra como prevista na prévia (nova).
            planned_id: r.planned_id.clone().filter(|_| r.status == RowStatus::New),
        });
    }

    let backup_id = if to_insert.len() > BACKUP_THRESHOLD {
        Some(crate::backup::create(ctx.db, &ctx.paths.backups_dir, BackupKind::PreImport)?.id)
    } else {
        None
    };

    let batch_id = new_id();
    let total_rows = preview.rows.len() as i64;
    // O autor é sempre "importação"; quando um cliente MCP importa, ele fica no detalhe.
    let author = match ctx.actor {
        crate::api::Actor::Mcp { client, .. } => {
            Author::import(format!("{} — {client}", preview.filename))
        }
        _ => Author::import(preview.filename.clone()),
    };
    let reason = t_args(
        "nucleo-importacao-historico",
        &crate::i18n::args(&[("arquivo", preview.filename.clone().into())]),
    );
    let written = ctx.write_as(&author, &reason, |tx| {
        let now = now_utc();
        tx.execute(
            "INSERT INTO import_batch (id, filename, format, file_sha256, account_id, total_rows, inserted, duplicates,
                                       errors, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, '[]', ?7, ?7)",
            params![
                batch_id,
                preview.filename,
                format_str(preview.format),
                preview.file_sha256,
                preview.account_id,
                total_rows,
                now
            ],
        )?;
        let statement_id = match &preview.statement {
            Some(info) => {
                let month = info.reference_month.as_deref().unwrap_or_default();
                let id = crate::cards::ensure_statement(tx, &preview.account_id, month)?;
                if info.total.is_some() {
                    crate::cards::set_informed_total(tx, &id, info.total)?;
                }
                crate::cards::apply_printed(
                    tx,
                    &id,
                    &crate::cards::Printed {
                        due_date: info.due_date.clone(),
                        minimum: info.minimum,
                        credit_limit: info.credit_limit,
                        previous_balance: info.previous_balance,
                    },
                )?;
                Some(id)
            }
            None => None,
        };
        let mut inserted = 0u32;
        let mut late_duplicates = 0u32;
        let mut seen_ext: HashSet<String> = HashSet::new();
        let mut hits: HashMap<Id, i64> = HashMap::new();
        let mut ids: Vec<Id> = Vec::new();
        let mut by_month: HashMap<String, Id> = HashMap::new();
        let mut settled = 0u32;
        for item in &to_insert {
            let mut t = item.txn.clone();
            if let Some(ext) = &t.external_id {
                let taken: bool = tx.query_row(
                    "SELECT EXISTS (SELECT 1 FROM txn WHERE account_id = ?1 AND external_id = ?2)",
                    params![t.account_id, ext],
                    |r| r.get(0),
                )?;
                if taken || !seen_ext.insert(ext.clone()) {
                    late_duplicates += 1;
                    continue;
                }
            }
            t.import_id = Some(batch_id.clone());
            // Fatura informada na linha ou no lote (o agente leu da tela); senão, a da fatura
            // em PDF: tudo na fatura do vencimento impresso (parcelas incluídas).
            if let Some(month) = &item.statement_month {
                let sid = match by_month.get(month) {
                    Some(id) => id.clone(),
                    None => {
                        let id = crate::cards::ensure_statement(tx, &preview.account_id, month)?;
                        by_month.insert(month.clone(), id.clone());
                        id
                    }
                };
                t.statement_id = Some(sid);
            } else if let Some(sid) = &statement_id {
                t.statement_id = Some(sid.clone());
            }
            let id = transactions::insert(tx, &t)?;
            if item.renamed {
                // A impressão digital segue a descrição do banco, para reimportações.
                tx.execute(
                    "UPDATE txn SET fingerprint = ?2 WHERE id = ?1",
                    params![id, transactions::fingerprint(&t.date, t.amount, &item.original_description)],
                )?;
            }
            if !item.tags.is_empty() {
                transactions::replace_tags(tx, &id, &item.tags)?;
            }
            for r in &item.rules {
                *hits.entry(r.clone()).or_default() += 1;
            }
            if let Some(pid) = &item.planned_id
                && settle_planned(tx, &id, pid)?
            {
                settled += 1;
            }
            ids.push(id);
            inserted += 1;
        }
        tx.execute(
            "UPDATE import_batch SET inserted = ?2, duplicates = ?3, errors = ?4 WHERE id = ?1",
            params![
                batch_id,
                inserted,
                duplicates + late_duplicates,
                serde_json::to_string(&errors)?
            ],
        )?;
        crate::rules::record_hits(tx, &hits)?;
        let paid = crate::debt::link_category_payments(tx, &ids)?;
        // Transferências entre contas marcadas como "sempre ligar": já entram ligadas.
        let linked = crate::transactions::transfers::auto_link(
            tx,
            &ids,
            crate::time::today_local(),
        )?;
        Ok((inserted, paid, linked, settled))
    });
    // A conta escolhida para o OFX fica lembrada também quando a importação vira proposta
    // da IA (a aprovação é genérica e não passa por aqui).
    if matches!(written, Ok(_) | Err(Error::Proposed { .. })) {
        remember_ofx_account(ctx.conn(), &preview);
    }
    let rec = written?;
    ctx.scratch
        .get_mut::<ImportScratch>(SCRATCH_KEY)
        .previews
        .retain(|p| p.token != args.token);
    let (inserted, installments_paid, transfers_linked, planned_settled) = rec.value;
    Ok(ImportResult {
        batch_id,
        inserted,
        installments_paid,
        transfers_linked,
        planned_settled,
        skipped: total_rows as u32 - inserted,
        change_set_id: rec.change_set_id,
        backup_id,
    })
}

/// Lembra em que conta o OFX foi gravado (`accounts::remember_ofx`), para a próxima
/// importação do mesmo arquivo já vir com ela. É só uma lembrança: uma falha aqui não
/// desfaz a importação.
fn remember_ofx_account(conn: &Connection, p: &StoredPreview) {
    let Some(info) = p.ofx.as_ref().filter(|_| p.format == ImportFormat::Ofx) else {
        return;
    };
    let args = crate::accounts::OfxMatchArgs {
        bank_id: info.bank_id.clone(),
        branch_id: info.branch_id.clone(),
        account_id: info.account_id.clone(),
        credit_card: Some(info.credit_card),
    };
    let today = crate::time::today_local();
    if let Err(e) = crate::accounts::remember_ofx(conn, &args, &p.account_id, today) {
        tracing::warn!("importação: não lembrei a conta do OFX ({e})");
    }
}

fn format_str(f: ImportFormat) -> &'static str {
    match f {
        ImportFormat::Ofx => "ofx",
        ImportFormat::Csv => "csv",
        ImportFormat::Pdf => "pdf",
    }
}

/// `import.list_batches`: lotes importados, com o conjunto de alterações de cada um.
pub fn list_batches(conn: &Connection, args: &ListBatchesArgs) -> Result<Vec<ImportBatch>> {
    let mut stmt = conn.prepare(
        "SELECT b.id, b.filename, b.format, b.file_sha256, b.account_id, a.name, b.total_rows, b.inserted,
                b.duplicates, b.errors, b.created_at,
                (SELECT ci.change_set_id FROM change_item ci JOIN change_set cs ON cs.id = ci.change_set_id
                  WHERE ci.table_name = 'import_batch' AND ci.row_id = b.id AND ci.op = 'insert'
                  ORDER BY cs.created_at DESC LIMIT 1)
         FROM import_batch b JOIN account a ON a.id = b.account_id
         WHERE (?1 IS NULL OR b.account_id = ?1)
         ORDER BY b.created_at DESC, b.id DESC LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(
            params![args.account_id, args.limit.unwrap_or(100).clamp(1, 1000)],
            |r| {
                let format: String = r.get(2)?;
                let errors: String = r.get(9)?;
                Ok(ImportBatch {
                    id: r.get(0)?,
                    filename: r.get(1)?,
                    format: match format.as_str() {
                        "ofx" => ImportFormat::Ofx,
                        "pdf" => ImportFormat::Pdf,
                        _ => ImportFormat::Csv,
                    },
                    file_sha256: r.get(3)?,
                    account_id: r.get(4)?,
                    account_name: r.get(5)?,
                    total_rows: r.get(6)?,
                    inserted: r.get(7)?,
                    duplicates: r.get(8)?,
                    errors: serde_json::from_str(&errors).unwrap_or_default(),
                    created_at: r.get(10)?,
                    change_set_id: r.get(11)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
