//! Regras do lembrete de apoio com datas fictícias, os canais embutidos e os comandos.

use chrono::NaiveDate;
use serde_json::json;

use super::*;
use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;

fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

/// Instalado em 01/01, usado `days` dias distintos.
fn used(days: u32) -> SupportState {
    SupportState {
        installed_at: Some(d("2026-01-01")),
        active_days: days,
        ..Default::default()
    }
}

const CALM: Quiet = Quiet {
    severe: false,
    view_as: false,
    demo: false,
};

#[test]
fn first_time_needs_30_days_and_10_days_of_use() {
    assert_eq!(decide(&used(10), d("2026-01-30"), CALM), None, "29 dias");
    assert_eq!(
        decide(&used(9), d("2026-02-15"), CALM),
        None,
        "9 dias de uso"
    );
    assert_eq!(
        decide(&used(10), d("2026-01-31"), CALM),
        Some(SupportVariant::Ask)
    );
    assert_eq!(
        decide(&SupportState::default(), d("2026-06-01"), CALM),
        None,
        "sem instalação"
    );
}

#[test]
fn interval_fatigue_and_yearly_cap() {
    let mut s = used(20);
    s.pending_until = Some(d("2026-03-01"));
    respond(&mut s, d("2026-03-01"), SupportChoice::Later);
    assert_eq!((s.dismiss_streak, s.shown_this_year), (1, 1));
    // 30 dias e 8 dias de uso depois da última.
    s.active_days = 27;
    assert_eq!(decide(&s, d("2026-03-31"), CALM), None, "7 dias de uso");
    s.active_days = 28;
    assert_eq!(decide(&s, d("2026-03-30"), CALM), None, "29 dias");
    assert_eq!(decide(&s, d("2026-03-31"), CALM), Some(SupportVariant::Ask));
    // Três "Agora não" seguidos: 90 dias.
    s.dismiss_streak = 3;
    assert_eq!(decide(&s, d("2026-05-29"), CALM), None);
    assert_eq!(decide(&s, d("2026-05-30"), CALM), Some(SupportVariant::Ask));
    // No máximo 6 por ano (e o contador zera no ano novo).
    s.dismiss_streak = 0;
    s.shown_this_year = 6;
    s.shown_year = Some(2026);
    assert_eq!(decide(&s, d("2026-12-01"), CALM), None);
    let mut t = s.clone();
    t.last_shown_at = Some(d("2026-12-01"));
    t.active_days_at_last = 0;
    assert_eq!(decide(&t, d("2027-01-15"), CALM), Some(SupportVariant::Ask));
    mark_shown(&mut t, d("2027-01-15"));
    assert_eq!((t.shown_year, t.shown_this_year), (Some(2027), 1));
}

#[test]
fn quiet_states_and_choices() {
    let s = used(30);
    let day = d("2026-06-01");
    for q in [
        Quiet {
            severe: true,
            ..CALM
        },
        Quiet {
            view_as: true,
            ..CALM
        },
        Quiet { demo: true, ..CALM },
    ] {
        assert_eq!(decide(&s, day, q), None, "{q:?}");
    }
    // "Já apoio": 12 meses de silêncio, depois o agradecimento.
    let mut a = s.clone();
    respond(&mut a, day, SupportChoice::Supporter);
    a.active_days += 50;
    assert_eq!(decide(&a, d("2027-05-31"), CALM), None);
    assert_eq!(
        decide(&a, d("2027-06-01"), CALM),
        Some(SupportVariant::Thanks)
    );
    // "Não mostrar mais": nunca, até religar.
    let mut n = s.clone();
    respond(&mut n, day, SupportChoice::Never);
    n.active_days += 100;
    assert_eq!(decide(&n, d("2028-01-01"), CALM), None);
    // "Como apoiar" zera o cansaço.
    let mut h = s.clone();
    h.dismiss_streak = 2;
    respond(&mut h, day, SupportChoice::How);
    assert_eq!(h.dismiss_streak, 0);
    // Dias de uso: um por dia, a instalação na primeira vez.
    let mut u = SupportState::default();
    assert!(touch(&mut u, day));
    assert!(!touch(&mut u, day));
    touch(&mut u, d("2026-06-02"));
    assert_eq!((u.installed_at, u.active_days), (Some(day), 2));
}

