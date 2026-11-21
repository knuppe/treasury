//! Feedback do agente para o desenvolvedor do Treasury (modo desenvolvedor, ajuste
//! `developer_tools`): bugs, melhorias, ferramentas que faltaram, dados confusos e lacunas
//! da documentação que o agente percebeu ao trabalhar. Ver docs/ai-agent.md.
//!
//! - O agente (e clientes MCP) só adicionam (`devfeedback.add`), reforçam
//!   (`devfeedback.bump`) e marcam como resolvido com o motivo (`devfeedback.resolve`), e só
//!   com o modo desenvolvedor ligado; gravam direto, sem proposta (é informação técnica, não
//!   dado da família), mas com conjunto de alterações (V150): mover no quadro, editar e
//!   mesclar podem ser desfeitos no histórico.
//! - Ao adicionar, um feedback aberto com título muito parecido é reforçado em vez de
//!   duplicado (+1 de importância e uma nota com o texto novo).
//! - O quadro (tela Desenvolvedor) tem uma coluna por status; dentro dela, a ordem manual
//!   (`position`) vem antes da importância. Vínculos ligam feedbacks (duplicado, depende de,
//!   relacionado); marcar como duplicado mescla no principal.
//! - Os outros status, a edição e a exclusão só pela interface (tela Desenvolvedor).
//! - Textos vindos de agentes passam pela mesma limpeza da memória de procedimentos
//!   (senhas, códigos, CPF, cartão, conta, e-mail, telefone): o feedback costuma virar
//!   issue no GitHub.

pub mod api;

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::time::now_utc;
use crate::transactions::normalize_description;

pub const MAX_TITLE: usize = 160;
pub const MAX_BODY: usize = 4_000;
pub const MAX_NOTE: usize = 1_000;
/// Campos estruturados (proposta, critério de aceite, esperado, obtido, passos).
pub const MAX_FIELD: usize = 2_000;
const MAX_POINTERS: usize = 20;
const MAX_POINTER: usize = 300;
const MAX_LABELS: usize = 12;
const MAX_LABEL: usize = 40;
const MAX_TOOL_CALLS: usize = 20;
const MAX_SHORT: usize = 120;
/// Similaridade mínima (Jaccard das palavras do título) para reforçar em vez de criar.
const DEDUPE_MIN: f64 = 0.6;
/// Ajuste que liga o modo desenvolvedor (Ajustes › Desenvolvedor).
pub const DEVELOPER_SETTING: &str = "developer_tools";

/// Parte do Treasury a que o feedback se refere.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DevFeedbackArea {
    Agente,
    Mcp,
    Importacao,
    Lancamentos,
    Carteira,
    Ui,
    Dados,
    #[default]
    Outro,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DevFeedbackKind {
    Bug,
    #[default]
    Melhoria,
    FerramentaFaltando,
    Dados,
    Documentacao,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DevFeedbackStatus {
    Aberto,
    EmAndamento,
    Resolvido,
    Descartado,
}

impl DevFeedbackStatus {
    fn is_open(self) -> bool {
        matches!(self, Self::Aberto | Self::EmAndamento)
    }
}

/// Quanto o problema pesa para o usuário (quem registrou avalia).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DevFeedbackImpact {
    Baixo,
    Medio,
    Alto,
}

/// O que cada nota da linha do tempo registra.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DevFeedbackNoteKind {
    /// Reforço: soma 1 de importância.
    #[default]
    Reforco,
    /// Mudança de status, com o motivo.
    Status,
    /// Um duplicado foi mesclado neste.
    Mescla,
    Comentario,
}

/// Vínculo entre feedbacks, lido de quem aponta: "este é duplicado de / depende de / se
/// relaciona com aquele".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DevFeedbackLinkKind {
    Duplicado,
    DependeDe,
    Relacionado,
}

/// Texto do enum como gravado no banco (o mesmo do JSON).
fn tag<T: Serialize>(v: T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn untag<T: for<'de> Deserialize<'de>>(s: &str) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::from(s)).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Um reforço: quem, quando e por quê.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackNote {
    pub id: Id,
    pub kind: DevFeedbackNoteKind,
    pub note: Option<String>,
    pub client: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedback {
    pub id: Id,
    pub title: String,
    pub body: String,
    pub area: DevFeedbackArea,
    pub kind: DevFeedbackKind,
    /// Começa em 1; cada reforço soma 1 (o usuário pode ajustar).
    #[ts(type = "number")]
    pub importance: i64,
    pub status: DevFeedbackStatus,
    /// Cliente que criou (ex.: "agente (nome da CLI)") ou `user`.
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_bumped_at: Option<String>,
    #[ts(type = "number")]
    pub bumps: i64,
    /// Ordem manual na coluna do status (1 = topo); `None` = pela importância.
    #[ts(type = "number | null")]
    pub position: Option<i64>,
    /// O que fazer (o texto livre fica em `body`, o problema).
    pub proposal: Option<String>,
    /// Como saber que está resolvido (teste sugerido).
    pub acceptance_criteria: Option<String>,
    /// Onde mexer: `caminho/arquivo.rs:120`.
    pub code_pointers: Vec<String>,
    pub labels: Vec<String>,
    /// Quem cuida (ex.: `agente-cli`, `usuario`).
    pub assignee: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub repro_steps: Option<String>,
    pub user_impact: Option<DevFeedbackImpact>,
    /// O usuário confirmou o fato.
    pub confirmed_by_user: bool,
    pub client_version: Option<String>,
    pub app_version: Option<String>,
    /// Chamadas que levaram ao problema (sem valores sensíveis).
    pub tool_calls: Vec<DevFeedbackToolCall>,
    /// Quando foi resolvido ou descartado.
    pub closed_at: Option<String>,
    /// Linha do tempo: reforços, mudanças de status e mesclas, do mais antigo ao mais novo.
    pub notes: Vec<DevFeedbackNote>,
    /// Vínculos com outros feedbacks, nos dois sentidos.
    pub links: Vec<DevFeedbackLink>,
}

/// Uma chamada de ferramenta que levou ao problema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackToolCall {
    pub tool: String,
    /// Parâmetros resumidos, sem valores nem nomes.
    #[serde(default)]
    #[ts(optional)]
    pub params: Option<String>,
    /// Resultado ou erro, resumido.
    #[serde(default)]
    #[ts(optional)]
    pub outcome: Option<String>,
}

