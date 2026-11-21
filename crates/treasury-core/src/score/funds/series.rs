//! Estatísticas de séries de preços (cotas de fundos, ETFs, cripto): retorno anualizado,
//! volatilidade, Sharpe, Sortino, drawdown e recuperação, consistência em janelas de 12
//! meses, tracking contra um índice e desvio de paridade. Funções puras sobre pontos
//! `(data, nível)` já ordenados; f64 só para razões.

use chrono::NaiveDate;

use crate::score::normalize::{mean, stdev};
use crate::time::add_months;

/// Ponto de uma série (data, nível).
pub type Point = (NaiveDate, f64);

/// Dias por ano usados na anualização pelo calendário.
const YEAR_DAYS: f64 = 365.25;

/// Pontos com nível positivo dentro de [from, to].
pub fn window(pts: &[Point], from: NaiveDate, to: NaiveDate) -> Vec<Point> {
    pts.iter()
        .filter(|p| p.0 >= from && p.0 <= to && p.1 > 0.0 && p.1.is_finite())
        .copied()
        .collect()
}

/// Último nível em ou antes de `d`.
pub fn level_at(pts: &[Point], d: NaiveDate) -> Option<f64> {
    let i = pts.partition_point(|p| p.0 <= d);
    (i > 0).then(|| pts[i - 1].1)
}

/// Anos entre o primeiro e o último ponto.
pub fn span_years(pts: &[Point]) -> f64 {
    match (pts.first(), pts.last()) {
        (Some(a), Some(b)) => (b.0 - a.0).num_days() as f64 / YEAR_DAYS,
        _ => 0.0,
    }
}

/// Retorno total entre o primeiro e o último ponto.
pub fn total_return(pts: &[Point]) -> Option<f64> {
    let (a, b) = (pts.first()?, pts.last()?);
    (a.1 > 0.0 && pts.len() >= 2).then(|| b.1 / a.1 - 1.0)
}

/// Retorno anualizado (geométrico, pelo calendário).
pub fn annualized(total: f64, years: f64) -> Option<f64> {
    (years > 0.0 && total > -1.0).then(|| (1.0 + total).powf(1.0 / years) - 1.0)
}

/// Retornos simples entre pontos consecutivos.
pub fn returns(pts: &[Point]) -> Vec<f64> {
    pts.windows(2)
        .filter(|w| w[0].1 > 0.0)
        .map(|w| w[1].1 / w[0].1 - 1.0)
        .collect()
}

/// Observações por ano (≈ 252 nos dias úteis, ≈ 365 em cripto).
pub fn periods_per_year(pts: &[Point]) -> Option<f64> {
    let y = span_years(pts);
    (y > 0.0 && pts.len() >= 2).then(|| (pts.len() - 1) as f64 / y)
}

/// Volatilidade anualizada: desvio-padrão dos retornos × √(observações por ano).
pub fn volatility(pts: &[Point]) -> Option<f64> {
    let r = returns(pts);
    Some(stdev(&r)? * periods_per_year(pts)?.sqrt())
}

/// Um ponto a cada `k`, terminando no último (retornos semanais a partir dos diários, menos
/// sensíveis a fechamentos em horários diferentes).
pub fn sample_every(pts: &[Point], k: usize) -> Vec<Point> {
    let k = k.max(1);
    let n = pts.len();
    pts.iter()
        .enumerate()
        .filter(|(i, _)| (n - 1 - i).is_multiple_of(k))
        .map(|(_, p)| *p)
        .collect()
}

/// Série de um benchmark nas datas dos pontos (último nível conhecido em cada data).
pub fn aligned(pts: &[Point], bench: &[Point]) -> Vec<(NaiveDate, f64, f64)> {
    pts.iter()
        .filter_map(|p| Some((p.0, p.1, level_at(bench, p.0)?)))
        .collect()
}

/// Resultado de risco e retorno numa janela.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskReturn {
    pub years: f64,
    /// Retorno anualizado do ativo e do livre de risco (ou benchmark) na janela.
    pub ret: f64,
    pub rf: f64,
    pub vol: f64,
    pub sharpe: Option<f64>,
    pub sortino: Option<f64>,
}

