use super::*;
use crate::reports::fixture::{self, cat, expense, new_db, write};
use crate::transactions::TxnKind;

fn d(s: &str) -> NaiveDate {
    parse_date(s).unwrap()
}

fn seeded() -> crate::reports::fixture::TestDb {
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let acc = fixture::account(tx, "Conta", "checking", "BRL", 2_000_000, "2025-01-01");
        let sal = cat(tx, "Salário");
        for m in 1..=12 {
            let date = format!("{}-05", month_of(add_months(d("2025-10-05"), m)));
            fixture::txn(
                tx,
                &acc,
                &date,
                1_000_000,
                TxnKind::Income,
                Some(&sal),
                None,
                "salário",
            );
            expense(tx, &acc, &date, -600_000, None, None, "gastos");
        }
        Ok(())
    });
    t
}

#[test]
fn base_and_scenario_with_child_and_house() {
    let t = seeded();
    let today = d("2026-10-15");
    let conn = t.db.conn();
    let base_only = simulate(
        conn,
        &ScenarioArgs {
            events: vec![],
            years: Some(1),
            real_return: Some(0.0),
            ..Default::default()
        },
        today,
    )
    .unwrap();
    assert_eq!(
        (base_only.monthly_income, base_only.monthly_expense),
        (1_000_000, 600_000)
    );
    // Saldo de hoje: 2.000.000 + 12 × 400.000 = 6.800.000; mais 12 meses de sobra.
    assert_eq!(base_only.base.end_liquid, 6_800_000 + 12 * 400_000);
    assert_eq!(base_only.net_worth_delta, 0);

    let child = Event::Monthly {
        label: "Filho".into(),
        start: "2026-11-01".into(),
        months: None,
        amount: -200_000,
    };
    let r = simulate(
        conn,
        &ScenarioArgs {
            name: Some("Filho".into()),
            events: vec![child.clone()],
            years: Some(2),
            real_return: Some(0.0),
            ..Default::default()
        },
        today,
    )
    .unwrap();
    assert_eq!(r.liquid_delta, -24 * 200_000);
    assert_eq!(r.net_worth_delta, -24 * 200_000);
    // A previsão de 12 meses soma o custo mensal.
    let last = r.forecast.points.last().unwrap();
    assert_eq!(last.scenario - last.base, -12 * 200_000);
    assert!(r.retirement.is_none());
    assert!(r.notes.iter().any(|n| n.contains("Aposentadoria")));

    // Imóvel de 500 mil: 100 mil de entrada, 400 mil financiados a 0% em 100 meses.
    let house = vec![
        Event::OneOff {
            label: "Compra".into(),
            date: "2026-12-01".into(),
            amount: -50_000_000,
        },
        Event::Loan {
            label: "Financiamento".into(),
            date: "2026-12-01".into(),
            principal: 40_000_000,
            annual_rate_pct: 0.0,
            months: 100,
        },
        Event::Asset {
            label: "Apartamento".into(),
            date: "2026-12-01".into(),
            value: 50_000_000,
            real_growth_pct: None,
        },
    ];
    let r = simulate(
        conn,
        &ScenarioArgs {
            events: house,
            years: Some(1),
            real_return: Some(0.0),
            ..Default::default()
        },
        today,
    )
    .unwrap();
    // Sai a entrada e 10 parcelas de 400 mil/100; o patrimônio não muda (sem juros nem valorização).
    assert_eq!(r.liquid_delta, -10_000_000 - 10 * 400_000);
    assert_eq!(r.net_worth_delta, 0);
    assert_eq!(r.scenario.first_short.as_deref(), Some("2026-12"));
}

#[test]
fn retirement_compares_with_the_saved_plan() {
    let t = seeded();
    let conn = t.db.conn();
    crate::settings::set(
        conn,
        crate::goals::retirement::PLAN_SETTING,
        &crate::goals::retirement::Plan {
            current_age: Some(40),
            retirement_age: Some(60),
            current_wealth: Some(50_000_000),
            monthly_contribution: Some(300_000),
            monthly_spending: Some(800_000),
            simulations: Some(300),
            ..Default::default()
        },
    )
    .unwrap();
    let r = simulate(
        conn,
        &ScenarioArgs {
            events: vec![Event::IncomePct {
                label: "Pausa".into(),
                start: "2026-11-01".into(),
                months: Some(24),
                pct: 0.0,
            }],
            ..Default::default()
        },
        d("2026-10-15"),
    )
    .unwrap();
    let ret = r.retirement.unwrap();
    assert!(ret.scenario_success <= ret.base_success);
    assert!(ret.scenario_wealth_at_retirement < ret.base_wealth_at_retirement);
    assert!(r.explanation.contains("Aposentadoria"));
    // Validação.
    assert!(
        simulate(
            conn,
            &ScenarioArgs {
                events: vec![Event::Loan {
                    label: "x".into(),
                    date: "2026-11-01".into(),
                    principal: 0,
                    annual_rate_pct: 5.0,
                    months: 10
                }],
                ..Default::default()
            },
            d("2026-10-15")
        )
        .is_err()
    );
}

/// Sem dinheiro disponível já hoje: falta desde o mês atual, e a explicação diz isso (não
/// "acaba" no mês seguinte).
#[test]
fn already_short_today_says_so() {
    let today = parse_date("2026-10-20").unwrap();
    let mut s = Start {
        liquid: -1_000_000,
        others: 0,
        income: 100_000,
        expense: 200_000,
    };
    let (p, _) = project(&s, &[], today, 3, 0.0).unwrap();
    assert_eq!(p.first_short.as_deref(), Some("2026-10"));
    s.liquid = 150_000;
    let (p, _) = project(&s, &[], today, 3, 0.0).unwrap();
    assert_eq!(p.first_short.as_deref(), Some("2026-12"));
}
