//! Provedores de séries fora do Brasil (docs/data-sources.md): BCE, Eurostat, OCDE,
//! Bank of Canada, ONS e Bank of England (Reino Unido) e Banco Mundial. Todos públicos e
//! sem chave.
//!
//! Cada série guarda em `index_series.source` o provedor e o código da consulta
//! (`ecb:EXR/D.USD.EUR.SP00.A`, `oecd:<fluxo>/<chave>`, `worldbank:PT`…). Os leitores de
//! resposta são funções puras com testes de fixture; a rede fica em [`fetch`].

pub mod boc;
pub mod eurostat;
pub mod nyfed;
pub mod sdmx;
pub mod uk;
pub mod worldbank;

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::ids::new_id;
use crate::time::{fmt_date, now_utc, parse_date};

/// Provedor de dados (prefixo de `index_series.source`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DataProvider {
    BcbSgs,
    Fred,
    Yahoo,
    Ecb,
    Eurostat,
    Oecd,
    Boc,
    Ons,
    Boe,
    WorldBank,
    /// IBGE, API do SIDRA (IPCA por grupo).
    IbgeSidra,
    /// Federal Reserve Bank of New York (EFFR, sem chave).
    NyFed,
}

/// Descrição de um provedor para a interface e a documentação.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataProviderInfo {
    pub id: DataProvider,
    pub name: String,
    pub homepage: String,
    /// Termos de uso, em uma frase.
    pub terms: String,
    pub key_required: bool,
}

impl DataProvider {
    pub const ALL: [DataProvider; 12] = [
        DataProvider::BcbSgs,
        DataProvider::Fred,
        DataProvider::Yahoo,
        DataProvider::Ecb,
        DataProvider::Eurostat,
        DataProvider::Oecd,
        DataProvider::Boc,
        DataProvider::Ons,
        DataProvider::Boe,
        DataProvider::WorldBank,
        DataProvider::IbgeSidra,
        DataProvider::NyFed,
    ];

    /// Prefixo de `source` (antes do primeiro `:`).
    pub fn prefix(self) -> &'static str {
        match self {
            DataProvider::BcbSgs => "bcb_sgs",
            DataProvider::Fred => "fred",
            DataProvider::Yahoo => "yahoo",
            DataProvider::Ecb => "ecb",
            DataProvider::Eurostat => "eurostat",
            DataProvider::Oecd => "oecd",
            DataProvider::Boc => "boc",
            DataProvider::Ons => "ons",
            DataProvider::Boe => "boe",
            DataProvider::WorldBank => "worldbank",
            DataProvider::IbgeSidra => "ibge_sidra",
            DataProvider::NyFed => "nyfed",
        }
    }

    /// Provedor e código de uma `source` (`ecb:EXR/D.USD.EUR.SP00.A` → (Ecb, "EXR/…")).
    pub fn of_source(source: &str) -> Option<(DataProvider, &str)> {
        let (head, code) = source.split_once(':')?;
        DataProvider::ALL
            .into_iter()
            .find(|p| p.prefix() == head)
            .map(|p| (p, code))
    }

    /// Liga/desliga em Ajustes › Fontes de dados.
    pub fn source(self) -> crate::sources::SourceId {
        use crate::sources::SourceId as S;
        match self {
            DataProvider::BcbSgs => S::Bcb,
            DataProvider::Fred => S::Fred,
            DataProvider::Yahoo => S::Yahoo,
            DataProvider::Ecb => S::Ecb,
            DataProvider::Eurostat => S::Eurostat,
            DataProvider::Oecd => S::Oecd,
            DataProvider::Boc => S::Boc,
            DataProvider::Ons => S::Ons,
            DataProvider::Boe => S::Boe,
            DataProvider::WorldBank => S::WorldBank,
            DataProvider::IbgeSidra => S::Ibge,
            DataProvider::NyFed => S::NyFed,
        }
    }

    /// Provedores que já existiam antes do catálogo de países: coletados sempre, como antes.
    pub fn is_legacy(self) -> bool {
        matches!(
            self,
            DataProvider::BcbSgs
                | DataProvider::Fred
                | DataProvider::Yahoo
                | DataProvider::IbgeSidra
        )
    }

    pub fn info(self) -> DataProviderInfo {
        let (homepage, key) = match self {
            DataProvider::BcbSgs => ("https://dadosabertos.bcb.gov.br", false),
            DataProvider::Fred => ("https://fred.stlouisfed.org", true),
            DataProvider::Yahoo => ("https://finance.yahoo.com", false),
            DataProvider::Ecb => ("https://data.ecb.europa.eu", false),
            DataProvider::Eurostat => ("https://ec.europa.eu/eurostat", false),
            DataProvider::Oecd => ("https://data-explorer.oecd.org", false),
            DataProvider::Boc => ("https://www.bankofcanada.ca/valet/docs", false),
            DataProvider::Ons => ("https://www.ons.gov.uk", false),
            DataProvider::Boe => ("https://www.bankofengland.co.uk/boeapps/database", false),
            DataProvider::WorldBank => ("https://data.worldbank.org", false),
            DataProvider::IbgeSidra => ("https://sidra.ibge.gov.br", false),
            DataProvider::NyFed => ("https://markets.newyorkfed.org", false),
        };
        // Nome e termos de uso no idioma da instalação (`nucleo-ativos-provedor-<prefixo>-…`).
        let text = |what: &str| {
            crate::i18n::t(&format!("nucleo-ativos-provedor-{}-{what}", self.prefix()))
        };
        DataProviderInfo {
            id: self,
            name: text("nome"),
            homepage: homepage.into(),
            terms: text("termos"),
            key_required: key,
        }
    }

    /// Fonte gravada nas taxas de câmbio espelhadas em `fx_rate`.
    pub fn fx_source(self) -> &'static str {
        match self {
            DataProvider::Ecb => "ecb_ref",
            DataProvider::Boc => "boc",
            DataProvider::Boe => "boe",
            _ => self.prefix(),
        }
    }
}

