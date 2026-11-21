//! Provedores IMAP com senha de app (docs/email-ingest.md §Provedores), conferidos nas páginas
//! de ajuda de cada um e por conexão direta (CAPABILITY antes do login) em 19/10/2026. O
//! authserv-id é o nome que o provedor põe no Authentication-Results; vem de cabeçalhos reais
//! publicados, e o do Zoho ainda não foi confirmado (dá para trocar na conta).
//!
//! Fora do catálogo: Outlook.com e Hotmail (só OAuth2 desde 16/09/2024, sem senha de app),
//! OAuth do Google e JMAP ficam para depois; POP3 e a API do Gmail, fora.

use serde::{Deserialize, Serialize, Serializer};
use ts_rs::TS;

use super::config::Security;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

/// Serializa a chave do catálogo como o texto no idioma da instalação.
fn translated<S: Serializer>(key: &&'static str, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(&t(key))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum UsernameStyle {
    /// O endereço completo.
    Address,
    /// O nome antes do @ (iCloud; o endereço completo também costuma funcionar).
    LocalPart,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, rename = "MailProvider")]
pub struct Provider {
    pub id: &'static str,
    pub label: &'static str,
    pub host: &'static str,
    #[ts(type = "number")]
    pub port: u16,
    pub security: Security,
    pub username: UsernameStyle,
    /// authserv-ids aceitos (separados por vírgula; `*.dominio` vale para os servidores dele).
    pub authserv_id: Option<&'static str>,
    /// Onde criar a senha de app.
    pub app_password_url: &'static str,
    /// O que o usuário precisa saber, em uma ou duas frases (chave do catálogo; sai traduzida).
    #[serde(serialize_with = "translated")]
    pub help: &'static str,
    /// Gmail: apagar é sempre mover para a lixeira (tirar a marca só arquiva).
    pub gmail: bool,
    /// Servidores que não marcam as pastas (RFC 6154): o nome de cada pasta especial.
    #[ts(type = "Array<[string, string]>")]
    pub folder_names: &'static [(&'static str, &'static str)],
    #[ts(skip)]
    pub domains: &'static [&'static str],
}

pub const CATALOG: &[Provider] = &[
    Provider {
        id: "gmail",
        label: "Gmail",
        host: "imap.gmail.com",
        port: 993,
        security: Security::Tls,
        username: UsernameStyle::Address,
        authserv_id: Some("mx.google.com"),
        app_password_url: "https://myaccount.google.com/apppasswords",
        help: "nucleo-email-provedor-gmail",
        gmail: true,
        folder_names: &[],
        domains: &["gmail.com", "googlemail.com"],
    },
    Provider {
        id: "icloud",
        label: "iCloud Mail",
        host: "imap.mail.me.com",
        port: 993,
        security: Security::Tls,
        username: UsernameStyle::LocalPart,
        authserv_id: Some(
            "spf.icloud.com,dkim-verifier.icloud.com,dmarc.icloud.com,arc.icloud.com,bimi.icloud.com",
        ),
        app_password_url: "https://account.apple.com",
        help: "nucleo-email-provedor-icloud",
        gmail: false,
        folder_names: &[
            ("Deleted Messages", "\\Trash"),
            ("Junk", "\\Junk"),
            ("Sent Messages", "\\Sent"),
            ("Drafts", "\\Drafts"),
            ("Archive", "\\Archive"),
        ],
        domains: &["icloud.com", "me.com", "mac.com"],
    },
    Provider {
        id: "yahoo",
        label: "Yahoo Mail",
        host: "imap.mail.yahoo.com",
        port: 993,
        security: Security::Tls,
        username: UsernameStyle::Address,
        authserv_id: Some("*.yahoo.com"),
        app_password_url: "https://login.yahoo.com/account/security",
        help: "nucleo-email-provedor-yahoo",
        gmail: false,
        folder_names: &[],
        domains: &["yahoo.com", "yahoo.com.br", "ymail.com", "rocketmail.com"],
    },
    Provider {
        id: "fastmail",
        label: "Fastmail",
        host: "imap.fastmail.com",
        port: 993,
        security: Security::Tls,
        username: UsernameStyle::Address,
        authserv_id: Some("*.messagingengine.com"),
        app_password_url: "https://www.fastmail.help/hc/en-us/articles/360058752854-App-passwords",
        help: "nucleo-email-provedor-fastmail",
        gmail: false,
        folder_names: &[],
        domains: &["fastmail.com", "fastmail.fm"],
    },
    Provider {
        id: "zoho",
        label: "Zoho Mail",
        host: "imap.zoho.com",
        port: 993,
        security: Security::Tls,
        username: UsernameStyle::Address,
        authserv_id: Some("mx.zohomail.com,mx.zoho.com,mx.zoho.eu,mx.zoho.in"),
        app_password_url: "https://www.zoho.com/mail/help/imap-access.html",
        help: "nucleo-email-provedor-zoho",
        gmail: false,
        folder_names: &[],
        domains: &[
            "zoho.com",
            "zohomail.com",
            "zoho.eu",
            "zohomail.eu",
            "zoho.in",
            "zohomail.in",
        ],
    },
    Provider {
        id: "proton",
        label: "Proton Mail (Bridge)",
        host: "127.0.0.1",
        port: 1143,
        security: Security::Starttls,
        username: UsernameStyle::Address,
        authserv_id: Some("mail.protonmail.ch"),
        app_password_url: "https://proton.me/support/imap-smtp-and-pop3-setup",
        help: "nucleo-email-provedor-proton",
        gmail: false,
        folder_names: &[],
        domains: &["proton.me", "protonmail.com", "protonmail.ch", "pm.me"],
    },
];

