//! Motor comum dos cálculos de fluxo de caixa (orçamento, metas e relatórios).
//!
//! - [`CategoryTree`]: a árvore de categorias em memória (são poucas dezenas), com caminho,
//!   raiz e nível de cada uma, para somar descendentes sem consultas recursivas repetidas.
//! - [`Fx`]: conversão para uma moeda-alvo com [`crate::fx::convert`], anotando moedas sem
//!   câmbio (ficam fora dos totais) e conversões estimadas.
//! - [`FlowQuery::aggregate`]: soma receitas e despesas em SQL (`GROUP BY`), sem carregar os
//!   lançamentos em memória. Lançamentos na moeda-alvo são somados direto no banco; os em
//!   outra moeda saem agrupados por lançamento, para converter na data de cada um.
//!
//! Regras (ver `docs/architecture.md`, "Cálculos centrais › Fluxo de caixa"):
//! - só `kind IN ('income','expense')` conta; transferências, pagamento de fatura e aportes não,
//!   exceto a transferência de uma perna só (PIX para terceiros…) com categoria, que conta
//!   como despesa (saída) ou receita (entrada) — ver [`FLOW_KIND`];
//! - realizado = `cleared`/`reconciled`; `planned` só quando pedido (comprometido/previsto);
//! - despesa = −soma dos valores (estorno positivo reduz o gasto); receita = soma;
//! - lançamento com divisão usa as partes: categoria da parte e titular da parte (a parte sem
//!   titular herda o titular do lançamento);
//! - competência usa `txn.date`; caixa usa o vencimento da fatura (`card_statement.due_date`)
//!   para lançamentos de cartão e `txn.date` para os demais.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use rusqlite::types::Value as SqlValue;

use crate::budget::Basis;
use crate::categories::CategoryKind;
use crate::error::{Error, Result};
use crate::fx::Converted;
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, fmt_month, month_bounds, parse_date, parse_month};

// ---------------------------------------------------------------------------
// Datas e textos (no idioma da instalação: `locales/<idioma>/nucleo-relatorios.ftl`)

/// `2026-10-14` → `14/10/2026` (pt-BR, es) ou `Oct 14, 2026` (en).
pub(crate) fn date_label(date: &str) -> String {
    match parse_date(date) {
        Ok(d) => crate::i18n::date(crate::i18n::current(), d),
        Err(_) => date.to_string(),
    }
}

/// `2026-10` → `out/2026` (pt-BR) ou `Oct 2026` (en).
pub(crate) fn month_label(month: &str) -> String {
    crate::ai::alerts::month_short(crate::i18n::current(), month)
}

/// Índice contínuo de meses (ano × 12 + mês − 1), para somar e comparar meses.
pub(crate) fn month_index(month: &str) -> Result<i32> {
    let (y, m) = parse_month(month)?;
    Ok(y * 12 + m as i32 - 1)
}

pub(crate) fn month_from_index(i: i32) -> String {
    fmt_month(i.div_euclid(12), i.rem_euclid(12) as u32 + 1)
}

/// Normaliza um mês (`2026-1` → `2026-01`), validando.
pub(crate) fn normalize_month(month: &str) -> Result<String> {
    let (y, m) = parse_month(month)?;
    Ok(fmt_month(y, m))
}

pub(crate) fn shift_month(month: &str, delta: i32) -> Result<String> {
    Ok(month_from_index(month_index(month)? + delta))
}

/// Meses de `from` a `to`, inclusive.
pub(crate) fn months_between(from: &str, to: &str) -> Result<Vec<String>> {
    let (a, b) = (month_index(from)?, month_index(to)?);
    if a > b {
        return Err(Error::invalid(t_args(
            "nucleo-relatorios-erro-mes-inicial-posterior",
            &args(&[
                ("de", from.to_string().into()),
                ("ate", to.to_string().into()),
            ]),
        )));
    }
    if b - a >= MAX_MONTHS {
        return Err(Error::invalid(t_args(
            "nucleo-relatorios-erro-periodo-longo",
            &args(&[("maximo", MAX_MONTHS.into())]),
        )));
    }
    Ok((a..=b).map(month_from_index).collect())
}

/// Limite de meses por consulta mês a mês (20 anos).
pub(crate) const MAX_MONTHS: i32 = 240;

