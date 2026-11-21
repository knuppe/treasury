//! Motor do score (§7): nota de cada métrica contra os pares (percentil winsorizado +
//! âncora absoluta, com nível, média e tendência nas métricas-chave), dimensões com
//! cobertura e imputação, pesos do perfil, red flags, confiança, sub-scores e explicação.
//! Puro: recebe as métricas do ativo e as dos pares já calculadas.

use std::collections::HashMap;

use chrono::NaiveDate;

use super::config::{Direction, Methodology, Profile, Unit};
use super::metrics::{MetricValue, Metrics};
use super::normalize::{interp, median, percentile_rank, sorted, winsorize};
use super::{
    DimensionScore, INSUFFICIENT_BAND, MetricScore, MissingItem, PeerGroup, ScoreFlag, ScoreStatus,
    insufficient_label,
};

/// Um par (emissor do universo) com as métricas brutas.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Peer {
    pub id: String,
    pub ticker: Option<String>,
    pub name: String,
    /// Setor fino (FCA, subsegmento do FII).
    pub sector: Option<String>,
    /// Segmento da metodologia.
    pub segment: String,
    pub metrics: Metrics,
}

pub struct Input<'a> {
    pub m: &'a Methodology,
    pub profile: &'a Profile,
    /// Emissor do ativo (para não se comparar consigo mesmo).
    pub id: Option<&'a str>,
    pub sector: Option<&'a str>,
    /// Setor como exibido (o `sector` é normalizado para comparar).
    pub sector_label: Option<&'a str>,
    pub segment: &'a str,
    pub segment_label: &'a str,
    pub metrics: &'a Metrics,
    pub peers: &'a [Peer],
    pub flags: Vec<ScoreFlag>,
    pub data_date: Option<NaiveDate>,
    pub today: NaiveDate,
    /// Fatores extras de confiança (liquidez baixa, divergência com a conferência).
    pub extra_confidence: Vec<(String, f64)>,
}

pub struct Output {
    pub status: ScoreStatus,
    pub score: Option<f64>,
    pub raw_score: Option<f64>,
    pub band: String,
    pub band_label: String,
    pub band_meaning: String,
    pub health: Option<f64>,
    pub health_raw: Option<f64>,
    pub attractiveness: Option<f64>,
    pub confidence: f64,
    pub imputed_share: f64,
    pub dimensions: Vec<DimensionScore>,
    pub flags: Vec<ScoreFlag>,
    pub missing: Vec<MissingItem>,
    pub top_drivers: Vec<String>,
    pub top_detractors: Vec<String>,
    pub peers: PeerGroup,
    pub stale: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Part {
    Level,
    Avg,
    Trend,
}

/// Nível de pares usado numa métrica.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Level {
    Sector,
    Segment,
    Class,
}

impl Level {
    fn key(self) -> &'static str {
        match self {
            Level::Sector => "sector",
            Level::Segment => "segment",
            Level::Class => "class",
        }
    }
}

/// Valor de uma parte da métrica; o pior (denominador negativo) vira um extremo.
fn part_value(mv: &MetricValue, part: Part, dir: Direction) -> Option<f64> {
    match part {
        Part::Level if mv.worst => Some(if dir == Direction::Lower { 1e15 } else { -1e15 }),
        Part::Level => mv.value,
        Part::Avg => mv.avg,
        Part::Trend => mv.trend,
    }
}

struct Ctx<'a> {
    m: &'a Methodology,
    profile: &'a Profile,
    peers: &'a [Peer],
    sector: Option<&'a str>,
    segment: &'a str,
}

struct Scored {
    score: Option<f64>,
    percentile: Option<f64>,
    absolute: Option<f64>,
    peers: Option<u32>,
    level: Option<Level>,
}

impl<'a> Ctx<'a> {
    fn in_level(&self, p: &Peer, level: Level) -> bool {
        match level {
            Level::Sector => {
                self.sector.is_some()
                    && p.sector.as_deref() == self.sector
                    && p.segment == self.segment
            }
            Level::Segment => p.segment == self.segment,
            Level::Class => true,
        }
    }

