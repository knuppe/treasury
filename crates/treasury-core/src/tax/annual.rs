//! Declaração de ajuste anual do IRPF (modelos completo e simplificado), como o programa da
//! Receita calcula o resumo: rendimentos tributáveis, deduções, base, imposto devido, imposto
//! já pago e saldo a pagar ou a restituir, e o modelo mais vantajoso.
//!
//! Regras (fontes em docs/brazil/income-tax-validation.md, seção 7):
//! - Tabela anual (Lei 9.250/1995 art. 11; tabelas da Receita para 2025 e 2026).
//! - Desconto simplificado de 20% dos rendimentos tributáveis, limitado a R$ 16.754,34 até o
//!   ano-calendário 2025 e R$ 17.640,00 a partir de 2026 (art. 10, IX e X).
//! - Deduções do completo (art. 8º, II): previdência oficial, dependentes (R$ 2.275,08 cada),
//!   instrução (R$ 3.561,50 por pessoa, sem transferir sobra entre pessoas), saúde (sem
//!   limite, menos o reembolsado), pensão alimentícia judicial ou por escritura, previdência
//!   complementar (PGBL/Fapi) até 12% dos rendimentos tributáveis e livro-caixa.
//! - Doações feitas na declaração ao fundo da criança e do adolescente e ao da pessoa idosa,
//!   cada uma até 3% do imposto (só no completo).
//! - Imposto pago no exterior, até a diferença entre o imposto com e sem o rendimento de fora,
//!   em cada modelo (Perguntas e Respostas IRPF 2026, pergunta 267).
//! - A partir do ano-calendário 2026, a redução anual da Lei 15.270/2025 (art. 11-A): até
//!   R$ 2.694,15 para rendimentos tributáveis até R$ 60.000,00; 8.429,73 − 0,095575 ×
//!   rendimentos até R$ 88.200,00; nada acima. Aplicada nos dois modelos, limitada ao imposto
//!   da tabela (a lei não distingue modelos; conferir quando sair o programa de 2027).
//!
//! 13º e PLR são de tributação exclusiva e não entram aqui.
//!
//! Arredondamento: o programa trunca ao centavo (alíquota × base, 3% das doações, 20% do
//! desconto, 12% do PGBL); conferido nos 65 casos de `tests/fixtures/ir-br/dirpf-2026`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Faixa da tabela anual: até `up_to` (centavos; `None` = acima da última), alíquota em
/// décimos de ponto percentual (75 = 7,5%) e parcela a deduzir em centavos.
#[derive(Debug, Clone, Copy)]
struct Bracket {
    up_to: Option<i64>,
    rate_permille: i64,
    deduction: i64,
}

/// Parâmetros do ano-calendário.
#[derive(Debug, Clone)]
pub struct AnnualRules {
    pub year: i32,
    brackets: [Bracket; 5],
    pub simplified_cap: i64,
    pub dependent: i64,
    pub education_cap: i64,
    /// Teto da previdência complementar, em centésimos de ponto percentual dos rendimentos.
    pub private_pension_pct: i64,
    /// Redução anual da Lei 15.270/2025 (a partir do ano-calendário 2026).
    pub reduction: Option<AnnualReduction>,
}

#[derive(Debug, Clone, Copy)]
pub struct AnnualReduction {
    /// Rendimentos até este valor: redução até `max` (centavos).
    pub full_up_to: i64,
    pub max: i64,
    /// Até este valor: `a − b × rendimentos` (a em centavos, b como fração).
    pub phase_out_up_to: i64,
    pub a: i64,
    /// b × 1.000.000 (0,095575 → 95_575).
    pub b_micro: i64,
}

