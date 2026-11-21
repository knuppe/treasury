//! Coleta da análise de ativos, no padrão das outras coletas: planejar (lê o banco) →
//! baixar e interpretar sem travar o banco → gravar, um arquivo por vez, com relatório de
//! erros por item (uma fonte fora do ar não impede as outras).
//!
//! Ordem: FCA (setores e tickers; define o universo de companhias com ações negociadas) →
//! DFP (10 exercícios) → ITR (3 anos, para os últimos 8 trimestres e o LTM) → LTM → informe
//! mensal de FII (6 anos) → informe trimestral de FII (3 anos) → IPE (2 anos, todas as companhias com ação negociada e os emissores cadastrados) → B3 COTAHIST
//! (pregões que faltam, até ~1 mês) → B3 proventos e eventos oficiais (coleta rolante) → SEC (ativos dos EUA cadastrados) → Banco Central IF.data
//! (5 trimestres) → cadastro dos fundos e ETFs cadastrados (CVM, carteiras teóricas da B3) →
//! criptoativos cadastrados (CoinGecko, DefiLlama) → universo do dia e radar de dividendos.
//! Arquivos que não mudaram (ETag) não são baixados de novo.

use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate, Weekday};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::net::{Fetched, Fetcher, FileMeta, HttpFetcher};
use super::{IssuerKind, Layout, Security, b3, cvm, items, sec, store};
use crate::error::{Error, Result};
use crate::time::{add_months, fmt_date, now_utc, parse_date, today_local};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SyncSource {
    Fca,
    Dfp,
    Itr,
    Fii,
    /// Informe trimestral de FII (imóveis, inquilinos, contratos, resultado).
    FiiQuarterly,
    Ipe,
    B3,
    Sec,
    /// Banco Central IF.data (indicadores prudenciais dos bancos).
    #[serde(rename = "bacen_ifdata")]
    Ifdata,
    /// Cadastro, taxas e prazos dos fundos e ETFs cadastrados (CVM) e carteiras teóricas
    /// dos índices da B3 (score de fundos e ETFs).
    Funds,
    /// Criptoativos cadastrados: CoinGecko e DefiLlama (score de cripto).
    Crypto,
    /// brapi.dev (opcional, com token): segunda fonte de indicadores e proventos.
    Brapi,
    /// Proventos e eventos em ações oficiais da B3 (API das empresas e fundos listados).
    B3Listados,
}

