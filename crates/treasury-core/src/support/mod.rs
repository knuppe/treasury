//! Apoio ao Treasury (docs/support.md): o canal de doação (GitHub Sponsors, embutido no build
//! em `support.json`) e as regras do lembrete na Visão geral.
//!
//! O estado do lembrete fica em `support-state.json`, na pasta do app e fora do cofre: datas e
//! contadores por instalação, nenhum dado financeiro. A decisão é uma função pura
//! ([`decide`]); os dados do cofre só podem silenciar o lembrete (alerta grave), nunca pedir
//! mais. Nada aqui faz rede, telemetria ou analytics: os links só abrem no navegador quando o
//! usuário clica.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};

const CHANNELS_JSON: &str = include_str!("support.json");
const STATE_FILE: &str = "support-state.json";

/// Primeira vez: dias desde a instalação e dias distintos de uso.
pub const FIRST_AFTER_DAYS: i64 = 30;
pub const FIRST_AFTER_ACTIVE: u32 = 10;
/// Entre duas aparições: dias corridos e dias de uso.
pub const INTERVAL_DAYS: i64 = 30;
pub const INTERVAL_ACTIVE: u32 = 8;
/// Três "Agora não" seguidos: o intervalo vira 90 dias.
pub const TIRED_STREAK: u32 = 3;
pub const TIRED_INTERVAL_DAYS: i64 = 90;
pub const MAX_PER_YEAR: u32 = 6;
/// "Já apoio": 12 meses em silêncio.
pub const SUPPORTER_SNOOZE_DAYS: i64 = 365;
/// O cartão espera a visita à Visão geral por até 3 dias depois do evento.
pub const PENDING_DAYS: i64 = 3;

/// Os canais, num lugar só. Um valor `<<PLACEHOLDER…>>` aparece como "em breve".
#[derive(Debug, Clone, Deserialize)]
struct RawChannels {
    github_sponsors_url: String,
    repository_url: String,
    signature: String,
    annual_summary: Option<AnnualSummary>,
}

/// Resumo anual de para onde foi o dinheiro (opcional, com a data).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualSummary {
    /// `YYYY-MM-DD`.
    pub date: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct SupportChannels {
    pub github_sponsors_url: Option<String>,
    pub repository_url: Option<String>,
    pub signature: String,
    pub annual_summary: Option<AnnualSummary>,
}

fn placeholder(s: &str) -> bool {
    let t = s.trim();
    t.is_empty() || t.starts_with("<<")
}

/// Só links https (nunca outro esquema).
fn url(s: &str) -> Option<String> {
    (!placeholder(s) && s.trim().starts_with("https://")).then(|| s.trim().to_string())
}

pub fn channels() -> Result<SupportChannels> {
    let raw: RawChannels = serde_json::from_str(CHANNELS_JSON)
        .map_err(|e| Error::Other(format!("support.json: {e}")))?;
    Ok(SupportChannels {
        github_sponsors_url: url(&raw.github_sponsors_url),
        repository_url: url(&raw.repository_url),
        signature: raw.signature,
        annual_summary: raw.annual_summary,
    })
}

// ---------------------------------------------------------------------------
// Estado do lembrete

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SupportState {
    pub installed_at: Option<NaiveDate>,
    /// Dias distintos em que o app foi usado (cofre aberto).
    pub active_days: u32,
    pub last_active_day: Option<NaiveDate>,
    /// `active_days` quando o cartão apareceu da última vez.
    pub active_days_at_last: u32,
    pub last_shown_at: Option<NaiveDate>,
    /// "Agora não" seguidos.
    pub dismiss_streak: u32,
    pub shown_this_year: u32,
    pub shown_year: Option<i32>,
    pub snooze_until: Option<NaiveDate>,
    /// "Não mostrar mais" (religa em Ajustes › Sobre).
    pub opted_out: bool,
    /// "Já apoio": depois do silêncio, volta como agradecimento.
    pub supporter: bool,
    /// Um evento de valor liberou o cartão até esta data.
    pub pending_until: Option<NaiveDate>,
    /// O cartão pendente já contou como mostrado.
    pub pending_counted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SupportVariant {
    /// O pedido.
    Ask,
    /// "Obrigado por apoiar" (quem disse "Já apoio", depois dos 12 meses).
    Thanks,
}

/// O que silencia o lembrete além do estado.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Quiet {
    /// Alerta grave: saldo negativo, conta ou dívida em atraso, renda que caiu muito.
    pub severe: bool,
    /// "Ver como" outro membro.
    pub view_as: bool,
    /// Cofre de demonstração.
    pub demo: bool,
}

