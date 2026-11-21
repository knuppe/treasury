//! Degraus e fechamento do mês com famílias de teste montadas em relação a hoje (a reserva do
//! mês atual vem de `goals::emergency_reserve`, que usa a data do sistema).

use rusqlite::Transaction;

use super::closing::{self, MonthCloseArgs, MonthCloseSetArgs};
use super::ladder::{self, NextStep};
use super::*;
use crate::reports::engine::shift_month;
use crate::reports::fixture::{self, TestDb, cat, expense, new_db, txn, write};
use crate::time::{month_of, today_local};
use crate::transactions::TxnKind;

fn m(i: i32) -> String {
    shift_month(&month_of(today_local()), -i).unwrap()
}

fn day(month: &str, d: u32) -> String {
    format!("{month}-{d:02}")
}

fn step(t: &TestDb) -> NextStep {
    ladder::next_step(t.db.conn(), today_local()).unwrap()
}

/// Quatro meses completos com salário e gastos com categoria (aluguel e mercado).
fn family(tx: &Transaction<'_>, opening: i64, income: i64, rent: i64, market: i64) -> String {
    let checking = fixture::account(tx, "Conta", "checking", "BRL", opening, &day(&m(6), 1));
    let (salary, rent_c, market_c) = (
        cat(tx, "Salário"),
        cat(tx, "Moradia › Aluguel"),
        cat(tx, "Alimentação › Mercado"),
    );
    for i in 1..=4 {
        let month = m(i);
        txn(
            tx,
            &checking,
            &day(&month, 5),
            income,
            TxnKind::Income,
            Some(&salary),
            None,
            "Salário",
        );
        expense(
            tx,
            &checking,
            &day(&month, 10),
            -rent,
            Some(&rent_c),
            None,
            "Aluguel",
        );
        expense(
            tx,
            &checking,
            &day(&month, 15),
            -market,
            Some(&market_c),
            None,
            "Mercado",
        );
    }
    checking
}

#[test]
fn the_ladder_starts_by_knowing_where_the_money_goes() {
    // Sem contas: o primeiro passo é cadastrar uma.
    let mut t = new_db();
    let s = step(&t);
    assert_eq!((s.rung, s.key.as_str()), (1, "know"));
    assert_eq!(s.title, "Saber para onde o dinheiro vai");
    assert_eq!(s.why, "Ainda não há contas cadastradas.");
    assert_eq!(s.route.as_deref(), Some("/contas"));
    assert_eq!(s.rungs.len(), 6);
    assert!(s.rungs.iter().all(|r| !r.done));

    // Um mês só de lançamentos: faltam meses.
    write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 0, &day(&m(2), 1));
        expense(
            tx,
            &checking,
            &day(&m(1), 10),
            -5_000,
            None,
            None,
            "Padaria",
        );
        Ok(())
    });
    let s = step(&t);
    assert_eq!(s.rung, 1);
    assert!(
        s.why.starts_with("Há 1 mês completo de lançamentos."),
        "{}",
        s.why
    );
    assert_eq!(s.route.as_deref(), Some("/lancamentos?importar=1"));
}

#[test]
fn uncategorized_transactions_keep_the_family_on_the_first_rung() {
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let checking = family(tx, 0, 500_000, 100_000, 50_000);
        // Muitos lançamentos recentes sem categoria.
        for d in 1..=9 {
            expense(tx, &checking, &day(&m(1), d), -1_000, None, None, "Compra");
            expense(tx, &checking, &day(&m(2), d), -1_000, None, None, "Compra");
        }
        Ok(())
    });
    let s = step(&t);
    assert_eq!((s.rung, s.key.as_str()), (1, "know"));
    assert!(
        s.why
            .contains("dos lançamentos dos últimos 90 dias têm categoria"),
        "{}",
        s.why
    );
    assert!(s.action.contains("lançamentos que faltam"), "{}", s.action);
    assert!(s.route.unwrap().starts_with("/lancamentos?uncategorized=1"));
}

#[test]
fn spending_above_income_is_the_second_rung() {
    let mut t = new_db();
    write(&mut t.db, |tx| {
        family(tx, 2_000_000, 500_000, 400_000, 200_000);
        Ok(())
    });
    let s = step(&t);
    assert_eq!((s.rung, s.key.as_str()), (2, "balance"));
    assert_eq!(
        s.why,
        "Nos últimos 3 meses saiu 20% a mais do que entrou, e em 3 deles os gastos passaram da renda."
    );
    assert_eq!(s.route.as_deref(), Some("/orcamento"));
    assert!(s.rungs[0].done && !s.rungs[1].done);
    // Textos sem valores em dinheiro.
    assert!(!s.why.contains("R$") && !s.action.contains("R$"));
}

