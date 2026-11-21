//! Uso e custo dos motores: tokens e custo estimado por chamada, motor e tarefa; tabela de
//! preços por modelo (editável); limites de gasto diário e mensal por provider de API.
//!
//! Custos em micro-dólares (USD × 10⁻⁶, inteiro): uma chamada pequena custa frações de
//! centavo. CLIs (assinatura do usuário) e modelos locais entram com custo zero.

use chrono::{Datelike, Local, NaiveDate, TimeZone};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{Engine, EngineKind, Provider};
use super::routing::AiTask;
use crate::error::{Error, Result};
use crate::ids::new_id;
use crate::time::{now_utc, today_local};

pub const CURRENCY: &str = "USD";

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct ModelPrice {
    pub provider: Provider,
    /// Nome do modelo ou prefixo (`claude-sonnet` vale para `claude-sonnet-4-5`); `*` = padrão do provider.
    pub model: String,
    #[ts(type = "number")]
    pub input_micros_per_mtok: i64,
    #[ts(type = "number")]
    pub output_micros_per_mtok: i64,
    #[ts(type = "number | null")]
    pub cached_micros_per_mtok: Option<i64>,
    /// `default` (tabela do app) ou `user` (editado).
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetPriceArgs {
    pub provider: Provider,
    pub model: String,
    #[ts(type = "number")]
    pub input_micros_per_mtok: i64,
    #[ts(type = "number")]
    pub output_micros_per_mtok: i64,
    #[ts(optional, type = "number")]
    pub cached_micros_per_mtok: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PriceKey {
    pub provider: Provider,
    pub model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct SpendLimit {
    pub provider: Option<Provider>,
    #[ts(type = "number | null")]
    pub daily_micros: Option<i64>,
    #[ts(type = "number | null")]
    pub monthly_micros: Option<i64>,
    /// Pedir confirmação antes de tarefas com custo estimado acima disto.
    #[ts(type = "number | null")]
    pub confirm_above_micros: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetLimitArgs {
    pub provider: Provider,
    #[serde(default)]
    #[ts(optional, type = "number | null")]
    pub daily_micros: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number | null")]
    pub monthly_micros: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number | null")]
    pub confirm_above_micros: Option<i64>,
}

/// Tokens de uma chamada.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Tokens {
    #[ts(type = "number")]
    pub input: i64,
    #[ts(type = "number")]
    pub output: i64,
    /// Entrada lida do cache do provider (já incluída em `input`).
    #[ts(type = "number")]
    pub cached: i64,
    /// Entrada gravada no cache do provider (já incluída em `input`; a Anthropic cobra 25%
    /// a mais por ela, e a leitura depois sai por 10%).
    #[serde(default)]
    #[ts(type = "number")]
    pub cache_write: i64,
}

impl Tokens {
    pub fn add(&mut self, o: Tokens) {
        self.input += o.input;
        self.output += o.output;
        self.cached += o.cached;
        self.cache_write += o.cache_write;
    }
}

/// Gravar no cache custa 25% a mais que a entrada comum (Anthropic).
const CACHE_WRITE_PERCENT: i64 = 125;

// ---------------------------------------------------------------------------
// Preços

fn price_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ModelPrice> {
    let p: String = r.get(0)?;
    Ok(ModelPrice {
        provider: Provider::parse(&p).unwrap_or(Provider::OpenaiCompatible),
        model: r.get(1)?,
        input_micros_per_mtok: r.get(2)?,
        output_micros_per_mtok: r.get(3)?,
        cached_micros_per_mtok: r.get(4)?,
        source: r.get(5)?,
        updated_at: r.get(6)?,
    })
}

const PRICE_COLS: &str = "provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at";

pub fn prices(conn: &Connection) -> Result<Vec<ModelPrice>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PRICE_COLS} FROM ai_model_price ORDER BY provider, model"
    ))?;
    Ok(stmt
        .query_map([], price_row)?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn set_price(conn: &Connection, a: &SetPriceArgs) -> Result<ModelPrice> {
    let model = a.model.trim();
    if model.is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-ia-preco-modelo-obrigatorio",
        )));
    }
    if a.input_micros_per_mtok < 0
        || a.output_micros_per_mtok < 0
        || a.cached_micros_per_mtok.is_some_and(|c| c < 0)
    {
        return Err(Error::invalid(crate::i18n::t("nucleo-ia-preco-negativo")));
    }
    conn.execute(
        "INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'user', ?6)
         ON CONFLICT (provider, model) DO UPDATE SET input_micros_per_mtok = excluded.input_micros_per_mtok,
           output_micros_per_mtok = excluded.output_micros_per_mtok, cached_micros_per_mtok = excluded.cached_micros_per_mtok,
           source = 'user', updated_at = excluded.updated_at",
        params![
            a.provider.as_str(),
            model,
            a.input_micros_per_mtok,
            a.output_micros_per_mtok,
            a.cached_micros_per_mtok,
            now_utc()
        ],
    )?;
    find_price(conn, a.provider, model)?
        .ok_or_else(|| Error::not_found(crate::i18n::t("nucleo-ia-preco")))
}

