use serde_json::json;

use super::*;
use crate::changes::{self, Author};
use crate::import::testutil::TestEnv;

fn cond(field: RuleField, op: RuleOp, value: serde_json::Value) -> RuleCondition {
    RuleCondition {
        field,
        op,
        value,
        value2: None,
    }
}

fn input(
    name: &str,
    priority: i64,
    conditions: Vec<RuleCondition>,
    actions: RuleActions,
) -> RuleInput {
    RuleInput {
        name: name.into(),
        priority: Some(priority),
        active: Some(true),
        match_all: Some(true),
        conditions,
        actions,
    }
}

fn subject<'a>(description: &'a str, amount: i64) -> Subject<'a> {
    Subject {
        description,
        payee: None,
        notes: None,
        amount,
        account_id: "conta",
        kind: if amount < 0 {
            TxnKind::Expense
        } else {
            TxnKind::Income
        },
    }
}

fn compiled(i: &RuleInput, id: &str) -> CompiledRule {
    CompiledRule::compile(id, &i.name, i.priority.unwrap(), i).unwrap()
}

fn create_rule(env: &mut TestEnv, i: &RuleInput) -> Id {
    changes::record(env.db.conn_mut(), &Author::user(), "regra", |tx| {
        super::create(tx, i)
    })
    .unwrap()
    .value
}

#[test]
fn text_ignores_case_and_accents() {
    let r = compiled(
        &input(
            "açougue",
            1,
            vec![cond(
                RuleField::Description,
                RuleOp::Contains,
                json!("Acougue"),
            )],
            RuleActions::default(),
        ),
        "a",
    );
    assert!(r.matches(&subject("PAG*AÇOUGUE BOI GORDO", -100)));
    assert!(!r.matches(&subject("PADARIA", -100)));

    let starts = compiled(
        &input(
            "s",
            1,
            vec![cond(
                RuleField::Description,
                RuleOp::StartsWith,
                json!("uber"),
            )],
            RuleActions::default(),
        ),
        "b",
    );
    assert!(starts.matches(&subject("UBER *TRIP", -1)));
    assert!(!starts.matches(&subject("PAG UBER", -1)));
    let equals = compiled(
        &input(
            "e",
            1,
            vec![cond(
                RuleField::Description,
                RuleOp::Equals,
                json!("netflix.com"),
            )],
            RuleActions::default(),
        ),
        "c",
    );
    assert!(equals.matches(&subject("NETFLIX COM", -1)));
    assert!(!equals.matches(&subject("NETFLIX COM BR", -1)));
}

#[test]
fn regex_and_amount_conditions() {
    let re = compiled(
        &input(
            "pix",
            1,
            vec![cond(
                RuleField::Description,
                RuleOp::Regex,
                json!(r"^pix (enviado|recebido)\b"),
            )],
            RuleActions::default(),
        ),
        "r",
    );
    assert!(re.matches(&subject("PIX ENVIADO - Fulano", -1)));
    assert!(
        re.matches(&subject("Píx recebido", 1)),
        "accents do not matter"
    );
    assert!(!re.matches(&subject("Compra PIX enviado", -1)));

    let mut between = input(
        "faixa",
        1,
        vec![cond(RuleField::Amount, RuleOp::Between, json!(-5000))],
        RuleActions::default(),
    );
    between.conditions[0].value2 = Some(-10000);
    let b = compiled(&between, "b");
    assert!(b.matches(&subject("x", -7500)));
    assert!(b.matches(&subject("x", -5000)));
    assert!(!b.matches(&subject("x", -4999)));
    let gte = compiled(
        &input(
            "gte",
            1,
            vec![cond(RuleField::Amount, RuleOp::Gte, json!("1.000,00"))],
            RuleActions::default(),
        ),
        "g",
    );
    assert!(gte.matches(&subject("salário", 100000)));
    assert!(!gte.matches(&subject("salário", 99999)));

    // OR: any condition.
    let mut any = input(
        "ou",
        1,
        vec![
            cond(RuleField::Description, RuleOp::Contains, json!("spotify")),
            cond(RuleField::Description, RuleOp::Contains, json!("deezer")),
        ],
        RuleActions::default(),
    );
    any.match_all = Some(false);
    let any = compiled(&any, "o");
    assert!(any.matches(&subject("DEEZER", -1)));
    let mut all = any.clone();
    all.match_all = true;
    assert!(!all.matches(&subject("DEEZER", -1)));
}

