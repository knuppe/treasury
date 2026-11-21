//! Dados de referência por país, versionados e embutidos no app (`data/<país>.json`): as
//! fontes, as alíquotas de consumo por grupo nos três cenários, os dados por estado (ICMS e
//! IPVA no Brasil, imposto sobre vendas nos EUA, IVA na União Europeia), as regras de renda
//! do trabalho por ano e o contexto (carga tributária do país). Cada número aponta para as
//! fontes de onde saiu; a versão e a data vão para a tela, para a estimativa ser auditável.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::rules::PayrollRules;
use crate::error::{Error, Result};

/// Uma fonte citada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxSource {
    pub id: String,
    pub publisher: String,
    pub title: String,
    pub url: String,
    /// Data de publicação ou atualização da fonte (`YYYY-MM-DD` ou `YYYY-MM`).
    #[ts(optional)]
    pub published: Option<String>,
    /// Quando a fonte foi consultada para esta versão.
    pub accessed: String,
}

/// Alíquota de um cenário: % do preço final, ou uma base do estado escolhido
/// (`vat_standard`, `vat_reduced`, `vat_super_reduced`, `sales_tax`, `exempt`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum RateSpec {
    Percent(f64),
    Basis(String),
}

/// Grupo de consumo: quanto do preço final é imposto, nos três cenários.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConsumptionGroup {
    pub key: String,
    pub label: String,
    /// `null` nos três: sem dado confiável (o gasto fica fora, à vista, como "sem dados").
    pub low: Option<RateSpec>,
    pub central: Option<RateSpec>,
    pub high: Option<RateSpec>,
    /// Produtos e serviços por trás dos números.
    pub basis: String,
    pub source_ids: Vec<String>,
    /// Brasil: mercadorias na alíquota modal do ICMS (recebem o ajuste do estado).
    #[serde(default)]
    pub icms_modal: bool,
}

/// Dados de um estado (UF, estado americano ou país da União Europeia).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxRegion {
    pub code: String,
    pub name: String,
    /// Alíquota modal do ICMS (%), com fundo de pobreza quando houver.
    #[ts(optional)]
    pub icms_modal: Option<f64>,
    /// IPVA de automóveis (% do valor venal).
    #[ts(optional)]
    pub ipva_car: Option<f64>,
    #[ts(optional)]
    pub ipva_motorcycle: Option<f64>,
    /// Isenção de IPVA por idade de fabricação na lei do estado (anos).
    #[serde(default)]
    #[ts(optional)]
    pub ipva_age_exempt_years: Option<u32>,
    /// Faixas e isenções do IPVA no estado, em uma frase.
    #[serde(default)]
    #[ts(optional)]
    pub ipva_notes: Option<String>,
    /// Imposto sobre vendas (estadual + local médio, %).
    #[ts(optional)]
    pub sales_tax: Option<f64>,
    #[ts(optional)]
    pub vat_standard: Option<f64>,
    #[ts(optional)]
    pub vat_reduced: Option<f64>,
    #[ts(optional)]
    pub vat_super_reduced: Option<f64>,
    pub source_ids: Vec<String>,
}

/// Contexto: a carga tributária do país, para comparação.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxContext {
    pub label: String,
    pub value: f64,
    pub year: i32,
    pub source_ids: Vec<String>,
}

/// Dias do ano trabalhados só para pagar impostos, na média do país (contexto, com fonte).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkDaysReference {
    pub year: i32,
    #[ts(type = "number")]
    pub days: u32,
    /// Até que dia do ano (`YYYY-MM-DD`).
    pub until: String,
    /// Carga tributária efetiva usada no estudo (%).
    pub share: f64,
    pub label: String,
    pub source_ids: Vec<String>,
}

/// Uma função de governo e a parte dela na despesa total.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendingFunction {
    pub key: String,
    pub label: String,
    /// % da despesa total do governo geral.
    pub share: f64,
    /// % do PIB.
    pub gdp_share: f64,
}

