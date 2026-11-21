use serde::Deserialize;
use serde_json::Value;

use super::{
    DevFeedbackBump, DevFeedbackInput, DevFeedbackLinkArgs, DevFeedbackListArgs, DevFeedbackPatch,
    DevFeedbackPriority, DevFeedbackResolve, DevFeedbackStatusArgs, DevFeedbackUnlink,
};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::t;

pub const READ_ONLY: &[&str] = &["list", "get", "markdown"];

/// Pontuação mínima da busca de ferramentas para sugerir uma que já existe (nome e título
/// batendo com mais de uma palavra do título do feedback).
const SUGGEST_MIN_SCORE: u32 = 7;

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

/// Exportação em Markdown: a interface monta o texto e o núcleo grava no caminho que a pessoa
/// escolheu no diálogo de salvar (o WebKit do app não baixa links de Blob).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExportArgs {
    path: String,
    text: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let client = match ctx.actor {
        Actor::User => "user".to_string(),
        Actor::Mcp { client, .. } => client.clone(),
        Actor::System | Actor::Mail { .. } => "sistema".to_string(),
    };
    let agent = matches!(ctx.actor, Actor::Mcp { .. });
    // Agentes só com o modo desenvolvedor ligado; gerem o quadro inteiro, menos gravar o
    // Markdown em arquivo (só a interface). Apagar e descartar viram proposta (política).
    if agent && !super::developer_mode(ctx.conn())? {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-geral-erro-modo-desenvolvedor",
        )));
    }
    if action == "export" && *ctx.actor != Actor::User {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-geral-erro-feedback-so-interface",
        )));
    }
    // Toda escrita passa pelo histórico de alterações (V150): desfazer na tela ou no histórico.
    let reason = |key: &str| t(&format!("nucleo-geral-motivo-feedback-{key}"));
    match action {
        "list" => {
            let a = parse_args::<DevFeedbackListArgs>(args)?;
            let items = super::list(ctx.conn(), &a)?;
            match a.group_by {
                Some(super::DevFeedbackGroupBy::Status) => to_value(super::board(items)),
                None => to_value(items),
            }
        }
        "markdown" => {
            let a = parse_args::<super::DevFeedbackMarkdownArgs>(args)?;
            to_value(super::markdown(ctx.conn(), &a)?)
        }
        "get" => to_value(super::detail(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "add" => {
            let a = parse_args::<DevFeedbackInput>(args)?;
            let added = ctx
                .write(&reason("registrar"), |tx| {
                    super::add(tx, &a, &client, agent)
                })?
                .value;
            let mut v = to_value(&added)?;
            // Pedido de ferramenta ou melhoria: as que já existem e talvez resolvam (o agente
            // confere antes de esperar pelo desenvolvedor).
            if agent
                && !added.deduplicated
                && matches!(
                    a.kind,
                    super::DevFeedbackKind::FerramentaFaltando | super::DevFeedbackKind::Melhoria
                )
            {
                let found = crate::mcp::find::suggest(&a.title, SUGGEST_MIN_SCORE, 3);
                if !found.is_empty() {
                    v["existing_tools"] = to_value(found)?;
                }
            }
            Ok(v)
        }
        "bump" => {
            let a = parse_args::<DevFeedbackBump>(args)?;
            to_value(
                ctx.write(&reason("reforcar"), |tx| {
                    super::bump(tx, &a, &client, agent)
                })?
                .value,
            )
        }
        "resolve" => {
            let a = parse_args::<DevFeedbackResolve>(args)?;
            to_value(
                ctx.write(&reason("resolver"), |tx| {
                    super::resolve(tx, &a, &client, agent)
                })?
                .value,
            )
        }
        "set_status" => {
            let a = parse_args::<DevFeedbackStatusArgs>(args)?;
            // Descartar, para agentes, é escrita sensível: vira proposta na fila de aprovação.
            if agent && a.status == super::DevFeedbackStatus::Descartado {
                ctx.cmd = "devfeedback.discard";
            }
            let rec = ctx.write(&reason("status"), |tx| {
                super::set_status(tx, &a, &client, agent)
            })?;
            with_change_set(to_value(rec.value)?, rec.change_set_id)
        }
        "set_priority" => {
            let a = parse_args::<DevFeedbackPriority>(args)?;
            let rec = ctx.write(&reason("prioridade"), |tx| super::set_priority(tx, &a))?;
            with_change_set(to_value(rec.value)?, rec.change_set_id)
        }
        "update" => {
            let p = parse_args::<DevFeedbackPatch>(args)?;
            let rec = ctx.write(&reason("editar"), |tx| super::update(tx, &p, agent))?;
            with_change_set(to_value(rec.value)?, rec.change_set_id)
        }
        "link" => {
            let a = parse_args::<DevFeedbackLinkArgs>(args)?;
            let rec = ctx.write(&reason("vincular"), |tx| super::link(tx, &a, &client))?;
            with_change_set(to_value(rec.value)?, rec.change_set_id)
        }
        "unlink" => {
            let a = parse_args::<DevFeedbackUnlink>(args)?;
            let rec = ctx.write(&reason("desvincular"), |tx| super::unlink(tx, &a))?;
            with_change_set(to_value(rec.value)?, rec.change_set_id)
        }
        "export" => {
            let a = parse_args::<ExportArgs>(args)?;
            let dest = crate::export::check_destination(&a.path, ctx.paths)?;
            std::fs::write(&dest, a.text.as_bytes())?;
            Ok(serde_json::json!({
                "path": dest.display().to_string(),
                "bytes": a.text.len(),
            }))
        }
        "delete" => {
            let id = parse_args::<IdArg>(args)?.id;
            let rec = ctx.write(&reason("apagar"), |tx| super::delete(tx, &id))?;
            Ok(serde_json::json!({ "deleted": id, "change_set_id": rec.change_set_id }))
        }
        _ => Err(Error::UnknownCommand(format!("devfeedback.{action}"))),
    }
}

/// O feedback com o id do conjunto de alterações, para o "Desfazer" da tela.
fn with_change_set(mut v: Value, cs: Option<String>) -> Result<Value> {
    if let Value::Object(m) = &mut v {
        m.insert("change_set_id".into(), to_value(cs)?);
    }
    Ok(v)
}
