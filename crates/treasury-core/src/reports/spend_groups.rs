//! Para onde vai a renda: as despesas pelos grupos de gasto das categorias (essenciais,
//! dívidas, futuro, não essencial), mês a mês e no período, com a parte da renda de cada grupo,
//! as maiores categorias de cada um e as metas (`budget::GroupTargets`).
//!
//! Regras (as do fluxo de caixa de [`super::engine`], mais o que é guardado):
//! - cada despesa entra no grupo da categoria em que foi lançada (a da parte, nas divisões);
//!   sem categoria, ou numa categoria sem grupo, em "não essencial". Estornos reduzem o grupo;
//! - o futuro também conta o que foi guardado sem categoria: os aportes (lançamentos do tipo
//!   investimento que saem de uma conta que não é de corretora) e as transferências de uma conta
//!   comum para as contas das metas ativas. Resgates não abatem: o relatório mostra o que foi
//!   guardado no mês;
//! - transferências entre contas comuns, pagamento de fatura e ajustes de saldo ficam fora;
//! - renda = as receitas do mês (as mesmas do fluxo de caixa);
//! - competência ou caixa como nos outros relatórios (aportes e transferências pela data do
//!   lançamento nas duas); com `member_id`, só o que é do membro (docs/members.md);
//! - valores na moeda base, em centavos, positivos (gasto); outras moedas convertidas pelo
//!   câmbio da data de cada lançamento (sem câmbio, ficam fora).

use std::collections::HashMap;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{
    CategoryTree, FlowQuery, Fx, Key, Params, month_dates, months_between, normalize_month, ratio,
    resolve_basis,
};
use super::{member::MemberScope, no_category};
use crate::budget::{Basis, GroupTargets};
use crate::categories::{CategoryKind, SpendGroup};
use crate::error::Result;
use crate::household::base_currency;
use crate::i18n::t;
use crate::ids::Id;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendGroupsArgs {
    /// Primeiro mês `YYYY-MM`.
    pub from: String,
    /// Último mês `YYYY-MM`.
    pub to: String,
    /// Padrão: o ajuste `budget_basis`.
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// Membro da família (docs/members.md): só o que é dele. Padrão: a família.
    #[ts(optional)]
    pub member_id: Option<Id>,
}