/// Para onde vai o dinheiro público: a despesa do governo geral (União, estados e municípios)
/// por função (Brasil: Tesouro Nacional, classificação COFOG).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpendingByFunction {
    pub year: i32,
    pub label: String,
    pub functions: Vec<SpendingFunction>,
    pub source_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxDataset {
    pub country: String,
    pub name: String,
    pub currency: String,
    /// Versão dos dados (`AAAA-MM`): vai junto em toda estimativa.
    pub version: String,
    /// Última revisão dos números (`YYYY-MM-DD`): passa de [`STALE_MONTHS`], a tela avisa.
    pub updated: String,
    /// Nível de detalhe: `completo` ou `esboço`.
    pub coverage: String,
    pub sources: Vec<TaxSource>,
    pub consumption: Vec<ConsumptionGroup>,
    pub regions: Vec<TaxRegion>,
    /// Brasil: ICMS de referência das alíquotas de consumo (o ajuste é a diferença do
    /// estado para ela).
    #[serde(default)]
    #[ts(optional)]
    pub reference_icms: Option<f64>,
    #[serde(default)]
    pub payroll: Vec<PayrollRules>,
    #[serde(default)]
    #[ts(optional)]
    pub context: Option<TaxContext>,
    /// Referência nacional dos dias de trabalho para pagar impostos (Brasil: a carga tributária bruta do Tesouro Nacional, em dias do ano).
    #[serde(default)]
    #[ts(optional)]
    pub work_days: Option<WorkDaysReference>,
    /// Despesa do governo geral por função (Brasil), para "para onde vai o imposto".
    #[serde(default)]
    #[ts(optional)]
    pub spending: Option<SpendingByFunction>,
    /// Como os números foram escolhidos, em frases curtas.
    pub methodology: Vec<String>,
    pub disclaimer: String,
}

/// Meses sem revisão até a tela marcar os dados como desatualizados.
pub const STALE_MONTHS: i64 = 6;

const BR: &str = include_str!("data/br.json");
const US: &str = include_str!("data/us.json");
const EU: &str = include_str!("data/eu.json");
/// Folha dos países adaptados sem dados de impostos sobre consumo (Reino Unido, Canadá,
/// Argentina e Chile; docs/international.md).
const PAYROLL: &str = include_str!("data/payroll.json");

#[derive(Debug, Clone, Deserialize)]
struct PayrollOnly {
    sources: Vec<TaxSource>,
    payroll: Vec<PayrollRules>,
}

/// As regras de folha de um desses países (`region` = o código do país) e as fontes.
pub fn payroll_only() -> Result<(Vec<TaxSource>, Vec<PayrollRules>)> {
    let d: PayrollOnly = serde_json::from_str(PAYROLL)
        .map_err(|e| Error::invalid(format!("dados da folha inválidos: {e}")))?;
    for r in &d.payroll {
        if r.region.is_none()
            || r.source_ids.is_empty()
            || !r
                .source_ids
                .iter()
                .all(|id| d.sources.iter().any(|s| &s.id == id))
            || r.ir_monthly
                .iter()
                .any(|b| !(0.0..=100.0).contains(&b.rate))
            || r.inss_bands
                .iter()
                .any(|b| !(0.0..=100.0).contains(&b.rate))
        {
            return Err(Error::invalid(format!(
                "dados da folha de {}: região ou fontes",
                r.region.as_deref().unwrap_or("?")
            )));
        }
    }
    Ok((d.sources, d.payroll))
}

/// Países com dados (o código da União Europeia é `EU`; o estado é o país-membro), com os
/// textos no idioma da instalação.
pub fn all() -> Result<Vec<TaxDataset>> {
    [BR, US, EU]
        .iter()
        .map(|s| parse(s).map(localize))
        .collect()
}

/// Chave do catálogo de um texto dos dados (`nucleo-impostos-…`).
fn data_key(d: &TaxDataset, what: &str) -> String {
    format!("nucleo-impostos-dados-{}-{what}", d.country.to_lowercase())
}

fn group_key(group: &str) -> String {
    format!("nucleo-impostos-grupo-{}", group.replace('_', "-"))
}

/// O texto dos dados no idioma da instalação. A tradução só vale enquanto o pt-BR da chave é
/// o texto do arquivo: dados atualizados sem a tradução nova ficam como estão (em pt-BR).
fn localized(key: &str, original: &str) -> String {
    use crate::i18n::{Locale, current, has_key, t, tr};
    if current() == Locale::PtBr
        || !has_key(Locale::PtBr, key)
        || tr(Locale::PtBr, key, None) != original
    {
        return original.to_string();
    }
    t(key)
}

