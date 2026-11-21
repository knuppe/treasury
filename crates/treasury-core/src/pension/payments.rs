//! Histórico de pagamentos de PGBL/VGBL (app Caixa Vida e Previdência, Brasilprev, Icatu…):
//! a seguradora mostra só a data e o valor de cada contribuição. As cotas saem da cota da CVM
//! do fundo do plano (o FIE, pelo CNPJ do ativo) no dia do pagamento ou no primeiro dia útil
//! seguinte, descontada a taxa de carregamento do plano, quando houver.
//!
//! Cada pagamento lido vira um aporte do plano (`pension_payment`, V141) na conta das cotas,
//! sem mexer no caixa (o débito já está no extrato da conta corrente). Com cota, o aporte já
//! nasce com a compra de cotas; sem ela, fica com a quantidade pendente e a coleta de cotações
//! cria a compra depois ([`fill_pending`]), sem reler o extrato nem duplicar. Sem conta das
//! cotas, a própria importação cria uma. O que os prints não deixam ler (valor ilegível, linha
//! cortada, mês que falta na série) volta como pendência, com o print de onde veio.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use chrono::{Datelike, Duration, NaiveDate};
use regex::Regex;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc, parse_date};

/// Dias depois do pagamento em que a cota ainda vale (fim de semana, feriado, D+1 da
/// seguradora).
const QUOTE_WINDOW_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPayment")]
pub struct Payment {
    /// Data do pagamento `YYYY-MM-DD`.
    pub date: String,
    /// Valor pago, em centavos.
    #[ts(type = "number")]
    pub amount: i64,
    /// De onde veio (o nome do print), quando lido de um anexo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPaymentsArgs")]
pub struct PaymentsArgs {
    pub asset_id: Id,
    /// Conta das cotas; sem ela, a que já tem as cotas do plano ou uma nova, criada junto.
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    #[serde(default)]
    pub payments: Vec<Payment>,
    /// Texto de cada print (o OCR do anexo, ou o colado), na ordem da rolagem: "R$ 50,00",
    /// "Pago", "11/11/2023". Prints que se sobrepõem não repetem os pagamentos.
    #[serde(default)]
    #[ts(optional)]
    pub texts: Option<Vec<String>>,
    /// Nome de cada print (na ordem de `texts`), para dizer de onde veio cada pendência.
    #[serde(default)]
    #[ts(optional)]
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "PensionPaymentStatus")]
pub enum LineStatus {
    /// Novo, com cota: vira compra de cotas.
    New,
    /// Já gravado numa importação anterior.
    Duplicate,
    /// Novo, sem cota da CVM no dia nem nos dias seguintes: o aporte fica gravado com a
    /// quantidade pendente, até a coleta trazer a cota.
    NoQuote,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPaymentLine")]
pub struct PaymentLine {
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// Dia da cota usada.
    pub quote_date: Option<String>,
    pub quote: Option<String>,
    /// Cotas compradas (depois do carregamento).
    pub quantity: Option<String>,
    /// Taxa de carregamento descontada, em centavos.
    #[ts(type = "number")]
    pub loading_fee: i64,
    pub status: LineStatus,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "PensionPaymentIssueKind")]
pub enum IssueKind {
    /// Data legível, valor não.
    UnreadableAmount,
    /// Valor legível, data não.
    UnreadableDate,
    /// Linha cortada no alto do print (só a data ou só o valor).
    CutTop,
    /// Linha cortada no fim do print.
    CutBottom,
    /// Mês sem pagamento no meio de uma série mensal.
    MissingMonth,
    /// Dia na borda entre dois prints com menos aportes que os outros dias da série: um
    /// pagamento pode ter ficado no vão entre eles.
    GapBetweenPrints,
}

impl IssueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            IssueKind::UnreadableAmount => "unreadable_amount",
            IssueKind::UnreadableDate => "unreadable_date",
            IssueKind::CutTop => "cut_top",
            IssueKind::CutBottom => "cut_bottom",
            IssueKind::MissingMonth => "missing_month",
            IssueKind::GapBetweenPrints => "gap_between_prints",
        }
    }
}

