//! Os fatos que os modelos de saúde financeira avaliam, lidos do cofre com as mesmas regras
//! dos relatórios (só receitas e despesas contam; transferências, pagamento de fatura e
//! aportes não; base do ajuste `budget_basis`; valores na moeda base).
//!
//! Janela: até 12 meses completos terminando no mês anterior ao atual (ou no mês pedido, se
//! for passado), sem os meses anteriores ao primeiro lançamento.

use std::collections::BTreeSet;

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, params};

use crate::budget::Basis;
use crate::categories::CategoryKind;
use crate::error::Result;
use crate::household::base_currency;
use crate::reports::engine::{
    CategoryTree, FlowQuery, Fx, Key, balances_at, current_month, div_round, first_flow_month,
    month_dates, months_between, resolve_basis, shift_month,
};
use crate::reports::member::MemberScope;
use crate::reports::{MonthFlow, NetWorthPoint, Period};
use crate::time::{fmt_date, month_of, parse_date};

/// Meses mínimos de histórico para avaliar (abaixo disso, os indicadores ficam sem dados).
pub const MIN_MONTHS: u32 = 3;
/// Meses da janela.
const WINDOW: i32 = 12;
/// Meses da média de gastos essenciais (como a reserva de emergência das metas).
const ESSENTIAL_MONTHS: usize = 6;
/// Folga do pagamento depois do vencimento (compensação, fim de semana, conciliação).
pub(crate) const GRACE_DAYS: i64 = 5;
/// Diferença tolerada entre a fatura e o pago (centavos de arredondamento).
const BILL_TOLERANCE: i64 = 100;
/// Mês `AAAA-MM` abreviado no idioma da instalação (`out/2026`, `Oct 2026`).
pub(crate) fn month_label(month: &str) -> String {
    crate::ai::alerts::month_short(crate::i18n::current(), month)
}

/// Nomes da categoria de moradia (raiz de despesa), em pt-BR: a categoria padrão é achada pelo
/// nome em pt-BR em qualquer idioma (`categories::br_names`).
const HOUSING_NAMES: &[&str] = &["moradia", "habitação", "habitacao", "casa"];

pub(crate) struct Facts {
    /// Mês avaliado.
    pub as_of: String,
    pub basis: Basis,
    /// Meses completos analisados, em ordem.
    pub months: Vec<String>,
    pub flows: Vec<MonthFlow>,
    pub income: i64,
    pub expense: i64,
    /// Média mensal das despesas essenciais (`is_essential`) dos últimos meses da janela.
    pub essential_avg: i64,
    pub essential_months: u32,
    /// Reserva: a meta de emergência (mês atual) ou o saldo das contas corrente, poupança e
    /// dinheiro.
    pub reserve: i64,
    pub reserve_from_goal: bool,
    /// Soma das parcelas de empréstimos que venceram na janela.
    pub debt_service: i64,
    /// Contratos com parcela na janela.
    pub loans: u32,
    pub installments_on_time: u32,
    pub installments_late: u32,
    pub bills_on_time: u32,
    pub bills_late: u32,
    /// Cartões com pagamento de fatura registrado (só esses entram na pontualidade).
    pub cards_tracked: u32,
    /// Meses (de vencimento) com fatura não paga inteira até o vencimento: rotativo.
    pub revolving_months: BTreeSet<String>,
    pub checking_accounts: u32,
    /// Meses em que alguma conta corrente fechou um dia negativa (cheque especial), contando
    /// o mês corrente até hoje.
    pub overdraft_months: BTreeSet<String>,
    /// Contas correntes negativas por 30 dias ou mais sem o cheque especial cadastrado:
    /// cheque especial de fato ou importação incompleta (falta o saldo inicial ou um extrato).
    pub overdraft_suspect: Vec<String>,
    pub budget_months: u32,
    pub budget_within: u32,
    /// Despesas de moradia (+ parcelas de financiamento imobiliário fora dela) na janela;
    /// `None` sem categoria de moradia nem financiamento.
    pub housing: Option<i64>,
    /// Patrimônio líquido mês a mês (até 13 pontos, terminando no mês avaliado).
    pub net_worth: Vec<NetWorthPoint>,
    pub currency: String,
    pub missing_fx: BTreeSet<String>,
    pub estimated: bool,
    /// Contas que entram na nota e não estão em dia (mês atual).
    pub stale: Vec<crate::health::StaleInput>,
}