/// Um vínculo visto deste feedback: `incoming` quando é o outro que aponta para este (ex.: o
/// outro é duplicado deste).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackLink {
    pub kind: DevFeedbackLinkKind,
    pub incoming: bool,
    pub related_id: Id,
    pub related_title: String,
    pub related_status: DevFeedbackStatus,
    pub created_by: String,
    pub created_at: String,
}

/// Um feedback com o histórico de alterações (mover, editar, mesclar), do mais novo ao mais
/// antigo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackDetail {
    pub feedback: DevFeedback,
    pub history: Vec<crate::changes::ChangeSetSummary>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackListArgs {
    #[ts(optional)]
    pub status: Option<DevFeedbackStatus>,
    #[ts(optional)]
    pub area: Option<DevFeedbackArea>,
    /// Palavras no título ou no texto (sem acento, sem caixa).
    #[ts(optional)]
    pub query: Option<String>,
    /// Só abertos e em andamento (o que o agente consulta antes de adicionar).
    #[ts(optional)]
    pub open_only: Option<bool>,
    #[ts(optional)]
    pub kind: Option<DevFeedbackKind>,
    /// Importância mínima e máxima (inclusive).
    #[ts(optional, type = "number")]
    pub min_importance: Option<i64>,
    #[ts(optional, type = "number")]
    pub max_importance: Option<i64>,
    #[ts(optional)]
    pub assignee: Option<String>,
    #[ts(optional)]
    pub label: Option<String>,
    /// Só os fechados (resolvidos ou descartados) a partir deste instante (RFC 3339): o cliente
    /// que abriu fica sabendo o que foi resolvido desde a última consulta.
    #[ts(optional)]
    pub closed_since: Option<String>,
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
    /// `status`: devolve o quadro (uma coluna por status) em vez da lista.
    #[ts(optional)]
    pub group_by: Option<DevFeedbackGroupBy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DevFeedbackGroupBy {
    Status,
}

/// O quadro: as colunas na ordem do fluxo, cada uma na ordem da fila.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackBoard {
    pub columns: Vec<DevFeedbackColumn>,
    #[ts(type = "number")]
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackColumn {
    pub status: DevFeedbackStatus,
    #[ts(type = "number")]
    pub count: usize,
    pub items: Vec<DevFeedback>,
}

/// Markdown pronto para colar num agente de código: um feedback (`id`) ou os do filtro.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackMarkdownArgs {
    #[ts(optional)]
    pub id: Option<Id>,
    #[ts(optional)]
    pub status: Option<DevFeedbackStatus>,
    #[ts(optional)]
    pub area: Option<DevFeedbackArea>,
    #[ts(optional)]
    pub kind: Option<DevFeedbackKind>,
    #[ts(optional, type = "number")]
    pub min_importance: Option<i64>,
    #[ts(optional)]
    pub assignee: Option<String>,
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackMarkdown {
    pub markdown: String,
    #[ts(type = "number")]
    pub count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackInput {
    pub title: String,
    /// O problema (texto livre).
    pub body: String,
    pub area: DevFeedbackArea,
    pub kind: DevFeedbackKind,
    #[serde(default)]
    #[ts(optional)]
    pub proposal: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub acceptance_criteria: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub code_pointers: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub assignee: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub expected: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub actual: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub repro_steps: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub user_impact: Option<DevFeedbackImpact>,
    #[serde(default)]
    #[ts(optional)]
    pub confirmed_by_user: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub client_version: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub tool_calls: Option<Vec<DevFeedbackToolCall>>,
    /// Feedbacks relacionados (vínculo `relacionado`).
    #[serde(default)]
    #[ts(optional)]
    pub related_ids: Option<Vec<Id>>,
}

/// Resultado de `devfeedback.add`: o feedback criado, ou o existente reforçado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackAdded {
    pub feedback: DevFeedback,
    /// Já havia um aberto com título muito parecido: ele foi reforçado.
    pub deduplicated: bool,
    pub message: String,
    /// Trechos sensíveis ocultados (senhas, documentos, contas…).
    pub redactions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackBump {
    pub id: Id,
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackStatusArgs {
    pub id: Id,
    pub status: DevFeedbackStatus,
    /// Por que mudou (vira nota na linha do tempo).
    #[serde(default)]
    #[ts(optional)]
    pub reason: Option<String>,
    /// Lugar na coluna nova (1 = topo); sem ele, vai pela importância.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub position: Option<i64>,
}

/// Prioridade: a importância e/ou o lugar manual na coluna (independente dos reforços).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackPriority {
    pub id: Id,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub importance: Option<i64>,
    /// 1 = topo da coluna.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub position: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackLinkArgs {
    pub id: Id,
    pub related_id: Id,
    pub kind: DevFeedbackLinkKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackUnlink {
    pub id: Id,
    pub related_id: Id,
}

/// Resultado de marcar como duplicado: o principal (com os reforços somados) e o duplicado,
/// já descartado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DevFeedbackLinked {
    pub feedback: DevFeedback,
    pub related: DevFeedback,
    pub merged: bool,
}

/// Um agente marca como resolvido, com o motivo (o que foi corrigido e onde).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackResolve {
    pub id: Id,
    pub reason: String,
}

/// Edição pela interface (campos ausentes ficam como estão).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DevFeedbackPatch {
    pub id: Id,
    #[ts(optional)]
    pub title: Option<String>,
    #[ts(optional)]
    pub body: Option<String>,
    #[ts(optional)]
    pub area: Option<DevFeedbackArea>,
    #[ts(optional)]
    pub kind: Option<DevFeedbackKind>,
    #[ts(optional, type = "number")]
    pub importance: Option<i64>,
    /// Texto vazio apaga o campo.
    #[ts(optional)]
    pub proposal: Option<String>,
    #[ts(optional)]
    pub acceptance_criteria: Option<String>,
    #[ts(optional)]
    pub code_pointers: Option<Vec<String>>,
    #[ts(optional)]
    pub labels: Option<Vec<String>>,
    #[ts(optional)]
    pub assignee: Option<String>,
    #[ts(optional)]
    pub expected: Option<String>,
    #[ts(optional)]
    pub actual: Option<String>,
    #[ts(optional)]
    pub repro_steps: Option<String>,
    #[ts(optional)]
    pub user_impact: Option<DevFeedbackImpact>,
    #[ts(optional)]
    pub confirmed_by_user: Option<bool>,
}