/// O que os prints não fecham: não entra na proposta e precisa de conferência.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPaymentIssue")]
pub struct Issue {
    pub kind: IssueKind,
    /// O print de onde veio.
    pub source: Option<String>,
    /// A data lida (ou o mês, `YYYY-MM`, no mês que falta).
    pub date: Option<String>,
    #[ts(type = "number | null")]
    pub amount: Option<i64>,
    /// O que aconteceu, no idioma da instalação.
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPaymentsPreview")]
pub struct PaymentsPreview {
    pub asset_id: Id,
    pub asset_name: String,
    pub lines: Vec<PaymentLine>,
    /// Novos (ainda não gravados), com ou sem cota.
    pub new_count: u32,
    /// Dos novos, os que já têm cota (viram compra de cotas na importação).
    #[serde(default)]
    pub with_quote_count: u32,
    pub duplicate_count: u32,
    /// Novos ainda sem cota (gravados com a quantidade pendente).
    pub no_quote_count: u32,
    /// Soma dos pagamentos novos (com e sem cota), em centavos.
    #[ts(type = "number")]
    pub new_total: i64,
    /// O que fazer com os pagamentos sem cota.
    pub hint: Option<String>,
    /// Pagamentos lidos dos prints.
    #[serde(default)]
    pub read_from_prints: u32,
    /// Nos prints: os que não estavam pagos e os repetidos pela sobreposição.
    #[serde(default)]
    pub skipped_not_paid: u32,
    #[serde(default)]
    pub overlap_removed: u32,
    /// O que os prints não deixaram ler, com o print de onde veio.
    #[serde(default)]
    pub issues: Vec<Issue>,
    /// Conta das cotas que recebe os aportes (`null`: será criada, com `account_to_create`).
    pub account_id: Option<Id>,
    pub account_name: Option<String>,
    pub account_to_create: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPaymentsResult")]
pub struct PaymentsResult {
    pub preview: PaymentsPreview,
    /// Aportes com cota (compra de cotas criada).
    pub trades_created: u32,
    /// Aportes gravados com a quantidade pendente (sem cota ainda).
    #[serde(default)]
    pub pending_saved: u32,
    /// Conta das cotas criada nesta importação.
    #[serde(default)]
    pub account_created: Option<Id>,
    /// Data de adesão gravada no plano (o primeiro pagamento), quando faltava.
    pub start_date_set: Option<String>,
}

fn invalid(key: &str) -> Error {
    Error::invalid(t(key))
}

// ---------------------------------------------------------------------------
// Leitura dos prints

static MONEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:R\$\s*)?\b(\d{1,3}(?:\.\d{3})*,\d{2})\b").unwrap());
static DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{2})/(\d{2})/(\d{4})\b").unwrap());
/// Situações de um pagamento que não entrou no plano.
const NOT_PAID: &[&str] = &[
    "pendente",
    "agendado",
    "cancelado",
    "estornado",
    "em aberto",
    "nao pago",
    "vencido",
    "recusado",
    "em processamento",
];

/// Pedaço de linha na borda do print (a outra metade está no print vizinho).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Partial {
    Date(NaiveDate),
    Amount(i64),
}

/// Uma linha que o print não fecha, ainda sem o nome do print.
type RawIssue = (IssueKind, Option<NaiveDate>, Option<i64>);

#[derive(Debug, Default)]
struct PrintRead {
    payments: Vec<(NaiveDate, i64)>,
    not_paid: u32,
    issues: Vec<RawIssue>,
    /// No alto do print, antes do primeiro pagamento.
    lead: Option<Partial>,
    /// No fim do print, depois do último.
    tail: Option<Partial>,
}

enum Tok {
    Amount(i64),
    Date(NaiveDate),
}

fn tokens(text: &str) -> Vec<(usize, usize, Tok)> {
    let mut out: Vec<(usize, usize, Tok)> = vec![];
    for c in MONEY.captures_iter(text) {
        let m = c.get(0).expect("m");
        let digits: String = c[1].chars().filter(char::is_ascii_digit).collect();
        if let Ok(v) = digits.parse::<i64>()
            && v > 0
        {
            out.push((m.start(), m.end(), Tok::Amount(v)));
        }
    }
    for c in DATE.captures_iter(text) {
        let m = c.get(0).expect("m");
        if let Some(d) = (|| {
            NaiveDate::from_ymd_opt(c[3].parse().ok()?, c[2].parse().ok()?, c[1].parse().ok()?)
        })() {
            out.push((m.start(), m.end(), Tok::Date(d)));
        }
    }
    out.sort_by_key(|t| t.0);
    out
}

fn not_paid(s: &str) -> bool {
    let f = crate::import::text::fold(s);
    NOT_PAID.iter().any(|w| f.contains(w))
}

/// Lê um print: cada valor com a sua data. No layout do app da Caixa o valor vem antes
/// (valor, situação, data); quando alguma linha traz a data antes do valor, vale o contrário.
/// Data sem valor (ou valor sem data) no meio vira pendência; nas bordas, pedaço de linha.
fn read_print(text: &str) -> PrintRead {
    let date_first = text.lines().any(|l| match (DATE.find(l), MONEY.find(l)) {
        (Some(d), Some(m)) => d.start() < m.start(),
        _ => false,
    });
    let toks = tokens(text);
    let mut r = PrintRead::default();
    let mut any = false;
    if date_first {
        let mut pending: Option<NaiveDate> = None;
        for (i, (_, end, tok)) in toks.iter().enumerate() {
            match tok {
                Tok::Date(d) => {
                    if let Some(old) = pending.replace(*d) {
                        r.issues
                            .push((IssueKind::UnreadableAmount, Some(old), None));
                    }
                }
                Tok::Amount(a) => match pending.take() {
                    Some(d) => {
                        let next = toks.get(i + 1).map_or(text.len(), |t| t.0);
                        if not_paid(&text[*end..next]) {
                            r.not_paid += 1;
                        } else {
                            r.payments.push((d, *a));
                        }
                        any = true;
                    }
                    None if !any => {
                        r.lead = Some(Partial::Amount(*a));
                        any = true;
                    }
                    None => r.issues.push((IssueKind::UnreadableDate, None, Some(*a))),
                },
            }
        }
        r.tail = pending.map(Partial::Date);
    } else {
        let mut pending: Option<(i64, usize)> = None;
        for (start, _, tok) in &toks {
            match tok {
                Tok::Amount(a) => {
                    if let Some((old, _)) = pending.replace((*a, *start)) {
                        r.issues.push((IssueKind::UnreadableDate, None, Some(old)));
                    }
                }
                Tok::Date(d) => match pending.take() {
                    Some((a, from)) => {
                        if not_paid(&text[from..*start]) {
                            r.not_paid += 1;
                        } else {
                            r.payments.push((*d, a));
                        }
                        any = true;
                    }
                    None if !any => {
                        r.lead = Some(Partial::Date(*d));
                        any = true;
                    }
                    None => r.issues.push((IssueKind::UnreadableAmount, Some(*d), None)),
                },
            }
        }
        r.tail = pending.map(|(a, _)| Partial::Amount(a));
    }
    r
}

