//! Exportação de dados em formato aberto (JSON completo e CSV de lançamentos).
//! Os arquivos saem **sem criptografia**; a interface avisa antes. Ficam com permissão
//! 0600 e nunca levam segredos (senha de backup, chaves de API).
//!
//! JSON (`format: "treasury-export"`, `version: 1`):
//!
//! ```json
//! {
//!   "format": "treasury-export", "version": 1, "schema_version": 20,
//!   "exported_at": "2026-10-14T12:00:00.000Z", "app_version": "0.1.0",
//!   "about": { "money": "...", "dates": "...", ... },
//!   "blob_columns": { "attachment": ["data"] },
//!   "excluded_tables": ["backup_record", "change_item", ...],
//!   "tables": { "account": [ { "id": "...", "name": "...", ... } ], ... }
//! }
//! ```
//!
//! Cada tabela de domínio vira uma lista de objetos coluna → valor, com os mesmos nomes do
//! esquema (ver `docs/architecture.md`). Colunas binárias vão em base64 padrão. Os textos do
//! `about` e o CSV (cabeçalhos, tipos, situação e formato de números e datas) seguem o idioma
//! da instalação; os nomes das chaves e das colunas do JSON não mudam.

pub mod api;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use rusqlite::Connection;
use rusqlite::types::ValueRef;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{self, Locale, args, t, t_args};
use crate::paths::TreasuryPaths;
use crate::transactions::{TxnFilter, TxnKind, TxnSort, TxnStatus, TxnView};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExportArgs {
    /// Caminho de destino escolhido pelo usuário (diálogo de salvar).
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExportCsvArgs {
    pub path: String,
    #[ts(optional)]
    pub filter: Option<crate::transactions::TxnFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExportResult {
    pub path: String,
    #[ts(type = "number")]
    pub bytes: u64,
    /// Linhas por tabela (JSON) ou total de linhas (CSV, chave `txn`).
    #[ts(type = "Record<string, number>")]
    pub counts: std::collections::BTreeMap<String, u64>,
}

pub const FORMAT: &str = "treasury-export";
pub const FORMAT_VERSION: u32 = 1;

/// Tabelas técnicas que não entram na exportação (histórico com blobs de changeset,
/// registro de backups, controle de migrações, segredos).
const EXCLUDED_TABLES: &[&str] = &[
    "change_set",
    "change_item",
    "backup_record",
    "refinery_schema_history",
    "secret",
    // Fila do e-mail de ingestão: cópias das mensagens e dos anexos (o que foi importado já
    // está nas tabelas da família) e o controle técnico das buscas.
    "mail_message",
    "mail_part",
    "mail_folder_state",
    "mail_seen",
];

fn is_excluded_table(name: &str) -> bool {
    name.starts_with("sqlite_") || EXCLUDED_TABLES.contains(&name) || name.contains("secret")
}

/// Ajuste que é (ou pode ser) segredo: nunca sai do cofre.
pub fn is_secret_setting(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    k.starts_with("backup.separate_key")
        || [
            "secret",
            "password",
            "senha",
            "token",
            "api_key",
            "apikey",
            "private",
            "credential",
        ]
        .iter()
        .any(|w| k.contains(w))
        || k.ends_with("_key")
        || k.ends_with(".key")
        || k == "key"
}

/// Destino válido: caminho absoluto, pasta existente, fora das pastas do Treasury.
pub(crate) fn check_destination(path: &str, paths: &TreasuryPaths) -> Result<PathBuf> {
    let p = PathBuf::from(path.trim());
    if !p.is_absolute() {
        return Err(Error::invalid(t("nucleo-geral-erro-exportar-caminho")));
    }
    let Some(name) = p.file_name() else {
        return Err(Error::invalid(t("nucleo-geral-erro-exportar-nome")));
    };
    let parent = p
        .parent()
        .ok_or_else(|| Error::invalid(t("nucleo-geral-erro-exportar-pasta-destino")))?;
    let parent = parent.canonicalize().map_err(|_| {
        Error::invalid(t_args(
            "nucleo-geral-erro-exportar-pasta-inexistente",
            &args(&[("pasta", parent.display().to_string().into())]),
        ))
    })?;
    if p.is_dir() {
        return Err(Error::invalid(t(
            "nucleo-geral-erro-exportar-destino-pasta",
        )));
    }
    for guarded in [
        &paths.root,
        &paths.backups_dir,
        &paths.data_dir().to_path_buf(),
    ] {
        if let Ok(g) = guarded.canonicalize()
            && parent.starts_with(&g)
        {
            return Err(Error::invalid(t(
                "nucleo-geral-erro-exportar-pasta-treasury",
            )));
        }
    }
    Ok(parent.join(name))
}

/// Grava num temporário 0600 na mesma pasta e renomeia (nunca deixa arquivo pela metade).
pub(crate) fn write_private(
    path: &Path,
    f: impl FnOnce(&mut BufWriter<File>) -> Result<()>,
) -> Result<u64> {
    let dir = path
        .parent()
        .ok_or_else(|| Error::invalid(t("nucleo-geral-erro-exportar-pasta")))?;
    let tmp = dir.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("export"),
        crate::ids::new_id()
    ));
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let file = opts.open(&tmp)?;
    let result = (|| {
        let mut w = BufWriter::new(file);
        f(&mut w)?;
        w.flush()?;
        w.get_ref().sync_all()?;
        Ok::<_, Error>(())
    })();
    if let Err(e) = result {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    crate::paths::set_mode(path, 0o600)?;
    Ok(std::fs::metadata(path)?.len())
}

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn value_json(v: ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Value::String(base64::engine::general_purpose::STANDARD.encode(b)),
    }
}

