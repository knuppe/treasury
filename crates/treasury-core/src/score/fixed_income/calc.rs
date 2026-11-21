//! Cálculos puros da renda fixa: prazo e duration, liquidez, volatilidade, curva do Tesouro,
//! taxa em % do CDI, spread, cobertura do FGC e nome do emissor para comparar. Taxas em
//! fração ao ano (0,0684 = 6,84% a.a.); dinheiro em `Decimal`, `f64` só para razões.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

use crate::assets::Liquidity;
use crate::research::to_f64;

/// Dias úteis por ano (convenção do CDI e do Tesouro).
const BUSINESS_DAYS: f64 = 252.0;
const DAYS_PER_YEAR: f64 = 365.25;

/// Anos entre duas datas (dias corridos ÷ 365,25).
pub fn years_between(from: NaiveDate, to: NaiveDate) -> f64 {
    (to - from).num_days() as f64 / DAYS_PER_YEAR
}

/// Duration de Macaulay (anos) de fluxos (data, valor) descontados à taxa anual `y`; só os
/// fluxos depois de `today`.
pub fn macaulay(today: NaiveDate, flows: &[(NaiveDate, f64)], y: f64) -> Option<f64> {
    let (mut pv, mut tpv) = (0.0, 0.0);
    for (d, cf) in flows.iter().filter(|(d, _)| *d > today) {
        let t = years_between(today, *d);
        let v = cf / (1.0 + y).powf(t);
        pv += v;
        tpv += t * v;
    }
    (pv > 0.0).then(|| tpv / pv)
}

/// Mesmo dia `months` meses antes (último dia do mês quando não existe).
fn months_before(d: NaiveDate, months: i32) -> NaiveDate {
    crate::time::add_months(d, -months)
}

/// Título com cupom semestral (`coupon` anual, ex.: 6% no IPCA+ com juros) até o vencimento.
pub fn coupon_duration(today: NaiveDate, maturity: NaiveDate, coupon: f64, y: f64) -> Option<f64> {
    let c = (1.0 + coupon).sqrt() - 1.0;
    let mut flows = vec![(maturity, 1.0 + c)];
    let mut k = 1;
    loop {
        let d = months_before(maturity, 6 * k);
        if d <= today {
            break;
        }
        flows.push((d, c));
        k += 1;
    }
    macaulay(today, &flows, y)
}

/// Título que paga `n` parcelas mensais iguais terminando no vencimento (Renda+, Educa+).
pub fn annuity_duration(today: NaiveDate, maturity: NaiveDate, n: u32, y: f64) -> Option<f64> {
    let flows: Vec<(NaiveDate, f64)> = (0..n as i32)
        .map(|k| (months_before(maturity, k), 1.0))
        .collect();
    macaulay(today, &flows, y)
}

/// Dias até o dinheiro estar disponível: liquidez diária = 1 dia (liquidação); carência e
/// vencimento contam os dias que faltam.
pub fn liquidity_days(
    liquidity: Liquidity,
    grace: Option<NaiveDate>,
    maturity: Option<NaiveDate>,
    today: NaiveDate,
) -> Option<f64> {
    let until = |d: NaiveDate| ((d - today).num_days().max(0) as f64).max(1.0);
    match liquidity {
        Liquidity::Daily => Some(1.0),
        Liquidity::AfterGrace => match grace {
            Some(g) => Some(until(g)),
            None => maturity.map(until),
        },
        Liquidity::AtMaturity => maturity.map(until),
    }
}

/// Volatilidade anualizada dos retornos logarítmicos diários (pelo menos 20 retornos).
pub fn annualized_volatility(prices: &[f64]) -> Option<f64> {
    let r: Vec<f64> = prices
        .windows(2)
        .filter(|w| w[0] > 0.0 && w[1] > 0.0)
        .map(|w| (w[1] / w[0]).ln())
        .collect();
    if r.len() < 20 {
        return None;
    }
    let n = r.len() as f64;
    let mean = r.iter().sum::<f64>() / n;
    let var = r.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    Some(var.sqrt() * BUSINESS_DAYS.sqrt())
}

