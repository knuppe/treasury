//! CSV de extratos e faturas: detecção de delimitador e da linha de cabeçalho, mapeamento
//! de colunas (índice ou nome) e presets dos bancos mais comuns.

use chrono::NaiveDate;

use super::{CsvMapping, CsvPreset};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::import::text::fold;

/// Linha lida do CSV (célula já sem espaços nas pontas).
#[derive(Debug, Clone)]
pub struct Record {
    /// Linha física no arquivo (1-based).
    pub line: u32,
    pub cells: Vec<String>,
}

const CANDIDATES: &[u8] = b";,\t|";

pub fn delimiter_str(d: u8) -> String {
    match d {
        b'\t' => "\t".into(),
        other => (other as char).to_string(),
    }
}

pub fn parse_delimiter(s: &str) -> Result<u8> {
    match s {
        "\t" | "\\t" | "tab" | "TAB" => Ok(b'\t'),
        s if s.len() == 1 && s.is_ascii() => Ok(s.as_bytes()[0]),
        _ => Err(Error::invalid(t_args(
            "nucleo-importacao-csv-erro-delimitador",
            &args(&[("valor", format!("{s:?}").into())]),
        ))),
    }
}

/// Lê todos os registros (tolerante a linhas com número diferente de colunas).
pub fn read_records(text: &str, delimiter: u8) -> Result<Vec<Record>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .quoting(true)
        .from_reader(text.as_bytes());
    let mut out = Vec::new();
    // Linha física pelo deslocamento em bytes (a contagem do crate pula linhas vazias).
    let bytes = text.as_bytes();
    let (mut seen_byte, mut seen_line) = (0usize, 1u32);
    for rec in rdr.records() {
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                return Err(Error::invalid(t_args(
                    "nucleo-importacao-csv-erro-malformado",
                    &args(&[("erro", e.to_string().into())]),
                )));
            }
        };
        let mut byte = rec.position().map(|p| p.byte() as usize).unwrap_or(0);
        // A posição pode apontar para as linhas vazias puladas antes do registro.
        while byte < bytes.len() && (bytes[byte] == b'\n' || bytes[byte] == b'\r') {
            byte += 1;
        }
        if byte > seen_byte {
            let end = byte.min(bytes.len());
            seen_line += bytes[seen_byte..end]
                .iter()
                .filter(|b| **b == b'\n')
                .count() as u32;
            seen_byte = end;
        }
        let line = seen_line;
        let cells: Vec<String> = rec
            .iter()
            .map(|c| c.trim().trim_matches('\u{feff}').trim().to_string())
            .collect();
        if cells.iter().all(|c| c.is_empty()) {
            continue;
        }
        out.push(Record { line, cells });
    }
    Ok(out)
}

/// Escolhe o delimitador pelo número de campos mais consistente entre as linhas.
pub fn detect_delimiter(text: &str) -> u8 {
    let mut best = (b';', 0usize, 0usize, usize::MAX); // (delim, linhas na moda, moda, prioridade)
    for (prio, &d) in CANDIDATES.iter().enumerate() {
        let Ok(recs) = read_records(text, d) else {
            continue;
        };
        let counts: Vec<usize> = recs.iter().take(300).map(|r| r.cells.len()).collect();
        let (mode, freq) = mode_of(&counts);
        if mode < 2 {
            continue;
        }
        let better = freq > best.1
            || (freq == best.1 && mode > best.2)
            || (freq == best.1 && mode == best.2 && prio < best.3);
        if better {
            best = (d, freq, mode, prio);
        }
    }
    best.0
}

fn mode_of(v: &[usize]) -> (usize, usize) {
    let mut counts = std::collections::BTreeMap::new();
    for &x in v {
        *counts.entry(x).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)))
        .unwrap_or((0, 0))
}

/// Remove as primeiras `n` linhas físicas.
pub fn skip_lines(text: &str, n: u32) -> &str {
    let mut rest = text;
    for _ in 0..n {
        match rest.find('\n') {
            Some(i) => rest = &rest[i + 1..],
            None => return "",
        }
    }
    rest
}

/// Layout detectado: quantas linhas pular antes da tabela e se a 1ª linha é cabeçalho.
#[derive(Debug, Clone)]
pub struct Layout {
    pub skip_rows: u32,
    pub has_header: bool,
    pub headers: Vec<String>,
}

