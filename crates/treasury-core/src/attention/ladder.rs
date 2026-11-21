//! "Seu próximo passo": em que degrau a família está e **uma** coisa para fazer agora
//! (docs/to-resolve.md). A nota de saúde financeira diz como a família está; o degrau diz
//! o que fazer primeiro.
//!
//! São seis degraus, sempre na mesma ordem, e o atual é o primeiro que ainda não foi cumprido:
//!
//! 1. `know`: saber para onde o dinheiro vai (3 meses completos de lançamentos e 80% dos
//!    últimos 90 dias com categoria);
//! 2. `balance`: gastar menos do que ganha (soma dos 3 últimos meses completos);
//! 3. `credit`: sair do crédito caro (sem rotativo do cartão nem cheque especial nos 3 últimos
//!    meses completos e no atual);
//! 4. `reserve`: montar a reserva de emergência (3 meses de gastos essenciais);
//! 5. `debts`: deixar as dívidas leves (parcelas de contratos em até 30% da renda);
//! 6. `invest`: investir e proteger o que tem (o topo: não se "cumpre").
//!
//! Os fatos são os da saúde financeira (`health::facts::Facts`): nenhuma conta é refeita aqui.
//! Sem dados, sem penalidade: o que não dá para medir (sem conta corrente, sem contratos)
//! conta como cumprido, e a falta de lançamentos é o próprio degrau 1. Os textos trazem meses
//! e porcentagens, nunca valores em dinheiro (como os da saúde financeira).

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ai::alerts::{Arg, Msg};
use crate::error::Result;
use crate::health::facts::Facts;
use crate::i18n;
use crate::reports::engine::shift_month;
use crate::time::{fmt_date, month_of};

/// Meses completos de lançamentos para o degrau 1.
pub const KNOW_MONTHS: u32 = 3;
/// Fração dos lançamentos dos últimos 90 dias com categoria para o degrau 1.
pub const KNOW_CATEGORIZED: f64 = 0.8;
/// Meses de gastos essenciais guardados para o degrau 4.
pub const RESERVE_MONTHS: f64 = 3.0;
/// Fração da renda em parcelas de contratos até a qual as dívidas estão leves.
pub const DEBT_SHARE: f64 = 0.30;

pub const RUNGS: [&str; 6] = ["know", "balance", "credit", "reserve", "debts", "invest"];

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LadderRung {
    pub key: String,
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NextStep {
    /// O degrau atual, de 1 a 6.
    pub rung: u32,
    pub key: String,
    pub title: String,
    /// Por que este é o degrau, com os números.
    pub why: String,
    /// Uma coisa para fazer agora.
    pub action: String,
    /// Onde fazer (rota do app).
    pub route: Option<String>,
    /// Os seis degraus, com os já cumpridos.
    pub rungs: Vec<LadderRung>,
    /// Leitura provisória: há conta do dia a dia sem extrato recente.
    pub estimated: bool,
}

/// O que cada degrau precisa dizer quando é o atual.
struct Pending {
    why: Msg,
    action: Msg,
    route: String,
}

fn title(key: &str) -> String {
    i18n::t(&format!("degraus-{key}-titulo"))
}

/// Lançamentos de receita e despesa dos últimos 90 dias: (com categoria, total).
fn categorized(conn: &Connection, today: NaiveDate) -> Result<(i64, i64, String)> {
    let from = fmt_date(today - Duration::days(90));
    let (with, total): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(t.category_id IS NOT NULL
                            OR EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)), 0),
                count(*)
           FROM txn t
          WHERE t.kind IN ('income', 'expense') AND t.status != 'planned'
            AND t.date BETWEEN ?1 AND ?2",
        params![from, fmt_date(today)],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok((with, total, from))
}

fn know(conn: &Connection, f: &Facts, today: NaiveDate) -> Result<Option<Pending>> {
    let accounts: i64 =
        conn.query_row("SELECT count(*) FROM account WHERE archived = 0", [], |r| {
            r.get(0)
        })?;
    if accounts == 0 {
        return Ok(Some(Pending {
            why: Msg::new("degraus-know-sem-contas"),
            action: Msg::new("degraus-know-sem-contas-acao"),
            route: "/contas".into(),
        }));
    }
    if f.n() < KNOW_MONTHS {
        return Ok(Some(Pending {
            why: Msg::new("degraus-know-meses")
                .arg("n", f.n() as i64)
                .arg("alvo", KNOW_MONTHS as i64),
            action: Msg::new("degraus-know-meses-acao"),
            route: "/lancamentos?importar=1".into(),
        }));
    }
    let (with, total, from) = categorized(conn, today)?;
    if total > 0 && (with as f64) < KNOW_CATEGORIZED * total as f64 {
        return Ok(Some(Pending {
            why: Msg::new("degraus-know-categorias")
                .arg("pct", Arg::pct(with as f64 / total as f64, 0))
                .arg("alvo", Arg::pct(KNOW_CATEGORIZED, 0)),
            action: Msg::new("degraus-know-categorias-acao").arg("n", total - with),
            route: format!(
                "/lancamentos?uncategorized=1&from={from}&to={}",
                fmt_date(today)
            ),
        }));
    }
    Ok(None)
}

