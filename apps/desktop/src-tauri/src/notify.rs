//! Notificações do sistema (docs/notifications.md): os alertas novos que as preferências
//! deixam passar (`RoutineReport::notices`) e o teste em Ajustes › Notificações. No macOS
//! pelo `osascript` (texto como argumento, nunca dentro do script), no Linux pelo
//! `notify-send`; no Windows por um toast do PowerShell (texto em variáveis de ambiente,
//! escapado para XML dentro do script; sem janela de console).

use std::process::Command;

use treasury_core::ApiError;

/// Toast do Windows 10/11 pela API do WinRT. O remetente é o do PowerShell (sempre
/// registrado); o título e o texto chegam por variáveis de ambiente e são escapados.
#[cfg(target_os = "windows")]
const WINDOWS_TOAST: &str = r#"
$ErrorActionPreference = 'Stop'
[void][Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime]
[void][Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime]
$t = [Security.SecurityElement]::Escape($env:TREASURY_NOTIFY_TITLE)
$b = [Security.SecurityElement]::Escape($env:TREASURY_NOTIFY_BODY)
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml("<toast><visual><binding template='ToastGeneric'><text>Treasury: $t</text><text>$b</text></binding></visual></toast>")
$app = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe'
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($app).Show([Windows.UI.Notifications.ToastNotification]::new($xml))
"#;

/// Mostra a notificação; falha sem barulho (só no log).
pub fn show(title: &str, body: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let status = Command::new("osascript")
        .args([
            "-e",
            "on run argv",
            "-e",
            "display notification (item 2 of argv) with title \"Treasury\" subtitle (item 1 of argv)",
            "-e",
            "end run",
            title,
            body,
        ])
        .status();
    #[cfg(target_os = "linux")]
    let status = Command::new("notify-send")
        .args(["--app-name=Treasury", title, body])
        .status();
    #[cfg(target_os = "windows")]
    let status = {
        use std::os::windows::process::CommandExt;
        /// Sem janela de console piscando.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                WINDOWS_TOAST,
            ])
            .env("TREASURY_NOTIFY_TITLE", title)
            .env("TREASURY_NOTIFY_BODY", body)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
    };
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let status: std::io::Result<std::process::ExitStatus> = {
        let _ = (title, body);
        return Err("notificações do sistema ainda não existem neste sistema".into());
    };
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("o sistema recusou a notificação ({s})")),
        Err(e) => Err(format!("não foi possível notificar: {e}")),
    }
}

/// Notificação de teste (botão em Ajustes › Notificações).
#[tauri::command]
pub fn system_notify(title: String, body: String) -> Result<(), ApiError> {
    let clip = |s: &str, n: usize| s.chars().take(n).collect::<String>();
    show(&clip(&title, 120), &clip(&body, 400)).map_err(|message| ApiError {
        code: "io".into(),
        message,
        details: None,
    })
}
