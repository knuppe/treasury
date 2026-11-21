use super::testkit::{Fx, d};
use super::*;
use crate::budget::Basis;

const TODAY: &str = "2026-10-14";

fn patch(id: &str) -> TxnPatch {
    TxnPatch {
        id: id.into(),
        account_id: None,
        date: None,
        amount: None,
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
    }
}

fn split(cat: Option<&str>, amount: i64) -> SplitInput {
    SplitInput {
        category_id: cat.map(str::to_string),
        member_id: None,
        amount,
        notes: None,
    }
}

fn transfer(from: &str, to: &str, amount: i64, to_amount: Option<i64>) -> TransferInput {
    TransferInput {
        from_account_id: from.into(),
        to_account_id: to.into(),
        date: "2026-10-05".into(),
        amount,
        to_amount,
        description: None,
        kind: None,
        status: None,
        member_id: None,
    }
}

fn q(fx: &Fx, f: TxnFilter) -> TxnPage {
    query(fx.conn(), &f).unwrap()
}

fn descs(p: &TxnPage) -> Vec<String> {
    p.items.iter().map(|t| t.description.clone()).collect()
}

#[test]
fn create_infers_kind_and_validates() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let out = fx.txn(&acc, "2026-10-01", -1000, "Padaria");
    let inc = fx.txn(&acc, "2026-10-01", 5000, "Salário");
    assert_eq!(fx.kind_of(&out), TxnKind::Expense);
    assert_eq!(fx.kind_of(&inc), TxnKind::Income);
    let v = get(fx.conn(), &out).unwrap();
    assert_eq!((v.currency.as_str(), v.status), ("BRL", TxnStatus::Cleared));
    assert_eq!(v.account_name, "Corrente");

    assert!(
        fx.txn_with(Fx::input(&acc, "2026-10-01", 0, "zero"))
            .is_err()
    );
    assert!(
        fx.txn_with(Fx::input(&acc, "2026-13-01", -1, "data"))
            .is_err()
    );
    assert!(
        fx.txn_with(Fx::input("nada", "2026-10-01", -1, "conta"))
            .is_err()
    );
    // Transferência de uma perna só (PIX para terceiros) pode; fatura precisa das duas.
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "transfer");
    i.kind = Some(TxnKind::Transfer);
    assert!(fx.txn_with(i).is_ok());
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "fatura");
    i.kind = Some(TxnKind::CardPayment);
    assert!(
        fx.txn_with(i).is_err(),
        "pagamento de fatura precisa de duas pernas"
    );
    // Categoria do tipo errado, arquivada ou em tipo que não usa categoria.
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "x");
    i.category_id = Some(fx.cat("Salário"));
    assert!(fx.txn_with(i).is_err());
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "x");
    i.kind = Some(TxnKind::Adjustment);
    i.category_id = Some(fx.cat("Mercado"));
    assert!(fx.txn_with(i).is_err());
    fx.conn()
        .execute(
            "UPDATE category SET archived = 1 WHERE name = 'Padaria'",
            [],
        )
        .unwrap();
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "x");
    i.category_id = Some(fx.cat("Padaria"));
    assert!(fx.txn_with(i).is_err());
    // Estorno: despesa positiva.
    let mut i = Fx::input(&acc, "2026-10-01", 300, "Estorno");
    i.kind = Some(TxnKind::Expense);
    i.category_id = Some(fx.cat("Mercado"));
    let id = fx.txn_with(i).unwrap();
    assert_eq!(fx.kind_of(&id), TxnKind::Expense);
}

