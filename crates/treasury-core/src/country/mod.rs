//! País onde a família mora (`household.country`, V123; docs/countries.md). O app se adapta a
//! ele: quais fontes de dados ficam ligadas, a inflação e o juro de referência, a folha de
//! pagamento (descontos e pagamentos extras), as categorias e regras prontas de um cofre novo,
//! as telas e as dicas. O Brasil continua completo; Estados Unidos, Portugal e Espanha têm
//! adaptação própria (docs/international.md); os outros países do catálogo de índices ficam no modo genérico, com a inflação, os
//! juros e o câmbio do catálogo e sem as fontes e telas que só valem para o Brasil.
//!
//! Nomes dos países, termos da folha, dicas e limites saem do catálogo
//! (`locales/*/nucleo-pais.ftl`) no idioma da instalação.

use std::collections::BTreeMap;

use rusqlite::{Connection, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{Locale, args, t, t_args};
use crate::sources::SourceId;

pub mod catalog;

/// Ajuste técnico: fontes desligadas pelo app por causa do país (voltam a ligar se o país
/// mudar e elas voltarem a fazer sentido). As que o usuário desligou à mão não entram.
pub const SOURCES_OFF_SETTING: &str = "country.sources_off";

/// Países com adaptação própria, na ordem da escolha.
pub const FEATURED: [&str; 10] = ["BR", "US", "PT", "ES", "GB", "CA", "FR", "DE", "AR", "CL"];

/// Adaptados na rodada 2 (docs/international.md): folha, recibo, categorias e regras prontas
/// pelo catálogo (`nucleo-pais-<código>-*`), inflação e juro pelos nomes do catálogo de índices.
const ADAPTED_2: [&str; 6] = ["GB", "CA", "FR", "DE", "AR", "CL"];

/// Quanto o app conhece do país.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CountrySupport {
    /// Tudo do app (Brasil).
    Full,
    /// Moeda, índices, folha, impostos, categorias e regras do país.
    Adapted,
    /// Moeda, inflação, juros e câmbio do catálogo; sem regras locais.
    Generic,
}

/// O que liga e desliga nas telas e no núcleo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CountryFeatures {
    /// Mercado brasileiro: B3 (COTAHIST, notas de corretagem, extratos), Tesouro Direto,
    /// fundos da CVM, Fundamentus e proventos anunciados.
    pub br_market: bool,
    /// IR do Brasil sobre investimentos: apuração mensal, DARF e declaração.
    pub br_income_tax: bool,
    /// Juros de mercado do Banco Central por modalidade e portabilidade de crédito.
    pub br_credit_rates: bool,
    /// Tabela FIPE para veículos.
    pub fipe: bool,
    /// FGTS no holerite e na rescisão.
    pub fgts: bool,
    /// Estimativa dos impostos pagos no ano (há dados do país).
    pub tax_burden: bool,
    /// Descontos da folha calculados pelas regras do país.
    pub payroll: bool,
    /// Cupons fiscais (NFC-e) item a item, com o imposto da nota (docs/receipts.md).
    pub receipts: bool,
    /// IPTU dos imóveis: o carnê do ano, os lembretes e o IPTU nos impostos pagos (docs/brazil/iptu.md).
    pub iptu: bool,
}

/// Como o país chama as coisas da folha de pagamento.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PayrollTerms {
    /// Contribuição do empregado descontada em folha.
    pub contribution: String,
    /// Imposto de renda retido na fonte.
    pub income_tax: String,
    /// Pagamentos extras do ano (`null`: não há).
    pub extra_pay: Option<String>,
    /// O documento do salário.
    pub payslip: String,
    /// O mesmo no plural ("holerites", "paystubs").
    pub payslips: String,
    /// Registro da empresa pagadora (`null`: o app não lê).
    pub employer_id: Option<String>,
    /// Nome de cada tipo de renda no país (chave: `IncomeSourceKind` em snake_case).
    pub kinds: BTreeMap<String, String>,
}

