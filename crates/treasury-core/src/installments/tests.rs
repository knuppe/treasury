//! Parcelas detectadas: leitura da descrição, séries, projeção, previsão e "ignorar".

use super::*;
use crate::transactions::testkit::{Fx, d};

#[test]
fn parses_common_patterns() {
    let p = parse("Lj Exemplo Centro - Parcela 1/2").unwrap();
    assert_eq!(
        (p.number, p.total, p.keyword, p.base.as_str()),
        (1, 2, true, "Lj Exemplo Centro")
    );
    let p = parse("PREST.EMPREST 003/005").unwrap();
    assert_eq!((p.number, p.total, p.keyword, p.loan), (3, 5, true, true));
    let p = parse("Mercadolivre*Mercadol - Parcela 8/10").unwrap();
    assert_eq!(
        (p.number, p.total, p.base.as_str()),
        (8, 10, "Mercadolivre*Mercadol")
    );
    let p = parse("LOJA X PARC 02/12").unwrap();
    assert_eq!((p.number, p.total), (2, 12));
    let p = parse("Curso online parcela 3 de 6").unwrap();
    assert_eq!((p.number, p.total), (3, 6));
    let p = parse("PARC FATURA 03/10").unwrap();
    assert!(p.bill && !p.loan);
    // Sem palavra: lido, mas só vale com a série confirmada.
    let p = parse("PADARIA CENTRAL 03/11").unwrap();
    assert!(!p.keyword);
    // Fora: número maior que o total, total 1, nada no fim.
    assert!(parse("Parcela 5/3").is_none());
    assert!(parse("Parcela 1/1").is_none());
    assert!(parse("Mercado Bom Preço").is_none());
}

fn buy(fx: &mut Fx, acc: &str, date: &str, cents: i64, desc: &str) -> Id {
    fx.txn(acc, date, -cents, desc)
}

#[test]
fn card_series_projects_the_remaining_installments() {
    let mut fx = Fx::new();
    let bank = fx.checking("Corrente", 1_000_000);
    // Fecha dia 3, vence dia 10, paga pela corrente.
    let card = fx.card("Cartão", 3, 10, None);
    fx.write(|tx| {
        tx.execute(
            "UPDATE credit_card SET payment_account_id = ?2 WHERE account_id = ?1",
            [&card, &bank],
        )?;
        Ok(())
    })
    .unwrap();
    buy(
        &mut fx,
        &card,
        "2026-09-10",
        10_000,
        "Mercadolivre*Mercadol - Parcela 7/10",
    );
    buy(
        &mut fx,
        &card,
        "2026-10-10",
        10_000,
        "Mercadolivre*Mercadol - Parcela 8/10",
    );
    // Data solta sem confirmação não vira série.
    buy(&mut fx, &card, "2026-10-12", 4_000, "PADARIA CENTRAL 12/10");

    let r = detect(fx.conn(), d("2026-10-20")).unwrap();
    assert_eq!(r.series.len(), 1, "{:?}", r.series);
    let s = &r.series[0];
    assert_eq!(
        (s.kind, s.number, s.total, s.remaining),
        (SeriesKind::Card, 8, 10, 2)
    );
    assert_eq!(s.remaining_amount, 20_000);
    assert!(s.confirmed_by_sequence);
    // A 8ª está na fatura de novembro (compras até 03/11); as próximas em dezembro e janeiro.
    let dues: Vec<&str> = s
        .projected
        .iter()
        .map(|p| p.due_date.as_deref().unwrap())
        .collect();
    assert_eq!(dues, ["2026-12-10", "2027-01-10"]);
    assert_eq!(s.end_month, "2027-01");

    // Entram na previsão: compra no cartão e pagamento pela corrente no vencimento.
    let f = crate::forecast::forecast(
        fx.conn(),
        &crate::forecast::ForecastArgs {
            months: Some(4),
            account_ids: None,
        },
        d("2026-10-20"),
    )
    .unwrap();
    let bank_events: Vec<(&str, i64)> = f
        .accounts
        .iter()
        .find(|a| a.account_id == bank)
        .unwrap()
        .events
        .iter()
        .filter(|e| e.source == crate::forecast::ForecastSource::Detected)
        .map(|e| (e.date.as_str(), e.amount))
        .collect();
    assert_eq!(
        bank_events,
        [("2026-12-10", -10_000), ("2027-01-10", -10_000)]
    );

    // Compromissos futuros do cartão também contam.
    let c = crate::cards::future_commitments(fx.conn(), Some(&card), 4, d("2026-10-20")).unwrap();
    let dec = c.iter().find(|m| m.month == "2026-12").unwrap();
    assert!(dec.amount >= 10_000);

    // Ignorar tira da lista e da previsão; desfazer traz de volta.
    let key = s.key.clone();
    fx.write(|tx| ignore(tx, s)).unwrap();
    assert!(
        detect(fx.conn(), d("2026-10-20"))
            .unwrap()
            .series
            .is_empty()
    );
    fx.write(|tx| unignore(tx, &key)).unwrap();
    assert_eq!(detect(fx.conn(), d("2026-10-20")).unwrap().series.len(), 1);
}

