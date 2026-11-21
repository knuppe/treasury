//! Conjuntos de alterações (D17): toda escrita (usuário, importação, IA) passa por
//! [`record`], que captura o antes/depois de cada linha com a extensão *session* do
//! SQLite. Isso dá histórico, desfazer/refazer e auditoria com um único mecanismo.

use std::collections::{BTreeMap, HashMap};

use rusqlite::hooks::Action;
use rusqlite::session::{ConflictAction, ConflictType, Session};
use rusqlite::types::ValueRef;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use ts_rs::TS;

use crate::error::{Error, Result};
// `t` é nome de tabela em várias funções daqui; o texto do catálogo é `tr`.
use crate::i18n::t as tr;
use crate::ids::{Id, new_id};
use crate::time::now_utc;

/// Tabelas técnicas que não entram no histórico.
pub const UNTRACKED_TABLES: &[&str] = &[
    "change_set",
    "change_item",
    "setting",
    "backup_record",
    "refinery_schema_history",
    "ai_audit_log",
    // Chaves de API do MCP (só hashes): segurança, não dado da família.
    "mcp_api_key",
    "ai_usage",
    "sqlite_sequence",
    // Propostas da IA: metadados da fila de aprovação (o diff fica no change_set pendente).
    "ai_proposal",
    // Contadores de acertos das regras (V20): estatística, não dado do usuário.
    "rule_hit",
    // Uso das notas da memória de procedimentos do agente (V87): estatística, como acima.
    "ai_procedure_use",
    // Lançamentos já analisados pela revisão contínua do agente (V107): controle técnico.
    "ai_review_seen",
    // Dados de mercado e derivados (fase 2): baixados ou recalculáveis, não são dados do usuário.
    // Lista de instituições do Banco Central (V94): referência pública, baixada.
    "bank_registry",
    "fundamentus_cache",
    // Leituras das fontes usadas no enriquecimento dos ativos (V130): referência pública.
    "asset_observation",
    "index_value",
    "price",
    "dividend_event",
    "position_snapshot",
    // Cotações intradiárias (V85): última cotação e pontos de 5 min, apagados em dias.
    "market_quote",
    "intraday_price",
    // IA (V71): configuração dos motores (como `setting`), segredos cifrados, uso, conversas
    // e a caixa de alertas. As escritas da IA em dados do usuário continuam passando pelo
    // histórico (propostas).
    "secret",
    "ai_engine",
    "ai_task_route",
    "ai_spend_limit",
    "ai_model_price",
    "ai_conversation",
    "ai_message",
    // Anexos das mensagens (V92): metadados e, só se o usuário escolher guardar, os bytes.
    "ai_attachment",
    "alert",
    // Análise de ativos (fase 3): dados baixados da CVM/SEC/B3 e o universo recalculado.
    "research_issuer",
    "research_security",
    "fundamental",
    "corporate_document",
    "research_quote",
    "research_file",
    "research_universe",
    // Score de ativos: pares do dia (recalculável) e snapshots (derivados, imutáveis).
    "score_universe",
    "score_snapshot",
    // Score em lote do universo (explorador de mercado, V143) e quem cumpre cada tela (V145).
    "market_score",
    "screen_match",
    // Proventos e eventos oficiais da B3 e o controle da coleta (V101).
    "research_dividend",
    "research_stock_event",
    "research_fetch_log",
    // Dados do score (V99): entregas de documentos, auditores e IF.data do Banco Central.
    "research_filing",
    "research_auditor",
    "bank_institution",
    "bank_prudential",
    "bank_ifdata_fetch",
    // E-mail de ingestão (V125): a fila, as peças, o estado das pastas e a auditoria. O que
    // conta para desfazer é o que a importação gravou; contas e regras ficam no histórico.
    "mail_folder_state",
    "mail_message",
    "mail_part",
    // Caixa de entrada de arquivos (V162): o arquivo guardado até a importação.
    "inbox_item",
    "mail_seen",
    "mail_audit",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AuthorKind {
    User,
    Import,
    Ai,
    System,
}

impl AuthorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthorKind::User => "user",
            AuthorKind::Import => "import",
            AuthorKind::Ai => "ai",
            AuthorKind::System => "system",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "import" => AuthorKind::Import,
            "ai" => AuthorKind::Ai,
            "system" => AuthorKind::System,
            _ => AuthorKind::User,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Author {
    pub kind: AuthorKind,
    /// Motor/modelo de IA, cliente MCP, lote de importação…
    pub detail: Option<String>,
}

impl Author {
    pub fn user() -> Self {
        Self {
            kind: AuthorKind::User,
            detail: None,
        }
    }
    pub fn system(detail: impl Into<String>) -> Self {
        Self {
            kind: AuthorKind::System,
            detail: Some(detail.into()),
        }
    }
    pub fn import(detail: impl Into<String>) -> Self {
        Self {
            kind: AuthorKind::Import,
            detail: Some(detail.into()),
        }
    }
    pub fn ai(detail: impl Into<String>) -> Self {
        Self {
            kind: AuthorKind::Ai,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ChangeStatus {
    Pending,
    Applied,
    Undone,
    Rejected,
}

impl ChangeStatus {
    fn parse(s: &str) -> Self {
        match s {
            "pending" => ChangeStatus::Pending,
            "undone" => ChangeStatus::Undone,
            "rejected" => ChangeStatus::Rejected,
            _ => ChangeStatus::Applied,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ChangeOp {
    Insert,
    Update,
    Delete,
}

impl ChangeOp {
    fn as_str(self) -> &'static str {
        match self {
            ChangeOp::Insert => "insert",
            ChangeOp::Update => "update",
            ChangeOp::Delete => "delete",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "insert" => ChangeOp::Insert,
            "delete" => ChangeOp::Delete,
            _ => ChangeOp::Update,
        }
    }
}

/// Resultado de uma escrita registrada.
#[derive(Debug)]
pub struct Recorded<T> {
    pub value: T,
    /// `None` quando a operação não alterou nenhuma linha rastreada.
    pub change_set_id: Option<Id>,
}

/// Executa `f` numa transação, captura todas as linhas alteradas e grava um
/// `change_set` aplicado. Se `f` falhar, nada é gravado.
pub fn record<T>(
    conn: &mut Connection,
    author: &Author,
    reason: &str,
    f: impl FnOnce(&Transaction<'_>) -> Result<T>,
) -> Result<Recorded<T>> {
    if reason.trim().is_empty() {
        return Err(Error::invalid(tr("nucleo-geral-erro-sem-motivo")));
    }
    let tx = conn.transaction()?;
    let (value, blob) = {
        let mut session = Session::new(&tx)?;
        session.table_filter(Some(|t: &str| !UNTRACKED_TABLES.contains(&t)));
        session.attach::<&str>(None)?;
        let value = f(&tx)?;
        let mut blob = Vec::new();
        session.changeset_strm(&mut blob)?;
        (value, blob)
    };
    if blob.is_empty() {
        tx.commit()?;
        return Ok(Recorded {
            value,
            change_set_id: None,
        });
    }
    let items = decode_changeset(&tx, &blob)?;
    let id = new_id();
    store_change_set(&tx, &id, author, reason, "applied", &blob, &items)?;
    tx.commit()?;
    withdraw_overlapping(conn, &id);
    Ok(Recorded {
        value,
        change_set_id: Some(id),
    })
}

/// Propostas pendentes que mexem nos mesmos dados que `id` acabou de mudar saem da fila
/// (aprovar depois sobrescreveria a mudança). Falha aqui não desfaz a escrita.
fn withdraw_overlapping(conn: &Connection, id: &str) {
    if let Err(e) = crate::ai::proposals::withdraw_overlapping(conn, id) {
        tracing::warn!("retirar propostas sobrepostas a {id}: {e}");
    }
}

fn store_change_set(
    tx: &Transaction<'_>,
    id: &str,
    author: &Author,
    reason: &str,
    status: &str,
    blob: &[u8],
    items: &[DecodedItem],
) -> Result<()> {
    let now = now_utc();
    let summary = summarize(items);
    tx.execute(
        "INSERT INTO change_set (id, created_at, author_kind, author_detail, reason, status, applied_at, changeset, summary)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, CASE WHEN ?6 = 'applied' THEN ?2 END, ?7, ?8)",
        params![id, now, author.kind.as_str(), author.detail, reason.trim(), status, blob, summary.to_string()],
    )?;
    let mut stmt = tx.prepare(
        "INSERT INTO change_item (change_set_id, seq, table_name, row_id, op, before, after)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for (seq, it) in items.iter().enumerate() {
        stmt.execute(params![
            id,
            seq as i64,
            it.table,
            it.row_id,
            it.op.as_str(),
            it.before.as_ref().map(|v| v.to_string()),
            it.after.as_ref().map(|v| v.to_string()),
        ])?;
    }
    Ok(())
}

/// Simula `f` (sem gravar nada nas tabelas de dados) e guarda o resultado como um
/// conjunto de alterações **pendente**, com o antes/depois exato de cada linha, para
/// aprovação. Devolve `None` se a operação não alteraria nenhuma linha rastreada.
/// Gravações em tabelas técnicas feitas por `f` também são descartadas.
pub fn propose<T>(
    conn: &mut Connection,
    author: &Author,
    reason: &str,
    f: impl FnOnce(&Transaction<'_>) -> Result<T>,
) -> Result<Recorded<T>> {
    if reason.trim().is_empty() {
        return Err(Error::invalid(tr("nucleo-geral-erro-sem-motivo")));
    }
    let tx = conn.transaction()?;
    let (value, blob) = {
        let mut session = Session::new(&tx)?;
        session.table_filter(Some(|t: &str| !UNTRACKED_TABLES.contains(&t)));
        session.attach::<&str>(None)?;
        let value = f(&tx)?;
        let mut blob = Vec::new();
        session.changeset_strm(&mut blob)?;
        (value, blob)
    };
    if blob.is_empty() {
        tx.rollback()?;
        return Ok(Recorded {
            value,
            change_set_id: None,
        });
    }
    let items = decode_changeset(&tx, &blob)?;
    tx.rollback()?;
    let id = new_id();
    let tx = conn.transaction()?;
    store_change_set(&tx, &id, author, reason, "pending", &blob, &items)?;
    tx.commit()?;
    Ok(Recorded {
        value,
        change_set_id: Some(id),
    })
}

/// Aplica um conjunto pendente. Falha com `conflict` se os dados mudaram desde a proposta.
pub fn approve(conn: &mut Connection, id: &str) -> Result<ChangeSetSummary> {
    let (status, blob) = load_blob(conn, id)?;
    if status != ChangeStatus::Pending {
        return Err(Error::conflict(tr("nucleo-geral-erro-aprovar")));
    }
    apply_without_fk(conn, &blob, |tx| {
        tx.execute(
            "UPDATE change_set SET status = 'applied', applied_at = ?2 WHERE id = ?1",
            params![id, now_utc()],
        )?;
        Ok(())
    })?;
    withdraw_overlapping(conn, id);
    Ok(get(conn, id)?.summary)
}

/// Rejeita um conjunto pendente (nada é aplicado).
pub fn reject(conn: &Connection, id: &str) -> Result<ChangeSetSummary> {
    let n = conn.execute(
        "UPDATE change_set SET status = 'rejected' WHERE id = ?1 AND status = 'pending'",
        [id],
    )?;
    if n == 0 {
        return Err(Error::conflict(tr("nucleo-geral-erro-rejeitar")));
    }
    Ok(get(conn, id)?.summary)
}

#[derive(Debug, Clone)]
struct DecodedItem {
    table: String,
    row_id: String,
    op: ChangeOp,
    before: Option<Value>,
    after: Option<Value>,
}

struct TableInfo {
    columns: Vec<String>,
    pk: Vec<usize>,
}

fn table_info(
    conn: &Connection,
    table: &str,
    cache: &mut HashMap<String, TableInfo>,
) -> Result<()> {
    if cache.contains_key(table) {
        return Ok(());
    }
    let mut stmt = conn.prepare("SELECT name, pk FROM pragma_table_info(?1) ORDER BY cid")?;
    let rows: Vec<(String, i64)> = stmt
        .query_map([table], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut pk: Vec<(i64, usize)> = rows
        .iter()
        .enumerate()
        .filter(|(_, (_, p))| *p > 0)
        .map(|(i, (_, p))| (*p, i))
        .collect();
    pk.sort();
    cache.insert(
        table.to_string(),
        TableInfo {
            columns: rows.into_iter().map(|(n, _)| n).collect(),
            pk: pk.into_iter().map(|(_, i)| i).collect(),
        },
    );
    Ok(())
}

fn value_to_json(v: ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => {
            use sha2::Digest;
            json!({ "$blob": { "size": b.len(), "sha256": hex::encode(sha2::Sha256::digest(b)) } })
        }
    }
}

fn decode_changeset(conn: &Connection, blob: &[u8]) -> Result<Vec<DecodedItem>> {
    use fallible_streaming_iterator::FallibleStreamingIterator;
    let mut cache: HashMap<String, TableInfo> = HashMap::new();
    let mut input: &[u8] = blob;
    let input_ref: &mut dyn std::io::Read = &mut input;
    let mut iter = rusqlite::session::ChangesetIter::start_strm(&input_ref)?;
    let mut out = Vec::new();
    while let Some(item) = iter.next()? {
        let op = item.op()?;
        let table = op.table_name().to_string();
        let ncols = op.number_of_columns() as usize;
        let code = op.code();
        table_info(conn, &table, &mut cache)?;
        let info = &cache[&table];
        let col_name = |i: usize| {
            info.columns
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("col{i}"))
        };
        let mut old = Map::new();
        let mut new = Map::new();
        for i in 0..ncols {
            if matches!(code, Action::SQLITE_DELETE | Action::SQLITE_UPDATE)
                && let Ok(v) = item.old_value(i)
            {
                old.insert(col_name(i), value_to_json(v));
            }
            if matches!(code, Action::SQLITE_INSERT | Action::SQLITE_UPDATE)
                && let Ok(v) = item.new_value(i)
            {
                new.insert(col_name(i), value_to_json(v));
            }
        }
        let source = if matches!(code, Action::SQLITE_INSERT) {
            &new
        } else {
            &old
        };
        let row_id = info
            .pk
            .iter()
            .map(|&i| match source.get(&col_name(i)) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => String::new(),
            })
            .collect::<Vec<_>>()
            .join(":");
        let (op, before, after) = match code {
            Action::SQLITE_INSERT => (ChangeOp::Insert, None, Some(Value::Object(new))),
            Action::SQLITE_DELETE => (ChangeOp::Delete, Some(Value::Object(old)), None),
            _ => {
                // Na atualização, as colunas de chave aparecem só em "old"; repetimos em "new".
                for &i in &info.pk {
                    if let Some(v) = old.get(&col_name(i)).cloned() {
                        new.entry(col_name(i)).or_insert(v);
                    }
                }
                (
                    ChangeOp::Update,
                    Some(Value::Object(old)),
                    Some(Value::Object(new)),
                )
            }
        };
        out.push(DecodedItem {
            table,
            row_id,
            op,
            before,
            after,
        });
    }
    Ok(out)
}

fn summarize(items: &[DecodedItem]) -> Value {
    let mut m: BTreeMap<&str, BTreeMap<&str, i64>> = BTreeMap::new();
    for it in items {
        *m.entry(it.op.as_str())
            .or_default()
            .entry(it.table.as_str())
            .or_default() += 1;
    }
    serde_json::to_value(m).unwrap_or(Value::Null)
}

// ---------------------------------------------------------------------------
// Consulta, desfazer e refazer

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChangeSetSummary {
    pub id: Id,
    pub created_at: String,
    pub author: Author,
    pub reason: String,
    pub status: ChangeStatus,
    pub applied_at: Option<String>,
    pub undone_at: Option<String>,
    /// `{"insert": {"txn": 3}, "update": {...}}`
    #[ts(type = "Record<string, Record<string, number>>")]
    pub summary: Value,
    #[ts(type = "number")]
    pub item_count: i64,
    /// Veio de uma proposta da IA (aprovada pelo usuário em `applied_at`, se aplicada).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub proposal_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChangeItem {
    #[ts(type = "number")]
    pub seq: i64,
    pub table_name: String,
    pub row_id: String,
    pub op: ChangeOp,
    #[ts(type = "Record<string, unknown> | null")]
    pub before: Option<Value>,
    #[ts(type = "Record<string, unknown> | null")]
    pub after: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChangeSetDetail {
    #[serde(flatten)]
    pub summary: ChangeSetSummary,
    pub items: Vec<ChangeItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChangeListArgs {
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
    #[ts(optional, type = "number")]
    pub offset: Option<i64>,
    #[ts(optional)]
    pub author_kind: Option<AuthorKind>,
    /// Só alterações que tocaram esta tabela/linha (ex.: histórico de um lançamento).
    #[ts(optional)]
    pub table_name: Option<String>,
    #[ts(optional)]
    pub row_id: Option<String>,
}

fn summary_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ChangeSetSummary> {
    let summary: String = r.get("summary")?;
    Ok(ChangeSetSummary {
        id: r.get("id")?,
        created_at: r.get("created_at")?,
        author: Author {
            kind: AuthorKind::parse(&r.get::<_, String>("author_kind")?),
            detail: r.get("author_detail")?,
        },
        reason: r.get("reason")?,
        status: ChangeStatus::parse(&r.get::<_, String>("status")?),
        applied_at: r.get("applied_at")?,
        undone_at: r.get("undone_at")?,
        summary: serde_json::from_str(&summary).unwrap_or(Value::Null),
        item_count: r.get("item_count")?,
        proposal_id: r.get("proposal_id")?,
    })
}

const SUMMARY_COLS: &str = "cs.id, cs.created_at, cs.author_kind, cs.author_detail, cs.reason, cs.status, cs.applied_at,
     cs.undone_at, cs.summary, (SELECT count(*) FROM change_item ci WHERE ci.change_set_id = cs.id) AS item_count,
     (SELECT p.id FROM ai_proposal p WHERE p.change_set_id = cs.id) AS proposal_id";

pub fn list(conn: &Connection, args: &ChangeListArgs) -> Result<Vec<ChangeSetSummary>> {
    let limit = args.limit.unwrap_or(50).clamp(1, 500);
    let offset = args.offset.unwrap_or(0).max(0);
    let sql = format!(
        "SELECT {SUMMARY_COLS} FROM change_set cs
         WHERE (?1 IS NULL OR cs.author_kind = ?1)
           AND (?2 IS NULL OR EXISTS (SELECT 1 FROM change_item ci WHERE ci.change_set_id = cs.id
                                      AND ci.table_name = ?2 AND (?3 IS NULL OR ci.row_id = ?3)))
         ORDER BY cs.created_at DESC, cs.id DESC LIMIT ?4 OFFSET ?5"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(
            params![
                args.author_kind.map(|k| k.as_str()),
                args.table_name,
                args.row_id,
                limit,
                offset
            ],
            summary_from_row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: &str) -> Result<ChangeSetDetail> {
    let summary = conn
        .query_row(
            &format!("SELECT {SUMMARY_COLS} FROM change_set cs WHERE cs.id = ?1"),
            [id],
            summary_from_row,
        )
        .optional()?
        .ok_or_else(|| not_found(id))?;
    let mut stmt = conn.prepare(
        "SELECT seq, table_name, row_id, op, before, after FROM change_item WHERE change_set_id = ?1 ORDER BY seq",
    )?;
    let items = stmt
        .query_map([id], |r| {
            let before: Option<String> = r.get(4)?;
            let after: Option<String> = r.get(5)?;
            Ok(ChangeItem {
                seq: r.get(0)?,
                table_name: r.get(1)?,
                row_id: r.get(2)?,
                op: ChangeOp::parse(&r.get::<_, String>(3)?),
                before: before.and_then(|s| serde_json::from_str(&s).ok()),
                after: after.and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ChangeSetDetail { summary, items })
}

fn load_blob(conn: &Connection, id: &str) -> Result<(ChangeStatus, Vec<u8>)> {
    let (status, blob): (String, Option<Vec<u8>>) = conn
        .query_row(
            "SELECT status, changeset FROM change_set WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| not_found(id))?;
    Ok((ChangeStatus::parse(&status), blob.unwrap_or_default()))
}

/// Aplica um changeset (desfazer/refazer) com as chaves estrangeiras desligadas. O changeset
/// já traz todas as linhas, inclusive as removidas em cascata; com as chaves ligadas, um
/// `ON DELETE CASCADE` (ex.: `budget` → `budget_line`, `txn` → `txn_split`) apagaria as
/// filhas antes da hora e o DELETE delas acusaria conflito. A integridade é conferida no
/// fim com `foreign_key_check`: se faltar um pai, é conflito e nada muda.
fn apply_without_fk(
    conn: &mut Connection,
    blob: &[u8],
    mark: impl FnOnce(&Transaction<'_>) -> Result<()>,
) -> Result<()> {
    fn run(
        conn: &mut Connection,
        blob: &[u8],
        mark: impl FnOnce(&Transaction<'_>) -> Result<()>,
    ) -> Result<()> {
        let tx = conn.transaction()?;
        apply_blob(&tx, blob)?;
        let broken: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM pragma_foreign_key_check)",
            [],
            |r| r.get(0),
        )?;
        if broken {
            return Err(conflict_error());
        }
        mark(&tx)?;
        tx.commit()?;
        Ok(())
    }
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    let res = run(conn, blob, mark);
    conn.pragma_update(None, "foreign_keys", "ON")?;
    res
}

/// `conjunto de alterações <id>` (o que não foi encontrado).
fn not_found(id: &str) -> Error {
    Error::not_found(crate::i18n::t_args(
        "nucleo-geral-conjunto",
        &crate::i18n::args(&[("id", id.to_string().into())]),
    ))
}

fn conflict_error() -> Error {
    Error::conflict(tr("nucleo-geral-erro-alteracoes-posteriores"))
}

/// Colunas de carimbo: mudam em toda escrita e não dizem nada sobre o conteúdo.
pub(crate) const STAMP_COLUMNS: &[&str] = &["updated_at", "created_at"];

/// Colunas preenchidas fora do histórico: a chave de origem das categorias padrão, dada aos
/// cofres antigos ao abrir (V127, `categories::backfill_seed_keys`). Uma diferença só nelas
/// também não impede desfazer nem refazer.
/// Também os produtos dos itens de cupom ligados ao abrir (V135, `food::on_open`).
const BACKFILLED_COLUMNS: &[&str] = &[
    "seed_key",
    "product_id",
    "gtin",
    "base_qty",
    "base_unit",
    "base_price",
];

/// Colunas de uma tabela só, preenchidas fora do histórico: as raízes da semente de produtos
/// que descem para um segmento ao abrir, e a ordem delas (V136, `food::seed::ensure`).
const BACKFILLED_TABLE_COLUMNS: &[(&str, &str)] = &[
    ("product_category", "parent_id"),
    ("product_category", "sort"),
];

/// Índices das colunas de carimbo (e das preenchidas fora do histórico) de cada tabela do
/// changeset.
fn stamp_columns(conn: &Connection, blob: &[u8]) -> Result<HashMap<String, Vec<usize>>> {
    use fallible_streaming_iterator::FallibleStreamingIterator;
    let mut cache: HashMap<String, TableInfo> = HashMap::new();
    let mut out: HashMap<String, Vec<usize>> = HashMap::new();
    let mut input: &[u8] = blob;
    let input_ref: &mut dyn std::io::Read = &mut input;
    let mut iter = rusqlite::session::ChangesetIter::start_strm(&input_ref)?;
    while let Some(item) = iter.next()? {
        let table = item.op()?.table_name().to_string();
        if out.contains_key(&table) {
            continue;
        }
        table_info(conn, &table, &mut cache)?;
        let idx = cache[&table]
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                STAMP_COLUMNS.contains(&c.as_str())
                    || BACKFILLED_COLUMNS.contains(&c.as_str())
                    || BACKFILLED_TABLE_COLUMNS.contains(&(table.as_str(), c.as_str()))
            })
            .map(|(i, _)| i)
            .collect();
        out.insert(table, idx);
    }
    Ok(out)
}

/// A linha atual difere do "antes" do changeset só nos carimbos (outra escrita mexeu em
/// outras colunas da mesma linha): aplicar não perde nada.
fn only_stamps_differ(
    item: &rusqlite::session::ChangesetItem,
    stamps: &HashMap<String, Vec<usize>>,
) -> bool {
    let Ok(op) = item.op() else {
        return false;
    };
    let Some(cols) = stamps.get(op.table_name()) else {
        return false;
    };
    (0..op.number_of_columns() as usize).all(|i| {
        cols.contains(&i)
            || match (item.old_value(i), item.conflict(i)) {
                (Ok(old), Ok(cur)) => old == cur,
                // Coluna fora da alteração (atualização só guarda as colunas mudadas).
                _ => true,
            }
    })
}

fn apply_blob(tx: &Transaction<'_>, blob: &[u8]) -> Result<()> {
    let stamps = stamp_columns(tx, blob)?;
    let mut input: &[u8] = blob;
    // Uma exclusão que não acha a linha porque um `ON DELETE CASCADE` de outra linha do
    // mesmo conjunto já a removeu (ex.: desfazer a criação de uma tag e do vínculo
    // txn_tag) já está no estado desejado: ignora. Uma linha que só mudou nos carimbos
    // (`updated_at`) desde então recebe a alteração. Qualquer outro conflito aborta.
    let on_conflict = move |t: ConflictType, item: rusqlite::session::ChangesetItem| {
        let is_delete = item
            .op()
            .map(|o| o.code() == Action::SQLITE_DELETE)
            .unwrap_or(false);
        if t == ConflictType::SQLITE_CHANGESET_NOTFOUND && is_delete {
            ConflictAction::SQLITE_CHANGESET_OMIT
        } else if t == ConflictType::SQLITE_CHANGESET_DATA && only_stamps_differ(&item, &stamps) {
            ConflictAction::SQLITE_CHANGESET_REPLACE
        } else {
            ConflictAction::SQLITE_CHANGESET_ABORT
        }
    };
    tx.apply_strm(&mut input, None::<fn(&str) -> bool>, on_conflict)
        .map_err(|_| conflict_error())
}

/// Desfaz um conjunto aplicado. Falha com `conflict` se alterações posteriores
/// tocaram nas mesmas linhas.
pub fn undo(conn: &mut Connection, id: &str) -> Result<ChangeSetSummary> {
    let (status, blob) = load_blob(conn, id)?;
    if status != ChangeStatus::Applied {
        return Err(Error::conflict(tr("nucleo-geral-erro-desfazer")));
    }
    let mut inverted = Vec::new();
    rusqlite::session::invert_strm(&mut &blob[..], &mut inverted)?;
    apply_without_fk(conn, &inverted, |tx| {
        tx.execute(
            "UPDATE change_set SET status = 'undone', undone_at = ?2 WHERE id = ?1",
            params![id, now_utc()],
        )?;
        Ok(())
    })?;
    withdraw_overlapping(conn, id);
    Ok(get(conn, id)?.summary)
}

/// Reaplica um conjunto desfeito.
pub fn redo(conn: &mut Connection, id: &str) -> Result<ChangeSetSummary> {
    let (status, blob) = load_blob(conn, id)?;
    if status != ChangeStatus::Undone {
        return Err(Error::conflict(tr("nucleo-geral-erro-refazer")));
    }
    apply_without_fk(conn, &blob, |tx| {
        tx.execute(
            "UPDATE change_set SET status = 'applied', applied_at = ?2, undone_at = NULL WHERE id = ?1",
            params![id, now_utc()],
        )?;
        Ok(())
    })?;
    withdraw_overlapping(conn, id);
    Ok(get(conn, id)?.summary)
}

/// Quantos conjuntos foram aplicados depois de um instante (usado pelo backup ao fechar).
pub fn count_since(conn: &Connection, since: Option<&str>) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT count(*) FROM change_set WHERE (?1 IS NULL OR created_at > ?1) OR (?1 IS NOT NULL AND undone_at > ?1)",
        [since],
        |r| r.get(0),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    fn db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        (dir, db)
    }

    fn add_member(tx: &Transaction<'_>, id: &str, name: &str) -> Result<()> {
        tx.execute(
            "INSERT INTO member (id, name, created_at, updated_at) VALUES (?1, ?2, 'a', 'a')",
            params![id, name],
        )?;
        Ok(())
    }

    fn names(conn: &Connection) -> Vec<String> {
        let mut s = conn.prepare("SELECT name FROM member ORDER BY id").unwrap();
        s.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    }

    #[test]
    fn records_insert_update_delete_with_before_after() {
        let (_d, mut db) = db();
        let r = record(db.conn_mut(), &Author::user(), "criar membros", |tx| {
            add_member(tx, "m1", "Ana")?;
            add_member(tx, "m2", "Bruno")
        })
        .unwrap();
        let id1 = r.change_set_id.unwrap();
        let d = get(db.conn(), &id1).unwrap();
        assert_eq!(d.items.len(), 2);
        let ana = d.items.iter().find(|i| i.row_id == "m1").unwrap();
        assert_eq!(ana.op, ChangeOp::Insert);
        assert!(ana.before.is_none());
        assert_eq!(ana.after.as_ref().unwrap()["name"], "Ana");
        assert_eq!(d.summary.summary["insert"]["member"], 2);

        let r2 = record(db.conn_mut(), &Author::ai("cli-teste"), "renomear", |tx| {
            tx.execute("UPDATE member SET name = 'Ana Maria' WHERE id = 'm1'", [])?;
            tx.execute("DELETE FROM member WHERE id = 'm2'", [])?;
            Ok(())
        })
        .unwrap();
        let d2 = get(db.conn(), r2.change_set_id.as_ref().unwrap()).unwrap();
        let upd = d2.items.iter().find(|i| i.op == ChangeOp::Update).unwrap();
        assert_eq!(upd.row_id, "m1");
        assert_eq!(upd.before.as_ref().unwrap()["name"], "Ana");
        assert_eq!(upd.after.as_ref().unwrap()["name"], "Ana Maria");
        assert!(
            upd.after.as_ref().unwrap().get("created_at").is_none(),
            "only changed columns"
        );
        let del = d2.items.iter().find(|i| i.op == ChangeOp::Delete).unwrap();
        assert_eq!(del.before.as_ref().unwrap()["name"], "Bruno");
        assert_eq!(d2.summary.author.kind, AuthorKind::Ai);
    }

    #[test]
    fn undo_and_redo_restore_state() {
        let (_d, mut db) = db();
        let a = record(db.conn_mut(), &Author::user(), "a", |tx| {
            add_member(tx, "m1", "Ana")
        })
        .unwrap();
        let b = record(db.conn_mut(), &Author::user(), "b", |tx| {
            tx.execute("UPDATE member SET name = 'Ana B' WHERE id = 'm1'", [])?;
            add_member(tx, "m2", "Caio")
        })
        .unwrap();
        assert_eq!(names(db.conn()), vec!["Ana B", "Caio"]);

        // Undoing "a" before "b" conflicts (b changed the same row).
        assert!(matches!(
            undo(db.conn_mut(), a.change_set_id.as_ref().unwrap()),
            Err(Error::Conflict(_))
        ));
        assert_eq!(
            names(db.conn()),
            vec!["Ana B", "Caio"],
            "conflict changes nothing"
        );

        let sb = undo(db.conn_mut(), b.change_set_id.as_ref().unwrap()).unwrap();
        assert_eq!(sb.status, ChangeStatus::Undone);
        assert_eq!(names(db.conn()), vec!["Ana"]);
        undo(db.conn_mut(), a.change_set_id.as_ref().unwrap()).unwrap();
        assert!(names(db.conn()).is_empty());

        redo(db.conn_mut(), a.change_set_id.as_ref().unwrap()).unwrap();
        redo(db.conn_mut(), b.change_set_id.as_ref().unwrap()).unwrap();
        assert_eq!(names(db.conn()), vec!["Ana B", "Caio"]);
        assert!(matches!(
            redo(db.conn_mut(), b.change_set_id.as_ref().unwrap()),
            Err(Error::Conflict(_))
        ));
    }

    #[test]
    fn undo_and_redo_with_cascading_children() {
        let (_d, mut db) = db();
        let cat = record(db.conn_mut(), &Author::user(), "categoria", |tx| {
            tx.execute(
                "INSERT INTO category (id, name, kind, created_at, updated_at) VALUES ('c1', 'Casa', 'expense', 'a', 'a')",
                [],
            )?;
            Ok(())
        })
        .unwrap()
        .change_set_id
        .unwrap();
        // Parent and child in the same set (budget → budget_line has ON DELETE CASCADE).
        let lines = record(db.conn_mut(), &Author::user(), "orçamento", |tx| {
            tx.execute(
                "INSERT INTO budget (id, month, created_at, updated_at) VALUES ('b1', '2026-10', 'a', 'a')",
                [],
            )?;
            tx.execute(
                "INSERT INTO budget_line (id, budget_id, category_id, amount, created_at, updated_at)
                 VALUES ('l1', 'b1', 'c1', 100, 'a', 'a')",
                [],
            )?;
            Ok(())
        })
        .unwrap()
        .change_set_id
        .unwrap();
        let count = |db: &Db| -> (i64, i64) {
            db.conn()
                .query_row(
                    "SELECT (SELECT count(*) FROM budget), (SELECT count(*) FROM budget_line)",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap()
        };
        // Undoing the category before the budget would leave the row orphaned: conflict.
        assert!(matches!(undo(db.conn_mut(), &cat), Err(Error::Conflict(_))));
        assert_eq!(count(&db), (1, 1));
        undo(db.conn_mut(), &lines).unwrap();
        assert_eq!(count(&db), (0, 0));
        redo(db.conn_mut(), &lines).unwrap();
        assert_eq!(count(&db), (1, 1));
        // Deleting the parent removes the child by cascade; undoing brings both back.
        let del = record(db.conn_mut(), &Author::user(), "apagar", |tx| {
            tx.execute("DELETE FROM budget WHERE id = 'b1'", [])?;
            Ok(())
        })
        .unwrap()
        .change_set_id
        .unwrap();
        assert_eq!(count(&db), (0, 0));
        undo(db.conn_mut(), &del).unwrap();
        assert_eq!(count(&db), (1, 1));
        let fk: i64 = db
            .conn()
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fk, 1, "foreign keys re-enabled");
    }

    #[test]
    fn failed_write_records_nothing_and_settings_are_untracked() {
        let (_d, mut db) = db();
        let r: Result<Recorded<()>> = record(db.conn_mut(), &Author::user(), "x", |tx| {
            add_member(tx, "m1", "Ana")?;
            Err(Error::invalid("falhou"))
        });
        assert!(r.is_err());
        assert!(names(db.conn()).is_empty());
        let r = record(db.conn_mut(), &Author::user(), "tema", |tx| {
            tx.execute(
                "INSERT INTO setting VALUES ('theme', '\"dark\"', 'now')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        assert!(r.change_set_id.is_none());
        assert!(
            list(db.conn(), &ChangeListArgs::default())
                .unwrap()
                .is_empty()
        );
        assert!(record(db.conn_mut(), &Author::user(), "  ", |_| Ok(())).is_err());
    }

    #[test]
    fn undo_tolerates_rows_removed_by_cascade() {
        let (_d, mut db) = db();
        db.conn()
            .execute_batch(
                "CREATE TABLE p (id TEXT PRIMARY KEY);
                 CREATE TABLE c (p_id TEXT NOT NULL REFERENCES p (id) ON DELETE CASCADE, v TEXT, PRIMARY KEY (p_id));",
            )
            .unwrap();
        let r = record(db.conn_mut(), &Author::user(), "pai e filho", |tx| {
            tx.execute("INSERT INTO p VALUES ('a')", [])?;
            tx.execute("INSERT INTO c VALUES ('a', 'x')", [])?;
            Ok(())
        })
        .unwrap();
        undo(db.conn_mut(), r.change_set_id.as_ref().unwrap()).unwrap();
        let n: i64 = db
            .conn()
            .query_row(
                "SELECT (SELECT count(*) FROM p) + (SELECT count(*) FROM c)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0);
        redo(db.conn_mut(), r.change_set_id.as_ref().unwrap()).unwrap();
        let v: String = db
            .conn()
            .query_row("SELECT v FROM c", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, "x");
    }

    #[test]
    fn list_filters_by_row() {
        let (_d, mut db) = db();
        record(db.conn_mut(), &Author::user(), "a", |tx| {
            add_member(tx, "m1", "Ana")
        })
        .unwrap();
        record(db.conn_mut(), &Author::import("ofx"), "b", |tx| {
            add_member(tx, "m2", "Bia")
        })
        .unwrap();
        let only_m2 = list(
            db.conn(),
            &ChangeListArgs {
                table_name: Some("member".into()),
                row_id: Some("m2".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(only_m2.len(), 1);
        assert_eq!(only_m2[0].reason, "b");
        let imports = list(
            db.conn(),
            &ChangeListArgs {
                author_kind: Some(AuthorKind::Import),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(imports.len(), 1);
    }
}

#[cfg(test)]
mod proposal_tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    fn count(conn: &Connection) -> i64 {
        conn.query_row("SELECT count(*) FROM member", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn propose_does_not_touch_data_until_approved() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let id = propose(db.conn_mut(), &Author::ai("agente"), "criar membro", |tx| {
            tx.execute("INSERT INTO member (id, name, created_at, updated_at) VALUES ('m1', 'Ana', 'a', 'a')", [])?;
            Ok(())
        })
        .unwrap()
        .change_set_id
        .unwrap();
        assert_eq!(count(db.conn()), 0, "nothing written before approval");
        let d = get(db.conn(), &id).unwrap();
        assert_eq!(d.summary.status, ChangeStatus::Pending);
        assert_eq!(
            d.items[0].after.as_ref().unwrap()["name"],
            "Ana",
            "exact diff available for review"
        );
        approve(db.conn_mut(), &id).unwrap();
        assert_eq!(count(db.conn()), 1);
        assert!(approve(db.conn_mut(), &id).is_err(), "cannot approve twice");
        // Approved becomes an ordinary change: it can be undone.
        undo(db.conn_mut(), &id).unwrap();
        assert_eq!(count(db.conn()), 0);
    }

    #[test]
    fn reject_and_conflicting_approval() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::user(), "m", |tx| {
            tx.execute("INSERT INTO member (id, name, created_at, updated_at) VALUES ('m1', 'Ana', 'a', 'a')", [])?;
            Ok(())
        })
        .unwrap();
        let p = propose(db.conn_mut(), &Author::ai("agente"), "renomear", |tx| {
            tx.execute("UPDATE member SET name = 'Ana Maria' WHERE id = 'm1'", [])?;
            Ok(())
        })
        .unwrap()
        .change_set_id
        .unwrap();
        // The user changed the same row after the proposal: approving gives a conflict.
        record(db.conn_mut(), &Author::user(), "u", |tx| {
            tx.execute("UPDATE member SET name = 'Ana B' WHERE id = 'm1'", [])?;
            Ok(())
        })
        .unwrap();
        assert!(matches!(
            approve(db.conn_mut(), &p),
            Err(Error::Conflict(_))
        ));
        let r = reject(db.conn(), &p).unwrap();
        assert_eq!(r.status, ChangeStatus::Rejected);
        let name: String = db
            .conn()
            .query_row("SELECT name FROM member", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Ana B");
        assert!(
            propose(db.conn_mut(), &Author::ai("x"), "nada", |_| Ok(()))
                .unwrap()
                .change_set_id
                .is_none()
        );
    }
}