#[test]
fn splits_must_close_and_follow_amount_changes() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let mercado = fx.cat("Mercado");
    let racao = fx.cat("Ração");
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "Mercado");
    i.splits = Some(vec![split(Some(&mercado), -600), split(Some(&racao), -300)]);
    assert!(fx.txn_with(i.clone()).is_err(), "divisão que não fecha");
    i.splits = Some(vec![
        split(Some(&mercado), -600),
        split(Some(&fx.cat("Salário")), -400),
    ]);
    assert!(
        fx.txn_with(i.clone()).is_err(),
        "categoria de receita na divisão"
    );
    i.splits = Some(vec![split(Some(&mercado), -600), split(Some(&racao), -400)]);
    i.tags = Some(vec!["casa".into(), "Casa".into(), " pets ".into()]);
    let id = fx.txn_with(i).unwrap();
    let v = get(fx.conn(), &id).unwrap();
    assert_eq!(v.splits.len(), 2);
    assert_eq!(v.splits[0].amount, -600);
    assert_eq!(
        v.tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        vec!["casa", "pets"]
    );
    // Mudar o valor sem mandar a divisão nova é recusado.
    let mut p = patch(&id);
    p.amount = Some(-1200);
    assert!(fx.write(|tx| update(tx, &p, d(TODAY))).is_err());
    p.splits = Some(vec![split(Some(&mercado), -800), split(Some(&racao), -400)]);
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    assert_eq!(get(fx.conn(), &id).unwrap().splits[0].amount, -800);
    // Lista vazia remove a divisão.
    let mut p = patch(&id);
    p.splits = Some(vec![]);
    p.tags = Some(vec![]);
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    let v = get(fx.conn(), &id).unwrap();
    assert!(v.splits.is_empty() && v.tags.is_empty());
}

#[test]
fn transfer_between_currencies_and_validation() {
    let mut fx = Fx::new();
    let a = fx.checking("Corrente", 100000);
    let b = fx.checking("Poupança", 0);
    let usd = fx.account("EUA", crate::accounts::AccountKind::Foreign, "USD", 0);
    let ids = fx
        .write(|tx| create_transfer(tx, &transfer(&a, &b, 10000, None), d(TODAY)))
        .unwrap();
    let from = get(fx.conn(), &ids.from_id).unwrap();
    let to = get(fx.conn(), &ids.to_id).unwrap();
    assert_eq!((from.amount, to.amount), (-10000, 10000));
    assert_eq!(from.transfer_id.as_deref(), Some(ids.transfer_id.as_str()));
    assert_eq!(from.counterpart_account_name.as_deref(), Some("Poupança"));
    assert_eq!(to.counterpart_account_id.as_deref(), Some(a.as_str()));
    assert_eq!(from.description, "Transferência para Poupança");
    assert_eq!(to.description, "Transferência de Corrente");
    assert_eq!(from.kind, TxnKind::Transfer);

    assert!(
        fx.write(|tx| create_transfer(tx, &transfer(&a, &usd, 54000, None), d(TODAY)))
            .is_err(),
        "moedas diferentes exigem to_amount"
    );
    let ids = fx
        .write(|tx| create_transfer(tx, &transfer(&a, &usd, 54000, Some(10000)), d(TODAY)))
        .unwrap();
    let to = get(fx.conn(), &ids.to_id).unwrap();
    assert_eq!((to.amount, to.currency.as_str()), (10000, "USD"));
    assert!(
        fx.write(|tx| create_transfer(tx, &transfer(&a, &b, 100, Some(90)), d(TODAY)))
            .is_err()
    );
    assert!(
        fx.write(|tx| create_transfer(tx, &transfer(&a, &a, 100, None), d(TODAY)))
            .is_err()
    );
    assert!(
        fx.write(|tx| create_transfer(tx, &transfer(&a, &b, -100, None), d(TODAY)))
            .is_err()
    );
    let mut bad = transfer(&a, &b, 100, None);
    bad.kind = Some(TxnKind::CardPayment);
    assert!(fx.write(|tx| create_transfer(tx, &bad, d(TODAY))).is_err());
    let mut bad = transfer(&a, &b, 100, None);
    bad.kind = Some(TxnKind::Income);
    assert!(fx.write(|tx| create_transfer(tx, &bad, d(TODAY))).is_err());
    // Saldos: transferência não cria nem destrói dinheiro.
    assert_eq!(
        crate::accounts::balance(fx.conn(), &a, TODAY, false).unwrap(),
        100000 - 10000 - 54000
    );
    assert_eq!(
        crate::accounts::balance(fx.conn(), &b, TODAY, false).unwrap(),
        10000
    );
}