/// Domínios sem senha de app (só OAuth2): a pasta local, por enquanto.
const OAUTH_ONLY: &[&str] = &[
    "outlook.com",
    "outlook.com.br",
    "hotmail.com",
    "hotmail.com.br",
    "live.com",
    "msn.com",
];

pub fn find(id: &str) -> Option<&'static Provider> {
    CATALOG.iter().find(|p| p.id == id)
}

fn domain_of(addr: &str) -> String {
    addr.rsplit_once('@')
        .map_or(addr, |(_, d)| d)
        .trim()
        .to_ascii_lowercase()
}

/// Configuração encontrada para um endereço.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MailDiscovery {
    /// Provedor do catálogo, se for um deles.
    pub provider: Option<String>,
    pub host: String,
    #[ts(type = "number")]
    pub port: u16,
    pub security: Security,
    /// Usuário sugerido.
    pub username: String,
    /// De onde veio: `catalog`, `autoconfig` (o servidor do domínio) ou `ispdb` (a base do
    /// Thunderbird).
    pub source: String,
}

/// Pelo catálogo (sem rede).
pub fn from_catalog(address: &str) -> Result<Option<MailDiscovery>> {
    let d = domain_of(address);
    if OAUTH_ONLY.contains(&d.as_str()) {
        return Err(Error::invalid(t("nucleo-email-provedor-outlook")));
    }
    Ok(CATALOG
        .iter()
        .find(|p| p.domains.contains(&d.as_str()))
        .map(|p| MailDiscovery {
            provider: Some(p.id.into()),
            host: p.host.into(),
            port: p.port,
            security: p.security,
            username: match p.username {
                UsernameStyle::Address => address.trim().to_ascii_lowercase(),
                UsernameStyle::LocalPart => address
                    .split('@')
                    .next()
                    .unwrap_or(address)
                    .trim()
                    .to_ascii_lowercase(),
            },
            source: "catalog".into(),
        }))
}

/// Lê o `config-v1.1.xml` (autoconfig do Thunderbird): o primeiro servidor IMAP com TLS.
pub fn parse_autoconfig(xml: &str, address: &str) -> Option<MailDiscovery> {
    let mut rest = xml;
    while let Some(i) = rest.find("<incomingServer") {
        let after = &rest[i..];
        let end = after.find("</incomingServer>")?;
        let block = &after[..end];
        rest = &after[end..];
        if !block[..block.find('>')?].contains("\"imap\"") {
            continue;
        }
        let tag = |name: &str| -> Option<String> {
            let open = format!("<{name}>");
            let s = block.find(&open)? + open.len();
            let e = block[s..].find(&format!("</{name}>"))? + s;
            Some(block[s..e].trim().to_string())
        };
        let security = match tag("socketType")?.to_ascii_uppercase().as_str() {
            "SSL" => Security::Tls,
            "STARTTLS" => Security::Starttls,
            _ => continue,
        };
        let host = tag("hostname")?.to_ascii_lowercase();
        if host.is_empty() || host.contains(['/', ' ', '@']) {
            continue;
        }
        let port: u16 = tag("port")?.parse().ok()?;
        let local = address.split('@').next().unwrap_or(address);
        let username = tag("username")
            .unwrap_or_else(|| "%EMAILADDRESS%".into())
            .replace("%EMAILADDRESS%", address)
            .replace("%EMAILLOCALPART%", local)
            .replace("%EMAILDOMAIN%", &domain_of(address));
        return Some(MailDiscovery {
            provider: None,
            host,
            port,
            security,
            username: username.trim().to_ascii_lowercase(),
            source: String::new(),
        });
    }
    None
}

