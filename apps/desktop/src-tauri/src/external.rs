//! Links para fora do app (documentação, painéis das fontes de dados, sites dos bancos,
//! links das respostas do agente) abrem no navegador padrão do sistema, nunca na própria
//! janela do Treasury. Só `http(s)`: nada de esquemas que rodem programas.

use std::process::Command;

use tauri::Url;
use treasury_core::ApiError;

fn err(code: &str, message: impl Into<String>) -> ApiError {
    ApiError {
        code: code.into(),
        message: message.into(),
        details: None,
    }
}

/// O endereço aceito: `http`/`https` com host.
fn checked(url: &str) -> Result<Url, ApiError> {
    let u = Url::parse(url.trim()).map_err(|_| err("invalid", "endereço inválido"))?;
    if !matches!(u.scheme(), "http" | "https") || u.host_str().is_none() {
        return Err(err(
            "invalid",
            "só endereços http e https abrem no navegador",
        ));
    }
    Ok(u)
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), ApiError> {
    let u = checked(&url)?;
    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut cmd = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("rundll32");
        c.arg("url.dll,FileProtocolHandler");
        c
    };
    cmd.arg(u.as_str())
        .spawn()
        .map(|_| ())
        .map_err(|e| err("io", format!("não foi possível abrir o navegador: {e}")))
}

#[cfg(test)]
mod tests {
    use super::checked;

    #[test]
    fn only_web_addresses_open() {
        assert!(checked("https://brapi.dev/dashboard").is_ok());
        assert!(checked(" http://www.bcb.gov.br ").is_ok());
        for bad in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "x-apple.systempreferences:",
            "ssh://host",
            "https://",
            "nada",
        ] {
            assert!(checked(bad).is_err(), "{bad}");
        }
    }
}
