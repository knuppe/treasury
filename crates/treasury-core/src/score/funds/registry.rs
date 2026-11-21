//! Cadastro dos fundos e ETFs cadastrados no app, pela CVM (dados abertos): registro de
//! fundos e classes da Resolução 175 (classe, benchmark, constituição, gestor, patrimônio)
//! e extrato de informações (taxas de administração e performance, prazos de resgate).
//! Os arquivos têm todos os fundos do país; só os CNPJs de interesse são guardados, no
//! ajuste técnico `score.fund_registry` (texto da CVM: dado, nunca instrução).
//!
//! ETFs da B3 não têm CNPJ no ticker: o nome do ativo (ou o nome longo do Yahoo) é
//! comparado com a denominação das classes de fundos de índice do registro. Fundos e planos
//! de previdência cadastrados sem CNPJ também são procurados pelo nome (`fund_key`), mas o
//! CNPJ achado fica só como sugestão (`fund_guesses`): quem confirma é o usuário, no ativo.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::research::cvm::{Zip, field, read_csv};
use crate::research::{cnpj_digits, norm};

pub const REGISTRO_URL: &str =
    "https://dados.cvm.gov.br/dados/FI/CAD/DADOS/registro_fundo_classe.zip";
pub const EXTRATO_URL: &str = "https://dados.cvm.gov.br/dados/FI/DOC/EXTRATO/DADOS/extrato_fi.csv";
pub const SETTING: &str = "score.fund_registry";
/// Cadastro e extrato mudam pouco: baixados de novo no máximo uma vez por semana (ou antes,
/// se aparecer fundo ou ETF novo).
pub const REFRESH_DAYS: i64 = 7;

/// Dados de um fundo (ou classe) no registro da CVM.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FundInfo {
    pub cnpj: String,
    pub name: String,
    /// Tipo da classe (FIF, FIIM = fundo de índice…).
    pub kind: String,
    /// Classificação CVM (Renda Fixa, Ações, Multimercado, Cambial).
    pub classification: Option<String>,
    pub anbima: Option<String>,
    /// Indicador de desempenho declarado (DI de um dia, Ibovespa…).
    pub benchmark: Option<String>,
    /// Constituição (a mais antiga entre o fundo e a classe).
    pub inception: Option<String>,
    pub manager: Option<String>,
    pub administrator: Option<String>,
    pub exclusive: bool,
    /// Patrimônio líquido do registro e a data dele.
    pub nav: Option<String>,
    pub nav_date: Option<String>,
    /// Do extrato: taxas como fração (0,01 = 1% a.a.).
    pub admin_fee: Option<String>,
    pub perf_fee: Option<String>,
    pub perf_benchmark: Option<String>,
    /// Dias de conversão + dias de pagamento do resgate.
    pub redemption_days: Option<i64>,
    pub extrato_date: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Registry {
    pub funds: BTreeMap<String, FundInfo>,
    /// ETF da B3 (id do ativo) → CNPJ achado pelo nome.
    #[serde(default)]
    pub etfs: BTreeMap<String, String>,
    /// Nome longo dos ETFs no Yahoo (id do ativo → nome), para achar o CNPJ.
    #[serde(default)]
    pub etf_names: BTreeMap<String, String>,
    /// Fundo cadastrado sem CNPJ (id do ativo) → CNPJ provável, achado pelo nome (um único
    /// fundo ativo com o mesmo nome). Sugestão: não entra na coleta de cotas.
    #[serde(default)]
    pub fund_guesses: BTreeMap<String, String>,
    pub registro_at: Option<String>,
    pub extrato_at: Option<String>,
}

pub fn load(conn: &Connection) -> Result<Registry> {
    Ok(crate::settings::get(conn, SETTING)?.unwrap_or_default())
}

pub fn save(conn: &Connection, r: &Registry) -> Result<()> {
    crate::settings::set(conn, SETTING, r)
}

