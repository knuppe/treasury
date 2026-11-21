//! Sankey: para onde vai a receita do período, com o período anterior equivalente ao lado.
//!
//! Fontes (coluna da esquerda): receitas por categoria de 1º nível, resgates de investimentos
//! e, quando o que sai passa do que entra, o déficit. Destinos (depois do nó central):
//! despesas por categoria de 1º nível (e subcategoria de nível 1), pagamento de dívidas
//! (categorias ligadas a um contrato), aportes em investimentos e a sobra.
//!
//! As regras de fluxo de caixa são as de [`super::engine`]: transferências entre contas
//! próprias e pagamento de fatura nunca entram (a compra no cartão já conta como despesa).
//! Aportes e resgates (`investment`) entram só pela perna numa conta que não é de corretora:
//! a que sai é aporte; a que entra é resgate quando a outra perna está numa corretora (ou
//! não existe). Uma transferência comum sem categoria entre o banco e uma corretora (ou
//! previdência) conta igual. Compras e vendas dentro da corretora ficam de fora: o dinheiro
//! já entrou no diagrama quando saiu do banco.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use chrono::Datelike;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{CategoryTree, FlowQuery, Fx, Key, ratio, validate_range};
use super::no_category;
use crate::budget::Basis;
use crate::error::Result;
use crate::household::base_currency;
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{add_months, fmt_date, month_bounds};

mod items;
pub use items::{
    SankeyItem, SankeyItems, SankeyItemsArgs, SankeyItemsGroup, SankeyItemsPeriod, sankey_items,
};

/// Profundidade padrão: categorias de 1º nível e subcategorias.
pub const SANKEY_DEFAULT_DEPTH: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SankeyNodeKind {
    Income,
    Hub,
    Expense,
    Subcategory,
    Savings,
    Deficit,
    /// Aportes em investimentos (destino).
    Investment,
    /// Resgates de investimentos (fonte).
    Redemption,
    /// Pagamento de dívidas (destino; parte das despesas).
    Debt,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyNode {
    pub id: String,
    pub name: String,
    pub kind: SankeyNodeKind,
    pub category_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyLink {
    pub source: String,
    pub target: String,
    #[ts(type = "number")]
    pub value: i64,
}

/// Totais de um período do Sankey.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyTotals {
    pub from: String,
    pub to: String,
    #[ts(type = "number")]
    pub income: i64,
    /// Todas as despesas (inclusive o pagamento de dívidas).
    #[ts(type = "number")]
    pub expense: i64,
    /// Parte das despesas lançada nas categorias ligadas a um contrato de dívida.
    #[ts(type = "number")]
    pub debt_payments: i64,
    /// Aportes: saídas de contas do dia a dia para investimentos.
    #[ts(type = "number")]
    pub investments: i64,
    /// Resgates: entradas vindas de investimentos.
    #[ts(type = "number")]
    pub redemptions: i64,
    /// O que sobrou: receitas + resgates − despesas − aportes (negativo = déficit).
    #[ts(type = "number")]
    pub leftover: i64,
    /// (receitas − despesas) ÷ receitas; `None` sem receitas.
    pub savings_rate: Option<f64>,
}

/// Um nó do diagrama nos dois períodos (também os que só existiram no anterior).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyDelta {
    pub id: String,
    pub name: String,
    pub kind: SankeyNodeKind,
    pub category_id: Option<Id>,
    /// Nó de que este sai (subcategoria → categoria, contrato → dívidas); `None` nas colunas
    /// ligadas ao nó central.
    pub parent_id: Option<String>,
    #[ts(type = "number")]
    pub current: i64,
    #[ts(type = "number")]
    pub previous: i64,
    #[ts(type = "number")]
    pub delta: i64,
    /// `delta ÷ previous`; `None` quando o anterior é zero.
    pub delta_pct: Option<f64>,
    /// `current ÷ receitas do período`; `None` sem receitas.
    pub share_of_income: Option<f64>,
}

