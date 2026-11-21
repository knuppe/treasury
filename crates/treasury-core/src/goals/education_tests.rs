//! Testes da meta de faculdade (docs/design/college-goal.md): a conta do plano, a inflação da
//! educação pelas séries, o cadastro com o histórico, as posições ligadas e o recálculo anual.
//! Pessoas, valores e datas são inventados.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::education::*;
use super::holdings::{HoldingInput, HoldingsSetArgs};
use super::*;
use crate::reports::fixture::{self, new_db, write};

fn d(s: &str) -> NaiveDate {
    parse_date(s).unwrap()
}

/// Exemplo do desenho: particular, 5 anos, mensalidade de hoje de R$ 2.500, extras de R$ 3.000
/// por ano, começando em fevereiro aos 18.
fn plan() -> EducationPlan {
    EducationPlan {
        member_id: None,
        expected_birth: None,
        duration_months: 60,
        fee: 250_000,
        fee_date: "2026-11-15".into(),
        yearly_extras: 300_000,
        inflation_mode: InflationMode::FixedSpread,
        fixed_spread: Some(0.0192),
        ..EducationPlan::default()
    }
}

fn infl() -> EducationInflation {
    EducationInflation {
        country: "BR".into(),
        mode: InflationMode::Series10y,
        series_id: Some("ipca_g8".into()),
        series_name: None,
        headline_id: Some("ipca".into()),
        headline_name: None,
        spread: 0.0,
        education_annual: Some(0.06),
        headline_annual: Some(0.04),
        from: Some("2016-01".into()),
        to: Some("2025-12".into()),
        years: 10,
        last_data: Some("2026-09".into()),
        missing: false,
    }
}

#[test]
fn the_design_example_adds_up() {
    let today = d("2026-11-15");
    let birth = Some(d("2018-06-15"));
    let r = compute(&plan(), birth, 0, infl(), today).unwrap();
    assert_eq!(r.start_date, "2036-02-01");
    assert_eq!(r.end_date, "2041-01-01");
    assert_eq!(r.months_to_start, 111);
    assert_eq!(r.phase, EducationPhase::Saving);
    assert_eq!(r.child_age, Some(8));
    // Dez reajustes (fevereiro de 2027 a 2036) de 1,92% sobre os R$ 2.500 de hoje.
    assert_eq!(
        r.fee_at_start,
        (250_000.0f64 * 1.0192f64.powi(10)).round() as i64
    );
    // O alvo é o valor presente dos pagamentos: menor que a soma.
    assert!(r.target_at_start < r.total_payments);
    assert!(
        (19_200_000..19_400_000).contains(&r.target_at_start),
        "{}",
        r.target_at_start
    );
    assert!(
        (150_000..152_500).contains(&r.monthly_indexed),
        "{}",
        r.monthly_indexed
    );
    assert_eq!(r.monthly_needed, r.monthly_indexed);
    // Só com a inflação geral, o aporte fica ~20% menor; o fixo em reais começa mais alto.
    assert!(r.monthly_headline_only < r.monthly_indexed * 85 / 100);
    assert!(r.monthly_flat > r.monthly_indexed);
    assert!(r.monthly_next_year > r.monthly_indexed);
    // Com R$ 20.000 já guardados, o aporte cai.
    let saved = compute(&plan(), birth, 2_000_000, infl(), today).unwrap();
    assert!(saved.monthly_indexed < r.monthly_indexed - 15_000);
    // Tabela ano a ano: de 2026 até o fim do curso, com os anos do curso marcados.
    assert_eq!(r.years.first().unwrap().year, 2026);
    assert_eq!(r.years.last().unwrap().year, 2041);
    let course: Vec<i32> = r
        .years
        .iter()
        .filter(|y| y.in_course)
        .map(|y| y.year)
        .collect();
    assert_eq!(course, (2036..=2041).collect::<Vec<_>>());
    // O saldo do plano chega perto de zero no fim (o aporte cobre o curso).
    let peak = r.years.iter().map(|y| y.planned_balance).max().unwrap();
    assert!(peak > r.target_at_start * 9 / 10, "{peak}");
}

