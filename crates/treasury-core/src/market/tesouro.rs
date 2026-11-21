//! Tesouro Transparente: preços e taxas históricos do Tesouro Direto (CSV público).
//! Usa o PU de venda da manhã (o valor de resgate antecipado) para avaliar a mercado. As
//! taxas (compra e venda da manhã) alimentam o score de renda fixa: a mediana diária por
//! família de títulos (10 anos) e a curva do dia mais recente, gravadas em `fundamental`
//! com emissor `tesouro:…` (sem migração).

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use chrono::NaiveDate;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const URL: &str = "https://www.tesourotransparente.gov.br/ckan/dataset/df56aa42-484a-4a59-8184-7676580c81e3/resource/796d2059-14e9-44e3-80c9-2d9e30b405c1/download/PrecoTaxaTesouroDireto.csv";

/// (tipo do título normalizado, vencimento) → pontos (data, PU venda).
pub type Prices = HashMap<(String, NaiveDate), Vec<(NaiveDate, Decimal)>>;

pub fn normalize_title(s: &str) -> String {
    crate::transactions::normalize_description(s)
}

fn dmy(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%d/%m/%Y").ok()
}

fn num(s: &str) -> Option<Decimal> {
    s.trim().replace('.', "").replace(',', ".").parse().ok()
}

/// Colunas usadas, achadas pelo nome no cabeçalho (a ordem pode mudar).
const COLS: [&str; 4] = [
    "Tipo Titulo",
    "Data Vencimento",
    "Data Base",
    "PU Venda Manha",
];

fn columns(header: &str) -> Option<[usize; 4]> {
    let names: Vec<String> = header
        .trim_start_matches('\u{feff}')
        .split(';')
        .map(normalize_title)
        .collect();
    let mut out = [0; 4];
    for (i, want) in COLS.iter().enumerate() {
        out[i] = names.iter().position(|n| *n == normalize_title(want))?;
    }
    Some(out)
}

pub fn parse(body: &str, since: NaiveDate) -> Prices {
    let mut out: Prices = HashMap::new();
    let mut lines = body.lines();
    let Some(idx) = lines.next().and_then(columns) else {
        return out;
    };
    for line in lines {
        let cols: Vec<&str> = line.split(';').collect();
        let get = |i: usize| cols.get(idx[i]).copied().unwrap_or("");
        let (Some(venc), Some(base), Some(pu)) = (dmy(get(1)), dmy(get(2)), num(get(3))) else {
            continue;
        };
        if base < since {
            continue;
        }
        out.entry((normalize_title(get(0)), venc))
            .or_default()
            .push((base, pu));
    }
    for v in out.values_mut() {
        v.sort_by_key(|(d, _)| *d);
        v.dedup_by_key(|(d, _)| *d);
    }
    out
}

/// Mínimo de linhas válidas do histórico completo (tem mais de 170 mil desde 2004).
const MIN_ROWS: usize = 1000;
/// Dias sem dado novo até o arquivo ser considerado parado.
const MAX_STALE_DAYS: i64 = 10;

/// Confere se o arquivo ainda tem o formato esperado antes de gravar qualquer preço: as
/// colunas pelo nome, quantidade de linhas, linhas legíveis, faixas plausíveis de preço e
/// data base recente. Erro aqui conta como falha da fonte.
pub fn validate(body: &str, today: NaiveDate) -> Result<(), String> {
    let mut lines = body.lines();
    let header = lines.next().unwrap_or("");
    let Some(idx) = columns(header) else {
        return Err(crate::i18n::t_args(
            "nucleo-ativos-erro-tesouro-colunas",
            &crate::i18n::args(&[("colunas", COLS.join(", ").into())]),
        ));
    };
    let (mut ok, mut bad) = (0usize, 0usize);
    let mut last: Option<NaiveDate> = None;
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let cols: Vec<&str> = line.split(';').collect();
        let get = |i: usize| cols.get(idx[i]).copied().unwrap_or("");
        match (dmy(get(1)), dmy(get(2)), num(get(3))) {
            (Some(_), Some(base), Some(pu))
                if pu > Decimal::ZERO && pu <= Decimal::from(1_000_000) =>
            {
                ok += 1;
                last = last.max(Some(base));
            }
            _ => bad += 1,
        }
    }
    if ok < MIN_ROWS {
        return Err(crate::i18n::t_args(
            "nucleo-ativos-erro-tesouro-poucas-linhas",
            &crate::i18n::args(&[("linhas", ok.to_string().into())]),
        ));
    }
    if bad * 20 > ok + bad {
        return Err(crate::i18n::t_args(
            "nucleo-ativos-erro-tesouro-linhas-ruins",
            &crate::i18n::args(&[
                ("ruins", bad.to_string().into()),
                ("total", (ok + bad).to_string().into()),
            ]),
        ));
    }
    if let Some(d) = last
        && (today - d).num_days() > MAX_STALE_DAYS
    {
        return Err(crate::i18n::t_args(
            "nucleo-ativos-erro-tesouro-parado",
            &crate::i18n::args(&[("data", crate::i18n::date(crate::i18n::current(), d).into())]),
        ));
    }
    Ok(())
}