/// CNPJ de um fundo cadastrado (o do ativo ou, nos ETFs, o achado pelo nome).
pub fn cnpj_of(reg: &Registry, asset_id: &str, cnpj: Option<&str>) -> Option<String> {
    cnpj.map(cnpj_digits)
        .filter(|c| c.len() == 14)
        .or_else(|| reg.etfs.get(asset_id).cloned())
}

/// Nome normalizado para comparar (sem acento, pontuação nem o sufixo de responsabilidade).
pub fn name_key(s: &str) -> String {
    let n = norm(s);
    let cut = [" responsabilidade limitada", " resp limitada", " resp ltda"]
        .iter()
        .fold(n, |acc, suf| match acc.find(suf) {
            Some(i) => acc[..i].to_string(),
            None => acc,
        });
    cut.trim().to_string()
}

/// Nome de fundo reduzido ao que o distingue, para casar abreviações ("FICFI RF") com a
/// denominação da CVM ("FUNDO DE INVESTIMENTO EM COTAS DE FUNDOS DE INVESTIMENTO RENDA FIXA",
/// ou "CLASSE … RESPONSABILIDADE LIMITADA" depois da Resolução 175): tira as palavras da
/// estrutura do fundo e expande as siglas de classe.
pub fn fund_key(s: &str) -> String {
    const STRUCTURE: &[&str] = &[
        "fundo",
        "fundos",
        "de",
        "do",
        "da",
        "dos",
        "das",
        "em",
        "investimento",
        "investimentos",
        "cotas",
        "cota",
        "fi",
        "fic",
        "ficfi",
        "fif",
        "ficfif",
        "fiq",
        "fc",
        "classe",
        "classes",
        "subclasse",
        // Classe de Investimento em Cotas (Resolução 175): o "FIC" do nome novo.
        "cic",
        "responsabilidade",
        "limitada",
        "ltda",
        "resp",
        "rl",
    ];
    let key = name_key(s);
    let mut out: Vec<&str> = vec![];
    for w in key.split_whitespace() {
        match w {
            "rf" => out.extend(["renda", "fixa"]),
            "cp" => out.extend(["credito", "privado"]),
            "lp" => out.extend(["longo", "prazo"]),
            "ref" => out.push("referenciado"),
            "mm" | "multi" => out.push("multimercado"),
            "prev" | "previd" => out.push("previdenciario"),
            "fia" => out.push("acoes"),
            w if STRUCTURE.contains(&w) => {}
            w => out.push(w),
        }
    }
    out.join(" ")
}

fn opt(s: String) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

fn date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim().get(..10)?, "%Y-%m-%d").ok()
}

/// O que sai do registro: fundos de interesse e ETFs achados pelo nome.
#[derive(Debug, Default)]
pub struct RegistroParsed {
    pub funds: Vec<FundInfo>,
    pub etfs: Vec<(String, String)>,
    /// Fundos sem CNPJ (id do ativo → CNPJ provável), só quando um único fundo ativo casa.
    pub fund_guesses: Vec<(String, String)>,
}

