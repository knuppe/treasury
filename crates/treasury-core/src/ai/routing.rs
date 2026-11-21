//! Roteamento por tarefa: para cada tarefa, a lista ordenada de motores a tentar (o
//! próximo entra se o anterior falhar, estiver sem chave ou sem cota).

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{self, Engine};
use crate::error::{Error, Result};
use crate::ids::Id;
use crate::vault::DbKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AiTask {
    Chat,
    Categorize,
    Pdf,
    MonthlyClose,
    Alerts,
    /// Estudo de um ativo com IA (docs/agent-enrichment.md §7).
    Study,
}

pub const ALL_TASKS: &[AiTask] = &[
    AiTask::Chat,
    AiTask::Categorize,
    AiTask::Pdf,
    AiTask::MonthlyClose,
    AiTask::Alerts,
    AiTask::Study,
];

impl AiTask {
    pub fn as_str(self) -> &'static str {
        match self {
            AiTask::Chat => "chat",
            AiTask::Categorize => "categorize",
            AiTask::Pdf => "pdf",
            AiTask::MonthlyClose => "monthly_close",
            AiTask::Alerts => "alerts",
            AiTask::Study => "study",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        ALL_TASKS.iter().copied().find(|t| t.as_str() == s)
    }

    pub fn label(self) -> &'static str {
        match self {
            AiTask::Chat => "Chat com o agente",
            AiTask::Categorize => "Categorização",
            AiTask::Pdf => "Leitura de PDF",
            AiTask::MonthlyClose => "Fechamento mensal",
            AiTask::Alerts => "Alertas",
            AiTask::Study => "Estudo de ativos",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskRoute {
    pub task: AiTask,
    pub label: String,
    /// Motores na ordem de tentativa.
    pub engine_ids: Vec<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetRouteArgs {
    pub task: AiTask,
    pub engine_ids: Vec<Id>,
}

pub fn route(conn: &Connection, task: AiTask) -> Result<Vec<Id>> {
    let mut stmt =
        conn.prepare("SELECT engine_id FROM ai_task_route WHERE task = ?1 ORDER BY position")?;
    Ok(stmt
        .query_map([task.as_str()], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn list(conn: &Connection) -> Result<Vec<TaskRoute>> {
    ALL_TASKS
        .iter()
        .map(|&t| {
            Ok(TaskRoute {
                task: t,
                label: t.label().into(),
                engine_ids: route(conn, t)?,
            })
        })
        .collect()
}

pub fn set(conn: &Connection, a: &SetRouteArgs) -> Result<TaskRoute> {
    let mut seen = std::collections::HashSet::new();
    for id in &a.engine_ids {
        if !seen.insert(id) {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-ia-rota-motor-repetido",
            )));
        }
        let exists: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM ai_engine WHERE id = ?1)",
            [id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(Error::not_found(crate::i18n::t_args(
                "nucleo-ia-motor-id",
                &crate::i18n::args(&[("id", id.to_string().into())]),
            )));
        }
    }
    conn.execute(
        "DELETE FROM ai_task_route WHERE task = ?1",
        [a.task.as_str()],
    )?;
    for (i, id) in a.engine_ids.iter().enumerate() {
        conn.execute(
            "INSERT INTO ai_task_route (task, position, engine_id) VALUES (?1, ?2, ?3)",
            params![a.task.as_str(), i as i64, id],
        )?;
    }
    Ok(TaskRoute {
        task: a.task,
        label: a.task.label().into(),
        engine_ids: route(conn, a.task)?,
    })
}

/// Motores a tentar, em ordem. Com `explicit`, só ele. Sem rota própria, tarefas de chat
/// usam a rota de `chat`; rotinas em segundo plano (`strict`) só usam a rota da própria
/// tarefa: dados só vão para um motor que o usuário escolheu para aquilo.
pub fn candidates(
    conn: &Connection,
    key: &DbKey,
    task: AiTask,
    explicit: Option<&str>,
    strict: bool,
) -> Result<Vec<Engine>> {
    if let Some(id) = explicit {
        return Ok(vec![engine::get(conn, key, id)?]);
    }
    let mut ids = route(conn, task)?;
    if ids.is_empty() && !strict && task != AiTask::Chat {
        ids = route(conn, AiTask::Chat)?;
    }
    ids.iter()
        .map(|id| engine::get(conn, key, id))
        .collect::<Result<Vec<_>>>()
        .map(|v| v.into_iter().filter(|e| e.enabled).collect())
}