/// Taxa da curva no prazo (anos), interpolada linearmente entre os vértices e constante
/// fora deles. `points` em qualquer ordem.
pub fn curve_rate(points: &[(f64, f64)], years: f64) -> Option<f64> {
    let mut p: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|(t, r)| t.is_finite() && r.is_finite())
        .collect();
    p.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (first, last) = (p.first()?, p.last()?);
    if years <= first.0 {
        return Some(first.1);
    }
    if years >= last.0 {
        return Some(last.1);
    }
    let w = p.windows(2).find(|w| years <= w[1].0)?;
    let ((x0, y0), (x1, y1)) = (w[0], w[1]);
    Some(if x1 == x0 {
        y1
    } else {
        y0 + (y1 - y0) * (years - x0) / (x1 - x0)
    })
}

/// Taxa diária equivalente (252 dias úteis).
fn daily(annual: f64) -> f64 {
    (1.0 + annual).powf(1.0 / BUSINESS_DAYS) - 1.0
}

/// Taxa anual expressa em % do CDI (razão das taxas diárias): 1,10 = 110% do CDI.
pub fn pct_of(annual: f64, reference: f64) -> Option<f64> {
    let r = daily(reference);
    (r > 0.0).then(|| daily(annual) / r)
}

/// Equivalente tributado de um rendimento isento: divide por (1 − IR).
pub fn gross_up(x: f64, ir: f64) -> f64 {
    x / (1.0 - ir)
}

/// IPCA+ em nominal com a inflação implícita entre o prefixado e o IPCA+ do mesmo prazo:
/// (1 + real) × (1 + pré) ÷ (1 + real do Tesouro) − 1.
pub fn real_to_nominal(real: f64, pre_ref: f64, real_ref: f64) -> f64 {
    (1.0 + real) * (1.0 + pre_ref) / (1.0 + real_ref) - 1.0
}

/// Juro real: (1 + nominal) ÷ (1 + inflação) − 1.
pub fn real_rate(nominal: f64, inflation: f64) -> f64 {
    (1.0 + nominal) / (1.0 + inflation) - 1.0
}

/// Remuneração anual da poupança (regra de 2012): Selic meta acima do limite → 0,5% a.m. +
/// TR; senão 70% da Selic + TR.
pub fn savings_yield(selic: f64, tr_12m: f64, threshold: f64, monthly: f64, share: f64) -> f64 {
    let base = if selic > threshold {
        (1.0 + monthly).powi(12) - 1.0
    } else {
        share * selic
    };
    (1.0 + base) * (1.0 + tr_12m) - 1.0
}

/// Cobertura do FGC de uma posição.
#[derive(Debug, Clone, PartialEq)]
pub struct FgcOut {
    /// Parte coberta (0–1), ponderada pelo valor da posição de cada titular.
    pub coverage: f64,
    /// Quanto a exposição dos titulares à instituição passa do limite por CPF.
    pub excess: Decimal,
    /// Maior exposição de um titular à instituição.
    pub max_exposure: Decimal,
}

/// Cobertura: para cada titular, min(1, limite ÷ exposição na instituição) × min(1, teto
/// global ÷ soma do que está coberto em todas as instituições), ponderada pela parte da
/// posição que é dele. Sem posição, a cobertura é total (dentro do limite).
pub fn fgc_coverage(
    position: &BTreeMap<String, Decimal>,
    at_institution: &BTreeMap<String, Decimal>,
    covered_total: &BTreeMap<String, Decimal>,
    limit: Decimal,
    global: Decimal,
) -> FgcOut {
    let (mut acc, mut total) = (0.0, 0.0);
    let mut excess = Decimal::ZERO;
    let mut max_exposure = Decimal::ZERO;
    for (who, v) in position {
        let exp = at_institution.get(who).copied().unwrap_or(*v).max(*v);
        max_exposure = max_exposure.max(exp);
        let inst = if exp > limit {
            excess += exp - limit;
            to_f64(limit / exp)
        } else {
            1.0
        };
        let all = covered_total.get(who).copied().unwrap_or(Decimal::ZERO);
        let glob = if all > global {
            to_f64(global / all)
        } else {
            1.0
        };
        let w = to_f64(*v);
        acc += w * inst * glob;
        total += w;
    }
    FgcOut {
        coverage: if total > 0.0 { acc / total } else { 1.0 },
        excess,
        max_exposure,
    }
}

