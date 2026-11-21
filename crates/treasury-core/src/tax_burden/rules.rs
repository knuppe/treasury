//! Regras de imposto sobre a renda e contribuições do trabalho, genéricas e alimentadas
//! pelos dados do país (`dataset`): tabela progressiva com parcela a deduzir (IRPF mensal),
//! faixas progressivas por fatia (INSS do empregado, com teto) e o redutor de 2026 (isenção
//! até um limite e redução decrescente até outro). Valores em centavos; alíquotas em %.
//!
//! O inverso (`gross_from_net`) acha o bruto mensal que, descontados INSS e IRRF, dá o
//! líquido recebido: é como a estimativa funciona quando só o salário líquido está no app.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Faixa da tabela progressiva do IR: até `up_to` (centavos; `None` = sem limite), alíquota e
/// parcela a deduzir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IrBracket {
    #[ts(type = "number | null")]
    pub up_to: Option<i64>,
    pub rate: f64,
    #[ts(type = "number")]
    pub deduction: i64,
}

/// Faixa de contribuição progressiva (por fatia): alíquota sobre a parte do salário até
/// `up_to`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ContributionBand {
    #[ts(type = "number")]
    pub up_to: i64,
    pub rate: f64,
}

/// Redução do imposto mensal (Lei 15.270/2025, art. 3º-A da Lei 9.250, a partir de 2026):
/// até `exempt_up_to` de renda tributável bruta, redução de até `max_reduction`; entre ele e
/// `phase_out_up_to`, redução de `a − b × renda`; sempre limitada ao imposto calculado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IrReducer {
    #[ts(type = "number")]
    pub exempt_up_to: i64,
    #[ts(type = "number")]
    pub max_reduction: i64,
    #[ts(type = "number")]
    pub phase_out_up_to: i64,
    /// Parcela fixa da redução, em centavos.
    #[ts(type = "number")]
    pub a: i64,
    /// Fator sobre a renda tributável (ex.: 0,133145).
    pub b: f64,
}

/// Mínimo de existência como abatimento ao rendimento (Portugal, CIRS art. 70.º, n.os 2 e 3):
/// até o valor de referência, o abatimento leva o rendimento coletável a `despesas gerais ÷
/// taxa do 1.º escalão`; daí até `L`, cai `slope` por euro de bruto; acima de `L`, cai
/// `upper_slope` até zerar. `L = referência − despesas gerais ÷ taxa ÷ divisor + limite do
/// 1.º escalão ÷ divisor`. A taxa e o limite do 1.º escalão saem da própria tabela. Os dois
/// valores em centavos são **anuais**: o cálculo divide pelos pagamentos do ano.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExistenceRelief {
    /// Valor de referência do ano (12 880 € em 2026).
    #[ts(type = "number")]
    pub reference: i64,
    /// Limite anual da dedução de despesas gerais familiares que a fórmula pressupõe (250 €).
    #[ts(type = "number")]
    pub general_expenses: i64,
    /// Queda do abatimento por euro de bruto entre a referência e `L` (2,60).
    pub slope: f64,
    /// Queda acima de `L` (1,35).
    pub upper_slope: f64,
    /// Divisor da fórmula de `L` (3,60).
    pub divisor: f64,
}

impl ExistenceRelief {
    /// O abatimento do período para um bruto e as deduções específicas já aplicadas.
    fn relief(&self, gross: i64, deductions: i64, r: &PayrollRules) -> i64 {
        let Some(first) = r.ir_monthly.first() else {
            return 0;
        };
        let (Some(le), true) = (first.up_to, first.rate > 0.0) else {
            return 0;
        };
        let periods = r.periods.max(1) as f64;
        let (rb, de, le) = (gross as f64, deductions as f64, le as f64);
        let vr = self.reference as f64 / periods;
        let expenses = self.general_expenses as f64 / periods / (first.rate / 100.0);
        let l = vr - expenses / self.divisor + le / self.divisor;
        let relief = if rb <= vr {
            vr - (de + expenses)
        } else if rb <= l {
            vr - self.slope * (rb - vr) - (de + expenses)
        } else {
            l - le - self.upper_slope * (rb - l) - de
        };
        round(relief).clamp(0, (gross - deductions).max(0))
    }
}

