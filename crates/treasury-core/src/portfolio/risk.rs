//! Risco da carteira (`portfolio.risk`): volatilidade, pior queda (drawdown), Sharpe contra o
//! CDI, correlação entre ativos e entre classes e a contribuição de cada classe e ativo para
//! o risco, para a carteira inteira e por classe.
//!
//! Base: retornos mensais (TWR por Modified Dietz, com proventos, em reais) de cada ativo da
//! carteira de hoje nos últimos N meses completos, os mesmos de Carteira › Rentabilidade.
//! A carteira e as classes são a média dos ativos ponderada pelo valor de hoje ("a carteira
//! de hoje no passado"); num mês em que um ativo ainda não existia, os pesos se redistribuem
//! entre os que existiam. Ao lado, o histórico real da carteira (com as compras e vendas de
//! cada época).
//!
//! Volatilidade = desvio-padrão mensal × √12; retorno anual geométrico; Sharpe = média do
//! excesso mensal sobre o CDI ÷ desvio-padrão mensal × √12; pior queda = maior recuo do
//! acumulado em relação ao topo anterior. Correlação de Pearson com pelo menos
//! [`MIN_OVERLAP`] meses em comum.

use std::collections::BTreeMap;

use chrono::Datelike;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AssetClass, PerformanceArgs, PortfolioArgs};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const DEFAULT_MONTHS: u32 = 36;
/// Meses em comum para uma correlação (e meses mínimos para as métricas de um ativo).
pub const MIN_OVERLAP: usize = 12;
/// Ativos na matriz de correlação (os maiores pesos).
pub const MAX_MATRIX_ASSETS: usize = 15;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "PortfolioRiskArgs")]
pub struct RiskArgs {
    /// Meses completos analisados (padrão 36, de 12 a 120).
    #[ts(optional)]
    #[serde(default)]
    pub months: Option<u32>,
}

