//! Execução do plano numa transação: cria os ativos usados, grava operações, proventos e
//! eventos na ordem das datas e calcula os fatores dos eventos pelas quantidades do motor
//! da carteira. A prévia roda isto numa transação desfeita no fim (simulação); o commit,
//! dentro de `ctx.write_as`.
//!
//! Cada linha é validada antes de qualquer `INSERT` (as funções de `portfolio::records`
//! validam tudo antes de gravar e o id externo é conferido antes), então uma linha que
//! falha não deixa gravação pela metade: ela vira erro da linha e a importação segue.

use std::collections::{HashMap, HashSet};

use chrono::{Duration, NaiveDate};
use rusqlite::{Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use super::plan::{Item, Leg, Op, Plan};
use super::{B3AssetOverride, B3PositionCheck, B3PositionLine, B3PositionStatus, B3RowStatus};
use crate::assets::{self, AssetClass, AssetInput};
use crate::error::Result;
use crate::i18n::{self, args, t, t_args};
use crate::ids::Id;
use crate::portfolio::records::{self, from_minor};
use crate::portfolio::{
    CorporateActionInput, CorporateActionKind, IncomeInput, IncomeKind, IncomeStatus, TradeInput,
    TradeSide, engine,
};
use crate::time::fmt_date;

#[derive(Debug, Clone, Default)]
pub(crate) struct Approval {
    pub factor: Option<Decimal>,
    pub unit_cost: Option<Decimal>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ApplyOpts {
    pub include_possible: bool,
    pub skip: HashSet<u32>,
    pub approvals: HashMap<u32, Approval>,
    pub overrides: HashMap<String, B3AssetOverride>,
    pub cash: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ItemOutcome {
    pub written: bool,
    pub error: Option<String>,
    /// Evento que precisa de aprovação (fator não calculável ou não exato).
    pub review: Option<String>,
    pub held: Option<Decimal>,
    pub file_qty: Option<Decimal>,
    pub factor: Option<Decimal>,
    pub factor_note: Option<String>,
    /// Leilão de fração gravado como venda (quantidade, preço).
    pub auction_sale: Option<(Decimal, Decimal)>,
    /// Leilão de fração gravado como provento (não havia fração na posição).
    pub auction_income: bool,
    /// Venda recusada por ser maior que a posição na data (falta histórico anterior).
    pub oversold: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Outcome {
    pub items: HashMap<u32, ItemOutcome>,
    /// Id de cada ativo do plano depois da execução (existente ou criado).
    pub asset_ids: Vec<Option<Id>>,
    pub assets_created: u32,
    pub trades: u32,
    pub incomes: u32,
    pub actions: u32,
}

/// Quantidades com diferença menor que isto são iguais (frações do Tesouro, arredondamento
/// de planilha).
fn tolerance() -> Decimal {
    Decimal::new(1, 6)
}

fn eligible(it: &Item, opts: &ApplyOpts) -> bool {
    !opts.skip.contains(&it.index)
        && match it.status {
            B3RowStatus::New => true,
            B3RowStatus::PossibleDuplicate => opts.include_possible,
            B3RowStatus::Review => opts.approvals.contains_key(&it.index),
            _ => false,
        }
}

struct Runner<'a, 't> {
    tx: &'a Transaction<'t>,
    plan: &'a Plan,
    opts: &'a ApplyOpts,
    out: Outcome,
}

/// Executa o plano. Erros de banco interrompem (a transação inteira é desfeita); erros de
/// validação de uma linha ficam no resultado dela.
pub(crate) fn run(tx: &Transaction<'_>, plan: &Plan, opts: &ApplyOpts) -> Result<Outcome> {
    let mut order: Vec<&Item> = plan.items.iter().collect();
    order.sort_by_key(|it| (it.date, it.phase(), it.index));
    let mut r = Runner {
        tx,
        plan,
        opts,
        out: Outcome {
            asset_ids: plan.assets.iter().map(|a| a.existing.clone()).collect(),
            ..Default::default()
        },
    };
    for (i, a) in plan.assets.iter().enumerate() {
        if let Some(id) = opts.overrides.get(&a.key).and_then(|o| o.asset_id.clone()) {
            r.out.asset_ids[i] = Some(id);
        }
    }
    for it in order {
        if !eligible(it, opts) {
            continue;
        }
        let res = r.item(it);
        let mut o = r.out.items.remove(&it.index).unwrap_or_default();
        match res {
            Ok(written) => o.written = written,
            Err(crate::error::Error::Db(e)) => return Err(e.into()),
            Err(e) => {
                o.oversold = r.oversold(it);
                o.error = Some(r.hint(it, e.message(), o.oversold));
            }
        }
        if o.written {
            match &it.op {
                Op::Trade { .. } => r.out.trades += 1,
                Op::Income { .. } => r.out.incomes += 1,
                Op::Action { .. } => r.out.actions += 1,
                Op::Auction { .. } if o.auction_income => r.out.incomes += 1,
                Op::Auction { .. } => r.out.trades += 1,
            }
        }
        r.out.items.insert(it.index, o);
    }
    Ok(r.out)
}

impl Runner<'_, '_> {
    fn outcome(&mut self, index: u32) -> &mut ItemOutcome {
        self.out.items.entry(index).or_default()
    }

    /// Venda maior que a posição da conta na data (a que a carteira recusa).
    fn oversold(&self, it: &Item) -> bool {
        let Op::Trade {
            side: TradeSide::Sell,
            qty,
            ..
        } = &it.op
        else {
            return false;
        };
        let (Some(account), Some(Some(asset))) = (&it.account, self.out.asset_ids.get(it.asset))
        else {
            return false;
        };
        engine::quantity_held(self.tx, asset, Some(account), &fmt_date(it.date))
            .is_ok_and(|held| *qty > held)
    }

    /// Dica para vendas que falham depois de uma transferência de custódia ignorada.
    fn hint(&self, it: &Item, msg: String, oversold: bool) -> String {
        match self.plan.transfers.get(&it.asset) {
            Some(transfers) if oversold => {
                let (d, inst) = &transfers[0];
                t_args(
                    "nucleo-importacao-b3-dica-custodia-venda",
                    &args(&[
                        ("erro", msg.into()),
                        ("data", i18n::date(i18n::current(), *d).into()),
                        ("instituicao", inst.clone().into()),
                    ]),
                )
            }
            _ => msg,
        }
    }

    fn ensure_asset(&mut self, a: usize) -> Result<Id> {
        if let Some(id) = &self.out.asset_ids[a] {
            return Ok(id.clone());
        }
        let p = &self.plan.assets[a];
        // Pode ter sido cadastrado entre a prévia e o commit.
        if let Some(t) = &p.ticker
            && let Some(found) = assets::find_by_ticker(self.tx, t)?
        {
            self.out.asset_ids[a] = Some(found.id.clone());
            return Ok(found.id);
        }
        let ov = self.opts.overrides.get(&p.key);
        let mut class = ov.and_then(|o| o.class).unwrap_or(p.class);
        let name = ov
            .and_then(|o| o.name.clone())
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| p.name.clone());
        let mut fi = ov
            .and_then(|o| o.fixed_income.clone())
            .or_else(|| p.fixed_income.clone().filter(|t| !t.rate.trim().is_empty()));
        let mut notes = vec![t("nucleo-importacao-b3-ativo-cadastrado")];
        if class == AssetClass::FixedIncome && fi.is_none() {
            class = AssetClass::Other;
            notes.push(t("nucleo-importacao-b3-ativo-renda-fixa"));
        }
        if class != AssetClass::FixedIncome {
            fi = None;
        }
        let treasury = class == AssetClass::TreasuryBr;
        let input = AssetInput {
            ticker: p.ticker.clone(),
            name: name.clone(),
            official_name: None,
            account_id: None,
            class,
            market: None,
            currency: None,
            country: None,
            isin: None,
            cnpj: None,
            treasury_code: treasury.then(|| name.clone()),
            sector: None,
            segment: None,
            // Tesouro: PU do Tesouro Transparente pelo nome do título.
            price_provider: treasury.then(|| "tesouro".to_string()),
            provider_symbol: treasury.then(|| name.clone()),
            notes: Some(notes.join(" ")),
            fixed_income: fi,
            pension: None,
        };
        let id = assets::create(self.tx, &input)?;
        self.out.asset_ids[a] = Some(id.clone());
        self.out.assets_created += 1;
        Ok(id)
    }

    fn external_taken(&self, table: &str, account: &str, ext: &Option<String>) -> Result<bool> {
        let Some(e) = ext else { return Ok(false) };
        Ok(self.tx.query_row(
            &format!(
                "SELECT EXISTS (SELECT 1 FROM {table} WHERE account_id = ?1 AND external_id = ?2)"
            ),
            params![account, e],
            |r| r.get(0),
        )?)
    }

    fn item(&mut self, it: &Item) -> Result<bool> {
        match &it.op {
            Op::Trade {
                side,
                qty,
                price,
                label,
                ..
            } => {
                let account = it.account.clone().expect("linha com conta");
                if self.external_taken("trade", &account, &it.external_id)? {
                    return Err(crate::error::Error::conflict(t(
                        "nucleo-importacao-b3-erro-operacao-gravada",
                    )));
                }
                let asset_id = self.ensure_asset(it.asset)?;
                self.trade(
                    &asset_id,
                    &account,
                    it.date,
                    *side,
                    *qty,
                    *price,
                    label,
                    &it.external_id,
                )?;
                Ok(true)
            }
            Op::Income {
                kind,
                qty,
                gross,
                tax,
                ..
            } => {
                let account = it.account.clone().expect("linha com conta");
                if self.external_taken("income_event", &account, &it.external_id)? {
                    return Err(crate::error::Error::conflict(t(
                        "nucleo-importacao-b3-erro-provento-gravado",
                    )));
                }
                let asset_id = self.ensure_asset(it.asset)?;
                self.income(
                    &asset_id,
                    &account,
                    it.date,
                    *kind,
                    *qty,
                    *gross,
                    *tax,
                    &it.external_id,
                )?;
                Ok(true)
            }
            Op::Auction { value, .. } => {
                let account = it.account.clone().expect("linha com conta");
                if self.external_taken("trade", &account, &it.external_id)?
                    || self.external_taken("income_event", &account, &it.external_id)?
                {
                    return Err(crate::error::Error::conflict(t(
                        "nucleo-importacao-b3-erro-leilao-gravado",
                    )));
                }
                let asset_id = self.ensure_asset(it.asset)?;
                let held =
                    engine::quantity_held(self.tx, &asset_id, Some(&account), &fmt_date(it.date))?;
                let frac = held - held.trunc();
                if frac > Decimal::ZERO {
                    let price = (from_minor(*value, "BRL") / frac).round_dp(8);
                    self.trade(
                        &asset_id,
                        &account,
                        it.date,
                        TradeSide::Sell,
                        frac,
                        price,
                        &t("nucleo-importacao-b3-origem-leilao"),
                        &it.external_id,
                    )?;
                    self.outcome(it.index).auction_sale = Some((frac, price));
                } else {
                    self.income(
                        &asset_id,
                        &account,
                        it.date,
                        IncomeKind::Other,
                        None,
                        *value,
                        0,
                        &it.external_id,
                    )?;
                    self.outcome(it.index).auction_income = true;
                }
                Ok(true)
            }
            Op::Action {
                kind,
                legs,
                unit_cost,
                movement,
                ..
            } => self.action(it, *kind, legs, *unit_cost, movement),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn trade(
        &mut self,
        asset_id: &str,
        account: &str,
        date: NaiveDate,
        side: TradeSide,
        qty: Decimal,
        price: Decimal,
        label: &str,
        ext: &Option<String>,
    ) -> Result<Id> {
        records::create_trade(
            self.tx,
            &TradeInput {
                asset_id: asset_id.to_string(),
                account_id: account.to_string(),
                date: fmt_date(date),
                side,
                quantity: qty.normalize().to_string(),
                price: price.normalize().to_string(),
                fees: None,
                taxes: None,
                fx_rate: None,
                day_trade: None,
                note_number: None,
                notes: Some(t_args(
                    "nucleo-importacao-b3-nota-importado",
                    &args(&[("origem", label.to_string().into())]),
                )),
                cash_movement: Some(self.opts.cash),
                external_id: ext.clone(),
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn income(
        &mut self,
        asset_id: &str,
        account: &str,
        date: NaiveDate,
        kind: IncomeKind,
        qty: Option<Decimal>,
        gross: i64,
        tax: i64,
        ext: &Option<String>,
    ) -> Result<Id> {
        let id = records::create_income(
            self.tx,
            &IncomeInput {
                asset_id: asset_id.to_string(),
                account_id: account.to_string(),
                kind,
                ex_date: None,
                pay_date: fmt_date(date),
                quantity: qty.map(|q| q.normalize().to_string()),
                gross,
                tax_withheld: Some(tax),
                fx_rate: None,
                status: Some(IncomeStatus::Received),
                cash_movement: Some(self.opts.cash),
            },
        )?;
        self.tx.execute(
            "UPDATE income_event SET source = 'b3', external_id = ?2 WHERE id = ?1",
            params![id, ext],
        )?;
        Ok(id)
    }

    fn action(
        &mut self,
        it: &Item,
        kind: CorporateActionKind,
        legs: &[Leg],
        unit_cost: Option<Decimal>,
        movement: &str,
    ) -> Result<bool> {
        let asset_id = self.ensure_asset(it.asset)?;
        let approval = self.opts.approvals.get(&it.index).cloned();
        let derived = derive_factor(self.tx, &asset_id, kind, legs, it.date)?;
        let o = self.outcome(it.index);
        o.held = derived.held;
        o.file_qty = Some(derived.file_qty);
        o.factor = derived.factor;
        o.factor_note = derived.note.clone();
        let factor = match (approval.as_ref().and_then(|a| a.factor), derived.factor) {
            (Some(f), _) => f,
            (None, Some(f)) if derived.exact || approval.is_some() => f,
            (None, Some(_)) => {
                o.review = derived.note.clone();
                return Ok(false);
            }
            (None, None) if approval.is_some() => {
                return Err(crate::error::Error::invalid(t(
                    "nucleo-importacao-b3-erro-fator",
                )));
            }
            (None, None) => {
                o.review = derived.note.clone();
                return Ok(false);
            }
        };
        o.factor = Some(factor);
        let unit_cost = approval.and_then(|a| a.unit_cost).or(unit_cost);
        let id = records::create_action(
            self.tx,
            &CorporateActionInput {
                asset_id,
                date: fmt_date(it.date),
                kind,
                factor: Some(factor.normalize().to_string()),
                unit_cost: unit_cost.map(|u| u.normalize().to_string()),
                amount: None,
                new_asset_id: None,
                notes: Some(t_args(
                    "nucleo-importacao-b3-nota-importado",
                    &args(&[("origem", movement.to_string().into())]),
                )),
            },
        )?;
        self.tx.execute(
            "UPDATE corporate_action SET source = 'b3' WHERE id = ?1",
            [&id],
        )?;
        Ok(true)
    }
}

struct Derived {
    factor: Option<Decimal>,
    held: Option<Decimal>,
    file_qty: Decimal,
    /// Fator "redondo" (até 4 casas) que explica as quantidades de todas as contas.
    exact: bool,
    note: Option<String>,
}

/// Fator do evento pelas quantidades: posição antes do evento (por conta, pelo motor) e a
/// quantidade da movimentação.
///
/// - desdobro: crédito das ações novas → fator = (antes + creditado) ÷ antes;
/// - bonificação: crédito das ações bonificadas → fator = creditado ÷ antes (ações novas
///   por ação);
/// - grupamento: débito da quantidade baixada → fator = (antes − baixado) ÷ antes; crédito
///   menor que a posição = quantidade resultante → fator = resultante ÷ antes.
///
/// A B3 credita a parte inteira e trata a fração à parte (fração e leilão), então o fator
/// é "arredondado": a menor quantidade de casas (até 6) tal que, em toda conta,
/// antes × fator fica entre a quantidade do arquivo e ela + 1.
fn derive_factor(
    tx: &Transaction<'_>,
    asset_id: &str,
    kind: CorporateActionKind,
    legs: &[Leg],
    date: NaiveDate,
) -> Result<Derived> {
    let before = fmt_date(date - Duration::days(1));
    // Por conta: (antes, alvo).
    let mut per: Vec<(Id, Decimal, Decimal)> = vec![];
    let mut file_qty = Decimal::ZERO;
    for l in legs {
        let Some(acc) = &l.account else { continue };
        file_qty += l.qty;
        let target_delta = l.qty;
        match per.iter_mut().find(|(a, ..)| a == acc) {
            Some(e) => {
                e.2 += if l.credit {
                    target_delta
                } else {
                    -target_delta
                }
            }
            None => {
                let held = engine::quantity_held(tx, asset_id, Some(acc), &before)?;
                per.push((
                    acc.clone(),
                    held,
                    if l.credit {
                        target_delta
                    } else {
                        -target_delta
                    },
                ));
            }
        }
    }
    let held: Decimal = per.iter().map(|(_, h, _)| *h).sum();
    let mut out = Derived {
        factor: None,
        held: Some(held),
        file_qty,
        exact: false,
        note: None,
    };
    if held.is_zero() {
        out.note = Some(t("nucleo-importacao-b3-fator-sem-posicao"));
        return Ok(out);
    }
    if file_qty.is_zero() {
        out.note = Some(t("nucleo-importacao-b3-fator-sem-quantidade"));
        return Ok(out);
    }
    // Alvo por conta: quantidade que antes × fator deve alcançar.
    let targets: Vec<(Decimal, Decimal)> = per
        .iter()
        .map(|(_, h, delta)| {
            let t = match kind {
                CorporateActionKind::Split => *h + *delta,
                CorporateActionKind::Bonus => delta.abs(),
                // Débito: baixa (antes − baixado). Crédito: a quantidade resultante.
                _ if *delta < Decimal::ZERO => *h + *delta,
                _ => *delta,
            };
            (*h, t)
        })
        .filter(|(h, _)| !h.is_zero())
        .collect();
    let total_t: Decimal = targets.iter().map(|(_, t)| *t).sum();
    let total_h: Decimal = targets.iter().map(|(h, _)| *h).sum();
    let raw = total_t / total_h;
    let fits = |f: Decimal| {
        targets.iter().all(|(h, t)| {
            let n = *h * f;
            n - *t > -tolerance() && n - *t < Decimal::ONE
        })
    };
    let mut chosen = None;
    for dp in 0..=6u32 {
        let f = raw.round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero);
        if f > Decimal::ZERO && fits(f) {
            chosen = Some((f, dp));
            break;
        }
    }
    let valid = |f: Decimal| match kind {
        CorporateActionKind::Split => f > Decimal::ONE,
        CorporateActionKind::ReverseSplit => f > Decimal::ZERO && f < Decimal::ONE,
        _ => f > Decimal::ZERO,
    };
    match chosen {
        Some((f, dp)) if valid(f) => {
            out.factor = Some(f);
            out.exact = dp <= 4;
            out.note = Some(t(if out.exact {
                "nucleo-importacao-b3-fator-calculado"
            } else {
                "nucleo-importacao-b3-fator-inexato"
            }));
        }
        _ if valid(raw) => {
            out.factor = Some(raw.round_dp(10));
            out.note = Some(t_args(
                "nucleo-importacao-b3-fator-aproximado",
                &args(&[("fator", raw.round_dp(6).normalize().to_string().into())]),
            ));
        }
        _ => {
            out.note = Some(t_args(
                "nucleo-importacao-b3-fator-incompativel",
                &args(&[("fator", raw.round_dp(6).normalize().to_string().into())]),
            ));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Conferência com a posição da B3

/// Dica para divergências de um ativo que teve transferência de custódia ignorada.
fn transfer_hint(plan: &Plan, asset: Option<usize>) -> Option<String> {
    let (d, inst) = plan.transfers.get(&asset?)?.first()?;
    Some(t_args(
        "nucleo-importacao-b3-dica-custodia",
        &args(&[
            ("data", i18n::date(i18n::current(), *d).into()),
            ("instituicao", inst.clone().into()),
        ]),
    ))
}

fn with_hint(note: String, hint: Option<String>) -> String {
    match hint {
        Some(h) => format!("{note}; {h}"),
        None => note,
    }
}

fn classes_at_b3(c: AssetClass) -> bool {
    matches!(
        c,
        AssetClass::StockBr
            | AssetClass::Fii
            | AssetClass::EtfBr
            | AssetClass::Bdr
            | AssetClass::TreasuryBr
            | AssetClass::FixedIncome
    )
}

/// Compara a quantidade do motor (com o que já foi gravado nesta transação) com a da B3,
/// por ativo e instituição, na data da posição.
pub(crate) fn position_check(
    tx: &Transaction<'_>,
    plan: &Plan,
    out: &Outcome,
    date: NaiveDate,
) -> Result<B3PositionCheck> {
    let d = fmt_date(date);
    // Soma linhas repetidas do mesmo ativo e instituição (várias contas na corretora).
    struct Group {
        asset: usize,
        inst: String,
        account: Option<Id>,
        qty: Decimal,
        close: Option<Decimal>,
        value: Option<i64>,
        first: usize,
    }
    let mut groups: Vec<Group> = vec![];
    for (i, p) in plan.positions.iter().enumerate() {
        let inst = crate::import::text::fold(&p.rec.institution);
        let value = p.rec.value.map(|v| records::to_minor(v, "BRL"));
        match groups
            .iter_mut()
            .find(|g| g.asset == p.asset && g.inst == inst)
        {
            Some(g) => {
                g.qty += p.rec.qty;
                g.value = match (g.value, value) {
                    (Some(a), Some(b)) => Some(a + b),
                    (a, b) => a.or(b),
                };
            }
            None => groups.push(Group {
                asset: p.asset,
                inst,
                account: p.account.clone(),
                qty: p.rec.qty,
                close: p.rec.close,
                value,
                first: i,
            }),
        }
    }
    let mut lines = vec![];
    let mut covered: HashSet<(Id, Id)> = HashSet::new();
    let (mut matched, mut divergent) = (0u32, 0u32);
    for Group {
        asset,
        account,
        qty,
        close,
        value,
        first,
        ..
    } in groups
    {
        let a = &plan.assets[asset];
        let rec = &plan.positions[first].rec;
        let asset_id = out.asset_ids[asset].clone().or(a.existing.clone());
        let mut line = B3PositionLine {
            asset_key: a.key.clone(),
            ticker: a.ticker.clone(),
            name: a.name.clone(),
            institution: Some(rec.institution.clone()),
            account_id: account.clone(),
            account_name: account
                .as_ref()
                .and_then(|id| plan.account_names.get(id).cloned()),
            asset_id: asset_id.clone(),
            b3_quantity: Some(qty.normalize().to_string()),
            computed_quantity: None,
            difference: None,
            close_price: close.map(|c| c.normalize().to_string()),
            b3_value: value,
            status: B3PositionStatus::NoAccount,
            source: Some(rec.src.clone()),
            note: None,
        };
        if let Some(acc) = &account {
            let computed = match &asset_id {
                Some(id) => {
                    covered.insert((id.clone(), acc.clone()));
                    engine::quantity_held(tx, id, Some(acc), &d)?
                }
                None => Decimal::ZERO,
            };
            let diff = computed - qty;
            line.computed_quantity = Some(computed.normalize().to_string());
            line.difference = Some(diff.normalize().to_string());
            if diff.abs() < tolerance() {
                line.status = B3PositionStatus::Match;
                matched += 1;
            } else {
                line.status = B3PositionStatus::Divergent;
                divergent += 1;
                let note = t(if computed.is_zero() {
                    "nucleo-importacao-b3-posicao-sem-operacao"
                } else if diff > Decimal::ZERO {
                    "nucleo-importacao-b3-posicao-a-mais"
                } else {
                    "nucleo-importacao-b3-posicao-a-menos"
                });
                line.note = Some(with_hint(note, transfer_hint(plan, Some(asset))));
            }
        } else {
            line.note = Some(t("nucleo-importacao-b3-posicao-sem-conta"));
        }
        lines.push(line);
    }
    // Posições do app que a B3 não lista nas contas do relatório.
    let accounts: HashSet<Id> = plan
        .positions
        .iter()
        .filter_map(|p| p.account.clone())
        .collect();
    let mut accounts: Vec<Id> = accounts.into_iter().collect();
    accounts.sort();
    for acc in accounts {
        let mut stmt = tx.prepare(
            "SELECT DISTINCT t.asset_id FROM trade t WHERE t.account_id = ?1 ORDER BY t.asset_id",
        )?;
        let ids: Vec<Id> = stmt
            .query_map([&acc], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for id in ids {
            if covered.contains(&(id.clone(), acc.clone())) {
                continue;
            }
            let asset = assets::get(tx, &id)?;
            if !classes_at_b3(asset.class) {
                continue;
            }
            let q = engine::quantity_held(tx, &id, Some(&acc), &d)?;
            if q.abs() < tolerance() {
                continue;
            }
            divergent += 1;
            let plan_asset = out
                .asset_ids
                .iter()
                .position(|x| x.as_deref() == Some(id.as_str()));
            let note = with_hint(
                t("nucleo-importacao-b3-posicao-nao-listada"),
                transfer_hint(plan, plan_asset),
            );
            lines.push(B3PositionLine {
                asset_key: super::rules::asset_key(asset.ticker.as_deref(), &asset.name),
                ticker: asset.ticker.clone(),
                name: asset.name.clone(),
                institution: None,
                account_id: Some(acc.clone()),
                account_name: plan.account_names.get(&acc).cloned(),
                asset_id: Some(id),
                b3_quantity: None,
                computed_quantity: Some(q.normalize().to_string()),
                difference: Some(q.normalize().to_string()),
                close_price: None,
                b3_value: None,
                status: B3PositionStatus::MissingInB3,
                source: None,
                note: Some(note),
            });
        }
    }
    Ok(B3PositionCheck {
        date: d,
        lines,
        matched,
        divergent,
    })
}