/// O modo desenvolvedor está ligado?
pub fn developer_mode(conn: &Connection) -> Result<bool> {
    crate::settings::get_or(conn, DEVELOPER_SETTING, false)
}

fn clean(field: &str, s: &str, max: usize) -> Result<String> {
    use crate::i18n::{args, t_args};
    let text = s.trim();
    if text.is_empty() {
        return Err(Error::invalid(t_args(
            "nucleo-geral-erro-feedback-vazio",
            &args(&[("campo", field.to_string().into())]),
        )));
    }
    if text.chars().count() > max {
        return Err(Error::invalid(t_args(
            "nucleo-geral-erro-feedback-longo",
            &args(&[("campo", field.to_string().into()), ("maximo", max.into())]),
        )));
    }
    Ok(text.to_string())
}

/// `feedback <id>` (o que não foi encontrado).
fn not_found(id: &str) -> Error {
    Error::not_found(crate::i18n::t_args(
        "nucleo-geral-feedback",
        &crate::i18n::args(&[("id", id.to_string().into())]),
    ))
}

fn clean_note(note: Option<&str>) -> Result<Option<String>> {
    match note.map(str::trim).filter(|n| !n.is_empty()) {
        None => Ok(None),
        Some(n) => clean("note", n, MAX_NOTE).map(Some),
    }
}

/// Texto escrito por um agente: sem senhas, documentos nem contas.
fn scrub(text: String, from_agent: bool, count: &mut u32) -> String {
    if !from_agent {
        return text;
    }
    let (out, n) = crate::ai::procedures::scrub(&text);
    *count += n;
    out
}

const STOPWORDS: &[&str] = &[
    "a", "o", "as", "os", "de", "da", "do", "das", "dos", "e", "em", "no", "na", "nos", "nas",
    "um", "uma", "para", "pra", "por", "com", "sem", "que", "ao", "the", "of", "to", "in",
];

/// Palavras do texto sem acento, sem caixa, sem as curtas e com o plural simples cortado
/// ("faturas" = "fatura").
fn words(title: &str) -> HashSet<String> {
    normalize_description(title)
        .split(' ')
        .filter(|w| !w.is_empty() && !STOPWORDS.contains(w))
        .map(|w| match w.strip_suffix('s') {
            Some(stem) if stem.len() >= 3 => stem.to_string(),
            _ => w.to_string(),
        })
        .collect()
}

/// Quão parecidos são dois títulos (0 a 1): iguais sem acento e caixa valem 1; senão, a
/// fração de palavras em comum (Jaccard), ou 0,9 quando todas as palavras do mais curto
/// (pelo menos três) estão no outro.
pub fn title_similarity(a: &str, b: &str) -> f64 {
    let (na, nb) = (normalize_description(a), normalize_description(b));
    if !na.is_empty() && na == nb {
        return 1.0;
    }
    let (wa, wb) = (words(a), words(b));
    if wa.is_empty() || wb.is_empty() {
        return 0.0;
    }
    let common = wa.intersection(&wb).count() as f64;
    let jaccard = common / wa.union(&wb).count() as f64;
    let shorter = wa.len().min(wb.len());
    if shorter >= 3 && common as usize == shorter {
        return jaccard.max(0.9);
    }
    jaccard
}

const COLUMNS: &str = "id, title, body, area, kind, importance, status, created_by, created_at, \
                       updated_at, last_bumped_at, bumps, position, proposal, acceptance_criteria, \
                       code_pointers, labels, assignee, expected, actual, repro_steps, user_impact, \
                       confirmed_by_user, client_version, app_version, tool_calls, closed_at";

/// Ordem do quadro e da lista: a manual primeiro, depois a importância e o reforço mais novo.
const ORDER: &str = "position IS NULL, position, importance DESC, \
                     COALESCE(last_bumped_at, created_at) DESC, created_at DESC, id DESC";

