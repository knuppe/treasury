//! Leitura do texto de notas de negociação no padrão SINACOR (usado pela maioria das
//! corretoras brasileiras): número, data do pregão, linhas de negócio e o resumo
//! financeiro com as taxas e o IRRF.
//!
//! O texto vem da extração do PDF, que nem sempre preserva as colunas; por isso as linhas
//! são reconhecidas por padrão (mercado, C/V, tipo de mercado … quantidade preço valor D/C)
//! e os valores do resumo pelo rótulo.

use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;

use crate::portfolio::TradeSide;

/// Uma linha de negócio da nota.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteLine {
    pub side: TradeSide,
    /// VISTA, FRACIONARIO, OPCAO DE COMPRA…
    pub market: String,
    /// "Especificação do título" como veio (ex.: "PETROBRAS PN N2").
    pub spec: String,
    /// Observação (ex.: `D` = day trade, `#` = negócio direto).
    pub obs: Option<String>,
    pub quantity: Decimal,
    pub price: Decimal,
    /// Valor da operação na nota (unidade maior).
    pub value: Decimal,
}

/// Custos do resumo financeiro, em unidade maior (R$).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NoteFees {
    pub settlement: Decimal,
    pub registration: Decimal,
    pub emoluments: Decimal,
    pub brokerage: Decimal,
    pub iss: Decimal,
    pub other: Decimal,
    pub irrf: Decimal,
}

impl NoteFees {
    /// Tudo o que entra no custo das operações (sem o IRRF, que é antecipação de imposto).
    pub fn costs(&self) -> Decimal {
        self.settlement
            + self.registration
            + self.emoluments
            + self.brokerage
            + self.iss
            + self.other
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedNote {
    pub number: Option<String>,
    pub date: Option<NaiveDate>,
    pub lines: Vec<NoteLine>,
    pub fees: NoteFees,
    /// "Líquido para" a data de liquidação, com sinal (positivo = crédito).
    pub net: Option<Decimal>,
    pub warnings: Vec<String>,
}

/// Maiúsculas, sem acentos, espaços normalizados (mantém pontuação dos números).
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for ch in s.chars() {
        let c = crate::transactions::fold_accent(ch).to_ascii_uppercase();
        if c.is_whitespace() || c == '\u{a0}' {
            if !space && !out.is_empty() {
                out.push(' ');
            }
            space = true;
        } else {
            out.push(c);
            space = false;
        }
    }
    out.trim_end().to_string()
}

/// "1.234,56" → 1234.56.
pub fn br_number(s: &str) -> Option<Decimal> {
    let t = s.trim().trim_end_matches(['D', 'C']).trim();
    if t.is_empty() {
        return None;
    }
    t.replace('.', "").replace(',', ".").parse().ok()
}

static LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        (?:^|\s)(?:\d-)?(?:BOVESPA|B3\ RV\ LISTADO|B3\ RV|B3)\s+
        (?P<side>C|V)\s+
        (?P<market>VISTA|FRACIONARIO|OPCAO\ DE\ COMPRA|OPCAO\ DE\ VENDA|EXERC\ OPC\ COMPRA|EXERC\ OPC\ VENDA|TERMO|LEILAO)\s+
        (?:\d{2}/\d{2}\s+)?
        (?P<spec>.+?)\s+
        (?:(?P<obs>[\#DFBAHXPYLTI2]{1,2})\s+)?
        (?P<qty>\d[\d.]*)\s+
        (?P<price>\d[\d.]*,\d{2,8})\s+
        (?P<value>\d[\d.]*,\d{2})\s+
        (?P<dc>D|C)\b",
    )
    .expect("regex da linha de negócio")
});

static DATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{2}/\d{2}/\d{4})\b").expect("regex de data"));
static INT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{1,12})\b").expect("regex de número"));
static AMOUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d[\d.]*,\d{2})(?:\s*([DC])\b)?").expect("regex de valor"));

/// Primeiro valor depois de `label` na mesma linha (ou o último, para o IRRF que traz a base
/// antes do valor).
fn amount_after(text: &str, label: &str, last: bool) -> Option<(Decimal, Option<char>)> {
    for line in text.lines() {
        let Some(pos) = line.find(label) else {
            continue;
        };
        let rest = &line[pos + label.len()..];
        let mut caps: Vec<(Decimal, Option<char>)> = AMOUNT
            .captures_iter(rest)
            .filter_map(|c| {
                let v = br_number(&c[1])?;
                let dc = c.get(2).and_then(|m| m.as_str().chars().next());
                Some((v, dc))
            })
            .collect();
        if caps.is_empty() {
            continue;
        }
        return Some(if last {
            caps.pop().expect("não vazio")
        } else {
            caps.remove(0)
        });
    }
    None
}

/// O resto da linha depois do rótulo, ou a linha seguinte quando o resto não tem dígitos
/// não vazia (muitas notas põem os rótulos numa linha e os valores na de baixo).
fn value_line<'a>(text: &'a str, label: &str) -> Option<&'a str> {
    let pos = text.find(label)?;
    let rest = &text[pos + label.len()..];
    let (same, after) = rest.split_once('\n').unwrap_or((rest, ""));
    if same.chars().any(|c| c.is_ascii_digit()) {
        Some(same)
    } else {
        after.lines().find(|l| !l.trim().is_empty())
    }
}

