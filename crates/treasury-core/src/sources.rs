//! Fontes externas de dados (cotações, índices, fundamentos, tabelas oficiais) que o app
//! consulta pela rede, cada uma com liga/desliga em Ajustes › Fontes de dados. Desligada,
//! a fonte não é consultada em nenhuma coleta, rotina ou comando; o que já foi baixado fica.

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};

/// Ajuste com os ids das fontes desligadas.
pub const DISABLED_SETTING: &str = "sources.disabled";
/// Ajuste técnico: saúde de cada fonte (dias seguidos com falha, último erro).
pub const HEALTH_SETTING: &str = "sources.health";
/// Dias seguidos com falha (site fora do ar, layout novo, dados inválidos) até a fonte ser
/// desligada sozinha, com aviso na caixa de alertas.
pub const FAIL_DAYS_TO_DISABLE: u32 = 3;

/// Como a fonte vem respondendo às coletas.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SourceHealth {
    /// Dias seguidos com falha (conta no máximo uma por dia; um sucesso zera).
    #[ts(type = "number")]
    pub failing_days: u32,
    pub last_fail_day: Option<String>,
    pub last_error: Option<String>,
    pub last_ok: Option<String>,
    /// Desligada pelo app depois das falhas (não pelo usuário).
    pub auto_disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SourceId {
    Yahoo,
    Tesouro,
    Cvm,
    B3,
    Sec,
    Bcb,
    Fred,
    Ecb,
    Eurostat,
    Oecd,
    Boc,
    Ons,
    Boe,
    WorldBank,
    Fipe,
    Fundamentus,
    Coingecko,
    Defillama,
    /// Coin Metrics (API Community, sem chave): endereços ativos das redes.
    CoinMetrics,
    /// mempool.space: blocos por pool de mineração do bitcoin.
    Mempool,
    Brapi,
    /// Tiingo, com a chave gratuita da pessoa: cotações dos Estados Unidos.
    Tiingo,
    /// EODHD, com a chave da pessoa: cotações de bolsas do mundo todo.
    Eodhd,
    /// IBGE (SIDRA): IPCA por grupo, para a inflação pessoal.
    Ibge,
    /// NY Fed: Fed Funds efetiva (EFFR), sem chave.
    NyFed,
    /// SEFAZ: consulta pública da NFC-e pelo link do QR do cupom.
    Sefaz,
    /// Tesouro Nacional (Siconfi): a arrecadação do município da família, pelo relatório que a
    /// prefeitura entrega (só o código do município vai).
    Siconfi,
    /// Autoconfiguração de e-mail: o servidor IMAP de um domínio fora do catálogo.
    MailAutoconfig,
    /// GLEIF: LEI (identificador global de entidade legal) do emissor, pelo CNPJ.
    Gleif,
    /// BrasilAPI: espelho público do cadastro de CNPJ da Receita Federal (só o CNPJ vai).
    #[serde(rename = "brasilapi")]
    BrasilApi,
    /// Open Food Facts: nome, marca, nutrientes, grupo NOVA e alérgenos pelo código de barras
    /// (só o GTIN vai). Desligada por padrão (docs/gtin-inflation.md §7).
    OpenFoodFacts,
    /// Feeds RSS de portais de economia (notícias do ativo): o mesmo pedido para todos; o
    /// filtro pelo ticker é feito no computador (docs/asset-overview.md).
    NewsFeeds,
}

