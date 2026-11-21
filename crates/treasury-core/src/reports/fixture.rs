//! Dados de teste com valores de referência calculados à mão (usados pelos testes de
//! orçamento, metas, relatórios e da API).
//!
//! Cenário de referência (moeda base BRL; cartão fecha dia 25 e vence dia 5):
//!
//! | data       | conta    | valor        | tipo / categoria                         | titular  |
//! |------------|----------|--------------|------------------------------------------|----------|
//! | 2026-08-10 | corrente | −2.900,00    | despesa Moradia › Aluguel                | Carlos  |
//! | 2026-09-05 | corrente | +10.000,00   | receita Salário                          | Carlos  |
//! | 2026-09-10 | corrente | −3.000,00    | despesa Moradia › Aluguel                | Carlos  |
//! | 2026-09-15 | corrente | −812,34      | despesa Alimentação › Mercado            | Ana      |
//! | 2026-09-10 | cartão   | −80,00       | despesa Assinaturas › Streaming (vence 05/10) | Ana |
//! | 2026-09-28 | cartão   | −120,00      | despesa Transporte › Combustível (vence 05/11) | Carlos |
//! | 2026-10-05 | corrente | +10.000,00   | receita Salário                          | Carlos  |
//! | 2026-10-05 | corr→cartão | 80,00     | pagamento da fatura de outubro (não conta) |        |
//! | 2026-10-06 | corr→poup | 1.000,00    | transferência (não conta)                |          |
//! | 2026-10-08 | corrente | −600,00      | dividida: −450 Mercado (herda Carlos), −150 Pets › Ração (Ana) | Carlos |
//! | 2026-10-10 | corrente | −3.000,00    | despesa Moradia › Aluguel                | Carlos  |
//! | 2026-10-12 | corrente | −250,00      | despesa Moradia › Energia                | —        |
//! | 2026-10-15 | corrente | +200,00      | receita Rendimentos › Dividendos         | Ana      |
//! | 2026-10-18 | dólar    | −USD 50,00   | despesa Lazer › Passeios (USD 5,00 em 15/10 = R$ 250,00) | Carlos |
//! | 2026-10-19 | ienes    | −JPY 1.000   | despesa Lazer › Passeios (sem câmbio)    | —        |
//! | 2026-10-20 | corrente | −200,00      | despesa Alimentação › Restaurantes       | Ana      |
//! | 2026-10-20 | cartão   | −500,00      | despesa Lazer › Viagens (vence 05/11)    | Carlos  |
//! | 2026-10-22 | corrente | +30,00       | estorno em Restaurantes                  | Ana      |
//! | 2026-10-25 | corrente | −40,00       | despesa sem categoria                    | —        |
//! | 2026-10-28 | corrente | −100,00      | **previsto** Moradia › Internet          | —        |
//! | 2026-10-30 | corr→poup | 500,00      | **previsto** transferência               |          |
//!
//! Outubro, competência: receitas 10.200,00; despesas 4.810,00 (Moradia 3.250, Lazer 750,
//! Alimentação 620, Pets 150, sem categoria 40); JPY fora do total.
//! Outubro, caixa: despesas 4.390,00 (Viagens sai, Streaming de 80 entra). Novembro, caixa:
//! 620,00 (Viagens 500 + Combustível 120), sem receitas → déficit.

use rusqlite::{Connection, Transaction, params};

use crate::changes::{Author, record};
use crate::error::Result;
use crate::ids::{Id, new_id};
use crate::storage::Db;
use crate::time::now_utc;
use crate::transactions::{NewTxn, SplitInput, TxnKind, TxnStatus, insert, replace_splits};
use crate::vault::KDF_TEST;

pub(crate) struct TestDb {
    pub db: Db,
    _dir: tempfile::TempDir,
}

/// Banco novo com a família "Teste" (Carlos e Ana) e as categorias padrão.
pub(crate) fn new_db() -> TestDb {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    record(
        db.conn_mut(),
        &Author::system("teste"),
        "dados iniciais",
        |tx| {
            crate::household::seed(tx, "Teste", "BRL", &["Carlos".into(), "Ana".into()])?;
            crate::categories::seed_defaults(tx)
        },
    )
    .unwrap();
    TestDb { db, _dir: dir }
}