/// Lê o zip do registro (registro_fundo.csv e registro_classe.csv). `etf_names`: id do ativo
/// → nomes candidatos (já com `name_key`); `fund_names`: fundos sem CNPJ, id do ativo → nome
/// já com `fund_key`.
pub fn parse_registro(
    bytes: &[u8],
    wanted: &HashSet<String>,
    etf_names: &BTreeMap<String, Vec<String>>,
    fund_names: &BTreeMap<String, String>,
) -> std::result::Result<RegistroParsed, String> {
    let mut guesses: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // Nome do ativo contido na denominação (CNPJ, se a classe é previdenciária).
    let mut partial: BTreeMap<String, BTreeSet<(String, bool)>> = BTreeMap::new();
    let mut z = Zip::open(bytes)?;
    // Fundos (gestor, administrador, constituição), pelo ID do registro.
    struct Fundo {
        cnpj: String,
        manager: Option<String>,
        administrator: Option<String>,
        inception: Option<NaiveDate>,
    }
    let mut fundos: BTreeMap<String, Fundo> = BTreeMap::new();
    if let Some(name) = z.find(|n| n.ends_with("registro_fundo.csv")) {
        z.each_row(&name, |h, r| {
            let id = field(r, h.idx(&["ID_Registro_Fundo"]));
            if id.is_empty() {
                return Ok(());
            }
            let inception = [h.idx(&["Data_Constituicao"]), h.idx(&["Data_Registro"])]
                .into_iter()
                .filter_map(|i| date(&field(r, i)))
                .min();
            fundos.insert(
                id,
                Fundo {
                    cnpj: cnpj_digits(&field(r, h.idx(&["CNPJ_Fundo"]))),
                    manager: opt(field(r, h.idx(&["Gestor"]))),
                    administrator: opt(field(r, h.idx(&["Administrador"]))),
                    inception,
                },
            );
            Ok(())
        })?;
    }
    let Some(classes) = z.find(|n| n.ends_with("registro_classe.csv")) else {
        return Err(crate::i18n::t("nucleo-score-erro-registro-sem-classes"));
    };
    let mut out = RegistroParsed::default();
    let mut found: HashSet<String> = HashSet::new();
    let mut etf_found: HashSet<String> = HashSet::new();
    z.each_row(&classes, |h, r| {
        let cnpj = cnpj_digits(&field(r, h.idx(&["CNPJ_Classe"])));
        let kind = field(r, h.idx(&["Tipo_Classe"]));
        let name = field(r, h.idx(&["Denominacao_Social"]));
        let active = !field(r, h.idx(&["Situacao"]))
            .to_lowercase()
            .contains("cancel");
        let index_fund = kind.contains("FIIM") || kind.to_lowercase().contains("indice");
        let mut etf_hits = vec![];
        if index_fund && active {
            let key = name_key(&name);
            for (asset, names) in etf_names {
                if !etf_found.contains(asset) && names.contains(&key) {
                    etf_hits.push(asset.clone());
                }
            }
        }
        let mut guessed = false;
        if active && !index_fund && !fund_names.is_empty() {
            let key = fund_key(&name);
            let words: HashSet<&str> = key.split_whitespace().collect();
            for (asset, k) in fund_names {
                if !k.is_empty() && *k == key {
                    guesses
                        .entry(asset.clone())
                        .or_default()
                        .insert(cnpj.clone());
                    guessed = true;
                } else if k.split_whitespace().count() >= 2
                    && k.split_whitespace().all(|w| words.contains(w))
                {
                    // Nome curto do plano ("CAIXA FIC PREV 250", "PLANOPREV ARROJADO") contido
                    // na denominação da CVM: candidato, se não houver nome igual.
                    partial
                        .entry(asset.clone())
                        .or_default()
                        .insert((cnpj.clone(), words.contains("previdenciario")));
                    guessed = true;
                }
            }
        }
        if !wanted.contains(&cnpj) && etf_hits.is_empty() && !guessed {
            return Ok(());
        }
        // Classe ativa vence a cancelada do mesmo CNPJ.
        if found.contains(&cnpj) && !active {
            return Ok(());
        }
        for a in etf_hits {
            etf_found.insert(a.clone());
            out.etfs.push((a, cnpj.clone()));
        }
        let fundo = fundos.get(&field(r, h.idx(&["ID_Registro_Fundo"])));
        let class_inception = date(&field(r, h.idx(&["Data_Constituicao"])))
            .into_iter()
            .chain(date(&field(r, h.idx(&["Data_Inicio"]))))
            .min();
        // Classe criada na adaptação à Resolução 175: vale a constituição do fundo de mesmo
        // CNPJ (ou a do fundo, se for a única classe).
        let inception = class_inception
            .into_iter()
            .chain(fundo.filter(|f| f.cnpj == cnpj).and_then(|f| f.inception))
            .min();
        let info = FundInfo {
            cnpj: cnpj.clone(),
            name,
            kind,
            classification: opt(field(r, h.idx(&["Classificacao"]))),
            anbima: opt(field(r, h.idx(&["Classificacao_Anbima"]))),
            benchmark: opt(field(r, h.idx(&["Indicador_Desempenho"]))),
            inception: inception.map(crate::time::fmt_date),
            manager: fundo.and_then(|f| f.manager.clone()),
            administrator: fundo.and_then(|f| f.administrator.clone()),
            exclusive: field(r, h.idx(&["Exclusivo"])).eq_ignore_ascii_case("S"),
            nav: opt(field(r, h.idx(&["Patrimonio_Liquido"]))),
            nav_date: opt(field(r, h.idx(&["Data_Patrimonio_Liquido"]))),
            ..Default::default()
        };
        out.funds.retain(|f| f.cnpj != cnpj);
        out.funds.push(info);
        found.insert(cnpj);
        Ok(())
    })?;
    // Sem nome igual, vale o único fundo cuja denominação contém o nome do ativo (entre
    // vários, o único previdenciário: planos de previdência aplicam em FIE).
    for (asset, cands) in partial {
        if guesses.contains_key(&asset) {
            continue;
        }
        let prev: BTreeSet<String> = cands
            .iter()
            .filter(|(_, p)| *p)
            .map(|(c, _)| c.clone())
            .collect();
        let all: BTreeSet<String> = cands.into_iter().map(|(c, _)| c).collect();
        let pick = if all.len() == 1 { all } else { prev };
        if pick.len() == 1 {
            guesses.insert(asset, pick);
        }
    }
    // Nome ambíguo (mais de um fundo ativo) não vira sugestão.
    for (asset, cnpjs) in guesses {
        if cnpjs.len() == 1 {
            out.fund_guesses
                .push((asset, cnpjs.into_iter().next().unwrap_or_default()));
        }
    }
    let keep: HashSet<String> = out
        .etfs
        .iter()
        .chain(&out.fund_guesses)
        .map(|(_, c)| c.clone())
        .chain(wanted.iter().cloned())
        .collect();
    out.funds.retain(|f| keep.contains(&f.cnpj));
    // Fundo de interesse sem classe (registro antigo): só os dados do fundo.
    for f in fundos.values() {
        if wanted.contains(&f.cnpj) && !found.contains(&f.cnpj) {
            out.funds.push(FundInfo {
                cnpj: f.cnpj.clone(),
                manager: f.manager.clone(),
                administrator: f.administrator.clone(),
                inception: f.inception.map(crate::time::fmt_date),
                ..Default::default()
            });
        }
    }
    Ok(out)
}

