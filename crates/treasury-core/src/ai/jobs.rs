//! Execução assíncrona do agente: o despachante é síncrono, então cada conversa (ou rotina)
//! vira um job numa thread própria. A interface consulta os eventos novos com
//! `ai.chat_poll {job_id, after_seq}`. O job nunca segura a trava do banco durante a rede:
//! lê o que precisa com `Api::with_session`, solta, chama o motor e grava depois.
//!
//! O [`Runtime`] fica dentro da `Api` (um por processo) e guarda os jobs, os tokens de
//! sessão das CLIs (o socket do MCP aceita esses tokens como "agente do app") e uma
//! referência fraca à própria `Api` para as threads.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{EngineKind, Provider};
use super::routing::AiTask;
use crate::api::{Api, SharedApi};
use crate::error::{Error, Result};
use crate::ids::{Id, new_id};
use crate::mcp::inproc::ToolNote;
use crate::time::now_utc;

/// Quanto tempo um job encerrado fica disponível para `chat_poll`.
const KEEP_FINISHED: Duration = Duration::from_secs(30 * 60);
/// Máximo de eventos guardados por job (texto parcial agrupado).
const MAX_EVENTS: usize = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum JobStatus {
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum JobKind {
    Chat,
    Routine,
}

/// Um evento do job, na ordem (`seq` começa em 1).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobEvent {
    #[ts(type = "number")]
    pub seq: u64,
    pub at: String,
    pub event: JobEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum JobEventKind {
    /// Qual motor está recebendo os dados agora (indicador permanente na interface).
    Engine {
        engine_id: Id,
        engine_name: String,
        kind: EngineKind,
        provider: Provider,
        model: Option<String>,
    },
    /// Texto parcial da resposta. Texto seguido de uma chamada de ferramenta é um passo
    /// intermediário (a interface o mostra recolhido com o raciocínio); o que vem depois da
    /// última ferramenta é a resposta. Ver `ai::activity`.
    Text {
        delta: String,
    },
    /// Raciocínio parcial do modelo (blocos de pensamento), mostrado recolhido.
    Reasoning {
        delta: String,
    },
    /// O modelo chamou uma ferramenta.
    ToolCall {
        name: String,
        summary: String,
    },
    /// Resultado resumido (sem os dados).
    ToolResult {
        name: String,
        ok: bool,
        summary: String,
    },
    /// Escrita proposta: aguarda aprovação na fila (`ai.approve`).
    Proposal {
        proposal_id: Id,
        tool: String,
        command: String,
    },
    /// Tokens e custo estimado de uma chamada.
    Usage {
        engine_id: Id,
        #[ts(type = "number")]
        input_tokens: i64,
        #[ts(type = "number")]
        output_tokens: i64,
        #[ts(type = "number")]
        cost_micros: i64,
        currency: String,
        priced: bool,
    },
    /// O motor falhou (ou está sem chave/cota) e o próximo da rota vai tentar.
    Fallback {
        engine_id: Id,
        engine_name: String,
        reason: String,
    },
    /// Pergunta do agente à pessoa (`ask_user`), à espera de `ai.question.answer`.
    UserQuestion {
        request_id: Id,
        question: super::ask::Question,
    },
    /// A pergunta foi respondida (`answer`) ou ficou sem resposta (`None`: tempo esgotado ou
    /// conversa cancelada).
    UserAnswer {
        request_id: Id,
        answer: Option<super::ask::Answer>,
    },
    /// Ferramenta de um MCP extra à espera da confirmação da pessoa (`ai.mcp_extra.decide`).
    ExternalApproval {
        request_id: Id,
        /// Rótulo do servidor.
        server: String,
        /// Nome original da ferramenta.
        tool: String,
        /// Parâmetros resumidos.
        args: String,
    },
    /// A confirmação foi respondida (ou expirou): `allowed` falso quando recusou ou expirou.
    ExternalDecision {
        request_id: Id,
        allowed: bool,
        /// Permitido no resto da conversa (as próximas chamadas do servidor passam direto).
        remembered: bool,
    },
    /// Etapa de rotina (texto curto).
    Progress {
        message: String,
    },
    Error {
        code: String,
        message: String,
    },
    /// Fim do job.
    Done {
        status: JobStatus,
        conversation_id: Option<Id>,
        message_id: Option<Id>,
        alert_id: Option<Id>,
    },
}

#[derive(Debug)]
struct JobState {
    events: Vec<JobEvent>,
    status: JobStatus,
    conversation_id: Option<Id>,
    finished_at: Option<Instant>,
    finished_utc: Option<String>,
    engine_name: Option<String>,
    /// Do que se trata (início da mensagem, nome da rotina), para a lista de tarefas.
    title: Option<String>,
    /// Etapa atual (ferramenta chamada, progresso da rotina, pedido ao usuário).
    activity: Option<String>,
    /// Onde abrir o resultado (ex.: o estudo com IA gravado).
    link: Option<JobLink>,
}

/// Tela do resultado de um job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum JobScreen {
    /// Ativo › Estudo (IA), com o estudo aberto.
    AssetStudy,
}

