//! Downloads da análise de ativos, atrás de um trait para que a coleta inteira seja testada
//! com fixtures, sem rede. Downloads condicionais (`If-None-Match`/`If-Modified-Since`): um
//! arquivo que não mudou desde a última coleta não é baixado nem processado de novo.

use std::time::Duration;

/// Metadados guardados em `research_file` depois de um download processado com sucesso.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FileMeta {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fetched {
    /// O servidor confirmou que o arquivo não mudou (304).
    NotModified,
    /// Arquivo inexistente (404): ano ainda sem dados, dia sem pregão.
    NotFound,
    Body {
        bytes: Vec<u8>,
        meta: FileMeta,
    },
}

pub type FetchResult = std::result::Result<Fetched, String>;

/// Quem baixa os arquivos. `user_agent` vazio = o padrão do app.
pub trait Fetcher: Send + Sync {
    fn get(&self, url: &str, cond: Option<&FileMeta>, user_agent: &str) -> FetchResult;
    /// Pausa depois de cada pedido (a SEC limita a 10 pedidos por segundo).
    fn polite_delay(&self, _url: &str) -> Duration {
        Duration::ZERO
    }
    /// GET com cabeçalhos extras (ex.: `Authorization` de uma API com token); o padrão ignora
    /// os cabeçalhos (fixtures dos testes).
    fn get_with(&self, url: &str, _headers: &[(&str, &str)], user_agent: &str) -> FetchResult {
        self.get(url, None, user_agent)
    }
    /// POST com corpo JSON (poucas APIs pedem; o padrão recusa).
    fn post(&self, _url: &str, _body: &str, _user_agent: &str) -> FetchResult {
        Err(crate::i18n::t("nucleo-pesquisa-erro-post"))
    }
}

/// Intervalo mínimo entre dois pedidos ao mesmo site (uso consciente: sem rajadas, e os
/// limites publicados de cada serviço com folga).
pub fn min_gap(url: &str) -> Duration {
    let host = host_of(url);
    let ms = match host.as_str() {
        h if h.ends_with("coingecko.com") => 6500, // API pública: ~10 pedidos por minuto
        h if h.ends_with("sec.gov") => 150,        // SEC: até 10 por segundo
        h if h.ends_with("olinda.bcb.gov.br") => 500,
        h if h.ends_with("api.bcb.gov.br") => 300,
        h if h.ends_with("b3.com.br") || h.ends_with("bmfbovespa.com.br") => 1500,
        h if h.ends_with("brapi.dev") => 1200,
        h if h.ends_with("fundamentus.com.br") => 1500,
        h if h.ends_with("finance.yahoo.com") => 500,
        h if h.ends_with("llama.fi") => 1000,
        h if h.ends_with("cvm.gov.br") => 300,
        _ => 250,
    };
    Duration::from_millis(ms)
}

/// Host de uma URL (minúsculo, sem porta).
pub fn host_of(url: &str) -> String {
    let rest = url.split("://").nth(1).unwrap_or(url);
    rest.split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Espera o intervalo mínimo desde o último pedido a este site (vale para o processo todo).
pub fn pace(url: &str) {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use std::time::Instant;
    static LAST: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    let host = host_of(url);
    let gap = min_gap(url);
    let wait = {
        let mut m = LAST
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        let next = m.get(&host).map(|t| *t + gap).unwrap_or(now);
        let start = next.max(now);
        m.insert(host, start);
        start - now
    };
    if !wait.is_zero() {
        std::thread::sleep(wait);
    }
}

/// Erro ao conectar. Recusa no aperto de mão do TLS ou certificado de emissor desconhecido
/// quase sempre é a rede no meio do caminho (filtro de segurança da empresa, antivírus, proxy
/// ou DNS que bloqueia o site), não o site nem o app: a mensagem diz isso, com o detalhe.
pub fn connect_error(url: &str, raw: &str) -> String {
    let l = raw.to_ascii_lowercase();
    let blocked = [
        "handshakefailure",
        "handshake failure",
        "unknownissuer",
        "unknown issuer",
        "invalidcertificate",
        "invalid peer certificate",
        "certificate",
    ]
    .iter()
    .any(|k| l.contains(k));
    if blocked {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-rede-bloqueada",
            &crate::i18n::args(&[
                ("site", host_of(url).into()),
                ("detalhe", raw.to_string().into()),
            ]),
        )
    } else {
        raw.to_string()
    }
}