/// Aposentadoria pública do país no planejador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PublicPension {
    /// Nome do benefício (INSS, Social Security, Segurança Social, previdência pública).
    pub name: String,
    /// Regras que o app calcula (`br_inss`, `us_ssa`, `pt_ss`); `null`: só o valor informado.
    pub rules: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CountryInfo {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub support: CountrySupport,
    pub features: CountryFeatures,
    /// Nome curto da inflação oficial (IPCA, CPI-U, IHPC…).
    pub inflation_label: String,
    /// Juro de referência para comparar investimentos e a reserva (CDI, Fed Funds, €STR…).
    pub rate_label: String,
    pub payroll: PayrollTerms,
    pub pension: PublicPension,
    /// Meses de gastos essenciais na reserva de emergência (sugestão do país).
    #[ts(type = "number")]
    pub emergency_months: u32,
    /// Boas práticas e o que é do país, em frases curtas (onboarding, Família e agente).
    pub tips: Vec<String>,
    /// O que o app ainda não cobre no país.
    pub limits: Vec<String>,
    /// Fontes de dados que fazem sentido no país.
    pub sources: Vec<SourceId>,
    /// Dados da estimativa de impostos (`BR`, `US`, `EU`) e o estado ou país-membro.
    pub tax_dataset: Option<String>,
    pub tax_region: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CountryChoice {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub support: CountrySupport,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CountryState {
    pub info: CountryInfo,
    pub choices: Vec<CountryChoice>,
    pub base_currency: String,
    /// O cofre já tem contas ou lançamentos: a moeda base só muda se pedida.
    pub has_data: bool,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct SetCountryArgs {
    pub country: String,
    /// Trocar a moeda base pela do país. Padrão: só num cofre sem contas nem lançamentos.
    #[serde(default)]
    #[ts(optional)]
    pub change_currency: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CountryChange {
    pub state: CountryState,
    /// Fontes desligadas e religadas pela troca.
    pub disabled: Vec<SourceId>,
    pub enabled: Vec<SourceId>,
    pub currency_changed: bool,
}

/// País da família (Brasil se o cofre ainda não tem a coluna ou a família).
pub fn family_country(conn: &Connection) -> String {
    conn.query_row("SELECT country FROM household LIMIT 1", [], |r| {
        r.get::<_, String>(0)
    })
    .unwrap_or_else(|_| "BR".into())
}

/// A família mora no Brasil.
pub fn is_brazil(conn: &Connection) -> bool {
    family_country(conn) == "BR"
}

/// Nome (em pt-BR) e moeda do país no catálogo (V84).
fn catalog_row(code: &str) -> Option<(String, String)> {
    catalog::COUNTRIES
        .iter()
        .find(|(c, ..)| *c == code)
        .map(|(_, n, cur)| ((*n).to_string(), (*cur).to_string()))
}

/// Nome do país no idioma da instalação (`nucleo-pais-nome-<código>`); fora do catálogo de
/// idiomas, o nome em pt-BR do catálogo de índices.
pub fn country_name(code: &str) -> String {
    let key = format!("nucleo-pais-nome-{}", code.to_ascii_lowercase());
    if crate::i18n::has_key(Locale::PtBr, &key) {
        t(&key)
    } else {
        catalog_row(code)
            .map(|r| r.0)
            .unwrap_or_else(|| code.to_string())
    }
}

fn outside_list(c: &str) -> Error {
    Error::invalid(t_args(
        "nucleo-pais-erro-fora-da-lista",
        &args(&[("pais", c.to_string().into())]),
    ))
}

/// Código do país-membro nos dados de impostos da União Europeia (a Grécia é `EL`).
fn eu_code(code: &str) -> &str {
    if code == "GR" { "EL" } else { code }
}

fn eu_member(code: &str) -> bool {
    crate::tax_burden::dataset::get("EU")
        .map(|d| d.regions.iter().any(|r| r.code == eu_code(code)))
        .unwrap_or(false)
}

pub fn support_of(code: &str) -> CountrySupport {
    match code {
        "BR" => CountrySupport::Full,
        "US" | "PT" | "ES" => CountrySupport::Adapted,
        c if ADAPTED_2.contains(&c) => CountrySupport::Adapted,
        _ => CountrySupport::Generic,
    }
}

/// Fontes que fazem sentido no país. No Brasil, todas (como sempre foi).
pub fn relevant_sources(conn: &Connection, code: &str) -> Result<Vec<SourceId>> {
    if code == "BR" {
        return Ok(SourceId::ALL.to_vec());
    }
    // Globais (cotações, cripto, fundamentos de empresas americanas, câmbio do BCE).
    let mut out = vec![
        SourceId::Yahoo,
        SourceId::Sec,
        SourceId::Ecb,
        SourceId::Oecd,
        SourceId::WorldBank,
        SourceId::Coingecko,
        SourceId::Defillama,
        SourceId::CoinMetrics,
        SourceId::Mempool,
        // Configuração automática do e-mail de ingestão: vale em qualquer país.
        SourceId::MailAutoconfig,
    ];
    // As fontes das séries do catálogo do país (inflação, juros, câmbio).
    let mut stmt = conn.prepare(
        "SELECT s.source FROM country_series cs JOIN index_series s ON s.id = cs.series_id
          WHERE cs.country = ?1",
    )?;
    let sources: Vec<String> = stmt
        .query_map([code], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for s in sources {
        if let Some((p, _)) = crate::indices::providers::DataProvider::of_source(&s) {
            out.push(p.source());
        }
    }
    match code {
        "US" => out.push(SourceId::Fred),
        "CA" => out.push(SourceId::Boc),
        "GB" => out.extend([SourceId::Ons, SourceId::Boe]),
        c if eu_member(c) => out.push(SourceId::Eurostat),
        _ => {}
    }
    // Na ordem de Ajustes › Fontes de dados.
    Ok(SourceId::ALL
        .into_iter()
        .filter(|s| out.contains(s))
        .collect())
}

/// Tipos de renda: (tipo, chave do nome no catálogo).
fn kinds(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, key)| ((*k).to_string(), t(key)))
        .collect()
}

/// Textos do catálogo, na ordem.
fn texts(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|k| t(k)).collect()
}

const GENERIC_KINDS: &[(&str, &str)] = &[
    ("clt", "nucleo-pais-tipo-empregado"),
    ("public_servant", "nucleo-pais-tipo-servidor"),
    ("retirement", "nucleo-pais-tipo-aposentadoria"),
    ("pension", "nucleo-pais-tipo-pensao"),
    ("pj", "nucleo-pais-tipo-prestador"),
    ("self_employed", "nucleo-pais-tipo-autonomo"),
    ("informal", "nucleo-pais-tipo-informal"),
    ("rental", "nucleo-pais-tipo-aluguel"),
    ("benefit", "nucleo-pais-tipo-beneficio"),
    ("other", "nucleo-pais-tipo-outra"),
];

/// Tudo o que o app sabe do país.
pub fn info(conn: &Connection, code: &str) -> Result<CountryInfo> {
    let code = code.trim().to_ascii_uppercase();
    let currency = match code.as_str() {
        "BR" => "BRL".to_string(),
        "US" => "USD".to_string(),
        "PT" | "ES" => "EUR".to_string(),
        c => catalog_row(c).ok_or_else(|| outside_list(c))?.1,
    };
    let name = country_name(&code);
    let support = support_of(&code);
    let eu = eu_member(&code);
    let catalog = crate::indices::catalog::profile(conn, &code)?;
    let first_name =
        |v: &[crate::indices::catalog::CatalogSeries]| v.first().map(|s| s.name.clone());
    let none = CountryFeatures {
        br_market: false,
        br_income_tax: false,
        br_credit_rates: false,
        fipe: false,
        fgts: false,
        tax_burden: eu,
        payroll: false,
        receipts: false,
        iptu: false,
    };
    let generic_payroll = PayrollTerms {
        contribution: t("nucleo-pais-folha-previdencia"),
        income_tax: t("nucleo-pais-folha-imposto-retido"),
        extra_pay: None,
        payslip: t("nucleo-pais-folha-recibo"),
        payslips: t("nucleo-pais-folha-recibos"),
        employer_id: None,
        kinds: kinds(GENERIC_KINDS),
    };
    let mut out = match code.as_str() {
        "BR" => CountryInfo {
            features: CountryFeatures {
                br_market: true,
                br_income_tax: true,
                br_credit_rates: true,
                fipe: true,
                fgts: true,
                tax_burden: true,
                payroll: true,
                receipts: true,
                iptu: true,
            },
            inflation_label: "IPCA".into(),
            rate_label: "CDI".into(),
            payroll: PayrollTerms {
                contribution: "INSS".into(),
                income_tax: "IRRF".into(),
                extra_pay: Some(t("nucleo-pais-br-extras")),
                payslip: t("nucleo-pais-br-holerite"),
                payslips: t("nucleo-pais-br-holerites"),
                employer_id: Some("CNPJ".into()),
                kinds: kinds(&[
                    ("clt", "nucleo-pais-br-tipo-clt"),
                    ("public_servant", "nucleo-pais-tipo-servidor"),
                    ("retirement", "nucleo-pais-tipo-aposentadoria"),
                    ("pension", "nucleo-pais-tipo-pensao"),
                    ("pj", "nucleo-pais-br-tipo-pj"),
                    ("self_employed", "nucleo-pais-tipo-autonomo"),
                    ("informal", "nucleo-pais-tipo-informal"),
                    ("rental", "nucleo-pais-tipo-aluguel"),
                    ("benefit", "nucleo-pais-tipo-beneficio"),
                    ("other", "nucleo-pais-tipo-outra"),
                ]),
            },
            emergency_months: 6,
            tips: texts(&[
                "nucleo-pais-br-dica-1",
                "nucleo-pais-br-dica-2",
                "nucleo-pais-br-dica-3",
                "nucleo-pais-br-dica-4",
            ]),
            limits: vec![],
            ..base_info(
                &code,
                &name,
                &currency,
                support,
                none,
                generic_payroll.clone(),
            )
        },
        "US" => CountryInfo {
            features: CountryFeatures {
                tax_burden: true,
                payroll: true,
                ..none
            },
            inflation_label: "CPI-U".into(),
            rate_label: "Fed Funds".into(),
            payroll: PayrollTerms {
                contribution: t("nucleo-pais-us-contribuicao"),
                income_tax: t("nucleo-pais-us-imposto"),
                extra_pay: None,
                payslip: t("nucleo-pais-us-recibo"),
                payslips: t("nucleo-pais-us-recibos"),
                employer_id: Some("EIN".into()),
                kinds: kinds(&[
                    ("clt", "nucleo-pais-us-tipo-empregado"),
                    ("public_servant", "nucleo-pais-tipo-servidor"),
                    ("retirement", "nucleo-pais-us-tipo-aposentadoria"),
                    ("pension", "nucleo-pais-tipo-pensao"),
                    ("pj", "nucleo-pais-us-tipo-contractor"),
                    ("self_employed", "nucleo-pais-us-tipo-autonomo"),
                    ("informal", "nucleo-pais-us-tipo-informal"),
                    ("rental", "nucleo-pais-tipo-aluguel"),
                    ("benefit", "nucleo-pais-us-tipo-beneficio"),
                    ("other", "nucleo-pais-tipo-outra"),
                ]),
            },
            emergency_months: 6,
            tips: texts(&US_TIPS),
            limits: texts(&[
                "nucleo-pais-us-limite-1",
                "nucleo-pais-us-limite-2",
                "nucleo-pais-us-limite-3",
            ]),
            ..base_info(
                &code,
                &name,
                &currency,
                support,
                none,
                generic_payroll.clone(),
            )
        },
        "PT" => CountryInfo {
            features: CountryFeatures {
                tax_burden: true,
                payroll: true,
                ..none
            },
            inflation_label: t("nucleo-pais-pt-inflacao"),
            rate_label: "€STR".into(),
            payroll: PayrollTerms {
                contribution: "Segurança Social".into(),
                income_tax: t("nucleo-pais-pt-imposto"),
                extra_pay: Some(t("nucleo-pais-pt-extras")),
                payslip: t("nucleo-pais-pt-recibo"),
                payslips: t("nucleo-pais-pt-recibos"),
                employer_id: Some("NIF".into()),
                kinds: kinds(&[
                    ("clt", "nucleo-pais-pt-tipo-conta-de-outrem"),
                    ("public_servant", "nucleo-pais-pt-tipo-funcao-publica"),
                    ("retirement", "nucleo-pais-pt-tipo-reforma"),
                    ("pension", "nucleo-pais-tipo-pensao"),
                    ("pj", "nucleo-pais-pt-tipo-independente"),
                    ("self_employed", "nucleo-pais-pt-tipo-conta-propria"),
                    ("informal", "nucleo-pais-tipo-informal"),
                    ("rental", "nucleo-pais-pt-tipo-rendas"),
                    ("benefit", "nucleo-pais-pt-tipo-prestacao"),
                    ("other", "nucleo-pais-tipo-outra"),
                ]),
            },
            emergency_months: 6,
            tips: texts(&PT_TIPS),
            limits: texts(&[
                "nucleo-pais-pt-limite-1",
                "nucleo-pais-pt-limite-2",
                "nucleo-pais-pt-limite-3",
            ]),
            ..base_info(
                &code,
                &name,
                &currency,
                support,
                none,
                generic_payroll.clone(),
            )
        },
        "ES" => CountryInfo {
            features: CountryFeatures {
                tax_burden: true,
                payroll: true,
                ..none
            },
            inflation_label: t("nucleo-pais-es-inflacao"),
            rate_label: "€STR".into(),
            payroll: PayrollTerms {
                contribution: "Seguridad Social".into(),
                income_tax: t("nucleo-pais-es-imposto"),
                extra_pay: Some(t("nucleo-pais-es-extras")),
                payslip: t("nucleo-pais-es-recibo"),
                payslips: t("nucleo-pais-es-recibos"),
                employer_id: Some("CIF".into()),
                kinds: kinds(&[
                    ("clt", "nucleo-pais-es-tipo-cuenta-ajena"),
                    ("public_servant", "nucleo-pais-es-tipo-funcionario"),
                    ("retirement", "nucleo-pais-es-tipo-jubilacion"),
                    ("pension", "nucleo-pais-tipo-pensao"),
                    ("pj", "nucleo-pais-es-tipo-autonomo"),
                    ("self_employed", "nucleo-pais-es-tipo-cuenta-propia"),
                    ("informal", "nucleo-pais-tipo-informal"),
                    ("rental", "nucleo-pais-tipo-aluguel"),
                    ("benefit", "nucleo-pais-es-tipo-prestacion"),
                    ("other", "nucleo-pais-tipo-outra"),
                ]),
            },
            emergency_months: 6,
            tips: texts(&ES_TIPS),
            limits: texts(&[
                "nucleo-pais-es-limite-1",
                "nucleo-pais-es-limite-2",
                "nucleo-pais-es-limite-3",
            ]),
            ..base_info(
                &code,
                &name,
                &currency,
                support,
                none,
                generic_payroll.clone(),
            )
        },
        c if ADAPTED_2.contains(&c) => {
            let k = c.to_ascii_lowercase();
            let key = |s: &str| format!("nucleo-pais-{k}-{s}");
            CountryInfo {
                features: CountryFeatures {
                    tax_burden: eu,
                    payroll: true,
                    ..none
                },
                payroll: PayrollTerms {
                    contribution: t(&key("contribuicao")),
                    income_tax: t(&key("imposto")),
                    extra_pay: (c == "AR").then(|| t(&key("extras"))),
                    payslip: t(&key("recibo")),
                    payslips: t(&key("recibos")),
                    employer_id: None,
                    kinds: kinds(GENERIC_KINDS),
                },
                emergency_months: if c == "GB" { 3 } else { 6 },
                tips: vec![
                    t(&key("dica-1")),
                    t(&key("dica-2")),
                    t(&key("dica-3")),
                    t("nucleo-pais-regra-50-30-20"),
                ],
                limits: vec![t(&key("limite-1")), t(&key("limite-2"))],
                ..base_info(
                    &code,
                    &name,
                    &currency,
                    support,
                    none,
                    generic_payroll.clone(),
                )
            }
        }
        _ => CountryInfo {
            tips: texts(&[
                "nucleo-pais-outro-dica-1",
                "nucleo-pais-outro-dica-2",
                "nucleo-pais-outro-dica-3",
            ]),
            limits: texts(&["nucleo-pais-outro-limite-1", "nucleo-pais-outro-limite-2"]),
            ..base_info(&code, &name, &currency, support, none, generic_payroll)
        },
    };
    // Outros países (e os adaptados na rodada 2): os nomes das séries do catálogo.
    if let Some(p) = &catalog
        && !["BR", "US", "PT", "ES"].contains(&code.as_str())
    {
        if let Some(n) = first_name(&p.inflation) {
            out.inflation_label = n;
        }
        if let Some(n) = first_name(&p.rates) {
            out.rate_label = n;
        }
    }
    out.sources = relevant_sources(conn, &code)?;
    (out.tax_dataset, out.tax_region) = match code.as_str() {
        "BR" => (Some("BR".into()), None),
        "US" => (Some("US".into()), None),
        c if eu => (Some("EU".into()), Some(eu_code(c).to_string())),
        _ => (None, None),
    };
    Ok(out)
}

/// A aposentadoria pública do país e as regras que o app conhece.
pub fn pension_of(code: &str) -> PublicPension {
    let (name, rules) = match code {
        "BR" => ("INSS".to_string(), Some("br_inss")),
        "US" => ("Social Security".to_string(), Some("us_ssa")),
        "PT" => ("Segurança Social".to_string(), Some("pt_ss")),
        // A pensão de jubilación ainda não é calculada: o valor informado.
        "ES" => ("Seguridad Social".to_string(), None),
        "GB" => ("State Pension".to_string(), None),
        "CA" => ("CPP/OAS".to_string(), None),
        "FR" => ("Retraite de base".to_string(), None),
        "DE" => ("Gesetzliche Rente".to_string(), None),
        "AR" => ("Jubilación (ANSES)".to_string(), None),
        "CL" => ("AFP/PGU".to_string(), None),
        _ => (t("nucleo-pais-previdencia-publica"), None),
    };
    PublicPension {
        name,
        rules: rules.map(Into::into),
    }
}

/// Nome da aposentadoria pública do país da família.
pub fn pension_name(conn: &Connection) -> String {
    pension_of(&family_country(conn)).name
}

fn base_info(
    code: &str,
    name: &str,
    currency: &str,
    support: CountrySupport,
    features: CountryFeatures,
    payroll: PayrollTerms,
) -> CountryInfo {
    CountryInfo {
        code: code.into(),
        name: name.into(),
        currency: currency.into(),
        support,
        features,
        inflation_label: t("nucleo-pais-inflacao"),
        rate_label: t("nucleo-pais-juro-referencia"),
        payroll,
        pension: pension_of(code),
        emergency_months: 6,
        tips: vec![],
        limits: vec![],
        sources: vec![],
        tax_dataset: None,
        tax_region: None,
    }
}

const US_TIPS: [&str; 5] = [
    "nucleo-pais-us-dica-1",
    "nucleo-pais-us-dica-2",
    "nucleo-pais-us-dica-3",
    "nucleo-pais-us-dica-4",
    "nucleo-pais-regra-50-30-20",
];

const ES_TIPS: [&str; 5] = [
    "nucleo-pais-es-dica-1",
    "nucleo-pais-es-dica-2",
    "nucleo-pais-es-dica-3",
    "nucleo-pais-es-dica-4",
    "nucleo-pais-regra-50-30-20",
];

const PT_TIPS: [&str; 5] = [
    "nucleo-pais-pt-dica-1",
    "nucleo-pais-pt-dica-2",
    "nucleo-pais-pt-dica-3",
    "nucleo-pais-pt-dica-4",
    "nucleo-pais-regra-50-30-20",
];

/// Países que a família pode escolher: os com adaptação primeiro, depois o catálogo na ordem
/// do nome no idioma da instalação. Vale sem cofre (a criação do cofre pergunta o país).
pub fn choices() -> Vec<CountryChoice> {
    let choice = |(code, _, currency): &(&str, &str, &str)| CountryChoice {
        code: (*code).to_string(),
        name: country_name(code),
        currency: (*currency).to_string(),
        support: support_of(code),
    };
    let featured = FEATURED
        .iter()
        .filter_map(|f| catalog::COUNTRIES.iter().find(|(c, ..)| c == f))
        .map(choice);
    let mut rest: Vec<CountryChoice> = catalog::COUNTRIES
        .iter()
        .filter(|(c, ..)| !FEATURED.contains(c))
        .map(choice)
        .collect();
    rest.sort_by_cached_key(|c| crate::import::text::fold(&c.name));
    featured.chain(rest).collect()
}

fn has_data(conn: &Connection) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM account) OR EXISTS (SELECT 1 FROM txn)",
        [],
        |r| r.get(0),
    )?)
}

