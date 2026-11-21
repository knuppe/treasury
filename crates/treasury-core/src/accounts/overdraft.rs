//! Cheque especial (limite de crédito) da conta corrente: o limite e as condições do banco
//! (juros ao mês e ao ano, desconto, IOF básico e adicional, CET) e, a partir do saldo, o
//! quanto do limite está em uso e uma **estimativa** do custo de 30 dias nesse uso:
//!
//! - juros = usado × taxa ao mês × (1 − desconto);
//! - IOF = usado × (alíquota básica ao dia × 30 + alíquota adicional).
//!
//! É estimativa ("medido, nunca inventado" vale para o saldo; o custo real depende de quantos
//! dias o saldo fica negativo e de como o banco capitaliza): a interface diz isso.

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::debt::{LoanIndexer, split_rate_index};
use crate::error::{Error, Result};
use crate::i18n;
use crate::time::now_utc;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OverdraftTerms {
    /// Limite, em centavos.
    #[ts(type = "number")]
    pub limit: i64,
    /// Taxa de juros ao mês (%), decimal em texto ("8,5" ou "8.5").
    #[serde(default)]
    pub monthly_rate: Option<String>,
    /// Percentual de desconto sobre a taxa (%).
    #[serde(default)]
    pub discount_pct: Option<String>,
    /// Taxa de juros cobrada ao ano (%).
    #[serde(default)]
    pub annual_rate: Option<String>,
    /// IOF alíquota básica, ao dia (%), ex.: "0,0082".
    #[serde(default)]
    pub iof_daily_rate: Option<String>,
    /// IOF alíquota adicional (%), ex.: "0,38".
    #[serde(default)]
    pub iof_additional_rate: Option<String>,
    /// CET ao mês (%).
    #[serde(default)]
    pub cet_month: Option<String>,
    /// CET ao ano (%).
    #[serde(default)]
    pub cet_year: Option<String>,
    /// Indexador da taxa pós-fixada ("4,75 + TR"): com ele, as taxas ao ano e ao mês são o
    /// spread sobre o índice. `None` (ou `none`): taxa pré-fixada.
    #[serde(default)]
    #[ts(optional)]
    pub indexer: Option<LoanIndexer>,
}

/// Uso do limite agora e o custo estimado de 30 dias nesse uso.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OverdraftStatus {
    #[ts(type = "number")]
    pub limit: i64,
    /// Parte do limite em uso (saldo negativo), em centavos.
    #[ts(type = "number")]
    pub used: i64,
    /// Limite − usado (negativo = acima do limite).
    #[ts(type = "number")]
    pub available: i64,
    /// Saldo + limite: o que dá para gastar.
    #[ts(type = "number")]
    pub spendable: i64,
    /// Taxa ao mês já com o desconto (%), quando informada. Com indexador, a taxa efetiva
    /// estimada: spread composto com o último valor do índice.
    pub effective_monthly_rate: Option<String>,
    /// Índice usado na taxa efetiva: o último valor (% ao mês) e a data dele.
    pub index: Option<IndexReading>,
    /// Juros estimados de 30 dias no uso atual, em centavos.
    #[ts(type = "number | null")]
    pub estimated_interest: Option<i64>,
    /// IOF estimado de 30 dias no uso atual, em centavos.
    #[ts(type = "number | null")]
    pub estimated_iof: Option<i64>,
    /// A conta não está em dia (`balance_status` diferente de `fresh`): o saldo negativo pode
    /// ser lançamento faltando, e o custo acima é só uma hipótese.
    #[serde(default)]
    pub uncertain: bool,
}

/// Último valor de um indexador, em % ao mês, com a data (o dia da série).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IndexReading {
    pub indexer: LoanIndexer,
    /// % ao mês (nas séries diárias, CDI e Selic, a taxa do dia composta em 21 dias úteis).
    pub monthly_pct: String,
    pub date: String,
    /// Valor projetado (sem dado publicado para a data).
    pub estimated: bool,
}

/// Série do indexador no cofre (`index_series`).
fn series_of(i: LoanIndexer) -> Option<&'static str> {
    Some(match i {
        LoanIndexer::None => return None,
        LoanIndexer::Tr => "tr",
        LoanIndexer::Ipca => "ipca",
        LoanIndexer::Igpm => "igpm",
        LoanIndexer::Inpc => "inpc",
        LoanIndexer::Incc => "incc",
        LoanIndexer::Cdi => "cdi",
        LoanIndexer::Selic => "selic",
        LoanIndexer::Poupanca => "poupanca",
    })
}

