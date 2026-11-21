//! Linha da vida (docs/life-expectancy.md): para cada membro, as fases da vida, onde ele está
//! hoje e até onde, pelas médias da população do país, a vida costuma ir para alguém nascido no
//! mesmo ano e do mesmo sexo. Desligada por padrão (`longevity.enabled`).
//!
//! - Tábuas da ONU (World Population Prospects 2024, variante média) embutidas no binário
//!   ([`data`]); nada sai da máquina e nada é guardado além dos ajustes: tudo sai da data de
//!   nascimento, do sexo e do país da família.
//! - Expectativa por geração e condicionada à idade de hoje ([`math`], §4.2): a mediana, o
//!   "1 em 4" e o "1 em 10" da curva de sobrevivência; a média só no detalhe técnico.
//! - Casal: chance de pelo menos um dos dois adultos estar vivo, por ano civil (§4.4), que é o
//!   horizonte do dinheiro da família (padrão do plano de aposentadoria, §7).
//! - No Brasil, a tábua de período do IBGE de 2024 como referência ([`PeriodRef`]).

pub mod api;
pub mod data;
pub mod math;
mod milestones;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod validation;

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use self::data::{TableSex, Tables};
use self::math::Curve;
use crate::error::{Error, Result};
use crate::household::{Member, MemberRelationship, MemberSex};
use crate::i18n::{Locale, args, t, t_args};

pub const ENABLED_SETTING: &str = "longevity.enabled";
pub const HIDDEN_SETTING: &str = "longevity.hidden_members";
pub const CHILDREN_END_SETTING: &str = "longevity.show_children_end";
pub const ADVANCED_SETTING: &str = "longevity.show_advanced_later_life";

/// Última idade da curva devolvida à interface.
pub const CURVE_END: u32 = 110;
/// Idade adulta (crianças: o final da barra só com o ajuste).
pub const ADULT_AGE: u32 = 18;
/// Horizonte do plano de aposentadoria com a função desligada (o de sempre).
pub const DEFAULT_PLAN_AGE: u32 = 95;
/// Maior horizonte que o planejador aceita.
pub const MAX_PLAN_AGE: u32 = 110;

// ---------------------------------------------------------------------------
// Tipos (contrato com a interface)

/// Fase da vida (docs/life-expectancy.md, seção 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LifeStage {
    Childhood,
    Adolescence,
    Youth,
    Adulthood,
    LaterLife,
}

impl LifeStage {
    /// Nome da fase no idioma da instalação.
    pub fn label(self) -> String {
        t(match self {
            LifeStage::Childhood => "nucleo-longevidade-fase-infancia",
            LifeStage::Adolescence => "nucleo-longevidade-fase-adolescencia",
            LifeStage::Youth => "nucleo-longevidade-fase-juventude",
            LifeStage::Adulthood => "nucleo-longevidade-fase-vida-adulta",
            LifeStage::LaterLife => "nucleo-longevidade-fase-longevidade",
        })
    }
}

/// Limites de uma fase da vida, em anos de idade (`to` exclusivo; `null` = sem fim).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StageInfo {
    pub stage: LifeStage,
    pub from: u32,
    pub to: Option<u32>,
}

/// As fases (§5): Infância 0–11 (ECA), Adolescência 12–17 (ECA), Juventude 18–29 (Estatuto da
/// Juventude), Vida adulta 30–59 e Longevidade 60+ (Estatuto da Pessoa Idosa e OMS). Iguais
/// para todos os países na primeira versão.
const STAGES: &[(LifeStage, u32, Option<u32>)] = &[
    (LifeStage::Childhood, 0, Some(12)),
    (LifeStage::Adolescence, 12, Some(18)),
    (LifeStage::Youth, 18, Some(30)),
    (LifeStage::Adulthood, 30, Some(60)),
    (LifeStage::LaterLife, 60, None),
];

pub fn stages() -> Vec<StageInfo> {
    STAGES
        .iter()
        .map(|(stage, from, to)| StageInfo {
            stage: *stage,
            from: *from,
            to: *to,
        })
        .collect()
}

/// A fase numa idade.
pub fn stage_of(age: u32) -> LifeStage {
    STAGES
        .iter()
        .find(|(_, from, to)| age >= *from && to.is_none_or(|t| age < t))
        .map_or(LifeStage::LaterLife, |s| s.0)
}

