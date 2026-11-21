//! Consulta de um ativo nas fontes públicas novas do enriquecimento (fase 2,
//! docs/agent-enrichment.md §4): classificação setorial da B3 em três níveis, site e
//! descrição da atividade (API pública das empresas listadas, pelo código CVM), LEI no GLEIF
//! e razão social e situação cadastral do CNPJ pela BrasilAPI (espelho da Receita Federal).
//!
//! - Só identificadores públicos saem do computador (código CVM, CNPJ); nada diz que o ativo
//!   está numa carteira.
//! - Cada fonte tem liga/desliga em Ajustes › Fontes de dados; um ativo consultado com
//!   sucesso só volta à fonte depois de 30 dias (`research_fetch_log`), salvo `refresh`.
//! - As leituras ficam em `asset_observation` e entram nas sugestões de
//!   [`super::enrich::data_gaps`]; nada é gravado no ativo (isso é `assets.enrich_apply`).
//! - A rede roda fora da trava do cofre. Textos das fontes são dados, nunca instruções; da
//!   BrasilAPI só a razão social e a situação são lidas (o resto da resposta, como o quadro
//!   de sócios, é descartado sem gravar).

use base64::Engine as _;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::AssetClass;
use super::enrich::{AssetData, FieldObservation};
use crate::error::Result;
use crate::research::net::{Fetched, Fetcher, HttpFetcher};
use crate::research::{cnpj_digits, format_cnpj};
use crate::sources::SourceId;
use crate::time::now_utc;

/// Fontes consultadas pela rede (as leituras delas vêm de `asset_observation`).
pub const NETWORK_SOURCES: &[&str] = &["b3", "gleif", "brasilapi", "brapi"];
/// Um ativo consultado com sucesso só volta à fonte depois disto.
pub const REFRESH_DAYS: i64 = 30;
/// Inexistente: nova tentativa depois de uma semana; erro: no dia seguinte.
const NOT_FOUND_DAYS: i64 = 7;
const ERROR_DAYS: i64 = 1;

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct LookupArgs {
    pub asset_id: String,
    /// Consulta de novo mesmo dentro dos 30 dias.
    #[serde(default)]
    #[ts(optional)]
    pub refresh: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct LookupAllArgs {
    /// Só estes ativos; sem ele, os não arquivados das classes brasileiras.
    #[ts(optional)]
    pub asset_ids: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub refresh: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LookupStatus {
    Ok,
    NotFound,
    /// Consultada há menos de 30 dias (ou erro/inexistente recente): não foi de novo.
    Fresh,
    /// Desligada em Ajustes › Fontes de dados.
    Disabled,
    /// Sem a chave que a fonte precisa (código CVM, CNPJ).
    NoKey,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LookupSource {
    pub source: String,
    pub status: LookupStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LookupReport {
    pub asset_id: String,
    pub sources: Vec<LookupSource>,
    /// Leituras novas gravadas nesta consulta.
    #[ts(type = "number")]
    pub observations: i64,
    /// Campos e sugestões do ativo depois da consulta.
    pub data: AssetData,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LookupAllReport {
    #[ts(type = "number")]
    pub assets: i64,
    /// Ativos com pelo menos uma fonte consultada de fato.
    #[ts(type = "number")]
    pub fetched: i64,
    #[ts(type = "number")]
    pub observations: i64,
    #[ts(type = "number")]
    pub errors: i64,
}

// ---------------------------------------------------------------------------------------
// URLs e interpretação das respostas (sem rede, testadas com fixtures)

const B3_BASE: &str =
    "https://sistemaswebb3-listados.b3.com.br/listedCompaniesProxy/CompanyCall/GetDetail/";

pub fn b3_detail_url(cvm_code: &str) -> String {
    let json = format!(
        "{{\"codeCVM\":\"{}\",\"language\":\"pt-br\"}}",
        cvm_code.trim().trim_start_matches('0')
    );
    format!(
        "{B3_BASE}{}",
        base64::engine::general_purpose::STANDARD.encode(json)
    )
}

pub fn gleif_url(cnpj: &str) -> String {
    let f = format_cnpj(&cnpj_digits(cnpj)).replace('/', "%2F");
    format!(
        "https://api.gleif.org/api/v1/lei-records?filter%5Bentity.registeredAs%5D={f}&page%5Bsize%5D=1"
    )
}

pub fn brasilapi_url(cnpj: &str) -> String {
    format!("https://brasilapi.com.br/api/cnpj/v1/{}", cnpj_digits(cnpj))
}

/// Cadastro de uma empresa listada na B3.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct B3Detail {
    /// Setor econômico, subsetor e segmento.
    pub classification: [Option<String>; 3],
    pub activity: Option<String>,
    pub website: Option<String>,
    /// Código de negociação → ISIN.
    pub isins: Vec<(String, String)>,
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(String::from)
}

pub fn parse_b3_detail(bytes: &[u8]) -> std::result::Result<Option<B3Detail>, String> {
    let v: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(e) => return Err(format!("B3: {e}")),
    };
    if !v.is_object() || s(&v, "codeCVM").is_none() && s(&v, "companyName").is_none() {
        return Ok(None);
    }
    let mut classification: [Option<String>; 3] = Default::default();
    if let Some(c) = s(&v, "industryClassification") {
        for (i, part) in c.split('/').map(str::trim).take(3).enumerate() {
            classification[i] = Some(part.to_string()).filter(|p| !p.is_empty());
        }
    }
    let website = s(&v, "website").map(|w| {
        if w.starts_with("http") {
            w
        } else {
            format!("https://{w}")
        }
    });
    let isins = v
        .get("otherCodes")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|c| Some((s(c, "code")?.to_uppercase(), s(c, "isin")?.to_uppercase())))
                .collect()
        })
        .unwrap_or_default();
    Ok(Some(B3Detail {
        classification,
        activity: s(&v, "activity"),
        website,
        isins,
    }))
}

