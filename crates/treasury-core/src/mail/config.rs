//! Contas e regras do e-mail de ingestão (docs/email-ingest.md): a pasta local vigiada ou a
//! caixa IMAP, as pastas e os remetentes de cada uma, as regras por prioridade e a retenção
//! (com o modo de teste de 7 dias antes de apagar qualquer coisa). A senha de app fica na
//! tabela `secret` (cifra por campo) e nunca sai daqui: a interface só vê se está cadastrada.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::body::{self, BodyTemplate};
use super::filter::{self, AccountFilter, DEFAULT_BLOCKED, RuleAction, RuleConditions, RuleSpec};
use crate::ai::secret::{self, SecretStatus};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};

/// Erro do catálogo com um argumento de texto.
fn invalid_with(key: &str, name: &'static str, value: String) -> Error {
    Error::invalid(t_args(key, &args(&[(name, value.into())])))
}
use crate::time::now_utc;
use crate::vault::DbKey;

/// Dias do modo de teste da exclusão.
pub const TEST_DAYS: i64 = 7;
/// Mensagens novas por dia e por conta.
pub const DAILY_MESSAGES: i64 = 200;
/// Leituras automáticas pela IA por dia e por conta.
pub const DAILY_AI: i64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "MailAccountKind")]
pub enum AccountKind {
    /// Pasta local vigiada (fase 1, sem credencial).
    Folder,
    Imap,
}