fn tables(conn: &Connection) -> Result<(Vec<String>, Vec<String>)> {
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
    let all: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(all.into_iter().partition(|t| !is_excluded_table(t)))
}

fn blob_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT name FROM pragma_table_info(?1) WHERE upper(type) = 'BLOB'")?;
    let cols = stmt
        .query_map([table], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(cols)
}

/// `export.json`: todas as tabelas de domínio num JSON aberto e documentado.
pub fn export_json(
    conn: &Connection,
    schema_version: i32,
    path: &str,
    paths: &TreasuryPaths,
) -> Result<ExportResult> {
    let dest = check_destination(path, paths)?;
    let (included, excluded) = tables(conn)?;
    let mut blobs = BTreeMap::new();
    for t in &included {
        let cols = blob_columns(conn, t)?;
        if !cols.is_empty() {
            blobs.insert(t.clone(), cols);
        }
    }
    let mut counts = BTreeMap::new();
    let bytes = write_private(&dest, |w| {
        let head = json!({
            "format": FORMAT,
            "version": FORMAT_VERSION,
            "schema_version": schema_version,
            "exported_at": crate::time::now_utc(),
            "app_version": crate::config::app_version(),
            "about": {
                "description": t("nucleo-geral-exportacao-descricao"),
                "tables": t("nucleo-geral-exportacao-tabelas"),
                "money": t("nucleo-geral-exportacao-dinheiro"),
                "decimals": t("nucleo-geral-exportacao-decimais"),
                "dates": t("nucleo-geral-exportacao-datas"),
                "ids": t("nucleo-geral-exportacao-ids"),
                "json_columns": t("nucleo-geral-exportacao-json"),
                "blobs": t("nucleo-geral-exportacao-binarios"),
                "secrets": t("nucleo-geral-exportacao-segredos")
            },
            "blob_columns": blobs,
            "excluded_tables": excluded,
        });
        let Value::Object(head) = head else {
            unreachable!("objeto literal")
        };
        w.write_all(b"{\n")?;
        for (k, v) in &head {
            serde_json::to_writer(&mut *w, k)?;
            w.write_all(b": ")?;
            serde_json::to_writer(&mut *w, v)?;
            w.write_all(b",\n")?;
        }
        w.write_all(b"\"tables\": {")?;
        for (ti, table) in included.iter().enumerate() {
            if ti > 0 {
                w.write_all(b",")?;
            }
            w.write_all(b"\n  ")?;
            serde_json::to_writer(&mut *w, table)?;
            w.write_all(b": [")?;
            let mut stmt = conn.prepare(&format!("SELECT * FROM {}", quote_ident(table)))?;
            let names: Vec<String> = stmt.column_names().map(|s| s.to_string()).collect();
            let key_col = names.iter().position(|n| n == "key");
            let mut rows = stmt.query([])?;
            let mut n = 0u64;
            while let Some(row) = rows.next()? {
                if table == "setting"
                    && let Some(k) = key_col
                {
                    let key: String = row.get(k)?;
                    if is_secret_setting(&key) {
                        continue;
                    }
                }
                let mut obj = Map::with_capacity(names.len());
                for (i, name) in names.iter().enumerate() {
                    obj.insert(name.clone(), value_json(row.get_ref(i)?));
                }
                w.write_all(if n == 0 { b"\n    " } else { b",\n    " })?;
                serde_json::to_writer(&mut *w, &obj)?;
                n += 1;
            }
            if n > 0 {
                w.write_all(b"\n  ")?;
            }
            w.write_all(b"]")?;
            counts.insert(table.clone(), n);
        }
        w.write_all(b"\n}\n}\n")?;
        Ok(())
    })?;
    Ok(ExportResult {
        path: dest.to_string_lossy().into_owned(),
        bytes,
        counts,
    })
}

