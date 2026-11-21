//! Holerite (contracheque) em PDF ou texto (docs/income.md): o layout `payslip.generico.v1`
//! lê competência, tipo (mensal, 13º, férias, PLR, adiantamento), CNPJ do empregador, as linhas
//! de vencimentos e descontos e os totais (bruto, descontos, líquido, FGTS). INSS e IRRF saem
//! das linhas. Nome e CPF do funcionário nunca são guardados.
//!
//! Importar grava o holerite na renda (a de mesmo CNPJ do empregador, a escolhida ou uma nova),
//! atualiza o bruto e o líquido de referência da renda e liga o holerite ao lançamento do
//! salário (mesmo valor líquido, perto do pagamento).

use std::sync::OnceLock;

use regex::Regex;

use super::payslip_specs::{self as specs, Num, Spec};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{IncomeSourceInput, IncomeSourceKind};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::import::text::{fold, fold_keep_punct};
use crate::time::now_utc;

/// Aviso de competência não achada (a importação tira o aviso quando a competência vem
/// informada).
const NO_MONTH: &str = "nucleo-renda-aviso-sem-competencia";

/// Erro sobre um arquivo enviado (`{ $arquivo }: …`).
fn file_error(key: &str, filename: &str) -> String {
    t_args(key, &args(&[("arquivo", filename.to_string().into())]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PayslipKind {
    Monthly,
    Advance,
    #[serde(rename = "thirteenth_1")]
    #[ts(rename = "thirteenth_1")]
    Thirteenth1,
    #[serde(rename = "thirteenth_2")]
    #[ts(rename = "thirteenth_2")]
    Thirteenth2,
    Vacation,
    Plr,
    Other,
}

impl PayslipKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Monthly => "monthly",
            Self::Advance => "advance",
            Self::Thirteenth1 => "thirteenth_1",
            Self::Thirteenth2 => "thirteenth_2",
            Self::Vacation => "vacation",
            Self::Plr => "plr",
            Self::Other => "other",
        }
    }

    pub(crate) fn parse(s: &str) -> Self {
        serde_json::from_value(serde_json::Value::String(s.into())).unwrap_or(Self::Other)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayslipItem {
    pub description: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// Desconto (senão, vencimento).
    pub deduction: bool,
}

/// O que o holerite diz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayslipData {
    /// `YYYY-MM`.
    pub reference_month: Option<String>,
    pub kind: PayslipKind,
    pub employer: Option<String>,
    pub employer_cnpj: Option<String>,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub social_security: i64,
    #[ts(type = "number")]
    pub income_tax: i64,
    #[ts(type = "number")]
    pub other_deductions: i64,
    #[ts(type = "number")]
    pub net: i64,
    #[ts(type = "number | null")]
    pub fgts: Option<i64>,
    #[ts(type = "number | null")]
    pub base_salary: Option<i64>,
    pub items: Vec<PayslipItem>,
    pub warnings: Vec<String>,
}

/// Como o documento escreve os números e os rótulos: holerite do Brasil, recibo de
/// vencimento de Portugal, nómina da Espanha ou paystub dos Estados Unidos (docs/countries.md,
/// docs/international.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Br,
    Pt,
    Es,
    Us,
    /// Os países da rodada 2 (Reino Unido, Canadá, França, Alemanha, Argentina, Chile), pelos
    /// rótulos de `payslip_specs`.
    Other(&'static Spec),
}

impl Style {
    /// Colunas do período e do ano, sem código de verba (paystub e payslip).
    fn ytd(self) -> bool {
        match self {
            Style::Us => true,
            Style::Other(s) => s.ytd_column,
            _ => false,
        }
    }
}

/// Expressão inteira no texto dobrado ("net pay" não casa "net paye").
fn phrase(t: &str, p: &str) -> bool {
    let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
    t.match_indices(p)
        .any(|(i, _)| edge(t[..i].chars().next_back()) && edge(t[i + p.len()..].chars().next()))
}

/// Palavra inteira no texto dobrado ("irs" não casa "primeiros").
fn word(t: &str, w: &str) -> bool {
    t.split(|c: char| !c.is_alphanumeric()).any(|x| x == w)
}

/// O estilo do documento pelos rótulos que ele usa.
pub fn style_of(text: &str) -> Style {
    let t = fold(text);
    let br_marks = ["inss", "irrf", "fgts", "cnpj", "holerite", "contracheque"]
        .iter()
        .any(|k| t.contains(k));
    if !br_marks {
        for spec in specs::ALL {
            let hits = spec.detect.iter().filter(|k| phrase(&t, k)).count();
            if hits >= 2 && spec.strong.iter().any(|k| phrase(&t, k)) {
                return Style::Other(spec);
            }
        }
    }
    let us = [
        "gross pay",
        "net pay",
        "federal income tax",
        "fed income tax",
        "federal withholding",
        "medicare",
        "pay period",
        "earnings statement",
        "pay stub",
        "paystub",
        "check date",
    ]
    .iter()
    .filter(|k| t.contains(*k))
    .count();
    if us >= 2 {
        return Style::Us;
    }
    let br = ["inss", "irrf", "fgts", "cnpj", "holerite", "contracheque"]
        .iter()
        .any(|k| t.contains(k));
    let es = [
        "nomina",
        "liquido a percibir",
        "total devengado",
        "total a deducir",
        "contingencias comunes",
        "formacion profesional",
        "base de cotizacion",
        "devengos",
    ]
    .iter()
    .filter(|k| t.contains(*k))
    .count()
        + usize::from(word(&t, "irpf"));
    if !br && es >= 2 {
        return Style::Es;
    }
    let pt = [
        "recibo de vencimento",
        "seguranca social",
        "seg. social",
        "total iliquido",
        "vencimento base",
        "subsidio de alimentacao",
        "subsidio de refeicao",
        "niss",
    ]
    .iter()
    .filter(|k| t.contains(*k))
    .count()
        + usize::from(word(&t, "irs"));
    if !br && pt >= 2 { Style::Pt } else { Style::Br }
}

fn amount_re(style: Style) -> &'static Regex {
    static BR: OnceLock<Regex> = OnceLock::new();
    static PT: OnceLock<Regex> = OnceLock::new();
    static US: OnceLock<Regex> = OnceLock::new();
    static SPACED: OnceLock<Regex> = OnceLock::new();
    static INT: OnceLock<Regex> = OnceLock::new();
    match style {
        Style::Other(s) => match s.num {
            Num::Dot => US.get_or_init(|| Regex::new(r"\d+(?:,\d{3})*\.\d{2}").expect("regex")),
            // França: também "2 500,00" com espaço comum.
            Num::Comma => SPACED.get_or_init(|| {
                Regex::new(r"\d+(?:[. \u{a0}\u{202f}]\d{3})*,\d{2}").expect("regex")
            }),
            // Chile: pesos sem centavos, com separador de milhar.
            Num::Int => INT.get_or_init(|| Regex::new(r"\d{1,3}(?:\.\d{3})+").expect("regex")),
        },
        Style::Br => BR.get_or_init(|| Regex::new(r"\d{1,3}(?:\.\d{3})*,\d{2}").expect("regex")),
        // Portugal e Espanha: 1.234,56, 1 234,56 (espaço inseparável) ou 1234,56.
        Style::Pt | Style::Es => {
            PT.get_or_init(|| Regex::new(r"\d+(?:[.\u{a0}\u{202f}]\d{3})*,\d{2}").expect("regex"))
        }
        // Estados Unidos: 1,234.56.
        Style::Us => US.get_or_init(|| Regex::new(r"\d+(?:,\d{3})*\.\d{2}").expect("regex")),
    }
}