const ALL_SOURCES: [SyncSource; 13] = [
    SyncSource::Fca,
    SyncSource::Dfp,
    SyncSource::Itr,
    SyncSource::Fii,
    SyncSource::FiiQuarterly,
    SyncSource::Ipe,
    SyncSource::B3,
    SyncSource::Sec,
    SyncSource::Ifdata,
    SyncSource::Funds,
    SyncSource::Crypto,
    SyncSource::Brapi,
    SyncSource::B3Listados,
];

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SyncArgs {
    /// Baixa de novo mesmo os arquivos que não mudaram.
    #[serde(default)]
    #[ts(optional)]
    pub force: Option<bool>,
    /// Só estas fontes (padrão: todas).
    #[serde(default)]
    #[ts(optional)]
    pub sources: Option<Vec<SyncSource>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SyncStatus {
    /// Baixado e gravado.
    Updated,
    /// Não mudou desde a última coleta.
    Unchanged,
    /// Arquivo inexistente (ano sem dados, dia sem pregão).
    NotFound,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SyncItem {
    pub source: SyncSource,
    pub label: String,
    pub url: Option<String>,
    pub status: SyncStatus,
    #[ts(type = "number")]
    pub bytes: u64,
    #[ts(type = "number")]
    pub rows: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResearchSyncReport {
    pub started_at: String,
    pub finished_at: String,
    pub ok: bool,
    pub items: Vec<SyncItem>,
    /// Total baixado nesta coleta.
    #[ts(type = "number")]
    pub bytes: u64,
    /// Papéis no universo recalculado.
    #[ts(type = "number")]
    pub universe: u64,
    /// Proventos projetados pelo radar.
    #[ts(type = "number")]
    pub radar: u64,
    /// Papéis do universo com score em lote (explorador de mercado).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub market_scores: Option<u64>,
    pub warnings: Vec<String>,
}

/// O que a coleta vai buscar (lido do banco antes da rede).
#[derive(Debug, Clone)]
pub struct Plan {
    pub today: NaiveDate,
    pub force: bool,
    pub sources: HashSet<SyncSource>,
    pub metas: HashMap<String, FileMeta>,
    /// Ativos dos EUA cadastrados: (ticker no padrão da SEC).
    pub us_tickers: Vec<String>,
    /// Subjacentes prováveis dos BDRs cadastrados (só baixados se a SEC conhecer o ticker).
    pub bdr_candidates: HashSet<String>,
    pub quote_dates: Vec<NaiveDate>,
    /// Meses fechados do COTAHIST que faltam no histórico de 13 meses (mais recentes primeiro,
    /// no máximo [`QUOTE_MONTHS_PER_RUN`] por coleta).
    pub quote_months: Vec<(i32, u32)>,
    pub sec_user_agent: String,
    pub fca_years: Vec<i32>,
    pub dfp_years: Vec<i32>,
    pub itr_years: Vec<i32>,
    pub fii_years: Vec<i32>,
    pub fii_quarterly_years: Vec<i32>,
    pub ipe_years: Vec<i32>,
    /// Última sondagem do trimestre mais recente do IF.data.
    pub ifdata_probe_at: Option<String>,
    /// Fontes baixadas de novo (sem ETag) uma vez, porque o parser ganhou itens novos.
    pub reparse: HashSet<SyncSource>,
    /// Cofre novo: grava a versão do parser no fim da coleta.
    pub record_parser: bool,
    /// Todas as fontes da releitura estão nesta coleta (só então a versão é gravada).
    pub reparse_all: bool,
}

/// Versão do parser da CVM. Ao subir, os arquivos já processados (marcados como sem mudança
/// pelo ETag) são baixados e lidos de novo uma vez, para gravar os itens novos.
/// 2: itens do score (DFP/ITR), auditores (FCA), ativo e passivo e entregas (FII mensal).
/// 3: itens do score nas empresas americanas (SEC companyfacts) e setor (SEC submissions).
/// 4: dívida das americanas também por notas de longo prazo e conversíveis (SEC).
/// 5: FII trimestral sem a receita por imóvel (inadimplência pela área, sem maior imóvel zero).
/// 6: FII trimestral, maior imóvel pela área.
pub const PARSER_VERSION: u32 = 6;
const PARSER_VERSION_SETTING: &str = "research.parser_version";
/// Fontes lidas de novo em cada versão do parser (só as das versões depois da gravada).
const REPARSE_BY_VERSION: &[(u32, &[SyncSource])] = &[
    (
        2,
        &[
            SyncSource::Fca,
            SyncSource::Dfp,
            SyncSource::Itr,
            SyncSource::Fii,
        ],
    ),
    (3, &[SyncSource::Sec]),
    (4, &[SyncSource::Sec]),
    (5, &[SyncSource::FiiQuarterly]),
    (6, &[SyncSource::FiiQuarterly]),
];

/// Fontes a reler de uma versão gravada até a atual.
fn reparse_sources(stored: u32) -> HashSet<SyncSource> {
    REPARSE_BY_VERSION
        .iter()
        .filter(|(v, _)| *v > stored && *v <= PARSER_VERSION)
        .flat_map(|(_, s)| s.iter().copied())
        .collect()
}

/// Dias úteis (seg–sex) que faltam no COTAHIST, até 22 pregões para trás.
/// Meses de cotações guardados (volatilidade de 1 ano e momentum 12-1 do universo).
pub const QUOTE_HISTORY_DAYS: i64 = 400;
/// Arquivos mensais por coleta: o histórico se completa em poucos dias, sem rajada.
pub const QUOTE_MONTHS_PER_RUN: usize = 4;
const QUOTE_MONTHS_SETTING: &str = "research.cotahist_months";

/// Meses fechados dos últimos 13 que ainda não foram carregados.
fn quote_months(conn: &Connection, today: NaiveDate) -> Result<Vec<(i32, u32)>> {
    let done: Vec<String> = crate::settings::get(conn, QUOTE_MONTHS_SETTING)?.unwrap_or_default();
    let mut out = vec![];
    for back in 1..=13 {
        let d = add_months(today, -back);
        let key = format!("{:04}-{:02}", d.year(), d.month());
        if !done.contains(&key) {
            out.push((d.year(), d.month()));
        }
    }
    out.truncate(QUOTE_MONTHS_PER_RUN);
    Ok(out)
}

fn quote_dates(conn: &Connection, today: NaiveDate, force: bool) -> Result<Vec<NaiveDate>> {
    let last: Option<String> = conn
        .query_row("SELECT max(date) FROM research_quote", [], |r| r.get(0))
        .optional()?
        .flatten();
    let floor = today - chrono::Duration::days(31);
    let start = match last.map(|d| parse_date(&d)).transpose()? {
        Some(d) if !force => (d + chrono::Duration::days(1)).max(floor),
        _ => floor,
    };
    let mut out = vec![];
    let mut d = start;
    while d <= today {
        if !matches!(d.weekday(), Weekday::Sat | Weekday::Sun) {
            out.push(d);
        }
        d += chrono::Duration::days(1);
    }
    let n = out.len();
    Ok(out.split_off(n.saturating_sub(22)))
}

pub fn plan(conn: &Connection, today: NaiveDate, args: &SyncArgs) -> Result<Plan> {
    let cy = today.year();
    let force = args.force.unwrap_or(false);
    let mut sources: HashSet<SyncSource> = match &args.sources {
        Some(v) if !v.is_empty() => v.iter().copied().collect(),
        _ => ALL_SOURCES.into_iter().collect(),
    };
    // Fontes desligadas em Ajustes › Fontes de dados ficam de fora.
    {
        use crate::sources::{SourceId, enabled};
        let (cvm, b3, sec, bcb) = (
            enabled(conn, SourceId::Cvm),
            enabled(conn, SourceId::B3),
            enabled(conn, SourceId::Sec),
            enabled(conn, SourceId::Bcb),
        );
        let crypto = enabled(conn, SourceId::Coingecko) || enabled(conn, SourceId::Defillama);
        sources.retain(|s| match s {
            SyncSource::B3 | SyncSource::B3Listados => b3,
            SyncSource::Sec => sec,
            SyncSource::Ifdata => bcb,
            SyncSource::Funds => cvm || b3,
            SyncSource::Crypto => crypto,
            SyncSource::Brapi => {
                enabled(conn, SourceId::Brapi) && super::brapi::token(conn).ok().flatten().is_some()
            }
            _ => cvm,
        });
    }
    let mut metas = HashMap::new();
    let mut stmt = conn.prepare("SELECT url, etag, last_modified FROM research_file")?;
    for r in stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            FileMeta {
                etag: r.get(1)?,
                last_modified: r.get(2)?,
            },
        ))
    })? {
        let (u, m) = r?;
        metas.insert(u, m);
    }
    let mut us_tickers = vec![];
    let mut bdr_candidates = HashSet::new();
    for a in crate::assets::list(conn, &Default::default())? {
        if matches!(
            a.class,
            crate::assets::AssetClass::StockUs | crate::assets::AssetClass::Reit
        ) && let Some(t) = &a.ticker
        {
            us_tickers.push(sec::normalize_ticker(t));
        }
        if a.class == crate::assets::AssetClass::Bdr
            && let Some(t) = &a.ticker
            && let Some(u) = crate::score::intl::bdr_underlying(conn, t)?
        {
            let u = sec::normalize_ticker(&u);
            bdr_candidates.insert(u.clone());
            us_tickers.push(u);
        }
    }
    us_tickers.sort();
    us_tickers.dedup();
    // Sem versão gravada: cofre de antes da versão 2 se já há arquivos baixados; senão, cofre
    // novo, que não tem nada a reler.
    let stored: Option<u32> = crate::settings::get(conn, PARSER_VERSION_SETTING)?;
    let fresh = stored.is_none() && metas.is_empty();
    let parser = stored.unwrap_or(if fresh { PARSER_VERSION } else { 1 });
    let reparse: HashSet<SyncSource> = reparse_sources(parser)
        .into_iter()
        .filter(|s| sources.contains(s))
        .collect();
    let reparse_all = !reparse.is_empty() && reparse.len() == reparse_sources(parser).len();
    Ok(Plan {
        today,
        force,
        sources,
        metas,
        us_tickers,
        bdr_candidates,
        quote_dates: quote_dates(conn, today, force)?,
        quote_months: quote_months(conn, today)?,
        sec_user_agent: super::config::load(conn)?.sec_user_agent,
        fca_years: vec![cy - 1, cy],
        dfp_years: ((cy - 10)..=cy).collect(),
        itr_years: vec![cy - 2, cy - 1, cy],
        fii_years: ((cy - 5)..=cy).collect(),
        fii_quarterly_years: vec![cy - 2, cy - 1, cy],
        ipe_years: vec![cy - 1, cy],
        ifdata_probe_at: crate::settings::get(conn, IFDATA_PROBE_SETTING)?,
        reparse,
        record_parser: fresh,
        reparse_all,
    })
}

