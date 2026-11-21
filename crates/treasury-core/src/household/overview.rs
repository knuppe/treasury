//! Finanças por membro da família (docs/members.md): o mês de cada um (o que entrou, saiu e
//! sobrou), a taxa de poupança de 12 meses, o patrimônio e o nível do perfil dele, mais o que
//! não é de ninguém em particular (contas conjuntas sem titular no lançamento, por exemplo).

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::reports::engine::{current_month, normalize_month, resolve_basis, shift_month};
use crate::reports::member::MemberScope;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MembersOverviewArgs {
    /// Padrão: mês corrente.
    #[ts(optional)]
    pub month: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberMonth {
    #[ts(type = "number")]
    pub income: i64,
    #[ts(type = "number")]
    pub expense: i64,
    #[ts(type = "number")]
    pub net: i64,
    /// (receitas − despesas) ÷ receitas nos últimos 12 meses; `null` sem receitas.
    pub savings_rate_12m: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberOverview {
    pub member: super::Member,
    pub level: crate::profile::Level,
    pub tone: crate::profile::Tone,
    /// O membro já fez o próprio questionário (senão vale o perfil da família).
    pub has_own_profile: bool,
    pub month: MemberMonth,
    /// Patrimônio dele hoje (fração nas contas e bens, menos as dívidas dele).
    #[ts(type = "number")]
    pub net_worth: i64,
    /// Contas em que é titular.
    pub accounts: u32,
    /// Contratos de dívida em que é titular.
    pub loans: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MembersOverview {
    pub month: String,
    pub currency: String,
    pub members: Vec<MemberOverview>,
    /// O que não é de ninguém em particular (sem titular nem conta de um titular só).
    pub shared: MemberMonth,
}

fn month_of(flows: &[crate::reports::MonthFlow]) -> MemberMonth {
    let last = flows.last();
    let (i, e) = flows
        .iter()
        .fold((0i64, 0i64), |(i, e), f| (i + f.income, e + f.expense));
    MemberMonth {
        income: last.map_or(0, |f| f.income),
        expense: last.map_or(0, |f| f.expense),
        net: last.map_or(0, |f| f.income - f.expense),
        savings_rate_12m: (i > 0).then(|| ((i - e) as f64 / i as f64 * 1000.0).round() / 1000.0),
    }
}

pub fn overview(conn: &Connection, a: &MembersOverviewArgs) -> Result<MembersOverview> {
    let month = match a.month.as_deref() {
        Some(m) => normalize_month(m)?,
        None => current_month(),
    };
    let basis = resolve_basis(conn, None)?;
    let from = shift_month(&month, -11)?;
    let today = crate::time::fmt_date(crate::time::today_local());
    let (_, last_day) = crate::reports::engine::month_dates(&month)?;
    let as_of = last_day.min(today);
    let family = month_of(&crate::reports::month_flow(conn, &from, &month, basis)?);
    let mut shared = family.clone();
    let mut members = vec![];
    for m in super::list_members(conn, false)? {
        let scope = MemberScope::load(conn, &m.id)?;
        let flows = crate::reports::month_flow_scoped(conn, &from, &month, basis, Some(&scope))?;
        let mm = month_of(&flows);
        shared.income -= mm.income;
        shared.expense -= mm.expense;
        shared.net -= mm.net;
        let nw = crate::reports::net_worth_scoped(conn, &as_of, Some(&scope))?;
        let profile = crate::profile::get_for(conn, Some(&m.id))?;
        let loans: u32 = conn.query_row(
            "SELECT count(*) FROM loan WHERE archived = 0 AND member_id = ?1",
            [&m.id],
            |r| r.get(0),
        )?;
        members.push(MemberOverview {
            level: profile.level,
            tone: profile.tone,
            has_own_profile: profile.source != crate::profile::Source::Familia,
            month: mm,
            net_worth: nw.net,
            accounts: scope.accounts.len() as u32,
            loans,
            member: m,
        });
    }
    shared.savings_rate_12m = None;
    Ok(MembersOverview {
        month,
        currency: crate::household::base_currency(conn)?,
        members,
        shared,
    })
}
