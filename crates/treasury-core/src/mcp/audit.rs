//! Log de auditoria das ferramentas usadas pela IA (`ai_audit_log`, fora do histórico de
//! alterações). Toda chamada de ferramenta MCP grava uma linha: cliente, ferramenta,
//! parâmetros resumidos, resultado e duração.
//!
//! Quando o banco não pode ser escrito (cofre bloqueado, ou `treasury mcp` com o app
//! fechado, que abre o banco só para leitura), a entrada vai para
//! `run/mcp-audit-pending.jsonl` **sem os parâmetros** (nada de dados financeiros em texto
//! puro) e é importada na próxima gravação com o banco aberto para escrita.

use std::io::Write;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use ts_rs::TS;

use crate::api::Api;
use crate::error::Result;
use crate::ids::{Id, new_id};
use crate::time::now_utc;

pub const PENDING_FILE: &str = "mcp-audit-pending.jsonl";

const MAX_STRING: usize = 80;
const MAX_ARRAY: usize = 10;
const MAX_DEPTH: usize = 3;
const MAX_PARAMS_BYTES: usize = 2000;

/// Uma chamada registrada (tela Agente › Auditoria).
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct AuditEntry {
    pub id: Id,
    /// Instante UTC (RFC 3339).
    pub created_at: String,
    /// Cliente MCP (ex.: `nome-do-cliente via treasury mcp`).
    pub client: String,
    /// Ferramenta (ex.: `query_transactions`) ou `resources/read`.
    pub tool: String,
    /// Parâmetros resumidos (textos longos e listas grandes cortados).
    #[ts(type = "Record<string, unknown>")]
    pub params: Value,
    pub ok: bool,
    /// `código: mensagem` quando `ok` é falso.
    pub error: Option<String>,
    #[ts(type = "number")]
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuditListArgs {
    /// Padrão 100, máximo 500.
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
    #[ts(optional, type = "number")]
    pub offset: Option<i64>,
    #[ts(optional)]
    pub client: Option<String>,
    #[ts(optional)]
    pub tool: Option<String>,
    /// Só chamadas com erro.
    #[ts(optional)]
    pub only_errors: Option<bool>,
}

/// O que é gravado de uma chamada (o id e o instante são gerados aqui).
#[derive(Debug, Clone)]
pub(crate) struct AuditRecord {
    pub client: String,
    pub tool: String,
    pub params: Value,
    pub error: Option<(String, String)>,
    pub duration_ms: i64,
}

impl AuditRecord {
    fn into_entry(self) -> AuditEntry {
        AuditEntry {
            id: new_id(),
            created_at: now_utc(),
            client: self.client,
            tool: self.tool,
            params: self.params,
            ok: self.error.is_none(),
            error: self.error.map(|(code, msg)| format!("{code}: {msg}")),
            duration_ms: self.duration_ms,
        }
    }
}

/// Linha do arquivo pendente: só metadados, nunca parâmetros ou mensagens.
#[derive(Debug, Serialize, Deserialize)]
struct PendingLine {
    id: Id,
    created_at: String,
    client: String,
    tool: String,
    ok: bool,
    error_code: Option<String>,
    duration_ms: i64,
}

/// Grava a chamada no banco, ou no arquivo pendente se o banco não puder ser escrito.
/// Nunca falha: auditoria não pode derrubar a ferramenta (falhas vão para o log técnico).
pub(crate) fn record(api: Option<&Api>, run_dir: Option<&Path>, rec: AuditRecord) {
    let entry = rec.into_entry();
    if let Some(api) = api {
        let written = api.with_session(|s, paths| {
            if s.db.is_read_only() {
                return Ok(false);
            }
            import_pending(s.db.conn(), &paths.run_dir);
            insert(s.db.conn(), &entry)?;
            Ok(true)
        });
        match written {
            Ok(true) => return,
            Ok(false) => {}
            Err(crate::Error::Locked | crate::Error::NoVault) => {}
            Err(e) => tracing::warn!("auditoria MCP: falha ao gravar no banco ({})", e.code()),
        }
    }
    let dir = run_dir
        .map(Path::to_path_buf)
        .or_else(|| api.map(|a| a.paths().run_dir.clone()));
    if let Some(dir) = dir
        && let Err(e) = append_pending(&dir, &entry)
    {
        tracing::warn!("auditoria MCP: falha ao gravar pendente ({e})");
    }
}

fn insert(conn: &Connection, e: &AuditEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO ai_audit_log (id, created_at, client, tool, params, ok, error, duration_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            e.id,
            e.created_at,
            e.client,
            e.tool,
            serde_json::to_string(&e.params)?,
            e.ok,
            e.error,
            e.duration_ms
        ],
    )?;
    Ok(())
}