pub fn state(conn: &Connection) -> Result<CountryState> {
    Ok(CountryState {
        info: info(conn, &family_country(conn))?,
        choices: choices(),
        base_currency: crate::household::base_currency(conn)?,
        has_data: has_data(conn)?,
    })
}

/// Liga as fontes do país e desliga as que não fazem sentido nele. Desliga só as que estavam
/// ligadas (e anota quais foram); religa só as que o app tinha desligado pelo país.
pub fn apply_sources(conn: &Connection, code: &str) -> Result<(Vec<SourceId>, Vec<SourceId>)> {
    let relevant = relevant_sources(conn, code)?;
    let mut off_by_country: Vec<String> =
        crate::settings::get_or(conn, SOURCES_OFF_SETTING, Vec::new())?;
    let (mut disabled, mut enabled) = (vec![], vec![]);
    for id in SourceId::ALL {
        let on = crate::sources::enabled(conn, id);
        let ours = off_by_country.iter().any(|k| k == id.key());
        if relevant.contains(&id) {
            if ours {
                off_by_country.retain(|k| k != id.key());
                if !on {
                    crate::sources::set_enabled(conn, id, true)?;
                    enabled.push(id);
                }
            }
        } else if on {
            crate::sources::set_enabled(conn, id, false)?;
            off_by_country.push(id.key().to_string());
            disabled.push(id);
        }
    }
    off_by_country.sort();
    off_by_country.dedup();
    if off_by_country.is_empty() {
        crate::settings::delete(conn, SOURCES_OFF_SETTING)?;
    } else {
        crate::settings::set(conn, SOURCES_OFF_SETTING, &off_by_country)?;
    }
    Ok((disabled, enabled))
}

