//! Cotas de fundos de investimento pelo informe diário da CVM (dados abertos).
//!
//! Arquivos mensais (`inf_diario_fi_AAAAMM.zip`) a partir de 2021 e anuais (`HIST/`) antes
//! disso, cada um com todos os fundos do país. Baixamos um arquivo por período e filtramos
//! pelos CNPJs da carteira, numa passada só, sem guardar o CSV inteiro em memória.
//!
//! Da mesma passada saem, por mês, o patrimônio líquido e o número de cotistas do último dia
//! e a captação líquida (aplicações − resgates) do mês, usados no score dos fundos.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Read, Seek};
use std::time::Duration;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

const BASE: &str = "https://dados.cvm.gov.br/dados/FI/DOC/INF_DIARIO/DADOS";
/// Primeiro mês publicado em arquivo mensal; antes disso, só os anuais em `HIST/`.
const FIRST_MONTHLY: (i32, u32) = (2021, 1);

/// CNPJ só com dígitos → pontos (data, valor da cota).
pub type Quotes = HashMap<String, Vec<(NaiveDate, Decimal)>>;

/// Resumo de um mês do informe diário de um fundo.
#[derive(Debug, Clone, PartialEq)]
pub struct FundMonth {
    /// Último dia do mês com informe.
    pub last: NaiveDate,
    /// Patrimônio líquido e cotistas no último dia.
    pub nav: Option<Decimal>,
    pub holders: Option<i64>,
    /// Soma das aplicações e dos resgates do mês.
    pub inflow: Decimal,
    pub outflow: Decimal,
    pub days: u32,
}

/// CNPJ só com dígitos → (ano, mês) → resumo.
pub type Months = HashMap<String, BTreeMap<(i32, u32), FundMonth>>;

pub fn cnpj_digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}

/// Arquivos que cobrem [from, to], do mais antigo ao mais recente.
pub fn files_for(from: NaiveDate, to: NaiveDate) -> Vec<String> {
    let mut out = vec![];
    let mut year = from.year();
    while year < FIRST_MONTHLY.0 && year <= to.year() {
        out.push(format!("{BASE}/HIST/inf_diario_fi_{year}.zip"));
        year += 1;
    }
    let (mut y, mut m) = if from.year() < FIRST_MONTHLY.0 {
        FIRST_MONTHLY
    } else {
        (from.year(), from.month())
    };
    while (y, m) <= (to.year(), to.month()) {
        out.push(format!("{BASE}/inf_diario_fi_{y}{m:02}.zip"));
        (y, m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    }
    out
}

/// Lê um CSV do informe diário (separador `;`) e guarda as cotas dos CNPJs pedidos.
/// Aceita o layout antigo (`CNPJ_FUNDO`) e o da resolução 175 (`CNPJ_FUNDO_CLASSE`,
/// `ID_SUBCLASSE`): só a cota da classe (sem subclasse) é usada.
pub fn parse_into(
    reader: impl Read,
    wanted: &HashSet<String>,
    since: NaiveDate,
    out: &mut Quotes,
) -> Result<(), String> {
    parse_full(reader, wanted, since, out, None)
}

/// Como [`parse_into`]; com `months`, resume também o mês de cada linha (todas as datas do
/// arquivo, sem o corte de `since`: os meses do arquivo vêm completos).
pub fn parse_full(
    reader: impl Read,
    wanted: &HashSet<String>,
    since: NaiveDate,
    out: &mut Quotes,
    mut months: Option<&mut Months>,
) -> Result<(), String> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(reader);
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    let col = |names: &[&str]| headers.iter().position(|h| names.contains(&h.trim()));
    let (Some(c_cnpj), Some(c_date), Some(c_quota)) = (
        col(&["CNPJ_FUNDO_CLASSE", "CNPJ_FUNDO"]),
        col(&["DT_COMPTC"]),
        col(&["VL_QUOTA"]),
    ) else {
        return Err(crate::i18n::t("nucleo-ativos-erro-cvm-layout"));
    };
    let c_sub = col(&["ID_SUBCLASSE"]);
    let (c_nav, c_holders, c_in, c_out) = (
        col(&["VL_PATRIM_LIQ"]),
        col(&["NR_COTST"]),
        col(&["CAPTC_DIA"]),
        col(&["RESG_DIA"]),
    );
    let mut rec = csv::ByteRecord::new();
    while rdr.read_byte_record(&mut rec).map_err(|e| e.to_string())? {
        let field = |i: usize| rec.get(i).map(|b| String::from_utf8_lossy(b));
        let Some(cnpj) = field(c_cnpj) else { continue };
        let digits = cnpj_digits(&cnpj);
        if !wanted.contains(&digits) {
            continue;
        }
        if c_sub.and_then(field).is_some_and(|s| !s.trim().is_empty()) {
            continue;
        }
        let (Some(d), Some(q)) = (field(c_date), field(c_quota)) else {
            continue;
        };
        let (Ok(date), Ok(quota)) = (
            NaiveDate::parse_from_str(d.trim(), "%Y-%m-%d"),
            q.trim().parse::<Decimal>(),
        ) else {
            continue;
        };
        if let Some(m) = months.as_deref_mut() {
            let dec = |c: Option<usize>| {
                c.and_then(field)
                    .and_then(|v| v.trim().parse::<Decimal>().ok())
            };
            let e = m
                .entry(digits.clone())
                .or_default()
                .entry((date.year(), date.month()))
                .or_insert(FundMonth {
                    last: date,
                    nav: None,
                    holders: None,
                    inflow: Decimal::ZERO,
                    outflow: Decimal::ZERO,
                    days: 0,
                });
            e.inflow += dec(c_in).unwrap_or_default();
            e.outflow += dec(c_out).unwrap_or_default();
            e.days += 1;
            if date >= e.last || e.nav.is_none() {
                e.last = date;
                e.nav = dec(c_nav).or(e.nav);
                // Cotistas zerados aparecem em informes recentes ainda não consolidados.
                e.holders = c_holders
                    .and_then(field)
                    .and_then(|v| v.trim().parse::<i64>().ok())
                    .filter(|h| *h > 0)
                    .or(e.holders);
            }
        }
        if date < since || quota <= Decimal::ZERO {
            continue;
        }
        out.entry(digits).or_default().push((date, quota));
    }
    Ok(())
}

