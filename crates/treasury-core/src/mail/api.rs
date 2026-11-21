//! Comandos `mail.*` (docs/email-ingest.md). Configurar contas, regras, senha e retenção é só
//! pela interface do app (nunca pela IA nem por cliente MCP); a fila e a auditoria podem ser
//! lidas. A senha de app nunca sai: só o "cadastrada" e os 4 últimos caracteres.

use serde::Deserialize;
use serde_json::{Value, json};
use ts_rs::TS;

use super::config::{self, AccountKind, MailAccountInput, MailRuleInput};
use super::filter::{self, Envelope};
use super::ingest::{self, By};
use super::store::{self, MailAuditArgs, MailListArgs, MailStatus};
use super::{ai, imap, local, providers, retention, sync};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::changes::{self, Author};
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::Id;

pub const READ_ONLY: &[&str] = &[
    "accounts.list",
    "accounts.get",
    "rules.list",
    "rules.test",
    "list",
    "get",
    "audit",
    "would_delete",
    "providers",
];

/// Ações atendidas fora da trava do cofre (rede e threads).
pub const API_LEVEL: &[&str] = &[
    "discover",
    "test_connection",
    "sync_now",
    "ask_ai",
    "apply_part",
];

#[derive(Deserialize)]
struct IdArg {
    id: Id,
}

#[derive(Deserialize)]
struct PasswordArgs {
    id: Id,
    password: crate::vault::Secret,
}

#[derive(Deserialize, Default)]
struct AccountArg {
    #[serde(default)]
    account_id: Option<Id>,
}

#[derive(Deserialize)]
struct MarkArgs {
    id: Id,
    status: MailStatus,
    #[serde(default)]
    result_ref: Option<String>,
}

/// "Testar" uma regra: contra um exemplo e contra as mensagens recentes da conta. Não importa
/// nem apaga nada.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct MailRuleTestArgs {
    pub account_id: Id,
    pub rule: MailRuleInput,
    #[serde(default)]
    #[ts(optional)]
    pub sample: Option<Envelope>,
}

/// `what`: chave da frase inteira ("… só pela interface do app") no catálogo.
fn user_only(ctx: &Ctx<'_>, what: &str) -> Result<()> {
    user_only_actor(ctx.actor, what)
}

