//! Extrato de um ativo: operações, eventos corporativos (desdobramento, grupamento,
//! bonificação, incorporação, troca de ticker, devolução de capital) e posições herdadas, na
//! ordem em que o motor da carteira os aplica (eventos de uma data antes das operações da
//! mesma data), com a quantidade e o preço médio **depois** de cada linha e a origem de cada
//! uma (nota, importação, agente, coleta do Yahoo, manual).
//!
//! Também aponta o erro mais comum de cadastro: o mesmo desdobramento contado duas vezes
//! (uma entrada a custo zero nas operações e o evento corporativo perto da mesma data).

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{load_book, replay};
use super::{CorporateActionKind, TradeFilter, TradeSide};
use crate::error::Result;
use crate::i18n::{self, t_args};
use crate::ids::Id;
use crate::time::fmt_date;

// Nome próprio no TypeScript: o `StatementArgs` das dívidas usa o mesmo nome em Rust.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetStatementArgs")]
pub struct StatementArgs {
    pub asset_id: Id,
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementLine {
    pub date: String,
    /// `buy`, `sell`, `split`, `reverse_split`, `bonus`, `subscription`, `merger`,
    /// `ticker_change`, `spin_off`, `capital_return`, `come_cotas` ou `inherited`.
    pub kind: String,
    /// Descrição em português ("Desdobramento 1:5", "Compra").
    pub label: String,
    /// Variação da quantidade nesta linha (com sinal).
    pub quantity_change: String,
    /// Preço unitário (operações).
    pub price: Option<String>,
    pub quantity_after: String,
    /// Preço médio depois da linha (moeda do ativo).
    pub average_price_after: Option<String>,
    pub account_name: Option<String>,
    /// `nota 12345`, `importação`, `agente`, `coleta do Yahoo`, `manual`.
    pub origin: String,
    pub trade_id: Option<Id>,
    pub action_id: Option<Id>,
    pub notes: Option<String>,
    /// Aviso desta linha (ex.: desdobramento possivelmente contado duas vezes).
    pub warning: Option<String>,
}

fn kind_name(k: CorporateActionKind) -> &'static str {
    match k {
        CorporateActionKind::Split => "split",
        CorporateActionKind::ReverseSplit => "reverse_split",
        CorporateActionKind::Bonus => "bonus",
        CorporateActionKind::Subscription => "subscription",
        CorporateActionKind::Merger => "merger",
        CorporateActionKind::TickerChange => "ticker_change",
        CorporateActionKind::SpinOff => "spin_off",
        CorporateActionKind::CapitalReturn => "capital_return",
    }
}

fn ratio(f: Decimal) -> String {
    // 5 → "1:5"; 0,1 → "10:1".
    if f >= Decimal::ONE {
        format!("1:{}", f.normalize())
    } else if !f.is_zero() {
        format!("{}:1", (Decimal::ONE / f).round_dp(4).normalize())
    } else {
        "0".into()
    }
}

fn action_label(k: CorporateActionKind, factor: Option<Decimal>) -> String {
    let f = factor.map(ratio).unwrap_or_default();
    let with =
        |key: &str, name: &'static str, v: String| t_args(key, &i18n::args(&[(name, v.into())]));
    match k {
        CorporateActionKind::Split => with("nucleo-carteira-extrato-desdobramento", "proporcao", f),
        CorporateActionKind::ReverseSplit => {
            with("nucleo-carteira-extrato-grupamento", "proporcao", f)
        }
        CorporateActionKind::Bonus => match factor {
            Some(x) => with(
                "nucleo-carteira-extrato-bonificacao-pct",
                "pct",
                super::records::fmt_dec(x * Decimal::from(100)),
            ),
            None => i18n::t("nucleo-carteira-extrato-bonificacao"),
        },
        CorporateActionKind::Subscription => i18n::t("nucleo-carteira-extrato-subscricao"),
        CorporateActionKind::Merger => i18n::t("nucleo-carteira-extrato-incorporacao"),
        CorporateActionKind::TickerChange => i18n::t("nucleo-carteira-extrato-troca-ticker"),
        CorporateActionKind::SpinOff => i18n::t("nucleo-carteira-extrato-cisao"),
        CorporateActionKind::CapitalReturn => i18n::t("nucleo-carteira-extrato-devolucao"),
    }
    .trim()
    .to_string()
}

