//! Tarefas em segundo plano (docs/background-tasks.md): coletas e downloads longos
//! aparecem no topo da interface com a etapa, o progresso e o botão de cancelar.
//!
//! O registro fica em memória, no processo. Quem roda o trabalho abre uma [`Task`]; a alça
//! fica guardada na própria thread ([`run`]), para os laços de baixo marcarem o progresso
//! ([`detail`], [`advance`], [`set_total`]) e conferirem o cancelamento ([`cancelled`]) sem
//! receber parâmetro novo. Cancelar é cooperativo: vale na próxima fronteira (entre um
//! arquivo e outro, entre um ativo e outro), e o que já foi gravado fica.
//!
//! O início e o fim de cada tarefa vão para um canal de difusão ([`subscribe`]); o app
//! repassa à interface (`tasks://changed`). O progresso não avisa: a interface relê
//! `tasks.list` a cada segundo enquanto há tarefa rodando.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::ids::new_id;
use crate::time::now_utc;

/// O que a tarefa faz (a interface traduz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaskKind {
    /// Índices, câmbio, cotações (inclusive o histórico de cotas da CVM) e proventos.
    MarketSync,
    /// Só índices e câmbio.
    IndicesSync,
    /// Fundamentos (CVM, SEC, B3, IF.data) e o score do mercado.
    ResearchSync,
    /// E-mail de ingestão de uma conta.
    MailSync,
    /// Pacote da atualização do app.
    UpdateDownload,
}

/// Etapa atual (a interface traduz; o detalhe, em [`TaskInfo::detail`], é texto livre).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaskStage {
    Indices,
    Prices,
    Funds,
    Income,
    Fundamentals,
    Scores,
    Mail,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaskStatus {
    Running,
    Done,
    Failed,
    Cancelled,
}

/// Quem pediu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaskOrigin {
    /// Rotina do app (coleta automática).
    Auto,
    /// Botão da interface.
    User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskInfo {
    pub id: String,
    pub kind: TaskKind,
    pub origin: TaskOrigin,
    pub status: TaskStatus,
    pub stage: Option<TaskStage>,
    /// O que está sendo feito agora (arquivo, ativo), como veio da fonte.
    pub detail: Option<String>,
    /// Passos concluídos (arquivos, ativos, bytes).
    #[ts(type = "number")]
    pub done: u64,
    /// Total, quando se sabe: a barra é determinada só com ele.
    #[ts(type = "number | null")]
    pub total: Option<u64>,
    /// `done` e `total` são bytes (download).
    pub bytes: bool,
    pub cancellable: bool,
    /// Cancelamento pedido e ainda não atendido.
    pub cancelling: bool,
    /// UTC RFC 3339.
    pub started_at: String,
    pub finished_at: Option<String>,
    /// Motivo da falha (mensagem do erro).
    pub error: Option<String>,
}

/// Aviso de tarefa que começou, terminou ou recebeu pedido de cancelar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskEvent {
    pub id: String,
    pub kind: TaskKind,
    pub status: TaskStatus,
}

/// Tarefas terminadas ficam na lista por este tempo (como as do agente).
const KEEP_FINISHED: Duration = Duration::from_secs(30 * 60);
/// Coleta automática cancelada não volta antes disto.
pub const SNOOZE_AFTER_CANCEL: Duration = Duration::from_secs(60 * 60);
const MAX_FINISHED: usize = 20;

struct Entry {
    info: Mutex<TaskInfo>,
    cancel: AtomicBool,
    started: Instant,
    finished: Mutex<Option<Instant>>,
}

#[derive(Default)]
struct Registry {
    entries: BTreeMap<String, Arc<Entry>>,
    /// Último cancelamento de cada tipo (para a rotina não recomeçar na hora).
    cancelled_at: BTreeMap<TaskKindKey, Instant>,
}

type TaskKindKey = u8;

fn key(k: TaskKind) -> TaskKindKey {
    k as u8
}

fn registry() -> MutexGuard<'static, Registry> {
    static REG: OnceLock<Mutex<Registry>> = OnceLock::new();
    REG.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