/// O último valor do indexador até `today` (séries do BCB e do IBGE já baixadas pelo app).
pub fn index_reading(
    conn: &Connection,
    indexer: Option<LoanIndexer>,
    today: chrono::NaiveDate,
) -> Result<Option<IndexReading>> {
    let Some(i) = indexer else { return Ok(None) };
    let Some(series) = series_of(i) else {
        return Ok(None);
    };
    let from = crate::time::add_months(today, -6);
    let Some(p) = crate::indices::points(conn, series, from, today)?.pop() else {
        return Ok(None);
    };
    let monthly = if i.daily_series().is_some() {
        // % ao dia → % ao mês (21 dias úteis).
        (crate::finance::pow_dec(
            Decimal::ONE + p.value / Decimal::ONE_HUNDRED,
            Decimal::from(21),
        ) - Decimal::ONE)
            * Decimal::ONE_HUNDRED
    } else {
        p.value
    };
    Ok(Some(IndexReading {
        indexer: i,
        monthly_pct: monthly.round_dp(4).normalize().to_string(),
        date: crate::time::fmt_date(p.date),
        estimated: p.estimated,
    }))
}

/// Erro de um campo das condições: `field` é a chave do rótulo do campo no catálogo.
fn field_error(key: &str, field: &str, extra: (&'static str, String)) -> Error {
    Error::invalid(i18n::t_args(
        key,
        &i18n::args(&[("campo", i18n::t(field).into()), (extra.0, extra.1.into())]),
    ))
}

/// `field`: chave do rótulo do campo (`nucleo-contas-cheque-campo-*`), para as mensagens.
fn dec(field: &str, v: &Option<String>) -> Result<Option<Decimal>> {
    let Some(s) = v.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let raw = s.trim_end_matches('%').trim();
    // "1.234,5" (milhar com ponto) ou "8.5": com vírgula, o ponto é de milhar.
    let txt = if raw.contains(',') {
        raw.replace('.', "").replace(',', ".")
    } else {
        raw.to_string()
    };
    let d: Decimal = txt.parse().map_err(|_| {
        field_error(
            "nucleo-contas-cheque-erro-numero",
            field,
            ("valor", s.to_string()),
        )
    })?;
    if d < Decimal::ZERO {
        return Err(Error::invalid(i18n::t_args(
            "nucleo-contas-cheque-erro-negativo",
            &i18n::args(&[("campo", i18n::t(field).into())]),
        )));
    }
    Ok(Some(d))
}

/// Confere e normaliza (decimais com ponto, sem `%`).
pub fn validate(t: &OverdraftTerms) -> Result<OverdraftTerms> {
    if t.limit < 0 {
        return Err(Error::invalid(i18n::t("nucleo-contas-cheque-erro-limite")));
    }
    // "4,75 + TR" na taxa ao ano (ou ao mês): o número é o spread e o TR vira o indexador.
    let mut t = t.clone();
    let mut indexer = t.indexer.filter(|i| *i != LoanIndexer::None);
    for field in [&mut t.annual_rate, &mut t.monthly_rate] {
        if let Some((rate, idx)) = field.as_deref().and_then(split_rate_index) {
            if indexer.is_some_and(|i| i != idx) {
                return Err(Error::invalid(i18n::t(
                    "nucleo-contas-cheque-erro-indexador",
                )));
            }
            indexer = Some(idx);
            *field = Some(rate);
        }
    }
    let t = &t;
    let norm = |field: &str, v: &Option<String>, max: Decimal| -> Result<Option<String>> {
        Ok(match dec(field, v)? {
            Some(d) if d > max => {
                return Err(field_error(
                    "nucleo-contas-cheque-erro-acima",
                    field,
                    ("max", max.to_string()),
                ));
            }
            Some(d) => Some(d.normalize().to_string()),
            None => None,
        })
    };
    let hundred = Decimal::from(100);
    let big = Decimal::from(10_000);
    Ok(OverdraftTerms {
        limit: t.limit,
        monthly_rate: norm(
            "nucleo-contas-cheque-campo-taxa-mes",
            &t.monthly_rate,
            hundred,
        )?,
        discount_pct: norm(
            "nucleo-contas-cheque-campo-desconto",
            &t.discount_pct,
            hundred,
        )?,
        annual_rate: norm("nucleo-contas-cheque-campo-taxa-ano", &t.annual_rate, big)?,
        iof_daily_rate: norm(
            "nucleo-contas-cheque-campo-iof-dia",
            &t.iof_daily_rate,
            Decimal::ONE,
        )?,
        iof_additional_rate: norm(
            "nucleo-contas-cheque-campo-iof-adicional",
            &t.iof_additional_rate,
            hundred,
        )?,
        cet_month: norm("nucleo-contas-cheque-campo-cet-mes", &t.cet_month, hundred)?,
        cet_year: norm("nucleo-contas-cheque-campo-cet-ano", &t.cet_year, big)?,
        indexer,
    })
}

pub fn load(conn: &Connection, account_id: &str) -> Result<Option<OverdraftTerms>> {
    Ok(conn
        .query_row(
            "SELECT limit_amount, monthly_rate, discount_pct, annual_rate, iof_daily_rate,
                    iof_additional_rate, cet_month, cet_year, indexer
               FROM account_overdraft WHERE account_id = ?1",
            [account_id],
            |r| {
                Ok(OverdraftTerms {
                    limit: r.get(0)?,
                    monthly_rate: r.get(1)?,
                    discount_pct: r.get(2)?,
                    annual_rate: r.get(3)?,
                    iof_daily_rate: r.get(4)?,
                    iof_additional_rate: r.get(5)?,
                    cet_month: r.get(6)?,
                    cet_year: r.get(7)?,
                    indexer: serde_json::from_value::<LoanIndexer>(serde_json::Value::String(
                        r.get::<_, String>(8)?,
                    ))
                    .ok()
                    .filter(|i| *i != LoanIndexer::None),
                })
            },
        )
        .optional()?)
}

pub fn upsert(tx: &Transaction<'_>, account_id: &str, t: &OverdraftTerms) -> Result<()> {
    let now = now_utc();
    tx.execute(
        "INSERT INTO account_overdraft (account_id, limit_amount, monthly_rate, discount_pct, annual_rate,
                                        iof_daily_rate, iof_additional_rate, cet_month, cet_year, indexer,
                                        created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?11, ?10, ?10)
         ON CONFLICT (account_id) DO UPDATE SET
            limit_amount = excluded.limit_amount, monthly_rate = excluded.monthly_rate,
            discount_pct = excluded.discount_pct, annual_rate = excluded.annual_rate,
            iof_daily_rate = excluded.iof_daily_rate, iof_additional_rate = excluded.iof_additional_rate,
            cet_month = excluded.cet_month, cet_year = excluded.cet_year, indexer = excluded.indexer,
            updated_at = excluded.updated_at
          WHERE limit_amount IS NOT excluded.limit_amount OR monthly_rate IS NOT excluded.monthly_rate
             OR discount_pct IS NOT excluded.discount_pct OR annual_rate IS NOT excluded.annual_rate
             OR iof_daily_rate IS NOT excluded.iof_daily_rate OR iof_additional_rate IS NOT excluded.iof_additional_rate
             OR cet_month IS NOT excluded.cet_month OR cet_year IS NOT excluded.cet_year
             OR indexer IS NOT excluded.indexer",
        params![
            account_id,
            t.limit,
            t.monthly_rate,
            t.discount_pct,
            t.annual_rate,
            t.iof_daily_rate,
            t.iof_additional_rate,
            t.cet_month,
            t.cet_year,
            now,
            indexer_str(t.indexer)
        ],
    )?;
    Ok(())
}

