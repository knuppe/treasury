//! Dados dos ativos com proveniência, fase 1 do enriquecimento (docs/agent-enrichment.md):
//! completa setor, segmento, ISIN, CNPJ e os campos novos do cadastro a partir do que o cofre
//! já baixou das fontes oficiais (emissores e códigos da CVM e da SEC, registro de fundos da
//! CVM) e do Fundamentus em cache. Sem rede e sem IA.
//!
//! - Cada leitura é uma [`FieldObservation`] (campo, valor, fonte, nível, data); as de um campo são
//!   normalizadas e comparadas, e o melhor nível vira o [`FieldCandidate`] com a confiança (§4.3).
//!   Duas fontes do mesmo nível que discordam = conflito: nenhuma vem escolhida.
//! - Cada campo tem uma origem (`asset_field_meta`). O que o usuário digitou (ou o que já
//!   estava preenchido antes desta função, `legacy`) nunca é trocado sem pedido explícito
//!   (`replace`): a diferença aparece só como divergência. Campo travado não recebe sugestão.
//! - [`apply`] só grava valores que estão numa observação atual do ativo (a IA não grava um
//!   setor "de cabeça") e passa pelo histórico de alterações: desfazer volta o valor e a
//!   origem juntos. Para a IA, o comando vira proposta sensível (nunca no modo autônomo).
//!
//! Textos vindos das fontes (setores, nomes, gestores) são dados, nunca instruções.

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Asset, AssetClass};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::research::{cnpj_digits, format_cnpj, norm};
use crate::time::now_utc;

/// Origem do valor de um campo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FieldOrigin {
    /// Digitado na interface.
    User,
    /// Já preenchido antes desta função, sem proveniência: protegido como o do usuário.
    Legacy,
    Import,
    /// Veio de uma sugestão aprovada.
    Enrichment,
    Search,
}

impl FieldOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            FieldOrigin::User => "user",
            FieldOrigin::Legacy => "legacy",
            FieldOrigin::Import => "import",
            FieldOrigin::Enrichment => "enrichment",
            FieldOrigin::Search => "search",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "user" => FieldOrigin::User,
            "import" => FieldOrigin::Import,
            "enrichment" => FieldOrigin::Enrichment,
            "search" => FieldOrigin::Search,
            _ => FieldOrigin::Legacy,
        }
    }
    /// Valor do usuário: só divergência, nunca troca sem pedido.
    fn protected(self) -> bool {
        matches!(self, FieldOrigin::User | FieldOrigin::Legacy)
    }
}

/// Um campo do catálogo: coluna do `asset` ou atributo (`asset_attribute`).
pub struct FieldDef {
    pub key: &'static str,
    pub column: bool,
    /// Classes em que o campo faz sentido (as que contam como "faltando").
    pub classes: &'static [AssetClass],
    /// Idade a partir da qual a observação perde 10 pontos de confiança.
    pub stale_days: i64,
}

use AssetClass as C;

const LISTED: &[AssetClass] = &[
    C::StockBr,
    C::Fii,
    C::EtfBr,
    C::Bdr,
    C::StockUs,
    C::EtfUs,
    C::Reit,
    C::StockIntl,
    C::EtfIntl,
];
const STOCKS: &[AssetClass] = &[C::StockBr, C::Bdr, C::StockUs, C::Reit, C::StockIntl];
const FUNDS: &[AssetClass] = &[C::EtfBr, C::Fund, C::Pension];

/// Catálogo dos campos enriquecidos na fase 1 (§3.1).
pub const FIELDS: &[FieldDef] = &[
    FieldDef {
        key: "ticker",
        column: true,
        classes: &[C::StockBr, C::Fii, C::EtfBr, C::Bdr],
        stale_days: 365,
    },
    FieldDef {
        key: "sector",
        column: true,
        classes: STOCKS,
        stale_days: 365,
    },
    FieldDef {
        key: "subsector",
        column: false,
        classes: &[C::StockBr, C::Bdr],
        stale_days: 365,
    },
    FieldDef {
        key: "segment",
        column: true,
        classes: &[C::Fii, C::StockBr, C::Bdr],
        stale_days: 365,
    },
    FieldDef {
        key: "isin",
        column: true,
        classes: LISTED,
        stale_days: 365,
    },
    FieldDef {
        key: "cnpj",
        column: true,
        classes: &[C::Fii, C::EtfBr, C::Fund, C::Pension],
        stale_days: 365,
    },
    FieldDef {
        key: "issuer_cnpj",
        column: false,
        classes: &[C::StockBr, C::Bdr],
        stale_days: 365,
    },
    FieldDef {
        key: "issuer_name",
        column: false,
        classes: STOCKS,
        stale_days: 365,
    },
    FieldDef {
        key: "share_kind",
        column: false,
        classes: &[C::StockBr, C::Fii, C::StockUs],
        stale_days: 365,
    },
    FieldDef {
        key: "listing_segment",
        column: false,
        classes: &[C::StockBr],
        stale_days: 365,
    },
    FieldDef {
        key: "cvm_code",
        column: false,
        classes: &[C::StockBr],
        stale_days: 365,
    },
    FieldDef {
        key: "cik",
        column: false,
        classes: &[C::StockUs, C::Reit],
        stale_days: 365,
    },
    FieldDef {
        key: "fii_mandate",
        column: false,
        classes: &[C::Fii],
        stale_days: 365,
    },
    FieldDef {
        key: "fii_management",
        column: false,
        classes: &[C::Fii],
        stale_days: 365,
    },
    FieldDef {
        key: "fund_class_cvm",
        column: false,
        classes: FUNDS,
        stale_days: 365,
    },
    FieldDef {
        key: "fund_class_anbima",
        column: false,
        classes: FUNDS,
        stale_days: 365,
    },
    FieldDef {
        key: "benchmark",
        column: false,
        classes: FUNDS,
        stale_days: 365,
    },
    FieldDef {
        key: "manager",
        column: false,
        classes: FUNDS,
        stale_days: 90,
    },
    FieldDef {
        key: "administrator",
        column: false,
        classes: FUNDS,
        stale_days: 90,
    },
    // Fase 2 (fontes da rede: B3, GLEIF, BrasilAPI).
    FieldDef {
        key: "lei",
        column: false,
        classes: &[C::StockBr, C::Bdr],
        stale_days: 365,
    },
    FieldDef {
        key: "website",
        column: false,
        classes: &[C::StockBr, C::Bdr],
        stale_days: 365,
    },
    FieldDef {
        key: "activity",
        column: false,
        classes: &[C::StockBr, C::Bdr],
        stale_days: 365,
    },
    FieldDef {
        key: "cnpj_status",
        column: false,
        classes: &[C::StockBr, C::Bdr, C::Fii, C::EtfBr, C::Fund, C::Pension],
        stale_days: 90,
    },
    // Tipo do plano de previdência (vgbl, pgbl, efpc): o valor atual é o do plano
    // (`pension_plan`) e a leitura vem da natureza jurídica do CNPJ (fundo de pensão).
    FieldDef {
        key: "pension_type",
        column: false,
        classes: &[C::Pension],
        stale_days: 365,
    },
];

