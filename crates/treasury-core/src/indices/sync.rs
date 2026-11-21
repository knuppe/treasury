//! Coleta de índices e câmbio em fontes oficiais e gratuitas (docs/data-sources.md):
//! BCB SGS (IPCA, IGP-M, CDI, Selic, TR, M2), BCB Olinda (PTAX e Focus) e FRED (CSV
//! público, sem chave: US CPI, US M2, Fed Funds).
//!
//! Em três passos, para não segurar o banco durante a rede: [`plan`] (lê o que falta),
//! [`fetch`] (só rede) e [`apply`] (grava). Cada valor guarda fonte e data de coleta.

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::providers::DataProvider;
use crate::error::{Error, Result};
use crate::ids::new_id;
use crate::time::{add_months, fmt_date, now_utc, parse_date, today_local};

const USER_AGENT: &str = crate::research::net::DEFAULT_USER_AGENT;
/// Início do histórico baixado na primeira coleta.
const MONTHLY_START: &str = "2000-01-01";
const DAILY_START: &str = "2012-01-01";
const FX_START: &str = "2015-01-01";

#[derive(Debug, Clone, PartialEq)]
pub struct PlanItem {
    pub series_id: String,
    pub source: String,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FxPlan {
    pub currency: String,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub series: Vec<PlanItem>,
    pub fx: Vec<FxPlan>,
    /// Último mês oficial do IPCA (para pedir as projeções Focus dos seguintes).
    pub ipca_last_official: Option<NaiveDate>,
    /// Chave da API da FRED, se cadastrada.
    pub fred_api_key: Option<String>,
    /// Projeções Focus do IPCA (só com o BCB ligado).
    pub focus: bool,
}

/// Resultado de um download: valores ou a mensagem de erro (a coleta segue com as demais).
pub type Download<T> = std::result::Result<Vec<T>, String>;

/// (data, valor, estimado).
pub type MarkedValue = (NaiveDate, Decimal, bool);

#[derive(Debug, Clone)]
pub struct Fetched {
    pub series: Vec<(PlanItem, Download<(NaiveDate, Decimal)>)>,
    /// Séries dos provedores do catálogo, com a marca de estimado de cada valor.
    pub marked: Vec<(PlanItem, Download<MarkedValue>)>,
    pub fx: Vec<(FxPlan, Download<PtaxQuote>)>,
    pub focus: Download<(NaiveDate, Decimal)>,
}

impl Default for Fetched {
    fn default() -> Self {
        Self {
            series: vec![],
            marked: vec![],
            fx: vec![],
            focus: Ok(vec![]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtaxQuote {
    pub date: NaiveDate,
    pub buy: Decimal,
    pub sell: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SeriesSyncResult {
    pub series_id: String,
    #[ts(type = "number")]
    pub written: u32,
    pub last_date: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SyncReport {
    pub started_at: String,
    pub finished_at: String,
    pub series: Vec<SeriesSyncResult>,
    pub fx: Vec<SeriesSyncResult>,
    /// Meses de IPCA estimados (Focus ou repetição do último dado).
    #[ts(type = "number")]
    pub ipca_estimated_months: u32,
    pub ok: bool,
}

fn last_date(conn: &Connection, series_id: &str, official_only: bool) -> Result<Option<NaiveDate>> {
    let d: Option<String> = conn
        .query_row(
            "SELECT max(date) FROM index_value WHERE series_id = ?1 AND (?2 = 0 OR estimated = 0)",
            params![series_id, official_only as i64],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    d.map(|s| parse_date(&s)).transpose()
}

/// O que precisa ser baixado (a partir do último valor oficial de cada série).
pub fn plan(conn: &Connection, today: NaiveDate) -> Result<Plan> {
    let mut stmt = conn.prepare("SELECT id, source, frequency FROM index_series ORDER BY id")?;
    let defs: Vec<(String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut p = Plan::default();
    // Séries de sempre (BCB, FRED, Yahoo) + as do país de referência (catálogo).
    let mut scope: std::collections::HashSet<String> =
        super::catalog::sync_scope(conn)?.into_iter().collect();
    // O índice oficial de alimentos do país (a comparação da inflação da cesta em Consumo).
    scope.extend(crate::food::profile::family(conn).official_series);
    // Fora do Brasil o câmbio vem do BCE: o euro contra a moeda base e contra as moedas das
    // contas e ativos (as conversões cruzam pelo euro; docs/countries.md).
    if !crate::country::is_brazil(conn) {
        let mut stmt = conn.prepare(
            "SELECT currency FROM account UNION SELECT currency FROM asset
              UNION SELECT base_currency FROM household UNION SELECT 'USD'",
        )?;
        let used: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for c in used.iter().filter(|c| c.as_str() != "EUR") {
            scope.insert(format!("fx_eur_{}", c.to_ascii_lowercase()));
        }
    }
    // No Brasil, as moedas das contas e ativos que não têm PTAX vêm do BCE e chegam ao real
    // cruzando pelo euro (EUR → BRL pela PTAX). Sem isso a pessoa teria de informar o câmbio.
    else {
        let mut stmt =
            conn.prepare("SELECT currency FROM account UNION SELECT currency FROM asset")?;
        let used: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for c in used.iter().filter(|c| {
            c.as_str() != "BRL"
                && !crate::market::exchange::PTAX_CURRENCIES.contains(&c.as_str())
                && crate::market::exchange::ECB_CURRENCIES.contains(&c.as_str())
        }) {
            scope.insert(format!("fx_eur_{}", c.to_ascii_lowercase()));
        }
    }
    for (id, source, freq) in defs {
        let Some((provider, _)) = DataProvider::of_source(&source) else {
            continue;
        };
        if !crate::sources::enabled(conn, provider.source()) {
            continue;
        }
        if !provider.is_legacy() && !scope.contains(&id) {
            continue;
        }
        let last = if super::providers::full_refresh(&source, &freq) {
            None
        } else {
            last_date(conn, &id, true)?
        };
        let from = match last {
            // Mensais: refaz o último mês (revisões); diárias: a partir do dia seguinte.
            Some(d) if freq == "monthly" => d,
            Some(d) => d.succ_opt().expect("data válida"),
            None => parse_date(if freq == "monthly" {
                MONTHLY_START
            } else {
                DAILY_START
            })?,
        };
        if from <= today {
            p.series.push(PlanItem {
                series_id: id,
                source,
                from,
                to: today,
            });
        }
    }
    p.ipca_last_official = last_date(conn, "ipca", true)?;
    p.fred_api_key =
        crate::settings::get::<String>(conn, FRED_KEY_SETTING)?.filter(|k| !k.trim().is_empty());
    // Câmbio: moedas usadas em contas e ativos, sempre USD.
    let mut stmt = conn.prepare(
        "SELECT currency FROM account WHERE currency <> 'BRL' UNION SELECT currency FROM asset WHERE currency <> 'BRL' UNION SELECT 'USD'",
    )?;
    let ccys: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let ptax = crate::sources::enabled(conn, crate::sources::SourceId::Bcb);
    p.focus = ptax;
    for c in ccys.into_iter().filter(|_| ptax) {
        let last: Option<String> = conn
            .query_row(
                "SELECT max(date) FROM fx_rate WHERE base = ?1 AND quote = 'BRL' AND source = 'bcb_ptax_venda'",
                [&c],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        let from = match last {
            Some(d) => parse_date(&d)?.succ_opt().expect("data válida"),
            None => parse_date(FX_START)?,
        };
        if from <= today {
            p.fx.push(FxPlan {
                currency: c,
                from,
                to: today,
            });
        }
    }
    Ok(p)
}

pub(crate) fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(40)))
        .build()
        .into()
}

pub(crate) fn get_text(agent: &ureq::Agent, url: &str) -> std::result::Result<String, String> {
    agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json, text/csv, */*")
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .with_config()
        .limit(64 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())
}

/// Baixa tudo o que o plano pede. Não toca no banco.
pub fn fetch(plan: &Plan) -> Fetched {
    let agent = agent();
    let mut out = Fetched::default();
    // Tarefa em segundo plano: uma série ou moeda por passo; cancelada, o resto fica para depois.
    crate::tasks::set_total(Some((plan.series.len() + plan.fx.len()) as u64));
    for item in &plan.series {
        if crate::tasks::cancelled() {
            return out;
        }
        crate::tasks::detail(item.series_id.clone());
        crate::tasks::advance();
        let res = if let Some(code) = item.source.strip_prefix("bcb_sgs:") {
            fetch_sgs(&agent, code, item.from, item.to)
        } else if let Some(id) = item.source.strip_prefix("fred:") {
            // API oficial da FRED (chave gratuita). O CSV público recusa clientes que não
            // sejam navegadores ou ferramentas conhecidas, então não é usado.
            match plan.fred_api_key.as_deref() {
                Some(key) => get_text(
                    &agent,
                    &format!(
                        "https://api.stlouisfed.org/fred/series/observations?series_id={id}&api_key={key}&file_type=json&observation_start={}",
                        fmt_date(item.from)
                    ),
                )
                .and_then(|s| parse_fred_json(&s).map_err(|e| e.to_string()))
                .map(|v| v.into_iter().filter(|(d, _)| *d >= item.from && *d <= item.to).collect()),
                None => Err(no_fred_key()),
            }
        } else if let Some(sym) = item.source.strip_prefix("yahoo:") {
            // Benchmarks (Ibovespa, S&P 500): nível = fechamento diário.
            crate::market::yahoo::fetch(sym, item.from).map(|s| {
                s.bars
                    .into_iter()
                    .filter(|b| b.date >= item.from && b.date <= item.to)
                    .map(|b| (b.date, b.close))
                    .collect()
            })
        } else {
            // Provedores do catálogo de países: os valores já vêm marcados (o Banco Mundial
            // é todo estimado).
            out.marked.push((
                item.clone(),
                super::providers::fetch(&agent, &item.source, item.from, item.to),
            ));
            continue;
        };
        out.series.push((item.clone(), res));
    }
    for f in &plan.fx {
        if crate::tasks::cancelled() {
            return out;
        }
        crate::tasks::detail(f.currency.clone());
        crate::tasks::advance();
        let res = fetch_ptax(&agent, &f.currency, f.from, f.to);
        out.fx.push((f.clone(), res));
    }
    if plan.focus {
        out.focus = fetch_focus_ipca(&agent);
    }
    out
}

/// SGS limita séries diárias a janelas de 10 anos: baixa em blocos de 5 anos.
fn fetch_sgs(
    agent: &ureq::Agent,
    code: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> std::result::Result<Vec<(NaiveDate, Decimal)>, String> {
    let mut all = vec![];
    let mut start = from;
    while start <= to {
        let end = add_months(start, 60).min(to);
        let url = format!(
            "https://api.bcb.gov.br/dados/serie/bcdata.sgs.{code}/dados?formato=json&dataInicial={}&dataFinal={}",
            start.format("%d/%m/%Y"),
            end.format("%d/%m/%Y")
        );
        match get_text(agent, &url) {
            Ok(body) => all.extend(parse_sgs_json(&body).map_err(|e| e.to_string())?),
            // O SGS responde 404 quando não há dados no intervalo.
            Err(e) if e.contains("404") => {}
            Err(e) => return Err(e),
        }
        start = end.succ_opt().expect("data válida");
    }
    all.sort_by_key(|(d, _)| *d);
    all.dedup_by_key(|(d, _)| *d);
    Ok(all)
}

fn fetch_ptax(
    agent: &ureq::Agent,
    ccy: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> std::result::Result<Vec<PtaxQuote>, String> {
    let url = format!(
        "https://olinda.bcb.gov.br/olinda/servico/PTAX/versao/v1/odata/CotacaoMoedaPeriodo(moeda=@moeda,dataInicial=@dataInicial,dataFinalCotacao=@dataFinalCotacao)?@moeda='{ccy}'&@dataInicial='{}'&@dataFinalCotacao='{}'&$format=json&$filter=tipoBoletim%20eq%20'Fechamento'&$select=cotacaoCompra,cotacaoVenda,dataHoraCotacao,tipoBoletim",
        from.format("%m-%d-%Y"),
        to.format("%m-%d-%Y")
    );
    get_text(agent, &url).and_then(|s| parse_ptax_json(&s).map_err(|e| e.to_string()))
}

fn fetch_focus_ipca(agent: &ureq::Agent) -> std::result::Result<Vec<(NaiveDate, Decimal)>, String> {
    let since = add_months(today_local(), -2);
    let url = format!(
        "https://olinda.bcb.gov.br/olinda/servico/Expectativas/versao/v1/odata/ExpectativaMercadoMensais?$filter=Indicador%20eq%20'IPCA'%20and%20baseCalculo%20eq%200%20and%20Data%20ge%20'{}'&$orderby=Data%20desc&$top=2000&$format=json&$select=Data,DataReferencia,Mediana",
        fmt_date(since)
    );
    get_text(agent, &url).and_then(|s| parse_focus_json(&s).map_err(|e| e.to_string()))
}

/// Número JSON → decimal exato pela representação textual mais curta (sem ruído binário).
fn num(n: &serde_json::Number) -> Result<Decimal> {
    dec_str(&n.to_string())
}

fn dec_str(s: &str) -> Result<Decimal> {
    s.trim()
        .replace(',', ".")
        .parse::<Decimal>()
        .map_err(|_| super::providers::invalid_number(s))
}

/// `[{"data":"01/09/2026","valor":"0.82"}, …]`
pub fn parse_sgs_json(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    #[derive(Deserialize)]
    struct Row {
        data: String,
        valor: Option<String>,
    }
    let rows: Vec<Row> =
        serde_json::from_str(body).map_err(|e| super::providers::bad_response("SGS", e))?;
    rows.into_iter()
        .filter_map(|r| {
            r.valor
                .filter(|v| !v.trim().is_empty())
                .map(|v| (r.data, v))
        })
        .map(|(data, valor)| {
            let d = NaiveDate::parse_from_str(&data, "%d/%m/%Y").map_err(|_| {
                Error::invalid(crate::i18n::t_args(
                    "nucleo-ativos-erro-sgs-data",
                    &crate::i18n::args(&[("data", data.clone().into())]),
                ))
            })?;
            Ok((d, dec_str(&valor)?))
        })
        .collect()
}

/// Aviso das séries da FRED sem a chave cadastrada, no idioma da instalação.
pub fn no_fred_key() -> String {
    crate::i18n::t("nucleo-ativos-aviso-sem-chave-fred")
}

/// Ajuste com a chave da API da FRED (dentro do banco cifrado; nunca exportada).
pub const FRED_KEY_SETTING: &str = "market.fred_api_key";

/// Resposta da API da FRED: `{"observations":[{"date":"2026-09-01","value":"336.298"}]}`
/// (`.` = sem valor).
pub fn parse_fred_json(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    #[derive(Deserialize)]
    struct Obs {
        date: String,
        value: String,
    }
    #[derive(Deserialize)]
    struct Resp {
        observations: Option<Vec<Obs>>,
        error_message: Option<String>,
    }
    let r: Resp =
        serde_json::from_str(body).map_err(|e| super::providers::bad_response("FRED", e))?;
    if let Some(m) = r.error_message {
        return Err(Error::invalid(format!("FRED: {m}")));
    }
    let mut out = vec![];
    for o in r.observations.unwrap_or_default() {
        if o.value.trim() == "." || o.value.trim().is_empty() {
            continue;
        }
        out.push((parse_date(&o.date)?, dec_str(&o.value)?));
    }
    Ok(out)
}

/// CSV do FRED: `observation_date,CPIAUCSL` / `2026-09-01,336.298` (`.` = sem valor).
pub fn parse_fred_csv(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    let mut out = vec![];
    for line in body.lines().skip(1) {
        let Some((d, v)) = line.split_once(',') else {
            continue;
        };
        let v = v.trim();
        if v.is_empty() || v == "." {
            continue;
        }
        out.push((parse_date(d.trim())?, dec_str(v)?));
    }
    Ok(out)
}

pub fn parse_ptax_json(body: &str) -> Result<Vec<PtaxQuote>> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Row {
        cotacao_compra: serde_json::Number,
        cotacao_venda: serde_json::Number,
        data_hora_cotacao: String,
        tipo_boletim: Option<String>,
    }
    #[derive(Deserialize)]
    struct Resp {
        value: Vec<Row>,
    }
    let r: Resp =
        serde_json::from_str(body).map_err(|e| super::providers::bad_response("PTAX", e))?;
    let mut out: BTreeMap<NaiveDate, PtaxQuote> = BTreeMap::new();
    for row in r.value {
        if row
            .tipo_boletim
            .as_deref()
            .is_some_and(|t| t != "Fechamento")
        {
            continue;
        }
        let d = parse_date(row.data_hora_cotacao.get(..10).unwrap_or_default())?;
        out.insert(
            d,
            PtaxQuote {
                date: d,
                buy: num(&row.cotacao_compra)?,
                sell: num(&row.cotacao_venda)?,
            },
        );
    }
    Ok(out.into_values().collect())
}

/// Mediana mais recente do Focus para cada mês de referência (`DataReferencia` = "MM/AAAA").
pub fn parse_focus_json(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Row {
        data: String,
        data_referencia: String,
        mediana: serde_json::Number,
    }
    #[derive(Deserialize)]
    struct Resp {
        value: Vec<Row>,
    }
    let r: Resp =
        serde_json::from_str(body).map_err(|e| super::providers::bad_response("Focus", e))?;
    let mut best: BTreeMap<NaiveDate, (String, Decimal)> = BTreeMap::new();
    for row in r.value {
        let Some((m, y)) = row.data_referencia.split_once('/') else {
            continue;
        };
        let (Ok(m), Ok(y)) = (m.parse::<u32>(), y.parse::<i32>()) else {
            continue;
        };
        let Some(month) = NaiveDate::from_ymd_opt(y, m, 1) else {
            continue;
        };
        let keep = best.get(&month).is_none_or(|(d, _)| row.data > *d);
        if keep {
            best.insert(month, (row.data, num(&row.mediana)?));
        }
    }
    Ok(best.into_iter().map(|(m, (_, v))| (m, v)).collect())
}

/// Ajuste com o último erro de coleta de cada série (para o status dos provedores).
pub const SYNC_ERRORS_SETTING: &str = "indices.sync_errors";

/// Último erro de coleta por série: `{série: {at, error}}`. Sai quando a série volta a
/// ser coletada com sucesso.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncErrors(pub BTreeMap<String, SyncError>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub at: String,
    pub error: String,
}

impl SyncErrors {
    pub fn load(conn: &Connection) -> Result<Self> {
        Ok(crate::settings::get(conn, SYNC_ERRORS_SETTING)?.unwrap_or_default())
    }
    pub fn save(&self, conn: &Connection) -> Result<()> {
        crate::settings::set(conn, SYNC_ERRORS_SETTING, self)
    }
    fn record(&mut self, series_id: &str, error: &str) {
        self.0.insert(
            series_id.to_string(),
            SyncError {
                at: now_utc(),
                error: error.chars().take(300).collect(),
            },
        );
    }
    fn clear(&mut self, series_id: &str) {
        self.0.remove(series_id);
    }
}

/// Grava o que foi baixado. Também estima os meses de IPCA ainda não divulgados
/// (Focus; sem Focus, repete o último dado), sempre marcados como estimados.
pub fn apply(conn: &Connection, fetched: &Fetched, today: NaiveDate) -> Result<SyncReport> {
    let mut report = SyncReport {
        started_at: now_utc(),
        ok: true,
        ..Default::default()
    };
    // SGS/FRED datam séries mensais no 1º dia do mês de referência; todos oficiais.
    let legacy = fetched.series.iter().map(|(item, res)| {
        (
            item,
            res.as_ref()
                .map(|v| v.iter().map(|(d, x)| (*d, *x, false)).collect::<Vec<_>>())
                .map_err(Clone::clone),
        )
    });
    let marked = fetched.marked.iter().map(|(item, res)| (item, res.clone()));
    let mut errors = SyncErrors::load(conn)?;
    for (item, res) in legacy.chain(marked) {
        let mut r = SeriesSyncResult {
            series_id: item.series_id.clone(),
            written: 0,
            last_date: None,
            error: None,
        };
        match res {
            Ok(vals) => {
                r.written =
                    super::upsert_values(conn, &item.series_id, &vals, &item.source)? as u32;
                if let Some((p, _)) = DataProvider::of_source(&item.source) {
                    super::providers::mirror_fx(conn, &item.series_id, p, &vals)?;
                }
                r.last_date = last_date(conn, &item.series_id, true)?
                    .or(last_date(conn, &item.series_id, false)?)
                    .map(fmt_date);
                errors.clear(&item.series_id);
            }
            Err(e) => {
                report.ok = false;
                errors.record(&item.series_id, &e);
                r.error = Some(e);
            }
        }
        report.series.push(r);
    }
    errors.save(conn)?;
    for (f, res) in &fetched.fx {
        let mut r = SeriesSyncResult {
            series_id: format!("{}/BRL", f.currency),
            written: 0,
            last_date: None,
            error: None,
        };
        match res {
            Ok(quotes) => {
                let now = now_utc();
                let mut stmt = conn.prepare(
                    "INSERT INTO fx_rate (id, base, quote, date, rate, source, fetched_at, created_at, updated_at)
                     VALUES (?1, ?2, 'BRL', ?3, ?4, ?5, ?6, ?6, ?6)
                     ON CONFLICT (base, quote, date, source) DO UPDATE SET rate = excluded.rate, fetched_at = excluded.fetched_at",
                )?;
                for q in quotes {
                    for (source, v) in [("bcb_ptax_venda", q.sell), ("bcb_ptax_compra", q.buy)] {
                        r.written += stmt.execute(params![
                            new_id(),
                            f.currency,
                            fmt_date(q.date),
                            v.normalize().to_string(),
                            source,
                            now
                        ])? as u32;
                    }
                }
                r.last_date = quotes.last().map(|q| fmt_date(q.date));
            }
            Err(e) => {
                report.ok = false;
                r.error = Some(e.clone());
            }
        }
        report.fx.push(r);
    }
    report.ipca_estimated_months = estimate_ipca(
        conn,
        fetched
            .focus
            .as_ref()
            .ok()
            .map(|v| v.as_slice())
            .unwrap_or(&[]),
        today,
    )?;
    report.finished_at = now_utc();
    Ok(report)
}

/// Preenche os meses do IPCA depois do último oficial até o mês corrente.
pub fn estimate_ipca(
    conn: &Connection,
    focus: &[(NaiveDate, Decimal)],
    today: NaiveDate,
) -> Result<u32> {
    let Some(last) = last_date(conn, "ipca", true)? else {
        return Ok(0);
    };
    let last_value: Option<String> = conn
        .query_row(
            "SELECT value FROM index_value WHERE series_id = 'ipca' AND date = ?1",
            [fmt_date(last)],
            |r| r.get(0),
        )
        .optional()?;
    let current = today.with_day(1).expect("dia 1");
    let mut month = add_months(last, 1);
    let mut vals = vec![];
    while month <= current {
        let (v, src) = match focus.iter().find(|(m, _)| *m == month) {
            Some((_, v)) => (*v, "bcb_focus"),
            None => match &last_value {
                Some(lv) => (dec_str(lv)?, "repeat_last"),
                None => break,
            },
        };
        vals.push((month, v, true, src));
        month = add_months(month, 1);
    }
    let mut n = 0;
    for (m, v, est, src) in vals {
        n += super::upsert_values(conn, "ipca", &[(m, v, est)], src)? as u32;
    }
    Ok(n)
}

/// Coleta completa usando a API (trava o banco só para planejar e gravar).
pub fn run(api: &crate::Api) -> Result<SyncReport> {
    use crate::tasks::{self, TaskKind, TaskOrigin, TaskStage};
    // Dentro da coleta geral é uma etapa dela; pedida sozinha, é tarefa própria.
    tasks::run(TaskKind::IndicesSync, TaskOrigin::User, true, || {
        tasks::stage(TaskStage::Indices);
        let today = today_local();
        let plan = api.with_session(|s, _| plan(s.db.conn(), today))?;
        let fetched = fetch(&plan);
        tasks::check()?;
        api.with_session(|s, _| apply(s.db.conn(), &fetched, today))
    })
}

/// Ajuste com o instante da última coleta automática.
pub const LAST_SYNC_SETTING: &str = "market.last_sync_at";

/// Coleta automática (rotina do app): roda se o cofre estiver aberto e a última coleta
/// tiver mais de `min_hours`. Devolve `None` quando não era hora.
pub fn run_if_due(api: &crate::Api, min_hours: i64) -> Result<Option<SyncReport>> {
    if !api.is_unlocked() {
        return Ok(None);
    }
    let last: Option<String> =
        api.with_session(|s, _| crate::settings::get(s.db.conn(), LAST_SYNC_SETTING))?;
    let due = match last
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
    {
        Some(t) => (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() >= min_hours,
        None => true,
    };
    if !due {
        return Ok(None);
    }
    let report = run(api)?;
    api.with_session(|s, _| {
        crate::settings::set(s.db.conn(), LAST_SYNC_SETTING, &report.finished_at)
    })?;
    Ok(Some(report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn parses_sources() {
        let sgs = parse_sgs_json(
            r#"[{"data":"01/08/2026","valor":"-0.32"},{"data":"01/09/2026","valor":"0.82"}]"#,
        )
        .unwrap();
        assert_eq!(
            sgs,
            vec![
                (d("2026-08-01"), dec!(-0.32)),
                (d("2026-09-01"), dec!(0.82))
            ]
        );
        let fred = parse_fred_csv("observation_date,CPIAUCSL\n2026-08-01,334.131\n2026-09-01,.\n")
            .unwrap();
        assert_eq!(fred, vec![(d("2026-08-01"), dec!(334.131))]);
        let ptax = parse_ptax_json(
            r#"{"value":[{"cotacaoCompra":5.2,"cotacaoVenda":5.21,"dataHoraCotacao":"2026-10-01 10:10:11.5","tipoBoletim":"Abertura"},
                         {"cotacaoCompra":5.3,"cotacaoVenda":5.31,"dataHoraCotacao":"2026-10-01 13:09:00.1","tipoBoletim":"Fechamento"}]}"#,
        )
        .unwrap();
        assert_eq!(
            ptax,
            vec![PtaxQuote {
                date: d("2026-10-01"),
                buy: dec!(5.3),
                sell: dec!(5.31)
            }]
        );
        let focus = parse_focus_json(
            r#"{"value":[{"Data":"2026-10-09","DataReferencia":"10/2026","Mediana":0.30},
                         {"Data":"2026-10-02","DataReferencia":"10/2026","Mediana":0.40},
                         {"Data":"2026-10-09","DataReferencia":"11/2026","Mediana":0.25}]}"#,
        )
        .unwrap();
        assert_eq!(
            focus,
            vec![(d("2026-10-01"), dec!(0.30)), (d("2026-11-01"), dec!(0.25))]
        );
        assert!(parse_sgs_json("<html>").is_err());
        let with_nulls = parse_sgs_json(
            r#"[{"data":"01/08/2001","valor":null},{"data":"01/12/2001","valor":"324300744"}]"#,
        )
        .unwrap();
        assert_eq!(with_nulls, vec![(d("2001-12-01"), dec!(324300744))]);
        let fred = parse_fred_json(r#"{"observations":[{"date":"2026-08-01","value":"334.131"},{"date":"2026-09-01","value":"."}]}"#).unwrap();
        assert_eq!(fred, vec![(d("2026-08-01"), dec!(334.131))]);
        assert!(parse_fred_json(r#"{"error_code":400,"error_message":"Bad Request"}"#).is_err());
    }

    #[test]
    fn plan_apply_and_estimates() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        let today = d("2026-10-14");
        let p = plan(c, today).unwrap();
        let ipca = p.series.iter().find(|i| i.series_id == "ipca").unwrap();
        assert_eq!(ipca.from, d(MONTHLY_START));
        assert!(p.fx.iter().any(|f| f.currency == "USD"));

        let fetched = Fetched {
            series: vec![
                (
                    ipca.clone(),
                    Ok(vec![
                        (d("2026-08-01"), dec!(-0.32)),
                        (d("2026-09-01"), dec!(0.82)),
                    ]),
                ),
                (
                    p.series
                        .iter()
                        .find(|i| i.series_id == "cdi")
                        .unwrap()
                        .clone(),
                    Err("rede indisponível".into()),
                ),
            ],
            fx: vec![(
                p.fx[0].clone(),
                Ok(vec![PtaxQuote {
                    date: d("2026-10-13"),
                    buy: dec!(5.40),
                    sell: dec!(5.41),
                }]),
            )],
            focus: Ok(vec![(d("2026-10-01"), dec!(0.30))]),
            ..Default::default()
        };
        let r = apply(c, &fetched, today).unwrap();
        assert!(!r.ok, "a falha do CDI aparece no relatório");
        assert_eq!(
            r.series
                .iter()
                .find(|s| s.series_id == "ipca")
                .unwrap()
                .last_date
                .as_deref(),
            Some("2026-09-01")
        );
        assert_eq!(r.ipca_estimated_months, 1);
        let pts = crate::indices::points(c, "ipca", d("2026-10-01"), d("2026-10-01")).unwrap();
        assert_eq!(pts[0].value, dec!(0.30));
        assert!(pts[0].estimated);
        // PTAX venda vira a taxa padrão de conversão.
        let conv = crate::fx::convert(c, 10000, "USD", "BRL", "2026-10-14")
            .unwrap()
            .unwrap();
        assert_eq!(conv.amount, 54100);
        // O próximo plano recomeça do último mês oficial (revisões) e do dia seguinte no câmbio.
        let p2 = plan(c, today).unwrap();
        assert_eq!(
            p2.series
                .iter()
                .find(|i| i.series_id == "ipca")
                .unwrap()
                .from,
            d("2026-09-01")
        );
        assert_eq!(
            p2.fx.iter().find(|f| f.currency == "USD").unwrap().from,
            d("2026-10-14")
        );
    }

    /// No Brasil, moeda sem PTAX vem do BCE e chega ao real cruzando pelo euro.
    #[test]
    fn currencies_without_ptax_come_from_the_ecb_through_the_euro() {
        use crate::accounts::AccountKind;
        let mut fx = crate::transactions::testkit::Fx::new();
        let today = d("2026-10-14");
        let ids = |c: &Connection| -> Vec<String> {
            plan(c, today)
                .unwrap()
                .series
                .into_iter()
                .map(|i| i.series_id)
                .collect()
        };
        assert!(!ids(fx.conn()).iter().any(|i| i.starts_with("fx_eur_")));
        fx.account("Conta em Hong Kong", AccountKind::Checking, "HKD", 0);
        // Com PTAX (dólar) ou sem BCE (novo dólar taiwanês), nada muda.
        fx.account("Conta nos EUA", AccountKind::Checking, "USD", 0);
        fx.account("Conta em Taiwan", AccountKind::Checking, "TWD", 0);
        let got = ids(fx.conn());
        let ecb: Vec<&String> = got.iter().filter(|i| i.starts_with("fx_eur_")).collect();
        assert_eq!(ecb, ["fx_eur_hkd"]);

        // 1 EUR = 9 HKD (BCE) e 1 EUR = 6,30 BRL (PTAX): 900 HKD = 100 EUR = 630 BRL.
        let c = fx.conn();
        super::super::providers::mirror_fx(
            c,
            "fx_eur_hkd",
            DataProvider::Ecb,
            &[(d("2026-10-13"), dec!(9), false)],
        )
        .unwrap();
        c.execute(
            "INSERT INTO fx_rate (id, base, quote, date, rate, source, fetched_at, created_at, updated_at)
             VALUES ('t1', 'EUR', 'BRL', '2026-10-13', '6.30', 'bcb_ptax_venda', 'x', 'x', 'x')",
            [],
        )
        .unwrap();
        let conv = crate::fx::convert(c, 90000, "HKD", "BRL", "2026-10-14")
            .unwrap()
            .unwrap();
        assert_eq!(conv.amount, 63000);
        assert_eq!(conv.source.as_deref(), Some("ecb_ref × bcb_ptax_venda"));
    }

    #[test]
    fn plan_follows_reference_country() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        let today = d("2026-10-14");
        // Brasil: exatamente as séries de sempre (BCB, FRED, Yahoo e o IPCA por grupo do IBGE).
        let br = plan(c, today).unwrap();
        assert!(br.series.iter().all(|i| {
            ["bcb_sgs:", "fred:", "yahoo:", "ibge_sidra:"]
                .iter()
                .any(|p| i.source.starts_with(p))
        }));
        let legacy: i64 = c
            .query_row(
                "SELECT count(*) FROM index_series WHERE source LIKE 'bcb_sgs:%' OR source LIKE 'fred:%' OR source LIKE 'yahoo:%' OR source LIKE 'ibge_sidra:%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(br.series.len() as i64, legacy);

        crate::indices::catalog::set_reference_country(c, Some("PT")).unwrap();
        let pt = plan(c, today).unwrap();
        let ids: Vec<&str> = pt.series.iter().map(|i| i.series_id.as_str()).collect();
        for id in ["ipca", "cdi", "hicp_pt", "wb_cpi_pt", "estr", "fx_eur_usd"] {
            assert!(ids.contains(&id), "{id} fora do plano: {ids:?}");
        }
        assert!(!ids.contains(&"cpi_jp"));
        let hicp = pt
            .series
            .iter()
            .find(|i| i.series_id == "hicp_pt")
            .unwrap()
            .clone();
        let wb = pt
            .series
            .iter()
            .find(|i| i.series_id == "wb_cpi_pt")
            .unwrap()
            .clone();
        let fx = pt
            .series
            .iter()
            .find(|i| i.series_id == "fx_eur_usd")
            .unwrap()
            .clone();
        let estr = pt
            .series
            .iter()
            .find(|i| i.series_id == "estr")
            .unwrap()
            .clone();

        let fetched = Fetched {
            marked: vec![
                (
                    hicp.clone(),
                    Ok(vec![
                        (d("2026-08-01"), dec!(104.07), false),
                        (d("2026-09-01"), dec!(105.08), false),
                    ]),
                ),
                (wb, Ok(vec![(d("2025-07-01"), dec!(150), true)])),
                (fx, Ok(vec![(d("2026-10-13"), dec!(1.1188), false)])),
                (estr, Err("BCE fora do ar".into())),
            ],
            ..Default::default()
        };
        let r = apply(c, &fetched, today).unwrap();
        assert!(!r.ok);
        let wbr = r
            .series
            .iter()
            .find(|s| s.series_id == "wb_cpi_pt")
            .unwrap();
        assert_eq!(wbr.last_date.as_deref(), Some("2025-07-01"), "só estimados");
        assert!(
            crate::indices::points(c, "wb_cpi_pt", d("2025-01-01"), today).unwrap()[0].estimated
        );
        // Câmbio espelhado para as conversões fora do real.
        let conv = crate::fx::convert(c, 10000, "EUR", "USD", "2026-10-14")
            .unwrap()
            .unwrap();
        assert_eq!(conv.amount, 11188);
        // Status dos provedores: BCE com erro (€STR), Eurostat fora de uso.
        let st = crate::indices::api::providers_status(c).unwrap();
        let ecb = st.iter().find(|p| p.info.id == DataProvider::Ecb).unwrap();
        assert_eq!(ecb.status, "error");
        assert!(ecb.error.as_deref().unwrap().contains("BCE fora do ar"));
        assert_eq!(ecb.last_date.as_deref(), Some("2026-10-13"));
        let es = st
            .iter()
            .find(|p| p.info.id == DataProvider::Eurostat)
            .unwrap();
        assert_eq!(es.status, "idle");
        // Mensais dos provedores novos são sempre baixadas inteiras; diárias continuam.
        let p2 = plan(c, today).unwrap();
        let h2 = p2.series.iter().find(|i| i.series_id == "hicp_pt").unwrap();
        assert_eq!(h2.from, d(MONTHLY_START));
        let f2 = p2
            .series
            .iter()
            .find(|i| i.series_id == "fx_eur_usd")
            .unwrap();
        assert_eq!(f2.from, d("2026-10-14"));
        // A nova coleta do €STR limpa o erro.
        let ok = Fetched {
            marked: vec![(
                p2.series
                    .iter()
                    .find(|i| i.series_id == "estr")
                    .unwrap()
                    .clone(),
                Ok(vec![(d("2026-10-13"), dec!(2.439), false)]),
            )],
            ..Default::default()
        };
        apply(c, &ok, today).unwrap();
        let st = crate::indices::api::providers_status(c).unwrap();
        assert_eq!(
            st.iter()
                .find(|p| p.info.id == DataProvider::Ecb)
                .unwrap()
                .status,
            "ok"
        );
    }

    /// Coleta real das séries de alguns países (sem as de sempre) e o deflator do país:
    /// `cargo test -p treasury-core -- --ignored live_country_sync --nocapture`.
    #[test]
    #[ignore = "usa a rede"]
    fn live_country_sync() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        let today = today_local();
        let only = std::env::var("TREASURY_LIVE_COUNTRIES").unwrap_or_default();
        for country in ["PT", "PL", "JP", "CA", "GB", "NZ", "US", "AR"] {
            if !only.is_empty() && !only.split(',').any(|c| c == country) {
                continue;
            }
            crate::indices::catalog::set_reference_country(c, Some(country)).unwrap();
            let mut p = plan(c, today).unwrap();
            p.series.retain(|i| {
                DataProvider::of_source(&i.source).is_some_and(|(pr, _)| !pr.is_legacy())
            });
            p.fx.clear();
            let f = fetch(&p);
            let r = apply(c, &f, today).unwrap();
            for s in &r.series {
                println!(
                    "{country} {:<14} {:>5} novos, até {:?} {}",
                    s.series_id,
                    s.written,
                    s.last_date,
                    s.error.as_deref().unwrap_or("")
                );
            }
            let f12 = crate::indices::deflator_factor(
                c,
                crate::indices::Deflator::Country,
                add_months(today, -12),
                today,
            )
            .unwrap();
            println!("{country}: inflação 12 meses = {f12:?}");
            // Argentina: o Banco Mundial não publicou 2025 e a projeção para em 24 meses.
            assert!(f12.is_some() || country == "AR", "{country} sem deflator");
        }
    }

    #[test]
    #[ignore = "usa a rede: cargo test -- --ignored live_fetch"]
    fn live_fetch() {
        let today = today_local();
        let p = Plan {
            series: vec![PlanItem {
                series_id: "ipca".into(),
                source: "bcb_sgs:433".into(),
                from: add_months(today, -6),
                to: today,
            }],
            fx: vec![FxPlan {
                currency: "USD".into(),
                from: add_months(today, -1),
                to: today,
            }],
            ipca_last_official: None,
            fred_api_key: None,
            focus: true,
        };
        let f = fetch(&p);
        assert!(f.series[0].1.as_ref().unwrap().len() >= 4);
        assert!(!f.fx[0].1.as_ref().unwrap().is_empty());
        assert!(!f.focus.unwrap().is_empty());
    }
}