/// Troca o país da família: a coluna, a moeda base (se pedida ou num cofre vazio), as fontes,
/// o país dos índices (volta ao automático, que segue a família) e o da estimativa de
/// impostos.
pub fn set(tx: &Transaction<'_>, a: &SetCountryArgs) -> Result<CountryChange> {
    let code = a.country.trim().to_ascii_uppercase();
    if !catalog::COUNTRIES.iter().any(|(c, ..)| *c == code) {
        return Err(outside_list(&a.country));
    }
    let i = info(tx, &code)?;
    let change_currency = a.change_currency.unwrap_or(!has_data(tx)?);
    let before = crate::household::base_currency(tx)?;
    tx.execute(
        "UPDATE household SET country = ?1, updated_at = ?2 WHERE country <> ?1",
        params![code, crate::time::now_utc()],
    )?;
    let currency_changed = change_currency && before != i.currency;
    if currency_changed {
        crate::household::update(
            tx,
            &crate::household::HouseholdPatch {
                name: None,
                base_currency: Some(i.currency.clone()),
            },
        )?;
    }
    let (disabled, enabled) = apply_sources(tx, &code)?;
    crate::settings::delete(tx, crate::indices::catalog::COUNTRY_SETTING)?;
    crate::tax_burden::follow_country(tx, i.tax_dataset.as_deref(), i.tax_region.as_deref())?;
    Ok(CountryChange {
        state: state(tx)?,
        disabled,
        enabled,
        currency_changed,
    })
}

