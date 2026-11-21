//! Fontes de valor de uma meta (docs/goal-sources.md): o valor guardado fora das contas, as
//! contas vinculadas (pelo saldo, nunca negativo: conta no vermelho é dívida, não reserva), as
//! posições ligadas (a conta inteira ou um ativo, com fração ou pote comum), as etiquetas de
//! ativos (todas as posições com a etiqueta, também as que a ganharem depois) e os lançamentos
//! ligados. [`current_value`] é a conta única que a tela, o MCP e a saúde financeira usam, com o
//! detalhamento por fonte, o valor que entrou (saldo, mercado ou líquido de resgate) e as
//! fontes desatualizadas.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Goal;
use super::holdings::{self, HoldingInput, HoldingsSetArgs, Valuer};
use crate::accounts::freshness::BalanceStatus;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::reports::engine::{Fx, balances_at};
use crate::time::{fmt_date, now_utc};

/// Valor de uma posição: o de mercado ou o líquido de resgate (renda fixa, já com IR e IOF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalValueMode {
    Market,
    #[default]
    NetRedemption,
}

impl GoalValueMode {
    pub fn as_str(self) -> &'static str {
        match self {
            GoalValueMode::Market => "market",
            GoalValueMode::NetRedemption => "net_redemption",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "market" => GoalValueMode::Market,
            _ => GoalValueMode::NetRedemption,
        }
    }
}

/// O valor que entrou na meta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalValueUsed {
    /// Saldo da conta (nunca negativo).
    Balance,
    Market,
    NetRedemption,
    /// Sem cotação: o custo.
    Cost,
    /// Valor informado (guardado fora das contas, lançamentos ligados).
    Amount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalSourceKind {
    /// Valor guardado fora das contas (`initial_amount`).
    Initial,
    /// A carteira inteira, na meta ligada ao plano de aposentadoria sem contas.
    Plan,
    Account,
    /// A conta inteira ligada como posição (caixa + ativos a mercado).
    AccountPositions,
    Asset,
    AssetTag,
    /// Lançamentos ligados à meta.
    Deposits,
}

/// Quanto uma fonte contribui para a meta hoje (na moeda da meta).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalSourceValue {
    pub kind: GoalSourceKind,
    /// A conta, o ativo ou a etiqueta.
    pub ref_id: Option<Id>,
    /// A conta da posição (`asset`).
    pub account_id: Option<Id>,
    pub name: String,
    #[ts(type = "number")]
    pub value: i64,
    pub value_used: GoalValueUsed,
    /// Fração da posição (sem ela, o todo ou o pote comum).
    pub share: Option<f64>,
    /// Saldo negativo da conta, que ficou de fora (é dívida, não reserva).
    #[ts(type = "number | null")]
    pub ignored_negative: Option<i64>,
    /// Valor que pode estar desatualizado: conta `possibly_incomplete` ou posição sem cotação.
    pub stale: bool,
}

/// Uma fonte ligada à meta (para desligar: `kind`, `ref_id` e, num ativo, `account_id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalSourceRef {
    pub kind: GoalSourceKind,
    pub ref_id: Id,
    pub account_id: Option<Id>,
    pub value_mode: Option<GoalValueMode>,
    pub share: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GoalValue {
    pub total: i64,
    pub breakdown: Vec<GoalSourceValue>,
}

impl GoalValue {
    pub fn stale_sources(&self) -> Vec<GoalSourceValue> {
        self.breakdown.iter().filter(|s| s.stale).cloned().collect()
    }
    /// Quanto as posições ligadas e as etiquetas valem (`None` sem nenhuma).
    pub fn holdings_value(&self) -> Option<i64> {
        let mut any = false;
        let mut sum = 0;
        for s in &self.breakdown {
            if matches!(
                s.kind,
                GoalSourceKind::Asset | GoalSourceKind::AccountPositions | GoalSourceKind::AssetTag
            ) {
                any = true;
                sum += s.value;
            }
        }
        any.then_some(sum)
    }
    pub fn of(&self, kind: GoalSourceKind) -> Option<i64> {
        let v: Vec<i64> = self
            .breakdown
            .iter()
            .filter(|s| s.kind == kind)
            .map(|s| s.value)
            .collect();
        (!v.is_empty()).then(|| v.iter().sum())
    }
}

struct TagRow {
    label_id: Id,
    value_mode: GoalValueMode,
}