/// Um arquivo a baixar.
pub(crate) struct Req<'a> {
    pub(crate) source: SyncSource,
    pub(crate) label: String,
    pub(crate) url: &'a str,
    /// User-Agent (vazio = o padrão do app).
    pub(crate) ua: &'a str,
    /// Guarda o ETag para a próxima coleta.
    pub(crate) remember: bool,
}

pub(crate) struct Runner<'a> {
    pub(crate) api: &'a crate::Api,
    pub(crate) fetcher: &'a dyn Fetcher,
    pub(crate) plan: &'a Plan,
    pub(crate) rep: ResearchSyncReport,
    /// Ignora o ETag (ex.: ativo cadastrado cujo emissor ainda não tem demonstrações).
    refetch: bool,
}

impl Runner<'_> {
    /// Baixa (condicional), interpreta fora do banco e grava numa transação.
    pub(crate) fn step<T>(
        &mut self,
        req: Req<'_>,
        parse: impl FnOnce(&[u8]) -> std::result::Result<T, String>,
        apply: impl FnOnce(&Connection, T, &str) -> Result<u64>,
    ) -> Result<SyncStatus> {
        let Req {
            source,
            label,
            url,
            ua,
            remember,
        } = req;
        // Tarefa em segundo plano: o arquivo atual e quantos já foram; cancelada, para aqui (o
        // que já veio fica gravado, cada arquivo na sua transação).
        crate::tasks::check()?;
        crate::tasks::detail(label.clone());
        crate::tasks::advance();
        let cond = if self.plan.force
            || self.refetch
            || !remember
            || self.plan.reparse.contains(&source)
        {
            None
        } else {
            self.plan.metas.get(url)
        };
        let mut item = SyncItem {
            source,
            label,
            url: Some(url.to_string()),
            status: SyncStatus::Error,
            bytes: 0,
            rows: 0,
            error: None,
        };
        match self.fetcher.get(url, cond, ua) {
            Err(e) => item.error = Some(e),
            Ok(Fetched::NotModified) => item.status = SyncStatus::Unchanged,
            Ok(Fetched::NotFound) => item.status = SyncStatus::NotFound,
            Ok(Fetched::Body { bytes, meta }) => {
                item.bytes = bytes.len() as u64;
                self.rep.bytes += item.bytes;
                match parse(&bytes) {
                    Err(e) => item.error = Some(e),
                    Ok(parsed) => {
                        drop(bytes);
                        let size = item.bytes;
                        let res = self.api.with_session(|s, _| {
                            let tx = s.db.conn_mut().transaction()?;
                            let now = now_utc();
                            let n = apply(&tx, parsed, &now)?;
                            if remember {
                                tx.execute(
                                    "INSERT INTO research_file (url, etag, last_modified, bytes, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5)
                                     ON CONFLICT (url) DO UPDATE SET etag = excluded.etag, last_modified = excluded.last_modified,
                                         bytes = excluded.bytes, fetched_at = excluded.fetched_at",
                                    params![url, meta.etag, meta.last_modified, size as i64, now],
                                )?;
                            }
                            tx.commit()?;
                            Ok(n)
                        });
                        match res {
                            Ok(n) => {
                                item.rows = n;
                                item.status = SyncStatus::Updated;
                            }
                            // Cofre bloqueado no meio da coleta: para tudo.
                            Err(e @ (Error::Locked | Error::NoVault)) => return Err(e),
                            Err(e) => item.error = Some(e.to_string()),
                        }
                    }
                }
            }
        }
        if item.error.is_some() {
            self.rep.ok = false;
        }
        let st = item.status;
        self.rep.items.push(item);
        let delay = self.fetcher.polite_delay(url);
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
        Ok(st)
    }

    pub(crate) fn with<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        self.api.with_session(|s, _| f(s.db.conn_mut()))
    }
}

fn apply_fca(conn: &Connection, f: cvm::Fca, now: &str) -> Result<u64> {
    for i in &f.issuers {
        // O leiaute que veio da DRE vale mais que o deduzido do setor.
        let mut i = i.clone();
        if let Some(e) = store::issuer(conn, &i.id)? {
            i.layout = e.layout;
        }
        store::upsert_issuer(conn, &i, now)?;
    }
    let known: HashSet<String> = f.issuers.iter().map(|i| i.id.clone()).collect();
    let mut n = f.issuers.len() as u64;
    n += store::replace_auditors(conn, &f.auditors, now)? as u64;
    for s in &f.securities {
        if known.contains(&s.issuer_id) || store::issuer(conn, &s.issuer_id)?.is_some() {
            store::upsert_security(conn, s, now)?;
            n += 1;
        }
    }
    Ok(n)
}

fn apply_filings(conn: &Connection, st: cvm::Statements, now: &str) -> Result<u64> {
    let mut n = store::upsert_filings(conn, &st.records, now)? as u64;
    for f in st.filings {
        let existing = store::issuer(conn, &f.cnpj)?;
        let layout = f
            .layout
            .or(existing.as_ref().map(|e| e.layout))
            .unwrap_or(Layout::Industrial);
        let issuer = super::Issuer {
            id: f.cnpj.clone(),
            kind: IssuerKind::CompanyBr,
            name: existing
                .as_ref()
                .map(|e| e.name.clone())
                .unwrap_or_else(|| f.name.clone()),
            cvm_code: f.cvm_code.clone(),
            sector: None,
            segment: None,
            mandate: None,
            layout,
            currency: "BRL".into(),
            source: existing
                .as_ref()
                .map(|e| e.source.clone())
                .unwrap_or_else(|| f.kind.source().into()),
        };
        store::upsert_issuer(conn, &issuer, now)?;
        n += store::upsert_facts(conn, &f.cnpj, &f.facts, now)? as u64;
    }
    Ok(n)
}

