//! Testes dos relatórios com o cenário de referência de `fixture` (valores à mão).

use std::collections::HashMap;

use super::fixture::{self, Reference, cat, expense, new_db, seed_reference, write};
use super::*;
use crate::time::parse_date;

fn reference() -> (fixture::TestDb, Reference) {
    let mut t = new_db();
    let r = seed_reference(&mut t.db);
    (t, r)
}

fn args(from: &str, to: &str, group_by: GroupBy) -> SpendingArgs {
    SpendingArgs {
        from: from.into(),
        to: to.into(),
        basis: Some(Basis::Accrual),
        kind: None,
        group_by: Some(group_by),
        member_ids: None,
        account_ids: None,
        category_ids: None,
    }
}

fn by_key(groups: &[GroupAmount]) -> HashMap<Option<String>, (i64, i64)> {
    groups
        .iter()
        .map(|g| (g.key.clone(), (g.amount, g.count)))
        .collect()
}

/// Em cada nó que recebe e emite fluxo, o que entra é igual ao que sai.
fn assert_sankey_balanced(s: &SankeyData) {
    let mut inflow: HashMap<&str, i64> = HashMap::new();
    let mut outflow: HashMap<&str, i64> = HashMap::new();
    for l in &s.links {
        assert!(l.value > 0, "link sem valor: {l:?}");
        *outflow.entry(&l.source).or_default() += l.value;
        *inflow.entry(&l.target).or_default() += l.value;
    }
    for (node, out) in &outflow {
        if let Some(i) = inflow.get(node) {
            assert_eq!(i, out, "nó {node} não fecha");
        }
    }
    let ids: Vec<&str> = s.nodes.iter().map(|n| n.id.as_str()).collect();
    for l in &s.links {
        assert!(ids.contains(&l.source.as_str()) && ids.contains(&l.target.as_str()));
    }
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "nós repetidos");
}

fn link(s: &SankeyData, source: &str, target: &str) -> Option<i64> {
    s.links
        .iter()
        .find(|l| l.source == source && l.target == target)
        .map(|l| l.value)
}

#[test]
fn month_flow_accrual_and_cash() {
    let (t, _) = reference();
    let c = t.db.conn();
    let f = month_flow(c, "2026-08", "2026-11", Basis::Accrual).unwrap();
    assert_eq!(f.len(), 4);
    assert_eq!(
        (f[0].income, f[0].expense, f[0].savings_rate),
        (0, 290_000, None)
    );
    // Setembro: aluguel 3.000 + mercado 812,34 + streaming 80 + combustível 120.
    assert_eq!((f[1].income, f[1].expense), (1_000_000, 401_234));
    // Outubro: transferência, pagamento de fatura e previstos não contam; JPY sem câmbio.
    let oct = &f[2];
    assert_eq!(
        (oct.income, oct.expense, oct.net),
        (1_020_000, 481_000, 539_000)
    );
    assert!((oct.savings_rate.unwrap() - 539_000.0 / 1_020_000.0).abs() < 1e-12);
    assert_eq!(oct.missing_fx, vec!["JPY".to_string()]);
    assert!(!oct.estimated);
    assert_eq!((f[3].income, f[3].expense), (0, 0));

    // Caixa: as compras no cartão contam no vencimento da fatura.
    let f = month_flow(c, "2026-09", "2026-11", Basis::Cash).unwrap();
    assert_eq!(f[0].expense, 381_234);
    assert_eq!(f[1].expense, 439_000);
    assert_eq!((f[2].income, f[2].expense), (0, 62_000));
}

