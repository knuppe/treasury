//! Atualização automática pelas releases do GitHub (docs/auto-update.md), com código nosso:
//! o `latest.json` (`manifest.rs`), a assinatura minisign (`minisign.rs`), a da YubiKey
//! (`update_sig.rs`) e a instalação de cada pacote (`install.rs`).
//!
//! - Só no build de release e só com o endereço do `latest.json` definido na compilação
//!   (`TREASURY_UPDATE_ENDPOINT`, que o scripts/release.sh tira de `TREASURY_UPDATE_REPO`).
//!   Sem ele, nenhuma conexão sai daqui.
//! - A versão anunciada precisa ser mais nova que a instalada. O pacote só vale com as duas
//!   assinaturas: a minisign da chave embutida (`update-minisign.pub`, a privada fica fora do
//!   repositório), feita para a mesma versão anunciada (contra rebaixar para um pacote antigo
//!   e assinado), e a de uma das YubiKeys do mantenedor (`update-keys.pub`). Sem chaves de
//!   YubiKey embutidas, nenhuma atualização é aceita.
//! - Vale para todos os pacotes: o `.app` do macOS, o AppImage, o `.deb` e o `.rpm` do Linux e
//!   o instalador NSIS ou o `.msi` do Windows. A entrada do `latest.json` é a
//!   `<sistema>-<arquitetura>-<pacote>` do pacote instalado.
//! - Modo no `config.toml` (Ajustes › Atualizações): baixar sozinho (padrão), só avisar ou
//!   desligado. Procura ao abrir e a cada 6 h; no "só avisar" a pessoa pede o download. Instalar sempre espera a pessoa (`update_install`), que vê a
//!   versão e as notas antes; o app reinicia e o cofre pede o desbloqueio de novo.
//! - Nada sobre a pessoa ou o cofre vai junto: só o pedido do `latest.json` e o download do
//!   pacote, só por https, com o agente de usuário `Treasury` (sem versão nem identificador).

mod install;
mod manifest;
mod minisign;

use std::io::Read;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Url};

use crate::AppState;
use manifest::{Installer, Release};
use treasury_core::config::UpdateMode;

/// Endereço do `latest.json`, fixado na compilação do release.
const ENDPOINT: Option<&str> = option_env!("TREASURY_UPDATE_ENDPOINT");
const FIRST_CHECK: Duration = Duration::from_secs(60);
const EVERY: Duration = Duration::from_secs(6 * 60 * 60);
const USER_AGENT: &str = "Treasury";
/// Limites de tamanho: o `latest.json` e o maior pacote (o `.app` tem perto de 100 MB).
const MAX_MANIFEST: u64 = 1024 * 1024;
const MAX_PACKAGE: u64 = 1024 * 1024 * 1024;