fn json_list<T: for<'de> Deserialize<'de>>(s: &str) -> Vec<T> {
    serde_json::from_str(s).unwrap_or_default()
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<DevFeedback> {
    Ok(DevFeedback {
        id: r.get(0)?,
        title: r.get(1)?,
        body: r.get(2)?,
        area: untag(&r.get::<_, String>(3)?)?,
        kind: untag(&r.get::<_, String>(4)?)?,
        importance: r.get(5)?,
        status: untag(&r.get::<_, String>(6)?)?,
        created_by: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
        last_bumped_at: r.get(10)?,
        bumps: r.get(11)?,
        position: r.get(12)?,
        proposal: r.get(13)?,
        acceptance_criteria: r.get(14)?,
        code_pointers: json_list(&r.get::<_, String>(15)?),
        labels: json_list(&r.get::<_, String>(16)?),
        assignee: r.get(17)?,
        expected: r.get(18)?,
        actual: r.get(19)?,
        repro_steps: r.get(20)?,
        user_impact: r
            .get::<_, Option<String>>(21)?
            .map(|s| untag(&s))
            .transpose()?,
        confirmed_by_user: r.get(22)?,
        client_version: r.get(23)?,
        app_version: r.get(24)?,
        tool_calls: json_list(&r.get::<_, String>(25)?),
        closed_at: r.get(26)?,
        notes: vec![],
        links: vec![],
    })
}

fn notes_of(conn: &Connection, id: &str) -> Result<Vec<DevFeedbackNote>> {
    let mut st = conn.prepare(
        "SELECT id, kind, note, client, created_at FROM dev_feedback_note
         WHERE feedback_id = ?1 ORDER BY created_at, rowid",
    )?;
    let out = st
        .query_map([id], |r| {
            Ok(DevFeedbackNote {
                id: r.get(0)?,
                kind: untag(&r.get::<_, String>(1)?)?,
                note: r.get(2)?,
                client: r.get(3)?,
                created_at: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(out)
}

fn links_of(conn: &Connection, id: &str) -> Result<Vec<DevFeedbackLink>> {
    let mut st = conn.prepare(
        "SELECT l.kind, l.feedback_id <> ?1, f.id, f.title, f.status, l.created_by, l.created_at
           FROM dev_feedback_link l
           JOIN dev_feedback f
             ON f.id = CASE WHEN l.feedback_id = ?1 THEN l.related_id ELSE l.feedback_id END
          WHERE l.feedback_id = ?1 OR l.related_id = ?1
          ORDER BY l.created_at, l.rowid",
    )?;
    let out = st
        .query_map([id], |r| {
            Ok(DevFeedbackLink {
                kind: untag(&r.get::<_, String>(0)?)?,
                incoming: r.get(1)?,
                related_id: r.get(2)?,
                related_title: r.get(3)?,
                related_status: untag(&r.get::<_, String>(4)?)?,
                created_by: r.get(5)?,
                created_at: r.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(out)
}

fn fill(conn: &Connection, f: &mut DevFeedback) -> Result<()> {
    f.notes = notes_of(conn, &f.id)?;
    f.links = links_of(conn, &f.id)?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<DevFeedback> {
    let mut f = conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM dev_feedback WHERE id = ?1"),
            [id],
            row,
        )
        .optional()?
        .ok_or_else(|| not_found(id))?;
    fill(conn, &mut f)?;
    Ok(f)
}

/// O feedback com o histórico de alterações dele (mover, editar, mesclar).
pub fn detail(conn: &Connection, id: &str) -> Result<DevFeedbackDetail> {
    let feedback = get(conn, id)?;
    let history = crate::changes::list(
        conn,
        &crate::changes::ChangeListArgs {
            limit: Some(30),
            table_name: Some("dev_feedback".into()),
            row_id: Some(id.to_string()),
            ..Default::default()
        },
    )?;
    Ok(DevFeedbackDetail { feedback, history })
}

/// Lista na ordem do quadro: a ordem manual, depois a importância (maior primeiro) e, no
/// empate, o reforçado mais recentemente.
pub fn list(conn: &Connection, a: &DevFeedbackListArgs) -> Result<Vec<DevFeedback>> {
    let mut st = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM dev_feedback
         WHERE (?1 IS NULL OR status = ?1) AND (?2 IS NULL OR area = ?2)
           AND (?3 = 0 OR status IN ('aberto', 'em_andamento'))
           AND (?4 IS NULL OR kind = ?4)
           AND (?5 IS NULL OR importance >= ?5) AND (?6 IS NULL OR importance <= ?6)
           AND (?7 IS NULL OR assignee = ?7)
           AND (?8 IS NULL OR (closed_at IS NOT NULL AND closed_at >= ?8))
         ORDER BY {ORDER}"
    ))?;
    let status = a.status.map(tag);
    let area = a.area.map(tag);
    let kind = a.kind.map(tag);
    let open_only = a.open_only.unwrap_or(false) as i64;
    let assignee = a
        .assignee
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let mut items = st
        .query_map(
            params![
                status,
                area,
                open_only,
                kind,
                a.min_importance,
                a.max_importance,
                assignee,
                a.closed_since
            ],
            row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if let Some(q) = a.query.as_deref().map(words).filter(|w| !w.is_empty()) {
        items.retain(|f| {
            let have = words(&format!("{} {}", f.title, f.body));
            q.iter().all(|w| have.contains(w))
        });
    }
    if let Some(label) = a.label.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let label = normalize_description(label);
        items.retain(|f| f.labels.iter().any(|l| normalize_description(l) == label));
    }
    if let Some(limit) = a.limit {
        items.truncate(limit.clamp(1, 500) as usize);
    }
    for f in &mut items {
        fill(conn, f)?;
    }
    Ok(items)
}

/// Feedback aberto com o título mais parecido (acima do limite).
fn similar_open(conn: &Connection, title: &str) -> Result<Option<(Id, f64)>> {
    let mut st = conn
        .prepare("SELECT id, title FROM dev_feedback WHERE status IN ('aberto', 'em_andamento')")?;
    let rows = st
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .map(|(id, t)| (id, title_similarity(title, &t)))
        .filter(|(_, s)| *s >= DEDUPE_MIN)
        .max_by(|a, b| a.1.total_cmp(&b.1)))
}

/// Adiciona um feedback, ou reforça o aberto de título muito parecido.
pub fn add(
    conn: &Connection,
    input: &DevFeedbackInput,
    client: &str,
    from_agent: bool,
) -> Result<DevFeedbackAdded> {
    add_inner(conn, input, client, from_agent, true)
}

/// Adiciona sem juntar com títulos parecidos (avisos automáticos por item, como "dados
/// ausentes em ITSA4" e "… em AMZN", que são problemas diferentes). Quem chama evita repetir
/// o mesmo título.
pub fn add_distinct(
    conn: &Connection,
    input: &DevFeedbackInput,
    client: &str,
) -> Result<DevFeedbackAdded> {
    add_inner(conn, input, client, false, false)
}

/// Fecha como resolvido o feedback aberto com este título exato, registrado por `client`,
/// com a explicação no fim do texto. Devolve se havia um.
pub fn resolve_auto(conn: &Connection, title: &str, client: &str, why: &str) -> Result<bool> {
    let n = conn.execute(
        "UPDATE dev_feedback SET status = 'resolvido', updated_at = ?3, closed_at = ?3,
                position = NULL, body = body || char(10) || char(10) || ?4
          WHERE title = ?1 AND created_by = ?2 AND status IN ('aberto', 'em_andamento')",
        params![title, client, now_utc(), why],
    )?;
    Ok(n > 0)
}

/// Texto opcional limpo: vazio vira `None`.
fn opt_text(field: &str, s: Option<&str>, max: usize) -> Result<Option<String>> {
    match s.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(t) => clean(field, t, max).map(Some),
    }
}

/// Lista de textos curtos: sem vazios nem repetidos, no máximo `max_n` com `max` caracteres.
fn clean_list(field: &str, items: &[String], max_n: usize, max: usize) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for it in items {
        let it = it.trim();
        if it.is_empty() || out.iter().any(|o| o == it) {
            continue;
        }
        out.push(clean(field, it, max)?);
    }
    if out.len() > max_n {
        return Err(Error::invalid(crate::i18n::t_args(
            "nucleo-geral-erro-feedback-itens",
            &crate::i18n::args(&[
                ("campo", field.to_string().into()),
                ("maximo", max_n.into()),
            ]),
        )));
    }
    Ok(out)
}

fn clean_calls(calls: &[DevFeedbackToolCall]) -> Result<Vec<DevFeedbackToolCall>> {
    calls
        .iter()
        .take(MAX_TOOL_CALLS)
        .map(|c| {
            Ok(DevFeedbackToolCall {
                tool: clean("tool_calls.tool", &c.tool, MAX_SHORT)?,
                params: opt_text("tool_calls.params", c.params.as_deref(), MAX_NOTE)?,
                outcome: opt_text("tool_calls.outcome", c.outcome.as_deref(), MAX_NOTE)?,
            })
        })
        .collect()
}

/// Os campos estruturados já limpos (e, de agentes, sem dados sensíveis).
struct Context {
    proposal: Option<String>,
    acceptance_criteria: Option<String>,
    code_pointers: Vec<String>,
    labels: Vec<String>,
    assignee: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    repro_steps: Option<String>,
    client_version: Option<String>,
    tool_calls: Vec<DevFeedbackToolCall>,
}

fn context(input: &DevFeedbackInput, from_agent: bool, n: &mut u32) -> Result<Context> {
    let mut text = |field: &str, v: Option<&str>, max: usize| -> Result<Option<String>> {
        Ok(opt_text(field, v, max)?.map(|t| scrub(t, from_agent, n)))
    };
    let proposal = text("proposal", input.proposal.as_deref(), MAX_FIELD)?;
    let acceptance_criteria = text(
        "acceptance_criteria",
        input.acceptance_criteria.as_deref(),
        MAX_FIELD,
    )?;
    let expected = text("expected", input.expected.as_deref(), MAX_FIELD)?;
    let actual = text("actual", input.actual.as_deref(), MAX_FIELD)?;
    let repro_steps = text("repro_steps", input.repro_steps.as_deref(), MAX_FIELD)?;
    let client_version = opt_text("client_version", input.client_version.as_deref(), MAX_SHORT)?;
    let assignee = opt_text("assignee", input.assignee.as_deref(), MAX_LABEL)?;
    let mut tool_calls = clean_calls(input.tool_calls.as_deref().unwrap_or_default())?;
    for c in &mut tool_calls {
        c.params = c.params.take().map(|t| scrub(t, from_agent, n));
        c.outcome = c.outcome.take().map(|t| scrub(t, from_agent, n));
    }
    Ok(Context {
        proposal,
        acceptance_criteria,
        code_pointers: clean_list(
            "code_pointers",
            input.code_pointers.as_deref().unwrap_or_default(),
            MAX_POINTERS,
            MAX_POINTER,
        )?,
        labels: clean_list(
            "labels",
            input.labels.as_deref().unwrap_or_default(),
            MAX_LABELS,
            MAX_LABEL,
        )?,
        assignee,
        expected,
        actual,
        repro_steps,
        client_version,
        tool_calls,
    })
}

fn add_inner(
    conn: &Connection,
    input: &DevFeedbackInput,
    client: &str,
    from_agent: bool,
    merge_similar: bool,
) -> Result<DevFeedbackAdded> {
    let mut redactions = 0;
    let title = scrub(
        clean("title", &input.title, MAX_TITLE)?,
        from_agent,
        &mut redactions,
    );
    let body = scrub(
        clean("body", &input.body, MAX_BODY)?,
        from_agent,
        &mut redactions,
    );
    let ctx = context(input, from_agent, &mut redactions)?;
    let related = input.related_ids.clone().unwrap_or_default();
    for r in &related {
        get(conn, r)?;
    }
    if let Some((id, _)) = similar_open(conn, &title)?.filter(|_| merge_similar) {
        // O texto novo vira a nota do reforço (cortado), para nada se perder.
        let note: String = format!("{title}: {body}").chars().take(MAX_NOTE).collect();
        let feedback = bump_inner(conn, &id, Some(note), client)?;
        let message = crate::i18n::t_args(
            "nucleo-geral-feedback-reforcado",
            &crate::i18n::args(&[
                ("titulo", feedback.title.clone().into()),
                ("importancia", feedback.importance.into()),
            ]),
        );
        return Ok(DevFeedbackAdded {
            feedback,
            deduplicated: true,
            message,
            redactions,
        });
    }
    let id = new_id();
    let now = now_utc();
    conn.execute(
        "INSERT INTO dev_feedback (id, title, body, area, kind, importance, status, created_by,
                                   created_at, updated_at, last_bumped_at, bumps, proposal,
                                   acceptance_criteria, code_pointers, labels, assignee, expected,
                                   actual, repro_steps, user_impact, confirmed_by_user,
                                   client_version, app_version, tool_calls)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 'aberto', ?6, ?7, ?7, NULL, 0, ?8, ?9, ?10, ?11, ?12, ?13,
                 ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
        params![
            id,
            title,
            body,
            tag(input.area),
            tag(input.kind),
            client,
            now,
            ctx.proposal,
            ctx.acceptance_criteria,
            serde_json::to_string(&ctx.code_pointers)?,
            serde_json::to_string(&ctx.labels)?,
            ctx.assignee,
            ctx.expected,
            ctx.actual,
            ctx.repro_steps,
            input.user_impact.map(tag),
            input.confirmed_by_user.unwrap_or(false),
            ctx.client_version,
            crate::config::app_version(),
            serde_json::to_string(&ctx.tool_calls)?,
        ],
    )?;
    for r in related.iter().filter(|r| **r != id) {
        insert_link(conn, &id, r, DevFeedbackLinkKind::Relacionado, client)?;
    }
    Ok(DevFeedbackAdded {
        feedback: get(conn, &id)?,
        deduplicated: false,
        message: crate::i18n::t("nucleo-geral-feedback-registrado"),
        redactions,
    })
}

fn insert_note(
    conn: &Connection,
    id: &str,
    kind: DevFeedbackNoteKind,
    note: Option<&str>,
    client: &str,
    at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO dev_feedback_note (id, feedback_id, kind, note, client, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![new_id(), id, tag(kind), note, client, at],
    )?;
    Ok(())
}

fn bump_inner(
    conn: &Connection,
    id: &str,
    note: Option<String>,
    client: &str,
) -> Result<DevFeedback> {
    let now = now_utc();
    let n = conn.execute(
        "UPDATE dev_feedback SET importance = importance + 1, bumps = bumps + 1,
                last_bumped_at = ?2, updated_at = ?2
         WHERE id = ?1",
        params![id, now],
    )?;
    if n == 0 {
        return Err(not_found(id));
    }
    insert_note(
        conn,
        id,
        DevFeedbackNoteKind::Reforco,
        note.as_deref(),
        client,
        &now,
    )?;
    get(conn, id)
}

/// Reforça um feedback: +1 de importância e uma nota (opcional) na linha do tempo.
/// Reforçar um resolvido ou descartado o reabre (o problema voltou).
pub fn bump(
    conn: &Connection,
    a: &DevFeedbackBump,
    client: &str,
    from_agent: bool,
) -> Result<DevFeedback> {
    let mut redactions = 0;
    let note = clean_note(a.note.as_deref())?.map(|n| scrub(n, from_agent, &mut redactions));
    let f = bump_inner(conn, &a.id, note, client)?;
    if !f.status.is_open() {
        return set_status(
            conn,
            &DevFeedbackStatusArgs {
                id: f.id,
                status: DevFeedbackStatus::Aberto,
                reason: None,
                position: None,
            },
            client,
            from_agent,
        );
    }
    Ok(f)
}

/// Marca como resolvido, com o motivo no fim do texto (quem resolveu e por quê), como na
/// resolução automática. Um já resolvido ou descartado fica como está (erro claro).
pub fn resolve(
    conn: &Connection,
    a: &DevFeedbackResolve,
    client: &str,
    from_agent: bool,
) -> Result<DevFeedback> {
    let f = get(conn, &a.id)?;
    if !f.status.is_open() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-geral-erro-feedback-ja-fechado",
        )));
    }
    let mut redactions = 0;
    let reason = clean_note(Some(&a.reason))?
        .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-geral-erro-feedback-motivo")))?;
    let reason = scrub(reason, from_agent, &mut redactions);
    let why = crate::i18n::t_args(
        "nucleo-geral-feedback-resolvido-por",
        &crate::i18n::args(&[
            ("cliente", client.to_string().into()),
            ("motivo", reason.into()),
        ]),
    );
    conn.execute(
        "UPDATE dev_feedback SET status = 'resolvido', updated_at = ?2, closed_at = ?2,
                position = NULL, body = body || char(10) || char(10) || ?3
          WHERE id = ?1",
        params![a.id, now_utc(), why],
    )?;
    get(conn, &a.id)
}