/// Pode mostrar o lembrete agora? Função pura: o estado, o dia e o que silencia.
pub fn decide(s: &SupportState, today: NaiveDate, q: Quiet) -> Option<SupportVariant> {
    if s.opted_out || q.severe || q.view_as || q.demo {
        return None;
    }
    if s.snooze_until.is_some_and(|d| today < d) {
        return None;
    }
    if s.shown_year == Some(today.year()) && s.shown_this_year >= MAX_PER_YEAR {
        return None;
    }
    let installed = s.installed_at?;
    match s.last_shown_at {
        None => {
            if today - installed < Duration::days(FIRST_AFTER_DAYS)
                || s.active_days < FIRST_AFTER_ACTIVE
            {
                return None;
            }
        }
        Some(last) => {
            let gap = if s.dismiss_streak >= TIRED_STREAK {
                TIRED_INTERVAL_DAYS
            } else {
                INTERVAL_DAYS
            };
            if today - last < Duration::days(gap)
                || s.active_days.saturating_sub(s.active_days_at_last) < INTERVAL_ACTIVE
            {
                return None;
            }
        }
    }
    Some(if s.supporter {
        SupportVariant::Thanks
    } else {
        SupportVariant::Ask
    })
}

/// Marca o dia de uso (e a instalação, na primeira vez).
pub fn touch(s: &mut SupportState, today: NaiveDate) -> bool {
    let mut changed = false;
    if s.installed_at.is_none() {
        s.installed_at = Some(today);
        changed = true;
    }
    if s.last_active_day != Some(today) {
        s.last_active_day = Some(today);
        s.active_days += 1;
        changed = true;
    }
    changed
}

/// Conta a aparição (uma vez por cartão).
pub fn mark_shown(s: &mut SupportState, today: NaiveDate) {
    if s.pending_counted {
        return;
    }
    s.pending_counted = true;
    s.last_shown_at = Some(today);
    s.active_days_at_last = s.active_days;
    if s.shown_year != Some(today.year()) {
        s.shown_year = Some(today.year());
        s.shown_this_year = 0;
    }
    s.shown_this_year += 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SupportChoice {
    /// "Como apoiar": abre a página.
    How,
    /// "Já apoio": 12 meses em silêncio, sem verificação.
    Supporter,
    /// "Agora não": conta para o cansaço.
    Later,
    /// "Não mostrar mais".
    Never,
}

pub fn respond(s: &mut SupportState, today: NaiveDate, c: SupportChoice) {
    mark_shown(s, today);
    s.pending_until = None;
    s.pending_counted = false;
    match c {
        SupportChoice::How => s.dismiss_streak = 0,
        SupportChoice::Supporter => {
            s.supporter = true;
            s.dismiss_streak = 0;
            s.snooze_until = Some(today + Duration::days(SUPPORTER_SNOOZE_DAYS));
        }
        SupportChoice::Later => s.dismiss_streak += 1,
        SupportChoice::Never => s.opted_out = true,
    }
}

// ---------------------------------------------------------------------------
// Arquivo de estado (por instalação, fora do cofre)

static LOCK: Mutex<()> = Mutex::new(());

fn state_path(root: &Path) -> PathBuf {
    root.join(STATE_FILE)
}

pub fn load(root: &Path) -> SupportState {
    std::fs::read_to_string(state_path(root))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(root: &Path, s: &SupportState) -> Result<()> {
    let p = state_path(root);
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(s)?)?;
    crate::paths::set_mode(&tmp, 0o600)?;
    std::fs::rename(&tmp, &p)?;
    Ok(())
}

/// Carrega, muda e grava o estado (uma escrita por vez).
pub fn update<T>(root: &Path, f: impl FnOnce(&mut SupportState) -> T) -> Result<T> {
    let _g = LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut s = load(root);
    let before = s.clone();
    let out = f(&mut s);
    if s != before {
        save(root, &s)?;
    }
    Ok(out)
}

/// Dia de uso (chamado pelo ciclo do app com o cofre aberto; grava no máximo uma vez por dia).
pub fn touch_today(root: &Path, today: NaiveDate) {
    static LAST: Mutex<Option<NaiveDate>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|p| p.into_inner());
    if *last == Some(today) {
        return;
    }
    if update(root, |s| touch(s, today)).is_ok() {
        *last = Some(today);
    }
}

// ---------------------------------------------------------------------------
// O que silencia, pelo cofre

/// Alerta grave: saldo negativo avisado há pouco, fatura ou parcela vencida e não paga, ou o
/// último mês com a renda bem abaixo da média dos três anteriores.
pub fn severe(conn: &Connection, today: NaiveDate) -> bool {
    let t = crate::time::fmt_date(today);
    let one = |sql: &str| -> bool {
        conn.query_row(sql, [&t], |r| r.get::<_, i64>(0))
            .map(|n| n > 0)
            .unwrap_or(false)
    };
    if one(
        "SELECT COUNT(*) FROM alert WHERE kind = 'negative_balance' AND status <> 'resolved'
            AND created_at >= date(?1, '-30 days')",
    ) {
        return true;
    }
    if one(
        "SELECT COUNT(*) FROM card_statement WHERE status IN ('closed', 'partial')
            AND due_date < date(?1, '-2 days') AND due_date >= date(?1, '-60 days')",
    ) {
        return true;
    }
    if one(
        "SELECT COUNT(*) FROM loan_installment i JOIN loan l ON l.id = i.loan_id
          WHERE l.archived = 0 AND i.status = 'planned'
            AND i.due_date < date(?1, '-5 days') AND i.due_date >= date(?1, '-60 days')",
    ) {
        return true;
    }
    // Renda do último mês fechado contra a média dos três antes dele.
    let income =
        |from_months: i64, to_months: i64| -> i64 {
            conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM txn WHERE kind = 'income' AND status <> 'planned'
              AND date >= date(?1, 'start of month', ?2) AND date < date(?1, 'start of month', ?3)",
            rusqlite::params![t, format!("-{from_months} months"), format!("-{to_months} months")],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        };
    let last = income(1, 0);
    let avg = income(4, 1) / 3;
    avg > 0 && last < avg * 6 / 10
}