/// Sharpe e Sortino com o livre de risco dado como série de níveis (ex.: CDI acumulado).
/// Sharpe = (R − Rf) ÷ σ (anualizados). Sortino = (R − Rf) ÷ desvio para baixo, em que o
/// desvio para baixo é √(média de min(0, rᵢ − rfᵢ)²) × √(observações por ano), com rfᵢ o
/// retorno do livre de risco no mesmo intervalo.
pub fn risk_return(pts: &[Point], rf: &[Point]) -> Option<RiskReturn> {
    let al = aligned(pts, rf);
    if al.len() < 3 {
        return None;
    }
    let p: Vec<Point> = al.iter().map(|x| (x.0, x.1)).collect();
    let b: Vec<Point> = al.iter().map(|x| (x.0, x.2)).collect();
    let years = span_years(&p);
    let ret = annualized(total_return(&p)?, years)?;
    let rfa = annualized(total_return(&b)?, years)?;
    let vol = volatility(&p)?;
    let ppy = periods_per_year(&p)?;
    let excess: Vec<f64> = al
        .windows(2)
        .map(|w| (w[1].1 / w[0].1 - 1.0) - (w[1].2 / w[0].2 - 1.0))
        .collect();
    let down = (excess.iter().map(|e| e.min(0.0).powi(2)).sum::<f64>() / excess.len() as f64)
        .sqrt()
        * ppy.sqrt();
    let ratio = |den: f64| (den > 1e-12).then(|| (ret - rfa) / den);
    Some(RiskReturn {
        years,
        ret,
        rf: rfa,
        vol,
        sharpe: ratio(vol),
        sortino: ratio(down),
    })
}

/// Retorno acima do benchmark e tracking error (anualizados) numa janela.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Relative {
    pub years: f64,
    /// Retorno total (não anualizado) do ativo e do benchmark na janela.
    pub total: f64,
    pub bench_total: f64,
    /// Diferença dos retornos anualizados.
    pub excess: f64,
    pub tracking_error: f64,
    pub information_ratio: Option<f64>,
}

pub fn relative(pts: &[Point], bench: &[Point]) -> Option<Relative> {
    let al = aligned(pts, bench);
    if al.len() < 3 {
        return None;
    }
    let p: Vec<Point> = al.iter().map(|x| (x.0, x.1)).collect();
    let b: Vec<Point> = al.iter().map(|x| (x.0, x.2)).collect();
    let years = span_years(&p);
    let (tp, tb) = (total_return(&p)?, total_return(&b)?);
    let excess = annualized(tp, years)? - annualized(tb, years)?;
    let diffs: Vec<f64> = al
        .windows(2)
        .map(|w| (w[1].1 / w[0].1) - (w[1].2 / w[0].2))
        .collect();
    let te = stdev(&diffs)? * periods_per_year(&p)?.sqrt();
    Some(Relative {
        years,
        total: tp,
        bench_total: tb,
        excess,
        tracking_error: te,
        information_ratio: (te > 1e-12).then(|| excess / te),
    })
}

/// Maior queda em relação ao pico anterior e quanto tempo levou para recuperar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drawdown {
    /// Queda como fração positiva (0,25 = −25%).
    pub depth: f64,
    pub peak: NaiveDate,
    pub trough: NaiveDate,
    /// Data em que o nível voltou ao pico (None = ainda não voltou).
    pub recovered: Option<NaiveDate>,
    /// Dias do fundo até a recuperação; sem recuperação, dias do pico até o último ponto.
    pub recovery_days: i64,
}

pub fn max_drawdown(pts: &[Point]) -> Option<Drawdown> {
    let first = pts.first()?;
    let mut peak = *first;
    let mut best = Drawdown {
        depth: 0.0,
        peak: first.0,
        trough: first.0,
        recovered: Some(first.0),
        recovery_days: 0,
    };
    for p in pts {
        if p.1 > peak.1 {
            peak = *p;
        }
        let dd = 1.0 - p.1 / peak.1;
        if dd > best.depth {
            best.depth = dd;
            best.peak = peak.0;
            best.trough = p.0;
        }
    }
    if best.depth > 0.0 {
        let peak_level = pts
            .iter()
            .find(|p| p.0 == best.peak)
            .map(|p| p.1)
            .unwrap_or(f64::INFINITY);
        best.recovered = pts
            .iter()
            .find(|p| p.0 > best.trough && p.1 >= peak_level)
            .map(|p| p.0);
        let last = pts.last()?.0;
        best.recovery_days = match best.recovered {
            Some(r) => (r - best.trough).num_days(),
            None => (last - best.peak).num_days(),
        };
    }
    Some(best)
}

