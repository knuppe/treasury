//! Coleta de cotações: Yahoo Finance (ações e FIIs da B3, BDRs, ETFs, ações/ETFs/REITs
//! dos EUA, cripto) e Tesouro Transparente (Tesouro Direto). Mesma estrutura da coleta de
//! índices: planejar (lê o banco), baixar (só rede) e gravar.
//!
//! Desdobramentos informados pelo Yahoo depois da primeira operação viram eventos
//! corporativos no histórico de alterações (autor sistema, desfazível), para que
//! quantidade e preço continuem coerentes; proventos vão para a agenda de dividendos.

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{PriceRow, cvm, tesouro, upsert_prices, yahoo};
use crate::assets::{Asset, AssetClass};
use crate::error::Result;
use crate::ids::new_id;
use crate::time::{add_months, fmt_date, now_utc, parse_date, today_local};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Yahoo,
    Tesouro,
    /// Informe diário da CVM (fundos), pelo CNPJ.
    Cvm,
    /// brapi.dev (com token): cotação do dia de papéis da B3, uma por papel por dia.
    Brapi,
    /// DefiLlama (aberta, sem chave): fechamento diário de cripto em dólar, pelo id da
    /// CoinGecko do ativo.
    Defillama,
    /// B3 COTAHIST: fechamento oficial dos papéis da B3, do que a análise de ativos já baixou
    /// (`research_quote`). Sem pedido de rede próprio.
    B3,
    /// Tiingo (chave da pessoa): ações, ETFs e REITs dos Estados Unidos.
    Tiingo,
    /// EODHD (chave da pessoa): bolsas do mundo todo.
    Eodhd,
}

/// Chaves das fontes de cotação que a própria pessoa cadastrou (nunca vão para log).
#[derive(Clone, Default)]
pub struct SeriesKeys {
    pub tiingo: Option<String>,
    pub eodhd: Option<String>,
}