/// Métricas de uma série mensal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "RiskMetrics")]
pub struct Metrics {
    /// Meses com retorno.
    pub months: u32,
    /// Retorno anual (geométrico) e volatilidade anual, frações.
    pub annual_return: f64,
    pub volatility: f64,
    /// Pior queda do acumulado (fração negativa, ex.: −0,18) e os meses do topo e do fundo.
    pub max_drawdown: f64,
    pub drawdown_peak: Option<String>,
    pub drawdown_trough: Option<String>,
    /// Mês em que voltou ao topo (`null` = ainda não voltou).
    pub drawdown_recovered: Option<String>,
    /// Sharpe contra o CDI (`null` sem CDI ou sem volatilidade).
    pub sharpe: Option<f64>,
    /// CDI anual nos mesmos meses.
    pub cdi_annual: Option<f64>,
    pub best_month: Option<f64>,
    pub worst_month: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ClassRisk")]
pub struct ClassRisk {
    pub class: AssetClass,
    pub label: String,
    /// Peso na carteira hoje (0–1).
    pub weight: f64,
    pub metrics: Option<Metrics>,
    /// Parte do risco da carteira (soma ≈ 1): peso × covariância com a carteira ÷ variância.
    pub risk_share: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetRisk")]
pub struct AssetRisk {
    pub asset_id: String,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub weight: f64,
    /// `null` com menos de [`MIN_OVERLAP`] meses de histórico.
    pub metrics: Option<Metrics>,
    pub risk_share: Option<f64>,
}

/// Matriz de correlação: `values[i][j]` entre `labels[i]` e `labels[j]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CorrelationMatrix")]
pub struct Matrix {
    pub ids: Vec<String>,
    pub labels: Vec<String>,
    pub values: Vec<Vec<Option<f64>>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "PortfolioRisk")]
pub struct Risk {
    pub from_month: String,
    pub to_month: String,
    pub months: Vec<String>,
    /// A carteira de hoje no passado (pesos de hoje).
    pub portfolio: Option<Metrics>,
    /// O histórico real da carteira (compras e vendas de cada época).
    pub realized: Option<Metrics>,
    /// Retorno mensal da carteira de hoje no passado e o CDI do mês (para o gráfico).
    pub portfolio_monthly: Vec<Option<f64>>,
    pub cdi_monthly: Vec<Option<f64>>,
    pub by_class: Vec<ClassRisk>,
    pub assets: Vec<AssetRisk>,
    pub class_correlation: Matrix,
    pub asset_correlation: Matrix,
    #[ts(type = "number")]
    pub market_value: i64,
    pub notes: Vec<String>,
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

fn std(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

fn round(v: f64, dp: i32) -> f64 {
    let f = 10f64.powi(dp);
    (v * f).round() / f
}

/// Métricas da série (`None` nos meses sem retorno), com o CDI de cada mês.
pub fn metrics(months: &[String], r: &[Option<f64>], cdi: &[Option<f64>]) -> Option<Metrics> {
    let pts: Vec<(usize, f64)> = r
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.map(|v| (i, v)))
        .collect();
    if pts.len() < 2 {
        return None;
    }
    let vals: Vec<f64> = pts.iter().map(|p| p.1).collect();
    let n = vals.len() as f64;
    let growth: f64 = vals.iter().map(|v| 1.0 + v).product();
    let annual = growth.max(0.0).powf(12.0 / n) - 1.0;
    let sd = std(&vals);
    // Pior queda.
    let (mut level, mut peak, mut peak_i) = (1.0f64, 1.0f64, None::<usize>);
    let (mut worst, mut worst_peak, mut worst_trough) = (0.0f64, None, None);
    for (i, v) in &pts {
        level *= 1.0 + v;
        if level > peak {
            peak = level;
            peak_i = Some(*i);
        }
        let dd = level / peak - 1.0;
        if dd < worst {
            worst = dd;
            worst_peak = peak_i;
            worst_trough = Some(*i);
        }
    }
    let recovered = worst_trough.and_then(|t| {
        let mut lv = 1.0f64;
        // Queda logo no começo: o topo é o nível inicial (1,0), antes de qualquer mês.
        let mut peak_level = worst_peak.is_none().then_some(1.0f64);
        let mut out = None;
        for (i, v) in &pts {
            lv *= 1.0 + v;
            if Some(*i) == worst_peak {
                peak_level = Some(lv);
            }
            if *i > t
                && let Some(p) = peak_level
                && lv >= p
            {
                out = Some(*i);
                break;
            }
        }
        out
    });
    // Mês do topo antes de qualquer alta = o mês anterior à série (início).
    let label = |i: Option<usize>| i.and_then(|i| months.get(i).cloned());
    let peak_label = match worst_peak {
        Some(i) => label(Some(i)),
        None if worst_trough.is_some() => pts
            .first()
            .and_then(|(i, _)| i.checked_sub(1))
            .and_then(|i| months.get(i).cloned())
            .or_else(|| Some("início".into())),
        None => None,
    };
    let excess: Vec<f64> = pts
        .iter()
        .filter_map(|(i, v)| cdi.get(*i).copied().flatten().map(|c| v - c))
        .collect();
    let cdi_vals: Vec<f64> = pts
        .iter()
        .filter_map(|(i, _)| cdi.get(*i).copied().flatten())
        .collect();
    let cdi_annual = (cdi_vals.len() >= pts.len() / 2 && !cdi_vals.is_empty()).then(|| {
        cdi_vals
            .iter()
            .map(|c| 1.0 + c)
            .product::<f64>()
            .powf(12.0 / cdi_vals.len() as f64)
            - 1.0
    });
    let sharpe = (cdi_annual.is_some() && sd > 1e-9 && excess.len() >= 2)
        .then(|| mean(&excess) / sd * 12f64.sqrt());
    Some(Metrics {
        months: pts.len() as u32,
        annual_return: round(annual, 6),
        volatility: round(sd * 12f64.sqrt(), 6),
        max_drawdown: round(worst, 6),
        drawdown_peak: peak_label,
        drawdown_trough: label(worst_trough),
        drawdown_recovered: label(recovered),
        sharpe: sharpe.map(|s| round(s, 4)),
        cdi_annual: cdi_annual.map(|c| round(c, 6)),
        best_month: vals.iter().copied().reduce(f64::max).map(|v| round(v, 6)),
        worst_month: vals.iter().copied().reduce(f64::min).map(|v| round(v, 6)),
    })
}

/// Correlação de Pearson nos meses em que as duas séries têm valor.
pub fn correlation(a: &[Option<f64>], b: &[Option<f64>]) -> Option<f64> {
    let (xs, ys): (Vec<f64>, Vec<f64>) = a
        .iter()
        .zip(b)
        .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
        .unzip();
    if xs.len() < MIN_OVERLAP {
        return None;
    }
    let (mx, my) = (mean(&xs), mean(&ys));
    let cov: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let vx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    let vy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum();
    if vx <= 1e-18 || vy <= 1e-18 {
        return None;
    }
    Some(round(cov / (vx * vy).sqrt(), 4))
}

/// Média ponderada mês a mês, redistribuindo o peso de quem não tem retorno no mês.
fn weighted(series: &[(&[Option<f64>], f64)], len: usize) -> Vec<Option<f64>> {
    (0..len)
        .map(|i| {
            let (mut sum, mut w) = (0.0, 0.0);
            for (s, wt) in series {
                if let Some(v) = s.get(i).copied().flatten() {
                    sum += v * wt;
                    w += wt;
                }
            }
            (w > 0.0).then(|| sum / w)
        })
        .collect()
}

/// Parte do risco: peso × cov(série, carteira) ÷ var(carteira), nos meses em comum.
fn risk_share(weight: f64, s: &[Option<f64>], p: &[Option<f64>]) -> Option<f64> {
    let (xs, ys): (Vec<f64>, Vec<f64>) = s
        .iter()
        .zip(p)
        .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
        .unzip();
    if xs.len() < MIN_OVERLAP {
        return None;
    }
    let (mx, my) = (mean(&xs), mean(&ys));
    let cov: f64 = xs
        .iter()
        .zip(&ys)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum::<f64>();
    let var: f64 = ys.iter().map(|y| (y - my).powi(2)).sum::<f64>();
    (var > 1e-18).then(|| round(weight * cov / var, 4))
}

fn matrix(rows: &[(String, String, Vec<Option<f64>>)]) -> Matrix {
    Matrix {
        ids: rows.iter().map(|r| r.0.clone()).collect(),
        labels: rows.iter().map(|r| r.1.clone()).collect(),
        values: rows
            .iter()
            .map(|a| {
                rows.iter()
                    .map(|b| {
                        if a.0 == b.0 {
                            Some(1.0)
                        } else {
                            correlation(&a.2, &b.2)
                        }
                    })
                    .collect()
            })
            .collect(),
    }
}

/// CDI do mês a partir do acumulado.
fn monthly_from_cumulative(c: &[Option<f64>]) -> Vec<Option<f64>> {
    let mut prev = Some(0.0);
    c.iter()
        .map(|x| {
            let out = match (x, prev) {
                (Some(now), Some(p)) => Some((1.0 + now) / (1.0 + p) - 1.0),
                _ => None,
            };
            prev = *x;
            out
        })
        .collect()
}

pub fn risk(conn: &Connection, args: &RiskArgs) -> Result<Risk> {
    let n = args.months.unwrap_or(DEFAULT_MONTHS);
    if !(12..=120).contains(&n) {
        return Err(Error::invalid(t("nucleo-carteira-risco-erro-meses")));
    }
    let today = crate::time::today_local();
    let last = crate::time::add_months(today.with_day(1).expect("dia 1"), -1);
    let first = crate::time::add_months(last, -(n as i32 - 1));
    let from_month = crate::time::fmt_month(first.year(), first.month());
    let to_month = crate::time::fmt_month(last.year(), last.month());
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
    let total: i64 = summary.market_value;
    let mut notes = vec![t_args(
        "nucleo-carteira-risco-nota-retornos",
        &i18n::args(&[
            ("de", from_month.clone().into()),
            ("ate", to_month.clone().into()),
            ("moeda", summary.base_currency.clone().into()),
        ]),
    )];
    let perf = |asset_id: Option<String>| {
        super::engine::performance(
            conn,
            &PerformanceArgs {
                from_month: Some(from_month.clone()),
                to_month: Some(to_month.clone()),
                deflator: None,
                asset_id,
                class: None,
                // O risco da carteira é mensal, qualquer que seja o ajuste da rentabilidade.
                method: Some(super::TwrMethod::Monthly),
            },
        )
    };
    let whole = perf(None)?;
    let months = whole.months.clone();
    let cdi = monthly_from_cumulative(&whole.cdi_cumulative);
    let realized = metrics(&months, &whole.monthly, &cdi);

    // Ativos de hoje, com o retorno mensal de cada um.
    struct Row {
        id: String,
        ticker: Option<String>,
        name: String,
        class: AssetClass,
        weight: f64,
        series: Vec<Option<f64>>,
    }
    let mut rows: Vec<Row> = vec![];
    for p in &summary.positions {
        let Some(v) = p.market_value_base.filter(|v| *v > 0) else {
            continue;
        };
        if rows.iter().any(|r| r.id == p.asset.id) {
            continue;
        }
        let series = perf(Some(p.asset.id.clone()))?.monthly;
        rows.push(Row {
            id: p.asset.id.clone(),
            ticker: p.asset.ticker.clone(),
            name: p.asset.name.clone(),
            class: p.asset.class,
            weight: if total > 0 {
                v as f64 / total as f64
            } else {
                0.0
            },
            series,
        });
    }
    rows.sort_by(|a, b| b.weight.total_cmp(&a.weight));
    let len = months.len();
    let portfolio_monthly = weighted(
        &rows
            .iter()
            .map(|r| (r.series.as_slice(), r.weight))
            .collect::<Vec<_>>(),
        len,
    );
    let portfolio = metrics(&months, &portfolio_monthly, &cdi);

    // Classes.
    let mut classes: BTreeMap<AssetClass, Vec<&Row>> = BTreeMap::new();
    for r in &rows {
        classes.entry(r.class).or_default().push(r);
    }
    let mut class_series: Vec<(AssetClass, f64, Vec<Option<f64>>)> = classes
        .iter()
        .map(|(c, rs)| {
            let w: f64 = rs.iter().map(|r| r.weight).sum();
            let s = weighted(
                &rs.iter()
                    .map(|r| (r.series.as_slice(), r.weight))
                    .collect::<Vec<_>>(),
                len,
            );
            (*c, w, s)
        })
        .collect();
    class_series.sort_by(|a, b| b.1.total_cmp(&a.1));
    let by_class: Vec<ClassRisk> = class_series
        .iter()
        .map(|(c, w, s)| ClassRisk {
            class: *c,
            label: c.label(),
            weight: round(*w, 6),
            metrics: metrics(&months, s, &cdi),
            risk_share: risk_share(*w, s, &portfolio_monthly),
        })
        .collect();
    let class_correlation = matrix(
        &class_series
            .iter()
            .map(|(c, _, s)| (c.as_str().to_string(), c.label(), s.clone()))
            .collect::<Vec<_>>(),
    );
    let enough = |s: &[Option<f64>]| s.iter().flatten().count() >= MIN_OVERLAP;
    let short: Vec<String> = rows
        .iter()
        .filter(|r| !enough(&r.series))
        .map(|r| r.ticker.clone().unwrap_or_else(|| r.name.clone()))
        .collect();
    if !short.is_empty() {
        notes.push(t_args(
            "nucleo-carteira-risco-nota-pouco-historico",
            &i18n::args(&[
                ("minimo", (MIN_OVERLAP as i64).into()),
                ("ativos", short.join(", ").into()),
            ]),
        ));
    }
    let asset_correlation = matrix(
        &rows
            .iter()
            .filter(|r| enough(&r.series))
            .take(MAX_MATRIX_ASSETS)
            .map(|r| {
                (
                    r.id.clone(),
                    r.ticker.clone().unwrap_or_else(|| r.name.clone()),
                    r.series.clone(),
                )
            })
            .collect::<Vec<_>>(),
    );
    let assets = rows
        .iter()
        .map(|r| AssetRisk {
            asset_id: r.id.clone(),
            ticker: r.ticker.clone(),
            name: r.name.clone(),
            class: r.class,
            weight: round(r.weight, 6),
            metrics: if enough(&r.series) {
                metrics(&months, &r.series, &cdi)
            } else {
                None
            },
            risk_share: risk_share(r.weight, &r.series, &portfolio_monthly),
        })
        .collect();
    if cdi.iter().all(Option::is_none) {
        notes.push(t("nucleo-carteira-risco-nota-sem-cdi"));
    }
    if summary.estimated || whole.estimated {
        notes.push(t("nucleo-carteira-risco-nota-estimados"));
    }
    Ok(Risk {
        from_month,
        to_month,
        months,
        portfolio,
        realized,
        portfolio_monthly: portfolio_monthly
            .iter()
            .map(|v| v.map(|x| round(x, 6)))
            .collect(),
        cdi_monthly: cdi.iter().map(|v| v.map(|x| round(x, 6))).collect(),
        by_class,
        assets,
        class_correlation,
        asset_correlation,
        market_value: total,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("2025-{:02}", i % 12 + 1)).collect()
    }

    #[test]
    fn metrics_volatility_drawdown_and_sharpe() {
        // +10%, −20%, +5%, +10%: topo no 1º mês, fundo no 2º, volta no 4º (1,1 × 0,8 × 1,05
        // × 1,1 = 1,0164 < 1,1: não volta) → sem recuperação.
        let r = [Some(0.10), Some(-0.20), Some(0.05), Some(0.10)];
        let cdi = [Some(0.01); 4];
        let m = metrics(&labels(4), &r, &cdi).unwrap();
        assert_eq!(m.months, 4);
        assert!((m.max_drawdown - -0.20).abs() < 1e-9, "{m:?}");
        assert_eq!(m.drawdown_peak.as_deref(), Some("2025-01"));
        assert_eq!(m.drawdown_trough.as_deref(), Some("2025-02"));
        assert_eq!(m.drawdown_recovered, None);
        let sd = std(&[0.10, -0.20, 0.05, 0.10]);
        assert!((m.volatility - sd * 12f64.sqrt()).abs() < 1e-6);
        let excess = mean(&[0.09, -0.21, 0.04, 0.09]);
        assert!((m.sharpe.unwrap() - excess / sd * 12f64.sqrt()).abs() < 1e-3);
        assert!((m.cdi_annual.unwrap() - (1.01f64.powi(12) - 1.0)).abs() < 1e-6);
        assert_eq!(m.worst_month, Some(-0.2));
        // Recupera quando passa do topo.
        let r = [Some(0.10), Some(-0.10), Some(0.20)];
        let m = metrics(&labels(3), &r, &[None; 3]).unwrap();
        assert_eq!(m.drawdown_recovered.as_deref(), Some("2025-03"));
        assert_eq!(m.sharpe, None, "sem CDI");
        // Queda logo no 1º mês: o topo é o início.
        let m = metrics(&labels(2), &[Some(-0.05), Some(0.01)], &[None; 2]).unwrap();
        assert_eq!(m.drawdown_peak.as_deref(), Some("início"));
        assert_eq!(m.drawdown_recovered, None);
        // E volta quando passa do nível inicial (antes, a recuperação nunca aparecia).
        let m = metrics(
            &labels(3),
            &[Some(-0.05), Some(0.01), Some(0.06)],
            &[None; 3],
        )
        .unwrap();
        assert_eq!(m.drawdown_peak.as_deref(), Some("início"));
        assert_eq!(m.drawdown_recovered.as_deref(), Some("2025-03"));
    }

    #[test]
    fn correlation_needs_overlap_and_detects_sign() {
        let a: Vec<Option<f64>> = (0..14).map(|i| Some((i as f64).sin())).collect();
        let b: Vec<Option<f64>> = a.iter().map(|x| x.map(|v| 2.0 * v + 0.01)).collect();
        let c: Vec<Option<f64>> = a.iter().map(|x| x.map(|v| -v)).collect();
        assert_eq!(correlation(&a, &b), Some(1.0));
        assert_eq!(correlation(&a, &c), Some(-1.0));
        let mut short = a.clone();
        for v in short.iter_mut().take(4) {
            *v = None;
        }
        assert_eq!(correlation(&short, &b), None, "só 10 meses em comum");
    }

    #[test]
    fn weights_move_to_assets_that_existed_and_shares_sum_to_one() {
        let a = vec![Some(0.02), Some(0.01), Some(-0.01)];
        let b = vec![None, Some(0.03), Some(0.05)];
        let p = weighted(&[(&a, 0.5), (&b, 0.5)], 3);
        assert_eq!(p[0], Some(0.02));
        assert!((p[1].unwrap() - 0.02).abs() < 1e-12);
        // Partes do risco somam 1 quando as séries cobrem os mesmos meses.
        let x: Vec<Option<f64>> = (0..20)
            .map(|i| Some(((i * 7) % 5) as f64 / 100.0))
            .collect();
        let y: Vec<Option<f64>> = (0..20).map(|i| Some(((i * 3) % 4) as f64 / 50.0)).collect();
        let port = weighted(&[(&x, 0.3), (&y, 0.7)], 20);
        let s = risk_share(0.3, &x, &port).unwrap() + risk_share(0.7, &y, &port).unwrap();
        assert!((s - 1.0).abs() < 1e-3, "{s}");
    }

    #[test]
    fn cdi_monthly_from_cumulative() {
        let m = monthly_from_cumulative(&[Some(0.01), Some(0.0201), None, Some(0.04)]);
        assert!((m[0].unwrap() - 0.01).abs() < 1e-12);
        assert!((m[1].unwrap() - 0.01).abs() < 1e-9);
        assert_eq!(m[2], None);
        assert_eq!(m[3], None);
    }
}