pub fn remove(tx: &Transaction<'_>, account_id: &str) -> Result<()> {
    tx.execute(
        "DELETE FROM account_overdraft WHERE account_id = ?1",
        [account_id],
    )?;
    Ok(())
}

fn pct_of(amount: i64, pct: Decimal) -> i64 {
    use rust_decimal::prelude::ToPrimitive;
    (Decimal::from(amount) * pct / Decimal::from(100))
        .round()
        .to_i64()
        .unwrap_or(0)
}

fn indexer_str(i: Option<LoanIndexer>) -> String {
    serde_json::to_value(i.unwrap_or(LoanIndexer::None))
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "none".into())
}

/// Taxa ao mês (%) das condições: a informada ou, sem ela, a ao ano convertida. Com
/// indexador, o spread composto com o último valor do índice:
/// (1 + spread ao mês) × (1 + índice ao mês) − 1.
fn monthly_rate(t: &OverdraftTerms, index: Option<&IndexReading>) -> Option<Decimal> {
    let hundred = Decimal::ONE_HUNDRED;
    let spread = dec("", &t.monthly_rate).ok().flatten().or_else(|| {
        dec("", &t.annual_rate).ok().flatten().map(|a| {
            (crate::finance::pow_dec(Decimal::ONE + a / hundred, Decimal::ONE / Decimal::from(12))
                - Decimal::ONE)
                * hundred
        })
    });
    match (t.indexer, index) {
        (Some(_), Some(ix)) => {
            let idx: Decimal = ix.monthly_pct.parse().ok()?;
            let s = spread.unwrap_or_default();
            Some(
                ((Decimal::ONE + s / hundred) * (Decimal::ONE + idx / hundred) - Decimal::ONE)
                    * hundred,
            )
        }
        // Pós-fixado sem o valor do índice no cofre: sem estimativa (melhor que só o spread).
        (Some(_), None) => None,
        (None, _) => dec("", &t.monthly_rate).ok().flatten(),
    }
}