/// Janelas móveis de 12 meses (passo mensal, terminando no último ponto e voltando de mês
/// em mês): (janelas, quantas o ativo superou o benchmark).
pub fn rolling_12m_wins(pts: &[Point], bench: &[Point]) -> (u32, u32) {
    let (Some(first), Some(last)) = (pts.first(), pts.last()) else {
        return (0, 0);
    };
    let (mut n, mut wins) = (0, 0);
    for k in 0.. {
        let end = add_months(last.0, -k);
        let start = add_months(last.0, -k - 12);
        if start < first.0 {
            break;
        }
        let r = |s: &[Point]| Some(level_at(s, end)? / level_at(s, start)? - 1.0);
        if let (Some(a), Some(b)) = (r(pts), r(bench)) {
            n += 1;
            if a > b {
                wins += 1;
            }
        }
    }
    (n, wins)
}

/// Paridade de uma stablecoin: (maior desvio, fração dos dias com desvio acima de
/// `threshold`, desvio médio), com `peg` = 1 para a paridade com o dólar.
pub fn parity(pts: &[Point], peg: f64, threshold: f64) -> Option<(f64, f64, f64)> {
    if pts.is_empty() || peg <= 0.0 {
        return None;
    }
    let devs: Vec<f64> = pts.iter().map(|p| (p.1 / peg - 1.0).abs()).collect();
    let max = devs.iter().copied().fold(0.0, f64::max);
    let off = devs.iter().filter(|d| **d > threshold).count() as f64 / devs.len() as f64;
    Some((max, off, mean(&devs)?))
}

