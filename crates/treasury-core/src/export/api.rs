use serde_json::Value;

use super::{ExportArgs, ExportCsvArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};

/// Exportar grava arquivos sem criptografia fora do cofre: nenhuma ação é somente leitura.
pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "json" => {
            let a: ExportArgs = parse_args(args)?;
            let version = ctx.db.schema_version()?;
            to_value(super::export_json(ctx.conn(), version, &a.path, ctx.paths)?)
        }
        "transactions_csv" => {
            let a: ExportCsvArgs = parse_args(args)?;
            to_value(super::export_transactions_csv(
                ctx.conn(),
                a.filter.as_ref(),
                &a.path,
                ctx.paths,
            )?)
        }
        _ => Err(Error::UnknownCommand(format!("export.{action}"))),
    }
}
