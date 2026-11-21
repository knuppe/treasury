//! Analítico de uma linha do fluxo (Sankey e comparativo): os lançamentos que compõem o
//! valor de um nó em cada período, pelas mesmas regras do diagrama, para a soma bater ao
//! centavo com a tabela.
//!
//! - Um item é um lançamento numa categoria (as partes de uma divisão na mesma categoria
//!   viram um item só), convertido para a moeda base na data do lançamento, como na
//!   agregação do diagrama.
//! - O diagrama deixa de fora os grupos com saldo negativo ou zero (estornos maiores que os
//!   lançamentos): a receita por categoria de 1º nível, a despesa por subcategoria de nível 1
//!   e o pagamento de dívidas por contrato. Os itens desses grupos vêm com `counted = false`
//!   e o motivo, como os de moeda sem câmbio.
//! - Sobra e déficit não têm lançamentos: os itens são os nós que entram na conta (fontes
//!   menos destinos), cada um com o nó de origem para abrir o analítico dele.

use std::collections::{BTreeMap, HashMap};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Ctx, Graph, Place, SankeyNodeKind, debt_categories, investment_legs, previous_period, suffix,
    without_sub,
};
use crate::budget::Basis;
use crate::error::{Error, Result};
use crate::household::base_currency;
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::reports::engine::{
    CATEGORY_EXPR, CategoryTree, FLOW_KIND, FlowQuery, Fx, Params, money, validate_range,
};
use crate::reports::explain::txn_label;
use crate::reports::no_category;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyItemsArgs {
    pub from: String,
    pub to: String,
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// A mesma profundidade do diagrama (padrão 2).
    #[ts(optional)]
    pub depth: Option<u8>,
    /// Nós do diagrama (ids de `SankeyData.comparison`); mais de um soma os nós, que não
    /// podem se sobrepor (como "Outras" no gráfico).
    pub node_ids: Vec<String>,
}

/// Um lançamento (ou a parte dele numa categoria) que compõe o valor de um nó.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyItem {
    /// Lançamento; `None` nos itens de sobra e déficit (que são nós do diagrama).
    pub txn_id: Option<Id>,
    /// Nó pedido de que o item faz parte.
    pub node_id: String,
    /// Sobra e déficit: o nó do diagrama que o item representa.
    pub component: Option<String>,
    pub date: Option<String>,
    /// Vencimento da fatura, quando a base é caixa e é ele que põe o lançamento no período.
    pub due_date: Option<String>,
    pub label: String,
    pub account: Option<String>,
    pub category_id: Option<Id>,
    /// Caminho da categoria (`Rendimentos › Dividendos`), "Sem categoria", a corretora dos
    /// aportes e resgates ou, na sobra e no déficit, "Entradas" e "Saídas".
    pub group: String,
    /// Quanto o item soma ao nó, na moeda base (estorno e saída na sobra = negativo).
    #[ts(type = "number")]
    pub amount: i64,
    /// O mesmo na moeda do lançamento, quando ela não é a base.
    #[ts(type = "number | null")]
    pub original_amount: Option<i64>,
    pub original_currency: Option<String>,
    /// Parte de um lançamento dividido.
    pub split: bool,
    /// Entra no valor do nó? Quando não, o motivo vem em `note`.
    pub counted: bool,
    pub note: Option<String>,
    pub estimated: bool,
}

/// Um período do analítico.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyItemsPeriod {
    pub from: String,
    pub to: String,
    /// Valor do nó no diagrama e no comparativo.
    #[ts(type = "number")]
    pub value: i64,
    /// Soma dos itens que contam (igual a `value`).
    #[ts(type = "number")]
    pub items_total: i64,
    pub items: Vec<SankeyItem>,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
    /// Por que o nó não aparece no período, quando é o caso (houve déficit em vez de sobra).
    pub note: Option<String>,
}

/// Um grupo (categoria) nos dois períodos, só com os itens que contam.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyItemsGroup {
    pub group: String,
    pub category_id: Option<Id>,
    #[ts(type = "number")]
    pub current: i64,
    #[ts(type = "number")]
    pub previous: i64,
    #[ts(type = "number")]
    pub delta: i64,
    pub current_count: u32,
    pub previous_count: u32,
}

