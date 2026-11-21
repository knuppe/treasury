//! Banco temporário com família e categorias iniciais, e atalhos para os testes do
//! livro-caixa (contas, lançamentos, cartões, recorrências).

use chrono::NaiveDate;
use rusqlite::{Connection, Transaction};

use crate::accounts::{AccountInput, AccountKind, CardDetails};
use crate::changes::{self, Author};
use crate::error::Result;
use crate::ids::Id;
use crate::storage::Db;
use crate::transactions::{TxnInput, TxnKind};
use crate::vault::KDF_TEST;

pub fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

pub struct Fx {
    _dir: tempfile::TempDir,
    pub db: Db,
}

impl Fx {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        changes::record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            crate::household::seed(tx, "Família", "BRL", &["Carlos".into(), "Ana".into()])?;
            crate::categories::seed_defaults(tx)
        })
        .unwrap();
        Self { _dir: dir, db }
    }

    pub fn conn(&self) -> &Connection {
        self.db.conn()
    }

    /// Escrita registrada (autor usuário); devolve o valor e o conjunto de alterações.
    pub fn write_cs<T>(
        &mut self,
        f: impl FnOnce(&Transaction<'_>) -> Result<T>,
    ) -> Result<(T, Option<Id>)> {
        let r = changes::record(self.db.conn_mut(), &Author::user(), "teste", f)?;
        Ok((r.value, r.change_set_id))
    }

    pub fn write<T>(&mut self, f: impl FnOnce(&Transaction<'_>) -> Result<T>) -> Result<T> {
        self.write_cs(f).map(|(v, _)| v)
    }

    pub fn undo(&mut self, id: &str) -> Result<()> {
        changes::undo(self.db.conn_mut(), id).map(|_| ())
    }

    /// Categoria pelo nome (único no conjunto inicial).
    pub fn cat(&self, name: &str) -> Id {
        self.conn()
            .query_row("SELECT id FROM category WHERE name = ?1", [name], |r| {
                r.get(0)
            })
            .unwrap()
    }

    pub fn member(&self, name: &str) -> Id {
        self.conn()
            .query_row("SELECT id FROM member WHERE name = ?1", [name], |r| {
                r.get(0)
            })
            .unwrap()
    }

    pub fn account(&mut self, name: &str, kind: AccountKind, currency: &str, opening: i64) -> Id {
        let input = AccountInput {
            name: name.into(),
            kind,
            currency: Some(currency.into()),
            institution_id: None,
            institution_name: None,
            member_ids: None,
            opening_balance: Some(opening),
            opening_date: Some("2026-01-01".into()),
            include_in_net_worth: None,
            color: None,
            notes: None,
            card: None,
            overdraft: None,
            branch: None,
            account_number: None,
            retirement_plan: None,
            retirement_coverage: None,
        };
        self.write(|tx| crate::accounts::create(tx, &input, d("2026-01-01")))
            .unwrap()
    }

    pub fn checking(&mut self, name: &str, opening: i64) -> Id {
        self.account(name, AccountKind::Checking, "BRL", opening)
    }

    pub fn card(&mut self, name: &str, closing_day: u32, due_day: u32, limit: Option<i64>) -> Id {
        let input = AccountInput {
            name: name.into(),
            kind: AccountKind::CreditCard,
            currency: None,
            institution_id: None,
            institution_name: None,
            member_ids: None,
            opening_balance: None,
            opening_date: Some("2026-01-01".into()),
            include_in_net_worth: None,
            color: None,
            notes: None,
            card: Some(CardDetails {
                credit_limit: limit,
                closing_day,
                due_day,
                payment_account_id: None,
                network: None,
                last4: None,
            }),
            overdraft: None,
            branch: None,
            account_number: None,
            retirement_plan: None,
            retirement_coverage: None,
        };
        self.write(|tx| crate::accounts::create(tx, &input, d("2026-01-01")))
            .unwrap()
    }

    pub fn input(account: &str, date: &str, amount: i64, desc: &str) -> TxnInput {
        TxnInput {
            account_id: account.into(),
            date: date.into(),
            amount,
            kind: None,
            status: None,
            description: desc.into(),
            payee: None,
            notes: None,
            category_id: None,
            member_id: None,
            splits: None,
            tags: None,
            project_id: None,
            card_plastic_id: None,
            orig_amount: None,
            orig_currency: None,
        }
    }

    /// Lançamento simples (tipo pelo sinal), efetivado.
    pub fn txn(&mut self, account: &str, date: &str, amount: i64, desc: &str) -> Id {
        let input = Self::input(account, date, amount, desc);
        self.write(|tx| crate::transactions::create(tx, &input, d("2026-10-14")))
            .unwrap()
    }

    pub fn txn_with(&mut self, input: TxnInput) -> Result<Id> {
        self.write(|tx| crate::transactions::create(tx, &input, d("2026-10-14")))
    }

    pub fn count(&self, sql: &str) -> i64 {
        self.conn().query_row(sql, [], |r| r.get(0)).unwrap()
    }

    pub fn kind_of(&self, id: &str) -> TxnKind {
        crate::transactions::get(self.conn(), id).unwrap().kind
    }
}