fn balance(f: &Facts) -> Option<Pending> {
    let last = &f.flows[f.flows.len().saturating_sub(3)..];
    let income: i64 = last.iter().map(|m| m.income).sum();
    let expense: i64 = last.iter().map(|m| m.expense).sum();
    if income <= 0 {
        return Some(Pending {
            why: Msg::new("degraus-balance-sem-renda").arg("n", last.len() as i64),
            action: Msg::new("degraus-balance-sem-renda-acao"),
            route: "/lancamentos?novo=receita".into(),
        });
    }
    if expense <= income {
        return None;
    }
    let over = last.iter().filter(|m| m.expense > m.income).count() as i64;
    Some(Pending {
        why: Msg::new("degraus-balance-acima")
            .arg("n", last.len() as i64)
            .arg("meses", over)
            .arg(
                "pct",
                Arg::pct((expense - income) as f64 / income as f64, 0),
            ),
        action: Msg::new("degraus-balance-acima-acao"),
        route: "/orcamento".into(),
    })
}

fn credit(f: &Facts, today: NaiveDate) -> Result<Option<Pending>> {
    let current = month_of(today);
    let from = shift_month(&current, -3)?;
    let recent = |m: &&String| m.as_str() >= from.as_str() && m.as_str() <= current.as_str();
    let revolving = f.revolving_months.iter().filter(recent).count() as i64;
    let overdraft = f.overdraft_months.iter().filter(recent).count() as i64;
    if revolving == 0 && overdraft == 0 {
        return Ok(None);
    }
    Ok(Some(Pending {
        why: Msg::new(match (revolving > 0, overdraft > 0) {
            (true, true) => "degraus-credit-ambos",
            (true, false) => "degraus-credit-rotativo",
            _ => "degraus-credit-cheque",
        })
        .arg("rotativo", revolving)
        .arg("cheque", overdraft),
        action: Msg::new("degraus-credit-acao"),
        route: "/dividas".into(),
    }))
}

fn reserve(f: &Facts) -> Option<Pending> {
    let last = &f.flows[f.flows.len().saturating_sub(3)..];
    // Sem categoria essencial marcada, a referência é o gasto médio dos últimos meses.
    let (base, essential) = if f.essential_avg > 0 {
        (f.essential_avg, true)
    } else {
        let total: i64 = last.iter().map(|m| m.expense).sum();
        (total / last.len().max(1) as i64, false)
    };
    if base <= 0 {
        return None;
    }
    let months = f.reserve.max(0) as f64 / base as f64;
    if months >= RESERVE_MONTHS {
        return None;
    }
    let target = if months < 1.0 {
        1
    } else {
        RESERVE_MONTHS as i64
    };
    Some(Pending {
        why: Msg::new(if essential {
            "degraus-reserve-essenciais"
        } else {
            "degraus-reserve-gastos"
        })
        .arg("meses", Arg::dec(months, 1))
        .arg("alvo", target),
        action: Msg::new("degraus-reserve-acao"),
        route: "/orcamento?aba=reserva".into(),
    })
}

fn debts(f: &Facts) -> Option<Pending> {
    if f.loans == 0 || f.income <= 0 {
        return None;
    }
    let share = f.debt_service as f64 / f.income as f64;
    if share <= DEBT_SHARE {
        return None;
    }
    Some(Pending {
        why: Msg::new("degraus-debts-peso")
            .arg("pct", Arg::pct(share, 0))
            .arg("alvo", Arg::pct(DEBT_SHARE, 0)),
        action: Msg::new("degraus-debts-acao"),
        route: "/dividas".into(),
    })
}

fn invest(conn: &Connection) -> Result<Pending> {
    let invested: bool = conn.query_row("SELECT EXISTS (SELECT 1 FROM trade)", [], |r| r.get(0))?;
    Ok(if invested {
        Pending {
            why: Msg::new("degraus-invest-porque"),
            action: Msg::new("degraus-invest-carteira-acao"),
            route: "/carteira?aba=alocacao".into(),
        }
    } else {
        Pending {
            why: Msg::new("degraus-invest-porque"),
            action: Msg::new("degraus-invest-comecar-acao"),
            route: "/orcamento?aba=metas".into(),
        }
    })
}

/// O degrau atual na data `today`.
pub fn next_step(conn: &Connection, today: NaiveDate) -> Result<NextStep> {
    let f = Facts::load(conn, &month_of(today), today, None)?;
    // Na ordem: o primeiro que tem algo pendente é o degrau atual.
    let mut pending: Option<(usize, Pending)> = None;
    if let Some(p) = know(conn, &f, today)? {
        pending = Some((0, p));
    } else if let Some(p) = balance(&f) {
        pending = Some((1, p));
    } else if let Some(p) = credit(&f, today)? {
        pending = Some((2, p));
    } else if let Some(p) = reserve(&f) {
        pending = Some((3, p));
    } else if let Some(p) = debts(&f) {
        pending = Some((4, p));
    }
    let (at, p) = match pending {
        Some(x) => x,
        None => (5, invest(conn)?),
    };
    let l = i18n::current();
    Ok(NextStep {
        rung: at as u32 + 1,
        key: RUNGS[at].into(),
        title: title(RUNGS[at]),
        why: p.why.render(l),
        action: p.action.render(l),
        route: Some(p.route),
        rungs: RUNGS
            .iter()
            .enumerate()
            .map(|(i, k)| LadderRung {
                key: (*k).into(),
                title: title(k),
                done: i < at,
            })
            .collect(),
        estimated: !f.stale.is_empty(),
    })
}