/// Séries mensais dos provedores novos são baixadas inteiras a cada coleta: são pequenas
/// e assim uma mudança de ano-base (o HICP passou a 2025=100 em 2026) não mistura bases.
pub fn full_refresh(source: &str, frequency: &str) -> bool {
    match DataProvider::of_source(source) {
        Some((p, _)) if !p.is_legacy() => frequency == "monthly",
        _ => false,
    }
}

/// Resposta do SIDRA (`/values/…`): a primeira linha é o cabeçalho; `D3C` é o período
/// (`AAAAMM`) e `V` o valor ("..." ou "-" quando não há).
pub fn parse_sidra(body: &str) -> Result<Vec<(NaiveDate, Decimal)>> {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(body).map_err(|e| bad_response("SIDRA", e))?;
    let mut out = vec![];
    for r in rows.iter().skip(1) {
        let (Some(p), Some(v)) = (r["D3C"].as_str(), r["V"].as_str()) else {
            continue;
        };
        let (Ok(y), Ok(m)) = (
            p.get(..4).unwrap_or("").parse::<i32>(),
            p.get(4..6).unwrap_or("").parse::<u32>(),
        ) else {
            continue;
        };
        let (Some(d), Ok(val)) = (
            NaiveDate::from_ymd_opt(y, m, 1),
            v.trim().parse::<Decimal>(),
        ) else {
            continue;
        };
        out.push((d, val));
    }
    out.sort_by_key(|(d, _)| *d);
    Ok(out)
}

/// Valores baixados: (data, valor, estimado).
pub type Values = Vec<(NaiveDate, Decimal, bool)>;

