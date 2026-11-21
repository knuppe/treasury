//! Banco Central — IF.data (`olinda.bcb.gov.br`, API OData pública, sem chave): indicadores
//! prudenciais e contábeis dos conglomerados prudenciais e instituições independentes
//! (`TipoInstituicao = 1`), por trimestre. Só parse de bytes e leitura/gravação das tabelas
//! técnicas `bank_institution`, `bank_prudential` e `bank_ifdata_fetch`; a coleta (planejar →
//! baixar sem travar o banco → gravar) fica em [`super::sync`].
//!
//! Relatórios usados:
//! - cadastro: CodInst, nome, tipo de consolidado (Tcb), segmento prudencial (Sr, S1 a S5),
//!   conglomerado prudencial e CNPJ (8 dígitos) da instituição líder;
//! - 1 (resumo): Basileia, ativo total, patrimônio líquido, lucro líquido, carteira de crédito,
//!   captações; 5 (capital): capital principal, nível I, razão de alavancagem, RWA;
//!   16 (carteira por nível de risco): inadimplência de 90 dias, ativos problemáticos, carteira
//!   total — baixados completos (todas as instituições), ~4 a 13 MB por trimestre;
//! - 2 (ativo): perda esperada; 4 (resultado): provisão, intermediação, tarifas, pessoal e
//!   administrativas — ~30 MB cada um completos, por isso só para as instituições de interesse
//!   (filtro `CodInst eq '…' or …`, em pedidos de até ~6 KB).
//!
//! Convenções (conferidas na API em 10/2026):
//! - valores em reais; razões (Basileia, capital, alavancagem) em fração;
//! - **resultados são do semestre até a data**: março e setembro trazem 3 meses; junho e
//!   dezembro, 6 (ex.: lucro do Itaú prudencial: 202503 = 11,1 bi, 202506 = 22,6 bi, 202509 =
//!   11,8 bi, 202512 = 24,1 bi). Os 12 meses saem de [`twelve_months`];
//! - despesas e perdas gravadas como magnitude positiva (`expected_loss`, `personnel_expense`,
//!   `admin_expense`); `provision_expense` é o resultado com perda esperada com o sinal trocado
//!   (positivo = despesa; uma reversão líquida fica negativa).

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::time::add_months;

pub const BASE: &str = "https://olinda.bcb.gov.br/olinda/servico/IFDATA/versao/v1/odata";
pub const SOURCE: &str = "bacen_ifdata";
/// Nome do "relatório" do cadastro em `bank_ifdata_fetch`.
pub const CADASTRO: &str = "cadastro";
/// Relatórios baixados completos.
pub const FULL_REPORTS: [&str; 3] = ["1", "5", "16"];
/// Relatórios baixados só para as instituições de interesse.
pub const FILTERED_REPORTS: [&str; 2] = ["2", "4"];
/// Trimestres guardados: o mais recente e os 4 anteriores.
pub const QUARTERS: usize = 5;
/// O trimestre mais recente é baixado de novo no máximo uma vez por semana (reapresentações).
pub const REFRESH_DAYS: i64 = 7;
/// Tamanho máximo de uma URL com filtro de instituições.
const MAX_URL: usize = 6000;
/// Resultados do IF.data: acumulados no semestre até a data (texto no idioma da instalação).
pub fn income_convention() -> String {
    crate::i18n::t("nucleo-pesquisa-ifdata-convencao")
}

/// Chaves dos itens gravados em `bank_prudential`.
pub mod items {
    pub const BASEL_RATIO: &str = "basel_ratio";
    pub const CET1_RATIO: &str = "cet1_ratio";
    pub const TIER1_RATIO: &str = "tier1_ratio";
    pub const LEVERAGE_RATIO: &str = "leverage_ratio";
    pub const RWA: &str = "rwa";
    pub const TOTAL_ASSETS: &str = "total_assets";
    pub const EQUITY: &str = "equity";
    /// Lucro líquido do semestre até a data (ver [`super::twelve_months`]).
    pub const NET_INCOME: &str = "net_income";
    pub const CREDIT_PORTFOLIO: &str = "credit_portfolio";
    pub const FUNDING: &str = "funding";
    /// Operações com atraso acima de 90 dias (relatório 16, "Inadimplência").
    pub const NPL90: &str = "npl90";
    pub const PROBLEM_ASSETS: &str = "problem_assets";
    /// Carteira de crédito classificada (relatório 16, "Total Geral").
    pub const CREDIT_TOTAL: &str = "credit_total";
    /// Perda esperada acumulada no balanço (magnitude).
    pub const EXPECTED_LOSS: &str = "expected_loss";
    /// Resultado com perda esperada do período, sinal trocado (positivo = despesa).
    pub const PROVISION_EXPENSE: &str = "provision_expense";
    pub const INTERMEDIATION_RESULT: &str = "intermediation_result";
    /// Rendas de tarifas bancárias + outras rendas de prestação de serviços.
    pub const FEE_INCOME: &str = "fee_income";
    pub const PERSONNEL_EXPENSE: &str = "personnel_expense";
    pub const ADMIN_EXPENSE: &str = "admin_expense";