#[test]
fn spending_groups_filters_and_counts() {
    let (t, r) = reference();
    let c = t.db.conn();
    let s = spending(c, &args("2026-10-01", "2026-10-31", GroupBy::Category)).unwrap();
    assert_eq!(s.total, 481_000);
    // 8 lançamentos: o dividido conta uma vez no total (e uma vez em cada grupo).
    assert_eq!(s.count, 8);
    assert_eq!(s.missing_fx, vec!["JPY".to_string()]);
    let g = by_key(&s.groups);
    let k = |p: &str| Some(cat(c, p));
    assert_eq!(g[&k("Moradia › Aluguel")], (300_000, 1));
    assert_eq!(g[&k("Moradia › Energia")], (25_000, 1));
    assert_eq!(g[&k("Alimentação › Mercado")], (45_000, 1));
    assert_eq!(g[&k("Pets › Ração")], (15_000, 1));
    assert_eq!(g[&k("Alimentação › Restaurantes")], (17_000, 2));
    assert_eq!(g[&k("Lazer › Viagens")], (50_000, 1));
    assert_eq!(g[&k("Lazer › Passeios")], (25_000, 1));
    assert_eq!(g[&None], (4_000, 1));
    assert_eq!(s.groups[0].label, "Aluguel");
    assert!((s.groups[0].share - 300_000.0 / 481_000.0).abs() < 1e-12);
    let mercado = s.groups.iter().find(|x| x.label == "Mercado").unwrap();
    assert_eq!(mercado.parent_key, k("Alimentação"));
    assert_eq!(mercado.path.as_deref(), Some("Alimentação › Mercado"));
    let none = s.groups.iter().find(|x| x.key.is_none()).unwrap();
    assert_eq!(none.label, "Sem categoria");

    let s = spending(
        c,
        &args("2026-10-01", "2026-10-31", GroupBy::ParentCategory),
    )
    .unwrap();
    let order: Vec<(&str, i64)> = s
        .groups
        .iter()
        .map(|g| (g.label.as_str(), g.amount))
        .collect();
    assert_eq!(
        order,
        vec![
            ("Moradia", 325_000),
            ("Lazer", 75_000),
            ("Alimentação", 62_000),
            ("Pets", 15_000),
            ("Sem categoria", 4_000)
        ]
    );
    // Alimentação: o dividido (Mercado) e dois lançamentos de restaurante.
    assert_eq!(s.groups[2].count, 3);

    let s = spending(c, &args("2026-10-01", "2026-10-31", GroupBy::Member)).unwrap();
    let g = by_key(&s.groups);
    // A parte do mercado sem titular herda Carlos; a da ração é da Ana.
    assert_eq!(g[&Some(r.carlos.clone())].0, 420_000);
    assert_eq!(g[&Some(r.ana.clone())].0, 32_000);
    assert_eq!(g[&None].0, 29_000);
    assert_eq!(s.groups[0].label, "Carlos");

    let s = spending(c, &args("2026-10-01", "2026-10-31", GroupBy::Account)).unwrap();
    let g = by_key(&s.groups);
    assert_eq!(g[&Some(r.checking.clone())].0, 406_000);
    assert_eq!(g[&Some(r.card.clone())].0, 50_000);
    assert_eq!(g[&Some(r.usd.clone())].0, 25_000);
    assert!(
        !g.contains_key(&Some(r.jpy.clone())),
        "sem câmbio, fora do total"
    );

    let s = spending(c, &args("2026-07-15", "2026-10-31", GroupBy::Month)).unwrap();
    let months: Vec<(Option<&str>, i64)> = s
        .groups
        .iter()
        .map(|g| (g.key.as_deref(), g.amount))
        .collect();
    assert_eq!(
        months,
        vec![
            (Some("2026-07"), 0),
            (Some("2026-08"), 290_000),
            (Some("2026-09"), 401_234),
            (Some("2026-10"), 481_000)
        ]
    );
    assert_eq!(s.groups[3].label, "out/2026");

    // Filtros: categoria (com subcategorias), titular, conta e receitas.
    let mut a = args("2026-10-01", "2026-10-31", GroupBy::ParentCategory);
    a.category_ids = Some(vec![cat(c, "Alimentação")]);
    assert_eq!(spending(c, &a).unwrap().total, 62_000);
    let mut a = args("2026-10-01", "2026-10-31", GroupBy::Category);
    a.member_ids = Some(vec![r.ana.clone()]);
    assert_eq!(spending(c, &a).unwrap().total, 32_000);
    let mut a = args("2026-10-01", "2026-10-31", GroupBy::Category);
    a.account_ids = Some(vec![r.card.clone()]);
    assert_eq!(spending(c, &a).unwrap().total, 50_000);
    let mut a = args("2026-10-01", "2026-10-31", GroupBy::ParentCategory);
    a.kind = Some(CategoryKind::Income);
    let s = spending(c, &a).unwrap();
    assert_eq!(s.total, 1_020_000);
    assert_eq!(s.groups[0].label, "Salário");
    assert_eq!(s.groups[1].label, "Rendimentos");

    assert!(spending(c, &args("2026-10-31", "2026-10-01", GroupBy::Category)).is_err());
}

#[test]
fn sankey_with_savings_closes_every_node() {
    let (t, _) = reference();
    let c = t.db.conn();
    let s = sankey(c, "2026-10-01", "2026-10-31", Basis::Accrual).unwrap();
    assert_sankey_balanced(&s);
    let id = |p: &str| cat(c, p);
    assert_eq!(
        link(&s, &format!("income:{}", id("Salário")), "hub"),
        Some(1_000_000)
    );
    assert_eq!(
        link(&s, &format!("income:{}", id("Rendimentos")), "hub"),
        Some(20_000)
    );
    assert_eq!(
        link(&s, "hub", &format!("expense:{}", id("Moradia"))),
        Some(325_000)
    );
    assert_eq!(
        link(&s, "hub", &format!("expense:{}", id("Lazer"))),
        Some(75_000)
    );
    assert_eq!(
        link(&s, "hub", &format!("expense:{}", id("Alimentação"))),
        Some(62_000)
    );
    assert_eq!(
        link(&s, "hub", &format!("expense:{}", id("Pets"))),
        Some(15_000)
    );
    assert_eq!(link(&s, "hub", "expense:none"), Some(4_000));
    assert_eq!(link(&s, "hub", "savings"), Some(539_000));
    assert_eq!(
        link(
            &s,
            &format!("expense:{}", id("Alimentação")),
            &format!("sub:{}", id("Alimentação › Restaurantes"))
        ),
        Some(17_000)
    );
    assert_eq!(
        link(
            &s,
            &format!("expense:{}", id("Moradia")),
            &format!("sub:{}", id("Moradia › Aluguel"))
        ),
        Some(300_000)
    );
    // A Internet prevista não entra; o grupo sem categoria é terminal.
    assert!(
        !s.links
            .iter()
            .any(|l| l.target.ends_with(&id("Moradia › Internet")))
    );
    assert!(!s.links.iter().any(|l| l.source == "expense:none"));
    assert!(!s.nodes.iter().any(|n| n.id == "deficit"));
    assert_eq!((s.total_income, s.total_expense), (1_020_000, 481_000));
    let hub = s.nodes.iter().find(|n| n.id == "hub").unwrap();
    assert_eq!(
        (hub.name.as_str(), hub.kind),
        ("Orçamento", SankeyNodeKind::Hub)
    );
    // Ids estáveis entre chamadas.
    let again = sankey(c, "2026-10-01", "2026-10-31", Basis::Accrual).unwrap();
    let ids = |s: &SankeyData| s.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(&s), ids(&again));
}