    /// Valores dos pares de um nível (sem o próprio emissor).
    fn values(&self, key: &str, part: Part, level: Level, skip: Option<&str>) -> Vec<f64> {
        let dir = self.m.metric(key).direction;
        sorted(
            self.peers
                .iter()
                .filter(|p| Some(p.id.as_str()) != skip && self.in_level(p, level))
                .filter_map(|p| p.metrics.get(key).and_then(|mv| part_value(mv, part, dir))),
        )
    }

    /// Primeiro nível com pares suficientes para a parte da métrica.
    fn pick(&self, key: &str, part: Part, skip: Option<&str>) -> Option<(Level, Vec<f64>)> {
        let min = self.m.normalization.peer_min;
        for level in [Level::Sector, Level::Segment, Level::Class] {
            let v = self.values(key, part, level, skip);
            if v.len() >= min {
                return Some((level, v));
            }
        }
        None
    }

    fn relative(&self, vals: &[f64], x: f64, dir: Direction) -> f64 {
        let n = &self.m.normalization;
        let (w, lo, hi) = winsorize(vals, n.winsor_low, n.winsor_high);
        let p = percentile_rank(&w, x.clamp(lo, hi)).unwrap_or(50.0);
        if dir == Direction::Lower {
            100.0 - p
        } else {
            p
        }
    }

    /// Nota de uma parte (nível, média ou tendência) da métrica.
    fn part_score(
        &self,
        key: &str,
        mv: &MetricValue,
        part: Part,
        skip: Option<&str>,
    ) -> Option<Scored> {
        let def = self.m.metric(key);
        let n = &self.m.normalization;
        let x = part_value(mv, part, def.direction)?;
        let absolute = match part {
            Part::Level | Part::Avg if mv.worst && part == Part::Level => 0.0,
            Part::Level | Part::Avg => interp(self.m.anchor(self.profile, key), x),
            Part::Trend => {
                let s = def.trend_scale.unwrap_or(1.0);
                let t = if def.direction == Direction::Lower {
                    -x
                } else {
                    x
                };
                interp(&[(-s, 0.0), (0.0, 50.0), (s, 100.0)], t)
            }
        };
        if def.absolute_only || def.direction == Direction::Range {
            return Some(Scored {
                score: Some(absolute),
                percentile: None,
                absolute: Some(absolute),
                peers: None,
                level: None,
            });
        }
        let Some((level, vals)) = self.pick(key, part, skip) else {
            return Some(Scored {
                score: Some(absolute),
                percentile: None,
                absolute: Some(absolute),
                peers: None,
                level: None,
            });
        };
        let pct = if mv.worst && part == Part::Level {
            0.0
        } else {
            self.relative(&vals, x, def.direction)
        };
        Some(Scored {
            score: Some(n.relative_weight * pct + n.absolute_weight * absolute),
            percentile: Some(pct),
            absolute: Some(absolute),
            peers: Some(vals.len() as u32),
            level: Some(level),
        })
    }

    /// Nota da métrica: nível; nas métricas-chave, 0,5·nível + 0,25·média + 0,25·tendência
    /// (renormalizado se faltar média ou tendência).
    fn metric_score(&self, key: &str, mv: &MetricValue, skip: Option<&str>) -> Option<Scored> {
        let def = self.m.metric(key);
        if def.unavailable.is_some() {
            return None;
        }
        let level = self.part_score(key, mv, Part::Level, skip)?;
        if !def.key || mv.worst {
            return Some(level);
        }
        let n = &self.m.normalization;
        let mut acc = n.level_weight * level.score.unwrap_or(0.0);
        let mut w = n.level_weight;
        for (part, pw) in [(Part::Avg, n.average_weight), (Part::Trend, n.trend_weight)] {
            if let Some(s) = self.part_score(key, mv, part, skip).and_then(|s| s.score) {
                acc += pw * s;
                w += pw;
            }
        }
        Some(Scored {
            score: Some(acc / w),
            ..level
        })
    }

