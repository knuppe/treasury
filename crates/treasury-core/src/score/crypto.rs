//! Criptoativos: rede, tokenomics, liquidez e risco (robustez dentro da classe, não comparável
//! com ações). Stablecoins: lastro, histórico de paridade, liquidez e regulação (§10.5).
//!
//! Dados: CoinGecko (valor de mercado, ranking, FDV, oferta, volume, máxima histórica,
//! desenvolvimento, categorias) e DefiLlama (TVL das redes, lista das stablecoins com o
//! lastro, valor de mercado e preços diários em dólar), gravados em `fundamental` com o
//! emissor `coingecko:<id>`; volatilidade e drawdown pelas cotações do app. Sem universo de
//! pares: só âncoras absolutas.

pub mod coingecko;
pub mod defillama;
pub mod onchain;
pub mod sync;

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};

use super::config::{Methodology, Unit};
use super::engine;
use super::funds::series::{self, Point};
use super::metrics::{MetricValue, Metrics, br};
use super::{AssetScore, ScoreFlag, taxonomy};
use crate::assets::Asset;
use crate::error::Result;
use crate::research::indicators::Data;
use crate::research::{Fact, PeriodType, store, to_f64};
use crate::time::{add_months, fmt_date};

/// Frase obrigatória: cripto não se compara com ações (no idioma da instalação).
pub fn not_comparable() -> String {
    crate::i18n::t("nucleo-score-cripto-nao-comparavel")
}
pub const SETTING: &str = "score.crypto";
/// Dados de mercado: no máximo uma coleta por dia.
pub const REFRESH_HOURS: i64 = 20;
const MIN_CLOSES_12M: usize = 120;

/// Itens gravados em `fundamental` (emissor `coingecko:<id>`).
pub mod keys {
    pub const MCAP: &str = "crypto_mcap_usd";
    pub const FDV: &str = "crypto_fdv_usd";
    pub const VOLUME: &str = "crypto_volume_usd";
    pub const PRICE: &str = "crypto_price_usd";
    pub const RANK: &str = "crypto_rank";
    pub const CIRCULATING: &str = "crypto_circulating";
    pub const TOTAL_SUPPLY: &str = "crypto_total_supply";
    pub const MAX_SUPPLY: &str = "crypto_max_supply";
    pub const WATCHLIST: &str = "crypto_watchlist_users";
    pub const COMMITS: &str = "crypto_commits_4w";
    pub const STARS: &str = "crypto_github_stars";
    pub const ATH_DRAWDOWN: &str = "crypto_ath_drawdown";
    pub const SUPPLY_GROWTH: &str = "crypto_supply_growth_1y";
    pub const TVL: &str = "crypto_tvl_usd";
    /// Valor de mercado pela DefiLlama (sem a CoinGecko).
    pub const MCAP_LLAMA: &str = "crypto_mcap_llama_usd";
    /// Preço diário em dólar (DefiLlama), um fato por dia.
    pub const PRICE_DAILY: &str = "crypto_price_usd_daily";
    /// Média de endereços ativos por dia em 30 dias (Coin Metrics).
    pub const ACTIVE_ADDRESSES: &str = "crypto_active_addresses_30d";
    /// Pools com mais da metade dos blocos da semana (mempool.space, só bitcoin).
    pub const NAKAMOTO: &str = "crypto_nakamoto_pools";
    /// Incidentes contra a rede na lista de ataques da DefiLlama.
    pub const INCIDENTS: &str = "crypto_chain_incidents";
}

pub fn issuer(id: &str) -> String {
    format!("coingecko:{id}")
}

// ---------------------------------------------------------------------------
// Configuração extra

#[derive(Debug, Clone, Deserialize)]
pub struct CryptoConfig {
    pub stablecoins: Vec<String>,
    pub known_ids: BTreeMap<String, String>,
    pub backing_scores: BTreeMap<String, f64>,
    pub depeg_threshold: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct Extras {
    crypto: CryptoConfig,
}

pub fn config() -> &'static CryptoConfig {
    static C: OnceLock<CryptoConfig> = OnceLock::new();
    C.get_or_init(|| {
        let e: Extras = serde_json::from_str(include_str!(
            "../../../../methodology/v0.1.0/classes/crypto.json"
        ))
        .expect("classes/crypto.json válido");
        e.crypto
    })
}