#[test]
fn sankey_with_deficit_and_direct_spending() {
    let (t, _) = reference();
    let c = t.db.conn();
    // Novembro por caixa: só as compras do cartão que vencem em 05/11, sem receitas.
    let s = sankey(c, "2026-11-01", "2026-11-30", Basis::Cash).unwrap();
    assert_sankey_balanced(&s);
    let id = |p: &str| cat(c, p);
    assert_eq!(link(&s, "deficit", "hub"), Some(62_000));
    assert_eq!(
        link(&s, "hub", &format!("expense:{}", id("Lazer"))),
        Some(50_000)
    );
    assert_eq!(
        link(&s, "hub", &format!("expense:{}", id("Transporte"))),
        Some(12_000)
    );
    assert_eq!(
        link(
            &s,
            &format!("expense:{}", id("Transporte")),
            &format!("sub:{}", id("Transporte › Combustível"))
        ),
        Some(12_000)
    );
    assert!(!s.nodes.iter().any(|n| n.id == "savings"));

    // Gasto lançado direto na categoria de 1º nível que também tem subcategorias.
    let mut t = new_db();
    let acc = write(&mut t.db, |tx| {
        let acc = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
        let lazer = cat(tx, "Lazer");
        let viagens = cat(tx, "Lazer › Viagens");
        expense(tx, &acc, "2026-03-02", -10_000, Some(&lazer), None, "lazer");
        expense(
            tx,
            &acc,
            "2026-03-03",
            -20_000,
            Some(&viagens),
            None,
            "viagem",
        );
        // Estorno maior que a compra: grupo negativo fica fora do diagrama.
        let cinema = cat(tx, "Lazer › Cinema e shows");
        expense(
            tx,
            &acc,
            "2026-03-04",
            -1_000,
            Some(&cinema),
            None,
            "cinema",
        );
        expense(
            tx,
            &acc,
            "2026-03-05",
            3_000,
            Some(&cinema),
            None,
            "estorno",
        );
        fixture::txn(
            tx,
            &acc,
            "2026-03-06",
            100_000,
            crate::transactions::TxnKind::Income,
            None,
            None,
            "salário",
        );
        Ok(acc)
    });
    let _ = acc;
    let c = t.db.conn();
    let s = sankey(c, "2026-03-01", "2026-03-31", Basis::Accrual).unwrap();
    assert_sankey_balanced(&s);
    let lazer = cat(c, "Lazer");
    assert_eq!(link(&s, "income:none", "hub"), Some(100_000));
    assert_eq!(link(&s, "hub", &format!("expense:{lazer}")), Some(30_000));
    let direct = s
        .nodes
        .iter()
        .find(|n| n.id == format!("sub:{lazer}"))
        .unwrap();
    assert_eq!(direct.name, "Lazer (sem subcategoria)");
    assert_eq!(
        link(&s, &format!("expense:{lazer}"), &format!("sub:{lazer}")),
        Some(10_000)
    );
    assert_eq!(link(&s, "hub", "savings"), Some(70_000));
    // O total de despesas continua o real (300 − 20 de estorno líquido do cinema).
    assert_eq!(s.total_expense, 28_000);
}

#[test]
fn previous_period_keeps_the_length() {
    let p = |a: &str, b: &str| previous_period(a, b).unwrap();
    let s = |a: &str, b: &str| (a.to_string(), b.to_string());
    // Mês, trimestre e ano inteiros recuam em meses.
    assert_eq!(p("2026-10-01", "2026-10-31"), s("2026-09-01", "2026-09-30"));
    assert_eq!(p("2026-03-01", "2026-03-31"), s("2026-02-01", "2026-02-28"));
    assert_eq!(p("2026-07-01", "2026-09-30"), s("2026-04-01", "2026-06-30"));
    assert_eq!(p("2026-01-01", "2026-12-31"), s("2025-01-01", "2025-12-31"));
    // Os demais recuam o mesmo número de dias.
    assert_eq!(p("2026-10-01", "2026-10-14"), s("2026-09-17", "2026-09-30"));
    assert_eq!(p("2026-01-15", "2026-02-14"), s("2025-12-15", "2026-01-14"));
    assert!(previous_period("2026-10-31", "2026-10-01").is_err());
}

#[test]
fn sankey_compares_with_the_previous_period() {
    let (t, _) = reference();
    let c = t.db.conn();
    let s = sankey(c, "2026-10-01", "2026-10-31", Basis::Accrual).unwrap();
    let prev = s.previous.as_ref().unwrap();
    assert_eq!(
        (prev.from.as_str(), prev.to.as_str()),
        ("2026-09-01", "2026-09-30")
    );
    assert_eq!((prev.income, prev.expense), (1_000_000, 401_234));
    assert_eq!(prev.leftover, 598_766);
    assert_eq!(
        (s.totals.income, s.totals.expense, s.totals.leftover),
        (1_020_000, 481_000, 539_000)
    );
    assert_eq!(s.totals.savings_rate, Some(539_000.0 / 1_020_000.0));
    let row = |id: &str| s.comparison.iter().find(|r| r.id == id).unwrap();
    let moradia = row(&format!("expense:{}", cat(c, "Moradia")));
    assert_eq!(
        (moradia.current, moradia.previous, moradia.delta),
        (325_000, 300_000, 25_000)
    );
    assert_eq!(moradia.delta_pct, Some(25_000.0 / 300_000.0));
    assert_eq!(moradia.share_of_income, Some(325_000.0 / 1_020_000.0));
    let savings = row("savings");
    assert_eq!((savings.current, savings.delta), (539_000, -59_766));
    // Pets não tinha gasto em setembro: sem variação percentual.
    let pets = row(&format!("expense:{}", cat(c, "Pets")));
    assert_eq!((pets.previous, pets.delta_pct), (0, None));
    // Subcategoria aponta para a categoria de 1º nível.
    let aluguel = row(&format!("sub:{}", cat(c, "Moradia › Aluguel")));
    assert_eq!(
        aluguel.parent_id.as_deref(),
        Some(format!("expense:{}", cat(c, "Moradia")).as_str())
    );
    // O que só existiu em setembro vem no fim, com o atual zerado.
    let transporte = row(&format!("expense:{}", cat(c, "Transporte")));
    assert_eq!((transporte.current, transporte.previous), (0, 12_000));
    let last_current = s.comparison.iter().rposition(|r| r.current > 0).unwrap();
    let first_previous_only = s.comparison.iter().position(|r| r.current == 0).unwrap();
    assert!(last_current < first_previous_only);
    assert!(!s.comparison.iter().any(|r| r.id == "hub"));

    // Sem comparação.
    let solo = sankey_with(c, "2026-10-01", "2026-10-31", Basis::Accrual, 2, false).unwrap();
    assert!(solo.previous.is_none() && solo.comparison.is_empty());
    assert_eq!(solo.links.len(), s.links.len());
}