impl SeriesKeys {
    pub fn load(conn: &Connection) -> Result<Self> {
        Ok(Self {
            tiingo: super::tiingo::token(conn)?,
            eodhd: super::eodhd::token(conn)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PriceTask {
    pub asset: Asset,
    pub provider: Provider,
    pub symbol: String,
    pub from: NaiveDate,
    pub first_trade: Option<NaiveDate>,
}

impl Provider {
    pub fn source(self) -> crate::sources::SourceId {
        match self {
            Provider::Yahoo => crate::sources::SourceId::Yahoo,
            Provider::Tesouro => crate::sources::SourceId::Tesouro,
            Provider::Cvm => crate::sources::SourceId::Cvm,
            Provider::Brapi => crate::sources::SourceId::Brapi,
            Provider::Defillama => crate::sources::SourceId::Defillama,
            Provider::B3 => crate::sources::SourceId::B3,
            Provider::Tiingo => crate::sources::SourceId::Tiingo,
            Provider::Eodhd => crate::sources::SourceId::Eodhd,
        }
    }

    /// Nome gravado em `price.source`, `dividend_event.source` e `corporate_action.source`
    /// pelas fontes que entregam a série inteira (preço, proventos e desdobramentos).
    fn series_source(self) -> &'static str {
        match self {
            Provider::Tiingo => "tiingo",
            Provider::Eodhd => "eodhd",
            _ => "yahoo",
        }
    }
}

/// Símbolo e provider padrão de um ativo (sobrescritos por `price_provider`/`provider_symbol`).
pub fn default_source(a: &Asset) -> Option<(Provider, String)> {
    let provider = match a.price_provider.as_deref() {
        Some("yahoo") => Some(Provider::Yahoo),
        Some("tesouro") => Some(Provider::Tesouro),
        Some("cvm") => Some(Provider::Cvm),
        Some("brapi") => Some(Provider::Brapi),
        Some("manual") | Some("none") => return None,
        _ => None,
    };
    if let Some(sym) = a.provider_symbol.clone().filter(|s| !s.trim().is_empty()) {
        return Some((provider.unwrap_or(Provider::Yahoo), sym));
    }
    // Fundos e previdência (fundo FIE do plano): cota oficial da CVM pelo CNPJ.
    if matches!(
        (provider, a.class),
        (Some(Provider::Cvm), _) | (None, AssetClass::Fund | AssetClass::Pension)
    ) {
        let cnpj = cvm::cnpj_digits(a.cnpj.as_deref().unwrap_or_default());
        return (cnpj.len() == 14).then_some((Provider::Cvm, cnpj));
    }
    // Tesouro Direto não tem ticker: identifica pelo nome/código do título.
    if matches!(
        (provider, a.class),
        (Some(Provider::Tesouro), _) | (None, AssetClass::TreasuryBr)
    ) {
        return Some((
            Provider::Tesouro,
            a.treasury_code.clone().unwrap_or(a.name.clone()),
        ));
    }
    let ticker = a.ticker.clone()?;
    if provider == Some(Provider::Brapi) {
        return is_b3(a.class).then(|| (Provider::Brapi, ticker.trim().to_uppercase()));
    }
    // Bolsa do catálogo com sufixo próprio (LSE ".L", Tóquio ".T"…).
    let suffixed = a
        .exchange
        .as_ref()
        .filter(|e| !e.yahoo_suffix.is_empty())
        .map(|e| format!("{}{}", ticker.trim().to_uppercase(), e.yahoo_suffix));
    match a.class {
        AssetClass::StockBr | AssetClass::Fii | AssetClass::EtfBr | AssetClass::Bdr => {
            Some((Provider::Yahoo, format!("{ticker}.SA")))
        }
        AssetClass::StockUs | AssetClass::EtfUs | AssetClass::Reit => Some((
            Provider::Yahoo,
            suffixed.unwrap_or_else(|| ticker.replace('.', "-")),
        )),
        AssetClass::StockIntl | AssetClass::EtfIntl => match &a.exchange {
            Some(e) if e.yahoo_suffix.is_empty() => {
                Some((Provider::Yahoo, ticker.replace('.', "-")))
            }
            Some(_) => suffixed.map(|s| (Provider::Yahoo, s)),
            None => None,
        },
        AssetClass::Crypto => Some((Provider::Yahoo, format!("{ticker}-{}", a.currency))),
        AssetClass::FixedIncome
        | AssetClass::TreasuryBr
        | AssetClass::Fund
        | AssetClass::Pension
        | AssetClass::Other => None,
    }
}

/// Cripto: a cotação vem da DefiLlama, pelo id da CoinGecko do ativo (fixado no ativo, dos
/// símbolos conhecidos ou achado pela coleta do score), salvo se a pessoa escolheu outra fonte
/// no ativo ou desligou a DefiLlama. Sem id, vale [`default_source`].
pub fn crypto_source(conn: &Connection, a: &Asset) -> Result<Option<(Provider, String)>> {
    if a.class != AssetClass::Crypto
        || !matches!(
            a.price_provider.as_deref(),
            None | Some("" | "defillama" | "coingecko")
        )
        || !crate::sources::enabled(conn, crate::sources::SourceId::Defillama)
    {
        return Ok(None);
    }
    let reg = crate::score::crypto::load(conn)?;
    Ok(reg
        .ids
        .get(&a.id)
        .cloned()
        .or_else(|| crate::score::crypto::local_id(a, &reg))
        .map(|id| (Provider::Defillama, id)))
}

/// Papéis da B3 pelo COTAHIST oficial: quando a pessoa escolhe no ativo (`b3_cotahist`) ou,
/// sem escolha, quando o Yahoo está desligado. É o fechamento do pregão anterior, sem cotação
/// durante o dia, e vai até onde a análise de ativos guarda (pouco mais de um ano).
pub fn b3_source(conn: &Connection, a: &Asset) -> Option<(Provider, String)> {
    use crate::sources::{SourceId, enabled};
    let chosen = a.price_provider.as_deref().filter(|p| !p.is_empty());
    let wanted = match chosen {
        Some("b3_cotahist") => true,
        None => !enabled(conn, SourceId::Yahoo),
        _ => false,
    };
    if !wanted || !is_b3(a.class) || !enabled(conn, SourceId::B3) {
        return None;
    }
    let ticker = a.ticker.as_deref()?.trim().to_uppercase();
    (!ticker.is_empty()).then_some((Provider::B3, ticker))
}

/// Fechamentos do COTAHIST já baixados pela análise de ativos, a partir de `from`.
fn cotahist_rows(conn: &Connection, ticker: &str, from: NaiveDate) -> Result<Vec<PriceRow>> {
    let mut stmt = conn.prepare(
        "SELECT date, close, volume FROM research_quote WHERE ticker = ?1 AND date >= ?2 ORDER BY date",
    )?;
    let rows: Vec<(String, String, Option<String>)> = stmt
        .query_map(params![ticker, fmt_date(from)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (date, close, volume) in rows {
        let (Ok(date), Ok(close)) = (parse_date(&date), close.parse::<Decimal>()) else {
            continue;
        };
        if close > Decimal::ZERO {
            out.push(PriceRow {
                date,
                open: None,
                high: None,
                low: None,
                close,
                adj_close: None,
                volume: volume.and_then(|v| v.parse().ok()),
            });
        }
    }
    Ok(out)
}

/// Fontes com a chave da própria pessoa (Tiingo e EODHD): valem quando ela as escolheu no
/// ativo ou, sem escolha, quando há chave cadastrada e a fonte está ligada. Papéis dos EUA
/// preferem a Tiingo; as outras bolsas (e os dos EUA sem Tiingo) vão para a EODHD. Papéis da B3
/// só vão para a EODHD se a pessoa escolher: lá a referência é a própria B3.
pub fn keyed_source(conn: &Connection, a: &Asset) -> Result<Option<(Provider, String)>> {
    use crate::sources::{SourceId, enabled};
    let chosen = a.price_provider.as_deref().filter(|p| !p.is_empty());
    if !matches!(chosen, None | Some("tiingo" | "eodhd")) {
        return Ok(None);
    }
    let Some(ticker) = a.ticker.as_deref().filter(|t| !t.trim().is_empty()) else {
        return Ok(None);
    };
    let us_listed = matches!(
        a.class,
        AssetClass::StockUs | AssetClass::EtfUs | AssetClass::Reit
    ) && a.exchange.as_ref().is_none_or(|e| e.country == "US");
    let listed = us_listed
        || matches!(a.class, AssetClass::StockIntl | AssetClass::EtfIntl)
        || (chosen.is_some() && is_b3(a.class));
    if !listed {
        return Ok(None);
    }
    let custom = a.provider_symbol.clone().filter(|s| !s.trim().is_empty());
    let tiingo = || -> Result<bool> {
        Ok(us_listed && enabled(conn, SourceId::Tiingo) && super::tiingo::token(conn)?.is_some())
    };
    if chosen != Some("eodhd") && tiingo()? {
        let sym = custom.unwrap_or_else(|| super::tiingo::symbol(ticker));
        return Ok(Some((Provider::Tiingo, sym)));
    }
    if chosen != Some("tiingo")
        && enabled(conn, SourceId::Eodhd)
        && super::eodhd::token(conn)?.is_some()
    {
        let code = match &a.exchange {
            Some(e) => super::eodhd::exchange_code(&e.code),
            None if us_listed => Some("US"),
            None if is_b3(a.class) => Some("SA"),
            None => None,
        };
        if let Some(sym) = custom.or_else(|| code.map(|c| super::eodhd::symbol(ticker, c))) {
            return Ok(Some((Provider::Eodhd, sym)));
        }
    }
    Ok(None)
}

/// Papel negociado na B3 (a brapi só cobre esses).
pub fn is_b3(c: AssetClass) -> bool {
    matches!(
        c,
        AssetClass::StockBr | AssetClass::Fii | AssetClass::EtfBr | AssetClass::Bdr
    )
}

pub(crate) fn tracked_assets(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT a.id FROM asset a WHERE a.archived = 0 AND (
            EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id) OR EXISTS (SELECT 1 FROM watchlist_item w WHERE w.asset_id = a.id))
         ORDER BY coalesce(a.ticker, a.name)",
    )?;
    let ids = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(ids)
}

/// Ativos com cotação a coletar: os acompanhados e os planos de previdência ainda sem aporte
/// (a cota da CVM converte o histórico de pagamentos em cotas e alimenta o score).
fn collected_assets(conn: &Connection) -> Result<Vec<String>> {
    let mut ids = tracked_assets(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id FROM asset WHERE archived = 0 AND class = 'pension'
            AND NOT EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = asset.id)",
    )?;
    let plans: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for id in plans {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Ok(ids)
}

/// "moeda do Yahoo (…) difere da do ativo (…)", no idioma da instalação.
pub(crate) fn currency_mismatch(yahoo: &str, asset: &str) -> String {
    crate::i18n::t_args(
        "nucleo-ativos-erro-moeda-yahoo",
        &crate::i18n::args(&[
            ("moeda_yahoo", yahoo.to_string().into()),
            ("moeda_ativo", asset.to_string().into()),
        ]),
    )
}

/// Ativos acompanhados sem coleta automática (renda fixa na curva não precisa de cotação).
pub fn unpriced(conn: &Connection) -> Result<Vec<UnpricedAsset>> {
    let mut out = vec![];
    for id in tracked_assets(conn)? {
        let a = crate::assets::get(conn, &id)?;
        if default_source(&a).is_some() || a.class.is_curve_valued() {
            continue;
        }
        let reason = crate::i18n::t(match (a.price_provider.as_deref(), a.class) {
            (Some("manual") | Some("none"), _) => "nucleo-ativos-sem-cotacao-manual",
            (_, AssetClass::Fund) => "nucleo-ativos-sem-cotacao-fundo-sem-cnpj",
            _ if a.ticker.is_none() => "nucleo-ativos-sem-cotacao-sem-ticker",
            (_, AssetClass::StockIntl | AssetClass::EtfIntl) => {
                "nucleo-ativos-sem-cotacao-sem-bolsa"
            }
            _ => "nucleo-ativos-sem-cotacao-sem-provedor",
        });
        out.push(UnpricedAsset {
            asset_id: a.id,
            ticker: a.ticker,
            name: a.name,
            class: a.class,
            reason,
        });
    }
    Ok(out)
}

/// Marca da adesão até onde as cotas do plano já foram buscadas.
fn history_key(asset_id: &str) -> String {
    format!("pension.cvm_history_from.{asset_id}")
}

/// Data de adesão de um plano de previdência (para buscar as cotas desde ela).
fn pension_start(conn: &Connection, a: &Asset) -> Result<Option<NaiveDate>> {
    if a.class != AssetClass::Pension {
        return Ok(None);
    }
    let d: Option<String> = conn
        .query_row(
            "SELECT start_date FROM pension_plan WHERE asset_id = ?1",
            [&a.id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    d.map(|d| parse_date(&d)).transpose()
}

/// Ativos com operações ou na watchlist que têm fonte de cotação (e os planos de previdência
/// sem aporte).
pub fn plan(conn: &Connection, today: NaiveDate) -> Result<Vec<PriceTask>> {
    let mut out = vec![];
    for id in collected_assets(conn)? {
        let asset = crate::assets::get(conn, &id)?;
        let Some((provider, symbol)) = crypto_source(conn, &asset)?
            .or(keyed_source(conn, &asset)?)
            .or(b3_source(conn, &asset))
            .or(default_source(&asset))
        else {
            continue;
        };
        if !crate::sources::enabled(conn, provider.source()) {
            continue;
        }
        let first_trade: Option<String> = conn
            .query_row(
                "SELECT min(date) FROM trade WHERE asset_id = ?1",
                [&id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        let first_trade = first_trade.map(|d| parse_date(&d)).transpose()?;
        let last: Option<String> = conn
            .query_row(
                "SELECT max(date) FROM price WHERE asset_id = ?1",
                [&id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        let from = match last {
            // Refaz os últimos dias (a barra do dia pode ter sido parcial).
            Some(d) => parse_date(&d)? - chrono::Duration::days(5),
            None => first_trade
                .map(|d| d - chrono::Duration::days(10))
                .unwrap_or_else(|| add_months(today, -24)),
        };
        // Previdência: as cotas desde a adesão do plano (os aportes antigos precisam da cota
        // do dia), quando a mais antiga gravada é posterior.
        let from = match pension_start(conn, &asset)? {
            Some(start) => {
                let first: Option<String> = conn
                    .query_row(
                        "SELECT min(date) FROM price WHERE asset_id = ?1",
                        [&id],
                        |r| r.get(0),
                    )
                    .optional()?
                    .flatten();
                // Uma vez por data de adesão: o fundo pode ter começado depois dela, e o
                // arquivo da CVM de cada mês é grande.
                let done = crate::settings::get::<String>(conn, &history_key(&id))?
                    .is_some_and(|d| d == fmt_date(start));
                let gap = !done
                    && match first {
                        Some(f) => parse_date(&f)? > start + chrono::Duration::days(7),
                        None => true,
                    };
                if gap { from.min(start) } else { from }
            }
            None => from,
        };
        // Fundo ainda sem o resumo mensal do informe (score): busca uma vez os últimos 25
        // meses, para o histórico de cotas, patrimônio, cotistas e captação.
        let from =
            if provider == Provider::Cvm && !crate::score::funds::has_fund_months(conn, &symbol)? {
                from.min(add_months(today, -crate::score::funds::FUND_HISTORY_MONTHS))
            } else {
                from
            };
        out.push(PriceTask {
            asset,
            provider,
            symbol,
            from,
            first_trade,
        });
    }
    Ok(out)
}

#[derive(Debug, Clone)]
pub enum Fetched {
    Yahoo(std::result::Result<yahoo::YahooSeries, String>),
    Tesouro(std::result::Result<Vec<(NaiveDate, Decimal)>, String>),
    Cvm(std::result::Result<Vec<(NaiveDate, Decimal)>, String>),
    /// Cotação da brapi (escolhida no ativo ou reserva do Yahoo que falhou).
    Brapi(std::result::Result<crate::research::brapi::BrapiQuote, String>),
    /// Fechamentos em dólar da DefiLlama.
    Defillama(std::result::Result<Vec<PriceRow>, String>),
    /// COTAHIST: lido do próprio cofre na hora de gravar.
    Local,
    /// Série da Tiingo ou da EODHD (a fonte é a da tarefa), no mesmo formato da do Yahoo.
    Keyed(std::result::Result<yahoo::YahooSeries, String>),
}

/// O que a coleta baixou, mais a saúde das fontes de arquivo único (para avisar e
/// desligar depois de dias seguidos de falha).
pub struct FetchOut {
    pub items: Vec<(PriceTask, Fetched)>,
    pub health: Vec<(crate::sources::SourceId, std::result::Result<(), String>)>,
    /// ETag/Last-Modified do CSV do Tesouro baixado agora.
    pub tesouro_meta: Option<tesouro::CsvMeta>,
    /// Taxas do Tesouro (medianas e curva) para o score de renda fixa, se pedidas.
    pub tesouro_rates: Option<tesouro::Rates>,
    /// Resumo mensal do informe diário dos fundos (patrimônio, cotistas, captação).
    pub fund_months: cvm::Months,
}

pub fn fetch(tasks: &[PriceTask]) -> Vec<(PriceTask, Fetched)> {
    fetch_with(tasks, None, None, today_local(), &SeriesKeys::default()).items
}

/// Como [`fetch`]; `tesouro_cond` pede o CSV do Tesouro só se ele mudou e `rates_since`
/// também lê as taxas desde a data (baixando o CSV mesmo sem título do Tesouro na carteira).
pub fn fetch_with(
    tasks: &[PriceTask],
    tesouro_cond: Option<&tesouro::CsvMeta>,
    rates_since: Option<NaiveDate>,
    today: NaiveDate,
    keys: &SeriesKeys,
) -> FetchOut {
    let mut out = vec![];
    let mut health = vec![];
    let mut tesouro_meta = None;
    let mut tesouro_rates = None;
    let treasury: Vec<&PriceTask> = tasks
        .iter()
        .filter(|t| t.provider == Provider::Tesouro)
        .collect();
    let tesouro_data = if treasury.is_empty() && rates_since.is_none() {
        None
    } else {
        let since = treasury.iter().map(|t| t.from).min().unwrap_or(today);
        let r = tesouro::fetch(since, rates_since, tesouro_cond, today);
        health.push((
            crate::sources::SourceId::Tesouro,
            r.as_ref().map(|_| ()).map_err(Clone::clone),
        ));
        Some(r.map(|d| match d {
            tesouro::Download::NotModified => None,
            tesouro::Download::Fresh(p, rates, m) => {
                tesouro_meta = Some(m);
                tesouro_rates = rates;
                Some(p)
            }
        }))
    };
    let funds: Vec<&PriceTask> = tasks
        .iter()
        .filter(|t| t.provider == Provider::Cvm)
        .collect();
    let mut fund_months = cvm::Months::new();
    let mut cvm_stopped: Option<String> = None;
    let cvm_data = if funds.is_empty() {
        None
    } else {
        let since = funds.iter().map(|t| t.from).min().expect("não vazio");
        let cnpjs: Vec<String> = funds.iter().map(|t| t.symbol.clone()).collect();
        Some(cvm::fetch_full(&cnpjs, since).map(|(q, m, stopped)| {
            fund_months = m;
            cvm_stopped = stopped;
            q
        }))
    };
    // Cancelada a pedido não é falha da fonte.
    if let Some(e) = &cvm_stopped
        && !crate::tasks::cancelled()
    {
        health.push((crate::sources::SourceId::Cvm, Err(e.clone())));
    }
    crate::tasks::stage(crate::tasks::TaskStage::Prices);
    crate::tasks::set_total(Some(tasks.len() as u64));
    for t in tasks {
        // Cancelada: os ativos que faltam ficam para a próxima coleta.
        if crate::tasks::cancelled() {
            break;
        }
        crate::tasks::detail(
            t.asset
                .ticker
                .clone()
                .unwrap_or_else(|| t.asset.name.clone()),
        );
        crate::tasks::advance();
        let f = match t.provider {
            Provider::Cvm => Fetched::Cvm(match &cvm_data {
                Some(Ok(all)) => match all.get(&t.symbol) {
                    Some(v) => Ok(v.iter().filter(|(d, _)| *d >= t.from).copied().collect()),
                    // A coleta parou antes dos meses deste fundo: fica para a próxima.
                    None if cvm_stopped.is_some() => Ok(vec![]),
                    None => Err(crate::i18n::t_args(
                        "nucleo-ativos-erro-cvm-fundo-ausente",
                        &crate::i18n::args(&[("fundo", t.symbol.clone().into())]),
                    )),
                },
                Some(Err(e)) => Err(e.clone()),
                None => Ok(vec![]),
            }),
            Provider::Yahoo => Fetched::Yahoo(yahoo::fetch(&t.symbol, t.from)),
            Provider::Defillama => {
                Fetched::Defillama(super::defillama::fetch(&t.symbol, t.from, today))
            }
            Provider::B3 => Fetched::Local,
            Provider::Tiingo => Fetched::Keyed(match keys.tiingo.as_deref() {
                Some(k) => super::tiingo::fetch(&t.symbol, t.from, k),
                None => Err(crate::i18n::t("nucleo-ativos-erro-fonte-sem-chave")),
            }),
            Provider::Eodhd => Fetched::Keyed(match keys.eodhd.as_deref() {
                Some(k) => super::eodhd::fetch(&t.symbol, t.from, k),
                None => Err(crate::i18n::t("nucleo-ativos-erro-fonte-sem-chave")),
            }),
            // Preenchida depois, por [`brapi_pass`] (precisa do token e da cota).
            Provider::Brapi => Fetched::Brapi(Err(crate::i18n::t("nucleo-ativos-brapi-sem-token"))),
            Provider::Tesouro => Fetched::Tesouro(match &tesouro_data {
                Some(Ok(Some(all))) => match_treasury(all, &t.asset),
                // Arquivo igual ao último: nada novo.
                Some(Ok(None)) => Ok(vec![]),
                Some(Err(e)) => Err(e.clone()),
                None => Ok(vec![]),
            }),
        };
        out.push((t.clone(), f));
    }
    FetchOut {
        items: out,
        health,
        tesouro_meta,
        tesouro_rates,
        fund_months,
    }
}

/// Acha o título no CSV pelo tipo (`treasury_code` sem o ano) e pelo vencimento.
fn match_treasury(
    all: &tesouro::Prices,
    a: &Asset,
) -> std::result::Result<Vec<(NaiveDate, Decimal)>, String> {
    let code = a.treasury_code.clone().unwrap_or(a.name.clone());
    let title = code
        .trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ')
        .to_string();
    let maturity = a
        .fixed_income
        .as_ref()
        .and_then(|f| f.maturity_date.as_deref())
        .and_then(|d| parse_date(d).ok());
    let year: Option<i32> = code.split_whitespace().last().and_then(|w| w.parse().ok());
    let key_title = tesouro::normalize_title(&title);
    // O ano no nome é o do vencimento, menos no Renda+ (ano da aposentadoria; vence 19
    // anos depois) e no Educa+ (ano do primeiro pagamento; vence 4 anos depois).
    let maturity_year = year.map(|y| {
        if key_title.contains("renda") {
            y + 19
        } else if key_title.contains("educa") {
            y + 4
        } else {
            y
        }
    });
    let found = all.iter().find(|((t, venc), _)| {
        *t == key_title
            && (maturity.is_some_and(|m| m == *venc)
                || (maturity.is_none()
                    && maturity_year.is_some_and(|y| chrono::Datelike::year(venc) == y)))
    });
    match found {
        Some((_, v)) => Ok(v.clone()),
        None => Err(crate::i18n::t_args(
            "nucleo-ativos-erro-tesouro-titulo-ausente",
            &crate::i18n::args(&[("titulo", code.to_string().into())]),
        )),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PriceSyncResult {
    pub asset_id: String,
    pub ticker: Option<String>,
    pub source: String,
    #[ts(type = "number")]
    pub prices: u32,
    #[ts(type = "number")]
    pub dividends: u32,
    #[ts(type = "number")]
    pub splits_added: u32,
    pub last_date: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PriceSyncReport {
    pub started_at: String,
    pub finished_at: String,
    pub assets: Vec<PriceSyncResult>,
    /// Ativos da carteira ou da watchlist que ficaram de fora por não terem fonte de cotação.
    pub without_source: Vec<UnpricedAsset>,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnpricedAsset {
    pub asset_id: String,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    /// Por que não há coleta e o que fazer.
    pub reason: String,
}

pub fn apply(conn: &mut Connection, fetched: &[(PriceTask, Fetched)]) -> Result<PriceSyncReport> {
    let mut rep = PriceSyncReport {
        started_at: now_utc(),
        ok: true,
        ..Default::default()
    };
    for (t, f) in fetched {
        let mut r = PriceSyncResult {
            asset_id: t.asset.id.clone(),
            ticker: t.asset.ticker.clone(),
            source: match t.provider {
                Provider::Yahoo => format!("yahoo:{}", t.symbol),
                Provider::Tesouro => "tesouro_transparente".into(),
                Provider::Cvm => format!("cvm:{}", t.symbol),
                Provider::Brapi => format!("brapi:{}", t.symbol),
                Provider::Defillama => format!("defillama:{}", t.symbol),
                Provider::B3 => format!("b3_cotahist:{}", t.symbol),
                Provider::Tiingo => format!("tiingo:{}", t.symbol),
                Provider::Eodhd => format!("eodhd:{}", t.symbol),
            },
            prices: 0,
            dividends: 0,
            splits_added: 0,
            last_date: None,
            error: None,
        };
        match f {
            Fetched::Yahoo(Ok(s)) | Fetched::Keyed(Ok(s)) => {
                let src = t.provider.series_source();
                if s.currency.as_deref().is_some_and(|c| c != t.asset.currency) {
                    r.error = Some(if src == "yahoo" {
                        currency_mismatch(
                            s.currency.as_deref().unwrap_or_default(),
                            &t.asset.currency,
                        )
                    } else {
                        crate::i18n::t_args(
                            "nucleo-ativos-erro-moeda-fonte",
                            &crate::i18n::args(&[
                                ("moeda_fonte", s.currency.clone().unwrap_or_default().into()),
                                ("moeda_ativo", t.asset.currency.clone().into()),
                            ]),
                        )
                    });
                    rep.ok = false;
                } else {
                    r.prices = upsert_prices(conn, &t.asset.id, &s.bars, src)? as u32;
                    r.dividends = upsert_dividends(conn, &t.asset, &s.dividends, src)?;
                    r.splits_added = add_splits(conn, t, &s.splits, src)?;
                    r.last_date = s.bars.last().map(|b| fmt_date(b.date));
                }
            }
            Fetched::Tesouro(Ok(points)) | Fetched::Cvm(Ok(points)) => {
                if let Some(start) = pension_start(conn, &t.asset)?
                    && t.from <= start
                {
                    crate::settings::set(conn, &history_key(&t.asset.id), &fmt_date(start))?;
                }
                let rows: Vec<PriceRow> = points
                    .iter()
                    .filter(|(d, _)| *d >= t.from)
                    .map(|(d, pu)| PriceRow {
                        date: *d,
                        open: None,
                        high: None,
                        low: None,
                        close: *pu,
                        adj_close: None,
                        volume: None,
                    })
                    .collect();
                let src = if t.provider == Provider::Cvm {
                    "cvm"
                } else {
                    "tesouro_transparente"
                };
                r.prices = upsert_prices(conn, &t.asset.id, &rows, src)? as u32;
                r.last_date = rows.last().map(|b| fmt_date(b.date));
            }
            Fetched::Brapi(Ok(q)) => {
                if q.currency.as_deref().is_some_and(|c| c != t.asset.currency) {
                    r.error = Some(currency_mismatch(
                        q.currency.as_deref().unwrap_or_default(),
                        &t.asset.currency,
                    ));
                    rep.ok = false;
                } else if let Some(close) = Decimal::from_f64_retain(q.price) {
                    let row = PriceRow {
                        date: q.date,
                        open: None,
                        high: None,
                        low: None,
                        close: close.round_dp(6),
                        adj_close: None,
                        volume: None,
                    };
                    r.source = format!("brapi:{}", t.asset.ticker.clone().unwrap_or_default());
                    r.prices = upsert_prices(conn, &t.asset.id, &[row], "brapi")? as u32;
                    r.last_date = Some(fmt_date(q.date));
                }
            }
            Fetched::Defillama(Ok(bars)) => match usd_to(conn, &t.asset.currency, bars)? {
                Ok(rows) => {
                    r.prices = upsert_prices(conn, &t.asset.id, &rows, "defillama")? as u32;
                    r.last_date = rows.last().map(|b| fmt_date(b.date));
                }
                Err(e) => {
                    r.error = Some(e);
                    rep.ok = false;
                }
            },
            Fetched::Local => {
                let rows = cotahist_rows(conn, &t.symbol, t.from)?;
                r.prices = upsert_prices(conn, &t.asset.id, &rows, "b3_cotahist")? as u32;
                r.last_date = rows.last().map(|b| fmt_date(b.date));
            }
            Fetched::Yahoo(Err(e))
            | Fetched::Keyed(Err(e))
            | Fetched::Defillama(Err(e))
            | Fetched::Tesouro(Err(e))
            | Fetched::Cvm(Err(e))
            | Fetched::Brapi(Err(e)) => {
                r.error = Some(e.clone());
                rep.ok = false;
            }
        }
        rep.assets.push(r);
    }
    rep.without_source = unpriced(conn)?;
    // Aportes de previdência que esperavam a cota do dia viram compra de cotas.
    crate::pension::payments::fill_pending(conn)?;
    rep.finished_at = now_utc();
    Ok(rep)
}

/// Fechamentos em dólar na moeda do ativo: iguais se o ativo é em dólar; senão, convertidos
/// pelo câmbio de cada dia (o último conhecido: cripto negocia no fim de semana, o câmbio não).
/// Dia sem câmbio fica de fora; sem câmbio nenhum, o erro diz o que falta.
fn usd_to(
    conn: &Connection,
    currency: &str,
    bars: &[PriceRow],
) -> Result<std::result::Result<Vec<PriceRow>, String>> {
    if currency == "USD" {
        return Ok(Ok(bars.to_vec()));
    }
    let mut out = vec![];
    for b in bars {
        if let Some((rate, ..)) = crate::fx::rate(conn, "USD", currency, &fmt_date(b.date))? {
            out.push(PriceRow {
                close: (b.close * rate).round_dp(8),
                ..b.clone()
            });
        }
    }
    if out.is_empty() && !bars.is_empty() {
        return Ok(Err(crate::i18n::t_args(
            "nucleo-ativos-erro-cripto-sem-cambio",
            &crate::i18n::args(&[("moeda", currency.to_string().into())]),
        )));
    }
    Ok(Ok(out))
}

fn upsert_dividends(
    conn: &Connection,
    a: &Asset,
    divs: &[yahoo::YahooDividend],
    source: &str,
) -> Result<u32> {
    let kind = if a.class == AssetClass::Fii {
        "fii_income"
    } else {
        "dividend"
    };
    let now = now_utc();
    let mut n = 0;
    for d in divs {
        n += conn.execute(
            "INSERT OR IGNORE INTO dividend_event (id, asset_id, kind, ex_date, amount_per_share, currency, estimated, source, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?8, ?7)",
            params![new_id(), a.id, kind, fmt_date(d.date), d.amount.normalize().to_string(), a.currency, now, source],
        )? as u32;
    }
    Ok(n)
}

/// Ajuste técnico: desdobramentos do Yahoo já vistos por ativo (`<asset_id>:<data>`). Um
/// desdobramento visto nunca volta sozinho: se o usuário desfizer ou apagar o evento, ele
/// fica de fora (o histórico de alterações não mexe nesta lista).
pub const SEEN_SPLITS_SETTING: &str = "market.yahoo_splits_seen";

fn seen_key(asset_id: &str, date: chrono::NaiveDate) -> String {
    format!("{asset_id}:{}", fmt_date(date))
}

/// Desdobramentos posteriores à primeira operação viram eventos corporativos registrados pelo
/// sistema, a não ser que:
/// - já exista um evento parecido (±10 dias);
/// - já tenha sido visto antes (e depois desfeito ou apagado pelo usuário);
/// - as operações já tragam o desdobramento: uma entrada sem custo (compra a 0, como as
///   importadas de agregadores e extratos de negociação) ou uma compra com a quantidade das
///   ações novas (bonificação com custo atribuído) até 10 dias do evento.
///
/// O Yahoo chama bonificação de "split" (10% vira 1,1:1). Vira bonificação quando a B3 diz
/// que é (eventos oficiais das empresas listadas) ou, em papéis da B3 sem esse dado, quando o
/// fator não é inteiro e fica abaixo de 2 (desdobramentos são 2:1, 5:1…).
fn add_splits(
    conn: &mut Connection,
    t: &PriceTask,
    splits: &[yahoo::YahooSplit],
    source: &str,
) -> Result<u32> {
    let Some(first) = t.first_trade else {
        return Ok(0);
    };
    let mut seen: Vec<String> = crate::settings::get_or(conn, SEEN_SPLITS_SETTING, Vec::new())?;
    let before = seen.len();
    // Os eventos do Yahoo que já estão no cofre contam como vistos.
    {
        let mut stmt = conn.prepare(
            "SELECT date FROM corporate_action WHERE asset_id = ?1 AND source IN ('yahoo', 'tiingo', 'eodhd')",
        )?;
        let dates: Vec<String> = stmt
            .query_map([&t.asset.id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for d in dates {
            let k = format!("{}:{d}", t.asset.id);
            if !seen.contains(&k) {
                seen.push(k);
            }
        }
    }
    let mut todo = vec![];
    for s in splits
        .iter()
        .filter(|s| s.date > first && s.factor != Decimal::ONE)
    {
        let key = seen_key(&t.asset.id, s.date);
        if seen.contains(&key) {
            continue;
        }
        let exists: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM corporate_action WHERE asset_id = ?1 AND kind IN ('split', 'reverse_split', 'bonus')
               AND abs(julianday(date) - julianday(?2)) <= 10)",
            params![t.asset.id, fmt_date(s.date)],
            |r| r.get(0),
        )?;
        let zero_cost: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM trade WHERE asset_id = ?1 AND side = 'buy'
               AND CAST(price AS REAL) = 0 AND abs(julianday(date) - julianday(?2)) <= 10)",
            params![t.asset.id, fmt_date(s.date)],
            |r| r.get(0),
        )?;
        let in_trades = zero_cost || bought_new_shares(conn, &t.asset.id, s)?;
        // Desfeito ou apagado pelo usuário (mesmo antes desta lista existir): fica de fora.
        let dismissed: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM change_item ci JOIN change_set cs ON cs.id = ci.change_set_id
               WHERE ci.table_name = 'corporate_action'
                 AND ((ci.op = 'insert' AND cs.status = 'undone' AND json_extract(ci.after, '$.asset_id') = ?1
                       AND abs(julianday(json_extract(ci.after, '$.date')) - julianday(?2)) <= 10)
                   OR (ci.op = 'delete' AND cs.status = 'applied' AND json_extract(ci.before, '$.asset_id') = ?1
                       AND abs(julianday(json_extract(ci.before, '$.date')) - julianday(?2)) <= 10)))",
            params![t.asset.id, fmt_date(s.date)],
            |r| r.get(0),
        )?;
        seen.push(key);
        if !exists && !in_trades && !dismissed {
            todo.push(s.clone());
        }
    }
    if seen.len() != before {
        crate::settings::set(conn, SEEN_SPLITS_SETTING, &seen)?;
    }
    if todo.is_empty() {
        return Ok(0);
    }
    let label = t.asset.ticker.clone().unwrap_or(t.asset.name.clone());
    let n = todo.len() as u32;
    crate::changes::record(
        conn,
        &crate::changes::Author::system(source),
        &if source == "yahoo" {
            crate::i18n::t_args(
                "nucleo-ativos-motivo-desdobramento-yahoo",
                &crate::i18n::args(&[("ativo", label.into())]),
            )
        } else {
            crate::i18n::t_args(
                "nucleo-ativos-motivo-desdobramento-coleta",
                &crate::i18n::args(&[("ativo", label.into())]),
            )
        },
        |tx| {
            for s in &todo {
                let bonus = is_bonus(tx, t, s)?;
                let (kind, factor, notes) = if bonus {
                    (
                        "bonus",
                        s.factor - Decimal::ONE,
                        crate::i18n::t("nucleo-ativos-nota-bonificacao-yahoo"),
                    )
                } else if s.factor > Decimal::ONE {
                    (
                        "split",
                        s.factor,
                        crate::i18n::t("nucleo-ativos-nota-evento-automatico"),
                    )
                } else {
                    (
                        "reverse_split",
                        s.factor,
                        crate::i18n::t("nucleo-ativos-nota-evento-automatico"),
                    )
                };
                let now = now_utc();
                tx.execute(
                "INSERT INTO corporate_action (id, asset_id, date, kind, factor, notes, source, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?8, ?7, ?7)",
                params![new_id(), t.asset.id, fmt_date(s.date), kind, factor.normalize().to_string(), notes, now, source],
            )?;
            }
            Ok(())
        },
    )?;
    Ok(n)
}

/// Bonificação (e não desdobramento): a B3 registra bonificação até 10 dias da data, ou, num
/// papel da B3 sem o evento oficial baixado, o fator não é inteiro e fica abaixo de 2.
fn is_bonus(conn: &Connection, t: &PriceTask, s: &yahoo::YahooSplit) -> Result<bool> {
    if s.factor <= Decimal::ONE {
        return Ok(false);
    }
    let b3 = t.symbol.ends_with(".SA");
    if !b3 {
        return Ok(false);
    }
    let ticker = t.asset.ticker.clone().unwrap_or_default().to_uppercase();
    let official: Vec<String> = conn
        .prepare_cached(
            "SELECT kind FROM research_stock_event WHERE ticker = ?1
               AND abs(julianday(ex_date) - julianday(?2)) <= 10",
        )?
        .query_map(params![ticker, fmt_date(s.date)], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    if !official.is_empty() {
        return Ok(official.iter().any(|k| k == "bonus"));
    }
    Ok(!s.factor.fract().is_zero() && s.factor < Decimal::TWO)
}

/// As operações já trazem as ações novas como compra com custo (bonificação importada de
/// agregador ou extrato): compra até 10 dias do evento com a quantidade que o fator dá sobre
/// a posição da véspera (tolerância de 1 ação ou 1%).
fn bought_new_shares(conn: &Connection, asset_id: &str, s: &yahoo::YahooSplit) -> Result<bool> {
    if s.factor <= Decimal::ONE {
        return Ok(false);
    }
    let eve = fmt_date(s.date - chrono::Duration::days(1));
    let held = crate::portfolio::engine::quantity_held(conn, asset_id, None, &eve)?;
    let extra = held * (s.factor - Decimal::ONE);
    if extra <= Decimal::ZERO {
        return Ok(false);
    }
    let tol = (extra / Decimal::from(100)).max(Decimal::ONE);
    let qtys: Vec<String> = conn
        .prepare_cached(
            "SELECT quantity FROM trade WHERE asset_id = ?1 AND side = 'buy'
               AND abs(julianday(date) - julianday(?2)) <= 10",
        )?
        .query_map(params![asset_id, fmt_date(s.date)], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(qtys
        .iter()
        .filter_map(|q| q.parse::<Decimal>().ok())
        .any(|q| (q - extra).abs() <= tol))
}

/// Token, cota e o que já foi buscado hoje, para a passada da brapi.
#[derive(Debug, Clone, Default)]
pub struct BrapiCtx {
    pub token: Option<String>,
    pub budget: i64,
    /// Papéis com cotação da brapi já buscada hoje.
    pub done_today: std::collections::HashSet<String>,
}

impl BrapiCtx {
    pub fn load(conn: &Connection, today: NaiveDate) -> Result<Self> {
        let on = crate::sources::enabled(conn, crate::sources::SourceId::Brapi);
        let token = if on {
            crate::research::brapi::token(conn)?
        } else {
            None
        };
        // `fetched_at` é UTC: "hoje" começa à meia-noite local, convertida para UTC (comparar
        // só a data UTC fazia o mesmo papel voltar à brapi à noite, depois das 21h no Brasil).
        let since = today
            .and_hms_opt(0, 0, 0)
            .and_then(|n| n.and_local_timezone(chrono::Local).earliest())
            .map(|d| {
                d.with_timezone(&chrono::Utc)
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            })
            .unwrap_or_else(|| fmt_date(today));
        let mut stmt = conn
            .prepare("SELECT key FROM research_fetch_log WHERE source = ?1 AND fetched_at >= ?2")?;
        let done_today = stmt
            .query_map(
                params![crate::research::brapi::QUOTE_LOG_SOURCE, since],
                |r| r.get(0),
            )?
            .collect::<rusqlite::Result<_>>()?;
        Ok(Self {
            token,
            budget: crate::research::brapi::budget(conn)?,
            done_today,
        })
    }
}

/// Cotações da brapi: os ativos que a escolheram e, como reserva, os papéis da B3 cujo Yahoo
/// falhou. No máximo uma por papel por dia e dentro da cota. Devolve os papéis buscados.
pub fn brapi_pass(
    items: &mut [(PriceTask, Fetched)],
    ctx: &mut BrapiCtx,
    f: &dyn crate::research::net::Fetcher,
) -> Vec<String> {
    let mut fetched = vec![];
    for (t, out) in items.iter_mut() {
        let chosen = t.provider == Provider::Brapi;
        let fallback = t.provider == Provider::Yahoo
            && is_b3(t.asset.class)
            && matches!(out, Fetched::Yahoo(Err(_)));
        if !chosen && !fallback {
            continue;
        }
        let Some(ticker) = t.asset.ticker.as_deref().map(|x| x.trim().to_uppercase()) else {
            continue;
        };
        let Some(token) = ctx.token.as_deref() else {
            continue;
        };
        if ctx.done_today.contains(&ticker) {
            if chosen {
                *out = Fetched::Brapi(Err(crate::i18n::t("nucleo-ativos-brapi-ja-hoje")));
            }
            continue;
        }
        if ctx.budget <= 0 {
            if chosen {
                *out = Fetched::Brapi(Err(crate::i18n::t("nucleo-ativos-brapi-limite")));
            }
            continue;
        }
        let auth = format!("Bearer {}", token.trim());
        let res = match f.get_with(
            &crate::research::brapi::quote_v2_url(&ticker),
            &[("Authorization", auth.as_str())],
            "",
        ) {
            Ok(crate::research::net::Fetched::Body { bytes, .. }) => {
                match crate::research::brapi::parse_quote_v2(&bytes) {
                    Ok(Some(q)) => Ok(q),
                    Ok(None) => Err(crate::i18n::t("nucleo-pesquisa-erro-brapi-sem-resultados")),
                    Err(e) => Err(e),
                }
            }
            Ok(_) => Err(crate::i18n::t("nucleo-pesquisa-erro-brapi-sem-resultados")),
            Err(e) => Err(e),
        };
        ctx.budget -= 1;
        ctx.done_today.insert(ticker.clone());
        fetched.push(ticker);
        // Reserva que também falhou: fica o erro do Yahoo (a fonte do ativo).
        if res.is_ok() || chosen {
            *out = Fetched::Brapi(res);
        }
    }
    fetched
}

pub fn run(api: &crate::Api) -> Result<PriceSyncReport> {
    crate::tasks::stage(crate::tasks::TaskStage::Prices);
    let today = today_local();
    let (tasks, cond, rates_since, brapi, keys) = api.with_session(|s, _| {
        let tasks = plan(s.db.conn(), today)?;
        // Download condicional só quando todo título já tem histórico gravado.
        let mut all_priced = true;
        for t in tasks.iter().filter(|t| t.provider == Provider::Tesouro) {
            let has: bool = s.db.conn().query_row(
                "SELECT EXISTS (SELECT 1 FROM price WHERE asset_id = ?1)",
                [&t.asset.id],
                |r| r.get(0),
            )?;
            all_priced &= has;
        }
        // Taxas para o score de renda fixa (sem nenhuma gravada, baixa o CSV inteiro).
        let rates = crate::score::fixed_income::rates_needed(s.db.conn(), today)?;
        let cond: Option<tesouro::CsvMeta> = if all_priced && !rates.is_some_and(|r| r.force) {
            crate::settings::get(s.db.conn(), tesouro::META_SETTING)?
        } else {
            None
        };
        let brapi = BrapiCtx::load(s.db.conn(), today)?;
        let keys = SeriesKeys::load(s.db.conn())?;
        Ok((tasks, cond, rates.map(|r| r.since), brapi, keys))
    })?;
    let mut out = fetch_with(&tasks, cond.as_ref(), rates_since, today, &keys);
    let mut brapi = brapi;
    let asked = brapi_pass(
        &mut out.items,
        &mut brapi,
        &crate::research::net::HttpFetcher::new(),
    );
    api.with_session(|s, _| {
        let rep = apply(s.db.conn_mut(), &out.items)?;
        record_brapi(s.db.conn(), &asked)?;
        if let Some(r) = &out.tesouro_rates {
            tesouro::store_rates(s.db.conn(), r, &now_utc())?;
        }
        crate::score::funds::store_fund_months(s.db.conn(), &out.fund_months)?;
        for (id, r) in &out.health {
            crate::sources::record_result(s.db.conn(), *id, r, today)?;
        }
        if let Some(m) = &out.tesouro_meta {
            crate::settings::set(s.db.conn(), tesouro::META_SETTING, m)?;
        }
        Ok(rep)
    })
}

/// Conta os pedidos feitos à brapi e marca os papéis buscados hoje.
pub fn record_brapi(conn: &Connection, asked: &[String]) -> Result<()> {
    crate::research::brapi::count(conn, asked.len() as i64)?;
    let now = now_utc();
    for t in asked {
        conn.execute(
            "INSERT INTO research_fetch_log (source, key, fetched_at, status) VALUES (?1, ?2, ?3, 'ok')
             ON CONFLICT (source, key) DO UPDATE SET fetched_at = excluded.fetched_at, status = 'ok'",
            params![crate::research::brapi::QUOTE_LOG_SOURCE, t, now],
        )?;
    }
    Ok(())
}

/// Índices + câmbio + cotações, numa coleta só.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataSyncReport {
    pub indices: crate::indices::sync::SyncReport,
    pub prices: PriceSyncReport,
    pub ok: bool,
}

/// Coleta pedida pelo usuário (botão Atualizar): aparece como tarefa em segundo plano.
pub fn run_all(api: &crate::Api) -> Result<DataSyncReport> {
    use crate::tasks::{self, TaskKind, TaskOrigin, TaskStage};
    tasks::run(TaskKind::MarketSync, TaskOrigin::User, true, || {
        tasks::stage(TaskStage::Indices);
        let indices = crate::indices::sync::run(api)?;
        tasks::check()?;
        let prices = run(api)?;
        tasks::check()?;
        // Lista de bancos do Banco Central: uma vez por semana.
        crate::household::banks::refresh_if_due(api);
        Ok(DataSyncReport {
            ok: indices.ok && prices.ok,
            indices,
            prices,
        })
    })
}

/// Rotina do app: coleta tudo se a última coleta tiver mais de `min_hours`.
pub fn run_all_if_due(api: &crate::Api, min_hours: i64) -> Result<Option<DataSyncReport>> {
    if !api.is_unlocked() {
        return Ok(None);
    }
    let key = crate::indices::sync::LAST_SYNC_SETTING;
    let last: Option<String> = api.with_session(|s, _| crate::settings::get(s.db.conn(), key))?;
    let due = match last
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
    {
        Some(t) => (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() >= min_hours,
        None => true,
    };
    // Cancelada há pouco: espera antes de recomeçar sozinha (docs/background-tasks.md).
    if !due || crate::tasks::snoozed(crate::tasks::TaskKind::MarketSync) {
        return Ok(None);
    }
    use crate::tasks::{self, TaskKind, TaskOrigin, TaskStage};
    let report = tasks::run(TaskKind::MarketSync, TaskOrigin::Auto, true, || {
        let report = run_all(api)?;
        api.with_session(|s, _| {
            crate::settings::set(s.db.conn(), key, &report.prices.finished_at)
        })?;
        // Contratos indexados (inclusive pós-fixados) passam a usar os índices recém-coletados.
        if let Err(e) = api.with_session(|s, _| crate::debt::refresh_indexed(s.db.conn_mut())) {
            tracing::warn!("recalcular contratos indexados: {e}");
        }
        // Proventos anunciados de ações e FIIs (Fundamentus, com cache de um dia).
        tasks::stage(TaskStage::Income);
        match crate::research::fundamentus::sync_income(api, &Default::default()) {
            Ok(r) if !r.errors.is_empty() => {
                tracing::warn!("proventos do Fundamentus: {:?}", r.errors)
            }
            Err(e) => tracing::warn!("proventos do Fundamentus: {e}"),
            _ => {}
        }
        Ok(report)
    });
    // O que veio antes do cancelamento já está gravado: as telas abertas se atualizam.
    crate::events::emit_system("market.sync");
    report.map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::{Author, record};
    use crate::storage::Db;
    use crate::vault::KDF_TEST;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    /// Cripto vem da DefiLlama, em dólar, e vira reais pelo câmbio de cada dia.
    #[test]
    fn crypto_prices_come_from_defillama_and_are_converted() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            for (id, t, cur, provider) in [
                ("c1", "BTC", "BRL", None),
                ("c2", "ETH", "USD", None),
                ("c3", "BTC", "USD", Some("yahoo")),
                ("c4", "MOEDAINVENTADA", "USD", None),
            ] {
                tx.execute(
                    "INSERT INTO asset (id, ticker, name, class, currency, price_provider, created_at, updated_at)
                     VALUES (?1, ?2, ?2, 'crypto', ?3, ?4, 'x', 'x')",
                    params![id, t, cur, provider],
                )?;
                tx.execute(
                    "INSERT INTO watchlist_item (id, asset_id, created_at, updated_at) VALUES (?1, ?1, 'x', 'x')",
                    [id],
                )?;
            }
            tx.execute(
                "INSERT INTO fx_rate (id, base, quote, date, rate, source, fetched_at, created_at, updated_at)
                 VALUES ('f1', 'USD', 'BRL', '2026-10-09', '5.50', 'bcb_ptax_venda', 'x', 'x', 'x')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        let p = plan(db.conn(), d("2026-10-14")).unwrap();
        let of = |id: &str| {
            let t = p.iter().find(|t| t.asset.id == id).unwrap();
            (t.provider, t.symbol.clone())
        };
        assert_eq!(of("c1"), (Provider::Defillama, "bitcoin".into()));
        assert_eq!(of("c2"), (Provider::Defillama, "ethereum".into()));
        // Quem escolheu o Yahoo no ativo continua nele; sem id conhecido, também.
        assert_eq!(of("c3"), (Provider::Yahoo, "BTC-USD".into()));
        assert_eq!(of("c4").0, Provider::Yahoo);
        // Com a DefiLlama desligada, volta a valer a fonte de antes.
        crate::sources::set_enabled(db.conn(), crate::sources::SourceId::Defillama, false).unwrap();
        assert!(
            plan(db.conn(), d("2026-10-14"))
                .unwrap()
                .iter()
                .all(|t| t.provider == Provider::Yahoo)
        );

        let bar = |date: &str, close| PriceRow {
            date: d(date),
            open: None,
            high: None,
            low: None,
            close,
            adj_close: None,
            volume: None,
        };
        // Sábado e domingo usam o câmbio de sexta; antes do primeiro câmbio, o dia fica de fora.
        let bars = vec![
            bar("2026-10-08", dec!(60000)),
            bar("2026-10-10", dec!(62000)),
            bar("2026-10-11", dec!(64000)),
        ];
        let fetched: Vec<(PriceTask, Fetched)> = p
            .iter()
            .filter(|t| t.provider == Provider::Defillama)
            .map(|t| (t.clone(), Fetched::Defillama(Ok(bars.clone()))))
            .collect();
        let rep = apply(db.conn_mut(), &fetched).unwrap();
        assert!(rep.ok);
        let prices = |id: &str| -> Vec<(String, String, String)> {
            db.conn()
                .prepare("SELECT date, close, source FROM price WHERE asset_id = ?1 ORDER BY date")
                .unwrap()
                .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        };
        assert_eq!(
            prices("c1"),
            vec![
                ("2026-10-10".into(), "341000".into(), "defillama".into()),
                ("2026-10-11".into(), "352000".into(), "defillama".into()),
            ]
        );
        assert_eq!(prices("c2").len(), 3);
        assert_eq!(prices("c2")[0].1, "60000");
    }

    /// Com o Yahoo desligado (ou por escolha no ativo), papéis da B3 usam o COTAHIST já baixado.
    #[test]
    fn b3_assets_fall_back_to_cotahist_without_yahoo() {
        use crate::sources::{SourceId, set_enabled};
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            for (id, t, class, provider) in [
                ("b1", "EXEN3", "stock_br", None),
                ("b2", "EXFI11", "fii", Some("b3_cotahist")),
                ("b3", "AAPL", "stock_us", None),
            ] {
                tx.execute(
                    "INSERT INTO asset (id, ticker, name, class, currency, price_provider, created_at, updated_at)
                     VALUES (?1, ?2, ?2, ?3, 'BRL', ?4, 'x', 'x')",
                    params![id, t, class, provider],
                )?;
                tx.execute(
                    "INSERT INTO watchlist_item (id, asset_id, created_at, updated_at) VALUES (?1, ?1, 'x', 'x')",
                    [id],
                )?;
            }
            Ok(())
        })
        .unwrap();
        db.conn()
            .execute_batch(
                "INSERT INTO research_quote (ticker, date, close, volume, trades, source, fetched_at) VALUES
                   ('EXEN3', '2026-10-12', '10.50', '1000', 10, 'b3_cotahist', 'x'),
                   ('EXEN3', '2026-10-13', '10.80', '1200', 12, 'b3_cotahist', 'x'),
                   ('EXFI11', '2026-10-13', '99.9', '500', 5, 'b3_cotahist', 'x');",
            )
            .unwrap();
        let today = d("2026-10-14");
        let p = plan(db.conn(), today).unwrap();
        let of =
            |p: &[PriceTask], id: &str| p.iter().find(|t| t.asset.id == id).map(|t| t.provider);
        // Com o Yahoo ligado só muda quem escolheu o COTAHIST no ativo.
        assert_eq!(of(&p, "b1"), Some(Provider::Yahoo));
        assert_eq!(of(&p, "b2"), Some(Provider::B3));
        set_enabled(db.conn(), SourceId::Yahoo, false).unwrap();
        let p = plan(db.conn(), today).unwrap();
        assert_eq!(of(&p, "b1"), Some(Provider::B3));
        // Papel de fora sem chave e sem Yahoo: fica sem fonte.
        assert_eq!(of(&p, "b3"), None);
        let fetched = fetch_with(&p, None, None, today, &SeriesKeys::default()).items;
        let rep = apply(db.conn_mut(), &fetched).unwrap();
        assert!(rep.ok);
        let rows: Vec<(String, String, String)> = db
            .conn()
            .prepare("SELECT asset_id, date, close FROM price WHERE source = 'b3_cotahist' ORDER BY asset_id, date")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(
            rows,
            vec![
                ("b1".into(), "2026-10-12".into(), "10.5".into()),
                ("b1".into(), "2026-10-13".into(), "10.8".into()),
                ("b2".into(), "2026-10-13".into(), "99.9".into()),
            ]
        );
    }

    /// Com a chave da pessoa, papéis dos EUA vão para a Tiingo e os de outras bolsas para a
    /// EODHD; sem chave (ou com a fonte desligada), fica a fonte de antes.
    #[test]
    fn keyed_sources_take_over_when_the_person_adds_a_key() {
        use crate::sources::{SourceId, set_enabled};
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            for (id, t, class, cur, market, provider) in [
                ("u1", "BRK.B", "stock_us", "USD", None, None),
                ("u2", "VOD", "stock_intl", "GBP", Some("LSE"), None),
                ("u3", "PETR4", "stock_br", "BRL", None, None),
                ("u4", "ITSA4", "stock_br", "BRL", None, Some("eodhd")),
                ("u5", "AAPL", "stock_us", "USD", None, Some("yahoo")),
                ("u6", "D05", "stock_intl", "SGD", Some("SGX"), None),
            ] {
                tx.execute(
                    "INSERT INTO asset (id, ticker, name, class, currency, market, price_provider, created_at, updated_at)
                     VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, 'x', 'x')",
                    params![id, t, class, cur, market, provider],
                )?;
                tx.execute(
                    "INSERT INTO watchlist_item (id, asset_id, created_at, updated_at) VALUES (?1, ?1, 'x', 'x')",
                    [id],
                )?;
            }
            Ok(())
        })
        .unwrap();
        let today = d("2026-10-14");
        let of = |c: &Connection| -> Vec<(String, Provider, String)> {
            let mut v: Vec<_> = plan(c, today)
                .unwrap()
                .into_iter()
                .map(|t| (t.asset.id.clone(), t.provider, t.symbol))
                .collect();
            v.sort_by(|a, b| a.0.cmp(&b.0));
            v
        };
        // Sem chave: o Yahoo, como antes (a escolha da EODHD sem chave não acha fonte melhor).
        assert!(of(db.conn()).iter().all(|(_, p, _)| *p == Provider::Yahoo));
        crate::settings::set(
            db.conn(),
            super::super::tiingo::TOKEN_SETTING,
            "chave-tiingo-de-teste",
        )
        .unwrap();
        crate::settings::set(
            db.conn(),
            super::super::eodhd::TOKEN_SETTING,
            "chave-eodhd-de-teste",
        )
        .unwrap();
        let got = of(db.conn());
        let p = |id: &str| {
            let (_, p, s) = got.iter().find(|(i, ..)| i == id).unwrap();
            (*p, s.as_str())
        };
        assert_eq!(p("u1"), (Provider::Tiingo, "BRK-B"));
        assert_eq!(p("u2"), (Provider::Eodhd, "VOD.LSE"));
        // B3: a referência continua sendo a de antes, salvo escolha da pessoa no ativo.
        assert_eq!(p("u3"), (Provider::Yahoo, "PETR4.SA"));
        assert_eq!(p("u4"), (Provider::Eodhd, "ITSA4.SA"));
        assert_eq!(p("u5"), (Provider::Yahoo, "AAPL"));
        // Bolsa sem código conhecido na EODHD: fica onde estava.
        assert_eq!(p("u6").0, Provider::Yahoo);
        // Tiingo desligada: os dos EUA vão para a EODHD.
        set_enabled(db.conn(), SourceId::Tiingo, false).unwrap();
        let got = of(db.conn());
        let (_, pr, sym) = got.iter().find(|(i, ..)| i == "u1").unwrap();
        assert_eq!((*pr, sym.as_str()), (Provider::Eodhd, "BRK-B.US"));

        // A série da fonte com chave grava preço, provento e desdobramento com o nome dela.
        set_enabled(db.conn(), SourceId::Tiingo, true).unwrap();
        let task = plan(db.conn(), today)
            .unwrap()
            .into_iter()
            .find(|t| t.asset.id == "u1")
            .unwrap();
        let series = yahoo::YahooSeries {
            currency: Some("USD".into()),
            bars: vec![PriceRow {
                date: d("2026-10-13"),
                open: None,
                high: None,
                low: None,
                close: dec!(480),
                adj_close: None,
                volume: None,
            }],
            dividends: vec![yahoo::YahooDividend {
                date: d("2026-10-01"),
                amount: dec!(0.5),
            }],
            splits: vec![],
        };
        let rep = apply(db.conn_mut(), &[(task, Fetched::Keyed(Ok(series)))]).unwrap();
        assert!(rep.ok);
        assert_eq!(rep.assets[0].source, "tiingo:BRK-B");
        let (ps, ds): (String, String) = db
            .conn()
            .query_row(
                "SELECT (SELECT source FROM price WHERE asset_id = 'u1'),
                        (SELECT source FROM dividend_event WHERE asset_id = 'u1')",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((ps.as_str(), ds.as_str()), ("tiingo", "tiingo"));
    }

    #[test]
    fn plans_sources_and_applies_yahoo_with_splits() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            tx.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at) VALUES ('xp', 'XP', 'brokerage', 'BRL', '2024-01-01', 'x', 'x')",
                [],
            )?;
            for (id, t, class, cur) in [("a1", "PETR4", "stock_br", "BRL"), ("a2", "AAPL", "stock_us", "USD"), ("a3", "BRK.B", "stock_us", "USD"), ("a4", "VERDE", "fund", "BRL")] {
                tx.execute(
                    "INSERT INTO asset (id, ticker, name, class, currency, created_at, updated_at) VALUES (?1, ?2, ?2, ?3, ?4, 'x', 'x')",
                    params![id, t, class, cur],
                )?;
            }
            tx.execute(
                "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
                 VALUES ('t1', 'a1', 'xp', '2026-01-10', 'buy', '100', '30', 'BRL', 'x', 'x')",
                [],
            )?;
            tx.execute("INSERT INTO watchlist_item (id, asset_id, created_at, updated_at) VALUES ('w', 'a3', 'x', 'x')", [])?;
            tx.execute("INSERT INTO watchlist_item (id, asset_id, created_at, updated_at) VALUES ('w4', 'a4', 'x', 'x')", [])?;
            Ok(())
        })
        .unwrap();
        let p = plan(db.conn(), d("2026-10-14")).unwrap();
        assert_eq!(p.len(), 2, "só ativos com operação ou na watchlist");
        let petr = p.iter().find(|t| t.asset.id == "a1").unwrap();
        assert_eq!(petr.symbol, "PETR4.SA");
        assert_eq!(petr.from, d("2025-12-31"));
        assert_eq!(
            p.iter().find(|t| t.asset.id == "a3").unwrap().symbol,
            "BRK-B"
        );

        let series = yahoo::YahooSeries {
            currency: Some("BRL".into()),
            bars: vec![PriceRow {
                date: d("2026-10-13"),
                open: None,
                high: None,
                low: None,
                close: dec!(19),
                adj_close: None,
                volume: None,
            }],
            dividends: vec![yahoo::YahooDividend {
                date: d("2026-08-20"),
                amount: dec!(0.5),
            }],
            splits: vec![yahoo::YahooSplit {
                date: d("2026-06-01"),
                factor: dec!(2),
            }],
        };
        let rep = apply(
            db.conn_mut(),
            &[(petr.clone(), Fetched::Yahoo(Ok(series.clone())))],
        )
        .unwrap();
        assert!(rep.ok);
        assert_eq!(
            rep.without_source.len(),
            1,
            "o fundo fica de fora, com o motivo"
        );
        assert_eq!(rep.without_source[0].asset_id, "a4");
        assert!(rep.without_source[0].reason.contains("CNPJ"));
        assert_eq!(
            (
                rep.assets[0].prices,
                rep.assets[0].dividends,
                rep.assets[0].splits_added
            ),
            (1, 1, 1)
        );
        // Reaplicar não duplica provento nem desdobramento.
        let rep2 = apply(
            db.conn_mut(),
            &[(petr.clone(), Fetched::Yahoo(Ok(series.clone())))],
        )
        .unwrap();
        assert_eq!(
            (rep2.assets[0].dividends, rep2.assets[0].splits_added),
            (0, 0)
        );
        // Desfeito pelo usuário, o desdobramento não volta na coleta seguinte.
        let cs: String = db
            .conn()
            .query_row(
                "SELECT id FROM change_set WHERE reason LIKE 'Desdobramento informado pelo Yahoo%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        crate::changes::undo(db.conn_mut(), &cs).unwrap();
        // Mesmo sem a lista de vistos (cofre antigo), o histórico diz que foi desfeito.
        crate::settings::set(db.conn(), SEEN_SPLITS_SETTING, &Vec::<String>::new()).unwrap();
        let again = apply(
            db.conn_mut(),
            &[(petr.clone(), Fetched::Yahoo(Ok(series.clone())))],
        )
        .unwrap();
        assert_eq!(again.assets[0].splits_added, 0, "desfeito não volta");
        let book = crate::portfolio::engine::load_book(db.conn(), "a1", None).unwrap();
        let st = crate::portfolio::engine::replay(db.conn(), &book, d("2026-10-14")).unwrap();
        assert_eq!(st.quantity(), dec!(100));
        // Volta a valer para o teste abaixo (refazer).
        crate::changes::redo(db.conn_mut(), &cs).unwrap();
        // A posição dobrou e a cotação nova vale para a nova quantidade.
        let book = crate::portfolio::engine::load_book(db.conn(), "a1", None).unwrap();
        let st = crate::portfolio::engine::replay(db.conn(), &book, d("2026-10-14")).unwrap();
        assert_eq!(st.quantity(), dec!(200));
        let hist = crate::changes::list(db.conn(), &Default::default()).unwrap();
        assert!(
            hist.iter()
                .any(|c| c.reason.contains("Desdobramento informado pelo Yahoo"))
        );
        // Erro de rede vira item do relatório, sem falhar a coleta.
        let rep3 = apply(
            db.conn_mut(),
            &[(petr.clone(), Fetched::Yahoo(Err("timeout".into())))],
        )
        .unwrap();
        assert!(!rep3.ok);
    }

    /// Operações que já trazem o desdobramento (entrada a custo zero, como nas importações
    /// de agregadores) não recebem o evento do Yahoo por cima: não conta duas vezes.
    #[test]
    fn split_already_in_trades_is_not_added_again() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            tx.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at) VALUES ('nu', 'Nu', 'brokerage', 'BRL', '2020-01-01', 'x', 'x')",
                [],
            )?;
            tx.execute(
                "INSERT INTO asset (id, ticker, name, class, currency, created_at, updated_at) VALUES ('b1', 'TSLA34', 'Tesla BDR', 'bdr', 'BRL', 'x', 'x')",
                [],
            )?;
            for (id, date, q, p) in [("t1", "2022-06-02", "100", "115"), ("t2", "2022-08-24", "200", "0")] {
                tx.execute(
                    "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
                     VALUES (?1, 'b1', 'nu', ?2, 'buy', ?3, ?4, 'BRL', 'x', 'x')",
                    params![id, date, q, p],
                )?;
            }
            Ok(())
        })
        .unwrap();
        let task = plan(db.conn(), d("2026-10-14"))
            .unwrap()
            .into_iter()
            .find(|t| t.asset.id == "b1")
            .unwrap();
        let series = yahoo::YahooSeries {
            currency: Some("BRL".into()),
            bars: vec![],
            dividends: vec![],
            splits: vec![yahoo::YahooSplit {
                date: d("2022-08-25"),
                factor: dec!(3),
            }],
        };
        let rep = apply(db.conn_mut(), &[(task, Fetched::Yahoo(Ok(series)))]).unwrap();
        assert_eq!(rep.assets[0].splits_added, 0);
        let book = crate::portfolio::engine::load_book(db.conn(), "b1", None).unwrap();
        let st = crate::portfolio::engine::replay(db.conn(), &book, d("2026-10-14")).unwrap();
        assert_eq!(
            st.quantity(),
            dec!(300),
            "100 + 200 do desdobramento, sem ×3 por cima"
        );
    }

    /// Bonificação: o Yahoo informa como "split" 1,1; vira bonificação de 10%, e não entra
    /// quando as operações já trazem as ações novas como compra com custo atribuído.
    #[test]
    fn yahoo_bonus_is_a_bonus_and_is_not_added_over_an_imported_buy() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            tx.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at) VALUES ('xp', 'XP', 'brokerage', 'BRL', '2020-01-01', 'x', 'x')",
                [],
            )?;
            for (id, tk) in [("a1", "ITSA4"), ("a2", "BBDC4")] {
                tx.execute(
                    "INSERT INTO asset (id, ticker, name, class, currency, created_at, updated_at) VALUES (?1, ?2, ?2, 'stock_br', 'BRL', 'x', 'x')",
                    params![id, tk],
                )?;
            }
            for (id, asset, date, q, p) in [
                ("t1", "a1", "2024-01-02", "1000", "10"),
                // Ações bonificadas importadas do agregador, com o custo atribuído.
                ("t2", "a1", "2024-12-27", "100", "18.45"),
                ("t3", "a2", "2024-01-02", "1000", "15"),
            ] {
                tx.execute(
                    "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
                     VALUES (?1, ?2, 'xp', ?3, 'buy', ?4, ?5, 'BRL', 'x', 'x')",
                    params![id, asset, date, q, p],
                )?;
            }
            Ok(())
        })
        .unwrap();
        let tasks = plan(db.conn(), d("2026-10-14")).unwrap();
        let series = || yahoo::YahooSeries {
            currency: Some("BRL".into()),
            bars: vec![],
            dividends: vec![],
            splits: vec![yahoo::YahooSplit {
                date: d("2024-12-23"),
                factor: dec!(1.1),
            }],
        };
        let fetched: Vec<_> = tasks
            .into_iter()
            .filter(|t| t.asset.id == "a1" || t.asset.id == "a2")
            .map(|t| (t, Fetched::Yahoo(Ok(series()))))
            .collect();
        assert_eq!(fetched.len(), 2);
        apply(db.conn_mut(), &fetched).unwrap();
        let actions = |asset: &str| -> Vec<(String, String)> {
            db.conn()
                .prepare("SELECT kind, factor FROM corporate_action WHERE asset_id = ?1")
                .unwrap()
                .query_map([asset], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        };
        assert!(
            actions("a1").is_empty(),
            "a compra importada já traz a bonificação"
        );
        assert_eq!(
            actions("a2"),
            vec![("bonus".to_string(), "0.1".to_string())]
        );
        let held =
            crate::portfolio::engine::quantity_held(db.conn(), "a2", None, "2026-10-14").unwrap();
        assert_eq!(held, dec!(1100));
    }

    #[test]
    #[ignore = "usa a rede: cargo test -- --ignored live_yahoo"]
    fn live_yahoo() {
        let s = yahoo::fetch("PETR4.SA", add_months(today_local(), -1)).unwrap();
        assert!(s.bars.len() > 10);
        assert_eq!(s.currency.as_deref(), Some("BRL"));
        let b = yahoo::fetch("^BVSP", add_months(today_local(), -1)).unwrap();
        assert!(!b.bars.is_empty());
    }

    #[test]
    fn renda_and_educa_names_find_their_maturity() {
        let mut all: tesouro::Prices = std::collections::HashMap::new();
        for (t, v) in [
            ("Tesouro Renda+ Aposentadoria Extra", "2049-12-15"),
            ("Tesouro Educa+", "2030-12-15"),
            ("Tesouro IPCA+", "2035-05-15"),
        ] {
            all.insert(
                (tesouro::normalize_title(t), d(v)),
                vec![(d("2026-10-15"), dec!(100))],
            );
        }
        let asset = |code: &str| Asset {
            id: "x".into(),
            ticker: None,
            name: code.into(),
            official_name: None,
            account_id: None,
            class: AssetClass::TreasuryBr,
            class_label: String::new(),
            market: None,
            currency: "BRL".into(),
            country: None,
            isin: None,
            cnpj: None,
            treasury_code: Some(code.into()),
            sector: None,
            segment: None,
            price_provider: None,
            provider_symbol: None,
            archived: false,
            notes: None,
            fixed_income: None,
            exchange: None,
            quote_symbol: None,
        };
        assert!(match_treasury(&all, &asset("Tesouro Renda+ Aposentadoria Extra 2030")).is_ok());
        assert!(match_treasury(&all, &asset("Tesouro Educa+ 2026")).is_ok());
        assert!(match_treasury(&all, &asset("Tesouro IPCA+ 2035")).is_ok());
        assert!(match_treasury(&all, &asset("Tesouro IPCA+ 2045")).is_err());
    }

    /// Servidor fixo: devolve a mesma cotação para qualquer papel e conta os pedidos.
    struct FixedQuote(std::sync::Mutex<Vec<String>>);
    impl crate::research::net::Fetcher for FixedQuote {
        fn get(
            &self,
            url: &str,
            _c: Option<&crate::research::net::FileMeta>,
            _ua: &str,
        ) -> crate::research::net::FetchResult {
            self.0.lock().unwrap().push(url.to_string());
            Ok(crate::research::net::Fetched::Body {
                bytes: br#"{"results":[{"symbol":"X","data":{"currency":"BRL","regularMarketPrice":36.5,
                    "regularMarketTime":"2026-10-21T20:07:00.000Z"}}]}"#
                    .to_vec(),
                meta: Default::default(),
            })
        }
    }

    #[test]
    fn brapi_backs_up_yahoo_and_serves_chosen_assets_once_a_day() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            tx.execute(
                "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at) VALUES ('xp', 'XP', 'brokerage', 'BRL', '2024-01-01', 'x', 'x')",
                [],
            )?;
            for (id, t, class, provider) in [
                ("a1", "TAEE11", "stock_br", None),
                ("a2", "HGLG11", "fii", Some("brapi")),
                ("a3", "AAPL", "stock_us", None),
            ] {
                tx.execute(
                    "INSERT INTO asset (id, ticker, name, class, currency, price_provider, created_at, updated_at)
                     VALUES (?1, ?2, ?2, ?3, ?4, ?5, 'x', 'x')",
                    params![id, t, class, if class == "stock_us" { "USD" } else { "BRL" }, provider],
                )?;
                tx.execute(
                    "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, currency, created_at, updated_at)
                     VALUES (?1, ?2, 'xp', '2026-01-10', 'buy', '10', '30', ?3, 'x', 'x')",
                    params![format!("t{id}"), id, if class == "stock_us" { "USD" } else { "BRL" }],
                )?;
            }
            Ok(())
        })
        .unwrap();
        let tasks = plan(db.conn(), d("2026-10-21")).unwrap();
        let provider = |id: &str| tasks.iter().find(|t| t.asset.id == id).unwrap().provider;
        assert_eq!(provider("a1"), Provider::Yahoo);
        assert_eq!(provider("a2"), Provider::Brapi);
        // Yahoo falhou para os dois papéis com Yahoo; a brapi só cobre o da B3.
        let mut items: Vec<(PriceTask, Fetched)> = tasks
            .iter()
            .map(|t| {
                let f = match t.provider {
                    Provider::Brapi => Fetched::Brapi(Err("pendente".into())),
                    _ => Fetched::Yahoo(Err("Yahoo fora".into())),
                };
                (t.clone(), f)
            })
            .collect();
        let f = FixedQuote(Default::default());
        // Sem token: nada muda.
        let mut ctx = BrapiCtx::default();
        assert!(brapi_pass(&mut items, &mut ctx, &f).is_empty());
        let mut ctx = BrapiCtx {
            token: Some("tok".into()),
            budget: 10,
            ..Default::default()
        };
        let asked = brapi_pass(&mut items, &mut ctx, &f);
        assert_eq!(asked.len(), 2, "{asked:?}");
        assert!(f.0.lock().unwrap().iter().all(|u| !u.contains("AAPL")));
        let rep = apply(db.conn_mut(), &items).unwrap();
        let by = |id: &str| {
            rep.assets
                .iter()
                .find(|r| r.asset_id == id)
                .unwrap()
                .clone()
        };
        assert_eq!(by("a1").source, "brapi:TAEE11");
        assert_eq!(by("a1").last_date.as_deref(), Some("2026-10-21"));
        assert!(by("a3").error.is_some());
        record_brapi(db.conn(), &asked).unwrap();
        let src: String = db
            .conn()
            .query_row("SELECT source FROM price WHERE asset_id = 'a2'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(src, "brapi");
        // A contagem do mês subiu e, no mesmo dia, nenhum papel volta à brapi.
        assert_eq!(
            crate::research::brapi::state(db.conn())
                .unwrap()
                .since_check,
            2
        );
        let mut ctx = BrapiCtx::load(db.conn(), today_local()).unwrap();
        ctx.token = Some("tok".into());
        let mut again: Vec<(PriceTask, Fetched)> = tasks
            .iter()
            .map(|t| (t.clone(), Fetched::Yahoo(Err("Yahoo fora".into()))))
            .collect();
        assert!(brapi_pass(&mut again, &mut ctx, &f).is_empty());
        // Sem cota: o ativo que escolheu a brapi explica.
        let mut ctx = BrapiCtx {
            token: Some("tok".into()),
            budget: 0,
            ..Default::default()
        };
        let mut chosen: Vec<(PriceTask, Fetched)> = tasks
            .iter()
            .filter(|t| t.provider == Provider::Brapi)
            .map(|t| (t.clone(), Fetched::Brapi(Err("pendente".into()))))
            .collect();
        brapi_pass(&mut chosen, &mut ctx, &f);
        assert!(matches!(&chosen[0].1, Fetched::Brapi(Err(e)) if e.contains("limite")));
    }
}
