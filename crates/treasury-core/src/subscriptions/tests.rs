use super::*;
use crate::reports::fixture::{self, expense, new_db, write};

fn d(s: &str) -> NaiveDate {
    parse_date(s).unwrap()
}

#[test]
fn detects_monthly_yearly_new_late_and_price_increases() {
    let today = d("2026-10-15");
    let mut t = new_db();
    let card = write(&mut t.db, |tx| {
        let card = fixture::card(tx, "Cartão", 25, 5);
        for (date, amount) in [
            ("2026-06-12", 3_990),
            ("2026-07-12", 3_990),
            ("2026-08-12", 3_990),
            ("2026-09-12", 3_990),
            ("2026-10-12", 4_490),
        ] {
            expense(tx, &card, date, -amount, None, None, "NETFLIX.COM 0012345");
        }
        for date in ["2026-09-01", "2026-10-01"] {
            expense(tx, &card, date, -12_000, None, None, "Academia Forma");
        }
        for date in ["2025-08-20", "2026-08-20"] {
            expense(tx, &card, date, -16_680, None, None, "Amazon Prime anual");
        }
        for date in ["2026-05-20", "2026-06-20", "2026-07-20", "2026-08-20"] {
            expense(tx, &card, date, -2_190, None, None, "Spotify");
        }
        // Mercado: irregular, não é assinatura.
        for (date, amount) in [
            ("2026-09-03", 25_000),
            ("2026-09-10", 8_000),
            ("2026-09-28", 31_000),
            ("2026-10-09", 12_000),
        ] {
            expense(tx, &card, date, -amount, None, None, "Supermercado");
        }
        // Valor muito diferente a cada mês: não é assinatura.
        for (date, amount) in [
            ("2026-07-02", 5_000),
            ("2026-08-02", 20_000),
            ("2026-09-02", 9_000),
        ] {
            expense(tx, &card, date, -amount, None, None, "Farmácia");
        }
        Ok(card)
    });
    let r = detect(t.db.conn(), today).unwrap();
    let names: Vec<&str> = r.items.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Academia Forma",
            "NETFLIX.COM",
            "Spotify",
            "Amazon Prime anual"
        ]
    );
    let get = |n: &str| r.items.iter().find(|s| s.name == n).unwrap();
    let nf = get("NETFLIX.COM");
    assert_eq!(
        (nf.cadence, nf.status),
        (SubscriptionCadence::Monthly, SubscriptionStatus::Active)
    );
    assert_eq!((nf.amount, nf.previous_amount), (4_490, Some(3_990)));
    assert!((nf.change_pct.unwrap() - 0.1253).abs() < 0.001);
    assert_eq!(nf.next_expected, "2026-11-12");
    let rec = nf.suggested_recurrence.as_ref().unwrap();
    assert_eq!(
        (rec.amount, rec.start_date.as_str()),
        (-4_490, "2026-11-12")
    );
    assert_eq!(rec.spec.by_month_day, Some(12));
    assert_eq!(get("Academia Forma").status, SubscriptionStatus::New);
    let prime = get("Amazon Prime anual");
    assert_eq!(
        (prime.cadence, prime.monthly_cost, prime.annual_cost),
        (SubscriptionCadence::Yearly, 1_390, 16_680)
    );
    assert_eq!(get("Spotify").status, SubscriptionStatus::Late);
    // Total: Netflix + Academia + Prime/12 (Spotify atrasada fica de fora).
    assert_eq!(r.monthly_total, 4_490 + 12_000 + 1_390);
    assert_eq!((r.increases, r.new_count, r.late_count), (1, 1, 1));

    // Com recorrência cadastrada não há sugestão.
    write(&mut t.db, |tx| {
        crate::recurrence::create(
            tx,
            &serde_json::from_value(serde_json::json!({
                "description": "Netflix.com", "account_id": card, "amount": -4_490, "kind": "expense",
                "spec": {"freq": "MONTHLY", "by_month_day": 12}, "start_date": "2026-11-12"
            }))
            .unwrap(),
            today,
        )?;
        crate::settings::set(tx, DISMISSED_SETTING, &vec![get("Spotify").key.clone()])
    });
    let r = detect(t.db.conn(), today).unwrap();
    let nf = r.items.iter().find(|s| s.name == "NETFLIX.COM").unwrap();
    assert!(nf.recurrence_id.is_some() && nf.suggested_recurrence.is_none());
    assert!(r.items.iter().all(|s| s.name != "Spotify"));
    assert_eq!(r.dismissed.len(), 1);
}

