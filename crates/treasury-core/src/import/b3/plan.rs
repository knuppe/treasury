//! Monta o plano da importação a partir dos registros lidos: instituições → contas, ativos
//! (existentes ou a criar, com a classe inferida), operações, proventos e eventos
//! propostos, duplicados e linhas ignoradas. O plano ainda não toca no banco; a simulação
//! e a gravação ficam em `apply`.
//!
//! Identificadores externos (estáveis entre reimportações do mesmo relatório):
//! - negociação: `b3:neg:` + hash(data, ticker, lado, quantidade, preço, instituição);
//! - liquidação/resgate: `b3:liq:`/`b3:mov:` + hash(data da movimentação, ativo, lado,
//!   quantidade, preço, instituição);
//! - provento: `b3:prov:` + hash(data, ativo, tipo, valor, instituição);
//! - leilão de fração: `b3:leilao:` + hash(data, ativo, valor, instituição).
//!
//! Linhas idênticas no mesmo arquivo (dois negócios iguais no mesmo dia) recebem `:2`,
//! `:3`…; a mesma linha em dois arquivos (períodos que se sobrepõem) é a mesma operação.

use std::collections::{HashMap, HashSet};

use chrono::{Duration, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};

use super::parse::{MoveRec, Parsed, PosRec, TradeRec};
use super::rules::{self, MarketRule, MoveRule};
use super::{
    B3AccountChoice, B3IgnoreRule, B3IgnoredRow, B3Institution, B3ReportKind, B3RowStatus,
    B3Source, B3TradeOrigin,
};
use crate::assets::{AssetClass, FixedIncomeTerms, Liquidity};
use crate::error::{Error, Result};
use crate::finance::calendar;
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::import::text::fold;
use crate::portfolio::records::{from_minor, to_minor};
use crate::portfolio::{CorporateActionKind, IncomeKind, TradeSide};
use crate::time::{fmt_date, parse_date};

/// Janela (dias) para juntar linhas do mesmo evento em instituições diferentes e para
/// achar um evento já cadastrado (ex.: desdobramento trazido pela coleta de cotações na
/// data "ex", enquanto a B3 mostra a data do crédito).
const EVENT_WINDOW_DAYS: i64 = 10;
/// Liquidação acontece até alguns dias depois do negócio (D+2 nas ações).
const SETTLEMENT_WINDOW_DAYS: i64 = 7;