/// Fontes desligadas até a pessoa ligar (as outras nascem ligadas).
pub const OPT_IN: &[SourceId] = &[SourceId::OpenFoodFacts];
/// Ajuste com os ids das fontes [`OPT_IN`] que a pessoa ligou.
pub const OPTED_IN_SETTING: &str = "sources.opted_in";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SourceInfo {
    pub id: SourceId,
    pub name: String,
    pub homepage: String,
    /// Para que o app usa a fonte, em uma frase.
    pub used_for: String,
    /// Termos, licença e atribuição que a fonte pede, no idioma da instalação (os avisos
    /// exigidos palavra por palavra ficam em inglês).
    pub terms: String,
    /// País dos dados (ISO 3166, `EU` para a zona do euro, `INT` para fontes globais).
    pub country: String,
    pub enabled: bool,
    /// Falhas recentes; `None` quando a última coleta deu certo (ou nunca houve).
    pub health: Option<SourceHealth>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct SourceToggleArgs {
    pub id: SourceId,
    pub enabled: bool,
}

impl SourceId {
    pub const ALL: [SourceId; 32] = [
        SourceId::Yahoo,
        SourceId::Fundamentus,
        SourceId::Tesouro,
        SourceId::Cvm,
        SourceId::B3,
        SourceId::Bcb,
        SourceId::Sec,
        SourceId::Fred,
        SourceId::NyFed,
        SourceId::Ecb,
        SourceId::Eurostat,
        SourceId::Oecd,
        SourceId::Boc,
        SourceId::Ons,
        SourceId::Boe,
        SourceId::WorldBank,
        SourceId::Fipe,
        SourceId::Coingecko,
        SourceId::Defillama,
        SourceId::CoinMetrics,
        SourceId::Mempool,
        SourceId::Brapi,
        SourceId::Tiingo,
        SourceId::Eodhd,
        SourceId::Ibge,
        SourceId::Sefaz,
        SourceId::Siconfi,
        SourceId::MailAutoconfig,
        SourceId::Gleif,
        SourceId::BrasilApi,
        SourceId::OpenFoodFacts,
        SourceId::NewsFeeds,
    ];

    pub fn key(self) -> &'static str {
        match self {
            SourceId::Yahoo => "yahoo",
            SourceId::Tesouro => "tesouro",
            SourceId::Cvm => "cvm",
            SourceId::B3 => "b3",
            SourceId::Sec => "sec",
            SourceId::Bcb => "bcb",
            SourceId::Fred => "fred",
            SourceId::Ecb => "ecb",
            SourceId::Eurostat => "eurostat",
            SourceId::Oecd => "oecd",
            SourceId::Boc => "boc",
            SourceId::Ons => "ons",
            SourceId::Boe => "boe",
            SourceId::WorldBank => "world_bank",
            SourceId::Fipe => "fipe",
            SourceId::Fundamentus => "fundamentus",
            SourceId::Coingecko => "coingecko",
            SourceId::Defillama => "defillama",
            SourceId::CoinMetrics => "coin_metrics",
            SourceId::Mempool => "mempool",
            SourceId::Brapi => "brapi",
            SourceId::Tiingo => "tiingo",
            SourceId::Eodhd => "eodhd",
            SourceId::Ibge => "ibge",
            SourceId::NyFed => "ny_fed",
            SourceId::Sefaz => "sefaz",
            SourceId::Siconfi => "siconfi",
            SourceId::MailAutoconfig => "mail_autoconfig",
            SourceId::Gleif => "gleif",
            SourceId::BrasilApi => "brasilapi",
            SourceId::OpenFoodFacts => "open_food_facts",
            SourceId::NewsFeeds => "news_feeds",
        }
    }

    /// Site da fonte.
    pub fn homepage(self) -> &'static str {
        match self {
            SourceId::Yahoo => "https://finance.yahoo.com",
            SourceId::Fundamentus => "https://fundamentus.com.br",
            SourceId::Tesouro => "https://www.tesourotransparente.gov.br",
            SourceId::Cvm => "https://dados.cvm.gov.br",
            SourceId::B3 => "https://www.b3.com.br",
            SourceId::Bcb => "https://dadosabertos.bcb.gov.br",
            SourceId::Sec => "https://www.sec.gov/edgar",
            SourceId::Fred => "https://fred.stlouisfed.org",
            SourceId::Ecb => "https://data.ecb.europa.eu",
            SourceId::Eurostat => "https://ec.europa.eu/eurostat",
            SourceId::Oecd => "https://data.oecd.org",
            SourceId::Boc => "https://www.bankofcanada.ca",
            SourceId::Ons => "https://www.ons.gov.uk",
            SourceId::Boe => "https://www.bankofengland.co.uk",
            SourceId::WorldBank => "https://data.worldbank.org",
            SourceId::Fipe => "https://veiculos.fipe.org.br",
            SourceId::Coingecko => "https://www.coingecko.com",
            SourceId::Defillama => "https://defillama.com",
            SourceId::CoinMetrics => "https://coinmetrics.io/community-network-data/",
            SourceId::Mempool => "https://mempool.space",
            SourceId::Brapi => "https://brapi.dev",
            SourceId::Tiingo => "https://www.tiingo.com",
            SourceId::Eodhd => "https://eodhd.com",
            SourceId::Ibge => "https://sidra.ibge.gov.br",
            SourceId::Sefaz => "https://www.nfe.fazenda.gov.br",
            SourceId::Siconfi => "https://siconfi.tesouro.gov.br",
            SourceId::MailAutoconfig => "https://autoconfig.thunderbird.net",
            SourceId::NyFed => "https://markets.newyorkfed.org",
            SourceId::Gleif => "https://www.gleif.org",
            SourceId::BrasilApi => "https://brasilapi.com.br",
            SourceId::OpenFoodFacts => "https://world.openfoodfacts.org",
            SourceId::NewsFeeds => "https://www.infomoney.com.br/feed/",
        }
    }

    /// Nome da fonte no idioma da instalação (`nucleo-fontes-<id>-nome`; nomes próprios ficam
    /// iguais em todos os idiomas).
    pub fn name(self) -> String {
        crate::i18n::t(&self.name_key())
    }

    /// Chave do nome (para os alertas, que montam o texto no idioma de quem lê).
    pub fn name_key(self) -> String {
        format!("nucleo-fontes-{}-nome", self.key())
    }

    /// Para que o app usa a fonte, em uma frase, no idioma da instalação.
    pub fn used_for(self) -> String {
        crate::i18n::t(&format!("nucleo-fontes-{}-uso", self.key()))
    }

    /// Termos, licença e atribuição da fonte. Toda fonte tem este texto (teste
    /// `names_and_uses_follow_the_locale`): fonte nova só entra com os termos lidos.
    pub fn terms(self) -> String {
        crate::i18n::t(&self.terms_key())
    }

    pub fn terms_key(self) -> String {
        format!("nucleo-fontes-{}-termos", self.key())
    }

    /// País dos dados que a fonte entrega (a bandeira em Ajustes › Fontes de dados).
    pub fn country(self) -> &'static str {
        match self {
            SourceId::Fundamentus
            | SourceId::Tesouro
            | SourceId::Cvm
            | SourceId::B3
            | SourceId::Bcb
            | SourceId::Fipe
            | SourceId::Brapi
            | SourceId::Ibge
            | SourceId::Sefaz
            | SourceId::Siconfi
            | SourceId::BrasilApi
            | SourceId::NewsFeeds => "BR",
            SourceId::Sec | SourceId::Fred | SourceId::NyFed | SourceId::Tiingo => "US",
            SourceId::Ecb | SourceId::Eurostat => "EU",
            SourceId::Boc => "CA",
            SourceId::Ons | SourceId::Boe => "GB",
            SourceId::Yahoo
            | SourceId::Eodhd
            | SourceId::Oecd
            | SourceId::WorldBank
            | SourceId::Coingecko
            | SourceId::Defillama
            | SourceId::CoinMetrics
            | SourceId::Mempool
            | SourceId::MailAutoconfig
            | SourceId::Gleif
            | SourceId::OpenFoodFacts => "INT",
        }
    }
}