pub fn parse_gleif(bytes: &[u8]) -> std::result::Result<Option<String>, String> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| format!("GLEIF: {e}"))?;
    Ok(v.get("data")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|r| s(r, "id"))
        .filter(|id| id.len() == 20 && id.chars().all(|c| c.is_ascii_alphanumeric())))
}

/// Razão social e situação cadastral (o resto da resposta é descartado).
#[derive(Debug, Clone, PartialEq)]
pub struct CnpjInfo {
    pub legal_name: Option<String>,
    pub status: Option<String>,
    /// Natureza jurídica (ex.: "Plano de Benefícios de Previdência Complementar Fechada").
    pub legal_nature: Option<String>,
}

pub fn parse_brasilapi(bytes: &[u8]) -> std::result::Result<Option<CnpjInfo>, String> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| format!("BrasilAPI: {e}"))?;
    if s(&v, "cnpj").is_none() {
        return Ok(None);
    }
    Ok(Some(CnpjInfo {
        legal_name: s(&v, "razao_social"),
        status: s(&v, "descricao_situacao_cadastral"),
        legal_nature: s(&v, "natureza_juridica"),
    }))
}

// ---------------------------------------------------------------------------------------
// Planejamento, rede e gravação

/// O que consultar para um ativo.
#[derive(Debug, Clone, Default)]
struct Keys {
    ticker: Option<String>,
    cvm_code: Option<String>,
    /// CNPJ do emissor (ações) ou do próprio fundo (FII, ETF, fundo).
    cnpj: Option<String>,
    stock: bool,
    pension: bool,
}

