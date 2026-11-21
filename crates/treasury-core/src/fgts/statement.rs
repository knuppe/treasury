//! Extrato "Histórico" da conta vinculada (Caixa), em PDF ou texto (docs/brazil/fgts.md, seção 3):
//! cabeçalho (empregador, CNPJ, datas, taxa de juros, valor para fins rescisórios) e as linhas
//! `DD/MM/AAAA HISTÓRICO R$ VALOR R$ TOTAL`, com o saldo corrido na ordem impressa (que não é
//! a ordem das datas). Nome, PIS/PASEP e carteira de trabalho não são lidos; o nº da conta só
//! vira hash.
//!
//! A leitura não depende de colunas: cada lançamento é uma data seguida de dois valores em
//! reais; o histórico é o texto entre eles ou, vazio, a última linha antes da data (a
//! distribuição de resultado vem em duas linhas, com a data e os valores no meio, e o
//! "12/AAAA" do ano-base logo depois).

use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use super::{EntryKind, data};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, parse_date};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsStatementFile")]
pub struct StatementFile {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsStatementArgs")]
pub struct StatementArgs {
    /// O PDF do extrato (o primeiro arquivo)…
    #[serde(default)]
    #[ts(optional)]
    pub files: Option<Vec<StatementFile>>,
    /// …ou o texto já extraído do PDF.
    #[serde(default)]
    #[ts(optional)]
    pub text: Option<String>,
    /// Conta FGTS do extrato; sem ela, a do mesmo nº de conta ou CNPJ, ou uma nova.
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Titular da conta nova (o membro do vínculo).
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Nome da conta nova (padrão: "FGTS · <empregador>").
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
    /// Motivo de cada saque (chave: `key` do grupo na prévia).
    #[serde(default)]
    #[ts(optional)]
    pub reasons: Option<BTreeMap<String, String>>,
    /// Conta que recebeu cada saque (chave: `key` do grupo): vira transferência.
    #[serde(default)]
    #[ts(optional)]
    pub destinations: Option<BTreeMap<String, Id>>,
    /// Dados do vínculo informados à parte: valem sobre os lidos do cabeçalho (para o que o
    /// extrato não trouxe ou a leitura errou).
    #[serde(default)]
    #[ts(optional)]
    pub employer_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub employer_cnpj: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub admission_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub option_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub leave_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub leave_code: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub category: Option<String>,
    /// Renda CLT do vínculo (para conferir os depósitos com o holerite).
    #[serde(default)]
    #[ts(optional)]
    pub income_source_id: Option<Id>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsStatementHeader")]
pub struct StatementHeader {
    pub employer_name: Option<String>,
    pub employer_cnpj: Option<String>,
    pub admission_date: Option<String>,
    pub option_date: Option<String>,
    pub leave_date: Option<String>,
    pub leave_code: Option<String>,
    pub category: Option<String>,
    /// Juros ao ano ("3").
    pub interest_rate: Option<String>,
    /// Valor para fins rescisórios (centavos).
    #[ts(type = "number | null")]
    pub rescission_base: Option<i64>,
    /// Data de emissão do extrato (ou a do último lançamento).
    pub statement_date: Option<String>,
    /// O nº da conta foi achado (só o hash é guardado).
    pub has_account_number: bool,
    #[serde(skip)]
    #[ts(skip)]
    pub account_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsStatementLine")]
pub struct StatementLine {
    /// Ordem no extrato (1, 2, …).
    pub seq: u32,
    pub date: String,
    pub history: String,
    pub kind: EntryKind,
    #[ts(type = "number")]
    pub amount: i64,
    /// Saldo impresso na linha.
    #[ts(type = "number")]
    pub total: i64,
    pub competence: Option<String>,
    pub coefficient: Option<String>,
    pub base_year: Option<i32>,
    pub withdrawal_code: Option<String>,
    pub withdrawal_ref: Option<String>,
    /// Grupo do saque (as partes de depósito e de JAM do mesmo saque).
    pub group: Option<String>,
    /// Já existe na conta (não entra de novo).
    pub exists: bool,
    /// JAM calculado pela regra (só nos créditos de JAM conferidos).
    #[ts(type = "number | null")]
    pub jam_expected: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsWithdrawalGroup")]
pub struct WithdrawalGroup {
    pub key: String,
    pub date: String,
    pub code: Option<String>,
    pub reference: Option<String>,
    /// Total sacado (positivo).
    #[ts(type = "number")]
    pub amount: i64,
    /// Motivo pelo código (o 99 é tratado como moradia) ou o informado; `null`: perguntar.
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsStatementPreview")]
pub struct StatementPreview {
    pub header: StatementHeader,
    pub lines: Vec<StatementLine>,
    /// Saldo antes do primeiro lançamento do extrato.
    #[ts(type = "number")]
    pub opening_balance: i64,
    #[ts(type = "number")]
    pub final_balance: i64,
    /// Cada saldo impresso = o anterior + o valor da linha.
    pub balance_ok: bool,
    pub balance_mismatches: u32,
    /// Conta reconhecida (`null`: será criada).
    pub account_id: Option<Id>,
    pub account_name: Option<String>,
    pub new_count: u32,
    pub existing_count: u32,
    /// Créditos de JAM conferidos e os que diferem do calculado.
    pub jam_checked: u32,
    pub jam_mismatches: u32,
    pub withdrawals: Vec<WithdrawalGroup>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsStatementImport")]
pub struct StatementImport {
    pub account_id: Id,
    pub inserted: u32,
    pub skipped: u32,
    pub preview: StatementPreview,
}

static DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{2})/(\d{2})/(\d{4})\b").unwrap());
static MONEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(-?)\s*R\$\s*(-?)\s*(\d{1,3}(?:\.\d{3})*,\d{2})").unwrap());
/// Lançamento: data, texto, valor e saldo.
static ENTRY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)\b(\d{2}/\d{2}/\d{4})\b([^\n]*?)((?:-\s*)?R\$\s*-?\s*\d{1,3}(?:\.\d{3})*,\d{2})\s+((?:-\s*)?R\$\s*-?\s*\d{1,3}(?:\.\d{3})*,\d{2})",
    )
    .unwrap()
});
static CNPJ: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d{2}\.\d{3}\.\d{3}/\d{4}-\d{2}").unwrap());
static ACCOUNT_NO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{3,5}/\d{6,}(?:-\d)?\b").unwrap());
static RATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d{1,2}(?:[.,]\d+)?)\s*%").unwrap());
static INLINE_BASE_YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"12/(\d{4})").unwrap());
static BASE_YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*12/(\d{4})\b").unwrap());
static COEF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+,\d{4,})").unwrap());
static CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bCOD\.?\s*([0-9]{1,3}[A-Z]?)\b(?:\s+(\S+))?").unwrap());
static ISSUED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)emi\w*\s+em\s*:?\s*(\d{2}/\d{2}/\d{4})").unwrap());

