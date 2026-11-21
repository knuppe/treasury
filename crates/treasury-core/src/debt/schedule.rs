//! Cronograma de amortização: SAC, Price e bullet (só juros, principal no fim), com carência
//! (só juros), correção monetária do saldo pelo indexador (TR, IPCA, IGP-M), seguros e taxas
//! fixos, e amortizações extraordinárias que reduzem o prazo ou a parcela. Tudo em centavos
//! com arredondamento metade para longe do zero; a taxa mensal é a equivalente à efetiva
//! anual. Nos pós-fixados de taxa diária (CDI, Selic), cada período traz o próprio fator de
//! juros (índice acumulado nos dias úteis × spread), calculado em `debt::postfixed`.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::finance::pow_dec;
use crate::time::add_months;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System {
    Sac,
    Price,
    /// Só juros a cada vencimento; o principal vai inteiro na última parcela.
    Bullet,
}

/// Período de um pós-fixado: vencimento (já no dia útil) e fator de juros do período
/// (índice acumulado × spread), com os dias úteis e se o índice foi projetado.
#[derive(Debug, Clone, PartialEq)]
pub struct Period {
    pub due: NaiveDate,
    pub factor: Decimal,
    pub business_days: u32,
    pub estimated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraMode {
    ReduceTerm,
    ReduceInstallment,
}

#[derive(Debug, Clone)]
pub struct Extra {
    /// Aplicada logo depois da parcela `after_number` (0 = antes da primeira).
    pub after_number: u32,
    pub amount: i64,
    pub mode: ExtraMode,
}

#[derive(Debug, Clone)]
pub struct Params {
    pub principal: i64,
    pub annual_rate_pct: Decimal,
    pub system: System,
    pub term: u32,
    pub grace: u32,
    pub first_due: NaiveDate,
    pub insurance: i64,
    pub fees: i64,
    /// Seguro MIP: fração ao mês sobre o saldo devedor (ex.: 0,0003 = 0,03% a.m.).
    pub mip_rate: Decimal,
    /// MIP em valor fixo por mês (centavos; 0 = usa a fração sobre o saldo).
    pub mip_fixed: i64,
    /// Seguro DFI do mês (fração sobre o valor do imóvel, já em centavos).
    pub dfi: i64,
    /// Fator de correção do saldo para cada parcela (1-based; ausente = 1). `bool` = estimado.
    pub index_factors: Vec<(Decimal, bool)>,
    pub extras: Vec<Extra>,
    /// Pós-fixado de taxa diária: um período por parcela (vazio = taxa mensal fixa).
    pub periods: Vec<Period>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub number: u32,
    pub due_date: NaiveDate,
    pub amortization: i64,
    pub interest: i64,
    pub adjustment: i64,
    /// Seguro total da parcela (fixo + MIP + DFI).
    pub insurance: i64,
    pub mip: i64,
    pub dfi: i64,
    pub fees: i64,
    pub total: i64,
    pub balance_after: i64,
    pub estimated: bool,
    pub extra_payment: i64,
    /// Pós-fixados: fator de juros do período e dias úteis.
    pub index_factor: Option<Decimal>,
    pub business_days: Option<u32>,
}

fn round(v: Decimal) -> Decimal {
    v.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
}

pub fn monthly_rate(annual_pct: Decimal) -> Decimal {
    pow_dec(
        Decimal::ONE + annual_pct / Decimal::ONE_HUNDRED,
        Decimal::ONE / Decimal::from(12),
    ) - Decimal::ONE
}

/// PMT da tabela Price: B × i ÷ (1 − (1+i)^−n).
pub fn price_payment(balance: Decimal, i: Decimal, n: u32) -> Decimal {
    if i.is_zero() {
        return balance / Decimal::from(n.max(1));
    }
    let f = pow_dec(Decimal::ONE + i, Decimal::from(n));
    balance * i * f / (f - Decimal::ONE)
}

/// Taxa mensal que faz o cronograma ter a parcela `payment` (sem seguros e taxas):
/// na Price é a parcela constante depois da carência; no SAC, a primeira parcela com
/// amortização (saldo cheio × taxa + saldo ÷ parcelas). `None` se a parcela não cobre o
/// valor financiado no prazo ou exige juros absurdos (acima de 50% a.m.).
pub fn solve_monthly_rate(
    principal: i64,
    payment: i64,
    amort_months: u32,
    system: System,
) -> Option<Decimal> {
    if principal <= 0 || payment <= 0 || amort_months == 0 {
        return None;
    }
    let (p, pmt, n) = (principal as f64, payment as f64, f64::from(amort_months));
    let i = match system {
        // No bullet a parcela é só juros: saldo × taxa.
        System::Bullet => pmt / p,
        System::Sac => (pmt - p / n) / p,
        System::Price => {
            if pmt * n < p - 0.5 {
                return None;
            }
            let f = |i: f64| {
                if i == 0.0 {
                    p / n
                } else {
                    p * i / (1.0 - (1.0 + i).powf(-n))
                }
            };
            let (mut lo, mut hi) = (0.0_f64, 0.5_f64);
            if f(hi) < pmt {
                return None;
            }
            for _ in 0..200 {
                let mid = (lo + hi) / 2.0;
                if f(mid) < pmt {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            (lo + hi) / 2.0
        }
    };
    if !(-1e-9..=0.5).contains(&i) {
        return None;
    }
    Decimal::from_f64(i.max(0.0)).map(|d| d.round_dp(12))
}

/// Taxa efetiva anual (% a.a.) equivalente a uma taxa mensal.
pub fn annual_pct_from_monthly(i: Decimal) -> Decimal {
    ((pow_dec(Decimal::ONE + i, Decimal::from(12)) - Decimal::ONE) * Decimal::ONE_HUNDRED)
        .round_dp(6)
}

/// Parcelas que faltam para quitar `balance` mantendo a amortização (SAC) ou a parcela
/// (Price) `kept`, à taxa mensal `i` (sem correção).
fn months_left(system: System, balance: Decimal, kept: Decimal, i: Decimal) -> u32 {
    let b = balance.to_f64().unwrap_or(0.0);
    let k = kept.to_f64().unwrap_or(0.0);
    let r = i.to_f64().unwrap_or(0.0);
    let n = match system {
        System::Sac | System::Bullet => b / k,
        System::Price if r == 0.0 => b / k,
        System::Price => {
            let q = 1.0 - b * r / k;
            if q <= 0.0 {
                return 600;
            }
            -q.ln() / (1.0 + r).ln()
        }
    };
    if n.is_finite() && n > 0.0 {
        (n - 1e-9).ceil().min(600.0) as u32
    } else {
        1
    }
}

pub fn build(p: &Params) -> Vec<Row> {
    let monthly = monthly_rate(p.annual_rate_pct);
    // Taxa do período k (1-based): a do período pós-fixado ou a mensal fixa.
    let rate_of = |k: u32| -> Decimal {
        match p.periods.get(k.max(1) as usize - 1).or(p.periods.last()) {
            Some(per) => per.factor - Decimal::ONE,
            None => monthly,
        }
    };
    let i = rate_of(1);
    let mut balance = Decimal::from(p.principal);
    let mut rows = vec![];
    // Depois de uma amortização extra que reduz o prazo, mantém-se a parcela (Price) ou a
    // amortização (SAC) e o prazo encurta.
    let mut fixed: Option<Decimal> = None;
    let mut term_left = p.term;
    for e in p.extras.iter().filter(|e| e.after_number == 0) {
        // Antes da primeira parcela, "reduzir prazo" mantém a parcela (Price) ou a
        // amortização (SAC) que o contrato teria sem a amortização.
        if e.mode == ExtraMode::ReduceTerm && fixed.is_none() {
            let n = Decimal::from(p.term.saturating_sub(p.grace).max(1));
            fixed = match p.system {
                System::Sac => Some(round(balance / n)),
                System::Price => Some(round(price_payment(
                    balance,
                    i,
                    p.term.saturating_sub(p.grace).max(1),
                ))),
                System::Bullet => None,
            };
        }
        balance = (balance - Decimal::from(e.amount)).max(Decimal::ZERO);
    }
    let mut k = 0u32;
    while balance > Decimal::ZERO && k < p.term + 600 {
        k += 1;
        let (factor, est) = p
            .index_factors
            .get(k as usize - 1)
            .copied()
            .unwrap_or((Decimal::ONE, false));
        let before = balance;
        balance = round(balance * factor);
        let adjustment = balance - before;
        // A parcela (Price) ou a amortização (SAC) mantida depois de uma amortização que
        // reduz o prazo também é corrigida pelo índice; sem isso, com TR/IPCA o saldo cresce
        // mais que a amortização fixa e o prazo "reduzido" passa do original.
        if let Some(f) = fixed {
            fixed = Some(round(f * factor));
        }
        let i = rate_of(k);
        let period = p.periods.get(k as usize - 1);
        let est = est || period.is_some_and(|x| x.estimated);
        let interest = round(balance * i);
        let remaining = term_left.saturating_sub(k - 1).max(1);
        let mut amort = if k <= p.grace {
            Decimal::ZERO
        } else {
            match (p.system, fixed) {
                (System::Sac, Some(a)) => a,
                (System::Sac, None) => round(
                    balance
                        / Decimal::from(
                            remaining
                                .saturating_sub(p.grace.saturating_sub(k - 1))
                                .max(1),
                        ),
                ),
                (System::Price, Some(pmt)) => round(pmt - interest),
                (System::Price, None) => {
                    let n = remaining
                        .saturating_sub(p.grace.saturating_sub(k - 1))
                        .max(1);
                    round(price_payment(balance, i, n) - interest)
                }
                (System::Bullet, _) => Decimal::ZERO,
            }
        };
        if amort > balance || (k >= term_left && fixed.is_none()) {
            amort = balance;
        }
        amort = amort.max(Decimal::ZERO);
        balance -= amort;
        let mut extra_total = Decimal::ZERO;
        for e in p.extras.iter().filter(|e| e.after_number == k) {
            let x = Decimal::from(e.amount).min(balance);
            balance -= x;
            extra_total += x;
            if balance > Decimal::ZERO {
                match e.mode {
                    ExtraMode::ReduceTerm => {
                        fixed = match p.system {
                            System::Sac => Some(amort),
                            System::Price => Some(amort + interest),
                            // No bullet o principal vai no fim: a amortização só reduz o saldo.
                            System::Bullet => None,
                        };
                    }
                    ExtraMode::ReduceInstallment => {
                        // Depois de um "reduzir prazo", o prazo a manter é o que restava
                        // com a parcela/amortização mantida, não o original + folga.
                        if let Some(f) = fixed {
                            term_left = k + months_left(p.system, balance + x, f, i);
                        }
                        fixed = None;
                    }
                }
            }
        }
        if fixed.is_some() {
            // Prazo passa a ser o necessário para quitar com a parcela/amortização mantida.
            term_left = p.term + 600;
        }
        // MIP sobre o saldo devedor do mês (já corrigido, antes de amortizar); DFI fixo.
        let (ins, mip, dfi) = if balance + amort > Decimal::ZERO {
            let mip = if p.mip_fixed > 0 {
                p.mip_fixed
            } else {
                round((balance + amort) * p.mip_rate).to_i64().unwrap_or(0)
            };
            (p.insurance + mip + p.dfi, mip, p.dfi)
        } else {
            (0, 0, 0)
        };
        let total = amort + interest + Decimal::from(ins + p.fees);
        rows.push(Row {
            number: k,
            due_date: period.map_or_else(|| add_months(p.first_due, k as i32 - 1), |x| x.due),
            amortization: amort.to_i64().unwrap_or(0),
            interest: interest.to_i64().unwrap_or(0),
            adjustment: adjustment.to_i64().unwrap_or(0),
            insurance: ins,
            mip,
            dfi,
            fees: p.fees,
            total: total.to_i64().unwrap_or(0),
            balance_after: balance.to_i64().unwrap_or(0),
            estimated: est,
            extra_payment: extra_total.to_i64().unwrap_or(0),
            index_factor: period.map(|x| x.factor),
            business_days: period.map(|x| x.business_days),
        });
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    fn params(system: System) -> Params {
        Params {
            principal: 12_000_000,            // R$ 120.000
            annual_rate_pct: dec!(12.682503), // ≈ 1% a.m.
            system,
            term: 120,
            grace: 0,
            first_due: d("2026-01-10"),
            insurance: 0,
            fees: 0,
            mip_rate: Decimal::ZERO,
            mip_fixed: 0,
            dfi: 0,
            index_factors: vec![],
            periods: vec![],
            extras: vec![],
        }
    }

    #[test]
    fn monthly_rate_from_annual() {
        let i = monthly_rate(dec!(12.682503));
        assert!((i - dec!(0.01)).abs() < dec!(0.0000001), "{i}");
    }

    #[test]
    fn sac_constant_amortization_decreasing_payment() {
        let r = build(&params(System::Sac));
        assert_eq!(r.len(), 120);
        assert_eq!(r[0].amortization, 100_000); // 120.000 / 120
        assert_eq!(r[0].interest, 120_000); // 1% de 120.000
        assert_eq!(r[0].total, 220_000);
        assert!(r[1].total < r[0].total);
        assert_eq!(r.last().unwrap().balance_after, 0);
        assert_eq!(r.iter().map(|x| x.amortization).sum::<i64>(), 12_000_000);
    }

    #[test]
    fn price_constant_payment() {
        let r = build(&params(System::Price));
        assert_eq!(r.len(), 120);
        // PMT = 120.000 × 0,01 / (1 − 1,01^−120) ≈ 1.721,65
        assert!((r[0].total - 172_165).abs() <= 1, "{}", r[0].total);
        assert!((r[60].total - r[0].total).abs() <= 1);
        assert_eq!(r.last().unwrap().balance_after, 0);
        assert_eq!(r.iter().map(|x| x.amortization).sum::<i64>(), 12_000_000);
    }

    #[test]
    fn grace_pays_interest_only() {
        let mut p = params(System::Sac);
        p.grace = 6;
        let r = build(&p);
        assert!(
            r[..6]
                .iter()
                .all(|x| x.amortization == 0 && x.interest == 120_000)
        );
        assert_eq!(r[6].amortization, 105_263); // 120.000 / 114 parcelas restantes
        assert_eq!(r.len(), 120);
        assert_eq!(r.last().unwrap().balance_after, 0);
    }

    #[test]
    fn extra_payment_reduce_term_vs_installment() {
        let base = build(&params(System::Price));
        let mut p = params(System::Price);
        p.extras = vec![Extra {
            after_number: 12,
            amount: 2_000_000,
            mode: ExtraMode::ReduceTerm,
        }];
        let term = build(&p);
        assert!(term.len() < 120, "prazo reduzido: {}", term.len());
        assert!(
            (term[20].total - base[0].total).abs() <= 1,
            "parcela mantida"
        );
        p.extras[0].mode = ExtraMode::ReduceInstallment;
        let inst = build(&p);
        assert_eq!(inst.len(), 120, "prazo mantido");
        assert!(inst[20].total < base[20].total, "parcela menor");
        let interest = |r: &[Row]| r.iter().map(|x| x.interest).sum::<i64>();
        assert!(
            interest(&term) < interest(&inst),
            "reduzir prazo economiza mais juros"
        );
        assert!(interest(&inst) < interest(&base));
    }

    #[test]
    fn reduce_term_with_indexer_shortens_the_contract() {
        for system in [System::Sac, System::Price] {
            let mut p = params(system);
            p.index_factors = vec![(dec!(1.003), true); 720];
            let base = build(&p);
            assert_eq!(base.len(), 120);
            p.extras = vec![Extra {
                after_number: 12,
                amount: 1_000_000,
                mode: ExtraMode::ReduceTerm,
            }];
            let term = build(&p);
            assert!(
                term.len() < base.len(),
                "{system:?}: prazo reduzido com correção ({} parcelas)",
                term.len()
            );
            assert_eq!(term.last().unwrap().balance_after, 0);
            // A amortização (SAC) ou a parcela (Price) mantida acompanha o índice.
            let kept = |r: &Row| match system {
                System::Sac | System::Bullet => r.amortization,
                System::Price => r.amortization + r.interest,
            };
            assert!(kept(&term[30]) > kept(&term[13]));
        }
    }

    #[test]
    fn reduce_term_before_the_first_installment() {
        for system in [System::Sac, System::Price] {
            let base = build(&params(system));
            let mut p = params(system);
            p.extras = vec![Extra {
                after_number: 0,
                amount: 2_000_000,
                mode: ExtraMode::ReduceTerm,
            }];
            let term = build(&p);
            assert!(term.len() < 120, "{system:?}: {} parcelas", term.len());
            if system == System::Price {
                assert!(
                    (term[0].total - base[0].total).abs() <= 1,
                    "parcela mantida"
                );
            } else {
                assert_eq!(term[0].amortization, base[0].amortization);
            }
            p.extras[0].mode = ExtraMode::ReduceInstallment;
            assert_eq!(build(&p).len(), 120);
        }
    }

    #[test]
    fn reduce_installment_after_reduce_term_keeps_the_shorter_term() {
        for system in [System::Sac, System::Price] {
            let mut p = params(system);
            p.extras = vec![Extra {
                after_number: 12,
                amount: 2_000_000,
                mode: ExtraMode::ReduceTerm,
            }];
            let term = build(&p);
            p.extras.push(Extra {
                after_number: 30,
                amount: 1_000_000,
                mode: ExtraMode::ReduceInstallment,
            });
            let both = build(&p);
            assert_eq!(
                both.len(),
                term.len(),
                "{system:?}: reduzir a parcela mantém o prazo já reduzido"
            );
            assert!(both[31].total < term[31].total, "{system:?}: parcela menor");
            assert_eq!(both.last().unwrap().balance_after, 0);
        }
    }

    #[test]
    fn indexed_balance_and_fees() {
        let mut p = params(System::Sac);
        p.index_factors = vec![(dec!(1.005), false); 3];
        p.insurance = 5_000;
        p.fees = 2_500;
        let r = build(&p);
        assert_eq!(r[0].adjustment, 60_000); // 0,5% de 120.000
        assert_eq!(r[0].interest, 120_600);
        assert_eq!(r[0].total, r[0].amortization + r[0].interest + 7_500);
        assert_eq!(r.last().unwrap().balance_after, 0);
    }

    /// Financiamento Caixa na Tabela Price com TR: a cada mês o saldo é corrigido pela TR
    /// primeiro; juros e amortização saem do saldo corrigido e a prestação (A+J) é a Price
    /// do saldo corrigido no prazo que resta, então cresce com a TR; MIP sobre o saldo
    /// corrigido, DFI e taxa de administração por cima. Com TR constante, A+J do mês k é a
    /// do mês 1 × (1+TR)^(k−1), e o saldo zera no prazo.
    #[test]
    fn caixa_price_with_tr_corrects_balance_then_recomputes_installment() {
        let mut p = params(System::Price);
        p.term = 360;
        p.principal = 30_000_000; // R$ 300.000
        p.annual_rate_pct = dec!(10.5);
        p.index_factors = vec![(dec!(1.0015), false); 360]; // TR 0,15% ao mês
        p.mip_rate = dec!(0.000247);
        p.dfi = 3_750;
        p.fees = 2_500;
        let r = build(&p);
        assert_eq!(r.len(), 360);
        assert_eq!(r.last().unwrap().balance_after, 0);
        let i = monthly_rate(p.annual_rate_pct);
        // Mês 1: saldo corrigido, depois juros sobre ele.
        let corrected = Decimal::from(p.principal) * dec!(1.0015);
        assert_eq!(
            r[0].adjustment,
            round(corrected - Decimal::from(p.principal))
                .to_i64()
                .unwrap()
        );
        assert_eq!(r[0].interest, round(corrected * i).to_i64().unwrap());
        // MIP sobre o saldo corrigido (antes de amortizar).
        assert_eq!(
            r[0].mip,
            round(corrected * dec!(0.000247)).to_i64().unwrap()
        );
        assert_eq!(
            r[0].total,
            r[0].amortization + r[0].interest + r[0].mip + 3_750 + 2_500
        );
        // A+J cresce com a TR (até o arredondamento de centavos).
        let aj = |k: usize| Decimal::from(r[k].amortization + r[k].interest);
        for k in [1usize, 12, 120, 300] {
            let expected = aj(0) * pow_dec(dec!(1.0015), Decimal::from(k as i64));
            let diff = (aj(k) - expected).abs();
            assert!(
                diff <= Decimal::from(2 + k as i64 / 50),
                "mês {}: {} × {expected}",
                k + 1,
                aj(k)
            );
        }
        // Sem TR, a mesma Price tem A+J fixa.
        p.index_factors = vec![];
        let flat = build(&p);
        assert!(
            (flat[0].amortization + flat[0].interest - flat[200].amortization - flat[200].interest)
                .abs()
                <= 1
        );
    }
}