#[test]
fn sankey_shows_investments_redemptions_and_debt_payments() {
    use crate::categories::{CategoryInput, CategoryKind, create};
    use crate::transactions::{TxnKind, TxnStatus};
    let mut t = new_db();
    let (fin, cons) = write(&mut t.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
        let broker = fixture::account(tx, "Corretora", "brokerage", "BRL", 0, "2026-01-01");
        let savings = fixture::account(tx, "Poupança", "savings", "BRL", 0, "2026-01-01");
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
        let aluguel = cat(tx, "Moradia › Aluguel");
        let mercado = cat(tx, "Alimentação › Mercado");
        let income = |date: &str, amount: i64| {
            let s = Some(salario.as_str());
            fixture::txn(
                tx,
                &checking,
                date,
                amount,
                TxnKind::Income,
                s,
                None,
                "salário",
            );
        };
        let invest = |from: &str, to: &str, date: &str, amount: i64, status: TxnStatus| {
            fixture::transfer(tx, from, to, date, amount, TxnKind::Investment, status);
        };
        let spend = |date: &str, amount: i64, cat: &str| {
            expense(tx, &checking, date, amount, Some(cat), None, "despesa");
        };
        // Fevereiro.
        income("2026-02-05", 800_000);
        spend("2026-02-10", -300_000, &aluguel);
        invest(
            &checking,
            &broker,
            "2026-02-06",
            100_000,
            TxnStatus::Cleared,
        );
        // Março.
        income("2026-03-05", 1_000_000);
        spend("2026-03-08", -150_000, &mercado);
        spend("2026-03-10", -200_000, &fin);
        spend("2026-03-11", -50_000, &cons);
        invest(
            &checking,
            &broker,
            "2026-03-06",
            300_000,
            TxnStatus::Cleared,
        );
        invest(&broker, &checking, "2026-03-20", 40_000, TxnStatus::Cleared);
        // Entre contas do dia a dia: conta uma vez, como aporte.
        invest(
            &checking,
            &savings,
            "2026-03-21",
            20_000,
            TxnStatus::Cleared,
        );
        // Compra de ativos dentro da corretora, transferência comum e aporte previsto: fora.
        fixture::txn(
            tx,
            &broker,
            "2026-03-22",
            -500_000,
            TxnKind::Investment,
            None,
            None,
            "compra",
        );
        fixture::transfer(
            tx,
            &checking,
            &savings,
            "2026-03-23",
            70_000,
            TxnKind::Transfer,
            TxnStatus::Cleared,
        );
        invest(&checking, &broker, "2026-03-28", 99_000, TxnStatus::Planned);
        // Transferência comum do banco para a corretora: também é aporte.
        fixture::transfer(
            tx,
            &checking,
            &broker,
            "2026-03-24",
            60_000,
            TxnKind::Transfer,
            TxnStatus::Cleared,
        );
        Ok((fin, cons))
    });
    let c = t.db.conn();
    let s = sankey(c, "2026-03-01", "2026-03-31", Basis::Accrual).unwrap();
    assert_sankey_balanced(&s);
    let tt = &s.totals;
    assert_eq!(
        (
            tt.income,
            tt.expense,
            tt.debt_payments,
            tt.investments,
            tt.redemptions,
            tt.leftover
        ),
        (1_000_000, 400_000, 250_000, 380_000, 40_000, 260_000)
    );
    assert_eq!(tt.savings_rate, Some(0.6));
    assert_eq!(link(&s, "redemption", "hub"), Some(40_000));
    assert_eq!(link(&s, "hub", "debt"), Some(250_000));
    assert_eq!(link(&s, "hub", "investment"), Some(380_000));
    assert_eq!(link(&s, "hub", "savings"), Some(260_000));
    assert_eq!(link(&s, "debt", &format!("debt:{fin}")), Some(200_000));
    assert_eq!(link(&s, "debt", &format!("debt:{cons}")), Some(50_000));
    // A dívida sai da categoria em que foi lançada.
    let dividas = cat(c, "Dívidas");
    assert!(!s.links.iter().any(|l| l.target.contains(&dividas)));
    let name = |id: &str| s.nodes.iter().find(|n| n.id == id).unwrap().name.clone();
    assert_eq!(name(&format!("debt:{fin}")), "Apartamento");
    assert_eq!(name("debt"), "Pagamento de dívidas");
    assert_eq!(name("savings"), "Sobra");
    // Fevereiro ao lado.
    let prev = s.previous.as_ref().unwrap();
    assert_eq!(
        (prev.income, prev.expense, prev.investments, prev.leftover),
        (800_000, 300_000, 100_000, 400_000)
    );
    let row = |id: &str| s.comparison.iter().find(|r| r.id == id).unwrap();
    assert_eq!(row("investment").delta, 280_000);
    assert_eq!(row("investment").delta_pct, Some(2.8));
    assert_eq!(row("savings").delta_pct, Some(-0.35));
    assert_eq!(
        row(&format!("debt:{fin}")).parent_id.as_deref(),
        Some("debt")
    );
    let moradia = row(&format!("expense:{}", cat(c, "Moradia")));
    assert_eq!((moradia.current, moradia.previous), (0, 300_000));

    // Profundidade 1: só as categorias de 1º nível, sem os contratos.
    let flat = sankey_with(c, "2026-03-01", "2026-03-31", Basis::Accrual, 1, true).unwrap();
    assert_sankey_balanced(&flat);
    assert_eq!(flat.depth, 1);
    assert!(
        !flat
            .nodes
            .iter()
            .any(|n| n.kind == SankeyNodeKind::Subcategory)
    );
    assert_eq!(link(&flat, "hub", "debt"), Some(250_000));

    // Aportes maiores que a receita: o déficit fecha o nó central.
    let mut t2 = new_db();
    write(&mut t2.db, |tx| {
        let checking = fixture::account(tx, "Conta", "checking", "BRL", 0, "2026-01-01");
        let broker = fixture::account(tx, "Corretora", "brokerage", "BRL", 0, "2026-01-01");
        fixture::txn(
            tx,
            &checking,
            "2026-03-05",
            100_000,
            TxnKind::Income,
            None,
            None,
            "renda",
        );
        fixture::transfer(
            tx,
            &checking,
            &broker,
            "2026-03-06",
            130_000,
            TxnKind::Investment,
            TxnStatus::Cleared,
        );
        Ok(())
    });
    let s = sankey(t2.db.conn(), "2026-03-01", "2026-03-31", Basis::Accrual).unwrap();
    assert_sankey_balanced(&s);
    assert_eq!(link(&s, "deficit", "hub"), Some(30_000));
    assert_eq!(s.totals.leftover, -30_000);
    assert_eq!(s.totals.savings_rate, Some(1.0));
}