#[derive(Debug, Clone)]
pub(crate) struct PlanAsset {
    pub key: String,
    pub ticker: Option<String>,
    pub name: String,
    pub class: AssetClass,
    pub class_reason: String,
    pub weak: bool,
    pub existing: Option<Id>,
    /// Termos parciais de renda fixa (sem a taxa, que a B3 não informa).
    pub fixed_income: Option<FixedIncomeTerms>,
    pub note: Option<String>,
    /// Usado por alguma linha proposta (só esses aparecem na prévia e podem ser criados).
    pub used: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Leg {
    pub account: Option<Id>,
    pub qty: Decimal,
    pub credit: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum Op {
    Trade {
        side: TradeSide,
        qty: Decimal,
        price: Decimal,
        origin: B3TradeOrigin,
        market: Option<String>,
        /// Texto para as observações da operação.
        label: String,
    },
    Income {
        kind: IncomeKind,
        qty: Option<Decimal>,
        gross: i64,
        tax: i64,
        net: i64,
        movement: String,
    },
    Action {
        kind: CorporateActionKind,
        legs: Vec<Leg>,
        unit_cost: Option<Decimal>,
        movement: String,
        sources: Vec<B3Source>,
    },
    /// Leilão de fração: venda da fração na posição (ou provento, se não houver fração).
    Auction { value: i64, movement: String },
}

#[derive(Debug, Clone)]
pub(crate) struct Item {
    pub index: u32,
    pub src: B3Source,
    pub date: NaiveDate,
    pub asset: usize,
    pub institution: String,
    pub account: Option<Id>,
    pub op: Op,
    pub external_id: Option<String>,
    pub status: B3RowStatus,
    pub duplicate_of: Option<Id>,
    pub note: Option<String>,
}

impl Item {
    /// Ordem de gravação no mesmo dia: eventos antes das operações (o motor aplica o evento
    /// na data "ex", antes dos negócios do dia), compras antes de vendas (para a venda não
    /// ficar maior que a posição por causa da ordem das linhas), proventos por último.
    pub fn phase(&self) -> u8 {
        match &self.op {
            Op::Action { .. } => 0,
            Op::Trade {
                side: TradeSide::Buy,
                ..
            } => 1,
            Op::Trade { .. } | Op::Auction { .. } => 2,
            Op::Income { .. } => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PlanPosition {
    pub rec: PosRec,
    pub asset: usize,
    pub account: Option<Id>,
}

#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub assets: Vec<PlanAsset>,
    pub items: Vec<Item>,
    pub ignored: Vec<B3IgnoredRow>,
    pub institutions: Vec<B3Institution>,
    pub positions: Vec<PlanPosition>,
    pub warnings: Vec<String>,
    pub uses_settlements: bool,
    pub period: (Option<NaiveDate>, Option<NaiveDate>),
    /// Transferências de custódia ignoradas por ativo (dica quando uma venda falha).
    pub transfers: HashMap<usize, Vec<(NaiveDate, String)>>,
    /// Contas das instituições do relatório de posição.
    pub account_names: HashMap<Id, String>,
}

// ---------------------------------------------------------------------------
// Instituições

struct AccountRow {
    id: Id,
    name: String,
    kind: String,
    text: String,
}

fn load_accounts(conn: &Connection) -> Result<Vec<AccountRow>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.kind, coalesce(i.name, '') FROM account a LEFT JOIN institution i ON i.id = a.institution_id
          WHERE a.archived = 0 AND a.kind <> 'credit_card' ORDER BY a.sort_order, a.name",
    )?;
    let rows = stmt
        .query_map([], |r| {
            let name: String = r.get(1)?;
            let inst: String = r.get(3)?;
            Ok(AccountRow {
                id: r.get(0)?,
                text: format!("{name} {inst}"),
                name,
                kind: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Conta sugerida pelo nome: a de maior pontuação, preferindo contas de corretora; nada
/// se houver empate.
fn suggest(tokens: &[String], accounts: &[AccountRow]) -> Option<Id> {
    let mut scored: Vec<(u32, bool, &AccountRow)> = accounts
        .iter()
        .map(|a| {
            (
                rules::institution_score(tokens, &a.text),
                a.kind == "brokerage",
                a,
            )
        })
        .filter(|(s, ..)| *s > 0)
        .collect();
    scored.sort_by_key(|a| std::cmp::Reverse((a.0, a.1)));
    match scored.as_slice() {
        [] => None,
        [one] => Some(one.2.id.clone()),
        [a, b, ..] if (a.0, a.1) != (b.0, b.1) => Some(a.2.id.clone()),
        _ => None,
    }
}

struct Institutions {
    list: Vec<B3Institution>,
    by_key: HashMap<String, usize>,
}

impl Institutions {
    fn register(&mut self, name: &str) {
        let key = fold(name);
        let i = *self.by_key.entry(key).or_insert_with(|| {
            self.list.push(B3Institution {
                name: if name.trim().is_empty() {
                    t("nucleo-importacao-b3-sem-instituicao")
                } else {
                    name.trim().to_string()
                },
                account_id: None,
                account_name: None,
                suggested_account_id: None,
                from_suggestion: false,
                rows: 0,
            });
            self.list.len() - 1
        });
        self.list[i].rows += 1;
    }

    fn account(&self, name: &str) -> Option<Id> {
        self.by_key
            .get(&fold(name))
            .and_then(|i| self.list[*i].account_id.clone())
    }

    fn display(&self, name: &str) -> String {
        self.by_key
            .get(&fold(name))
            .map(|i| self.list[*i].name.clone())
            .unwrap_or_else(|| name.to_string())
    }
}

fn resolve_institutions(
    conn: &Connection,
    parsed: &Parsed,
    choices: &[B3AccountChoice],
) -> Result<Institutions> {
    let mut inst = Institutions {
        list: vec![],
        by_key: HashMap::new(),
    };
    for r in &parsed.trades {
        inst.register(&r.institution);
    }
    for r in &parsed.moves {
        inst.register(&r.institution);
    }
    for r in &parsed.positions {
        inst.register(&r.institution);
    }
    let accounts = load_accounts(conn)?;
    for c in choices {
        if let Some(id) = &c.account_id
            && !accounts.iter().any(|a| a.id == *id)
        {
            return Err(Error::not_found(crate::accounts::account_ref(id)));
        }
    }
    let by_key = inst.by_key.clone();
    for (key, i) in by_key {
        let item = &mut inst.list[i];
        item.suggested_account_id = suggest(&rules::institution_tokens(&item.name), &accounts);
        // A escolha pode vir pelo nome do arquivo ou pelo nome exibido na prévia.
        let display = fold(&item.name);
        match choices
            .iter()
            .find(|c| fold(&c.institution) == key || fold(&c.institution) == display)
        {
            Some(c) => item.account_id = c.account_id.clone(),
            None => {
                item.account_id = item.suggested_account_id.clone();
                item.from_suggestion = item.account_id.is_some();
            }
        }
        item.account_name = item
            .account_id
            .as_ref()
            .and_then(|id| accounts.iter().find(|a| a.id == *id))
            .map(|a| a.name.clone());
    }
    Ok(inst)
}

// ---------------------------------------------------------------------------
// Ativos

#[derive(Default)]
struct AssetInfo {
    ticker: Option<String>,
    name: Option<String>,
    sheet_class: Option<AssetClass>,
    position: Option<PosRec>,
}

struct Assets {
    list: Vec<PlanAsset>,
    by_key: HashMap<String, usize>,
}

impl Assets {
    fn idx(&self, key: &str) -> usize {
        self.by_key[key]
    }
}

fn find_product(conn: &Connection, name: &str) -> Result<Option<Id>> {
    let want = fold(name);
    let mut stmt = conn.prepare(
        "SELECT id, name, coalesce(treasury_code, '') FROM asset WHERE ticker IS NULL ORDER BY archived, created_at",
    )?;
    let rows: Vec<(Id, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .find(|(_, n, t)| fold(n) == want || (!t.is_empty() && fold(t) == want))
        .map(|(id, ..)| id))
}

fn resolve_assets(conn: &Connection, parsed: &Parsed) -> Result<Assets> {
    let mut info: Vec<(String, AssetInfo)> = vec![];
    let mut pos_of: HashMap<String, usize> = HashMap::new();
    let mut note = |key: String, ticker: Option<String>, name: Option<String>| -> usize {
        let i = *pos_of.entry(key.clone()).or_insert_with(|| {
            info.push((key, AssetInfo::default()));
            info.len() - 1
        });
        let a = &mut info[i].1;
        if a.ticker.is_none() {
            a.ticker = ticker;
        }
        if a.name.is_none() {
            a.name = name;
        }
        i
    };
    for r in &parsed.trades {
        if let Some(t) = rules::normalize_ticker(&r.code) {
            note(t.clone(), Some(t), None);
        }
    }
    for r in &parsed.moves {
        let (t, n) = rules::split_product(&r.product);
        let has_name = t.as_deref() != Some(n.as_str());
        note(rules::asset_key(t.as_deref(), &n), t, has_name.then_some(n));
    }
    let mut pos_assets = vec![];
    for r in &parsed.positions {
        let (pt, n) = rules::split_product(&r.product);
        let t = r.code.as_deref().and_then(rules::normalize_ticker).or(pt);
        let has_name = t.as_deref() != Some(n.as_str());
        let i = note(rules::asset_key(t.as_deref(), &n), t, has_name.then_some(n));
        pos_assets.push((i, r.clone()));
    }
    for (i, r) in pos_assets {
        let a = &mut info[i].1;
        if a.sheet_class.is_none() {
            a.sheet_class = r.sheet_class;
        }
        if a.position.is_none() {
            a.position = Some(r);
        }
    }
    let mut list = vec![];
    let mut by_key = HashMap::new();
    for (key, a) in info {
        let name = a
            .name
            .clone()
            .or_else(|| a.ticker.clone())
            .unwrap_or_else(|| key.clone());
        let existing = match &a.ticker {
            Some(t) => crate::assets::find_by_ticker(conn, t)?,
            None => match find_product(conn, &name)? {
                Some(id) => Some(crate::assets::get(conn, &id)?),
                None => None,
            },
        };
        let mut p = PlanAsset {
            key: key.clone(),
            ticker: a.ticker.clone(),
            name: name.clone(),
            class: AssetClass::Other,
            class_reason: String::new(),
            weak: false,
            existing: None,
            fixed_income: None,
            note: None,
            used: false,
        };
        match existing {
            Some(e) => {
                p.class = e.class;
                p.class_reason = t("nucleo-importacao-b3-classe-ja-cadastrado");
                p.name = e.name.clone();
                p.existing = Some(e.id);
            }
            None => {
                let g =
                    rules::infer_class(a.ticker.as_deref(), &name, a.sheet_class, a.name.is_some());
                p.class = g.class;
                p.class_reason = g.reason;
                p.weak = g.weak;
                if p.class == AssetClass::FixedIncome {
                    p.fixed_income = partial_terms(&name, a.position.as_ref());
                    p.weak = true;
                    p.note = Some(t("nucleo-importacao-b3-renda-fixa-sem-taxa"));
                }
            }
        }
        by_key.insert(key, list.len());
        list.push(p);
    }
    Ok(Assets { list, by_key })
}

/// Termos de renda fixa que dá para tirar da posição (tipo, indexador, vencimento,
/// emissor). A taxa fica vazia.
fn partial_terms(name: &str, pos: Option<&PosRec>) -> Option<FixedIncomeTerms> {
    let kind = rules::fixed_income_kind(name)?;
    let indexer = pos
        .and_then(|p| p.indexer.as_deref())
        .and_then(rules::indexer)?;
    Some(FixedIncomeTerms {
        kind,
        indexer,
        rate: String::new(),
        issue_date: None,
        maturity_date: pos.and_then(|p| p.maturity).map(fmt_date),
        grace_until: None,
        liquidity: Liquidity::AtMaturity,
        ir_exempt: kind.default_ir_exempt(),
        issuer: pos.and_then(|p| p.issuer.clone()),
    })
}

// ---------------------------------------------------------------------------
// Identificadores externos

fn short_hash(parts: &[&str]) -> String {
    let h = Sha256::digest(parts.join("|").as_bytes());
    hex::encode(h)[..20].to_string()
}

#[derive(Default)]
struct ExternalIds {
    per_file: HashMap<(usize, String), u32>,
}

impl ExternalIds {
    fn make(&mut self, prefix: &str, file: usize, parts: &[&str]) -> String {
        let base = format!("{prefix}:{}", short_hash(parts));
        let n = self.per_file.entry((file, base.clone())).or_insert(0);
        *n += 1;
        if *n == 1 { base } else { format!("{base}:{n}") }
    }
}

fn dec_str(d: Decimal) -> String {
    d.normalize().to_string()
}

fn side_str(s: TradeSide) -> &'static str {
    match s {
        TradeSide::Buy => "buy",
        TradeSide::Sell => "sell",
    }
}

/// `n` dias úteis antes de `d` (calendário de feriados nacionais).
fn sub_business_days(mut d: NaiveDate, n: u32) -> NaiveDate {
    let mut left = n;
    while left > 0 {
        d -= Duration::days(1);
        if calendar::is_business_day(d) {
            left -= 1;
        }
    }
    d
}

/// Bruto e imposto retido a partir do líquido. JCP vem líquido da B3 (retido na fonte pela
/// alíquota da data, `crate::portfolio::records::jcp_rate`: 15% até 2025, 17,5% desde
/// 2026): bruto = líquido ÷ (1 − alíquota); os demais proventos não têm retenção. A data é a
/// do crédito na movimentação da B3.
pub(crate) fn gross_and_tax(kind: IncomeKind, net: i64, on: NaiveDate) -> (i64, i64) {
    if kind == IncomeKind::Jcp {
        let keep = Decimal::ONE - crate::portfolio::records::jcp_rate(on);
        let gross = to_minor(from_minor(net, "BRL") / keep, "BRL");
        (gross, gross - net)
    } else {
        (net, 0)
    }
}

// ---------------------------------------------------------------------------
// Montagem

/// Dados comuns de uma linha da movimentação.
struct MoveCtx {
    key: String,
    asset: usize,
    class: AssetClass,
    desc: String,
    /// Ativo listado (ação, FII, ETF, BDR): a liquidação duplica a negociação.
    listed: bool,
}

struct Builder<'a> {
    conn: &'a Connection,
    inst: Institutions,
    assets: Assets,
    items: Vec<Item>,
    ignored: Vec<B3IgnoredRow>,
    ext: ExternalIds,
    transfers: HashMap<usize, Vec<(NaiveDate, String)>>,
}

impl Builder<'_> {
    fn ignore_with(
        &mut self,
        src: &B3Source,
        report: B3ReportKind,
        rule: B3IgnoreRule,
        description: String,
        reason: String,
    ) {
        self.ignored.push(B3IgnoredRow {
            source: src.clone(),
            report,
            rule,
            description,
            reason,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        src: &B3Source,
        date: NaiveDate,
        asset: usize,
        institution: &str,
        op: Op,
        external_id: Option<String>,
        note: Option<String>,
    ) {
        self.assets.list[asset].used = true;
        let account = self.inst.account(institution);
        self.items.push(Item {
            index: self.items.len() as u32,
            src: src.clone(),
            date,
            asset,
            institution: self.inst.display(institution),
            status: if account.is_some() {
                B3RowStatus::New
            } else {
                B3RowStatus::NoAccount
            },
            account,
            op,
            external_id,
            duplicate_of: None,
            note,
        });
    }

    fn trade_rec(&mut self, r: &TradeRec) {
        let desc = format!(
            "{} · {} · {} · {}",
            r.market,
            r.code,
            dec_str(r.qty),
            dec_str(r.price)
        );
        if let MarketRule::Ignore(rule) = rules::market_rule(&r.market) {
            let reason = if rule == B3IgnoreRule::UnsupportedMarket {
                format!("{}: {}", rule.reason(), r.market)
            } else {
                rule.reason()
            };
            self.ignore_with(&r.src, B3ReportKind::Trades, rule, desc, reason);
            return;
        }
        let Some(ticker) = rules::normalize_ticker(&r.code) else {
            self.ignore_with(
                &r.src,
                B3ReportKind::Trades,
                B3IgnoreRule::Invalid,
                desc,
                t_args(
                    "nucleo-importacao-b3-codigo-desconhecido",
                    &args(&[("codigo", r.code.clone().into())]),
                ),
            );
            return;
        };
        let asset = self.assets.idx(&ticker);
        let ext = self.ext.make(
            "b3:neg",
            r.file,
            &[
                &fmt_date(r.date),
                &ticker,
                side_str(r.side),
                &dec_str(r.qty),
                &dec_str(r.price),
                &fold(&r.institution),
            ],
        );
        let code: String = r.code.split_whitespace().collect::<String>().to_uppercase();
        let fractional = code != ticker && code.ends_with('F');
        let label = if r.market.is_empty() {
            t("nucleo-importacao-b3-origem-negociacao")
        } else {
            t_args(
                "nucleo-importacao-b3-origem-negociacao-mercado",
                &args(&[("mercado", r.market.clone().into())]),
            )
        };
        self.push(
            &r.src,
            r.date,
            asset,
            &r.institution,
            Op::Trade {
                side: r.side,
                qty: r.qty,
                price: r.price,
                origin: B3TradeOrigin::Trades,
                market: Some(r.market.clone()).filter(|m| !m.is_empty()),
                label,
            },
            Some(ext),
            fractional.then(|| {
                t_args(
                    "nucleo-importacao-b3-fracionario",
                    &args(&[
                        ("codigo", code.clone().into()),
                        ("ticker", ticker.clone().into()),
                    ]),
                )
            }),
        );
    }

    fn skip_move(&mut self, r: &MoveRec, rule: B3IgnoreRule, desc: &str, reason: Option<&str>) {
        let reason = reason.map(str::to_string).unwrap_or_else(|| rule.reason());
        self.ignore_with(
            &r.src,
            B3ReportKind::Movements,
            rule,
            desc.to_string(),
            reason,
        );
    }

    /// Uma linha da movimentação. Devolve `true` se uma liquidação de ativo listado virou
    /// operação (relatório de negociação ausente nesse período).
    fn move_rec(
        &mut self,
        r: &MoveRec,
        coverage: Option<(NaiveDate, NaiveDate)>,
        all: &[MoveRec],
        actions: &mut Vec<(MoveRec, usize, CorporateActionKind)>,
    ) -> bool {
        let (ticker, name) = rules::split_product(&r.product);
        let key = rules::asset_key(ticker.as_deref(), &name);
        let asset = self.assets.idx(&key);
        let class = self.assets.list[asset].class;
        let m = MoveCtx {
            listed: ticker.is_some()
                && !matches!(class, AssetClass::TreasuryBr | AssetClass::FixedIncome),
            desc: format!(
                "{} · {} · {} · {}",
                t(if r.credit {
                    "nucleo-importacao-b3-credito"
                } else {
                    "nucleo-importacao-b3-debito"
                }),
                r.movement,
                r.product,
                r.qty.map(dec_str).unwrap_or_else(|| "-".into())
            ),
            key,
            asset,
            class,
        };
        match rules::move_rule(&r.movement) {
            MoveRule::Settlement => return self.move_trade(r, &m, false, coverage),
            MoveRule::Redemption => return self.move_trade(r, &m, true, coverage),
            MoveRule::Income(kind) => self.move_income(r, &m, kind, false, all),
            MoveRule::ForwardedIncome(kind) => self.move_income(r, &m, kind, true, all),
            MoveRule::Action(kind) => {
                self.assets.list[asset].used = true;
                actions.push((r.clone(), asset, kind));
            }
            MoveRule::Fraction => self.skip_move(r, B3IgnoreRule::Fraction, &m.desc, None),
            MoveRule::FractionAuction => {
                let Some(value) = r.value.filter(|v| !v.is_zero()) else {
                    self.skip_move(r, B3IgnoreRule::NoValue, &m.desc, None);
                    return false;
                };
                let ext = self.ext.make(
                    "b3:leilao",
                    r.file,
                    &[
                        &fmt_date(r.date),
                        &m.key,
                        &dec_str(value),
                        &fold(&r.institution),
                    ],
                );
                let op = Op::Auction {
                    value: to_minor(value, "BRL"),
                    movement: r.movement.clone(),
                };
                self.push(&r.src, r.date, asset, &r.institution, op, Some(ext), None);
            }
            MoveRule::Ignore(rule) => {
                if rule == B3IgnoreRule::CustodyTransfer {
                    let inst = self.inst.display(&r.institution);
                    self.transfers
                        .entry(asset)
                        .or_default()
                        .push((r.date, inst));
                }
                let reason = (rule == B3IgnoreRule::UnknownMovement)
                    .then(|| format!("{}: {}", rule.reason(), r.movement));
                self.skip_move(r, rule, &m.desc, reason.as_deref());
            }
        }
        false
    }

    /// Liquidação, compra/venda, resgate ou vencimento → operação.
    fn move_trade(
        &mut self,
        r: &MoveRec,
        m: &MoveCtx,
        redemption: bool,
        coverage: Option<(NaiveDate, NaiveDate)>,
    ) -> bool {
        let settlement = m.listed && !redemption;
        if settlement && coverage.is_some_and(|(a, b)| r.date >= a && r.date <= b) {
            self.skip_move(r, B3IgnoreRule::SettlementDuplicate, &m.desc, None);
            return false;
        }
        let Some(qty) = r.qty.filter(|q| !q.is_zero()) else {
            let why = t("nucleo-importacao-b3-sem-quantidade");
            self.skip_move(r, B3IgnoreRule::Invalid, &m.desc, Some(&why));
            return false;
        };
        let price = match (r.unit_price, r.value) {
            (Some(p), _) if !p.is_zero() => p,
            (_, Some(v)) => (v / qty).round_dp(8),
            _ => {
                let why = t("nucleo-importacao-b3-sem-preco-valor");
                self.skip_move(r, B3IgnoreRule::Invalid, &m.desc, Some(&why));
                return false;
            }
        };
        let side = if redemption || !r.credit {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };
        let liq = crate::i18n::date(crate::i18n::current(), r.date);
        let (date, origin, prefix, label, note) = if settlement {
            (
                sub_business_days(r.date, 2),
                B3TradeOrigin::Settlement,
                "b3:liq",
                t_args(
                    "nucleo-importacao-b3-origem-liquidacao",
                    &args(&[("data", liq.clone().into())]),
                ),
                Some(t_args(
                    "nucleo-importacao-b3-data-estimada",
                    &args(&[("data", liq.clone().into())]),
                )),
            )
        } else {
            (
                r.date,
                B3TradeOrigin::Movement,
                "b3:mov",
                t_args(
                    "nucleo-importacao-b3-origem-movimentacao",
                    &args(&[("movimentacao", r.movement.clone().into())]),
                ),
                None,
            )
        };
        let ext = self.ext.make(
            prefix,
            r.file,
            &[
                &fmt_date(r.date),
                &m.key,
                side_str(side),
                &dec_str(qty),
                &dec_str(price),
                &fold(&r.institution),
            ],
        );
        let op = Op::Trade {
            side,
            qty,
            price,
            origin,
            market: None,
            label,
        };
        self.push(&r.src, date, m.asset, &r.institution, op, Some(ext), note);
        settlement
    }

    /// Provento em dinheiro.
    fn move_income(
        &mut self,
        r: &MoveRec,
        m: &MoveCtx,
        kind: IncomeKind,
        forwarded: bool,
        all: &[MoveRec],
    ) {
        if !r.credit {
            self.skip_move(r, B3IgnoreRule::IncomeReversal, &m.desc, None);
            return;
        }
        let Some(value) = r.value.filter(|v| !v.is_zero()) else {
            self.skip_move(r, B3IgnoreRule::NoValue, &m.desc, None);
            return;
        };
        // Repasse ("- Transferido") com a linha principal no arquivo: já contado.
        let main_exists = || {
            all.iter().any(|o| {
                matches!(rules::move_rule(&o.movement), MoveRule::Income(_))
                    && o.date == r.date
                    && o.value == r.value
                    && fold(&o.institution) == fold(&r.institution)
                    && {
                        let (t, n) = rules::split_product(&o.product);
                        rules::asset_key(t.as_deref(), &n) == m.key
                    }
            })
        };
        if forwarded && main_exists() {
            self.skip_move(r, B3IgnoreRule::ForwardedIncome, &m.desc, None);
            return;
        }
        // "Rendimento" é o nome da B3 para o provento de FII; em outros ativos vira juros
        // (renda fixa/Tesouro) ou "outros".
        let kind = match (kind, m.class) {
            (IncomeKind::FiiIncome, AssetClass::Fii) => IncomeKind::FiiIncome,
            (IncomeKind::FiiIncome, AssetClass::FixedIncome | AssetClass::TreasuryBr) => {
                IncomeKind::Interest
            }
            (IncomeKind::FiiIncome, _) => IncomeKind::Other,
            (k, _) => k,
        };
        let net = to_minor(value, "BRL");
        let (gross, tax) = gross_and_tax(kind, net, r.date);
        let kind_s = serde_json::to_value(kind)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        let ext = self.ext.make(
            "b3:prov",
            r.file,
            &[
                &fmt_date(r.date),
                &m.key,
                &kind_s,
                &dec_str(value),
                &fold(&r.institution),
            ],
        );
        let mut notes: Vec<String> = vec![];
        if forwarded {
            notes.push(t("nucleo-importacao-b3-nota-repasse"));
        }
        if kind == IncomeKind::Jcp {
            notes.push(t("nucleo-importacao-b3-nota-jcp"));
        }
        let op = Op::Income {
            kind,
            qty: r.qty,
            gross,
            tax,
            net,
            movement: r.movement.clone(),
        };
        let note = (!notes.is_empty()).then(|| notes.join("; "));
        self.push(&r.src, r.date, m.asset, &r.institution, op, Some(ext), note);
    }

    /// Junta as linhas de desdobro/grupamento/bonificação do mesmo ativo e tipo (uma por
    /// instituição, em datas próximas) num só evento.
    fn action_groups(&mut self, mut rows: Vec<(MoveRec, usize, CorporateActionKind)>) {
        rows.sort_by_key(|a| (a.1, a.2 as u8, a.0.date));
        let mut groups: Vec<Vec<(MoveRec, usize, CorporateActionKind)>> = vec![];
        for r in rows {
            match groups.last_mut() {
                Some(g)
                    if g[0].1 == r.1
                        && g[0].2 == r.2
                        && (r.0.date - g[0].0.date).num_days() <= EVENT_WINDOW_DAYS =>
                {
                    g.push(r)
                }
                _ => groups.push(vec![r]),
            }
        }
        for g in groups {
            let (first, asset, kind) = (&g[0].0, g[0].1, g[0].2);
            let legs: Vec<Leg> = g
                .iter()
                .map(|(m, ..)| Leg {
                    account: self.inst.account(&m.institution),
                    qty: m.qty.unwrap_or_default(),
                    credit: m.credit,
                })
                .collect();
            let unit_cost = g
                .iter()
                .filter_map(|(m, ..)| m.unit_price)
                .find(|p| !p.is_zero());
            let any_account = legs.iter().any(|l| l.account.is_some());
            let institution = g
                .iter()
                .map(|(m, ..)| self.inst.display(&m.institution))
                .collect::<Vec<_>>()
                .join(", ");
            self.items.push(Item {
                index: self.items.len() as u32,
                src: first.src.clone(),
                date: first.date,
                asset,
                institution,
                account: None,
                op: Op::Action {
                    kind,
                    legs,
                    unit_cost,
                    movement: first.movement.clone(),
                    sources: g.iter().map(|(m, ..)| m.src.clone()).collect(),
                },
                external_id: None,
                status: if any_account {
                    B3RowStatus::New
                } else {
                    B3RowStatus::NoAccount
                },
                duplicate_of: None,
                note: None,
            });
        }
    }
}

/// Monta o plano. `choices` são as escolhas de conta do usuário.
pub(crate) fn build(
    conn: &Connection,
    parsed: &Parsed,
    choices: &[B3AccountChoice],
) -> Result<Plan> {
    let inst = resolve_institutions(conn, parsed, choices)?;
    let assets = resolve_assets(conn, parsed)?;
    let mut b = Builder {
        conn,
        inst,
        assets,
        items: vec![],
        ignored: vec![],
        ext: ExternalIds::default(),
        transfers: HashMap::new(),
    };
    for bad in &parsed.bad {
        b.ignore_with(
            &bad.src,
            bad.report,
            B3IgnoreRule::Invalid,
            bad.description.clone(),
            bad.message.clone(),
        );
    }
    // Período coberto pelo relatório de negociação (só mercado à vista): liquidações dentro
    // dele duplicariam os negócios e são ignoradas.
    let spot: Vec<NaiveDate> = parsed
        .trades
        .iter()
        .filter(|t| rules::market_rule(&t.market) == MarketRule::Spot)
        .map(|t| t.date)
        .collect();
    let coverage = match (spot.iter().min(), spot.iter().max()) {
        (Some(a), Some(z)) => Some((*a, *z + Duration::days(SETTLEMENT_WINDOW_DAYS))),
        _ => None,
    };
    for r in &parsed.trades {
        b.trade_rec(r);
    }
    let mut actions = vec![];
    let mut uses_settlements = false;
    for r in &parsed.moves {
        uses_settlements |= b.move_rec(r, coverage, &parsed.moves, &mut actions);
    }
    b.action_groups(actions);

    let mut positions = vec![];
    for r in &parsed.positions {
        let (pt, n) = rules::split_product(&r.product);
        let t = r.code.as_deref().and_then(rules::normalize_ticker).or(pt);
        let asset = b.assets.idx(&rules::asset_key(t.as_deref(), &n));
        positions.push(PlanPosition {
            rec: r.clone(),
            asset,
            account: b.inst.account(&r.institution),
        });
    }

    mark_duplicates(&mut b)?;

    let mut warnings = parsed.warnings.clone();
    if uses_settlements {
        warnings.push(t("nucleo-importacao-b3-aviso-liquidacoes"));
    }
    let without: Vec<String> = b
        .inst
        .list
        .iter()
        .filter(|i| i.account_id.is_none())
        .map(|i| i.name.clone())
        .collect();
    if !without.is_empty() {
        warnings.push(t_args(
            "nucleo-importacao-b3-aviso-sem-conta",
            &args(&[("instituicoes", without.join(", ").into())]),
        ));
    }
    let dates: Vec<NaiveDate> = parsed
        .trades
        .iter()
        .map(|t| t.date)
        .chain(parsed.moves.iter().map(|m| m.date))
        .collect();
    let account_names = b
        .inst
        .list
        .iter()
        .filter_map(|i| Some((i.account_id.clone()?, i.account_name.clone()?)))
        .collect();
    Ok(Plan {
        assets: b.assets.list,
        items: b.items,
        ignored: b.ignored,
        institutions: b.inst.list,
        positions,
        warnings,
        uses_settlements,
        period: (dates.iter().min().copied(), dates.iter().max().copied()),
        transfers: b.transfers,
        account_names,
    })
}

// ---------------------------------------------------------------------------
// Duplicados

struct Existing {
    id: Id,
    date: NaiveDate,
    qty: Decimal,
    price: Decimal,
    external_id: Option<String>,
}

fn existing_trades(
    conn: &Connection,
    asset: &str,
    account: &str,
    side: TradeSide,
) -> Result<Vec<Existing>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, quantity, price, external_id FROM trade WHERE asset_id = ?1 AND account_id = ?2 AND side = ?3 ORDER BY date, created_at",
    )?;
    let rows: Vec<(Id, String, String, String, Option<String>)> = stmt
        .query_map(params![asset, account, side_str(side)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(|(id, d, q, p, e)| {
            Ok(Existing {
                id,
                date: parse_date(&d)?,
                qty: q.parse().unwrap_or_default(),
                price: p.parse().unwrap_or_default(),
                external_id: e,
            })
        })
        .collect()
}

fn is_b3(ext: &Option<String>, prefix: &str) -> bool {
    ext.as_deref().is_some_and(|e| e.starts_with(prefix))
}

fn mark_duplicates(b: &mut Builder<'_>) -> Result<()> {
    let conn = b.conn;
    // 1. A mesma linha em dois arquivos desta importação.
    let mut seen: HashMap<String, String> = HashMap::new();
    for it in b.items.iter_mut() {
        if let Some(e) = &it.external_id {
            if let Some(first) = seen.get(e) {
                if it.status == B3RowStatus::New || it.status == B3RowStatus::NoAccount {
                    it.status = B3RowStatus::Duplicate;
                    it.note = Some(t_args(
                        "nucleo-importacao-b3-nota-repetida",
                        &args(&[("origem", first.clone().into())]),
                    ));
                }
            } else {
                seen.insert(
                    e.clone(),
                    t_args(
                        "nucleo-importacao-b3-arquivo-linha",
                        &args(&[
                            ("arquivo", it.src.filename.clone().into()),
                            ("linha", it.src.line.into()),
                        ]),
                    ),
                );
            }
        }
    }
    // 2. Já gravada antes, ou parecida com um lançamento feito à mão.
    let mut consumed: HashSet<Id> = HashSet::new();
    for it in b.items.iter_mut() {
        if it.status != B3RowStatus::New {
            continue;
        }
        let asset = b.assets.list[it.asset].existing.clone();
        match &it.op {
            Op::Trade {
                side,
                qty,
                price,
                origin,
                ..
            } => {
                let account = it.account.clone().expect("New tem conta");
                let ext = it.external_id.clone().unwrap_or_default();
                let hit: Option<Id> = conn
                    .query_row(
                        "SELECT id FROM trade WHERE account_id = ?1 AND external_id = ?2",
                        params![account, ext],
                        |r| r.get(0),
                    )
                    .optional()?;
                if let Some(id) = hit {
                    it.status = B3RowStatus::Duplicate;
                    it.duplicate_of = Some(id);
                    it.note = Some(t("nucleo-importacao-b3-nota-ja-importada"));
                    continue;
                }
                let Some(asset) = asset else { continue };
                let existing = existing_trades(conn, &asset, &account, *side)?;
                let window = |e: &Existing, from: NaiveDate, to: NaiveDate| {
                    e.date >= from && e.date <= to && e.qty == *qty && !consumed.contains(&e.id)
                };
                let (other_prefix, from, to) = match origin {
                    B3TradeOrigin::Trades => (
                        "b3:liq",
                        it.date,
                        it.date + Duration::days(SETTLEMENT_WINDOW_DAYS),
                    ),
                    B3TradeOrigin::Settlement => (
                        "b3:neg",
                        it.date - Duration::days(SETTLEMENT_WINDOW_DAYS),
                        it.date + Duration::days(SETTLEMENT_WINDOW_DAYS),
                    ),
                    _ => ("b3:none", it.date, it.date),
                };
                // A mesma operação vinda do outro relatório (negociação × liquidação).
                if let Some(e) = existing
                    .iter()
                    .find(|e| is_b3(&e.external_id, other_prefix) && window(e, from, to))
                {
                    consumed.insert(e.id.clone());
                    it.status = B3RowStatus::Duplicate;
                    it.duplicate_of = Some(e.id.clone());
                    it.note = Some(t(if other_prefix == "b3:liq" {
                        "nucleo-importacao-b3-nota-ja-liquidacao"
                    } else {
                        "nucleo-importacao-b3-nota-ja-negociacao"
                    }));
                    continue;
                }
                // Lançada à mão: mesma data, quantidade e preço (liquidação: preço livre e
                // data na janela, porque a data é estimada).
                let manual = existing.iter().find(|e| {
                    !is_b3(&e.external_id, "b3:")
                        && match origin {
                            B3TradeOrigin::Settlement => window(e, from, to),
                            _ => window(e, it.date, it.date) && e.price == *price,
                        }
                });
                if let Some(e) = manual {
                    consumed.insert(e.id.clone());
                    it.status = B3RowStatus::PossibleDuplicate;
                    it.duplicate_of = Some(e.id.clone());
                    it.note = Some(t("nucleo-importacao-b3-nota-operacao-manual"));
                }
            }
            Op::Income { net, .. } | Op::Auction { value: net, .. } => {
                let account = it.account.clone().expect("New tem conta");
                let ext = it.external_id.clone().unwrap_or_default();
                let hit: Option<Id> = conn
                    .query_row(
                        "SELECT id FROM income_event WHERE account_id = ?1 AND external_id = ?2
                         UNION ALL SELECT id FROM trade WHERE account_id = ?1 AND external_id = ?2 LIMIT 1",
                        params![account, ext],
                        |r| r.get(0),
                    )
                    .optional()?;
                if let Some(id) = hit {
                    it.status = B3RowStatus::Duplicate;
                    it.duplicate_of = Some(id);
                    it.note = Some(t("nucleo-importacao-nota-ja-importado"));
                    continue;
                }
                let Some(asset) = asset else { continue };
                let mut stmt = conn.prepare_cached(
                    "SELECT id FROM income_event WHERE asset_id = ?1 AND account_id = ?2 AND pay_date = ?3 AND net = ?4
                       AND (external_id IS NULL OR external_id NOT LIKE 'b3:%') ORDER BY created_at",
                )?;
                let ids: Vec<Id> = stmt
                    .query_map(params![asset, account, fmt_date(it.date), net], |r| {
                        r.get(0)
                    })?
                    .collect::<rusqlite::Result<_>>()?;
                if let Some(id) = ids.into_iter().find(|id| !consumed.contains(id)) {
                    consumed.insert(id.clone());
                    it.status = B3RowStatus::PossibleDuplicate;
                    it.duplicate_of = Some(id);
                    it.note = Some(t("nucleo-importacao-b3-nota-provento-manual"));
                }
            }
            Op::Action { kind, .. } => {
                let Some(asset) = asset else { continue };
                let kind_s = serde_json::to_value(kind)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                let mut stmt = conn.prepare_cached(
                    "SELECT id, date FROM corporate_action WHERE asset_id = ?1 AND kind = ?2",
                )?;
                let rows: Vec<(Id, String)> = stmt
                    .query_map(params![asset, kind_s], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<rusqlite::Result<_>>()?;
                if let Some((id, d)) = rows.into_iter().find(|(_, d)| {
                    parse_date(d).is_ok_and(|d| (d - it.date).num_days().abs() <= EVENT_WINDOW_DAYS)
                }) {
                    it.status = B3RowStatus::Duplicate;
                    it.duplicate_of = Some(id);
                    it.note = Some(t_args(
                        "nucleo-importacao-b3-nota-evento-cadastrado",
                        &args(&[("data", d.into())]),
                    ));
                }
            }
        }
    }
    Ok(())
}