/// Ids da coluna `status` na ordem do quadro.
fn column(conn: &Connection, status: &str) -> Result<Vec<(Id, Option<i64>)>> {
    let mut st = conn.prepare(&format!(
        "SELECT id, position FROM dev_feedback WHERE status = ?1 ORDER BY {ORDER}"
    ))?;
    let out = st
        .query_map([status], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(out)
}

/// Põe `id` no lugar `position` (1 = topo) da coluna dele e renumera a coluna inteira (só
/// grava as linhas cujo lugar mudou).
fn place(conn: &Connection, id: &str, position: i64) -> Result<()> {
    if position < 1 {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-geral-erro-feedback-posicao",
        )));
    }
    let status: String = conn
        .query_row("SELECT status FROM dev_feedback WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or_else(|| not_found(id))?;
    let mut col = column(conn, &status)?;
    let at = col.iter().position(|(c, _)| c == id).unwrap_or(0);
    let me = col.remove(at);
    let to = ((position - 1) as usize).min(col.len());
    col.insert(to, me);
    for (i, (fid, cur)) in col.iter().enumerate() {
        let want = i as i64 + 1;
        if *cur != Some(want) {
            conn.execute(
                "UPDATE dev_feedback SET position = ?2 WHERE id = ?1",
                params![fid, want],
            )?;
        }
    }
    Ok(())
}