/// Fluxo fontes → orçamento → destinos → subcategorias, com o período anterior.
///
/// Ids de nó estáveis: `income:<categoria de 1º nível>`, `income:none`, `redemption`,
/// `deficit`, `hub`, `expense:<categoria de 1º nível>`, `expense:none`,
/// `sub:<subcategoria de nível 1>` (ou `sub:<categoria de 1º nível>` para o gasto lançado
/// direto nela, quando ela também tem subcategorias com gasto), `debt`, `debt:<categoria do
/// contrato>` (só com dois contratos ou mais), `investment` e `savings`. Sem links de valor
/// zero; em cada nó intermediário a soma do que entra é igual à do que sai. Grupos com saldo
/// negativo (estornos maiores que as compras no período) ficam fora do diagrama.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyData {
    pub from: String,
    pub to: String,
    pub basis: Basis,
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    #[ts(type = "number")]
    pub total_income: i64,
    #[ts(type = "number")]
    pub total_expense: i64,
    pub currency: String,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
    /// Faturas de cartão pagas no período (moeda base). Em competência, as compras delas
    /// contam no mês em que foram feitas; em caixa, aparecem aqui, nas categorias.
    #[ts(type = "number")]
    pub card_bills_paid: i64,
    /// 1 = só categorias de 1º nível; 2 = também subcategorias e contratos.
    pub depth: u8,
    pub totals: SankeyTotals,
    /// Período anterior equivalente (`None` quando a comparação não foi pedida).
    pub previous: Option<SankeyTotals>,
    /// Cada nó (menos o central) nos dois períodos, na ordem do diagrama; os que só
    /// existiram no anterior vêm no fim. Vazio sem comparação.
    pub comparison: Vec<SankeyDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyArgs {
    pub from: String,
    pub to: String,
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// 1 = só categorias de 1º nível; 2 (padrão) = também subcategorias.
    #[ts(optional)]
    pub depth: Option<u8>,
    /// Compara com o período anterior equivalente (padrão: sim).
    #[ts(optional)]
    pub compare: Option<bool>,
}

/// Período anterior de mesma duração: meses inteiros recuam o mesmo número de meses
/// (trimestre → trimestre anterior, ano → ano anterior); os demais, o mesmo número de dias.
pub fn previous_period(from: &str, to: &str) -> Result<(String, String)> {
    let (a, b) = validate_range(from, to)?;
    let whole_months = a.day() == 1 && b == month_bounds(b.year(), b.month()).1;
    if whole_months {
        let n = (b.year() - a.year()) * 12 + b.month() as i32 - a.month() as i32 + 1;
        let start = add_months(a, -n);
        let end_month = add_months(month_bounds(b.year(), b.month()).0, -n);
        let end = month_bounds(end_month.year(), end_month.month()).1;
        return Ok((fmt_date(start), fmt_date(end)));
    }
    let days = (b - a).num_days() + 1;
    let end = a.pred_opt().unwrap_or(a);
    Ok((fmt_date(a - chrono::Duration::days(days)), fmt_date(end)))
}

pub fn sankey(conn: &Connection, from: &str, to: &str, basis: Basis) -> Result<SankeyData> {
    sankey_with(conn, from, to, basis, SANKEY_DEFAULT_DEPTH, true)
}

pub fn sankey_with(
    conn: &Connection,
    from: &str,
    to: &str,
    basis: Basis,
    depth: u8,
    compare: bool,
) -> Result<SankeyData> {
    validate_range(from, to)?;
    let depth = depth.clamp(1, 2);
    let base = base_currency(conn)?;
    let tree = CategoryTree::load(conn)?;
    let debts = debt_categories(conn)?;
    let ctx = Ctx {
        conn,
        tree: &tree,
        debts: &debts,
        base: &base,
        basis,
        depth,
    };
    let cur = ctx.graph(from, to)?;
    let (previous, comparison) = if compare {
        let (pf, pt) = previous_period(from, to)?;
        let prev = ctx.graph(&pf, &pt)?;
        let rows = compare_nodes(&cur, &prev);
        (Some(prev.totals), rows)
    } else {
        (None, vec![])
    };
    let card_bills_paid = card_bills_paid(conn, from, to, &base)?;
    Ok(SankeyData {
        from: from.to_string(),
        to: to.to_string(),
        basis,
        nodes: cur.nodes,
        links: cur.links,
        total_income: cur.totals.income,
        total_expense: cur.totals.expense,
        currency: base,
        missing_fx: cur.missing.into_iter().collect(),
        estimated: cur.estimated,
        card_bills_paid,
        depth,
        totals: cur.totals,
        previous,
        comparison,
    })
}

