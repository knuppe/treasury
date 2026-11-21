//! Caixa de entrada de arquivos (docs/inbox.md): com o cofre aberto, a cada 3 s o
//! núcleo lê a pasta de entrada (`import::inbox::scan`; padrão `~/Treasury/Inbox`, ajuste
//! `watch_inbox`). O que chegou vira item da caixa (guardado no cofre) e sai da pasta; a
//! interface recebe `inbox://new` para avisar. A leitura vai direto ao núcleo (sem `api_call`):
//! não conta como atividade do usuário para o bloqueio automático.

use std::time::Duration;

use tauri::{AppHandle, Emitter, EventTarget, Manager};

use crate::AppState;

/// Intervalo entre as leituras da pasta.
const EVERY: Duration = Duration::from_secs(3);

pub fn spawn(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("treasury-inbox".into())
        .spawn(move || {
            let mut warned = false;
            loop {
                std::thread::sleep(EVERY);
                let api = &app.state::<AppState>().api;
                if !api.is_unlocked() {
                    continue;
                }
                match api.with_session(|s, _| treasury_core::import::inbox::scan(s.db.conn())) {
                    Ok(r) => {
                        warned = false;
                        for item in r.added {
                            let _ = app.emit_to(EventTarget::webview("main"), "inbox://new", item);
                        }
                    }
                    Err(e) => {
                        if !warned {
                            tracing::warn!("caixa de entrada: {e}");
                            warned = true;
                        }
                    }
                }
            }
        });
    if let Err(e) = spawned {
        tracing::warn!("acompanhamento da caixa de entrada: {e}");
    }
}