#[test]
fn modes_change_the_contribution() {
    let today = d("2026-11-15");
    let birth = Some(d("2018-06-15"));
    let mut p = plan();
    p.contribution_mode = ContributionMode::Flat;
    let flat = compute(&p, birth, 0, infl(), today).unwrap();
    assert_eq!(flat.monthly_needed, flat.monthly_flat);
    p.inflation_mode = InflationMode::Headline;
    p.contribution_mode = ContributionMode::Indexed;
    let head = compute(&p, birth, 0, infl(), today).unwrap();
    assert_eq!(head.fee_at_start, 250_000);
    assert_eq!(head.monthly_indexed, head.monthly_headline_only);
    // Duas parcelas por ano (semestre) com o mesmo valor por parcela: um sexto dos pagamentos.
    let mut sem = plan();
    sem.payments_per_year = 2;
    sem.yearly_extras = 0;
    let s = compute(&sem, birth, 0, infl(), today).unwrap();
    let mut monthly = plan();
    monthly.yearly_extras = 0;
    let m = compute(&monthly, birth, 0, infl(), today).unwrap();
    assert_eq!(s.total_payments * 6, m.total_payments);
    // Pública sem mensalidade: o custo é o de vida, que não sobe com a educação.
    let mut public = plan();
    public.institution_type = InstitutionType::Public;
    public.fee = 0;
    public.yearly_extras = 0;
    public.monthly_living = 150_000;
    let pb = compute(&public, birth, 0, infl(), today).unwrap();
    assert_eq!(pb.total_payments, 150_000 * 60);
}

#[test]
fn during_the_course_it_says_how_far_the_money_goes() {
    let birth = Some(d("2018-06-15"));
    // No terceiro mês do segundo ano, com o suficiente para pouco mais de um ano.
    let today = d("2037-04-10");
    let r = compute(&plan(), birth, 3_500_000, infl(), today).unwrap();
    assert_eq!(r.phase, EducationPhase::Paying);
    assert_eq!(r.months_to_start, -14);
    assert_eq!(r.uncovered_year, Some(3));
    assert!(r.shortfall > 0 && r.monthly_needed > 0);
    // Com o bastante, cobre tudo.
    let rich = compute(&plan(), birth, 50_000_000, infl(), today).unwrap();
    assert_eq!(rich.uncovered_year, None);
    assert_eq!(rich.shortfall, 0);
    // Depois do curso: terminou, sem aporte.
    let done = compute(&plan(), birth, 0, infl(), d("2041-03-01")).unwrap();
    assert_eq!(done.phase, EducationPhase::Finished);
    assert_eq!(done.monthly_needed, 0);
    assert!(done.years.is_empty());
}

#[test]
fn the_start_date_wins_and_a_birth_is_needed_without_it() {
    let mut p = plan();
    assert!(compute(&p, None, 0, infl(), d("2026-11-15")).is_err());
    p.start_date = Some("2035-08-20".into());
    let r = compute(&p, None, 0, infl(), d("2026-11-15")).unwrap();
    assert_eq!(r.start_date, "2035-08-01");
}

/// IPCA de 4% e educação de 6% ao ano (taxas mensais constantes), 11 anos completos.
fn seed_series(conn: &rusqlite::Connection, last_year: i32) {
    let monthly = |annual: f64| (1.0 + annual).powf(1.0 / 12.0) - 1.0;
    for (id, annual) in [("ipca", 0.04), ("ipca_g8", 0.06)] {
        let pct = Decimal::from_f64_retain(monthly(annual) * 100.0)
            .unwrap()
            .round_dp(8);
        let mut values = vec![];
        for y in last_year - 10..=last_year {
            for m in 1..=12 {
                values.push((NaiveDate::from_ymd_opt(y, m, 1).unwrap(), pct, false));
            }
        }
        crate::indices::upsert_values(conn, id, &values, "teste").unwrap();
    }
}

#[test]
fn the_spread_comes_from_ten_complete_years_of_the_series() {
    let t = new_db();
    let conn = t.db.conn();
    let none = inflation(conn, InflationMode::Series10y, None, None, d("2026-11-15")).unwrap();
    assert!(none.missing);
    assert_eq!(none.spread, 0.0);
    assert_eq!(none.series_id.as_deref(), Some("ipca_g8"));
    assert_eq!(none.headline_id.as_deref(), Some("ipca"));
    seed_series(conn, 2025);
    let i = inflation(conn, InflationMode::Series10y, None, None, d("2026-11-15")).unwrap();
    assert!(!i.missing);
    assert_eq!(i.years, 10);
    assert_eq!(i.from.as_deref(), Some("2016-01"));
    assert_eq!(i.to.as_deref(), Some("2025-12"));
    assert!(
        (i.spread - (1.06 / 1.04 - 1.0)).abs() < 1e-6,
        "{}",
        i.spread
    );
    assert!((i.headline_annual.unwrap() - 0.04).abs() < 1e-6);
    // Só a inflação geral, ou uma diferença fixa.
    let h = inflation(conn, InflationMode::Headline, None, None, d("2026-11-15")).unwrap();
    assert_eq!(h.spread, 0.0);
    let f = inflation(
        conn,
        InflationMode::FixedSpread,
        None,
        Some(0.025),
        d("2026-11-15"),
    )
    .unwrap();
    assert_eq!(f.spread, 0.025);
}