fn keys(conn: &Connection, a: &super::Asset) -> Result<Keys> {
    let mut k = Keys {
        ticker: a.ticker.clone(),
        ..Default::default()
    };
    let issuer = crate::research::data::issuer_of_asset(conn, a)?;
    let row: Option<(String, Option<String>)> = match &issuer {
        Some(id) => conn
            .query_row(
                "SELECT kind, cvm_code FROM research_issuer WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?,
        None => None,
    };
    match a.class {
        AssetClass::StockBr | AssetClass::Bdr => {
            k.stock = true;
            if let Some((kind, code)) = &row
                && kind == "company_br"
            {
                k.cvm_code = code.clone();
                k.cnpj = issuer.clone().filter(|c| c.len() == 14);
            }
            if k.cvm_code.is_none() {
                k.cvm_code = super::enrich::attributes_of(conn, &a.id)?
                    .get("cvm_code")
                    .cloned();
            }
        }
        AssetClass::Fii | AssetClass::EtfBr | AssetClass::Fund | AssetClass::Pension => {
            k.pension = a.class == AssetClass::Pension;
            k.cnpj = a
                .cnpj
                .as_deref()
                .map(cnpj_digits)
                .filter(|c| c.len() == 14)
                .or_else(|| issuer.filter(|c| c.len() == 14));
        }
        _ => {}
    }
    Ok(k)
}

fn log_key(source: &str, key: &str) -> (String, String) {
    (format!("enrich_{source}"), key.to_string())
}

/// A fonte já foi consultada para esta chave há pouco tempo.
fn fresh(conn: &Connection, source: &str, key: &str) -> Result<bool> {
    let (src, k) = log_key(source, key);
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT fetched_at, status FROM research_fetch_log WHERE source = ?1 AND key = ?2",
            params![src, k],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((at, status)) = row else {
        return Ok(false);
    };
    let days = match status.as_str() {
        "ok" => REFRESH_DAYS,
        "not_found" => NOT_FOUND_DAYS,
        _ => ERROR_DAYS,
    };
    Ok(chrono::DateTime::parse_from_rfc3339(&at)
        .is_ok_and(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_days() < days))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Src {
    B3,
    Gleif,
    BrasilApi,
    /// Tickers renomeados (brapi.dev, sem token: não gasta a cota).
    Brapi,
}

impl Src {
    fn key(self) -> &'static str {
        match self {
            Src::B3 => "b3",
            Src::Gleif => "gleif",
            Src::BrasilApi => "brasilapi",
            Src::Brapi => "brapi",
        }
    }
    fn source_id(self) -> SourceId {
        match self {
            Src::B3 => SourceId::B3,
            Src::Gleif => SourceId::Gleif,
            Src::BrasilApi => SourceId::BrasilApi,
            Src::Brapi => SourceId::Brapi,
        }
    }
}

struct Planned {
    src: Src,
    key: Option<String>,
    run: bool,
    status: Option<LookupStatus>,
}

/// Uma leitura a gravar.
struct Reading {
    field: &'static str,
    value: String,
    scheme: Option<&'static str>,
    source: &'static str,
    source_ref: Option<String>,
}

fn readings_b3(d: &B3Detail, ticker: Option<&str>, url: &str) -> Vec<Reading> {
    let mut out = vec![];
    let mut push = |field: &'static str, v: Option<&String>, scheme: Option<&'static str>| {
        if let Some(v) = v {
            out.push(Reading {
                field,
                value: v.clone(),
                scheme,
                source: "b3",
                source_ref: Some(url.to_string()),
            });
        }
    };
    push("sector", d.classification[0].as_ref(), Some("b3"));
    push("subsector", d.classification[1].as_ref(), Some("b3"));
    push("segment", d.classification[2].as_ref(), Some("b3"));
    push("activity", d.activity.as_ref(), None);
    push("website", d.website.as_ref(), None);
    if let Some(t) = ticker {
        let isin = d
            .isins
            .iter()
            .find(|(c, _)| c.eq_ignore_ascii_case(t))
            .map(|(_, i)| i);
        push("isin", isin, None);
    }
    out
}

fn fetch_one(
    f: &dyn Fetcher,
    src: Src,
    key: &str,
    k: &Keys,
) -> (LookupStatus, Option<String>, Vec<Reading>) {
    let url = match src {
        Src::B3 => b3_detail_url(key),
        Src::Gleif => gleif_url(key),
        Src::BrasilApi => brasilapi_url(key),
        Src::Brapi => crate::research::brapi::resolve_url(&[key.to_string()]),
    };
    let bytes = match f.get(&url, None, "") {
        Ok(Fetched::Body { bytes, .. }) => bytes,
        Ok(Fetched::NotFound) => return (LookupStatus::NotFound, None, vec![]),
        Ok(Fetched::NotModified) => return (LookupStatus::Fresh, None, vec![]),
        Err(e) => return (LookupStatus::Error, Some(e), vec![]),
    };
    let parsed: std::result::Result<Vec<Reading>, String> = match src {
        Src::B3 => parse_b3_detail(&bytes).map(|d| {
            d.map(|d| readings_b3(&d, k.ticker.as_deref(), &url))
                .unwrap_or_default()
        }),
        Src::Gleif => parse_gleif(&bytes).map(|lei| {
            lei.map(|l| {
                vec![Reading {
                    field: "lei",
                    source_ref: Some(format!("https://search.gleif.org/#/record/{l}")),
                    value: l,
                    scheme: None,
                    source: "gleif",
                }]
            })
            .unwrap_or_default()
        }),
        Src::Brapi => {
            crate::research::brapi::parse_resolve(&bytes).map(|r| renamed_readings(&r, key))
        }
        Src::BrasilApi => parse_brasilapi(&bytes).map(|i| {
            let mut out = vec![];
            if let Some(i) = i {
                if k.stock
                    && let Some(n) = i.legal_name.clone()
                {
                    out.push(Reading {
                        field: "issuer_name",
                        value: n,
                        scheme: None,
                        source: "brasilapi",
                        source_ref: None,
                    });
                }
                // Plano de fundo de pensão (EFPC): a Receita diz pela natureza jurídica, e a
                // razão social do CNPJ é a da entidade que administra o plano.
                if k.pension
                    && i.legal_nature.as_deref().is_some_and(|n| {
                        crate::research::norm(n).contains("previdencia complementar fechada")
                    })
                {
                    out.push(Reading {
                        field: "pension_type",
                        value: "efpc".into(),
                        scheme: None,
                        source: "brasilapi",
                        source_ref: None,
                    });
                    if let Some(n) = i.legal_name.clone() {
                        out.push(Reading {
                            field: "administrator",
                            value: n,
                            scheme: None,
                            source: "brasilapi",
                            source_ref: None,
                        });
                    }
                }
                if let Some(st) = i.status {
                    out.push(Reading {
                        field: "cnpj_status",
                        value: st,
                        scheme: None,
                        source: "brasilapi",
                        source_ref: None,
                    });
                }
            }
            out
        }),
    };
    match parsed {
        Ok(r) if r.is_empty() && src != Src::Brapi => (LookupStatus::NotFound, None, r),
        Ok(r) => (LookupStatus::Ok, None, r),
        Err(e) => (LookupStatus::Error, Some(e), vec![]),
    }
}

/// Resultado de uma fonte: fonte, chave, situação, detalhe do erro e leituras.
type Fetch = (Src, String, LookupStatus, Option<String>, Vec<Reading>);

fn store(conn: &Connection, asset_id: &str, results: &[Fetch]) -> Result<i64> {
    let now = now_utc();
    let today = crate::time::today_local();
    let mut n = 0;
    for (src, key, status, detail, readings) in results {
        let (s, k) = log_key(src.key(), key);
        let st = match status {
            LookupStatus::Ok => "ok",
            LookupStatus::NotFound => "not_found",
            _ => "error",
        };
        conn.execute(
            "INSERT INTO research_fetch_log (source, key, fetched_at, status, detail) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (source, key) DO UPDATE SET fetched_at = excluded.fetched_at, status = excluded.status,
                 detail = excluded.detail",
            params![s, k, now, st, detail],
        )?;
        let health = match status {
            LookupStatus::Error => Err(detail.clone().unwrap_or_default()),
            _ => Ok(()),
        };
        crate::sources::record_result(conn, src.source_id(), &health, today)?;
        for r in readings {
            let norm = super::enrich::normalize(r.field, &r.value);
            conn.execute(
                "INSERT INTO asset_observation (id, asset_id, field, value, value_norm, scheme, source_id, tier,
                                                source_ref, published_at, observed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    crate::ids::new_id(),
                    asset_id,
                    r.field,
                    r.value,
                    norm,
                    r.scheme,
                    r.source,
                    tier_of(r.source),
                    r.source_ref,
                    crate::time::fmt_date(today),
                    now
                ],
            )?;
            n += 1;
        }
    }
    Ok(n)
}