fn money(s: &str, style: Style) -> Option<i64> {
    match style {
        Style::Other(spec) => {
            let keep = |c: char| {
                c.is_ascii_digit() || matches!((spec.num, c), (Num::Dot, '.') | (Num::Comma, ','))
            };
            let clean: String = s.chars().filter(|c| keep(*c)).collect();
            match spec.num {
                Num::Int => clean.parse().ok(),
                Num::Dot => {
                    let v: rust_decimal::Decimal = clean.parse().ok()?;
                    use rust_decimal::prelude::ToPrimitive;
                    (v * rust_decimal::Decimal::ONE_HUNDRED).round().to_i64()
                }
                Num::Comma => crate::money::parse_amount(&clean, "EUR").ok(),
            }
        }
        Style::Br => crate::money::parse_amount(s, "BRL").ok(),
        Style::Pt | Style::Es => {
            let clean: String = s
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == ',')
                .collect();
            crate::money::parse_amount(&clean, "EUR").ok()
        }
        Style::Us => {
            let clean: String = s
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let v: rust_decimal::Decimal = clean.parse().ok()?;
            use rust_decimal::prelude::ToPrimitive;
            (v * rust_decimal::Decimal::ONE_HUNDRED).round().to_i64()
        }
    }
}

/// Parece um holerite (do Brasil, de Portugal ou dos Estados Unidos).
pub fn looks_like(text: &str) -> bool {
    let t = fold(text);
    let br = [
        "holerite",
        "contracheque",
        "contra cheque",
        "recibo de pagamento",
        "demonstrativo de pagamento",
        "recibo de salario",
        "folha de pagamento",
        "demonstrativo de rendimentos",
    ]
    .iter()
    .any(|k| t.contains(k))
        && t.contains("liquido");
    let pt = [
        "recibo de vencimento",
        "recibo de remuneracoes",
        "folha de vencimento",
    ]
    .iter()
    .any(|k| t.contains(k))
        && t.contains("liquido");
    let es = [
        "nomina",
        "recibo de salarios",
        "recibo individual justificativo",
    ]
    .iter()
    .any(|k| t.contains(k))
        && t.contains("liquido");
    let us = [
        "pay stub",
        "paystub",
        "earnings statement",
        "pay statement",
        "statement of earnings",
        "payroll",
        "pay period",
    ]
    .iter()
    .any(|k| t.contains(k))
        && t.contains("net pay");
    // Os países da rodada 2: o estilo reconhecido e o líquido.
    let other = matches!(style_of(text), Style::Other(s) if s.net.iter().any(|k| phrase(&t, k)));
    br || pt || es || us || other
}

/// Primeiro valor depois de um dos rótulos (texto em minúsculas, sem acentos).
fn after(low: &str, labels: &[&str], style: Style) -> Option<i64> {
    for l in labels {
        if let Some(i) = low.find(l) {
            let w: String = low[i + l.len()..].chars().take(80).collect();
            if let Some(m) = amount_re(style).find(&w) {
                return money(m.as_str(), style);
            }
        }
    }
    None
}

/// Como [`after`], mas na última ocorrência do rótulo.
fn after_last(low: &str, labels: &[&str], style: Style) -> Option<i64> {
    for l in labels {
        if let Some(i) = low.rfind(l) {
            let w: String = low[i + l.len()..].chars().take(80).collect();
            if let Some(m) = amount_re(style).find(&w) {
                return money(m.as_str(), style);
            }
        }
    }
    None
}

/// Mês pelo nome em português, espanhol, inglês, francês ou alemão.
fn month_number(name: &str) -> Option<u32> {
    [&MONTHS, &MONTHS_ES, &MONTHS_EN, &MONTHS_FR, &MONTHS_DE]
        .iter()
        .find_map(|list| list.iter().position(|x| *x == name))
        .map(|i| i as u32 + 1)
}

const MONTHS_FR: [&str; 12] = [
    "janvier",
    "fevrier",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "aout",
    "septembre",
    "octobre",
    "novembre",
    "decembre",
];

const MONTHS_DE: [&str; 12] = [
    "januar",
    "februar",
    "marz",
    "april",
    "mai",
    "juni",
    "juli",
    "august",
    "september",
    "oktober",
    "november",
    "dezember",
];

const MONTHS_ES: [&str; 12] = [
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre",
];

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

const MONTHS_EN: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

fn reference_month(low: &str) -> Option<String> {
    static NUM: OnceLock<Regex> = OnceLock::new();
    let num =
        NUM.get_or_init(|| Regex::new(r"\b(0?[1-9]|1[0-2])\s*/\s*(20\d{2})\b").expect("regex"));
    static NAME: OnceLock<Regex> = OnceLock::new();
    let name = NAME.get_or_init(|| {
        Regex::new(r"\b(janeiro|fevereiro|marco|abril|maio|junho|julho|agosto|setembro|outubro|novembro|dezembro|enero|febrero|marzo|mayo|junio|julio|septiembre|octubre|noviembre|diciembre|january|february|march|april|may|june|july|august|september|october|november|december|janvier|fevrier|mars|avril|mai|juin|juillet|aout|septembre|octobre|novembre|decembre|januar|februar|marz|juni|juli|oktober|dezember)\s*(?:de\s*|/\s*)?(20\d{2})\b")
            .expect("regex")
    });
    let fmt = |m: u32, y: &str| format!("{y}-{m:02}");
    // Perto dos rótulos primeiro.
    for l in [
        "competencia",
        "referencia",
        "mes/ano",
        "periodo",
        "mes de referencia",
    ] {
        if let Some(i) = low.find(l) {
            let w: String = low[i..].chars().take(60).collect();
            if let Some(c) = name.captures(&w) {
                let m = month_number(&c[1])?;
                return Some(fmt(m, &c[2]));
            }
            if let Some(c) = num.captures(&w) {
                return Some(fmt(c[1].parse().ok()?, &c[2]));
            }
        }
    }
    if let Some(c) = name.captures(low) {
        let m = month_number(&c[1])?;
        return Some(fmt(m, &c[2]));
    }
    if let Some(c) = num.captures(low) {
        return Some(fmt(c[1].parse().ok()?, &c[2]));
    }
    // Data ISO (2026-10-25), comum no Canadá.
    static ISO: OnceLock<Regex> = OnceLock::new();
    let iso =
        ISO.get_or_init(|| Regex::new(r"\b(20\d{2})-(0[1-9]|1[0-2])-\d{2}\b").expect("regex"));
    iso.captures(low)
        .and_then(|c| Some(fmt(c[2].parse().ok()?, &c[1])))
}