/// Ajuste técnico: ETag e Last-Modified do último CSV baixado (download condicional).
pub const META_SETTING: &str = "tesouro.csv_meta";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CsvMeta {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub enum Download {
    /// O arquivo não mudou desde o último download (nada novo para gravar).
    NotModified,
    /// Preços desde `since` e, se pedidas, as taxas desde `rates_since`.
    Fresh(Prices, Option<Rates>, CsvMeta),
}

/// Baixa o histórico (cerca de 15 MB). Com `cond`, pede só se mudou; com `rates_since`,
/// também lê as taxas (score de renda fixa).
pub fn fetch(
    since: NaiveDate,
    rates_since: Option<NaiveDate>,
    cond: Option<&CsvMeta>,
    today: NaiveDate,
) -> Result<Download, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(180)))
        .http_status_as_error(false)
        .build()
        .into();
    let call = || {
        let mut req = agent.get(URL).header(
            "User-Agent",
            "Treasury/0.1 (app local de financas pessoais)",
        );
        if let Some(m) = cond {
            if let Some(e) = &m.etag {
                req = req.header("If-None-Match", e);
            }
            if let Some(l) = &m.last_modified {
                req = req.header("If-Modified-Since", l);
            }
        }
        req.call().map_err(|e| e.to_string())
    };
    // O site do Tesouro Transparente às vezes recusa por um tempo (429 com Retry-After, ou
    // conexão reiniciada): espera o que ele pede (até 1 min) e tenta de novo, uma vez.
    let mut resp = match call() {
        Ok(r) if r.status().as_u16() == 429 => {
            let wait = r
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse::<u64>().ok())
                .unwrap_or(30)
                .min(60);
            std::thread::sleep(Duration::from_secs(wait));
            call()?
        }
        Ok(r) => r,
        Err(e) if crate::sources::is_transient(&e) => {
            std::thread::sleep(Duration::from_secs(15));
            call()?
        }
        Err(e) => return Err(e),
    };
    let status = resp.status().as_u16();
    if status == 304 {
        return Ok(Download::NotModified);
    }
    if status != 200 {
        return Err(crate::i18n::t_args(
            "nucleo-ativos-erro-tesouro-http",
            &crate::i18n::args(&[("status", status.to_string().into())]),
        ));
    }
    let header = |n: &str| {
        resp.headers()
            .get(n)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    let meta = CsvMeta {
        etag: header("etag"),
        last_modified: header("last-modified"),
    };
    let body = resp
        .body_mut()
        .with_config()
        .limit(256 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    if body.trim_start().starts_with('<') {
        return Err(crate::i18n::t("nucleo-ativos-erro-tesouro-pagina"));
    }
    validate(&body, today)?;
    let rates = rates_since.map(|d| parse_rates(&body, d));
    Ok(Download::Fresh(parse(&body, since), rates, meta))
}

// ---------------------------------------------------------------------------
// Taxas (score de renda fixa)

/// Tipo do título pelo nome normalizado (`normalize_title`).
pub fn kind_of(title: &str) -> Option<&'static str> {
    let t = normalize_title(title);
    let juros = t.contains("juros");
    Some(if t.contains("selic") {
        "selic"
    } else if t.contains("educa") {
        "educa"
    } else if t.contains("renda") {
        "renda"
    } else if t.contains("igpm") {
        "igpm_juros"
    } else if t.contains("ipca") {
        if juros { "ipca_juros" } else { "ipca" }
    } else if t.contains("prefixado") {
        if juros {
            "prefixado_juros"
        } else {
            "prefixado"
        }
    } else {
        return None;
    })
}