    /// Nota de uma dimensão para um conjunto de métricas (sem imputação): (nota, cobertura).
    fn dimension_raw(
        &self,
        metrics: &Metrics,
        dim: &[(String, f64)],
        skip: Option<&str>,
    ) -> (Option<f64>, f64) {
        let total: f64 = dim.iter().map(|(_, w)| w).sum();
        let (mut acc, mut cov) = (0.0, 0.0);
        for (k, w) in dim {
            let Some(mv) = metrics.get(k) else { continue };
            if let Some(s) = self.metric_score(k, mv, skip).and_then(|s| s.score) {
                acc += w * s;
                cov += w;
            }
        }
        (
            (cov > 0.0).then(|| acc / cov),
            if total > 0.0 { cov / total } else { 0.0 },
        )
    }

    /// Mediana das notas da dimensão entre os pares (do nível mais fino com pares suficientes).
    fn peer_dimension_median(&self, dim: &[(String, f64)], skip: Option<&str>) -> Option<f64> {
        let min_cov = self.m.missing.dimension_min_coverage;
        for level in [Level::Sector, Level::Segment, Level::Class] {
            let group: Vec<&Peer> = self
                .peers
                .iter()
                .filter(|p| Some(p.id.as_str()) != skip && self.in_level(p, level))
                .collect();
            if group.len() < self.m.normalization.peer_min {
                continue;
            }
            let vals = sorted(group.iter().filter_map(|p| {
                let (s, c) = self.dimension_raw(&p.metrics, dim, Some(&p.id));
                s.filter(|_| c >= min_cov)
            }));
            if vals.len() >= self.m.normalization.peer_min {
                return median(&vals);
            }
        }
        None
    }
}

/// Valor formatado para as explicações.
pub fn fmt_value(unit: Unit, v: f64) -> String {
    let num = |x: f64, dp: usize| {
        crate::ai::alerts::decimal(crate::i18n::current(), &format!("{x:.dp$}"))
    };
    let with =
        |key: &str, x: String| crate::i18n::t_args(key, &crate::i18n::args(&[("valor", x.into())]));
    match unit {
        Unit::Percent => format!("{}%", num(v * 100.0, 1)),
        Unit::Multiple => format!("{}x", num(v, 2)),
        Unit::Score => num(v, if v.fract() == 0.0 { 0 } else { 2 }),
        Unit::Count => num(v, 0),
        Unit::Days => with("nucleo-score-valor-dias", num(v, 0)),
        Unit::Years => with("nucleo-score-valor-anos", num(v, 1)),
        Unit::Money => {
            if v.abs() >= 1e9 {
                with("nucleo-score-valor-bi", num(v / 1e9, 1))
            } else if v.abs() >= 1e6 {
                with("nucleo-score-valor-mi", num(v / 1e6, 1))
            } else {
                with("nucleo-score-valor-mil", num(v / 1e3, 0))
            }
        }
    }
}

fn level_label(level: Level, sector: Option<&str>, segment_label: &str) -> String {
    use crate::i18n::{args, t, t_args};
    match level {
        Level::Sector => t_args(
            "nucleo-score-pares-do-setor",
            &args(&[("setor", sector.unwrap_or("").to_string().into())]),
        ),
        Level::Segment => t_args(
            "nucleo-score-pares-do-segmento",
            &args(&[("segmento", segment_label.to_string().into())]),
        ),
        Level::Class => t("nucleo-score-pares-da-classe"),
    }
}