/// Pagamentos de um print só (para testes e conferência).
pub fn parse_print(text: &str) -> (Vec<Payment>, u32) {
    let r = read_print(text);
    let ps = r
        .payments
        .into_iter()
        .map(|(d, a)| Payment {
            date: fmt_date(d),
            amount: a,
            source: None,
        })
        .collect();
    (ps, r.not_paid)
}

fn issue_message(kind: IssueKind, date: Option<&str>, amount: Option<i64>, source: &str) -> String {
    let key = match kind {
        IssueKind::UnreadableAmount => "nucleo-planejamento-previdencia-pagamentos-ilegivel-valor",
        IssueKind::UnreadableDate => "nucleo-planejamento-previdencia-pagamentos-ilegivel-data",
        IssueKind::CutTop => "nucleo-planejamento-previdencia-pagamentos-cortada-alto",
        IssueKind::CutBottom => "nucleo-planejamento-previdencia-pagamentos-cortada-fim",
        IssueKind::MissingMonth => "nucleo-planejamento-previdencia-pagamentos-mes-faltando",
        IssueKind::GapBetweenPrints => "nucleo-planejamento-previdencia-pagamentos-vao",
    };
    let l = crate::i18n::current();
    let lido = match (date, amount) {
        (Some(d), _) if d.len() == 7 => crate::reports::engine::month_label(d),
        (Some(d), _) => parse_date(d).map_or(d.to_string(), |x| crate::i18n::date(l, x)),
        (None, Some(a)) => crate::i18n::money(l, a, "BRL"),
        (None, None) => "?".to_string(),
    };
    t_args(
        key,
        &args(&[("lido", lido.into()), ("print", source.to_string().into())]),
    )
}

/// O resultado da leitura de todos os prints.
#[derive(Debug, Default)]
pub struct Prints {
    pub payments: Vec<Payment>,
    pub not_paid: u32,
    pub overlap: u32,
    pub issues: Vec<Issue>,
}

/// Junta os prints na ordem da rolagem: o começo de um que repete o fim do anterior entra uma
/// vez só; a linha cortada entre dois prints é montada com as duas metades; o resto do que
/// não fecha vira pendência com o print de onde veio. No fim, os meses que faltam numa série
/// mensal.
pub fn read_prints(texts: &[String], sources: &[String]) -> Prints {
    let mut out = Prints::default();
    let mut all: Vec<(NaiveDate, i64, String)> = vec![];
    let mut raw: Vec<(IssueKind, Option<NaiveDate>, Option<i64>, String)> = vec![];
    let mut prev_tail: Option<(Partial, String)> = None;
    // Bordas entre prints vizinhos: (último dia do anterior, primeiro dia deste, os dois
    // prints), quando nada se repetiu entre eles.
    let mut edges: Vec<(Option<NaiveDate>, Option<NaiveDate>, String, String)> = vec![];
    let mut prev_src: Option<String> = None;
    for (i, text) in texts.iter().enumerate() {
        let src = sources
            .get(i)
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| format!("{}", i + 1));
        let r = read_print(text);
        out.not_paid += r.not_paid;
        raw.extend(r.issues.into_iter().map(|(k, d, a)| (k, d, a, src.clone())));
        let first = r.payments.first().copied();
        let last = all.last().map(|x| (x.0, x.1));
        // A metade de cima deste print com a de baixo do anterior.
        let mut lead = r.lead;
        match (prev_tail.take(), lead) {
            (Some((Partial::Amount(a), _)), Some(Partial::Date(d)))
            | (Some((Partial::Date(d), _)), Some(Partial::Amount(a))) => {
                all.push((d, a, src.clone()));
                lead = None;
            }
            (Some((tail, tsrc)), _) => {
                // O fim cortado do anterior aparece inteiro no começo deste: sobreposição.
                let repeated = match (tail, first) {
                    (Partial::Amount(a), Some((_, b))) => a == b,
                    (Partial::Date(d), Some((e, _))) => d == e,
                    _ => false,
                };
                if !repeated {
                    let (d, a) = partial_parts(tail);
                    raw.push((IssueKind::CutBottom, d, a, tsrc));
                }
            }
            (None, _) => {}
        }
        if let Some(l) = lead {
            // O alto cortado deste já estava inteiro no fim do anterior.
            let repeated = match (l, last) {
                (Partial::Date(d), Some((e, _))) => d == e,
                (Partial::Amount(a), Some((_, b))) => a == b,
                _ => false,
            };
            if !repeated {
                let (d, a) = partial_parts(l);
                raw.push((IssueKind::CutTop, d, a, src.clone()));
            }
        }
        let ps: Vec<(NaiveDate, i64, String)> = r
            .payments
            .into_iter()
            .map(|(d, a)| (d, a, src.clone()))
            .collect();
        let k = (1..=ps.len().min(all.len()))
            .rev()
            .find(|&k| {
                all[all.len() - k..]
                    .iter()
                    .zip(&ps[..k])
                    .all(|(x, y)| x.0 == y.0 && x.1 == y.1)
            })
            .unwrap_or(0);
        out.overlap += k as u32;
        if let Some(p) = prev_src.take()
            && k == 0
        {
            edges.push((
                all.last().map(|x| x.0),
                ps.first().map(|x| x.0),
                p,
                src.clone(),
            ));
        }
        all.extend(ps.into_iter().skip(k));
        prev_tail = r.tail.map(|t| (t, src.clone()));
        prev_src = Some(src);
    }
    raw.extend(gaps_between(&all, &edges));
    if let Some((tail, src)) = prev_tail {
        let (d, a) = partial_parts(tail);
        raw.push((IssueKind::CutBottom, d, a, src));
    }
    // O mês de uma linha ilegível já é pendência: não conta como mês que falta.
    let flagged: BTreeSet<(i32, u32)> = raw
        .iter()
        .filter_map(|x| x.1.map(|d| (d.year(), d.month())))
        .collect();
    raw.extend(missing_months(&all).into_iter().filter(|x| {
        x.1.is_none_or(|d| !flagged.contains(&(d.year(), d.month())))
    }));
    out.issues = raw
        .into_iter()
        .map(|(kind, d, a, src)| {
            let date = d.map(|d| {
                if kind == IssueKind::MissingMonth {
                    format!("{:04}-{:02}", d.year(), d.month())
                } else {
                    fmt_date(d)
                }
            });
            Issue {
                message: issue_message(kind, date.as_deref(), a, &src),
                kind,
                source: (kind != IssueKind::MissingMonth).then_some(src),
                date,
                amount: a,
            }
        })
        .collect();
    out.payments = all
        .into_iter()
        .map(|(d, a, s)| Payment {
            date: fmt_date(d),
            amount: a,
            source: Some(s),
        })
        .collect();
    out
}