/// Tentativas quando o site pede calma (429) ou está indisponível (503).
pub const MAX_ATTEMPTS: u32 = 3;
/// Espera máxima entre tentativas.
pub const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// Espera antes da tentativa `attempt` (1, 2…): o `Retry-After` do site, senão 2, 4, 8 s…
pub fn backoff(attempt: u32, retry_after: Option<&str>) -> Duration {
    let asked = retry_after
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_secs);
    asked
        .unwrap_or_else(|| Duration::from_secs(2u64.saturating_pow(attempt.min(6))))
        .min(MAX_BACKOFF)
}

/// Como o app se apresenta a todo site: nome, versão e onde achar o projeto (o contato que
/// serviços como o Open Food Facts pedem). Nunca se passa por navegador.
pub const DEFAULT_USER_AGENT: &str = concat!(
    "Treasury/",
    env!("CARGO_PKG_VERSION"),
    " (app local de financas pessoais; +https://github.com/knuppe/treasury)"
);

/// Limite de tamanho de um arquivo baixado (o maior arquivo da CVM tem ~35 MB).
const MAX_BYTES: u64 = 256 * 1024 * 1024;

pub struct HttpFetcher {
    agent: ureq::Agent,
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpFetcher {
    pub fn new() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(300)))
            .http_status_as_error(false)
            .build()
            .into();
        Self { agent }
    }
}

