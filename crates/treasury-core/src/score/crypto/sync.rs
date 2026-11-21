//! Coleta do score de cripto, dentro da coleta da análise de ativos: listas da DefiLlama
//! (redes e stablecoins), ids da CoinGecko pelo símbolo, dados de cada moeda (CoinGecko,
//! com pausa entre pedidos), preços diários em dólar e valor de mercado (DefiLlama). Só os
//! criptoativos cadastrados; cada moeda no máximo uma vez por dia.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use rusqlite::Connection;

use super::{
    CryptoRegistry, REFRESH_HOURS, coingecko, defillama, fact, issuer, keys, load, local_id,
    onchain, save, symbol_of,
};
use crate::error::{Error, Result};
use crate::research::net::Fetched;
use crate::research::store;
use crate::research::sync::{Req, Runner, SyncItem, SyncSource, SyncStatus};
use crate::sources::{SourceId, enabled};

fn older_than(at: Option<&str>, hours: i64) -> bool {
    at.and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .is_none_or(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() >= hours)
}

struct Plan {
    /// Id do ativo → símbolo.
    assets: BTreeMap<String, String>,
    cg: bool,
    llama: bool,
    cm: bool,
    mempool: bool,
}

fn plan(conn: &Connection) -> Result<Plan> {
    let mut assets = BTreeMap::new();
    for a in crate::assets::list(conn, &Default::default())? {
        if a.class == crate::assets::AssetClass::Crypto
            && let Some(s) = symbol_of(&a)
        {
            assets.insert(a.id.clone(), s);
        }
    }
    Ok(Plan {
        assets,
        cg: enabled(conn, SourceId::Coingecko),
        llama: enabled(conn, SourceId::Defillama),
        cm: enabled(conn, SourceId::CoinMetrics),
        mempool: enabled(conn, SourceId::Mempool),
    })
}

/// Resolve os ids que não precisam de rede e devolve os símbolos ainda sem id.
fn resolve_local(conn: &Connection) -> Result<Vec<String>> {
    let mut reg = load(conn)?;
    let mut missing = BTreeSet::new();
    for a in crate::assets::list(conn, &Default::default())? {
        if a.class != crate::assets::AssetClass::Crypto {
            continue;
        }
        match local_id(&a, &reg) {
            Some(id) => {
                reg.ids.insert(a.id.clone(), id);
            }
            None if !reg.ids.contains_key(&a.id) => {
                if let Some(s) = symbol_of(&a) {
                    missing.insert(s);
                }
            }
            None => {}
        }
    }
    save(conn, &reg)?;
    Ok(missing.into_iter().collect())
}

fn apply_markets(conn: &Connection, ids: Vec<(String, String)>) -> Result<u64> {
    let mut reg = load(conn)?;
    let by: BTreeMap<String, String> = ids.into_iter().collect();
    let mut n = 0;
    for a in crate::assets::list(conn, &Default::default())? {
        if a.class == crate::assets::AssetClass::Crypto
            && !reg.ids.contains_key(&a.id)
            && let Some(id) = symbol_of(&a).and_then(|s| by.get(&s))
        {
            reg.ids.insert(a.id.clone(), id.clone());
            n += 1;
        }
    }
    save(conn, &reg)?;
    Ok(n)
}

fn apply_coin(conn: &Connection, c: coingecko::Coin, now: &str, today: NaiveDate) -> Result<u64> {
    let doc = crate::i18n::t_args(
        "nucleo-score-cripto-doc-coingecko",
        &crate::i18n::args(&[
            ("id", c.id.clone().into()),
            ("data", crate::time::fmt_date(today).into()),
        ]),
    );
    let mut facts = vec![];
    let mut push = |item: &str, v: Option<f64>, cur: &str| {
        if let Some(f) = v.and_then(|x| fact(item, today, x, cur, "coingecko", &doc)) {
            facts.push(f);
        }
    };
    push(keys::MCAP, c.market_cap_usd, "USD");
    push(keys::FDV, c.fdv_usd, "USD");
    push(keys::VOLUME, c.volume_usd, "USD");
    push(keys::PRICE, c.price_usd, "USD");
    push(keys::RANK, c.rank, "count");
    push(keys::CIRCULATING, c.circulating, "count");
    push(keys::TOTAL_SUPPLY, c.total_supply, "count");
    push(keys::MAX_SUPPLY, c.max_supply, "count");
    push(keys::WATCHLIST, c.watchlist_users, "count");
    push(keys::COMMITS, c.commits_4w, "count");
    push(keys::STARS, c.stars, "count");
    push(keys::ATH_DRAWDOWN, c.ath_drawdown, "ratio");
    let n = store::upsert_facts(conn, &issuer(&c.id), &facts, now)?;
    let mut reg = load(conn)?;
    let stable = coingecko::is_stablecoin(&c);
    let m = reg.coins.entry(c.id.clone()).or_default();
    m.name = c.name;
    m.symbol = c.symbol;
    m.categories = c.categories;
    m.genesis_date = c.genesis_date.map(crate::time::fmt_date);
    m.stablecoin = stable;
    m.coingecko_at = Some(now.to_string());
    save(conn, &reg)?;
    Ok(n as u64)
}