const MONTHS: [&str; 12] = [
    "janeiro",
    "fevereiro",
    "marco",
    "abril",
    "maio",
    "junho",
    "julho",
    "agosto",
    "setembro",
    "outubro",
    "novembro",
    "dezembro",
];

fn money(s: &str) -> Option<i64> {
    let c = MONEY.captures(s)?;
    let neg = !c[1].is_empty() || !c[2].is_empty();
    let digits: String = c[3].chars().filter(char::is_ascii_digit).collect();
    let v: i64 = digits.parse().ok()?;
    Some(if neg { -v } else { v })
}

fn date_iso(s: &str) -> Option<String> {
    let c = DATE.captures(s)?;
    let d = NaiveDate::from_ymd_opt(c[3].parse().ok()?, c[2].parse().ok()?, c[1].parse().ok()?)?;
    Some(fmt_date(d))
}

/// Competência de "115-DEPOSITO MARCO 2024", "DEPOSITO MAR/2024" ou "DEPOSITO 03/2024".
fn competence(history: &str) -> Option<String> {
    let f = crate::import::text::fold(history);
    let words: Vec<&str> = f.split(' ').collect();
    for (i, w) in words.iter().enumerate() {
        let month = MONTHS
            .iter()
            .position(|m| m == w || (w.len() == 3 && m.starts_with(w)));
        if let (Some(m), Some(y)) = (month, words.get(i + 1))
            && y.len() == 4
            && y.chars().all(|c| c.is_ascii_digit())
        {
            return Some(format!("{y}-{:02}", m + 1));
        }
        if w.len() <= 2
            && w.chars().all(|c| c.is_ascii_digit())
            && let Some(y) = words.get(i + 1)
            && y.len() == 4
            && y.chars().all(|c| c.is_ascii_digit())
            && let Ok(m) = w.parse::<u32>()
            && (1..=12).contains(&m)
        {
            return Some(format!("{y}-{m:02}"));
        }
    }
    None
}

/// Tipo da linha pelo histórico.
pub fn classify(history: &str, amount: i64) -> EntryKind {
    let f = crate::import::text::fold(history);
    if f.contains("dist") && f.contains("resultado") {
        EntryKind::Distribution
    } else if f.contains("reposicao") {
        EntryKind::Reversal
    } else if f.contains("saque") && !f.contains("cancelamento") {
        EntryKind::Withdrawal
    } else if f.contains("jam") && (f.contains("regulariza") || f.contains("cancelamento")) {
        EntryKind::JamAdjustment
    } else if f.contains("jam") {
        EntryKind::Jam
    } else if f.contains("deposito") && f.starts_with("650") {
        EntryKind::DepositExtra
    } else if f.contains("deposito") {
        EntryKind::Deposit
    } else if amount < 0 {
        EntryKind::Withdrawal
    } else {
        EntryKind::JamAdjustment
    }
}

fn is_label_line(l: &str) -> bool {
    let f = crate::import::text::fold(l);
    [
        "data lancamentos",
        "data historico",
        "valor total",
        "lancamentos valor",
    ]
    .iter()
    .any(|k| f.contains(k))
}

/// Par "valor saldo" solto, com ou sem "R$" (a linha que a leitura por data não pegou).
static PAIR: LazyLock<Regex> = LazyLock::new(|| {
    let v = r"(?:-\s*)?(?:R\$\s*-?\s*)?\b\d{1,3}(?:\.\d{3})*,\d{2}\b";
    Regex::new(&format!(r"({v})\s+({v})")).unwrap()
});

/// Valor em reais, com ou sem "R$".
fn amount_of(s: &str) -> Option<i64> {
    if let Some(v) = money(s) {
        return Some(v);
    }
    let neg = s.trim_start().starts_with('-');
    let digits: String = s.chars().filter(char::is_ascii_digit).collect();
    let v: i64 = digits.parse().ok()?;
    Some(if neg { -v } else { v })
}