/// Escreve dados de teste num conjunto de alterações.
pub(crate) fn write<T>(db: &mut Db, f: impl FnOnce(&Transaction<'_>) -> Result<T>) -> T {
    record(db.conn_mut(), &Author::system("teste"), "dados de teste", f)
        .unwrap()
        .value
}

/// Categoria pelo caminho (`Alimentação › Mercado`).
pub(crate) fn cat(conn: &Connection, path: &str) -> Id {
    let tree = super::engine::CategoryTree::load(conn).unwrap();
    tree.by_id
        .values()
        .find(|c| c.path == path)
        .unwrap_or_else(|| panic!("categoria {path}"))
        .id
        .clone()
}

pub(crate) fn member(conn: &Connection, name: &str) -> Id {
    conn.query_row("SELECT id FROM member WHERE name = ?1", [name], |r| {
        r.get(0)
    })
    .unwrap()
}

pub(crate) fn account(
    tx: &Transaction<'_>,
    name: &str,
    kind: &str,
    currency: &str,
    opening: i64,
    opening_date: &str,
) -> Id {
    let id = new_id();
    tx.execute(
        "INSERT INTO account (id, name, kind, currency, opening_balance, opening_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![id, name, kind, currency, opening, opening_date, now_utc()],
    )
    .unwrap();
    id
}

pub(crate) fn card(tx: &Transaction<'_>, name: &str, closing_day: u32, due_day: u32) -> Id {
    let id = account(tx, name, "credit_card", "BRL", 0, "2026-01-01");
    tx.execute(
        "INSERT INTO credit_card (account_id, closing_day, due_day, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, closing_day, due_day, now_utc()],
    )
    .unwrap();
    id
}

pub(crate) fn fx_rate(tx: &Transaction<'_>, base: &str, quote: &str, date: &str, rate: &str) {
    crate::fx::set_manual(
        tx,
        &crate::fx::FxRateInput {
            base: base.into(),
            quote: quote.into(),
            date: date.into(),
            rate: rate.into(),
        },
    )
    .unwrap();
}

/// Lançamento efetivado na moeda da conta.
pub(crate) fn new_txn(
    conn: &Connection,
    account: &str,
    date: &str,
    amount: i64,
    kind: TxnKind,
    category: Option<&str>,
    member: Option<&str>,
) -> NewTxn {
    let currency: String = conn
        .query_row(
            "SELECT currency FROM account WHERE id = ?1",
            [account],
            |r| r.get(0),
        )
        .unwrap();
    let mut t = NewTxn::new(account, date, amount, &currency, kind, "lançamento");
    t.category_id = category.map(str::to_string);
    t.member_id = member.map(str::to_string);
    t
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn txn(
    tx: &Transaction<'_>,
    account: &str,
    date: &str,
    amount: i64,
    kind: TxnKind,
    category: Option<&str>,
    member: Option<&str>,
    description: &str,
) -> Id {
    let mut t = new_txn(tx, account, date, amount, kind, category, member);
    t.description = description.into();
    insert(tx, &t).unwrap()
}

pub(crate) fn expense(
    tx: &Transaction<'_>,
    account: &str,
    date: &str,
    amount: i64,
    category: Option<&str>,
    member: Option<&str>,
    description: &str,
) -> Id {
    txn(
        tx,
        account,
        date,
        amount,
        TxnKind::Expense,
        category,
        member,
        description,
    )
}

/// Transferência entre contas (duas pernas com o mesmo `transfer_id`).
pub(crate) fn transfer(
    tx: &Transaction<'_>,
    from: &str,
    to: &str,
    date: &str,
    amount: i64,
    kind: TxnKind,
    status: TxnStatus,
) -> (Id, Id) {
    let tid = new_id();
    let mut a = new_txn(tx, from, date, -amount, kind, None, None);
    a.transfer_id = Some(tid.clone());
    a.status = status;
    a.description = "transferência".into();
    let mut b = new_txn(tx, to, date, amount, kind, None, None);
    b.transfer_id = Some(tid);
    b.status = status;
    b.description = "transferência".into();
    (insert(tx, &a).unwrap(), insert(tx, &b).unwrap())
}

pub(crate) struct Reference {
    pub checking: Id,
    pub savings: Id,
    pub card: Id,
    pub usd: Id,
    pub jpy: Id,
    pub carlos: Id,
    pub ana: Id,
    pub split_txn: Id,
    pub internet_planned: Id,
    pub planned_transfer_out: Id,
    /// Fatura que vence em 05/11/2026.
    pub nov_statement: Id,
}

/// Grava o cenário de referência (ver o comentário do módulo).
pub(crate) fn seed_reference(db: &mut Db) -> Reference {
    write(db, |tx| {
        let c = |p: &str| cat(tx, p);
        let carlos = member(tx, "Carlos");
        let ana = member(tx, "Ana");
        let g = Some(carlos.as_str());
        let a = Some(ana.as_str());
        let checking = account(
            tx,
            "Conta Corrente",
            "checking",
            "BRL",
            100_000,
            "2026-01-01",
        );
        let savings = account(tx, "Poupança", "savings", "BRL", 0, "2026-01-01");
        let card = card(tx, "Cartão", 25, 5);
        let usd = account(tx, "Conta em dólar", "foreign", "USD", 20_000, "2026-01-01");
        let jpy = account(tx, "Ienes", "foreign", "JPY", 10_000, "2026-01-01");
        fx_rate(tx, "USD", "BRL", "2026-10-15", "5.00");

        let aluguel = c("Moradia › Aluguel");
        let mercado = c("Alimentação › Mercado");
        let restaurantes = c("Alimentação › Restaurantes");
        let passeios = c("Lazer › Passeios");

        // Agosto e setembro.
        expense(
            tx,
            &checking,
            "2026-08-10",
            -290_000,
            Some(&aluguel),
            g,
            "Aluguel agosto",
        );
        txn(
            tx,
            &checking,
            "2026-09-05",
            1_000_000,
            TxnKind::Income,
            Some(&c("Salário")),
            g,
            "Salário setembro",
        );
        expense(
            tx,
            &checking,
            "2026-09-10",
            -300_000,
            Some(&aluguel),
            g,
            "Aluguel setembro",
        );
        expense(
            tx,
            &checking,
            "2026-09-15",
            -81_234,
            Some(&mercado),
            a,
            "Mercado setembro",
        );
        expense(
            tx,
            &card,
            "2026-09-10",
            -8_000,
            Some(&c("Assinaturas › Streaming")),
            a,
            "Streaming",
        );
        expense(
            tx,
            &card,
            "2026-09-28",
            -12_000,
            Some(&c("Transporte › Combustível")),
            g,
            "Posto",
        );

        // Outubro.
        txn(
            tx,
            &checking,
            "2026-10-05",
            1_000_000,
            TxnKind::Income,
            Some(&c("Salário")),
            g,
            "Salário outubro",
        );
        // Pagamento da fatura de outubro (a perna do cartão fica ligada à fatura paga).
        let oct_statement: Id = tx
            .query_row(
                "SELECT id FROM card_statement WHERE account_id = ?1 AND reference_month = '2026-10'",
                [&card],
                |r| r.get(0),
            )
            .unwrap();
        let tid = new_id();
        let mut out = new_txn(
            tx,
            &checking,
            "2026-10-05",
            -8_000,
            TxnKind::CardPayment,
            None,
            None,
        );
        out.transfer_id = Some(tid.clone());
        insert(tx, &out).unwrap();
        let mut inn = new_txn(
            tx,
            &card,
            "2026-10-05",
            8_000,
            TxnKind::CardPayment,
            None,
            None,
        );
        inn.transfer_id = Some(tid);
        inn.statement_id = Some(oct_statement.clone());
        insert(tx, &inn).unwrap();
        tx.execute(
            "UPDATE card_statement SET status = 'paid' WHERE id = ?1",
            [&oct_statement],
        )
        .unwrap();

        transfer(
            tx,
            &checking,
            &savings,
            "2026-10-06",
            100_000,
            TxnKind::Transfer,
            TxnStatus::Cleared,
        );
        let mut split = new_txn(
            tx,
            &checking,
            "2026-10-08",
            -60_000,
            TxnKind::Expense,
            None,
            g,
        );
        split.description = "Supermercado com ração".into();
        let split_txn = insert(tx, &split).unwrap();
        replace_splits(
            tx,
            &split_txn,
            -60_000,
            &[
                SplitInput {
                    category_id: Some(mercado.clone()),
                    member_id: None,
                    amount: -45_000,
                    notes: None,
                },
                SplitInput {
                    category_id: Some(c("Pets › Ração")),
                    member_id: Some(ana.clone()),
                    amount: -15_000,
                    notes: None,
                },
            ],
        )?;
        expense(
            tx,
            &checking,
            "2026-10-10",
            -300_000,
            Some(&aluguel),
            g,
            "Aluguel outubro",
        );
        expense(
            tx,
            &checking,
            "2026-10-12",
            -25_000,
            Some(&c("Moradia › Energia")),
            None,
            "Energia",
        );
        txn(
            tx,
            &checking,
            "2026-10-15",
            20_000,
            TxnKind::Income,
            Some(&c("Rendimentos › Dividendos")),
            a,
            "Dividendos",
        );
        expense(
            tx,
            &usd,
            "2026-10-18",
            -5_000,
            Some(&passeios),
            g,
            "Museu em Nova York",
        );
        expense(
            tx,
            &jpy,
            "2026-10-19",
            -1_000,
            Some(&passeios),
            None,
            "Templo em Kyoto",
        );
        expense(
            tx,
            &checking,
            "2026-10-20",
            -20_000,
            Some(&restaurantes),
            a,
            "Restaurante",
        );
        expense(
            tx,
            &card,
            "2026-10-20",
            -50_000,
            Some(&c("Lazer › Viagens")),
            g,
            "Passagem",
        );
        expense(
            tx,
            &checking,
            "2026-10-22",
            3_000,
            Some(&restaurantes),
            a,
            "Estorno do restaurante",
        );
        expense(
            tx,
            &checking,
            "2026-10-25",
            -4_000,
            None,
            None,
            "Pix sem categoria",
        );

        // Previstos.
        let mut internet = new_txn(
            tx,
            &checking,
            "2026-10-28",
            -10_000,
            TxnKind::Expense,
            Some(&c("Moradia › Internet")),
            None,
        );
        internet.status = TxnStatus::Planned;
        internet.description = "Internet".into();
        let internet_planned = insert(tx, &internet).unwrap();
        let (planned_transfer_out, _) = transfer(
            tx,
            &checking,
            &savings,
            "2026-10-30",
            50_000,
            TxnKind::Transfer,
            TxnStatus::Planned,
        );
        let nov_statement: Id = tx
            .query_row(
                "SELECT id FROM card_statement WHERE account_id = ?1 AND reference_month = '2026-11'",
                [&card],
                |r| r.get(0),
            )
            .unwrap();
        Ok(Reference {
            checking,
            savings,
            card,
            usd,
            jpy,
            carlos,
            ana,
            split_txn,
            internet_planned,
            planned_transfer_out,
            nov_statement,
        })
    })
}

/// Linhas do orçamento de outubro/2026 do cenário: Moradia 3.500, Mercado 500,
/// Restaurantes 150, Lazer 600, Transporte 300.
pub(crate) fn reference_budget_lines(conn: &Connection) -> Vec<crate::budget::BudgetLineInput> {
    [
        ("Moradia", 350_000),
        ("Alimentação › Mercado", 50_000),
        ("Alimentação › Restaurantes", 15_000),
        ("Lazer", 60_000),
        ("Transporte", 30_000),
    ]
    .into_iter()
    .map(|(p, amount)| crate::budget::BudgetLineInput {
        category_id: cat(conn, p),
        amount,
    })
    .collect()
}
