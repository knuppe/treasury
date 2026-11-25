//! Relatórios e dados das visualizações (painel, Sankey, treemap, calendário, evolução,
//! comparação, titulares, patrimônio). Toda medida pode ser explicada (`explain`).
//!
//! Regras de fluxo de caixa: só `income` e `expense` contam; transferências, pagamento de
//! fatura e aportes não. O gasto do cartão conta na compra (competência) ou no vencimento
//! da fatura (caixa). Divisões distribuem o valor entre categorias/titulares. Estornos
//! (despesa positiva) reduzem o gasto. Valores convertidos para a moeda base. O Sankey
//! mostra aportes e resgates de investimentos à parte, fora de receitas e despesas.

pub mod api;
pub(crate) mod engine;
mod explain;
#[cfg(test)]
pub(crate) mod fixture;
pub mod inflation;
pub mod member;
pub mod pdf_report;
pub(crate) mod pdf_writer;
mod sankey;
pub mod spend_groups;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_pdf;
#[cfg(test)]
mod tests_sankey_items;
#[cfg(test)]
mod tests_spend_groups;

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::budget::Basis;
use crate::categories::CategoryKind;
use crate::error::{Error, Result};
use crate::household::base_currency;
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, parse_date};
use engine::{
    CategoryTree, FlowQuery, Fx, Key, MAX_MONTHS, balances_at, current_month, first_flow_month,
    month_dates, month_index, month_label, months_between, normalize_month, ratio, resolve_basis,
    shift_month, today, validate_range,
};

