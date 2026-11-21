//! Lista oficial de instituições do Banco Central (participantes do STR, com ISPB e código
//! COMPE), usada para escolher o banco ao cadastrar uma conta e para reconhecer o banco de um
//! OFX (`BANKID`). O app traz uma cópia (`bcb_participantes_str.csv`) e a atualiza do site do
//! Banco Central uma vez por semana, com o cofre aberto.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::time::now_utc;

pub const SOURCE_URL: &str =
    "https://www.bcb.gov.br/content/estabilidadefinanceira/str1/ParticipantesSTR.csv";
const BUNDLED: &str = include_str!("bcb_participantes_str.csv");
/// Ajuste técnico: quando a lista foi atualizada pela última vez (do site ou da cópia).
pub const UPDATED_SETTING: &str = "banks.updated_at";
const REFRESH_DAYS: i64 = 7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Bank {
    pub ispb: String,
    /// Código COMPE (3 dígitos), quando a instituição tem.
    pub code: Option<String>,
    pub short_name: String,
    pub full_name: String,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct BankSearchArgs {
    /// Nome, código COMPE ou ISPB.
    #[serde(default)]
    #[ts(optional)]
    pub query: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

/// Lê o CSV oficial (UTF-8 com BOM, vírgula, aspas nos nomes com vírgula).
pub fn parse_csv(text: &str) -> Vec<Bank> {
    let text = text.trim_start_matches('\u{feff}');
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut out = vec![];
    for rec in rdr.records().flatten() {
        let get = |i: usize| rec.get(i).map(str::trim).unwrap_or_default().to_string();
        let ispb = get(0);
        if ispb.len() != 8 || !ispb.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let code = get(2);
        let code = (code.len() == 3 && code.chars().all(|c| c.is_ascii_digit())).then_some(code);
        out.push(Bank {
            ispb,
            code,
            short_name: get(1),
            full_name: get(5),
        });
    }
    out
}

fn store(conn: &Connection, banks: &[Bank]) -> Result<usize> {
    if banks.len() < 50 {
        return Err(Error::Other(crate::i18n::t(
            "nucleo-geral-erro-bancos-incompleta",
        )));
    }
    let now = now_utc();
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM bank_registry", [])?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO bank_registry (ispb, code, short_name, full_name, compe, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for b in banks {
            stmt.execute(params![
                b.ispb,
                b.code,
                b.short_name,
                b.full_name,
                b.code.is_some(),
                now
            ])?;
        }
    }
    tx.commit()?;
    Ok(banks.len())
}

/// Carrega a cópia que vem com o app se a tabela estiver vazia.
pub fn ensure_seeded(conn: &Connection) -> Result<()> {
    let n: i64 = conn.query_row("SELECT count(*) FROM bank_registry", [], |r| r.get(0))?;
    if n == 0 {
        store(conn, &parse_csv(BUNDLED))?;
    }
    Ok(())
}

fn fold(s: &str) -> String {
    crate::import::notes::parse::normalize(s)
}

/// Busca por nome (sem acentos), COMPE ou ISPB (bancos com COMPE primeiro).
pub fn search(conn: &Connection, a: &BankSearchArgs) -> Result<Vec<Bank>> {
    ensure_seeded(conn)?;
    let q = a.query.as_deref().map(str::trim).unwrap_or_default();
    let qf = fold(q);
    let qd: String = q.chars().filter(char::is_ascii_digit).collect();
    let limit = a.limit.unwrap_or(30).clamp(1, 500) as usize;
    let mut stmt = conn.prepare(
        "SELECT ispb, code, short_name, full_name FROM bank_registry
          ORDER BY (code IS NULL), CAST(code AS INTEGER), short_name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Bank {
                ispb: r.get(0)?,
                code: r.get(1)?,
                short_name: r.get(2)?,
                full_name: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let numeric = !qd.is_empty() && qd.len() == q.len();
    Ok(rows
        .into_iter()
        .filter(|b| {
            if q.is_empty() {
                return true;
            }
            if numeric {
                return b.ispb == qd
                    || b.code
                        .as_deref()
                        .is_some_and(|c| c.trim_start_matches('0') == qd.trim_start_matches('0'));
            }
            fold(&b.short_name).contains(&qf) || fold(&b.full_name).contains(&qf)
        })
        .take(limit)
        .collect())
}

/// Banco pelo `BANKID` de um OFX (COMPE com ou sem zeros à esquerda, ou ISPB).
pub fn by_ofx_bank_id(conn: &Connection, bank_id: &str) -> Result<Option<Bank>> {
    let digits: String = bank_id.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return Ok(None);
    }
    ensure_seeded(conn)?;
    let code = format!("{:0>3}", digits.trim_start_matches('0'));
    Ok(search(
        conn,
        &BankSearchArgs {
            query: Some(if digits.len() == 8 { digits } else { code }),
            limit: Some(1),
        },
    )?
    .into_iter()
    .next())
}

/// A lista está para ser atualizada (nunca atualizada do site ou há mais de 7 dias)?
pub fn refresh_due(conn: &Connection) -> Result<bool> {
    let last: Option<String> = crate::settings::get(conn, UPDATED_SETTING)?;
    Ok(
        match last.and_then(|l| chrono::DateTime::parse_from_rfc3339(&l).ok()) {
            Some(t) => {
                chrono::Utc::now().signed_duration_since(t) >= chrono::Duration::days(REFRESH_DAYS)
            }
            None => true,
        },
    )
}

fn banks_error(e: impl std::fmt::Display) -> Error {
    Error::Other(crate::i18n::t_args(
        "nucleo-geral-erro-bancos",
        &crate::i18n::args(&[("erro", e.to_string().into())]),
    ))
}

/// Baixa a lista do Banco Central (fora da trava da sessão) e grava.
pub fn refresh(api: &crate::Api) -> Result<usize> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(30)))
        .build()
        .into();
    let body = agent
        .get(SOURCE_URL)
        .header("User-Agent", "Treasury (gestão financeira local)")
        .call()
        .map_err(banks_error)?
        .body_mut()
        .read_to_string()
        .map_err(banks_error)?;
    let banks = parse_csv(&body);
    api.with_session(|s, _| {
        let n = store(s.db.conn(), &banks)?;
        crate::settings::set(s.db.conn(), UPDATED_SETTING, &now_utc())?;
        Ok(n)
    })
}

/// Rotina: atualiza se devida (erros de rede não interrompem nada).
pub fn refresh_if_due(api: &crate::Api) {
    let due = api
        .with_session(|s, _| {
            Ok(
                crate::sources::enabled(s.db.conn(), crate::sources::SourceId::Bcb)
                    && refresh_due(s.db.conn())?,
            )
        })
        .unwrap_or(false);
    if due && let Err(e) = refresh(api) {
        tracing::warn!("{e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_list_parses_and_finds_banks_by_code_name_and_ofx_id() {
        let banks = parse_csv(BUNDLED);
        assert!(banks.len() > 300, "{}", banks.len());
        let itau = banks
            .iter()
            .find(|b| b.code.as_deref() == Some("341"))
            .unwrap();
        assert!(itau.short_name.contains("ITA"));
        // Nomes com vírgula entre aspas não quebram as colunas.
        assert!(banks.iter().all(|b| b.ispb.len() == 8));
    }
}