/// Lê todos os CSVs de um zip baixado.
pub fn parse_zip(
    zip: impl Read + Seek,
    wanted: &HashSet<String>,
    since: NaiveDate,
    out: &mut Quotes,
) -> Result<(), String> {
    parse_zip_full(zip, wanted, since, out, None)
}

pub fn parse_zip_full(
    zip: impl Read + Seek,
    wanted: &HashSet<String>,
    since: NaiveDate,
    out: &mut Quotes,
    mut months: Option<&mut Months>,
) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(zip).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().map_err(|e| e.to_string())?.to_ascii_lowercase();
        if name.ends_with(".csv") {
            parse_full(file, wanted, since, out, months.as_deref_mut())?;
        }
    }
    Ok(())
}

fn finish(out: &mut Quotes) {
    for v in out.values_mut() {
        v.sort_by_key(|(d, _)| *d);
        v.dedup_by_key(|(d, _)| *d);
    }
}

/// Baixa e filtra os arquivos que cobrem [since, hoje]. Um mês ainda não publicado
/// (404 no mês corrente logo na virada) é ignorado.
pub fn fetch(cnpjs: &[String], since: NaiveDate) -> Result<Quotes, String> {
    fetch_full(cnpjs, since).map(|(q, _, _)| q)
}

/// Como [`fetch`], com o resumo mensal (patrimônio, cotistas e captação) dos mesmos fundos.
/// Os arquivos vão do mais antigo ao mais novo; o servidor da CVM às vezes demora ou cai no
/// meio de uma coleta longa (o histórico de um plano de previdência desde a adesão), então um
/// arquivo que falha (depois de uma nova tentativa) encerra a coleta com o que já veio, e o
/// erro vai no terceiro valor: o que veio é contínuo desde `since`, e a próxima coleta
/// continua da última cota gravada. Erro só quando nem o primeiro arquivo veio.
pub fn fetch_full(
    cnpjs: &[String],
    since: NaiveDate,
) -> Result<(Quotes, Months, Option<String>), String> {
    let wanted: HashSet<String> = cnpjs.iter().map(|c| cnpj_digits(c)).collect();
    let mut out = Quotes::new();
    let mut months = Months::new();
    if wanted.is_empty() {
        return Ok((out, months, None));
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(600)))
        .http_status_as_error(false)
        .build()
        .into();
    let today = crate::time::today_local();
    let files = files_for(since, today);
    let last = files.len().saturating_sub(1);
    let download = |url: &str| -> Result<Option<Vec<u8>>, String> {
        let mut resp = agent
            .get(url)
            .header(
                "User-Agent",
                "Treasury/0.1 (app local de financas pessoais)",
            )
            .call()
            .map_err(|e| format!("CVM: {e}"))?;
        let status = resp.status().as_u16();
        if status == 404 {
            return Ok(None);
        }
        if status != 200 {
            return Err(crate::i18n::t_args(
                "nucleo-ativos-erro-cvm-status",
                &crate::i18n::args(&[
                    ("status", status.to_string().into()),
                    ("url", url.to_string().into()),
                ]),
            ));
        }
        resp.body_mut()
            .with_config()
            .limit(512 * 1024 * 1024)
            .read_to_vec()
            .map(Some)
            .map_err(|e| format!("CVM: {e}"))
    };
    // Tarefa em segundo plano (docs/background-tasks.md): um passo por arquivo.
    crate::tasks::stage(crate::tasks::TaskStage::Funds);
    crate::tasks::set_total(Some(files.len() as u64));
    for (i, url) in files.iter().enumerate() {
        // Cancelado: fica o que veio, sem erro de fonte.
        if crate::tasks::cancelled() {
            finish(&mut out);
            return Ok((out, months, Some(crate::error::Error::Cancelled.message())));
        }
        crate::tasks::detail(file_label(url));
        crate::tasks::advance();
        // Uma nova tentativa antes de desistir do arquivo.
        let got = download(url).or_else(|_| download(url));
        let bytes = match got {
            Ok(Some(b)) => b,
            // Mês corrente ainda não publicado (virada do mês).
            Ok(None) if i == last => continue,
            Ok(None) => {
                let e = crate::i18n::t_args(
                    "nucleo-ativos-erro-cvm-status",
                    &crate::i18n::args(&[
                        ("status", "404".to_string().into()),
                        ("url", url.clone().into()),
                    ]),
                );
                return stop(i, e, out, months);
            }
            Err(e) => return stop(i, e, out, months),
        };
        if let Err(e) = parse_zip_full(
            std::io::Cursor::new(bytes),
            &wanted,
            since,
            &mut out,
            Some(&mut months),
        ) {
            return stop(i, e, out, months);
        }
    }
    finish(&mut out);
    Ok((out, months, None))
}

