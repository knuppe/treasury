//! Asset data through the dispatcher: suggestions from the vault's sources, confidence,
//! conflicts, protection of the user's value, undo, lock and the AI policy.

use serde_json::{Value, json};

use super::*;
use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    (dir, api)
}

fn sql(api: &Api, s: &str) {
    api.with_session(|s2, _| Ok(s2.db.conn().execute_batch(s)?))
        .unwrap();
}

/// Taesa (CVM's FCA) with the code TAEE11 and Fundamentus in cache.
fn seed_taesa(api: &Api, fundamentus_sector: &str) {
    let today = crate::time::fmt_date(crate::time::today_local());
    sql(
        api,
        &format!(
            "INSERT INTO research_issuer (id, kind, name, cvm_code, sector, layout, currency, source, fetched_at)
             VALUES ('07859971000130', 'company_br', 'TRANSMISSORA ALIANÇA DE ENERGIA ELÉTRICA S.A.', '20257',
                     'Energia Elétrica', 'industrial', 'BRL', 'cvm_fca', '{today}T10:00:00Z');
             INSERT INTO research_security (ticker, issuer_id, class, share_kind, isin, listing_segment, active, source, fetched_at)
             VALUES ('TAEE11', '07859971000130', 'stock_br', 'UNIT', 'BRTAEECDAM10', 'Nível 2', 1, 'cvm_fca', '{today}T10:00:00Z');
             INSERT INTO fundamentus_cache (ticker, page, fetched_at, found, payload) VALUES ('TAEE11', 'details',
               '{today}T10:00:00Z', 1, '{payload}');",
            payload = serde_json::to_string(&json!({
                "ticker": "TAEE11", "fii": false, "found": true, "name": "TAESA",
                "sector": fundamentus_sector, "subsector": "Energia Elétrica", "price": null,
                "price_date": null, "indicators": [], "oscillations": [], "dividends": [],
                "fetched_at": format!("{today}T10:00:00Z"), "from_cache": false,
                "source_url": "https://www.fundamentus.com.br/detalhes.php?papel=TAEE11"
            }))
            .unwrap()
        ),
    );
}

fn create_asset(api: &Api, extra: Value) -> String {
    let mut a = json!({"ticker": "TAEE11", "name": "Taesa", "class": "stock_br"});
    a.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    api.call("assets.create", a).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn gaps(api: &Api, id: &str) -> AssetData {
    let v = api
        .call(
            "assets.data_gaps",
            json!({"asset_ids": [id], "include_ok": true}),
        )
        .unwrap();
    let g: DataGaps = serde_json::from_value(v).unwrap();
    g.assets.into_iter().next().unwrap()
}

fn field<'a>(d: &'a AssetData, f: &str) -> &'a FieldStatus {
    d.fields.iter().find(|x| x.field == f).unwrap()
}

#[test]
fn check_digits() {
    assert!(valid_cnpj("33.000.167/0001-01"));
    assert!(valid_cnpj("07859971000130"));
    assert!(!valid_cnpj("33.000.167/0001-02"));
    assert!(!valid_cnpj("11111111111111"));
    assert!(valid_isin("BRTAEECDAM10"));
    assert!(valid_isin("US0378331005"));
    assert!(!valid_isin("US0378331006"));
    assert!(!valid_isin("BRTAEE"));
}

