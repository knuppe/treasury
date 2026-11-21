use super::*;
use crate::cards::InstallmentInput;
use crate::transactions::testkit::{Fx, d};
use crate::transactions::{TxnPatch, TxnStatus};

fn base_input(name: &str, kind: AccountKind) -> AccountInput {
    AccountInput {
        name: name.into(),
        kind,
        currency: None,
        institution_id: None,
        institution_name: None,
        member_ids: None,
        opening_balance: None,
        opening_date: None,
        include_in_net_worth: None,
        color: None,
        notes: None,
        card: None,
        overdraft: None,
        branch: None,
        account_number: None,
        retirement_plan: None,
        retirement_coverage: None,
    }
}

fn card_details(closing: u32, due: u32) -> CardDetails {
    CardDetails {
        credit_limit: Some(500000),
        closing_day: closing,
        due_day: due,
        payment_account_id: None,
        network: Some("Visa".into()),
        last4: Some("1234".into()),
    }
}

fn empty_patch(id: &str) -> AccountPatch {
    AccountPatch {
        id: id.into(),
        name: None,
        institution_id: None,
        member_ids: None,
        opening_balance: None,
        opening_date: None,
        include_in_net_worth: None,
        archived: None,
        color: None,
        notes: None,
        sort_order: None,
        card: None,
        overdraft: None,
        branch: None,
        account_number: None,
        retirement_plan: None,
        retirement_coverage: None,
    }
}

const TODAY: &str = "2026-10-14";

#[test]
fn create_with_defaults_and_validation() {
    let mut fx = Fx::new();
    let carlos = fx.member("Carlos");
    let mut i = base_input("Corrente", AccountKind::Checking);
    i.member_ids = Some(vec![carlos.clone(), carlos.clone()]);
    let id = fx.write(|tx| create(tx, &i, d(TODAY))).unwrap();
    let v = get(fx.conn(), &id, d(TODAY)).unwrap();
    assert_eq!(v.account.currency, "BRL");
    assert_eq!(v.account.opening_date, TODAY);
    assert_eq!(v.account.member_ids, vec![carlos]);
    assert!(v.account.card.is_none());
    assert_eq!(v.available_credit, None);

    let i = base_input("Cartão", AccountKind::CreditCard);
    assert!(
        fx.write(|tx| create(tx, &i, d(TODAY))).is_err(),
        "cartão sem dados"
    );
    let mut i = base_input("Poupança", AccountKind::Savings);
    i.card = Some(card_details(3, 10));
    assert!(fx.write(|tx| create(tx, &i, d(TODAY))).is_err());
    let mut i = base_input("Cartão", AccountKind::CreditCard);
    let mut c = card_details(3, 10);
    c.last4 = Some("12a4".into());
    i.card = Some(c);
    assert!(fx.write(|tx| create(tx, &i, d(TODAY))).is_err());
    let mut c = card_details(32, 10);
    c.last4 = None;
    i.card = Some(c);
    assert!(fx.write(|tx| create(tx, &i, d(TODAY))).is_err());
    let mut c = card_details(3, 10);
    c.payment_account_id = Some(id.clone());
    i.card = Some(c);
    let card = fx.write(|tx| create(tx, &i, d(TODAY))).unwrap();
    // Outro cartão não pode ser a conta pagadora.
    let mut i2 = base_input("Cartão 2", AccountKind::CreditCard);
    let mut c2 = card_details(3, 10);
    c2.payment_account_id = Some(card.clone());
    i2.card = Some(c2);
    assert!(fx.write(|tx| create(tx, &i2, d(TODAY))).is_err());
    let mut i3 = base_input("Moeda ruim", AccountKind::Foreign);
    i3.currency = Some("dolar".into());
    assert!(fx.write(|tx| create(tx, &i3, d(TODAY))).is_err());
}