#[test]
fn calendar_has_every_day() {
    let (t, _) = reference();
    let c = t.db.conn();
    let cal = calendar(c, "2026-10-01", "2026-10-31", Basis::Accrual).unwrap();
    assert_eq!(cal.days.len(), 31);
    let day = |d: &str| cal.days.iter().find(|x| x.date == d).unwrap();
    assert_eq!((day("2026-10-01").expense, day("2026-10-01").count), (0, 0));
    assert_eq!(
        (day("2026-10-05").income, day("2026-10-05").count),
        (1_000_000, 1)
    );
    assert_eq!(
        (day("2026-10-08").expense, day("2026-10-08").count),
        (60_000, 1)
    );
    assert_eq!(
        (day("2026-10-20").expense, day("2026-10-20").count),
        (70_000, 2)
    );
    assert_eq!(day("2026-10-22").expense, -3_000);
    assert_eq!(day("2026-10-28").expense, 0, "previsto não é gasto");
    assert_eq!(cal.max_expense, 300_000);
    assert_eq!((cal.total_expense, cal.total_income), (481_000, 1_020_000));
    assert_eq!(cal.days.first().unwrap().date, "2026-10-01");
    assert_eq!(cal.days.last().unwrap().date, "2026-10-31");
    // Por caixa, a passagem sai de 20/10.
    let cash = calendar(c, "2026-10-01", "2026-10-31", Basis::Cash).unwrap();
    assert_eq!(cash.days[19].expense, 20_000);
    assert_eq!(cash.days[4].expense, 8_000, "streaming vence em 05/10");
    // Dias vazios num período sem lançamentos.
    let empty = calendar(c, "2025-02-01", "2025-02-28", Basis::Accrual).unwrap();
    assert_eq!(empty.days.len(), 28);
    assert!(empty.days.iter().all(|d| d.expense == 0 && d.count == 0));
    assert_eq!(empty.max_expense, 0);
}

#[test]
fn monthly_evolution_top_n_and_moving_average() {
    let (t, _) = reference();
    let c = t.db.conn();
    let e = monthly_evolution(c, 3, "2026-10", 2, Basis::Accrual).unwrap();
    assert_eq!(e.months, vec!["2026-08", "2026-09", "2026-10"]);
    let series: Vec<(&str, Vec<i64>)> = e
        .series
        .iter()
        .map(|s| (s.label.as_str(), s.values.clone()))
        .collect();
    assert_eq!(
        series,
        vec![
            ("Moradia", vec![290_000, 300_000, 325_000]),
            ("Alimentação", vec![0, 81_234, 62_000]),
            ("Outras", vec![0, 20_000, 94_000]),
        ]
    );
    assert_eq!(e.series[2].key.as_deref(), Some("others"));
    assert_eq!(e.totals, vec![290_000, 401_234, 481_000]);
    // Média móvel desde o primeiro mês com dados (agosto): 2.900; (2.900 + 4.012,34) ÷ 2;
    // (2.900 + 4.012,34 + 4.810) ÷ 3 = 3.907,446… → 3.907,45.
    assert_eq!(e.moving_avg_3, vec![290_000, 345_617, 390_745]);
    assert_eq!(e.income, vec![0, 1_000_000, 1_020_000]);
    for (i, total) in e.totals.iter().enumerate() {
        assert_eq!(e.series.iter().map(|s| s.values[i]).sum::<i64>(), *total);
    }
    // Com N grande, sem "Outras".
    let e = monthly_evolution(c, 12, "2026-10", 20, Basis::Accrual).unwrap();
    assert_eq!(e.months.len(), 12);
    assert!(e.series.iter().all(|s| s.key.as_deref() != Some("others")));
    assert_eq!(e.series.len(), 7);
}

#[test]
fn compare_months_by_parent_category() {
    let (t, _) = reference();
    let c = t.db.conn();
    let p = |a: &str, b: &str| Period {
        from: a.into(),
        to: b.into(),
    };
    let cmp = compare(
        c,
        &p("2026-09-01", "2026-09-30"),
        &p("2026-10-01", "2026-10-31"),
        Basis::Accrual,
        CategoryKind::Expense,
    )
    .unwrap();
    let rows: Vec<(&str, i64, i64, i64)> = cmp
        .rows
        .iter()
        .map(|r| (r.label.as_str(), r.a, r.b, r.delta))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Lazer", 0, 75_000, 75_000),
            ("Moradia", 300_000, 325_000, 25_000),
            ("Alimentação", 81_234, 62_000, -19_234),
            ("Pets", 0, 15_000, 15_000),
            ("Transporte", 12_000, 0, -12_000),
            ("Assinaturas", 8_000, 0, -8_000),
            ("Sem categoria", 0, 4_000, 4_000),
        ]
    );
    assert_eq!(cmp.rows[0].delta_ratio, None);
    assert!((cmp.rows[1].delta_ratio.unwrap() - 25_000.0 / 300_000.0).abs() < 1e-12);
    assert_eq!(cmp.rows[4].delta_ratio, Some(-1.0));
    assert_eq!((cmp.total_a, cmp.total_b), (401_234, 481_000));
}

