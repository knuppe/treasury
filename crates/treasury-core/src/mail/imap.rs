//! Fase 2: a caixa IMAP com senha de app (docs/email-ingest.md §IMAP). TLS obrigatório, com os
//! certificados das autoridades públicas (webpki-roots); a única exceção é o Proton Mail
//! Bridge no próprio computador, com o certificado dele fixado pelo SHA-256. A conexão dura
//! uma busca: conecta, lê, apaga o que a retenção mandar, sai; a senha decifrada vive só
//! nesse intervalo (e some com o cofre bloqueado).
//!
//! Por pasta permitida: UIDVALIDITY e o último UID visto. Primeira busca: os últimos 30 dias.
//! Antes de baixar, só os cabeçalhos (`BODY.PEEK[HEADER]`, sem marcar como lida); as que
//! passam nos filtros (e as encaminhadas, que só se decidem pelo corpo) são baixadas inteiras.

use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use imap_proto::{Capability, NameAttribute};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

use super::config::{MailAccount, Security};
use super::filter::{self, Envelope};
use super::ingest::{Incoming, Outcome, Prepared};
use super::message::{self, MAX_MESSAGE_BYTES};
use super::providers;
use super::retention::Candidate;
use crate::error::{Error, Result};
use crate::i18n::{self, args, t_args};

/// Texto do catálogo com um argumento.
fn with(key: &str, name: &'static str, value: impl Into<String>) -> String {
    t_args(key, &args(&[(name, value.into().into())]))
}

/// Mensagens novas por pasta e por busca (o resto fica para a próxima).
const MAX_PER_FOLDER: usize = 200;
/// Primeira busca: os últimos N dias.
pub const FIRST_SYNC_DAYS: i64 = 30;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const IO_TIMEOUT: Duration = Duration::from_secs(90);

type Tls = rustls::StreamOwned<rustls::ClientConnection, TcpStream>;
pub type Session = imap::Session<Tls>;

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// Certificado fixado (Proton Bridge local): vale o SHA-256 do certificado, e só ele.
#[derive(Debug)]
struct Pinned {
    sha256: Vec<u8>,
    provider: Arc<rustls::crypto::CryptoProvider>,
}

impl ServerCertVerifier for Pinned {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        if Sha256::digest(end_entity.as_ref()).as_slice() == self.sha256.as_slice() {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(i18n::t(
                "nucleo-email-imap-erro-certificado-bridge",
            )))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

/// SHA-256 (hex) do certificado: aceita o `cert.pem` exportado pelo Bridge ou o próprio hex.
pub fn fingerprint(input: &str) -> Result<String> {
    let t = input.trim();
    if t.contains("BEGIN CERTIFICATE") {
        use base64::Engine as _;
        let b64: String = t
            .lines()
            .skip_while(|l| !l.contains("BEGIN CERTIFICATE"))
            .skip(1)
            .take_while(|l| !l.contains("END CERTIFICATE"))
            .collect::<Vec<_>>()
            .join("");
        let der = base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|_| Error::invalid(i18n::t("nucleo-email-imap-erro-cert-base64")))?;
        return Ok(hex::encode(Sha256::digest(&der)));
    }
    let h = t.replace(':', "").to_ascii_lowercase();
    if h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(h)
    } else {
        Err(Error::invalid(i18n::t(
            "nucleo-email-imap-erro-cert-escolha",
        )))
    }
}

fn tls_config(pinned: Option<&str>) -> Result<Arc<rustls::ClientConfig>> {
    let p = provider();
    let builder = rustls::ClientConfig::builder_with_provider(p.clone())
        .with_safe_default_protocol_versions()
        .map_err(|e| Error::Other(format!("TLS: {e}")))?;
    let cfg = match pinned {
        Some(fp) => {
            let sha = hex::decode(fp)
                .map_err(|_| Error::invalid(i18n::t("nucleo-email-imap-erro-cert-invalido")))?;
            builder
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(Pinned {
                    sha256: sha,
                    provider: p,
                }))
                .with_no_client_auth()
        }
        None => {
            let mut roots = rustls::RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            builder.with_root_certificates(roots).with_no_client_auth()
        }
    };
    Ok(Arc::new(cfg))
}