#[test]
fn invalid_rules_are_rejected() {
    let env = TestEnv::new();
    let conn = env.db.conn();
    let actions = RuleActions {
        tags: Some(vec!["x".into()]),
        ..Default::default()
    };
    let bad_regex = input(
        "x",
        1,
        vec![cond(RuleField::Description, RuleOp::Regex, json!("(abc"))],
        actions.clone(),
    );
    assert!(validate(conn, &bad_regex).is_err());
    let text_on_amount = input(
        "x",
        1,
        vec![cond(RuleField::Amount, RuleOp::Contains, json!(10))],
        actions.clone(),
    );
    assert!(validate(conn, &text_on_amount).is_err());
    let float = input(
        "x",
        1,
        vec![cond(RuleField::Amount, RuleOp::Equals, json!(10.5))],
        actions.clone(),
    );
    assert!(validate(conn, &float).is_err());
    let no_actions = input(
        "x",
        1,
        vec![cond(RuleField::Description, RuleOp::Contains, json!("a"))],
        RuleActions::default(),
    );
    assert!(validate(conn, &no_actions).is_err());
    let no_conditions = input("x", 1, vec![], actions.clone());
    assert!(validate(conn, &no_conditions).is_err());
    let unknown_category = input(
        "x",
        1,
        vec![cond(RuleField::Description, RuleOp::Contains, json!("a"))],
        RuleActions {
            category_id: Some("nao-existe".into()),
            ..Default::default()
        },
    );
    assert!(matches!(
        validate(conn, &unknown_category),
        Err(Error::NotFound(_))
    ));
    let bad_kind = input(
        "x",
        1,
        vec![cond(RuleField::Kind, RuleOp::Equals, json!("despesa"))],
        actions,
    );
    assert!(validate(conn, &bad_kind).is_err());
}

#[test]
fn priority_first_rule_defining_a_field_wins() {
    let mut env = TestEnv::new();
    let delivery = env.category("Delivery");
    let transporte = env.category("Transporte");
    let ana = env.member("Ana");
    let low = create_rule(
        &mut env,
        &input(
            "uber genérico",
            50,
            vec![cond(
                RuleField::Description,
                RuleOp::Contains,
                json!("uber"),
            )],
            RuleActions {
                category_id: Some(transporte.clone()),
                tags: Some(vec!["apps".into()]),
                ..Default::default()
            },
        ),
    );
    let high = create_rule(
        &mut env,
        &input(
            "uber eats",
            10,
            vec![cond(
                RuleField::Description,
                RuleOp::Contains,
                json!("uber eats"),
            )],
            RuleActions {
                category_id: Some(delivery.clone()),
                member_id: Some(ana.clone()),
                ..Default::default()
            },
        ),
    );
    let set = RuleSet::load_active(env.db.conn()).unwrap();
    let out = set.evaluate(&subject("UBER EATS *PEDIDO", -4500));
    assert_eq!(out.category_id.as_ref(), Some(&delivery));
    assert_eq!(out.member_id.as_ref(), Some(&ana));
    assert_eq!(out.tags, Some(vec!["apps".to_string()]));
    assert_eq!(out.matched, vec![high.clone(), low.clone()]);
    assert_eq!(out.applied, vec![high.clone(), low.clone()]);
    assert_eq!(out.first_rule(), Some(&high));

    let out = set.evaluate(&subject("UBER *TRIP", -2000));
    assert_eq!(out.category_id.as_ref(), Some(&transporte));
    assert_eq!(out.applied, vec![low.clone()]);

    // An inactive rule does not count; requested by id, it does.
    changes::record(env.db.conn_mut(), &Author::user(), "desativar", |tx| {
        let mut i = input(
            "uber eats",
            10,
            vec![cond(
                RuleField::Description,
                RuleOp::Contains,
                json!("uber eats"),
            )],
            RuleActions {
                category_id: Some(delivery.clone()),
                ..Default::default()
            },
        );
        i.active = Some(false);
        update(tx, &high, &i)
    })
    .unwrap();
    let set = RuleSet::load_active(env.db.conn()).unwrap();
    assert_eq!(set.rules().len(), 1);
    let set = RuleSet::load(env.db.conn(), Some(std::slice::from_ref(&high))).unwrap();
    assert_eq!(set.rules().len(), 1);
    assert!(RuleSet::load(env.db.conn(), Some(&["x".to_string()])).is_err());

    // Listing by priority and JSON of the columns.
    let rules = list(env.db.conn()).unwrap();
    assert_eq!(rules[0].id, high);
    let raw: String = env
        .db
        .conn()
        .query_row("SELECT conditions FROM rule WHERE id = ?1", [&low], |r| {
            r.get(0)
        })
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["match_all"], true);
    assert_eq!(v["items"][0]["op"], "contains");
}

