//! Posições ligadas a uma meta (docs/design/college-goal.md §7), para qualquer tipo de meta:
//! a conta inteira (caixa + posições a mercado) ou um ativo dentro dela, numa fração (50% do
//! título para cada filho) ou, sem fração, num pote comum dividido entre as metas que o usam.
//! O valor é o de mercado de hoje, convertido para a moeda da meta. O app mostra o vencimento do
//! título ao lado da data da meta, como fato, sem sugerir nada.

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Goal;
use super::education::PoolRule;
use super::sources::{GoalValueMode, GoalValueUsed};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::reports::engine::{Fx, balances_at};
use crate::time::{fmt_date, now_utc};

/// Margem nas somas de frações (arredondamento do `REAL`).
const EPS: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HoldingInput {
    pub account_id: Id,
    /// Sem ele, a conta inteira (caixa + posições).
    #[ts(optional)]
    pub asset_id: Option<Id>,
    /// Fração (0,01–1); sem ela, pote comum dividido entre as metas.
    #[ts(optional)]
    pub share: Option<f64>,
    /// Valor de mercado ou líquido de resgate (padrão: líquido, quando a posição tem).
    #[serde(default)]
    #[ts(optional)]
    pub value_mode: Option<GoalValueMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HoldingsSetArgs {
    pub goal_id: Id,
    pub holdings: Vec<HoldingInput>,
}

/// Uma posição ligada à meta, com quanto vale para ela hoje (na moeda da meta).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HoldingView {
    pub id: Id,
    pub account_id: Id,
    pub account_name: String,
    pub asset_id: Option<Id>,
    pub asset_name: Option<String>,
    pub ticker: Option<String>,
    /// `null`: pote comum.
    pub share: Option<f64>,
    /// No pote comum, a fração que coube a esta meta.
    pub pool_share: Option<f64>,
    /// A conta ou o ativo inteiro.
    #[ts(type = "number")]
    pub full_value: i64,
    /// A parte desta meta.
    #[ts(type = "number")]
    pub value: i64,
    pub currency: String,
    /// Vencimento do título (renda fixa).
    pub maturity_date: Option<String>,
    /// Sem cotação: o ativo vale pelo custo.
    pub missing_price: bool,
    pub value_mode: GoalValueMode,
    /// O valor que entrou: de mercado, líquido de resgate ou o custo (sem cotação).
    pub value_used: GoalValueUsed,
}

#[derive(Debug, Clone)]
struct Row {
    id: Id,
    goal_id: Id,
    account_id: Id,
    asset_id: Option<Id>,
    share: Option<f64>,
    value_mode: GoalValueMode,
}

fn rows_where(conn: &Connection, filter: &str, arg: &str) -> Result<Vec<Row>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT h.id, h.goal_id, h.account_id, h.asset_id, h.share, h.value_mode FROM goal_holding h
           JOIN goal g ON g.id = h.goal_id
          WHERE {filter} ORDER BY h.created_at, h.id"
    ))?;
    let rows = stmt
        .query_map([arg], |r| {
            Ok(Row {
                id: r.get(0)?,
                goal_id: r.get(1)?,
                account_id: r.get(2)?,
                asset_id: r.get(3)?,
                share: r.get(4)?,
                value_mode: GoalValueMode::parse(&r.get::<_, String>(5)?),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

fn goal_rows(conn: &Connection, goal_id: &str) -> Result<Vec<Row>> {
    rows_where(conn, "h.goal_id = ?1", goal_id)
}

/// A meta tem posições ligadas?
pub fn has_any(conn: &Connection, goal_id: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM goal_holding WHERE goal_id = ?1)",
        [goal_id],
        |r| r.get(0),
    )?)
}

/// Valor de uma posição hoje, na moeda base.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PositionValue {
    /// A mercado (sem cotação: o custo).
    pub market: i64,
    /// Líquido de resgate (renda fixa, já com IR e IOF), quando há.
    pub net: Option<i64>,
    pub missing: bool,
}

