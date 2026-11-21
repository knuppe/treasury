//! Lembretes e notificações do sistema (docs/notifications.md).
//!
//! Os lembretes são determinísticos e rodam sozinhos (sem motor de IA), no máximo uma vez
//! por hora com o cofre aberto, e viram alertas na caixa do Agente (a mesma chave nunca
//! gera dois):
//!
//! - contas e faturas vencendo nos próximos dias;
//! - IPTU: a parcela vencendo e o carnê do ano ainda não registrado (só no Brasil);
//! - data-com de proventos dos ativos da carteira;
//! - preço da watchlist cruzando o alerta (acima ou abaixo);
//! - saldo de conta ficando negativo nos próximos 30 dias (previsão de saldo);
//! - assinatura com aumento de preço ou cobrança nova;
//! - orçamento estourado no mês;
//! - cartão: limite quase todo usado, anuidade renovando em 30 dias, cobrança de anuidade que
//!   não confere com a regra, fatura que entrou no rotativo, encargos acima do teto legal e
//!   cashback ou pontos que não vieram no prazo (docs/design/cards.md);
//! - pontos e milhas vencendo (90, 30 e 7 dias antes).
//!
//! Os alertas novos (destes lembretes, das rotinas do agente e do score) viram notificação
//! do sistema conforme as preferências em Ajustes › Notificações; por padrão sem valores
//! no texto (aparecem na tela bloqueada).
//!
//! Os textos vão pela chave (`locales/*/alertas.ftl`, V128) e saem no idioma da instalação na
//! hora de mostrar ou de notificar; a deduplicação e os episódios não dependem do texto.

pub mod api;
#[cfg(test)]
mod tests;

use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::ai::alerts::{self, Alert, AlertSeverity, Arg, Msg, NewAlert, Part};
use crate::error::Result;
use crate::ids::Id;
use crate::time::{fmt_date, month_of, now_utc, parse_date};

pub const PREFS_SETTING: &str = "notify.prefs";
const LAST_RUN: &str = "notify.reminders.last_run";
const EVERY_MIN: i64 = 60;
const SOURCE: &str = "lembretes";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct NotifyPrefs {
    /// Notificações do sistema (os lembretes vão para a caixa de alertas mesmo assim).
    pub system: bool,
    /// Valores em reais no texto da notificação.
    pub show_amounts: bool,
    pub bills: bool,
    /// Avisar contas e faturas com quantos dias de antecedência (0 a 15).
    pub bills_days: u32,
    pub dividends: bool,
    pub price_targets: bool,
    pub negative_balance: bool,
    pub subscriptions: bool,
    pub budget: bool,
    /// Score de saúde dos ativos (queda de faixa, red flag nova).
    pub scores: bool,
    /// Demais alertas do agente (fechamento do mês, rebalanceamento).
    pub agent: bool,
    /// Contas sem extrato recente (saldo possivelmente incompleto).
    pub stale_accounts: bool,
}

impl Default for NotifyPrefs {
    fn default() -> Self {
        Self {
            system: true,
            show_amounts: false,
            bills: true,
            bills_days: 3,
            dividends: true,
            price_targets: true,
            negative_balance: true,
            subscriptions: true,
            budget: true,
            scores: true,
            agent: true,
            stale_accounts: true,
        }
    }
}

/// Notificação pronta para o sistema operacional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SystemNotice {
    pub alert_id: Id,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReminderRun {
    pub created: Vec<Id>,
}

pub fn prefs(conn: &Connection) -> Result<NotifyPrefs> {
    crate::settings::get_or(conn, PREFS_SETTING, NotifyPrefs::default())
}

pub fn save_prefs(conn: &Connection, p: &NotifyPrefs) -> Result<NotifyPrefs> {
    let mut p = p.clone();
    p.bills_days = p.bills_days.min(15);
    crate::settings::set(conn, PREFS_SETTING, &p)?;
    Ok(p)
}

/// Preferência que controla um tipo de alerta.
fn allowed(p: &NotifyPrefs, kind: &str) -> bool {
    match kind {
        "due_soon" | "iptu_due" | "iptu_missing" => p.bills,
        "dividend_datacom" => p.dividends,
        "price_target" => p.price_targets,
        "negative_balance" => p.negative_balance,
        "subscription_increase" | "subscription_new" => p.subscriptions,
        "budget_over" | "budget_pace" | "education_review" | "education_finished" => p.budget,
        "asset_score" => p.scores,
        "stale_accounts" => p.stale_accounts,
        _ => p.agent,
    }
}

/// Alertas novos → notificações do sistema, conforme as preferências.
pub fn notices(conn: &Connection, new: &[Alert]) -> Result<Vec<SystemNotice>> {
    let p = prefs(conn)?;
    if !p.system {
        return Ok(vec![]);
    }
    Ok(new
        .iter()
        .filter(|a| allowed(&p, &a.kind))
        .map(|a| SystemNotice {
            alert_id: a.id.clone(),
            title: a.title.clone(),
            body: if p.show_amounts {
                a.body.clone()
            } else {
                crate::i18n::t("alertas-notificacao-detalhes")
            },
        })
        .collect())
}