/// Encontra o início da tabela (pula preâmbulos como "Extrato de conta corrente",
/// "Agência: …") e decide se a primeira linha da tabela é cabeçalho.
pub fn detect_layout(text: &str, delimiter: u8) -> Result<Layout> {
    let recs = read_records(text, delimiter)?;
    let counts: Vec<usize> = recs.iter().map(|r| r.cells.len()).collect();
    let (mode, _) = mode_of(&counts);
    let Some(first) = recs.iter().position(|r| r.cells.len() == mode) else {
        return Ok(Layout {
            skip_rows: 0,
            has_header: false,
            headers: vec![],
        });
    };
    let rec = &recs[first];
    let has_header = !rec.cells.iter().any(|c| looks_like_data(c))
        && recs
            .get(first + 1)
            .is_none_or(|n| n.cells.iter().any(|c| looks_like_data(c)));
    let headers = if has_header {
        rec.cells.clone()
    } else {
        (1..=rec.cells.len())
            .map(|i| format!("Coluna {i}"))
            .collect()
    };
    Ok(Layout {
        skip_rows: rec.line.saturating_sub(1),
        has_header,
        headers,
    })
}

fn looks_like_data(cell: &str) -> bool {
    let c = cell.trim();
    if c.is_empty() {
        return false;
    }
    guess_date(c).is_some() || crate::money::parse_amount(c, "BRL").is_ok()
}

const COMMON_DATE_FORMATS: &[&str] = &["%d/%m/%Y", "%Y-%m-%d", "%d/%m/%y", "%d-%m-%Y", "%d.%m.%Y"];

fn guess_date(c: &str) -> Option<NaiveDate> {
    COMMON_DATE_FORMATS.iter().find_map(|f| {
        NaiveDate::parse_and_remainder(c, f)
            .ok()
            .map(|(d, _)| d)
            .filter(plausible)
    })
}

/// `%Y` aceita "26" como ano 26; datas fora de 1900–2200 são outro formato.
fn plausible(d: &NaiveDate) -> bool {
    (1900..=2200).contains(&chrono::Datelike::year(d))
}

/// Data com um ou mais formatos (`%d/%m/%Y|%d/%m/%y`); aceita hora depois da data e,
/// como último recurso, ISO `AAAA-MM-DD` (inequívoco).
pub fn parse_date(cell: &str, formats: &str) -> Option<NaiveDate> {
    let c = cell.trim();
    for f in formats.split('|').map(str::trim).filter(|f| !f.is_empty()) {
        if let Ok((d, rest)) = NaiveDate::parse_and_remainder(c, f)
            && plausible(&d)
            && (rest.is_empty() || rest.starts_with([' ', 'T', ',']))
        {
            return Some(d);
        }
    }
    NaiveDate::parse_and_remainder(c, "%Y-%m-%d")
        .ok()
        .map(|(d, _)| d)
        .filter(plausible)
}

/// Valor de uma célula. `decimal_comma`: `1.234,56`; senão `1,234.56`.
pub fn parse_cell_amount(cell: &str, decimal_comma: bool, currency: &str) -> Result<i64> {
    let c = cell.trim();
    if decimal_comma {
        return crate::money::parse_amount(c, currency);
    }
    let without_thousands = c.replace(',', "");
    let canonical = match without_thousands.rfind('.') {
        Some(i) => format!(
            "{},{}",
            &without_thousands[..i],
            &without_thousands[i + 1..]
        ),
        None => without_thousands,
    };
    crate::money::parse_amount(&canonical, currency).map_err(|_| {
        Error::invalid(t_args(
            "nucleo-importacao-erro-valor",
            &args(&[("valor", c.to_string().into())]),
        ))
    })
}

/// Referência a coluna(s): índice (`0`), nome do cabeçalho (sem diferenciar maiúsculas e
/// acentos) ou várias unidas por `+` (`Histórico+Descrição`).
pub fn resolve_columns(spec: &str, headers: Option<&[String]>) -> Result<Vec<usize>> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err(Error::invalid(t("nucleo-importacao-csv-erro-sem-coluna")));
    }
    if let Some(h) = headers {
        let key = fold(spec);
        if let Some(i) = h.iter().position(|x| fold(x) == key) {
            return Ok(vec![i]);
        }
    }
    if let Ok(i) = spec.parse::<usize>() {
        return Ok(vec![i]);
    }
    if spec.contains('+') {
        let mut out = Vec::new();
        for part in spec.split('+') {
            out.extend(resolve_columns(part, headers)?);
        }
        return Ok(out);
    }
    Err(Error::invalid(match headers {
        Some(h) => t_args(
            "nucleo-importacao-csv-erro-coluna",
            &args(&[
                ("coluna", spec.to_string().into()),
                ("colunas", h.join(", ").into()),
            ]),
        ),
        None => t_args(
            "nucleo-importacao-csv-erro-coluna-sem-cabecalho",
            &args(&[("coluna", spec.to_string().into())]),
        ),
    }))
}

