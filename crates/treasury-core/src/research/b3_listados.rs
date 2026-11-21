//! B3, API pública das empresas e fundos listados (a que o site da B3 usa nas páginas
//! "Empresas listadas" e "Fundos listados"): proventos em dinheiro (dividendos, JCP,
//! rendimentos e amortizações de FII) e eventos em ações (desdobramento, grupamento,
//! bonificação) de todo o universo, como fonte oficial para o DY, o Bazin e a conferência
//! dos proventos vindos do Yahoo e do Fundamentus.
//!
//! Pedidos (o argumento é um JSON em base64 no caminho):
//! - `GetListedSupplementCompany` (código do emissor, ex.: `PETR`): proventos em dinheiro dos
//!   últimos ~12 meses por ISIN, com data de pagamento, nome de pregão e eventos em ações.
//! - `GetListedCashDividends` (nome de pregão, páginas de 120): histórico completo por tipo de
//!   ação (ON, PN, PNA, UNT…), sem data de pagamento. Só para emissores dos ativos
//!   cadastrados, uma vez.
//! - `GetListedSupplementFunds` (código do fundo, ex.: `HGLG`): rendimentos e amortizações
//!   dos últimos ~12 meses e eventos em cotas.
//!
//! Uso consciente: um pedido por vez, com o ritmo por site do [`super::net`] (1,5 s na B3) e
//! nova tentativa em 429/503; coleta rolante pelo `research_fetch_log` (cadastrados a cada
//! ~20 h, o resto do universo em lotes pequenos, os mais antigos primeiro).

use std::collections::{BTreeMap, HashMap, HashSet};

use base64::Engine as _;
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde::Deserialize;

use super::net::Fetched;
use super::sync::{Runner, SyncItem, SyncSource, SyncStatus};
use crate::error::{Error, Result};
use crate::time::{fmt_date, now_utc};

/// Valor de `source` nas tabelas e no controle de coleta.
pub const SOURCE: &str = "b3_listados";
const BASE: &str = "https://sistemaswebb3-listados.b3.com.br";
/// Maior página aceita pela API (acima disso ela devolve uma lista vazia).
pub const PAGE_SIZE: u32 = 120;
/// Limite de páginas do histórico por emissor (as maiores têm ~3).
pub const MAX_PAGES: u32 = 8;
/// Ajuste técnico: quantos emissores e fundos do universo (fora os cadastrados) por dia.
pub const BATCH_SETTING: &str = "research.b3_listados_batch";
/// Lote padrão: 60 por dia, ou o necessário para o universo inteiro em ~7 dias.
pub const DEFAULT_BATCH: usize = 60;
pub const CYCLE_DAYS: usize = 7;

/// Lote do dia: o ajuste técnico, senão o maior entre 60 e o universo ÷ 7.
pub fn default_batch(universe: usize) -> usize {
    DEFAULT_BATCH.max(universe.div_ceil(CYCLE_DAYS))
}
/// Papel consultado com sucesso só volta depois de ~20 h.
pub const REFRESH_HOURS: i64 = 20;
/// Erro: nova tentativa depois de um dia; inexistente: depois de uma semana.
pub const ERROR_RETRY_HOURS: i64 = 24;
pub const NOT_FOUND_RETRY_HOURS: i64 = 24 * 7;
/// Erros de rede seguidos até a coleta da B3 parar (o resto fica para a próxima).
const MAX_CONSECUTIVE_ERRORS: u32 = 5;

/// Texto JSON com escape de tudo o que não é ASCII simples: o base64 resultante nunca tem
/// `/` nem `+` (que quebrariam o caminho da URL).
fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ' '..='~' if !matches!(c, '?' | '>' | '~') => out.push(c),
            _ => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{u:04x}"));
                }
            }
        }
    }
    out.push('"');
    out
}

fn encode(json: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
}

/// Complemento da companhia (proventos recentes, eventos em ações, nome de pregão).
pub fn company_url(code: &str) -> String {
    format!(
        "{BASE}/listedCompaniesProxy/CompanyCall/GetListedSupplementCompany/{}",
        encode(&format!(
            "{{\"issuingCompany\":{},\"language\":\"pt-br\"}}",
            json_str(code)
        ))
    )
}

/// Histórico de proventos em dinheiro da companhia (por nome de pregão).
pub fn cash_url(trading_name: &str, page: u32) -> String {
    format!(
        "{BASE}/listedCompaniesProxy/CompanyCall/GetListedCashDividends/{}",
        encode(&format!(
            "{{\"language\":\"pt-br\",\"pageNumber\":{page},\"pageSize\":{PAGE_SIZE},\"tradingName\":{}}}",
            json_str(trading_name.trim())
        ))
    )
}

/// Complemento do FII (rendimentos e amortizações recentes, eventos em cotas).
pub fn fund_url(code: &str) -> String {
    format!(
        "{BASE}/fundsProxy/fundsCall/GetListedSupplementFunds/{}",
        encode(&format!(
            "{{\"cnpj\":\"0\",\"identifierFund\":{},\"typeFund\":7}}",
            json_str(code)
        ))
    )
}

// ---------------------------------------------------------------------------------------
// Parse (só bytes → estruturas; testado com amostras no formato real)
// ---------------------------------------------------------------------------------------

/// Provento em dinheiro como a B3 informa.
#[derive(Debug, Clone, PartialEq)]
pub struct CashRow {
    /// ISIN (complemento) ou tipo da ação (`ON`, `PN`, `UNT`… no histórico).
    pub share: String,
    pub label: String,
    /// Último dia com direito (data-com).
    pub ex_date: NaiveDate,
    pub value: Decimal,
    /// Valor por quantas ações (cotação por lote antiga).
    pub per_shares: i64,
    pub approved_on: Option<NaiveDate>,
    pub pay_date: Option<NaiveDate>,
}

