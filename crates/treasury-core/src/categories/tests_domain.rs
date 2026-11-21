use super::*;
use crate::transactions::testkit::Fx;

fn input(name: &str, kind: CategoryKind, parent: Option<&str>) -> CategoryInput {
    CategoryInput {
        name: name.into(),
        kind,
        parent_id: parent.map(str::to_string),
        icon: None,
        color: None,
        is_essential: None,
        spend_group: None,
    }
}

fn patch(id: &str) -> CategoryPatch {
    CategoryPatch {
        id: id.into(),
        name: None,
        parent_id: None,
        icon: None,
        color: None,
        is_essential: None,
        spend_group: None,
        archived: None,
        sort_order: None,
    }
}

#[test]
fn list_has_paths_and_tree_order() {
    let fx = Fx::new();
    let all = list(fx.conn(), &CategoryListArgs::default()).unwrap();
    let mercado = all.iter().find(|c| c.name == "Mercado").unwrap();
    assert_eq!(mercado.path, "Alimentação › Mercado");
    // Parents before children; expenses before income.
    let pos = |n: &str| all.iter().position(|c| c.name == n).unwrap();
    assert!(pos("Alimentação") < pos("Mercado"));
    assert!(pos("Outras despesas") < pos("Salário"));
    let tree = tree(
        fx.conn(),
        &CategoryListArgs {
            kind: Some(CategoryKind::Income),
            include_archived: None,
        },
    )
    .unwrap();
    assert!(tree.iter().all(|n| n.category.kind == CategoryKind::Income));
    let rend = tree
        .iter()
        .find(|n| n.category.name == "Rendimentos")
        .unwrap();
    assert_eq!(rend.children.len(), 6);
    assert_eq!(path_map(fx.conn()).unwrap()[&mercado.id], mercado.path);
}

#[test]
fn create_validates_kind_and_siblings() {
    let mut fx = Fx::new();
    let alim = fx.cat("Alimentação");
    let feira = fx
        .write(|tx| create(tx, &input("Feira", CategoryKind::Expense, Some(&alim))))
        .unwrap();
    assert_eq!(feira.path, "Alimentação › Feira");
    assert!(feira.is_essential, "inherits essential from the parent");
    assert!(matches!(
        fx.write(|tx| create(tx, &input("Bônus", CategoryKind::Income, Some(&alim)))),
        Err(Error::Invalid(_))
    ));
    // Same name at the same level, ignoring case and accents.
    assert!(matches!(
        fx.write(|tx| create(tx, &input("PADARIA", CategoryKind::Expense, Some(&alim)))),
        Err(Error::Conflict(_))
    ));
    assert!(matches!(
        fx.write(|tx| create(tx, &input("alimentacao", CategoryKind::Expense, None))),
        Err(Error::Conflict(_))
    ));
    // The same name under another parent is allowed.
    let lazer = fx.cat("Lazer");
    fx.write(|tx| create(tx, &input("Feira", CategoryKind::Expense, Some(&lazer))))
        .unwrap();
}

#[test]
fn update_prevents_cycles_and_kind_mixing() {
    let mut fx = Fx::new();
    let alim = fx.cat("Alimentação");
    let mercado = fx.cat("Mercado");
    let salario = fx.cat("Salário");
    let mut p = patch(&alim);
    p.parent_id = Some(Some(mercado.clone()));
    assert!(fx.write(|tx| update(tx, &p)).is_err(), "cycle");
    let mut p = patch(&alim);
    p.parent_id = Some(Some(alim.clone()));
    assert!(fx.write(|tx| update(tx, &p)).is_err(), "its own parent");
    let mut p = patch(&mercado);
    p.parent_id = Some(Some(salario));
    assert!(fx.write(|tx| update(tx, &p)).is_err(), "income × expense");
    // Move to the root and rename.
    let mut p = patch(&mercado);
    p.parent_id = Some(None);
    p.name = Some("Supermercado".into());
    p.archived = Some(true);
    let c = fx.write(|tx| update(tx, &p)).unwrap();
    assert_eq!(c.path, "Supermercado");
    assert!(c.parent_id.is_none());
    let visible = list(fx.conn(), &CategoryListArgs::default()).unwrap();
    assert!(!visible.iter().any(|c| c.name == "Supermercado"));
    let all = list(
        fx.conn(),
        &CategoryListArgs {
            kind: None,
            include_archived: Some(true),
        },
    )
    .unwrap();
    assert!(all.iter().any(|c| c.name == "Supermercado"));
}

