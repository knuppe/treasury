//! Sugestões proativas do agente companheiro (`ai.suggestions`): o que vale fazer agora,
//! calculado **de forma determinística** a partir dos dados (nenhum motor de IA é
//! chamado, nada sai do computador). Cada sugestão traz uma pergunta pronta para o agente
//! e a tela onde resolver; a interface ordena pela tela aberta.
//!
//! Regras: primeiro cadastro (sem contas), lançamentos sem categoria, contas sem extrato
//! recente, fatura vencendo, orçamento estourado, ativos sem cotação, corretora sem
//! operações recentes (importação da B3), aplicações de liquidez diária fora de qualquer meta
//! (ligar à reserva de emergência), propostas pendentes e alertas novos.
//!
//! Título, detalhe e pedido saem do catálogo (`locales/*/sugestoes.ftl`) no idioma da
//! instalação: o pedido vai ao agente como mensagem do usuário.

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::i18n::{self, Args, args, t_args};
use crate::time::{fmt_date, month_of};

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SuggestionArgs {
    /// Tela aberta (`painel`, `contas`, `lancamentos`…): as sugestões dela vêm primeiro.
    #[ts(optional)]
    pub screen: Option<String>,
    /// Máximo de sugestões (padrão 6).
    #[ts(optional)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SuggestionKind {
    /// Nenhuma conta cadastrada: começar pelo cadastro guiado.
    Onboarding,
    Uncategorized,
    /// Conta sem lançamentos recentes: importar o extrato.
    StaleImport,
    CardDue,
    BudgetOver,
    AssetNoPrice,
    /// Corretora sem operações recentes: importar da Área do Investidor B3.
    B3Import,
    /// Aplicações de liquidez diária (CDB/RDB com liquidez diária, Tesouro Selic, fundo DI)
    /// fora de qualquer meta: ligar à reserva de emergência.
    ReserveLiquidity,
    PendingProposals,
    NewAlerts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SuggestionSeverity {
    Info,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Suggestion {
    /// Chave estável (para dispensar na interface).
    pub id: String,
    pub kind: SuggestionKind,
    pub severity: SuggestionSeverity,
    pub title: String,
    pub detail: String,
    /// Pedido pronto para o agente.
    pub prompt: String,
    /// Onde resolver no app (rota com `#`), se houver.
    pub route: Option<String>,
    /// Telas em que a sugestão é mais relevante.
    pub screens: Vec<String>,
    #[ts(type = "number")]
    pub count: i64,
}

/// Dias sem lançamentos para sugerir a importação do extrato.
pub const STALE_DAYS: i64 = 30;
/// Dias sem operações numa corretora para sugerir a importação da B3.
pub const B3_STALE_DAYS: i64 = 45;
/// Antecedência do aviso de fatura.
pub const CARD_DUE_DAYS: i64 = 7;

fn names(list: &[String]) -> String {
    let shown: Vec<&str> = list.iter().take(3).map(String::as_str).collect();
    let rest = list.len().saturating_sub(shown.len());
    let s = shown.join(", ");
    if rest > 0 {
        t_args(
            "sugestoes-nomes-mais",
            &args(&[("nomes", s.into()), ("n", (rest as i64).into())]),
        )
    } else {
        s
    }
}

/// Uma contagem como argumento (`{ $n }`).
fn count(n: i64) -> Args {
    args(&[("n", n.into())])
}

fn screens(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// Calcula as sugestões na data `today`.
pub fn compute(conn: &Connection, a: &SuggestionArgs, today: NaiveDate) -> Result<Vec<Suggestion>> {
    let mut out: Vec<Suggestion> = vec![];
    let t = fmt_date(today);
    let accounts: i64 =
        conn.query_row("SELECT count(*) FROM account WHERE archived = 0", [], |r| {
            r.get(0)
        })?;
    if accounts == 0 {
        out.push(Suggestion {
            id: "onboarding".into(),
            kind: SuggestionKind::Onboarding,
            severity: SuggestionSeverity::Info,
            title: i18n::t("sugestoes-cadastro-titulo"),
            detail: i18n::t("sugestoes-cadastro-detalhe"),
            prompt: onboarding_prompt(crate::country::is_brazil(conn)),
            route: Some("/contas".into()),
            screens: screens(&["painel", "contas", "lancamentos", "carteira"]),
            count: 0,
        });
    }

    // Lançamentos sem categoria (últimos 90 dias, efetivados, sem divisão).
    let from = fmt_date(today - Duration::days(90));
    let uncategorized: i64 = conn.query_row(
        "SELECT count(*) FROM txn t
         WHERE t.kind IN ('income', 'expense') AND t.category_id IS NULL AND t.status != 'planned'
           AND t.date BETWEEN ?1 AND ?2
           AND NOT EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)",
        params![from, t],
        |r| r.get(0),
    )?;
    if uncategorized > 0 {
        out.push(Suggestion {
            id: "uncategorized".into(),
            kind: SuggestionKind::Uncategorized,
            severity: SuggestionSeverity::Info,
            title: t_args("sugestoes-sem-categoria-titulo", &count(uncategorized)),
            detail: i18n::t("sugestoes-sem-categoria-detalhe"),
            prompt: t_args(
                "sugestoes-sem-categoria-pedido",
                &args(&[("n", uncategorized.into()), ("desde", from.clone().into())]),
            ),
            route: Some(format!("/lancamentos?uncategorized=1&from={from}&to={t}")),
            screens: screens(&["lancamentos", "painel", "orcamento"]),
            count: uncategorized,
        });
    }

    // Contas de movimento sem lançamentos recentes.
    let stale_before = fmt_date(today - Duration::days(STALE_DAYS));
    let mut stmt = conn.prepare(
        "SELECT a.name, (SELECT max(date) FROM txn WHERE account_id = a.id AND status != 'planned')
         FROM account a
         WHERE a.archived = 0 AND a.kind IN ('checking', 'savings', 'credit_card')
         ORDER BY a.sort_order, a.name",
    )?;
    let stale: Vec<(String, Option<String>)> = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, last)| last.as_deref().is_none_or(|d| d < stale_before.as_str()))
        .collect();
    if !stale.is_empty() {
        let list: Vec<String> = stale.iter().map(|(n, _)| n.clone()).collect();
        out.push(Suggestion {
            id: "stale_import".into(),
            kind: SuggestionKind::StaleImport,
            severity: SuggestionSeverity::Info,
            title: t_args("sugestoes-sem-extrato-titulo", &count(list.len() as i64)),
            detail: t_args(
                "sugestoes-sem-extrato-detalhe",
                &args(&[("contas", names(&list).into()), ("dias", STALE_DAYS.into())]),
            ),
            prompt: t_args(
                "sugestoes-sem-extrato-pedido",
                &args(&[("contas", names(&list).into()), ("dias", STALE_DAYS.into())]),
            ),
            route: Some("/lancamentos?aba=importar".into()),
            screens: screens(&["contas", "lancamentos", "painel"]),
            count: list.len() as i64,
        });
    }

    // Faturas a vencer nos próximos dias, ainda não pagas.
    let until = fmt_date(today + Duration::days(CARD_DUE_DAYS));
    let mut stmt = conn.prepare(
        "SELECT a.name, s.due_date, a.currency,
                -COALESCE((SELECT sum(amount) FROM txn WHERE statement_id = s.id AND status != 'planned'), 0)
         FROM card_statement s JOIN account a ON a.id = s.account_id
         WHERE a.archived = 0 AND s.status != 'paid' AND s.due_date BETWEEN ?1 AND ?2
         ORDER BY s.due_date",
    )?;
    let due: Vec<(String, String, String, i64)> = stmt
        .query_map(params![t, until], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, _, _, total)| *total > 0)
        .collect();
    if let Some((name, date, cur, total)) = due.first() {
        let l = i18n::current();
        let a = args(&[
            ("cartao", name.clone().into()),
            ("data", date.clone().into()),
            ("valor", i18n::money(l, *total, cur).into()),
            ("n", (due.len() as i64 - 1).into()),
        ]);
        out.push(Suggestion {
            id: "card_due".into(),
            kind: SuggestionKind::CardDue,
            severity: SuggestionSeverity::Warning,
            title: t_args(
                "sugestoes-fatura-titulo",
                &args(&[
                    ("cartao", name.clone().into()),
                    ("data", br_date(date).into()),
                ]),
            ),
            detail: t_args(
                if due.len() > 1 {
                    "sugestoes-fatura-detalhe-mais"
                } else {
                    "sugestoes-fatura-detalhe"
                },
                &a,
            ),
            prompt: t_args("sugestoes-fatura-pedido", &a),
            route: Some("/contas".into()),
            screens: screens(&["contas", "painel", "lancamentos"]),
            count: due.len() as i64,
        });
    }

    // Orçamento do mês: categorias de despesa acima do previsto.
    let month = month_of(today);
    if let Ok(b) = crate::budget::get(conn, &month, None) {
        let over: Vec<&crate::budget::BudgetLineStatus> = b
            .lines
            .iter()
            .filter(|l| {
                l.kind == crate::categories::CategoryKind::Expense
                    && l.has_own_line
                    && l.planned > 0
                    && l.actual > l.planned
            })
            .collect();
        if !over.is_empty() {
            let list: Vec<String> = over.iter().map(|l| l.category_path.clone()).collect();
            let excess: i64 = over.iter().map(|l| l.actual - l.planned).sum();
            out.push(Suggestion {
                id: "budget_over".into(),
                kind: SuggestionKind::BudgetOver,
                severity: SuggestionSeverity::Warning,
                title: t_args("sugestoes-orcamento-titulo", &count(over.len() as i64)),
                detail: t_args(
                    "sugestoes-orcamento-detalhe",
                    &args(&[
                        ("categorias", names(&list).into()),
                        (
                            "valor",
                            i18n::money(i18n::current(), excess, &b.currency).into(),
                        ),
                    ]),
                ),
                prompt: t_args(
                    "sugestoes-orcamento-pedido",
                    &args(&[
                        ("mes", month.clone().into()),
                        ("categorias", names(&list).into()),
                    ]),
                ),
                route: Some(format!("/orcamento?mes={month}")),
                screens: screens(&["orcamento", "painel"]),
                count: over.len() as i64,
            });
        }
    }

    // Ativos com posição sem cotação recente (exceto renda fixa, avaliada pela curva).
    let price_since = fmt_date(today - Duration::days(15));
    let mut stmt = conn.prepare(
        "SELECT coalesce(a.ticker, a.name) FROM asset a
         WHERE a.archived = 0 AND a.class NOT IN ('fixed_income', 'pension')
           AND EXISTS (SELECT 1 FROM trade WHERE asset_id = a.id)
           AND NOT EXISTS (SELECT 1 FROM price p WHERE p.asset_id = a.id AND p.date >= ?1)
         ORDER BY 1",
    )?;
    let no_price: Vec<String> = stmt
        .query_map([&price_since], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    if !no_price.is_empty() {
        out.push(Suggestion {
            id: "asset_no_price".into(),
            kind: SuggestionKind::AssetNoPrice,
            severity: SuggestionSeverity::Info,
            title: t_args("sugestoes-cotacao-titulo", &count(no_price.len() as i64)),
            detail: t_args(
                "sugestoes-cotacao-detalhe",
                &args(&[("ativos", names(&no_price).into())]),
            ),
            prompt: t_args(
                "sugestoes-cotacao-pedido",
                &args(&[("ativos", names(&no_price).into())]),
            ),
            route: Some("/carteira".into()),
            screens: screens(&["carteira", "ativos"]),
            count: no_price.len() as i64,
        });
    }

    // Corretoras sem operações recentes: sugerir a importação da B3 (só no Brasil).
    let b3_before = fmt_date(today - Duration::days(B3_STALE_DAYS));
    let mut stmt = conn.prepare(
        "SELECT a.name, (SELECT max(date) FROM trade WHERE account_id = a.id)
         FROM account a WHERE a.archived = 0 AND a.kind = 'brokerage' ORDER BY a.name",
    )?;
    let brokers: Vec<String> = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, last)| last.as_deref().is_none_or(|d| d < b3_before.as_str()))
        .map(|(n, _)| n)
        .collect();
    if !brokers.is_empty() && crate::country::is_brazil(conn) {
        out.push(Suggestion {
            id: "b3_import".into(),
            kind: SuggestionKind::B3Import,
            severity: SuggestionSeverity::Info,
            title: i18n::t("sugestoes-b3-titulo"),
            detail: t_args(
                "sugestoes-b3-detalhe",
                &args(&[
                    ("corretoras", names(&brokers).into()),
                    ("dias", B3_STALE_DAYS.into()),
                ]),
            ),
            prompt: t_args(
                "sugestoes-b3-pedido",
                &args(&[("corretoras", names(&brokers).into())]),
            ),
            route: Some("/carteira".into()),
            screens: screens(&["carteira", "ativos"]),
            count: brokers.len() as i64,
        });
    }

    // Liquidez diária fora de qualquer meta: a reserva de emergência pode contar com ela.
    let liquid = daily_liquidity_outside_goals(conn)?;
    if !liquid.is_empty() {
        out.push(Suggestion {
            id: "reserve_liquidity".into(),
            kind: SuggestionKind::ReserveLiquidity,
            severity: SuggestionSeverity::Info,
            title: t_args("sugestoes-reserva-titulo", &count(liquid.len() as i64)),
            detail: t_args(
                "sugestoes-reserva-detalhe",
                &args(&[("ativos", names(&liquid).into())]),
            ),
            prompt: t_args(
                "sugestoes-reserva-pedido",
                &args(&[("ativos", names(&liquid).into())]),
            ),
            route: Some("/orcamento?aba=metas".into()),
            screens: screens(&["orcamento", "carteira", "painel"]),
            count: liquid.len() as i64,
        });
    }

    let pending = super::proposals::pending_count(conn)?;
    if pending > 0 {
        out.push(Suggestion {
            id: "pending_proposals".into(),
            kind: SuggestionKind::PendingProposals,
            severity: SuggestionSeverity::Info,
            title: t_args("sugestoes-propostas-titulo", &count(pending)),
            detail: i18n::t("sugestoes-propostas-detalhe"),
            prompt: i18n::t("sugestoes-propostas-pedido"),
            route: Some("/agente?aba=aprovacao".into()),
            screens: screens(&["agente"]),
            count: pending,
        });
    }

    let alerts: i64 =
        conn.query_row("SELECT count(*) FROM alert WHERE status = 'new'", [], |r| {
            r.get(0)
        })?;
    if alerts > 0 {
        out.push(Suggestion {
            id: "new_alerts".into(),
            kind: SuggestionKind::NewAlerts,
            severity: SuggestionSeverity::Info,
            title: t_args("sugestoes-alertas-titulo", &count(alerts)),
            detail: i18n::t("sugestoes-alertas-detalhe"),
            prompt: i18n::t("sugestoes-alertas-pedido"),
            route: Some("/agente?aba=alertas".into()),
            screens: screens(&["agente", "painel"]),
            count: alerts,
        });
    }

    // Ordem: a tela aberta primeiro, depois avisos, depois a ordem acima.
    let screen = a.screen.as_deref().unwrap_or_default();
    let mut indexed: Vec<(usize, Suggestion)> = out.into_iter().enumerate().collect();
    indexed.sort_by_key(|(i, s)| {
        (
            !(s.kind == SuggestionKind::Onboarding || s.screens.iter().any(|x| x == screen)),
            std::cmp::Reverse(s.severity),
            *i,
        )
    });
    let limit = a.limit.unwrap_or(6).clamp(1, 20) as usize;
    Ok(indexed.into_iter().map(|(_, s)| s).take(limit).collect())
}