/// Evento em ações (desdobramento, grupamento, bonificação).
#[derive(Debug, Clone, PartialEq)]
pub struct StockRow {
    pub share: String,
    pub label: String,
    pub ex_date: NaiveDate,
    /// Como a B3 informa: percentual nos desdobramentos e bonificações (100 = 1 para 2),
    /// fator nos grupamentos (0,1 = 10 para 1).
    pub factor: Decimal,
    pub approved_on: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Supplement {
    pub code: String,
    pub trading_name: Option<String>,
    pub cash: Vec<CashRow>,
    pub stock: Vec<StockRow>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CashPage {
    pub total_pages: u32,
    pub rows: Vec<CashRow>,
}

/// Número com vírgula decimal e ponto de milhar (`"1.234,56"`, `"0,47156696000"`).
pub fn parse_num(s: &str) -> Option<Decimal> {
    let t = s.trim().replace('.', "").replace(',', ".");
    if t.is_empty() {
        return None;
    }
    t.parse::<Decimal>().ok().map(|d| d.normalize())
}

/// Data `dd/mm/aaaa` (o `31/12/9999` da B3 quer dizer "sem data").
pub fn parse_br_date(s: &str) -> Option<NaiveDate> {
    let d = NaiveDate::parse_from_str(s.trim(), "%d/%m/%Y").ok()?;
    (1901..9000)
        .contains(&chrono::Datelike::year(&d))
        .then_some(d)
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawCash {
    isin_code: Option<String>,
    asset_issued: Option<String>,
    rate: Option<String>,
    label: Option<String>,
    last_date_prior: Option<String>,
    approved_on: Option<String>,
    payment_date: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawStock {
    isin_code: Option<String>,
    asset_issued: Option<String>,
    factor: Option<String>,
    label: Option<String>,
    last_date_prior: Option<String>,
    approved_on: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawSupplement {
    code: Option<String>,
    trading_name: Option<String>,
    fund: Option<String>,
    cash_dividends: Option<Vec<RawCash>>,
    stock_dividends: Option<Vec<RawStock>>,
}

fn opt_date(s: &Option<String>) -> Option<NaiveDate> {
    s.as_deref().and_then(parse_br_date)
}

fn supplement(raw: RawSupplement) -> Supplement {
    let cash = raw
        .cash_dividends
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| {
            Some(CashRow {
                share: c.isin_code.or(c.asset_issued)?.trim().to_uppercase(),
                label: c.label.unwrap_or_default().trim().to_string(),
                ex_date: opt_date(&c.last_date_prior)?,
                value: c.rate.as_deref().and_then(parse_num)?,
                per_shares: 1,
                approved_on: opt_date(&c.approved_on),
                pay_date: opt_date(&c.payment_date),
            })
        })
        .filter(|c| c.value > Decimal::ZERO)
        .collect();
    let stock = raw
        .stock_dividends
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| {
            Some(StockRow {
                share: s.isin_code.or(s.asset_issued)?.trim().to_uppercase(),
                label: s.label.unwrap_or_default().trim().to_string(),
                ex_date: opt_date(&s.last_date_prior)?,
                factor: s.factor.as_deref().and_then(parse_num)?,
                approved_on: opt_date(&s.approved_on),
            })
        })
        .collect();
    Supplement {
        code: raw.code.unwrap_or_default().trim().to_uppercase(),
        trading_name: raw
            .trading_name
            .or(raw.fund)
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty()),
        cash,
        stock,
    }
}

/// Complemento da companhia. `None`: a B3 não conhece o código (lista vazia).
pub fn parse_company(bytes: &[u8]) -> std::result::Result<Option<Supplement>, String> {
    let v: Vec<RawSupplement> = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-b3-companhia",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    Ok(v.into_iter()
        .next()
        .map(supplement)
        .filter(|s| !s.code.is_empty()))
}

/// Complemento do FII. `None`: fundo desconhecido.
pub fn parse_fund(bytes: &[u8]) -> std::result::Result<Option<Supplement>, String> {
    let t = String::from_utf8_lossy(bytes);
    if t.trim().is_empty() || t.trim() == "null" {
        return Ok(None);
    }
    let v: RawSupplement = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-b3-fundo",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    Ok(Some(supplement(v)).filter(|s| !s.code.is_empty()))
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawPageInfo {
    total_pages: Option<u32>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawHist {
    type_stock: Option<String>,
    date_approval: Option<String>,
    value_cash: Option<String>,
    corporate_action: Option<String>,
    last_date_prior_ex: Option<String>,
    quoted_per_shares: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawCashPage {
    page: Option<RawPageInfo>,
    results: Option<Vec<RawHist>>,
}

/// Uma página do histórico de proventos em dinheiro.
pub fn parse_cash_page(bytes: &[u8]) -> std::result::Result<CashPage, String> {
    let v: RawCashPage = serde_json::from_slice(bytes).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-b3-historico",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let rows = v
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(|h| {
            let per = h
                .quoted_per_shares
                .as_deref()
                .and_then(parse_num)
                .and_then(|d| rust_decimal::prelude::ToPrimitive::to_i64(&d.trunc()))
                .filter(|n| *n > 0)
                .unwrap_or(1);
            Some(CashRow {
                share: h.type_stock?.trim().to_uppercase(),
                label: h.corporate_action.unwrap_or_default().trim().to_string(),
                ex_date: opt_date(&h.last_date_prior_ex)?,
                value: h.value_cash.as_deref().and_then(parse_num)?,
                per_shares: per,
                approved_on: opt_date(&h.date_approval),
                pay_date: None,
            })
        })
        .filter(|c| c.value > Decimal::ZERO)
        .collect();
    Ok(CashPage {
        total_pages: v.page.and_then(|p| p.total_pages).unwrap_or(0),
        rows,
    })
}

// ---------------------------------------------------------------------------------------
// Tipos e códigos de negociação
// ---------------------------------------------------------------------------------------

/// Tipo do provento (`research_dividend.kind`) pelo texto da B3.
pub fn cash_kind(label: &str, fund: bool) -> &'static str {
    let t = crate::import::text::fold(label);
    if t.contains("amort") || t.contains("restit") || t.contains("rest cap") || t.contains("reduc")
    {
        "amortization"
    } else if t.contains("jrs") || t.contains("juros") || t.contains("jcp") {
        "jcp"
    } else if t.contains("divid") {
        "dividend"
    } else if fund && t.contains("rend") {
        "fii_income"
    } else {
        "other"
    }
}

/// Tipo do evento em ações (`research_stock_event.kind`).
pub fn stock_kind(label: &str) -> &'static str {
    let t = crate::import::text::fold(label);
    if t.contains("desdob") {
        "split"
    } else if t.contains("grupam") {
        "reverse_split"
    } else if t.contains("bonif") {
        "bonus"
    } else {
        "other"
    }
}

/// Quantas ações novas por ação antiga: desdobramento e bonificação vêm em percentual
/// (100 = +100%, 1 para 2); grupamento, como fator (0,1 = 10 para 1).
pub fn multiplier(kind: &str, factor: Decimal) -> Option<Decimal> {
    match kind {
        "split" | "bonus" if factor > Decimal::ZERO => {
            Some(Decimal::ONE + factor / Decimal::ONE_HUNDRED)
        }
        "reverse_split" if factor > Decimal::ZERO && factor < Decimal::ONE => Some(factor),
        _ => None,
    }
}

/// Classe da ação pelo ISIN (`BRPETRACNPR6` → PN, `BRTAEECDAM10` → UNT, `BRHGLGCTF004` →
/// CTF) ou pelo tipo do histórico (`ON`, `PNA`, `UNT`). Direitos, recibos e bônus: `None`.
pub fn share_class(share: &str) -> Option<&'static str> {
    let s = share.trim().to_uppercase();
    if s.len() == 12 && s.starts_with("BR") {
        // BR + emissor (4) + espécie: ACNOR (ON), ACNPR (PN), ACNPA… (PNA…), CDAM (unit),
        // CTF (cota de fundo) + controle.
        if &s[6..9] == "CTF" {
            return Some("CTF");
        }
        if &s[6..9] == "CDA" {
            return Some("UNT");
        }
        return match &s[6..11] {
            "ACNOR" => Some("ON"),
            "ACNPR" => Some("PN"),
            "ACNPA" => Some("PNA"),
            "ACNPB" => Some("PNB"),
            "ACNPC" => Some("PNC"),
            "ACNPD" => Some("PND"),
            _ => None,
        };
    }
    match s.as_str() {
        "ON" => Some("ON"),
        "PN" => Some("PN"),
        "PNA" => Some("PNA"),
        "PNB" => Some("PNB"),
        "PNC" => Some("PNC"),
        "PND" => Some("PND"),
        "UNT" | "UNIT" => Some("UNT"),
        "CI" | "CTF" => Some("CTF"),
        _ => None,
    }
}

fn suffix(class: &str) -> &'static str {
    match class {
        "ON" => "3",
        "PN" => "4",
        "PNA" => "5",
        "PNB" => "6",
        "PNC" => "7",
        "PND" => "8",
        _ => "11",
    }
}

/// Código de negociação conhecido (universo baixado, cotações ou ativo cadastrado).
#[derive(Debug, Clone, PartialEq)]
pub struct Known {
    pub ticker: String,
    /// ON, PN, UNIT, CTF (do FCA ou do informe de FII); `None` se só a cotação é conhecida.
    pub share_kind: Option<String>,
}

/// Código do emissor no ticker (`PETR4` → `PETR`, `TAEE11` → `TAEE`, `B3SA3` → `B3SA`).
pub fn code_of(ticker: &str) -> Option<String> {
    let t = ticker.trim().to_uppercase();
    let (code, rest) = (t.get(..4)?, t.get(4..)?);
    (code.chars().all(|c| c.is_ascii_alphanumeric())
        && code.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && !rest.is_empty()
        && rest.len() <= 2
        && rest.chars().all(|c| c.is_ascii_digit()))
    .then(|| code.to_string())
}

/// Ticker de um provento da B3: o sufixo padrão do tipo (ON 3, PN 4, PNA 5, PNB 6, UNT e
/// cota 11) se o papel existe; senão, o único papel conhecido do mesmo tipo (`share_kind`).
pub fn ticker_for(code: &str, share: &str, known: &[Known]) -> Option<String> {
    let class = share_class(share)?;
    let code = code.trim().to_uppercase();
    let candidate = format!("{code}{}", suffix(class));
    if known.iter().any(|k| k.ticker == candidate) {
        return Some(candidate);
    }
    let kind = match class {
        "ON" => "ON",
        "UNT" => "UNIT",
        "CTF" => "CTF",
        _ => return None,
    };
    let mut same = known
        .iter()
        .filter(|k| k.ticker.starts_with(&code) && k.share_kind.as_deref() == Some(kind));
    match (same.next(), same.next()) {
        (Some(k), None) => Some(k.ticker.clone()),
        _ => None,
    }
}

