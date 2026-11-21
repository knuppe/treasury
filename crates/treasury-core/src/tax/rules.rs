//! Regras de IR de renda variável para pessoa física, versionadas por ano. Mudanças de lei
//! entram como um novo bloco em [`TaxRules::for_year`], sem alterar anos anteriores.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct TaxRules {
    pub year: i32,
    /// Operações comuns (swing trade) em ações, ETFs e BDRs.
    pub swing_rate: Decimal,
    pub day_trade_rate: Decimal,
    /// Ganho na venda de cotas de FII/Fiagro.
    pub fii_rate: Decimal,
    /// Isenção de ganhos em ações no mercado à vista quando o total vendido no mês (swing)
    /// não passa deste valor (centavos). Não vale para ETFs, BDRs, FIIs nem day trade.
    pub stock_exemption_limit: i64,
    /// IRRF "dedo-duro" sobre o valor de venda em operações comuns e FIIs.
    pub irrf_swing: Decimal,
    /// IRRF sobre o ganho em day trade.
    pub irrf_day_trade: Decimal,
    /// DARF abaixo deste valor (centavos) não é pago: acumula para o mês seguinte.
    pub darf_minimum: i64,
    /// Código de receita do DARF de renda variável PF.
    pub darf_code: &'static str,
    /// Aplicações financeiras no exterior (Lei 14.754/2023, a partir de 2024): alíquota anual.
    pub foreign_rate: Decimal,
    /// Criptoativos em corretora no Brasil: isenção quando o total vendido no mês (no Brasil e
    /// no exterior) não passa deste valor (centavos). Lei 9.250/1995 art. 22, II.
    pub crypto_exemption_limit: i64,
    /// Ganho de capital (Lei 8.981/1995 art. 21, redação da Lei 13.259/2016): alíquota sobre a
    /// parcela do ganho até cada limite (centavos; `None` = acima do último).
    pub capital_gain_bands: [(Option<i64>, Decimal); 4],
    /// Código de receita do DARF de ganho de capital (inclui criptoativos).
    pub capital_gain_darf_code: &'static str,
}

impl TaxRules {
    pub fn for_year(year: i32) -> Self {
        // Regras vigentes desde a Lei 11.033/2004 e a IN RFB 1.585/2015; exterior pela Lei
        // 14.754/2023. Revisar a cada ano.
        Self {
            year,
            swing_rate: Decimal::new(15, 2),
            day_trade_rate: Decimal::new(20, 2),
            fii_rate: Decimal::new(20, 2),
            stock_exemption_limit: 2_000_000,
            irrf_swing: Decimal::new(5, 5),
            irrf_day_trade: Decimal::new(1, 2),
            darf_minimum: 1_000,
            darf_code: "6015",
            foreign_rate: Decimal::new(15, 2),
            crypto_exemption_limit: 3_500_000,
            capital_gain_bands: [
                (Some(500_000_000), Decimal::new(15, 2)),
                (Some(1_000_000_000), Decimal::new(175, 3)),
                (Some(3_000_000_000), Decimal::new(20, 2)),
                (None, Decimal::new(225, 3)),
            ],
            capital_gain_darf_code: "4600",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn current_rules() {
        let r = TaxRules::for_year(2026);
        assert_eq!(r.swing_rate, dec!(0.15));
        assert_eq!(r.irrf_swing, dec!(0.00005));
        assert_eq!(r.stock_exemption_limit, 2_000_000);
        assert_eq!(r.darf_code, "6015");
    }
}