impl Facts {
    pub fn n(&self) -> u32 {
        self.months.len() as u32
    }

    /// Há meses suficientes para avaliar.
    pub fn enough(&self) -> bool {
        self.n() >= MIN_MONTHS
    }

    pub fn period(&self) -> Option<Period> {
        let (a, b) = (self.months.first()?, self.months.last()?);
        Some(Period {
            from: month_dates(a).ok()?.0,
            to: month_dates(b).ok()?.1,
        })
    }

    /// `nov/2025–out/2026` (no idioma da instalação: `Nov 2025–Oct 2026`).
    pub fn period_label(&self) -> String {
        match (self.months.first(), self.months.last()) {
            (Some(a), Some(b)) if a == b => month_label(a),
            (Some(a), Some(b)) => format!("{}–{}", month_label(a), month_label(b)),
            _ => crate::i18n::t("nucleo-saude-sem-meses"),
        }
    }

    fn empty(as_of: &str, basis: Basis, currency: String) -> Self {
        Facts {
            as_of: as_of.to_string(),
            basis,
            months: vec![],
            flows: vec![],
            income: 0,
            expense: 0,
            essential_avg: 0,
            essential_months: 0,
            reserve: 0,
            reserve_from_goal: false,
            debt_service: 0,
            loans: 0,
            installments_on_time: 0,
            installments_late: 0,
            bills_on_time: 0,
            bills_late: 0,
            cards_tracked: 0,
            revolving_months: BTreeSet::new(),
            checking_accounts: 0,
            overdraft_months: BTreeSet::new(),
            overdraft_suspect: Vec::new(),
            budget_months: 0,
            budget_within: 0,
            housing: None,
            net_worth: vec![],
            currency,
            missing_fx: BTreeSet::new(),
            estimated: false,
            stale: vec![],
        }
    }

