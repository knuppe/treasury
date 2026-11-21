use serde::Deserialize;
use serde_json::Value;

use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const READ_ONLY: &[&str] = &[
    "list",
    "convert",
    "exchange_quote",
    "operations",
    "reference_rate",
];

#[derive(Deserialize, Default)]
struct ListArgs {
    base: Option<String>,
    quote: Option<String>,
    limit: Option<i64>,
}

#[derive(Deserialize, Default)]
struct OpsArgs {
    account_id: Option<String>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "exchange_quote" => to_value(super::exchange::quote(ctx.conn(), &parse_args(args)?)?),
        "operations" => {
            let a: OpsArgs = parse_args(args)?;
            to_value(super::exchange::list(ctx.conn(), a.account_id.as_deref())?)
        }
        "exchange" => {
            let a: super::exchange::ExchangeArgs = parse_args(args)?;
            let q = super::exchange::quote(ctx.conn(), &a)?;
            let reason = t_args(
                "nucleo-contas-historico-cambio",
                &i18n::args(&[
                    ("origem", q.from_currency.clone().into()),
                    ("destino", q.to_currency.clone().into()),
                    ("data", crate::transactions::util::date_text(&a.date).into()),
                ]),
            );
            let today = crate::time::today_local();
            let id = ctx
                .write(&reason, |tx| super::exchange::execute(tx, &a, today))?
                .value;
            let ops = super::exchange::list(ctx.conn(), None)?;
            to_value(ops.into_iter().find(|o| o.id == id))
        }
        "list" => {
            let a: ListArgs = parse_args(args)?;
            let mut stmt = ctx.conn().prepare(
                "SELECT id, base, quote, date, rate, source, fetched_at FROM fx_rate
                 WHERE (?1 IS NULL OR base = ?1) AND (?2 IS NULL OR quote = ?2) ORDER BY date DESC LIMIT ?3",
            )?;
            let rows = stmt
                .query_map(
                    rusqlite::params![a.base, a.quote, a.limit.unwrap_or(200)],
                    super::row_to_rate,
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            to_value(rows)
        }
        "convert" => {
            let a: super::ConvertArgs = parse_args(args)?;
            to_value(super::convert(
                ctx.conn(),
                a.amount,
                &a.from,
                &a.to,
                &a.date,
            )?)
        }
        "set" => {
            let input: super::FxRateInput = parse_args(args)?;
            let out = ctx.write(&t("nucleo-contas-historico-cambio-manual"), |tx| {
                super::set_manual(tx, &input)
            })?;
            to_value(out.value)
        }
        "delete" => {
            #[derive(Deserialize)]
            struct IdArg {
                id: String,
            }
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-contas-historico-cambio-removido"), |tx| {
                let n = tx.execute("DELETE FROM fx_rate WHERE id = ?1", [&a.id])?;
                if n == 0 {
                    return Err(Error::not_found(t("nucleo-contas-ref-taxa-cambio")));
                }
                Ok(())
            })?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("fx.{action}"))),
    }
}
