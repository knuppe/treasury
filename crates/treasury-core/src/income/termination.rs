//! Simulação de demissão (Brasil): as verbas da rescisão pelo motivo, os descontos de INSS e
//! IRRF, o FGTS (depósito do mês, multa e saque) e o seguro-desemprego (direito, valor e
//! parcelas). É estimativa: a rescisão oficial é a do empregador (TRCT), e convenções
//! coletivas, médias de variáveis, faltas e adiantamentos não entram.
//!
//! Regras (CLT e leis citadas nas linhas):
//! - Aviso prévio: 30 dias + 3 por ano completo, até 90 (Lei 12.506/2011), só quando o
//!   empregador dispensa; no acordo, metade do indenizado (CLT art. 484-A). O indenizado
//!   projeta o contrato para o 13º e as férias (CLT art. 487, §1º; OJ 82 da SDI-1 do TST).
//! - Saldo de salário: salário ÷ 30 × dias do mês até o desligamento.
//! - 13º e férias proporcionais: 1/12 por mês com 15 dias ou mais; com 1/3 nas férias
//!   (CF art. 7º, XVII). Justa causa perde os dois (Lei 4.090/1962, art. 3º; Súmula 171 do
//!   TST); pedido de demissão mantém as férias proporcionais (Súmula 261).
//! - Contrato de experiência terminado antes pelo empregador: metade do que faltava
//!   (CLT art. 479); pelo empregado, desconto de até o mesmo valor (art. 480).
//! - FGTS: 8% do saldo, do 13º e do aviso indenizado (Lei 8.036/1990, art. 15; Súmula 305);
//!   multa de 40% sem justa causa e 20% no acordo (art. 18); saque integral sem justa causa,
//!   no fim de contrato a termo e na quebra pelo empregador, 80% no acordo (art. 20).
//! - INSS e IRRF só sobre o saldo de salário e o 13º (tributado à parte); aviso indenizado,
//!   férias indenizadas com 1/3, indenização do art. 479 e a multa do FGTS são indenizações
//!   (Lei 7.713/1988, art. 6º, V; Súmula 386 do STJ). Tabelas do mês do desligamento.
//! - Seguro-desemprego (Lei 7.998/1990, art. 3º e 4º; Resolução CODEFAT 957/2022): só na
//!   dispensa sem justa causa; carência de 12 meses nos últimos 18 no 1º pedido, 9 nos
//!   últimos 12 no 2º e 6 a partir do 3º; 3 a 5 parcelas pelos meses trabalhados nos últimos
//!   36; valor pela média dos 3 últimos salários, entre o salário mínimo e o teto.

use chrono::{Datelike, Months, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::tax_burden::rules;

/// Motivo do desligamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TerminationReason {
    /// Dispensa sem justa causa.
    SemJustaCausa,
    PedidoDemissao,
    JustaCausa,
    /// Acordo entre as partes (CLT art. 484-A).
    Acordo,
    /// Fim do contrato de experiência na data combinada.
    FimExperiencia,
    /// Contrato de experiência terminado antes pelo empregador (art. 479).
    QuebraExperienciaEmpregador,
    /// Contrato de experiência terminado antes pelo empregado (art. 480).
    QuebraExperienciaEmpregado,
}

