//! Instala um pacote já baixado e com as duas assinaturas conferidas, do jeito de cada
//! pacote:
//!
//! - **macOS (`.app.tar.gz`)**: extrai com o `tar` do sistema ao lado do app e troca o
//!   `.app` inteiro por renomeação; sem permissão na pasta, pede a senha de administrador
//!   (`osascript … with administrator privileges`).
//! - **AppImage**: grava o arquivo novo ao lado do atual e renomeia por cima (a troca é de uma
//!   vez: ou fica o antigo, ou o novo).
//! - **`.deb` e `.rpm`**: `pkexec dpkg -i` ou `pkexec rpm -U` (o sistema pede a senha). Sem o
//!   `pkexec`, ou com a senha recusada, o pacote conferido fica numa pasta e a mensagem diz
//!   onde, para instalar à mão.
//! - **Windows (NSIS ou `.msi`)**: abre o instalador em modo passivo, que fecha o app, instala
//!   e abre a versão nova.

use std::io;
use std::path::{Path, PathBuf};

use super::manifest::Installer;

/// O que o app faz depois.
#[derive(Debug, PartialEq, Eq)]
pub enum After {
    /// Reabrir (macOS e Linux).
    Restart,
    /// Sair já: o instalador do Windows reabre a versão nova.
    #[cfg_attr(not(windows), allow(dead_code))]
    Exit,
}

/// A instalação parou depois de gravar o pacote conferido: dá para instalar à mão.
#[derive(Debug)]
pub struct Manual {
    pub reason: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum Failure {
    Failed(String),
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    Manual(Manual),
}

impl From<io::Error> for Failure {
    fn from(e: io::Error) -> Self {
        Self::Failed(e.to_string())
    }
}

impl From<String> for Failure {
    fn from(e: String) -> Self {
        Self::Failed(e)
    }
}

impl From<&str> for Failure {
    fn from(e: &str) -> Self {
        Self::Failed(e.to_string())
    }
}

/// Começo de cada formato, para não entregar ao instalador um pacote de outro tipo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Gzip,
    Elf,
    Deb,
    Rpm,
    Exe,
    Msi,
}

pub fn kind_of(bytes: &[u8]) -> Option<Kind> {
    let starts = |m: &[u8]| bytes.starts_with(m);
    if starts(&[0x1f, 0x8b]) {
        Some(Kind::Gzip)
    } else if starts(b"\x7fELF") {
        Some(Kind::Elf)
    } else if starts(b"!<arch>\ndebian-binary") {
        Some(Kind::Deb)
    } else if starts(&[0xed, 0xab, 0xee, 0xdb]) {
        Some(Kind::Rpm)
    } else if starts(b"MZ") {
        Some(Kind::Exe)
    } else if starts(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]) {
        Some(Kind::Msi)
    } else {
        None
    }
}

/// O formato que cada pacote instalado espera.
pub fn expected(installer: Installer) -> Kind {
    match installer {
        Installer::App => Kind::Gzip,
        Installer::AppImage => Kind::Elf,
        Installer::Deb => Kind::Deb,
        Installer::Rpm => Kind::Rpm,
        Installer::Nsis => Kind::Exe,
        Installer::Msi => Kind::Msi,
    }
}

/// Pasta nova, só da pessoa (0700), com nome aleatório.
fn private_dir(parent: &Path, prefix: &str) -> io::Result<PathBuf> {
    let mut rnd = [0u8; 8];
    getrandom::fill(&mut rnd).map_err(|e| io::Error::other(e.to_string()))?;
    let dir = parent.join(format!("{prefix}{}", hex::encode(rnd)));
    let mut b = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b.create(&dir)?;
    Ok(dir)
}

