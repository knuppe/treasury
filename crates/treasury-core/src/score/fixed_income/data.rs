//! Leituras da renda fixa: taxas e curva do Tesouro (gravadas pela coleta de cotações em
//! `fundamental`), índices (CDI, Selic meta, IPCA, TR), emissor no IF.data ou na CVM,
//! posições por conta e titular (FGC).

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;

use super::calc;
use super::params;
use crate::assets::{Asset, AssetClass, AssetFilter, FixedIncomeKind};
use crate::error::Result;
use crate::market::tesouro::{self, Quote};
use crate::research::ifdata::{self, BankInstitution};
use crate::research::{Issuer, IssuerKind, store, to_f64};
use crate::time::{add_months, fmt_date, parse_date};

// ---------------------------------------------------------------------------
// Tesouro

/// O que a coleta de cotações precisa baixar para o score de renda fixa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatesNeed {
    /// Início da janela de taxas (histórico de 10 anos).
    pub since: NaiveDate,
    /// Nada gravado ainda: baixar o CSV mesmo que não tenha mudado.
    pub force: bool,
}

/// Taxas do Tesouro a baixar: só com a fonte ligada, renda fixa acompanhada (com operação ou
/// na watchlist) e medianas gravadas com mais de `rates_refresh_days` dias.
pub fn rates_needed(conn: &Connection, today: NaiveDate) -> Result<Option<RatesNeed>> {
    if !crate::sources::enabled(conn, crate::sources::SourceId::Tesouro) {
        return Ok(None);
    }
    let tracked: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM asset a WHERE a.archived = 0
            AND a.class IN ('fixed_income', 'treasury_br')
            AND (EXISTS (SELECT 1 FROM trade t WHERE t.asset_id = a.id)
                 OR EXISTS (SELECT 1 FROM watchlist_item w WHERE w.asset_id = a.id)))",
        [],
        |r| r.get(0),
    )?;
    if !tracked {
        return Ok(None);
    }
    let p = params();
    let since = add_months(today, -12 * p.rate_history_years as i32 - 1);
    let latest: Option<String> = conn.query_row(
        "SELECT max(period_end) FROM fundamental WHERE issuer_id IN (?1, ?2) AND item = 'rate_median'",
        params![
            tesouro::family_issuer("real"),
            tesouro::family_issuer("nominal")
        ],
        |r| r.get(0),
    )?;
    Ok(match latest.map(|d| parse_date(&d)).transpose()? {
        None => Some(RatesNeed { since, force: true }),
        Some(d) if (today - d).num_days() > p.rates_refresh_days => Some(RatesNeed {
            since,
            force: false,
        }),
        Some(_) => None,
    })
}