/// Primeiro e último dia de um mês `YYYY-MM`.
pub(crate) fn month_dates(month: &str) -> Result<(String, String)> {
    let (y, m) = parse_month(month)?;
    let (a, b) = month_bounds(y, m);
    Ok((fmt_date(a), fmt_date(b)))
}

pub(crate) fn today() -> NaiveDate {
    crate::time::today_local()
}

pub(crate) fn current_month() -> String {
    let t = today();
    fmt_month(t.year(), t.month())
}

/// Valida um período de datas (`from` ≤ `to`).
pub(crate) fn validate_range(from: &str, to: &str) -> Result<(NaiveDate, NaiveDate)> {
    let (a, b) = (parse_date(from)?, parse_date(to)?);
    if a > b {
        return Err(Error::invalid(t_args(
            "nucleo-relatorios-erro-data-inicial-posterior",
            &args(&[
                ("de", date_label(from).into()),
                ("ate", date_label(to).into()),
            ]),
        )));
    }
    Ok((a, b))
}

/// Base de datas pedida ou, se omitida, o ajuste `budget_basis`.
pub(crate) fn resolve_basis(conn: &Connection, basis: Option<Basis>) -> Result<Basis> {
    match basis {
        Some(b) => Ok(b),
        None => Ok(crate::api::system::load_app_settings(conn)?.budget_basis),
    }
}

/// Base do diagrama de fluxo pedida ou, se omitida, o ajuste `flow_basis` (padrão caixa).
pub(crate) fn resolve_flow_basis(conn: &Connection, basis: Option<Basis>) -> Result<Basis> {
    match basis {
        Some(b) => Ok(b),
        None => Ok(crate::api::system::load_app_settings(conn)?.flow_basis),
    }
}

/// Base de datas por extenso ("competência", "caixa").
pub(crate) fn basis_label(basis: Basis) -> String {
    t(match basis {
        Basis::Accrual => "nucleo-relatorios-base-competencia",
        Basis::Cash => "nucleo-relatorios-base-caixa",
    })
}

/// Razão com uma casa decimal no idioma: `0.5323` → `53,2%` (pt-BR) ou `53.2%` (en).
pub(crate) fn percent(r: f64) -> String {
    crate::ai::alerts::pct(crate::i18n::current(), r, 1)
}

/// Dinheiro no idioma da instalação (`R$ 1.234,56`, `USD 1.234,56`).
pub(crate) fn money(amount: i64, currency: &str) -> String {
    crate::i18n::money(crate::i18n::current(), amount, currency)
}

/// Razão `a ÷ b` para exibição (`None` quando `b` ≤ 0).
pub(crate) fn ratio(a: i64, b: i64) -> Option<f64> {
    (b > 0).then(|| a as f64 / b as f64)
}

/// Divisão inteira arredondando a metade para longe do zero.
pub(crate) fn div_round(a: i64, b: i64) -> i64 {
    if b == 0 {
        return 0;
    }
    let q = a / b;
    let r = a % b;
    if (r.abs() * 2) >= b.abs() {
        q + if (a < 0) != (b < 0) { -1 } else { 1 }
    } else {
        q
    }
}

/// Divisão inteira arredondando para cima (valores positivos).
pub(crate) fn div_ceil(a: i64, b: i64) -> i64 {
    if b <= 0 {
        return 0;
    }
    if a <= 0 {
        return a / b;
    }
    (a + b - 1) / b
}

// ---------------------------------------------------------------------------
// Árvore de categorias

#[derive(Debug, Clone)]
pub(crate) struct CatInfo {
    pub id: Id,
    pub parent_id: Option<Id>,
    pub name: String,
    pub kind: CategoryKind,
    pub color: Option<String>,
    /// Essencial: nas despesas, os grupos de gasto `essential` e `debt` (ver
    /// [`CatInfo::spend_group`]).
    pub is_essential: bool,
    /// Grupo de gasto (V129); `None` nas receitas.
    pub spend_group: Option<crate::categories::SpendGroup>,
    pub archived: bool,
    /// `Alimentação › Mercado`.
    pub path: String,
    /// Categoria de primeiro nível (ela mesma, se for raiz).
    pub root_id: Id,
    /// 0 = raiz.
    pub depth: usize,
}