#[test]
fn the_only_donation_channel_is_github_sponsors() {
    let c = channels().unwrap();
    assert_eq!(
        c.github_sponsors_url.as_deref(),
        Some("https://github.com/sponsors/knuppe")
    );
    assert_eq!(
        c.repository_url.as_deref(),
        Some("https://github.com/knuppe/treasury")
    );
    assert_eq!(c.signature, "Knuppe");
    // Nenhum Pix nem outro canal nos dados embutidos.
    assert!(!CHANNELS_JSON.contains("pix") && !CHANNELS_JSON.contains("liberapay"));
    assert_eq!(url("http://exemplo.com"), None);
    assert_eq!(url("<<PLACEHOLDER>>"), None);
    assert_eq!(
        url("https://github.com/sponsors/x").as_deref(),
        Some("https://github.com/sponsors/x")
    );
}

#[test]
fn commands_wait_for_a_value_event_and_respect_the_ui() {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let root = paths.root.clone();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    // Sem histórico de uso suficiente: o evento não libera o cartão.
    assert_eq!(
        api.call("support.event", json!({"event": "import"}))
            .unwrap()["pending"],
        false
    );
    // Instalado há 40 dias, usado em 12: libera; "Ver como" silencia.
    let today = crate::time::today_local();
    update(&root, |s| {
        s.installed_at = Some(today - Duration::days(40));
        s.active_days = 12;
        s.last_active_day = Some(today);
    })
    .unwrap();
    assert_eq!(
        api.call("support.prompt", json!({})).unwrap()["variant"],
        serde_json::Value::Null,
        "sem evento, sem cartão"
    );
    assert_eq!(
        api.call("support.event", json!({"event": "import", "view_as": true}))
            .unwrap()["pending"],
        false
    );
    assert_eq!(
        api.call("support.event", json!({"event": "import"}))
            .unwrap()["pending"],
        true
    );
    assert_eq!(
        api.call("support.prompt", json!({})).unwrap()["variant"],
        "ask"
    );
    assert_eq!(
        api.call("support.prompt", json!({"view_as": true}))
            .unwrap()["variant"],
        serde_json::Value::Null
    );
    api.call("support.seen", json!({})).unwrap();
    api.call("support.seen", json!({})).unwrap();
    assert_eq!(load(&root).shown_this_year, 1, "conta uma vez");
    api.call("support.respond", json!({"choice": "never"}))
        .unwrap();
    assert!(load(&root).opted_out);
    assert_eq!(
        api.call("support.prompt", json!({})).unwrap()["variant"],
        serde_json::Value::Null
    );
    api.call("support.set_opted_out", json!({"opted_out": false}))
        .unwrap();
    assert!(!load(&root).opted_out);
    // O agente lê os canais, mas não mexe no lembrete.
    let mcp = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    assert!(
        api.call_as(&mcp, "support.respond", json!({"choice": "never"}))
            .is_err()
    );
    assert!(
        api.call("support.event", json!({"event": "outro"}))
            .is_err()
    );
    // Nada de dado financeiro no arquivo de estado.
    let raw = std::fs::read_to_string(root.join(STATE_FILE)).unwrap();
    assert!(!raw.contains("amount") && !raw.contains("saldo"));
}

#[test]
fn severe_alerts_silence() {
    let dir = tempfile::tempdir().unwrap();
    let mut db =
        crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST).unwrap();
    let today = d("2026-10-19");
    assert!(!severe(db.conn(), today));
    crate::changes::record(db.conn_mut(), &crate::changes::Author::system("t"), "seed", |tx| {
        tx.execute(
            "INSERT INTO alert (id, kind, severity, title, source, created_at, updated_at)
             VALUES ('a', 'negative_balance', 'warning', 'Saldo negativo', 't', '2026-10-15T00:00:00Z', 'x')",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    assert!(severe(db.conn(), today));
}