fn kind_label(k: TxnKind) -> String {
    t(match k {
        TxnKind::Income => "nucleo-geral-csv-receita",
        TxnKind::Expense => "nucleo-geral-csv-despesa",
        TxnKind::Transfer => "nucleo-geral-csv-transferencia",
        TxnKind::CardPayment => "nucleo-geral-csv-pagamento-fatura",
        TxnKind::Investment => "nucleo-geral-csv-investimento",
        TxnKind::Adjustment => "nucleo-geral-csv-ajuste",
    })
}

fn status_label(s: TxnStatus) -> String {
    t(match s {
        TxnStatus::Planned => "nucleo-geral-csv-previsto",
        TxnStatus::Cleared => "nucleo-geral-csv-efetivado",
        TxnStatus::Reconciled => "nucleo-geral-csv-conciliado",
    })
}

/// Valor com o separador decimal `sep` e sem separador de milhar (`-1234,56`, `-1234.56`).
fn plain_amount(amount: i64, currency: &str, sep: char) -> String {
    let units = crate::money::minor_units(currency);
    let neg = amount < 0;
    let abs = amount.unsigned_abs();
    let div = 10u64.pow(units);
    let sign = if neg { "-" } else { "" };
    if units == 0 {
        format!("{sign}{abs}")
    } else {
        format!(
            "{sign}{}{sep}{:0width$}",
            abs / div,
            abs % div,
            width = units as usize
        )
    }
}

/// Valor com vírgula decimal e sem separador de milhar (`-1234,56`), bom para planilhas.
pub fn decimal_comma(amount: i64, currency: &str) -> String {
    plain_amount(amount, currency, ',')
}

/// Formato da planilha no idioma da instalação: as planilhas em português e espanhol abrem
/// `;` com vírgula decimal e dd/mm/aaaa; as em inglês, `,` com ponto decimal e mm/dd/aaaa.
#[derive(Clone, Copy)]
struct CsvStyle {
    delimiter: u8,
    decimal: char,
    date: &'static str,
}

impl CsvStyle {
    fn of(l: Locale) -> Self {
        match l {
            Locale::En => CsvStyle {
                delimiter: b',',
                decimal: '.',
                date: "%m/%d/%Y",
            },
            _ => CsvStyle {
                delimiter: b';',
                decimal: ',',
                date: "%d/%m/%Y",
            },
        }
    }

    fn amount(self, amount: i64, currency: &str) -> String {
        plain_amount(amount, currency, self.decimal)
    }

    fn date(self, iso: &str) -> String {
        match crate::time::parse_date(iso) {
            Ok(d) => d.format(self.date).to_string(),
            Err(_) => iso.to_string(),
        }
    }
}

/// Texto vindo de extratos pode começar com `=`, `+`, `-`, `@`: planilhas executariam
/// como fórmula. Prefixa com apóstrofo (dado continua legível, nunca vira comando).
fn safe_text(s: &str) -> String {
    if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s.to_string()
    }
}

