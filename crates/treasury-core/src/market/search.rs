//! Autocompletar de tickers: ativos já cadastrados, o universo da análise de ativos
//! (`research_security`) e a busca do Yahoo (rede, fora do lock do banco, com cache curto).
//! Cada sugestão traz o que o cadastro precisa: ticker, bolsa, classe, moeda e nome.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::exchange;
use super::yahoo::{self, SearchHit};
use crate::assets::AssetClass;
use crate::error::Result;

/// Resultados do Yahoo ficam em memória por este tempo.
pub const CACHE_TTL: Duration = Duration::from_secs(10 * 60);
const CACHE_MAX: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SuggestionOrigin {
    /// Já cadastrado no cofre.
    Asset,
    /// Universo da análise de ativos (B3/CVM, SEC).
    Research,
    /// Busca do Yahoo Finance.
    Yahoo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TickerSuggestion {
    pub origin: SuggestionOrigin,
    /// Ativo já cadastrado.
    pub asset_id: Option<String>,
    /// Ticker na bolsa (PETR4, VOD, 7203).
    pub ticker: String,
    /// Símbolo no Yahoo (PETR4.SA, VOD.L), quando conhecido.
    pub symbol: Option<String>,
    pub name: String,
    pub class: Option<AssetClass>,
    /// Código da bolsa no catálogo (B3, LSE…); `None` se a bolsa não está cadastrada.
    pub exchange: Option<String>,
    /// Nome da bolsa (do catálogo ou como o Yahoo mostra).
    pub exchange_name: Option<String>,
    pub country: Option<String>,
    pub currency: Option<String>,
    /// Ação, ETF, FII, BDR, REIT, Cripto…
    pub type_label: String,
    pub sector: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TickerSearchArgs {
    pub query: String,
    #[ts(optional)]
    pub limit: Option<u32>,
    /// Consultar o Yahoo (padrão: sim).
    #[ts(optional)]
    pub online: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TickerSearch {
    pub query: String,
    pub results: Vec<TickerSuggestion>,
    /// O Yahoo foi consultado (ou veio do cache).
    pub online: bool,
    /// Falha da busca online (os resultados locais continuam).
    pub error: Option<String>,
}

pub fn type_label(class: Option<AssetClass>, quote_type: Option<&str>) -> String {
    match class {
        Some(AssetClass::StockBr | AssetClass::StockUs | AssetClass::StockIntl) => {
            crate::i18n::t("nucleo-ativos-tipo-acao")
        }
        Some(AssetClass::EtfBr | AssetClass::EtfUs | AssetClass::EtfIntl) => "ETF".into(),
        Some(AssetClass::Fii) => "FII".into(),
        Some(AssetClass::Bdr) => "BDR".into(),
        Some(AssetClass::Reit) => "REIT".into(),
        Some(AssetClass::Crypto) => AssetClass::Crypto.label(),
        Some(
            c @ (AssetClass::FixedIncome
            | AssetClass::TreasuryBr
            | AssetClass::Fund
            | AssetClass::Pension
            | AssetClass::Other),
        ) => c.label(),
        None => match quote_type {
            Some("ETF") => "ETF".into(),
            Some("MUTUALFUND") => crate::i18n::t("nucleo-ativos-tipo-fundo"),
            Some("CRYPTOCURRENCY") => AssetClass::Crypto.label(),
            _ => crate::i18n::t("nucleo-ativos-tipo-acao"),
        },
    }
}

fn norm(q: &str) -> String {
    q.trim().to_lowercase()
}

/// Ativos cadastrados e o universo da análise que batem com o texto.
pub fn local(conn: &Connection, query: &str, limit: usize) -> Result<Vec<TickerSuggestion>> {
    let q = norm(query);
    if q.is_empty() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let assets = crate::assets::list(
        conn,
        &crate::assets::AssetFilter {
            text: Some(q.clone()),
            ..Default::default()
        },
    )?;
    for a in assets.into_iter().filter(|a| a.ticker.is_some()) {
        out.push(TickerSuggestion {
            origin: SuggestionOrigin::Asset,
            asset_id: Some(a.id.clone()),
            ticker: a.ticker.clone().unwrap_or_default(),
            symbol: a.quote_symbol.clone(),
            name: a.name.clone(),
            class: Some(a.class),
            exchange: a
                .exchange
                .as_ref()
                .map(|e| e.code.clone())
                .or(a.market.clone()),
            exchange_name: a.exchange.as_ref().map(|e| e.name.clone()),
            country: a.country.clone(),
            currency: Some(a.currency.clone()),
            type_label: type_label(Some(a.class), None),
            sector: a.sector.clone(),
        });
    }
    let mut stmt = conn.prepare(
        "SELECT s.ticker, s.class, i.name, i.sector, i.currency FROM research_security s
           JOIN research_issuer i ON i.id = s.issuer_id
          WHERE s.active = 1 AND (lower(s.ticker) LIKE ?1 || '%' OR lower(i.name) LIKE '%' || ?1 || '%')
          ORDER BY lower(s.ticker) = ?1 DESC, lower(s.ticker) LIKE ?1 || '%' DESC, length(s.ticker), s.ticker
          LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![q, limit as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let b3 = exchange::get(conn, "B3")?;
    for (ticker, class, name, sector, currency) in rows {
        if out.iter().any(|s| s.ticker.eq_ignore_ascii_case(&ticker)) {
            continue;
        }
        let class = AssetClass::parse(&class).ok();
        let us = class.is_some_and(AssetClass::is_us);
        let ex = if us { None } else { b3.as_ref() };
        out.push(TickerSuggestion {
            origin: SuggestionOrigin::Research,
            asset_id: None,
            symbol: ex
                .map(|e| exchange::yahoo_symbol(&ticker, e))
                .or_else(|| us.then(|| ticker.replace('.', "-"))),
            ticker,
            name: name.split_whitespace().collect::<Vec<_>>().join(" "),
            type_label: type_label(class, None),
            class,
            exchange: ex.map(|e| e.code.clone()),
            exchange_name: ex.map(|e| e.name.clone()),
            country: Some(if us { "US".into() } else { "BR".into() }),
            currency: Some(currency),
            sector,
        });
    }
    Ok(out)
}

/// Classe, bolsa e moeda de um resultado do Yahoo.
pub fn classify(conn: &Connection, hit: &SearchHit) -> Result<TickerSuggestion> {
    let sector = hit.sector.clone();
    if hit.quote_type == "CRYPTOCURRENCY" {
        let (base, quote) = hit
            .symbol
            .split_once('-')
            .map(|(a, b)| (a.to_string(), Some(b.to_string())))
            .unwrap_or((hit.symbol.clone(), None));
        return Ok(TickerSuggestion {
            origin: SuggestionOrigin::Yahoo,
            asset_id: None,
            ticker: base,
            symbol: Some(hit.symbol.clone()),
            name: hit.name.clone(),
            class: Some(AssetClass::Crypto),
            exchange: Some("CRYPTO".into()),
            exchange_name: hit.exchange_display.clone(),
            country: None,
            currency: quote,
            type_label: AssetClass::Crypto.label(),
            sector,
        });
    }
    let ex = exchange::for_yahoo(conn, &hit.symbol, hit.exchange.as_deref())?;
    let ticker = exchange::ticker_from_yahoo(&hit.symbol, ex.as_ref());
    let etf = hit.quote_type == "ETF";
    let class = match ex.as_ref().map(|e| e.country.as_str()) {
        Some("BR") => {
            let known: Option<String> = rusqlite::OptionalExtension::optional(conn.query_row(
                "SELECT class FROM research_security WHERE ticker = ?1",
                [&ticker],
                |r| r.get(0),
            ))?;
            match known.and_then(|c| AssetClass::parse(&c).ok()) {
                Some(c) => Some(c),
                None if etf => Some(AssetClass::EtfBr),
                None => Some(
                    crate::import::b3::rules::infer_class(Some(&ticker), &hit.name, None, true)
                        .class,
                ),
            }
        }
        Some("US") => Some(match hit.quote_type.as_str() {
            "ETF" => AssetClass::EtfUs,
            "MUTUALFUND" => AssetClass::Other,
            _ if hit
                .industry
                .as_deref()
                .is_some_and(|i| i.to_uppercase().contains("REIT")) =>
            {
                AssetClass::Reit
            }
            _ => AssetClass::StockUs,
        }),
        Some(_) if hit.quote_type == "MUTUALFUND" => Some(AssetClass::Other),
        Some(_) if etf => Some(AssetClass::EtfIntl),
        Some(_) => Some(AssetClass::StockIntl),
        None => None,
    };
    Ok(TickerSuggestion {
        origin: SuggestionOrigin::Yahoo,
        asset_id: None,
        ticker,
        symbol: Some(hit.symbol.clone()),
        name: hit.name.clone(),
        type_label: type_label(class, Some(&hit.quote_type)),
        class,
        exchange: ex.as_ref().map(|e| e.code.clone()),
        exchange_name: ex
            .as_ref()
            .map(|e| e.name.clone())
            .or(hit.exchange_display.clone()),
        country: ex.as_ref().map(|e| e.country.clone()),
        currency: ex.as_ref().map(|e| e.currency.clone()),
        sector,
    })
}

/// Junta locais e online: cadastrados primeiro, depois o universo, depois o Yahoo; o ticker
/// exato digitado sobe ao topo. Um resultado do Yahoo igual a um local (mesmo ticker e bolsa)
/// some, e o local ganha o símbolo do Yahoo.
pub fn merge(
    query: &str,
    mut local: Vec<TickerSuggestion>,
    online: Vec<TickerSuggestion>,
    limit: usize,
) -> Vec<TickerSuggestion> {
    for o in online {
        let dup = local.iter_mut().find(|l| {
            l.ticker.eq_ignore_ascii_case(&o.ticker)
                && (l.exchange == o.exchange || l.symbol.is_some() && l.symbol == o.symbol)
        });
        match dup {
            Some(l) => {
                if l.symbol.is_none() {
                    l.symbol = o.symbol.clone();
                }
            }
            None => local.push(o),
        }
    }
    let q = query.trim().to_uppercase();
    let rank = |s: &TickerSuggestion| -> u8 {
        let exact = s.ticker == q || s.symbol.as_deref() == Some(q.as_str());
        let prefix = s.ticker.starts_with(&q);
        let origin = match s.origin {
            SuggestionOrigin::Asset => 0,
            SuggestionOrigin::Research => 1,
            SuggestionOrigin::Yahoo => 2,
        };
        (if exact {
            0
        } else if prefix {
            3
        } else {
            6
        }) + origin
    };
    // Ordenação estável: dentro do mesmo grupo vale a ordem de chegada (relevância do Yahoo).
    local.sort_by_key(rank);
    local.truncate(limit);
    local
}

struct Cache {
    map: HashMap<String, (Instant, Vec<SearchHit>)>,
}

static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| {
    Mutex::new(Cache {
        map: HashMap::new(),
    })
});

/// Busca do Yahoo com cache de [`CACHE_TTL`] por texto.
pub fn yahoo_cached(query: &str) -> std::result::Result<Vec<SearchHit>, String> {
    let key = norm(query);
    if let Some((at, hits)) = CACHE
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .map
        .get(&key)
        && at.elapsed() < CACHE_TTL
    {
        return Ok(hits.clone());
    }
    let hits = yahoo::search(&key, 12)?;
    let mut c = CACHE.lock().unwrap_or_else(|p| p.into_inner());
    if c.map.len() >= CACHE_MAX {
        c.map.retain(|_, (at, _)| at.elapsed() < CACHE_TTL);
        if c.map.len() >= CACHE_MAX {
            c.map.clear();
        }
    }
    c.map.insert(key, (Instant::now(), hits.clone()));
    Ok(hits)
}

/// `market.search`: locais (com o banco) → Yahoo (sem segurar o banco) → classificação.
pub fn run(api: &crate::Api, args: &TickerSearchArgs) -> Result<TickerSearch> {
    api.with_session(|s, _| {
        crate::sources::ensure_enabled(s.db.conn(), crate::sources::SourceId::Yahoo)
    })?;
    let limit = args.limit.unwrap_or(12).clamp(1, 30) as usize;
    let query = args.query.trim().to_string();
    if query.is_empty() {
        return Ok(TickerSearch {
            query,
            results: vec![],
            online: false,
            error: None,
        });
    }
    let local_hits = api.with_session(|s, _| local(s.db.conn(), &query, limit))?;
    let want_online = args.online.unwrap_or(true) && query.chars().count() >= 2;
    let (hits, error) = if want_online {
        match yahoo_cached(&query) {
            Ok(h) => (h, None),
            Err(e) => (
                vec![],
                Some(crate::i18n::t_args(
                    "nucleo-ativos-erro-busca-yahoo",
                    &crate::i18n::args(&[("erro", e.into())]),
                )),
            ),
        }
    } else {
        (vec![], None)
    };
    let online = want_online && error.is_none();
    let results = api.with_session(|s, _| {
        let classified = hits
            .iter()
            .map(|h| classify(s.db.conn(), h))
            .collect::<Result<Vec<_>>>()?;
        Ok(merge(&query, local_hits, classified, limit))
    })?;
    Ok(TickerSearch {
        query,
        results,
        online,
        error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::{Author, record};
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(format!(
            "{}/tests/fixtures/yahoo/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    }

    fn db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
            tx.execute(
                "INSERT INTO asset (id, ticker, name, class, market, currency, created_at, updated_at)
                 VALUES ('vod', 'VOD', 'Vodafone (minha)', 'stock_intl', 'LSE', 'GBP', 'x', 'x')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        let c = db.conn();
        c.execute(
            "INSERT INTO research_issuer (id, kind, name, currency, source, fetched_at)
             VALUES ('11', 'fii', 'CSHG Logística FII', 'BRL', 't', 'x'), ('22', 'company_br', 'Petróleo Brasileiro S.A.', 'BRL', 't', 'x')",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO research_security (ticker, issuer_id, class, source, fetched_at)
             VALUES ('HGLG11', '11', 'fii', 't', 'x'), ('PETR4', '22', 'stock_br', 't', 'x'), ('PETR3', '22', 'stock_br', 't', 'x')",
            [],
        )
        .unwrap();
        (dir, db)
    }

    fn by_symbol<'a>(v: &'a [TickerSuggestion], s: &str) -> &'a TickerSuggestion {
        v.iter()
            .find(|x| x.symbol.as_deref() == Some(s))
            .unwrap_or_else(|| panic!("{s} em {v:?}"))
    }

    #[test]
    fn classifies_yahoo_hits_by_exchange_and_type() {
        let (_d, db) = db();
        let c = db.conn();
        let hits = yahoo::parse_search(&fixture("search_toyota.json")).unwrap();
        let s: Vec<TickerSuggestion> = hits.iter().map(|h| classify(c, h).unwrap()).collect();
        let t = by_symbol(&s, "7203.T");
        assert_eq!(
            (
                t.ticker.as_str(),
                t.class,
                t.exchange.as_deref(),
                t.currency.as_deref(),
                t.country.as_deref()
            ),
            (
                "7203",
                Some(AssetClass::StockIntl),
                Some("TSE"),
                Some("JPY"),
                Some("JP")
            )
        );
        assert_eq!(t.type_label, "Ação");
        let tm = by_symbol(&s, "TM");
        assert_eq!(
            (tm.class, tm.exchange.as_deref(), tm.currency.as_deref()),
            (Some(AssetClass::StockUs), Some("NYSE"), Some("USD"))
        );
        let tmh = by_symbol(&s, "TMH");
        assert_eq!(
            (tmh.class, tmh.exchange.as_deref(), tmh.type_label.as_str()),
            (Some(AssetClass::EtfUs), Some("ARCA"), "ETF")
        );
        let ba = by_symbol(&s, "TM.BA");
        assert_eq!(
            (ba.class, ba.currency.as_deref()),
            (Some(AssetClass::StockIntl), Some("ARS"))
        );
        // Balcão americano (OTC) não está no catálogo: fica sem bolsa e sem classe.
        let otc = by_symbol(&s, "TOYOF");
        assert_eq!((otc.exchange.as_deref(), otc.class), (None, None));
        assert_eq!(otc.exchange_name.as_deref(), Some("OTC Markets"));

        let v: Vec<TickerSuggestion> = yahoo::parse_search(&fixture("search_vodafone.json"))
            .unwrap()
            .iter()
            .map(|h| classify(c, h).unwrap())
            .collect();
        let de = by_symbol(&v, "VODI.DE");
        assert_eq!(
            (
                de.ticker.as_str(),
                de.exchange.as_deref(),
                de.currency.as_deref()
            ),
            ("VODI", Some("XETRA"), Some("EUR"))
        );
        let ns = by_symbol(&v, "IDEA.NS");
        assert_eq!(
            (ns.exchange.as_deref(), ns.currency.as_deref()),
            (Some("NSE"), Some("INR"))
        );

        // B3: classe pelo universo da análise; sem ele, pelas regras do importador.
        let hit = |sym: &str, name: &str| SearchHit {
            symbol: sym.into(),
            name: name.into(),
            exchange: Some("SAO".into()),
            exchange_display: None,
            quote_type: "EQUITY".into(),
            sector: None,
            industry: None,
        };
        assert_eq!(
            classify(c, &hit("HGLG11.SA", "Cshg")).unwrap().class,
            Some(AssetClass::Fii)
        );
        assert_eq!(
            classify(
                c,
                &hit("XPML11.SA", "XP Malls Fundo De Investimento Imobiliario")
            )
            .unwrap()
            .class,
            Some(AssetClass::Fii)
        );
        assert_eq!(
            classify(c, &hit("AAPL34.SA", "Apple DRN")).unwrap().class,
            Some(AssetClass::Bdr)
        );
        let ivvb = classify(c, &hit("IVVB11.SA", "ISHARES S&P 500 FUNDO DE INDICE")).unwrap();
        assert_eq!(
            (ivvb.class, ivvb.ticker.as_str(), ivvb.currency.as_deref()),
            (Some(AssetClass::EtfBr), "IVVB11", Some("BRL"))
        );
        let btc = classify(
            c,
            &SearchHit {
                symbol: "BTC-USD".into(),
                name: "Bitcoin USD".into(),
                exchange: Some("CCC".into()),
                exchange_display: Some("CCC".into()),
                quote_type: "CRYPTOCURRENCY".into(),
                sector: None,
                industry: None,
            },
        )
        .unwrap();
        assert_eq!(
            (btc.ticker.as_str(), btc.class, btc.currency.as_deref()),
            ("BTC", Some(AssetClass::Crypto), Some("USD"))
        );
    }

    #[test]
    fn merges_local_assets_research_and_yahoo() {
        let (_d, db) = db();
        let c = db.conn();
        let loc = local(c, "petr", 10).unwrap();
        assert_eq!(loc.len(), 2);
        assert!(loc.iter().all(|s| s.origin == SuggestionOrigin::Research));
        assert_eq!(loc[0].symbol.as_deref(), Some("PETR3.SA"));
        assert_eq!(loc[0].exchange.as_deref(), Some("B3"));
        let online: Vec<TickerSuggestion> = yahoo::parse_search(&fixture("search_petr4.json"))
            .unwrap()
            .iter()
            .map(|h| classify(c, h).unwrap())
            .collect();
        let m = merge("PETR4", loc, online, 10);
        assert_eq!(m[0].ticker, "PETR4", "ticker exato primeiro");
        assert_eq!(
            m[0].origin,
            SuggestionOrigin::Research,
            "duplicata do Yahoo some"
        );
        assert_eq!(m.len(), 2);

        // Ativo cadastrado aparece antes de tudo e com o id; o mesmo papel no Yahoo some.
        let loc = local(c, "vod", 10).unwrap();
        assert_eq!(loc[0].asset_id.as_deref(), Some("vod"));
        assert_eq!(loc[0].symbol.as_deref(), Some("VOD.L"));
        let online: Vec<TickerSuggestion> = yahoo::parse_search(&fixture("search_vodafone.json"))
            .unwrap()
            .iter()
            .map(|h| classify(c, h).unwrap())
            .collect();
        let m = merge("vod", loc, online, 4);
        assert_eq!(m.len(), 4, "limite");
        assert_eq!(m[0].origin, SuggestionOrigin::Asset);
        assert_eq!(
            m.iter()
                .filter(|s| s.symbol.as_deref() == Some("VOD.L"))
                .count(),
            1
        );
        assert_eq!(
            m[1].symbol.as_deref(),
            Some("VOD"),
            "VOD (Nasdaq) é outro papel"
        );
        assert!(local(c, "  ", 5).unwrap().is_empty());
    }
}