/// Aplicações com posição aberta e liquidez diária (renda fixa com liquidez diária, Tesouro
/// Selic, fundo DI ou referenciado) que não estão ligadas a nenhuma meta ativa, nem pelo ativo
/// nem pela conta inteira.
fn daily_liquidity_outside_goals(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT coalesce(a.ticker, a.name) FROM asset a
           LEFT JOIN fixed_income_terms f ON f.asset_id = a.id
          WHERE a.archived = 0
            AND ((f.liquidity = 'daily' AND f.kind <> 'other') OR f.kind = 'tesouro_selic'
                 OR (a.class = 'fund' AND (' ' || lower(a.name) || ' ' LIKE '% di %'
                                           OR lower(a.name) LIKE '%referenciado%')))
            AND (SELECT coalesce(sum(CASE WHEN t.side = 'buy' THEN CAST(t.quantity AS REAL)
                                          ELSE -CAST(t.quantity AS REAL) END), 0)
                   FROM trade t WHERE t.asset_id = a.id) > 0
            AND NOT EXISTS (
                SELECT 1 FROM goal_holding h JOIN goal g ON g.id = h.goal_id
                 WHERE g.archived = 0
                   AND (h.asset_id = a.id
                        OR (h.asset_id IS NULL
                            AND h.account_id IN (SELECT t.account_id FROM trade t WHERE t.asset_id = a.id))))
          ORDER BY a.name",
    )?;
    let out = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(out)
}