#[test]
fn editing_and_deleting_transfer_legs_keeps_them_consistent() {
    let mut fx = Fx::new();
    let a = fx.checking("Corrente", 0);
    let b = fx.checking("Poupança", 0);
    let c = fx.checking("Outra", 0);
    let usd = fx.account("EUA", crate::accounts::AccountKind::Foreign, "USD", 0);
    let ids = fx
        .write(|tx| create_transfer(tx, &transfer(&a, &b, 10000, None), d(TODAY)))
        .unwrap();
    let mut p = patch(&ids.to_id);
    p.amount = Some(12000);
    p.date = Some("2026-10-07".into());
    p.description = Some("Reserva".into());
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    let from = get(fx.conn(), &ids.from_id).unwrap();
    assert_eq!(
        (from.amount, from.date.as_str(), from.description.as_str()),
        (-12000, "2026-10-07", "Reserva")
    );
    // Sinal da perna, tipo e mesma conta nas duas pernas são recusados.
    let mut p = patch(&ids.to_id);
    p.amount = Some(-5);
    assert!(fx.write(|tx| update(tx, &p, d(TODAY))).is_err());
    let mut p = patch(&ids.to_id);
    p.kind = Some(TxnKind::Income);
    assert!(fx.write(|tx| update(tx, &p, d(TODAY))).is_err());
    let mut p = patch(&ids.to_id);
    p.account_id = Some(a.clone());
    assert!(fx.write(|tx| update(tx, &p, d(TODAY))).is_err());
    let mut p = patch(&ids.to_id);
    p.account_id = Some(usd.clone());
    assert!(
        fx.write(|tx| update(tx, &p, d(TODAY))).is_err(),
        "outra moeda"
    );
    let mut p = patch(&ids.to_id);
    p.account_id = Some(c.clone());
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    assert_eq!(
        get(fx.conn(), &ids.from_id)
            .unwrap()
            .counterpart_account_id
            .as_deref(),
        Some(c.as_str())
    );
    // Status previsto acompanha nas duas pernas.
    fx.write(|tx| {
        set_status(
            tx,
            std::slice::from_ref(&ids.from_id),
            TxnStatus::Planned,
            d(TODAY),
        )
    })
    .unwrap();
    assert_eq!(
        get(fx.conn(), &ids.to_id).unwrap().status,
        TxnStatus::Planned
    );
    fx.write(|tx| {
        set_status(
            tx,
            std::slice::from_ref(&ids.to_id),
            TxnStatus::Cleared,
            d(TODAY),
        )
    })
    .unwrap();
    assert_eq!(
        get(fx.conn(), &ids.from_id).unwrap().status,
        TxnStatus::Cleared
    );
    // Entre moedas, mudar o valor de uma perna não mexe na outra.
    let x = fx
        .write(|tx| create_transfer(tx, &transfer(&a, &usd, 5400, Some(1000)), d(TODAY)))
        .unwrap();
    let mut p = patch(&x.from_id);
    p.amount = Some(-5500);
    fx.write(|tx| update(tx, &p, d(TODAY))).unwrap();
    assert_eq!(get(fx.conn(), &x.to_id).unwrap().amount, 1000);

    // Apagar uma perna apaga as duas; desfazer restaura as duas.
    let (n, cs) = fx
        .write_cs(|tx| delete(tx, std::slice::from_ref(&ids.from_id), d(TODAY)))
        .unwrap();
    assert_eq!(n, 2);
    assert!(get(fx.conn(), &ids.to_id).is_err());
    fx.undo(&cs.unwrap()).unwrap();
    let from = get(fx.conn(), &ids.from_id).unwrap();
    let to = get(fx.conn(), &ids.to_id).unwrap();
    assert_eq!((from.amount, to.amount), (-12000, 12000));
    assert_eq!(from.transfer_id, to.transfer_id);
}

