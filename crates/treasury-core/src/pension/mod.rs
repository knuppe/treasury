//! Previdência privada (VGBL e PGBL): o plano de um ativo da classe `pension`, os aportes
//! com a idade de cada um e a simulação de resgate com o IR do regime escolhido.
//!
//! - Aportes e resgates são as operações do ativo (cotas do fundo FIE); os resgates consomem
//!   os aportes mais antigos primeiro (PEPS), como fazem as seguradoras.
//! - O valor de hoje vem da última cota (coleta da CVM pelo CNPJ do fundo ou cota informada);
//!   sem cota, o preço do último aporte.
//! - **VGBL**: o IR incide só sobre o rendimento. **PGBL**: sobre o valor todo resgatado
//!   (os aportes foram deduzidos na declaração).
//! - **Regressivo** (Lei 11.053/2004): 35% até 2 anos de cada aporte, caindo 5 pontos a cada
//!   2 anos até 10% depois de 10 anos; é definitivo. **Progressivo**: 15% retidos na fonte e
//!   ajuste pela tabela anual na declaração.

pub mod api;
pub mod payments;
pub mod statement;
#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::i18n::{self, args, t, t_args};
use crate::ids::Id;
use crate::time::{add_months, fmt_date, now_utc, parse_date};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PlanType {
    Vgbl,
    Pgbl,
    /// Previdência fechada (fundo de pensão, EFPC): no IR, como o PGBL.
    Efpc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TaxRegime {
    Progressive,
    Regressive,
}

/// Plano de previdência de um ativo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionPlan {
    pub asset_id: Id,
    pub plan_type: PlanType,
    pub tax_regime: TaxRegime,
    pub insurer: Option<String>,
    /// Taxa de administração % a.a.
    pub admin_fee_pct: Option<String>,
    /// Taxa de carregamento % sobre os aportes.
    pub loading_fee_pct: Option<String>,
    pub start_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionPlanInput {
    pub asset_id: Id,
    pub plan_type: PlanType,
    pub tax_regime: TaxRegime,
    #[serde(default)]
    #[ts(optional)]
    pub insurer: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub admin_fee_pct: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub loading_fee_pct: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub start_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub notes: Option<String>,
}

/// Mudança parcial do plano (`assets.update`): o que vier substitui, o resto fica como está
/// (sem plano gravado, VGBL regressivo).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionPlanPatch {
    #[serde(default)]
    #[ts(optional)]
    pub plan_type: Option<PlanType>,
    #[serde(default)]
    #[ts(optional)]
    pub tax_regime: Option<TaxRegime>,
    #[serde(default)]
    #[ts(optional)]
    pub insurer: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub admin_fee_pct: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub loading_fee_pct: Option<String>,
}

/// Aplica a mudança parcial ao plano do ativo.
pub fn patch_plan(
    tx: &Transaction<'_>,
    asset_id: &str,
    p: &PensionPlanPatch,
) -> Result<PensionPlan> {
    let cur = get_plan(tx, asset_id)?;
    let keep = |new: &Option<String>, old: Option<String>| new.clone().or(old);
    set_plan(
        tx,
        &PensionPlanInput {
            asset_id: asset_id.to_string(),
            plan_type: p
                .plan_type
                .or(cur.as_ref().map(|c| c.plan_type))
                .unwrap_or(PlanType::Vgbl),
            tax_regime: p
                .tax_regime
                .or(cur.as_ref().map(|c| c.tax_regime))
                .unwrap_or(TaxRegime::Regressive),
            insurer: keep(&p.insurer, cur.as_ref().and_then(|c| c.insurer.clone())),
            admin_fee_pct: keep(
                &p.admin_fee_pct,
                cur.as_ref().and_then(|c| c.admin_fee_pct.clone()),
            ),
            loading_fee_pct: keep(
                &p.loading_fee_pct,
                cur.as_ref().and_then(|c| c.loading_fee_pct.clone()),
            ),
            start_date: cur.as_ref().and_then(|c| c.start_date.clone()),
            notes: cur.as_ref().and_then(|c| c.notes.clone()),
        },
    )
}