fn test_rule(conn: &rusqlite::Connection, a: &MailRuleTestArgs) -> Result<Value> {
    filter::validate(&a.rule.conditions)?;
    let acc = config::get_account(conn, None, &a.account_id)?;
    let mut rules = config::rules_for(conn, &a.account_id)?;
    let tested_id = a.rule.id.clone().unwrap_or_else(|| "teste".into());
    rules.retain(|r| r.id != tested_id);
    let priority = a.rule.priority.unwrap_or(100);
    let tested = config::MailRule {
        id: tested_id.clone(),
        account_id: a.rule.account_id.clone(),
        name: a.rule.name.clone(),
        priority,
        enabled: true,
        conditions: a.rule.conditions.clone(),
        action: a.rule.action.unwrap_or(filter::RuleAction::Review),
        ai: a.rule.ai.unwrap_or_default(),
        retention: a.rule.retention.clone(),
        template: a.rule.template.clone(),
        created_at: String::new(),
        updated_at: String::new(),
    };
    let pos = rules
        .iter()
        .position(|r| r.priority > priority)
        .unwrap_or(rules.len());
    rules.insert(pos, tested);
    let f = config::account_filter(&acc);
    let specs = config::specs(&rules);
    let sample = a.sample.as_ref().map(|e| {
        let mut e = e.clone();
        e.subject = filter::clean_subject(&e.subject);
        if e.folder.is_empty() {
            e.folder = "INBOX".into();
        }
        filter::decide(&f, &specs, &e)
    });
    // As mensagens recentes da fila, com o que a regra decidiria agora.
    let recent: Vec<Value> = store::list(
        conn,
        &MailListArgs {
            account_id: Some(a.account_id.clone()),
            status: None,
            limit: Some(50),
        },
    )?
    .into_iter()
    .map(|m| {
        let env = Envelope {
            folder: m.folder.clone(),
            from: m.sender.clone().unwrap_or_default(),
            original_from: m.original_sender.clone(),
            subject: m.subject.clone().unwrap_or_default(),
            date: m.date.clone(),
            size: m.size as u64,
            attachments: m.parts.iter().map(|p| p.filename.clone()).collect(),
            ..Default::default()
        };
        let d = filter::decide(&f, &specs, &env);
        json!({
            "message_id": m.id,
            "subject": m.subject,
            "sender": m.original_sender.or(m.sender),
            "matches": d.rule_id.as_deref() == Some(tested_id.as_str()),
            "decision": d,
        })
    })
    .collect();
    Ok(json!({ "sample": sample, "recent": recent }))
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "providers" => to_value(providers::CATALOG),
        "accounts.list" => to_value(config::list_accounts(ctx.conn(), Some(ctx.db.key()))?),
        "accounts.get" => to_value(config::get_account(
            ctx.conn(),
            Some(ctx.db.key()),
            &parse_args::<IdArg>(args)?.id,
        )?),
        "accounts.save" => {
            user_only(ctx, "nucleo-email-so-app-configurar")?;
            let mut a: MailAccountInput = parse_args(args)?;
            if let Some(c) = a.pinned_cert.as_deref().filter(|c| !c.trim().is_empty()) {
                a.pinned_cert = Some(imap::fingerprint(c)?);
            }
            let root = ctx.paths.root.clone();
            let id = ctx
                .write(&t("nucleo-email-historico-conta-salva"), |tx| {
                    config::save_account(tx, &a, &root)
                })?
                .value;
            to_value(config::get_account(ctx.conn(), Some(ctx.db.key()), &id)?)
        }
        "accounts.delete" => {
            user_only(ctx, "nucleo-email-so-app-apagar-contas")?;
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-email-historico-conta-apagada"), |tx| {
                config::delete_account(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "accounts.set_password" => {
            user_only(ctx, "nucleo-email-so-app-senha")?;
            let a: PasswordArgs = parse_args(args)?;
            to_value(config::set_password(
                ctx.conn(),
                ctx.db.key(),
                &a.id,
                &a.password,
            )?)
        }
        "accounts.confirm_retention" => {
            user_only(ctx, "nucleo-email-so-app-confirmar-exclusao")?;
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-email-historico-exclusao-confirmada"), |tx| {
                config::confirm_retention(tx, &a.id)
            })?;
            to_value(config::get_account(ctx.conn(), Some(ctx.db.key()), &a.id)?)
        }
        "rules.list" => to_value(config::list_rules(
            ctx.conn(),
            parse_args::<AccountArg>(args)?.account_id.as_deref(),
        )?),
        "rules.save" => {
            user_only(ctx, "nucleo-email-so-app-regras")?;
            let a: MailRuleInput = parse_args(args)?;
            let id = ctx
                .write(&t("nucleo-email-historico-regra-salva"), |tx| {
                    config::save_rule(tx, &a)
                })?
                .value;
            to_value(config::get_rule(ctx.conn(), &id)?)
        }
        "rules.delete" => {
            user_only(ctx, "nucleo-email-so-app-apagar-regras")?;
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-email-historico-regra-apagada"), |tx| {
                config::delete_rule(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "rules.test" => test_rule(ctx.conn(), &parse_args(args)?),
        "list" => to_value(store::list(ctx.conn(), &parse_args::<MailListArgs>(args)?)?),
        "get" => to_value(store::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "audit" => to_value(store::audit_list(
            ctx.conn(),
            &parse_args::<MailAuditArgs>(args)?,
        )?),
        "would_delete" => {
            let a: AccountArg = parse_args(args)?;
            let id = a
                .account_id
                .ok_or_else(|| Error::invalid(t("nucleo-email-erro-informe-conta")))?;
            to_value(retention::would_delete(ctx.conn(), &id)?)
        }
        "part_content" => {
            user_only(ctx, "nucleo-email-so-app-anexos")?;
            use base64::Engine as _;
            let a: IdArg = parse_args(args)?;
            let c = store::part_content(ctx.conn(), &a.id)?;
            Ok(json!({
                "filename": c.filename,
                "mime": c.mime,
                "kind": c.kind,
                "content_base64": base64::engine::general_purpose::STANDARD.encode(&c.bytes),
            }))
        }
        "mark_part" => {
            user_only(ctx, "nucleo-email-so-app-revisar")?;
            let a: MarkArgs = parse_args(args)?;
            if !matches!(
                a.status,
                MailStatus::Imported | MailStatus::Ignored | MailStatus::Duplicate
            ) {
                return Err(Error::invalid(t("nucleo-email-erro-marcar")));
            }
            let conn = ctx.conn();
            store::set_part(conn, &a.id, a.status, a.result_ref.as_deref(), None)?;
            let msg = store::message_of_part(conn, &a.id)?;
            let account: Option<String> = conn.query_row(
                "SELECT account_id FROM mail_message WHERE id = ?1",
                [&msg],
                |r| r.get(0),
            )?;
            store::audit(
                conn,
                account.as_deref(),
                "reviewed",
                Some(&msg),
                &crate::i18n::t_args(
                    "nucleo-email-auditoria-peca-marcada",
                    &crate::i18n::args(&[("status", a.status.as_str().into())]),
                ),
            )?;
            to_value(store::get(conn, &msg)?)
        }
        "ignore" => {
            user_only(ctx, "nucleo-email-so-app-revisar")?;
            let a: IdArg = parse_args(args)?;
            let conn = ctx.conn();
            store::ignore_message(conn, &a.id)?;
            let m = store::get(conn, &a.id)?;
            store::audit(
                conn,
                Some(&m.account_id),
                "reviewed",
                Some(&a.id),
                &t("nucleo-email-auditoria-ignorada"),
            )?;
            to_value(m)
        }
        _ => Err(Error::UnknownCommand(format!("mail.{action}"))),
    }
}

/// `what`: chave da frase inteira ("… só pela interface do app") no catálogo.
fn user_only_actor(actor: &Actor, what: &str) -> Result<()> {
    if *actor != Actor::User {
        return Err(Error::Forbidden(t(what)));
    }
    Ok(())
}

#[derive(Deserialize)]
struct DiscoverArgs {
    address: String,
}

#[derive(Deserialize)]
struct TestArgs {
    id: Id,
    /// Senha digitada agora (não é guardada); sem ela, a cadastrada.
    #[serde(default)]
    password: Option<crate::vault::Secret>,
}

#[derive(Deserialize)]
struct PartArgs {
    part_id: Id,
    #[serde(default)]
    account_id: Option<Id>,
}

/// Comandos de rede e threads (chamados por `api::system::handle_unlocked`).
pub fn handle_api_level(
    api: &crate::Api,
    actor: &Actor,
    action: &str,
    args: &Value,
) -> Result<Value> {
    user_only_actor(actor, "nucleo-email-so-app-ingestao")?;
    let args = args.clone();
    match action {
        "discover" => {
            let a: DiscoverArgs = parse_args(args)?;
            if let Some(d) = providers::from_catalog(&a.address)? {
                return to_value(d);
            }
            api.with_session(|s, _| {
                crate::sources::ensure_enabled(
                    s.db.conn(),
                    crate::sources::SourceId::MailAutoconfig,
                )
            })?;
            to_value(providers::discover(&a.address)?)
        }
        "test_connection" => {
            let a: TestArgs = parse_args(args)?;
            let acc = api.with_session(|s, _| config::get_account(s.db.conn(), None, &a.id))?;
            let out = match acc.kind {
                AccountKind::Folder => {
                    let root =
                        std::path::PathBuf::from(acc.folder_path.clone().unwrap_or_default());
                    let files = local::scan(&root, &acc.allowed_folders, &acc.blocked_folders)?;
                    json!({ "files": files.len(), "folder": acc.folder_path })
                }
                AccountKind::Imap => {
                    let stored = match a.password.as_deref().filter(|p| !p.trim().is_empty()) {
                        Some(_) => None,
                        None => api.with_session(|s, _| {
                            crate::ai::secret::get(
                                s.db.conn(),
                                s.db.key(),
                                &config::secret_id(&a.id),
                            )
                        })?,
                    };
                    let pw = a
                        .password
                        .as_deref()
                        .map(String::as_str)
                        .filter(|p| !p.trim().is_empty())
                        .map(str::trim)
                        .or(stored.as_ref().map(|p| p.as_str()))
                        .ok_or_else(|| Error::invalid(t("nucleo-email-erro-informe-senha")))?;
                    let t = imap::test(&acc, pw)?;
                    to_value(t)?
                }
            };
            // Deu certo: um backup restaurado volta a buscar.
            if acc.needs_reconfirm {
                api.with_session(|s, _| {
                    changes::record(
                        s.db.conn_mut(),
                        &Author::user(),
                        &t("nucleo-email-historico-conexao-confirmada"),
                        |tx| {
                            tx.execute(
                                "UPDATE mail_account SET needs_reconfirm = 0 WHERE id = ?1",
                                [&a.id],
                            )?;
                            Ok(())
                        },
                    )
                    .map(|_| ())
                })?;
            }
            Ok(out)
        }
        "sync_now" => {
            let a: IdArg = parse_args(args)?;
            let acc =
                api.with_session(|s, _| config::get_account(s.db.conn(), Some(s.db.key()), &a.id))?;
            if acc.needs_reconfirm {
                return Err(Error::conflict(t("nucleo-email-erro-reconfirmar")));
            }
            let shared = api.ai_runtime().shared()?;
            Ok(json!({ "started": sync::start(&shared, &a.id)? }))
        }
        "ask_ai" => {
            let a: PartArgs = parse_args(args)?;
            api.with_session(|s, _| ai::allowed(s.db.conn(), &a.part_id))?;
            let shared = api.ai_runtime().shared()?;
            Ok(json!({ "job_id": ai::start(&shared, &a.part_id)? }))
        }
        "apply_part" => {
            let a: PartArgs = parse_args(args)?;
            let res = ingest::apply_part(api, &a.part_id, By::User, a.account_id.as_deref());
            let message = match &res {
                Ok(ingest::Applied::Imported(_)) => t("nucleo-email-importada"),
                Ok(ingest::Applied::Duplicate) => t("nucleo-email-ja-estava"),
                Ok(ingest::Applied::Review(why)) => why.clone(),
                Err(e) => e.message(),
            };
            let status = api.with_session(|s, _| {
                ingest::record_applied(
                    s.db.conn(),
                    &a.part_id,
                    &res,
                    &t("nucleo-email-auditoria-importada-revisao"),
                )
            })?;
            res?;
            crate::events::emit_system("mail.apply");
            Ok(json!({ "status": status, "message": message }))
        }
        _ => Err(Error::UnknownCommand(format!("mail.{action}"))),
    }
}
