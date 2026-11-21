//! Consulta às fontes da rede com respostas fixas (sem rede): B3, GLEIF e BrasilAPI viram
//! leituras, a B3 vira o esquema principal do setor com o FCA conferindo, o cache de 30 dias
//! e as fontes desligadas.

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::json;

use super::*;
use crate::api::Api;
use crate::assets::enrich::{FieldState, FieldStatus};
use crate::paths::TreasuryPaths;
use crate::research::net::{FetchResult, FileMeta};

#[derive(Default)]
struct Fixed {
    files: HashMap<String, Vec<u8>>,
    calls: Mutex<Vec<String>>,
}

impl Fetcher for Fixed {
    fn get(&self, url: &str, _c: Option<&FileMeta>, _ua: &str) -> FetchResult {
        self.calls.lock().unwrap().push(url.to_string());
        Ok(match self.files.get(url) {
            Some(b) => Fetched::Body {
                bytes: b.clone(),
                meta: FileMeta::default(),
            },
            None => Fetched::NotFound,
        })
    }
}

const B3: &str = r#"{"issuingCompany":"TAEE","companyName":"TRANSMISSORA ALIANÇA DE ENERGIA ELÉTRICA S.A.","cnpj":"07859971000130","industryClassification":"Utilidade Pública / Energia Elétrica / Energia Elétrica","activity":"Transmissão de Energia Elétrica.","website":"www.taesa.com.br","code":"TAEE11","codeCVM":"20257","otherCodes":[{"code":"TAEE3","isin":"BRTAEEACNOR9"},{"code":"TAEE11","isin":"BRTAEECDAM10"}]}"#;
const GLEIF: &str =
    r#"{"data":[{"type":"lei-records","id":"5493000J801JZRCMFE49","attributes":{}}]}"#;
const BRASILAPI: &str = r#"{"cnpj":"07859971000130","razao_social":"TRANSMISSORA ALIANCA DE ENERGIA ELETRICA S.A.","descricao_situacao_cadastral":"ATIVA","qsa":[{"nome_socio":"FULANO"}]}"#;

fn fixed() -> Fixed {
    let mut f = Fixed::default();
    f.files
        .insert(b3_detail_url("20257"), B3.as_bytes().to_vec());
    f.files
        .insert(gleif_url("07859971000130"), GLEIF.as_bytes().to_vec());
    f.files.insert(
        brasilapi_url("07859971000130"),
        BRASILAPI.as_bytes().to_vec(),
    );
    f.files.insert(
        crate::research::brapi::resolve_url(&["TAEE11".into()]),
        br#"{"results":[{"requestedSymbol":"TAEE11","symbol":"TAEE11","changed":false,"status":"active"}]}"#.to_vec(),
    );
    f
}

