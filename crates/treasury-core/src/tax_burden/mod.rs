//! Estimativa de impostos pagos (docs/taxes.md): quanto da vida financeira do ano foi
//! imposto, a partir dos dados do app, em três cenários (conservador, provável, elevado).
//!
//! - **Consumo**: o gasto de cada categoria vezes a fração do preço que é imposto no grupo
//!   dela (`mapping`), com as alíquotas do país (`dataset`, cada uma com fonte e data) e, no
//!   Brasil, o ajuste do ICMS do estado nas mercadorias da alíquota modal.
//! - **Renda e contribuições do trabalho**: pelos informes de rendimentos, quando o usuário
//!   os lança; senão, estimados a partir do salário líquido recebido (`rules`).
//! - **Impostos pagos direto** (IR, IOF, IPTU, IPVA…): o que foi lançado nas categorias deles.
//! - **Patrimônio**: IPVA estimado pelo valor de cada veículo quando nenhum foi lançado.
//! - **IR retido em proventos** (JCP, dividendos do exterior), dos registros da carteira.
//!
//! É estimativa, não valor oficial: cada linha diz de onde veio o número.

pub mod api;
pub mod city;
pub mod dataset;
pub mod mapping;
pub mod rules;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_countries;

use std::collections::{BTreeMap, HashMap};

use chrono::Datelike;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use self::dataset::{RateSpec, TaxContext, TaxDataset, TaxRegion};
use self::mapping::{CategoryChoice, CategoryNode, GroupKind};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{now_utc, today_local};

const COUNTRY: &str = "tax_burden.country";
const REGION: &str = "tax_burden.region";
const DEPENDENTS: &str = "tax_burden.dependents";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Scenario {
    Conservative,
    Probable,
    High,
}

/// Uma alíquota (% do preço) em cada cenário.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RateTriple {
    pub conservative: f64,
    pub probable: f64,
    pub high: f64,
}

/// Um valor (centavos) em cada cenário.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AmountTriple {
    #[ts(type = "number")]
    pub conservative: i64,
    #[ts(type = "number")]
    pub probable: i64,
    #[ts(type = "number")]
    pub high: i64,
}

impl AmountTriple {
    fn same(v: i64) -> Self {
        Self {
            conservative: v,
            probable: v,
            high: v,
        }
    }
    fn add(&mut self, o: AmountTriple) {
        self.conservative += o.conservative;
        self.probable += o.probable;
        self.high += o.high;
    }
}

/// Preferências da estimativa (técnicas: ficam em `setting`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxSettings {
    /// País dos dados (`BR`, `US`, `EU`).
    pub country: String,
    /// UF, estado americano ou país da União Europeia.
    pub region: Option<String>,
    /// Dependentes para a estimativa do IR a partir do líquido.
    pub dependents: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxSettingsPatch {
    #[ts(optional)]
    pub country: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub region: Option<Option<String>>,
    #[ts(optional)]
    pub dependents: Option<u32>,
}

/// Dados e estado que valem pelo país da família (docs/countries.md), quando o usuário não
/// escolheu outro.
fn family_default(conn: &Connection) -> (String, Option<String>) {
    let c = crate::country::family_country(conn);
    match crate::country::info(conn, &c) {
        Ok(i) => (i.tax_dataset.unwrap_or_else(|| "BR".into()), i.tax_region),
        Err(_) => ("BR".into(), None),
    }
}

pub fn settings(conn: &Connection) -> Result<TaxSettings> {
    let (country, region) = family_default(conn);
    let chosen: Option<String> = crate::settings::get(conn, COUNTRY)?;
    let region = match &chosen {
        Some(c) if *c != country => crate::settings::get(conn, REGION)?,
        _ => crate::settings::get(conn, REGION)?.or(region),
    };
    // Os dados do Brasil só valem num cofre do Brasil: escolhidos antes de a família mudar de
    // país, deixam de valer.
    let chosen = chosen.filter(|c| c != "BR" || crate::country::is_brazil(conn));
    Ok(TaxSettings {
        country: chosen.unwrap_or(country),
        region,
        dependents: crate::settings::get_or(conn, DEPENDENTS, 0u32)?,
    })
}

/// O país da família mudou: a estimativa passa a usar os dados dele (o estado escolhido fica
/// se o país dos dados não mudou). Sem dados do país, nada muda.
pub fn follow_country(
    conn: &Connection,
    dataset: Option<&str>,
    region: Option<&str>,
) -> Result<()> {
    let Some(ds) = dataset else {
        return Ok(());
    };
    let before: Option<String> = crate::settings::get(conn, COUNTRY)?;
    crate::settings::set(conn, COUNTRY, ds)?;
    match region {
        Some(r) => crate::settings::set(conn, REGION, r)?,
        None if before.as_deref() != Some(ds) => crate::settings::delete(conn, REGION)?,
        None => {}
    }
    Ok(())
}

pub fn set_settings(conn: &Connection, p: &TaxSettingsPatch) -> Result<TaxSettings> {
    let mut s = settings(conn)?;
    if let Some(c) = &p.country {
        s.country = dataset::get(c)?.country;
        if s.country == "BR" && !crate::country::is_brazil(conn) {
            return Err(Error::invalid(t("nucleo-impostos-erro-brasil")));
        }
        // Outro país: o estado anterior deixa de valer.
        if p.region.is_none() {
            s.region = None;
        }
    }
    if let Some(r) = &p.region {
        s.region = r.clone().filter(|r| !r.trim().is_empty());
    }
    if let Some(d) = p.dependents {
        if d > 20 {
            return Err(Error::invalid(t("nucleo-impostos-erro-dependentes")));
        }
        s.dependents = d;
    }
    if let Some(r) = &s.region {
        let ds = dataset::get(&s.country)?;
        if !ds.regions.iter().any(|x| x.code.eq_ignore_ascii_case(r)) {
            return Err(Error::invalid(t_args(
                "nucleo-impostos-erro-estado",
                &args(&[
                    ("estado", r.clone().into()),
                    ("dados", ds.name.clone().into()),
                ]),
            )));
        }
    }
    crate::settings::set(conn, COUNTRY, &s.country)?;
    match &s.region {
        Some(r) => crate::settings::set(conn, REGION, r)?,
        None => {
            conn.execute("DELETE FROM setting WHERE key = ?1", [REGION])?;
        }
    }
    crate::settings::set(conn, DEPENDENTS, &s.dependents)?;
    Ok(s)
}

