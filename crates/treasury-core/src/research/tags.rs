//! Características dos papéis como tags combináveis no explorador do Mercado ("Nunca deu
//! prejuízo", "Proventos há 10 anos", "Dívida baixa", "Cumpre Bazin"…). O catálogo é
//! versionado em `methodology/tags/v<versão>/tags.json`: cada tag tem regras por classe, e
//! cada regra é uma lista de condições do screener sobre os indicadores do universo do dia
//! (ou um método da aba Métodos). Nada é gravado: as tags saem do universo na hora.
//!
//! Uma tag descreve o dado público (CVM, B3, SEC); não é recomendação (CVM Res. 20/2021).

use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::methods;
use super::screen::{self, Condition, ScreenQuery, UniverseRow};
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::i18n::{t, t_args};

pub const VERSION: &str = "0.1.0";
const CATALOG: &str = include_str!("../../../../methodology/tags/v0.1.0/tags.json");

#[derive(Debug, Clone, Deserialize)]
struct Rule {
    classes: Vec<AssetClass>,
    all: Vec<Condition>,
}

#[derive(Debug, Clone, Deserialize)]
struct TagDef {
    key: String,
    group: String,
    #[serde(default)]
    rules: Vec<Rule>,
    /// Tag de método: o papel cumpre todos os critérios do método.
    #[serde(default)]
    method: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Catalog {
    tags: Vec<TagDef>,
}

static DEFS: LazyLock<Vec<TagDef>> = LazyLock::new(|| {
    serde_json::from_str::<Catalog>(CATALOG)
        .expect("methodology/tags: catálogo válido")
        .tags
});

fn defs() -> &'static [TagDef] {
    &DEFS
}

/// Uma tag do catálogo, com a regra em texto e quantos papéis a têm hoje.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "AssetTag")]
pub struct TagInfo {
    pub key: String,
    pub label: String,
    /// Grupo para exibir (histórico, proventos, solidez, rentabilidade, score, preço,
    /// mercado, fii, métodos).
    pub group: String,
    pub group_label: String,
    /// A regra em texto, por classe ("Ações: P/L entre 0 e 10").
    pub rule: String,
    pub classes: Vec<AssetClass>,
    /// Papéis do universo com a tag hoje.
    #[ts(type = "number")]
    pub count: u32,
    pub version: String,
}

fn class_label(c: AssetClass) -> String {
    t(match c {
        AssetClass::Fii => "nucleo-tag-classe-fii",
        AssetClass::StockUs => "nucleo-tag-classe-eua",
        _ => "nucleo-tag-classe-acoes",
    })
}