fn disabled(conn: &Connection) -> Result<Vec<String>> {
    crate::settings::get_or(conn, DISABLED_SETTING, Vec::<String>::new())
}

fn opted_in(conn: &Connection) -> Result<Vec<String>> {
    crate::settings::get_or(conn, OPTED_IN_SETTING, Vec::<String>::new())
}

/// A fonte pode ser consultada: ligada por padrão, salvo as [`OPT_IN`], que só valem depois
/// que a pessoa liga. Erro de leitura conta como ligada (e como desligada, nas `OPT_IN`).
pub fn enabled(conn: &Connection, id: SourceId) -> bool {
    if OPT_IN.contains(&id) {
        return opted_in(conn).is_ok_and(|o| o.iter().any(|k| k == id.key()))
            && disabled(conn).map_or(true, |d| !d.iter().any(|k| k == id.key()));
    }
    disabled(conn).map_or(true, |d| !d.iter().any(|k| k == id.key()))
}

/// Grava a escolha de uma fonte [`OPT_IN`] (ligada só com a pessoa).
fn set_opt_in(conn: &Connection, id: SourceId, on: bool) -> Result<()> {
    let mut o = opted_in(conn)?;
    o.retain(|k| k != id.key());
    if on {
        o.push(id.key().to_string());
    }
    o.sort();
    crate::settings::set(conn, OPTED_IN_SETTING, &o)
}

