//! API do núcleo: um despachante JSON `call(cmd, args)` usado pela interface (comando
//! Tauri `api_call`), pelo servidor de desenvolvimento e pelo servidor MCP.
//!
//! Comandos são `modulo.acao`. Cada módulo de domínio expõe `handle(ctx, acao, args)` e a
//! lista `READ_ONLY` das ações sem efeito colateral (as únicas liberadas para clientes MCP
//! somente leitura).

pub mod crud;
pub mod policy;
pub(crate) mod system;

use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::changes::{self, Author, Recorded};
use crate::error::{Error, Result};
use crate::i18n::{t, t_args};
use crate::paths::TreasuryPaths;
use crate::storage::Db;

pub use system::{
    AppInfo, DOWNLOADS_DELETE_SETTING, LOCK_ON_SLEEP_SETTING, VaultState, VaultStatus,
    WATCH_DOWNLOADS_SETTING,
};

/// Quem está chamando. A política de permissões da fase 4 se apoia nisto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    /// A interface do app (o usuário).
    User,
    /// Cliente MCP (agente interno, CLI local ou cliente externo).
    Mcp { client: String, read_only: bool },
    /// Rotinas internas (backup, recorrências).
    System,
    /// E-mail de ingestão (docs/email-ingest.md): o que uma regra aplica sozinha, com o
    /// autor "E-mail: <regra>" no histórico.
    Mail { rule: String },
}

impl Actor {
    pub fn author(&self) -> Author {
        match self {
            Actor::User => Author::user(),
            Actor::Mcp { client, .. } => Author::ai(client.clone()),
            Actor::System => Author::system("rotina"),
            Actor::Mail { rule } => Author::system(format!("E-mail: {rule}")),
        }
    }
}

/// Dados temporários da sessão desbloqueada, por módulo (ex.: prévias de importação).
#[derive(Default)]
pub struct Scratch {
    map: HashMap<&'static str, Box<dyn Any + Send>>,
}

impl Scratch {
    pub fn get_mut<T: Default + Send + 'static>(&mut self, module: &'static str) -> &mut T {
        let entry = self
            .map
            .entry(module)
            .or_insert_with(|| Box::new(T::default()));
        if !entry.is::<T>() {
            *entry = Box::new(T::default());
        }
        entry.downcast_mut::<T>().expect("tipo conferido acima")
    }
}

/// Sessão aberta: banco desbloqueado e estado de atividade.
pub struct Session {
    pub db: Db,
    pub last_activity: Instant,
    pub scratch: Scratch,
    /// Instante (UTC) do último backup feito nesta sessão, para o backup ao fechar.
    pub last_backup_at: Option<String>,
}

/// Contexto entregue aos módulos de domínio.
pub struct Ctx<'a> {
    pub db: &'a mut Db,
    pub paths: &'a TreasuryPaths,
    pub actor: &'a Actor,
    pub scratch: &'a mut Scratch,
    /// Comando em execução (`modulo.acao`), para a política de escrita da IA.
    pub cmd: &'a str,
}

/// Relatório de um comando em lote, entregue junto da proposta quando a escrita vira
/// proposta (o que foi criado, reaproveitado e pulado, com o motivo).
#[derive(Default)]
pub struct PendingReport(pub Option<Value>);