    /// Resultados do período (semestre até a data), que somam no tempo.
    pub const FLOWS: &[&str] = &[
        NET_INCOME,
        PROVISION_EXPENSE,
        INTERMEDIATION_RESULT,
        FEE_INCOME,
        PERSONNEL_EXPENSE,
        ADMIN_EXPENSE,
    ];
    /// Razões (fração).
    pub const RATIOS: &[&str] = &[BASEL_RATIO, CET1_RATIO, TIER1_RATIO, LEVERAGE_RATIO];
}

/// Conta de um relatório → (item, sinal). Contas que caem no mesmo item são somadas.
fn mapping(report: &str, conta: &str) -> Option<(&'static str, i64)> {
    use items::*;
    Some(match (report, conta) {
        ("1", "79664") => (BASEL_RATIO, 1),
        ("1", "140220") => (TOTAL_ASSETS, 1),
        ("1", "140246") => (EQUITY, 1),
        ("1", "141870") => (NET_INCOME, 1),
        ("1", "141873") => (CREDIT_PORTFOLIO, 1),
        ("1", "140239") => (FUNDING, 1),
        ("5", "79659") => (CET1_RATIO, 1),
        ("5", "79660") => (TIER1_RATIO, 1),
        ("5", "79661") => (LEVERAGE_RATIO, 1),
        ("5", "79665") => (RWA, 1),
        ("16", "148834") => (NPL90, 1),
        ("16", "148833") => (PROBLEM_ASSETS, 1),
        ("16", "24454") => (CREDIT_TOTAL, 1),
        // Perda esperada (negativa no relatório) de operações de crédito, arrendamento,
        // outras operações com característica de crédito e transações de pagamento.
        ("2", "140202" | "140208" | "140213" | "145832") => (EXPECTED_LOSS, -1),
        ("4", "141842") => (PROVISION_EXPENSE, -1),
        ("4", "141851") => (INTERMEDIATION_RESULT, 1),
        ("4", "141856" | "141857") => (FEE_INCOME, 1),
        ("4", "141858") => (PERSONNEL_EXPENSE, -1),
        ("4", "141859") => (ADMIN_EXPENSE, -1),
        _ => return None,
    })
}

pub fn report_label(report: &str) -> String {
    crate::i18n::t(match report {
        CADASTRO => "nucleo-pesquisa-ifdata-relatorio-cadastro",
        "1" => "nucleo-pesquisa-ifdata-relatorio-resumo",
        "2" => "nucleo-pesquisa-ifdata-relatorio-ativo",
        "4" => "nucleo-pesquisa-ifdata-relatorio-resultado",
        "5" => "nucleo-pesquisa-ifdata-relatorio-capital",
        "16" => "nucleo-pesquisa-ifdata-relatorio-carteira",
        _ => "nucleo-pesquisa-ifdata-relatorio",
    })
}

// ---------------------------------------------------------------------------
// Períodos e URLs

/// `2026-06` → `202606`.
pub fn ano_mes(period: &str) -> String {
    period.replace('-', "")
}

/// `202606` → `2026-06`.
fn period_of(ano_mes: &str) -> Option<String> {
    let t = ano_mes.trim();
    (t.len() == 6 && t.chars().all(|c| c.is_ascii_digit()))
        .then(|| format!("{}-{}", &t[..4], &t[4..]))
}

fn period_date(period: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&format!("{period}-01"), "%Y-%m-%d").ok()
}

fn fmt_period(d: NaiveDate) -> String {
    format!("{:04}-{:02}", d.year(), d.month())
}

/// Último fim de trimestre (março, junho, setembro, dezembro) até `today`.
pub fn quarter_end(today: NaiveDate) -> String {
    let m = today.month() - (today.month() - 1) % 3 + 2;
    let m = if m > today.month() { m - 3 } else { m };
    if m == 0 {
        format!("{:04}-12", today.year() - 1)
    } else {
        format!("{:04}-{:02}", today.year(), m)
    }
}

/// `n` trimestres a partir de `latest`, para trás (`latest` incluído).
pub fn quarters_back(latest: &str, n: usize) -> Vec<String> {
    let Some(d) = period_date(latest) else {
        return vec![];
    };
    (0..n)
        .map(|i| fmt_period(add_months(d, -3 * i as i32)))
        .collect()
}

/// Mês `months` antes/depois do período.
fn shift(period: &str, months: i32) -> Option<String> {
    period_date(period).map(|d| fmt_period(add_months(d, months)))
}

/// Cadastro do trimestre; `probe` pede uma linha só (para saber se o trimestre já saiu).
pub fn cadastro_url(period: &str, probe: bool) -> String {
    format!(
        "{BASE}/IfDataCadastro(AnoMes=@AnoMes)?@AnoMes={}&$format=json{}",
        ano_mes(period),
        if probe { "&$top=1" } else { "" }
    )
}

