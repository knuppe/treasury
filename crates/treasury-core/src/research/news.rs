//! Notícias de um ativo (docs/asset-overview.md §2): feeds RSS de portais de economia (o
//! mesmo pedido para todo mundo; o filtro pelo ticker e pelo nome é feito no computador, então
//! nem o ticker sai). Só título, portal, data e link: nunca o texto da matéria. Guardado na
//! memória por 30 minutos. A busca do Google Notícias não é consultada pelo app (os termos de lá
//! não aceitam acesso automático): vai só o endereço da busca, para a pessoa abrir no navegador.
//! Notícia é texto de terceiros: dado, nunca instrução.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::net::{Fetched, Fetcher};
use crate::import::text::fold;
use crate::sources::SourceId;

/// Portais com feed RSS público de mercado (conferidos em 27/10/2026).
pub const FEEDS: &[(&str, &str)] = &[
    ("InfoMoney", "https://www.infomoney.com.br/feed/"),
    ("Money Times", "https://www.moneytimes.com.br/feed/"),
    ("Seu Dinheiro", "https://www.seudinheiro.com/feed/"),
    ("E-Investidor", "https://einvestidor.estadao.com.br/feed/"),
    ("Suno", "https://www.suno.com.br/noticias/feed/"),
];

const CACHE_FOR: Duration = Duration::from_secs(30 * 60);
/// Até quantas notícias a lista devolve.
pub const MAX_ITEMS: usize = 40;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NewsItem {
    pub title: String,
    pub link: String,
    /// Portal (ou o veículo, quando o feed o informa em `<source>`).
    pub source: String,
    /// `YYYY-MM-DD`, quando o feed traz a data.
    #[ts(optional)]
    pub date: Option<String>,
}