#[test]
fn undo_delete_restores_splits_tags_and_attachments() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "Mercado");
    i.splits = Some(vec![
        split(Some(&fx.cat("Mercado")), -700),
        split(None, -300),
    ]);
    i.tags = Some(vec!["viagem-2026".into()]);
    let id = fx.txn_with(i).unwrap();
    fx.write(|tx| {
        add_attachment(
            tx,
            &AttachmentInput {
                txn_id: id.clone(),
                filename: "nota.txt".into(),
                mime: "text/plain".into(),
                content_base64: b64::encode(b"recibo"),
            },
        )
    })
    .unwrap();
    let (_, cs) = fx
        .write_cs(|tx| delete(tx, std::slice::from_ref(&id), d(TODAY)))
        .unwrap();
    assert_eq!(fx.count("SELECT count(*) FROM attachment"), 0);
    assert_eq!(fx.count("SELECT count(*) FROM txn_split"), 0);
    fx.undo(&cs.unwrap()).unwrap();
    let v = get(fx.conn(), &id).unwrap();
    assert_eq!(v.splits.len(), 2);
    assert_eq!(v.tags[0].name, "viagem-2026");
    assert_eq!(v.attachment_count, 1);
}

#[test]
fn reconciled_cannot_be_deleted() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "x");
    i.status = Some(TxnStatus::Reconciled);
    let id = fx.txn_with(i).unwrap();
    assert!(matches!(
        fx.write(|tx| delete(tx, std::slice::from_ref(&id), d(TODAY))),
        Err(Error::Conflict(_))
    ));
    fx.write(|tx| set_status(tx, std::slice::from_ref(&id), TxnStatus::Cleared, d(TODAY)))
        .unwrap();
    fx.write(|tx| delete(tx, std::slice::from_ref(&id), d(TODAY)))
        .unwrap();
}

#[test]
fn set_category_in_bulk() {
    let mut fx = Fx::new();
    let a = fx.checking("Corrente", 0);
    let b = fx.checking("Poupança", 0);
    let mercado = fx.cat("Mercado");
    let t1 = fx.txn(&a, "2026-10-01", -1000, "um");
    let mut i = Fx::input(&a, "2026-10-01", -1000, "dividido");
    i.splits = Some(vec![
        split(Some(&fx.cat("Padaria")), -500),
        split(None, -500),
    ]);
    let t2 = fx.txn_with(i).unwrap();
    // Aporte não usa categoria (transferência usa: PIX que paga um serviço). Só aportes: erro;
    // misturados, o aporte fica de fora e volta na contagem.
    let mut inv = transfer(&a, &b, 100, None);
    inv.kind = Some(TxnKind::Investment);
    let tr = fx.write(|tx| create_transfer(tx, &inv, d(TODAY))).unwrap();
    assert!(
        fx.write(|tx| set_category(tx, std::slice::from_ref(&tr.from_id), Some(&mercado)))
            .is_err()
    );
    let n = fx
        .write(|tx| {
            set_category(
                tx,
                &[t1.clone(), t2.clone(), t1.clone(), tr.from_id.clone()],
                Some(&mercado),
            )
        })
        .unwrap();
    assert_eq!(n, (2, 1));
    assert_eq!(get(fx.conn(), &tr.from_id).unwrap().category_id, None);
    let v = get(fx.conn(), &t2).unwrap();
    assert_eq!(v.category_path.as_deref(), Some("Alimentação › Mercado"));
    assert!(
        v.splits.is_empty(),
        "a categoria passa a valer para o total"
    );
    let n = fx
        .write(|tx| set_category(tx, std::slice::from_ref(&t1), None))
        .unwrap();
    assert_eq!(n, (1, 0));
    assert_eq!(get(fx.conn(), &t1).unwrap().category_id, None);
}