#[test]
fn balances_projected_reconciled_and_last_date() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 100000);
    fx.txn(&acc, "2026-10-01", -20000, "Aluguel");
    let mut i = Fx::input(&acc, "2026-10-10", 5000, "Reembolso");
    i.status = Some(TxnStatus::Reconciled);
    fx.txn_with(i).unwrap();
    let mut i = Fx::input(&acc, "2026-10-20", -10000, "Previsto do mês");
    i.status = Some(TxnStatus::Planned);
    fx.txn_with(i).unwrap();
    let mut i = Fx::input(&acc, "2026-11-05", -3000, "Previsto de novembro");
    i.status = Some(TxnStatus::Planned);
    fx.txn_with(i).unwrap();
    fx.txn(&acc, "2026-10-20", -1000, "Agendado efetivado");

    let v = get(fx.conn(), &acc, d(TODAY)).unwrap();
    assert_eq!(v.balance, 85000);
    assert_eq!(v.balance, balance(fx.conn(), &acc, TODAY, false).unwrap());
    assert_eq!(v.projected_balance, 74000);
    assert_eq!(
        v.projected_balance,
        balance(fx.conn(), &acc, "2026-10-31", true).unwrap()
    );
    assert_eq!(v.reconciled_balance, 105000);
    assert_eq!(v.last_txn_date.as_deref(), Some("2026-10-10"));
    assert_eq!(v.balance_base, Some(85000));
    assert!(!v.fx_estimated);
}

#[test]
fn foreign_balance_in_base_currency() {
    let mut fx = Fx::new();
    let usd = fx.account("Conta EUA", AccountKind::Foreign, "USD", 10000);
    let eur = fx.account("Conta Europa", AccountKind::Foreign, "EUR", 10000);
    crate::fx::set_manual(
        fx.conn(),
        &crate::fx::FxRateInput {
            base: "USD".into(),
            quote: "BRL".into(),
            date: "2026-10-10".into(),
            rate: "5.25".into(),
        },
    )
    .unwrap();
    let all = list(fx.conn(), false, d(TODAY)).unwrap();
    let u = all.iter().find(|a| a.account.id == usd).unwrap();
    assert_eq!(u.balance_base, Some(52500));
    assert!(!u.fx_estimated);
    let later = get(fx.conn(), &usd, d("2026-11-30")).unwrap();
    assert!(later.fx_estimated, "câmbio antigo é estimado");
    let e = all.iter().find(|a| a.account.id == eur).unwrap();
    assert_eq!(e.balance_base, None);
}

#[test]
fn available_credit_counts_unpaid_and_future_installments() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, Some(500000));
    fx.txn(&card, "2026-10-01", -10000, "Mercado");
    let mut planned = Fx::input(&card, "2026-10-20", -5000, "Assinatura prevista");
    planned.status = Some(TxnStatus::Planned);
    fx.txn_with(planned).unwrap();
    let plan = InstallmentInput {
        account_id: card.clone(),
        description: "Geladeira".into(),
        total_amount: 300000,
        installments: 10,
        purchase_date: TODAY.into(),
        category_id: None,
        member_id: None,
        tags: None,
    };
    fx.write(|tx| crate::cards::create_installments(tx, &plan, d(TODAY)))
        .unwrap();
    let v = get(fx.conn(), &card, d(TODAY)).unwrap();
    // 500.000 − 10.000 (compra) − 300.000 (todas as parcelas); a assinatura prevista não conta.
    assert_eq!(v.available_credit, Some(190000));
    assert_eq!(v.balance, -10000 - 30000);
}

#[test]
fn delete_only_without_transactions_and_undo_restores() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let card = fx.card("Cartão", 3, 10, None);
    let t = fx.txn(&card, "2026-10-01", -100, "x");
    assert!(matches!(
        fx.write(|tx| delete(tx, &card)),
        Err(Error::Conflict(_))
    ));
    fx.write(|tx| crate::transactions::delete(tx, &[t], d(TODAY)))
        .unwrap();
    assert_eq!(fx.count("SELECT count(*) FROM card_statement"), 1);
    let (_, cs) = fx.write_cs(|tx| delete(tx, &card)).unwrap();
    assert_eq!(fx.count("SELECT count(*) FROM card_statement"), 0);
    assert_eq!(fx.count("SELECT count(*) FROM credit_card"), 0);
    fx.undo(&cs.unwrap()).unwrap();
    let v = get(fx.conn(), &card, d(TODAY)).unwrap();
    assert_eq!(v.account.card.unwrap().closing_day, 3);
    assert_eq!(fx.count("SELECT count(*) FROM card_statement"), 1);
    fx.write(|tx| delete(tx, &acc)).unwrap();
    assert!(get(fx.conn(), &acc, d(TODAY)).is_err());
}