fn apply_lists(
    conn: &Connection,
    (chains, stables): (Vec<defillama::Chain>, Vec<defillama::Stable>),
    symbols: &BTreeSet<String>,
    now: &str,
    today: NaiveDate,
) -> Result<u64> {
    let mut reg: CryptoRegistry = load(conn)?;
    let registered: BTreeSet<String> = reg.ids.values().cloned().collect();
    let keep = |id: &str, sym: &str| symbols.contains(sym) || registered.contains(id);
    let chains: BTreeMap<String, (String, f64)> = chains
        .iter()
        .filter(|c| keep(&c.gecko_id, &c.symbol))
        .map(|c| (c.gecko_id.clone(), (c.symbol.clone(), c.tvl_usd)))
        .collect();
    let stables: BTreeMap<String, defillama::Stable> = stables
        .into_iter()
        .filter(|s| keep(&s.gecko_id, &s.symbol))
        .map(|s| (s.gecko_id.clone(), s))
        .collect();
    reg.chains = chains;
    reg.stables = stables;
    reg.lists_at = Some(now.to_string());
    let mut n = 0;
    for (id, (_, tvl)) in &reg.chains {
        if let Some(f) = fact(
            keys::TVL,
            today,
            *tvl,
            "USD",
            "defillama",
            "DefiLlama /v2/chains",
        ) {
            n += store::upsert_facts(conn, &issuer(id), &[f], now)?;
        }
    }
    save(conn, &reg)?;
    Ok(n as u64)
}

fn apply_chart(conn: &Connection, id: &str, pts: Vec<(NaiveDate, f64)>, now: &str) -> Result<u64> {
    let facts: Vec<_> = pts
        .iter()
        .filter_map(|(d, p)| {
            fact(
                keys::PRICE_DAILY,
                *d,
                *p,
                "USD",
                "defillama",
                "DefiLlama coins/chart",
            )
        })
        .collect();
    let n = store::upsert_facts(conn, &issuer(id), &facts, now)?;
    let mut reg = load(conn)?;
    reg.coins.entry(id.to_string()).or_default().llama_at = Some(now.to_string());
    save(conn, &reg)?;
    Ok(n as u64)
}