impl PositionValue {
    /// O valor no modo pedido e qual valor entrou.
    pub(crate) fn pick(&self, mode: GoalValueMode) -> (i64, GoalValueUsed) {
        match (mode, self.net) {
            _ if self.missing => (self.market, GoalValueUsed::Cost),
            (GoalValueMode::NetRedemption, Some(n)) => (n, GoalValueUsed::NetRedemption),
            _ => (self.market, GoalValueUsed::Market),
        }
    }
}

/// Valores de hoje das contas e posições, na moeda base (a carteira é calculada uma vez).
pub(crate) struct Valuer {
    pub(crate) base: String,
    /// (conta, ativo) → valor na moeda base.
    pub(crate) positions: HashMap<(Id, Id), PositionValue>,
    /// Caixa da conta: (saldo, moeda).
    cash: HashMap<Id, (i64, String)>,
    pub(crate) today: String,
}

impl Valuer {
    pub(crate) fn new(conn: &Connection, today: NaiveDate) -> Result<Self> {
        let today_s = fmt_date(today);
        let summary = crate::portfolio::engine::summary(
            conn,
            &crate::portfolio::PortfolioArgs {
                as_of: Some(today_s.clone()),
                account_ids: None,
                deflator: None,
                by_account: Some(true),
                include_closed: None,
            },
        )?;
        let mut positions = HashMap::new();
        for p in &summary.positions {
            let Some(acc) = &p.account_id else { continue };
            let (market, missing) = match p.market_value_base {
                Some(v) => (v, false),
                None => (p.cost_base, true),
            };
            positions.insert(
                (acc.clone(), p.asset.id.clone()),
                PositionValue {
                    market,
                    net: p.net_redemption_base,
                    missing,
                },
            );
        }
        let cash = balances_at(conn, &today_s)?
            .into_iter()
            .map(|a| (a.id, (a.balance, a.currency)))
            .collect();
        Ok(Self {
            base: summary.base_currency,
            positions,
            cash,
            today: today_s,
        })
    }

    /// O valor da conta inteira (ou do ativo nela), na moeda `fx.target`, e qual valor entrou;
    /// `skip_cash` deixa o caixa de fora (a conta já entra na meta pelo saldo).
    fn full(
        &self,
        fx: &mut Fx<'_>,
        account: &str,
        asset: Option<&str>,
        skip_cash: bool,
        mode: GoalValueMode,
    ) -> Result<(i64, bool, GoalValueUsed)> {
        let mut base_total = 0;
        let mut missing = false;
        let mut used = GoalValueUsed::Market;
        for ((acc, ast), p) in &self.positions {
            if acc == account && asset.is_none_or(|a| a == ast) {
                let (v, u) = p.pick(mode);
                base_total += v;
                missing |= p.missing;
                if u != GoalValueUsed::Market {
                    used = u;
                }
            }
        }
        let mut total = fx
            .convert(base_total, &self.base, &self.today)?
            .map_or(0, |c| c.amount);
        if asset.is_none()
            && !skip_cash
            && let Some((bal, cur)) = self.cash.get(account)
            && let Some(c) = fx.convert(*bal, cur, &self.today)?
        {
            total += c.amount;
        }
        Ok((total, missing, used))
    }
}