/// Provento pronto para gravar.
#[derive(Debug, Clone, PartialEq)]
pub struct DivRec {
    pub ticker: String,
    pub kind: &'static str,
    pub ex_date: NaiveDate,
    pub value: Decimal,
    pub per_shares: i64,
    pub approved_on: Option<NaiveDate>,
    pub pay_date: Option<NaiveDate>,
    pub label: String,
}

/// Evento em ações pronto para gravar.
#[derive(Debug, Clone, PartialEq)]
pub struct EventRec {
    pub ticker: String,
    pub kind: &'static str,
    pub ex_date: NaiveDate,
    pub factor: Decimal,
    pub approved_on: Option<NaiveDate>,
    pub label: String,
}

/// Proventos com o ticker de cada linha (as de papéis desconhecidos ficam de fora).
pub fn map_cash(code: &str, rows: &[CashRow], known: &[Known], fund: bool) -> Vec<DivRec> {
    // Fundo: a B3 às vezes repete a mesma linha (data-com, valor, aprovação e pagamento).
    let mut seen: Vec<&CashRow> = vec![];
    rows.iter()
        .filter(|r| {
            if !fund {
                return true;
            }
            let dup = seen.contains(r);
            seen.push(r);
            !dup
        })
        .filter_map(|r| {
            Some(DivRec {
                ticker: ticker_for(code, &r.share, known)?,
                kind: cash_kind(&r.label, fund),
                ex_date: r.ex_date,
                value: r.value,
                per_shares: r.per_shares,
                approved_on: r.approved_on,
                pay_date: r.pay_date,
                label: r.label.clone(),
            })
        })
        .collect()
}

pub fn map_stock(code: &str, rows: &[StockRow], known: &[Known]) -> Vec<EventRec> {
    rows.iter()
        .filter_map(|r| {
            Some(EventRec {
                ticker: ticker_for(code, &r.share, known)?,
                kind: stock_kind(&r.label),
                ex_date: r.ex_date,
                factor: r.factor,
                approved_on: r.approved_on,
                label: r.label.clone(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Planejamento da coleta rolante
// ---------------------------------------------------------------------------------------

/// Emissor (companhia) ou fundo a consultar.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    pub code: String,
    pub fund: bool,
    /// Algum ativo cadastrado é deste emissor/fundo.
    pub registered: bool,
}

impl Target {
    /// Chave no `research_fetch_log` (`cia:PETR`, `fii:HGLG`).
    pub fn key(&self) -> String {
        format!("{}:{}", if self.fund { "fii" } else { "cia" }, self.code)
    }
    /// Chave do histórico completo de proventos (`hist:PETR`).
    pub fn history_key(&self) -> String {
        format!("hist:{}", self.code)
    }
}

/// Última consulta de uma chave.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub at: DateTime<Utc>,
    pub status: String,
}

fn min_age_hours(status: &str) -> i64 {
    match status {
        "not_found" => NOT_FOUND_RETRY_HOURS,
        "error" | "empty" => ERROR_RETRY_HOURS,
        _ => REFRESH_HOURS,
    }
}

fn due(e: Option<&LogEntry>, now: DateTime<Utc>) -> bool {
    e.is_none_or(|e| (now - e.at).num_hours() >= min_age_hours(&e.status))
}

/// O que consultar nesta coleta: (1) os emissores e fundos dos ativos cadastrados com a
/// última consulta vencida (~20 h; erro, um dia; inexistente, uma semana); (2) até `batch`
/// outros do universo por dia, os nunca consultados e depois os mais antigos primeiro. O lote
/// conta as consultas do universo das últimas 20 h: várias coletas no mesmo dia não passam
/// dele. `force` refaz os cadastrados.
pub fn plan_batch(
    targets: &[Target],
    log: &HashMap<String, LogEntry>,
    now: DateTime<Utc>,
    batch: usize,
    force: bool,
) -> Vec<Target> {
    let mut out: Vec<Target> = targets
        .iter()
        .filter(|t| t.registered && (force || due(log.get(&t.key()), now)))
        .cloned()
        .collect();
    out.sort();
    let others: Vec<&Target> = targets.iter().filter(|t| !t.registered).collect();
    let recent = others
        .iter()
        .filter(|t| {
            log.get(&t.key())
                .is_some_and(|e| (now - e.at).num_hours() < REFRESH_HOURS)
        })
        .count();
    let budget = batch.saturating_sub(recent);
    let mut cands: Vec<(Option<DateTime<Utc>>, &Target)> = others
        .into_iter()
        .filter(|t| due(log.get(&t.key()), now))
        .map(|t| (log.get(&t.key()).map(|e| e.at), t))
        .collect();
    cands.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    out.extend(cands.into_iter().take(budget).map(|(_, t)| t.clone()));
    out
}

/// O histórico completo (só emissores cadastrados) é baixado uma vez; erro, de novo depois
/// de um dia.
pub fn history_due(t: &Target, log: &HashMap<String, LogEntry>, now: DateTime<Utc>) -> bool {
    !t.fund
        && t.registered
        && match log.get(&t.history_key()) {
            None => true,
            Some(e) => e.status == "error" && (now - e.at).num_hours() >= ERROR_RETRY_HOURS,
        }
}

/// Códigos de negociação conhecidos por código de emissor.
pub type KnownByCode = HashMap<String, Vec<Known>>;

/// Emissores e fundos com papéis negociados (cotação no COTAHIST) e os dos ativos
/// cadastrados, com os códigos de negociação conhecidos de cada um.
pub fn targets(conn: &Connection) -> Result<(Vec<Target>, KnownByCode)> {
    let mut known: KnownByCode = HashMap::new();
    let mut push = |ticker: &str, kind: Option<String>| {
        if let Some(c) = code_of(ticker) {
            let v = known.entry(c).or_default();
            if !v.iter().any(|k| k.ticker == ticker) {
                v.push(Known {
                    ticker: ticker.to_string(),
                    share_kind: kind,
                });
            }
        }
    };
    let secs: Vec<(String, String, Option<String>)> = conn
        .prepare(
            "SELECT ticker, class, share_kind FROM research_security
             WHERE active = 1 AND class IN ('stock_br', 'fii') ORDER BY ticker",
        )?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let traded: HashSet<String> = conn
        .prepare("SELECT DISTINCT ticker FROM research_quote")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut set: BTreeMap<(String, bool), bool> = BTreeMap::new();
    for (t, class, kind) in &secs {
        push(t, kind.clone());
        if traded.contains(t)
            && let Some(c) = code_of(t)
        {
            set.entry((c, class == "fii")).or_insert(false);
        }
    }
    for a in crate::assets::list(conn, &Default::default())? {
        let fund = match a.class {
            crate::assets::AssetClass::StockBr => false,
            crate::assets::AssetClass::Fii => true,
            _ => continue,
        };
        let Some(t) = a.ticker.as_deref().map(|t| t.trim().to_uppercase()) else {
            continue;
        };
        let Some(c) = code_of(&t) else { continue };
        push(&t, None);
        set.insert((c, fund), true);
    }
    // Cotações de papéis dos emissores escolhidos (ex.: a unit que o FCA não lista).
    for t in &traded {
        if let Some(c) = code_of(t)
            && (set.contains_key(&(c.clone(), false)) || set.contains_key(&(c, true)))
        {
            push(t, None);
        }
    }
    let targets = set
        .into_iter()
        .map(|((code, fund), registered)| Target {
            code,
            fund,
            registered,
        })
        .collect();
    Ok((targets, known))
}

pub fn load_log(conn: &Connection) -> Result<HashMap<String, LogEntry>> {
    let rows: Vec<(String, String, String)> = conn
        .prepare("SELECT key, fetched_at, status FROM research_fetch_log WHERE source = ?1")?
        .query_map([SOURCE], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(k, at, status)| {
            let at = DateTime::parse_from_rfc3339(&at).ok()?.with_timezone(&Utc);
            Some((k, LogEntry { at, status }))
        })
        .collect())
}

// ---------------------------------------------------------------------------------------
// Gravação
// ---------------------------------------------------------------------------------------

pub fn log_fetch(
    conn: &Connection,
    key: &str,
    status: &str,
    detail: Option<&str>,
    now: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO research_fetch_log (source, key, fetched_at, status, detail) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (source, key) DO UPDATE SET fetched_at = excluded.fetched_at, status = excluded.status,
             detail = excluded.detail",
        params![SOURCE, key, now, status, detail],
    )?;
    Ok(())
}

/// Grava proventos. `full`: a lista é o histórico completo dos papéis (substitui tudo deles);
/// senão, substitui só a partir da data-com mais antiga de cada papel na lista.
pub fn store_dividends(conn: &Connection, rows: &[DivRec], full: bool, now: &str) -> Result<u64> {
    let mut from: BTreeMap<&str, NaiveDate> = BTreeMap::new();
    for r in rows {
        let e = from.entry(r.ticker.as_str()).or_insert(r.ex_date);
        *e = (*e).min(r.ex_date);
    }
    for (t, d) in &from {
        if full {
            conn.execute(
                "DELETE FROM research_dividend WHERE ticker = ?1 AND source = ?2",
                params![t, SOURCE],
            )?;
        } else {
            conn.execute(
                "DELETE FROM research_dividend WHERE ticker = ?1 AND source = ?2 AND ex_date >= ?3",
                params![t, SOURCE, fmt_date(*d)],
            )?;
        }
    }
    let mut stmt = conn.prepare_cached(
        "INSERT OR REPLACE INTO research_dividend (ticker, kind, ex_date, value, per_shares, approved_on, pay_date,
                                                   label, seq, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?;
    // Parcelas iguais (mesmo papel, tipo, data e valor) são linhas distintas, na ordem de
    // pagamento.
    let mut sorted: Vec<&DivRec> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        (&a.ticker, a.kind, a.ex_date, a.value, a.pay_date)
            .cmp(&(&b.ticker, b.kind, b.ex_date, b.value, b.pay_date))
    });
    let mut seq: HashMap<(String, &str, NaiveDate, String), i64> = HashMap::new();
    let mut n = 0;
    for r in sorted {
        let v = super::dstr(r.value);
        let s = seq
            .entry((r.ticker.clone(), r.kind, r.ex_date, v.clone()))
            .or_insert(0);
        n += stmt.execute(params![
            r.ticker,
            r.kind,
            fmt_date(r.ex_date),
            v,
            r.per_shares,
            r.approved_on.map(fmt_date),
            r.pay_date.map(fmt_date),
            r.label,
            *s,
            SOURCE,
            now
        ])? as u64;
        *s += 1;
    }
    Ok(n)
}

pub fn store_events(conn: &Connection, rows: &[EventRec], now: &str) -> Result<u64> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO research_stock_event (ticker, kind, ex_date, factor, approved_on, label, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (ticker, kind, ex_date, source) DO UPDATE SET factor = excluded.factor,
             approved_on = excluded.approved_on, label = excluded.label, fetched_at = excluded.fetched_at",
    )?;
    let mut n = 0;
    for r in rows {
        n += stmt.execute(params![
            r.ticker,
            r.kind,
            fmt_date(r.ex_date),
            super::dstr(r.factor),
            r.approved_on.map(fmt_date),
            r.label,
            SOURCE,
            now
        ])? as u64;
    }
    Ok(n)
}

