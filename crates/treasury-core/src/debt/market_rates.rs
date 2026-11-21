//! Taxas de juros de mercado (docs/data-sources.md): a taxa média que cada banco cobrou da
//! pessoa física em cada modalidade de crédito, publicada pelo Banco Central ("Taxas de
//! juros por instituição financeira", API Olinda `taxaJuros`, sem chave): semanal para
//! crédito pessoal, consignado, veículo, cheque especial e cartão; mensal para o
//! financiamento imobiliário.
//!
//! Para que serve: comparar o contrato do usuário com o mercado ("seu crédito pessoal custa
//! 6,9% a.m.; a mediana dos bancos é 4,1% e o mais barato cobra 1,9%") e dar números à
//! portabilidade e ao plano de quitação, sem IA.
//!
//! A coleta guarda as últimas semanas em `credit_rate` (V119) e é refeita quando os dados
//! têm mais de [`STALE_DAYS`] dias; sem rede, vale o que já foi baixado.

use std::time::Duration;

use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::{LoanIndexer, LoanKind};
use crate::error::Result;
use crate::i18n::{args, t, t_args};
use crate::time::{now_utc, today_local};

/// Refaz a coleta quando a última tem mais que isso.
pub const STALE_DAYS: i64 = 7;
/// Bancos mais baratos mostrados por modalidade.
pub const BEST: usize = 5;
const BASE: &str = "https://olinda.bcb.gov.br/olinda/servico/taxaJuros/versao/v2/odata";
const SEGMENT_PF: &str = "PESSOA%20F%C3%8DSICA";

/// Modalidades que o app mostra: o nome no BCB (casa com os dados baixados) e a chave do nome
/// curto no catálogo.
pub const MODALITIES: &[(&str, &str)] = &[
    (
        "Cheque especial - Prefixado",
        "nucleo-planejamento-mercado-modalidade-cheque-especial",
    ),
    (
        "Cartão de crédito - rotativo total - Prefixado",
        "nucleo-planejamento-mercado-modalidade-cartao-rotativo",
    ),
    (
        "Cartão de crédito - parcelado - Prefixado",
        "nucleo-planejamento-mercado-modalidade-cartao-parcelado",
    ),
    (
        "Crédito pessoal não consignado - Prefixado",
        "nucleo-planejamento-mercado-modalidade-credito-pessoal",
    ),
    (
        "Crédito pessoal consignado privado - Prefixado",
        "nucleo-planejamento-mercado-modalidade-consignado-privado",
    ),
    (
        "Crédito pessoal consignado público - Prefixado",
        "nucleo-planejamento-mercado-modalidade-consignado-publico",
    ),
    (
        "Crédito pessoal consignado INSS - Prefixado",
        "nucleo-planejamento-mercado-modalidade-consignado-inss",
    ),
    (
        "Aquisição de veículos - Prefixado",
        "nucleo-planejamento-mercado-modalidade-veiculo",
    ),
    (
        "Aquisição de outros bens - Prefixado",
        "nucleo-planejamento-mercado-modalidade-outros-bens",
    ),
    (
        "Financiamento imobiliário com taxas de mercado - Prefixado",
        "nucleo-planejamento-mercado-modalidade-imobiliario-pre",
    ),
    (
        "Financiamento imobiliário com taxas de mercado - Pós-fixado referenciado em TR",
        "nucleo-planejamento-mercado-modalidade-imobiliario-tr",
    ),
    (
        "Financiamento imobiliário com taxas de mercado - Pós-fixado referenciado em IPCA",
        "nucleo-planejamento-mercado-modalidade-imobiliario-ipca",
    ),
];

/// Nome curto da modalidade no idioma da instalação (sem ela na lista: o nome do BCB).
fn label_of(modality: &str) -> String {
    MODALITIES
        .iter()
        .find(|(m, _)| *m == modality)
        .map_or_else(|| modality.to_string(), |(_, key)| t(key))
}