#[test]
fn update_fields_members_and_archive() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let ana = fx.member("Ana");
    let mut p = empty_patch(&acc);
    p.name = Some("Conta conjunta".into());
    p.member_ids = Some(vec![ana.clone()]);
    p.archived = Some(true);
    p.notes = Some(Some("principal".into()));
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    let v = get(fx.conn(), &acc, d(TODAY)).unwrap();
    assert_eq!(v.account.name, "Conta conjunta");
    assert_eq!(v.account.member_ids, vec![ana]);
    assert!(v.account.archived);
    assert!(list(fx.conn(), false, d(TODAY)).unwrap().is_empty());
    assert_eq!(list(fx.conn(), true, d(TODAY)).unwrap().len(), 1);
    let mut p = empty_patch(&acc);
    p.card = Some(card_details(3, 10));
    assert!(fx.write(|tx| update(tx, &p, d(TODAY))).is_err());
    // Patch vazio não grava nada.
    let (_, cs) = fx
        .write_cs(|tx| update(tx, &empty_patch(&acc), d(TODAY)))
        .unwrap();
    assert!(cs.is_none());
}

#[test]
fn balance_history_by_day_and_month() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 1000);
    fx.txn(&acc, "2026-08-20", 100, "agosto");
    fx.txn(&acc, "2026-10-02", 500, "a");
    fx.txn(&acc, "2026-10-05", -200, "b");
    let mut i = Fx::input(&acc, "2026-10-06", -50, "previsto");
    i.status = Some(TxnStatus::Planned);
    fx.txn_with(i).unwrap();
    let args = |from: &str, to: &str, interval, planned| BalanceHistoryArgs {
        id: acc.clone(),
        from: from.into(),
        to: to.into(),
        interval: Some(interval),
        include_planned: Some(planned),
    };
    let pts = balance_history(
        fx.conn(),
        &args("2026-10-01", "2026-10-06", BalanceInterval::Day, false),
    )
    .unwrap();
    let vals: Vec<i64> = pts.iter().map(|p| p.balance).collect();
    assert_eq!(vals, vec![1100, 1600, 1600, 1600, 1400, 1400]);
    assert_eq!(pts[0].date, "2026-10-01");
    let pts = balance_history(
        fx.conn(),
        &args("2026-10-01", "2026-10-06", BalanceInterval::Day, true),
    )
    .unwrap();
    assert_eq!(pts.last().unwrap().balance, 1350);
    let pts = balance_history(
        fx.conn(),
        &args("2026-07-15", "2026-10-10", BalanceInterval::Month, false),
    )
    .unwrap();
    let got: Vec<(&str, i64)> = pts.iter().map(|p| (p.date.as_str(), p.balance)).collect();
    assert_eq!(
        got,
        vec![
            ("2026-07-31", 1000),
            ("2026-08-31", 1100),
            ("2026-09-30", 1100),
            ("2026-10-10", 1400)
        ]
    );
    assert!(
        balance_history(
            fx.conn(),
            &args("2026-10-06", "2026-10-01", BalanceInterval::Day, false)
        )
        .is_err()
    );
    assert!(
        balance_history(
            fx.conn(),
            &args("2000-01-01", "2026-10-01", BalanceInterval::Day, false)
        )
        .is_err()
    );
}