/// Plano informado junto com o cadastro do ativo (`assets.create`), sem o id do ativo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionPlanTerms {
    pub plan_type: PlanType,
    pub tax_regime: TaxRegime,
    #[serde(default)]
    #[ts(optional)]
    pub insurer: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub admin_fee_pct: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub loading_fee_pct: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub start_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub notes: Option<String>,
}

impl PensionPlanTerms {
    pub fn for_asset(&self, asset_id: &str) -> PensionPlanInput {
        PensionPlanInput {
            asset_id: asset_id.to_string(),
            plan_type: self.plan_type,
            tax_regime: self.tax_regime,
            insurer: self.insurer.clone(),
            admin_fee_pct: self.admin_fee_pct.clone(),
            loading_fee_pct: self.loading_fee_pct.clone(),
            start_date: self.start_date.clone(),
            notes: self.notes.clone(),
        }
    }
}

/// Um aporte ainda no plano (o que sobrou dele depois dos resgates).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionLot {
    pub date: String,
    /// Cotas que restam do aporte.
    pub quantity: String,
    /// Quanto foi aportado (o que resta), em centavos.
    #[ts(type = "number")]
    pub cost: i64,
    /// Valor hoje, em centavos.
    #[ts(type = "number")]
    pub value: i64,
    #[ts(type = "number")]
    pub gain: i64,
    /// Alíquota do regressivo hoje (% ), mesmo no progressivo (para comparar).
    pub regressive_rate_pct: u32,
    /// Quando o aporte cai para a próxima faixa (sem data: já está em 10%).
    pub next_rate_pct: Option<u32>,
    pub next_rate_date: Option<String>,
}

/// Resgate simulado.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WithdrawalSim {
    /// Valor bruto resgatado, em centavos.
    #[ts(type = "number")]
    pub gross: i64,
    /// Base do IR: o rendimento (VGBL) ou o valor todo (PGBL).
    #[ts(type = "number")]
    pub tax_base: i64,
    /// IR retido (definitivo no regressivo; 15% na fonte no progressivo).
    #[ts(type = "number")]
    pub tax: i64,
    #[ts(type = "number")]
    pub net: i64,
    /// Alíquota efetiva sobre o bruto, em %.
    pub effective_rate_pct: String,
    /// Como o IR foi calculado, em uma frase.
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PensionView {
    pub asset_id: Id,
    pub asset_name: String,
    pub plan: Option<PensionPlan>,
    pub lots: Vec<PensionLot>,
    /// Aportado (o que resta), valor hoje e rendimento, em centavos.
    #[ts(type = "number")]
    pub contributed: i64,
    #[ts(type = "number")]
    pub value: i64,
    #[ts(type = "number")]
    pub gain: i64,
    /// Cota usada no valor de hoje e a data dela.
    pub price: Option<String>,
    pub price_date: Option<String>,
    /// Resgate de tudo hoje.
    pub full_withdrawal: WithdrawalSim,
    /// Aportes que mudam de faixa nos próximos 90 dias (valor, em centavos).
    #[ts(type = "number")]
    pub changing_band_soon: i64,
    pub warnings: Vec<String>,
    /// Contas que guardam as cotas do plano (o plano é o ativo; a conta é a custódia).
    #[serde(default)]
    pub accounts: Vec<PlanAccount>,
    /// Aportes conhecidos (data e valor) ainda sem cota da CVM: entram na posição quando a
    /// coleta trouxer a cota.
    #[serde(default)]
    pub pending_payments: u32,
    #[serde(default)]
    #[ts(type = "number")]
    pub pending_amount: i64,
    #[serde(default)]
    pub pending_since: Option<String>,
    /// Aportes do histórico de pagamentos (com e sem cota), do mais recente ao mais antigo.
    #[serde(default)]
    pub payments: Vec<payments::PaymentRecord>,
    /// O que os prints não fecharam (valor ilegível, linha cortada, mês que falta…).
    #[serde(default)]
    pub payment_issues: Vec<payments::Issue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "PensionPlanAccount")]