pub fn field_def(key: &str) -> Result<&'static FieldDef> {
    FIELDS.iter().find(|f| f.key == key).ok_or_else(|| {
        Error::invalid(t_args(
            "nucleo-ativos-dados-erro-campo",
            &args(&[("campo", key.to_string().into())]),
        ))
    })
}

/// Nível da fonte (§4.1): 1 oficial, 2 emissor, 3 secundária, 4 IA.
fn tier_weight(tier: u8) -> i64 {
    match tier {
        1 => 90,
        2 => 80,
        3 => 60,
        _ => 15,
    }
}

/// Faixa da confiança: alta ≥ 85, média 60 a 84, baixa < 60.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FieldBand {
    High,
    Medium,
    Low,
}

fn band(c: i64) -> FieldBand {
    if c >= 85 {
        FieldBand::High
    } else if c >= 60 {
        FieldBand::Medium
    } else {
        FieldBand::Low
    }
}

/// Uma leitura de uma fonte para um campo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldObservation {
    pub field: String,
    pub value: String,
    /// Valor normalizado para comparar fontes (setor canônico, CNPJ só com dígitos…).
    pub value_norm: String,
    /// Esquema da classificação (`cvm_fca`, `sic`, `fundamentus`…).
    pub scheme: Option<String>,
    /// `cvm_fca`, `cvm_fii`, `cvm_registro`, `sec`, `b3`, `fundamentus`.
    pub source: String,
    pub tier: u8,
    pub source_ref: Option<String>,
    /// Data do dado na fonte (AAAA-MM-DD), quando conhecida.
    pub as_of: Option<String>,
    /// Ressalva da leitura: `matched_by_name` (fundo achado pelo nome, não pelo CNPJ).
    pub note: Option<String>,
    #[serde(skip)]
    #[ts(skip)]
    penalty: i64,
}

impl FieldObservation {
    /// Leitura gravada (fontes da rede, `asset_observation`).
    pub(crate) fn new(
        field: &str,
        value: &str,
        scheme: Option<String>,
        source: &str,
        tier: u8,
        source_ref: Option<String>,
        as_of: Option<String>,
    ) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
            value_norm: normalize(field, value),
            scheme,
            source: source.into(),
            tier,
            source_ref,
            as_of,
            note: None,
            penalty: 0,
        }
    }
}

/// Ressalva das leituras que só conferem outra fonte (o setor do FCA quando há a
/// classificação da B3): não disputam, só somam concordância.
const CONFERENCE: &str = "conference";

/// O valor sugerido para um campo, com a confiança.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldCandidate {
    pub value: String,
    pub source: String,
    pub tier: u8,
    pub as_of: Option<String>,
    pub source_ref: Option<String>,
    #[ts(type = "number")]
    pub confidence: i64,
    pub band: FieldBand,
    /// Outras fontes que dizem o mesmo (depois da normalização).
    pub agreeing: Vec<String>,
    pub note: Option<String>,
}

