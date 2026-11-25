use serde::Deserialize;
use serde_json::Value;

use super::*;
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const READ_ONLY: &[&str] = &[
    "summary",
    "position",
    "performance",
    "bridge",
    "benchmark_series",
    "contributions",
    "list_trades",
    "trade_summary",
    "get_trade",
    "list_actions",
    "list_income",
    "list_targets",
    "rebalance",
    "list_watchlist",
    "dividend_calendar",
    "passive_income",
    "pending_income",
    "income_ledger",
    "come_cotas",
    "fee_cost",
    "concentration",
    "compare_fixed_income",
    "simulate_fixed_income_sale",
    "risk",
    "quick_income_preview",
    "statement",
    "sales",
    "uncovered_sales",
    "cost_method",
    "plans",
    "plans_pending",
];

#[derive(Deserialize)]
struct ComeCotasArgs {
    asset_id: String,
    #[serde(default)]
    account_id: Option<String>,
}

#[derive(Deserialize)]
struct SetComeCotasArgs {
    asset_id: String,
    /// `None` volta à regra automática.
    regime: Option<come_cotas::Regime>,
}

#[derive(Deserialize)]
struct IncomesArgs {
    items: Vec<IncomeInput>,
}

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
struct IdsArg {
    ids: Vec<String>,
}

/// Motivo de apagar `n` registros: a chave tem a variante de um registro só e a do plural.
fn removed(key: &str, n: usize) -> String {
    t_args(key, &i18n::args(&[("n", (n as i64).into())]))
}

/// Apaga vários registros num conjunto de alterações só: um desfazer restaura todos, e um
/// id que não existe cancela o lote inteiro. `key`: o motivo, com a contagem em `$n`.
fn delete_many(
    ctx: &mut Ctx<'_>,
    args: Value,
    key: &str,
    delete: fn(&rusqlite::Transaction<'_>, &str) -> Result<()>,
) -> Result<Value> {
    let mut seen = std::collections::HashSet::new();
    let ids: Vec<String> = parse_args::<IdsArg>(args)?
        .ids
        .into_iter()
        .filter(|id| seen.insert(id.clone()))
        .collect();
    if ids.is_empty() {
        return Err(Error::invalid(t("nucleo-carteira-erro-nada-selecionado")));
    }
    let reason = removed(key, ids.len());
    let r = ctx.write(&reason, |tx| {
        for id in &ids {
            delete(tx, id)?;
        }
        Ok(ids.len())
    })?;
    to_value(crate::transactions::BulkResult {
        updated: r.value as i64,
        change_set_id: r.change_set_id,
        skipped: 0,
    })
}

#[derive(Deserialize, Default)]
struct AssetOpt {
    asset_id: Option<String>,
}

#[derive(Deserialize, Default)]
struct IncomeListArgs {
    asset_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Deserialize)]
struct PositionArgs {
    asset_id: String,
    account_id: Option<String>,
    as_of: Option<String>,
    deflator: Option<crate::indices::Deflator>,
}

#[derive(Deserialize)]
struct SetTargetsArgs {
    scope: TargetScope,
    targets: Vec<TargetInput>,
}

