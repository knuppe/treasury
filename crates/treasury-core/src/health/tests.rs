//! Saúde financeira com famílias de teste montadas em relação a hoje (a reserva do mês atual
//! vem de `goals::emergency_reserve`, que usa a data do sistema).

use rusqlite::{Transaction, params};
use serde_json::json;

use super::*;
use crate::ids::{Id, new_id};
use crate::reports::engine::{month_dates, shift_month};
use crate::reports::fixture::{self, TestDb, account, card, cat, expense, new_db, new_txn, txn};
use crate::time::{fmt_date, now_utc, today_local};
use crate::transactions::{TxnKind, TxnStatus, insert};

/// Conferência de saldo de hoje: a conta fica em dia sem mudar o saldo nem os fluxos.
fn fresh(tx: &Transaction<'_>, account: &str) {
    tx.execute(
        "INSERT INTO balance_sync (id, account_id, date, statement_balance, computed_balance,
                                   difference, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, 0, 0, ?4, ?4)",
        params![new_id(), account, fmt_date(today_local()), now_utc()],
    )
    .unwrap();
}

fn cur() -> String {
    month_of(today_local())
}

fn m(i: i32) -> String {
    shift_month(&cur(), -i).unwrap()
}

fn day(month: &str, d: u32) -> String {
    format!("{month}-{d:02}")
}

fn metric<'a>(r: &'a HealthReport, key: &str) -> &'a HealthMetric {
    r.metrics
        .iter()
        .find(|m| m.key == key)
        .unwrap_or_else(|| panic!("indicador {key}"))
}

fn report(t: &TestDb, model: HealthModel, as_of: Option<String>) -> HealthReport {
    score_with(
        t.db.conn(),
        model,
        &HealthScoreArgs {
            as_of,
            member_id: None,
        },
        today_local(),
    )
    .unwrap()
}

/// Empréstimo com uma parcela por mês (vencimento no dia 10, metade juros) nos meses
/// `months` (do mais antigo ao mais novo); as `unpaid` últimas ficam em aberto.
fn loan(tx: &Transaction<'_>, kind: &str, months: &[String], amount: i64, unpaid: usize) -> Id {
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO loan (id, name, kind, principal, annual_rate, system, contract_date, first_due_date,
                           term_months, created_at, updated_at)
         VALUES (?1, 'Empréstimo', ?2, ?3, '20', 'price', ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            kind,
            amount / 2 * months.len() as i64,
            day(&months[0], 1),
            day(&months[0], 10),
            months.len() as i64,
            now
        ],
    )
    .unwrap();
    let n = months.len();
    for (k, mo) in months.iter().enumerate() {
        let paid = k < n - unpaid;
        let due = day(mo, 10);
        tx.execute(
            "INSERT INTO loan_installment (id, loan_id, number, due_date, amortization, interest, total,
                                           balance_after, status, paid_date, paid_amount, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?11, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![
                new_id(),
                id,
                k as i64 + 1,
                due,
                amount / 2,
                amount / 2 * (n - k - 1) as i64,
                if paid { "paid" } else { "planned" },
                paid.then(|| due.clone()),
                paid.then_some(amount),
                now,
                amount
            ],
        )
        .unwrap();
    }
    id
}

