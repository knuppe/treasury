//! Comandos `ai.*`. Os que fazem rede ou disparam jobs (`chat_*`, `test_engine`,
//! `detect_clis`, `routines.run`, `jobs.list`) são atendidos no nível da `Api`
//! ([`handle_api_level`]), sem segurar a trava do banco.

use rusqlite::params;
use serde::Deserialize;
use serde_json::Value;

use super::*;
use crate::api::{Actor, Api, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::ids::new_id;
use crate::time::now_utc;

pub const READ_ONLY: &[&str] = &[
    "list_proposals",
    "get_proposal",
    "pending_count",
    "memory",
    "procedures.list",
    "procedures.get",
    "alerts.list",
    "alerts.count",
    "suggestions",
    "playbook",
    "find_tools",
    "chat_history",
    "attachment_read",
    "memory_recall",
];

/// Ações atendidas fora da sessão (threads e rede).
pub const API_LEVEL: &[&str] = &[
    "chat_start",
    "chat_poll",
    "chat_cancel",
    "jobs.list",
    "test_engine",
    "detect_clis",
    "routines.run",
    "routines.status",
    "attachments.prepare",
    "attachments.discard",
    "mcp_extra.test",
    "mcp_extra.decide",
    "question.answer",
    "cli_mcps.list",
];

#[derive(Deserialize)]
struct EngineIdArg {
    engine_id: String,
}

#[derive(Deserialize)]
struct SecretArgs {
    engine_id: String,
    value: String,
}

#[derive(Deserialize, Default)]
struct LimitArg {
    #[serde(default)]
    limit: Option<i64>,
    /// Só as conversas da página deste ativo.
    #[serde(default)]
    asset_id: Option<String>,
}

#[derive(Deserialize)]
struct IdsArg {
    ids: Vec<String>,
}

fn user_only_actor(actor: &Actor, what: &str) -> Result<()> {
    if *actor != Actor::User {
        return Err(Error::Forbidden(format!("{what} só pela interface do app")));
    }
    Ok(())
}

/// Comandos de rede e jobs (chamados por `api::system::handle_unlocked`).
pub fn handle_api_level(api: &Api, actor: &Actor, action: &str, args: &Value) -> Result<Value> {
    let args = args.clone();
    match action {
        "chat_start" => {
            user_only_actor(actor, "conversar com o agente")?;
            to_value(agent::start(api, parse_args(args)?)?)
        }
        "chat_poll" => to_value(api.ai_runtime().poll(&parse_args(args)?)?),
        "chat_cancel" => {
            #[derive(Deserialize)]
            struct A {
                job_id: String,
            }
            api.ai_runtime().cancel(&parse_args::<A>(args)?.job_id)?;
            Ok(Value::Null)
        }
        "jobs.list" => to_value(api.ai_runtime().list()),
        "test_engine" => {
            user_only_actor(actor, "testar motores")?;
            let a: EngineIdArg = parse_args(args)?;
            to_value(agent::test_engine(api, &a.engine_id)?)
        }
        "detect_clis" => {
            user_only_actor(actor, "detectar CLIs")?;
            to_value(cli::detect())
        }
        "routines.run" => {
            user_only_actor(actor, "rodar rotinas")?;
            let a: routines::RunRoutineArgs = parse_args(args)?;
            let shared = api.ai_runtime().shared()?;
            to_value(serde_json::json!({ "job_id": routines::start(&shared, a.routine)? }))
        }
        "routines.status" => to_value(routines::status(api)?),
        // Anexos: preparados fora da trava do banco (ler PDF ou planilha leva tempo) e
        // guardados só na memória até o envio.
        "attachments.prepare" => {
            user_only_actor(actor, "anexar arquivos")?;
            let a: attachments::PrepareArgs = parse_args(args)?;
            let prepared = attachments::prepare_args(&a)?;
            let staged = api.ai_runtime().attachments.put(prepared);
            to_value(staged.info(false))
        }
        "attachments.discard" => {
            user_only_actor(actor, "remover anexos")?;
            let a: IdsArg = parse_args(args)?;
            api.ai_runtime().attachments.remove(&a.ids);
            Ok(Value::Null)
        }
        // MCPs extras: o teste conecta no servidor (processo ou rede) fora da trava do banco.
        "mcp_extra.test" => {
            user_only_actor(actor, "testar MCPs extras")?;
            let a: IdArg = parse_args(args)?;
            let r = api.with_session(|s, _| {
                let server = mcp_extra::get(s.db.conn(), s.db.key(), &a.id)?;
                mcp_extra::resolve(s.db.conn(), s.db.key(), &server)
            })?;
            to_value(mcp_extra::test(&r)?)
        }
        // MCPs que as CLIs instaladas carregam: lê as configurações e roda `grok inspect`
        // (até 5 s) fora da trava do banco.
        "cli_mcps.list" => {
            user_only_actor(actor, "ver os MCPs das CLIs")?;
            let allowed = api.with_session(|s, _| mcp_extra::allowed_cli_mcps(s.db.conn()))?;
            let found = cli::user_mcps_by_cli(
                dirs::home_dir().as_deref(),
                &api.paths().run_dir.join("grok-workspace"),
            );
            to_value(mcp_extra::cli_user_mcps(&found, &allowed))
        }
        "question.answer" => {
            user_only_actor(actor, "responder ao agente")?;
            #[derive(Deserialize)]
            struct A {
                request_id: String,
                #[serde(flatten)]
                answer: ask::Answer,
            }
            let a: A = parse_args(args)?;
            api.ai_runtime().answer_question(&a.request_id, &a.answer)?;
            Ok(Value::Null)
        }
        "mcp_extra.decide" => {
            user_only_actor(actor, "confirmar ferramentas externas")?;
            #[derive(Deserialize)]
            struct A {
                request_id: String,
                allow: bool,
                /// `conversation`: permitir o servidor no resto da conversa.
                #[serde(default)]
                scope: Option<String>,
            }
            let a: A = parse_args(args)?;
            let remember = a.scope.as_deref() == Some("conversation");
            api.ai_runtime()
                .decide_external(&a.request_id, a.allow, remember)?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("ai.{action}"))),
    }
}

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