// ---------------------------------------------------------------------------
// Escolhas por categoria

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxCategoryView {
    pub category_id: Id,
    pub path: String,
    pub kind: crate::categories::CategoryKind,
    /// Grupo que vale (escolha, padrão ou herdado do pai); `null` = sem grupo.
    pub group_key: Option<String>,
    /// `user`, `default` ou `parent`.
    pub origin: Option<String>,
    /// Grupo que valeria sem a escolha do usuário (padrão pelo nome ou herdado do pai).
    pub default_group_key: Option<String>,
    /// Alíquota própria (% do preço), quando o usuário fixou uma.
    pub rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxCategoryPatch {
    pub category_id: Id,
    /// Grupo (`null` volta ao padrão).
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub group_key: Option<Option<String>>,
    /// Alíquota própria em % do preço (`null` volta à do país).
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub rate: Option<Option<f64>>,
}

fn category_nodes(conn: &Connection) -> Result<HashMap<Id, (CategoryNode, String)>> {
    let mut stmt = conn.prepare("SELECT id, parent_id, name, kind FROM category")?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, Id>(0)?,
                r.get::<_, Option<Id>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    // O grupo padrão sai do nome em pt-BR: vale com as categorias em qualquer idioma.
    let br = crate::categories::br_names(conn)?;
    Ok(rows
        .into_iter()
        .map(|(id, parent_id, name, kind)| {
            let name = br.get(&id).cloned().unwrap_or(name);
            (id, (CategoryNode { name, parent_id }, kind))
        })
        .collect())
}

fn choices(conn: &Connection) -> Result<HashMap<Id, CategoryChoice>> {
    let mut stmt = conn.prepare("SELECT category_id, group_key, rate FROM tax_category")?;
    Ok(stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, Id>(0)?,
                CategoryChoice {
                    group_key: r.get(1)?,
                    rate: r.get(2)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn categories(conn: &Connection) -> Result<Vec<TaxCategoryView>> {
    let nodes = category_nodes(conn)?;
    let plain: HashMap<Id, CategoryNode> = nodes
        .iter()
        .map(|(id, (n, _))| {
            (
                id.clone(),
                CategoryNode {
                    name: n.name.clone(),
                    parent_id: n.parent_id.clone(),
                },
            )
        })
        .collect();
    let ch = choices(conn)?;
    let paths = crate::categories::path_map(conn)?;
    let mut out: Vec<TaxCategoryView> = nodes
        .iter()
        .filter(|(_, (_, kind))| kind == "expense")
        .map(|(id, _)| {
            let resolved = mapping::resolve(id, &plain, &ch);
            let default = mapping::resolve(id, &plain, &HashMap::new()).map(|(g, _)| g);
            TaxCategoryView {
                default_group_key: default,
                category_id: id.clone(),
                path: paths.get(id).cloned().unwrap_or_default(),
                kind: crate::categories::CategoryKind::Expense,
                origin: resolved.as_ref().map(|(_, o)| o.to_string()),
                group_key: resolved.map(|(g, _)| g),
                rate: ch.get(id).and_then(|c| c.rate),
            }
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Grupos que existem em algum lugar (consumo do país, diretos ou fora do consumo).
fn known_group(country: &TaxDataset, key: &str) -> bool {
    mapping::kind_of(key) != GroupKind::Consumption
        || country.consumption.iter().any(|g| g.key == key)
}

pub fn set_category(tx: &Transaction<'_>, p: &TaxCategoryPatch) -> Result<()> {
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM category WHERE id = ?1 AND kind = 'expense')",
        [&p.category_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(Error::not_found(t_args(
            "nucleo-impostos-categoria-despesa",
            &args(&[("id", p.category_id.clone().into())]),
        )));
    }
    let cur: Option<(Option<String>, Option<f64>)> = tx
        .query_row(
            "SELECT group_key, rate FROM tax_category WHERE category_id = ?1",
            [&p.category_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (mut group, mut rate) = cur.clone().unwrap_or((None, None));
    if let Some(g) = &p.group_key {
        group = g.clone().filter(|g| !g.trim().is_empty());
        if let Some(g) = &group {
            let ds = dataset::get(&settings(tx)?.country)?;
            if !known_group(&ds, g) {
                return Err(Error::invalid(t_args(
                    "nucleo-impostos-erro-grupo",
                    &args(&[("grupo", g.clone().into())]),
                )));
            }
        }
    }
    if let Some(r) = p.rate {
        if r.is_some_and(|r| !(0.0..100.0).contains(&r)) {
            return Err(Error::invalid(t("nucleo-impostos-erro-aliquota")));
        }
        rate = r;
    }
    let now = now_utc();
    if group.is_none() && rate.is_none() {
        tx.execute(
            "DELETE FROM tax_category WHERE category_id = ?1",
            [&p.category_id],
        )?;
    } else if cur.is_some() {
        tx.execute(
            "UPDATE tax_category SET group_key = ?2, rate = ?3, updated_at = ?4 WHERE category_id = ?1",
            params![p.category_id, group, rate, now],
        )?;
    } else {
        tx.execute(
            "INSERT INTO tax_category (category_id, group_key, rate, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![p.category_id, group, rate, now],
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Informes de rendimentos

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeReport {
    pub id: Id,
    pub year: i32,
    pub member_id: Option<Id>,
    pub payer: String,
    /// Rendimentos tributáveis no ano (sem o 13º).
    #[ts(type = "number")]
    pub gross: i64,
    /// Contribuição previdenciária oficial (INSS) retida.
    #[ts(type = "number")]
    pub social_security: i64,
    /// IR retido na fonte (sem o do 13º).
    #[ts(type = "number")]
    pub income_tax: i64,
    #[ts(type = "number")]
    pub thirteenth_gross: i64,
    /// IR retido sobre o 13º (tributação exclusiva).
    #[ts(type = "number")]
    pub thirteenth_tax: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IncomeReportInput {
    /// Com `id`, altera o informe.
    #[ts(optional)]
    pub id: Option<Id>,
    pub year: i32,
    #[ts(optional)]
    pub member_id: Option<Id>,
    pub payer: String,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(optional, type = "number")]
    pub social_security: Option<i64>,
    #[ts(optional, type = "number")]
    pub income_tax: Option<i64>,
    #[ts(optional, type = "number")]
    pub thirteenth_gross: Option<i64>,
    #[ts(optional, type = "number")]
    pub thirteenth_tax: Option<i64>,
    #[ts(optional)]
    pub notes: Option<String>,
}

pub fn income_reports(conn: &Connection, year: Option<i32>) -> Result<Vec<IncomeReport>> {
    let mut stmt = conn.prepare(
        "SELECT id, year, member_id, payer, gross, social_security, income_tax, thirteenth_gross,
                thirteenth_tax, notes
           FROM tax_income_report WHERE (?1 IS NULL OR year = ?1) ORDER BY year DESC, payer",
    )?;
    Ok(stmt
        .query_map([year], |r| {
            Ok(IncomeReport {
                id: r.get(0)?,
                year: r.get(1)?,
                member_id: r.get(2)?,
                payer: r.get(3)?,
                gross: r.get(4)?,
                social_security: r.get(5)?,
                income_tax: r.get(6)?,
                thirteenth_gross: r.get(7)?,
                thirteenth_tax: r.get(8)?,
                notes: r.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn save_income_report(tx: &Transaction<'_>, i: &IncomeReportInput) -> Result<Id> {
    if i.payer.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-impostos-erro-fonte-pagadora")));
    }
    if !(2000..=2100).contains(&i.year) {
        return Err(Error::invalid(t("nucleo-impostos-erro-ano-informe")));
    }
    let values = [
        i.gross,
        i.social_security.unwrap_or(0),
        i.income_tax.unwrap_or(0),
        i.thirteenth_gross.unwrap_or(0),
        i.thirteenth_tax.unwrap_or(0),
    ];
    if values.iter().any(|v| *v < 0) {
        return Err(Error::invalid(t("nucleo-impostos-erro-informe-negativo")));
    }
    if let Some(m) = &i.member_id {
        let ok: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM member WHERE id = ?1)",
            [m],
            |r| r.get(0),
        )?;
        if !ok {
            return Err(Error::not_found(t_args(
                "nucleo-geral-membro",
                &args(&[("id", m.clone().into())]),
            )));
        }
    }
    let notes = i
        .notes
        .clone()
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let now = now_utc();
    let id = match &i.id {
        Some(id) => {
            let n = tx.execute(
                "UPDATE tax_income_report SET year = ?2, member_id = ?3, payer = ?4, gross = ?5,
                        social_security = ?6, income_tax = ?7, thirteenth_gross = ?8,
                        thirteenth_tax = ?9, notes = ?10, updated_at = ?11 WHERE id = ?1",
                params![
                    id,
                    i.year,
                    i.member_id,
                    i.payer.trim(),
                    values[0],
                    values[1],
                    values[2],
                    values[3],
                    values[4],
                    notes,
                    now
                ],
            )?;
            if n == 0 {
                return Err(Error::not_found(report_ref(id)));
            }
            id.clone()
        }
        None => {
            let id = new_id();
            tx.execute(
                "INSERT INTO tax_income_report (id, year, member_id, payer, gross, social_security,
                        income_tax, thirteenth_gross, thirteenth_tax, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
                params![
                    id,
                    i.year,
                    i.member_id,
                    i.payer.trim(),
                    values[0],
                    values[1],
                    values[2],
                    values[3],
                    values[4],
                    notes,
                    now
                ],
            )?;
            id
        }
    };
    Ok(id)
}

/// "informe <id>", para o erro de não encontrado.
fn report_ref(id: &str) -> String {
    t_args(
        "nucleo-impostos-informe",
        &args(&[("id", id.to_string().into())]),
    )
}

pub fn delete_income_report(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM tax_income_report WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(report_ref(id)));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Estimativa

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EstimateArgs {
    /// Ano (padrão: o atual, até hoje).
    #[ts(optional)]
    pub year: Option<i32>,
}

/// Parte da estimativa por tipo de imposto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaxPartKind {
    /// Embutidos no preço do que foi consumido.
    Consumption,
    /// IR sobre o trabalho (retido) e pago direto.
    IncomeTax,
    /// Contribuição previdenciária do trabalhador.
    SocialSecurity,
    /// IPTU, IPVA.
    Property,
    /// IR retido nos proventos da carteira.
    Investments,
    /// IOF e outros tributos lançados.
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxPart {
    pub kind: TaxPartKind,
    pub label: String,
    pub amount: AmountTriple,
    /// Tem estimativa (e não só valores lançados ou informados).
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConsumptionLine {
    pub category_id: Option<Id>,
    pub path: String,
    #[ts(type = "number")]
    pub spent: i64,
    pub group_key: String,
    pub group_label: String,
    /// `user`, `default` ou `parent`.
    pub origin: String,
    /// % do preço em cada cenário (com o ajuste do estado e a alíquota própria).
    pub rates: RateTriple,
    /// Alíquota própria do usuário (vale nos três cenários).
    pub own_rate: Option<f64>,
    /// Ajuste do ICMS do estado (pontos percentuais) já somado em `rates`.
    pub region_adjustment: f64,
    pub taxes: AmountTriple,
    pub source_ids: Vec<String>,
}

/// Gasto que ficou fora da estimativa de consumo, e por quê.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnestimatedLine {
    pub category_id: Option<Id>,
    pub path: String,
    #[ts(type = "number")]
    pub amount: i64,
    /// `uncategorized`, `unmapped`, `not_consumption` ou `no_rate`.
    pub reason: String,
}

/// Imposto lançado como despesa (IR, IOF, IPTU, IPVA…).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DirectLine {
    pub category_id: Option<Id>,
    pub path: String,
    pub group_key: String,
    pub label: String,
    #[ts(type = "number")]
    pub amount: i64,
}

/// Renda do trabalho e o que foi retido, por titular.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayrollLine {
    pub member_id: Option<Id>,
    pub member_name: Option<String>,
    /// `reported` (informe de rendimentos), `payslips` (holerites importados) ou `estimated`
    /// (a partir do líquido).
    pub origin: String,
    /// Líquido recebido no app (salário, férias, 13º).
    #[ts(type = "number")]
    pub net: i64,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub social_security: i64,
    #[ts(type = "number")]
    pub income_tax: i64,
    #[ts(type = "number")]
    pub months: i64,
    /// Leis das regras usadas na estimativa.
    pub legal_basis: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyLine {
    pub asset_id: Id,
    pub name: String,
    /// `ipva` (estimado pelo valor do veículo) ou `iptu` (pelos carnês registrados em Bens).
    pub tax: String,
    #[ts(type = "number")]
    pub value: i64,
    pub rate: f64,
    #[ts(type = "number")]
    pub amount: i64,
    pub source_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DatasetInfo {
    pub country: String,
    pub name: String,
    pub version: String,
    pub updated: String,
    pub coverage: String,
    /// Mais de `dataset::STALE_MONTHS` meses sem revisão.
    pub stale: bool,
}

/// Uma função de governo e a parte do imposto estimado que corresponde a ela.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DestinationLine {
    pub key: String,
    pub label: String,
    /// % da despesa total do governo geral.
    pub share: f64,
    pub amounts: AmountTriple,
}

/// Para onde vai o imposto: o total estimado repartido pela despesa do governo geral por
/// função (Brasil: Tesouro Nacional, COFOG).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxDestination {
    pub year: i32,
    pub label: String,
    pub source_ids: Vec<String>,
    pub lines: Vec<DestinationLine>,
}

/// Reparte `totals` pelas funções; a sobra do arredondamento vai para a maior função, e a soma
/// de cada cenário fecha com o total.
fn destination(sp: &dataset::SpendingByFunction, totals: AmountTriple) -> TaxDestination {
    let sum: f64 = sp.functions.iter().map(|f| f.share).sum();
    let part = |total: i64, share: f64| (total as f64 * share / sum.max(1e-9)).round() as i64;
    let mut lines: Vec<DestinationLine> = sp
        .functions
        .iter()
        .map(|f| DestinationLine {
            key: f.key.clone(),
            label: f.label.clone(),
            share: f.share,
            amounts: AmountTriple {
                conservative: part(totals.conservative, f.share),
                probable: part(totals.probable, f.share),
                high: part(totals.high, f.share),
            },
        })
        .collect();
    if let Some(top) = (0..lines.len()).max_by(|a, b| lines[*a].share.total_cmp(&lines[*b].share)) {
        let given = |f: fn(&AmountTriple) -> i64| lines.iter().map(|l| f(&l.amounts)).sum::<i64>();
        let (c, p, h) = (
            totals.conservative - given(|a| a.conservative),
            totals.probable - given(|a| a.probable),
            totals.high - given(|a| a.high),
        );
        lines[top].amounts.conservative += c;
        lines[top].amounts.probable += p;
        lines[top].amounts.high += h;
    }
    TaxDestination {
        year: sp.year,
        label: sp.label.clone(),
        source_ids: sp.source_ids.clone(),
        lines,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaxEstimate {
    pub year: i32,
    pub from: String,
    pub to: String,
    pub currency: String,
    pub settings: TaxSettings,
    pub region_name: Option<String>,
    pub dataset: DatasetInfo,
    pub totals: AmountTriple,
    pub parts: Vec<TaxPart>,
    pub consumption: Vec<ConsumptionLine>,
    /// Gasto de consumo que entrou na estimativa.
    #[ts(type = "number")]
    pub consumption_spent: i64,
    pub unestimated: Vec<UnestimatedLine>,
    pub direct: Vec<DirectLine>,
    pub payroll: Vec<PayrollLine>,
    #[ts(type = "number")]
    pub investments_withheld: i64,
    pub property: Vec<PropertyLine>,
    /// Renda bruta estimada: o que entrou no ano mais o retido na fonte.
    #[ts(type = "number")]
    pub income_base: i64,
    /// Imposto total ÷ renda bruta (0–1), por cenário.
    pub share_of_income: RateTriple,
    /// Imposto de consumo ÷ gasto de consumo (0–1), por cenário.
    pub share_of_consumption: RateTriple,
    /// Dias do ano que a renda da família levou para pagar os impostos (imposto ÷ renda bruta
    /// × dias do ano), por cenário; sem renda conhecida, `null`.
    pub work_days: Option<RateTriple>,
    /// A referência nacional (no Brasil, a carga tributária bruta do Tesouro em dias), com fonte.
    pub work_days_reference: Option<dataset::WorkDaysReference>,
    /// Para onde vai o imposto (só em cofre do Brasil, com imposto estimado).
    pub destination: Option<TaxDestination>,
    pub context: Option<TaxContext>,
    /// Avisos (categorias sem grupo, salário sem titular, IPTU não lançado…).
    pub notes: Vec<String>,
}

fn spec_value(spec: &RateSpec, region: Option<&TaxRegion>) -> Option<f64> {
    // Imposto cobrado por fora do preço (IVA, imposto sobre vendas): a fração do preço
    // pago que é imposto é r / (100 + r).
    let share = |r: f64| r / (100.0 + r) * 100.0;
    match spec {
        RateSpec::Percent(p) => Some(*p),
        RateSpec::Basis(b) => match b.as_str() {
            "exempt" => Some(0.0),
            "vat_standard" => region?.vat_standard.map(share),
            "vat_reduced" => region?.vat_reduced.or(region?.vat_standard).map(share),
            "vat_super_reduced" => region?
                .vat_super_reduced
                .or(region?.vat_reduced)
                .or(region?.vat_standard)
                .map(share),
            "sales_tax" => region?.sales_tax.map(share),
            _ => None,
        },
    }
}

fn tax_of(spent: i64, rate: f64) -> i64 {
    (spent as f64 * rate / 100.0).round() as i64
}

fn ratio(a: i64, b: i64) -> f64 {
    if b > 0 { a as f64 / b as f64 } else { 0.0 }
}

/// Meses entre a data de revisão dos dados e hoje.
fn months_since(date: &str) -> i64 {
    let Ok(d) = crate::time::parse_date(&format!("{}-01", &date[..7.min(date.len())])) else {
        return 0;
    };
    let t = today_local();
    (t.year() as i64 - d.year() as i64) * 12 + t.month() as i64 - d.month() as i64
}

/// Nome de um grupo de imposto direto (IR, IOF, IPTU, IPVA, outros) no idioma da instalação.
fn direct_label(group: &str) -> String {
    match group {
        "tax_income" => t("impostos-grupo-ir-pago"),
        "tax_iof" => t("impostos-grupo-iof-pago"),
        "tax_property_iptu" => t("impostos-grupo-iptu"),
        "tax_property_ipva" => t("impostos-grupo-ipva"),
        "tax_other" => t("impostos-grupo-outros-pagos"),
        _ => mapping::DIRECT
            .iter()
            .find(|(k, _)| *k == group)
            .map(|(_, l)| l.to_string())
            .unwrap_or_default(),
    }
}

/// Grupos de renda do trabalho pelo nome da categoria de receita.
fn wage_kind(name: &str) -> Option<&'static str> {
    // `fold` mantém o "º" (é letra): "13º salário" vira "13º salario".
    let f = crate::import::text::fold(name).replace('º', "o");
    match f.as_str() {
        "salario" | "ferias" | "horas extras" | "comissoes" => Some("monthly"),
        "13o salario" | "13 salario" | "decimo terceiro" | "decimo terceiro salario" => {
            Some("thirteenth")
        }
        // Portugal: os subsídios têm retenção própria, separada do salário do mês.
        "subsidio de natal" | "subsidio de ferias" => Some("thirteenth"),
        "horas extraordinarias" | "premios e bonus" => Some("monthly"),
        // Estados Unidos.
        "bonus" => Some("monthly"),
        _ => None,
    }
}

pub fn estimate(conn: &Connection, a: &EstimateArgs) -> Result<TaxEstimate> {
    let today = today_local();
    let year = a.year.unwrap_or(today.year());
    let from = format!("{year}-01-01");
    let to = if year == today.year() {
        crate::time::fmt_date(today)
    } else {
        format!("{year}-12-31")
    };
    if year > today.year() {
        return Err(Error::invalid(t("nucleo-impostos-erro-ano-futuro")));
    }
    let s = settings(conn)?;
    let ds = dataset::get(&s.country)?;
    let region = s
        .region
        .as_deref()
        .and_then(|r| ds.regions.iter().find(|x| x.code.eq_ignore_ascii_case(r)));
    let mut notes: Vec<String> = vec![];
    if region.is_none() && !ds.regions.is_empty() {
        notes.push(t(match ds.country.as_str() {
            "BR" => "nucleo-impostos-nota-escolha-uf",
            "US" => "nucleo-impostos-nota-escolha-estado",
            _ => "nucleo-impostos-nota-escolha-pais",
        }));
    }
    let currency = crate::household::base_currency(conn)?;

    // Gasto por categoria (competência, na moeda da casa).
    let spending = crate::reports::spending(
        conn,
        &crate::reports::SpendingArgs {
            from: from.clone(),
            to: to.clone(),
            basis: Some(crate::budget::Basis::Accrual),
            kind: Some(crate::categories::CategoryKind::Expense),
            group_by: Some(crate::reports::GroupBy::Category),
            member_ids: None,
            account_ids: None,
            category_ids: None,
        },
    )?;
    let nodes = category_nodes(conn)?;
    let plain: HashMap<Id, CategoryNode> = nodes
        .iter()
        .map(|(id, (n, _))| {
            (
                id.clone(),
                CategoryNode {
                    name: n.name.clone(),
                    parent_id: n.parent_id.clone(),
                },
            )
        })
        .collect();
    let ch = choices(conn)?;
    let icms_adjust = match (ds.reference_icms, region.and_then(|r| r.icms_modal)) {
        (Some(reference), Some(state)) => state - reference,
        _ => 0.0,
    };

    // Cupons fiscais ligados a lançamentos (docs/receipts.md): nessas compras vale o imposto
    // da nota (impresso, ou estimado item a item), no lugar da média da categoria.
    // Onde o imposto é somado no fim do cupom (EUA, Canadá), só as compras com o imposto
    // impresso: as outras seguem pela alíquota do estado.
    let linked = if ds.country == "BR" {
        crate::receipts::linked_in(conn, &from, &to, false)?
    } else if crate::food::profile::of(&ds.country).item_tax
        == crate::food::profile::ItemTax::Printed
    {
        crate::receipts::linked_in(conn, &from, &to, true)?
    } else {
        vec![]
    };
    let mut receipt_spent: HashMap<Option<Id>, i64> = HashMap::new();
    let (mut receipts_paid, mut receipts_tax) = (0i64, 0i64);
    for (cat, paid, tax) in &linked {
        *receipt_spent.entry(cat.clone()).or_default() += paid;
        receipts_paid += paid;
        receipts_tax += tax;
    }

    let mut consumption: Vec<ConsumptionLine> = vec![];
    let mut unestimated: Vec<UnestimatedLine> = vec![];
    let mut direct: Vec<DirectLine> = vec![];
    for g in &spending.groups {
        let mut g = g.clone();
        g.amount = (g.amount - receipt_spent.get(&g.key).copied().unwrap_or(0)).max(0);
        if g.amount == 0 {
            continue;
        }
        let path = g.path.clone().unwrap_or_else(|| g.label.clone());
        let Some(cat) = &g.key else {
            unestimated.push(UnestimatedLine {
                category_id: None,
                path: t("nucleo-impostos-sem-categoria"),
                amount: g.amount,
                reason: "uncategorized".into(),
            });
            continue;
        };
        let Some((group, origin)) = mapping::resolve(cat, &plain, &ch) else {
            unestimated.push(UnestimatedLine {
                category_id: Some(cat.clone()),
                path,
                amount: g.amount,
                reason: "unmapped".into(),
            });
            continue;
        };
        match mapping::kind_of(&group) {
            GroupKind::DirectTax => direct.push(DirectLine {
                category_id: Some(cat.clone()),
                path,
                label: direct_label(&group),
                group_key: group,
                amount: g.amount,
            }),
            GroupKind::NotConsumption => unestimated.push(UnestimatedLine {
                category_id: Some(cat.clone()),
                path,
                amount: g.amount,
                reason: "not_consumption".into(),
            }),
            GroupKind::Consumption => {
                let own = ch.get(cat).and_then(|c| c.rate);
                let def = ds.consumption.iter().find(|x| x.key == group);
                let (rates, adjust, sources, label) = match (own, def) {
                    (Some(r), d) => (
                        RateTriple {
                            conservative: r,
                            probable: r,
                            high: r,
                        },
                        0.0,
                        vec![],
                        d.map(|d| d.label.clone()).unwrap_or_else(|| group.clone()),
                    ),
                    (None, Some(d)) => {
                        let adj = if d.icms_modal { icms_adjust } else { 0.0 };
                        let v = |sp: &Option<RateSpec>| {
                            spec_value(sp.as_ref()?, region).map(|x| (x + adj).max(0.0))
                        };
                        match (v(&d.low), v(&d.central), v(&d.high)) {
                            (Some(c), Some(p), Some(h)) => (
                                RateTriple {
                                    conservative: c,
                                    probable: p,
                                    high: h,
                                },
                                adj,
                                d.source_ids.clone(),
                                d.label.clone(),
                            ),
                            _ => {
                                unestimated.push(UnestimatedLine {
                                    category_id: Some(cat.clone()),
                                    path,
                                    amount: g.amount,
                                    reason: "no_rate".into(),
                                });
                                continue;
                            }
                        }
                    }
                    (None, None) => {
                        unestimated.push(UnestimatedLine {
                            category_id: Some(cat.clone()),
                            path,
                            amount: g.amount,
                            reason: "no_rate".into(),
                        });
                        continue;
                    }
                };
                consumption.push(ConsumptionLine {
                    category_id: Some(cat.clone()),
                    path,
                    spent: g.amount,
                    group_key: group,
                    group_label: label,
                    origin: origin.to_string(),
                    taxes: AmountTriple {
                        conservative: tax_of(g.amount, rates.conservative),
                        probable: tax_of(g.amount, rates.probable),
                        high: tax_of(g.amount, rates.high),
                    },
                    rates,
                    own_rate: own,
                    region_adjustment: adjust,
                    source_ids: sources,
                })
            }
        }
    }
    if receipts_paid > 0 {
        let rate = receipts_tax as f64 / receipts_paid as f64 * 100.0;
        consumption.push(ConsumptionLine {
            category_id: None,
            path: t("nucleo-impostos-cupons"),
            spent: receipts_paid,
            group_key: "receipts".into(),
            group_label: t("nucleo-impostos-grupo-cupons"),
            origin: "receipt".into(),
            rates: RateTriple {
                conservative: rate,
                probable: rate,
                high: rate,
            },
            own_rate: None,
            region_adjustment: 0.0,
            taxes: AmountTriple::same(receipts_tax),
            source_ids: vec!["lei_12741".into()],
        });
        let all: i64 = consumption.iter().map(|c| c.spent).sum();
        notes.push(t_args(
            "nucleo-impostos-nota-cupons",
            &args(&[(
                "parte",
                (receipts_paid as f64 / all.max(1) as f64 * 100.0)
                    .round()
                    .to_string()
                    .into(),
            )]),
        ));
    }
    consumption.sort_by_key(|c| std::cmp::Reverse(c.taxes.probable));
    let unmapped = unestimated
        .iter()
        .filter(|u| u.reason == "unmapped")
        .count();
    if unmapped > 0 {
        notes.push(t_args(
            "nucleo-impostos-nota-sem-grupo",
            &args(&[("n", (unmapped as i64).into())]),
        ));
    }

    // Renda do trabalho.
    let payroll = payroll_lines(conn, &ds, &s, year, &from, &to, &mut notes)?;

    // IR retido nos proventos da carteira (na moeda da casa).
    let investments_withheld: i64 = conn.query_row(
        "SELECT coalesce(sum(tax_withheld), 0) FROM income_event
          WHERE pay_date BETWEEN ?1 AND ?2 AND currency = ?3",
        params![from, to, currency],
        |r| r.get(0),
    )?;

    // Patrimônio: IPVA pelo valor dos veículos, se nenhum IPVA foi lançado no ano.
    let ipva_recorded = direct
        .iter()
        .any(|d| d.group_key == "tax_property_ipva" && d.amount > 0);
    let mut property = vec![];
    // O IPVA é do Brasil: fora dele, nem a estimativa nem o aviso de estado sem tabela.
    if !ipva_recorded && crate::country::is_brazil(conn) {
        property = ipva_lines(conn, region, &to, &mut notes)?;
    }
    let iptu_recorded = direct.iter().any(|d| d.group_key == "tax_property_iptu");
    // IPTU pelos carnês registrados em Bens, se nenhum IPTU foi lançado no período.
    let mut iptu_from_bills = false;
    if ds.country == "BR" && !iptu_recorded {
        for b in crate::physical::iptu::period_taxes(conn, &from, &to)? {
            iptu_from_bills = true;
            property.push(PropertyLine {
                asset_id: b.asset_id,
                name: b.asset_name,
                tax: "iptu".into(),
                value: b.venal,
                rate: if b.venal > 0 {
                    b.amount as f64 / b.venal as f64 * 100.0
                } else {
                    0.0
                },
                amount: b.amount,
                source_ids: vec![],
            });
        }
        if iptu_from_bills {
            notes.push(crate::i18n::t("iptu-impostos-nota"));
        }
    }
    let has_real_estate: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM physical_asset WHERE kind = 'real_estate' AND archived = 0
           AND (sale_date IS NULL OR sale_date >= ?1))",
        [&from],
        |r| r.get(0),
    )?;
    if ds.country == "BR" && has_real_estate && !iptu_recorded && !iptu_from_bills {
        notes.push(t("nucleo-impostos-nota-sem-iptu"));
    }

    // Partes e totais.
    let mut parts: BTreeMap<TaxPartKind, (AmountTriple, bool)> = BTreeMap::new();
    let mut add = |k: TaxPartKind, v: AmountTriple, est: bool| {
        let e = parts.entry(k).or_insert((AmountTriple::default(), false));
        e.0.add(v);
        e.1 |= est;
    };
    for c in &consumption {
        add(TaxPartKind::Consumption, c.taxes, true);
    }
    for d in &direct {
        let k = match d.group_key.as_str() {
            "tax_income" => TaxPartKind::IncomeTax,
            "tax_property_iptu" | "tax_property_ipva" => TaxPartKind::Property,
            _ => TaxPartKind::Other,
        };
        add(k, AmountTriple::same(d.amount), false);
    }
    for p in &payroll {
        let est = p.origin == "estimated";
        add(
            TaxPartKind::IncomeTax,
            AmountTriple::same(p.income_tax),
            est,
        );
        add(
            TaxPartKind::SocialSecurity,
            AmountTriple::same(p.social_security),
            est,
        );
    }
    if investments_withheld > 0 {
        add(
            TaxPartKind::Investments,
            AmountTriple::same(investments_withheld),
            false,
        );
    }
    for p in &property {
        add(TaxPartKind::Property, AmountTriple::same(p.amount), true);
    }
    let label = |k: TaxPartKind| match k {
        TaxPartKind::Consumption => "nucleo-impostos-parte-consumo",
        TaxPartKind::IncomeTax => "nucleo-impostos-parte-renda",
        TaxPartKind::SocialSecurity => "nucleo-impostos-parte-previdencia",
        TaxPartKind::Property => "nucleo-impostos-parte-patrimonio",
        TaxPartKind::Investments => "nucleo-impostos-parte-proventos",
        TaxPartKind::Other => "nucleo-impostos-parte-outros",
    };
    let parts: Vec<TaxPart> = parts
        .into_iter()
        .map(|(kind, (amount, estimated))| TaxPart {
            kind,
            label: t(label(kind)),
            amount,
            estimated,
        })
        .collect();
    let mut totals = AmountTriple::default();
    for p in &parts {
        totals.add(p.amount);
    }
    let consumption_spent: i64 = consumption.iter().map(|c| c.spent).sum();
    let consumption_tax = parts
        .iter()
        .find(|p| p.kind == TaxPartKind::Consumption)
        .map(|p| p.amount)
        .unwrap_or_default();
    // Renda bruta: o que entrou no ano (receitas) mais o retido na fonte.
    let received: i64 = conn.query_row(
        "SELECT coalesce(sum(amount), 0) FROM txn
          WHERE kind = 'income' AND status <> 'planned' AND date BETWEEN ?1 AND ?2 AND currency = ?3",
        params![from, to, currency],
        |r| r.get(0),
    )?;
    let withheld: i64 = payroll.iter().map(|p| p.gross - p.net).sum::<i64>() + investments_withheld;
    let income_base = received + withheld.max(0);
    let share = |v: AmountTriple, base: i64| RateTriple {
        conservative: ratio(v.conservative, base),
        probable: ratio(v.probable, base),
        high: ratio(v.high, base),
    };
    let months_old = months_since(&ds.updated);
    Ok(TaxEstimate {
        year,
        from,
        to,
        currency,
        region_name: region.map(|r| r.name.clone()),
        dataset: DatasetInfo {
            country: ds.country.clone(),
            name: ds.name.clone(),
            version: ds.version.clone(),
            updated: ds.updated.clone(),
            coverage: ds.coverage.clone(),
            stale: months_old > dataset::STALE_MONTHS,
        },
        share_of_income: share(totals, income_base),
        share_of_consumption: share(consumption_tax, consumption_spent),
        work_days: (income_base > 0).then(|| {
            let days = if chrono::NaiveDate::from_ymd_opt(year, 2, 29).is_some() {
                366.0
            } else {
                365.0
            };
            let s = share(totals, income_base);
            let d = |x: f64| (x * days * 10.0).round() / 10.0;
            RateTriple {
                conservative: d(s.conservative),
                probable: d(s.probable),
                high: d(s.high),
            }
        }),
        work_days_reference: ds.work_days.clone(),
        destination: ds
            .spending
            .as_ref()
            .filter(|_| totals.probable > 0 && crate::country::is_brazil(conn))
            .map(|sp| destination(sp, totals)),
        totals,
        parts,
        consumption,
        consumption_spent,
        unestimated,
        direct,
        payroll,
        investments_withheld,
        property,
        income_base,
        context: ds.context.clone(),
        settings: s,
        notes,
    })
}

/// Regras de renda que valem no mês (`YYYY-MM`) e no estado ou país-membro: as mais recentes
/// que já começaram (as da União Europeia são de um país-membro, `region`).
pub fn rules_for<'a>(
    ds: &'a TaxDataset,
    month: &str,
    region: Option<&str>,
) -> Option<&'a rules::PayrollRules> {
    ds.payroll
        .iter()
        .filter(|r| r.from.as_str() <= month)
        .filter(|r| r.region.is_none() || r.region.as_deref() == region)
        .max_by(|a, b| a.from.cmp(&b.from))
}

/// Estado civil de quem recebe a renda, para escolher a tabela do imposto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filing {
    /// Solteiro, ou casado cujo cônjuge também tem renda (cada um pela tabela individual).
    Single,
    /// Casado com cônjuge sem renda cadastrada: declaração conjunta.
    Joint,
    /// Dependentes e nenhum cônjuge na família.
    SingleParent,
}

/// O estado civil pelo cadastro da família, como o empregado o declara ao empregador (nos
/// Estados Unidos, o passo 1(c) e a caixa do passo 2 do formulário W-4; IRS, Publicação 15-T):
/// sem cônjuge cadastrado, quem tem dependentes sustenta a casa sozinho; com cônjuge, o casal
/// (titular e cônjuge) declara em conjunto, e quando os dois têm renda cada um segue a tabela
/// individual. Outros membros com renda própria (um filho, por exemplo) são solteiros.
pub fn filing(conn: &Connection, member: Option<&str>, dependents: u32) -> Filing {
    let couple: Vec<(Id, bool)> = conn
        .prepare(
            "SELECT id, COALESCE(relationship, '') = 'conjuge' FROM member
              WHERE archived = 0
                AND (is_primary = 1 OR COALESCE(relationship, '') IN ('titular', 'conjuge'))",
        )
        .and_then(|mut st| {
            st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();
    if !couple.iter().any(|(_, spouse)| *spouse) {
        return if dependents > 0 {
            Filing::SingleParent
        } else {
            Filing::Single
        };
    }
    // Renda sem titular conhecido conta como do casal.
    if member.is_some_and(|m| !couple.iter().any(|(id, _)| id == m)) {
        return Filing::Single;
    }
    let today = crate::time::fmt_date(today_local());
    let earns = |id: &str| {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM income_source WHERE member_id = ?1 AND archived = 0
                            AND (end_date IS NULL OR end_date >= ?2))",
            params![id, today],
            |r| r.get::<_, bool>(0),
        )
        .unwrap_or(false)
    };
    let partner_earns = couple
        .iter()
        .filter(|(id, _)| Some(id.as_str()) != member)
        .any(|(id, _)| earns(id));
    // Sem saber de quem é a renda, as duas pessoas do casal precisam ter renda para valer a
    // tabela individual.
    let both = match member {
        Some(_) => partner_earns,
        None => couple.iter().filter(|(id, _)| earns(id)).count() >= 2,
    };
    if both { Filing::Single } else { Filing::Joint }
}

/// As regras para a renda de um membro com `dependents`, pelo estado civil do cadastro
/// ([`filing`]); só muda onde o país tem regras próprias para ele.
pub fn for_person(
    conn: &Connection,
    r: &rules::PayrollRules,
    member: Option<&str>,
    dependents: u32,
) -> rules::PayrollRules {
    if r.single_parent.is_none() && r.joint.is_none() {
        return r.clone();
    }
    match filing(conn, member, dependents) {
        Filing::Single => r.for_single(),
        Filing::Joint => r.for_joint(),
        Filing::SingleParent => r.for_single_parent(),
    }
}

/// O país (ou o estado escolhido) tem regras de renda.
fn has_payroll(ds: &TaxDataset, region: Option<&str>) -> bool {
    ds.payroll
        .iter()
        .any(|r| r.region.is_none() || r.region.as_deref() == region)
}

fn payroll_lines(
    conn: &Connection,
    ds: &TaxDataset,
    s: &TaxSettings,
    year: i32,
    from: &str,
    to: &str,
    notes: &mut Vec<String>,
) -> Result<Vec<PayrollLine>> {
    let members: HashMap<Id, String> = crate::household::list_members(conn, true)?
        .into_iter()
        .map(|m| (m.id, m.name))
        .collect();
    let reports = income_reports(conn, Some(year))?;
    let mut out: Vec<PayrollLine> = vec![];
    // Informes: por titular (ou da casa), somados.
    let mut reported: BTreeMap<Option<Id>, (i64, i64, i64)> = BTreeMap::new();
    for r in &reports {
        let e = reported.entry(r.member_id.clone()).or_default();
        e.0 += r.gross + r.thirteenth_gross;
        e.1 += r.social_security;
        e.2 += r.income_tax + r.thirteenth_tax;
    }
    // Salário líquido recebido no app, por titular e mês.
    // Pelo nome em pt-BR: vale com as categorias em qualquer idioma.
    let wage_cats: HashMap<Id, &'static str> = crate::categories::br_categories(conn)?
        .into_iter()
        .filter(|c| c.kind == crate::categories::CategoryKind::Income)
        .filter_map(|c| wage_kind(&c.name).map(|k| (c.id, k)))
        .collect();
    let mut stmt = conn.prepare(
        "SELECT t.member_id, substr(t.date, 1, 7), t.category_id, sum(t.amount) FROM txn t
          WHERE t.kind = 'income' AND t.status <> 'planned' AND t.date BETWEEN ?1 AND ?2
            AND t.category_id IS NOT NULL
          GROUP BY t.member_id, substr(t.date, 1, 7), t.category_id",
    )?;
    let rows = stmt
        .query_map([from, to], |r| {
            Ok((
                r.get::<_, Option<Id>>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Id>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    // titular → (mês → líquido mensal, 13º líquido)
    let mut net: BTreeMap<Option<Id>, (BTreeMap<String, i64>, i64)> = BTreeMap::new();
    for (member, month, cat, amount) in rows {
        match wage_cats.get(&cat) {
            Some(&"monthly") => {
                *net.entry(member).or_default().0.entry(month).or_default() += amount
            }
            Some(&"thirteenth") => net.entry(member).or_default().1 += amount,
            _ => {}
        }
    }
    let name = |m: &Option<Id>| m.as_ref().and_then(|id| members.get(id).cloned());
    // Holerites do ano (docs/income.md), para quem não tem informe: valores reais da folha.
    let mut stmt = conn.prepare(
        "SELECT s.member_id, SUM(p.gross), SUM(p.social_security), SUM(p.income_tax), SUM(p.net),
                COUNT(DISTINCT p.reference_month)
           FROM payslip p JOIN income_source s ON s.id = p.source_id
          WHERE substr(p.reference_month, 1, 4) = ?1 GROUP BY s.member_id",
    )?;
    let slips = stmt
        .query_map([year.to_string()], |r| {
            Ok((
                Some(r.get::<_, Id>(0)?),
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (member, gross, inss, ir, net_sum, months) in slips {
        if reported.contains_key(&member) {
            continue;
        }
        out.push(PayrollLine {
            member_id: member.clone(),
            member_name: name(&member),
            origin: "payslips".into(),
            net: net_sum,
            gross,
            social_security: inss,
            income_tax: ir,
            months,
            legal_basis: vec![],
        });
        // O líquido desse membro já está nos holerites: não estima de novo.
        net.remove(&member);
    }
    for (member, (gross, inss, ir)) in &reported {
        let received = net
            .get(member)
            .map(|(months, th)| months.values().sum::<i64>() + th)
            .unwrap_or(0);
        out.push(PayrollLine {
            member_id: member.clone(),
            member_name: name(member),
            origin: "reported".into(),
            net: received,
            gross: *gross,
            social_security: *inss,
            income_tax: *ir,
            months: net.get(member).map_or(0, |(m, _)| m.len() as i64),
            legal_basis: vec![],
        });
    }
    if !has_payroll(ds, s.region.as_deref()) {
        if !net.is_empty() {
            notes.push(t_args(
                "nucleo-impostos-nota-sem-regras",
                &args(&[("dados", ds.name.clone().into())]),
            ));
        }
        return Ok(out);
    }
    for (member, (months, thirteenth)) in &net {
        if reported.contains_key(member) {
            continue;
        }
        let (mut gross, mut inss, mut ir, mut received) = (0, 0, 0, 0);
        let mut basis: Vec<String> = vec![];
        let mut run = |net: i64, month: &str| {
            let Some(r) = rules_for(ds, month, s.region.as_deref()) else {
                return;
            };
            let r = &for_person(conn, r, member.as_deref(), s.dependents);
            let g = rules::gross_from_net(net, s.dependents, r);
            let (i, t) = rules::payroll(g, s.dependents, r);
            gross += g;
            inss += i;
            ir += t;
            received += net;
            if !basis.contains(&r.legal_basis) {
                basis.push(r.legal_basis.clone());
            }
        };
        for (month, v) in months {
            run(*v, month);
        }
        if *thirteenth > 0 {
            // 13º: tributação exclusiva, separada do salário, pela tabela de dezembro.
            run(*thirteenth, &format!("{year}-12"));
        }
        out.push(PayrollLine {
            member_id: member.clone(),
            member_name: name(member),
            origin: "estimated".into(),
            net: received,
            gross,
            social_security: inss,
            income_tax: ir,
            months: months.len() as i64,
            legal_basis: basis,
        });
    }
    if out.iter().any(|p| p.origin == "estimated") {
        notes.push(t("nucleo-impostos-nota-salario-estimado"));
    }
    Ok(out)
}

fn ipva_lines(
    conn: &Connection,
    region: Option<&TaxRegion>,
    to: &str,
    notes: &mut Vec<String>,
) -> Result<Vec<PropertyLine>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, fipe_vehicle_type, fipe_year_code FROM physical_asset
          WHERE kind = 'vehicle' AND (sale_date IS NULL OR sale_date > ?1) AND purchase_date <= ?1",
    )?;
    let vehicles = stmt
        .query_map([to], |r| {
            Ok((
                r.get::<_, Id>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if vehicles.is_empty() {
        return Ok(vec![]);
    }
    let Some(region) = region.filter(|r| r.ipva_car.is_some()) else {
        notes.push(t("nucleo-impostos-nota-sem-ipva"));
        return Ok(vec![]);
    };
    let mut out = vec![];
    for (id, name, fipe_type, fipe_year) in vehicles {
        let value = crate::physical::view(
            conn,
            &id,
            &crate::physical::PhysicalReadArgs {
                as_of: Some(to.to_string()),
                deflator: None,
                include_archived: Some(true),
            },
        )?
        .figures
        .value;
        // Isenção por idade: a do estado ou, desde 2026, 20 anos (EC 137/2025).
        let year: i32 = to[..4].parse().unwrap_or(0);
        let made: Option<i32> = fipe_year
            .as_deref()
            .and_then(|y| y.get(..4))
            .and_then(|y| y.parse().ok());
        let limit = match (region.ipva_age_exempt_years, year >= 2026) {
            (Some(s), true) => Some(s.min(20)),
            (Some(s), false) => Some(s),
            (None, true) => Some(20),
            (None, false) => None,
        };
        if let (Some(m), Some(l)) = (made, limit)
            && year - m >= l as i32
        {
            continue;
        }
        let moto = fipe_type.as_deref() == Some("motos");
        let rate = if moto {
            region.ipva_motorcycle.or(region.ipva_car)
        } else {
            region.ipva_car
        }
        .unwrap_or(0.0);
        out.push(PropertyLine {
            asset_id: id,
            name,
            tax: "ipva".into(),
            value,
            rate,
            amount: tax_of(value, rate),
            source_ids: region.source_ids.clone(),
        });
    }
    notes.push(t("nucleo-impostos-nota-ipva-estimado"));
    Ok(out)
}
