//! Concentração da carteira (docs/concentration.md): quanto um ativo, um grupo de ativos (uma
//! etiqueta marcada como grupo) ou uma classe pesa contra o limite. Funciona sem configuração:
//! 20% por ativo e 30% por grupo; a classe só tem limite quando há alvo de alocação (alvo +
//! tolerância). Acima do limite, `warning`; a partir de 1,5 vez o limite, `critical`. O Tesouro
//! Direto e a renda fixa com FGC dentro da garantia ficam fora do limite padrão por ativo.

use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AssetBrief, PortfolioSummary};
use crate::error::{Error, Result};
use crate::i18n::t;
use crate::ids::Id;

pub const ASSET_DEFAULT_SETTING: &str = "concentration.asset_default_pct";
pub const GROUP_DEFAULT_SETTING: &str = "concentration.group_default_pct";
pub const ASSET_DEFAULT_PCT: f64 = 20.0;
pub const GROUP_DEFAULT_PCT: f64 = 30.0;
/// A partir deste múltiplo do limite, o alerta é crítico.
pub const CRITICAL_FACTOR: f64 = 1.5;

/// O que um limite cobre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ConcentrationScope")]
pub enum Scope {
    /// Um ativo (`asset_id`).
    Asset,
    /// O padrão de ativo.
    AssetDefault,
    /// O padrão de grupo.
    GroupDefault,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetConcentrationLimitArgs {
    pub scope: Scope,
    /// Com `scope = asset`.
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<Id>,
    /// % da carteira; `null` volta ao padrão.
    #[serde(default)]
    pub pct: Option<f64>,
}

pub fn defaults(conn: &Connection) -> Result<(f64, f64)> {
    Ok((
        crate::settings::get_or(conn, ASSET_DEFAULT_SETTING, ASSET_DEFAULT_PCT)?,
        crate::settings::get_or(conn, GROUP_DEFAULT_SETTING, GROUP_DEFAULT_PCT)?,
    ))
}

/// Grava um limite (ativo ou padrão). Sem `pct`, o do ativo volta ao padrão e o padrão volta
/// ao do app.
pub fn set_limit(conn: &Connection, a: &SetConcentrationLimitArgs) -> Result<()> {
    let pct = crate::assets::labels::check_pct(a.pct)?;
    match a.scope {
        Scope::Asset => {
            let id = a
                .asset_id
                .as_deref()
                .ok_or_else(|| Error::invalid(t("nucleo-carteira-limite-sem-ativo")))?;
            crate::assets::get(conn, id)?;
            conn.execute(
                "UPDATE asset SET concentration_limit_pct = ?2 WHERE id = ?1",
                rusqlite::params![id, pct],
            )?;
        }
        Scope::AssetDefault => match pct {
            Some(p) => crate::settings::set(conn, ASSET_DEFAULT_SETTING, &p)?,
            None => crate::settings::delete(conn, ASSET_DEFAULT_SETTING)?,
        },
        Scope::GroupDefault => match pct {
            Some(p) => crate::settings::set(conn, GROUP_DEFAULT_SETTING, &p)?,
            None => crate::settings::delete(conn, GROUP_DEFAULT_SETTING)?,
        },
    }
    Ok(())
}

/// Limite próprio do ativo (sem ele, o padrão).
pub fn asset_limit(conn: &Connection, asset_id: &str) -> Result<Option<f64>> {
    Ok(conn.query_row(
        "SELECT concentration_limit_pct FROM asset WHERE id = ?1",
        [asset_id],
        |r| r.get(0),
    )?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ConcentrationKind")]
pub enum Kind {
    Asset,
    Group,
    Class,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ConcentrationSeverity")]
pub enum Severity {
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConcentrationAlert {
    pub kind: Kind,
    /// Id do ativo, nome da etiqueta ou classe.
    #[serde(rename = "ref")]
    #[ts(rename = "ref")]
    pub reference: String,
    pub label: String,
    /// Peso atual e limite, frações da carteira (0,32 = 32%).
    pub weight: f64,
    pub limit: f64,
    pub severity: Severity,
    /// Valor acima do limite, na moeda base: (peso − limite) × valor da carteira.
    #[ts(type = "number")]
    pub excess_base: i64,
    /// Ativos do grupo (ou o próprio ativo).
    pub assets: Vec<AssetBrief>,
}

/// Peso de cada ativo (somado entre contas) e o valor total da carteira.
fn weights(s: &PortfolioSummary) -> (Vec<(AssetBrief, i64)>, i64) {
    let mut by: Vec<(AssetBrief, i64)> = vec![];
    for p in &s.positions {
        let v = p.market_value_base.unwrap_or(p.cost_base).max(0);
        match by.iter_mut().find(|(a, _)| a.id == p.asset.id) {
            Some(e) => e.1 += v,
            None => by.push((p.asset.clone(), v)),
        }
    }
    (by, s.market_value.max(0))
}

fn severity(weight: f64, limit: f64) -> Option<Severity> {
    if weight >= limit * CRITICAL_FACTOR {
        Some(Severity::Critical)
    } else if weight > limit {
        Some(Severity::Warning)
    } else {
        None
    }
}

/// O limite padrão por ativo não vale para o Tesouro Direto (risco do governo) nem para a
/// renda fixa coberta pelo FGC enquanto o valor do ativo cabe na garantia por instituição
/// (CDB, RDB, LC, LCI, LCA e poupança até R$ 250 mil): a reserva da família costuma estar
/// toda num título só, e isso não é o risco de concentração que o alerta quer mostrar. Um
/// limite próprio no ativo vale sempre.
fn default_exempt(conn: &Connection, a: &AssetBrief, value_base: i64) -> Result<bool> {
    use crate::assets::{AssetClass, FixedIncomeKind as K};
    match a.class {
        AssetClass::TreasuryBr => Ok(true),
        AssetClass::FixedIncome => {
            let Some(fi) = crate::assets::get(conn, &a.id)?.fixed_income else {
                return Ok(false);
            };
            if matches!(
                fi.kind,
                K::TesouroSelic | K::TesouroPrefixado | K::TesouroIpca | K::TesouroRenda
            ) {
                return Ok(true);
            }
            if crate::score::fixed_income::fgc_covered(fi.kind) != Some(true)
                || crate::household::base_currency(conn)? != "BRL"
            {
                return Ok(false);
            }
            let limit = crate::score::fixed_income::params().fgc_limit() * Decimal::from(100);
            Ok(Decimal::from(value_base) <= limit)
        }
        _ => Ok(false),
    }
}

/// O limite que vale para o ativo: o próprio, o padrão, ou nenhum (isento do padrão).
fn limit_for(
    conn: &Connection,
    a: &AssetBrief,
    value_base: i64,
    default: f64,
) -> Result<Option<f64>> {
    if let Some(own) = asset_limit(conn, &a.id)? {
        return Ok(Some(own / 100.0));
    }
    if default_exempt(conn, a, value_base)? {
        return Ok(None);
    }
    Ok(Some(default / 100.0))
}

fn excess(weight: f64, limit: f64, total: i64) -> i64 {
    ((weight - limit) * total as f64).round() as i64
}

/// Os alertas de concentração da carteira: por ativo, por grupo (etiquetas marcadas) e por
/// classe (só com alvo), do mais grave ao mais leve.
pub fn alerts(conn: &Connection, s: &PortfolioSummary) -> Result<Vec<ConcentrationAlert>> {
    let (assets, total) = weights(s);
    if total <= 0 {
        return Ok(vec![]);
    }
    let (asset_default, group_default) = defaults(conn)?;
    let mut out = vec![];
    for (a, v) in &assets {
        let weight = *v as f64 / total as f64;
        let Some(limit) = limit_for(conn, a, *v, asset_default)? else {
            continue;
        };
        if let Some(sev) = severity(weight, limit) {
            out.push(ConcentrationAlert {
                kind: Kind::Asset,
                reference: a.id.clone(),
                label: a.ticker.clone().unwrap_or_else(|| a.name.clone()),
                weight,
                limit,
                severity: sev,
                excess_base: excess(weight, limit, total),
                assets: vec![a.clone()],
            });
        }
    }
    for l in crate::assets::labels::list(conn)?
        .into_iter()
        .filter(|l| l.is_concentration_group)
    {
        let members: Vec<&(AssetBrief, i64)> = assets
            .iter()
            .filter(|(a, _)| l.asset_ids.contains(&a.id))
            .collect();
        let v: i64 = members.iter().map(|(_, v)| v).sum();
        let weight = v as f64 / total as f64;
        let limit = l.concentration_limit_pct.unwrap_or(group_default) / 100.0;
        if let Some(sev) = severity(weight, limit) {
            out.push(ConcentrationAlert {
                kind: Kind::Group,
                reference: l.name.clone(),
                label: l.name.clone(),
                weight,
                limit,
                severity: sev,
                excess_base: excess(weight, limit, total),
                assets: members.iter().map(|(a, _)| a.clone()).collect(),
            });
        }
    }
    let targets = super::engine::targets_by_class(conn)?;
    for c in &s.by_class {
        let Some((pct, tol)) = targets.get(c.class.as_str()) else {
            continue;
        };
        let limit = (pct + tol) / 100.0;
        if limit <= 0.0 {
            continue;
        }
        if let Some(sev) = severity(c.weight, limit) {
            out.push(ConcentrationAlert {
                kind: Kind::Class,
                reference: c.class.as_str().to_string(),
                label: c.label.clone(),
                weight: c.weight,
                limit,
                severity: sev,
                excess_base: excess(c.weight, limit, total),
                assets: vec![],
            });
        }
    }
    out.sort_by(|a, b| {
        (b.severity == Severity::Critical)
            .cmp(&(a.severity == Severity::Critical))
            .then(b.excess_base.cmp(&a.excess_base))
    });
    Ok(out)
}

/// Um ativo ou grupo com o peso e o limite que vale para ele.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConcentrationItem {
    #[serde(rename = "ref")]
    #[ts(rename = "ref")]
    pub reference: String,
    pub label: String,
    pub weight: f64,
    pub limit: f64,
    /// O limite é o próprio (e não o padrão).
    pub custom_limit: bool,
    /// Fora do limite padrão (Tesouro Direto, ou renda fixa com FGC dentro da garantia).
    #[serde(default)]
    pub exempt: bool,
    #[ts(type = "number")]
    pub value_base: i64,
    pub assets: Vec<AssetBrief>,
}

/// `portfolio.concentration` (`get_concentration`): os alertas e o detalhe de cada ativo e grupo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConcentrationReport {
    #[ts(type = "number")]
    pub market_value_base: i64,
    pub asset_default_pct: f64,
    pub group_default_pct: f64,
    pub alerts: Vec<ConcentrationAlert>,
    /// Do maior peso para o menor.
    pub assets: Vec<ConcentrationItem>,
    pub groups: Vec<ConcentrationItem>,
}

pub fn report(conn: &Connection, s: &PortfolioSummary) -> Result<ConcentrationReport> {
    let (assets, total) = weights(s);
    let (asset_default, group_default) = defaults(conn)?;
    let w = |v: i64| {
        if total > 0 {
            v as f64 / total as f64
        } else {
            0.0
        }
    };
    let mut items = vec![];
    for (a, v) in &assets {
        let own = asset_limit(conn, &a.id)?;
        let limit = limit_for(conn, a, *v, asset_default)?;
        items.push(ConcentrationItem {
            reference: a.id.clone(),
            label: a.ticker.clone().unwrap_or_else(|| a.name.clone()),
            weight: w(*v),
            // Isento do padrão: 100% (nunca passa), e `exempt` diz por quê.
            limit: limit.unwrap_or(1.0),
            exempt: limit.is_none(),
            custom_limit: own.is_some(),
            value_base: *v,
            assets: vec![a.clone()],
        });
    }
    items.sort_by_key(|i| std::cmp::Reverse(i.value_base));
    let mut groups = vec![];
    for l in crate::assets::labels::list(conn)?
        .into_iter()
        .filter(|l| l.is_concentration_group)
    {
        let members: Vec<&(AssetBrief, i64)> = assets
            .iter()
            .filter(|(a, _)| l.asset_ids.contains(&a.id))
            .collect();
        let v: i64 = members.iter().map(|(_, v)| v).sum();
        groups.push(ConcentrationItem {
            reference: l.name.clone(),
            label: l.name.clone(),
            weight: w(v),
            exempt: false,
            limit: l.concentration_limit_pct.unwrap_or(group_default) / 100.0,
            custom_limit: l.concentration_limit_pct.is_some(),
            value_base: v,
            assets: members.iter().map(|(a, _)| a.clone()).collect(),
        });
    }
    Ok(ConcentrationReport {
        market_value_base: total,
        asset_default_pct: asset_default,
        group_default_pct: group_default,
        alerts: alerts(conn, s)?,
        assets: items,
        groups,
    })
}

/// (ativo, valor atual, limite do ativo, espaço, isento do padrão).
pub type Room = (AssetBrief, i64, f64, i64, bool);

/// Espaço de cada ativo da carteira até o limite (do ativo e de cada grupo dele) com o total
/// depois do aporte: (ativo, valor atual, limite do ativo, espaço, isento do padrão). Grupo
/// cheio zera o espaço dos ativos dele. O ativo isento do limite padrão (Tesouro, renda fixa
/// com FGC) continua com o padrão para espalhar o aporte, mas não aparece como acima do limite.
pub fn rooms(conn: &Connection, s: &PortfolioSummary, total_after: i64) -> Result<Vec<Room>> {
    let (assets, _) = weights(s);
    let (asset_default, group_default) = defaults(conn)?;
    let groups: Vec<(Vec<Id>, f64)> = crate::assets::labels::list(conn)?
        .into_iter()
        .filter(|l| l.is_concentration_group)
        .map(|l| {
            (
                l.asset_ids,
                l.concentration_limit_pct.unwrap_or(group_default) / 100.0,
            )
        })
        .collect();
    let mut out = vec![];
    for (a, v) in assets {
        let exempt = limit_for(conn, &a, v, asset_default)?.is_none();
        let limit = asset_limit(conn, &a.id)?.unwrap_or(asset_default) / 100.0;
        let mut room = ((limit * total_after as f64).round() as i64 - v).max(0);
        for (members, glimit) in &groups {
            if !members.contains(&a.id) {
                continue;
            }
            let gv: i64 = s
                .positions
                .iter()
                .filter(|p| members.contains(&p.asset.id))
                .map(|p| p.market_value_base.unwrap_or(p.cost_base).max(0))
                .sum();
            room = room.min(((glimit * total_after as f64).round() as i64 - gv).max(0));
        }
        out.push((a, v, limit, room, exempt));
    }
    Ok(out)
}