/// Onde abrir o resultado de um job (a lista de tarefas mostra "Abrir").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobLink {
    pub screen: JobScreen,
    pub asset_id: Option<Id>,
    pub id: Option<Id>,
}

#[derive(Debug)]
pub struct Job {
    pub id: Id,
    pub kind: JobKind,
    pub task: AiTask,
    pub created_at: String,
    cancel: AtomicBool,
    state: Mutex<JobState>,
    /// Início e tempo-limite de uma execução por CLI (para o aviso de tempo ao agente).
    deadline: Mutex<Option<(std::time::Instant, std::time::Duration)>>,
}

/// A partir de quanto tempo restante o agente é avisado (o maior entre os dois).
const TIME_WARNING_MIN: std::time::Duration = std::time::Duration::from_secs(4 * 60);
const TIME_WARNING_SHARE: f64 = 0.2;

impl Job {
    fn new(kind: JobKind, task: AiTask, conversation_id: Option<Id>) -> Self {
        Self {
            id: new_id(),
            kind,
            task,
            created_at: now_utc(),
            cancel: AtomicBool::new(false),
            deadline: Mutex::new(None),
            state: Mutex::new(JobState {
                events: vec![],
                status: JobStatus::Running,
                conversation_id,
                finished_at: None,
                finished_utc: None,
                engine_name: None,
                title: None,
                activity: None,
                link: None,
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, JobState> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn push(&self, event: JobEventKind) {
        let mut st = self.lock();
        if st.status != JobStatus::Running && !matches!(event, JobEventKind::Done { .. }) {
            return;
        }
        if let JobEventKind::Engine { engine_name, .. } = &event {
            st.engine_name = Some(engine_name.clone());
        }
        let activity = match &event {
            JobEventKind::ToolCall { name, summary } => Some(if summary.trim().is_empty() {
                format!("Usando {name}")
            } else {
                summary.clone()
            }),
            JobEventKind::Progress { message } => Some(message.clone()),
            JobEventKind::Proposal { .. } => Some("Proposta aguardando aprovação".into()),
            JobEventKind::Fallback { engine_name, .. } => {
                Some(format!("Tentando outro motor ({engine_name})"))
            }
            JobEventKind::Text { .. } => Some("Escrevendo a resposta".into()),
            JobEventKind::Reasoning { .. } => Some("Pensando".into()),
            _ => None,
        };
        if let Some(a) = activity {
            st.activity = Some(a.chars().take(160).collect());
        }
        // Texto parcial seguido: junta no último evento para não crescer sem limite.
        if st.events.len() >= MAX_EVENTS / 2 {
            match (&event, st.events.last_mut().map(|e| &mut e.event)) {
                (JobEventKind::Text { delta }, Some(JobEventKind::Text { delta: last }))
                | (
                    JobEventKind::Reasoning { delta },
                    Some(JobEventKind::Reasoning { delta: last }),
                ) => {
                    last.push_str(delta);
                    return;
                }
                _ => {}
            }
        }
        if st.events.len() >= MAX_EVENTS {
            return;
        }
        let seq = st.events.len() as u64 + 1;
        st.events.push(JobEvent {
            seq,
            at: now_utc(),
            event,
        });
    }

    pub fn set_conversation(&self, id: &str) {
        self.lock().conversation_id = Some(id.to_string());
    }

    /// Marca o começo de uma execução com tempo-limite (CLI como motor).
    pub fn set_deadline(&self, limit: std::time::Duration) {
        *self.deadline.lock().unwrap_or_else(|p| p.into_inner()) =
            Some((std::time::Instant::now(), limit));
    }

    /// Aviso de tempo para o agente quando falta pouco para o tempo-limite (anexado ao
    /// resultado das ferramentas do Treasury): o bastante para ele guardar o que coletou e
    /// fechar com um resumo. `None` longe do fim ou sem tempo-limite.
    pub fn time_notice(&self) -> Option<String> {
        let (start, limit) = (*self.deadline.lock().unwrap_or_else(|p| p.into_inner()))?;
        let left = limit.saturating_sub(start.elapsed());
        let threshold = TIME_WARNING_MIN.max(limit.mul_f64(TIME_WARNING_SHARE));
        if left > threshold {
            return None;
        }
        let minutes = left.as_secs().div_ceil(60);
        Some(format!(
            "[Treasury] Time warning: this conversation is cut off in about {minutes} min. Stop \
             exploring now: save what you collected (propose the records, or `remember` a short \
             note with the data and where you stopped) and finish with a summary of what is done \
             and what is missing, so the user can continue in a new message."
        ))
    }

    pub fn conversation_id(&self) -> Option<Id> {
        self.lock().conversation_id.clone()
    }

    pub fn engine_name(&self) -> Option<String> {
        self.lock().engine_name.clone()
    }

    /// Título da tarefa (uma linha, até 120 caracteres).
    /// Resultado para abrir pela lista de tarefas.
    pub fn set_link(&self, link: JobLink) {
        self.lock().link = Some(link);
    }

    pub fn set_title(&self, title: &str) {
        let one = title.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut t: String = one.chars().take(120).collect();
        if one.chars().count() > 120 {
            t.push('…');
        }
        self.lock().title = Some(t).filter(|t| !t.is_empty());
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn status(&self) -> JobStatus {
        self.lock().status
    }

    /// Encerra o job (uma vez) com o evento final.
    pub fn finish(&self, status: JobStatus, message_id: Option<Id>, alert_id: Option<Id>) {
        let conversation_id = {
            let st = self.lock();
            if st.status != JobStatus::Running {
                return;
            }
            st.conversation_id.clone()
        };
        self.push(JobEventKind::Done {
            status,
            conversation_id,
            message_id,
            alert_id,
        });
        let mut st = self.lock();
        st.status = status;
        st.finished_at = Some(Instant::now());
        st.finished_utc = Some(now_utc());
    }

    pub fn last_seq(&self) -> u64 {
        self.lock().events.last().map_or(0, |e| e.seq)
    }

    /// Alguma ferramenta foi chamada depois de `seq`.
    pub fn tool_calls_since(&self, seq: u64) -> bool {
        self.lock()
            .events
            .iter()
            .any(|e| e.seq > seq && matches!(e.event, JobEventKind::ToolCall { .. }))
    }

    pub fn events_after(&self, after: u64) -> Vec<JobEvent> {
        self.lock()
            .events
            .iter()
            .filter(|e| e.seq > after)
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobInfo {
    pub job_id: Id,
    pub kind: JobKind,
    pub task: AiTask,
    pub status: JobStatus,
    pub created_at: String,
    pub conversation_id: Option<Id>,
    pub engine_name: Option<String>,
    /// Do que se trata (início da mensagem, nome da rotina).
    pub title: Option<String>,
    /// Etapa atual ou a última.
    pub activity: Option<String>,
    pub finished_at: Option<String>,
    /// Cancelamento pedido e ainda não concluído.
    pub cancelling: bool,
    /// Onde abrir o resultado, quando há um (ex.: o estudo com IA).
    pub link: Option<JobLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatPollArgs {
    pub job_id: Id,
    /// Último `seq` já recebido (0 no começo).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub after_seq: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatPoll {
    pub job_id: Id,
    pub status: JobStatus,
    pub events: Vec<JobEvent>,
    /// Passe como `after_seq` na próxima consulta.
    #[ts(type = "number")]
    pub next_seq: u64,
    pub conversation_id: Option<Id>,
}

/// Token de sessão de uma CLI executada pelo app.
#[derive(Debug, Clone)]
pub struct AgentGrant {
    pub client: String,
    pub job_id: Id,
}

#[derive(Default)]
pub struct Runtime {
    this: OnceLock<Weak<Api>>,
    jobs: Mutex<HashMap<Id, Arc<Job>>>,
    grants: Mutex<HashMap<String, AgentGrant>>,
    mcp_command: Mutex<Option<PathBuf>>,
    /// Instante do último `run_due`, para devolver os alertas novos desde então.
    pub(crate) last_alert_check: Mutex<Option<String>>,
    /// Anexos preparados e ainda não enviados (só na memória; ver `ai::attachments`).
    pub attachments: super::attachments::Staging,
    /// Anexos por conversa (ou job), só na memória enquanto o cofre está aberto: os do
    /// turno em andamento e os de turnos anteriores da mesma conversa. Servem às
    /// ferramentas que leem o arquivo (importar a nota anexada, `read_attachment`) e ao
    /// histórico (o anexo volta ao motor sem o usuário anexar de novo). Somem ao bloquear o
    /// cofre; os de um job sem conversa, quando o job termina.
    /// Ordem de uso: a conversa usada por último fica no fim.
    turn_files: Mutex<Vec<(String, ConversationFiles)>>,
    /// Confirmações de ferramentas de MCPs extras à espera da pessoa: id do pedido → (job,
    /// decisão). `None` enquanto ninguém respondeu.
    external: Mutex<HashMap<Id, ExternalPending>>,
    /// Perguntas do agente à espera da pessoa: id do pedido → (formulário, resposta).
    questions: Mutex<HashMap<Id, (super::ask::Question, Option<super::ask::Answer>)>>,
    /// Servidores de MCPs extras que a pessoa liberou na conversa inteira ("Permitir nesta
    /// conversa"): conversa (ou job sem conversa) → ids dos servidores. Só na memória: some ao
    /// fechar o app ou bloquear o cofre.
    external_grants: Mutex<HashMap<String, std::collections::HashSet<Id>>>,
    /// MCPs extras abertos para um job de CLI: o servidor MCP do Treasury oferece as
    /// ferramentas deles à CLI e as chama por aqui (com a confirmação na conversa).
    extras: Mutex<HashMap<Id, Arc<Mutex<super::mcp_extra::Toolbox>>>>,
}

/// Um pedido de confirmação de ferramenta externa à espera da pessoa.
struct ExternalPending {
    decision: Option<bool>,
    remember: bool,
}

/// Quanto o agente espera a pessoa confirmar uma ferramenta externa antes de desistir.
pub const EXTERNAL_APPROVAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// Anexos de uma conversa na memória: todos (os mais novos no fim) e os do último turno.
#[derive(Default)]
struct ConversationFiles {
    all: Vec<Arc<super::attachments::Attachment>>,
    last_turn: Vec<Arc<super::attachments::Attachment>>,
}

/// Anexos guardados na memória por conversa (os mais antigos saem primeiro).
const MAX_CONVERSATION_FILES: usize = 30;
const MAX_CONVERSATION_BYTES: usize = 150 * 1024 * 1024;
/// Conversas com anexos na memória (as menos recentes saem primeiro).
const MAX_CONVERSATIONS_WITH_FILES: usize = 8;

impl Runtime {
    /// Começa um turno: os anexos dele passam a ser os "do turno" e se somam aos que a
    /// conversa já tinha na memória (sem repetir; os mais antigos saem acima dos limites).
    pub(crate) fn set_turn_files(
        &self,
        key: &str,
        files: Vec<Arc<super::attachments::Attachment>>,
    ) {
        let mut g = self.turn_files.lock().unwrap_or_else(|p| p.into_inner());
        let pos = g.iter().position(|(k, _)| k == key);
        if files.is_empty() && pos.is_none() {
            return;
        }
        // Conversa usada agora vai para o fim da fila.
        let (_, mut entry) = match pos {
            Some(i) => g.remove(i),
            None => (key.to_string(), ConversationFiles::default()),
        };
        for f in &files {
            entry.all.retain(|a| a.id != f.id);
            entry.all.push(f.clone());
        }
        entry.last_turn = files;
        while entry.all.len() > MAX_CONVERSATION_FILES
            || (entry.all.len() > 1
                && entry.all.iter().map(|a| a.data.len()).sum::<usize>() > MAX_CONVERSATION_BYTES)
        {
            let old = entry.all.remove(0);
            entry.last_turn.retain(|a| a.id != old.id);
        }
        while g.len() >= MAX_CONVERSATIONS_WITH_FILES {
            g.remove(0);
        }
        g.push((key.to_string(), entry));
    }

    /// Esquece os anexos de uma conversa (ou de um job sem conversa).
    pub(crate) fn drop_turn_files(&self, key: &str) {
        self.turn_files
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retain(|(k, _)| k != key);
    }

    fn with_files<T>(
        &self,
        key: &str,
        f: impl FnOnce(&ConversationFiles) -> Option<T>,
    ) -> Option<T> {
        let g = self.turn_files.lock().unwrap_or_else(|p| p.into_inner());
        g.iter().find(|(k, _)| k == key).and_then(|(_, c)| f(c))
    }

    /// Anexo pelo nome (sem diferenciar maiúsculas; o mais recente com esse nome), pelo id
    /// ou pela ordem no turno atual (`1`, `2`…).
    pub(crate) fn turn_file(
        &self,
        key: &str,
        name_or_n: &str,
    ) -> Option<Arc<super::attachments::Attachment>> {
        let want = name_or_n.trim();
        self.with_files(key, |files| {
            // A ordem: no último turno quando ele tem anexos; senão, na conversa toda (a
            // CLI manda os anexos num turno e chama a ferramenta noutro).
            if let Ok(n) = want.parse::<usize>() {
                let i = n.checked_sub(1)?;
                return if files.last_turn.is_empty() {
                    files.all.get(i).cloned()
                } else {
                    files.last_turn.get(i).cloned()
                };
            }
            files
                .all
                .iter()
                .rev()
                .find(|a| a.id == want || a.name.eq_ignore_ascii_case(want))
                .or_else(|| {
                    // Id encurtado pelo modelo (o começo do UUID), se só um anexo bate.
                    let mut hits = files
                        .all
                        .iter()
                        .filter(|a| want.len() >= 6 && a.id.starts_with(want));
                    let first = hits.next()?;
                    hits.next().is_none().then_some(first)
                })
                .cloned()
        })
    }

    /// Anexos da conversa na memória: (ordem no último turno, se estiver nele; o anexo).
    pub(crate) fn conversation_files(
        &self,
        key: &str,
    ) -> Vec<(Option<usize>, Arc<super::attachments::Attachment>)> {
        self.with_files(key, |f| {
            Some(
                f.all
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let n = if f.last_turn.is_empty() {
                            Some(i + 1)
                        } else {
                            f.last_turn.iter().position(|x| x.id == a.id).map(|i| i + 1)
                        };
                        (n, a.clone())
                    })
                    .collect(),
            )
        })
        .unwrap_or_default()
    }

    /// Ids dos anexos da conversa que estão na memória.
    pub(crate) fn conversation_file_ids(&self, key: &str) -> Vec<String> {
        self.with_files(key, |f| Some(f.all.iter().map(|a| a.id.clone()).collect()))
            .unwrap_or_default()
    }

    /// Anexo da conversa na memória pelo id (para o histórico).
    pub(crate) fn conversation_file(
        &self,
        key: &str,
        id: &str,
    ) -> Option<Arc<super::attachments::Attachment>> {
        self.with_files(key, |files| files.all.iter().find(|a| a.id == id).cloned())
    }

    /// Guarda uma referência fraca à `Api` compartilhada (as threads dos jobs precisam
    /// dela). Chamado por `Api::into_shared`, pelo socket do MCP e pelas rotinas.
    pub fn attach(api: &SharedApi) {
        let _ = api.ai_runtime().this.set(Arc::downgrade(api));
    }

    pub fn shared(&self) -> Result<SharedApi> {
        self.this.get().and_then(Weak::upgrade).ok_or_else(|| {
            Error::Other(
                "o agente precisa da API compartilhada (use Api::into_shared ao iniciar)".into(),
            )
        })
    }

    /// Executável do `treasury mcp` passado às CLIs (padrão: o executável em execução).
    pub fn mcp_command(&self) -> Result<PathBuf> {
        match self
            .mcp_command
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
        {
            Some(p) => Ok(p),
            None => Ok(std::env::current_exe()?),
        }
    }

    /// Troca o executável do `treasury mcp` (testes, `treasury-dev`).
    pub fn set_mcp_command(&self, path: Option<PathBuf>) {
        *self.mcp_command.lock().unwrap_or_else(|p| p.into_inner()) = path;
    }

    fn prune(&self) {
        let mut jobs = self.jobs.lock().unwrap_or_else(|p| p.into_inner());
        jobs.retain(|_, j| {
            j.lock()
                .finished_at
                .is_none_or(|t| t.elapsed() < KEEP_FINISHED)
        });
    }

    pub(crate) fn new_job(
        &self,
        kind: JobKind,
        task: AiTask,
        conversation_id: Option<Id>,
    ) -> Arc<Job> {
        self.prune();
        let job = Arc::new(Job::new(kind, task, conversation_id));
        self.jobs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(job.id.clone(), job.clone());
        job
    }

    pub fn job(&self, id: &str) -> Option<Arc<Job>> {
        self.jobs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(id)
            .cloned()
    }

    pub fn list(&self) -> Vec<JobInfo> {
        self.prune();
        let jobs = self.jobs.lock().unwrap_or_else(|p| p.into_inner());
        let mut out: Vec<JobInfo> = jobs
            .values()
            .map(|j| {
                let st = j.lock();
                JobInfo {
                    job_id: j.id.clone(),
                    kind: j.kind,
                    task: j.task,
                    status: st.status,
                    created_at: j.created_at.clone(),
                    conversation_id: st.conversation_id.clone(),
                    engine_name: st.engine_name.clone(),
                    title: st.title.clone(),
                    activity: st.activity.clone(),
                    finished_at: st.finished_utc.clone(),
                    cancelling: st.status == JobStatus::Running && j.is_cancelled(),
                    link: st.link.clone(),
                }
            })
            .collect();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        out
    }

    /// Rotina com job em andamento (não roda duas vezes ao mesmo tempo).
    pub(crate) fn running_task(&self, kind: JobKind, task: AiTask) -> bool {
        self.jobs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .values()
            .any(|j| j.kind == kind && j.task == task && j.status() == JobStatus::Running)
    }

    pub fn poll(&self, a: &ChatPollArgs) -> Result<ChatPoll> {
        let job = self
            .job(&a.job_id)
            .ok_or_else(|| Error::not_found(crate::i18n::t("nucleo-ia-job-expirou")))?;
        let events = job.events_after(a.after_seq.unwrap_or(0));
        let next_seq = events
            .last()
            .map(|e| e.seq)
            .unwrap_or(a.after_seq.unwrap_or(0));
        Ok(ChatPoll {
            job_id: job.id.clone(),
            status: job.status(),
            events,
            next_seq,
            conversation_id: job.conversation_id(),
        })
    }

    pub fn cancel(&self, id: &str) -> Result<()> {
        let job = self.job(id).ok_or_else(|| Error::not_found("job"))?;
        job.cancel();
        Ok(())
    }

    /// Cancela tudo (ao bloquear o cofre: nenhuma CLI continua rodando).
    pub fn cancel_all(&self) {
        for j in self.jobs.lock().unwrap_or_else(|p| p.into_inner()).values() {
            j.cancel();
        }
        self.grants
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
        // Anexos não enviados e os das conversas também somem (nunca ficam fora do cofre
        // aberto).
        self.attachments.clear();
        self.turn_files
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
    }

    /// Registra um token de sessão para a CLI de um job.
    pub(crate) fn grant(&self, client: String, job_id: &str) -> Result<String> {
        let mut raw = [0u8; 32];
        getrandom::fill(&mut raw).map_err(|e| Error::Crypto(e.to_string()))?;
        let token = hex::encode(raw);
        self.grants
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(
                token.clone(),
                AgentGrant {
                    client,
                    job_id: job_id.to_string(),
                },
            );
        Ok(token)
    }

    pub(crate) fn revoke(&self, token: &str) {
        self.grants
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(token);
    }

    /// Token de agente válido (job ainda em andamento).
    pub fn grant_for(&self, token: &str) -> Option<AgentGrant> {
        let g = self
            .grants
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(token)
            .cloned()?;
        let running = self
            .job(&g.job_id)
            .is_some_and(|j| j.status() == JobStatus::Running && !j.is_cancelled());
        running.then_some(g)
    }

    /// Pede à pessoa, na conversa, a confirmação de uma ferramenta de um MCP extra e espera a
    /// resposta (o job roda na própria thread). Servidor já liberado nesta conversa passa
    /// direto. `false` se recusou, se o tempo acabou ou se o job foi cancelado.
    pub fn ask_external(
        &self,
        job: &Job,
        server_id: &str,
        server: &str,
        tool: &str,
        args: &str,
    ) -> bool {
        let scope = job.conversation_id().unwrap_or_else(|| job.id.clone());
        if self
            .external_grants
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&scope)
            .is_some_and(|g| g.contains(server_id))
        {
            return true;
        }
        let request_id = crate::ids::new_id();
        self.external
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(
                request_id.clone(),
                ExternalPending {
                    decision: None,
                    remember: false,
                },
            );
        job.push(JobEventKind::ExternalApproval {
            request_id: request_id.clone(),
            server: server.into(),
            tool: tool.into(),
            args: args.into(),
        });
        let start = std::time::Instant::now();
        let (decision, remember) = loop {
            if let Some(p) = self
                .external
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .get(&request_id)
                && let Some(d) = p.decision
            {
                break (d, p.remember);
            }
            if job.is_cancelled() || start.elapsed() > EXTERNAL_APPROVAL_TIMEOUT {
                break (false, false);
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        };
        self.external
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&request_id);
        if decision && remember {
            self.external_grants
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .entry(scope)
                .or_default()
                .insert(server_id.to_string());
        }
        job.push(JobEventKind::ExternalDecision {
            request_id,
            allowed: decision,
            remembered: decision && remember,
        });
        decision
    }

    /// Mostra a pergunta do agente na conversa e espera a resposta (o job roda na própria
    /// thread). `None` se o tempo acabou ou a conversa foi cancelada.
    pub fn ask_user(&self, job: &Job, q: &super::ask::Question) -> Option<super::ask::Answer> {
        let request_id = crate::ids::new_id();
        self.questions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(request_id.clone(), (q.clone(), None));
        job.push(JobEventKind::UserQuestion {
            request_id: request_id.clone(),
            question: q.clone(),
        });
        let start = std::time::Instant::now();
        let answer = loop {
            if let Some((_, Some(a))) = self
                .questions
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .get(&request_id)
            {
                break Some(a.clone());
            }
            if job.is_cancelled() || start.elapsed() > super::ask::TIMEOUT {
                break None;
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        };
        self.questions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&request_id);
        job.push(JobEventKind::UserAnswer {
            request_id,
            answer: answer.clone(),
        });
        answer
    }

    /// A resposta da pessoa ao formulário (`ai.question.answer`), conferida contra ele.
    pub fn answer_question(
        &self,
        request_id: &str,
        answer: &super::ask::Answer,
    ) -> crate::error::Result<()> {
        let mut g = self.questions.lock().unwrap_or_else(|p| p.into_inner());
        match g.get_mut(request_id) {
            Some((q, slot)) if slot.is_none() => {
                let a = super::ask::check(q, answer).map_err(|e| {
                    crate::error::Error::invalid(crate::i18n::t_args(
                        "nucleo-ia-pergunta-resposta-invalida",
                        &crate::i18n::args(&[("erro", e.into())]),
                    ))
                })?;
                *slot = Some(a);
                Ok(())
            }
            _ => Err(crate::error::Error::NotFound(crate::i18n::t(
                "nucleo-ia-pergunta-expirada",
            ))),
        }
    }

    /// Tira as liberações "nesta conversa" (ao bloquear o cofre ou apagar a conversa).
    pub fn forget_external_grants(&self, conversation: Option<&str>) {
        let mut g = self
            .external_grants
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        match conversation {
            Some(c) => {
                g.remove(c);
            }
            None => g.clear(),
        }
    }

    /// Guarda os MCPs extras abertos para o job de uma CLI (até [`Self::drop_extras`]).
    pub(crate) fn set_extras(&self, job_id: &str, tb: super::mcp_extra::Toolbox) {
        self.extras
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(job_id.to_string(), Arc::new(Mutex::new(tb)));
    }

    pub(crate) fn extras(&self, job_id: &str) -> Option<Arc<Mutex<super::mcp_extra::Toolbox>>> {
        self.extras
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(job_id)
            .cloned()
    }

    /// Fecha as sessões dos MCPs extras do job (processos filhos e conexões HTTP).
    pub(crate) fn drop_extras(&self, job_id: &str) {
        let tb = self
            .extras
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(job_id);
        drop(tb);
    }

    /// A resposta da pessoa a um pedido de confirmação (`ai.mcp_extra.decide`). `remember`:
    /// permitir este servidor no resto da conversa, sem perguntar de novo.
    pub fn decide_external(
        &self,
        request_id: &str,
        allow: bool,
        remember: bool,
    ) -> crate::error::Result<()> {
        let mut g = self.external.lock().unwrap_or_else(|p| p.into_inner());
        match g.get_mut(request_id) {
            Some(p) if p.decision.is_none() => {
                p.decision = Some(allow);
                p.remember = allow && remember;
                Ok(())
            }
            _ => Err(crate::error::Error::NotFound(crate::i18n::t(
                "nucleo-mcpx-erro-pedido",
            ))),
        }
    }

    /// Uma ferramenta foi chamada para o job (agente interno ou CLI pelo socket).
    pub fn note_tool(&self, job_id: &str, note: ToolNote) {
        let Some(job) = self.job(job_id) else {
            return;
        };
        push_tool(&job, &note);
    }
}

/// Eventos de uma chamada de ferramenta.
pub(crate) fn push_tool(job: &Job, note: &ToolNote) {
    job.push(JobEventKind::ToolCall {
        name: note.name.clone(),
        summary: note.args.clone(),
    });
    job.push(JobEventKind::ToolResult {
        name: note.name.clone(),
        ok: note.ok,
        summary: note.summary.clone(),
    });
    if let Some(p) = &note.proposal_id {
        job.push(JobEventKind::Proposal {
            proposal_id: p.clone(),
            tool: note.name.clone(),
            command: note.command.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_tool_waits_for_the_user_and_cancel_declines() {
        let rt = std::sync::Arc::new(Runtime::default());
        let job = rt.new_job(JobKind::Chat, AiTask::Chat, None);
        let (rt2, job2) = (rt.clone(), job.clone());
        let t = std::thread::spawn(move || {
            rt2.ask_external(&job2, "srv", "Celular", "abrir_app", "{}")
        });
        // The request shows up in the events; the person allows it.
        let request_id = loop {
            let pending: Vec<Id> = rt.external.lock().unwrap().keys().cloned().collect();
            if let Some(id) = pending.first() {
                break id.clone();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        rt.decide_external(&request_id, true, false).unwrap();
        assert!(t.join().unwrap());
        assert!(
            rt.decide_external(&request_id, false, false).is_err(),
            "already answered"
        );
        // Cancelling the conversation declines what was waiting.
        let (rt3, job3) = (rt.clone(), job.clone());
        let t =
            std::thread::spawn(move || rt3.ask_external(&job3, "srv", "Celular", "ligar", "{}"));
        while rt.external.lock().unwrap().is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        rt.cancel(&job.id).unwrap();
        assert!(!t.join().unwrap());
    }

    #[test]
    fn allowing_in_the_conversation_skips_the_next_asks_for_that_server() {
        let rt = std::sync::Arc::new(Runtime::default());
        let job = rt.new_job(JobKind::Chat, AiTask::Chat, Some("conv-1".into()));
        let wait_pending = |rt: &Runtime| loop {
            let id = rt.external.lock().unwrap().keys().next().cloned();
            if let Some(id) = id {
                break id;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        let (rt2, job2) = (rt.clone(), job.clone());
        let t = std::thread::spawn(move || rt2.ask_external(&job2, "srv", "Celular", "tap", "{}"));
        let id = wait_pending(&rt);
        rt.decide_external(&id, true, true).unwrap();
        assert!(t.join().unwrap());
        assert!(job.events_after(0).iter().any(|e| matches!(
            e.event,
            JobEventKind::ExternalDecision {
                remembered: true,
                ..
            }
        )));
        // The next call to the same server in the same conversation goes straight through.
        assert!(rt.ask_external(&job, "srv", "Celular", "swipe", "{}"));
        assert!(rt.external.lock().unwrap().is_empty());
        // Another conversation (or another server) asks again.
        let other = rt.new_job(JobKind::Chat, AiTask::Chat, Some("conv-2".into()));
        let (rt3, other2) = (rt.clone(), other.clone());
        let t =
            std::thread::spawn(move || rt3.ask_external(&other2, "srv", "Celular", "tap", "{}"));
        let id = wait_pending(&rt);
        rt.decide_external(&id, false, true).unwrap();
        assert!(!t.join().unwrap(), "declining does not grant");
        // Locking the vault forgets the grants.
        rt.forget_external_grants(None);
        let (rt4, job4) = (rt.clone(), job.clone());
        let t = std::thread::spawn(move || rt4.ask_external(&job4, "srv", "Celular", "tap", "{}"));
        let id = wait_pending(&rt);
        rt.decide_external(&id, true, false).unwrap();
        assert!(t.join().unwrap());
    }

    #[test]
    fn job_list_shows_title_activity_and_cancelling() {
        let rt = Runtime::default();
        let job = rt.new_job(JobKind::Chat, AiTask::Chat, None);
        job.set_title("  Categorize   os lançamentos\nde outubro  ");
        job.push(JobEventKind::ToolCall {
            name: "query_transactions".into(),
            summary: "Lendo os lançamentos de outubro".into(),
        });
        let info = rt.list().remove(0);
        assert_eq!(
            info.title.as_deref(),
            Some("Categorize os lançamentos de outubro")
        );
        assert_eq!(
            info.activity.as_deref(),
            Some("Lendo os lançamentos de outubro")
        );
        assert_eq!(info.status, JobStatus::Running);
        assert!(!info.cancelling);

        rt.cancel(&job.id).unwrap();
        assert!(
            rt.list()[0].cancelling,
            "cancellation requested, still running"
        );
        job.finish(JobStatus::Cancelled, None, None);
        let info = rt.list().remove(0);
        assert_eq!(info.status, JobStatus::Cancelled);
        assert!(!info.cancelling && info.finished_at.is_some());

        let long = "x".repeat(300);
        job.set_title(&long);
        assert_eq!(rt.list()[0].title.as_ref().unwrap().chars().count(), 121);
    }
}