/// Erro claro quando a fonte está desligada.
pub fn ensure_enabled(conn: &Connection, id: SourceId) -> Result<()> {
    if enabled(conn, id) {
        Ok(())
    } else {
        Err(disabled_error(id))
    }
}

/// "A fonte … está desligada em Ajustes › Fontes de dados", no idioma da instalação.
pub fn disabled_error(id: SourceId) -> Error {
    Error::invalid(crate::i18n::t_args(
        "nucleo-fontes-erro-desligada",
        &crate::i18n::args(&[("fonte", id.name().into())]),
    ))
}

/// Falha de rede ou do site (conexão reiniciada ou recusada, tempo esgotado, DNS, limite de
/// pedidos, servidor fora do ar): passageira e sem risco de gravar dado errado. Não desliga a
/// fonte; com dias seguidos, vira feedback para investigar.
pub fn is_transient(error: &str) -> bool {
    let e = error.to_lowercase();
    const NETWORK: &[&str] = &[
        "connection reset",
        "connection refused",
        "connection aborted",
        "connection closed",
        "broken pipe",
        "timed out",
        "timeout",
        "os error 54",
        "os error 60",
        "os error 61",
        "os error 104",
        "os error 110",
        "os error 111",
        "os error 10053",
        "os error 10054",
        "os error 10060",
        "dns",
        "failed to lookup",
        "name or service not known",
        "nodename nor servname",
        "network is unreachable",
        "host is unreachable",
        "unexpected eof",
        "tls",
    ];
    if NETWORK.iter().any(|k| e.contains(k)) {
        return true;
    }
    // Status HTTP de limite de pedidos e de servidor indisponível, como aparece nas mensagens.
    ["429", "500", "502", "503", "504"].iter().any(|code| {
        e.match_indices(code).any(|(i, _)| {
            let before = e[..i].chars().next_back();
            let after = e[i + code.len()..].chars().next();
            !before.is_some_and(|c| c.is_ascii_digit())
                && !after.is_some_and(|c| c.is_ascii_digit())
        })
    })
}

/// Título do feedback de uma fonte com falhas, no idioma `l`.
fn feedback_title_in(l: crate::i18n::Locale, id: SourceId) -> String {
    crate::i18n::tr(
        l,
        "nucleo-fontes-feedback-titulo",
        Some(&crate::i18n::args(&[(
            "fonte",
            crate::i18n::tr(l, &id.name_key(), None).into(),
        )])),
    )
}

/// O título em todos os idiomas (o feedback pode ter sido aberto em outro).
fn feedback_titles(id: SourceId) -> Vec<String> {
    let mut v: Vec<String> = crate::i18n::AVAILABLE
        .iter()
        .map(|l| feedback_title_in(*l, id))
        .collect();
    v.sort();
    v.dedup();
    v
}

const FEEDBACK_CLIENT: &str = "fontes de dados";

