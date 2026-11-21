//! Alerta "dados dos ativos para completar": um por episódio. O episódio abre quando as
//! fontes do cofre têm sugestões para campos vazios, conflitos ou atualizações
//! ([`super::enrich::data_gaps`]) e fecha quando não sobra nada (o alerta aberto vira
//! resolvido); o próximo episódio pode alertar de novo. Conferido no `tick` do app a cada
//! [`EVERY_HOURS`] e logo depois de uma consulta às fontes.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::ai::alerts::{self, AlertSeverity, AlertStatus, AlertStatusArgs, Msg, NewAlert};
use crate::error::Result;
use crate::time::now_utc;

pub const KIND: &str = "asset_data_gaps";
const SETTING: &str = "assets.data_alert";
pub const EVERY_HOURS: i64 = 6;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct State {
    /// Número do episódio atual (a chave de deduplicação do alerta).
    episode: i64,
    /// Alerta do episódio aberto, se houver.
    alert_id: Option<String>,
    last_check: Option<String>,
}

fn due(s: &State) -> bool {
    s.last_check
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .is_none_or(|t| {
            (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() >= EVERY_HOURS
        })
}

/// Confere se já passou da hora; banco só de leitura ou sem ativos: nada.
pub fn check_due(conn: &Connection) -> Result<Option<String>> {
    let s: State = crate::settings::get_or(conn, SETTING, State::default())?;
    if !due(&s) {
        return Ok(None);
    }
    check(conn)
}

/// Confere agora: abre o episódio (e cria o alerta) ou fecha. Devolve o alerta criado.
pub fn check(conn: &Connection) -> Result<Option<String>> {
    let mut s: State = crate::settings::get_or(conn, SETTING, State::default())?;
    s.last_check = Some(now_utc());
    let g = super::enrich::data_gaps(conn, &super::enrich::DataGapsArgs::default())?;
    let sum = &g.summary;
    let pending = sum.fill + sum.conflicts + sum.updates;
    let mut created = None;
    if pending == 0 {
        if let Some(id) = s.alert_id.take() {
            alerts::set_status(
                conn,
                &AlertStatusArgs {
                    ids: vec![id],
                    status: AlertStatus::Resolved,
                },
            )
            .ok();
            s.episode += 1;
        }
    } else if s.alert_id.is_none() {
        // Exemplos: até três tickers com sugestão de setor (o caso mais comum).
        let examples: Vec<String> = g
            .assets
            .iter()
            .filter(|a| a.fields.iter().any(|f| f.field == "sector"))
            .chain(g.assets.iter())
            .filter_map(|a| a.ticker.clone().or(Some(a.name.clone())))
            .fold(vec![], |mut v, t| {
                if !v.contains(&t) && v.len() < 3 {
                    v.push(t);
                }
                v
            });
        let assets = g.assets.len() as i64;
        created = alerts::create(
            conn,
            &NewAlert {
                kind: KIND,
                severity: AlertSeverity::Info,
                title: Msg::new("alertas-dados-ativos-titulo").arg("n", pending),
                body: vec![
                    Msg::new("alertas-dados-ativos-corpo")
                        .arg("campos", pending)
                        .arg("ativos", assets)
                        .arg("exemplos", examples.join(", "))
                        .into(),
                    "\n\n".into(),
                    Msg::new("alertas-dados-ativos-onde").into(),
                ],
                source: "dados dos ativos",
                dedupe_key: Some(format!("{KIND}:{}", s.episode)),
                ref_table: None,
                ref_id: None,
                data: json!({
                    "fill": sum.fill, "fill_high": sum.fill_high, "conflicts": sum.conflicts,
                    "updates": sum.updates, "missing_sector": sum.missing_sector, "assets": assets,
                }),
            },
        )?;
        // Já existia (mesma chave): o episódio continua com o alerta antigo.
        s.alert_id = created.clone().or(Some(String::new()));
    }
    crate::settings::set(conn, SETTING, &s)?;
    Ok(created)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    #[test]
    fn one_alert_per_episode() {
        let dir = tempfile::tempdir().unwrap();
        let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
        let api = Api::new_unlocked_process(paths).unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        let run = |api: &Api| api.with_session(|s, _| super::check(s.db.conn())).unwrap();
        // Nothing to fill in: no alert.
        assert!(run(&api).is_none());
        api.with_session(|s, _| {
            Ok(s.db.conn().execute_batch(
                "INSERT INTO research_issuer (id, kind, name, cvm_code, sector, layout, currency, source, fetched_at)
                 VALUES ('07859971000130', 'company_br', 'TAESA', '20257', 'Energia Elétrica', 'industrial', 'BRL',
                         'cvm_fca', '2026-10-01T00:00:00Z');
                 INSERT INTO research_security (ticker, issuer_id, class, isin, active, source, fetched_at)
                 VALUES ('TAEE11', '07859971000130', 'stock_br', 'BRTAEECDAM10', 1, 'cvm_fca', '2026-10-01T00:00:00Z');",
            )?)
        })
        .unwrap();
        let id = api
            .call(
                "assets.create",
                json!({"ticker": "TAEE11", "name": "Taesa", "class": "stock_br"}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        let alert = run(&api).expect("opens the episode");
        // The same episode does not alert again.
        assert!(run(&api).is_none());
        let get = |api: &Api| {
            let l = api.call("ai.alerts.list", json!({})).unwrap();
            l.as_array()
                .unwrap()
                .iter()
                .find(|a| a["id"] == alert.as_str())
                .cloned()
                .unwrap()
        };
        let a = get(&api);
        assert_eq!(a["kind"], "asset_data_gaps");
        assert!(a["title"].as_str().unwrap().contains("completar"), "{a}");
        assert!(a["body"].as_str().unwrap().contains("TAEE11"), "{a}");
        assert_eq!(a["target"]["screen"], "asset_data");
        // Everything filled in: the alert is resolved and the episode closes.
        let g = api
            .call("assets.data_gaps", json!({"asset_ids": [id]}))
            .unwrap();
        let items: Vec<_> = g["assets"][0]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["state"] == "fill")
            .map(|f| json!({"asset_id": id, "field": f["field"], "value": f["candidate"]["value"]}))
            .collect();
        api.call("assets.enrich_apply", json!({ "items": items }))
            .unwrap();
        assert!(run(&api).is_none());
        let resolved = api
            .call("ai.alerts.list", json!({"status": "resolved"}))
            .unwrap();
        assert!(
            resolved
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["id"] == alert.as_str())
        );
    }
}