/// Pagamentos de fatura feitos no período: a perna que entra no cartão (efetivada), na moeda
/// base (sem câmbio, fica de fora).
fn card_bills_paid(conn: &Connection, from: &str, to: &str, base: &str) -> Result<i64> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.amount, t.currency, t.date FROM txn t JOIN account a ON a.id = t.account_id
          WHERE a.kind = 'credit_card' AND t.kind = 'card_payment' AND t.amount > 0
            AND t.status <> 'planned' AND t.date >= ?1 AND t.date <= ?2",
    )?;
    let mut q = stmt.query(rusqlite::params![from, to])?;
    let mut total = 0;
    while let Some(r) = q.next()? {
        let (amount, currency, date): (i64, String, String) = (r.get(0)?, r.get(1)?, r.get(2)?);
        if currency == base {
            total += amount;
        } else if let Some(c) = crate::fx::convert(conn, amount, &currency, base, &date)? {
            total += c.amount;
        }
    }
    Ok(total)
}

// ---------------------------------------------------------------------------
// Montagem

struct Ctx<'a> {
    conn: &'a Connection,
    tree: &'a CategoryTree,
    /// Categoria ligada a um contrato → nome do contrato.
    debts: &'a BTreeMap<Id, String>,
    base: &'a str,
    basis: Basis,
    depth: u8,
}

struct Graph {
    nodes: Vec<SankeyNode>,
    links: Vec<SankeyLink>,
    /// Valor de cada nó ([`node_values`]) e também das subcategorias que o diagrama não
    /// desenha: o gasto direto de uma categoria sem subcategorias com gasto (`sub:<raiz>`) e
    /// o contrato único (`debt:<categoria>`). Assim o comparativo mostra no período anterior
    /// o valor real de uma linha que só no atual virou nó.
    values: HashMap<String, i64>,
    totals: SankeyTotals,
    missing: BTreeSet<String>,
    estimated: bool,
}

/// Onde um valor de receita ou despesa entra no diagrama.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Place {
    /// Receita, pela categoria de 1º nível (`None` = sem categoria).
    Income(Option<Id>),
    /// Despesa: categoria de 1º nível e subcategoria de nível 1 (`None` = lançada direto na
    /// categoria de 1º nível, ou sem categoria).
    Expense { root: Option<Id>, sub: Option<Id> },
    /// Pagamento de dívida, pela categoria do contrato.
    Debt(Id),
}

/// Cada categoria de 1º nível: soma das partes positivas (gasto direto + subcategorias).
struct RootFlow {
    root: Option<Id>,
    value: i64,
    subs: Vec<(Option<Id>, i64)>,
}

/// Ordem: maior valor primeiro, depois nome.
fn by_value<K>(v: &mut [(K, i64)], name: impl Fn(&K) -> String) {
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| name(&a.0).cmp(&name(&b.0))));
}

