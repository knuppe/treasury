use serde_json::Value;

use super::{NoteCommitArgs, NotePreviewArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};

/// A prévia só lê os arquivos e o banco.
pub const READ_ONLY: &[&str] = &["preview"];

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "preview" => to_value(super::preview(ctx, &parse_args::<NotePreviewArgs>(args)?)?),
        "commit" => to_value(super::commit(ctx, &parse_args::<NoteCommitArgs>(args)?)?),
        _ => Err(Error::UnknownCommand(format!("notes.{action}"))),
    }
}