#[test]
fn copies_generic_descriptions_and_same_series_in_two_accounts_count_once() {
    let today = d("2026-10-25");
    let mut t = new_db();
    let (card, checking, other) = write(&mut t.db, |tx| {
        let card = fixture::card(tx, "Cartão Nubank", 1, 8);
        let checking = fixture::account(tx, "Conta Nubank", "checking", "BRL", 0, "2026-01-01");
        let other = fixture::account(tx, "Conta Inter", "checking", "BRL", 0, "2026-01-01");
        // Seguro no cartão, com o CSV do cartão importado também na conta (mesmo id do banco).
        // A última cobrança foi em 08/09: a de 08/10 não veio.
        for (i, date) in ["2026-06-08", "2026-07-08", "2026-08-08", "2026-09-08"]
            .into_iter()
            .enumerate()
        {
            let ext = format!("68f1c0de-0000-4000-8000-00000000000{i}");
            for acc in [&card, &checking] {
                let id = expense(tx, acc, date, -2_990, None, None, "Seguro Residencial");
                tx.execute("UPDATE txn SET external_id = ?1 WHERE id = ?2", [&ext, &id])?;
            }
        }
        // Descrição genérica: contas diferentes pagas por débito em conta, de valor parecido.
        for (date, amount) in [
            ("2026-06-06", 15_000),
            ("2026-07-06", 16_000),
            ("2026-08-06", 15_500),
            ("2026-09-06", 15_800),
        ] {
            expense(tx, &checking, date, -amount, None, None, "Débito em conta");
        }
        // A mesma série em duas contas sem o id do banco (extrato importado na conta errada).
        for date in ["2026-07-15", "2026-08-15", "2026-09-15", "2026-10-15"] {
            expense(tx, &other, date, -1_990, None, None, "Spotify");
            expense(tx, &card, date, -1_990, None, None, "SPOTIFY");
        }
        Ok((card, checking, other))
    });
    let r = detect(t.db.conn(), today).unwrap();
    let found: Vec<(&str, &str, SubscriptionStatus)> = r
        .items
        .iter()
        .map(|s| (s.name.as_str(), s.account_id.as_str(), s.status))
        .collect();
    assert_eq!(
        found,
        [
            (
                "Seguro Residencial",
                card.as_str(),
                SubscriptionStatus::Late
            ),
            ("SPOTIFY", card.as_str(), SubscriptionStatus::Active),
        ]
    );
    assert_eq!((r.late_count, r.monthly_total), (1, 1_990));
    assert!(r.items.iter().all(|s| s.account_id != checking));

    // Dispensar a série pela chave da outra conta também a tira da lista.
    let spotify_other = format!("spotify|{other}");
    write(&mut t.db, |tx| {
        crate::settings::set(tx, DISMISSED_SETTING, &vec![spotify_other.clone()])
    });
    let r = detect(t.db.conn(), today).unwrap();
    let names: Vec<&str> = r.items.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Seguro Residencial"]);
}

/// Parcela de contrato (financiamento na categoria do contrato, obra ligada à parcela) não é
/// assinatura; a mensalidade da escola é conta fixa, fora do total das assinaturas; o
/// streaming é assinatura.
#[test]
fn contract_installments_are_out_and_school_is_a_fixed_bill() {
    let today = d("2026-10-15");
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let acc = fixture::account(tx, "Conta", "checking", "BRL", 0, "2025-01-01");
        let school = fixture::cat(tx, "Educação › Escola");
        let fun = fixture::cat(tx, "Lazer › Cinema e shows");
        let car = fixture::cat(tx, "Transporte › Manutenção do veículo");
        let now = crate::time::now_utc();
        let loan = |tx: &rusqlite::Transaction<'_>, name: &str, category: Option<&str>| {
            let id = crate::ids::new_id();
            tx.execute(
                "INSERT INTO loan (id, name, principal, annual_rate, system, contract_date, first_due_date,
                                   term_months, category_id, created_at, updated_at)
                 VALUES (?1, ?2, 3000000, '18', 'price', '2026-01-01', '2026-02-10', 48, ?3, ?4, ?4)",
                rusqlite::params![id, name, category, now],
            )
            .unwrap();
            id
        };
        loan(tx, "Veículo", Some(&car));
        let obra = loan(tx, "Obra", None);
        for (k, mo) in ["2026-06", "2026-07", "2026-08", "2026-09", "2026-10"]
            .iter()
            .enumerate()
        {
            expense(
                tx,
                &acc,
                &format!("{mo}-05"),
                -2_990,
                Some(&fun),
                None,
                "StreamFlix",
            );
            expense(
                tx,
                &acc,
                &format!("{mo}-08"),
                -150_000,
                Some(&school),
                None,
                "Boleto Instituto Exemplo",
            );
            expense(
                tx,
                &acc,
                &format!("{mo}-10"),
                -98_000,
                Some(&car),
                None,
                "Banco Exemplo Financiamento",
            );
            let paid = expense(
                tx,
                &acc,
                &format!("{mo}-12"),
                -75_000,
                None,
                None,
                "Construtora Exemplo",
            );
            tx.execute(
                "INSERT INTO loan_installment (id, loan_id, number, due_date, amortization, interest, total,
                                               balance_after, status, paid_date, paid_amount, txn_id,
                                               created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 50000, 25000, 75000, 0, 'paid', ?4, 75000, ?5, ?6, ?6)",
                rusqlite::params![
                    crate::ids::new_id(),
                    obra,
                    k as i64 + 1,
                    format!("{mo}-12"),
                    paid,
                    now
                ],
            )
            .unwrap();
        }
        Ok(())
    });
    let r = detect(t.db.conn(), today).unwrap();
    let names: Vec<&str> = r.items.iter().map(|s| s.name.as_str()).collect();
    assert!(
        !names.iter().any(|n| n.contains("Financiamento")),
        "{names:?}"
    );
    assert!(
        !names.iter().any(|n| n.contains("Construtora")),
        "{names:?}"
    );
    let get = |n: &str| r.items.iter().find(|s| s.name == n).unwrap();
    assert_eq!(get("StreamFlix").kind, SubscriptionKind::Subscription);
    assert_eq!(
        get("Boleto Instituto Exemplo").kind,
        SubscriptionKind::FixedBill
    );
    assert_eq!((r.monthly_total, r.fixed_monthly_total), (2_990, 150_000));
}
