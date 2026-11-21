//! Arrecadação da cidade da família (Brasil): os tributos que o município arrecadou nos últimos
//! 12 meses, por tributo e por habitante, lidos do relatório que a prefeitura entrega ao
//! Tesouro Nacional (Siconfi, RREO, Anexo 03). A lista de municípios vem do IBGE.
//!
//! Só existe em cofre do Brasil. A coleta acontece quando a pessoa pede (`refresh`), nunca numa
//! leitura: o último resultado fica guardado no cofre com a data, e a leitura só o devolve. Os
//! pedidos levam o código do município (ou a UF, na lista) e nada da família.

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::sources::SourceId;

const CITY: &str = "tax_burden.city";
const REVENUE: &str = "tax_burden.city_revenue";

/// Coluna do Anexo 03 com a soma dos últimos 12 meses.
const TOTAL_COLUMN: &str = "TOTAL (ÚLTIMOS 12 MESES)";
/// Contas do Anexo 03 (`cod_conta`): o total de impostos, taxas e contribuições de melhoria e
/// as partes dele.
const TOTAL_ACCOUNT: &str = "ReceitaTributariaLiquidaExcetoTransferenciasEFUNDEB";
const PARTS: &[(&str, &str)] = &[
    ("iptu", "IPTULiquidoExcetoTransferenciasEFUNDEB"),
    ("iss", "ISSLiquidoExcetoTransferenciasEFUNDEB"),
    ("itbi", "ITBILiquidoExcetoTransferenciasEFUNDEB"),
    ("irrf", "IRRFLiquidoExcetoTransferenciasEFUNDEB"),
    ("outros", "OutrasReceitasTributarias"),
];

/// Um município (código do IBGE de 7 dígitos).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct City {
    pub code: String,
    pub name: String,
    pub uf: String,
}

/// Um tributo municipal e o que ele rendeu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CityRevenueLine {
    /// `iptu`, `iss`, `itbi`, `irrf` (o imposto de renda retido dos servidores, que fica com o
    /// município) ou `outros` (taxas e contribuições de melhoria).
    pub key: String,
    pub label: String,
    #[ts(type = "number")]
    pub amount: i64,
    #[ts(type = "number | null")]
    pub per_capita: Option<i64>,
}

/// O que a cidade arrecadou em tributos municipais nos últimos 12 meses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CityRevenue {
    pub city: City,
    /// Exercício e bimestre do relatório (1 a 6).
    pub year: i32,
    pub bimester: u32,
    /// Último mês dos 12 somados (`YYYY-MM`).
    pub until: String,
    /// Impostos, taxas e contribuições de melhoria (centavos).
    #[ts(type = "number")]
    pub total: i64,
    pub lines: Vec<CityRevenueLine>,
    /// População do município no relatório (a estimativa do IBGE que o Tesouro usa).
    #[ts(type = "number | null")]
    pub population: Option<i64>,
    #[ts(type = "number | null")]
    pub per_capita: Option<i64>,
    /// Dia da coleta (`YYYY-MM-DD`).
    pub fetched: String,
    /// Endereço consultado, para conferir.
    pub source_url: String,
}

/// A cidade escolhida e o último resultado guardado (se for dela).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CityState {
    pub city: Option<City>,
    pub revenue: Option<CityRevenue>,
    /// A fonte do Tesouro está ligada em Ajustes › Fontes de dados.
    pub source_enabled: bool,
}

fn ensure_brazil(conn: &Connection) -> Result<()> {
    if crate::country::is_brazil(conn) {
        Ok(())
    } else {
        Err(Error::invalid(t("nucleo-impostos-erro-brasil")))
    }
}

fn label(key: &str) -> String {
    t(&format!("nucleo-impostos-cidade-{key}"))
}

/// A cidade e o resultado guardado. Fora do Brasil, vazio.
pub fn state(conn: &Connection) -> Result<CityState> {
    if !crate::country::is_brazil(conn) {
        return Ok(CityState {
            city: None,
            revenue: None,
            source_enabled: false,
        });
    }
    let city: Option<City> = crate::settings::get(conn, CITY)?;
    let revenue: Option<CityRevenue> = crate::settings::get(conn, REVENUE)?;
    let revenue = revenue
        .filter(|r| city.as_ref().is_some_and(|c| c.code == r.city.code))
        .map(|mut r| {
            // Os nomes dos tributos no idioma de agora.
            for l in &mut r.lines {
                l.label = label(&l.key);
            }
            r
        });
    Ok(CityState {
        city,
        revenue,
        source_enabled: crate::sources::enabled(conn, SourceId::Siconfi),
    })
}

