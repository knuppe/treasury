//! Autenticação do remetente (docs/email-ingest.md §Segurança). Vale só o Authentication-Results
//! mais acima, e só se o authserv-id for o do provedor da conta (ex.: `mx.google.com`): um
//! cabeçalho igual escrito pelo remetente fica abaixo e não conta. Autenticado = DMARC pass, ou
//! DKIM pass com o domínio alinhado ao "De". Numa encaminhada, a autenticação vale para quem
//! encaminhou; o remetente original conta como não autenticado. Não autenticado não é
//! bloqueado: só perde o "aplicar sozinho".

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::i18n::{args, t, t_args};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailAuthCheck")]
pub struct AuthCheck {
    /// O "De" está autenticado pelo provedor.
    pub authenticated: bool,
    /// Em uma frase ("DMARC pass (mx.google.com)", "sem Authentication-Results do provedor").
    pub detail: String,
}

fn domain(addr: &str) -> String {
    addr.rsplit_once('@')
        .map_or(addr, |(_, d)| d)
        .trim()
        .trim_end_matches('>')
        .to_ascii_lowercase()
}

/// Alinhamento relaxado: o domínio assinado é o do "De" ou um domínio pai dele.
fn aligned(signed: &str, from_domain: &str) -> bool {
    let s = signed.trim().trim_start_matches('@').to_ascii_lowercase();
    !s.is_empty() && (from_domain == s || from_domain.ends_with(&format!(".{s}")))
}

/// O authserv-id casa com a lista do provedor: nome exato ou `*.dominio` (Yahoo e Fastmail
/// mudam o nome do servidor a cada mensagem).
pub fn authserv_matches(list: &str, id: &str) -> bool {
    let id = id.trim().to_ascii_lowercase();
    !id.is_empty()
        && list
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .any(|p| {
                let p = p.to_ascii_lowercase();
                match p.strip_prefix("*.") {
                    Some(suffix) => id.ends_with(&format!(".{suffix}")),
                    None => id == p,
                }
            })
}