fn child(t: &mut fixture::TestDb, birth: &str) -> Id {
    let id = fixture::member(t.db.conn(), "Ana");
    t.db.conn()
        .execute(
            "UPDATE member SET birth_date = ?2, relationship = 'filho' WHERE id = ?1",
            rusqlite::params![id, birth],
        )
        .unwrap();
    id
}

#[test]
fn saving_creates_the_goal_and_keeps_the_history() {
    let mut t = new_db();
    let ana = child(&mut t, "2018-06-15");
    let today = d("2026-11-15");
    let mut p = plan();
    p.member_id = Some(ana.clone());
    let args = EducationSaveArgs {
        goal_id: None,
        name: None,
        plan: p.clone(),
        initial_amount: Some(2_000_000),
        currency: None,
    };
    let id = write(&mut t.db, |tx| save(tx, &args, today));
    let conn = t.db.conn();
    let g = super::load(conn, &id).unwrap();
    assert_eq!(g.kind, GoalKind::Education);
    assert_eq!(g.name, "Faculdade de Ana");
    assert_eq!(g.target_date.as_deref(), Some("2036-02-01"));
    let v = plan_view(conn, &id, today).unwrap();
    assert_eq!(v.child_name.as_deref(), Some("Ana"));
    assert_eq!(v.goal.current_amount, 2_000_000);
    assert_eq!(g.target_amount, v.result.target_at_start);
    assert_eq!(g.monthly_contribution, Some(v.result.monthly_needed));
    assert_eq!(v.reviews.len(), 1);
    assert_eq!(v.reviews[0].reason, ReviewReason::Created);
    // A lista de metas traz o resumo do plano.
    let brief = v.goal.education.clone().unwrap();
    assert_eq!(brief.child_name.as_deref(), Some("Ana"));
    assert_eq!(brief.monthly_needed, v.result.monthly_needed);
    // Só a mensalidade nova: o recálculo fica como "mensalidade atualizada".
    p.fee = 270_000;
    p.fee_date = "2027-03-01".into();
    let upd = EducationSaveArgs {
        goal_id: Some(id.clone()),
        plan: p.clone(),
        name: None,
        initial_amount: None,
        currency: None,
    };
    write(&mut t.db, |tx| save(tx, &upd, today));
    let r = reviews(t.db.conn(), &id).unwrap();
    assert_eq!(r.len(), 2);
    assert!(r.iter().any(|x| x.reason == ReviewReason::FeeUpdate));
    // Salvar igual não grava outro recálculo.
    write(&mut t.db, |tx| save(tx, &upd, today));
    assert_eq!(reviews(t.db.conn(), &id).unwrap().len(), 2);
    // Visão somada.
    let o = overview(t.db.conn(), today).unwrap();
    assert_eq!(o.plans.len(), 1);
    assert_eq!(o.monthly_total, o.plans[0].brief.monthly_needed);
    assert!(o.overlap_years.is_empty());
    // Apagar a meta leva o plano junto.
    write(&mut t.db, |tx| super::delete(tx, &id));
    assert!(super::education::load(t.db.conn(), &id).unwrap().is_none());
}

#[test]
fn a_child_without_birth_date_is_refused() {
    let mut t = new_db();
    let ana = fixture::member(t.db.conn(), "Ana");
    let mut p = plan();
    p.member_id = Some(ana);
    let args = EducationSaveArgs {
        goal_id: None,
        name: None,
        plan: p,
        initial_amount: None,
        currency: None,
    };
    let err = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::system("teste"),
        "x",
        |tx| save(tx, &args, d("2026-11-15")),
    )
    .unwrap_err();
    assert!(err.to_string().contains("nascimento"), "{err}");
}