/// Retirada gradual do abatimento por dependente pela renda do ano (Estados Unidos, IRC
/// § 24(b)): o total cai `cut` a cada `step` (ou fração) de renda acima de `from`. Valores
/// **anuais**, em centavos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreditPhaseOut {
    #[ts(type = "number")]
    pub from: i64,
    #[ts(type = "number")]
    pub step: i64,
    #[ts(type = "number")]
    pub cut: i64,
}

/// O que muda nas regras conforme o estado civil de quem recebe: a tabela e o desconto de quem
/// sustenta dependentes sem cônjuge (Estados Unidos: head of household) ou declara em conjunto
/// (married filing jointly), o abatimento fixo (Portugal: 335 € de despesas gerais da família
/// monoparental) e a renda em que o abatimento por dependente começa a cair. O que não vem
/// fica como nas regras gerais.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FilingRules {
    #[serde(default)]
    #[ts(optional)]
    pub ir_monthly: Option<Vec<IrBracket>>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub simplified_discount: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub personal_credit: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub dependent_credit_phase_out: Option<CreditPhaseOut>,
}

/// Regras a partir de um mês: IRPF mensal e INSS do empregado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayrollRules {
    /// Primeiro mês em que valem (`YYYY-MM`).
    pub from: String,
    /// Lei ou ato que dá os números.
    pub legal_basis: String,
    pub ir_monthly: Vec<IrBracket>,
    /// Dedução mensal por dependente (centavos).
    #[ts(type = "number")]
    pub dependent_deduction: i64,
    /// Desconto simplificado mensal (substitui as deduções legais quando maior).
    #[ts(type = "number")]
    pub simplified_discount: i64,
    #[ts(optional)]
    pub reducer: Option<IrReducer>,
    pub inss_bands: Vec<ContributionBand>,
    /// Fontes destes números (ids da lista de fontes do país).
    pub source_ids: Vec<String>,
    /// País-membro a que as regras valem, nos dados da União Europeia (`PT`).
    #[serde(default)]
    #[ts(optional)]
    pub region: Option<String>,
    /// A contribuição do empregado sai da base do imposto (Brasil e Portugal: sim; o FICA
    /// americano, não).
    #[serde(default = "yes")]
    pub contribution_deductible: bool,
    /// Abatimento no imposto por dependente, por período (crédito por filho nos EUA, dedução
    /// à coleta em Portugal), em centavos.
    #[serde(default)]
    #[ts(type = "number")]
    pub dependent_credit: i64,
    /// Mínimo de existência como abatimento ao rendimento (Portugal).
    #[serde(default)]
    #[ts(optional)]
    pub existence: Option<ExistenceRelief>,
    /// Retirada do abatimento por dependente acima de uma renda (Estados Unidos).
    #[serde(default)]
    #[ts(optional)]
    pub dependent_credit_phase_out: Option<CreditPhaseOut>,
    /// Regras de quem sustenta dependentes sem cônjuge (Estados Unidos e Portugal).
    #[serde(default)]
    #[ts(optional)]
    pub single_parent: Option<FilingRules>,
    /// Regras de quem declara em conjunto com um cônjuge sem renda (Estados Unidos).
    #[serde(default)]
    #[ts(optional)]
    pub joint: Option<FilingRules>,
    /// Parte da contribuição que **não** sai da base do imposto, por fatia do bruto como em
    /// `inss_bands` (França: CSG e CRDS não dedutíveis; Alemanha: seguro-desemprego e a parte
    /// do seguro-saúde que paga o auxílio-doença).
    #[serde(default)]
    #[ts(optional)]
    pub non_deductible_bands: Option<Vec<ContributionBand>>,
    /// Abatimento no imposto pelo bruto do período, em pontos `[bruto até, abatimento]` com
    /// interpolação linear e zero depois do último (Espanha: "deducción por obtención de
    /// rendimientos del trabajo" de quem ganha perto do salário mínimo).
    #[serde(default)]
    #[ts(optional, type = "Array<[number, number]>")]
    pub earned_credit: Option<Vec<[i64; 2]>>,
    /// Pagamentos de salário por ano em que a tabela foi dividida (Portugal e Espanha: 14).
    #[serde(default = "twelve")]
    #[ts(type = "number")]
    pub periods: u32,
    /// Dedução fixa da base por período, somada às legais (Espanha: os 2.000 € de "otros gastos
    /// deducibles"), em centavos.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub fixed_deduction: Option<i64>,
    /// Abatimento fixo no imposto por período (Espanha: o mínimo pessoal de 5.550 € tributado
    /// a 19%), em centavos.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub personal_credit: Option<i64>,
    /// Redução da base pelo rendimento líquido do trabalho (bruto − contribuição), por
    /// período, em pontos `[líquido até, redução]` com interpolação linear e zero depois do
    /// último (Espanha: "reducción por obtención de rendimientos del trabajo").
    #[serde(default)]
    #[ts(optional, type = "Array<[number, number]>")]
    pub work_reduction: Option<Vec<[i64; 2]>>,
}

