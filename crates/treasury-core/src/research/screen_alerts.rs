//! Alertas de tela salva (docs/design/market-explorer.md, seção 7.5): a tela com
//! `alert` ligado avisa quando um papel novo passa a cumprir os filtros. Um alerta por
//! episódio: o papel entrou; de novo só se ele sair e voltar depois de 30 dias. Quem sai só
//! gera aviso se estiver na carteira ("PETR4 deixou de cumprir a sua tela").
//!
//! A primeira passada de uma tela só registra quem já cumpre (linha de base, sem alerta).
//! Roda na coleta, depois do universo e do score do dia.

use std::collections::HashSet;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};

use super::indicators::BazinParams;
use super::screen::{self, ScreenArgs};
use crate::ai::alerts::{AlertSeverity, Msg, NewAlert, create};
use crate::error::Result;
use crate::time::fmt_date;

/// Dias fora da tela para um papel que volta gerar outro alerta.
const REENTRY_DAYS: i64 = 30;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Report {
    pub screens: u32,
    pub entered: u32,
    pub left_held: u32,
}

/// Confere as telas com alerta ligado e cria os alertas de quem entrou (e de quem saiu
/// estando na carteira).
pub fn run(conn: &Connection, today: NaiveDate, bazin: BazinParams) -> Result<Report> {
    let mut rep = Report::default();
    let screens: Vec<screen::Screen> = screen::list(conn)?
        .into_iter()
        .filter(|s| s.alert)
        .collect();
    if screens.is_empty() {
        return Ok(rep);
    }
    let (rows, as_of) = screen::universe(conn, today, bazin)?;
    let held: HashSet<String> = conn
        .prepare(
            "SELECT DISTINCT a.ticker FROM asset a JOIN trade t ON t.asset_id = a.id
              WHERE a.ticker IS NOT NULL",
        )?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<_>>()?;
    let day = fmt_date(today);
    for s in screens {
        rep.screens += 1;
        let result = screen::run_screen(
            conn,
            &rows,
            &as_of,
            &ScreenArgs {
                filters: Some(s.filters.clone()),
                limit: Some(500),
                ..Default::default()
            },
        )?;
        let now: HashSet<String> = result.items.iter().map(|i| i.ticker.clone()).collect();
        let baseline: bool = !conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM screen_match WHERE screen_id = ?1)",
            [&s.id],
            |r| r.get(0),
        )?;
        for t in &now {
            let prev: Option<(String, Option<String>)> = conn
                .query_row(
                    "SELECT first_seen, left_at FROM screen_match WHERE screen_id = ?1 AND ticker = ?2",
                    params![s.id, t],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let new_episode = match &prev {
                None => true,
                Some((_, Some(left))) => crate::time::parse_date(left)
                    .map(|l| (today - l).num_days() >= REENTRY_DAYS)
                    .unwrap_or(true),
                Some((_, None)) => false,
            };
            let first_seen = match (&prev, new_episode) {
                (Some((f, _)), false) => f.clone(),
                _ => day.clone(),
            };
            conn.execute(
                "INSERT INTO screen_match (screen_id, ticker, first_seen, last_seen, left_at)
                 VALUES (?1, ?2, ?3, ?4, NULL)
                 ON CONFLICT (screen_id, ticker) DO UPDATE SET first_seen = excluded.first_seen,
                     last_seen = excluded.last_seen, left_at = NULL",
                params![s.id, t, first_seen, day],
            )?;
            if new_episode && !baseline {
                let created = create(
                    conn,
                    &NewAlert {
                        kind: "screen_match",
                        severity: AlertSeverity::Info,
                        title: Msg::new("alertas-tela-entrou-titulo")
                            .arg("ticker", t.as_str())
                            .arg("tela", s.name.as_str()),
                        body: vec![
                            Msg::new("alertas-tela-entrou-corpo")
                                .arg("ticker", t.as_str())
                                .arg("tela", s.name.as_str())
                                .into(),
                        ],
                        source: "research",
                        dedupe_key: Some(format!("screen:{}:{t}:{first_seen}", s.id)),
                        ref_table: Some("screen"),
                        ref_id: Some(s.id.clone()),
                        data: serde_json::json!({"screen_id": s.id, "ticker": t}),
                    },
                )?;
                if created.is_some() {
                    rep.entered += 1;
                }
            }
        }
        // Quem saiu: marca a saída; na carteira, avisa.
        let gone: Vec<String> = conn
            .prepare("SELECT ticker FROM screen_match WHERE screen_id = ?1 AND left_at IS NULL")?
            .query_map([&s.id], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .filter(|t| !now.contains(t))
            .collect();
        for t in gone {
            conn.execute(
                "UPDATE screen_match SET left_at = ?3 WHERE screen_id = ?1 AND ticker = ?2",
                params![s.id, t, day],
            )?;
            if held.contains(&t) {
                let created = create(
                    conn,
                    &NewAlert {
                        kind: "screen_left",
                        severity: AlertSeverity::Info,
                        title: Msg::new("alertas-tela-saiu-titulo")
                            .arg("ticker", t.as_str())
                            .arg("tela", s.name.as_str()),
                        body: vec![
                            Msg::new("alertas-tela-saiu-corpo")
                                .arg("ticker", t.as_str())
                                .arg("tela", s.name.as_str())
                                .into(),
                        ],
                        source: "research",
                        dedupe_key: Some(format!("screen-left:{}:{t}:{day}", s.id)),
                        ref_table: Some("screen"),
                        ref_id: Some(s.id.clone()),
                        data: serde_json::json!({"screen_id": s.id, "ticker": t}),
                    },
                )?;
                if created.is_some() {
                    rep.left_held += 1;
                }
            }
        }
    }
    Ok(rep)
}
