//! Metodologia versionada do score (pesos, limiares e segmentos em JSON, embutidos no
//! binário a partir de `methodology/v<versão>/`). Nada de peso ou limiar fica no código: as
//! funções de cálculo leem daqui.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};

/// Versão da metodologia em uso (SemVer, ver methodology/CHANGELOG.md).
pub const METHODOLOGY_VERSION: &str = "0.1.0";

const WEIGHTS: &str = include_str!("../../../../methodology/v0.1.0/weights.json");
const THRESHOLDS: &str = include_str!("../../../../methodology/v0.1.0/thresholds.json");
const SEGMENTS: &str = include_str!("../../../../methodology/v0.1.0/segments.json");
/// Extensões por classe (cada uma com dimensões, métricas, perfis, flags e segmentos).
const CLASS_FILES: &[(&str, &str)] = &[
    (
        "fixed_income",
        include_str!("../../../../methodology/v0.1.0/classes/fixed_income.json"),
    ),
    (
        "funds",
        include_str!("../../../../methodology/v0.1.0/classes/funds.json"),
    ),
    (
        "crypto",
        include_str!("../../../../methodology/v0.1.0/classes/crypto.json"),
    ),
    (
        "intl",
        include_str!("../../../../methodology/v0.1.0/classes/intl.json"),
    ),
];

/// Direção de uma métrica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ScoreDirection")]
pub enum Direction {
    /// Maior é melhor.
    Higher,
    /// Menor é melhor.
    Lower,
    /// Faixa ótima: só a curva absoluta (sem percentil).
    Range,
}

