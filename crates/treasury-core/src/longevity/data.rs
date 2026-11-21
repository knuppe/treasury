//! Tábuas embutidas no binário: `data/longevity/wpp2024.zip`, gerado por
//! `scripts/longevity-data.py` (formato no cabeçalho do script). Lidas uma vez por processo.
//!
//! - `qx.bin`: qx × 65535 em u16 LE, na ordem país, sexo, ano, idade. A última idade é o grupo
//!   aberto (100+), guardado como o q anual de uma força de mortalidade constante com a
//!   expectativa da tábua nessa idade; o cálculo repete esse q dali em diante.
//! - `e0_cohort.bin`: expectativa ao nascer por geração × 100, na ordem país, sexo, ano de
//!   nascimento; 0 = sem dado.
//! - `ibge2024.json`: tábua de período do IBGE (só Brasil), como referência.

use std::io::{Cursor, Read};
use std::sync::OnceLock;

use serde::Deserialize;

use super::math::Mortality;
use crate::error::{Error, Result};
use crate::i18n::{args, t_args};

const EMBEDDED: &[u8] = include_bytes!("../../data/longevity/wpp2024.zip");

/// Sexo da tábua. Sem sexo informado (ou "prefiro não informar"): ambos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableSex {
    Both,
    Female,
    Male,
}