/// Grava (ou limpa) a cidade da família. O resultado guardado de outra cidade deixa de valer.
pub fn set_city(conn: &Connection, city: Option<City>) -> Result<CityState> {
    ensure_brazil(conn)?;
    match city {
        Some(c) => {
            let code = c.code.trim().to_string();
            if code.len() != 7 || !code.bytes().all(|b| b.is_ascii_digit()) {
                return Err(Error::invalid(t("nucleo-impostos-cidade-erro-codigo")));
            }
            let uf = c.uf.trim().to_uppercase();
            let name = c.name.trim().to_string();
            if uf.len() != 2 || name.is_empty() || name.chars().count() > 80 {
                return Err(Error::invalid(t("nucleo-impostos-cidade-erro-codigo")));
            }
            crate::settings::set(conn, CITY, &City { code, name, uf })?;
        }
        None => crate::settings::delete(conn, CITY)?,
    }
    let s = state(conn)?;
    if s.revenue.is_none() {
        crate::settings::delete(conn, REVENUE)?;
    }
    Ok(s)
}

/// Os municípios de uma UF (IBGE), em ordem alfabética.
pub fn cities(conn: &Connection, uf: &str) -> Result<Vec<City>> {
    ensure_brazil(conn)?;
    let uf = uf.trim().to_uppercase();
    if uf.len() != 2 || !uf.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(Error::invalid(t("nucleo-impostos-cidade-erro-uf")));
    }
    crate::sources::ensure_enabled(conn, SourceId::Ibge)?;
    let url =
        format!("https://servicodados.ibge.gov.br/api/v1/localidades/estados/{uf}/municipios");
    let body = get(&url)?;
    parse_cities(&body, &uf)
}

fn get(url: &str) -> Result<String> {
    let agent = crate::indices::sync::agent();
    crate::indices::sync::get_text(&agent, url).map_err(|e| {
        Error::invalid(t_args(
            "nucleo-impostos-cidade-erro-consulta",
            &args(&[("erro", e.into())]),
        ))
    })
}

pub(crate) fn parse_cities(body: &str, uf: &str) -> Result<Vec<City>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|_| Error::invalid(t("nucleo-impostos-cidade-erro-resposta")))?;
    let mut out: Vec<City> = v
        .as_array()
        .ok_or_else(|| Error::invalid(t("nucleo-impostos-cidade-erro-resposta")))?
        .iter()
        .filter_map(|m| {
            let code = match &m["id"] {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => return None,
            };
            let name = m["nome"].as_str()?.trim().to_string();
            (code.len() == 7 && !name.is_empty()).then(|| City {
                code,
                name,
                uf: uf.to_string(),
            })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn rreo_url(code: &str, year: i32, bimester: u32) -> String {
    format!(
        "https://apidatalake.tesouro.gov.br/ords/siconfi/tt/rreo?an_exercicio={year}&nr_periodo={bimester}\
         &co_tipo_demonstrativo=RREO&no_anexo=RREO-Anexo%2003&id_ente={code}"
    )
}

/// Os bimestres a tentar, do mais recente já encerrado para trás (o relatório sai até 30 dias
/// depois do fim do bimestre, e há prefeituras que atrasam).
pub(crate) fn periods(today: NaiveDate) -> Vec<(i32, u32)> {
    let closed = (today.month() - 1) / 2;
    let mut out: Vec<(i32, u32)> = (1..=closed).rev().map(|b| (today.year(), b)).collect();
    out.extend((1..=6).rev().map(|b| (today.year() - 1, b)));
    out.truncate(6);
    out
}

/// Lê o Anexo 03 de um bimestre. `None`: o município ainda não entregou esse relatório.
pub(crate) fn parse_rreo(
    body: &str,
    city: &City,
    year: i32,
    bimester: u32,
    fetched: &str,
    source_url: &str,
) -> Result<Option<CityRevenue>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|_| Error::invalid(t("nucleo-impostos-cidade-erro-resposta")))?;
    let items = v["items"]
        .as_array()
        .ok_or_else(|| Error::invalid(t("nucleo-impostos-cidade-erro-resposta")))?;
    let cents = |account: &str| {
        items
            .iter()
            .find(|r| r["coluna"] == TOTAL_COLUMN && r["cod_conta"] == account)
            .and_then(|r| r["valor"].as_f64())
            .map(|x| (x * 100.0).round() as i64)
    };
    let Some(total) = cents(TOTAL_ACCOUNT).filter(|t| *t > 0) else {
        return Ok(None);
    };
    let population = items
        .iter()
        .find_map(|r| r["populacao"].as_i64())
        .filter(|p| *p > 0);
    let per = |amount: i64| population.map(|p| (amount as f64 / p as f64).round() as i64);
    let lines = PARTS
        .iter()
        .filter_map(|(key, account)| {
            let amount = cents(account)?;
            Some(CityRevenueLine {
                key: (*key).into(),
                label: label(key),
                amount,
                per_capita: per(amount),
            })
        })
        .collect();
    Ok(Some(CityRevenue {
        city: city.clone(),
        year,
        bimester,
        until: format!("{year}-{:02}", bimester * 2),
        total,
        lines,
        population,
        per_capita: per(total),
        fetched: fetched.into(),
        source_url: source_url.into(),
    }))
}