/// O que compõe uma linha do fluxo no período e no anterior equivalente.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SankeyItems {
    pub node_ids: Vec<String>,
    pub name: String,
    pub kind: SankeyNodeKind,
    pub basis: Basis,
    pub currency: String,
    pub current: SankeyItemsPeriod,
    pub previous: SankeyItemsPeriod,
    /// Por grupo, maior variação primeiro: a soma dos `delta` é a variação da linha.
    pub groups: Vec<SankeyItemsGroup>,
}

const DERIVED: [&str; 2] = ["savings", "deficit"];

fn validate_node(id: &str) -> Result<()> {
    let plain = matches!(
        id,
        "savings" | "deficit" | "investment" | "redemption" | "debt"
    );
    let prefixed = ["income:", "expense:", "sub:", "debt:"]
        .iter()
        .any(|p| id.strip_prefix(p).is_some_and(|rest| !rest.is_empty()));
    if plain || prefixed {
        Ok(())
    } else {
        Err(Error::invalid(t_args(
            "nucleo-relatorios-erro-no-desconhecido",
            &args(&[("no", id.to_string().into())]),
        )))
    }
}

pub fn sankey_items(
    conn: &Connection,
    from: &str,
    to: &str,
    basis: Basis,
    depth: u8,
    node_ids: &[String],
) -> Result<SankeyItems> {
    validate_range(from, to)?;
    if node_ids.is_empty() {
        return Err(Error::invalid(t("nucleo-relatorios-erro-sem-nos")));
    }
    for id in node_ids {
        validate_node(id)?;
    }
    let base = base_currency(conn)?;
    let tree = CategoryTree::load(conn)?;
    let debts = debt_categories(conn)?;
    let ctx = Ctx {
        conn,
        tree: &tree,
        debts: &debts,
        base: &base,
        basis,
        depth: depth.clamp(1, 2),
    };
    let (pf, pt) = previous_period(from, to)?;
    let (current, cur_graph) = ctx.items_period(from, to, node_ids)?;
    let (previous, prev_graph) = ctx.items_period(&pf, &pt, node_ids)?;
    let first = &node_ids[0];
    let (name, kind) = cur_graph
        .nodes
        .iter()
        .chain(&prev_graph.nodes)
        .find(|n| &n.id == first)
        .map(|n| (n.name.clone(), n.kind))
        .unwrap_or_else(|| ctx.describe(first));
    let groups = groups(&current.items, &previous.items);
    Ok(SankeyItems {
        node_ids: node_ids.to_vec(),
        name,
        kind,
        basis,
        currency: base,
        current,
        previous,
        groups,
    })
}

/// Grupos dos dois períodos (só itens que contam), maior variação primeiro.
fn groups(current: &[SankeyItem], previous: &[SankeyItem]) -> Vec<SankeyItemsGroup> {
    let mut by: BTreeMap<(String, Option<Id>), SankeyItemsGroup> = BTreeMap::new();
    let mut add = |items: &[SankeyItem], cur: bool| {
        for i in items.iter().filter(|i| i.counted) {
            let g = by
                .entry((i.group.clone(), i.category_id.clone()))
                .or_insert_with(|| SankeyItemsGroup {
                    group: i.group.clone(),
                    category_id: i.category_id.clone(),
                    current: 0,
                    previous: 0,
                    delta: 0,
                    current_count: 0,
                    previous_count: 0,
                });
            if cur {
                g.current += i.amount;
                g.current_count += 1;
            } else {
                g.previous += i.amount;
                g.previous_count += 1;
            }
        }
    };
    add(current, true);
    add(previous, false);
    let mut out: Vec<SankeyItemsGroup> = by
        .into_values()
        .map(|mut g| {
            g.delta = g.current - g.previous;
            g
        })
        .collect();
    out.sort_by(|a, b| {
        b.delta
            .abs()
            .cmp(&a.delta.abs())
            .then_with(|| a.group.cmp(&b.group))
    });
    out
}

/// Nós em que um valor de receita ou despesa entra: o de 1º nível (ou de dívidas) e a
/// subcategoria (ou o contrato).
fn nodes_of(place: &Place) -> (String, Option<String>) {
    match place {
        Place::Income(root) => (format!("income:{}", suffix(root)), None),
        Place::Debt(cat) => ("debt".into(), Some(format!("debt:{cat}"))),
        Place::Expense { root, sub } => (
            format!("expense:{}", suffix(root)),
            root.as_ref()
                .map(|r| format!("sub:{}", sub.as_ref().unwrap_or(r))),
        ),
    }
}

