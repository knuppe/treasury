//! Analítico das linhas do fluxo (`sankey_items`): a soma dos itens bate ao centavo com cada
//! célula do comparativo.

use super::fixture::{self, cat, expense, new_db, seed_reference, write};
use super::*;
use crate::transactions::{TxnKind, TxnStatus};

fn items(c: &Connection, from: &str, to: &str, basis: Basis, nodes: &[&str]) -> SankeyItems {
    let ids: Vec<String> = nodes.iter().map(|s| s.to_string()).collect();
    sankey_items(c, from, to, basis, SANKEY_DEFAULT_DEPTH, &ids).unwrap()
}

/// Para cada linha do comparativo: os itens de cada período somam o valor da célula, e os
/// grupos somam a variação.
fn assert_every_row_adds_up(c: &Connection, from: &str, to: &str, basis: Basis, depth: u8) {
    let s = sankey_with(c, from, to, basis, depth, true).unwrap();
    assert!(!s.comparison.is_empty());
    for row in &s.comparison {
        let it = sankey_items(c, from, to, basis, depth, std::slice::from_ref(&row.id)).unwrap();
        let ctx = format!(
            "{} ({}) {from}..{to} {basis:?} profundidade {depth}",
            row.name, row.id
        );
        assert_eq!(it.current.value, row.current, "valor atual de {ctx}");
        assert_eq!(it.current.items_total, row.current, "itens atuais de {ctx}");
        assert_eq!(it.previous.value, row.previous, "valor anterior de {ctx}");
        assert_eq!(
            it.previous.items_total, row.previous,
            "itens anteriores de {ctx}"
        );
        let delta: i64 = it.groups.iter().map(|g| g.delta).sum();
        assert_eq!(delta, row.delta, "grupos de {ctx}");
        assert_eq!(it.name, row.name, "nome de {ctx}");
    }
}

#[test]
fn every_comparison_cell_matches_its_items() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let c = t.db.conn();
    for basis in [Basis::Accrual, Basis::Cash] {
        for depth in [1, 2] {
            assert_every_row_adds_up(c, "2026-10-01", "2026-10-31", basis, depth);
            assert_every_row_adds_up(c, "2026-11-01", "2026-11-30", basis, depth);
            assert_every_row_adds_up(c, "2026-09-01", "2026-10-31", basis, depth);
        }
    }
}

