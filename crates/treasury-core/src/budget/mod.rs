//! Orçamento mensal por categoria, previsto × realizado, por competência ou caixa.

pub mod api;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap};

use chrono::Datelike;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::categories::CategoryKind;
use crate::error::{Error, Result};
use crate::household::base_currency;
use crate::i18n::{self, t, t_args};
use crate::ids::{Id, new_id};
use crate::reports::engine::{
    self, CategoryTree, FlowQuery, Fx, Key, Status, complete_months_before, first_flow_month,
    month_dates, month_index, month_label, normalize_month, ratio, resolve_basis,
};
use crate::time::now_utc;

/// Base de datas para orçamento e relatórios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Basis {
    /// Competência: data da compra.
    #[default]
    Accrual,
    /// Caixa: data do pagamento (vencimento da fatura para compras no cartão).
    Cash,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BudgetLineStatus {
    pub category_id: Id,
    pub category_name: String,
    pub category_path: String,
    pub parent_id: Option<Id>,
    pub kind: CategoryKind,
    /// Previsto (positivo). Para categorias-pai sem linha própria, soma dos filhos.
    #[ts(type = "number")]
    pub planned: i64,
    /// Realizado (positivo = gasto, para despesas; recebido, para receitas). Inclui subcategorias.
    #[ts(type = "number")]
    pub actual: i64,
    /// Previsto − realizado.
    #[ts(type = "number")]
    pub remaining: i64,
    /// Realizado ÷ previsto (null sem previsto).
    pub ratio: Option<f64>,
    /// Lançamentos previstos ainda não efetivados no mês (comprometido).
    #[ts(type = "number")]
    pub committed: i64,
    pub has_own_line: bool,
    pub color: Option<String>,
    /// Categoria arquivada (aparece enquanto tiver valores no mês).
    pub archived: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BudgetTotals {
    /// Soma do previsto das categorias de primeiro nível de despesa.
    #[ts(type = "number")]
    pub planned_expense: i64,
    /// Todas as despesas realizadas do mês (inclusive sem categoria).
    #[ts(type = "number")]
    pub actual_expense: i64,
    #[ts(type = "number")]
    pub planned_income: i64,
    #[ts(type = "number")]
    pub actual_income: i64,
    /// Despesas previstas ainda não efetivadas no mês.
    #[ts(type = "number")]
    pub committed_expense: i64,
    #[ts(type = "number")]
    pub committed_income: i64,
    /// Despesas realizadas sem categoria (só entram no total).
    #[ts(type = "number")]
    pub uncategorized_expense: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BudgetStatus {
    pub month: String,
    pub basis: Basis,
    /// Uma linha por categoria com previsto ou realizado, pais antes dos filhos.
    pub lines: Vec<BudgetLineStatus>,
    pub totals: BudgetTotals,
    /// Fração do mês já decorrida (0–1), para comparar o ritmo do gasto.
    pub month_elapsed: f64,
    pub currency: String,
    /// Moedas sem câmbio (valores fora do total).
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BudgetGetArgs {
    pub month: String,
    /// Padrão: o ajuste `budget_basis`.
    #[ts(optional)]
    pub basis: Option<Basis>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BudgetLineInput {
    pub category_id: Id,
    /// Positivo; 0 remove a linha.
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetBudgetArgs {
    pub month: String,
    pub lines: Vec<BudgetLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CopyBudgetArgs {
    pub from_month: String,
    pub to_month: String,
    /// Sobrescrever linhas já existentes no destino.
    #[ts(optional)]
    pub overwrite: Option<bool>,
}

/// Sugestão de orçamento: média do gasto de cada categoria de primeiro nível nos meses
/// completos anteriores, arredondada para cima ao múltiplo de R$ 10.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SuggestBudgetArgs {
    pub month: String,
    /// Meses anteriores para a média (padrão 3).
    #[ts(optional)]
    pub months_back: Option<u32>,
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// `average` (padrão) ou `median`: a mediana dos meses corta um mês fora da curva.
    #[serde(default)]
    #[ts(optional)]
    pub method: Option<SuggestMethod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SuggestMethod {
    Average,
    Median,
}

/// Mediana de valores (par: a média dos dois do meio).
fn median(mut v: Vec<i64>) -> i64 {
    v.sort_unstable();
    let n = v.len();
    match n {
        0 => 0,
        _ if n % 2 == 1 => v[n / 2],
        _ => (v[n / 2 - 1] + v[n / 2]) / 2,
    }
}

/// Fração do mês decorrida em relação a hoje (meses passados = 1, futuros = 0).
fn month_elapsed(month: &str) -> Result<f64> {
    let today = engine::today();
    let (y, m) = crate::time::parse_month(month)?;
    let cur = today.year() * 12 + today.month0() as i32;
    let this = y * 12 + m as i32 - 1;
    Ok(if this < cur {
        1.0
    } else if this > cur {
        0.0
    } else {
        today.day() as f64 / crate::time::days_in_month(y, m) as f64
    })
}

/// Linhas do orçamento do mês: categoria → valor.
fn lines_of(conn: &Connection, month: &str) -> Result<BTreeMap<Id, i64>> {
    let mut stmt = conn.prepare_cached(
        "SELECT bl.category_id, bl.amount FROM budget_line bl JOIN budget b ON b.id = bl.budget_id
         WHERE b.month = ?1",
    )?;
    let rows = stmt
        .query_map([month], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Previsto × realizado do mês. Realizado e comprometido somam as subcategorias; categorias
/// com gasto e sem orçamento também aparecem; pais sem linha própria somam o previsto dos
/// filhos (uma linha no pai vale para o pai e todas as subcategorias).
pub fn get(conn: &Connection, month: &str, basis: Option<Basis>) -> Result<BudgetStatus> {
    let month = normalize_month(month)?;
    let basis = resolve_basis(conn, basis)?;
    let (from, to) = month_dates(&month)?;
    let base = base_currency(conn)?;
    let tree = CategoryTree::load(conn)?;
    let own_lines = lines_of(conn, &month)?;

    let mut fx = Fx::new(conn, &base);
    let mut q = FlowQuery::new(&from, &to, basis);
    let realized = q.aggregate(conn, &mut fx, Key::Category, Key::None)?;
    q.status = Status::Planned;
    let planned_txns = q.aggregate(conn, &mut fx, Key::Category, Key::None)?;

    // Realizado e comprometido de cada categoria, somados nos ancestrais. Cada categoria conta
    // só os lançamentos do seu tipo (despesa numa categoria de despesa, receita na de receita).
    let mut actual: HashMap<Id, i64> = HashMap::new();
    let mut committed: HashMap<Id, i64> = HashMap::new();
    for (agg, target) in [(&realized, &mut actual), (&planned_txns, &mut committed)] {
        for ((cat, _), cell) in &agg.cells {
            let Some(info) = cat.as_deref().and_then(|c| tree.get(c)) else {
                continue;
            };
            let v = cell.amount(info.kind);
            if v == 0 {
                continue;
            }
            for a in tree.ancestors_inclusive(&info.id) {
                if a.kind == info.kind {
                    *target.entry(a.id.clone()).or_default() += v;
                }
            }
        }
    }
    // Previsto: a linha própria ou, sem ela, a soma do previsto dos filhos (de baixo para cima).
    let mut planned: HashMap<Id, i64> = HashMap::new();
    for id in tree.order.iter().rev() {
        let v = match own_lines.get(id) {
            Some(v) => *v,
            None => tree
                .children_of(id)
                .iter()
                .map(|c| planned.get(c).copied().unwrap_or(0))
                .sum(),
        };
        if v != 0 {
            planned.insert(id.clone(), v);
        }
    }

    let mut lines = vec![];
    let mut totals = BudgetTotals::default();
    for id in &tree.order {
        let info = tree.get(id).expect("categoria da árvore");
        let p = planned.get(id).copied().unwrap_or(0);
        let a = actual.get(id).copied().unwrap_or(0);
        let c = committed.get(id).copied().unwrap_or(0);
        let has_own_line = own_lines.contains_key(id);
        if p == 0 && a == 0 && c == 0 && !has_own_line {
            continue;
        }
        if info.depth == 0 {
            match info.kind {
                CategoryKind::Expense => totals.planned_expense += p,
                CategoryKind::Income => totals.planned_income += p,
            }
        }
        lines.push(BudgetLineStatus {
            category_id: id.clone(),
            category_name: info.name.clone(),
            category_path: info.path.clone(),
            parent_id: info.parent_id.clone(),
            kind: info.kind,
            planned: p,
            actual: a,
            remaining: p - a,
            ratio: ratio(a, p),
            committed: c,
            has_own_line,
            color: tree.color(id),
            archived: info.archived,
        });
    }
    let r = realized.total();
    let pl = planned_txns.total();
    totals.actual_expense = r.expense;
    totals.actual_income = r.income;
    totals.committed_expense = pl.expense;
    totals.committed_income = pl.income;
    totals.uncategorized_expense = realized
        .cells
        .get(&(None, None))
        .map(|c| c.expense)
        .unwrap_or(0);
    let mut missing = realized.missing_fx.clone();
    missing.extend(planned_txns.missing_fx.iter().cloned());
    Ok(BudgetStatus {
        month_elapsed: month_elapsed(&month)?,
        month,
        basis,
        lines,
        totals,
        currency: base,
        missing_fx: missing.into_iter().collect(),
        estimated: realized.estimated || planned_txns.estimated,
    })
}

/// Id do orçamento do mês, criando se preciso.
fn ensure_budget(tx: &Transaction<'_>, month: &str) -> Result<Id> {
    if let Some(id) = tx
        .query_row("SELECT id FROM budget WHERE month = ?1", [month], |r| {
            r.get::<_, String>(0)
        })
        .optional()?
    {
        return Ok(id);
    }
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO budget (id, month, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, month, now],
    )?;
    Ok(id)
}

fn upsert_line(
    tx: &Transaction<'_>,
    budget_id: &str,
    category_id: &str,
    amount: i64,
) -> Result<()> {
    tx.execute(
        "INSERT INTO budget_line (id, budget_id, category_id, amount, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT (budget_id, category_id) DO UPDATE SET amount = excluded.amount,
           updated_at = excluded.updated_at WHERE budget_line.amount <> excluded.amount",
        params![new_id(), budget_id, category_id, amount, now_utc()],
    )?;
    Ok(())
}

/// Grava linhas do orçamento do mês (cria o orçamento se preciso). Valor 0 remove a linha.
pub fn set_lines(tx: &Transaction<'_>, args: &SetBudgetArgs) -> Result<()> {
    let month = normalize_month(&args.month)?;
    for l in &args.lines {
        if l.amount < 0 {
            return Err(Error::invalid(t("nucleo-orcamento-erro-previsto-negativo")));
        }
        let exists: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM category WHERE id = ?1)",
            [&l.category_id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(Error::not_found(t_args(
                "nucleo-orcamento-categoria",
                &i18n::args(&[("id", l.category_id.clone().into())]),
            )));
        }
    }
    let budget_id: Option<Id> = tx
        .query_row("SELECT id FROM budget WHERE month = ?1", [&month], |r| {
            r.get(0)
        })
        .optional()?;
    let budget_id = match budget_id {
        Some(id) => id,
        None if args.lines.iter().any(|l| l.amount > 0) => ensure_budget(tx, &month)?,
        None => return Ok(()),
    };
    for l in &args.lines {
        if l.amount == 0 {
            tx.execute(
                "DELETE FROM budget_line WHERE budget_id = ?1 AND category_id = ?2",
                params![budget_id, l.category_id],
            )?;
        } else {
            upsert_line(tx, &budget_id, &l.category_id, l.amount)?;
        }
    }
    Ok(())
}

/// Copia as linhas de um mês para outro (categorias arquivadas ficam de fora). Sem
/// `overwrite`, linhas que já existem no destino são mantidas. Devolve quantas linhas gravou.
pub fn copy(tx: &Transaction<'_>, args: &CopyBudgetArgs) -> Result<usize> {
    let from = normalize_month(&args.from_month)?;
    let to = normalize_month(&args.to_month)?;
    if from == to {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meses-iguais")));
    }
    let src = lines_of(tx, &from)?;
    if src.is_empty() {
        return Err(Error::not_found(t_args(
            "nucleo-orcamento-orcamento-de",
            &i18n::args(&[("mes", month_label(&from).into())]),
        )));
    }
    let archived: std::collections::HashSet<String> = {
        let mut stmt = tx.prepare("SELECT id FROM category WHERE archived = 1")?;
        stmt.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let dest = lines_of(tx, &to)?;
    let budget_id = ensure_budget(tx, &to)?;
    let mut n = 0;
    for (cat, amount) in src {
        if archived.contains(&cat) || (!args.overwrite.unwrap_or(false) && dest.contains_key(&cat))
        {
            continue;
        }
        upsert_line(tx, &budget_id, &cat, amount)?;
        n += 1;
    }
    Ok(n)
}

/// Média (ou mediana, com `method`) do gasto de cada categoria de primeiro nível (despesa, não
/// arquivada) nos `months_back` meses completos anteriores ao mês (padrão 3), arredondada para
/// cima ao múltiplo de R$ 10. Meses anteriores ao primeiro lançamento não entram; na mediana,
/// mês sem gasto na categoria conta como zero.
pub fn suggest(conn: &Connection, args: &SuggestBudgetArgs) -> Result<Vec<BudgetLineInput>> {
    let month = normalize_month(&args.month)?;
    let basis = resolve_basis(conn, args.basis)?;
    let n = args.months_back.unwrap_or(3).clamp(1, 24);
    // Só meses completos: antes do mês pedido e antes do mês corrente.
    let before = if month_index(&month)? > month_index(&engine::current_month())? {
        engine::current_month()
    } else {
        month.clone()
    };
    let first = first_flow_month(conn)?;
    let Some(first) = first else {
        return Ok(vec![]);
    };
    let months = complete_months_before(&before, n, Some(&first))?;
    let (Some(a), Some(b)) = (months.first(), months.last()) else {
        return Ok(vec![]);
    };
    let (from, _) = month_dates(a)?;
    let (_, to) = month_dates(b)?;
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let mut q = FlowQuery::new(&from, &to, basis);
    q.kind = Some(CategoryKind::Expense);
    let by_month = args.method == Some(SuggestMethod::Median);
    let agg = q.aggregate(
        conn,
        &mut fx,
        Key::RootCategory,
        if by_month { Key::Month } else { Key::None },
    )?;
    let tree = CategoryTree::load(conn)?;
    let step = 10 * 10i64.pow(crate::money::minor_units(&base));
    let count = months.len() as i64;
    let mut out = vec![];
    for root in &tree.roots {
        let info = tree.get(root).expect("raiz da árvore");
        if info.kind != CategoryKind::Expense || info.archived {
            continue;
        }
        let amount = if by_month {
            let per_month: Vec<i64> = months
                .iter()
                .map(|m| {
                    agg.cells
                        .get(&(Some(root.clone()), Some(m.clone())))
                        .map(|c| c.expense)
                        .unwrap_or(0)
                })
                .collect();
            median(per_month)
        } else {
            let total = agg
                .cells
                .get(&(Some(root.clone()), None))
                .map(|c| c.expense)
                .unwrap_or(0);
            engine::div_ceil(total.max(0), count)
        };
        if amount <= 0 {
            continue;
        }
        out.push(BudgetLineInput {
            category_id: root.clone(),
            amount: engine::div_ceil(amount, step) * step,
        });
    }
    Ok(out)
}

/// Meses com orçamento (com ao menos uma linha), do mais recente ao mais antigo.
pub fn list_months(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT b.month FROM budget b WHERE EXISTS (SELECT 1 FROM budget_line l WHERE l.budget_id = b.id)
         ORDER BY b.month DESC",
    )?;
    let rows = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Metas por grupo de gasto (para onde vai a renda)

/// Meta de cada grupo de gasto, em fração da renda do mês (0–1); `null` = sem meta. Opcionais e
/// editáveis; o padrão é o guia 50/30/20 (essenciais 50%, não essencial 30%, futuro 20%) e
/// dívidas sem meta (a dívida paga acima do mínimo sai do futuro).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GroupTargets {
    pub essential: Option<f64>,
    pub debt: Option<f64>,
    pub future: Option<f64>,
    pub non_essential: Option<f64>,
}

impl Default for GroupTargets {
    fn default() -> Self {
        GroupTargets {
            essential: Some(0.5),
            debt: None,
            future: Some(0.2),
            non_essential: Some(0.3),
        }
    }
}

impl GroupTargets {
    /// Meta de um grupo.
    pub fn of(&self, g: crate::categories::SpendGroup) -> Option<f64> {
        use crate::categories::SpendGroup::*;
        match g {
            Essential => self.essential,
            Debt => self.debt,
            Future => self.future,
            NonEssential => self.non_essential,
        }
    }
}

/// Ajuste do cofre com as metas por grupo (sem ele, [`GroupTargets::default`]).
const GROUP_TARGETS_SETTING: &str = "budget.group_targets";

/// Metas por grupo de gasto do cofre.
pub fn group_targets(conn: &Connection) -> Result<GroupTargets> {
    Ok(crate::settings::get(conn, GROUP_TARGETS_SETTING)?.unwrap_or_default())
}

/// Grava as metas por grupo (ajuste do cofre, fora do histórico, como os outros ajustes). Cada
/// meta vai de 0 a 1; `null` tira a meta do grupo.
pub fn set_group_targets(conn: &Connection, targets: &GroupTargets) -> Result<GroupTargets> {
    let all = [
        targets.essential,
        targets.debt,
        targets.future,
        targets.non_essential,
    ];
    if all.iter().flatten().any(|v| !(0.0..=1.0).contains(v)) {
        return Err(Error::invalid(t("nucleo-orcamento-erro-meta-grupo")));
    }
    crate::settings::set(conn, GROUP_TARGETS_SETTING, targets)?;
    group_targets(conn)
}