/// Uma linha para o agente (em inglês, como o resto das instruções): onde a família mora e o
/// que não sugerir fora do Brasil.
pub fn prompt_line(conn: &Connection) -> String {
    let code = family_country(conn);
    let Ok(i) = info(conn, &code) else {
        return String::new();
    };
    if code == "BR" {
        return "The family lives in Brazil.".into();
    }
    format!(
        "The family lives in {} ({code}, currency {}): use that country's rules, products and \
         terms. Reference inflation: {}; reference interest rate: {}. Do not suggest Brazilian \
         products or rules (Tesouro Direto, CDB, LCI, FGTS, INSS, income tax with DARF, Pix) \
         unless the family has something in Brazil.{}",
        i.name,
        i.currency,
        i.inflation_label,
        i.rate_label,
        if i.support == CountrySupport::Generic {
            " The app does not know this country's taxes and payroll: make it clear when a \
             calculation is approximate."
        } else {
            ""
        }
    )
}

/// Textos prontos do app (dicas da saúde financeira, insights, avisos) nos termos do país:
/// tira o que só existe no Brasil (Tesouro Selic, CDB, FGTS, consignado, a resolução do CMN,
/// os juros de mercado do Banco Central) e troca pelos produtos de lá. Os trechos procurados e
/// os que entram no lugar são do catálogo (`nucleo-pais-*-br`), no idioma da instalação: um
/// teste confere que os textos de origem trazem os trechos em cada idioma.
pub fn localize_text(code: &str, text: &str) -> String {
    if code == "BR" {
        return text.to_string();
    }
    let reserve = match code {
        "US" => "nucleo-pais-reserva-us",
        "PT" => "nucleo-pais-reserva-pt",
        "ES" => "nucleo-pais-reserva-es",
        _ => "nucleo-pais-reserva-outro",
    };
    let overdraft = match code {
        "US" => "nucleo-pais-cheque-especial-us",
        "PT" => "nucleo-pais-cheque-especial-pt",
        "ES" => "nucleo-pais-cheque-especial-es",
        _ => "nucleo-pais-cheque-especial-br",
    };
    let swaps = [
        (t("nucleo-pais-reserva-br"), t(reserve)),
        (t("nucleo-pais-fgts-br"), t("nucleo-pais-fgts-outro")),
        (
            format!(" {}", t("nucleo-pais-consignado-br")),
            String::new(),
        ),
        (
            format!(" {}", t("nucleo-pais-juros-mercado-br")),
            String::new(),
        ),
        (t("nucleo-pais-tarifas-br"), t("nucleo-pais-tarifas-outro")),
        (t("nucleo-pais-cheque-especial-br"), t(overdraft)),
    ];
    let mut out = text.to_string();
    for (from, to) in swaps {
        if from.trim().is_empty() || from == to {
            continue;
        }
        out = out.replace(&from, &to);
    }
    out
}