#[derive(Debug, Default)]
pub(crate) struct CategoryTree {
    pub by_id: HashMap<Id, CatInfo>,
    /// Ordem de exibição: despesas antes de receitas, pais antes dos filhos, por
    /// `sort_order` e nome.
    pub order: Vec<Id>,
    pub children: HashMap<Id, Vec<Id>>,
    pub roots: Vec<Id>,
}

impl CategoryTree {
    pub fn load(conn: &Connection) -> Result<Self> {
        struct Raw {
            id: Id,
            parent_id: Option<Id>,
            name: String,
            kind: CategoryKind,
            color: Option<String>,
            is_essential: bool,
            spend_group: Option<crate::categories::SpendGroup>,
            archived: bool,
            sort_order: i64,
        }
        let mut stmt = conn.prepare(
            "SELECT id, parent_id, name, kind, color, is_essential, archived, sort_order, spend_group
             FROM category",
        )?;
        let raws: Vec<Raw> = stmt
            .query_map([], |r| {
                let spend_group = r
                    .get::<_, Option<String>>(8)?
                    .as_deref()
                    .and_then(crate::categories::SpendGroup::parse);
                Ok(Raw {
                    id: r.get(0)?,
                    parent_id: r.get(1)?,
                    name: r.get(2)?,
                    kind: CategoryKind::parse(&r.get::<_, String>(3)?),
                    color: r.get(4)?,
                    // O grupo decide (mesmo se um desfazer antigo mexeu só no `is_essential`).
                    is_essential: spend_group.map_or(
                        r.get::<_, i64>(5)? != 0,
                        crate::categories::SpendGroup::is_essential,
                    ),
                    spend_group,
                    archived: r.get::<_, i64>(6)? != 0,
                    sort_order: r.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let ids: BTreeSet<&str> = raws.iter().map(|r| r.id.as_str()).collect();
        let mut kids: HashMap<Option<Id>, Vec<&Raw>> = HashMap::new();
        for r in &raws {
            // Pai inexistente (não deveria acontecer com FK) vira raiz.
            let p = r
                .parent_id
                .clone()
                .filter(|p| ids.contains(p.as_str()) && *p != r.id);
            kids.entry(p).or_default().push(r);
        }
        for v in kids.values_mut() {
            v.sort_by(|a, b| {
                (
                    a.kind != CategoryKind::Expense,
                    a.sort_order,
                    &a.name,
                    &a.id,
                )
                    .cmp(&(
                        b.kind != CategoryKind::Expense,
                        b.sort_order,
                        &b.name,
                        &b.id,
                    ))
            });
        }
        let mut tree = CategoryTree::default();
        // DFS iterativa a partir das raízes; os pais entram em `by_id` antes dos filhos.
        // `seen` protege contra ciclos.
        let mut seen = BTreeSet::new();
        let mut stack: Vec<(&Raw, Option<Id>)> = Vec::new();
        if let Some(roots) = kids.get(&None) {
            for root in roots.iter().rev() {
                stack.push((root, None));
            }
        }
        while let Some((raw, parent_id)) = stack.pop() {
            if !seen.insert(raw.id.clone()) {
                continue;
            }
            let parent = parent_id.as_deref().and_then(|p| tree.by_id.get(p));
            let info = CatInfo {
                id: raw.id.clone(),
                parent_id: parent.map(|p| p.id.clone()),
                name: raw.name.clone(),
                kind: raw.kind,
                color: raw.color.clone(),
                is_essential: raw.is_essential,
                spend_group: raw.spend_group,
                archived: raw.archived,
                path: match parent {
                    Some(p) => format!("{}{}{}", p.path, crate::categories::PATH_SEP, raw.name),
                    None => raw.name.clone(),
                },
                root_id: parent
                    .map(|p| p.root_id.clone())
                    .unwrap_or_else(|| raw.id.clone()),
                depth: parent.map(|p| p.depth + 1).unwrap_or(0),
            };
            match &info.parent_id {
                Some(p) => tree
                    .children
                    .entry(p.clone())
                    .or_default()
                    .push(info.id.clone()),
                None => tree.roots.push(info.id.clone()),
            }
            tree.order.push(info.id.clone());
            tree.by_id.insert(info.id.clone(), info);
            if let Some(children) = kids.get(&Some(raw.id.clone())) {
                for c in children.iter().rev() {
                    stack.push((c, Some(raw.id.clone())));
                }
            }
        }
        Ok(tree)
    }

    pub fn get(&self, id: &str) -> Option<&CatInfo> {
        self.by_id.get(id)
    }

    pub fn children_of(&self, id: &str) -> &[Id] {
        self.children.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// A própria categoria e seus ancestrais, do mais próximo à raiz.
    pub fn ancestors_inclusive(&self, id: &str) -> Vec<&CatInfo> {
        let mut out = Vec::new();
        let mut cur = self.by_id.get(id);
        while let Some(c) = cur {
            out.push(c);
            if out.len() > 64 {
                break;
            }
            cur = c.parent_id.as_deref().and_then(|p| self.by_id.get(p));
        }
        out
    }

    /// Cor da categoria ou do ancestral mais próximo que tenha cor.
    pub fn color(&self, id: &str) -> Option<String> {
        self.ancestors_inclusive(id)
            .into_iter()
            .find_map(|c| c.color.clone())
    }

    /// Ancestral de nível 1 (filho direto da raiz) de uma categoria de nível ≥ 1.
    pub fn level1(&self, id: &str) -> Option<&CatInfo> {
        self.ancestors_inclusive(id)
            .into_iter()
            .find(|c| c.depth == 1)
    }
}

// ---------------------------------------------------------------------------
// Câmbio

/// Taxa usada numa conversão (para listar como fonte em `explain`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UsedRate {
    pub currency: String,
    pub rate_date: String,
    pub source: String,
    pub rate: String,
    pub estimated: bool,
}

/// Conversões para uma moeda-alvo com [`crate::fx::convert`] (na data do lançamento).
/// Anota as moedas sem câmbio (o valor fica fora do total) e se alguma taxa foi estimada.
pub(crate) struct Fx<'c> {
    conn: &'c Connection,
    pub target: String,
    pub missing: BTreeSet<String>,
    pub estimated: bool,
    pub used: BTreeSet<UsedRate>,
}

impl<'c> Fx<'c> {
    pub fn new(conn: &'c Connection, target: &str) -> Self {
        Self {
            conn,
            target: target.to_string(),
            missing: BTreeSet::new(),
            estimated: false,
            used: BTreeSet::new(),
        }
    }

    /// Converte `amount` (em `currency`) para a moeda-alvo na data. `None` sem câmbio.
    pub fn convert(
        &mut self,
        amount: i64,
        currency: &str,
        date: &str,
    ) -> Result<Option<Converted>> {
        if currency == self.target {
            return Ok(Some(Converted {
                amount,
                currency: self.target.clone(),
                rate: "1".into(),
                rate_date: None,
                source: None,
                estimated: false,
            }));
        }
        match crate::fx::convert(self.conn, amount, currency, &self.target, date)? {
            Some(c) => {
                if c.estimated {
                    self.estimated = true;
                }
                self.used.insert(UsedRate {
                    currency: currency.to_string(),
                    rate_date: c.rate_date.clone().unwrap_or_default(),
                    source: c.source.clone().unwrap_or_default(),
                    rate: c.rate.clone(),
                    estimated: c.estimated,
                });
                Ok(Some(c))
            }
            None => {
                self.missing.insert(currency.to_string());
                Ok(None)
            }
        }
    }

    pub fn missing_vec(&self) -> Vec<String> {
        self.missing.iter().cloned().collect()
    }
}

// ---------------------------------------------------------------------------
// Agregação do fluxo de caixa

/// Critério de agrupamento de uma agregação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Key {
    None,
    /// Categoria em que o valor foi lançado (a da parte, em divisões).
    Category,
    /// Categoria de primeiro nível.
    RootCategory,
    /// Titular (o da parte, ou o do lançamento se a parte não tiver).
    Member,
    Account,
    /// `YYYY-MM` da data efetiva (competência ou caixa).
    Month,
    /// Data efetiva.
    Day,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// Efetivados e conciliados.
    Realized,
    /// Só previstos.
    Planned,
}

/// Valor da categoria do lançamento ou da parte da divisão.
/// Tipo do lançamento no fluxo de caixa: receita e despesa como estão; transferência de uma
/// perna só (sem a outra conta da família) entra pela categoria, se tiver uma, com o sinal
/// dizendo se é despesa ou receita. O resto (transferências entre contas, fatura, aporte,
/// ajuste) fica fora.
pub(crate) const FLOW_KIND: &str = "(CASE WHEN t.kind IN ('income', 'expense') THEN t.kind \
     WHEN t.kind = 'transfer' AND t.transfer_id IS NULL AND t.category_id IS NOT NULL \
     THEN CASE WHEN t.amount < 0 THEN 'expense' ELSE 'income' END END)";

pub(crate) const CATEGORY_EXPR: &str =
    "CASE WHEN s.id IS NULL THEN t.category_id ELSE s.category_id END";
/// Titular de um lançamento: o da parte, o do lançamento ou, sem nenhum, o dono da conta
/// quando ela tem um titular só (conta conjunta sem titular no lançamento fica sem titular:
/// é da família). Ver docs/members.md.
pub(crate) const MEMBER_EXPR: &str = "COALESCE(s.member_id, t.member_id, \
     (SELECT CASE WHEN count(*) = 1 THEN min(am.member_id) END FROM account_member am \
       WHERE am.account_id = t.account_id))";
const ROOT_CTE: &str = "WITH RECURSIVE cat_root(id, root_id) AS (
    SELECT id, id FROM category WHERE parent_id IS NULL
    UNION ALL SELECT c.id, cr.root_id FROM category c JOIN cat_root cr ON c.parent_id = cr.id) ";

/// Parâmetros posicionais montados junto com o SQL.
#[derive(Default)]
pub(crate) struct Params(pub Vec<SqlValue>);

impl Params {
    pub fn bind(&mut self, v: impl Into<SqlValue>) -> String {
        self.0.push(v.into());
        format!("?{}", self.0.len())
    }
    fn bind_list(&mut self, ids: &[Id]) -> String {
        ids.iter()
            .map(|id| self.bind(id.clone()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Filtro de uma agregação de receitas/despesas.
#[derive(Debug, Clone)]
pub(crate) struct FlowQuery<'a> {
    pub from: String,
    pub to: String,
    pub basis: Basis,
    pub status: Status,
    /// `None` = receitas e despesas.
    pub kind: Option<CategoryKind>,
    pub member_ids: Option<&'a [Id]>,
    pub account_ids: Option<&'a [Id]>,
    /// Já expandido com os descendentes.
    pub category_ids: Option<&'a [Id]>,
    /// `Some(true)`: só os lançamentos que pagaram uma parcela de dívida
    /// (`loan_installment.txn_id`); `Some(false)`: só os que não pagaram.
    pub loan_linked: Option<bool>,
}

/// Soma de um grupo, já na moeda-alvo. `expense` é positivo para gasto.
#[derive(Debug, Clone, Default)]
pub(crate) struct Cell {
    pub income: i64,
    pub expense: i64,
    pub income_count: i64,
    pub expense_count: i64,
    pub missing_fx: BTreeSet<String>,
    pub estimated: bool,
}

impl Cell {
    pub fn amount(&self, kind: CategoryKind) -> i64 {
        match kind {
            CategoryKind::Income => self.income,
            CategoryKind::Expense => self.expense,
        }
    }
    pub fn count(&self, kind: CategoryKind) -> i64 {
        match kind {
            CategoryKind::Income => self.income_count,
            CategoryKind::Expense => self.expense_count,
        }
    }
    pub fn add(&mut self, o: &Cell) {
        self.income += o.income;
        self.expense += o.expense;
        self.income_count += o.income_count;
        self.expense_count += o.expense_count;
        self.missing_fx.extend(o.missing_fx.iter().cloned());
        self.estimated |= o.estimated;
    }
}

pub(crate) type GroupKey = (Option<String>, Option<String>);

#[derive(Debug, Default)]
pub(crate) struct Agg {
    pub cells: BTreeMap<GroupKey, Cell>,
    pub missing_fx: BTreeSet<String>,
    pub estimated: bool,
}

impl Agg {
    pub fn total(&self) -> Cell {
        let mut t = Cell::default();
        for c in self.cells.values() {
            t.add(c);
        }
        t.missing_fx.extend(self.missing_fx.iter().cloned());
        t.estimated |= self.estimated;
        t
    }
    pub fn missing_vec(&self) -> Vec<String> {
        self.missing_fx.iter().cloned().collect()
    }
}

impl<'a> FlowQuery<'a> {
    pub fn new(from: &str, to: &str, basis: Basis) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            basis,
            status: Status::Realized,
            kind: None,
            member_ids: None,
            account_ids: None,
            category_ids: None,
            loan_linked: None,
        }
    }

    /// Data efetiva do lançamento conforme a base.
    pub fn eff_expr(&self) -> &'static str {
        match self.basis {
            Basis::Accrual => "t.date",
            Basis::Cash => "COALESCE(cs.due_date, t.date)",
        }
    }

