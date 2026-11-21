//! Pasta Downloads (Ajustes › Preferências, ajuste `watch_downloads`, desligado por padrão):
//! com o cofre aberto, o app olha a pasta a cada 3 s e, quando chega um arquivo que o
//! Treasury importa (extrato OFX/CSV, nota de corretagem em PDF, relatório da B3), avisa a
//! interface (`downloads://file`) para oferecer a importação.
//!
//! - Só arquivos **novos**: o que já estava na pasta quando o acompanhamento começou (ao
//!   ligar o ajuste ou abrir o cofre) fica de fora.
//! - Só depois de o download terminar: o tamanho e a data precisam ficar iguais em duas
//!   leituras seguidas; arquivos temporários dos navegadores (`.crdownload`, `.part`,
//!   `.download`…) e ocultos são ignorados.
//! - Para reconhecer o formato, o arquivo (até 25 MB) é lido na memória e passa pelo
//!   `import.sniff` do núcleo; nada é gravado nem sai do computador. Formato desconhecido
//!   não gera aviso. O conteúdo só vai para a interface se o usuário aceitar
//!   (`downloads_read`, que só lê arquivos diretamente dentro da pasta acompanhada).
//!
//! Sem dependência nova: varredura com `read_dir` (sem `notify`), numa thread própria.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, EventTarget, Manager};
use treasury_core::ApiError;
use treasury_core::import::sniff::{FileKind, SniffArgs, sniff};

use crate::AppState;

/// Intervalo entre as leituras da pasta.
const EVERY: Duration = Duration::from_secs(3);
/// Leituras seguidas com o mesmo tamanho e data para o download contar como terminado.
pub const STABLE_CHECKS: u8 = 2;
/// Maior arquivo reconhecido (e lido para a importação).
pub const MAX_BYTES: u64 = 25 * 1024 * 1024;
/// Extensões que a importação aceita (imagens vão para o agente de IA).
const SUPPORTED: &[&str] = &[
    "ofx", "qfx", "csv", "txt", "pdf", "xlsx", "xls", "ods", "png", "jpg", "jpeg", "heic", "heif",
    "webp", "gif",
];
/// Arquivos parciais dos navegadores (Chrome, Firefox, Safari, Opera, Edge…).
const PARTIAL: &[&str] = &[
    "crdownload",
    "part",
    "partial",
    "download",
    "opdownload",
    "tmp",
];

/// Um arquivo da pasta, como a varredura viu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub name: String,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

/// O nome serve para a importação? (ocultos, parciais e extensões de fora: não)
pub fn candidate(name: &str) -> bool {
    if name.starts_with('.') || name.starts_with("~$") {
        return false;
    }
    let lower = name.to_lowercase();
    let Some((stem, ext)) = lower.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty() && SUPPORTED.contains(&ext) && !PARTIAL.contains(&ext)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Track {
    /// Já estava na pasta (ou é grande demais): nunca vira aviso.
    Ignored,
    /// Arquivo novo esperando o download terminar.
    Pending {
        size: u64,
        modified: Option<SystemTime>,
        same: u8,
    },
    /// Já avisado.
    Offered,
}

/// Estado da varredura: o que já se viu na pasta. Lógica pura (testável sem disco).
#[derive(Debug, Default)]
pub struct Watcher {
    primed: bool,
    files: HashMap<String, Track>,
}

impl Watcher {
    /// Recebe a listagem atual e devolve os arquivos novos cujo download terminou.
    /// A primeira listagem só registra o que já existe.
    pub fn observe(&mut self, listing: Vec<Seen>) -> Vec<Seen> {
        let names: std::collections::HashSet<&str> =
            listing.iter().map(|f| f.name.as_str()).collect();
        // Apagado (ou renomeado): se voltar, é novo.
        self.files.retain(|name, _| names.contains(name.as_str()));
        let mut ready = Vec::new();
        for f in listing {
            if !candidate(&f.name) {
                continue;
            }
            if !self.primed {
                self.files.insert(f.name, Track::Ignored);
                continue;
            }
            let next = match self.files.get(&f.name) {
                None if f.size > MAX_BYTES => Track::Ignored,
                None => Track::Pending {
                    size: f.size,
                    modified: f.modified,
                    same: 1,
                },
                Some(Track::Pending { .. }) if f.size > MAX_BYTES => Track::Ignored,
                Some(Track::Pending {
                    size,
                    modified,
                    same,
                }) => {
                    if *size == f.size && *modified == f.modified {
                        let same = same.saturating_add(1);
                        if same >= STABLE_CHECKS && f.size > 0 {
                            ready.push(f.clone());
                            Track::Offered
                        } else {
                            Track::Pending {
                                size: f.size,
                                modified: f.modified,
                                same,
                            }
                        }
                    } else {
                        Track::Pending {
                            size: f.size,
                            modified: f.modified,
                            same: 1,
                        }
                    }
                }
                Some(t) => t.clone(),
            };
            self.files.insert(f.name, next);
        }
        self.primed = true;
        ready
    }
}

/// Lista os arquivos comuns da pasta (sem pastas nem atalhos) com nome que interessa.
fn list(dir: &Path) -> std::io::Result<Vec<Seen>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let Ok(entry) = entry else { continue };
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !candidate(&name) {
            continue;
        }
        // `DirEntry::metadata` não segue atalhos: um atalho não é arquivo comum.
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        out.push(Seen {
            name,
            size: meta.len(),
            modified: meta.modified().ok(),
        });
    }
    Ok(out)
}

