use chrono::NaiveDate;
use serde_json::json;

use super::{ForecastArgs, daily_allowance, forecast};
use crate::reports::fixture::{self, cat, expense, new_db, write};
use crate::transactions::TxnKind;

fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

fn input<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> T {
    serde_json::from_value(v).unwrap()
}

#[test]
fn balance_forecast_from_recurrences_cards_and_loans() {
    let today = d("2026-10-15");
    let mut t = new_db();
    let (checking, card) = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 100_000, "2026-01-01");
        let card = fixture::card(tx, "Cartão", 25, 5);
        tx.execute(
            "UPDATE credit_card SET payment_account_id = ?1 WHERE account_id = ?2",
            [&checking, &card],
        )
        .unwrap();
        let salario = cat(tx, "Salário");
        let aluguel = cat(tx, "Moradia › Aluguel");
        for (desc, amount, kind, cat, day) in [
            ("Salário", 300_000, "income", &salario, 5),
            ("Aluguel", 350_000, "expense", &aluguel, 10),
        ] {
            crate::recurrence::create(
                tx,
                &input(json!({
                    "description": desc, "account_id": checking, "amount": amount,
                    "kind": kind, "category_id": cat,
                    "spec": {"freq": "MONTHLY", "by_month_day": day},
                    "start_date": format!("2026-11-{day:02}"),
                })),
                today,
            )
            .unwrap();
        }
        // Compra no cartão: fatura fecha dia 25 e vence 05/11, paga pela conta.
        expense(tx, &card, "2026-10-12", -20_000, None, None, "mercado");
        Ok((checking, card))
    });
    let f = forecast(t.db.conn(), &ForecastArgs::default(), today).unwrap();
    assert_eq!(
        (f.from.as_str(), f.to.as_str()),
        ("2026-10-15", "2027-01-15")
    );
    let acc = |id: &str| f.accounts.iter().find(|a| a.account_id == id).unwrap();
    let c = acc(&checking);
    assert_eq!(c.start, 100_000);
    let pts: Vec<(&str, i64)> = c
        .points
        .iter()
        .map(|p| (p.date.as_str(), p.balance))
        .collect();
    assert_eq!(
        pts,
        [
            ("2026-10-15", 100_000),
            ("2026-11-05", 380_000),
            ("2026-11-10", 30_000),
            ("2026-12-05", 330_000),
            ("2026-12-10", -20_000),
            ("2027-01-05", 280_000),
            ("2027-01-10", -70_000),
        ]
    );
    assert_eq!(c.first_negative.as_deref(), Some("2026-12-10"));
    assert_eq!((c.min, c.min_date.as_str()), (-70_000, "2027-01-10"));
    // Janeiro vem da recorrência além do horizonte gerado.
    assert!(
        c.events
            .iter()
            .any(|e| e.date == "2027-01-05" && e.source == super::ForecastSource::Recurrence)
    );
    let k = acc(&card);
    assert_eq!((k.start, k.end), (-20_000, 0));
    assert!(
        k.first_negative.is_none(),
        "cartão negativo é dívida, não alerta"
    );
    assert_eq!(f.total_start, 80_000);
    assert_eq!(f.total_end, -70_000);
    // Só uma conta, 12 meses.
    let only = forecast(
        t.db.conn(),
        &ForecastArgs {
            months: Some(12),
            account_ids: Some(vec![checking.clone()]),
        },
        today,
    )
    .unwrap();
    assert_eq!((only.accounts.len(), only.to.as_str()), (1, "2027-10-15"));
    assert!(
        forecast(
            t.db.conn(),
            &ForecastArgs {
                months: None,
                account_ids: Some(vec!["nao-existe".into()]),
            },
            today,
        )
        .is_err()
    );
}