/// Texto sem datas, valores, rótulos da tabela e o "12/AAAA" do ano-base (o histórico de uma
/// linha recuperada).
fn loose_history(s: &str) -> String {
    let s = DATE.replace_all(s, " ");
    let s = PAIR.replace_all(&s, " ");
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !is_label_line(l) && !BASE_YEAR.is_match(l))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lançamentos fora do padrão "data texto valor saldo" num trecho entre dois que estão nele: a
/// data, o histórico e os valores em linhas diferentes (a distribuição de resultado vem assim
/// em alguns extratos). Só vale o par que fecha com o saldo anterior ou com o seguinte, então
/// um valor solto qualquer não vira lançamento. Devolve (data, histórico, valor, saldo, fim).
fn recover(
    gap: &str,
    mut prev_total: i64,
    next: Option<(i64, i64)>,
) -> Vec<(String, String, i64, i64, usize)> {
    let mut out = vec![];
    let mut from = 0;
    for c in PAIR.captures_iter(gap) {
        let whole = c.get(0).expect("m");
        let (Some(amount), Some(total)) = (amount_of(&c[1]), amount_of(&c[2])) else {
            continue;
        };
        let closes_next = next.is_some_and(|(a, t)| t - a == total);
        if prev_total + amount != total && !closes_next {
            continue;
        }
        // A data: a última antes do par neste pedaço ou, sem ela, a primeira depois.
        let before = &gap[from..whole.start()];
        let date = DATE
            .find_iter(before)
            .last()
            .or_else(|| DATE.find(&gap[whole.end()..]))
            .and_then(|d| date_iso(d.as_str()));
        let Some(date) = date else { continue };
        let mut history = loose_history(before);
        // O "12/AAAA" do ano-base entre a data e os valores ("…ANO BASE" numa linha, "12/AAAA
        // R$ … R$ …" na outra) é desta linha, não o fim da anterior.
        let after_date = DATE
            .find_iter(before)
            .last()
            .map_or("", |d| &before[d.end()..]);
        if let Some(y) = after_date.lines().find_map(|l| BASE_YEAR.captures(l)) {
            history = format!("{history} 12/{}", &y[1]);
        }
        out.push((date, history, amount, total, whole.end()));
        prev_total = total;
        from = whole.end();
    }
    out
}

/// Monta a linha classificada (competência, coeficiente, ano-base, código do saque).
fn make_line(seq: u32, date: String, history: &str, amount: i64, total: i64) -> StatementLine {
    let history = history.split_whitespace().collect::<Vec<_>>().join(" ");
    let kind = classify(&history, amount);
    let mut line = StatementLine {
        seq,
        date,
        history,
        kind,
        amount,
        total,
        competence: None,
        coefficient: None,
        base_year: None,
        withdrawal_code: None,
        withdrawal_ref: None,
        group: None,
        exists: false,
        jam_expected: None,
    };
    match kind {
        EntryKind::Deposit | EntryKind::DepositExtra => {
            line.competence = competence(&line.history);
        }
        EntryKind::Jam => {
            line.coefficient = COEF.captures(&line.history).map(|m| m[1].replace(',', "."));
        }
        EntryKind::Distribution => {
            line.base_year = INLINE_BASE_YEAR
                .captures(&line.history)
                .and_then(|m| m[1].parse().ok());
        }
        EntryKind::Withdrawal => {
            if let Some(m) = CODE.captures(&line.history) {
                line.withdrawal_code = Some(m[1].to_ascii_uppercase());
                line.withdrawal_ref = m
                    .get(2)
                    .map(|x| x.as_str().to_string())
                    .filter(|r| r.chars().any(|c| c.is_ascii_digit()));
            }
            line.group = Some(format!(
                "{}|{}|{}",
                line.date,
                line.withdrawal_code.as_deref().unwrap_or(""),
                line.withdrawal_ref.as_deref().unwrap_or("")
            ));
        }
        _ => {}
    }
    line
}

/// "12/AAAA" logo no começo do trecho depois da distribuição: o ano-base dela.
fn take_base_year(lines: &mut [StatementLine], after: &str) {
    if let Some(last) = lines.last_mut()
        && last.kind == EntryKind::Distribution
        && last.base_year.is_none()
        && let Some(y) = BASE_YEAR.captures(after.trim_start())
    {
        last.base_year = y[1].parse().ok();
    }
}

