//! Come-cotas: o IR que os fundos abertos sujeitos à regra recolhem no último dia útil de
//! maio e de novembro, resgatando cotas do cotista (Lei 14.754/2023, art. 17 da IN RFB
//! 1.585/2015): 15% do rendimento nos fundos de longo prazo, 20% nos de curto prazo. Fundos
//! de ações, previdência, ETFs e FIIs não têm come-cotas.
//!
//! Por lote (PEPS por conta, como no resgate): rendimento = (cota do dia − cota base) ×
//! cotas do lote, só quando positivo; cotas resgatadas = IR ÷ cota do dia; a cota base do
//! lote passa a ser a do come-cotas (o rendimento já tributado não é tributado de novo).
//! Perdas não geram crédito aqui (a compensação acontece no resgate, fora do escopo).
//!
//! O resultado entra no livro do ativo como evento sintético: a quantidade cai e o custo
//! total fica (o preço médio sobe), sem resultado realizado.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::TradeSide;
use super::engine::TradeRow;
use crate::assets::{Asset, AssetClass};
use crate::error::{Error, Result};
use crate::finance::calendar::is_business_day;
use crate::i18n::{self, t, t_args};
use crate::time::fmt_date;

/// Ajuste com a regra escolhida por ativo (sobrepõe a automática).
pub const REGIME_SETTING: &str = "portfolio.come_cotas_regimes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ComeCotasRegime")]
pub enum Regime {
    /// Longo prazo (carteira com prazo médio acima de 365 dias): 15%.
    Long,
    /// Curto prazo: 20%.
    Short,
    /// Sem come-cotas (fundo de ações, previdência, ETF, FII…).
    Exempt,
}

impl Regime {
    pub fn rate(self) -> Decimal {
        match self {
            Regime::Long => Decimal::new(15, 2),
            Regime::Short => Decimal::new(20, 2),
            Regime::Exempt => Decimal::ZERO,
        }
    }
}

/// Regra de um ativo e de onde veio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ComeCotasRule")]
pub struct Rule {
    pub regime: Regime,
    /// `user` (escolhida), `cvm` (classificação do registro da CVM) ou `default`.
    pub origin: String,
    pub note: String,
}

/// Um come-cotas aplicado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ComeCotasEvent")]
pub struct Event {
    pub date: String,
    pub account_id: String,
    /// Cotas resgatadas para pagar o IR.
    pub shares: String,
    /// Cota do dia.
    pub price: String,
    /// Rendimento tributado e IR (moeda do ativo, unidade maior, decimal em texto).
    pub income: String,
    pub tax: String,
    pub rate: String,
}

/// Fundos sujeitos: a classe `fund` (fundos abertos da CVM).
pub fn applies_to(asset: &Asset) -> bool {
    asset.class == AssetClass::Fund
}

fn overrides(conn: &Connection) -> Result<BTreeMap<String, Regime>> {
    crate::settings::get_or(conn, REGIME_SETTING, BTreeMap::new())
}

/// Regra do ativo: a escolhida pelo usuário, senão a da classificação da CVM (ações não têm
/// come-cotas), senão longo prazo (a maioria dos fundos de renda fixa e multimercado).
pub fn rule(conn: &Connection, asset: &Asset) -> Result<Rule> {
    if !applies_to(asset) {
        return Ok(Rule {
            regime: Regime::Exempt,
            origin: "default".into(),
            note: t("nucleo-carteira-come-cotas-so-fundos"),
        });
    }
    if let Some(r) = overrides(conn)?.get(&asset.id) {
        return Ok(Rule {
            regime: *r,
            origin: "user".into(),
            note: t("nucleo-carteira-come-cotas-escolhida"),
        });
    }
    let cnpj = asset
        .cnpj
        .as_deref()
        .map(crate::research::cnpj_digits)
        .unwrap_or_default();
    let info = (!cnpj.is_empty())
        .then(|| crate::score::funds::registry::load(conn).ok())
        .flatten()
        .and_then(|r| r.funds.get(&cnpj).cloned());
    if let Some(f) = info {
        let class = f.classification.clone().unwrap_or_default();
        let norm = crate::research::norm(&class);
        if norm.contains("acoes") || f.kind.to_uppercase().starts_with("FII") {
            return Ok(Rule {
                regime: Regime::Exempt,
                origin: "cvm".into(),
                note: t_args(
                    "nucleo-carteira-come-cotas-cvm-isento",
                    &i18n::args(&[("classe", class.clone().into())]),
                ),
            });
        }
        if !class.is_empty() {
            return Ok(Rule {
                regime: Regime::Long,
                origin: "cvm".into(),
                note: t_args(
                    "nucleo-carteira-come-cotas-cvm-longo",
                    &i18n::args(&[("classe", class.clone().into())]),
                ),
            });
        }
    }
    Ok(Rule {
        regime: Regime::Long,
        origin: "default".into(),
        note: t("nucleo-carteira-come-cotas-sem-classificacao"),
    })
}

pub fn set_rule(conn: &Connection, asset_id: &str, regime: Option<Regime>) -> Result<()> {
    let mut map = overrides(conn)?;
    match regime {
        Some(r) => {
            map.insert(asset_id.to_string(), r);
        }
        None => {
            map.remove(asset_id);
        }
    }
    crate::settings::set(conn, REGIME_SETTING, &map)
}

