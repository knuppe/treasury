//! Leitura das planilhas em tabelas de células: .xlsx/.xls/.ods (todas as abas, pelo
//! `calamine`) e CSV exportado (`;` ou `,`, vírgula ou ponto decimal, datas dd/mm/aaaa).
//!
//! Números e datas podem vir como células tipadas (Excel) ou como texto (CSV, ou Excel com
//! "1.234,56" digitado como texto). A interpretação do texto fica em [`parse_number`] e
//! [`parse_date_text`]; "-" e vazio significam "sem valor".

use std::io::Cursor;

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::B3FileFormat;
use crate::error::{Error, Result};
use crate::i18n::{self, args, t, t_args};
use crate::import::{csv_file, text};

/// Mensagem do catálogo com um argumento de texto.
fn msg(key: &str, name: &'static str, value: String) -> String {
    t_args(key, &args(&[(name, value.into())]))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Empty,
    Text(String),
    Num(Decimal),
    Date(NaiveDate),
}

impl Cell {
    /// Vazio ou só um traço (a B3 usa "-" para "não se aplica").
    pub fn is_blank(&self) -> bool {
        match self {
            Cell::Empty => true,
            Cell::Text(s) => is_blank_text(s),
            _ => false,
        }
    }

    /// Texto para exibição e comparação.
    pub fn text(&self) -> String {
        match self {
            Cell::Empty => String::new(),
            Cell::Text(s) => s.clone(),
            Cell::Num(n) => n.normalize().to_string(),
            Cell::Date(d) => d.format("%d/%m/%Y").to_string(),
        }
    }

    /// Número da célula. `Ok(None)` = sem valor; `Err` = texto que não é número.
    pub fn number(
        &self,
        decimal_comma: Option<bool>,
    ) -> std::result::Result<Option<Decimal>, String> {
        match self {
            Cell::Num(n) => Ok(Some(*n)),
            Cell::Empty => Ok(None),
            Cell::Text(s) if is_blank_text(s) => Ok(None),
            Cell::Text(s) => parse_number(s, decimal_comma)
                .map(Some)
                .ok_or_else(|| msg("nucleo-importacao-b3-celula-numero", "valor", s.clone())),
            Cell::Date(d) => Err(msg(
                "nucleo-importacao-b3-celula-data-no-numero",
                "data",
                i18n::date(i18n::current(), *d),
            )),
        }
    }

    /// Data da célula. `Ok(None)` = sem valor.
    pub fn date(&self) -> std::result::Result<Option<NaiveDate>, String> {
        match self {
            Cell::Date(d) => Ok(Some(*d)),
            Cell::Empty => Ok(None),
            Cell::Text(s) if is_blank_text(s) => Ok(None),
            Cell::Text(s) => parse_date_text(s)
                .map(Some)
                .ok_or_else(|| msg("nucleo-importacao-erro-data", "valor", s.clone())),
            // Data gravada como número de série do Excel sem formato de data.
            Cell::Num(n) => {
                let serial = n.trunc().to_string().parse::<i64>().unwrap_or(0);
                if (20_000..80_000).contains(&serial) {
                    Ok(excel_serial_date(serial))
                } else {
                    Err(msg("nucleo-importacao-erro-data", "valor", n.to_string()))
                }
            }
        }
    }
}

fn is_blank_text(s: &str) -> bool {
    let t = s.trim();
    t.is_empty() || t.chars().all(|c| c == '-' || c == '–' || c == '—')
}

fn excel_serial_date(serial: i64) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(1899, 12, 30)?.checked_add_signed(chrono::Duration::days(serial))
}

/// Linha da planilha.
#[derive(Debug, Clone)]
pub struct SheetRow {
    /// Linha física (1-based) no arquivo ou na aba.
    pub line: u32,
    pub cells: Vec<Cell>,
}

/// Uma aba (no CSV, o arquivo inteiro).
#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub rows: Vec<SheetRow>,
}

#[derive(Debug, Clone)]
pub struct Workbook {
    pub format: B3FileFormat,
    pub tables: Vec<Table>,
    /// CSV: codificação detectada.
    pub encoding: Option<String>,
    /// CSV: o arquivo usa vírgula decimal (`Some(true)`), ponto (`Some(false)`) ou não deu
    /// para saber (`None`, cada valor decide).
    pub decimal_comma: Option<bool>,
}