/// Prazo que os termos da API da CoinGecko dão para renovar um dado guardado.
pub const COINGECKO_MAX_AGE_HOURS: i64 = 24;

/// O dado da CoinGecko coletado em `at` (RFC 3339) passou do prazo de renovação.
pub fn coingecko_stale(at: &str, now: chrono::DateTime<chrono::Utc>) -> bool {
    chrono::DateTime::parse_from_rfc3339(at)
        .is_ok_and(|t| (now - t.with_timezone(&chrono::Utc)).num_hours() >= COINGECKO_MAX_AGE_HOURS)
}

/// A data da coleta (`2026-11-18T10:00:00Z`) no formato das notas do score.
fn br_date(at: &str) -> String {
    at.get(..10)
        .and_then(|d| crate::time::parse_date(d).ok())
        .map(br)
        .unwrap_or_else(|| at.to_string())
}

// ---------------------------------------------------------------------------
// Cadastro técnico (ids, categorias, stablecoins)

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CoinMeta {
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub genesis_date: Option<String>,
    #[serde(default)]
    pub stablecoin: bool,
    pub coingecko_at: Option<String>,
    pub llama_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CryptoRegistry {
    /// Id do ativo → id da CoinGecko.
    #[serde(default)]
    pub ids: BTreeMap<String, String>,
    #[serde(default)]
    pub coins: BTreeMap<String, CoinMeta>,
    /// Stablecoins da DefiLlama de interesse (id da CoinGecko → dados).
    #[serde(default)]
    pub stables: BTreeMap<String, defillama::Stable>,
    /// Redes da DefiLlama de interesse (id → símbolo, TVL em dólar).
    #[serde(default)]
    pub chains: BTreeMap<String, (String, f64)>,
    pub lists_at: Option<String>,
    /// Ativos da Coin Metrics com endereços ativos na API Community, e quando foi lido.
    #[serde(default)]
    pub cm_assets: BTreeSet<String>,
    #[serde(default)]
    pub onchain_at: Option<String>,
}

pub fn load(conn: &Connection) -> Result<CryptoRegistry> {
    Ok(crate::settings::get(conn, SETTING)?.unwrap_or_default())
}

pub fn save(conn: &Connection, r: &CryptoRegistry) -> Result<()> {
    crate::settings::set(conn, SETTING, r)
}

/// Símbolo de um criptoativo cadastrado (ticker sem o par: "BTC-USD" → "BTC").
pub fn symbol_of(a: &Asset) -> Option<String> {
    let t = a.ticker.as_deref()?.trim().to_uppercase();
    let s = t.split(['-', '/']).next().unwrap_or(&t).to_string();
    (!s.is_empty()).then_some(s)
}

/// Id da CoinGecko fixado no ativo (fonte de cotação `coingecko` + símbolo do provedor).
pub fn pinned_id(a: &Asset) -> Option<String> {
    (a.price_provider.as_deref() == Some("coingecko"))
        .then(|| a.provider_symbol.clone())
        .flatten()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
}

/// Id sem rede: fixado no ativo, tabela da metodologia, stablecoins e redes da DefiLlama.
pub fn local_id(a: &Asset, reg: &CryptoRegistry) -> Option<String> {
    if let Some(p) = pinned_id(a) {
        return Some(p);
    }
    let sym = symbol_of(a)?;
    if let Some(id) = config().known_ids.get(&sym) {
        return Some(id.clone());
    }
    let stable = reg
        .stables
        .values()
        .filter(|s| s.symbol == sym)
        .max_by(|x, y| {
            x.circulating
                .unwrap_or(0.0)
                .total_cmp(&y.circulating.unwrap_or(0.0))
        })
        .map(|s| s.gecko_id.clone());
    stable.or_else(|| {
        reg.chains
            .iter()
            .filter(|(_, (s, _))| *s == sym)
            .max_by(|x, y| x.1.1.total_cmp(&y.1.1))
            .map(|(id, _)| id.clone())
    })
}

pub fn is_stablecoin(reg: &CryptoRegistry, id: Option<&str>, symbol: Option<&str>) -> bool {
    id.is_some_and(|i| {
        reg.coins.get(i).is_some_and(|c| c.stablecoin) || reg.stables.contains_key(i)
    }) || symbol.is_some_and(|s| config().stablecoins.iter().any(|x| x == s))
}

fn dec(x: f64) -> Option<Decimal> {
    Decimal::from_f64(x).map(|d| d.round_dp(12).normalize())
}

/// Fato instantâneo de um criptoativo.
pub fn fact(
    item: &str,
    end: NaiveDate,
    value: f64,
    currency: &str,
    source: &str,
    doc: &str,
) -> Option<Fact> {
    Some(Fact {
        item: item.into(),
        period_type: PeriodType::Instant,
        start: None,
        end,
        value: dec(value)?,
        currency: currency.into(),
        source: source.into(),
        document: doc.into(),
        priority: 2,
    })
}

// ---------------------------------------------------------------------------
// Métricas

fn latest<'a>(facts: &'a [Fact], item: &str) -> Option<&'a Fact> {
    facts
        .iter()
        .filter(|f| f.item == item && f.period_type == PeriodType::Instant)
        .max_by_key(|f| f.end)
}

