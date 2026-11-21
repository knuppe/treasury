//! Simulador de quitação (docs/debt-payoff.md): com um valor extra por mês, em que ordem
//! atacar as dívidas. Compara três caminhos, mês a mês:
//!
//! - **mínimo**: só as parcelas, sem extra (a linha de base);
//! - **bola de neve**: o extra vai para a dívida de menor saldo; quitada uma, a parcela dela
//!   soma ao extra da próxima (vitórias rápidas);
//! - **avalanche**: o mesmo, mas pela maior taxa de juros primeiro (menos juros no total).
//!
//! As dívidas são os empréstimos e financiamentos ativos (saldo, taxa contratada e a parte
//! de juros + amortização da próxima parcela) e as que o usuário informar na hora
//! (cheque especial, rotativo, um parcelamento fora do app). É uma aproximação: taxa fixa,
//! parcela constante (no SAC ela cai com o tempo), sem a correção do indexador nem seguros
//! e tarifas.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use fluent_bundle::FluentValue;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::schedule::{monthly_rate, price_payment};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{add_months, month_of};

/// Teto da simulação: 50 anos.
const MAX_MONTHS: u32 = 600;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CustomDebt {
    pub name: String,
    /// Saldo devedor hoje, em centavos.
    #[ts(type = "number")]
    pub balance: i64,
    /// Juros ao ano em %, como texto decimal (`"151.8"` = 151,8% a.a.).
    pub annual_rate: String,
    /// Pagamento mínimo por mês, em centavos.
    #[ts(type = "number")]
    pub min_payment: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayoffArgs {
    /// Quanto sobra por mês para pagar além das parcelas, em centavos (padrão 0).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub extra_monthly: Option<i64>,
    /// Só estes contratos (padrão: todos os ativos com saldo).
    #[serde(default)]
    #[ts(optional)]
    pub loan_ids: Option<Vec<Id>>,
    /// Dívidas fora do app (cheque especial, rotativo...).
    #[serde(default)]
    #[ts(optional)]
    pub custom: Option<Vec<CustomDebt>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PayoffStrategy {
    Minimum,
    Snowball,
    Avalanche,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayoffDebt {
    /// Contrato no app (`None` = informada na simulação).
    pub loan_id: Option<Id>,
    pub name: String,
    #[ts(type = "number")]
    pub balance: i64,
    pub annual_rate: String,
    pub monthly_rate: String,
    #[ts(type = "number")]
    pub min_payment: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayoffDebtResult {
    pub name: String,
    pub loan_id: Option<Id>,
    /// Ordem de quitação (1 = a primeira).
    pub order: u32,
    /// Mês da última parcela (`YYYY-MM`); `None` se não quita em 50 anos.
    pub paid_off: Option<String>,
    pub months: Option<u32>,
    #[ts(type = "number")]
    pub interest: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayoffMonth {
    pub month: String,
    /// Saldo devedor somado no fim do mês.
    #[ts(type = "number")]
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayoffResult {
    pub strategy: PayoffStrategy,
    /// Quita tudo em até 50 anos (o mínimo pode não cobrir os juros).
    pub feasible: bool,
    pub months: u32,
    /// Mês em que a última dívida acaba (`YYYY-MM`).
    pub debt_free: Option<String>,
    #[ts(type = "number")]
    pub total_interest: i64,
    #[ts(type = "number")]
    pub total_paid: i64,
    /// Juros a menos que só pagando o mínimo.
    #[ts(type = "number")]
    pub interest_saved: i64,
    /// Meses a menos que só pagando o mínimo.
    pub months_saved: i32,
    pub debts: Vec<PayoffDebtResult>,
    pub series: Vec<PayoffMonth>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayoffPlan {
    pub start: String,
    #[ts(type = "number")]
    pub extra_monthly: i64,
    /// Parcelas mínimas somadas + extra: o que sai por mês nas estratégias.
    #[ts(type = "number")]
    pub monthly_budget: i64,
    pub debts: Vec<PayoffDebt>,
    pub minimum: PayoffResult,
    pub snowball: PayoffResult,
    pub avalanche: PayoffResult,
    /// A que paga menos juros (empate: bola de neve, pelas vitórias rápidas).
    pub recommended: PayoffStrategy,
    pub explanation: String,
    pub notes: Vec<String>,
}

struct Debt {
    loan_id: Option<Id>,
    name: String,
    balance: i64,
    rate: Decimal,
    annual: String,
    min: i64,
}

pub fn payoff_plan(conn: &Connection, a: &PayoffArgs, today: NaiveDate) -> Result<PayoffPlan> {
    let extra = a.extra_monthly.unwrap_or(0);
    if extra < 0 {
        return Err(Error::Invalid(t(
            "nucleo-planejamento-quitacao-erro-extra-negativo",
        )));
    }
    let mut debts = Vec::new();
    let mut notes = Vec::new();
    let loans = super::list(conn, false)?;
    if let Some(ids) = &a.loan_ids
        && let Some(missing) = ids.iter().find(|id| !loans.iter().any(|l| &l.id == *id))
    {
        return Err(Error::NotFound(t_args(
            "nucleo-planejamento-quitacao-contrato",
            &args(&[("id", missing.clone().into())]),
        )));
    }
    for l in loans {
        if l.balance <= 0 || a.loan_ids.as_ref().is_some_and(|ids| !ids.contains(&l.id)) {
            continue;
        }
        // `monthly_rate` da view vem em %.
        let rate = Decimal::from_str(&l.monthly_rate).unwrap_or_default() / Decimal::ONE_HUNDRED;
        let min = match &l.next_installment {
            Some(n) if n.amortization + n.interest + n.adjustment > 0 => {
                n.amortization + n.interest + n.adjustment
            }
            _ => price_payment(Decimal::from(l.balance), rate, l.remaining_count.max(1))
                .round()
                .to_i64()
                .unwrap_or(l.balance),
        };
        // TR, IPCA e afins corrigem por cima da taxa; o CDI e a Selic do contrato entram
        // como percentual do índice, que a projeção ainda não reproduz.
        let rate = match l.indexer {
            super::LoanIndexer::None => rate,
            i if i.daily_series().is_some() => {
                notes.push(t_args(
                    "nucleo-planejamento-quitacao-nota-indexador",
                    &args(&[("contrato", l.name.clone().into())]),
                ));
                rate
            }
            i => with_index(conn, &l.name, rate, i, today, &mut notes)?,
        };
        if l.system == super::AmortSystem::Sac {
            notes.push(t_args(
                "nucleo-planejamento-quitacao-nota-sac",
                &args(&[("contrato", l.name.clone().into())]),
            ));
        }
        debts.push(Debt {
            loan_id: Some(l.id),
            name: l.name,
            balance: l.balance,
            rate,
            annual: l.annual_rate,
            min,
        });
    }
    for c in a.custom.iter().flatten() {
        let name = c.name.trim();
        if name.is_empty() || c.balance <= 0 || c.min_payment <= 0 {
            return Err(Error::Invalid(t(
                "nucleo-planejamento-quitacao-erro-divida-incompleta",
            )));
        }
        // "4,75 + TR": taxa sobre o índice, como no cadastro do contrato e do limite.
        let (text, indexer) = match super::split_rate_index(&c.annual_rate) {
            Some((num, idx)) => (num, idx),
            None => (c.annual_rate.clone(), super::LoanIndexer::None),
        };
        let annual = Decimal::from_str(&text.replace(',', ".")).map_err(|_| {
            Error::Invalid(t_args(
                "nucleo-planejamento-divida-erro-taxa-invalida",
                &args(&[("taxa", c.annual_rate.clone().into())]),
            ))
        })?;
        if annual.is_sign_negative() {
            return Err(Error::Invalid(t(
                "nucleo-planejamento-quitacao-erro-taxa-negativa",
            )));
        }
        let (rate, annual) = if indexer == super::LoanIndexer::None {
            (monthly_rate(annual), annual.normalize().to_string())
        } else {
            let r = with_index(conn, name, monthly_rate(annual), indexer, today, &mut notes)?;
            (r, annual_of(r))
        };
        debts.push(Debt {
            loan_id: None,
            name: name.to_string(),
            balance: c.balance,
            rate,
            annual,
            min: c.min_payment,
        });
    }
    if debts.is_empty() {
        return Err(Error::Invalid(t(
            "nucleo-planejamento-quitacao-erro-sem-dividas",
        )));
    }
    let budget: i64 = debts.iter().map(|d| d.min).sum::<i64>() + extra;
    let minimum = simulate(&debts, PayoffStrategy::Minimum, 0, today);
    let mut snowball = simulate(&debts, PayoffStrategy::Snowball, extra, today);
    let mut avalanche = simulate(&debts, PayoffStrategy::Avalanche, extra, today);
    // Sem quitar só com o mínimo, não há base para comparar (os juros crescem sem fim).
    if minimum.feasible {
        for r in [&mut snowball, &mut avalanche] {
            r.interest_saved = minimum.total_interest - r.total_interest;
            r.months_saved = minimum.months as i32 - r.months as i32;
        }
    }
    let recommended = if avalanche.total_interest < snowball.total_interest {
        PayoffStrategy::Avalanche
    } else {
        PayoffStrategy::Snowball
    };
    let currency = crate::household::base_currency(conn)?;
    let explanation = explain(&minimum, &snowball, &avalanche, recommended, &currency);
    if !minimum.feasible {
        notes.push(t("nucleo-planejamento-quitacao-nota-minimo-nao-quita"));
    }
    notes.dedup();
    Ok(PayoffPlan {
        start: month_of(add_months(today, 1)),
        extra_monthly: extra,
        monthly_budget: budget,
        debts: debts
            .iter()
            .map(|d| PayoffDebt {
                loan_id: d.loan_id.clone(),
                name: d.name.clone(),
                balance: d.balance,
                annual_rate: d.annual.clone(),
                monthly_rate: (d.rate * Decimal::ONE_HUNDRED)
                    .round_dp(4)
                    .normalize()
                    .to_string(),
                min_payment: d.min,
            })
            .collect(),
        minimum,
        snowball,
        avalanche,
        recommended,
        explanation,
        notes,
    })
}

/// Taxa ao mês (fração) somada ao último valor do indexador, mantido constante na projeção;
/// sem o índice baixado, fica só a taxa contratada. A nota diz qual das duas.
fn with_index(
    conn: &Connection,
    name: &str,
    rate: Decimal,
    indexer: super::LoanIndexer,
    today: NaiveDate,
    notes: &mut Vec<String>,
) -> Result<Decimal> {
    let reading = crate::accounts::overdraft::index_reading(conn, Some(indexer), today)?;
    let Some(r) = reading else {
        notes.push(t_args(
            "nucleo-planejamento-quitacao-nota-indexador",
            &args(&[("contrato", name.to_string().into())]),
        ));
        return Ok(rate);
    };
    let idx = Decimal::from_str(&r.monthly_pct).unwrap_or_default() / Decimal::ONE_HUNDRED;
    notes.push(t_args(
        "nucleo-planejamento-quitacao-nota-indexador-valor",
        &args(&[
            ("contrato", name.to_string().into()),
            ("indice", indexer.label().into()),
            (
                "valor",
                crate::ai::alerts::decimal(crate::i18n::current(), &r.monthly_pct).into(),
            ),
            ("data", crate::transactions::util::date_text(&r.date).into()),
        ]),
    ));
    Ok((Decimal::ONE + rate) * (Decimal::ONE + idx) - Decimal::ONE)
}

/// % ao ano equivalente à taxa ao mês (fração), para mostrar.
fn annual_of(monthly: Decimal) -> String {
    ((crate::finance::pow_dec(Decimal::ONE + monthly, Decimal::from(12)) - Decimal::ONE)
        * Decimal::ONE_HUNDRED)
        .round_dp(2)
        .normalize()
        .to_string()
}

fn explain(
    min: &PayoffResult,
    snow: &PayoffResult,
    aval: &PayoffResult,
    rec: PayoffStrategy,
    currency: &str,
) -> String {
    let l = crate::i18n::current();
    let money = |c: i64| -> FluentValue<'static> { crate::i18n::money(l, c, currency).into() };
    // `YYYY-MM` → `MM/YYYY` (numérico, igual em todos os idiomas).
    let when = |r: &PayoffResult| -> FluentValue<'static> {
        match &r.debt_free {
            Some(m) if m.len() == 7 => format!("{}/{}", &m[5..], &m[..4]),
            Some(m) => m.clone(),
            None => t("nucleo-planejamento-quitacao-mais-de-50-anos"),
        }
        .into()
    };
    let best = if rec == PayoffStrategy::Avalanche {
        aval
    } else {
        snow
    };
    let mut parts = vec![if min.feasible {
        t_args(
            "nucleo-planejamento-quitacao-explicacao-minimo",
            &args(&[("quando", when(min)), ("juros", money(min.total_interest))]),
        )
    } else {
        t("nucleo-planejamento-quitacao-explicacao-minimo-nunca")
    }];
    parts.push(t_args(
        "nucleo-planejamento-quitacao-explicacao-estrategias",
        &args(&[
            ("bola_quando", when(snow)),
            ("bola_juros", money(snow.total_interest)),
            ("avalanche_quando", when(aval)),
            ("avalanche_juros", money(aval.total_interest)),
        ]),
    ));
    let (name, other) = if rec == PayoffStrategy::Avalanche {
        (t("nucleo-planejamento-quitacao-avalanche"), snow)
    } else {
        (t("nucleo-planejamento-quitacao-bola-de-neve"), aval)
    };
    parts.push(if snow.total_interest == aval.total_interest {
        t("nucleo-planejamento-quitacao-explicacao-iguais")
    } else if min.feasible {
        t_args(
            "nucleo-planejamento-quitacao-explicacao-economia",
            &args(&[
                ("estrategia", name.into()),
                ("valor", money(best.interest_saved)),
            ]),
        )
    } else {
        t_args(
            "nucleo-planejamento-quitacao-explicacao-menos-juros",
            &args(&[
                ("estrategia", name.into()),
                ("valor", money(other.total_interest - best.total_interest)),
            ]),
        )
    });
    parts.join(" ")
}

fn simulate(
    debts: &[Debt],
    strategy: PayoffStrategy,
    extra: i64,
    today: NaiveDate,
) -> PayoffResult {
    // Ordem de ataque com o que sobra.
    let mut order: Vec<usize> = (0..debts.len()).collect();
    match strategy {
        PayoffStrategy::Snowball | PayoffStrategy::Minimum => order.sort_by(|&a, &b| {
            debts[a]
                .balance
                .cmp(&debts[b].balance)
                .then(debts[b].rate.cmp(&debts[a].rate))
        }),
        PayoffStrategy::Avalanche => order.sort_by(|&a, &b| {
            debts[b]
                .rate
                .cmp(&debts[a].rate)
                .then(debts[a].balance.cmp(&debts[b].balance))
        }),
    }
    let mut bal: Vec<i64> = debts.iter().map(|d| d.balance).collect();
    let mut interest = vec![0i64; debts.len()];
    let mut done: Vec<Option<u32>> = vec![None; debts.len()];
    let budget: i64 = debts.iter().map(|d| d.min).sum::<i64>() + extra;
    let mut series = Vec::new();
    let mut paid_total = 0i64;
    let mut month = 0u32;
    let month_label = |m: u32| month_of(add_months(today, m as i32));
    // Se a dívida cresce sem parar (parcela abaixo dos juros), para cedo.
    let runaway = debts
        .iter()
        .map(|d| d.balance)
        .sum::<i64>()
        .saturating_mul(100);
    while month < MAX_MONTHS && bal.iter().any(|b| *b > 0) && bal.iter().sum::<i64>() < runaway {
        month += 1;
        for i in 0..debts.len() {
            if bal[i] > 0 {
                let j = (Decimal::from(bal[i]) * debts[i].rate)
                    .round()
                    .to_i64()
                    .unwrap_or(0);
                bal[i] += j;
                interest[i] += j;
            }
        }
        // Parcelas mínimas; no mínimo, o que uma dívida quitada deixa de pagar não vai para
        // as outras.
        let mut left = if strategy == PayoffStrategy::Minimum {
            0
        } else {
            budget
        };
        for i in 0..debts.len() {
            if bal[i] > 0 {
                let pay = debts[i].min.min(bal[i]);
                bal[i] -= pay;
                paid_total += pay;
                left -= pay;
            }
        }
        for &i in &order {
            if left <= 0 {
                break;
            }
            if bal[i] > 0 {
                let pay = left.min(bal[i]);
                bal[i] -= pay;
                paid_total += pay;
                left -= pay;
            }
        }
        for i in 0..debts.len() {
            if bal[i] == 0 && done[i].is_none() {
                done[i] = Some(month);
            }
        }
        series.push(PayoffMonth {
            month: month_label(month),
            balance: bal.iter().sum(),
        });
    }
    let feasible = bal.iter().all(|b| *b <= 0);
    // Ordem de quitação: pelo mês; empates pela ordem de ataque.
    let rank: BTreeMap<(u32, usize), usize> = order
        .iter()
        .enumerate()
        .map(|(pos, &i)| ((done[i].unwrap_or(u32::MAX), pos), i))
        .collect();
    let mut results: Vec<PayoffDebtResult> = rank
        .values()
        .enumerate()
        .map(|(n, &i)| PayoffDebtResult {
            name: debts[i].name.clone(),
            loan_id: debts[i].loan_id.clone(),
            order: n as u32 + 1,
            paid_off: done[i].map(month_label),
            months: done[i],
            interest: interest[i],
        })
        .collect();
    results.sort_by_key(|r| r.order);
    PayoffResult {
        strategy,
        feasible,
        months: month,
        debt_free: feasible.then(|| month_label(month)),
        total_interest: interest.iter().sum(),
        total_paid: paid_total,
        interest_saved: 0,
        months_saved: 0,
        debts: results,
        series,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::fixture::new_db;

    fn custom(name: &str, balance: i64, rate: &str, min: i64) -> CustomDebt {
        CustomDebt {
            name: name.into(),
            balance,
            annual_rate: rate.into(),
            min_payment: min,
        }
    }

    #[test]
    fn snowball_and_avalanche_against_minimum() {
        let t = new_db();
        let today = crate::time::parse_date("2026-10-15").unwrap();
        let args = PayoffArgs {
            extra_monthly: Some(50_000),
            loan_ids: None,
            custom: Some(vec![
                custom("Cheque especial", 300_000, "150", 30_000),
                custom("Parcelamento", 100_000, "20", 10_000),
                custom("Consignado", 800_000, "25", 25_000),
            ]),
        };
        let p = payoff_plan(t.db.conn(), &args, today).unwrap();
        assert_eq!(p.start, "2026-11");
        assert_eq!(p.monthly_budget, 115_000);
        assert!(p.minimum.feasible && p.snowball.feasible && p.avalanche.feasible);
        // Bola de neve começa pelo menor saldo; avalanche pela maior taxa.
        assert_eq!(p.snowball.debts[0].name, "Parcelamento");
        assert_eq!(p.avalanche.debts[0].name, "Cheque especial");
        assert!(p.avalanche.total_interest < p.snowball.total_interest);
        assert!(p.snowball.total_interest < p.minimum.total_interest);
        assert!(p.avalanche.months <= p.snowball.months);
        assert!(p.snowball.months < p.minimum.months);
        assert_eq!(p.recommended, PayoffStrategy::Avalanche);
        assert_eq!(
            p.avalanche.interest_saved,
            p.minimum.total_interest - p.avalanche.total_interest
        );
        // Pago = saldo inicial + juros.
        for r in [&p.minimum, &p.snowball, &p.avalanche] {
            assert_eq!(r.total_paid, 1_200_000 + r.total_interest);
            assert_eq!(r.series.last().unwrap().balance, 0);
            assert_eq!(
                r.debt_free.as_deref(),
                r.series.last().map(|m| m.month.as_str())
            );
        }
        assert!(p.explanation.contains("Avalanche"));
    }

    #[test]
    fn minimum_that_does_not_cover_interest_never_ends() {
        let t = new_db();
        let today = crate::time::parse_date("2026-10-15").unwrap();
        let args = PayoffArgs {
            extra_monthly: Some(0),
            loan_ids: None,
            custom: Some(vec![custom("Rotativo", 500_000, "400", 5_000)]),
        };
        let p = payoff_plan(t.db.conn(), &args, today).unwrap();
        assert!(!p.minimum.feasible);
        assert_eq!(p.minimum.debt_free, None);
        assert_eq!(p.minimum.debts[0].paid_off, None);
        assert!(p.notes.iter().any(|n| n.contains("não cobre os juros")));
        assert!(payoff_plan(t.db.conn(), &PayoffArgs::default(), today).is_err());
    }

    #[test]
    fn custom_debt_with_rate_plus_tr_uses_the_latest_index() {
        let t = new_db();
        let today = crate::time::parse_date("2026-10-15").unwrap();
        let args = |rate: &str| PayoffArgs {
            extra_monthly: Some(0),
            loan_ids: None,
            custom: Some(vec![custom("Limite", 100_000, rate, 10_000)]),
        };
        // Sem a TR baixada: só a taxa contratada, com aviso.
        let p = payoff_plan(t.db.conn(), &args("4,75 + TR"), today).unwrap();
        assert_eq!(p.debts[0].monthly_rate, "0.3875");
        assert!(p.notes.iter().any(|n| n.contains("sem a correção")));
        t.db.conn()
            .execute(
                "INSERT INTO index_value (series_id, date, value, source, fetched_at)
                 VALUES ('tr', '2026-09-01', 0.15, 'test', '2026-10-01T00:00:00Z')",
                [],
            )
            .unwrap();
        // (1 + 0,3875%) × (1 + 0,15%) − 1 ≈ 0,538% a.m.
        let p = payoff_plan(t.db.conn(), &args("TR + 4,75"), today).unwrap();
        assert_eq!(p.debts[0].monthly_rate, "0.538");
        assert_eq!(p.debts[0].annual_rate, "6.65");
        assert!(p.notes.iter().any(|n| n.contains("TR de 0,15%")));
    }
}