/// Últimos dias úteis de maio e novembro entre `from` (exclusive) e `to` (inclusive).
pub fn dates(from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
    let mut out = vec![];
    for y in from.year()..=to.year() {
        for m in [5u32, 11] {
            let Some(first_next) = NaiveDate::from_ymd_opt(
                if m == 12 { y + 1 } else { y },
                if m == 12 { 1 } else { m + 1 },
                1,
            ) else {
                continue;
            };
            let mut d = first_next.pred_opt().expect("data válida");
            while !is_business_day(d) {
                d = d.pred_opt().expect("data válida");
            }
            if d > from && d <= to {
                out.push(d);
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
struct Lot {
    qty: Decimal,
    /// Cota base do rendimento (compra ou último come-cotas).
    base: Decimal,
}

/// Come-cotas das operações (todas as contas do livro), até `until`. `price` dá a cota do
/// dia (ou `None` sem cota: o come-cotas daquela data fica de fora, com aviso).
pub(crate) fn events(
    trades: &[TradeRow],
    regime: Regime,
    until: NaiveDate,
    mut price: impl FnMut(NaiveDate) -> Result<Option<Decimal>>,
    warnings: &mut Vec<String>,
) -> Result<Vec<Event>> {
    if regime == Regime::Exempt || trades.is_empty() {
        return Ok(vec![]);
    }
    let rate = regime.rate();
    let first = trades.iter().map(|t| t.date).min().expect("não vazio");
    let mut cc_dates = dates(first, until).into_iter().peekable();
    let mut lots: BTreeMap<String, Vec<Lot>> = BTreeMap::new();
    let mut out = vec![];
    let mut sorted: Vec<&TradeRow> = trades.iter().collect();
    sorted.sort_by_key(|t| t.date);
    let mut i = 0;
    loop {
        let next_trade = sorted.get(i).map(|t| t.date);
        let next_cc = cc_dates.peek().copied();
        // O come-cotas vale no fim do dia: operações do mesmo dia entram antes.
        let take_cc = match (next_trade, next_cc) {
            (_, None) => false,
            (None, Some(_)) => true,
            (Some(t), Some(c)) => c < t,
        };
        if take_cc {
            let d = cc_dates.next().expect("peek");
            let Some(p) = price(d)? else {
                if lots
                    .values()
                    .any(|l| l.iter().any(|x| x.qty > Decimal::ZERO))
                {
                    warnings.push(t_args(
                        "nucleo-carteira-come-cotas-sem-cota",
                        &i18n::args(&[("data", crate::time::fmt_date(d).into())]),
                    ));
                }
                continue;
            };
            for (acc, ls) in lots.iter_mut() {
                let mut income = Decimal::ZERO;
                for l in ls.iter_mut().filter(|l| l.qty > Decimal::ZERO) {
                    if p > l.base {
                        income += (p - l.base) * l.qty;
                    }
                }
                if income <= Decimal::ZERO {
                    continue;
                }
                let tax = income * rate;
                let shares = tax / p;
                // Cotas resgatadas proporcionalmente ao IR de cada lote; base = cota do dia.
                for l in ls.iter_mut().filter(|l| l.qty > Decimal::ZERO) {
                    if p > l.base {
                        let lot_tax = (p - l.base) * l.qty * rate;
                        l.qty -= lot_tax / p;
                        l.base = p;
                    }
                }
                out.push(Event {
                    date: fmt_date(d),
                    account_id: acc.clone(),
                    shares: shares.round_dp(8).normalize().to_string(),
                    price: p.normalize().to_string(),
                    income: income.round_dp(2).normalize().to_string(),
                    tax: tax.round_dp(2).normalize().to_string(),
                    rate: rate.normalize().to_string(),
                });
            }
            continue;
        }
        let Some(t) = sorted.get(i) else { break };
        i += 1;
        if t.date > until {
            break;
        }
        let ls = lots.entry(t.account_id.clone()).or_default();
        match t.side {
            TradeSide::Buy => ls.push(Lot {
                qty: t.qty,
                base: t.price,
            }),
            TradeSide::Sell => {
                let mut left = t.qty;
                for l in ls.iter_mut() {
                    if left <= Decimal::ZERO {
                        break;
                    }
                    let take = left.min(l.qty);
                    l.qty -= take;
                    left -= take;
                }
                ls.retain(|l| l.qty > Decimal::ZERO);
            }
        }
    }
    Ok(out)
}

/// Come-cotas de um ativo cadastrado (todas as contas ou uma), com a regra usada.
pub fn for_asset(
    conn: &Connection,
    asset_id: &str,
    account_id: Option<&str>,
    until: NaiveDate,
) -> Result<(Rule, Vec<Event>, Vec<String>)> {
    let asset = crate::assets::get(conn, asset_id)?;
    let rule = rule(conn, &asset)?;
    let accounts = account_id.map(|a| vec![a.to_string()]);
    let trades = super::engine::load_trades_pub(conn, asset_id, accounts.as_deref())?;
    let mut warnings = vec![];
    let evs = events(
        &trades,
        rule.regime,
        until,
        |d| super::engine::fund_quota_on(conn, asset_id, d),
        &mut warnings,
    )?;
    Ok((rule, evs, warnings))
}

pub(crate) fn parse(v: &str) -> Result<Decimal> {
    v.parse().map_err(|_| {
        Error::invalid(t_args(
            "nucleo-carteira-come-cotas-erro-valor",
            &i18n::args(&[("valor", v.to_string().into())]),
        ))
    })
}