/// Uso e custo estimado com o saldo atual da conta (`index`: último valor do indexador, nas
/// taxas pós-fixadas; ver [`index_reading`]).
pub fn status(t: &OverdraftTerms, balance: i64, index: Option<IndexReading>) -> OverdraftStatus {
    let used = (-balance).max(0);
    let rate = monthly_rate(t, index.as_ref());
    let discount = dec("", &t.discount_pct).ok().flatten().unwrap_or_default();
    let effective = rate.map(|r| r * (Decimal::ONE - discount / Decimal::from(100)));
    let iof_daily = dec("", &t.iof_daily_rate).ok().flatten();
    let iof_add = dec("", &t.iof_additional_rate).ok().flatten();
    let iof = match (iof_daily, iof_add) {
        (None, None) => None,
        (d, a) => Some(pct_of(
            used,
            d.unwrap_or_default() * Decimal::from(30) + a.unwrap_or_default(),
        )),
    };
    OverdraftStatus {
        limit: t.limit,
        used,
        available: t.limit - used,
        spendable: balance + t.limit,
        effective_monthly_rate: effective.map(|e| e.round_dp(4).normalize().to_string()),
        index: index.filter(|_| t.indexer.is_some()),
        estimated_interest: effective.map(|e| pct_of(used, e)),
        estimated_iof: iof,
        uncertain: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_interest_and_iof_on_the_used_part() {
        let t = validate(&OverdraftTerms {
            limit: 500_000,
            monthly_rate: Some("8,00%".into()),
            discount_pct: Some("25".into()),
            annual_rate: Some("151,82".into()),
            iof_daily_rate: Some("0,0082".into()),
            iof_additional_rate: Some("0,38".into()),
            cet_month: Some("8,9".into()),
            cet_year: Some("178,5".into()),
            indexer: None,
        })
        .unwrap();
        assert_eq!(t.monthly_rate.as_deref(), Some("8"));
        assert_eq!(t.iof_daily_rate.as_deref(), Some("0.0082"));
        let s = status(&t, -100_000, None);
        assert_eq!(
            (s.used, s.available, s.spendable),
            (100_000, 400_000, 400_000)
        );
        assert_eq!(s.effective_monthly_rate.as_deref(), Some("6"));
        assert_eq!(s.estimated_interest, Some(6_000), "R$ 1.000 × 6%");
        // 0,0082% × 30 + 0,38% = 0,626% de R$ 1.000 = R$ 6,26.
        assert_eq!(s.estimated_iof, Some(626));
        // Saldo positivo: nada em uso.
        let s = status(&t, 25_000, None);
        assert_eq!((s.used, s.available, s.spendable), (0, 500_000, 525_000));
        assert_eq!(s.estimated_interest, Some(0));
        assert!(
            validate(&OverdraftTerms {
                limit: -1,
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            validate(&OverdraftTerms {
                limit: 1,
                monthly_rate: Some("abc".into()),
                ..Default::default()
            })
            .is_err()
        );
        // Milhar com ponto e decimal com vírgula.
        let t = validate(&OverdraftTerms {
            limit: 1,
            annual_rate: Some("1.234,5".into()),
            ..Default::default()
        });
        assert!(t.is_ok());
    }
}

#[cfg(test)]
mod tests_indexed {
    use super::*;

    /// "4,75 + TR" (pedido de 21/10/2026): spread de 4,75% a.a. sobre a TR. A taxa efetiva
    /// do mês compõe o spread mensal com o último valor da TR.
    #[test]
    fn rate_plus_tr_is_spread_and_indexer_with_the_effective_monthly_rate() {
        let t = validate(&OverdraftTerms {
            limit: 1_000_000,
            annual_rate: Some("4,75 + TR".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(t.annual_rate.as_deref(), Some("4.75"));
        assert_eq!(t.indexer, Some(LoanIndexer::Tr));
        // Indexador também do outro lado ("TR + 4,75% a.a."), e indexadores em conflito.
        let t2 = validate(&OverdraftTerms {
            limit: 1,
            annual_rate: Some("TR + 4,75% a.a.".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            (t2.annual_rate.as_deref(), t2.indexer),
            (Some("4.75"), Some(LoanIndexer::Tr))
        );
        assert!(
            validate(&OverdraftTerms {
                limit: 1,
                annual_rate: Some("4,75 + TR".into()),
                monthly_rate: Some("1 + CDI".into()),
                ..Default::default()
            })
            .is_err()
        );
        let tr = IndexReading {
            indexer: LoanIndexer::Tr,
            monthly_pct: "0.15".into(),
            date: "2026-10-01".into(),
            estimated: false,
        };
        let s = status(&t, -100_000, Some(tr.clone()));
        // 1,0475^(1/12) × 1,0015 − 1 ≈ 0,538% a.m.
        let eff: Decimal = s
            .effective_monthly_rate
            .as_deref()
            .unwrap()
            .parse()
            .unwrap();
        assert!(
            (eff - Decimal::new(538, 3)).abs() < Decimal::new(2, 4),
            "{eff}"
        );
        assert_eq!(s.estimated_interest, Some(538), "R$ 1.000 × 0,538%");
        assert_eq!(s.index, Some(tr));
        // Sem o valor da TR no cofre: sem estimativa de juros.
        assert_eq!(status(&t, -100_000, None).estimated_interest, None);
    }
}

/// Saldo negativo de uma conta num período: dias no vermelho, o menor saldo e os meses
/// (`YYYY-MM`) em que ficou negativa. É a mesma conta do insight de cheque especial e da
/// métrica de crédito caro da saúde financeira: saldo inicial (abertura + lançamentos
/// efetivados antes do período) e os movimentos efetivados dia a dia.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NegativeSpell {
    pub days: i64,
    pub min: i64,
    pub months: std::collections::BTreeSet<String>,
}

pub fn negative_spell(
    conn: &Connection,
    account_id: &str,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> Result<NegativeSpell> {
    use crate::time::{fmt_date, month_of, parse_date};
    let before = fmt_date(from - chrono::Duration::days(1));
    let start: i64 = conn.query_row(
        "SELECT CASE WHEN a.opening_date <= ?2 THEN a.opening_balance ELSE 0 END
                + COALESCE((SELECT SUM(t.amount) FROM txn t WHERE t.account_id = a.id AND t.date <= ?2
                              AND t.status IN ('cleared', 'reconciled')), 0)
         FROM account a WHERE a.id = ?1",
        params![account_id, before],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare_cached(
        "SELECT date, SUM(amount) FROM txn WHERE account_id = ?1 AND status IN ('cleared', 'reconciled')
           AND date >= ?2 AND date <= ?3 GROUP BY date ORDER BY date",
    )?;
    let moves: Vec<(String, i64)> = stmt
        .query_map(params![account_id, fmt_date(from), fmt_date(to)], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = NegativeSpell {
        min: start,
        ..Default::default()
    };
    // Dias [de, até) com o saldo negativo.
    let mark = |out: &mut NegativeSpell, a: chrono::NaiveDate, b: chrono::NaiveDate| {
        out.days += (b - a).num_days();
        let mut d = a;
        while d < b {
            out.months.insert(month_of(d));
            d += chrono::Duration::days(1);
        }
    };
    let mut bal = start;
    let mut prev = from;
    for (d, amt) in &moves {
        let day = parse_date(d)?;
        if bal < 0 {
            mark(&mut out, prev, day);
        }
        bal += amt;
        out.min = out.min.min(bal);
        prev = day;
    }
    if bal < 0 {
        mark(&mut out, prev, to + chrono::Duration::days(1));
    }
    Ok(out)
}
