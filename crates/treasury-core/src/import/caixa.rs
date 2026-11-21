//! "Extrato por período" da Caixa (PDF do Internet Banking; docs/brazil/caixa-statement-by-period.md).
//! O PDF não tem texto: a página é uma imagem, lida pelo OCR local (`crate::ocr::read_scanned`);
//! a foto ou o print do mesmo extrato também serve. O leitor trabalha sobre as linhas do OCR:
//!
//! - cabeçalho: período dos lançamentos e `SALDO ANTERIOR` (abertura);
//! - tabela, do mais recente para o mais antigo: `DD/MM/AAAA - HH:MM:SS  doc  histórico
//!   [favorecido] [CPF/CNPJ mascarado]  valor C|D  saldo C|D`; `SALDO DIA` é o fechamento do
//!   dia, não lançamento; o CPF/CNPJ é descartado;
//! - conferência: abertura + lançamentos de cada dia = `SALDO DIA` (ou, com saldo corrido, linha
//!   a linha); o que não fechar vira aviso na prévia.

use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;
use sha2::{Digest, Sha256};

use super::text::fold;

/// Marcadores do leiaute (texto dobrado): ao menos três, com o título ou o período.
pub fn looks_like(text: &str) -> bool {
    let t = fold(text);
    let title = t.contains("extrato por periodo") || t.contains("periodo dos lancamentos");
    if !title {
        return false;
    }
    let header = (t.contains("nr doc") || t.contains("nr. doc"))
        && t.contains("historico")
        && t.contains("favorecido")
        && t.contains("saldo");
    let marks = [
        t.contains("extrato por periodo"),
        t.contains("periodo dos lancamentos"),
        t.contains("saldo anterior"),
        header,
        t.contains("alo caixa") || t.contains("sac caixa") || t.contains("0800 725 7474"),
        t.contains("saldo dia"),
    ];
    marks.iter().filter(|m| **m).count() >= 3
}

/// Um lançamento do extrato.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Linha no texto lido (1 = a primeira).
    pub line: u32,
    pub date: NaiveDate,
    /// `HH:MM:SS` (só para a ordem e o id externo).
    pub time: String,
    pub doc: String,
    pub history: String,
    pub payee: Option<String>,
    /// Centavos, com sinal (D = negativo).
    pub amount: i64,
    /// Saldo impresso na linha, quando houver.
    pub balance: Option<i64>,
}

impl Row {
    /// `sha256("caixa-periodo|" + data + hora + doc + valor + histórico)`: o mesmo lançamento
    /// lido de novo (ou num período sobreposto) não duplica.
    pub fn external_id(&self) -> String {
        let key = format!(
            "caixa-periodo|{}{}{}{}{}",
            self.date, self.time, self.doc, self.amount, self.history
        );
        format!(
            "caixa:{}",
            &hex::encode(Sha256::digest(key.as_bytes()))[..32]
        )
    }
}

/// Um dia que não fechou: o fechamento impresso e o calculado.
#[derive(Debug, Clone, PartialEq)]
pub struct DayProblem {
    pub date: NaiveDate,
    pub printed: i64,
    pub computed: i64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Statement {
    pub period: Option<(NaiveDate, NaiveDate)>,
    /// `SALDO ANTERIOR`.
    pub opening: Option<i64>,
    /// O `SALDO DIA` mais recente.
    pub closing: Option<i64>,
    /// Agência / operação.conta, como impresso (só para sugerir a conta).
    pub account: Option<String>,
    /// Lançamentos em ordem cronológica.
    pub rows: Vec<Row>,
    /// Fechamento de cada dia (`SALDO DIA`), em ordem cronológica.
    pub closings: Vec<(NaiveDate, i64)>,
    /// Dias em que a conta não fechou.
    pub problems: Vec<DayProblem>,
    /// Abertura + lançamentos = fechamento.
    pub total_ok: Option<bool>,
}

static ROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(\d{2})/(\d{2})/(\d{4})\s*-\s*(\d{2}):(\d{2}):(\d{2})\s+([0-9OoDQIl]{4,8})\s+(.*)$",
    )
    .unwrap()
});
/// Valor com C/D (colado ou não), com o ponto do milhar opcional.
static MONEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d{1,3}(?:\.\d{3})+|\d+),(\d{2})\s*([CD])\b").unwrap());
static DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d{2})/(\d{2})/(\d{4})").unwrap());
static ACCOUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{4}\s*/\s*\d{3,4}\.\d{4,14}-\d\b").unwrap());