#[test]
fn by_member_grid() {
    let (t, r) = reference();
    let c = t.db.conn();
    let m = by_member(c, "2026-10-01", "2026-10-31", Basis::Accrual).unwrap();
    let cols: Vec<(Option<&str>, &str, i64)> = m
        .members
        .iter()
        .map(|g| (g.key.as_deref(), g.label.as_str(), g.amount))
        .collect();
    assert_eq!(
        cols,
        vec![
            (Some(r.carlos.as_str()), "Carlos", 420_000),
            (Some(r.ana.as_str()), "Ana", 32_000),
            (None, "Sem membro", 29_000),
        ]
    );
    assert_eq!(m.total, 481_000);
    let rows: Vec<(&str, Vec<i64>)> = m
        .categories
        .iter()
        .map(|s| (s.label.as_str(), s.values.clone()))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Moradia", vec![300_000, 0, 25_000]),
            ("Lazer", vec![75_000, 0, 0]),
            ("Alimentação", vec![45_000, 17_000, 0]),
            ("Pets", vec![0, 15_000, 0]),
            ("Sem categoria", vec![0, 0, 4_000]),
        ]
    );
}

#[test]
fn net_worth_assets_liabilities_and_fx() {
    let (t, r) = reference();
    let c = t.db.conn();
    let nw = net_worth(c, "2026-10-31").unwrap();
    let bal: HashMap<&str, (i64, Option<i64>, bool)> = nw
        .accounts
        .iter()
        .map(|a| {
            (
                a.account_id.as_str(),
                (a.balance, a.balance_base, a.fx_estimated),
            )
        })
        .collect();
    assert_eq!(bal[r.checking.as_str()], (934_766, Some(934_766), false));
    assert_eq!(bal[r.savings.as_str()], (100_000, Some(100_000), false));
    assert_eq!(bal[r.card.as_str()], (-62_000, Some(-62_000), false));
    // USD 150,00 × 5,00, taxa de 15/10 (16 dias antes): estimada.
    assert_eq!(bal[r.usd.as_str()], (15_000, Some(75_000), true));
    assert_eq!(bal[r.jpy.as_str()], (9_000, None, false));
    assert_eq!(nw.assets, 934_766 + 100_000 + 75_000);
    assert_eq!(nw.liabilities, 62_000);
    assert_eq!(nw.net, 1_047_766);
    assert_eq!(nw.missing_fx, vec!["JPY".to_string()]);
    assert!(nw.estimated);
    // Igual a accounts::balance.
    assert_eq!(
        crate::accounts::balance(c, &r.checking, "2026-10-31", false).unwrap(),
        934_766
    );

    let nw = net_worth(c, "2026-10-20").unwrap();
    assert_eq!(nw.net, 935_766 + 100_000 + 75_000 - 62_000);
    assert!(!nw.estimated);

    // Conta arquivada: só entra com saldo ≠ 0; conta fora do patrimônio nunca entra.
    let mut t2 = new_db();
    write(&mut t2.db, |tx| {
        let a = fixture::account(tx, "Antiga", "checking", "BRL", 0, "2026-01-01");
        let b = fixture::account(
            tx,
            "Antiga com saldo",
            "checking",
            "BRL",
            5_000,
            "2026-01-01",
        );
        let x = fixture::account(tx, "Fora", "savings", "BRL", 7_000, "2026-01-01");
        tx.execute(
            "UPDATE account SET archived = 1 WHERE id IN (?1, ?2)",
            [&a, &b],
        )?;
        tx.execute(
            "UPDATE account SET include_in_net_worth = 0 WHERE id = ?1",
            [&x],
        )?;
        Ok(())
    });
    let nw = net_worth(t2.db.conn(), "2026-10-31").unwrap();
    let names: Vec<&str> = nw.accounts.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["Antiga com saldo"]);
    assert_eq!(nw.net, 5_000);
}

#[test]
fn net_worth_history_month_ends() {
    let (t, _) = reference();
    let c = t.db.conn();
    let h = net_worth_history(c, 12, "2026-10").unwrap();
    // Contas abertas em 01/01/2026: novembro e dezembro de 2025 ficam de fora.
    assert_eq!(h.len(), 10);
    assert_eq!(h[0].month, "2026-01");
    assert_eq!(h[0].as_of, "2026-01-31");
    // Sem câmbio de USD antes de 15/10, e nunca de JPY.
    assert_eq!(
        (h[0].assets, h[0].liabilities, h[0].net),
        (100_000, 0, 100_000)
    );
    assert_eq!(h[0].missing_fx, vec!["JPY".to_string(), "USD".to_string()]);
    let aug = &h[7];
    assert_eq!(
        (aug.month.as_str(), aug.net, aug.liabilities),
        ("2026-08", -190_000, 190_000)
    );
    let sep = &h[8];
    assert_eq!(
        (sep.assets, sep.liabilities, sep.net),
        (428_766, 20_000, 408_766)
    );
    let oct = &h[9];
    assert_eq!(oct.net, 1_047_766);
    assert!(oct.estimated);
    assert_eq!(oct.net, net_worth(c, "2026-10-31").unwrap().net);
}

