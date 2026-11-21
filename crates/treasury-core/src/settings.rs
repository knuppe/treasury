//! Configurações guardadas dentro do banco (tabela `setting`, valores JSON).
//! Não passam pelo histórico de alterações.

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::Result;
use crate::time::now_utc;

pub fn get<T: DeserializeOwned>(conn: &Connection, key: &str) -> Result<Option<T>> {
    let raw: Option<String> = conn
        .query_row("SELECT value FROM setting WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?;
    Ok(match raw {
        Some(s) => Some(serde_json::from_str(&s)?),
        None => None,
    })
}

pub fn get_or<T: DeserializeOwned>(conn: &Connection, key: &str, default: T) -> Result<T> {
    Ok(get(conn, key)?.unwrap_or(default))
}

pub fn set<T: Serialize + ?Sized>(conn: &Connection, key: &str, value: &T) -> Result<()> {
    conn.execute(
        "INSERT INTO setting (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, serde_json::to_string(value)?, now_utc()],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM setting WHERE key = ?1", [key])?;
    Ok(())
}