/// Ajustes da linha da vida (`longevity.*` do cofre). Padrão: tudo desligado.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LongevitySettings {
    /// A função está ligada para a família.
    #[serde(default)]
    pub enabled: bool,
    /// Membros que esconderam a própria barra.
    #[serde(default)]
    pub hidden_members: Vec<String>,
    /// Mostrar o final esfumado da barra para os menores de 18.
    #[serde(default)]
    pub show_children_end: bool,
    /// Subdivisão visual da longevidade avançada (80+).
    #[serde(default)]
    pub show_advanced_later_life: bool,
}

/// Chance (0 a 1) de estar vivo numa idade, sabendo a idade de hoje.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SurvivalPoint {
    pub age: u32,
    pub survival: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneKind {
    Retirement,
    Inss,
    PlanEnd,
    DebtEnd,
    Goal,
    LifeEvent,
}

/// Marco financeiro sobre a barra (aposentadoria, INSS, fim do plano, fim de dívida, meta,
/// evento de vida do planejador).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LifeMilestone {
    #[ts(inline)]
    pub kind: MilestoneKind,
    pub age: u32,
    pub year: i32,
    pub label: String,
    /// Valor ligado ao marco, em centavos (meta, evento), quando houver.
    #[ts(type = "number | null")]
    pub amount: Option<i64>,
}

/// Referência da tábua de período do país (só Brasil: IBGE 2024).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PeriodRef {
    pub source: String,
    /// Expectativa ao nascer da tábua de período.
    pub e0: f64,
    /// Anos que ainda restam na idade de hoje, pela tábua de período.
    pub remaining_at_age: f64,
}

/// Perspectiva de longevidade de um membro (médias da população, por geração, condicionada à
/// idade de hoje).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MemberOutlook {
    pub member_id: String,
    pub name: String,
    pub age: u32,
    pub birth_year: i32,
    pub sex: Option<MemberSex>,
    pub stage: LifeStage,
    /// Idade em que metade das pessoas como o membro ainda está viva.
    pub median_age: f64,
    /// Idade que 1 em 4 passa.
    pub p25_age: f64,
    /// Idade que 1 em 10 passa.
    pub p10_age: f64,
    /// Idade média (idade de hoje + anos que ainda restam, em média).
    pub mean_age: f64,
    pub remaining_mean: f64,
    /// Expectativa ao nascer da geração (sem dado antes de 1950).
    pub cohort_e0: Option<f64>,
    pub period_ref: Option<PeriodRef>,
    pub curve: Vec<SurvivalPoint>,
    pub milestones: Vec<LifeMilestone>,
    /// Menor de 18.
    pub child: bool,
    /// Mostrar o final esfumado (falso para criança sem o ajuste).
    pub show_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MissingField {
    BirthDate,
    Sex,
}

/// O que falta no cadastro de um membro para a linha da vida.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MissingLifeData {
    pub member_id: String,
    pub name: String,
    #[ts(inline)]
    pub missing: Vec<MissingField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct YearProbability {
    pub year: i32,
    pub probability: f64,
}

/// Fonte das tábuas, no idioma da instalação.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LongevitySource {
    pub name: String,
    /// Texto de atribuição da ONU (e do IBGE no Brasil).
    pub attribution: String,
    /// Dados sintéticos (desenvolvimento), sem as tábuas reais.
    pub synthetic: bool,
}

/// Linha da vida da família (com a função desligada: `enabled: false` e listas vazias).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FamilyOutlook {
    pub enabled: bool,
    pub members: Vec<MemberOutlook>,
    pub missing: Vec<MissingLifeData>,
    /// Casal: chance de pelo menos um adulto vivo, por ano civil.
    #[ts(inline)]
    pub any_adult_alive: Option<Vec<YearProbability>>,
    pub any_alive_p25_year: Option<i32>,
    pub any_alive_p10_year: Option<i32>,
    pub country: String,
    pub source: LongevitySource,
}

/// Risco de longevidade no resultado do planejador de aposentadoria (§7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RetirementLongevity {
    /// Idade até onde o plano vai (a do membro principal).
    pub plan_age: u32,
    /// Chance de passar dessa idade (do casal, quando há cônjuge: pelo menos um vivo).
    pub survival: f64,
    /// A idade "1 em 10", arredondada para cima.
    pub suggested_age: u32,
}