/// Começos das notas que o lote do agente grava nas operações, em todos os idiomas (a nota fica
/// no idioma da hora em que foi gravada).
fn agent_note_prefixes() -> Vec<String> {
    const MARK: &str = "\u{1}";
    let blank = i18n::args(&[
        ("fonte", MARK.into()),
        ("data", MARK.into()),
        ("diferenca", MARK.into()),
    ]);
    let mut out: Vec<String> = vec![];
    for l in i18n::AVAILABLE {
        for k in ["nucleo-carteira-nota-lote", "nucleo-carteira-nota-posicao"] {
            let full = i18n::tr(l, k, Some(&blank));
            let prefix = full.split(MARK).next().unwrap_or_default().trim_end();
            if !prefix.is_empty() && !out.iter().any(|p| p == prefix) {
                out.push(prefix.to_string());
            }
        }
    }
    out
}

fn trade_origin(tr: &super::Trade, agent: &[String]) -> String {
    if let Some(n) = tr.note_number.as_deref().filter(|n| !n.is_empty()) {
        return t_args(
            "nucleo-carteira-extrato-origem-nota",
            &i18n::args(&[("numero", n.to_string().into())]),
        );
    }
    let notes = tr.notes.as_deref().unwrap_or_default();
    if agent.iter().any(|p| notes.starts_with(p.as_str())) {
        return i18n::t("nucleo-carteira-extrato-origem-agente");
    }
    if tr.import_id.is_some() {
        return i18n::t("nucleo-carteira-extrato-origem-importacao");
    }
    i18n::t("nucleo-carteira-extrato-origem-manual")
}

fn action_origin(source: &str) -> String {
    match source {
        "yahoo" => i18n::t("nucleo-carteira-extrato-origem-yahoo"),
        "tiingo" | "eodhd" => i18n::t("nucleo-carteira-extrato-origem-coleta"),
        "user" | "manual" => i18n::t("nucleo-carteira-extrato-origem-manual"),
        "b3" => i18n::t("nucleo-carteira-extrato-origem-b3"),
        "ai" | "agent" => i18n::t("nucleo-carteira-extrato-origem-agente"),
        other => other.to_string(),
    }
}

fn days_apart(a: NaiveDate, b: NaiveDate) -> i64 {
    (a - b).num_days().abs()
}

