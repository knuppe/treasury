//! Extrato de previdência (docs/brazil/pension-statement.md): lê o "Extrato Financeiro do
//! Participante" de fundos de pensão (layout da Quanta Previdência) em PDF ou em texto, mostra
//! a prévia e grava no ativo de previdência: o plano (tipo, regime, taxas, adesão), cada
//! contribuição como compra de cotas (a do participante e a extra do empregador), a cota de
//! cada contribuição e a do extrato como cotação, e, se pedido, os seguros de risco
//! (invalidez e morte) como despesa numa conta.
//!
//! A leitura não depende das colunas: cada linha da tabela começa em "MM/AAAA DD/MM/AAAA
//! DD/MM/AAAA" (competência, vencimento, pagamento), seguida do histórico e de nove números
//! (valor pago, aposentadoria, valor rentabilizado, rentabilidade, rentabilidade %, cotas,
//! invalidez, pensão e multa). Históricos quebrados em duas linhas ("Contribuições Extras" /
//! "Empregador") não atrapalham: o tipo sai dos números e do texto da própria linha. Nome e
//! CPF do participante não são lidos.

use std::collections::HashSet;

use chrono::NaiveDate;
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{PensionPlanTerms, PlanType, TaxRegime};
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::fmt_date;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionStatementFile {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionStatementArgs {
    /// O PDF do extrato (o primeiro arquivo)…
    #[serde(default)]
    #[ts(optional)]
    pub files: Option<Vec<PensionStatementFile>>,
    /// …ou o texto já extraído do PDF.
    #[serde(default)]
    #[ts(optional)]
    pub text: Option<String>,
    /// Ativo de previdência do extrato; sem ele, o do mesmo CNPJ ou um novo.
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
    /// Conta das cotas (obrigatória para gravar).
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Lançar os seguros de risco (invalidez e morte) como despesa nesta conta.
    #[serde(default)]
    #[ts(optional)]
    pub risk_account_id: Option<Id>,
    /// Índices da prévia que não devem ser gravados.
    #[serde(default)]
    #[ts(optional)]
    pub skip: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PensionRowKind {
    /// Contribuição do participante (mensal, extra ou portabilidade).
    Participant,
    /// Contribuição extra do empregador ou instituidor.
    Employer,
    /// Seguro de risco de invalidez (não vira cota).
    RiskDisability,
    /// Seguro de risco de morte, a pensão (não vira cota).
    RiskDeath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PensionRowStatus {
    New,
    /// Já importada (mesma competência, pagamento, tipo, valor e cotas).
    Duplicate,
    /// Seguro de risco sem conta para a despesa: só informativo.
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionStatementRow {
    pub index: u32,
    /// `AAAA-MM`.
    pub competence: String,
    pub due_date: String,
    pub paid_date: String,
    pub kind: PensionRowKind,
    pub description: String,
    /// Valor pago (aposentadoria + invalidez + pensão), em centavos.
    #[ts(type = "number")]
    pub paid: i64,
    /// Parte que vira cotas.
    #[ts(type = "number")]
    pub retirement: i64,
    /// Valor das cotas desta contribuição no extrato.
    #[ts(type = "number")]
    pub current_value: i64,
    #[ts(type = "number")]
    pub risk: i64,
    /// Cotas (decimal em texto).
    pub quotas: String,
    /// Preço da cota na contribuição (aposentadoria ÷ cotas).
    pub quota_price: Option<String>,
    pub status: PensionRowStatus,
    pub external_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionStatementHeader {
    pub plan_name: Option<String>,
    pub plan_cnpj: Option<String>,
    pub cnpb: Option<String>,
    pub administrator: Option<String>,
    pub adhesion: Option<String>,
    pub admin_fee_pct: Option<String>,
    pub loading_fee_pct: Option<String>,
    pub tax_regime: Option<TaxRegime>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionStatementTotals {
    #[ts(type = "number")]
    pub participant: i64,
    #[ts(type = "number")]
    pub employer: i64,
    #[ts(type = "number")]
    pub risk: i64,
    /// Soma das cotas das linhas.
    pub quotas: String,
    /// Units the statement accounts for: the ones already recorded in the plan plus the new rows
    /// of this statement (a statement of a few months only lists that period).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub accounted_quotas: Option<String>,
    /// Saldo de cotas, valor e data da cota e saldo do participante, como no rodapé.
    pub quota_balance: Option<String>,
    pub quota_value: Option<String>,
    pub quota_date: Option<String>,
    #[ts(type = "number | null")]
    pub balance: Option<i64>,
    /// Balance from the units (the recorded ones plus the new rows) × the unit value.
    #[ts(type = "number | null")]
    pub computed_balance: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionStatementPreview {
    pub header: PensionStatementHeader,
    pub rows: Vec<PensionStatementRow>,
    pub totals: PensionStatementTotals,
    /// Ativo que recebe o extrato (o pedido ou o do mesmo CNPJ); `None`: será criado.
    pub asset_id: Option<Id>,
    pub asset_name: String,
    pub warnings: Vec<String>,
    /// Layout reconhecido (docs/layouts.md).
    pub layout: crate::layouts::LayoutInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionStatementResult {
    pub asset_id: Id,
    pub created_asset: bool,
    pub contributions: u32,
    pub skipped: u32,
    pub risk_expenses: u32,
    pub prices: u32,
}

// ---------------------------------------------------------------------------
// Leitura

fn re(s: &str) -> Regex {
    Regex::new(s).expect("regex válida")
}

/// `1.234,56` → centavos.
fn money(s: &str) -> Option<i64> {
    crate::money::parse_amount(s, "BRL").ok()
}

/// `22.222,318823` → decimal.
fn decimal(s: &str) -> Option<Decimal> {
    s.trim()
        .trim_end_matches('%')
        .replace('.', "")
        .replace(',', ".")
        .parse()
        .ok()
}

fn date(s: &str) -> Option<String> {
    NaiveDate::parse_from_str(s, "%d/%m/%Y").ok().map(fmt_date)
}

/// O valor de um rótulo: logo depois dele; senão, colado antes (`38.912,62SALDO PARTICIPANTE`,
/// como o leitor de PDF do app entrega); senão, o primeiro até `window` caracteres adiante.
fn after(text: &str, label: &Regex, pattern: &Regex, window: usize) -> Option<String> {
    let m = label.find(text)?;
    let next = text[m.end()..].trim_start_matches([':', ' ', '\n', '\t']);
    // Valor colado ao rótulo seguinte (`0,25% a.aTaxa de…`) é daquele rótulo.
    if let Some(v) = pattern
        .find(next)
        .filter(|v| v.start() == 0 && !next[v.end()..].starts_with(|c: char| c.is_alphabetic()))
    {
        return Some(v.as_str().to_string());
    }
    let before = text[..m.start()].trim_end();
    if let Some(v) = pattern
        .find_iter(before)
        .filter(|v| v.end() == before.len())
        .last()
    {
        return Some(v.as_str().to_string());
    }
    let rest = &text[m.end()..];
    let end = rest
        .char_indices()
        .nth(window)
        .map_or(rest.len(), |(i, _)| i);
    pattern.find(&rest[..end]).map(|x| x.as_str().to_string())
}

/// Texto do PDF (por página, sem quebras de página no meio das linhas).
pub fn pdf_text(bytes: &[u8]) -> Result<String> {
    crate::pdf::pages(bytes).map(|p| p.join("\n")).map_err(|e| {
        Error::invalid(t_args(
            "nucleo-planejamento-previdencia-erro-pdf",
            &args(&[("erro", e.to_string().into())]),
        ))
    })
}

fn parse_header(text: &str) -> PensionStatementHeader {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cnpj = re(r"\d{2}\.\d{3}\.\d{3}/\d{4}-\d{2}");
    let pct = re(r"\d+(,\d+)?\s*%(\s*a\.?\s*a\.?)?");
    let plan_cnpj = after(&flat, &re(r"(?i)CNPJ\s+(do\s+)?Plano"), &cnpj, 400)
        .or_else(|| cnpj.find(&flat).map(|m| m.as_str().to_string()));
    let regime = if re(r"(?i)\bregressiv").is_match(&flat) {
        Some(TaxRegime::Regressive)
    } else if re(r"(?i)\bprogressiv").is_match(&flat) {
        Some(TaxRegime::Progressive)
    } else {
        None
    };
    let fee = |label: &str| {
        after(&flat, &re(label), &pct, 300).map(|p| {
            p.split('%')
                .next()
                .unwrap_or_default()
                .trim()
                .replace(',', ".")
        })
    };
    // `PLANOPREV-Arrojado` (às vezes colado ao rótulo seguinte: `PLANOPREV-ArrojadoPlano:`).
    let plan_name = re(r"\b([A-Z][A-Z0-9]+-[A-Z][a-zà-ü0-9]+)")
        .captures_iter(&flat)
        .map(|c| c[1].to_string())
        .next();
    let administrator = if re(r"(?i)\bquanta\b").is_match(&flat) {
        Some("Quanta Previdência".to_string())
    } else {
        None
    };
    PensionStatementHeader {
        plan_name,
        plan_cnpj,
        cnpb: re(r"\b\d{4}\.\d{4}-\d{2}")
            .find(&flat)
            .map(|m| m.as_str().to_string()),
        administrator,
        adhesion: re(r"(\d{2}/\d{2}/\d{4})\s+00:00(:00)?")
            .captures(&flat)
            .and_then(|c| date(&c[1])),
        admin_fee_pct: fee(r"(?i)Taxa\s+de\s+Administra"),
        loading_fee_pct: fee(r"(?i)Taxa\s+de\s+Carregamento"),
        tax_regime: regime,
    }
}

struct RawRow {
    competence: String,
    due: String,
    paid_date: String,
    inline: String,
    nums: Vec<String>,
}

fn parse_rows(text: &str) -> Vec<RawRow> {
    let toks: Vec<&str> = text.split_whitespace().collect();
    let comp = re(r"^\d{2}/\d{4}$");
    let day = re(r"^\d{2}/\d{2}/\d{4}$");
    let num = re(r"^-?\d{1,3}(\.\d{3})*,\d+%?$");
    let mut out = vec![];
    let mut i = 0;
    while i + 2 < toks.len() {
        if !(comp.is_match(toks[i]) && day.is_match(toks[i + 1]) && day.is_match(toks[i + 2])) {
            i += 1;
            continue;
        }
        let mut j = i + 3;
        let mut inline = vec![];
        while j < toks.len() && !num.is_match(toks[j]) {
            if comp.is_match(toks[j]) && j + 2 < toks.len() && day.is_match(toks[j + 1]) {
                break;
            }
            inline.push(toks[j]);
            j += 1;
        }
        let nums: Vec<String> = toks[j..]
            .iter()
            .take(9)
            .take_while(|t| num.is_match(t))
            .map(|t| t.to_string())
            .collect();
        if nums.len() == 9 && nums[4].ends_with('%') {
            let (m, y) = toks[i].split_once('/').unwrap_or(("01", "1900"));
            out.push(RawRow {
                competence: format!("{y}-{m}"),
                due: date(toks[i + 1]).unwrap_or_default(),
                paid_date: date(toks[i + 2]).unwrap_or_default(),
                inline: inline.join(" "),
                nums,
            });
            i = j + 9;
        } else {
            i += 3;
        }
    }
    out
}

/// Linhas como o leitor de PDF do app entrega o extrato da Quanta: os nove números colados e
/// na ordem inversa (multa, pensão, invalidez, cotas, %, rentabilidade, valor rentabilizado,
/// aposentadoria, valor pago), o histórico e as datas também invertidas (pagamento,
/// vencimento, competência).
fn parse_rows_reversed(text: &str) -> Vec<RawRow> {
    const M: &str = r"-?\d{1,3}(?:\.\d{3})*,\d{2}";
    let row = re(&format!(
        r"(?s)({M})({M})({M})(\d{{1,3}}(?:\.\d{{3}})*,\d{{8}})(-?\d{{1,3}}(?:\.\d{{3}})*,\d{{2}}%)({M})({M})({M})({M})\s*([^\d]*?)\s*(\d{{2}}/\d{{2}}/\d{{4}})(\d{{2}}/\d{{2}}/\d{{4}})(\d{{2}})/(\d{{4}})"
    ));
    row.captures_iter(text)
        .map(|c| RawRow {
            competence: format!("{}-{}", &c[14], &c[13]),
            due: date(&c[12]).unwrap_or_default(),
            paid_date: date(&c[11]).unwrap_or_default(),
            inline: c[10].split_whitespace().collect::<Vec<_>>().join(" "),
            // Na ordem da tabela: pago, aposentadoria, rentabilizado, rentabilidade, %,
            // cotas, invalidez, pensão, multa.
            nums: [9, 8, 7, 6, 5, 4, 3, 2, 1]
                .iter()
                .map(|i| c[*i].to_string())
                .collect(),
        })
        .collect()
}

fn parse_totals(text: &str) -> PensionStatementTotals {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let quota6 = re(r"\d{1,3}(\.\d{3})*,\d{6}\b");
    let quota10 = re(r"\d+,\d{9,10}\b");
    let money_re = re(r"\d{1,3}(\.\d{3})*,\d{2}\b");
    let day = re(r"\d{2}/\d{2}/\d{4}");
    PensionStatementTotals {
        quota_balance: after(&flat, &re(r"(?i)SALDO\s+DE\s+COTAS"), &quota6, 200)
            .and_then(|s| decimal(&s))
            .map(|d| d.normalize().to_string()),
        quota_value: after(&flat, &re(r"(?i)Valor\s+da\s+Cota"), &quota10, 200)
            .and_then(|s| decimal(&s))
            .map(|d| d.normalize().to_string()),
        quota_date: after(&flat, &re(r"(?i)Data\s+da\s+Cota"), &day, 200).and_then(|d| date(&d)),
        balance: after(&flat, &re(r"(?i)SALDO\s+PARTICIPANTE"), &money_re, 200)
            .and_then(|s| money(&s)),
        ..Default::default()
    }
}

/// Layout `pension.quanta.v1-colunas` (docs/layouts.md).
pub fn matches_columns(text: &str) -> bool {
    !parse_rows(text).is_empty()
}

/// Layout `pension.quanta.v1-colado`.
pub fn matches_glued(text: &str) -> bool {
    !parse_rows_reversed(text).is_empty()
}

/// Lê o texto do extrato, pelo layout reconhecido (ou recusa com o motivo).
pub fn parse_with_layout(
    text: &str,
) -> Result<(
    crate::layouts::LayoutInfo,
    PensionStatementHeader,
    Vec<PensionStatementRow>,
    PensionStatementTotals,
)> {
    let layout = crate::layouts::require(
        &crate::layouts::Doc {
            format: crate::layouts::LayoutFormat::Pdf,
            text,
            headers: &[],
        },
        crate::layouts::LayoutSubject::PensionStatement,
    )?;
    let (h, r, t) = parse(text)?;
    Ok((layout, h, r, t))
}

/// Lê o texto do extrato.
pub fn parse(
    text: &str,
) -> Result<(
    PensionStatementHeader,
    Vec<PensionStatementRow>,
    PensionStatementTotals,
)> {
    let (a, b) = (parse_rows(text), parse_rows_reversed(text));
    let raw = if b.len() > a.len() { b } else { a };
    if raw.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-planejamento-previdencia-erro-sem-contribuicoes",
        )));
    }
    let header = parse_header(text);
    let mut totals = parse_totals(text);
    let mut rows = vec![];
    let mut sum_quotas = Decimal::ZERO;
    for (k, r) in raw.into_iter().enumerate() {
        let n = &r.nums;
        let paid = money(&n[0]).unwrap_or(0);
        let retirement = money(&n[1]).unwrap_or(0);
        let current_value = money(&n[2]).unwrap_or(0);
        let quotas = decimal(&n[5]).unwrap_or_default();
        let disability = money(&n[6]).unwrap_or(0);
        let death = money(&n[7]).unwrap_or(0);
        let inline = r.inline.to_lowercase();
        let kind = if retirement > 0 {
            if inline.contains("mensal") || inline.contains("participante") {
                PensionRowKind::Participant
            } else if inline.contains("empregador")
                || inline.contains("instituidor")
                || inline.contains("extra")
                || inline.trim().is_empty()
            {
                PensionRowKind::Employer
            } else {
                PensionRowKind::Participant
            }
        } else if disability > 0 || inline.contains("invalidez") {
            PensionRowKind::RiskDisability
        } else {
            PensionRowKind::RiskDeath
        };
        let description = match kind {
            PensionRowKind::Participant if !r.inline.trim().is_empty() => r.inline.clone(),
            PensionRowKind::Participant => "Contribuição do participante".into(),
            PensionRowKind::Employer => "Contribuição extra do empregador".into(),
            PensionRowKind::RiskDisability => "Contribuição risco invalidez".into(),
            PensionRowKind::RiskDeath => "Contribuição risco morte".into(),
        };
        let quota_price = (quotas > Decimal::ZERO && retirement > 0).then(|| {
            (Decimal::from(retirement) / Decimal::from(100) / quotas)
                .round_dp(10)
                .normalize()
                .to_string()
        });
        if matches!(kind, PensionRowKind::Participant | PensionRowKind::Employer) {
            sum_quotas += quotas;
        }
        let external_id = format!(
            "prev:{}:{}:{}:{}:{}",
            r.competence,
            r.paid_date,
            match kind {
                PensionRowKind::Participant => "p",
                PensionRowKind::Employer => "e",
                PensionRowKind::RiskDisability => "i",
                PensionRowKind::RiskDeath => "m",
            },
            paid,
            quotas.normalize()
        );
        rows.push(PensionStatementRow {
            index: k as u32,
            competence: r.competence,
            due_date: r.due,
            paid_date: r.paid_date,
            kind,
            description,
            paid,
            retirement,
            current_value,
            risk: disability + death,
            quotas: quotas.normalize().to_string(),
            quota_price,
            status: PensionRowStatus::New,
            external_id,
        });
        match kind {
            PensionRowKind::Participant => totals.participant += retirement,
            PensionRowKind::Employer => totals.employer += retirement,
            _ => totals.risk += disability + death,
        }
    }
    totals.quotas = sum_quotas.normalize().to_string();
    totals.computed_balance = totals
        .quota_value
        .as_deref()
        .and_then(|v| v.parse::<Decimal>().ok())
        .map(|v| {
            (sum_quotas * v * Decimal::from(100))
                .round()
                .try_into()
                .unwrap_or(0)
        });
    Ok((header, rows, totals))
}

fn text_of(a: &PensionStatementArgs) -> Result<String> {
    if let Some(t) = a.text.as_deref().filter(|t| !t.trim().is_empty()) {
        return Ok(t.to_string());
    }
    let f = a
        .files
        .as_deref()
        .and_then(|f| f.first())
        .ok_or_else(|| Error::invalid(t("nucleo-planejamento-previdencia-erro-sem-arquivo")))?;
    let bytes = crate::transactions::b64::decode(&f.content_base64)?;
    pdf_text(&bytes)
}

// ---------------------------------------------------------------------------
// Prévia e gravação

fn digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}

/// O ativo do extrato: o pedido (precisa ser de previdência) ou o de mesmo CNPJ.
fn target_asset(
    conn: &Connection,
    a: &PensionStatementArgs,
    h: &PensionStatementHeader,
) -> Result<Option<(Id, String)>> {
    if let Some(id) = &a.asset_id {
        let asset = crate::assets::get(conn, id)?;
        if asset.class != AssetClass::Pension {
            return Err(Error::invalid(t(
                "nucleo-planejamento-previdencia-erro-extrato-classe",
            )));
        }
        return Ok(Some((asset.id, asset.name)));
    }
    let Some(cnpj) = h.plan_cnpj.as_deref().map(digits) else {
        return Ok(None);
    };
    let mut stmt = conn.prepare("SELECT id, name, cnpj FROM asset WHERE class = 'pension'")?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, Id>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .find(|(_, _, c)| c.as_deref().map(digits).as_deref() == Some(cnpj.as_str()))
        .map(|(id, name, _)| (id, name)))
}

fn existing_ids(conn: &Connection, asset_id: Option<&str>) -> Result<HashSet<String>> {
    let Some(id) = asset_id else {
        return Ok(HashSet::new());
    };
    let mut stmt = conn
        .prepare("SELECT external_id FROM trade WHERE asset_id = ?1 AND external_id IS NOT NULL")?;
    let ids = stmt
        .query_map([id], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<HashSet<_>>>()?;
    Ok(ids)
}

pub fn preview(conn: &Connection, a: &PensionStatementArgs) -> Result<PensionStatementPreview> {
    let text = text_of(a)?;
    let (layout, header, mut rows, mut totals) = parse_with_layout(&text)?;
    let target = target_asset(conn, a, &header)?;
    let seen = existing_ids(conn, target.as_ref().map(|t| t.0.as_str()))?;
    for r in &mut rows {
        r.status = match r.kind {
            PensionRowKind::RiskDisability | PensionRowKind::RiskDeath
                if a.risk_account_id.is_none() =>
            {
                PensionRowStatus::Info
            }
            _ if seen.contains(&r.external_id) => PensionRowStatus::Duplicate,
            _ => PensionRowStatus::New,
        };
    }
    // The statement's balance is the whole position and its rows may be only a period (the last
    // months): what must add up to it is what the plan already has plus the new rows.
    let held = match &target {
        Some((id, _)) => crate::portfolio::engine::quantity_held(
            conn,
            id,
            None,
            &crate::time::fmt_date(crate::time::today_local()),
        )?,
        None => Decimal::ZERO,
    };
    let new_quotas: Decimal = rows
        .iter()
        .filter(|r| {
            r.status == PensionRowStatus::New
                && matches!(
                    r.kind,
                    PensionRowKind::Participant | PensionRowKind::Employer
                )
        })
        .filter_map(|r| r.quotas.parse::<Decimal>().ok())
        .sum();
    let accounted = held + new_quotas;
    totals.accounted_quotas = Some(accounted.normalize().to_string());
    totals.computed_balance = totals
        .quota_value
        .as_deref()
        .and_then(|v| v.parse::<Decimal>().ok())
        .map(|v| {
            (accounted * v * Decimal::from(100))
                .round()
                .try_into()
                .unwrap_or(0)
        });
    let mut warnings = vec![];
    if let Some(b) = totals.quota_balance.as_deref()
        && let Ok(b) = b.parse::<Decimal>()
        && (b - accounted).abs() > Decimal::new(1, 4)
    {
        warnings.push(t_args(
            "nucleo-planejamento-previdencia-aviso-cotas",
            &args(&[
                (
                    "soma",
                    decimal_label(&accounted.normalize().to_string()).into(),
                ),
                ("saldo", decimal_label(&b.to_string()).into()),
            ]),
        ));
    }
    if let (Some(b), Some(c)) = (totals.balance, totals.computed_balance)
        && (b - c).abs() > 100
    {
        let l = crate::i18n::current();
        warnings.push(t_args(
            "nucleo-planejamento-previdencia-aviso-saldo",
            &args(&[
                ("calculado", crate::i18n::money(l, c, "BRL").into()),
                ("extrato", crate::i18n::money(l, b, "BRL").into()),
            ]),
        ));
    }
    if header.plan_cnpj.is_none() {
        warnings.push(t("nucleo-planejamento-previdencia-aviso-sem-cnpj"));
    }
    let asset_name = target
        .as_ref()
        .map(|t| t.1.clone())
        .or_else(|| header.plan_name.clone())
        .unwrap_or_else(|| t("nucleo-planejamento-previdencia-ativo-padrao"));
    Ok(PensionStatementPreview {
        header,
        rows,
        totals,
        asset_id: target.map(|t| t.0),
        asset_name,
        warnings,
        layout,
    })
}

/// Categoria `pai › filha` pelos nomes em pt-BR (vale com as categorias em qualquer idioma).
fn find_category(tx: &Transaction<'_>, parent: &str, child: &str) -> Result<Option<Id>> {
    let path = format!("{parent}{}{child}", crate::categories::PATH_SEP);
    crate::categories::find_br_path(tx, None, &path)
}

/// Erro interno ao montar o aporte ou a despesa do extrato.
fn internal_error(e: impl std::fmt::Display) -> String {
    t_args(
        "nucleo-planejamento-previdencia-erro-interno",
        &args(&[("erro", e.to_string().into())]),
    )
}

/// Número decimal (`"12.5"`) com o separador do idioma.
fn decimal_label(s: &str) -> String {
    crate::ai::alerts::decimal(crate::i18n::current(), s)
}

/// Grava o extrato (dentro de `ctx.write`).
pub fn import(
    tx: &Transaction<'_>,
    a: &PensionStatementArgs,
    today: NaiveDate,
) -> Result<PensionStatementResult> {
    let account = a
        .account_id
        .as_deref()
        .ok_or_else(|| Error::invalid(t("nucleo-planejamento-previdencia-erro-conta")))?;
    crate::accounts::currency_and_kind(tx, account)?;
    let p = preview(tx, a)?;
    let h = &p.header;
    let skip: HashSet<u32> = a.skip.clone().unwrap_or_default().into_iter().collect();
    let terms = |existing: Option<super::PensionPlan>| PensionPlanTerms {
        plan_type: existing.as_ref().map_or(PlanType::Efpc, |e| e.plan_type),
        tax_regime: h
            .tax_regime
            .or(existing.as_ref().map(|e| e.tax_regime))
            .unwrap_or(TaxRegime::Regressive),
        insurer: h
            .administrator
            .clone()
            .or(existing.as_ref().and_then(|e| e.insurer.clone())),
        admin_fee_pct: h
            .admin_fee_pct
            .clone()
            .or(existing.as_ref().and_then(|e| e.admin_fee_pct.clone())),
        loading_fee_pct: h
            .loading_fee_pct
            .clone()
            .or(existing.as_ref().and_then(|e| e.loading_fee_pct.clone())),
        start_date: h
            .adhesion
            .clone()
            .or(existing.as_ref().and_then(|e| e.start_date.clone())),
        notes: existing.and_then(|e| e.notes),
    };
    let (asset_id, created_asset) = match &p.asset_id {
        Some(id) => {
            let existing = super::get_plan(tx, id)?;
            super::set_plan(tx, &terms(existing).for_asset(id))?;
            (id.clone(), false)
        }
        None => {
            let mut input = serde_json::json!({
                "name": p.asset_name,
                "class": "pension",
                "price_provider": "manual",
                "notes": h.cnpb.as_ref().map(|c| format!("CNPB {c}")),
                "pension": terms(None),
            });
            if let Some(c) = &h.plan_cnpj {
                input["cnpj"] = serde_json::json!(c);
            }
            let input =
                serde_json::from_value(input).map_err(|e| Error::Other(internal_error(e)))?;
            (crate::assets::create(tx, &input)?, true)
        }
    };
    let (mut contributions, mut skipped, mut risk_expenses) = (0, 0, 0);
    let mut prices = vec![];
    let risk_cat = match &a.risk_account_id {
        Some(_) => find_category(tx, "Seguros", "Seguro de vida")?,
        None => None,
    };
    for r in &p.rows {
        if skip.contains(&r.index) || r.status == PensionRowStatus::Duplicate {
            skipped += 1;
            continue;
        }
        match r.kind {
            PensionRowKind::Participant | PensionRowKind::Employer => {
                let Some(price) = &r.quota_price else {
                    skipped += 1;
                    continue;
                };
                let trade = serde_json::json!({
                    "asset_id": asset_id,
                    "account_id": account,
                    "date": r.paid_date,
                    "side": "buy",
                    "quantity": r.quotas,
                    "price": price,
                    "cash_movement": false,
                    "external_id": r.external_id,
                    "notes": t_args(
                        "nucleo-planejamento-previdencia-nota-aporte",
                        &args(&[
                            ("descricao", r.description.clone().into()),
                            ("competencia", r.competence.clone().into()),
                        ]),
                    ),
                });
                crate::portfolio::records::create_trade(
                    tx,
                    &serde_json::from_value(trade).map_err(|e| Error::Other(internal_error(e)))?,
                )?;
                contributions += 1;
                if let (Ok(d), Ok(c)) = (crate::time::parse_date(&r.paid_date), price.parse()) {
                    prices.push((d, c));
                }
            }
            PensionRowKind::RiskDisability | PensionRowKind::RiskDeath => {
                let Some(acc) = &a.risk_account_id else {
                    continue;
                };
                let exists: bool = tx
                    .query_row(
                        "SELECT 1 FROM txn WHERE account_id = ?1 AND date = ?2 AND amount = ?3 AND description = ?4",
                        params![acc, r.paid_date, -r.risk, r.description],
                        |_| Ok(true),
                    )
                    .optional()?
                    .unwrap_or(false);
                if exists {
                    skipped += 1;
                    continue;
                }
                let mut v = serde_json::json!({
                    "account_id": acc,
                    "date": r.paid_date,
                    "amount": -r.risk,
                    "kind": "expense",
                    "status": "cleared",
                    "description": r.description,
                    "notes": t_args(
                        "nucleo-planejamento-previdencia-nota-risco",
                        &args(&[
                            ("plano", p.asset_name.clone().into()),
                            ("competencia", r.competence.clone().into()),
                        ]),
                    ),
                });
                if let Some(c) = &risk_cat {
                    v["category_id"] = serde_json::json!(c);
                }
                crate::transactions::create(
                    tx,
                    &serde_json::from_value(v).map_err(|e| Error::Other(internal_error(e)))?,
                    today,
                )?;
                risk_expenses += 1;
            }
        }
    }
    // The unit value printed on the statement, on its date (or today: the statement is the
    // newest thing the family has when the date did not come).
    if let Some(Ok(v)) = p.totals.quota_value.as_deref().map(str::parse) {
        let d = p
            .totals
            .quota_date
            .as_deref()
            .and_then(|d| crate::time::parse_date(d).ok())
            .unwrap_or(today);
        prices.push((d, v));
    }
    let rows: Vec<crate::market::PriceRow> = prices
        .iter()
        .map(|(d, c)| crate::market::PriceRow {
            date: *d,
            open: None,
            high: None,
            low: None,
            close: *c,
            adj_close: None,
            volume: None,
        })
        .collect();
    let n_prices = crate::market::upsert_prices(tx, &asset_id, &rows, "manual")? as u32;
    Ok(PensionStatementResult {
        asset_id,
        created_asset,
        contributions,
        skipped,
        risk_expenses,
        prices: n_prices,
    })
}