#[test]
fn holdings_count_toward_the_goal_with_shares_and_a_pool() {
    let mut t = new_db();
    let today = crate::time::today_local();
    let open = fmt_date(crate::time::add_months(today, -2));
    let (acc, g1, g2) = write(&mut t.db, |tx| {
        let acc = fixture::account(tx, "Corretora", "brokerage", "BRL", 1_000_000, &open);
        let mk = |name: &str, date: &str, target: i64| {
            super::create(
                tx,
                &GoalInput {
                    name: name.into(),
                    kind: GoalKind::Other,
                    target_amount: target,
                    currency: None,
                    target_date: Some(date.into()),
                    initial_amount: None,
                    monthly_contribution: None,
                    account_ids: None,
                    notes: None,
                    real_return: None,
                },
            )
        };
        let g1 = mk("Primeira", "2030-02-01", 300_000)?;
        let g2 = mk("Segunda", "2034-02-01", 2_000_000)?;
        Ok((acc, g1, g2))
    });
    // Metade da conta para a primeira meta.
    let half = HoldingsSetArgs {
        goal_id: g1.clone(),
        holdings: vec![HoldingInput {
            account_id: acc.clone(),
            asset_id: None,
            share: Some(0.5),
            value_mode: None,
        }],
    };
    write(&mut t.db, |tx| super::holdings::set(tx, &half));
    let v1 = get(t.db.conn(), &g1).unwrap();
    assert_eq!(v1.holdings_value, Some(500_000));
    assert_eq!(v1.current_amount, 500_000);
    // Mais de 100% entre as metas é recusado.
    let too_much = HoldingsSetArgs {
        goal_id: g2.clone(),
        holdings: vec![HoldingInput {
            account_id: acc.clone(),
            asset_id: None,
            share: Some(0.6),
            value_mode: None,
        }],
    };
    let err = crate::changes::record(
        t.db.conn_mut(),
        &crate::changes::Author::system("teste"),
        "x",
        |tx| super::holdings::set(tx, &too_much),
    )
    .unwrap_err();
    assert!(err.to_string().contains("100%"), "{err}");
    // Pote comum: as duas sem fração; quem começa primeiro recebe primeiro (até o alvo).
    for g in [&g1, &g2] {
        let pool = HoldingsSetArgs {
            goal_id: g.to_string(),
            holdings: vec![HoldingInput {
                account_id: acc.clone(),
                asset_id: None,
                share: None,
                value_mode: None,
            }],
        };
        write(&mut t.db, |tx| super::holdings::set(tx, &pool));
    }
    let a = get(t.db.conn(), &g1).unwrap();
    let b = get(t.db.conn(), &g2).unwrap();
    assert_eq!(a.holdings_value, Some(300_000));
    assert_eq!(b.holdings_value, Some(700_000));
    // Desligar tudo.
    let none = HoldingsSetArgs {
        goal_id: g1.clone(),
        holdings: vec![],
    };
    write(&mut t.db, |tx| super::holdings::set(tx, &none));
    assert_eq!(get(t.db.conn(), &g1).unwrap().holdings_value, None);
    assert_eq!(
        get(t.db.conn(), &g2).unwrap().holdings_value,
        Some(1_000_000)
    );
}

#[test]
fn the_annual_review_runs_once_from_april_and_flags_big_changes() {
    let mut t = new_db();
    let ana = child(&mut t, "2018-06-15");
    let mut p = plan();
    p.member_id = Some(ana);
    let args = EducationSaveArgs {
        goal_id: None,
        name: None,
        plan: p,
        initial_amount: None,
        currency: None,
    };
    let id = write(&mut t.db, |tx| save(tx, &args, d("2026-11-15")));
    let conn = t.db.conn();
    // Antes de abril, nada.
    assert!(annual_review(conn, d("2027-03-20")).unwrap().is_empty());
    assert_eq!(reviews(conn, &id).unwrap().len(), 1);
    // Em abril grava o recálculo; só avisa se o aporte mudou mais que o limite.
    let changes = annual_review(conn, d("2027-04-05")).unwrap();
    let r = reviews(conn, &id).unwrap();
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].reason, ReviewReason::Annual);
    let old = r[1].monthly_needed as f64;
    let new = r[0].monthly_needed as f64;
    assert_eq!(
        !changes.is_empty(),
        ((new - old).abs() / old) > REVIEW_ALERT_CHANGE
    );
    // No mesmo ano, não repete.
    assert!(annual_review(conn, d("2027-09-01")).unwrap().is_empty());
    assert_eq!(reviews(conn, &id).unwrap().len(), 2);
}
