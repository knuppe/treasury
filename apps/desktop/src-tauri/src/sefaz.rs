//! Consulta da NFC-e numa janela do próprio app (docs/ocr-receipts.md §4): para as SEFAZ que
//! pedem captcha, a pessoa resolve o captcha ali, como faria no navegador, e o app lê o texto da
//! página do cupom quando ela aparece. Nada contorna o captcha.
//!
//! - Só `https` em domínios `.gov.br`; a janela não navega para fora deles. A exceção são os
//!   quadros dos serviços de captcha que as SEFAZ usam (Cloudflare Turnstile em SC, reCAPTCHA,
//!   hCaptcha) e os quadros vazios (`about:blank`): o WKWebView passa a navegação de cada quadro
//!   pela mesma regra, e sem eles o captcha não carrega e a SEFAZ responde "Não foi possível
//!   validar o CAPTCHA". Liberar o quadro só deixa a pessoa resolver o captcha; nada o contorna.
//! - A janela não tem nenhuma permissão (a capability é só da webview `main`): o texto volta por
//!   uma navegação para `treasury-sefaz://`, que é interceptada e cancelada aqui, e chega à
//!   interface no evento `sefaz://page`. A interface lê o texto com o leitor de sempre
//!   (`receipts.preview` com `text`) e mostra a prévia antes de gravar.
//! - A sessão da janela (cookies do captcha) some quando ela fecha.

use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use treasury_core::ApiError;

const LABEL: &str = "sefaz";
const SCHEME: &str = "treasury-sefaz";
/// Evento com o texto da página do cupom.
pub const EVENT: &str = "sefaz://page";

/// Manda o texto da página quando ela tem cara de cupom (ou sempre, com `force`).
fn capture_script(force: bool) -> String {
    format!(
        "(function(){{try{{var t=(document.body&&document.body.innerText)||'';\
         if({force}||/valor a pagar|qtd\\.? ?total de itens|quantidade total de itens|valor total r\\$/i.test(t)){{\
         location.href='{SCHEME}://captura/#'+encodeURIComponent(t.slice(0,200000));}}}}catch(e){{}}}})();"
    )
}

fn err(message: impl Into<String>) -> ApiError {
    ApiError {
        code: "invalid".into(),
        message: message.into(),
        details: None,
    }
}

fn allowed(u: &Url) -> bool {
    u.scheme() == "https"
        && u.host_str()
            .is_some_and(|h| h == "gov.br" || h.ends_with(".gov.br"))
}

/// Serviços de captcha das páginas de consulta (o quadro do desafio vem deles).
const CAPTCHA_HOSTS: &[&str] = &[
    "challenges.cloudflare.com",
    "www.google.com",
    "www.gstatic.com",
    "www.recaptcha.net",
    "hcaptcha.com",
    "newassets.hcaptcha.com",
    "js.hcaptcha.com",
];

/// O que a janela pode carregar: a consulta (`.gov.br`), os quadros do captcha e os quadros
/// vazios que as páginas criam. No Google, só os caminhos do reCAPTCHA.
fn allowed_in_window(u: &Url) -> bool {
    if allowed(u) {
        return true;
    }
    if u.scheme() == "about" {
        return matches!(u.path(), "blank" | "srcdoc");
    }
    let Some(host) = u.host_str() else {
        return false;
    };
    u.scheme() == "https"
        && CAPTCHA_HOSTS.contains(&host)
        && (!matches!(host, "www.google.com" | "www.gstatic.com")
            || u.path().starts_with("/recaptcha/"))
}