/// Leitura do ticker atual quando a brapi diz que o do ativo mudou.
fn renamed_readings(r: &[crate::research::brapi::Renamed], ticker: &str) -> Vec<Reading> {
    r.iter()
        .filter(|x| x.from.eq_ignore_ascii_case(ticker))
        .map(|x| Reading {
            field: "ticker",
            value: x.to.clone(),
            scheme: None,
            source: "brapi",
            source_ref: x.effective.as_ref().map(|d| {
                format!(
                    "{}/api/v2/tickers/renames?symbols={}#{d}",
                    crate::research::brapi::BASE,
                    x.from
                )
            }),
        })
        .collect()
}

/// Nível de uma fonte da rede (§4.1): B3 e GLEIF oficiais, espelho da Receita e brapi
/// secundários.
pub fn tier_of(source: &str) -> u8 {
    match source {
        "brasilapi" | "brapi" => 3,
        _ => 1,
    }
}

/// Leituras mais recentes das fontes da rede para um ativo (uma por campo e fonte).
pub fn stored(conn: &Connection, asset_id: &str) -> Result<Vec<FieldObservation>> {
    let mut stmt = conn.prepare(
        "SELECT field, value, scheme, source_id, tier, source_ref, published_at FROM asset_observation o
          WHERE asset_id = ?1 AND source_id IN ('b3', 'gleif', 'brasilapi', 'brapi')
            AND observed_at = (SELECT max(observed_at) FROM asset_observation x
                                WHERE x.asset_id = o.asset_id AND x.field = o.field AND x.source_id = o.source_id)
          GROUP BY field, source_id",
    )?;
    let rows = stmt
        .query_map([asset_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .map(|(field, value, scheme, source, tier, source_ref, as_of)| {
            FieldObservation::new(
                &field, &value, scheme, &source, tier as u8, source_ref, as_of,
            )
        })
        .collect())
}

/// `assets.enrich_lookup`: consulta as fontes da rede para um ativo e devolve as sugestões.
pub fn run(api: &crate::Api, a: &LookupArgs) -> Result<LookupReport> {
    run_with(api, a, &HttpFetcher::new())
}

pub fn run_with(api: &crate::Api, a: &LookupArgs, f: &dyn Fetcher) -> Result<LookupReport> {
    let force = a.refresh.unwrap_or(false);
    // 1. Chaves e o que está fresco, com o banco.
    let (k, plan) = api.with_session(|s, _| {
        let conn = s.db.conn();
        let asset = super::get(conn, &a.asset_id)?;
        let k = keys(conn, &asset)?;
        let mut plan = vec![];
        for (src, key) in [
            (Src::B3, k.cvm_code.clone().filter(|_| k.stock)),
            (Src::Gleif, k.cnpj.clone().filter(|_| k.stock)),
            (Src::BrasilApi, k.cnpj.clone()),
            (
                Src::Brapi,
                k.ticker.clone().map(|t| t.trim().to_uppercase()),
            ),
        ] {
            let applies = match src {
                Src::B3 | Src::Gleif => k.stock,
                Src::BrasilApi => k.stock || k.cnpj.is_some() || is_fund(asset.class),
                Src::Brapi => crate::market::sync::is_b3(asset.class),
            };
            if !applies {
                continue;
            }
            let (run, status) = match &key {
                None => (false, Some(LookupStatus::NoKey)),
                Some(_) if !crate::sources::enabled(conn, src.source_id()) => {
                    (false, Some(LookupStatus::Disabled))
                }
                Some(key) if !force && fresh(conn, src.key(), key)? => {
                    (false, Some(LookupStatus::Fresh))
                }
                Some(_) => (true, None),
            };
            plan.push(Planned {
                src,
                key,
                run,
                status,
            });
        }
        Ok((k, plan))
    })?;
    // 2. Rede, sem segurar o banco.
    let mut sources = vec![];
    let mut results = vec![];
    for p in plan {
        match (p.run, &p.key) {
            (true, Some(key)) => {
                let (status, detail, readings) = fetch_one(f, p.src, key, &k);
                sources.push(LookupSource {
                    source: p.src.key().into(),
                    status,
                    detail: detail.clone(),
                });
                results.push((p.src, key.clone(), status, detail, readings));
            }
            _ => sources.push(LookupSource {
                source: p.src.key().into(),
                status: p.status.unwrap_or(LookupStatus::Fresh),
                detail: None,
            }),
        }
    }
    // 3. Grava as leituras e devolve as sugestões. Banco só de leitura (MCP com o app
    // fechado): devolve sem gravar.
    let observations = if results.is_empty() {
        0
    } else {
        match api.with_session(|s, _| {
            let n = store(s.db.conn(), &a.asset_id, &results)?;
            // Leituras novas podem abrir (ou fechar) o episódio do alerta dos dados.
            if let Err(e) = super::data_alert::check(s.db.conn()) {
                tracing::warn!("alerta dos dados dos ativos: {e}");
            }
            Ok(n)
        }) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!("leituras do enriquecimento: {e}");
                0
            }
        }
    };
    // Fundos, previdência e ETFs: o registro de fundos da CVM (baixado na coleta da análise,
    // sem rede aqui) diz se o CNPJ é o de um fundo e, se não for, o provável pelo nome.
    if let Some(cvm) = api.with_session(|s, _| cvm_status(s.db.conn(), &a.asset_id))? {
        sources.push(cvm);
    }
    let data = api.with_session(|s, _| {
        let g = super::enrich::data_gaps(
            s.db.conn(),
            &super::enrich::DataGapsArgs {
                asset_ids: Some(vec![a.asset_id.clone()]),
                include_ok: Some(true),
                ..Default::default()
            },
        )?;
        Ok(g.assets.into_iter().next())
    })?;
    Ok(LookupReport {
        asset_id: a.asset_id.clone(),
        sources,
        observations,
        data: data.expect("o ativo existe (conferido acima)"),
    })
}

