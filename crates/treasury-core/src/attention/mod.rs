//! "Para resolver": uma lista só com o que precisa da pessoa (docs/to-resolve.md), igual
//! para quem tem e para quem não tem motor de IA.
//!
//! O app já percebia tudo isto, em três lugares com modelos diferentes: os alertas da caixa do
//! Agente (`ai::alerts`, alimentados pelos lembretes de `notify`), as percepções do Painel
//! (`insights`) e as sugestões do painel lateral (`ai::suggestions`). Este módulo só **lê e
//! compõe**: nenhuma regra nova de dinheiro mora aqui.
//!
//! De onde vem cada item:
//!
//! - **vencimentos** (`due`): os previstos e as faturas a pagar, direto de
//!   `reports::upcoming`, na antecedência das notificações, mais os previstos vencidos há até
//!   30 dias. É o estado de agora: uma conta paga sai da lista sem ninguém resolver alerta;
//! - **dados** (`data`): as contas do dia a dia sem extrato recente (`accounts::freshness`), os
//!   arquivos esperando na caixa de entrada e as tabelas anuais embutidas que passaram da
//!   validade (`datasets`);
//! - **tarefas** (`todo`): as sugestões determinísticas (sem categoria, orçamento estourado,
//!   ativo sem cotação, propostas a aprovar…) e as regras de categoria prontas para criar
//!   (`rules::learned`);
//! - **alertas** (`alert`): os alertas novos ou vistos dos últimos 30 dias que não têm uma
//!   versão "de agora" acima (`LIVE_ALERT_KINDS`);
//! - **percepções** (`insight`): só as de atenção. As informativas, as notícias boas e a
//!   inflação pessoal continuam no cartão "O que o Treasury percebeu".
//!
//! Dispensar: um alerta vira resolvido; o resto fica adiado no ajuste `attention.dismissed`
//! (dentro do cofre, vale em qualquer computador) e volta sozinho se continuar valendo.

pub mod api;
pub mod closing;
pub mod ladder;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_steps;

use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::AccountView;
use crate::ai::alerts::{
    self, Alert, AlertListArgs, AlertScreen, AlertSeverity, AlertStatus, AlertStatusArgs,
    AlertTarget, Arg, Msg,
};
use crate::ai::suggestions::{self, Suggestion, SuggestionArgs, SuggestionKind};
use crate::error::{Error, Result};
use crate::i18n::{self, args, t_args};
use crate::ids::Id;
use crate::insights::{Insight, InsightKind, InsightTone};
use crate::reports::{UpcomingItem, UpcomingKind};
use crate::time::{fmt_date, parse_date};