pub fn statement(conn: &Connection, args: &StatementArgs) -> Result<Vec<StatementLine>> {
    let accounts = args.account_id.clone().map(|a| vec![a]);
    let book = load_book(conn, &args.asset_id, accounts.as_deref())?;
    let views = super::records::list_trades(
        conn,
        &TradeFilter {
            asset_id: Some(args.asset_id.clone()),
            account_id: args.account_id.clone(),
            date_from: None,
            date_to: None,
            limit: Some(5000),
            offset: None,
        },
    )?;
    let by_id: std::collections::HashMap<&str, &super::Trade> =
        views.iter().map(|t| (t.id.as_str(), t)).collect();
    let actions = super::records::list_actions(conn, Some(&args.asset_id))?;
    let zero_buys: Vec<NaiveDate> = book
        .trades
        .iter()
        .filter(|t| t.side == TradeSide::Buy && t.price.is_zero())
        .map(|t| t.date)
        .collect();
    let split_dates: Vec<NaiveDate> = book
        .actions
        .iter()
        .filter(|a| {
            matches!(
                a.kind,
                CorporateActionKind::Split | CorporateActionKind::Bonus
            )
        })
        .map(|a| a.date)
        .collect();
    let double = i18n::t("nucleo-carteira-extrato-aviso-desdobramento-duplo");
    let agent = agent_note_prefixes();

    let mut out = vec![];
    let (mut ti, mut ai, mut ii) = (0usize, 0usize, 0usize);
    let mut prev_qty = Decimal::ZERO;
    loop {
        let t = book.trades.get(ti);
        let a = book.actions.get(ai);
        let i = book.inherited.get(ii);
        let next = [t.map(|x| x.date), a.map(|x| x.date)]
            .into_iter()
            .flatten()
            .min();
        let (date, line) = if let Some(inh) = i.filter(|inh| next.is_none_or(|d| inh.date <= d)) {
            ii += 1;
            (inh.date, None)
        } else {
            match (t, a) {
                (None, None) => break,
                (Some(_), None) => (book.trades[ti].date, Some(true)),
                (None, Some(_)) => (book.actions[ai].date, Some(false)),
                (Some(t), Some(a)) => {
                    if a.date <= t.date {
                        (a.date, Some(false))
                    } else {
                        (t.date, Some(true))
                    }
                }
            }
        };
        match line {
            Some(true) => ti += 1,
            Some(false) => ai += 1,
            None => {}
        }
        let st = replay(conn, &book.prefix(ti, ai, date), date)?;
        let qty = st.quantity();
        let avg = (!qty.is_zero()).then(|| (st.cost() / qty).round_dp(6).normalize().to_string());
        let change = (qty - prev_qty).normalize().to_string();
        prev_qty = qty;
        let mut l = StatementLine {
            date: fmt_date(date),
            kind: "inherited".into(),
            label: i18n::t("nucleo-carteira-extrato-herdada"),
            quantity_change: change,
            price: None,
            quantity_after: qty.normalize().to_string(),
            average_price_after: avg,
            account_name: None,
            origin: i18n::t("nucleo-carteira-extrato-origem-sistema"),
            trade_id: None,
            action_id: None,
            notes: None,
            warning: None,
        };
        match line {
            Some(true) => {
                let tr = &book.trades[ti - 1];
                let v = by_id.get(tr.id.as_str());
                l.kind = if tr.side == TradeSide::Buy {
                    "buy"
                } else {
                    "sell"
                }
                .into();
                l.label = i18n::t(match (tr.side, tr.price.is_zero()) {
                    (TradeSide::Buy, true) => "nucleo-carteira-extrato-entrada-sem-custo",
                    (TradeSide::Buy, false) => "nucleo-carteira-extrato-compra",
                    (TradeSide::Sell, _) => "nucleo-carteira-extrato-venda",
                });
                l.price = Some(tr.price.normalize().to_string());
                l.trade_id = Some(tr.id.clone());
                if let Some(v) = v {
                    l.account_name = Some(v.account_name.clone());
                    l.origin = trade_origin(v, &agent);
                    l.notes = v.notes.clone();
                }
                if tr.side == TradeSide::Buy
                    && tr.price.is_zero()
                    && split_dates.iter().any(|d| days_apart(*d, tr.date) <= 10)
                {
                    l.warning = Some(double.clone());
                }
            }
            Some(false) => {
                let ac = &book.actions[ai - 1];
                if let Some(shares) = ac.come_cotas {
                    l.kind = "come_cotas".into();
                    l.label = t_args(
                        "nucleo-carteira-extrato-come-cotas",
                        &i18n::args(&[(
                            "cotas",
                            super::records::fmt_dec(shares.round_dp(6)).into(),
                        )]),
                    );
                    l.origin = i18n::t("nucleo-carteira-extrato-origem-sistema");
                    out.push(l);
                    continue;
                }
                // Os eventos sintéticos (come-cotas) não estão na lista do banco.
                let real = book.actions[..ai - 1]
                    .iter()
                    .filter(|a| a.come_cotas.is_none())
                    .count();
                let view = actions.get(real);
                l.kind = kind_name(ac.kind).into();
                l.label = action_label(ac.kind, ac.factor);
                l.action_id = view.map(|v| v.id.clone());
                l.origin = view.map_or_else(
                    || i18n::t("nucleo-carteira-extrato-origem-sistema"),
                    |v| action_origin(&v.source),
                );
                l.notes = view.and_then(|v| v.notes.clone());
                if matches!(
                    ac.kind,
                    CorporateActionKind::Split | CorporateActionKind::Bonus
                ) && zero_buys.iter().any(|d| days_apart(*d, ac.date) <= 10)
                {
                    l.warning = Some(double.clone());
                }
            }
            None => {}
        }
        out.push(l);
    }
    flag_bonus_bought(&mut out);
    Ok(out)
}

