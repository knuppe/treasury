use serde_json::Value;

use super::{DetectArgs, ImportCommitArgs, ImportPreviewArgs, ListBatchesArgs};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t};

/// `detect` só lê o arquivo enviado; `preview` guarda estado na sessão e prepara uma
/// escrita, por isso não está aqui.
pub const READ_ONLY: &[&str] = &[
    "list_batches",
    "list_presets",
    "detect",
    "sniff",
    "layouts",
    "card_pdf_preview",
    "inbox_list",
    "inbox_file",
    "read_document",
];

/// Sem `account_id`, um OFX escolhe a conta sozinho pelo banco, agência e número
/// (`accounts::match_ofx`); um CSV continua precisando dela.
fn with_ofx_account(ctx: &Ctx<'_>, mut args: Value) -> Result<Value> {
    let given = args
        .get("account_id")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.trim().is_empty());
    if given {
        return Ok(args);
    }
    let d: DetectArgs = parse_args(args.clone())?;
    let det = super::detect(&d)?;
    let Some(ofx) = det.ofx.filter(|_| det.format == super::ImportFormat::Ofx) else {
        return Err(Error::invalid(t("nucleo-importacao-erro-sem-conta-csv")));
    };
    let m = crate::accounts::match_ofx(
        ctx.conn(),
        &crate::accounts::OfxMatchArgs {
            bank_id: ofx.bank_id,
            branch_id: ofx.branch_id,
            account_id: ofx.account_id,
            credit_card: Some(ofx.credit_card),
        },
        crate::time::today_local(),
    )?;
    match m.account_id {
        Some(id) => {
            args["account_id"] = Value::String(id.to_string());
            Ok(args)
        }
        None => Err(Error::invalid(i18n::t_args(
            "nucleo-importacao-erro-sem-conta",
            &i18n::args(&[(
                "motivo",
                m.reason
                    .unwrap_or_else(|| t("nucleo-importacao-conta-nao-reconhecida"))
                    .into(),
            )]),
        ))),
    }
}

/// Grava uma prévia (lançamentos lidos de uma página ou de um arquivo) com o relatório do
/// que ficou de fora; para a IA, vira uma proposta só.
fn commit_preview(ctx: &mut Ctx<'_>, prev: super::ImportPreview, include: bool) -> Result<Value> {
    let skipped: Vec<Value> = prev
        .rows
        .iter()
        .filter(|r| {
            !matches!(r.status, super::RowStatus::New)
                && !(include && r.status == super::RowStatus::PossibleDuplicate)
        })
        .take(40)
        .map(|r| {
            serde_json::json!({
                "line": r.line,
                "status": r.status,
                "description": r.description,
                "note": r.note.clone().or_else(|| r.error.clone()),
            })
        })
        .collect();
    let report = serde_json::json!({
        "counts": prev.counts,
        "warnings": prev.warnings,
        "not_imported": skipped,
    });
    let new = prev.counts.new
        + if include {
            prev.counts.possible_duplicate
        } else {
            0
        };
    if new == 0 {
        return Ok(serde_json::json!({
            "status": "nothing_new",
            "message": t("nucleo-importacao-nada-novo"),
            "report": report,
        }));
    }
    ctx.set_report(report.clone());
    let res = super::commit(
        ctx,
        &ImportCommitArgs {
            token: prev.token,
            skip_indexes: None,
            include_possible_duplicates: Some(include),
            overrides: None,
        },
    )?;
    Ok(serde_json::json!({ "status": "applied", "result": res, "report": report }))
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "detect" => {
            let mut a: DetectArgs = parse_args(args)?;
            a.country = Some(crate::country::family_country(ctx.conn()));
            to_value(super::detect(&a)?)
        }
        "sniff" => to_value(super::sniff::sniff(&parse_args(args)?)?),
        // Documento em Markdown para o agente (docs/pdf-markdown.md).
        "read_document" => to_value(crate::pdf_markdown::read_document(&parse_args(args)?)?),
        // Caixa de entrada de arquivos: lê a pasta antes de listar (o que chegou entra agora).
        "inbox_list" => {
            super::inbox::scan(ctx.conn())?;
            to_value(super::inbox::list(ctx.conn(), &parse_args(args)?)?)
        }
        "inbox_scan" => to_value(super::inbox::scan(ctx.conn())?),
        "inbox_file" => {
            #[derive(serde::Deserialize)]
            struct A {
                id: String,
            }
            to_value(super::inbox::file(ctx.conn(), &parse_args::<A>(args)?.id)?)
        }
        "inbox_set_status" => {
            #[derive(serde::Deserialize)]
            struct A {
                id: String,
                status: super::inbox::InboxStatus,
            }
            let a: A = parse_args(args)?;
            to_value(super::inbox::set_status(ctx.conn(), &a.id, a.status)?)
        }
        // Layouts que o app sabe ler (docs/layouts.md).
        "layouts" => to_value(crate::layouts::known(None)),
        "card_pdf_preview" => to_value(super::card_pdf_preview(&parse_args(args)?)?),
        "preview" => to_value(super::preview(
            ctx,
            &parse_args::<ImportPreviewArgs>(args)?,
        )?),
        "commit" => to_value(super::commit(ctx, &parse_args::<ImportCommitArgs>(args)?)?),
        "rows" => {
            let a: super::RowsImportArgs = parse_args(args)?;
            let include = a.include_possible_duplicates.unwrap_or(false);
            let prev = super::preview_rows(ctx, &a)?;
            commit_preview(ctx, prev, include)
        }
        // Extrato em arquivo (OFX ou CSV) lido e gravado numa chamada (agentes e MCP).
        "file" => {
            #[derive(serde::Deserialize)]
            struct A {
                #[serde(flatten)]
                preview: ImportPreviewArgs,
                #[serde(default)]
                include_possible_duplicates: Option<bool>,
            }
            let a: A = parse_args(with_ofx_account(ctx, args)?)?;
            let include = a.include_possible_duplicates.unwrap_or(false);
            let prev = super::preview(ctx, &a.preview)?;
            if prev.already_imported {
                return Ok(serde_json::json!({
                    "status": "nothing_new",
                    "message": t("nucleo-importacao-arquivo-repetido"),
                    "report": {"counts": prev.counts, "warnings": prev.warnings},
                }));
            }
            commit_preview(ctx, prev, include)
        }
        "list_batches" => to_value(super::list_batches(
            ctx.conn(),
            &parse_args::<ListBatchesArgs>(args)?,
        )?),
        "list_presets" => to_value(super::csv_file::presets()),
        _ => Err(Error::UnknownCommand(format!("import.{action}"))),
    }
}