/// O que fazer com um campo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FieldState {
    /// Preenchido e de acordo com as fontes (ou travado, ou sem fonte para conferir).
    Ok,
    /// Vazio e sem fonte no cofre.
    Missing,
    /// Vazio, com sugestão.
    Fill,
    /// Valor de uma sugestão anterior (ou importação) que a fonte mudou: troca sensível.
    Update,
    /// O usuário digitou outro valor: só aviso, troca só se ele pedir.
    Diverge,
    /// Fontes do mesmo nível discordam: o usuário escolhe.
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldStatus {
    pub field: String,
    pub current: Option<String>,
    pub origin: Option<FieldOrigin>,
    pub locked: bool,
    /// Fonte, data e confiança do valor atual (quando veio de uma sugestão aprovada).
    pub current_source: Option<String>,
    pub current_as_of: Option<String>,
    #[ts(type = "number | null")]
    pub current_confidence: Option<i64>,
    pub state: FieldState,
    pub candidate: Option<FieldCandidate>,
    /// Em conflito: as opções, uma por valor.
    pub options: Vec<FieldCandidate>,
    /// Efeito da troca no score (segmento dos pares), quando muda.
    pub effect: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetData {
    pub asset_id: String,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub fields: Vec<FieldStatus>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataGapsSummary {
    #[ts(type = "number")]
    pub assets: i64,
    #[ts(type = "number")]
    pub missing_sector: i64,
    #[ts(type = "number")]
    pub missing_isin: i64,
    /// Campos vazios com sugestão.
    #[ts(type = "number")]
    pub fill: i64,
    /// Sugestões de alta confiança (marcadas de saída no "aprovar todos").
    #[ts(type = "number")]
    pub fill_high: i64,
    #[ts(type = "number")]
    pub updates: i64,
    #[ts(type = "number")]
    pub conflicts: i64,
    #[ts(type = "number")]
    pub divergences: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataGaps {
    pub summary: DataGapsSummary,
    pub assets: Vec<AssetData>,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct DataGapsArgs {
    /// Só estes ativos; sem ele, todos os não arquivados.
    #[ts(optional)]
    pub asset_ids: Option<Vec<String>>,
    /// Só ativos com operações (em carteira ou encerrados).
    #[ts(optional)]
    pub held_only: Option<bool>,
    /// Inclui os campos sem nada a fazer (`ok` e `missing`); padrão: só os com ação.
    #[ts(optional)]
    pub include_ok: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct EnrichItem {
    pub asset_id: String,
    pub field: String,
    /// Um dos valores observados para o campo (ver `assets.data_gaps`).
    pub value: String,
    /// Troca um valor já preenchido (do usuário ou de outra fonte). Sem ele, só campos vazios
    /// ou o valor de uma sugestão anterior.
    #[serde(default)]
    #[ts(optional)]
    pub replace: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct EnrichApplyArgs {
    pub items: Vec<EnrichItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EnrichApplied {
    #[ts(type = "number")]
    pub fields: i64,
    #[ts(type = "number")]
    pub assets: i64,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct FieldLockArgs {
    pub asset_id: String,
    pub field: String,
    pub locked: bool,
}

/// Origem e todas as leituras de um campo ("de onde veio").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldSources {
    pub status: FieldStatus,
    /// Leituras atuais das fontes no cofre.
    pub observations: Vec<FieldObservation>,
    /// Leituras gravadas quando uma sugestão foi aprovada (mais novas primeiro).
    pub history: Vec<StoredFieldObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoredFieldObservation {
    pub value: String,
    pub source: String,
    pub tier: u8,
    pub source_ref: Option<String>,
    pub published_at: Option<String>,
    pub observed_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AssetFieldSources {
    pub asset_id: String,
    pub fields: Vec<FieldSources>,
}

// ---------------------------------------------------------------------------------------
// Validação e normalização

/// CNPJ com os dois dígitos verificadores certos (14 dígitos, não todos iguais).
pub fn valid_cnpj(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 14 || d.iter().all(|x| *x == d[0]) {
        return false;
    }
    let dv = |n: usize| {
        let w: &[u32] = if n == 12 {
            &[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]
        } else {
            &[6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]
        };
        let s: u32 = d[..n].iter().zip(w).map(|(a, b)| a * b).sum();
        let r = s % 11;
        if r < 2 { 0 } else { 11 - r }
    };
    dv(12) == d[12] && dv(13) == d[13]
}

/// ISIN (ISO 6166) com o dígito verificador certo.
pub fn valid_isin(s: &str) -> bool {
    let s = s.trim().to_uppercase();
    let b = s.as_bytes();
    if b.len() != 12
        || !b[..2].iter().all(u8::is_ascii_uppercase)
        || !b[2..11].iter().all(u8::is_ascii_alphanumeric)
        || !b[11].is_ascii_digit()
    {
        return false;
    }
    // Letras viram números (A = 10 … Z = 35); depois, Luhn sobre a sequência de dígitos.
    let mut digits = String::new();
    for c in s[..11].chars() {
        match c.to_digit(36) {
            Some(v) => digits.push_str(&v.to_string()),
            None => return false,
        }
    }
    let mut sum = 0;
    for (i, c) in digits.chars().rev().enumerate() {
        let mut v = c.to_digit(10).unwrap_or(0);
        if i % 2 == 0 {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
    }
    (10 - sum % 10) % 10 == u32::from(b[11] - b'0')
}

/// Valor normalizado de um campo para comparar fontes.
pub fn normalize(field: &str, value: &str) -> String {
    match field {
        "sector" | "subsector" => crate::score::taxonomy::normalize_sector(value),
        "cnpj" | "issuer_cnpj" => cnpj_digits(value),
        "isin" | "share_kind" | "cik" | "cvm_code" | "ticker" => value.trim().to_uppercase(),
        _ => {
            let n = norm(value);
            let n = n
                .strip_suffix(" s a")
                .or_else(|| n.strip_suffix(" sa"))
                .unwrap_or(&n);
            n.trim().to_string()
        }
    }
}

fn clean(s: Option<&str>) -> Option<String> {
    s.map(str::trim).filter(|s| !s.is_empty()).map(String::from)
}

fn date_part(s: &str) -> String {
    s.chars().take(10).collect()
}

// ---------------------------------------------------------------------------------------
// Leituras do cofre

struct Obs<'a> {
    out: &'a mut Vec<FieldObservation>,
}

impl Obs<'_> {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        field: &str,
        value: Option<&str>,
        scheme: Option<&str>,
        source: &str,
        tier: u8,
        source_ref: Option<&str>,
        as_of: Option<&str>,
        note: Option<&str>,
        penalty: i64,
    ) {
        let Some(value) = clean(value) else {
            return;
        };
        let value = match field {
            "cnpj" | "issuer_cnpj" => {
                if !valid_cnpj(&value) {
                    return;
                }
                format_cnpj(&cnpj_digits(&value))
            }
            "isin" => {
                if !valid_isin(&value) {
                    return;
                }
                value.to_uppercase()
            }
            _ => value,
        };
        let value_norm = normalize(field, &value);
        if value_norm.is_empty() {
            return;
        }
        self.out.push(FieldObservation {
            field: field.into(),
            value,
            value_norm,
            scheme: scheme.map(String::from),
            source: source.into(),
            tier,
            source_ref: source_ref.map(String::from),
            as_of: as_of.map(date_part),
            note: note.map(String::from),
            penalty,
        });
    }
}

/// Fonte e nível de um registro da pesquisa (`research_issuer.source`).
fn research_source(s: &str) -> (&'static str, u8) {
    match s {
        "cvm_fii" => ("cvm_fii", 1),
        x if x.starts_with("sec") => ("sec", 1),
        x if x.starts_with("b3") => ("b3", 1),
        "brapi" => ("brapi", 3),
        _ => ("cvm_fca", 1),
    }
}

struct IssuerRow {
    id: String,
    kind: String,
    name: String,
    cvm_code: Option<String>,
    sector: Option<String>,
    segment: Option<String>,
    mandate: Option<String>,
    source: String,
    fetched_at: String,
}

fn issuer_row(conn: &Connection, id: &str) -> Result<Option<IssuerRow>> {
    Ok(conn
        .query_row(
            "SELECT id, kind, name, cvm_code, sector, segment, mandate, source, fetched_at
               FROM research_issuer WHERE id = ?1",
            [id],
            |r| {
                Ok(IssuerRow {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    name: r.get(2)?,
                    cvm_code: r.get(3)?,
                    sector: r.get(4)?,
                    segment: r.get(5)?,
                    mandate: r.get(6)?,
                    source: r.get(7)?,
                    fetched_at: r.get(8)?,
                })
            },
        )
        .optional()?)
}

fn security_fetched_at(conn: &Connection, ticker: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT fetched_at FROM research_security WHERE ticker = ?1",
            [ticker],
            |r| r.get(0),
        )
        .optional()?)
}

/// Retrato do Fundamentus em cache, mesmo fora do prazo de frescor (setor muda pouco).
fn fundamentus(
    conn: &Connection,
    ticker: &str,
) -> Result<Option<crate::research::fundamentus::FundamentusSnapshot>> {
    let row: Option<String> = conn
        .query_row(
            "SELECT payload FROM fundamentus_cache WHERE ticker = ?1 AND page = 'details' AND found = 1",
            [ticker],
            |r| r.get(0),
        )
        .optional()?;
    Ok(row.and_then(|p| serde_json::from_str(&p).ok()))
}

/// Contexto compartilhado entre os ativos de uma leitura (o registro de fundos é um ajuste
/// grande: lido uma vez).
#[derive(Default)]
pub struct Sources {
    registry: Option<crate::score::funds::registry::Registry>,
}

impl Sources {
    fn registry(&mut self, conn: &Connection) -> Result<&crate::score::funds::registry::Registry> {
        if self.registry.is_none() {
            self.registry = Some(crate::score::funds::registry::load(conn)?);
        }
        Ok(self.registry.as_ref().expect("carregado acima"))
    }
}

/// Todas as leituras do cofre para um ativo.
pub fn observations(
    conn: &Connection,
    a: &Asset,
    src: &mut Sources,
) -> Result<Vec<FieldObservation>> {
    let mut out = vec![];
    let mut o = Obs { out: &mut out };
    let class = a.class;
    // Emissor e código de negociação (CVM: FCA e informe de FII; SEC).
    let security = match a.ticker.as_deref() {
        Some(t) if class.is_exchange_listed() => {
            crate::research::data::security_of(conn, t, Some(class))?
        }
        _ => None,
    };
    let issuer_id = crate::research::data::issuer_of_asset(conn, a)?;
    let issuer = match &issuer_id {
        Some(i) => issuer_row(conn, i)?,
        None => None,
    };
    if let Some(i) = &issuer {
        let (source, tier) = research_source(&i.source);
        let at = Some(i.fetched_at.as_str());
        let is_fii = i.kind == "fii";
        let us = i.kind == "company_us";
        if !is_fii && STOCKS.contains(&class) {
            let scheme = if us { "sic" } else { "cvm_fca" };
            o.push(
                "sector",
                i.sector.as_deref(),
                Some(scheme),
                source,
                tier,
                None,
                at,
                None,
                0,
            );
            o.push(
                "issuer_name",
                Some(&i.name),
                None,
                source,
                tier,
                None,
                at,
                None,
                0,
            );
        }
        if is_fii && class == C::Fii {
            o.push(
                "segment",
                i.segment.as_deref(),
                Some("cvm_fii"),
                source,
                tier,
                None,
                at,
                None,
                0,
            );
            o.push("cnpj", Some(&i.id), None, source, tier, None, at, None, 0);
            o.push(
                "fii_mandate",
                i.mandate.as_deref(),
                None,
                source,
                tier,
                None,
                at,
                None,
                0,
            );
        }
        if i.kind == "company_br" && matches!(class, C::StockBr | C::Bdr) {
            o.push(
                "issuer_cnpj",
                Some(&i.id),
                None,
                source,
                tier,
                None,
                at,
                None,
                0,
            );
            o.push(
                "cvm_code",
                i.cvm_code.as_deref(),
                None,
                source,
                tier,
                None,
                at,
                None,
                0,
            );
        }
        if us && matches!(class, C::StockUs | C::Reit | C::EtfUs) {
            let cik = i.id.strip_prefix("cik:").map(|c| c.trim_start_matches('0'));
            o.push("cik", cik, None, source, tier, None, at, None, 0);
        }
    }
    if let Some(s) = &security {
        let (source, tier) = research_source(&s.source);
        let at = security_fetched_at(conn, &s.ticker)?;
        let at = at.as_deref();
        o.push(
            "isin",
            s.isin.as_deref(),
            None,
            source,
            tier,
            None,
            at,
            None,
            0,
        );
        if matches!(class, C::StockBr | C::Fii | C::StockUs) {
            o.push(
                "share_kind",
                s.share_kind.as_deref(),
                None,
                source,
                tier,
                None,
                at,
                None,
                0,
            );
        }
        if class == C::StockBr {
            o.push(
                "listing_segment",
                s.listing_segment.as_deref(),
                None,
                source,
                tier,
                None,
                at,
                None,
                0,
            );
        }
    }
    // Fundamentus em cache (secundária): setor e subsetor das ações; segmento e gestão dos FIIs.
    if let Some(t) = a
        .ticker
        .as_deref()
        .filter(|_| matches!(class, C::StockBr | C::Bdr | C::Fii))
        && let Some(f) = fundamentus(conn, &t.to_uppercase())?
    {
        let at = Some(f.fetched_at.as_str());
        let url = Some(f.source_url.as_str());
        if f.fii && class == C::Fii {
            o.push(
                "segment",
                f.sector.as_deref(),
                Some("fundamentus"),
                "fundamentus",
                3,
                url,
                at,
                None,
                0,
            );
            o.push(
                "fii_management",
                f.subsector.as_deref(),
                None,
                "fundamentus",
                3,
                url,
                at,
                None,
                0,
            );
        } else if !f.fii && class != C::Fii {
            o.push(
                "sector",
                f.sector.as_deref(),
                Some("fundamentus"),
                "fundamentus",
                3,
                url,
                at,
                None,
                0,
            );
            o.push(
                "subsector",
                f.subsector.as_deref(),
                Some("fundamentus"),
                "fundamentus",
                3,
                url,
                at,
                None,
                0,
            );
        }
    }
    // Registro de fundos e classes da CVM: ETFs da B3, fundos e previdência.
    let efpc = class == C::Pension && crate::pension::is_efpc(conn, &a.id)?;
    if FUNDS.contains(&class) && !efpc {
        let reg = src.registry(conn)?;
        let own = a.cnpj.as_deref().map(cnpj_digits).filter(|c| c.len() == 14);
        // Sem CNPJ no ativo: o achado pelo nome (ETF) ou o provável (fundo) é só sugestão,
        // com 25 pontos a menos.
        let (cnpj, by_name) = match own {
            // CNPJ do ativo fora do registro da CVM (o do plano de previdência, não o do FIE):
            // o fundo achado pelo nome vira sugestão, como no ativo sem CNPJ.
            Some(c)
                if reg.registro_at.is_some()
                    && !reg.funds.contains_key(&c)
                    && reg.fund_guesses.contains_key(&a.id) =>
            {
                (reg.fund_guesses.get(&a.id).cloned(), true)
            }
            Some(c) => (Some(c), false),
            None => (
                reg.etfs
                    .get(&a.id)
                    .or_else(|| reg.fund_guesses.get(&a.id))
                    .cloned(),
                true,
            ),
        };
        if let Some(c) = cnpj
            && let Some(f) = reg.funds.get(&c)
        {
            let at = reg.registro_at.as_deref();
            let note = by_name.then_some("matched_by_name");
            let pen = if by_name { 25 } else { 0 };
            let s = "cvm_registro";
            if by_name {
                o.push("cnpj", Some(&f.cnpj), None, s, 1, None, at, note, pen);
            }
            o.push(
                "fund_class_cvm",
                f.classification.as_deref(),
                None,
                s,
                1,
                None,
                at,
                note,
                pen,
            );
            o.push(
                "fund_class_anbima",
                f.anbima.as_deref(),
                None,
                s,
                1,
                None,
                at,
                note,
                pen,
            );
            o.push(
                "benchmark",
                f.benchmark.as_deref(),
                None,
                s,
                1,
                None,
                at,
                note,
                pen,
            );
            o.push(
                "manager",
                f.manager.as_deref(),
                None,
                s,
                1,
                None,
                at,
                note,
                pen,
            );
            o.push(
                "administrator",
                f.administrator.as_deref(),
                None,
                s,
                1,
                None,
                at,
                note,
                pen,
            );
        }
    }
    // Fontes consultadas pela rede (B3, GLEIF, BrasilAPI), da última consulta.
    out.extend(
        super::lookup::stored(conn, &a.id)?
            .into_iter()
            .filter(|o| field_def(&o.field).is_ok()),
    );
    // Esquema principal dos brasileiros: a classificação da B3. O setor único do FCA passa a
    // conferência e concorda quando bate com qualquer um dos três níveis.
    let b3: Vec<String> = out
        .iter()
        .filter(|o| {
            o.source == "b3" && matches!(o.field.as_str(), "sector" | "subsector" | "segment")
        })
        .map(|o| o.value_norm.clone())
        .collect();
    if let Some(b3_sector) = out
        .iter()
        .find(|o| o.source == "b3" && o.field == "sector")
        .map(|o| o.value_norm.clone())
    {
        for o in out
            .iter_mut()
            .filter(|o| o.field == "sector" && o.source == "cvm_fca")
        {
            o.note = Some(CONFERENCE.into());
            if b3.contains(&o.value_norm) {
                o.value_norm = b3_sector.clone();
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------
// Confiança e conflitos

fn age_days(as_of: Option<&str>, today: chrono::NaiveDate) -> Option<i64> {
    let d = chrono::NaiveDate::parse_from_str(as_of?, "%Y-%m-%d").ok()?;
    Some((today - d).num_days())
}

/// Candidatos de um campo, do mais confiável ao menos: um por valor normalizado. Conflito =
/// os dois primeiros têm o mesmo nível.
fn rank(
    obs: &[&FieldObservation],
    def: &FieldDef,
    today: chrono::NaiveDate,
) -> (Vec<FieldCandidate>, bool) {
    let (conference, obs): (Vec<&FieldObservation>, Vec<&FieldObservation>) = obs
        .iter()
        .partition(|o| o.note.as_deref() == Some(CONFERENCE));
    let mut groups: BTreeMap<&str, Vec<&FieldObservation>> = BTreeMap::new();
    for o in &obs {
        groups.entry(o.value_norm.as_str()).or_default().push(o);
    }
    // Melhor leitura de cada grupo: menor nível, depois a mais nova.
    let mut best: Vec<(&FieldObservation, Vec<&FieldObservation>)> = groups
        .into_values()
        .map(|mut g| {
            // Mesmo nível: a B3 (esquema principal) à frente, depois a mais nova.
            g.sort_by(|a, b| {
                a.tier
                    .cmp(&b.tier)
                    .then((b.source == "b3").cmp(&(a.source == "b3")))
                    .then(b.as_of.cmp(&a.as_of))
            });
            (g[0], g)
        })
        .collect();
    best.sort_by(|a, b| a.0.tier.cmp(&b.0.tier).then(b.1.len().cmp(&a.1.len())));
    let top_tier = best.first().map(|b| b.0.tier);
    let conflict = best.iter().filter(|b| Some(b.0.tier) == top_tier).count() > 1;
    let mut out = vec![];
    for (lead, group) in &best {
        let mut agreeing: Vec<String> = group
            .iter()
            .chain(
                conference
                    .iter()
                    .filter(|o| o.value_norm == lead.value_norm),
            )
            .map(|o| o.source.clone())
            .filter(|s| *s != lead.source)
            // O Fundamentus repete a classificação da B3: não é uma segunda opinião dela.
            .filter(|s| !(lead.source == "b3" && s == "fundamentus"))
            .collect();
        agreeing.sort();
        agreeing.dedup();
        let mut c = tier_weight(lead.tier) + 5 * agreeing.len().min(2) as i64;
        // Outra fonte de nível igual ou maior discorda.
        if best
            .iter()
            .any(|(o, _)| o.value_norm != lead.value_norm && o.tier <= lead.tier)
        {
            c -= 25;
        }
        if age_days(lead.as_of.as_deref(), today).is_some_and(|d| d > def.stale_days) {
            c -= 10;
        }
        c -= lead.penalty;
        if lead.tier >= 4 {
            c = c.min(45);
        }
        let c = c.clamp(0, 100);
        out.push(FieldCandidate {
            value: lead.value.clone(),
            source: lead.source.clone(),
            tier: lead.tier,
            as_of: lead.as_of.clone(),
            source_ref: lead.source_ref.clone(),
            confidence: c,
            band: band(c),
            agreeing,
            note: lead.note.clone(),
        });
    }
    (out, conflict)
}

struct Meta {
    origin: FieldOrigin,
    source: Option<String>,
    as_of: Option<String>,
    confidence: Option<i64>,
    locked: bool,
}

fn metas(conn: &Connection, asset_id: &str) -> Result<BTreeMap<String, Meta>> {
    let mut stmt = conn.prepare(
        "SELECT field, origin, source_id, as_of, confidence, locked FROM asset_field_meta WHERE asset_id = ?1",
    )?;
    let rows = stmt
        .query_map([asset_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                Meta {
                    origin: FieldOrigin::parse(&r.get::<_, String>(1)?),
                    source: r.get(2)?,
                    as_of: r.get(3)?,
                    confidence: r.get(4)?,
                    locked: r.get::<_, i64>(5)? == 1,
                },
            ))
        })?
        .collect::<rusqlite::Result<BTreeMap<_, _>>>()?;
    Ok(rows)
}

fn attributes(conn: &Connection, asset_id: &str) -> Result<BTreeMap<String, String>> {
    let mut stmt = conn.prepare("SELECT field, value FROM asset_attribute WHERE asset_id = ?1")?;
    let rows = stmt
        .query_map([asset_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<BTreeMap<_, _>>>()?;
    Ok(rows)
}

/// Tipo do plano de previdência gravado (vgbl, pgbl, efpc).
fn pension_type(conn: &Connection, asset_id: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT plan_type FROM pension_plan WHERE asset_id = ?1",
            [asset_id],
            |r| r.get(0),
        )
        .optional()?)
}

fn column_value(a: &Asset, field: &str) -> Option<String> {
    let v = match field {
        "sector" => &a.sector,
        "segment" => &a.segment,
        "isin" => &a.isin,
        "cnpj" => &a.cnpj,
        "ticker" => &a.ticker,
        _ => return None,
    };
    clean(v.as_deref())
}

/// Atributos gravados de um ativo (campo → valor), para o Cadastro.
pub fn attributes_of(conn: &Connection, asset_id: &str) -> Result<BTreeMap<String, String>> {
    attributes(conn, asset_id)
}

/// Efeito de trocar o setor de uma ação BR no score: o segmento dos pares muda.
fn sector_effect(a: &Asset, old: Option<&str>, new: &str) -> Option<String> {
    if a.class != C::StockBr {
        return None;
    }
    let m = crate::score::config::methodology();
    let before = crate::score::taxonomy::equity_segment(m, old, None);
    let after = crate::score::taxonomy::equity_segment(m, Some(new), None);
    if before == after {
        return None;
    }
    let label = m
        .equity_br
        .segments
        .get(&after)
        .map(|s| s.label.clone())
        .unwrap_or(after);
    Some(t_args(
        "nucleo-ativos-dados-efeito-score",
        &args(&[("segmento", label.into())]),
    ))
}

/// Estado de cada campo do catálogo para um ativo.
fn statuses(
    conn: &Connection,
    a: &Asset,
    obs: &[FieldObservation],
    today: chrono::NaiveDate,
) -> Result<Vec<FieldStatus>> {
    let metas = metas(conn, &a.id)?;
    let attrs = attributes(conn, &a.id)?;
    // Previdência fechada (EFPC) não é fundo da CVM: sem classe CVM nem ANBIMA a cobrar.
    let efpc = a.class == AssetClass::Pension && crate::pension::is_efpc(conn, &a.id)?;
    let mut out = vec![];
    for def in FIELDS {
        let mine: Vec<&FieldObservation> = obs.iter().filter(|o| o.field == def.key).collect();
        let current = if def.key == "pension_type" {
            pension_type(conn, &a.id)?
        } else if def.column {
            column_value(a, def.key)
        } else {
            attrs.get(def.key).cloned()
        };
        let applies = def.classes.contains(&a.class)
            && !(efpc && matches!(def.key, "fund_class_cvm" | "fund_class_anbima"));
        if !applies && mine.is_empty() && current.is_none() {
            continue;
        }
        let meta = metas.get(def.key);
        let locked = meta.is_some_and(|m| m.locked);
        // Sem registro de origem e preenchido = anterior a esta função (protegido).
        let origin = match (meta, &current) {
            (Some(m), _) => Some(m.origin),
            (None, Some(_)) => Some(FieldOrigin::Legacy),
            (None, None) => None,
        };
        let (ranked, conflict) = rank(&mine, def, today);
        let cur_norm = current.as_deref().map(|c| normalize(def.key, c));
        let agrees =
            |c: &FieldCandidate| cur_norm.as_deref() == Some(normalize(def.key, &c.value).as_str());
        let (state, candidate, options) = if locked {
            (FieldState::Ok, None, vec![])
        } else if ranked.is_empty() {
            let st = if current.is_some() {
                FieldState::Ok
            } else {
                FieldState::Missing
            };
            (st, None, vec![])
        } else if current.is_some() && ranked.iter().any(|c| agrees(c) && c.tier == ranked[0].tier)
        {
            // O valor atual é o de uma das fontes de maior nível: nada a fazer.
            (FieldState::Ok, None, vec![])
        } else if conflict {
            let top = ranked[0].tier;
            let opts: Vec<FieldCandidate> = ranked.into_iter().filter(|c| c.tier == top).collect();
            (FieldState::Conflict, None, opts)
        } else {
            let c = ranked.into_iter().next().expect("não vazio");
            let st = match origin {
                None => FieldState::Fill,
                // Ticker renomeado: a fonte diz que o código mudou (não é o usuário que
                // digitou outro): troca sensível, item a item.
                Some(_) if def.key == "ticker" => FieldState::Update,
                Some(o) if o.protected() => FieldState::Diverge,
                Some(_) => FieldState::Update,
            };
            (st, Some(c), vec![])
        };
        let effect = match (&candidate, def.key) {
            (Some(c), "sector") => sector_effect(a, current.as_deref(), &c.value),
            _ => None,
        };
        out.push(FieldStatus {
            field: def.key.into(),
            current,
            origin,
            locked,
            current_source: meta.and_then(|m| m.source.clone()),
            current_as_of: meta.and_then(|m| m.as_of.clone()),
            current_confidence: meta.and_then(|m| m.confidence),
            state,
            candidate,
            options,
            effect,
        });
    }
    Ok(out)
}

fn asset_data(
    conn: &Connection,
    a: &Asset,
    src: &mut Sources,
    today: chrono::NaiveDate,
) -> Result<(AssetData, Vec<FieldObservation>)> {
    let obs = observations(conn, a, src)?;
    let fields = statuses(conn, a, &obs, today)?;
    Ok((
        AssetData {
            asset_id: a.id.clone(),
            ticker: a.ticker.clone(),
            name: a.name.clone(),
            class: a.class,
            fields,
        },
        obs,
    ))
}

/// Ativos e campos faltando, sugestões, conflitos e divergências (`assets.data_gaps`).
pub fn data_gaps(conn: &Connection, args: &DataGapsArgs) -> Result<DataGaps> {
    let today = crate::time::today_local();
    let list = match &args.asset_ids {
        Some(ids) => ids
            .iter()
            .map(|id| super::get(conn, id))
            .collect::<Result<Vec<_>>>()?,
        None => super::list(
            conn,
            &super::AssetFilter {
                held_only: args.held_only,
                ..Default::default()
            },
        )?,
    };
    let mut src = Sources::default();
    let mut sum = DataGapsSummary::default();
    let mut assets = vec![];
    for a in &list {
        let (mut d, _) = asset_data(conn, a, &mut src, today)?;
        sum.assets += 1;
        for f in &d.fields {
            let applicable = field_def(&f.field).is_ok_and(|def| def.classes.contains(&a.class));
            if f.current.is_none() && applicable && f.field == "sector" {
                sum.missing_sector += 1;
            }
            if f.current.is_none() && applicable && f.field == "isin" {
                sum.missing_isin += 1;
            }
            match f.state {
                FieldState::Fill => {
                    sum.fill += 1;
                    if f.candidate
                        .as_ref()
                        .is_some_and(|c| c.band == FieldBand::High)
                    {
                        sum.fill_high += 1;
                    }
                }
                FieldState::Update => sum.updates += 1,
                FieldState::Conflict => sum.conflicts += 1,
                FieldState::Diverge => sum.divergences += 1,
                FieldState::Ok | FieldState::Missing => {}
            }
        }
        if !args.include_ok.unwrap_or(false) {
            d.fields
                .retain(|f| !matches!(f.state, FieldState::Ok | FieldState::Missing));
        }
        if args.include_ok.unwrap_or(false) || !d.fields.is_empty() {
            assets.push(d);
        }
    }
    Ok(DataGaps {
        summary: sum,
        assets,
    })
}

/// Origem e leituras de cada campo de um ativo (`assets.field_sources`).
pub fn field_sources(conn: &Connection, asset_id: &str) -> Result<AssetFieldSources> {
    let a = super::get(conn, asset_id)?;
    let today = crate::time::today_local();
    let (d, obs) = asset_data(conn, &a, &mut Sources::default(), today)?;
    let mut stmt = conn.prepare(
        "SELECT value, source_id, tier, source_ref, published_at, observed_at FROM asset_observation
          WHERE asset_id = ?1 AND field = ?2 ORDER BY observed_at DESC LIMIT 20",
    )?;
    let mut fields = vec![];
    for status in d.fields {
        let history = stmt
            .query_map(params![asset_id, status.field], |r| {
                Ok(StoredFieldObservation {
                    value: r.get(0)?,
                    source: r.get(1)?,
                    tier: r.get::<_, i64>(2)? as u8,
                    source_ref: r.get(3)?,
                    published_at: r.get(4)?,
                    observed_at: r.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let observations = obs
            .iter()
            .filter(|o| o.field == status.field)
            .cloned()
            .collect();
        fields.push(FieldSources {
            status,
            observations,
            history,
        });
    }
    Ok(AssetFieldSources {
        asset_id: asset_id.into(),
        fields,
    })
}

// ---------------------------------------------------------------------------------------
// Escritas

fn upsert_meta(
    tx: &Transaction<'_>,
    asset_id: &str,
    field: &str,
    origin: FieldOrigin,
    c: Option<&FieldCandidate>,
    now: &str,
) -> Result<()> {
    tx.execute(
        "INSERT INTO asset_field_meta (asset_id, field, origin, source_id, source_ref, as_of, confidence, locked, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)
         ON CONFLICT (asset_id, field) DO UPDATE SET origin = excluded.origin, source_id = excluded.source_id,
             source_ref = excluded.source_ref, as_of = excluded.as_of, confidence = excluded.confidence,
             updated_at = excluded.updated_at",
        params![
            asset_id,
            field,
            origin.as_str(),
            c.map(|c| c.source.clone()),
            c.and_then(|c| c.source_ref.clone()),
            c.and_then(|c| c.as_of.clone()),
            c.map(|c| c.confidence),
            now
        ],
    )?;
    Ok(())
}

fn label_of(a: &Asset) -> String {
    a.ticker.clone().unwrap_or_else(|| a.name.clone())
}

/// Grava os campos escolhidos com proveniência (`assets.enrich_apply`). Cada valor tem de
/// estar numa leitura atual do ativo; campo travado é recusado; valor preenchido só é trocado
/// com `replace` (salvo o de uma sugestão anterior).
pub fn apply(tx: &Transaction<'_>, items: &[EnrichItem]) -> Result<EnrichApplied> {
    if items.is_empty() {
        return Err(Error::invalid(t("nucleo-ativos-dados-erro-vazio")));
    }
    let today = crate::time::today_local();
    let now = now_utc();
    let mut src = Sources::default();
    let mut by_asset: BTreeMap<&str, Vec<&EnrichItem>> = BTreeMap::new();
    for i in items {
        field_def(&i.field)?;
        by_asset.entry(i.asset_id.as_str()).or_default().push(i);
    }
    let mut fields = 0;
    for (asset_id, list) in &by_asset {
        let a = super::get(tx, asset_id)?;
        let obs = observations(tx, &a, &mut src)?;
        let metas = metas(tx, asset_id)?;
        let attrs = attributes(tx, asset_id)?;
        let mut seen = std::collections::BTreeSet::new();
        for item in list {
            let def = field_def(&item.field)?;
            if !seen.insert(def.key) {
                return Err(Error::invalid(t_args(
                    "nucleo-ativos-dados-erro-repetido",
                    &args(&[("campo", def.key.into()), ("ativo", label_of(&a).into())]),
                )));
            }
            let meta = metas.get(def.key);
            if meta.is_some_and(|m| m.locked) {
                return Err(Error::conflict(t_args(
                    "nucleo-ativos-dados-erro-travado",
                    &args(&[("campo", def.key.into()), ("ativo", label_of(&a).into())]),
                )));
            }
            let want = normalize(def.key, &item.value);
            let mine: Vec<&FieldObservation> = obs.iter().filter(|o| o.field == def.key).collect();
            let (ranked, _) = rank(&mine, def, today);
            let Some(c) = ranked.iter().find(|c| normalize(def.key, &c.value) == want) else {
                return Err(Error::invalid(t_args(
                    "nucleo-ativos-dados-erro-sem-fonte",
                    &args(&[
                        ("campo", def.key.into()),
                        ("ativo", label_of(&a).into()),
                        ("valor", item.value.clone().into()),
                    ]),
                )));
            };
            let current = if def.key == "pension_type" {
                pension_type(tx, asset_id)?
            } else if def.column {
                column_value(&a, def.key)
            } else {
                attrs.get(def.key).cloned()
            };
            let origin = meta
                .map(|m| m.origin)
                .or(current.as_ref().map(|_| FieldOrigin::Legacy));
            let same = current.as_deref().map(|v| normalize(def.key, v)) == Some(want.clone());
            if current.is_some()
                && !same
                && !item.replace.unwrap_or(false)
                && origin.is_some_and(FieldOrigin::protected)
            {
                return Err(Error::conflict(t_args(
                    "nucleo-ativos-dados-erro-preenchido",
                    &args(&[("campo", def.key.into()), ("ativo", label_of(&a).into())]),
                )));
            }
            if def.key == "pension_type" {
                // O tipo mora no plano de previdência (cria o plano se ainda não houver).
                tx.execute(
                    "INSERT INTO pension_plan (asset_id, plan_type, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)
                     ON CONFLICT (asset_id) DO UPDATE SET plan_type = excluded.plan_type, updated_at = excluded.updated_at",
                    params![asset_id, c.value, now],
                )?;
            } else if def.column {
                tx.execute(
                    &format!(
                        "UPDATE asset SET {} = ?2, updated_at = ?3 WHERE id = ?1",
                        def.key
                    ),
                    params![asset_id, c.value, now],
                )?;
            } else {
                let scheme = mine
                    .iter()
                    .find(|o| o.source == c.source && o.value == c.value)
                    .and_then(|o| o.scheme.clone());
                tx.execute(
                    "INSERT INTO asset_attribute (asset_id, field, value, scheme, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT (asset_id, field) DO UPDATE SET value = excluded.value, scheme = excluded.scheme,
                         updated_at = excluded.updated_at",
                    params![asset_id, def.key, c.value, scheme, now],
                )?;
            }
            upsert_meta(
                tx,
                asset_id,
                def.key,
                FieldOrigin::Enrichment,
                Some(c),
                &now,
            )?;
            // As leituras usadas ficam registradas ("de onde veio").
            for o in mine {
                tx.execute(
                    "INSERT INTO asset_observation (id, asset_id, field, value, value_norm, scheme, source_id, tier,
                                                    source_ref, published_at, observed_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        crate::ids::new_id(),
                        asset_id,
                        o.field,
                        o.value,
                        o.value_norm,
                        o.scheme,
                        o.source,
                        o.tier,
                        o.source_ref,
                        o.as_of,
                        now
                    ],
                )?;
            }
            fields += 1;
        }
    }
    Ok(EnrichApplied {
        fields,
        assets: by_asset.len() as i64,
    })
}

/// Trava ou destrava um campo ("não sugerir mudanças neste campo").
pub fn set_lock(tx: &Transaction<'_>, a: &FieldLockArgs) -> Result<()> {
    let def = field_def(&a.field)?;
    let asset = super::get(tx, &a.asset_id)?;
    let now = now_utc();
    let current = if def.column {
        column_value(&asset, def.key)
    } else {
        attributes(tx, &a.asset_id)?.get(def.key).cloned()
    };
    let origin = if current.is_some() {
        FieldOrigin::Legacy
    } else {
        FieldOrigin::User
    };
    tx.execute(
        "INSERT INTO asset_field_meta (asset_id, field, origin, locked, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (asset_id, field) DO UPDATE SET locked = excluded.locked, updated_at = excluded.updated_at",
        params![a.asset_id, def.key, origin.as_str(), a.locked as i64, now],
    )?;
    Ok(())
}

/// O usuário editou um campo: a origem passa a ser `user` (apagar o valor apaga a origem,
/// salvo se o campo estiver travado).
pub fn mark_user(
    tx: &Transaction<'_>,
    asset_id: &str,
    field: &str,
    value: Option<&str>,
) -> Result<()> {
    let now = now_utc();
    if clean(value).is_some() {
        upsert_meta(tx, asset_id, field, FieldOrigin::User, None, &now)?;
    } else {
        tx.execute(
            "DELETE FROM asset_field_meta WHERE asset_id = ?1 AND field = ?2 AND locked = 0",
            params![asset_id, field],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
