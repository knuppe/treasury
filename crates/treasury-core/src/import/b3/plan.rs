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
    B3AccountChoice, B3IgnoreRule, B3IgnoredRow, B3Institution, B3MatchReason, B3NewAccount,
    B3OpeningChoice, B3OpeningProposal, B3ReportKind, B3RowStatus, B3Source, B3TradeOrigin,
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
/// How far back an exercised subscription explains an "Atualização" credit.
const SUBSCRIPTION_WINDOW_DAYS: i64 = 60;
/// An income row this close with the same quantity shows the "Atualização" is a snapshot.
const SNAPSHOT_WINDOW_DAYS: i64 = 45;

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
    /// ISIN from the position report (Tesouro, listed assets).
    pub isin: Option<String>,
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
        /// The file has no price: the row stays in review until the cost is given.
        needs_cost: bool,
    },
    Income {
        kind: IncomeKind,
        qty: Option<Decimal>,
        gross: i64,
        tax: i64,
        net: i64,
        movement: String,
        /// Tesouro and taxable fixed income interest: the B3 reports the gross amount and
        /// the tax follows the regressive table by the age of the lots, computed on apply.
        regressive: bool,
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
    /// The id an older importer gave this row (institution name instead of the custodian group).
    pub legacy_external_id: Option<String>,
    pub status: B3RowStatus,
    pub duplicate_of: Option<Id>,
    pub note: Option<String>,
    /// Settlement date of a trade taken from the movements report (its `date` is estimated).
    pub settled: Option<NaiveDate>,
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
    pub openings: Vec<B3OpeningProposal>,
    /// Accounts the import creates: the placeholder id the preview used and what to create.
    pub new_accounts: Vec<(Id, B3NewAccount)>,
}

