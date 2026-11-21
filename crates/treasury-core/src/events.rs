//! Avisos de dados alterados, para as telas abertas se atualizarem em tempo real quando o
//! agente, um cliente MCP ou uma rotina grava algo. Todo comando que não é só leitura e
//! termina bem vira um [`DataEvent`] num canal de difusão: o app repassa à interface
//! (`data://changed`) e o servidor MCP HTTP serve como SSE em `GET /events`.
//!
//! O aviso diz só o comando, o módulo e quem fez; nunca valores, descrições ou argumentos.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use ts_rs::TS;

use crate::api::Actor;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataEvent {
    /// Sequência crescente no processo (para descartar repetidos).
    #[ts(type = "number")]
    pub seq: u64,
    /// UTC RFC 3339.
    pub at: String,
    /// Comando que gravou (`modulo.acao`).
    pub cmd: String,
    pub module: String,
    /// `user`, `mcp` (agente do app ou cliente externo) ou `system`.
    pub actor: String,
    /// Nome do cliente MCP, quando foi um.
    pub client: Option<String>,
}

const CAPACITY: usize = 256;
static BUS: OnceLock<broadcast::Sender<DataEvent>> = OnceLock::new();
static SEQ: AtomicU64 = AtomicU64::new(0);

fn bus() -> &'static broadcast::Sender<DataEvent> {
    BUS.get_or_init(|| broadcast::channel(CAPACITY).0)
}

/// Novo ouvinte (quem fica para trás perde os mais velhos, sem travar ninguém).
pub fn subscribe() -> broadcast::Receiver<DataEvent> {
    bus().subscribe()
}

/// Comandos que gravam mas não mudam o que as telas mostram (ou já se atualizam sozinhos).
fn quiet(cmd: &str) -> bool {
    matches!(
        cmd,
        "ai.chat_poll"
            | "ai.jobs.list"
            | "ai.routines.status"
            | "ai.detect_clis"
            | "ai.test_engine"
            | "ai.attachments.prepare"
            | "ai.attachments.discard"
            | "market.search"
            | "physical.fipe_options"
            | "import.detect"
            | "import.sniff"
            | "import.preview"
            | "notes.preview"
    ) || cmd.starts_with("vault.")
}

/// Chamado por `Api::call_as` depois de um comando de escrita que deu certo.
pub fn after_command(actor: &Actor, cmd: &str) {
    if quiet(cmd) || bus().receiver_count() == 0 {
        return;
    }
    let (actor, client) = match actor {
        Actor::User => ("user", None),
        Actor::Mcp { client, .. } => ("mcp", Some(client.clone())),
        Actor::System | Actor::Mail { .. } => ("system", None),
    };
    let _ = bus().send(DataEvent {
        seq: SEQ.fetch_add(1, Ordering::Relaxed) + 1,
        at: crate::time::now_utc(),
        cmd: cmd.to_string(),
        module: cmd.split_once('.').map_or(cmd, |(m, _)| m).to_string(),
        actor: actor.to_string(),
        client,
    });
}

/// Rotinas fora dos comandos (coletas automáticas) também avisam.
pub fn emit_system(cmd: &str) {
    after_command(&Actor::System, cmd);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::{Actor, Api};
    use crate::paths::TreasuryPaths;

    #[test]
    fn writes_are_announced_reads_are_not() {
        let dir = tempfile::tempdir().unwrap();
        let api =
            Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
                .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        let mut rx = super::subscribe();
        api.call("accounts.list", json!({})).unwrap();
        let mcp = Actor::Mcp {
            client: "teste".into(),
            read_only: false,
        };
        api.call_as(
            &mcp,
            "accounts.create",
            json!({"name": "Conta", "kind": "checking"}),
        )
        .unwrap();
        // Other tests run in parallel in the same process: look for ours.
        let mut found = None;
        while let Ok(e) = rx.try_recv() {
            assert_ne!(e.cmd, "accounts.list");
            if e.cmd == "accounts.create" && e.client.as_deref() == Some("teste") {
                found = Some(e);
            }
        }
        let e = found.expect("write notice");
        assert_eq!(e.module, "accounts");
        assert_eq!(e.actor, "mcp");
    }
}
