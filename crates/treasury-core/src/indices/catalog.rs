//! Catálogo de países (tabelas `country_profile` e `country_series`, migração V84): qual
//! série de inflação, de juros de referência e de câmbio vale para cada país, e o país de
//! referência da família (ajuste `indices.reference_country`; sem ele, o país da família ou,
//! fora do catálogo, a moeda base).
//!
//! O Brasil continua exatamente como antes: IPCA, CDI/Selic e PTAX. Outros países acrescentam
//! as próprias séries à coleta e ao deflator "inflação do país".

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::providers::DataProvider;
use crate::error::{Error, Result};

/// Ajuste com o país de referência (ISO 3166-1 alfa-2, ou `EA` para a zona do euro).
pub const COUNTRY_SETTING: &str = "indices.reference_country";

/// País padrão para uma moeda base, quando o usuário não escolheu.
pub fn default_country_for_currency(currency: &str) -> &'static str {
    match currency {
        "USD" => "US",
        "EUR" => "EA",
        "CAD" => "CA",
        "GBP" => "GB",
        "JPY" => "JP",
        "CHF" => "CH",
        "AUD" => "AU",
        "MXN" => "MX",
        "ARS" => "AR",
        _ => "BR",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CatalogSeries {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub unit: String,
    pub frequency: String,
    pub source: String,
    pub provider: Option<DataProvider>,
    /// Prioridade no papel: 0 é a principal, as demais são alternativas quando falta dado.
    #[ts(type = "number")]
    pub priority: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CountryProfile {
    pub country: String,
    pub name: String,
    pub currency: String,
    /// Fonte do câmbio: `bcb_ptax`, `ecb`, `boc` ou `boe`.
    pub fx_provider: String,
    pub notes: Option<String>,
    pub inflation: Vec<CatalogSeries>,
    pub rates: Vec<CatalogSeries>,
    pub fx: Vec<CatalogSeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CountryCatalog {
    /// País de referência em uso.
    pub country: String,
    /// `true` quando veio da moeda base (o usuário não escolheu).
    pub automatic: bool,
    pub base_currency: String,
    pub countries: Vec<CountryProfile>,
}

fn base_currency(conn: &Connection) -> Result<String> {
    Ok(conn
        .query_row("SELECT base_currency FROM household LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .optional()?
        .unwrap_or_else(|| "BRL".into()))
}

fn exists(conn: &Connection, country: &str) -> Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM country_profile WHERE country = ?1",
            [country],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// País de referência e se ele foi deduzido da moeda base.
pub fn reference_country(conn: &Connection) -> Result<(String, bool)> {
    if let Some(c) = crate::settings::get::<String>(conn, COUNTRY_SETTING)?
        && exists(conn, &c)?
    {
        return Ok((c, false));
    }
    // O país da família (docs/countries.md). Um cofre do Brasil com outra moeda base (de antes
    // do país existir) continua seguindo a moeda.
    let family = crate::country::family_country(conn);
    let currency = base_currency(conn)?;
    if (family != "BR" || currency == "BRL") && exists(conn, &family)? {
        return Ok((family, true));
    }
    let c = default_country_for_currency(&currency);
    Ok((c.to_string(), true))
}

/// Escolhe o país de referência (`None` volta ao automático).
pub fn set_reference_country(conn: &Connection, country: Option<&str>) -> Result<String> {
    match country.map(|c| c.trim().to_ascii_uppercase()) {
        None => crate::settings::delete(conn, COUNTRY_SETTING)?,
        Some(c) if c.is_empty() => crate::settings::delete(conn, COUNTRY_SETTING)?,
        Some(c) => {
            if !exists(conn, &c)? {
                return Err(Error::invalid(crate::i18n::t_args(
                    "nucleo-ativos-erro-pais-fora",
                    &crate::i18n::args(&[("pais", c.into())]),
                )));
            }
            crate::settings::set(conn, COUNTRY_SETTING, &c)?;
        }
    }
    Ok(reference_country(conn)?.0)
}

fn series_of(conn: &Connection, country: &str) -> Result<BTreeMap<String, Vec<CatalogSeries>>> {
    let mut stmt = conn.prepare(
        "SELECT cs.role, s.id, s.name, s.kind, s.unit, s.frequency, s.source, cs.priority
         FROM country_series cs JOIN index_series s ON s.id = cs.series_id
         WHERE cs.country = ?1 ORDER BY cs.role, cs.priority, s.id",
    )?;
    let rows = stmt
        .query_map([country], |r| {
            let source: String = r.get(6)?;
            Ok((
                r.get::<_, String>(0)?,
                CatalogSeries {
                    id: r.get(1)?,
                    name: r.get(2)?,
                    kind: r.get(3)?,
                    unit: r.get(4)?,
                    frequency: r.get(5)?,
                    provider: DataProvider::of_source(&source).map(|(p, _)| p),
                    source,
                    priority: r.get(7)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut out: BTreeMap<String, Vec<CatalogSeries>> = BTreeMap::new();
    for (role, s) in rows {
        out.entry(role).or_default().push(s);
    }
    Ok(out)
}

/// Nome do país no idioma da instalação (o gravado no catálogo é o nome em pt-BR).
fn country_label(code: &str, stored: String) -> String {
    use crate::i18n::{Locale, has_key};
    if code == "EA" {
        crate::i18n::t("ajustes-pais-zona-euro")
    } else if has_key(
        Locale::PtBr,
        &format!("nucleo-pais-nome-{}", code.to_ascii_lowercase()),
    ) {
        crate::country::country_name(code)
    } else {
        stored
    }
}

/// Nota do país no idioma da instalação: as notas que o catálogo grava (V84) têm tradução;
/// outra nota passa como está.
fn country_note(note: Option<String>) -> Option<String> {
    use crate::i18n::{Locale, t, tr};
    const NOTES: [&str; 3] = [
        "nucleo-ativos-pais-nota-br",
        "nucleo-ativos-pais-nota-us",
        "nucleo-ativos-pais-nota-banco-mundial",
    ];
    let note = note?;
    Some(
        NOTES
            .iter()
            .find(|k| tr(Locale::PtBr, k, None) == note)
            .map_or(note, |k| t(k)),
    )
}

pub fn profile(conn: &Connection, country: &str) -> Result<Option<CountryProfile>> {
    let row: Option<(String, String, String, String, Option<String>)> = conn
        .query_row(
            "SELECT country, name, currency, fx_provider, notes FROM country_profile WHERE country = ?1",
            [country],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?;
    let Some((country, name, currency, fx_provider, notes)) = row else {
        return Ok(None);
    };
    let mut s = series_of(conn, &country)?;
    Ok(Some(CountryProfile {
        inflation: s.remove("inflation").unwrap_or_default(),
        rates: s.remove("rate").unwrap_or_default(),
        fx: s.remove("fx").unwrap_or_default(),
        name: country_label(&country, name),
        country,
        currency,
        fx_provider,
        notes: country_note(notes),
    }))
}

pub fn catalog(conn: &Connection) -> Result<CountryCatalog> {
    let (country, automatic) = reference_country(conn)?;
    let mut stmt = conn.prepare("SELECT country FROM country_profile")?;
    let codes: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut countries = vec![];
    for c in codes {
        if let Some(p) = profile(conn, &c)? {
            countries.push(p);
        }
    }
    // Em ordem alfabética do nome no idioma (sem os acentos contarem).
    countries.sort_by_cached_key(|p| crate::import::text::fold(&p.name));
    Ok(CountryCatalog {
        country,
        automatic,
        base_currency: base_currency(conn)?,
        countries,
    })
}

/// Séries de inflação do país de referência, em ordem de prioridade: (id, kind).
pub fn inflation_series(conn: &Connection) -> Result<Vec<(String, String)>> {
    let (country, _) = reference_country(conn)?;
    let mut stmt = conn.prepare(
        "SELECT s.id, s.kind FROM country_series cs JOIN index_series s ON s.id = cs.series_id
         WHERE cs.country = ?1 AND cs.role = 'inflation' ORDER BY cs.priority",
    )?;
    let rows = stmt
        .query_map(params![country], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Juro de referência do país de referência: (nome, série) da primeira série de juros em
/// taxa anual (`annual_rate`) do catálogo.
pub fn reference_rate(conn: &Connection) -> Result<Option<(String, String)>> {
    let (country, _) = reference_country(conn)?;
    Ok(conn
        .query_row(
            "SELECT s.name, s.id FROM country_series cs JOIN index_series s ON s.id = cs.series_id
              WHERE cs.country = ?1 AND cs.role = 'rate' AND s.kind = 'annual_rate'
              ORDER BY cs.priority LIMIT 1",
            params![country],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?)
}

/// Séries que a coleta busca por causa do país de referência (além das de sempre).
pub fn sync_scope(conn: &Connection) -> Result<Vec<String>> {
    let (country, _) = reference_country(conn)?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT series_id FROM country_series WHERE country = ?1 ORDER BY series_id",
    )?;
    let rows = stmt
        .query_map([country], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn catalog_covers_countries_and_defaults_to_brazil() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        assert_eq!(reference_country(c).unwrap(), ("BR".into(), true));
        let br = profile(c, "BR").unwrap().unwrap();
        assert_eq!(br.inflation.len(), 1);
        assert_eq!(br.inflation[0].id, "ipca");
        assert_eq!(br.fx_provider, "bcb_ptax");
        assert_eq!(
            inflation_series(c).unwrap(),
            vec![("ipca".into(), "monthly_rate".into())]
        );

        let cat = catalog(c).unwrap();
        assert!(cat.countries.len() >= 60, "{}", cat.countries.len());
        // Todo país tem ao menos uma inflação; toda série do catálogo tem fonte conhecida.
        for p in &cat.countries {
            assert!(!p.inflation.is_empty(), "{} sem inflação", p.country);
            for s in p.inflation.iter().chain(&p.rates).chain(&p.fx) {
                assert!(s.provider.is_some(), "{}: fonte {}", s.id, s.source);
            }
        }
        let pt = profile(c, "PT").unwrap().unwrap();
        assert_eq!(pt.inflation[0].id, "hicp_pt");
        assert_eq!(pt.inflation[1].id, "wb_cpi_pt");
        assert_eq!(pt.rates[0].id, "estr");
        assert!(pt.fx.iter().any(|s| s.id == "fx_eur_usd"));
        let us = profile(c, "US").unwrap().unwrap();
        assert_eq!(
            us.inflation
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            vec!["us_cpi", "cpi_us_oecd", "wb_cpi_us"]
        );

        assert_eq!(set_reference_country(c, Some("pt")).unwrap(), "PT");
        assert_eq!(reference_country(c).unwrap(), ("PT".into(), false));
        let scope = sync_scope(c).unwrap();
        assert!(scope.contains(&"hicp_pt".to_string()));
        assert!(scope.contains(&"estr".to_string()));
        assert!(!scope.contains(&"cpi_jp".to_string()));
        assert!(set_reference_country(c, Some("XX")).is_err());
        assert_eq!(set_reference_country(c, None).unwrap(), "BR");
        assert!(
            sync_scope(c)
                .unwrap()
                .iter()
                // `ipca_g8`: a inflação da educação (meta de faculdade, V148).
                .all(|s| ["ipca", "cdi", "selic_meta", "ipca_g8"].contains(&s.as_str()))
        );
        assert_eq!(default_country_for_currency("EUR"), "EA");
    }

    #[test]
    fn country_names_and_notes_follow_the_locale() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        // Em pt-BR, o nome e a nota são os gravados no catálogo.
        let stored: (String, Option<String>) = c
            .query_row(
                "SELECT name, notes FROM country_profile WHERE country = 'NZ'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        let nz = profile(c, "NZ").unwrap().unwrap();
        assert_eq!((nz.name, nz.notes), stored);
        assert_eq!(profile(c, "EA").unwrap().unwrap().name, "Zona do euro");
        crate::i18n::set_current(crate::i18n::Locale::En);
        let nz = profile(c, "NZ").unwrap().unwrap();
        assert_eq!(nz.name, "New Zealand");
        assert!(
            nz.notes
                .unwrap()
                .starts_with("Annual inflation from the World Bank")
        );
        let cat = catalog(c).unwrap();
        let names: Vec<&str> = cat.countries.iter().map(|p| p.name.as_str()).collect();
        assert!(
            names.contains(&"Germany") && names.contains(&"Euro area"),
            "{names:?}"
        );
        let mut sorted = names.clone();
        sorted.sort_by_key(|n| crate::import::text::fold(n));
        assert_eq!(names, sorted);
        crate::i18n::set_current(crate::i18n::Locale::PtBr);
    }

    #[test]
    fn provider_errors_keep_the_portuguese_article() {
        use crate::indices::providers::bad_response;
        assert_eq!(
            bad_response("FRED", "x").message(),
            "resposta da FRED inválida: x"
        );
        assert_eq!(
            bad_response("Eurostat", "x").message(),
            "resposta do Eurostat inválida: x"
        );
        crate::i18n::set_current(crate::i18n::Locale::En);
        assert_eq!(
            bad_response("FRED", "x").message(),
            "invalid response from FRED: x"
        );
        let p = crate::indices::providers::DataProvider::WorldBank.info();
        assert!(p.name.starts_with("World Bank"), "{}", p.name);
        crate::i18n::set_current(crate::i18n::Locale::PtBr);
    }
}