/// Nome do emissor para comparar: normalizado, sem palavras genéricas ("banco", "S.A.",
/// "prudencial"…). Comparação conservadora: só nomes que ficam iguais.
pub fn core_name(s: &str, stopwords: &[String]) -> String {
    crate::research::norm(s)
        .split_whitespace()
        .filter(|w| !stopwords.iter().any(|x| x == w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Ano (4 dígitos) no fim de um nome ("Tesouro IPCA+ 2035").
pub fn trailing_year(s: &str) -> Option<i32> {
    s.split(|c: char| !c.is_ascii_digit())
        .filter(|w| w.len() == 4)
        .filter_map(|w| w.parse::<i32>().ok())
        .rfind(|y| (1990..=2100).contains(y))
}

/// Mês de referência (ano, mês) de uma data.
pub fn ym(d: NaiveDate) -> (i32, u32) {
    (d.year(), d.month())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    #[test]
    fn durations() {
        let today = d("2026-10-16");
        // Sem cupom: o prazo.
        let flows = [(d("2031-10-16"), 1.0)];
        let z = macaulay(today, &flows, 0.07).unwrap();
        assert!((z - years_between(today, d("2031-10-16"))).abs() < 1e-12);
        assert!((z - 5.0).abs() < 0.01);
        // Dois fluxos iguais em 1 e 2 anos a 0%: duration 1,5.
        let two = [(d("2027-10-16"), 1.0), (d("2028-10-16"), 1.0)];
        assert!((macaulay(today, &two, 0.0).unwrap() - 1.5).abs() < 0.01);
        // IPCA+ com juros 2035 (cupom 6% a.a., 18 pagamentos semestrais de 15/11/2026 a
        // 15/05/2035) a 6,82%: 6,6237 anos, conferido à parte (Python) com o mesmo fluxo.
        let c = coupon_duration(today, d("2035-05-15"), 0.06, 0.0682).unwrap();
        assert!((c - 6.623_676_810).abs() < 1e-6, "{c}");
        // Renda+ 2049: 240 parcelas mensais de jan/2030 a dez/2049 a 6,77% → 11,0823 anos.
        let r = annuity_duration(today, d("2049-12-15"), 240, 0.0677).unwrap();
        assert!((r - 11.082_347_486).abs() < 1e-6, "{r}");
        // Fluxo mensal em 12 meses a 0%: média simples dos prazos.
        let a = annuity_duration(d("2026-12-01"), d("2027-12-15"), 12, 0.0).unwrap();
        let avg = (0..12)
            .map(|k| years_between(d("2026-12-01"), months_before(d("2027-12-15"), k)))
            .sum::<f64>()
            / 12.0;
        assert!((a - avg).abs() < 1e-9);
    }

    #[test]
    fn liquidity_volatility_and_curve() {
        let today = d("2026-10-16");
        assert_eq!(
            liquidity_days(Liquidity::Daily, None, Some(d("2030-01-01")), today),
            Some(1.0)
        );
        assert_eq!(
            liquidity_days(Liquidity::AtMaturity, None, Some(d("2026-11-15")), today),
            Some(30.0)
        );
        assert_eq!(
            liquidity_days(
                Liquidity::AfterGrace,
                Some(d("2026-10-26")),
                Some(d("2030-01-01")),
                today
            ),
            Some(10.0)
        );
        assert_eq!(
            liquidity_days(Liquidity::AtMaturity, None, None, today),
            None
        );
        // Vencido: dinheiro disponível (1 dia).
        assert_eq!(
            liquidity_days(Liquidity::AtMaturity, None, Some(d("2026-01-01")), today),
            Some(1.0)
        );
        // Preço alternando +1%/−1%: retornos ±ln(1,01) → desvio ≈ 0,00995 × √252.
        let mut p = vec![100.0];
        for i in 0..40 {
            let last = *p.last().unwrap();
            p.push(if i % 2 == 0 { last * 1.01 } else { last / 1.01 });
        }
        let v = annualized_volatility(&p).unwrap();
        let expected = (1.01f64).ln() * (40.0f64 / 39.0).sqrt() * 252f64.sqrt();
        assert!((v - expected).abs() < 1e-9, "{v} × {expected}");
        assert_eq!(annualized_volatility(&p[..5]), None);
        let c = [(1.0, 0.13), (3.0, 0.12), (5.0, 0.125)];
        assert_eq!(curve_rate(&c, 0.5), Some(0.13));
        assert!((curve_rate(&c, 2.0).unwrap() - 0.125).abs() < 1e-12);
        assert!((curve_rate(&c, 4.0).unwrap() - 0.1225).abs() < 1e-12);
        assert_eq!(curve_rate(&c, 9.0), Some(0.125));
        assert_eq!(curve_rate(&[], 1.0), None);
    }

    #[test]
    fn rates_in_pct_of_cdi() {
        // A mesma taxa = 100% do CDI.
        assert!((pct_of(0.149, 0.149).unwrap() - 1.0).abs() < 1e-12);
        // 12% pré com CDI de 14,9%: ≈ 0,8159 (razão das taxas diárias).
        let p = pct_of(0.12, 0.149).unwrap();
        let expect = (1.12f64.powf(1.0 / 252.0) - 1.0) / (1.149f64.powf(1.0 / 252.0) - 1.0);
        assert!(
            (p - expect).abs() < 1e-12 && (p - 0.8159).abs() < 0.0001,
            "{p}"
        );
        // LCI de 92% do CDI isenta, IR de 15%: 108,2% equivalente.
        assert!((gross_up(0.92, 0.15) - 1.082_352_9).abs() < 1e-6);
        // IPCA + 7% com Tesouro IPCA+ 6,8% e prefixado 12,8%: 12,8% × 1,07/1,068.
        let n = real_to_nominal(0.07, 0.128, 0.068);
        assert!((n - (1.07 * 1.128 / 1.068 - 1.0)).abs() < 1e-12);
        assert!((real_rate(0.15, 0.05) - (1.15 / 1.05 - 1.0)).abs() < 1e-12);
        // Poupança com Selic 15%: 0,5% a.m. (6,17% a.a.) + TR 2%.
        let s = savings_yield(0.15, 0.02, 0.085, 0.005, 0.7);
        assert!((s - (1.005f64.powi(12) * 1.02 - 1.0)).abs() < 1e-12);
        // Selic 8%: 70% × 8% = 5,6% + TR 0.
        assert!((savings_yield(0.08, 0.0, 0.085, 0.005, 0.7) - 0.056).abs() < 1e-12);
    }

    #[test]
    fn fgc_per_holder_institution_and_global_cap() {
        let m = |kv: &[(&str, Decimal)]| -> BTreeMap<String, Decimal> {
            kv.iter().map(|(k, v)| (k.to_string(), *v)).collect()
        };
        let limit = dec!(250000);
        let global = dec!(1000000);
        // Dentro do limite.
        let o = fgc_coverage(
            &m(&[("ana", dec!(100000))]),
            &m(&[("ana", dec!(200000))]),
            &m(&[("ana", dec!(200000))]),
            limit,
            global,
        );
        assert_eq!(o.coverage, 1.0);
        assert_eq!(o.excess, Decimal::ZERO);
        // Ana tem 500 mil no banco (este CDB: 300 mil) → 50%; Beto, 100 mil → 100%.
        let o = fgc_coverage(
            &m(&[("ana", dec!(300000)), ("beto", dec!(100000))]),
            &m(&[("ana", dec!(500000)), ("beto", dec!(100000))]),
            &m(&[("ana", dec!(250000)), ("beto", dec!(100000))]),
            limit,
            global,
        );
        assert!((o.coverage - (300000.0 * 0.5 + 100000.0) / 400000.0).abs() < 1e-12);
        assert_eq!(o.excess, dec!(250000));
        assert_eq!(o.max_exposure, dec!(500000));
        // Teto global: 1,25 mi cobertos em 5 bancos → 80% de cada um.
        let o = fgc_coverage(
            &m(&[("ana", dec!(250000))]),
            &m(&[("ana", dec!(250000))]),
            &m(&[("ana", dec!(1250000))]),
            limit,
            global,
        );
        assert!((o.coverage - 0.8).abs() < 1e-12);
        // Sem posição: dentro do limite.
        let o = fgc_coverage(&m(&[]), &m(&[]), &m(&[]), limit, global);
        assert_eq!(o.coverage, 1.0);
    }

    #[test]
    fn issuer_names_and_years() {
        let stop: Vec<String> = ["banco", "s", "a", "sa", "prudencial", "do"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(core_name("Banco Exemplo S.A.", &stop), "exemplo");
        assert_eq!(core_name("BANCO EXEMPLO - PRUDENCIAL", &stop), "exemplo");
        assert_eq!(core_name("Banco do Brasil", &stop), "brasil");
        assert_ne!(core_name("Banco de Brasília", &stop), "brasil");
        assert_eq!(trailing_year("Tesouro IPCA+ 2035"), Some(2035));
        assert_eq!(trailing_year("Tesouro Selic"), None);
        assert_eq!(trailing_year("NTN-B 15/05/2045"), Some(2045));
    }
}
