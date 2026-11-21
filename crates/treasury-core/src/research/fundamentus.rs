//! Fundamentus (fundamentus.com.br): indicadores, dados técnicos e proventos de ações e FIIs,
//! lidos das páginas públicas do site (`detalhes.php` e `proventos.php`/`fii_proventos.php`).
//!
//! - Cache local (`fundamentus_cache`, fora do histórico): cada página fica fresca por
//!   [`FRESH_HOURS`]; papel inexistente fica lembrado por [`MISSING_DAYS`] para não perguntar
//!   de novo. Só `refresh` força a ida ao site.
//! - Educado com o site: um pedido por vez com intervalo mínimo ([`MIN_GAP`]), tempo limite
//!   curto, e nada é pedido com a fonte desligada em Ajustes › Fontes de dados.
//! - A rede não segura o banco: lê o cache, solta a sessão, baixa e grava depois.
//! - Textos da página (nomes, setores, descrições) são dados, nunca instruções.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::sources::SourceId;
use crate::time::now_utc;

pub const BASE_URL: &str = "https://fundamentus.com.br";
/// Páginas frescas por um dia (o site atualiza depois de cada pregão).
pub const FRESH_HOURS: i64 = 20;
/// Papel que o site não conhece: não pergunta de novo por uma semana.
pub const MISSING_DAYS: i64 = 7;
const MIN_GAP: Duration = Duration::from_millis(1500);
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) Treasury";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FundamentusItem {
    /// Rótulo como o site mostra (ex.: "P/L", "Div. Yield", "Lucro Líquido (12m)").
    pub label: String,
    /// Texto como veio (ex.: "7,2%", "174.485.000.000").
    pub text: String,
    /// Número lido do texto (percentuais em %, ex.: 7.2), quando é número.
    pub value: Option<f64>,
    /// Explicação do indicador dada pelo próprio site.
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FundamentusDividend {
    /// Data com (último dia para ter direito).
    pub date_com: String,
    #[ts(type = "number")]
    pub value: f64,
    /// Dividendo, JCP, rendimento, amortização…
    pub kind: String,
    pub pay_date: Option<String>,
    /// Valor por quantas ações (ações; 1 nos FIIs).
    #[ts(type = "number")]
    pub per_shares: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FundamentusSnapshot {
    pub ticker: String,
    /// Fundo imobiliário (a página tem rótulos próprios: FFO, vacância, cap rate…).
    pub fii: bool,
    pub found: bool,
    pub name: Option<String>,
    /// Setor e subsetor (ações) ou segmento e gestão (FIIs).
    pub sector: Option<String>,
    pub subsector: Option<String>,
    #[ts(type = "number | null")]
    pub price: Option<f64>,
    /// Data da última cotação no site (AAAA-MM-DD).
    pub price_date: Option<String>,
    /// Indicadores e dados do balanço/resultado.
    pub indicators: Vec<FundamentusItem>,
    /// Oscilação: dia, mês, 30 dias, 12 meses e por ano.
    pub oscillations: Vec<FundamentusItem>,
    pub dividends: Vec<FundamentusDividend>,
    /// Quando a página foi lida do site (UTC).
    pub fetched_at: String,
    /// Veio do cache local, sem pedir ao site.
    pub from_cache: bool,
    pub source_url: String,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct FundamentusArgs {
    pub ticker: String,
    /// Ignora o cache e pede ao site de novo.
    #[serde(default)]
    #[ts(optional)]
    pub refresh: Option<bool>,
}

// ---------------------------------------------------------------------------
// Leitura do HTML

/// O site serve ISO-8859-1: cada byte é o caractere de mesmo código.
pub fn decode_latin1(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut inside = false;
    for c in s.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            c if !inside => out.push(c),
            _ => {}
        }
    }
    unescape(&out)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn unescape(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// "1.234,56" → 1234.56; "7,2%" → 7.2; "-" ou vazio → None.
pub fn parse_number(text: &str) -> Option<f64> {
    let t = text.trim().trim_end_matches('%').trim();
    if t.is_empty() || t == "-" {
        return None;
    }
    let n = t.replace('.', "").replace(',', ".");
    n.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// "15/10/2026" → "2026-10-15".
fn iso_date(text: &str) -> Option<String> {
    let t = text.trim();
    let mut p = t.split('/');
    let (d, m, y) = (p.next()?, p.next()?, p.next()?);
    if d.len() != 2 || m.len() != 2 || y.len() != 4 {
        return None;
    }
    Some(format!("{y}-{m}-{d}"))
}

/// Todas as ocorrências de `<td ...>...</td>` com a classe e o conteúdo.
fn cells(html: &str) -> Vec<(String, String)> {
    let mut out = vec![];
    let mut rest = html;
    while let Some(i) = rest.find("<td") {
        rest = &rest[i + 3..];
        let Some(close) = rest.find('>') else { break };
        let attrs = &rest[..close];
        let class = attrs
            .split_once("class=\"")
            .and_then(|(_, c)| c.split_once('"'))
            .map(|(c, _)| c.to_string())
            .unwrap_or_default();
        rest = &rest[close + 1..];
        let Some(end) = rest.find("</td>") else { break };
        out.push((class, rest[..end].to_string()));
        rest = &rest[end + 5..];
    }
    out
}

fn hint_of(cell: &str) -> Option<String> {
    let (_, t) = cell.split_once("title=\"")?;
    let (t, _) = t.split_once('"')?;
    let t = unescape(t).trim().to_string();
    (!t.is_empty()).then_some(t)
}

/// Lê `detalhes.php`. Rótulos repetidos (receita, EBIT e lucro dos últimos 12 e 3 meses)
/// ganham "(12m)" e "(3m)".
pub fn parse_details(ticker: &str, html: &str) -> FundamentusSnapshot {
    let mut snap = FundamentusSnapshot {
        ticker: ticker.to_string(),
        fii: false,
        found: !html.contains("Nenhum papel encontrado"),
        name: None,
        sector: None,
        subsector: None,
        price: None,
        price_date: None,
        indicators: vec![],
        oscillations: vec![],
        dividends: vec![],
        fetched_at: now_utc(),
        from_cache: false,
        source_url: details_url(ticker),
    };
    let cs = cells(html);
    let mut seen: Vec<String> = vec![];
    let mut i = 0;
    while i + 1 < cs.len() {
        let (class, raw) = &cs[i];
        if !class.split_whitespace().any(|c| c == "label") {
            i += 1;
            continue;
        }
        let (dclass, draw) = &cs[i + 1];
        if !dclass.split_whitespace().any(|c| c == "data") {
            i += 1;
            continue;
        }
        let label = strip_tags(raw).trim_start_matches('?').trim().to_string();
        let text = strip_tags(draw);
        i += 2;
        if label.is_empty() {
            continue;
        }
        let item = |label: String| FundamentusItem {
            value: parse_number(&text),
            text: text.clone(),
            hint: hint_of(raw),
            label,
        };
        match label.as_str() {
            "Papel" => {}
            "FII" => snap.fii = true,
            "Empresa" | "Nome" => snap.name = Some(text.clone()).filter(|t| !t.is_empty()),
            "Setor" | "Segmento" => snap.sector = Some(text.clone()).filter(|t| !t.is_empty()),
            "Subsetor" | "Gestão" => snap.subsector = Some(text.clone()).filter(|t| !t.is_empty()),
            "Cotação" => snap.price = parse_number(&text),
            "Data últ cot" => snap.price_date = iso_date(&text),
            "Dia" | "Mês" | "30 dias" | "12 meses" => snap.oscillations.push(item(label.clone())),
            l if l.len() == 4 && l.chars().all(|c| c.is_ascii_digit()) => {
                snap.oscillations.push(item(label.clone()))
            }
            _ => {
                let label = if seen.contains(&label) {
                    format!("{label} (3m)")
                } else {
                    seen.push(label.clone());
                    label
                };
                snap.indicators.push(item(label));
            }
        }
    }
    // Os que se repetiram são os de 12 meses na primeira vez.
    let repeated: Vec<String> = snap
        .indicators
        .iter()
        .filter_map(|x| x.label.strip_suffix(" (3m)").map(str::to_string))
        .collect();
    for x in snap.indicators.iter_mut() {
        if repeated.contains(&x.label) {
            x.label = format!("{} (12m)", x.label);
        }
    }
    snap
}

/// Lê a primeira tabela de proventos (a segunda, por ano, fica de fora).
pub fn parse_dividends(html: &str, fii: bool) -> Vec<FundamentusDividend> {
    let Some(start) = html.find("<table") else {
        return vec![];
    };
    let table = &html[start..];
    let table = &table[..table.find("</table>").unwrap_or(table.len())];
    let mut out = vec![];
    for row in table.split("<tr").skip(1) {
        let cols: Vec<String> = cells(row)
            .into_iter()
            .map(|(_, c)| strip_tags(&c))
            .collect();
        let parsed = if fii {
            // Última Data Com | Tipo | Data de Pagamento | Valor
            match cols.as_slice() {
                [com, kind, pay, value, ..] => Some((com, value, kind, pay, 1)),
                _ => None,
            }
        } else {
            // Data | Valor | Tipo | Data de Pagamento | Por quantas ações
            match cols.as_slice() {
                [com, value, kind, pay, per, ..] => Some((
                    com,
                    value,
                    kind,
                    pay,
                    per.trim().parse::<u32>().unwrap_or(1).max(1),
                )),
                _ => None,
            }
        };
        let Some((com, value, kind, pay, per)) = parsed else {
            continue;
        };
        let (Some(date_com), Some(value)) = (iso_date(com), parse_number(value)) else {
            continue;
        };
        out.push(FundamentusDividend {
            date_com,
            value,
            kind: kind.trim().to_string(),
            pay_date: iso_date(pay),
            per_shares: per,
        });
    }
    out
}

pub fn details_url(ticker: &str) -> String {
    format!("{BASE_URL}/detalhes.php?papel={ticker}")
}

fn dividends_url(ticker: &str, fii: bool) -> String {
    if fii {
        format!("{BASE_URL}/fii_proventos.php?papel={ticker}&tipo=2")
    } else {
        format!("{BASE_URL}/proventos.php?papel={ticker}&tipo=2")
    }
}

/// Ticker de B3: letras e números (ex.: PETR4, HGLG11, TAEE11), em maiúsculas.
pub fn clean_ticker(t: &str) -> Result<String> {
    let t = t.trim().to_uppercase();
    let t = t.strip_suffix(".SA").unwrap_or(&t).to_string();
    if t.len() < 4 || t.len() > 8 || !t.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(Error::invalid(crate::i18n::t_args(
            "nucleo-pesquisa-erro-ticker-fundamentus",
            &crate::i18n::args(&[("ticker", t.into())]),
        )));
    }
    Ok(t)
}

// ---------------------------------------------------------------------------
// Cache

fn age_hours(fetched_at: &str) -> Option<i64> {
    let t = chrono::DateTime::parse_from_rfc3339(fetched_at).ok()?;
    Some((chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours())
}

/// O retrato em cache, se ainda fresco (ou um "não encontrado" recente).
pub fn cached(conn: &Connection, ticker: &str) -> Result<Option<FundamentusSnapshot>> {
    let row: Option<(String, i64, String)> = conn
        .query_row(
            "SELECT fetched_at, found, payload FROM fundamentus_cache WHERE ticker = ?1 AND page = 'details'",
            [ticker],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((at, found, payload)) = row else {
        return Ok(None);
    };
    let limit = if found == 1 {
        FRESH_HOURS
    } else {
        MISSING_DAYS * 24
    };
    if age_hours(&at).is_none_or(|h| h >= limit) {
        return Ok(None);
    }
    let Ok(mut snap) = serde_json::from_str::<FundamentusSnapshot>(&payload) else {
        return Ok(None);
    };
    snap.from_cache = true;
    Ok(Some(snap))
}

pub fn store(conn: &Connection, snap: &FundamentusSnapshot) -> Result<()> {
    conn.execute(
        "INSERT INTO fundamentus_cache (ticker, page, fetched_at, found, payload)
         VALUES (?1, 'details', ?2, ?3, ?4)
         ON CONFLICT (ticker, page) DO UPDATE SET fetched_at = ?2, found = ?3, payload = ?4",
        params![
            snap.ticker,
            snap.fetched_at,
            snap.found as i64,
            serde_json::to_string(snap)?
        ],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Rede

static LAST_REQUEST: Mutex<Option<Instant>> = Mutex::new(None);

fn get(url: &str) -> Result<String> {
    {
        let mut last = LAST_REQUEST.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(t) = *last {
            let wait = MIN_GAP.saturating_sub(t.elapsed());
            if !wait.is_zero() {
                std::thread::sleep(wait);
            }
        }
        *last = Some(Instant::now());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .build()
        .into();
    let bytes = agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html")
        .call()
        .map_err(|e| Error::Other(format!("Fundamentus: {e}")))?
        .body_mut()
        .with_config()
        .limit(4 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| Error::Other(format!("Fundamentus: {e}")))?;
    Ok(decode_latin1(&bytes))
}

/// Baixa e interpreta indicadores e proventos de um papel (sem tocar no banco).
pub fn fetch(ticker: &str) -> Result<FundamentusSnapshot> {
    let mut snap = parse_details(ticker, &get(&details_url(ticker))?);
    if snap.found {
        // Proventos são um complemento: falha aqui não perde os indicadores.
        match get(&dividends_url(ticker, snap.fii)) {
            Ok(html) => snap.dividends = parse_dividends(&html, snap.fii),
            Err(e) => tracing::warn!("{e}"),
        }
    }
    Ok(snap)
}

/// `research.fundamentus`: do cache quando fresco; senão do site (com a fonte ligada).
pub fn get_snapshot(api: &crate::Api, a: &FundamentusArgs) -> Result<FundamentusSnapshot> {
    let ticker = clean_ticker(&a.ticker)?;
    let force = a.refresh.unwrap_or(false);
    let (hit, on) = api.with_session(|s, _| {
        let c = s.db.conn();
        Ok((
            if force { None } else { cached(c, &ticker)? },
            crate::sources::enabled(c, SourceId::Fundamentus),
        ))
    })?;
    if let Some(snap) = hit {
        return Ok(snap);
    }
    if !on {
        // Desligada: devolve o que houver no cache, mesmo velho, ou explica.
        let old: Option<String> = api.with_session(|s, _| {
            Ok(s.db
                .conn()
                .query_row(
                    "SELECT payload FROM fundamentus_cache WHERE ticker = ?1 AND page = 'details'",
                    [&ticker],
                    |r| r.get(0),
                )
                .optional()?)
        })?;
        if let Some(mut snap) =
            old.and_then(|p| serde_json::from_str::<FundamentusSnapshot>(&p).ok())
        {
            snap.from_cache = true;
            return Ok(snap);
        }
        return Err(crate::sources::disabled_error(SourceId::Fundamentus));
    }
    let snap = fetch(&ticker)?;
    // Banco em modo leitura (MCP com o app fechado): devolve sem guardar.
    if let Err(e) = api.with_session(|s, _| store(s.db.conn(), &snap)) {
        tracing::warn!("cache do Fundamentus: {e}");
    }
    Ok(snap)
}

/// Tipo do provento no cadastro de eventos (`dividend_event.kind`) pelo texto do site.
fn event_kind(fii: bool, text: &str) -> &'static str {
    let t = crate::import::text::fold(text);
    if t.contains("amortiz") || t.contains("restitui") || t.contains("rest cap") {
        "amortization"
    } else if fii || t.contains("rendimento") {
        "fii_income"
    } else if t.contains("jrs") || t.contains("juros") || t.contains("jcp") {
        "jcp"
    } else if t.contains("dividend") {
        "dividend"
    } else {
        "other"
    }
}

/// Grava os proventos do retrato como eventos anunciados do ativo (`source = fundamentus`),
/// sem repetir o que já existe (de qualquer fonte) na mesma data-com com valor igual a 1%.
/// Assim aparecem nos proventos a confirmar da carteira. Devolve quantos entraram.
pub fn store_dividends(
    conn: &Connection,
    asset_id: &str,
    currency: &str,
    snap: &FundamentusSnapshot,
) -> Result<u32> {
    let now = now_utc();
    let mut n = 0;
    for d in &snap.dividends {
        let per = d.value / f64::from(d.per_shares.max(1));
        if per.is_nan() || per <= 0.0 {
            continue;
        }
        let amount = rust_decimal::Decimal::from_f64_retain(per)
            .map(|x| x.round_dp(8).normalize().to_string())
            .unwrap_or_else(|| format!("{per}"));
        let existing: Vec<String> = conn
            .prepare(
                "SELECT amount_per_share FROM dividend_event WHERE asset_id = ?1 AND ex_date = ?2",
            )?
            .query_map(params![asset_id, d.date_com], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        if existing.iter().any(|e| {
            e.parse::<f64>()
                .is_ok_and(|x| (x - per).abs() <= per.abs() * 0.01 + 1e-9)
        }) {
            continue;
        }
        n += conn.execute(
            "INSERT OR IGNORE INTO dividend_event (id, asset_id, kind, ex_date, pay_date, amount_per_share, currency,
                                                   estimated, source, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 'fundamentus', ?8)",
            params![
                crate::ids::new_id(),
                asset_id,
                event_kind(snap.fii, &d.kind),
                d.date_com,
                d.pay_date,
                amount,
                currency,
                now
            ],
        )? as u32;
    }
    Ok(n)
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct FundamentusIncomeArgs {
    /// Só este ativo (padrão: ações e FIIs da B3 com operações ou acompanhados).
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    /// Ignora o cache e pede ao site de novo.
    #[serde(default)]
    #[ts(optional)]
    pub refresh: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FundamentusIncomeReport {
    #[ts(type = "number")]
    pub assets: u32,
    /// Proventos novos gravados como eventos anunciados.
    #[ts(type = "number")]
    pub events_added: u32,
    pub errors: Vec<String>,
}

/// Proventos anunciados de ações e FIIs da B3 pelo Fundamentus (com o cache de um dia).
pub fn sync_income(api: &crate::Api, a: &FundamentusIncomeArgs) -> Result<FundamentusIncomeReport> {
    let assets: Vec<(String, String, String)> = api.with_session(|s, _| {
        let c = s.db.conn();
        let ids = match &a.asset_id {
            Some(id) => vec![id.clone()],
            None => crate::market::sync::tracked_assets(c)?,
        };
        let mut out = vec![];
        for id in ids {
            let row: Option<(Option<String>, String, String)> = c
                .query_row(
                    "SELECT ticker, class, currency FROM asset WHERE id = ?1",
                    [&id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()?;
            if let Some((Some(t), class, cur)) = row
                && (class == "stock_br" || class == "fii")
                && clean_ticker(&t).is_ok()
            {
                out.push((id, t, cur));
            }
        }
        Ok(out)
    })?;
    let mut rep = FundamentusIncomeReport::default();
    // Fonte desligada: a rotina não insiste (o pedido de um ativo explica o motivo).
    let on =
        api.with_session(|s, _| Ok(crate::sources::enabled(s.db.conn(), SourceId::Fundamentus)))?;
    if !on && a.asset_id.is_none() {
        return Ok(rep);
    }
    for (id, ticker, currency) in assets {
        rep.assets += 1;
        match get_snapshot(
            api,
            &FundamentusArgs {
                ticker: ticker.clone(),
                refresh: a.refresh,
            },
        ) {
            Ok(snap) if snap.found => {
                match api.with_session(|s, _| store_dividends(s.db.conn(), &id, &currency, &snap)) {
                    Ok(n) => rep.events_added += n,
                    Err(e) => rep.errors.push(format!("{ticker}: {e}")),
                }
            }
            Ok(_) => {}
            Err(e) => rep.errors.push(format!("{ticker}: {e}")),
        }
    }
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DETAILS: &str = r##"<table><tr>
<td class="label w15"><span class="help tips" title="Código da ação">?</span><span class="txt">Papel</span></td>
<td class="data w35"><span class="txt">TEST4</span></td>
<td class="label destaque w2"><span class="help tips" title="Cotação de fechamento">?</span><span class="txt">Cotação</span></td>
<td class="data destaque w3"><span class="txt">15,56</span></td></tr><tr>
<td class="label"><span class="txt">Data últ cot</span></td><td class="data"><span class="txt">15/10/2026</span></td>
<td class="label"><span class="txt">Empresa</span></td><td class="data"><span class="txt">TESTE S.A. PN</span></td>
<td class="label"><span class="txt">Setor</span></td><td class="data"><span class="txt"><a href="x">Energia Elétrica</a></span></td>
<td class="label"><span class="txt">Subsetor</span></td><td class="data"><span class="txt">Energia</span></td></tr>
<tr><td class="label"><span class="txt">Dia</span></td><td class="data"><span class="oscil"><font color="#F75D59">-1,08%</font></span></td>
<td class="label"><span class="help tips" title="Preço dividido pelo lucro">?</span><span class="txt">P/L</span></td><td class="data"><span class="txt">9,61</span></td>
<td class="label"><span class="txt">2025</span></td><td class="data"><span class="oscil">58,33%</span></td>
<td class="label"><span class="txt">Div. Yield</span></td><td class="data"><span class="txt">7,2%</span></td>
<td class="label"><span class="txt">Valor de mercado</span></td><td class="data"><span class="txt">174.485.000.000</span></td>
<td class="label"><span class="txt">P/EBIT</span></td><td class="data"><span class="txt">-</span></td></tr>
<tr><td class="label"><span class="txt">Lucro Líquido</span></td><td class="data"><span class="txt">18.161.000.000</span></td>
<td class="label"><span class="txt">Lucro Líquido</span></td><td class="data"><span class="txt">5.244.000.000</span></td>
<td class="label"><span class="txt"></span></td><td class="data"><span class="txt"></span></td></tr></table>"##;

    const DIVIDENDS: &str = r#"<table id="resultado"><thead><tr><th>Data</th><th>Valor</th><th>Tipo</th><th>Data de Pagamento</th><th>Por quantas ações</th></tr></thead>
<tbody><tr class=""><td>31/08/2026</td><td>0,0242</td><td>JRS CAP PROPRIO</td><td>01/10/2026 </td><td>1</td></tr>
<tr class="par"><td>18/06/2026</td><td>1,38</td><td>DIVIDENDO</td><td>-</td><td>100</td></tr></tbody></table>
<table><tbody><tr><td>2026</td><td>0,327</td></tr></tbody></table>"#;

    const FII_DIVIDENDS: &str = r#"<table id="resultado"><tbody>
<tr><td>30/09/2026</td><td>Rendimento</td><td>15/10/2026</td><td>1,17</td></tr></tbody></table>"#;

    #[test]
    fn details_labels_numbers_and_repeated_periods() {
        let s = parse_details("TEST4", DETAILS);
        assert!(s.found && !s.fii);
        assert_eq!(s.price, Some(15.56));
        assert_eq!(s.price_date.as_deref(), Some("2026-10-15"));
        assert_eq!(s.name.as_deref(), Some("TESTE S.A. PN"));
        assert_eq!(s.sector.as_deref(), Some("Energia Elétrica"));
        assert_eq!(s.subsector.as_deref(), Some("Energia"));
        let get = |l: &str| s.indicators.iter().find(|x| x.label == l).cloned();
        let pl = get("P/L").unwrap();
        assert_eq!(pl.value, Some(9.61));
        assert_eq!(pl.hint.as_deref(), Some("Preço dividido pelo lucro"));
        assert_eq!(get("Div. Yield").unwrap().value, Some(7.2));
        assert_eq!(
            get("Valor de mercado").unwrap().value,
            Some(174_485_000_000.0)
        );
        assert_eq!(get("P/EBIT").unwrap().value, None);
        assert_eq!(
            get("Lucro Líquido (12m)").unwrap().value,
            Some(18_161_000_000.0)
        );
        assert_eq!(
            get("Lucro Líquido (3m)").unwrap().value,
            Some(5_244_000_000.0)
        );
        let osc: Vec<&str> = s.oscillations.iter().map(|x| x.label.as_str()).collect();
        assert_eq!(osc, ["Dia", "2025"]);
        assert_eq!(s.oscillations[0].value, Some(-1.08));
    }

    #[test]
    fn dividends_of_stocks_and_fiis_and_missing_papers() {
        let d = parse_dividends(DIVIDENDS, false);
        assert_eq!(d.len(), 2, "só a primeira tabela");
        assert_eq!(d[0].date_com, "2026-08-31");
        assert_eq!(d[0].value, 0.0242);
        assert_eq!(d[0].kind, "JRS CAP PROPRIO");
        assert_eq!(d[0].pay_date.as_deref(), Some("2026-10-01"));
        assert_eq!(d[1].per_shares, 100);
        assert_eq!(d[1].pay_date, None);
        let f = parse_dividends(FII_DIVIDENDS, true);
        assert_eq!(f[0].value, 1.17);
        assert_eq!(f[0].kind, "Rendimento");
        assert!(!parse_details("ZZZZ9", "<div>Nenhum papel encontrado</div>").found);
        assert_eq!(clean_ticker(" itsa4.sa ").unwrap(), "ITSA4");
        assert!(clean_ticker("../x").is_err());
    }

    #[test]
    fn dividends_become_announced_events_without_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST)
            .unwrap();
        let c = db.conn();
        c.execute(
            "INSERT INTO asset (id, ticker, name, class, currency, created_at, updated_at)
             VALUES ('a', 'TEST4', 'Teste', 'stock_br', 'BRL', 'x', 'x')",
            [],
        )
        .unwrap();
        // A mesma data-com já veio do Yahoo com o valor por ação igual.
        c.execute(
            "INSERT INTO dividend_event (id, asset_id, kind, ex_date, amount_per_share, currency, estimated, source, fetched_at)
             VALUES ('y', 'a', 'dividend', '2026-06-18', '0.0138', 'BRL', 0, 'yahoo', 'x')",
            [],
        )
        .unwrap();
        let mut snap = parse_details("TEST4", DETAILS);
        snap.dividends = parse_dividends(DIVIDENDS, false);
        // 1,38 por 100 ações = 0,0138 (já existe); JCP de 0,0242 entra.
        assert_eq!(store_dividends(c, "a", "BRL", &snap).unwrap(), 1);
        assert_eq!(
            store_dividends(c, "a", "BRL", &snap).unwrap(),
            0,
            "não repete"
        );
        let (kind, pay): (String, Option<String>) = c
            .query_row(
                "SELECT kind, pay_date FROM dividend_event WHERE source = 'fundamentus'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(kind, "jcp");
        assert_eq!(pay.as_deref(), Some("2026-10-01"));
        assert_eq!(event_kind(true, "Rendimento"), "fii_income");
        assert_eq!(event_kind(true, "Amortização"), "amortization");
    }

    #[test]
    fn cache_is_fresh_for_a_day_and_missing_for_a_week() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST)
            .unwrap();
        let c = db.conn();
        let mut s = parse_details("TEST4", DETAILS);
        store(c, &s).unwrap();
        let hit = cached(c, "TEST4").unwrap().unwrap();
        assert!(hit.from_cache);
        assert_eq!(hit.price, Some(15.56));
        s.fetched_at = (chrono::Utc::now() - chrono::Duration::hours(30)).to_rfc3339();
        store(c, &s).unwrap();
        assert!(cached(c, "TEST4").unwrap().is_none(), "velho");
        let mut missing = parse_details("ZZZZ9", "Nenhum papel encontrado");
        missing.fetched_at = (chrono::Utc::now() - chrono::Duration::days(2)).to_rfc3339();
        store(c, &missing).unwrap();
        assert!(!cached(c, "ZZZZ9").unwrap().unwrap().found);
    }

    /// Páginas salvas do site (`FUNDAMENTUS_DIR` com detalhes e proventos), para conferir o
    /// parser com o HTML de verdade: `cargo test -- --ignored real_pages`.
    #[test]
    #[ignore = "precisa de páginas salvas do site"]
    fn real_pages_from_env() {
        let dir = std::env::var("FUNDAMENTUS_DIR").unwrap();
        let read = |f: &str| decode_latin1(&std::fs::read(format!("{dir}/{f}")).unwrap());
        let s = parse_details("ITSA4", &read("itsa4.html"));
        println!("{s:#?}");
        assert!(s.found && s.price.is_some() && s.indicators.len() > 20);
        let f = parse_details("HGLG11", &read("hglg.html"));
        assert!(f.fii, "{f:#?}");
        assert!(!parse_dividends(&read("prov.html"), false).is_empty());
        let fd = parse_dividends(&read("fiiprov.html"), true);
        println!("{:?}", &fd[..3]);
        assert!(!fd.is_empty());
    }
}