fn parse_one(chunk: &str) -> ParsedNote {
    let mut warnings = vec![];
    let date = value_line(chunk, "DATA PREGAO")
        .and_then(|w| DATE.captures(w))
        .or_else(|| DATE.captures(chunk))
        .and_then(|c| NaiveDate::parse_from_str(&c[1], "%d/%m/%Y").ok());
    let number = value_line(chunk, "NR. NOTA")
        .or_else(|| value_line(chunk, "NR.NOTA"))
        .or_else(|| value_line(chunk, "NUMERO DA NOTA"))
        .and_then(|w| {
            // Pula a data e a folha: o número da nota é o maior inteiro da linha.
            let w = DATE.replace_all(w, " ");
            INT.captures_iter(&w)
                .map(|c| c[1].to_string())
                .max_by_key(|s| s.len())
        });
    let mut lines = vec![];
    for line in chunk.lines() {
        for c in LINE.captures_iter(line) {
            let (Some(q), Some(p), Some(v)) = (
                br_number(&c["qty"]),
                br_number(&c["price"]),
                br_number(&c["value"]),
            ) else {
                continue;
            };
            let side = if &c["side"] == "C" {
                TradeSide::Buy
            } else {
                TradeSide::Sell
            };
            if (q * p - v).abs() > Decimal::new(5, 2).max(v * Decimal::new(1, 3)) {
                warnings.push(crate::i18n::t_args(
                    "nucleo-importacao-corretagem-aviso-valor",
                    &crate::i18n::args(&[
                        ("especificacao", c["spec"].trim().to_string().into()),
                        ("calculado", (q * p).round_dp(2).to_string().into()),
                        ("valor", v.to_string().into()),
                    ]),
                ));
            }
            lines.push(NoteLine {
                side,
                market: c["market"].to_string(),
                spec: c["spec"].trim().to_string(),
                obs: c.name("obs").map(|m| m.as_str().to_string()),
                quantity: q,
                price: p,
                value: v,
            });
        }
    }
    let get = |label: &str| amount_after(chunk, label, false).map(|(v, _)| v);
    let fees = NoteFees {
        settlement: get("TAXA DE LIQUIDACAO").unwrap_or_default(),
        registration: get("TAXA DE REGISTRO").unwrap_or_default(),
        emoluments: get("EMOLUMENTOS").unwrap_or_default(),
        brokerage: get("TAXA OPERACIONAL")
            .or_else(|| get("CORRETAGEM"))
            .unwrap_or_default()
            + get("EXECUCAO").unwrap_or_default(),
        iss: get("IMPOSTOS").or_else(|| get("ISS")).unwrap_or_default(),
        other: get("TAXA DE TERMO/OPCOES").unwrap_or_default()
            + get("TAXA A.N.A.").unwrap_or_default()
            + get("TAXA DE CUSTODIA").unwrap_or_default()
            // Depositária (layout de 2025): transferência de ativos da B3.
            + get("TAXA DE TRANSFERENCIA DE ATIVOS").unwrap_or_default()
            + get("OUTRAS").unwrap_or_default(),
        irrf: amount_after(chunk, "I.R.R.F.", true)
            .or_else(|| amount_after(chunk, "IRRF", true))
            .map(|(v, _)| v)
            .unwrap_or_default(),
    };
    let net = amount_after(chunk, "LIQUIDO PARA", true)
        .map(|(v, dc)| if dc == Some('D') { -v } else { v });
    if date.is_none() {
        warnings.push(crate::i18n::t(
            "nucleo-importacao-corretagem-aviso-sem-data",
        ));
    }
    ParsedNote {
        number,
        date,
        lines,
        fees,
        net,
        warnings,
    }
}