/// Taxas e prazos do extrato de informações (uma linha por fundo, a mais recente).
#[derive(Debug, Clone, PartialEq)]
pub struct Extrato {
    pub cnpj: String,
    pub date: Option<String>,
    pub admin_fee: Option<String>,
    pub perf_fee: Option<String>,
    pub perf_benchmark: Option<String>,
    pub redemption_days: Option<i64>,
}

/// Percentual da CVM (1.5 = 1,5%) → fração em texto exato.
fn pct(s: &str) -> Option<String> {
    let d = crate::research::parse_decimal(s)?;
    Some(crate::research::dstr(d / rust_decimal::Decimal::from(100)))
}

pub fn parse_extrato(
    bytes: &[u8],
    wanted: &HashSet<String>,
) -> std::result::Result<Vec<Extrato>, String> {
    let mut by: BTreeMap<String, Extrato> = BTreeMap::new();
    read_csv(bytes, |h, r| {
        let cnpj = cnpj_digits(&field(r, h.idx(&["CNPJ_FUNDO_CLASSE", "CNPJ_FUNDO"])));
        if !wanted.contains(&cnpj) {
            return Ok(());
        }
        let date = opt(field(r, h.idx(&["DT_COMPTC"])));
        if by
            .get(&cnpj)
            .is_some_and(|e| e.date.as_deref() >= date.as_deref())
        {
            return Ok(());
        }
        let has_perf = field(r, h.idx(&["EXISTE_TAXA_PERFM"]));
        let perf = pct(&field(r, h.idx(&["TAXA_PERFM"])))
            .or_else(|| has_perf.eq_ignore_ascii_case("N").then(|| "0".to_string()));
        let days = |k: &str| field(r, h.idx(&[k])).trim().parse::<i64>().ok();
        let redemption = match (days("QT_DIA_CONVERSAO_COTA"), days("QT_DIA_PAGTO_RESGATE")) {
            (None, None) => None,
            (a, b) => Some(a.unwrap_or(0) + b.unwrap_or(0)),
        };
        by.insert(
            cnpj.clone(),
            Extrato {
                cnpj,
                date,
                admin_fee: pct(&field(r, h.idx(&["TAXA_ADM"]))),
                perf_fee: perf,
                perf_benchmark: opt(field(r, h.idx(&["PARAM_TAXA_PERFM"]))),
                redemption_days: redemption,
            },
        );
        Ok(())
    })?;
    Ok(by.into_values().collect())
}

