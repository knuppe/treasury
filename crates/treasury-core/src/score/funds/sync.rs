//! Coleta do score de fundos e ETFs, dentro da coleta da análise de ativos: nome longo dos
//! ETFs da B3 no Yahoo (para achar o CNPJ), registro de fundos e classes e extrato da CVM
//! (só os fundos e ETFs cadastrados) e carteiras teóricas dos índices da B3 replicados pelos
//! ETFs cadastrados. Planeja lendo o banco, baixa sem travá-lo e grava um arquivo por vez.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use chrono::NaiveDate;

use super::{b3_index, etf_config, index_rule, registry};
use crate::assets::AssetClass;
use crate::error::Result;
use crate::research::sync::{Req, Runner, SyncSource};
use crate::sources::{SourceId, enabled};

fn older_than(at: Option<&str>, hours: i64) -> bool {
    at.and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .is_none_or(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_hours() >= hours)
}

struct Plan {
    wanted: HashSet<String>,
    /// ETFs da B3 sem CNPJ: id do ativo → (ticker, nomes candidatos já normalizados).
    unmapped: BTreeMap<String, (String, Vec<String>)>,
    /// Fundos e previdência sem CNPJ: id do ativo → nome com `registry::fund_key`.
    unnamed: BTreeMap<String, String>,
    registry_due: bool,
    portfolios_due: BTreeSet<String>,
    cvm: bool,
    b3: bool,
    yahoo: bool,
}

fn plan(conn: &rusqlite::Connection, force: bool) -> Result<Plan> {
    let reg = registry::load(conn)?;
    let ports = b3_index::load(conn)?;
    let cfg = etf_config();
    let mut wanted = HashSet::new();
    let mut unmapped = BTreeMap::new();
    let mut unnamed = BTreeMap::new();
    let mut indices = BTreeSet::new();
    for a in crate::assets::list(conn, &Default::default())? {
        match a.class {
            // Fundo de pensão (EFPC) não está na CVM: nada a procurar lá.
            AssetClass::Pension if crate::pension::is_efpc(conn, &a.id)? => {}
            AssetClass::Fund | AssetClass::Pension => {
                match registry::cnpj_of(&reg, &a.id, a.cnpj.as_deref()) {
                    Some(c) => {
                        // CNPJ fora do registro da CVM (depois de baixado): em previdência, é
                        // quase sempre o do plano, não o do FIE onde ele aplica. Procura o
                        // fundo pelo nome, como sugestão.
                        if reg.registro_at.is_some() && !reg.funds.contains_key(&c) {
                            unnamed.insert(a.id.clone(), registry::fund_key(&a.name));
                        }
                        wanted.insert(c);
                    }
                    None => {
                        unnamed.insert(a.id.clone(), registry::fund_key(&a.name));
                    }
                }
            }
            AssetClass::EtfBr => {
                let cnpj = registry::cnpj_of(&reg, &a.id, a.cnpj.as_deref());
                let mut names: Vec<&str> = vec![a.name.as_str()];
                if let Some(i) = cnpj.as_ref().and_then(|c| reg.funds.get(c)) {
                    names.push(i.name.as_str());
                }
                if let Some(n) = reg.etf_names.get(&a.id) {
                    names.push(n.as_str());
                }
                if let Some(i) =
                    index_rule(cfg, a.ticker.as_deref(), &names).and_then(|r| r.b3_index.clone())
                {
                    indices.insert(i);
                }
                match cnpj {
                    Some(c) => {
                        wanted.insert(c);
                    }
                    None => {
                        let keys = names.iter().map(|n| registry::name_key(n)).collect();
                        unmapped.insert(a.id.clone(), (a.ticker.clone().unwrap_or_default(), keys));
                    }
                }
            }
            _ => {}
        }
    }
    // Fundo sem CNPJ só força o registro enquanto não tem sugestão.
    let missing = wanted.iter().any(|c| !reg.funds.contains_key(c))
        || !unmapped.is_empty()
        || unnamed.keys().any(|id| !reg.fund_guesses.contains_key(id));
    let at = reg.registro_at.as_deref();
    let registry_due = (!wanted.is_empty() || !unmapped.is_empty() || !unnamed.is_empty())
        && (force
            || older_than(at, registry::REFRESH_DAYS * 24)
            || (missing && older_than(at, 20)));
    let portfolios_due = indices
        .iter()
        .filter(|i| {
            force
                || ports
                    .get(*i)
                    .is_none_or(|p| older_than(Some(&p.fetched_at), b3_index::REFRESH_HOURS))
        })
        .cloned()
        .collect();
    Ok(Plan {
        wanted,
        unmapped,
        unnamed,
        registry_due,
        portfolios_due,
        cvm: enabled(conn, SourceId::Cvm),
        b3: enabled(conn, SourceId::B3),
        yahoo: enabled(conn, SourceId::Yahoo),
    })
}

