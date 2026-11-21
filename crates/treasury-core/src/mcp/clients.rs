//! Aviso de cliente MCP novo (docs/local-security.md, item 14). Cada conexão externa aceita
//! pelo socket é comparada com os clientes já vistos neste cofre (nome do cliente + programa
//! que abriu o `treasury mcp`); um par novo vira um aviso na tela de Integrações MCP, com o
//! programa e o PID, até a pessoa dispensar. Detecção, não bloqueio: quem tem a chave entra,
//! mas não entra em silêncio.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;
use crate::settings::{get_or, set};

pub const KNOWN_SETTING: &str = "mcp.known_clients";
pub const NOTICES_SETTING: &str = "mcp.new_client_notices";
const MAX_KNOWN: usize = 50;
const MAX_NOTICES: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct McpClientNotice {
    /// Nome que o cliente informou (já limpo).
    pub client: String,
    /// Caminho do programa por trás da conexão (quem abriu o `treasury mcp`), se deu para ler.
    pub program: Option<String>,
    pub pid: Option<i32>,
    /// UTC RFC 3339.
    pub at: String,
}

/// Registra uma conexão aceita; `true` quando o par cliente + programa é novo.
pub fn note(
    conn: &Connection,
    client: &str,
    program: Option<&str>,
    pid: Option<i32>,
) -> Result<bool> {
    let id = format!("{client}|{}", program.unwrap_or_default());
    let mut known: Vec<String> = get_or(conn, KNOWN_SETTING, Vec::new())?;
    if known.contains(&id) {
        return Ok(false);
    }
    known.push(id);
    let extra = known.len().saturating_sub(MAX_KNOWN);
    known.drain(..extra);
    set(conn, KNOWN_SETTING, &known)?;
    let mut list = notices(conn)?;
    list.push(McpClientNotice {
        client: client.to_string(),
        program: program.map(str::to_string),
        pid,
        at: crate::time::now_utc(),
    });
    let extra = list.len().saturating_sub(MAX_NOTICES);
    list.drain(..extra);
    set(conn, NOTICES_SETTING, &list)?;
    Ok(true)
}

pub fn notices(conn: &Connection) -> Result<Vec<McpClientNotice>> {
    get_or(conn, NOTICES_SETTING, Vec::new())
}

pub fn dismiss(conn: &Connection) -> Result<()> {
    crate::settings::delete(conn, NOTICES_SETTING)
}

/// Programa e PID por trás de uma conexão do socket. Quando o processo do outro lado é o
/// próprio `treasury mcp`, quem interessa é quem o abriu (Claude Desktop, um terminal, um
/// script): devolve o pai.
pub fn peer_program(pid: Option<i32>) -> (Option<String>, Option<i32>) {
    let Some(pid) = pid.filter(|p| *p > 0) else {
        return (None, None);
    };
    let path = proc::path(pid);
    let ours = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
    let is_treasury = path.as_deref().is_some_and(|p| {
        ours.as_deref()
            .is_some_and(|o| std::path::Path::new(p).file_name().is_some_and(|n| n == o))
    });
    if is_treasury && let Some(parent) = proc::parent(pid).filter(|p| *p > 1) {
        return (proc::path(parent), Some(parent));
    }
    (path, Some(pid))
}

#[cfg(target_os = "macos")]
mod proc {
    pub fn path(pid: i32) -> Option<String> {
        let mut buf = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
        // SAFETY: buffer do tamanho informado; a função escreve no máximo isso.
        let n = unsafe { libc::proc_pidpath(pid, buf.as_mut_ptr().cast(), buf.len() as u32) };
        (n > 0).then(|| String::from_utf8_lossy(&buf[..n as usize]).into_owned())
    }

    pub fn parent(pid: i32) -> Option<i32> {
        // SAFETY: `proc_bsdinfo` é só números; zerado é um valor válido.
        let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
        // SAFETY: buffer do tamanho da estrutura pedida (PROC_PIDTBSDINFO).
        let n = unsafe {
            libc::proc_pidinfo(
                pid,
                libc::PROC_PIDTBSDINFO,
                0,
                (&mut info as *mut libc::proc_bsdinfo).cast(),
                size,
            )
        };
        (n == size).then_some(info.pbi_ppid as i32)
    }
}

#[cfg(target_os = "linux")]
mod proc {
    pub fn path(pid: i32) -> Option<String> {
        std::fs::read_link(format!("/proc/{pid}/exe"))
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    }

    pub fn parent(pid: i32) -> Option<i32> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        // pid (nome) estado ppid …; o nome pode ter espaços e parênteses.
        stat.rsplit_once(')')?
            .1
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod proc {
    pub fn path(_pid: i32) -> Option<String> {
        None
    }

    pub fn parent(_pid: i32) -> Option<i32> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_pair_is_noticed_once_and_dismissed() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::storage::Db::create(
            &dir.path().join("t.db"),
            "senha-de-teste",
            crate::vault::KDF_TEST,
        )
        .unwrap();
        let c = db.conn();
        assert!(
            note(
                c,
                "Cliente A",
                Some("/Applications/A.app/Contents/MacOS/A"),
                Some(10)
            )
            .unwrap()
        );
        assert!(
            !note(
                c,
                "Cliente A",
                Some("/Applications/A.app/Contents/MacOS/A"),
                Some(11)
            )
            .unwrap()
        );
        // Mesmo nome, outro programa: novo (alguém se passando pelo cliente conhecido).
        assert!(note(c, "Cliente A", Some("/tmp/script"), Some(12)).unwrap());
        let list = notices(c).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[1].program.as_deref(), Some("/tmp/script"));
        assert_eq!(list[1].pid, Some(12));
        dismiss(c).unwrap();
        assert!(notices(c).unwrap().is_empty());
        assert!(!note(c, "Cliente A", Some("/tmp/script"), None).unwrap());
    }

    /// O próprio processo de teste é o "nosso binário": quem aparece é o pai (o cargo).
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn the_peer_program_is_read_from_the_system() {
        let me = std::process::id() as i32;
        let exe = std::env::current_exe().unwrap();
        let name = exe.file_name().unwrap().to_string_lossy().into_owned();
        assert!(proc::path(me).unwrap().ends_with(&name));
        let parent = proc::parent(me).unwrap();
        let (path, pid) = peer_program(Some(me));
        assert_eq!(pid, Some(parent));
        assert!(path.is_some());
        assert_eq!(peer_program(None), (None, None));
    }
}