/// Lê o extrato. Erro quando não há lançamento nenhum.
pub fn parse(text: &str) -> Result<(StatementHeader, Vec<StatementLine>)> {
    let matches: Vec<_> = ENTRY.captures_iter(text).collect();
    if matches.is_empty() {
        return Err(Error::invalid(t("nucleo-fgts-erro-extrato-vazio")));
    }
    let first_start = matches[0].get(0).expect("m").start();
    let header = parse_header(&text[..first_start], text);
    let mut lines: Vec<StatementLine> = vec![];
    let mut prev_end = first_start;
    let values = |c: &regex::Captures<'_>| (money(&c[3]).unwrap_or(0), money(&c[4]).unwrap_or(0));
    for c in &matches {
        let whole = c.get(0).expect("m");
        let gap = &text[prev_end..whole.start()];
        let mut cut = 0;
        if let Some(prev_total) = lines.last().map(|l| l.total) {
            for (date, history, amount, total, end) in recover(gap, prev_total, Some(values(c))) {
                take_base_year(&mut lines, &gap[cut..]);
                let seq = lines.len() as u32 + 1;
                lines.push(make_line(seq, date, &history, amount, total));
                cut = end;
            }
        }
        let pre = &gap[cut..];
        take_base_year(&mut lines, pre);
        let inner = c[2].trim();
        let history = if inner.is_empty() {
            pre.lines()
                .map(str::trim)
                .rfind(|l| !l.is_empty() && !is_label_line(l) && !BASE_YEAR.is_match(l))
                .unwrap_or("")
                .to_string()
        } else {
            inner.to_string()
        };
        let (amount, total) = values(c);
        let seq = lines.len() as u32 + 1;
        lines.push(make_line(
            seq,
            date_iso(&c[1]).unwrap_or_default(),
            &history,
            amount,
            total,
        ));
        prev_end = whole.end();
    }
    // Depois do último lançamento no padrão: os que vierem fora dele e o ano-base no fim.
    let tail = &text[prev_end..];
    let mut cut = 0;
    let prev_total = lines.last().map_or(0, |l| l.total);
    for (date, history, amount, total, end) in recover(tail, prev_total, None) {
        take_base_year(&mut lines, &tail[cut..]);
        let seq = lines.len() as u32 + 1;
        lines.push(make_line(seq, date, &history, amount, total));
        cut = end;
    }
    take_base_year(&mut lines, &tail[cut..]);
    let mut header = header;
    if header.statement_date.is_none() {
        header.statement_date = lines.iter().map(|l| l.date.clone()).max();
    }
    Ok((header, lines))
}

fn parse_header(head: &str, all: &str) -> StatementHeader {
    let mut h = StatementHeader {
        statement_date: ISSUED.captures(all).and_then(|c| date_iso(&c[1])),
        ..Default::default()
    };
    // A data de emissão não é data do vínculo.
    let head = ISSUED.replace_all(head, " ");
    let lines: Vec<&str> = head
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let joined = lines.join("\n");
    let cnpj = find_cnpj(&joined);
    if let Some((s, e, formatted)) = &cnpj {
        h.employer_cnpj = Some(formatted.clone());
        // Nome antes do CNPJ na mesma linha, que pode quebrar na seguinte (sem números).
        let line_start = joined[..*s].rfind('\n').map_or(0, |p| p + 1);
        let line_end = joined[*e..].find('\n').map_or(joined.len(), |p| e + p);
        let mut name = joined[line_start..*s].trim().to_string();
        if !name.is_empty()
            && let Some(next) = joined[line_end..]
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
            && !next.chars().any(|c| c.is_ascii_digit())
            && !is_header_label(next)
        {
            name = format!("{name} {next}");
        }
        let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
        let has_letters = name.chars().any(char::is_alphabetic);
        h.employer_name = (has_letters && !is_header_label(&name)).then_some(name);
    }
    if h.employer_name.is_none() {
        h.employer_name = employer_below_label(&lines);
    }
    // Datas do vínculo: admissão, opção e afastamento (com o código), nessa ordem, a partir do
    // rótulo da admissão ou do CNPJ.
    let anchor = ADMISSION
        .find(&joined)
        .map(|m| m.end())
        .or(cnpj.as_ref().map(|c| c.1))
        .unwrap_or(0);
    let after = &joined[anchor..];
    let mut dates = DATE.find_iter(after);
    h.admission_date = dates.next().and_then(|d| date_iso(d.as_str()));
    h.option_date = dates.next().and_then(|d| date_iso(d.as_str()));
    if let Some(d) = dates.next() {
        h.leave_date = date_iso(d.as_str());
        h.leave_code = after[d.end()..]
            .split_whitespace()
            .next()
            .filter(|c| c.len() <= 3 && c.chars().all(|x| x.is_ascii_alphanumeric()))
            .filter(|c| c.chars().any(|x| x.is_ascii_alphabetic()))
            .map(str::to_string);
    }
    let flat = joined.split_whitespace().collect::<Vec<_>>().join(" ");
    let digits_of = |s: &str| -> String { s.chars().filter(char::is_ascii_digit).collect() };
    let account = ACCOUNT_NO
        .find(&flat)
        .map(|m| digits_of(m.as_str()))
        .or_else(|| account_below_label(&flat));
    if let Some(digits) = account {
        h.has_account_number = true;
        h.account_hash = Some(hex::encode(Sha256::digest(digits.as_bytes())));
    }
    h.interest_rate = RATE
        .captures(&flat)
        .map(|c| c[1].replace(',', ".").trim_end_matches(".0").to_string());
    h.rescission_base = MONEY.find(&flat).and_then(|m| money(m.as_str()));
    h.category = category_below_label(&flat);
    h
}

static ADMISSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)admiss\S*").unwrap());
static CATEGORY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bcategoria\b").unwrap());
static ACCOUNT_LABEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bconta\b(?:\s+vinculada)?").unwrap());
/// CNPJ sem a pontuação (14 dígitos seguidos, ou 13 quando o zero da frente caiu), aceito só
/// com os dígitos verificadores certos.
static CNPJ_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b\d{13,14}\b").unwrap());

/// Os 14 dígitos do CNPJ sem pontuação (com o zero da frente que o extrato tirou), se conferem.
fn cnpj_digits(d: &str) -> Option<String> {
    let d = match d.len() {
        13 => format!("0{d}"),
        14 => d.to_string(),
        _ => return None,
    };
    crate::assets::enrich::valid_cnpj(&d).then_some(d)
}