#[test]
fn items_show_splits_refunds_currencies_and_card_due_dates() {
    let mut t = new_db();
    seed_reference(&mut t.db);
    let c = t.db.conn();
    let id = |p: &str| cat(c, p);
    let (oct_from, oct_to) = ("2026-10-01", "2026-10-31");

    // Rendimentos: o que compõe outubro e setembro, com a subcategoria.
    let node = format!("income:{}", id("Rendimentos"));
    let r = items(c, oct_from, oct_to, Basis::Accrual, &[&node]);
    assert_eq!(
        (r.name.as_str(), r.kind),
        ("Rendimentos", SankeyNodeKind::Income)
    );
    assert_eq!((r.current.value, r.previous.value), (20_000, 0));
    let [div] = r.current.items.as_slice() else {
        panic!("{:?}", r.current.items)
    };
    assert_eq!(div.group, "Rendimentos › Dividendos");
    assert_eq!(
        (div.amount, div.date.as_deref()),
        (20_000, Some("2026-10-15"))
    );
    assert_eq!(r.groups.len(), 1);
    assert_eq!((r.groups[0].current, r.groups[0].delta), (20_000, 20_000));

    // Divisão: só a parte do Mercado, marcada como parte.
    let mercado = format!("sub:{}", id("Alimentação › Mercado"));
    let m = items(c, oct_from, oct_to, Basis::Accrual, &[&mercado]);
    let part = m.current.items.iter().find(|i| i.split).unwrap();
    assert_eq!(part.amount, 45_000);
    assert_eq!(m.previous.items_total, 81_234);

    // Estorno entra negativo.
    let rest = format!("sub:{}", id("Alimentação › Restaurantes"));
    let r = items(c, oct_from, oct_to, Basis::Accrual, &[&rest]);
    let mut amounts: Vec<i64> = r.current.items.iter().map(|i| i.amount).collect();
    amounts.sort();
    assert_eq!(amounts, vec![-3_000, 20_000]);

    // Dólar convertido na data; iene sem câmbio fica listado, fora do total.
    let passeios = format!("sub:{}", id("Lazer › Passeios"));
    let p = items(c, oct_from, oct_to, Basis::Accrual, &[&passeios]);
    let usd = p
        .current
        .items
        .iter()
        .find(|i| i.original_currency.as_deref() == Some("USD"))
        .unwrap();
    assert_eq!(
        (usd.amount, usd.original_amount, usd.counted),
        (25_000, Some(5_000), true)
    );
    let jpy = p
        .current
        .items
        .iter()
        .find(|i| i.original_currency.as_deref() == Some("JPY"))
        .unwrap();
    assert!(!jpy.counted && jpy.note.as_deref().unwrap().contains("Sem câmbio"));
    assert_eq!(p.current.items_total, 25_000);
    assert_eq!(p.current.missing_fx, vec!["JPY".to_string()]);

    // Caixa: a compra de setembro no cartão entra em outubro pelo vencimento da fatura.
    let streaming = format!("expense:{}", id("Assinaturas"));
    let s = items(c, oct_from, oct_to, Basis::Cash, &[&streaming]);
    let [i] = s.current.items.as_slice() else {
        panic!("{:?}", s.current.items)
    };
    assert_eq!(
        (i.date.as_deref(), i.due_date.as_deref(), i.amount),
        (Some("2026-09-10"), Some("2026-10-05"), 8_000)
    );
    assert_eq!(s.previous.items_total, 0);
    let a = items(c, oct_from, oct_to, Basis::Accrual, &[&streaming]);
    assert_eq!((a.current.value, a.previous.value), (0, 8_000));
    assert!(a.previous.items[0].due_date.is_none());

    // Sobra: entradas menos saídas, cada uma apontando para o nó dela.
    let sv = items(c, oct_from, oct_to, Basis::Accrual, &["savings"]);
    assert_eq!(sv.current.items_total, 539_000);
    let salario = sv
        .current
        .items
        .iter()
        .find(|i| i.label == "Salário")
        .unwrap();
    let income_node = format!("income:{}", id("Salário"));
    assert_eq!(
        (
            salario.component.as_deref(),
            salario.amount,
            salario.group.as_str()
        ),
        (Some(income_node.as_str()), 1_000_000, "Entradas")
    );
    assert!(
        sv.current
            .items
            .iter()
            .filter(|i| i.group == "Saídas")
            .all(|i| i.amount < 0)
    );
    // Novembro por caixa teve déficit: sem sobra, com o motivo.
    let nov = items(
        c,
        "2026-11-01",
        "2026-11-30",
        Basis::Cash,
        &["savings", "deficit"],
    );
    assert!(
        nov.current
            .note
            .as_deref()
            .unwrap()
            .contains("Não houve sobra")
    );
    assert_eq!(nov.current.items_total, 62_000);

    // Vários nós somam (como "Outras" no gráfico); nós que se sobrepõem, não.
    let pets = format!("expense:{}", id("Pets"));
    let both = items(
        c,
        oct_from,
        oct_to,
        Basis::Accrual,
        &[&pets, "expense:none"],
    );
    assert_eq!(both.current.items_total, 19_000);
    let alim = format!("expense:{}", id("Alimentação"));
    let ids = vec![alim, mercado];
    assert!(sankey_items(c, oct_from, oct_to, Basis::Accrual, 2, &ids).is_err());
    let bad = vec!["hub".to_string()];
    assert!(sankey_items(c, oct_from, oct_to, Basis::Accrual, 2, &bad).is_err());
}

#[test]
fn negative_groups_are_listed_but_left_out() {
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let acc = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
        let lazer = cat(tx, "Lazer");
        let viagens = cat(tx, "Lazer › Viagens");
        let cinema = cat(tx, "Lazer › Cinema e shows");
        // Março: Lazer só com gasto direto (sem subcategorias no diagrama).
        expense(tx, &acc, "2026-03-02", -10_000, Some(&lazer), None, "lazer");
        // Abril: direto, Viagens e Cinema com estorno maior que a compra.
        expense(tx, &acc, "2026-04-02", -5_000, Some(&lazer), None, "lazer");
        expense(
            tx,
            &acc,
            "2026-04-03",
            -20_000,
            Some(&viagens),
            None,
            "viagem",
        );
        expense(
            tx,
            &acc,
            "2026-04-04",
            -1_000,
            Some(&cinema),
            None,
            "cinema",
        );
        expense(
            tx,
            &acc,
            "2026-04-05",
            3_000,
            Some(&cinema),
            None,
            "estorno",
        );
        Ok(())
    });
    let c = t.db.conn();
    let lazer = cat(c, "Lazer");
    let (from, to) = ("2026-04-01", "2026-04-30");
    assert_every_row_adds_up(c, from, to, Basis::Accrual, 2);
    let root = items(c, from, to, Basis::Accrual, &[&format!("expense:{lazer}")]);
    assert_eq!(root.current.items_total, 25_000);
    let out: Vec<&SankeyItem> = root.current.items.iter().filter(|i| !i.counted).collect();
    assert_eq!(out.len(), 2);
    assert!(
        out[0]
            .note
            .as_deref()
            .unwrap()
            .contains("Cinema e shows somou")
    );

    // A linha "Lazer (sem subcategoria)" mostra em março o gasto direto, mesmo que lá a
    // categoria não tivesse subcategorias no diagrama.
    let s = sankey(c, from, to, Basis::Accrual).unwrap();
    let direct = s
        .comparison
        .iter()
        .find(|r| r.id == format!("sub:{lazer}"))
        .unwrap();
    assert_eq!((direct.current, direct.previous), (5_000, 10_000));
}