/// Apaga a pasta ao sair, a não ser que ela tenha de ficar (pacote para instalar à mão).
struct Scratch {
    dir: PathBuf,
    keep: bool,
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.keep {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
}

/// Instala o pacote. `appimage` é o caminho do AppImage em uso (variável `APPIMAGE`).
pub fn install(
    installer: Installer,
    version: &str,
    bytes: &[u8],
    appimage: Option<PathBuf>,
) -> Result<After, Failure> {
    if kind_of(bytes) != Some(expected(installer)) {
        return Err(Failure::Failed(format!(
            "o pacote baixado não é um {}",
            installer.name()
        )));
    }
    match installer {
        #[cfg(target_os = "macos")]
        Installer::App => macos::install(bytes).map(|()| After::Restart),
        #[cfg(target_os = "linux")]
        Installer::AppImage => {
            let path = appimage.ok_or("o AppImage em uso não foi encontrado")?;
            linux::install_appimage(&path, bytes)?;
            Ok(After::Restart)
        }
        #[cfg(target_os = "linux")]
        Installer::Deb => linux::install_package(version, bytes, "deb", &["dpkg", "-i"]),
        #[cfg(target_os = "linux")]
        Installer::Rpm => linux::install_package(version, bytes, "rpm", &["rpm", "-U"]),
        #[cfg(windows)]
        Installer::Nsis | Installer::Msi => windows::install(installer, version, bytes),
        _ => {
            let _ = (version, appimage);
            Err(Failure::Failed(format!(
                "pacote {} não se instala neste sistema",
                installer.name()
            )))
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::{Failure, Scratch, private_dir};

    /// O `.app` em uso (o executável fica em `<app>.app/Contents/MacOS/`).
    fn current_app() -> Result<PathBuf, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        exe.ancestors()
            .find(|p| p.extension().is_some_and(|e| e == "app"))
            .map(Path::to_path_buf)
            .ok_or_else(|| "o app em uso não está num .app".into())
    }

    /// Texto entre aspas simples para o shell, dentro de uma string do AppleScript.
    fn quoted(p: &Path) -> String {
        let s = p.to_string_lossy().replace('\'', r"'\''");
        format!("'{s}'").replace('\\', r"\\").replace('"', "\\\"")
    }

    pub fn install(bytes: &[u8]) -> Result<(), Failure> {
        install_at(&current_app()?, bytes)
    }

    /// Troca o `.app` em `target` pelo do pacote.
    pub fn install_at(target: &Path, bytes: &[u8]) -> Result<(), Failure> {
        let target = target.to_path_buf();
        let parent = target.parent().ok_or("app sem pasta")?;
        // Ao lado do app, para a troca ser uma renomeação no mesmo disco; sem permissão na
        // pasta, na pasta temporária, e a troca vai com a senha de administrador.
        let (dir, beside) = match private_dir(parent, ".treasury-update-") {
            Ok(d) => (d, true),
            Err(_) => (
                private_dir(&std::env::temp_dir(), "treasury-update-")?,
                false,
            ),
        };
        let scratch = Scratch {
            dir: dir.clone(),
            keep: false,
        };
        let archive = dir.join("pacote.tar.gz");
        std::fs::write(&archive, bytes)?;
        let out = dir.join("novo");
        std::fs::create_dir(&out)?;
        let status = Command::new("/usr/bin/tar")
            .arg("-xzf")
            .arg(&archive)
            .arg("-C")
            .arg(&out)
            .status()?;
        if !status.success() {
            return Err(Failure::Failed("o pacote não abriu (tar)".into()));
        }
        let _ = std::fs::remove_file(&archive);
        let apps: Vec<PathBuf> = std::fs::read_dir(&out)?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "app") && p.is_dir())
            .collect();
        let [new_app] = apps.as_slice() else {
            return Err(Failure::Failed("o pacote não tem um .app".into()));
        };
        if beside && swap(&target, new_app, &dir.join("antigo.app")).is_ok() {
            drop(scratch);
            let _ = Command::new("/usr/bin/touch").arg(&target).status();
            return Ok(());
        }
        let script = format!(
            "do shell script \"/bin/rm -rf {old} && /bin/mv -f {new} {old}\" with administrator privileges",
            old = quoted(&target),
            new = quoted(new_app),
        );
        let status = Command::new("/usr/bin/osascript")
            .arg("-e")
            .arg(script)
            .status()?;
        if !status.success() {
            return Err(Failure::Failed(
                "a troca do app precisa da senha de administrador".into(),
            ));
        }
        drop(scratch);
        let _ = Command::new("/usr/bin/touch").arg(&target).status();
        Ok(())
    }