/// O que o registro de fundos da CVM diz do CNPJ de um fundo, plano de previdência ou ETF.
fn cvm_status(conn: &Connection, asset_id: &str) -> Result<Option<LookupSource>> {
    use crate::i18n::{args, t, t_args};
    use crate::score::funds::registry;
    let a = super::get(conn, asset_id)?;
    if !matches!(
        a.class,
        AssetClass::Fund | AssetClass::Pension | AssetClass::EtfBr
    ) {
        return Ok(None);
    }
    let src = |status, detail: Option<String>| {
        Some(LookupSource {
            source: "cvm".into(),
            status,
            detail,
        })
    };
    if !crate::sources::enabled(conn, SourceId::Cvm) {
        return Ok(src(LookupStatus::Disabled, None));
    }
    if a.class == AssetClass::Pension && crate::pension::is_efpc(conn, &a.id)? {
        return Ok(src(
            LookupStatus::NotFound,
            Some(t("nucleo-score-fundo-efpc-sem-cotas")),
        ));
    }
    let reg = registry::load(conn)?;
    if reg.registro_at.is_none() {
        return Ok(src(
            LookupStatus::NotFound,
            Some(t("nucleo-ativos-consulta-cvm-sem-registro")),
        ));
    }
    let guess = reg.fund_guesses.get(&a.id).map(|g| {
        (
            g,
            reg.funds.get(g).map(|f| f.name.clone()).unwrap_or_default(),
        )
    });
    let own = a.cnpj.as_deref().map(cnpj_digits).filter(|c| c.len() == 14);
    Ok(
        match (registry::cnpj_of(&reg, &a.id, a.cnpj.as_deref()), guess) {
            (Some(c), _) if reg.funds.contains_key(&c) => {
                src(LookupStatus::Ok, reg.funds.get(&c).map(|f| f.name.clone()))
            }
            (Some(c), Some((g, name))) => src(
                LookupStatus::NotFound,
                Some(t_args(
                    "nucleo-score-fundo-cnpj-plano-provavel",
                    &args(&[
                        ("cnpj", format_cnpj(&c).into()),
                        ("provavel", format_cnpj(g).into()),
                        ("nome", name.into()),
                    ]),
                )),
            ),
            (Some(c), None) => src(
                LookupStatus::NotFound,
                Some(t_args(
                    "nucleo-score-fundo-cnpj-fora-do-registro",
                    &args(&[
                        ("cnpj", format_cnpj(&c).into()),
                        (
                            "data",
                            reg.registro_at
                                .as_deref()
                                .and_then(|d| d.get(..10))
                                .unwrap_or("")
                                .to_string()
                                .into(),
                        ),
                        (
                            "tipo",
                            if a.class == AssetClass::Pension {
                                "previdencia"
                            } else {
                                "fundo"
                            }
                            .into(),
                        ),
                    ]),
                )),
            ),
            (None, Some((g, name))) if own.is_none() => src(
                LookupStatus::NoKey,
                Some(t_args(
                    "nucleo-score-fundo-sem-cnpj-provavel",
                    &args(&[("cnpj", format_cnpj(g).into()), ("nome", name.into())]),
                )),
            ),
            _ => src(LookupStatus::NoKey, Some(t("nucleo-score-fundo-sem-cnpj"))),
        },
    )
}