#[test]
fn upcoming_planned_and_card_due() {
    let (t, r) = reference();
    let c = t.db.conn();
    let items = upcoming(c, parse_date("2026-10-14").unwrap(), 30).unwrap();
    let got: Vec<(&str, UpcomingKind, i64, &str)> = items
        .iter()
        .map(|i| (i.date.as_str(), i.kind, i.amount, i.ref_id.as_str()))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                "2026-10-28",
                UpcomingKind::PlannedTxn,
                -10_000,
                r.internet_planned.as_str()
            ),
            (
                "2026-10-30",
                UpcomingKind::PlannedTxn,
                -50_000,
                r.planned_transfer_out.as_str()
            ),
            (
                "2026-11-05",
                UpcomingKind::CardDue,
                -62_000,
                r.nov_statement.as_str()
            ),
        ]
    );
    assert_eq!(items[0].description, "Internet");
    assert_eq!(items[2].description, "Fatura Cartão (nov/2026)");
    assert_eq!(items[2].account_name, "Cartão");
    let short = upcoming(c, parse_date("2026-10-14").unwrap(), 15).unwrap();
    assert_eq!(short.len(), 1);
    // A fatura de outubro está paga.
    let past = upcoming(c, parse_date("2026-10-01").unwrap(), 10).unwrap();
    assert!(past.iter().all(|i| i.kind != UpcomingKind::CardDue));
}

fn sum_sources(e: &Explanation) -> i64 {
    e.sources.iter().map(|s| s.amount).sum()
}

#[test]
fn explain_values_match_sources() {
    let (t, r) = reference();
    let c = t.db.conn();
    let ex = |metric, params: serde_json::Value, limit| {
        explain(
            c,
            &ExplainArgs {
                metric,
                params,
                limit,
            },
        )
        .unwrap()
    };
    let p = serde_json::json!({"from": "2026-10-01", "to": "2026-10-31", "basis": "accrual"});
    let e = ex(Metric::Spending, p.clone(), None);
    assert_eq!(e.value, 481_000);
    assert_eq!(
        e.value,
        spending(c, &args("2026-10-01", "2026-10-31", GroupBy::Category))
            .unwrap()
            .total
    );
    assert_eq!(sum_sources(&e), e.value);
    assert_eq!(e.count, 8);
    assert!(!e.truncated);
    assert_eq!(e.missing_fx, vec!["JPY".to_string()]);
    assert!(e.formula.starts_with(
        "8 lançamentos de despesa efetivados de 01/10/2026 a 31/10/2026 (competência)"
    ));
    assert!(e.formula.contains("R$ 4.810,00"));
    let fx = e
        .sources
        .iter()
        .find(|s| s.kind == SourceKind::FxRate)
        .unwrap();
    assert_eq!((fx.amount, fx.date.as_deref()), (0, Some("2026-10-15")));
    assert!(fx.label.contains("1 USD = 5 BRL"));
    let jpy = e.sources.iter().find(|s| s.label.contains("JPY")).unwrap();
    assert_eq!(jpy.amount, 0);
    assert!(jpy.label.contains("sem câmbio"));
    let usd = e
        .sources
        .iter()
        .find(|s| s.label.contains("USD 50,00"))
        .unwrap();
    assert_eq!(usd.amount, 25_000);
    let split = e
        .sources
        .iter()
        .find(|s| s.id.as_deref() == Some(r.split_txn.as_str()))
        .unwrap();
    assert_eq!(split.amount, 60_000);
    assert!(split.label.contains("2 partes da divisão"));
    let refund = e
        .sources
        .iter()
        .find(|s| s.label.starts_with("Estorno"))
        .unwrap();
    assert_eq!(refund.amount, -3_000);

    // Por categoria (com subcategorias) e por titular.
    let mut p2 = p.clone();
    p2["category_id"] = cat(c, "Alimentação").into();
    let e = ex(Metric::Spending, p2, None);
    assert_eq!((e.value, sum_sources(&e), e.count), (62_000, 62_000, 3));
    assert!(e.formula.contains("em Alimentação e subcategorias"));
    let split = e
        .sources
        .iter()
        .find(|s| s.id.as_deref() == Some(r.split_txn.as_str()))
        .unwrap();
    assert_eq!(split.amount, 45_000);
    let mut p3 = p.clone();
    p3["member_id"] = r.ana.clone().into();
    let e = ex(Metric::Spending, p3, None);
    assert_eq!((e.value, sum_sources(&e)), (32_000, 32_000));
    assert!(e.formula.contains("de Ana"));

    // Caixa: a compra de outubro no cartão vence em novembro.
    let e = ex(
        Metric::Spending,
        serde_json::json!({"from": "2026-11-01", "to": "2026-11-30", "basis": "cash"}),
        None,
    );
    assert_eq!((e.value, sum_sources(&e)), (62_000, 62_000));
    assert!(
        e.sources[0]
            .label
            .contains("fatura com vencimento em 05/11/2026")
    );

    let e = ex(Metric::Income, p.clone(), None);
    assert_eq!(
        (e.value, sum_sources(&e), e.count),
        (1_020_000, 1_020_000, 2)
    );

    let e = ex(
        Metric::SavingsRate,
        serde_json::json!({"month": "2026-10", "basis": "accrual"}),
        None,
    );
    assert_eq!(e.value, 539_000);
    assert_eq!(sum_sources(&e), 539_000);
    assert!((e.ratio.unwrap() - 539_000.0 / 1_020_000.0).abs() < 1e-12);
    assert!(e.formula.contains("= 52,8%"), "{}", e.formula);

    let e = ex(
        Metric::AccountBalance,
        serde_json::json!({"account_id": r.checking, "as_of": "2026-10-31"}),
        None,
    );
    assert_eq!(e.value, 934_766);
    assert_eq!(
        e.value,
        crate::accounts::balance(c, &r.checking, "2026-10-31", false).unwrap()
    );
    assert_eq!(sum_sources(&e), e.value);
    assert_eq!(e.count, 14);
    assert_eq!(e.sources[0].kind, SourceKind::Opening);
    assert!(
        e.formula
            .starts_with("saldo inicial de R$ 1.000,00 + 14 lançamentos efetivados até 31/10/2026"),
        "{}",
        e.formula
    );
    let e = ex(
        Metric::AccountBalance,
        serde_json::json!({"account_id": r.checking, "as_of": "2026-10-31"}),
        Some(5),
    );
    assert!(e.truncated);
    assert_eq!(e.sources.len(), 6);
    assert_eq!(e.value, 934_766);
    // Antes da abertura o saldo inicial não conta.
    let e = ex(
        Metric::AccountBalance,
        serde_json::json!({"account_id": r.checking, "as_of": "2025-12-31"}),
        None,
    );
    assert_eq!((e.value, e.sources.len()), (0, 0));
    assert!(e.formula.contains("a conta foi aberta em 01/01/2026"));

    let e = ex(
        Metric::NetWorth,
        serde_json::json!({"as_of": "2026-10-20"}),
        None,
    );
    assert_eq!(e.value, net_worth(c, "2026-10-20").unwrap().net);
    assert_eq!(sum_sources(&e), e.value);
    assert_eq!(e.count, 5);
    assert!(e.sources.iter().any(|s| s.kind == SourceKind::FxRate));
    assert!(e.formula.contains("sem câmbio para JPY"));

    assert!(
        explain(
            c,
            &ExplainArgs {
                metric: Metric::AccountBalance,
                params: serde_json::json!({"account_id": "nao-existe"}),
                limit: None
            }
        )
        .is_err()
    );
}