/// Como foi cada fonte (para dizer por que veio pouco).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NewsSourceStatus {
    pub source: String,
    pub ok: bool,
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NewsView {
    pub items: Vec<NewsItem>,
    pub sources: Vec<NewsSourceStatus>,
    /// Endereço da busca do ticker no Google Notícias, para abrir no navegador. O app não o
    /// consulta.
    #[ts(optional)]
    pub search_url: Option<String>,
    pub feeds_enabled: bool,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct NewsArgs {
    #[serde(default)]
    #[ts(optional)]
    pub asset_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ticker: Option<String>,
}

fn unescape(s: &str) -> String {
    let s = s.trim();
    let s = s
        .strip_prefix("<![CDATA[")
        .and_then(|x| x.strip_suffix("]]>"))
        .unwrap_or(s);
    let mut out = s
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#039;", "'")
        .replace("&apos;", "'")
        .replace("&#8211;", "–")
        .replace("&#8216;", "‘")
        .replace("&#8217;", "’")
        .replace("&#8220;", "“")
        .replace("&#8221;", "”")
        .replace("&nbsp;", " ");
    out = out.replace("&amp;", "&");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// O conteúdo do primeiro `<tag …>…</tag>` do trecho.
fn tag<'a>(item: &'a str, name: &str) -> Option<(&'a str, &'a str)> {
    let open = format!("<{name}");
    let mut from = 0;
    loop {
        let i = from + item[from..].find(&open)?;
        let after = &item[i + open.len()..];
        // `<source` não pode casar com `<sourceUrl`.
        if after.starts_with(['>', ' ', '\t', '\n', '\r', '/']) {
            let gt = after.find('>')?;
            let attrs = &after[..gt];
            let body_start = i + open.len() + gt + 1;
            let close = format!("</{name}>");
            let end = body_start + item[body_start..].find(&close)?;
            return Some((attrs, &item[body_start..end]));
        }
        from = i + open.len();
    }
}

fn rss_date(s: &str) -> Option<String> {
    let s = s.trim();
    DateTime::parse_from_rfc2822(s)
        .map(|d| d.date_naive())
        .or_else(|_| DateTime::parse_from_rfc3339(s).map(|d| d.date_naive()))
        .ok()
        .or_else(|| NaiveDate::parse_from_str(s.get(..10)?, "%Y-%m-%d").ok())
        .map(crate::time::fmt_date)
}

/// Lê os itens de um feed RSS 2.0 (título, link, data, veículo e a descrição para o filtro).
pub fn parse_rss(xml: &str, source: &str) -> Vec<(NewsItem, String)> {
    let mut out = vec![];
    let mut rest = xml;
    while let Some(i) = rest.find("<item") {
        let Some(j) = rest[i..].find("</item>") else {
            break;
        };
        let item = &rest[i..i + j];
        rest = &rest[i + j + 7..];
        let Some(title) = tag(item, "title").map(|(_, b)| unescape(b)) else {
            continue;
        };
        let Some(link) = tag(item, "link").map(|(_, b)| unescape(b)) else {
            continue;
        };
        if title.is_empty() || !link.starts_with("https://") && !link.starts_with("http://") {
            continue;
        }
        let date = tag(item, "pubDate").and_then(|(_, b)| rss_date(b));
        // Em feeds agregados, o veículo vem em `<source>` e no fim do título (" - Valor").
        let (title, source) = match tag(item, "source").map(|(_, b)| unescape(b)) {
            Some(s) if !s.is_empty() => {
                let t = title
                    .strip_suffix(&format!(" - {s}"))
                    .map(str::to_string)
                    .unwrap_or(title);
                (t, s)
            }
            _ => (title, source.to_string()),
        };
        let desc = tag(item, "description")
            .map(|(_, b)| unescape(b))
            .unwrap_or_default();
        out.push((
            NewsItem {
                title,
                link,
                source,
                date,
            },
            desc,
        ));
    }
    out
}

/// Palavras que identificam o ativo no texto: o ticker e os nomes (só os com 4+ letras).
pub fn terms(ticker: Option<&str>, names: &[&str]) -> Vec<String> {
    let mut t: Vec<String> = vec![];
    if let Some(tk) = ticker.map(str::trim).filter(|s| !s.is_empty()) {
        t.push(fold(tk));
    }
    const GENERIC: &[&str] = &[
        "banco",
        "brasil",
        "brasileiro",
        "brasileira",
        "companhia",
        "cia",
        "grupo",
        "holding",
        "participacoes",
        "fundo",
        "investimento",
        "imobiliario",
        "energia",
        "petroleo",
    ];
    for n in names {
        let f = fold(n);
        // O nome de pregão costuma ser uma palavra só (PETROBRAS, ITAUUNIBANCO): a primeira
        // palavra distintiva do nome serve.
        if let Some(w) = f
            .split(' ')
            .find(|w| w.len() >= 4 && !GENERIC.contains(w) && w.chars().all(char::is_alphanumeric))
            && !t.contains(&w.to_string())
        {
            t.push(w.to_string());
        }
    }
    t
}

/// A notícia fala do ativo: algum termo aparece como palavra no título ou na descrição.
pub fn mentions(title: &str, desc: &str, terms: &[String]) -> bool {
    let text = format!(" {} ", fold(&format!("{title} {desc}")));
    terms.iter().any(|t| text.contains(&format!(" {t} ")))
}

/// Itens de um feed com a descrição (só para o filtro).
type FeedItems = Vec<(NewsItem, String)>;

static CACHE: LazyLock<Mutex<HashMap<String, (Instant, FeedItems)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// O feed, da memória (30 minutos) ou da rede.
fn feed(
    f: &dyn Fetcher,
    url: &str,
    source: &str,
) -> std::result::Result<Vec<(NewsItem, String)>, String> {
    if let Some((at, items)) = CACHE.lock().ok().and_then(|c| c.get(url).cloned())
        && at.elapsed() < CACHE_FOR
    {
        return Ok(items);
    }
    super::net::pace(url);
    let bytes = match f.get(url, None, "")? {
        Fetched::Body { bytes, .. } => bytes,
        Fetched::NotFound => return Err("404".into()),
        Fetched::NotModified => vec![],
    };
    let items = parse_rss(&String::from_utf8_lossy(&bytes), source);
    if let Ok(mut c) = CACHE.lock() {
        c.insert(url.to_string(), (Instant::now(), items.clone()));
    }
    Ok(items)
}

/// Página de busca do Google Notícias com o ticker, para abrir no navegador (português,
/// Brasil). Nunca é pedida pelo app.
pub fn search_url(ticker: &str) -> Option<String> {
    let q: String = ticker
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    (!q.is_empty())
        .then(|| format!("https://news.google.com/search?q={q}&hl=pt-BR&gl=BR&ceid=BR:pt-419"))
}

/// Junta as fontes ligadas, filtra pelo ativo e ordena da mais nova para a mais antiga.
pub fn collect(f: &dyn Fetcher, ticker: Option<&str>, names: &[&str], feeds_on: bool) -> NewsView {
    let terms = terms(ticker, names);
    let mut items: Vec<NewsItem> = vec![];
    let mut sources = vec![];
    let mut push = |source: &str, r: std::result::Result<Vec<(NewsItem, String)>, String>| match r {
        Ok(list) => {
            sources.push(NewsSourceStatus {
                source: source.to_string(),
                ok: true,
                error: None,
            });
            for (it, desc) in list {
                if mentions(&it.title, &desc, &terms)
                    && !items
                        .iter()
                        .any(|x| x.link == it.link || x.title == it.title)
                {
                    items.push(it);
                }
            }
        }
        Err(e) => sources.push(NewsSourceStatus {
            source: source.to_string(),
            ok: false,
            error: Some(e),
        }),
    };
    if feeds_on && !terms.is_empty() {
        for (name, url) in FEEDS {
            push(name, feed(f, url, name));
        }
    }
    items.sort_by(|a, b| b.date.cmp(&a.date));
    items.truncate(MAX_ITEMS);
    NewsView {
        items,
        sources,
        search_url: ticker.and_then(search_url),
        feeds_enabled: feeds_on,
    }
}

/// `research.news`: as notícias do ativo (rede fora da trava do banco).
pub fn run(api: &crate::api::Api, a: &NewsArgs) -> crate::error::Result<NewsView> {
    let (ticker, names, feeds_on) = api.with_session(|s, _| {
        let conn = s.db.conn();
        let (ticker, mut names) = match a.asset_id.as_deref().filter(|s| !s.is_empty()) {
            Some(id) => {
                let asset = crate::assets::get(conn, id)?;
                let names = vec![asset.name.clone()];
                (asset.ticker, names)
            }
            None => (a.ticker.clone().map(|t| t.trim().to_uppercase()), vec![]),
        };
        // O nome de pregão do universo (ex.: PETROBRAS).
        if let Some(tk) = &ticker {
            use rusqlite::OptionalExtension;
            let n: Option<String> = conn
                .query_row(
                    "SELECT name FROM research_universe WHERE ticker = ?1",
                    [tk],
                    |r| r.get(0),
                )
                .optional()?;
            names.extend(n);
        }
        Ok((
            ticker,
            names,
            crate::sources::enabled(conn, SourceId::NewsFeeds),
        ))
    })?;
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let f = super::net::HttpFetcher::new();
    Ok(collect(&f, ticker.as_deref(), &names, feeds_on))
}

#[cfg(test)]
pub(crate) fn clear_cache() {
    if let Ok(mut c) = CACHE.lock() {
        c.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::net::FetchResult;

    /// Feed inventado no formato dos portais (WordPress) e de um agregador (com `<source>`).
    const PORTAL: &str = r#"<?xml version="1.0"?><rss><channel><title>Portal</title>
<item><title><![CDATA[Exemplo Energia (EXEN3) anuncia dividendos]]></title>
<link>https://portal.exemplo.invalid/noticia-1</link>
<pubDate>Mon, 26 Oct 2026 18:30:00 -0300</pubDate>
<description><![CDATA[A companhia aprovou &amp; pagará…]]></description></item>
<item><title>Ibovespa fecha em alta</title><link>https://portal.exemplo.invalid/noticia-2</link>
<pubDate>Tue, 27 Oct 2026 10:00:00 -0300</pubDate>
<description>Destaque para a EXEMPLOENERGIA no fim do pregão.</description></item>
<item><title>Dólar cai</title><link>https://portal.exemplo.invalid/noticia-3</link></item>
</channel></rss>"#;
    const AGGREGATOR: &str = r#"<rss><channel><item><title>EXEN3 sobe 5% - Jornal Exemplo</title>
<link>https://agregador.exemplo.invalid/abc</link><pubDate>Wed, 28 Oct 2026 09:00:00 GMT</pubDate>
<source url="https://jornal.exemplo.invalid">Jornal Exemplo</source></item></channel></rss>"#;

    struct Fake;
    impl Fetcher for Fake {
        fn get(&self, url: &str, _: Option<&super::super::net::FileMeta>, _: &str) -> FetchResult {
            assert!(
                !url.contains("google"),
                "o app não consulta o Google: {url}"
            );
            let body = if url.contains("infomoney") {
                PORTAL
            } else {
                return Err("fora do ar".into());
            };
            Ok(Fetched::Body {
                bytes: body.as_bytes().to_vec(),
                meta: Default::default(),
            })
        }
    }

    #[test]
    fn parses_rss_and_filters_on_this_computer() {
        let items = parse_rss(PORTAL, "Portal");
        assert_eq!(items.len(), 3);
        assert_eq!(
            items[0].0.title,
            "Exemplo Energia (EXEN3) anuncia dividendos"
        );
        assert_eq!(items[0].0.date.as_deref(), Some("2026-10-26"));
        assert_eq!(items[0].1, "A companhia aprovou & pagará…");
        let t = terms(Some("EXEN3"), &["EXEMPLOENERGIA", "Exemplo Energia S.A."]);
        assert_eq!(t, vec!["exen3", "exemploenergia", "exemplo"]);
        assert!(mentions(&items[0].0.title, &items[0].1, &t));
        assert!(mentions(&items[1].0.title, &items[1].1, &t));
        assert!(!mentions(&items[2].0.title, &items[2].1, &t));
        let g = parse_rss(AGGREGATOR, "Agregador");
        assert_eq!(g[0].0.title, "EXEN3 sobe 5%");
        assert_eq!(g[0].0.source, "Jornal Exemplo");
    }

    #[test]
    fn collects_from_the_sources_turned_on() {
        clear_cache();
        let v = collect(&Fake, Some("EXEN3"), &["EXEMPLOENERGIA"], true);
        assert_eq!(v.items.len(), 2);
        // Mais nova primeiro.
        assert_eq!(v.items[0].date.as_deref(), Some("2026-10-27"));
        assert!(v.sources.iter().any(|s| s.source == "InfoMoney" && s.ok));
        assert!(
            v.sources
                .iter()
                .any(|s| !s.ok && s.error.as_deref() == Some("fora do ar"))
        );
        // A busca do Google vai só como endereço para o navegador, com o ticker e nada mais.
        assert_eq!(
            v.search_url.as_deref(),
            Some("https://news.google.com/search?q=EXEN3&hl=pt-BR&gl=BR&ceid=BR:pt-419")
        );
        // Com os feeds desligados, nenhum pedido.
        let v = collect(&Fake, Some("EXEN3"), &[], false);
        assert!(v.items.is_empty() && v.sources.is_empty() && !v.feeds_enabled);
        assert!(
            collect(&Fake, None, &["Exemplo"], false)
                .search_url
                .is_none()
        );
    }
}

/// Conferência com a rede de verdade (fora da CI): `TREASURY_LIVE=1 cargo test -p treasury-core
/// --lib live_news -- --ignored --nocapture`. Só números e títulos públicos na saída.
#[cfg(test)]
#[test]
#[ignore]
fn live_news() {
    if std::env::var("TREASURY_LIVE").is_err() {
        return;
    }
    let f = super::net::HttpFetcher::new();
    let v = collect(&f, Some("PETR4"), &["PETROBRAS"], true);
    for s in &v.sources {
        eprintln!("{} ok={} {:?}", s.source, s.ok, s.error);
    }
    for i in v.items.iter().take(8) {
        eprintln!("{:?} {} | {}", i.date, i.source, i.title);
    }
    let link = "https://www.rad.cvm.gov.br/ENET/frmDownloadDocumento.aspx?Tela=ext&descTipo=IPE&CodigoInstituicao=1&numProtocolo=1576827&numSequencia=1101533&numVersao=1";
    let (text, ocr) = super::doc_read::text_of(&f, "live", link).unwrap();
    eprintln!("documento: {} caracteres, ocr={ocr}", text.chars().count());
}