/// Na série com vários aportes por dia (o mais comum: três no mesmo dia), o dia na borda
/// entre dois prints que ficou com menos aportes que o normal: o que faltou pode estar no
/// vão entre eles.
fn gaps_between(
    all: &[(NaiveDate, i64, String)],
    edges: &[(Option<NaiveDate>, Option<NaiveDate>, String, String)],
) -> Vec<(IssueKind, Option<NaiveDate>, Option<i64>, String)> {
    let mut per_day: std::collections::BTreeMap<NaiveDate, usize> = Default::default();
    for x in all {
        *per_day.entry(x.0).or_default() += 1;
    }
    let mut freq: std::collections::BTreeMap<usize, usize> = Default::default();
    for n in per_day.values() {
        *freq.entry(*n).or_default() += 1;
    }
    let Some((&usual, _)) = freq.iter().max_by_key(|(n, f)| (**f, **n)) else {
        return vec![];
    };
    if usual < 2 || per_day.len() < 3 {
        return vec![];
    }
    let mut out = vec![];
    let mut seen: BTreeSet<NaiveDate> = BTreeSet::new();
    for (last, first, a, b) in edges {
        for d in [last, first].into_iter().flatten() {
            if per_day.get(d).copied().unwrap_or(0) < usual && seen.insert(*d) {
                out.push((
                    IssueKind::GapBetweenPrints,
                    Some(*d),
                    None,
                    format!("{a} / {b}"),
                ));
            }
        }
    }
    out
}

fn partial_parts(p: Partial) -> (Option<NaiveDate>, Option<i64>) {
    match p {
        Partial::Date(d) => (Some(d), None),
        Partial::Amount(a) => (None, Some(a)),
    }
}

/// Meses sem pagamento entre o primeiro e o último, quando a série é mensal (três meses ou
/// mais, e a maior parte deles com pagamento).
fn missing_months(
    all: &[(NaiveDate, i64, String)],
) -> Vec<(IssueKind, Option<NaiveDate>, Option<i64>, String)> {
    let months: BTreeSet<(i32, u32)> = all.iter().map(|x| (x.0.year(), x.0.month())).collect();
    let (Some(first), Some(last)) = (months.first(), months.last()) else {
        return vec![];
    };
    let mut gaps = vec![];
    let (mut y, mut m) = *first;
    let mut span = 0;
    while (y, m) <= *last {
        span += 1;
        if !months.contains(&(y, m)) {
            gaps.push((y, m));
        }
        (y, m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    }
    if months.len() < 3 || gaps.len() * 2 > span {
        return vec![];
    }
    // Adesão no fim de um mês e o primeiro débito só no seguinte ao próximo: o mês do meio
    // fica vazio de propósito.
    let first_day = all.iter().map(|x| x.0).min();
    if let Some(f) = first_day
        && f.day() >= 15
    {
        let next = if f.month() == 12 {
            (f.year() + 1, 1)
        } else {
            (f.year(), f.month() + 1)
        };
        gaps.retain(|g| *g != next);
    }
    gaps.into_iter()
        .filter_map(|(y, m)| NaiveDate::from_ymd_opt(y, m, 1))
        .map(|d| (IssueKind::MissingMonth, Some(d), None, String::new()))
        .collect()
}

// ---------------------------------------------------------------------------
// Prévia e importação

/// Cota do dia do pagamento ou do primeiro dia com cota até [`QUOTE_WINDOW_DAYS`] depois.
fn quote_on(conn: &Connection, asset_id: &str, date: &str) -> Result<Option<(String, Decimal)>> {
    let d = parse_date(date)?;
    let until = fmt_date(d + Duration::days(QUOTE_WINDOW_DAYS));
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT date, close FROM price WHERE asset_id = ?1 AND date >= ?2 AND date <= ?3
              ORDER BY date LIMIT 1",
            params![asset_id, date, until],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.and_then(|(d, c)| {
        c.parse::<Decimal>()
            .ok()
            .filter(|q| *q > Decimal::ZERO)
            .map(|q| (d, q))
    }))
}