impl AccountKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::Imap => "imap",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "MailSecurity")]
pub enum Security {
    /// TLS desde o início (porta 993).
    #[default]
    Tls,
    /// STARTTLS (Proton Bridge local, porta 1143).
    Starttls,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MailboxKind {
    /// Caixa só para o Treasury (alimentada por encaminhamento): o recomendado.
    #[default]
    Dedicated,
    /// A caixa principal de alguém: nenhuma pasta permitida até o usuário escolher.
    Main,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RetentionMode {
    /// Nunca apaga (padrão).
    #[default]
    Never,
    /// Depois de importada ou repetida, passada a carência.
    AfterSuccess,
    /// Depois de X dias, em qualquer estado final.
    AfterDays,
    /// O que vier primeiro.
    Both,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LocalRetention {
    /// Move para a subpasta `Processados` (padrão).
    #[default]
    Move,
    /// Manda para a Lixeira do sistema.
    Trash,
}

/// Retenção: quando o Treasury tira do provedor (ou da pasta) o que já processou.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailRetention")]
pub struct Retention {
    #[serde(default)]
    pub mode: RetentionMode,
    /// Para `after_days` e `both`.
    #[serde(default = "default_days")]
    pub days: u32,
    /// Carência depois do sucesso (padrão: 7 dias).
    #[serde(default = "default_grace")]
    pub grace_days: u32,
    /// Pasta local: mover para `Processados` ou mandar para a Lixeira.
    #[serde(default)]
    pub local: LocalRetention,
    /// Gmail: apagar também da lixeira (desligado: a mensagem fica 30 dias lá).
    #[serde(default)]
    pub purge_trash: bool,
}

fn default_days() -> u32 {
    30
}

fn default_grace() -> u32 {
    7
}

impl Default for Retention {
    fn default() -> Self {
        Self {
            mode: RetentionMode::Never,
            days: default_days(),
            grace_days: default_grace(),
            local: LocalRetention::Move,
            purge_trash: false,
        }
    }
}

impl Retention {
    pub fn deletes(&self) -> bool {
        self.mode != RetentionMode::Never
    }

    fn validate(&self) -> Result<()> {
        if !(1..=3650).contains(&self.days) {
            return Err(Error::invalid(t("nucleo-email-config-erro-retencao-dias")));
        }
        if self.grace_days > 90 {
            return Err(Error::invalid(t("nucleo-email-config-erro-carencia")));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RetentionState {
    /// Nada é apagado.
    Off,
    /// Modo de teste: só mostra o que teria apagado.
    Testing,
    /// O teste acabou: apaga só depois da confirmação explícita (até lá, continua em teste).
    AwaitingConfirmation,
    Active,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "MailAiMode")]
pub enum AiMode {
    /// Nunca (padrão).
    #[default]
    Never,
    /// Só quando o usuário pede, na fila de revisão.
    OnRequest,
    /// Sozinha, até 10 por dia (o resultado é sempre proposta).
    Auto,
}

impl AiMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::OnRequest => "on_request",
            Self::Auto => "auto",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "on_request" => Self::OnRequest,
            "auto" => Self::Auto,
            _ => Self::Never,
        }
    }
}

/// Resultado da última busca da conta.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailSyncStatus {
    pub at: String,
    pub ok: bool,
    pub message: String,
    /// Mensagens novas vistas.
    #[ts(type = "number")]
    pub seen: u32,
    /// Passaram nos filtros e foram para a fila (ou aplicadas).
    #[ts(type = "number")]
    pub passed: u32,
    /// Recusadas pelos filtros (só a contagem fica).
    #[ts(type = "number")]
    pub rejected: u32,
    /// Aplicadas sozinhas.
    #[ts(type = "number")]
    pub applied: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailAccount {
    pub id: Id,
    pub name: String,
    pub kind: AccountKind,
    pub folder_path: Option<String>,
    /// Provedor do catálogo (`gmail`, `icloud`…) ou `manual`.
    pub provider: Option<String>,
    pub host: Option<String>,
    #[ts(type = "number | null")]
    pub port: Option<u16>,
    pub security: Security,
    pub username: Option<String>,
    /// SHA-256 do certificado fixado (só Proton Bridge local).
    pub pinned_cert: Option<String>,
    /// authserv-id do provedor no Authentication-Results.
    pub authserv_id: Option<String>,
    pub mailbox_kind: MailboxKind,
    pub allowed_folders: Vec<String>,
    pub blocked_folders: Vec<String>,
    pub allow_senders: Vec<String>,
    pub block_senders: Vec<String>,
    /// Endereços da família (o padrão `membros` das regras).
    pub family_addresses: Vec<String>,
    /// Sem regra casada: `ignore` (padrão) ou `review`.
    pub default_action: RuleAction,
    pub retention: Retention,
    pub retention_test_until: Option<String>,
    pub retention_confirmed: bool,
    pub retention_state: RetentionState,
    pub keep_original: bool,
    #[ts(type = "number")]
    pub interval_minutes: u32,
    pub enabled: bool,
    /// Backup restaurado: confirme a conexão (Testar) antes de buscar de novo.
    pub needs_reconfirm: bool,
    /// Buscando agora.
    pub syncing: bool,
    /// Senha de app (só IMAP): se está cadastrada, nunca o valor.
    pub password: Option<SecretStatus>,
    pub status: Option<MailSyncStatus>,
    /// Mensagens esperando revisão.
    #[ts(type = "number")]
    pub pending: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailAccountInput {
    /// Vazio: conta nova.
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<Id>,
    pub name: String,
    pub kind: Option<AccountKind>,
    #[serde(default)]
    #[ts(optional)]
    pub folder_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub provider: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub host: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub port: Option<u16>,
    #[serde(default)]
    #[ts(optional)]
    pub security: Option<Security>,
    #[serde(default)]
    #[ts(optional)]
    pub username: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub pinned_cert: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub authserv_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub mailbox_kind: Option<MailboxKind>,
    #[serde(default)]
    #[ts(optional)]
    pub allowed_folders: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub blocked_folders: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub allow_senders: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub block_senders: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub family_addresses: Option<Vec<String>>,
    #[serde(default)]
    #[ts(optional)]
    pub default_action: Option<RuleAction>,
    #[serde(default)]
    #[ts(optional)]
    pub retention: Option<Retention>,
    #[serde(default)]
    #[ts(optional)]
    pub keep_original: Option<bool>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub interval_minutes: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailRule {
    pub id: Id,
    /// Nula: vale para todas as contas.
    pub account_id: Option<Id>,
    pub name: String,
    #[ts(type = "number")]
    pub priority: i64,
    pub enabled: bool,
    pub conditions: RuleConditions,
    pub action: RuleAction,
    pub ai: AiMode,
    /// Retenção própria (sobrescreve a da conta).
    pub retention: Option<Retention>,
    /// Modelo do corpo do e-mail (aviso que os layouts prontos não leem).
    pub template: Option<BodyTemplate>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailRuleInput {
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<Id>,
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    pub name: String,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub priority: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub conditions: RuleConditions,
    #[serde(default)]
    #[ts(optional)]
    pub action: Option<RuleAction>,
    #[serde(default)]
    #[ts(optional)]
    pub ai: Option<AiMode>,
    #[serde(default)]
    #[ts(optional)]
    pub retention: Option<Retention>,
    #[serde(default)]
    #[ts(optional)]
    pub template: Option<BodyTemplate>,
}

pub fn secret_id(account_id: &str) -> String {
    format!("mail:{account_id}")
}

pub fn status_key(account_id: &str) -> String {
    format!("mail.status:{account_id}")
}

fn json_list(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

const ACCOUNT_COLS: &str =
    "id, name, kind, folder_path, provider, host, port, security, username, pinned_cert,
    authserv_id, mailbox_kind, allowed_folders, blocked_folders, allow_senders, block_senders,
    family_addresses, default_action, retention, retention_test_until, retention_confirmed,
    keep_original, interval_minutes, enabled, needs_reconfirm, created_at, updated_at";

fn account_row(r: &Row<'_>) -> rusqlite::Result<MailAccount> {
    let kind: String = r.get(2)?;
    let security: String = r.get(7)?;
    let mailbox: String = r.get(11)?;
    let default_action: String = r.get(17)?;
    let retention: String = r.get(18)?;
    Ok(MailAccount {
        id: r.get(0)?,
        name: r.get(1)?,
        kind: if kind == "imap" {
            AccountKind::Imap
        } else {
            AccountKind::Folder
        },
        folder_path: r.get(3)?,
        provider: r.get(4)?,
        host: r.get(5)?,
        port: r
            .get::<_, Option<i64>>(6)?
            .and_then(|p| u16::try_from(p).ok()),
        security: if security == "starttls" {
            Security::Starttls
        } else {
            Security::Tls
        },
        username: r.get(8)?,
        pinned_cert: r.get(9)?,
        authserv_id: r.get(10)?,
        mailbox_kind: if mailbox == "main" {
            MailboxKind::Main
        } else {
            MailboxKind::Dedicated
        },
        allowed_folders: json_list(&r.get::<_, String>(12)?),
        blocked_folders: json_list(&r.get::<_, String>(13)?),
        allow_senders: json_list(&r.get::<_, String>(14)?),
        block_senders: json_list(&r.get::<_, String>(15)?),
        family_addresses: json_list(&r.get::<_, String>(16)?),
        default_action: RuleAction::parse(&default_action),
        retention: serde_json::from_str(&retention).unwrap_or_default(),
        retention_test_until: r.get(19)?,
        retention_confirmed: r.get::<_, i64>(20)? != 0,
        retention_state: RetentionState::Off,
        keep_original: r.get::<_, i64>(21)? != 0,
        interval_minutes: r.get::<_, i64>(22)?.clamp(5, 120) as u32,
        enabled: r.get::<_, i64>(23)? != 0,
        needs_reconfirm: r.get::<_, i64>(24)? != 0,
        syncing: false,
        password: None,
        status: None,
        pending: 0,
        created_at: r.get(25)?,
        updated_at: r.get(26)?,
    })
}

/// Estado da exclusão: desligada, em teste, esperando confirmação ou ativa.
pub fn retention_state(conn: &Connection, acc: &MailAccount, now: &str) -> Result<RetentionState> {
    if !deletes_anything(conn, acc)? {
        return Ok(RetentionState::Off);
    }
    if acc.retention_confirmed {
        return Ok(RetentionState::Active);
    }
    Ok(match acc.retention_test_until.as_deref() {
        Some(until) if until > now => RetentionState::Testing,
        _ => RetentionState::AwaitingConfirmation,
    })
}

/// A conta ou alguma regra dela tem exclusão ligada.
fn deletes_anything(conn: &Connection, acc: &MailAccount) -> Result<bool> {
    if acc.retention.deletes() {
        return Ok(true);
    }
    Ok(rules_for(conn, &acc.id)?
        .iter()
        .any(|r| r.retention.as_ref().is_some_and(Retention::deletes)))
}

fn decorate(conn: &Connection, key: Option<&DbKey>, mut acc: MailAccount) -> Result<MailAccount> {
    let now = now_utc();
    acc.retention_state = retention_state(conn, &acc, &now)?;
    acc.syncing = super::sync::is_running(&acc.id);
    if acc.kind == AccountKind::Imap
        && let Some(k) = key
    {
        acc.password = Some(secret::status(conn, k, &secret_id(&acc.id))?);
    }
    acc.status = crate::settings::get(conn, &status_key(&acc.id))?;
    acc.pending = conn.query_row(
        "SELECT COUNT(*) FROM mail_message WHERE account_id = ?1 AND status = 'awaiting_review'",
        [&acc.id],
        |r| r.get(0),
    )?;
    Ok(acc)
}

pub fn list_accounts(conn: &Connection, key: Option<&DbKey>) -> Result<Vec<MailAccount>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {ACCOUNT_COLS} FROM mail_account ORDER BY created_at, name"
    ))?;
    let rows: Vec<MailAccount> = stmt
        .query_map([], account_row)?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|a| decorate(conn, key, a)).collect()
}

pub fn get_account(conn: &Connection, key: Option<&DbKey>, id: &str) -> Result<MailAccount> {
    let acc = conn
        .query_row(
            &format!("SELECT {ACCOUNT_COLS} FROM mail_account WHERE id = ?1"),
            [id],
            account_row,
        )
        .optional()?
        .ok_or_else(|| {
            Error::not_found(t_args(
                "nucleo-email-ref-conta",
                &args(&[("id", id.to_string().into())]),
            ))
        })?;
    decorate(conn, key, acc)
}

fn clean_list(v: Option<Vec<String>>) -> Option<Vec<String>> {
    v.map(|l| {
        let mut out: Vec<String> = vec![];
        for s in l {
            let t = s.trim().to_string();
            if !t.is_empty() && !out.contains(&t) {
                out.push(t);
            }
        }
        out
    })
}

fn check_senders(list: &[String]) -> Result<()> {
    filter::validate(&RuleConditions {
        from: list.to_vec(),
        ..Default::default()
    })
}

/// Pasta local: absoluta, existente e fora dos dados do Treasury.
fn check_folder(path: &str, forbidden: &Path) -> Result<String> {
    let p = Path::new(path.trim());
    if !p.is_absolute() {
        return Err(Error::invalid(t("nucleo-email-config-erro-pasta-caminho")));
    }
    let real = p.canonicalize().map_err(|_| {
        invalid_with(
            "nucleo-email-config-erro-pasta-nao-encontrada",
            "pasta",
            p.display().to_string(),
        )
    })?;
    if !real.is_dir() {
        return Err(Error::invalid(t("nucleo-email-config-erro-nao-pasta")));
    }
    if let Ok(f) = forbidden.canonicalize()
        && real.starts_with(&f)
    {
        return Err(Error::invalid(t("nucleo-email-config-erro-pasta-treasury")));
    }
    Ok(real.to_string_lossy().into_owned())
}

/// Liga o modo de teste quando a exclusão é ligada; desliga tudo quando ela é desligada.
pub fn sync_test_mode(conn: &Connection, account_id: &str) -> Result<()> {
    let acc = get_account(conn, None, account_id)?;
    if !deletes_anything(conn, &acc)? {
        conn.execute(
            "UPDATE mail_account SET retention_test_until = NULL, retention_confirmed = 0 WHERE id = ?1",
            [account_id],
        )?;
    } else if acc.retention_test_until.is_none() && !acc.retention_confirmed {
        let until = (chrono::Utc::now() + chrono::Duration::days(TEST_DAYS))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        conn.execute(
            "UPDATE mail_account SET retention_test_until = ?2 WHERE id = ?1",
            params![account_id, until],
        )?;
    }
    Ok(())
}

/// Cria ou altera uma conta. Trocar o servidor ou o usuário apaga a senha cadastrada (ela
/// nunca vai para outro servidor sem o usuário digitar de novo).
pub fn save_account(conn: &Connection, a: &MailAccountInput, forbidden: &Path) -> Result<Id> {
    let name = a.name.trim();
    if name.is_empty() {
        return Err(Error::invalid(t("nucleo-email-config-erro-nome-conta")));
    }
    let old = match &a.id {
        Some(id) => Some(get_account(conn, None, id)?),
        None => None,
    };
    let kind = a
        .kind
        .or(old.as_ref().map(|o| o.kind))
        .ok_or_else(|| Error::invalid(t("nucleo-email-config-erro-tipo")))?;
    if old.as_ref().is_some_and(|o| o.kind != kind) {
        return Err(Error::invalid(t("nucleo-email-config-erro-tipo-fixo")));
    }
    let mailbox_kind = a
        .mailbox_kind
        .or(old.as_ref().map(|o| o.mailbox_kind))
        .unwrap_or_default();
    let (mut folder_path, mut provider, mut host, mut port, mut security, mut username) =
        (None, None, None, None, Security::Tls, None);
    let mut pinned = None;
    let mut authserv = None;
    match kind {
        AccountKind::Folder => {
            let p = a
                .folder_path
                .as_deref()
                .filter(|p| !p.trim().is_empty())
                .or(old.as_ref().and_then(|o| o.folder_path.as_deref()))
                .ok_or_else(|| Error::invalid(t("nucleo-email-config-erro-pasta-escolha")))?;
            folder_path = Some(check_folder(p, forbidden)?);
        }
        AccountKind::Imap => {
            // Outro provedor: servidor, porta e authserv-id vêm dele, não da conta antiga.
            let changed =
                a.provider.is_some() && old.as_ref().is_some_and(|o| o.provider != a.provider);
            let o = old.as_ref().filter(|_| !changed);
            let prov = super::providers::find(
                a.provider
                    .as_deref()
                    .or(o.and_then(|o| o.provider.as_deref()))
                    .unwrap_or("manual"),
            );
            provider = Some(prov.map_or("manual", |p| p.id).to_string());
            host = a
                .host
                .as_deref()
                .map(str::trim)
                .filter(|h| !h.is_empty())
                .map(str::to_ascii_lowercase)
                .or_else(|| o.and_then(|o| o.host.clone()))
                .or_else(|| prov.map(|p| p.host.to_string()));
            let h = host
                .as_deref()
                .ok_or_else(|| Error::invalid(t("nucleo-email-config-erro-servidor")))?;
            if h.contains(['/', ' ', '@', ':']) {
                return Err(Error::invalid(t(
                    "nucleo-email-config-erro-servidor-invalido",
                )));
            }
            port = a
                .port
                .or(o.and_then(|o| o.port))
                .or(prov.map(|p| p.port))
                .or(Some(993));
            if port == Some(0) {
                return Err(Error::invalid(t("nucleo-email-config-erro-porta")));
            }
            security = a
                .security
                .or(o.map(|o| o.security))
                .or(prov.map(|p| p.security))
                .unwrap_or_default();
            username = a
                .username
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())
                .map(str::to_string)
                .or_else(|| old.as_ref().and_then(|o| o.username.clone()));
            if username.is_none() {
                return Err(Error::invalid(t("nucleo-email-config-erro-usuario")));
            }
            let local = matches!(h, "127.0.0.1" | "localhost" | "::1");
            pinned = a
                .pinned_cert
                .as_deref()
                .map(|c| c.trim().replace(':', "").to_ascii_lowercase())
                .filter(|c| !c.is_empty())
                .or_else(|| o.and_then(|o| o.pinned_cert.clone()));
            if let Some(c) = &pinned {
                if !local {
                    return Err(Error::invalid(t(
                        "nucleo-email-config-erro-certificado-local",
                    )));
                }
                if c.len() != 64 || !c.chars().all(|x| x.is_ascii_hexdigit()) {
                    return Err(Error::invalid(t(
                        "nucleo-email-config-erro-certificado-formato",
                    )));
                }
            }
            if security == Security::Starttls && !local {
                return Err(Error::invalid(t("nucleo-email-config-erro-starttls")));
            }
            authserv = a
                .authserv_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_ascii_lowercase)
                .or_else(|| o.and_then(|o| o.authserv_id.clone()))
                .or_else(|| prov.and_then(|p| p.authserv_id.map(str::to_string)));
        }
    }
    let allowed = clean_list(a.allowed_folders.clone())
        .or_else(|| old.as_ref().map(|o| o.allowed_folders.clone()))
        .unwrap_or_else(|| match mailbox_kind {
            MailboxKind::Dedicated => vec!["INBOX".into()],
            MailboxKind::Main => vec![],
        });
    let blocked = clean_list(a.blocked_folders.clone())
        .or_else(|| old.as_ref().map(|o| o.blocked_folders.clone()))
        .unwrap_or_else(|| DEFAULT_BLOCKED.iter().map(|s| s.to_string()).collect());
    for f in allowed.iter().chain(&blocked) {
        if f.len() > 200 || f.chars().any(char::is_control) {
            return Err(invalid_with(
                "nucleo-email-config-erro-pasta",
                "pasta",
                f.clone(),
            ));
        }
    }
    let allow_senders = clean_list(a.allow_senders.clone())
        .or_else(|| old.as_ref().map(|o| o.allow_senders.clone()))
        .unwrap_or_default();
    let block_senders = clean_list(a.block_senders.clone())
        .or_else(|| old.as_ref().map(|o| o.block_senders.clone()))
        .unwrap_or_default();
    check_senders(&allow_senders)?;
    check_senders(&block_senders)?;
    let family: Vec<String> = clean_list(a.family_addresses.clone())
        .or_else(|| old.as_ref().map(|o| o.family_addresses.clone()))
        .unwrap_or_default()
        .into_iter()
        .map(|f| f.to_ascii_lowercase())
        .collect();
    if let Some(bad) = family.iter().find(|f| !f.contains('@') || f.contains(' ')) {
        return Err(invalid_with(
            "nucleo-email-config-erro-endereco",
            "endereco",
            bad.clone(),
        ));
    }
    let default_action = match a
        .default_action
        .or(old.as_ref().map(|o| o.default_action))
        // A pasta local é do próprio usuário: o que cai nela vai para a revisão.
        .unwrap_or(if kind == AccountKind::Folder {
            RuleAction::Review
        } else {
            RuleAction::Ignore
        }) {
        RuleAction::Review => RuleAction::Review,
        _ => RuleAction::Ignore,
    };
    let retention = a
        .retention
        .clone()
        .or(old.as_ref().map(|o| o.retention.clone()))
        .unwrap_or_default();
    retention.validate()?;
    let interval = a
        .interval_minutes
        .or(old.as_ref().map(|o| o.interval_minutes))
        .unwrap_or(15);
    if !(5..=120).contains(&interval) {
        return Err(Error::invalid(t("nucleo-email-config-erro-intervalo")));
    }
    let keep = a
        .keep_original
        .or(old.as_ref().map(|o| o.keep_original))
        .unwrap_or(true);
    let enabled = a
        .enabled
        .or(old.as_ref().map(|o| o.enabled))
        .unwrap_or(true);
    let now = now_utc();
    let j = |v: &Vec<String>| serde_json::to_string(v).unwrap_or_else(|_| "[]".into());
    let id = match &old {
        Some(o) => {
            // Outro servidor ou outro usuário: a senha cadastrada não vai junto.
            if o.host != host || o.username != username || o.port != port {
                secret::delete(conn, &secret_id(&o.id))?;
                conn.execute(
                    "DELETE FROM mail_folder_state WHERE account_id = ?1",
                    [&o.id],
                )?;
            }
            conn.execute(
                "UPDATE mail_account SET name = ?2, folder_path = ?3, provider = ?4, host = ?5, port = ?6,
                    security = ?7, username = ?8, pinned_cert = ?9, authserv_id = ?10, mailbox_kind = ?11,
                    allowed_folders = ?12, blocked_folders = ?13, allow_senders = ?14, block_senders = ?15,
                    family_addresses = ?16, default_action = ?17, retention = ?18, keep_original = ?19,
                    interval_minutes = ?20, enabled = ?21, updated_at = ?22 WHERE id = ?1",
                params![
                    o.id,
                    name,
                    folder_path,
                    provider,
                    host,
                    port,
                    if security == Security::Starttls { "starttls" } else { "tls" },
                    username,
                    pinned,
                    authserv,
                    if mailbox_kind == MailboxKind::Main { "main" } else { "dedicated" },
                    j(&allowed),
                    j(&blocked),
                    j(&allow_senders),
                    j(&block_senders),
                    j(&family),
                    default_action.as_str(),
                    serde_json::to_string(&retention)?,
                    keep as i64,
                    interval as i64,
                    enabled as i64,
                    now
                ],
            )?;
            o.id.clone()
        }
        None => {
            let id = new_id();
            conn.execute(
                &format!(
                    "INSERT INTO mail_account ({ACCOUNT_COLS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                        ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, NULL, 0, ?20, ?21, ?22, 0, ?23, ?23)"
                ),
                params![
                    id,
                    name,
                    kind.as_str(),
                    folder_path,
                    provider,
                    host,
                    port,
                    if security == Security::Starttls { "starttls" } else { "tls" },
                    username,
                    pinned,
                    authserv,
                    if mailbox_kind == MailboxKind::Main { "main" } else { "dedicated" },
                    j(&allowed),
                    j(&blocked),
                    j(&allow_senders),
                    j(&block_senders),
                    j(&family),
                    default_action.as_str(),
                    serde_json::to_string(&retention)?,
                    keep as i64,
                    interval as i64,
                    enabled as i64,
                    now
                ],
            )?;
            id
        }
    };
    sync_test_mode(conn, &id)?;
    Ok(id)
}