#[test]
fn daily_allowance_divides_what_is_left_by_the_days_left() {
    let today = d("2026-10-15");
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
        let salario = cat(tx, "Salário");
        fixture::txn(
            tx,
            &checking,
            "2026-10-05",
            500_000,
            TxnKind::Income,
            Some(&salario),
            None,
            "salário",
        );
        expense(tx, &checking, "2026-10-14", -100_000, None, None, "mercado");
        expense(tx, &checking, "2026-10-15", -5_000, None, None, "padaria");
        let mut p = fixture::new_txn(
            tx,
            &checking,
            "2026-10-20",
            -30_000,
            TxnKind::Expense,
            None,
            None,
        );
        p.status = crate::transactions::TxnStatus::Planned;
        p.description = "internet".into();
        crate::transactions::insert(tx, &p).unwrap();
        crate::goals::create(
            tx,
            &input(
                json!({"name": "Reserva", "kind": "emergency", "target_amount": 1_000_000,
                          "monthly_contribution": 50_000}),
            ),
        )
        .unwrap();
        Ok(())
    });
    let a = daily_allowance(t.db.conn(), today).unwrap();
    assert_eq!(a.days_left, 17);
    assert_eq!(
        (
            a.income,
            a.spent,
            a.spent_today,
            a.fixed_remaining,
            a.savings_remaining
        ),
        (500_000, 105_000, 5_000, 30_000, 50_000)
    );
    // O que sobra depois de hoje; o de hoje divide o que sobrava no começo do dia (os
    // R$ 50 de hoje não saem duas vezes).
    assert_eq!(a.available, 315_000);
    assert_eq!(a.per_day, 320_000 / 17);
    assert_eq!(a.today_left, 320_000 / 17 - 5_000);
    assert_eq!(a.per_day_from_tomorrow, 315_000 / 16);
    assert_eq!(a.status, "ok");
    assert!(a.explanation.contains("17 dia(s)"));
    assert!(
        a.explanation.contains("gastos até ontem R$ 1.000,00"),
        "{}",
        a.explanation
    );
    assert!(
        a.explanation
            .contains("a partir de amanhã, R$ 196,87 por dia"),
        "{}",
        a.explanation
    );
    // No idioma da instalação.
    crate::i18n::set_current(crate::i18n::Locale::En);
    let en = daily_allowance(t.db.conn(), today).unwrap().explanation;
    crate::i18n::set_current(crate::i18n::Locale::PtBr);
    assert!(
        en.starts_with("This month's income R$5,000.00")
            && en.contains("for 17 days: R$188.23 a day."),
        "{en}"
    );
}

/// Relato de 20/10/2026 (cofre de demonstração): R$ 284,25 gastos hoje passam do valor do dia
/// (R$ 186,34). Hoje acabou, mas o mês ainda tem sobra: o valor de amanhã em diante aparece.
#[test]
fn today_used_up_still_gives_the_daily_amount_from_tomorrow() {
    let today = d("2026-10-20");
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
        let salario = cat(tx, "Salário");
        fixture::txn(
            tx,
            &checking,
            "2026-10-05",
            557_723,
            TxnKind::Income,
            Some(&salario),
            None,
            "salário",
        );
        expense(
            tx,
            &checking,
            "2026-10-12",
            -334_119,
            None,
            None,
            "mercado e contas",
        );
        expense(tx, &checking, "2026-10-20", -28_425, None, None, "farmácia");
        Ok(())
    });
    let a = daily_allowance(t.db.conn(), today).unwrap();
    assert_eq!(a.days_left, 12);
    assert_eq!(a.available, 557_723 - 334_119 - 28_425);
    assert_eq!(a.per_day, (557_723 - 334_119) / 12);
    assert_eq!(a.today_left, 0);
    assert_eq!(a.per_day_from_tomorrow, (557_723 - 334_119 - 28_425) / 11);
    assert_eq!(a.status, "ok");
}

#[test]
fn planned_card_purchases_leave_the_paying_account_on_the_due_date() {
    let today = d("2026-10-15");
    let mut t = new_db();
    let (checking, card) = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 100_000, "2026-01-01");
        let card = fixture::card(tx, "Cartão", 25, 5);
        tx.execute(
            "UPDATE credit_card SET payment_account_id = ?1 WHERE account_id = ?2",
            [&checking, &card],
        )
        .unwrap();
        // Assinatura no cartão todo dia 12: os previstos gerados (nov e dez) e, além do
        // horizonte gerado, a de janeiro (que vence em fevereiro, fora da janela).
        crate::recurrence::create(
            tx,
            &input(json!({
                "description": "Streaming", "account_id": card, "amount": -5_000,
                "kind": "expense",
                "spec": {"freq": "MONTHLY", "by_month_day": 12},
                "start_date": "2026-11-12",
            })),
            today,
        )
        .unwrap();
        Ok((checking, card))
    });
    let f = forecast(t.db.conn(), &ForecastArgs::default(), today).unwrap();
    let acc = |id: &str| f.accounts.iter().find(|a| a.account_id == id).unwrap();
    let pts: Vec<(&str, i64)> = acc(&checking)
        .points
        .iter()
        .map(|p| (p.date.as_str(), p.balance))
        .collect();
    assert_eq!(
        pts,
        [
            ("2026-10-15", 100_000),
            ("2026-12-05", 95_000),
            ("2027-01-05", 90_000),
        ]
    );
    // O cartão volta a zero depois de cada pagamento; a compra de janeiro fica na fatura.
    assert_eq!(acc(&card).end, -5_000);
    // Quatro meses: a de janeiro (da regra, além do horizonte gerado) é paga em fevereiro.
    let f = forecast(
        t.db.conn(),
        &ForecastArgs {
            months: Some(4),
            account_ids: None,
        },
        today,
    )
    .unwrap();
    let c = f
        .accounts
        .iter()
        .find(|a| a.account_id == checking)
        .unwrap();
    assert_eq!(c.end, 85_000);
    assert!(c.events.iter().any(|e| e.date == "2027-02-05"
        && e.amount == -5_000
        && e.source == super::ForecastSource::Recurrence));
}