pub(crate) fn run(r: &mut Runner<'_>, _today: NaiveDate) -> Result<()> {
    let force = r.plan.force;
    let p = r.with(|c| plan(c, force))?;
    if p.registry_due && p.cvm {
        // Nome longo dos ETFs sem CNPJ (o Yahoo traz a denominação da CVM).
        let mut names: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (id, (ticker, keys)) in &p.unmapped {
            let mut keys = keys.clone();
            if p.yahoo && !ticker.is_empty() {
                let url = registry::yahoo_meta_url(&format!("{}.SA", ticker.to_uppercase()));
                let id2 = id.clone();
                let mut got: Option<String> = None;
                r.step(
                    Req {
                        source: SyncSource::Funds,
                        label: crate::i18n::t_args(
                            "nucleo-score-passo-yahoo-etf",
                            &crate::i18n::args(&[("ticker", ticker.to_string().into())]),
                        ),
                        url: &url,
                        ua: "Mozilla/5.0",
                        remember: false,
                    },
                    registry::parse_yahoo_name,
                    |c, name, _| {
                        let Some(n) = name else { return Ok(0) };
                        let mut reg = registry::load(c)?;
                        reg.etf_names.insert(id2, n.clone());
                        registry::save(c, &reg)?;
                        got = Some(n);
                        Ok(1)
                    },
                )?;
                if let Some(n) = got {
                    keys.push(registry::name_key(&n));
                }
            }
            names.insert(id.clone(), keys);
        }
        let wanted = p.wanted.clone();
        r.step(
            Req {
                source: SyncSource::Funds,
                label: crate::i18n::t("nucleo-score-fundo-fonte-registro"),
                url: registry::REGISTRO_URL,
                ua: "",
                remember: false,
            },
            |b| registry::parse_registro(b, &wanted, &names, &p.unnamed),
            registry::apply_registro,
        )?;
        // O extrato vale também para os ETFs achados agora (que não estão nele, mas não custa).
        let wanted = r.with(|c| {
            let reg = registry::load(c)?;
            let mut w = p.wanted.clone();
            w.extend(reg.etfs.values().cloned());
            Ok(w)
        })?;
        r.step(
            Req {
                source: SyncSource::Funds,
                label: crate::i18n::t("nucleo-score-fundo-fonte-extrato"),
                url: registry::EXTRATO_URL,
                ua: "",
                remember: false,
            },
            |b| registry::parse_extrato(b, &wanted),
            registry::apply_extrato,
        )?;
    }
    // Índices achados agora pelo nome do registro também entram.
    let indices: BTreeSet<String> = if p.registry_due {
        r.with(|c| plan(c, force))?.portfolios_due
    } else {
        p.portfolios_due.clone()
    };
    if p.b3 {
        for idx in indices {
            let url = b3_index::url(&idx);
            let i2 = idx.clone();
            r.step(
                Req {
                    source: SyncSource::Funds,
                    label: crate::i18n::t_args(
                        "nucleo-score-passo-carteira",
                        &crate::i18n::args(&[("indice", idx.to_string().into())]),
                    ),
                    url: &url,
                    ua: "",
                    remember: false,
                },
                b3_index::parse,
                |c, parsed, now| b3_index::apply(c, &i2, parsed, now),
            )?;
        }
    }
    Ok(())
}