impl Plan {
    /// Replaces an account id everywhere in the plan (the account created by the commit
    /// takes the place of the preview's placeholder).
    pub(crate) fn remap_account(&mut self, from: &Id, to: &Id) {
        let fix = |x: &mut Option<Id>| {
            if x.as_ref() == Some(from) {
                *x = Some(to.clone());
            }
        };
        for it in &mut self.items {
            fix(&mut it.account);
            if let Op::Action { legs, .. } = &mut it.op {
                for l in legs {
                    fix(&mut l.account);
                }
            }
        }
        for p in &mut self.positions {
            fix(&mut p.account);
        }
        for i in &mut self.institutions {
            fix(&mut i.account_id);
            fix(&mut i.suggested_account_id);
        }
        if let Some(n) = self.account_names.remove(from) {
            self.account_names.insert(to.clone(), n);
        }
        for o in &mut self.openings {
            if o.account_id == *from {
                o.account_id = to.clone();
            }
        }
        for (id, _) in &mut self.new_accounts {
            if id == from {
                *id = to.clone();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Instituições

struct AccountRow {
    id: Id,
    name: String,
    kind: String,
    text: String,
    /// Group key of the account's institution (to match a custodian by a known alias).
    inst_group: Option<String>,
    inst_cnpj: Option<String>,
}

/// Account kinds that can hold assets (the same list the import dialog shows).
pub(crate) const CUSTODY_KINDS: &[&str] = &[
    "brokerage",
    "foreign",
    "checking",
    "savings",
    "retirement",
    "other",
];

fn load_accounts(conn: &Connection) -> Result<Vec<AccountRow>> {
    let kinds = CUSTODY_KINDS
        .iter()
        .map(|k| format!("'{k}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut stmt = conn.prepare(&format!(
        "SELECT a.id, a.name, a.kind, i.name, i.cnpj FROM account a LEFT JOIN institution i ON i.id = a.institution_id
          WHERE a.archived = 0 AND a.kind IN ({kinds}) ORDER BY a.sort_order, a.name",
    ))?;
    let rows = stmt
        .query_map([], |r| {
            let name: String = r.get(1)?;
            let inst: Option<String> = r.get(3)?;
            Ok(AccountRow {
                id: r.get(0)?,
                text: format!("{name} {}", inst.clone().unwrap_or_default()),
                name,
                kind: r.get(2)?,
                inst_group: inst.as_deref().map(rules::custodian_group),
                inst_cnpj: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Conta sugerida pelo nome: a de maior pontuação. Listed assets and Tesouro go only to a
/// brokerage account (a bank's checking account with a similar name is another company:
/// Nubank × Nu Invest); a bank's own fixed income (RDB, CDB) may go to any account. Nada se
/// houver empate.
fn suggest(tokens: &[String], accounts: &[AccountRow], listed: bool) -> Option<Id> {
    let mut scored: Vec<(u32, bool, &AccountRow)> = accounts
        .iter()
        .filter(|a| !listed || matches!(a.kind.as_str(), "brokerage" | "foreign"))
        .map(|a| {
            (
                rules::institution_score(tokens, &a.text),
                listed && a.kind == "brokerage",
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

/// Saved custodian → account mapping (`import_institution_map`), for a valid account.
fn saved_mapping(
    conn: &Connection,
    keys: &[String],
    accounts: &[AccountRow],
) -> Result<Option<Id>> {
    for k in keys {
        let id: Option<Id> = conn
            .query_row(
                "SELECT account_id FROM import_institution_map WHERE source = 'b3' AND key = ?1",
                [k],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(id) = id
            && accounts.iter().any(|a| a.id == id)
        {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

/// Keys a custodian is saved under: its group with each B3 account number, then the group.
pub(crate) fn mapping_keys(inst: &B3Institution) -> Vec<String> {
    let group = rules::custodian_group(&inst.name);
    let mut keys: Vec<String> = inst
        .b3_accounts
        .iter()
        .map(|a| format!("{group}#{a}"))
        .collect();
    keys.push(group);
    keys
}

struct Institutions {
    list: Vec<B3Institution>,
    /// Folded name of the file → custodian group.
    by_key: HashMap<String, usize>,
    by_group: HashMap<String, usize>,
    /// The group holds listed assets or Tesouro (prefers a brokerage account).
    listed: Vec<bool>,
}

impl Institutions {
    fn register(&mut self, name: &str, listed: bool, b3_account: Option<&str>) {
        let key = fold(name);
        let group = rules::custodian_group(name);
        let i = match self.by_group.get(&group) {
            Some(i) => *i,
            None => {
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
                    match_reason: None,
                    aliases: vec![],
                    b3_accounts: vec![],
                    mapped: false,
                    proposed_new_account: None,
                    new_account: None,
                    summary: Default::default(),
                });
                self.listed.push(false);
                self.by_group.insert(group, self.list.len() - 1);
                self.list.len() - 1
            }
        };
        self.by_key.insert(key, i);
        let item = &mut self.list[i];
        item.rows += 1;
        let trimmed = name.trim();
        if !trimmed.is_empty() && !item.aliases.iter().any(|a| fold(a) == fold(trimmed)) {
            item.aliases.push(trimmed.to_string());
        }
        if let Some(acc) = b3_account.map(str::trim).filter(|a| !a.is_empty())
            && !item.b3_accounts.iter().any(|a| a == acc)
        {
            item.b3_accounts.push(acc.to_string());
        }
        self.listed[i] |= listed;
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

/// Listed asset or Tesouro (vs. a bank's own fixed income).
fn is_listed_product(product: &str, code: Option<&str>) -> bool {
    let id = rules::identify(product, code);
    id.ticker.is_some() || id.key.starts_with("tesouro:")
}

/// The account the import offers to create for a custodian without a suggestion.
fn proposed_account(
    inst: &B3Institution,
    listed: bool,
    first_date: Option<NaiveDate>,
) -> B3NewAccount {
    use crate::accounts::AccountKind;
    use crate::household::InstitutionKind;
    let known = rules::known_custodian(&inst.name);
    let broker = known.is_some_and(|c| c.broker)
        || inst.aliases.iter().any(|a| rules::looks_like_broker(a))
        || (listed
            && !inst
                .aliases
                .iter()
                .any(|a| fold(a).contains("banco") || fold(a).contains("cooperativa")));
    let short = rules::custodian_short_name(&inst.name);
    B3NewAccount {
        name: if broker {
            short.clone()
        } else {
            t_args(
                "nucleo-importacao-b3-conta-investimentos",
                &args(&[("nome", short.clone().into())]),
            )
        },
        kind: if broker {
            AccountKind::Brokerage
        } else {
            AccountKind::Other
        },
        institution_name: known.map(|c| c.display.to_string()).unwrap_or(short),
        institution_kind: if broker {
            InstitutionKind::Broker
        } else {
            InstitutionKind::Bank
        },
        // The opening positions, when accepted, are dated the day before the file.
        opening_date: first_date.map(|d| fmt_date(d - Duration::days(1))),
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
        by_group: HashMap::new(),
        listed: vec![],
    };
    for r in &parsed.trades {
        inst.register(&r.institution, true, None);
    }
    for r in &parsed.moves {
        inst.register(&r.institution, is_listed_product(&r.product, None), None);
    }
    for r in &parsed.positions {
        inst.register(
            &r.institution,
            is_listed_product(&r.product, r.code.as_deref()),
            r.b3_account.as_deref(),
        );
    }
    let accounts = load_accounts(conn)?;
    for c in choices {
        if c.new_account.is_none()
            && let Some(id) = &c.account_id
            && !accounts.iter().any(|a| a.id == *id)
        {
            return Err(Error::not_found(crate::accounts::account_ref(id)));
        }
    }
    let first_date = parsed
        .trades
        .iter()
        .map(|t| t.date)
        .chain(parsed.moves.iter().map(|m| m.date))
        .min();
    for i in 0..inst.list.len() {
        let listed = inst.listed[i];
        let item = &mut inst.list[i];
        let group = rules::custodian_group(&item.name);
        let known = rules::known_custodian(&item.name);
        // 1. Saved mapping; 2. the account's institution is this custodian; 3. similar name.
        let saved = saved_mapping(conn, &mapping_keys(item), &accounts)?;
        let same: Vec<&AccountRow> = accounts
            .iter()
            .filter(|a| {
                a.inst_group.as_deref() == Some(group.as_str())
                    || known.and_then(|k| k.cnpj).is_some_and(|c| {
                        a.inst_cnpj.as_deref().is_some_and(|x| {
                            x.chars().filter(char::is_ascii_digit).collect::<String>() == c
                        })
                    })
            })
            .collect();
        let same_pick = match same.as_slice() {
            [one] => Some(one.id.clone()),
            many if !many.is_empty() => {
                let brokers: Vec<_> = many.iter().filter(|a| a.kind == "brokerage").collect();
                match brokers.as_slice() {
                    [one] if listed => Some(one.id.clone()),
                    _ => None,
                }
            }
            _ => None,
        };
        let (suggested, reason) = if let Some(id) = saved.clone() {
            (Some(id), Some(B3MatchReason::Saved))
        } else if let Some(id) = same_pick {
            (Some(id), Some(B3MatchReason::SameInstitution))
        } else {
            let mut tokens = rules::institution_tokens(&item.name);
            for a in &item.aliases {
                for tk in rules::institution_tokens(a) {
                    if !tokens.contains(&tk) {
                        tokens.push(tk);
                    }
                }
            }
            match suggest(&tokens, &accounts, listed) {
                Some(id) => (Some(id), Some(B3MatchReason::SimilarName)),
                None => (None, None),
            }
        };
        item.suggested_account_id = suggested.clone();
        item.mapped = saved.is_some();
        // A choice may name any of the group's names or the name shown in the preview.
        let names: Vec<String> = item
            .aliases
            .iter()
            .map(|a| fold(a))
            .chain([fold(&item.name)])
            .collect();
        match choices
            .iter()
            .find(|c| names.contains(&fold(&c.institution)))
        {
            Some(c) if c.new_account.is_some() => {
                item.new_account = c.new_account.clone();
                item.account_id = c.account_id.clone();
                item.match_reason = Some(B3MatchReason::Created);
            }
            Some(c) => {
                item.account_id = c.account_id.clone();
                item.match_reason = if c.account_id.is_some() && c.account_id == saved {
                    Some(B3MatchReason::Saved)
                } else {
                    c.account_id.as_ref().map(|_| B3MatchReason::Chosen)
                };
            }
            None => {
                item.account_id = suggested;
                item.from_suggestion = item.account_id.is_some();
                item.match_reason = reason;
            }
        }
        item.account_name = match &item.new_account {
            Some(n) => Some(n.name.clone()),
            None => item
                .account_id
                .as_ref()
                .and_then(|id| accounts.iter().find(|a| a.id == *id))
                .map(|a| a.name.clone()),
        };
        if item.account_id.is_none() && item.suggested_account_id.is_none() {
            item.proposed_new_account = Some(proposed_account(item, listed, first_date));
        }
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
    contract: Option<String>,
    /// Issuer of a contract: the position's `Emissor` column wins over the movement's suffix.
    issuer: Option<String>,
    /// Product text of a contract (for its type).
    product: Option<String>,
    isin: Option<String>,
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

/// Existing asset without a ticker for a product of the file: the same ISIN, the same
/// contract code in the name, the same Tesouro bond (type and year) or the same name.
fn find_product(
    conn: &Connection,
    key: &str,
    name: &str,
    contract: Option<&str>,
    isin: Option<&str>,
) -> Result<Option<Id>> {
    let want = fold(name);
    let mut stmt = conn.prepare(
        "SELECT id, name, coalesce(treasury_code, ''), coalesce(isin, '') FROM asset WHERE ticker IS NULL ORDER BY archived, created_at",
    )?;
    let rows: Vec<(Id, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    if let Some(isin) = isin.filter(|i| !i.is_empty())
        && let Some((id, ..)) = rows.iter().find(|r| r.3.eq_ignore_ascii_case(isin))
    {
        return Ok(Some(id.clone()));
    }
    if let Some(code) = contract {
        let code = fold(code);
        return Ok(rows
            .into_iter()
            .find(|r| fold(&r.1).split(' ').any(|w| w == code))
            .map(|r| r.0));
    }
    let treasury = key.starts_with("tesouro:");
    Ok(rows
        .into_iter()
        .find(|(_, n, t, _)| {
            fold(n) == want
                || (!t.is_empty() && fold(t) == want)
                || (treasury
                    && (rules::treasury_key(n).as_deref() == Some(key)
                        || rules::treasury_key(t).as_deref() == Some(key)))
        })
        .map(|(id, ..)| id))
}

fn resolve_assets(conn: &Connection, parsed: &Parsed) -> Result<Assets> {
    let mut info: Vec<(String, AssetInfo)> = vec![];
    let mut pos_of: HashMap<String, usize> = HashMap::new();
    let mut note = |id: rules::ProductId, product: &str| -> usize {
        let i = *pos_of.entry(id.key.clone()).or_insert_with(|| {
            info.push((id.key.clone(), AssetInfo::default()));
            info.len() - 1
        });
        let a = &mut info[i].1;
        if a.ticker.is_none() {
            a.ticker = id.ticker;
        }
        if a.contract.is_none() {
            a.contract = id.contract;
            a.product = Some(product.to_string());
        }
        if a.issuer.is_none() {
            a.issuer = id.issuer;
        }
        if a.name.is_none() && id.has_name {
            a.name = Some(id.name);
        }
        i
    };
    for r in &parsed.trades {
        if let Some(t) = rules::normalize_ticker(&r.code) {
            note(
                rules::ProductId {
                    key: t.clone(),
                    name: t.clone(),
                    ticker: Some(t),
                    has_name: false,
                    contract: None,
                    issuer: None,
                },
                &r.code,
            );
        }
    }
    for r in &parsed.moves {
        note(rules::identify(&r.product, None), &r.product);
    }
    let mut pos_assets = vec![];
    for r in &parsed.positions {
        let i = note(rules::identify(&r.product, r.code.as_deref()), &r.product);
        pos_assets.push((i, r.clone()));
    }
    for (i, r) in pos_assets {
        let a = &mut info[i].1;
        if a.sheet_class.is_none() {
            a.sheet_class = r.sheet_class;
        }
        if a.contract.is_some()
            && let Some(issuer) = r.issuer.clone()
        {
            a.issuer = Some(issuer);
        }
        if a.isin.is_none() {
            a.isin = r.isin.clone();
        }
        if a.position.is_none() {
            a.position = Some(r);
        }
    }
    let mut list = vec![];
    let mut by_key = HashMap::new();
    for (key, mut a) in info {
        if let Some(code) = &a.contract {
            let product = a.product.clone().unwrap_or_default();
            a.name = Some(rules::contract_name(&product, code, a.issuer.as_deref()));
        }
        let name = a
            .name
            .clone()
            .or_else(|| a.ticker.clone())
            .unwrap_or_else(|| key.clone());
        let existing = match &a.ticker {
            Some(t) => crate::assets::find_by_ticker(conn, t)?,
            None => {
                match find_product(conn, &key, &name, a.contract.as_deref(), a.isin.as_deref())? {
                    Some(id) => Some(crate::assets::get(conn, &id)?),
                    None => None,
                }
            }
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
            isin: a.isin.clone(),
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
        issue_date: pos.and_then(|p| p.issue_date).map(fmt_date),
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
    per_file: HashMap<(usize, String, String), u32>,
}

/// External id of a row, and the id an older version of the importer gave it (with the
/// institution's name instead of the custodian group), still checked for duplicates.
pub(crate) struct ExtId {
    pub id: String,
    pub legacy: String,
}

impl ExternalIds {
    /// `group`: the custodian group of the institution (the same for the names of a renamed
    /// broker, so the same row under both names is one row; stable when the account the
    /// custodian goes to changes, since duplicates are looked up per account). The counter
    /// that tells apart
    /// identical rows of one file (two equal trades on the same day) runs per name, so the
    /// copy of a row under the other name gets the same id and is marked as duplicate.
    fn make(
        &mut self,
        prefix: &str,
        file: usize,
        parts: &[&str],
        institution: &str,
        group: Option<&str>,
    ) -> ExtId {
        let inst = fold(institution);
        let with = |last: &str| {
            let mut p: Vec<&str> = parts.to_vec();
            p.push(last);
            format!("{prefix}:{}", short_hash(&p))
        };
        let legacy = with(&inst);
        let base = match group {
            Some(g) => with(g),
            None => legacy.clone(),
        };
        let n = self.per_file.entry((file, base.clone(), inst)).or_insert(0);
        *n += 1;
        let suffix = |b: &str| {
            if *n == 1 {
                b.to_string()
            } else {
                format!("{b}:{n}")
            }
        };
        ExtId {
            id: suffix(&base),
            legacy: suffix(&legacy),
        }
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
        external_id: Option<ExtId>,
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
            legacy_external_id: external_id
                .as_ref()
                .map(|e| e.legacy.clone())
                .filter(|l| Some(l) != external_id.as_ref().map(|e| &e.id)),
            external_id: external_id.map(|e| e.id),
            duplicate_of: None,
            note,
            settled: None,
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
            ],
            &r.institution,
            Some(&rules::custodian_group(&r.institution)),
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
                needs_cost: false,
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
        let id = rules::identify(&r.product, None);
        let (ticker, key) = (id.ticker, id.key);
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
            MoveRule::Update => self.move_update(r, &m, all),
            MoveRule::FractionAuction => {
                let Some(value) = r.value.filter(|v| !v.is_zero()) else {
                    self.skip_move(r, B3IgnoreRule::NoValue, &m.desc, None);
                    return false;
                };
                let ext = self.ext.make(
                    "b3:leilao",
                    r.file,
                    &[&fmt_date(r.date), &m.key, &dec_str(value)],
                    &r.institution,
                    Some(&rules::custodian_group(&r.institution)),
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
            ],
            &r.institution,
            Some(&rules::custodian_group(&r.institution)),
        );
        let op = Op::Trade {
            side,
            qty,
            price,
            origin,
            market: None,
            label,
            needs_cost: false,
        };
        self.push(&r.src, date, m.asset, &r.institution, op, Some(ext), note);
        if let Some(it) = self.items.last_mut() {
            it.settled = Some(r.date);
        }
        settlement
    }

    /// "Atualização": a snapshot of the custody (ignored) or the arrival of subscribed
    /// shares (a buy at the subscription price).
    fn move_update(&mut self, r: &MoveRec, m: &MoveCtx, all: &[MoveRec]) {
        let Some(qty) = r.qty.filter(|q| !q.is_zero() && r.credit) else {
            self.skip_move(r, B3IgnoreRule::BalanceUpdate, &m.desc, None);
            return;
        };
        let ticker = self.assets.list[m.asset].ticker.clone();
        let Some(root) = ticker.as_deref().and_then(rules::ticker_root) else {
            self.skip_move(r, B3IgnoreRule::BalanceUpdate, &m.desc, None);
            return;
        };
        // The same custodian, under any of its names (a renamed broker appears under two).
        let group = rules::custodian_group(&r.institution);
        let same_inst = |o: &MoveRec| rules::custodian_group(&o.institution) == group;
        let window = |o: &MoveRec| {
            o.date <= r.date && (r.date - o.date).num_days() <= SUBSCRIPTION_WINDOW_DAYS
        };
        // Exercised subscription of the same root, before this row.
        let mut subscribed: Vec<&MoveRec> = all
            .iter()
            .filter(|o| {
                same_inst(o)
                    && window(o)
                    && rules::exercised_subscription(&o.movement)
                    && rules::identify(&o.product, None)
                        .ticker
                        .as_deref()
                        .and_then(rules::ticker_root)
                        == Some(root)
            })
            .collect();
        // Evidence that the quantity is the position already held: the quantity of an
        // income row of the same asset, another snapshot with the same quantity, or the
        // position recorded in the vault.
        let same_asset =
            |o: &MoveRec| same_inst(o) && rules::identify(&o.product, None).key == m.key;
        let snapshot = all.iter().any(|o| {
            !std::ptr::eq(o, r)
                && same_asset(o)
                && o.qty == Some(qty)
                && match rules::move_rule(&o.movement) {
                    MoveRule::Update => true,
                    MoveRule::Income(_) => {
                        (o.date - r.date).num_days().abs() <= SNAPSHOT_WINDOW_DAYS
                    }
                    _ => false,
                }
        }) || self.held_in_vault(m.asset, &r.institution, r.date) == Some(qty);
        // A receipt or request is strong evidence; a right alone may still be unexercised,
        // so a quantity that repeats the position wins over it.
        let receipt = subscribed.iter().any(|o| {
            let f = fold(&o.movement);
            f.starts_with("recibo") || f.starts_with("solicitacao")
        });
        if snapshot && !receipt {
            self.skip_move(r, B3IgnoreRule::CustodySnapshot, &m.desc, None);
            return;
        }
        subscribed.sort_by_key(|o| std::cmp::Reverse(o.date));
        // The receipt brings the price; a right or request may not.
        let price = subscribed
            .iter()
            .filter_map(|o| o.unit_price.filter(|p| !p.is_zero()))
            .next();
        let (origin, label, note) = if subscribed.is_empty() {
            (
                B3TradeOrigin::CustodyUpdate,
                t("nucleo-importacao-b3-origem-atualizacao"),
                t_args(
                    "nucleo-importacao-b3-nota-atualizacao-confirmar",
                    &args(&[("quantidade", dec_str(qty).into())]),
                ),
            )
        } else {
            (
                B3TradeOrigin::Subscription,
                t("nucleo-importacao-b3-origem-subscricao"),
                t(if price.is_some() {
                    "nucleo-importacao-b3-nota-subscricao"
                } else {
                    "nucleo-importacao-b3-nota-subscricao-sem-custo"
                }),
            )
        };
        let needs_cost = price.is_none();
        let price = price.unwrap_or_default();
        let ext = self.ext.make(
            "b3:mov",
            r.file,
            &[&fmt_date(r.date), &m.key, "buy", &dec_str(qty), "update"],
            &r.institution,
            Some(&rules::custodian_group(&r.institution)),
        );
        let op = Op::Trade {
            side: TradeSide::Buy,
            qty,
            price,
            origin,
            market: None,
            label,
            needs_cost,
        };
        self.push(
            &r.src,
            r.date,
            m.asset,
            &r.institution,
            op,
            Some(ext),
            Some(note),
        );
        if needs_cost
            && let Some(it) = self.items.last_mut()
            && it.status == B3RowStatus::New
        {
            it.status = B3RowStatus::Review;
        }
    }

    /// Quantity of the asset already recorded in the vault for the institution's account.
    fn held_in_vault(&self, asset: usize, institution: &str, on: NaiveDate) -> Option<Decimal> {
        let id = self.assets.list[asset].existing.clone()?;
        let account = self.inst.account(institution)?;
        crate::portfolio::engine::quantity_held(self.conn, &id, Some(&account), &fmt_date(on)).ok()
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
                    && rules::identify(&o.product, None).key == m.key
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
        let a = &self.assets.list[m.asset];
        let exempt = rules::fixed_income_kind(&a.name).is_some_and(|k| k.default_ir_exempt())
            || a.fixed_income.as_ref().is_some_and(|t| t.ir_exempt);
        let regressive = kind == IncomeKind::Interest
            && matches!(m.class, AssetClass::TreasuryBr | AssetClass::FixedIncome)
            && !exempt;
        let kind_s = serde_json::to_value(kind)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        let ext = self.ext.make(
            "b3:prov",
            r.file,
            &[&fmt_date(r.date), &m.key, &kind_s, &dec_str(value)],
            &r.institution,
            Some(&rules::custodian_group(&r.institution)),
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
            regressive,
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
                legacy_external_id: None,
                status: if any_account {
                    B3RowStatus::New
                } else {
                    B3RowStatus::NoAccount
                },
                duplicate_of: None,
                note: None,
                settled: None,
            });
        }
    }
}

impl Builder<'_> {
    /// Opening positions the file suggests, per asset and account: the quantity of the
    /// earliest income row or custody snapshot, else the position report, minus what the
    /// file moves before it; at least what the file sells beyond its own purchases. Skipped
    /// when the vault already holds the asset in the account (the history is there).
    fn opening_suggestions(
        &self,
        parsed: &Parsed,
        positions: &[PlanPosition],
        date: NaiveDate,
        position_date: NaiveDate,
        chosen: &[B3OpeningChoice],
    ) -> Result<Vec<B3OpeningProposal>> {
        type Key = (usize, Id);
        let mut flows: HashMap<Key, Vec<(NaiveDate, Decimal)>> = HashMap::new();
        let mut order: Vec<Key> = vec![];
        let touch = |k: &Key, order: &mut Vec<Key>| {
            if !order.contains(k) {
                order.push(k.clone());
            }
        };
        for it in &self.items {
            match &it.op {
                Op::Trade {
                    side, qty, origin, ..
                } if *origin != B3TradeOrigin::Opening => {
                    let Some(acc) = &it.account else { continue };
                    let k = (it.asset, acc.clone());
                    touch(&k, &mut order);
                    let q = if *side == TradeSide::Buy { *qty } else { -*qty };
                    flows.entry(k).or_default().push((it.date, q));
                }
                Op::Action {
                    kind: CorporateActionKind::Bonus,
                    legs,
                    ..
                } => {
                    for l in legs.iter().filter(|l| l.credit) {
                        let Some(acc) = &l.account else { continue };
                        let k = (it.asset, acc.clone());
                        flows.entry(k).or_default().push((it.date, l.qty));
                    }
                }
                _ => {}
            }
        }
        // Earliest evidence of the quantity held: income rows and custody snapshots.
        let snapshots: HashSet<(String, u32)> = self
            .ignored
            .iter()
            .filter(|i| i.rule == B3IgnoreRule::CustodySnapshot)
            .map(|i| (i.source.filename.clone(), i.source.line))
            .collect();
        let mut evidence: HashMap<Key, (NaiveDate, Decimal, B3Source, String)> = HashMap::new();
        for r in &parsed.moves {
            let Some(q) = r.qty.filter(|q| !q.is_zero()) else {
                continue;
            };
            let what = match rules::move_rule(&r.movement) {
                MoveRule::Income(_) | MoveRule::ForwardedIncome(_) if r.credit => {
                    "nucleo-importacao-b3-abertura-provento"
                }
                MoveRule::Update if snapshots.contains(&(r.src.filename.clone(), r.src.line)) => {
                    "nucleo-importacao-b3-abertura-retrato"
                }
                _ => continue,
            };
            let Some(acc) = self.inst.account(&r.institution) else {
                continue;
            };
            let asset = self.assets.idx(&rules::identify(&r.product, None).key);
            let k = (asset, acc);
            touch(&k, &mut order);
            let e = evidence
                .entry(k)
                .or_insert((r.date, q, r.src.clone(), what.to_string()));
            if r.date < e.0 {
                *e = (r.date, q, r.src.clone(), what.to_string());
            }
        }
        let mut reported: HashMap<Key, (Decimal, B3Source)> = HashMap::new();
        for p in positions {
            let Some(acc) = &p.account else { continue };
            let k = (p.asset, acc.clone());
            touch(&k, &mut order);
            let e = reported
                .entry(k)
                .or_insert((Decimal::ZERO, p.rec.src.clone()));
            e.0 += p.rec.qty;
        }
        let mut out = vec![];
        for k in order {
            let (asset, acc) = &k;
            let a = &self.assets.list[*asset];
            if let Some(id) = &a.existing
                && crate::portfolio::engine::quantity_held(
                    self.conn,
                    id,
                    Some(acc),
                    &fmt_date(date),
                )
                .is_ok_and(|q| !q.is_zero())
            {
                continue;
            }
            let mut f = flows.get(&k).cloned().unwrap_or_default();
            // Same day: buys before sells, like the apply order.
            f.sort_by_key(|(d, q)| (*d, *q < Decimal::ZERO));
            let cum = |pred: &dyn Fn(NaiveDate) -> bool| -> Decimal {
                f.iter().filter(|(d, _)| pred(*d)).map(|(_, q)| *q).sum()
            };
            let mut running = Decimal::ZERO;
            let mut shortfall = Decimal::ZERO;
            for (_, q) in &f {
                running += *q;
                shortfall = shortfall.max(-running);
            }
            let (base, why, src) = match (evidence.get(&k), reported.get(&k)) {
                (Some((d, q, src, what)), _) => {
                    let base = *q - cum(&|x| x < *d);
                    let why = t_args(
                        what,
                        &args(&[
                            ("data", crate::i18n::date(crate::i18n::current(), *d).into()),
                            ("quantidade", dec_str(*q).into()),
                        ]),
                    );
                    (base, why, Some(src.clone()))
                }
                (None, Some((q, src))) => (
                    *q - cum(&|x| x <= position_date),
                    t_args(
                        "nucleo-importacao-b3-abertura-posicao",
                        &args(&[
                            (
                                "data",
                                crate::i18n::date(crate::i18n::current(), position_date).into(),
                            ),
                            ("quantidade", dec_str(*q).into()),
                        ]),
                    ),
                    Some(src.clone()),
                ),
                (None, None) => (
                    Decimal::ZERO,
                    t("nucleo-importacao-b3-abertura-vendas"),
                    None,
                ),
            };
            let (qty, why) = if shortfall > base {
                (shortfall, t("nucleo-importacao-b3-abertura-vendas"))
            } else {
                (base, why)
            };
            if qty <= Decimal::new(1, 6) {
                continue;
            }
            let institution = self
                .inst
                .list
                .iter()
                .find(|i| i.account_id.as_ref() == Some(acc))
                .map(|i| i.name.clone())
                .unwrap_or_default();
            out.push(B3OpeningProposal {
                asset_key: a.key.clone(),
                ticker: a.ticker.clone(),
                name: a.name.clone(),
                institution,
                account_id: acc.clone(),
                date: fmt_date(date),
                quantity: dec_str(qty),
                evidence: why,
                source: src,
                chosen: chosen
                    .iter()
                    .any(|c| c.asset_key == a.key && c.account_id == *acc),
            });
        }
        Ok(out)
    }

    /// The opening positions the user accepted become buys the day before the file.
    fn opening_items(&mut self, chosen: &[B3OpeningChoice], date: NaiveDate) -> Result<()> {
        for c in chosen {
            let Some(&asset) = self.assets.by_key.get(&c.asset_key) else {
                return Err(Error::invalid(t_args(
                    "nucleo-importacao-b3-erro-ativo-fora",
                    &args(&[("ativo", c.asset_key.clone().into())]),
                )));
            };
            let Some(inst) = self
                .inst
                .list
                .iter()
                .find(|i| i.account_id.as_ref() == Some(&c.account_id))
                .map(|i| i.name.clone())
            else {
                return Err(Error::not_found(crate::accounts::account_ref(
                    &c.account_id,
                )));
            };
            let qty: Decimal =
                crate::portfolio::records::dec(&c.quantity, "nucleo-carteira-campo-quantidade")?;
            let cost = c
                .unit_cost
                .as_deref()
                .filter(|u| !u.trim().is_empty())
                .map(|u| crate::portfolio::records::dec(u, "nucleo-carteira-campo-custo"))
                .transpose()?;
            if qty <= Decimal::ZERO {
                return Err(Error::invalid(t("nucleo-importacao-b3-sem-quantidade")));
            }
            self.assets.list[asset].used = true;
            let key = self.assets.list[asset].key.clone();
            let ext = format!(
                "b3:abertura:{}",
                short_hash(&[&key, &c.account_id, &fmt_date(date)])
            );
            self.items.push(Item {
                index: self.items.len() as u32,
                src: B3Source {
                    filename: String::new(),
                    sheet: String::new(),
                    line: 0,
                },
                date,
                asset,
                institution: inst,
                account: Some(c.account_id.clone()),
                op: Op::Trade {
                    side: TradeSide::Buy,
                    qty,
                    price: cost.unwrap_or_default(),
                    origin: B3TradeOrigin::Opening,
                    market: None,
                    label: t("nucleo-importacao-b3-origem-abertura"),
                    needs_cost: cost.is_none(),
                },
                external_id: Some(ext),
                legacy_external_id: None,
                status: if cost.is_some() {
                    B3RowStatus::New
                } else {
                    B3RowStatus::Review
                },
                duplicate_of: None,
                note: Some(t("nucleo-importacao-b3-nota-abertura")),
                settled: None,
            });
        }
        Ok(())
    }
}

/// Monta o plano. `choices` são as escolhas de conta do usuário.
pub(crate) fn build(
    conn: &Connection,
    parsed: &Parsed,
    choices: &[B3AccountChoice],
    openings: &[B3OpeningChoice],
    position_date: NaiveDate,
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
        let asset = b
            .assets
            .idx(&rules::identify(&r.product, r.code.as_deref()).key);
        positions.push(PlanPosition {
            rec: r.clone(),
            asset,
            account: b.inst.account(&r.institution),
        });
    }

    let dates: Vec<NaiveDate> = parsed
        .trades
        .iter()
        .map(|t| t.date)
        .chain(parsed.moves.iter().map(|m| m.date))
        .collect();
    let opening_date = dates.iter().min().map(|d| *d - Duration::days(1));
    let mut opening_list = vec![];
    if let Some(od) = opening_date {
        opening_list = b.opening_suggestions(parsed, &positions, od, position_date, openings)?;
        b.opening_items(openings, od)?;
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
        openings: opening_list,
        new_accounts: vec![],
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
    /// Recorded from a brokerage note (real trade date, price with fees).
    from_note: bool,
}

fn existing_trades(
    conn: &Connection,
    asset: &str,
    account: &str,
    side: TradeSide,
) -> Result<Vec<Existing>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, quantity, price, external_id, note_number IS NOT NULL FROM trade WHERE asset_id = ?1 AND account_id = ?2 AND side = ?3 ORDER BY date, created_at",
    )?;
    type Row = (Id, String, String, String, Option<String>, bool);
    let rows: Vec<Row> = stmt
        .query_map(params![asset, account, side_str(side)], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(|(id, d, q, p, e, from_note)| {
            Ok(Existing {
                id,
                date: parse_date(&d)?,
                qty: q.parse().unwrap_or_default(),
                price: p.parse().unwrap_or_default(),
                external_id: e,
                from_note,
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
                let legacy = it.legacy_external_id.clone().unwrap_or_else(|| ext.clone());
                let hit: Option<Id> = conn
                    .query_row(
                        "SELECT id FROM trade WHERE account_id = ?1 AND external_id IN (?2, ?3) LIMIT 1",
                        params![account, ext, legacy],
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
                // From a brokerage note: the note has the real date and the price with fees, so
                // it wins. A settlement matches a note dated up to 5 business days before it.
                let note_window = match (origin, it.settled) {
                    (B3TradeOrigin::Settlement, Some(liq)) => {
                        Some((sub_business_days(liq, 5), liq))
                    }
                    (B3TradeOrigin::Trades, _) => Some((it.date, it.date)),
                    _ => None,
                };
                if let Some((from, to)) = note_window
                    && let Some(e) = existing.iter().find(|e| {
                        e.from_note && !is_b3(&e.external_id, "b3:") && window(e, from, to)
                    })
                {
                    consumed.insert(e.id.clone());
                    it.status = B3RowStatus::Duplicate;
                    it.duplicate_of = Some(e.id.clone());
                    it.note = Some(t("nucleo-importacao-b3-nota-ja-nota-corretagem"));
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
                let legacy = it.legacy_external_id.clone().unwrap_or_else(|| ext.clone());
                let hit: Option<Id> = conn
                    .query_row(
                        "SELECT id FROM income_event WHERE account_id = ?1 AND external_id IN (?2, ?3)
                         UNION ALL SELECT id FROM trade WHERE account_id = ?1 AND external_id IN (?2, ?3) LIMIT 1",
                        params![account, ext, legacy],
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
                // Recorded from the official notice (income ledger): the same payment, with the
                // custodian's rounding of a cent and the payment a few days apart at most.
                let official: Vec<Id> = conn
                    .prepare_cached(
                        "SELECT id FROM income_event WHERE asset_id = ?1 AND account_id = ?2 AND source = 'official'
                           AND abs(julianday(pay_date) - julianday(?3)) <= 3 AND abs(net - ?4) <= 1
                         ORDER BY abs(julianday(pay_date) - julianday(?3)), abs(net - ?4)",
                    )?
                    .query_map(params![asset, account, fmt_date(it.date), net], |r| r.get(0))?
                    .collect::<rusqlite::Result<_>>()?;
                if let Some(id) = official.into_iter().find(|id| !consumed.contains(id)) {
                    consumed.insert(id.clone());
                    it.status = B3RowStatus::Duplicate;
                    it.duplicate_of = Some(id);
                    it.note = Some(t("nucleo-importacao-b3-nota-provento-oficial"));
                    continue;
                }
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
            Op::Action { kind, legs, .. } => {
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
                    continue;
                }
                // A bonus recorded before as a buy (by another importer or by hand): the same
                // quantity a few days apart, without a brokerage note. Recording both would
                // count the shares twice and the buy's price spoils the average cost.
                if *kind == CorporateActionKind::Bonus {
                    let credited: Decimal = legs.iter().filter(|l| l.credit).map(|l| l.qty).sum();
                    let accounts: Vec<&Id> =
                        legs.iter().filter_map(|l| l.account.as_ref()).collect();
                    let mut stmt = conn.prepare_cached(
                        "SELECT id, account_id, date, quantity FROM trade WHERE asset_id = ?1 AND side = 'buy'
                           AND note_number IS NULL AND abs(julianday(date) - julianday(?2)) <= ?3
                         ORDER BY abs(julianday(date) - julianday(?2))",
                    )?;
                    let rows: Vec<(Id, Id, String, String)> = stmt
                        .query_map(params![asset, fmt_date(it.date), EVENT_WINDOW_DAYS], |r| {
                            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                        })?
                        .collect::<rusqlite::Result<_>>()?;
                    if let Some((id, _, d, _)) = rows.into_iter().find(|(id, acc, _, q)| {
                        !consumed.contains(id)
                            && accounts.contains(&acc)
                            && q.parse::<Decimal>().is_ok_and(|q| q == credited)
                    }) {
                        consumed.insert(id.clone());
                        it.status = B3RowStatus::PossibleDuplicate;
                        it.duplicate_of = Some(id);
                        it.note = Some(t_args(
                            "nucleo-importacao-b3-nota-bonificacao-compra",
                            &args(&[("data", d.into())]),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