/// Falhas seguidas viram feedback para investigar: um por fonte, reforçado a cada novo dia
/// de falha, e fechado sozinho quando a coleta volta a funcionar.
fn feedback_failure(conn: &Connection, id: SourceId, h: &SourceHealth, disabled_now: bool) {
    let open: Option<String> = feedback_titles(id).iter().find_map(|t| {
        conn.query_row(
            "SELECT id FROM dev_feedback WHERE title = ?1 AND created_by = ?2
               AND status IN ('aberto', 'em_andamento')",
            rusqlite::params![t, FEEDBACK_CLIENT],
            |r| r.get(0),
        )
        .ok()
    });
    let error = h.last_error.clone().unwrap_or_default();
    let note = crate::i18n::t_args(
        "nucleo-fontes-feedback-nota",
        &crate::i18n::args(&[
            ("dias", i64::from(h.failing_days).into()),
            ("erro", error.clone().into()),
        ]),
    );
    if let Some(fid) = open {
        let _ = crate::devfeedback::bump(
            conn,
            &crate::devfeedback::DevFeedbackBump {
                id: fid,
                note: Some(note),
            },
            FEEDBACK_CLIENT,
            false,
        );
        return;
    }
    let key = if disabled_now {
        "nucleo-fontes-feedback-corpo-desligada"
    } else {
        "nucleo-fontes-feedback-corpo-passageira"
    };
    let body = crate::i18n::t_args(
        key,
        &crate::i18n::args(&[
            ("fonte", id.name().into()),
            ("dias", i64::from(h.failing_days).into()),
            ("erro", error.into()),
            ("site", id.homepage().to_string().into()),
        ]),
    );
    let _ = crate::devfeedback::add_distinct(
        conn,
        &crate::devfeedback::DevFeedbackInput {
            title: feedback_title_in(crate::i18n::current(), id),
            body,
            area: crate::devfeedback::DevFeedbackArea::Dados,
            kind: crate::devfeedback::DevFeedbackKind::Bug,
            ..Default::default()
        },
        FEEDBACK_CLIENT,
    );
}

/// A coleta voltou: fecha o feedback aberto da fonte, com a data.
fn feedback_recovered(conn: &Connection, id: SourceId, day: &str) {
    let why = crate::i18n::t_args(
        "nucleo-fontes-feedback-resolvido",
        &crate::i18n::args(&[("data", day.to_string().into())]),
    );
    for title in feedback_titles(id) {
        let _ = crate::devfeedback::resolve_auto(conn, &title, FEEDBACK_CLIENT, &why);
    }
}

fn health_map(conn: &Connection) -> Result<HashMap<String, SourceHealth>> {
    crate::settings::get_or(conn, HEALTH_SETTING, HashMap::new())
}