/// Chave de cada pagamento: a data, o valor e a ordem entre pagamentos iguais.
fn keys(payments: &[Payment]) -> Vec<String> {
    let mut seen: std::collections::HashMap<(String, i64), u32> = Default::default();
    payments
        .iter()
        .map(|p| {
            let n = seen.entry((p.date.clone(), p.amount)).or_default();
            *n += 1;
            format!("prev-pag:{}:{}:{}", p.date, p.amount, n)
        })
        .collect()
}

fn loading_fee_pct(conn: &Connection, asset_id: &str) -> Result<Decimal> {
    let v: Option<String> = conn
        .query_row(
            "SELECT loading_fee_pct FROM pension_plan WHERE asset_id = ?1",
            [asset_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(v.and_then(|s| s.replace(',', ".").parse::<Decimal>().ok())
        .filter(|p| *p > Decimal::ZERO && *p < Decimal::from(100))
        .unwrap_or(Decimal::ZERO))
}

fn fee_of(amount: i64, fee_pct: Decimal) -> i64 {
    (Decimal::from(amount) * fee_pct / Decimal::from(100))
        .round()
        .try_into()
        .unwrap_or(0)
}

fn quantity_of(amount: i64, fee: i64, quote: Decimal) -> String {
    (Decimal::new(amount - fee, 2) / quote)
        .round_dp(8)
        .normalize()
        .to_string()
}

/// Contas que já guardam as cotas do plano (operações ou aportes), a mais usada primeiro.
pub fn plan_accounts(conn: &Connection, asset_id: &str) -> Result<Vec<(Id, String)>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name FROM account a
           JOIN (SELECT account_id, count(*) AS n FROM trade WHERE asset_id = ?1 GROUP BY account_id
                 UNION ALL
                 SELECT account_id, count(*) FROM pension_payment WHERE asset_id = ?1
                  GROUP BY account_id) u ON u.account_id = a.id
          GROUP BY a.id ORDER BY sum(u.n) DESC, a.name",
    )?;
    let rows = stmt
        .query_map([asset_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// A conta escolhida (id e nome) ou o nome da que será criada.
type Target = (Option<(Id, String)>, Option<String>);

/// Conta das cotas: a informada, a que já tem as cotas do plano, ou o nome da que será
/// criada.
fn target_account(conn: &Connection, a: &PaymentsArgs, asset_name: &str) -> Result<Target> {
    if let Some(id) = a
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        crate::accounts::currency_and_kind(conn, id)?;
        let name: String =
            conn.query_row("SELECT name FROM account WHERE id = ?1", [id], |r| r.get(0))?;
        return Ok((Some((id.to_string(), name)), None));
    }
    if let Some(x) = plan_accounts(conn, &a.asset_id)?.into_iter().next() {
        return Ok((Some(x), None));
    }
    Ok((
        None,
        Some(t_args(
            "nucleo-planejamento-previdencia-pagamentos-conta-nova",
            &args(&[("plano", asset_name.to_string().into())]),
        )),
    ))
}

/// Os pagamentos informados e, depois deles, os lidos dos prints.
fn all_payments(a: &PaymentsArgs) -> (Vec<Payment>, Prints) {
    let prints = read_prints(
        a.texts.as_deref().unwrap_or_default(),
        a.sources.as_deref().unwrap_or_default(),
    );
    let mut out = a.payments.clone();
    out.extend(prints.payments.iter().cloned());
    (out, prints)
}

fn already_saved(conn: &Connection, asset_id: &str, key: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM trade WHERE asset_id = ?1 AND external_id = ?2)
             OR EXISTS (SELECT 1 FROM pension_payment WHERE asset_id = ?1 AND external_id = ?2)",
        params![asset_id, key],
        |r| r.get(0),
    )?)
}