/// Receita ou despesa de um lançamento numa categoria, na moeda do lançamento.
struct FlowRow {
    txn_id: Id,
    category_id: Option<Id>,
    income: bool,
    currency: String,
    date: String,
    eff: String,
    amount: i64,
    split: bool,
    description: String,
    payee: Option<String>,
    account: String,
}

impl Ctx<'_> {
    fn items_period(
        &self,
        from: &str,
        to: &str,
        node_ids: &[String],
    ) -> Result<(SankeyItemsPeriod, Graph)> {
        let graph = self.graph(from, to)?;
        let value = node_ids
            .iter()
            .map(|id| graph.values.get(id).copied().unwrap_or(0))
            .sum();
        let wants = |id: &str| node_ids.iter().any(|n| n == id);
        let mut fx = Fx::new(self.conn, self.base);
        let mut items = vec![];
        let mut notes = vec![];
        for id in node_ids.iter().filter(|id| DERIVED.contains(&id.as_str())) {
            match derived_items(&graph, id) {
                Some(v) => items.extend(v),
                None => notes.push(absent_note(&graph, id, self.base)),
            }
        }
        if node_ids
            .iter()
            .any(|id| !DERIVED.contains(&id.as_str()) && id != "investment" && id != "redemption")
        {
            items.extend(self.flow_items(from, to, &mut fx, &wants)?);
        }
        if wants("investment") || wants("redemption") {
            items.extend(self.investment_items(from, to, &mut fx, &wants)?);
        }
        let items_total = items.iter().filter(|i| i.counted).map(|i| i.amount).sum();
        let estimated = fx.estimated || items.iter().any(|i| i.estimated);
        Ok((
            SankeyItemsPeriod {
                from: from.to_string(),
                to: to.to_string(),
                value,
                items_total,
                items,
                missing_fx: fx.missing_vec(),
                estimated,
                note: (!notes.is_empty()).then(|| notes.join(" ")),
            },
            graph,
        ))
    }

    fn flow_rows(&self, from: &str, to: &str) -> Result<Vec<FlowRow>> {
        let q = FlowQuery::new(from, to, self.basis);
        let mut p = Params::default();
        let eff = q.eff_expr();
        let tail = q.sql_tail(&mut p, false);
        let sql = format!(
            "SELECT t.id, {CATEGORY_EXPR} AS cat, {FLOW_KIND} AS fk, t.currency, t.date, {eff},
                    SUM(COALESCE(s.amount, t.amount)), COUNT(s.id), t.description, t.payee,
                    (SELECT name FROM account WHERE id = t.account_id)
             {tail}
             GROUP BY t.id, cat, fk
             ORDER BY {eff}, t.date, t.id, cat"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(p.0.iter()), |r| {
                Ok(FlowRow {
                    txn_id: r.get(0)?,
                    category_id: r.get(1)?,
                    income: r.get::<_, String>(2)? == "income",
                    currency: r.get(3)?,
                    date: r.get(4)?,
                    eff: r.get(5)?,
                    amount: r.get(6)?,
                    split: r.get::<_, i64>(7)? > 0,
                    description: r.get(8)?,
                    payee: r.get(9)?,
                    account: r.get::<_, Option<String>>(10)?.unwrap_or_default(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Receitas, despesas e dívidas dos nós pedidos. Um grupo só conta com saldo positivo
    /// no período, como no diagrama.
    fn flow_items(
        &self,
        from: &str,
        to: &str,
        fx: &mut Fx<'_>,
        wants: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<SankeyItem>> {
        let rows = self.flow_rows(from, to)?;
        let mut placed = Vec::with_capacity(rows.len());
        let mut units: HashMap<Place, i64> = HashMap::new();
        for row in rows {
            let place = self.place(row.category_id.as_deref(), row.income);
            let conv = fx.convert(row.amount, &row.currency, &row.date)?;
            let sign = if row.income { 1 } else { -1 };
            let amount = conv.as_ref().map(|c| sign * c.amount);
            if let Some(a) = amount {
                *units.entry(place.clone()).or_default() += a;
            }
            let estimated = conv.as_ref().is_some_and(|c| c.estimated);
            placed.push((row, place, amount, sign, estimated));
        }
        let mut out = vec![];
        for (row, place, amount, sign, estimated) in placed {
            let (main, sub) = nodes_of(&place);
            let matches: Vec<&String> = std::iter::once(&main)
                .chain(sub.as_ref())
                .filter(|id| wants(id))
                .collect();
            let Some(node_id) = matches.first() else {
                continue;
            };
            if matches.len() > 1 {
                return Err(Error::invalid(t("nucleo-relatorios-erro-nos-sobrepostos")));
            }
            let unit = units.get(&place).copied().unwrap_or(0);
            let (counted, note) = match amount {
                None => (false, Some(no_fx_note(&row.currency, self.base))),
                Some(_) if unit <= 0 => (
                    false,
                    Some(t_args(
                        "nucleo-relatorios-fluxo-grupo-negativo",
                        &args(&[
                            ("grupo", self.unit_label(&place).into()),
                            ("valor", money(unit, self.base).into()),
                        ]),
                    )),
                ),
                Some(_) => (true, None),
            };
            let info = row.category_id.as_deref().and_then(|c| self.tree.get(c));
            out.push(SankeyItem {
                txn_id: Some(row.txn_id),
                node_id: (*node_id).clone(),
                component: None,
                date: Some(row.date.clone()),
                due_date: (row.eff != row.date).then(|| row.eff.clone()),
                label: txn_label(&row.description, row.payee.as_deref()),
                account: Some(row.account),
                category_id: info.map(|c| c.id.clone()),
                group: info.map(|c| c.path.clone()).unwrap_or_else(no_category),
                amount: amount.unwrap_or(0),
                original_amount: (row.currency != self.base).then_some(sign * row.amount),
                original_currency: (row.currency != self.base).then_some(row.currency),
                split: row.split,
                counted,
                note,
                estimated,
            });
        }
        Ok(out)
    }

    /// Aportes (nó `investment`) e resgates (`redemption`), um por lançamento.
    fn investment_items(
        &self,
        from: &str,
        to: &str,
        fx: &mut Fx<'_>,
        wants: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<SankeyItem>> {
        let mut out = vec![];
        for leg in investment_legs(self.conn, from, to)? {
            let node = if leg.out { "investment" } else { "redemption" };
            if !wants(node) {
                continue;
            }
            let sign = if leg.out { -1 } else { 1 };
            let conv = fx.convert(leg.amount, &leg.currency, &leg.date)?;
            let foreign = leg.currency != self.base;
            out.push(SankeyItem {
                txn_id: Some(leg.txn_id),
                node_id: node.into(),
                component: None,
                date: Some(leg.date.clone()),
                due_date: None,
                label: txn_label(&leg.description, leg.payee.as_deref()),
                account: Some(leg.account.clone()),
                category_id: None,
                group: leg.other_account.unwrap_or(leg.account),
                amount: conv.as_ref().map_or(0, |c| sign * c.amount),
                original_amount: foreign.then_some(sign * leg.amount),
                original_currency: foreign.then_some(leg.currency.clone()),
                split: false,
                counted: conv.is_some(),
                note: conv.is_none().then(|| no_fx_note(&leg.currency, self.base)),
                estimated: conv.as_ref().is_some_and(|c| c.estimated),
            });
        }
        Ok(out)
    }

    /// Nome do grupo que decide se um valor entra no diagrama.
    fn unit_label(&self, place: &Place) -> String {
        match place {
            Place::Income(None) => t("nucleo-relatorios-fluxo-receitas-sem-categoria"),
            Place::Income(root) => t_args(
                "nucleo-relatorios-fluxo-receita-de",
                &args(&[("categoria", self.name_of(root).into())]),
            ),
            Place::Debt(cat) => t_args(
                "nucleo-relatorios-fluxo-pagamento-de",
                &args(&[(
                    "contrato",
                    self.debts.get(cat).cloned().unwrap_or_default().into(),
                )]),
            ),
            Place::Expense { root: None, .. } => no_category(),
            Place::Expense { root, sub: None } => without_sub(&self.name_of(root)),
            Place::Expense { sub, .. } => self.name_of(sub),
        }
    }

    /// Nome e tipo de um nó que não aparece em nenhum dos dois períodos.
    fn describe(&self, id: &str) -> (String, SankeyNodeKind) {
        let cat = |rest: &str| (rest != "none").then(|| rest.to_string());
        if let Some(rest) = id.strip_prefix("income:") {
            let name = match cat(rest) {
                None => t("nucleo-relatorios-fluxo-receitas-sem-categoria"),
                c => self.name_of(&c),
            };
            return (name, SankeyNodeKind::Income);
        }
        if let Some(rest) = id.strip_prefix("expense:") {
            return (self.name_of(&cat(rest)), SankeyNodeKind::Expense);
        }
        if let Some(rest) = id.strip_prefix("sub:") {
            let name = match self.tree.get(rest) {
                Some(c) if c.depth == 0 => without_sub(&c.name),
                _ => self.name_of(&cat(rest)),
            };
            return (name, SankeyNodeKind::Subcategory);
        }
        if let Some(rest) = id.strip_prefix("debt:") {
            let name = self.debts.get(rest).cloned().unwrap_or_default();
            return (name, SankeyNodeKind::Subcategory);
        }
        match id {
            "debt" => (t("nucleo-relatorios-fluxo-dividas"), SankeyNodeKind::Debt),
            "investment" => (
                t("nucleo-relatorios-fluxo-investimentos"),
                SankeyNodeKind::Investment,
            ),
            "redemption" => (
                t("nucleo-relatorios-fluxo-resgates"),
                SankeyNodeKind::Redemption,
            ),
            "deficit" => (
                t("nucleo-relatorios-fluxo-deficit"),
                SankeyNodeKind::Deficit,
            ),
            _ => (t("nucleo-relatorios-fluxo-sobra"), SankeyNodeKind::Savings),
        }
    }
}

/// Sobra (fontes − destinos) ou déficit (destinos − fontes): um item por nó ligado ao
/// orçamento. `None` quando o nó não existe no período.
fn derived_items(graph: &Graph, id: &str) -> Option<Vec<SankeyItem>> {
    graph.values.get(id)?;
    let sign = if id == "savings" { 1 } else { -1 };
    let node = |nid: &str| graph.nodes.iter().find(|n| n.id == nid);
    let mut out = vec![];
    for l in &graph.links {
        let (component, amount, group) = if l.target == "hub" && l.source != id {
            (
                &l.source,
                sign * l.value,
                "nucleo-relatorios-fluxo-entradas",
            )
        } else if l.source == "hub" && l.target != id {
            (&l.target, -sign * l.value, "nucleo-relatorios-fluxo-saidas")
        } else {
            continue;
        };
        let n = node(component);
        out.push(SankeyItem {
            txn_id: None,
            node_id: id.to_string(),
            component: Some(component.clone()),
            date: None,
            due_date: None,
            label: n
                .map(|n| n.name.clone())
                .unwrap_or_else(|| component.clone()),
            account: None,
            category_id: n.and_then(|n| n.category_id.clone()),
            group: t(group),
            amount,
            original_amount: None,
            original_currency: None,
            split: false,
            counted: true,
            note: None,
            estimated: false,
        });
    }
    Some(out)
}

/// Nota do item sem câmbio na data.
fn no_fx_note(from: &str, to: &str) -> String {
    t_args(
        "nucleo-relatorios-fluxo-sem-cambio",
        &args(&[
            ("de", from.to_string().into()),
            ("para", to.to_string().into()),
        ]),
    )
}

fn absent_note(graph: &Graph, id: &str, base: &str) -> String {
    match (
        id,
        graph.values.get(if id == "savings" {
            "deficit"
        } else {
            "savings"
        }),
    ) {
        ("savings", Some(v)) => t_args(
            "nucleo-relatorios-fluxo-sem-sobra-deficit",
            &args(&[("valor", money(*v, base).into())]),
        ),
        ("savings", None) => t("nucleo-relatorios-fluxo-sem-sobra"),
        (_, Some(v)) => t_args(
            "nucleo-relatorios-fluxo-sem-deficit-sobra",
            &args(&[("valor", money(*v, base).into())]),
        ),
        _ => t("nucleo-relatorios-fluxo-sem-deficit"),
    }
}