fn is_fund(c: AssetClass) -> bool {
    matches!(
        c,
        AssetClass::Fii | AssetClass::EtfBr | AssetClass::Fund | AssetClass::Pension
    )
}

/// Classes que as fontes da rede da fase 2 cobrem.
pub fn covered(c: AssetClass) -> bool {
    matches!(c, AssetClass::StockBr | AssetClass::Bdr) || is_fund(c)
}

/// Resolve de uma vez os tickers da B3 ainda não conferidos (ou todos, com `force`).
pub fn resolve_batch(
    api: &crate::Api,
    ids: &[String],
    force: bool,
    f: &dyn Fetcher,
) -> Result<i64> {
    let due: Vec<(String, String)> = api.with_session(|s, _| {
        let conn = s.db.conn();
        if !crate::sources::enabled(conn, SourceId::Brapi) {
            return Ok(vec![]);
        }
        let mut out = vec![];
        for id in ids {
            let a = super::get(conn, id)?;
            if let Some(t) = a.ticker.as_deref().map(|t| t.trim().to_uppercase())
                && crate::market::sync::is_b3(a.class)
                && (force || !fresh(conn, Src::Brapi.key(), &t)?)
            {
                out.push((id.clone(), t));
            }
        }
        Ok(out)
    })?;
    let mut n = 0;
    for chunk in due.chunks(20) {
        let tickers: Vec<String> = chunk.iter().map(|(_, t)| t.clone()).collect();
        let res = match f.get(&crate::research::brapi::resolve_url(&tickers), None, "") {
            Ok(Fetched::Body { bytes, .. }) => crate::research::brapi::parse_resolve(&bytes),
            Ok(_) => Ok(vec![]),
            Err(e) => Err(e),
        };
        let renamed = match res {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("tickers renomeados (brapi): {e}");
                break;
            }
        };
        n += api.with_session(|s, _| {
            let mut n = 0;
            for (id, t) in chunk {
                let readings = renamed_readings(&renamed, t);
                // Ticker sem renome também é resposta boa (volta em 30 dias).
                let status = LookupStatus::Ok;
                n += store(
                    s.db.conn(),
                    id,
                    &[(Src::Brapi, t.clone(), status, None, readings)],
                )?;
            }
            Ok(n)
        })?;
    }
    Ok(n)
}

