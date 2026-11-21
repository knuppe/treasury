//! Duas apurações do imposto da pessoa física fora da declaração de ajuste:
//!
//! - **PLR** (participação nos lucros, Lei 10.101/2000 art. 3º §§ 5º a 11 e anexo): tabela
//!   anual exclusiva, recalculada sobre o total pago no ano (§7º); só a pensão alimentícia
//!   abate (Perguntas e Respostas IRPF 2026, pergunta 179). Tabela de jan–abr/2025 e a de maio
//!   de 2025 em diante, que segue em 2026 (Receita, tabelas 2025 e 2026).
//! - **Carnê-leão** (rendimentos de pessoas físicas e do exterior): tabela mensal do mês,
//!   com previdência oficial, dependentes, pensão e livro-caixa, ou o desconto simplificado
//!   mensal, e a redução da Lei 15.270/2025 a partir de 2026; DARF 0190 até o último dia útil
//!   do mês seguinte (perguntas 266 e 267).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::t;
use crate::tax_burden::rules::{IrBracket, ir_table, monthly_irrf};

fn plr_brackets(month: &str) -> Vec<IrBracket> {
    let b = |up_to: Option<i64>, rate: f64, deduction: i64| IrBracket {
        up_to,
        rate,
        deduction,
    };
    if month < "2025-05" {
        vec![
            b(Some(764_080), 0.0, 0),
            b(Some(992_228), 7.5, 57_306),
            b(Some(1_316_700), 15.0, 131_723),
            b(Some(1_638_038), 22.5, 230_476),
            b(None, 27.5, 312_378),
        ]
    } else {
        vec![
            b(Some(821_440), 0.0, 0),
            b(Some(992_228), 7.5, 61_608),
            b(Some(1_316_700), 15.0, 136_025),
            b(Some(1_638_038), 22.5, 234_778),
            b(None, 27.5, 316_680),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlrArgs {
    /// Mês do pagamento (AAAA-MM).
    pub month: String,
    /// Valor desta PLR.
    #[ts(type = "number")]
    pub gross: i64,
    /// PLR já paga no mesmo ano (o imposto é recalculado sobre o total).
    #[serde(default)]
    #[ts(type = "number")]
    pub paid_before: i64,
    /// IR já retido nessas PLR anteriores.
    #[serde(default)]
    #[ts(type = "number")]
    pub withheld_before: i64,
    /// Pensão alimentícia descontada desta PLR.
    #[serde(default)]
    #[ts(type = "number")]
    pub alimony: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlrTax {
    /// IR desta PLR (o do total do ano menos o já retido).
    #[ts(type = "number")]
    pub tax: i64,
    #[ts(type = "number")]
    pub net: i64,
}

pub fn plr(a: &PlrArgs) -> Result<PlrTax> {
    crate::time::parse_month(&a.month)?;
    let total = a.paid_before + a.gross - a.alimony.max(0);
    let year_tax = ir_table(total, &plr_brackets(&a.month));
    let tax = (year_tax - a.withheld_before).max(0);
    Ok(PlrTax {
        tax,
        net: a.gross - tax,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CarneLeaoArgs {
    pub month: String,
    /// Recebido no mês de pessoas físicas e do exterior.
    #[ts(type = "number")]
    pub income: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub social_security: i64,
    #[serde(default)]
    pub dependents: u32,
    #[serde(default)]
    #[ts(type = "number")]
    pub alimony: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub livro_caixa: i64,
    /// Imposto pago no exterior sobre esses rendimentos no mês (abate do devido).
    #[serde(default)]
    #[ts(type = "number")]
    pub foreign_tax: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CarneLeao {
    #[ts(type = "number")]
    pub tax: i64,
    pub revenue_code: String,
    pub due_date: String,
    /// O valor fica abaixo de R$ 10: acumula para o mês seguinte (não se paga DARF).
    pub below_minimum: bool,
}

pub fn carne_leao(a: &CarneLeaoArgs) -> Result<CarneLeao> {
    crate::time::parse_month(&a.month)?;
    let ds = crate::tax_burden::dataset::get("BR")?;
    let rules = crate::tax_burden::rules_for(&ds, &a.month, None)
        .cloned()
        .ok_or_else(|| Error::invalid(t("nucleo-impostos-ir-erro-ano")))?;
    // Previdência, pensão e livro-caixa somam nas deduções legais, comparadas ao desconto
    // simplificado mensal dentro de `monthly_irrf`.
    let legal = a.social_security.max(0) + a.alimony.max(0) + a.livro_caixa.max(0);
    let tax = (monthly_irrf(a.income, legal, a.dependents, &rules) - a.foreign_tax.max(0)).max(0);
    let r = super::rules::TaxRules::for_year(a.month[..4].parse().unwrap_or(2026));
    Ok(CarneLeao {
        tax,
        revenue_code: "0190".into(),
        due_date: crate::time::fmt_date(super::engine::darf_due_date(&a.month)?),
        below_minimum: tax > 0 && tax < r.darf_minimum,
    })
}