#[test]
fn reconciliation_marks_cleared_and_locks_values() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 10000);
    let a = fx.txn(&acc, "2026-10-01", -2000, "a");
    let b = fx.txn(&acc, "2026-10-12", -1000, "depois do extrato");
    let mut i = Fx::input(&acc, "2026-10-05", -300, "previsto");
    i.status = Some(TxnStatus::Planned);
    let p = fx.txn_with(i).unwrap();
    let args = ReconcileArgs {
        account_id: acc.clone(),
        statement_date: "2026-10-10".into(),
        statement_balance: 8500,
        apply: Some(true),
    };
    let r = reconcile_check(fx.conn(), &args).unwrap();
    assert_eq!(r.computed_balance, 8000);
    assert_eq!(r.difference, 500);
    let ok = ReconcileArgs {
        statement_balance: 8000,
        ..args
    };
    assert_eq!(reconcile_check(fx.conn(), &ok).unwrap().difference, 0);
    let n = fx
        .write(|tx| mark_reconciled(tx, &acc, &ok.statement_date))
        .unwrap();
    assert_eq!(n, 1);
    let status = |fx: &Fx, id: &str| crate::transactions::get(fx.conn(), id).unwrap().status;
    assert_eq!(status(&fx, &a), TxnStatus::Reconciled);
    assert_eq!(status(&fx, &b), TxnStatus::Cleared);
    assert_eq!(status(&fx, &p), TxnStatus::Planned);
    assert_eq!(
        get(fx.conn(), &acc, d(TODAY)).unwrap().reconciled_balance,
        8000
    );
    // Conciliado: valor travado, descrição livre.
    let mut patch = TxnPatch {
        id: a.clone(),
        account_id: None,
        date: None,
        amount: Some(-2500),
        kind: None,
        status: None,
        description: None,
        payee: None,
        notes: None,
        category_id: None,
        member_id: None,
        splits: None,
        tags: None,
        transfer_account_id: None,
        transfer_amount: None,
        project_id: None,
        card_plastic_id: None,
        orig_amount: None,
        orig_currency: None,
    };
    assert!(matches!(
        fx.write(|tx| crate::transactions::update(tx, &patch, d(TODAY))),
        Err(Error::Conflict(_))
    ));
    patch.amount = None;
    patch.description = Some("Aluguel de outubro".into());
    fx.write(|tx| crate::transactions::update(tx, &patch, d(TODAY)))
        .unwrap();
    // Desconciliar junto com a mudança de valor é aceito.
    patch.amount = Some(-2500);
    patch.status = Some(TxnStatus::Cleared);
    fx.write(|tx| crate::transactions::update(tx, &patch, d(TODAY)))
        .unwrap();
    assert_eq!(status(&fx, &a), TxnStatus::Cleared);
}

#[test]
fn changing_card_cycle_recomputes_open_statements() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 10, 20, None);
    let old = fx.txn(&card, "2026-10-05", -100, "fatura de outubro");
    let mid = fx.txn(&card, "2026-10-12", -200, "fatura de novembro");
    let nov = fx.txn(&card, "2026-11-12", -300, "fatura de dezembro");
    let month = |fx: &Fx, id: &str| {
        crate::transactions::get(fx.conn(), id)
            .unwrap()
            .statement_due_date
            .unwrap()
    };
    assert_eq!(month(&fx, &old), "2026-10-20");
    assert_eq!(month(&fx, &mid), "2026-11-20");
    assert_eq!(month(&fx, &nov), "2026-12-20");
    let mut p = empty_patch(&card);
    let mut c = card_details(15, 25);
    c.credit_limit = None;
    p.card = Some(c);
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    // A fatura de outubro já fechou e não muda; a de novembro ganha as novas datas e
    // recebe a compra de 12/11; a de dezembro ficou vazia e some.
    assert_eq!(month(&fx, &old), "2026-10-20");
    assert_eq!(month(&fx, &mid), "2026-11-25");
    assert_eq!(month(&fx, &nov), "2026-11-25");
    assert_eq!(
        fx.count("SELECT count(*) FROM card_statement WHERE reference_month = '2026-12'"),
        0
    );
    let closing: String = fx
        .conn()
        .query_row(
            "SELECT closing_date FROM card_statement WHERE reference_month = '2026-11'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(closing, "2026-11-15");
}

