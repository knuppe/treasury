//! Figura 07 · Inflação dos relatórios: o gasto de cada mês em valor nominal e corrigido pela
//! inflação até o último mês da janela (em dinheiro de hoje), com o índice usado, a data do
//! último dado e os meses que dependem de dado estimado. No Brasil é o IPCA; fora dele, o
//! índice de inflação do país de referência (CPI-U, HICP…, docs/countries.md).
//!
//! Por categoria, compara o gasto médio dos três primeiros meses da janela com o dos três
//! últimos e põe ao lado a inflação entre os meios dessas duas janelas: o que subiu mais que a
//! inflação ficou mais caro de verdade (ou a família gastou mais), o que subiu menos ficou mais
//! barato em termos reais.

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{months_between, normalize_month, shift_month};
use super::monthly_evolution;
use crate::budget::Basis;
use crate::error::Result;
use crate::indices::{self, Deflator};
use crate::time::parse_date;

/// Índice de inflação usado na correção.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InflationIndex {
    pub series_id: String,
    pub name: String,
    /// Fonte (`ibge_sidra:1737`, `fred:CPIAUCSL`…).
    pub source: String,
    /// Data do último dado não estimado da série.
    pub last_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InflationCategory {
    pub key: Option<String>,
    pub label: String,
    pub color: Option<String>,
    /// Média mensal dos três primeiros meses da janela.
    #[ts(type = "number")]
    pub first: i64,
    /// Média mensal dos três últimos meses.
    #[ts(type = "number")]
    pub last: i64,
    /// last ÷ first − 1; null sem gasto no começo.
    pub change: Option<f64>,
    /// Inflação entre os meios das duas janelas (fração).
    pub inflation: Option<f64>,
    /// change − inflation, em pontos (fração); positivo = subiu acima da inflação.
    pub above_inflation: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InflationReport {
    pub months: Vec<String>,
    pub basis: Basis,
    pub currency: String,
    /// Gasto de cada mês (positivo).
    #[ts(type = "number[]")]
    pub nominal: Vec<i64>,
    /// O mesmo gasto corrigido até o fim do último mês; null quando falta o índice.
    #[ts(type = "(number | null)[]")]
    pub real: Vec<Option<i64>>,
    /// A correção do mês usou dado estimado (mês ainda não divulgado ou projeção).
    pub estimated: Vec<bool>,
    /// Inflação acumulada na janela (do início do primeiro mês ao fim do último).
    pub inflation: Option<f64>,
    /// A inflação acumulada usou dado estimado.
    pub inflation_estimated: bool,
    pub index: Option<InflationIndex>,
    pub categories: Vec<InflationCategory>,
    /// Primeiro mês da comparação por categoria (o primeiro com gasto); null sem meses
    /// suficientes.
    pub compare_from: Option<String>,
    pub missing_fx: Vec<String>,
    /// Alguma conversão de moeda usou câmbio estimado.
    pub fx_estimated: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InflationArgs {
    /// Meses na janela (padrão 12).
    #[ts(optional)]
    pub months: Option<u32>,
    /// Último mês (padrão: o atual).
    #[ts(optional)]
    pub end_month: Option<String>,
    #[ts(optional)]
    pub basis: Option<Basis>,
}

fn month_start(month: &str) -> Result<NaiveDate> {
    parse_date(&format!("{month}-01"))
}

/// Primeiro dia do mês seguinte (fim exclusivo do mês).
fn month_end(month: &str) -> Result<NaiveDate> {
    month_start(&shift_month(month, 1)?)
}

/// Fator de inflação entre duas datas: (1 + inflação), e se usou dado estimado. O IPCA do
/// mês corrente (e às vezes do anterior) ainda não saiu: os meses além do último dado seguem
/// a média das três últimas variações mensais, marcados como estimados. Fora do Brasil, o
/// próprio deflator do país já projeta o fim da série.
fn factor(conn: &Connection, from: NaiveDate, to: NaiveDate) -> Result<Option<(f64, bool)>> {
    if let Some(f) = indices::deflator_factor(conn, Deflator::Ipca, from, to)? {
        return Ok(f.factor.to_f64().map(|x| (x, f.estimated)));
    }
    if !crate::country::is_brazil(conn) {
        return Ok(None);
    }
    let last: Option<String> = conn
        .query_row(
            "SELECT MAX(date) FROM index_value WHERE series_id = 'ipca'",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    let Some(last) = last.and_then(|d| parse_date(&d).ok()) else {
        return Ok(None);
    };
    // Fim exclusivo do último mês publicado.
    let published_end = month_end(&last.format("%Y-%m").to_string())?;
    let (base, mut estimated) = if published_end > from {
        match indices::deflator_factor(conn, Deflator::Ipca, from, published_end.min(to))? {
            Some(f) => (f.factor.to_f64().unwrap_or(1.0), f.estimated),
            None => return Ok(None),
        }
    } else {
        (1.0, false)
    };
    let start = published_end.max(from);
    if to <= start {
        return Ok(Some((base, estimated)));
    }
    let recent = indices::monthly_rates(
        conn,
        "ipca",
        published_end - chrono::Duration::days(95),
        published_end - chrono::Duration::days(1),
    )?;
    let tail: Vec<f64> = recent
        .iter()
        .rev()
        .take(3)
        .filter_map(|r| r.pct.to_f64())
        .collect();
    if tail.is_empty() {
        return Ok(None);
    }
    let monthly = tail.iter().sum::<f64>() / tail.len() as f64 / 100.0;
    let months = (to - start).num_days() as f64 / 30.4375;
    estimated = true;
    Ok(Some((base * (1.0 + monthly).powf(months), estimated)))
}

/// Série que a correção usa: o IPCA no Brasil, senão a primeira de inflação do país.
fn index_info(conn: &Connection) -> Result<Option<InflationIndex>> {
    let id = if crate::country::is_brazil(conn) {
        Some("ipca".to_string())
    } else {
        indices::catalog::inflation_series(conn)?
            .into_iter()
            .next()
            .map(|(id, _)| id)
    };
    let Some(id) = id else { return Ok(None) };
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT name, source FROM index_series WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((name, source)) = row else {
        return Ok(None);
    };
    let last_date: Option<String> = conn
        .query_row(
            "SELECT MAX(date) FROM index_value WHERE series_id = ?1 AND estimated = 0",
            params![id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    Ok(Some(InflationIndex {
        series_id: id,
        name,
        source,
        last_date,
    }))
}

fn avg(v: &[i64]) -> i64 {
    if v.is_empty() {
        return 0;
    }
    super::engine::div_round(v.iter().sum(), v.len() as i64)
}

pub fn inflation(conn: &Connection, a: &InflationArgs, basis: Basis) -> Result<InflationReport> {
    let end = normalize_month(
        &a.end_month
            .clone()
            .unwrap_or_else(super::engine::current_month),
    )?;
    let n = a.months.unwrap_or(12).clamp(3, 36);
    let evo = monthly_evolution(conn, n, &end, 6, basis)?;
    let months = months_between(&shift_month(&end, -(n as i32 - 1))?, &end)?;
    let window_end = month_end(&end)?;

    let mut real = Vec::with_capacity(months.len());
    let mut estimated = Vec::with_capacity(months.len());
    for (m, nominal) in months.iter().zip(&evo.totals) {
        match factor(conn, month_end(m)?, window_end)? {
            Some((f, est)) => {
                real.push(Some((*nominal as f64 * f).round() as i64));
                estimated.push(est);
            }
            None => {
                real.push(None);
                estimated.push(false);
            }
        }
    }
    let total = factor(conn, month_start(&months[0])?, window_end)?;

    // Por categoria: três primeiros × três últimos meses, com a inflação entre os meios. O
    // começo é o primeiro mês com gasto (um cofre novo não compara contra meses vazios), e
    // precisa sobrar pelo menos seis meses dali até o fim.
    let start = evo.totals.iter().position(|v| *v > 0);
    let mut categories = vec![];
    let compare_from = start.filter(|i| months.len() - i >= 6);
    if let Some(i0) = compare_from {
        let mid_first = month_end(&months[i0 + 1])?;
        let mid_last = month_end(&months[months.len() - 2])?;
        let infl = factor(conn, mid_first, mid_last)?.map(|(f, _)| f - 1.0);
        for s in &evo.series {
            let first = avg(&s.values[i0..i0 + 3]);
            let last = avg(&s.values[s.values.len() - 3..]);
            let change = (first > 0).then(|| last as f64 / first as f64 - 1.0);
            categories.push(InflationCategory {
                key: s.key.clone(),
                label: s.label.clone(),
                color: s.color.clone(),
                first,
                last,
                change,
                inflation: infl,
                above_inflation: change.zip(infl).map(|(c, i)| c - i),
            });
        }
    }

    let compare_from_month = compare_from.map(|i| months[i].clone());
    Ok(InflationReport {
        nominal: evo.totals.clone(),
        months,
        basis,
        currency: evo.currency.clone(),
        real,
        estimated,
        inflation: total.map(|(f, _)| f - 1.0),
        inflation_estimated: total.is_some_and(|(_, e)| e),
        index: index_info(conn)?,
        categories,
        compare_from: compare_from_month,
        missing_fx: evo.missing_fx.clone(),
        fx_estimated: evo.estimated,
    })
}