/// Paga `share` (0–1) de cada fatura do cartão com vencimento antes de hoje, no vencimento.
fn pay_statements(tx: &Transaction<'_>, card: &str, from: &str, share: f64) {
    let today = crate::time::fmt_date(today_local());
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = tx
            .prepare(
                "SELECT s.id, s.due_date, COALESCE(SUM(-t.amount), 0) FROM card_statement s
                   JOIN txn t ON t.statement_id = s.id
                  WHERE s.account_id = ?1 AND s.due_date < ?2 GROUP BY s.id",
            )
            .unwrap();
        stmt.query_map(params![card, today], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    for (sid, due, total) in rows {
        let amount = (total as f64 * share).round() as i64;
        let tid = new_id();
        let mut out = new_txn(tx, from, &due, -amount, TxnKind::CardPayment, None, None);
        out.transfer_id = Some(tid.clone());
        insert(tx, &out).unwrap();
        let mut inn = new_txn(tx, card, &due, amount, TxnKind::CardPayment, None, None);
        inn.transfer_id = Some(tid);
        inn.statement_id = Some(sid);
        insert(tx, &inn).unwrap();
    }
}

/// Família organizada: salário de R$ 10.000, gastos de R$ 4.800 (aluguel, mercado e a
/// parcela do carro de R$ 1.000 em dia, essenciais), cartão pago inteiro, orçamento
/// cumprido e 14 meses de histórico.
fn good_household() -> TestDb {
    let mut t = new_db();
    fixture::write(&mut t.db, |tx| {
        let checking = account(tx, "Corrente", "checking", "BRL", 500_000, &day(&m(14), 1));
        let savings = account(tx, "Poupança", "savings", "BRL", 0, &day(&m(14), 1));
        let nubank = card(tx, "Cartão", 25, 5);
        for a in [&checking, &savings, &nubank] {
            fresh(tx, a);
        }
        let (salary, rent, food, fun) = (
            cat(tx, "Salário"),
            cat(tx, "Moradia › Aluguel"),
            cat(tx, "Alimentação › Mercado"),
            cat(tx, "Lazer › Passeios"),
        );
        let months: Vec<String> = (1..=14).rev().map(m).collect();
        for mo in &months {
            txn(
                tx,
                &checking,
                &day(mo, 5),
                1_000_000,
                TxnKind::Income,
                Some(&salary),
                None,
                "Salário",
            );
            expense(
                tx,
                &checking,
                &day(mo, 8),
                -250_000,
                Some(&rent),
                None,
                "Aluguel",
            );
            expense(
                tx,
                &checking,
                &day(mo, 12),
                -100_000,
                Some(&food),
                None,
                "Mercado",
            );
            expense(
                tx,
                &nubank,
                &day(mo, 10),
                -30_000,
                Some(&fun),
                None,
                "Cinema",
            );
            expense(
                tx,
                &checking,
                &day(mo, 10),
                -100_000,
                Some(&cat(tx, "Transporte")),
                None,
                "Parcela do carro",
            );
            fixture::transfer(
                tx,
                &checking,
                &savings,
                &day(mo, 6),
                300_000,
                TxnKind::Transfer,
                TxnStatus::Cleared,
            );
        }
        loan(tx, "vehicle", &months, 100_000, 0);
        pay_statements(tx, &nubank, &checking, 1.0);
        for mo in &months[months.len() - 3..] {
            crate::budget::set_lines(
                tx,
                &crate::budget::SetBudgetArgs {
                    month: mo.clone(),
                    lines: vec![crate::budget::BudgetLineInput {
                        category_id: cat(tx, "Moradia"),
                        amount: 600_000,
                    }],
                },
            )?;
        }
        Ok(())
    });
    t
}

/// Família no aperto: renda de R$ 5.000, gastos maiores que a renda, parcela de R$ 2.000
/// com atrasos, fatura paga pela metade (rotativo), conta corrente negativa e sem reserva.
fn bad_household() -> TestDb {
    let mut t = new_db();
    fixture::write(&mut t.db, |tx| {
        let checking = account(tx, "Corrente", "checking", "BRL", 100_000, &day(&m(14), 1));
        let nubank = card(tx, "Cartão", 25, 5);
        for a in [&checking, &nubank] {
            fresh(tx, a);
        }
        let (salary, rent, food, fun) = (
            cat(tx, "Salário"),
            cat(tx, "Moradia › Aluguel"),
            cat(tx, "Alimentação › Mercado"),
            cat(tx, "Lazer › Passeios"),
        );
        let months: Vec<String> = (1..=14).rev().map(m).collect();
        for mo in &months {
            txn(
                tx,
                &checking,
                &day(mo, 5),
                500_000,
                TxnKind::Income,
                Some(&salary),
                None,
                "Salário",
            );
            expense(
                tx,
                &checking,
                &day(mo, 8),
                -200_000,
                Some(&rent),
                None,
                "Aluguel",
            );
            expense(
                tx,
                &checking,
                &day(mo, 12),
                -150_000,
                Some(&food),
                None,
                "Mercado",
            );
            expense(
                tx,
                &nubank,
                &day(mo, 14),
                -100_000,
                Some(&fun),
                None,
                "Shopping",
            );
            expense(
                tx,
                &checking,
                &day(mo, 10),
                -200_000,
                Some(&cat(tx, "Outras despesas")),
                None,
                "Parcela",
            );
        }
        loan(tx, "personal", &months, 200_000, 3);
        pay_statements(tx, &nubank, &checking, 0.5);
        Ok(())
    });
    t
}

#[test]
fn isf_good_household_scores_high_with_every_metric_good() {
    let t = good_household();
    let r = report(&t, HealthModel::Isf, None);
    assert!(r.enabled);
    assert_eq!(r.months_analyzed, 12);
    assert_eq!(r.period.as_ref().unwrap().to, month_dates(&m(1)).unwrap().1);
    for mt in &r.metrics {
        assert_eq!(
            mt.status,
            HealthStatus::Good,
            "{}: {:?}",
            mt.key,
            mt.value_text
        );
        assert!(mt.suggestions.is_empty(), "{}", mt.key);
        assert!(!mt.computation.is_empty() && !mt.explanation.is_empty());
    }
    // 4.800 de despesas sobre 10.000 (a transferência para a poupança não conta).
    let sr = metric(&r, "taxa_poupanca");
    assert!((sr.value.unwrap() - 0.52).abs() < 1e-9, "{:?}", sr.value);
    assert_eq!(sr.value_text.as_deref(), Some("52,0%"));
    assert_eq!(sr.amounts.len(), 3);
    assert!((metric(&r, "comprometimento").value.unwrap() - 0.10).abs() < 1e-9);
    assert_eq!(metric(&r, "contas_em_dia").value, Some(1.0));
    assert_eq!(
        metric(&r, "orcamento").value_text.as_deref(),
        Some("3 de 3 meses")
    );
    assert_eq!(metric(&r, "credito_caro").value, Some(0.0));
    assert!(metric(&r, "reserva").value.unwrap() >= 6.0);
    let score = r.score.unwrap();
    assert!(score >= 83.0, "{score}");
    assert_eq!(r.band.as_ref().unwrap().label, "Ótima");
    assert_eq!(r.dimensions.len(), 4);
    assert!(r.dimensions.iter().all(|d| d.status == HealthStatus::Good));
    assert!(
        r.summary.contains("acima da média nacional de 56,7"),
        "{}",
        r.summary
    );
    assert_eq!(r.reference.as_ref().unwrap().score, 56.7);
    assert_eq!(r.bands.len(), 7);
}

#[test]
fn ratios_good_household_is_healthy() {
    let t = good_household();
    let r = report(&t, HealthModel::Ratios, None);
    assert!(r.dimensions.is_empty() && r.reference.is_none());
    assert_eq!(r.metrics.len(), 6);
    // Aluguel de 2.500 sobre 10.000.
    assert!((metric(&r, "moradia").value.unwrap() - 0.25).abs() < 1e-9);
    assert!(r.metrics.iter().all(|m| m.status == HealthStatus::Good));
    assert_eq!(r.score, Some(100.0));
    assert_eq!(r.band.as_ref().unwrap().label, "Saudável");
    let weights: f64 = r.metrics.iter().map(|m| m.weight).sum();
    assert!((weights - 1.0).abs() < 1e-9);
}

#[test]
fn bad_household_scores_low_with_suggestions() {
    let t = bad_household();
    let r = report(&t, HealthModel::Isf, None);
    let score = r.score.unwrap();
    assert!(score < 50.0, "{score}");
    assert!(
        ["Ruim", "Muito baixa"].contains(&r.band.as_ref().unwrap().label.as_str()),
        "{:?}",
        r.band
    );
    // (5.000 − 6.500) ÷ 5.000.
    let sr = metric(&r, "taxa_poupanca");
    assert!((sr.value.unwrap() + 0.30).abs() < 1e-9, "{:?}", sr.value);
    assert_eq!(sr.status, HealthStatus::Bad);
    assert!(!sr.suggestions.is_empty() && sr.suggestions.len() <= 3);
    assert!((metric(&r, "comprometimento").value.unwrap() - 0.40).abs() < 1e-9);
    assert_eq!(metric(&r, "comprometimento").status, HealthStatus::Bad);
    let credit = metric(&r, "credito_caro");
    assert_eq!(credit.status, HealthStatus::Bad);
    assert!(credit.value.unwrap() >= 6.0, "{:?}", credit.value_text);
    assert!(credit.computation.contains("cheque especial em"));
    assert!(credit.computation.contains("fatura não paga inteira"));
    let on_time = metric(&r, "contas_em_dia");
    assert!(on_time.value.unwrap() < 0.9, "{:?}", on_time.value_text);
    assert_eq!(on_time.status, HealthStatus::Bad);
    assert_eq!(metric(&r, "reserva").status, HealthStatus::Bad);
    assert_eq!(metric(&r, "patrimonio").status, HealthStatus::Bad);
    assert_eq!(metric(&r, "meses_no_azul").value, Some(0.0));
    // Sem orçamento: sem dados, fora da nota (não penaliza).
    let budget = metric(&r, "orcamento");
    assert_eq!(budget.status, HealthStatus::Unknown);
    assert_eq!(budget.points, None);
    assert_eq!(budget.suggestions.len(), 1);
    assert!(
        r.summary.contains("abaixo da média nacional"),
        "{}",
        r.summary
    );

    let r = report(&t, HealthModel::Ratios, None);
    assert!(r.score.unwrap() < 40.0, "{:?}", r.score);
    assert_eq!(r.band.as_ref().unwrap().label, "Crítica");
    // 2.000 de aluguel sobre 5.000: 40% (atenção).
    assert_eq!(metric(&r, "moradia").status, HealthStatus::Ok);
    assert!(r.summary.contains("Ponto mais fraco"), "{}", r.summary);
}

#[test]
fn short_history_is_unknown_instead_of_penalized() {
    let mut t = new_db();
    fixture::write(&mut t.db, |tx| {
        let checking = account(tx, "Corrente", "checking", "BRL", -50_000, &day(&m(2), 1));
        for mo in [m(2), m(1)] {
            txn(
                tx,
                &checking,
                &day(&mo, 5),
                100_000,
                TxnKind::Income,
                Some(&cat(tx, "Salário")),
                None,
                "Salário",
            );
            expense(
                tx,
                &checking,
                &day(&mo, 6),
                -300_000,
                Some(&cat(tx, "Moradia › Aluguel")),
                None,
                "Aluguel",
            );
        }
        Ok(())
    });
    for model in [HealthModel::Isf, HealthModel::Ratios] {
        let r = report(&t, model, None);
        assert_eq!(r.months_analyzed, 2);
        assert_eq!(r.score, None, "{model:?}");
        assert!(r.band.is_none());
        assert!(
            r.summary.starts_with("Dados insuficientes"),
            "{}",
            r.summary
        );
        for mt in &r.metrics {
            assert_eq!(mt.status, HealthStatus::Unknown, "{}", mt.key);
            assert_eq!((mt.points, mt.value), (None, None), "{}", mt.key);
            assert!(
                mt.computation.starts_with("Sem dados"),
                "{}: {}",
                mt.key,
                mt.computation
            );
        }
        assert!(r.notes.iter().any(|n| n.contains("em vez de penalizar")));
    }
    // Cofre sem nenhum lançamento.
    let t = new_db();
    let r = report(&t, HealthModel::Isf, None);
    assert_eq!(
        (r.months_analyzed, r.score, r.period.is_none()),
        (0, None, true)
    );
}

#[test]
fn past_month_ends_the_window_there_and_off_disables() {
    let t = good_household();
    let r = report(&t, HealthModel::Isf, Some(m(3)));
    assert_eq!(r.as_of, m(3));
    assert_eq!(r.months_analyzed, 12);
    assert_eq!(
        r.period.as_ref().unwrap().from,
        month_dates(&m(14)).unwrap().0
    );
    assert_eq!(r.period.as_ref().unwrap().to, month_dates(&m(3)).unwrap().1);
    assert!(r.score.unwrap() >= 83.0);
    // Mês futuro: recusado.
    let err = score_with(
        t.db.conn(),
        HealthModel::Isf,
        &HealthScoreArgs {
            as_of: Some(shift_month(&cur(), 1).unwrap()),
            member_id: None,
        },
        today_local(),
    );
    assert!(err.is_err());
    let off = report(&t, HealthModel::Off, None);
    assert!(!off.enabled);
    assert!(off.metrics.is_empty() && off.dimensions.is_empty() && off.score.is_none());
}

#[test]
fn mortgage_outside_housing_counts_as_housing() {
    let mut t = new_db();
    fixture::write(&mut t.db, |tx| {
        let checking = account(tx, "Corrente", "checking", "BRL", 0, &day(&m(6), 1));
        let months: Vec<String> = (1..=6).rev().map(m).collect();
        for mo in &months {
            txn(
                tx,
                &checking,
                &day(mo, 5),
                1_000_000,
                TxnKind::Income,
                Some(&cat(tx, "Salário")),
                None,
                "Salário",
            );
            expense(
                tx,
                &checking,
                &day(mo, 9),
                -100_000,
                Some(&cat(tx, "Moradia › Condomínio")),
                None,
                "Condomínio",
            );
        }
        loan(tx, "mortgage", &months, 250_000, 0);
        Ok(())
    });
    let r = report(&t, HealthModel::Ratios, None);
    // (1.000 de condomínio + 2.500 da parcela) ÷ 10.000.
    assert!((metric(&r, "moradia").value.unwrap() - 0.35).abs() < 1e-9);
    assert_eq!(metric(&r, "moradia").status, HealthStatus::Ok);
}

#[test]
fn commands_settings_and_permissions() {
    use crate::api::{Actor, Api};
    use crate::paths::TreasuryPaths;
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    let s = api.call("health.get_settings", json!({})).unwrap();
    assert_eq!(s["model"], "isf");
    assert_eq!(s["models"].as_array().unwrap().len(), 3);
    assert_eq!(s["models"][0]["source_url"], SOURCE_URL);
    let r = api.call("health.score", json!({})).unwrap();
    assert_eq!(r["model"], "isf");
    assert_eq!(r["enabled"], true);

    let reader = Actor::Mcp {
        client: "cli-teste".into(),
        read_only: true,
    };
    let writer = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(api.call_as(&reader, "health.score", json!({})).is_ok());
    assert!(
        api.call_as(&reader, "health.get_settings", json!({}))
            .is_ok()
    );
    for actor in [&reader, &writer, &Actor::System] {
        let e = api
            .call_as(actor, "health.set_settings", json!({"model": "off"}))
            .unwrap_err();
        assert!(matches!(e, crate::Error::Forbidden(_)), "{actor:?}: {e}");
    }
    assert!(
        api.call("health.set_settings", json!({"model": "bogus"}))
            .is_err()
    );
    let s = api
        .call("health.set_settings", json!({"model": "ratios"}))
        .unwrap();
    assert_eq!(s["model"], "ratios");
    assert_eq!(
        api.call("health.score", json!({})).unwrap()["model"],
        "ratios"
    );
    api.call("health.set_settings", json!({"model": "off"}))
        .unwrap();
    let r = api
        .call("health.score", json!({"as_of": "2026-01"}))
        .unwrap();
    assert_eq!(r["enabled"], false);
    assert_eq!(r["as_of"], "2026-01");
}

/// Os textos saem no idioma da instalação, com números e meses no formato dele; as chaves, os
/// valores e as notas não mudam.
#[test]
fn texts_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let t = bad_household();
    set_current(Locale::En);
    let r = report(&t, HealthModel::Isf, None);
    set_current(Locale::PtBr);
    assert!(
        r.summary.contains("below the national average of 56.7"),
        "{}",
        r.summary
    );
    assert!(r.summary.contains("Weakest dimension:"), "{}", r.summary);
    let sr = metric(&r, "taxa_poupanca");
    assert_eq!(sr.label, "Savings rate");
    assert_eq!(sr.value_text.as_deref(), Some("−30.0%"));
    assert_eq!(sr.amounts[0].label, "Income");
    let credit = metric(&r, "credito_caro");
    assert!(
        credit.computation.contains("overdraft in"),
        "{}",
        credit.computation
    );
    assert!(
        credit
            .value_text
            .as_deref()
            .unwrap()
            .contains(" of 12 months")
    );
    assert_eq!(metric(&r, "orcamento").suggestions.len(), 1);
    assert!(metric(&r, "orcamento").suggestions[0].contains("05 · Budget"));
    assert_eq!(r.dimensions[0].label, "Behavior");
    assert_eq!(
        r.reference.as_ref().unwrap().short_label,
        "National average"
    );
    assert!(["Poor", "Very low"].contains(&r.band.as_ref().unwrap().label.as_str()));

    set_current(Locale::Es);
    let models = models();
    set_current(Locale::PtBr);
    assert_eq!(models[1].label, "Indicadores clásicos");
    assert_eq!(models[2].label, "Desactivado");
}