/// Estado da atualização, para Ajustes e para o aviso de "pronta para instalar".
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateStatus {
    /// Este app sabe onde procurar (release com o endereço definido).
    pub configured: bool,
    pub current: String,
    pub checking: bool,
    /// UTC RFC 3339 da última procura que terminou.
    pub last_check: Option<String>,
    pub available: Option<AvailableUpdate>,
    /// Baixada e com a assinatura conferida: falta só a pessoa confirmar.
    pub ready: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AvailableUpdate {
    pub version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
}

#[derive(Default)]
pub struct UpdateState {
    inner: tokio::sync::Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    status: UpdateStatus,
    pending: Option<(Release, Vec<u8>)>,
}

fn endpoint() -> Option<Url> {
    if cfg!(debug_assertions) {
        return None;
    }
    ENDPOINT
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .and_then(|e| Url::parse(e).ok())
        .filter(|u| u.scheme() == "https")
}

fn mode(app: &AppHandle) -> UpdateMode {
    let state = app.state::<AppState>();
    treasury_core::config::update_settings(&state.api.paths().config_path).mode
}

async fn status_of(app: &AppHandle) -> UpdateStatus {
    let state = app.state::<UpdateState>();
    let mut s = state.inner.lock().await.status.clone();
    s.configured = endpoint().is_some();
    s.current = app.package_info().version.to_string();
    s
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .https_only(true)
        .user_agent(USER_AGENT)
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_global(Some(Duration::from_secs(30 * 60)))
        .http_status_as_error(false)
        .build()
        .into()
}

/// Lê o `latest.json` e escolhe o pacote desta instalação.
fn fetch_release(
    agent: &ureq::Agent,
    url: &Url,
    current: &semver::Version,
) -> Result<Option<Release>, String> {
    let mut resp = agent
        .get(url.as_str())
        .header("Accept", "application/json")
        .call()
        .map_err(|e| e.to_string())?;
    match resp.status().as_u16() {
        // Sem release publicada ainda (o GitHub responde 404): nada novo, não é erro.
        404 | 204 => return Ok(None),
        200 => {}
        s => return Err(format!("latest.json: HTTP {s}")),
    }
    let body = resp
        .body_mut()
        .with_config()
        .limit(MAX_MANIFEST)
        .read_to_vec()
        .map_err(|e| e.to_string())?;
    let (os, arch) = manifest::os()
        .zip(manifest::arch())
        .ok_or("sistema sem atualização automática")?;
    let targets = manifest::targets(os, arch, Installer::current());
    manifest::parse(&body, current, &targets)
}

/// Baixa o pacote, com a barra nas tarefas em segundo plano.
fn fetch_package(agent: &ureq::Agent, release: &Release) -> Result<Vec<u8>, String> {
    use treasury_core::tasks::{Task, TaskKind, TaskOrigin, TaskStage};
    let task = Task::start(TaskKind::UpdateDownload, TaskOrigin::Auto, false);
    task.stage(TaskStage::Download);
    task.detail(format!("Treasury {}", release.version));
    let got = (|| {
        let mut resp = agent
            .get(release.url.as_str())
            .header("Accept", "application/octet-stream")
            .call()
            .map_err(|e| e.to_string())?;
        if resp.status().as_u16() != 200 {
            return Err(format!("pacote: HTTP {}", resp.status().as_u16()));
        }
        let total = resp
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        if total.is_some_and(|t| t > MAX_PACKAGE) {
            return Err("pacote grande demais".into());
        }
        let mut reader = resp.body_mut().with_config().limit(MAX_PACKAGE).reader();
        let mut out = Vec::with_capacity(total.unwrap_or(0) as usize);
        let mut buf = vec![0u8; 256 * 1024];
        loop {
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
            task.bytes(out.len() as u64, total);
        }
        Ok(out)
    })();
    task.finish(
        &got.as_ref()
            .map(|_| ())
            .map_err(|e| treasury_core::error::Error::Other(e.clone())),
    );
    got
}

/// O que a procura achou.
enum Found {
    /// Nenhuma versão nova, ou nenhuma release publicada ainda.
    Nothing,
    /// Versão nova, ainda não baixada (modo "só avisar").
    Available(Box<Release>),
    /// Baixada e com as duas assinaturas conferidas.
    Ready(Box<Release>, Vec<u8>),
}

/// Procura e, com `download`, baixa e confere as duas assinaturas (rede: fora do runtime
/// assíncrono).
fn find_and_verify(url: &Url, current: &semver::Version, download: bool) -> Result<Found, String> {
    let agent = agent();
    let Some(release) = fetch_release(&agent, url, current)? else {
        return Ok(Found::Nothing);
    };
    // Antes de baixar: o pacote só vem das releases deste repositório. Um `latest.json`
    // adulterado não faz o app buscar em outro lugar (e expor o endereço da pessoa).
    if !manifest::same_releases(url, &release.url) {
        return Err(format!(
            "pacote fora das releases do Treasury: {}",
            release.url
        ));
    }
    if !download {
        return Ok(Found::Available(Box::new(release)));
    }
    let bytes = fetch_package(&agent, &release)?;
    minisign_check(&release, &bytes)?;
    hardware_check(&release, &bytes)?;
    Ok(Found::Ready(Box::new(release), bytes))
}

/// Procura e, se houver versão nova, avisa (`update://available`) ou, com `download`, baixa,
/// confere e avisa que está pronta (`update://ready`).
async fn check(app: &AppHandle, download: bool) -> UpdateStatus {
    let Some(url) = endpoint() else {
        return status_of(app).await;
    };
    let state = app.state::<UpdateState>();
    {
        let mut g = state.inner.lock().await;
        if g.status.checking || g.status.ready {
            drop(g);
            return status_of(app).await;
        }
        g.status.checking = true;
        g.status.error = None;
    }
    let current = app.package_info().version.clone();
    let result =
        tauri::async_runtime::spawn_blocking(move || find_and_verify(&url, &current, download))
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r);
    let mut g = state.inner.lock().await;
    g.status.checking = false;
    g.status.last_check = Some(treasury_core::time::now_utc());
    match result {
        Ok(Found::Ready(release, bytes)) => {
            g.status.available = Some(available(&release));
            g.status.ready = true;
            tracing::info!("atualização {} baixada e conferida", release.version);
            g.pending = Some((*release, bytes));
            let _ = app.emit("update://ready", g.status.available.clone());
        }
        Ok(Found::Available(release)) => {
            // Avisa uma vez por versão, não a cada procura.
            let new = g.status.available.as_ref().map(|a| a.version.as_str())
                != Some(release.version.to_string().as_str());
            g.status.available = Some(available(&release));
            tracing::info!("versão {} disponível (só avisar)", release.version);
            if new {
                let _ = app.emit("update://available", g.status.available.clone());
            }
        }
        Ok(Found::Nothing) => g.status.available = None,
        Err(e) => {
            tracing::warn!("atualização: {e}");
            g.status.error = Some(e);
        }
    }
    drop(g);
    status_of(app).await
}

fn available(release: &Release) -> AvailableUpdate {
    AvailableUpdate {
        version: release.version.to_string(),
        notes: release.notes.clone(),
        date: release.date.clone(),
    }
}

