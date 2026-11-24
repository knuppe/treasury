//! Banco de dados (Ajustes › Banco de dados): tamanho do arquivo, maiores tabelas, rotinas de
//! manutenção (verificação de integridade, ANALYZE, otimização, VACUUM) e expurgo do que não
//! é dado da família (histórico antigo de alterações, auditoria, conversas do agente, caches
//! de mercado e da análise de ativos).
//!
//! Regras do expurgo: só pela interface do app, sempre com prévia (`database.purge_preview`)
//! e com backup verificado antes de apagar. Lançamentos, contas, carteira, contratos e
//! cadastros nunca entram aqui. Propostas pendentes da IA nunca são apagadas.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

pub const READ_ONLY: &[&str] = &["info", "purge_preview", "table_rows"];

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TableSize {
    pub name: String,
    #[ts(type = "number")]
    pub rows: i64,
    /// Páginas da tabela e dos índices dela.
    #[ts(type = "number")]
    pub bytes: i64,
    #[ts(type = "number")]
    pub index_bytes: i64,
    /// O que a tabela guarda, em poucas palavras (quando conhecida).
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DatabaseInfo {
    pub path: String,
    /// Tamanho do arquivo do banco (com o WAL, se houver).
    #[ts(type = "number")]
    pub file_bytes: i64,
    #[ts(type = "number")]
    pub page_size: i64,
    #[ts(type = "number")]
    pub pages: i64,
    /// Espaço livre dentro do arquivo (recuperável com VACUUM).
    #[ts(type = "number")]
    pub free_bytes: i64,
    pub sqlite_version: String,
    pub cipher_version: Option<String>,
    /// Última migração aplicada.
    #[ts(type = "number")]
    pub schema_version: i64,
    #[ts(type = "number")]
    pub change_sets: i64,
    pub oldest_change: Option<String>,
    /// Tabelas da maior para a menor.
    pub tables: Vec<TableSize>,
    pub last_maintenance_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PurgeKind {
    /// Conjuntos de alterações já aplicados, desfeitos ou rejeitados (perde o desfazer).
    ChangeHistory,
    /// Registro de auditoria do agente e dos clientes MCP.
    AiAudit,
    /// Conversas do agente não fixadas (e os anexos delas).
    AiConversations,
    /// Cotações intradiárias e cache do Fundamentus (baixados de novo quando preciso).
    MarketCache,
    /// Dados baixados da análise de ativos (CVM, B3, SEC): a próxima coleta baixa tudo.
    ResearchData,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct PurgeArgs {
    pub kind: PurgeKind,
    /// Só o que é mais velho que isso (padrão: 180 dias; ignorado nos dados da análise).
    #[serde(default)]
    #[ts(optional)]
    pub older_than_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PurgePreview {
    pub kind: PurgeKind,
    pub label: String,
    /// O que se perde ao apagar, em uma frase.
    pub consequence: String,
    pub cutoff: Option<String>,
    #[ts(type = "number")]
    pub rows: i64,
    /// Estimativa do espaço liberado (proporcional às linhas apagadas).
    #[ts(type = "number")]
    pub bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PurgeResult {
    pub kind: PurgeKind,
    #[ts(type = "number")]
    pub rows: i64,
    pub backup_id: String,
    #[ts(type = "number")]
    pub file_bytes_before: i64,
    #[ts(type = "number")]
    pub file_bytes_after: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MaintenanceAction {
    IntegrityCheck,
    Analyze,
    Optimize,
    Vacuum,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct MaintenanceArgs {
    pub action: MaintenanceAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MaintenanceResult {
    pub action: MaintenanceAction,
    pub ok: bool,
    /// Mensagens da verificação de integridade ("ok" quando íntegro).
    pub messages: Vec<String>,
    #[ts(type = "number")]
    pub file_bytes_before: i64,
    #[ts(type = "number")]
    pub file_bytes_after: i64,
    #[ts(type = "number")]
    pub millis: i64,
}

const LAST_MAINTENANCE: &str = "database.last_maintenance_at";
const DEFAULT_DAYS: u32 = 180;

/// O que a tabela guarda, em poucas palavras, no idioma da instalação (vazio se desconhecida).
fn group_of(table: &str) -> String {
    let key = match table {
        "change_set" | "change_item" => "nucleo-geral-grupo-historico",
        "txn" | "txn_split" | "txn_tag" | "tag" | "attachment" | "import_batch" => {
            "nucleo-geral-grupo-lancamentos"
        }
        "account" | "account_member" | "credit_card" | "card_statement" | "card_plastic"
        | "account_overdraft" => "nucleo-geral-grupo-contas",
        "trade" | "income_event" | "corporate_action" | "asset" | "fixed_income_terms"
        | "position_snapshot" | "target_allocation" | "watchlist_item" | "dividend_event" => {
            "nucleo-geral-grupo-carteira"
        }
        "price" | "intraday_price" | "market_quote" | "fx_rate" | "index_value"
        | "index_series" | "fundamentus_cache" | "holiday" | "exchange" | "bank_registry"
        | "country_profile" | "country_series" => "nucleo-geral-grupo-cotacoes",
        x if x.starts_with("research_")
            || x.starts_with("bank_") && x != "bank_registry"
            || x == "fundamental"
            || x == "corporate_document" =>
        {
            "nucleo-geral-grupo-analise"
        }
        x if x.starts_with("ai_") || x == "alert" => "nucleo-geral-grupo-agente",
        x if x.starts_with("loan") => "nucleo-geral-grupo-dividas",
        x if x.starts_with("physical_") => "nucleo-geral-grupo-patrimonio",
        _ => return String::new(),
    };
    t(key)
}

fn file_bytes(ctx: &Ctx<'_>) -> i64 {
    let p = &ctx.paths.db_path;
    let size = |p: &std::path::Path| std::fs::metadata(p).map(|m| m.len() as i64).unwrap_or(0);
    let mut wal = p.as_os_str().to_owned();
    wal.push("-wal");
    size(p) + size(std::path::Path::new(&wal))
}

/// O SQLCipher devolve alguns pragmas como texto (`cipher_page_size`).
fn pragma_i64(conn: &Connection, name: &str) -> Result<i64> {
    let v: rusqlite::types::Value = conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))?;
    Ok(match v {
        rusqlite::types::Value::Integer(i) => i,
        rusqlite::types::Value::Text(t) => t.trim().parse().unwrap_or(0),
        _ => 0,
    })
}

pub fn tables(conn: &Connection) -> Result<Vec<TableSize>> {
    let names: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
              AND sql NOT LIKE 'CREATE VIRTUAL%'",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    // Bytes por objeto (tabelas e índices) de uma vez, pela tabela virtual dbstat.
    let mut by_object: std::collections::HashMap<String, i64> = conn
        .prepare("SELECT name, sum(pgsize) FROM dbstat GROUP BY name")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for name in names {
        let rows: i64 = conn.query_row(&format!("SELECT count(*) FROM \"{name}\""), [], |r| {
            r.get(0)
        })?;
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'index' AND tbl_name = ?1")?
            .query_map([&name], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let index_bytes: i64 = indexes
            .iter()
            .filter_map(|i| by_object.remove(i))
            .sum::<i64>()
            + by_object
                .iter()
                .filter(|(k, _)| k.starts_with(&format!("sqlite_autoindex_{name}_")))
                .map(|(_, v)| *v)
                .sum::<i64>();
        let own = by_object.get(&name).copied().unwrap_or(0);
        out.push(TableSize {
            group: group_of(&name),
            name,
            rows,
            bytes: own + index_bytes,
            index_bytes,
        });
    }
    out.sort_by(|a, b| b.bytes.cmp(&a.bytes).then(a.name.cmp(&b.name)));
    Ok(out)
}

pub fn info(ctx: &Ctx<'_>) -> Result<DatabaseInfo> {
    let conn = ctx.conn();
    let page_size = pragma_i64(conn, "page_size")?;
    let pages = pragma_i64(conn, "page_count")?;
    let free = pragma_i64(conn, "freelist_count")? * page_size;
    let cipher: Option<String> = conn
        .query_row("PRAGMA cipher_version", [], |r| r.get(0))
        .ok();
    let (change_sets, oldest): (i64, Option<String>) = conn.query_row(
        "SELECT count(*), min(created_at) FROM change_set",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok(DatabaseInfo {
        path: ctx.paths.db_path.display().to_string(),
        file_bytes: file_bytes(ctx),
        page_size,
        pages,
        free_bytes: free,
        sqlite_version: conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?,
        cipher_version: cipher,
        schema_version: conn
            .query_row(
                "SELECT max(version) FROM refinery_schema_history",
                [],
                |r| r.get::<_, Option<i64>>(0),
            )?
            .unwrap_or(0),
        change_sets,
        oldest_change: oldest,
        tables: tables(conn)?,
        last_maintenance_at: crate::settings::get(conn, LAST_MAINTENANCE)?,
    })
}

/// (chave do rótulo, chave da consequência, tabelas e filtros `(tabela, WHERE)` com `?1` =
/// corte).
fn plan(
    kind: PurgeKind,
) -> (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
) {
    match kind {
        PurgeKind::ChangeHistory => (
            "nucleo-geral-expurgo-historico",
            "nucleo-geral-expurgo-historico-consequencia",
            &[(
                "change_set",
                "status IN ('applied', 'undone', 'rejected') AND created_at < ?1",
            )],
        ),
        PurgeKind::AiAudit => (
            "nucleo-geral-expurgo-auditoria",
            "nucleo-geral-expurgo-auditoria-consequencia",
            &[("ai_audit_log", "created_at < ?1")],
        ),
        PurgeKind::AiConversations => (
            "nucleo-geral-expurgo-conversas",
            "nucleo-geral-expurgo-conversas-consequencia",
            &[("ai_conversation", "pinned = 0 AND updated_at < ?1")],
        ),
        PurgeKind::MarketCache => (
            "nucleo-geral-expurgo-mercado",
            "nucleo-geral-expurgo-mercado-consequencia",
            &[
                ("intraday_price", "ts < CAST(strftime('%s', ?1) AS INTEGER)"),
                ("fundamentus_cache", "fetched_at < ?1"),
            ],
        ),
        PurgeKind::ResearchData => (
            "nucleo-geral-expurgo-analise",
            "nucleo-geral-expurgo-analise-consequencia",
            &[
                ("fundamental", "1"),
                ("research_quote", "1"),
                ("corporate_document", "1"),
                ("research_universe", "1"),
                ("market_score", "1"),
                ("research_file", "1"),
                ("research_filing", "1"),
                ("research_auditor", "1"),
                ("bank_institution", "1"),
                ("bank_prudential", "1"),
                ("bank_ifdata_fetch", "1"),
            ],
        ),
    }
}

fn cutoff_of(a: &PurgeArgs) -> Option<String> {
    (a.kind != PurgeKind::ResearchData).then(|| {
        let days = a.older_than_days.unwrap_or(DEFAULT_DAYS).max(1);
        (chrono::Utc::now() - chrono::Duration::days(days as i64))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    })
}

fn exists(conn: &Connection, table: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1)",
        [table],
        |r| r.get(0),
    )?)
}

pub fn preview(conn: &Connection, a: &PurgeArgs) -> Result<PurgePreview> {
    let (label, consequence, targets) = plan(a.kind);
    let cutoff = cutoff_of(a);
    let sizes = tables(conn)?;
    let mut rows = 0i64;
    let mut bytes = 0f64;
    for (table, filter) in targets {
        if !exists(conn, table)? {
            continue;
        }
        let n: i64 = conn.query_row(
            &format!("SELECT count(*) FROM \"{table}\" WHERE {filter}"),
            params![cutoff.as_deref().unwrap_or("")],
            |r| r.get(0),
        )?;
        rows += n;
        if let Some(t) = sizes.iter().find(|t| t.name == *table)
            && t.rows > 0
        {
            bytes += t.bytes as f64 * n as f64 / t.rows as f64;
        }
    }
    Ok(PurgePreview {
        kind: a.kind,
        label: t(label),
        consequence: t(consequence),
        cutoff,
        rows,
        bytes: bytes.round() as i64,
    })
}

fn purge(ctx: &mut Ctx<'_>, a: &PurgeArgs) -> Result<PurgeResult> {
    let before = file_bytes(ctx);
    let rec = crate::backup::create(
        ctx.db,
        &ctx.paths.backups_dir,
        crate::backup::BackupKind::Manual,
    )?;
    if !rec.verified {
        return Err(Error::Other(t_args(
            "nucleo-geral-erro-expurgo-backup",
            &args(&[("erro", rec.verify_error.unwrap_or_default().into())]),
        )));
    }
    let (_, _, targets) = plan(a.kind);
    let cutoff = cutoff_of(a);
    let conn = ctx.conn();
    let tx = conn.unchecked_transaction()?;
    let mut rows = 0i64;
    for (table, filter) in targets {
        if !exists(&tx, table)? {
            continue;
        }
        rows += tx.execute(
            &format!("DELETE FROM \"{table}\" WHERE {filter}"),
            params![cutoff.as_deref().unwrap_or("")],
        )? as i64;
    }
    if a.kind == PurgeKind::ResearchData {
        tx.execute(
            "DELETE FROM setting WHERE key IN (?1, ?2)",
            params![
                crate::research::config::LAST_SYNC_SETTING,
                crate::research::config::LAST_REPORT_SETTING
            ],
        )?;
    }
    tx.commit()?;
    crate::ai::history::vacuum(conn)?;
    crate::settings::set(conn, LAST_MAINTENANCE, &crate::time::now_utc())?;
    Ok(PurgeResult {
        kind: a.kind,
        rows,
        backup_id: rec.id,
        file_bytes_before: before,
        file_bytes_after: file_bytes(ctx),
    })
}

fn run(ctx: &Ctx<'_>, action: MaintenanceAction) -> Result<MaintenanceResult> {
    let started = std::time::Instant::now();
    let before = file_bytes(ctx);
    let conn = ctx.conn();
    let mut messages = vec![];
    let mut ok = true;
    match action {
        MaintenanceAction::IntegrityCheck => {
            messages = conn
                .prepare("PRAGMA integrity_check")?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            ok = messages.len() == 1 && messages[0] == "ok";
            let fk: i64 =
                conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
                    r.get(0)
                })?;
            if fk > 0 {
                ok = false;
                messages.push(t_args(
                    "nucleo-geral-referencias-quebradas",
                    &args(&[("n", fk.into())]),
                ));
            }
        }
        MaintenanceAction::Analyze => conn.execute_batch("ANALYZE")?,
        MaintenanceAction::Optimize => conn.execute_batch("PRAGMA optimize")?,
        MaintenanceAction::Vacuum => crate::ai::history::vacuum(conn)?,
    }
    crate::settings::set(conn, LAST_MAINTENANCE, &crate::time::now_utc())?;
    Ok(MaintenanceResult {
        action,
        ok,
        messages,
        file_bytes_before: before,
        file_bytes_after: file_bytes(ctx),
        millis: started.elapsed().as_millis() as i64,
    })
}

// ---------------------------------------------------------------------------
// Linhas de uma tabela (Ajustes › Banco de dados, clique duplo numa tabela)

/// Linhas mostradas, no máximo.
pub const TABLE_ROWS_MAX: u32 = 100;
const TEXT_MAX: usize = 300;
const MASK: &str = "•••••• (oculto)";

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TableRowsArgs {
    pub table: String,
    /// Padrão e máximo: 100.
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TableColumn {
    pub name: String,
    /// Tipo declarado (`TEXT`, `INTEGER`…).
    pub decl_type: String,
    pub primary_key: bool,
    /// Valores ocultos (chaves, tokens, segredos cifrados).
    pub masked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TableRows {
    pub table: String,
    pub group: String,
    pub columns: Vec<TableColumn>,
    /// Valores já prontos para mostrar: texto (cortado em 300 caracteres), número, nulo,
    /// `<blob N bytes>` ou o valor oculto.
    #[ts(type = "Array<Array<string | number | null>>")]
    pub rows: Vec<Vec<serde_json::Value>>,
    #[ts(type = "number")]
    pub total_rows: i64,
    /// Ordem das linhas (`rowid` = ordem de gravação).
    pub order: String,
    pub notes: Vec<String>,
}

/// Colunas que guardam segredo pelo nome.
fn secret_column(table: &str, col: &str) -> bool {
    let c = col.to_ascii_lowercase();
    (table == "secret" && c != "id" && c != "hint" && !c.ends_with("_at"))
        || [
            "ciphertext",
            "nonce",
            "key_hash",
            "api_key",
            "password",
            "secret",
            "token",
            "wrapped_key",
            "salt",
        ]
        .iter()
        .any(|k| c == *k || c.ends_with(&format!("_{k}")))
}

/// Ajustes cujo valor é segredo (tokens de fontes, chave do backup separado…).
fn secret_setting(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    [
        "token",
        "api_key",
        "secret",
        "password",
        "separate_key",
        "unlock",
        "pin",
    ]
    .iter()
    .any(|w| k.contains(w))
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

pub fn table_rows(conn: &Connection, a: &TableRowsArgs) -> Result<TableRows> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1
                         AND name NOT LIKE 'sqlite_%')",
        [&a.table],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(Error::NotFound(t_args(
            "nucleo-geral-tabela",
            &args(&[("tabela", a.table.clone().into())]),
        )));
    }
    let limit = a.limit.unwrap_or(TABLE_ROWS_MAX).clamp(1, TABLE_ROWS_MAX);
    let table = quote_ident(&a.table);
    let mut columns = vec![];
    {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(5)?,
            ))
        })?;
        for r in rows {
            let (name, decl, pk) = r?;
            columns.push(TableColumn {
                masked: secret_column(&a.table, &name),
                name,
                decl_type: decl,
                primary_key: pk > 0,
            });
        }
    }
    let without_rowid: bool = conn
        .query_row(
            "SELECT sql LIKE '%WITHOUT ROWID%' FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [&a.table],
            |r| r.get(0),
        )
        .unwrap_or(false);
    let order = if without_rowid {
        t("nucleo-geral-ordem-chave-primaria")
    } else {
        "rowid".to_string()
    };
    let sql = format!(
        "SELECT * FROM {table} {} LIMIT {limit}",
        if without_rowid { "" } else { "ORDER BY rowid" }
    );
    let setting_key = (a.table == "setting")
        .then(|| columns.iter().position(|c| c.name == "key"))
        .flatten();
    let mut stmt = conn.prepare(&sql)?;
    let n = columns.len();
    let mut rows = vec![];
    let mut q = stmt.query([])?;
    let mut masked_settings = 0;
    while let Some(r) = q.next()? {
        let mut out = Vec::with_capacity(n);
        let hide_row = match setting_key {
            Some(i) => r.get::<_, String>(i).is_ok_and(|k| secret_setting(&k)),
            None => false,
        };
        if hide_row {
            masked_settings += 1;
        }
        for (i, c) in columns.iter().enumerate() {
            use rusqlite::types::ValueRef;
            let v = r.get_ref(i)?;
            let hide = c.masked || (hide_row && c.name == "value");
            out.push(match v {
                ValueRef::Null => serde_json::Value::Null,
                _ if hide => MASK.into(),
                ValueRef::Integer(x) => x.into(),
                ValueRef::Real(x) => serde_json::Number::from_f64(x)
                    .map_or(serde_json::Value::Null, serde_json::Value::Number),
                ValueRef::Text(b) => {
                    let s = String::from_utf8_lossy(b);
                    if s.chars().count() > TEXT_MAX {
                        format!("{}…", s.chars().take(TEXT_MAX).collect::<String>()).into()
                    } else {
                        s.into_owned().into()
                    }
                }
                ValueRef::Blob(b) => format!("<blob {} bytes>", b.len()).into(),
            });
        }
        rows.push(out);
    }
    let total_rows: i64 =
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?;
    let mut notes = vec![];
    if total_rows > i64::from(limit) {
        notes.push(t_args(
            "nucleo-geral-mostrando-linhas",
            &args(&[("limite", limit.into()), ("total", total_rows.into())]),
        ));
    }
    if columns.iter().any(|c| c.masked) || masked_settings > 0 {
        notes.push(t("nucleo-geral-segredos-ocultos"));
    }
    Ok(TableRows {
        group: group_of(&a.table),
        table: a.table.clone(),
        columns,
        rows,
        total_rows,
        order,
        notes,
    })
}