/// Valores de um relatório no trimestre (todas as instituições, ou só `codes`).
pub fn values_url(period: &str, report: &str, codes: Option<&[String]>) -> String {
    let mut url = format!(
        "{BASE}/IfDataValores(AnoMes=@AnoMes,TipoInstituicao=@TipoInstituicao,Relatorio=@Relatorio)?@AnoMes={}&@TipoInstituicao=1&@Relatorio=%27{report}%27&$format=json",
        ano_mes(period)
    );
    if let Some(codes) = codes.filter(|c| !c.is_empty()) {
        let filter: Vec<String> = codes
            .iter()
            .map(|c| format!("CodInst%20eq%20%27{}%27", url_safe(c)))
            .collect();
        url.push_str("&$filter=");
        url.push_str(&filter.join("%20or%20"));
    }
    url
}

/// CodInst só com letras e dígitos (vem do próprio cadastro; nunca entra texto livre na URL).
fn url_safe(code: &str) -> String {
    code.chars().filter(char::is_ascii_alphanumeric).collect()
}

/// Divide as instituições em grupos cuja URL fica abaixo de ~6 KB.
pub fn chunk_codes(period: &str, report: &str, codes: &[String]) -> Vec<Vec<String>> {
    let base = values_url(period, report, None).len() + "&$filter=".len();
    let mut out: Vec<Vec<String>> = vec![];
    let mut cur: Vec<String> = vec![];
    let mut len = base;
    for c in codes {
        let add = "CodInst%20eq%20%27%27".len() + url_safe(c).len() + "%20or%20".len();
        if !cur.is_empty() && len + add > MAX_URL {
            out.push(std::mem::take(&mut cur));
            len = base;
        }
        cur.push(c.clone());
        len += add;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// ---------------------------------------------------------------------------
// Parse

/// Instituição do cadastro do IF.data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankInstitution {
    pub cod_inst: String,
    pub name: String,
    /// CNPJ (8 dígitos) da instituição líder.
    pub cnpj8: Option<String>,
    /// Tipo de consolidado bancário (B1, B2, B3S, B3C, B4, N1…).
    pub tcb: Option<String>,
    /// Segmento prudencial (S1 a S5); o Banco Central deixa vazio em alguns conglomerados.
    pub segment: Option<String>,
    /// Conglomerado prudencial (CodConglomeradoPrudencial).
    pub prudential_code: Option<String>,
    pub active: bool,
}

#[derive(Deserialize)]
struct OData<T> {
    value: Vec<T>,
}

#[derive(Deserialize)]
struct CadastroRow {
    #[serde(rename = "CodInst")]
    cod_inst: Option<String>,
    #[serde(rename = "NomeInstituicao")]
    name: Option<String>,
    #[serde(rename = "Tcb")]
    tcb: Option<String>,
    #[serde(rename = "Sr")]
    segment: Option<String>,
    #[serde(rename = "CodConglomeradoPrudencial")]
    prudential: Option<String>,
    #[serde(rename = "CnpjInstituicaoLider")]
    cnpj: Option<String>,
    #[serde(rename = "Situacao")]
    status: Option<String>,
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}

fn json_err(e: serde_json::Error) -> String {
    crate::i18n::t_args(
        "nucleo-pesquisa-erro-ifdata-invalida",
        &crate::i18n::args(&[("erro", e.to_string().into())]),
    )
}

/// Cadastro de um trimestre.
pub fn parse_cadastro(bytes: &[u8]) -> std::result::Result<Vec<BankInstitution>, String> {
    let d: OData<CadastroRow> = serde_json::from_slice(bytes).map_err(json_err)?;
    let mut out: HashMap<String, BankInstitution> = HashMap::new();
    for r in d.value {
        let Some(cod) = clean(r.cod_inst) else {
            continue;
        };
        let cnpj8 = clean(r.cnpj)
            .map(|c| super::cnpj_digits(&c))
            .filter(|c| c.len() == 8);
        out.insert(
            cod.clone(),
            BankInstitution {
                name: clean(r.name).unwrap_or_else(|| cod.clone()),
                cod_inst: cod,
                cnpj8,
                tcb: clean(r.tcb),
                segment: clean(r.segment),
                prudential_code: clean(r.prudential),
                active: r.status.as_deref().map(str::trim) == Some("A"),
            },
        );
    }
    let mut v: Vec<BankInstitution> = out.into_values().collect();
    v.sort_by(|a, b| a.cod_inst.cmp(&b.cod_inst));
    Ok(v)
}

/// O trimestre já saiu? (resposta com alguma linha.)
pub fn parse_probe(bytes: &[u8]) -> std::result::Result<bool, String> {
    let d: OData<serde_json::Value> = serde_json::from_slice(bytes).map_err(json_err)?;
    Ok(!d.value.is_empty())
}

#[derive(Deserialize)]
struct ValueRow {
    #[serde(rename = "CodInst")]
    cod_inst: Option<String>,
    #[serde(rename = "AnoMes")]
    ano_mes: Option<serde_json::Value>,
    #[serde(rename = "Conta")]
    conta: Option<serde_json::Value>,
    #[serde(rename = "Saldo")]
    saldo: Option<serde_json::Value>,
}

fn json_text(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.trim().to_string()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Valor de um item numa instituição e trimestre.
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub cod_inst: String,
    /// YYYY-MM.
    pub period: String,
    pub item: &'static str,
    pub value: Decimal,
}

/// Valores de um relatório (só as contas usadas; linhas repetidas valem uma vez).
pub fn parse_values(bytes: &[u8], report: &str) -> std::result::Result<Vec<Value>, String> {
    let d: OData<ValueRow> = serde_json::from_slice(bytes).map_err(json_err)?;
    // (instituição, período, conta) → valor; a API às vezes repete a mesma linha.
    let mut lines: HashMap<(String, String, String), Decimal> = HashMap::new();
    for r in d.value {
        let (Some(cod), Some(am), Some(conta), Some(saldo)) = (
            clean(r.cod_inst),
            r.ano_mes.as_ref().and_then(json_text),
            r.conta.as_ref().and_then(json_text),
            r.saldo.as_ref().and_then(json_text),
        ) else {
            continue;
        };
        if mapping(report, &conta).is_none() {
            continue;
        }
        let (Some(period), Some(v)) = (period_of(&am), super::parse_decimal(&saldo)) else {
            continue;
        };
        lines.insert((cod, period, conta), v);
    }
    let mut sums: HashMap<(String, String, &'static str), Decimal> = HashMap::new();
    for ((cod, period, conta), v) in lines {
        let (item, sign) = mapping(report, &conta).expect("filtrado acima");
        *sums.entry((cod, period, item)).or_default() += v * Decimal::from(sign);
    }
    let mut out: Vec<Value> = sums
        .into_iter()
        .map(|((cod_inst, period, item), v)| Value {
            value: if items::RATIOS.contains(&item) {
                v.round_dp(12).normalize()
            } else {
                v.round_dp(2).normalize()
            },
            cod_inst,
            period,
            item,
        })
        .collect();
    out.sort_by(|a, b| (&a.cod_inst, &a.period, a.item).cmp(&(&b.cod_inst, &b.period, b.item)));
    Ok(out)
}

// ---------------------------------------------------------------------------
// Gravação

/// Um relatório já baixado.
#[derive(Debug, Clone, PartialEq)]
pub struct Fetch {
    /// Instituições pedidas (`None` = relatório completo).
    pub codes: Option<Vec<String>>,
    pub fetched_at: String,
}

/// Relatórios já baixados: (período, relatório) → quando e para quem.
pub fn fetched(conn: &Connection) -> Result<HashMap<(String, String), Fetch>> {
    let mut stmt =
        conn.prepare("SELECT period, report, codes, fetched_at FROM bank_ifdata_fetch")?;
    let rows: Vec<(String, String, Option<String>, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .map(|(p, rep, codes, at)| {
            (
                (p, rep),
                Fetch {
                    codes: codes.and_then(|c| serde_json::from_str(&c).ok()),
                    fetched_at: at,
                },
            )
        })
        .collect())
}

/// Marca um relatório como baixado; nos filtrados, junta as instituições às já pedidas.
fn mark_fetched(
    conn: &Connection,
    period: &str,
    report: &str,
    codes: Option<&[String]>,
    rows: u64,
    now: &str,
) -> Result<()> {
    let codes_json = match codes {
        None => None,
        Some(new) => {
            let old: Option<Option<String>> = conn
                .query_row(
                    "SELECT codes FROM bank_ifdata_fetch WHERE period = ?1 AND report = ?2",
                    params![period, report],
                    |r| r.get(0),
                )
                .optional()?;
            let mut all: Vec<String> = old
                .flatten()
                .and_then(|c| serde_json::from_str::<Vec<String>>(&c).ok())
                .unwrap_or_default();
            all.extend(new.iter().cloned());
            all.sort();
            all.dedup();
            Some(serde_json::to_string(&all)?)
        }
    };
    conn.execute(
        "INSERT INTO bank_ifdata_fetch (period, report, codes, rows, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (period, report) DO UPDATE SET codes = excluded.codes,
             rows = CASE WHEN excluded.codes IS NULL THEN excluded.rows ELSE bank_ifdata_fetch.rows + excluded.rows END,
             fetched_at = excluded.fetched_at",
        params![period, report, codes_json, rows as i64, now],
    )?;
    Ok(())
}

/// Grava o cadastro do trimestre mais recente. Segmento, CNPJ e conglomerado vazios não
/// apagam o que já se sabia.
pub fn apply_cadastro(
    conn: &Connection,
    period: &str,
    rows: Vec<BankInstitution>,
    now: &str,
) -> Result<u64> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO bank_institution (cod_inst, name, cnpj8, tcb, segment, prudential_code, active, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (cod_inst) DO UPDATE SET name = excluded.name,
             cnpj8 = coalesce(excluded.cnpj8, bank_institution.cnpj8),
             tcb = coalesce(excluded.tcb, bank_institution.tcb),
             segment = coalesce(excluded.segment, bank_institution.segment),
             prudential_code = coalesce(excluded.prudential_code, bank_institution.prudential_code),
             active = excluded.active, fetched_at = excluded.fetched_at",
    )?;
    let mut n = 0;
    for i in &rows {
        n += stmt.execute(params![
            i.cod_inst,
            i.name,
            i.cnpj8,
            i.tcb,
            i.segment,
            i.prudential_code,
            i.active as i64,
            now
        ])? as u64;
    }
    mark_fetched(conn, period, CADASTRO, None, n, now)?;
    Ok(n)
}

/// Grava os valores de um relatório e marca o pedido como feito.
pub fn apply_values(
    conn: &Connection,
    period: &str,
    report: &str,
    codes: Option<&[String]>,
    vals: Vec<Value>,
    now: &str,
) -> Result<u64> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO bank_prudential (cod_inst, period, item, value, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (cod_inst, period, item) DO UPDATE SET value = excluded.value,
             source = excluded.source, fetched_at = excluded.fetched_at",
    )?;
    let mut n = 0;
    for v in &vals {
        n += stmt.execute(params![
            v.cod_inst,
            v.period,
            v.item,
            super::dstr(v.value),
            SOURCE,
            now
        ])? as u64;
    }
    mark_fetched(conn, period, report, codes, n, now)?;
    Ok(n)
}

