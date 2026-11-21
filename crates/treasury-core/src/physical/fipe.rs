//! Tabela FIPE pela API pública e gratuita `parallelum.com.br/fipe/api/v1` (sem chave).
//!
//! A consulta não segura o banco durante a rede: lê os códigos do veículo, baixa o preço e
//! grava uma avaliação com fonte `fipe` (com desfazer). Os testes usam respostas gravadas;
//! só o teste `#[ignore]` fala com a rede.

use std::time::Duration;

use rusqlite::{OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::FipeVehicleType;
use crate::debt::kind_str;
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::ids::new_id;
use crate::time::{fmt_date, now_utc, today_local};

pub const BASE_URL: &str = "https://parallelum.com.br/fipe/api/v1";
const USER_AGENT: &str = concat!("Treasury/", env!("CARGO_PKG_VERSION"));

/// Opção de marca, modelo ou ano-combustível.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FipeOption {
    pub code: String,
    pub name: String,
}

/// Preço médio de um veículo no mês de referência.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FipeQuote {
    #[ts(type = "number")]
    pub value: i64,
    pub brand: String,
    pub model: String,
    pub model_year: i32,
    pub fuel: String,
    pub fipe_code: String,
    /// Ex.: "outubro de 2026".
    pub reference_month: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FipeOptionsArgs {
    pub vehicle_type: FipeVehicleType,
    /// Sem marca: lista as marcas; com marca: os modelos; com modelo: os anos.
    #[ts(optional)]
    pub brand_code: Option<String>,
    #[ts(optional)]
    pub model_code: Option<String>,
}

fn path_type(t: FipeVehicleType) -> String {
    kind_str(&t)
}

fn safe(code: &str) -> Result<&str> {
    let c = code.trim();
    if c.is_empty() || !c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-') {
        return Err(Error::invalid(t_args(
            "nucleo-carteira-fipe-erro-codigo",
            &i18n::args(&[("codigo", code.to_string().into())]),
        )));
    }
    Ok(c)
}

/// URL da lista de marcas, modelos (com marca) ou anos (com marca e modelo).
pub fn options_url(a: &FipeOptionsArgs) -> Result<String> {
    let t = path_type(a.vehicle_type);
    Ok(match (&a.brand_code, &a.model_code) {
        (None, _) => format!("{BASE_URL}/{t}/marcas"),
        (Some(b), None) => format!("{BASE_URL}/{t}/marcas/{}/modelos", safe(b)?),
        (Some(b), Some(m)) => format!(
            "{BASE_URL}/{t}/marcas/{}/modelos/{}/anos",
            safe(b)?,
            safe(m)?
        ),
    })
}

pub fn quote_url(t: FipeVehicleType, brand: &str, model: &str, year: &str) -> Result<String> {
    Ok(format!(
        "{BASE_URL}/{}/marcas/{}/modelos/{}/anos/{}",
        path_type(t),
        safe(brand)?,
        safe(model)?,
        safe(year)?
    ))
}

fn code_of(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Lista de opções: `[{codigo, nome}]` (marcas, anos) ou `{modelos: [...], anos: [...]}`.
pub fn parse_options(body: &str) -> Result<Vec<FipeOption>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|_| Error::invalid(t("nucleo-carteira-fipe-erro-ilegivel")))?;
    let arr = match &v {
        Value::Array(a) => a,
        Value::Object(o) => o
            .get("modelos")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::invalid(t("nucleo-carteira-fipe-erro-sem-modelos")))?,
        _ => return Err(Error::invalid(t("nucleo-carteira-fipe-erro-inesperada"))),
    };
    arr.iter()
        .map(|o| {
            Ok(FipeOption {
                code: o
                    .get("codigo")
                    .and_then(code_of)
                    .ok_or_else(|| Error::invalid(t("nucleo-carteira-fipe-erro-sem-codigo")))?,
                name: o
                    .get("nome")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            })
        })
        .collect()
}

