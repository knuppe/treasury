//! Rendimento do FGTS e a comparação com outros destinos (docs/brazil/fgts.md, seção 6.4): a TIR da
//! conta (fluxos = saldo inicial, depósitos e saques; valor final = saldo) e quanto os mesmos
//! fluxos, nas mesmas datas, teriam virado corrigidos pelo IPCA, na poupança, em 100% e 85% do
//! CDI (a régua de uma aplicação isenta, como o FGTS) e acompanhando a expansão do M2. Mais a
//! tabela por ano (JAM, JAM + distribuição, IPCA, CDI, M2) e a compensação da ADI 5090 nos anos
//! em que JAM + distribuição ficou abaixo do IPCA. Só números, com fonte; sem recomendação.

use std::collections::BTreeMap;

use chrono::{Datelike, Months, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::EntryKind;
use crate::error::Result;
use crate::i18n::t;
use crate::ids::Id;
use crate::time::{fmt_date, parse_date};

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsCompareArgs")]
pub struct CompareArgs {
    /// Uma conta; sem ela, todas as contas FGTS juntas.
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// Período (padrão: do primeiro lançamento até hoje).
    #[serde(default)]
    #[ts(optional)]
    pub from: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsBenchmark")]
pub struct Benchmark {
    /// `ipca`, `poupanca`, `cdi`, `cdi_85`, `m2`.
    pub id: String,
    pub label: String,
    /// Quanto os mesmos fluxos teriam virado (`null`: falta a série no cofre).
    #[ts(type = "number | null")]
    pub final_value: Option<i64>,
    /// Diferença para o FGTS (positivo: o destino teria dado mais).
    #[ts(type = "number | null")]
    pub difference: Option<i64>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsYearRow")]
pub struct YearRow {
    pub year: i32,
    /// Ano incompleto (o corrente, ou o primeiro/último do período).
    pub partial: bool,
    /// Rendimento no ano, frações (0,03 = 3%).
    pub jam: Option<f64>,
    /// Com a distribuição de resultado do ano-base (`null`: ainda não creditada).
    pub jam_distribution: Option<f64>,
    pub ipca: Option<f64>,
    pub cdi: Option<f64>,
    pub m2: Option<f64>,
    /// Compensação estimada pela ADI 5090 (desde 2024, quando JAM + distribuição < IPCA).
    #[ts(type = "number | null")]
    pub compensation: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "FgtsCompare")]
pub struct Compare {
    pub from: String,
    pub to: String,
    #[ts(type = "number")]
    pub final_balance: i64,
    /// TIR ao ano (fração).
    pub irr: Option<f64>,
    #[ts(type = "number")]
    pub deposits: i64,
    #[ts(type = "number")]
    pub withdrawals: i64,
    #[ts(type = "number")]
    pub jam: i64,
    #[ts(type = "number")]
    pub distribution: i64,
    pub benchmarks: Vec<Benchmark>,
    pub years: Vec<YearRow>,
    pub notes: Vec<String>,
}

/// Uma série do cofre em memória, com o fator de crescimento entre duas datas.
pub struct Series {
    kind: SeriesKind,
    points: Vec<(NaiveDate, f64)>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SeriesKind {
    /// % ao mês, um valor por mês (IPCA) ou por dia de início do período (poupança).
    MonthlyRate,
    /// % ao dia útil (CDI).
    DailyRate,
    /// Nível (M2).
    Level,
}

impl Series {
    pub fn new(kind: SeriesKind, points: Vec<(NaiveDate, f64)>) -> Self {
        Self { kind, points }
    }

    fn load(
        conn: &Connection,
        id: &str,
        kind: SeriesKind,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Option<Self>> {
        let pts: Vec<(NaiveDate, f64)> = crate::indices::points(
            conn,
            id,
            from.checked_sub_months(Months::new(2)).unwrap_or(from),
            to,
        )?
        .into_iter()
        .filter_map(|p| Some((p.date, p.value.to_string().parse().ok()?)))
        .collect();
        Ok((!pts.is_empty()).then(|| Self::new(kind, pts)))
    }

    /// Valor vigente numa data (o último ponto ≤ data).
    fn at(&self, d: NaiveDate) -> Option<f64> {
        let i = self.points.partition_point(|(x, _)| *x <= d);
        (i > 0).then(|| self.points[i - 1].1)
    }

    /// Fator de crescimento de `a` a `b` (`None` sem dados cobrindo o período).
    pub fn growth(&self, a: NaiveDate, b: NaiveDate, share: f64) -> Option<f64> {
        if b <= a {
            return Some(1.0);
        }
        match self.kind {
            SeriesKind::DailyRate => {
                let first = self.points.first()?.0;
                if first > a {
                    return None;
                }
                Some(
                    self.points
                        .iter()
                        .filter(|(d, _)| *d >= a && *d < b)
                        .fold(1.0, |f, (_, v)| f * (1.0 + share * v / 100.0)),
                )
            }
            SeriesKind::MonthlyRate => {
                let mut f = 1.0;
                let mut m = NaiveDate::from_ymd_opt(a.year(), a.month(), 1)?;
                while m < b {
                    let next = m.checked_add_months(Months::new(1))?;
                    let start = a.max(m);
                    let end = b.min(next);
                    let frac = (end - start).num_days() as f64 / (next - m).num_days() as f64;
                    let r = self.at(m)?;
                    f *= (1.0 + share * r / 100.0).powf(frac);
                    m = next;
                }
                Some(f)
            }
            SeriesKind::Level => {
                let la = self.level(a)?;
                let lb = self.level(b)?;
                Some((lb / la).powf(share))
            }
        }
    }

    /// Nível interpolado entre os meses (geométrico pelos dias).
    fn level(&self, d: NaiveDate) -> Option<f64> {
        let i = self.points.partition_point(|(x, _)| *x <= d);
        if i == 0 {
            return None;
        }
        let (d0, v0) = self.points[i - 1];
        match self.points.get(i) {
            Some(&(d1, v1)) if v0 > 0.0 => {
                let w = (d - d0).num_days() as f64 / (d1 - d0).num_days().max(1) as f64;
                Some(v0 * (v1 / v0).powf(w))
            }
            _ => Some(v0),
        }
    }
}

/// Fluxos externos (positivo: entrou no FGTS) e o valor final: a TIR ao ano.
pub fn irr(flows: &[(NaiveDate, f64)], end: NaiveDate, final_value: f64) -> Option<f64> {
    if flows.is_empty() {
        return None;
    }
    let npv = |r: f64| -> f64 {
        flows
            .iter()
            .map(|(d, v)| v * (1.0 + r).powf((end - *d).num_days() as f64 / 365.0))
            .sum::<f64>()
            - final_value
    };
    let (mut lo, mut hi) = (-0.9, 5.0);
    if npv(lo).signum() == npv(hi).signum() {
        return None;
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if npv(mid).signum() == npv(lo).signum() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// O valor final dos fluxos crescendo pela série.
pub fn grown(
    series: &Series,
    flows: &[(NaiveDate, f64)],
    end: NaiveDate,
    share: f64,
) -> Option<f64> {
    flows
        .iter()
        .map(|(d, v)| series.growth(*d, end, share).map(|g| v * g))
        .sum()
}

struct Row {
    date: NaiveDate,
    kind: EntryKind,
    amount: i64,
    coefficient: Option<f64>,
    base_year: Option<i32>,
}

pub fn compare(conn: &Connection, a: &CompareArgs, today: NaiveDate) -> Result<Compare> {
    super::require_brazil(conn)?;
    let accounts: Vec<super::AccountView> = super::list(conn, today)?
        .into_iter()
        .filter(|x| a.account_id.as_ref().is_none_or(|id| &x.account_id == id))
        .collect();
    let mut rows = vec![];
    let mut opening = vec![];
    for acc in &accounts {
        let (ob, od): (i64, String) = conn.query_row(
            "SELECT opening_balance, opening_date FROM account WHERE id = ?1",
            [&acc.account_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if ob != 0 {
            opening.push((parse_date(&od)?, ob));
        }
        for e in super::entries(conn, &acc.account_id)? {
            rows.push(Row {
                date: parse_date(&e.date)?,
                kind: e.kind,
                amount: e.amount,
                coefficient: e.coefficient.as_deref().and_then(|c| c.parse().ok()),
                base_year: e.base_year,
            });
        }
    }
    rows.sort_by_key(|r| r.date);
    let first = rows
        .first()
        .map(|r| r.date)
        .into_iter()
        .chain(opening.iter().map(|o| o.0))
        .min()
        .unwrap_or(today);
    let from = a
        .from
        .as_deref()
        .map(parse_date)
        .transpose()?
        .unwrap_or(first);
    let to =
        a.to.as_deref()
            .map(parse_date)
            .transpose()?
            .unwrap_or(today);
    // Saldo antes do período entra como o primeiro fluxo.
    let mut start_balance: i64 = opening.iter().filter(|o| o.0 < from).map(|o| o.1).sum();
    start_balance += rows
        .iter()
        .filter(|r| r.date < from)
        .map(|r| r.amount)
        .sum::<i64>();
    let mut flows: Vec<(NaiveDate, f64)> = vec![];
    if start_balance != 0 {
        flows.push((from, start_balance as f64));
    }
    for o in opening.iter().filter(|o| o.0 >= from && o.0 <= to) {
        flows.push((o.0, o.1 as f64));
    }
    let (mut deposits, mut withdrawals, mut jam, mut distribution) = (0, 0, 0, 0);
    for r in rows.iter().filter(|r| r.date >= from && r.date <= to) {
        match r.kind {
            EntryKind::Deposit | EntryKind::DepositExtra => {
                deposits += r.amount;
                flows.push((r.date, r.amount as f64));
            }
            EntryKind::Withdrawal | EntryKind::Reversal => {
                withdrawals -= r.amount;
                flows.push((r.date, r.amount as f64));
            }
            EntryKind::Jam | EntryKind::JamAdjustment => jam += r.amount,
            EntryKind::Distribution => distribution += r.amount,
        }
    }
    let final_balance = start_balance
        + opening
            .iter()
            .filter(|o| o.0 >= from && o.0 <= to)
            .map(|o| o.1)
            .sum::<i64>()
        + rows
            .iter()
            .filter(|r| r.date >= from && r.date <= to)
            .map(|r| r.amount)
            .sum::<i64>();
    let rate = irr(&flows, to, final_balance as f64);
    let ipca = Series::load(conn, "ipca", SeriesKind::MonthlyRate, from, to)?;
    let poup = Series::load(conn, "poupanca", SeriesKind::MonthlyRate, from, to)?;
    let cdi = Series::load(conn, "cdi", SeriesKind::DailyRate, from, to)?;
    let m2 = Series::load(conn, "m2_br", SeriesKind::Level, from, to)?;
    let bench = |id: &str, label: &str, s: &Option<Series>, share: f64, source: &str| {
        let v = s.as_ref().and_then(|s| grown(s, &flows, to, share));
        Benchmark {
            id: id.into(),
            label: t(label),
            final_value: v.map(|x| x.round() as i64),
            difference: v.map(|x| x.round() as i64 - final_balance),
            source: source.into(),
        }
    };
    let benchmarks = vec![
        bench(
            "ipca",
            "nucleo-fgts-comparar-ipca",
            &ipca,
            1.0,
            "IBGE/BCB (SGS 433)",
        ),
        bench(
            "poupanca",
            "nucleo-fgts-comparar-poupanca",
            &poup,
            1.0,
            "BCB (SGS 195)",
        ),
        bench(
            "cdi_85",
            "nucleo-fgts-comparar-cdi-85",
            &cdi,
            0.85,
            "B3/BCB (SGS 12)",
        ),
        bench(
            "cdi",
            "nucleo-fgts-comparar-cdi",
            &cdi,
            1.0,
            "B3/BCB (SGS 12)",
        ),
        bench("m2", "nucleo-fgts-comparar-m2", &m2, 1.0, "BCB (SGS 27810)"),
    ];
    // Tabela por ano.
    let mut years = vec![];
    let mut dist_by_base: BTreeMap<i32, i64> = BTreeMap::new();
    for r in rows.iter().filter(|r| r.kind == EntryKind::Distribution) {
        let y = r.base_year.unwrap_or(r.date.year() - 1);
        *dist_by_base.entry(y).or_default() += r.amount;
    }
    let total_opening: i64 = opening.iter().map(|o| o.1).sum();
    let balance_at = |d: NaiveDate| -> i64 {
        total_opening
            + rows
                .iter()
                .filter(|r| r.date <= d)
                .map(|r| r.amount)
                .sum::<i64>()
    };
    for y in from.year()..=to.year() {
        let ys = NaiveDate::from_ymd_opt(y, 1, 1).expect("data").max(from);
        let ye = NaiveDate::from_ymd_opt(y + 1, 1, 1).expect("data").min(to);
        let partial = ys.month() != 1
            || ys.day() != 1
            || ye != NaiveDate::from_ymd_opt(y + 1, 1, 1).expect("data");
        let coefs: Vec<f64> = rows
            .iter()
            .filter(|r| r.kind == EntryKind::Jam && r.date >= ys && r.date < ye)
            .filter_map(|r| r.coefficient)
            .collect();
        let jam_y = (!coefs.is_empty()).then(|| coefs.iter().fold(1.0, |f, c| f * (1.0 + c)) - 1.0);
        let dec31 = NaiveDate::from_ymd_opt(y, 12, 31).expect("data");
        let base = balance_at(dec31);
        let jd = match (jam_y, dist_by_base.get(&y)) {
            (Some(j), Some(d)) if base > 0 => {
                Some((1.0 + j) * (1.0 + *d as f64 / base as f64) - 1.0)
            }
            _ => None,
        };
        let g = |s: &Option<Series>| {
            s.as_ref()
                .and_then(|s| s.growth(ys, ye, 1.0))
                .map(|f| f - 1.0)
        };
        let ipca_y = g(&ipca);
        // ADI 5090: só a partir de 12/06/2024, sem efeito retroativo.
        let compensation = match (jd, ipca_y) {
            (Some(r), Some(i)) if y >= 2024 && r < i => {
                let start = balance_at(ys.pred_opt().unwrap_or(ys)).max(0);
                Some(((i - r) * start as f64).round() as i64)
            }
            _ => None,
        };
        years.push(YearRow {
            year: y,
            partial,
            jam: jam_y,
            jam_distribution: jd,
            ipca: ipca_y,
            cdi: g(&cdi),
            m2: g(&m2),
            compensation,
        });
    }
    let mut notes = vec![t("nucleo-fgts-nota-comparar"), t("nucleo-fgts-nota-adi")];
    if benchmarks.iter().any(|b| b.final_value.is_none()) {
        notes.push(t("nucleo-fgts-nota-sem-serie"));
    }
    Ok(Compare {
        from: fmt_date(from),
        to: fmt_date(to),
        final_balance,
        irr: rate,
        deposits,
        withdrawals,
        jam,
        distribution,
        benchmarks,
        years,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn irr_of_a_single_deposit() {
        // 1.000 vira 1.100 em 365 dias: 10% ao ano.
        let r = irr(&[(d("2025-01-01"), 1000.0)], d("2026-01-01"), 1100.0).unwrap();
        assert!((r - 0.10).abs() < 1e-6, "{r}");
    }

    #[test]
    fn growth_by_monthly_rate_daily_rate_and_level() {
        // 1% ao mês em jan e fev: 1,01².
        let m = Series::new(
            SeriesKind::MonthlyRate,
            vec![(d("2025-01-01"), 1.0), (d("2025-02-01"), 1.0)],
        );
        let g = m.growth(d("2025-01-01"), d("2025-03-01"), 1.0).unwrap();
        assert!((g - 1.0201).abs() < 1e-9);
        // Meio mês: 1,01^(15/31).
        let g = m.growth(d("2025-01-01"), d("2025-01-16"), 1.0).unwrap();
        assert!((g - 1.01f64.powf(15.0 / 31.0)).abs() < 1e-9);
        // Diária: dois dias de 0,05% a 85%.
        let c = Series::new(
            SeriesKind::DailyRate,
            vec![(d("2025-01-02"), 0.05), (d("2025-01-03"), 0.05)],
        );
        let g = c.growth(d("2025-01-02"), d("2025-01-04"), 0.85).unwrap();
        assert!((g - 1.000425f64.powi(2)).abs() < 1e-12);
        // Nível: 100 → 110.
        let l = Series::new(
            SeriesKind::Level,
            vec![(d("2025-01-01"), 100.0), (d("2025-02-01"), 110.0)],
        );
        assert!((l.growth(d("2025-01-01"), d("2025-02-01"), 1.0).unwrap() - 1.1).abs() < 1e-12);
        // Fluxos: 1.000 em jan e 500 em fev, com 1% ao mês até março.
        let v = grown(
            &m,
            &[(d("2025-01-01"), 1000.0), (d("2025-02-01"), 500.0)],
            d("2025-03-01"),
            1.0,
        )
        .unwrap();
        assert!((v - (1020.1 + 505.0)).abs() < 1e-6);
    }
}
