//! Dados oficiais do FGTS embutidos, com fonte e data (docs/brazil/fgts.md, seção 2). Revisar todo
//! ano em agosto, quando o Conselho Curador aprova a distribuição de resultado.

/// Juros da conta vinculada: 3% ao ano, capitalização mensal (Lei 8.036/1990, art. 13).
pub const ANNUAL_INTEREST: f64 = 0.03;

/// Depósito mensal: 8% da remuneração (Lei 8.036/1990, art. 15).
pub const DEPOSIT_RATE: f64 = 0.08;

/// Dia do crédito do JAM: 10 até março de 2024, 21 desde abril de 2024 (Lei 14.438/2022).
pub fn jam_day(year: i32, month: u32) -> u32 {
    if (year, month) < (2024, 4) { 10 } else { 21 }
}

/// Prazo do depósito do empregador: dia 20 do mês seguinte (FGTS Digital, desde 01/03/2024);
/// antes, dia 7.
pub fn deposit_due_day(year: i32, month: u32) -> u32 {
    if (year, month) < (2024, 3) { 7 } else { 20 }
}

/// Índice de distribuição de resultado por real de saldo em 31/12 do ano-base.
pub struct Distribution {
    pub base_year: i32,
    pub index: f64,
    /// Data da aprovação pelo Conselho Curador.
    pub approved: &'static str,
    pub source: &'static str,
}

pub const DISTRIBUTIONS: &[Distribution] = &[
    Distribution {
        base_year: 2024,
        index: 0.020_429_19,
        approved: "2025-07-24",
        source: "Caixa, perguntas frequentes sobre os rendimentos do FGTS",
    },
    Distribution {
        base_year: 2025,
        index: 0.018_683_41,
        approved: "2026-07-28",
        source: "Conselho Curador do FGTS (R$ 13,04 bi, 89% do lucro de 2025)",
    },
];

pub fn distribution(base_year: i32) -> Option<&'static Distribution> {
    DISTRIBUTIONS.iter().find(|d| d.base_year == base_year)
}

/// Índice estimado para um ano-base sem índice oficial: a média dos últimos três conhecidos.
pub fn estimated_distribution() -> f64 {
    let last: Vec<f64> = DISTRIBUTIONS
        .iter()
        .rev()
        .take(3)
        .map(|d| d.index)
        .collect();
    if last.is_empty() {
        0.0
    } else {
        last.iter().sum::<f64>() / last.len() as f64
    }
}

/// Faixa do saque-aniversário: saldo até (centavos), alíquota e parcela adicional (Lei
/// 8.036/1990, Anexo, incluído pela Lei 13.932/2019; Circular Caixa 889/2020).
pub struct AnniversaryBand {
    pub up_to: Option<i64>,
    pub rate: f64,
    pub extra: i64,
}

pub const ANNIVERSARY: &[AnniversaryBand] = &[
    AnniversaryBand {
        up_to: Some(50_000),
        rate: 0.50,
        extra: 0,
    },
    AnniversaryBand {
        up_to: Some(100_000),
        rate: 0.40,
        extra: 5_000,
    },
    AnniversaryBand {
        up_to: Some(500_000),
        rate: 0.30,
        extra: 15_000,
    },
    AnniversaryBand {
        up_to: Some(1_000_000),
        rate: 0.20,
        extra: 65_000,
    },
    AnniversaryBand {
        up_to: Some(1_500_000),
        rate: 0.15,
        extra: 115_000,
    },
    AnniversaryBand {
        up_to: Some(2_000_000),
        rate: 0.10,
        extra: 190_000,
    },
    AnniversaryBand {
        up_to: None,
        rate: 0.05,
        extra: 290_000,
    },
];

pub const ANNIVERSARY_SOURCE: &str = "Lei 8.036/1990, Anexo (Lei 13.932/2019)";

/// Valor do saque-aniversário para o saldo total (centavos): uma faixa só, pelo saldo total.
pub fn anniversary_amount(total: i64) -> (i64, &'static AnniversaryBand) {
    let band = ANNIVERSARY
        .iter()
        .find(|b| b.up_to.is_none_or(|u| total <= u))
        .expect("última faixa sem limite");
    let v =
        (total.max(0) as f64 * band.rate).round() as i64 + if total > 0 { band.extra } else { 0 };
    (v.min(total.max(0)), band)
}

/// Motivos de saque e os códigos conhecidos (docs/brazil/fgts.md, seção 3). O código 99 é genérico;
/// sem outra informação, é tratado como uso na moradia.
pub fn reason_of_code(code: &str) -> Option<&'static str> {
    let c = code.trim().to_ascii_uppercase();
    Some(match c.as_str() {
        "01" | "1" => "rescisao",
        "50" | "19E" => "emergencial",
        "60" => "aniversario",
        "91" | "92" | "93" | "94" | "95" | "96" | "99" => "moradia",
        _ => return None,
    })
}

pub const REASONS: &[&str] = &[
    "rescisao",
    "aposentadoria",
    "moradia",
    "aniversario",
    "doenca",
    "emergencial",
    "outros",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anniversary_bands_at_the_edges() {
        assert_eq!(anniversary_amount(50_000).0, 25_000);
        // 500,01: 40% + 50 = 250,00 (rounded).
        assert_eq!(anniversary_amount(50_001).0, 25_000);
        assert_eq!(anniversary_amount(2_000_000).0, 390_000);
        // 20.000,01: 5% + 2.900 = 3.900,00.
        assert_eq!(anniversary_amount(2_000_001).0, 390_000);
        assert_eq!(anniversary_amount(850_000).0, 235_000);
        assert_eq!(anniversary_amount(0).0, 0);
    }

    #[test]
    fn jam_day_moves_to_21_in_april_2024() {
        assert_eq!(jam_day(2024, 3), 10);
        assert_eq!(jam_day(2024, 4), 21);
        assert_eq!(deposit_due_day(2024, 2), 7);
        assert_eq!(deposit_due_day(2024, 3), 20);
    }
}