    /// `scope`: a saúde de um membro (lançamentos em que é titular, a fração dele nas contas,
    /// os contratos dele; sem orçamento, que é da família).
    pub fn load(
        conn: &Connection,
        as_of: &str,
        today: NaiveDate,
        scope: Option<&MemberScope>,
    ) -> Result<Facts> {
        let member_ids = scope.map(|s| s.ids.as_slice());
        let share = |id: &str| crate::reports::member::account_share(scope, id);
        let current = month_of(today);
        let is_current = as_of == current;
        let end = if is_current {
            shift_month(as_of, -1)?
        } else {
            as_of.to_string()
        };
        // Data de referência para atrasos: hoje, ou o fim do mês avaliado.
        let ref_date = if is_current {
            today
        } else {
            parse_date(&month_dates(as_of)?.1)?
        };
        let basis = resolve_basis(conn, None)?;
        let currency = base_currency(conn)?;
        let mut f = Facts::empty(as_of, basis, currency.clone());
        let months = match first_flow_month(conn)? {
            Some(first) if first <= end => {
                let start = shift_month(&end, -(WINDOW - 1))?;
                months_between(start.max(first).as_str(), &end)?
            }
            _ => vec![],
        };
        let (Some(first), Some(last)) = (months.first(), months.last()) else {
            return Ok(f);
        };
        let (from, _) = month_dates(first)?;
        let (_, to) = month_dates(last)?;
        f.months = months.clone();
        if is_current {
            f.stale = stale_inputs(conn, today, scope)?;
            f.estimated |= !f.stale.is_empty();
        }
        let mut fx = Fx::new(conn, &currency);
        let tree = CategoryTree::load(conn)?;

        // Fluxo mês a mês.
        f.flows = crate::reports::month_flow_scoped(conn, first, last, basis, scope)?;
        for m in &f.flows {
            f.income += m.income;
            f.expense += m.expense;
            f.missing_fx.extend(m.missing_fx.iter().cloned());
            f.estimated |= m.estimated;
        }

        // Gastos essenciais e reserva.
        let ess = &months[months.len().saturating_sub(ESSENTIAL_MONTHS)..];
        if is_current && current == current_month() && scope.is_none() {
            // Mês atual: a mesma conta da reserva de emergência das metas.
            let er = crate::goals::emergency_reserve(
                conn,
                &crate::goals::EmergencyReserveArgs {
                    target_months: None,
                    months_back: Some(ESSENTIAL_MONTHS as u32),
                    include_annual: None,
                },
            )?;
            f.essential_avg = er.avg_essential_monthly;
            f.essential_months = er.months_analyzed;
            f.reserve = er.current_amount;
            f.reserve_from_goal = er.goal_id.is_some();
            f.missing_fx.extend(er.missing_fx);
            f.estimated |= er.estimated;
        } else {
            let (a, _) = month_dates(&ess[0])?;
            let (_, b) = month_dates(ess.last().expect("ao menos um mês"))?;
            let mut q = FlowQuery::new(&a, &b, basis);
            q.kind = Some(CategoryKind::Expense);
            q.member_ids = member_ids;
            let agg = q.aggregate(conn, &mut fx, Key::Category, Key::None)?;
            let essential: i64 = agg
                .cells
                .iter()
                .filter(|((c, _), _)| {
                    c.as_deref()
                        .and_then(|c| tree.get(c))
                        .is_some_and(|c| c.is_essential)
                })
                .map(|(_, cell)| cell.expense)
                .sum();
            f.essential_avg = div_round(essential, ess.len() as i64);
            f.essential_months = ess.len() as u32;
            let at = fmt_date(ref_date);
            for a in balances_at(conn, &at)? {
                use crate::accounts::AccountKind::*;
                if a.archived
                    || !a.include_in_net_worth
                    || !matches!(a.kind, Checking | Savings | Cash)
                    || share(&a.id) <= 0.0
                {
                    continue;
                }
                let balance = crate::reports::member::scale(a.balance, share(&a.id));
                // Conta no vermelho é dívida (cheque especial), não reduz a reserva.
                if let Some(c) = fx.convert(balance, &a.currency, &at)? {
                    f.reserve += c.amount.max(0);
                }
            }
        }

        // Moradia: raízes de despesa com nome de moradia (o atual ou o da categoria padrão
        // em pt-BR, para valer em qualquer idioma).
        let br_names = crate::categories::br_names(conn)?;
        let is_housing = |n: &str| HOUSING_NAMES.contains(&n.trim().to_lowercase().as_str());
        let housing_roots: BTreeSet<String> = tree
            .roots
            .iter()
            .filter_map(|id| tree.get(id))
            .filter(|c| {
                c.kind == CategoryKind::Expense
                    && (is_housing(&c.name) || br_names.get(&c.id).is_some_and(|n| is_housing(n)))
            })
            .map(|c| c.id.clone())
            .collect();
        let in_housing = |cat: Option<&str>| {
            cat.and_then(|c| tree.get(c))
                .is_some_and(|c| housing_roots.contains(&c.root_id))
        };

        // Parcelas de empréstimos e financiamentos.
        let grace = Duration::days(GRACE_DAYS);
        let mut loans = BTreeSet::new();
        let mut mortgage_outside_housing = 0i64;
        let mut has_mortgage = false;
        {
            let mut stmt = conn.prepare(
                "SELECT li.due_date, li.total, li.status, li.paid_date, l.id, l.kind, l.category_id, l.currency
                   FROM loan_installment li JOIN loan l ON l.id = li.loan_id
                  WHERE l.archived = 0 AND li.due_date BETWEEN ?1 AND ?2
                    AND (?3 IS NULL OR l.member_id = ?3)
                  ORDER BY li.due_date",
            )?;
            let mut rows = stmt.query(params![from, to, scope.map(|s| s.member_id.as_str())])?;
            while let Some(r) = rows.next()? {
                let due: String = r.get(0)?;
                let total: i64 = r.get(1)?;
                let status: String = r.get(2)?;
                let paid_date: Option<String> = r.get(3)?;
                let loan: String = r.get(4)?;
                let kind: String = r.get(5)?;
                let category: Option<String> = r.get(6)?;
                let cur: String = r.get(7)?;
                let Some(c) = fx.convert(total, &cur, &due)? else {
                    continue;
                };
                f.debt_service += c.amount;
                loans.insert(loan);
                if kind == "mortgage" {
                    has_mortgage = true;
                    if !in_housing(category.as_deref()) {
                        mortgage_outside_housing += c.amount;
                    }
                }
                let limit = parse_date(&due)? + grace;
                if status == "paid" {
                    // Sem data de pagamento (marcada como paga): no vencimento.
                    let late = paid_date
                        .as_deref()
                        .and_then(|p| parse_date(p).ok())
                        .is_some_and(|p| p > limit);
                    if late {
                        f.installments_late += 1;
                    } else {
                        f.installments_on_time += 1;
                    }
                } else if limit < ref_date {
                    f.installments_late += 1;
                }
            }
        }
        f.loans = loans.len() as u32;

        // Faturas de cartão vencidas na janela, dos cartões com pagamento registrado.
        {
            let mut stmt = conn.prepare(
                "SELECT s.account_id, substr(s.due_date, 1, 7), s.due_date,
                        COALESCE(SUM(CASE WHEN t.kind <> 'card_payment' AND t.status <> 'planned' THEN -t.amount END), 0),
                        COALESCE(SUM(CASE WHEN t.kind = 'card_payment' AND t.status <> 'planned'
                                           AND t.date <= date(s.due_date, ?3) THEN t.amount END), 0)
                   FROM card_statement s LEFT JOIN txn t ON t.statement_id = s.id
                  WHERE s.due_date BETWEEN ?1 AND ?2
                    AND EXISTS (SELECT 1 FROM txn p WHERE p.account_id = s.account_id
                                AND p.kind = 'card_payment' AND p.status <> 'planned')
                  GROUP BY s.id ORDER BY s.due_date",
            )?;
            let mut rows = stmt.query(params![from, to, format!("+{GRACE_DAYS} days")])?;
            let mut cards = BTreeSet::new();
            while let Some(r) = rows.next()? {
                let account: String = r.get(0)?;
                if share(&account) <= 0.0 {
                    continue;
                }
                let month: String = r.get(1)?;
                let due: String = r.get(2)?;
                let total: i64 = r.get(3)?;
                let paid: i64 = r.get(4)?;
                cards.insert(account);
                if total <= 0 || parse_date(&due)? + grace >= ref_date {
                    continue;
                }
                if paid + BILL_TOLERANCE >= total {
                    f.bills_on_time += 1;
                } else {
                    f.bills_late += 1;
                    f.revolving_months.insert(month);
                }
            }
            f.cards_tracked = cards.len() as u32;
        }

        // Cheque especial: dias com saldo negativo nas contas correntes.
        // Mesma conta do insight de cheque especial, até hoje no mês corrente (um saldo
        // negativo recente não pode esperar o mês fechar).
        let until = if is_current {
            fmt_date(today)
        } else {
            to.clone()
        };
        overdraft(conn, &mut f, &from, &until, scope)?;

        // Orçamento: meses da janela com orçamento de despesas.
        let budgeted: BTreeSet<String> = if scope.is_some() {
            BTreeSet::new()
        } else {
            crate::budget::list_months(conn)?.into_iter().collect()
        };
        for m in months.iter().filter(|m| budgeted.contains(*m)) {
            let b = crate::budget::get(conn, m, Some(basis))?;
            if b.totals.planned_expense > 0 {
                f.budget_months += 1;
                if b.totals.actual_expense <= b.totals.planned_expense {
                    f.budget_within += 1;
                }
            }
        }

        // Moradia na janela.
        if !housing_roots.is_empty() {
            let mut q = FlowQuery::new(&from, &to, basis);
            q.kind = Some(CategoryKind::Expense);
            q.member_ids = member_ids;
            let agg = q.aggregate(conn, &mut fx, Key::RootCategory, Key::None)?;
            let spent: i64 = agg
                .cells
                .iter()
                .filter(|((k, _), _)| k.as_ref().is_some_and(|k| housing_roots.contains(k)))
                .map(|(_, c)| c.expense)
                .sum();
            f.housing = Some(spent + mortgage_outside_housing);
        } else if has_mortgage {
            f.housing = Some(mortgage_outside_housing);
        }

        // Patrimônio: 13 pontos (12 meses de variação) até o mês avaliado.
        f.net_worth =
            crate::reports::net_worth_history_scoped(conn, WINDOW as u32 + 1, as_of, scope)?;
        for p in &f.net_worth {
            f.missing_fx.extend(p.missing_fx.iter().cloned());
        }

        f.missing_fx.extend(fx.missing.iter().cloned());
        f.estimated |= fx.estimated;
        Ok(f)
    }
}