/// Conta negativa por 30 dias ou mais (até hoje, no mês corrente) entra no crédito caro, como
/// no insight de cheque especial; sem o limite cadastrado, a conta diz que pode ser cheque
/// especial ou importação incompleta. Com o limite, só o uso.
#[test]
fn long_negative_balance_counts_as_expensive_credit() {
    let mut t = good_household();
    let imported = fixture::write(&mut t.db, |tx| {
        // Importada sem o saldo inicial: só saídas desde o começo do mês passado.
        let acc = account(tx, "Conta importada", "checking", "BRL", 0, &day(&m(14), 1));
        fresh(tx, &acc);
        txn(
            tx,
            &acc,
            &day(&m(1), 1),
            -50_000,
            TxnKind::Expense,
            None,
            None,
            "Pix enviado",
        );
        Ok(acc)
    });
    let r = report(&t, HealthModel::Isf, None);
    let credit = metric(&r, "credito_caro");
    assert!(credit.value.unwrap() >= 2.0, "{:?}", credit.value_text);
    assert!(
        credit
            .computation
            .contains("possível cheque especial ou importação incompleta em Conta importada"),
        "{}",
        credit.computation
    );
    // Com o cheque especial cadastrado: é uso do limite, sem a dúvida.
    fixture::write(&mut t.db, |tx| {
        crate::accounts::overdraft::upsert(
            tx,
            &imported,
            &crate::accounts::overdraft::OverdraftTerms {
                limit: 100_000,
                ..Default::default()
            },
        )?;
        Ok(())
    });
    let credit = metric(&report(&t, HealthModel::Isf, None), "credito_caro").clone();
    assert!(credit.value.unwrap() >= 2.0);
    assert!(
        !credit.computation.contains("importação incompleta"),
        "{}",
        credit.computation
    );
    assert!(credit.computation.contains("cheque especial em"));
}