/// Paystub: a data de pagamento ou o fim do período (`MM/DD/YYYY`), ou "September 2026".
fn reference_month_us(low: &str) -> Option<String> {
    static DATE: OnceLock<Regex> = OnceLock::new();
    let date = DATE.get_or_init(|| {
        Regex::new(r"\b(0?[1-9]|1[0-2])/(0?[1-9]|[12]\d|3[01])/(20\d{2})\b").expect("regex")
    });
    let fmt =
        |m: &str, y: &str| -> Option<String> { Some(format!("{y}-{:02}", m.parse::<u32>().ok()?)) };
    for l in [
        "pay date",
        "check date",
        "advice date",
        "period ending",
        "period end",
        "pay period",
    ] {
        if let Some(i) = low.find(l) {
            let w: String = low[i..].chars().take(80).collect();
            // "Pay Period 09/01/2026 - 09/15/2026": vale o fim.
            if let Some(c) = date.captures_iter(&w).last() {
                return fmt(&c[1], &c[3]);
            }
        }
    }
    if let Some(c) = date.captures_iter(low).last() {
        return fmt(&c[1], &c[3]);
    }
    MONTHS_EN.iter().enumerate().find_map(|(i, m)| {
        let at = low.find(m)?;
        let rest: String = low[at + m.len()..].chars().take(8).collect();
        let y: String = rest.trim().chars().take(4).collect();
        (y.len() == 4 && y.starts_with("20") && y.chars().all(|c| c.is_ascii_digit()))
            .then(|| format!("{y}-{:02}", i + 1))
    })
}

fn kind_of(low: &str) -> PayslipKind {
    // O título fica no começo do documento.
    // Só as linhas sem valor: uma verba "13 Salário Adiantado" no meio de um holerite mensal
    // não muda o tipo dele.
    static AMOUNT: OnceLock<Regex> = OnceLock::new();
    let amount = AMOUNT.get_or_init(|| Regex::new(r"\d[\d.,]*[.,]\d{2}\b").expect("regex"));
    let head: String = low
        .lines()
        .take(12)
        .filter(|l| !amount.is_match(l))
        .collect::<Vec<_>>()
        .join(" ");
    let h = fold(&head);
    let h = h.replace(['º', '°'], "o");
    if h.contains("13 salario") || h.contains("decimo terceiro") || h.contains("13o salario") {
        if h.contains("1a parcela") || h.contains("primeira parcela") || h.contains("adiantamento")
        {
            PayslipKind::Thirteenth1
        } else {
            PayslipKind::Thirteenth2
        }
    } else if h.contains("subsidio de natal")
        || h.contains("paga extra")
        || h.contains("paga extraordinaria")
        || h.contains("aguinaldo")
    {
        // Portugal: o subsídio de Natal faz o papel do 13º; Espanha: as pagas extra.
        PayslipKind::Thirteenth2
    } else if h.contains("recibo de ferias")
        || h.contains("aviso de ferias")
        || h.contains("subsidio de ferias")
    {
        PayslipKind::Vacation
    } else if h.contains("participacao nos lucros") || h.split_whitespace().any(|w| w == "plr") {
        PayslipKind::Plr
    } else if h.contains("adiantamento salarial") || h.contains("adiantamento quinzenal") {
        PayslipKind::Advance
    } else {
        PayslipKind::Monthly
    }
}

const DEDUCTION_WORDS: &[&str] = &[
    "inss",
    "irrf",
    "imposto de renda",
    "i r r f",
    "desconto",
    "desc ",
    "vale transporte",
    "vale-transporte",
    "v transporte",
    "plano",
    "assist",
    "odont",
    "medic",
    "saude",
    "contrib",
    "sindic",
    "adiantamento",
    "pensao",
    "emprestimo",
    "consignado",
    "falta",
    "atraso",
    "seguro",
    "coparticip",
    "farmacia",
    "mensalidade",
    "previdencia privada",
    "prev privada",
    "dsr s falta",
];

const DEDUCTION_WORDS_PT: &[&str] = &[
    "seguranca social",
    "seg. social",
    "seg social",
    "tsu",
    "cga",
    "adse",
    "retencao",
    "ret. irs",
    "quota sindical",
    "sindicato",
    "penhora",
    "adiantamento",
    "seguro de saude",
    "falta",
    "desconto",
];

const DEDUCTION_WORDS_ES: &[&str] = &[
    "contingencias comunes",
    "desempleo",
    "formacion profesional",
    "seguridad social",
    "irpf",
    "retencion",
    "anticipo",
    "embargo",
    "cuota sindical",
    "seguro medico",
    "deduccion",
];

const DEDUCTION_WORDS_US: &[&str] = &[
    "federal income tax",
    "fed income tax",
    "federal withholding",
    "fed withholding",
    "fed w/h",
    "state income tax",
    "state tax",
    "local tax",
    "city tax",
    "social security",
    "soc sec",
    "oasdi",
    "fica",
    "medicare",
    "401(k)",
    "401k",
    "403(b)",
    "403b",
    "roth",
    "medical",
    "dental",
    "vision",
    "hsa",
    "fsa",
    "pre-tax",
    "post-tax",
    "pre tax",
    "post tax",
    "pretax",
    "garnishment",
    "union dues",
    "insurance",
    "sdi",
    "deduction",
];

/// Linhas que são totais, bases ou rótulos (não vencimento nem desconto).
fn summary_line(f: &str, style: Style) -> bool {
    let keys: &[&str] = match style {
        Style::Br => &[
            "total",
            "liquido",
            "base calc",
            "base de calc",
            "base inss",
            "base irrf",
            "base fgts",
            "fgts",
            "salario base",
            "salario contratual",
            "faixa irrf",
            "sal contr",
        ],
        Style::Pt => &[
            "total",
            "liquido",
            "base de incidencia",
            "taxa irs",
            "taxa de retencao",
            "remuneracao base mensal",
        ],
        Style::Es => &[
            "total",
            "liquido",
            "base de cotizacion",
            "base sujeta",
            "base irpf",
            "base de irpf",
            "determinacion de las bases",
            "remuneracion mensual",
            "prorrata",
            "tipo",
        ],
        Style::Other(s) => s.summary,
        Style::Us => &[
            "total",
            "net pay",
            "gross pay",
            "taxable wages",
            "federal taxable",
            "ytd",
            "year to date",
            "rate",
            "hours",
        ],
    };
    keys.iter()
        .any(|k| f.starts_with(k) || f.contains(&format!(" {k}")))
}

