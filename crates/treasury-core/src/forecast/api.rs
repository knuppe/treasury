//! `forecast.*`: previsão de saldo e "quanto posso gastar hoje" (só leitura).

use serde_json::Value;

use super::ForecastArgs;
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &["balance", "daily_allowance"];

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "balance" => to_value(super::forecast(
            ctx.conn(),
            &parse_args::<ForecastArgs>(args)?,
            today_local(),
        )?),
        "daily_allowance" => to_value(super::daily_allowance(ctx.conn(), today_local())?),
        _ => Err(Error::UnknownCommand(format!("forecast.{action}"))),
    }
}