pub fn delete_account(conn: &Connection, id: &str) -> Result<()> {
    get_account(conn, None, id)?;
    secret::delete(conn, &secret_id(id))?;
    crate::settings::delete(conn, &status_key(id))?;
    conn.execute("DELETE FROM mail_account WHERE id = ?1", [id])?;
    Ok(())
}

/// Cadastra a senha de app (só pela interface; nunca devolvida).
pub fn set_password(conn: &Connection, key: &DbKey, id: &str, value: &str) -> Result<SecretStatus> {
    let acc = get_account(conn, None, id)?;
    if acc.kind != AccountKind::Imap {
        return Err(Error::invalid(t(
            "nucleo-email-config-erro-pasta-sem-senha",
        )));
    }
    let v = value.trim();
    if v.len() > 512 {
        return Err(Error::invalid(t("nucleo-email-config-erro-senha-longa")));
    }
    secret::set(conn, key, &secret_id(id), v)
}

/// Confirma a exclusão depois do modo de teste.
pub fn confirm_retention(conn: &Connection, id: &str) -> Result<()> {
    let acc = get_account(conn, None, id)?;
    match acc.retention_state {
        RetentionState::AwaitingConfirmation => {}
        RetentionState::Testing => {
            return Err(Error::conflict(t(
                "nucleo-email-config-erro-exclusao-teste",
            )));
        }
        RetentionState::Off | RetentionState::Active => {
            return Err(Error::conflict(t(
                "nucleo-email-config-erro-exclusao-nenhuma",
            )));
        }
    }
    conn.execute(
        "UPDATE mail_account SET retention_confirmed = 1, updated_at = ?2 WHERE id = ?1",
        params![id, now_utc()],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Regras

const RULE_COLS: &str = "id, account_id, name, priority, enabled, conditions, action, ai, retention, template, created_at, updated_at";

fn rule_row(r: &Row<'_>) -> rusqlite::Result<MailRule> {
    let conditions: String = r.get(5)?;
    let action: String = r.get(6)?;
    let ai: String = r.get(7)?;
    let retention: Option<String> = r.get(8)?;
    let template: Option<String> = r.get(9)?;
    Ok(MailRule {
        id: r.get(0)?,
        account_id: r.get(1)?,
        name: r.get(2)?,
        priority: r.get(3)?,
        enabled: r.get::<_, i64>(4)? != 0,
        conditions: serde_json::from_str(&conditions).unwrap_or_default(),
        action: RuleAction::parse(&action),
        ai: AiMode::parse(&ai),
        retention: retention.and_then(|s| serde_json::from_str(&s).ok()),
        template: template.and_then(|s| serde_json::from_str(&s).ok()),
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
    })
}

/// Todas as regras (de uma conta e as gerais, ou todas), por prioridade.
pub fn list_rules(conn: &Connection, account_id: Option<&str>) -> Result<Vec<MailRule>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {RULE_COLS} FROM mail_rule WHERE ?1 IS NULL OR account_id = ?1 OR account_id IS NULL
          ORDER BY priority, created_at"
    ))?;
    Ok(stmt
        .query_map([account_id], rule_row)?
        .collect::<rusqlite::Result<_>>()?)
}