/// Série de níveis a partir de taxas diárias em % (CDI): nível 1 antes da primeira data,
/// multiplicado por (1 + taxa/100) em cada dia.
pub fn level_from_daily_rates(rates: &[(NaiveDate, f64)]) -> Vec<Point> {
    let mut lvl = 1.0;
    let mut out = Vec::with_capacity(rates.len() + 1);
    if let Some(f) = rates.first() {
        out.push((f.0 - chrono::Duration::days(1), 1.0));
    }
    for (d, r) in rates {
        lvl *= 1.0 + r / 100.0;
        out.push((*d, lvl));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn drawdown_and_recovery_by_hand() {
        // 100 → 120 (pico) → 90 (−25%) → 110 → 121 (recupera em 05/01) → 100.
        let pts = vec![
            (d("2026-01-01"), 100.0),
            (d("2026-01-02"), 120.0),
            (d("2026-01-03"), 90.0),
            (d("2026-01-04"), 110.0),
            (d("2026-01-05"), 121.0),
            (d("2026-01-06"), 100.0),
        ];
        let dd = max_drawdown(&pts).unwrap();
        assert!(close(dd.depth, 0.25));
        assert_eq!(dd.peak, d("2026-01-02"));
        assert_eq!(dd.trough, d("2026-01-03"));
        assert_eq!(dd.recovered, Some(d("2026-01-05")));
        assert_eq!(dd.recovery_days, 2);
        // Sem recuperação: dias desde o pico.
        let dd = max_drawdown(&pts[..4]).unwrap();
        assert_eq!(dd.recovered, None);
        assert_eq!(dd.recovery_days, 2);
        // Série só subindo: sem queda.
        let up = vec![(d("2026-01-01"), 1.0), (d("2026-01-02"), 1.1)];
        assert_eq!(max_drawdown(&up).unwrap().depth, 0.0);
    }

    #[test]
    fn sharpe_and_sortino_by_hand() {
        // Um ano (365 dias) com 4 retornos: +10%, −5%, +10%, −5% → total
        // 1,1 · 0,95 · 1,1 · 0,95 = 1,092025.
        // Livre de risco constante (nível 1): Rf = 0.
        let dates = [
            "2025-01-01",
            "2025-04-02",
            "2025-07-02",
            "2025-10-01",
            "2026-01-01",
        ];
        let lv = [100.0, 110.0, 104.5, 114.95, 109.2025];
        let pts: Vec<Point> = dates.iter().zip(lv).map(|(s, v)| (d(s), v)).collect();
        let rf: Vec<Point> = dates.iter().map(|s| (d(s), 1.0)).collect();
        let r = risk_return(&pts, &rf).unwrap();
        let years = 365.0 / 365.25;
        let ret = 1.092025f64.powf(1.0 / years) - 1.0;
        assert!(close(r.ret, ret), "{}", r.ret);
        assert!(close(r.rf, 0.0));
        // Retornos 0,10; −0,05; 0,10; −0,05: média 0,025, desvio amostral √(4·0,075²/3).
        let sd = (4.0 * 0.075f64.powi(2) / 3.0).sqrt();
        let ppy = 4.0 / years;
        assert!(close(r.vol, sd * ppy.sqrt()), "{}", r.vol);
        assert!(close(r.sharpe.unwrap(), ret / (sd * ppy.sqrt())));
        // Desvio para baixo: √((0,05² + 0,05²)/4) × √ppy.
        let down = (2.0 * 0.0025f64 / 4.0).sqrt() * ppy.sqrt();
        assert!(close(r.sortino.unwrap(), ret / down));
    }

    #[test]
    fn relative_and_tracking() {
        let dates = ["2025-01-01", "2025-07-02", "2026-01-01"];
        let etf: Vec<Point> = dates
            .iter()
            .zip([100.0, 104.0, 109.0])
            .map(|(s, v)| (d(s), v))
            .collect();
        let idx: Vec<Point> = dates
            .iter()
            .zip([1000.0, 1050.0, 1100.0])
            .map(|(s, v)| (d(s), v))
            .collect();
        let r = relative(&etf, &idx).unwrap();
        assert!(close(r.total, 0.09));
        assert!(close(r.bench_total, 0.10));
        // Diferenças dos retornos: 0,04 − 0,05 = −0,01; 109/104 − 1100/1050.
        let d2 = 109.0 / 104.0 - 1100.0 / 1050.0;
        let te = stdev(&[-0.01, d2]).unwrap() * (2.0f64 / (365.0 / 365.25)).sqrt();
        assert!(close(r.tracking_error, te), "{}", r.tracking_error);
    }

    #[test]
    fn consistency_counts_monthly_windows() {
        // Fundo rende 1% ao mês (12 meses: +12,68%); benchmark 0,5% ao mês nos primeiros 12
        // meses e 3% depois.
        let mut f = vec![];
        let mut b = vec![];
        let (mut lf, mut lb) = (100.0, 100.0);
        for i in 0..=24 {
            let dt = add_months(d("2024-01-31"), i);
            f.push((dt, lf));
            b.push((dt, lb));
            lf *= 1.01;
            lb *= if i < 12 { 1.005 } else { 1.03 };
        }
        // Janelas terminando de 01/2026 a 01/2025: 13. O fundo vence as janelas com até dois
        // meses a 3% (1,005¹⁰ · 1,03² = 1,1152 < 1,1268; com três, 1,1429): 01, 02 e 03/2025.
        let (n, wins) = rolling_12m_wins(&f, &b);
        assert_eq!(n, 13);
        assert_eq!(wins, 3);
    }

    #[test]
    fn parity_deviation() {
        let pts: Vec<Point> = [1.0, 0.999, 1.002, 0.97, 1.0]
            .iter()
            .enumerate()
            .map(|(i, v)| (d("2026-01-01") + chrono::Duration::days(i as i64), *v))
            .collect();
        let (max, off, avg) = parity(&pts, 1.0, 0.005).unwrap();
        assert!(close(max, 0.03));
        assert!(close(off, 0.2));
        assert!(close(avg, (0.001 + 0.002 + 0.03) / 5.0));
    }

    #[test]
    fn sampling_keeps_the_last_point() {
        let pts: Vec<Point> = (0..11)
            .map(|i| (d("2026-01-01") + chrono::Duration::days(i), i as f64 + 1.0))
            .collect();
        let w = sample_every(&pts, 5);
        assert_eq!(
            w.iter().map(|p| p.1).collect::<Vec<_>>(),
            vec![1.0, 6.0, 11.0]
        );
    }

    #[test]
    fn cdi_levels_compound_daily() {
        let r = vec![(d("2026-01-02"), 0.05), (d("2026-01-05"), 0.05)];
        let l = level_from_daily_rates(&r);
        assert_eq!(l[0], (d("2026-01-01"), 1.0));
        assert!(close(l[2].1, 1.0005 * 1.0005));
        assert!(close(level_at(&l, d("2026-01-03")).unwrap(), 1.0005));
    }
}