fn bus() -> &'static broadcast::Sender<TaskEvent> {
    static BUS: OnceLock<broadcast::Sender<TaskEvent>> = OnceLock::new();
    BUS.get_or_init(|| broadcast::channel(64).0)
}

/// Novo ouvinte dos avisos de início e fim.
pub fn subscribe() -> broadcast::Receiver<TaskEvent> {
    bus().subscribe()
}

fn announce(info: &TaskInfo) {
    if bus().receiver_count() > 0 {
        let _ = bus().send(TaskEvent {
            id: info.id.clone(),
            kind: info.kind,
            status: info.status,
        });
    }
}

fn lock(e: &Entry) -> MutexGuard<'_, TaskInfo> {
    e.info.lock().unwrap_or_else(|p| p.into_inner())
}

/// Alça de uma tarefa em andamento. Ao sair de escopo sem [`Task::finish`], termina como
/// concluída (ou cancelada, se o cancelamento foi pedido).
pub struct Task {
    entry: Arc<Entry>,
    ended: bool,
}

impl Task {
    pub fn start(kind: TaskKind, origin: TaskOrigin, cancellable: bool) -> Task {
        let id = new_id();
        let info = TaskInfo {
            id: id.clone(),
            kind,
            origin,
            status: TaskStatus::Running,
            stage: None,
            detail: None,
            done: 0,
            total: None,
            bytes: false,
            cancellable,
            cancelling: false,
            started_at: now_utc(),
            finished_at: None,
            error: None,
        };
        announce(&info);
        let entry = Arc::new(Entry {
            info: Mutex::new(info),
            cancel: AtomicBool::new(false),
            started: Instant::now(),
            finished: Mutex::new(None),
        });
        let mut reg = registry();
        prune(&mut reg);
        reg.entries.insert(id, entry.clone());
        Task {
            entry,
            ended: false,
        }
    }

    pub fn id(&self) -> String {
        lock(&self.entry).id.clone()
    }

    /// Nova etapa: zera o detalhe e o progresso.
    pub fn stage(&self, stage: TaskStage) {
        let mut i = lock(&self.entry);
        i.stage = Some(stage);
        i.detail = None;
        i.done = 0;
        i.total = None;
        i.bytes = false;
    }

    pub fn detail(&self, text: impl Into<String>) {
        lock(&self.entry).detail = Some(text.into());
    }

    pub fn set_total(&self, total: Option<u64>) {
        lock(&self.entry).total = total;
    }

    pub fn advance(&self) {
        self.advance_by(1);
    }

    pub fn advance_by(&self, n: u64) {
        lock(&self.entry).done += n;
    }

    /// Progresso em bytes (download).
    pub fn bytes(&self, done: u64, total: Option<u64>) {
        let mut i = lock(&self.entry);
        i.bytes = true;
        i.done = done;
        i.total = total;
    }

    pub fn is_cancelled(&self) -> bool {
        self.entry.cancel.load(Ordering::Relaxed)
    }

    /// Termina com o resultado: erro vira falha (cancelamento, cancelada).
    pub fn finish<T>(mut self, res: &Result<T>) {
        let status = match res {
            Ok(_) if self.is_cancelled() => TaskStatus::Cancelled,
            Ok(_) => TaskStatus::Done,
            Err(Error::Cancelled) => TaskStatus::Cancelled,
            Err(_) => TaskStatus::Failed,
        };
        let error = match res {
            Err(e) if status == TaskStatus::Failed => Some(e.message()),
            _ => None,
        };
        self.end(status, error);
    }