fn tag_rows(conn: &Connection, goal_id: &str) -> Result<Vec<TagRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT ref_id, value_mode FROM goal_source
          WHERE goal_id = ?1 AND kind = 'asset_tag' ORDER BY created_at, id",
    )?;
    let rows = stmt
        .query_map([goal_id], |r| {
            Ok(TagRow {
                label_id: r.get(0)?,
                value_mode: GoalValueMode::parse(&r.get::<_, String>(1)?),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

fn label_name(conn: &Connection, id: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT name FROM asset_label WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?)
}

fn label_assets(conn: &Connection, id: &str) -> Result<HashSet<Id>> {
    let mut stmt =
        conn.prepare_cached("SELECT asset_id FROM asset_label_link WHERE label_id = ?1")?;
    let v = stmt
        .query_map([id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// As fontes ligadas à meta (contas, posições e etiquetas).
pub fn refs(conn: &Connection, goal: &Goal) -> Result<Vec<GoalSourceRef>> {
    let mut out: Vec<GoalSourceRef> = goal
        .account_ids
        .iter()
        .map(|id| GoalSourceRef {
            kind: GoalSourceKind::Account,
            ref_id: id.clone(),
            account_id: None,
            value_mode: None,
            share: None,
        })
        .collect();
    let mut stmt = conn.prepare_cached(
        "SELECT account_id, asset_id, share, value_mode FROM goal_holding
          WHERE goal_id = ?1 ORDER BY created_at, id",
    )?;
    let rows: Vec<(Id, Option<Id>, Option<f64>, String)> = stmt
        .query_map([&goal.id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    for (account, asset, share, mode) in rows {
        let mode = Some(GoalValueMode::parse(&mode));
        out.push(match asset {
            Some(a) => GoalSourceRef {
                kind: GoalSourceKind::Asset,
                ref_id: a,
                account_id: Some(account),
                value_mode: mode,
                share,
            },
            None => GoalSourceRef {
                kind: GoalSourceKind::AccountPositions,
                ref_id: account,
                account_id: None,
                value_mode: mode,
                share,
            },
        });
    }
    for r in tag_rows(conn, &goal.id)? {
        out.push(GoalSourceRef {
            kind: GoalSourceKind::AssetTag,
            ref_id: r.label_id,
            account_id: None,
            value_mode: Some(r.value_mode),
            share: None,
        });
    }
    Ok(out)
}

/// O valor da meta hoje, fonte por fonte, na moeda da meta.
pub(crate) fn current_value(
    conn: &Connection,
    goal: &Goal,
    today: NaiveDate,
    fx: &mut Fx<'_>,
) -> Result<GoalValue> {
    let today_s = fmt_date(today);
    let mut out = GoalValue::default();
    let push = |out: &mut GoalValue, s: GoalSourceValue| {
        out.total += s.value;
        out.breakdown.push(s);
    };
    if goal.initial_amount != 0 {
        push(
            &mut out,
            GoalSourceValue {
                kind: GoalSourceKind::Initial,
                ref_id: None,
                account_id: None,
                name: t("nucleo-orcamento-meta-fonte-inicial"),
                value: goal.initial_amount,
                value_used: GoalValueUsed::Amount,
                share: None,
                ignored_negative: None,
                stale: false,
            },
        );
    }
    // Meta ligada ao plano de aposentadoria, sem contas: a carteira inteira.
    let plan_linked = goal.kind == super::GoalKind::Retirement
        && super::retirement::linked_goal(conn)?.as_deref() == Some(goal.id.as_str());
    if plan_linked
        && goal.account_ids.is_empty()
        && let Some(c) = fx.convert(
            super::retirement::portfolio_value(conn)?,
            &crate::household::base_currency(conn)?,
            &today_s,
        )?
    {
        push(
            &mut out,
            GoalSourceValue {
                kind: GoalSourceKind::Plan,
                ref_id: None,
                account_id: None,
                name: t("nucleo-orcamento-meta-fonte-carteira"),
                value: c.amount,
                value_used: GoalValueUsed::Market,
                share: None,
                ignored_negative: None,
                stale: false,
            },
        );
    }
    // Contas: o saldo, nunca negativo (conta no vermelho é dívida, não reserva).
    if !goal.account_ids.is_empty() {
        let status: HashMap<Id, Option<BalanceStatus>> = crate::accounts::list(conn, true, today)?
            .into_iter()
            .map(|a| (a.account.id, a.balance_status))
            .collect();
        for a in balances_at(conn, &today_s)?
            .into_iter()
            .filter(|a| goal.account_ids.contains(&a.id))
        {
            let Some(c) = fx.convert(a.balance, &a.currency, &today_s)? else {
                continue;
            };
            push(
                &mut out,
                GoalSourceValue {
                    kind: GoalSourceKind::Account,
                    ref_id: Some(a.id.clone()),
                    account_id: None,
                    name: a.name.clone(),
                    value: c.amount.max(0),
                    value_used: GoalValueUsed::Balance,
                    share: None,
                    ignored_negative: (c.amount < 0).then_some(c.amount),
                    stale: status.get(&a.id).copied().flatten()
                        == Some(BalanceStatus::PossiblyIncomplete),
                },
            );
        }
    }
    // Lançamentos ligados (a saída na conta de origem é o aporte na meta).
    let deposits = super::linked_deposits(conn, goal, &today_s)?;
    if !deposits.is_empty() {
        let mut sum = 0;
        for (date, amount, currency) in &deposits {
            if let Some(c) = fx.convert(-amount, currency, date)? {
                sum += c.amount;
            }
        }
        push(
            &mut out,
            GoalSourceValue {
                kind: GoalSourceKind::Deposits,
                ref_id: None,
                account_id: None,
                name: t("nucleo-orcamento-meta-fonte-lancamentos"),
                value: sum,
                value_used: GoalValueUsed::Amount,
                share: None,
                ignored_negative: None,
                stale: false,
            },
        );
    }
    // Posições ligadas (contas inteiras ou ativos, com fração ou pote comum).
    let views = holdings::views(conn, goal, today)?;
    let mut held: HashSet<(Id, Option<Id>)> = HashSet::new();
    for h in views {
        held.insert((h.account_id.clone(), h.asset_id.clone()));
        let (kind, ref_id, account_id, name) = match &h.asset_id {
            Some(a) => (
                GoalSourceKind::Asset,
                a.clone(),
                Some(h.account_id.clone()),
                match (&h.ticker, &h.asset_name) {
                    (Some(tk), _) => format!("{tk} · {}", h.account_name),
                    (None, Some(n)) => format!("{n} · {}", h.account_name),
                    _ => h.account_name.clone(),
                },
            ),
            None => (
                GoalSourceKind::AccountPositions,
                h.account_id.clone(),
                None,
                h.account_name.clone(),
            ),
        };
        push(
            &mut out,
            GoalSourceValue {
                kind,
                ref_id: Some(ref_id),
                account_id,
                name,
                value: h.value,
                value_used: h.value_used,
                share: h.share,
                ignored_negative: None,
                stale: h.missing_price,
            },
        );
    }
    // Etiquetas: todas as posições com a etiqueta, menos as já ligadas por posição.
    let tags = tag_rows(conn, &goal.id)?;
    if !tags.is_empty() {
        let valuer = Valuer::new(conn, today)?;
        for tag in tags {
            let Some(name) = label_name(conn, &tag.label_id)? else {
                continue;
            };
            let assets = label_assets(conn, &tag.label_id)?;
            let (mut base, mut used, mut stale) = (0i64, GoalValueUsed::Market, false);
            for ((account, asset), p) in &valuer.positions {
                if !assets.contains(asset)
                    || held.contains(&(account.clone(), Some(asset.clone())))
                    || held.contains(&(account.clone(), None))
                {
                    continue;
                }
                let (v, u) = p.pick(tag.value_mode);
                base += v;
                stale |= p.missing;
                if u != GoalValueUsed::Market {
                    used = u;
                }
            }
            let value = fx
                .convert(base, &valuer.base, &valuer.today)?
                .map_or(0, |c| c.amount);
            push(
                &mut out,
                GoalSourceValue {
                    kind: GoalSourceKind::AssetTag,
                    ref_id: Some(tag.label_id),
                    account_id: None,
                    name,
                    value,
                    value_used: used,
                    share: None,
                    ignored_negative: None,
                    stale,
                },
            );
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Ligar e desligar

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalLinkKind {
    Account,
    Asset,
    AssetTag,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalLinkArgs {
    pub goal_id: Id,
    pub kind: GoalLinkKind,
    /// A conta, o ativo ou a etiqueta (id ou nome).
    pub ref_id: String,
    /// Conta do ativo (sem ela, a única conta em que o ativo está).
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Fração do ativo (0,01–1); sem ela, o ativo inteiro ou o pote comum.
    #[serde(default)]
    #[ts(optional)]
    pub share: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub value_mode: Option<GoalValueMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalUnlinkArgs {
    pub goal_id: Id,
    pub kind: GoalLinkKind,
    pub ref_id: String,
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
}

fn label_id(conn: &Connection, r: &str) -> Result<Id> {
    let r = r.trim();
    conn.query_row(
        "SELECT id FROM asset_label WHERE id = ?1 OR name = ?1 COLLATE NOCASE ORDER BY id = ?1 DESC LIMIT 1",
        [r],
        |x| x.get(0),
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-orcamento-meta-erro-etiqueta",
            &args(&[("etiqueta", r.to_string().into())]),
        ))
    })
}

/// A conta do ativo: a pedida ou a única com o ativo.
fn asset_account(conn: &Connection, asset: &str, account: Option<&str>) -> Result<Id> {
    if let Some(a) = account.filter(|a| !a.trim().is_empty()) {
        return Ok(a.trim().to_string());
    }
    let mut stmt =
        conn.prepare_cached("SELECT DISTINCT account_id FROM trade WHERE asset_id = ?1")?;
    let accounts: Vec<Id> = stmt
        .query_map([asset], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    match accounts.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(Error::invalid(t(
            "nucleo-orcamento-meta-erro-ativo-sem-conta",
        ))),
        _ => Err(Error::invalid(t(
            "nucleo-orcamento-meta-erro-ativo-varias-contas",
        ))),
    }
}

fn holding_inputs(conn: &Connection, goal_id: &str) -> Result<Vec<HoldingInput>> {
    let mut stmt = conn.prepare_cached(
        "SELECT account_id, asset_id, share, value_mode FROM goal_holding
          WHERE goal_id = ?1 ORDER BY created_at, id",
    )?;
    let v = stmt
        .query_map([goal_id], |r| {
            Ok(HoldingInput {
                account_id: r.get(0)?,
                asset_id: r.get(1)?,
                share: r.get(2)?,
                value_mode: Some(GoalValueMode::parse(&r.get::<_, String>(3)?)),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Liga uma fonte à meta (conta, ativo ou etiqueta); ligar de novo troca a fração e o valor.
pub fn link(tx: &Transaction<'_>, a: &GoalLinkArgs) -> Result<()> {
    let goal = super::load(tx, &a.goal_id)?;
    if goal.archived {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-arquivada")));
    }
    let r = a.ref_id.trim();
    match a.kind {
        GoalLinkKind::Account => {
            let ok: bool = tx.query_row(
                "SELECT EXISTS (SELECT 1 FROM account WHERE id = ?1)",
                [r],
                |x| x.get(0),
            )?;
            if !ok {
                return Err(Error::not_found(t_args(
                    "nucleo-educacao-conta",
                    &args(&[("id", r.to_string().into())]),
                )));
            }
            tx.execute(
                "INSERT OR IGNORE INTO goal_account (goal_id, account_id) VALUES (?1, ?2)",
                params![goal.id, r],
            )?;
        }
        GoalLinkKind::Asset => {
            let account = asset_account(tx, r, a.account_id.as_deref())?;
            let mut list = holding_inputs(tx, &goal.id)?;
            list.retain(|h| !(h.account_id == account && h.asset_id.as_deref() == Some(r)));
            list.push(HoldingInput {
                account_id: account,
                asset_id: Some(r.to_string()),
                share: a.share,
                value_mode: a.value_mode,
            });
            holdings::set(
                tx,
                &HoldingsSetArgs {
                    goal_id: goal.id.clone(),
                    holdings: list,
                },
            )?;
        }
        GoalLinkKind::AssetTag => {
            let label = label_id(tx, r)?;
            let mode = a.value_mode.unwrap_or_default();
            let now = now_utc();
            tx.execute(
                "INSERT INTO goal_source (id, goal_id, kind, ref_id, value_mode, created_at, updated_at)
                 VALUES (?1, ?2, 'asset_tag', ?3, ?4, ?5, ?5)
                 ON CONFLICT (goal_id, kind, ref_id) DO UPDATE SET value_mode = excluded.value_mode,
                                                                   updated_at = excluded.updated_at",
                params![new_id(), goal.id, label, mode.as_str(), now],
            )?;
        }
    }
    tx.execute(
        "UPDATE goal SET updated_at = ?2 WHERE id = ?1",
        params![goal.id, now_utc()],
    )?;
    Ok(())
}

/// Desliga uma fonte da meta.
pub fn unlink(tx: &Transaction<'_>, a: &GoalUnlinkArgs) -> Result<()> {
    let goal = super::load(tx, &a.goal_id)?;
    let r = a.ref_id.trim();
    let n = match a.kind {
        GoalLinkKind::Account => tx.execute(
            "DELETE FROM goal_account WHERE goal_id = ?1 AND account_id = ?2",
            params![goal.id, r],
        )?,
        GoalLinkKind::Asset => match a.account_id.as_deref().filter(|x| !x.trim().is_empty()) {
            Some(acc) => tx.execute(
                "DELETE FROM goal_holding WHERE goal_id = ?1 AND asset_id = ?2 AND account_id = ?3",
                params![goal.id, r, acc.trim()],
            )?,
            None => tx.execute(
                "DELETE FROM goal_holding WHERE goal_id = ?1 AND asset_id = ?2",
                params![goal.id, r],
            )?,
        },
        GoalLinkKind::AssetTag => {
            let label = label_id(tx, r).unwrap_or_else(|_| r.to_string());
            tx.execute(
                "DELETE FROM goal_source WHERE goal_id = ?1 AND kind = 'asset_tag' AND ref_id = ?2",
                params![goal.id, label],
            )?
        }
    };
    if n == 0 {
        return Err(Error::not_found(t(
            "nucleo-orcamento-meta-erro-fonte-nao-ligada",
        )));
    }
    tx.execute(
        "UPDATE goal SET updated_at = ?2 WHERE id = ?1",
        params![goal.id, now_utc()],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Sugestão: investimentos de liquidez diária

/// Uma posição de liquidez diária que pode entrar na meta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GoalLiquidityCandidate {
    pub account_id: Id,
    pub account_name: String,
    pub asset_id: Id,
    pub name: String,
    pub ticker: Option<String>,
    /// Valor líquido de resgate (ou de mercado), na moeda base.
    #[ts(type = "number")]
    pub value: i64,
    pub value_used: GoalValueUsed,
    pub currency: String,
}

/// Posições de liquidez diária (renda fixa com liquidez diária, Tesouro Selic, fundo DI ou
/// referenciado) que ainda não estão nesta meta.
pub fn liquidity_candidates(
    conn: &Connection,
    goal_id: &str,
    today: NaiveDate,
) -> Result<Vec<GoalLiquidityCandidate>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.ticker FROM asset a
           LEFT JOIN fixed_income_terms f ON f.asset_id = a.id
          WHERE a.archived = 0
            AND ((f.liquidity = 'daily' AND f.kind <> 'other') OR f.kind = 'tesouro_selic'
                 OR (a.class = 'fund' AND (' ' || lower(a.name) || ' ' LIKE '% di %'
                                           OR lower(a.name) LIKE '%referenciado%')))",
    )?;
    let assets: HashMap<Id, (String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))?
        .collect::<rusqlite::Result<_>>()?;
    if assets.is_empty() {
        return Ok(vec![]);
    }
    let mut linked =
        conn.prepare("SELECT account_id, asset_id FROM goal_holding WHERE goal_id = ?1")?;
    let held: HashSet<(Id, Option<Id>)> = linked
        .query_map([goal_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let valuer = Valuer::new(conn, today)?;
    let names: HashMap<Id, String> = {
        let mut s = conn.prepare("SELECT id, name FROM account")?;
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?
    };
    let mut out: Vec<GoalLiquidityCandidate> = valuer
        .positions
        .iter()
        .filter(|((acc, asset), _)| {
            assets.contains_key(asset)
                && !held.contains(&(acc.clone(), Some(asset.clone())))
                && !held.contains(&(acc.clone(), None))
        })
        .map(|((acc, asset), p)| {
            let (value, used) = p.pick(GoalValueMode::NetRedemption);
            let (name, ticker) = assets[asset].clone();
            GoalLiquidityCandidate {
                account_id: acc.clone(),
                account_name: names.get(acc).cloned().unwrap_or_default(),
                asset_id: asset.clone(),
                name,
                ticker,
                value,
                value_used: used,
                currency: valuer.base.clone(),
            }
        })
        .filter(|c| c.value > 0)
        .collect();
    out.sort_by(|a, b| b.value.cmp(&a.value).then(a.name.cmp(&b.name)));
    Ok(out)
}