/// Linha lida segundo o mapeamento.
#[derive(Debug, Clone, Default)]
pub struct CsvRow {
    pub line: u32,
    pub date: Option<NaiveDate>,
    pub amount: Option<i64>,
    pub description: String,
    pub payee: Option<String>,
    pub external_id: Option<String>,
    pub error: Option<String>,
}

pub struct ParsedCsv {
    pub delimiter: u8,
    pub skip_rows: u32,
    pub headers: Vec<String>,
    pub rows: Vec<CsvRow>,
}

struct Cols {
    date: Vec<usize>,
    amount: Option<Vec<usize>>,
    debit: Option<Vec<usize>>,
    credit: Option<Vec<usize>>,
    description: Vec<usize>,
    payee: Option<Vec<usize>>,
    external_id: Option<Vec<usize>>,
}

fn joined(cells: &[String], cols: &[usize]) -> String {
    cols.iter()
        .filter_map(|&i| cells.get(i))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" - ")
}

/// Favorecido a partir do descritor da maquininha: sem o prefixo do adquirente ou do app
/// (`IFD*`, `MP*`, `ZP*`, `CAPPTA *`, `PAG*`…) e sem o país no fim (`BRA`). Créditos sem
/// estabelecimento ("Depósito transferido") ficam sem favorecido.
pub fn card_payee(descriptor: &str) -> String {
    let mut s = descriptor.trim();
    if fold(s).starts_with("deposito") {
        return String::new();
    }
    if let Some(i) = s.find('*')
        && i <= 10
        && s[..i]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == ' ')
    {
        s = s[i + 1..].trim_start();
    }
    for suffix in [" BRA", " BR"] {
        if s.len() > suffix.len() && s.to_uppercase().ends_with(suffix) {
            s = s[..s.len() - suffix.len()].trim_end();
            break;
        }
    }
    s.to_string()
}

/// Lê o CSV com o mapeamento. Linhas sem data e sem valor (rodapés, totais em branco) são
/// descartadas; demais problemas viram linhas com erro.
pub fn parse(text: &str, mapping: &CsvMapping, currency: &str) -> Result<ParsedCsv> {
    let delimiter = match mapping.delimiter.as_deref().filter(|d| !d.is_empty()) {
        Some(d) => parse_delimiter(d)?,
        None => detect_delimiter(text),
    };
    let skip_rows = match mapping.skip_rows {
        Some(n) => n,
        None => detect_layout(text, delimiter)?.skip_rows,
    };
    let body = skip_lines(text, skip_rows);
    let mut recs = read_records(body, delimiter)?.into_iter();
    let headers: Option<Vec<String>> = if mapping.has_header {
        recs.next().map(|r| r.cells)
    } else {
        None
    };
    let h = headers.as_deref();
    let opt = |s: &Option<String>| -> Result<Option<Vec<usize>>> {
        match s.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) => resolve_columns(s, h).map(Some),
            None => Ok(None),
        }
    };
    let cols = Cols {
        date: resolve_columns(&mapping.date_column, h)?,
        amount: opt(&mapping.amount_column)?,
        debit: opt(&mapping.debit_column)?,
        credit: opt(&mapping.credit_column)?,
        description: resolve_columns(&mapping.description_column, h)?,
        payee: opt(&mapping.payee_column)?,
        external_id: opt(&mapping.external_id_column)?,
    };
    if cols.amount.is_none() && cols.debit.is_none() && cols.credit.is_none() {
        return Err(Error::invalid(t("nucleo-importacao-csv-erro-sem-valor")));
    }
    let mut rows = Vec::new();
    for rec in recs {
        let cells = &rec.cells;
        let line = rec.line + skip_rows;
        let date_cell = joined(cells, &cols.date);
        let amount_cells: Vec<String> = [&cols.amount, &cols.debit, &cols.credit]
            .iter()
            .filter_map(|c| c.as_ref())
            .map(|c| joined(cells, c))
            .collect();
        if date_cell.is_empty() && amount_cells.iter().all(|c| c.is_empty()) {
            continue;
        }
        let mut row = CsvRow {
            line,
            description: joined(cells, &cols.description),
            payee: cols
                .payee
                .as_ref()
                .map(|c| joined(cells, c))
                .map(|p| {
                    if mapping.card_descriptor == Some(true) {
                        card_payee(&p)
                    } else {
                        p
                    }
                })
                .filter(|s| !s.is_empty()),
            external_id: cols
                .external_id
                .as_ref()
                .map(|c| joined(cells, c))
                .filter(|s| !s.is_empty()),
            ..Default::default()
        };
        row.date = parse_date(&date_cell, &mapping.date_format);
        if row.date.is_none() {
            row.error = Some(t_args(
                "nucleo-importacao-csv-erro-data",
                &args(&[
                    ("valor", format!("{date_cell:?}").into()),
                    ("formato", mapping.date_format.clone().into()),
                ]),
            ));
        }
        match amount_of(cells, &cols, mapping, currency) {
            Ok(a) => row.amount = Some(if mapping.invert_sign { -a } else { a }),
            Err(e) => {
                row.error.get_or_insert_with(|| e.message());
            }
        }
        rows.push(row);
    }
    Ok(ParsedCsv {
        delimiter,
        skip_rows,
        headers: headers.unwrap_or_default(),
        rows,
    })
}

