//! As buscas (docs/email-ingest.md §Buscas): no ciclo das rotinas do app (`ai::run_due`, a cada
//! 30 s), cada conta ligada é buscada quando passa o intervalo dela (15 min, de 5 a 120), numa
//! thread própria; "Buscar agora" dispara na hora. Só com o app aberto e o cofre desbloqueado.
//! Cada busca: lê o que é novo, grava na fila, aplica sozinho o que a regra deixa, chama a IA
//! (regra com IA automática), roda a retenção e guarda o resumo.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::params;

use super::config::{self, AccountKind, MailSyncStatus};
use super::ingest::{self, Outcome, Prepared};
use super::retention::{self, Plan};
use super::{ai, imap, local, store};
use crate::api::{Api, SharedApi};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::time::now_utc;

static RUNNING: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

fn running() -> std::sync::MutexGuard<'static, BTreeSet<String>> {
    RUNNING.lock().unwrap_or_else(|p| p.into_inner())
}

pub fn is_running(account_id: &str) -> bool {
    running().contains(account_id)
}

/// Contas que já passaram do intervalo (e estão prontas para buscar).
fn due(api: &Api) -> Result<Vec<Id>> {
    api.with_session(|s, _| {
        let now = chrono::Utc::now();
        Ok(config::list_accounts(s.db.conn(), Some(s.db.key()))?
            .into_iter()
            .filter(|a| a.enabled && !a.needs_reconfirm)
            .filter(|a| {
                a.kind == AccountKind::Folder
                    || a.password
                        .as_ref()
                        .is_some_and(|p| p.set && !p.needs_reentry)
            })
            .filter(|a| {
                a.status
                    .as_ref()
                    .and_then(|st| chrono::DateTime::parse_from_rfc3339(&st.at).ok())
                    .is_none_or(|at| {
                        now - at.with_timezone(&chrono::Utc)
                            >= chrono::Duration::minutes(a.interval_minutes as i64)
                    })
            })
            .map(|a| a.id)
            .collect())
    })
}

/// Chamado pelas rotinas do app: dispara as buscas vencidas.
pub fn run_due(api: &SharedApi) {
    if !api.is_unlocked() || api.paths().db_path.as_os_str().is_empty() {
        return;
    }
    let read_only = api
        .with_session(|s, _| Ok(s.db.is_read_only()))
        .unwrap_or(true);
    if read_only {
        return;
    }
    match due(api) {
        Ok(ids) => {
            for id in ids {
                let _ = start_as(api, &id, crate::tasks::TaskOrigin::Auto);
            }
        }
        Err(e) => tracing::warn!("e-mail de ingestão: {}", e.code()),
    }
}

/// Dispara a busca de uma conta numa thread (falso: já estava buscando).
pub fn start(api: &SharedApi, account_id: &str) -> Result<bool> {
    start_as(api, account_id, crate::tasks::TaskOrigin::User)
}

/// Como [`start`], dizendo quem pediu (a busca aparece nas tarefas em segundo plano).
pub fn start_as(
    api: &SharedApi,
    account_id: &str,
    origin: crate::tasks::TaskOrigin,
) -> Result<bool> {
    if !running().insert(account_id.to_string()) {
        return Ok(false);
    }
    let api = api.clone();
    let id = account_id.to_string();
    let spawned = std::thread::Builder::new()
        .name("treasury-mail".into())
        .spawn(move || {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::tasks::run(crate::tasks::TaskKind::MailSync, origin, false, || {
                    crate::tasks::stage(crate::tasks::TaskStage::Mail);
                    sync_account(&api, &id)
                })
            }));
            if let Ok(Err(e)) = &r {
                // Sem dados pessoais no log: só o código do erro.
                tracing::warn!("e-mail de ingestão: busca falhou ({})", e.code());
            }
            running().remove(&id);
        });
    if spawned.is_err() {
        running().remove(account_id);
        return Err(Error::Other(t("nucleo-email-sync-erro-iniciar")));
    }
    Ok(true)
}

/// Uma busca completa (bloqueante). O resumo vai para `mail.status:<conta>` e a auditoria.
pub fn sync_account(api: &SharedApi, account_id: &str) -> Result<MailSyncStatus> {
    let p = api.with_session(|s, _| Prepared::load(s.db.conn(), account_id))?;
    let res = match p.account.kind {
        AccountKind::Folder => sync_folder(api, &p),
        AccountKind::Imap => sync_imap(api, &p),
    };
    let status = match res {
        Ok(mut st) => {
            st.ok = true;
            st
        }
        Err(e) => MailSyncStatus {
            ok: false,
            message: e.message(),
            ..Default::default()
        },
    };
    let status = MailSyncStatus {
        at: now_utc(),
        ..status
    };
    api.with_session(|s, _| {
        let conn = s.db.conn();
        crate::settings::set(conn, &config::status_key(account_id), &status)?;
        store::audit(
            conn,
            Some(account_id),
            "sync",
            None,
            &if status.ok {
                let summary = t_args(
                    "nucleo-email-sync-resumo",
                    &args(&[
                        ("novas", status.seen.into()),
                        ("fila", status.passed.into()),
                        ("recusadas", status.rejected.into()),
                        ("aplicadas", status.applied.into()),
                    ]),
                );
                if status.message.is_empty() {
                    summary
                } else {
                    format!("{summary} ({})", status.message)
                }
            } else {
                t_args(
                    "nucleo-email-sync-falhou",
                    &args(&[("erro", status.message.clone().into())]),
                )
            },
        )
    })?;
    crate::events::emit_system("mail.sync");
    Ok(status)
}