#[test]
fn loan_on_the_account_and_missing_installment() {
    let mut fx = Fx::new();
    let bank = fx.checking("Corrente", 1_000_000);
    buy(
        &mut fx,
        &bank,
        "2026-09-05",
        50_000,
        "PREST.EMPREST 003/005",
    );
    let r = detect(fx.conn(), d("2026-09-20")).unwrap();
    let s = &r.series[0];
    assert_eq!((s.kind, s.number, s.remaining), (SeriesKind::Loan, 3, 2));
    let dates: Vec<&str> = s.projected.iter().map(|p| p.date.as_str()).collect();
    assert_eq!(dates, ["2026-10-05", "2026-11-05"]);
    // A 4ª não apareceu: aviso entre 15 e 45 dias de atraso; depois sai.
    let late = detect(fx.conn(), d("2026-10-25")).unwrap();
    assert!(late.series[0].missing);
    assert!(
        detect(fx.conn(), d("2026-11-25"))
            .unwrap()
            .series
            .is_empty()
    );
    // Última parcela: nada a projetar.
    buy(
        &mut fx,
        &bank,
        "2026-10-05",
        50_000,
        "PREST.EMPREST 005/005",
    );
    assert!(
        detect(fx.conn(), d("2026-10-06"))
            .unwrap()
            .series
            .is_empty()
    );
}

#[test]
fn two_purchases_in_the_same_store_are_two_series() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    // Duas compras em 10x na mesma loja, uma em agosto e outra em setembro (outro valor), e
    // duas iguais no mesmo mês (o mesmo produto duas vezes).
    for (date, n) in [("2026-08-10", 1), ("2026-09-10", 2), ("2026-10-10", 3)] {
        buy(&mut fx, &card, date, 10_000, &format!("LOJA X PARC {n}/10"));
    }
    for (date, n) in [("2026-09-11", 1), ("2026-10-11", 2)] {
        buy(&mut fx, &card, date, 25_000, &format!("LOJA X PARC {n}/10"));
        buy(&mut fx, &card, date, 25_000, &format!("LOJA X PARC {n}/10"));
    }
    let r = detect(fx.conn(), d("2026-10-20")).unwrap();
    let mut got: Vec<(u32, i64)> = r.series.iter().map(|s| (s.number, s.amount)).collect();
    got.sort();
    assert_eq!(
        got,
        [(2, 25_000), (2, 25_000), (3, 10_000)],
        "{:?}",
        r.series
    );
    let keys: std::collections::HashSet<&str> = r.series.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys.len(), 3, "cada compra com a sua chave");
    assert_eq!(r.remaining_total, 7 * 10_000 + 2 * 8 * 25_000);
}

#[test]
fn closed_statement_not_imported_still_pays_on_the_due_date() {
    let mut fx = Fx::new();
    let bank = fx.checking("Corrente", 1_000_000);
    let card = fx.card("Cartão", 3, 10, None);
    fx.write(|tx| {
        tx.execute(
            "UPDATE credit_card SET payment_account_id = ?2 WHERE account_id = ?1",
            [&card, &bank],
        )?;
        Ok(())
    })
    .unwrap();
    buy(&mut fx, &card, "2026-09-10", 10_000, "LOJA Y Parcela 7/10");
    buy(&mut fx, &card, "2026-10-10", 10_000, "LOJA Y Parcela 8/10");
    // A fatura com a 9ª fechou em 03/12 e vence em 10/12; o extrato ainda não veio.
    let today = d("2026-12-05");
    let r = detect(fx.conn(), today).unwrap();
    let s = &r.series[0];
    assert!(!s.missing);
    let dues: Vec<&str> = s
        .projected
        .iter()
        .map(|p| p.due_date.as_deref().unwrap())
        .collect();
    assert_eq!(dues, ["2026-12-10", "2027-01-10"]);
    let f = crate::forecast::forecast(
        fx.conn(),
        &crate::forecast::ForecastArgs {
            months: Some(2),
            account_ids: None,
        },
        today,
    )
    .unwrap();
    let events = |id: &str| -> Vec<(String, i64)> {
        f.accounts
            .iter()
            .find(|a| a.account_id == id)
            .unwrap()
            .events
            .iter()
            .filter(|e| e.source == crate::forecast::ForecastSource::Detected)
            .map(|e| (e.date.clone(), e.amount))
            .collect()
    };
    assert_eq!(
        events(&bank),
        [
            ("2026-12-10".to_string(), -10_000),
            ("2027-01-10".to_string(), -10_000)
        ]
    );
    // No cartão, só a 10ª (a 9ª já está na fatura fechada, fora do saldo até a importação).
    assert_eq!(
        events(&card),
        [
            ("2027-01-03".to_string(), -10_000),
            ("2027-01-10".to_string(), 10_000)
        ]
    );
}