/// Padrões do planejador de aposentadoria a partir do membro (`goals.retirement_defaults`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RetirementDefaults {
    pub current_age: Option<u32>,
    /// Sugestão do horizonte: a idade "1 em 10" com a linha da vida ligada; 95 desligada.
    pub life_expectancy: u32,
    pub from_longevity: bool,
}

// ---------------------------------------------------------------------------
// Ajustes

/// A função está ligada (sem o ajuste ou com erro: desligada).
pub fn enabled(conn: &Connection) -> bool {
    crate::settings::get_or(conn, ENABLED_SETTING, false).unwrap_or(false)
}

pub fn set_enabled(conn: &Connection, on: bool) -> Result<()> {
    crate::settings::set(conn, ENABLED_SETTING, &on)
}

pub fn settings(conn: &Connection) -> Result<LongevitySettings> {
    Ok(LongevitySettings {
        enabled: crate::settings::get_or(conn, ENABLED_SETTING, false)?,
        hidden_members: crate::settings::get_or(conn, HIDDEN_SETTING, Vec::new())?,
        show_children_end: crate::settings::get_or(conn, CHILDREN_END_SETTING, false)?,
        show_advanced_later_life: crate::settings::get_or(conn, ADVANCED_SETTING, false)?,
    })
}

/// Grava os ajustes (os membros escondidos são conferidos e sem repetição).
pub fn save_settings(conn: &Connection, s: &LongevitySettings) -> Result<LongevitySettings> {
    let mut hidden: Vec<String> = vec![];
    for id in &s.hidden_members {
        crate::household::get_member(conn, id)?;
        if !hidden.contains(id) {
            hidden.push(id.clone());
        }
    }
    crate::settings::set(conn, ENABLED_SETTING, &s.enabled)?;
    crate::settings::set(conn, HIDDEN_SETTING, &hidden)?;
    crate::settings::set(conn, CHILDREN_END_SETTING, &s.show_children_end)?;
    crate::settings::set(conn, ADVANCED_SETTING, &s.show_advanced_later_life)?;
    settings(conn)
}

// ---------------------------------------------------------------------------
// Cálculo por membro

/// Idade completa numa data (`None` antes de nascer).
pub(crate) fn age_on(birth: NaiveDate, on: NaiveDate) -> Option<u32> {
    let mut a = on.year() - birth.year();
    if (on.month(), on.day()) < (birth.month(), birth.day()) {
        a -= 1;
    }
    u32::try_from(a).ok()
}

fn table_sex(sex: Option<MemberSex>) -> TableSex {
    match sex {
        Some(MemberSex::Male) => TableSex::Male,
        Some(MemberSex::Female) => TableSex::Female,
        _ => TableSex::Both,
    }
}

fn round_to(x: f64, places: i32) -> f64 {
    let f = 10f64.powi(places);
    (x * f).round() / f
}

/// Onde calcular: as tábuas, o país e hoje.
pub(crate) struct Basis<'a> {
    pub tables: &'a Tables,
    pub country: usize,
    pub country_code: String,
    pub today: NaiveDate,
}

impl<'a> Basis<'a> {
    pub fn new(conn: &Connection, tables: &'a Tables, today: NaiveDate) -> Result<Self> {
        let code = crate::country::family_country(conn);
        let country = tables.country(&code).ok_or_else(|| {
            Error::invalid(t_args(
                "nucleo-longevidade-erro-pais",
                &args(&[("pais", crate::country::country_name(&code).into())]),
            ))
        })?;
        Ok(Self {
            tables,
            country,
            country_code: code,
            today,
        })
    }

    fn brazil(&self) -> bool {
        self.country_code == "BR"
    }
}

/// Data de nascimento e idade de hoje de um membro.
fn birth_and_age(m: &Member, today: NaiveDate) -> Option<(NaiveDate, u32)> {
    let birth = crate::time::parse_date(m.profile.birth_date.as_deref()?).ok()?;
    Some((birth, age_on(birth, today)?))
}