    fn end(&mut self, status: TaskStatus, error: Option<String>) {
        if self.ended {
            return;
        }
        self.ended = true;
        let info = {
            let mut i = lock(&self.entry);
            i.status = status;
            i.error = error;
            i.cancelling = false;
            i.finished_at = Some(now_utc());
            i.clone()
        };
        *self
            .entry
            .finished
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(Instant::now());
        if status == TaskStatus::Cancelled && info.origin == TaskOrigin::Auto {
            registry()
                .cancelled_at
                .insert(key(info.kind), Instant::now());
        }
        tracing::debug!(
            "tarefa {:?} terminou: {:?} em {:?}",
            info.kind,
            status,
            self.entry.started.elapsed()
        );
        announce(&info);
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        let status = if std::thread::panicking() {
            TaskStatus::Failed
        } else if self.is_cancelled() {
            TaskStatus::Cancelled
        } else {
            TaskStatus::Done
        };
        self.end(status, None);
    }
}

fn prune(reg: &mut Registry) {
    let finished: Vec<(String, Instant)> = reg
        .entries
        .iter()
        .filter_map(|(id, e)| {
            let at = *e.finished.lock().unwrap_or_else(|p| p.into_inner());
            at.map(|at| (id.clone(), at))
        })
        .collect();
    let mut keep: Vec<(String, Instant)> = finished
        .iter()
        .filter(|(_, at)| at.elapsed() < KEEP_FINISHED)
        .cloned()
        .collect();
    keep.sort_by_key(|(_, at)| std::cmp::Reverse(*at));
    keep.truncate(MAX_FINISHED);
    for (id, _) in finished {
        if !keep.iter().any(|(k, _)| *k == id) {
            reg.entries.remove(&id);
        }
    }
}

/// Em andamento primeiro, depois as terminadas, das mais novas para as mais velhas.
pub fn list() -> Vec<TaskInfo> {
    let mut reg = registry();
    prune(&mut reg);
    let mut out: Vec<TaskInfo> = reg.entries.values().map(|e| lock(e).clone()).collect();
    out.sort_by(|a, b| {
        (a.status != TaskStatus::Running)
            .cmp(&(b.status != TaskStatus::Running))
            .then_with(|| b.started_at.cmp(&a.started_at))
    });
    out
}

/// Pede o cancelamento (atendido na próxima fronteira da tarefa).
pub fn cancel(id: &str) -> Result<TaskInfo> {
    let entry =
        registry().entries.get(id).cloned().ok_or_else(|| {
            Error::not_found(crate::i18n::t("nucleo-geral-tarefa-nao-encontrada"))
        })?;
    let info = {
        let mut i = lock(&entry);
        if i.status != TaskStatus::Running {
            return Ok(i.clone());
        }
        if !i.cancellable {
            return Err(Error::invalid(crate::i18n::t(
                "nucleo-geral-tarefa-nao-cancelavel",
            )));
        }
        i.cancelling = true;
        i.clone()
    };
    entry.cancel.store(true, Ordering::Relaxed);
    announce(&info);
    Ok(info)
}

/// A rotina automática deste tipo foi cancelada há pouco (não recomeça sozinha).
pub fn snoozed(kind: TaskKind) -> bool {
    registry()
        .cancelled_at
        .get(&key(kind))
        .is_some_and(|at| at.elapsed() < SNOOZE_AFTER_CANCEL)
}

thread_local! {
    static CURRENT: RefCell<Option<Arc<Entry>>> = const { RefCell::new(None) };
}

fn with_current(f: impl FnOnce(&Task)) {
    CURRENT.with(|c| {
        if let Some(entry) = c.borrow().as_ref() {
            let t = Task {
                entry: entry.clone(),
                ended: true,
            };
            f(&t);
        }
    });
}

/// Roda `f` como tarefa nesta thread. Dentro de outra tarefa, `f` roda nela mesma (sem
/// tarefa nova): a coleta de índices dentro da coleta geral é uma etapa, não outra tarefa.
pub fn run<T>(
    kind: TaskKind,
    origin: TaskOrigin,
    cancellable: bool,
    f: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if CURRENT.with(|c| c.borrow().is_some()) {
        return f();
    }
    let task = Task::start(kind, origin, cancellable);
    CURRENT.with(|c| *c.borrow_mut() = Some(task.entry.clone()));
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            CURRENT.with(|c| *c.borrow_mut() = None);
        }
    }
    let reset = Reset;
    let res = f();
    drop(reset);
    // Pedido de cancelar atendido com o que já tinha vindo gravado: termina como cancelada.
    let res = match res {
        Ok(_) if task.is_cancelled() => Err(Error::Cancelled),
        r => r,
    };
    task.finish(&res);
    res
}

