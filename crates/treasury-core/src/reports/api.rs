use serde_json::Value;

use super::{
    CompareArgs, DashboardArgs, ExplainArgs, MonthFlowArgs, MonthlyEvolutionArgs, NetWorthArgs,
    NetWorthHistoryArgs, PeriodArgs, SANKEY_DEFAULT_DEPTH, SankeyArgs, SankeyItemsArgs,
    SpendingArgs, UpcomingArgs, engine,
};
use crate::api::{Ctx, parse_args, to_value};
use crate::categories::CategoryKind;
use crate::error::{Error, Result};
use crate::time::{fmt_date, parse_date};

pub const READ_ONLY: &[&str] = &[
    "dashboard",
    "net_worth",
    "net_worth_history",
    "month_flow",
    "spending",
    "sankey",
    "sankey_items",
    "calendar",
    "monthly_evolution",
    "compare",
    "by_member",
    "upcoming",
    "explain",
    "insights",
    "inflation",
    "spend_groups",
];

/// Recorte de um membro, se pedido.
fn scope(
    conn: &rusqlite::Connection,
    member_id: Option<&str>,
) -> Result<Option<super::member::MemberScope>> {
    member_id
        .filter(|m| !m.is_empty())
        .map(|m| super::member::MemberScope::load(conn, m))
        .transpose()
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    let conn = ctx.conn();
    match action {
        "dashboard" => {
            let a: DashboardArgs = parse_args(args)?;
            let scope = scope(conn, a.member_id.as_deref())?;
            to_value(super::dashboard_scoped(
                conn,
                a.month.as_deref(),
                a.basis,
                scope.as_ref(),
            )?)
        }
        "net_worth" => {
            let a: NetWorthArgs = parse_args(args)?;
            let as_of = a.as_of.unwrap_or_else(|| fmt_date(engine::today()));
            to_value(super::net_worth(conn, &as_of)?)
        }
        "net_worth_history" => {
            let a: NetWorthHistoryArgs = parse_args(args)?;
            let end = a.end_month.unwrap_or_else(engine::current_month);
            let scope = scope(conn, a.member_id.as_deref())?;
            to_value(super::net_worth_history_scoped(
                conn,
                a.months.unwrap_or(12),
                &end,
                scope.as_ref(),
            )?)
        }
        "month_flow" => {
            let a: MonthFlowArgs = parse_args(args)?;
            let basis = engine::resolve_basis(conn, a.basis)?;
            let scope = scope(conn, a.member_id.as_deref())?;
            to_value(super::month_flow_scoped(
                conn,
                &a.from_month,
                &a.to_month,
                basis,
                scope.as_ref(),
            )?)
        }
        "spending" => to_value(super::spending(conn, &parse_args::<SpendingArgs>(args)?)?),
        "sankey" => {
            let a: SankeyArgs = parse_args(args)?;
            let basis = engine::resolve_flow_basis(conn, a.basis)?;
            to_value(super::sankey_with(
                conn,
                &a.from,
                &a.to,
                basis,
                a.depth.unwrap_or(SANKEY_DEFAULT_DEPTH),
                a.compare.unwrap_or(true),
            )?)
        }
        "sankey_items" => {
            let a: SankeyItemsArgs = parse_args(args)?;
            let basis = engine::resolve_flow_basis(conn, a.basis)?;
            to_value(super::sankey_items(
                conn,
                &a.from,
                &a.to,
                basis,
                a.depth.unwrap_or(SANKEY_DEFAULT_DEPTH),
                &a.node_ids,
            )?)
        }
        "calendar" => {
            let a: PeriodArgs = parse_args(args)?;
            let basis = engine::resolve_basis(conn, a.basis)?;
            to_value(super::calendar(conn, &a.from, &a.to, basis)?)
        }
        "monthly_evolution" => {
            let a: MonthlyEvolutionArgs = parse_args(args)?;
            let basis = engine::resolve_basis(conn, a.basis)?;
            let end = a.end_month.unwrap_or_else(engine::current_month);
            to_value(super::monthly_evolution(
                conn,
                a.months.unwrap_or(12),
                &end,
                a.top_n.unwrap_or(6),
                basis,
            )?)
        }
        "compare" => {
            let a: CompareArgs = parse_args(args)?;
            let basis = engine::resolve_basis(conn, a.basis)?;
            to_value(super::compare(
                conn,
                &a.a,
                &a.b,
                basis,
                a.kind.unwrap_or(CategoryKind::Expense),
            )?)
        }
        "by_member" => {
            let a: PeriodArgs = parse_args(args)?;
            let basis = engine::resolve_basis(conn, a.basis)?;
            to_value(super::by_member(conn, &a.from, &a.to, basis)?)
        }
        "upcoming" => {
            let a: UpcomingArgs = parse_args(args)?;
            let from = match a.from.as_deref() {
                Some(d) => parse_date(d)?,
                None => engine::today(),
            };
            to_value(super::upcoming(conn, from, a.days.unwrap_or(30))?)
        }
        // O que o Treasury percebe sem IA (crate::insights).
        "insights" => {
            let mut v = to_value(crate::insights::compute(
                ctx.conn(),
                crate::time::today_local(),
            )?)?;
            // Avisos nos termos do país da família (docs/countries.md).
            crate::country::localize_value(ctx.conn(), &mut v);
            Ok(v)
        }
        "explain" => to_value(super::explain(conn, &parse_args::<ExplainArgs>(args)?)?),
        // Para onde vai a renda: despesas pelos grupos de gasto das categorias, mês a mês.
        "spend_groups" => to_value(super::spend_groups::spend_groups(
            conn,
            &parse_args::<super::spend_groups::SpendGroupsArgs>(args)?,
        )?),
        // Figura 07 · Inflação: gasto nominal × corrigido pelo índice do país.
        "inflation" => {
            let a: super::inflation::InflationArgs = parse_args(args)?;
            let basis = engine::resolve_basis(conn, a.basis)?;
            to_value(super::inflation::inflation(conn, &a, basis)?)
        }
        // Relatório do mês em PDF, gravado onde a pessoa escolheu (como as exportações).
        "pdf" => {
            let a: super::pdf_report::PdfArgs = parse_args(args)?;
            to_value(super::pdf_report::write(ctx.conn(), &a, ctx.paths)?)
        }
        _ => Err(Error::UnknownCommand(format!("reports.{action}"))),
    }
}