/// As regras ligadas que valem para a conta, por prioridade.
pub fn rules_for(conn: &Connection, account_id: &str) -> Result<Vec<MailRule>> {
    Ok(list_rules(conn, Some(account_id))?
        .into_iter()
        .filter(|r| r.enabled)
        .collect())
}

pub fn get_rule(conn: &Connection, id: &str) -> Result<MailRule> {
    conn.query_row(
        &format!("SELECT {RULE_COLS} FROM mail_rule WHERE id = ?1"),
        [id],
        rule_row,
    )
    .optional()?
    .ok_or_else(|| {
        Error::not_found(t_args(
            "nucleo-email-ref-regra",
            &args(&[("id", id.to_string().into())]),
        ))
    })
}

pub fn save_rule(conn: &Connection, a: &MailRuleInput) -> Result<Id> {
    let name = a.name.trim();
    if name.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-regra-erro-nome")));
    }
    if name.chars().count() > 80 {
        return Err(Error::invalid(t("nucleo-email-config-erro-nome-regra")));
    }
    filter::validate(&a.conditions)?;
    if let Some(t) = &a.template {
        body::validate_template(t)?;
    }
    if let Some(r) = &a.retention {
        r.validate()?;
    }
    if let Some(acc) = &a.account_id {
        get_account(conn, None, acc)?;
    }
    let old = match &a.id {
        Some(id) => Some(get_rule(conn, id)?),
        None => None,
    };
    let priority = a
        .priority
        .or(old.as_ref().map(|o| o.priority))
        .unwrap_or(100)
        .clamp(0, 10_000);
    let action = a
        .action
        .or(old.as_ref().map(|o| o.action))
        .unwrap_or(RuleAction::Review);
    let ai = a.ai.or(old.as_ref().map(|o| o.ai)).unwrap_or_default();
    let enabled = a
        .enabled
        .or(old.as_ref().map(|o| o.enabled))
        .unwrap_or(true);
    let now = now_utc();
    let conditions = serde_json::to_string(&a.conditions)?;
    let retention = a
        .retention
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let template = a
        .template
        .as_ref()
        .filter(|t| !t.pattern.trim().is_empty())
        .map(serde_json::to_string)
        .transpose()?;
    let id = match old {
        Some(o) => {
            conn.execute(
                "UPDATE mail_rule SET account_id = ?2, name = ?3, priority = ?4, enabled = ?5, conditions = ?6,
                    action = ?7, ai = ?8, retention = ?9, template = ?10, updated_at = ?11 WHERE id = ?1",
                params![
                    o.id,
                    a.account_id,
                    name,
                    priority,
                    enabled as i64,
                    conditions,
                    action.as_str(),
                    ai.as_str(),
                    retention,
                    template,
                    now
                ],
            )?;
            o.id
        }
        None => {
            let id = new_id();
            conn.execute(
                &format!("INSERT INTO mail_rule ({RULE_COLS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)"),
                params![
                    id,
                    a.account_id,
                    name,
                    priority,
                    enabled as i64,
                    conditions,
                    action.as_str(),
                    ai.as_str(),
                    retention,
                    template,
                    now
                ],
            )?;
            id
        }
    };
    refresh_test_modes(conn)?;
    Ok(id)
}