impl AnnualRules {
    /// Regras do ano-calendário (2025 = exercício 2026; 2026 = exercício 2027). Anos sem regra
    /// própria usam a do ano mais próximo conhecido.
    pub fn for_year(year: i32) -> Self {
        let b = |up_to: Option<i64>, rate_permille: i64, deduction: i64| Bracket {
            up_to,
            rate_permille,
            deduction,
        };
        if year >= 2026 {
            // Tabela "a partir do exercício 2027" (Receita, tabelas 2026); Lei 15.270/2025.
            Self {
                year,
                brackets: [
                    b(Some(2_914_560), 0, 0),
                    b(Some(3_391_980), 75, 218_592),
                    b(Some(4_501_260), 150, 472_991),
                    b(Some(5_597_616), 225, 810_585),
                    b(None, 275, 1_090_466),
                ],
                simplified_cap: 1_764_000,
                dependent: 227_508,
                education_cap: 356_150,
                private_pension_pct: 1200,
                reduction: Some(AnnualReduction {
                    full_up_to: 6_000_000,
                    max: 269_415,
                    phase_out_up_to: 8_820_000,
                    a: 842_973,
                    b_micro: 95_575,
                }),
            }
        } else {
            // Exercício 2026 (ano-calendário 2025): Lei 15.191/2025, tabelas da Receita 2025.
            Self {
                year,
                brackets: [
                    b(Some(2_846_720), 0, 0),
                    b(Some(3_391_980), 75, 213_504),
                    b(Some(4_501_260), 150, 467_903),
                    b(Some(5_597_616), 225, 805_497),
                    b(None, 275, 1_085_378),
                ],
                simplified_cap: 1_675_434,
                dependent: 227_508,
                education_cap: 356_150,
                private_pension_pct: 1200,
                reduction: None,
            }
        }
    }

    /// Imposto da tabela anual sobre a base (alíquota × base truncada ao centavo).
    pub fn table_tax(&self, base: i64) -> i64 {
        if base <= 0 {
            return 0;
        }
        let br = self
            .brackets
            .iter()
            .find(|b| b.up_to.is_none_or(|u| base <= u))
            .unwrap_or(&self.brackets[4]);
        (base * br.rate_permille / 1000 - br.deduction).max(0)
    }

    /// Redução anual da Lei 15.270/2025 sobre o imposto da tabela.
    fn reduction(&self, taxable: i64, tax: i64) -> i64 {
        let Some(r) = self.reduction else { return 0 };
        let cut = if taxable <= r.full_up_to {
            r.max
        } else if taxable <= r.phase_out_up_to {
            (r.a - taxable * r.b_micro / 1_000_000).max(0).min(r.max)
        } else {
            0
        };
        cut.min(tax)
    }
}

/// Rendimento tributável de pessoa jurídica (salário, aposentadoria, pró-labore), sem o 13º.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AnnualEmployment")]
pub struct Employment {
    #[ts(type = "number")]
    pub income: i64,
    /// Contribuição previdenciária oficial (INSS) descontada.
    #[ts(type = "number")]
    pub social_security: i64,
    /// IRRF retido.
    #[ts(type = "number")]
    pub withheld: i64,
}

/// Rendimentos de pessoas físicas e do exterior no ano (carnê-leão), já somados.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AnnualSelfEmployed")]
pub struct SelfEmployed {
    #[ts(type = "number")]
    pub income: i64,
    /// A parte recebida do exterior (para o limite do imposto pago lá fora).
    #[ts(type = "number")]
    pub abroad: i64,
    #[ts(type = "number")]
    pub social_security: i64,
    #[ts(type = "number")]
    pub alimony: i64,
    #[ts(type = "number")]
    pub livro_caixa: i64,
    /// Carnê-leão pago.
    #[ts(type = "number")]
    pub carne_leao: i64,
}

/// Dados da declaração do ano, em centavos.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AnnualInput")]
pub struct Input {
    #[ts(type = "number")]
    pub year: i32,
    pub employment: Vec<Employment>,
    pub self_employed: SelfEmployed,
    #[ts(type = "number")]
    pub dependents: u32,
    /// Despesas de saúde dedutíveis (já sem o reembolsado).
    #[ts(type = "number")]
    pub medical: i64,
    /// Despesas de instrução por pessoa (titular e cada dependente).
    #[ts(type = "Array<number>")]
    pub education: Vec<i64>,
    /// Contribuições a PGBL/Fapi.
    #[ts(type = "number")]
    pub private_pension: i64,
    /// Pensão alimentícia judicial ou por escritura pública.
    #[ts(type = "number")]
    pub alimony: i64,
    /// Doações na declaração: fundo da criança e do adolescente e fundo da pessoa idosa.
    #[ts(type = "number")]
    pub donation_children: i64,
    #[ts(type = "number")]
    pub donation_elderly: i64,
    /// Imposto complementar pago no ano.
    #[ts(type = "number")]
    pub complementary_tax: i64,
    /// Imposto pago no exterior sobre rendimentos do trabalho (carnê-leão de fora).
    #[ts(type = "number")]
    pub foreign_tax_paid: i64,
    /// IRRF da Lei 11.033/2004 ("dedo-duro") não compensado no ano.
    #[ts(type = "number")]
    pub irrf_lei_11033: i64,
}