#[derive(Deserialize)]
struct AssetIdArg {
    asset_id: String,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "summary" => to_value(engine::summary(
            ctx.conn(),
            &parse_args::<PortfolioArgs>(args)?,
        )?),
        "position" => {
            let a: PositionArgs = parse_args(args)?;
            let at = match &a.as_of {
                Some(d) => crate::time::parse_date(d)?,
                None => crate::time::today_local(),
            };
            let accounts = a.account_id.clone().map(|x| vec![x]);
            let book = engine::load_book(ctx.conn(), &a.asset_id, accounts.as_deref())?;
            let (mut pv, _) = engine::position_view(
                ctx.conn(),
                &book,
                at,
                a.deflator.unwrap_or_default(),
                a.account_id,
            )?;
            let pending: i64 = crate::pension::payments::pending_by_asset(
                ctx.conn(),
                &crate::time::fmt_date(at),
                accounts.as_deref(),
            )?
            .into_iter()
            .filter(|p| p.0 == a.asset_id)
            .map(|p| p.2)
            .sum();
            if pending > 0 {
                engine::add_pending(&mut pv, pending);
            }
            to_value(pv)
        }
        // Método de custo das vendas de renda variável (o do país da família ou o de Ajustes).
        "cost_method" => to_value(super::cost_method::state(ctx.conn())?),
        "set_cost_method" => {
            let a: super::cost_method::SetCostMethodArgs = parse_args(args)?;
            let out = ctx.write(&t("nucleo-carteira-motivo-metodo-custo"), |tx| {
                super::cost_method::set(tx, &a)
            })?;
            to_value(out.value)
        }
        // Livro de vendas: resultado nominal e real de cada venda.
        "sales" => to_value(super::sales::sales(
            ctx.conn(),
            &parse_args::<super::sales::SalesArgs>(args)?,
        )?),
        // Sales larger than the position on their date (the purchase is missing).
        "uncovered_sales" => to_value(super::uncovered::list(
            ctx.conn(),
            &parse_args::<super::uncovered::UncoveredSalesArgs>(args)?,
        )?),
        "accept_uncovered_sale" => {
            let a: super::uncovered::UncoveredSaleAcceptArgs = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-motivo-venda-sem-compra"), |tx| {
                super::uncovered::accept(tx, &a)
            })?;
            to_value(super::uncovered::list(
                ctx.conn(),
                &super::uncovered::UncoveredSalesArgs::default(),
            )?)
        }
        "bridge" => to_value(super::bridge::bridge(ctx.conn(), &parse_args(args)?)?),
        "performance" => to_value(engine::performance(
            ctx.conn(),
            &parse_args::<PerformanceArgs>(args)?,
        )?),
        "rebalance" => to_value(engine::rebalance(
            ctx.conn(),
            &parse_args::<RebalanceArgs>(args)?,
        )?),
        "dividend_calendar" => to_value(engine::dividend_calendar(
            ctx.conn(),
            &parse_args::<DividendCalendarArgs>(args)?,
        )?),
        "pending_income" => to_value(engine::pending_income(
            ctx.conn(),
            &parse_args::<PendingIncomeArgs>(args)?,
        )?),
        "income_ledger" => to_value(income_ledger::build(
            ctx.conn(),
            &parse_args::<income_ledger::LedgerArgs>(args)?,
        )?),
        "income_rebuild" => {
            let a: income_ledger::RebuildArgs = parse_args(args)?;
            to_value(income_ledger::rebuild(ctx, &a)?)
        }
        "income_auto" => {
            #[derive(Deserialize)]
            struct AutoArgs {
                enabled: bool,
            }
            income_ledger::ensure_user(ctx)?;
            let a: AutoArgs = parse_args(args)?;
            income_ledger::set_auto(ctx.conn(), a.enabled)?;
            Ok(serde_json::json!({ "auto": a.enabled }))
        }
        "passive_income" => to_value(engine::passive_income(
            ctx.conn(),
            &parse_args::<PassiveIncomeArgs>(args)?,
        )?),

        "list_trades" => to_value(records::list_trades(
            ctx.conn(),
            &parse_args::<TradeFilter>(args)?,
        )?),
        // Totals of an asset's trades (summary above the trade list on the asset page).
        "trade_summary" => to_value(super::trade_summary::summary(
            ctx.conn(),
            &parse_args(args)?,
        )?),
        "get_trade" => to_value(records::get_trade(
            ctx.conn(),
            &parse_args::<IdArg>(args)?.id,
        )?),
        "bulk" => {
            let a: bulk::BulkArgs = parse_args(args)?;
            let today = crate::time::today_local();
            let plan = ctx.dry_run(|tx| bulk::apply(tx, &a, today))?;
            if plan.total_created() == 0 {
                return Ok(serde_json::json!({
                    "status": "nothing_new",
                    "message": t("nucleo-carteira-nada-novo"),
                    "report": bulk::report_value(&plan),
                }));
            }
            ctx.set_report(bulk::report_value(&plan));
            let r = ctx.write(&plan.reason(&a.source), |tx| bulk::apply(tx, &a, today))?;
            Ok(serde_json::json!({
                "status": "applied",
                "change_set_id": r.change_set_id,
                "report": bulk::report_value(&r.value),
            }))
        }
        "create_trade" => {
            // `adjustment`: the trade only aligns the quantity with what the institution shows
            // (no money moved): recorded without cash and marked, so it is not a sale to report.
            #[derive(Deserialize)]
            struct A {
                #[serde(flatten)]
                trade: TradeInput,
                #[serde(default)]
                adjustment: Option<bool>,
            }
            let A {
                trade: mut t,
                adjustment,
            } = parse_args(args)?;
            let adjustment = adjustment.unwrap_or(false);
            if adjustment {
                if t.cash_movement == Some(true) {
                    return Err(Error::invalid(i18n::t(
                        "nucleo-carteira-erro-ajuste-com-caixa",
                    )));
                }
                t.cash_movement = Some(false);
            }
            let r = ctx.write(
                &i18n::t("nucleo-carteira-motivo-operacao-registrada"),
                |tx| {
                    let id = records::create_trade(tx, &t)?;
                    if adjustment {
                        records::set_adjustment(tx, &id, true)?;
                    }
                    Ok(id)
                },
            )?;
            to_value(records::get_trade(ctx.conn(), &r.value)?)
        }
        "update_trade" => {
            let p: TradePatch = parse_args(args)?;
            let id = p.id.clone();
            ctx.write(&t("nucleo-carteira-motivo-operacao-alterada"), |tx| {
                records::update_trade(tx, &p)
            })?;
            to_value(records::get_trade(ctx.conn(), &id)?)
        }
        "delete_trade" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(
                &removed("nucleo-carteira-motivo-operacoes-removidas", 1),
                |tx| records::delete_trade(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        "delete_trades" => delete_many(
            ctx,
            args,
            "nucleo-carteira-motivo-operacoes-removidas",
            records::delete_trade,
        ),

        "statement" => to_value(statement::statement(ctx.conn(), &parse_args(args)?)?),
        "list_actions" => to_value(records::list_actions(
            ctx.conn(),
            parse_args::<AssetOpt>(args)?.asset_id.as_deref(),
        )?),
        "create_action" => {
            let a: CorporateActionInput = parse_args(args)?;
            let r = ctx.write(&t("nucleo-carteira-motivo-evento-registrado"), |tx| {
                records::create_action(tx, &a)
            })?;
            to_value(
                records::list_actions(ctx.conn(), Some(&a.asset_id))?
                    .into_iter()
                    .find(|x| x.id == r.value),
            )
        }
        "delete_action" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(
                &removed("nucleo-carteira-motivo-eventos-removidos", 1),
                |tx| records::delete_action(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        "delete_actions" => delete_many(
            ctx,
            args,
            "nucleo-carteira-motivo-eventos-removidos",
            records::delete_action,
        ),

        "list_income" => {
            let a: IncomeListArgs = parse_args(args)?;
            to_value(records::list_income(
                ctx.conn(),
                a.asset_id.as_deref(),
                a.from.as_deref(),
                a.to.as_deref(),
            )?)
        }
        "create_income" => {
            let i: IncomeInput = parse_args(args)?;
            let r = ctx.write(&t("nucleo-carteira-motivo-provento-registrado"), |tx| {
                records::create_income(tx, &i)
            })?;
            to_value(
                records::list_income(ctx.conn(), Some(&i.asset_id), None, None)?
                    .into_iter()
                    .find(|x| x.id == r.value),
            )
        }
        "create_incomes" => {
            // Vários proventos de uma vez (os sugeridos que a família confirmou): uma
            // alteração só, um "Desfazer" para todos.
            let a: IncomesArgs = parse_args(args)?;
            if a.items.is_empty() {
                return Err(Error::invalid(t("nucleo-carteira-erro-sem-proventos")));
            }
            if a.items.len() > 500 {
                return Err(Error::invalid(t("nucleo-carteira-erro-proventos-demais")));
            }
            let n = a.items.len();
            let reason = t_args(
                "nucleo-carteira-motivo-proventos-confirmados",
                &i18n::args(&[("n", (n as i64).into())]),
            );
            let r = ctx.write(&reason, |tx| {
                a.items
                    .iter()
                    .map(|i| records::create_income(tx, i))
                    .collect::<Result<Vec<_>>>()
            })?;
            Ok(serde_json::json!({"created": r.value.len(), "change_set_id": r.change_set_id}))
        }
        // Aportes programados: nada é gravado sem a pessoa confirmar cada aporte.
        "plans" => to_value(plans::list(ctx.conn(), crate::time::today_local())?),
        "plans_pending" => to_value(plans::pending(ctx.conn(), crate::time::today_local())?),
        "plan_save" => {
            let i: plans::InvestmentPlanInput = parse_args(args)?;
            let key = if i.id.is_some() {
                "nucleo-carteira-motivo-plano-alterado"
            } else {
                "nucleo-carteira-motivo-plano-criado"
            };
            let r = ctx.write(&t(key), |tx| plans::save(tx, &i))?;
            to_value(plans::get(
                ctx.conn(),
                &r.value,
                crate::time::today_local(),
            )?)
        }
        "plan_delete" => {
            let a: IdArg = parse_args(args)?;
            let r = ctx.write(&t("nucleo-carteira-motivo-plano-removido"), |tx| {
                plans::delete(tx, &a.id)
            })?;
            Ok(serde_json::json!({"change_set_id": r.change_set_id}))
        }
        "plan_confirm" => {
            let a: plans::PlanConfirmArgs = parse_args(args)?;
            let r = ctx.write(&t("nucleo-carteira-motivo-aporte-confirmado"), |tx| {
                plans::confirm(tx, &a)
            })?;
            Ok(serde_json::json!({
                "trade": records::get_trade(ctx.conn(), &r.value)?,
                "change_set_id": r.change_set_id,
            }))
        }
        "plan_skip" => {
            let a: plans::PlanSkipArgs = parse_args(args)?;
            let r = ctx.write(&t("nucleo-carteira-motivo-aporte-pulado"), |tx| {
                plans::skip(tx, &a)
            })?;
            Ok(serde_json::json!({"change_set_id": r.change_set_id}))
        }
        "fee_cost" => to_value(fee_cost::fee_cost(ctx.conn(), &parse_args(args)?)?),
        "concentration" => {
            let a: PortfolioArgs = parse_args(if args.is_null() {
                serde_json::json!({})
            } else {
                args
            })?;
            let s = engine::summary(ctx.conn(), &a)?;
            to_value(concentration::report(ctx.conn(), &s)?)
        }
        "set_concentration_limit" => {
            let a: concentration::SetConcentrationLimitArgs = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-motivo-limite"), |tx| {
                concentration::set_limit(tx, &a)
            })?;
            let s = engine::summary(ctx.conn(), &PortfolioArgs::default())?;
            to_value(concentration::report(ctx.conn(), &s)?)
        }
        "compare_fixed_income" => to_value(compare::compare(ctx.conn(), &parse_args(args)?)?),
        "simulate_fixed_income_sale" => {
            to_value(compare::simulate_sale(ctx.conn(), &parse_args(args)?)?)
        }
        "risk" => to_value(super::risk::risk(ctx.conn(), &parse_args(args)?)?),
        // Comparar com qualquer série e maiores contribuições (docs/portfolio-compare.md).
        "benchmark_series" => to_value(super::benchmark::benchmark_series(
            ctx.conn(),
            &parse_args(args)?,
        )?),
        "contributions" => to_value(super::contrib::contributions(
            ctx.conn(),
            &parse_args(args)?,
        )?),
        "come_cotas" => {
            let a: ComeCotasArgs = parse_args(args)?;
            let (rule, events, warnings) = come_cotas::for_asset(
                ctx.conn(),
                &a.asset_id,
                a.account_id.as_deref(),
                crate::time::today_local(),
            )?;
            Ok(serde_json::json!({"rule": rule, "events": events, "warnings": warnings}))
        }
        "set_come_cotas" => {
            let a: SetComeCotasArgs = parse_args(args)?;
            crate::assets::get(ctx.conn(), &a.asset_id)?;
            ctx.write(&t("nucleo-carteira-motivo-come-cotas"), |tx| {
                come_cotas::set_rule(tx, &a.asset_id, a.regime)
            })?;
            let asset = crate::assets::get(ctx.conn(), &a.asset_id)?;
            to_value(come_cotas::rule(ctx.conn(), &asset)?)
        }
        "quick_income_preview" => {
            let a: quick_income::QuickIncomeArgs = parse_args(args)?;
            to_value(quick_income::preview(
                ctx.conn(),
                &a,
                crate::time::today_local(),
            )?)
        }
        "quick_income" => {
            let a: quick_income::QuickIncomeArgs = parse_args(args)?;
            let today = crate::time::today_local();
            let plan = ctx.dry_run(|tx| quick_income::apply(tx, &a, today))?;
            if plan.created == 0 {
                return Ok(serde_json::json!({
                    "status": "nothing_new",
                    "message": t("nucleo-carteira-nenhuma-linha-pronta"),
                    "report": plan,
                }));
            }
            ctx.set_report(serde_json::to_value(&plan)?);
            let r = ctx.write(&quick_income::reason(&plan), |tx| {
                quick_income::apply(tx, &a, today)
            })?;
            Ok(serde_json::json!({
                "status": "applied",
                "change_set_id": r.change_set_id,
                "report": r.value,
            }))
        }
        // The credit of distributions recorded as received without a cash entry (the ledger's
        // `cash_pending`), in one change set.
        "income_cash" => {
            #[derive(Deserialize)]
            struct A {
                ids: Vec<Id>,
            }
            let a: A = parse_args(args)?;
            income_ledger::ensure_user(ctx)?;
            if a.ids.is_empty() {
                return Err(Error::invalid(t("nucleo-carteira-erro-sem-proventos")));
            }
            let reason = crate::i18n::t_args(
                "nucleo-carteira-razao-motivo-caixa",
                &i18n::args(&[("n", (a.ids.len() as i64).into())]),
            );
            let r = ctx.write(&reason, |tx| {
                for id in &a.ids {
                    records::add_income_cash(tx, id)?;
                }
                Ok(())
            })?;
            Ok(serde_json::json!({
                "created": a.ids.len(),
                "change_set_id": r.change_set_id,
            }))
        }
        "delete_income" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(
                &removed("nucleo-carteira-motivo-proventos-removidos", 1),
                |tx| records::delete_income(tx, &a.id),
            )?;
            Ok(Value::Null)
        }
        "delete_incomes" => delete_many(
            ctx,
            args,
            "nucleo-carteira-motivo-proventos-removidos",
            records::delete_income,
        ),

        "list_targets" => to_value(records::list_targets(ctx.conn())?),
        "set_targets" => {
            let a: SetTargetsArgs = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-motivo-alvos"), |tx| {
                records::set_targets(tx, a.scope, &a.targets)
            })?;
            to_value(records::list_targets(ctx.conn())?)
        }

        "list_watchlist" => to_value(records::list_watch(ctx.conn())?),
        "watch" => {
            let w: WatchlistInput = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-motivo-watchlist"), |tx| {
                records::upsert_watch(tx, &w)
            })?;
            to_value(records::list_watch(ctx.conn())?)
        }
        "unwatch" => {
            let a: AssetIdArg = parse_args(args)?;
            ctx.write(&t("nucleo-carteira-motivo-watchlist-removido"), |tx| {
                tx.execute(
                    "DELETE FROM watchlist_item WHERE asset_id = ?1",
                    [&a.asset_id],
                )?;
                Ok(())
            })?;
            to_value(records::list_watch(ctx.conn())?)
        }
        _ => Err(Error::UnknownCommand(format!("portfolio.{action}"))),
    }
}