pub fn delete_rule(conn: &Connection, id: &str) -> Result<()> {
    get_rule(conn, id)?;
    conn.execute("DELETE FROM mail_rule WHERE id = ?1", [id])?;
    refresh_test_modes(conn)
}

/// A retenção de uma regra pode ligar (ou desligar) a exclusão de várias contas.
fn refresh_test_modes(conn: &Connection) -> Result<()> {
    let ids: Vec<String> = conn
        .prepare("SELECT id FROM mail_account")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for id in ids {
        sync_test_mode(conn, &id)?;
    }
    Ok(())
}

/// O que o filtro precisa da conta.
pub fn account_filter(acc: &MailAccount) -> AccountFilter {
    AccountFilter {
        allowed_folders: acc.allowed_folders.clone(),
        blocked_folders: acc.blocked_folders.clone(),
        allow_senders: acc.allow_senders.clone(),
        block_senders: acc.block_senders.clone(),
        family: acc.family_addresses.clone(),
        default_review: acc.default_action == RuleAction::Review,
    }
}

pub fn specs(rules: &[MailRule]) -> Vec<RuleSpec> {
    rules
        .iter()
        .map(|r| RuleSpec {
            id: r.id.clone(),
            name: r.name.clone(),
            conditions: r.conditions.clone(),
            action: r.action,
        })
        .collect()
}
