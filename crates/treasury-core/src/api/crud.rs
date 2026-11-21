//! `crud.delete`: apagar qualquer registro da família por um comando só (`kind` + `id`),
//! repassado ao comando de apagar do módulo dono. Para a IA, a escrita usa a política do
//! comando de destino (apagar é sempre **sensível**: aprovação item a item, nunca no modo
//! autônomo), e a proposta mostra todas as linhas afetadas, inclusive o que cai junto.

use serde::Deserialize;
use serde_json::{Value, json};
use ts_rs::TS;

use super::{Ctx, dispatch, parse_args};
use crate::error::{Error, Result};

/// O que pode ser apagado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DeleteKind {
    Transaction,
    Account,
    Asset,
    Category,
    Rule,
    Recurrence,
    Goal,
    Trade,
    Income,
    CorporateAction,
    Institution,
    Loan,
    Tag,
    PhysicalAsset,
    PhysicalImprovement,
    PhysicalValuation,
}

impl DeleteKind {
    /// Comando de destino (`módulo.ação`).
    pub fn command(self) -> &'static str {
        match self {
            DeleteKind::Transaction => "transactions.delete",
            DeleteKind::Account => "accounts.delete",
            DeleteKind::Asset => "assets.delete",
            DeleteKind::Category => "categories.delete",
            DeleteKind::Rule => "rules.delete",
            DeleteKind::Recurrence => "recurrences.delete",
            DeleteKind::Goal => "goals.delete",
            DeleteKind::Trade => "portfolio.delete_trade",
            DeleteKind::Income => "portfolio.delete_income",
            DeleteKind::CorporateAction => "portfolio.delete_action",
            DeleteKind::Institution => "household.delete_institution",
            DeleteKind::Loan => "debt.delete",
            DeleteKind::Tag => "transactions.delete_tag",
            DeleteKind::PhysicalAsset => "physical.delete",
            DeleteKind::PhysicalImprovement => "physical.delete_improvement",
            DeleteKind::PhysicalValuation => "physical.delete_valuation",
        }
    }
}

#[derive(Deserialize)]
struct DeleteArgs {
    kind: DeleteKind,
    id: String,
    /// Categoria: quem recebe o que era da apagada.
    #[serde(default)]
    reassign_to: Option<String>,
    /// Recorrência: apagar também os previstos gerados (padrão: sim).
    #[serde(default)]
    delete_planned: Option<bool>,
}

pub(super) fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "delete" => {
            let a: DeleteArgs = parse_args(args)?;
            if a.id.trim().is_empty() {
                return Err(Error::invalid(crate::i18n::t(
                    "nucleo-base-erro-informe-id",
                )));
            }
            let cmd = a.kind.command();
            let mut target = match a.kind {
                DeleteKind::Transaction => json!({ "ids": [a.id] }),
                _ => json!({ "id": a.id }),
            };
            if let Some(r) = a.reassign_to.filter(|_| a.kind == DeleteKind::Category) {
                target["reassign_to"] = json!(r);
            }
            if let Some(p) = a
                .delete_planned
                .filter(|_| a.kind == DeleteKind::Recurrence)
            {
                target["delete_planned"] = json!(p);
            }
            let (module, action) = cmd.split_once('.').expect("comando com módulo");
            let mut sub = Ctx {
                db: &mut *ctx.db,
                paths: ctx.paths,
                actor: ctx.actor,
                scratch: &mut *ctx.scratch,
                cmd,
            };
            dispatch(&mut sub, module, action, target)
        }
        _ => Err(Error::UnknownCommand(format!("crud.{action}"))),
    }
}
