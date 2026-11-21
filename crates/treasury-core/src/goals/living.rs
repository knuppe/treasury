//! Gastos anuais (IPVA, IPTU, seguro, material escolar, anuidades) no custo de vida: as
//! categorias de despesa que aparecem em poucos meses de um ano viram uma provisão mensal
//! (total do ano ÷ 12). A reserva de emergência usa essa provisão no lugar dos picos.
//!
//! Só regras sobre o histórico: precisa de 12 meses completos de lançamentos; com menos, a
//! lista fica vazia e `complete_year` diz por quê.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::categories::CategoryKind;
use crate::error::Result;
use crate::ids::Id;
use crate::reports::engine::{
    self, CategoryTree, FlowQuery, Fx, Key, complete_months_before, div_round, first_flow_month,
    month_dates,
};

/// Meses olhados para achar os gastos anuais.
pub const YEAR_MONTHS: u32 = 12;
/// Uma categoria com gasto em até tantos meses do ano conta como gasto anual.
pub const MAX_MONTHS_WITH_SPEND: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualExpense {
    pub category_id: Id,
    /// `Transporte › IPVA`.
    pub category: String,
    /// Essencial (grupos `essential` e `debt`): entra na reserva de emergência.
    pub essential: bool,
    /// Gasto nos 12 meses.
    #[ts(type = "number")]
    pub total: i64,
    /// Meses (`YYYY-MM`) em que houve gasto.
    pub months: Vec<String>,
    /// total ÷ 12.
    #[ts(type = "number")]
    pub monthly: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualExpenses {
    /// Do maior para o menor.
    pub items: Vec<AnnualExpense>,
    /// Provisão mensal de todos os gastos anuais.
    #[ts(type = "number")]
    pub monthly: i64,
    /// Provisão mensal só dos essenciais (a que entra na reserva).
    #[ts(type = "number")]
    pub essential_monthly: i64,
    /// Meses analisados (`YYYY-MM`).
    pub months: Vec<String>,
    /// Há 12 meses completos de histórico (sem isso, nada é detectado).
    pub complete_year: bool,
}

impl AnnualExpenses {
    /// Categorias essenciais tratadas como anuais.
    pub fn essential_ids(&self) -> BTreeSet<&str> {
        self.items
            .iter()
            .filter(|i| i.essential)
            .map(|i| i.category_id.as_str())
            .collect()
    }
}

/// Gastos anuais dos últimos 12 meses completos, na moeda de `fx`.
pub(crate) fn annual_expenses(
    conn: &Connection,
    fx: &mut Fx<'_>,
    basis: crate::budget::Basis,
    today: chrono::NaiveDate,
) -> Result<AnnualExpenses> {
    let months = match first_flow_month(conn)? {
        Some(first) => {
            complete_months_before(&engine::current_month_of(today), YEAR_MONTHS, Some(&first))?
        }
        None => vec![],
    };
    let mut out = AnnualExpenses {
        complete_year: months.len() as u32 == YEAR_MONTHS,
        months: months.clone(),
        ..AnnualExpenses::default()
    };
    if !out.complete_year {
        return Ok(out);
    }
    let (from, _) = month_dates(&months[0])?;
    let (_, to) = month_dates(months.last().expect("12 meses"))?;
    let mut q = FlowQuery::new(&from, &to, basis);
    q.kind = Some(CategoryKind::Expense);
    let agg = q.aggregate(conn, fx, Key::Category, Key::Month)?;
    let mut by_cat: BTreeMap<String, Vec<(String, i64)>> = BTreeMap::new();
    for ((cat, month), cell) in &agg.cells {
        if let (Some(c), Some(m)) = (cat, month)
            && cell.expense > 0
        {
            by_cat
                .entry(c.clone())
                .or_default()
                .push((m.clone(), cell.expense));
        }
    }
    let tree = CategoryTree::load(conn)?;
    for (cat, list) in by_cat {
        if list.len() as u32 > MAX_MONTHS_WITH_SPEND {
            continue;
        }
        let Some(info) = tree.get(&cat) else { continue };
        let total: i64 = list.iter().map(|(_, v)| v).sum();
        out.items.push(AnnualExpense {
            category_id: cat.clone(),
            category: info.path.clone(),
            essential: info.is_essential,
            total,
            months: list.into_iter().map(|(m, _)| m).collect(),
            monthly: div_round(total, YEAR_MONTHS as i64),
        });
    }
    out.items
        .sort_by(|a, b| b.total.cmp(&a.total).then(a.category.cmp(&b.category)));
    let all: i64 = out.items.iter().map(|i| i.total).sum();
    let essential: i64 = out
        .items
        .iter()
        .filter(|i| i.essential)
        .map(|i| i.total)
        .sum();
    out.monthly = div_round(all, YEAR_MONTHS as i64);
    out.essential_monthly = div_round(essential, YEAR_MONTHS as i64);
    Ok(out)
}