impl PayrollRules {
    fn with(&self, f: Option<FilingRules>) -> PayrollRules {
        let mut r = self.clone();
        r.single_parent = None;
        r.joint = None;
        let Some(f) = f else {
            return r;
        };
        if let Some(t) = f.ir_monthly {
            r.ir_monthly = t;
        }
        if let Some(d) = f.simplified_discount {
            r.simplified_discount = d;
        }
        if f.personal_credit.is_some() {
            r.personal_credit = f.personal_credit;
        }
        if f.dependent_credit_phase_out.is_some() {
            r.dependent_credit_phase_out = f.dependent_credit_phase_out;
        }
        r
    }

    /// As regras de quem sustenta dependentes sem cônjuge, quando o país tem regras próprias
    /// (senão, as gerais).
    pub fn for_single_parent(&self) -> PayrollRules {
        self.with(self.single_parent.clone())
    }

    /// As regras de quem declara em conjunto com um cônjuge sem renda, quando o país as tem.
    pub fn for_joint(&self) -> PayrollRules {
        self.with(self.joint.clone())
    }

    /// As regras gerais (pessoa solteira, ou casada cujo cônjuge também tem renda).
    pub fn for_single(&self) -> PayrollRules {
        self.with(None)
    }

    /// Abatimento do período pelos dependentes, já com a retirada pela renda do ano.
    fn dependents_credit(&self, gross: i64, dependents: u32) -> i64 {
        let full = self.dependent_credit * dependents as i64;
        let Some(p) = &self.dependent_credit_phase_out else {
            return full;
        };
        let periods = self.periods.max(1) as i64;
        // O bruto do período vem arredondado ao centavo: a folga de um centavo por pagamento
        // evita contar um degrau a mais por causa disso.
        let over = gross * periods - p.from - periods;
        if over <= 0 || p.step <= 0 {
            return full;
        }
        // "ou fração": arredonda o número de degraus para cima.
        let steps = (over + p.step - 1) / p.step;
        (full - round((steps * p.cut) as f64 / periods as f64)).max(0)
    }
}

/// A redução da base para um líquido do trabalho (ver `PayrollRules::work_reduction`); a
/// mesma interpolação serve ao abatimento pelo bruto (`PayrollRules::earned_credit`).
pub fn work_reduction(net: i64, points: &[[i64; 2]]) -> i64 {
    let Some(first) = points.first() else {
        return 0;
    };
    if net <= first[0] {
        return first[1];
    }
    for w in points.windows(2) {
        let ([x0, y0], [x1, y1]) = (w[0], w[1]);
        if net <= x1 {
            let f = (net - x0) as f64 / (x1 - x0).max(1) as f64;
            return round(y0 as f64 + (y1 - y0) as f64 * f).max(0);
        }
    }
    0
}

fn yes() -> bool {
    true
}

fn twelve() -> u32 {
    12
}

fn round(v: f64) -> i64 {
    v.round() as i64
}