    fn key_expr(&self, k: Key) -> String {
        match k {
            Key::None => "NULL".into(),
            Key::Category => CATEGORY_EXPR.into(),
            Key::RootCategory => "r.root_id".into(),
            Key::Member => MEMBER_EXPR.into(),
            Key::Account => "t.account_id".into(),
            Key::Month => format!("substr({}, 1, 7)", self.eff_expr()),
            Key::Day => self.eff_expr().into(),
        }
    }

    /// `FROM … WHERE …` comum às consultas de fluxo.
    pub fn sql_tail(&self, p: &mut Params, root_join: bool) -> String {
        let mut s = String::from(" FROM txn t LEFT JOIN txn_split s ON s.txn_id = t.id");
        if self.basis == Basis::Cash {
            s.push_str(" LEFT JOIN card_statement cs ON cs.id = t.statement_id");
        }
        if root_join {
            s.push_str(&format!(
                " LEFT JOIN cat_root r ON r.id = ({CATEGORY_EXPR})"
            ));
        }
        s.push_str(&format!(
            " WHERE {FLOW_KIND} {}",
            match self.kind {
                None => "IN ('income', 'expense')",
                Some(CategoryKind::Income) => "= 'income'",
                Some(CategoryKind::Expense) => "= 'expense'",
            }
        ));
        s.push_str(match self.status {
            Status::Realized => " AND t.status IN ('cleared', 'reconciled')",
            Status::Planned => " AND t.status = 'planned'",
        });
        let (from, to) = (p.bind(self.from.clone()), p.bind(self.to.clone()));
        match self.basis {
            Basis::Accrual => s.push_str(&format!(" AND t.date BETWEEN {from} AND {to}")),
            // Duas condições indexáveis: data do lançamento (fora do cartão) ou fatura que
            // vence no período.
            Basis::Cash => s.push_str(&format!(
                " AND ((t.statement_id IS NULL AND t.date BETWEEN {from} AND {to})
                   OR t.statement_id IN (SELECT id FROM card_statement WHERE due_date BETWEEN {from} AND {to}))"
            )),
        }
        if let Some(ids) = self.account_ids.filter(|v| !v.is_empty()) {
            s.push_str(&format!(" AND t.account_id IN ({})", p.bind_list(ids)));
        }
        if let Some(ids) = self.member_ids.filter(|v| !v.is_empty()) {
            s.push_str(&format!(" AND {MEMBER_EXPR} IN ({})", p.bind_list(ids)));
        }
        if let Some(linked) = self.loan_linked {
            s.push_str(&format!(
                " AND t.id {} (SELECT txn_id FROM loan_installment WHERE txn_id IS NOT NULL)",
                if linked { "IN" } else { "NOT IN" }
            ));
        }
        if let Some(ids) = self.category_ids {
            if ids.is_empty() {
                s.push_str(" AND 0");
            } else {
                s.push_str(&format!(" AND ({CATEGORY_EXPR}) IN ({})", p.bind_list(ids)));
            }
        }
        s
    }