fn setup() -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    api.with_session(|s, _| {
        Ok(s.db.conn().execute_batch(
            "INSERT INTO research_issuer (id, kind, name, cvm_code, sector, layout, currency, source, fetched_at)
             VALUES ('07859971000130', 'company_br', 'TAESA', '20257', 'Energia Elétrica', 'industrial', 'BRL',
                     'cvm_fca', '2026-10-01T00:00:00Z');
             INSERT INTO research_security (ticker, issuer_id, class, share_kind, isin, active, source, fetched_at)
             VALUES ('TAEE11', '07859971000130', 'stock_br', 'UNIT', 'BRTAEECDAM10', 1, 'cvm_fca', '2026-10-01T00:00:00Z');",
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
    (dir, api, id)
}

fn field<'a>(d: &'a AssetData, f: &str) -> &'a FieldStatus {
    d.fields.iter().find(|x| x.field == f).unwrap()
}

#[test]
fn parsers() {
    let d = parse_b3_detail(B3.as_bytes()).unwrap().unwrap();
    assert_eq!(
        d.classification,
        [
            Some("Utilidade Pública".into()),
            Some("Energia Elétrica".into()),
            Some("Energia Elétrica".into())
        ]
    );
    assert_eq!(d.website.as_deref(), Some("https://www.taesa.com.br"));
    assert!(parse_b3_detail(b"{}").unwrap().is_none());
    assert!(parse_b3_detail(b"<html>").is_err());
    assert_eq!(
        parse_gleif(GLEIF.as_bytes()).unwrap().as_deref(),
        Some("5493000J801JZRCMFE49")
    );
    assert_eq!(parse_gleif(br#"{"data":[]}"#).unwrap(), None);
    let i = parse_brasilapi(BRASILAPI.as_bytes()).unwrap().unwrap();
    assert_eq!(i.status.as_deref(), Some("ATIVA"));
    assert!(gleif_url("07859971000130").contains("07.859.971%2F0001-30"));
}

#[test]
fn lookup_feeds_suggestions_and_respects_cache_and_toggles() {
    let (_d, api, id) = setup();
    let f = fixed();
    let r = run_with(
        &api,
        &LookupArgs {
            asset_id: id.clone(),
            refresh: None,
        },
        &f,
    )
    .unwrap();
    assert!(
        r.sources.iter().all(|s| s.status == LookupStatus::Ok),
        "{:?}",
        r.sources
    );
    assert!(r.observations >= 7, "{}", r.observations);
    // Setor pela B3 (principal); o FCA ("Energia Elétrica") bate com o subsetor e confirma.
    let sector = field(&r.data, "sector");
    assert_eq!(sector.state, FieldState::Fill);
    let c = sector.candidate.as_ref().unwrap();
    assert_eq!(
        (c.value.as_str(), c.source.as_str()),
        ("Utilidade Pública", "b3")
    );
    assert_eq!(c.agreeing, vec!["cvm_fca".to_string()]);
    assert_eq!(c.confidence, 95);
    assert_eq!(
        field(&r.data, "segment").candidate.as_ref().unwrap().value,
        "Energia Elétrica"
    );
    // ISIN: CVM e B3 concordam.
    assert_eq!(
        field(&r.data, "isin")
            .candidate
            .as_ref()
            .unwrap()
            .confidence,
        95
    );
    assert_eq!(
        field(&r.data, "lei").candidate.as_ref().unwrap().value,
        "5493000J801JZRCMFE49"
    );
    let st = field(&r.data, "cnpj_status").candidate.as_ref().unwrap();
    assert_eq!((st.value.as_str(), st.confidence), ("ATIVA", 60));
    // Do quadro de sócios nada fica gravado.
    let socio: i64 = api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT count(*) FROM asset_observation WHERE value LIKE '%FULANO%'",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(socio, 0);
    // Gravar a sugestão da B3 funciona como as do cofre.
    api.call(
        "assets.enrich_apply",
        json!({"items": [{"asset_id": id, "field": "sector", "value": "Utilidade Pública"}]}),
    )
    .unwrap();
    // Dentro dos 30 dias, não vai de novo à rede.
    let n = f.calls.lock().unwrap().len();
    let r = run_with(
        &api,
        &LookupArgs {
            asset_id: id.clone(),
            refresh: None,
        },
        &f,
    )
    .unwrap();
    assert!(r.sources.iter().all(|s| s.status == LookupStatus::Fresh));
    assert_eq!(f.calls.lock().unwrap().len(), n);
    assert_eq!(field(&r.data, "sector").state, FieldState::Ok);
    // Fonte desligada não é consultada.
    api.call("sources.set", json!({"id": "gleif", "enabled": false}))
        .unwrap();
    let r = run_with(
        &api,
        &LookupArgs {
            asset_id: id,
            refresh: Some(true),
        },
        &f,
    )
    .unwrap();
    let g = r.sources.iter().find(|s| s.source == "gleif").unwrap();
    assert_eq!(g.status, LookupStatus::Disabled);
    assert!(
        !f.calls.lock().unwrap()[n..]
            .iter()
            .any(|u| u.contains("gleif"))
    );
}

#[test]
fn source_ids_serialize_as_their_keys() {
    for id in SourceId::ALL {
        assert_eq!(serde_json::to_value(id).unwrap(), json!(id.key()), "{id:?}");
    }
}

/// Confere as três fontes de verdade (rede): `cargo test -p treasury-core live_sources -- --ignored`.
#[test]
#[ignore]
fn live_sources() {
    let f = HttpFetcher::new();
    let body = |url: &str| match f.get(url, None, "").unwrap() {
        Fetched::Body { bytes, .. } => bytes,
        other => panic!("{url}: {other:?}"),
    };
    let d = parse_b3_detail(&body(&b3_detail_url("20257")))
        .unwrap()
        .unwrap();
    assert_eq!(d.classification[0].as_deref(), Some("Utilidade Pública"));
    let lei = parse_gleif(&body(&gleif_url("33000167000101"))).unwrap();
    assert_eq!(lei.as_deref(), Some("5493000J801JZRCMFE49"));
    let i = parse_brasilapi(&body(&brasilapi_url("07859971000130")))
        .unwrap()
        .unwrap();
    assert!(i.legal_name.is_some() && i.status.is_some(), "{i:?}");
}

/// Plano de previdência cadastrado com o CNPJ do plano: a consulta diz, pela fonte `cvm`, que o
/// CNPJ não está no registro de fundos e sugere o fundo achado pelo nome (que o usuário
/// confirma); com o CNPJ do fundo, a fonte responde `ok` com o nome do fundo.
#[test]
fn pension_with_the_plans_cnpj_gets_the_likely_fund_from_the_cvm_registry() {
    let (_d, api, _) = setup();
    let id = api
        .call(
            "assets.create",
            json!({"name": "CAIXA FIC PREV 250", "class": "pension", "cnpj": "11.444.777/0001-61"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let lookup = |api: &Api| {
        run_with(
            api,
            &LookupArgs {
                asset_id: id.clone(),
                refresh: Some(true),
            },
            &Fixed::default(),
        )
        .unwrap()
    };
    // Registro ainda não baixado: diz como baixar.
    let r = lookup(&api);
    let cvm = r.sources.iter().find(|s| s.source == "cvm").unwrap();
    assert_eq!(cvm.status, LookupStatus::NotFound);
    assert!(cvm.detail.as_deref().unwrap().contains("CVM"));

    let fund = "32893503000120";
    api.with_session(|s, _| {
        let mut reg = crate::score::funds::registry::Registry {
            registro_at: Some("2026-10-21T08:00:00Z".into()),
            ..Default::default()
        };
        reg.funds.insert(
            fund.into(),
            crate::score::funds::registry::FundInfo {
                cnpj: fund.into(),
                name: "CAIXA FIC PREVIDENCIÁRIO 250 RENDA FIXA LONGO PRAZO".into(),
                classification: Some("Renda Fixa".into()),
                ..Default::default()
            },
        );
        reg.fund_guesses.insert(id.clone(), fund.into());
        crate::score::funds::registry::save(s.db.conn(), &reg)
    })
    .unwrap();
    let r = lookup(&api);
    let cvm = r.sources.iter().find(|s| s.source == "cvm").unwrap();
    assert_eq!(cvm.status, LookupStatus::NotFound);
    let detail = cvm.detail.as_deref().unwrap();
    assert!(
        detail.contains("11.444.777/0001-61") && detail.contains("32.893.503/0001-20"),
        "{detail}"
    );
    // A sugestão do CNPJ do fundo aparece nos dados do ativo, para o usuário confirmar.
    let cnpj = field(&r.data, "cnpj");
    let suggested: Vec<&str> = cnpj
        .candidate
        .iter()
        .chain(&cnpj.options)
        .map(|c| c.value.as_str())
        .collect();
    assert!(
        suggested
            .iter()
            .any(|v| v.contains("32893503000120") || v.contains("32.893.503/0001-20")),
        "{suggested:?}"
    );

    // Com o CNPJ do fundo no ativo, a CVM confirma.
    api.call(
        "assets.update",
        json!({"id": id, "cnpj": "32.893.503/0001-20"}),
    )
    .unwrap();
    let r = lookup(&api);
    let cvm = r.sources.iter().find(|s| s.source == "cvm").unwrap();
    assert_eq!(cvm.status, LookupStatus::Ok);
    assert!(
        cvm.detail
            .as_deref()
            .unwrap()
            .contains("PREVIDENCIÁRIO 250")
    );
}

/// Plano de fundo de pensão (caso da Quanta Previdência, 21/10/2026): a Receita diz pela
/// natureza jurídica que o CNPJ é de um plano de previdência complementar fechada. A consulta
/// sugere o tipo "efpc" (o usuário confirma), não procura fundo na CVM e explica que as cotas
/// vêm do extrato. Confirmado, o plano passa a fundo de pensão.
#[test]
fn closed_pension_plan_is_recognized_from_the_receitas_legal_nature() {
    let (_d, api, _) = setup();
    let id = api
        .call(
            "assets.create",
            json!({"name": "PLANOPREV-Arrojado", "class": "pension", "cnpj": "11.444.777/0001-61"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut f = Fixed::default();
    f.files.insert(
        brasilapi_url("11444777000161"),
        r#"{"cnpj":"11444777000161","razao_social":"PLANO DE BENEFICIOS PREVIDENCIARIOS","descricao_situacao_cadastral":"ATIVA","natureza_juridica":"Plano de Benefícios de Previdência Complementar Fechada"}"#
            .as_bytes()
            .to_vec(),
    );
    let r = run_with(
        &api,
        &LookupArgs {
            asset_id: id.clone(),
            refresh: Some(true),
        },
        &f,
    )
    .unwrap();
    let kind = field(&r.data, "pension_type");
    let c = kind.candidate.as_ref().expect("sugestão do tipo");
    assert_eq!((c.value.as_str(), c.source.as_str()), ("efpc", "brasilapi"));
    let cvm = r.sources.iter().find(|s| s.source == "cvm").unwrap();
    assert_eq!(cvm.status, LookupStatus::NotFound);
    assert!(
        cvm.detail.as_deref().unwrap().contains("extrato"),
        "{cvm:?}"
    );
    // Já reconhecido pela leitura da Receita, mesmo antes de confirmar.
    assert!(
        api.with_session(|s, _| crate::pension::is_efpc(s.db.conn(), &id))
            .unwrap()
    );

    api.call(
        "assets.enrich_apply",
        json!({"items": [{"asset_id": id, "field": "pension_type", "value": "efpc"}]}),
    )
    .unwrap();
    let plan: String = api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT plan_type FROM pension_plan WHERE asset_id = ?1",
                [&id],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(plan, "efpc");
    // A razão social vira a administradora sugerida, e a classe CVM/ANBIMA não é cobrada
    // (feedback 01a184bb-5c45).
    let r = run_with(
        &api,
        &LookupArgs {
            asset_id: id.clone(),
            refresh: Some(true),
        },
        &f,
    )
    .unwrap();
    let adm = field(&r.data, "administrator");
    let c = adm.candidate.as_ref().expect("administradora sugerida");
    assert_eq!(
        (c.value.as_str(), c.source.as_str()),
        ("PLANO DE BENEFICIOS PREVIDENCIARIOS", "brasilapi")
    );
    assert!(
        r.data
            .fields
            .iter()
            .all(|f| f.field != "fund_class_cvm" && f.field != "fund_class_anbima"),
        "{:?}",
        r.data.fields.iter().map(|f| &f.field).collect::<Vec<_>>()
    );
    api.call(
        "assets.enrich_apply",
        json!({"items": [{"asset_id": id, "field": "administrator", "value": "PLANO DE BENEFICIOS PREVIDENCIARIOS"}]}),
    )
    .unwrap();
}

#[test]
fn renamed_ticker_becomes_a_sensitive_suggestion() {
    let (_d, api, _) = setup();
    let id = api
        .call(
            "assets.create",
            json!({"ticker": "ELET3", "name": "Eletrobras", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut f = Fixed::default();
    f.files.insert(
        crate::research::brapi::resolve_url(&["ELET3".into()]),
        br#"{"results":[{"requestedSymbol":"ELET3","symbol":"AXIA3","changed":true,"status":"renamed","effectiveDate":"2025-11-10"}]}"#.to_vec(),
    );
    let r = run_with(
        &api,
        &LookupArgs {
            asset_id: id.clone(),
            refresh: None,
        },
        &f,
    )
    .unwrap();
    let b = r.sources.iter().find(|s| s.source == "brapi").unwrap();
    assert_eq!(b.status, LookupStatus::Ok);
    // Sem token: a resolução não usa a cota.
    assert!(
        f.calls
            .lock()
            .unwrap()
            .iter()
            .any(|u| u.contains("tickers/resolve"))
    );
    let t = field(&r.data, "ticker");
    assert_eq!(t.state, FieldState::Update);
    assert_eq!(t.candidate.as_ref().unwrap().value, "AXIA3");
    // Só troca com pedido explícito.
    assert!(
        api.call(
            "assets.enrich_apply",
            json!({"items": [{"asset_id": id, "field": "ticker", "value": "AXIA3"}]}),
        )
        .is_err()
    );
    api.call(
        "assets.enrich_apply",
        json!({"items": [{"asset_id": id, "field": "ticker", "value": "AXIA3", "replace": true}]}),
    )
    .unwrap();
    let a = api.call("assets.get", json!({"id": id})).unwrap();
    assert_eq!(a["ticker"], "AXIA3");
}