pub fn delete_price(conn: &Connection, a: &PriceKey) -> Result<()> {
    conn.execute(
        "DELETE FROM ai_model_price WHERE provider = ?1 AND model = ?2",
        params![a.provider.as_str(), a.model.trim()],
    )?;
    Ok(())
}

/// Preço do modelo: nome exato, maior prefixo cadastrado, ou o padrão `*` do provider.
pub fn find_price(
    conn: &Connection,
    provider: Provider,
    model: &str,
) -> Result<Option<ModelPrice>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PRICE_COLS} FROM ai_model_price WHERE provider = ?1"
    ))?;
    let all: Vec<ModelPrice> = stmt
        .query_map([provider.as_str()], price_row)?
        .collect::<rusqlite::Result<_>>()?;
    let model_l = model.to_ascii_lowercase();
    let best = all
        .iter()
        .filter(|p| p.model != "*" && model_l.starts_with(&p.model.to_ascii_lowercase()))
        .max_by_key(|p| p.model.len())
        .or_else(|| all.iter().find(|p| p.model == "*"));
    Ok(best.cloned())
}

fn per_mtok(tokens: i64, micros_per_mtok: i64) -> i64 {
    // Arredonda para cima: o custo estimado nunca fica abaixo do real por arredondamento.
    let v = tokens.max(0) as i128 * micros_per_mtok.max(0) as i128;
    ((v + 999_999) / 1_000_000) as i64
}

/// Custo de uma chamada (`None` sem preço cadastrado). CLIs e locais: zero.
pub fn cost(conn: &Connection, e: &Engine, t: Tokens) -> Result<Option<i64>> {
    if e.kind != EngineKind::Api {
        return Ok(Some(0));
    }
    let Some(p) = find_price(conn, e.provider, e.model.as_deref().unwrap_or(""))? else {
        return Ok(None);
    };
    let cached = t.cached.clamp(0, t.input.max(0));
    let written = t.cache_write.clamp(0, t.input.max(0) - cached);
    let fresh = t.input.max(0) - cached - written;
    let cached_price = p.cached_micros_per_mtok.unwrap_or(p.input_micros_per_mtok);
    Ok(Some(
        per_mtok(fresh, p.input_micros_per_mtok)
            + per_mtok(cached, cached_price)
            + per_mtok(written, p.input_micros_per_mtok * CACHE_WRITE_PERCENT / 100)
            + per_mtok(t.output, p.output_micros_per_mtok),
    ))
}

/// Tokens aproximados de um texto (≈ 4 caracteres por token).
pub fn approx_tokens(chars: usize) -> i64 {
    (chars as i64 + 3) / 4
}

// ---------------------------------------------------------------------------
// Limites