/// Um modelo da declaração.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AnnualModel")]
pub struct Model {
    #[ts(type = "number")]
    pub deductions: i64,
    #[ts(type = "number")]
    pub base: i64,
    /// Imposto da tabela, antes da redução e das doações.
    #[ts(type = "number")]
    pub table_tax: i64,
    /// Redução anual da Lei 15.270/2025 (a partir de 2026).
    #[ts(type = "number")]
    pub reduction: i64,
    /// Doações deduzidas do imposto (só no completo).
    #[ts(type = "number")]
    pub donations: i64,
    #[ts(type = "number")]
    pub tax_due: i64,
    /// Imposto já pago: IRRF, carnê-leão, complementar, exterior (com limite), Lei 11.033.
    #[ts(type = "number")]
    pub tax_paid: i64,
    #[ts(type = "number")]
    pub to_pay: i64,
    #[ts(type = "number")]
    pub refund: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AnnualResult")]
pub struct Output {
    #[ts(type = "number")]
    pub year: i32,
    #[ts(type = "number")]
    pub taxable_income: i64,
    pub full: Model,
    pub simplified: Model,
    /// "completo" ou "simplificado": o de menor imposto devido (empate: completo).
    pub best: String,
    /// Imposto devido do melhor modelo ÷ rendimentos tributáveis, em centésimos de ponto
    /// percentual, truncado (457 = 4,57%).
    #[ts(type = "number")]
    pub effective_rate_bp: i64,
}

fn pct(v: i64, hundredths: i64) -> i64 {
    v * hundredths / 10_000
}

/// Calcula os dois modelos da declaração.
pub fn compute(input: &Input) -> Output {
    let r = AnnualRules::for_year(input.year);
    let se = &input.self_employed;
    let employment: i64 = input.employment.iter().map(|e| e.income).sum();
    let taxable = employment + se.income;
    let withheld: i64 = input.employment.iter().map(|e| e.withheld).sum();
    let base_paid = withheld + se.carne_leao + input.complementary_tax + input.irrf_lei_11033;

    // Completo.
    let social: i64 = input
        .employment
        .iter()
        .map(|e| e.social_security)
        .sum::<i64>()
        + se.social_security;
    let education: i64 = input
        .education
        .iter()
        .map(|v| (*v).clamp(0, r.education_cap))
        .sum();
    let pension = input
        .private_pension
        .clamp(0, pct(taxable, r.private_pension_pct));
    let full_deductions = social
        + r.dependent * input.dependents as i64
        + education
        + input.medical.max(0)
        + input.alimony.max(0)
        + se.alimony.max(0)
        + pension
        + se.livro_caixa.max(0);
    // Desconto simplificado.
    let simplified_of = |income: i64| (income * 20 / 100).min(r.simplified_cap);
    let simple_discount = simplified_of(taxable);

    let model = |deductions: i64, base_without_foreign: i64, donations_allowed: bool| {
        let base = (taxable - deductions).max(0);
        let table = r.table_tax(base);
        let reduction = r.reduction(taxable, table);
        let after = table - reduction;
        let donations = if donations_allowed {
            let cap = after * 3 / 100;
            input.donation_children.clamp(0, cap) + input.donation_elderly.clamp(0, cap)
        } else {
            0
        };
        let tax_due = (after - donations).max(0);
        // Imposto pago lá fora: até o imposto que o rendimento de fora acrescenta.
        let foreign = if input.foreign_tax_paid > 0 && se.abroad > 0 {
            let without = r.table_tax(base_without_foreign);
            input.foreign_tax_paid.min((table - without).max(0))
        } else {
            0
        };
        let paid = base_paid + foreign;
        Model {
            deductions,
            base,
            table_tax: table,
            reduction,
            donations,
            tax_due,
            tax_paid: paid,
            to_pay: (tax_due - paid).max(0),
            refund: (paid - tax_due).max(0),
        }
    };
    let full = model(
        full_deductions,
        (taxable - se.abroad - full_deductions).max(0),
        true,
    );
    let simplified = model(
        simple_discount,
        (taxable - se.abroad - simplified_of(taxable - se.abroad)).max(0),
        false,
    );
    // Empate: o programa indica o completo (casos de referência empate-modelos e isento).
    let best_full = full.tax_due <= simplified.tax_due;
    let best_tax = if best_full {
        full.tax_due
    } else {
        simplified.tax_due
    };
    Output {
        year: input.year,
        taxable_income: taxable,
        effective_rate_bp: if taxable > 0 {
            best_tax * 10_000 / taxable
        } else {
            0
        },
        best: if best_full {
            "completo"
        } else {
            "simplificado"
        }
        .into(),
        full,
        simplified,
    }
}

/// Imposto mínimo das altas rendas (IRPFM, Lei 9.250/1995 art. 16-A, incluído pela Lei
/// 15.270/2025), a partir do ano-calendário 2026: quem soma mais de R$ 600 mil de rendimentos
/// no ano (inclusive isentos e de tributação exclusiva, menos as exclusões do §1º) paga
/// alíquota de (rendimentos ÷ 60.000 − 10)%, até 10% a partir de R$ 1,2 milhão, sobre esse
/// total, menos o IR da declaração, o retido exclusivamente na fonte, o da Lei 14.754 e outros
/// definitivos (§3º, I a IV); depois, menos os 10% retidos sobre dividendos (§5º). O redutor do
/// art. 16-B (carga da empresa + a da pessoa acima de 34%) depende do balanço de cada empresa e
/// não é calculado: o valor é um teto.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "IrpfmInput")]
pub struct MinimumTaxInput {
    #[ts(type = "number")]
    pub year: i32,
    /// Soma dos rendimentos do ano que entram na base (§1º).
    #[ts(type = "number")]
    pub total_income: i64,
    /// IR devido na declaração de ajuste (art. 12).
    #[ts(type = "number")]
    pub annual_tax: i64,
    /// IR retido exclusivamente na fonte sobre rendimentos da base (13º, PLR, JCP, aplicações).
    #[ts(type = "number")]
    pub exclusive_tax: i64,
    /// IR definitivo sobre ganhos em bolsa e o da Lei 14.754 (exterior).
    #[ts(type = "number")]
    pub definitive_tax: i64,
    /// Retenção de 10% sobre dividendos (art. 6º-A).
    #[ts(type = "number")]
    pub dividend_withheld: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "Irpfm")]