fn amount_of(cells: &[String], cols: &Cols, m: &CsvMapping, currency: &str) -> Result<i64> {
    let cell = |c: &Option<Vec<usize>>| c.as_ref().map(|c| joined(cells, c)).unwrap_or_default();
    if cols.amount.is_some() {
        let v = cell(&cols.amount);
        if !v.is_empty() || (cols.debit.is_none() && cols.credit.is_none()) {
            return parse_cell_amount(&v, m.decimal_comma, currency);
        }
    }
    let (d, c) = (cell(&cols.debit), cell(&cols.credit));
    if d.is_empty() && c.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-importacao-csv-erro-linha-sem-valor",
        )));
    }
    let mut total = 0i64;
    if !c.is_empty() {
        total += parse_cell_amount(&c, m.decimal_comma, currency)?.abs();
    }
    if !d.is_empty() {
        total -= parse_cell_amount(&d, m.decimal_comma, currency)?.abs();
    }
    Ok(total)
}

#[allow(clippy::too_many_arguments)]
fn mapping(
    delimiter: Option<&str>,
    date: (&str, &str),
    amount: Option<&str>,
    debit_credit: Option<(&str, &str)>,
    description: &str,
    external_id: Option<&str>,
    decimal_comma: bool,
    invert_sign: bool,
) -> CsvMapping {
    CsvMapping {
        delimiter: delimiter.map(str::to_string),
        has_header: true,
        skip_rows: None,
        date_column: date.0.into(),
        date_format: date.1.into(),
        amount_column: amount.map(str::to_string),
        debit_column: debit_credit.map(|d| d.0.to_string()),
        credit_column: debit_credit.map(|d| d.1.to_string()),
        description_column: description.into(),
        payee_column: None,
        external_id_column: external_id.map(str::to_string),
        decimal_comma,
        invert_sign,
        encoding: None,
        card_descriptor: None,
    }
}

pub const GENERIC_PRESET: &str = "generico";

/// Presets conhecidos. O cabeçalho de cada um é a base da sugestão em `import.detect`.
pub fn presets() -> Vec<CsvPreset> {
    // `name`: chave do nome no catálogo (o banco fica; o tipo de conta sai no idioma).
    let p = |id: &str, name: &str, mapping: CsvMapping| CsvPreset {
        id: id.into(),
        name: t(name),
        mapping,
    };
    vec![
        p(
            "nubank_conta",
            "nucleo-importacao-preset-nubank-conta",
            mapping(
                Some(","),
                ("Data", "%d/%m/%Y"),
                Some("Valor"),
                None,
                "Descrição",
                Some("Identificador"),
                false,
                false,
            ),
        ),
        p(
            "nubank_cartao",
            "nucleo-importacao-preset-nubank-cartao",
            mapping(
                Some(","),
                ("date", "%Y-%m-%d"),
                Some("amount"),
                None,
                "title",
                None,
                false,
                true,
            ),
        ),
        p(
            "inter",
            "nucleo-importacao-preset-inter",
            mapping(
                Some(";"),
                ("Data Lançamento", "%d/%m/%Y"),
                Some("Valor"),
                None,
                "Histórico+Descrição",
                None,
                true,
                false,
            ),
        ),
        p(
            "itau",
            "nucleo-importacao-preset-itau",
            mapping(
                Some(";"),
                ("data", "%d/%m/%Y"),
                Some("valor (R$)"),
                None,
                "lançamento",
                None,
                true,
                false,
            ),
        ),
        p(
            "c6",
            "nucleo-importacao-preset-c6",
            mapping(
                Some(";"),
                ("Data Lançamento", "%d/%m/%Y"),
                None,
                Some(("Saída(R$)", "Entrada(R$)")),
                "Título+Descrição",
                None,
                true,
                false,
            ),
        ),
        p(
            "c6_cartao",
            "nucleo-importacao-preset-c6-cartao",
            mapping(
                Some(";"),
                ("Data de Compra", "%d/%m/%Y"),
                Some("Valor (em R$)"),
                None,
                "Descrição+Parcela",
                None,
                false,
                true,
            ),
        ),
        // Cartão de benefício Flash (VA/VR): `Data,Hora,Movimentação,Valor,Meio de Pagamento,Saldo`,
        // valores "-R$ 40,90"; sem id, a data, a hora, o estabelecimento e o valor identificam.
        p(
            "flash",
            "nucleo-importacao-preset-flash",
            CsvMapping {
                payee_column: Some("Movimentação".into()),
                external_id_column: Some("Data+Hora+Movimentação+Valor".into()),
                card_descriptor: Some(true),
                ..mapping(
                    Some(","),
                    ("Data", "%d/%m/%Y"),
                    Some("Valor"),
                    None,
                    "Movimentação",
                    None,
                    true,
                    false,
                )
            },
        ),
        p(
            GENERIC_PRESET,
            "nucleo-importacao-preset-generico",
            CsvMapping {
                delimiter: None,
                has_header: true,
                skip_rows: None,
                date_column: "0".into(),
                date_format: "%d/%m/%Y|%Y-%m-%d".into(),
                amount_column: Some("2".into()),
                debit_column: None,
                credit_column: None,
                description_column: "1".into(),
                payee_column: None,
                external_id_column: None,
                decimal_comma: true,
                invert_sign: false,
                encoding: None,
                card_descriptor: None,
            },
        ),
    ]
}