#[test]
fn fills_empty_fields_from_the_vault_and_undoes() {
    let (_d, api) = api();
    seed_taesa(&api, "Energia Elétrica");
    let id = create_asset(&api, json!({}));
    let d = gaps(&api, &id);
    let sector = field(&d, "sector");
    assert_eq!(sector.state, FieldState::Fill);
    let c = sector.candidate.as_ref().unwrap();
    assert_eq!(c.value, "Energia Elétrica");
    assert_eq!(c.source, "cvm_fca");
    // Official (90) + Fundamentus agreeing (+5).
    assert_eq!(c.confidence, 95);
    assert_eq!(c.band, FieldBand::High);
    assert_eq!(c.agreeing, vec!["fundamentus".to_string()]);
    assert_eq!(
        field(&d, "isin").candidate.as_ref().unwrap().value,
        "BRTAEECDAM10"
    );
    assert_eq!(
        field(&d, "issuer_cnpj").candidate.as_ref().unwrap().value,
        "07.859.971/0001-30"
    );
    assert_eq!(
        field(&d, "share_kind").candidate.as_ref().unwrap().value,
        "UNIT"
    );
    let sub = field(&d, "subsector").candidate.as_ref().unwrap();
    assert_eq!((sub.source.as_str(), sub.confidence), ("fundamentus", 60));

    let r = api
        .call(
            "assets.enrich_apply",
            json!({"items": [
                {"asset_id": id, "field": "sector", "value": "energia eletrica"},
                {"asset_id": id, "field": "listing_segment", "value": "Nível 2"},
            ]}),
        )
        .unwrap();
    assert_eq!(r["fields"], 2);
    let a = api.call("assets.get", json!({"id": id})).unwrap();
    // Stores the source's value, not the one typed in the request.
    assert_eq!(a["sector"], "Energia Elétrica");
    let src: AssetFieldSources = serde_json::from_value(
        api.call("assets.field_sources", json!({"asset_id": id}))
            .unwrap(),
    )
    .unwrap();
    let s = src
        .fields
        .iter()
        .find(|f| f.status.field == "sector")
        .unwrap();
    assert_eq!(s.status.origin, Some(FieldOrigin::Enrichment));
    assert_eq!(s.status.current_source.as_deref(), Some("cvm_fca"));
    assert_eq!(s.status.current_confidence, Some(95));
    assert_eq!(s.status.state, FieldState::Ok);
    assert_eq!(s.history.len(), 2);
    let ls = src
        .fields
        .iter()
        .find(|f| f.status.field == "listing_segment")
        .unwrap();
    assert_eq!(ls.status.current.as_deref(), Some("Nível 2"));

    // Undo restores the value and the origin together.
    let last = api.call("changes.list", json!({})).unwrap();
    api.call("changes.undo", json!({"id": last[0]["id"]}))
        .unwrap();
    let d = gaps(&api, &id);
    assert_eq!(field(&d, "sector").state, FieldState::Fill);
    assert_eq!(field(&d, "sector").origin, None);
    assert_eq!(field(&d, "listing_segment").current, None);
}

#[test]
fn never_overwrites_the_user_and_respects_the_lock() {
    let (_d, api) = api();
    seed_taesa(&api, "Energia Elétrica");
    let id = create_asset(&api, json!({"sector": "Transmissão"}));
    let d = gaps(&api, &id);
    let s = field(&d, "sector");
    // No origin record: predates this feature, protected as the user's.
    assert_eq!(s.origin, Some(FieldOrigin::Legacy));
    assert_eq!(s.state, FieldState::Diverge);
    let item = json!({"items": [{"asset_id": id, "field": "sector", "value": "Energia Elétrica"}]});
    let e = api.call("assets.enrich_apply", item.clone()).unwrap_err();
    assert!(
        e.message().contains("já tem um valor seu"),
        "{}",
        e.message()
    );
    // Editing by hand marks the origin as the user's.
    api.call(
        "assets.update",
        json!({"id": id, "sector": "Utilidade Pública"}),
    )
    .unwrap();
    assert_eq!(
        field(&gaps(&api, &id), "sector").origin,
        Some(FieldOrigin::User)
    );
    // Saving the form without changing the field does not touch another field's origin.
    api.call(
        "assets.enrich_apply",
        json!({"items": [{"asset_id": id, "field": "isin", "value": "BRTAEECDAM10"}]}),
    )
    .unwrap();
    api.call(
        "assets.update",
        json!({"id": id, "sector": "Utilidade Pública", "isin": "BRTAEECDAM10"}),
    )
    .unwrap();
    assert_eq!(
        field(&gaps(&api, &id), "isin").origin,
        Some(FieldOrigin::Enrichment)
    );
    // An explicit request overrides.
    api.call(
        "assets.enrich_apply",
        json!({"items": [{"asset_id": id, "field": "sector", "value": "Energia Elétrica", "replace": true}]}),
    )
    .unwrap();
    // Locked: neither suggestion nor write.
    api.call(
        "assets.field_lock",
        json!({"asset_id": id, "field": "subsector", "locked": true}),
    )
    .unwrap();
    let d = gaps(&api, &id);
    assert!(field(&d, "subsector").locked);
    assert_eq!(field(&d, "subsector").state, FieldState::Ok);
    let e = api
        .call(
            "assets.enrich_apply",
            json!({"items": [{"asset_id": id, "field": "subsector", "value": "Energia Elétrica"}]}),
        )
        .unwrap_err();
    assert!(e.message().contains("travado"), "{}", e.message());
    // A value that no source reported is refused.
    let e = api
        .call(
            "assets.enrich_apply",
            json!({"items": [{"asset_id": id, "field": "cvm_code", "value": "99999"}]}),
        )
        .unwrap_err();
    assert!(
        e.message().contains("não está em nenhuma fonte"),
        "{}",
        e.message()
    );
}