/// Lê o arquivo: planilha do Excel (pela assinatura) ou CSV.
pub fn read(bytes: &[u8], filename: &str) -> Result<Workbook> {
    if bytes.starts_with(b"%PDF") {
        return Err(Error::invalid(t("nucleo-importacao-b3-erro-pdf")));
    }
    let is_zip = bytes.starts_with(b"PK\x03\x04");
    let is_ole = bytes.starts_with(b"\xD0\xCF\x11\xE0");
    if is_zip || is_ole {
        return read_excel(bytes, filename);
    }
    read_csv(bytes, filename)
}

/// Teto de células por aba (o calamine monta o intervalo denso do canto até a última célula).
const MAX_CELLS: u64 = 2_000_000;
/// Teto do XML descompactado de todas as abas.
const MAX_SHEET_XML: u64 = 128 * 1024 * 1024;

/// Confere, antes de abrir, que as abas de um XLSX/ODS não são gigantes: o tamanho
/// descompactado e a última célula (linha × coluna) de cada aba. Uma planilha com uma célula
/// em A1 e outra em XFD1048576 pediria bilhões de células na memória.
fn check_size(bytes: &[u8], filename: &str) -> Result<()> {
    use std::io::Read;
    let Ok(mut z) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return Ok(());
    };
    let too_big = || {
        Error::invalid(t_args(
            "nucleo-importacao-b3-erro-planilha-grande",
            &args(&[
                ("arquivo", filename.to_string().into()),
                ("max", MAX_CELLS.into()),
            ]),
        ))
    };
    let cell = regex::Regex::new(r#"<c [^>]*?r="([A-Z]{1,3})([0-9]{1,7})""#).expect("regex");
    let mut total = 0u64;
    for i in 0..z.len() {
        let mut f = z
            .by_index(i)
            .map_err(|e| Error::invalid(format!("{filename}: {e}")))?;
        let name = f.name().map(|n| n.to_string()).unwrap_or_default();
        if !(name.starts_with("xl/worksheets/") || name == "content.xml") {
            continue;
        }
        total += f.size();
        if total > MAX_SHEET_XML {
            return Err(too_big());
        }
        if !name.starts_with("xl/worksheets/") {
            continue;
        }
        let mut xml = String::new();
        f.by_ref()
            .take(MAX_SHEET_XML)
            .read_to_string(&mut xml)
            .map_err(|e| Error::invalid(format!("{filename}: {e}")))?;
        let (mut rows, mut cols) = (0u64, 0u64);
        for c in cell.captures_iter(&xml) {
            let col = c[1]
                .bytes()
                .fold(0u64, |a, b| a * 26 + u64::from(b - b'A' + 1));
            let row: u64 = c[2].parse().unwrap_or(0);
            rows = rows.max(row);
            cols = cols.max(col);
            if rows.saturating_mul(cols) > MAX_CELLS {
                return Err(too_big());
            }
        }
    }
    Ok(())
}

fn read_excel(bytes: &[u8], filename: &str) -> Result<Workbook> {
    if bytes.starts_with(b"PK\x03\x04") {
        check_size(bytes, filename)?;
    }
    let mut wb = open_workbook_auto_from_rs(Cursor::new(bytes.to_vec())).map_err(|e| {
        Error::invalid(t_args(
            "nucleo-importacao-b3-erro-abrir",
            &args(&[
                ("arquivo", filename.to_string().into()),
                ("erro", e.to_string().into()),
            ]),
        ))
    })?;
    let mut tables = vec![];
    for name in wb.sheet_names() {
        let range = match wb.worksheet_range(&name) {
            Ok(r) => r,
            Err(e) => {
                return Err(Error::invalid(t_args(
                    "nucleo-importacao-b3-erro-aba",
                    &args(&[
                        ("aba", name.clone().into()),
                        ("arquivo", filename.to_string().into()),
                        ("erro", e.to_string().into()),
                    ]),
                )));
            }
        };
        let first_row = range.start().map(|(r, _)| r).unwrap_or(0);
        let mut rows = vec![];
        for (i, r) in range.rows().enumerate() {
            let cells: Vec<Cell> = r.iter().map(cell_from_data).collect();
            if cells.iter().all(Cell::is_blank) {
                continue;
            }
            rows.push(SheetRow {
                line: first_row + i as u32 + 1,
                cells,
            });
        }
        tables.push(Table { name, rows });
    }
    Ok(Workbook {
        format: B3FileFormat::Xlsx,
        tables,
        encoding: None,
        decimal_comma: None,
    })
}

