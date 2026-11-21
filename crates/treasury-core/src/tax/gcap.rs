//! Ganho de capital na venda de imóveis, bens móveis e participações fora de bolsa (pessoa
//! física), como o programa GCAP da Receita calcula:
//!
//! - ganho bruto = valor de venda − corretagem paga − custo de aquisição (com benfeitorias e
//!   custos de compra, como ITBI e escritura);
//! - imóveis: redução da Lei 7.713/1988 art. 18 para os adquiridos até 1988 (100% até 1969,
//!   95% em 1970 e 5 pontos a menos por ano, 5% em 1988), depois os fatores da Lei
//!   11.196/2005 art. 40: FR1 = 1/1,0060^m1 (meses de jan/1996, ou da compra, até nov/2005,
//!   só para compras até nov/2005) e FR2 = 1/1,0035^m2 (meses de dez/2005, ou da compra, até a
//!   venda), cada um sobre o que sobrou do anterior, contando os dois meses das pontas;
//! - isenções: único imóvel vendido por até R$ 440 mil sem outra venda de imóvel em 5 anos
//!   (Lei 9.250 art. 23); bens de pequeno valor, com vendas de bens da mesma natureza no mês
//!   até R$ 35 mil (art. 22); imóvel residencial com o dinheiro aplicado em outro imóvel
//!   residencial em 180 dias, na proporção do que foi aplicado (Lei 11.196 art. 39);
//! - imposto: 15% até R$ 5 milhões, 17,5% até 10, 20% até 30 e 22,5% acima, por faixa (Lei
//!   8.981/1995 art. 21), em DARF 4600 até o último dia útil do mês seguinte.
//!
//! Percentuais com 6 casas e valores truncados ao centavo, como o programa; conferido nos
//! casos de referência do GCAP em `tests/fixtures/ir-br/dirpf-2026`.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::rules::TaxRules;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "GainAssetKind")]
pub enum AssetKind {
    /// Imóvel (casa, apartamento, terreno, sala).
    RealEstate,
    /// Bem móvel (veículo, obra de arte, joia, equipamento).
    Movable,
    /// Quotas ou ações fora de bolsa.
    Holding,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CapitalGainInput")]
pub struct Input {
    pub kind: AssetKind,
    pub acquired_on: String,
    pub sold_on: String,
    #[ts(type = "number")]
    pub sale_value: i64,
    /// Corretagem paga por quem vende.
    #[serde(default)]
    #[ts(type = "number")]
    pub brokerage: i64,
    /// Custo de aquisição com benfeitorias e custos da compra.
    #[ts(type = "number")]
    pub cost: i64,
    /// Único imóvel, sem outra venda de imóvel nos últimos 5 anos.
    #[serde(default)]
    pub only_property: bool,
    /// Total vendido no mês em bens da mesma natureza (padrão: esta venda).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub month_sales_same_kind: Option<i64>,
    /// Imóvel residencial: valor aplicado em outro imóvel residencial em até 180 dias.
    #[serde(default)]
    #[ts(type = "number")]
    pub reinvested: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CapitalGainReductions")]
pub struct Reductions {
    #[ts(type = "number")]
    pub lei_7713: i64,
    #[ts(type = "number")]
    pub fr1: i64,
    #[ts(type = "number")]
    pub fr2: i64,
    #[ts(type = "number")]
    pub only_property: i64,
    #[ts(type = "number")]
    pub reinvestment: i64,
    /// Percentuais aplicados (texto com 6 casas): Lei 7.713, FR1, FR2.
    pub lei_7713_pct: String,
    pub fr1_pct: String,
    pub fr2_pct: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CapitalGain")]
pub struct Output {
    #[ts(type = "number")]
    pub net_value: i64,
    #[ts(type = "number")]
    pub gross_gain: i64,
    pub reductions: Reductions,
    /// Ganho depois das reduções (base do imposto, se não for de pequeno valor).
    #[ts(type = "number")]
    pub taxable_gain: i64,
    /// Isento por pequeno valor (vendas do mês até R$ 35 mil).
    pub small_value: bool,
    #[ts(type = "number")]
    pub tax: i64,
    /// Parte do ganho que fica isenta (reduções e isenções).
    #[ts(type = "number")]
    pub exempt: i64,
    pub revenue_code: String,
    pub due_date: String,
}

/// Limite de pequeno valor (vendas no mês de bens da mesma natureza).
const SMALL_VALUE: i64 = 3_500_000;
/// Limite do único imóvel.
const ONLY_PROPERTY: i64 = 44_000_000;

/// Meses de `from` a `to`, contando os dois.
fn months_inclusive(from: NaiveDate, to: NaiveDate) -> i64 {
    ((to.year() - from.year()) * 12 + to.month() as i32 - from.month() as i32 + 1).max(0) as i64
}

/// Percentual de redução de um fator 1/base^m, com 6 casas (como o programa).
fn reduction_pct(base: f64, months: i64) -> Decimal {
    if months <= 0 {
        return Decimal::ZERO;
    }
    let pct = (1.0 - 1.0 / base.powi(months as i32)) * 100.0;
    Decimal::from_f64(pct)
        .unwrap_or_default()
        .round_dp_with_strategy(6, RoundingStrategy::MidpointNearestEven)
}

fn apply(v: i64, pct: Decimal) -> i64 {
    (Decimal::from(v) * pct / Decimal::ONE_HUNDRED)
        .trunc()
        .to_i64()
        .unwrap_or(0)
}

fn fmt_pct(p: Decimal) -> String {
    format!("{:.6}", p)
}

pub fn compute(i: &Input) -> crate::error::Result<Output> {
    let acquired = crate::time::parse_date(&i.acquired_on)?;
    let sold = crate::time::parse_date(&i.sold_on)?;
    let r = TaxRules::for_year(sold.year());
    let net_value = i.sale_value - i.brokerage.max(0);
    let gross_gain = (net_value - i.cost).max(0);
    let mut red = Reductions::default();
    let mut gain = gross_gain;
    if i.kind == AssetKind::RealEstate {
        // Lei 7.713/1988 art. 18: imóveis adquiridos até 1988.
        let y = acquired.year();
        let pct = if y <= 1969 {
            Decimal::from(100)
        } else if y <= 1988 {
            Decimal::from(100 - 5 * (y - 1969))
        } else {
            Decimal::ZERO
        };
        red.lei_7713 = apply(gain, pct);
        red.lei_7713_pct = fmt_pct(pct);
        gain -= red.lei_7713;
        // FR1: de jan/1996 (ou da compra) a nov/2005, só para compras até nov/2005.
        let nov_2005 = NaiveDate::from_ymd_opt(2005, 11, 30).expect("data");
        let pct1 = if acquired <= nov_2005 {
            let start = acquired.max(NaiveDate::from_ymd_opt(1996, 1, 1).expect("data"));
            reduction_pct(1.006, months_inclusive(start, nov_2005))
        } else {
            Decimal::ZERO
        };
        red.fr1 = apply(gain, pct1);
        red.fr1_pct = fmt_pct(pct1);
        gain -= red.fr1;
        // FR2: de dez/2005 (ou da compra) até a venda.
        let start = acquired.max(NaiveDate::from_ymd_opt(2005, 12, 1).expect("data"));
        let pct2 = reduction_pct(1.0035, months_inclusive(start, sold));
        red.fr2 = apply(gain, pct2);
        red.fr2_pct = fmt_pct(pct2);
        gain -= red.fr2;
        if i.only_property && i.sale_value <= ONLY_PROPERTY {
            red.only_property = gain;
            gain = 0;
        } else if i.reinvested > 0 && i.sale_value > 0 {
            let share = Decimal::from(i.reinvested.min(i.sale_value)) / Decimal::from(i.sale_value);
            red.reinvestment = (Decimal::from(gain) * share).trunc().to_i64().unwrap_or(0);
            gain -= red.reinvestment;
        }
    }
    let month_sales = i.month_sales_same_kind.unwrap_or(i.sale_value);
    let small_value = month_sales <= SMALL_VALUE;
    let tax = if small_value || gain <= 0 {
        0
    } else {
        let mut prev = 0i64;
        let mut tax = Decimal::ZERO;
        for (up_to, rate) in r.capital_gain_bands {
            let top = up_to.unwrap_or(i64::MAX);
            let slice = gain.min(top) - prev;
            if slice <= 0 {
                break;
            }
            tax += Decimal::from(slice) * rate;
            prev = top;
        }
        tax.trunc().to_i64().unwrap_or(0)
    };
    let taxable_gain = gain;
    let exempt = gross_gain - if small_value { 0 } else { taxable_gain };
    Ok(Output {
        net_value,
        gross_gain,
        reductions: red,
        taxable_gain,
        small_value,
        tax,
        exempt,
        revenue_code: r.capital_gain_darf_code.into(),
        due_date: crate::time::fmt_date(super::engine::darf_due_date(&crate::time::month_of(
            sold,
        ))?),
    })
}

/// Ganho de capital de um bem do 02 · Bens: o custo é o da ficha (compra, custos da compra e
/// benfeitorias); a venda vem do bem vendido ou, para simular, dos argumentos.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhysicalGainArgs {
    pub asset_id: String,
    #[serde(default)]
    #[ts(optional)]
    pub sale_date: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub sale_price: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub sale_costs: Option<i64>,
    #[serde(default)]
    pub only_property: bool,
    #[serde(default)]
    #[ts(type = "number")]
    pub reinvested: i64,
}

pub fn for_physical(
    conn: &rusqlite::Connection,
    a: &PhysicalGainArgs,
) -> crate::error::Result<Output> {
    use crate::physical::PhysicalKind as K;
    let v = crate::physical::view(
        conn,
        &a.asset_id,
        &crate::physical::PhysicalReadArgs {
            as_of: a.sale_date.clone(),
            deflator: None,
            include_archived: Some(true),
        },
    )?;
    let (Some(sold_on), Some(price)) = (
        a.sale_date.clone().or(v.sale_date.clone()),
        a.sale_price.or(v.sale_price),
    ) else {
        return Err(crate::error::Error::invalid(crate::i18n::t(
            "nucleo-impostos-gcap-erro-venda",
        )));
    };
    compute(&Input {
        kind: match v.kind {
            K::RealEstate | K::Land => AssetKind::RealEstate,
            _ => AssetKind::Movable,
        },
        acquired_on: v.purchase_date.clone(),
        sold_on,
        sale_value: price,
        brokerage: a.sale_costs.unwrap_or(v.sale_costs),
        cost: v.figures.cost_basis,
        only_property: a.only_property,
        month_sales_same_kind: None,
        reinvested: a.reinvested,
    })
}