/// Recalcula o LTM de todos os emissores com demonstrações e poda acumulados antigos do
/// ITR (ficam os últimos 8 trimestres e o comparativo do LTM).
pub fn recompute_ltm(conn: &Connection) -> Result<u64> {
    let now = now_utc();
    let ids: Vec<String> = conn
        .prepare(
            "SELECT DISTINCT issuer_id FROM fundamental WHERE period_type IN ('annual', 'ytd')",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut n = 0;
    for id in ids {
        let latest_itr: Option<String> = conn
            .query_row(
                "SELECT max(period_end) FROM fundamental WHERE issuer_id = ?1 AND source = 'cvm_itr'",
                [&id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        if let Some(l) = latest_itr {
            let cut = add_months(parse_date(&l)?, -24);
            conn.execute(
                "DELETE FROM fundamental WHERE issuer_id = ?1 AND source = 'cvm_itr' AND period_end < ?2",
                params![id, fmt_date(cut)],
            )?;
        }
        let facts = store::load_facts(conn, &id)?;
        let ltm = super::periods::compute_ltm(&facts);
        n += ltm.len() as u64;
        store::replace_ltm(conn, &id, &ltm, &now)?;
    }
    Ok(n)
}

fn apply_fii(conn: &Connection, inf: cvm::FiiInforme, now: &str, today: NaiveDate) -> Result<u64> {
    for i in &inf.issuers {
        store::upsert_issuer(conn, i, now)?;
    }
    for s in &inf.securities {
        store::upsert_security(conn, s, now)?;
    }
    let mut by: HashMap<String, Vec<super::Fact>> = HashMap::new();
    for (id, f) in inf.facts {
        by.entry(id).or_default().push(f);
    }
    let mut n = (inf.issuers.len() + inf.securities.len()) as u64;
    for (id, facts) in by {
        n += store::upsert_facts(conn, &id, &facts, now)? as u64;
    }
    n += store::upsert_filings(conn, &inf.filings, now)? as u64;
    // O rendimento por cota guarda 6 anos (Bazin); o valor patrimonial por cota e a taxa de
    // administração, 25 meses (variação de 12 meses do score de saúde); o resto, 13 meses.
    conn.execute(
        "DELETE FROM fundamental WHERE period_type = 'month' AND item NOT IN (?1, ?2, ?3) AND period_end < ?4",
        params![
            items::INCOME_PER_SHARE,
            items::NAV_PER_SHARE,
            items::FII_ADMIN_FEE_PCT,
            fmt_date(add_months(today, -13))
        ],
    )?;
    conn.execute(
        "DELETE FROM fundamental WHERE period_type = 'month' AND item IN (?1, ?2) AND period_end < ?3",
        params![
            items::NAV_PER_SHARE,
            items::FII_ADMIN_FEE_PCT,
            fmt_date(add_months(today, -25))
        ],
    )?;
    Ok(n)
}

fn apply_fii_quarterly(
    conn: &Connection,
    q: cvm::FiiQuarterly,
    now: &str,
    today: NaiveDate,
) -> Result<u64> {
    let mut by: HashMap<String, Vec<super::Fact>> = HashMap::new();
    for (id, f) in q.facts {
        by.entry(id).or_default().push(f);
    }
    let mut n = 0;
    for (id, facts) in by {
        // O informe de um trimestre substitui o anterior inteiro: um item que deixou de sair
        // (o maior imóvel zerado de antes) não fica para trás.
        let ends: HashSet<String> = facts.iter().map(|f| fmt_date(f.end)).collect();
        for end in &ends {
            conn.execute(
                "DELETE FROM fundamental WHERE issuer_id = ?1 AND source = 'cvm_fii_trimestral'
                   AND period_end = ?2",
                params![id, end],
            )?;
        }
        n += store::upsert_facts(conn, &id, &facts, now)? as u64;
    }
    n += store::upsert_filings(conn, &q.filings, now)? as u64;
    // Três anos de informes trimestrais (os anos baixados).
    conn.execute(
        "DELETE FROM fundamental WHERE source = 'cvm_fii_trimestral' AND period_end < ?1",
        [fmt_date(add_months(today, -36))],
    )?;
    Ok(n)
}

fn apply_ipe(conn: &Connection, docs: Vec<cvm::IpeDoc>, now: &str) -> Result<u64> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO corporate_document (id, issuer_id, company_name, category, doc_type, species, subject, reference_date,
                                         delivered_at, version, link, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'cvm_ipe', ?12)
         ON CONFLICT (id) DO UPDATE SET company_name = excluded.company_name, category = excluded.category,
             doc_type = excluded.doc_type, species = excluded.species, subject = excluded.subject,
             reference_date = excluded.reference_date, delivered_at = excluded.delivered_at,
             version = excluded.version, link = excluded.link, fetched_at = excluded.fetched_at",
    )?;
    let mut n = 0;
    for d in &docs {
        n += stmt.execute(params![
            d.id,
            d.cnpj,
            d.company,
            d.category,
            d.doc_type,
            d.species,
            d.subject,
            d.reference_date,
            d.delivered_at,
            d.version,
            d.link,
            now
        ])? as u64;
    }
    Ok(n)
}

fn apply_quotes(conn: &Connection, quotes: Vec<b3::Quote>, now: &str) -> Result<u64> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO research_quote (ticker, date, close, volume, trades, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'b3_cotahist', ?6)
         ON CONFLICT (ticker, date) DO UPDATE SET close = excluded.close, volume = excluded.volume,
             trades = excluded.trades, fetched_at = excluded.fetched_at",
    )?;
    // Nomes de pregão (para reconhecer o ativo nas notas de corretagem).
    crate::import::notes::store_trading_names(conn, &quotes)?;
    let mut n = 0;
    for q in &quotes {
        n += stmt.execute(params![
            q.ticker,
            fmt_date(q.date),
            super::dstr(q.close),
            super::dstr(q.volume),
            q.trades,
            now
        ])? as u64;
    }
    Ok(n)
}

fn apply_sec_tickers(
    conn: &Connection,
    map: HashMap<String, sec::TickerEntry>,
    wanted: &[String],
    now: &str,
) -> Result<u64> {
    let mut n = 0;
    for t in wanted {
        let Some(e) = map.get(t) else { continue };
        let id = sec::issuer_id(e.cik);
        store::upsert_issuer(
            conn,
            &super::Issuer {
                id: id.clone(),
                kind: IssuerKind::CompanyUs,
                name: e.title.clone(),
                cvm_code: None,
                sector: None,
                segment: None,
                mandate: None,
                layout: Layout::Industrial,
                currency: "USD".into(),
                source: "sec".into(),
            },
            now,
        )?;
        store::upsert_security(
            conn,
            &Security {
                ticker: t.clone(),
                issuer_id: id,
                class: "stock_us".into(),
                share_kind: Some("COMMON".into()),
                unit_composition: None,
                isin: None,
                listing_segment: None,
                active: true,
                source: "sec".into(),
            },
            now,
        )?;
        n += 1;
    }
    Ok(n)
}

/// Coleta completa (ou só as fontes pedidas) com a rede de verdade.
pub fn run(api: &crate::Api, args: &SyncArgs) -> Result<ResearchSyncReport> {
    use crate::tasks::{self, TaskKind, TaskOrigin};
    tasks::run(TaskKind::ResearchSync, TaskOrigin::User, true, || {
        run_with(api, args, &HttpFetcher::new(), today_local())
    })
}

