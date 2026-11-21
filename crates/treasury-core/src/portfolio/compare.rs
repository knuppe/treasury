//! Calculadoras de renda fixa do Brasil, sem gravar nada no cofre:
//!
//! - `portfolio.compare_fixed_income`: ofertas que a pessoa digitou (CDB, RDB, LC, LCI, LCA,
//!   CRI, CRA, debêntures, Tesouro e poupança), lado a lado e **na ordem em que vieram**: valor
//!   bruto, IOF, IR, custódia da B3, líquido, rendimento líquido ao ano e acima da inflação, e a
//!   equivalência em % do CDI (tributável e isento). Não há "melhor": comparar números que a
//!   pessoa trouxe não é recomendação (CVM Res. 20/2021).
//! - `portfolio.simulate_fixed_income_sale`: vender um prefixado ou IPCA+ antes do vencimento,
//!   pela taxa de mercado do dia (marcação a mercado), contra levar até o fim; e a taxa que um
//!   título novo até o mesmo vencimento precisa pagar para a troca empatar.
//!
//! Regras (estimativa, conferidas em 31/10/2026; a MP 1.303/2025, que mudaria a tributação de
//! aplicações financeiras e de LCI/LCA, perdeu a validade em 08/10/2025 sem ser votada):
//! - IR regressivo por dias corridos (22,5%, 20%, 17,5%, 15%) e isenção de LCI, LCA, CRI, CRA
//!   e poupança para pessoa física: Lei 11.033/2004, art. 1º e art. 3º, II e IV; debêntures
//!   incentivadas: Lei 12.431/2011, art. 2º.
//! - IOF regressivo sobre o rendimento até o 29º dia: Decreto 6.306/2007, art. 32 e anexo.
//! - Custódia do Tesouro Direto: 0,20% a.a. da B3 sobre o valor dos títulos; Tesouro Selic sem
//!   taxa até R$ 10 mil (parâmetros em `methodology/v0.1.0/classes/fixed_income.json`). O IR
//!   aqui incide sobre o rendimento bruto, sem abater a custódia (o lado conservador).
//! - Poupança: 0,5% a.m. + TR com a Selic meta acima de 8,5% a.a.; senão, 70% da Selic + TR,
//!   só nos aniversários mensais (Lei 8.177/1991, art. 12, redação da Lei 12.703/2012).
//! - FGC: CDB, RDB, LC, LCI, LCA e poupança até R$ 250 mil por CPF e instituição.
//!
//! Prazos em dias úteis (base 252, calendário de feriados nacionais) para os juros e em dias
//! corridos para o IR e o IOF. CDI, Selic, IPCA e TR: os do cofre (o último CDI, a Selic meta,
//! a projeção Focus ou os últimos 12 meses do IPCA e a última TR), trocáveis pela pessoa.

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::assets::FixedIncomeKind;
use crate::error::{Error, Result};
use crate::finance::calendar::business_days_between;
use crate::finance::fixed_income::{ir_rate, net_redemption};
use crate::i18n::{self, t, t_args};
use crate::score::fixed_income as fi;
use crate::time::{add_months, fmt_date, parse_date, today_local};

/// Teto das ofertas numa comparação.
pub const MAX_OFFERS: usize = 12;