/// Muda de coluna (status), com o motivo na linha do tempo e, se veio, o lugar na coluna
/// nova; sem lugar, entra pela importância. Fechar marca `closed_at`; reabrir limpa.
pub fn set_status(
    conn: &Connection,
    a: &DevFeedbackStatusArgs,
    client: &str,
    from_agent: bool,
) -> Result<DevFeedback> {
    let cur = get(conn, &a.id)?;
    let mut redactions = 0;
    let reason = clean_note(a.reason.as_deref())?.map(|r| scrub(r, from_agent, &mut redactions));
    let now = now_utc();
    if cur.status != a.status {
        conn.execute(
            "UPDATE dev_feedback SET status = ?2, updated_at = ?3, position = NULL,
                    closed_at = CASE WHEN ?2 IN ('resolvido', 'descartado') THEN ?3 END
             WHERE id = ?1",
            params![a.id, tag(a.status), now],
        )?;
    }
    if cur.status != a.status || reason.is_some() {
        let status = crate::i18n::t_args(
            "nucleo-geral-feedback-status-nota",
            &crate::i18n::args(&[("status", tag(a.status).into())]),
        );
        let label = match &reason {
            Some(r) => format!("{status}: {r}"),
            None => status,
        };
        insert_note(
            conn,
            &a.id,
            DevFeedbackNoteKind::Status,
            Some(&label),
            client,
            &now,
        )?;
    }
    if let Some(p) = a.position {
        place(conn, &a.id, p)?;
    }
    get(conn, &a.id)
}

/// Prioridade: a importância (1 a 1000) e/ou o lugar manual na coluna.
pub fn set_priority(conn: &Connection, a: &DevFeedbackPriority) -> Result<DevFeedback> {
    if a.importance.is_none() && a.position.is_none() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-geral-erro-feedback-prioridade",
        )));
    }
    let cur = get(conn, &a.id)?;
    if let Some(imp) = a.importance {
        check_importance(imp)?;
        if imp != cur.importance {
            conn.execute(
                "UPDATE dev_feedback SET importance = ?2, updated_at = ?3 WHERE id = ?1",
                params![a.id, imp, now_utc()],
            )?;
        }
    }
    if let Some(p) = a.position {
        place(conn, &a.id, p)?;
    }
    get(conn, &a.id)
}

