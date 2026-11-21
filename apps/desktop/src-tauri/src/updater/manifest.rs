//! O `latest.json` da release: a versão anunciada, as notas e, por pacote, o endereço e as
//! duas assinaturas. O formato é o que o `scripts/sign-release.sh` monta.
//!
//! A entrada de cada pacote é `<sistema>-<arquitetura>-<pacote>` (`darwin-aarch64-app`,
//! `linux-x86_64-appimage`, `windows-x86_64-nsis`…), com `<sistema>-<arquitetura>` de
//! reserva para o pacote padrão do sistema.

use std::collections::HashMap;

use serde::Deserialize;
use tauri::Url;

#[derive(Debug, Deserialize)]
struct Raw {
    version: String,
    notes: Option<String>,
    pub_date: Option<String>,
    #[serde(default)]
    platforms: HashMap<String, Platform>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Platform {
    pub url: String,
    /// O `.sig` minisign em base64.
    pub signature: String,
    /// A assinatura SSHSIG da YubiKey (texto `-----BEGIN SSH SIGNATURE-----`).
    pub yubikey_signature: Option<String>,
}

/// Uma versão anunciada, com o pacote desta instalação.
#[derive(Debug, Clone)]
pub struct Release {
    pub version: semver::Version,
    pub notes: Option<String>,
    pub date: Option<String>,
    pub url: Url,
    pub platform: Platform,
}

/// Pacote instalado, pela marca que o empacotador do Tauri grava no executável.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Installer {
    App,
    AppImage,
    Deb,
    Rpm,
    Nsis,
    Msi,
}

impl Installer {
    pub fn name(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::AppImage => "appimage",
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::Nsis => "nsis",
            Self::Msi => "msi",
        }
    }

    /// O desta instalação. No Linux e no Windows, sem a marca (compilado à mão), nenhum.
    pub fn current() -> Option<Self> {
        use tauri::utils::config::BundleType;
        use tauri::utils::platform::bundle_type;
        bundle_type().map(|b| match b {
            BundleType::App | BundleType::Dmg => Self::App,
            BundleType::AppImage => Self::AppImage,
            BundleType::Deb => Self::Deb,
            BundleType::Rpm => Self::Rpm,
            BundleType::Nsis => Self::Nsis,
            BundleType::Msi => Self::Msi,
        })
    }
}

pub fn os() -> Option<&'static str> {
    if cfg!(target_os = "macos") {
        Some("darwin")
    } else if cfg!(target_os = "linux") {
        Some("linux")
    } else if cfg!(target_os = "windows") {
        Some("windows")
    } else {
        None
    }
}

pub fn arch() -> Option<&'static str> {
    if cfg!(target_arch = "x86_64") {
        Some("x86_64")
    } else if cfg!(target_arch = "aarch64") {
        Some("aarch64")
    } else {
        None
    }
}

/// Entradas a procurar, na ordem: a do pacote e a do sistema.
pub fn targets(os: &str, arch: &str, installer: Option<Installer>) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(i) = installer {
        out.push(format!("{os}-{arch}-{}", i.name()));
    }
    out.push(format!("{os}-{arch}"));
    out
}

/// Lê o `latest.json` e escolhe o pacote. `Ok(None)`: a versão anunciada não é mais nova que
/// a instalada.
pub fn parse(
    json: &[u8],
    current: &semver::Version,
    targets: &[String],
) -> Result<Option<Release>, String> {
    let raw: Raw =
        serde_json::from_slice(json).map_err(|e| format!("latest.json ilegível: {e}"))?;
    let version = semver::Version::parse(raw.version.trim().trim_start_matches('v'))
        .map_err(|_| format!("versão ilegível no latest.json: {}", raw.version))?;
    if version <= *current {
        return Ok(None);
    }
    let platform = targets
        .iter()
        .find_map(|t| raw.platforms.get(t))
        .cloned()
        .ok_or_else(|| {
            format!(
                "a versão {version} não tem pacote para {}",
                targets.join(", ")
            )
        })?;
    let url =
        Url::parse(&platform.url).map_err(|_| format!("endereço ilegível: {}", platform.url))?;
    Ok(Some(Release {
        version,
        notes: raw.notes,
        date: raw.pub_date,
        url,
        platform,
    }))
}