impl Ctx<'_> {
    fn name_of(&self, id: &Option<Id>) -> String {
        id.as_deref()
            .and_then(|i| self.tree.get(i))
            .map(|c| c.name.clone())
            .unwrap_or_else(no_category)
    }

    /// Categoria de contrato que cobre `cat` (ela mesma ou um ancestral).
    fn debt_of(&self, cat: &str) -> Option<Id> {
        if self.debts.is_empty() {
            return None;
        }
        self.tree
            .ancestors_inclusive(cat)
            .into_iter()
            .find(|c| self.debts.contains_key(&c.id))
            .map(|c| c.id.clone())
    }

    /// Lugar no diagrama de um valor lançado em `cat` (receita ou despesa).
    fn place(&self, cat: Option<&str>, income: bool) -> Place {
        let info = cat.and_then(|c| self.tree.get(c));
        if income {
            return Place::Income(info.map(|c| c.root_id.clone()));
        }
        if let Some(d) = info.and_then(|c| self.debt_of(&c.id)) {
            return Place::Debt(d);
        }
        match info {
            Some(c) if c.depth == 0 => Place::Expense {
                root: Some(c.id.clone()),
                sub: None,
            },
            Some(c) => Place::Expense {
                root: Some(c.root_id.clone()),
                sub: self.tree.level1(&c.id).map(|l| l.id.clone()),
            },
            None => Place::Expense {
                root: None,
                sub: None,
            },
        }
    }

    fn graph(&self, from: &str, to: &str) -> Result<Graph> {
        let mut fx = Fx::new(self.conn, self.base);
        let agg = FlowQuery::new(from, to, self.basis).aggregate(
            self.conn,
            &mut fx,
            Key::Category,
            Key::None,
        )?;
        let (investments, redemptions) = investment_flows(self.conn, &mut fx, from, to)?;
        // Receitas por categoria de 1º nível; despesas por 1º nível → subcategoria de nível 1
        // (`None` = lançado direto na categoria de 1º nível); dívidas por categoria do contrato.
        let mut income: BTreeMap<Option<Id>, i64> = BTreeMap::new();
        let mut expense: BTreeMap<Option<Id>, BTreeMap<Option<Id>, i64>> = BTreeMap::new();
        let mut debt: BTreeMap<Id, i64> = BTreeMap::new();
        for ((cat, _), cell) in &agg.cells {
            if cell.income != 0
                && let Place::Income(root) = self.place(cat.as_deref(), true)
            {
                *income.entry(root).or_default() += cell.income;
            }
            if cell.expense == 0 {
                continue;
            }
            match self.place(cat.as_deref(), false) {
                Place::Debt(d) => *debt.entry(d).or_default() += cell.expense,
                Place::Expense { root, sub } => {
                    *expense.entry(root).or_default().entry(sub).or_default() += cell.expense
                }
                Place::Income(_) => {}
            }
        }

        let mut income_links: Vec<(Option<Id>, i64)> =
            income.into_iter().filter(|(_, v)| *v > 0).collect();
        by_value(&mut income_links, |k| self.name_of(k));
        let total_in: i64 = income_links.iter().map(|(_, v)| v).sum();

        let mut roots: Vec<RootFlow> = expense
            .into_iter()
            .map(|(root, subs)| {
                let mut subs: Vec<(Option<Id>, i64)> =
                    subs.into_iter().filter(|(_, v)| *v > 0).collect();
                by_value(&mut subs, |k| self.name_of(k));
                RootFlow {
                    value: subs.iter().map(|(_, v)| v).sum(),
                    root,
                    subs,
                }
            })
            .filter(|r| r.value > 0)
            .collect();
        roots.sort_by(|a, b| {
            b.value
                .cmp(&a.value)
                .then_with(|| self.name_of(&a.root).cmp(&self.name_of(&b.root)))
        });
        let mut debt_links: Vec<(Id, i64)> = debt.into_iter().filter(|(_, v)| *v > 0).collect();
        by_value(&mut debt_links, |k| {
            self.debts.get(k).cloned().unwrap_or_default()
        });
        let total_debt: i64 = debt_links.iter().map(|(_, v)| v).sum();
        let total_roots: i64 = roots.iter().map(|r| r.value).sum();

        let sources = total_in + redemptions;
        let uses = total_roots + total_debt + investments;

        let mut g = GraphBuilder::default();
        let hub = || SankeyNode {
            id: "hub".into(),
            name: t("nucleo-relatorios-fluxo-orcamento"),
            kind: SankeyNodeKind::Hub,
            category_id: None,
        };
        for (root, v) in &income_links {
            let id = format!("income:{}", suffix(root));
            g.node(SankeyNode {
                id: id.clone(),
                name: if root.is_some() {
                    self.name_of(root)
                } else {
                    t("nucleo-relatorios-fluxo-receitas-sem-categoria")
                },
                kind: SankeyNodeKind::Income,
                category_id: root.clone(),
            });
            g.node(hub());
            g.link(&id, "hub", *v);
        }
        if redemptions > 0 {
            g.node(special(
                "redemption",
                t("nucleo-relatorios-fluxo-resgates"),
                SankeyNodeKind::Redemption,
            ));
            g.node(hub());
            g.link("redemption", "hub", redemptions);
        }
        if uses > sources {
            g.node(special(
                "deficit",
                t("nucleo-relatorios-fluxo-deficit"),
                SankeyNodeKind::Deficit,
            ));
            g.node(hub());
            g.link("deficit", "hub", uses - sources);
        }
        for r in &roots {
            let id = format!("expense:{}", suffix(&r.root));
            g.node(hub());
            g.node(SankeyNode {
                id: id.clone(),
                name: self.name_of(&r.root),
                kind: SankeyNodeKind::Expense,
                category_id: r.root.clone(),
            });
            g.link("hub", &id, r.value);
        }
        if total_debt > 0 {
            g.node(hub());
            let only = (debt_links.len() == 1).then(|| debt_links[0].0.clone());
            g.node(SankeyNode {
                id: "debt".into(),
                name: t("nucleo-relatorios-fluxo-dividas"),
                kind: SankeyNodeKind::Debt,
                category_id: only,
            });
            g.link("hub", "debt", total_debt);
        }
        if investments > 0 {
            g.node(hub());
            g.node(special(
                "investment",
                t("nucleo-relatorios-fluxo-investimentos"),
                SankeyNodeKind::Investment,
            ));
            g.link("hub", "investment", investments);
        }
        if sources > uses {
            g.node(hub());
            g.node(special(
                "savings",
                t("nucleo-relatorios-fluxo-sobra"),
                SankeyNodeKind::Savings,
            ));
            g.link("hub", "savings", sources - uses);
        }
        if self.depth >= 2 {
            for r in &roots {
                // Sem subcategorias com gasto, a categoria de 1º nível é terminal.
                if r.root.is_none() || !r.subs.iter().any(|(s, _)| s.is_some()) {
                    continue;
                }
                let parent = format!("expense:{}", suffix(&r.root));
                for (sub, v) in &r.subs {
                    let (id, name, cat) = match sub {
                        Some(s) => (format!("sub:{s}"), self.name_of(sub), Some(s.clone())),
                        None => (
                            format!("sub:{}", suffix(&r.root)),
                            without_sub(&self.name_of(&r.root)),
                            r.root.clone(),
                        ),
                    };
                    g.node(SankeyNode {
                        id: id.clone(),
                        name,
                        kind: SankeyNodeKind::Subcategory,
                        category_id: cat,
                    });
                    g.link(&parent, &id, *v);
                }
            }
            // Um contrato só: o nó de dívidas já é ele.
            if debt_links.len() > 1 {
                for (cat, v) in &debt_links {
                    let id = format!("debt:{cat}");
                    g.node(SankeyNode {
                        id: id.clone(),
                        name: self.debts.get(cat).cloned().unwrap_or_default(),
                        kind: SankeyNodeKind::Subcategory,
                        category_id: Some(cat.clone()),
                    });
                    g.link("debt", &id, *v);
                }
            }
        }

        let mut values: HashMap<String, i64> = node_values(&g.links)
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        if self.depth >= 2 {
            for r in &roots {
                if let (Some(root), [(None, v)]) = (&r.root, r.subs.as_slice()) {
                    values.entry(format!("sub:{root}")).or_insert(*v);
                }
            }
            if let [(cat, v)] = debt_links.as_slice() {
                values.entry(format!("debt:{cat}")).or_insert(*v);
            }
        }
        let total = agg.total();
        let mut missing: BTreeSet<String> = agg.missing_fx.clone();
        missing.extend(fx.missing.iter().cloned());
        Ok(Graph {
            nodes: g.nodes,
            links: g.links,
            values,
            totals: SankeyTotals {
                from: from.to_string(),
                to: to.to_string(),
                income: total.income,
                expense: total.expense,
                debt_payments: total_debt,
                investments,
                redemptions,
                leftover: total.income + redemptions - total.expense - investments,
                savings_rate: ratio(total.income - total.expense, total.income),
            },
            missing,
            estimated: agg.estimated || fx.estimated,
        })
    }
}