fn kind_str(k: MemoryKind) -> &'static str {
    match k {
        MemoryKind::Profile => "profile",
        MemoryKind::FamilyRule => "family_rule",
        MemoryKind::Note => "note",
    }
}

fn user_only(ctx: &Ctx<'_>, what: &str) -> Result<()> {
    if *ctx.actor != Actor::User {
        return Err(Error::Forbidden(format!("{what} só pela interface do app")));
    }
    Ok(())
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        // --- Motores e segredos ---------------------------------------------------------
        "engines.providers" => to_value(engine::providers()),
        // --- MCPs extras (só pela interface; os segredos nunca voltam) ------------------
        "mcp_extra.list" => {
            user_only(ctx, "ver MCPs extras")?;
            to_value(mcp_extra::list(ctx.conn(), ctx.db.key())?)
        }
        "mcp_extra.save" => {
            user_only(ctx, "configurar MCPs extras")?;
            let i: mcp_extra::ServerInput = parse_args(args)?;
            to_value(mcp_extra::save(ctx.conn(), ctx.db.key(), &i)?)
        }
        "cli_mcps.set" => {
            user_only(ctx, "liberar MCPs das CLIs")?;
            #[derive(Deserialize)]
            struct A {
                allowed: Vec<String>,
            }
            let a: A = parse_args(args)?;
            to_value(mcp_extra::set_allowed_cli_mcps(ctx.conn(), &a.allowed)?)
        }
        "mcp_extra.delete" => {
            user_only(ctx, "configurar MCPs extras")?;
            let a: IdArg = parse_args(args)?;
            mcp_extra::delete(ctx.conn(), ctx.db.key(), &a.id)?;
            Ok(Value::Null)
        }
        "engines.list" => to_value(engine::list(ctx.conn(), ctx.db.key())?),
        "engines.get" => {
            let a: IdArg = parse_args(args)?;
            to_value(engine::get(ctx.conn(), ctx.db.key(), &a.id)?)
        }
        "engines.create" => {
            user_only(ctx, "configurar motores")?;
            let a: engine::EngineInput = parse_args(args)?;
            let created = engine::create(ctx.conn(), ctx.db.key(), &a)?;
            super::settings::routines_off_after_first_engine(ctx.conn())?;
            to_value(created)
        }
        "engines.update" => {
            user_only(ctx, "configurar motores")?;
            let a: engine::EnginePatch = parse_args(args)?;
            to_value(engine::update(ctx.conn(), ctx.db.key(), &a)?)
        }
        "engines.delete" => {
            user_only(ctx, "configurar motores")?;
            let a: IdArg = parse_args(args)?;
            engine::delete(ctx.conn(), &a.id)?;
            Ok(Value::Null)
        }
        "set_secret" => {
            user_only(ctx, "cadastrar chaves")?;
            let a: SecretArgs = parse_args(args)?;
            engine::get(ctx.conn(), ctx.db.key(), &a.engine_id)?;
            to_value(secret::set(
                ctx.conn(),
                ctx.db.key(),
                &engine::secret_id(&a.engine_id),
                &a.value,
            )?)
        }
        "delete_secret" => {
            user_only(ctx, "apagar chaves")?;
            let a: EngineIdArg = parse_args(args)?;
            secret::delete(ctx.conn(), &engine::secret_id(&a.engine_id))?;
            to_value(secret::SecretStatus::default())
        }
        // --- Roteamento, limites, preços e uso -------------------------------------------
        "routes.list" => to_value(routing::list(ctx.conn())?),
        "routes.set" => {
            user_only(ctx, "configurar o roteamento")?;
            to_value(routing::set(ctx.conn(), &parse_args(args)?)?)
        }
        "limits.list" => to_value(usage::limits(ctx.conn())?),
        "limits.set" => {
            user_only(ctx, "configurar limites de gasto")?;
            to_value(usage::set_limit(ctx.conn(), &parse_args(args)?)?)
        }
        "prices.list" => to_value(usage::prices(ctx.conn())?),
        "prices.set" => {
            user_only(ctx, "editar preços")?;
            to_value(usage::set_price(ctx.conn(), &parse_args(args)?)?)
        }
        "prices.delete" => {
            user_only(ctx, "editar preços")?;
            usage::delete_price(ctx.conn(), &parse_args(args)?)?;
            Ok(Value::Null)
        }
        "usage_summary" => to_value(usage::summary(ctx.conn(), &parse_args(args)?)?),
        // --- Ajustes ---------------------------------------------------------------------
        "settings.get" => to_value(settings::load(ctx.conn())?),
        "settings.set" => {
            user_only(ctx, "ajustes de IA")?;
            to_value(settings::apply(ctx.conn(), &parse_args(args)?)?)
        }
        // --- Conversas -------------------------------------------------------------------
        "conversations.list" => {
            let a: LimitArg = parse_args(args)?;
            let limit = a.limit.unwrap_or(50);
            match a.asset_id.as_deref().filter(|s| !s.trim().is_empty()) {
                Some(asset) => to_value(conversation::list_for_asset(ctx.conn(), asset, limit)?),
                None => to_value(conversation::list(ctx.conn(), limit)?),
            }
        }
        "conversations.get" => {
            let a: IdArg = parse_args(args)?;
            to_value(conversation::get(ctx.conn(), &a.id)?)
        }
        "conversations.export" => {
            user_only(ctx, "exportar conversas")?;
            let a: super::export::ConversationExportArgs = parse_args(args)?;
            to_value(super::export::export(
                ctx.conn(),
                ctx.db.key(),
                ctx.paths,
                &a,
            )?)
        }
        "conversations.delete" => {
            user_only(ctx, "apagar conversas")?;
            let a: IdArg = parse_args(args)?;
            conversation::delete(ctx.conn(), &a.id)?;
            Ok(Value::Null)
        }
        "conversations.pin" => {
            user_only(ctx, "fixar conversas")?;
            #[derive(Deserialize)]
            struct A {
                id: String,
                pinned: bool,
            }
            let a: A = parse_args(args)?;
            conversation::set_pinned(ctx.conn(), &a.id, a.pinned)?;
            Ok(Value::Null)
        }
        "attachments.get" => {
            user_only(ctx, "abrir anexos")?;
            let a: IdArg = parse_args(args)?;
            to_value(attachments::get_data(ctx.conn(), &a.id)?)
        }
        "history.stats" => to_value(history::stats(ctx.conn())?),
        "history.cleanup" => {
            user_only(ctx, "limpar o histórico")?;
            to_value(history::cleanup(ctx.conn(), chrono::Utc::now())?)
        }
        "conversations.clear" => {
            user_only(ctx, "apagar conversas")?;
            to_value(serde_json::json!({ "deleted": conversation::clear(ctx.conn())? }))
        }
        // --- Sugestões do companheiro (determinísticas, sem motor) ---------------------
        "suggestions" => to_value(suggestions::compute(
            ctx.conn(),
            &parse_args(args)?,
            crate::time::today_local(),
        )?),
        // --- Alertas e rotinas -----------------------------------------------------------
        "alerts.list" => to_value(alerts::list(ctx.conn(), &parse_args(args)?)?),
        "alerts.count" => to_value(alerts::counts(ctx.conn())?),
        "alerts.set_status" => {
            user_only(ctx, "atualizar alertas")?;
            to_value(alerts::set_status(ctx.conn(), &parse_args(args)?)?)
        }
        "alerts.delete" => {
            user_only(ctx, "apagar alertas")?;
            let a: IdsArg = parse_args(args)?;
            to_value(serde_json::json!({ "deleted": alerts::delete(ctx.conn(), &a.ids)? }))
        }
        // --- Fila de aprovação e memória -------------------------------------------------
        "list_proposals" => to_value(proposals::list(
            ctx.conn(),
            &parse_args::<ProposalListArgs>(args)?,
        )?),
        "get_proposal" => to_value(proposals::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "pending_count" => to_value(proposals::pending_count(ctx.conn())?),
        "approve" => {
            user_only(ctx, "aprovar propostas")?;
            let a: ReviewArgs = parse_args(args)?;
            if a.ids.is_empty() {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-ia-proposta-selecione",
                )));
            }
            // Backup antes de aplicar um lote de escritas da IA.
            crate::backup::create(
                ctx.db,
                &ctx.paths.backups_dir,
                crate::backup::BackupKind::PreAi,
            )?;
            to_value(proposals::approve(
                ctx.db.conn_mut(),
                &a.ids,
                a.note.as_deref(),
            )?)
        }
        "reject" => {
            user_only(ctx, "rejeitar propostas")?;
            let a: ReviewArgs = parse_args(args)?;
            to_value(proposals::reject(
                ctx.db.conn_mut(),
                &a.ids,
                a.note.as_deref(),
            )?)
        }
        // O agente retira a própria proposta pendente (docs/ai-agent.md › Permissões e controle).
        "withdraw_proposal" => {
            #[derive(Deserialize)]
            struct A {
                id: String,
                reason: Option<String>,
            }
            let a: A = parse_args(args)?;
            let Actor::Mcp { client, .. } = ctx.actor else {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-agente-retirar-so-agente",
                )));
            };
            to_value(proposals::withdraw_own(
                ctx.conn(),
                &a.id,
                client,
                a.reason.as_deref(),
            )?)
        }
        "find_tools" => to_value(crate::mcp::find::run(parse_args(args)?)),
        "playbook" => {
            #[derive(Deserialize, Default)]
            struct A {
                topic: Option<String>,
            }
            let a: A = parse_args(args).unwrap_or_default();
            match a.topic.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
                None => to_value(
                    playbooks::PLAYBOOKS
                        .iter()
                        .map(|p| serde_json::json!({"topic": p.topic, "title": p.title, "summary": p.summary}))
                        .collect::<Vec<_>>(),
                ),
                Some(t) => {
                    let p = playbooks::find(t).ok_or_else(|| {
                        Error::invalid(format!(
                            "guia {t} não existe; assuntos: {}",
                            playbooks::PLAYBOOKS
                                .iter()
                                .map(|p| format!("`{}` ({})", p.topic, p.title))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    })?;
                    to_value(p)
                }
            }
        }
        "chat_history" => to_value(conversation::history(ctx.conn(), &parse_args(args)?)?),
        "attachment_read" => to_value(attachments::read_text(ctx.conn(), &parse_args(args)?)?),
        "memory_recall" => to_value(memory_graph::recall(ctx.conn(), &parse_args(args)?)?),
        "memory" => {
            let mut stmt = ctx.conn().prepare("SELECT id, kind, content, created_by, created_at, updated_at FROM ai_memory ORDER BY kind, created_at")?;
            let rows = stmt
                .query_map([], |r| {
                    let kind: String = r.get(1)?;
                    Ok(MemoryItem {
                        id: r.get(0)?,
                        kind: match kind.as_str() {
                            "profile" => MemoryKind::Profile,
                            "family_rule" => MemoryKind::FamilyRule,
                            _ => MemoryKind::Note,
                        },
                        content: r.get(2)?,
                        created_by: r.get(3)?,
                        created_at: r.get(4)?,
                        updated_at: r.get(5)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            to_value(rows)
        }
        "remember" => {
            let a: RememberArgs = parse_args(args)?;
            let content = a.content.trim().to_string();
            if content.is_empty() || content.chars().count() > 2000 {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-ia-memoria-nota-tamanho",
                )));
            }
            let by = if *ctx.actor == Actor::User {
                "user"
            } else {
                "ai"
            };
            let id = new_id();
            let now = now_utc();
            ctx.write(&crate::i18n::t("nucleo-ia-memoria-gestor"), |tx| {
                tx.execute(
                    "INSERT INTO ai_memory (id, kind, content, created_by, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                    params![id, kind_str(a.kind), content, by, now],
                )?;
                Ok(())
            })?;
            to_value(serde_json::json!({ "id": id }))
        }
        // --- Memória de procedimentos do agente ------------------------------------------
        "procedures.list" => to_value(procedures::list(ctx.conn(), &parse_args(args)?)?),
        "procedures.get" => to_value(procedures::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "procedures.save" => {
            let a: procedures::ProcedureSave = parse_args(args)?;
            let prepared = procedures::prepare(ctx.conn(), &a)?;
            let (by, client) = match ctx.actor {
                Actor::Mcp { client, .. } => ("ai", Some(client.clone())),
                _ => ("user", None),
            };
            let reason = crate::i18n::t(if prepared.created {
                "nucleo-ia-memoria-nota-nova"
            } else {
                "nucleo-ia-memoria-nota-atualizada"
            });
            ctx.write(&reason, |tx| {
                procedures::write(tx, &prepared, by, client.as_deref())
            })?;
            to_value(procedures::ProcedureSaved {
                id: prepared.id.clone(),
                created: prepared.created,
                redactions: prepared.redactions,
                message: if prepared.created {
                    "Gravado na memória de procedimentos (visível em Agente › Memória).".into()
                } else {
                    "Nota da memória de procedimentos atualizada.".into()
                },
            })
        }
        "procedures.outcome" => {
            to_value(procedures::record_outcome(ctx.conn(), &parse_args(args)?)?)
        }
        "procedures.delete" => {
            user_only(ctx, "apagar a memória de procedimentos")?;
            let a: IdArg = parse_args(args)?;
            ctx.write(&crate::i18n::t("nucleo-ia-memoria-nota-apagada"), |tx| {
                procedures::delete(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "forget" => {
            user_only(ctx, "apagar memória")?;
            let a: IdArg = parse_args(args)?;
            ctx.write(&crate::i18n::t("nucleo-ia-memoria-gestor-removida"), |tx| {
                if tx.execute("DELETE FROM ai_memory WHERE id = ?1", [&a.id])? == 0 {
                    return Err(Error::not_found(crate::i18n::t("nucleo-ia-memoria-nota")));
                }
                Ok(())
            })?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("ai.{action}"))),
    }
}