/// Registra o resultado de uma coleta da fonte. Falha conta um dia (várias no mesmo dia
/// contam uma vez). Com [`FAIL_DAYS_TO_DISABLE`] dias seguidos, vira feedback para investigar
/// (reforçado a cada novo dia) e, se a falha não for passageira ([`is_transient`]: rede, limite
/// de pedidos, site fora do ar), a fonte é desligada com um alerta explicando o motivo.
/// Sucesso zera e fecha o feedback. Devolve `true` se a fonte foi desligada agora.
pub fn record_result(
    conn: &Connection,
    id: SourceId,
    result: &std::result::Result<(), String>,
    today: NaiveDate,
) -> Result<bool> {
    let mut all = health_map(conn)?;
    let day = crate::time::fmt_date(today);
    let mut h = all.remove(id.key()).unwrap_or_default();
    let mut disabled_now = false;
    match result {
        Ok(()) => {
            if h.failing_days >= FAIL_DAYS_TO_DISABLE || h.auto_disabled {
                feedback_recovered(conn, id, &day);
            }
            h = SourceHealth {
                last_ok: Some(day),
                ..SourceHealth::default()
            };
        }
        Err(e) => {
            let new_day = h.last_fail_day.as_deref() != Some(day.as_str());
            if new_day {
                h.failing_days += 1;
                h.last_fail_day = Some(day.clone());
            }
            h.last_error = Some(e.chars().take(300).collect());
            if h.failing_days >= FAIL_DAYS_TO_DISABLE && enabled(conn, id) && !is_transient(e) {
                let mut off = disabled(conn)?;
                off.push(id.key().to_string());
                off.sort();
                crate::settings::set(conn, DISABLED_SETTING, &off)?;
                h.auto_disabled = true;
                disabled_now = true;
                crate::ai::alerts::create(
                    conn,
                    &crate::ai::alerts::NewAlert {
                        kind: "source_disabled",
                        severity: crate::ai::alerts::AlertSeverity::Warning,
                        // O nome da fonte vai como chave: sai no idioma de quem lê.
                        title: crate::ai::alerts::Msg::new("alertas-fonte-desligada-titulo")
                            .arg("fonte", crate::ai::alerts::Msg::new(&id.name_key())),
                        body: vec![
                            crate::ai::alerts::Msg::new("alertas-fonte-desligada-corpo")
                                .arg("fonte", crate::ai::alerts::Msg::new(&id.name_key()))
                                .arg("dias", i64::from(h.failing_days))
                                .arg(
                                    "erro",
                                    match h.last_error.as_deref() {
                                        Some(e) => crate::ai::alerts::Arg::from(e),
                                        None => {
                                            crate::ai::alerts::Msg::new("alertas-erro-desconhecido")
                                                .into()
                                        }
                                    },
                                )
                                .into(),
                        ],
                        source: "sources",
                        dedupe_key: Some(format!("source_disabled:{}:{day}", id.key())),
                        ref_table: None,
                        ref_id: None,
                        data: serde_json::json!({"source": id.key(), "failing_days": h.failing_days}),
                    },
                )?;
            }
            if new_day && h.failing_days >= FAIL_DAYS_TO_DISABLE {
                feedback_failure(conn, id, &h, disabled_now);
            }
        }
    }
    all.insert(id.key().to_string(), h);
    crate::settings::set(conn, HEALTH_SETTING, &all)?;
    Ok(disabled_now)
}

pub fn list(conn: &Connection) -> Result<Vec<SourceInfo>> {
    let off = disabled(conn)?;
    let health = health_map(conn)?;
    Ok(SourceId::ALL
        .into_iter()
        .map(|id| SourceInfo {
            id,
            name: id.name(),
            homepage: id.homepage().into(),
            used_for: id.used_for(),
            terms: id.terms(),
            country: id.country().into(),
            enabled: enabled(conn, id) && !off.iter().any(|k| k == id.key()),
            health: health
                .get(id.key())
                .filter(|h| h.failing_days > 0 || h.auto_disabled)
                .cloned(),
        })
        .collect())
}

/// Liga ou desliga uma fonte (sem mexer na saúde dela).
pub fn set_enabled(conn: &Connection, id: SourceId, on: bool) -> Result<()> {
    if OPT_IN.contains(&id) {
        set_opt_in(conn, id, on)?;
    }
    let mut off = disabled(conn)?;
    off.retain(|k| k != id.key());
    if !on {
        off.push(id.key().to_string());
    }
    off.sort();
    crate::settings::set(conn, DISABLED_SETTING, &off)
}

pub fn set(conn: &Connection, a: &SourceToggleArgs) -> Result<Vec<SourceInfo>> {
    if OPT_IN.contains(&a.id) {
        set_opt_in(conn, a.id, a.enabled)?;
    }
    let mut off = disabled(conn)?;
    off.retain(|k| k != a.id.key());
    if !a.enabled {
        off.push(a.id.key().to_string());
    }
    off.sort();
    crate::settings::set(conn, DISABLED_SETTING, &off)?;
    // Ligada de novo pelo usuário: as falhas antigas não contam mais.
    if a.enabled {
        let mut all = health_map(conn)?;
        if all.remove(a.id.key()).is_some() {
            crate::settings::set(conn, HEALTH_SETTING, &all)?;
        }
    }
    list(conn)
}

pub const READ_ONLY: &[&str] = &["list"];