/// Apaga trimestres anteriores a `oldest` (ficam os 5 mais recentes).
pub fn prune(conn: &Connection, oldest: &str) -> Result<()> {
    conn.execute("DELETE FROM bank_prudential WHERE period < ?1", [oldest])?;
    conn.execute("DELETE FROM bank_ifdata_fetch WHERE period < ?1", [oldest])?;
    Ok(())
}

/// CNPJs (8 dígitos) de interesse: bancos com ações na CVM (leiaute `bank`) e emissores de
/// renda fixa cadastrados com CNPJ ou identificados pelo nome.
pub fn target_cnpj8(conn: &Connection) -> Result<HashSet<String>> {
    let mut out = HashSet::new();
    let mut stmt = conn.prepare(
        "SELECT id FROM research_issuer WHERE kind = 'company_br' AND layout = 'bank'
         UNION SELECT cnpj FROM asset WHERE class = 'fixed_income' AND archived = 0 AND cnpj IS NOT NULL",
    )?;
    for r in stmt.query_map([], |r| r.get::<_, String>(0))? {
        let d = super::cnpj_digits(&r?);
        if d.len() >= 8 {
            out.insert(d[..8].to_string());
        }
    }
    // Emissores de renda fixa sem CNPJ, achados pelo nome no cadastro do IF.data.
    out.extend(crate::score::fixed_income::matched_bank_cnpj8(conn)?);
    Ok(out)
}