#[test]
fn apply_to_new_fills_empty_fields_and_respects_transfers() {
    let mut env = TestEnv::new();
    let streaming = env.category("Streaming");
    create_rule(
        &mut env,
        &input(
            "netflix",
            1,
            vec![cond(
                RuleField::Description,
                RuleOp::Contains,
                json!("netflix"),
            )],
            RuleActions {
                category_id: Some(streaming.clone()),
                tags: Some(vec!["assinatura".into()]),
                rename_to: Some("Netflix".into()),
                ..Default::default()
            },
        ),
    );
    create_rule(
        &mut env,
        &input(
            "ted própria",
            2,
            vec![cond(
                RuleField::Description,
                RuleOp::Contains,
                json!("mesma titularidade"),
            )],
            RuleActions {
                kind: Some(TxnKind::Transfer),
                category_id: Some(streaming.clone()),
                ..Default::default()
            },
        ),
    );
    let mut t = NewTxn::new(
        "c",
        "2026-10-01",
        -3990,
        "BRL",
        TxnKind::Expense,
        "NETFLIX.COM 0800",
    );
    let out = apply_to_new(env.db.conn(), &mut t).unwrap();
    assert_eq!(t.category_id.as_ref(), Some(&streaming));
    assert_eq!(t.description, "Netflix");
    assert_eq!(out.tags, Some(vec!["assinatura".to_string()]));

    let mut t = NewTxn::new(
        "c",
        "2026-10-01",
        -100000,
        "BRL",
        TxnKind::Expense,
        "TED MESMA TITULARIDADE",
    );
    apply_to_new(env.db.conn(), &mut t).unwrap();
    assert_eq!(t.kind, TxnKind::Transfer);
    // A one-leg transfer also takes a category (the account in the flow through it).
    assert!(t.category_id.is_some());

    let other = env.category("Cinema e shows");
    let mut t = NewTxn::new("c", "2026-10-01", -3990, "BRL", TxnKind::Expense, "NETFLIX");
    t.category_id = Some(other.clone());
    apply_to_new(env.db.conn(), &mut t).unwrap();
    assert_eq!(
        t.category_id,
        Some(other),
        "a chosen category is not replaced"
    );
}