fn suffix(id: &Option<Id>) -> String {
    id.clone().unwrap_or_else(|| "none".to_string())
}

/// "Moradia (sem subcategoria)": o gasto direto numa categoria de 1º nível.
pub(super) fn without_sub(name: &str) -> String {
    t_args(
        "nucleo-relatorios-fluxo-sem-subcategoria",
        &args(&[("categoria", name.to_string().into())]),
    )
}

fn special(id: &str, name: String, kind: SankeyNodeKind) -> SankeyNode {
    SankeyNode {
        id: id.into(),
        name,
        kind,
        category_id: None,
    }
}

#[derive(Default)]
struct GraphBuilder {
    nodes: Vec<SankeyNode>,
    links: Vec<SankeyLink>,
    seen: HashSet<String>,
}

impl GraphBuilder {
    fn node(&mut self, n: SankeyNode) {
        if self.seen.insert(n.id.clone()) {
            self.nodes.push(n);
        }
    }
    fn link(&mut self, source: &str, target: &str, value: i64) {
        self.links.push(SankeyLink {
            source: source.into(),
            target: target.into(),
            value,
        });
    }
}

/// Categorias ligadas a contratos de dívida (`loan.category_id`) → nome do contrato
/// (os ativos antes dos arquivados).
fn debt_categories(conn: &Connection) -> Result<BTreeMap<Id, String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT category_id, name FROM loan WHERE category_id IS NOT NULL ORDER BY archived, name",
    )?;
    let mut out = BTreeMap::new();
    let rows = stmt.query_map([], |r| Ok((r.get::<_, Id>(0)?, r.get::<_, String>(1)?)))?;
    for row in rows {
        let (cat, name) = row?;
        out.entry(cat).or_insert(name);
    }
    Ok(out)
}