/// `assets.enrich_lookup_all`: consulta, um por vez, os ativos cobertos (só a interface).
pub fn run_all(api: &crate::Api, a: &LookupAllArgs) -> Result<LookupAllReport> {
    let ids: Vec<String> = api.with_session(|s, _| {
        let conn = s.db.conn();
        Ok(match &a.asset_ids {
            Some(ids) => ids.clone(),
            None => super::list(conn, &Default::default())?
                .into_iter()
                .filter(|x| covered(x.class))
                .map(|x| x.id)
                .collect(),
        })
    })?;
    let f = HttpFetcher::new();
    let mut out = LookupAllReport::default();
    // Tickers renomeados em lotes de 20 (sem token, 20 pedidos por minuto por IP): cada
    // ativo depois só vê a leitura fresca.
    out.observations += resolve_batch(api, &ids, a.refresh.unwrap_or(false), &f)?;
    for id in ids {
        let r = run_with(
            api,
            &LookupArgs {
                asset_id: id,
                refresh: a.refresh,
            },
            &f,
        )?;
        out.assets += 1;
        out.observations += r.observations;
        if r.sources.iter().any(|s| s.status == LookupStatus::Error) {
            out.errors += 1;
        }
        if r.sources
            .iter()
            .any(|s| matches!(s.status, LookupStatus::Ok | LookupStatus::NotFound))
        {
            out.fetched += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