pub struct PlanAccount {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WithdrawalArgs {
    pub asset_id: Id,
    /// Valor bruto a resgatar, em centavos (sem ele, tudo).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    /// Data do resgate (padrão: hoje).
    #[serde(default)]
    #[ts(optional)]
    pub date: Option<String>,
}

fn regime_str(r: TaxRegime) -> &'static str {
    match r {
        TaxRegime::Progressive => "progressive",
        TaxRegime::Regressive => "regressive",
    }
}

fn type_str(t: PlanType) -> &'static str {
    match t {
        PlanType::Vgbl => "vgbl",
        PlanType::Pgbl => "pgbl",
        PlanType::Efpc => "efpc",
    }
}

/// Alíquota do regressivo pela idade do aporte, e a próxima (com a data em que muda).
///
/// Lei 11.053/2004 art. 1º: 35% para prazo de acumulação "inferior ou igual a 2 anos", 30% para
/// "superior a 2 e inferior ou igual a 4", e assim de 2 em 2 anos até 10% acima de 10 anos. No
/// dia em que o aporte completa 2 anos ainda vale 35%; a faixa seguinte começa no dia depois.
pub fn regressive_rate(contributed: NaiveDate, at: NaiveDate) -> (u32, Option<(u32, NaiveDate)>) {
    for (k, rate) in [35u32, 30, 25, 20, 15].iter().enumerate() {
        let until = add_months(contributed, 24 * (k as i32 + 1));
        if at <= until {
            return (*rate, Some((rate - 5, until + chrono::Duration::days(1))));
        }
    }
    (10, None)
}

fn opt_dec(v: Option<&str>, what: &str) -> Result<Option<String>> {
    match v.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => {
            let d: Decimal = s.replace(',', ".").parse().map_err(|_| {
                Error::invalid(t_args(
                    "nucleo-planejamento-previdencia-erro-taxa-invalida",
                    &args(&[("taxa", t(what).into()), ("valor", s.to_string().into())]),
                ))
            })?;
            if d < Decimal::ZERO || d > Decimal::from(100) {
                return Err(Error::invalid(t_args(
                    "nucleo-planejamento-previdencia-erro-taxa-faixa",
                    &args(&[("taxa", t(what).into())]),
                )));
            }
            Ok(Some(d.normalize().to_string()))
        }
    }
}