/// Família de taxas do tipo: `real` (IPCA+, Renda+, Educa+) ou `nominal` (prefixados). O
/// Tesouro Selic (ágio sobre a Selic) e o IGP-M+ (fora de venda) não entram nas medianas.
pub fn family_of(kind: &str) -> Option<&'static str> {
    match kind {
        "ipca" | "ipca_juros" | "renda" | "educa" => Some("real"),
        "prefixado" | "prefixado_juros" => Some("nominal"),
        _ => None,
    }
}

/// Ano de vencimento a partir do ano no nome: o Renda+ leva o ano da aposentadoria (vence 19
/// anos depois) e o Educa+ o do primeiro pagamento (vence 4 anos depois).
pub fn maturity_year(kind: &str, name_year: i32) -> i32 {
    match kind {
        "renda" => name_year + 19,
        "educa" => name_year + 4,
        _ => name_year,
    }
}

/// Taxas de um título num dia (frações a.a.; PU em reais).
#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub kind: String,
    pub maturity: NaiveDate,
    pub date: NaiveDate,
    pub rate_buy: Decimal,
    pub rate_sell: Decimal,
    pub pu_buy: Decimal,
    pub pu_sell: Decimal,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rates {
    /// (família, data) → mediana das taxas de compra dos títulos com mais de um ano até o
    /// vencimento (fração a.a.).
    pub medians: BTreeMap<(String, NaiveDate), Decimal>,
    /// Todos os títulos no dia mais recente do arquivo.
    pub curve: Vec<Quote>,
}

/// Títulos a menos de um ano do vencimento ficam fora da mediana (a taxa perto do
/// vencimento oscila sem significado econômico; ex.: −1,78% no IPCA+ de agosto/2020).
const MEDIAN_MIN_DAYS: i64 = 365;

pub fn parse_rates(body: &str, since: NaiveDate) -> Rates {
    let mut lines = body.lines();
    let names: Vec<String> = lines
        .next()
        .unwrap_or("")
        .trim_start_matches('\u{feff}')
        .split(';')
        .map(normalize_title)
        .collect();
    let want = [
        "Tipo Titulo",
        "Data Vencimento",
        "Data Base",
        "Taxa Compra Manha",
        "Taxa Venda Manha",
        "PU Compra Manha",
        "PU Venda Manha",
    ];
    let Some(idx) = want
        .iter()
        .map(|w| names.iter().position(|n| *n == normalize_title(w)))
        .collect::<Option<Vec<usize>>>()
    else {
        return Rates::default();
    };
    let pct = |d: Decimal| d / Decimal::from(100);
    let mut by_day: BTreeMap<(String, NaiveDate), Vec<Decimal>> = BTreeMap::new();
    let mut all: Vec<Quote> = vec![];
    for line in lines {
        let cols: Vec<&str> = line.split(';').collect();
        let get = |i: usize| cols.get(idx[i]).copied().unwrap_or("");
        let (Some(kind), Some(maturity), Some(date)) = (kind_of(get(0)), dmy(get(1)), dmy(get(2)))
        else {
            continue;
        };
        if date < since {
            continue;
        }
        let (Some(rb), Some(rs), Some(pb), Some(ps)) =
            (num(get(3)), num(get(4)), num(get(5)), num(get(6)))
        else {
            continue;
        };
        if let Some(f) = family_of(kind)
            && (maturity - date).num_days() > MEDIAN_MIN_DAYS
        {
            by_day
                .entry((f.to_string(), date))
                .or_default()
                .push(pct(rb));
        }
        all.push(Quote {
            kind: kind.to_string(),
            maturity,
            date,
            rate_buy: pct(rb),
            rate_sell: pct(rs),
            pu_buy: pb,
            pu_sell: ps,
        });
    }
    let medians = by_day
        .into_iter()
        .map(|(k, mut v)| {
            v.sort();
            let n = v.len();
            let m = if n % 2 == 1 {
                v[n / 2]
            } else {
                (v[n / 2 - 1] + v[n / 2]) / Decimal::from(2)
            };
            (k, m)
        })
        .collect();
    let last = all.iter().map(|q| q.date).max();
    let mut curve: Vec<Quote> = all.into_iter().filter(|q| Some(q.date) == last).collect();
    curve.sort_by(|a, b| (&a.kind, a.maturity).cmp(&(&b.kind, b.maturity)));
    curve.dedup_by(|a, b| a.kind == b.kind && a.maturity == b.maturity);
    Rates { medians, curve }
}