    /// Soma receitas e despesas agrupadas por até duas chaves, convertidas para `fx.target`.
    pub fn aggregate(&self, conn: &Connection, fx: &mut Fx<'_>, k1: Key, k2: Key) -> Result<Agg> {
        let root = k1 == Key::RootCategory || k2 == Key::RootCategory;
        let mut p = Params::default();
        let base = p.bind(fx.target.clone());
        let sql = format!(
            "{cte}SELECT {e1} AS k1, {e2} AS k2, {FLOW_KIND} AS fk, t.currency,
                    CASE WHEN t.currency = {base} THEN NULL ELSE t.id END AS fx_txn,
                    MIN(t.date), SUM(COALESCE(s.amount, t.amount)), COUNT(DISTINCT t.id)
             {fw}
             GROUP BY k1, k2, fk, t.currency, fx_txn",
            cte = if root { ROOT_CTE } else { "" },
            e1 = self.key_expr(k1),
            e2 = self.key_expr(k2),
            fw = self.sql_tail(&mut p, root),
        );
        let mut stmt = conn.prepare_cached(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(p.0.iter()))?;
        let mut agg = Agg::default();
        while let Some(r) = rows.next()? {
            let key: GroupKey = (r.get(0)?, r.get(1)?);
            let kind: String = r.get(2)?;
            let currency: String = r.get(3)?;
            let date: String = r.get(5)?;
            let amount: i64 = r.get(6)?;
            let count: i64 = r.get(7)?;
            let cell = agg.cells.entry(key).or_default();
            let Some(conv) = fx.convert(amount, &currency, &date)? else {
                cell.missing_fx.insert(currency.clone());
                agg.missing_fx.insert(currency);
                continue;
            };
            if conv.estimated {
                cell.estimated = true;
                agg.estimated = true;
            }
            if kind == "income" {
                cell.income += conv.amount;
                cell.income_count += count;
            } else {
                cell.expense -= conv.amount;
                cell.expense_count += count;
            }
        }
        Ok(agg)
    }
}

