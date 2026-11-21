//! Custo de longo prazo das taxas (`portfolio.fee_cost`): quanto a taxa de administração, a de
//! performance e a de carregamento de fundos e planos de previdência tiram do patrimônio em 10,
//! 20 e 30 anos, contra o mesmo dinheiro sem taxas e contra um cenário de taxa menor.
//!
//! Tudo em valores reais de hoje: o retorno suposto é real (acima do IPCA), como no resto do
//! app. Juros compostos mensais; aportes mensais opcionais (divididos pelos ativos na
//! proporção do valor de hoje), com a taxa de carregamento descontada de cada aporte.
//!
//! - Taxa de administração: do extrato do fundo no registro da CVM ou do cadastro do plano.
//! - Performance: só pesa com um excesso esperado sobre o benchmark (`excess_return`, padrão
//!   0: a taxa de performance não custa nada se o fundo não bater o índice).
//! - Cenário alternativo: taxa de administração `alternative_fee` (padrão 0,5% a.a.), sem
//!   performance nem carregamento.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AssetBrief, PortfolioArgs};
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const DEFAULT_REAL_RETURN: f64 = 0.04;
pub const DEFAULT_ALTERNATIVE_FEE: f64 = 0.005;
pub const DEFAULT_HORIZONS: [u32; 3] = [10, 20, 30];

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeeCostArgs {
    /// Retorno real bruto suposto, fração a.a. (padrão 0,04).
    #[serde(default)]
    #[ts(optional)]
    pub real_return: Option<f64>,
    /// Taxa de administração do cenário alternativo, fração a.a. (padrão 0,005).
    #[serde(default)]
    #[ts(optional)]
    pub alternative_fee: Option<f64>,
    /// Excesso de retorno esperado sobre o benchmark (para a taxa de performance), fração a.a.
    #[serde(default)]
    #[ts(optional)]
    pub excess_return: Option<f64>,
    /// Aporte mensal total em centavos (dividido pelos ativos na proporção do valor).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub monthly_contribution: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub horizons: Option<Vec<u32>>,
}