#[test]
fn apply_dry_run_write_hits_and_undo() {
    let mut env = TestEnv::new();
    let conta = env.add_account("Conta", "checking");
    let mercado = env.category("Mercado");
    let restaurantes = env.category("Restaurantes");
    let a = env.txn(&conta, "2026-10-01", -12000, "SUPERMERCADO BOM PRECO", None);
    let b = env.txn(
        &conta,
        "2026-10-02",
        -8000,
        "Supermercado Bom Preço Filial",
        None,
    );
    let c = env.txn(
        &conta,
        "2026-10-03",
        -5000,
        "SUPERMERCADO XYZ",
        Some(&restaurantes),
    );
    env.txn(&conta, "2026-10-04", -5000, "FARMACIA", None);
    let rule = create_rule(
        &mut env,
        &input(
            "mercado",
            1,
            vec![cond(
                RuleField::Description,
                RuleOp::Contains,
                json!("supermercado"),
            )],
            RuleActions {
                category_id: Some(mercado.clone()),
                tags: Some(vec!["casa".into()]),
                ..Default::default()
            },
        ),
    );

    // rules.test: catches the 3 supermarkets (including the one already categorized).
    let tested = test(
        env.db.conn(),
        &RuleTestArgs {
            rule: get_input(&env, &rule),
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tested.len(), 3);

    let args = RuleApplyArgs {
        rule_ids: None,
        filter: None,
        only_uncategorized: None,
        dry_run: Some(true),
    };
    let r = apply(&mut env.ctx(), &args).unwrap();
    assert_eq!((r.matched, r.updated), (2, 2));
    assert!(r.change_set_id.is_none());
    assert!(
        r.sample
            .iter()
            .all(|v| v.category_id.as_ref() == Some(&mercado))
    );
    assert!(
        r.sample[0]
            .category_path
            .as_deref()
            .unwrap()
            .contains("Mercado")
    );
    assert_eq!(
        env.count("SELECT count(*) FROM txn WHERE category_id IS NOT NULL"),
        1,
        "a simulation does not write"
    );

    let r = apply(
        &mut env.ctx(),
        &RuleApplyArgs {
            dry_run: None,
            ..args.clone()
        },
    )
    .unwrap();
    assert_eq!(r.updated, 2);
    let cs = r.change_set_id.clone().unwrap();
    let cat = |env: &TestEnv, id: &str| -> Option<String> {
        env.db
            .conn()
            .query_row("SELECT category_id FROM txn WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap()
    };
    assert_eq!(cat(&env, &a), Some(mercado.clone()));
    assert_eq!(cat(&env, &b), Some(mercado.clone()));
    assert_eq!(
        cat(&env, &c),
        Some(restaurantes.clone()),
        "only without a category"
    );
    assert_eq!(r.sample[0].tags[0].name, "casa");
    assert_eq!(get(env.db.conn(), &rule).unwrap().hits, 2);

    // Again: nothing to change.
    let again = apply(&mut env.ctx(), &args).unwrap();
    assert_eq!(again.updated, 0);

    // Also on the categorized ones.
    let all = apply(
        &mut env.ctx(),
        &RuleApplyArgs {
            only_uncategorized: Some(false),
            dry_run: Some(true),
            ..args.clone()
        },
    )
    .unwrap();
    assert_eq!(all.updated, 1);

    changes::undo(env.db.conn_mut(), &cs).unwrap();
    assert_eq!(cat(&env, &a), None);
    assert_eq!(env.count("SELECT count(*) FROM txn_tag"), 0);
    assert_eq!(
        get(env.db.conn(), &rule).unwrap().hits,
        2,
        "hits are statistics, outside the history"
    );
}

fn get_input(env: &TestEnv, id: &str) -> RuleInput {
    let r = get(env.db.conn(), id).unwrap();
    RuleInput {
        name: r.name,
        priority: Some(r.priority),
        active: Some(r.active),
        match_all: Some(r.match_all),
        conditions: r.conditions,
        actions: r.actions,
    }
}

#[test]
fn suggests_rule_from_transaction() {
    let mut env = TestEnv::new();
    let conta = env.add_account("Conta", "checking");
    let transporte = env.category("Transporte");
    let delivery = env.category("Delivery");
    env.txn(
        &conta,
        "2026-10-01",
        -4500,
        "UBER EATS *PEDIDO",
        Some(&delivery),
    );
    let t = env.txn(
        &conta,
        "2026-10-02",
        -2000,
        "UBER *TRIP HELP.UBER.COM",
        Some(&transporte),
    );
    let s = suggest_from_txn(env.db.conn(), &t).unwrap();
    assert_eq!(s.conditions[0].op, RuleOp::Contains);
    assert_eq!(
        s.conditions[0].value,
        json!("uber trip"),
        "\"uber\" alone would catch the delivery"
    );
    assert_eq!(s.actions.category_id, Some(transporte));
    assert!(validate(env.db.conn(), &s).is_ok());

    let p = env.txn(
        &conta,
        "2026-10-03",
        -1590,
        "Compra no débito - PADARIA SAO JOAO 123456",
        None,
    );
    let s = suggest_from_txn(env.db.conn(), &p).unwrap();
    assert_eq!(s.conditions[0].value, json!("padaria"));
    assert!(s.actions.category_id.is_none());
    assert!(suggest_from_txn(env.db.conn(), "nada").is_err());
    assert_eq!(
        significant_run("PIX ENVIADO - JOAO DA SILVA"),
        vec!["joao".to_string()],
        "stops at linking words"
    );
}