fn ok(v: f64, period: impl Into<String>, source: impl Into<String>) -> MetricValue {
    MetricValue {
        period: Some(period.into()),
        source: Some(source.into()),
        ..MetricValue::ok(v)
    }
}

fn noted(mut v: MetricValue, note: impl Into<String>) -> MetricValue {
    v.note = Some(note.into());
    v
}

fn pct(x: f64) -> String {
    crate::ai::alerts::pct(crate::i18n::current(), x, 1)
}

/// Entradas puras das métricas de um criptoativo.
pub struct CryptoInput<'a> {
    pub facts: &'a [Fact],
    pub meta: Option<&'a CoinMeta>,
    pub stable: Option<&'a defillama::Stable>,
    /// Cotações do app (moeda do ativo) e a moeda delas.
    pub prices: &'a [Point],
    pub price_currency: &'a str,
    /// Dólar em reais (PTAX) e a data da taxa.
    pub usd_brl: Option<(f64, String)>,
    pub today: NaiveDate,
}

pub struct CryptoOutput {
    pub metrics: Metrics,
    pub data_date: Option<NaiveDate>,
    pub notes: Vec<String>,
}

pub fn crypto_metrics(i: &CryptoInput<'_>) -> CryptoOutput {
    let mut ms = Metrics::new();
    let mut notes = vec![];
    let f = i.facts;
    let cg = |k: &str| latest(f, k).map(|x| (to_f64(x.value), x.end));
    let period = |d: NaiveDate| {
        crate::i18n::t_args(
            "nucleo-score-cripto-periodo-coleta",
            &crate::i18n::args(&[("data", br(d).into())]),
        )
    };
    let src_cg = "CoinGecko";
    let src_ll = "DefiLlama";
    // `what`: chave do dado que falta (`nucleo-score-cripto-dado-*`).
    let no_cg = |what: &str| {
        MetricValue::missing(crate::i18n::t_args(
            "nucleo-score-cripto-sem-coingecko",
            &crate::i18n::args(&[("dado", crate::i18n::t(what).into())]),
        ))
    };
    let brl = |usd: f64| i.usd_brl.as_ref().map(|(r, _)| usd * r);
    // Valores em dólar convertidos pela PTAX (a taxa vai na fonte: em valores monetários a
    // explicação mostra o valor, não a nota).
    let fx_src = |src: &str| match &i.usd_brl {
        Some((r, d)) => crate::i18n::t_args(
            "nucleo-score-cripto-fonte-dolar",
            &crate::i18n::args(&[
                ("fonte", src.to_string().into()),
                (
                    "taxa",
                    crate::ai::alerts::decimal(crate::i18n::current(), &format!("{r:.4}")).into(),
                ),
                ("data", d.clone().into()),
            ]),
        ),
        None => src.to_string(),
    };
    let money = |usd: Option<(f64, NaiveDate)>, src: &str, what: &str| -> MetricValue {
        match usd {
            Some((v, d)) => match brl(v) {
                Some(b) => ok(b, period(d), fx_src(src)),
                None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-ptax")),
            },
            None => no_cg(what),
        }
    };
    // Mercado.
    let stable_mcap = i.stable.and_then(|s| {
        (s.peg_currency() == "USD")
            .then_some(s.circulating)
            .flatten()
            .map(|c| c * s.price_usd.unwrap_or(1.0))
    });
    let mcap_usd = cg(keys::MCAP)
        .map(|x| (x, src_cg))
        .or_else(|| cg(keys::MCAP_LLAMA).map(|x| (x, src_ll)))
        .or_else(|| stable_mcap.map(|m| ((m, i.today), src_ll)));
    ms.insert(
        "crypto_market_cap".into(),
        match mcap_usd {
            Some((v, src)) => money(Some(v), src, "nucleo-score-cripto-dado-valor-mercado"),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-valor-mercado")),
        },
    );
    ms.insert(
        "crypto_rank".into(),
        match cg(keys::RANK) {
            Some((v, d)) => ok(v, period(d), src_cg),
            None => no_cg("nucleo-score-cripto-dado-ranking"),
        },
    );
    let vol = cg(keys::VOLUME);
    ms.insert(
        "crypto_volume".into(),
        money(vol, src_cg, "nucleo-score-cripto-dado-volume"),
    );
    ms.insert(
        "crypto_turnover".into(),
        match (vol, mcap_usd) {
            (Some((v, d)), Some(((m, _), _))) if m > 0.0 => ok(v / m, period(d), src_cg),
            _ => no_cg("nucleo-score-cripto-dado-volume-valor"),
        },
    );
    // Tokenomics.
    ms.insert(
        "crypto_fdv_to_mcap".into(),
        match (cg(keys::FDV), cg(keys::MCAP)) {
            (Some((fdv, d)), Some((m, _))) if m > 0.0 => ok(fdv / m, period(d), src_cg),
            _ => no_cg("nucleo-score-cripto-dado-fdv"),
        },
    );
    ms.insert(
        "crypto_supply_emitted".into(),
        match (cg(keys::CIRCULATING), cg(keys::MAX_SUPPLY)) {
            (Some((c, d)), Some((mx, _))) if mx > 0.0 => ok((c / mx).min(1.0), period(d), src_cg),
            (Some(_), None) => {
                MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-oferta-maxima"))
            }
            _ => no_cg("nucleo-score-cripto-dado-oferta"),
        },
    );
    ms.insert(
        "crypto_supply_inflation".into(),
        match cg(keys::SUPPLY_GROWTH) {
            Some((g, d)) => ok(g, crate::research::indicators::ltm_until(d), src_cg),
            None => no_cg("nucleo-score-cripto-dado-historico"),
        },
    );
    // Rede e adoção.
    ms.insert(
        "crypto_tvl".into(),
        match cg(keys::TVL) {
            Some((v, d)) => match brl(v) {
                Some(b) => ok(b, period(d), fx_src(src_ll)),
                None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-ptax")),
            },
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-tvl")),
        },
    );
    ms.insert(
        "crypto_watchlist_users".into(),
        match cg(keys::WATCHLIST) {
            Some((v, d)) => ok(v, period(d), src_cg),
            None => no_cg("nucleo-score-cripto-dado-usuarios"),
        },
    );
    // Segurança: tempo de rede.
    ms.insert(
        "crypto_age_years".into(),
        match i
            .meta
            .and_then(|m| m.genesis_date.as_deref())
            .and_then(|s| crate::time::parse_date(s).ok())
        {
            Some(g) => ok(
                (i.today - g).num_days() as f64 / 365.25,
                crate::i18n::t_args(
                    "nucleo-score-cripto-periodo-genese",
                    &crate::i18n::args(&[("data", br(g).into())]),
                ),
                src_cg,
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-genese")),
        },
    );
    // On-chain (Coin Metrics, mempool.space, DefiLlama).
    ms.insert(
        "crypto_active_addresses".into(),
        match cg(keys::ACTIVE_ADDRESSES) {
            Some((v, d)) => noted(
                ok(
                    v,
                    crate::i18n::t_args(
                        "nucleo-score-cripto-periodo-media-30d",
                        &crate::i18n::args(&[("data", br(d).into())]),
                    ),
                    "Coin Metrics (Community)",
                ),
                crate::i18n::t("nucleo-score-cripto-nota-coin-metrics"),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-coin-metrics")),
        },
    );
    ms.insert(
        "crypto_nakamoto".into(),
        match cg(keys::NAKAMOTO) {
            Some((v, d)) => noted(
                ok(
                    v,
                    crate::i18n::t_args(
                        "nucleo-score-cripto-periodo-semana",
                        &crate::i18n::args(&[("data", br(d).into())]),
                    ),
                    "mempool.space",
                ),
                crate::i18n::t("nucleo-score-cripto-nota-nakamoto"),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-nakamoto")),
        },
    );
    ms.insert(
        "crypto_hacks".into(),
        match cg(keys::INCIDENTS) {
            Some((n, d)) => noted(
                ok(
                    (100.0 - 50.0 * n).max(0.0),
                    period(d),
                    crate::i18n::t("nucleo-score-cripto-fonte-ataques"),
                ),
                crate::i18n::t_args(
                    "nucleo-score-cripto-nota-ataques",
                    &crate::i18n::args(&[("n", i64::from(n as u32).into())]),
                ),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-ataques")),
        },
    );
    ms.insert(
        "crypto_unlocks".into(),
        match i.meta {
            Some(m) if onchain::is_proof_of_work(&m.categories) => noted(
                ok(
                    0.0,
                    crate::i18n::t("nucleo-score-cripto-periodo-regra"),
                    crate::i18n::t("nucleo-score-cripto-fonte-categorias"),
                ),
                crate::i18n::t("nucleo-score-cripto-nota-pow"),
            ),
            _ => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-sem-desbloqueios")),
        },
    );
    // Desenvolvimento.
    for (key, item, what) in [
        (
            "crypto_commits_4w",
            keys::COMMITS,
            "nucleo-score-cripto-dado-commits",
        ),
        (
            "crypto_github_stars",
            keys::STARS,
            "nucleo-score-cripto-dado-estrelas",
        ),
    ] {
        ms.insert(
            key.into(),
            match cg(item) {
                Some((v, d)) => ok(
                    v,
                    period(d),
                    crate::i18n::t("nucleo-score-cripto-fonte-github"),
                ),
                None => no_cg(what),
            },
        );
    }
    // Risco: cotações do app; sem elas, preços em dólar da DefiLlama.
    let usd_hist: Vec<Point> = {
        let mut v: Vec<Point> = f
            .iter()
            .filter(|x| x.item == keys::PRICE_DAILY)
            .map(|x| (x.end, to_f64(x.value)))
            .filter(|p| p.1 > 0.0)
            .collect();
        v.sort_by_key(|p| p.0);
        v
    };
    let last_year = |pts: &[Point]| -> Vec<Point> {
        match pts.last() {
            Some(l) => series::window(pts, add_months(l.0, -12), l.0),
            None => vec![],
        }
    };
    let app = last_year(i.prices);
    let (risk_pts, risk_src) = if app.len() >= MIN_CLOSES_12M {
        (app, app_quotes(i.price_currency))
    } else {
        (
            last_year(&usd_hist),
            crate::i18n::t("nucleo-score-cripto-fonte-precos-dolar"),
        )
    };
    let risk_period = match (risk_pts.first(), risk_pts.last()) {
        (Some(a), Some(b)) => crate::i18n::t_args(
            "nucleo-score-periodo-de-a",
            &crate::i18n::args(&[("de", br(a.0).into()), ("ate", br(b.0).into())]),
        ),
        _ => String::new(),
    };
    let enough = risk_pts.len() >= MIN_CLOSES_12M;
    ms.insert(
        "crypto_volatility".into(),
        match series::volatility(&risk_pts).filter(|_| enough) {
            Some(v) => ok(v, &risk_period, &risk_src),
            None => MetricValue::missing(crate::i18n::t_args(
                "nucleo-score-cripto-poucas-cotacoes",
                &crate::i18n::args(&[("n", (MIN_CLOSES_12M as i64).into())]),
            )),
        },
    );
    ms.insert(
        "crypto_max_drawdown".into(),
        match series::max_drawdown(&risk_pts).filter(|_| enough) {
            Some(dd) => noted(
                ok(dd.depth, &risk_period, &risk_src),
                crate::i18n::t_args(
                    "nucleo-score-nota-pico-fundo",
                    &crate::i18n::args(&[
                        ("pico", br(dd.peak).into()),
                        ("fundo", br(dd.trough).into()),
                    ]),
                ),
            ),
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-menos-12-meses")),
        },
    );
    ms.insert(
        "crypto_ath_drawdown".into(),
        match cg(keys::ATH_DRAWDOWN) {
            Some((v, d)) => ok(v, period(d), src_cg),
            None => no_cg("nucleo-score-cripto-dado-maxima"),
        },
    );
    // Stablecoin: lastro e paridade.
    let cfg = config();
    ms.insert(
        "stable_backing".into(),
        match i.stable {
            Some(s) => match cfg.backing_scores.get(&s.mechanism) {
                Some(v) => noted(
                    ok(
                        *v,
                        crate::i18n::t("nucleo-score-cripto-periodo-stablecoins"),
                        src_ll,
                    ),
                    backing_label(&s.mechanism),
                ),
                None => MetricValue::missing(crate::i18n::t_args(
                    "nucleo-score-cripto-lastro-sem-nota",
                    &crate::i18n::args(&[("lastro", s.mechanism.clone().into())]),
                )),
            },
            None => MetricValue::missing(crate::i18n::t("nucleo-score-cripto-fora-stablecoins")),
        },
    );
    let peg = i
        .stable
        .map(|s| match s.peg_currency().as_str() {
            "REAL" => "BRL".to_string(),
            c => c.to_string(),
        })
        .unwrap_or_else(|| "USD".into());
    let parity_pts: Option<(Vec<Point>, String)> = if peg == "USD" && usd_hist.len() >= 30 {
        Some((
            last_year(&usd_hist),
            crate::i18n::t("nucleo-score-cripto-fonte-precos-dolar"),
        ))
    } else if i.price_currency == peg && i.prices.len() >= 30 {
        Some((last_year(i.prices), app_quotes(&peg)))
    } else {
        None
    };
    match parity_pts.and_then(|(p, s)| Some((series::parity(&p, 1.0, cfg.depeg_threshold)?, p, s)))
    {
        Some(((max, off, avg), p, s)) => {
            let per = crate::i18n::t_args(
                "nucleo-score-cripto-periodo-dias",
                &crate::i18n::args(&[
                    ("de", br(p[0].0).into()),
                    ("ate", br(p[p.len() - 1].0).into()),
                    ("dias", (p.len() as i64).into()),
                ]),
            );
            ms.insert(
                "stable_max_deviation".into(),
                noted(
                    ok(max, &per, &s),
                    crate::i18n::t_args(
                        "nucleo-score-cripto-nota-paridade",
                        &crate::i18n::args(&[("moeda", peg.clone().into())]),
                    ),
                ),
            );
            ms.insert("stable_depeg_days".into(), ok(off, &per, &s));
            ms.insert("stable_mean_deviation".into(), ok(avg, &per, &s));
        }
        None => {
            let why = MetricValue::missing(crate::i18n::t_args(
                "nucleo-score-cripto-sem-precos-paridade",
                &crate::i18n::args(&[("moeda", peg.clone().into())]),
            ));
            for k in [
                "stable_max_deviation",
                "stable_depeg_days",
                "stable_mean_deviation",
            ] {
                ms.insert(k.into(), why.clone());
            }
        }
    }
    let data_date = [
        latest(f, keys::MCAP).map(|x| x.end),
        latest(f, keys::MCAP_LLAMA).map(|x| x.end),
        i.prices.last().map(|p| p.0),
        usd_hist.last().map(|p| p.0),
    ]
    .into_iter()
    .flatten()
    .max();
    if i.prices.len() >= MIN_CLOSES_12M && i.price_currency != "USD" {
        notes.push(crate::i18n::t_args(
            "nucleo-score-cripto-nota-cambio",
            &crate::i18n::args(&[("moeda", i.price_currency.to_string().into())]),
        ));
    }
    CryptoOutput {
        metrics: ms,
        data_date,
        notes,
    }
}

/// "cotações do app em <moeda>".
fn app_quotes(currency: &str) -> String {
    crate::i18n::t_args(
        "nucleo-score-cripto-fonte-cotacoes-app",
        &crate::i18n::args(&[("moeda", currency.to_string().into())]),
    )
}

/// Lastro declarado, no idioma da instalação.
pub fn backing_label(m: &str) -> String {
    match m {
        "fiat-backed" => crate::i18n::t("nucleo-score-cripto-lastro-fiat"),
        "crypto-backed" => crate::i18n::t("nucleo-score-cripto-lastro-cripto"),
        "algorithmic" => crate::i18n::t("nucleo-score-cripto-lastro-algoritmico"),
        x => x.to_string(),
    }
}

fn flag(m: &Methodology, key: &str, detail: String) -> Option<ScoreFlag> {
    let d = m.flags.get(key)?;
    Some(ScoreFlag {
        key: key.into(),
        label: m.flag_label(key),
        severity: d.severity,
        cap: d.cap,
        penalty: d.penalty,
        detail,
    })
}

pub fn crypto_flags(
    m: &Methodology,
    stable: bool,
    mechanism: Option<&str>,
    ms: &Metrics,
) -> Vec<ScoreFlag> {
    let val = |k: &str| ms.get(k).and_then(|x| x.value);
    let thr = |k: &str| m.flags.get(k).and_then(|f| f.threshold);
    let mut v = vec![];
    if stable {
        if let (Some(x), Some(t)) = (val("stable_max_deviation"), thr("stable_depeg"))
            && x > t
        {
            v.extend(flag(
                m,
                "stable_depeg",
                crate::i18n::t_args(
                    "nucleo-score-cripto-flag-desvio",
                    &crate::i18n::args(&[("desvio", pct(x).into())]),
                ),
            ));
        }
        if mechanism == Some("algorithmic") {
            v.extend(flag(
                m,
                "stable_algorithmic",
                crate::i18n::t("nucleo-score-cripto-flag-algoritmico"),
            ));
        }
    } else {
        if let (Some(x), Some(t)) = (val("crypto_fdv_to_mcap"), thr("crypto_high_fdv"))
            && x > t
        {
            v.extend(flag(
                m,
                "crypto_high_fdv",
                crate::i18n::t_args(
                    "nucleo-score-cripto-flag-fdv",
                    &crate::i18n::args(&[("vezes", engine::fmt_value(Unit::Multiple, x).into())]),
                ),
            ));
        }
        if let (Some(x), Some(t)) = (val("crypto_market_cap"), thr("crypto_small_cap"))
            && x < t
        {
            v.extend(flag(
                m,
                "crypto_small_cap",
                crate::i18n::t_args(
                    "nucleo-score-cripto-flag-valor-mercado",
                    &crate::i18n::args(&[("valor", engine::fmt_value(Unit::Money, x).into())]),
                ),
            ));
        }
    }
    v
}

pub fn assess(
    conn: &Connection,
    m: &Methodology,
    d: &Data,
    today: NaiveDate,
) -> Result<AssetScore> {
    let asset: Option<Asset> = match &d.subject.asset_id {
        Some(id) => Some(crate::assets::get(conn, id)?),
        None => None,
    };
    let reg = load(conn)?;
    let id = asset
        .as_ref()
        .and_then(|a| reg.ids.get(&a.id).cloned().or_else(|| local_id(a, &reg)));
    let symbol = asset.as_ref().and_then(symbol_of);
    let stable = is_stablecoin(&reg, id.as_deref(), symbol.as_deref());
    let overrides = taxonomy::overrides(conn)?;
    let segment = d
        .subject
        .ticker
        .as_ref()
        .and_then(|t| overrides.get(t).cloned())
        .unwrap_or_else(|| if stable { "stablecoin" } else { "default" }.to_string());
    let cs = &m.class_segments["crypto"];
    let segment = if cs.segments.contains_key(&segment) {
        segment
    } else {
        cs.default_segment.clone()
    };
    let seg = &cs.segments[&segment];
    let profile = &m.profiles[&seg.profile];
    let mut s = super::assess::shell_for(
        m,
        d,
        "crypto",
        &segment,
        &seg.label,
        &seg.profile,
        Some(profile),
        today,
    );
    s.notes.push(not_comparable());
    let Some(a) = asset else {
        return Ok(super::assess::without_data_for(
            s,
            &crate::i18n::t("nucleo-score-cripto-dado-criptoativo"),
            &crate::i18n::t("nucleo-score-cripto-so-cadastrados"),
        ));
    };
    let facts = match &id {
        Some(i) => store::load_facts(conn, &issuer(i))?,
        None => vec![],
    };
    let prices: Vec<Point> = crate::research::data::price_series(
        conn,
        &a.id,
        today - chrono::Duration::days(400),
        today,
    )?
    .into_iter()
    .map(|(dt, p)| (dt, to_f64(p)))
    .filter(|p| p.1 > 0.0)
    .collect();
    let usd_brl =
        crate::fx::rate(conn, "USD", "BRL", &fmt_date(today))?.map(|(r, rd, _, _)| (to_f64(r), rd));
    let stable_info = id.as_ref().and_then(|i| reg.stables.get(i));
    let o = crypto_metrics(&CryptoInput {
        facts: &facts,
        meta: id.as_ref().and_then(|i| reg.coins.get(i)),
        stable: stable_info,
        prices: &prices,
        price_currency: &a.currency,
        usd_brl,
        today,
    });
    let flags = crypto_flags(
        m,
        segment == "stablecoin",
        stable_info.map(|x| x.mechanism.as_str()),
        &o.metrics,
    );
    let out = engine::run(engine::Input {
        m,
        profile,
        id: None,
        sector: None,
        sector_label: None,
        segment: &segment,
        segment_label: &s.segment_label.clone(),
        metrics: &o.metrics,
        peers: &[],
        flags,
        data_date: o.data_date,
        today,
        extra_confidence: vec![],
    });
    super::assess::apply(&mut s, out, vec![], vec![], o.data_date);
    match &id {
        Some(i) => s.notes.push(crate::i18n::t_args(
            "nucleo-score-cripto-nota-id",
            &crate::i18n::args(&[("id", i.clone().into())]),
        )),
        None => s
            .notes
            .push(crate::i18n::t("nucleo-score-cripto-nota-sem-id")),
    }
    // A CoinGecko pede que dado guardado seja renovado em até 24 horas: passado disso, o score
    // diz de quando é o dado em vez de mostrá-lo como atual.
    if let Some(at) = id
        .as_ref()
        .and_then(|i| reg.coins.get(i))
        .and_then(|c| c.coingecko_at.as_deref())
        && coingecko_stale(at, chrono::Utc::now())
    {
        s.notes.push(crate::i18n::t_args(
            "nucleo-score-cripto-nota-coingecko-antiga",
            &crate::i18n::args(&[("data", br_date(at).into())]),
        ));
    }
    s.notes.extend(o.notes);
    s.notes
        .push(crate::i18n::t("nucleo-score-cripto-nota-sem-pares"));
    Ok(s)
}