pub use explain::explain;
pub use sankey::{
    SANKEY_DEFAULT_DEPTH, SankeyArgs, SankeyData, SankeyDelta, SankeyItem, SankeyItems,
    SankeyItemsArgs, SankeyItemsGroup, SankeyItemsPeriod, SankeyLink, SankeyNode, SankeyNodeKind,
    SankeyTotals, previous_period, sankey, sankey_items, sankey_with,
};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Period {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AccountBalanceItem {
    pub account_id: Id,
    pub name: String,
    pub kind: crate::accounts::AccountKind,
    pub currency: String,
    #[ts(type = "number")]
    pub balance: i64,
    #[ts(type = "number | null")]
    pub balance_base: Option<i64>,
    pub fx_estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NetWorth {
    pub as_of: String,
    pub base_currency: String,
    #[ts(type = "number")]
    pub assets: i64,
    /// Positivo (valor devido).
    #[ts(type = "number")]
    pub liabilities: i64,
    #[ts(type = "number")]
    pub net: i64,
    /// Valor de mercado da carteira de investimentos (já incluído em `assets`).
    #[ts(type = "number")]
    pub investments: i64,
    /// Valor dos bens físicos (imóveis, veículos…; já incluído em `assets`). Os
    /// financiamentos ligados a eles entram em `debts`.
    #[ts(type = "number")]
    pub physical: i64,
    /// Saldo devedor de empréstimos e financiamentos (já incluído em `liabilities`).
    #[ts(type = "number")]
    pub debts: i64,
    pub accounts: Vec<AccountBalanceItem>,
    /// Moedas sem câmbio disponível (ficaram fora do total).
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthFlow {
    pub month: String,
    #[ts(type = "number")]
    pub income: i64,
    /// Positivo.
    #[ts(type = "number")]
    pub expense: i64,
    #[ts(type = "number")]
    pub net: i64,
    /// (receitas − despesas) ÷ receitas; null sem receitas.
    pub savings_rate: Option<f64>,
    /// Moedas sem câmbio (valores fora do total).
    pub missing_fx: Vec<String>,
    /// Alguma conversão usou câmbio estimado.
    pub estimated: bool,
    /// O mês em camadas: consumo, parcelas de contrato, aportes e resgates, ajustes de saldo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub bridge: Option<MonthBridge>,
}

/// Ponte do mês (fechamento): a despesa separada em consumo e parcelas de dívida (com juros e
/// amortização do cronograma), o que foi para investimentos e voltou deles, e os ajustes de
/// sincronia de saldo, que ficam fora do resultado. Centavos, positivos (ajustes com sinal).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthBridge {
    /// Despesas fora das parcelas de contrato.
    #[ts(type = "number")]
    pub consumption: i64,
    /// Lançamentos que pagaram parcelas de dívida (em qualquer categoria).
    #[ts(type = "number")]
    pub debt_service: i64,
    /// Do cronograma das parcelas pagas no mês.
    #[ts(type = "number")]
    pub debt_interest: i64,
    #[ts(type = "number")]
    pub debt_amortization: i64,
    /// Aportes (lançamentos do tipo investimento que saíram) e resgates (que entraram).
    #[ts(type = "number")]
    pub invested: i64,
    #[ts(type = "number")]
    pub redeemed: i64,
    /// Ajustes de sincronia de saldo (com sinal): não são receita nem consumo.
    #[ts(type = "number")]
    pub adjustments: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum UpcomingKind {
    PlannedTxn,
    CardDue,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpcomingItem {
    pub date: String,
    pub kind: UpcomingKind,
    pub description: String,
    /// Com sinal (− sai, + entra). Faturas: −(valor a pagar).
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    pub account_id: Id,
    pub account_name: String,
    /// Lançamento previsto ou fatura.
    pub ref_id: Id,
    /// Recorrência que gerou o previsto (para quem pediu para não ser avisado dela).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub recurrence_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GroupAmount {
    /// Id do grupo (categoria, titular, conta) ou `YYYY-MM`; `null` = sem categoria/titular.
    pub key: Option<String>,
    pub label: String,
    pub parent_key: Option<String>,
    /// Caminho completo da categoria (`Alimentação › Mercado`), quando o grupo é categoria.
    pub path: Option<String>,
    #[ts(type = "number")]
    pub amount: i64,
    /// Fração do total (0–1).
    pub share: f64,
    /// Lançamentos no grupo (cada lançamento dividido conta uma vez por grupo).
    #[ts(type = "number")]
    pub count: i64,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GroupBy {
    /// Categoria-folha (com `parent_key` para montar treemap/sunburst).
    #[default]
    Category,
    /// Só categorias de primeiro nível.
    ParentCategory,
    Member,
    Account,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendingArgs {
    pub from: String,
    pub to: String,
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// `expense` (padrão) ou `income`.
    #[ts(optional)]
    pub kind: Option<crate::categories::CategoryKind>,
    #[ts(optional)]
    pub group_by: Option<GroupBy>,
    #[ts(optional)]
    pub member_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub account_ids: Option<Vec<Id>>,
    #[ts(optional)]
    pub category_ids: Option<Vec<Id>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendingSummary {
    pub from: String,
    pub to: String,
    pub basis: Basis,
    pub kind: CategoryKind,
    pub currency: String,
    #[ts(type = "number")]
    pub total: i64,
    /// Lançamentos distintos no período (um dividido conta uma vez).
    #[ts(type = "number")]
    pub count: i64,
    pub groups: Vec<GroupAmount>,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CalendarDay {
    pub date: String,
    #[ts(type = "number")]
    pub expense: i64,
    #[ts(type = "number")]
    pub income: i64,
    #[ts(type = "number")]
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CalendarData {
    pub from: String,
    pub to: String,
    pub basis: Basis,
    /// Um ponto por dia do período, inclusive dias sem movimento.
    pub days: Vec<CalendarDay>,
    #[ts(type = "number")]
    pub max_expense: i64,
    #[ts(type = "number")]
    pub total_expense: i64,
    #[ts(type = "number")]
    pub total_income: i64,
    pub currency: String,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EvolutionSeries {
    /// Id da categoria; `null` = sem categoria; `"others"` = "Outras" (evolução mensal).
    pub key: Option<String>,
    pub label: String,
    pub color: Option<String>,
    #[ts(type = "number[]")]
    pub values: Vec<i64>,
}

/// Barras empilhadas por categoria, mês a mês, com média móvel de 3 meses.
/// `series`: as N categorias de primeiro nível com mais gasto no período (e "Outras").
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthlyEvolution {
    pub months: Vec<String>,
    pub basis: Basis,
    pub series: Vec<EvolutionSeries>,
    #[ts(type = "number[]")]
    pub totals: Vec<i64>,
    /// Média do gasto do mês e dos dois anteriores (usa meses antes da janela quando há dados).
    #[ts(type = "number[]")]
    pub moving_avg_3: Vec<i64>,
    #[ts(type = "number[]")]
    pub income: Vec<i64>,
    pub currency: String,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComparisonRow {
    pub key: Option<String>,
    pub label: String,
    #[ts(type = "number")]
    pub a: i64,
    #[ts(type = "number")]
    pub b: i64,
    /// b − a.
    #[ts(type = "number")]
    pub delta: i64,
    /// (b − a) ÷ a; null quando a ≤ 0.
    pub delta_ratio: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PeriodComparison {
    pub a: Period,
    pub b: Period,
    pub basis: Basis,
    pub kind: CategoryKind,
    /// Categorias de primeiro nível, da maior variação absoluta para a menor.
    pub rows: Vec<ComparisonRow>,
    #[ts(type = "number")]
    pub total_a: i64,
    #[ts(type = "number")]
    pub total_b: i64,
    pub currency: String,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberBreakdown {
    pub from: String,
    pub to: String,
    /// Colunas: titulares (último = sem titular).
    pub members: Vec<GroupAmount>,
    /// Linhas: categorias de primeiro nível com o valor de cada titular, na ordem de `members`.
    pub categories: Vec<EvolutionSeries>,
    pub basis: Basis,
    #[ts(type = "number")]
    pub total: i64,
    pub currency: String,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NetWorthPoint {
    pub month: String,
    /// Último dia do mês (data dos saldos).
    pub as_of: String,
    #[ts(type = "number")]
    pub assets: i64,
    #[ts(type = "number")]
    pub liabilities: i64,
    #[ts(type = "number")]
    pub net: i64,
    /// Valor de mercado da carteira no fim do mês (já incluído em `assets`).
    #[ts(type = "number")]
    pub investments: i64,
    /// Valor dos bens físicos no fim do mês (já incluído em `assets`).
    #[ts(type = "number")]
    pub physical: i64,
    /// Saldo devedor de empréstimos no fim do mês (já incluído em `liabilities`).
    #[ts(type = "number")]
    pub debts: i64,
    pub missing_fx: Vec<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Dashboard {
    pub month: String,
    pub basis: Basis,
    pub net_worth: NetWorth,
    pub flow: MonthFlow,
    /// 12 meses terminando em `month`.
    pub flow_history: Vec<MonthFlow>,
    /// Taxa de poupança dos últimos 12 meses.
    pub savings_rate_12m: Option<f64>,
    /// Até 5 categorias de primeiro nível com mais gasto no mês.
    pub top_categories: Vec<GroupAmount>,
    /// Próximos 30 dias a partir de hoje.
    pub upcoming: Vec<UpcomingItem>,
    /// Previsto de despesas do orçamento do mês (null sem orçamento).
    #[ts(type = "number | null")]
    pub budget_planned: Option<i64>,
    /// Despesas realizadas no mês.
    #[ts(type = "number")]
    pub budget_actual: i64,
    /// Lançamentos de receita/despesa do mês sem categoria (ou com parte sem categoria).
    #[ts(type = "number")]
    pub uncategorized_count: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SourceKind {
    Txn,
    Account,
    FxRate,
    Opening,
    Index,
    /// Valor de mercado da carteira de investimentos.
    Portfolio,
    /// Saldo devedor de empréstimo ou financiamento.
    Loan,
    /// Valor de um bem físico (avaliação ou estimativa).
    Physical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ValueSource {
    pub kind: SourceKind,
    pub id: Option<Id>,
    pub label: String,
    pub date: Option<String>,
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    /// Fonte externa (ex.: `manual`, `bcb_ptax_venda`).
    pub source: Option<String>,
    pub estimated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Metric {
    /// `params`: {account_id, as_of?}
    AccountBalance,
    /// `params`: {as_of?}
    NetWorth,
    /// `params`: {from, to, basis?, category_id?, member_id?}
    Spending,
    /// `params`: {from, to, basis?}
    Income,
    /// `params`: {month, basis?}. `value` = receitas − despesas; `ratio` = taxa de poupança.
    SavingsRate,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExplainArgs {
    pub metric: Metric,
    #[ts(type = "Record<string, unknown>")]
    pub params: serde_json::Value,
    /// Máximo de lançamentos listados (padrão 200, mais recentes primeiro). Saldo inicial,
    /// contas e taxas de câmbio são sempre listados.
    #[ts(optional, type = "number")]
    pub limit: Option<u32>,
}

/// "Medido, nunca inventado": de onde vem um número.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Explanation {
    pub metric: Metric,
    #[ts(type = "number")]
    pub value: i64,
    pub currency: String,
    /// Fórmula em português, ex.: "saldo inicial + 132 lançamentos efetivados até 14/10/2026".
    pub formula: String,
    /// Soma dos `amount` das fontes = `value` (quando `truncated` é falso). Fontes de câmbio
    /// têm `amount` 0: só informam a taxa usada.
    pub sources: Vec<ValueSource>,
    /// Há mais fontes do que as listadas.
    pub truncated: bool,
    pub estimated: bool,
    /// Razão exibida junto do valor (taxa de poupança).
    pub ratio: Option<f64>,
    /// Lançamentos (ou contas) considerados.
    #[ts(type = "number")]
    pub count: i64,
    /// Moedas sem câmbio (valores fora do total).
    pub missing_fx: Vec<String>,
}

// ---------------------------------------------------------------------------
// Argumentos dos comandos

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DashboardArgs {
    /// Padrão: mês corrente.
    #[ts(optional)]
    pub month: Option<String>,
    /// Padrão: o ajuste `budget_basis`.
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// Membro da família (docs/members.md): só o que é dele. Padrão: a família.
    #[ts(optional)]
    pub member_id: Option<Id>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NetWorthArgs {
    /// Padrão: hoje.
    #[ts(optional)]
    pub as_of: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NetWorthHistoryArgs {
    /// Meses (padrão 12).
    #[ts(optional)]
    pub months: Option<u32>,
    /// Padrão: mês corrente.
    #[ts(optional)]
    pub end_month: Option<String>,
    /// Membro da família (docs/members.md): só o que é dele. Padrão: a família.
    #[ts(optional)]
    pub member_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthFlowArgs {
    pub from_month: String,
    pub to_month: String,
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// Membro da família (docs/members.md): só o que é dele. Padrão: a família.
    #[ts(optional)]
    pub member_id: Option<Id>,
}

/// Período com base de datas (Sankey, calendário, titulares).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PeriodArgs {
    pub from: String,
    pub to: String,
    #[ts(optional)]
    pub basis: Option<Basis>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthlyEvolutionArgs {
    /// Meses (padrão 12).
    #[ts(optional)]
    pub months: Option<u32>,
    /// Padrão: mês corrente.
    #[ts(optional)]
    pub end_month: Option<String>,
    /// Categorias com série própria (padrão 6); as demais vão para "Outras".
    #[ts(optional)]
    pub top_n: Option<u32>,
    #[ts(optional)]
    pub basis: Option<Basis>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompareArgs {
    pub a: Period,
    pub b: Period,
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// `expense` (padrão) ou `income`.
    #[ts(optional)]
    pub kind: Option<CategoryKind>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpcomingArgs {
    /// Dias à frente (padrão 30).
    #[ts(optional)]
    pub days: Option<u32>,
    /// Início da janela (padrão: hoje).
    #[ts(optional)]
    pub from: Option<String>,
}

// ---------------------------------------------------------------------------
// Rótulos

/// "Sem categoria", no idioma da instalação.
fn no_category() -> String {
    t("nucleo-relatorios-sem-categoria")
}

/// "Sem membro", no idioma da instalação.
fn no_member() -> String {
    t("nucleo-relatorios-sem-membro")
}

const OTHERS_KEY: &str = "others";

struct MemberLabel {
    name: String,
    color: Option<String>,
    archived: bool,
    sort_order: i64,
}

struct Labels {
    members: HashMap<Id, MemberLabel>,
    accounts: HashMap<Id, (String, Option<String>)>,
}

impl Labels {
    fn load(conn: &Connection) -> Result<Self> {
        let mut members = HashMap::new();
        let mut stmt =
            conn.prepare_cached("SELECT id, name, color, archived, sort_order FROM member")?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            members.insert(
                r.get::<_, String>(0)?,
                MemberLabel {
                    name: r.get(1)?,
                    color: r.get(2)?,
                    archived: r.get::<_, i64>(3)? != 0,
                    sort_order: r.get(4)?,
                },
            );
        }
        let mut accounts = HashMap::new();
        let mut stmt = conn.prepare_cached("SELECT id, name, color FROM account")?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            accounts.insert(r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?));
        }
        Ok(Self { members, accounts })
    }
}

/// Rótulo, caminho, pai e cor de um grupo de categoria.
fn category_group(
    tree: &CategoryTree,
    key: Option<&str>,
) -> (String, Option<String>, Option<String>, Option<String>) {
    match key.and_then(|k| tree.get(k)) {
        Some(c) => (
            c.name.clone(),
            Some(c.path.clone()),
            c.parent_id.clone(),
            tree.color(&c.id),
        ),
        None => (
            key.map(str::to_string).unwrap_or_else(no_category),
            None,
            None,
            None,
        ),
    }
}

fn share(amount: i64, total: i64) -> f64 {
    if total > 0 {
        amount as f64 / total as f64
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Patrimônio

/// Patrimônio numa data. Ativos = saldos positivos; passivos = saldos negativos (dívida do
/// cartão, cheque especial). Entram as contas com `include_in_net_worth` que já existiam na
/// data (não arquivadas) ou que têm saldo ≠ 0 (arquivadas inclusive).
pub fn net_worth(conn: &Connection, as_of: &str) -> Result<NetWorth> {
    net_worth_scoped(conn, as_of, None)
}

/// Patrimônio da família ou, com `scope`, de um membro (a fração dele nas contas e bens e as
/// dívidas em que é titular).
pub fn net_worth_scoped(
    conn: &Connection,
    as_of: &str,
    scope: Option<&member::MemberScope>,
) -> Result<NetWorth> {
    parse_date(as_of)?;
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let mut out = NetWorth {
        as_of: as_of.to_string(),
        base_currency: base.clone(),
        assets: 0,
        liabilities: 0,
        net: 0,
        investments: 0,
        physical: 0,
        debts: 0,
        accounts: vec![],
        missing_fx: vec![],
        estimated: false,
    };
    for mut a in balances_at(conn, as_of)? {
        if !a.include_in_net_worth || !((!a.archived && a.exists_at(as_of)) || a.balance != 0) {
            continue;
        }
        let share = member::account_share(scope, &a.id);
        if share <= 0.0 {
            continue;
        }
        a.balance = member::scale(a.balance, share);
        let conv = fx.convert(a.balance, &a.currency, as_of)?;
        let (balance_base, fx_estimated) = match &conv {
            Some(c) => (Some(c.amount), c.estimated),
            None => (None, false),
        };
        if let Some(v) = balance_base {
            if v >= 0 {
                out.assets += v;
            } else {
                out.liabilities += -v;
            }
        }
        out.accounts.push(AccountBalanceItem {
            account_id: a.id,
            name: a.name,
            kind: a.kind,
            currency: a.currency,
            balance: a.balance,
            balance_base,
            fx_estimated,
        });
    }
    let (mut inv, inv_est) = investments_at(conn, as_of, scope);
    // Cotas capital de cooperativa: o último saldo informado entra como investimento (só no
    // patrimônio da família: não ficam por conta).
    if scope.is_none() {
        for (ccy, bal) in crate::accounts::capital::net_worth_at(conn, as_of).unwrap_or_default() {
            if let Some(c) = fx.convert(bal, &ccy, as_of)? {
                inv += c.amount;
            }
        }
    }
    out.investments = inv;
    out.assets += inv;
    let (phys, phys_est) = physical_at(&physical_loaded(conn), &mut fx, as_of, scope)?;
    out.physical = phys;
    out.assets += phys;
    out.debts = debts_at(conn, as_of, scope);
    out.liabilities += out.debts;
    out.net = out.assets - out.liabilities;
    out.missing_fx = fx.missing_vec();
    out.estimated = fx.estimated || inv_est || phys_est;
    Ok(out)
}

/// Bens físicos carregados para o patrimônio. Falhas não derrubam o patrimônio (ficam de
/// fora, como a carteira).
fn physical_loaded(conn: &Connection) -> Vec<crate::physical::Loaded> {
    crate::physical::load_all(conn).unwrap_or_else(|e| {
        tracing::warn!("patrimônio: bens físicos indisponíveis: {e}");
        vec![]
    })
}

/// Valor dos bens físicos numa data, na moeda base: (valor, estimado). Os financiamentos
/// ligados já contam em `debts` (não há dupla contagem: o bem entra pelo valor bruto).
pub(crate) fn physical_at(
    loaded: &[crate::physical::Loaded],
    fx: &mut Fx<'_>,
    as_of: &str,
    scope: Option<&member::MemberScope>,
) -> Result<(i64, bool)> {
    let mut total = 0;
    let mut estimated = false;
    for item in crate::physical::net_worth_items(loaded, parse_date(as_of)?) {
        let share = scope.map_or(1.0, |s| s.physical_share(&item.id));
        if share <= 0.0 {
            continue;
        }
        let value = member::scale(item.value, share);
        if let Some(c) = fx.convert(value, &item.currency, as_of)? {
            total += c.amount;
            estimated |= item.estimated;
        }
    }
    Ok((total, estimated))
}

/// Dívidas numa data: todas, ou as do membro (contratos em que é titular).
fn debts_at(conn: &Connection, as_of: &str, scope: Option<&member::MemberScope>) -> i64 {
    match scope {
        None => crate::debt::total_balance_at(conn, as_of).unwrap_or(0),
        Some(s) => crate::debt::member_balance_at(conn, &s.member_id, as_of).unwrap_or(0),
    }
}

/// Valor de mercado da carteira numa data (0 sem operações). Falhas da carteira não
/// derrubam o patrimônio: o valor fica de fora e é marcado como estimado. Com `scope`, a
/// fração do membro em cada conta com operações.
fn investments_at(
    conn: &Connection,
    as_of: &str,
    scope: Option<&member::MemberScope>,
) -> (i64, bool) {
    if let Some(scope) = scope {
        let accounts: Vec<Id> = conn
            .prepare(
                "SELECT account_id FROM trade WHERE date <= ?1
                 UNION SELECT account_id FROM pension_payment WHERE date <= ?1",
            )
            .and_then(|mut st| {
                st.query_map([as_of], |r| r.get(0))?
                    .collect::<rusqlite::Result<Vec<Id>>>()
            })
            .unwrap_or_default();
        let (mut total, mut estimated) = (0, false);
        for id in accounts {
            let share = scope.account_share(&id);
            if share <= 0.0 {
                continue;
            }
            let args = crate::portfolio::PortfolioArgs {
                as_of: Some(as_of.to_string()),
                account_ids: Some(vec![id]),
                deflator: Some(crate::indices::Deflator::None),
                by_account: None,
                include_closed: None,
            };
            match crate::portfolio::engine::summary(conn, &args) {
                Ok(s) => {
                    total += member::scale(s.market_value, share);
                    estimated |= s.estimated || !s.missing_prices.is_empty();
                }
                Err(_) => estimated = true,
            }
        }
        return (total, estimated);
    }
    let has_trades: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM trade WHERE date <= ?1)
                 OR EXISTS (SELECT 1 FROM pension_payment WHERE date <= ?1)",
            [as_of],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if !has_trades {
        return (0, false);
    }
    let args = crate::portfolio::PortfolioArgs {
        as_of: Some(as_of.to_string()),
        account_ids: None,
        deflator: Some(crate::indices::Deflator::None),
        by_account: None,
        include_closed: None,
    };
    match crate::portfolio::engine::summary(conn, &args) {
        Ok(s) => (s.market_value, s.estimated || !s.missing_prices.is_empty()),
        Err(e) => {
            tracing::warn!("patrimônio: carteira indisponível: {e}");
            (0, true)
        }
    }
}

/// Patrimônio mês a mês pelo saldo no último dia de cada mês. Meses anteriores à abertura
/// da primeira conta (sem nenhuma conta existente) ficam de fora.
pub fn net_worth_history(
    conn: &Connection,
    months: u32,
    end_month: &str,
) -> Result<Vec<NetWorthPoint>> {
    net_worth_history_scoped(conn, months, end_month, None)
}

pub fn net_worth_history_scoped(
    conn: &Connection,
    months: u32,
    end_month: &str,
    scope: Option<&member::MemberScope>,
) -> Result<Vec<NetWorthPoint>> {
    let end = normalize_month(end_month)?;
    let n = months.clamp(1, MAX_MONTHS as u32) as i32;
    let list = months_between(&shift_month(&end, -(n - 1))?, &end)?;
    let (_, end_date) = month_dates(&end)?;
    let base = base_currency(conn)?;
    struct Acc {
        currency: String,
        opening_balance: i64,
        opening_date: String,
        /// (mês, soma) em ordem.
        sums: Vec<(String, i64)>,
    }
    let mut accounts: BTreeMap<Id, Acc> = BTreeMap::new();
    {
        let mut stmt = conn.prepare_cached(
            "SELECT id, currency, opening_balance, opening_date FROM account WHERE include_in_net_worth = 1",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            accounts.insert(
                r.get(0)?,
                Acc {
                    currency: r.get(1)?,
                    opening_balance: r.get(2)?,
                    opening_date: r.get(3)?,
                    sums: vec![],
                },
            );
        }
        let mut stmt = conn.prepare_cached(
            "SELECT t.account_id, substr(t.date, 1, 7) AS m, SUM(t.amount)
             FROM txn t JOIN account a ON a.id = t.account_id AND a.include_in_net_worth = 1
             WHERE t.status IN ('cleared', 'reconciled') AND t.date <= ?1
             GROUP BY t.account_id, m ORDER BY t.account_id, m",
        )?;
        let mut rows = stmt.query([&end_date])?;
        while let Some(r) = rows.next()? {
            let id: String = r.get(0)?;
            if let Some(a) = accounts.get_mut(&id) {
                a.sums.push((r.get(1)?, r.get(2)?));
            }
        }
    }
    // Saldo acumulado de cada conta, avançando mês a mês.
    let mut cursor: BTreeMap<&str, (usize, i64, bool)> = accounts
        .keys()
        .map(|k| (k.as_str(), (0usize, 0i64, false)))
        .collect();
    let mut fx = Fx::new(conn, &base);
    let physical = physical_loaded(conn);
    let mut out = Vec::with_capacity(list.len());
    for month in list {
        let (_, last) = month_dates(&month)?;
        let mut p = NetWorthPoint {
            month: month.clone(),
            as_of: last.clone(),
            assets: 0,
            liabilities: 0,
            net: 0,
            investments: 0,
            physical: 0,
            debts: 0,
            missing_fx: vec![],
            estimated: false,
        };
        let mut any = false;
        for (id, a) in &accounts {
            let c = cursor.get_mut(id.as_str()).expect("todas as contas");
            while c.0 < a.sums.len() && a.sums[c.0].0 <= month {
                c.1 += a.sums[c.0].1;
                c.2 = true;
                c.0 += 1;
            }
            let opened = a.opening_date <= last;
            if !opened && !c.2 {
                continue;
            }
            any = true;
            let share = member::account_share(scope, id);
            if share <= 0.0 {
                continue;
            }
            let balance = member::scale(c.1 + if opened { a.opening_balance } else { 0 }, share);
            if balance == 0 {
                continue;
            }
            match fx.convert(balance, &a.currency, &last)? {
                Some(conv) => {
                    p.estimated |= conv.estimated;
                    if conv.amount >= 0 {
                        p.assets += conv.amount;
                    } else {
                        p.liabilities += -conv.amount;
                    }
                }
                None => {
                    if !p.missing_fx.contains(&a.currency) {
                        p.missing_fx.push(a.currency.clone());
                    }
                }
            }
        }
        let (inv, inv_est) = investments_at(conn, &last, scope);
        let (phys, phys_est) = physical_at(&physical, &mut fx, &last, scope)?;
        if !any && inv == 0 && phys == 0 {
            continue;
        }
        p.investments = inv;
        p.assets += inv;
        p.estimated |= inv_est;
        p.physical = phys;
        p.assets += phys;
        p.estimated |= phys_est;
        p.debts = debts_at(conn, &last, scope);
        p.liabilities += p.debts;
        p.net = p.assets - p.liabilities;
        p.missing_fx.sort();
        out.push(p);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Fluxo do mês

/// Receitas, despesas e taxa de poupança mês a mês (inclusive meses sem movimento).
pub fn month_flow(
    conn: &Connection,
    from_month: &str,
    to_month: &str,
    basis: Basis,
) -> Result<Vec<MonthFlow>> {
    month_flow_scoped(conn, from_month, to_month, basis, None)
}

/// Receitas e despesas mês a mês da família ou, com `scope`, de um membro.
pub fn month_flow_scoped(
    conn: &Connection,
    from_month: &str,
    to_month: &str,
    basis: Basis,
    scope: Option<&member::MemberScope>,
) -> Result<Vec<MonthFlow>> {
    let months = months_between(from_month, to_month)?;
    let (from, _) = month_dates(&months[0])?;
    let (_, to) = month_dates(months.last().expect("ao menos um mês"))?;
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let mut q = FlowQuery::new(&from, &to, basis);
    q.member_ids = scope.map(|s| s.ids.as_slice());
    let agg = q.aggregate(conn, &mut fx, Key::Month, Key::None)?;
    q.loan_linked = Some(true);
    q.kind = Some(CategoryKind::Expense);
    let debt = q.aggregate(conn, &mut fx, Key::Month, Key::None)?;
    let extra = bridge_extras(conn, &mut fx, &from, &to, scope)?;
    Ok(months
        .into_iter()
        .map(|m| {
            let c = agg
                .cells
                .get(&(Some(m.clone()), None))
                .cloned()
                .unwrap_or_default();
            let debt_service = debt
                .cells
                .get(&(Some(m.clone()), None))
                .map(|c| c.expense)
                .unwrap_or(0);
            let mut bridge = extra.get(&m).cloned().unwrap_or_default();
            bridge.debt_service = debt_service;
            bridge.consumption = c.expense - debt_service;
            let net = c.income - c.expense;
            MonthFlow {
                month: m,
                income: c.income,
                expense: c.expense,
                net,
                savings_rate: ratio(net, c.income),
                missing_fx: c.missing_fx.into_iter().collect(),
                estimated: c.estimated,
                bridge: Some(bridge),
            }
        })
        .collect())
}

/// Juros e amortização das parcelas pagas, aportes, resgates e ajustes de saldo por mês (pela
/// data do lançamento), na moeda base.
fn bridge_extras(
    conn: &Connection,
    fx: &mut Fx<'_>,
    from: &str,
    to: &str,
    scope: Option<&member::MemberScope>,
) -> Result<HashMap<String, MonthBridge>> {
    let mut out: HashMap<String, MonthBridge> = HashMap::new();
    let members: Vec<Id> = scope.map(|s| s.ids.clone()).unwrap_or_default();
    let in_scope =
        |member: Option<String>| members.is_empty() || member.is_some_and(|m| members.contains(&m));
    let mut st = conn.prepare(
        "SELECT substr(t.date, 1, 7), t.date, t.currency, t.kind, t.amount,
                COALESCE(t.member_id, (SELECT CASE WHEN count(*) = 1 THEN min(am.member_id) END
                                         FROM account_member am WHERE am.account_id = t.account_id)),
                li.interest, li.amortization + li.extra_payment
           FROM txn t
           LEFT JOIN loan_installment li ON li.txn_id = t.id
          WHERE t.status IN ('cleared', 'reconciled') AND t.date BETWEEN ?1 AND ?2
            AND (t.kind IN ('investment', 'adjustment') OR li.id IS NOT NULL)",
    )?;
    let rows = st
        .query_map([from, to], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, Option<i64>>(7)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (month, date, currency, kind, amount, member, interest, amort) in rows {
        if !in_scope(member) {
            continue;
        }
        let mut conv = |v: i64| -> Result<i64> {
            Ok(fx
                .convert(v, &currency, &date)?
                .map(|c| c.amount)
                .unwrap_or(0))
        };
        let b = out.entry(month).or_default();
        match kind.as_str() {
            "investment" if amount < 0 => b.invested += conv(-amount)?,
            "investment" => b.redeemed += conv(amount)?,
            "adjustment" => b.adjustments += conv(amount)?,
            _ => {}
        }
        if let (Some(i), Some(a)) = (interest, amort) {
            b.debt_interest += conv(i)?;
            b.debt_amortization += conv(a)?;
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Gastos agrupados (treemap, sunburst, barras)

pub fn spending(conn: &Connection, a: &SpendingArgs) -> Result<SpendingSummary> {
    validate_range(&a.from, &a.to)?;
    let basis = resolve_basis(conn, a.basis)?;
    let kind = a.kind.unwrap_or(CategoryKind::Expense);
    let group_by = a.group_by.unwrap_or_default();
    let cats = match &a.category_ids {
        Some(ids) if !ids.is_empty() => Some(crate::categories::with_descendants(conn, ids)?),
        _ => None,
    };
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let mut q = FlowQuery::new(&a.from, &a.to, basis);
    q.kind = Some(kind);
    q.member_ids = a.member_ids.as_deref();
    q.account_ids = a.account_ids.as_deref();
    q.category_ids = cats.as_deref();
    let key = match group_by {
        GroupBy::Category => Key::Category,
        GroupBy::ParentCategory => Key::RootCategory,
        GroupBy::Member => Key::Member,
        GroupBy::Account => Key::Account,
        GroupBy::Month => Key::Month,
    };
    let agg = q.aggregate(conn, &mut fx, key, Key::None)?;
    let tree = CategoryTree::load(conn)?;
    let labels = Labels::load(conn)?;
    let mut cells: BTreeMap<Option<String>, (i64, i64)> = agg
        .cells
        .iter()
        .map(|((k, _), c)| (k.clone(), (c.amount(kind), c.count(kind))))
        .filter(|(_, (amt, n))| *amt != 0 || *n != 0)
        .collect();
    if group_by == GroupBy::Month {
        for m in months_between(&a.from[..7], &a.to[..7])? {
            cells.entry(Some(m)).or_insert((0, 0));
        }
    }
    let total: i64 = cells.values().map(|(v, _)| v).sum();
    // Lançamentos distintos (um dividido conta uma vez, mesmo aparecendo em vários grupos).
    let count = q
        .aggregate(conn, &mut fx, Key::None, Key::None)?
        .total()
        .count(kind);
    let mut groups: Vec<GroupAmount> = cells
        .into_iter()
        .map(|(key, (amount, n))| {
            let (label, path, parent_key, color) = match group_by {
                GroupBy::Category | GroupBy::ParentCategory => {
                    category_group(&tree, key.as_deref())
                }
                GroupBy::Member => match key.as_deref().and_then(|k| labels.members.get(k)) {
                    Some(m) => (m.name.clone(), None, None, m.color.clone()),
                    None => (key.clone().unwrap_or_else(no_member), None, None, None),
                },
                GroupBy::Account => match key.as_deref().and_then(|k| labels.accounts.get(k)) {
                    Some((name, color)) => (name.clone(), None, None, color.clone()),
                    None => (key.clone().unwrap_or_default(), None, None, None),
                },
                GroupBy::Month => (
                    key.as_deref().map(month_label).unwrap_or_default(),
                    None,
                    None,
                    None,
                ),
            };
            GroupAmount {
                key,
                label,
                parent_key,
                path,
                amount,
                share: share(amount, total),
                count: n,
                color,
            }
        })
        .collect();
    if group_by != GroupBy::Month {
        groups.sort_by(|x, y| y.amount.cmp(&x.amount).then_with(|| x.label.cmp(&y.label)));
    }
    Ok(SpendingSummary {
        from: a.from.clone(),
        to: a.to.clone(),
        basis,
        kind,
        currency: base,
        total,
        count,
        groups,
        missing_fx: agg.missing_vec(),
        estimated: agg.estimated,
    })
}

// ---------------------------------------------------------------------------
// Calendário (heatmap)

/// Limite de dias do calendário (~10 anos).
const MAX_CALENDAR_DAYS: i64 = 3700;

pub fn calendar(conn: &Connection, from: &str, to: &str, basis: Basis) -> Result<CalendarData> {
    let (a, b) = validate_range(from, to)?;
    if (b - a).num_days() > MAX_CALENDAR_DAYS {
        return Err(Error::invalid(t("nucleo-relatorios-erro-calendario-longo")));
    }
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let agg = FlowQuery::new(from, to, basis).aggregate(conn, &mut fx, Key::Day, Key::None)?;
    let mut days = Vec::with_capacity((b - a).num_days() as usize + 1);
    let mut d = a;
    while d <= b {
        let key = (Some(fmt_date(d)), None);
        let c = agg.cells.get(&key).cloned().unwrap_or_default();
        days.push(CalendarDay {
            date: fmt_date(d),
            expense: c.expense,
            income: c.income,
            count: c.income_count + c.expense_count,
        });
        d = d.succ_opt().expect("data válida");
    }
    let total = agg.total();
    Ok(CalendarData {
        from: from.to_string(),
        to: to.to_string(),
        basis,
        max_expense: days.iter().map(|d| d.expense).max().unwrap_or(0).max(0),
        days,
        total_expense: total.expense,
        total_income: total.income,
        currency: base,
        missing_fx: agg.missing_vec(),
        estimated: agg.estimated,
    })
}

// ---------------------------------------------------------------------------
// Evolução mensal

pub fn monthly_evolution(
    conn: &Connection,
    months: u32,
    end_month: &str,
    top_n: u32,
    basis: Basis,
) -> Result<MonthlyEvolution> {
    let end = normalize_month(end_month)?;
    let n = months.clamp(1, MAX_MONTHS as u32) as i32;
    let list = months_between(&shift_month(&end, -(n - 1))?, &end)?;
    // Dois meses antes da janela, para a média móvel dos primeiros meses.
    let ext = months_between(&shift_month(&list[0], -2)?, &end)?;
    let (from, _) = month_dates(&ext[0])?;
    let (_, to) = month_dates(&end)?;
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let agg = FlowQuery::new(&from, &to, basis).aggregate(
        conn,
        &mut fx,
        Key::RootCategory,
        Key::Month,
    )?;
    let tree = CategoryTree::load(conn)?;
    let pos: HashMap<&str, usize> = list
        .iter()
        .enumerate()
        .map(|(i, m)| (m.as_str(), i))
        .collect();
    let ext_pos: HashMap<&str, usize> = ext
        .iter()
        .enumerate()
        .map(|(i, m)| (m.as_str(), i))
        .collect();
    let mut ext_totals = vec![0i64; ext.len()];
    let mut income = vec![0i64; list.len()];
    let mut by_root: BTreeMap<Option<String>, Vec<i64>> = BTreeMap::new();
    for ((root, month), c) in &agg.cells {
        let Some(month) = month.as_deref() else {
            continue;
        };
        if let Some(&i) = ext_pos.get(month) {
            ext_totals[i] += c.expense;
        }
        if let Some(&i) = pos.get(month) {
            income[i] += c.income;
            if c.expense != 0 {
                by_root
                    .entry(root.clone())
                    .or_insert_with(|| vec![0; list.len()])[i] += c.expense;
            }
        }
    }
    let mut ranked: Vec<(Option<String>, Vec<i64>, i64)> = by_root
        .into_iter()
        .map(|(k, v)| {
            let t = v.iter().sum();
            (k, v, t)
        })
        .filter(|(_, v, _)| v.iter().any(|x| *x != 0))
        .collect();
    ranked.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    let top = top_n as usize;
    let mut series = vec![];
    let mut others = vec![0i64; list.len()];
    let mut has_others = false;
    for (i, (key, values, _)) in ranked.into_iter().enumerate() {
        if i < top {
            let (label, _, _, color) = category_group(&tree, key.as_deref());
            series.push(EvolutionSeries {
                key,
                label,
                color,
                values,
            });
        } else {
            has_others = true;
            for (o, v) in others.iter_mut().zip(values) {
                *o += v;
            }
        }
    }
    if has_others {
        series.push(EvolutionSeries {
            key: Some(OTHERS_KEY.into()),
            label: t("nucleo-relatorios-outras"),
            color: None,
            values: others,
        });
    }
    // Média móvel: só meses a partir do primeiro com dados.
    let first = match first_flow_month(conn)? {
        Some(f) => month_index(&f)?,
        None => i32::MAX,
    };
    let ext0 = month_index(&ext[0])?;
    let mut moving = Vec::with_capacity(list.len());
    for i in 2..ext.len() {
        let mi = ext0 + i as i32;
        let lo = (mi - 2).max(first).min(mi);
        let (a, b) = ((lo - ext0) as usize, i);
        let sum: i64 = ext_totals[a..=b].iter().sum();
        moving.push(engine::div_round(sum, (b - a + 1) as i64));
    }
    Ok(MonthlyEvolution {
        totals: ext_totals[2..].to_vec(),
        months: list,
        basis,
        series,
        moving_avg_3: moving,
        income,
        currency: base,
        missing_fx: agg.missing_vec(),
        estimated: agg.estimated,
    })
}

// ---------------------------------------------------------------------------
// Comparação de períodos

pub fn compare(
    conn: &Connection,
    a: &Period,
    b: &Period,
    basis: Basis,
    kind: CategoryKind,
) -> Result<PeriodComparison> {
    validate_range(&a.from, &a.to)?;
    validate_range(&b.from, &b.to)?;
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let run = |p: &Period, fx: &mut Fx<'_>| {
        let mut q = FlowQuery::new(&p.from, &p.to, basis);
        q.kind = Some(kind);
        q.aggregate(conn, fx, Key::RootCategory, Key::None)
    };
    let agg_a = run(a, &mut fx)?;
    let agg_b = run(b, &mut fx)?;
    let tree = CategoryTree::load(conn)?;
    let mut vals: BTreeMap<Option<String>, (i64, i64)> = BTreeMap::new();
    for ((k, _), c) in &agg_a.cells {
        vals.entry(k.clone()).or_default().0 += c.amount(kind);
    }
    for ((k, _), c) in &agg_b.cells {
        vals.entry(k.clone()).or_default().1 += c.amount(kind);
    }
    let mut rows: Vec<ComparisonRow> = vals
        .into_iter()
        .filter(|(_, (x, y))| *x != 0 || *y != 0)
        .map(|(key, (x, y))| {
            let (label, _, _, color) = category_group(&tree, key.as_deref());
            ComparisonRow {
                key,
                label,
                a: x,
                b: y,
                delta: y - x,
                delta_ratio: ratio(y - x, x),
                color,
            }
        })
        .collect();
    rows.sort_by(|x, y| {
        y.delta
            .abs()
            .cmp(&x.delta.abs())
            .then_with(|| x.label.cmp(&y.label))
    });
    let mut missing = agg_a.missing_fx.clone();
    missing.extend(agg_b.missing_fx.iter().cloned());
    Ok(PeriodComparison {
        a: a.clone(),
        b: b.clone(),
        basis,
        kind,
        rows,
        total_a: agg_a.total().amount(kind),
        total_b: agg_b.total().amount(kind),
        currency: base,
        missing_fx: missing.into_iter().collect(),
        estimated: agg_a.estimated || agg_b.estimated,
    })
}

// ---------------------------------------------------------------------------
// Titulares

pub fn by_member(conn: &Connection, from: &str, to: &str, basis: Basis) -> Result<MemberBreakdown> {
    validate_range(from, to)?;
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let mut q = FlowQuery::new(from, to, basis);
    q.kind = Some(CategoryKind::Expense);
    let per_member = q.aggregate(conn, &mut fx, Key::Member, Key::None)?;
    let grid = q.aggregate(conn, &mut fx, Key::Member, Key::RootCategory)?;
    let labels = Labels::load(conn)?;
    let tree = CategoryTree::load(conn)?;
    let totals: HashMap<Option<String>, (i64, i64)> = per_member
        .cells
        .iter()
        .map(|((m, _), c)| (m.clone(), (c.expense, c.expense_count)))
        .collect();
    let total: i64 = totals.values().map(|(v, _)| v).sum();
    // Titulares ativos (mesmo sem gasto) e arquivados com gasto, na ordem cadastrada;
    // por último, "Sem membro" quando houver.
    let has_value = |k: &Option<String>| totals.get(k).is_some_and(|t| t.0 != 0 || t.1 != 0);
    let mut ids: Vec<(&Id, &MemberLabel)> = labels
        .members
        .iter()
        .filter(|(id, m)| !m.archived || has_value(&Some((*id).clone())))
        .collect();
    ids.sort_by(|a, b| (a.1.sort_order, &a.1.name, a.0).cmp(&(b.1.sort_order, &b.1.name, b.0)));
    let mut keys: Vec<Option<String>> = ids.iter().map(|(id, _)| Some((*id).clone())).collect();
    // Titulares que não existem mais (não deveria acontecer), em ordem estável.
    let mut unknown: Vec<Option<String>> = totals
        .keys()
        .filter(|k| k.is_some() && !keys.contains(k))
        .cloned()
        .collect();
    unknown.sort();
    keys.extend(unknown);
    if has_value(&None) {
        keys.push(None);
    }
    let members: Vec<GroupAmount> = keys
        .iter()
        .map(|k| {
            let (amount, count) = totals.get(k).copied().unwrap_or((0, 0));
            let (label, color) = match k.as_deref().and_then(|id| labels.members.get(id)) {
                Some(m) => (m.name.clone(), m.color.clone()),
                None => (k.clone().unwrap_or_else(no_member), None),
            };
            GroupAmount {
                key: k.clone(),
                label,
                parent_key: None,
                path: None,
                amount,
                share: share(amount, total),
                count,
                color,
            }
        })
        .collect();
    let index: HashMap<&Option<String>, usize> =
        keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
    let mut rows: BTreeMap<Option<String>, Vec<i64>> = BTreeMap::new();
    for ((m, root), c) in &grid.cells {
        if c.expense == 0 {
            continue;
        }
        if let Some(&i) = index.get(m) {
            rows.entry(root.clone())
                .or_insert_with(|| vec![0; keys.len()])[i] += c.expense;
        }
    }
    let mut categories: Vec<(i64, EvolutionSeries)> = rows
        .into_iter()
        .map(|(key, values)| {
            let (label, _, _, color) = category_group(&tree, key.as_deref());
            (
                values.iter().sum(),
                EvolutionSeries {
                    key,
                    label,
                    color,
                    values,
                },
            )
        })
        .collect();
    categories.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.label.cmp(&b.1.label)));
    let mut missing = per_member.missing_fx.clone();
    missing.extend(grid.missing_fx.iter().cloned());
    Ok(MemberBreakdown {
        from: from.to_string(),
        to: to.to_string(),
        members,
        categories: categories.into_iter().map(|(_, s)| s).collect(),
        basis,
        total,
        currency: base,
        missing_fx: missing.into_iter().collect(),
        estimated: per_member.estimated || grid.estimated,
    })
}

// ---------------------------------------------------------------------------
// Próximos vencimentos

/// Lançamentos previstos e vencimentos de fatura entre `from` e `from + days`.
///
/// - Previstos fora de fatura (os de cartão aparecem pela fatura). De transferências e
///   aportes, só a perna que sai; pagamentos de fatura previstos ficam de fora (a fatura
///   já aparece).
/// - Faturas não pagas: valor = compras − estornos (inclusive parcelas previstas) − pagamentos
///   já ligados à fatura.
/// - Archived accounts bring no planned item, and a closed card no statement (an archived
///   card that still owes keeps its statement).
pub fn upcoming(conn: &Connection, from: NaiveDate, days: u32) -> Result<Vec<UpcomingItem>> {
    let to = from + chrono::Duration::days(days.min(3660) as i64);
    let (from_s, to_s) = (fmt_date(from), fmt_date(to));
    let mut out = vec![];
    {
        let mut stmt = conn.prepare_cached(
            "SELECT t.id, t.date, t.description, t.amount, t.currency, t.account_id, a.name, c.name,
                    t.recurrence_id
             FROM txn t JOIN account a ON a.id = t.account_id
             LEFT JOIN category c ON c.id = t.category_id
             WHERE t.status = 'planned' AND t.date BETWEEN ?1 AND ?2 AND t.statement_id IS NULL
               AND a.archived = 0
               AND t.kind <> 'card_payment'
               AND NOT (t.kind IN ('transfer', 'investment') AND t.amount > 0)
             ORDER BY t.date, t.id",
        )?;
        let mut rows = stmt.query([&from_s, &to_s])?;
        while let Some(r) = rows.next()? {
            let desc: String = r.get(2)?;
            let cat: Option<String> = r.get(7)?;
            out.push(UpcomingItem {
                ref_id: r.get(0)?,
                date: r.get(1)?,
                kind: UpcomingKind::PlannedTxn,
                description: if desc.trim().is_empty() {
                    cat.unwrap_or_else(|| t("nucleo-relatorios-sem-descricao"))
                } else {
                    desc
                },
                amount: r.get(3)?,
                currency: r.get(4)?,
                account_id: r.get(5)?,
                account_name: r.get(6)?,
                recurrence_id: r.get(8)?,
            });
        }
    }
    {
        let mut stmt = conn.prepare_cached(
            "SELECT cs.id, cs.account_id, a.name, a.currency, cs.due_date, cs.reference_month,
                    COALESCE((SELECT -SUM(t.amount) FROM txn t
                              WHERE t.statement_id = cs.id AND t.kind <> 'card_payment'), 0),
                    COALESCE((SELECT SUM(t.amount) FROM txn t
                              WHERE t.statement_id = cs.id AND t.kind = 'card_payment'
                                AND t.status IN ('cleared', 'reconciled')), 0)
             FROM card_statement cs JOIN account a ON a.id = cs.account_id
             WHERE cs.due_date BETWEEN ?1 AND ?2 AND cs.status <> 'paid' AND a.closed_on IS NULL
             ORDER BY cs.due_date, a.name",
        )?;
        let mut rows = stmt.query([&from_s, &to_s])?;
        while let Some(r) = rows.next()? {
            let total: i64 = r.get(6)?;
            let paid: i64 = r.get(7)?;
            let remaining = total - paid;
            if remaining <= 0 {
                continue;
            }
            let name: String = r.get(2)?;
            let month: String = r.get(5)?;
            out.push(UpcomingItem {
                ref_id: r.get(0)?,
                account_id: r.get(1)?,
                description: t_args(
                    "nucleo-relatorios-fatura",
                    &args(&[
                        ("cartao", name.clone().into()),
                        ("mes", month_label(&month).into()),
                    ]),
                ),
                account_name: name,
                currency: r.get(3)?,
                date: r.get(4)?,
                kind: UpcomingKind::CardDue,
                amount: -remaining,
                recurrence_id: None,
            });
        }
    }
    out.sort_by(|a, b| {
        (&a.date, a.kind == UpcomingKind::PlannedTxn, &a.description).cmp(&(
            &b.date,
            b.kind == UpcomingKind::PlannedTxn,
            &b.description,
        ))
    });
    Ok(out)
}

// ---------------------------------------------------------------------------
// Painel

pub fn dashboard(conn: &Connection, month: Option<&str>) -> Result<Dashboard> {
    dashboard_with(conn, month, None)
}

pub fn dashboard_with(
    conn: &Connection,
    month: Option<&str>,
    basis: Option<Basis>,
) -> Result<Dashboard> {
    dashboard_scoped(conn, month, basis, None)
}

/// Painel da família ou, com `scope`, de um membro (sem orçamento: ele é da família).
pub fn dashboard_scoped(
    conn: &Connection,
    month: Option<&str>,
    basis: Option<Basis>,
    scope: Option<&member::MemberScope>,
) -> Result<Dashboard> {
    let month = match month {
        Some(m) => normalize_month(m)?,
        None => current_month(),
    };
    let basis = resolve_basis(conn, basis)?;
    let today = today();
    let (first, last) = month_dates(&month)?;
    let today_s = fmt_date(today);
    // Patrimônio no fim do mês (ou hoje, no mês corrente e em meses futuros).
    let as_of = if last < today_s {
        last.clone()
    } else {
        today_s
    };
    let net_worth = net_worth_scoped(conn, &as_of, scope)?;
    let flow_history = month_flow_scoped(conn, &shift_month(&month, -11)?, &month, basis, scope)?;
    let flow = flow_history.last().cloned().expect("12 meses");
    let flow_expense = flow.expense;
    let (inc, exp): (i64, i64) = flow_history
        .iter()
        .fold((0, 0), |(i, e), f| (i + f.income, e + f.expense));
    let top = spending(
        conn,
        &SpendingArgs {
            from: first.clone(),
            to: last.clone(),
            basis: Some(basis),
            kind: Some(CategoryKind::Expense),
            group_by: Some(GroupBy::ParentCategory),
            member_ids: scope.map(|s| s.ids.clone()),
            account_ids: None,
            category_ids: None,
        },
    )?;
    let budget = crate::budget::get(conn, &month, Some(basis))?;
    let has_budget = scope.is_none() && budget.lines.iter().any(|l| l.has_own_line);
    let base = base_currency(conn)?;
    let mut fx = Fx::new(conn, &base);
    let mut q = FlowQuery::new(&first, &last, basis);
    q.member_ids = scope.map(|s| s.ids.as_slice());
    let cats = q.aggregate(conn, &mut fx, Key::Category, Key::None)?;
    let uncategorized_count = cats
        .cells
        .get(&(None, None))
        .map(|c| c.income_count + c.expense_count)
        .unwrap_or(0);
    Ok(Dashboard {
        month,
        basis,
        net_worth,
        flow,
        savings_rate_12m: ratio(inc - exp, inc),
        flow_history,
        top_categories: top
            .groups
            .into_iter()
            .filter(|g| g.amount > 0)
            .take(5)
            .collect(),
        upcoming: upcoming(conn, today, 30)?
            .into_iter()
            .filter(|u| member::account_share(scope, &u.account_id) > 0.0)
            .collect(),
        budget_planned: has_budget.then_some(budget.totals.planned_expense),
        budget_actual: if scope.is_some() {
            flow_expense
        } else {
            budget.totals.actual_expense
        },
        uncategorized_count,
        currency: base,
    })
}