/// Marca os meses em que alguma conta corrente terminou um dia com saldo negativo (a mesma
/// conta do insight de cheque especial: `accounts::overdraft::negative_spell`) e as contas
/// negativas por 30 dias ou mais sem o cheque especial cadastrado.
fn overdraft(
    conn: &Connection,
    f: &mut Facts,
    from: &str,
    to: &str,
    scope: Option<&MemberScope>,
) -> Result<()> {
    let accounts: Vec<(String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT id, name FROM account
              WHERE kind = 'checking' AND opening_date <= ?1
                AND (archived = 0 OR EXISTS (SELECT 1 FROM txn t WHERE t.account_id = account.id
                                              AND t.date BETWEEN ?2 AND ?1))
              ORDER BY sort_order, name",
        )?;
        stmt.query_map(params![to, from], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<Vec<(String, String)>>>()?
            .into_iter()
            .filter(|(id, _)| crate::reports::member::account_share(scope, id) > 0.0)
            .collect()
    };
    f.checking_accounts = accounts.len() as u32;
    let (from_d, to_d) = (parse_date(from)?, parse_date(to)?);
    for (id, name) in accounts {
        // Conta desatualizada (mês atual): o vermelho pode ser lançamento faltando, não
        // necessariamente cheque especial; fica em `stale` para conferir.
        if f.stale.iter().any(|s| s.account_id == id) {
            continue;
        }
        let spell = crate::accounts::overdraft::negative_spell(conn, &id, from_d, to_d)?;
        f.overdraft_months.extend(spell.months);
        if spell.days >= SUSPECT_DAYS && crate::accounts::overdraft::load(conn, &id)?.is_none() {
            f.overdraft_suspect.push(name);
        }
    }
    Ok(())
}

/// As contas que entram na nota (no patrimônio, de movimento do dia a dia) e não estão em dia.
fn stale_inputs(
    conn: &Connection,
    today: NaiveDate,
    scope: Option<&MemberScope>,
) -> Result<Vec<crate::health::StaleInput>> {
    Ok(
        crate::accounts::freshness::everyday_incomplete(conn, today)?
            .into_iter()
            .filter(|a| crate::reports::member::account_share(scope, &a.account.id) > 0.0)
            .map(|a| crate::health::StaleInput {
                negative: a.balance < 0 && !a.account.kind.is_liability(),
                balance: a.balance,
                currency: a.account.currency.clone(),
                last_txn_date: a.last_txn_date.clone(),
                staleness_days: a.staleness_days,
                missed_month: a.missed_month.clone(),
                kind: a.account.kind,
                name: a.account.name.clone(),
                account_id: a.account.id.clone(),
            })
            .collect(),
    )
}

/// Dias no vermelho sem o cheque especial cadastrado que pedem conferência.
const SUSPECT_DAYS: i64 = 30;