/// Alvo e data de uma meta, para dividir o pote comum.
fn goal_order(conn: &Connection, goal_id: &str) -> Result<(Option<String>, i64, String)> {
    Ok(conn.query_row(
        "SELECT target_date, target_amount, created_at FROM goal WHERE id = ?1",
        [goal_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?)
}

/// Quanto do pote comum `value` (já sem as frações explícitas) cabe a `goal_id`: pela regra do
/// plano da meta que começa primeiro (quem começa primeiro recebe primeiro, ou proporcional ao
/// alvo).
fn pool_part(conn: &Connection, goal_id: &str, members: &[Id], value: i64) -> Result<(i64, f64)> {
    if value <= 0 || members.is_empty() {
        return Ok((0, 0.0));
    }
    let mut goals = vec![];
    for g in members {
        let (date, target, created) = goal_order(conn, g)?;
        goals.push((date, created, g.clone(), target.max(0)));
    }
    // Sem data vai por último.
    goals.sort_by(|a, b| (a.0.is_none(), &a.0, &a.1, &a.2).cmp(&(b.0.is_none(), &b.0, &b.1, &b.2)));
    let rule = match super::education::load(conn, &goals[0].2)? {
        Some(p) => p.pool_rule,
        None => PoolRule::EarliestFirst,
    };
    let part = match rule {
        PoolRule::EarliestFirst => {
            let mut left = value;
            let mut mine = 0;
            for (i, (_, _, g, target)) in goals.iter().enumerate() {
                // O último leva o que sobrar.
                let take = if i + 1 == goals.len() {
                    left
                } else {
                    left.min(*target)
                };
                if g == goal_id {
                    mine = take;
                }
                left -= take;
            }
            mine
        }
        PoolRule::Proportional => {
            let total: i64 = goals.iter().map(|g| g.3).sum();
            let target = goals.iter().find(|g| g.2 == goal_id).map_or(0, |g| g.3);
            if total > 0 {
                (value as f64 * target as f64 / total as f64).round() as i64
            } else {
                value / goals.len() as i64
            }
        }
    };
    Ok((part, part as f64 / value as f64))
}

/// As posições da meta com o valor de hoje.
pub fn views(conn: &Connection, goal: &Goal, today: NaiveDate) -> Result<Vec<HoldingView>> {
    let rows = goal_rows(conn, &goal.id)?;
    if rows.is_empty() {
        return Ok(vec![]);
    }
    let valuer = Valuer::new(conn, today)?;
    let mut fx = Fx::new(conn, &goal.currency);
    let mut out = vec![];
    for r in rows {
        let skip_cash = goal.account_ids.contains(&r.account_id);
        let (full, missing, used) = valuer.full(
            &mut fx,
            &r.account_id,
            r.asset_id.as_deref(),
            skip_cash,
            r.value_mode,
        )?;
        let (value, pool_share) = match r.share {
            Some(s) => ((full as f64 * s).round() as i64, None),
            None => {
                // O pote: o que sobra das frações explícitas, dividido entre as metas sem fração.
                let same = same_holding(conn, &r.account_id, r.asset_id.as_deref())?;
                let explicit: f64 = same.iter().filter_map(|x| x.share).sum();
                let members: Vec<Id> = same
                    .iter()
                    .filter(|x| x.share.is_none())
                    .map(|x| x.goal_id.clone())
                    .collect();
                let pool = (full as f64 * (1.0 - explicit).max(0.0)).round() as i64;
                let (part, frac) = pool_part(conn, &goal.id, &members, pool)?;
                (part, Some(frac * (1.0 - explicit).max(0.0)))
            }
        };
        let account_name: String = conn
            .query_row(
                "SELECT name FROM account WHERE id = ?1",
                [&r.account_id],
                |x| x.get(0),
            )
            .unwrap_or_default();
        let (asset_name, ticker, maturity) = match &r.asset_id {
            Some(a) => {
                let (n, tk): (String, Option<String>) = conn
                    .query_row("SELECT name, ticker FROM asset WHERE id = ?1", [a], |x| {
                        Ok((x.get(0)?, x.get(1)?))
                    })
                    .unwrap_or_default();
                let m: Option<String> = conn
                    .query_row(
                        "SELECT maturity_date FROM fixed_income_terms WHERE asset_id = ?1",
                        [a],
                        |x| x.get(0),
                    )
                    .optional()?
                    .flatten();
                (Some(n), tk, m)
            }
            None => (None, None, None),
        };
        out.push(HoldingView {
            id: r.id,
            account_id: r.account_id,
            account_name,
            asset_id: r.asset_id,
            asset_name,
            ticker,
            share: r.share,
            pool_share,
            full_value: full,
            value,
            currency: goal.currency.clone(),
            maturity_date: maturity,
            missing_price: missing,
            value_mode: r.value_mode,
            value_used: used,
        });
    }
    Ok(out)
}

/// Soma do que as posições ligadas valem para a meta hoje.
pub fn value_for(conn: &Connection, goal: &Goal, today: NaiveDate) -> Result<Option<i64>> {
    if !has_any(conn, &goal.id)? {
        return Ok(None);
    }
    Ok(Some(
        views(conn, goal, today)?.iter().map(|h| h.value).sum(),
    ))
}

/// Uma meta na classificação "Objetivo" da carteira.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ObjectiveSlice {
    pub goal_id: Id,
    pub goal_name: String,
    pub kind: super::GoalKind,
    /// Quanto as posições ligadas valem para a meta hoje (moeda base).
    #[ts(type = "number")]
    pub value: i64,
    /// Fração da carteira (0–1).
    pub share: f64,
    /// Alvo da meta (moeda base).
    #[ts(type = "number")]
    pub target_amount: i64,
    pub positions: u32,
}

/// A carteira por objetivo: cada meta com posições ligadas, pelos pesos (frações e pote
/// comum), e o que não está ligado a nenhuma.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ObjectiveAllocation {
    pub currency: String,
    /// Posições da carteira a mercado (moeda base).
    #[ts(type = "number")]
    pub total: i64,
    pub slices: Vec<ObjectiveSlice>,
    /// O que não está ligado a nenhuma meta (nunca negativo: o caixa das contas ligadas pode
    /// passar do total das posições).
    #[ts(type = "number")]
    pub unassigned: i64,
    pub unassigned_share: f64,
}