/// O que vem depois de gravar: aplicar sozinho e chamar a IA.
fn after_store(api: &SharedApi, stored: &[(Id, Vec<Id>, Vec<Id>)], st: &mut MailSyncStatus) {
    for (_, auto, ai_parts) in stored {
        st.applied += ingest::apply_all(api, auto);
        for part in ai_parts {
            if let Err(e) = ai::start(api, part) {
                let _ = api.with_session(|s, _| {
                    store::set_part(
                        s.db.conn(),
                        part,
                        store::MailStatus::Unrecognized,
                        None,
                        Some(&format!("IA: {e}")),
                    )
                    .map(|_| ())
                });
            }
        }
    }
}

fn sync_folder(api: &SharedApi, p: &Prepared) -> Result<MailSyncStatus> {
    let acc = &p.account;
    let root = PathBuf::from(
        acc.folder_path
            .as_deref()
            .ok_or_else(|| Error::invalid(t("nucleo-email-sync-erro-sem-pasta")))?,
    );
    let files = local::scan(&root, &acc.allowed_folders, &acc.blocked_folders)?;
    let mut st = MailSyncStatus::default();
    let mut stored = vec![];
    for f in files {
        if !api.is_unlocked() {
            break;
        }
        if api.with_session(|s, _| local::known(s.db.conn(), &acc.id, &f))? {
            continue;
        }
        let inc = match local::incoming(&f) {
            Ok(i) => i,
            Err(_) => continue,
        };
        st.seen += 1;
        let out = api.with_session(|s, _| {
            let conn = s.db.conn();
            let o = ingest::store_incoming(conn, p, inc)?;
            if matches!(o, Outcome::Rejected(_) | Outcome::Known) {
                local::mark_seen(conn, &acc.id, &f)?;
            }
            Ok(o)
        })?;
        match out {
            Outcome::Rejected(_) => st.rejected += 1,
            Outcome::Known => {}
            Outcome::Limit => {
                st.message = t("nucleo-email-sync-limite");
                break;
            }
            Outcome::Stored {
                message_id,
                auto,
                ai,
            } => {
                st.passed += 1;
                stored.push((message_id, auto, ai));
            }
        }
    }
    after_store(api, &stored, &mut st);
    // Retenção: mover para Processados ou mandar para a Lixeira.
    let plan = api.with_session(|s, _| retention::plan(s.db.conn(), p, chrono::Utc::now()))?;
    if let Plan::Delete(list) = plan {
        let fallback = api.paths().root.join("removidos");
        let results: Vec<_> = list
            .into_iter()
            .map(|c| {
                let r = local::remove(&root, &c, &fallback);
                (c, r)
            })
            .collect();
        api.with_session(|s, _| retention::record(s.db.conn(), &acc.id, &results))?;
    }
    Ok(st)
}

fn sync_imap(api: &SharedApi, p: &Prepared) -> Result<MailSyncStatus> {
    let acc = &p.account;
    if acc.needs_reconfirm {
        return Err(Error::conflict(t("nucleo-email-erro-reconfirmar")));
    }
    let mut c = {
        let password = api.with_session(|s, _| {
            crate::ai::secret::get(s.db.conn(), s.db.key(), &config::secret_id(&acc.id))
        })?;
        let password =
            password.ok_or_else(|| Error::invalid(t("nucleo-email-sync-erro-sem-senha")))?;
        imap::connect(acc, &password)?
        // A senha decifrada (Zeroizing) some aqui.
    };
    let folders = imap::list_folders(&mut c, acc.provider.as_deref())?;
    let fetched = imap::fetch_new(
        &mut c,
        p,
        &folders,
        |folder| {
            api.with_session(|s, _| {
                use rusqlite::OptionalExtension;
                Ok(s.db
                    .conn()
                    .query_row(
                        "SELECT uidvalidity, last_uid FROM mail_folder_state WHERE account_id = ?1 AND folder = ?2",
                        params![acc.id, folder],
                        |r| Ok((r.get::<_, i64>(0)? as u32, r.get::<_, i64>(1)? as u32)),
                    )
                    .optional()?)
            })
        },
        |folder, validity, last| {
            api.with_session(|s, _| {
                s.db.conn().execute(
                    "INSERT INTO mail_folder_state (account_id, folder, uidvalidity, last_uid, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT (account_id, folder) DO UPDATE SET uidvalidity = excluded.uidvalidity,
                        last_uid = excluded.last_uid, updated_at = excluded.updated_at",
                    params![acc.id, folder, validity as i64, last as i64, now_utc()],
                )?;
                Ok(())
            })
        },
        |inc| api.with_session(|s, _| ingest::store_incoming(s.db.conn(), p, inc)),
    )?;
    let mut st = MailSyncStatus {
        seen: fetched.seen,
        rejected: fetched.rejected,
        passed: fetched.stored.len() as u32,
        message: if fetched.limit {
            t("nucleo-email-sync-limite")
        } else {
            String::new()
        },
        ..Default::default()
    };
    // Retenção ainda com a conexão aberta.
    let plan = api.with_session(|s, _| retention::plan(s.db.conn(), p, chrono::Utc::now()))?;
    if let Plan::Delete(list) = plan {
        let results = imap::delete(&mut c, acc, &folders, list);
        api.with_session(|s, _| retention::record(s.db.conn(), &acc.id, &results))?;
    }
    drop(c);
    after_store(api, &fetched.stored, &mut st);
    Ok(st)
}