fn check_importance(importance: i64) -> Result<()> {
    if !(1..=1_000).contains(&importance) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-geral-erro-feedback-importancia",
        )));
    }
    Ok(())
}

pub fn update(conn: &Connection, p: &DevFeedbackPatch, from_agent: bool) -> Result<DevFeedback> {
    let cur = get(conn, &p.id)?;
    let mut n = 0;
    let mut sc = |t: String| scrub(t, from_agent, &mut n);
    let title = match &p.title {
        Some(t) => sc(clean("title", t, MAX_TITLE)?),
        None => cur.title,
    };
    let body = match &p.body {
        Some(b) => sc(clean("body", b, MAX_BODY)?),
        None => cur.body,
    };
    let importance = p.importance.unwrap_or(cur.importance);
    check_importance(importance)?;
    // Campo ausente fica; texto vazio apaga.
    let mut text = |field: &str, new: &Option<String>, old: Option<String>, max: usize| match new {
        None => Ok(old),
        Some(v) => opt_text(field, Some(v), max).map(|t| t.map(&mut sc)),
    };
    let proposal = text("proposal", &p.proposal, cur.proposal, MAX_FIELD)?;
    let acceptance = text(
        "acceptance_criteria",
        &p.acceptance_criteria,
        cur.acceptance_criteria,
        MAX_FIELD,
    )?;
    let expected = text("expected", &p.expected, cur.expected, MAX_FIELD)?;
    let actual = text("actual", &p.actual, cur.actual, MAX_FIELD)?;
    let repro = text("repro_steps", &p.repro_steps, cur.repro_steps, MAX_FIELD)?;
    let assignee = match &p.assignee {
        None => cur.assignee,
        Some(v) => opt_text("assignee", Some(v), MAX_LABEL)?,
    };
    let pointers = match &p.code_pointers {
        None => cur.code_pointers,
        Some(v) => clean_list("code_pointers", v, MAX_POINTERS, MAX_POINTER)?,
    };
    let labels = match &p.labels {
        None => cur.labels,
        Some(v) => clean_list("labels", v, MAX_LABELS, MAX_LABEL)?,
    };
    conn.execute(
        "UPDATE dev_feedback SET title = ?2, body = ?3, area = ?4, kind = ?5, importance = ?6,
                updated_at = ?7, proposal = ?8, acceptance_criteria = ?9, code_pointers = ?10,
                labels = ?11, assignee = ?12, expected = ?13, actual = ?14, repro_steps = ?15,
                user_impact = ?16, confirmed_by_user = ?17
         WHERE id = ?1",
        params![
            p.id,
            title,
            body,
            tag(p.area.unwrap_or(cur.area)),
            tag(p.kind.unwrap_or(cur.kind)),
            importance,
            now_utc(),
            proposal,
            acceptance,
            serde_json::to_string(&pointers)?,
            serde_json::to_string(&labels)?,
            assignee,
            expected,
            actual,
            repro,
            p.user_impact.or(cur.user_impact).map(tag),
            p.confirmed_by_user.unwrap_or(cur.confirmed_by_user),
        ],
    )?;
    get(conn, &p.id)
}

fn insert_link(
    conn: &Connection,
    id: &str,
    related: &str,
    kind: DevFeedbackLinkKind,
    client: &str,
) -> Result<()> {
    // Um vínculo por par, em qualquer sentido: o novo substitui o anterior.
    conn.execute(
        "DELETE FROM dev_feedback_link
          WHERE (feedback_id = ?1 AND related_id = ?2) OR (feedback_id = ?2 AND related_id = ?1)",
        params![id, related],
    )?;
    conn.execute(
        "INSERT INTO dev_feedback_link (id, feedback_id, related_id, kind, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![new_id(), id, related, tag(kind), client, now_utc()],
    )?;
    Ok(())
}

/// Liga `id` a `related_id`. `duplicado`: `id` é duplicado de `related_id`, que fica como
/// principal; os reforços (notas e importância) passam para o principal e o duplicado é
/// descartado, com a explicação nos dois.
pub fn link(conn: &Connection, a: &DevFeedbackLinkArgs, client: &str) -> Result<DevFeedbackLinked> {
    if a.id == a.related_id {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-geral-erro-feedback-vinculo-proprio",
        )));
    }
    let dup = get(conn, &a.id)?;
    let main = get(conn, &a.related_id)?;
    insert_link(conn, &a.id, &a.related_id, a.kind, client)?;
    let merged = a.kind == DevFeedbackLinkKind::Duplicado;
    if merged {
        let now = now_utc();
        conn.execute(
            "UPDATE dev_feedback_note SET feedback_id = ?2
              WHERE feedback_id = ?1 AND kind = 'reforco'",
            params![dup.id, main.id],
        )?;
        conn.execute(
            "UPDATE dev_feedback SET importance = MIN(importance + ?2, 1000),
                    bumps = bumps + ?3, last_bumped_at = ?4, updated_at = ?4
              WHERE id = ?1",
            params![main.id, dup.importance, dup.bumps + 1, now],
        )?;
        let into = crate::i18n::t_args(
            "nucleo-geral-feedback-mesclado",
            &crate::i18n::args(&[("titulo", dup.title.clone().into())]),
        );
        insert_note(
            conn,
            &main.id,
            DevFeedbackNoteKind::Mescla,
            Some(&into),
            client,
            &now,
        )?;
        if dup.status.is_open() {
            let why = crate::i18n::t_args(
                "nucleo-geral-feedback-duplicado-de",
                &crate::i18n::args(&[("titulo", main.title.clone().into())]),
            );
            set_status(
                conn,
                &DevFeedbackStatusArgs {
                    id: dup.id.clone(),
                    status: DevFeedbackStatus::Descartado,
                    reason: Some(why),
                    position: None,
                },
                client,
                false,
            )?;
        }
    }
    Ok(DevFeedbackLinked {
        feedback: get(conn, &a.id)?,
        related: get(conn, &a.related_id)?,
        merged,
    })
}