/// Linha de totais das colunas ("Totais", "Total geral"), não um vencimento.
fn totals_row(f: &str) -> bool {
    let t = f.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ');
    t.starts_with("totais") || t.starts_with("total geral") || t.starts_with("totalizacao")
}

/// O valor em `line[start..end]` tem sinal negativo: "-R$ 400,00", "- 400,00", "(400,00)" ou
/// "400,00-" (alguns sistemas põem o sinal depois).
fn negative(line: &str, start: usize, end: usize) -> bool {
    let before = line[..start]
        .trim_end()
        .trim_end_matches(['$', '€'])
        .trim_end_matches(['R', 'r'])
        .trim_end();
    let after = &line[end..];
    before.ends_with(['-', '−', '–', '('])
        || after.starts_with(['-', '−'])
        || (after.starts_with(')') && before.ends_with('('))
}

/// Até 3 vencimentos (fora o primeiro, o salário) que somam `gap`, para virar descontos.
fn subset_summing(items: &[PayslipItem], gap: i64) -> Option<Vec<usize>> {
    let cand: Vec<usize> = items
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, i)| !i.deduction)
        .map(|(n, _)| n)
        .collect();
    for (a, &i) in cand.iter().enumerate() {
        if items[i].amount == gap {
            return Some(vec![i]);
        }
        for (b, &j) in cand.iter().enumerate().skip(a + 1) {
            if items[i].amount + items[j].amount == gap {
                return Some(vec![i, j]);
            }
            for &k in cand.iter().skip(b + 1) {
                if items[i].amount + items[j].amount + items[k].amount == gap {
                    return Some(vec![i, j, k]);
                }
            }
        }
    }
    None
}

pub fn parse(text: &str) -> PayslipData {
    parse_as(text, style_of(text))
}