/// Procura a configuração de um endereço: o catálogo, o autoconfig do próprio domínio
/// (`https://autoconfig.<domínio>/…` e `/.well-known/…`) e a base do Thunderbird. Só o
/// domínio sai do computador, nunca o endereço.
pub fn discover(address: &str) -> Result<MailDiscovery> {
    if let Some(d) = from_catalog(address)? {
        return Ok(d);
    }
    let domain = domain_of(address);
    if domain.is_empty()
        || !domain.contains('.')
        || !domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(Error::invalid(t("nucleo-email-provedor-erro-endereco")));
    }
    let agent = crate::indices::sync::agent();
    let urls = [
        (
            format!("https://autoconfig.{domain}/mail/config-v1.1.xml"),
            "autoconfig",
        ),
        (
            format!("https://{domain}/.well-known/autoconfig/mail/config-v1.1.xml"),
            "autoconfig",
        ),
        (
            format!("https://autoconfig.thunderbird.net/v1.1/{domain}"),
            "ispdb",
        ),
    ];
    for (url, source) in urls {
        if let Ok(xml) = crate::indices::sync::get_text(&agent, &url)
            && let Some(mut d) = parse_autoconfig(&xml, address.trim())
        {
            if d.security == Security::Starttls
                && !matches!(d.host.as_str(), "127.0.0.1" | "localhost")
            {
                // O Treasury exige TLS desde o início fora do Bridge local.
                if d.port == 143 {
                    d.port = 993;
                    d.security = Security::Tls;
                } else {
                    continue;
                }
            }
            d.source = source.into();
            return Ok(d);
        }
    }
    Err(Error::invalid(t_args(
        "nucleo-email-provedor-erro-descoberta",
        &args(&[("dominio", domain.into())]),
    )))
}

/// Marca de uma pasta pelo nome, para provedores que não marcam (iCloud).
pub fn flag_by_name(provider: Option<&str>, folder: &str) -> Option<&'static str> {
    let p = find(provider?)?;
    p.folder_names
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(folder))
        .map(|(_, f)| *f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_and_autoconfig() {
        let g = from_catalog("Familia@Gmail.com").unwrap().unwrap();
        assert_eq!(
            (g.host.as_str(), g.port, g.username.as_str()),
            ("imap.gmail.com", 993, "familia@gmail.com")
        );
        let i = from_catalog("ana@icloud.com").unwrap().unwrap();
        assert_eq!(i.username, "ana");
        assert!(from_catalog("x@hotmail.com").is_err());
        assert!(from_catalog("x@exemplo.com.br").unwrap().is_none());
        let xml = r#"<clientConfig version="1.1"><emailProvider id="x">
            <incomingServer type="pop3"><hostname>pop.x.com</hostname><port>995</port><socketType>SSL</socketType></incomingServer>
            <incomingServer type="imap"><hostname>imap.x.com</hostname><port>993</port><socketType>SSL</socketType>
            <username>%EMAILLOCALPART%</username></incomingServer></emailProvider></clientConfig>"#;
        let d = parse_autoconfig(xml, "ana@x.com").unwrap();
        assert_eq!(
            (d.host.as_str(), d.port, d.security, d.username.as_str()),
            ("imap.x.com", 993, Security::Tls, "ana")
        );
        assert_eq!(
            flag_by_name(Some("icloud"), "deleted messages"),
            Some("\\Trash")
        );
        assert_eq!(flag_by_name(Some("gmail"), "Trash"), None);
    }
}
