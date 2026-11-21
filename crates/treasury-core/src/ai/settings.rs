//! Ajustes da IA (Ajustes › IA), guardados na tabela `setting`.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::attachments::{self, Retention};
use crate::error::{Error, Result};
use crate::settings as kv;

pub const KEEP_HISTORY: &str = "ai.keep_history";
pub const ROUTINE_REVIEW: &str = "ai.routine.review";
pub const ROUTINE_MONTHLY_CLOSE: &str = "ai.routine.monthly_close";
pub const ROUTINE_ALERTS: &str = "ai.routine.alerts";

/// Logo depois do primeiro motor do cofre: as rotinas (que mandam dados ao motor em segundo plano)
/// começam desligadas, e a pessoa liga as que quiser em Agente › Configurar. Cofres que já
/// tinham motor continuam com o padrão de antes (ligadas), salvo o que a pessoa escolheu.
pub fn routines_off_after_first_engine(conn: &Connection) -> Result<()> {
    let engines: i64 = conn.query_row("SELECT count(*) FROM ai_engine", [], |r| r.get(0))?;
    if engines != 1 {
        return Ok(());
    }
    for key in [ROUTINE_REVIEW, ROUTINE_MONTHLY_CLOSE, ROUTINE_ALERTS] {
        if kv::get::<bool>(conn, key)?.is_none() {
            kv::set(conn, key, &false)?;
        }
    }
    Ok(())
}
/// Guardar os anexos no cofre (`none`, `days7`, `always`). Padrão: não guardar.
pub const ATTACHMENT_RETENTION: &str = "ai.attachments.retention";
/// Apagar conversas sem uso há N dias (0 = nunca). Padrão: 90.
pub const HISTORY_RETENTION_DAYS: &str = "ai.history.retention_days";
pub const DEFAULT_HISTORY_RETENTION_DAYS: i64 = 90;
/// Opções de [`HISTORY_RETENTION_DAYS`].
pub const HISTORY_RETENTION_CHOICES: &[i64] = &[0, 30, 90, 180, 365];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RoutineSettings {
    /// Revisão contínua: categorizar lançamentos sem categoria e sugerir regras (propostas).
    pub review: bool,
    /// Fechamento mensal com números e 3 a 5 recomendações (vira alerta).
    pub monthly_close: bool,
    /// Alertas: orçamento estourando, vencimentos, carteira desbalanceada.
    pub alerts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AiSettings {
    /// Aplicar sem perguntar as escritas reversíveis da IA (desligado por padrão).
    pub autonomous: bool,
    /// Ferramentas de escrita para clientes MCP externos (sempre como proposta).
    pub mcp_allow_writes: bool,
    /// Guardar as conversas do agente (sem isso, cada pergunta começa do zero).
    pub keep_history: bool,
    /// Guardar os anexos no cofre. Padrão (recomendado): não guardar.
    pub attachment_retention: Retention,
    /// Apagar conversas não fixadas sem uso há N dias (0 = nunca). Padrão: 90.
    #[ts(type = "number")]
    pub history_retention_days: i64,
    pub routines: RoutineSettings,
    /// Tempo-limite de uma resposta do agente por CLI, em minutos (15, 30, 60 ou 120).
    /// Coletas longas (MCPs extras, aparelho) pedem mais. Padrão: 15.
    #[ts(type = "number")]
    pub cli_timeout_minutes: i64,
}

/// Patch de [`AiSettings`]: só os campos enviados mudam.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AiSettingsPatch {
    #[ts(optional)]
    pub autonomous: Option<bool>,
    #[ts(optional)]
    pub mcp_allow_writes: Option<bool>,
    #[ts(optional)]
    pub keep_history: Option<bool>,
    #[ts(optional)]
    pub attachment_retention: Option<Retention>,
    #[ts(optional, type = "number")]
    pub history_retention_days: Option<i64>,
    #[ts(optional)]
    pub routines: Option<RoutinePatch>,
    #[ts(optional, type = "number")]
    pub cli_timeout_minutes: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RoutinePatch {
    #[ts(optional)]
    pub review: Option<bool>,
    #[ts(optional)]
    pub monthly_close: Option<bool>,
    #[ts(optional)]
    pub alerts: Option<bool>,
}

/// Opções do tempo-limite das respostas por CLI, em minutos.
pub const CLI_TIMEOUT_CHOICES: [i64; 4] = [15, 30, 60, 120];
const CLI_TIMEOUT_MINUTES: &str = "ai.cli_timeout_minutes";

/// Tempo-limite das respostas por CLI (minutos); valor fora das opções volta ao padrão.
pub fn cli_timeout_minutes(conn: &Connection) -> Result<i64> {
    let m: i64 = kv::get_or(conn, CLI_TIMEOUT_MINUTES, 15)?;
    Ok(if CLI_TIMEOUT_CHOICES.contains(&m) {
        m
    } else {
        15
    })
}

pub fn load(conn: &Connection) -> Result<AiSettings> {
    Ok(AiSettings {
        autonomous: kv::get_or(conn, crate::api::policy::AUTONOMOUS_SETTING, false)?,
        mcp_allow_writes: kv::get_or(conn, crate::mcp::ALLOW_WRITES_SETTING, false)?,
        keep_history: kv::get_or(conn, KEEP_HISTORY, true)?,
        attachment_retention: Retention::parse(&kv::get_or(
            conn,
            ATTACHMENT_RETENTION,
            Retention::None.as_str().to_string(),
        )?),
        history_retention_days: kv::get_or(
            conn,
            HISTORY_RETENTION_DAYS,
            DEFAULT_HISTORY_RETENTION_DAYS,
        )?,
        // Rotinas ligadas por padrão: sem rota própria para a tarefa elas são
        // determinísticas (nenhum dado sai do computador) e só criam propostas e alertas.
        cli_timeout_minutes: cli_timeout_minutes(conn)?,
        routines: RoutineSettings {
            review: kv::get_or(conn, ROUTINE_REVIEW, true)?,
            monthly_close: kv::get_or(conn, ROUTINE_MONTHLY_CLOSE, true)?,
            alerts: kv::get_or(conn, ROUTINE_ALERTS, true)?,
        },
    })
}

pub fn apply(conn: &Connection, p: &AiSettingsPatch) -> Result<AiSettings> {
    if let Some(d) = p.history_retention_days
        && !HISTORY_RETENTION_CHOICES.contains(&d)
    {
        return Err(Error::invalid(crate::i18n::t("nucleo-ia-retencao")));
    }
    if let Some(d) = p.history_retention_days {
        kv::set(conn, HISTORY_RETENTION_DAYS, &d)?;
    }
    if let Some(m) = p.cli_timeout_minutes {
        if !CLI_TIMEOUT_CHOICES.contains(&m) {
            return Err(Error::invalid(crate::i18n::t("nucleo-ia-tempo-cli")));
        }
        kv::set(conn, CLI_TIMEOUT_MINUTES, &m)?;
    }
    if let Some(r) = p.attachment_retention {
        kv::set(conn, ATTACHMENT_RETENTION, &r.as_str().to_string())?;
        // Vale também para o que já está guardado ("não guardar" apaga os bytes).
        attachments::apply_retention(conn, r)?;
    }
    let pairs: [(&str, Option<bool>); 6] = [
        (crate::api::policy::AUTONOMOUS_SETTING, p.autonomous),
        (crate::mcp::ALLOW_WRITES_SETTING, p.mcp_allow_writes),
        (KEEP_HISTORY, p.keep_history),
        (ROUTINE_REVIEW, p.routines.as_ref().and_then(|r| r.review)),
        (
            ROUTINE_MONTHLY_CLOSE,
            p.routines.as_ref().and_then(|r| r.monthly_close),
        ),
        (ROUTINE_ALERTS, p.routines.as_ref().and_then(|r| r.alerts)),
    ];
    for (k, v) in pairs {
        if let Some(v) = v {
            kv::set(conn, k, &v)?;
        }
    }
    load(conn)
}
