//! `retirement.*` (docs/international.md): os planos de aposentadoria do país e quanto entrou
//! no ano em cada conta contra o limite.

use chrono::Datelike;
use serde::Deserialize;
use serde_json::Value;

use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["plans", "status", "statuses"];

#[derive(Deserialize)]
struct AccountArg {
    account_id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let today = today_local();
    match action {
        "plans" => {
            let country = crate::country::family_country(ctx.conn());
            to_value(super::plans(Some(&country), today.year()))
        }
        "status" => {
            let a: AccountArg = parse_args(args)?;
            to_value(super::status(ctx.conn(), &a.account_id, today)?)
        }
        "statuses" => to_value(super::statuses(ctx.conn(), today)?),
        _ => Err(Error::UnknownCommand(format!("retirement.{action}"))),
    }
}