#[test]
fn query_text_ignores_case_and_accents() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    fx.txn(&acc, "2026-10-01", -1000, "PAG*Padaria São João");
    let mut i = Fx::input(&acc, "2026-10-02", -2000, "Compra");
    i.payee = Some("Açougue Boi Gordo".into());
    fx.txn_with(i).unwrap();
    let mut i = Fx::input(&acc, "2026-10-03", -3000, "Outro");
    i.notes = Some("presente da vovó 50% off".into());
    fx.txn_with(i).unwrap();
    let text = |s: &str| TxnFilter {
        text: Some(s.into()),
        ..Default::default()
    };
    assert_eq!(
        descs(&q(&fx, text("sao joao"))),
        vec!["PAG*Padaria São João"]
    );
    assert_eq!(
        descs(&q(&fx, text("SÃO JOÃO"))),
        vec!["PAG*Padaria São João"]
    );
    assert_eq!(
        descs(&q(&fx, text("padaria joao"))),
        vec!["PAG*Padaria São João"]
    );
    assert_eq!(
        descs(&q(&fx, text("acougue"))),
        vec!["Compra"],
        "busca no favorecido"
    );
    assert_eq!(
        descs(&q(&fx, text("VOVO"))),
        vec!["Outro"],
        "busca nas notas"
    );
    assert_eq!(q(&fx, text("50%")).total, 1, "% é literal");
    assert_eq!(q(&fx, text("5_%")).total, 0, "_ é literal");
    assert_eq!(q(&fx, text("padaria boi")).total, 0, "todas as palavras");
    assert_eq!(q(&fx, text("   ")).total, 3);
}