/// Emissor (em `fundamental`) das medianas de uma família e dos títulos da curva.
pub const RATES_PREFIX: &str = "tesouro:";
pub const CURVE_PREFIX: &str = "tesouro:titulo:";
pub const RATES_SOURCE: &str = "tesouro_transparente";
const RATES_DOC: &str = "PrecoTaxaTesouroDireto.csv (Tesouro Transparente)";

pub fn family_issuer(family: &str) -> String {
    format!("{RATES_PREFIX}{family}")
}

pub fn title_issuer(kind: &str, maturity: NaiveDate) -> String {
    format!("{CURVE_PREFIX}{kind}:{}", maturity.format("%Y-%m-%d"))
}

/// Grava as medianas e troca a curva pela do dia mais recente.
pub fn store_rates(conn: &Connection, r: &Rates, now: &str) -> crate::error::Result<usize> {
    use crate::research::{Fact, PeriodType, store};
    let fact = |item: &str, end: NaiveDate, value: Decimal, ccy: &str, doc: &str| Fact {
        item: item.into(),
        period_type: PeriodType::Instant,
        start: None,
        end,
        value,
        currency: ccy.into(),
        source: RATES_SOURCE.into(),
        document: doc.into(),
        priority: 0,
    };
    let mut by_family: BTreeMap<&str, Vec<Fact>> = BTreeMap::new();
    for ((f, d), v) in &r.medians {
        by_family.entry(f.as_str()).or_default().push(fact(
            "rate_median",
            *d,
            *v,
            "ratio",
            &format!("{RATES_DOC}: mediana das taxas de compra"),
        ));
    }
    let mut n = 0;
    for (f, facts) in by_family {
        let oldest = facts.iter().map(|x| x.end).min();
        n += store::upsert_facts(conn, &family_issuer(f), &facts, now)?;
        // A janela anda: o que ficou antes do início do arquivo lido sai.
        if let Some(o) = oldest {
            conn.execute(
                "DELETE FROM fundamental WHERE issuer_id = ?1 AND period_end < ?2",
                rusqlite::params![family_issuer(f), crate::time::fmt_date(o)],
            )?;
        }
    }
    if !r.curve.is_empty() {
        conn.execute(
            "DELETE FROM fundamental WHERE issuer_id LIKE ?1",
            [format!("{CURVE_PREFIX}%")],
        )?;
        for q in &r.curve {
            let doc = format!("{RATES_DOC}: taxas e PU da manhã");
            let facts = [
                fact("rate_buy", q.date, q.rate_buy, "ratio", &doc),
                fact("rate_sell", q.date, q.rate_sell, "ratio", &doc),
                fact("pu_buy", q.date, q.pu_buy, "BRL", &doc),
                fact("pu_sell", q.date, q.pu_sell, "BRL", &doc),
            ];
            n += store::upsert_facts(conn, &title_issuer(&q.kind, q.maturity), &facts, now)?;
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use rust_decimal_macros::dec;

    #[test]
    fn parses_csv() {
        let csv = "Tipo Titulo;Data Vencimento;Data Base;Taxa Compra Manha;Taxa Venda Manha;PU Compra Manha;PU Venda Manha;PU Base Manha\n\
                   Tesouro IPCA+;15/05/2035;13/10/2026;7,10;7,22;2.512,40;2.498,77;2.498,77\n\
                   Tesouro IPCA+;15/05/2035;10/10/2026;7,12;7,24;2.508,00;2.494,10;2.494,10\n\
                   Tesouro Selic;01/03/2031;13/10/2026;0,09;0,10;20001,60;19982,63;19982,63\n\
                   Tesouro IPCA+;15/05/2035;01/01/2020;3,00;3,10;1.000,00;990,00;990,00\n";
        let p = parse(csv, dmy("01/01/2026").unwrap());
        let ipca = &p[&(normalize_title("Tesouro IPCA+"), dmy("15/05/2035").unwrap())];
        assert_eq!(ipca.len(), 2, "datas antes de `since` ficam de fora");
        assert_eq!(ipca[1], (dmy("13/10/2026").unwrap(), dec!(2498.77)));
        assert!(p.contains_key(&(normalize_title("Tesouro Selic"), dmy("01/03/2031").unwrap())));
    }

    #[test]
    fn parses_rates_into_family_medians_and_latest_curve() {
        let csv = "Tipo Titulo;Data Vencimento;Data Base;Taxa Compra Manha;Taxa Venda Manha;PU Compra Manha;PU Venda Manha;PU Base Manha\n\
                   Tesouro IPCA+;15/05/2035;13/10/2026;7,10;7,22;2.512,40;2.498,77;2.498,77\n\
                   Tesouro IPCA+ com Juros Semestrais;15/05/2045;13/10/2026;6,90;7,02;4.512,40;4.498,77;4.498,77\n\
                   Tesouro Renda+ Aposentadoria Extra;15/12/2049;13/10/2026;6,80;6,92;2.163,88;2.136,55;2.136,55\n\
                   Tesouro IPCA+;15/08/2026;10/10/2026;-1,00;-0,90;4.000,00;3.999,00;3.999,00\n\
                   Tesouro IPCA+;15/05/2035;10/10/2026;7,12;7,24;2.508,00;2.494,10;2.494,10\n\
                   Tesouro Prefixado;01/01/2029;13/10/2026;12,64;12,76;770,85;768,69;768,69\n\
                   Tesouro Selic;01/03/2031;13/10/2026;0,09;0,10;20001,60;19982,63;19982,63\n\
                   Tesouro IPCA+;15/05/2035;01/01/2010;6,00;6,10;1.000,00;990,00;990,00\n";
        let r = parse_rates(csv, dmy("01/01/2016").unwrap());
        let day = dmy("13/10/2026").unwrap();
        // Mediana de 7,10%, 6,90% e 6,80% (IPCA+, IPCA+ com juros e Renda+).
        assert_eq!(r.medians[&("real".to_string(), day)], dec!(0.069));
        // 10/10: o título a menos de um ano do vencimento fica fora.
        assert_eq!(
            r.medians[&("real".to_string(), dmy("10/10/2026").unwrap())],
            dec!(0.0712)
        );
        assert_eq!(r.medians[&("nominal".to_string(), day)], dec!(0.1264));
        assert!(!r.medians.keys().any(|(f, _)| f == "selic"));
        assert!(!r.medians.keys().any(|(_, d)| d.year() < 2016));
        // Curva: só o dia mais recente, todos os títulos.
        assert_eq!(r.curve.len(), 5);
        assert!(r.curve.iter().all(|q| q.date == day));
        let ipca = r.curve.iter().find(|q| q.kind == "ipca").unwrap();
        assert_eq!(ipca.rate_sell, dec!(0.0722));
        assert_eq!(ipca.pu_buy, dec!(2512.40));
        assert_eq!(
            kind_of("Tesouro IGPM+ com Juros Semestrais"),
            Some("igpm_juros")
        );
        assert_eq!(kind_of("Tesouro Educa+"), Some("educa"));
        assert_eq!(
            kind_of("Tesouro Prefixado com Juros Semestrais"),
            Some("prefixado_juros")
        );
        assert_eq!(kind_of("CDB"), None);
        assert_eq!(maturity_year("renda", 2030), 2049);
    }

    #[test]
    fn validates_layout_rows_ranges_and_freshness() {
        let today = dmy("16/10/2026").unwrap();
        let header = "Tipo Titulo;Data Vencimento;Data Base;Taxa Compra Manha;Taxa Venda Manha;PU Compra Manha;PU Venda Manha;PU Base Manha";
        let rows = |base: &str, n: usize| {
            (0..n)
                .map(|i| {
                    format!(
                        "Tesouro Selic;01/03/{};{base};0,09;0,10;20001,60;19982,63;19982,63",
                        2027 + i % 5
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let good = format!("{header}\n{}", rows("15/10/2026", 1200));
        assert!(validate(&good, today).is_ok());
        // Colunas trocadas de lugar continuam valendo (achadas pelo nome).
        let swapped = good.replacen(
            "Data Base;Taxa Compra Manha",
            "Taxa Compra Manha;Data Base",
            1,
        );
        assert!(columns(swapped.lines().next().unwrap()).is_some());
        let renamed = good.replacen("PU Venda Manha", "Preco Venda", 1);
        assert!(validate(&renamed, today).unwrap_err().contains("layout"));
        let few = format!("{header}\n{}", rows("15/10/2026", 10));
        assert!(validate(&few, today).unwrap_err().contains("poucas linhas"));
        let stale = format!("{header}\n{}", rows("01/09/2026", 1200));
        assert!(validate(&stale, today).unwrap_err().contains("parados"));
        let junk = format!(
            "{header}\n{}\n{}",
            rows("15/10/2026", 1000),
            "x;y;z\n".repeat(200)
        );
        assert!(validate(&junk, today).unwrap_err().contains("ilegíveis"));
    }
}