#[test]
fn dashboard_reference_month() {
    let (mut t, _) = reference();
    crate::settings::set(t.db.conn(), "budget_basis", &Basis::Accrual).unwrap();
    let d = dashboard(t.db.conn(), Some("2026-10")).unwrap();
    assert_eq!((d.flow.income, d.flow.expense), (1_020_000, 481_000));
    assert_eq!(d.flow_history.len(), 12);
    assert_eq!(d.flow_history[0].month, "2025-11");
    assert_eq!(d.flow_history[11].month, "2026-10");
    // 12 meses: receitas 20.200, despesas 2.900 + 4.012,34 + 4.810.
    let expected = (2_020_000 - 1_172_234) as f64 / 2_020_000.0;
    assert!((d.savings_rate_12m.unwrap() - expected).abs() < 1e-12);
    let top: Vec<&str> = d.top_categories.iter().map(|g| g.label.as_str()).collect();
    assert_eq!(
        top,
        vec!["Moradia", "Lazer", "Alimentação", "Pets", "Sem categoria"]
    );
    assert_eq!(d.uncategorized_count, 1);
    assert_eq!(d.budget_planned, None);
    assert_eq!(d.budget_actual, 481_000);

    let lines = fixture::reference_budget_lines(t.db.conn());
    write(&mut t.db, |tx| {
        crate::budget::set_lines(
            tx,
            &crate::budget::SetBudgetArgs {
                month: "2026-10".into(),
                lines,
            },
        )
    });
    let d = dashboard(t.db.conn(), Some("2026-10")).unwrap();
    assert_eq!(d.budget_planned, Some(505_000));
    // Caixa pelo ajuste.
    crate::settings::set(t.db.conn(), "budget_basis", &Basis::Cash).unwrap();
    let d = dashboard(t.db.conn(), Some("2026-10")).unwrap();
    assert_eq!(d.basis, Basis::Cash);
    assert_eq!(d.flow.expense, 439_000);
}

/// Fórmulas, fontes, rótulos e erros saem no idioma da instalação.
#[test]
fn texts_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let (t, r) = reference();
    let c = t.db.conn();
    set_current(Locale::En);
    let ex = |metric, params: serde_json::Value| {
        explain(
            c,
            &ExplainArgs {
                metric,
                params,
                limit: None,
            },
        )
        .unwrap()
    };
    let spending = ex(
        Metric::Spending,
        serde_json::json!({"from": "2026-10-01", "to": "2026-10-31", "basis": "accrual"}),
    );
    let savings = ex(
        Metric::SavingsRate,
        serde_json::json!({"month": "2026-10", "basis": "accrual"}),
    );
    let nw = ex(Metric::NetWorth, serde_json::json!({"as_of": "2026-10-20"}));
    let s = sankey(c, "2026-10-01", "2026-10-31", Basis::Accrual).unwrap();
    let d = dashboard(c, Some("2026-10")).unwrap();
    let err = crate::reports::engine::validate_range("2026-10-02", "2026-10-01")
        .unwrap_err()
        .message();
    set_current(Locale::PtBr);
    assert!(
        spending.formula.starts_with(
            "8 cleared expense transactions from Oct 1, 2026 to Oct 31, 2026 (accrual)"
        ),
        "{}",
        spending.formula
    );
    assert!(
        spending.formula.contains("R$4,810.00"),
        "{}",
        spending.formula
    );
    assert!(spending.formula.contains("no exchange rate for JPY"));
    let split = spending
        .sources
        .iter()
        .find(|s| s.id.as_deref() == Some(r.split_txn.as_str()))
        .unwrap();
    assert!(
        split.label.contains("(2 parts of the split)"),
        "{}",
        split.label
    );
    assert!(savings.formula.contains("= 52.8%"), "{}", savings.formula);
    assert!(
        nw.formula
            .contains("adding up the balance of 5 accounts on Oct 20, 2026"),
        "{}",
        nw.formula
    );
    let name = |id: &str| s.nodes.iter().find(|n| n.id == id).unwrap().name.clone();
    assert_eq!(name("hub"), "Budget");
    assert!(d.top_categories.iter().any(|g| g.label == "Uncategorized"));
    assert_eq!(
        err,
        "the start date (Oct 2, 2026) is after the end date (Oct 1, 2026)"
    );
}
