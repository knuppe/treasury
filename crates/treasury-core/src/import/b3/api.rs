use serde_json::Value;

use super::{B3CommitArgs, B3File, B3PreviewArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};

/// `detect` só lê o arquivo enviado; `preview` guarda estado na sessão e prepara uma
/// escrita, por isso não está aqui (mesma política de `import`).
pub const READ_ONLY: &[&str] = &["detect"];

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "detect" => to_value(super::detect(&parse_args::<B3File>(args)?)?),
        "preview" => to_value(super::preview(ctx, &parse_args::<B3PreviewArgs>(args)?)?),
        "commit" => to_value(super::commit(ctx, &parse_args::<B3CommitArgs>(args)?)?),
        _ => Err(Error::UnknownCommand(format!("b3.{action}"))),
    }
}