/// Perna de aporte ou resgate que entra no diagrama (na moeda da conta).
struct InvestmentLeg {
    txn_id: Id,
    /// Saída (aporte); senão, entrada (resgate).
    out: bool,
    currency: String,
    date: String,
    amount: i64,
    description: String,
    payee: Option<String>,
    account: String,
    /// Conta da outra perna (a corretora), quando houver.
    other_account: Option<String>,
}

/// Aportes e resgates efetivados no período, um por lançamento (pela data do lançamento).
fn investment_legs(conn: &Connection, from: &str, to: &str) -> Result<Vec<InvestmentLeg>> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.id, t.amount < 0, t.currency, t.date, t.amount, t.description, t.payee, a.name,
                (SELECT oa.name FROM txn o JOIN account oa ON oa.id = o.account_id
                  WHERE o.transfer_id = t.transfer_id AND o.id <> t.id)
         FROM txn t JOIN account a ON a.id = t.account_id
         WHERE t.status IN ('cleared', 'reconciled')
           AND t.date BETWEEN ?1 AND ?2 AND a.kind NOT IN ('brokerage', 'retirement')
           AND ((t.kind = 'investment' AND (t.amount < 0 OR NOT EXISTS (
                    SELECT 1 FROM txn o JOIN account oa ON oa.id = o.account_id
                    WHERE o.transfer_id = t.transfer_id AND o.id <> t.id
                      AND oa.kind NOT IN ('brokerage', 'retirement'))))
             -- Transferência comum (sem categoria) entre o banco e uma corretora ou previdência
             -- também é aporte (ida) ou resgate (volta).
             OR (t.kind = 'transfer' AND t.category_id IS NULL AND EXISTS (
                    SELECT 1 FROM txn o JOIN account oa ON oa.id = o.account_id
                    WHERE o.transfer_id = t.transfer_id AND o.id <> t.id
                      AND oa.kind IN ('brokerage', 'retirement'))))
         ORDER BY t.date, t.id",
    )?;
    let rows = stmt
        .query_map([from, to], |r| {
            Ok(InvestmentLeg {
                txn_id: r.get(0)?,
                out: r.get(1)?,
                currency: r.get(2)?,
                date: r.get(3)?,
                amount: r.get(4)?,
                description: r.get(5)?,
                payee: r.get(6)?,
                account: r.get(7)?,
                other_account: r.get(8)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Aportes e resgates do período na moeda base: cada lançamento convertido na sua data.
fn investment_flows(
    conn: &Connection,
    fx: &mut Fx<'_>,
    from: &str,
    to: &str,
) -> Result<(i64, i64)> {
    let (mut aportes, mut resgates) = (0, 0);
    for leg in investment_legs(conn, from, to)? {
        let Some(c) = fx.convert(leg.amount, &leg.currency, &leg.date)? else {
            continue;
        };
        if leg.out {
            aportes -= c.amount;
        } else {
            resgates += c.amount;
        }
    }
    Ok((aportes, resgates))
}

/// Valor de cada nó: o maior entre o que entra e o que sai.
fn node_values(links: &[SankeyLink]) -> HashMap<&str, i64> {
    let mut inflow: HashMap<&str, i64> = HashMap::new();
    let mut outflow: HashMap<&str, i64> = HashMap::new();
    for l in links {
        *outflow.entry(l.source.as_str()).or_default() += l.value;
        *inflow.entry(l.target.as_str()).or_default() += l.value;
    }
    let mut out = inflow.clone();
    for (k, v) in outflow {
        let e = out.entry(k).or_default();
        *e = (*e).max(v);
    }
    out
}

/// Nó de origem de cada nó que não sai do central (subcategorias, contratos).
fn parents(links: &[SankeyLink]) -> HashMap<&str, &str> {
    links
        .iter()
        .filter(|l| l.source != "hub" && l.target != "hub")
        .map(|l| (l.target.as_str(), l.source.as_str()))
        .collect()
}

fn compare_nodes(cur: &Graph, prev: &Graph) -> Vec<SankeyDelta> {
    let (cp, pp) = (parents(&cur.links), parents(&prev.links));
    let income = cur.totals.income;
    let row = |n: &SankeyNode, parent: Option<&&str>| {
        let current = cur.values.get(&n.id).copied().unwrap_or(0);
        let previous = prev.values.get(&n.id).copied().unwrap_or(0);
        let delta = current - previous;
        SankeyDelta {
            id: n.id.clone(),
            name: n.name.clone(),
            kind: n.kind,
            category_id: n.category_id.clone(),
            parent_id: parent.map(|p| p.to_string()),
            current,
            previous,
            delta,
            delta_pct: (previous > 0).then(|| delta as f64 / previous as f64),
            share_of_income: ratio(current, income),
        }
    };
    let mut out: Vec<SankeyDelta> = cur
        .nodes
        .iter()
        .filter(|n| n.kind != SankeyNodeKind::Hub)
        .map(|n| row(n, cp.get(n.id.as_str())))
        .collect();
    let seen: HashSet<&str> = cur.nodes.iter().map(|n| n.id.as_str()).collect();
    out.extend(
        prev.nodes
            .iter()
            .filter(|n| n.kind != SankeyNodeKind::Hub && !seen.contains(n.id.as_str()))
            .map(|n| row(n, pp.get(n.id.as_str()))),
    );
    out
}