/// A pasta de downloads do usuário: a do sistema, `~/Downloads` ou `~/Download`.
fn user_downloads_dir(app: &AppHandle) -> Option<PathBuf> {
    let mut options = Vec::new();
    if let Ok(d) = app.path().download_dir() {
        options.push(d);
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        options.push(home.join("Downloads"));
        options.push(home.join("Download"));
    }
    options.into_iter().find(|d| d.is_dir())
}

/// Pasta acompanhada agora (`None` com o ajuste desligado ou o cofre bloqueado).
#[derive(Default)]
pub struct DownloadsWatch {
    dir: Mutex<Option<PathBuf>>,
}

impl DownloadsWatch {
    fn set(&self, dir: Option<PathBuf>) {
        *self.dir.lock().unwrap_or_else(|p| p.into_inner()) = dir;
    }

    fn get(&self) -> Option<PathBuf> {
        self.dir.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
}

/// O que a interface recebe em `downloads://file`.
#[derive(Debug, Clone, Serialize)]
struct Offer {
    path: String,
    name: String,
    size: u64,
    kind: FileKind,
    label: String,
    format: Option<String>,
}

fn enabled(app: &AppHandle) -> bool {
    let api = &app.state::<AppState>().api;
    api.is_unlocked()
        && api
            .with_session(|s, _| {
                treasury_core::settings::get_or(
                    s.db.conn(),
                    treasury_core::api::WATCH_DOWNLOADS_SETTING,
                    false,
                )
            })
            .unwrap_or(false)
}

/// Reconhece o arquivo pelo núcleo (direto, sem `api_call`: não conta como atividade do
/// usuário para o bloqueio automático).
fn classify(dir: &Path, f: &Seen) -> Option<Offer> {
    use base64::Engine;
    let path = dir.join(&f.name);
    let bytes = std::fs::read(&path).ok()?;
    let r = sniff(&SniffArgs {
        filename: f.name.clone(),
        content_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
    })
    .ok()?;
    (r.kind != FileKind::Unknown).then(|| Offer {
        path: path.to_string_lossy().into_owned(),
        name: f.name.clone(),
        size: bytes.len() as u64,
        kind: r.kind,
        label: r.label,
        format: r.format,
    })
}

/// Liga a varredura (thread própria; parada enquanto o ajuste estiver desligado).
pub fn spawn(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("treasury-downloads".into())
        .spawn(move || {
            let mut watcher: Option<(PathBuf, Watcher)> = None;
            let mut warned = false;
            loop {
                std::thread::sleep(EVERY);
                let dir = enabled(&app).then(|| user_downloads_dir(&app)).flatten();
                let Some(dir) = dir else {
                    watcher = None;
                    warned = false;
                    app.state::<DownloadsWatch>().set(None);
                    continue;
                };
                if watcher.as_ref().is_none_or(|(d, _)| d != &dir) {
                    watcher = Some((dir.clone(), Watcher::default()));
                    app.state::<DownloadsWatch>().set(dir.canonicalize().ok());
                }
                let listing = match list(&dir) {
                    Ok(l) => {
                        warned = false;
                        l
                    }
                    Err(e) => {
                        // macOS: sem permissão para a pasta Downloads (Privacidade).
                        if !warned {
                            tracing::warn!("pasta Downloads: {e}");
                            warned = true;
                        }
                        continue;
                    }
                };
                let Some((_, w)) = watcher.as_mut() else {
                    continue;
                };
                for f in w.observe(listing) {
                    if let Some(offer) = classify(&dir, &f) {
                        let _ =
                            app.emit_to(EventTarget::webview("main"), "downloads://file", offer);
                    }
                }
            }
        });
    if let Err(e) = spawned {
        tracing::warn!("acompanhamento da pasta Downloads: {e}");
    }
}

fn err(code: &str, message: impl Into<String>) -> ApiError {
    ApiError {
        code: code.into(),
        message: message.into(),
        details: None,
    }
}

/// Confere que `path` é um arquivo importável diretamente dentro de `dir` (já canônico).
pub fn validate(dir: &Path, path: &Path) -> Result<PathBuf, ApiError> {
    let p = path
        .canonicalize()
        .map_err(|_| err("not_found", "arquivo não encontrado"))?;
    let name_ok = p
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(candidate);
    if p.parent() != Some(dir) || !name_ok {
        return Err(err(
            "forbidden",
            "só arquivos importáveis da pasta Downloads",
        ));
    }
    let meta = std::fs::metadata(&p).map_err(|e| err("io", e.to_string()))?;
    if !meta.is_file() {
        return Err(err("forbidden", "não é um arquivo"));
    }
    if meta.len() > MAX_BYTES {
        return Err(err("too_large", "arquivo maior que 25 MB"));
    }
    Ok(p)
}

/// Lê um arquivo oferecido (o usuário aceitou a importação). Só com o acompanhamento
/// ligado, só arquivos diretamente dentro da pasta acompanhada.
#[tauri::command]
pub async fn downloads_read(app: AppHandle, path: String) -> Result<Value, ApiError> {
    use base64::Engine;
    let Some(dir) = app.state::<DownloadsWatch>().get() else {
        return Err(err(
            "forbidden",
            "o acompanhamento da pasta Downloads está desligado",
        ));
    };
    if !app.state::<AppState>().api.is_unlocked() {
        return Err(err("vault_locked", "o cofre está bloqueado"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let p = validate(&dir, Path::new(&path))?;
        let bytes = std::fs::read(&p).map_err(|e| err("io", e.to_string()))?;
        Ok(json!({
            "filename": p.file_name().map(|n| n.to_string_lossy().into_owned()),
            "size": bytes.len(),
            "content_base64": base64::engine::general_purpose::STANDARD.encode(bytes),
        }))
    })
    .await
    .map_err(|e| err("internal", e.to_string()))?
}

/// Manda o arquivo para a Lixeira do sistema (o usuário recupera de lá, se precisar).
fn move_to_trash(p: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::{NSFileManager, NSString, NSURL};
        let url = NSURL::fileURLWithPath(&NSString::from_str(&p.to_string_lossy()));
        NSFileManager::defaultManager()
            .trashItemAtURL_resultingItemURL_error(&url, None)
            .map_err(|e| e.localizedDescription().to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        freedesktop_trash(p)
    }
}

/// Lixeira do Linux (especificação freedesktop.org): o arquivo vai para
/// `~/.local/share/Trash/files` com a ficha em `info`, para o gerenciador de arquivos
/// restaurar.
#[cfg(not(target_os = "macos"))]
fn freedesktop_trash(p: &Path) -> Result<(), String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .ok_or("sem pasta pessoal")?
        .join("Trash");
    let (files, info) = (base.join("files"), base.join("info"));
    std::fs::create_dir_all(&files).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&info).map_err(|e| e.to_string())?;
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or("arquivo sem nome")?;
    let mut target = name.clone();
    let mut n = 1;
    while files.join(&target).exists() || info.join(format!("{target}.trashinfo")).exists() {
        n += 1;
        target = format!("{n} {name}");
    }
    let encoded: String = p
        .to_string_lossy()
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();
    let when: String = treasury_core::time::now_utc().chars().take(19).collect();
    std::fs::write(
        info.join(format!("{target}.trashinfo")),
        format!("[Trash Info]\nPath={encoded}\nDeletionDate={when}\n"),
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(p, files.join(&target)).map_err(|e| e.to_string())
}

/// Manda para a Lixeira um arquivo oferecido, depois de importado com sucesso. Só com o
/// acompanhamento e o ajuste "apagar depois de importar" ligados, só arquivos
/// diretamente dentro da pasta acompanhada.
#[tauri::command]
pub async fn downloads_trash(app: AppHandle, path: String) -> Result<Value, ApiError> {
    let Some(dir) = app.state::<DownloadsWatch>().get() else {
        return Err(err(
            "forbidden",
            "o acompanhamento da pasta Downloads está desligado",
        ));
    };
    let api = &app.state::<AppState>().api;
    if !api.is_unlocked() {
        return Err(err("vault_locked", "o cofre está bloqueado"));
    }
    let on = api
        .with_session(|s, _| {
            treasury_core::settings::get_or(
                s.db.conn(),
                treasury_core::api::DOWNLOADS_DELETE_SETTING,
                false,
            )
        })
        .unwrap_or(false);
    if !on {
        return Err(err(
            "forbidden",
            "ligue \"Apagar de Downloads depois de importar\" para o app apagar arquivos",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let p = validate(&dir, Path::new(&path))?;
        move_to_trash(&p).map_err(|e| err("io", format!("não foi possível apagar: {e}")))?;
        Ok(json!({ "trashed": p.file_name().map(|n| n.to_string_lossy().into_owned()) }))
    })
    .await
    .map_err(|e| err("internal", e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(name: &str, size: u64, t: u64) -> Seen {
        Seen {
            name: name.into(),
            size,
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(t)),
        }
    }

    fn names(v: Vec<Seen>) -> Vec<String> {
        v.into_iter().map(|s| s.name).collect()
    }

    #[test]
    fn candidates_skip_partial_hidden_and_other_formats() {
        for ok in [
            "extrato.ofx",
            "Fatura.QFX",
            "nubank.csv",
            "nota 123.pdf",
            "export.txt",
            "negociacao.xlsx",
            "foto.jpg",
            "comprovante.PNG",
            "IMG_0001.HEIC",
        ] {
            assert!(candidate(ok), "{ok}");
        }
        for no in [
            "extrato.ofx.crdownload",
            "nota.pdf.part",
            "nota.pdf.download",
            "x.opdownload",
            ".DS_Store",
            ".extrato.ofx",
            "~$planilha.xlsx",
            "foto.jpg.crdownload",
            "setup.dmg",
            "semextensao",
            ".ofx",
        ] {
            assert!(!candidate(no), "{no}");
        }
    }

    #[test]
    fn only_new_files_after_the_size_settles() {
        let mut w = Watcher::default();
        // What was already in the folder never becomes a notice.
        assert!(w.observe(vec![seen("antigo.ofx", 10, 1)]).is_empty());
        assert!(w.observe(vec![seen("antigo.ofx", 10, 1)]).is_empty());
        // Download in progress: it grows, then stays the same for two reads.
        let mut listing = vec![seen("antigo.ofx", 10, 1), seen("novo.ofx", 100, 2)];
        assert!(w.observe(listing.clone()).is_empty(), "first read");
        listing[1] = seen("novo.ofx", 200, 3);
        assert!(w.observe(listing.clone()).is_empty(), "still growing");
        assert_eq!(names(w.observe(listing.clone())), ["novo.ofx"]);
        // Notified only once.
        assert!(w.observe(listing.clone()).is_empty());
        // Deleted and downloaded again: notified again.
        assert!(w.observe(vec![seen("antigo.ofx", 10, 1)]).is_empty());
        assert!(w.observe(listing.clone()).is_empty());
        assert_eq!(names(w.observe(listing)), ["novo.ofx"]);
    }

    #[test]
    fn ignores_empty_huge_and_partial_files() {
        let mut w = Watcher::default();
        w.observe(vec![]);
        let listing = vec![
            seen("vazio.csv", 0, 1),
            seen("enorme.pdf", MAX_BYTES + 1, 1),
            seen("nota.pdf.crdownload", 50, 1),
            seen(".oculto.ofx", 50, 1),
        ];
        for _ in 0..4 {
            assert!(w.observe(listing.clone()).is_empty());
        }
        // The partial file becomes the final file (renamed by the browser): now it counts.
        let done = vec![seen("nota.pdf", 50, 2)];
        assert!(w.observe(done.clone()).is_empty());
        assert_eq!(names(w.observe(done)), ["nota.pdf"]);
    }

    #[test]
    fn several_files_together() {
        let mut w = Watcher::default();
        w.observe(vec![]);
        let listing = vec![seen("a.ofx", 5, 1), seen("b.csv", 6, 1)];
        assert!(w.observe(listing.clone()).is_empty());
        let mut got = names(w.observe(listing));
        got.sort();
        assert_eq!(got, ["a.ofx", "b.csv"]);
    }

    #[test]
    fn read_only_inside_the_watched_folder() {
        let base = std::env::temp_dir().join(format!(
            "treasury-downloads-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let dir = base.join("Downloads");
        let sub = dir.join("pasta");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(dir.join("extrato.ofx"), b"OFX").unwrap();
        std::fs::write(dir.join("setup.dmg"), b"DMG").unwrap();
        std::fs::write(sub.join("dentro.ofx"), b"OFX").unwrap();
        std::fs::write(base.join("fora.ofx"), b"OFX").unwrap();
        let canon = dir.canonicalize().unwrap();
        assert!(validate(&canon, &dir.join("extrato.ofx")).is_ok());
        let no = |p: PathBuf| validate(&canon, &p).unwrap_err().code;
        assert_eq!(no(dir.join("setup.dmg")), "forbidden");
        assert_eq!(no(sub.join("dentro.ofx")), "forbidden");
        assert_eq!(no(dir.join("..").join("fora.ofx")), "forbidden");
        assert_eq!(no(dir.join("nao-existe.ofx")), "not_found");
        #[cfg(unix)]
        {
            // Shortcut inside Downloads pointing outside: refused.
            std::os::unix::fs::symlink(base.join("fora.ofx"), dir.join("atalho.ofx")).unwrap();
            assert_eq!(no(dir.join("atalho.ofx")), "forbidden");
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