/// Posição e o CNPJ formatado: com a pontuação, ou os 13 ou 14 dígitos que conferem.
fn find_cnpj(text: &str) -> Option<(usize, usize, String)> {
    if let Some(m) = CNPJ.find(text) {
        return Some((m.start(), m.end(), m.as_str().to_string()));
    }
    CNPJ_DIGITS
        .find_iter(text)
        .find_map(|m| cnpj_digits(m.as_str()).map(|d| (m, d)))
        .map(|(m, d)| {
            (
                m.start(),
                m.end(),
                format!(
                    "{}.{}.{}/{}-{}",
                    &d[..2],
                    &d[2..5],
                    &d[5..8],
                    &d[8..12],
                    &d[12..]
                ),
            )
        })
}

/// Empregador na linha de baixo do rótulo (ou depois dos dois-pontos): o texto até o primeiro
/// número, completado pela linha seguinte quando ela não tem números nem é rótulo.
fn employer_below_label(lines: &[&str]) -> Option<String> {
    let i = lines.iter().position(|l| {
        crate::import::text::fold(l)
            .trim_start()
            .starts_with("empregador")
    })?;
    let label = lines[i];
    let inline = label
        .split_once(':')
        .map(|(_, rest)| rest.trim())
        .filter(|r| !r.is_empty());
    let (first, rest) = match inline {
        Some(r) => (r, &lines[i + 1..]),
        None => (*lines.get(i + 1)?, lines.get(i + 2..).unwrap_or(&[])),
    };
    if is_header_label(first) {
        return None;
    }
    let words: Vec<&str> = first
        .split_whitespace()
        .take_while(|w| !w.chars().any(|c| c.is_ascii_digit()))
        .collect();
    if words.is_empty() {
        return None;
    }
    let mut name = words.join(" ");
    let complete = words.len() == first.split_whitespace().count();
    if let Some(next) = rest.first().filter(|n| !is_header_label(n)) {
        // A continuação vem sozinha ou, quando o nome quebra antes da data de admissão, com
        // ela na mesma linha ("EXEMPLO 03/02/2020 …"): só as palavras antes do primeiro número.
        let more: Vec<&str> = next
            .split_whitespace()
            .take_while(|w| !w.chars().any(|c| c.is_ascii_digit()))
            .collect();
        let whole_line = more.len() == next.split_whitespace().count();
        if !more.is_empty() && (whole_line || complete) {
            name = format!("{name} {}", more.join(" "));
        }
    }
    Some(name.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Categoria: o primeiro número curto (1 a 3 dígitos, sozinho) depois do rótulo. Datas e
/// valores no meio (os rótulos em linha, os valores na de baixo) não contam.
fn category_below_label(flat: &str) -> Option<String> {
    let m = CATEGORY.find(flat)?;
    flat[m.end()..]
        .split_whitespace()
        .take(12)
        .map(|w| w.trim_matches(|c: char| c == ':' || c == ';' || c == ','))
        .find(|w| (1..=3).contains(&w.len()) && w.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_string)
}

/// Nº da conta noutro formato: depois de um rótulo "conta", o primeiro número de 12 dígitos ou
/// mais (com `/`, `.` ou `-` no meio) que não é data nem CNPJ. PIS e carteira têm menos.
fn account_below_label(flat: &str) -> Option<String> {
    ACCOUNT_LABEL.find_iter(flat).find_map(|m| {
        flat[m.end()..]
            .split_whitespace()
            .take(12)
            .filter(|w| {
                w.chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, '/' | '.' | '-'))
            })
            .filter(|w| !DATE.is_match(w) && !CNPJ.is_match(w))
            .map(|w| w.chars().filter(char::is_ascii_digit).collect::<String>())
            .find(|d| d.len() >= 12 && cnpj_digits(d).is_none())
    })
}

fn is_header_label(l: &str) -> bool {
    const WORDS: &[&str] = &[
        "empregador",
        "inscricao",
        "admissao",
        "opcao",
        "afastamento",
        "categoria",
        "conta",
        "taxa",
        "rescisorios",
        "trabalhador",
        "pis",
        "pasep",
        "optante",
    ];
    crate::import::text::fold(l)
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| WORDS.contains(&w))
}

/// JAM pela regra (docs/brazil/fgts.md, seção 4): truncar(base × coeficiente), com a base = saldo
/// até o crédito anterior (pela data) mais regularizações e reposições do período, menos os
/// saques do período. Devolve, para cada linha de JAM com crédito anterior, o calculado.
pub fn expected_jam(lines: &[StatementLine], opening: i64) -> Vec<Option<i64>> {
    let mut out = vec![None; lines.len()];
    let jams: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.kind == EntryKind::Jam && l.coefficient.is_some())
        .map(|(i, _)| i)
        .collect();
    for w in jams.windows(2) {
        let (p, k) = (w[0], w[1]);
        let (dp, dk) = (&lines[p].date, &lines[k].date);
        let mut base = opening;
        for l in lines {
            // Até o crédito anterior, tudo; no período, regularizações e reposições (antes do
            // crédito) e saques (até o dia do crédito).
            let counts = &l.date <= dp
                || (&l.date < dk
                    && matches!(l.kind, EntryKind::JamAdjustment | EntryKind::Reversal))
                || (&l.date <= dk && l.kind == EntryKind::Withdrawal);
            if counts {
                base += l.amount;
            }
        }
        // Créditos de JAM do mesmo dia do anterior (não acontece) ficam de fora.
        let coef: f64 = lines[k]
            .coefficient
            .as_deref()
            .and_then(|c| c.parse().ok())
            .unwrap_or(0.0);
        let v = (base.max(0) as f64 * coef + 1e-6).floor() as i64;
        out[k] = Some(v);
    }
    out
}