/// Unidade de exibição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ScoreUnit")]
pub enum Unit {
    Percent,
    Multiple,
    Score,
    Count,
    Days,
    Years,
    Money,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricDef {
    pub label: String,
    pub unit: Unit,
    pub direction: Direction,
    pub formula: String,
    /// Pontos (x, nota) da âncora absoluta, em ordem crescente de x.
    pub anchor: Vec<(f64, f64)>,
    /// Métrica-chave: nível (12 meses) + média de 5 anos + tendência (§7.4).
    #[serde(default)]
    pub key: bool,
    /// Inclinação anual que vale nota 0/100 na âncora da tendência (±).
    #[serde(default)]
    pub trend_scale: Option<f64>,
    /// Só âncora absoluta (sem percentil contra os pares).
    #[serde(default)]
    pub absolute_only: bool,
    /// Sem fonte de dados: o motivo (a métrica aparece como indisponível).
    #[serde(default)]
    pub unavailable: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Normalization {
    pub peer_min: usize,
    pub winsor_low: f64,
    pub winsor_high: f64,
    pub relative_weight: f64,
    pub absolute_weight: f64,
    pub level_weight: f64,
    pub average_weight: f64,
    pub trend_weight: f64,
    pub history_years: u32,
    pub tax_rate_br: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Missing {
    pub dimension_min_coverage: f64,
    pub impute_factor: f64,
    pub max_imputed_share: f64,
    pub stale_days: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Confidence {
    pub peer_level_factor: BTreeMap<String, f64>,
    pub stale_factor: f64,
    pub low_liquidity_factor: f64,
    pub reconcile_factor: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Reconciliation {
    pub tolerance: f64,
    pub confidence_tolerance: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BankValuation {
    pub cost_of_equity: f64,
    pub growth: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BandDef {
    pub key: String,
    pub label: String,
    pub min: f64,
    pub meaning: String,
}

impl BandDef {
    /// Nome da faixa no idioma da instalação (`score-faixa-<chave>`); faixa sem chave no
    /// catálogo: o nome da metodologia.
    pub fn label_text(&self) -> String {
        let key = format!("score-faixa-{}", self.key);
        if crate::i18n::has_key(crate::i18n::Locale::PtBr, &key) {
            crate::i18n::t(&key)
        } else {
            self.label.clone()
        }
    }

    /// O que a faixa quer dizer, no idioma da instalação (`nucleo-score-faixa-<chave>`).
    pub fn meaning_text(&self) -> String {
        let key = format!("nucleo-score-faixa-{}", self.key);
        if crate::i18n::has_key(crate::i18n::Locale::PtBr, &key) {
            crate::i18n::t(&key)
        } else {
            self.meaning.clone()
        }
    }
}

/// Gravidade de uma red flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ScoreFlagSeverity")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlagDef {
    pub label: String,
    #[serde(default)]
    pub threshold: Option<f64>,
    #[serde(default)]
    pub cap: Option<f64>,
    #[serde(default)]
    pub penalty: Option<f64>,
    #[serde(default)]
    pub min_tenure_years: Option<f64>,
    pub severity: Severity,
}

/// Tipo de evento qualitativo (§8.2).
#[derive(Debug, Clone, Deserialize)]
pub struct EventKindDef {
    pub label: String,
    #[serde(default)]
    pub cap: Option<f64>,
    #[serde(default)]
    pub penalty: Option<f64>,
    pub severity: Severity,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsConfig {
    pub half_life_quarters: f64,
    pub kinds: BTreeMap<String, EventKindDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Alerts {
    pub drop_points: f64,
    /// Faixa a partir da qual uma piora de faixa vira alerta (`atencao`: sair de Robusto ou
    /// Saudável para Atenção ou Risco; Robusto → Saudável não alerta).
    #[serde(default)]
    pub band_floor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ThresholdsFile {
    methodology_version: String,
    metrics: BTreeMap<String, MetricDef>,
    normalization: Normalization,
    missing: Missing,
    confidence: Confidence,
    reconciliation: Reconciliation,
    bank_valuation: BankValuation,
    bands: Vec<BandDef>,
    flags: BTreeMap<String, FlagDef>,
    alerts: Alerts,
    events: EventsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DimensionWeights {
    pub weight: f64,
    /// Métrica → peso relativo dentro da dimensão (ordem do arquivo).
    pub metrics: indexmap_like::OrderedMap,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub label: String,
    #[serde(default)]
    pub attractiveness: Vec<String>,
    #[serde(default)]
    pub critical: Vec<String>,
    #[serde(default)]
    pub anchors: BTreeMap<String, Vec<(f64, f64)>>,
    #[serde(default)]
    pub dimension_labels: BTreeMap<String, String>,
    pub dimensions: indexmap_like::OrderedDims,
}

#[derive(Debug, Clone, Deserialize)]
struct WeightsFile {
    methodology_version: String,
    dimensions: BTreeMap<String, String>,
    profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SegmentDef {
    pub label: String,
    pub profile: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SectorRule {
    pub segment: String,
    pub sectors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EquitySegments {
    pub default_segment: String,
    pub layout_segments: BTreeMap<String, String>,
    pub segments: BTreeMap<String, SegmentDef>,
    pub sector_map: Vec<SectorRule>,
    /// Setores das holdings com nome abreviado → setor das operacionais.
    #[serde(default)]
    pub sector_aliases: BTreeMap<String, String>,
    pub listing_scores: BTreeMap<String, f64>,
    /// Segmentos que só valem com o leiaute certo das demonstrações (bancos: indicadores do
    /// IF.data pelo CNPJ do próprio banco) → leiaute exigido.
    #[serde(default)]
    pub layout_required: BTreeMap<String, String>,
    /// Segmento das holdings cujo setor herdado exige outro leiaute (holding de banco que
    /// publica demonstrações de empresa comum, como a Itaúsa).
    #[serde(default)]
    pub holding_segment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReitComposition {
    pub papel_min_cri_share: f64,
    pub fof_min_fii_share: f64,
    pub tijolo_min_real_estate_share: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReitSegments {
    pub default_segment: String,
    pub composition: ReitComposition,
    pub segments: BTreeMap<String, SegmentDef>,
    pub noisy_subsegments: Vec<String>,
}

/// Segmentos de uma classe das extensões.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ClassSegments {
    pub default_segment: String,
    pub segments: BTreeMap<String, SegmentDef>,
}

#[derive(Debug, Clone, Deserialize)]
struct ClassFile {
    methodology_version: String,
    #[serde(default)]
    dimensions: BTreeMap<String, String>,
    #[serde(default)]
    metrics: BTreeMap<String, MetricDef>,
    #[serde(default)]
    profiles: BTreeMap<String, Profile>,
    #[serde(default)]
    flags: BTreeMap<String, FlagDef>,
    #[serde(default)]
    segments: BTreeMap<String, ClassSegments>,
    #[serde(default)]
    aliases: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PeerLevelDef {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SegmentsFile {
    methodology_version: String,
    classes: BTreeMap<String, String>,
    equity_br: EquitySegments,
    reit: ReitSegments,
    peer_levels: Vec<PeerLevelDef>,
}

/// Metodologia carregada e validada.
#[derive(Debug, Clone)]
pub struct Methodology {
    pub version: String,
    pub dimensions: BTreeMap<String, String>,
    pub profiles: BTreeMap<String, Profile>,
    pub metrics: BTreeMap<String, MetricDef>,
    pub normalization: Normalization,
    pub missing: Missing,
    pub confidence: Confidence,
    pub reconciliation: Reconciliation,
    pub bank_valuation: BankValuation,
    pub bands: Vec<BandDef>,
    pub flags: BTreeMap<String, FlagDef>,
    pub alerts: Alerts,
    pub events: EventsConfig,
    pub classes: BTreeMap<String, String>,
    pub equity_br: EquitySegments,
    pub reit: ReitSegments,
    pub peer_levels: Vec<PeerLevelDef>,
    /// Segmentos das classes das extensões (renda fixa, fundos, cripto, exterior…).
    pub class_segments: BTreeMap<String, ClassSegments>,
    /// Mapeamentos das extensões (tabela → chave → valor; ex.: SIC → segmento, BDR → ticker).
    pub aliases: BTreeMap<String, BTreeMap<String, String>>,
}

/// Mapas que preservam a ordem do arquivo (a ordem de exibição das dimensões e métricas).
pub mod indexmap_like {
    use serde::Deserialize;
    use serde::de::{Deserializer, MapAccess, Visitor};

    use super::DimensionWeights;

    #[derive(Debug, Clone, Default)]
    pub struct OrderedMap(pub Vec<(String, f64)>);

    #[derive(Debug, Clone, Default)]
    pub struct OrderedDims(pub Vec<(String, DimensionWeights)>);

    struct V<T>(std::marker::PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
        type Value = Vec<(String, T)>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("um objeto")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut m: A) -> Result<Self::Value, A::Error> {
            let mut out = vec![];
            while let Some((k, v)) = m.next_entry::<String, T>()? {
                out.push((k, v));
            }
            Ok(out)
        }
    }

    impl<'de> Deserialize<'de> for OrderedMap {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            Ok(OrderedMap(d.deserialize_map(V::<f64>(Default::default()))?))
        }
    }

    impl<'de> Deserialize<'de> for OrderedDims {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            Ok(OrderedDims(d.deserialize_map(V::<DimensionWeights>(
                Default::default(),
            ))?))
        }
    }
}

impl Methodology {
    fn parse() -> Result<Self> {
        let bad = |what: &str, e: serde_json::Error| {
            Error::invalid(format!("metodologia do score ({what}): {e}"))
        };
        let w: WeightsFile = serde_json::from_str(WEIGHTS).map_err(|e| bad("weights", e))?;
        let t: ThresholdsFile =
            serde_json::from_str(THRESHOLDS).map_err(|e| bad("thresholds", e))?;
        let s: SegmentsFile = serde_json::from_str(SEGMENTS).map_err(|e| bad("segments", e))?;
        let m = Methodology {
            version: t.methodology_version.clone(),
            dimensions: w.dimensions,
            profiles: w.profiles,
            metrics: t.metrics,
            normalization: t.normalization,
            missing: t.missing,
            confidence: t.confidence,
            reconciliation: t.reconciliation,
            bank_valuation: t.bank_valuation,
            bands: t.bands,
            flags: t.flags,
            alerts: t.alerts,
            events: t.events,
            classes: s.classes,
            equity_br: s.equity_br,
            reit: s.reit,
            peer_levels: s.peer_levels,
            class_segments: BTreeMap::new(),
            aliases: BTreeMap::new(),
        };
        let mut m = m;
        for (name, text) in CLASS_FILES {
            let f: ClassFile = serde_json::from_str(text).map_err(|e| bad(name, e))?;
            if f.methodology_version != m.version {
                return Err(Error::invalid(format!(
                    "metodologia do score: {name} é da versão {}",
                    f.methodology_version
                )));
            }
            let dup = |what: &str, k: &String| {
                Error::invalid(format!(
                    "metodologia do score: {what} {k} repetida em {name}"
                ))
            };
            for (k, v) in f.dimensions {
                if m.dimensions.insert(k.clone(), v).is_some() {
                    return Err(dup("dimensão", &k));
                }
            }
            for (k, v) in f.metrics {
                if m.metrics.insert(k.clone(), v).is_some() {
                    return Err(dup("métrica", &k));
                }
            }
            for (k, v) in f.profiles {
                if m.profiles.insert(k.clone(), v).is_some() {
                    return Err(dup("perfil", &k));
                }
            }
            for (k, v) in f.flags {
                if m.flags.insert(k.clone(), v).is_some() {
                    return Err(dup("flag", &k));
                }
            }
            for (k, v) in f.segments {
                if m.class_segments.insert(k.clone(), v).is_some() {
                    return Err(dup("classe", &k));
                }
            }
            for (k, v) in f.aliases {
                if m.aliases.insert(k.clone(), v).is_some() {
                    return Err(dup("tabela de mapeamento", &k));
                }
            }
        }
        for v in [&w.methodology_version, &s.methodology_version] {
            if *v != m.version {
                return Err(Error::invalid(format!(
                    "metodologia do score: versões diferentes nos arquivos ({v} × {})",
                    m.version
                )));
            }
        }
        m.validate()?;
        Ok(m)
    }

    /// Confere a consistência dos arquivos (rodado nos testes e ao carregar).
    pub fn validate(&self) -> Result<()> {
        let err = |s: String| Err(Error::invalid(format!("metodologia do score: {s}")));
        for (k, d) in &self.metrics {
            if d.anchor.len() < 2 {
                return err(format!("âncora de {k} precisa de 2 pontos ou mais"));
            }
            if d.anchor.windows(2).any(|w| w[1].0 <= w[0].0) {
                return err(format!("âncora de {k} fora de ordem"));
            }
            if d.anchor.iter().any(|p| !(0.0..=100.0).contains(&p.1)) {
                return err(format!("âncora de {k} com nota fora de 0–100"));
            }
            if d.key && d.trend_scale.is_none() {
                return err(format!("métrica-chave {k} sem trend_scale"));
            }
        }
        for (pk, p) in &self.profiles {
            let total: f64 = p.dimensions.0.iter().map(|(_, d)| d.weight).sum();
            if (total - 100.0).abs() > 1e-9 {
                return err(format!("pesos do perfil {pk} somam {total}, não 100"));
            }
            for (dk, d) in &p.dimensions.0 {
                if !self.dimensions.contains_key(dk) {
                    return err(format!("dimensão desconhecida {dk} em {pk}"));
                }
                if d.weight < 0.0 || d.weight > 35.0 {
                    return err(format!("peso de {dk} em {pk} fora de 0–35"));
                }
                if d.metrics.0.is_empty() {
                    return err(format!("dimensão {dk} de {pk} sem métricas"));
                }
                for (mk, mw) in &d.metrics.0 {
                    if !self.metrics.contains_key(mk) {
                        return err(format!("métrica desconhecida {mk} em {pk}/{dk}"));
                    }
                    if *mw <= 0.0 {
                        return err(format!("peso de {mk} em {pk}/{dk} deve ser positivo"));
                    }
                }
            }
            for c in p.critical.iter().chain(p.anchors.keys()) {
                if !p
                    .dimensions
                    .0
                    .iter()
                    .any(|(_, d)| d.metrics.0.iter().any(|(m, _)| m == c))
                {
                    return err(format!("{c} citada em {pk} mas fora das dimensões"));
                }
            }
            for c in &p.critical {
                if self.metrics[c].unavailable.is_some() {
                    return err(format!("{c} é crítica em {pk} mas não tem fonte"));
                }
            }
            for a in &p.attractiveness {
                if !self.dimensions.contains_key(a) {
                    return err(format!("dimensão de atratividade desconhecida {a} em {pk}"));
                }
            }
        }
        for seg in self
            .equity_br
            .segments
            .values()
            .chain(self.reit.segments.values())
        {
            if !self.profiles.contains_key(&seg.profile) {
                return err(format!("perfil inexistente: {}", seg.profile));
            }
        }
        for (class, cs) in &self.class_segments {
            if !cs.segments.contains_key(&cs.default_segment) {
                return err(format!("segmento padrão de {class} não existe"));
            }
            for seg in cs.segments.values() {
                if !self.profiles.contains_key(&seg.profile) {
                    return err(format!("perfil inexistente: {}", seg.profile));
                }
            }
        }
        if let Some(f) = &self.alerts.band_floor
            && !self.bands.iter().any(|b| &b.key == f)
        {
            return err(format!("faixa inexistente em alerts.band_floor: {f}"));
        }
        for seg in self
            .equity_br
            .layout_required
            .keys()
            .chain(self.equity_br.holding_segment.iter())
        {
            if !self.equity_br.segments.contains_key(seg) {
                return err(format!("segmento inexistente: {seg}"));
            }
        }
        for r in &self.equity_br.sector_map {
            if !self.equity_br.segments.contains_key(&r.segment) {
                return err(format!(
                    "segmento inexistente no mapa de setores: {}",
                    r.segment
                ));
            }
        }
        if self.bands.windows(2).any(|w| w[1].min >= w[0].min) || self.bands.is_empty() {
            return err("faixas devem vir da maior para a menor".into());
        }
        let n = &self.normalization;
        if (n.relative_weight + n.absolute_weight - 1.0).abs() > 1e-9
            || (n.level_weight + n.average_weight + n.trend_weight - 1.0).abs() > 1e-9
        {
            return err("coeficientes da normalização devem somar 1".into());
        }
        Ok(())
    }

    /// Segmentos de uma classe (ações, FIIs e as extensões): chave → (rótulo, perfil).
    pub fn segments_of(&self, class: &str) -> Vec<(String, &SegmentDef)> {
        let map = match class {
            "equity_br" => Some(&self.equity_br.segments),
            "reit" => Some(&self.reit.segments),
            c => self.class_segments.get(c).map(|x| &x.segments),
        };
        map.map(|m| m.iter().map(|(k, v)| (k.clone(), v)).collect())
            .unwrap_or_default()
    }

    pub fn metric(&self, key: &str) -> &MetricDef {
        &self.metrics[key]
    }

    /// Âncora da métrica no perfil (a do perfil vence a do catálogo).
    pub fn anchor<'a>(&'a self, profile: &'a Profile, key: &str) -> &'a [(f64, f64)] {
        profile
            .anchors
            .get(key)
            .map(Vec::as_slice)
            .unwrap_or(&self.metrics[key].anchor)
    }

    /// Nome da dimensão no idioma da instalação; o nome próprio de um perfil (bancos, renda
    /// fixa) é o da metodologia.
    pub fn dimension_label(&self, profile: &Profile, key: &str) -> String {
        if let Some(own) = profile.dimension_labels.get(key) {
            return own.clone();
        }
        match self.dimensions.get(key) {
            Some(label) => text("dimensao", key, label),
            None => key.to_string(),
        }
    }

    /// Nome da métrica no idioma da instalação.
    pub fn metric_label(&self, key: &str) -> String {
        text("metrica", key, &self.metrics[key].label)
    }

    /// Fórmula da métrica (como é calculada) no idioma da instalação.
    pub fn metric_formula(&self, key: &str) -> String {
        text("formula", key, &self.metrics[key].formula)
    }

    /// Por que a métrica não tem dado (sem fonte estruturada), no idioma da instalação.
    pub fn metric_unavailable(&self, key: &str) -> Option<String> {
        self.metrics[key]
            .unavailable
            .as_deref()
            .map(|why| text("indisponivel", key, why))
    }

    /// Nome do perfil (`classe/segmento`) no idioma da instalação.
    pub fn profile_label(&self, key: &str) -> String {
        match self.profiles.get(key) {
            Some(p) => text("perfil", &key.replace('/', "-"), &p.label),
            None => String::new(),
        }
    }

    /// Nome do segmento de uma classe no idioma da instalação.
    pub fn segment_label(&self, class: &str, key: &str, label: &str) -> String {
        text("segmento", &format!("{class}-{key}"), label)
    }

    /// Nome da classe no idioma da instalação.
    pub fn class_label(&self, class: &str) -> String {
        match self.classes.get(class) {
            Some(label) => text("classe", class, label),
            None => class.to_string(),
        }
    }

    /// Nome da red flag no idioma da instalação.
    pub fn flag_label(&self, key: &str) -> String {
        match self.flags.get(key) {
            Some(f) => text("flag", key, &f.label),
            None => key.to_string(),
        }
    }

    /// Nome do tipo de evento no idioma da instalação.
    pub fn event_label(&self, key: &str) -> String {
        match self.events.kinds.get(key) {
            Some(k) => text("evento", key, &k.label),
            None => key.to_string(),
        }
    }

    /// Faixa de um score (as faixas vêm da maior para a menor).
    pub fn band(&self, score: f64) -> &BandDef {
        self.bands
            .iter()
            .find(|b| score.round() >= b.min)
            .unwrap_or_else(|| self.bands.last().expect("validado: há faixas"))
    }
}

/// Texto da metodologia no idioma da instalação (`nucleo-metodologia-<tipo>-<id>`). O catálogo
/// traz o que vale fora do Brasil (ações no exterior, ETFs listados fora e criptoativos); o que
/// só existe aqui (ações da B3, FIIs, fundos da CVM, renda fixa) não tem chave e sai com o
/// texto da metodologia, em português.
fn text(kind: &str, id: &str, fallback: &str) -> String {
    let key = format!("nucleo-metodologia-{kind}-{id}");
    if crate::i18n::has_key(crate::i18n::Locale::PtBr, &key) {
        crate::i18n::t(&key)
    } else {
        fallback.to_string()
    }
}

/// Metodologia atual (carregada uma vez).
pub fn methodology() -> &'static Methodology {
    static M: OnceLock<Methodology> = OnceLock::new();
    M.get_or_init(|| Methodology::parse().expect("metodologia do score embutida é válida"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_methodology_is_valid() {
        let m = Methodology::parse().expect("válida");
        assert_eq!(m.version, METHODOLOGY_VERSION);
        assert!(m.profiles.contains_key("equity_br/default"));
        assert!(m.profiles.contains_key("equity_br/banks"));
        assert!(m.profiles.contains_key("reit/tijolo"));
        // A ordem do arquivo é preservada.
        let p = &m.profiles["equity_br/default"];
        assert_eq!(p.dimensions.0[0].0, "rentabilidade");
        assert_eq!(p.dimensions.0[0].1.metrics.0[0].0, "roic");
    }

    #[test]
    fn bands_follow_the_spec() {
        let m = methodology();
        assert_eq!(m.band(100.0).label, "Robusto");
        assert_eq!(m.band(75.0).label, "Robusto");
        assert_eq!(m.band(74.4).label, "Saudável");
        assert_eq!(m.band(60.0).label, "Saudável");
        assert_eq!(m.band(59.0).label, "Atenção");
        assert_eq!(m.band(40.0).label, "Atenção");
        assert_eq!(m.band(39.4).label, "Risco");
        assert_eq!(m.band(0.0).label, "Risco");
    }

    #[test]
    fn profile_anchor_overrides_catalog() {
        let m = methodology();
        let u = &m.profiles["equity_br/utilities"];
        let d = &m.profiles["equity_br/default"];
        assert_ne!(
            m.anchor(u, "net_debt_ebitda"),
            m.anchor(d, "net_debt_ebitda")
        );
        assert_eq!(m.anchor(d, "roe"), m.metrics["roe"].anchor.as_slice());
    }
    /// Os textos da metodologia que valem fora do Brasil saem no idioma do app; o pt-BR do
    /// catálogo repete a metodologia, e o que só existe aqui continua com o texto dela.
    #[test]
    fn methodology_texts_follow_the_app_language() {
        use crate::i18n::{Locale, catalog_keys, set_current, t};
        let m = methodology();
        const PREFIX: &str = "nucleo-metodologia-";
        let keys: Vec<&String> = catalog_keys(Locale::PtBr)
            .iter()
            .filter(|k| k.starts_with(PREFIX))
            .collect();
        assert!(keys.len() > 200, "{}", keys.len());
        set_current(Locale::PtBr);
        for key in keys {
            let (kind, id) = key[PREFIX.len()..].split_once('-').unwrap();
            let source = match kind {
                "metrica" => m.metrics.get(id).map(|d| d.label.clone()),
                "formula" => m.metrics.get(id).map(|d| d.formula.clone()),
                "indisponivel" => m.metrics.get(id).and_then(|d| d.unavailable.clone()),
                "dimensao" => m.dimensions.get(id).cloned(),
                "classe" => m.classes.get(id).cloned(),
                "flag" => m.flags.get(id).map(|f| f.label.clone()),
                "evento" => m.events.kinds.get(id).map(|k| k.label.clone()),
                "perfil" => m
                    .profiles
                    .iter()
                    .find(|(k, _)| k.replace('/', "-") == id)
                    .map(|(_, p)| p.label.clone()),
                "segmento" => m.class_segments.iter().find_map(|(class, cs)| {
                    cs.segments
                        .iter()
                        .find(|(k, _)| format!("{class}-{k}") == id)
                        .map(|(_, s)| s.label.clone())
                }),
                _ => None,
            };
            assert_eq!(source, Some(t(key)), "{key}");
        }
        set_current(Locale::En);
        assert_eq!(m.metric_label("net_margin"), "Net margin");
        assert_eq!(
            m.metric_formula("current_ratio"),
            "Current assets ÷ current liabilities"
        );
        assert_eq!(
            m.profile_label("equity_intl/tech"),
            "Stocks outside Brazil: technology"
        );
        assert_eq!(
            m.segment_label("etf", "exterior", "x"),
            "Listed outside Brazil"
        );
        assert_eq!(m.class_label("crypto"), "Crypto assets");
        assert_eq!(
            m.flag_label("negative_equity"),
            "Negative shareholders' equity"
        );
        assert_eq!(m.event_label("litigio"), "Material litigation");
        let intl = &m.profiles["equity_intl/default"];
        assert_eq!(m.dimension_label(intl, "rentabilidade"), "Profitability");
        // Só no Brasil: o texto da metodologia, em português.
        assert_eq!(m.profile_label("reit/tijolo"), "FII de tijolo");
        assert_eq!(m.class_label("reit"), "Fundos imobiliários");
        assert_eq!(m.flag_label("late_filing"), m.flags["late_filing"].label);
        let banks = &m.profiles["equity_br/banks"];
        assert_eq!(m.dimension_label(banks, "solvencia"), "Capital e liquidez");
        set_current(Locale::Es);
        assert_eq!(m.metric_label("pb"), "P/VL");
        set_current(Locale::PtBr);
    }
}