/// Coleta com um `Fetcher` qualquer (testes usam fixtures).
pub fn run_with(
    api: &crate::Api,
    args: &SyncArgs,
    fetcher: &dyn Fetcher,
    today: NaiveDate,
) -> Result<ResearchSyncReport> {
    let plan = api.with_session(|s, _| plan(s.db.conn(), today, args))?;
    crate::tasks::stage(crate::tasks::TaskStage::Fundamentals);
    let mut r = Runner {
        api,
        fetcher,
        plan: &plan,
        rep: ResearchSyncReport {
            started_at: now_utc(),
            ok: true,
            ..Default::default()
        },
        refetch: false,
    };
    let on = |s: SyncSource| plan.sources.contains(&s);

    if on(SyncSource::Fca) {
        for y in &plan.fca_years {
            r.step(
                Req {
                    source: SyncSource::Fca,
                    label: format!("CVM FCA {y}"),
                    url: &cvm::fca_url(*y),
                    ua: "",
                    remember: true,
                },
                cvm::parse_fca,
                apply_fca,
            )?;
        }
    }
    // Emissores dos ativos cadastrados (o mapa ticker → CNPJ vem do FCA, logo acima) e
    // companhias com ações negociadas.
    let registered = r.with(|c| super::data::registered_issuers(c))?;
    let keep: HashSet<String> = r.with(|c| {
        let mut k: HashSet<String> = c
            .prepare("SELECT DISTINCT issuer_id FROM research_security WHERE class = 'stock_br' AND active = 1")?
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        k.extend(registered.iter().cloned());
        Ok(k)
    })?;
    let keep_all = keep.is_empty();
    let keep_fn = |c: &str| keep_all || keep.contains(c);
    // Emissor cadastrado sem nenhuma demonstração: baixa DFP/ITR de novo mesmo sem
    // mudança, uma vez por emissor (há emissores que não publicam DFP).
    let tried: HashSet<String> = r
        .with(|c| crate::settings::get::<Vec<String>>(c, REFETCH_SETTING))?
        .unwrap_or_default()
        .into_iter()
        .collect();
    let without_facts: Vec<String> = r.with(|c| {
        let mut stmt =
            c.prepare_cached("SELECT EXISTS (SELECT 1 FROM fundamental WHERE issuer_id = ?1)")?;
        let mut out = vec![];
        for i in &registered {
            if !tried.contains(i) && !stmt.query_row([i], |row| row.get::<_, bool>(0))? {
                out.push(i.clone());
            }
        }
        Ok(out)
    })?;
    r.refetch = !without_facts.is_empty() && (on(SyncSource::Dfp) || on(SyncSource::Itr));
    if r.refetch {
        let mut all: Vec<String> = tried.into_iter().chain(without_facts).collect();
        all.sort();
        r.with(|c| crate::settings::set(c, REFETCH_SETTING, &all))?;
    }
    let mut warnings: Vec<String> = vec![];
    for (src, kind, years) in [
        (SyncSource::Dfp, cvm::FilingKind::Dfp, &plan.dfp_years),
        (SyncSource::Itr, cvm::FilingKind::Itr, &plan.itr_years),
    ] {
        if !on(src) {
            continue;
        }
        for y in years {
            let url = match kind {
                cvm::FilingKind::Dfp => cvm::dfp_url(*y),
                cvm::FilingKind::Itr => cvm::itr_url(*y),
            };
            let mut w = vec![];
            r.step(
                Req {
                    source: src,
                    label: format!("CVM {} {y}", kind.label()),
                    url: &url,
                    ua: "",
                    remember: true,
                },
                |b| {
                    let f = cvm::parse_statements_full(b, kind, &keep_fn)?;
                    w = f.filings.iter().flat_map(|x| x.warnings.clone()).collect();
                    Ok(f)
                },
                apply_filings,
            )?;
            warnings.extend(w);
        }
    }
    r.refetch = false;
    if on(SyncSource::Fii) {
        for y in &plan.fii_years {
            r.step(
                Req {
                    source: SyncSource::Fii,
                    label: crate::i18n::t_args(
                        "nucleo-pesquisa-passo-fii-mensal",
                        &crate::i18n::args(&[("ano", y.to_string().into())]),
                    ),
                    url: &cvm::fii_url(*y),
                    ua: "",
                    remember: true,
                },
                cvm::parse_fii,
                |c, inf, now| apply_fii(c, inf, now, today),
            )?;
        }
    }
    if on(SyncSource::FiiQuarterly) {
        // Só os FIIs listados já conhecidos (vindos do informe mensal).
        let funds: HashSet<String> = r.with(|c| {
            Ok(
                c.prepare("SELECT id FROM research_issuer WHERE kind = 'fii'")?
                    .query_map([], |row| row.get(0))?
                    .collect::<rusqlite::Result<_>>()?,
            )
        })?;
        for y in &plan.fii_quarterly_years {
            r.step(
                Req {
                    source: SyncSource::FiiQuarterly,
                    label: crate::i18n::t_args(
                        "nucleo-pesquisa-passo-fii-trimestral",
                        &crate::i18n::args(&[("ano", y.to_string().into())]),
                    ),
                    url: &cvm::fii_quarterly_url(*y),
                    ua: "",
                    remember: true,
                },
                |b| cvm::parse_fii_quarterly(b, &|c| funds.contains(c)),
                |c, q, now| apply_fii_quarterly(c, q, now, today),
            )?;
        }
    }
    if on(SyncSource::Ipe) {
        // Todas as companhias com ação negociada (o explorador e a página de um papel fora
        // do cadastro também mostram documentos), mais os emissores cadastrados.
        let keep_fn = &keep_fn;
        for y in &plan.ipe_years {
            r.step(
                Req {
                    source: SyncSource::Ipe,
                    label: format!("CVM IPE {y}"),
                    url: &cvm::ipe_url(*y),
                    ua: "",
                    remember: false,
                },
                |b| cvm::parse_ipe(b, &|c| keep_fn(c)),
                apply_ipe,
            )?;
        }
    }
    if on(SyncSource::B3) {
        for d in &plan.quote_dates {
            r.step(
                Req {
                    source: SyncSource::B3,
                    label: format!("B3 COTAHIST {d}"),
                    url: &b3::daily_url(*d),
                    ua: "",
                    remember: false,
                },
                b3::parse_zip,
                apply_quotes,
            )?;
        }
        for (y, mo) in &plan.quote_months {
            let st = r.step(
                Req {
                    source: SyncSource::B3,
                    label: format!("B3 COTAHIST {mo:02}/{y}"),
                    url: &b3::monthly_url(*y, *mo),
                    ua: "",
                    remember: true,
                },
                b3::parse_zip,
                apply_quotes,
            )?;
            if matches!(st, SyncStatus::Updated | SyncStatus::Unchanged) {
                r.with(|c| {
                    let mut done: Vec<String> =
                        crate::settings::get(c, QUOTE_MONTHS_SETTING)?.unwrap_or_default();
                    done.push(format!("{y:04}-{mo:02}"));
                    done.sort();
                    done.dedup();
                    // Só os meses ainda dentro da janela.
                    let floor = add_months(today, -14);
                    done.retain(|m| {
                        m.as_str() >= format!("{:04}-{:02}", floor.year(), floor.month()).as_str()
                    });
                    crate::settings::set(c, QUOTE_MONTHS_SETTING, &done)
                })?;
            }
        }
        r.with(|c| {
            let last: Option<String> = c
                .query_row("SELECT max(date) FROM research_quote", [], |row| row.get(0))
                .optional()?
                .flatten();
            if let Some(l) = last {
                let cut = parse_date(&l)? - chrono::Duration::days(QUOTE_HISTORY_DAYS);
                c.execute(
                    "DELETE FROM research_quote WHERE date < ?1",
                    [fmt_date(cut)],
                )?;
            }
            Ok(())
        })?;
    }
    if on(SyncSource::Brapi) {
        let (token, due) = r.with(|c| {
            Ok((
                super::brapi::token(c)?,
                super::brapi::due_tickers(c, chrono::Utc::now())?,
            ))
        })?;
        if let Some(token) = token {
            let auth = format!("Bearer {}", token.trim());
            // Plano e consumo da conta (a cada poucas horas): a conferência usa módulos pagos.
            let mut st = r.with(|c| super::brapi::state(c))?;
            if st.stale(chrono::Utc::now()) {
                match super::brapi::fetch_usage(r.fetcher, &token) {
                    Ok(u) => st = r.with(|c| super::brapi::store_usage(c, &u, &now_utc()))?,
                    Err(e) => tracing::warn!("consumo da brapi: {e}"),
                }
            }
            let skip = |label: String| SyncItem {
                source: SyncSource::Brapi,
                label,
                url: None,
                status: SyncStatus::Skipped,
                bytes: 0,
                rows: 0,
                error: None,
            };
            let due = if !st.paid() {
                if !due.is_empty() {
                    r.rep
                        .items
                        .push(skip(crate::i18n::t("nucleo-pesquisa-brapi-plano-gratuito")));
                }
                vec![]
            } else {
                due
            };
            for t in due {
                if r.with(|c| super::brapi::budget(c))? <= 0 {
                    r.rep
                        .items
                        .push(skip(crate::i18n::t("nucleo-pesquisa-brapi-limite")));
                    break;
                }
                let url = super::brapi::quote_url(&t);
                let mut item = SyncItem {
                    source: SyncSource::Brapi,
                    label: format!("brapi.dev {t}"),
                    url: Some(url.clone()),
                    status: SyncStatus::Error,
                    bytes: 0,
                    rows: 0,
                    error: None,
                };
                let now = now_utc();
                let res = r
                    .fetcher
                    .get_with(&url, &[("Authorization", auth.as_str())], "");
                r.with(|c| super::brapi::count(c, 1))?;
                let stop = matches!(&res, Err(e) if e.contains("token"));
                match res {
                    Ok(Fetched::Body { bytes, .. }) => {
                        item.bytes = bytes.len() as u64;
                        r.rep.bytes += item.bytes;
                        match super::brapi::parse(&t, &bytes, &now) {
                            Ok(snap) => {
                                r.with(|c| super::brapi::store(c, &snap))?;
                                item.rows = snap.metrics.len() as u64;
                                item.status = SyncStatus::Updated;
                            }
                            Err(e) => {
                                r.with(|c| super::brapi::store_failure(c, &t, &now, "error", &e))?;
                                item.error = Some(e);
                            }
                        }
                    }
                    Ok(Fetched::NotFound) | Ok(Fetched::NotModified) => {
                        r.with(|c| super::brapi::store_failure(c, &t, &now, "not_found", ""))?;
                        item.status = SyncStatus::NotFound;
                    }
                    Err(e) => {
                        r.with(|c| super::brapi::store_failure(c, &t, &now, "error", &e))?;
                        item.error = Some(e);
                    }
                }
                if item.error.is_some() {
                    r.rep.ok = false;
                }
                r.rep.items.push(item);
                if stop {
                    // Token recusado: não insiste nos outros papéis.
                    break;
                }
            }
        }
    }
    if on(SyncSource::B3Listados) {
        super::b3_listados::sync(&mut r, today)?;
    }
    if on(SyncSource::Sec) && !plan.us_tickers.is_empty() {
        let ua = plan.sec_user_agent.clone();
        let wanted = plan.us_tickers.clone();
        let default_ua = super::config::ResearchSettings {
            sec_user_agent: ua.clone(),
            ..Default::default()
        }
        .sec_user_agent_is_default();
        if default_ua {
            // Sem e-mail a SEC responde 403 a tudo: não insiste (nem conta como falha da fonte).
            warnings.push(super::config::sec_hint());
            r.rep.items.push(SyncItem {
                source: SyncSource::Sec,
                label: crate::i18n::t("nucleo-pesquisa-passo-sec-email"),
                url: None,
                status: SyncStatus::Skipped,
                bytes: 0,
                rows: 0,
                error: Some(super::config::sec_hint()),
            });
        }
        if !default_ua {
            r.step(
                Req {
                    source: SyncSource::Sec,
                    label: "SEC company_tickers.json".into(),
                    url: sec::TICKERS_URL,
                    ua: &ua,
                    remember: true,
                },
                sec::parse_company_tickers,
                |c, map, now| apply_sec_tickers(c, map, &wanted, now),
            )?;
            let ciks: Vec<(String, u64)> = r.with(|c| {
                let mut out = vec![];
                for t in &wanted {
                    if let Some(s) = store::security(c, t)?.filter(|s| s.class == "stock_us")
                        && let Some(cik) = s
                            .issuer_id
                            .strip_prefix("cik:")
                            .and_then(|x| x.parse().ok())
                    {
                        out.push((t.clone(), cik));
                    }
                }
                Ok(out)
            })?;
            for t in &wanted {
                if !ciks.iter().any(|(x, _)| x == t) && !plan.bdr_candidates.contains(t) {
                    r.rep.items.push(SyncItem {
                        source: SyncSource::Sec,
                        label: format!("SEC {t}"),
                        url: None,
                        status: SyncStatus::Skipped,
                        bytes: 0,
                        rows: 0,
                        error: Some(crate::i18n::t_args(
                            "nucleo-pesquisa-erro-sec-sem-ticker",
                            &crate::i18n::args(&[("ticker", t.clone().into())]),
                        )),
                    });
                }
            }
            for (t, cik) in ciks {
                let st = r.step(
                    Req {
                        source: SyncSource::Sec,
                        label: format!("SEC companyfacts {t}"),
                        url: &sec::facts_url(cik),
                        ua: &ua,
                        remember: true,
                    },
                    sec::parse_companyfacts,
                    |c, (name, facts), now| {
                        let id = sec::issuer_id(cik);
                        if !name.is_empty() {
                            c.execute(
                                "UPDATE research_issuer SET name = ?2 WHERE id = ?1",
                                params![id, name],
                            )?;
                        }
                        Ok(store::upsert_facts(c, &id, &facts, now)? as u64)
                    },
                )?;
                if st == SyncStatus::Error
                    && r.rep
                        .items
                        .last()
                        .and_then(|i| i.error.as_deref())
                        .is_some_and(|e| e.contains("403"))
                {
                    warnings.push(super::config::sec_hint());
                }
                // Setor (SIC) para os pares e o segmento do score de saúde.
                r.step(
                Req {
                    source: SyncSource::Sec,
                    label: crate::i18n::t_args("nucleo-pesquisa-passo-sec-cadastro", &crate::i18n::args(&[("ticker", t.clone().into())])),
                    url: &sec::submissions_url(cik),
                    ua: &ua,
                    remember: true,
                },
                sec::parse_submissions,
                |c, (sic, desc), _now| {
                    Ok(c.execute(
                        "UPDATE research_issuer SET sector = coalesce(?2, sector), segment = coalesce(?3, segment) WHERE id = ?1",
                        params![sec::issuer_id(cik), desc, sic],
                    )? as u64)
                },
            )?;
            }
        }
    }
    if on(SyncSource::Ifdata) {
        sync_ifdata(&mut r, today)?;
    }
    if on(SyncSource::Funds) {
        crate::score::funds::sync::run(&mut r, today)?;
    }
    if on(SyncSource::Crypto) {
        crate::score::crypto::sync::run(&mut r, today)?;
    }
    // Derivados: LTM, universo do dia e radar de dividendos.
    r.with(|c| {
        let tx = c.transaction()?;
        recompute_ltm(&tx)?;
        tx.commit()?;
        Ok(())
    })?;
    let bazin = r.with(|c| super::config::load(c).map(|s| s.bazin()))?;
    r.rep.universe = r.with(|c| {
        let tx = c.transaction()?;
        let n = super::screen::recompute_universe(&tx, today, bazin)?;
        tx.commit()?;
        Ok(n as u64)
    })?;
    // Score em lote do universo (explorador de mercado): uma falha aqui não derruba a coleta.
    match crate::score::market::run(|f| r.with(|c| f(c)), today) {
        Ok(m) => r.rep.market_scores = Some(u64::from(m.scored)),
        Err(e) => warnings.push(crate::i18n::t_args(
            "nucleo-pesquisa-aviso-score-lote",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )),
    }
    // Alertas das telas salvas com aviso ligado (papel novo que passou a cumprir).
    if let Err(e) = r.with(|c| super::screen_alerts::run(c, today, bazin).map(|_| ())) {
        warnings.push(crate::i18n::t_args(
            "nucleo-pesquisa-aviso-alerta-tela",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        ));
    }
    r.rep.radar = r.with(|c| {
        let ids = super::data::held_or_watched(c)?;
        let res = super::radar::run(c, &ids, today)?;
        Ok(res.written)
    })?;
    warnings.sort();
    warnings.dedup();
    // Correções de escala das ações viram um aviso só.
    const SCALE: &str = ": quantidade de ações informada em milhares";
    let (scaled, mut other): (Vec<String>, Vec<String>) =
        warnings.into_iter().partition(|w| w.contains(SCALE));
    if !scaled.is_empty() {
        let mut names: Vec<String> = scaled
            .iter()
            .filter_map(|w| w.split(SCALE).next().map(str::to_string))
            .collect();
        names.dedup();
        let shown: Vec<String> = names.iter().take(5).cloned().collect();
        other.push(crate::i18n::t_args(
            "nucleo-pesquisa-aviso-acoes-em-milhares",
            &crate::i18n::args(&[
                ("n", (names.len() as i64).into()),
                (
                    "companhias",
                    format!(
                        "{}{}",
                        shown.join(", "),
                        if names.len() > 5 { ", …" } else { "" }
                    )
                    .into(),
                ),
            ]),
        ));
    }
    r.rep.warnings = other;
    r.rep.finished_at = now_utc();
    let rep = r.rep;
    // Releitura completa (todas as fontes do parser, sem erro): grava a versão nova.
    let reparsed = plan.record_parser
        || plan.reparse_all
            && !rep
                .items
                .iter()
                .any(|i| plan.reparse.contains(&i.source) && i.status == SyncStatus::Error);
    // Cancelada: a data da última coleta fica como estava, para a próxima continuar daqui.
    crate::tasks::check()?;
    api.with_session(|s, _| {
        if reparsed {
            crate::settings::set(s.db.conn(), PARSER_VERSION_SETTING, &PARSER_VERSION)?;
        }
        crate::settings::set(
            s.db.conn(),
            super::config::LAST_SYNC_SETTING,
            &rep.finished_at,
        )?;
        crate::settings::set(s.db.conn(), super::config::LAST_REPORT_SETTING, &rep)?;
        Ok(())
    })?;
    Ok(rep)
}

