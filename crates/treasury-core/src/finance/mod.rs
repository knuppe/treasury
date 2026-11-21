//! Matemática financeira pura (sem banco): XIRR, TWR, deflação por índices, calendário
//! de dias úteis, renda fixa (CDI, prefixado, IPCA+), IR regressivo e IOF, preço médio.
//!
//! Taxas e retornos são razões em `f64` (exibição e comparação). Valores monetários e
//! quantidades continuam exatos: centavos `i64` e `rust_decimal::Decimal`.

pub mod average_cost;
pub mod calendar;
pub mod fixed_income;
pub mod inflation;
pub mod returns;

use rust_decimal::MathematicalOps;
use rust_decimal::prelude::*;

/// `base^exp` para fatores financeiros: parte inteira do expoente exata (`powi`), parte
/// fracionária em `f64` (erro relativo ~1e-16, irrelevante para centavos). Evita o
/// `powd` do rust_decimal, que estoura a pilha com alguns expoentes.
pub fn pow_dec(base: Decimal, exp: Decimal) -> Decimal {
    let int = exp.trunc();
    let frac = exp - int;
    let ip = match int.to_i64() {
        Some(i) => base.powi(i),
        None => return Decimal::ZERO,
    };
    if frac.is_zero() {
        return ip;
    }
    let fp = base
        .to_f64()
        .unwrap_or(1.0)
        .powf(frac.to_f64().unwrap_or(0.0));
    ip * Decimal::from_f64(fp).unwrap_or(Decimal::ONE)
}

#[cfg(test)]
mod pow_tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn pow_dec_exact_and_fractional() {
        assert_eq!(pow_dec(dec!(1.1), dec!(1)), dec!(1.1));
        assert_eq!(pow_dec(dec!(1.1), dec!(2)), dec!(1.21));
        assert_eq!(pow_dec(dec!(1.21), dec!(0.5)).round_dp(10), dec!(1.1));
        assert!((pow_dec(dec!(1.1), dec!(1.5)) - dec!(1.15368973)).abs() < dec!(0.0000001));
    }
}