pub fn preset(id: &str) -> Result<CsvPreset> {
    presets().into_iter().find(|p| p.id == id).ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-importacao-ref-preset",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

fn referenced_columns(m: &CsvMapping) -> Vec<String> {
    let mut v = vec![m.date_column.clone(), m.description_column.clone()];
    v.extend(m.amount_column.clone());
    v.extend(m.debit_column.clone());
    v.extend(m.credit_column.clone());
    v.extend(m.external_id_column.clone());
    v.extend(m.payee_column.clone());
    v.into_iter()
        .flat_map(|s| s.split('+').map(str::to_string).collect::<Vec<_>>())
        .collect()
}

/// Preset cujo cabeçalho bate com o do arquivo (todas as colunas usadas existem); entre
/// vários, o que usa mais colunas.
pub fn suggest_preset(headers: &[String]) -> Option<String> {
    let have: Vec<String> = headers.iter().map(|h| fold(h)).collect();
    presets()
        .into_iter()
        .filter(|p| p.id != GENERIC_PRESET)
        .filter_map(|p| {
            let cols = referenced_columns(&p.mapping);
            cols.iter()
                .all(|c| have.contains(&fold(c)))
                .then_some((cols.len(), p.id))
        })
        .max_by_key(|(n, _)| *n)
        .map(|(_, id)| id)
}

/// Países em que o ponto é o separador decimal nos extratos (nos outros, a vírgula).
pub const DOT_DECIMAL_COUNTRIES: &[&str] = &[
    "US", "GB", "CA", "AU", "NZ", "IE", "IN", "MX", "JP", "CN", "SG", "HK", "IL", "PH", "MY", "TH",
];

// ---------------------------------------------------------------------------
// Mapeamento adivinhado (CSV de banco sem preset; docs/international.md)

/// Formato das datas de uma coluna pelos valores: ISO, dia antes do mês ou mês antes do dia
/// (EUA). Um valor com o primeiro número acima de 12 decide "dia primeiro"; com o segundo
/// acima de 12, "mês primeiro"; sem nenhum, vale `month_first`.
pub fn guess_date_format(values: &[&str], month_first: bool) -> Option<String> {
    let mut parts = vec![];
    for v in values {
        let v = v.trim();
        let head = v.split([' ', 'T']).next().unwrap_or("");
        if head.is_empty() {
            continue;
        }
        if NaiveDate::parse_from_str(head, "%Y-%m-%d").is_ok() {
            return Some("%Y-%m-%d".into());
        }
        let sep = head.chars().find(|c| matches!(c, '/' | '-' | '.'))?;
        let nums: Vec<&str> = head.split(sep).collect();
        if nums.len() != 3
            || nums
                .iter()
                .any(|n| n.is_empty() || !n.chars().all(|c| c.is_ascii_digit()))
        {
            return None;
        }
        parts.push((
            sep,
            nums[0].parse::<u32>().ok()?,
            nums[1].parse::<u32>().ok()?,
            nums[2].len(),
        ));
    }
    let (sep, _, _, ylen) = *parts.first()?;
    let day_first = parts.iter().any(|p| p.1 > 12);
    let month_first_seen = parts.iter().any(|p| p.2 > 12);
    let mf = match (day_first, month_first_seen) {
        (true, false) => false,
        (false, true) => true,
        (false, false) => month_first,
        (true, true) => return None,
    };
    let y = if ylen == 2 { "%y" } else { "%Y" };
    let f = if mf {
        format!("%m{sep}%d{sep}{y}")
    } else {
        format!("%d{sep}%m{sep}{y}")
    };
    // Confere: todas as datas da amostra valem nesse formato.
    values
        .iter()
        .filter(|v| !v.trim().is_empty())
        .all(|v| parse_date(v, &f).is_some())
        .then_some(f)
}

/// Vírgula decimal pelos valores: o último separador de um valor com os dois decide; só com
/// vírgula seguida de 1 ou 2 dígitos, vírgula; só com ponto seguido de 1 ou 2 dígitos, ponto.
pub fn guess_decimal_comma(values: &[&str], default: bool) -> bool {
    let (mut comma, mut dot) = (0, 0);
    for v in values {
        let v = v.trim();
        let (c, d) = (v.rfind(','), v.rfind('.'));
        match (c, d) {
            (Some(c), Some(d)) => {
                if c > d {
                    comma += 1
                } else {
                    dot += 1
                }
            }
            (Some(c), None) if (1..=2).contains(&(v.len() - c - 1)) => comma += 1,
            (None, Some(d)) if (1..=2).contains(&(v.len() - d - 1)) => dot += 1,
            _ => {}
        }
    }
    if comma == dot { default } else { comma > dot }
}

fn is_amount(v: &str) -> bool {
    let c: String = v
        .trim()
        .chars()
        .filter(|c| {
            !c.is_whitespace() && !matches!(c, '$' | '€' | '£' | 'R' | '(' | ')' | '+' | '-' | '−')
        })
        .collect();
    !c.is_empty()
        && c.chars().any(|ch| ch.is_ascii_digit())
        && c.chars()
            .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == ',')
}