// ---------------------------------------------------------------------------
// Saldos

#[derive(Debug, Clone)]
pub(crate) struct AccountRow {
    pub id: Id,
    pub name: String,
    pub kind: crate::accounts::AccountKind,
    pub currency: String,
    pub include_in_net_worth: bool,
    pub archived: bool,
    pub opening_date: String,
    /// Saldo na data (efetivados + conciliados).
    pub balance: i64,
}

impl AccountRow {
    /// A conta já existia na data (aberta até ela) ou tem movimento até ela.
    pub fn exists_at(&self, as_of: &str) -> bool {
        self.opening_date.as_str() <= as_of || self.balance != 0
    }
}

/// Saldo de todas as contas numa data, numa consulta só. Igual a
/// [`crate::accounts::balance`] (saldo inicial + lançamentos efetivados até a data), exceto
/// antes da data de abertura da conta, quando o saldo inicial ainda não conta.
pub(crate) fn balances_at(conn: &Connection, as_of: &str) -> Result<Vec<AccountRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT a.id, a.name, a.kind, a.currency, a.include_in_net_worth, a.archived,
                a.opening_date,
                CASE WHEN a.opening_date <= ?1 THEN a.opening_balance ELSE 0 END
                + COALESCE((SELECT SUM(t.amount) FROM txn t WHERE t.account_id = a.id AND t.date <= ?1
                              AND t.status IN ('cleared', 'reconciled')), 0)
         FROM account a ORDER BY a.sort_order, a.name, a.id",
    )?;
    let rows = stmt
        .query_map([as_of], |r| {
            Ok(AccountRow {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: crate::accounts::AccountKind::parse(&r.get::<_, String>(2)?)
                    .unwrap_or(crate::accounts::AccountKind::Other),
                currency: r.get(3)?,
                include_in_net_worth: r.get::<_, i64>(4)? != 0,
                archived: r.get::<_, i64>(5)? != 0,
                opening_date: r.get(6)?,
                balance: r.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Primeiro mês com algum lançamento de receita/despesa efetivado (para não dividir médias
/// por meses anteriores ao uso do app).
pub(crate) fn first_flow_month(conn: &Connection) -> Result<Option<String>> {
    let d: Option<String> = conn.query_row(
        "SELECT MIN(date) FROM txn WHERE kind IN ('income', 'expense') AND status IN ('cleared', 'reconciled')",
        [],
        |r| r.get(0),
    )?;
    Ok(d.map(|d| d[..7].to_string()))
}

/// Os `n` meses completos anteriores a `before` (exclusivo), em ordem cronológica, sem os
/// meses anteriores a `first` (quando houver).
pub(crate) fn complete_months_before(
    before: &str,
    n: u32,
    first: Option<&str>,
) -> Result<Vec<String>> {
    let end = month_index(before)? - 1;
    let start = end - n as i32 + 1;
    let first = match first {
        Some(f) => month_index(f)?,
        None => i32::MIN,
    };
    Ok((start.max(first)..=end).map(month_from_index).collect())
}

/// `YYYY-MM` de uma data.
pub(crate) fn current_month_of(d: NaiveDate) -> String {
    fmt_month(d.year(), d.month())
}