/// Curva do dia mais recente gravado (todos os títulos).
pub fn curve(conn: &Connection) -> Result<Vec<Quote>> {
    let mut stmt = conn.prepare_cached(
        "SELECT issuer_id, item, period_end, value FROM fundamental WHERE issuer_id LIKE ?1",
    )?;
    let mut by: BTreeMap<(String, NaiveDate), Quote> = BTreeMap::new();
    let rows = stmt
        .query_map([format!("{}%", tesouro::CURVE_PREFIX)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (id, item, end, value) in rows {
        let Some(rest) = id.strip_prefix(tesouro::CURVE_PREFIX) else {
            continue;
        };
        let Some((kind, mat)) = rest.split_once(':') else {
            continue;
        };
        let (Ok(maturity), Ok(date), Ok(v)) =
            (parse_date(mat), parse_date(&end), value.parse::<Decimal>())
        else {
            continue;
        };
        let q = by.entry((kind.to_string(), maturity)).or_insert(Quote {
            kind: kind.to_string(),
            maturity,
            date,
            rate_buy: Decimal::ZERO,
            rate_sell: Decimal::ZERO,
            pu_buy: Decimal::ZERO,
            pu_sell: Decimal::ZERO,
        });
        match item.as_str() {
            "rate_buy" => q.rate_buy = v,
            "rate_sell" => q.rate_sell = v,
            "pu_buy" => q.pu_buy = v,
            "pu_sell" => q.pu_sell = v,
            _ => {}
        }
    }
    Ok(by.into_values().collect())
}

/// Vértices (anos até o vencimento, taxa de compra) dos títulos sem cupom de um tipo.
pub fn curve_points(curve: &[Quote], kind: &str, today: NaiveDate) -> Vec<(f64, f64)> {
    curve
        .iter()
        .filter(|q| q.kind == kind && q.maturity > today)
        .map(|q| (calc::years_between(today, q.maturity), to_f64(q.rate_buy)))
        .collect()
}

/// Medianas diárias da família (`real` ou `nominal`) na janela.
pub fn rate_history(
    conn: &Connection,
    family: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<f64>> {
    let mut stmt = conn.prepare_cached(
        "SELECT value FROM fundamental WHERE issuer_id = ?1 AND item = 'rate_median'
           AND period_end >= ?2 AND period_end <= ?3",
    )?;
    let rows = stmt
        .query_map(
            params![tesouro::family_issuer(family), fmt_date(from), fmt_date(to)],
            |r| r.get::<_, String>(0),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|v| v.parse::<Decimal>().ok().map(to_f64))
        .collect())
}

/// Título do Tesouro de um ativo: tipo (pelo código, nome ou termos) e vencimento (dos
/// termos ou, pelo ano no nome, da curva).
#[derive(Debug, Clone, PartialEq)]
pub struct Title {
    pub kind: String,
    pub maturity: Option<NaiveDate>,
    pub quote: Option<Quote>,
}

pub fn title_of(a: &Asset, curve: &[Quote]) -> Option<Title> {
    let text = a.treasury_code.clone().unwrap_or_else(|| a.name.clone());
    let kind = tesouro::kind_of(&text)
        .or_else(|| tesouro::kind_of(&a.name))
        .or_else(|| match a.fixed_income.as_ref()?.kind {
            FixedIncomeKind::TesouroSelic => Some("selic"),
            FixedIncomeKind::TesouroPrefixado => Some("prefixado"),
            FixedIncomeKind::TesouroIpca => Some("ipca"),
            FixedIncomeKind::TesouroRenda => Some("renda"),
            _ => None,
        })?
        .to_string();
    let terms_maturity = a
        .fixed_income
        .as_ref()
        .and_then(|t| t.maturity_date.as_deref())
        .and_then(|d| parse_date(d).ok());
    let year = calc::trailing_year(&text).or_else(|| calc::trailing_year(&a.name));
    let quote = curve
        .iter()
        .find(|q| {
            q.kind == kind
                && match terms_maturity {
                    Some(m) => q.maturity == m,
                    None => year.is_some_and(|y| {
                        chrono::Datelike::year(&q.maturity) == tesouro::maturity_year(&kind, y)
                    }),
                }
        })
        .cloned();
    Some(Title {
        maturity: terms_maturity.or(quote.as_ref().map(|q| q.maturity)),
        kind,
        quote,
    })
}

/// Vencimento já passado do título: pelos termos, ou pelo ano no nome quando o título não
/// está mais na curva do Tesouro (o arquivo traz os títulos até a véspera do vencimento; um
/// título do ano corrente fora da curva já venceu). Devolve como mostrar a data.
pub fn matured(a: &Asset, curve: &[Quote], today: NaiveDate) -> Option<String> {
    let title = title_of(a, curve)?;
    if let Some(m) = title.maturity {
        return (m <= today).then(|| crate::score::metrics::br(m));
    }
    if title.quote.is_some() {
        return None;
    }
    let text = a.treasury_code.clone().unwrap_or_else(|| a.name.clone());
    let y = calc::trailing_year(&text).or_else(|| calc::trailing_year(&a.name))?;
    let year = tesouro::maturity_year(&title.kind, y);
    let this = chrono::Datelike::year(&today);
    // Só com a curva baixada dá para dizer que um título do ano corrente saiu dela.
    let curve_fresh = curve.iter().any(|q| (today - q.date).num_days() <= 15);
    (year < this || (year == this && curve_fresh)).then(|| year.to_string())
}

// ---------------------------------------------------------------------------
// Índices

/// CDI de hoje anualizado (último dia publicado, até 10 dias antes).
pub fn cdi_annual(conn: &Connection, today: NaiveDate) -> Result<Option<(f64, NaiveDate)>> {
    let pts = crate::indices::daily_rates(conn, "cdi", today - chrono::Duration::days(10), today)?;
    Ok(pts
        .last()
        .map(|(d, r)| ((1.0 + to_f64(*r) / 100.0).powf(252.0) - 1.0, *d)))
}

/// Selic meta (fração a.a.) numa data.
pub fn selic_meta(conn: &Connection, at: NaiveDate) -> Result<Option<f64>> {
    Ok(crate::indices::level_on(conn, "selic_meta", at)?.map(|v| to_f64(v) / 100.0))
}

/// Variações mensais oficiais (sem projeção) de uma série mensal: (ano, mês) → fração.
pub fn monthly(
    conn: &Connection,
    series: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<BTreeMap<(i32, u32), f64>> {
    Ok(crate::indices::monthly_rates(conn, series, from, to)?
        .into_iter()
        .filter(|r| !r.estimated)
        .map(|r| ((r.year, r.month), to_f64(r.pct) / 100.0))
        .collect())
}

/// Acumulado dos 12 meses oficiais terminados no mês de `upto` ou antes (o último mês
/// publicado). `None` sem 12 meses seguidos.
pub fn twelve_months(m: &BTreeMap<(i32, u32), f64>, upto: (i32, u32)) -> Option<(f64, (i32, u32))> {
    let (&last, _) = m.range(..=upto).next_back()?;
    let mut acc = 1.0;
    let (mut y, mut mo) = last;
    for _ in 0..12 {
        acc *= 1.0 + m.get(&(y, mo))?;
        if mo == 1 {
            y -= 1;
            mo = 12;
        } else {
            mo -= 1;
        }
    }
    Some((acc - 1.0, last))
}

/// Juro real da Selic (Selic meta ÷ IPCA de 12 meses) no fim de cada mês da janela.
pub fn real_selic_history(
    conn: &Connection,
    ipca: &BTreeMap<(i32, u32), f64>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<f64>> {
    let mut out = vec![];
    let mut d = from;
    while d <= to {
        let eom = add_months(d, 1) - chrono::Duration::days(1);
        let at = eom.min(to);
        if let (Some(s), Some((i, _))) = (selic_meta(conn, at)?, twelve_months(ipca, calc::ym(at)))
        {
            out.push(calc::real_rate(s, i));
        }
        d = add_months(d, 1);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Emissor

fn cnpj_of(a: &Asset) -> Option<String> {
    let c = crate::research::cnpj_digits(a.cnpj.as_deref()?);
    (c.len() >= 8).then_some(c)
}

fn issuer_text(a: &Asset) -> Option<String> {
    a.fixed_income
        .as_ref()
        .and_then(|t| t.issuer.clone())
        .filter(|s| !s.trim().is_empty())
}

/// Como o emissor foi identificado (para a nota da métrica).
#[derive(Debug, Clone, PartialEq)]
pub enum Match {
    Cnpj,
    Name(String),
}

/// Instituição do IF.data que reporta (conglomerado prudencial) para um código.
fn reporting(conn: &Connection, i: BankInstitution) -> Result<BankInstitution> {
    if let Some(p) = i.prudential_code.as_ref().filter(|p| **p != i.cod_inst)
        && let Some(c) = ifdata::institution(conn, p)?
    {
        return Ok(c);
    }
    Ok(i)
}

/// Banco pelo nome: só quando o nome sem as palavras genéricas fica igual ao de uma
/// instituição do cadastro do IF.data e todas as que batem são do mesmo conglomerado.
pub fn bank_by_name(conn: &Connection, text: &str) -> Result<Option<BankInstitution>> {
    let stop = &params().issuer_stopwords;
    let core = calc::core_name(text, stop);
    if core.len() < 2 {
        return Ok(None);
    }
    let mut stmt = conn.prepare_cached("SELECT cod_inst, name FROM bank_institution")?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut found: Vec<BankInstitution> = vec![];
    for (cod, name) in &rows {
        if calc::core_name(name, stop) != core {
            continue;
        }
        if let Some(i) = ifdata::institution(conn, cod)? {
            let r = reporting(conn, i)?;
            if !found.iter().any(|f| f.cod_inst == r.cod_inst) {
                found.push(r);
            }
        }
    }
    if found.len() == 1 {
        return Ok(found.pop());
    }
    if !found.is_empty() {
        return Ok(None);
    }
    // Marca conhecida ("Nubank", "Itaú"): o CNPJ do emissor, conferido pelo nome que o
    // IF.data dá a ele (um CNPJ errado na lista não casa com banco nenhum).
    let words: Vec<&str> = core.split_whitespace().collect();
    for (brand, cnpj8, token) in BANK_ALIASES {
        if words.contains(brand)
            && let Some(i) = ifdata::institution_for_cnpj8(conn, cnpj8)?
            && crate::research::norm(&i.name)
                .split_whitespace()
                .any(|w| w == *token)
        {
            return Ok(Some(reporting(conn, i)?));
        }
    }
    // Uma palavra só e marcante ("Credivale"): a instituição cujo nome tem essa palavra, se for
    // de um conglomerado só.
    if let [w] = words.as_slice()
        && w.len() >= 5
    {
        for (cod, name) in &rows {
            if crate::research::norm(name)
                .split_whitespace()
                .any(|x| x == *w)
                && let Some(i) = ifdata::institution(conn, cod)?
            {
                let r = reporting(conn, i)?;
                if !found.iter().any(|f| f.cod_inst == r.cod_inst) {
                    found.push(r);
                }
            }
        }
    }
    Ok(if found.len() == 1 { found.pop() } else { None })
}

/// Marcas pelas quais os emissores aparecem nos títulos → CNPJ (8 dígitos) da instituição
/// que emite e a palavra que o nome dela tem no cadastro do IF.data (a conferência).
const BANK_ALIASES: &[(&str, &str, &str)] = &[
    ("nubank", "30680829", "nu"),
    ("nu", "30680829", "nu"),
    ("inter", "00416968", "inter"),
    ("itau", "60701190", "itau"),
    ("bradesco", "60746948", "bradesco"),
    ("santander", "90400888", "santander"),
    ("caixa", "00360305", "caixa"),
    ("btg", "30306294", "btg"),
    ("c6", "31872495", "c6"),
    ("xp", "33264668", "xp"),
    ("sofisa", "60889128", "sofisa"),
    ("daycoval", "62232889", "daycoval"),
    ("pan", "59285411", "pan"),
    ("bmg", "61186680", "bmg"),
    ("safra", "58160789", "safra"),
    ("original", "92894922", "original"),
];

/// Banco emissor: pelo CNPJ do cadastro do título, senão pelo nome do emissor.
pub fn bank_issuer(conn: &Connection, a: &Asset) -> Result<Option<(BankInstitution, Match)>> {
    // O CNPJ do banco pode ser o de uma instituição do conglomerado (no cadastro do IF.data o
    // CNPJ de cada uma é o do líder): sobe para o conglomerado prudencial que reporta.
    if let Some(c) = cnpj_of(a)
        && let Some(i) = ifdata::institution_for_cnpj8(conn, &c[..8])?
    {
        return Ok(Some((reporting(conn, i)?, Match::Cnpj)));
    }
    if let Some(t) = issuer_text(a)
        && let Some(i) = bank_by_name(conn, &t)?
    {
        return Ok(Some((i, Match::Name(t))));
    }
    Ok(None)
}

/// CNPJs (8 dígitos) ou códigos do IF.data dos bancos achados pelo nome do emissor nos
/// títulos sem CNPJ (para a coleta baixar os relatórios 2 e 4 deles também).
pub fn matched_bank_cnpj8(conn: &Connection) -> Result<HashSet<String>> {
    let mut out = HashSet::new();
    let f = AssetFilter {
        classes: Some(vec![AssetClass::FixedIncome]),
        ..Default::default()
    };
    for a in crate::assets::list(conn, &f)? {
        if cnpj_of(&a).is_some() {
            continue;
        }
        if let Some(t) = issuer_text(&a)
            && let Some(i) = bank_by_name(conn, &t)?
        {
            out.insert(i.cnpj8.clone().unwrap_or(i.cod_inst.clone()));
        }
    }
    Ok(out)
}

/// Companhia aberta (CVM) emissora: pelo CNPJ do título, senão pelo nome igual.
pub fn company_issuer(conn: &Connection, a: &Asset) -> Result<Option<(Issuer, Match)>> {
    if let Some(c) = cnpj_of(a).filter(|c| c.len() == 14)
        && let Some(i) = store::issuer(conn, &c)?.filter(|i| i.kind == IssuerKind::CompanyBr)
    {
        return Ok(Some((i, Match::Cnpj)));
    }
    let Some(t) = issuer_text(a) else {
        return Ok(None);
    };
    let stop = &params().issuer_stopwords;
    let core = calc::core_name(&t, stop);
    if core.len() < 2 {
        return Ok(None);
    }
    let mut stmt =
        conn.prepare_cached("SELECT id, name FROM research_issuer WHERE kind = 'company_br'")?;
    let ids: Vec<String> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .filter_map(|r| r.ok())
        .filter(|(_, n)| calc::core_name(n, stop) == core)
        .map(|(id, _)| id)
        .collect();
    if ids.len() != 1 {
        return Ok(None);
    }
    Ok(store::issuer(conn, &ids[0])?.map(|i| (i, Match::Name(t))))
}

// ---------------------------------------------------------------------------
// Posições (FGC)

/// Titular de uma conta sem titulares cadastrados (todas contam como o mesmo CPF).
pub const UNKNOWN_HOLDER: &str = "?";

/// Valor (BRL) da posição do ativo por titular: cada conta dividida igualmente entre os
/// titulares (conta conjunta, como o FGC faz).
pub fn position_by_holder(
    conn: &Connection,
    asset_id: &str,
    today: NaiveDate,
) -> Result<BTreeMap<String, Decimal>> {
    use crate::portfolio::engine;
    let accounts: Vec<String> = conn
        .prepare_cached("SELECT DISTINCT account_id FROM trade WHERE asset_id = ?1")?
        .query_map([asset_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out: BTreeMap<String, Decimal> = BTreeMap::new();
    for acc in accounts {
        let book = engine::load_book(conn, asset_id, Some(std::slice::from_ref(&acc)))?;
        let st = engine::replay(conn, &book, today)?;
        if st.quantity().is_zero() {
            continue;
        }
        let v = engine::value_state(conn, &book, &st, today)?
            .value_base
            .unwrap_or_else(|| st.cost_base());
        let mut holders: Vec<String> = conn
            .prepare_cached("SELECT member_id FROM account_member WHERE account_id = ?1")?
            .query_map([&acc], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        if holders.is_empty() {
            holders.push(UNKNOWN_HOLDER.into());
        }
        let share = v / Decimal::from(holders.len() as u64);
        for h in holders {
            *out.entry(h).or_default() += share;
        }
    }
    Ok(out)
}

/// Títulos garantidos pelo FGC (CDB, RDB, LC, LCI, LCA e poupança).
pub fn fgc_covered(kind: FixedIncomeKind) -> Option<bool> {
    use FixedIncomeKind::*;
    match kind {
        Cdb | Rdb | Lc | Lci | Lca | Poupanca => Some(true),
        Cri | Cra | Debenture | DebentureIncentivada | TesouroSelic | TesouroPrefixado
        | TesouroIpca | TesouroRenda => Some(false),
        // "Outro" pode ser LF (sem FGC) ou um título garantido: não dá para saber.
        Other => None,
    }
}

/// Chave da instituição para o limite do FGC: conglomerado do IF.data, CNPJ ou nome.
pub fn institution_key(conn: &Connection, a: &Asset) -> Result<String> {
    if let Some((i, _)) = bank_issuer(conn, a)? {
        return Ok(format!("ifdata:{}", i.cod_inst));
    }
    if let Some(c) = cnpj_of(a) {
        return Ok(format!("cnpj:{}", &c[..8]));
    }
    if let Some(t) = issuer_text(a) {
        let core = calc::core_name(&t, &params().issuer_stopwords);
        if !core.is_empty() {
            return Ok(format!("nome:{core}"));
        }
    }
    Ok(format!("ativo:{}", a.id))
}

/// Exposição da família aos títulos garantidos: por titular, na instituição do ativo e o
/// total coberto em todas (cada instituição limitada ao teto por CPF).
pub struct Exposure {
    pub at_institution: BTreeMap<String, Decimal>,
    pub covered_total: BTreeMap<String, Decimal>,
    /// Títulos garantidos da família na mesma instituição (inclui o próprio).
    pub titles: usize,
}

pub fn exposure(conn: &Connection, key: &str, today: NaiveDate) -> Result<Exposure> {
    let limit = params().fgc_limit();
    let f = AssetFilter {
        classes: Some(vec![AssetClass::FixedIncome]),
        ..Default::default()
    };
    let mut by_inst: BTreeMap<String, BTreeMap<String, Decimal>> = BTreeMap::new();
    let mut titles = 0;
    for a in crate::assets::list(conn, &f)? {
        let Some(t) = &a.fixed_income else { continue };
        if fgc_covered(t.kind) != Some(true) {
            continue;
        }
        let pos = position_by_holder(conn, &a.id, today)?;
        if pos.is_empty() {
            continue;
        }
        let k = institution_key(conn, &a)?;
        if k == key {
            titles += 1;
        }
        let e = by_inst.entry(k).or_default();
        for (h, v) in pos {
            *e.entry(h).or_default() += v;
        }
    }
    let mut covered_total: BTreeMap<String, Decimal> = BTreeMap::new();
    for m in by_inst.values() {
        for (h, v) in m {
            *covered_total.entry(h.clone()).or_default() += (*v).min(limit);
        }
    }
    Ok(Exposure {
        at_institution: by_inst.remove(key).unwrap_or_default(),
        covered_total,
        titles,
    })
}

/// Nomes dos titulares (membros da família) para as notas.
pub fn holder_names(conn: &Connection) -> Result<BTreeMap<String, String>> {
    let mut stmt = conn.prepare_cached("SELECT id, name FROM member")?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Preços (PU) gravados para o ativo na janela.
pub fn prices(
    conn: &Connection,
    asset_id: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<f64>> {
    Ok(
        crate::research::data::price_series(conn, asset_id, from, to)?
            .into_iter()
            .map(|(_, p)| to_f64(p))
            .collect(),
    )
}