fn authserv_of(header: &str) -> String {
    header
        .split(';')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Confere o "De" contra os Authentication-Results do provedor. Valem só os de cima, em
/// sequência, com o authserv-id da lista (`authserv`, separada por vírgula): o iCloud e o
/// Proton escrevem um cabeçalho por verificação. O primeiro de outro servidor encerra o bloco.
pub fn check(auth_results: &[String], authserv: Option<&str>, from: &str) -> AuthCheck {
    let Some(expected) = authserv.map(str::trim).filter(|s| !s.is_empty()) else {
        return AuthCheck {
            authenticated: false,
            detail: t("nucleo-email-auth-sem-provedor"),
        };
    };
    let Some(top) = auth_results.first() else {
        return AuthCheck {
            authenticated: false,
            detail: t("nucleo-email-auth-sem-cabecalho"),
        };
    };
    if !authserv_matches(expected, &authserv_of(top)) {
        return AuthCheck {
            authenticated: false,
            detail: t_args(
                "nucleo-email-auth-outro-servidor",
                &args(&[("servidor", authserv_of(top).into())]),
            ),
        };
    }
    let block: Vec<&String> = auth_results
        .iter()
        .take_while(|h| authserv_matches(expected, &authserv_of(h)))
        .collect();
    let id = authserv_of(top);
    let from_domain = domain(from);
    let mut dkim_ok = false;
    for header in block {
        for res in header.split(';').skip(1) {
            let low = res.trim().to_ascii_lowercase();
            let props = |key: &str| -> Option<String> {
                low.split_whitespace().find_map(|t| {
                    t.strip_prefix(&format!("{key}="))
                        .map(|v| v.trim_matches(|c| c == ';' || c == '"').to_string())
                })
            };
            if low.starts_with("dmarc=pass") {
                let hf = props("header.from");
                if hf.as_deref().is_none_or(|d| aligned(d, &from_domain)) {
                    return AuthCheck {
                        authenticated: true,
                        detail: format!("DMARC pass ({id})"),
                    };
                }
            }
            if low.starts_with("dkim=pass") {
                let d = props("header.d").or_else(|| props("header.i").map(|i| domain(&i)));
                if d.as_deref().is_some_and(|d| aligned(d, &from_domain)) {
                    dkim_ok = true;
                }
            }
        }
    }
    if dkim_ok {
        AuthCheck {
            authenticated: true,
            detail: t_args(
                "nucleo-email-auth-dkim",
                &args(&[("servidor", id.clone().into())]),
            ),
        }
    } else {
        AuthCheck {
            authenticated: false,
            detail: t_args(
                "nucleo-email-auth-falhou",
                &args(&[("servidor", id.clone().into())]),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_providers_topmost_header_counts() {
        let good = vec![
            "mx.google.com; dkim=pass header.i=@nubank.com.br header.s=s1 header.b=abc; spf=pass smtp.mailfrom=x@nubank.com.br; dmarc=pass (p=REJECT sp=REJECT dis=NONE) header.from=nubank.com.br".to_string(),
        ];
        let c = check(&good, Some("mx.google.com"), "todomundo@nubank.com.br");
        assert!(c.authenticated, "{c:?}");
        // Another authserv-id on top (written by the sender): does not count.
        let forged = vec![
            "evil.example; dmarc=pass header.from=nubank.com.br".to_string(),
            good[0].clone(),
        ];
        assert!(!check(&forged, Some("mx.google.com"), "todomundo@nubank.com.br").authenticated);
        // DMARC from another domain does not authenticate the "From".
        assert!(!check(&good, Some("mx.google.com"), "x@itau.com.br").authenticated);
        // Aligned DKIM (subdomain of the "From") is enough.
        let dkim = vec!["mx.google.com; dkim=pass header.d=itau.com.br; dmarc=none".to_string()];
        assert!(check(&dkim, Some("mx.google.com"), "a@mail.itau.com.br").authenticated);
        let unaligned =
            vec!["mx.google.com; dkim=pass header.d=mailchimp.com; dmarc=fail".to_string()];
        assert!(!check(&unaligned, Some("mx.google.com"), "a@itau.com.br").authenticated);
        // Local folder: there is no provider.
        assert!(!check(&good, None, "todomundo@nubank.com.br").authenticated);
        // iCloud: one header per check, all of them on top and from the list.
        let icloud = "spf.icloud.com,dkim-verifier.icloud.com,dmarc.icloud.com";
        let split = vec![
            "dmarc.icloud.com; dmarc=pass header.from=nubank.com.br".to_string(),
            "dkim-verifier.icloud.com; dkim=pass header.d=nubank.com.br".to_string(),
            "spf.icloud.com; spf=pass smtp.mailfrom=nubank.com.br".to_string(),
        ];
        assert!(check(&split, Some(icloud), "a@nubank.com.br").authenticated);
        // The block ends at the first one from another server: the DMARC written below does not
        // count.
        let broken = vec![
            "spf.icloud.com; spf=pass smtp.mailfrom=nubank.com.br".to_string(),
            "evil.example; x=1".to_string(),
            "dmarc.icloud.com; dmarc=pass header.from=nubank.com.br".to_string(),
        ];
        assert!(!check(&broken, Some(icloud), "a@nubank.com.br").authenticated);
        // Yahoo: the server name changes; the suffix counts.
        let yahoo = vec![
            "atlas321.free.mail.ne1.yahoo.com; dmarc=pass(p=REJECT) header.from=itau.com.br"
                .to_string(),
        ];
        assert!(check(&yahoo, Some("*.yahoo.com"), "x@itau.com.br").authenticated);
        assert!(!authserv_matches("*.yahoo.com", "yahoo.com.evil.net"));
    }
}