pub fn parse_as(text: &str, style: Style) -> PayslipData {
    let low = fold_keep_punct(text);
    let deduction_words = match style {
        Style::Br => DEDUCTION_WORDS,
        Style::Pt => DEDUCTION_WORDS_PT,
        Style::Es => DEDUCTION_WORDS_ES,
        Style::Other(s) => s.deductions,
        Style::Us => DEDUCTION_WORDS_US,
    };
    let mut items = vec![];
    // Linha "Totais" com as colunas de proventos e descontos (os totais impressos).
    let mut printed: Option<(i64, i64)> = None;
    // FGTS na coluna informativa (linha com código), quando não há "FGTS do mês".
    let mut fgts_line: Option<i64> = None;
    for raw in text.lines() {
        let line = raw.trim();
        let amounts: Vec<_> = amount_re(style).find_iter(line).collect();
        let Some(last) = amounts.last() else {
            continue;
        };
        let lead = fold(line);
        if !style.ytd() && totals_row(&lead) {
            let mut pos = 0;
            let mut neg = 0;
            for m in &amounts {
                let v = money(m.as_str(), style).unwrap_or(0);
                if negative(line, m.start(), m.end()) {
                    neg += v;
                } else if pos == 0 {
                    pos = v;
                }
            }
            if pos > 0 || neg > 0 {
                printed = Some((pos, neg));
            }
            continue;
        }
        // Paystub: colunas "Current" e "YTD"; vale a do período (a penúltima).
        let value = if style.ytd() && amounts.len() >= 2 {
            amounts[amounts.len() - 2]
        } else {
            *last
        };
        let desc_end = amounts.first().map_or(0, |m| m.start());
        // Código da verba na frente (Brasil, Portugal); no paystub, "401(k)" é o nome.
        let desc: String = if style.ytd() {
            line[..desc_end].trim().to_string()
        } else {
            // Só o código seguido de espaço sai ("220 1/3 Férias" fica "1/3 Férias").
            let d = line[..desc_end].trim();
            let d = match d.find(|c: char| !c.is_ascii_digit()) {
                Some(i) if i > 0 && d[i..].starts_with([' ', '-', '\t']) => {
                    d[i..].trim_start_matches([' ', '-', '\t'])
                }
                _ => d,
            };
            d.to_string()
        };
        // Sem o símbolo da moeda e o sinal que ficam antes do primeiro valor.
        let desc = desc
            .trim_end_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '−' | '$' | '€'))
            .trim_end_matches("R")
            .trim_end_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '−' | '$'))
            .to_string();
        let f = fold(&desc);
        // Linha com código na frente é verba (até "salário base"); sem código, totais e bases
        // do rodapé ficam de fora.
        let coded = !style.ytd() && line.starts_with(|c: char| c.is_ascii_digit());
        let total = f.starts_with("total") || f.contains("liquido") || f.contains("net pay");
        // Coluna "Informativas" (FGTS do mês) e bases não são vencimento nem desconto.
        let informative = style == Style::Br && (word(&f, "fgts") || f.starts_with("base "));
        if informative && word(&f, "fgts") && fgts_line.is_none() {
            fgts_line = money(last.as_str(), style);
        }
        // "EI" (Canadá), "KV" (Alemanha): siglas de duas letras dos países da rodada 2.
        let short = match style {
            Style::Other(s) => {
                desc.len() < 2
                    || (desc.len() == 2
                        && !s
                            .contribution_word
                            .iter()
                            .chain(s.tax_word)
                            .any(|w| word(&fold(&desc), w)))
            }
            _ => desc.len() < 3,
        };
        if short || f.is_empty() || total || informative || (!coded && summary_line(&f, style)) {
            continue;
        }
        // CNPJ, CPF, NIF ou datas não são linhas de valor.
        if f.chars().all(|c| c.is_ascii_digit() || c == ' ') {
            continue;
        }
        let Some(v) = money(value.as_str(), style) else {
            continue;
        };
        if v == 0 {
            continue;
        }
        // Valor com sinal ("-R$ 400,00", "400,00-") é desconto, seja qual for o nome.
        let deduction = negative(line, value.start(), value.end())
            || deduction_words
                .iter()
                .any(|w| f.contains(w.trim()) || f.starts_with(w.trim()))
            || (style == Style::Pt && (word(&f, "irs") || word(&f, "ss")))
            || (style == Style::Es && (word(&f, "mei") || word(&f, "ss")))
            || matches!(style, Style::Other(s) if s.contribution_word.iter().chain(s.tax_word).any(|w| word(&f, w)))
            || (style == Style::Us && (word(&f, "fit") || word(&f, "sit") || word(&f, "fitw")));
        items.push(PayslipItem {
            description: desc.chars().take(120).collect(),
            amount: v,
            deduction,
        });
    }
    let earnings: i64 = items
        .iter()
        .filter(|i| !i.deduction)
        .map(|i| i.amount)
        .sum();
    let deductions: i64 = items.iter().filter(|i| i.deduction).map(|i| i.amount).sum();
    let (gross_l, ded_l, net_l): (&[&str], &[&str], &[&str]) = match style {
        Style::Br => (
            &[
                "total de vencimentos",
                "total vencimentos",
                "total de proventos",
                "total proventos",
                "total bruto",
                "total dos vencimentos",
            ],
            &[
                "total de descontos",
                "total descontos",
                "total dos descontos",
            ],
            &[
                "liquido a receber",
                "valor liquido",
                "liquido a pagar",
                "total liquido",
                "liquido do mes",
                "liquido",
            ],
        ),
        Style::Pt => (
            &[
                "total iliquido",
                "total bruto",
                "total abonos",
                "total de abonos",
                "total de remuneracoes",
                "total remuneracoes",
            ],
            &["total de descontos", "total descontos"],
            &[
                "liquido a receber",
                "total liquido",
                "valor liquido",
                "liquido a pagar",
                "liquido",
            ],
        ),
        Style::Es => (
            &["total devengado", "total devengos", "total bruto"],
            &["total a deducir", "total deducciones"],
            &[
                "liquido total a percibir",
                "liquido a percibir",
                "total a percibir",
                "liquido total",
                "liquido",
            ],
        ),
        Style::Other(s) => (s.gross, s.ded_total, s.net),
        Style::Us => (
            &[
                "gross pay",
                "total gross",
                "gross earnings",
                "total earnings",
            ],
            &["total deductions", "total taxes and deductions"],
            &["net pay", "take home", "net amount"],
        ),
    };
    let printed_gross = after(&low, gross_l, style).or(printed.map(|p| p.0).filter(|v| *v > 0));
    let printed_ded = after(&low, ded_l, style)
        // Paystub com impostos e deduções em totais separados: soma as linhas.
        .filter(|_| style != Style::Us)
        .or(printed.map(|p| p.1).filter(|v| *v > 0));
    // Linhas que não fecham com os totais impressos: um vencimento a mais que é justamente o
    // que falta nos descontos (desconto sem sinal e sem nome de desconto) vira desconto.
    let mut flipped = vec![];
    let (mut earnings, mut deductions) = (earnings, deductions);
    if let (Some(g), Some(d)) = (printed_gross, printed_ded)
        && earnings - g > 0
        && earnings - g == d - deductions
        && let Some(idx) = subset_summing(&items, earnings - g)
    {
        for i in idx {
            items[i].deduction = true;
            flipped.push(items[i].description.clone());
        }
        earnings = g;
        deductions = d;
    }
    let gross = printed_gross.unwrap_or(earnings);
    let total_ded = printed_ded.unwrap_or(deductions);
    let net = after(&low, net_l, style).unwrap_or(gross - total_ded);
    let pick = |words: &[&str], whole: &[&str]| -> i64 {
        items
            .iter()
            .filter(|i| i.deduction)
            .filter(|i| {
                let f = fold(&i.description);
                (words.iter().any(|w| f.contains(w)) || whole.iter().any(|w| word(&f, w)))
                    && !f.contains("base")
            })
            .map(|i| i.amount)
            .sum()
    };
    let (inss, irrf) = match style {
        Style::Br => (
            pick(
                &[
                    "inss",
                    "rpps",
                    "previdencia social",
                    "prev social",
                    "contribuicao previdenciaria",
                ],
                &[],
            ),
            pick(&["irrf", "imposto de renda", "i r r f", "ir fonte"], &[]),
        ),
        Style::Pt => (
            pick(
                &[
                    "seguranca social",
                    "seg. social",
                    "seg social",
                    "tsu",
                    "cga",
                    "caixa geral",
                ],
                &["ss"],
            ),
            pick(&["retencao", "ret. irs"], &["irs"]),
        ),
        Style::Es => (
            pick(
                &[
                    "contingencias comunes",
                    "desempleo",
                    "formacion profesional",
                    "seguridad social",
                ],
                &["mei", "ss"],
            ),
            pick(&["retencion", "irpf"], &["irpf"]),
        ),
        Style::Other(s) => (
            pick(s.contribution, s.contribution_word),
            pick(s.tax, s.tax_word),
        ),
        Style::Us => (
            pick(
                &["social security", "soc sec", "oasdi", "fica", "medicare"],
                &["med"],
            ),
            pick(
                &[
                    "federal income tax",
                    "fed income tax",
                    "federal withholding",
                    "fed withholding",
                    "fed w/h",
                    "state income tax",
                    "state tax",
                    "local tax",
                    "city tax",
                ],
                &["fit", "fitw", "sit"],
            ),
        ),
    };
    let other = (total_ded - inss - irrf).max(0);
    let mut warnings = vec![];
    let reference_month = match style {
        Style::Us => reference_month_us(&low),
        _ => reference_month(&low),
    };
    if reference_month.is_none() {
        warnings.push(t(NO_MONTH));
    }
    let currency = match style {
        Style::Br => "BRL",
        Style::Pt | Style::Es => "EUR",
        Style::Us => "USD",
        Style::Other(s) => s.currency,
    };
    if gross - total_ded != net {
        let l = crate::i18n::current();
        warnings.push(t_args(
            "nucleo-renda-aviso-nao-bate",
            &args(&[
                (
                    "calculado",
                    crate::i18n::money(l, gross - total_ded, currency).into(),
                ),
                ("liquido", crate::i18n::money(l, net, currency).into()),
            ]),
        ));
    }
    if !flipped.is_empty() {
        warnings.push(t_args(
            "nucleo-renda-aviso-sem-sinal",
            &args(&[("linhas", flipped.join(", ").into())]),
        ));
    } else if printed_gross.is_some_and(|g| g != earnings)
        || printed_ded.is_some_and(|d| d != deductions)
    {
        let l = crate::i18n::current();
        let m = |v: i64| crate::i18n::money(l, v, currency);
        warnings.push(t_args(
            "nucleo-renda-aviso-totais",
            &args(&[
                ("vencimentos", m(earnings).into()),
                ("bruto", m(gross).into()),
                ("descontos", m(deductions).into()),
                ("total_descontos", m(total_ded).into()),
            ]),
        ));
    }
    if gross == 0 {
        warnings.push(t("nucleo-renda-aviso-sem-vencimentos"));
    }
    // CNPJ do empregador só no Brasil; o NIF (Portugal) e o EIN (EUA) ficam de fora, para não
    // misturar com o número do trabalhador.
    let (employer, employer_cnpj) = if style == Style::Br {
        employer_of(text)
    } else {
        (None, None)
    };
    PayslipData {
        reference_month,
        kind: kind_of(&low),
        employer: employer.map(|e| e.chars().take(120).collect()),
        employer_cnpj,
        gross,
        social_security: inss,
        income_tax: irrf,
        other_deductions: other,
        net,
        fgts: (style == Style::Br)
            .then(|| {
                after(
                    &low,
                    &[
                        "fgts do mes",
                        "fgts mes",
                        "deposito fgts",
                        "valor fgts",
                        "fgts:",
                    ],
                    style,
                )
                .or(fgts_line)
            })
            .flatten(),
        // O rodapé repete o salário base depois da linha do item (que tem a referência antes).
        base_salary: match style {
            Style::Br => after_last(
                &low,
                &["salario base", "salario contratual", "sal. contratual"],
                style,
            ),
            Style::Pt => after(
                &low,
                &[
                    "vencimento base",
                    "remuneracao base",
                    "retribuicao base",
                    "ordenado base",
                ],
                style,
            ),
            Style::Es => after(&low, &["salario base", "sueldo base"], style),
            Style::Other(s) => after(&low, s.base_salary, style),
            Style::Us => None,
        },
        items,
        warnings,
    }
}