/// Coluna pelo nome do cabeçalho (palavras sem acento, em vários idiomas).
fn header_like(headers: &[String], words: &[&str]) -> Option<usize> {
    headers.iter().position(|h| {
        let h = fold(h);
        words
            .iter()
            .any(|w| h == *w || h.split(' ').any(|p| p == *w))
    })
}

/// Um mapeamento para o CSV de um banco sem preset: a coluna das datas (e o formato), o valor
/// (uma coluna com sinal, ou débito e crédito), a vírgula decimal e a descrição. `month_first`
/// e `decimal_comma` decidem quando a amostra não basta (o padrão do país da família).
pub fn guess_mapping(
    headers: &[String],
    rows: &[Vec<String>],
    month_first: bool,
    decimal_comma: bool,
) -> Option<CsvMapping> {
    let n = headers
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0));
    let col = |i: usize| -> Vec<&str> {
        rows.iter()
            .filter_map(|r| r.get(i).map(String::as_str))
            .filter(|v| !v.trim().is_empty())
            .collect()
    };
    // Datas: a coluna "date/data/fecha" se tiver datas; senão a primeira com datas.
    let date_named = header_like(
        headers,
        &[
            "date",
            "data",
            "fecha",
            "posted",
            "posting",
            "transaction",
            "lancamento",
            "operacion",
            "movimento",
        ],
    );
    let date_col = date_named
        .into_iter()
        .chain(0..n)
        .find(|i| !col(*i).is_empty() && guess_date_format(&col(*i), month_first).is_some())?;
    let date_format = guess_date_format(&col(date_col), month_first)?;
    let numeric = |i: usize| {
        let v = col(i);
        i != date_col && !v.is_empty() && v.iter().all(|x| is_amount(x))
    };
    let debit = header_like(
        headers,
        &[
            "debit",
            "debito",
            "debe",
            "withdrawal",
            "withdrawals",
            "saida",
            "cargo",
        ],
    )
    .filter(|i| (0..n).contains(i));
    let credit = header_like(
        headers,
        &[
            "credit", "credito", "haber", "deposit", "deposits", "entrada", "abono",
        ],
    )
    .filter(|i| (0..n).contains(i));
    let named_amount = header_like(
        headers,
        &["amount", "valor", "importe", "montante", "value", "quantia"],
    )
    .filter(|i| numeric(*i));
    let (amount, debit, credit) = match (named_amount, debit, credit) {
        (Some(a), _, _) => (Some(a), None, None),
        (None, Some(d), Some(c)) if d != c => (None, Some(d), Some(c)),
        // Sem nome: a primeira coluna numérica (o saldo costuma vir depois).
        _ => ((0..n).find(|i| numeric(*i)).map(Some)?, None, None),
    };
    let money_cols: Vec<usize> = [amount, debit, credit].into_iter().flatten().collect();
    let samples: Vec<&str> = money_cols.iter().flat_map(|i| col(*i)).collect();
    let comma = guess_decimal_comma(&samples, decimal_comma);
    let description = header_like(
        headers,
        &[
            "description",
            "descricao",
            "historico",
            "concepto",
            "memo",
            "details",
            "detalhe",
            "payee",
            "descripcion",
            "narrative",
        ],
    )
    .filter(|i| *i != date_col && !money_cols.contains(i))
    .or_else(|| {
        // A coluna de texto mais longa.
        (0..n)
            .filter(|i| *i != date_col && !money_cols.contains(i) && !numeric(*i))
            .max_by_key(|i| col(*i).iter().map(|v| v.len()).sum::<usize>())
    })?;
    Some(CsvMapping {
        delimiter: None,
        has_header: !headers.is_empty(),
        skip_rows: None,
        date_column: date_col.to_string(),
        date_format,
        amount_column: amount.map(|a| a.to_string()),
        debit_column: debit.map(|d| d.to_string()),
        credit_column: credit.map(|c| c.to_string()),
        description_column: description.to_string(),
        payee_column: None,
        external_id_column: None,
        decimal_comma: comma,
        invert_sign: false,
        encoding: None,
        card_descriptor: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_delimiters() {
        assert_eq!(
            detect_delimiter(
                "Data;Descrição;Valor\n01/10/2026;Padaria, centro;-12,50\n02/10/2026;Mercado;-1.234,56\n"
            ),
            b';'
        );
        assert_eq!(
            detect_delimiter(
                "date,title,amount\n2026-10-01,Padaria,12.50\n2026-10-02,\"Loja; X\",3.00\n"
            ),
            b','
        );
        assert_eq!(detect_delimiter("a\tb\tc\n1\t2\t3\n"), b'\t');
    }

    #[test]
    fn layout_skips_preamble() {
        let t = "Extrato Conta Corrente\nConta ;12345678\nPeríodo ;01/09/2026 a 30/09/2026\n\nData Lançamento;Histórico;Descrição;Valor;Saldo\n01/09/2026;Pix enviado;Fulano;-50,00;1.184,56\n";
        let l = detect_layout(t, b';').unwrap();
        assert_eq!(l.skip_rows, 4);
        assert!(l.has_header);
        assert_eq!(l.headers[0], "Data Lançamento");
        let l = detect_layout("01/10/2026;Padaria;-1,00\n02/10/2026;X;2,00\n", b';').unwrap();
        assert!(!l.has_header);
        assert_eq!(l.headers, vec!["Coluna 1", "Coluna 2", "Coluna 3"]);
    }

    #[test]
    fn columns_by_name_index_and_join() {
        let h: Vec<String> = ["Data", "Histórico", "Descrição", "Valor (R$)"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(resolve_columns("descricao", Some(&h)).unwrap(), vec![2]);
        assert_eq!(resolve_columns("VALOR (R$)", Some(&h)).unwrap(), vec![3]);
        assert_eq!(resolve_columns("0", Some(&h)).unwrap(), vec![0]);
        assert_eq!(
            resolve_columns("Histórico+Descrição", Some(&h)).unwrap(),
            vec![1, 2]
        );
        assert!(resolve_columns("Nada", Some(&h)).is_err());
        assert!(resolve_columns("Nada", None).is_err());
    }

    #[test]
    fn dates_and_amounts() {
        assert_eq!(
            parse_date("01/10/2026 14:33", "%d/%m/%Y").map(|d| d.to_string()),
            Some("2026-10-01".into())
        );
        assert_eq!(
            parse_date("01/10/26", "%d/%m/%Y|%d/%m/%y").map(|d| d.to_string()),
            Some("2026-10-01".into())
        );
        assert_eq!(
            parse_date("2026-10-01", "%d/%m/%Y").map(|d| d.to_string()),
            Some("2026-10-01".into())
        );
        assert_eq!(parse_date("31/02/2026", "%d/%m/%Y"), None);
        assert_eq!(
            parse_cell_amount("-1.234,56", true, "BRL").unwrap(),
            -123456
        );
        assert_eq!(parse_cell_amount("R$ 10,00", true, "BRL").unwrap(), 1000);
        assert_eq!(parse_cell_amount("1,234.56", false, "BRL").unwrap(), 123456);
        assert_eq!(parse_cell_amount("-12.5", false, "BRL").unwrap(), -1250);
        assert_eq!(parse_cell_amount("45", false, "BRL").unwrap(), 4500);
        assert!(parse_cell_amount("1.234", false, "BRL").is_err());
    }

    #[test]
    fn debit_credit_columns_and_invert() {
        let t = "Data;Desc;Débito;Crédito\n01/10/2026;Padaria;12,50;\n02/10/2026;Salário;;5.000,00\n03/10/2026;Vazio;;\n";
        let mut m = mapping(
            None,
            ("Data", "%d/%m/%Y"),
            None,
            Some(("Débito", "Crédito")),
            "Desc",
            None,
            true,
            false,
        );
        let p = parse(t, &m, "BRL").unwrap();
        assert_eq!(p.delimiter, b';');
        let amounts: Vec<Option<i64>> = p.rows.iter().map(|r| r.amount).collect();
        assert_eq!(amounts, vec![Some(-1250), Some(500000), None]);
        assert!(p.rows[2].error.is_some());
        m.invert_sign = true;
        let p = parse(t, &m, "BRL").unwrap();
        assert_eq!(p.rows[0].amount, Some(1250));
    }

    #[test]
    fn preset_suggestion_by_header() {
        let h = |s: &str| s.split(',').map(str::to_string).collect::<Vec<_>>();
        assert_eq!(
            suggest_preset(&h("Data,Valor,Identificador,Descrição")).as_deref(),
            Some("nubank_conta")
        );
        assert_eq!(
            suggest_preset(&h("date,title,amount")).as_deref(),
            Some("nubank_cartao")
        );
        assert_eq!(
            suggest_preset(&h("Data Lançamento,Histórico,Descrição,Valor,Saldo")).as_deref(),
            Some("inter")
        );
        assert_eq!(
            suggest_preset(&h("Data Lançamento,Data Contábil,Título,Descrição,Entrada(R$),Saída(R$),Saldo do Dia(R$)")).as_deref(),
            Some("c6")
        );
        assert_eq!(suggest_preset(&h("foo,bar")), None);
        assert!(preset("generico").is_ok());
        assert!(preset("nao_existe").is_err());
    }

    #[test]
    fn guesses_us_and_european_bank_csvs() {
        let h = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let r = |v: &[&[&str]]| {
            v.iter()
                .map(|row| row.iter().map(|s| s.to_string()).collect())
                .collect::<Vec<Vec<String>>>()
        };
        // Banco americano inventado: mês antes do dia, ponto decimal, saldo depois do valor.
        let m = guess_mapping(
            &h(&["Posting Date", "Description", "Amount", "Type", "Balance"]),
            &r(&[
                &[
                    "10/03/2026",
                    "COFFEE SHOP EXAMPLE",
                    "-4.50",
                    "DEBIT",
                    "1,234.56",
                ],
                &[
                    "10/15/2026",
                    "PAYROLL EXAMPLE INC",
                    "2,500.00",
                    "CREDIT",
                    "3,730.06",
                ],
            ]),
            true,
            false,
        )
        .unwrap();
        assert_eq!(m.date_column, "0");
        assert_eq!(m.date_format, "%m/%d/%Y");
        assert_eq!(m.amount_column.as_deref(), Some("2"));
        assert_eq!(m.description_column, "1");
        assert!(!m.decimal_comma);
        // Ambíguo (dias até 12): o padrão do país decide.
        let amb = r(&[&["03/04/2026", "X", "1.00"]]);
        assert_eq!(
            guess_mapping(&h(&["Date", "Memo", "Amount"]), &amb, false, false)
                .unwrap()
                .date_format,
            "%d/%m/%Y"
        );
        // Banco europeu inventado: débito e crédito separados, vírgula decimal.
        let m = guess_mapping(
            &h(&["Fecha", "Concepto", "Debe", "Haber", "Saldo"]),
            &r(&[
                &[
                    "15/10/2026",
                    "SUPERMERCADO EJEMPLO",
                    "23,40",
                    "",
                    "1.200,00",
                ],
                &["16/10/2026", "NOMINA EJEMPLO", "", "1.850,00", "3.050,00"],
            ]),
            false,
            true,
        )
        .unwrap();
        assert_eq!(m.date_format, "%d/%m/%Y");
        assert_eq!(m.debit_column.as_deref(), Some("2"));
        assert_eq!(m.credit_column.as_deref(), Some("3"));
        assert!(m.decimal_comma);
        assert_eq!(
            guess_date_format(&["2026-10-01"], true).as_deref(),
            Some("%Y-%m-%d")
        );
        assert_eq!(
            parse_cell_amount("1,234.56", false, "USD").unwrap(),
            123_456
        );
    }
}