#[test]
fn delete_requires_reassign_when_used_and_moves_everything() {
    let mut fx = Fx::new();
    let acc = fx.checking("Conta", 0);
    let mercado = fx.cat("Mercado");
    let padaria = fx.cat("Padaria");
    let racao = fx.cat("Ração");
    let mut i = Fx::input(&acc, "2026-10-01", -1000, "Mercado do bairro");
    i.category_id = Some(mercado.clone());
    fx.txn_with(i).unwrap();
    let mut i = Fx::input(&acc, "2026-10-02", -1000, "Compra dividida");
    i.splits = Some(vec![
        crate::transactions::SplitInput {
            category_id: Some(mercado.clone()),
            member_id: None,
            amount: -600,
            notes: None,
        },
        crate::transactions::SplitInput {
            category_id: Some(racao),
            member_id: None,
            amount: -400,
            notes: None,
        },
    ]);
    fx.txn_with(i).unwrap();
    // Budget with both categories in the same month, rule pointing to Mercado.
    fx.conn()
        .execute_batch(&format!(
            "INSERT INTO budget (id, month, created_at, updated_at) VALUES ('b1', '2026-10', 'a', 'a');
             INSERT INTO budget_line VALUES ('l1', 'b1', '{mercado}', 50000, 'a', 'a');
             INSERT INTO budget_line VALUES ('l2', 'b1', '{padaria}', 10000, 'a', 'a');
             INSERT INTO rule (id, name, conditions, actions, created_at, updated_at)
               VALUES ('r1', 'mercado', '[]', '{{\"category_id\": \"{mercado}\"}}', 'a', 'a');"
        ))
        .unwrap();
    let u = usage(fx.conn(), &mercado).unwrap();
    assert_eq!(
        (u.transactions, u.splits, u.budget_lines, u.rules),
        (1, 1, 1, 1)
    );
    assert!(matches!(
        fx.write(|tx| delete(tx, &mercado, None)),
        Err(Error::Conflict(_))
    ));
    let salario = fx.cat("Salário");
    assert!(fx.write(|tx| delete(tx, &mercado, Some(&salario))).is_err());
    let moved = fx.write(|tx| delete(tx, &mercado, Some(&padaria))).unwrap();
    assert_eq!(moved.transactions, 1);
    assert_eq!(moved.splits, 1);
    assert_eq!(moved.rules, 1);
    assert_eq!(
        fx.count("SELECT amount FROM budget_line WHERE budget_id = 'b1'"),
        60000,
        "lines of the same month summed"
    );
    assert_eq!(fx.count("SELECT count(*) FROM budget_line"), 1);
    let actions: String = fx
        .conn()
        .query_row("SELECT actions FROM rule WHERE id = 'r1'", [], |r| r.get(0))
        .unwrap();
    assert!(actions.contains(&padaria));
    assert_eq!(
        fx.count(&format!(
            "SELECT count(*) FROM txn WHERE category_id = '{padaria}'"
        )),
        1
    );
    // Unused: deleted outright; with subcategories: not.
    let alim = fx.cat("Alimentação");
    assert!(matches!(
        fx.write(|tx| delete(tx, &alim, None)),
        Err(Error::Conflict(_))
    ));
    let delivery = fx.cat("Delivery");
    fx.write(|tx| delete(tx, &delivery, None)).unwrap();
    assert!(get(fx.conn(), &delivery).is_err());
}

#[test]
fn merge_moves_children_and_merges_same_names() {
    let mut fx = Fx::new();
    let acc = fx.checking("Conta", 0);
    let compras = fx.cat("Compras");
    let lazer = fx.cat("Lazer");
    // "Hobbies" exists under Lazer; we also create "Hobbies" under Compras.
    let hob2 = fx
        .write(|tx| create(tx, &input("hobbies", CategoryKind::Expense, Some(&compras))))
        .unwrap();
    let mut i = Fx::input(&acc, "2026-10-01", -500, "Tinta");
    i.category_id = Some(hob2.id.clone());
    let t = fx.txn_with(i).unwrap();
    let into = fx.write(|tx| merge(tx, &compras, &lazer)).unwrap();
    assert_eq!(into.id, lazer);
    assert!(get(fx.conn(), &compras).is_err());
    assert!(
        get(fx.conn(), &hob2.id).is_err(),
        "merged with Lazer's Hobbies"
    );
    let hobbies = fx.cat("Hobbies");
    assert_eq!(
        crate::transactions::get(fx.conn(), &t).unwrap().category_id,
        Some(hobbies)
    );
    // Vestuário went inside Lazer.
    let vest = get(fx.conn(), &fx.cat("Vestuário")).unwrap();
    assert_eq!(vest.path, "Lazer › Vestuário");
    // Merging with its own subcategory is rejected.
    let viagens = fx.cat("Viagens");
    assert!(fx.write(|tx| merge(tx, &lazer, &viagens)).is_err());
}