/// Empregador e CNPJ do holerite brasileiro (a linha do CNPJ).
fn employer_of(text: &str) -> (Option<String>, Option<String>) {
    static CNPJ: OnceLock<Regex> = OnceLock::new();
    let cnpj_re = CNPJ.get_or_init(|| {
        Regex::new(r"\b(\d{2})\.?(\d{3})\.?(\d{3})/(\d{4})-?(\d{2})\b").expect("regex")
    });
    match text
        .lines()
        .find_map(|l| cnpj_re.captures(l).map(|c| (l, c)))
    {
        Some((line, c)) => {
            let digits = format!("{}{}{}{}{}", &c[1], &c[2], &c[3], &c[4], &c[5]);
            let before = line[..c.get(0).map_or(0, |m| m.start())]
                .trim()
                .trim_end_matches(|ch: char| !ch.is_alphanumeric())
                .trim_end_matches("CNPJ")
                .trim_end_matches("cnpj")
                .trim()
                .trim_end_matches(|ch: char| !ch.is_alphanumeric())
                .trim()
                .to_string();
            let before = strip_label(&before);
            let name = if !before.is_empty() && before.chars().any(char::is_alphabetic) {
                Some(before)
            } else {
                // CNPJ numa linha só: o nome vem do rótulo ("Empresa: …") ou da linha de cima.
                labelled_employer(text).or_else(|| {
                    let lines: Vec<&str> = text.lines().collect();
                    let at = lines.iter().position(|l| std::ptr::eq(*l, line))?;
                    lines[..at]
                        .iter()
                        .rev()
                        .map(|l| strip_label(l.trim()))
                        .find(|l| !l.is_empty())
                        .filter(|l| {
                            l.chars().filter(|c| c.is_alphabetic()).count() >= 3
                                && !looks_like(l)
                                && !l.chars().any(|c| c.is_ascii_digit())
                        })
                })
            };
            (name, Some(digits))
        }
        None => (labelled_employer(text), None),
    }
}

/// Tira o rótulo da frente do nome do empregador ("Empresa: X", "Razão Social - X").
fn strip_label(s: &str) -> String {
    let f = fold_keep_punct(s);
    for l in [
        "empresa",
        "empregador",
        "razao social",
        "nome da empresa",
        "nome empresarial",
        "estabelecimento",
    ] {
        // Só com separador depois do rótulo ("Empresa: X"); "EMPRESA EXEMPLO LTDA" é o nome.
        let sep = f[l.len().min(f.len())..]
            .trim_start()
            .starts_with([':', '-', '.']);
        if f.starts_with(l) && sep {
            // O rótulo tem o mesmo número de caracteres no original e no dobrado (só acentos).
            let rest: String = s.chars().skip(l.chars().count()).collect();
            return rest
                .trim_start_matches(|c: char| c == ':' || c == '-' || c == '.' || c.is_whitespace())
                .trim()
                .to_string();
        }
    }
    s.trim().to_string()
}

/// O empregador por um rótulo em qualquer linha ("Empresa: X").
fn labelled_employer(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let f = fold_keep_punct(l.trim());
        [
            "empresa:",
            "empregador:",
            "razao social:",
            "nome da empresa:",
        ]
        .iter()
        .any(|k| f.starts_with(k))
        .then(|| strip_label(l.trim()))
        .map(|n| {
            // Sem o CNPJ e o que vier depois, se estiver na mesma linha.
            n.split(['\t'])
                .next()
                .unwrap_or("")
                .split(" CNPJ")
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .filter(|n| n.chars().any(char::is_alphabetic))
    })
}

/// O layout genérico reconhece: parece holerite e tem bruto e líquido.
pub fn matches_generic(text: &str) -> bool {
    looks_like(text) && {
        let p = parse(text);
        p.gross > 0 && p.net > 0
    }
}