/// Contribuição progressiva por fatia, limitada ao teto (a última faixa).
pub fn contribution(gross: i64, bands: &[ContributionBand]) -> i64 {
    let mut prev = 0;
    let mut total = 0.0;
    for b in bands {
        if gross <= prev {
            break;
        }
        let slice = gross.min(b.up_to) - prev;
        total += slice as f64 * b.rate / 100.0;
        prev = b.up_to;
    }
    round(total)
}

/// IR pela tabela progressiva sobre a base (já descontados INSS e deduções).
pub fn ir_table(base: i64, brackets: &[IrBracket]) -> i64 {
    if base <= 0 {
        return 0;
    }
    let b = brackets
        .iter()
        .find(|b| b.up_to.is_none_or(|u| base <= u))
        .or_else(|| brackets.last());
    match b {
        Some(b) => (round(base as f64 * b.rate / 100.0) - b.deduction).max(0),
        None => 0,
    }
}

/// IRRF mensal sobre o salário do mês: base = bruto − INSS − deduções (o maior entre as
/// legais e o desconto simplificado), com o redutor sobre a renda tributável quando houver.
/// Fora do Brasil (docs/countries.md): a contribuição pode não sair da base, toda (FICA) ou
/// em parte (CSG/CRDS na França); o dependente pode abater no imposto, com retirada pela
/// renda (crédito por filho nos EUA); o mínimo de existência abate o rendimento (Portugal);
/// e pode haver dedução e abatimentos fixos ou pelo bruto (Espanha).
pub fn monthly_irrf(gross: i64, inss: i64, dependents: u32, r: &PayrollRules) -> i64 {
    let contribution = if r.contribution_deductible {
        let kept = r
            .non_deductible_bands
            .as_deref()
            .map_or(0, |b| contribution(gross, b));
        (inss - kept).max(0)
    } else {
        0
    };
    let legal = contribution
        + r.dependent_deduction * dependents as i64
        + r.fixed_deduction.unwrap_or(0)
        + r.work_reduction
            .as_deref()
            .map_or(0, |p| work_reduction(gross - inss, p));
    let deductions = legal.max(r.simplified_discount);
    let relief = r
        .existence
        .as_ref()
        .map_or(0, |e| e.relief(gross, deductions, r));
    let tax = ir_table(gross - deductions - relief, &r.ir_monthly);
    let cut = match &r.reducer {
        Some(red) if gross <= red.exempt_up_to => red.max_reduction,
        Some(red) if gross < red.phase_out_up_to => (red.a - round(red.b * gross as f64)).max(0),
        _ => 0,
    };
    let tax = (tax - cut.min(tax)).max(0);
    let credits = r.dependents_credit(gross, dependents)
        + r.personal_credit.unwrap_or(0)
        + r.earned_credit
            .as_deref()
            .map_or(0, |p| work_reduction(gross, p));
    (tax - credits).max(0)
}

/// Descontos do mês para um bruto: INSS e IRRF.
pub fn payroll(gross: i64, dependents: u32, r: &PayrollRules) -> (i64, i64) {
    let inss = contribution(gross, &r.inss_bands);
    (inss, monthly_irrf(gross, inss, dependents, r))
}