/// Conta do dia a dia sem novidade há semanas: a nota sai estimada, a conta aparece em
/// `stale_inputs`, e o saldo negativo dela não vira cheque especial nem suspeita.
#[test]
fn stale_account_makes_the_score_estimated_and_its_negative_is_not_overdraft() {
    let mut t = good_household();
    assert!(report(&t, HealthModel::Isf, None).stale_inputs.is_empty());
    fixture::write(&mut t.db, |tx| {
        let acc = account(tx, "Conta parada", "checking", "BRL", 0, &day(&m(14), 1));
        txn(
            tx,
            &acc,
            &day(&m(2), 1),
            -50_000,
            TxnKind::Expense,
            None,
            None,
            "Pix enviado",
        );
        Ok(())
    });
    let r = report(&t, HealthModel::Isf, None);
    assert!(r.estimated);
    assert_eq!(r.stale_inputs.len(), 1, "{:?}", r.stale_inputs);
    let s = &r.stale_inputs[0];
    assert_eq!(
        (s.name.as_str(), s.negative, s.balance),
        ("Conta parada", true, -50_000)
    );
    assert!(
        r.notes.iter().any(|n| n.contains("Conta parada")),
        "{:?}",
        r.notes
    );
    let credit = metric(&r, "credito_caro");
    assert!(
        !credit.computation.contains("Conta parada"),
        "{}",
        credit.computation
    );
    // Um mês passado não olha a atualização de hoje.
    assert!(
        report(&t, HealthModel::Isf, Some(m(1)))
            .stale_inputs
            .is_empty()
    );
}