/// Quando o trimestre mais recente do IF.data foi sondado pela última vez (UTC RFC 3339).
const IFDATA_PROBE_SETTING: &str = "research.ifdata_probe_at";
/// Sondagem do trimestre novo do IF.data no máximo a cada 20 horas.
const IFDATA_PROBE_HOURS: i64 = 20;

fn older_than(at: &str, hours: i64) -> bool {
    chrono::DateTime::parse_from_rfc3339(at)
        .map(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() >= hours)
        .unwrap_or(true)
}

/// Banco Central IF.data: acha o trimestre mais recente (sondando do trimestre corrente para
/// trás; os dados saem uns 3 meses depois), baixa o cadastro e os relatórios 1, 5 e 16
/// completos do trimestre mais recente e dos 4 anteriores, e os relatórios 2 e 4 só para as
/// instituições de interesse. Trimestres passados não mudam: só se baixa o que falta; o mais
/// recente, de novo no máximo uma vez por semana. Um pedido por vez, sem travar o banco.
fn sync_ifdata(r: &mut Runner<'_>, today: NaiveDate) -> Result<()> {
    use super::ifdata as ifd;
    let force = r.plan.force;
    let done = r.with(|c| ifd::fetched(c))?;
    let known = done
        .keys()
        .filter(|(_, rep)| rep == ifd::CADASTRO)
        .map(|(p, _)| p.clone())
        .max();
    let current = ifd::quarter_end(today);
    let mut latest = known.clone();
    let may_probe = force
        || r.plan
            .ifdata_probe_at
            .as_deref()
            .is_none_or(|t| older_than(t, IFDATA_PROBE_HOURS));
    if known.as_deref() != Some(current.as_str()) && may_probe {
        for p in ifd::quarters_back(&current, 4) {
            if known.as_deref().is_some_and(|k| p.as_str() <= k) {
                break;
            }
            let url = ifd::cadastro_url(&p, true);
            let res = r.fetcher.get(&url, None, "");
            let delay = r.fetcher.polite_delay(&url);
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
            match res.map(|f| match f {
                Fetched::Body { bytes, .. } => ifd::parse_probe(&bytes),
                _ => Ok(false),
            }) {
                Ok(Ok(true)) => {
                    latest = Some(p);
                    break;
                }
                Ok(Ok(false)) => {}
                Ok(Err(e)) | Err(e) => {
                    r.rep.ok = false;
                    r.rep.items.push(SyncItem {
                        source: SyncSource::Ifdata,
                        label: crate::i18n::t_args(
                            "nucleo-pesquisa-passo-ifdata-sondagem",
                            &crate::i18n::args(&[("periodo", p.to_string().into())]),
                        ),
                        url: Some(url),
                        status: SyncStatus::Error,
                        bytes: 0,
                        rows: 0,
                        error: Some(e),
                    });
                    break;
                }
            }
        }
        let now = now_utc();
        r.with(|c| crate::settings::set(c, IFDATA_PROBE_SETTING, &now))?;
    }
    let Some(latest) = latest else {
        r.rep.items.push(SyncItem {
            source: SyncSource::Ifdata,
            label: crate::i18n::t("nucleo-pesquisa-passo-ifdata"),
            url: None,
            status: SyncStatus::NotFound,
            bytes: 0,
            rows: 0,
            error: None,
        });
        return Ok(());
    };
    let periods = ifd::quarters_back(&latest, ifd::QUARTERS);
    // Pedido a refazer: nunca feito, forçado, ou o trimestre mais recente com mais de 7 dias.
    let stale = |p: &str, rep: &str| match done.get(&(p.to_string(), rep.to_string())) {
        None => true,
        Some(f) => force || (p == latest && older_than(&f.fetched_at, ifd::REFRESH_DAYS * 24)),
    };
    let ifdata = crate::i18n::t("nucleo-pesquisa-passo-ifdata");
    let label = |rep: &str, p: &str| format!("{ifdata} {} {p}", ifd::report_label(rep));
    if stale(&latest, ifd::CADASTRO) {
        let p = latest.clone();
        r.step(
            Req {
                source: SyncSource::Ifdata,
                label: label(ifd::CADASTRO, &latest),
                url: &ifd::cadastro_url(&latest, false),
                ua: "",
                remember: false,
            },
            ifd::parse_cadastro,
            |c, rows, now| ifd::apply_cadastro(c, &p, rows, now),
        )?;
    }
    for p in &periods {
        for rep in ifd::FULL_REPORTS {
            if !stale(p, rep) {
                continue;
            }
            r.step(
                Req {
                    source: SyncSource::Ifdata,
                    label: label(rep, p),
                    url: &ifd::values_url(p, rep, None),
                    ua: "",
                    remember: false,
                },
                |b| ifd::parse_values(b, rep),
                |c, vals, now| ifd::apply_values(c, p, rep, None, vals, now),
            )?;
        }
    }
    let cnpj8 = r.with(|c| ifd::target_cnpj8(c))?;
    for p in &periods {
        let targets = r.with(|c| ifd::target_codes(c, &cnpj8, p))?;
        for rep in ifd::FILTERED_REPORTS {
            let missing: Vec<String> = match done.get(&(p.clone(), rep.to_string())) {
                Some(f) if !stale(p, rep) => {
                    let have: HashSet<&String> = f.codes.iter().flatten().collect();
                    targets
                        .iter()
                        .filter(|c| !have.contains(c))
                        .cloned()
                        .collect()
                }
                _ => targets.clone(),
            };
            let chunks = ifd::chunk_codes(p, rep, &missing);
            let n = chunks.len();
            for (i, codes) in chunks.into_iter().enumerate() {
                let part = if n > 1 {
                    format!(
                        " ({})",
                        crate::i18n::t_args(
                            "nucleo-pesquisa-passo-parte",
                            &crate::i18n::args(&[
                                ("parte", ((i + 1) as i64).into()),
                                ("partes", (n as i64).into()),
                            ]),
                        )
                    )
                } else {
                    String::new()
                };
                r.step(
                    Req {
                        source: SyncSource::Ifdata,
                        label: format!("{}{part}", label(rep, p)),
                        url: &ifd::values_url(p, rep, Some(&codes)),
                        ua: "",
                        remember: false,
                    },
                    |b| ifd::parse_values(b, rep),
                    |c, vals, now| ifd::apply_values(c, p, rep, Some(&codes), vals, now),
                )?;
            }
        }
    }
    if let Some(oldest) = periods.last() {
        r.with(|c| ifd::prune(c, oldest))?;
    }
    Ok(())
}