#[test]
fn same_tier_disagreement_is_a_conflict() {
    let (_d, api) = api();
    seed_taesa(&api, "Utilidade Pública");
    let id = create_asset(&api, json!({}));
    // Secondary disagreeing with an official one: not a conflict, but the official gets no bonus.
    let s = field(&gaps(&api, &id), "sector").clone();
    assert_eq!(s.state, FieldState::Fill);
    assert_eq!(s.candidate.unwrap().confidence, 90);
    // Two different official readings of the same field (FII code and issuer).
    sql(
        &api,
        "INSERT INTO research_issuer (id, kind, name, segment, layout, currency, source, fetched_at)
         VALUES ('11728688000147', 'fii', 'HGLG', 'Logística', 'fund', 'BRL', 'cvm_fii', '2026-10-01T00:00:00Z');",
    );
    let fii = api
        .call("assets.create", json!({"ticker": "HGLG11", "name": "CSHG Logística", "class": "fii", "cnpj": "11.728.688/0001-47"}))
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    sql(
        &api,
        &format!(
            "INSERT INTO fundamentus_cache (ticker, page, fetched_at, found, payload) VALUES ('HGLG11', 'details',
             '2026-10-01T00:00:00Z', 1, '{}');",
            serde_json::to_string(&json!({
                "ticker": "HGLG11", "fii": true, "found": true, "name": null, "sector": "Logística",
                "subsector": "Ativa", "price": null, "price_date": null, "indicators": [],
                "oscillations": [], "dividends": [], "fetched_at": "2026-10-01T00:00:00Z",
                "from_cache": false, "source_url": "https://www.fundamentus.com.br/detalhes.php?papel=HGLG11"
            }))
            .unwrap()
        ),
    );
    let d = gaps(&api, &fii);
    let seg = field(&d, "segment");
    assert_eq!(seg.candidate.as_ref().unwrap().value, "Logística");
    assert_eq!(seg.candidate.as_ref().unwrap().confidence, 95);
    assert_eq!(
        field(&d, "fii_management")
            .candidate
            .as_ref()
            .unwrap()
            .value,
        "Ativa"
    );
    // The FII's CNPJ is already on the asset and matches the CVM: nothing to do.
    assert_eq!(field(&d, "cnpj").state, FieldState::Ok);

    // Conflict: two level-1 observations with different values.
    let def = field_def("sector").unwrap();
    let mk = |v: &str, src: &str| FieldObservation {
        field: "sector".into(),
        value: v.into(),
        value_norm: normalize("sector", v),
        scheme: None,
        source: src.into(),
        tier: 1,
        source_ref: None,
        as_of: None,
        note: None,
        penalty: 0,
    };
    let o1 = mk("Varejo", "cvm_fca");
    let o2 = mk("Comércio", "b3");
    let (ranked, conflict) = rank(&[&o1, &o2], def, crate::time::today_local());
    assert!(conflict);
    assert_eq!(ranked.len(), 2);
    assert!(ranked.iter().all(|c| c.confidence == 65));
}

#[test]
fn ai_writes_become_sensitive_proposals_and_lock_is_user_only() {
    let (_d, api) = api();
    seed_taesa(&api, "Energia Elétrica");
    let id = create_asset(&api, json!({}));
    api.call("ai.settings.set", json!({"autonomous": true}))
        .unwrap();
    let ai = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let e = api
        .call_as(
            &ai,
            "assets.enrich_apply",
            json!({"items": [{"asset_id": id, "field": "sector", "value": "Energia Elétrica"}]}),
        )
        .unwrap();
    // Even in autonomous mode: pending proposal.
    assert_eq!(e["status"], "pending_approval");
    let a = api.call("assets.get", json!({"id": id})).unwrap();
    assert_eq!(a["sector"], Value::Null);
    let p = api.call("ai.list_proposals", json!({})).unwrap();
    assert_eq!(p[0]["level"], "sensitive");
    let e = api
        .call_as(
            &ai,
            "assets.field_lock",
            json!({"asset_id": id, "field": "sector", "locked": true}),
        )
        .unwrap_err();
    assert!(matches!(e, crate::error::Error::Forbidden(_)), "{e:?}");
    // Read access allowed for read-only clients.
    let reader = Actor::Mcp {
        client: "leitor".into(),
        read_only: true,
    };
    api.call_as(&reader, "assets.data_gaps", json!({})).unwrap();
}