/// Bruto mensal que, descontados INSS e IRRF, dá `net` (busca binária; o líquido cresce
/// com o bruto).
pub fn gross_from_net(net: i64, dependents: u32, r: &PayrollRules) -> i64 {
    if net <= 0 {
        return 0;
    }
    let net_of = |g: i64| {
        let (inss, irrf) = payroll(g, dependents, r);
        g - inss - irrf
    };
    let (mut lo, mut hi) = (net, net * 2 + 1);
    while net_of(hi) < net {
        hi *= 2;
    }
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if net_of(mid) < net {
            lo = mid
        } else {
            hi = mid
        }
    }
    if net_of(lo) >= net { lo } else { hi }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tabelas de 2025 (Lei 15.191/2025, a partir de maio; INSS da Portaria MPS/MF 6/2025),
    /// em centavos: só para conferir o cálculo.
    fn r2025() -> PayrollRules {
        PayrollRules {
            from: "2025-05".into(),
            legal_basis: String::new(),
            ir_monthly: vec![
                IrBracket {
                    up_to: Some(242_880),
                    rate: 0.0,
                    deduction: 0,
                },
                IrBracket {
                    up_to: Some(282_665),
                    rate: 7.5,
                    deduction: 18_216,
                },
                IrBracket {
                    up_to: Some(375_105),
                    rate: 15.0,
                    deduction: 39_416,
                },
                IrBracket {
                    up_to: Some(466_468),
                    rate: 22.5,
                    deduction: 67_549,
                },
                IrBracket {
                    up_to: None,
                    rate: 27.5,
                    deduction: 90_873,
                },
            ],
            dependent_deduction: 18_959,
            simplified_discount: 60_720,
            reducer: None,
            inss_bands: vec![
                ContributionBand {
                    up_to: 151_800,
                    rate: 7.5,
                },
                ContributionBand {
                    up_to: 279_388,
                    rate: 9.0,
                },
                ContributionBand {
                    up_to: 419_083,
                    rate: 12.0,
                },
                ContributionBand {
                    up_to: 815_741,
                    rate: 14.0,
                },
            ],
            source_ids: vec![],
            region: None,
            contribution_deductible: true,
            dependent_credit: 0,
            existence: None,
            dependent_credit_phase_out: None,
            single_parent: None,
            joint: None,
            non_deductible_bands: None,
            earned_credit: None,
            periods: 12,
            fixed_deduction: None,
            personal_credit: None,
            work_reduction: None,
        }
    }

    #[test]
    fn inss_is_progressive_by_slice_and_capped() {
        let r = r2025();
        assert_eq!(contribution(151_800, &r.inss_bands), 11_385);
        // 1518 × 7,5% + (2793,88 − 1518) × 9% + (3000 − 2793,88) × 12%
        assert_eq!(
            contribution(300_000, &r.inss_bands),
            11_385 + 11_483 + 2_473
        );
        let teto = contribution(815_741, &r.inss_bands);
        assert_eq!(contribution(2_000_000, &r.inss_bands), teto);
    }

    #[test]
    fn irrf_uses_the_larger_deduction_and_the_bracket() {
        let r = r2025();
        // R$ 10.000: INSS no teto; base = 10000 − INSS; 27,5% − 908,73.
        let (inss, irrf) = payroll(1_000_000, 0, &r);
        let base = 1_000_000 - inss;
        assert_eq!(irrf, (base as f64 * 0.275).round() as i64 - 90_873);
        // R$ 2.500: o desconto simplificado (607,20) é maior que o INSS.
        let (_, irrf) = payroll(250_000, 0, &r);
        assert_eq!(irrf, 0, "base 1892,80 fica na faixa isenta");
    }

    #[test]
    fn reducer_exempts_up_to_the_limit_and_phases_out() {
        let mut r = r2025();
        r.reducer = Some(IrReducer {
            exempt_up_to: 500_000,
            max_reduction: 31_289,
            phase_out_up_to: 735_000,
            a: 97_862,
            b: 0.133145,
        });
        assert_eq!(payroll(500_000, 0, &r).1, 0);
        let full = {
            let mut s = r.clone();
            s.reducer = None;
            payroll(900_000, 0, &s).1
        };
        assert_eq!(
            payroll(900_000, 0, &r).1,
            full,
            "acima do limite, sem redução"
        );
        let mid = payroll(600_000, 0, &r).1;
        assert!(
            mid > 0
                && mid
                    < payroll(600_000, 0, &{
                        let mut s = r.clone();
                        s.reducer = None;
                        s
                    })
                    .1
        );
    }

    #[test]
    fn gross_from_net_inverts_payroll() {
        let r = r2025();
        for gross in [150_000, 300_000, 650_000, 1_200_000, 3_000_000] {
            let (inss, irrf) = payroll(gross, 0, &r);
            let net = gross - inss - irrf;
            let back = gross_from_net(net, 0, &r);
            let (i2, r2) = payroll(back, 0, &r);
            assert!(
                (back - i2 - r2 - net).abs() <= 1,
                "{gross} → {net} → {back}"
            );
            assert!((back - gross).abs() <= 2, "{gross} ≈ {back}");
        }
    }
}