/// [`localize_text`] em todos os textos de uma resposta.
pub fn localize_value(conn: &Connection, v: &mut serde_json::Value) {
    let code = family_country(conn);
    if code == "BR" {
        return;
    }
    fn walk(code: &str, v: &mut serde_json::Value) {
        match v {
            serde_json::Value::String(s) => *s = localize_text(code, s),
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(code, x)),
            serde_json::Value::Object(o) => o.values_mut().for_each(|x| walk(code, x)),
            _ => {}
        }
    }
    walk(&code, v);
}

pub const READ_ONLY: &[&str] = &["get"];

/// `country.get` e `country.set` (só pela interface).
pub fn handle(
    ctx: &mut crate::api::Ctx<'_>,
    action: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    use crate::api::{parse_args, to_value};
    match action {
        "get" => to_value(state(ctx.conn())?),
        "set" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(t("nucleo-pais-erro-so-interface")));
            }
            let a: SetCountryArgs = parse_args(args)?;
            let name = info(ctx.conn(), &a.country)?.name;
            let reason = t_args(
                "nucleo-pais-motivo",
                &crate::i18n::args(&[("pais", name.into())]),
            );
            to_value(ctx.write(&reason, |tx| set(tx, &a))?.value)
        }
        _ => Err(Error::UnknownCommand(format!("country.{action}"))),
    }
}

#[cfg(test)]
mod tests;