/// Divide o texto em notas (um PDF pode trazer várias) e junta as páginas de continuação
/// da mesma nota. Notas sem nenhuma linha de negócio são descartadas.
pub fn parse_text(text: &str) -> Vec<ParsedNote> {
    let norm: String = text.lines().map(normalize).collect::<Vec<_>>().join("\n");
    let markers = ["NOTA DE NEGOCIACAO", "NOTA DE CORRETAGEM"];
    let mut starts: Vec<usize> = markers
        .iter()
        .flat_map(|m| norm.match_indices(m).map(|(i, _)| i))
        .collect();
    starts.sort_unstable();
    starts.dedup();
    if starts.first() != Some(&0) {
        starts.insert(0, 0);
    }
    let mut chunks = vec![];
    for (i, s) in starts.iter().enumerate() {
        let e = starts.get(i + 1).copied().unwrap_or(norm.len());
        chunks.push(&norm[*s..e]);
    }
    let mut notes: Vec<ParsedNote> = vec![];
    for c in chunks {
        let n = parse_one(c);
        if let Some(prev) = notes
            .iter_mut()
            .find(|p| p.number.is_some() && p.number == n.number && p.date == n.date)
        {
            // Página de continuação: o resumo só vem completo na última.
            prev.lines.extend(n.lines);
            if n.fees != NoteFees::default() {
                prev.fees = n.fees;
            }
            if n.net.is_some() {
                prev.net = n.net;
            }
            prev.warnings.extend(n.warnings);
        } else if !n.lines.is_empty() || n.fees != NoteFees::default() {
            notes.push(n);
        }
    }
    notes.retain(|n| !n.lines.is_empty());
    notes
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    pub const SAMPLE: &str = "NOTA DE NEGOCIAÇÃO\n\
        Nr. nota Folha Data pregão\n\
        123456 1 13/10/2026\n\
        Corretora Exemplo CTVM S.A.\n\
        Negócios realizados\n\
        Q Negociação C/V Tipo mercado Prazo Especificação do título Obs. (*) Quantidade Preço / Ajuste Valor Operação / Ajuste D/C\n\
        1-BOVESPA C VISTA PETROBRAS PN N2 100 28,50 2.850,00 D\n\
        1-BOVESPA C FRACIONARIO ITAUSA PN N1 7 10,00 70,00 D\n\
        1-BOVESPA V VISTA FII HGLG PAX HGLG11 CI # 10 160,00 1.600,00 C\n\
        Resumo dos Negócios\n\
        Taxa de liquidação 0,95 D\n\
        Taxa de Registro 0,00 D\n\
        Emolumentos 0,23 D\n\
        Taxa Operacional 0,00 D\n\
        Impostos 0,00 D\n\
        I.R.R.F. s/ operações, base R$ 1.600,00 0,08\n\
        Outras 0,00 D\n\
        Líquido para 15/10/2026 1.321,26 D\n";

    #[test]
    fn parses_sinacor_lines_fees_and_header() {
        let notes = parse_text(SAMPLE);
        assert_eq!(notes.len(), 1);
        let n = &notes[0];
        assert_eq!(n.number.as_deref(), Some("123456"));
        assert_eq!(n.date, NaiveDate::from_ymd_opt(2026, 10, 13));
        assert_eq!(n.lines.len(), 3);
        assert_eq!(n.lines[0].spec, "PETROBRAS PN N2");
        assert_eq!(n.lines[0].side, TradeSide::Buy);
        assert_eq!(
            (n.lines[0].quantity, n.lines[0].price),
            (dec!(100), dec!(28.50))
        );
        assert_eq!(n.lines[1].market, "FRACIONARIO");
        assert_eq!(n.lines[2].side, TradeSide::Sell);
        assert_eq!(n.lines[2].obs.as_deref(), Some("#"));
        assert_eq!(n.lines[2].spec, "FII HGLG PAX HGLG11 CI");
        assert_eq!(n.fees.settlement, dec!(0.95));
        assert_eq!(n.fees.emoluments, dec!(0.23));
        assert_eq!(
            n.fees.irrf,
            dec!(0.08),
            "IRRF is the last value on the line, not the base"
        );
        assert_eq!(n.fees.costs(), dec!(1.18));
        assert_eq!(n.net, Some(dec!(-1321.26)));
        assert!(n.warnings.is_empty(), "{:?}", n.warnings);
    }

    #[test]
    fn joins_continuation_pages_and_splits_notes() {
        let page2 = SAMPLE
            .replace("123456", "123457")
            .replace("13/10/2026", "14/10/2026");
        let first_page_only = "NOTA DE NEGOCIACAO\nNr. nota Folha Data pregão\n999 1 01/10/2026\n\
            1-BOVESPA C VISTA VALE ON NM 10 60,00 600,00 D\nCONTINUA...\n";
        let last_page = "NOTA DE NEGOCIACAO\nNr. nota Folha Data pregão\n999 2 01/10/2026\n\
            1-BOVESPA C VISTA WEG ON NM 5 40,00 200,00 D\nTaxa de liquidação 0,22 D\nEmolumentos 0,04 D\n";
        let text = format!("{SAMPLE}\n{page2}\n{first_page_only}\n{last_page}");
        let notes = parse_text(&text);
        assert_eq!(notes.len(), 3);
        let n = notes
            .iter()
            .find(|n| n.number.as_deref() == Some("999"))
            .unwrap();
        assert_eq!(n.lines.len(), 2);
        assert_eq!(n.fees.costs(), dec!(0.26));
    }

    #[test]
    fn day_trade_obs_and_mismatch_warning() {
        let text = "NOTA DE NEGOCIACAO\nData pregão 02/10/2026\nNr. nota 55\n\
            1-BOVESPA C VISTA BRADESCO PN N1 D 100 15,00 1.500,00 D\n\
            1-BOVESPA V VISTA BRADESCO PN N1 D 100 15,50 1.560,00 C\n";
        let n = &parse_text(text)[0];
        assert_eq!(n.lines[0].obs.as_deref(), Some("D"));
        assert_eq!(n.lines[0].spec, "BRADESCO PN N1");
        assert_eq!(n.warnings.len(), 1, "{:?}", n.warnings);
    }
}