pub(crate) fn run(r: &mut Runner<'_>, today: NaiveDate) -> Result<()> {
    let force = r.plan.force;
    let p = r.with(|c| plan(c))?;
    if p.assets.is_empty() {
        return Ok(());
    }
    let symbols: BTreeSet<String> = p.assets.values().cloned().collect();
    // Listas da DefiLlama (redes e stablecoins): uma vez por dia.
    let lists_at = r.with(|c| Ok(load(c)?.lists_at))?;
    if p.llama && (force || older_than(lists_at.as_deref(), REFRESH_HOURS)) {
        let mut chains: Option<Vec<defillama::Chain>> = None;
        r.step(
            Req {
                source: SyncSource::Crypto,
                label: crate::i18n::t("nucleo-score-cripto-passo-tvl"),
                url: defillama::CHAINS_URL,
                ua: "",
                remember: false,
            },
            defillama::parse_chains,
            |_, v, _| {
                let n = v.len() as u64;
                chains = Some(v);
                Ok(n)
            },
        )?;
        let syms = symbols.clone();
        r.step(
            Req {
                source: SyncSource::Crypto,
                label: crate::i18n::t("nucleo-score-cripto-passo-stablecoins"),
                url: defillama::STABLES_URL,
                ua: "",
                remember: false,
            },
            defillama::parse_stables,
            |c, stables, now| {
                apply_lists(c, (chains.unwrap_or_default(), stables), &syms, now, today)
            },
        )?;
    }
    // Ids da CoinGecko: locais (fixado no ativo, tabela, listas da DefiLlama) e, para os que
    // faltarem, a busca por símbolo.
    let missing = r.with(|c| resolve_local(c))?;
    // Depois de uma falha da CoinGecko (fora do ar, limite de pedidos, rede bloqueada), os
    // outros pedidos a ela ficam para a próxima coleta.
    let mut cg_up = p.cg;
    if cg_up && !missing.is_empty() {
        let url = coingecko::markets_url(&missing);
        let st = r.step(
            Req {
                source: SyncSource::Crypto,
                label: crate::i18n::t("nucleo-score-cripto-passo-ids"),
                url: &url,
                ua: "",
                remember: false,
            },
            coingecko::parse_markets,
            |c, ids, _| apply_markets(c, ids),
        )?;
        cg_up = st != SyncStatus::Error;
    }
    let reg = r.with(|c| load(c))?;
    let ids: BTreeSet<String> = p
        .assets
        .keys()
        .filter_map(|a| reg.ids.get(a).cloned())
        .collect();
    for id in &ids {
        let meta = reg.coins.get(id);
        if cg_up
            && (force || older_than(meta.and_then(|m| m.coingecko_at.as_deref()), REFRESH_HOURS))
        {
            let url = coingecko::coin_url(id);
            let st = r.step(
                Req {
                    source: SyncSource::Crypto,
                    label: format!("CoinGecko: {id}"),
                    url: &url,
                    ua: "",
                    remember: false,
                },
                coingecko::parse_coin,
                |c, coin, now| apply_coin(c, coin, now, today),
            )?;
            cg_up = st != SyncStatus::Error;
            if st == SyncStatus::Updated {
                let url = coingecko::chart_url(id);
                let id2 = id.clone();
                r.step(
                    Req {
                        source: SyncSource::Crypto,
                        label: crate::i18n::t_args(
                            "nucleo-score-cripto-passo-historico",
                            &crate::i18n::args(&[("id", id.clone().into())]),
                        ),
                        url: &url,
                        ua: "",
                        remember: false,
                    },
                    coingecko::parse_supply_growth,
                    |c, g, now| match g.and_then(|(d, v)| {
                        fact(
                            keys::SUPPLY_GROWTH,
                            d,
                            v,
                            "ratio",
                            "coingecko",
                            &crate::i18n::t("nucleo-score-cripto-doc-market-chart"),
                        )
                    }) {
                        Some(f) => Ok(store::upsert_facts(c, &issuer(&id2), &[f], now)? as u64),
                        None => Ok(0),
                    },
                )?;
            }
        }
        if p.llama && (force || older_than(meta.and_then(|m| m.llama_at.as_deref()), REFRESH_HOURS))
        {
            let url = defillama::chart_url(id, today);
            let id2 = id.clone();
            r.step(
                Req {
                    source: SyncSource::Crypto,
                    label: crate::i18n::t_args(
                        "nucleo-score-cripto-passo-precos",
                        &crate::i18n::args(&[("id", id.clone().into())]),
                    ),
                    url: &url,
                    ua: "",
                    remember: false,
                },
                defillama::parse_chart,
                |c, pts, now| apply_chart(c, &id2, pts, now),
            )?;
        }
    }
    // On-chain (uma vez por dia): ataques às redes, endereços ativos e pools do bitcoin.
    let onchain_at = r.with(|c| Ok(load(c)?.onchain_at))?;
    if !ids.is_empty() && (force || older_than(onchain_at.as_deref(), REFRESH_HOURS)) {
        run_onchain(r, &p, &ids, today)?;
    }
    // Valor de mercado pela DefiLlama (POST): vale quando a CoinGecko não responde.
    if p.llama && !ids.is_empty() {
        let list: Vec<String> = ids.iter().cloned().collect();
        let body = defillama::mcaps_body(&list);
        let mut item = SyncItem {
            source: SyncSource::Crypto,
            label: crate::i18n::t("nucleo-score-cripto-passo-valor-mercado"),
            url: Some(defillama::MCAPS_URL.into()),
            status: SyncStatus::Error,
            bytes: 0,
            rows: 0,
            error: None,
        };
        match r.fetcher.post(defillama::MCAPS_URL, &body, "") {
            Err(e) => item.error = Some(e),
            Ok(Fetched::Body { bytes, .. }) => {
                item.bytes = bytes.len() as u64;
                match defillama::parse_mcaps(&bytes) {
                    Err(e) => item.error = Some(e),
                    Ok(m) => {
                        let res = r.with(|c| {
                            let now = crate::time::now_utc();
                            let mut n = 0;
                            for (id, v) in m {
                                if let Some(f) = fact(
                                    keys::MCAP_LLAMA,
                                    today,
                                    v,
                                    "USD",
                                    "defillama",
                                    "DefiLlama coins/mcaps",
                                ) {
                                    n += store::upsert_facts(c, &issuer(&id), &[f], &now)?;
                                }
                            }
                            Ok(n as u64)
                        });
                        match res {
                            Ok(n) => {
                                item.rows = n;
                                item.status = SyncStatus::Updated;
                            }
                            Err(e @ (Error::Locked | Error::NoVault)) => return Err(e),
                            Err(e) => item.error = Some(e.to_string()),
                        }
                    }
                }
            }
            Ok(_) => item.status = SyncStatus::NotFound,
        }
        if item.error.is_some() {
            r.rep.ok = false;
        }
        r.rep.items.push(item);
    }
    Ok(())
}