pub fn limits(conn: &Connection) -> Result<Vec<SpendLimit>> {
    let mut stmt = conn.prepare(
        "SELECT provider, daily_micros, monthly_micros, confirm_above_micros FROM ai_spend_limit ORDER BY provider",
    )?;
    Ok(stmt
        .query_map([], |r| {
            let p: String = r.get(0)?;
            Ok(SpendLimit {
                provider: Provider::parse(&p).ok(),
                daily_micros: r.get(1)?,
                monthly_micros: r.get(2)?,
                confirm_above_micros: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn limit_for(conn: &Connection, provider: Provider) -> Result<SpendLimit> {
    Ok(conn
        .query_row(
            "SELECT daily_micros, monthly_micros, confirm_above_micros FROM ai_spend_limit WHERE provider = ?1",
            [provider.as_str()],
            |r| {
                Ok(SpendLimit {
                    provider: Some(provider),
                    daily_micros: r.get(0)?,
                    monthly_micros: r.get(1)?,
                    confirm_above_micros: r.get(2)?,
                })
            },
        )
        .optional()?
        .unwrap_or(SpendLimit {
            provider: Some(provider),
            ..Default::default()
        }))
}

pub fn set_limit(conn: &Connection, a: &SetLimitArgs) -> Result<SpendLimit> {
    if a.provider.kind() != EngineKind::Api {
        return Err(Error::invalid(crate::i18n::t("nucleo-ia-limite-so-api")));
    }
    for v in [a.daily_micros, a.monthly_micros, a.confirm_above_micros]
        .into_iter()
        .flatten()
    {
        if v < 0 {
            return Err(Error::invalid(crate::i18n::t("nucleo-ia-limite-negativo")));
        }
    }
    conn.execute(
        "INSERT INTO ai_spend_limit (provider, daily_micros, monthly_micros, confirm_above_micros, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (provider) DO UPDATE SET daily_micros = excluded.daily_micros, monthly_micros = excluded.monthly_micros,
           confirm_above_micros = excluded.confirm_above_micros, updated_at = excluded.updated_at",
        params![
            a.provider.as_str(),
            a.daily_micros,
            a.monthly_micros,
            a.confirm_above_micros,
            now_utc()
        ],
    )?;
    limit_for(conn, a.provider)
}

/// Meia-noite local de `d` como instante UTC (RFC 3339), para comparar com `created_at`.
fn local_midnight_utc(d: NaiveDate) -> String {
    Local
        .from_local_datetime(&d.and_hms_opt(0, 0, 0).expect("meia-noite"))
        .earliest()
        .map(|t| {
            t.with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        })
        .unwrap_or_else(|| format!("{d}T00:00:00Z"))
}

/// Início (UTC, RFC 3339) do dia e do mês locais de hoje.
fn local_bounds() -> (String, String) {
    let today = today_local();
    let month_start = today.with_day(1).expect("dia 1");
    (local_midnight_utc(today), local_midnight_utc(month_start))
}

/// Gasto de hoje e do mês (micro-dólares) de um provider.
pub fn spent(conn: &Connection, provider: Provider) -> Result<(i64, i64)> {
    let (day, month) = local_bounds();
    let q = |since: &str| -> Result<i64> {
        Ok(conn.query_row(
            "SELECT COALESCE(SUM(cost_micros), 0) FROM ai_usage WHERE provider = ?1 AND created_at >= ?2",
            params![provider.as_str(), since],
            |r| r.get(0),
        )?)
    };
    Ok((q(&day)?, q(&month)?))
}

/// Confere os limites antes de uma chamada com custo estimado `estimate`. O erro explica
/// qual limite seria passado.
pub fn check_limits(
    conn: &Connection,
    e: &Engine,
    estimate: i64,
) -> std::result::Result<(), String> {
    if e.kind != EngineKind::Api {
        return Ok(());
    }
    let lim = limit_for(conn, e.provider).map_err(|x| x.to_string())?;
    let (day, month) = spent(conn, e.provider).map_err(|x| x.to_string())?;
    if let Some(d) = lim.daily_micros
        && day + estimate > d
    {
        return Err(limit_reached("nucleo-ia-limite-diario", e.provider, day, d));
    }
    if let Some(m) = lim.monthly_micros
        && month + estimate > m
    {
        return Err(limit_reached(
            "nucleo-ia-limite-mensal",
            e.provider,
            month,
            m,
        ));
    }
    Ok(())
}

/// Mensagem do limite de gasto atingido (diário ou mensal).
fn limit_reached(key: &str, provider: Provider, spent: i64, limit: i64) -> String {
    crate::i18n::t_args(
        key,
        &crate::i18n::args(&[
            ("provider", provider.label().into()),
            ("gasto", fmt_usd(spent).into()),
            ("limite", fmt_usd(limit).into()),
        ]),
    )
}

pub fn fmt_usd(micros: i64) -> String {
    format!("US$ {:.4}", micros as f64 / 1_000_000.0).replace('.', ",")
}

// ---------------------------------------------------------------------------
// Registro

pub struct UsageRecord<'a> {
    pub engine: &'a Engine,
    pub task: AiTask,
    pub job_id: Option<&'a str>,
    pub tokens: Tokens,
    pub error: Option<&'a str>,
}

/// Grava uma chamada e devolve (custo, tem preço).
pub fn record(conn: &Connection, u: &UsageRecord<'_>) -> Result<(i64, bool)> {
    let c = cost(conn, u.engine, u.tokens)?;
    conn.execute(
        "INSERT INTO ai_usage (id, created_at, engine_id, engine_name, provider, kind, model, task, job_id,
           input_tokens, output_tokens, cached_tokens, cost_micros, priced, ok, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            new_id(),
            now_utc(),
            u.engine.id,
            u.engine.name,
            u.engine.provider.as_str(),
            u.engine.kind.as_str(),
            u.engine.model,
            u.task.as_str(),
            u.job_id,
            u.tokens.input,
            u.tokens.output,
            u.tokens.cached,
            c.unwrap_or(0),
            c.is_some() as i64,
            u.error.is_none() as i64,
            u.error.map(|e| e.chars().take(300).collect::<String>()),
        ],
    )?;
    Ok((c.unwrap_or(0), c.is_some()))
}

// ---------------------------------------------------------------------------
// Resumo (painel de uso)

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UsageSummaryArgs {
    /// Primeiro dia `YYYY-MM-DD` (padrão: dia 1 do mês atual).
    #[ts(optional)]
    pub from: Option<String>,
    /// Último dia `YYYY-MM-DD` (padrão: hoje).
    #[ts(optional)]
    pub to: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UsageTotals {
    #[ts(type = "number")]
    pub calls: i64,
    #[ts(type = "number")]
    pub input_tokens: i64,
    #[ts(type = "number")]
    pub output_tokens: i64,
    #[ts(type = "number")]
    pub cost_micros: i64,
    /// Chamadas sem preço cadastrado para o modelo (custo fora do total).
    #[ts(type = "number")]
    pub unpriced_calls: i64,
    #[ts(type = "number")]
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UsageGroup {
    /// Provider, id do motor, tarefa ou mês `YYYY-MM`.
    pub key: String,
    pub label: String,
    pub kind: Option<EngineKind>,
    pub totals: UsageTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProviderSpend {
    pub provider: Provider,
    pub label: String,
    #[ts(type = "number")]
    pub today_micros: i64,
    #[ts(type = "number")]
    pub month_micros: i64,
    pub limit: SpendLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UsageSummary {
    pub from: String,
    pub to: String,
    /// Custos em micro-unidades desta moeda (USD × 10⁻⁶).
    pub currency: String,
    pub totals: UsageTotals,
    pub by_provider: Vec<UsageGroup>,
    pub by_engine: Vec<UsageGroup>,
    pub by_task: Vec<UsageGroup>,
    pub by_month: Vec<UsageGroup>,
    /// Gasto de hoje e do mês por provider de API, com os limites.
    pub spend: Vec<ProviderSpend>,
}

pub fn summary(conn: &Connection, a: &UsageSummaryArgs) -> Result<UsageSummary> {
    let today = today_local();
    let from = match &a.from {
        Some(f) => crate::time::parse_date(f)?,
        None => today.with_day(1).expect("dia 1"),
    };
    let to = match &a.to {
        Some(t) => crate::time::parse_date(t)?,
        None => today,
    };
    if from > to {
        return Err(Error::invalid(crate::i18n::t("nucleo-ia-uso-periodo")));
    }
    // created_at é UTC: os limites são as meias-noites locais convertidas para UTC (às 21h
    // em Brasília já é o dia seguinte em UTC, e o uso da noite precisa contar no dia certo).
    let from_s = local_midnight_utc(from);
    let to_s = local_midnight_utc(to.succ_opt().unwrap_or(to));
    let group = |expr: &str, label_expr: &str| -> Result<Vec<UsageGroup>> {
        let sql = format!(
            "SELECT {expr} AS k, MAX({label_expr}) AS l, MAX(kind), COUNT(*), SUM(input_tokens), SUM(output_tokens),
               SUM(cost_micros), SUM(CASE WHEN priced = 0 THEN 1 ELSE 0 END), SUM(CASE WHEN ok = 0 THEN 1 ELSE 0 END)
             FROM ai_usage WHERE created_at >= ?1 AND created_at < ?2 GROUP BY k ORDER BY SUM(cost_micros) DESC, k"
        );
        let mut stmt = conn.prepare(&sql)?;
        Ok(stmt
            .query_map(params![from_s, to_s], |r| {
                let kind: String = r.get(2)?;
                Ok(UsageGroup {
                    key: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    label: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    kind: match kind.as_str() {
                        "api" => Some(EngineKind::Api),
                        "cli" => Some(EngineKind::Cli),
                        "local" => Some(EngineKind::Local),
                        _ => None,
                    },
                    totals: UsageTotals {
                        calls: r.get(3)?,
                        input_tokens: r.get(4)?,
                        output_tokens: r.get(5)?,
                        cost_micros: r.get(6)?,
                        unpriced_calls: r.get(7)?,
                        errors: r.get(8)?,
                    },
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    };
    let by_provider: Vec<UsageGroup> = group("provider", "provider")?
        .into_iter()
        .map(|mut g| {
            if let Ok(p) = Provider::parse(&g.key) {
                g.label = p.label().into();
            }
            g
        })
        .collect();
    let by_engine = group("COALESCE(engine_id, engine_name)", "engine_name")?;
    let by_task: Vec<UsageGroup> = group("task", "task")?
        .into_iter()
        .map(|mut g| {
            if let Some(t) = AiTask::parse(&g.key) {
                g.label = t.label().into();
            }
            g
        })
        .collect();
    let by_month = group(
        "strftime('%Y-%m', created_at, 'localtime')",
        "strftime('%Y-%m', created_at, 'localtime')",
    )?;
    let mut totals = UsageTotals::default();
    for g in &by_provider {
        totals.calls += g.totals.calls;
        totals.input_tokens += g.totals.input_tokens;
        totals.output_tokens += g.totals.output_tokens;
        totals.cost_micros += g.totals.cost_micros;
        totals.unpriced_calls += g.totals.unpriced_calls;
        totals.errors += g.totals.errors;
    }
    let mut spend = vec![];
    let mut provs: Vec<Provider> = super::engine::ALL_PROVIDERS
        .iter()
        .copied()
        .filter(|p| p.kind() == EngineKind::Api)
        .collect();
    provs.retain(|p| {
        by_provider.iter().any(|g| g.key == p.as_str())
            || limits(conn)
                .map(|l| l.iter().any(|x| x.provider == Some(*p)))
                .unwrap_or(false)
    });
    for p in provs {
        let (d, m) = spent(conn, p)?;
        spend.push(ProviderSpend {
            provider: p,
            label: p.label().into(),
            today_micros: d,
            month_micros: m,
            limit: limit_for(conn, p)?,
        });
    }
    Ok(UsageSummary {
        from: from.to_string(),
        to: to.to_string(),
        currency: CURRENCY.into(),
        totals,
        by_provider,
        by_engine,
        by_task,
        by_month,
        spend,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_rounds_up_and_uses_cache_price() {
        assert_eq!(per_mtok(1, 3_000_000), 3);
        assert_eq!(per_mtok(1_000_000, 3_000_000), 3_000_000);
        assert_eq!(per_mtok(1, 1), 1);
        assert_eq!(per_mtok(0, 5), 0);
        assert_eq!(approx_tokens(8), 2);
    }
}