/// Desfaz o vínculo entre os dois (em qualquer sentido). Uma mescla já feita fica: use o
/// desfazer do histórico para voltar atrás nela.
pub fn unlink(conn: &Connection, a: &DevFeedbackUnlink) -> Result<DevFeedback> {
    let n = conn.execute(
        "DELETE FROM dev_feedback_link
          WHERE (feedback_id = ?1 AND related_id = ?2) OR (feedback_id = ?2 AND related_id = ?1)",
        params![a.id, a.related_id],
    )?;
    if n == 0 {
        return Err(Error::not_found(crate::i18n::t_args(
            "nucleo-geral-feedback-vinculo",
            &crate::i18n::args(&[
                ("id", a.id.clone().into()),
                ("outro", a.related_id.clone().into()),
            ]),
        )));
    }
    get(conn, &a.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM dev_feedback_link WHERE feedback_id = ?1 OR related_id = ?1",
        [id],
    )?;
    conn.execute("DELETE FROM dev_feedback_note WHERE feedback_id = ?1", [id])?;
    let n = conn.execute("DELETE FROM dev_feedback WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(not_found(id));
    }
    Ok(())
}

/// As colunas do quadro (Aberto, Em andamento, Resolvido, Descartado), mantendo a ordem.
pub fn board(items: Vec<DevFeedback>) -> DevFeedbackBoard {
    let count = items.len();
    let columns = [
        DevFeedbackStatus::Aberto,
        DevFeedbackStatus::EmAndamento,
        DevFeedbackStatus::Resolvido,
        DevFeedbackStatus::Descartado,
    ]
    .into_iter()
    .map(|status| {
        let items: Vec<_> = items
            .iter()
            .filter(|f| f.status == status)
            .cloned()
            .collect();
        DevFeedbackColumn {
            status,
            count: items.len(),
            items,
        }
    })
    .collect();
    DevFeedbackBoard { columns, count }
}

/// Markdown de um feedback para um agente de código: contexto, problema, esperado × obtido,
/// passos, proposta, critério de aceite, onde mexer, chamadas, relacionados, reforços e como
/// fechar pelo MCP.
pub fn feedback_markdown(f: &DevFeedback) -> String {
    use crate::i18n::{args, t, t_args};
    let h = |k: &str| t(&format!("nucleo-geral-feedback-md-{k}"));
    let mut out = vec![
        format!("# {}", f.title),
        String::new(),
        t_args(
            "nucleo-geral-feedback-md-contexto",
            &args(&[
                ("id", f.id.clone().into()),
                ("area", tag(f.area).into()),
                ("tipo", tag(f.kind).into()),
                ("prioridade", f.importance.into()),
                ("status", tag(f.status).into()),
                ("reforcos", f.bumps.into()),
            ]),
        ),
        String::new(),
        format!("## {}", h("problema")),
        String::new(),
        f.body.trim().to_string(),
    ];
    let mut section = |title: String, body: String| {
        out.extend([String::new(), format!("## {title}"), String::new(), body]);
    };
    if f.expected.is_some() || f.actual.is_some() {
        let mut lines = vec![];
        if let Some(e) = &f.expected {
            lines.push(format!("- {}: {e}", h("esperado")));
        }
        if let Some(a) = &f.actual {
            lines.push(format!("- {}: {a}", h("obtido")));
        }
        section(h("esperado-obtido"), lines.join("\n"));
    }
    if let Some(r) = &f.repro_steps {
        section(h("passos"), r.clone());
    }
    if let Some(p) = &f.proposal {
        section(h("proposta"), p.clone());
    }
    section(
        h("aceite"),
        f.acceptance_criteria
            .clone()
            .unwrap_or_else(|| h("sem-aceite")),
    );
    if !f.code_pointers.is_empty() {
        let list = f
            .code_pointers
            .iter()
            .map(|p| format!("- `{p}`"))
            .collect::<Vec<_>>();
        section(h("onde"), list.join("\n"));
    }
    if !f.tool_calls.is_empty() {
        let list = f
            .tool_calls
            .iter()
            .map(|c| {
                let mut l = format!("- `{}`", c.tool);
                if let Some(p) = &c.params {
                    l.push_str(&format!(" ({p})"));
                }
                if let Some(o) = &c.outcome {
                    l.push_str(&format!(": {o}"));
                }
                l
            })
            .collect::<Vec<_>>();
        section(h("chamadas"), list.join("\n"));
    }
    if !f.links.is_empty() {
        let list = f
            .links
            .iter()
            .map(|l| {
                format!(
                    "- {} {}: {} (`{}`, {})",
                    tag(l.kind),
                    if l.incoming { "←" } else { "→" },
                    l.related_title,
                    l.related_id,
                    tag(l.related_status)
                )
            })
            .collect::<Vec<_>>();
        section(h("relacionados"), list.join("\n"));
    }
    let notes = f
        .notes
        .iter()
        .filter_map(|n| {
            n.note
                .as_ref()
                .map(|t| format!("- {} · {} · {}: {t}", n.created_at, tag(n.kind), n.client))
        })
        .collect::<Vec<_>>();
    if !notes.is_empty() {
        section(h("reforcos"), notes.join("\n"));
    }
    out.extend([
        String::new(),
        "---".into(),
        String::new(),
        t_args(
            "nucleo-geral-feedback-md-fechar",
            &args(&[("id", f.id.clone().into())]),
        ),
    ]);
    format!("{}\n", out.join("\n"))
}

pub fn markdown(conn: &Connection, a: &DevFeedbackMarkdownArgs) -> Result<DevFeedbackMarkdown> {
    let items = match &a.id {
        Some(id) => vec![get(conn, id)?],
        None => list(
            conn,
            &DevFeedbackListArgs {
                status: a.status,
                area: a.area,
                kind: a.kind,
                min_importance: a.min_importance,
                assignee: a.assignee.clone(),
                open_only: Some(a.status.is_none()),
                limit: Some(a.limit.unwrap_or(10).clamp(1, 50)),
                ..Default::default()
            },
        )?,
    };
    let markdown = items
        .iter()
        .map(feedback_markdown)
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok(DevFeedbackMarkdown {
        markdown,
        count: items.len(),
    })
}