#[test]
fn investments_redemptions_and_debts_add_up() {
    use crate::categories::{CategoryInput, CategoryKind, create};
    let mut t = new_db();
    let (fin, cons) = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
        let broker = fixture::account(tx, "Corretora", "brokerage", "BRL", 0, "2026-01-01");
        let new_cat = |name: &str, parent: Option<&str>| {
            create(
                tx,
                &CategoryInput {
                    name: name.into(),
                    kind: CategoryKind::Expense,
                    parent_id: parent.map(str::to_string),
                    icon: None,
                    color: None,
                    is_essential: None,
                    spend_group: None,
                },
            )
            .unwrap()
            .id
        };
        let dividas = new_cat("Dívidas", None);
        let fin = new_cat("Financiamento", Some(&dividas));
        let cons = new_cat("Consignado", Some(&dividas));
        for (name, cat) in [("Apartamento", &fin), ("Consignado do Itaú", &cons)] {
            tx.execute(
                "INSERT INTO loan (id, name, principal, annual_rate, system, contract_date,
                                   first_due_date, term_months, category_id, created_at, updated_at)
                 VALUES (?1, ?2, 10000000, '10', 'price', '2025-01-01', '2025-02-01', 120, ?3, ?4, ?4)",
                rusqlite::params![crate::ids::new_id(), name, cat, crate::time::now_utc()],
            )
            .unwrap();
        }
        let salario = cat(tx, "Salário");
        fixture::txn(
            tx,
            &checking,
            "2026-02-05",
            800_000,
            TxnKind::Income,
            Some(&salario),
            None,
            "salário",
        );
        fixture::txn(
            tx,
            &checking,
            "2026-03-05",
            900_000,
            TxnKind::Income,
            Some(&salario),
            None,
            "salário",
        );
        // Fevereiro: um contrato só (o nó de dívidas é ele); março: os dois.
        expense(
            tx,
            &checking,
            "2026-02-10",
            -200_000,
            Some(&fin),
            None,
            "parcela",
        );
        expense(
            tx,
            &checking,
            "2026-03-10",
            -200_000,
            Some(&fin),
            None,
            "parcela",
        );
        expense(
            tx,
            &checking,
            "2026-03-11",
            -50_000,
            Some(&cons),
            None,
            "consignado",
        );
        for (from, to, date, amount) in [
            (&checking, &broker, "2026-02-06", 100_000),
            (&checking, &broker, "2026-03-06", 300_000),
            (&broker, &checking, "2026-03-20", 40_000),
        ] {
            fixture::transfer(
                tx,
                from,
                to,
                date,
                amount,
                TxnKind::Investment,
                TxnStatus::Cleared,
            );
        }
        Ok((fin, cons))
    });
    let c = t.db.conn();
    let (from, to) = ("2026-03-01", "2026-03-31");
    assert_every_row_adds_up(c, from, to, Basis::Accrual, 2);
    assert_every_row_adds_up(c, from, to, Basis::Accrual, 1);
    let inv = items(c, from, to, Basis::Accrual, &["investment"]);
    assert_eq!(
        (inv.current.items_total, inv.previous.items_total),
        (300_000, 100_000)
    );
    assert_eq!(inv.current.items[0].group, "Corretora");
    let red = items(c, from, to, Basis::Accrual, &["redemption"]);
    assert_eq!(red.current.items_total, 40_000);
    // Em fevereiro só havia o financiamento: a linha do contrato mostra o valor dele.
    let s = sankey(c, from, to, Basis::Accrual).unwrap();
    let row = |id: &str| s.comparison.iter().find(|r| r.id == id).unwrap();
    assert_eq!(row(&format!("debt:{fin}")).previous, 200_000);
    assert_eq!(row(&format!("debt:{cons}")).previous, 0);
    let debt = items(c, from, to, Basis::Accrual, &["debt"]);
    assert_eq!(
        (debt.current.items_total, debt.previous.items_total),
        (250_000, 200_000)
    );
}