impl Ctx<'_> {
    pub fn conn(&self) -> &rusqlite::Connection {
        self.db.conn()
    }

    /// Guarda o relatório do lote para a resposta da proposta.
    pub fn set_report(&mut self, report: Value) {
        self.scratch.get_mut::<PendingReport>("pending_report").0 = Some(report);
    }

    /// Roda `f` numa transação desfeita no fim (prévia de um lote: nada fica gravado).
    pub fn dry_run<T>(
        &mut self,
        f: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T>,
    ) -> Result<T> {
        let tx = self.db.conn_mut().transaction()?;
        let out = f(&tx);
        tx.rollback()?;
        out
    }

    /// Escreve dentro de um conjunto de alterações com o autor do chamador.
    pub fn write<T>(
        &mut self,
        reason: &str,
        f: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T>,
    ) -> Result<Recorded<T>> {
        let author = self.actor.author();
        self.write_as(&author, reason, f)
    }

    /// Escreve com um autor específico (ex.: importação). Para clientes MCP/IA a escrita
    /// vira proposta pendente (salvo no modo autônomo, só para escritas reversíveis) e a
    /// chamada termina com [`Error::Proposed`].
    pub fn write_as<T>(
        &mut self,
        author: &Author,
        reason: &str,
        f: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T>,
    ) -> Result<Recorded<T>> {
        let Actor::Mcp { client, read_only } = self.actor else {
            // O e-mail de ingestão assina tudo com a regra; a importação continua importação.
            if let Actor::Mail { rule } = self.actor {
                let mail = match author.kind {
                    changes::AuthorKind::Import => Author::import(format!(
                        "E-mail: {rule} — {}",
                        author.detail.as_deref().unwrap_or("")
                    )),
                    _ => Author::system(format!("E-mail: {rule}")),
                };
                return changes::record(self.db.conn_mut(), &mail, reason, f);
            }
            return changes::record(self.db.conn_mut(), author, reason, f);
        };
        if *read_only {
            return Err(Error::Forbidden(t("nucleo-base-erro-cliente-leitura")));
        }
        // Memória de procedimentos do próprio agente e feedback ao desenvolvedor: direto, com
        // histórico e desfazer.
        if policy::is_agent_memory(self.cmd) || policy::is_dev_feedback(self.cmd) {
            return changes::record(self.db.conn_mut(), &Author::ai(client.clone()), reason, f);
        }
        let level =
            policy::level(self.cmd).ok_or_else(|| Error::Forbidden(not_for_ai(self.cmd)))?;
        // O que a IA leu de um e-mail nunca é aplicado sozinho: vira sempre proposta.
        let autonomous = level == policy::Level::Reversible
            && !client.starts_with(crate::mail::AI_CLIENT_PREFIX)
            && crate::settings::get_or(self.db.conn(), policy::AUTONOMOUS_SETTING, false)?;
        // Importação continua com o autor do lote (o detalhe já cita o cliente); o resto
        // fica com autoria da IA e o cliente no detalhe.
        let ai_author = if author.kind == changes::AuthorKind::Import {
            author.clone()
        } else {
            Author::ai(client.clone())
        };
        if autonomous {
            return changes::record(self.db.conn_mut(), &ai_author, reason, f);
        }
        let rec = changes::propose(self.db.conn_mut(), &ai_author, reason, f)?;
        let Some(cs) = rec.change_set_id else {
            return Ok(rec);
        };
        let pid = crate::ids::new_id();
        self.db.conn().execute(
            "INSERT INTO ai_proposal (id, change_set_id, client, command, level, status, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7)",
            rusqlite::params![pid, cs, client, self.cmd, level.as_str(), reason.trim(), crate::time::now_utc()],
        )?;
        Err(Error::Proposed {
            proposal_id: pid,
            change_set_id: cs,
        })
    }
}

pub(crate) enum State {
    Closed,
    Open(Box<Session>),
}

pub struct Api {
    /// Caminhos do cofre ativo (trocam com `vaults.select`, sempre com o cofre bloqueado).
    paths: RwLock<TreasuryPaths>,
    state: Mutex<State>,
    /// Trava exclusiva do processo (instância única).
    _lock: Option<File>,
    auto_lock: Mutex<Duration>,
    /// Jobs do agente, tokens de sessão das CLIs (ver `ai::jobs`).
    ai: crate::ai::jobs::Runtime,
}

pub type SharedApi = Arc<Api>;

pub const DEFAULT_AUTO_LOCK_MINUTES: u64 = 15;