/// Curva de sobrevivência do membro a partir de hoje (`None` sem data de nascimento).
fn member_curve(b: &Basis<'_>, m: &Member) -> Option<(NaiveDate, u32, Curve)> {
    let (birth, age) = birth_and_age(m, b.today)?;
    let view = b.tables.view(b.country, table_sex(m.profile.sex));
    let c = math::curve(&view, age, b.today.year());
    Some((birth, age, c))
}

fn period_ref(b: &Basis<'_>, sex: TableSex, age: u32) -> Option<PeriodRef> {
    if !b.brazil() {
        return None;
    }
    let ibge = b.tables.ibge.as_ref()?.of(sex);
    let at = (age as usize).min(ibge.ex.len() - 1);
    Some(PeriodRef {
        source: t(if b.tables.synthetic {
            "nucleo-longevidade-ibge-fonte-sintetica"
        } else {
            "nucleo-longevidade-ibge-fonte"
        }),
        e0: round_to(ibge.ex[0], 1),
        remaining_at_age: round_to(ibge.ex[at], 1),
    })
}

/// Membro principal (quem cuida das finanças): o marcado como principal, senão o titular.
fn primary(members: &[Member]) -> Option<&Member> {
    members
        .iter()
        .find(|m| m.is_primary && !m.archived)
        .or_else(|| {
            members.iter().find(|m| {
                !m.archived && m.profile.relationship == Some(MemberRelationship::Titular)
            })
        })
}

/// O outro adulto do casal: o cônjuge do principal (ou o principal, para o cônjuge).
fn partner_of<'m>(members: &'m [Member], m: &Member) -> Option<&'m Member> {
    let main = primary(members)?;
    if main.id == m.id {
        members.iter().find(|x| {
            x.id != m.id
                && !x.archived
                && x.profile.relationship == Some(MemberRelationship::Conjuge)
        })
    } else if m.profile.relationship == Some(MemberRelationship::Conjuge) {
        Some(main)
    } else {
        None
    }
}

fn member_outlook(
    conn: &Connection,
    b: &Basis<'_>,
    m: &Member,
    s: &LongevitySettings,
    primary_id: Option<&str>,
) -> Result<Option<MemberOutlook>> {
    let Some((birth, age, c)) = member_curve(b, m) else {
        return Ok(None);
    };
    let sex = table_sex(m.profile.sex);
    let remaining = c.remaining_mean();
    let child = age < ADULT_AGE;
    let curve = (age..=CURVE_END.max(age))
        .map(|a| SurvivalPoint {
            age: a,
            survival: round_to(c.at(a), 4),
        })
        .collect();
    Ok(Some(MemberOutlook {
        member_id: m.id.clone(),
        name: m.name.clone(),
        age,
        birth_year: birth.year(),
        sex: m.profile.sex,
        stage: stage_of(age),
        median_age: round_to(c.quantile(0.5), 1),
        p25_age: round_to(c.quantile(0.25), 1),
        p10_age: round_to(c.quantile(0.10), 1),
        mean_age: round_to(f64::from(age) + remaining, 1),
        remaining_mean: round_to(remaining, 1),
        cohort_e0: b
            .tables
            .cohort_e0(b.country, sex, birth.year())
            .map(|v| round_to(v, 1)),
        period_ref: period_ref(b, sex, age),
        curve,
        milestones: milestones::of_member(conn, m, birth, primary_id, b.today)?,
        child,
        show_end: !child || s.show_children_end,
    }))
}

fn no_birth_date(m: &Member) -> Error {
    Error::invalid(t_args(
        "nucleo-longevidade-erro-sem-nascimento",
        &args(&[("nome", m.name.clone().into())]),
    ))
}

/// A linha da vida de um membro (erro `invalid` sem data de nascimento).
pub fn outlook(conn: &Connection, member_id: &str) -> Result<MemberOutlook> {
    outlook_on(
        conn,
        data::embedded()?,
        member_id,
        crate::time::today_local(),
    )
}

pub fn outlook_on(
    conn: &Connection,
    tables: &Tables,
    member_id: &str,
    today: NaiveDate,
) -> Result<MemberOutlook> {
    let m = crate::household::get_member(conn, member_id)?;
    let members = crate::household::list_members(conn, false)?;
    let b = Basis::new(conn, tables, today)?;
    let s = settings(conn)?;
    member_outlook(conn, &b, &m, &s, primary(&members).map(|p| p.id.as_str()))?
        .ok_or_else(|| no_birth_date(&m))
}