#[test]
fn query_filters_sorting_paging_and_totals() {
    let mut fx = Fx::new();
    let a = fx.checking("Corrente", 0);
    let usd = fx.account("EUA", crate::accounts::AccountKind::Foreign, "USD", 0);
    let ana = fx.member("Ana");
    let mercado = fx.cat("Mercado");
    let alim = fx.cat("Alimentação");
    let mut i = Fx::input(&a, "2026-10-01", -1000, "Mercado A");
    i.category_id = Some(mercado.clone());
    i.member_id = Some(ana.clone());
    i.tags = Some(vec!["casa".into()]);
    fx.txn_with(i).unwrap();
    let mut i = Fx::input(&a, "2026-10-02", -5000, "Dividido");
    i.splits = Some(vec![
        split(Some(&fx.cat("Restaurantes")), -2000),
        split(Some(&fx.cat("Cinema e shows")), -3000),
    ]);
    fx.txn_with(i).unwrap();
    let mut i = Fx::input(&a, "2026-10-03", 20000, "Salário");
    i.category_id = Some(fx.cat("Salário"));
    fx.txn_with(i).unwrap();
    fx.txn(&a, "2026-10-04", -700, "Sem categoria");
    let mut i = Fx::input(&a, "2026-10-05", -300, "Previsto");
    i.status = Some(TxnStatus::Planned);
    fx.txn_with(i).unwrap();
    fx.txn(&usd, "2026-10-06", -900, "Em dólar");

    let all = q(&fx, TxnFilter::default());
    assert_eq!(all.total, 6);
    assert_eq!(
        all.items[0].description, "Em dólar",
        "data decrescente por padrão"
    );
    let brl = all.totals.iter().find(|t| t.currency == "BRL").unwrap();
    assert_eq!((brl.inflow, brl.outflow, brl.net), (20000, 7000, 13000));
    let us = all.totals.iter().find(|t| t.currency == "USD").unwrap();
    assert_eq!((us.inflow, us.outflow, us.net), (0, 900, -900));

    // Página com totais do filtro inteiro.
    let page = q(
        &fx,
        TxnFilter {
            limit: Some(2),
            offset: Some(2),
            sort: Some(TxnSort::DateAsc),
            ..Default::default()
        },
    );
    assert_eq!(descs(&page), vec!["Salário", "Sem categoria"]);
    assert_eq!(page.total, 6);
    let by_amount = q(
        &fx,
        TxnFilter {
            sort: Some(TxnSort::AmountAsc),
            limit: Some(1),
            ..Default::default()
        },
    );
    assert_eq!(descs(&by_amount), vec!["Dividido"]);
    let by_amount = q(
        &fx,
        TxnFilter {
            sort: Some(TxnSort::AmountDesc),
            limit: Some(1),
            ..Default::default()
        },
    );
    assert_eq!(descs(&by_amount), vec!["Salário"]);

    // Categoria-pai inclui subcategorias (e partes de divisão) por padrão.
    let f = TxnFilter {
        category_ids: Some(vec![alim.clone()]),
        sort: Some(TxnSort::DateAsc),
        ..Default::default()
    };
    assert_eq!(descs(&q(&fx, f.clone())), vec!["Mercado A", "Dividido"]);
    let f = TxnFilter {
        include_subcategories: Some(false),
        ..f
    };
    assert_eq!(q(&fx, f).total, 0);
    let f = TxnFilter {
        category_ids: Some(vec![fx.cat("Lazer")]),
        ..Default::default()
    };
    assert_eq!(descs(&q(&fx, f)), vec!["Dividido"]);

    // Faixa de valor absoluto.
    let f = TxnFilter {
        amount_min: Some(700),
        amount_max: Some(1000),
        sort: Some(TxnSort::DateAsc),
        ..Default::default()
    };
    assert_eq!(
        descs(&q(&fx, f)),
        vec!["Mercado A", "Sem categoria", "Em dólar"]
    );

    let f = TxnFilter {
        member_ids: Some(vec![ana]),
        ..Default::default()
    };
    assert_eq!(q(&fx, f).total, 1);
    let tag = list_tags(fx.conn()).unwrap().remove(0);
    assert_eq!(tag.txn_count, 1);
    let f = TxnFilter {
        tag_ids: Some(vec![tag.id]),
        ..Default::default()
    };
    assert_eq!(descs(&q(&fx, f)), vec!["Mercado A"]);
    let f = TxnFilter {
        uncategorized: Some(true),
        ..Default::default()
    };
    assert_eq!(
        descs(&q(&fx, f)),
        vec!["Em dólar", "Previsto", "Sem categoria"]
    );
    let f = TxnFilter {
        uncategorized: Some(false),
        ..Default::default()
    };
    assert_eq!(q(&fx, f).total, 3);
    let f = TxnFilter {
        statuses: Some(vec![TxnStatus::Planned]),
        ..Default::default()
    };
    assert_eq!(descs(&q(&fx, f)), vec!["Previsto"]);
    let f = TxnFilter {
        kinds: Some(vec![TxnKind::Income]),
        ..Default::default()
    };
    assert_eq!(descs(&q(&fx, f)), vec!["Salário"]);
    let f = TxnFilter {
        account_ids: Some(vec![usd]),
        ..Default::default()
    };
    assert_eq!(q(&fx, f).total, 1);
    let f = TxnFilter {
        date_from: Some("2026-10-02".into()),
        date_to: Some("2026-10-03".into()),
        ..Default::default()
    };
    assert_eq!(q(&fx, f).total, 2);
    let bad = TxnFilter {
        date_from: Some("2026-10-05".into()),
        date_to: Some("2026-10-01".into()),
        ..Default::default()
    };
    assert!(query(fx.conn(), &bad).is_err());
    // query_ids devolve tudo, sem paginação.
    let ids = query_ids(
        fx.conn(),
        &TxnFilter {
            limit: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(ids.len(), 6);
}

#[test]
fn cash_basis_uses_statement_due_date() {
    let mut fx = Fx::new();
    let bank = fx.checking("Corrente", 0);
    let card = fx.card("Cartão", 25, 5, None);
    fx.txn(&card, "2026-09-28", -1000, "Compra depois do fechamento");
    fx.txn(&card, "2026-10-10", -2000, "Compra de outubro");
    fx.txn(&bank, "2026-10-15", -300, "Conta de luz");
    let oct = |basis| TxnFilter {
        date_from: Some("2026-10-01".into()),
        date_to: Some("2026-10-31".into()),
        basis: Some(basis),
        sort: Some(TxnSort::DateAsc),
        ..Default::default()
    };
    assert_eq!(
        descs(&q(&fx, oct(Basis::Accrual))),
        vec!["Compra de outubro", "Conta de luz"]
    );
    // A compra de 28/09 fecha em 25/10 e vence em 05/11; a de 10/10 também vence em 05/11.
    assert_eq!(descs(&q(&fx, oct(Basis::Cash))), vec!["Conta de luz"]);
    let nov = TxnFilter {
        date_from: Some("2026-11-01".into()),
        date_to: Some("2026-11-30".into()),
        basis: Some(Basis::Cash),
        sort: Some(TxnSort::DateAsc),
        ..Default::default()
    };
    let page = q(&fx, nov);
    assert_eq!(
        descs(&page),
        vec!["Compra depois do fechamento", "Compra de outubro"]
    );
    assert_eq!(
        page.items[0].statement_due_date.as_deref(),
        Some("2026-11-05")
    );
}

#[test]
fn tags_crud() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let t = fx
        .write(|tx| {
            create_tag(
                tx,
                &TagInput {
                    name: "Viagem".into(),
                    color: Some("#00ff00".into()),
                },
            )
        })
        .unwrap();
    assert!(matches!(
        fx.write(|tx| create_tag(
            tx,
            &TagInput {
                name: "VIAGEM".into(),
                color: None
            }
        )),
        Err(Error::Conflict(_))
    ));
    let mut i = Fx::input(&acc, "2026-10-01", -100, "x");
    i.tags = Some(vec!["viagem".into()]);
    let id = fx.txn_with(i).unwrap();
    assert_eq!(
        get(fx.conn(), &id).unwrap().tags[0].id,
        t.id,
        "reusa a tag existente"
    );
    let u = fx
        .write(|tx| {
            update_tag(
                tx,
                &TagPatch {
                    id: t.id.clone(),
                    name: Some("Férias".into()),
                    color: Some(None),
                },
            )
        })
        .unwrap();
    assert_eq!((u.name.as_str(), u.color.as_deref()), ("Férias", None));
    fx.write(|tx| delete_tag(tx, &t.id)).unwrap();
    assert!(get(fx.conn(), &id).unwrap().tags.is_empty());
    assert!(list_tags(fx.conn()).unwrap().is_empty());
}

#[test]
fn attachments_roundtrip_and_size_limit() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let id = fx.txn(&acc, "2026-10-01", -100, "x");
    let content: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
    let meta = fx
        .write(|tx| {
            add_attachment(
                tx,
                &AttachmentInput {
                    txn_id: id.clone(),
                    filename: "../comprovante.pdf".into(),
                    mime: "application/pdf".into(),
                    content_base64: b64::encode(&content),
                },
            )
        })
        .unwrap();
    assert_eq!(meta.filename, "comprovante.pdf");
    assert_eq!(meta.size_bytes, 5000);
    assert_eq!(list_attachments(fx.conn(), &id).unwrap().len(), 1);
    let got = get_attachment(fx.conn(), &meta.id).unwrap();
    assert_eq!(b64::decode(&got.content_base64).unwrap(), content);
    assert_eq!(got.meta.sha256, meta.sha256);
    assert_eq!(get(fx.conn(), &id).unwrap().attachment_count, 1);

    let big = "A".repeat((MAX_ATTACHMENT_BYTES / 3 + 10) * 4);
    let err = fx.write(|tx| {
        add_attachment(
            tx,
            &AttachmentInput {
                txn_id: id.clone(),
                filename: "grande.bin".into(),
                mime: String::new(),
                content_base64: big,
            },
        )
    });
    assert!(matches!(err, Err(Error::Invalid(_))));
    let err = fx.write(|tx| {
        add_attachment(
            tx,
            &AttachmentInput {
                txn_id: "nada".into(),
                filename: "a.txt".into(),
                mime: "text/plain".into(),
                content_base64: "YQ==".into(),
            },
        )
    });
    assert!(matches!(err, Err(Error::NotFound(_))));
    fx.write(|tx| delete_attachment(tx, &meta.id)).unwrap();
    assert!(get_attachment(fx.conn(), &meta.id).is_err());
}