pub fn run(i: Input<'_>) -> Output {
    let m = i.m;
    let p = i.profile;
    let cx = Ctx {
        m,
        profile: p,
        peers: i.peers,
        sector: i.sector,
        segment: i.segment,
    };
    let skip = i.id;
    let min_cov = m.missing.dimension_min_coverage;
    let mut dims = vec![];
    let mut missing = vec![];
    let mut contributions: Vec<(f64, String)> = vec![];
    let mut imputed_w = 0.0;
    let mut covered_w = 0.0;
    let mut level_factors: Vec<(f64, f64)> = vec![];
    let total_w: f64 = p.dimensions.0.iter().map(|(_, d)| d.weight).sum();
    let level_factor = |l: Option<Level>| {
        m.confidence
            .peer_level_factor
            .get(l.map(Level::key).unwrap_or("none"))
            .copied()
            .unwrap_or(1.0)
    };

    for (dk, dw) in &p.dimensions.0 {
        let label = m.dimension_label(p, dk);
        let dim_total: f64 = dw.metrics.0.iter().map(|(_, w)| w).sum();
        let mut ms = vec![];
        let (mut acc, mut cov) = (0.0, 0.0);
        for (mk, w) in &dw.metrics.0 {
            let def = m.metric(mk);
            let critical = p.critical.contains(mk);
            let mv = i.metrics.get(mk).cloned().unwrap_or_else(|| {
                MetricValue::missing(
                    m.metric_unavailable(mk)
                        .unwrap_or_else(|| crate::i18n::t("nucleo-score-sem-dado")),
                )
            });
            let mv = match m.metric_unavailable(mk) {
                Some(why) if mv.value.is_none() => MetricValue::missing(why),
                _ => mv,
            };
            let s = cx.metric_score(mk, &mv, skip);
            let score = s.as_ref().and_then(|s| s.score);
            if let Some(sc) = score {
                acc += w * sc;
                cov += w;
                if def.direction != Direction::Range && !def.absolute_only {
                    level_factors.push((
                        dw.weight * w / dim_total,
                        level_factor(s.as_ref().and_then(|s| s.level)),
                    ));
                }
            } else if !missing.iter().any(|x: &MissingItem| x.key == *mk) {
                missing.push(MissingItem {
                    key: mk.clone(),
                    label: m.metric_label(mk),
                    dimension: label.clone(),
                    reason: mv
                        .missing
                        .clone()
                        .unwrap_or_else(|| crate::i18n::t("nucleo-score-sem-dado")),
                    critical,
                });
            }
            ms.push(MetricScore {
                key: mk.clone(),
                label: m.metric_label(mk),
                unit: def.unit,
                direction: def.direction,
                formula: m.metric_formula(mk),
                weight: *w,
                value: mv.value,
                score: score.map(|x| (x * 10.0).round() / 10.0),
                percentile: s.as_ref().and_then(|s| s.percentile).map(|x| x.round()),
                absolute: s.as_ref().and_then(|s| s.absolute).map(|x| x.round()),
                peers: s.as_ref().and_then(|s| s.peers),
                peer_level: s
                    .as_ref()
                    .and_then(|s| s.level)
                    .map(|l| level_label(l, i.sector_label.or(i.sector), i.segment_label)),
                average: mv.avg,
                trend: mv.trend,
                period: mv.period.clone(),
                source: mv.source.clone(),
                note: mv.note.clone(),
                missing: score.is_none().then(|| {
                    mv.missing
                        .clone()
                        .unwrap_or_else(|| crate::i18n::t("nucleo-score-sem-dado"))
                }),
                critical,
                worst: mv.worst,
            });
        }
        let coverage = if dim_total > 0.0 {
            cov / dim_total
        } else {
            0.0
        };
        let mut imputed = false;
        let score = if coverage >= min_cov && cov > 0.0 {
            covered_w += dw.weight * coverage;
            Some(acc / cov)
        } else {
            imputed = true;
            imputed_w += dw.weight;
            covered_w += dw.weight * coverage;
            let med = cx
                .peer_dimension_median(&dw.metrics.0, skip)
                .unwrap_or(50.0);
            Some(med * m.missing.impute_factor)
        };
        // Contribuições (para os principais fatores), só das métricas medidas.
        if !imputed && cov > 0.0 {
            for x in &ms {
                if let Some(sc) = x.score {
                    let c = dw.weight / total_w * (x.weight / cov) * (sc - 50.0);
                    contributions.push((c, explain(m, x)));
                }
            }
        }
        dims.push(DimensionScore {
            key: dk.clone(),
            label,
            weight: dw.weight,
            score: score.map(|x| (x * 10.0).round() / 10.0),
            coverage,
            imputed,
            attractiveness: p.attractiveness.contains(dk),
            metrics: ms,
        });
    }

    let critical_missing = missing.iter().any(|x| x.critical);
    let imputed_share = if total_w > 0.0 {
        imputed_w / total_w
    } else {
        0.0
    };
    let weighted = |pred: &dyn Fn(&DimensionScore) -> bool| -> Option<f64> {
        let (mut a, mut w) = (0.0, 0.0);
        for d in dims.iter().filter(|d| pred(d)) {
            if let Some(s) = d.score {
                a += d.weight * s;
                w += d.weight;
            }
        }
        (w > 0.0).then(|| a / w)
    };
    let raw = weighted(&|_| true);
    let health_raw = weighted(&|d| !d.attractiveness);
    let attractiveness = weighted(&|d| d.attractiveness);
    let cap = i
        .flags
        .iter()
        .filter_map(|f| f.cap)
        .fold(f64::INFINITY, f64::min);
    let penalty: f64 = i.flags.iter().filter_map(|f| f.penalty).sum();
    let apply = |s: f64| (s - penalty).min(cap).clamp(0.0, 100.0);
    let r1 = |x: f64| (x * 10.0).round() / 10.0;

    // Confiança: cobertura ponderada × frescor × tamanho dos pares × extras.
    let coverage_factor = if total_w > 0.0 {
        covered_w / total_w
    } else {
        0.0
    };
    let stale = i
        .data_date
        .is_none_or(|d| (i.today - d).num_days() > m.missing.stale_days);
    let fresh = if stale {
        m.confidence.stale_factor
    } else {
        1.0
    };
    let lw: f64 = level_factors.iter().map(|x| x.0).sum();
    let peer_factor = if lw > 0.0 {
        level_factors.iter().map(|x| x.0 * x.1).sum::<f64>() / lw
    } else {
        level_factor(None)
    };
    let extra: f64 = i.extra_confidence.iter().map(|x| x.1).product();
    let confidence = (coverage_factor * fresh * peer_factor * extra * 100.0).round() / 100.0;

    let mut notes = vec![];
    if stale {
        notes.push(match i.data_date {
            Some(d) => crate::i18n::t_args(
                "nucleo-score-nota-defasados",
                &crate::i18n::args(&[("data", super::metrics::br(d).into())]),
            ),
            None => crate::i18n::t("nucleo-score-nota-sem-data"),
        });
    }
    for (why, f) in &i.extra_confidence {
        notes.push(crate::i18n::t_args(
            "nucleo-score-nota-confianca",
            &crate::i18n::args(&[
                (
                    "fator",
                    fmt_value(Unit::Multiple, *f)
                        .trim_end_matches('x')
                        .to_string()
                        .into(),
                ),
                ("motivo", why.clone().into()),
            ]),
        ));
    }

    let (status, score, band, band_label, band_meaning) = if critical_missing {
        let names: Vec<String> = missing
            .iter()
            .filter(|x| x.critical)
            .map(|x| x.label.clone())
            .collect();
        (
            ScoreStatus::InsufficientData,
            None,
            INSUFFICIENT_BAND.to_string(),
            insufficient_label(),
            crate::i18n::t_args(
                "nucleo-score-sem-score-falta",
                &crate::i18n::args(&[("metricas", join_pt(&names).into())]),
            ),
        )
    } else if let Some(r) = raw {
        let s = r1(apply(r));
        if imputed_share > m.missing.max_imputed_share {
            (
                ScoreStatus::Partial,
                Some(s),
                INSUFFICIENT_BAND.to_string(),
                insufficient_label(),
                crate::i18n::t_args(
                    "nucleo-score-parcial",
                    &crate::i18n::args(&[
                        ("imputado", format!("{:.0}", imputed_share * 100.0).into()),
                        (
                            "limite",
                            format!("{:.0}", m.missing.max_imputed_share * 100.0).into(),
                        ),
                    ]),
                ),
            )
        } else {
            let b = m.band(s);
            (
                ScoreStatus::Ok,
                Some(s),
                b.key.clone(),
                b.label_text(),
                b.meaning_text(),
            )
        }
    } else {
        (
            ScoreStatus::InsufficientData,
            None,
            INSUFFICIENT_BAND.to_string(),
            insufficient_label(),
            crate::i18n::t("nucleo-score-sem-metricas"),
        )
    };
    let insufficient = status == ScoreStatus::InsufficientData;
    if insufficient {
        for d in &mut dims {
            d.score = None;
        }
    }

    contributions.sort_by(|a, b| b.0.total_cmp(&a.0));
    let top_drivers: Vec<String> = contributions
        .iter()
        .filter(|c| c.0 > 0.5)
        .take(3)
        .map(|c| c.1.clone())
        .collect();
    let top_detractors: Vec<String> = contributions
        .iter()
        .rev()
        .filter(|c| c.0 < -0.5)
        .take(3)
        .map(|c| c.1.clone())
        .collect();

    let count = |level: Level| {
        i.peers
            .iter()
            .filter(|x| Some(x.id.as_str()) != skip && cx.in_level(x, level))
            .count() as u32
    };
    Output {
        status,
        score,
        raw_score: if insufficient { None } else { raw.map(r1) },
        band,
        band_label,
        band_meaning,
        health_raw: if insufficient {
            None
        } else {
            health_raw.map(r1)
        },
        health: if insufficient {
            None
        } else {
            health_raw.map(|h| r1(apply(h)))
        },
        attractiveness: if insufficient {
            None
        } else {
            attractiveness.map(r1)
        },
        confidence,
        imputed_share,
        dimensions: dims,
        flags: i.flags,
        missing,
        top_drivers: if insufficient { vec![] } else { top_drivers },
        top_detractors: if insufficient { vec![] } else { top_detractors },
        peers: PeerGroup {
            sector: i.sector_label.or(i.sector).map(str::to_string),
            segment: i.segment.to_string(),
            segment_label: i.segment_label.to_string(),
            sector_size: count(Level::Sector),
            segment_size: count(Level::Segment),
            class_size: count(Level::Class),
        },
        stale,
        notes,
    }
}