// ---------------------------------------------------------------------------------------
// Coleta
// ---------------------------------------------------------------------------------------

/// Um pedido: corpo, inexistente (`None`) ou erro.
/// Resposta de um pedido à B3.
enum Got {
    Body(Vec<u8>),
    NotFound,
    /// 200 sem conteúdo duas vezes seguidas: a B3 não tem dado para o código (fundo
    /// encerrado ou incorporado, papel fora da API). Não é erro da coleta.
    Empty,
}

fn get(r: &mut Runner<'_>, url: &str) -> std::result::Result<Got, String> {
    // A API às vezes responde 200 com o corpo vazio: uma nova tentativa (no ritmo do site).
    for _ in 0..2 {
        let res = r.fetcher.get(url, None, "");
        let delay = r.fetcher.polite_delay(url);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
        match res? {
            Fetched::Body { bytes, .. } if bytes.iter().all(u8::is_ascii_whitespace) => {}
            Fetched::Body { bytes, .. } => {
                r.rep.bytes += bytes.len() as u64;
                return Ok(Got::Body(bytes));
            }
            Fetched::NotFound => return Ok(Got::NotFound),
            Fetched::NotModified => return Err(crate::i18n::t("nucleo-pesquisa-erro-304")),
        }
    }
    Ok(Got::Empty)
}

/// Histórico completo de uma companhia (todas as páginas, até [`MAX_PAGES`]).
fn history(
    r: &mut Runner<'_>,
    trading_name: &str,
    requests: &mut u32,
) -> std::result::Result<Vec<CashRow>, String> {
    let mut rows = vec![];
    let mut page = 1;
    loop {
        *requests += 1;
        let Got::Body(b) = get(r, &cash_url(trading_name, page))? else {
            break;
        };
        let p = parse_cash_page(&b)?;
        rows.extend(p.rows);
        if page >= p.total_pages.min(MAX_PAGES) {
            break;
        }
        page += 1;
    }
    Ok(rows)
}

/// Contadores da coleta (para o relatório e o teste).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outcome {
    pub planned: usize,
    pub requests: u32,
    pub ok: u32,
    pub not_found: u32,
    pub errors: u32,
    pub rows: u64,
    pub stopped: bool,
}

