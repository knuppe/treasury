//! Comandos `profile.*`. Ler é livre (o agente usa o tom); mudar, só pela interface.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use super::{Level, Profile, Source, Tone};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};

pub const READ_ONLY: &[&str] = &["get", "questions"];

/// Membro da família (vazio: o perfil da família, de quem administra o cofre).
#[derive(Deserialize, Default)]
struct MemberArg {
    #[serde(default)]
    member_id: Option<String>,
}

#[derive(Deserialize)]
struct AnswerArgs {
    answers: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    member_id: Option<String>,
}

#[derive(Deserialize, Default)]
struct SetArgs {
    #[serde(default)]
    level: Option<Level>,
    #[serde(default)]
    tone: Option<Tone>,
    #[serde(default)]
    member_id: Option<String>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    if !READ_ONLY.contains(&action) && *ctx.actor != Actor::User {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-perfil-erro-so-interface",
        )));
    }
    match action {
        "get" => {
            let a: MemberArg = parse_args(args)?;
            to_value(super::get_for(ctx.conn(), a.member_id.as_deref())?)
        }
        "questions" => {
            let a: MemberArg = parse_args(args)?;
            to_value(super::questions_for(a.member_id.is_some()))
        }
        // Questionário respondido: nível e tom pelas respostas.
        "answer" => {
            let mut a: AnswerArgs = parse_args(args)?;
            // A linha da vida é da família: o questionário de um membro não mexe nela.
            if a.member_id.is_some() {
                a.answers.remove(super::LONGEVITY_QUESTION);
            }
            let p = super::from_answers(a.answers)?;
            super::save_for(ctx.conn(), a.member_id.as_deref(), &p)?;
            // "Sim, quero ver" liga a função; "Não" e "Decidir depois" deixam como está
            // (desligada num cofre novo).
            if a.member_id.is_none()
                && p.answers
                    .get(super::LONGEVITY_QUESTION)
                    .and_then(|v| v.first())
                    .is_some_and(|v| v == "yes")
            {
                crate::longevity::set_enabled(ctx.conn(), true)?;
            }
            to_value(p)
        }
        // "Já sei o que quero": avançado, sem perguntas.
        "skip" => {
            let a: MemberArg = parse_args(args)?;
            let p = Profile {
                level: Level::Avancado,
                tone: Tone::Tecnico,
                source: Source::Pulou,
                goals: vec![],
                income: None,
                answers: BTreeMap::new(),
                onboarding_pending: false,
                updated_at: Some(crate::time::now_utc()),
            };
            super::save_for(ctx.conn(), a.member_id.as_deref(), &p)?;
            to_value(p)
        }
        // Ajustes › Perfil: troca o nível ou o tom (as respostas ficam guardadas).
        "set" => {
            let a: SetArgs = parse_args(args)?;
            let member = a.member_id.as_deref();
            let mut p = super::get_for(ctx.conn(), member)?;
            if let Some(l) = a.level {
                if l != p.level {
                    p.source = Source::Escolha;
                }
                p.level = l;
            }
            if let Some(t) = a.tone {
                p.tone = t;
            }
            p.onboarding_pending = false;
            p.updated_at = Some(crate::time::now_utc());
            super::save_for(ctx.conn(), member, &p)?;
            to_value(p)
        }
        _ => Err(Error::UnknownCommand(format!("profile.{action}"))),
    }
}