impl TableSex {
    pub fn code(self) -> &'static str {
        match self {
            TableSex::Both => "both",
            TableSex::Female => "female",
            TableSex::Male => "male",
        }
    }

    fn position(self) -> usize {
        match self {
            TableSex::Both => 0,
            TableSex::Female => 1,
            TableSex::Male => 2,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Index {
    source: String,
    countries: Vec<String>,
    sexes: Vec<String>,
    years: [i32; 2],
    ages: [u32; 2],
    cohort_years: [i32; 2],
}

/// Uma tábua do IBGE: qx (0 a 1) e ex, idade a idade até 90 (90 ou mais).
#[derive(Debug, Clone, Deserialize)]
pub struct IbgeTable {
    pub qx: Vec<f64>,
    pub ex: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ibge {
    pub both: IbgeTable,
    pub female: IbgeTable,
    pub male: IbgeTable,
}

impl Ibge {
    pub fn of(&self, sex: TableSex) -> &IbgeTable {
        match sex {
            TableSex::Both => &self.both,
            TableSex::Female => &self.female,
            TableSex::Male => &self.male,
        }
    }
}

/// As tábuas carregadas.
#[derive(Debug, Clone)]
pub struct Tables {
    /// Tábua sintética de desenvolvimento (`index.json` com `source = synthetic`).
    pub synthetic: bool,
    pub countries: Vec<String>,
    /// Posição de cada [`TableSex`] (ambos, mulheres, homens) na ordem dos dados.
    sex_pos: [usize; 3],
    nsex: usize,
    /// Primeiro e último ano de `qx` (depois do último, repete o último).
    pub years: (i32, i32),
    /// Última idade de `qx` (o grupo aberto); a primeira é sempre 0.
    pub last_age: u32,
    /// Nascimentos com a expectativa ao nascer por geração.
    pub cohorts: (i32, i32),
    qx: Vec<u16>,
    e0: Vec<u16>,
    pub ibge: Option<Ibge>,
}

fn read_entry(z: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<Vec<u8>> {
    let f = z.by_name(name).ok()?;
    let mut out = Vec::new();
    // Limite contra arquivo corrompido (o maior tem uns 3 MB).
    f.take(64 * 1024 * 1024).read_to_end(&mut out).ok()?;
    Some(out)
}

fn u16s(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect()
}

impl Tables {
    /// Monta as tábuas a partir das partes (o leitor do zip e os testes).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        synthetic: bool,
        countries: Vec<String>,
        sexes: &[&str],
        years: (i32, i32),
        last_age: u32,
        cohorts: (i32, i32),
        qx: Vec<u16>,
        e0: Vec<u16>,
        ibge: Option<Ibge>,
    ) -> std::result::Result<Self, String> {
        let pos = |s: TableSex| {
            sexes
                .iter()
                .position(|x| *x == s.code())
                .ok_or_else(|| format!("sexo {} ausente", s.code()))
        };
        let sex_pos = [
            pos(TableSex::Both)?,
            pos(TableSex::Female)?,
            pos(TableSex::Male)?,
        ];
        if years.1 < years.0 || cohorts.1 < cohorts.0 || last_age == 0 {
            return Err("faixas de anos ou idades inválidas".into());
        }
        let ny = (years.1 - years.0 + 1) as usize;
        let nc = (cohorts.1 - cohorts.0 + 1) as usize;
        let na = last_age as usize + 1;
        let n = countries.len() * sexes.len();
        if qx.len() != n * ny * na {
            return Err(format!(
                "qx.bin com {} valores; esperados {}",
                qx.len(),
                n * ny * na
            ));
        }
        if e0.len() != n * nc {
            return Err(format!(
                "e0_cohort.bin com {} valores; esperados {}",
                e0.len(),
                n * nc
            ));
        }
        Ok(Self {
            synthetic,
            countries,
            sex_pos,
            nsex: sexes.len(),
            years,
            last_age,
            cohorts,
            qx,
            e0,
            ibge,
        })
    }

    /// Lê o zip no formato de `scripts/longevity-data.py`.
    pub fn from_zip(bytes: &[u8]) -> std::result::Result<Self, String> {
        let mut z = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let index: Index =
            serde_json::from_slice(&read_entry(&mut z, "index.json").ok_or("sem index.json")?)
                .map_err(|e| format!("index.json: {e}"))?;
        if index.ages[0] != 0 {
            return Err("as idades precisam começar em 0".into());
        }
        let qx = u16s(&read_entry(&mut z, "qx.bin").ok_or("sem qx.bin")?);
        let e0 = u16s(&read_entry(&mut z, "e0_cohort.bin").ok_or("sem e0_cohort.bin")?);
        let ibge = match read_entry(&mut z, "ibge2024.json") {
            Some(b) => {
                let t: Ibge =
                    serde_json::from_slice(&b).map_err(|e| format!("ibge2024.json: {e}"))?;
                for s in [TableSex::Both, TableSex::Female, TableSex::Male] {
                    let tab = t.of(s);
                    if tab.qx.is_empty() || tab.qx.len() != tab.ex.len() {
                        return Err("ibge2024.json incompleto".into());
                    }
                }
                Some(t)
            }
            None => None,
        };
        let sexes: Vec<&str> = index.sexes.iter().map(String::as_str).collect();
        Self::from_parts(
            index.source == "synthetic",
            index.countries,
            &sexes,
            (index.years[0], index.years[1]),
            index.ages[1],
            (index.cohort_years[0], index.cohort_years[1]),
            qx,
            e0,
            ibge,
        )
    }

    /// Posição do país (ISO 3166-1 alfa-2) nos dados.
    pub fn country(&self, code: &str) -> Option<usize> {
        self.countries.iter().position(|c| c == code)
    }

    /// q no ano e idade, com os limites: antes do primeiro ano, o primeiro; depois do último,
    /// o último; depois da última idade, o grupo aberto.
    pub fn q(&self, country: usize, sex: TableSex, year: i32, age: u32) -> f64 {
        let y = (year.clamp(self.years.0, self.years.1) - self.years.0) as usize;
        let a = age.min(self.last_age) as usize;
        let ny = (self.years.1 - self.years.0 + 1) as usize;
        let na = self.last_age as usize + 1;
        let i = ((country * self.nsex + self.sex_pos[sex.position()]) * ny + y) * na + a;
        f64::from(self.qx[i]) / 65535.0
    }

    /// Expectativa ao nascer por geração (calculada pelo script, de 1950 a 2100).
    pub fn cohort_e0(&self, country: usize, sex: TableSex, birth_year: i32) -> Option<f64> {
        if !(self.cohorts.0..=self.cohorts.1).contains(&birth_year) {
            return None;
        }
        let nc = (self.cohorts.1 - self.cohorts.0 + 1) as usize;
        let i = (country * self.nsex + self.sex_pos[sex.position()]) * nc
            + (birth_year - self.cohorts.0) as usize;
        let v = self.e0[i];
        (v > 0).then(|| f64::from(v) / 100.0)
    }

    /// As taxas de um país e sexo.
    pub fn view(&self, country: usize, sex: TableSex) -> View<'_> {
        View {
            tables: self,
            country,
            sex,
        }
    }
}

/// Taxas de um país e sexo (o que o cálculo usa).
pub struct View<'a> {
    tables: &'a Tables,
    country: usize,
    sex: TableSex,
}

impl Mortality for View<'_> {
    fn q(&self, year: i32, age: u32) -> f64 {
        self.tables.q(self.country, self.sex, year, age)
    }
}

/// As tábuas embutidas, carregadas uma vez.
pub fn embedded() -> Result<&'static Tables> {
    static TABLES: OnceLock<std::result::Result<Tables, String>> = OnceLock::new();
    TABLES
        .get_or_init(|| Tables::from_zip(EMBEDDED))
        .as_ref()
        .map_err(|e| {
            Error::Other(t_args(
                "nucleo-longevidade-erro-dados",
                &args(&[("erro", e.clone().into())]),
            ))
        })
}