/// Como fica o aviso prévio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum NoticeMode {
    /// Pago sem trabalhar (projeta o contrato).
    Indenizado,
    /// Trabalhado até a data informada (já recebido como salário).
    Trabalhado,
    /// Pedido de demissão sem cumprir o aviso: o empregador pode descontar 30 dias.
    NaoCumprido,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct TerminationInput {
    /// Renda (CLT) de onde vêm o salário, a admissão, os dependentes e a conta do FGTS.
    #[serde(default)]
    #[ts(optional)]
    pub source_id: Option<Id>,
    /// Salário bruto mensal (centavos); sem ele, o da renda.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub gross_monthly: Option<i64>,
    /// Admissão (`YYYY-MM-DD`); sem ela, o início da renda ou a admissão no FGTS.
    #[serde(default)]
    #[ts(optional)]
    pub admission_date: Option<String>,
    /// Último dia de trabalho (`YYYY-MM-DD`).
    pub termination_date: String,
    pub reason: TerminationReason,
    /// Padrão: indenizado quando o empregador dispensa, trabalhado no pedido.
    #[serde(default)]
    #[ts(optional)]
    pub notice: Option<NoticeMode>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub dependents: Option<u32>,
    /// Dias de férias vencidas ainda não gozadas (0 a 60).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub overdue_vacation_days: Option<u32>,
    /// Saldo do FGTS para fins rescisórios deste vínculo (centavos); sem ele, o da conta do
    /// FGTS ligada à renda.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub fgts_balance: Option<i64>,
    /// Fim combinado do contrato de experiência (`YYYY-MM-DD`), para a quebra antecipada.
    #[serde(default)]
    #[ts(optional)]
    pub contract_end: Option<String>,
    /// Quantas vezes já recebeu o seguro-desemprego (0 = este é o primeiro pedido).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub previous_claims: Option<u32>,
    /// Meses trabalhados com carteira nos últimos 36 (todos os empregos); sem ele, os deste
    /// vínculo.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub months_worked_36: Option<u32>,
    /// Os últimos salários brutos antes da dispensa (até 3, centavos); sem eles, o salário.
    #[serde(default)]
    #[ts(optional, type = "Array<number>")]
    pub last_salaries: Option<Vec<i64>>,
}