fn tcp(host: &str, port: u16) -> Result<TcpStream> {
    let addrs: Vec<_> = (host, port)
        .to_socket_addrs()
        .map_err(|e| {
            Error::Other(t_args(
                "nucleo-email-imap-erro-servidor-nao-achado",
                &args(&[
                    ("servidor", host.to_string().into()),
                    ("erro", e.to_string().into()),
                ]),
            ))
        })?
        .collect();
    let mut last = None;
    for a in addrs {
        match TcpStream::connect_timeout(&a, CONNECT_TIMEOUT) {
            Ok(s) => {
                s.set_read_timeout(Some(IO_TIMEOUT))?;
                s.set_write_timeout(Some(IO_TIMEOUT))?;
                return Ok(s);
            }
            Err(e) => last = Some(e),
        }
    }
    let target = format!("{host}:{port}");
    Err(Error::Other(match last {
        Some(e) => t_args(
            "nucleo-email-imap-erro-conexao-detalhe",
            &args(&[("servidor", target.into()), ("erro", e.to_string().into())]),
        ),
        None => with("nucleo-email-imap-erro-conexao", "servidor", target),
    }))
}

fn wrap(host: &str, s: TcpStream, pinned: Option<&str>) -> Result<Tls> {
    let name = ServerName::try_from(host.to_string())
        .map_err(|_| Error::invalid(with("nucleo-email-imap-erro-servidor", "servidor", host)))?;
    let conn = rustls::ClientConnection::new(tls_config(pinned)?, name)
        .map_err(|e| Error::Other(format!("TLS: {e}")))?;
    let mut tls = rustls::StreamOwned::new(conn, s);
    // Aperto de mão agora, para o erro de certificado aparecer aqui (e não no login).
    while tls.conn.is_handshaking() {
        tls.conn.complete_io(&mut tls.sock).map_err(|e| {
            Error::Other(t_args(
                "nucleo-email-imap-erro-tls",
                &args(&[
                    ("servidor", host.to_string().into()),
                    ("erro", e.to_string().into()),
                ]),
            ))
        })?;
    }
    Ok(tls)
}