impl Api {
    /// Cria a API e toma a trava exclusiva em `~/.treasury/run/treasury.lock`, para que
    /// nunca existam dois escritores.
    pub fn new(paths: TreasuryPaths) -> Result<Self> {
        paths.ensure_dirs()?;
        // Idioma da instalação (docs/i18n-design.md), antes de qualquer cofre.
        crate::i18n::set_current(crate::i18n::load(&paths.config_path));
        let lock = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(paths.lock_path())?;
        crate::paths::set_mode(&paths.lock_path(), 0o600)?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => return Err(Error::AlreadyOpen),
            Err(std::fs::TryLockError::Error(e)) => return Err(e.into()),
        }
        Ok(Self {
            paths: RwLock::new(paths),
            state: Mutex::new(State::Closed),
            _lock: Some(lock),
            auto_lock: Mutex::new(Duration::from_secs(DEFAULT_AUTO_LOCK_MINUTES * 60)),
            ai: Default::default(),
        })
    }

    /// Sem trava de processo (testes).
    pub fn new_unlocked_process(paths: TreasuryPaths) -> Result<Self> {
        paths.ensure_dirs()?;
        Ok(Self {
            paths: RwLock::new(paths),
            state: Mutex::new(State::Closed),
            _lock: None,
            auto_lock: Mutex::new(Duration::from_secs(DEFAULT_AUTO_LOCK_MINUTES * 60)),
            ai: Default::default(),
        })
    }

    /// API sobre um banco já aberto em modo leitura, sem trava de processo e sem as rotinas
    /// de desbloqueio (migração, backup, recorrências). Usada pelo `treasury mcp` com o app
    /// fechado: o app continua podendo abrir enquanto um cliente MCP externo está conectado.
    pub fn open_read_only(paths: TreasuryPaths, db: Db) -> Result<Self> {
        if !db.is_read_only() {
            return Err(Error::invalid(
                "open_read_only exige um banco aberto em modo leitura",
            ));
        }
        Ok(Self {
            paths: RwLock::new(paths),
            state: Mutex::new(State::Open(Box::new(Session {
                db,
                last_activity: Instant::now(),
                scratch: Scratch::default(),
                last_backup_at: None,
            }))),
            _lock: None,
            // Sem bloqueio automático: a sessão dura o processo do cliente MCP.
            auto_lock: Mutex::new(Duration::ZERO),
            ai: Default::default(),
        })
    }

    /// Caminhos do cofre ativo (uma cópia: trocam quando o cofre muda).
    pub fn paths(&self) -> TreasuryPaths {
        self.paths.read().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Troca o cofre ativo. Só com o cofre bloqueado (quem chama bloqueia antes).
    pub(crate) fn set_paths(&self, paths: TreasuryPaths) -> Result<()> {
        if self.is_unlocked() {
            return Err(Error::conflict(t("nucleo-base-erro-bloqueie-antes")));
        }
        paths.ensure_dirs()?;
        *self.paths.write().unwrap_or_else(|p| p.into_inner()) = paths;
        Ok(())
    }

    /// Embrulha numa `Arc` e registra a referência que os jobs do agente usam. Prefira isto
    /// a `Arc::new(api)`: sem o registro, `ai.chat_start` falha até o socket do MCP ou as
    /// rotinas registrarem a `Api` compartilhada.
    pub fn into_shared(self) -> SharedApi {
        let api = Arc::new(self);
        crate::ai::jobs::Runtime::attach(&api);
        api
    }

    /// Jobs do agente e tokens de sessão das CLIs.
    pub fn ai_runtime(&self) -> &crate::ai::jobs::Runtime {
        &self.ai
    }

    pub(crate) fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn auto_lock_after(&self) -> Duration {
        *self.auto_lock.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub(crate) fn set_auto_lock_after(&self, d: Duration) {
        *self.auto_lock.lock().unwrap_or_else(|p| p.into_inner()) = d;
    }

    pub fn is_unlocked(&self) -> bool {
        matches!(*self.state(), State::Open(_))
    }

    /// Chamada da interface (o usuário).
    pub fn call(&self, cmd: &str, args: Value) -> Result<Value> {
        self.call_as(&Actor::User, cmd, args)
    }

    pub fn call_as(&self, actor: &Actor, cmd: &str, args: Value) -> Result<Value> {
        let out = self.call_as_inner(actor, cmd, args);
        // Pedido de estudo aprovado: o app dispara o estudo (docs/asset-page.md).
        if out.is_ok() && cmd == "ai.approve" {
            crate::assets::study::start_requested(self);
        }
        // Telas abertas (e clientes SSE) ficam sabendo de toda escrita que deu certo.
        if out.is_ok()
            && let Some((module, action)) = cmd.split_once('.')
            && !is_read_only(module, action)
        {
            crate::events::after_command(actor, cmd);
        }
        out
    }

    fn call_as_inner(&self, actor: &Actor, cmd: &str, args: Value) -> Result<Value> {
        let (module, action) = cmd
            .split_once('.')
            .ok_or_else(|| Error::UnknownCommand(cmd.to_string()))?;
        if let Actor::Mcp { read_only, .. } = actor
            && !is_read_only(module, action)
        {
            if *read_only {
                return Err(Error::Forbidden(t_args(
                    "nucleo-base-erro-comando-leitura",
                    &crate::i18n::args(&[("comando", cmd.to_string().into())]),
                )));
            }
            // Escritores MCP: só os comandos da política, sempre como proposta (a memória
            // de procedimentos do agente grava direto; a ferramenta só existe para ele; o
            // feedback ao desenvolvedor também, só no modo desenvolvedor).
            if policy::level(cmd).is_none()
                && !policy::is_agent_memory(cmd)
                && !policy::is_dev_feedback(cmd)
                && !policy::is_own_proposal(cmd)
            {
                return Err(Error::Forbidden(not_for_ai(cmd)));
            }
        }
        // Comandos que funcionam com o cofre bloqueado.
        if let Some(res) = system::handle_unlocked(self, actor, module, action, &args) {
            let mut args = args;
            scrub_secrets(&mut args);
            return res;
        }
        let mut guard = self.state();
        let State::Open(session) = &mut *guard else {
            return Err(if self.paths().db_path.exists() {
                Error::Locked
            } else {
                Error::NoVault
            });
        };
        if *actor == Actor::User {
            session.last_activity = Instant::now();
        }
        let Session { db, scratch, .. } = &mut **session;
        let paths = self.paths();
        let mut ctx = Ctx {
            db,
            paths: &paths,
            actor,
            scratch,
            cmd,
        };
        let out = dispatch(&mut ctx, module, action, args);
        let report = scratch.get_mut::<PendingReport>("pending_report").0.take();
        match out {
            Err(Error::Proposed {
                proposal_id,
                change_set_id,
            }) => {
                let mut v = serde_json::json!({
                    "status": "pending_approval",
                    "proposal_id": proposal_id,
                    "change_set_id": change_set_id,
                    "message": t("nucleo-base-proposta-criada"),
                });
                if let Some(r) = report {
                    v["report"] = r;
                }
                Ok(v)
            }
            other => other,
        }
    }

    /// Executa uma função com acesso direto à sessão (rotinas internas, servidor MCP).
    pub fn with_session<T>(
        &self,
        f: impl FnOnce(&mut Session, &TreasuryPaths) -> Result<T>,
    ) -> Result<T> {
        let mut guard = self.state();
        match &mut *guard {
            State::Open(s) => f(s, &self.paths()),
            State::Closed => Err(if self.paths().db_path.exists() {
                Error::Locked
            } else {
                Error::NoVault
            }),
        }
    }

    /// O computador vai dormir, a tela foi bloqueada ou outra pessoa entrou na sessão (a
    /// casca do app avisa): bloqueia o cofre se o ajuste `lock_on_sleep` estiver ligado
    /// (padrão). Devolve se bloqueou; com o cofre já fechado, nada a fazer.
    pub fn lock_on_sleep(&self) -> Result<bool> {
        let Ok(on) = self
            .with_session(|s, _| crate::settings::get_or(s.db.conn(), LOCK_ON_SLEEP_SETTING, true))
        else {
            return Ok(false);
        };
        if !on {
            return Ok(false);
        }
        system::lock(self, true)?;
        Ok(true)
    }

    /// Chamado periodicamente (≈ a cada 30 s): bloqueio automático e backup diário.
    pub fn tick(&self) -> Result<TickOutcome> {
        let idle = match &*self.state() {
            State::Open(s) => {
                // Dia de uso, para o lembrete de apoio (fora do cofre, sem dado financeiro).
                if !s.db.is_read_only() {
                    crate::support::touch_today(&self.paths().root, crate::time::today_local());
                }
                // O ajuste pode ter mudado em `settings.set`.
                if let Ok(Some(m)) = crate::settings::get::<u64>(s.db.conn(), "auto_lock_minutes")
                    && !cfg!(test)
                {
                    self.set_auto_lock_after(Duration::from_secs(m * 60));
                }
                s.last_activity.elapsed()
            }
            State::Closed => return Ok(TickOutcome::default()),
        };
        let auto_lock = self.auto_lock_after();
        if !auto_lock.is_zero() && idle >= auto_lock {
            system::lock(self, true)?;
            return Ok(TickOutcome {
                locked: true,
                ..Default::default()
            });
        }
        // Dados dos ativos para completar: um alerta por episódio (a cada poucas horas).
        let _ = self.with_session(|s, _| {
            if s.db.is_read_only() {
                return Ok(None);
            }
            crate::assets::data_alert::check_due(s.db.conn())
        });
        let backed_up = self.with_session(|s, paths| {
            if crate::backup::daily_due(s.db.conn(), chrono::Local::now())? {
                crate::backup::create(
                    &s.db,
                    &paths.backups_dir,
                    crate::backup::BackupKind::AutoDaily,
                )?;
                s.last_backup_at = Some(crate::time::now_utc());
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(TickOutcome {
            locked: false,
            backup_created: backed_up,
        })
    }

    /// Ao fechar o app: backup se houve mudanças e bloqueio.
    pub fn shutdown(&self) -> Result<()> {
        system::lock(self, true)
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct TickOutcome {
    pub locked: bool,
    pub backup_created: bool,
}

/// Lê os argumentos de um comando. `null` vira o valor padrão do tipo, se houver.
/// Como [`parse_args`], mas lendo de um `&Value` (os comandos que funcionam com o cofre
/// bloqueado): os segredos vão direto para os campos [`crate::vault::Secret`] e o original é
/// zerado depois por [`scrub_secrets`].
pub fn parse_ref_args<T: DeserializeOwned>(args: &Value) -> Result<T> {
    let empty = Value::Object(Default::default());
    let src = if args.is_null() { &empty } else { args };
    T::deserialize(src).map_err(|e| Error::invalid(e.to_string()))
}

/// Campos com senha, PIN, código de recuperação ou segredo de chave física.
const SECRET_FIELDS: &[&str] = &[
    "password",
    "new_password",
    "current",
    "new",
    "pin",
    "code",
    "master_password",
    "backup_password",
    "shares",
    "hmac_secret",
];

/// Zera, no lugar, os textos dos campos secretos dos argumentos de um comando do cofre
/// (docs/local-security.md, item 6). O `Value` chegou pela IPC; sem isso, a senha ficaria
/// na memória liberada até ser reaproveitada.
pub(crate) fn scrub_secrets(v: &mut Value) {
    use zeroize::Zeroize;
    fn wipe(v: &mut Value) {
        match v {
            Value::String(s) => s.zeroize(),
            Value::Array(a) => a.iter_mut().for_each(wipe),
            _ => {}
        }
    }
    if let Value::Object(m) = v {
        for (k, val) in m.iter_mut() {
            if SECRET_FIELDS.contains(&k.as_str()) {
                wipe(val);
            }
        }
    }
}

pub fn parse_args<T: DeserializeOwned>(args: Value) -> Result<T> {
    let args = if args.is_null() {
        Value::Object(Default::default())
    } else {
        args
    };
    serde_json::from_value(args).map_err(|e| {
        Error::invalid(t_args(
            "nucleo-base-erro-argumentos",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        ))
    })
}

/// "<comando> não pode ser feito pela IA".
fn not_for_ai(cmd: &str) -> String {
    t_args(
        "nucleo-base-erro-nao-pela-ia",
        &crate::i18n::args(&[("comando", cmd.to_string().into())]),
    )
}

pub fn to_value<T: Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}

fn is_read_only(module: &str, action: &str) -> bool {
    let list: &[&str] = match module {
        "app" => &["info", "countries"],
        "tasks" => &["list"],
        "vault" => &["status"],
        "changes" => changes_api::READ_ONLY,
        "backup" => &["list", "health"],
        "settings" => &["get"],
        "household" => crate::household::api::READ_ONLY,
        "accounts" => crate::accounts::api::READ_ONLY,
        "attention" => crate::attention::api::READ_ONLY,
        "categories" => crate::categories::api::READ_ONLY,
        "transactions" => crate::transactions::api::READ_ONLY,
        "recurrences" => crate::recurrence::api::READ_ONLY,
        "cards" => crate::cards::api::READ_ONLY,
        "rules" => crate::rules::api::READ_ONLY,
        "import" => crate::import::api::READ_ONLY,
        "b3" => crate::import::b3::api::READ_ONLY,
        "notes" => crate::import::notes::api::READ_ONLY,
        "budget" => crate::budget::api::READ_ONLY,
        "goals" => crate::goals::api::READ_ONLY,
        "longevity" => crate::longevity::api::READ_ONLY,
        "demo" => crate::demo::api::READ_ONLY,
        "forecast" => crate::forecast::api::READ_ONLY,
        "notify" => crate::notify::api::READ_ONLY,
        "income" => crate::income::api::READ_ONLY,
        "receipts" => crate::receipts::READ_ONLY,
        "payments" => crate::payments::READ_ONLY,
        "scenarios" => crate::scenarios::api::READ_ONLY,
        "subscriptions" => crate::subscriptions::api::READ_ONLY,
        "installments" => crate::installments::api::READ_ONLY,
        "reports" => crate::reports::api::READ_ONLY,
        "fx" => crate::fx::api::READ_ONLY,
        "mcp" => crate::mcp::api::READ_ONLY,
        "export" => &[],
        "assets" => crate::assets::api::READ_ONLY,
        "portfolio" => crate::portfolio::api::READ_ONLY,
        "indices" => crate::indices::api::READ_ONLY,
        "market" => crate::market::api::READ_ONLY,
        "tax" => crate::tax::api::READ_ONLY,
        "tax_burden" => crate::tax_burden::api::READ_ONLY,
        "research" => crate::research::api::READ_ONLY,
        "ai" => crate::ai::api::READ_ONLY,
        "debt" => crate::debt::api::READ_ONLY,
        "physical" => crate::physical::api::READ_ONLY,
        "pension" => crate::pension::api::READ_ONLY,
        "fgts" => crate::fgts::api::READ_ONLY,
        "retirement" => crate::retirement::api::READ_ONLY,
        "food" => crate::food::api::READ_ONLY,
        "projects" => crate::projects::api::READ_ONLY,
        "sources" => crate::sources::READ_ONLY,
        "country" => crate::country::READ_ONLY,
        "database" => crate::maintenance::READ_ONLY,
        "health" => crate::health::api::READ_ONLY,
        "devfeedback" => crate::devfeedback::api::READ_ONLY,
        "score" => crate::score::api::READ_ONLY,
        "profile" => crate::profile::api::READ_ONLY,
        "mail" => crate::mail::READ_ONLY,
        "support" => crate::support::READ_ONLY,
        "terms" => &["status"],
        "system" => &["locale"],
        _ => &[],
    };
    list.contains(&action)
}

fn dispatch(ctx: &mut Ctx<'_>, module: &str, action: &str, args: Value) -> Result<Value> {
    match module {
        "crud" => crud::handle(ctx, action, args),
        "changes" => changes_api::handle(ctx, action, args),
        "backup" => system::handle_backup(ctx, action, args),
        "settings" => system::handle_settings(ctx, action, args),
        "profile" => crate::profile::api::handle(ctx, action, args),
        "household" => crate::household::api::handle(ctx, action, args),
        "sources" => crate::sources::handle(ctx, action, args),
        "country" => crate::country::handle(ctx, action, args),
        "database" => crate::maintenance::handle(ctx, action, args),
        "accounts" => crate::accounts::api::handle(ctx, action, args),
        "attention" => crate::attention::api::handle(ctx, action, args),
        "categories" => crate::categories::api::handle(ctx, action, args),
        "transactions" => crate::transactions::api::handle(ctx, action, args),
        "recurrences" => crate::recurrence::api::handle(ctx, action, args),
        "cards" => crate::cards::api::handle(ctx, action, args),
        "rules" => crate::rules::api::handle(ctx, action, args),
        "import" => crate::import::api::handle(ctx, action, args),
        "b3" => crate::import::b3::api::handle(ctx, action, args),
        "notes" => crate::import::notes::api::handle(ctx, action, args),
        "budget" => crate::budget::api::handle(ctx, action, args),
        "goals" => crate::goals::api::handle(ctx, action, args),
        "longevity" => crate::longevity::api::handle(ctx, action, args),
        "reports" => crate::reports::api::handle(ctx, action, args),
        "fx" => crate::fx::api::handle(ctx, action, args),
        "export" => crate::export::api::handle(ctx, action, args),
        "assets" => crate::assets::api::handle(ctx, action, args),
        "portfolio" => crate::portfolio::api::handle(ctx, action, args),
        "indices" => crate::indices::api::handle(ctx, action, args),
        "market" => crate::market::api::handle(ctx, action, args),
        "tax" => crate::tax::api::handle(ctx, action, args),
        "tax_burden" => crate::tax_burden::api::handle(ctx, action, args),
        "research" => crate::research::api::handle(ctx, action, args),
        "ai" => crate::ai::api::handle(ctx, action, args),
        "debt" => crate::debt::api::handle(ctx, action, args),
        "demo" => crate::demo::api::handle(ctx, action, args),
        "forecast" => crate::forecast::api::handle(ctx, action, args),
        "notify" => crate::notify::api::handle(ctx, action, args),
        "income" => crate::income::api::handle(ctx, action, args),
        "receipts" => crate::receipts::handle(ctx, action, args),
        "payments" => crate::payments::handle(ctx, action, args),
        "mail" => crate::mail::handle(ctx, action, args),
        "support" => crate::support::handle(ctx, action, args),
        "scenarios" => crate::scenarios::api::handle(ctx, action, args),
        "subscriptions" => crate::subscriptions::api::handle(ctx, action, args),
        "installments" => crate::installments::api::handle(ctx, action, args),
        "physical" => crate::physical::api::handle(ctx, action, args),
        "pension" => crate::pension::api::handle(ctx, action, args),
        "fgts" => crate::fgts::api::handle(ctx, action, args),
        "retirement" => crate::retirement::api::handle(ctx, action, args),
        "food" => crate::food::api::handle(ctx, action, args),
        "projects" => crate::projects::api::handle(ctx, action, args),
        "health" => crate::health::api::handle(ctx, action, args),
        "mcp" => crate::mcp::api::handle(ctx, action, args),
        "devfeedback" => crate::devfeedback::api::handle(ctx, action, args),
        "score" => crate::score::api::handle(ctx, action, args),
        _ => Err(Error::UnknownCommand(format!("{module}.{action}"))),
    }
}

mod changes_api {
    use super::*;
    use crate::changes::ChangeListArgs;

    pub const READ_ONLY: &[&str] = &["list", "get"];

    #[derive(serde::Deserialize)]
    struct IdArg {
        id: String,
    }

    pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
        match action {
            "list" => to_value(changes::list(
                ctx.conn(),
                &parse_args::<ChangeListArgs>(args)?,
            )?),
            "get" => to_value(changes::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
            "undo" => to_value(changes::undo(
                ctx.db.conn_mut(),
                &parse_args::<IdArg>(args)?.id,
            )?),
            "redo" => to_value(changes::redo(
                ctx.db.conn_mut(),
                &parse_args::<IdArg>(args)?.id,
            )?),
            _ => Err(Error::UnknownCommand(format!("changes.{action}"))),
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_agent;
#[cfg(test)]
mod tests_agent_attachments;
#[cfg(test)]
mod tests_agent_memory;
#[cfg(test)]
mod tests_ai;
#[cfg(test)]
mod tests_groq;
#[cfg(test)]
mod tests_import;
#[cfg(test)]
mod tests_leaks;
#[cfg(test)]
mod tests_ledger;
#[cfg(test)]
mod tests_markets;
#[cfg(test)]
mod tests_recovery;
#[cfg(test)]
mod tests_reports;
#[cfg(test)]
mod tests_study;
#[cfg(test)]
mod tests_upgrade;