/// "CVM 2024-05" (mensal) ou "CVM 2019" (anual) a partir do endereço do arquivo.
fn file_label(url: &str) -> String {
    let name = url.rsplit('/').next().unwrap_or(url);
    let digits: String = name.chars().filter(char::is_ascii_digit).collect();
    match digits.len() {
        6 => format!("CVM {}-{}", &digits[..4], &digits[4..]),
        4 => format!("CVM {digits}"),
        _ => format!("CVM {name}"),
    }
}

/// Coleta interrompida no arquivo `i`: o que veio antes dele, com o erro; nada, se foi o
/// primeiro.
fn stop(
    i: usize,
    e: String,
    mut out: Quotes,
    months: Months,
) -> Result<(Quotes, Months, Option<String>), String> {
    if i == 0 {
        return Err(e);
    }
    finish(&mut out);
    Ok((out, months, Some(e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_file_keeps_what_came_before_it() {
        let mut q = Quotes::new();
        q.insert(
            "1".into(),
            vec![(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), Decimal::ONE)],
        );
        // No meio da coleta: o que veio fica, com o erro.
        let (got, _, err) = stop(3, "CVM: timeout".into(), q.clone(), Months::new()).unwrap();
        assert_eq!(got["1"].len(), 1);
        assert_eq!(err.as_deref(), Some("CVM: timeout"));
        // Logo no primeiro arquivo: erro.
        assert!(stop(0, "CVM: timeout".into(), q, Months::new()).is_err());
    }
    use rust_decimal_macros::dec;
    use std::io::Write;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn files_cover_annual_history_then_months() {
        let f = files_for(d("2019-11-20"), d("2021-02-03"));
        assert_eq!(
            f,
            vec![
                format!("{BASE}/HIST/inf_diario_fi_2019.zip"),
                format!("{BASE}/HIST/inf_diario_fi_2020.zip"),
                format!("{BASE}/inf_diario_fi_202101.zip"),
                format!("{BASE}/inf_diario_fi_202102.zip"),
            ]
        );
        assert_eq!(files_for(d("2026-12-31"), d("2027-01-02")).len(), 2);
    }

    #[test]
    fn parses_both_layouts_and_skips_subclasses() {
        let wanted: HashSet<String> =
            ["00017024000153".to_string(), "11111111000111".into()].into();
        let new = "TP_FUNDO_CLASSE;CNPJ_FUNDO_CLASSE;ID_SUBCLASSE;DT_COMPTC;VL_TOTAL;VL_QUOTA;VL_PATRIM_LIQ;CAPTC_DIA;RESG_DIA;NR_COTST\n\
            CLASSES - FIF;00.017.024/0001-53;;2026-09-01;1107741.39;44.636723400000;1214792.63;0.00;0.00;1\n\
            CLASSES - FIF;00.017.024/0001-53;SUB1;2026-09-01;1;99.0;1;0;0;1\n\
            CLASSES - FIF;00.017.024/0001-53;;2026-09-02;1108302.04;44.655755500000;1215310.59;0.00;0.00;1\n\
            CLASSES - FIF;22.222.222/0001-22;;2026-09-02;1;10.0;1;0;0;1\n";
        let old = "TP_FUNDO;CNPJ_FUNDO;DT_COMPTC;VL_TOTAL;VL_QUOTA;VL_PATRIM_LIQ;CAPTC_DIA;RESG_DIA;NR_COTST\n\
            FI;11.111.111/0001-11;2020-12-30;1;1.5;1;0;0;1\n\
            FI;11.111.111/0001-11;2019-01-02;1;1.0;1;0;0;1\n";
        // O mesmo arquivo dentro de um zip, como a CVM publica.
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            z.start_file("inf_diario_fi_202609.csv", opts).unwrap();
            z.write_all(new.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf.set_position(0);
        let mut out = Quotes::new();
        parse_zip(buf, &wanted, d("2020-01-01"), &mut out).unwrap();
        parse_into(old.as_bytes(), &wanted, d("2020-01-01"), &mut out).unwrap();
        finish(&mut out);
        assert_eq!(
            out["00017024000153"],
            vec![
                (d("2026-09-01"), dec!(44.636723400000)),
                (d("2026-09-02"), dec!(44.655755500000))
            ]
        );
        assert_eq!(out["11111111000111"], vec![(d("2020-12-30"), dec!(1.5))]);
        assert!(!out.contains_key("22222222000122"));
        assert!(parse_into("A;B\n1;2\n".as_bytes(), &wanted, d("2020-01-01"), &mut out).is_err());
        // Resumo do mês: patrimônio e cotistas do último dia, captação somada, sem o corte
        // de `since` e sem as subclasses.
        let mut months = Months::new();
        let mut q = Quotes::new();
        let month = "TP_FUNDO_CLASSE;CNPJ_FUNDO_CLASSE;ID_SUBCLASSE;DT_COMPTC;VL_TOTAL;VL_QUOTA;VL_PATRIM_LIQ;CAPTC_DIA;RESG_DIA;NR_COTST\n\
            CLASSES - FIF;00.017.024/0001-53;;2026-09-02;1;44.6;1215310.59;100.00;10.50;3\n\
            CLASSES - FIF;00.017.024/0001-53;;2026-09-01;1;44.5;1214792.63;0.00;5.00;2\n\
            CLASSES - FIF;00.017.024/0001-53;SUB1;2026-09-03;1;99.0;9;999;0;9\n\
            CLASSES - FIF;00.017.024/0001-53;;2026-10-01;1;44.7;1300000;0;0;4\n\
            CLASSES - FIF;00.017.024/0001-53;;2026-10-02;1;44.8;1300001;0;0;0\n";
        parse_full(
            month.as_bytes(),
            &wanted,
            d("2026-09-02"),
            &mut q,
            Some(&mut months),
        )
        .unwrap();
        let m = &months["00017024000153"];
        let sep = &m[&(2026, 9)];
        assert_eq!(sep.last, d("2026-09-02"));
        assert_eq!(sep.nav, Some(dec!(1215310.59)));
        assert_eq!(sep.holders, Some(3));
        assert_eq!(sep.inflow, dec!(100.00));
        assert_eq!(sep.outflow, dec!(15.50));
        assert_eq!(sep.days, 2);
        // Cotistas zerados no último dia: vale o último informado.
        assert_eq!(m[&(2026, 10)].holders, Some(4));
        assert_eq!(m[&(2026, 10)].nav, Some(dec!(1300001)));
        assert_eq!(q["00017024000153"].len(), 3);
    }

    #[test]
    #[ignore = "rede: baixa o informe diário da CVM"]
    fn live_cvm_month() {
        let since = crate::time::add_months(crate::time::today_local(), -1);
        let q = fetch(&["00.017.024/0001-53".into()], since).unwrap();
        assert!(q.values().next().is_some_and(|v| !v.is_empty()), "{q:?}");
    }
}