/// Texto do extrato: o do PDF e, junto com ele, o texto informado (que só completa o
/// cabeçalho); sem PDF, o texto informado.
fn texts_of(a: &StatementArgs) -> Result<(String, Option<String>)> {
    let given = a
        .text
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .map(str::to_string);
    let Some(f) = a.files.as_deref().and_then(|f| f.first()) else {
        return given
            .map(|t| (t, None))
            .ok_or_else(|| Error::invalid(t("nucleo-fgts-erro-sem-arquivo")));
    };
    let bytes = crate::transactions::b64::decode(&f.content_base64)?;
    // Texto puro (o anexo de texto chega assim) ou PDF.
    let text = if bytes.starts_with(b"%PDF") {
        crate::pdf::pages(&bytes)
            .map(|p| p.join("\n"))
            .map_err(|e| {
                Error::invalid(t_args(
                    "nucleo-fgts-erro-pdf",
                    &args(&[("erro", e.to_string().into())]),
                ))
            })?
    } else {
        match String::from_utf8(bytes) {
            Ok(t) => t,
            Err(e) => {
                return Err(Error::invalid(t_args(
                    "nucleo-fgts-erro-pdf",
                    &args(&[("erro", e.to_string().into())]),
                )));
            }
        }
    };
    Ok((text, given))
}

/// Completa o cabeçalho lido do PDF com o do texto informado (só o que faltou).
fn fill_header(h: &mut StatementHeader, extra: &str) {
    let head_end = ENTRY.find(extra).map_or(extra.len(), |m| m.start());
    let x = parse_header(&extra[..head_end], extra);
    macro_rules! fill {
        ($($f:ident),*) => { $( if h.$f.is_none() { h.$f = x.$f.clone(); } )* };
    }
    fill!(
        employer_name,
        employer_cnpj,
        admission_date,
        option_date,
        leave_date,
        leave_code,
        category,
        interest_rate,
        rescission_base,
        statement_date,
        account_hash
    );
    h.has_account_number = h.account_hash.is_some();
}

