//! Ambiente de teste com banco temporário, família, categorias e contas criadas por SQL
//! direto (os comandos de CRUD de contas são de outro módulo).

use rusqlite::params;

use crate::api::{Actor, Ctx, Scratch};
use crate::changes::{self, Author};
use crate::ids::{Id, new_id};
use crate::paths::TreasuryPaths;
use crate::storage::Db;
use crate::transactions::{NewTxn, TxnKind};
use crate::vault::KDF_TEST;

pub struct TestEnv {
    pub _dir: tempfile::TempDir,
    pub db: Db,
    pub paths: TreasuryPaths,
    pub scratch: Scratch,
    pub actor: Actor,
}

impl TestEnv {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
        paths.ensure_dirs().unwrap();
        let mut db = Db::create(&paths.db_path, "senha", KDF_TEST).unwrap();
        changes::record(db.conn_mut(), &Author::system("teste"), "seed", |tx| {
            crate::household::seed(tx, "Família", "BRL", &["Ana".into(), "Bruno".into()])?;
            crate::categories::seed_defaults(tx)
        })
        .unwrap();
        Self {
            _dir: dir,
            db,
            paths,
            scratch: Scratch::default(),
            actor: Actor::User,
        }
    }

    pub fn ctx(&mut self) -> Ctx<'_> {
        Ctx {
            db: &mut self.db,
            paths: &self.paths,
            actor: &self.actor,
            scratch: &mut self.scratch,
            cmd: "test",
        }
    }

    pub fn add_account(&mut self, name: &str, kind: &str) -> Id {
        let id = new_id();
        self.db
            .conn()
            .execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'BRL', '2026-01-01', 'x', 'x')",
                params![id, name, kind],
            )
            .unwrap();
        id
    }

    /// Cartão que fecha no dia `closing` e vence no dia `due`.
    pub fn add_card(&mut self, name: &str, closing: u32, due: u32) -> Id {
        let id = self.add_account(name, "credit_card");
        self.db
            .conn()
            .execute(
                "INSERT INTO credit_card (account_id, credit_limit, closing_day, due_day, created_at, updated_at)
                 VALUES (?1, 500000, ?2, ?3, 'x', 'x')",
                params![id, closing, due],
            )
            .unwrap();
        id
    }

    /// Categoria pelo nome (a primeira, se houver repetição).
    pub fn category(&self, name: &str) -> Id {
        self.db
            .conn()
            .query_row(
                "SELECT id FROM category WHERE name = ?1 ORDER BY parent_id IS NOT NULL LIMIT 1",
                [name],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| panic!("categoria {name}"))
    }

    pub fn member(&self, name: &str) -> Id {
        self.db
            .conn()
            .query_row("SELECT id FROM member WHERE name = ?1", [name], |r| {
                r.get(0)
            })
            .unwrap()
    }

    /// Lançamento gravado como o usuário gravaria (com histórico).
    pub fn txn(
        &mut self,
        account: &str,
        date: &str,
        amount: i64,
        description: &str,
        category: Option<&str>,
    ) -> Id {
        let mut t = NewTxn::new(
            account,
            date,
            amount,
            "BRL",
            if amount < 0 {
                TxnKind::Expense
            } else {
                TxnKind::Income
            },
            description,
        );
        t.category_id = category.map(str::to_string);
        changes::record(self.db.conn_mut(), &Author::user(), "lançamento", |tx| {
            crate::transactions::insert(tx, &t)
        })
        .unwrap()
        .value
    }

    pub fn count(&self, sql: &str) -> i64 {
        self.db.conn().query_row(sql, [], |r| r.get(0)).unwrap()
    }
}