/// `%XX` → bytes (o texto vem com `encodeURIComponent`).
fn percent_decode(s: &str) -> String {
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match (
            b[i],
            b.get(i + 1).copied().and_then(hex),
            b.get(i + 2).copied().and_then(hex),
        ) {
            (b'%', Some(h), Some(l)) => {
                out.push(h << 4 | l);
                i += 3;
            }
            (c, _, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Abre (ou traz para frente) a janela da consulta no link do QR.
#[tauri::command]
pub async fn sefaz_open(app: AppHandle, url: String) -> Result<(), ApiError> {
    let u = Url::parse(url.trim()).map_err(|e| err(e.to_string()))?;
    if !allowed(&u) {
        return Err(err(
            "a consulta precisa ser um endereço https de um site .gov.br",
        ));
    }
    if let Some(w) = app.get_webview_window(LABEL) {
        w.navigate(u).map_err(|e| err(e.to_string()))?;
        let _ = w.set_focus();
        return Ok(());
    }
    let handle = app.clone();
    WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::External(u))
        .title("SEFAZ")
        .inner_size(520.0, 820.0)
        .incognito(true)
        .on_navigation(move |u| {
            if u.scheme() == SCHEME {
                let text = percent_decode(u.fragment().unwrap_or(""));
                let _ = handle.emit_to("main", EVENT, text);
                let h = handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(w) = h.get_webview_window(LABEL) {
                        let _ = w.close();
                    }
                });
                return false;
            }
            allowed_in_window(u)
        })
        .on_page_load(|w, p| {
            if p.event() == PageLoadEvent::Finished {
                let _ = w.eval(capture_script(false));
            }
        })
        .build()
        .map_err(|e| err(e.to_string()))?;
    Ok(())
}

/// Lê a página aberta agora, mesmo sem cara de cupom (quando a detecção automática falha).
#[tauri::command]
pub async fn sefaz_capture(app: AppHandle) -> Result<bool, ApiError> {
    match app.get_webview_window(LABEL) {
        Some(w) => {
            w.eval(capture_script(true))
                .map_err(|e| err(e.to_string()))?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_gov_br() {
        let ok = |s: &str| allowed(&Url::parse(s).unwrap());
        assert!(ok("https://www.fazenda.pr.gov.br/nfce/qrcode?p=1"));
        assert!(ok("https://nfce.sefaz.pe.gov.br/x"));
        assert!(!ok("http://www.fazenda.pr.gov.br/nfce"));
        assert!(!ok("https://gov.br.example.com/"));
        assert!(!ok("https://exemplo.com/?u=gov.br"));
    }

    #[test]
    fn the_captcha_frames_load_and_nothing_else() {
        let ok = |s: &str| allowed_in_window(&Url::parse(s).unwrap());
        // Cloudflare Turnstile (SEFAZ of SC) and empty frames.
        assert!(ok(
            "https://challenges.cloudflare.com/cdn-cgi/challenge-platform/h/b/turnstile/if/ov2/av0/rcv/abc"
        ));
        assert!(ok("about:blank"));
        assert!(ok("about:srcdoc"));
        // reCAPTCHA only through its own paths; hCaptcha.
        assert!(ok("https://www.google.com/recaptcha/api2/anchor?k=x"));
        assert!(ok("https://www.recaptcha.net/recaptcha/api2/bframe"));
        assert!(ok(
            "https://newassets.hcaptcha.com/captcha/v1/abc/static/hcaptcha.html"
        ));
        assert!(!ok("https://www.google.com/search?q=nfce"));
        assert!(!ok("http://challenges.cloudflare.com/x"));
        assert!(!ok("https://exemplo.com/"));
        assert!(!ok("about:config"));
        // The lookup goes on.
        assert!(ok("https://sat.sef.sc.gov.br/nfce/consulta?p=1"));
    }

    #[test]
    fn decodes_the_captured_text() {
        assert_eq!(
            percent_decode("ARROZ%205KG%0AVALOR%20A%20PAGAR%20R%24%2024%2C90%20%C3%A7"),
            "ARROZ 5KG\nVALOR A PAGAR R$ 24,90 ç"
        );
        assert!(capture_script(false).contains("treasury-sefaz://captura/#"));
    }
}