/// Classificação "Objetivo" da carteira (docs/scheduled-contributions.md §2).
pub fn objectives(conn: &Connection, today: NaiveDate) -> Result<ObjectiveAllocation> {
    let valuer = Valuer::new(conn, today)?;
    let base = valuer.base.clone();
    let total: i64 = valuer.positions.values().map(|p| p.market).sum();
    let mut stmt = conn.prepare_cached(
        "SELECT DISTINCT g.id FROM goal g JOIN goal_holding h ON h.goal_id = g.id
          WHERE g.archived = 0 ORDER BY g.created_at, g.id",
    )?;
    let ids: Vec<Id> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut fx = Fx::new(conn, &base);
    let mut slices = vec![];
    for id in ids {
        let goal = super::load(conn, &id)?;
        let views = views(conn, &goal, today)?;
        let value: i64 = views.iter().map(|h| h.value).sum();
        let value = fx
            .convert(value, &goal.currency, &valuer.today)?
            .map_or(0, |c| c.amount);
        let target = fx
            .convert(goal.target_amount, &goal.currency, &valuer.today)?
            .map_or(0, |c| c.amount);
        slices.push(ObjectiveSlice {
            goal_id: goal.id.clone(),
            goal_name: goal.name.clone(),
            kind: goal.kind,
            value,
            share: 0.0,
            target_amount: target,
            positions: views.len() as u32,
        });
    }
    let assigned: i64 = slices.iter().map(|s| s.value).sum();
    let unassigned = (total - assigned).max(0);
    let whole = (assigned + unassigned).max(1) as f64;
    for s in &mut slices {
        s.share = s.value as f64 / whole;
    }
    slices.sort_by(|a, b| b.value.cmp(&a.value).then(a.goal_name.cmp(&b.goal_name)));
    Ok(ObjectiveAllocation {
        currency: base,
        total,
        slices,
        unassigned,
        unassigned_share: unassigned as f64 / whole,
    })
}