fn cell_from_data(d: &Data) -> Cell {
    match d {
        Data::Empty | Data::Error(_) => Cell::Empty,
        Data::Int(i) => Cell::Num(Decimal::from(*i)),
        Data::Float(f) => float_cell(*f),
        Data::String(s) => Cell::Text(s.trim().to_string()),
        Data::Bool(b) => Cell::Text(if *b { "VERDADEIRO" } else { "FALSO" }.into()),
        Data::DateTime(dt) if dt.is_datetime() => {
            let (y, m, d, ..) = dt.to_ymd_hms_milli();
            NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
                .map(Cell::Date)
                .unwrap_or(Cell::Empty)
        }
        Data::DateTime(dt) => float_cell(dt.as_f64()),
        Data::DateTimeIso(s) => s
            .get(..10)
            .and_then(|p| NaiveDate::parse_from_str(p, "%Y-%m-%d").ok())
            .map(Cell::Date)
            .unwrap_or_else(|| Cell::Text(s.clone())),
        Data::DurationIso(s) => Cell::Text(s.clone()),
    }
}

/// `f64` do Excel → decimal pela representação mais curta (30.45, não 30.449999…).
fn float_cell(f: f64) -> Cell {
    if !f.is_finite() {
        return Cell::Empty;
    }
    format!("{f}")
        .parse::<Decimal>()
        .ok()
        .or_else(|| rust_decimal::prelude::FromPrimitive::from_f64(f))
        .map(Cell::Num)
        .unwrap_or(Cell::Empty)
}

fn read_csv(bytes: &[u8], filename: &str) -> Result<Workbook> {
    let dec = text::decode(bytes, None, None)?;
    let delim = csv_file::detect_delimiter(&dec.text);
    let recs = csv_file::read_records(&dec.text, delim)?;
    if recs.is_empty() {
        return Err(Error::invalid(msg(
            "nucleo-importacao-b3-erro-sem-linhas",
            "arquivo",
            filename.to_string(),
        )));
    }
    let rows: Vec<SheetRow> = recs
        .into_iter()
        .map(|r| SheetRow {
            line: r.line,
            cells: r
                .cells
                .into_iter()
                .map(|c| {
                    if c.is_empty() {
                        Cell::Empty
                    } else {
                        Cell::Text(c)
                    }
                })
                .collect(),
        })
        .collect();
    let decimal_comma = detect_decimal_comma(&rows);
    let name = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .to_string();
    Ok(Workbook {
        format: B3FileFormat::Csv,
        tables: vec![Table { name, rows }],
        encoding: Some(dec.encoding),
        decimal_comma,
    })
}

/// Vírgula ou ponto decimal no arquivo inteiro, por votos. "1.234" (milhar no formato
/// brasileiro) não vota.
fn detect_decimal_comma(rows: &[SheetRow]) -> Option<bool> {
    let (mut comma, mut dot) = (0usize, 0usize);
    for r in rows.iter().take(500) {
        for c in &r.cells {
            let Cell::Text(s) = c else { continue };
            let t = clean_number_text(s);
            if t.is_empty() || !t.chars().any(|c| c.is_ascii_digit()) {
                continue;
            }
            if !t
                .chars()
                .all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == '-')
            {
                continue;
            }
            // Valores ambíguos ("1.234", "0,068") não votam.
            match (t.contains(','), t.contains('.')) {
                (true, true) if t.rfind(',') > t.rfind('.') => comma += 1,
                (true, true) => dot += 1,
                (true, false) if !looks_like_thousands(&t, ',') => comma += 1,
                (false, true) if !looks_like_thousands(&t, '.') => dot += 1,
                _ => {}
            }
        }
    }
    match (comma, dot) {
        (0, 0) => None,
        (c, d) => Some(c >= d),
    }
}