fn money(caps: &regex::Captures<'_>) -> Option<i64> {
    let int: i64 = caps[1].replace('.', "").parse().ok()?;
    let cents: i64 = caps[2].parse().ok()?;
    let v = int * 100 + cents;
    Some(if &caps[3] == "D" { -v } else { v })
}

fn date(d: &str, m: &str, y: &str) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?)
}

/// Histórico e favorecido: o que fica entre o documento e o valor, sem o CPF/CNPJ mascarado.
fn split_middle(mid: &str) -> (String, Option<String>) {
    let parts: Vec<String> = mid
        .split("  ")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        // CPF/CNPJ mascarado (asteriscos e dígitos): descartado.
        .filter(|p| !p.contains('*'))
        .map(|p| {
            p.split_whitespace()
                .filter(|w| !w.contains('*'))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|p| !p.is_empty())
        .collect();
    match parts.as_slice() {
        [] => (String::new(), None),
        [one] => {
            // Sem o espaço grande: o histórico vai até a primeira palavra em caixa mista.
            let words: Vec<&str> = one.split_whitespace().collect();
            let cut = words
                .iter()
                .position(|w| {
                    w.chars().any(|c| c.is_lowercase()) && w.chars().any(|c| c.is_uppercase())
                })
                .unwrap_or(words.len());
            let history = words[..cut].join(" ");
            let payee = (cut < words.len()).then(|| words[cut..].join(" "));
            (history, payee)
        }
        [h, rest @ ..] => (h.clone(), Some(rest.join(" "))),
    }
}

fn fix_doc(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'O' | 'o' | 'D' | 'Q' => '0',
            'I' | 'l' => '1',
            c => c,
        })
        .collect()
}

/// Lê o extrato a partir das linhas do OCR (ou do texto do PDF, se um dia vier com texto).
pub fn parse(text: &str) -> Statement {
    let mut st = Statement::default();
    // (ordem impressa, linha)
    let mut printed: Vec<(usize, Row)> = vec![];
    let mut closings: Vec<(NaiveDate, i64)> = vec![];
    let mut seen: std::collections::HashSet<String> = Default::default();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let f = fold(line);
        if let Some(c) = ROW.captures(line) {
            let Some(d) = date(&c[1], &c[2], &c[3]) else {
                continue;
            };
            let time = format!("{}:{}:{}", &c[4], &c[5], &c[6]);
            let doc = fix_doc(&c[7]);
            let rest = &c[8];
            let ms: Vec<regex::Captures<'_>> = MONEY.captures_iter(rest).collect();
            let Some(first) = ms.first() else {
                continue;
            };
            let (amount_c, balance_c) = if ms.len() >= 2 {
                (&ms[ms.len() - 2], Some(&ms[ms.len() - 1]))
            } else {
                (first, None)
            };
            let Some(amount) = money(amount_c) else {
                continue;
            };
            let balance = balance_c.and_then(money);
            let mid = &rest[..amount_c.get(0).map_or(rest.len(), |m| m.start())];
            let (history, payee) = split_middle(mid);
            if fold(&history).starts_with("saldo dia") {
                if let Some(b) = balance
                    && !closings.iter().any(|(x, _)| *x == d)
                {
                    closings.push((d, b));
                }
                continue;
            }
            if amount == 0 {
                continue;
            }
            let row = Row {
                line: i as u32 + 1,
                date: d,
                time,
                doc,
                history,
                payee,
                amount,
                balance,
            };
            // Página repetida (o mesmo lançamento duas vezes no texto): fica um.
            if seen.insert(row.external_id()) {
                printed.push((printed.len(), row));
            }
            continue;
        }
        if f.contains("saldo anterior") && st.opening.is_none() {
            st.opening = MONEY.captures(line).and_then(|c| money(&c));
        } else if f.contains("periodo dos lancamentos") && st.period.is_none() {
            let ds: Vec<NaiveDate> = DATE
                .captures_iter(line)
                .filter_map(|c| date(&c[1], &c[2], &c[3]))
                .collect();
            if ds.len() >= 2 {
                st.period = Some((ds[0], ds[1]));
            }
        } else if f.starts_with("conta") && st.account.is_none() {
            st.account = ACCOUNT.find(line).map(|m| m.as_str().to_string());
        }
    }
    // Cronológica: a impressão vai do mais recente ao mais antigo, também dentro do dia.
    printed.sort_by(|(ia, a), (ib, b)| (a.date, &a.time).cmp(&(b.date, &b.time)).then(ib.cmp(ia)));
    st.rows = printed.into_iter().map(|(_, r)| r).collect();
    closings.sort_by_key(|(d, _)| *d);
    // Sem `SALDO DIA`: o saldo das linhas de um dia, se forem todos iguais (fechamento).
    for r in &st.rows {
        if closings.iter().any(|(d, _)| *d == r.date) {
            continue;
        }
        let same: Vec<i64> = st
            .rows
            .iter()
            .filter(|x| x.date == r.date)
            .filter_map(|x| x.balance)
            .collect();
        if !same.is_empty() && same.iter().all(|b| *b == same[0]) {
            closings.push((r.date, same[0]));
            closings.sort_by_key(|(d, _)| *d);
        }
    }
    st.closing = closings.last().map(|(_, b)| *b);
    check(&mut st, &closings);
    st.closings = closings;
    st
}