/// Etapa da tarefa corrente desta thread (sem tarefa, nada).
pub fn stage(s: TaskStage) {
    with_current(|t| t.stage(s));
}

pub fn detail(text: impl Into<String>) {
    let text = text.into();
    with_current(|t| t.detail(text));
}

pub fn set_total(total: Option<u64>) {
    with_current(|t| t.set_total(total));
}

pub fn advance() {
    with_current(|t| t.advance());
}

pub fn advance_by(n: u64) {
    with_current(|t| t.advance_by(n));
}

/// Cancelamento pedido para a tarefa corrente desta thread.
pub fn cancelled() -> bool {
    let mut out = false;
    with_current(|t| out = t.is_cancelled());
    out
}

/// `Err(Error::Cancelled)` se o cancelamento foi pedido.
pub fn check() -> Result<()> {
    if cancelled() {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(id: &str) -> TaskInfo {
        list()
            .into_iter()
            .find(|t| t.id == id)
            .expect("tarefa na lista")
    }

    #[test]
    fn progress_is_listed_and_the_end_is_recorded() {
        let mut id = String::new();
        let out = run(TaskKind::IndicesSync, TaskOrigin::User, true, || {
            CURRENT.with(|c| id = lock(c.borrow().as_ref().unwrap()).id.clone());
            stage(TaskStage::Indices);
            set_total(Some(3));
            detail("IPCA");
            advance();
            let t = find(&id);
            assert_eq!(t.status, TaskStatus::Running);
            assert_eq!((t.done, t.total), (1, Some(3)));
            assert_eq!(t.detail.as_deref(), Some("IPCA"));
            assert_eq!(t.stage, Some(TaskStage::Indices));
            // Tarefa dentro de tarefa vira etapa da mesma.
            run(TaskKind::MarketSync, TaskOrigin::User, true, || Ok(()))?;
            Ok(7)
        })
        .unwrap();
        assert_eq!(out, 7);
        let t = find(&id);
        assert_eq!(t.status, TaskStatus::Done);
        assert!(t.finished_at.is_some());
        // Fora da tarefa, os marcadores não fazem nada.
        advance();
        assert!(!cancelled());
    }

    #[test]
    fn cancel_is_cooperative_and_snoozes_the_routine() {
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let (go, wait) = std::sync::mpsc::channel::<()>();
        let worker = std::thread::spawn(move || {
            run(TaskKind::ResearchSync, TaskOrigin::Auto, true, || {
                CURRENT.with(|c| {
                    tx.send(lock(c.borrow().as_ref().unwrap()).id.clone())
                        .unwrap()
                });
                wait.recv().unwrap();
                check()?;
                Ok(())
            })
        });
        let id = rx.recv().unwrap();
        let info = cancel(&id).unwrap();
        assert!(info.cancelling);
        go.send(()).unwrap();
        let res = worker.join().unwrap();
        assert!(matches!(res, Err(Error::Cancelled)));
        assert_eq!(find(&id).status, TaskStatus::Cancelled);
        assert!(snoozed(TaskKind::ResearchSync));
        assert!(cancel("nao-existe").is_err());
    }

    #[test]
    fn failures_keep_the_message_and_uncancellable_refuses() {
        let t = Task::start(TaskKind::UpdateDownload, TaskOrigin::Auto, false);
        let id = t.id();
        assert!(cancel(&id).is_err());
        t.bytes(512, Some(1024));
        assert!(find(&id).bytes);
        t.finish::<()>(&Err(Error::invalid("assinatura inválida")));
        let info = find(&id);
        assert_eq!(info.status, TaskStatus::Failed);
        assert_eq!(info.error.as_deref(), Some("assinatura inválida"));
    }
}