/// A assinatura minisign do pacote, feita para a versão anunciada.
fn minisign_check(release: &Release, bytes: &[u8]) -> Result<(), String> {
    let invalid = |motivo: String| {
        treasury_core::i18n::t_args(
            "casca-atualizacao-minisign-invalida",
            &treasury_core::i18n::args(&[("motivo", motivo.into())]),
        )
    };
    let key = minisign::embedded().map_err(invalid)?;
    let sig = minisign::decode_field(&release.platform.signature).map_err(invalid)?;
    let trusted = key.verify(&sig, bytes).map_err(invalid)?;
    let announced = release.version.to_string();
    match minisign::signed_version(&trusted) {
        Some(v)
            if semver::Version::parse(v.trim_start_matches('v')).ok()
                == Some(release.version.clone()) =>
        {
            Ok(())
        }
        signed => Err(treasury_core::i18n::t_args(
            "casca-atualizacao-versao-assinada",
            &treasury_core::i18n::args(&[
                ("assinada", signed.unwrap_or("?").to_string().into()),
                ("anunciada", announced.into()),
            ]),
        )),
    }
}

/// Confere a segunda assinatura (YubiKey) do pacote, que vem no `latest.json`, ao lado da
/// minisign da plataforma (`yubikey_signature`). Falha fechado: um app sem chaves de hardware
/// embutidas não aceita atualização nenhuma, porque só a minisign não basta
/// (docs/trust.md).
fn hardware_check(release: &Release, bytes: &[u8]) -> Result<(), String> {
    let keys = crate::update_sig::embedded()?;
    if keys.is_empty() {
        return Err(treasury_core::i18n::t("casca-atualizacao-sem-chaves"));
    }
    let signature = release
        .platform
        .yubikey_signature
        .as_deref()
        .ok_or_else(|| treasury_core::i18n::t("casca-atualizacao-sem-yubikey"))?;
    let version = release.version.to_string();
    let namespace = crate::update_sig::namespace(&version);
    let signer = crate::update_sig::verify(&keys, signature, bytes, &namespace).map_err(|e| {
        treasury_core::i18n::t_args(
            "casca-atualizacao-yubikey-invalida",
            &treasury_core::i18n::args(&[("motivo", e.into())]),
        )
    })?;
    tracing::info!("atualização {version} assinada pela YubiKey ({signer})");
    Ok(())
}

/// Procura ao abrir (depois de um minuto) e a cada 6 h, conforme o modo do ajuste: baixa e
/// confere sozinho, só avisa, ou não procura.
pub fn spawn(app: AppHandle) {
    if endpoint().is_none() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK).await;
        let mut every = tokio::time::interval(EVERY);
        loop {
            every.tick().await;
            match mode(&app) {
                UpdateMode::Download => {
                    check(&app, true).await;
                }
                UpdateMode::Notify => {
                    check(&app, false).await;
                }
                UpdateMode::Off => {}
            }
        }
    });
}

#[tauri::command]
pub async fn update_status(app: AppHandle) -> UpdateStatus {
    status_of(&app).await
}

/// "Procurar agora" (vale em qualquer modo) e "Baixar" (`download: true`). Sem `download`, só
/// o modo "só avisar" deixa de baixar.
#[tauri::command]
pub async fn update_check(app: AppHandle, download: Option<bool>) -> UpdateStatus {
    let download = download.unwrap_or(mode(&app) != UpdateMode::Notify);
    check(&app, download).await
}

/// Instala a versão já baixada e conferida e reinicia o app.
#[tauri::command]
pub async fn update_install(app: AppHandle) -> Result<(), String> {
    let installer = Installer::current().ok_or_else(|| {
        "pacote instalado desconhecido: atualize pelas releases do Treasury".to_string()
    })?;
    let state = app.state::<UpdateState>();
    let Some((release, bytes)) = state.inner.lock().await.pending.take() else {
        return Err(treasury_core::i18n::t("casca-atualizacao-nada-baixado"));
    };
    let version = release.version.to_string();
    tracing::info!("instalando a atualização {version}");
    #[cfg(target_os = "linux")]
    let appimage = app.env().appimage.map(std::path::PathBuf::from);
    #[cfg(not(target_os = "linux"))]
    let appimage = None;
    let (result, bytes) = tauri::async_runtime::spawn_blocking(move || {
        let r = install::install(installer, &version, &bytes, appimage);
        (r, bytes)
    })
    .await
    .map_err(|e| e.to_string())?;
    match result {
        Ok(install::After::Restart) => app.restart(),
        Ok(install::After::Exit) => app.exit(0),
        Err(install::Failure::Manual(m)) => {
            // O pacote conferido ficou numa pasta: a pessoa instala pelo sistema.
            state.inner.lock().await.pending = Some((release, bytes));
            return Err(treasury_core::i18n::t_args(
                "casca-atualizacao-instalar-manual",
                &treasury_core::i18n::args(&[
                    ("motivo", m.reason.into()),
                    ("caminho", m.path.display().to_string().into()),
                ]),
            ));
        }
        Err(install::Failure::Failed(e)) => {
            // Instalação cancelada ou falha (pedido de senha do administrador recusado): a
            // versão baixada e conferida continua pronta para tentar de novo.
            state.inner.lock().await.pending = Some((release, bytes));
            return Err(e);
        }
    }
    Ok(())
}
