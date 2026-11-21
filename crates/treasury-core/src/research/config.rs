//! Ajustes da análise de ativos (tabela `setting`): identificação exigida pela SEC e as
//! premissas padrão do preço justo de Bazin.

use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::indicators::BazinParams;
use crate::error::{Error, Result};

const KEY: &str = "research.settings";
pub const LAST_SYNC_SETTING: &str = "research.last_sync_at";
pub const LAST_REPORT_SETTING: &str = "research.last_report";

/// User-Agent padrão: a SEC exige um contato real; o usuário deve trocar o e-mail.
pub const DEFAULT_SEC_USER_AGENT: &str = "Treasury/0.1 contato@exemplo";
/// Aviso para trocar o e-mail do User-Agent da SEC, no idioma da instalação.
pub fn sec_hint() -> String {
    crate::i18n::t("nucleo-pesquisa-aviso-sec")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct ResearchSettings {
    /// User-Agent enviado à SEC EDGAR (nome do app + e-mail de contato).
    pub sec_user_agent: String,
    /// Taxa mínima do preço justo de Bazin (fração: 0,06 = 6%).
    pub bazin_rate: f64,
    /// Anos da média de dividendos do Bazin.
    pub bazin_years: u32,
}

impl Default for ResearchSettings {
    fn default() -> Self {
        Self {
            sec_user_agent: DEFAULT_SEC_USER_AGENT.into(),
            bazin_rate: 0.06,
            bazin_years: 5,
        }
    }
}

impl ResearchSettings {
    pub fn bazin(&self) -> BazinParams {
        BazinParams {
            rate: Decimal::from_f64_retain(self.bazin_rate)
                .map(|d| d.round_dp(6))
                .unwrap_or(Decimal::new(6, 2)),
            years: self.bazin_years,
        }
    }
    pub fn sec_user_agent_is_default(&self) -> bool {
        let ua = self.sec_user_agent.trim();
        ua.is_empty() || ua == DEFAULT_SEC_USER_AGENT || ua.contains("@exemplo")
    }
}

/// Ajustes com o aviso da SEC para a interface.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResearchSettingsView {
    pub settings: ResearchSettings,
    pub sec_user_agent_is_default: bool,
    /// Aviso para configurar o e-mail (quando o padrão está em uso).
    pub sec_hint: Option<String>,
    pub last_sync_at: Option<String>,
}

pub fn load(conn: &Connection) -> Result<ResearchSettings> {
    crate::settings::get_or(conn, KEY, ResearchSettings::default())
}

pub fn view(conn: &Connection) -> Result<ResearchSettingsView> {
    let s = load(conn)?;
    Ok(ResearchSettingsView {
        sec_user_agent_is_default: s.sec_user_agent_is_default(),
        sec_hint: s.sec_user_agent_is_default().then(sec_hint),
        settings: s,
        last_sync_at: crate::settings::get(conn, LAST_SYNC_SETTING)?,
    })
}

/// Taxa do Bazin informada em fração (0,06) ou em percentual (6).
pub fn normalize_rate(r: f64) -> Result<f64> {
    let r = if r > 1.0 { r / 100.0 } else { r };
    if !(r > 0.0 && r <= 1.0) || !r.is_finite() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-bazin-taxa",
        )));
    }
    Ok(r)
}

pub fn validate_years(y: u32) -> Result<u32> {
    if !(1..=10).contains(&y) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-bazin-anos",
        )));
    }
    Ok(y)
}

pub fn save(conn: &Connection, s: &ResearchSettings) -> Result<ResearchSettings> {
    let ua = s.sec_user_agent.trim();
    if ua.is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-sec-ua-vazio",
        )));
    }
    if ua.len() > 200 || ua.chars().any(|c| c.is_control()) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-sec-ua-invalido",
        )));
    }
    if !ua.contains('@') {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-pesquisa-erro-sec-ua-email",
        )));
    }
    let clean = ResearchSettings {
        sec_user_agent: ua.to_string(),
        bazin_rate: normalize_rate(s.bazin_rate)?,
        bazin_years: validate_years(s.bazin_years)?,
    };
    crate::settings::set(conn, KEY, &clean)?;
    Ok(clean)
}
