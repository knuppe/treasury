//! Normalização (§7.3): âncora absoluta por trechos lineares, winsorização, percentil
//! contra os pares, z-score robusto e as estatísticas de série (média, desvio, inclinação).
//! Funções puras.

/// Nota 0–100 de `x` na âncora (pontos `(x, nota)` em ordem crescente de x; fora da
/// tabela vale o extremo).
pub fn interp(anchor: &[(f64, f64)], x: f64) -> f64 {
    let Some(first) = anchor.first() else {
        return 50.0;
    };
    if x <= first.0 {
        return first.1;
    }
    for w in anchor.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if x <= x1 {
            if x1 == x0 {
                return y1;
            }
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0);
        }
    }
    anchor.last().map(|p| p.1).unwrap_or(50.0)
}

/// Quantil (interpolação linear entre as ordens) de valores já ordenados.
pub fn quantile(sorted: &[f64], q: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let q = q.clamp(0.0, 1.0);
    let pos = q * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    Some(sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64))
}

/// Valores finitos ordenados.
pub fn sorted(values: impl IntoIterator<Item = f64>) -> Vec<f64> {
    let mut v: Vec<f64> = values.into_iter().filter(|x| x.is_finite()).collect();
    v.sort_by(f64::total_cmp);
    v
}

/// Winsoriza os pares nos quantis `low`/`high` (devolve os valores cortados, ordenados, e
/// os limites).
pub fn winsorize(sorted_peers: &[f64], low: f64, high: f64) -> (Vec<f64>, f64, f64) {
    let (Some(lo), Some(hi)) = (quantile(sorted_peers, low), quantile(sorted_peers, high)) else {
        return (vec![], f64::NAN, f64::NAN);
    };
    (
        sorted_peers.iter().map(|x| x.clamp(lo, hi)).collect(),
        lo,
        hi,
    )
}

/// Percentil (0–100) de `x` entre os pares: fração dos pares abaixo + metade dos empates
/// (rank médio), robusto a outliers e legível ("melhor que 80% dos pares").
pub fn percentile_rank(sorted_peers: &[f64], x: f64) -> Option<f64> {
    if sorted_peers.is_empty() || !x.is_finite() {
        return None;
    }
    let below = sorted_peers.partition_point(|v| *v < x);
    let upto = sorted_peers.partition_point(|v| *v <= x);
    let ties = upto - below;
    Some(100.0 * (below as f64 + 0.5 * ties as f64) / sorted_peers.len() as f64)
}

/// Mediana de valores ordenados.
pub fn median(sorted: &[f64]) -> Option<f64> {
    quantile(sorted, 0.5)
}

/// Z-score robusto `(x − mediana) / (1,4826 · MAD)` convertido para 0–100 por `100·Φ(z)`
/// (alternativa ao percentil, §7.3 item 4).
pub fn robust_z_score(sorted_peers: &[f64], x: f64) -> Option<f64> {
    let med = median(sorted_peers)?;
    let dev = sorted(sorted_peers.iter().map(|v| (v - med).abs()));
    let mad = median(&dev)? * 1.4826;
    if mad <= 0.0 || !x.is_finite() {
        return None;
    }
    Some(100.0 * phi((x - med) / mad))
}

/// Distribuição normal acumulada (aproximação de Abramowitz e Stegun 7.1.26).
pub fn phi(z: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * z.abs() / std::f64::consts::SQRT_2);
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-(z * z) / 2.0).exp();
    if z >= 0.0 {
        0.5 * (1.0 + y)
    } else {
        0.5 * (1.0 - y)
    }
}

pub fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// Desvio-padrão amostral (n − 1).
pub fn stdev(v: &[f64]) -> Option<f64> {
    if v.len() < 2 {
        return None;
    }
    let m = mean(v)?;
    Some((v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt())
}

/// Inclinação da reta de mínimos quadrados de `(x, y)` (unidades de y por unidade de x).
pub fn slope(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() < 3 {
        return None;
    }
    let n = points.len() as f64;
    let mx = points.iter().map(|p| p.0).sum::<f64>() / n;
    let my = points.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|p| (p.0 - mx).powi(2)).sum();
    if sxx == 0.0 {
        return None;
    }
    let sxy: f64 = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    Some(sxy / sxx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interp_clamps_and_interpolates() {
        let a = [(1.5, 0.0), (8.0, 100.0)];
        assert_eq!(interp(&a, 0.0), 0.0);
        assert_eq!(interp(&a, 1.5), 0.0);
        assert!((interp(&a, 4.75) - 50.0).abs() < 1e-9);
        assert_eq!(interp(&a, 20.0), 100.0);
        // Faixa ótima: sobe, platô, desce.
        let r = [(0.0, 20.0), (0.25, 100.0), (0.75, 100.0), (1.0, 50.0)];
        assert_eq!(interp(&r, 0.5), 100.0);
        assert!((interp(&r, 0.875) - 75.0).abs() < 1e-9);
    }

    #[test]
    fn percentile_uses_mid_rank() {
        let p = sorted([1.0, 2.0, 3.0, 4.0]);
        assert_eq!(percentile_rank(&p, 0.5), Some(0.0));
        assert_eq!(percentile_rank(&p, 2.0), Some(37.5));
        assert_eq!(percentile_rank(&p, 2.5), Some(50.0));
        assert_eq!(percentile_rank(&p, 10.0), Some(100.0));
        assert_eq!(percentile_rank(&[], 1.0), None);
    }

    #[test]
    fn winsorize_cuts_the_tails() {
        let p = sorted((1..=41).map(f64::from).chain([1000.0]));
        let (w, lo, hi) = winsorize(&p, 0.025, 0.975);
        assert!(lo > 1.0 && hi < 1000.0);
        assert_eq!(*w.last().unwrap(), hi);
        assert_eq!(w.len(), p.len());
    }

    #[test]
    fn robust_z_and_phi() {
        assert!((phi(0.0) - 0.5).abs() < 1e-7);
        assert!((phi(1.96) - 0.975).abs() < 1e-3);
        let p = sorted([1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((robust_z_score(&p, 3.0).unwrap() - 50.0).abs() < 1e-6);
        assert!(robust_z_score(&p, 5.0).unwrap() > 80.0);
    }

    #[test]
    fn series_stats() {
        assert_eq!(mean(&[1.0, 2.0, 3.0]), Some(2.0));
        assert!((stdev(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap() - 2.138).abs() < 1e-3);
        let s = slope(&[(0.0, 1.0), (1.0, 3.0), (2.0, 5.0)]).unwrap();
        assert!((s - 2.0).abs() < 1e-12);
        assert_eq!(slope(&[(0.0, 1.0), (1.0, 2.0)]), None);
    }
}
