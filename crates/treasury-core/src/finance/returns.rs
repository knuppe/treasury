//! Retorno ponderado pelo dinheiro (XIRR) e pelo tempo (TWR).

use chrono::NaiveDate;

/// Um fluxo: negativo = aporte (saiu do bolso), positivo = resgate/valor final.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flow {
    pub date: NaiveDate,
    pub amount: f64,
}

fn npv(rate: f64, flows: &[Flow], t0: NaiveDate) -> f64 {
    flows
        .iter()
        .map(|f| f.amount / (1.0 + rate).powf((f.date - t0).num_days() as f64 / 365.0))
        .sum()
}

fn dnpv(rate: f64, flows: &[Flow], t0: NaiveDate) -> f64 {
    flows
        .iter()
        .map(|f| {
            let t = (f.date - t0).num_days() as f64 / 365.0;
            -t * f.amount / (1.0 + rate).powf(t + 1.0)
        })
        .sum()
}

/// XIRR (convenção do Excel: base 365 dias). `None` se não houver fluxos dos dois sinais
/// ou se não convergir.
pub fn xirr(flows: &[Flow]) -> Option<f64> {
    if flows.len() < 2
        || !flows.iter().any(|f| f.amount < 0.0)
        || !flows.iter().any(|f| f.amount > 0.0)
    {
        return None;
    }
    let t0 = flows.iter().map(|f| f.date).min()?;
    // Newton-Raphson a partir de alguns chutes.
    for guess in [0.1, 0.0, -0.5, 1.0, 5.0] {
        let mut r = guess;
        for _ in 0..100 {
            let v = npv(r, flows, t0);
            let d = dnpv(r, flows, t0);
            if d.abs() < 1e-12 {
                break;
            }
            let next = r - v / d;
            if !next.is_finite() || next <= -0.999_999 {
                break;
            }
            if (next - r).abs() < 1e-10 {
                if npv(next, flows, t0).abs()
                    < 1e-6 * flows.iter().map(|f| f.amount.abs()).sum::<f64>().max(1.0)
                {
                    return Some(next);
                }
                break;
            }
            r = next;
        }
    }
    // Bisseção em [-0.9999, 100].
    let (mut lo, mut hi) = (-0.9999_f64, 100.0_f64);
    let (mut flo, fhi) = (npv(lo, flows, t0), npv(hi, flows, t0));
    if flo.signum() == fhi.signum() {
        return None;
    }
    for _ in 0..300 {
        let mid = (lo + hi) / 2.0;
        let fm = npv(mid, flows, t0);
        if fm.abs() < 1e-9 || (hi - lo) < 1e-12 {
            return Some(mid);
        }
        if fm.signum() == flo.signum() {
            lo = mid;
            flo = fm;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Um dia da carteira para o TWR: valor no fim do dia e fluxo líquido externo do dia
/// (aporte positivo, resgate negativo), considerado no início do dia.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayValue {
    pub date: NaiveDate,
    pub value_end: f64,
    pub net_flow: f64,
}

/// TWR encadeando sub-períodos: Π (V_t ÷ (V_{t−1} + F_t)) − 1. Dias com base zero são
/// ignorados (carteira vazia). `None` sem pelo menos um sub-período válido.
pub fn twr(days: &[DayValue]) -> Option<f64> {
    let mut acc = 1.0;
    let mut prev: Option<f64> = None;
    let mut any = false;
    for d in days {
        let base = prev.unwrap_or(0.0) + d.net_flow;
        if base.abs() > 1e-9 {
            acc *= d.value_end / base;
            any = true;
        }
        prev = Some(d.value_end);
    }
    any.then_some(acc - 1.0)
}

/// Anualiza um retorno acumulado num período de `days` dias corridos.
pub fn annualize(total_return: f64, days: i64) -> Option<f64> {
    if days <= 0 || total_return <= -1.0 {
        return None;
    }
    Some((1.0 + total_return).powf(365.0 / days as f64) - 1.0)
}

/// Retorno real: (1 + nominal) ÷ (1 + inflação) − 1.
pub fn real_return(nominal: f64, inflation: f64) -> f64 {
    (1.0 + nominal) / (1.0 + inflation) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn xirr_matches_excel_reference() {
        // Exemplo da documentação do Excel (XIRR = 0,373362535).
        let flows = [
            Flow {
                date: d("2008-01-01"),
                amount: -10000.0,
            },
            Flow {
                date: d("2008-03-01"),
                amount: 2750.0,
            },
            Flow {
                date: d("2008-10-30"),
                amount: 4250.0,
            },
            Flow {
                date: d("2009-02-15"),
                amount: 3250.0,
            },
            Flow {
                date: d("2009-04-01"),
                amount: 2750.0,
            },
        ];
        let r = xirr(&flows).unwrap();
        assert!((r - 0.373362535).abs() < 1e-6, "{r}");
    }

    #[test]
    fn xirr_simple_and_negative() {
        // 1000 → 1100 em exatamente 365 dias = 10% a.a.
        let r = xirr(&[
            Flow {
                date: d("2025-01-01"),
                amount: -1000.0,
            },
            Flow {
                date: d("2026-01-01"),
                amount: 1100.0,
            },
        ])
        .unwrap();
        assert!((r - 0.10).abs() < 1e-9);
        let r = xirr(&[
            Flow {
                date: d("2025-01-01"),
                amount: -1000.0,
            },
            Flow {
                date: d("2026-01-01"),
                amount: 800.0,
            },
        ])
        .unwrap();
        assert!((r + 0.20).abs() < 1e-9);
        assert!(
            xirr(&[Flow {
                date: d("2025-01-01"),
                amount: -1000.0
            }])
            .is_none()
        );
        assert!(
            xirr(&[
                Flow {
                    date: d("2025-01-01"),
                    amount: 10.0
                },
                Flow {
                    date: d("2025-02-01"),
                    amount: 10.0
                }
            ])
            .is_none()
        );
    }

    #[test]
    fn twr_neutralizes_flows() {
        // Começa com 1000, sobe 10%; aporte de 1000; cai 10%.
        // TWR = 1,1 × 0,9 − 1 = −1%, independentemente do aporte.
        let days = [
            DayValue {
                date: d("2026-01-01"),
                value_end: 1000.0,
                net_flow: 1000.0,
            },
            DayValue {
                date: d("2026-01-02"),
                value_end: 1100.0,
                net_flow: 0.0,
            },
            DayValue {
                date: d("2026-01-03"),
                value_end: 1890.0,
                net_flow: 1000.0,
            },
        ];
        let r = twr(&days).unwrap();
        assert!((r - (1.1 * 0.9 - 1.0)).abs() < 1e-12, "{r}");
        assert!(twr(&[]).is_none());
    }

    #[test]
    fn real_and_annualized() {
        assert!((real_return(0.10, 0.05) - 0.047619047).abs() < 1e-8);
        assert!((annualize(0.21, 730).unwrap() - 0.1).abs() < 1e-3);
        assert!(annualize(0.1, 0).is_none());
    }
}