/// Uma verba (positiva) ou um desconto (negativo).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminationLine {
    /// `saldo_salario`, `aviso_previo`, `decimo_terceiro`, `ferias_proporcionais`,
    /// `ferias_vencidas`, `indenizacao_479`, `desconto_aviso`, `desconto_480`, `inss`,
    /// `inss_13`, `irrf`, `irrf_13`.
    pub code: String,
    pub label: String,
    /// Detalhe da conta ("12 dias", "7/12 + 1/3").
    pub detail: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// Entra na base de INSS e IRRF.
    pub taxable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminationFgts {
    /// Saldo informado ou o da conta do FGTS ligada (centavos).
    #[ts(type = "number | null")]
    pub balance: Option<i64>,
    /// Depósito de 8% sobre as verbas da rescisão.
    #[ts(type = "number")]
    pub deposit: i64,
    /// Multa em % (40, 20 ou 0).
    pub fine_rate: u32,
    #[ts(type = "number")]
    pub fine: i64,
    /// Pode sacar o saldo (no todo ou em parte).
    pub withdrawal_allowed: bool,
    /// % do saldo que pode sacar (100, 80 ou 0).
    pub withdrawal_rate: u32,
    /// Saldo + depósito sacáveis, mais a multa (centavos).
    #[ts(type = "number")]
    pub withdrawable: i64,
    /// Modalidade saque-aniversário: o saldo fica retido, só a multa sai.
    pub anniversary_blocks_balance: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnemploymentInsurance {
    pub eligible: bool,
    /// Por que não tem direito.
    pub reason: Option<String>,
    #[ts(type = "number")]
    pub average_salary: i64,
    #[ts(type = "number")]
    pub installment: i64,
    pub installments: u32,
    #[ts(type = "number")]
    pub total: i64,
    /// Ato e vigência da tabela usada.
    pub table: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminationResult {
    pub reason: TerminationReason,
    pub notice: Option<NoticeMode>,
    /// Dias de aviso prévio (o pago, quando indenizado).
    pub notice_days: u32,
    /// Fim do contrato com a projeção do aviso indenizado.
    pub projected_end: String,
    pub years_of_service: u32,
    pub lines: Vec<TerminationLine>,
    /// Verbas antes dos descontos.
    #[ts(type = "number")]
    pub gross: i64,
    /// Descontos (INSS, IRRF e os do aviso ou do art. 480), positivos.
    #[ts(type = "number")]
    pub deductions: i64,
    /// O que a empresa paga na rescisão.
    #[ts(type = "number")]
    pub net: i64,
    pub fgts: TerminationFgts,
    pub unemployment: UnemploymentInsurance,
    /// Rescisão líquida + FGTS sacável + seguro-desemprego.
    #[ts(type = "number")]
    pub total_available: i64,
    /// Quanto do salário líquido de hoje esse total cobre, em meses.
    pub months_covered: Option<f64>,
    pub warnings: Vec<String>,
    /// Base legal das tabelas de INSS e IRRF usadas.
    pub payroll_rules: String,
}

/// Tabela do seguro-desemprego: até `band1`, 80% da média; até `band2`, `fixed` + 50% do que
/// passa de `band1`; acima, o teto; nunca abaixo do salário mínimo.
struct UiTable {
    from: &'static str,
    min_wage: i64,
    band1: i64,
    fixed: i64,
    band2: i64,
    cap: i64,
    source: &'static str,
}

/// Atualizar todo janeiro (reajuste pelo INPC, publicado pelo Ministério do Trabalho).
const UI_TABLES: &[UiTable] = &[
    UiTable {
        from: "2025-01-11",
        min_wage: 151_800,
        band1: 213_876,
        fixed: 171_101,
        band2: 356_496,
        cap: 242_411,
        source: "MTE, tabela de 11/01/2025 (Lei 7.998/1990; Res. CODEFAT 957/2022)",
    },
    UiTable {
        from: "2026-01-11",
        min_wage: 162_100,
        band1: 222_217,
        fixed: 177_774,
        band2: 370_399,
        cap: 251_865,
        source: "MTE, tabela de 11/01/2026 (Lei 7.998/1990; Res. CODEFAT 957/2022)",
    },
];

fn ui_table(day: &str) -> (&'static UiTable, bool) {
    match UI_TABLES.iter().rev().find(|t| t.from <= day) {
        Some(t) => (t, false),
        None => (&UI_TABLES[0], true),
    }
}

/// Valor da parcela para uma média salarial.
pub fn unemployment_installment(average: i64, day: &str) -> i64 {
    let (t, _) = ui_table(day);
    let v = if average <= t.band1 {
        average * 8 / 10
    } else if average <= t.band2 {
        t.fixed + (average - t.band1) / 2
    } else {
        t.cap
    };
    v.clamp(t.min_wage, t.cap)
}

fn date(field: &str, s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map_err(|_| {
        Error::invalid(t_args(
            "nucleo-renda-erro-rescisao-data",
            &args(&[("campo", field.to_string().into())]),
        ))
    })
}

/// Anos completos entre as datas.
fn full_years(from: NaiveDate, to: NaiveDate) -> u32 {
    let mut y = to.year() - from.year();
    if (to.month(), to.day()) < (from.month(), from.day()) {
        y -= 1;
    }
    y.max(0) as u32
}

/// Meses completos entre as datas (de dia a dia).
fn full_months(from: NaiveDate, to: NaiveDate) -> u32 {
    let mut m = (to.year() - from.year()) * 12 + to.month() as i32 - from.month() as i32;
    if to.day() < from.day() {
        m -= 1;
    }
    m.max(0) as u32
}

/// Avos do 13º: meses do ano do fim projetado com 15 dias ou mais de contrato.
fn thirteenth_avos(admission: NaiveDate, end: NaiveDate) -> u32 {
    (1..=end.month())
        .filter(|&m| {
            let first = NaiveDate::from_ymd_opt(end.year(), m, 1).unwrap();
            let last = first
                .checked_add_months(Months::new(1))
                .and_then(|d| d.pred_opt())
                .unwrap();
            let start = first.max(admission);
            let stop = last.min(end);
            stop >= start && (stop - start).num_days() + 1 >= 15
        })
        .count() as u32
}

/// Avos das férias proporcionais: meses do período aquisitivo em curso (desde o último
/// aniversário da admissão), contando a fração de 15 dias ou mais.
fn vacation_avos(admission: NaiveDate, end: NaiveDate) -> u32 {
    let years = full_years(admission, end);
    let start = admission
        .checked_add_months(Months::new(12 * years))
        .unwrap_or(admission);
    let months = full_months(start, end);
    let after = start
        .checked_add_months(Months::new(months))
        .unwrap_or(start);
    let extra = ((end - after).num_days() + 1 >= 15) as u32;
    (months + extra).min(12)
}

/// Salário ÷ 30 × dias, arredondado.
fn per_day(salary: i64, days: i64) -> i64 {
    (salary * days * 2 + 30) / 60
}

fn pct(v: i64, rate: i64) -> i64 {
    (v * rate * 2 + 100) / 200
}

fn line(code: &str, detail: String, amount: i64, taxable: bool) -> TerminationLine {
    TerminationLine {
        code: code.to_string(),
        label: t(&format!("nucleo-renda-rescisao-{}", code.replace('_', "-"))),
        detail,
        amount,
        taxable,
    }
}

/// Dados da renda e da conta do FGTS ligada a ela.
struct Prefill {
    gross: Option<i64>,
    net: Option<i64>,
    admission: Option<String>,
    dependents: Option<u32>,
    fgts_balance: Option<i64>,
    anniversary: bool,
}

fn prefill(conn: &Connection, source_id: &str) -> Result<Prefill> {
    let s = super::get_source(conn, source_id)?;
    let today = crate::time::today_local();
    let fgts = crate::fgts::list(conn, today)?
        .into_iter()
        .find(|a| a.income_source_id.as_deref() == Some(source_id) && !a.archived);
    Ok(Prefill {
        gross: s.gross_monthly,
        net: s.net_monthly,
        admission: s
            .start_date
            .clone()
            .or_else(|| fgts.as_ref().and_then(|a| a.admission_date.clone())),
        dependents: Some(s.dependents),
        fgts_balance: fgts
            .as_ref()
            .map(|a| a.rescission_base.unwrap_or(a.balance)),
        anniversary: fgts.as_ref().and_then(|a| a.modality.as_deref()) == Some("aniversario"),
    })
}

pub fn simulate(conn: &Connection, input: &TerminationInput) -> Result<TerminationResult> {
    crate::fgts::require_brazil(conn)?;
    let pre = match &input.source_id {
        Some(id) => Some(prefill(conn, id)?),
        None => None,
    };
    let salary = input
        .gross_monthly
        .or(pre.as_ref().and_then(|p| p.gross))
        .filter(|g| *g > 0)
        .ok_or_else(|| Error::invalid(t("nucleo-renda-erro-rescisao-salario")))?;
    let admission = input
        .admission_date
        .clone()
        .or(pre.as_ref().and_then(|p| p.admission.clone()))
        .ok_or_else(|| Error::invalid(t("nucleo-renda-erro-rescisao-admissao")))?;
    let admission = date("admission_date", &admission)?;
    let end = date("termination_date", &input.termination_date)?;
    if end < admission {
        return Err(Error::invalid(t("nucleo-renda-erro-rescisao-ordem")));
    }
    let reason = input.reason;
    let dependents = input
        .dependents
        .or(pre.as_ref().and_then(|p| p.dependents))
        .unwrap_or(0);
    let overdue = input.overdue_vacation_days.unwrap_or(0).min(60) as i64;
    let fgts_balance = input
        .fgts_balance
        .or(pre.as_ref().and_then(|p| p.fgts_balance));
    let anniversary = pre.as_ref().is_some_and(|p| p.anniversary);
    let mut warnings = vec![t("nucleo-renda-rescisao-aviso-estimativa")];
    use TerminationReason as R;

    let years = full_years(admission, end);
    let employer_notice = matches!(reason, R::SemJustaCausa | R::Acordo);
    let notice = match reason {
        R::SemJustaCausa | R::Acordo => Some(input.notice.unwrap_or(NoticeMode::Indenizado)),
        R::PedidoDemissao => Some(input.notice.unwrap_or(NoticeMode::Trabalhado)),
        _ => None,
    };
    let full_notice = (30 + 3 * years).min(90);
    let notice_days = match (reason, notice) {
        (R::Acordo, Some(NoticeMode::Indenizado)) => full_notice / 2,
        (_, _) if employer_notice => full_notice,
        (R::PedidoDemissao, _) => 30,
        _ => 0,
    };
    let indemnified = employer_notice && notice == Some(NoticeMode::Indenizado);
    let projected = if indemnified {
        end + chrono::Days::new(notice_days as u64)
    } else {
        end
    };

    let mut lines = vec![];
    // Saldo de salário: os dias do mês até o desligamento (mês de 30 dias).
    let days = end.day().min(30) as i64;
    let saldo = per_day(salary, days);
    lines.push(line(
        "saldo_salario",
        t_args("nucleo-renda-rescisao-n-dias", &args(&[("n", days.into())])),
        saldo,
        true,
    ));
    let mut aviso = 0;
    if indemnified {
        aviso = per_day(salary, notice_days as i64);
        lines.push(line(
            "aviso_previo",
            t_args(
                "nucleo-renda-rescisao-n-dias",
                &args(&[("n", i64::from(notice_days).into())]),
            ),
            aviso,
            false,
        ));
    }
    let mut thirteenth = 0;
    if reason != R::JustaCausa {
        let avos = thirteenth_avos(admission, projected);
        thirteenth = salary * avos as i64 / 12;
        if avos > 0 {
            lines.push(line(
                "decimo_terceiro",
                format!("{avos}/12"),
                thirteenth,
                true,
            ));
        }
        if projected.month() == 12 {
            warnings.push(t("nucleo-renda-rescisao-aviso-13-adiantado"));
        }
        let vavos = vacation_avos(admission, projected);
        if vavos > 0 {
            let base = salary * vavos as i64 / 12;
            lines.push(line(
                "ferias_proporcionais",
                format!("{vavos}/12 + 1/3"),
                base + base / 3,
                false,
            ));
        }
    }
    if overdue > 0 {
        let base = per_day(salary, overdue);
        lines.push(line(
            "ferias_vencidas",
            t_args(
                "nucleo-renda-rescisao-n-dias-terco",
                &args(&[("n", overdue.into())]),
            ),
            base + base / 3,
            false,
        ));
    }
    let mut discounts = 0;
    if matches!(
        reason,
        R::QuebraExperienciaEmpregador | R::QuebraExperienciaEmpregado
    ) {
        let contract_end = input
            .contract_end
            .as_deref()
            .ok_or_else(|| Error::invalid(t("nucleo-renda-erro-rescisao-fim-contrato")))?;
        let contract_end = date("contract_end", contract_end)?;
        let left = (contract_end - end).num_days().max(0);
        let half = per_day(salary, left) / 2;
        if reason == R::QuebraExperienciaEmpregador {
            lines.push(line(
                "indenizacao_479",
                t_args(
                    "nucleo-renda-rescisao-metade-dias",
                    &args(&[("n", left.into())]),
                ),
                half,
                false,
            ));
        } else {
            discounts += half;
            lines.push(line(
                "desconto_480",
                t_args(
                    "nucleo-renda-rescisao-metade-dias",
                    &args(&[("n", left.into())]),
                ),
                -half,
                false,
            ));
        }
    }
    if reason == R::PedidoDemissao && notice == Some(NoticeMode::NaoCumprido) {
        discounts += salary;
        lines.push(line(
            "desconto_aviso",
            t_args("nucleo-renda-rescisao-n-dias", &args(&[("n", 30.into())])),
            -salary,
            false,
        ));
    }

    // INSS e IRRF: tabelas do mês do desligamento; o 13º à parte.
    let ds = crate::tax_burden::dataset::get("BR")?;
    let month = end.format("%Y-%m").to_string();
    let rules = crate::tax_burden::rules_for(&ds, &month, None)
        .ok_or_else(|| Error::invalid(t("nucleo-renda-erro-rescisao-tabela")))?;
    let (inss, irrf) = rules::payroll(saldo, dependents, rules);
    let (inss13, irrf13) = rules::payroll(thirteenth, dependents, rules);
    for (code, v) in [
        ("inss", inss),
        ("irrf", irrf),
        ("inss_13", inss13),
        ("irrf_13", irrf13),
    ] {
        if v > 0 {
            lines.push(line(code, String::new(), -v, false));
        }
    }
    let gross: i64 = lines
        .iter()
        .filter(|l| l.amount > 0)
        .map(|l| l.amount)
        .sum();
    let deductions = discounts + inss + irrf + inss13 + irrf13;
    let net = gross - deductions;

    // FGTS.
    let deposit = pct(saldo + thirteenth + aviso, 8);
    let fine_rate: u32 = match reason {
        R::SemJustaCausa | R::QuebraExperienciaEmpregador => 40,
        R::Acordo => 20,
        _ => 0,
    };
    let fine_base = fgts_balance.unwrap_or(0) + deposit;
    let fine = pct(fine_base, fine_rate as i64);
    let withdrawal_rate: u32 = match reason {
        R::SemJustaCausa | R::QuebraExperienciaEmpregador | R::FimExperiencia => 100,
        R::Acordo => 80,
        _ => 0,
    };
    let balance_out = if anniversary {
        0
    } else {
        pct(fine_base, withdrawal_rate as i64)
    };
    if fgts_balance.is_none() {
        warnings.push(t("nucleo-renda-rescisao-aviso-sem-fgts"));
    }
    if anniversary && withdrawal_rate > 0 {
        warnings.push(t("nucleo-renda-rescisao-aviso-aniversario"));
    }
    let fgts = TerminationFgts {
        balance: fgts_balance,
        deposit,
        fine_rate,
        fine,
        withdrawal_allowed: withdrawal_rate > 0 && !anniversary,
        withdrawal_rate,
        withdrawable: balance_out + fine,
        anniversary_blocks_balance: anniversary && withdrawal_rate > 0,
    };

    let unemployment = unemployment(input, reason, admission, end, salary);
    let total_available = net + fgts.withdrawable + unemployment.total;
    let monthly_net = pre
        .as_ref()
        .and_then(|p| p.net)
        .filter(|n| *n > 0)
        .or_else(|| {
            let (i, r) = rules::payroll(salary, dependents, rules);
            Some(salary - i - r).filter(|n| *n > 0)
        });
    Ok(TerminationResult {
        reason,
        notice,
        notice_days,
        projected_end: projected.to_string(),
        years_of_service: years,
        lines,
        gross,
        deductions,
        net,
        fgts,
        unemployment,
        total_available,
        months_covered: monthly_net
            .map(|n| (total_available as f64 / n as f64 * 10.0).round() / 10.0),
        warnings,
        payroll_rules: rules.legal_basis.clone(),
    })
}

fn unemployment(
    input: &TerminationInput,
    reason: TerminationReason,
    admission: NaiveDate,
    end: NaiveDate,
    salary: i64,
) -> UnemploymentInsurance {
    let day = end.to_string();
    let (table, old) = ui_table(&day);
    let mut source = table.source.to_string();
    if old {
        source.push_str(&format!(" · {}", t("nucleo-renda-rescisao-tabela-antiga")));
    }
    let salaries: Vec<i64> = input
        .last_salaries
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|s| *s > 0)
        .take(3)
        .collect();
    let average = if salaries.is_empty() {
        salary
    } else {
        salaries.iter().sum::<i64>() / salaries.len() as i64
    };
    let none = |why: String| UnemploymentInsurance {
        eligible: false,
        reason: Some(why),
        average_salary: average,
        installment: 0,
        installments: 0,
        total: 0,
        table: source.clone(),
    };
    if !matches!(
        reason,
        TerminationReason::SemJustaCausa | TerminationReason::QuebraExperienciaEmpregador
    ) {
        return none(t("nucleo-renda-rescisao-seguro-motivo"));
    }
    let months = input
        .months_worked_36
        .unwrap_or_else(|| full_months(admission, end).min(36));
    let claim = input.previous_claims.unwrap_or(0) + 1;
    // Carência pela janela de cada pedido (com só o total dos 36 meses, a janela é o limite).
    let (need, window) = match claim {
        1 => (12, 18),
        2 => (9, 12),
        _ => (6, 6),
    };
    if months.min(window) < need {
        return none(t_args(
            "nucleo-renda-rescisao-seguro-carencia",
            &args(&[
                ("meses", i64::from(need).into()),
                ("janela", i64::from(window).into()),
                ("pedido", i64::from(claim).into()),
            ]),
        ));
    }
    let installments = match (claim, months) {
        (_, m) if m >= 24 => 5,
        (_, m) if m >= 12 => 4,
        (1, _) => 0,
        _ => 3,
    };
    let installment = unemployment_installment(average, &day);
    UnemploymentInsurance {
        eligible: true,
        reason: None,
        average_salary: average,
        installment,
        installments,
        total: installment * installments as i64,
        table: source,
    }
}