fn explain(m: &Methodology, x: &MetricScore) -> String {
    let def = m.metric(&x.key);
    let v = match (x.worst, x.value) {
        (true, _) => x
            .note
            .clone()
            .unwrap_or_else(|| crate::i18n::t("nucleo-score-pior-caso")),
        (_, Some(v)) if (def.unit == Unit::Score || v >= 99.0) && x.note.is_some() => {
            x.note.clone().unwrap_or_default()
        }
        (_, Some(v)) => fmt_value(def.unit, v),
        _ => "—".into(),
    };
    match (x.percentile, x.peers, &x.peer_level) {
        (Some(p), Some(n), Some(l)) => crate::i18n::t_args(
            "nucleo-score-explica-pares",
            &crate::i18n::args(&[
                ("metrica", x.label.clone().into()),
                ("valor", v.into()),
                ("percentil", format!("{p:.0}").into()),
                ("pares", n.to_string().into()),
                ("nivel", l.clone().into()),
            ]),
        ),
        _ => crate::i18n::t_args(
            "nucleo-score-explica-absoluta",
            &crate::i18n::args(&[
                ("metrica", x.label.clone().into()),
                ("valor", v.into()),
                ("nota", format!("{:.0}", x.score.unwrap_or(0.0)).into()),
            ]),
        ),
    }
}

pub(crate) fn join_pt(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => crate::i18n::t_args(
            "nucleo-score-lista",
            &crate::i18n::args(&[
                ("itens", items[..n - 1].join(", ").into()),
                ("ultimo", items[n - 1].clone().into()),
            ]),
        ),
    }
}

/// Índice dos pares por id (para achar o próprio ativo no universo).
pub fn by_id(peers: &[Peer]) -> HashMap<&str, &Peer> {
    peers.iter().map(|p| (p.id.as_str(), p)).collect()
}