// ---------------------------------------------------------------------------
// Família

/// O casal (principal e cônjuge), adultos com data de nascimento.
fn couple(b: &Basis<'_>, members: &[Member]) -> Option<(Curve, Curve)> {
    let main = primary(members)?;
    let other = partner_of(members, main)?;
    let (_, a1, c1) = member_curve(b, main)?;
    let (_, a2, c2) = member_curve(b, other)?;
    (a1 >= ADULT_AGE && a2 >= ADULT_AGE).then_some((c1, c2))
}

pub fn family(conn: &Connection) -> Result<FamilyOutlook> {
    family_on(conn, data::embedded()?, crate::time::today_local())
}

pub fn family_on(conn: &Connection, tables: &Tables, today: NaiveDate) -> Result<FamilyOutlook> {
    let s = settings(conn)?;
    let country = crate::country::family_country(conn);
    let source = source_of(tables, &country);
    if !s.enabled {
        return Ok(FamilyOutlook {
            enabled: false,
            members: vec![],
            missing: vec![],
            any_adult_alive: None,
            any_alive_p25_year: None,
            any_alive_p10_year: None,
            country,
            source,
        });
    }
    let b = Basis::new(conn, tables, today)?;
    let members = crate::household::list_members(conn, false)?;
    let primary_id = primary(&members).map(|p| p.id.clone());
    let mut outlooks = vec![];
    let mut missing = vec![];
    for m in members.iter().filter(|m| !s.hidden_members.contains(&m.id)) {
        let mut lacks = vec![];
        if birth_and_age(m, today).is_none() {
            lacks.push(MissingField::BirthDate);
        }
        if m.profile.sex.is_none() {
            lacks.push(MissingField::Sex);
        }
        if !lacks.is_empty() {
            missing.push(MissingLifeData {
                member_id: m.id.clone(),
                name: m.name.clone(),
                missing: lacks,
            });
        }
        if let Some(o) = member_outlook(conn, &b, m, &s, primary_id.as_deref())? {
            outlooks.push(o);
        }
    }
    let (mut points, mut p25, mut p10) = (None, None, None);
    if let Some((c1, c2)) = couple(&b, &members) {
        let year = today.year();
        let end = CURVE_END.saturating_sub(c1.from.min(c2.from)).max(1);
        let mut list = vec![];
        for k in 0..=end {
            let p = math::either_alive(&c1, &c2, k);
            let y = year + k as i32;
            if p25.is_none() && p <= 0.25 {
                p25 = Some(y);
            }
            if p10.is_none() && p <= 0.10 {
                p10 = Some(y);
            }
            list.push(YearProbability {
                year: y,
                probability: round_to(p, 4),
            });
            if p < 0.001 {
                break;
            }
        }
        points = Some(list);
    }
    Ok(FamilyOutlook {
        enabled: true,
        members: outlooks,
        missing,
        any_adult_alive: points,
        any_alive_p25_year: p25,
        any_alive_p10_year: p10,
        country,
        source,
    })
}

/// Fonte e atribuição no idioma da instalação.
pub fn source_of(tables: &Tables, country: &str) -> LongevitySource {
    let brazil = country == "BR" && tables.ibge.is_some();
    let name = t(if tables.synthetic {
        "nucleo-longevidade-fonte-nome-sintetica"
    } else if brazil {
        "nucleo-longevidade-fonte-nome-brasil"
    } else {
        "nucleo-longevidade-fonte-nome"
    });
    let mut attribution = t("nucleo-longevidade-atribuicao-onu");
    if brazil {
        attribution.push(' ');
        attribution.push_str(&t("nucleo-longevidade-atribuicao-ibge"));
    }
    if tables.synthetic {
        attribution.push(' ');
        attribution.push_str(&t("nucleo-longevidade-atribuicao-sintetica"));
    }
    LongevitySource {
        name,
        attribution,
        synthetic: tables.synthetic,
    }
}

// ---------------------------------------------------------------------------
// Aposentadoria (§7)