pub struct MinimumTax {
    pub applies: bool,
    #[ts(type = "number")]
    pub total_income: i64,
    /// Alíquota em centésimos de ponto percentual (250 = 2,5%).
    #[ts(type = "number")]
    pub rate_bp: i64,
    /// Alíquota × rendimentos.
    #[ts(type = "number")]
    pub gross: i64,
    /// Deduções do §3º (IR da declaração, exclusivo e definitivo).
    #[ts(type = "number")]
    pub credits: i64,
    /// Depois dos 10% retidos sobre dividendos; negativo = retido a mais (a compensar).
    #[ts(type = "number")]
    pub due: i64,
}

/// Limite de rendimentos para o IRPFM e onde a alíquota chega a 10% (centavos).
const IRPFM_FROM: i64 = 60_000_000;
const IRPFM_FULL: i64 = 120_000_000;

pub fn minimum_tax(i: &MinimumTaxInput) -> MinimumTax {
    let applies = i.year >= 2026 && i.total_income > IRPFM_FROM;
    if !applies {
        return MinimumTax {
            applies,
            total_income: i.total_income,
            rate_bp: 0,
            gross: 0,
            credits: 0,
            due: 0,
        };
    }
    // (REND ÷ 60.000 − 10)%, em centésimos de ponto: REND(centavos) ÷ 60.000,00 × 100 − 1.000.
    let rate_bp = if i.total_income >= IRPFM_FULL {
        1000
    } else {
        (i.total_income * 100 / 6_000_000 - 1000).clamp(0, 1000)
    };
    let gross = i.total_income * rate_bp / 10_000;
    let credits = i.annual_tax + i.exclusive_tax + i.definitive_tax;
    let after = (gross - credits).max(0);
    MinimumTax {
        applies,
        total_income: i.total_income,
        rate_bp,
        gross,
        credits,
        due: after - i.dividend_withheld,
    }
}