/// Preço de um veículo (`{Valor: "R$ 84.753,00", CodigoFipe, MesReferencia, …}`).
pub fn parse_quote(body: &str) -> Result<FipeQuote> {
    let v: Value = serde_json::from_str(body)
        .map_err(|_| Error::invalid(t("nucleo-carteira-fipe-erro-ilegivel")))?;
    if let Some(err) = v.get("error").and_then(Value::as_str) {
        return Err(Error::invalid(t_args(
            "nucleo-carteira-fipe-erro",
            &i18n::args(&[("erro", err.to_string().into())]),
        )));
    }
    let s = |k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let price = s("Valor");
    if price.is_empty() {
        return Err(Error::invalid(t("nucleo-carteira-fipe-erro-sem-preco")));
    }
    Ok(FipeQuote {
        value: crate::money::parse_amount(&price, "BRL")?,
        brand: s("Marca"),
        model: s("Modelo"),
        model_year: v.get("AnoModelo").and_then(Value::as_i64).unwrap_or(0) as i32,
        fuel: s("Combustivel"),
        fipe_code: s("CodigoFipe"),
        reference_month: s("MesReferencia"),
    })
}

fn get_text(url: &str) -> std::result::Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .with_config()
        .limit(8 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())
}

fn network(e: String) -> Error {
    Error::invalid(t_args(
        "nucleo-carteira-fipe-erro-rede",
        &i18n::args(&[("erro", e.into())]),
    ))
}

/// Marcas, modelos ou anos (rede).
pub fn fetch_options(a: &FipeOptionsArgs) -> Result<Vec<FipeOption>> {
    parse_options(&get_text(&options_url(a)?).map_err(network)?)
}

pub fn fetch_quote(t: FipeVehicleType, brand: &str, model: &str, year: &str) -> Result<FipeQuote> {
    parse_quote(&get_text(&quote_url(t, brand, model, year)?).map_err(network)?)
}

/// Grava a cotação como avaliação de hoje (substitui uma consulta FIPE do mesmo dia) e,
/// se o bem ainda não tinha, guarda o código FIPE. Recusa códigos que não conferem.
pub fn apply_quote(tx: &Transaction<'_>, asset_id: &str, q: &FipeQuote) -> Result<()> {
    let expected: Option<String> = tx.query_row(
        "SELECT fipe_code FROM physical_asset WHERE id = ?1",
        [asset_id],
        |r| r.get(0),
    )?;
    if let Some(e) = expected.as_deref().filter(|e| !e.is_empty())
        && !q.fipe_code.is_empty()
        && e != q.fipe_code
    {
        return Err(Error::invalid(t_args(
            "nucleo-carteira-fipe-erro-codigo-diferente",
            &i18n::args(&[
                ("esperado", e.to_string().into()),
                ("consulta", q.fipe_code.clone().into()),
            ]),
        )));
    }
    let now = now_utc();
    if expected.is_none() && !q.fipe_code.is_empty() {
        tx.execute(
            "UPDATE physical_asset SET fipe_code = ?2, updated_at = ?3 WHERE id = ?1",
            params![asset_id, q.fipe_code, now],
        )?;
    }
    let today = fmt_date(today_local());
    let notes = format!(
        "FIPE {} · {} · {} {} {}",
        q.reference_month, q.fipe_code, q.brand, q.model, q.model_year
    );
    let same_day: Option<String> = tx
        .query_row(
            "SELECT id FROM physical_valuation WHERE asset_id = ?1 AND date = ?2 AND source = 'fipe'",
            params![asset_id, today],
            |r| r.get(0),
        )
        .optional()?;
    match same_day {
        Some(id) => {
            tx.execute(
                "UPDATE physical_valuation SET value = ?2, notes = ?3, updated_at = ?4 WHERE id = ?1",
                params![id, q.value, notes, now],
            )?;
        }
        None => {
            tx.execute(
                "INSERT INTO physical_valuation (id, asset_id, date, value, source, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'fipe', ?5, ?6, ?6)",
                params![new_id(), asset_id, today, q.value, notes, now],
            )?;
        }
    }
    Ok(())
}

