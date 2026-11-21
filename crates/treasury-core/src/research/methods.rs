//! Métodos do explorador de mercado (docs/design/market-explorer.md, seção 5): cada
//! método é uma receita versionada em `methodology/screens/v<versão>/*.json` (classes, setores
//! que não se aplicam, filtros de higiene, critérios e ordem), nunca no código. O núcleo
//! roda a receita sobre o universo do dia e diz, papel a papel, quais critérios ele cumpriu e
//! por quanto ficou de fora.
//!
//! Cumprir um método é cumprir critérios públicos de um livro ou artigo; não é recomendação
//! de investimento (CVM Res. 20/2021).

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::indicators::{self, BazinParams};
use super::screen::{self, UniverseRow};
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

pub const VERSION: &str = "0.1.0";

const RECIPES: &[&str] = &[
    include_str!("../../../../methodology/screens/v0.1.0/graham_defensivo.json"),
    include_str!("../../../../methodology/screens/v0.1.0/bazin.json"),
    include_str!("../../../../methodology/screens/v0.1.0/magic_formula.json"),
    include_str!("../../../../methodology/screens/v0.1.0/qualidade_preco_justo.json"),
    include_str!("../../../../methodology/screens/v0.1.0/fii_desconto.json"),
];

#[derive(Debug, Clone, Deserialize)]
struct Recipe {
    key: String,
    version: String,
    classes: Vec<AssetClass>,
    #[serde(default)]
    exclude_sectors: Vec<String>,
    #[serde(default)]
    hygiene: Hygiene,
    criteria: Vec<Criterion>,
    order: Order,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct Hygiene {
    min_liquidity: Option<f64>,
    min_liquidity_fii: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct Criterion {
    id: String,
    field: Option<String>,
    #[serde(default)]
    fields: Vec<String>,
    op: String,
    #[serde(default)]
    value: Value,
    /// Sem o dado, o critério não derruba o papel (vale o que dá para conferir).
    #[serde(default)]
    optional: bool,
    /// Adaptado à B3 (o original está na explicação do método).
    #[serde(default)]
    adapted: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OrderKey {
    field: String,
    dir: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Order {
    RankSum { rank_sum: Vec<OrderKey> },
    Field { field: String, dir: String },
}

fn recipes() -> Vec<Recipe> {
    RECIPES
        .iter()
        .map(|s| serde_json::from_str(s).expect("receita de método válida"))
        .collect()
}

fn recipe(key: &str) -> Result<Recipe> {
    recipes().into_iter().find(|r| r.key == key).ok_or_else(|| {
        Error::invalid(t_args(
            "nucleo-metodo-erro-desconhecido",
            &args(&[("metodo", key.to_string().into())]),
        ))
    })
}

/// Um critério como a tela mostra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MethodCriterion")]
pub struct CriterionInfo {
    pub id: String,
    /// "Liquidez corrente ≥ 2".
    pub label: String,
    pub optional: bool,
    /// Adaptado à B3: o que o livro pede.
    pub adapted: Option<String>,
}

/// Um método com a receita inteira, para a interface e o agente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ScreenMethod")]
pub struct MethodInfo {
    pub key: String,
    pub version: String,
    pub name: String,
    /// A ideia em uma frase.
    pub idea: String,
    /// Livro ou artigo.
    pub reference: String,
    /// A quem não se aplica (texto).
    pub not_applicable: Option<String>,
    pub classes: Vec<AssetClass>,
    pub hygiene: String,
    pub criteria: Vec<CriterionInfo>,
    /// Como ordena.
    pub order: String,
}

/// Um critério conferido num papel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MethodCheck")]
pub struct Check {
    pub id: String,
    pub label: String,
    #[ts(type = "number | null")]
    pub actual: Option<f64>,
    /// Valor como a tela mostra ("16,2", "4,1%", "—").
    pub shown: String,
    pub passed: bool,
    /// Faltou o dado (num critério opcional, não derruba).
    pub missing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MethodRow")]
pub struct Row {
    pub ticker: String,
    pub name: String,
    pub class: AssetClass,
    pub sector: Option<String>,
    pub asset_id: Option<String>,
    /// Posição na ordem do método (1 = primeiro).
    pub rank: u32,
    #[ts(type = "number | null")]
    pub order_value: Option<f64>,
    #[ts(type = "number | null")]
    pub score: Option<f64>,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MethodRun")]
pub struct Run {
    pub method: MethodInfo,
    pub as_of: String,
    /// Papéis avaliados (classe e setor do método, depois da higiene).
    pub evaluated: u32,
    pub passing: u32,
    pub rows: Vec<Row>,
}

/// Por que um papel passa ou fica de fora de um método.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MethodExplain")]
pub struct Explain {
    pub method_key: String,
    pub ticker: String,
    pub passed: bool,
    /// Fora antes dos critérios: classe, setor ou higiene (texto).
    pub excluded: Option<String>,
    pub checks: Vec<Check>,
    pub rank: Option<u32>,
}

/// Número no idioma do app com `d` casas (0, 1 ou 2), agrupando os milhares.
fn num(v: f64, d: usize) -> String {
    let cents = if d == 0 {
        v.round() * 100.0
    } else {
        (v * 100.0).round()
    };
    let s = crate::i18n::number(crate::i18n::current(), cents as i64);
    match d {
        0 => s[..s.len() - 3].to_string(),
        1 => s[..s.len() - 1].to_string(),
        _ => s,
    }
}

fn field_label(f: &str) -> String {
    match indicators::def(f) {
        Some(d) => d.label(),
        None => t(&format!("nucleo-metodo-campo-{f}")),
    }
}

fn fmt(field: &str, v: f64) -> String {
    let percent =
        indicators::def(field).is_some_and(|d| d.unit == indicators::IndicatorUnit::Percent);
    if percent {
        format!("{}%", num(v * 100.0, 1))
    } else if v.fract() == 0.0 && v.abs() < 1e9 {
        num(v, 0)
    } else {
        num(v, 2)
    }
}

fn criterion_label(c: &Criterion) -> String {
    let field = c.field.clone().unwrap_or_default();
    let n = |v: &Value| v.as_f64().map(|x| fmt(&field, x)).unwrap_or_default();
    let name = field_label(&field);
    match c.op.as_str() {
        "gte" => format!("{name} ≥ {}", n(&c.value)),
        "gt" => format!("{name} > {}", n(&c.value)),
        "lte" => format!("{name} ≤ {}", n(&c.value)),
        "lt" => format!("{name} < {}", n(&c.value)),
        "eq" => format!("{name} = {}", n(&c.value)),
        "between" => {
            let a = c.value.as_array().cloned().unwrap_or_default();
            format!(
                "{name} {} – {}",
                a.first().map(n).unwrap_or_default(),
                a.get(1).map(n).unwrap_or_default()
            )
        }
        "percentile_gte" => t_args(
            "nucleo-metodo-criterio-percentil",
            &args(&[
                ("campo", name.into()),
                (
                    "pct",
                    num(c.value.as_f64().unwrap_or(0.0) * 100.0, 0).into(),
                ),
            ]),
        ),
        "product_lte" => {
            let names: Vec<String> = c.fields.iter().map(|f| field_label(f)).collect();
            format!("{} ≤ {}", names.join(" × "), n(&c.value))
        }
        "missing" => t_args(
            "nucleo-metodo-criterio-sem",
            &args(&[("campo", name.into())]),
        ),
        _ => name,
    }
}

fn info(r: &Recipe) -> MethodInfo {
    let k = |suffix: &str| t(&format!("nucleo-metodo-{}{suffix}", r.key));
    let na = k("-nao-se-aplica");
    let hygiene = match (r.hygiene.min_liquidity, r.hygiene.min_liquidity_fii) {
        (Some(a), Some(f)) => t_args(
            "nucleo-metodo-higiene-acoes-fiis",
            &args(&[("acoes", num(a, 0).into()), ("fiis", num(f, 0).into())]),
        ),
        (Some(a), None) => t_args(
            "nucleo-metodo-higiene",
            &args(&[("liquidez", num(a, 0).into())]),
        ),
        _ => t("nucleo-metodo-higiene-nenhuma"),
    };
    MethodInfo {
        key: r.key.clone(),
        version: r.version.clone(),
        name: k(""),
        idea: k("-ideia"),
        reference: k("-fonte"),
        not_applicable: (!na.is_empty() && na != "-").then_some(na),
        classes: r.classes.clone(),
        hygiene,
        criteria: r
            .criteria
            .iter()
            .map(|c| CriterionInfo {
                id: c.id.clone(),
                label: criterion_label(c),
                optional: c.optional,
                adapted: c
                    .adapted
                    .then(|| t(&format!("nucleo-metodo-{}-{}-original", r.key, c.id))),
            })
            .collect(),
        order: k("-ordem"),
    }
}

/// Métodos com a receita, na ordem de exibição.
pub fn list() -> Vec<MethodInfo> {
    recipes().iter().map(info).collect()
}

fn excluded(r: &Recipe, u: &UniverseRow) -> Option<String> {
    if !r.classes.contains(&u.class) {
        return Some(t("nucleo-metodo-fora-classe"));
    }
    let sector = super::norm(u.sector.as_deref().unwrap_or_default());
    let segment = super::norm(u.segment.as_deref().unwrap_or_default());
    if r.exclude_sectors
        .iter()
        .any(|s| sector.contains(s.as_str()) || segment.contains(s.as_str()))
    {
        return Some(t_args(
            "nucleo-metodo-fora-setor",
            &args(&[("setor", u.sector.clone().unwrap_or_default().into())]),
        ));
    }
    let min = if u.class == AssetClass::Fii {
        r.hygiene.min_liquidity_fii.or(r.hygiene.min_liquidity)
    } else {
        r.hygiene.min_liquidity
    };
    if let Some(min) = min
        && u.metrics.get("liquidity").is_none_or(|l| *l < min)
    {
        return Some(t_args(
            "nucleo-metodo-fora-liquidez",
            &args(&[("minimo", num(min, 0).into())]),
        ));
    }
    None
}

/// Percentil (0–1) de cada papel num campo, entre os elegíveis do método.
fn percentiles(rows: &[&UniverseRow], field: &str) -> HashMap<String, f64> {
    let mut v: Vec<(&str, f64)> = rows
        .iter()
        .filter_map(|r| r.metrics.get(field).map(|x| (r.ticker.as_str(), *x)))
        .collect();
    v.sort_by(|a, b| a.1.total_cmp(&b.1));
    let n = v.len().max(1) as f64;
    v.iter()
        .enumerate()
        .map(|(i, (t0, _))| {
            (
                t0.to_string(),
                if n > 1.0 { i as f64 / (n - 1.0) } else { 1.0 },
            )
        })
        .collect()
}

fn check(c: &Criterion, u: &UniverseRow, pct: &HashMap<String, HashMap<String, f64>>) -> Check {
    let field = c.field.clone().unwrap_or_default();
    let label = criterion_label(c);
    let val = |v: &Value| v.as_f64();
    if c.op == "missing" {
        let text = match field.as_str() {
            "red_flags" => u.red_flags.clone(),
            "score_band" => u.score_band.clone(),
            f => u.metrics.get(f).map(|x| x.to_string()),
        };
        return Check {
            id: c.id.clone(),
            label,
            actual: None,
            shown: text.clone().unwrap_or_else(|| "—".into()),
            passed: text.is_none(),
            missing: false,
        };
    }
    let actual = match c.op.as_str() {
        "product_lte" => c
            .fields
            .iter()
            .map(|f| u.metrics.get(f).copied())
            .collect::<Option<Vec<f64>>>()
            .map(|v| v.iter().product()),
        "percentile_gte" => pct.get(&field).and_then(|m| m.get(&u.ticker)).copied(),
        _ => u.metrics.get(&field).copied(),
    };
    let Some(x) = actual else {
        return Check {
            id: c.id.clone(),
            label,
            actual: None,
            shown: "—".into(),
            passed: c.optional,
            missing: true,
        };
    };
    let passed = match c.op.as_str() {
        "gte" | "percentile_gte" => val(&c.value).is_some_and(|v| x >= v),
        "gt" => val(&c.value).is_some_and(|v| x > v),
        "lte" | "product_lte" => val(&c.value).is_some_and(|v| x <= v),
        "lt" => val(&c.value).is_some_and(|v| x < v),
        "eq" => val(&c.value).is_some_and(|v| (x - v).abs() < 1e-9),
        "between" => c.value.as_array().is_some_and(|a| {
            let lo = a.first().and_then(Value::as_f64).unwrap_or(f64::MIN);
            let hi = a.get(1).and_then(Value::as_f64).unwrap_or(f64::MAX);
            x >= lo && x <= hi
        }),
        _ => false,
    };
    let shown = match c.op.as_str() {
        "percentile_gte" => format!("{}%", num(x * 100.0, 0)),
        "product_lte" => num(x, 1),
        _ => fmt(&field, x),
    };
    Check {
        id: c.id.clone(),
        label,
        actual: Some(x),
        shown,
        passed,
        missing: false,
    }
}

/// Cada elegível com as suas conferências; e quem ficou de fora antes dos critérios (motivo).
type Evaluated<'a> = (Vec<(&'a UniverseRow, Vec<Check>)>, HashMap<String, String>);

/// Elegíveis, percentis e as conferências de cada papel.
fn evaluate<'a>(r: &Recipe, universe: &'a [UniverseRow]) -> Evaluated<'a> {
    let mut out_of: HashMap<String, String> = HashMap::new();
    let mut eligible = vec![];
    for u in universe {
        match excluded(r, u) {
            Some(why) => {
                out_of.insert(u.ticker.clone(), why);
            }
            None => eligible.push(u),
        }
    }
    let pct: HashMap<String, HashMap<String, f64>> = r
        .criteria
        .iter()
        .filter(|c| c.op == "percentile_gte")
        .filter_map(|c| c.field.clone())
        .map(|f| {
            // Percentil dentro da mesma classe (ações com ações, FIIs com FIIs).
            let mut m = HashMap::new();
            for class in &r.classes {
                let same: Vec<&UniverseRow> = eligible
                    .iter()
                    .copied()
                    .filter(|u| u.class == *class)
                    .collect();
                m.extend(percentiles(&same, &f));
            }
            (f, m)
        })
        .collect();
    let checked = eligible
        .into_iter()
        .map(|u| (u, r.criteria.iter().map(|c| check(c, u, &pct)).collect()))
        .collect();
    (checked, out_of)
}

/// Ordem do método entre os que passaram: posição (1 = primeiro) e o valor de ordenação.
fn rank(r: &Recipe, passing: &[&UniverseRow]) -> BTreeMap<String, (u32, Option<f64>)> {
    let pos = |field: &str, desc: bool| -> HashMap<String, usize> {
        let mut v: Vec<(&str, f64)> = passing
            .iter()
            .map(|u| {
                (
                    u.ticker.as_str(),
                    u.metrics
                        .get(field)
                        .copied()
                        .unwrap_or(if desc { f64::MIN } else { f64::MAX }),
                )
            })
            .collect();
        v.sort_by(|a, b| {
            let o = a.1.total_cmp(&b.1);
            (if desc { o.reverse() } else { o }).then(a.0.cmp(b.0))
        });
        v.iter()
            .enumerate()
            .map(|(i, (t0, _))| (t0.to_string(), i + 1))
            .collect()
    };
    let mut scored: Vec<(String, f64, Option<f64>)> = match &r.order {
        Order::Field { field, dir } => {
            let p = pos(field, dir == "desc");
            passing
                .iter()
                .map(|u| {
                    (
                        u.ticker.clone(),
                        p[&u.ticker] as f64,
                        u.metrics.get(field).copied(),
                    )
                })
                .collect()
        }
        Order::RankSum { rank_sum } => {
            let ps: Vec<HashMap<String, usize>> = rank_sum
                .iter()
                .map(|k| pos(&k.field, k.dir == "desc"))
                .collect();
            passing
                .iter()
                .map(|u| {
                    let sum: usize = ps.iter().map(|p| p[&u.ticker]).sum();
                    (u.ticker.clone(), sum as f64, Some(sum as f64))
                })
                .collect()
        }
    };
    scored.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    scored
        .into_iter()
        .enumerate()
        .map(|(i, (t0, _, v))| (t0, (i as u32 + 1, v)))
        .collect()
}

fn run_on(r: &Recipe, universe: &[UniverseRow], as_of: &str) -> Run {
    let (checked, _) = evaluate(r, universe);
    let evaluated = checked.len() as u32;
    let passing: Vec<(&UniverseRow, Vec<Check>)> = checked
        .into_iter()
        .filter(|(_, c)| c.iter().all(|x| x.passed))
        .collect();
    let ranks = rank(r, &passing.iter().map(|(u, _)| *u).collect::<Vec<_>>());
    let mut rows: Vec<Row> = passing
        .into_iter()
        .map(|(u, checks)| {
            let (rank, order_value) = ranks.get(&u.ticker).copied().unwrap_or((0, None));
            Row {
                ticker: u.ticker.clone(),
                name: u.name.clone(),
                class: u.class,
                sector: u.sector.clone(),
                asset_id: u.asset_id.clone(),
                rank,
                order_value,
                score: u.metrics.get("score").copied(),
                checks,
            }
        })
        .collect();
    rows.sort_by_key(|r0| r0.rank);
    Run {
        method: info(r),
        as_of: as_of.to_string(),
        evaluated,
        passing: rows.len() as u32,
        rows,
    }
}

/// Roda um método sobre o universo do dia.
pub fn run(conn: &Connection, key: &str, today: NaiveDate, bazin: BazinParams) -> Result<Run> {
    let r = recipe(key)?;
    let (universe, as_of) = screen::universe(conn, today, bazin)?;
    Ok(run_on(&r, &universe, &as_of))
}

/// Por que um papel passa ou fica de fora de um método, critério a critério.
pub fn explain(
    conn: &Connection,
    key: &str,
    ticker: &str,
    today: NaiveDate,
    bazin: BazinParams,
) -> Result<Explain> {
    let r = recipe(key)?;
    let ticker = ticker.trim().to_uppercase();
    let (universe, as_of) = screen::universe(conn, today, bazin)?;
    let Some(u) = universe.iter().find(|u| u.ticker == ticker) else {
        return Err(Error::not_found(t_args(
            "nucleo-pesquisa-erro-fora-do-universo",
            &args(&[("ticker", ticker.clone().into())]),
        )));
    };
    let (checked, out_of) = evaluate(&r, &universe);
    if let Some(why) = out_of.get(&ticker) {
        return Ok(Explain {
            method_key: r.key,
            ticker,
            passed: false,
            excluded: Some(why.clone()),
            checks: vec![],
            rank: None,
        });
    }
    let checks = checked
        .iter()
        .find(|(x, _)| x.ticker == u.ticker)
        .map(|(_, c)| c.clone())
        .unwrap_or_default();
    let passed = checks.iter().all(|c| c.passed);
    let rank = passed
        .then(|| {
            run_on(&r, &universe, &as_of)
                .rows
                .iter()
                .find(|x| x.ticker == ticker)
                .map(|x| x.rank)
        })
        .flatten();
    Ok(Explain {
        method_key: r.key,
        ticker,
        passed,
        excluded: None,
        checks,
        rank,
    })
}

/// Selos: ticker → métodos que ele cumpre hoje.
pub fn badges(
    conn: &Connection,
    today: NaiveDate,
    bazin: BazinParams,
) -> Result<BTreeMap<String, Vec<String>>> {
    let (universe, as_of) = screen::universe(conn, today, bazin)?;
    Ok(badges_on(&universe, &as_of))
}

/// Selos sobre um universo já carregado.
pub fn badges_on(universe: &[UniverseRow], as_of: &str) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in recipes() {
        for row in run_on(&r, universe, as_of).rows {
            out.entry(row.ticker).or_default().push(r.key.clone());
        }
    }
    out
}

/// Quantos papéis cumprem cada método hoje (cartões da aba Métodos).
pub fn counts(
    conn: &Connection,
    today: NaiveDate,
    bazin: BazinParams,
) -> Result<BTreeMap<String, u32>> {
    let (universe, as_of) = screen::universe(conn, today, bazin)?;
    Ok(recipes()
        .iter()
        .map(|r| (r.key.clone(), run_on(r, &universe, &as_of).passing))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(t: &str, class: AssetClass, sector: &str, m: &[(&str, f64)]) -> UniverseRow {
        UniverseRow {
            ticker: t.into(),
            asset_id: None,
            issuer_id: None,
            name: format!("{t} S.A."),
            class,
            sector: Some(sector.into()),
            segment: None,
            currency: "BRL".into(),
            metrics: m.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            score_band: None,
            red_flags: None,
        }
    }

    #[test]
    fn every_recipe_loads_with_its_texts() {
        let all = list();
        assert_eq!(all.len(), 5);
        for m in &all {
            assert_eq!(m.version, VERSION);
            assert!(!m.name.starts_with("nucleo-"), "{}", m.name);
            assert!(!m.criteria.is_empty());
            for c in &m.criteria {
                if let Some(a) = &c.adapted {
                    assert!(!a.starts_with("nucleo-"), "{a}");
                }
            }
        }
    }

    #[test]
    fn magic_formula_sums_both_ranks_and_skips_financials() {
        let r = recipe("magic_formula").unwrap();
        let liq = ("liquidity", 5e6);
        let universe = vec![
            row(
                "AAAA3",
                AssetClass::StockBr,
                "Varejo",
                &[liq, ("earnings_yield", 0.20), ("magic_roc", 0.10)],
            ),
            row(
                "BBBB3",
                AssetClass::StockBr,
                "Varejo",
                &[liq, ("earnings_yield", 0.10), ("magic_roc", 0.40)],
            ),
            row(
                "CCCC3",
                AssetClass::StockBr,
                "Varejo",
                &[liq, ("earnings_yield", 0.15), ("magic_roc", 0.30)],
            ),
            row(
                "BANC4",
                AssetClass::StockBr,
                "Bancos",
                &[liq, ("earnings_yield", 0.5), ("magic_roc", 0.9)],
            ),
            row(
                "ILIQ3",
                AssetClass::StockBr,
                "Varejo",
                &[
                    ("liquidity", 1e3),
                    ("earnings_yield", 0.9),
                    ("magic_roc", 0.9),
                ],
            ),
            row(
                "NEGA3",
                AssetClass::StockBr,
                "Varejo",
                &[liq, ("earnings_yield", -0.1), ("magic_roc", 0.3)],
            ),
        ];
        let run = run_on(&r, &universe, "2026-10-27");
        assert_eq!(
            run.evaluated, 4,
            "bank and illiquid are left out before the criteria"
        );
        // EY: A 1, C 2, B 3; ROC: B 1, C 2, A 3 → sums A 4, B 4, C 4 → tie broken by ticker.
        let order: Vec<&str> = run.rows.iter().map(|x| x.ticker.as_str()).collect();
        assert_eq!(order, ["AAAA3", "BBBB3", "CCCC3"]);
        assert_eq!(run.rows[0].order_value, Some(4.0));
    }

    #[test]
    fn graham_checks_product_and_percentile_and_explains_what_failed() {
        let r = recipe("graham_defensivo").unwrap();
        let base = |pl: f64, cap: f64| {
            vec![
                ("liquidity", 5e6),
                ("market_cap", cap),
                ("current_ratio", 2.5),
                ("loss_years_10y", 0.0),
                ("dividend_years", 12.0),
                ("eps_growth_10y", 0.5),
                ("pl", pl),
                ("pvp", 1.2),
                ("graham_upside", 0.3),
            ]
        };
        let universe = vec![
            row("GOOD3", AssetClass::StockBr, "Indústria", &base(10.0, 9e9)),
            row("CARO3", AssetClass::StockBr, "Indústria", &base(20.0, 8e9)),
            row("PEQU3", AssetClass::StockBr, "Indústria", &base(8.0, 1e8)),
        ];
        let run = run_on(&r, &universe, "2026-10-27");
        assert_eq!(
            run.rows
                .iter()
                .map(|x| x.ticker.as_str())
                .collect::<Vec<_>>(),
            ["GOOD3"]
        );
        let (checked, _) = evaluate(&r, &universe);
        let caro = &checked.iter().find(|(u, _)| u.ticker == "CARO3").unwrap().1;
        let failed: Vec<&str> = caro
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.id.as_str())
            .collect();
        // Mid-universe (percentile 50 < 67), P/E 20 > 15 and P/E × P/B = 24 > 22.5.
        assert_eq!(failed, ["tamanho", "pl", "pl_pvp"]);
        let pl_pvp = caro.iter().find(|c| c.id == "pl_pvp").unwrap();
        assert_eq!(pl_pvp.actual, Some(24.0));
        let pequena = &checked.iter().find(|(u, _)| u.ticker == "PEQU3").unwrap().1;
        assert!(pequena.iter().any(|c| c.id == "tamanho" && !c.passed));
    }

    #[test]
    fn optional_criteria_pass_without_data() {
        let r = recipe("fii_desconto").unwrap();
        let universe = vec![row(
            "FUND11",
            AssetClass::Fii,
            "Fundos imobiliários",
            &[
                ("liquidity", 1e6),
                ("pvp", 0.9),
                ("dy", 0.10),
                ("holders", 20000.0),
            ],
        )];
        let run = run_on(&r, &universe, "2026-10-27");
        assert_eq!(run.passing, 1);
        assert!(
            run.rows[0]
                .checks
                .iter()
                .any(|c| c.id == "vacancia" && c.missing && c.passed)
        );
    }
}