fn group_of(fx: &Fx, name: &str) -> (Option<SpendGroup>, bool) {
    let c = get(fx.conn(), &fx.cat(name)).unwrap();
    (c.spend_group, c.is_essential)
}

#[test]
fn seeding_gives_each_expense_its_default_group() {
    let fx = Fx::new();
    use SpendGroup::*;
    for (name, group) in [
        ("Moradia", Essential),
        ("Aluguel", Essential),
        ("Mercado", Essential),
        ("Restaurantes", NonEssential),
        ("Delivery", NonEssential),
        ("Aplicativos", NonEssential),
        ("Lazer", NonEssential),
        ("Viagens", NonEssential),
        ("Streaming", NonEssential),
        ("Pagamentos", Essential),
        ("Contas e boletos", Essential),
        ("Empréstimos", Debt),
        ("Financiamentos", Debt),
        ("Parcelas", Debt),
        ("Juros e multas", Debt),
        ("Tarifas bancárias", Debt),
        ("Imposto de renda", Essential),
        ("Poupança e investimentos", Future),
        ("Reserva de emergência", Future),
        ("Aportes em investimentos", Future),
        ("Poupança dos filhos", Future),
        ("Brinquedos", NonEssential),
        ("Outras despesas", NonEssential),
    ] {
        assert_eq!(
            group_of(&fx, name),
            (Some(group), group.is_essential()),
            "{name}"
        );
    }
    // Income has no group.
    assert_eq!(group_of(&fx, "Salário"), (None, false));
    let all = list(fx.conn(), &CategoryListArgs::default()).unwrap();
    assert!(
        all.iter()
            .all(|c| (c.kind == CategoryKind::Expense) == c.spend_group.is_some())
    );
    assert!(
        all.iter()
            .all(|c| c.is_essential == c.spend_group.is_some_and(SpendGroup::is_essential))
    );
}

#[test]
fn created_categories_inherit_the_parent_group() {
    let mut fx = Fx::new();
    let alim = fx.cat("Alimentação");
    let poup = fx.cat("Poupança e investimentos");
    let emp = fx.cat("Empréstimos");
    let mut make = |name: &str, parent: Option<&str>, essential: Option<bool>| {
        let mut i = input(name, CategoryKind::Expense, parent);
        i.is_essential = essential;
        fx.write(|tx| create(tx, &i)).unwrap()
    };
    let feira = make("Feira", Some(&alim), None);
    assert_eq!(
        (feira.spend_group, feira.is_essential),
        (Some(SpendGroup::Essential), true)
    );
    let doces = make("Doces", Some(&alim), Some(false));
    assert_eq!(doces.spend_group, Some(SpendGroup::NonEssential));
    let cripto = make("Cripto", Some(&poup), None);
    assert_eq!(
        (cripto.spend_group, cripto.is_essential),
        (Some(SpendGroup::Future), false)
    );
    // Debt counts as essential; marking it essential keeps it debt, unmarking removes it.
    let consignado = make("Consignado", Some(&emp), None);
    assert_eq!(
        (consignado.spend_group, consignado.is_essential),
        (Some(SpendGroup::Debt), true)
    );
    let marcada_divida = make("Cheque especial", Some(&emp), Some(true));
    assert_eq!(marcada_divida.spend_group, Some(SpendGroup::Debt));
    let quitada = make("Quitada", Some(&emp), Some(false));
    assert_eq!(
        (quitada.spend_group, quitada.is_essential),
        (Some(SpendGroup::NonEssential), false)
    );
    let previdencia = make("Previdência da empresa", Some(&poup), Some(true));
    assert_eq!(
        (previdencia.spend_group, previdencia.is_essential),
        (Some(SpendGroup::Essential), true)
    );
    let solta = make("Solta", None, None);
    assert_eq!(solta.spend_group, Some(SpendGroup::NonEssential));
    let marcada = make("Marcada", None, Some(true));
    assert_eq!(marcada.spend_group, Some(SpendGroup::Essential));
    let mut i = input("Escolhida", CategoryKind::Expense, Some(&alim));
    i.spend_group = Some(SpendGroup::Future);
    i.is_essential = Some(true);
    let c = fx.write(|tx| create(tx, &i)).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::Future), false)
    );
    // Income: no group, and asking for one is an error.
    let bonus = fx
        .write(|tx| create(tx, &input("Bônus", CategoryKind::Income, None)))
        .unwrap();
    assert_eq!((bonus.spend_group, bonus.is_essential), (None, false));
    let mut i = input("Prêmio", CategoryKind::Income, None);
    i.spend_group = Some(SpendGroup::Debt);
    assert!(matches!(
        fx.write(|tx| create(tx, &i)),
        Err(Error::Invalid(_))
    ));
}