/// Busca no Tesouro o relatório mais recente da cidade escolhida e guarda o resultado.
pub fn refresh(conn: &Connection) -> Result<CityState> {
    ensure_brazil(conn)?;
    let city: City = crate::settings::get(conn, CITY)?
        .ok_or_else(|| Error::invalid(t("nucleo-impostos-cidade-erro-sem-cidade")))?;
    crate::sources::ensure_enabled(conn, SourceId::Siconfi)?;
    let today = crate::time::today_local();
    let fetched = crate::time::fmt_date(today);
    let mut found = None;
    let mut failure = None;
    for (year, bimester) in periods(today) {
        let url = rreo_url(&city.code, year, bimester);
        match get(&url).and_then(|b| parse_rreo(&b, &city, year, bimester, &fetched, &url)) {
            Ok(Some(r)) => {
                found = Some(r);
                break;
            }
            Ok(None) => {}
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    let outcome = match (&found, &failure) {
        (Some(_), _) => Ok(()),
        (None, Some(e)) => Err(e.message()),
        // Sem relatório nos últimos seis bimestres: a fonte respondeu, o dado não existe.
        (None, None) => Ok(()),
    };
    let _ = crate::sources::record_result(conn, SourceId::Siconfi, &outcome, today);
    match (found, failure) {
        (Some(r), _) => crate::settings::set(conn, REVENUE, &r)?,
        (None, Some(e)) => return Err(e),
        (None, None) => {
            return Err(Error::invalid(t_args(
                "nucleo-impostos-cidade-erro-sem-relatorio",
                &args(&[("cidade", city.name.clone().into())]),
            )));
        }
    }
    state(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn city() -> City {
        City {
            code: "9999999".into(),
            name: "Vila Exemplo".into(),
            uf: "SP".into(),
        }
    }

    /// Resposta inventada no formato do Siconfi (duas colunas, cinco contas).
    fn body() -> String {
        let row = |col: &str, account: &str, value: f64| {
            serde_json::json!({"exercicio": 2026, "periodo": 4, "cod_ibge": 9999999, "populacao": 20000,
                "coluna": col, "cod_conta": account, "conta": "x", "valor": value})
        };
        serde_json::json!({"items": [
            row("<MR>", TOTAL_ACCOUNT, 1.0),
            row(TOTAL_COLUMN, TOTAL_ACCOUNT, 30_000_000.50),
            row(TOTAL_COLUMN, "IPTULiquidoExcetoTransferenciasEFUNDEB", 10_000_000.0),
            row(TOTAL_COLUMN, "ISSLiquidoExcetoTransferenciasEFUNDEB", 15_000_000.0),
            row(TOTAL_COLUMN, "ITBILiquidoExcetoTransferenciasEFUNDEB", 2_000_000.0),
            row(TOTAL_COLUMN, "IRRFLiquidoExcetoTransferenciasEFUNDEB", 2_500_000.0),
            row(TOTAL_COLUMN, "OutrasReceitasTributarias", 500_000.50),
            row(TOTAL_COLUMN, "RREO3ReceitaPatrimonial", 9_999_999.0),
        ], "hasMore": false})
        .to_string()
    }

    #[test]
    fn reads_the_twelve_month_total_by_tax_and_per_inhabitant() {
        let r = parse_rreo(&body(), &city(), 2026, 4, "2026-11-18", "https://exemplo")
            .unwrap()
            .unwrap();
        assert_eq!(r.total, 3_000_000_050);
        assert_eq!(r.population, Some(20_000));
        // R$ 30.000.000,50 ÷ 20 mil habitantes = R$ 1.500,00.
        assert_eq!(r.per_capita, Some(150_000));
        assert_eq!(r.until, "2026-08");
        let keys: Vec<&str> = r.lines.iter().map(|l| l.key.as_str()).collect();
        assert_eq!(keys, ["iptu", "iss", "itbi", "irrf", "outros"]);
        assert_eq!(r.lines.iter().map(|l| l.amount).sum::<i64>(), r.total);
        assert_eq!(r.lines[0].per_capita, Some(50_000));
        // Sem o relatório do bimestre: nada.
        let empty = r#"{"items": [], "hasMore": false}"#;
        assert!(
            parse_rreo(empty, &city(), 2026, 5, "2026-11-18", "u")
                .unwrap()
                .is_none()
        );
        assert!(parse_rreo("<html>", &city(), 2026, 5, "2026-11-18", "u").is_err());
    }

    #[test]
    fn tries_the_latest_closed_bimesters_first() {
        let d = |y, m, day| NaiveDate::from_ymd_opt(y, m, day).unwrap();
        assert_eq!(
            periods(d(2026, 11, 18)),
            [
                (2026, 5),
                (2026, 4),
                (2026, 3),
                (2026, 2),
                (2026, 1),
                (2025, 6)
            ]
        );
        assert_eq!(periods(d(2026, 1, 10))[0], (2025, 6));
        assert_eq!(periods(d(2026, 3, 1))[..2], [(2026, 1), (2025, 6)]);
    }

    #[test]
    fn lists_the_cities_of_a_state_sorted_by_name() {
        let body = r#"[{"id": 9999902, "nome": "Vila Exemplo"}, {"id": 9999901, "nome": "Alto Exemplo"},
                       {"id": 12, "nome": "curto demais"}]"#;
        let list = parse_cities(body, "SP").unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Alto Exemplo");
        assert_eq!(list[1].code, "9999902");
        assert!(parse_cities("{}", "SP").is_err());
    }
}