/// `sources.list` e `sources.set` (só o usuário liga e desliga fontes).
pub fn handle(
    ctx: &mut crate::api::Ctx<'_>,
    action: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    use crate::api::{parse_args, to_value};
    match action {
        "list" => to_value(list(ctx.conn())?),
        "set" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(crate::i18n::t(
                    "nucleo-fontes-erro-so-interface",
                )));
            }
            to_value(set(ctx.conn(), &parse_args::<SourceToggleArgs>(args)?)?)
        }
        _ => Err(Error::UnknownCommand(format!("sources.{action}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn toggles_are_remembered_and_checked() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        // Todas nascem ligadas, menos as que dependem da pessoa (Open Food Facts).
        assert!(
            list(c)
                .unwrap()
                .iter()
                .all(|s| s.enabled != OPT_IN.contains(&s.id))
        );
        set(
            c,
            &SourceToggleArgs {
                id: SourceId::OpenFoodFacts,
                enabled: true,
            },
        )
        .unwrap();
        assert!(enabled(c, SourceId::OpenFoodFacts));
        let all = list(c).unwrap();
        let country = |id| all.iter().find(|s| s.id == id).unwrap().country.as_str();
        assert_eq!(country(SourceId::B3), "BR");
        assert_eq!(country(SourceId::Sec), "US");
        assert_eq!(country(SourceId::Yahoo), "INT");
        set(
            c,
            &SourceToggleArgs {
                id: SourceId::Fundamentus,
                enabled: false,
            },
        )
        .unwrap();
        assert!(!enabled(c, SourceId::Fundamentus));
        assert!(enabled(c, SourceId::Yahoo));
        let e = ensure_enabled(c, SourceId::Fundamentus).unwrap_err();
        assert!(e.to_string().contains("Fundamentus"), "{e}");
        set(
            c,
            &SourceToggleArgs {
                id: SourceId::Fundamentus,
                enabled: true,
            },
        )
        .unwrap();
        assert!(enabled(c, SourceId::Fundamentus));
    }

    #[test]
    fn names_and_uses_follow_the_locale() {
        use crate::i18n::{Locale, has_key, set_current};
        for id in SourceId::ALL {
            for l in [Locale::PtBr, Locale::En, Locale::Es] {
                assert!(has_key(l, &id.name_key()), "{l:?}: {}", id.name_key());
                assert!(
                    has_key(l, &format!("nucleo-fontes-{}-uso", id.key())),
                    "{l:?}: {}",
                    id.key()
                );
                assert!(has_key(l, &id.terms_key()), "{l:?}: {}", id.terms_key());
            }
        }
        // Os avisos que a fonte exige palavra por palavra não mudam com o idioma.
        assert!(SourceId::Fred.terms().contains("not endorsed or certified"));
        assert!(
            SourceId::Boe
                .terms()
                .contains("SONIA and/or SONIA Compounded Index")
        );
        assert!(
            SourceId::NyFed
                .terms()
                .contains("Terms of Use posted at newyorkfed.org")
        );
        assert!(
            SourceId::Ons
                .terms()
                .contains("Open Government Licence v3.0")
        );
        assert_eq!(SourceId::WorldBank.name(), "Banco Mundial");
        set_current(Locale::En);
        assert_eq!(SourceId::WorldBank.name(), "World Bank");
        assert_eq!(SourceId::Yahoo.name(), "Yahoo Finance");
        assert!(SourceId::Ecb.used_for().contains("Euro area"));
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        set_enabled(db.conn(), SourceId::Oecd, false).unwrap();
        let e = ensure_enabled(db.conn(), SourceId::Oecd).unwrap_err();
        assert_eq!(
            e.message(),
            "the OECD source is turned off in Settings › Data sources"
        );
        set_current(Locale::PtBr);
    }

    #[test]
    fn network_failures_keep_the_source_on_and_become_a_feedback() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        let day = |d: u32| NaiveDate::from_ymd_opt(2026, 10, d).unwrap();
        let err: std::result::Result<(), String> =
            Err("io: Connection reset by peer (os error 54)".into());
        for d in 1..=3 {
            assert!(!record_result(c, SourceId::Tesouro, &err, day(d)).unwrap());
        }
        // Continua ligada, sem o alerta de fonte desligada, e com um feedback para investigar.
        assert!(enabled(c, SourceId::Tesouro));
        let count = |sql: &str| -> i64 { c.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(
            count("SELECT count(*) FROM alert WHERE kind = 'source_disabled'"),
            0
        );
        assert_eq!(
            count("SELECT count(*) FROM dev_feedback WHERE status = 'aberto'"),
            1
        );
        // Um novo dia de falha reforça o mesmo; o mesmo dia de novo, não.
        record_result(c, SourceId::Tesouro, &err, day(4)).unwrap();
        record_result(c, SourceId::Tesouro, &err, day(4)).unwrap();
        assert_eq!(count("SELECT count(*) FROM dev_feedback"), 1);
        assert_eq!(count("SELECT bumps FROM dev_feedback"), 1);
        // A coleta voltou: o feedback fecha sozinho.
        record_result(c, SourceId::Tesouro, &Ok(()), day(5)).unwrap();
        assert_eq!(
            count("SELECT count(*) FROM dev_feedback WHERE status = 'resolvido'"),
            1
        );
        // Erro de dado (layout novo) desliga e também vira feedback.
        let bad: std::result::Result<(), String> = Err("layout novo: coluna PU sumiu".into());
        for d in 6..=8 {
            record_result(c, SourceId::Tesouro, &bad, day(d)).unwrap();
        }
        assert!(!enabled(c, SourceId::Tesouro));
        assert_eq!(
            count("SELECT count(*) FROM dev_feedback WHERE status = 'aberto'"),
            1
        );
    }

    #[test]
    fn transient_errors_are_network_and_overload() {
        for e in [
            "io: Connection reset by peer (os error 54)",
            "timed out reading response",
            "Erro HTTP 429 no Tesouro Transparente",
            "status 503",
            "failed to lookup address information",
        ] {
            assert!(is_transient(e), "{e}");
        }
        for e in [
            "layout novo: coluna PU sumiu",
            "CSV sem a coluna Data Base",
            "4290 linhas",
        ] {
            assert!(!is_transient(e), "{e}");
        }
    }

    #[test]
    fn three_failing_days_disable_with_alert_and_success_resets() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        let c = db.conn();
        let day = |d: u32| NaiveDate::from_ymd_opt(2026, 10, d).unwrap();
        let err: std::result::Result<(), String> = Err("layout novo: coluna PU sumiu".into());
        // Várias falhas no mesmo dia contam uma; um sucesso zera.
        record_result(c, SourceId::Tesouro, &err, day(1)).unwrap();
        record_result(c, SourceId::Tesouro, &err, day(1)).unwrap();
        record_result(c, SourceId::Tesouro, &Ok(()), day(2)).unwrap();
        assert!(list(c).unwrap().iter().all(|s| s.health.is_none()));
        assert!(!record_result(c, SourceId::Tesouro, &err, day(3)).unwrap());
        assert!(!record_result(c, SourceId::Tesouro, &err, day(4)).unwrap());
        let info = list(c).unwrap();
        let t = info.iter().find(|s| s.id == SourceId::Tesouro).unwrap();
        assert_eq!(t.health.as_ref().unwrap().failing_days, 2);
        assert!(t.enabled);
        assert!(record_result(c, SourceId::Tesouro, &err, day(5)).unwrap());
        assert!(!enabled(c, SourceId::Tesouro));
        let alerts: i64 = c
            .query_row(
                "SELECT count(*) FROM alert WHERE kind = 'source_disabled'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(alerts, 1);
        let t = list(c)
            .unwrap()
            .into_iter()
            .find(|s| s.id == SourceId::Tesouro)
            .unwrap();
        assert!(t.health.unwrap().auto_disabled);
        // Ligada de novo: começa do zero.
        set(
            c,
            &SourceToggleArgs {
                id: SourceId::Tesouro,
                enabled: true,
            },
        )
        .unwrap();
        let t = list(c)
            .unwrap()
            .into_iter()
            .find(|s| s.id == SourceId::Tesouro)
            .unwrap();
        assert!(t.enabled && t.health.is_none());
    }
}