// ---------------------------------------------------------------------------
// Comandos

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct SupportInfo {
    pub channels: SupportChannels,
    pub opted_out: bool,
    pub supporter: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct SupportPrompt {
    pub variant: Option<SupportVariant>,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct SupportEventArgs {
    /// `budget_month` (mês fechado no Orçamento), `import` (importação terminada) ou
    /// `tax_report` (relatório anual do IR).
    pub event: String,
    /// "Ver como" outro membro.
    #[serde(default)]
    #[ts(optional)]
    pub view_as: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct SupportRespondArgs {
    pub choice: SupportChoice,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct SupportOptArgs {
    pub opted_out: bool,
}

pub const READ_ONLY: &[&str] = &["info", "prompt"];

const EVENTS: &[&str] = &["budget_month", "import", "tax_report"];

fn quiet(ctx: &crate::api::Ctx<'_>, view_as: bool, today: NaiveDate) -> Quiet {
    Quiet {
        severe: severe(ctx.conn(), today),
        view_as,
        demo: crate::settings::get_or(ctx.conn(), crate::demo::SETTING, false).unwrap_or(false),
    }
}

pub fn handle(
    ctx: &mut crate::api::Ctx<'_>,
    action: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    use crate::api::{Actor, parse_args, to_value};
    // O lembrete é da interface: o agente e os clientes MCP não mexem nele.
    if *ctx.actor != Actor::User && !READ_ONLY.contains(&action) {
        return Err(Error::Forbidden(crate::i18n::t(
            "nucleo-geral-erro-apoio-so-interface",
        )));
    }
    let root = ctx.paths.root.clone();
    let today = crate::time::today_local();
    match action {
        "info" => {
            let s = load(&root);
            to_value(SupportInfo {
                channels: channels()?,
                opted_out: s.opted_out,
                supporter: s.supporter,
            })
        }
        // Um evento de valor: se as regras deixam, o cartão fica esperando a Visão geral.
        "event" => {
            let a: SupportEventArgs = parse_args(args)?;
            if !EVENTS.contains(&a.event.as_str()) {
                return Err(Error::invalid(crate::i18n::t_args(
                    "nucleo-geral-erro-evento",
                    &crate::i18n::args(&[("evento", a.event.clone().into())]),
                )));
            }
            let q = quiet(ctx, a.view_as.unwrap_or(false), today);
            let v = update(&root, |s| {
                touch(s, today);
                if s.pending_until.is_some_and(|d| d >= today) {
                    return true;
                }
                let ok = decide(s, today, q).is_some();
                if ok {
                    s.pending_until = Some(today + Duration::days(PENDING_DAYS));
                    s.pending_counted = false;
                }
                ok
            })?;
            Ok(serde_json::json!({ "pending": v }))
        }
        // A Visão geral pergunta se há cartão (e confere de novo o que silencia).
        "prompt" => {
            #[derive(Deserialize, Default)]
            struct A {
                #[serde(default)]
                view_as: Option<bool>,
            }
            let a: A = parse_args(args)?;
            let s = load(&root);
            let pending = s.pending_until.is_some_and(|d| d >= today);
            let q = quiet(ctx, a.view_as.unwrap_or(false), today);
            let variant = if !pending || s.opted_out || q.severe || q.view_as || q.demo {
                None
            } else if s.supporter {
                Some(SupportVariant::Thanks)
            } else {
                Some(SupportVariant::Ask)
            };
            to_value(SupportPrompt { variant })
        }
        // O cartão apareceu na tela: conta uma vez.
        "seen" => {
            update(&root, |s| {
                if s.pending_until.is_some_and(|d| d >= today) {
                    mark_shown(s, today);
                }
            })?;
            Ok(serde_json::Value::Null)
        }
        "respond" => {
            let a: SupportRespondArgs = parse_args(args)?;
            update(&root, |s| respond(s, today, a.choice))?;
            Ok(serde_json::Value::Null)
        }
        // Ajustes › Sobre: religar (ou desligar) o lembrete.
        "set_opted_out" => {
            let a: SupportOptArgs = parse_args(args)?;
            update(&root, |s| {
                s.opted_out = a.opted_out;
                if !a.opted_out {
                    s.dismiss_streak = 0;
                }
            })?;
            Ok(serde_json::Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("support.{action}"))),
    }
}

#[cfg(test)]
mod tests;