/// Fato on-chain de cada moeda (fora as stablecoins), gravado de uma vez.
fn put_each(
    conn: &Connection,
    item: &str,
    vals: &[(String, f64, NaiveDate)],
    unit: &str,
    source: &str,
    doc: &str,
    now: &str,
) -> Result<u64> {
    let mut n = 0;
    for (id, v, d) in vals {
        if let Some(f) = fact(item, *d, *v, unit, source, doc) {
            n += store::upsert_facts(conn, &issuer(id), &[f], now)?;
        }
    }
    Ok(n as u64)
}

fn run_onchain(
    r: &mut Runner<'_>,
    p: &Plan,
    ids: &BTreeSet<String>,
    today: NaiveDate,
) -> Result<()> {
    let reg = r.with(|c| load(c))?;
    // Redes (sem stablecoins), com o nome e o símbolo da CoinGecko.
    let coins: Vec<(String, String, String)> = ids
        .iter()
        .filter_map(|id| {
            let m = reg.coins.get(id)?;
            (!m.stablecoin && !reg.stables.contains_key(id))
                .then(|| (id.clone(), m.name.clone(), m.symbol.to_lowercase()))
        })
        .collect();
    if coins.is_empty() {
        return Ok(());
    }
    if p.llama {
        let cs = coins.clone();
        r.step(
            Req {
                source: SyncSource::Crypto,
                label: crate::i18n::t("nucleo-score-cripto-passo-ataques"),
                url: onchain::HACKS_URL,
                ua: "",
                remember: false,
            },
            onchain::parse_chain_hacks,
            |c, hacks, now| {
                let vals: Vec<(String, f64, NaiveDate)> = cs
                    .iter()
                    .filter(|(_, name, _)| !name.trim().is_empty())
                    .map(|(id, name, _)| {
                        (
                            id.clone(),
                            onchain::incidents_for(&hacks, name) as f64,
                            today,
                        )
                    })
                    .collect();
                put_each(
                    c,
                    keys::INCIDENTS,
                    &vals,
                    "count",
                    "defillama",
                    "DefiLlama /hacks",
                    now,
                )
            },
        )?;
    }
    if p.cm {
        let mut covered: Option<BTreeSet<String>> = None;
        r.step(
            Req {
                source: SyncSource::Crypto,
                label: crate::i18n::t("nucleo-score-cripto-passo-redes"),
                url: onchain::CM_CATALOG_URL,
                ua: "",
                remember: false,
            },
            onchain::parse_cm_catalog,
            |c, set, _| {
                let n = set.len() as u64;
                let mut reg = load(c)?;
                reg.cm_assets = set.clone();
                save(c, &reg)?;
                covered = Some(set);
                Ok(n)
            },
        )?;
        let covered = covered.unwrap_or(reg.cm_assets.clone());
        let by_cm: BTreeMap<String, String> = coins
            .iter()
            .filter(|(_, _, sym)| covered.contains(sym))
            .map(|(id, _, sym)| (sym.clone(), id.clone()))
            .collect();
        if !by_cm.is_empty() {
            let assets: Vec<String> = by_cm.keys().cloned().collect();
            let url = onchain::cm_addresses_url(&assets, today);
            r.step(
                Req {
                    source: SyncSource::Crypto,
                    label: crate::i18n::t("nucleo-score-cripto-passo-enderecos"),
                    url: &url,
                    ua: "",
                    remember: false,
                },
                onchain::parse_cm_addresses,
                |c, m, now| {
                    let vals: Vec<(String, f64, NaiveDate)> = m
                        .into_iter()
                        .filter_map(|(a, (v, d))| Some((by_cm.get(&a)?.clone(), v, d)))
                        .collect();
                    put_each(
                        c,
                        keys::ACTIVE_ADDRESSES,
                        &vals,
                        "count",
                        "coinmetrics",
                        "Coin Metrics Community API (AdrActCnt)",
                        now,
                    )
                },
            )?;
        }
    }
    if p.mempool && coins.iter().any(|(id, ..)| id == "bitcoin") {
        r.step(
            Req {
                source: SyncSource::Crypto,
                label: crate::i18n::t("nucleo-score-cripto-passo-pools"),
                url: onchain::MEMPOOL_POOLS_URL,
                ua: "",
                remember: false,
            },
            onchain::parse_mempool_pools,
            |c, (k, _), now| {
                put_each(
                    c,
                    keys::NAKAMOTO,
                    &[("bitcoin".to_string(), k as f64, today)],
                    "count",
                    "mempool",
                    "mempool.space /api/v1/mining/pools/1w",
                    now,
                )
            },
        )?;
    }
    r.with(|c| {
        let mut reg = load(c)?;
        reg.onchain_at = Some(crate::time::now_utc());
        save(c, &reg)
    })?;
    Ok(())
}