/// Emissores cadastrados para os quais a DFP/ITR já foi baixada de novo por falta de dados.
const REFETCH_SETTING: &str = "research.refetched_issuers";

/// Rotina do app: coleta se a última tiver mais de `min_hours`.
pub fn run_if_due(api: &crate::Api, min_hours: i64) -> Result<Option<ResearchSyncReport>> {
    if !api.is_unlocked() {
        return Ok(None);
    }
    let last: Option<String> = api
        .with_session(|s, _| crate::settings::get(s.db.conn(), super::config::LAST_SYNC_SETTING))?;
    let due = match last
        .as_deref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
    {
        Some(t) => (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() >= min_hours,
        None => true,
    };
    // Cancelada há pouco: espera antes de recomeçar sozinha (docs/background-tasks.md).
    if !due || crate::tasks::snoozed(crate::tasks::TaskKind::ResearchSync) {
        return Ok(None);
    }
    use crate::tasks::{self, TaskKind, TaskOrigin};
    let rep = tasks::run(TaskKind::ResearchSync, TaskOrigin::Auto, true, || {
        run(api, &SyncArgs::default())
    });
    // O que veio antes do cancelamento já está gravado: as telas abertas se atualizam.
    crate::events::emit_system("research.sync");
    rep.map(Some)
}