/// Coleta rolante dos proventos e eventos oficiais da B3, dentro da coleta da análise de
/// ativos. Rede fora do banco; cada emissor gravado numa transação.
pub(crate) fn sync(r: &mut Runner<'_>, today: NaiveDate) -> Result<Outcome> {
    let force = r.plan.force;
    let now = Utc::now();
    let (targets, known, log, batch) = r.with(|c| {
        let (t, k) = targets(c)?;
        let universe = t.iter().filter(|x| !x.registered).count();
        let batch: usize = crate::settings::get::<u32>(c, BATCH_SETTING)?
            .map(|b| b as usize)
            .unwrap_or_else(|| default_batch(universe));
        Ok((t, k, load_log(c)?, batch))
    })?;
    let planned = plan_batch(&targets, &log, now, batch, force);
    let mut out = Outcome {
        planned: planned.len(),
        ..Default::default()
    };
    let mut consecutive = 0u32;
    let mut last_error: Option<String> = None;
    let mut errors: Vec<SyncItem> = vec![];
    let none: Vec<Known> = vec![];
    for t in &planned {
        if consecutive >= MAX_CONSECUTIVE_ERRORS {
            out.stopped = true;
            break;
        }
        let url = if t.fund {
            fund_url(&t.code)
        } else {
            company_url(&t.code)
        };
        out.requests += 1;
        let parsed = get(r, &url).and_then(|b| match b {
            Got::Empty => Ok(Err(())),
            Got::NotFound => Ok(Ok(None)),
            Got::Body(b) if t.fund => parse_fund(&b).map(Ok),
            Got::Body(b) => parse_company(&b).map(Ok),
        });
        let known = known.get(&t.code).unwrap_or(&none);
        let stamp = now_utc();
        match parsed {
            Err(e) => {
                consecutive += 1;
                out.errors += 1;
                last_error = Some(e.clone());
                r.with(|c| log_fetch(c, &t.key(), "error", Some(&e), &stamp))?;
                errors.push(SyncItem {
                    source: SyncSource::B3Listados,
                    label: format!("B3 proventos {}", t.code),
                    url: Some(url),
                    status: SyncStatus::Error,
                    bytes: 0,
                    rows: 0,
                    error: Some(e),
                });
            }
            // Sem conteúdo: sem dado para o código; volta no dia seguinte, sem contar como erro.
            Ok(Err(())) => {
                consecutive = 0;
                out.not_found += 1;
                r.with(|c| {
                    log_fetch(
                        c,
                        &t.key(),
                        "empty",
                        Some(&crate::i18n::t("nucleo-pesquisa-erro-b3-vazia")),
                        &stamp,
                    )
                })?;
            }
            Ok(Ok(None)) => {
                consecutive = 0;
                out.not_found += 1;
                r.with(|c| log_fetch(c, &t.key(), "not_found", None, &stamp))?;
            }
            Ok(Ok(Some(sup))) => {
                consecutive = 0;
                // Histórico completo (uma vez por emissor cadastrado).
                let mut hist: Option<std::result::Result<Vec<CashRow>, String>> = None;
                if history_due(t, &log, now)
                    && let Some(name) = sup.trading_name.as_deref()
                {
                    hist = Some(history(r, name, &mut out.requests));
                }
                let cash = map_cash(&t.code, &sup.cash, known, t.fund);
                let events = map_stock(&t.code, &sup.stock, known);
                let hist_rows = match &hist {
                    Some(Ok(rows)) => Some(map_cash(&t.code, rows, known, false)),
                    _ => None,
                };
                let res = r.with(|c| {
                    let tx = c.transaction()?;
                    let mut n = 0;
                    if let Some(h) = &hist_rows {
                        n += store_dividends(&tx, h, true, &stamp)?;
                        log_fetch(
                            &tx,
                            &t.history_key(),
                            "ok",
                            Some(&crate::i18n::t_args(
                                "nucleo-pesquisa-b3-log-proventos",
                                &crate::i18n::args(&[("n", (h.len() as i64).into())]),
                            )),
                            &stamp,
                        )?;
                    } else if let Some(Err(e)) = &hist {
                        log_fetch(&tx, &t.history_key(), "error", Some(e), &stamp)?;
                    }
                    n += store_dividends(&tx, &cash, false, &stamp)?;
                    n += store_events(&tx, &events, &stamp)?;
                    log_fetch(
                        &tx,
                        &t.key(),
                        "ok",
                        Some(&crate::i18n::t_args(
                            "nucleo-pesquisa-b3-log-proventos-eventos",
                            &crate::i18n::args(&[
                                ("proventos", (cash.len() as i64).into()),
                                ("eventos", (events.len() as i64).into()),
                            ]),
                        )),
                        &stamp,
                    )?;
                    tx.commit()?;
                    Ok(n)
                });
                match res {
                    Ok(n) => {
                        out.ok += 1;
                        out.rows += n;
                    }
                    Err(e @ (Error::Locked | Error::NoVault)) => return Err(e),
                    Err(e) => {
                        out.errors += 1;
                        last_error = Some(e.to_string());
                    }
                }
                if let Some(Err(e)) = hist {
                    out.errors += 1;
                    errors.push(SyncItem {
                        source: SyncSource::B3Listados,
                        label: crate::i18n::t_args(
                            "nucleo-pesquisa-b3-passo-historico",
                            &crate::i18n::args(&[("codigo", t.code.clone().into())]),
                        ),
                        url: None,
                        status: SyncStatus::Error,
                        bytes: 0,
                        rows: 0,
                        error: Some(e),
                    });
                }
            }
        }
    }
    let label = crate::i18n::t_args(
        "nucleo-pesquisa-b3-passo-resumo",
        &crate::i18n::args(&[
            ("feitos", i64::from(out.ok + out.not_found).into()),
            ("total", (targets.len() as i64).into()),
            ("pedidos", i64::from(out.requests).into()),
        ]),
    );
    let status = if out.planned == 0 {
        SyncStatus::Unchanged
    } else if out.ok > 0 {
        SyncStatus::Updated
    } else if out.errors > 0 {
        SyncStatus::Error
    } else {
        SyncStatus::NotFound
    };
    let mut summary = SyncItem {
        source: SyncSource::B3Listados,
        label,
        url: None,
        status,
        bytes: 0,
        rows: out.rows,
        error: None,
    };
    if out.stopped {
        summary.error = Some(crate::i18n::t_args(
            "nucleo-pesquisa-b3-parou",
            &crate::i18n::args(&[("vezes", (MAX_CONSECUTIVE_ERRORS as i64).into())]),
        ));
    } else if out.planned == 0 {
        summary.error = Some(crate::i18n::t("nucleo-pesquisa-b3-nada-a-atualizar"));
    }
    if out.errors > 0 {
        r.rep.ok = false;
    }
    r.rep.items.push(summary);
    r.rep.items.extend(errors);
    // Saúde da fonte: falhou tudo = um dia de falha; algum sucesso zera.
    if out.requests > 0 {
        let res: std::result::Result<(), String> = if out.ok > 0 || out.errors == 0 {
            Ok(())
        } else {
            Err(last_error.unwrap_or_else(|| crate::i18n::t("nucleo-pesquisa-erro-b3-api")))
        };
        r.with(|c| {
            crate::sources::record_result(c, crate::sources::SourceId::B3, &res, today)?;
            Ok(())
        })?;
    }
    Ok(out)
}

#[cfg(test)]
pub(crate) mod fixtures {
    //! Samples in the real format of the responses (trimmed).