pub fn preview(conn: &Connection, a: &PaymentsArgs) -> Result<PaymentsPreview> {
    let asset = crate::assets::get(conn, &a.asset_id)?;
    if asset.class != AssetClass::Pension {
        return Err(invalid(
            "nucleo-planejamento-previdencia-pagamentos-erro-classe",
        ));
    }
    let (payments, prints) = all_payments(a);
    if payments.is_empty() && prints.issues.is_empty() {
        return Err(invalid(
            "nucleo-planejamento-previdencia-pagamentos-erro-vazio",
        ));
    }
    let fee_pct = loading_fee_pct(conn, &a.asset_id)?;
    let keys = keys(&payments);
    let mut lines = vec![];
    for (p, key) in payments.iter().zip(&keys) {
        parse_date(&p.date)?;
        if p.amount <= 0 {
            return Err(invalid(
                "nucleo-planejamento-previdencia-pagamentos-erro-valor",
            ));
        }
        let dup = already_saved(conn, &a.asset_id, key)?;
        let fee = fee_of(p.amount, fee_pct);
        let quote = quote_on(conn, &a.asset_id, &p.date)?;
        let status = match (dup, &quote) {
            (true, _) => LineStatus::Duplicate,
            (false, None) => LineStatus::NoQuote,
            (false, Some(_)) => LineStatus::New,
        };
        lines.push(PaymentLine {
            date: p.date.clone(),
            amount: p.amount,
            quote_date: quote.as_ref().map(|(d, _)| d.clone()),
            quantity: quote.as_ref().map(|(_, q)| quantity_of(p.amount, fee, *q)),
            quote: quote.map(|(_, q)| q.normalize().to_string()),
            loading_fee: fee,
            status,
            source: p.source.clone(),
        });
    }
    let count = |s: LineStatus| lines.iter().filter(|l| l.status == s).count() as u32;
    let no_quote = count(LineStatus::NoQuote);
    let hint = (no_quote > 0).then(|| {
        if crate::market::sync::default_source(&asset).is_none() {
            t("nucleo-planejamento-previdencia-pagamentos-sem-cnpj")
        } else {
            let first = lines
                .iter()
                .filter(|l| l.status == LineStatus::NoQuote)
                .map(|l| l.date.as_str())
                .min()
                .unwrap_or_default();
            t_args(
                "nucleo-planejamento-previdencia-pagamentos-sem-cota",
                &args(&[("n", no_quote.into()), ("data", first.to_string().into())]),
            )
        }
    });
    let (account, to_create) = target_account(conn, a, &asset.name)?;
    Ok(PaymentsPreview {
        asset_id: asset.id,
        asset_name: asset.name,
        new_count: count(LineStatus::New) + no_quote,
        with_quote_count: count(LineStatus::New),
        duplicate_count: count(LineStatus::Duplicate),
        no_quote_count: no_quote,
        new_total: lines
            .iter()
            .filter(|l| l.status != LineStatus::Duplicate)
            .map(|l| l.amount)
            .sum(),
        lines,
        hint,
        read_from_prints: prints.payments.len() as u32,
        skipped_not_paid: prints.not_paid,
        overlap_removed: prints.overlap,
        issues: prints.issues,
        account_id: account.as_ref().map(|x| x.0.clone()),
        account_name: account.map(|x| x.1).or(to_create.clone()),
        account_to_create: to_create,
    })
}

/// Compra de cotas de um aporte (sem mexer no caixa).
fn buy(
    tx: &Transaction<'_>,
    asset_id: &str,
    account_id: &str,
    (date, amount, key): (&str, i64, &str),
    fee_pct: Decimal,
    quote: Decimal,
) -> Result<Id> {
    let fee = fee_of(amount, fee_pct);
    crate::portfolio::records::create_trade(
        tx,
        &crate::portfolio::TradeInput {
            asset_id: asset_id.to_string(),
            account_id: account_id.to_string(),
            date: date.to_string(),
            side: crate::portfolio::TradeSide::Buy,
            quantity: quantity_of(amount, fee, quote),
            price: quote.normalize().to_string(),
            fees: (fee > 0).then_some(fee),
            taxes: None,
            fx_rate: None,
            day_trade: None,
            note_number: None,
            notes: None,
            cash_movement: Some(false),
            external_id: Some(key.to_string()),
        },
    )
}