/// Como a oferta rende.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum OfferIndex {
    /// `rate` % do CDI (ex.: 110).
    CdiPct,
    /// CDI + `rate` % a.a.
    CdiPlus,
    /// Selic + `rate` % a.a. (Tesouro Selic: o ágio ou deságio, em geral perto de zero).
    SelicPlus,
    /// `rate` % a.a. prefixado.
    Prefixed,
    /// IPCA + `rate` % a.a.
    IpcaPlus,
    /// Regra da poupança (`rate` ignorado).
    Savings,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FixedIncomeOffer {
    /// Nome que a pessoa deu ("CDB banco A"); sem nome, o tipo e a taxa.
    #[serde(default)]
    #[ts(optional)]
    pub label: Option<String>,
    pub kind: FixedIncomeKind,
    pub index: OfferIndex,
    /// % do CDI ou taxa em % a.a., conforme `index`.
    #[serde(default)]
    pub rate: f64,
    /// Data do resgate (`YYYY-MM-DD`); sem ela, `months` meses depois do início.
    #[serde(default)]
    #[ts(optional)]
    pub end: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub months: Option<u32>,
    /// Isento de IR e IOF; padrão pelo tipo (LCI, LCA, CRI, CRA, incentivada e poupança).
    #[serde(default)]
    #[ts(optional)]
    pub ir_exempt: Option<bool>,
    /// Taxa de custódia ou administração, fração a.a.; padrão: a da B3 no Tesouro, zero no resto.
    #[serde(default)]
    #[ts(optional)]
    pub custody: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompareFixedIncomeArgs {
    /// Valor aplicado em centavos (BRL).
    #[ts(type = "number")]
    pub amount: i64,
    /// Data da aplicação; padrão hoje.
    #[serde(default)]
    #[ts(optional)]
    pub start: Option<String>,
    pub offers: Vec<FixedIncomeOffer>,
    /// Premissas em fração a.a. (CDI, Selic meta, IPCA) e a.m. (TR); padrão: as do cofre.
    #[serde(default)]
    #[ts(optional)]
    pub cdi: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub selic: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub ipca: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub tr: Option<f64>,
}

/// Premissa usada e de onde veio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RateAssumption {
    pub value: f64,
    /// `user` (digitada), `vault` (série do cofre), `projection` (Focus) ou `default`.
    pub source: String,
    /// Data do dado do cofre, quando houver.
    pub date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RateAssumptions {
    /// CDI, fração a.a. (o Tesouro Selic rende a Selic efetiva, que fica junto do CDI).
    pub cdi: RateAssumption,
    /// Selic meta, fração a.a. (só para a regra da poupança).
    pub selic: RateAssumption,
    /// IPCA esperado, fração a.a.
    pub ipca: RateAssumption,
    /// TR, fração a.m.
    pub tr: RateAssumption,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OfferResult {
    pub label: String,
    pub kind: FixedIncomeKind,
    pub index: OfferIndex,
    pub rate: f64,
    pub start: String,
    pub end: String,
    #[ts(type = "number")]
    pub calendar_days: i64,
    #[ts(type = "number")]
    pub business_days: i64,
    pub ir_exempt: bool,
    /// Alíquota de IR do prazo (zero se isento).
    pub ir_rate: f64,
    /// Garantia do FGC pelo tipo (`None` quando o tipo não diz).
    pub fgc: Option<bool>,
    #[ts(type = "number")]
    pub invested: i64,
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub iof: i64,
    #[ts(type = "number")]
    pub ir: i64,
    #[ts(type = "number")]
    pub custody: i64,
    #[ts(type = "number")]
    pub net: i64,
    #[ts(type = "number")]
    pub net_income: i64,
    /// Rendimento líquido equivalente ao ano (base 252).
    pub net_annual: f64,
    /// O mesmo, acima do IPCA suposto.
    pub real_annual: f64,
    /// % do CDI que um título tributável (CDB) precisaria pagar no mesmo prazo para dar o mesmo
    /// líquido.
    pub taxable_equivalent_pct_cdi: Option<f64>,
    /// % do CDI que um título isento (LCI/LCA) precisaria pagar para dar o mesmo líquido.
    pub exempt_equivalent_pct_cdi: Option<f64>,
    /// Avisos da linha (IOF por resgate antes de 30 dias, poupança fora do aniversário…).
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FixedIncomeComparison {
    pub assumptions: RateAssumptions,
    /// Na ordem das ofertas recebidas (sem ordenar por "melhor").
    pub offers: Vec<OfferResult>,
    /// Bases legais e o aviso de estimativa, no idioma do app.
    pub notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Premissas

fn pick(
    user: Option<f64>,
    vault: Option<(f64, Option<NaiveDate>, &str)>,
    fallback: f64,
) -> RateAssumption {
    match (user, vault) {
        (Some(v), _) => RateAssumption {
            value: v,
            source: "user".into(),
            date: None,
        },
        (None, Some((v, d, src))) => RateAssumption {
            value: v,
            source: src.into(),
            date: d.map(fmt_date),
        },
        (None, None) => RateAssumption {
            value: fallback,
            source: "default".into(),
            date: None,
        },
    }
}

/// IPCA esperado para os próximos 12 meses: a projeção Focus, quando o cofre tem os 12 meses;
/// senão, os últimos 12 meses oficiais.
fn ipca_expected(
    conn: &Connection,
    today: NaiveDate,
) -> Result<Option<(f64, Option<NaiveDate>, &'static str)>> {
    let first = today.with_day(1).expect("dia 1");
    let ahead = crate::indices::monthly_rates(conn, "ipca", first, add_months(first, 11))?;
    if ahead.len() == 12 && ahead.iter().any(|r| r.estimated) {
        let acc = ahead.iter().fold(1.0, |a, r| {
            a * (1.0 + r.pct.to_f64().unwrap_or(0.0) / 100.0)
        });
        return Ok(Some((acc - 1.0, None, "projection")));
    }
    let official = fi::monthly(conn, "ipca", add_months(first, -24), today)?;
    let upto = (today.year(), today.month());
    Ok(fi::twelve_months(&official, upto)
        .map(|(v, (y, m))| (v, NaiveDate::from_ymd_opt(y, m, 1), "vault")))
}

/// Última TR mensal (fração a.m.) até hoje.
fn tr_last(
    conn: &Connection,
    today: NaiveDate,
) -> Result<Option<(f64, Option<NaiveDate>, &'static str)>> {
    let pts = crate::indices::points(conn, "tr", today - chrono::Duration::days(62), today)?;
    Ok(pts.last().map(|p| {
        (
            p.value.to_f64().unwrap_or(0.0) / 100.0,
            Some(p.date),
            "vault",
        )
    }))
}

/// Premissas do cofre, com as que a pessoa digitou por cima. Sem dados, CDI e Selic de 10% a.a.,
/// IPCA de 4,5% e TR zero, sempre marcados como `default`.
pub fn assumptions(
    conn: &Connection,
    cdi: Option<f64>,
    selic: Option<f64>,
    ipca: Option<f64>,
    tr: Option<f64>,
) -> Result<RateAssumptions> {
    let today = today_local();
    let cdi_vault = if cdi.is_none() {
        fi::cdi_annual(conn, today)?.map(|(v, d)| (v, Some(d), "vault"))
    } else {
        None
    };
    let selic_vault = if selic.is_none() {
        fi::selic_meta(conn, today)?.map(|v| (v, None, "vault"))
    } else {
        None
    };
    let ipca_vault = if ipca.is_none() {
        ipca_expected(conn, today)?
    } else {
        None
    };
    let tr_vault = if tr.is_none() {
        tr_last(conn, today)?
    } else {
        None
    };
    let cdi = pick(cdi, cdi_vault, 0.10);
    let selic_fallback = cdi.value + 0.001;
    Ok(RateAssumptions {
        selic: pick(selic, selic_vault, selic_fallback),
        cdi,
        ipca: pick(ipca, ipca_vault, 0.045),
        tr: pick(tr, tr_vault, 0.0),
    })
}

// ---------------------------------------------------------------------------
// Cálculo

fn is_treasury(kind: FixedIncomeKind) -> bool {
    use FixedIncomeKind::*;
    matches!(
        kind,
        TesouroSelic | TesouroPrefixado | TesouroIpca | TesouroRenda
    )
}

/// Meses cheios de aniversário entre as datas (a poupança só rende neles).
fn anniversaries(start: NaiveDate, end: NaiveDate) -> u32 {
    let mut n = 0;
    while add_months(start, n as i32 + 1) <= end {
        n += 1;
    }
    n
}

/// Rendimento mensal da poupança (fração), pela Selic meta e pela TR.
pub fn savings_monthly(selic_target: f64, tr: f64) -> f64 {
    let p = &fi::params().poupanca;
    let base = if selic_target > p.selic_threshold {
        p.monthly_rate
    } else {
        p.selic_share * ((1.0 + selic_target).powf(1.0 / 12.0) - 1.0)
    };
    base + tr
}

/// Fator bruto da oferta no período.
fn gross_factor(index: OfferIndex, rate: f64, du: i64, months: u32, a: &RateAssumptions) -> f64 {
    let years = du as f64 / 252.0;
    match index {
        OfferIndex::CdiPct => {
            let day = (1.0 + a.cdi.value).powf(1.0 / 252.0) - 1.0;
            (1.0 + day * rate / 100.0).powf(du as f64)
        }
        OfferIndex::CdiPlus => ((1.0 + a.cdi.value) * (1.0 + rate / 100.0)).powf(years),
        OfferIndex::SelicPlus => ((1.0 + a.cdi.value) * (1.0 + rate / 100.0)).powf(years),
        OfferIndex::Prefixed => (1.0 + rate / 100.0).powf(years),
        OfferIndex::IpcaPlus => ((1.0 + a.ipca.value) * (1.0 + rate / 100.0)).powf(years),
        OfferIndex::Savings => {
            (1.0 + savings_monthly(a.selic.value, a.tr.value)).powi(months as i32)
        }
    }
}

fn cents(v: f64) -> i64 {
    v.round() as i64
}

fn dec_cents(c: i64) -> Decimal {
    Decimal::new(c, 2)
}

fn to_cents(d: Decimal) -> i64 {
    (d * Decimal::ONE_HUNDRED).round().to_i64().unwrap_or(0)
}

/// (IOF, IR) de um resgate, em centavos.
fn taxes(invested: i64, gross: i64, days: i64, exempt: bool) -> (i64, i64) {
    let r = net_redemption(dec_cents(invested), dec_cents(gross), days, exempt);
    (to_cents(r.iof), to_cents(r.ir))
}

/// Custódia estimada no período: taxa a.a. sobre o valor médio (início e fim), pro rata dos
/// dias corridos; no Tesouro Selic, só sobre o que passa da isenção.
fn custody_cost(kind: FixedIncomeKind, fee: f64, invested: i64, gross: i64, days: i64) -> i64 {
    if fee <= 0.0 || days <= 0 {
        return 0;
    }
    let mut avg = (invested + gross) as f64 / 2.0;
    if kind == FixedIncomeKind::TesouroSelic {
        let exempt = fi::params().selic_exempt().to_f64().unwrap_or(0.0) * 100.0;
        avg = (avg - exempt).max(0.0);
    }
    cents(avg * fee * days as f64 / 365.0)
}

/// Líquido de um título pós-fixado no CDI a `pct` % no prazo, tributável ou isento.
fn net_at_pct_cdi(
    pct: f64,
    exempt: bool,
    amount: i64,
    du: i64,
    days: i64,
    a: &RateAssumptions,
) -> i64 {
    let gross = cents(amount as f64 * gross_factor(OfferIndex::CdiPct, pct, du, 0, a));
    let (iof, ir) = taxes(amount, gross, days, exempt);
    gross - iof - ir
}

/// % do CDI que dá `target` de líquido no prazo (bisseção em [0, 1000]).
fn equivalent_pct(
    target: i64,
    exempt: bool,
    amount: i64,
    du: i64,
    days: i64,
    a: &RateAssumptions,
) -> Option<f64> {
    if du <= 0 || a.cdi.value <= 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (0.0f64, 1000.0f64);
    if net_at_pct_cdi(hi, exempt, amount, du, days, a) < target
        || net_at_pct_cdi(lo, exempt, amount, du, days, a) > target
    {
        return None;
    }
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if net_at_pct_cdi(mid, exempt, amount, du, days, a) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((hi * 10.0).round() / 10.0)
}

fn annualized(net: i64, invested: i64, du: i64) -> f64 {
    if invested <= 0 || du <= 0 {
        return 0.0;
    }
    (net as f64 / invested as f64).powf(252.0 / du as f64) - 1.0
}

fn default_label(o: &FixedIncomeOffer) -> String {
    let kind = t(&format!("nucleo-rf-tipo-{}", kind_key(o.kind)));
    let rate = match o.index {
        OfferIndex::CdiPct => t_args(
            "nucleo-rf-taxa-cdi-pct",
            &i18n::args(&[("taxa", o.rate.into())]),
        ),
        OfferIndex::CdiPlus => t_args(
            "nucleo-rf-taxa-cdi-mais",
            &i18n::args(&[("taxa", o.rate.into())]),
        ),
        OfferIndex::SelicPlus => t_args(
            "nucleo-rf-taxa-selic-mais",
            &i18n::args(&[("taxa", o.rate.into())]),
        ),
        OfferIndex::Prefixed => t_args(
            "nucleo-rf-taxa-pre",
            &i18n::args(&[("taxa", o.rate.into())]),
        ),
        OfferIndex::IpcaPlus => t_args(
            "nucleo-rf-taxa-ipca-mais",
            &i18n::args(&[("taxa", o.rate.into())]),
        ),
        OfferIndex::Savings => return kind,
    };
    format!("{kind} {rate}")
}

fn kind_key(k: FixedIncomeKind) -> &'static str {
    use FixedIncomeKind::*;
    match k {
        Cdb => "cdb",
        Rdb => "rdb",
        Lc => "lc",
        Lci => "lci",
        Lca => "lca",
        Cri => "cri",
        Cra => "cra",
        Debenture => "debenture",
        DebentureIncentivada => "debenture-incentivada",
        TesouroSelic => "tesouro-selic",
        TesouroPrefixado => "tesouro-prefixado",
        TesouroIpca => "tesouro-ipca",
        TesouroRenda => "tesouro-renda",
        Poupanca => "poupanca",
        Other => "outro",
    }
}

fn evaluate(
    o: &FixedIncomeOffer,
    amount: i64,
    start: NaiveDate,
    a: &RateAssumptions,
) -> Result<OfferResult> {
    let end = match (&o.end, o.months) {
        (Some(e), _) => parse_date(e)?,
        (None, Some(m)) if m > 0 => add_months(start, m as i32),
        _ => return Err(Error::invalid(t("nucleo-rf-erro-prazo"))),
    };
    if end <= start {
        return Err(Error::invalid(t("nucleo-rf-erro-prazo")));
    }
    if !o.rate.is_finite()
        || (o.index != OfferIndex::Savings && o.index != OfferIndex::SelicPlus && o.rate <= -100.0)
    {
        return Err(Error::invalid(t("nucleo-rf-erro-taxa")));
    }
    let days = (end - start).num_days();
    let du = business_days_between(start, end);
    let months = anniversaries(start, end);
    let exempt = o.ir_exempt.unwrap_or_else(|| o.kind.default_ir_exempt());
    let gross = cents(amount as f64 * gross_factor(o.index, o.rate, du, months, a));
    let (iof, ir) = taxes(amount, gross, days, exempt);
    let fee = o.custody.unwrap_or(if is_treasury(o.kind) {
        fi::params().custody.b3_fee
    } else {
        0.0
    });
    let custody = custody_cost(o.kind, fee, amount, gross, days);
    let net = gross - iof - ir - custody;
    let net_annual = annualized(net, amount, du);
    let mut notes = Vec::new();
    if !exempt && days < 30 {
        notes.push(t("nucleo-rf-nota-iof"));
    }
    if o.index == OfferIndex::Savings && add_months(start, months as i32) < end {
        notes.push(t("nucleo-rf-nota-aniversario"));
    }
    if custody > 0 {
        notes.push(t("nucleo-rf-nota-custodia"));
    }
    Ok(OfferResult {
        label: o
            .label
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(|| default_label(o), str::to_string),
        kind: o.kind,
        index: o.index,
        rate: o.rate,
        start: fmt_date(start),
        end: fmt_date(end),
        calendar_days: days,
        business_days: du,
        ir_exempt: exempt,
        ir_rate: if exempt {
            0.0
        } else {
            ir_rate(days).to_f64().unwrap_or(0.0)
        },
        fgc: fi::fgc_covered(o.kind),
        invested: amount,
        gross,
        iof,
        ir,
        custody,
        net,
        net_income: net - amount,
        net_annual,
        real_annual: (1.0 + net_annual) / (1.0 + a.ipca.value) - 1.0,
        taxable_equivalent_pct_cdi: equivalent_pct(net, false, amount, du, days, a),
        exempt_equivalent_pct_cdi: equivalent_pct(net, true, amount, du, days, a),
        notes,
    })
}

fn legal_notes() -> Vec<String> {
    vec![
        t("nucleo-rf-fonte-ir"),
        t("nucleo-rf-fonte-iof"),
        t("nucleo-rf-fonte-custodia"),
        t("nucleo-rf-fonte-poupanca"),
        t("nucleo-rf-aviso"),
    ]
}

pub fn compare(conn: &Connection, args: &CompareFixedIncomeArgs) -> Result<FixedIncomeComparison> {
    if args.amount <= 0 {
        return Err(Error::invalid(t("nucleo-rf-erro-valor")));
    }
    if args.offers.is_empty() || args.offers.len() > MAX_OFFERS {
        return Err(Error::invalid(t_args(
            "nucleo-rf-erro-ofertas",
            &i18n::args(&[("max", (MAX_OFFERS as i64).into())]),
        )));
    }
    let start = match &args.start {
        Some(s) => parse_date(s)?,
        None => today_local(),
    };
    let a = assumptions(conn, args.cdi, args.selic, args.ipca, args.tr)?;
    let offers = args
        .offers
        .iter()
        .map(|o| evaluate(o, args.amount, start, &a))
        .collect::<Result<Vec<_>>>()?;
    Ok(FixedIncomeComparison {
        assumptions: a,
        offers,
        notes: legal_notes(),
    })
}

// ---------------------------------------------------------------------------
// Venda antes do vencimento

/// Título que paga tudo no vencimento (sem cupom).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SaleIndex {
    Prefixed,
    IpcaPlus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FixedIncomeSaleArgs {
    /// Valor aplicado em centavos.
    #[ts(type = "number")]
    pub amount: i64,
    pub start: String,
    pub maturity: String,
    /// Data da venda; padrão hoje.
    #[serde(default)]
    #[ts(optional)]
    pub sale_date: Option<String>,
    pub index: SaleIndex,
    /// Taxa contratada, % a.a. (prefixada ou o juro real do IPCA+).
    pub contracted_rate: f64,
    /// Taxa de mercado hoje para o mesmo vencimento, % a.a.
    pub market_rate: f64,
    #[serde(default)]
    #[ts(optional)]
    pub ir_exempt: Option<bool>,
    /// IPCA suposto, fração a.a. (padrão: o do cofre).
    #[serde(default)]
    #[ts(optional)]
    pub ipca: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SaleLeg {
    #[ts(type = "number")]
    pub gross: i64,
    #[ts(type = "number")]
    pub iof: i64,
    #[ts(type = "number")]
    pub ir: i64,
    #[ts(type = "number")]
    pub net: i64,
    #[ts(type = "number")]
    pub calendar_days: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FixedIncomeSale {
    pub ipca: RateAssumption,
    #[ts(type = "number")]
    pub business_days_elapsed: i64,
    #[ts(type = "number")]
    pub business_days_remaining: i64,
    /// Valor na curva (pela taxa contratada) na data da venda.
    #[ts(type = "number")]
    pub curve_value: i64,
    /// Preço de mercado na data da venda (pela taxa de mercado).
    #[ts(type = "number")]
    pub market_value: i64,
    /// Mercado − curva: positivo quando os juros caíram desde a compra.
    #[ts(type = "number")]
    pub mark_to_market: i64,
    /// Vender na data, pelo preço de mercado.
    pub sale: SaleLeg,
    /// Levar até o vencimento.
    pub hold: SaleLeg,
    /// Taxa (% a.a., mesmo indexador) que um título novo da data da venda até o vencimento
    /// precisa pagar para que o líquido da venda reaplicado empate com levar até o fim.
    pub breakeven_rate: Option<f64>,
    pub notes: Vec<String>,
}

fn leg(invested: i64, gross: i64, days: i64, exempt: bool) -> SaleLeg {
    let (iof, ir) = taxes(invested, gross, days, exempt);
    SaleLeg {
        gross,
        iof,
        ir,
        net: gross - iof - ir,
        calendar_days: days,
    }
}

pub fn simulate_sale(conn: &Connection, args: &FixedIncomeSaleArgs) -> Result<FixedIncomeSale> {
    if args.amount <= 0 {
        return Err(Error::invalid(t("nucleo-rf-erro-valor")));
    }
    let start = parse_date(&args.start)?;
    let maturity = parse_date(&args.maturity)?;
    let sale = match &args.sale_date {
        Some(s) => parse_date(s)?,
        None => today_local(),
    };
    if !(start < sale && sale < maturity) {
        return Err(Error::invalid(t("nucleo-rf-erro-datas-venda")));
    }
    for r in [args.contracted_rate, args.market_rate] {
        if !r.is_finite() || r <= -100.0 {
            return Err(Error::invalid(t("nucleo-rf-erro-taxa")));
        }
    }
    let ipca = match args.index {
        SaleIndex::Prefixed => pick(Some(0.0), None, 0.0),
        SaleIndex::IpcaPlus => assumptions(conn, Some(0.0), Some(0.0), args.ipca, Some(0.0))?.ipca,
    };
    let exempt = args.ir_exempt.unwrap_or(false);
    let du_total = business_days_between(start, maturity);
    let du_done = business_days_between(start, sale);
    let du_rest = du_total - du_done;
    let rc = 1.0 + args.contracted_rate / 100.0;
    let rm = 1.0 + args.market_rate / 100.0;
    let infl = 1.0 + ipca.value;
    let p = args.amount as f64;
    // Valor nominal atualizado: o aplicado corrigido pelo IPCA suposto até a venda (prefixado: o
    // próprio aplicado).
    let vna_sale = p * infl.powf(du_done as f64 / 252.0);
    let curve = cents(vna_sale * rc.powf(du_done as f64 / 252.0));
    let market =
        cents(vna_sale * rc.powf(du_total as f64 / 252.0) / rm.powf(du_rest as f64 / 252.0));
    let at_maturity =
        cents(p * infl.powf(du_total as f64 / 252.0) * rc.powf(du_total as f64 / 252.0));
    let sale_leg = leg(args.amount, market, (sale - start).num_days(), exempt);
    let hold_leg = leg(
        args.amount,
        at_maturity,
        (maturity - start).num_days(),
        exempt,
    );
    // Reaplicar o líquido da venda até o vencimento, com IR do zero (novo prazo).
    let rest_days = (maturity - sale).num_days();
    let reinvest = |rate: f64| -> i64 {
        let g = cents(
            sale_leg.net as f64
                * infl.powf(du_rest as f64 / 252.0)
                * (1.0 + rate / 100.0).powf(du_rest as f64 / 252.0),
        );
        leg(sale_leg.net, g, rest_days, exempt).net
    };
    let breakeven = if du_rest > 0 && sale_leg.net > 0 {
        let (mut lo, mut hi) = (-50.0f64, 200.0f64);
        if reinvest(lo) <= hold_leg.net && reinvest(hi) >= hold_leg.net {
            for _ in 0..80 {
                let mid = (lo + hi) / 2.0;
                if reinvest(mid) < hold_leg.net {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            Some((hi * 100.0).round() / 100.0)
        } else {
            None
        }
    } else {
        None
    };
    let mut notes = vec![t("nucleo-rf-nota-venda"), t("nucleo-rf-aviso")];
    if args.index == SaleIndex::IpcaPlus {
        notes.insert(0, t("nucleo-rf-nota-venda-ipca"));
    }
    Ok(FixedIncomeSale {
        ipca,
        business_days_elapsed: du_done,
        business_days_remaining: du_rest,
        curve_value: curve,
        market_value: market,
        mark_to_market: market - curve,
        sale: sale_leg,
        hold: hold_leg,
        breakeven_rate: breakeven,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(cdi: f64) -> RateAssumptions {
        let a = |v: f64| RateAssumption {
            value: v,
            source: "user".into(),
            date: None,
        };
        RateAssumptions {
            cdi: a(cdi),
            selic: a(cdi + 0.001),
            ipca: a(0.045),
            tr: a(0.0),
        }
    }

    fn offer(kind: FixedIncomeKind, index: OfferIndex, rate: f64, months: u32) -> FixedIncomeOffer {
        FixedIncomeOffer {
            label: None,
            kind,
            index,
            rate,
            end: None,
            months: Some(months),
            ir_exempt: None,
            custody: None,
        }
    }

    fn d(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn cdb_pays_regressive_ir_and_lci_is_exempt() {
        let a = fixed(0.14);
        let start = d("2026-11-03");
        let cdb = evaluate(
            &offer(FixedIncomeKind::Cdb, OfferIndex::CdiPct, 100.0, 24),
            1_000_000,
            start,
            &a,
        )
        .unwrap();
        assert_eq!(cdb.ir_rate, 0.15, "more than 720 days");
        assert_eq!(cdb.iof, 0);
        assert!(cdb.ir > 0);
        assert_eq!(cdb.net, cdb.gross - cdb.ir);
        assert_eq!(cdb.fgc, Some(true));
        let lci = evaluate(
            &offer(FixedIncomeKind::Lci, OfferIndex::CdiPct, 90.0, 24),
            1_000_000,
            start,
            &a,
        )
        .unwrap();
        assert!(lci.ir_exempt);
        assert_eq!(lci.ir, 0);
        assert_eq!(lci.net, lci.gross);
        // Equivalence: a CDB must pay more than 90% ÷ (1 − 15%) (not linear), but above
        // 100% of the CDI, and the CDB at that rate gives the same net as the LCI.
        let eq = lci.taxable_equivalent_pct_cdi.unwrap();
        assert!(eq > 100.0 && eq < 106.0, "{eq}");
        let du = lci.business_days;
        let days = lci.calendar_days;
        let back = net_at_pct_cdi(eq, false, 1_000_000, du, days, &a);
        assert!((back - lci.net).abs() < 200, "{back} × {}", lci.net);
        // And the CDB at 100% is worth an exempt one at less than 90% of the CDI.
        let ex = cdb.exempt_equivalent_pct_cdi.unwrap();
        assert!(ex > 80.0 && ex < 90.0, "{ex}");
        assert!((cdb.taxable_equivalent_pct_cdi.unwrap() - 100.0).abs() < 0.2);
    }

    #[test]
    fn short_redemption_pays_iof() {
        let a = fixed(0.14);
        let mut o = offer(FixedIncomeKind::Cdb, OfferIndex::CdiPct, 100.0, 1);
        o.months = None;
        o.end = Some("2026-11-13".into());
        let r = evaluate(&o, 1_000_000, d("2026-11-03"), &a).unwrap();
        assert_eq!(r.calendar_days, 10);
        assert!(r.iof > 0);
        assert_eq!(r.ir_rate, 0.225);
        assert_eq!(r.notes.len(), 1);
    }

    #[test]
    fn treasury_pays_custody_and_selic_has_the_exemption() {
        let a = fixed(0.14);
        let start = d("2026-11-03");
        let small = evaluate(
            &offer(
                FixedIncomeKind::TesouroSelic,
                OfferIndex::SelicPlus,
                0.0,
                12,
            ),
            500_000,
            start,
            &a,
        )
        .unwrap();
        assert_eq!(small.custody, 0, "up to R$ 10 thousand in Tesouro Selic");
        let pre = evaluate(
            &offer(
                FixedIncomeKind::TesouroPrefixado,
                OfferIndex::Prefixed,
                13.0,
                12,
            ),
            1_000_000,
            start,
            &a,
        )
        .unwrap();
        // 0.20% per year on the average value (~R$ 10,650) for one year ≈ R$ 21.
        assert!((2_000..2_300).contains(&pre.custody), "{}", pre.custody);
        assert_eq!(pre.fgc, Some(false));
        assert_eq!(pre.net, pre.gross - pre.ir - pre.custody);
    }

    #[test]
    fn savings_follows_the_selic_rule_on_anniversaries() {
        let mut a = fixed(0.14);
        assert!((savings_monthly(0.15, 0.0) - 0.005).abs() < 1e-12);
        let low = savings_monthly(0.08, 0.0);
        assert!((low - 0.7 * (1.08f64.powf(1.0 / 12.0) - 1.0)).abs() < 1e-12);
        a.tr.value = 0.001;
        let mut o = offer(FixedIncomeKind::Poupanca, OfferIndex::Savings, 0.0, 12);
        let r = evaluate(&o, 1_000_000, d("2026-01-10"), &a).unwrap();
        assert_eq!(r.gross, cents(1_000_000.0 * 1.006f64.powi(12)));
        assert!(r.ir_exempt && r.ir == 0);
        // Redemption one day before the 12th anniversary: only 11 months.
        o.months = None;
        o.end = Some("2027-01-09".into());
        let r = evaluate(&o, 1_000_000, d("2026-01-10"), &a).unwrap();
        assert_eq!(r.gross, cents(1_000_000.0 * 1.006f64.powi(11)));
        assert_eq!(r.notes.len(), 1);
    }

    #[test]
    fn rejects_bad_input() {
        let a = fixed(0.14);
        let mut o = offer(FixedIncomeKind::Cdb, OfferIndex::CdiPct, 100.0, 0);
        assert!(evaluate(&o, 1_000_000, d("2026-11-03"), &a).is_err());
        o.months = None;
        o.end = Some("2026-11-01".into());
        assert!(evaluate(&o, 1_000_000, d("2026-11-03"), &a).is_err());
    }

    #[test]
    fn sale_marks_to_market_and_finds_the_breakeven() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST)
            .unwrap();
        let conn = db.conn();
        // Fixed-rate at 12% bought in Jan 2026 until Jan 2030; sale in Jan 2028 with the market at
        // 10%: interest fell, the price rises above the curve.
        let args = FixedIncomeSaleArgs {
            amount: 1_000_000,
            start: "2026-01-05".into(),
            maturity: "2030-01-02".into(),
            sale_date: Some("2028-01-05".into()),
            index: SaleIndex::Prefixed,
            contracted_rate: 12.0,
            market_rate: 10.0,
            ir_exempt: None,
            ipca: None,
        };
        let s = simulate_sale(conn, &args).unwrap();
        assert!(s.mark_to_market > 0, "{s:?}");
        assert!(s.market_value > s.curve_value);
        assert_eq!(s.sale.net, s.sale.gross - s.sale.ir);
        assert_eq!(
            s.hold.gross,
            cents(
                1_000_000.0
                    * 1.12f64.powf(
                        s.business_days_elapsed as f64 / 252.0
                            + s.business_days_remaining as f64 / 252.0
                    )
            )
        );
        // With the market at 10%, reinvesting at 10% comes close to breaking even (the IR restarts
        // from zero, the
        // breakeven rate comes out a little above the market rate).
        let be = s.breakeven_rate.unwrap();
        assert!(be > 10.0 && be < 10.6, "{be}");
        // Market equal to the contracted one: price = curve.
        let same = simulate_sale(
            conn,
            &FixedIncomeSaleArgs {
                market_rate: 12.0,
                ..args.clone()
            },
        )
        .unwrap();
        assert!(same.mark_to_market.abs() <= 1);
        // Market above: loses on the sale.
        let up = simulate_sale(
            conn,
            &FixedIncomeSaleArgs {
                market_rate: 14.0,
                ..args.clone()
            },
        )
        .unwrap();
        assert!(up.mark_to_market < 0);
        // Dates out of order.
        let bad = FixedIncomeSaleArgs {
            sale_date: Some("2030-06-01".into()),
            ..args
        };
        assert!(simulate_sale(conn, &bad).is_err());
    }
}