/// Os dados do vínculo informados à parte valem sobre os do cabeçalho.
fn apply_overrides(h: &mut StatementHeader, a: &StatementArgs) -> Result<()> {
    let clean = |v: &Option<String>| {
        v.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    for (slot, v) in [
        (&mut h.admission_date, &a.admission_date),
        (&mut h.option_date, &a.option_date),
        (&mut h.leave_date, &a.leave_date),
    ] {
        if let Some(d) = clean(v) {
            *slot = Some(fmt_date(parse_date(&d)?));
        }
    }
    if let Some(c) = clean(&a.employer_cnpj) {
        h.employer_cnpj = Some(find_cnpj(&c).map_or(c, |x| x.2));
    }
    for (slot, v) in [
        (&mut h.employer_name, &a.employer_name),
        (&mut h.leave_code, &a.leave_code),
        (&mut h.category, &a.category),
    ] {
        if let Some(x) = clean(v) {
            *slot = Some(x);
        }
    }
    Ok(())
}

/// Conta FGTS do extrato: a pedida, a do mesmo nº de conta, a do mesmo CNPJ, a que já tem
/// estes lançamentos ou a do mesmo empregador.
fn target_account(
    conn: &Connection,
    a: &StatementArgs,
    h: &StatementHeader,
    lines: &[StatementLine],
) -> Result<Option<(Id, String)>> {
    if let Some(id) = &a.account_id {
        let (_, kind) = crate::accounts::currency_and_kind(conn, id)?;
        if kind != crate::accounts::AccountKind::Fgts {
            return Err(Error::invalid(t("nucleo-fgts-erro-nao-e-fgts")));
        }
        let name: String =
            conn.query_row("SELECT name FROM account WHERE id = ?1", [id], |r| r.get(0))?;
        return Ok(Some((id.clone(), name)));
    }
    let by = |sql: &str, v: &str| -> Result<Option<(Id, String)>> {
        Ok(conn
            .query_row(sql, [v], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()?)
    };
    if let Some(hash) = &h.account_hash
        && let Some(x) = by(
            "SELECT a.id, a.name FROM fgts_account f JOIN account a ON a.id = f.account_id
              WHERE f.account_hash = ?1",
            hash,
        )?
    {
        return Ok(Some(x));
    }
    if let Some(cnpj) = &h.employer_cnpj
        && let Some(x) = by(
            "SELECT a.id, a.name FROM fgts_account f JOIN account a ON a.id = f.account_id
              WHERE f.employer_cnpj = ?1 AND a.archived = 0 ORDER BY a.created_at LIMIT 1",
            cnpj,
        )?
    {
        return Ok(Some(x));
    }
    // Sem nº da conta nem CNPJ no cabeçalho: a conta que já tem os lançamentos deste extrato
    // (o mesmo PDF importado de novo), e por fim a do mesmo empregador pelo nome.
    if let Some(x) = by_entries(conn, lines)? {
        return Ok(Some(x));
    }
    if let Some(name) = &h.employer_name {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, f.employer_name FROM fgts_account f
               JOIN account a ON a.id = f.account_id
              WHERE a.archived = 0 AND f.employer_name IS NOT NULL ORDER BY a.created_at",
        )?;
        let want = crate::import::text::fold(name);
        let found = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, Id>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .find(|(_, _, e)| crate::import::text::fold(e) == want)
            .map(|(id, n, _)| (id, n));
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

/// A conta FGTS com mais lançamentos iguais aos do extrato (data, tipo, valor e saldo
/// impresso): pelo menos três, ou todos quando o extrato é curto. Ativas antes das arquivadas.
fn by_entries(conn: &Connection, lines: &[StatementLine]) -> Result<Option<(Id, String)>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name FROM fgts_account f JOIN account a ON a.id = f.account_id
          ORDER BY a.archived, a.created_at",
    )?;
    let accounts = stmt
        .query_map([], |r| Ok((r.get::<_, Id>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let need = lines.len().min(3);
    let mut best: Option<(usize, (Id, String))> = None;
    for acc in accounts {
        let keys = existing_keys(conn, &acc.0)?;
        let n = lines.iter().filter(|l| keys.contains(&key_of(l))).count();
        if n >= need.max(1) && best.as_ref().is_none_or(|(b, _)| n > *b) {
            best = Some((n, acc));
        }
    }
    Ok(best.map(|(_, a)| a))
}

fn key_of(l: &StatementLine) -> EntryKey {
    (
        l.date.clone(),
        l.kind.as_str().to_string(),
        l.amount,
        Some(l.total),
    )
}

/// Data, tipo, valor e saldo impresso de um lançamento (para não importar de novo).
type EntryKey = (String, String, i64, Option<i64>);

/// Lançamentos já gravados na conta.
fn existing_keys(conn: &Connection, account_id: &str) -> Result<HashSet<EntryKey>> {
    let mut stmt = conn.prepare(
        "SELECT t.date, e.kind, t.amount, e.statement_total FROM fgts_entry e
           JOIN txn t ON t.id = e.txn_id WHERE e.account_id = ?1",
    )?;
    let rows = stmt
        .query_map([account_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<HashSet<_>>>()?;
    Ok(rows)
}

pub fn preview(conn: &Connection, a: &StatementArgs) -> Result<StatementPreview> {
    super::require_brazil(conn)?;
    let (text, extra) = texts_of(a)?;
    let (mut header, mut lines) = parse(&text)?;
    if let Some(x) = &extra {
        fill_header(&mut header, x);
    }
    apply_overrides(&mut header, a)?;
    let opening = lines[0].total - lines[0].amount;
    let mut mismatches = 0;
    let mut running = opening;
    for l in &lines {
        running += l.amount;
        if running != l.total {
            mismatches += 1;
            running = l.total;
        }
    }
    let expected = expected_jam(&lines, opening);
    let mut jam_checked = 0;
    let mut jam_mismatches = 0;
    for (l, e) in lines.iter_mut().zip(expected) {
        if let Some(e) = e {
            jam_checked += 1;
            if e != l.amount {
                jam_mismatches += 1;
            }
            l.jam_expected = Some(e);
        }
    }
    let target = target_account(conn, a, &header, &lines)?;
    let mut existing = 0;
    if let Some((id, _)) = &target {
        let keys = existing_keys(conn, id)?;
        for l in &mut lines {
            if keys.contains(&key_of(l)) {
                l.exists = true;
                existing += 1;
            }
        }
    }
    // Saques agrupados (partes de depósito e de JAM do mesmo saque).
    let mut groups: BTreeMap<String, WithdrawalGroup> = BTreeMap::new();
    for l in lines.iter().filter(|l| l.kind == EntryKind::Withdrawal) {
        let key = l.group.clone().unwrap_or_default();
        let g = groups
            .entry(key.clone())
            .or_insert_with(|| WithdrawalGroup {
                key: key.clone(),
                date: l.date.clone(),
                code: l.withdrawal_code.clone(),
                reference: l.withdrawal_ref.clone(),
                amount: 0,
                reason: a
                    .reasons
                    .as_ref()
                    .and_then(|r| r.get(&key).cloned())
                    .or_else(|| {
                        l.withdrawal_code
                            .as_deref()
                            .and_then(data::reason_of_code)
                            .map(str::to_string)
                    }),
            });
        g.amount += -l.amount;
    }
    let mut warnings = vec![];
    if mismatches > 0 {
        warnings.push(t_args(
            "nucleo-fgts-aviso-saldo",
            &args(&[("n", mismatches.into())]),
        ));
    }
    if jam_mismatches > 0 {
        warnings.push(t_args(
            "nucleo-fgts-aviso-jam",
            &args(&[("n", jam_mismatches.into())]),
        ));
    }
    if header.interest_rate.as_deref().is_some_and(|r| r != "3") {
        warnings.push(t("nucleo-fgts-aviso-juros-progressivos"));
    }
    if groups.values().any(|g| g.code.as_deref() == Some("99")) {
        warnings.push(t("nucleo-fgts-aviso-codigo-99"));
    }
    if header.employer_name.is_none()
        || header.employer_cnpj.is_none()
        || header.admission_date.is_none()
    {
        warnings.push(t("nucleo-fgts-aviso-cabecalho"));
    }
    let final_balance = lines.last().map_or(opening, |l| l.total);
    Ok(StatementPreview {
        new_count: lines.len() as u32 - existing,
        existing_count: existing,
        account_id: target.as_ref().map(|t| t.0.clone()),
        account_name: target.map(|t| t.1),
        header,
        opening_balance: opening,
        final_balance,
        balance_ok: mismatches == 0,
        balance_mismatches: mismatches,
        jam_checked,
        jam_mismatches,
        withdrawals: groups.into_values().collect(),
        warnings,
        lines,
    })
}

/// Grava a prévia: a conta (nova ou a reconhecida), os dados do vínculo e os lançamentos novos.
pub fn import(
    tx: &Transaction<'_>,
    a: &StatementArgs,
    today: NaiveDate,
) -> Result<StatementImport> {
    let p = preview(tx, a)?;
    let account_id = match &p.account_id {
        Some(id) => {
            reuse_account(tx, id, a, today)?;
            id.clone()
        }
        None => {
            let employer = p
                .header
                .employer_name
                .clone()
                .unwrap_or_else(|| t("nucleo-fgts-empregador"));
            let name = a
                .name
                .clone()
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| format!("FGTS · {employer}"));
            let first = p.lines.iter().map(|l| l.date.as_str()).min().unwrap_or("");
            crate::accounts::create(
                tx,
                &crate::accounts::AccountInput {
                    name,
                    kind: crate::accounts::AccountKind::Fgts,
                    currency: Some("BRL".into()),
                    institution_id: None,
                    institution_name: Some("Caixa Econômica Federal".into()),
                    member_ids: a.member_id.clone().map(|m| vec![m]),
                    opening_balance: Some(p.opening_balance),
                    opening_date: (!first.is_empty()).then(|| first.to_string()),
                    include_in_net_worth: Some(true),
                    color: None,
                    notes: None,
                    card: None,
                    overdraft: None,
                    branch: None,
                    account_number: None,
                    retirement_plan: None,
                    retirement_coverage: None,
                },
                today,
            )?
        }
    };
    let h = &p.header;
    let now = crate::time::now_utc();
    tx.execute(
        "UPDATE fgts_account SET
            employer_name = COALESCE(?2, employer_name), employer_cnpj = COALESCE(?3, employer_cnpj),
            admission_date = COALESCE(?4, admission_date), option_date = COALESCE(?5, option_date),
            leave_date = COALESCE(?6, leave_date), leave_code = COALESCE(?7, leave_code),
            category = COALESCE(?8, category), interest_rate = COALESCE(?9, interest_rate),
            account_hash = COALESCE(?10, account_hash),
            rescission_base = COALESCE(?11, rescission_base),
            rescission_base_date = CASE WHEN ?11 IS NULL THEN rescission_base_date ELSE ?12 END,
            last_statement_date = COALESCE(?12, last_statement_date), updated_at = ?13
          WHERE account_id = ?1",
        params![
            account_id,
            h.employer_name,
            h.employer_cnpj,
            h.admission_date,
            h.option_date,
            h.leave_date,
            h.leave_code,
            h.category,
            h.interest_rate,
            h.account_hash,
            h.rescission_base,
            h.statement_date,
            now
        ],
    )?;
    if let Some(src) = a
        .income_source_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        let patch = super::AccountPatch {
            account_id: account_id.clone(),
            income_source_id: Some(Some(src.to_string())),
            ..Default::default()
        };
        super::update_account(tx, &patch)?;
    }
    let reasons: BTreeMap<String, Option<String>> = p
        .withdrawals
        .iter()
        .map(|g| (g.key.clone(), g.reason.clone()))
        .collect();
    let mut inserted = 0;
    for l in p.lines.iter().filter(|l| !l.exists) {
        let dest = l
            .group
            .as_ref()
            .and_then(|g| a.destinations.as_ref().and_then(|d| d.get(g)))
            .filter(|_| l.kind == EntryKind::Withdrawal && l.amount < 0);
        let entry = super::NewEntry {
            kind: l.kind,
            competence: l.competence.clone(),
            coefficient: l.coefficient.clone(),
            base_year: l.base_year,
            withdrawal_code: l.withdrawal_code.clone(),
            withdrawal_reason: l
                .group
                .as_ref()
                .and_then(|g| reasons.get(g).cloned().flatten()),
            withdrawal_ref: l.withdrawal_ref.clone(),
            statement_total: Some(l.total),
            statement_seq: Some(l.seq),
        };
        super::insert_entry(
            tx,
            &account_id,
            &l.date,
            l.amount,
            &l.history,
            &entry,
            dest.map(String::as_str),
            today,
        )?;
        inserted += 1;
    }
    Ok(StatementImport {
        account_id,
        inserted,
        skipped: p.existing_count,
        preview: p,
    })
}

/// Conta reconhecida numa nova importação: recebe o titular informado quando ainda não tem
/// nenhum, e o nome informado.
fn reuse_account(
    tx: &Transaction<'_>,
    id: &str,
    a: &StatementArgs,
    today: NaiveDate,
) -> Result<()> {
    let has_member: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM account_member WHERE account_id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    let mut patch = serde_json::json!({ "id": id });
    if let Some(m) = a.member_id.as_deref().filter(|m| !m.trim().is_empty())
        && !has_member
    {
        patch["member_ids"] = serde_json::json!([m]);
    }
    if let Some(n) = a.name.as_deref().filter(|n| !n.trim().is_empty()) {
        patch["name"] = serde_json::json!(n.trim());
    }
    if patch.as_object().is_some_and(|o| o.len() > 1) {
        let patch: crate::accounts::AccountPatch = serde_json::from_value(patch)?;
        crate::accounts::update(tx, &patch, today)?;
    }
    Ok(())
}

/// Data do extrato como `NaiveDate` (para testes e projeções).
pub fn statement_day(p: &StatementPreview) -> Option<NaiveDate> {
    p.header
        .statement_date
        .as_deref()
        .and_then(|d| parse_date(d).ok())
}