fn rule_text(d: &TagDef, method_names: &BTreeMap<String, String>) -> String {
    if let Some(m) = &d.method {
        return t_args(
            "nucleo-tag-regra-metodo",
            &crate::i18n::args(&[(
                "metodo",
                method_names
                    .get(m)
                    .cloned()
                    .unwrap_or_else(|| m.clone())
                    .into(),
            )]),
        );
    }
    let many = d.rules.len() > 1;
    d.rules
        .iter()
        .map(|r| {
            let text = screen::describe(&ScreenQuery {
                conditions: r.all.clone(),
                ..Default::default()
            });
            if many {
                let classes: Vec<String> = r.classes.iter().map(|c| class_label(*c)).collect();
                let mut seen = HashSet::new();
                let classes: Vec<String> = classes
                    .into_iter()
                    .filter(|c| seen.insert(c.clone()))
                    .collect();
                format!("{}: {text}", classes.join(", "))
            } else {
                text
            }
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn classes_of(d: &TagDef) -> Vec<AssetClass> {
    if d.method.is_some() {
        return methods::list()
            .into_iter()
            .find(|m| Some(&m.key) == d.method.as_ref())
            .map(|m| m.classes)
            .unwrap_or_default();
    }
    let mut v: Vec<AssetClass> = vec![];
    for c in d.rules.iter().flat_map(|r| r.classes.iter()) {
        if !v.contains(c) {
            v.push(*c);
        }
    }
    v
}

/// Tags de cada papel do universo (ticker → chaves, na ordem do catálogo).
pub fn of_universe(rows: &[UniverseRow], as_of: &str) -> BTreeMap<String, Vec<String>> {
    let badges = methods::badges_on(rows, as_of);
    let none = HashSet::new();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for d in defs() {
        for u in rows {
            let has = match &d.method {
                Some(m) => badges.get(&u.ticker).is_some_and(|b| b.contains(m)),
                None => d.rules.iter().any(|r| {
                    r.classes.contains(&u.class)
                        && screen::matches(
                            u,
                            &ScreenQuery {
                                conditions: r.all.clone(),
                                ..Default::default()
                            },
                            &none,
                            &none,
                        )
                }),
            };
            if has {
                out.entry(u.ticker.clone()).or_default().push(d.key.clone());
            }
        }
    }
    out
}

/// O catálogo, com quantos papéis do universo têm cada tag hoje.
pub fn catalog(rows: &[UniverseRow], as_of: &str) -> Vec<TagInfo> {
    let tagged = of_universe(rows, as_of);
    let names: BTreeMap<String, String> = methods::list()
        .into_iter()
        .map(|m| (m.key, m.name))
        .collect();
    defs()
        .iter()
        .map(|d| TagInfo {
            key: d.key.clone(),
            label: t(&format!("nucleo-tag-{}", d.key)),
            group: d.group.clone(),
            group_label: t(&format!("nucleo-tag-grupo-{}", d.group)),
            rule: rule_text(d, &names),
            classes: classes_of(d),
            count: tagged.values().filter(|v| v.contains(&d.key)).count() as u32,
            version: VERSION.into(),
        })
        .collect()
}

/// Rótulo de uma tag (para descrever um filtro).
pub fn label(key: &str) -> String {
    if defs().iter().any(|d| d.key == key) {
        t(&format!("nucleo-tag-{key}"))
    } else {
        key.to_string()
    }
}

/// Recusa tags fora do catálogo.
pub fn validate(keys: &[String]) -> Result<()> {
    let known: HashSet<&str> = defs().iter().map(|d| d.key.as_str()).collect();
    match keys.iter().find(|k| !known.contains(k.as_str())) {
        Some(k) => Err(Error::invalid(t_args(
            "nucleo-tag-erro-desconhecida",
            &crate::i18n::args(&[
                ("tag", k.clone().into()),
                (
                    "tags",
                    defs()
                        .iter()
                        .map(|d| d.key.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .into(),
                ),
            ]),
        ))),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(ticker: &str, class: AssetClass, m: &[(&str, f64)]) -> UniverseRow {
        UniverseRow {
            ticker: ticker.into(),
            asset_id: None,
            issuer_id: None,
            name: ticker.into(),
            class,
            sector: Some("Indústria".into()),
            segment: None,
            currency: "BRL".into(),
            metrics: m.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            score_band: None,
            red_flags: None,
        }
    }

    #[test]
    fn catalog_loads_with_labels_and_rules() {
        let d = defs();
        assert!(d.len() >= 20);
        let keys: HashSet<&str> = d.iter().map(|x| x.key.as_str()).collect();
        assert_eq!(keys.len(), d.len(), "unique keys");
        let fields: HashSet<String> = crate::research::indicators::catalog()
            .into_iter()
            .map(|i| i.key)
            .collect();
        for t in d {
            for c in t.rules.iter().flat_map(|r| r.all.iter()) {
                assert!(
                    fields.contains(&c.field)
                        || ["score_band", "red_flags"].contains(&c.field.as_str()),
                    "{}: {}",
                    t.key,
                    c.field
                );
            }
            if let Some(m) = &t.method {
                assert!(methods::list().iter().any(|x| &x.key == m), "{m}");
            }
        }
        let c = catalog(&[], "2026-10-27");
        let never = c.iter().find(|x| x.key == "nunca_prejuizo").unwrap();
        assert!(!never.label.starts_with("nucleo-"), "{}", never.label);
        assert!(!never.group_label.starts_with("nucleo-"));
        assert!(
            never.rule.contains("prejuízo") || never.rule.contains("10"),
            "{}",
            never.rule
        );
        let dy = c.iter().find(|x| x.key == "dy_alto").unwrap();
        assert!(dy.rule.contains("FIIs"), "rule per class: {}", dy.rule);
    }

    #[test]
    fn tags_follow_the_rule_of_each_class() {
        let rows = vec![
            row(
                "BOMA3",
                AssetClass::StockBr,
                &[
                    ("loss_years_10y", 0.0),
                    ("eps_growth_10y", 0.4),
                    ("dy", 0.07),
                    ("dividend_years", 12.0),
                    ("net_debt_ebitda", 0.5),
                ],
            ),
            // Without the 10 fiscal years: "never made a loss" does not apply.
            row(
                "NOVA3",
                AssetClass::StockBr,
                &[("loss_years_10y", 0.0), ("dy", 0.07)],
            ),
            // FII: a 7% DY is not high for an FII (the FII rule asks for 10%).
            row(
                "XPTO11",
                AssetClass::Fii,
                &[("dy", 0.07), ("holders", 200_000.0)],
            ),
        ];
        let t = of_universe(&rows, "2026-10-27");
        let has = |tk: &str, k: &str| t.get(tk).is_some_and(|v| v.iter().any(|x| x == k));
        assert!(has("BOMA3", "nunca_prejuizo"));
        assert!(has("BOMA3", "proventos_10_anos") && has("BOMA3", "proventos_5_anos"));
        assert!(has("BOMA3", "dy_alto") && has("BOMA3", "divida_baixa"));
        assert!(!has("NOVA3", "nunca_prejuizo"));
        assert!(has("NOVA3", "dy_alto"));
        assert!(!has("XPTO11", "dy_alto"));
        assert!(has("XPTO11", "muitos_cotistas"));
        assert!(validate(&["nunca_prejuizo".into()]).is_ok());
        assert!(validate(&["inventada".into()]).is_err());
    }
}
