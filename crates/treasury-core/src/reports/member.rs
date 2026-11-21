//! Recorte de um membro da família (docs/members.md): o que é dele nos relatórios, no painel
//! e na saúde financeira.
//!
//! - Lançamentos: o titular da parte ou do lançamento; sem nenhum, o dono da conta quando ela
//!   tem um titular só (`engine::MEMBER_EXPR`).
//! - Contas e bens: a fração do membro (1 ÷ nº de titulares); conta conjunta de 2 entra pela
//!   metade. Contas e bens sem titular ficam de fora (são da família).
//! - Dívidas: os contratos com o membro como titular.

use std::collections::HashMap;

use rusqlite::Connection;

use crate::error::Result;
use crate::ids::Id;

#[derive(Debug, Clone)]
pub struct MemberScope {
    pub member_id: Id,
    /// Conta → fração do membro.
    pub accounts: HashMap<Id, f64>,
    /// Bem físico → fração do membro.
    pub physical: HashMap<Id, f64>,
    /// Para os filtros dos fluxos (`FlowQuery::member_ids`).
    pub ids: Vec<Id>,
}

fn shares(conn: &Connection, sql: &str, member_id: &str) -> Result<HashMap<Id, f64>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map([member_id], |r| {
            Ok((r.get::<_, Id>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .map(|(id, n)| (id, 1.0 / n as f64))
        .collect())
}

impl MemberScope {
    pub fn load(conn: &Connection, member_id: &str) -> Result<Self> {
        crate::household::get_member(conn, member_id)?;
        Ok(MemberScope {
            member_id: member_id.to_string(),
            accounts: shares(
                conn,
                "SELECT am.account_id, (SELECT count(*) FROM account_member x WHERE x.account_id = am.account_id)
                   FROM account_member am WHERE am.member_id = ?1",
                member_id,
            )?,
            physical: shares(
                conn,
                "SELECT pm.asset_id, (SELECT count(*) FROM physical_asset_member x
                                        WHERE x.asset_id = pm.asset_id)
                   FROM physical_asset_member pm WHERE pm.member_id = ?1",
                member_id,
            )?,
            ids: vec![member_id.to_string()],
        })
    }

    pub fn account_share(&self, id: &str) -> f64 {
        self.accounts.get(id).copied().unwrap_or(0.0)
    }

    pub fn physical_share(&self, id: &str) -> f64 {
        self.physical.get(id).copied().unwrap_or(0.0)
    }
}

/// `v × fração`, arredondado ao centavo.
pub(crate) fn scale(v: i64, share: f64) -> i64 {
    if share >= 1.0 {
        v
    } else {
        (v as f64 * share).round() as i64
    }
}

/// Fração da conta no recorte (1 sem recorte).
pub(crate) fn account_share(scope: Option<&MemberScope>, id: &str) -> f64 {
    scope.map_or(1.0, |s| s.account_share(id))
}
