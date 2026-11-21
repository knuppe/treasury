//! Marcos financeiros sobre a barra (docs/life-expectancy.md §6.1), só com o que o núcleo já
//! sabe, sem inventar: o plano de aposentadoria salvo (aposentadoria, início do benefício
//! público, até onde o plano vai, eventos de vida), as metas com data, a meta de
//! aposentadoria e o fim das dívidas pelo cronograma.
//!
//! O plano é da família (as idades são as do membro principal). Metas e dívidas vão para o
//! membro delas: a dívida pelo titular do contrato, a meta pelo dono único das contas ligadas;
//! sem dono, ficam com o membro principal (quem cuida das finanças da família).

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;

use super::{LifeMilestone, MilestoneKind, age_on};
use crate::error::Result;
use crate::household::Member;
use crate::i18n::{args, t, t_args};

/// Marcos de um membro, por idade.
pub fn of_member(
    conn: &Connection,
    member: &Member,
    birth: NaiveDate,
    primary_id: Option<&str>,
    today: NaiveDate,
) -> Result<Vec<LifeMilestone>> {
    let is_primary = primary_id == Some(member.id.as_str());
    let base = crate::household::base_currency(conn)?;
    let mut out = vec![];
    let at_age = |age: u32| (age, birth.year() + age as i32);
    let at_date = |d: NaiveDate| age_on(birth, d).map(|a| (a, d.year()));

    let mut plan_has_retirement = false;
    if is_primary {
        let plan = crate::goals::retirement::saved(conn)?;
        if plan.current_age.is_some() {
            let retire = plan
                .retirement_age
                .unwrap_or(65.max(plan.current_age.unwrap_or(65)));
            let (age, year) = at_age(retire);
            out.push(LifeMilestone {
                kind: MilestoneKind::Retirement,
                age,
                year,
                label: t("nucleo-longevidade-marco-aposentadoria"),
                amount: None,
            });
            plan_has_retirement = true;
            // Benefício público: o informado; sem ele, o calculado pelas regras.
            let mut inss = plan
                .inss_monthly
                .filter(|m| *m > 0)
                .map(|m| (plan.inss_age.unwrap_or(retire), Some(m)));
            if inss.is_none()
                && let Ok(Some(e)) = crate::goals::retirement::public_estimate(&plan, retire, today)
                && let Some(start) = plan.inss_age.or(e.start_age)
            {
                inss = Some((start, e.monthly));
            }
            if let Some((start, monthly)) = inss {
                let (age, year) = at_age(start);
                out.push(LifeMilestone {
                    kind: MilestoneKind::Inss,
                    age,
                    year,
                    label: t(if crate::country::is_brazil(conn) {
                        "nucleo-longevidade-marco-inss"
                    } else {
                        "nucleo-longevidade-marco-previdencia"
                    }),
                    amount: monthly,
                });
            }
            // Saque do FGTS na aposentadoria (docs/brazil/fgts.md, seção 6.7).
            if plan.include_fgts != Some(false)
                && let Ok(Some(v)) =
                    crate::fgts::retirement_withdrawal(conn, plan.current_age, retire, today)
            {
                let (age, year) = at_age(retire);
                out.push(LifeMilestone {
                    kind: MilestoneKind::LifeEvent,
                    age,
                    year,
                    label: t("nucleo-fgts-evento-aposentadoria"),
                    amount: Some(v),
                });
            }
            let (age, year) = at_age(plan.life_expectancy.unwrap_or(95));
            out.push(LifeMilestone {
                kind: MilestoneKind::PlanEnd,
                age,
                year,
                label: t("nucleo-longevidade-marco-fim-plano"),
                amount: None,
            });
            for e in plan.events.iter().flatten() {
                let (age, year) = at_age(e.age);
                out.push(LifeMilestone {
                    kind: MilestoneKind::LifeEvent,
                    age,
                    year,
                    label: e.label.clone(),
                    amount: Some(e.amount),
                });
            }
        }
    }

    // Metas com data (a de aposentadoria ligada ao plano já é o marco do plano).
    let linked = crate::goals::retirement::linked_goal(conn)?;
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.kind, g.target_date, g.target_amount, g.currency,
                (SELECT CASE WHEN count(DISTINCT am.member_id) = 1 THEN min(am.member_id) END
                   FROM goal_account ga JOIN account_member am ON am.account_id = ga.account_id
                  WHERE ga.goal_id = g.id)
           FROM goal g
          WHERE g.archived = 0 AND g.target_date IS NOT NULL
          ORDER BY g.target_date, g.id",
    )?;
    let goals = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (id, name, kind, date, target, currency, owner) in goals {
        let mine = match &owner {
            Some(o) => *o == member.id,
            None => is_primary,
        };
        let Ok(d) = crate::time::parse_date(&date) else {
            continue;
        };
        if !mine || d < today {
            continue;
        }
        let retirement = kind == "retirement";
        if retirement && (linked.as_deref() == Some(id.as_str()) || plan_has_retirement) {
            continue;
        }
        let Some((age, year)) = at_date(d) else {
            continue;
        };
        out.push(LifeMilestone {
            kind: if retirement {
                MilestoneKind::Retirement
            } else {
                MilestoneKind::Goal
            },
            age,
            year,
            label: name,
            amount: (currency == base).then_some(target),
        });
    }

    // Fim das dívidas: a última parcela do cronograma.
    let mut stmt = conn.prepare(
        "SELECT l.id, l.name, l.currency, l.member_id, max(i.due_date)
           FROM loan l JOIN loan_installment i ON i.loan_id = l.id
          WHERE l.archived = 0
          GROUP BY l.id
          ORDER BY max(i.due_date), l.id",
    )?;
    let loans = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let today_s = crate::time::fmt_date(today);
    for (id, name, currency, owner, last) in loans {
        let mine = match &owner {
            Some(o) => *o == member.id,
            None => is_primary,
        };
        let Ok(d) = crate::time::parse_date(&last) else {
            continue;
        };
        if !mine || d < today {
            continue;
        }
        let Some((age, year)) = at_date(d) else {
            continue;
        };
        let balance = crate::debt::balance_at(conn, &id, &today_s)?;
        out.push(LifeMilestone {
            kind: MilestoneKind::DebtEnd,
            age,
            year,
            label: t_args(
                "nucleo-longevidade-marco-divida",
                &args(&[("nome", name.into())]),
            ),
            amount: (currency == base).then_some(balance),
        });
    }
    out.sort_by_key(|m| (m.age, m.year, m.kind as u8));
    Ok(out)
}