pub fn handle(
    ctx: &mut Ctx<'_>,
    action: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    let user_only = |ctx: &Ctx<'_>| -> Result<()> {
        if *ctx.actor != Actor::User {
            return Err(Error::Forbidden(t("nucleo-geral-erro-banco-so-interface")));
        }
        Ok(())
    };
    match action {
        "info" => to_value(info(ctx)?),
        // Dados da família em bruto: só a interface (nem MCP nem rotinas).
        "table_rows" => {
            user_only(ctx)?;
            to_value(table_rows(ctx.conn(), &parse_args(args)?)?)
        }
        "purge_preview" => to_value(preview(ctx.conn(), &parse_args(args)?)?),
        "purge" => {
            user_only(ctx)?;
            if ctx.db.is_read_only() {
                return Err(Error::Forbidden(t("nucleo-geral-erro-banco-leitura")));
            }
            to_value(purge(ctx, &parse_args(args)?)?)
        }
        "run" => {
            user_only(ctx)?;
            let a: MaintenanceArgs = parse_args(args)?;
            to_value(run(ctx, a.action)?)
        }
        _ => Err(Error::UnknownCommand(format!("database.{action}"))),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    fn api() -> (tempfile::TempDir, Api) {
        let dir = tempfile::tempdir().unwrap();
        let api =
            Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
                .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        (dir, api)
    }

    #[test]
    fn table_rows_shows_the_first_rows_and_hides_secrets() {
        let (_d, api) = api();
        for i in 0..3 {
            api.call(
                "accounts.create",
                json!({"name": format!("Conta {i}"), "kind": "checking", "opening_date": "2026-01-01"}),
            )
            .unwrap();
        }
        let r = api
            .call(
                "database.table_rows",
                json!({"table": "account", "limit": 2}),
            )
            .unwrap();
        assert_eq!(r["rows"].as_array().unwrap().len(), 2);
        assert_eq!(r["total_rows"], 3);
        let cols: Vec<&str> = r["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap())
            .collect();
        let name = cols.iter().position(|c| *c == "name").unwrap();
        assert_eq!(r["rows"][0][name], "Conta 0");
        assert!(r["notes"][0].as_str().unwrap().contains("2 primeiras de 3"));
        // Source token stored in the settings: hidden; the other settings, not.
        api.with_session(|s, _| {
            crate::settings::set(s.db.conn(), "research.brapi_token", "segredo-123")?;
            crate::settings::set(s.db.conn(), "health.settings", "visivel")
        })
        .unwrap();
        let s = api
            .call("database.table_rows", json!({"table": "setting"}))
            .unwrap()
            .to_string();
        assert!(!s.contains("segredo-123"));
        assert!(s.contains("visivel"));
        assert!(s.contains("oculto"));
        // A table that does not exist and SQLite's internal tables: no.
        assert!(
            api.call("database.table_rows", json!({"table": "nada"}))
                .is_err()
        );
        assert!(
            api.call("database.table_rows", json!({"table": "sqlite_master"}))
                .is_err()
        );
        assert!(
            api.call(
                "database.table_rows",
                json!({"table": "x\"; DROP TABLE account; --"})
            )
            .is_err()
        );
    }

    #[test]
    fn info_lists_tables_by_size() {
        let (_d, api) = api();
        let i = api.call("database.info", json!({})).unwrap();
        assert!(i["file_bytes"].as_i64().unwrap() > 0);
        assert_eq!(
            i["schema_version"].as_i64().unwrap(),
            crate::storage::latest_schema_version() as i64
        );
        let tables = i["tables"].as_array().unwrap();
        assert!(
            tables
                .iter()
                .any(|t| t["name"] == "category" && t["rows"].as_i64() > Some(50))
        );
        let sizes: Vec<i64> = tables
            .iter()
            .map(|t| t["bytes"].as_i64().unwrap())
            .collect();
        assert!(sizes.windows(2).all(|w| w[0] >= w[1]), "largest first");
        let cat = tables.iter().find(|t| t["name"] == "txn").unwrap();
        assert_eq!(cat["group"], "lançamentos");
    }

    #[test]
    fn purge_preview_backup_and_delete_old_history_only() {
        let (_d, api) = api();
        // Old change set (applied) and an old pending one, which stays.
        api.with_session(|s, _| {
            let c = s.db.conn();
            c.execute_batch(
                "INSERT INTO change_set (id, created_at, author_kind, reason, status) VALUES
                   ('old', '2020-01-01T00:00:00Z', 'user', 'velho', 'applied'),
                   ('pend', '2020-01-01T00:00:00Z', 'ai', 'pendente', 'pending');",
            )?;
            Ok(())
        })
        .unwrap();
        let p = api
            .call(
                "database.purge_preview",
                json!({"kind": "change_history", "older_than_days": 365}),
            )
            .unwrap();
        assert_eq!(p["rows"], 1);
        let r = api
            .call(
                "database.purge",
                json!({"kind": "change_history", "older_than_days": 365}),
            )
            .unwrap();
        assert_eq!(r["rows"], 1);
        assert!(r["backup_id"].as_str().is_some_and(|b| !b.is_empty()));
        let left: Vec<String> = api
            .with_session(|s, _| {
                Ok(s.db
                    .conn()
                    .prepare("SELECT id FROM change_set WHERE created_at < '2021-01-01'")?
                    .query_map([], |r| r.get(0))?
                    .collect::<rusqlite::Result<_>>()?)
            })
            .unwrap();
        assert_eq!(left, ["pend"]);
        // The vault creation (recent) stays.
        assert!(
            !api.call("changes.list", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn maintenance_actions_and_ai_is_refused() {
        let (_d, api) = api();
        let r = api
            .call("database.run", json!({"action": "integrity_check"}))
            .unwrap();
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["messages"][0], "ok");
        for a in ["analyze", "optimize", "vacuum"] {
            assert_eq!(
                api.call("database.run", json!({"action": a})).unwrap()["ok"],
                true
            );
        }
        let agent = crate::api::Actor::Mcp {
            client: "agente".into(),
            read_only: false,
        };
        assert!(
            api.call_as(&agent, "database.purge", json!({"kind": "ai_audit"}))
                .is_err()
        );
    }
}