/// Plano de fundo de pensão (EFPC, fiscalizado pela Previc): pelo tipo do plano ou pela
/// natureza jurídica do CNPJ lida na Receita ("Plano de Benefícios de Previdência Complementar
/// Fechada", consulta de referência). Não é fundo da CVM: as cotas só vêm do extrato.
pub fn is_efpc(conn: &Connection, asset_id: &str) -> Result<bool> {
    if get_plan(conn, asset_id)?.is_some_and(|p| p.plan_type == PlanType::Efpc) {
        return Ok(true);
    }
    let seen: Option<String> = conn
        .query_row(
            "SELECT value FROM asset_observation WHERE asset_id = ?1 AND field = 'pension_type'
              ORDER BY observed_at DESC LIMIT 1",
            [asset_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(seen.as_deref() == Some("efpc"))
}

pub fn get_plan(conn: &Connection, asset_id: &str) -> Result<Option<PensionPlan>> {
    Ok(conn
        .query_row(
            "SELECT plan_type, tax_regime, insurer, admin_fee_pct, loading_fee_pct, start_date, notes
               FROM pension_plan WHERE asset_id = ?1",
            [asset_id],
            |r| {
                Ok(PensionPlan {
                    asset_id: asset_id.to_string(),
                    plan_type: match r.get::<_, String>(0)?.as_str() {
                        "pgbl" => PlanType::Pgbl,
                        "efpc" => PlanType::Efpc,
                        _ => PlanType::Vgbl,
                    },
                    tax_regime: if r.get::<_, String>(1)? == "progressive" {
                        TaxRegime::Progressive
                    } else {
                        TaxRegime::Regressive
                    },
                    insurer: r.get(2)?,
                    admin_fee_pct: r.get(3)?,
                    loading_fee_pct: r.get(4)?,
                    start_date: r.get(5)?,
                    notes: r.get(6)?,
                })
            },
        )
        .optional()?)
}

pub fn set_plan(tx: &Transaction<'_>, i: &PensionPlanInput) -> Result<PensionPlan> {
    let a = crate::assets::get(tx, &i.asset_id)?;
    if a.class != AssetClass::Pension {
        return Err(Error::invalid(t(
            "nucleo-planejamento-previdencia-erro-classe",
        )));
    }
    if let Some(d) = i.start_date.as_deref().filter(|d| !d.trim().is_empty()) {
        parse_date(d)?;
    }
    let clean = |s: &Option<String>| {
        s.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    let now = now_utc();
    tx.execute(
        "INSERT INTO pension_plan (asset_id, plan_type, tax_regime, insurer, admin_fee_pct, loading_fee_pct, start_date, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
         ON CONFLICT (asset_id) DO UPDATE SET plan_type = ?2, tax_regime = ?3, insurer = ?4, admin_fee_pct = ?5,
              loading_fee_pct = ?6, start_date = ?7, notes = ?8, updated_at = ?9",
        params![
            i.asset_id,
            type_str(i.plan_type),
            regime_str(i.tax_regime),
            clean(&i.insurer),
            opt_dec(
                i.admin_fee_pct.as_deref(),
                "nucleo-planejamento-previdencia-taxa-administracao"
            )?,
            opt_dec(
                i.loading_fee_pct.as_deref(),
                "nucleo-planejamento-previdencia-taxa-carregamento"
            )?,
            clean(&i.start_date),
            clean(&i.notes),
            now
        ],
    )?;
    get_plan(tx, &i.asset_id)?
        .ok_or_else(|| Error::not_found(t("nucleo-planejamento-previdencia-plano")))
}

struct Lot {
    date: NaiveDate,
    qty: Decimal,
    /// Custo por cota (preço + taxas rateadas).
    unit_cost: Decimal,
}

/// Aportes que restam até `at`, em ordem (os resgates consomem os mais antigos).
fn lots(conn: &Connection, asset_id: &str, at: NaiveDate) -> Result<(Vec<Lot>, Option<Decimal>)> {
    let mut stmt = conn.prepare(
        "SELECT date, side, quantity, price, fees FROM trade
          WHERE asset_id = ?1 AND date <= ?2 ORDER BY date, side, created_at",
    )?;
    let rows: Vec<(String, String, String, String, i64)> = stmt
        .query_map(params![asset_id, fmt_date(at)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut out: Vec<Lot> = vec![];
    let mut last_price = None;
    for (d, side, q, p, fees) in rows {
        let (Ok(qty), Ok(price)) = (q.parse::<Decimal>(), p.parse::<Decimal>()) else {
            continue;
        };
        last_price = Some(price);
        if qty <= Decimal::ZERO {
            continue;
        }
        if side == "buy" {
            let unit_cost = price + Decimal::from(fees) / Decimal::from(100) / qty;
            out.push(Lot {
                date: parse_date(&d)?,
                qty,
                unit_cost,
            });
        } else {
            let mut left = qty;
            for l in out.iter_mut() {
                if left.is_zero() {
                    break;
                }
                let take = l.qty.min(left);
                l.qty -= take;
                left -= take;
            }
            out.retain(|l| !l.qty.is_zero());
        }
    }
    Ok((out, last_price))
}

fn cents(v: Decimal) -> i64 {
    (v * Decimal::from(100)).round().to_i64().unwrap_or(0)
}

/// Cota mais recente até a data (coleta ou informada).
fn price_at(conn: &Connection, asset_id: &str, at: NaiveDate) -> Result<Option<(Decimal, String)>> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT close, date FROM price WHERE asset_id = ?1 AND date <= ?2 ORDER BY date DESC LIMIT 1",
            params![asset_id, fmt_date(at)],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.and_then(|(c, d)| c.parse::<Decimal>().ok().map(|p| (p, d))))
}

/// Resgate dos aportes mais antigos primeiro, com o IR de cada pedaço.
fn simulate(
    lots: &[Lot],
    price: Decimal,
    plan: Option<&PensionPlan>,
    amount: Option<i64>,
    at: NaiveDate,
) -> WithdrawalSim {
    let plan_type = plan.map_or(PlanType::Vgbl, |p| p.plan_type);
    let regime = plan.map_or(TaxRegime::Regressive, |p| p.tax_regime);
    let mut left = amount.map(|a| Decimal::from(a.max(0)) / Decimal::from(100));
    let (mut gross, mut base, mut tax) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
    for l in lots {
        let value = l.qty * price;
        if value <= Decimal::ZERO {
            continue;
        }
        let part = match left {
            Some(rest) if rest <= Decimal::ZERO => break,
            Some(rest) => rest.min(value),
            None => value,
        };
        let frac = part / value;
        let gain = ((price - l.unit_cost) * l.qty * frac).max(Decimal::ZERO);
        let b = match plan_type {
            PlanType::Vgbl => gain,
            PlanType::Pgbl | PlanType::Efpc => part,
        };
        let rate = match regime {
            TaxRegime::Regressive => Decimal::from(regressive_rate(l.date, at).0),
            TaxRegime::Progressive => Decimal::from(15),
        };
        gross += part;
        base += b;
        tax += b * rate / Decimal::from(100);
        if let Some(rest) = left.as_mut() {
            *rest -= part;
        }
    }
    let (g, b, t) = (cents(gross), cents(base), cents(tax));
    let eff = if g > 0 {
        (Decimal::from(t) * Decimal::from(100) / Decimal::from(g)).round_dp(2)
    } else {
        Decimal::ZERO
    };
    let note = i18n::t(match (plan_type, regime) {
        (PlanType::Vgbl, TaxRegime::Regressive) => {
            "nucleo-planejamento-previdencia-nota-vgbl-regressivo"
        }
        (PlanType::Pgbl, TaxRegime::Regressive) => {
            "nucleo-planejamento-previdencia-nota-pgbl-regressivo"
        }
        (PlanType::Vgbl, TaxRegime::Progressive) => {
            "nucleo-planejamento-previdencia-nota-vgbl-progressivo"
        }
        (PlanType::Efpc, TaxRegime::Regressive) => {
            "nucleo-planejamento-previdencia-nota-efpc-regressivo"
        }
        (PlanType::Efpc, TaxRegime::Progressive) => {
            "nucleo-planejamento-previdencia-nota-efpc-progressivo"
        }
        (PlanType::Pgbl, TaxRegime::Progressive) => {
            "nucleo-planejamento-previdencia-nota-pgbl-progressivo"
        }
    });
    WithdrawalSim {
        gross: g,
        tax_base: b,
        tax: t,
        net: g - t,
        effective_rate_pct: eff.normalize().to_string(),
        note,
    }
}

pub fn view(conn: &Connection, asset_id: &str, at: NaiveDate) -> Result<PensionView> {
    let a = crate::assets::get(conn, asset_id)?;
    let plan = get_plan(conn, asset_id)?;
    let (lots, last_trade_price) = lots(conn, asset_id, at)?;
    let mut warnings = vec![];
    if plan.is_none() {
        warnings.push(t("nucleo-planejamento-previdencia-aviso-sem-plano"));
    }
    let (price, price_date) = match price_at(conn, asset_id, at)? {
        Some((p, d)) => (Some(p), Some(d)),
        None => {
            if !lots.is_empty() {
                warnings.push(t("nucleo-planejamento-previdencia-aviso-sem-cota"));
            }
            (last_trade_price, None)
        }
    };
    let p = price.unwrap_or(Decimal::ZERO);
    let soon = add_months(at, 3);
    let mut changing = 0i64;
    let out: Vec<PensionLot> = lots
        .iter()
        .map(|l| {
            let value = cents(l.qty * p);
            let cost = cents(l.qty * l.unit_cost);
            let (rate, next) = regressive_rate(l.date, at);
            if let Some((_, d)) = next
                && d <= soon
            {
                changing += value;
            }
            PensionLot {
                date: fmt_date(l.date),
                quantity: l.qty.normalize().to_string(),
                cost,
                value,
                gain: value - cost,
                regressive_rate_pct: rate,
                next_rate_pct: next.map(|(r, _)| r),
                next_rate_date: next.map(|(_, d)| fmt_date(d)),
            }
        })
        .collect();
    let contributed: i64 = out.iter().map(|l| l.cost).sum();
    let value: i64 = out.iter().map(|l| l.value).sum();
    let (pending_payments, pending_amount, pending_since) =
        payments::pending_summary(conn, asset_id)?;
    let accounts = payments::plan_accounts(conn, asset_id)?
        .into_iter()
        .map(|(id, name)| PlanAccount { id, name })
        .collect();
    if pending_payments > 0 {
        warnings.push(t_args(
            "nucleo-planejamento-previdencia-aviso-aportes-sem-cota",
            &args(&[("n", pending_payments.into())]),
        ));
    }
    Ok(PensionView {
        accounts,
        pending_payments,
        pending_amount,
        pending_since,
        payments: payments::records(conn, asset_id)?,
        payment_issues: payments::saved_issues(conn, asset_id)?,
        asset_id: a.id,
        asset_name: a.name,
        full_withdrawal: simulate(&lots, p, plan.as_ref(), None, at),
        plan,
        lots: out,
        // Os aportes ainda sem cota entram pelo valor aportado (sem rendimento).
        contributed: contributed + pending_amount,
        value: value + pending_amount,
        gain: value - contributed,
        price: price.map(|p| p.normalize().to_string()),
        price_date,
        changing_band_soon: if matches!(
            get_plan(conn, asset_id)?.map(|p| p.tax_regime),
            None | Some(TaxRegime::Regressive)
        ) {
            changing
        } else {
            0
        },
        warnings,
    })
}

pub fn simulate_withdrawal(
    conn: &Connection,
    w: &WithdrawalArgs,
    today: NaiveDate,
) -> Result<WithdrawalSim> {
    let at = match w.date.as_deref() {
        Some(d) => parse_date(d)?,
        None => today,
    };
    if w.amount.is_some_and(|a| a <= 0) {
        return Err(Error::invalid(t(
            "nucleo-planejamento-previdencia-erro-resgate-positivo",
        )));
    }
    let plan = get_plan(conn, &w.asset_id)?;
    let (lots, last) = lots(conn, &w.asset_id, at)?;
    let price = price_at(conn, &w.asset_id, at)?
        .map(|(p, _)| p)
        .or(last)
        .unwrap_or(Decimal::ZERO);
    let total = cents(lots.iter().map(|l| l.qty * price).sum::<Decimal>());
    if let Some(a) = w.amount
        && a > total
    {
        return Err(Error::invalid(t_args(
            "nucleo-planejamento-previdencia-erro-resgate-saldo",
            // Plano PGBL/VGBL: previdência brasileira, em reais.
            &args(&[("saldo", i18n::money(i18n::current(), total, "BRL").into())]),
        )));
    }
    Ok(simulate(&lots, price, plan.as_ref(), w.amount, at))
}

/// Saldo informado de um plano numa data (o que a seguradora ou o fundo de pensão mostra):
/// vira a cota do dia (saldo ÷ cotas), para planos sem cota pública, como a previdência
/// fechada. O rendimento fica sendo o saldo menos os aportes. Devolve a cota gravada.
pub fn set_balance(conn: &Connection, asset_id: &str, date: &str, balance: i64) -> Result<String> {
    let asset = crate::assets::get(conn, asset_id)?;
    if asset.class != AssetClass::Pension {
        return Err(Error::invalid(t("nucleo-previdencia-saldo-so-previdencia")));
    }
    if balance <= 0 {
        return Err(Error::invalid(t("nucleo-previdencia-saldo-positivo")));
    }
    let day = parse_date(date)?;
    let book = crate::portfolio::engine::load_book(conn, asset_id, None)?;
    let qty = crate::portfolio::engine::replay(conn, &book, day)?.quantity();
    if qty <= Decimal::ZERO {
        return Err(Error::invalid(t_args(
            "nucleo-previdencia-saldo-sem-cotas",
            &args(&[(
                "data",
                crate::i18n::date(crate::i18n::current(), day).into(),
            )]),
        )));
    }
    let value = crate::portfolio::records::from_minor(balance, &asset.currency);
    let close = (value / qty).round_dp(10).normalize();
    crate::market::upsert_prices(
        conn,
        asset_id,
        &[crate::market::PriceRow {
            date: day,
            open: None,
            high: None,
            low: None,
            close,
            adj_close: None,
            volume: None,
        }],
        "manual",
    )?;
    Ok(close.to_string())
}