/// O pacote vem do mesmo repositório do `latest.json`: https, mesmo host e o caminho
/// `/<dono>/<repositório>/releases/download/`.
pub fn same_releases(endpoint: &Url, download: &Url) -> bool {
    let Some(base) = endpoint.path().split("/releases/").next() else {
        return false;
    };
    download.scheme() == "https"
        && download.host_str() == endpoint.host_str()
        && download.port() == endpoint.port()
        && download
            .path()
            .starts_with(&format!("{base}/releases/download/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn v(s: &str) -> semver::Version {
        semver::Version::parse(s).unwrap()
    }

    fn manifest() -> Vec<u8> {
        serde_json::to_vec(&json!({
            "version": "1.2.0",
            "notes": "Treasury 1.2.0",
            "pub_date": "2026-11-20T21:00:00Z",
            "platforms": {
                "darwin-aarch64-app": {"url": "https://exemplo.invalid/a.app.tar.gz", "signature": "s1", "yubikey_signature": "y1"},
                "darwin-aarch64": {"url": "https://exemplo.invalid/a.app.tar.gz", "signature": "s1", "yubikey_signature": "y1"},
                "linux-x86_64-deb": {"url": "https://exemplo.invalid/a.deb", "signature": "s2", "yubikey_signature": "y2"},
                "linux-x86_64": {"url": "https://exemplo.invalid/a.AppImage", "signature": "s3"}
            }
        }))
        .unwrap()
    }

    #[test]
    fn picks_the_package_then_the_system_default() {
        let deb = targets("linux", "x86_64", Some(Installer::Deb));
        assert_eq!(deb, ["linux-x86_64-deb", "linux-x86_64"]);
        let r = parse(&manifest(), &v("1.1.9"), &deb).unwrap().unwrap();
        assert_eq!(r.version, v("1.2.0"));
        assert_eq!(r.platform.signature, "s2");
        assert_eq!(r.date.as_deref(), Some("2026-11-20T21:00:00Z"));
        // AppImage sem entrada própria: a do sistema.
        let appimage = targets("linux", "x86_64", Some(Installer::AppImage));
        let r = parse(&manifest(), &v("1.1.9"), &appimage).unwrap().unwrap();
        assert_eq!(r.platform.signature, "s3");
        assert_eq!(r.platform.yubikey_signature, None);
        // Sem pacote para o sistema: erro (não é "nada novo").
        let win = targets("windows", "x86_64", Some(Installer::Nsis));
        assert!(parse(&manifest(), &v("1.1.9"), &win).is_err());
    }

    #[test]
    fn only_newer_versions_count() {
        let t = targets("darwin", "aarch64", Some(Installer::App));
        assert!(parse(&manifest(), &v("1.2.0"), &t).unwrap().is_none());
        assert!(parse(&manifest(), &v("1.3.0"), &t).unwrap().is_none());
        assert!(
            parse(&manifest(), &v("1.2.0-beta.1"), &t)
                .unwrap()
                .is_some()
        );
        let prefixed = br#"{"version": "v1.2.1", "platforms": {"darwin-aarch64": {"url": "https://exemplo.invalid/x", "signature": "s"}}}"#;
        assert_eq!(
            parse(prefixed, &v("1.2.0"), &t).unwrap().unwrap().version,
            v("1.2.1")
        );
        assert!(parse(b"{}", &v("1.0.0"), &t).is_err());
        assert!(parse(br#"{"version": "nova"}"#, &v("1.0.0"), &t).is_err());
    }

    #[test]
    fn packages_only_come_from_the_same_releases() {
        let endpoint =
            Url::parse("https://github.com/knuppe/treasury/releases/latest/download/latest.json")
                .unwrap();
        let ok = |u: &str| same_releases(&endpoint, &Url::parse(u).unwrap());
        assert!(ok(
            "https://github.com/knuppe/treasury/releases/download/v1.2.3/Treasury_1.2.3_aarch64.app.tar.gz"
        ));
        assert!(!ok(
            "http://github.com/knuppe/treasury/releases/download/v1.2.3/x.tar.gz"
        ));
        assert!(!ok(
            "https://exemplo.invalid/knuppe/treasury/releases/download/v1.2.3/x.tar.gz"
        ));
        assert!(!ok(
            "https://github.com/outro/treasury/releases/download/v1.2.3/x.tar.gz"
        ));
        assert!(!ok(
            "https://github.com/knuppe/treasury-falso/releases/download/v1/x.tar.gz"
        ));
        assert!(!ok(
            "https://github.com:8443/knuppe/treasury/releases/download/v1/x.tar.gz"
        ));
    }
}
