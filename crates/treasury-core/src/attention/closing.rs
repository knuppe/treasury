//! Fechar o mês em cinco passos (docs/to-resolve.md): um roteiro curto, sem IA, para o mês
//! que acabou de terminar. Toda a inteligência do app depende de dados completos, e a causa
//! mais comum de número errado não é bug: é extrato que não foi importado.
//!
//! Os passos, na ordem:
//!
//! 1. `statements`: trazer os extratos das contas paradas (a cobertura de `attention`);
//! 2. `categories`: dar categoria aos lançamentos do mês;
//! 3. `balances`: conferir o saldo de cada conta com o do banco (a pessoa marca);
//! 4. `review`: resolver o que está em "Para resolver" (vencimentos, dados e tarefas);
//! 5. `summary`: ler o resumo do mês (a pessoa marca).
//!
//! Os passos 1, 2 e 4 são medidos na hora; o 3 e o 5 ficam no ajuste `attention.month_close`,
//! junto com a data em que a pessoa fechou o mês. Mês fechado não volta a aparecer, mesmo que
//! um lançamento novo chegue depois.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Attention, AttentionSource, CoverageStatus};
use crate::ai::alerts::{Arg, Msg};
use crate::error::{Error, Result};
use crate::i18n;
use crate::reports::engine::{month_dates, shift_month};
use crate::time::{month_of, now_utc};

/// Ajuste com o estado de cada mês: `YYYY-MM` → o que a pessoa marcou.
pub const SETTING: &str = "attention.month_close";
/// Meses guardados no ajuste.
const KEEP_MONTHS: usize = 24;

pub const STEPS: [&str; 5] = ["statements", "categories", "balances", "review", "summary"];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    balances: bool,
    #[serde(default)]
    summary: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    closed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CloseStep {
    pub key: String,
    pub title: String,
    /// O que falta, com os números; vazio quando está feito.
    pub detail: String,
    pub done: bool,
    /// A pessoa marca (os demais são medidos na hora).
    pub manual: bool,
    /// Onde fazer (rota do app).
    pub route: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthClose {
    /// O mês do roteiro (`YYYY-MM`): por padrão, o último que terminou.
    pub month: String,
    /// Há lançamentos no mês: sem eles não há o que fechar e a interface não mostra o roteiro.
    pub applicable: bool,
    pub steps: Vec<CloseStep>,
    #[ts(type = "number")]
    pub done: i64,
    /// Quando a pessoa fechou o mês.
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthCloseArgs {
    /// Mês `YYYY-MM` (padrão: o último que terminou).
    #[serde(default)]
    #[ts(optional)]
    pub month: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonthCloseSetArgs {
    pub month: String,
    /// `balances`, `summary` (os passos que a pessoa marca) ou `closed` (fechar ou reabrir).
    pub step: String,
    pub done: bool,
}

fn stored(conn: &Connection) -> Result<BTreeMap<String, Stored>> {
    crate::settings::get_or(conn, SETTING, BTreeMap::new())
}

/// O mês pedido ou o último que terminou; nunca o atual nem um futuro.
fn resolve_month(month: Option<&str>, today: NaiveDate) -> Result<String> {
    let last = shift_month(&month_of(today), -1)?;
    match month {
        None => Ok(last),
        Some(m) => {
            month_dates(m)?;
            if m > last.as_str() {
                return Err(Error::invalid(i18n::t("fechamento-erro-mes")));
            }
            Ok(m.to_string())
        }
    }
}

/// Lançamentos de receita e despesa do mês: (sem categoria, total).
fn uncategorized(conn: &Connection, from: &str, to: &str) -> Result<(i64, i64)> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(t.category_id IS NULL
                            AND NOT EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)), 0),
                count(*)
           FROM txn t
          WHERE t.kind IN ('income', 'expense') AND t.status != 'planned'
            AND t.date BETWEEN ?1 AND ?2",
        params![from, to],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}

/// O roteiro do mês, com a lista "Para resolver" já calculada.
pub fn month_close(
    conn: &Connection,
    args: &MonthCloseArgs,
    attention: &Attention,
    today: NaiveDate,
) -> Result<MonthClose> {
    let l = i18n::current();
    let month = resolve_month(args.month.as_deref(), today)?;
    let (from, to) = month_dates(&month)?;
    let st = stored(conn)?.remove(&month).unwrap_or_default();
    let (missing, total) = uncategorized(conn, &from, &to)?;

    let stale = &attention.coverage.stale;
    // Vencimentos, dados e tarefas ainda abertos, fora os que os passos 1 e 2 já cobrem.
    let open = attention
        .items
        .iter()
        .filter(|i| {
            matches!(
                i.source,
                AttentionSource::Due | AttentionSource::Data | AttentionSource::Todo
            ) && !matches!(i.kind.as_str(), "stale_accounts" | "uncategorized")
        })
        .count() as i64;

    let step = |key: &str, done: bool, manual: bool, detail: Option<Msg>, route: &str| CloseStep {
        key: key.into(),
        title: i18n::t(&format!("fechamento-{key}-titulo")),
        detail: if done {
            String::new()
        } else {
            detail.map(|m| m.render(l)).unwrap_or_default()
        },
        done,
        manual,
        route: Some(route.into()),
    };
    let steps = vec![
        step(
            "statements",
            attention.coverage.status != CoverageStatus::Partial,
            false,
            Some(
                Msg::new("fechamento-statements-falta")
                    .arg("n", stale.len() as i64)
                    .arg(
                        "contas",
                        Arg::join(
                            stale.iter().map(|a| Arg::Text(a.name.clone())).collect(),
                            ", ",
                        ),
                    ),
            ),
            "/lancamentos?importar=1",
        ),
        step(
            "categories",
            missing == 0,
            false,
            Some(Msg::new("fechamento-categories-falta").arg("n", missing)),
            &format!("/lancamentos?uncategorized=1&from={from}&to={to}"),
        ),
        step(
            "balances",
            st.balances,
            true,
            Some(Msg::new("fechamento-balances-falta")),
            "/contas",
        ),
        step(
            "review",
            open == 0,
            false,
            Some(Msg::new("fechamento-review-falta").arg("n", open)),
            "/painel?resolver=1",
        ),
        step(
            "summary",
            st.summary,
            true,
            Some(Msg::new("fechamento-summary-falta")),
            &format!("/relatorios?mes={month}"),
        ),
    ];
    Ok(MonthClose {
        applicable: total > 0,
        done: steps.iter().filter(|s| s.done).count() as i64,
        steps,
        closed_at: st.closed_at,
        month,
    })
}

/// Marca um passo manual, fecha o mês ou reabre.
pub fn set(conn: &Connection, a: &MonthCloseSetArgs, today: NaiveDate) -> Result<()> {
    let month = resolve_month(Some(&a.month), today)?;
    let mut all = stored(conn)?;
    let st = all.entry(month).or_default();
    match a.step.as_str() {
        "balances" => st.balances = a.done,
        "summary" => st.summary = a.done,
        "closed" => st.closed_at = a.done.then(now_utc),
        _ => return Err(Error::invalid(i18n::t("fechamento-erro-passo"))),
    }
    // Só os meses mais recentes ficam guardados.
    while all.len() > KEEP_MONTHS {
        let Some(oldest) = all.keys().next().cloned() else {
            break;
        };
        all.remove(&oldest);
    }
    crate::settings::set(conn, SETTING, &all)
}