/// A mesma conta (e ativo) em todas as metas não arquivadas.
fn same_holding(conn: &Connection, account: &str, asset: Option<&str>) -> Result<Vec<Row>> {
    Ok(
        rows_where(conn, "g.archived = 0 AND h.account_id = ?1", account)?
            .into_iter()
            .filter(|r| r.asset_id.as_deref() == asset)
            .collect(),
    )
}

/// Troca as posições ligadas à meta.
pub fn set(tx: &Transaction<'_>, a: &HoldingsSetArgs) -> Result<()> {
    let goal = super::load(tx, &a.goal_id)?;
    if goal.archived {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-arquivada")));
    }
    let mut seen = std::collections::HashSet::new();
    for h in &a.holdings {
        if !seen.insert((h.account_id.clone(), h.asset_id.clone())) {
            return Err(Error::invalid(t("nucleo-educacao-erro-posicao-repetida")));
        }
        let name: Option<String> = tx
            .query_row(
                "SELECT name FROM account WHERE id = ?1",
                [&h.account_id],
                |r| r.get(0),
            )
            .optional()?;
        let name = name.ok_or_else(|| {
            Error::not_found(t_args(
                "nucleo-educacao-conta",
                &args(&[("id", h.account_id.clone().into())]),
            ))
        })?;
        if let Some(asset) = &h.asset_id {
            let ok: bool = tx.query_row(
                "SELECT EXISTS (SELECT 1 FROM asset WHERE id = ?1)",
                [asset],
                |r| r.get(0),
            )?;
            if !ok {
                return Err(Error::not_found(t_args(
                    "nucleo-educacao-ativo",
                    &args(&[("id", asset.clone().into())]),
                )));
            }
        }
        if let Some(s) = h.share {
            if !(0.01 - EPS..=1.0 + EPS).contains(&s) {
                return Err(Error::invalid(t("nucleo-educacao-erro-fracao")));
            }
            // As frações do mesmo ativo (ou conta) entre as metas não passam de 100%.
            let others: f64 = same_holding(tx, &h.account_id, h.asset_id.as_deref())?
                .iter()
                .filter(|r| r.goal_id != a.goal_id)
                .filter_map(|r| r.share)
                .sum();
            if others + s > 1.0 + EPS {
                return Err(Error::invalid(t_args(
                    "nucleo-educacao-erro-fracao-soma",
                    &args(&[
                        ("nome", name.into()),
                        (
                            "livre",
                            crate::ai::alerts::pct(
                                crate::i18n::current(),
                                (1.0 - others).max(0.0),
                                0,
                            )
                            .into(),
                        ),
                    ]),
                )));
            }
        }
    }
    let old = goal_rows(tx, &a.goal_id)?;
    let now = now_utc();
    for r in &old {
        if !a
            .holdings
            .iter()
            .any(|h| h.account_id == r.account_id && h.asset_id == r.asset_id)
        {
            tx.execute("DELETE FROM goal_holding WHERE id = ?1", [&r.id])?;
        }
    }
    for h in &a.holdings {
        let mode = h.value_mode.unwrap_or_default();
        match old
            .iter()
            .find(|r| r.account_id == h.account_id && r.asset_id == h.asset_id)
        {
            Some(r) if r.share == h.share && r.value_mode == mode => {}
            Some(r) => {
                tx.execute(
                    "UPDATE goal_holding SET share = ?2, value_mode = ?3, updated_at = ?4 WHERE id = ?1",
                    params![r.id, h.share, mode.as_str(), now],
                )?;
            }
            None => {
                tx.execute(
                    "INSERT INTO goal_holding (id, goal_id, account_id, asset_id, share, value_mode,
                                               created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                    params![
                        new_id(),
                        a.goal_id,
                        h.account_id,
                        h.asset_id,
                        h.share,
                        mode.as_str(),
                        now
                    ],
                )?;
            }
        }
    }
    Ok(())
}