/// Valores projetados num horizonte (centavos, valores reais de hoje).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeeProjection {
    pub years: u32,
    #[ts(type = "number")]
    pub no_fees: i64,
    #[ts(type = "number")]
    pub current: i64,
    #[ts(type = "number")]
    pub alternative: i64,
    /// O que as taxas de hoje tiram (sem taxas − com as taxas de hoje).
    #[ts(type = "number")]
    pub cost: i64,
    /// O que o cenário de taxa menor devolve (alternativo − taxas de hoje).
    #[ts(type = "number")]
    pub savings: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeeHolding {
    pub asset: AssetBrief,
    #[ts(type = "number")]
    pub value: i64,
    /// Frações a.a. (administração e performance) e sobre o aporte (carregamento).
    pub admin_fee: f64,
    pub performance_fee: Option<f64>,
    pub loading_fee: Option<f64>,
    /// De onde vieram as taxas.
    pub source: String,
    pub projections: Vec<FeeProjection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeeCost {
    pub real_return: f64,
    pub alternative_fee: f64,
    pub excess_return: f64,
    #[ts(type = "number")]
    pub monthly_contribution: i64,
    pub holdings: Vec<FeeHolding>,
    pub totals: Vec<FeeProjection>,
    /// Fundos e planos sem taxa conhecida (fora da conta).
    pub missing: Vec<String>,
    pub notes: Vec<String>,
}

/// Valor futuro com juros compostos mensais (taxa anual) e aporte no fim de cada mês.
pub fn future_value(start: f64, annual: f64, monthly: f64, years: u32) -> f64 {
    let g = (1.0 + annual).powf(1.0 / 12.0) - 1.0;
    let n = f64::from(years * 12);
    let growth = (1.0 + g).powf(n);
    let annuity = if g.abs() < 1e-12 {
        n
    } else {
        (growth - 1.0) / g
    };
    start * growth + monthly * annuity
}

/// Retorno líquido de administração (sobre o patrimônio) e de performance (sobre o excesso).
fn net_return(gross: f64, admin: f64, perf: f64, excess: f64) -> f64 {
    (1.0 + gross) * (1.0 - admin) - 1.0 - perf * excess.max(0.0)
}

#[allow(clippy::too_many_arguments)]
fn project(
    value: f64,
    monthly: f64,
    gross: f64,
    admin: f64,
    perf: f64,
    loading: f64,
    alt: f64,
    excess: f64,
    years: u32,
) -> FeeProjection {
    let no_fees = future_value(value, gross, monthly, years);
    let current = future_value(
        value,
        net_return(gross, admin, perf, excess),
        monthly * (1.0 - loading),
        years,
    );
    let alternative = future_value(value, net_return(gross, alt, 0.0, 0.0), monthly, years);
    let c = |v: f64| v.round() as i64;
    FeeProjection {
        years,
        no_fees: c(no_fees),
        current: c(current),
        alternative: c(alternative),
        cost: c(no_fees - current),
        savings: c(alternative - current),
    }
}

fn pct(s: Option<&str>) -> Option<f64> {
    s.and_then(|v| v.trim().replace(',', ".").parse::<f64>().ok())
        .map(|v| v / 100.0)
}

fn frac(s: Option<&str>) -> Option<f64> {
    s.and_then(|v| v.trim().parse::<f64>().ok())
}

/// Ativo, valor, administração, performance, carregamento e a origem das taxas.
type FeeRow = (AssetBrief, i64, f64, Option<f64>, Option<f64>, String);

pub fn fee_cost(conn: &rusqlite::Connection, a: &FeeCostArgs) -> Result<FeeCost> {
    let gross = a.real_return.unwrap_or(DEFAULT_REAL_RETURN);
    let alt = a.alternative_fee.unwrap_or(DEFAULT_ALTERNATIVE_FEE);
    let excess = a.excess_return.unwrap_or(0.0);
    if !(-0.5..=0.5).contains(&gross)
        || !(0.0..=0.2).contains(&alt)
        || !(0.0..=0.5).contains(&excess)
    {
        return Err(Error::invalid(t("nucleo-carteira-custo-erro-parametros")));
    }
    let monthly_total = a.monthly_contribution.unwrap_or(0).max(0);
    let horizons = a
        .horizons
        .clone()
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| DEFAULT_HORIZONS.to_vec());
    if horizons.iter().any(|y| *y == 0 || *y > 60) {
        return Err(Error::invalid(t("nucleo-carteira-custo-erro-horizontes")));
    }
    let summary = super::engine::summary(
        conn,
        &PortfolioArgs {
            as_of: None,
            account_ids: None,
            deflator: None,
            by_account: None,
            include_closed: None,
        },
    )?;
    let registry = crate::score::funds::registry::load(conn).unwrap_or_default();
    let mut rows: Vec<FeeRow> = vec![];
    let mut missing = vec![];
    for p in &summary.positions {
        let class = p.asset.class;
        if !matches!(class, AssetClass::Fund | AssetClass::Pension) {
            continue;
        }
        let Some(value) = p.market_value_base.filter(|v| *v > 0) else {
            continue;
        };
        let label = p
            .asset
            .ticker
            .clone()
            .unwrap_or_else(|| p.asset.name.clone());
        let fees = if class == AssetClass::Pension {
            crate::pension::get_plan(conn, &p.asset.id)?.and_then(|plan| {
                let admin = pct(plan.admin_fee_pct.as_deref())?;
                Some((
                    admin,
                    None,
                    pct(plan.loading_fee_pct.as_deref()),
                    t("nucleo-carteira-custo-fonte-previdencia"),
                ))
            })
        } else {
            let asset = crate::assets::get(conn, &p.asset.id)?;
            let cnpj = asset
                .cnpj
                .as_deref()
                .map(crate::research::cnpj_digits)
                .unwrap_or_default();
            registry.funds.get(&cnpj).and_then(|f| {
                let admin = frac(f.admin_fee.as_deref())?;
                Some((
                    admin,
                    frac(f.perf_fee.as_deref()).filter(|v| *v > 0.0),
                    None,
                    match f.extrato_date.as_deref() {
                        Some(d) => t_args(
                            "nucleo-carteira-custo-fonte-cvm-data",
                            &i18n::args(&[("data", d.to_string().into())]),
                        ),
                        None => t("nucleo-carteira-custo-fonte-cvm"),
                    },
                ))
            })
        };
        match fees {
            Some((admin, perf, loading, source)) => {
                rows.push((p.asset.clone(), value, admin, perf, loading, source))
            }
            None => missing.push(label),
        }
    }
    let total_value: i64 = rows.iter().map(|r| r.1).sum();
    let mut holdings = vec![];
    let mut totals: Vec<FeeProjection> = horizons
        .iter()
        .map(|y| FeeProjection {
            years: *y,
            ..Default::default()
        })
        .collect();
    for (asset, value, admin, perf, loading, source) in rows {
        let monthly = if total_value > 0 {
            monthly_total as f64 * value as f64 / total_value as f64
        } else {
            0.0
        };
        let projections: Vec<FeeProjection> = horizons
            .iter()
            .map(|y| {
                project(
                    value as f64,
                    monthly,
                    gross,
                    admin,
                    perf.unwrap_or(0.0),
                    loading.unwrap_or(0.0),
                    alt.min(admin),
                    excess,
                    *y,
                )
            })
            .collect();
        for (t, p) in totals.iter_mut().zip(&projections) {
            t.no_fees += p.no_fees;
            t.current += p.current;
            t.alternative += p.alternative;
            t.cost += p.cost;
            t.savings += p.savings;
        }
        holdings.push(FeeHolding {
            asset,
            value,
            admin_fee: admin,
            performance_fee: perf,
            loading_fee: loading,
            source,
            projections,
        });
    }
    holdings.sort_by(|a, b| {
        let last = |h: &FeeHolding| h.projections.last().map_or(0, |p| p.cost);
        last(b).cmp(&last(a))
    });
    let pct = |v: f64, d: u8| crate::ai::alerts::pct(i18n::current(), v, d);
    let mut notes = vec![
        t_args(
            "nucleo-carteira-custo-nota-retorno",
            &i18n::args(&[("retorno", pct(gross, 1).into())]),
        ),
        t_args(
            "nucleo-carteira-custo-nota-alternativa",
            &i18n::args(&[("taxa", pct(alt, 2).into())]),
        ),
    ];
    if excess <= 0.0 {
        notes.push(t("nucleo-carteira-custo-nota-performance"));
    }
    Ok(FeeCost {
        real_return: gross,
        alternative_fee: alt,
        excess_return: excess,
        monthly_contribution: monthly_total,
        holdings,
        totals,
        missing,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fees_compound_over_decades() {
        // R$ 100 mil, 4% real, 2% de administração × 0,5%, sem aportes, 30 anos.
        let p = project(100_000.0, 0.0, 0.04, 0.02, 0.0, 0.0, 0.005, 0.0, 30);
        let no_fees = 100_000.0 * 1.04f64.powi(30);
        assert!((p.no_fees as f64 - no_fees).abs() < 1.0, "{p:?}");
        let current = 100_000.0 * (1.04f64 * 0.98).powi(30);
        assert!((p.current as f64 - current).abs() < 1.0, "{p:?}");
        // ~44% do que seria o patrimônio sem taxas.
        assert!(p.cost as f64 / no_fees > 0.4);
        assert!(p.savings > 0);
        // Carregamento só pesa sobre os aportes.
        let with_loading = project(0.0, 1000.0, 0.04, 0.0, 0.0, 0.05, 0.0, 0.0, 10);
        assert!((with_loading.current as f64 / with_loading.no_fees as f64 - 0.95).abs() < 1e-4);
    }
}