/// Confere dia a dia (ou linha a linha, com saldo corrido) e o total.
fn check(st: &mut Statement, closings: &[(NaiveDate, i64)]) {
    let mut running = st.opening;
    let mut days: Vec<NaiveDate> = st.rows.iter().map(|r| r.date).collect();
    days.extend(closings.iter().map(|(d, _)| *d));
    days.sort();
    days.dedup();
    for d in days {
        let rows: Vec<&Row> = st.rows.iter().filter(|r| r.date == d).collect();
        let closing = closings.iter().find(|(x, _)| *x == d).map(|(_, b)| *b);
        let balances: Vec<i64> = rows.iter().filter_map(|r| r.balance).collect();
        let corrido = balances.len() == rows.len()
            && rows.len() > 1
            && balances.iter().any(|b| *b != balances[0]);
        if corrido {
            // Saldo corrido: cada linha = a anterior + o valor.
            let mut prev = running;
            for r in &rows {
                let b = r.balance.unwrap_or_default();
                if let Some(p) = prev
                    && p + r.amount != b
                {
                    st.problems.push(DayProblem {
                        date: d,
                        printed: b,
                        computed: p + r.amount,
                    });
                }
                prev = Some(b);
            }
            running = prev;
            continue;
        }
        let sum: i64 = rows.iter().map(|r| r.amount).sum();
        match (running, closing) {
            (Some(p), Some(c)) => {
                if p + sum != c {
                    st.problems.push(DayProblem {
                        date: d,
                        printed: c,
                        computed: p + sum,
                    });
                }
                // O próximo dia começa do impresso (um erro não contamina os outros).
                running = Some(c);
            }
            (None, Some(c)) => running = Some(c),
            (Some(p), None) => running = Some(p + sum),
            (None, None) => {}
        }
    }
    if let (Some(o), Some(c)) = (st.opening, st.closing) {
        let sum: i64 = st.rows.iter().map(|r| r.amount).sum();
        st.total_ok = Some(o + sum == c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/caixa-extrato-periodo.ocr.txt");

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    fn assert_expected(st: &Statement) {
        let got: Vec<(NaiveDate, i64, &str)> = st
            .rows
            .iter()
            .map(|r| (r.date, r.amount, r.history.as_str()))
            .collect();
        assert_eq!(
            got,
            vec![
                (d("2025-03-05"), 200_000, "CRED PIX CHAVE"),
                (d("2025-03-07"), -12_050, "PAG BOLETO"),
                (d("2025-03-07"), -7_990, "COMPRA ELO"),
                (d("2025-03-12"), -320_000, "DEBITO PRESTACAO HAB"),
            ]
        );
        assert_eq!(st.opening, Some(125_000));
        assert_eq!(st.closing, Some(-15_040));
        assert_eq!(st.total_ok, Some(true));
        assert!(st.problems.is_empty(), "{:?}", st.problems);
        // Masked CPF/CNPJ in no field.
        let all = format!("{:?}", st.rows);
        assert!(!all.contains('*') && !all.contains("123.456") && !all.contains("12.345"));
    }

    #[test]
    fn reads_the_invented_statement() {
        assert!(looks_like(FIXTURE));
        let st = parse(FIXTURE);
        assert_expected(&st);
        assert_eq!(st.period, Some((d("2025-03-01"), d("2025-03-15"))));
        assert_eq!(st.account.as_deref(), Some("0001 / 3701.000012345678-9"));
        assert_eq!(st.closings.len(), 3);
        let pix = &st.rows[0];
        assert_eq!(pix.payee.as_deref(), Some("Fulano Beltrano Si"));
        assert_eq!(pix.doc, "011926");
        assert_eq!(st.rows[2].payee.as_deref(), Some("Loja Exemplo Ltda"));
        assert_eq!(st.rows[3].payee, None);
        // The external id does not change between reads and is different per transaction.
        let again = parse(FIXTURE);
        assert_eq!(st.rows[1].external_id(), again.rows[1].external_id());
        assert_ne!(st.rows[1].external_id(), st.rows[2].external_id());
    }

    #[test]
    fn dirty_ocr_glued_cd_and_letter_o_in_doc() {
        let dirty = FIXTURE
            .replace("79,90 D", "79,90D")
            .replace("2.000,00 C  3.250,00 C", "2.000,00C  3.250,00C")
            .replace("004417", "OO4417")
            .replace("3.200,00 D", "3200,00 D");
        let st = parse(&dirty);
        assert_expected(&st);
        assert_eq!(st.rows[2].doc, "004417");
    }

    #[test]
    fn a_wrong_digit_is_flagged_on_its_day() {
        let st = parse(&FIXTURE.replace("120,50 D", "126,50 D"));
        assert_eq!(st.problems.len(), 1);
        assert_eq!(st.problems[0].date, d("2025-03-07"));
        assert_eq!(st.problems[0].printed, 304_960);
        assert_eq!(st.total_ok, Some(false));
        // The transaction stays there, for the person to fix.
        assert_eq!(st.rows.len(), 4);
    }

    #[test]
    fn two_pages_with_repeated_headers_do_not_duplicate() {
        let lines: Vec<&str> = FIXTURE.lines().collect();
        let header = &lines[..9];
        let (first, second) = lines[9..16].split_at(3);
        let footer = &lines[16..];
        let two = [header, first, footer, header, second, footer]
            .concat()
            .join("\n");
        let st = parse(&two);
        assert_expected(&st);
    }

    #[test]
    fn running_balance_line_by_line_is_accepted() {
        let corrido = FIXTURE
            .replace(
                "07/03/2025 - 10:02:11  120333  PAG BOLETO  Empresa Ficticia S  120,50 D  3.049,60 C",
                "07/03/2025 - 10:02:11  120333  PAG BOLETO  Empresa Ficticia S  120,50 D  3.129,50 C",
            )
            .replace("07/03/2025 - 00:00:00  000000  SALDO DIA  0,00 C  3.049,60 C\n", "");
        let st = parse(&corrido);
        assert_expected(&st);
    }

    #[test]
    fn other_documents_are_not_this_layout() {
        assert!(!looks_like(
            "Extrato de conta corrente\nSaldo anterior 10,00\nSaldo dia"
        ));
        assert!(!looks_like("Comprovante de Pix\nValor R$ 10,00"));
    }
}