/// Grava o que veio do registro (substitui os dados de cadastro, mantém os do extrato).
pub fn apply_registro(conn: &Connection, p: RegistroParsed, now: &str) -> Result<u64> {
    let mut reg = load(conn)?;
    let n = p.funds.len() + p.etfs.len();
    // As sugestões são refeitas a cada registro (some a do fundo que ganhou CNPJ).
    reg.fund_guesses = p.fund_guesses.into_iter().collect();
    for f in p.funds {
        let old = reg.funds.remove(&f.cnpj).unwrap_or_default();
        reg.funds.insert(
            f.cnpj.clone(),
            FundInfo {
                admin_fee: old.admin_fee,
                perf_fee: old.perf_fee,
                perf_benchmark: old.perf_benchmark,
                redemption_days: old.redemption_days,
                extrato_date: old.extrato_date,
                ..f
            },
        );
    }
    for (asset, cnpj) in p.etfs {
        reg.etfs.insert(asset, cnpj);
    }
    reg.registro_at = Some(now.to_string());
    save(conn, &reg)?;
    Ok(n as u64)
}

pub fn apply_extrato(conn: &Connection, rows: Vec<Extrato>, now: &str) -> Result<u64> {
    let mut reg = load(conn)?;
    let n = rows.len();
    for e in rows {
        let f = reg.funds.entry(e.cnpj.clone()).or_insert_with(|| FundInfo {
            cnpj: e.cnpj.clone(),
            ..Default::default()
        });
        f.admin_fee = e.admin_fee;
        f.perf_fee = e.perf_fee;
        f.perf_benchmark = e.perf_benchmark;
        f.redemption_days = e.redemption_days;
        f.extrato_date = e.date;
    }
    reg.extrato_at = Some(now.to_string());
    save(conn, &reg)?;
    Ok(n as u64)
}

/// Nome longo de um ETF no Yahoo (meta do gráfico).
pub fn parse_yahoo_name(bytes: &[u8]) -> std::result::Result<Option<String>, String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let meta = &v["chart"]["result"][0]["meta"];
    Ok(meta["longName"]
        .as_str()
        .or(meta["shortName"].as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()))
}

