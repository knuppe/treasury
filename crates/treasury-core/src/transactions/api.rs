use serde::Deserialize;
use serde_json::Value;

use super::{
    AttachmentInput, BulkResult, EditTagsArgs, SameDescriptionArgs, SetCategoryArgs, SetMemberArgs,
    SetStatusArgs, TagInput, TagPatch, TransferInput, TransferResult, TxnFilter, TxnInput, TxnKind,
    TxnPatch,
};
use crate::api::{Actor, Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::time::today_local;

pub const READ_ONLY: &[&str] = &[
    "query",
    "get",
    "list_tags",
    "suggest",
    "list_attachments",
    "get_attachment",
    "transfer_candidates",
    "transfer_match",
    "operation_candidates",
    "operation_link_preview",
    "same_description",
    "query_ids",
    "cross_account_duplicates",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct IdsArg {
    ids: Vec<String>,
}

#[derive(Deserialize)]
struct CandidatesArg {
    txn_id: Option<String>,
    txn_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct TxnIdArg {
    txn_id: String,
}

/// Descrição curta para o motivo do histórico.
pub(crate) fn short(desc: &str) -> String {
    let d = desc.trim();
    if d.chars().count() > 60 {
        format!("{}…", d.chars().take(59).collect::<String>())
    } else if d.is_empty() {
        t("nucleo-lancamentos-sem-descricao")
    } else {
        d.to_string()
    }
}

/// Motivo do histórico com a descrição curta (`{ $descricao }`).
pub(crate) fn with_desc(key: &str, desc: &str) -> String {
    t_args(key, &i18n::args(&[("descricao", short(desc).into())]))
}

/// Motivo do histórico com uma contagem (`{ $n }`).
pub(crate) fn with_n(key: &str, n: usize) -> String {
    t_args(key, &i18n::args(&[("n", n.into())]))
}

/// Destino do anexo salvo pela interface.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveAttachmentArgs {
    id: String,
    path: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let today = today_local();
    match action {
        "query" => to_value(super::query(ctx.conn(), &parse_args::<TxnFilter>(args)?)?),
        "get" => to_value(super::get(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "create" => {
            let input: TxnInput = parse_args(args)?;
            let reason = with_desc("nucleo-lancamentos-historico-criado", &input.description);
            let id = ctx
                .write(&reason, |tx| {
                    let id = super::create(tx, &input, today)?;
                    crate::debt::link_category_payments(tx, std::slice::from_ref(&id))?;
                    Ok(id)
                })?
                .value;
            to_value(super::get(ctx.conn(), &id)?)
        }
        "update" => {
            let p: TxnPatch = parse_args(args)?;
            let cur = super::get(ctx.conn(), &p.id)?;
            let reason = with_desc("nucleo-lancamentos-historico-alterado", &cur.description);
            ctx.write(&reason, |tx| {
                super::update(tx, &p, today)?;
                crate::debt::link_category_payments(tx, std::slice::from_ref(&p.id))?;
                Ok(())
            })?;
            to_value(super::get(ctx.conn(), &p.id)?)
        }
        "delete" => {
            let a: IdsArg = parse_args(args)?;
            if a.ids.is_empty() {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-nenhum-selecionado",
                )));
            }
            let reason = if a.ids.len() == 1 {
                let cur = super::get(ctx.conn(), &a.ids[0])?;
                with_desc("nucleo-lancamentos-historico-apagado", &cur.description)
            } else {
                with_n("nucleo-lancamentos-historico-apagados", a.ids.len())
            };
            let out = ctx.write(&reason, |tx| super::delete(tx, &a.ids, today))?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "create_transfer" => {
            let input: TransferInput = parse_args(args)?;
            let key = match input.kind {
                Some(TxnKind::CardPayment) => "nucleo-lancamentos-historico-pagamento-fatura",
                Some(TxnKind::Investment) => "nucleo-lancamentos-historico-aporte",
                _ => "nucleo-lancamentos-historico-transferencia",
            };
            let name = |id: &str| -> String {
                ctx.conn()
                    .query_row("SELECT name FROM account WHERE id = ?1", [id], |r| r.get(0))
                    .unwrap_or_else(|_| "?".into())
            };
            let reason = t_args(
                key,
                &i18n::args(&[
                    ("origem", name(&input.from_account_id).into()),
                    ("destino", name(&input.to_account_id).into()),
                ]),
            );
            let ids = ctx
                .write(&reason, |tx| super::create_transfer(tx, &input, today))?
                .value;
            to_value(TransferResult {
                transfer_id: ids.transfer_id,
                from: super::get(ctx.conn(), &ids.from_id)?,
                to: super::get(ctx.conn(), &ids.to_id)?,
            })
        }
        "set_category" => {
            let a: SetCategoryArgs = parse_args(args)?;
            let reason = match &a.category_id {
                Some(c) => t_args(
                    "nucleo-lancamentos-historico-categoria-definida",
                    &i18n::args(&[
                        (
                            "categoria",
                            crate::categories::get(ctx.conn(), c)?.path.into(),
                        ),
                        ("n", a.ids.len().into()),
                    ]),
                ),
                None => with_n(
                    "nucleo-lancamentos-historico-categoria-removida",
                    a.ids.len(),
                ),
            };
            let out = ctx.write(&reason, |tx| {
                let n = super::set_category(tx, &a.ids, a.category_id.as_deref())?;
                crate::debt::link_category_payments(tx, &a.ids)?;
                Ok(n)
            })?;
            to_value(BulkResult {
                updated: out.value.0 as i64,
                change_set_id: out.change_set_id,
                skipped: out.value.1 as i64,
            })
        }
        "set_status" => {
            let a: SetStatusArgs = parse_args(args)?;
            let reason = t_args(
                "nucleo-lancamentos-historico-status",
                &i18n::args(&[
                    ("status", super::status_label(a.status).into()),
                    ("n", a.ids.len().into()),
                ]),
            );
            let out = ctx.write(&reason, |tx| super::set_status(tx, &a.ids, a.status, today))?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "set_member" => {
            let a: SetMemberArgs = parse_args(args)?;
            let reason = match &a.member_id {
                Some(m) => t_args(
                    "nucleo-lancamentos-historico-membro-definido",
                    &i18n::args(&[
                        (
                            "membro",
                            ctx.conn()
                                .query_row("SELECT name FROM member WHERE id = ?1", [m], |r| {
                                    r.get::<_, String>(0)
                                })
                                .map_err(|_| Error::not_found(super::util::member_ref(m)))?
                                .into(),
                        ),
                        ("n", a.ids.len().into()),
                    ]),
                ),
                None => with_n("nucleo-lancamentos-historico-membro-removido", a.ids.len()),
            };
            let out = ctx.write(&reason, |tx| {
                super::set_member(tx, &a.ids, a.member_id.as_deref())
            })?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "edit_tags" => {
            let a: EditTagsArgs = parse_args(args)?;
            let reason = with_n("nucleo-lancamentos-historico-tags", a.ids.len());
            let out = ctx.write(&reason, |tx| super::edit_tags(tx, &a))?;
            to_value(BulkResult {
                updated: out.value as i64,
                change_set_id: out.change_set_id,
                skipped: 0,
            })
        }
        "same_description" => to_value(super::same_description(
            ctx.conn(),
            &parse_args::<SameDescriptionArgs>(args)?,
        )?),
        "query_ids" => {
            let mut f: TxnFilter = parse_args(args)?;
            f.limit = None;
            f.offset = None;
            to_value(super::query_ids(ctx.conn(), &f)?)
        }
        "list_tags" => to_value(super::list_tags(ctx.conn())?),
        "transfer_candidates" => to_value(super::transfers::candidates(
            ctx.conn(),
            &parse_args(args)?,
            today,
        )?),
        "transfer_match" => to_value(super::transfers::match_account(
            ctx.conn(),
            &parse_args(args)?,
        )?),
        "link_transfers" => {
            let a: super::transfers::LinkTransfersArgs = parse_args(args)?;
            if a.pairs.is_empty() {
                return Err(Error::invalid(t("nucleo-lancamentos-erro-nenhum-par")));
            }
            let reason = if a.pairs.len() == 1 {
                t("nucleo-lancamentos-historico-par-ligado")
            } else {
                with_n("nucleo-lancamentos-historico-pares-ligados", a.pairs.len())
            };
            let out = ctx.write(&reason, |tx| {
                for p in &a.pairs {
                    super::transfers::link_pair(tx, &p.from_id, &p.to_id, today)?;
                }
                Ok(a.pairs.len() as u32)
            })?;
            // "Sempre ligar entre estas contas" é preferência do usuário.
            if a.remember.unwrap_or(false) && *ctx.actor == crate::api::Actor::User {
                let accounts: Vec<(String, String)> = a
                    .pairs
                    .iter()
                    .map(|p| {
                        let acc = |id: &str| -> Result<String> {
                            Ok(ctx.conn().query_row(
                                "SELECT account_id FROM txn WHERE id = ?1",
                                [id],
                                |r| r.get(0),
                            )?)
                        };
                        Ok((acc(&p.from_id)?, acc(&p.to_id)?))
                    })
                    .collect::<Result<_>>()?;
                super::transfers::remember_accounts(ctx.conn(), &accounts)?;
            }
            to_value(super::transfers::LinkTransfersResult {
                linked: out.value,
                change_set_id: out.change_set_id,
            })
        }
        "operation_candidates" => {
            // Um lançamento (`txn_id`) ou vários (`txn_ids`, para o vínculo em lote).
            let a: CandidatesArg = parse_args(args)?;
            let ids: Vec<String> = a
                .txn_ids
                .unwrap_or_default()
                .into_iter()
                .chain(a.txn_id)
                .collect();
            if ids.is_empty() {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-informe-lancamento",
                )));
            }
            to_value(super::operations::candidates_many(ctx.conn(), &ids)?)
        }
        "operation_link_preview" => to_value(super::operations::preview_many(
            ctx.conn(),
            &parse_args(args)?,
        )?),
        "link_operations" => {
            let a: super::operations::BulkLinkArgs = parse_args(args)?;
            let target = super::operations::label(ctx.conn(), a.kind, &a.target_id)?
                .unwrap_or_else(|| t("nucleo-lancamentos-operacao"));
            let n = super::util::dedup_ids(&a.txn_ids).len();
            let reason = t_args(
                "nucleo-lancamentos-historico-vinculados",
                &i18n::args(&[("n", n.into()), ("operacao", target.into())]),
            );
            let out = ctx.write(&reason, |tx| super::operations::link_many(tx, &a))?;
            let (label, linked, skipped) = out.value;
            to_value(super::operations::BulkLinkResult {
                label,
                linked,
                skipped,
                change_set_id: out.change_set_id,
            })
        }
        "link_operation" => {
            let a: super::operations::LinkArgs = parse_args(args)?;
            let target = super::operations::label(ctx.conn(), a.kind, &a.target_id)?
                .unwrap_or_else(|| t("nucleo-lancamentos-operacao"));
            let desc = short(&super::load_row(ctx.conn(), &a.txn_id)?.description);
            let reason = t_args(
                "nucleo-lancamentos-historico-vinculado",
                &i18n::args(&[("descricao", desc.into()), ("operacao", target.into())]),
            );
            let out = ctx.write(&reason, |tx| super::operations::link(tx, &a))?;
            to_value(super::operations::LinkResult {
                operation: out.value,
                change_set_id: out.change_set_id,
            })
        }
        "unlink_operation" => {
            let a: TxnIdArg = parse_args(args)?;
            let desc = short(&super::load_row(ctx.conn(), &a.txn_id)?.description);
            let reason = t_args(
                "nucleo-lancamentos-historico-desvinculado",
                &i18n::args(&[("descricao", desc.into())]),
            );
            let out = ctx.write(&reason, |tx| super::operations::unlink(tx, &a.txn_id))?;
            to_value(serde_json::json!({ "change_set_id": out.change_set_id }))
        }
        "unlink_transfer" => {
            let a: super::transfers::UnlinkTransferArgs = parse_args(args)?;
            let out = ctx.write(
                &t("nucleo-lancamentos-historico-transferencia-desligada"),
                |tx| super::transfers::unlink(tx, &a.txn_id, today),
            )?;
            // Recusar o par é preferência do usuário (o agente não mexe nela).
            if a.dismiss.unwrap_or(true)
                && *ctx.actor == crate::api::Actor::User
                && let [from, to] = out.value.as_slice()
            {
                super::transfers::dismiss(
                    ctx.conn(),
                    &super::transfers::TransferPair {
                        from_id: from.clone(),
                        to_id: to.clone(),
                    },
                )?;
            }
            to_value(super::transfers::UnlinkTransferResult {
                txn_ids: out.value,
                change_set_id: out.change_set_id,
            })
        }
        "dismiss_transfer" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(t(
                    "nucleo-lancamentos-erro-recusar-so-interface",
                )));
            }
            super::transfers::dismiss(ctx.conn(), &parse_args(args)?)?;
            Ok(Value::Null)
        }
        "forget_transfer_accounts" => {
            #[derive(Deserialize)]
            struct A {
                a: String,
                b: String,
            }
            let x: A = parse_args(args)?;
            super::transfers::forget_accounts(ctx.conn(), &x.a, &x.b)?;
            Ok(Value::Null)
        }
        "suggest" => to_value(super::suggest::suggest(
            ctx.conn(),
            &parse_args::<super::suggest::TxnSuggestArgs>(args)?,
        )?),
        "create_tag" => {
            let input: TagInput = parse_args(args)?;
            let reason = t_args(
                "nucleo-lancamentos-historico-tag-criada",
                &i18n::args(&[("nome", input.name.trim().to_string().into())]),
            );
            to_value(
                ctx.write(&reason, |tx| super::create_tag(tx, &input))?
                    .value,
            )
        }
        "update_tag" => {
            let p: TagPatch = parse_args(args)?;
            to_value(
                ctx.write(&t("nucleo-lancamentos-historico-tag-alterada"), |tx| {
                    super::update_tag(tx, &p)
                })?
                .value,
            )
        }
        "delete_tag" => {
            let a: IdArg = parse_args(args)?;
            let name = super::get_tag(ctx.conn(), &a.id)?.name;
            let reason = t_args(
                "nucleo-lancamentos-historico-tag-apagada",
                &i18n::args(&[("nome", name.into())]),
            );
            ctx.write(&reason, |tx| super::delete_tag(tx, &a.id))?;
            Ok(Value::Null)
        }
        "list_attachments" => to_value(super::list_attachments(
            ctx.conn(),
            &parse_args::<TxnIdArg>(args)?.txn_id,
        )?),
        "add_attachment" => {
            let input: AttachmentInput = parse_args(args)?;
            let reason = with_desc(
                "nucleo-lancamentos-historico-anexo-adicionado",
                &input.filename,
            );
            to_value(
                ctx.write(&reason, |tx| super::add_attachment(tx, &input))?
                    .value,
            )
        }
        "get_attachment" => to_value(super::get_attachment(
            ctx.conn(),
            &parse_args::<IdArg>(args)?.id,
        )?),
        "cross_account_duplicates" => to_value(super::duplicates::list(
            ctx.conn(),
            &parse_args::<super::duplicates::CrossAccountDuplicatesArgs>(args)?,
        )?),
        "remove_copies" => {
            // Apagar cópias entre contas: a pessoa revisou e confirmou na tela. Só a
            // interface (o agente propõe com as ferramentas de apagar de sempre).
            if *ctx.actor != Actor::User {
                return Err(Error::Forbidden(t(
                    "nucleo-lancamentos-erro-copias-so-interface",
                )));
            }
            let a: IdsArg = parse_args(args)?;
            let (ok, skipped) = super::duplicates::removable(ctx.conn(), &a.ids)?;
            if ok.is_empty() {
                return to_value(super::duplicates::RemoveCopiesResult {
                    removed: 0,
                    balance_syncs_adjusted: 0,
                    skipped,
                    change_set_id: None,
                });
            }
            let reason = with_n("nucleo-lancamentos-historico-copias-apagadas", ok.len());
            let out = ctx.write(&reason, |tx| super::duplicates::remove(tx, &ok, today))?;
            to_value(super::duplicates::RemoveCopiesResult {
                removed: out.value.0 as i64,
                balance_syncs_adjusted: out.value.1 as i64,
                skipped,
                change_set_id: out.change_set_id,
            })
        }
        "save_attachment" => {
            // Salvar no disco o anexo, no caminho escolhido no diálogo de salvar (o WebKit do
            // app não baixa links de Blob). Só a interface: agente não grava arquivos.
            if *ctx.actor != Actor::User {
                return Err(Error::Forbidden(t(
                    "nucleo-lancamentos-erro-anexo-so-interface",
                )));
            }
            let a: SaveAttachmentArgs = parse_args(args)?;
            let dest = crate::export::check_destination(&a.path, ctx.paths)?;
            // O cadastro primeiro (anexo inexistente sai com a mensagem de sempre).
            super::attachment_meta(ctx.conn(), &a.id)?;
            let data: Vec<u8> = ctx.conn().query_row(
                "SELECT data FROM attachment WHERE id = ?1",
                [&a.id],
                |r| r.get(0),
            )?;
            std::fs::write(&dest, &data)?;
            Ok(serde_json::json!({"path": dest.display().to_string(), "bytes": data.len()}))
        }
        "delete_attachment" => {
            let a: IdArg = parse_args(args)?;
            let name = super::attachment_meta(ctx.conn(), &a.id)?.filename;
            ctx.write(
                &with_desc("nucleo-lancamentos-historico-anexo-apagado", &name),
                |tx| super::delete_attachment(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("transactions.{action}"))),
    }
}