/// Ajuste com os itens adiados: id → dia até o qual ficam fora da lista.
pub const DISMISSED_SETTING: &str = "attention.dismissed";
/// Previstos vencidos entram na lista por este tempo.
pub const OVERDUE_DAYS: i64 = 30;
/// Alertas mais antigos que isto ficam só em Agente › Alertas.
pub const ALERT_DAYS: i64 = 30;
/// Tipos de alerta que a lista calcula na hora (vencimentos, orçamento e contas sem extrato):
/// o alerta gravado fica de fora para o mesmo fato não aparecer duas vezes, nem continuar
/// aparecendo depois de resolvido na prática.
pub const LIVE_ALERT_KINDS: &[&str] = &["due_soon", "budget_over", "stale_accounts"];
/// Chave do aviso de tabelas embutidas vencidas (`datasets`).
pub const OUTDATED_TABLES_ID: &str = "data:outdated_tables";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AttentionSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AttentionSource {
    Due,
    Alert,
    Data,
    Todo,
    Insight,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttentionItem {
    /// Chave estável: `due:<ref>:<data>`, `alert:<id>`, `data:<o quê>`, `todo:<tipo>` ou
    /// `insight:<id>`.
    pub id: String,
    pub source: AttentionSource,
    /// `due_soon`, `overdue`, `stale_accounts`, `inbox`, `uncategorized`, `budget_over`… ou o
    /// tipo do alerta ou da percepção.
    pub kind: String,
    pub severity: AttentionSeverity,
    pub title: String,
    /// Os números e o porquê.
    pub body: String,
    /// O que fazer, em uma frase, quando houver.
    pub action: Option<String>,
    /// Onde resolver (rota do app), quando não for um destino de alerta.
    pub route: Option<String>,
    /// Onde resolver, no formato dos alertas (conta, fatura, ativo…).
    pub target: Option<AlertTarget>,
    /// O alerta gravado, quando o item é um alerta (dispensar = resolver o alerta).
    pub alert_id: Option<Id>,
    /// Quantas coisas o item representa (12 lançamentos sem categoria = 12).
    #[ts(type = "number")]
    pub count: i64,
    /// Dia a que o item se refere (vencimento, criação do alerta).
    pub date: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttentionCounts {
    #[ts(type = "number")]
    pub total: i64,
    /// Críticos e de atenção.
    #[ts(type = "number")]
    pub urgent: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CoverageStatus {
    /// Nenhuma conta do dia a dia cadastrada: nada a dizer.
    Empty,
    /// Todas as contas do dia a dia estão em dia.
    Complete,
    /// Pelo menos uma está sem extrato recente: os totais podem estar incompletos.
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CoverageAccount {
    pub account_id: Id,
    pub name: String,
    /// Último lançamento efetivado.
    pub last_txn_date: Option<String>,
    #[ts(type = "number | null")]
    pub staleness_days: Option<i64>,
}

/// Até quando os números valem (docs/account-freshness.md): o Painel, a previsão de saldo e o
/// "quanto posso gastar hoje" mostram isto ao lado dos totais.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataCoverage {
    pub status: CoverageStatus,
    /// Contas do dia a dia consideradas.
    #[ts(type = "number")]
    pub accounts: i64,
    /// As que estão com o saldo possivelmente incompleto.
    pub stale: Vec<CoverageAccount>,
    /// O dia até o qual todas as contas do dia a dia têm novidade (lançamento, importação ou
    /// conferência de saldo): a novidade mais antiga entre elas. `None` sem contas ou quando
    /// alguma nunca teve novidade.
    pub complete_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Attention {
    /// Do mais grave para o menos grave.
    pub items: Vec<AttentionItem>,
    pub counts: AttentionCounts,
    pub coverage: DataCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttentionDismissArgs {
    pub id: String,
}

// ---------------------------------------------------------------------------
// Adiados

fn dismissed(conn: &Connection) -> Result<BTreeMap<String, String>> {
    crate::settings::get_or(conn, DISMISSED_SETTING, BTreeMap::new())
}

/// Por quantos dias um item dispensado fica fora da lista. Vencimentos e percepções trazem a
/// data ou o mês na chave e não voltam; dados e tarefas voltam em uma semana se continuarem
/// valendo.
fn snooze_days(id: &str) -> i64 {
    // Tabela vencida só se resolve atualizando o app: não insiste toda semana.
    // O lembrete das compras sem itens traz o mês na chave: dispensado, só volta no outro mês.
    if id.starts_with("due:")
        || id.starts_with("insight:")
        || id.starts_with("todo:purchase_items:")
        || id == OUTDATED_TABLES_ID
    {
        90
    } else {
        7
    }
}

/// Dispensa um item: o alerta é resolvido; os demais ficam adiados no cofre.
pub fn dismiss(conn: &Connection, id: &str, today: NaiveDate) -> Result<()> {
    if let Some(alert_id) = id.strip_prefix("alert:") {
        alerts::set_status(
            conn,
            &AlertStatusArgs {
                ids: vec![alert_id.to_string()],
                status: AlertStatus::Resolved,
            },
        )?;
        return Ok(());
    }
    let known = ["due:", "data:", "todo:", "insight:"];
    if !known.iter().any(|p| id.starts_with(p)) || id.len() > 200 {
        return Err(Error::invalid(i18n::t("atencao-erro-item")));
    }
    let today_s = fmt_date(today);
    let mut map = dismissed(conn)?;
    // Faxina: o que já venceu sai do ajuste.
    map.retain(|_, until| until.as_str() >= today_s.as_str());
    map.insert(
        id.to_string(),
        fmt_date(today + Duration::days(snooze_days(id))),
    );
    crate::settings::set(conn, DISMISSED_SETTING, &map)
}

// ---------------------------------------------------------------------------
// Cobertura

/// A novidade mais recente da conta: lançamento, importação ou conferência de saldo.
fn last_news(a: &AccountView) -> Option<String> {
    match (&a.last_txn_date, &a.last_import_at) {
        (Some(t), Some(i)) => Some(t.max(i).clone()),
        (Some(d), None) | (None, Some(d)) => Some(d.clone()),
        (None, None) => None,
    }
}

fn coverage_of(everyday: &[AccountView]) -> DataCoverage {
    use crate::accounts::freshness::BalanceStatus;
    let stale: Vec<CoverageAccount> = everyday
        .iter()
        .filter(|a| a.balance_status == Some(BalanceStatus::PossiblyIncomplete))
        .map(|a| CoverageAccount {
            account_id: a.account.id.clone(),
            name: a.account.name.clone(),
            last_txn_date: a.last_txn_date.clone(),
            staleness_days: a.staleness_days,
        })
        .collect();
    let news: Vec<Option<String>> = everyday.iter().map(last_news).collect();
    let complete_until = if news.is_empty() || news.iter().any(Option::is_none) {
        None
    } else {
        news.into_iter().flatten().min()
    };
    DataCoverage {
        status: if everyday.is_empty() {
            CoverageStatus::Empty
        } else if stale.is_empty() {
            CoverageStatus::Complete
        } else {
            CoverageStatus::Partial
        },
        accounts: everyday.len() as i64,
        stale,
        complete_until,
    }
}

/// Até quando os números valem, pelas contas do dia a dia.
pub fn coverage(conn: &Connection, today: NaiveDate) -> Result<DataCoverage> {
    Ok(coverage_of(&crate::accounts::freshness::everyday(
        conn, today,
    )?))
}

// ---------------------------------------------------------------------------
// Itens

fn target(screen: AlertScreen) -> AlertTarget {
    AlertTarget {
        screen,
        id: None,
        account_id: None,
        month: None,
        date: None,
    }
}

/// Vencimentos: o que vence dentro da antecedência das notificações e os previstos vencidos.
fn due_items(conn: &Connection, today: NaiveDate) -> Result<Vec<AttentionItem>> {
    let l = i18n::current();
    let ahead = crate::notify::prefs(conn)?.bills_days.max(1);
    let from = today - Duration::days(OVERDUE_DAYS);
    let items = crate::reports::upcoming(conn, from, OVERDUE_DAYS as u32 + ahead)?;
    Ok(items
        .into_iter()
        .filter(|u| u.amount < 0)
        .filter_map(|u: UpcomingItem| {
            let date = parse_date(&u.date).ok()?;
            let days = (date - today).num_days();
            let card = u.kind == UpcomingKind::CardDue;
            // Fatura vencida e não paga é assunto do alerta de rotativo, com os encargos.
            if card && days < 0 {
                return None;
            }
            let title = if days < 0 {
                Msg::new("atencao-venceu-titulo")
                    .arg("descricao", u.description.clone())
                    .arg("data", Arg::day_month(&u.date))
            } else {
                let when = Msg::new("alertas-quando").arg("dias", days);
                if card {
                    Msg::new("alertas-vence-fatura").arg("conta", u.account_name.clone())
                } else {
                    Msg::new("alertas-vence-conta").arg("descricao", u.description.clone())
                }
                .arg("quando", when)
            };
            let body = Msg::new(if days < 0 {
                "atencao-venceu-corpo"
            } else {
                "alertas-vence-corpo"
            })
            .arg("valor", Arg::money(-u.amount, &u.currency))
            .arg("conta", u.account_name.clone())
            .arg("data", Arg::day_month(&u.date));
            Some(AttentionItem {
                id: format!("due:{}:{}", u.ref_id, u.date),
                source: AttentionSource::Due,
                kind: if days < 0 { "overdue" } else { "due_soon" }.into(),
                severity: if days <= 1 {
                    AttentionSeverity::Warning
                } else {
                    AttentionSeverity::Info
                },
                title: title.render(l),
                body: body.render(l),
                action: None,
                route: None,
                target: Some(if card {
                    AlertTarget {
                        id: Some(u.ref_id.clone()),
                        account_id: Some(u.account_id.clone()),
                        ..target(AlertScreen::CardStatement)
                    }
                } else {
                    AlertTarget {
                        id: Some(u.ref_id.clone()),
                        account_id: Some(u.account_id.clone()),
                        date: Some(u.date.clone()),
                        ..target(AlertScreen::Transactions)
                    }
                }),
                alert_id: None,
                count: 1,
                date: Some(u.date),
            })
        })
        .collect())
}

/// Contas sem extrato recente e arquivos esperando na caixa de entrada.
fn data_items(
    conn: &Connection,
    stale: &[AccountView],
    today: NaiveDate,
) -> Result<Vec<AttentionItem>> {
    let l = i18n::current();
    let mut out = vec![];
    if !stale.is_empty() {
        let (title, body) = crate::notify::stale_accounts_text(stale);
        out.push(AttentionItem {
            id: "data:stale_accounts".into(),
            source: AttentionSource::Data,
            kind: "stale_accounts".into(),
            severity: AttentionSeverity::Warning,
            title: title.render(l),
            body: alerts::render_parts(&body, l),
            action: None,
            route: None,
            target: Some(AlertTarget {
                account_id: stale.first().map(|a| a.account.id.clone()),
                ..target(AlertScreen::Import)
            }),
            alert_id: None,
            count: stale.len() as i64,
            date: None,
        });
    }
    // Tabelas anuais embutidas que passaram da validade (INSS e equivalentes do país).
    let tables = crate::datasets::outdated(conn, today);
    if !tables.is_empty() {
        let names = Arg::join(
            tables
                .iter()
                .map(|t| {
                    Msg::new("atencao-tabelas-item")
                        .arg("nome", t.label.clone())
                        .arg("ano", t.year.to_string())
                        .into()
                })
                .collect(),
            "; ",
        );
        out.push(AttentionItem {
            id: OUTDATED_TABLES_ID.into(),
            source: AttentionSource::Data,
            kind: "outdated_tables".into(),
            severity: AttentionSeverity::Info,
            title: Msg::new("atencao-tabelas-titulo")
                .arg("n", tables.len() as i64)
                .render(l),
            body: Msg::new("atencao-tabelas-corpo")
                .arg("tabelas", names)
                .render(l),
            action: Some(i18n::t("atencao-tabelas-acao")),
            route: Some("/ajustes?secao=sobre".into()),
            target: None,
            alert_id: None,
            count: tables.len() as i64,
            date: None,
        });
    }
    let waiting = crate::import::inbox::list(conn, &crate::import::inbox::InboxListArgs::default())?
        .into_iter()
        .filter(|i| i.status == crate::import::inbox::InboxStatus::Pending)
        .count() as i64;
    if waiting > 0 {
        out.push(AttentionItem {
            id: "data:inbox".into(),
            source: AttentionSource::Data,
            kind: "inbox".into(),
            severity: AttentionSeverity::Info,
            title: t_args("atencao-caixa-titulo", &args(&[("n", waiting.into())])),
            body: i18n::t("atencao-caixa-corpo"),
            action: None,
            route: Some("/lancamentos?aba=caixa".into()),
            target: None,
            alert_id: None,
            count: waiting,
            date: None,
        });
    }
    Ok(out)
}

fn todo_kind(k: SuggestionKind) -> Option<&'static str> {
    use SuggestionKind::*;
    match k {
        Onboarding => Some("onboarding"),
        Uncategorized => Some("uncategorized"),
        BudgetOver => Some("budget_over"),
        AssetNoPrice => Some("asset_no_price"),
        B3Import => Some("b3_import"),
        ReserveLiquidity => Some("reserve_liquidity"),
        PendingProposals => Some("pending_proposals"),
        // Contas sem extrato e faturas vêm das regras de agora (`data_items`, `due_items`);
        // "há alertas novos" não faz sentido dentro da própria lista.
        StaleImport | CardDue | NewAlerts => None,
    }
}

fn todo_items(conn: &Connection, today: NaiveDate) -> Result<Vec<AttentionItem>> {
    let all = suggestions::compute(
        conn,
        &SuggestionArgs {
            screen: None,
            limit: Some(20),
        },
        today,
    )?;
    let mut out: Vec<AttentionItem> = all
        .into_iter()
        .filter_map(|s: Suggestion| {
            let kind = todo_kind(s.kind)?;
            Some(AttentionItem {
                id: format!("todo:{kind}"),
                source: AttentionSource::Todo,
                kind: kind.into(),
                severity: match s.severity {
                    suggestions::SuggestionSeverity::Warning => AttentionSeverity::Warning,
                    suggestions::SuggestionSeverity::Info => AttentionSeverity::Info,
                },
                title: s.title,
                body: s.detail,
                action: None,
                route: s.route,
                target: None,
                alert_id: None,
                count: s.count.max(1),
                date: None,
            })
        })
        .collect();
    // Regras de categoria que o app pode oferecer (a mesma categoria três vezes).
    let learned = crate::rules::learned::learned(conn, today)?;
    if !learned.is_empty() {
        let names: Vec<String> = learned
            .iter()
            .take(3)
            .map(|r| r.key.to_uppercase())
            .collect();
        out.push(AttentionItem {
            id: "todo:learned_rules".into(),
            source: AttentionSource::Todo,
            kind: "learned_rules".into(),
            severity: AttentionSeverity::Info,
            title: t_args(
                "atencao-regras-titulo",
                &args(&[("n", (learned.len() as i64).into())]),
            ),
            body: t_args(
                "atencao-regras-corpo",
                &args(&[("nomes", names.join(", ").into())]),
            ),
            action: None,
            route: Some("/lancamentos?aba=regras".into()),
            target: None,
            alert_id: None,
            count: learned.len() as i64,
            date: None,
        });
    }
    // Compras de mercado do mês sem itens, só para quem já usa Consumo (docs/consumption.md).
    let first = chrono::Datelike::with_day(&today, 1).unwrap_or(today);
    let open = crate::food::market_without_items(conn, first, today)?;
    if open.len() >= 2 {
        out.push(AttentionItem {
            id: format!("todo:purchase_items:{}", &fmt_date(today)[..7]),
            source: AttentionSource::Todo,
            kind: "purchase_items".into(),
            severity: AttentionSeverity::Info,
            title: t_args(
                "atencao-compras-titulo",
                &args(&[("n", (open.len() as i64).into())]),
            ),
            body: crate::i18n::t("atencao-compras-corpo"),
            action: None,
            route: Some(format!("/lancamentos?detalhar={}", open[0])),
            target: None,
            alert_id: None,
            count: open.len() as i64,
            date: None,
        });
    }
    Ok(out)
}

fn alert_items(conn: &Connection, today: NaiveDate) -> Result<Vec<AttentionItem>> {
    let since = fmt_date(today - Duration::days(ALERT_DAYS));
    let open = alerts::list(
        conn,
        &AlertListArgs {
            status: None,
            kind: None,
            limit: Some(200),
        },
    )?;
    Ok(open
        .into_iter()
        .filter(|a: &Alert| !LIVE_ALERT_KINDS.contains(&a.kind.as_str()))
        .filter(|a| a.updated_at.get(..10).is_some_and(|d| d >= since.as_str()))
        .map(|a| AttentionItem {
            id: format!("alert:{}", a.id),
            source: AttentionSource::Alert,
            kind: a.kind,
            severity: match a.severity {
                AlertSeverity::Critical => AttentionSeverity::Critical,
                AlertSeverity::Warning => AttentionSeverity::Warning,
                AlertSeverity::Info => AttentionSeverity::Info,
            },
            title: a.title,
            body: a.body,
            action: None,
            route: None,
            target: a.target,
            alert_id: Some(a.id),
            count: 1,
            date: a.created_at.get(..10).map(str::to_string),
        })
        .collect())
}

fn insight_items(conn: &Connection, today: NaiveDate) -> Result<Vec<AttentionItem>> {
    Ok(crate::insights::compute(conn, today)?
        .items
        .into_iter()
        // A inflação pessoal é informação, não tarefa: fica no cartão de percepções, com a
        // tabela dos grupos.
        .filter(|i: &Insight| {
            i.tone == InsightTone::Warning && i.kind != InsightKind::PersonalInflation
        })
        .map(|i| AttentionItem {
            id: format!("insight:{}", i.id),
            source: AttentionSource::Insight,
            kind: serde_json::to_value(i.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default(),
            severity: AttentionSeverity::Warning,
            title: i.title,
            body: i.body,
            action: i.action,
            route: i.screen.map(|s| format!("/{s}")),
            target: None,
            alert_id: None,
            count: 1,
            date: None,
        })
        .collect())
}

/// A lista na data `today`: o que não foi dispensado, do mais grave para o menos grave e, na
/// mesma gravidade, vencimentos, alertas, dados, tarefas e percepções.
pub fn list(conn: &Connection, today: NaiveDate) -> Result<Attention> {
    let everyday = crate::accounts::freshness::everyday(conn, today)?;
    let coverage = coverage_of(&everyday);
    let stale: Vec<AccountView> = everyday
        .into_iter()
        .filter(|a| coverage.stale.iter().any(|s| s.account_id == a.account.id))
        .collect();

    let mut items = due_items(conn, today)?;
    items.extend(alert_items(conn, today)?);
    items.extend(data_items(conn, &stale, today)?);
    items.extend(todo_items(conn, today)?);
    items.extend(insight_items(conn, today)?);

    let today_s = fmt_date(today);
    let off = dismissed(conn)?;
    items.retain(|i| {
        off.get(&i.id)
            .is_none_or(|until| until.as_str() < today_s.as_str())
    });
    // Estável: dentro da gravidade e da origem, fica a ordem de cada fonte.
    items.sort_by_key(|i| (i.severity, i.source));

    let counts = AttentionCounts {
        total: items.len() as i64,
        urgent: items
            .iter()
            .filter(|i| i.severity != AttentionSeverity::Info)
            .count() as i64,
    };
    Ok(Attention {
        items,
        counts,
        coverage,
    })
}