#[test]
fn group_and_essential_stay_in_sync_on_update_and_set_group_is_undoable() {
    let mut fx = Fx::new();
    let pets = fx.cat("Pets");
    let moradia = fx.cat("Moradia");
    let emp = fx.cat("Empréstimos");
    let salario = fx.cat("Salário");
    let viagens = fx.cat("Viagens");
    let mut p = patch(&pets);
    p.is_essential = Some(true);
    let c = fx.write(|tx| update(tx, &p)).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::Essential), true)
    );
    p.is_essential = Some(false);
    let c = fx.write(|tx| update(tx, &p)).unwrap();
    assert_eq!(c.spend_group, Some(SpendGroup::NonEssential));
    // Debt counts as essential: marking keeps the group; unmarking makes it non-essential.
    let mut p = patch(&emp);
    p.is_essential = Some(true);
    let c = fx.write(|tx| update(tx, &p)).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::Debt), true)
    );
    p.is_essential = Some(false);
    let c = fx.write(|tx| update(tx, &p)).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::NonEssential), false)
    );
    // Future marked as essential goes to essentials.
    let mut p = patch(&fx.cat("Reserva de emergência"));
    p.is_essential = Some(true);
    let c = fx.write(|tx| update(tx, &p)).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::Essential), true)
    );
    // The group wins over "essential".
    let mut p = patch(&moradia);
    p.spend_group = Some(SpendGroup::Future);
    p.is_essential = Some(true);
    let c = fx.write(|tx| update(tx, &p)).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::Future), false)
    );
    let mut p = patch(&salario);
    p.spend_group = Some(SpendGroup::Essential);
    assert!(matches!(
        fx.write(|tx| update(tx, &p)),
        Err(Error::Invalid(_))
    ));

    // In batch: the income in the list stays as is; repeating does not count again.
    let ids = vec![pets.clone(), viagens.clone(), salario.clone(), pets.clone()];
    let (n, cs1) = fx
        .write_cs(|tx| set_group(tx, &ids, SpendGroup::Essential))
        .unwrap();
    assert_eq!(n, 2);
    for id in [&pets, &viagens] {
        let c = get(fx.conn(), id).unwrap();
        assert_eq!(
            (c.spend_group, c.is_essential),
            (Some(SpendGroup::Essential), true)
        );
    }
    assert_eq!(get(fx.conn(), &salario).unwrap().spend_group, None);
    let (n, cs2) = fx
        .write_cs(|tx| set_group(tx, std::slice::from_ref(&pets), SpendGroup::Debt))
        .unwrap();
    assert_eq!(n, 1);
    let c = get(fx.conn(), &pets).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::Debt), true)
    );
    // Undo restores the group and "essential" together.
    fx.undo(&cs2.unwrap()).unwrap();
    let c = get(fx.conn(), &pets).unwrap();
    assert_eq!(
        (c.spend_group, c.is_essential),
        (Some(SpendGroup::Essential), true)
    );
    fx.undo(&cs1.unwrap()).unwrap();
    for id in [&pets, &viagens] {
        let c = get(fx.conn(), id).unwrap();
        assert_eq!(
            (c.spend_group, c.is_essential),
            (Some(SpendGroup::NonEssential), false)
        );
    }
    assert!(matches!(
        fx.write(|tx| set_group(tx, &["nao-existe".to_string()], SpendGroup::Future)),
        Err(Error::NotFound(_))
    ));
}