pub fn import(tx: &Transaction<'_>, a: &PaymentsArgs, today: NaiveDate) -> Result<PaymentsResult> {
    let preview = preview(tx, a)?;
    let (account_id, account_created) = match &preview.account_id {
        Some(id) => (id.clone(), None),
        None => {
            let insurer: Option<String> = tx
                .query_row(
                    "SELECT insurer FROM pension_plan WHERE asset_id = ?1",
                    [&a.asset_id],
                    |r| r.get(0),
                )
                .optional()?
                .flatten();
            let first = preview.lines.iter().map(|l| l.date.as_str()).min();
            let id = crate::accounts::create(
                tx,
                &crate::accounts::AccountInput {
                    name: preview.account_to_create.clone().unwrap_or_default(),
                    kind: crate::accounts::AccountKind::Brokerage,
                    currency: Some("BRL".into()),
                    institution_id: None,
                    institution_name: insurer.filter(|i| !i.trim().is_empty()),
                    member_ids: None,
                    opening_balance: Some(0),
                    opening_date: first.map(str::to_string),
                    include_in_net_worth: Some(true),
                    color: None,
                    notes: Some(t_args(
                        "nucleo-planejamento-previdencia-pagamentos-conta-nota",
                        &args(&[("plano", preview.asset_name.clone().into())]),
                    )),
                    card: None,
                    overdraft: None,
                    branch: None,
                    account_number: None,
                    retirement_plan: None,
                    retirement_coverage: None,
                },
                today,
            )?;
            (id.clone(), Some(id))
        }
    };
    let (payments, _) = all_payments(a);
    let keys = keys(&payments);
    let fee_pct = loading_fee_pct(tx, &a.asset_id)?;
    let now = now_utc();
    let (mut created, mut pending) = (0, 0);
    for ((line, key), p) in preview.lines.iter().zip(&keys).zip(&payments) {
        let trade_id = match (line.status, &line.quote) {
            (LineStatus::Duplicate, _) => continue,
            (LineStatus::New, Some(q)) => {
                let q: Decimal = q.parse().unwrap_or(Decimal::ZERO);
                created += 1;
                Some(buy(
                    tx,
                    &a.asset_id,
                    &account_id,
                    (&line.date, line.amount, key),
                    fee_pct,
                    q,
                )?)
            }
            _ => {
                pending += 1;
                None
            }
        };
        tx.execute(
            "INSERT INTO pension_payment (id, asset_id, account_id, date, amount, external_id,
                trade_id, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                new_id(),
                a.asset_id,
                account_id,
                line.date,
                line.amount,
                key,
                trade_id,
                p.source,
                now
            ],
        )?;
    }
    save_issues(tx, &a.asset_id, &payments, &preview.issues)?;
    // Sem data de adesão, a do primeiro pagamento: a coleta de cotações busca o histórico
    // desde ela e completa os aportes que ficaram sem cota.
    let first = payments.iter().map(|p| p.date.as_str()).min();
    let start_date_set = match first {
        Some(d)
            if tx.execute(
                "UPDATE pension_plan SET start_date = ?2, updated_at = ?3
                  WHERE asset_id = ?1 AND (start_date IS NULL OR start_date > ?2)",
                params![a.asset_id, d, now],
            )? > 0 =>
        {
            Some(d.to_string())
        }
        _ => None,
    };
    if created == 0 && pending == 0 && preview.issues.is_empty() {
        return Err(Error::invalid(
            preview
                .issues
                .first()
                .map(|i| i.message.clone())
                .unwrap_or_else(|| t("nucleo-planejamento-previdencia-pagamentos-nada")),
        ));
    }
    Ok(PaymentsResult {
        preview,
        trades_created: created,
        pending_saved: pending,
        account_created,
        start_date_set,
    })
}