/// Consulta a FIPE para o bem e grava a avaliação (só o banco fica travado nas pontas).
pub fn refresh(api: &crate::Api, asset_id: &str) -> Result<super::PhysicalAssetView> {
    api.with_session(|s, _| {
        crate::sources::ensure_enabled(s.db.conn(), crate::sources::SourceId::Fipe)
    })?;
    let (t, b, m, y, _) = api.with_session(|s, _| super::fipe_ref(s.db.conn(), asset_id))?;
    let q = fetch_quote(t, &b, &m, &y)?;
    api.with_session(|s, _| {
        crate::changes::record(
            s.db.conn_mut(),
            &crate::changes::Author::user(),
            &t_args(
                "nucleo-carteira-fipe-motivo",
                &i18n::args(&[("mes", q.reference_month.clone().into())]),
            ),
            |tx| apply_quote(tx, asset_id, &q),
        )?;
        super::view(s.db.conn(), asset_id, &super::PhysicalReadArgs::default())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRANDS: &str =
        r#"[{"codigo":"1","nome":"Acura"},{"codigo":"59","nome":"VW - VolksWagen"}]"#;
    const MODELS: &str = r#"{"modelos":[{"codigo":5940,"nome":"AMAROK High.CD 2.0 16V TDI 4x4 Dies. Aut"}],"anos":[{"codigo":"2014-3","nome":"2014 Diesel"}]}"#;
    const YEARS: &str =
        r#"[{"codigo":"2022-3","nome":"2022 Diesel"},{"codigo":"2014-3","nome":"2014 Diesel"}]"#;
    const QUOTE: &str = r#"{"TipoVeiculo":1,"Valor":"R$ 84.753,00","Marca":"VW - VolksWagen","Modelo":"AMAROK High.CD 2.0 16V TDI 4x4 Dies. Aut","AnoModelo":2014,"Combustivel":"Diesel","CodigoFipe":"005340-6","MesReferencia":"outubro de 2026","SiglaCombustivel":"D"}"#;

    #[test]
    fn parses_recorded_responses() {
        let b = parse_options(BRANDS).unwrap();
        assert_eq!(
            b[1],
            FipeOption {
                code: "59".into(),
                name: "VW - VolksWagen".into()
            }
        );
        let m = parse_options(MODELS).unwrap();
        assert_eq!(m[0].code, "5940", "código numérico vira texto");
        assert_eq!(parse_options(YEARS).unwrap()[1].code, "2014-3");
        let q = parse_quote(QUOTE).unwrap();
        assert_eq!(q.value, 8_475_300);
        assert_eq!(q.fipe_code, "005340-6");
        assert_eq!(q.model_year, 2014);
        assert_eq!(q.reference_month, "outubro de 2026");
        assert!(parse_quote(r#"{"error":"Nenhum veículo encontrado"}"#).is_err());
        assert!(parse_quote("<html>").is_err());
    }

    #[test]
    fn urls_are_built_from_safe_codes() {
        let a = FipeOptionsArgs {
            vehicle_type: FipeVehicleType::Carros,
            brand_code: Some("59".into()),
            model_code: Some("5940".into()),
        };
        assert_eq!(
            options_url(&a).unwrap(),
            "https://parallelum.com.br/fipe/api/v1/carros/marcas/59/modelos/5940/anos"
        );
        assert_eq!(
            quote_url(FipeVehicleType::Motos, "80", "123", "2020-1").unwrap(),
            "https://parallelum.com.br/fipe/api/v1/motos/marcas/80/modelos/123/anos/2020-1"
        );
        assert!(quote_url(FipeVehicleType::Carros, "../x", "1", "1").is_err());
    }

    #[test]
    #[ignore = "rede: consulta a API pública da FIPE"]
    fn live_quote() {
        let q = fetch_quote(FipeVehicleType::Carros, "59", "5940", "2014-3").unwrap();
        assert!(q.value > 0 && !q.fipe_code.is_empty(), "{q:?}");
        let years = fetch_options(&FipeOptionsArgs {
            vehicle_type: FipeVehicleType::Carros,
            brand_code: Some("59".into()),
            model_code: Some("5940".into()),
        })
        .unwrap();
        assert!(!years.is_empty());
    }
}