#[test]
fn views_batch_load_in_requested_order() {
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let ids: Vec<Id> = fx
        .write(|tx| {
            (0..1200)
                .map(|i| {
                    let mut t = NewTxn::new(
                        &acc,
                        "2026-10-01",
                        -(i + 1),
                        "BRL",
                        TxnKind::Expense,
                        &format!("t{i}"),
                    );
                    t.status = TxnStatus::Cleared;
                    let id = insert(tx, &t)?;
                    replace_tags(tx, &id, &[format!("tag{}", i % 3)])?;
                    Ok(id)
                })
                .collect::<Result<Vec<_>>>()
        })
        .unwrap();
    let mut wanted: Vec<Id> = ids.iter().rev().cloned().collect();
    wanted.push("inexistente".into());
    let got = views(fx.conn(), &wanted).unwrap();
    assert_eq!(got.len(), 1200);
    assert_eq!(got[0].description, "t1199");
    assert_eq!(got[1199].description, "t0");
    assert!(got.iter().all(|v| v.tags.len() == 1));
    assert_eq!(got[0].tags[0].name, "tag2");
}

/// Erros, rótulos e as descrições padrão saem no idioma da instalação; os dados do usuário
/// (nomes de contas e categorias) ficam como estão.
#[test]
fn messages_follow_the_installation_language() {
    use crate::i18n::{Locale, set_current};
    let mut fx = Fx::new();
    let acc = fx.checking("Corrente", 0);
    let sav = fx.checking("Poupança", 0);
    let mercado = fx.cat("Mercado");
    set_current(Locale::En);
    let e = fx
        .txn_with(Fx::input(&acc, "2026-10-01", 0, "zero"))
        .unwrap_err();
    assert_eq!(e.message(), "the transaction amount can't be zero");
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "Mercado");
    i.splits = Some(vec![
        split(Some(&mercado), -600),
        split(Some(&mercado), -300),
    ]);
    assert_eq!(
        fx.txn_with(i).unwrap_err().message(),
        "the sum of the parts (\u{2212}R$9.00) differs from the total (\u{2212}R$10.00)"
    );
    let e = fx
        .txn_with(Fx::input("nada", "2026-10-01", -1, "x"))
        .unwrap_err();
    assert_eq!(e.message(), "not found: account nada");
    assert_eq!(kind_label(TxnKind::CardPayment), "card bill payment");
    assert_eq!(status_label(TxnStatus::Reconciled), "reconciled");
    let ids = fx
        .write(|tx| create_transfer(tx, &transfer(&acc, &sav, 500, None), d(TODAY)))
        .unwrap();
    assert_eq!(
        get(fx.conn(), &ids.from_id).unwrap().description,
        "Transfer to Poupança"
    );
    set_current(Locale::Es);
    assert_eq!(crate::transactions::api::short(""), "(sin descripción)");
    set_current(Locale::PtBr);
    let ids = fx
        .write(|tx| create_transfer(tx, &transfer(&sav, &acc, 100, None), d(TODAY)))
        .unwrap();
    assert_eq!(
        get(fx.conn(), &ids.from_id).unwrap().description,
        "Transferência para Corrente"
    );
}