/// Guarda as pendências desta importação (sem repetir as já guardadas) e tira as que os
/// pagamentos informados agora resolvem: o mesmo dia (ou o mesmo mês, no mês que faltava).
fn save_issues(
    tx: &Transaction<'_>,
    asset_id: &str,
    payments: &[Payment],
    issues: &[Issue],
) -> Result<()> {
    let days: BTreeSet<&str> = payments.iter().map(|p| p.date.as_str()).collect();
    let months: BTreeSet<&str> = payments.iter().filter_map(|p| p.date.get(..7)).collect();
    let saved: Vec<(Id, String, Option<String>, Option<i64>)> = {
        let mut stmt = tx.prepare(
            "SELECT id, kind, date, amount FROM pension_payment_issue WHERE asset_id = ?1",
        )?;
        stmt.query_map([asset_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let new_here = |kind: &str, date: &Option<String>, amount: Option<i64>| {
        issues
            .iter()
            .any(|i| i.kind.as_str() == kind && i.date == *date && i.amount == amount)
    };
    for (id, kind, date, amount) in &saved {
        let solved = match date.as_deref() {
            Some(d) if kind == IssueKind::MissingMonth.as_str() => months.contains(d),
            // Um vão no dia só se resolve com um pagamento novo nele, não com os mesmos.
            Some(d) => days.contains(d) && !new_here(kind, date, *amount),
            None => false,
        };
        if solved {
            tx.execute("DELETE FROM pension_payment_issue WHERE id = ?1", [id])?;
        }
    }
    let now = now_utc();
    for i in issues {
        let exists = saved
            .iter()
            .any(|(_, k, d, a)| k == i.kind.as_str() && *d == i.date && *a == i.amount);
        if exists {
            continue;
        }
        tx.execute(
            "INSERT INTO pension_payment_issue (id, asset_id, kind, date, amount, source, message,
                created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                new_id(),
                asset_id,
                i.kind.as_str(),
                i.date,
                i.amount,
                i.source,
                i.message,
                now
            ],
        )?;
    }
    Ok(())
}

/// Um aporte do plano (para a tela do ativo e o MCP): a data e o valor lidos, e a compra
/// de cotas quando já tem.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPaymentRecord")]
pub struct PaymentRecord {
    pub id: Id,
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub account_id: Id,
    pub account_name: String,
    /// `pending_quote`: ainda sem cota da CVM; `bought`: já virou compra de cotas.
    pub status: String,
    pub quantity: Option<String>,
    pub price: Option<String>,
    pub source: Option<String>,
}

/// Aportes gravados do plano, do mais recente ao mais antigo.
pub fn records(conn: &Connection, asset_id: &str) -> Result<Vec<PaymentRecord>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.date, p.amount, p.account_id, a.name, t.quantity, t.price, p.source
           FROM pension_payment p JOIN account a ON a.id = p.account_id
           LEFT JOIN trade t ON t.id = p.trade_id
          WHERE p.asset_id = ?1 ORDER BY p.date DESC, p.amount DESC",
    )?;
    let rows = stmt
        .query_map([asset_id], |r| {
            let qty: Option<String> = r.get(5)?;
            Ok(PaymentRecord {
                id: r.get(0)?,
                date: r.get(1)?,
                amount: r.get(2)?,
                account_id: r.get(3)?,
                account_name: r.get(4)?,
                status: if qty.is_some() {
                    "bought"
                } else {
                    "pending_quote"
                }
                .into(),
                quantity: qty,
                price: r.get(6)?,
                source: r.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Pendências guardadas do plano (o que os prints não fecharam).
pub fn saved_issues(conn: &Connection, asset_id: &str) -> Result<Vec<Issue>> {
    let mut stmt = conn.prepare(
        "SELECT kind, date, amount, source, message FROM pension_payment_issue
          WHERE asset_id = ?1 ORDER BY date, created_at",
    )?;
    let rows = stmt
        .query_map([asset_id], |r| {
            let kind: String = r.get(0)?;
            Ok((kind, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<Vec<(String, Option<String>, Option<i64>, Option<String>, String)>>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(k, date, amount, source, message)| {
            let kind: IssueKind = serde_json::from_value(serde_json::Value::from(k)).ok()?;
            Some(Issue {
                kind,
                source,
                date,
                amount,
                message,
            })
        })
        .collect())
}

/// Aportes de previdência ainda sem cota até a data, por ativo e conta: (ativo, conta,
/// soma, primeiro dia). Entram na carteira e no patrimônio pelo valor aportado.
pub fn pending_by_asset(
    conn: &Connection,
    as_of: &str,
    accounts: Option<&[Id]>,
) -> Result<Vec<(Id, Id, i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT asset_id, account_id, sum(amount), min(date) FROM pension_payment
          WHERE trade_id IS NULL AND date <= ?1 GROUP BY asset_id, account_id",
    )?;
    let rows = stmt
        .query_map([as_of], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<Vec<(Id, Id, i64, String)>>>()?;
    Ok(rows
        .into_iter()
        .filter(|r| accounts.is_none_or(|a| a.contains(&r.1)))
        .collect())
}

/// O que a importação vai propor, para a resposta da proposta e o texto da fila.
pub fn report(p: &PaymentsPreview) -> serde_json::Value {
    serde_json::json!({
        "asset_name": p.asset_name,
        "with_quote": p.with_quote_count,
        "pending_quote": p.no_quote_count,
        "duplicates": p.duplicate_count,
        "account_name": p.account_name,
        "account_to_create": p.account_to_create,
        "issues": p.issues,
        "hint": p.hint,
    })
}

/// Texto da proposta: quantos aportes entram, com e sem cota, e quantas pendências.
pub fn reason(p: &PaymentsPreview) -> String {
    t_args(
        "nucleo-planejamento-previdencia-motivo-pagamentos-n",
        &args(&[
            ("plano", p.asset_name.clone().into()),
            ("com", p.with_quote_count.into()),
            ("sem", p.no_quote_count.into()),
            ("pendencias", (p.issues.len() as u32).into()),
        ]),
    )
}

/// Completa os aportes que estavam sem cota com a compra de cotas, quando a coleta trouxe a
/// cota do dia. Roda depois de cada coleta; devolve quantos completou.
pub fn fill_pending(conn: &mut Connection) -> Result<u32> {
    type Row = (Id, Id, Id, String, i64, String);
    let rows: Vec<Row> = {
        let mut stmt = conn.prepare(
            "SELECT id, asset_id, account_id, date, amount, external_id FROM pension_payment
              WHERE trade_id IS NULL ORDER BY asset_id, date",
        )?;
        stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let mut ready = vec![];
    for r in rows {
        if let Some((_, q)) = quote_on(conn, &r.1, &r.3)? {
            ready.push((r, q));
        }
    }
    if ready.is_empty() {
        return Ok(0);
    }
    let n = ready.len() as u32;
    crate::changes::record(
        conn,
        &crate::changes::Author::system("cvm"),
        &t_args(
            "nucleo-planejamento-previdencia-motivo-cotas-completadas",
            &args(&[("n", n.into())]),
        ),
        |tx| {
            for ((id, asset, account, date, amount, key), q) in &ready {
                let fee_pct = loading_fee_pct(tx, asset)?;
                let trade = buy(tx, asset, account, (date, *amount, key), fee_pct, *q)?;
                tx.execute(
                    "UPDATE pension_payment SET trade_id = ?2, updated_at = ?3 WHERE id = ?1",
                    params![id, trade, now_utc()],
                )?;
            }
            Ok(())
        },
    )?;
    Ok(n)
}

/// Aportes do plano ainda sem cota: quantos, a soma e o mais antigo.
pub fn pending_summary(conn: &Connection, asset_id: &str) -> Result<(u32, i64, Option<String>)> {
    Ok(conn.query_row(
        "SELECT count(*), coalesce(sum(amount), 0), min(date) FROM pension_payment
          WHERE asset_id = ?1 AND trade_id IS NULL",
        [asset_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?)
}