#[test]
fn institution_by_name_reuses_or_creates_in_the_same_change() {
    let mut fx = Fx::new();
    let mut i = base_input("Nubank", AccountKind::CreditCard);
    i.card = Some(card_details(3, 10));
    i.institution_name = Some("  Nubank ".into());
    let card = fx.write(|tx| create(tx, &i, d(TODAY))).unwrap();
    let v = get(fx.conn(), &card, d(TODAY)).unwrap();
    assert_eq!(v.account.institution_name.as_deref(), Some("Nubank"));
    let kind: String = fx
        .conn()
        .query_row("SELECT kind FROM institution", [], |r| r.get(0))
        .unwrap();
    assert_eq!(kind, "issuer", "cartão cria emissor");
    // Mesmo nome (outra caixa): reaproveita a instituição.
    let mut i = base_input("Conta Nu", AccountKind::Checking);
    i.institution_name = Some("NUBANK".into());
    let acc = fx.write(|tx| create(tx, &i, d(TODAY))).unwrap();
    let v2 = get(fx.conn(), &acc, d(TODAY)).unwrap();
    assert_eq!(v2.account.institution_id, v.account.institution_id);
    let n: i64 = fx
        .conn()
        .query_row("SELECT count(*) FROM institution", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    // `institution_id` tem precedência sobre o nome.
    let mut i = base_input("XP", AccountKind::Brokerage);
    i.institution_id = v.account.institution_id.clone();
    i.institution_name = Some("XP Investimentos".into());
    let xp = fx.write(|tx| create(tx, &i, d(TODAY))).unwrap();
    assert_eq!(
        get(fx.conn(), &xp, d(TODAY))
            .unwrap()
            .account
            .institution_id,
        v.account.institution_id
    );
}

#[test]
fn ofx_finds_the_account_by_bank_branch_and_number() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let api = crate::api::Api::new(
        crate::paths::TreasuryPaths::with_root(dir.path().join(".t")).unwrap(),
    )
    .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "C"}),
    )
    .unwrap();
    let itau = api
        .call(
            "household.create_institution",
            json!({"name": "Itaú", "kind": "bank", "code": "341"}),
        )
        .unwrap()["id"]
        .clone();
    let nu = api
        .call(
            "household.create_institution",
            json!({"name": "Nubank", "kind": "bank", "code": "260"}),
        )
        .unwrap()["id"]
        .clone();
    let a = api
        .call(
            "accounts.create",
            json!({"name": "Itaú CC", "kind": "checking", "institution_id": itau,
            "branch": "0123", "account_number": "45678-9"}),
        )
        .unwrap();
    assert_eq!(a["account_number"], "45678-9");
    api.call(
        "accounts.create",
        json!({"name": "Nu CC", "kind": "checking", "institution_id": nu,
        "account_number": "45678-9"}),
    )
    .unwrap();
    // BANKID com zeros, conta sem o dígito, agência certa: Itaú (o Nubank tem o mesmo número,
    // mas é outro banco).
    let m = api
        .call(
            "accounts.match_ofx",
            json!({"bank_id": "0341", "branch_id": "123", "account_id": "45678"}),
        )
        .unwrap();
    assert_eq!(m["account_id"], a["id"], "{m}");
    assert_eq!(m["bank"]["code"], "341");
    // Sem o banco, os dois combinam: escolha manual.
    let m = api
        .call("accounts.match_ofx", json!({"account_id": "456789"}))
        .unwrap();
    assert!(m["account_id"].is_null());
    assert!(m["reason"].as_str().unwrap().contains("mais de uma"));
    // Lista do Banco Central: busca por nome e código.
    let banks = api
        .call("household.banks", json!({"query": "itau"}))
        .unwrap();
    assert!(
        banks.as_array().unwrap().iter().any(|b| b["code"] == "341"),
        "{banks}"
    );
    let b = api
        .call("household.banks", json!({"query": "260"}))
        .unwrap();
    assert_eq!(b[0]["code"], "260");
}

/// Erros, a nota do ajuste e as dicas da sincronia de saldo saem no idioma da instalação.
#[test]
fn messages_follow_the_installation_language() {
    use crate::i18n::{Locale, set_current};
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let card = fx.card("Cartão", 3, 10, None);
    fx.txn(&card, "2026-10-01", -100, "x");
    fx.txn(&acc, "2026-10-01", -2500, "Padaria");
    set_current(Locale::En);
    let e = fx.write(|tx| delete(tx, &card)).unwrap_err();
    assert_eq!(
        e.message(),
        "the account Cartão has 1 transaction; delete it first or archive the account"
    );
    let id = fx
        .write(|tx| {
            sync::sync_balance(
                tx,
                &sync::SyncBalanceArgs {
                    account_id: acc.clone(),
                    date: "2026-10-02".into(),
                    balance: -5000,
                    note: None,
                    reconcile: Some(false),
                },
                d(TODAY),
            )
        })
        .unwrap();
    let s = sync::get(fx.conn(), &id).unwrap();
    let adj = crate::transactions::get(fx.conn(), s.adjustment_txn_id.as_deref().unwrap()).unwrap();
    assert_eq!(adj.description, "Balance sync");
    assert_eq!(
        adj.notes.as_deref(),
        Some("Bank balance on Oct 2, 2026: \u{2212}R$50.00; Treasury had \u{2212}R$25.00.")
    );
    let h = sync::hints(fx.conn(), &id).unwrap();
    assert_eq!(
        h.hints[0].message,
        "1 transaction with the exact amount of the difference (R$25.00): it may have been entered extra (or missed) in Treasury."
    );
    set_current(Locale::PtBr);
}