/// "1.234" ou "12.345.678": grupos de três dígitos, sem parte decimal.
fn looks_like_thousands(t: &str, sep: char) -> bool {
    let t = t.trim_start_matches('-');
    let mut parts = t.split(sep);
    let first = parts.next().unwrap_or("");
    let rest: Vec<&str> = parts.collect();
    !rest.is_empty()
        && (1..=3).contains(&first.len())
        && first.chars().all(|c| c.is_ascii_digit())
        && rest
            .iter()
            .all(|p| p.len() == 3 && p.chars().all(|c| c.is_ascii_digit()))
}

fn clean_number_text(s: &str) -> String {
    s.trim()
        .trim_start_matches("R$")
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}')
        .collect()
}

/// Número escrito como texto: "1.234,56", "1234,56", "30.45", "1,234.56", "R$ 10,00",
/// "-5". `decimal_comma` é a convenção do arquivo, quando conhecida; sem ela, vírgula
/// sozinha é decimal (padrão brasileiro) e ponto em grupos de três é milhar.
pub fn parse_number(s: &str, decimal_comma: Option<bool>) -> Option<Decimal> {
    let mut t = clean_number_text(s);
    if t.is_empty() {
        return None;
    }
    let negative = if t.starts_with('(') && t.ends_with(')') {
        t = t[1..t.len() - 1].to_string();
        true
    } else if let Some(rest) = t.strip_prefix('-') {
        t = rest.to_string();
        true
    } else {
        false
    };
    if !t.chars().any(|c| c.is_ascii_digit())
        || !t
            .chars()
            .all(|c| c.is_ascii_digit() || c == ',' || c == '.')
    {
        return None;
    }
    let (has_c, has_d) = (t.contains(','), t.contains('.'));
    let normalized = match (has_c, has_d) {
        // Os dois: o último é o separador decimal.
        (true, true) => {
            if t.rfind(',') > t.rfind('.') {
                t.replace('.', "").replace(',', ".")
            } else {
                t.replace(',', "")
            }
        }
        // Só vírgula: decimal, exceto "1,234" num arquivo de ponto decimal.
        (true, false) => {
            if decimal_comma == Some(false) && looks_like_thousands(&t, ',') {
                t.replace(',', "")
            } else {
                t.replace(',', ".")
            }
        }
        // Só ponto: decimal, exceto "1.234" (milhar) fora de um arquivo de ponto decimal.
        (false, true) => {
            if decimal_comma != Some(false) && looks_like_thousands(&t, '.') {
                t.replace('.', "")
            } else {
                t
            }
        }
        (false, false) => t,
    };
    let v: Decimal = normalized.parse().ok()?;
    Some(if negative { -v } else { v })
}

/// Data em texto: dd/mm/aaaa (padrão da B3), dd/mm/aa, aaaa-mm-dd, dd-mm-aaaa, dd.mm.aaaa;
/// ignora a hora, se vier.
pub fn parse_date_text(s: &str) -> Option<NaiveDate> {
    let t = s.trim();
    let t = t.split_whitespace().next().unwrap_or(t);
    let t = t.split('T').next().unwrap_or(t);
    for f in ["%d/%m/%Y", "%Y-%m-%d", "%d-%m-%Y", "%d.%m.%Y", "%d/%m/%y"] {
        if let Ok(d) = NaiveDate::parse_from_str(t, f) {
            return Some(d);
        }
    }
    None
}

#[cfg(test)]
mod size_tests {
    use std::io::Write;

    use super::*;

    fn xlsx_with(sheet: &str) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let o = zip::write::SimpleFileOptions::default();
            z.start_file("xl/worksheets/sheet1.xml", o).unwrap();
            z.write_all(sheet.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn giant_sparse_sheets_are_refused_before_parsing() {
        let bomb = xlsx_with(
            r#"<sheetData><row r="1"><c r="A1"/></row><row r="1048576"><c r="XFD1048576"/></row></sheetData>"#,
        );
        let e = check_size(&bomb, "x.xlsx").unwrap_err().to_string();
        assert!(e.contains("grande demais"), "{e}");
        let ok = xlsx_with(r#"<sheetData><row r="1"><c r="A1"/><c r="T5000"/></row></sheetData>"#);
        assert!(check_size(&ok, "y.xlsx").is_ok());
    }
}