/// Nome, grupos de consumo, metodologia, aviso e rótulos de contexto no idioma da instalação
/// (fontes, estados e bases legais ficam como estão).
fn localize(mut d: TaxDataset) -> TaxDataset {
    d.name = localized(&data_key(&d, "nome"), &d.name);
    d.disclaimer = localized(&data_key(&d, "aviso"), &d.disclaimer);
    let keys: Vec<String> = (1..=d.methodology.len())
        .map(|i| data_key(&d, &format!("metodo-{i}")))
        .collect();
    for (m, k) in d.methodology.iter_mut().zip(keys) {
        *m = localized(&k, m);
    }
    let (context, days) = (data_key(&d, "contexto"), data_key(&d, "dias"));
    if let Some(c) = &mut d.context {
        c.label = localized(&context, &c.label);
    }
    if let Some(w) = &mut d.work_days {
        w.label = localized(&days, &w.label);
    }
    let spending = data_key(&d, "destino");
    if let Some(sp) = &mut d.spending {
        sp.label = localized(&spending, &sp.label);
        for f in &mut sp.functions {
            f.label = localized(
                &format!("nucleo-impostos-funcao-{}", f.key.replace('_', "-")),
                &f.label,
            );
        }
    }
    for g in &mut d.consumption {
        g.label = localized(&group_key(&g.key), &g.label);
    }
    d
}

/// Os dados que um cofre pode usar: os do Brasil (e o que só existe para o Brasil) só aparecem
/// quando a família mora no Brasil.
pub fn all_for(brazil: bool) -> Result<Vec<TaxDataset>> {
    Ok(all()?
        .into_iter()
        .filter(|d| brazil || d.country != "BR")
        .collect())
}

pub fn get(country: &str) -> Result<TaxDataset> {
    all()?
        .into_iter()
        .find(|d| d.country.eq_ignore_ascii_case(country))
        .ok_or_else(|| {
            Error::not_found(crate::i18n::t_args(
                "nucleo-impostos-dados-de",
                &crate::i18n::args(&[("pais", country.to_string().into())]),
            ))
        })
}

fn parse(json: &str) -> Result<TaxDataset> {
    let d: TaxDataset = serde_json::from_str(json)
        .map_err(|e| Error::invalid(format!("dados de impostos inválidos: {e}")))?;
    validate(&d)?;
    Ok(d)
}

/// Toda alíquota e regra aponta para fontes que existem, e as % ficam entre 0 e 100.
fn validate(d: &TaxDataset) -> Result<()> {
    let known = |id: &String| d.sources.iter().any(|s| &s.id == id);
    let bad = |what: String| Err(Error::invalid(format!("dados de {}: {what}", d.country)));
    for g in &d.consumption {
        let rated = [&g.low, &g.central, &g.high].iter().any(|r| r.is_some());
        if rated && (g.source_ids.is_empty() || !g.source_ids.iter().all(known)) {
            return bad(format!("grupo {} sem fonte conhecida", g.key));
        }
        for r in [&g.low, &g.central, &g.high].into_iter().flatten() {
            if let RateSpec::Percent(p) = r
                && !(0.0..100.0).contains(p)
            {
                return bad(format!("alíquota fora de 0–100% em {}", g.key));
            }
        }
    }
    for r in &d.regions {
        if !r.source_ids.iter().all(known) {
            return bad(format!("estado {} com fonte desconhecida", r.code));
        }
    }
    if let Some(w) = &d.work_days
        && (w.source_ids.is_empty() || !w.source_ids.iter().all(known))
    {
        return bad("dias de trabalho sem fonte conhecida".into());
    }
    if let Some(sp) = &d.spending {
        let total: f64 = sp.functions.iter().map(|f| f.share).sum();
        if sp.source_ids.is_empty() || !sp.source_ids.iter().all(known) {
            return bad("despesa por função sem fonte conhecida".into());
        }
        if !(99.5..=100.5).contains(&total) {
            return bad(format!("despesa por função soma {total:.2}%"));
        }
    }
    for p in &d.payroll {
        if p.source_ids.is_empty() || !p.source_ids.iter().all(known) {
            return bad(format!("regras de {} sem fonte conhecida", p.from));
        }
    }
    Ok(())
}