    /// Troca o app por renomeação; se a segunda falhar, o antigo volta.
    fn swap(target: &Path, new_app: &Path, backup: &Path) -> std::io::Result<()> {
        std::fs::rename(target, backup)?;
        if let Err(e) = std::fs::rename(new_app, target) {
            let _ = std::fs::rename(backup, target);
            return Err(e);
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use std::process::Command;

        #[test]
        fn swaps_the_whole_app_bundle() {
            let base = std::env::temp_dir().join(format!("treasury-mac-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&base);
            let apps = base.join("Applications");
            let old = apps.join("Treasury.app/Contents/MacOS");
            std::fs::create_dir_all(&old).unwrap();
            std::fs::write(old.join("treasury"), b"antigo").unwrap();
            std::fs::write(old.join("sobra"), b"so no antigo").unwrap();
            // O pacote como o release.yml monta: `tar -czf … -C <pasta> Treasury.app`.
            let build = base.join("build/Treasury.app/Contents/MacOS");
            std::fs::create_dir_all(&build).unwrap();
            std::fs::write(build.join("treasury"), b"novo").unwrap();
            let tgz = base.join("pacote.tar.gz");
            let ok = Command::new("/usr/bin/tar")
                .arg("-czf")
                .arg(&tgz)
                .arg("-C")
                .arg(base.join("build"))
                .arg("Treasury.app")
                .status()
                .unwrap();
            assert!(ok.success());
            let bytes = std::fs::read(&tgz).unwrap();
            super::install_at(&apps.join("Treasury.app"), &bytes).unwrap();
            let exe = apps.join("Treasury.app/Contents/MacOS");
            assert_eq!(std::fs::read(exe.join("treasury")).unwrap(), b"novo");
            assert!(!exe.join("sobra").exists(), "o bundle antigo sai inteiro");
            let left: Vec<_> = std::fs::read_dir(&apps)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            assert_eq!(left, ["Treasury.app"], "sem pasta de trabalho sobrando");
            // Um pacote sem .app não toca no instalado.
            let empty = base.join("vazio.tar.gz");
            std::fs::create_dir_all(base.join("nada")).unwrap();
            std::fs::write(base.join("nada/leia.txt"), b"x").unwrap();
            Command::new("/usr/bin/tar")
                .arg("-czf")
                .arg(&empty)
                .arg("-C")
                .arg(base.join("nada"))
                .arg("leia.txt")
                .status()
                .unwrap();
            assert!(
                super::install_at(&apps.join("Treasury.app"), &std::fs::read(&empty).unwrap())
                    .is_err()
            );
            assert_eq!(std::fs::read(exe.join("treasury")).unwrap(), b"novo");
            let _ = std::fs::remove_dir_all(&base);
        }

        #[test]
        fn paths_are_quoted_for_the_shell_inside_applescript() {
            let q = super::quoted(std::path::Path::new("/Applications/O'Treasury \"x\".app"));
            assert_eq!(q, r#"'/Applications/O'\\''Treasury \"x\".app'"#);
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::Command;

    use super::{After, Failure, Manual, Scratch, private_dir};

    /// Grava o AppImage novo ao lado do atual e renomeia por cima.
    pub fn install_appimage(path: &Path, bytes: &[u8]) -> Result<(), Failure> {
        let dir = path.parent().ok_or("AppImage sem pasta")?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Treasury.AppImage".into());
        let tmp = dir.join(format!(".{name}.novo"));
        let mode = std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o777)
            .unwrap_or(0o755)
            | 0o500;
        let write = || -> std::io::Result<()> {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(bytes)?;
            f.sync_all()?;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(mode))?;
            std::fs::rename(&tmp, path)
        };
        write().map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            Failure::Failed(format!("o AppImage não foi trocado: {e}"))
        })
    }

    /// `.deb` ou `.rpm` com o `pkexec` (o sistema pede a senha).
    pub fn install_package(
        version: &str,
        bytes: &[u8],
        ext: &str,
        command: &[&str],
    ) -> Result<After, Failure> {
        let dir = private_dir(&std::env::temp_dir(), "treasury-update-")?;
        let mut scratch = Scratch { dir, keep: false };
        let pkg = scratch.dir.join(format!("Treasury_{version}.{ext}"));
        std::fs::write(&pkg, bytes)?;
        let result = Command::new("pkexec").args(command).arg(&pkg).status();
        match result {
            Ok(s) if s.success() => Ok(After::Restart),
            other => {
                scratch.keep = true;
                let reason = match other {
                    Ok(s) => format!("pkexec terminou com {s}"),
                    Err(e) => format!("pkexec indisponível: {e}"),
                };
                Err(Failure::Manual(Manual { reason, path: pkg }))
            }
        }
    }
}

#[cfg(windows)]
mod windows {
    use std::process::Command;

    use super::{After, Failure, Installer, Scratch, private_dir};

    /// Abre o instalador em modo passivo e sai: ele fecha o que restar do app, instala e
    /// abre a versão nova.
    pub fn install(installer: Installer, version: &str, bytes: &[u8]) -> Result<After, Failure> {
        let dir = private_dir(&std::env::temp_dir(), "treasury-update-")?;
        // A pasta fica: o instalador ainda roda depois que o app sai.
        let scratch = Scratch { dir, keep: true };
        let (name, mut cmd) = match installer {
            Installer::Nsis => {
                let path = scratch
                    .dir
                    .join(format!("Treasury_{version}_x64-setup.exe"));
                std::fs::write(&path, bytes)?;
                let mut c = Command::new(&path);
                c.args(["/P", "/UPDATE", "/R"]);
                (path, c)
            }
            _ => {
                let path = scratch.dir.join(format!("Treasury_{version}_x64.msi"));
                std::fs::write(&path, bytes)?;
                let msiexec = std::env::var("SYSTEMROOT")
                    .map(|r| format!(r"{r}\System32\msiexec.exe"))
                    .unwrap_or_else(|_| "msiexec.exe".into());
                let mut c = Command::new(msiexec);
                c.arg("/i")
                    .arg(&path)
                    .args(["/passive", "/promptrestart", "AUTOLAUNCHAPP=True"]);
                (path, c)
            }
        };
        cmd.spawn()
            .map_err(|e| Failure::Failed(format!("{}: {e}", name.display())))?;
        Ok(After::Exit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_package_needs_its_own_format() {
        assert_eq!(kind_of(&[0x1f, 0x8b, 8, 0]), Some(Kind::Gzip));
        assert_eq!(kind_of(b"\x7fELF\x02\x01"), Some(Kind::Elf));
        assert_eq!(kind_of(b"!<arch>\ndebian-binary   "), Some(Kind::Deb));
        assert_eq!(kind_of(&[0xed, 0xab, 0xee, 0xdb, 3]), Some(Kind::Rpm));
        assert_eq!(kind_of(b"MZ\x90\0"), Some(Kind::Exe));
        assert_eq!(
            kind_of(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1, 0]),
            Some(Kind::Msi)
        );
        assert_eq!(kind_of(b"<html>"), None);
        // Um .deb no lugar do AppImage não chega ao instalador.
        let r = install(
            Installer::AppImage,
            "9.9.9",
            b"!<arch>\ndebian-binary",
            None,
        );
        assert!(matches!(r, Err(Failure::Failed(_))));
    }

    #[test]
    fn scratch_dirs_are_private_and_go_away() {
        let base = std::env::temp_dir();
        let dir = private_dir(&base, "treasury-teste-").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }
        drop(Scratch {
            dir: dir.clone(),
            keep: false,
        });
        assert!(!dir.exists());
    }
}