#[test]
fn balance_status_follows_the_days_since_the_last_news() {
    use freshness::BalanceStatus::*;
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 100000);
    fx.txn(&acc, "2026-10-11", -2000, "Padaria");
    let at = |fx: &Fx, day: &str| {
        let v = get(fx.conn(), &acc, d(day)).unwrap();
        (v.staleness_days, v.balance_status)
    };
    assert_eq!(at(&fx, "2026-10-14"), (Some(3), Some(Fresh)));
    assert_eq!(at(&fx, "2026-10-23"), (Some(12), Some(Aging)));
    assert_eq!(at(&fx, "2026-11-05"), (Some(25), Some(PossiblyIncomplete)));
    // Uma conferência de saldo conta como novidade.
    fx.conn()
        .execute(
            "INSERT INTO balance_sync (id, account_id, date, statement_balance, computed_balance,
                                       difference, created_at, updated_at)
             VALUES ('s1', ?1, '2026-11-01', 98000, 98000, 0, 'x', 'x')",
            [&acc],
        )
        .unwrap();
    let v = get(fx.conn(), &acc, d("2026-11-05")).unwrap();
    assert_eq!(v.last_import_at.as_deref(), Some("2026-11-01"));
    assert_eq!((v.staleness_days, v.balance_status), (Some(4), Some(Fresh)));
    // Conta arquivada fica sem status.
    let mut p = empty_patch(&acc);
    p.archived = Some(true);
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    assert_eq!(get(fx.conn(), &acc, d(TODAY)).unwrap().balance_status, None);
}

#[test]
fn a_monthly_account_without_last_month_is_possibly_incomplete() {
    use freshness::BalanceStatus::*;
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 100000);
    for day in ["2026-06-10", "2026-07-10", "2026-08-10"] {
        fx.txn(&acc, day, -1000, "Conta de luz");
    }
    // Setembro em branco; um lançamento em 02/10 deixa a conta com poucos dias.
    fx.txn(&acc, "2026-10-02", -1000, "Mercado");
    let v = get(fx.conn(), &acc, d("2026-10-05")).unwrap();
    assert_eq!(v.staleness_days, Some(3));
    assert_eq!(v.balance_status, Some(PossiblyIncomplete));
    assert_eq!(v.missed_month.as_deref(), Some("2026-09"));
    // Com setembro lançado, volta a ficar em dia.
    fx.txn(&acc, "2026-09-15", -1000, "Farmácia");
    let v = get(fx.conn(), &acc, d("2026-10-05")).unwrap();
    assert_eq!((v.balance_status, v.missed_month), (Some(Fresh), None));
}

#[test]
fn staleness_limits_by_account_kind_are_respected() {
    use freshness::BalanceStatus::*;
    let mut fx = Fx::new();
    let vr = fx.account("Vale-refeição", AccountKind::Benefit, "BRL", 50000);
    fx.txn(&vr, "2026-09-01", -3000, "Almoço");
    let status = |fx: &Fx| get(fx.conn(), &vr, d("2026-10-11")).unwrap().balance_status;
    // 40 dias: no padrão do benefício (20 e 45 dias), envelhecendo.
    assert_eq!(status(&fx), Some(Aging));
    fx.write(|tx| {
        freshness::set(
            tx,
            &freshness::SetStalenessArgs {
                kind: AccountKind::Benefit,
                limits: Some(freshness::StalenessLimits {
                    aging_days: 30,
                    incomplete_days: 35,
                }),
            },
        )
    })
    .unwrap();
    assert_eq!(status(&fx), Some(PossiblyIncomplete));
    let s = freshness::settings(fx.conn()).unwrap();
    let benefit = s.iter().find(|x| x.kind == AccountKind::Benefit).unwrap();
    assert!(benefit.custom);
    assert_eq!(benefit.limits.incomplete_days, 35);
    // Limites inválidos são recusados; sem limites, volta ao padrão.
    let bad = freshness::SetStalenessArgs {
        kind: AccountKind::Benefit,
        limits: Some(freshness::StalenessLimits {
            aging_days: 30,
            incomplete_days: 10,
        }),
    };
    assert!(fx.write(|tx| freshness::set(tx, &bad)).is_err());
    fx.write(|tx| {
        freshness::set(
            tx,
            &freshness::SetStalenessArgs {
                kind: AccountKind::Benefit,
                limits: None,
            },
        )
    })
    .unwrap();
    assert_eq!(status(&fx), Some(Aging));
}