/// Uma das maiores categorias de um grupo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendGroupCategory {
    /// `null`: aportes, transferências para metas ou despesas sem categoria (o rótulo diz).
    pub category_id: Option<Id>,
    /// Caminho da categoria (`Alimentação › Mercado`), "Aportes", "Transferências para metas" ou
    /// "Sem categoria".
    pub label: String,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendGroupAmount {
    pub group: SpendGroup,
    /// Centavos na moeda base, positivo = gasto (estornos reduzem).
    #[ts(type = "number")]
    pub amount: i64,
    /// `amount` ÷ receitas (0–1); `null` sem receitas.
    pub share_of_income: Option<f64>,
    /// Até 3 maiores categorias do grupo, da maior para a menor.
    pub top: Vec<SpendGroupCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendGroupMonth {
    /// `YYYY-MM`, ou `total` no total do período.
    pub month: String,
    /// Receitas do mês (centavos).
    #[ts(type = "number")]
    pub income: i64,
    /// Soma dos quatro grupos.
    #[ts(type = "number")]
    pub spending: i64,
    /// Sempre os quatro, nesta ordem: essential, debt, future, non_essential.
    pub groups: Vec<SpendGroupAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendGroupsReport {
    /// Moeda base da família.
    pub currency: String,
    /// Um por mês do período, em ordem.
    pub months: Vec<SpendGroupMonth>,
    /// Soma do período (`month` = `total`; as maiores categorias são as do período).
    pub total: SpendGroupMonth,
    pub targets: GroupTargets,
}

/// De onde veio um valor do grupo.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Item {
    Category(Id),
    Uncategorized,
    /// Aportes em investimentos.
    Contributions,
    /// Transferências para as contas das metas.
    Goals,
}

/// Valores de um mês: renda e, por grupo, o valor de cada origem.
#[derive(Debug, Default, Clone)]
struct Bucket {
    income: i64,
    groups: HashMap<SpendGroup, HashMap<Item, i64>>,
}

impl Bucket {
    fn add(&mut self, g: SpendGroup, item: Item, amount: i64) {
        *self.groups.entry(g).or_default().entry(item).or_default() += amount;
    }

    /// Passa `amount` de um item de um grupo para o mesmo item em outro grupo.
    fn shift(&mut self, from: SpendGroup, to: SpendGroup, item: Item, amount: i64) {
        if let Some(items) = self.groups.get_mut(&from) {
            if let Some(v) = items.get_mut(&item) {
                *v -= amount;
                if *v == 0 {
                    items.remove(&item);
                }
            }
            if items.is_empty() {
                self.groups.remove(&from);
            }
        }
        self.add(to, item, amount);
    }

    fn merge(&mut self, o: &Bucket) {
        self.income += o.income;
        for (g, items) in &o.groups {
            for (item, v) in items {
                self.add(*g, item.clone(), *v);
            }
        }
    }
}

/// Titular de um lançamento sem divisão (aportes e transferências): o do lançamento ou, sem
/// ele, o dono da conta quando ela tem um titular só (como `engine::MEMBER_EXPR`).
const TXN_MEMBER_EXPR: &str = "COALESCE(t.member_id, \
     (SELECT CASE WHEN count(*) = 1 THEN min(am.member_id) END FROM account_member am \
       WHERE am.account_id = t.account_id))";

/// Contas ligadas a metas ativas.
const GOAL_ACCOUNTS: &str = "(SELECT ga.account_id FROM goal_account ga \
     JOIN goal g ON g.id = ga.goal_id WHERE g.archived = 0)";

/// Grupo de uma categoria (a de receita, ou sem grupo, cai em "não essencial").
fn group_of(tree: &CategoryTree, id: &str) -> SpendGroup {
    match tree.get(id) {
        Some(c) if c.kind == CategoryKind::Expense => c.spend_group.unwrap_or(if c.is_essential {
            SpendGroup::Essential
        } else {
            SpendGroup::NonEssential
        }),
        _ => SpendGroup::NonEssential,
    }
}

/// O que foi guardado sem categoria no período, por mês: aportes e transferências para metas.
fn saved(
    conn: &Connection,
    fx: &mut Fx<'_>,
    from: &str,
    to: &str,
    member: Option<&str>,
) -> Result<Vec<(String, Item, i64)>> {
    let mut out = vec![];
    for (item, cond) in [
        (
            Item::Contributions,
            "t.kind = 'investment' AND a.kind NOT IN ('brokerage', 'retirement')".to_string(),
        ),
        (
            Item::Goals,
            format!(
                "t.kind = 'transfer' AND t.transfer_id IS NOT NULL AND a.kind NOT IN ('brokerage', 'retirement')
                 AND t.account_id NOT IN {GOAL_ACCOUNTS}
                 AND EXISTS (SELECT 1 FROM txn o WHERE o.transfer_id = t.transfer_id
                               AND o.id <> t.id AND o.account_id IN {GOAL_ACCOUNTS})"
            ),
        ),
    ] {
        let mut p = Params::default();
        let (a, b) = (p.bind(from.to_string()), p.bind(to.to_string()));
        let by_member = match member {
            Some(m) => format!(" AND {TXN_MEMBER_EXPR} = {}", p.bind(m.to_string())),
            None => String::new(),
        };
        let sql = format!(
            "SELECT t.date, t.currency, t.amount FROM txn t JOIN account a ON a.id = t.account_id
             WHERE {cond} AND t.amount < 0 AND t.status IN ('cleared', 'reconciled')
               AND t.date BETWEEN {a} AND {b}{by_member}
             ORDER BY t.date, t.id"
        );
        let mut stmt = conn.prepare_cached(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(p.0.iter()), |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for (date, currency, amount) in rows {
            if let Some(c) = fx.convert(-amount, &currency, &date)? {
                out.push((date[..7].to_string(), item.clone(), c.amount));
            }
        }
    }
    Ok(out)
}

/// Os quatro grupos de um mês (ou do período), com a parte da renda e as maiores categorias.
fn month_view(month: String, b: &Bucket, tree: &CategoryTree) -> SpendGroupMonth {
    let label = |item: &Item| match item {
        Item::Category(id) => tree
            .get(id)
            .map(|c| c.path.clone())
            .unwrap_or_else(no_category),
        Item::Uncategorized => no_category(),
        Item::Contributions => t("nucleo-relatorios-grupos-aportes"),
        Item::Goals => t("nucleo-relatorios-grupos-metas"),
    };
    let empty = HashMap::new();
    let groups: Vec<SpendGroupAmount> = SpendGroup::ALL
        .into_iter()
        .map(|g| {
            let items = b.groups.get(&g).unwrap_or(&empty);
            let amount: i64 = items.values().sum();
            let mut top: Vec<SpendGroupCategory> = items
                .iter()
                .filter(|(_, v)| **v > 0)
                .map(|(item, v)| SpendGroupCategory {
                    category_id: match item {
                        Item::Category(id) => Some(id.clone()),
                        _ => None,
                    },
                    label: label(item),
                    amount: *v,
                })
                .collect();
            top.sort_by(|x, y| y.amount.cmp(&x.amount).then_with(|| x.label.cmp(&y.label)));
            top.truncate(3);
            SpendGroupAmount {
                group: g,
                amount,
                share_of_income: ratio(amount, b.income),
                top,
            }
        })
        .collect();
    SpendGroupMonth {
        month,
        income: b.income,
        spending: groups.iter().map(|g| g.amount).sum(),
        groups,
    }
}

/// Para onde foi a renda de `from` a `to` (meses `YYYY-MM`), por grupo de gasto.
pub fn spend_groups(conn: &Connection, a: &SpendGroupsArgs) -> Result<SpendGroupsReport> {
    let months = months_between(&normalize_month(&a.from)?, &normalize_month(&a.to)?)?;
    let basis = resolve_basis(conn, a.basis)?;
    let scope = a
        .member_id
        .as_deref()
        .filter(|m| !m.is_empty())
        .map(|m| MemberScope::load(conn, m))
        .transpose()?;
    let (from, _) = month_dates(&months[0])?;
    let (_, to) = month_dates(months.last().expect("ao menos um mês"))?;
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let tree = CategoryTree::load(conn)?;
    let pos: HashMap<&str, usize> = months
        .iter()
        .enumerate()
        .map(|(i, m)| (m.as_str(), i))
        .collect();
    let mut buckets = vec![Bucket::default(); months.len()];

    let mut q = FlowQuery::new(&from, &to, basis);
    q.member_ids = scope.as_ref().map(|s| s.ids.as_slice());
    let agg = q.aggregate(conn, &mut fx, Key::Month, Key::Category)?;
    for ((month, cat), cell) in &agg.cells {
        let Some(&i) = month.as_deref().and_then(|m| pos.get(m)) else {
            continue;
        };
        let b = &mut buckets[i];
        b.income += cell.income;
        if cell.expense == 0 {
            continue;
        }
        match cat.as_deref().filter(|c| tree.get(c).is_some()) {
            Some(c) => b.add(
                group_of(&tree, c),
                Item::Category(c.to_string()),
                cell.expense,
            ),
            None => b.add(SpendGroup::NonEssential, Item::Uncategorized, cell.expense),
        }
    }
    // Parcela de contrato paga é serviço da dívida, qualquer que seja a categoria do lançamento.
    q.loan_linked = Some(true);
    q.kind = Some(crate::categories::CategoryKind::Expense);
    let debt = q.aggregate(conn, &mut fx, Key::Month, Key::Category)?;
    for ((month, cat), cell) in &debt.cells {
        let Some(&i) = month.as_deref().and_then(|m| pos.get(m)) else {
            continue;
        };
        if cell.expense == 0 {
            continue;
        }
        let (group, item) = match cat.as_deref().filter(|c| tree.get(c).is_some()) {
            Some(c) => (group_of(&tree, c), Item::Category(c.to_string())),
            None => (SpendGroup::NonEssential, Item::Uncategorized),
        };
        if group != SpendGroup::Debt {
            buckets[i].shift(group, SpendGroup::Debt, item, cell.expense);
        }
    }
    let member = scope.as_ref().map(|s| s.member_id.as_str());
    for (month, item, amount) in saved(conn, &mut fx, &from, &to, member)? {
        if let Some(&i) = pos.get(month.as_str()) {
            buckets[i].add(SpendGroup::Future, item, amount);
        }
    }

    let mut all = Bucket::default();
    for b in &buckets {
        all.merge(b);
    }
    Ok(SpendGroupsReport {
        currency: base,
        months: months
            .into_iter()
            .zip(&buckets)
            .map(|(m, b)| month_view(m, b, &tree))
            .collect(),
        total: month_view("total".into(), &all, &tree),
        targets: crate::budget::group_targets(conn)?,
    })
}