/// Horizonte de um membro (com o cônjuge, quando há): a chance de passar da idade `plan_age`
/// e a idade "1 em 10" arredondada para cima, nas idades do membro.
fn horizon(b: &Basis<'_>, members: &[Member], m: &Member, plan_age: u32) -> Option<(f64, u32)> {
    let (_, age, own) = member_curve(b, m)?;
    let other = partner_of(members, m)
        .and_then(|p| member_curve(b, p))
        .filter(|(_, a, _)| *a >= ADULT_AGE)
        .map(|(_, _, c)| c);
    let k = plan_age.saturating_sub(age);
    let (survival, p10) = match &other {
        Some(o) => (
            math::either_alive(&own, o, k),
            f64::from(age) + math::either_alive_until(&own, o, 0.10),
        ),
        None => (own.after(k), own.quantile(0.10)),
    };
    // Pelo menos um ano depois de hoje, sem passar do que o planejador aceita.
    let floor = (age + 1).min(MAX_PLAN_AGE);
    let suggested = (p10 - 1e-9)
        .ceil()
        .clamp(f64::from(floor), f64::from(MAX_PLAN_AGE)) as u32;
    Some((round_to(survival, 4), suggested))
}

/// Membro do plano: o pedido ou o principal.
fn plan_member<'m>(members: &'m [Member], member_id: Option<&str>) -> Option<&'m Member> {
    match member_id {
        Some(id) => members.iter().find(|m| m.id == id),
        None => primary(members),
    }
}

/// O bloco `longevity` do resultado da simulação: `None` com a função desligada ou sem a
/// data de nascimento do membro principal.
pub fn retirement_block(
    conn: &Connection,
    plan_age: u32,
    today: NaiveDate,
) -> Result<Option<RetirementLongevity>> {
    if !enabled(conn) {
        return Ok(None);
    }
    retirement_block_on(conn, data::embedded()?, plan_age, today)
}

pub fn retirement_block_on(
    conn: &Connection,
    tables: &Tables,
    plan_age: u32,
    today: NaiveDate,
) -> Result<Option<RetirementLongevity>> {
    let members = crate::household::list_members(conn, false)?;
    let Some(m) = plan_member(&members, None) else {
        return Ok(None);
    };
    let b = Basis::new(conn, tables, today)?;
    Ok(
        horizon(&b, &members, m, plan_age).map(|(survival, suggested_age)| RetirementLongevity {
            plan_age,
            survival,
            suggested_age,
        }),
    )
}

/// Padrões do planejador (`goals.retirement_defaults`): a idade de hoje pela data de
/// nascimento e o horizonte "1 em 10" (do casal, quando há cônjuge) com a função ligada; com
/// ela desligada, 95.
pub fn retirement_defaults(
    conn: &Connection,
    member_id: Option<&str>,
) -> Result<RetirementDefaults> {
    let tables = if enabled(conn) {
        Some(data::embedded()?)
    } else {
        None
    };
    retirement_defaults_on(conn, tables, member_id, crate::time::today_local())
}

pub fn retirement_defaults_on(
    conn: &Connection,
    tables: Option<&Tables>,
    member_id: Option<&str>,
    today: NaiveDate,
) -> Result<RetirementDefaults> {
    let members = crate::household::list_members(conn, true)?;
    if let Some(id) = member_id {
        crate::household::get_member(conn, id)?;
    }
    let m = plan_member(&members, member_id);
    let current_age = m.and_then(|m| birth_and_age(m, today)).map(|(_, a)| a);
    let from_tables = match (tables, m) {
        (Some(t), Some(m)) if current_age.is_some() => {
            let b = Basis::new(conn, t, today)?;
            horizon(&b, &members, m, DEFAULT_PLAN_AGE).map(|(_, s)| s)
        }
        _ => None,
    };
    Ok(RetirementDefaults {
        current_age,
        life_expectancy: from_tables.unwrap_or(DEFAULT_PLAN_AGE),
        from_longevity: from_tables.is_some(),
    })
}

// ---------------------------------------------------------------------------
// Textos (MCP)

/// Número com uma casa decimal no idioma (`84,2` / `84.2`).
pub(crate) fn dec1(l: Locale, x: f64) -> String {
    let s = format!("{x:.1}");
    match l {
        Locale::En => s,
        _ => s.replace('.', ","),
    }
}