/// Modalidade do mercado que corresponde a um contrato.
pub fn modality_for(kind: LoanKind, indexer: LoanIndexer) -> Option<&'static str> {
    Some(match kind {
        LoanKind::Personal => "Crédito pessoal não consignado - Prefixado",
        LoanKind::Payroll => "Crédito pessoal consignado privado - Prefixado",
        LoanKind::Vehicle => "Aquisição de veículos - Prefixado",
        LoanKind::Mortgage => match indexer {
            LoanIndexer::Tr => {
                "Financiamento imobiliário com taxas de mercado - Pós-fixado referenciado em TR"
            }
            LoanIndexer::Ipca => {
                "Financiamento imobiliário com taxas de mercado - Pós-fixado referenciado em IPCA"
            }
            LoanIndexer::None => "Financiamento imobiliário com taxas de mercado - Prefixado",
            _ => return None,
        },
        LoanKind::Student | LoanKind::Other => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MarketRate")]
pub struct Rate {
    pub institution: String,
    /// % a.m. e % a.a.
    pub rate_month: f64,
    pub rate_year: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MarketModality")]
pub struct Modality {
    pub modality: String,
    pub label: String,
    pub period_start: String,
    pub period_end: String,
    /// Bancos com taxa publicada no período.
    pub count: u32,
    /// % a.m.: mínima, quartil inferior, mediana e máxima.
    pub min: f64,
    pub p25: f64,
    pub median: f64,
    pub max: f64,
    /// Mediana em % a.a.
    pub median_year: f64,
    pub best: Vec<Rate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "LoanRateCheck")]
pub struct LoanCheck {
    pub loan_id: String,
    pub loan_name: String,
    pub modality: String,
    pub modality_label: String,
    /// Taxa do contrato, % a.m.
    pub rate_month: f64,
    pub median: f64,
    pub p25: f64,
    pub min: f64,
    /// Fração dos bancos que cobram menos que o contrato (0 a 1).
    pub cheaper_share: f64,
    /// `below` (abaixo do quartil inferior), `typical` (até a mediana), `above` (acima da
    /// mediana) ou `far_above` (acima de 1,5× a mediana).
    pub verdict: String,
    /// Uma frase com os números, para a tela e o agente.
    pub summary: String,
    pub best: Vec<Rate>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MarketRates")]
pub struct MarketRates {
    pub fetched_at: Option<String>,
    pub modalities: Vec<Modality>,
    pub loans: Vec<LoanCheck>,
    /// Aviso da coleta (sem rede, fonte desligada…); os números são os últimos baixados.
    pub warning: Option<String>,
    pub source: String,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .build()
        .into()
}

fn get_json(url: &str) -> std::result::Result<Value, String> {
    let body = crate::indices::sync::get_text(&agent(), url)?;
    serde_json::from_str(&body).map_err(|e| {
        t_args(
            "nucleo-planejamento-mercado-erro-resposta",
            &args(&[("erro", e.to_string().into())]),
        )
    })
}

/// Uma linha baixada: (início, fim, modalidade, banco, cnpj8, % a.m., % a.a., posição).
pub type Row = (
    String,
    String,
    String,
    String,
    String,
    f64,
    f64,
    Option<i64>,
);

fn rows_of(v: &Value, monthly: bool) -> Vec<Row> {
    v["value"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            let (start, end) = if monthly {
                let ym = r["anoMes"].as_str()?;
                let first = format!("{ym}-01");
                let last = chrono::NaiveDate::parse_from_str(&first, "%Y-%m-%d")
                    .ok()
                    .map(|d| crate::time::add_months(d, 1).pred_opt().expect("data"))
                    .map(crate::time::fmt_date)?;
                (first, last)
            } else {
                (
                    r["InicioPeriodo"].as_str()?.to_string(),
                    r["FimPeriodo"].as_str()?.to_string(),
                )
            };
            Some((
                start,
                end,
                r["Modalidade"].as_str()?.to_string(),
                r["InstituicaoFinanceira"].as_str()?.to_string(),
                r["cnpj8"].as_str().unwrap_or("").to_string(),
                r["TaxaJurosAoMes"].as_f64()?,
                r["TaxaJurosAoAno"].as_f64()?,
                r["Posicao"].as_i64(),
            ))
        })
        .collect()
}

/// Baixa o período mais recente (semanal da pessoa física e mensal do imobiliário). Só rede.
pub fn fetch() -> std::result::Result<Vec<Row>, String> {
    let latest = get_json(&format!(
        "{BASE}/TaxasJurosDiariaPorInicioPeriodo?$orderby=InicioPeriodo%20desc&$top=1&$select=InicioPeriodo&$format=json"
    ))?;
    let start = latest["value"][0]["InicioPeriodo"]
        .as_str()
        .ok_or_else(|| t("nucleo-planejamento-mercado-erro-sem-periodo"))?
        .to_string();
    let weekly = get_json(&format!(
        "{BASE}/TaxasJurosDiariaPorInicioPeriodo?$filter=InicioPeriodo%20eq%20%27{start}%27%20and%20Segmento%20eq%20%27{SEGMENT_PF}%27&$top=10000&$format=json"
    ))?;
    let mut out = rows_of(&weekly, false);
    // Imobiliário: mensal (falha aqui não perde o semanal).
    if let Ok(m) = get_json(&format!(
        "{BASE}/TaxasJurosMensalPorMes?$orderby=anoMes%20desc&$top=1&$select=anoMes&$format=json"
    )) && let Some(ym) = m["value"][0]["anoMes"].as_str()
        && let Ok(monthly) = get_json(&format!(
            "{BASE}/TaxasJurosMensalPorMes?$filter=anoMes%20eq%20%27{ym}%27&$top=5000&$format=json"
        ))
    {
        out.extend(rows_of(&monthly, true));
    }
    if out.is_empty() {
        return Err(t("nucleo-planejamento-mercado-erro-sem-taxas"));
    }
    Ok(out)
}

/// Grava o que foi baixado e apaga o que tem mais de 120 dias.
pub fn store(conn: &Connection, rows: &[Row]) -> Result<usize> {
    let now = now_utc();
    let mut n = 0;
    for (start, end, modality, inst, cnpj8, m, y, pos) in rows {
        n += conn.execute(
            "INSERT INTO credit_rate (period_start, period_end, modality, institution, cnpj8, rate_month, rate_year, position, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT (period_start, modality, cnpj8) DO UPDATE SET period_end = excluded.period_end,
               institution = excluded.institution, rate_month = excluded.rate_month,
               rate_year = excluded.rate_year, position = excluded.position, fetched_at = excluded.fetched_at",
            params![start, end, modality, inst, if cnpj8.is_empty() { inst } else { cnpj8 }, m, y, pos, now],
        )?;
    }
    let cutoff = crate::time::fmt_date(today_local() - chrono::Duration::days(120));
    conn.execute("DELETE FROM credit_rate WHERE period_start < ?1", [cutoff])?;
    Ok(n)
}

/// Os dados estão velhos (ou não existem).
pub fn stale(conn: &Connection) -> Result<bool> {
    let last: Option<String> =
        conn.query_row("SELECT max(fetched_at) FROM credit_rate", [], |r| r.get(0))?;
    Ok(
        match last.and_then(|l| chrono::DateTime::parse_from_rfc3339(&l).ok()) {
            Some(t) => {
                chrono::Utc::now().signed_duration_since(t) > chrono::Duration::days(STALE_DAYS)
            }
            None => true,
        },
    )
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = (sorted.len() - 1) as f64 * q;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// O período mais recente de cada modalidade, com as estatísticas.
pub fn modalities(conn: &Connection) -> Result<Vec<Modality>> {
    let mut out = vec![];
    for (modality, key) in MODALITIES {
        let latest: Option<(String, String)> = conn
            .query_row(
                "SELECT period_start, period_end FROM credit_rate WHERE modality = ?1
                  ORDER BY period_start DESC LIMIT 1",
                [modality],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let Some((start, end)) = latest else { continue };
        let mut stmt = conn.prepare_cached(
            "SELECT institution, rate_month, rate_year FROM credit_rate
              WHERE modality = ?1 AND period_start = ?2 ORDER BY rate_month, institution",
        )?;
        let rates: Vec<Rate> = stmt
            .query_map(params![modality, start], |r| {
                Ok(Rate {
                    institution: r.get(0)?,
                    rate_month: r.get(1)?,
                    rate_year: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        if rates.is_empty() {
            continue;
        }
        let months: Vec<f64> = rates.iter().map(|r| r.rate_month).collect();
        let years: Vec<f64> = rates.iter().map(|r| r.rate_year).collect();
        out.push(Modality {
            modality: modality.to_string(),
            label: t(key),
            period_start: start,
            period_end: end,
            count: rates.len() as u32,
            min: months[0],
            p25: round2(percentile(&months, 0.25)),
            median: round2(percentile(&months, 0.5)),
            max: *months.last().expect("não vazio"),
            median_year: round2(percentile(&years, 0.5)),
            best: rates.into_iter().take(BEST).collect(),
        });
    }
    Ok(out)
}

/// Taxa com 2 casas no idioma: `1,90%` (pt-BR), `1.90%` (en).
fn pct(v: f64) -> String {
    format!(
        "{}%",
        crate::ai::alerts::decimal(crate::i18n::current(), &format!("{v:.2}"))
    )
}

/// Compara a taxa de um contrato (% a.m.) com a modalidade do mercado.
pub fn check(
    loan_id: &str,
    loan_name: &str,
    rate_month: f64,
    m: &Modality,
    all: &[f64],
) -> LoanCheck {
    let cheaper = all.iter().filter(|r| **r < rate_month).count();
    let share = if all.is_empty() {
        0.0
    } else {
        cheaper as f64 / all.len() as f64
    };
    let verdict = if rate_month <= m.p25 {
        "below"
    } else if rate_month <= m.median {
        "typical"
    } else if rate_month <= m.median * 1.5 {
        "above"
    } else {
        "far_above"
    };
    let best = m.best.first();
    let summary = match verdict {
        "below" | "typical" => t_args(
            "nucleo-planejamento-mercado-resumo-dentro",
            &args(&[
                ("contrato", loan_name.to_string().into()),
                ("taxa", pct(rate_month).into()),
                ("mediana", pct(m.median).into()),
                ("bancos", m.count.into()),
            ]),
        ),
        _ => t_args(
            "nucleo-planejamento-mercado-resumo-acima",
            &args(&[
                ("contrato", loan_name.to_string().into()),
                ("taxa", pct(rate_month).into()),
                ("mediana", pct(m.median).into()),
                ("mais_baratos", cheaper.into()),
                ("bancos", all.len().into()),
                (
                    "melhor",
                    best.map(|b| {
                        t_args(
                            "nucleo-planejamento-mercado-resumo-melhor",
                            &args(&[
                                ("banco", b.institution.clone().into()),
                                ("taxa", pct(b.rate_month).into()),
                            ]),
                        )
                    })
                    .unwrap_or_default()
                    .into(),
                ),
            ]),
        ),
    };
    LoanCheck {
        loan_id: loan_id.to_string(),
        loan_name: loan_name.to_string(),
        modality: m.modality.clone(),
        modality_label: label_of(&m.modality),
        rate_month: round2(rate_month),
        median: m.median,
        p25: m.p25,
        min: m.min,
        cheaper_share: (share * 100.0).round() / 100.0,
        verdict: verdict.into(),
        summary,
        best: m.best.clone(),
    }
}

/// Taxas do mercado e a comparação dos contratos ativos.
pub fn market_rates(conn: &Connection) -> Result<MarketRates> {
    let mods = modalities(conn)?;
    let fetched_at: Option<String> =
        conn.query_row("SELECT max(fetched_at) FROM credit_rate", [], |r| r.get(0))?;
    let mut loans = vec![];
    for l in super::list(conn, false)? {
        let Some(modality) = modality_for(l.kind, l.indexer) else {
            continue;
        };
        let Some(m) = mods.iter().find(|m| m.modality == modality) else {
            continue;
        };
        let Some(rate) = l
            .monthly_rate
            .parse::<Decimal>()
            .ok()
            .and_then(|d| d.to_f64())
            .filter(|r| *r > 0.0)
        else {
            continue;
        };
        let mut stmt = conn.prepare_cached(
            "SELECT rate_month FROM credit_rate WHERE modality = ?1 AND period_start = ?2",
        )?;
        let all: Vec<f64> = stmt
            .query_map(params![m.modality, m.period_start], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        loans.push(check(&l.id, &l.name, rate, m, &all));
    }
    Ok(MarketRates {
        fetched_at,
        modalities: mods,
        loans,
        warning: None,
        source: t("nucleo-planejamento-mercado-fonte"),
    })
}

/// Atualiza (se velho ou pedido) e devolve. Rede fora da trava do banco; sem rede, os
/// últimos dados baixados com o aviso.
pub fn refresh_and_get(api: &crate::api::Api, force: bool) -> Result<MarketRates> {
    let (enabled, is_stale) = api.with_session(|s, _| {
        Ok((
            crate::sources::enabled(s.db.conn(), crate::sources::SourceId::Bcb),
            stale(s.db.conn())?,
        ))
    })?;
    let mut warning = None;
    if !enabled {
        warning = Some(t("nucleo-planejamento-mercado-aviso-fonte-desligada"));
    } else if force || is_stale {
        match fetch() {
            Ok(rows) => {
                api.with_session(|s, _| store(s.db.conn(), &rows))?;
            }
            Err(e) => {
                warning = Some(t_args(
                    "nucleo-planejamento-mercado-aviso-falha",
                    &args(&[("erro", e.into())]),
                ))
            }
        }
    }
    let mut out = api.with_session(|s, _| market_rates(s.db.conn()))?;
    if out.modalities.is_empty() && warning.is_none() {
        warning = Some(t("nucleo-planejamento-mercado-aviso-sem-taxas"));
    }
    out.warning = warning;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_weekly_and_monthly_rows() {
        let weekly: Value = serde_json::from_str(r#"{"value":[{"InicioPeriodo":"2026-09-25","FimPeriodo":"2026-10-01","Segmento":"PESSOA FÍSICA","Modalidade":"Crédito pessoal não consignado - Prefixado","Posicao":1,"InstituicaoFinanceira":"BANCO A","TaxaJurosAoMes":1.9,"TaxaJurosAoAno":25.34,"cnpj8":"00000001"}]}"#).unwrap();
        let r = rows_of(&weekly, false);
        assert_eq!(r[0].0, "2026-09-25");
        assert_eq!(r[0].5, 1.9);
        let monthly: Value = serde_json::from_str(r#"{"value":[{"Mes":"Set-2026","Modalidade":"Financiamento imobiliário com taxas de mercado - Prefixado","Posicao":1,"InstituicaoFinanceira":"CAIXA","TaxaJurosAoMes":0.87,"TaxaJurosAoAno":10.94,"cnpj8":"00360305","anoMes":"2026-09"}]}"#).unwrap();
        let r = rows_of(&monthly, true);
        assert_eq!(
            (r[0].0.as_str(), r[0].1.as_str()),
            ("2026-09-01", "2026-09-30")
        );
    }

    fn modality(rates: &[f64]) -> (Modality, Vec<f64>) {
        let mut s = rates.to_vec();
        s.sort_by(f64::total_cmp);
        (
            Modality {
                modality: "Crédito pessoal não consignado - Prefixado".into(),
                label: "Crédito pessoal".into(),
                period_start: "2026-09-25".into(),
                period_end: "2026-10-01".into(),
                count: s.len() as u32,
                min: s[0],
                p25: round2(percentile(&s, 0.25)),
                median: round2(percentile(&s, 0.5)),
                max: *s.last().unwrap(),
                median_year: 0.0,
                best: vec![Rate {
                    institution: "BANCO A".into(),
                    rate_month: s[0],
                    rate_year: 0.0,
                }],
            },
            s,
        )
    }

    #[test]
    fn a_loan_is_compared_with_the_market() {
        let (m, all) = modality(&[1.9, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0]);
        assert_eq!(m.median, 5.0);
        let c = check("l1", "Empréstimo", 9.0, &m, &all);
        assert_eq!(c.verdict, "far_above");
        assert!(
            c.summary.contains("6 de 7 bancos cobram menos"),
            "{}",
            c.summary
        );
        assert!(c.summary.contains("BANCO A"));
        let c = check("l1", "Empréstimo", 3.0, &m, &all);
        assert_eq!(c.verdict, "below");
        assert!(c.summary.contains("dentro do que o mercado cobra"));
        assert_eq!(
            modality_for(LoanKind::Mortgage, LoanIndexer::Tr),
            Some("Financiamento imobiliário com taxas de mercado - Pós-fixado referenciado em TR")
        );
        assert_eq!(modality_for(LoanKind::Mortgage, LoanIndexer::Cdi), None);
    }

    #[test]
    #[ignore = "usa a rede: cargo test -- --ignored market_rates_live"]
    fn market_rates_live() {
        let rows = fetch().unwrap();
        let mods: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.2.as_str()).collect();
        eprintln!("{} linhas; {mods:#?}", rows.len());
        for (m, _) in MODALITIES {
            assert!(mods.contains(m), "faltou {m}");
        }
    }
}