/// Baixa uma série de um provedor novo. Não toca no banco.
pub fn fetch(
    agent: &ureq::Agent,
    source: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> std::result::Result<Values, String> {
    let Some((provider, code)) = DataProvider::of_source(source) else {
        return Err(crate::i18n::t_args(
            "nucleo-ativos-erro-fonte-desconhecida",
            &crate::i18n::args(&[("fonte", source.to_string().into())]),
        ));
    };
    let get = |url: &str| super::sync::get_text(agent, url);
    let official = |v: Vec<(NaiveDate, Decimal)>| -> Values {
        v.into_iter()
            .filter(|(d, _)| *d >= month_start(from) && *d <= to)
            .map(|(d, v)| (d, v, false))
            .collect()
    };
    let parse_err = |e: Error| e.to_string();
    match provider {
        DataProvider::Ecb => {
            let url = format!(
                "https://data-api.ecb.europa.eu/service/data/{code}?startPeriod={}&endPeriod={}&format=csvdata&detail=dataonly",
                fmt_date(from),
                fmt_date(to)
            );
            match get(&url) {
                Ok(body) => sdmx::parse_csv(&body).map(official).map_err(parse_err),
                // Sem observações no intervalo (fim de semana, mês ainda não publicado).
                Err(e) if e.contains("404") => Ok(vec![]),
                Err(e) => Err(e),
            }
        }
        DataProvider::Oecd => {
            // Alternativas separadas por `|` (o fluxo COICOP 2018 substitui o antigo país a
            // país): fica a que chega mais longe.
            let mut found: Vec<Vec<(NaiveDate, Decimal)>> = vec![];
            let mut last_err = None;
            for alt in code.split('|') {
                let url = format!(
                    "https://sdmx.oecd.org/public/rest/data/{alt}?startPeriod={}&format=csvfile",
                    from.format("%Y-%m")
                );
                match get(&url).and_then(|b| sdmx::parse_csv(&b).map_err(parse_err)) {
                    Ok(v) => found.push(v),
                    // NoRecordsFound: o país não está (ou não está mais) neste fluxo.
                    Err(e) if e.contains("404") => found.push(vec![]),
                    Err(e) => last_err = Some(e),
                }
            }
            let last = |v: &Vec<(NaiveDate, Decimal)>| v.last().map(|(d, _)| *d);
            match found
                .into_iter()
                .reduce(|a, b| if last(&b) > last(&a) { b } else { a })
            {
                Some(v) => Ok(official(v)),
                None => Err(last_err
                    .unwrap_or_else(|| crate::i18n::t("nucleo-ativos-erro-oecd-sem-resposta"))),
            }
        }
        DataProvider::Eurostat => {
            let (dataset, query) = code.split_once('?').unwrap_or((code, ""));
            let url = format!(
                "https://ec.europa.eu/eurostat/api/dissemination/statistics/1.0/data/{dataset}?{query}&sinceTimePeriod={}",
                from.format("%Y-%m")
            );
            get(&url)
                .and_then(|b| eurostat::parse_json(&b).map_err(parse_err))
                .map(official)
        }
        DataProvider::Boc => {
            let url = format!(
                "https://www.bankofcanada.ca/valet/observations/{code}/json?start_date={}&end_date={}",
                fmt_date(from),
                fmt_date(to)
            );
            get(&url)
                .and_then(|b| boc::parse_valet(&b, code).map_err(parse_err))
                .map(official)
        }
        DataProvider::Ons => {
            let (dataset, cdid) = code.split_once('/').ok_or_else(|| {
                crate::i18n::t_args(
                    "nucleo-ativos-erro-codigo-ons",
                    &crate::i18n::args(&[("codigo", code.to_string().into())]),
                )
            })?;
            let url = format!(
                "https://www.ons.gov.uk/economy/inflationandpriceindices/timeseries/{cdid}/{dataset}/data"
            );
            get(&url)
                .and_then(|b| uk::parse_ons(&b).map_err(parse_err))
                .map(official)
        }
        DataProvider::Boe => {
            let url = format!(
                "https://www.bankofengland.co.uk/boeapps/database/_iadb-fromshowcolumns.asp?csv.x=yes&Datefrom={}&Dateto=now&SeriesCodes={code}&CSVF=TN&UsingCodes=Y&VPD=Y&VFD=N",
                from.format("%d/%b/%Y")
            );
            get(&url)
                .and_then(|b| uk::parse_boe_csv(&b).map_err(parse_err))
                .map(official)
        }
        DataProvider::WorldBank => {
            let url = format!(
                "https://api.worldbank.org/v2/country/{code}/indicator/FP.CPI.TOTL.ZG?format=json&per_page=20000"
            );
            // Anual → mensal: todos os pontos são estimados, e a série vem inteira.
            get(&url)
                .and_then(|b| worldbank::parse_json(&b).map_err(parse_err))
                .map(|annual| {
                    worldbank::annual_to_monthly(&annual)
                        .into_iter()
                        .map(|(d, v)| (d, v, true))
                        .collect()
                })
        }
        DataProvider::IbgeSidra => {
            // `tabela/categoria` da variação mensal (v63) do Brasil, por período AAAAMM. A
            // tabela 7060 (IPCA por grupo) começa em 01/2020.
            let (table, cat) = code.split_once('/').ok_or_else(|| {
                crate::i18n::t_args(
                    "nucleo-ativos-erro-codigo-sidra",
                    &crate::i18n::args(&[("codigo", code.to_string().into())]),
                )
            })?;
            let start = from.max(NaiveDate::from_ymd_opt(2020, 1, 1).expect("data"));
            if start > to {
                return Ok(vec![]);
            }
            let url = format!(
                "https://apisidra.ibge.gov.br/values/t/{table}/n1/all/v/63/p/{}-{}/c315/{cat}",
                start.format("%Y%m"),
                to.format("%Y%m")
            );
            get(&url)
                .and_then(|b| parse_sidra(&b).map_err(parse_err))
                .map(official)
        }
        DataProvider::NyFed => {
            // Taxas diárias por intervalo; a API aceita um ano por vez sem paginar.
            let mut all = vec![];
            let mut start = from;
            while start <= to {
                let end = crate::time::add_months(start, 12).min(to);
                let url = format!(
                    "https://markets.newyorkfed.org/api/rates/unsecured/{code}/search.json?startDate={}&endDate={}",
                    fmt_date(start),
                    fmt_date(end)
                );
                all.extend(get(&url).and_then(|b| nyfed::parse_rates(&b).map_err(parse_err))?);
                start = end.succ_opt().expect("data válida");
            }
            Ok(official(sorted(all)))
        }
        DataProvider::BcbSgs | DataProvider::Fred | DataProvider::Yahoo => {
            Err(crate::i18n::t_args(
                "nucleo-ativos-erro-adaptador-proprio",
                &crate::i18n::args(&[("fonte", provider.prefix().into())]),
            ))
        }
    }
}

fn month_start(d: NaiveDate) -> NaiveDate {
    use chrono::Datelike;
    d.with_day(1).expect("dia 1")
}

/// Data de período SDMX/Eurostat: `2026-10-13`, `2026-09`, `2026-Q3`, `2026M09`, `2026`.
/// Mensais e trimestrais ficam no 1º dia do (primeiro) mês, como as séries do SGS.
pub fn parse_period(s: &str) -> Result<NaiveDate> {
    let s = s.trim();
    let bad = || {
        Error::invalid(crate::i18n::t_args(
            "nucleo-ativos-erro-periodo",
            &crate::i18n::args(&[("periodo", s.to_string().into())]),
        ))
    };
    if s.len() == 10 {
        return parse_date(s);
    }
    let ym = |y: &str, m: u32| -> Result<NaiveDate> {
        let y: i32 = y.parse().map_err(|_| bad())?;
        NaiveDate::from_ymd_opt(y, m, 1).ok_or_else(bad)
    };
    if let Some((y, q)) = s.split_once("-Q") {
        let q: u32 = q.parse().map_err(|_| bad())?;
        if !(1..=4).contains(&q) {
            return Err(bad());
        }
        return ym(y, (q - 1) * 3 + 1);
    }
    if let Some((y, m)) = s.split_once('-').or_else(|| s.split_once('M')) {
        return ym(y, m.parse().map_err(|_| bad())?);
    }
    if s.len() == 4 {
        return ym(s, 1);
    }
    Err(bad())
}

/// Número em texto → decimal; vazio, `NaN` e `.` contam como sem valor.
pub fn parse_value(s: &str) -> Result<Option<Decimal>> {
    let t = s.trim();
    if t.is_empty() || t == "." || t.eq_ignore_ascii_case("nan") || t.eq_ignore_ascii_case("na") {
        return Ok(None);
    }
    t.parse::<Decimal>()
        .or_else(|_| Decimal::from_scientific(t))
        .map(Some)
        .map_err(|_| invalid_number(s))
}

/// "número inválido: …" no idioma da instalação.
pub(crate) fn invalid_number(s: &str) -> Error {
    Error::invalid(crate::i18n::t_args(
        "nucleo-ativos-erro-numero",
        &crate::i18n::args(&[("numero", s.to_string().into())]),
    ))
}

/// "resposta do <fonte> inválida: <erro>" no idioma da instalação.
pub(crate) fn bad_response(fonte: &str, erro: impl std::fmt::Display) -> Error {
    Error::invalid(crate::i18n::t_args(
        "nucleo-ativos-erro-resposta-invalida",
        &crate::i18n::args(&[
            ("fonte", fonte.to_string().into()),
            ("erro", erro.to_string().into()),
        ]),
    ))
}

/// Ordena por data e remove datas repetidas (fica o último valor lido).
pub fn sorted(mut v: Vec<(NaiveDate, Decimal)>) -> Vec<(NaiveDate, Decimal)> {
    v.reverse();
    v.sort_by_key(|(d, _)| *d);
    v.dedup_by_key(|(d, _)| *d);
    v
}

/// Par de moedas de uma série de câmbio (`fx_eur_usd` → ("EUR", "USD"): 1 EUR = x USD).
/// Um sufixo depois do par distingue fontes do mesmo par (`fx_usd_cad_boc`).
pub fn fx_pair(series_id: &str) -> Option<(String, String)> {
    let mut parts = series_id.strip_prefix("fx_")?.split('_');
    let (b, q) = (parts.next()?, parts.next()?);
    (b.len() == 3 && q.len() == 3).then(|| (b.to_ascii_uppercase(), q.to_ascii_uppercase()))
}

/// Copia uma série de câmbio para `fx_rate`, que é onde as conversões procuram.
/// Pares com BRL ficam de fora no Brasil: lá a PTAX continua sendo a única fonte de conversão
/// para reais (fora do Brasil o BCB fica desligado e o BCE dá o real).
pub fn mirror_fx(
    conn: &Connection,
    series_id: &str,
    provider: DataProvider,
    values: &[(NaiveDate, Decimal, bool)],
) -> Result<usize> {
    let Some((base, quote)) = fx_pair(series_id) else {
        return Ok(0);
    };
    if (base == "BRL" || quote == "BRL") && crate::country::is_brazil(conn) {
        return Ok(0);
    }
    let now = now_utc();
    let mut stmt = conn.prepare(
        "INSERT INTO fx_rate (id, base, quote, date, rate, source, fetched_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?7)
         ON CONFLICT (base, quote, date, source) DO UPDATE SET rate = excluded.rate, fetched_at = excluded.fetched_at",
    )?;
    let mut n = 0;
    for (d, v, _) in values {
        if *v <= Decimal::ZERO {
            continue;
        }
        n += stmt.execute(params![
            new_id(),
            base,
            quote,
            fmt_date(*d),
            v.normalize().to_string(),
            provider.fx_source(),
            now
        ])?;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn periods_values_and_sources() {
        assert_eq!(parse_period("2026-10-13").unwrap(), d("2026-10-13"));
        assert_eq!(parse_period("2026-09").unwrap(), d("2026-09-01"));
        assert_eq!(parse_period("2026M09").unwrap(), d("2026-09-01"));
        assert_eq!(parse_period("2026-Q3").unwrap(), d("2026-07-01"));
        assert_eq!(parse_period("2025").unwrap(), d("2025-01-01"));
        assert!(parse_period("2026-Q5").is_err());
        assert!(parse_period("ontem").is_err());
        assert_eq!(parse_value(" 1.5 ").unwrap(), Some(dec!(1.5)));
        assert_eq!(parse_value("NaN").unwrap(), None);
        assert_eq!(parse_value("").unwrap(), None);
        assert_eq!(parse_value("1e-2").unwrap(), Some(dec!(0.01)));
        assert!(parse_value("x").is_err());
        assert_eq!(
            DataProvider::of_source("ecb:EXR/D.USD.EUR.SP00.A"),
            Some((DataProvider::Ecb, "EXR/D.USD.EUR.SP00.A"))
        );
        assert_eq!(
            DataProvider::of_source("bcb_sgs:433"),
            Some((DataProvider::BcbSgs, "433"))
        );
        assert_eq!(DataProvider::of_source("desconhecida:1737"), None);
        assert!(full_refresh("ecb:HICP/M.PT.N.000000.4D0.INX", "monthly"));
        assert!(!full_refresh("ecb:EXR/D.USD.EUR.SP00.A", "daily"));
        assert!(!full_refresh("fred:CPIAUCSL", "monthly"));
        assert_eq!(fx_pair("fx_eur_usd"), Some(("EUR".into(), "USD".into())));
        assert_eq!(fx_pair("estr"), None);
        assert_eq!(
            fx_pair("fx_usd_cad_boc"),
            Some(("USD".into(), "CAD".into()))
        );
        assert_eq!(
            sorted(vec![
                (d("2026-02-01"), dec!(2)),
                (d("2026-01-01"), dec!(1)),
                (d("2026-02-01"), dec!(3))
            ]),
            vec![(d("2026-01-01"), dec!(1)), (d("2026-02-01"), dec!(3))]
        );
    }

    #[test]
    fn mirrors_fx_except_brl() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST)
            .unwrap();
        let c = db.conn();
        let vals = vec![(d("2026-10-13"), dec!(1.1188), false)];
        assert_eq!(
            mirror_fx(c, "fx_eur_usd", DataProvider::Ecb, &vals).unwrap(),
            1
        );
        assert_eq!(
            mirror_fx(c, "fx_eur_brl", DataProvider::Ecb, &vals).unwrap(),
            0
        );
        let conv = crate::fx::convert(c, 10000, "EUR", "USD", "2026-10-14")
            .unwrap()
            .unwrap();
        assert_eq!(conv.amount, 11188);
        assert_eq!(conv.source.as_deref(), Some("ecb_ref"));
        // Inverso: USD → EUR pela mesma taxa.
        assert!(
            crate::fx::convert(c, 11188, "USD", "EUR", "2026-10-14")
                .unwrap()
                .is_some()
        );
        assert!(
            crate::fx::convert(c, 100, "EUR", "BRL", "2026-10-14")
                .unwrap()
                .is_none()
        );
    }

    /// Uma consulta pequena a cada provedor novo (rede): `cargo test -p treasury-core --
    /// --ignored live_providers --nocapture`.
    #[test]
    #[ignore = "usa a rede"]
    fn live_providers() {
        let agent = super::super::sync::agent();
        let today = crate::time::today_local();
        let back = |m| crate::time::add_months(today, m);
        let cases: &[(&str, NaiveDate)] = &[
            ("ecb:EXR/D.USD.EUR.SP00.A", back(-1)),
            ("ecb:EST/B.EU000A2X2A25.WT", back(-1)),
            ("ecb:HICP/M.PT.N.000000.4D0.INX", back(-12)),
            (
                "eurostat:prc_hicp_minr?geo=PL&unit=I15&coicop18=TOTAL",
                back(-12),
            ),
            (
                "oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/JPN.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/JPN.M.N.CPI.IX._T.N._Z",
                back(-12),
            ),
            (
                "oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/JPN.M.IR3TIB.PA._Z._Z._Z._Z.N",
                back(-12),
            ),
            ("boc:FXUSDCAD", back(-1)),
            ("boc:V41690973", back(-12)),
            ("boc:AVG.INTWO", back(-1)),
            ("ons:mm23/d7bt", back(-12)),
            ("boe:IUDSOIA", back(-1)),
            ("boe:XUDLUSS", back(-1)),
            ("worldbank:PT", back(-24)),
        ];
        let mut failed = vec![];
        for (src, from) in cases {
            match fetch(&agent, src, *from, today) {
                Ok(v) if !v.is_empty() => {
                    let (ld, lv, le) = v.last().unwrap();
                    println!(
                        "ok   {src}: {} pontos, último {ld} = {lv} (estimado: {le})",
                        v.len()
                    );
                }
                Ok(_) => failed.push(format!("{src}: vazio")),
                Err(e) => failed.push(format!("{src}: {e}")),
            }
        }
        assert!(failed.is_empty(), "falharam: {failed:#?}");
    }

    #[test]
    fn sidra_monthly_variation_by_group() {
        let body = r#"[{"D3C":"Mês (Código)","V":"Valor"},
            {"D3C":"202608","D3N":"agosto 2026","V":"-0.34"},
            {"D3C":"202609","D3N":"setembro 2026","V":"0.83"},
            {"D3C":"202610","D3N":"outubro 2026","V":"..."}]"#;
        let v = parse_sidra(body).unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        assert_eq!(v[1].1.to_string(), "0.83");
        assert_eq!(
            DataProvider::of_source("ibge_sidra:7060/7170"),
            Some((DataProvider::IbgeSidra, "7060/7170"))
        );
        assert!(DataProvider::IbgeSidra.is_legacy());
    }
}