pub fn yahoo_meta_url(symbol: &str) -> String {
    format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=1d&interval=1d",
        symbol.replace('^', "%5E")
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::research::cvm::fixtures::zip_latin1;

    pub(crate) fn registro_zip() -> Vec<u8> {
        zip_latin1(&[
            (
                "registro_fundo.csv",
                "ID_Registro_Fundo;CNPJ_Fundo;Codigo_CVM;Data_Registro;Data_Constituicao;Tipo_Fundo;Denominacao_Social;Data_Cancelamento;Situacao;Data_Inicio_Situacao;Data_Adaptacao_RCVM175;Data_Inicio_Exercicio_Social;Data_Fim_Exercicio_Social;Patrimonio_Liquido;Data_Patrimonio_Liquido;Diretor;CNPJ_Administrador;Administrador;Tipo_Pessoa_Gestor;CPF_CNPJ_Gestor;Gestor\n\
                 7847;10406511000161;27;2008-11-19;2008-11-19;FIIM;ISHARES IBOVESPA CLASSE DE ÍNDICE - RESPONSABILIDADE LIMITADA;;Em Funcionamento Normal;2008-11-19;2025-04-23;;;9498172265.94;2025-04-30;X;01522368000182;BANCO BNP PARIBAS BRASIL S/A;PJ;10979208000158;BLACKROCK BRASIL GESTORA DE INVESTIMENTOS LTDA\n\
                 60588;32893503000120;339210;2019-10-04;2019-10-03;FI;TREND DI SIMPLES FUNDO DE INVESTIMENTO EM RENDA FIXA SIMPLES RESPONSABILIDADE LIMITADA;;Em Funcionamento Normal;2019-10-07;2024-10-28;;;2646855052.17;2024-12-27;Y;05389174000101;XP SERVIÇOS FINANCEIROS DTVM Ltda.;PJ;37918829000188;XP ALLOCATION ASSET MANAGEMENT LTDA\n"
                    .to_string(),
            ),
            (
                "registro_classe.csv",
                "ID_Registro_Fundo;ID_Registro_Classe;CNPJ_Classe;Codigo_CVM;Data_Registro;Data_Constituicao;Data_Inicio;Tipo_Classe;Denominacao_Social;Situacao;Data_Inicio_Situacao;Classificacao;Indicador_Desempenho;Classe_Cotas;Classificacao_Anbima;Tributacao_Longo_Prazo;Entidade_Investimento;Permitido_Aplicacao_CemPorCento_Exterior;Classe_ESG;Forma_Condominio;Exclusivo;Publico_Alvo;Patrimonio_Liquido;Data_Patrimonio_Liquido;CNPJ_Auditor;Auditor;CNPJ_Custodiante;Custodiante;CNPJ_Controlador;Controlador\n\
                 7847;36216;10406511000161;612;2025-04-23;2025-04-23;2025-04-23;Classes de Cotas de Fundos FIIM;ISHARES IBOVESPA CLASSE DE ÍNDICE - RESPONSABILIDADE LIMITADA;Em Funcionamento Normal;2025-04-23;;;;;;;;N;;;;17792473611.23;2026-10-08;1;A;2;B;;\n\
                 60588;9261;32893503000120;100366;2024-10-28;2019-10-03;2024-10-28;Classes de Cotas de Fundos FIF;TREND DI SIMPLES FUNDO DE INVESTIMENTO EM RENDA FIXA SIMPLES RESPONSABILIDADE LIMITADA;Em Funcionamento Normal;2024-10-28;Renda Fixa;DI de um dia;N;Renda Fixa Simples;N/A;;N;N;Aberto;N;Público Geral;1711278285.77;2026-10-14;1;A;2;B;3;C\n\
                 1;2;99999999000199;1;2020-01-01;2020-01-01;2020-01-01;Classes de Cotas de Fundos FIF;OUTRO;Em Funcionamento Normal;2020-01-01;Ações;Ibovespa;N;;;;N;N;Aberto;N;;1;2026-10-14;;;;;;\n"
                    .to_string(),
            ),
        ])
    }

    #[test]
    fn registry_keeps_wanted_funds_and_finds_etfs_by_name() {
        let wanted: HashSet<String> = ["32893503000120".to_string()].into();
        let names: BTreeMap<String, Vec<String>> = [(
            "etf1".to_string(),
            vec![name_key(
                "ISHARES IBOVESPA CLASSE DE ÍNDICE - RESPONSABILIDADE LIMITADA",
            )],
        )]
        .into();
        let p = parse_registro(&registro_zip(), &wanted, &names, &BTreeMap::new()).unwrap();
        assert_eq!(p.etfs, vec![("etf1".into(), "10406511000161".into())]);
        assert_eq!(p.funds.len(), 2);
        let trend = p.funds.iter().find(|f| f.cnpj == "32893503000120").unwrap();
        assert_eq!(trend.classification.as_deref(), Some("Renda Fixa"));
        assert_eq!(trend.benchmark.as_deref(), Some("DI de um dia"));
        assert_eq!(trend.inception.as_deref(), Some("2019-10-03"));
        assert_eq!(
            trend.manager.as_deref(),
            Some("XP ALLOCATION ASSET MANAGEMENT LTDA")
        );
        assert_eq!(trend.nav.as_deref(), Some("1711278285.77"));
        // ETF: a classe foi criada na adaptação (2025), o fundo é de 2008.
        let bova = p.funds.iter().find(|f| f.cnpj == "10406511000161").unwrap();
        assert_eq!(bova.inception.as_deref(), Some("2008-11-19"));
        assert!(bova.name.contains("ÍNDICE"));
    }

    #[test]
    fn extrato_fees_as_fractions() {
        let csv = "TP_FUNDO_CLASSE;CNPJ_FUNDO_CLASSE;DENOM_SOCIAL;DT_COMPTC;TAXA_ADM;EXISTE_TAXA_PERFM;TAXA_PERFM;PARAM_TAXA_PERFM;QT_DIA_CONVERSAO_COTA;QT_DIA_PAGTO_RESGATE\n\
            FI;32.893.503/0001-20;TREND;2021-09-06;0.020000;N;;;0;0\n\
            FI;28.858.089/0001-40;4UM;2022-09-29;0.400000;S;20.000000000000;IMA-B5;30;1\n\
            FI;28.858.089/0001-40;4UM;2020-01-01;9.000000;N;;;0;0\n\
            FI;11.111.111/0001-11;OUTRO;2022-09-29;1;N;;;0;0\n";
        let wanted: HashSet<String> =
            ["32893503000120".to_string(), "28858089000140".into()].into();
        let rows = parse_extrato(csv.as_bytes(), &wanted).unwrap();
        assert_eq!(rows.len(), 2);
        let um = rows.iter().find(|r| r.cnpj == "28858089000140").unwrap();
        assert_eq!(um.admin_fee.as_deref(), Some("0.004"));
        assert_eq!(um.perf_fee.as_deref(), Some("0.2"));
        assert_eq!(um.perf_benchmark.as_deref(), Some("IMA-B5"));
        assert_eq!(um.redemption_days, Some(31));
        let trend = rows.iter().find(|r| r.cnpj == "32893503000120").unwrap();
        assert_eq!(trend.admin_fee.as_deref(), Some("0.0002"));
        assert_eq!(trend.perf_fee.as_deref(), Some("0"));
    }

    #[test]
    fn yahoo_meta_name() {
        let body = r#"{"chart":{"result":[{"meta":{"longName":"ISHARES IBOVESPA CLASSE DE ÍNDICE - RESPONSABILIDADE LIMITADA","shortName":"ISHARES BOVACI"}}]}}"#.as_bytes();
        let n = parse_yahoo_name(body).unwrap().unwrap();
        assert_eq!(
            name_key(&n),
            name_key("iShares Ibovespa Classe de Índice - Responsabilidade Limitada")
        );
        assert_eq!(name_key(&n), "ishares ibovespa classe de indice");
    }

    #[test]
    fn fund_key_matches_abbreviations_and_res175_names() {
        let mine = fund_key("NU RESERVA IMEDIATA FICFI RF REFERENCIADO DI");
        assert_eq!(mine, "nu reserva imediata renda fixa referenciado di");
        for cvm in [
            "NU RESERVA IMEDIATA FUNDO DE INVESTIMENTO EM COTAS DE FUNDOS DE INVESTIMENTO RENDA \
             FIXA REFERENCIADO DI",
            "NU RESERVA IMEDIATA FIC FIF RENDA FIXA REFERENCIADO DI - RESPONSABILIDADE LIMITADA",
            "NU RESERVA IMEDIATA CLASSE DE INVESTIMENTO EM COTAS RENDA FIXA REFERENCIADO DI RESP \
             LIMITADA",
            // Como está no registro da CVM desde 2025.
            "NU RESERVA IMEDIATA FIF CIC RENDA FIXA REFERENCIADO DI RESPONSABILIDADE LIMITADA",
        ] {
            assert_eq!(fund_key(cvm), mine, "{cvm}");
        }
        assert_ne!(fund_key("NU RESERVA PLANEJADA FIC FIF RF"), mine);
        // O master do mesmo nome não é o fundo vendido ao público.
        assert_ne!(
            fund_key("NU RESERVA IMEDIATA MASTER FIF RF REFERENCIADO DI RESPONSABILIDADE LIMITADA"),
            mine
        );
    }

    #[test]
    fn registry_guesses_cnpj_of_fund_without_one_only_when_unique() {
        let wanted: HashSet<String> = HashSet::new();
        let names: BTreeMap<String, String> = [
            (
                "trend".to_string(),
                fund_key("TREND DI SIMPLES FI RF SIMPLES"),
            ),
            (
                "nada".to_string(),
                fund_key("FUNDO QUE NAO EXISTE FIC FI RF"),
            ),
        ]
        .into();
        let p = parse_registro(&registro_zip(), &wanted, &BTreeMap::new(), &names).unwrap();
        assert_eq!(
            p.fund_guesses,
            vec![("trend".to_string(), "32893503000120".to_string())]
        );
        // O fundo achado vem junto (classificação para o segmento); os outros não.
        assert_eq!(p.funds.len(), 1, "{:?}", p.funds);
        assert_eq!(p.funds[0].classification.as_deref(), Some("Renda Fixa"));
    }

    /// Plano de previdência com o CNPJ do plano (fora do registro): o nome curto do plano
    /// ("CAIXA FIC PREV 250") contido na denominação de um único fundo vira sugestão; entre
    /// vários, vale o único previdenciário; nome de uma palavra só não casa.
    #[test]
    fn short_pension_names_match_the_fund_whose_name_contains_them() {
        let head = "ID_Registro_Fundo;ID_Registro_Classe;CNPJ_Classe;Codigo_CVM;Data_Registro;Data_Constituicao;Data_Inicio;Tipo_Classe;Denominacao_Social;Situacao;Data_Inicio_Situacao;Classificacao;Indicador_Desempenho;Classe_Cotas;Classificacao_Anbima;Tributacao_Longo_Prazo;Entidade_Investimento;Permitido_Aplicacao_CemPorCento_Exterior;Classe_ESG;Forma_Condominio;Exclusivo;Publico_Alvo;Patrimonio_Liquido;Data_Patrimonio_Liquido;CNPJ_Auditor;Auditor;CNPJ_Custodiante;Custodiante;CNPJ_Controlador;Controlador";
        let row = |cnpj: &str, name: &str| {
            format!(
                "1;1;{cnpj};1;2020-01-01;2020-01-01;2020-01-01;Classes de Cotas de Fundos FIF;{name};Em Funcionamento Normal;2020-01-01;Renda Fixa;;N;Previdência RF;;;N;N;Aberto;N;;1;2026-10-14;;;;;;"
            )
        };
        let classes = [
            row(
                "11111111000111",
                "CAIXA FIC PREVIDENCIÁRIO 250 RENDA FIXA LONGO PRAZO",
            ),
            row("22222222000122", "CAIXA FIC 250 RENDA FIXA LONGO PRAZO"),
            row(
                "33333333000133",
                "PLANOPREV ARROJADO FUNDO DE INVESTIMENTO MULTIMERCADO PREVIDENCIÁRIO",
            ),
            row("44444444000144", "CAIXA RENDA FIXA"),
        ]
        .join("\n");
        let zip = zip_latin1(&[("registro_classe.csv", format!("{head}\n{classes}\n"))]);
        let names: BTreeMap<String, String> = [
            ("caixa".to_string(), fund_key("CAIXA FIC PREV 250")),
            ("planoprev".to_string(), fund_key("PLANOPREV-Arrojado")),
            ("so-caixa".to_string(), fund_key("CAIXA")),
        ]
        .into();
        let p = parse_registro(&zip, &HashSet::new(), &BTreeMap::new(), &names).unwrap();
        let g: BTreeMap<String, String> = p.fund_guesses.into_iter().collect();
        assert_eq!(g.get("caixa").map(String::as_str), Some("11111111000111"));
        assert_eq!(
            g.get("planoprev").map(String::as_str),
            Some("33333333000133")
        );
        assert!(!g.contains_key("so-caixa"));
        assert!(p.funds.iter().any(|f| f.cnpj == "11111111000111"));
        assert!(!p.funds.iter().any(|f| f.cnpj == "44444444000144"));
    }
}