pub fn list(conn: &Connection, a: &AuditListArgs) -> Result<Vec<AuditEntry>> {
    let limit = a.limit.unwrap_or(100).clamp(1, 500);
    let offset = a.offset.unwrap_or(0).max(0);
    let mut stmt = conn.prepare(
        "SELECT id, created_at, client, tool, params, ok, error, duration_ms FROM ai_audit_log
         WHERE (?1 IS NULL OR client = ?1) AND (?2 IS NULL OR tool = ?2) AND (?3 = 0 OR ok = 0)
         ORDER BY created_at DESC, id DESC LIMIT ?4 OFFSET ?5",
    )?;
    let rows = stmt
        .query_map(
            params![
                a.client,
                a.tool,
                a.only_errors.unwrap_or(false),
                limit,
                offset
            ],
            |r| {
                let params: String = r.get(4)?;
                Ok(AuditEntry {
                    id: r.get(0)?,
                    created_at: r.get(1)?,
                    client: r.get(2)?,
                    tool: r.get(3)?,
                    params: serde_json::from_str(&params).unwrap_or(Value::Null),
                    ok: r.get(5)?,
                    error: r.get(6)?,
                    duration_ms: r.get(7)?,
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn pending_path(run_dir: &Path) -> PathBuf {
    run_dir.join(PENDING_FILE)
}

fn append_pending(run_dir: &Path, e: &AuditEntry) -> std::io::Result<()> {
    crate::paths::create_private_dir(run_dir).map_err(std::io::Error::other)?;
    let line = PendingLine {
        id: e.id.clone(),
        created_at: e.created_at.clone(),
        client: e.client.clone(),
        tool: e.tool.clone(),
        ok: e.ok,
        error_code: e
            .error
            .as_deref()
            .map(|s| s.split(':').next().unwrap_or(s).to_string()),
        duration_ms: e.duration_ms,
    };
    let mut text = serde_json::to_string(&line).map_err(std::io::Error::other)?;
    text.push('\n');
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    // Uma escrita por linha (O_APPEND): processos concorrentes não se misturam.
    opts.open(pending_path(run_dir))?.write_all(text.as_bytes())
}

/// Importa as entradas pendentes para o banco (aberto para escrita).
pub(crate) fn import_pending(conn: &Connection, run_dir: &Path) {
    let path = pending_path(run_dir);
    if !path.exists() {
        return;
    }
    // Renomeia antes de ler, para não perder linhas acrescentadas durante a importação.
    let taken = run_dir.join(format!("{PENDING_FILE}.{}", new_id()));
    if std::fs::rename(&path, &taken).is_err() {
        return;
    }
    let text = std::fs::read_to_string(&taken).unwrap_or_default();
    let mut imported = 0usize;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(p) = serde_json::from_str::<PendingLine>(line) else {
            continue;
        };
        let entry = AuditEntry {
            id: p.id,
            created_at: p.created_at,
            client: p.client,
            tool: p.tool,
            params: json!({}),
            ok: p.ok,
            error: p.error_code,
            duration_ms: p.duration_ms,
        };
        // `OR IGNORE`: uma importação interrompida pode repetir linhas.
        let res = conn.execute(
            "INSERT OR IGNORE INTO ai_audit_log (id, created_at, client, tool, params, ok, error, duration_ms)
             VALUES (?1, ?2, ?3, ?4, '{}', ?5, ?6, ?7)",
            params![
                entry.id,
                entry.created_at,
                entry.client,
                entry.tool,
                entry.ok,
                entry.error,
                entry.duration_ms
            ],
        );
        match res {
            Ok(_) => imported += 1,
            Err(e) => {
                tracing::warn!("auditoria MCP: falha ao importar pendentes ({e})");
                // Devolve o arquivo para tentar de novo depois.
                let _ = std::fs::rename(&taken, &path);
                return;
            }
        }
    }
    let _ = std::fs::remove_file(&taken);
    if imported > 0 {
        tracing::info!("auditoria MCP: {imported} entradas pendentes importadas");
    }
}

/// Resumo dos parâmetros para o log: corta textos longos, listas grandes e aninhamento
/// profundo; parâmetros enormes viram só o tamanho.
pub(crate) fn summarize_params(v: &Value) -> Value {
    let out = summarize(v, 0);
    let size = serde_json::to_string(&out).map_or(0, |s| s.len());
    if size > MAX_PARAMS_BYTES {
        return json!({ "_resumo": format!("parâmetros grandes demais ({size} bytes)") });
    }
    match out {
        Value::Null => json!({}),
        other => other,
    }
}

fn summarize(v: &Value, depth: usize) -> Value {
    match v {
        Value::String(s) if s.chars().count() > MAX_STRING => {
            let cut: String = s.chars().take(MAX_STRING).collect();
            Value::String(format!("{cut}…"))
        }
        Value::Array(items) if depth >= MAX_DEPTH => {
            Value::String(format!("[{} itens]", items.len()))
        }
        Value::Object(map) if depth >= MAX_DEPTH => {
            Value::String(format!("{{{} campos}}", map.len()))
        }
        Value::Array(items) => {
            let mut out: Vec<Value> = items
                .iter()
                .take(MAX_ARRAY)
                .map(|i| summarize(i, depth + 1))
                .collect();
            if items.len() > MAX_ARRAY {
                out.push(Value::String(format!("+{} itens", items.len() - MAX_ARRAY)));
            }
            Value::Array(out)
        }
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), summarize(v, depth + 1)))
                .collect::<Map<_, _>>(),
        ),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_are_summarized() {
        let long = "x".repeat(300);
        let ids: Vec<String> = (0..25).map(|i| format!("id{i}")).collect();
        let s = summarize_params(&json!({"text": long, "account_ids": ids, "limit": 10}));
        assert_eq!(s["limit"], 10);
        assert!(s["text"].as_str().unwrap().ends_with('…'));
        assert!(s["text"].as_str().unwrap().chars().count() <= MAX_STRING + 1);
        let arr = s["account_ids"].as_array().unwrap();
        assert_eq!(arr.len(), MAX_ARRAY + 1);
        assert_eq!(arr.last().unwrap(), "+15 itens");
        assert_eq!(summarize_params(&Value::Null), json!({}));
        let deep = json!({"a": {"b": {"c": {"d": 1}}}});
        assert_eq!(summarize_params(&deep)["a"]["b"]["c"], "{1 campos}");
    }
}