/// Dia e mês de uma data ISO (14/10, Oct 14).
fn br_date(iso: &str) -> String {
    match crate::time::parse_date(iso) {
        Ok(d) => crate::ai::alerts::day_month(i18n::current(), d),
        Err(_) => iso.to_string(),
    }
}

/// Pedido do cadastro guiado ("onde vocês têm contas?"), no idioma da instalação. Fora do
/// Brasil, sem a B3 e as notas de corretagem.
pub fn onboarding_prompt(brazil: bool) -> String {
    i18n::t(if brazil {
        "sugestoes-cadastro-pedido"
    } else {
        "sugestoes-cadastro-pedido-exterior"
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::api::{Api, SharedApi};
    use crate::paths::TreasuryPaths;

    fn vault() -> (tempfile::TempDir, SharedApi) {
        let dir = tempfile::tempdir().unwrap();
        let paths = TreasuryPaths::with_root(dir.path().join(".t")).unwrap();
        let api = Api::new_unlocked_process(paths).unwrap().into_shared();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
        )
        .unwrap();
        (dir, api)
    }

    fn kinds(api: &Api, screen: &str, today: &str) -> Vec<SuggestionKind> {
        let today = crate::time::parse_date(today).unwrap();
        api.with_session(|s, _| {
            compute(
                s.db.conn(),
                &SuggestionArgs {
                    screen: Some(screen.into()),
                    limit: Some(20),
                },
                today,
            )
        })
        .unwrap()
        .into_iter()
        .map(|s| s.kind)
        .collect()
    }

    #[test]
    fn suggestions_follow_the_data_and_the_screen() {
        let (_d, api) = vault();
        assert_eq!(
            kinds(&api, "painel", "2026-10-14"),
            vec![SuggestionKind::Onboarding]
        );
        let acc = api
            .call(
                "accounts.create",
                json!({"name": "Itaú", "kind": "checking", "opening_date": "2026-01-01"}),
            )
            .unwrap()["id"]
            .clone();
        let card = api
            .call(
                "accounts.create",
                json!({"name": "Nubank", "kind": "credit_card", "opening_date": "2026-01-01",
                       "card": {"credit_limit": 100000, "closing_day": 3, "due_day": 10,
                                "payment_account_id": acc, "network": null, "last4": null}}),
            )
            .unwrap()["id"]
            .clone();
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-01", "amount": -5000, "description": "Mercado"}),
        )
        .unwrap();
        api.call(
            "transactions.create",
            json!({"account_id": card, "date": "2026-10-02", "amount": -12000, "description": "Loja"}),
        )
        .unwrap();
        api.call(
            "accounts.create",
            json!({"name": "XP", "kind": "brokerage", "opening_date": "2026-01-01"}),
        )
        .unwrap();
        let k = kinds(&api, "lancamentos", "2026-10-14");
        assert_eq!(k[0], SuggestionKind::Uncategorized, "{k:?}");
        assert!(!k.contains(&SuggestionKind::Onboarding));
        assert!(!k.contains(&SuggestionKind::StaleImport), "{k:?}");
        assert!(k.contains(&SuggestionKind::B3Import));
        // A fatura de outubro (fechamento 3) tem a compra do dia 2 e vence dia 10.
        let k = kinds(&api, "contas", "2026-10-05");
        assert!(k.contains(&SuggestionKind::CardDue), "{k:?}");
        assert_eq!(
            k[0],
            SuggestionKind::CardDue,
            "aviso da tela vem primeiro: {k:?}"
        );
        // Dois meses depois, as contas estão sem extrato.
        let k = kinds(&api, "contas", "2026-12-20");
        assert!(k.contains(&SuggestionKind::StaleImport), "{k:?}");
        let s = api
            .call("ai.suggestions", json!({"screen": "carteira"}))
            .unwrap();
        assert_eq!(s[0]["kind"], "b3_import");
        assert!(
            s[0]["prompt"]
                .as_str()
                .unwrap()
                .contains("Área do Investidor")
        );
    }

    /// O cartão e o pedido saem no idioma da instalação (o pedido vai ao agente como mensagem
    /// do usuário).
    #[test]
    fn suggestions_follow_the_locale() {
        use crate::i18n::{Locale, set_current};
        let (_d, api) = vault();
        let today = crate::time::parse_date("2026-10-14").unwrap();
        let first = |api: &Api| {
            api.with_session(|s, _| compute(s.db.conn(), &SuggestionArgs::default(), today))
                .unwrap()
                .remove(0)
        };
        set_current(Locale::En);
        let s = first(&api);
        assert_eq!(s.kind, SuggestionKind::Onboarding);
        assert_eq!(s.title, "Add the family's accounts");
        assert!(
            s.prompt.starts_with("I want to record where the family"),
            "{}",
            s.prompt
        );
        set_current(Locale::PtBr);
        let s = first(&api);
        assert_eq!(s.title, "Cadastrar as contas da família");
        assert!(s.prompt.ends_with("Não invente saldos: pergunte."));
    }

    /// Aplicação de liquidez diária fora das metas: sugere ligar à reserva; ligada, a
    /// sugestão some e a reserva (e a saúde financeira) passa a contar com ela.
    #[test]
    fn daily_liquidity_outside_goals_suggests_the_emergency_reserve() {
        let (_d, api) = vault();
        let acc = api
            .call(
                "accounts.create",
                json!({"name": "Banco Exemplo", "kind": "checking", "opening_date": "2026-01-01"}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        let rdb = api
            .call(
                "assets.create",
                json!({"name": "RDB Banco Exemplo 100% CDI", "class": "fixed_income",
                       "account_id": acc,
                       "fixed_income": {"kind": "rdb", "indexer": "cdi", "rate": "100",
                                        "issue_date": "2026-09-01", "liquidity": "daily",
                                        "ir_exempt": false}}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        api.call(
            "portfolio.create_trade",
            json!({"asset_id": rdb, "account_id": acc, "date": "2026-09-01", "side": "buy",
                   "quantity": "2000", "price": "1", "cash_movement": false}),
        )
        .unwrap();
        assert!(kinds(&api, "orcamento", "2026-10-14").contains(&SuggestionKind::ReserveLiquidity));
        let goal = api
            .call(
                "goals.create",
                json!({"name": "Reserva", "kind": "emergency", "target_amount": 3000000}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        api.call(
            "goals.holdings_set",
            json!({"goal_id": goal, "holdings": [{"account_id": acc, "asset_id": rdb, "share": 1.0}]}),
        )
        .unwrap();
        assert!(
            !kinds(&api, "orcamento", "2026-10-14").contains(&SuggestionKind::ReserveLiquidity)
        );
        let er = api.call("goals.emergency_reserve", json!({})).unwrap();
        assert!(er["current_amount"].as_i64().unwrap() >= 200_000, "{er}");
        assert_eq!(er["goal_id"], goal.as_str());
    }
}