/// STARTTLS à mão (o Bridge local): lê a saudação, pede o TLS e espera o OK, sem ler nada
/// além da linha da resposta.
fn starttls(mut s: TcpStream) -> Result<TcpStream> {
    use std::io::{Read, Write};
    fn line(s: &mut TcpStream) -> Result<String> {
        let mut buf = vec![];
        let mut b = [0u8; 1];
        while buf.len() < 4096 {
            if s.read(&mut b)? == 0 {
                break;
            }
            buf.push(b[0]);
            if buf.ends_with(b"\r\n") {
                break;
            }
        }
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
    let greeting = line(&mut s)?;
    if !greeting.starts_with("* OK") {
        return Err(Error::Other(i18n::t("nucleo-email-imap-erro-nao-imap")));
    }
    s.write_all(b"a0 STARTTLS\r\n")?;
    for _ in 0..20 {
        let l = line(&mut s)?;
        if l.is_empty() {
            break;
        }
        if let Some(rest) = l.strip_prefix("a0 ") {
            if rest.starts_with("OK") {
                return Ok(s);
            }
            return Err(Error::Other(i18n::t(
                "nucleo-email-imap-erro-starttls-recusado",
            )));
        }
    }
    Err(Error::Other(i18n::t(
        "nucleo-email-imap-erro-starttls-sem-resposta",
    )))
}

fn imap_err(e: imap::Error) -> Error {
    Error::Other(format!("IMAP: {e}"))
}

/// Uma conexão aberta, com o que o servidor sabe fazer.
pub struct Conn {
    pub session: Session,
    pub caps: Vec<String>,
}

impl Conn {
    fn has(&self, cap: &str) -> bool {
        self.caps.iter().any(|c| c.eq_ignore_ascii_case(cap))
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        let _ = self.session.logout();
    }
}

/// Conecta e entra. A senha não aparece em erro nenhum.
pub fn connect(acc: &MailAccount, password: &str) -> Result<Conn> {
    let host = acc
        .host
        .as_deref()
        .ok_or_else(|| Error::invalid(i18n::t("nucleo-email-imap-erro-sem-servidor")))?;
    let port = acc.port.unwrap_or(993);
    let user = acc
        .username
        .as_deref()
        .ok_or_else(|| Error::invalid(i18n::t("nucleo-email-imap-erro-sem-usuario")))?;
    let local = matches!(host, "127.0.0.1" | "::1" | "localhost");
    let pinned = acc.pinned_cert.as_deref().filter(|_| local);
    if acc.security == Security::Starttls && !local {
        return Err(Error::invalid(i18n::t(
            "nucleo-email-imap-erro-starttls-local",
        )));
    }
    let tls = match acc.security {
        Security::Tls => wrap(host, tcp(host, port)?, pinned)?,
        Security::Starttls => wrap(host, starttls(tcp(host, port)?)?, pinned)?,
    };
    let mut client = imap::Client::new(tls);
    if acc.security == Security::Tls {
        client.read_greeting().map_err(imap_err)?;
    }
    let mut session = client.login(user, password).map_err(|(e, _)| match e {
        imap::Error::No(_) | imap::Error::Bad(_) => {
            Error::invalid(i18n::t("nucleo-email-imap-erro-login"))
        }
        other => imap_err(other),
    })?;
    // As capacidades mudam depois do login (Gmail e Bridge só mostram MOVE e UIDPLUS aí).
    let caps = session
        .capabilities()
        .map(|c| {
            c.iter()
                .map(|x| match x {
                    Capability::Imap4rev1 => "IMAP4rev1".to_string(),
                    Capability::Auth(a) => format!("AUTH={a}"),
                    Capability::Atom(a) => a.to_string(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(Conn { session, caps })
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailFolder")]
pub struct FolderInfo {
    pub name: String,
    /// Marcas RFC 6154 (`\Trash`, `\Junk`…), ou a do nome conhecido do provedor.
    pub flags: Vec<String>,
    pub selectable: bool,
}

fn attr_flag(a: &NameAttribute<'_>) -> Option<&'static str> {
    Some(match a {
        NameAttribute::All => "\\All",
        NameAttribute::Archive => "\\Archive",
        NameAttribute::Drafts => "\\Drafts",
        NameAttribute::Flagged => "\\Flagged",
        NameAttribute::Junk => "\\Junk",
        NameAttribute::Sent => "\\Sent",
        NameAttribute::Trash => "\\Trash",
        NameAttribute::NoSelect => "\\Noselect",
        _ => return None,
    })
}

pub fn list_folders(c: &mut Conn, provider: Option<&str>) -> Result<Vec<FolderInfo>> {
    let names = c.session.list(None, Some("*")).map_err(imap_err)?;
    let mut out = vec![];
    for n in names.iter() {
        let mut flags: Vec<String> = n
            .attributes()
            .iter()
            .filter_map(attr_flag)
            .map(str::to_string)
            .collect();
        if let Some(f) = providers::flag_by_name(provider, n.name())
            && !flags.iter().any(|x| x == f)
        {
            flags.push(f.into());
        }
        let selectable = !flags.iter().any(|f| f == "\\Noselect");
        flags.retain(|f| f != "\\Noselect");
        out.push(FolderInfo {
            name: n.name().to_string(),
            flags,
            selectable,
        });
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailConnectionTest")]
pub struct ConnectionTest {
    pub folders: Vec<FolderInfo>,
    /// Pastas que o Treasury vai ler (permitidas e não bloqueadas).
    pub reads: Vec<String>,
    /// Pasta da lixeira (para onde a retenção move).
    pub trash: Option<String>,
    /// Como a retenção apagaria nesse servidor, em uma frase.
    pub delete_how: String,
    pub capabilities: Vec<String>,
}

fn trash_of(folders: &[FolderInfo]) -> Option<&FolderInfo> {
    folders
        .iter()
        .find(|f| f.selectable && f.flags.iter().any(|x| x == "\\Trash"))
}

fn delete_how(acc: &MailAccount, c: &Conn, folders: &[FolderInfo]) -> String {
    let gmail = providers::find(acc.provider.as_deref().unwrap_or("")).is_some_and(|p| p.gmail);
    match (trash_of(folders), c.has("MOVE"), c.has("UIDPLUS")) {
        (Some(t), _, _) if gmail => with("nucleo-email-imap-apaga-gmail", "pasta", t.name.clone()),
        (Some(t), true, _) => with("nucleo-email-imap-apaga-move", "pasta", t.name.clone()),
        (Some(t), false, true) => with("nucleo-email-imap-apaga-copia", "pasta", t.name.clone()),
        (None, _, true) => i18n::t("nucleo-email-imap-apaga-marca"),
        _ => i18n::t("nucleo-email-imap-apaga-nao"),
    }
}

/// A pasta é lida? (permitida e não bloqueada, pelo nome ou pela marca)
fn reads(acc: &MailAccount, f: &FolderInfo) -> bool {
    f.selectable
        && acc
            .allowed_folders
            .iter()
            .any(|p| filter::folder_matches(p, &f.name, &f.flags))
        && !acc
            .blocked_folders
            .iter()
            .any(|p| filter::folder_matches(p, &f.name, &f.flags))
}

/// Testa a conexão: entra, lista as pastas e diz o que seria lido e como apagaria.
pub fn test(acc: &MailAccount, password: &str) -> Result<ConnectionTest> {
    let mut c = connect(acc, password)?;
    let folders = list_folders(&mut c, acc.provider.as_deref())?;
    Ok(ConnectionTest {
        reads: folders
            .iter()
            .filter(|f| reads(acc, f))
            .map(|f| f.name.clone())
            .collect(),
        trash: trash_of(&folders).map(|f| f.name.clone()),
        delete_how: delete_how(acc, &c, &folders),
        capabilities: c.caps.clone(),
        folders,
    })
}

/// Resultado de uma busca na caixa.
#[derive(Debug, Default)]
pub struct Fetched {
    pub seen: u32,
    pub rejected: u32,
    pub stored: Vec<(String, Vec<String>, Vec<String>)>,
    /// Parou no limite do dia.
    pub limit: bool,
}

/// `19-Sep-2026` (data do SEARCH do IMAP).
fn imap_date(d: chrono::NaiveDate) -> String {
    d.format("%d-%b-%Y").to_string()
}

/// Busca as mensagens novas das pastas lidas e grava as que passam (cada gravação segura o
/// cofre só o tempo dela). `store` recebe a mensagem já baixada.
pub fn fetch_new(
    c: &mut Conn,
    p: &Prepared,
    folders: &[FolderInfo],
    mut state: impl FnMut(&str) -> Result<Option<(u32, u32)>>,
    mut save_state: impl FnMut(&str, u32, u32) -> Result<()>,
    mut store: impl FnMut(Incoming) -> Result<Outcome>,
) -> Result<Fetched> {
    let acc = &p.account;
    let mut out = Fetched::default();
    let since = crate::time::today_local() - chrono::Duration::days(FIRST_SYNC_DAYS);
    for f in folders.iter().filter(|f| reads(acc, f)) {
        let mb = c.session.examine(&f.name).map_err(imap_err)?;
        let Some(uidvalidity) = mb.uid_validity else {
            continue;
        };
        let last = match state(&f.name)? {
            Some((v, l)) if v == uidvalidity => Some(l),
            // UIDVALIDITY mudou (ou primeira vez): recomeça pelos últimos 30 dias.
            _ => None,
        };
        let query = match last {
            Some(l) => format!("UID {}:*", l.saturating_add(1)),
            None => format!("SINCE {}", imap_date(since)),
        };
        let mut uids: Vec<u32> = c
            .session
            .uid_search(&query)
            .map_err(imap_err)?
            .into_iter()
            .filter(|u| last.is_none_or(|l| *u > l))
            .collect();
        uids.sort_unstable();
        uids.truncate(MAX_PER_FOLDER);
        let mut done = last.unwrap_or(0);
        'chunks: for chunk in uids.chunks(25) {
            let set = chunk
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let heads = c
                .session
                .uid_fetch(&set, "(UID RFC822.SIZE BODY.PEEK[HEADER])")
                .map_err(imap_err)?;
            let mut list: Vec<(u32, u32, Vec<u8>)> = heads
                .iter()
                .filter_map(|h| {
                    Some((
                        h.uid?,
                        h.size.unwrap_or(0),
                        h.header().unwrap_or_default().to_vec(),
                    ))
                })
                .collect();
            list.sort_by_key(|x| x.0);
            for (uid, size, header) in list {
                out.seen += 1;
                let decision = message::parse_headers(&header).map(|h| {
                    let forward = h.forward_hint
                        || filter::sender_matches("membros", &h.from, &acc.family_addresses);
                    let env = Envelope {
                        folder: f.name.clone(),
                        folder_flags: f.flags.clone(),
                        from: h.from.clone(),
                        to: h.to.clone(),
                        subject: h.subject.clone(),
                        date: h.date.clone(),
                        size: size as u64,
                        ..Default::default()
                    };
                    let d = p.pre_filter(&env);
                    // A encaminhada só se decide pelo remetente original, que está no corpo.
                    d.passed || forward
                });
                if decision != Some(true) {
                    out.rejected += 1;
                    done = uid;
                    continue;
                }
                if size as usize > MAX_MESSAGE_BYTES {
                    out.rejected += 1;
                    done = uid;
                    continue;
                }
                let body = c
                    .session
                    .uid_fetch(uid.to_string(), "BODY.PEEK[]")
                    .map_err(imap_err)?;
                let Some(raw) = body.iter().find_map(|b| b.body().map(<[u8]>::to_vec)) else {
                    done = uid;
                    continue;
                };
                match store(Incoming {
                    folder: f.name.clone(),
                    folder_flags: f.flags.clone(),
                    uid: Some(uid),
                    uidvalidity: Some(uidvalidity),
                    file_name: None,
                    raw,
                    is_message: true,
                    file_date: None,
                })? {
                    Outcome::Limit => {
                        out.limit = true;
                        break 'chunks;
                    }
                    Outcome::Rejected(_) => out.rejected += 1,
                    Outcome::Known => {}
                    Outcome::Stored {
                        message_id,
                        auto,
                        ai,
                    } => out.stored.push((message_id, auto, ai)),
                }
                done = uid;
            }
        }
        if done > 0 || last.is_none() {
            save_state(&f.name, uidvalidity, done)?;
        }
        if out.limit {
            break;
        }
    }
    Ok(out)
}

/// Apaga (pela retenção) o que está na lista. Cada uma confere que a pasta continua lida e
/// que o UIDVALIDITY é o mesmo de quando a mensagem foi baixada.
pub fn delete(
    c: &mut Conn,
    acc: &MailAccount,
    folders: &[FolderInfo],
    list: Vec<Candidate>,
) -> Vec<(Candidate, std::result::Result<String, String>)> {
    let gmail = providers::find(acc.provider.as_deref().unwrap_or("")).is_some_and(|p| p.gmail);
    let trash = trash_of(folders).cloned();
    let (has_move, has_uidplus) = (c.has("MOVE"), c.has("UIDPLUS"));
    let mut out = vec![];
    for cand in list {
        let res = (|| -> std::result::Result<String, String> {
            let f = folders
                .iter()
                .find(|f| f.name == cand.folder)
                .ok_or_else(|| i18n::t("nucleo-email-imap-pasta-sumiu"))?;
            if !reads(acc, f) {
                return Err(i18n::t("nucleo-email-imap-pasta-nao-lida"));
            }
            let uid = cand
                .uid
                .ok_or_else(|| i18n::t("nucleo-email-imap-sem-uid"))?;
            let mb = c.session.select(&f.name).map_err(|e| e.to_string())?;
            if mb.uid_validity != cand.uidvalidity {
                return Err(i18n::t("nucleo-email-imap-uidvalidity"));
            }
            let u = uid.to_string();
            match (&trash, has_move, has_uidplus) {
                (Some(t), true, _) => {
                    c.session.uid_mv(&u, &t.name).map_err(|e| e.to_string())?;
                }
                (Some(t), false, true) if !gmail => {
                    c.session.uid_copy(&u, &t.name).map_err(|e| e.to_string())?;
                    c.session
                        .uid_store(&u, "+FLAGS.SILENT (\\Deleted)")
                        .map_err(|e| e.to_string())?;
                    c.session.uid_expunge(&u).map_err(|e| e.to_string())?;
                }
                (None, _, true) if !gmail => {
                    c.session
                        .uid_store(&u, "+FLAGS.SILENT (\\Deleted)")
                        .map_err(|e| e.to_string())?;
                    c.session.uid_expunge(&u).map_err(|e| e.to_string())?;
                    return Ok(i18n::t("nucleo-email-imap-apagada"));
                }
                _ => {
                    return Err(i18n::t("nucleo-email-imap-nao-apaga"));
                }
            }
            let t = trash.as_ref().map(|t| t.name.as_str()).unwrap_or("");
            if gmail && cand.retention.purge_trash {
                purge_from_trash(c, t, cand.message_header_id.as_deref())?;
                return Ok(with("nucleo-email-imap-movida-apagada", "pasta", t));
            }
            Ok(with("nucleo-email-imap-movida", "pasta", t))
        })();
        out.push((cand, res));
    }
    out
}

/// Gmail com "apagar também da lixeira": acha a mensagem na lixeira pelo Message-ID.
fn purge_from_trash(
    c: &mut Conn,
    trash: &str,
    msgid: Option<&str>,
) -> std::result::Result<(), String> {
    let id = msgid.ok_or_else(|| i18n::t("nucleo-email-imap-lixeira-sem-id"))?;
    if id.contains(['"', '\\', '\r', '\n']) || id.len() > 250 {
        return Err(i18n::t("nucleo-email-imap-lixeira-id-estranho"));
    }
    c.session.select(trash).map_err(|e| e.to_string())?;
    let uids = c
        .session
        .uid_search(format!("HEADER Message-ID \"{id}\""))
        .map_err(|e| e.to_string())?;
    if uids.len() != 1 {
        return Err(i18n::t("nucleo-email-imap-lixeira-nao-achou"));
    }
    let u = uids.into_iter().next().unwrap_or_default().to_string();
    c.session
        .uid_store(&u, "+FLAGS.SILENT (\\Deleted)")
        .map_err(|e| e.to_string())?;
    c.session.uid_expunge(&u).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_and_caps() {
        assert_eq!(
            fingerprint(
                &"AB:"
                    .repeat(31)
                    .chars()
                    .chain("AB".chars())
                    .collect::<String>()
            )
            .unwrap()
            .len(),
            64
        );
        assert!(fingerprint("x").is_err());
        assert_eq!(
            imap_date(chrono::NaiveDate::from_ymd_opt(2026, 9, 19).unwrap()),
            "19-Sep-2026"
        );
    }
}