    pub const COMPANY_PETR: &str = r#"[{"stockCapital":"205.431.960.490,52","tradingName":"PETROBRAS   ","code":"PETR","codeCVM":"9512","segment":"N2",
      "cashDividends":[
        {"assetIssued":"BRPETRACNOR9","paymentDate":"21/12/2026","rate":"0,47156696000","relatedTo":"Anual/2026","approvedOn":"06/08/2026","isinCode":"BRPETRACNOR9","label":"DIVIDENDO","lastDatePrior":"21/08/2026","remarks":""},
        {"assetIssued":"BRPETRACNOR9","paymentDate":"23/11/2026","rate":"0,67407131000","relatedTo":"Anual/2026","approvedOn":"06/08/2026","isinCode":"BRPETRACNOR9","label":"JRS CAP PROPRIO","lastDatePrior":"21/08/2026","remarks":""},
        {"assetIssued":"BRPETRACNPR6","paymentDate":"21/12/2026","rate":"0,47156696000","relatedTo":"Anual/2026","approvedOn":"06/08/2026","isinCode":"BRPETRACNPR6","label":"DIVIDENDO","lastDatePrior":"21/08/2026","remarks":""},
        {"assetIssued":"BRPETRACNOR9","paymentDate":"20/08/2026","rate":"0,35048636000","relatedTo":"Anual/2026","approvedOn":"11/05/2026","isinCode":"BRPETRACNOR9","label":"JRS CAP PROPRIO","lastDatePrior":"01/06/2026","remarks":""},
        {"assetIssued":"BRPETRACNOR9","paymentDate":"21/09/2026","rate":"0,35048636000","relatedTo":"Anual/2026","approvedOn":"11/05/2026","isinCode":"BRPETRACNOR9","label":"JRS CAP PROPRIO","lastDatePrior":"01/06/2026","remarks":""},
        {"assetIssued":"BRPETRACNOR9","paymentDate":"20/05/2026","rate":"0,01649003000","relatedTo":"Anual/2025","approvedOn":"16/04/2026","isinCode":"BRPETRACNOR9","label":"RENDIMENTO","lastDatePrior":"22/04/2026","remarks":""},
        {"assetIssued":"BRPETRDBS036","paymentDate":"20/05/2026","rate":"1,00000000000","relatedTo":"","approvedOn":"16/04/2026","isinCode":"BRPETRDBS036","label":"JUROS","lastDatePrior":"22/04/2026","remarks":""}],
      "stockDividends":[
        {"assetIssued":"BRPETRACNOR9","factor":"100,00000000000","approvedOn":"25/04/2008","isinCode":"BRPETRACNOR9","label":"DESDOBRAMENTO","lastDatePrior":"25/04/2008","remarks":""}],
      "subscriptions":[]}]"#;

    pub const FUND_HGLG: &str = r#"{"quotedPer":"25/06/2010","form":"Escritural","fund":"FII HGLG PAX","tradingLot":"1","quantity":"45.601.745","code":"HGLG","codeCVM":null,"segment":"BOLSA",
      "cashDividends":[
        {"assetIssued":"BRHGLGCTF004","paymentDate":"15/10/2026","rate":"1,17000000000","relatedTo":"SETEMBRO/2026","approvedOn":"30/09/2026","isinCode":"BRHGLGCTF004","label":"RENDIMENTO","lastDatePrior":"30/09/2026","remarks":""},
        {"assetIssued":"BRHGLGR26M17","paymentDate":"15/10/2026","rate":"1,09200000000","relatedTo":"SETEMBRO/2026","approvedOn":"30/09/2026","isinCode":"BRHGLGR26M17","label":"RENDIMENTO","lastDatePrior":"30/09/2026","remarks":""},
        {"assetIssued":"BRHGLGCTF004","paymentDate":"15/09/2026","rate":"1,17000000000","relatedTo":"AGOSTO/2026","approvedOn":"31/08/2026","isinCode":"BRHGLGCTF004","label":"RENDIMENTO","lastDatePrior":"31/08/2026","remarks":""},
        {"assetIssued":"BRHGLGCTF004","paymentDate":"15/09/2026","rate":"2,00000000000","relatedTo":"AGOSTO/2026","approvedOn":"31/08/2026","isinCode":"BRHGLGCTF004","label":"AMORTIZACAO","lastDatePrior":"31/08/2026","remarks":""}],
      "stockDividends":[
        {"assetIssued":"BRHGLGCTF004","factor":"900,00000000000","approvedOn":"17/04/2018","isinCode":"BRHGLGCTF004","label":"DESDOBRAMENTO","lastDatePrior":"17/04/2018","remarks":""}],
      "subscriptions":[]}"#;

    pub fn cash_page(total_pages: u32, rows: &[(&str, &str, &str, &str)]) -> String {
        let rows: Vec<String> = rows
            .iter()
            .map(|(t, label, date, v)| {
                format!(
                    r#"{{"typeStock":"{t}","dateApproval":"{date}","valueCash":"{v}","ratio":"1","corporateAction":"{label}","lastDatePriorEx":"{date}","dateClosingPricePriorExDate":"{date}","closingPricePriorExDate":"49,34","quotedPerShares":"1","corporateActionPrice":"1,366176","lastDateTimePriorEx":"2026-08-21T00:00:00"}}"#
                )
            })
            .collect();
        format!(
            r#"{{"page":{{"pageNumber":1,"pageSize":120,"totalRecords":{},"totalPages":{total_pages}}},"results":[{}]}}"#,
            rows.len(),
            rows.join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }

    fn known(list: &[(&str, Option<&str>)]) -> Vec<Known> {
        list.iter()
            .map(|(t, k)| Known {
                ticker: t.to_string(),
                share_kind: k.map(str::to_string),
            })
            .collect()
    }

    #[test]
    fn urls_are_base64_json_without_slashes() {
        let u = company_url("PETR");
        let arg = u.rsplit('/').next().unwrap();
        let json = base64::engine::general_purpose::STANDARD
            .decode(arg)
            .unwrap();
        assert_eq!(
            String::from_utf8(json).unwrap(),
            r#"{"issuingCompany":"PETR","language":"pt-br"}"#
        );
        assert!(u.starts_with(
            "https://sistemaswebb3-listados.b3.com.br/listedCompaniesProxy/CompanyCall/GetListedSupplementCompany/"
        ));
        // Name with an accent and characters that would generate "/" or "+" in base64.
        for name in ["SÃO MARTINHO", "A?B>C~D", "PETROBRAS"] {
            let u = cash_url(name, 2);
            let arg = u.rsplit('/').next().unwrap();
            assert!(!arg.contains('+'), "{u}");
            let back: serde_json::Value = serde_json::from_slice(
                &base64::engine::general_purpose::STANDARD
                    .decode(arg)
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(back["tradingName"], name);
            assert_eq!(back["pageSize"], 120);
            assert_eq!(back["pageNumber"], 2);
        }
        let f = fund_url("HGLG");
        let back = base64::engine::general_purpose::STANDARD
            .decode(f.rsplit('/').next().unwrap())
            .unwrap();
        assert_eq!(
            String::from_utf8(back).unwrap(),
            r#"{"cnpj":"0","identifierFund":"HGLG","typeFund":7}"#
        );
    }

    #[test]
    fn parses_company_fund_and_history() {
        let s = parse_company(fixtures::COMPANY_PETR.as_bytes())
            .unwrap()
            .unwrap();
        assert_eq!(s.code, "PETR");
        assert_eq!(s.trading_name.as_deref(), Some("PETROBRAS"));
        assert_eq!(s.cash.len(), 7);
        assert_eq!(s.cash[0].value, dec!(0.47156696));
        assert_eq!(s.cash[0].ex_date, d("2026-08-21"));
        assert_eq!(s.cash[0].pay_date, Some(d("2026-12-21")));
        assert_eq!(s.cash[0].approved_on, Some(d("2026-08-06")));
        assert_eq!(s.stock.len(), 1);
        assert_eq!(s.stock[0].factor, dec!(100));
        assert_eq!(parse_company(b"[]").unwrap(), None);
        assert!(parse_company(b"<html>").is_err());

        let f = parse_fund(fixtures::FUND_HGLG.as_bytes()).unwrap().unwrap();
        assert_eq!(f.code, "HGLG");
        assert_eq!(f.trading_name.as_deref(), Some("FII HGLG PAX"));
        assert_eq!(f.cash.len(), 4);
        assert_eq!(parse_fund(b"null").unwrap(), None);

        let page = fixtures::cash_page(
            3,
            &[
                ("ON", "JRS CAP PROPRIO", "21/08/2026", "0,67407131"),
                ("UNT", "DIVIDENDO", "15/05/2026", "1.234,5"),
            ],
        );
        let p = parse_cash_page(page.as_bytes()).unwrap();
        assert_eq!(p.total_pages, 3);
        assert_eq!(p.rows.len(), 2);
        assert_eq!(p.rows[1].value, dec!(1234.5));
        assert_eq!(p.rows[1].share, "UNT");
        assert_eq!(p.rows[0].pay_date, None);
        // Page too large: the API returns everything null.
        let empty = parse_cash_page(
            br#"{"page":{"pageNumber":1,"pageSize":500,"totalRecords":null,"totalPages":null},"results":[]}"#,
        )
        .unwrap();
        assert_eq!(empty, CashPage::default());
        assert_eq!(parse_br_date("31/12/9999"), None);
        assert_eq!(parse_num("205.431.960.490,52"), Some(dec!(205431960490.52)));
    }

    #[test]
    fn kinds_and_multipliers() {
        assert_eq!(cash_kind("JRS CAP PROPRIO", false), "jcp");
        assert_eq!(cash_kind("DIVIDENDO", false), "dividend");
        assert_eq!(cash_kind("RENDIMENTO", true), "fii_income");
        assert_eq!(cash_kind("RENDIMENTO", false), "other");
        assert_eq!(cash_kind("AMORTIZACAO", true), "amortization");
        assert_eq!(cash_kind("RESTITUICAO CAPITAL", false), "amortization");
        assert_eq!(stock_kind("DESDOBRAMENTO"), "split");
        assert_eq!(stock_kind("GRUPAMENTO"), "reverse_split");
        assert_eq!(stock_kind("BONIFICACAO"), "bonus");
        assert_eq!(multiplier("split", dec!(100)), Some(dec!(2)));
        assert_eq!(multiplier("split", dec!(900)), Some(dec!(10)));
        assert_eq!(multiplier("bonus", dec!(5)), Some(dec!(1.05)));
        assert_eq!(multiplier("reverse_split", dec!(0.1)), Some(dec!(0.1)));
        assert_eq!(multiplier("other", dec!(3)), None);
    }

    #[test]
    fn maps_share_types_to_tickers() {
        let k = known(&[
            ("TAEE3", Some("ON")),
            ("TAEE4", Some("PN")),
            ("TAEE11", Some("UNIT")),
            ("USIM5", Some("PN")),
            ("USIM3", Some("ON")),
            ("BBDC4", None),
            ("ABCD3", Some("ON")),
            ("XPTO11", Some("CTF")),
        ]);
        assert_eq!(share_class("BRTAEECDAM10"), Some("UNT"));
        assert_eq!(share_class("BRPETRACNOR9"), Some("ON"));
        assert_eq!(share_class("BRUSIMACNPA6"), Some("PNA"));
        assert_eq!(share_class("BRHGLGR26M17"), None, "subscription receipt");
        assert_eq!(share_class("BRPETRDBS036"), None, "debenture");
        assert_eq!(ticker_for("TAEE", "ON", &k).as_deref(), Some("TAEE3"));
        assert_eq!(ticker_for("TAEE", "PN", &k).as_deref(), Some("TAEE4"));
        assert_eq!(ticker_for("TAEE", "UNT", &k).as_deref(), Some("TAEE11"));
        assert_eq!(
            ticker_for("TAEE", "BRTAEECDAM10", &k).as_deref(),
            Some("TAEE11")
        );
        assert_eq!(ticker_for("USIM", "PNA", &k).as_deref(), Some("USIM5"));
        assert_eq!(ticker_for("USIM", "PN", &k), None, "USIM4 does not exist");
        assert_eq!(ticker_for("USIM", "PNB", &k), None);
        assert_eq!(
            ticker_for("BBDC", "BRBBDCACNPR8", &k).as_deref(),
            Some("BBDC4")
        );
        assert_eq!(
            ticker_for("XPTO", "BRXPTOCTF001", &k).as_deref(),
            Some("XPTO11")
        );
        assert_eq!(ticker_for("NADA", "ON", &k), None);
        assert_eq!(code_of("PETR4").as_deref(), Some("PETR"));
        assert_eq!(code_of("TAEE11").as_deref(), Some("TAEE"));
        assert_eq!(code_of("B3SA3").as_deref(), Some("B3SA"));
        assert_eq!(code_of("AAPL"), None);
        assert_eq!(code_of("PETR4F"), None);

        let s = parse_company(fixtures::COMPANY_PETR.as_bytes())
            .unwrap()
            .unwrap();
        let k = known(&[("PETR3", Some("ON")), ("PETR4", Some("PN"))]);
        let rows = map_cash("PETR", &s.cash, &k, false);
        assert_eq!(rows.len(), 6, "the debenture is left out");
        assert_eq!(rows[2].ticker, "PETR4");
        assert_eq!(rows[5].kind, "other");
        let ev = map_stock("PETR", &s.stock, &k);
        assert_eq!(ev[0].ticker, "PETR3");
        assert_eq!(ev[0].kind, "split");
    }

    #[test]
    fn rolling_plan_registered_first_then_oldest_with_daily_budget() {
        let now = Utc::now();
        let ago = |h: i64| now - chrono::Duration::hours(h);
        let t = |code: &str, reg: bool| Target {
            code: code.into(),
            fund: false,
            registered: reg,
        };
        let targets = vec![
            t("PETR", true),
            t("ITUB", true),
            t("AAAA", false),
            t("BBBB", false),
            t("CCCC", false),
            t("DDDD", false),
            t("EEEE", false),
            t("FFFF", false),
        ];
        let mut log = HashMap::new();
        let put = |log: &mut HashMap<String, LogEntry>, k: &str, h: i64, st: &str| {
            log.insert(
                format!("cia:{k}"),
                LogEntry {
                    at: ago(h),
                    status: st.into(),
                },
            );
        };
        put(&mut log, "PETR", 2, "ok"); // registered, queried recently
        put(&mut log, "ITUB", 21, "ok"); // registered, overdue
        put(&mut log, "AAAA", 100, "ok");
        put(&mut log, "BBBB", 30, "ok");
        put(&mut log, "CCCC", 30, "error"); // error over a day ago: tries again
        put(&mut log, "DDDD", 30, "not_found"); // non-existent: only after a week
        // EEEE and FFFF never queried.
        let codes = |v: Vec<Target>| v.into_iter().map(|t| t.code).collect::<Vec<_>>();
        assert_eq!(
            codes(plan_batch(&targets, &log, now, 3, false)),
            vec!["ITUB", "EEEE", "FFFF", "AAAA"]
        );
        assert_eq!(
            codes(plan_batch(&targets, &log, now, 10, false)),
            vec!["ITUB", "EEEE", "FFFF", "AAAA", "BBBB", "CCCC"]
        );
        assert_eq!(
            codes(plan_batch(&targets, &log, now, 0, true)),
            vec!["ITUB", "PETR"],
            "forced: all registered"
        );
        // The day's batch already used: two universe queries in the last 20 h.
        put(&mut log, "EEEE", 1, "ok");
        put(&mut log, "FFFF", 1, "ok");
        assert_eq!(
            codes(plan_batch(&targets, &log, now, 3, false)),
            vec!["ITUB", "AAAA"]
        );
        assert_eq!(default_batch(300), 60);
        assert_eq!(default_batch(600), 86);
        // History: registered only, once (error, after a day).
        assert!(history_due(&t("ITUB", true), &log, now));
        assert!(!history_due(&t("AAAA", false), &log, now));
        log.insert(
            "hist:ITUB".into(),
            LogEntry {
                at: ago(2),
                status: "error".into(),
            },
        );
        assert!(!history_due(&t("ITUB", true), &log, now));
        log.insert(
            "hist:ITUB".into(),
            LogEntry {
                at: ago(200),
                status: "ok".into(),
            },
        );
        assert!(!history_due(&t("ITUB", true), &log, now));
    }
}

#[cfg(test)]
mod sync_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use super::super::net::{FetchResult, Fetched, Fetcher, FileMeta};
    use super::super::sync::{SyncArgs, SyncSource, SyncStatus, run_with};
    use super::super::{Issuer, IssuerKind, Layout, Security, store};
    use super::*;
    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    #[derive(Default)]
    struct Fixture {
        files: HashMap<String, Vec<u8>>,
        errors: HashMap<String, String>,
        calls: Mutex<Vec<String>>,
    }

    impl Fetcher for Fixture {
        fn get(&self, url: &str, _cond: Option<&FileMeta>, _ua: &str) -> FetchResult {
            self.calls.lock().unwrap().push(url.to_string());
            if let Some(e) = self.errors.get(url) {
                return Err(e.clone());
            }
            match self.files.get(url) {
                None => Ok(Fetched::NotFound),
                Some(b) => Ok(Fetched::Body {
                    bytes: b.clone(),
                    meta: FileMeta::default(),
                }),
            }
        }
    }

    fn issuer(id: &str, kind: IssuerKind, name: &str) -> Issuer {
        Issuer {
            id: id.into(),
            kind,
            name: name.into(),
            cvm_code: None,
            sector: None,
            segment: None,
            mandate: None,
            layout: if kind == IssuerKind::Fii {
                Layout::Fund
            } else {
                Layout::Industrial
            },
            currency: "BRL".into(),
            source: "teste".into(),
        }
    }

    fn security(t: &str, issuer: &str, class: &str, kind: &str) -> Security {
        Security {
            ticker: t.into(),
            issuer_id: issuer.into(),
            class: class.into(),
            share_kind: Some(kind.into()),
            unit_composition: None,
            isin: None,
            listing_segment: None,
            active: true,
            source: "teste".into(),
        }
    }

    #[test]
    fn rolling_sync_stores_official_dividends_and_skips_on_second_run() {
        let dir = tempfile::tempdir().unwrap();
        let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
        let api = Arc::new(Api::new_unlocked_process(paths).unwrap());
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "Família Teste"}),
        )
        .unwrap();
        let petr = api
            .call(
                "assets.create",
                json!({"ticker": "PETR4", "name": "Petrobras PN", "class": "stock_br"}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        api.call(
            "assets.create",
            json!({"ticker": "HGLG11", "name": "CSHG Logística", "class": "fii"}),
        )
        .unwrap();
        api.with_session(|s, _| {
            let c = s.db.conn();
            let now = "2026-10-15T10:00:00Z";
            store::upsert_issuer(c, &issuer("33000167000101", IssuerKind::CompanyBr, "PETROBRAS"), now)?;
            store::upsert_issuer(c, &issuer("33592510000154", IssuerKind::CompanyBr, "VALE"), now)?;
            store::upsert_issuer(c, &issuer("11222333000100", IssuerKind::Fii, "XPTO FII"), now)?;
            store::upsert_security(c, &security("PETR3", "33000167000101", "stock_br", "ON"), now)?;
            store::upsert_security(c, &security("PETR4", "33000167000101", "stock_br", "PN"), now)?;
            store::upsert_security(c, &security("VALE3", "33592510000154", "stock_br", "ON"), now)?;
            store::upsert_security(c, &security("XPTO11", "11222333000100", "fii", "CTF"), now)?;
            store::upsert_security(c, &security("SEMN3", "33592510000154", "stock_br", "ON"), now)?;
            for (t, close) in [("PETR4", "40"), ("PETR3", "42"), ("VALE3", "60"), ("XPTO11", "95")] {
                c.execute(
                    "INSERT INTO research_quote (ticker, date, close, volume, trades, source, fetched_at)
                     VALUES (?1, '2026-10-15', ?2, '1000000', 10, 'b3_cotahist', 'x')",
                    params![t, close],
                )?;
            }
            // Yahoo calendar different from B3's (wrong sum on the ex date).
            c.execute(
                "INSERT INTO dividend_event (id, asset_id, kind, ex_date, amount_per_share, currency, estimated, source, fetched_at)
                 VALUES ('y1', ?1, 'dividend', '2026-08-24', '1.0', 'BRL', 0, 'yahoo', 'x')",
                [&petr],
            )?;
            Ok(())
        })
        .unwrap();

        let mut f = Fixture::default();
        f.files.insert(
            company_url("PETR"),
            fixtures::COMPANY_PETR.as_bytes().to_vec(),
        );
        f.files.insert(
            cash_url("PETROBRAS", 1),
            fixtures::cash_page(
                2,
                &[
                    ("ON", "JRS CAP PROPRIO", "21/08/2026", "0,67407131"),
                    ("PN", "JRS CAP PROPRIO", "21/08/2026", "0,67407131"),
                    ("PN", "DIVIDENDO", "22/12/2025", "0,29642144"),
                    ("PN", "DIVIDENDO", "16/04/2024", "1,0"),
                ],
            )
            .into_bytes(),
        );
        f.files.insert(
            cash_url("PETROBRAS", 2),
            fixtures::cash_page(2, &[("PN", "DIVIDENDO", "10/05/2023", "0,5")]).into_bytes(),
        );
        f.files
            .insert(fund_url("HGLG"), fixtures::FUND_HGLG.as_bytes().to_vec());
        f.errors
            .insert(company_url("VALE"), "tempo esgotado".into());
        // Fund with no B3 data: 200 with no content (twice) is not a collection error.
        f.files.insert(fund_url("XPTO"), b"  ".to_vec());
        let args = SyncArgs {
            force: None,
            sources: Some(vec![SyncSource::B3Listados]),
        };
        let today = crate::time::parse_date("2026-10-16").unwrap();
        let rep = run_with(&api, &args, &f, today).unwrap();
        let calls = f.calls.lock().unwrap().clone();
        assert_eq!(
            calls.len(),
            7,
            "PETR + 2 pages, HGLG, VALE and XPTO (twice): {calls:#?}"
        );
        assert!(
            !rep.items.iter().any(|i| i.label.contains("XPTO")),
            "no content does not become an item with a problem"
        );
        let summary = rep
            .items
            .iter()
            .find(|i| {
                i.source == SyncSource::B3Listados
                    && i.url.is_none()
                    && i.status != SyncStatus::Error
            })
            .unwrap();
        assert_eq!(summary.status, SyncStatus::Updated);
        assert!(
            rep.items
                .iter()
                .any(|i| i.error.as_deref() == Some("tempo esgotado"))
        );
        assert!(!rep.ok);

        api.with_session(|s, _| {
            let c = s.db.conn();
            let rows = |t: &str| -> Vec<(String, String, String, Option<String>)> {
                c.prepare("SELECT ex_date, kind, value, pay_date FROM research_dividend WHERE ticker = ?1 ORDER BY ex_date, kind, seq")
                    .unwrap()
                    .query_map([t], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                    .unwrap()
                    .collect::<rusqlite::Result<_>>()
                    .unwrap()
            };
            // PETR4: history (before 21/08/2026) + supplement (from 21/08/2026,
            // with payment date).
            let p4 = rows("PETR4");
            assert_eq!(p4.len(), 4, "{p4:#?}");
            assert_eq!(p4[0].0, "2023-05-10");
            assert_eq!(p4[3], ("2026-08-21".into(), "dividend".into(), "0.47156696".into(), Some("2026-12-21".into())));
            // PETR3: JCP in two equal installments becomes two rows.
            let p3 = rows("PETR3");
            assert_eq!(p3.iter().filter(|r| r.0 == "2026-06-01").count(), 2, "{p3:#?}");
            assert!(p3.iter().any(|r| r.1 == "other"), "RENDIMENTO of a company");
            // FII: shares (without the subscription receipt), with the amortization separate.
            let h = rows("HGLG11");
            assert_eq!(h.len(), 3, "{h:#?}");
            assert!(h.iter().any(|r| r.1 == "amortization"));
            let ev: i64 = c.query_row("SELECT count(*) FROM research_stock_event WHERE ticker IN ('PETR3', 'HGLG11')", [], |r| r.get(0))?;
            assert_eq!(ev, 2);
            let log: HashMap<String, String> = c
                .prepare("SELECT key, status FROM research_fetch_log WHERE source = 'b3_listados'")?
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?;
            assert_eq!(log["cia:PETR"], "ok");
            assert_eq!(log["hist:PETR"], "ok");
            assert_eq!(log["fii:HGLG"], "ok");
            assert_eq!(log["cia:VALE"], "error");
            assert_eq!(log["fii:XPTO"], "empty");
            assert!(!log.contains_key("cia:SEMN"), "no quote: outside the universe");
            // Fund that B3 answered with no income: no yield (source without points); what
            // came back empty, or queried long ago, stays with no source (the estimate applies).
            c.execute(
                "INSERT INTO research_fetch_log (source, key, fetched_at, status) VALUES
                   ('b3_listados', 'fii:ZERO', '2026-10-15T10:00:00Z', 'ok'),
                   ('b3_listados', 'fii:VELH', '2026-06-01T10:00:00Z', 'ok')",
                [],
            )?;
            let sel = |t: &str| super::super::dividends::select(c, None, Some(t), today).unwrap();
            let (pts, src) = sel("ZERO11");
            assert!(pts.is_empty() && src.is_some_and(|s| s.contains("B3")));
            assert_eq!(sel("XPTO11").1, None);
            assert_eq!(sel("VELH11").1, None);
            // A repeated yield row counts once; the amortization (share at 80 the day before
            // the record date) puts the previous yield on the scale of what is left.
            c.execute_batch(
                "INSERT INTO research_dividend (ticker, kind, ex_date, value, per_shares, pay_date, label, seq, source, fetched_at) VALUES
                   ('AMRT11', 'fii_income', '2025-12-30', '15.3', 1, '2026-03-27', 'RENDIMENTO', 0, 'b3_listados', 'x'),
                   ('AMRT11', 'fii_income', '2025-12-30', '15.3', 1, '2026-03-27', 'RENDIMENTO', 1, 'b3_listados', 'x'),
                   ('AMRT11', 'amortization', '2026-04-07', '76.5', 1, '2026-04-14', 'AMORTIZACAO RF', 0, 'b3_listados', 'x');
                 INSERT INTO research_quote (ticker, date, close, volume, source, fetched_at) VALUES
                   ('AMRT11', '2026-04-06', '80', '1000', 'b3_cotahist', 'x');",
            )?;
            let off = super::super::dividends::Official::load(c, "AMRT11")?;
            assert_eq!(off.cash.len(), 2, "{off:#?}");
            assert_eq!(off.capital.len(), 1);
            let pts = off.points(today);
            assert_eq!(pts.len(), 1);
            assert_eq!(pts[0].amount.round_dp(6), Decimal::new(669375, 6), "15,30 × 3,5 ÷ 80");
            let (_, src) = sel("AMRT11");
            assert!(src.unwrap().contains("07/04/2026"));
            Ok(())
        })
        .unwrap();

        // Second collection right after: nothing overdue, no requests.
        let rep2 = run_with(&api, &args, &f, today).unwrap();
        assert_eq!(f.calls.lock().unwrap().len(), 7);
        assert!(
            rep2.items
                .iter()
                .any(|i| i.source == SyncSource::B3Listados && i.status == SyncStatus::Unchanged)
        );

        // Indicators: the calendar (Yahoo) diverges, B3 becomes the DY reference.
        let v = api
            .call("research.indicators", json!({"asset_id": petr}))
            .unwrap();
        let dy = v["indicators"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["key"] == "dy")
            .unwrap()
            .clone();
        assert!(
            (dy["value"].as_f64().unwrap() - 0.7679884 / 40.0).abs() < 1e-9,
            "{dy}"
        );
        assert!(dy["source"].as_str().unwrap().contains("B3"), "{dy}");
        // Security only in the universe: B3's DY.
        let p3 = api
            .call("research.indicators", json!({"ticker": "PETR3"}))
            .unwrap();
        let dy3 = p3["indicators"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["key"] == "dy")
            .unwrap()
            .clone();
        assert!(dy3["value"].as_f64().unwrap() > 0.0, "{dy3}");
        let chk = api
            .call("research.dividends_check", json!({"asset_id": petr}))
            .unwrap();
        assert_eq!(chk["material"], true, "{chk:#}");
        assert_eq!(chk["reference"], "b3");
        assert_eq!(chk["b3_status"], "ok");
        assert_eq!(chk["value_diffs"], 1, "{chk:#}");
    }
}