// ---------------------------------------------------------------------------
// Prévia e importação

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayslipFile {
    pub filename: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayslipArgs {
    /// O PDF do holerite (ou `text`).
    #[serde(default)]
    #[ts(optional)]
    pub files: Option<Vec<PayslipFile>>,
    #[serde(default)]
    #[ts(optional)]
    pub text: Option<String>,
    /// Senha do PDF (costuma ser parte do CPF): zerada depois do uso.
    #[serde(default)]
    #[ts(optional, type = "string")]
    pub password: Option<crate::vault::Secret>,
    /// Renda que recebe o holerite (padrão: a de mesmo CNPJ do empregador).
    #[serde(default)]
    #[ts(optional)]
    pub source_id: Option<Id>,
    /// Sem renda: o membro da renda nova (CLT, com o empregador do holerite).
    #[serde(default)]
    #[ts(optional)]
    pub member_id: Option<Id>,
    /// Competência quando o holerite não traz (`YYYY-MM`).
    #[serde(default)]
    #[ts(optional)]
    pub reference_month: Option<String>,
    /// Substitui o holerite da mesma renda, competência e tipo.
    #[serde(default)]
    #[ts(optional)]
    pub replace: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayslipTxnMatch {
    pub txn_id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub account_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayslipPreview {
    pub layout: crate::layouts::LayoutInfo,
    pub data: PayslipData,
    pub source_id: Option<Id>,
    pub source_name: Option<String>,
    /// Sem renda: será criada uma (CLT) para o membro informado.
    pub creates_source: bool,
    /// Já existe holerite da renda nesta competência e tipo.
    pub duplicate: bool,
    pub txn: Option<PayslipTxnMatch>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayslipResult {
    pub payslip_id: Id,
    pub source_id: Id,
    pub created_source: bool,
    pub replaced: bool,
    pub txn_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Payslip {
    pub id: Id,
    pub source_id: Id,
    pub source_name: String,
    pub member_name: String,
    pub reference_month: String,
    pub kind: PayslipKind,
    pub pay_date: Option<String>,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub social_security: i64,
    #[ts(type = "number")]
    pub income_tax: i64,
    #[ts(type = "number")]
    pub other_deductions: i64,
    #[ts(type = "number")]
    pub net: i64,
    #[ts(type = "number | null")]
    pub fgts: Option<i64>,
    pub items: Vec<PayslipItem>,
    pub txn_id: Option<Id>,
}

/// Texto do holerite (PDF ou texto) e o hash do arquivo.
fn read(a: &PayslipArgs) -> Result<(String, Option<String>)> {
    if let Some(f) = a.files.as_ref().and_then(|f| f.first()) {
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(f.content_base64.trim())
            .map_err(|_| Error::invalid(file_error("nucleo-renda-erro-base64", &f.filename)))?;
        let sha = crate::import::engine_sha256(&bytes);
        // Anexo de texto (o PDF já convertido, um .txt): vale o texto.
        if !bytes.starts_with(b"%PDF")
            && let Ok(t) = std::str::from_utf8(&bytes)
            && !t.trim().is_empty()
        {
            return Ok((t.to_string(), Some(sha)));
        }
        if bytes.starts_with(&[0x89, b'P', b'N', b'G']) || bytes.starts_with(&[0xFF, 0xD8]) {
            return Err(Error::invalid(file_error(
                "nucleo-renda-erro-imagem",
                &f.filename,
            )));
        }
        let text =
            crate::pdf::text(&bytes, a.password.as_deref().map(String::as_str)).map_err(|e| {
                let m = e.to_lowercase();
                if m.contains("password") || m.contains("decrypt") || m.contains("encrypt") {
                    Error::invalid(t("nucleo-renda-erro-pdf-senha"))
                } else {
                    Error::invalid(t_args(
                        "nucleo-renda-erro-pdf-ilegivel",
                        &args(&[("erro", e.to_string().into())]),
                    ))
                }
            })?;
        return Ok((text, Some(sha)));
    }
    match a.text.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(t) => Ok((t.to_string(), None)),
        None => Err(Error::invalid(t("nucleo-renda-erro-sem-arquivo"))),
    }
}

fn source_for(conn: &Connection, a: &PayslipArgs, d: &PayslipData) -> Result<Option<(Id, String)>> {
    if let Some(id) = &a.source_id {
        let s = super::get_source(conn, id)?;
        return Ok(Some((s.id, s.name)));
    }
    if let Some(c) = &d.employer_cnpj {
        return Ok(conn
            .query_row(
                "SELECT id, name FROM income_source WHERE employer_cnpj = ?1 AND archived = 0
                   AND (?2 IS NULL OR member_id = ?2) ORDER BY created_at LIMIT 1",
                params![c, a.member_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?);
    }
    Ok(None)
}

fn txn_match(
    conn: &Connection,
    account: Option<&str>,
    month: &str,
    net: i64,
) -> Result<Option<PayslipTxnMatch>> {
    let first = format!("{month}-01");
    Ok(conn
        .query_row(
            "SELECT t.id, t.date, t.amount, a.name FROM txn t JOIN account a ON a.id = t.account_id
              WHERE t.kind IN ('income', 'transfer') AND t.amount = ?1 AND t.status <> 'planned'
                AND t.date BETWEEN date(?2, '-5 days') AND date(?2, '+1 month', '+10 days')
                AND (?3 IS NULL OR t.account_id = ?3)
                AND NOT EXISTS (SELECT 1 FROM payslip p WHERE p.txn_id = t.id)
              ORDER BY abs(julianday(t.date) - julianday(?2, '+1 month')) LIMIT 1",
            params![net, first, account],
            |r| {
                Ok(PayslipTxnMatch {
                    txn_id: r.get(0)?,
                    date: r.get(1)?,
                    amount: r.get(2)?,
                    account_name: r.get(3)?,
                })
            },
        )
        .optional()?)
}

/// PLR no Brasil: o IRRF do holerite contra a tabela exclusiva da PLR, recalculada com as PLR
/// já recebidas no ano da mesma fonte (Lei 10.101/2000 art. 3º §§ 5º e 7º). Diferença acima
/// de R$ 1,00 vira aviso (pode ser pensão alimentícia descontada, que a tabela abate).
pub(crate) fn plr_check(
    conn: &Connection,
    source: Option<&str>,
    d: &PayslipData,
) -> Result<Option<String>> {
    let (PayslipKind::Plr, Some(month)) = (d.kind, d.reference_month.as_deref()) else {
        return Ok(None);
    };
    if crate::country::family_country(conn) != "BR" || d.gross <= 0 {
        return Ok(None);
    }
    let (before, withheld): (i64, i64) = match source {
        Some(id) => conn.query_row(
            "SELECT coalesce(sum(gross), 0), coalesce(sum(income_tax), 0) FROM payslip
              WHERE source_id = ?1 AND kind = 'plr' AND substr(reference_month, 1, 4) = ?2
                AND reference_month < ?3",
            params![id, &month[..4], month],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?,
        None => (0, 0),
    };
    let expected = crate::tax::exclusive::plr(&crate::tax::exclusive::PlrArgs {
        month: month.to_string(),
        gross: d.gross,
        paid_before: before,
        withheld_before: withheld,
        alimony: 0,
    })?
    .tax;
    if (d.income_tax - expected).abs() <= 100 {
        return Ok(None);
    }
    let money = |c: i64| crate::i18n::money(crate::i18n::current(), c, "BRL");
    Ok(Some(t_args(
        "nucleo-renda-aviso-plr-irrf",
        &args(&[
            ("retido", money(d.income_tax).into()),
            ("esperado", money(expected).into()),
        ]),
    )))
}

pub fn preview(conn: &Connection, a: &PayslipArgs) -> Result<PayslipPreview> {
    let (text, _) = read(a)?;
    // Parece holerite mas faltam os totais: diz o que faltou (o caso de um texto resumido ou
    // redigitado, em vez do texto do documento).
    if looks_like(&text) {
        let p = parse(&text);
        let missing = match (p.gross <= 0, p.net <= 0) {
            (true, true) => Some(t_args(
                "nucleo-renda-faltam-dois",
                &args(&[
                    ("um", t("nucleo-renda-falta-bruto").into()),
                    ("outro", t("nucleo-renda-falta-liquido").into()),
                ]),
            )),
            (true, false) => Some(t("nucleo-renda-falta-bruto")),
            (false, true) => Some(t("nucleo-renda-falta-liquido")),
            (false, false) => None,
        };
        if let Some(missing) = missing {
            return Err(Error::invalid(t_args(
                "nucleo-renda-erro-incompleto",
                &args(&[("faltou", missing.into())]),
            )));
        }
    }
    let layout = crate::layouts::require(
        &crate::layouts::Doc {
            format: crate::layouts::LayoutFormat::Pdf,
            text: &text,
            headers: &[],
        },
        crate::layouts::LayoutSubject::Payslip,
    )?;
    let mut data = parse(&text);
    if let Some(m) = &a.reference_month {
        crate::time::parse_month(m)?;
        data.reference_month = Some(m.clone());
        let no_month = t(NO_MONTH);
        data.warnings.retain(|w| *w != no_month);
    }
    let source = source_for(conn, a, &data)?;
    let mut warnings = data.warnings.clone();
    if source.is_none() && a.member_id.is_none() {
        warnings.push(t("nucleo-renda-aviso-escolha-renda"));
    }
    if let Some(w) = plr_check(conn, source.as_ref().map(|s| s.0.as_str()), &data)? {
        warnings.push(w);
    }
    let duplicate = match (&source, &data.reference_month) {
        (Some((id, _)), Some(m)) => conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM payslip WHERE source_id = ?1 AND reference_month = ?2 AND kind = ?3)",
            params![id, m, data.kind.as_str()],
            |r| r.get(0),
        )?,
        _ => false,
    };
    let account = match &source {
        Some((id, _)) => super::get_source(conn, id)?.account_id,
        None => None,
    };
    let txn = match &data.reference_month {
        Some(m) if data.net > 0 => txn_match(conn, account.as_deref(), m, data.net)?,
        _ => None,
    };
    Ok(PayslipPreview {
        layout,
        creates_source: source.is_none() && a.member_id.is_some(),
        source_id: source.as_ref().map(|s| s.0.clone()),
        source_name: source.map(|s| s.1),
        duplicate,
        txn,
        warnings,
        data,
    })
}

pub fn import(tx: &Transaction<'_>, a: &PayslipArgs) -> Result<PayslipResult> {
    let p = preview(tx, a)?;
    let (text_sha, data) = (read(a)?.1, p.data);
    let month = data
        .reference_month
        .clone()
        .ok_or_else(|| Error::invalid(t("nucleo-renda-erro-competencia")))?;
    if data.gross <= 0 || data.net <= 0 {
        return Err(Error::invalid(t("nucleo-renda-erro-ilegivel")));
    }
    let (source_id, created) = match p.source_id {
        Some(id) => (id, false),
        None => {
            let member = a
                .member_id
                .clone()
                .ok_or_else(|| Error::invalid(t("nucleo-renda-erro-escolha-renda")))?;
            // A renda nova recebe o nome no idioma da instalação (como as categorias padrão).
            let name = match &data.employer {
                Some(e) => t_args(
                    "nucleo-renda-nome-salario-de",
                    &args(&[("empregador", e.clone().into())]),
                ),
                None => t("nucleo-renda-nome-salario"),
            };
            let id = super::create_source(
                tx,
                &IncomeSourceInput {
                    member_id: member,
                    name,
                    kind: IncomeSourceKind::Clt,
                    employer: data.employer.clone(),
                    employer_cnpj: data.employer_cnpj.clone(),
                    gross_monthly: None,
                    net_monthly: None,
                    variable: None,
                    pay_day: None,
                    has_thirteenth: Some(true),
                    vacation_month: None,
                    dependents: None,
                    account_id: None,
                    category_id: None,
                    start_date: None,
                    end_date: None,
                    notes: None,
                },
            )?;
            (id, true)
        }
    };
    let mut replaced = false;
    if p.duplicate {
        if !a.replace.unwrap_or(false) {
            return Err(Error::conflict(t_args(
                "nucleo-renda-erro-duplicado",
                &args(&[("mes", month.clone().into())]),
            )));
        }
        tx.execute(
            "DELETE FROM payslip WHERE source_id = ?1 AND reference_month = ?2 AND kind = ?3",
            params![source_id, month, data.kind.as_str()],
        )?;
        replaced = true;
    }
    let id = new_id();
    let now = now_utc();
    let txn_id = p.txn.as_ref().map(|t| t.txn_id.clone());
    tx.execute(
        "INSERT INTO payslip (id, source_id, reference_month, kind, pay_date, gross, social_security, income_tax,
                              other_deductions, net, fgts, items, layout, file_sha256, txn_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16)",
        params![
            id,
            source_id,
            month,
            data.kind.as_str(),
            p.txn.as_ref().map(|t| t.date.clone()),
            data.gross,
            data.social_security,
            data.income_tax,
            data.other_deductions,
            data.net,
            data.fgts,
            serde_json::to_string(&data.items)?,
            p.layout.id,
            text_sha,
            txn_id,
            now
        ],
    )?;
    // O holerite mensal mais recente vira o bruto e o líquido de referência da renda.
    if data.kind == PayslipKind::Monthly {
        let latest: String = tx.query_row(
            "SELECT MAX(reference_month) FROM payslip WHERE source_id = ?1 AND kind = 'monthly'",
            [&source_id],
            |r| r.get(0),
        )?;
        if latest == month {
            tx.execute(
                "UPDATE income_source SET gross_monthly = ?2, net_monthly = ?3,
                        employer = COALESCE(employer, ?4), employer_cnpj = COALESCE(employer_cnpj, ?5),
                        updated_at = ?6
                  WHERE id = ?1",
                params![source_id, data.gross, data.net, data.employer, data.employer_cnpj, now],
            )?;
        }
    }
    Ok(PayslipResult {
        payslip_id: id,
        source_id,
        created_source: created,
        replaced,
        txn_id,
    })
}

pub fn list(
    conn: &Connection,
    source_id: Option<&str>,
    member_id: Option<&str>,
) -> Result<Vec<Payslip>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.source_id, s.name, m.name, p.reference_month, p.kind, p.pay_date, p.gross, p.social_security,
                p.income_tax, p.other_deductions, p.net, p.fgts, p.items, p.txn_id
           FROM payslip p JOIN income_source s ON s.id = p.source_id JOIN member m ON m.id = s.member_id
          WHERE (?1 IS NULL OR p.source_id = ?1) AND (?2 IS NULL OR s.member_id = ?2)
          ORDER BY p.reference_month DESC, p.kind",
    )?;
    let rows = stmt.query_map(params![source_id, member_id], |r| {
        let items: String = r.get(13)?;
        Ok(Payslip {
            id: r.get(0)?,
            source_id: r.get(1)?,
            source_name: r.get(2)?,
            member_name: r.get(3)?,
            reference_month: r.get(4)?,
            kind: PayslipKind::parse(&r.get::<_, String>(5)?),
            pay_date: r.get(6)?,
            gross: r.get(7)?,
            social_security: r.get(8)?,
            income_tax: r.get(9)?,
            other_deductions: r.get(10)?,
            net: r.get(11)?,
            fgts: r.get(12)?,
            items: serde_json::from_str(&items).unwrap_or_default(),
            txn_id: r.get(14)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM payslip WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(t_args(
            "nucleo-renda-erro-holerite",
            &args(&[("id", id.to_string().into())]),
        )));
    }
    Ok(())
}