/// Roda os lembretes se já passou uma hora desde a última vez.
pub fn run_due(conn: &Connection, today: NaiveDate) -> Result<Option<ReminderRun>> {
    let last: Option<String> = crate::settings::get(conn, LAST_RUN)?;
    let due = match last
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
    {
        Some(t) => (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_minutes() >= EVERY_MIN,
        None => true,
    };
    if !due {
        return Ok(None);
    }
    let r = run(conn, today)?;
    crate::settings::set(conn, LAST_RUN, &now_utc())?;
    Ok(Some(r))
}

/// Confere tudo agora e cria os alertas que faltam.
pub fn run(conn: &Connection, today: NaiveDate) -> Result<ReminderRun> {
    let p = prefs(conn)?;
    let mut new = vec![];
    // Tipos que viram um alerta por episódio (a condição dura dias): os que a conferência
    // rodou, para fechar os episódios que acabaram.
    let mut ran: Vec<&'static str> = vec![];
    // Cada conferência é independente: uma que falha não impede as outras.
    let checks: [(&str, Option<&'static str>, bool, Check); 11] = [
        ("vencimentos", None, p.bills, bills),
        ("IPTU", None, p.bills, iptu),
        ("data-com", None, p.dividends, datacom),
        (
            "watchlist",
            Some("price_target"),
            p.price_targets,
            price_targets,
        ),
        (
            "saldo negativo",
            Some("negative_balance"),
            p.negative_balance,
            negative_balance,
        ),
        ("assinaturas", None, p.subscriptions, subscriptions),
        ("orçamento", None, p.budget, budget),
        ("faculdade", None, p.budget, education),
        ("cartões", None, p.bills, cards),
        ("pontos", None, p.bills, loyalty),
        ("contas sem extrato", None, p.stale_accounts, stale_accounts),
    ];
    for (name, episodic, on, check) in checks {
        if !on {
            continue;
        }
        match check(conn, today, &p) {
            Ok(v) => {
                new.extend(v);
                ran.extend(episodic);
            }
            Err(e) => tracing::warn!("lembrete de {name}: {e}"),
        }
    }
    let mut created = vec![];
    for a in new.iter().filter(|a| !EPISODIC.contains(&a.kind)) {
        if let Some(id) = alerts::create(conn, &a.as_new())? {
            created.push(id);
        }
    }
    for kind in ran {
        created.extend(episodes(conn, kind, &new, today)?);
    }
    Ok(ReminderRun { created })
}

/// Condições que duram dias (conta negativa na previsão, preço fora da faixa da watchlist):
/// um alerta por episódio, atualizado no lugar a cada conferência.
const EPISODIC: &[&str] = &["negative_balance", "price_target"];

/// O alerta aberto do episódio de cada referência (conta, ativo).
fn open_episodes(conn: &Connection, kind: &str) -> Result<Vec<(Id, String, serde_json::Value)>> {
    let mut stmt = conn.prepare(
        "SELECT id, COALESCE(ref_id, ''), data FROM alert
          WHERE kind = ?1 AND source = ?2 AND json_extract(data, '$.episode.open') = 1
          ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![kind, SOURCE], |r| {
        Ok((
            r.get::<_, Id>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    let mut out = vec![];
    for r in rows {
        let (id, ref_id, data) = r?;
        out.push((id, ref_id, serde_json::from_str(&data).unwrap_or_default()));
    }
    Ok(out)
}

/// Os alertas antigos (um por dia, de antes dos episódios): fica o mais recente de cada
/// referência, como episódio aberto; os outros saem.
fn merge_legacy(conn: &Connection, kind: &str) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, COALESCE(ref_id, ''), data FROM alert
          WHERE kind = ?1 AND source = ?2 AND json_extract(data, '$.episode') IS NULL
          ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![kind, SOURCE], |r| {
            Ok((
                r.get::<_, Id>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut kept: Vec<String> = open_episodes(conn, kind)?
        .into_iter()
        .map(|e| e.1)
        .collect();
    let mut gone = vec![];
    for (id, ref_id, data) in rows {
        if kept.contains(&ref_id) {
            gone.push(id);
            continue;
        }
        let mut d: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
        d["episode"] = json!({"open": true, "worst": d.get("min").cloned()});
        conn.execute(
            "UPDATE alert SET data = ?2 WHERE id = ?1",
            params![id, serde_json::to_string(&d)?],
        )?;
        kept.push(ref_id);
    }
    alerts::delete(conn, &gone)?;
    Ok(())
}

/// Piorou o bastante para avisar de novo: o ponto mais baixo caiu mais de 20% (e R$ 100).
fn much_worse(new: i64, worst: i64) -> bool {
    new < worst - (worst.abs() / 5).max(10_000)
}

fn episodes(
    conn: &Connection,
    kind: &'static str,
    found: &[Owned],
    today: NaiveDate,
) -> Result<Vec<Id>> {
    merge_legacy(conn, kind)?;
    let open = open_episodes(conn, kind)?;
    let now = now_utc();
    let mut created = vec![];
    let active: Vec<&Owned> = found.iter().filter(|a| a.kind == kind).collect();
    for a in &active {
        let ref_id = a.ref_id.clone().unwrap_or_default();
        match open.iter().find(|e| e.1 == ref_id) {
            Some((id, _, old)) => {
                // Mesmo episódio: atualiza no lugar, sem avisar, salvo se piorou muito.
                let worst = old["episode"]["worst"].as_i64();
                let worse = match (a.worst, worst) {
                    (Some(n), Some(w)) => much_worse(n, w),
                    _ => false,
                };
                let mut d = a.data.clone();
                d["episode"] = json!({
                    "open": true,
                    "since": old["episode"]["since"].clone(),
                    "worst": match (a.worst, worst) {
                        (Some(n), Some(w)) => Some(n.min(w)),
                        (n, w) => n.or(w),
                    },
                });
                if worse {
                    d["notify_at"] = json!(now);
                } else if let Some(t) = old.get("notify_at") {
                    d["notify_at"] = t.clone();
                }
                alerts::update_content(conn, id, a.severity, Some(&a.title), &a.body, &d, worse)?;
            }
            None => {
                // Episódio novo (a primeira vez ou depois de voltar ao normal).
                let mut n = a.as_new();
                n.dedupe_key = Some(format!("{}:{ref_id}:{now}", a.kind));
                n.data["episode"] =
                    json!({"open": true, "since": fmt_date(today), "worst": a.worst});
                if let Some(id) = alerts::create(conn, &n)? {
                    created.push(id);
                }
            }
        }
    }
    // Episódios que acabaram: fecham (e saem dos novos) com uma linha de por quê.
    for (id, ref_id, old) in open {
        if active
            .iter()
            .any(|a| a.ref_id.as_deref() == Some(ref_id.as_str()))
        {
            continue;
        }
        let a = alerts::get(conn, &id)?;
        let mut d = old.clone();
        d["episode"]["open"] = json!(false);
        d["episode"]["closed"] = json!(fmt_date(today));
        // Uma linha de por quê no fim do corpo (pela chave: o texto segue o idioma de quem lê;
        // num alerta de antes da V128, depois do texto gravado).
        let why = match kind {
            "negative_balance" => {
                Msg::new("alertas-saldo-negativo-fim").arg("data", Arg::day_month(&fmt_date(today)))
            }
            _ => Msg::new("alertas-cotacao-normal"),
        };
        let (title, mut body) = alerts::stored(conn, &id)?;
        if !body
            .iter()
            .any(|p| matches!(p, Part::Msg(m) if m.key == why.key))
        {
            body.push(Part::from(" "));
            body.push(why.into());
        }
        alerts::update_content(conn, &id, a.severity, title.as_ref(), &body, &d, false)?;
        if a.status != alerts::AlertStatus::Resolved {
            alerts::set_status(
                conn,
                &alerts::AlertStatusArgs {
                    ids: vec![id],
                    status: alerts::AlertStatus::Resolved,
                },
            )?;
        }
    }
    Ok(created)
}

type Check = fn(&Connection, NaiveDate, &NotifyPrefs) -> Result<Vec<Owned>>;

/// `NewAlert` com dono (o de `alerts` empresta o tipo e a origem).
struct Owned {
    kind: &'static str,
    severity: AlertSeverity,
    title: Msg,
    body: Vec<Part>,
    dedupe_key: String,
    ref_table: Option<&'static str>,
    ref_id: Option<String>,
    data: serde_json::Value,
    /// Episódios: o número que, quando piora muito, avisa de novo (menor = pior).
    worst: Option<i64>,
}

impl Owned {
    fn as_new(&self) -> NewAlert<'_> {
        NewAlert {
            kind: self.kind,
            severity: self.severity,
            title: self.title.clone(),
            body: self.body.clone(),
            source: SOURCE,
            dedupe_key: Some(self.dedupe_key.clone()),
            ref_table: self.ref_table,
            ref_id: self.ref_id.clone(),
            data: self.data.clone(),
        }
    }
}

/// "hoje", "amanhã", "em 3 dias" (os dias contados na criação do alerta).
fn when(today: NaiveDate, date: &str) -> Msg {
    let days = parse_date(date)
        .map(|d| (d - today).num_days())
        .unwrap_or(0);
    Msg::new("alertas-quando").arg("dias", days.max(0))
}

/// Contas e faturas a pagar (mesma chave das rotinas do agente: nunca duas vezes).
fn bills(conn: &Connection, today: NaiveDate, p: &NotifyPrefs) -> Result<Vec<Owned>> {
    let items = crate::reports::upcoming(conn, today, p.bills_days)?;
    Ok(items
        .into_iter()
        .filter(|u| u.amount < 0)
        .map(|u| {
            let card = u.kind == crate::reports::UpcomingKind::CardDue;
            let days = parse_date(&u.date)
                .map(|d| (d - today).num_days())
                .unwrap_or(0);
            Owned {
                kind: "due_soon",
                severity: if days <= 1 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Info
                },
                title: if card {
                    Msg::new("alertas-vence-fatura").arg("conta", u.account_name.clone())
                } else {
                    Msg::new("alertas-vence-conta").arg("descricao", u.description.clone())
                }
                .arg("quando", when(today, &u.date)),
                body: vec![
                    Msg::new("alertas-vence-corpo")
                        .arg("valor", Arg::money(-u.amount, &u.currency))
                        .arg("conta", u.account_name.clone())
                        .arg("data", Arg::day_month(&u.date))
                        .into(),
                ],
                dedupe_key: format!("due:{}:{}", u.ref_id, u.date),
                ref_table: Some(if card { "card_statement" } else { "txn" }),
                ref_id: Some(u.ref_id.clone()),
                data: serde_json::to_value(&u).unwrap_or_default(),
                worst: None,
            }
        })
        .collect())
}

/// IPTU (só no Brasil): a parcela da forma escolhida que vence nos próximos dias (um alerta
/// por parcela) e, uma vez por ano, o carnê do ano que ainda não foi registrado.
fn iptu(conn: &Connection, today: NaiveDate, p: &NotifyPrefs) -> Result<Vec<Owned>> {
    if !crate::country::is_brazil(conn) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let until = today + Duration::days(p.bills_days.max(3) as i64);
    for d in crate::physical::iptu::due_installments(conn, today, until)? {
        let due = parse_date(&d.due_date)?;
        let days = (due - today).num_days();
        let part = if d.number == 0 {
            Msg::new("iptu-cota-unica")
        } else {
            Msg::new("iptu-parcela-n").arg("n", d.number.to_string())
        };
        let quando = match days {
            ..=0 => Msg::new("iptu-quando-hoje"),
            1 => Msg::new("iptu-quando-amanha"),
            n => Msg::new("iptu-quando-dias").arg("dias", n.to_string()),
        };
        // Os mesmos argumentos no título e no corpo.
        let a = |key: &str| {
            Msg::new(key)
                .arg("ano", d.year.to_string())
                .arg("imovel", d.asset_name.clone())
                .arg("parcela", Arg::lower(part.clone()))
                .arg("quando", quando.clone())
                .arg("valor", Arg::money(d.amount, &d.currency))
                .arg("data", Arg::date(&d.due_date))
        };
        out.push(Owned {
            kind: "iptu_due",
            severity: if days <= 1 {
                AlertSeverity::Warning
            } else {
                AlertSeverity::Info
            },
            title: a(if days < 0 {
                "iptu-alerta-titulo-vencido"
            } else {
                "iptu-alerta-titulo"
            }),
            body: vec![
                a(if d.undecided {
                    "iptu-alerta-corpo-decidir"
                } else {
                    "iptu-alerta-corpo"
                })
                .into(),
            ],
            dedupe_key: format!("iptu:{}", d.id),
            ref_table: Some("property_tax_installment"),
            ref_id: Some(d.id.clone()),
            data: json!({"due_date": d.due_date, "amount": d.amount, "currency": d.currency}),
            worst: None,
        });
    }
    for m in crate::physical::iptu::missing_bills(conn, today)? {
        let a = |key: &str| {
            Msg::new(key)
                .arg("ano", m.year.to_string())
                .arg("imovel", m.asset_name.clone())
        };
        out.push(Owned {
            kind: "iptu_missing",
            severity: AlertSeverity::Info,
            title: a("iptu-alerta-falta-titulo"),
            body: vec![a("iptu-alerta-falta-corpo").into()],
            dedupe_key: format!("iptu_missing:{}:{}", m.asset_id, m.year),
            ref_table: Some("physical_asset"),
            ref_id: Some(m.asset_id.clone()),
            data: json!({"year": m.year}),
            worst: None,
        });
    }
    Ok(out)
}

/// Data-com nos próximos 3 dias de ativos que você tem.
fn datacom(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    let to = today + Duration::days(3);
    let items = crate::portfolio::engine::dividend_calendar(
        conn,
        &crate::portfolio::DividendCalendarArgs {
            from: fmt_date(today),
            to: fmt_date(to + Duration::days(60)),
            only_mine: Some(true),
        },
    )?;
    Ok(items
        .into_iter()
        .filter(|i| !i.estimated)
        .filter_map(|i| {
            let ex = i.ex_date.clone()?;
            let d = parse_date(&ex).ok()?;
            if d < today || d > to {
                return None;
            }
            let name = i.asset.ticker.clone().unwrap_or_else(|| i.asset.name.clone());
            // Na chave de deduplicação (não muda) e no seletor do título.
            let (kind, tipo) = match i.kind {
                crate::portfolio::IncomeKind::Jcp => ("JCP", "jcp"),
                crate::portfolio::IncomeKind::FiiIncome => ("rendimento", "rendimento"),
                _ => ("provento", "provento"),
            };
            let opt = |m: Option<Msg>| m.map(Arg::from).unwrap_or_else(|| Arg::from(""));
            Some(Owned {
                kind: "dividend_datacom",
                severity: AlertSeverity::Info,
                title: Msg::new("alertas-datacom-titulo")
                    .arg("ativo", name.clone())
                    .arg("quando", when(today, &ex))
                    .arg("tipo", tipo),
                body: vec![
                    Msg::new("alertas-datacom-corpo")
                        .arg("ativo", name.clone())
                        .arg("data", Arg::day_month(&ex))
                        .arg("moeda", i.currency.clone())
                        .arg("valor", Arg::decimal(&i.amount_per_share))
                        .arg(
                            "pagamento",
                            opt(i.pay_date.as_deref().map(|p| {
                                Msg::new("alertas-datacom-pagamento").arg("data", Arg::day_month(p))
                            })),
                        )
                        .arg(
                            "liquido",
                            opt(i.expected_net.map(|v| {
                                Msg::new("alertas-datacom-liquido")
                                    .arg("valor", Arg::money(v, &i.currency))
                            })),
                        )
                        .into(),
                ],
                dedupe_key: format!("datacom:{}:{}:{ex}", i.asset.id, kind),
                ref_table: Some("asset"),
                ref_id: Some(i.asset.id.clone()),
                data: json!({"ex_date": ex, "pay_date": i.pay_date, "amount_per_share": i.amount_per_share}),
                worst: None,
            })
        })
        .collect())
}

/// Watchlist: cotação acima ou abaixo do alerta (uma vez por dia).
fn price_targets(conn: &Connection, _today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    Ok(crate::portfolio::records::list_watch(conn)?
        .into_iter()
        .filter(|w| !w.triggered.is_empty())
        .map(|w| {
            let name = w.asset.ticker.clone().unwrap_or_else(|| w.asset.name.clone());
            let close = w.price.as_ref().map(|p| p.close.clone()).unwrap_or_default();
            let price = close.replace('.', ",");
            // Os mesmos gatilhos de `list_watch` (cotação ≥ acima, ≤ abaixo), pela chave.
            let dec = |s: &Option<String>| {
                s.as_deref()
                    .and_then(|v| v.parse::<rust_decimal::Decimal>().ok())
            };
            let p = close.parse::<rust_decimal::Decimal>().ok();
            let above = dec(&w.alert_above).filter(|a| p.is_some_and(|p| p >= *a));
            let below = dec(&w.alert_below).filter(|b| p.is_some_and(|p| p <= *b));
            let title = match (above, below) {
                (Some(a), Some(b)) => Msg::new("alertas-cotacao-ambos")
                    .arg("acima", a.normalize().to_string())
                    .arg("abaixo", b.normalize().to_string()),
                (None, Some(b)) => {
                    Msg::new("alertas-cotacao-abaixo").arg("limite", b.normalize().to_string())
                }
                (Some(a), None) => {
                    Msg::new("alertas-cotacao-acima").arg("limite", a.normalize().to_string())
                }
                // Sem cotação para conferir de novo: o texto da watchlist, como veio.
                (None, None) => Msg::new("alertas-cotacao-gatilhos")
                    .arg("gatilhos", w.triggered.join(" e ")),
            }
            .arg("ativo", name.clone());
            Owned {
                kind: "price_target",
                severity: AlertSeverity::Info,
                title,
                body: vec![
                    Msg::new("alertas-cotacao-corpo")
                        .arg("ativo", name.clone())
                        .arg("moeda", w.asset.currency.clone())
                        .arg("preco", Arg::decimal(&close))
                        .into(),
                ],
                dedupe_key: format!("price_target:{}", w.id),
                ref_table: Some("asset"),
                ref_id: Some(w.asset.id.clone()),
                data: json!({"price": price, "alert_above": w.alert_above, "alert_below": w.alert_below}),
                worst: None,
            }
        })
        .collect())
}

/// Conta que fica negativa nos próximos 30 dias.
fn negative_balance(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    let f = crate::forecast::forecast(
        conn,
        &crate::forecast::ForecastArgs {
            months: Some(1),
            account_ids: None,
        },
        today,
    )?;
    Ok(f.accounts
        .into_iter()
        .filter_map(|a| {
            let date = a.first_negative.clone()?;
            Some(Owned {
                kind: "negative_balance",
                severity: AlertSeverity::Warning,
                // Sem data no título: o mesmo alerta segue valendo enquanto a conta fica negativa.
                title: Msg::new("alertas-saldo-negativo-titulo").arg("conta", a.name.clone()),
                body: vec![
                    Msg::new("alertas-saldo-negativo-corpo")
                        .arg("conta", a.name.clone())
                        .arg("data", Arg::day_month(&date))
                        .arg("minimo", Arg::money(a.min, &a.currency))
                        .arg("data_minimo", Arg::day_month(&a.min_date))
                        .into(),
                ],
                dedupe_key: format!("negative:{}", a.account_id),
                ref_table: Some("account"),
                ref_id: Some(a.account_id.clone()),
                data: json!({"first_negative": date, "min": a.min, "min_date": a.min_date}),
                worst: Some(a.min),
            })
        })
        .collect())
}

fn subscriptions(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    use crate::subscriptions::SubscriptionStatus;
    let r = crate::subscriptions::detect(conn, today)?;
    let recent = |d: &str| parse_date(d).is_ok_and(|d| (today - d).num_days() <= 15);
    let mut out = vec![];
    for s in r.items {
        if let (Some(c), Some(prev)) = (s.change_pct, s.previous_amount)
            && c > 0.0
            && recent(&s.last_date)
        {
            out.push(Owned {
                kind: "subscription_increase",
                severity: AlertSeverity::Info,
                title: Msg::new("alertas-assinatura-aumento-titulo")
                    .arg("nome", s.name.clone())
                    .arg("pct", Arg::pct(c, 0)),
                body: vec![
                    Msg::new("alertas-assinatura-aumento-corpo")
                        .arg("antes", Arg::money(prev, &s.currency))
                        .arg("agora", Arg::money(s.amount, &s.currency))
                        .arg("data", Arg::day_month(&s.last_date))
                        .arg("conta", s.account_name.clone())
                        .into(),
                ],
                dedupe_key: format!("sub_increase:{}:{}", s.key, s.last_date),
                ref_table: Some("txn"),
                ref_id: s.charges.last().map(|c| c.txn_id.clone()),
                data: json!({"previous": prev, "amount": s.amount, "change_pct": c}),
                worst: None,
            });
        }
        if s.status == SubscriptionStatus::New && recent(&s.last_date) {
            out.push(Owned {
                kind: "subscription_new",
                severity: AlertSeverity::Info,
                title: Msg::new("alertas-assinatura-nova-titulo").arg("nome", s.name.clone()),
                body: vec![
                    Msg::new("alertas-assinatura-nova-corpo")
                        .arg("valor", Arg::money(s.amount, &s.currency))
                        .arg(
                            "periodo",
                            if s.cadence == crate::subscriptions::SubscriptionCadence::Yearly {
                                "ano"
                            } else {
                                "mes"
                            },
                        )
                        .arg("data", Arg::day_month(&s.first_date))
                        .arg("conta", s.account_name.clone())
                        .into(),
                ],
                dedupe_key: format!("sub_new:{}", s.key),
                ref_table: Some("txn"),
                ref_id: s.charges.last().map(|c| c.txn_id.clone()),
                data: json!({"amount": s.amount, "first_date": s.first_date}),
                worst: None,
            });
        }
    }
    Ok(out)
}

/// Categorias de despesa acima do previsto no mês (mesma chave das rotinas do agente).
/// Meta de faculdade (docs/design/college-goal.md §6): o recálculo anual, a partir de abril,
/// avisa só quando o aporte muda mais de 10% ou quando o curso terminou.
fn education(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    let year = today.year();
    Ok(crate::goals::education::annual_review(conn, today)?
        .into_iter()
        .map(|c| {
            if c.finished {
                Owned {
                    kind: "education_finished",
                    severity: AlertSeverity::Info,
                    title: Msg::new("educacao-alerta-terminou-titulo").arg("nome", c.name.clone()),
                    body: vec![Msg::new("educacao-alerta-terminou-corpo").into()],
                    dedupe_key: format!("education_finished:{}", c.goal_id),
                    ref_table: Some("goal"),
                    ref_id: Some(c.goal_id.clone()),
                    data: json!({"goal_id": c.goal_id}),
                    worst: None,
                }
            } else {
                Owned {
                    kind: "education_review",
                    severity: if c.new_monthly > c.old_monthly {
                        AlertSeverity::Warning
                    } else {
                        AlertSeverity::Info
                    },
                    title: Msg::new("educacao-alerta-recalculo-titulo").arg("nome", c.name.clone()),
                    body: vec![
                        Msg::new("educacao-alerta-recalculo-corpo")
                            .arg("antes", Arg::money(c.old_monthly, &c.currency))
                            .arg("depois", Arg::money(c.new_monthly, &c.currency))
                            .into(),
                    ],
                    dedupe_key: format!("education_review:{year}:{}", c.goal_id),
                    ref_table: Some("goal"),
                    ref_id: Some(c.goal_id.clone()),
                    data: json!({"goal_id": c.goal_id, "old": c.old_monthly, "new": c.new_monthly, "currency": c.currency}),
                    worst: None,
                }
            }
        })
        .collect())
}

/// Pontos e milhas vencendo: um aviso por programa, data de vencimento e faixa (90, 30 e 7
/// dias antes).
fn loyalty(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    let mut out = vec![];
    for p in crate::cards::loyalty::list(conn, today)? {
        if !p.active {
            continue;
        }
        let list = crate::cards::loyalty::entries(conn, &p.id)?;
        let (lots, _) = crate::cards::loyalty::open_lots(&list, today);
        let mut by_date: std::collections::BTreeMap<String, i64> = Default::default();
        for (d, pts) in lots {
            if let Some(d) = d {
                *by_date.entry(d).or_default() += pts;
            }
        }
        for (date, pts) in by_date {
            let days = (parse_date(&date)? - today).num_days();
            let Some(band) = [7, 30, 90].into_iter().find(|b| days <= *b) else {
                continue;
            };
            out.push(Owned {
                kind: "loyalty_expiring",
                severity: if band <= 7 {
                    AlertSeverity::Critical
                } else if band <= 30 {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Info
                },
                title: Msg::new("alertas-pontos-vencendo-titulo")
                    .arg("programa", p.name.clone())
                    .arg("quando", when(today, &date)),
                body: vec![
                    Msg::new("alertas-pontos-vencendo-corpo")
                        .arg("pontos", pts.to_string())
                        .arg("data", Arg::date(&date))
                        .arg("membro", p.member_name.clone().unwrap_or_default())
                        .into(),
                ],
                dedupe_key: format!("loyalty_expiring:{}:{date}:{band}", p.id),
                ref_table: None,
                ref_id: Some(p.id.clone()),
                data: json!({"program_id": p.id, "date": date, "points": pts}),
                worst: None,
            });
        }
    }
    Ok(out)
}

/// Cartões: uso do limite a partir de 80% e de 95% (um aviso por faixa e fatura), anuidade
/// renovando nos próximos 30 dias (um por renovação) e cobrança de anuidade que não confere
/// com a regra (um por cobrança).
fn cards(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    let ids: Vec<(String, String)> = conn
        .prepare(
            "SELECT a.id, a.name FROM account a JOIN credit_card c ON c.account_id = a.id
              WHERE a.archived = 0 ORDER BY a.sort_order, a.name",
        )?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (id, name) in ids {
        let o = crate::cards::limits::overview(conn, &id, today)?;
        if let (Some(bp), Some(limit)) = (o.utilization_bp, o.credit_limit) {
            let band = crate::cards::limits::WARN_BP
                .iter()
                .rev()
                .find(|b| bp >= **b)
                .copied();
            if let Some(band) = band {
                let month = o
                    .open_statement
                    .as_ref()
                    .map(|s| s.reference_month.clone())
                    .unwrap_or_else(|| month_of(today));
                out.push(Owned {
                    kind: "card_limit",
                    severity: if band >= 9500 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::Warning
                    },
                    title: Msg::new("alertas-cartao-limite-titulo")
                        .arg("cartao", name.clone())
                        .arg("pct", Arg::pct(f64::from(bp) / 10_000.0, 0)),
                    body: vec![
                        Msg::new("alertas-cartao-limite-corpo")
                            .arg("usado", Arg::money(o.used, &o.currency))
                            .arg("limite", Arg::money(limit, &o.currency))
                            .arg("disponivel", Arg::money(limit - o.used, &o.currency))
                            .into(),
                    ],
                    dedupe_key: format!("card_limit:{id}:{month}:{band}"),
                    ref_table: Some("account"),
                    ref_id: Some(id.clone()),
                    data: json!({"utilization_bp": bp, "used": o.used, "limit": limit, "currency": o.currency}),
                    worst: None,
                });
            }
        }
        if let Some(r) = &o.revolving {
            let br = o.brazil_rules;
            out.push(Owned {
                kind: "card_revolving",
                severity: AlertSeverity::Critical,
                title: Msg::new("alertas-cartao-rotativo-titulo")
                    .arg("cartao", name.clone())
                    .arg("mes", Arg::month(&r.reference_month)),
                body: vec![
                    Msg::new(if br {
                        "alertas-cartao-rotativo-corpo-br"
                    } else {
                        "alertas-cartao-rotativo-corpo"
                    })
                    .arg("valor", Arg::money(r.unpaid, &r.currency))
                    .arg("data", Arg::day_month(&r.due_date))
                    .into(),
                ],
                dedupe_key: format!("card_revolving:{}", r.statement_id),
                ref_table: Some("card_statement"),
                ref_id: Some(r.statement_id.clone()),
                data: serde_json::to_value(r).unwrap_or_default(),
                worst: None,
            });
            if br && r.over_cap {
                out.push(Owned {
                    kind: "card_revolving_cap",
                    severity: AlertSeverity::Critical,
                    title: Msg::new("alertas-cartao-rotativo-teto-titulo")
                        .arg("cartao", name.clone()),
                    body: vec![
                        Msg::new("alertas-cartao-rotativo-teto-corpo")
                            .arg("encargos", Arg::money(r.charges, &r.currency))
                            .arg("valor", Arg::money(r.unpaid, &r.currency))
                            .into(),
                    ],
                    dedupe_key: format!("card_revolving_cap:{}", r.statement_id),
                    ref_table: Some("card_statement"),
                    ref_id: Some(r.statement_id.clone()),
                    data: serde_json::to_value(r).unwrap_or_default(),
                    worst: None,
                });
            }
        }
        let rewards = crate::cards::rewards::view(conn, &id, today)?;
        let points = rewards
            .program
            .as_ref()
            .is_some_and(|p| p.kind == crate::cards::rewards::RewardKind::Points);
        let amount = |v: Option<i64>| -> Arg {
            if points {
                Arg::from(v.unwrap_or(0).to_string())
            } else {
                Arg::money(v.unwrap_or(0), &rewards.currency)
            }
        };
        for s in rewards.statements.iter().filter(|s| s.missing) {
            let (title, body) = if points {
                (
                    "alertas-cartao-pontos-titulo",
                    "alertas-cartao-pontos-corpo",
                )
            } else {
                (
                    "alertas-cartao-cashback-titulo",
                    "alertas-cartao-cashback-corpo",
                )
            };
            out.push(Owned {
                kind: "card_cashback_missing",
                severity: AlertSeverity::Warning,
                title: Msg::new(title)
                    .arg("cartao", name.clone())
                    .arg("mes", Arg::month(&s.reference_month)),
                body: vec![
                    Msg::new(body)
                        .arg("previsto", amount(s.expected))
                        .arg("recebido", amount(s.received))
                        .arg("data", Arg::day_month(&s.due_by))
                        .into(),
                ],
                dedupe_key: format!("card_cashback_missing:{}", s.statement_id),
                ref_table: Some("card_statement"),
                ref_id: Some(s.statement_id.clone()),
                data: serde_json::to_value(s).unwrap_or_default(),
                worst: None,
            });
        }
        let Some(fee) = &o.annual_fee else { continue };
        if let Some(renewal) = &fee.next_renewal
            && let Ok(d) = parse_date(renewal)
            && (d - today).num_days() <= 30
        {
            out.push(Owned {
                kind: "card_fee_renewal",
                severity: AlertSeverity::Info,
                title: Msg::new("alertas-cartao-anuidade-renova-titulo")
                    .arg("cartao", name.clone())
                    .arg("mes", Arg::month(&renewal[..7])),
                body: vec![
                    Msg::new("alertas-cartao-anuidade-renova-corpo")
                        .arg("valor", Arg::money(fee.rule.amount, &o.currency))
                        .arg("pago", Arg::money(fee.paid_12m, &o.currency))
                        .into(),
                ],
                dedupe_key: format!("card_fee_renewal:{id}:{renewal}"),
                ref_table: Some("account"),
                ref_id: Some(id.clone()),
                data: json!({"renewal": renewal, "amount": fee.rule.amount, "paid_12m": fee.paid_12m}),
                worst: None,
            });
        }
        for c in fee.charges.iter().filter(|c| c.issue.is_some()) {
            let key = match c.issue {
                Some(crate::cards::annual_fee::FeeIssue::ChargedWhileWaived) => {
                    "alertas-cartao-anuidade-isenta-corpo"
                }
                Some(crate::cards::annual_fee::FeeIssue::ExtraInstallment) => {
                    "alertas-cartao-anuidade-parcela-extra-corpo"
                }
                _ => "alertas-cartao-anuidade-valor-corpo",
            };
            out.push(Owned {
                kind: "card_fee_issue",
                severity: AlertSeverity::Warning,
                title: Msg::new("alertas-cartao-anuidade-confira-titulo")
                    .arg("cartao", name.clone()),
                body: vec![
                    Msg::new(key)
                        .arg("valor", Arg::money(c.amount, &o.currency))
                        .arg("esperado", Arg::money(c.expected.unwrap_or(0), &o.currency))
                        .arg("data", Arg::day_month(&c.date))
                        .into(),
                ],
                dedupe_key: format!("card_fee_issue:{}", c.txn_id),
                ref_table: Some("txn"),
                ref_id: Some(c.txn_id.clone()),
                data: serde_json::to_value(c).unwrap_or_default(),
                worst: None,
            });
        }
    }
    Ok(out)
}

/// Título e corpo de "N contas sem extrato recente", com a data do último lançamento de cada
/// uma. O lembrete semanal e a lista de pendências (`attention`) dizem a mesma frase.
pub(crate) fn stale_accounts_text(stale: &[crate::accounts::AccountView]) -> (Msg, Vec<Part>) {
    let items: Vec<Arg> = stale
        .iter()
        .map(|a| {
            let m = match &a.last_txn_date {
                Some(d) => Msg::new("alertas-contas-desatualizadas-item")
                    .arg("conta", a.account.name.clone())
                    .arg("data", Arg::day_month(d)),
                None => Msg::new("alertas-contas-desatualizadas-item-sem")
                    .arg("conta", a.account.name.clone()),
            };
            Arg::Typed(alerts::Typed::Msg(m))
        })
        .collect();
    (
        Msg::new("alertas-contas-desatualizadas-titulo").arg("n", Arg::Int(stale.len() as i64)),
        vec![
            Msg::new("alertas-contas-desatualizadas-corpo")
                .arg("contas", Arg::join(items, "; "))
                .into(),
        ],
    )
}

/// Contas do dia a dia com o saldo possivelmente incompleto (docs/account-freshness.md): um
/// lembrete por semana, com o atalho para a importação.
fn stale_accounts(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    let stale = crate::accounts::freshness::everyday_incomplete(conn, today)?;
    if stale.is_empty() {
        return Ok(vec![]);
    }
    let (title, body) = stale_accounts_text(&stale);
    let week = today.iso_week();
    Ok(vec![Owned {
        kind: "stale_accounts",
        severity: AlertSeverity::Info,
        title,
        body,
        dedupe_key: format!("stale_accounts:{}-W{:02}", week.year(), week.week()),
        ref_table: Some("import"),
        ref_id: stale.first().map(|a| a.account.id.clone()),
        data: json!({"accounts": stale.iter().map(|a| json!({
            "account_id": a.account.id,
            "name": a.account.name,
            "last_txn_date": a.last_txn_date,
            "staleness_days": a.staleness_days,
        })).collect::<Vec<_>>()}),
        worst: None,
    }])
}

fn budget(conn: &Connection, today: NaiveDate, _: &NotifyPrefs) -> Result<Vec<Owned>> {
    let month = month_of(today);
    let b = crate::budget::get(conn, &month, None)?;
    Ok(b.lines
        .into_iter()
        .filter(|l| {
            l.kind == crate::categories::CategoryKind::Expense && l.planned > 0 && l.actual > l.planned
        })
        .map(|l| {
            let ratio = l.actual as f64 / l.planned as f64;
            Owned {
                kind: "budget_over",
                severity: if ratio >= 1.2 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                title: Msg::new("alertas-orcamento-estourado-titulo")
                    .arg("categoria", l.category_path.clone()),
                body: vec![
                    Msg::new("alertas-orcamento-estourado-corpo")
                        .arg("gasto", Arg::money(l.actual, &b.currency))
                        .arg("previsto", Arg::money(l.planned, &b.currency))
                        .arg("pct", Arg::pct(ratio, 0))
                        .into(),
                ],
                dedupe_key: format!("budget_over:{month}:{}", l.category_id),
                ref_table: Some("category"),
                ref_id: Some(l.category_id.clone()),
                data: json!({"month": month, "planned": l.planned, "actual": l.actual, "ratio": ratio, "currency": b.currency}),
                worst: None,
            }
        })
        .collect())
}