/// Instituições pedidas nos relatórios filtrados (2 e 4) de um trimestre: segmentos S1 a S3,
/// conglomerados de bancos sem segmento informado (o Banco Central deixa o do Bradesco vazio) e
/// as instituições dos CNPJs de interesse. Sempre o código que reporta (o conglomerado
/// prudencial, ou a própria instituição independente) e, quando o resumo do trimestre já foi
/// gravado, só os códigos que aparecem nele.
pub fn target_codes(
    conn: &Connection,
    cnpj8: &HashSet<String>,
    period: &str,
) -> Result<Vec<String>> {
    let all = institutions(conn)?;
    let reported: HashSet<String> = conn
        .prepare_cached("SELECT DISTINCT cod_inst FROM bank_prudential WHERE period = ?1 AND item = 'total_assets'")?
        .query_map([period], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out: Vec<String> = all
        .iter()
        .filter(|i| {
            let seg = i.segment.as_deref();
            matches!(seg, Some("S1" | "S2" | "S3"))
                || (seg.is_none() && is_bank(i) && i.cod_inst.starts_with('C'))
                || i.cnpj8.as_ref().is_some_and(|c| cnpj8.contains(c))
                || cnpj8.contains(&i.cod_inst)
        })
        .map(|i| {
            i.prudential_code
                .clone()
                .unwrap_or_else(|| i.cod_inst.clone())
        })
        .filter(|c| reported.is_empty() || reported.contains(c))
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

// ---------------------------------------------------------------------------
// Leitura

fn row_to_institution(r: &rusqlite::Row<'_>) -> rusqlite::Result<BankInstitution> {
    Ok(BankInstitution {
        cod_inst: r.get("cod_inst")?,
        name: r.get("name")?,
        cnpj8: r.get("cnpj8")?,
        tcb: r.get("tcb")?,
        segment: r.get("segment")?,
        prudential_code: r.get("prudential_code")?,
        active: r.get::<_, i64>("active")? == 1,
    })
}

fn institutions(conn: &Connection) -> Result<Vec<BankInstitution>> {
    let mut stmt = conn.prepare_cached("SELECT * FROM bank_institution ORDER BY cod_inst")?;
    let rows = stmt
        .query_map([], row_to_institution)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn institution(conn: &Connection, cod_inst: &str) -> Result<Option<BankInstitution>> {
    Ok(conn
        .prepare_cached("SELECT * FROM bank_institution WHERE cod_inst = ?1")?
        .query_row([cod_inst], row_to_institution)
        .optional()?)
}

/// Banco pelo tipo de consolidado (B1, B2, B3S, B3C, B4).
fn is_bank(i: &BankInstitution) -> bool {
    i.tcb
        .as_deref()
        .is_some_and(|t| t.to_ascii_lowercase().starts_with('b'))
}

/// Instituição do IF.data de um CNPJ (8 primeiros dígitos): o conglomerado prudencial da
/// instituição líder quando há; senão a própria instituição (independente, cujo código é o
/// CNPJ).
pub fn institution_for_cnpj8(conn: &Connection, cnpj8: &str) -> Result<Option<BankInstitution>> {
    let cnpj8: String = super::cnpj_digits(cnpj8).chars().take(8).collect();
    if cnpj8.len() != 8 {
        return Ok(None);
    }
    let mut stmt = conn.prepare_cached(
        "SELECT * FROM bank_institution WHERE cnpj8 = ?1 ORDER BY active DESC, cod_inst",
    )?;
    let rows: Vec<BankInstitution> = stmt
        .query_map([&cnpj8], row_to_institution)?
        .collect::<rusqlite::Result<_>>()?;
    for r in &rows {
        if let Some(p) = &r.prudential_code {
            if p == &r.cod_inst {
                return Ok(Some(r.clone()));
            }
            if let Some(c) = institution(conn, p)? {
                return Ok(Some(c));
            }
        }
    }
    if let Some(r) = rows.into_iter().next() {
        return Ok(Some(r));
    }
    institution(conn, &cnpj8)
}

/// Por que um banco (CNPJ da companhia aberta) está sem indicadores do IF.data: fonte
/// desligada, nada baixado ainda, CNPJ fora do cadastro ou instituição sem valores.
pub fn diagnose(conn: &Connection, cnpj: &str) -> Result<String> {
    let cnpj8: String = super::cnpj_digits(cnpj).chars().take(8).collect();
    if !crate::sources::enabled(conn, crate::sources::SourceId::Bcb) {
        return Ok(crate::i18n::t("nucleo-pesquisa-ifdata-fonte-desligada"));
    }
    let downloaded: Option<String> = conn
        .query_row(
            "SELECT max(period) FROM bank_prudential WHERE item = 'basel_ratio'",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    let Some(period) = downloaded else {
        return Ok(crate::i18n::t("nucleo-pesquisa-ifdata-nao-baixado"));
    };
    match institution_for_cnpj8(conn, &cnpj8)? {
        None => Ok(crate::i18n::t_args(
            "nucleo-pesquisa-ifdata-fora-do-cadastro",
            &crate::i18n::args(&[("cnpj", cnpj8.into()), ("trimestre", period.into())]),
        )),
        Some(i) => Ok(crate::i18n::t_args(
            "nucleo-pesquisa-ifdata-sem-valores",
            &crate::i18n::args(&[
                ("banco", i.name.into()),
                ("codigo", i.cod_inst.into()),
                ("trimestre", period.into()),
            ]),
        )),
    }
}

/// Valores por trimestre (YYYY-MM) de uma instituição.
pub fn series(
    conn: &Connection,
    cod_inst: &str,
) -> Result<BTreeMap<String, HashMap<String, Decimal>>> {
    let mut stmt = conn.prepare_cached(
        "SELECT period, item, value FROM bank_prudential WHERE cod_inst = ?1 ORDER BY period",
    )?;
    let mut out: BTreeMap<String, HashMap<String, Decimal>> = BTreeMap::new();
    for r in stmt.query_map([cod_inst], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (p, item, v) = r?;
        if let Ok(v) = v.parse::<Decimal>() {
            out.entry(p).or_default().insert(item, v);
        }
    }
    Ok(out)
}

/// Bancos comparáveis num trimestre: tipo de consolidado bancário (B…) e segmento S1 a S4 (ou
/// conglomerado sem segmento informado), com os valores do trimestre.
pub fn peers(
    conn: &Connection,
    period: &str,
) -> Result<Vec<(BankInstitution, HashMap<String, Decimal>)>> {
    let mut vals: HashMap<String, HashMap<String, Decimal>> = HashMap::new();
    let mut stmt =
        conn.prepare_cached("SELECT cod_inst, item, value FROM bank_prudential WHERE period = ?1")?;
    for r in stmt.query_map([period], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (c, item, v) = r?;
        if let Ok(v) = v.parse::<Decimal>() {
            vals.entry(c).or_default().insert(item, v);
        }
    }
    let mut out = vec![];
    for i in institutions(conn)? {
        let seg_ok = match i.segment.as_deref() {
            Some(s) => matches!(s, "S1" | "S2" | "S3" | "S4"),
            None => i.cod_inst.starts_with('C'),
        };
        if !is_bank(&i) || !seg_ok {
            continue;
        }
        if let Some(v) = vals.remove(&i.cod_inst) {
            out.push((i, v));
        }
    }
    Ok(out)
}

/// Resultado dos últimos 12 meses terminados em `period`, a partir dos valores do semestre até
/// a data: junho/dezembro = semestre + semestre anterior; março/setembro = trimestre + semestre
/// anterior + (semestre de um ano antes − trimestre de um ano antes). `None` se faltar algum.
pub fn twelve_months(
    series: &BTreeMap<String, HashMap<String, Decimal>>,
    period: &str,
    item: &str,
) -> Option<Decimal> {
    let v = |p: Option<String>| p.and_then(|p| series.get(&p)?.get(item).copied());
    let month = period_date(period)?.month();
    match month {
        6 | 12 => Some(v(Some(period.to_string()))? + v(shift(period, -6))?),
        3 | 9 => Some(
            v(Some(period.to_string()))? + v(shift(period, -3))? + v(shift(period, -9))?
                - v(shift(period, -12))?,
        ),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    //! Made-up replies in the real IF.data format (OData JSON).

    /// Registry: Banco Exemplo (leader 22333444, conglomerate C0080001, S2) and its institution,
    /// a large bank with no segment, an independent cooperative and a finance company.
    pub fn cadastro(period: &str) -> Vec<u8> {
        let am = period.replace('-', "");
        format!(
            r#"{{"@odata.context":"x","value":[
            {{"CodInst":"C0010001","Data":"{am}","NomeInstituicao":"BANCO EXEMPLO","Tcb":"B1","Sr":"S2","CodConglomeradoPrudencial":"C0080001","CnpjInstituicaoLider":"22333444","Situacao":"A"}},
            {{"CodInst":"C0080001","Data":"{am}","NomeInstituicao":"BANCO EXEMPLO - PRUDENCIAL","Tcb":"B1","Sr":"S2","CodConglomeradoPrudencial":"C0080001","CnpjInstituicaoLider":"22333444","Situacao":"A"}},
            {{"CodInst":"C0080002","Data":"{am}","NomeInstituicao":"GRANDE - PRUDENCIAL","Tcb":"B1","Sr":null,"CodConglomeradoPrudencial":"C0080002","CnpjInstituicaoLider":"55666777","Situacao":"A"}},
            {{"CodInst":"12345678","Data":"{am}","NomeInstituicao":"COOPERATIVA","Tcb":"B3C","Sr":"S5","CodConglomeradoPrudencial":null,"CnpjInstituicaoLider":null,"Situacao":"A"}},
            {{"CodInst":"C0080003","Data":"{am}","NomeInstituicao":"FINANCEIRA - PRUDENCIAL","Tcb":"N1","Sr":"S3","CodConglomeradoPrudencial":"C0080003","CnpjInstituicaoLider":"88999000","Situacao":"A"}}
            ]}}"#
        )
        .into_bytes()
    }

    fn row(cod: &str, am: &str, report: &str, conta: &str, saldo: &str) -> String {
        format!(
            r#"{{"TipoInstituicao":1,"CodInst":"{cod}","AnoMes":"{am}","NomeRelatorio":"x","NumeroRelatorio":"{report}","Grupo":null,"Conta":"{conta}","NomeColuna":"x","DescricaoColuna":"x","Saldo":{saldo}}}"#
        )
    }

    /// Values of a report: `rows` = (CodInst, account, balance).
    pub fn values(period: &str, report: &str, rows: &[(&str, &str, &str)]) -> Vec<u8> {
        let am = period.replace('-', "");
        let v: Vec<String> = rows
            .iter()
            .map(|(c, conta, s)| row(c, &am, report, conta, s))
            .collect();
        format!(r#"{{"@odata.context":"x","value":[{}]}}"#, v.join(",")).into_bytes()
    }

    pub fn empty() -> Vec<u8> {
        br#"{"@odata.context":"x","value":[]}"#.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn periods_and_urls() {
        let d = |s| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
        assert_eq!(quarter_end(d("2026-10-16")), "2026-09");
        assert_eq!(quarter_end(d("2026-09-30")), "2026-09");
        assert_eq!(quarter_end(d("2026-02-10")), "2025-12");
        assert_eq!(quarter_end(d("2026-03-01")), "2026-03");
        assert_eq!(
            quarters_back("2026-06", 5),
            vec!["2026-06", "2026-03", "2025-12", "2025-09", "2025-06"]
        );
        assert_eq!(
            values_url("2026-06", "1", Some(&["C0080099".into(), "x'or 1".into()])),
            format!(
                "{BASE}/IfDataValores(AnoMes=@AnoMes,TipoInstituicao=@TipoInstituicao,Relatorio=@Relatorio)?@AnoMes=202606&@TipoInstituicao=1&@Relatorio=%271%27&$format=json&$filter=CodInst%20eq%20%27C0080099%27%20or%20CodInst%20eq%20%27xor1%27"
            )
        );
        assert!(cadastro_url("2026-06", true).ends_with("@AnoMes=202606&$format=json&$top=1"));
        let codes: Vec<String> = (0..400).map(|i| format!("C00{i:05}")).collect();
        let chunks = chunk_codes("2026-06", "4", &codes);
        assert!(chunks.len() > 1);
        assert_eq!(chunks.iter().map(Vec::len).sum::<usize>(), 400);
        for c in &chunks {
            assert!(values_url("2026-06", "4", Some(c)).len() <= MAX_URL);
        }
    }

    #[test]
    fn parses_cadastro_and_values() {
        let cad = parse_cadastro(&cadastro("2026-06")).unwrap();
        assert_eq!(cad.len(), 5);
        let ex = cad.iter().find(|i| i.cod_inst == "C0080001").unwrap();
        assert_eq!(
            (ex.cnpj8.as_deref(), ex.segment.as_deref(), ex.active),
            (Some("22333444"), Some("S2"), true)
        );
        assert!(parse_probe(&empty()).is_ok_and(|x| !x));
        assert!(parse_probe(&cadastro("2026-06")).unwrap());
        assert!(parse_cadastro(b"<html>erro</html>").is_err());

        // Repeated row counts once; accounts summed into the same item; sign of expenses.
        let r2 = values(
            "2026-06",
            "2",
            &[
                ("C0080001", "140202", "-100.5"),
                ("C0080001", "140202", "-100.5"),
                ("C0080001", "140208", "-10"),
                ("C0080001", "145832", "-2"),
                ("C0080001", "140201", "5000"),
            ],
        );
        let v = parse_values(&r2, "2").unwrap();
        assert_eq!(
            v,
            vec![Value {
                cod_inst: "C0080001".into(),
                period: "2026-06".into(),
                item: items::EXPECTED_LOSS,
                value: dec!(112.5),
            }]
        );
        let r4 = values(
            "2026-06",
            "4",
            &[
                ("C0080001", "141842", "-30"),
                ("C0080001", "141856", "7"),
                ("C0080001", "141857", "3"),
                ("C0080001", "141858", "-12"),
                ("C0080001", "141859", "-8"),
                ("C0080001", "141851", "90"),
            ],
        );
        let v: HashMap<&str, Decimal> = parse_values(&r4, "4")
            .unwrap()
            .into_iter()
            .map(|x| (x.item, x.value))
            .collect();
        assert_eq!(v[items::PROVISION_EXPENSE], dec!(30));
        assert_eq!(v[items::FEE_INCOME], dec!(10));
        assert_eq!(v[items::PERSONNEL_EXPENSE], dec!(12));
        assert_eq!(v[items::ADMIN_EXPENSE], dec!(8));
        assert_eq!(v[items::INTERMEDIATION_RESULT], dec!(90));
        let r5 = values("2026-06", "5", &[("C0080001", "79659", "1.23e-1")]);
        assert_eq!(parse_values(&r5, "5").unwrap()[0].value, dec!(0.123));
    }

    /// On the network: the June 2026 quarter of Itaú prudential (`cargo test -- --ignored`).
    #[test]
    #[ignore]
    fn live_ifdata() {
        use crate::research::net::{Fetched, Fetcher, HttpFetcher};
        let f = HttpFetcher::new();
        let get = |url: &str| match f.get(url, None, "").unwrap() {
            Fetched::Body { bytes, .. } => bytes,
            other => panic!("{url}: {other:?}"),
        };
        assert!(parse_probe(&get(&cadastro_url("2026-06", true))).unwrap());
        let codes = ["C0080099".to_string()];
        for rep in ["1", "4"] {
            let v = parse_values(&get(&values_url("2026-06", rep, Some(&codes))), rep).unwrap();
            println!("{rep}: {v:?}");
            assert!(!v.is_empty());
        }
    }

    #[test]
    fn twelve_months_from_semester_to_date() {
        let mut s: BTreeMap<String, HashMap<String, Decimal>> = BTreeMap::new();
        for (p, v) in [
            ("2025-03", 11),
            ("2025-06", 22),
            ("2025-09", 12),
            ("2025-12", 24),
            ("2026-03", 13),
            ("2026-06", 26),
        ] {
            s.entry(p.into())
                .or_default()
                .insert(items::NET_INCOME.into(), Decimal::from(v));
        }
        let ltm = |p| twelve_months(&s, p, items::NET_INCOME);
        // December: 2nd half + 1st half.
        assert_eq!(ltm("2025-12"), Some(dec!(46)));
        assert_eq!(ltm("2026-06"), Some(dec!(50)));
        // Mar/26: 1st qtr/26 + 2nd half/25 + (1st half/25 − 1st qtr/25) = 13 + 24 + 22 − 11.
        assert_eq!(ltm("2026-03"), Some(dec!(48)));
        // Sep/25 without Dec/24 and Sep/24.
        assert_eq!(ltm("2025-09"), None);
    }
}