#[test]
fn yearly_essential_expenses_come_back_as_estimates() {
    let today = d("2026-10-15");
    let mut t = new_db();
    let checking = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 2_000_000, "2025-09-01");
        let card = fixture::card(tx, "Cartão", 25, 5);
        let rent = cat(tx, "Moradia › Aluguel");
        let ipva = cat(tx, "Transporte › IPVA e licenciamento");
        let school = cat(tx, "Educação › Material escolar");
        let leisure = cat(tx, "Lazer › Passeios");
        // Doze meses completos de histórico (out/2025 a set/2026) com o aluguel todo mês.
        for i in 0..12 {
            let date = crate::time::add_months(d("2025-10-10"), i);
            expense(
                tx,
                &checking,
                &crate::time::fmt_date(date),
                -200_000,
                Some(&rent),
                None,
                "Aluguel",
            );
        }
        // IPVA: uma vez no ano, em novembro passado, pela conta. Volta em novembro deste ano.
        expense(
            tx,
            &checking,
            "2025-11-20",
            -180_000,
            Some(&ipva),
            None,
            "IPVA 2025",
        );
        // Material escolar no cartão: fica de fora (a fatura não é estimada).
        expense(
            tx,
            &card,
            "2025-12-05",
            -90_000,
            Some(&school),
            None,
            "Papelaria",
        );
        // Passeio único e não essencial: não é obrigação anual.
        expense(
            tx,
            &checking,
            "2025-12-20",
            -300_000,
            Some(&leisure),
            None,
            "Viagem",
        );
        Ok(checking)
    });
    let conn = t.db.conn();
    let f = forecast(
        conn,
        &ForecastArgs {
            months: Some(3),
            account_ids: None,
        },
        today,
    )
    .unwrap();
    let acc = f
        .accounts
        .iter()
        .find(|a| a.account_id == checking)
        .unwrap();
    let seasonal: Vec<_> = acc
        .events
        .iter()
        .filter(|e| e.source == super::ForecastSource::Seasonal)
        .collect();
    assert_eq!(seasonal.len(), 1, "{:?}", acc.events);
    assert_eq!(seasonal[0].date, "2026-11-20");
    assert_eq!(seasonal[0].amount, -180_000);
    assert_eq!(
        seasonal[0].description,
        "IPVA 2025 (estimativa pelo valor de 2025)"
    );
    assert_eq!(acc.end, acc.start - 180_000);
    // Nenhuma estimativa em outras contas.
    assert!(
        f.accounts
            .iter()
            .filter(|a| a.account_id != checking)
            .all(|a| a
                .events
                .iter()
                .all(|e| e.source != super::ForecastSource::Seasonal))
    );

    // Com um previsto na mesma categoria perto da data, a estimativa sai (nada em dobro).
    write(&mut t.db, |tx| {
        let ipva = cat(tx, "Transporte › IPVA e licenciamento");
        let mut p = fixture::new_txn(
            tx,
            &checking,
            "2026-11-25",
            -190_000,
            TxnKind::Expense,
            Some(&ipva),
            None,
        );
        p.status = crate::transactions::TxnStatus::Planned;
        p.description = "IPVA 2026".into();
        crate::transactions::insert(tx, &p)?;
        Ok(())
    });
    let f = forecast(
        t.db.conn(),
        &ForecastArgs {
            months: Some(3),
            account_ids: None,
        },
        today,
    )
    .unwrap();
    let acc = f
        .accounts
        .iter()
        .find(|a| a.account_id == checking)
        .unwrap();
    assert!(
        acc.events
            .iter()
            .all(|e| e.source != super::ForecastSource::Seasonal)
    );
    assert_eq!(acc.end, acc.start - 190_000);
}