impl Fetcher for HttpFetcher {
    fn get(&self, url: &str, cond: Option<&FileMeta>, user_agent: &str) -> FetchResult {
        let ua = if user_agent.trim().is_empty() {
            DEFAULT_USER_AGENT
        } else {
            user_agent
        };
        let build = || {
            let mut req = self
                .agent
                .get(url)
                .header("User-Agent", ua)
                .header("Accept", "*/*");
            if url.contains("olinda.bcb.gov.br") {
                // IF.data: respostas de até ~30 MB, mas um pedido preso não segura a coleta.
                req = req
                    .config()
                    .timeout_global(Some(Duration::from_secs(120)))
                    .build();
            }
            if let Some(c) = cond {
                if let Some(e) = &c.etag {
                    req = req.header("If-None-Match", e);
                }
                if let Some(m) = &c.last_modified {
                    req = req.header("If-Modified-Since", m);
                }
            }
            req
        };
        let mut attempt = 0;
        let mut resp = loop {
            attempt += 1;
            pace(url);
            let resp = build()
                .call()
                .map_err(|e| connect_error(url, &e.to_string()))?;
            let status = resp.status().as_u16();
            if matches!(status, 429 | 503) && attempt < MAX_ATTEMPTS {
                let ra = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                std::thread::sleep(backoff(attempt, ra.as_deref()));
                continue;
            }
            break resp;
        };
        let status = resp.status().as_u16();
        match status {
            304 => return Ok(Fetched::NotModified),
            404 => return Ok(Fetched::NotFound),
            200..=299 => {}
            403 | 429 | 503 => {
                return Err(crate::i18n::t_args(
                    "nucleo-pesquisa-erro-servidor-recusou",
                    &crate::i18n::args(&[
                        ("status", status.to_string().into()),
                        ("tentativas", i64::from(attempt).into()),
                    ]),
                ));
            }
            _ => return Err(format!("HTTP {status}")),
        }
        let header = |name: &str| {
            resp.headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        let meta = FileMeta {
            etag: header("etag"),
            last_modified: header("last-modified"),
        };
        let bytes = resp
            .body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()
            .map_err(|e| e.to_string())?;
        Ok(Fetched::Body { bytes, meta })
    }

    fn get_with(&self, url: &str, headers: &[(&str, &str)], user_agent: &str) -> FetchResult {
        let ua = if user_agent.trim().is_empty() {
            DEFAULT_USER_AGENT
        } else {
            user_agent
        };
        let mut attempt = 0;
        let mut resp = loop {
            attempt += 1;
            pace(url);
            let mut req = self
                .agent
                .get(url)
                .header("User-Agent", ua)
                .header("Accept", "*/*");
            for (k, v) in headers {
                req = req.header(*k, *v);
            }
            let resp = req.call().map_err(|e| connect_error(url, &e.to_string()))?;
            let status = resp.status().as_u16();
            if matches!(status, 429 | 503) && attempt < MAX_ATTEMPTS {
                let ra = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                std::thread::sleep(backoff(attempt, ra.as_deref()));
                continue;
            }
            break resp;
        };
        match resp.status().as_u16() {
            404 => return Ok(Fetched::NotFound),
            200..=299 => {}
            s @ 401..=403 => {
                return Err(crate::i18n::t_args(
                    "nucleo-pesquisa-erro-token-recusado",
                    &crate::i18n::args(&[("status", s.to_string().into())]),
                ));
            }
            s => {
                return Err(crate::i18n::t_args(
                    "nucleo-pesquisa-erro-http-tentativas",
                    &crate::i18n::args(&[
                        ("status", s.to_string().into()),
                        ("tentativas", i64::from(attempt).into()),
                    ]),
                ));
            }
        }
        let bytes = resp
            .body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()
            .map_err(|e| e.to_string())?;
        Ok(Fetched::Body {
            bytes,
            meta: FileMeta::default(),
        })
    }

    fn post(&self, url: &str, body: &str, user_agent: &str) -> FetchResult {
        let ua = if user_agent.trim().is_empty() {
            DEFAULT_USER_AGENT
        } else {
            user_agent
        };
        pace(url);
        let mut resp = self
            .agent
            .post(url)
            .header("User-Agent", ua)
            .header("Content-Type", "application/json")
            .send(body)
            .map_err(|e| connect_error(url, &e.to_string()))?;
        match resp.status().as_u16() {
            404 => return Ok(Fetched::NotFound),
            200..=299 => {}
            s => return Err(format!("HTTP {s}")),
        }
        let bytes = resp
            .body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()
            .map_err(|e| e.to_string())?;
        Ok(Fetched::Body {
            bytes,
            meta: FileMeta::default(),
        })
    }

    fn polite_delay(&self, _url: &str) -> Duration {
        // O ritmo por site é aplicado antes de cada pedido ([`pace`]).
        Duration::ZERO
    }
}

#[cfg(test)]
mod pace_tests {
    use super::*;

    #[test]
    fn blocked_tls_is_explained() {
        let e = connect_error(
            "https://api.coingecko.com/api/v3/coins/bitcoin",
            "io: received fatal alert: HandshakeFailure",
        );
        assert!(
            e.contains("api.coingecko.com") && e.contains("bloqueou"),
            "{e}"
        );
        assert!(e.contains("HandshakeFailure"));
        assert_eq!(connect_error("https://x.com", "timeout"), "timeout");
    }

    #[test]
    fn hosts_gaps_and_backoff() {
        assert_eq!(
            host_of("https://data.sec.gov:443/api/x?y=1"),
            "data.sec.gov"
        );
        assert_eq!(
            min_gap("https://api.coingecko.com/api/v3/coins"),
            Duration::from_millis(6500)
        );
        assert_eq!(
            min_gap("https://sistemaswebb3-listados.b3.com.br/listedCompaniesProxy/x"),
            Duration::from_millis(1500)
        );
        assert_eq!(backoff(1, None), Duration::from_secs(2));
        assert_eq!(backoff(3, None), Duration::from_secs(8));
        assert_eq!(backoff(1, Some("30")), Duration::from_secs(30));
        assert_eq!(backoff(1, Some("600")), MAX_BACKOFF);
        // Dois pedidos seguidos ao mesmo site respeitam o intervalo.
        let t = std::time::Instant::now();
        pace("https://exemplo.invalid/a");
        pace("https://exemplo.invalid/b");
        assert!(t.elapsed() >= Duration::from_millis(240));
    }
}
