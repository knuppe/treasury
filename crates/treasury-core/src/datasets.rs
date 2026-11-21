//! Prazo de validade das tabelas embutidas no app (docs/invariants.md). Tabelas anuais, como o
//! piso e o teto do INSS, vêm dentro do binário e só mudam numa versão nova do Treasury. Uma
//! tabela vencida não quebra nada: a conta sai, só que com os valores do ano anterior. Este
//! registro existe para o app **dizer isso** em vez de errar em silêncio.
//!
//! Cada tabela vale até o fim do ano dela. O aviso começa em 1º de fevereiro do ano seguinte,
//! porque as portarias de reajuste saem em janeiro e uma versão nova do app leva alguns dias.
//! Só entram as tabelas do país da família.
//!
//! A estimativa de impostos pagos tem o próprio aviso de dados antigos (`tax_burden`,
//! `months_old`), e as regras do IR são escolhidas pelo ano da apuração (`tax::rules`).

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Uma tabela anual embutida.
pub struct Dataset {
    pub key: &'static str,
    /// Chave do nome no catálogo (`tabelas-*`).
    pub label_key: &'static str,
    /// País da família em que a tabela é usada.
    pub country: &'static str,
    pub year: i32,
    pub source: &'static str,
}

/// As tabelas registradas. O ano e a fonte vêm da própria tabela: atualizar a tabela atualiza
/// o registro.
pub fn all() -> Vec<Dataset> {
    vec![
        Dataset {
            key: "inss",
            label_key: "tabelas-inss",
            country: "BR",
            year: crate::goals::inss::TABLE.year,
            source: crate::goals::inss::TABLE.source,
        },
        Dataset {
            key: "pension_pt",
            label_key: "tabelas-pensao-pt",
            country: "PT",
            year: crate::goals::pension_pt::TABLE.year,
            source: crate::goals::pension_pt::TABLE.source,
        },
        Dataset {
            key: "pension_us",
            label_key: "tabelas-pensao-us",
            country: "US",
            year: crate::goals::pension_us::TABLE.year,
            source: crate::goals::pension_us::TABLE.source,
        },
    ]
}

/// A tabela do ano `year` já deveria ter sido trocada em `today`?
pub fn is_outdated(year: i32, today: NaiveDate) -> bool {
    NaiveDate::from_ymd_opt(year + 1, 2, 1).is_some_and(|limit| today >= limit)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OutdatedTable {
    pub key: String,
    /// Nome no idioma da instalação.
    pub label: String,
    pub year: i32,
    pub source: String,
}

/// As tabelas do país da família que passaram da validade.
pub fn outdated(conn: &Connection, today: NaiveDate) -> Vec<OutdatedTable> {
    let country = crate::country::family_country(conn);
    all()
        .into_iter()
        .filter(|d| d.country == country && is_outdated(d.year, today))
        .map(|d| OutdatedTable {
            key: d.key.into(),
            label: crate::i18n::t(d.label_key),
            year: d.year,
            source: d.source.into(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn a_table_is_valid_through_january_of_the_next_year() {
        assert!(!is_outdated(2026, d(2026, 12, 31)));
        assert!(!is_outdated(2026, d(2027, 1, 31)));
        assert!(is_outdated(2026, d(2027, 2, 1)));
        assert!(is_outdated(2025, d(2027, 6, 1)));
    }

    #[test]
    fn every_registered_table_has_a_name_a_year_and_a_source() {
        let today = crate::time::today_local();
        for t in all() {
            assert!(
                crate::i18n::has_key(crate::i18n::Locale::PtBr, t.label_key),
                "{}",
                t.key
            );
            assert!(!t.source.is_empty(), "{}", t.key);
            // No table may be more than a year ahead of the calendar (typo).
            assert!(t.year <= today.year() + 1, "{}: {}", t.key, t.year);
        }
    }

    #[test]
    fn only_the_tables_of_the_family_country_are_reported() {
        let t = crate::reports::fixture::new_db();
        let conn = t.db.conn();
        // Brazilian family: only INSS, and only after the deadline.
        let year = crate::goals::inss::TABLE.year;
        assert!(outdated(conn, d(year, 12, 1)).is_empty());
        let late = outdated(conn, d(year + 1, 3, 1));
        assert_eq!(late.len(), 1, "{late:?}");
        assert_eq!(late[0].key, "inss");
        assert_eq!(late[0].year, year);
        assert_eq!(late[0].label, "INSS (piso e teto)");
    }
}