/// Bonificação contada duas vezes: o evento (desdobramento ou bonificação) e, a até 10 dias,
/// uma compra com custo (o custo atribuído, como os agregadores importam) com a mesma
/// quantidade de ações novas.
fn flag_bonus_bought(out: &mut [StatementLine]) {
    let msg = i18n::t("nucleo-carteira-extrato-aviso-bonificacao-dupla");
    let num = |s: &str| s.parse::<Decimal>().ok();
    let date = |s: &str| crate::time::parse_date(s).ok();
    let events: Vec<(usize, NaiveDate, Decimal)> = out
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.kind.as_str(), "split" | "bonus"))
        .filter_map(|(i, l)| Some((i, date(&l.date)?, num(&l.quantity_change)?)))
        .filter(|(_, _, q)| *q > Decimal::ZERO)
        .collect();
    let mut hits = vec![];
    for (i, l) in out.iter().enumerate() {
        if l.kind != "buy" || l.price.as_deref().and_then(num).is_none_or(|p| p.is_zero()) {
            continue;
        }
        let (Some(d), Some(q)) = (date(&l.date), num(&l.quantity_change)) else {
            continue;
        };
        for (j, ed, eq) in &events {
            let tol = (*eq / Decimal::from(100)).max(Decimal::ONE);
            if days_apart(*ed, d) <= 10 && (q - *eq).abs() <= tol {
                hits.push(i);
                hits.push(*j);
            }
        }
    }
    for i in hits {
        if out[i].warning.is_none() {
            out[i].warning = Some(msg.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    #[test]
    fn statement_interleaves_events_with_the_running_position() {
        let dir = tempfile::tempdir().unwrap();
        let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "C"}),
        )
        .unwrap();
        let acc = api
            .call(
                "accounts.create",
                json!({"name": "Nu", "kind": "brokerage"}),
            )
            .unwrap()["id"]
            .clone();
        let a = api
            .call(
                "assets.create",
                json!({"ticker": "TSLA34", "name": "Tesla BDR", "class": "bdr"}),
            )
            .unwrap()["id"]
            .clone();
        let trade = |date: &str, side: &str, q: &str, p: &str| {
            api.call(
                "portfolio.create_trade",
                json!({"asset_id": a, "account_id": acc, "date": date, "side": side,
                       "quantity": q, "price": p, "cash_movement": false}),
            )
            .unwrap();
        };
        trade("2022-06-02", "buy", "100", "120");
        // Split entered as a zero-cost buy AND as an event: counted twice.
        trade("2022-08-24", "buy", "200", "0");
        api.call(
            "portfolio.create_action",
            json!({"asset_id": a, "date": "2022-08-25", "kind": "split", "factor": "3"}),
        )
        .unwrap();
        trade("2023-01-10", "sell", "50", "100");
        let lines = api
            .call("portfolio.statement", json!({"asset_id": a}))
            .unwrap();
        let l = lines.as_array().unwrap();
        let cols: Vec<(String, String, String)> = l
            .iter()
            .map(|x| {
                (
                    x["kind"].as_str().unwrap().into(),
                    x["quantity_change"].as_str().unwrap().into(),
                    x["quantity_after"].as_str().unwrap().into(),
                )
            })
            .collect();
        assert_eq!(
            cols,
            vec![
                ("buy".into(), "100".into(), "100".into()),
                ("buy".into(), "200".into(), "300".into()),
                ("split".into(), "600".into(), "900".into()),
                ("sell".into(), "-50".into(), "850".into()),
            ]
        );
        assert_eq!(l[2]["label"], "Desdobramento 1:3");
        assert!(l[1]["warning"].as_str().unwrap().contains("duas vezes"));
        assert!(l[2]["warning"].is_string());
        assert_eq!(l[0]["average_price_after"], "120");
        assert_eq!(l[2]["origin"], "manual");
        assert!(l[3]["warning"].is_null());
    }

    #[test]
    fn bonus_event_and_a_priced_buy_of_the_new_shares_are_flagged() {
        let dir = tempfile::tempdir().unwrap();
        let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "C"}),
        )
        .unwrap();
        let acc = api
            .call(
                "accounts.create",
                json!({"name": "XP", "kind": "brokerage"}),
            )
            .unwrap()["id"]
            .clone();
        let a = api
            .call(
                "assets.create",
                json!({"ticker": "ITSA4", "name": "Itaúsa", "class": "stock_br"}),
            )
            .unwrap()["id"]
            .clone();
        let trade = |date: &str, q: &str, p: &str| {
            api.call(
                "portfolio.create_trade",
                json!({"asset_id": a, "account_id": acc, "date": date, "side": "buy",
                       "quantity": q, "price": p, "cash_movement": false}),
            )
            .unwrap();
        };
        trade("2024-01-02", "1000", "10");
        api.call(
            "portfolio.create_action",
            json!({"asset_id": a, "date": "2024-12-23", "kind": "split", "factor": "1.1"}),
        )
        .unwrap();
        // The same 100 new shares, imported as a buy with the attributed cost.
        trade("2024-12-27", "100", "18.45");
        trade("2025-03-10", "100", "9");
        let lines = api
            .call("portfolio.statement", json!({"asset_id": a}))
            .unwrap();
        let w: Vec<bool> = lines
            .as_array()
            .unwrap()
            .iter()
            .map(|x| {
                x["warning"]
                    .as_str()
                    .is_some_and(|w| w.contains("bonificação contada duas vezes"))
            })
            .collect();
        assert_eq!(w, vec![false, true, true, false], "{lines}");
    }
}
