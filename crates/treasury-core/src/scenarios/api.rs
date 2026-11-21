//! `scenarios.*`: simular um cenário "e se" (lê) e os cenários salvos (salvar e apagar só
//! pela interface; ficam no ajuste `scenarios.saved`, fora do histórico).

use serde::Deserialize;
use serde_json::Value;

use super::{SAVED_SETTING, SavedScenario, ScenarioArgs, saved, simulate};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["simulate", "list"];

#[derive(Deserialize)]
struct NameArg {
    name: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "simulate" => to_value(simulate(
            ctx.conn(),
            &parse_args::<ScenarioArgs>(args)?,
            today_local(),
        )?),
        "list" => to_value(saved(ctx.conn())?),
        "save" | "delete" if *ctx.actor != Actor::User => Err(Error::Forbidden(t(
            "nucleo-planejamento-cenario-erro-so-interface",
        ))),
        "save" => {
            let s: SavedScenario = parse_args(args)?;
            let name = s.name.trim().to_string();
            if name.is_empty() {
                return Err(Error::Invalid(t(
                    "nucleo-planejamento-cenario-erro-sem-nome",
                )));
            }
            super::validate(&s.events)?;
            let mut all = saved(ctx.conn())?;
            all.retain(|x| x.name != name);
            all.push(SavedScenario {
                name,
                events: s.events,
            });
            all.sort_by(|a, b| a.name.cmp(&b.name));
            crate::settings::set(ctx.conn(), SAVED_SETTING, &all)?;
            to_value(all)
        }
        "delete" => {
            let name = parse_args::<NameArg>(args)?.name;
            let mut all = saved(ctx.conn())?;
            all.retain(|x| x.name != name);
            crate::settings::set(ctx.conn(), SAVED_SETTING, &all)?;
            to_value(all)
        }
        _ => Err(Error::UnknownCommand(format!("scenarios.{action}"))),
    }
}