/// Cabeçalhos do CSV (chaves do catálogo, no idioma da instalação).
const CSV_HEADER: &[&str] = &[
    "nucleo-geral-csv-data",
    "nucleo-geral-csv-conta",
    "nucleo-geral-csv-descricao",
    "nucleo-geral-csv-favorecido",
    "nucleo-geral-csv-categoria",
    "nucleo-geral-csv-titular",
    "nucleo-geral-csv-valor",
    "nucleo-geral-csv-moeda",
    "nucleo-geral-csv-tipo",
    "nucleo-geral-csv-situacao",
    "nucleo-geral-csv-tags",
    "nucleo-geral-csv-notas",
    "nucleo-geral-csv-vencimento-fatura",
    "nucleo-geral-csv-parcela",
    "nucleo-geral-csv-divisao",
    "nucleo-geral-csv-id",
];

fn csv_row(
    v: &TxnView,
    paths: &std::collections::HashMap<String, String>,
    style: CsvStyle,
    no_category: &str,
) -> Vec<String> {
    let parcel = match (v.installment_number, v.installment_total) {
        (Some(n), Some(t)) => format!("{n}/{t}"),
        (Some(n), None) => n.to_string(),
        _ => String::new(),
    };
    let splits = v
        .splits
        .iter()
        .map(|s| {
            let cat = s
                .category_id
                .as_ref()
                .and_then(|c| paths.get(c).cloned())
                .unwrap_or_else(|| no_category.to_string());
            format!("{cat}: {}", style.amount(s.amount, &v.currency))
        })
        .collect::<Vec<_>>()
        .join(" | ");
    vec![
        style.date(&v.date),
        safe_text(&v.account_name),
        safe_text(&v.description),
        safe_text(v.payee.as_deref().unwrap_or("")),
        safe_text(v.category_path.as_deref().unwrap_or("")),
        safe_text(v.member_name.as_deref().unwrap_or("")),
        style.amount(v.amount, &v.currency),
        v.currency.clone(),
        kind_label(v.kind),
        status_label(v.status),
        safe_text(
            &v.tags
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        safe_text(v.notes.as_deref().unwrap_or("")),
        v.statement_due_date
            .as_deref()
            .map(|d| style.date(d))
            .unwrap_or_default(),
        parcel,
        safe_text(&splits),
        v.id.clone(),
    ]
}

/// `export.transactions_csv`: CSV para planilhas (UTF-8 com BOM), no idioma da instalação:
/// colunas, tipos e situação traduzidos; em português e espanhol, `;`, vírgula decimal e
/// datas dd/mm/aaaa; em inglês, `,`, ponto decimal e datas mm/dd/aaaa.
pub fn export_transactions_csv(
    conn: &Connection,
    filter: Option<&TxnFilter>,
    path: &str,
    paths: &TreasuryPaths,
) -> Result<ExportResult> {
    let dest = check_destination(path, paths)?;
    let mut f = filter.cloned().unwrap_or_default();
    f.sort = Some(f.sort.unwrap_or(TxnSort::DateAsc));
    let ids = crate::rules::views::filter_ids(conn, &f)?;
    let cat_paths = crate::rules::views::category_paths(conn)?;
    let mut total = 0u64;
    let style = CsvStyle::of(i18n::current());
    let header: Vec<String> = CSV_HEADER.iter().map(|k| t(k)).collect();
    let no_category = t("nucleo-geral-csv-sem-categoria");
    let bytes = write_private(&dest, |w| {
        w.write_all(b"\xEF\xBB\xBF")?;
        let mut csv = csv::WriterBuilder::new()
            .delimiter(style.delimiter)
            .quote_style(csv::QuoteStyle::Necessary)
            .from_writer(&mut *w);
        let map_csv = |e: csv::Error| {
            Error::Other(t_args(
                "nucleo-geral-erro-csv",
                &args(&[("erro", e.to_string().into())]),
            ))
        };
        csv.write_record(&header).map_err(map_csv)?;
        for chunk in ids.chunks(500) {
            for v in crate::rules::views::load(conn, chunk)? {
                csv.write_record(csv_row(&v, &cat_paths, style, &no_category))
                    .map_err(map_csv)?;
                total += 1;
            }
        }
        csv.flush()?;
        Ok(())
    })?;
    Ok(ExportResult {
        path: dest.to_string_lossy().into_owned(),
        bytes,
        counts: BTreeMap::from([("txn".to_string(), total)]),
    })
}

#[cfg(test)]
mod tests;