#[test]
fn the_reserve_comes_before_investing_and_the_top_has_no_end() {
    // Sobra todo mês, mas a reserva ainda não cobre três meses de essenciais.
    let mut t = new_db();
    write(&mut t.db, |tx| {
        family(tx, 0, 500_000, 200_000, 100_000);
        Ok(())
    });
    let s = step(&t);
    assert_eq!((s.rung, s.key.as_str()), (4, "reserve"), "{s:?}");
    assert!(s.why.starts_with("A reserva cobre "), "{}", s.why);
    assert!(s.why.ends_with("O próximo marco é 3 meses."), "{}", s.why);
    assert_eq!(s.route.as_deref(), Some("/orcamento?aba=reserva"));
    assert_eq!(s.rungs.iter().filter(|r| r.done).count(), 3);

    // Reserva montada, sem contratos: o topo.
    let mut t = new_db();
    write(&mut t.db, |tx| {
        family(tx, 5_000_000, 500_000, 200_000, 100_000);
        Ok(())
    });
    let s = step(&t);
    assert_eq!((s.rung, s.key.as_str()), (6, "invest"), "{s:?}");
    assert_eq!(s.rungs.iter().filter(|r| r.done).count(), 5);
    assert!(s.action.contains("Crie uma meta"), "{}", s.action);
    assert_eq!(s.route.as_deref(), Some("/orcamento?aba=metas"));
}

#[test]
fn closing_the_month_measures_three_steps_and_stores_the_other_two() {
    let today = today_local();
    let last = m(1);
    let mut t = new_db();
    // Sem lançamentos no mês, não há o que fechar.
    let empty = list(t.db.conn(), today).unwrap();
    let c = closing::month_close(t.db.conn(), &MonthCloseArgs::default(), &empty, today).unwrap();
    assert_eq!(c.month, last);
    assert!(!c.applicable);

    write(&mut t.db, |tx| {
        let checking = family(tx, 0, 500_000, 200_000, 100_000);
        expense(
            tx,
            &checking,
            &day(&last, 20),
            -3_000,
            None,
            None,
            "Sem categoria",
        );
        // A corrente conferida hoje fica em dia em qualquer dia do mês (sem isso, depois do dia
        // 10 o último lançamento, do dia 20 do mês passado, passa de 20 dias).
        tx.execute(
            "INSERT INTO balance_sync (id, account_id, date, statement_balance, computed_balance,
                                       difference, created_at, updated_at)
             VALUES ('sync-hoje', ?1, ?2, 0, 0, 0, 'x', 'x')",
            rusqlite::params![checking, crate::time::fmt_date(today)],
        )?;
        // Poupança parada há meses: falta extrato.
        let savings = fixture::account(tx, "Poupança", "savings", "BRL", 0, &day(&m(8), 1));
        expense(tx, &savings, &day(&m(6), 3), -100, None, None, "Tarifa");
        Ok(())
    });
    let conn = t.db.conn();
    let close = |conn: &rusqlite::Connection| {
        let a = list(conn, today).unwrap();
        closing::month_close(conn, &MonthCloseArgs::default(), &a, today).unwrap()
    };
    let c = close(conn);
    assert!(c.applicable);
    let keys: Vec<&str> = c.steps.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys, closing::STEPS);
    let by =
        |c: &closing::MonthClose, k: &str| c.steps.iter().find(|s| s.key == k).unwrap().clone();
    let st = by(&c, "statements");
    assert!(!st.done && !st.manual);
    assert_eq!(st.detail, "1 conta sem extrato recente: Poupança.");
    let ca = by(&c, "categories");
    assert!(!ca.done);
    assert_eq!(ca.detail, "1 lançamento do mês sem categoria.");
    assert!(ca.route.unwrap().contains(&format!("from={last}-01")));
    assert!(by(&c, "balances").manual && !by(&c, "balances").done);
    assert!(by(&c, "summary").manual && !by(&c, "summary").done);
    assert_eq!(c.closed_at, None);

    // Os passos manuais ficam no cofre; os medidos mudam com os dados.
    let set = |step: &str, done: bool| {
        closing::set(
            conn,
            &MonthCloseSetArgs {
                month: last.clone(),
                step: step.into(),
                done,
            },
            today,
        )
    };
    set("balances", true).unwrap();
    set("summary", true).unwrap();
    conn.execute(
        "UPDATE txn SET category_id = (SELECT id FROM category LIMIT 1) WHERE description = 'Sem categoria'",
        [],
    )
    .unwrap();
    let c = close(conn);
    assert!(by(&c, "balances").done && by(&c, "summary").done && by(&c, "categories").done);
    assert_eq!(by(&c, "categories").detail, "");
    assert_eq!(c.done, c.steps.iter().filter(|s| s.done).count() as i64);

    set("closed", true).unwrap();
    assert!(close(conn).closed_at.is_some());
    set("closed", false).unwrap();
    assert_eq!(close(conn).closed_at, None);

    assert!(set("qualquer", true).is_err());
    // O mês atual ainda não terminou.
    assert!(
        closing::set(
            conn,
            &MonthCloseSetArgs {
                month: m(0),
                step: "balances".into(),
                done: true,
            },
            today,
        )
        .is_err()
    );
}
